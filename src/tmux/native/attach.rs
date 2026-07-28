//! Interactive attach path: composite pane grids onto the client's tty.
//!
//! This is the "real tty emulation" milestone that closes the `attach-session`
//! gap in the capability matrix. Where the command-client path uses the imsg
//! file protocol (`MSG_WRITE_OPEN`/`WRITE`/`CLOSE`) to stream stdout/stderr
//! back, an attach client hands its tty fds to the server via `SCM_RIGHTS`
//! (`MSG_IDENTIFY_STDIN`/`STDOUT`) and the server drives that tty directly.
//!
//! The prototype implements a single-pane compositor:
//!
//! - On `attach-session -t <name>` the server resolves the target session,
//!   checks that the passed fds are ttys (otherwise it returns
//!   "open terminal failed: not a terminal", matching real tmux and making the
//!   matrix report `OK` for a recognized command even when the harness passes
//!   `/dev/null`), and enters the attach loop.
//! - The attach loop renders the session's active pane (libghostty-vt grid → VT
//!   sequences via `Terminal::dump_vt`) onto the client's tty fd, forwards
//!   keystrokes from the tty fd into the pane's pty master, and handles
//!   `MSG_RESIZE` / `MSG_DETACH` from the imsg control plane.
//! - Input handling forwards unbound bytes to the pane and resolves configured
//!   `prefix`/`prefix2`, root, prefix, and supported copy-mode bindings through
//!   the server's semantic key tables. The full copy-mode selection/search
//!   command set remains out of scope.
//!
//! Threading: the pane's pty reader thread (in `pane.rs`) continuously drains
//! the child's output into the `Terminal`. The attach loop polls for four
//! sources: tty input, imsg control messages, pane-output notifications, and
//! client-local status refresh deadlines. Grid changes are detected by diffing
//! the last rendered VT. No extra threads are spawned per attach; the loop is
//! single-threaded and both fds are non-blocking.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::integration::status::StatusHub;
use crate::tmux::codec::ImsgReader;
use crate::tmux::message::{Frame, Message, PROTOCOL_VERSION};
use crate::tmux::traits::{FrameReader, FrameWriter};

use super::cmd_send_keys::base64_encode;
use super::command;
use super::format;
use super::key::{basic_key_bytes, key_from_byte, parse_key_name, KeyBase, KeyCode, SpecialKey};
use super::latmon::LatMon;
use super::mouse::{self, MouseEvent, MouseInputState, MousePosition, MouseProtocol};
#[cfg(test)]
use super::mouse::{MouseButton, MouseEventKind};
use super::pane::{OutputSubscription, Pane, PaneInputStats};
use super::state::{
    copy_search_segments, copy_selection_segments, ClientAction, ClientKey, CopyState, MenuItem,
    MenuRequest, ModeBindingUpdate, ModeEdit, ModeKind, ModePrompt, ModeView, ModeViewKeyResult,
    OverlayRequest, PopupRequest, ServerState,
};
use super::status;
use super::term::{self, ResolvedTerm, TerminalCapabilities, TerminalIdentity};

#[cfg(test)]
const PREFIX: u8 = 0x02;

/// Internal capability needed by the event-driven attach loop. This stays
/// separate from the public `FrameReader` compatibility contract.
pub(crate) trait AttachFrameReader: FrameReader + AsRawFd {
    fn has_buffered_frame(&self) -> bool;
}

impl AttachFrameReader for ImsgReader {
    fn has_buffered_frame(&self) -> bool {
        ImsgReader::has_buffered_frame(self)
    }
}

/// Client-local `status-interval` deadline.
///
/// Reconfiguring to the same interval preserves the existing deadline, so an
/// unrelated status invalidation does not postpone the clock. A changed
/// interval (including status being turned on or off) restarts it like tmux's
/// `status_timer_start`.
#[derive(Debug)]
struct StatusTimer {
    interval: Option<Duration>,
    deadline: Option<Instant>,
}

impl StatusTimer {
    fn new(interval: Option<Duration>, now: Instant) -> Self {
        Self {
            interval,
            deadline: interval.map(|duration| status_deadline(now, duration)),
        }
    }

    fn configure(&mut self, interval: Option<Duration>, now: Instant) {
        if self.interval == interval {
            return;
        }
        self.interval = interval;
        self.deadline = interval.map(|duration| status_deadline(now, duration));
    }

    /// Milliseconds for `poll(2)`, rounded up so a sub-millisecond remainder
    /// cannot spin. `-1` retains the ordinary indefinite wait when disabled.
    fn poll_timeout(&self, now: Instant) -> i32 {
        let Some(deadline) = self.deadline else {
            return -1;
        };
        let remaining = deadline.saturating_duration_since(now);
        if remaining.is_zero() {
            return 0;
        }
        remaining
            .as_nanos()
            .saturating_add(999_999)
            .checked_div(1_000_000)
            .unwrap_or(u128::MAX)
            .min(i32::MAX as u128) as i32
    }

    /// Advance an expired repeating timer and report that status must be
    /// recomposed. Scheduling from `now` matches libevent's callback cadence
    /// and avoids a burst of catch-up redraws after a delayed compositor.
    fn take_expired(&mut self, now: Instant) -> bool {
        if !self.deadline.is_some_and(|deadline| now >= deadline) {
            return false;
        }
        self.deadline = self.interval.map(|duration| status_deadline(now, duration));
        true
    }
}

fn status_deadline(now: Instant, interval: Duration) -> Instant {
    now.checked_add(interval)
        .unwrap_or_else(|| now + Duration::from_secs(i32::MAX as u64))
}

fn deadline_poll_timeout(deadline: Option<Instant>, now: Instant) -> i32 {
    let Some(deadline) = deadline else {
        return -1;
    };
    let remaining = deadline.saturating_duration_since(now);
    if remaining.is_zero() {
        return 0;
    }
    remaining
        .as_nanos()
        .saturating_add(999_999)
        .checked_div(1_000_000)
        .unwrap_or(u128::MAX)
        .min(i32::MAX as u128) as i32
}

fn minimum_poll_timeout(left: i32, right: i32) -> i32 {
    match (left, right) {
        (-1, timeout) | (timeout, -1) => timeout,
        (left, right) => left.min(right),
    }
}

/// A destructive binding that tmux guards behind a `confirm-before` `(y/n)`
/// prompt. The attach loop shows the prompt and only runs the action once the
/// user answers `y`.
#[derive(Clone, Debug, PartialEq, Eq)]
enum ConfirmAction {
    /// `C-b x` → `confirm-before … kill-pane`.
    KillPane,
    /// `C-b &` → `confirm-before … kill-window`.
    KillWindow,
    Command(Vec<String>),
}

struct ActiveConfirm {
    prompt: String,
    action: ConfirmAction,
    confirm_key: u8,
    default_yes: bool,
    reply: Option<std::sync::mpsc::Sender<super::state::PromptCompletion>>,
}

/// What a resolved prefix binding tells the attach loop to do.
enum PrefixOutcome {
    /// Detach the client (`C-b d`), ending the attach loop.
    Detach,
    /// Send a literal prefix byte to the pane (`C-b C-b`, i.e. `send-prefix`).
    SendPrefix(Vec<u8>),
    /// Enter the attached client's copy-mode view, optionally paging up.
    CopyMode {
        page_up: bool,
        page_down: bool,
        slider: bool,
        mouse: Option<MouseEvent>,
        begin_selection: bool,
    },
    /// Raise a `confirm-before` prompt (`C-b x` / `C-b &`). The loop shows
    /// `prompt` in the status line and waits for the `y`/`n` answer before
    /// running `action`.
    Confirm {
        prompt: String,
        action: ConfirmAction,
    },
    Prompt {
        args: Vec<String>,
    },
    Message {
        text: String,
        duration: Duration,
    },
    ViewOutput(Vec<u8>),
    /// The binding ran (or was a no-op / unknown key). `changed` is true when it
    /// altered the window/pane layout, so the compositor must redraw the
    /// now-active pane.
    Handled {
        changed: bool,
    },
}

enum ActiveOverlay {
    Menu {
        request: MenuRequest,
        selected: usize,
        reply: Option<std::sync::mpsc::Sender<super::state::PromptCompletion>>,
    },
    Popup {
        request: Box<PopupRequest>,
        pane: Pane,
        exit_status: Option<i32>,
        reply: Option<std::sync::mpsc::Sender<super::state::PromptCompletion>>,
    },
    DisplayPanes {
        deadline: Instant,
        command: Vec<String>,
        accept_input: bool,
        reply: Option<std::sync::mpsc::Sender<super::state::PromptCompletion>>,
    },
}

impl ActiveOverlay {
    fn from_request(
        request: OverlayRequest,
        reply: Option<std::sync::mpsc::Sender<super::state::PromptCompletion>>,
        cols: u16,
        rows: u16,
    ) -> io::Result<Option<Self>> {
        Ok(match request {
            OverlayRequest::Clear => None,
            OverlayRequest::Menu(request) => {
                let selected = request.selected.min(request.items.len().saturating_sub(1));
                Some(Self::Menu {
                    request,
                    selected,
                    reply,
                })
            }
            OverlayRequest::DisplayPanes {
                duration_ms,
                command,
                accept_input,
            } => Some(Self::DisplayPanes {
                deadline: Instant::now()
                    .checked_add(Duration::from_millis(duration_ms))
                    .unwrap_or_else(Instant::now),
                command,
                accept_input,
                reply,
            }),
            OverlayRequest::Popup(request) => {
                let outer_width =
                    overlay_dimension(request.width.as_deref(), cols, 50).clamp(3, cols.max(3));
                let outer_height =
                    overlay_dimension(request.height.as_deref(), rows, 50).clamp(3, rows.max(3));
                let inner_width = outer_width.saturating_sub(if request.border { 2 } else { 0 });
                let inner_height = outer_height.saturating_sub(if request.border { 2 } else { 0 });
                let argv = if request.argv.is_empty() {
                    vec![std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())]
                } else if request.argv.len() == 1 {
                    vec![
                        "/bin/sh".to_string(),
                        "-c".to_string(),
                        request.argv[0].clone(),
                    ]
                } else {
                    request.argv.clone()
                };
                let refs = argv.iter().map(String::as_str).collect::<Vec<_>>();
                let pane = Pane::spawn_in(
                    &refs,
                    request.cwd.as_deref(),
                    inner_width.max(1),
                    inner_height.max(1),
                )?;
                Some(Self::Popup {
                    request: Box::new(request),
                    pane,
                    exit_status: None,
                    reply,
                })
            }
        })
    }

    fn complete(&mut self, result: command::CommandResult, inserted: bool) {
        let reply = match self {
            Self::Menu { reply, .. }
            | Self::Popup { reply, .. }
            | Self::DisplayPanes { reply, .. } => reply.take(),
        };
        if let Some(reply) = reply {
            let _ = reply.send(super::state::PromptCompletion {
                stdout: result.stdout,
                stderr: result.stderr,
                exit: result.exit,
                inserted,
            });
        }
    }

    fn poll_timeout(&self, now: Instant) -> i32 {
        match self {
            Self::DisplayPanes { deadline, .. } => deadline_poll_timeout(Some(*deadline), now),
            Self::Popup { .. } => 50,
            Self::Menu { .. } => -1,
        }
    }

    fn resize(&mut self, cols: u16, rows: u16) {
        if let Self::Popup { request, pane, .. } = self {
            let width = overlay_dimension(request.width.as_deref(), cols, 50)
                .max(3)
                .min(cols.max(3));
            let height = overlay_dimension(request.height.as_deref(), rows, 50)
                .max(3)
                .min(rows.max(3));
            let inset = u16::from(request.border) * 2;
            let _ = pane.resize(
                width.saturating_sub(inset).max(1),
                height.saturating_sub(inset).max(1),
            );
        }
    }
}

fn overlay_dimension(value: Option<&str>, available: u16, default_percent: u16) -> u16 {
    match value {
        Some(value) if value.ends_with('%') => value
            .trim_end_matches('%')
            .parse::<u32>()
            .ok()
            .map(|percent| (u32::from(available) * percent / 100) as u16)
            .unwrap_or(available),
        Some(value) => value.parse().unwrap_or(available),
        None => (u32::from(available) * u32::from(default_percent) / 100) as u16,
    }
}

struct CommandPrompt {
    args: Vec<String>,
    tail: Vec<String>,
    spec: command::CommandPromptSpec,
    page: usize,
    values: Vec<String>,
    labels: Vec<String>,
    initials: Vec<String>,
    input: Vec<char>,
    cursor: usize,
    last: String,
    yank: Option<String>,
    history_index: usize,
    vi_command: bool,
    quote_next: bool,
    completion: Option<PromptCompletionMenu>,
    action: CommandPromptAction,
    frozen_frame: Option<Vec<u8>>,
    external: Option<super::state::ActiveCommandPrompt>,
}

enum CommandPromptAction {
    Command,
    ModeCommand { item_target: String },
    ModeSearch { target: String },
    ModeFilter { target: String },
    ModeEdit { target: String, edit: ModeEdit },
}

struct PromptCompletionMenu {
    items: Vec<PromptCompletionItem>,
    selected: usize,
    start: usize,
    end: usize,
    replace_entire: bool,
}

struct PromptCompletionItem {
    label: String,
    replacement: String,
}

enum PromptCompletion {
    None,
    Replace(String),
    Menu {
        items: Vec<PromptCompletionItem>,
        replace_entire: bool,
    },
}

enum CommandPromptInput {
    Continue,
    Finish(command::CommandResult),
    Cancel,
}

impl CommandPrompt {
    fn new(
        args: Vec<String>,
        external: Option<super::state::ActiveCommandPrompt>,
        state: &Arc<Mutex<ServerState>>,
        hub: &StatusHub,
        context: &command::ClientContext,
    ) -> Result<Self, String> {
        let prompt_end = args.iter().position(|arg| arg == ";");
        let prompt_args = prompt_end.map_or_else(|| args.clone(), |end| args[..end].to_vec());
        let tail = prompt_end
            .and_then(|end| args.get(end + 1..))
            .unwrap_or_default()
            .to_vec();
        let spec = command::command_prompt_spec(&prompt_args)?;
        let agents = hub.snapshot().panes;
        let labels = spec
            .pages
            .iter()
            .map(|page| command::expand_command_prompt_format(&page.label, state, &agents, context))
            .collect::<Vec<_>>();
        let initials = spec
            .pages
            .iter()
            .map(|page| {
                command::expand_command_prompt_format(&page.initial, state, &agents, context)
            })
            .collect::<Vec<_>>();
        let last = initials.first().cloned().unwrap_or_default();
        let initial = if spec.incremental {
            String::new()
        } else {
            last.clone()
        };
        let cursor = initial.chars().count();
        Ok(Self {
            args: prompt_args,
            tail,
            spec,
            page: 0,
            values: Vec::new(),
            labels,
            initials,
            input: initial.chars().collect(),
            cursor,
            last,
            yank: None,
            history_index: 0,
            vi_command: false,
            quote_next: false,
            completion: None,
            action: CommandPromptAction::Command,
            frozen_frame: None,
            external,
        })
    }

    fn for_mode(
        request: ModePrompt,
        target: &str,
        state: &Arc<Mutex<ServerState>>,
        hub: &StatusHub,
        context: &command::ClientContext,
    ) -> Result<Self, String> {
        let (args, action) = match request {
            ModePrompt::Search => (
                vec![
                    "command-prompt".to_string(),
                    "-T".to_string(),
                    "search".to_string(),
                    "-p".to_string(),
                    "(search)".to_string(),
                ],
                CommandPromptAction::ModeSearch {
                    target: target.to_string(),
                },
            ),
            ModePrompt::Filter { initial } => (
                vec![
                    "command-prompt".to_string(),
                    "-T".to_string(),
                    "search".to_string(),
                    "-I".to_string(),
                    initial,
                    "-p".to_string(),
                    "(filter)".to_string(),
                ],
                CommandPromptAction::ModeFilter {
                    target: target.to_string(),
                },
            ),
            ModePrompt::Command { item_target } => (
                vec![
                    "command-prompt".to_string(),
                    "-p".to_string(),
                    "(current)".to_string(),
                ],
                CommandPromptAction::ModeCommand { item_target },
            ),
            ModePrompt::Edit(edit) => (
                vec![
                    "command-prompt".to_string(),
                    "-I".to_string(),
                    edit.initial().to_string(),
                    "-p".to_string(),
                    edit.prompt(),
                ],
                CommandPromptAction::ModeEdit {
                    target: target.to_string(),
                    edit,
                },
            ),
        };
        let mut prompt = Self::new(args, None, state, hub, context)?;
        prompt.action = action;
        Ok(prompt)
    }

    fn label(&self) -> &str {
        self.labels
            .get(self.page)
            .map(String::as_str)
            .unwrap_or(":")
    }

    fn input(&self) -> String {
        self.input.iter().collect()
    }

    fn display(&self) -> String {
        format!("{}{}", self.label(), self.input())
    }

    fn display_cursor(&self) -> usize {
        self.label().chars().count() + self.cursor
    }

    fn formatted_display(&self, state: &ServerState, target: &str, cols: usize) -> (String, usize) {
        let input = self.input();
        let prefix = if let Some(message_format) = state.option_for_target(target, "message-format")
        {
            let mut vars = format::Vars::new();
            vars.set("message", self.label().to_string())
                .set("prompt-input", input.clone())
                .set("command_prompt", if self.vi_command { "1" } else { "0" });
            format::expand(message_format, &vars)
        } else {
            self.label().to_string()
        };
        let prefix = clip_prompt_display(&prefix, 0, cols);
        let prefix_width = format::display_width(&prefix);
        let available = cols.saturating_sub(prefix_width);
        let mut rendered_input = render_prompt_input(&self.input[..self.cursor]);
        rendered_input.push_str(&render_prompt_input(&self.input[self.cursor..]));
        let cursor_width = prompt_input_width(&self.input[..self.cursor]);
        let offset = if cursor_width >= available && available != 0 {
            cursor_width - available + 1
        } else {
            0
        };
        let visible_input = clip_prompt_display(&rendered_input, offset, available);
        let cursor = prefix_width + cursor_width.saturating_sub(offset);
        (format!("{prefix}{visible_input}"), cursor.min(cols))
    }

    fn run(
        &self,
        values: &[String],
        state: &Arc<Mutex<ServerState>>,
        hub: &StatusHub,
        context: &command::ClientContext,
    ) -> command::CommandResult {
        if let Some(value) = values.last() {
            match &self.action {
                CommandPromptAction::ModeSearch { target } => {
                    return match state
                        .lock()
                        .map_err(|_| io::Error::other("server state poisoned"))
                        .and_then(|mut state| state.mode_view_search(target, value))
                    {
                        Ok(()) => command::CommandResult::ok(""),
                        Err(error) => command::CommandResult::err(format!("{error}\n")),
                    };
                }
                CommandPromptAction::ModeFilter { target } => {
                    return match state
                        .lock()
                        .map_err(|_| io::Error::other("server state poisoned"))
                        .and_then(|mut state| state.mode_view_filter(target, value))
                    {
                        Ok(()) => command::CommandResult::ok(""),
                        Err(error) => command::CommandResult::err(format!("{error}\n")),
                    };
                }
                CommandPromptAction::ModeEdit { target, edit } => {
                    return run_mode_edit(edit, value, target, state, hub, context);
                }
                CommandPromptAction::ModeCommand { item_target } => {
                    return run_mode_command(value, item_target, state, hub, context);
                }
                CommandPromptAction::Command => {}
            }
        }
        let mut result = command::run_command_prompt_template(
            &self.args,
            values,
            state,
            &hub.snapshot().panes,
            context,
        );
        if result.exit == 0 && !self.tail.is_empty() {
            let tail = command::run_with_context(&self.tail, state, &hub.snapshot().panes, context);
            result.append_stdout(&tail);
            result.stderr.push_str(&tail.stderr);
            result.exit = tail.exit;
        }
        result
    }

    fn complete(
        &mut self,
        result: &command::CommandResult,
        state: &Arc<Mutex<ServerState>>,
        context: &command::ClientContext,
    ) {
        if let Some(external) = self.external.take() {
            external.complete(super::state::PromptCompletion {
                stdout: result.stdout.clone(),
                stderr: result.stderr.clone(),
                exit: result.exit,
                inserted: true,
            });
        } else if !result.stdout_data().is_empty() {
            if let Some(session_id) = context.current_session_id {
                append_view_output(state, &format!("${session_id}"), result.stdout_data());
            }
        }
    }

    fn cancel_external(&mut self) {
        if let Some(external) = self.external.take() {
            external.cancel();
        }
    }

    fn initial_incremental(
        &self,
        state: &Arc<Mutex<ServerState>>,
        hub: &StatusHub,
        context: &command::ClientContext,
    ) {
        if self.spec.incremental {
            let mut values = self.values.clone();
            values.push("=".to_string());
            let _ = self.run(&values, state, hub, context);
        }
    }

    fn changed(
        &mut self,
        prefix: char,
        state: &Arc<Mutex<ServerState>>,
        hub: &StatusHub,
        context: &command::ClientContext,
    ) {
        if self.spec.incremental {
            let mut values = self.values.clone();
            values.push(format!("{prefix}{}", self.input()));
            let _ = self.run(&values, state, hub, context);
        }
    }

    fn finish_page(
        &mut self,
        state: &Arc<Mutex<ServerState>>,
        hub: &StatusHub,
        context: &command::ClientContext,
    ) -> CommandPromptInput {
        let input = self.input();
        if !input.is_empty() {
            if let Ok(mut st) = state.lock() {
                st.add_prompt_history(&self.spec.prompt_type, &input);
            }
        }
        if self.spec.incremental {
            return CommandPromptInput::Cancel;
        }
        self.values.push(input);
        self.page += 1;
        if self.page < self.spec.pages.len() {
            let initial = self.initials.get(self.page).cloned().unwrap_or_default();
            self.last = initial.clone();
            self.input = initial.chars().collect();
            self.cursor = self.input.len();
            self.history_index = 0;
            self.vi_command = false;
            self.quote_next = false;
            self.completion = None;
            return CommandPromptInput::Continue;
        }
        CommandPromptInput::Finish(self.run(&self.values, state, hub, context))
    }

    fn delete_previous_word(&mut self, separators: &str) {
        if self.cursor == 0 {
            self.yank = Some(String::new());
            return;
        }
        let class = |character: char| {
            if character == ' ' {
                0
            } else if separators.contains(character) {
                1
            } else {
                2
            }
        };
        let mut start = self.cursor;
        while start > 0 && self.input[start - 1] == ' ' {
            start -= 1;
        }
        if start > 0 {
            let wanted = class(self.input[start - 1]);
            while start > 0 && class(self.input[start - 1]) == wanted {
                start -= 1;
            }
        }
        self.yank = Some(self.input[start..self.cursor].iter().collect());
        self.input.drain(start..self.cursor);
        self.cursor = start;
    }

    fn move_word_forward(&mut self, vi: bool, separators: &str) {
        let size = self.input.len();
        let mut index = self.cursor;
        if !vi {
            while index != size && self.input[index] == ' ' {
                index += 1;
            }
        }
        if index == size {
            self.cursor = index;
            return;
        }
        let separator = separators.contains(self.input[index]) && self.input[index] != ' ';
        loop {
            index += 1;
            if index == size {
                break;
            }
            if self.input[index] == ' ' {
                if vi {
                    while index != size && self.input[index] == ' ' {
                        index += 1;
                    }
                }
                break;
            }
            if separator != separators.contains(self.input[index]) {
                break;
            }
        }
        self.cursor = index;
    }

    fn move_word_end(&mut self, separators: &str) {
        let size = self.input.len();
        let mut index = self.cursor;
        if index == size {
            return;
        }
        loop {
            index += 1;
            if index == size {
                self.cursor = index;
                return;
            }
            if self.input[index] != ' ' {
                break;
            }
        }
        let separator = separators.contains(self.input[index]);
        loop {
            index += 1;
            if index == size
                || self.input[index] == ' '
                || separator != separators.contains(self.input[index])
            {
                break;
            }
        }
        self.cursor = index.saturating_sub(1);
    }

    fn move_word_backward(&mut self, separators: &str) {
        let mut index = self.cursor;
        while index != 0 {
            index -= 1;
            if self.input[index] != ' ' {
                break;
            }
        }
        let separator = self
            .input
            .get(index)
            .is_some_and(|character| separators.contains(*character));
        while index != 0 {
            index -= 1;
            if self.input[index] == ' ' || separator != separators.contains(self.input[index]) {
                index += 1;
                break;
            }
        }
        self.cursor = index;
    }

    fn paste(&mut self, state: &Arc<Mutex<ServerState>>) {
        let source = self.yank.clone().unwrap_or_else(|| {
            state
                .lock()
                .ok()
                .and_then(|st| st.buffer(None).map(prompt_paste_text))
                .unwrap_or_default()
        });
        let inserted = source.chars().collect::<Vec<_>>();
        self.input
            .splice(self.cursor..self.cursor, inserted.iter().copied());
        self.cursor += inserted.len();
    }

    fn replace_completion(&mut self, start: usize, end: usize, replacement: &str) {
        self.input.splice(start..end, replacement.chars());
        self.cursor = start + replacement.chars().count();
    }

    fn complete_word(
        &mut self,
        state: &Arc<Mutex<ServerState>>,
        context: &command::ClientContext,
    ) -> bool {
        let Some((start, end)) = prompt_word_range(&self.input, self.cursor) else {
            return false;
        };
        let word = self.input[start..end].iter().collect::<String>();
        if word.len() >= 64 {
            return false;
        }
        let completion = state
            .lock()
            .ok()
            .map(|state| {
                command_prompt_completion(
                    &state,
                    context,
                    &self.spec.prompt_type,
                    &word,
                    start == 0,
                )
            })
            .unwrap_or(PromptCompletion::None);
        match completion {
            PromptCompletion::None => false,
            PromptCompletion::Replace(replacement) => {
                self.replace_completion(start, end, &replacement);
                true
            }
            PromptCompletion::Menu {
                items,
                replace_entire,
            } => {
                self.completion = Some(PromptCompletionMenu {
                    items,
                    selected: 0,
                    start,
                    end,
                    replace_entire,
                });
                false
            }
        }
    }

    fn handle_completion_key(&mut self, key: &str) -> bool {
        let Some(menu) = self.completion.as_mut() else {
            return false;
        };
        let last = menu.items.len().saturating_sub(1);
        let mut choose = None;
        let mut close = false;
        if let Some(index) = key
            .chars()
            .next()
            .filter(|_| key.chars().count() == 1)
            .and_then(|character| character.to_digit(10))
            .map(|index| index as usize)
            .filter(|index| *index < menu.items.len())
        {
            choose = Some(index);
            close = true;
        } else {
            match key {
                "BTab" | "Up" | "k" => {
                    menu.selected = if menu.selected == 0 {
                        last
                    } else {
                        menu.selected - 1
                    };
                }
                "Tab" => {
                    if menu.selected == last {
                        close = true;
                    } else {
                        menu.selected += 1;
                    }
                }
                "Down" | "j" => {
                    menu.selected = if menu.selected == last {
                        0
                    } else {
                        menu.selected + 1
                    };
                }
                "PPage" | "C-b" => menu.selected = menu.selected.saturating_sub(5),
                "NPage" => menu.selected = (menu.selected + 5).min(last),
                "Home" | "g" => menu.selected = 0,
                "End" | "G" => menu.selected = last,
                "Enter" | "C-m" => {
                    choose = Some(menu.selected);
                    close = true;
                }
                "BSpace" | "Escape" | "C-[" | "C-c" | "C-g" | "q" => close = true,
                _ => {}
            }
        }
        if close {
            let menu = self.completion.take().expect("completion menu checked");
            if let Some(item) = choose.and_then(|index| menu.items.get(index)) {
                if menu.replace_entire {
                    self.input = item.replacement.chars().collect();
                    self.cursor = self.input.len();
                } else {
                    self.replace_completion(menu.start, menu.end, &item.replacement);
                }
            }
        }
        true
    }

    fn handle_key(
        &mut self,
        key: &str,
        state: &Arc<Mutex<ServerState>>,
        hub: &StatusHub,
        context: &command::ClientContext,
    ) -> CommandPromptInput {
        if self.handle_completion_key(key) {
            return CommandPromptInput::Continue;
        }
        if self.spec.key {
            self.input = key.chars().collect();
            self.cursor = self.input.len();
            return self.finish_page(state, hub, context);
        }
        if self.spec.numeric {
            if key.chars().count() == 1
                && key
                    .chars()
                    .next()
                    .is_some_and(|character| character.is_ascii_digit())
            {
                self.input
                    .push(key.chars().next().expect("numeric key checked"));
                self.cursor = self.input.len();
                return CommandPromptInput::Continue;
            }
            return self.finish_page(state, hub, context);
        }
        if self.spec.single || self.quote_next {
            self.quote_next = false;
            let character = match key {
                "C-Space" => Some('\0'),
                "BSpace" => Some('\u{7f}'),
                "Space" => Some(' '),
                _ if key.starts_with("C-") && key.chars().count() == 3 => key
                    .chars()
                    .nth(2)
                    .map(|character| ((character.to_ascii_lowercase() as u8) & 0x1f) as char),
                _ if key.chars().count() == 1 => key.chars().next(),
                _ => None,
            };
            if let Some(character) = character {
                self.input.insert(self.cursor, character);
                self.cursor += 1;
                self.changed('=', state, hub, context);
                if self.spec.single {
                    return if self.input.len() == 1 {
                        self.finish_page(state, hub, context)
                    } else {
                        CommandPromptInput::Cancel
                    };
                }
            }
            return CommandPromptInput::Continue;
        }

        let target = context
            .current_session_id
            .map(|id| format!("${id}"))
            .unwrap_or_default();
        let separators = state
            .lock()
            .ok()
            .and_then(|st| {
                st.option_for_target(&target, "word-separators")
                    .map(str::to_string)
            })
            .unwrap_or_else(|| " !\"#$%&'()*+,-./:;<=>?@[\\]^`{|}~".to_string());
        let vi_keys = state
            .lock()
            .ok()
            .is_some_and(|st| st.option_for_target(&target, "status-keys") == Some("vi"));
        if self.vi_command {
            match key {
                "i" => {
                    self.vi_command = false;
                    return CommandPromptInput::Continue;
                }
                "a" => {
                    self.cursor = (self.cursor + 1).min(self.input.len());
                    self.vi_command = false;
                    return CommandPromptInput::Continue;
                }
                "A" => {
                    self.cursor = self.input.len();
                    self.vi_command = false;
                    return CommandPromptInput::Continue;
                }
                "I" => {
                    self.cursor = 0;
                    self.vi_command = false;
                    return CommandPromptInput::Continue;
                }
                "C" => {
                    self.vi_command = false;
                    if self.cursor < self.input.len() {
                        self.input.truncate(self.cursor);
                        self.changed('=', state, hub, context);
                    }
                    return CommandPromptInput::Continue;
                }
                "s" => {
                    self.vi_command = false;
                    if self.cursor < self.input.len() {
                        self.input.remove(self.cursor);
                        self.changed('=', state, hub, context);
                    }
                    return CommandPromptInput::Continue;
                }
                "S" => {
                    self.vi_command = false;
                    self.input.clear();
                    self.cursor = 0;
                    self.changed('=', state, hub, context);
                    return CommandPromptInput::Continue;
                }
                "Escape" | "C-[" => return CommandPromptInput::Continue,
                "$" => self.cursor = self.input.len(),
                "0" | "^" => self.cursor = 0,
                "h" | "BSpace" => self.cursor = self.cursor.saturating_sub(1),
                "l" | "Right" => self.cursor = (self.cursor + 1).min(self.input.len()),
                "x" | "DC" if self.cursor < self.input.len() => {
                    self.input.remove(self.cursor);
                    self.changed('=', state, hub, context);
                }
                "X" | "C-h" => {
                    if self.input.is_empty() && self.spec.backspace_exit {
                        return CommandPromptInput::Cancel;
                    }
                    if self.cursor > 0 {
                        self.cursor -= 1;
                        self.input.remove(self.cursor);
                        self.changed('=', state, hub, context);
                    }
                }
                "D" if self.cursor < self.input.len() => {
                    self.input.truncate(self.cursor);
                    self.changed('=', state, hub, context);
                }
                "d" => {
                    self.input.clear();
                    self.cursor = 0;
                    self.changed('=', state, hub, context);
                }
                "b" => {
                    self.move_word_backward(&separators);
                    self.changed('=', state, hub, context);
                }
                "B" => {
                    self.move_word_backward("");
                    self.changed('=', state, hub, context);
                }
                "e" => {
                    self.move_word_end(&separators);
                    self.changed('=', state, hub, context);
                }
                "E" => {
                    self.move_word_end("");
                    self.changed('=', state, hub, context);
                }
                "w" => {
                    self.move_word_forward(true, &separators);
                    self.changed('=', state, hub, context);
                }
                "W" => {
                    self.move_word_forward(true, "");
                    self.changed('=', state, hub, context);
                }
                "p" => {
                    self.paste(state);
                    self.changed('=', state, hub, context);
                }
                "Up" | "k" => {
                    if let Ok(st) = state.lock() {
                        let history = st.prompt_history(&self.spec.prompt_type);
                        if self.history_index < history.len() {
                            self.history_index += 1;
                            self.input = history[history.len() - self.history_index]
                                .chars()
                                .collect();
                            self.cursor = self.input.len();
                        }
                    }
                    self.changed('=', state, hub, context);
                }
                "Down" | "j" if self.history_index > 0 => {
                    self.history_index -= 1;
                    if let Ok(st) = state.lock() {
                        let history = st.prompt_history(&self.spec.prompt_type);
                        self.input = if self.history_index == 0 {
                            Vec::new()
                        } else {
                            history[history.len() - self.history_index]
                                .chars()
                                .collect()
                        };
                        self.cursor = self.input.len();
                    }
                    self.changed('=', state, hub, context);
                }
                "q" | "C-c" => return CommandPromptInput::Cancel,
                "Enter" | "C-m" => return self.finish_page(state, hub, context),
                _ => {}
            }
            return CommandPromptInput::Continue;
        }
        if vi_keys
            && !matches!(
                key,
                "C-a"
                    | "C-c"
                    | "C-e"
                    | "C-g"
                    | "C-h"
                    | "Tab"
                    | "C-k"
                    | "C-n"
                    | "C-p"
                    | "C-t"
                    | "C-u"
                    | "C-v"
                    | "C-w"
                    | "C-y"
                    | "Space"
                    | "Enter"
                    | "C-m"
                    | "C-Left"
                    | "C-Right"
                    | "BSpace"
                    | "DC"
                    | "Down"
                    | "End"
                    | "Home"
                    | "Left"
                    | "Right"
                    | "Up"
                    | "Escape"
                    | "C-["
            )
            && (key.chars().count() != 1 || key.chars().next().is_some_and(char::is_control))
        {
            return CommandPromptInput::Continue;
        }
        match key {
            "Enter" | "C-m" => return self.finish_page(state, hub, context),
            "Escape" | "C-[" => {
                if vi_keys {
                    self.vi_command = true;
                    self.cursor = self.cursor.saturating_sub(1);
                    return CommandPromptInput::Continue;
                }
                return CommandPromptInput::Cancel;
            }
            "C-c" | "C-g" => return CommandPromptInput::Cancel,
            "Left" | "C-b" => self.cursor = self.cursor.saturating_sub(1),
            "Right" | "C-f" => self.cursor = (self.cursor + 1).min(self.input.len()),
            "Home" | "C-a" => self.cursor = 0,
            "End" | "C-e" => self.cursor = self.input.len(),
            "BSpace" | "C-h" => {
                if self.cursor == 0 && self.spec.backspace_exit {
                    return CommandPromptInput::Cancel;
                }
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.input.remove(self.cursor);
                    self.changed('=', state, hub, context);
                }
            }
            "DC" | "C-d" => {
                if self.cursor < self.input.len() {
                    self.input.remove(self.cursor);
                    self.changed('=', state, hub, context);
                }
            }
            "C-u" => {
                self.input.clear();
                self.cursor = 0;
                self.changed('=', state, hub, context);
            }
            "C-k" => {
                self.input.truncate(self.cursor);
                self.changed('=', state, hub, context);
            }
            "C-w" => {
                self.delete_previous_word(&separators);
                self.changed('=', state, hub, context);
            }
            "C-y" => {
                self.paste(state);
                self.changed('=', state, hub, context);
            }
            "C-t" => {
                let end = if self.cursor < self.input.len() {
                    self.cursor + 1
                } else {
                    self.cursor
                };
                if end >= 2 {
                    self.input.swap(end - 2, end - 1);
                    self.cursor = end;
                    self.changed('=', state, hub, context);
                }
            }
            "C-v" => self.quote_next = true,
            "Tab" => {
                if self.complete_word(state, context) {
                    self.changed('=', state, hub, context);
                }
            }
            "M-b" | "C-Left" => {
                self.move_word_backward(&separators);
                self.changed('=', state, hub, context);
            }
            "M-f" | "C-Right" => {
                self.move_word_forward(false, &separators);
                self.changed('=', state, hub, context);
            }
            "Up" | "C-p" => {
                if let Ok(st) = state.lock() {
                    let history = st.prompt_history(&self.spec.prompt_type);
                    if self.history_index < history.len() {
                        self.history_index += 1;
                        self.input = history[history.len() - self.history_index]
                            .chars()
                            .collect();
                        self.cursor = self.input.len();
                    }
                }
                self.changed('=', state, hub, context);
            }
            "Down" | "C-n" => {
                if self.history_index > 0 {
                    self.history_index -= 1;
                    if let Ok(st) = state.lock() {
                        let history = st.prompt_history(&self.spec.prompt_type);
                        self.input = if self.history_index == 0 {
                            Vec::new()
                        } else {
                            history[history.len() - self.history_index]
                                .chars()
                                .collect()
                        };
                        self.cursor = self.input.len();
                    }
                    self.changed('=', state, hub, context);
                }
            }
            "C-r" if self.spec.incremental => {
                let prefix = if self.input.is_empty() {
                    self.input = self.last.chars().collect();
                    self.cursor = self.input.len();
                    '='
                } else {
                    '-'
                };
                self.changed(prefix, state, hub, context);
            }
            "C-s" if self.spec.incremental => {
                let prefix = if self.input.is_empty() {
                    self.input = self.last.chars().collect();
                    self.cursor = self.input.len();
                    '='
                } else {
                    '+'
                };
                self.changed(prefix, state, hub, context);
            }
            _ => {
                let text = match key {
                    "Space" => Some(" "),
                    _ if key.chars().count() == 1
                        && !key.chars().next().is_some_and(char::is_control) =>
                    {
                        Some(key)
                    }
                    _ => None,
                };
                if let Some(text) = text {
                    let inserted = text.chars().collect::<Vec<_>>();
                    self.input
                        .splice(self.cursor..self.cursor, inserted.iter().copied());
                    self.cursor += inserted.len();
                    self.changed('=', state, hub, context);
                    if self.spec.single {
                        return self.finish_page(state, hub, context);
                    }
                }
            }
        }
        CommandPromptInput::Continue
    }
}

fn run_mode_command(
    value: &str,
    item_target: &str,
    state: &Arc<Mutex<ServerState>>,
    hub: &StatusHub,
    context: &command::ClientContext,
) -> command::CommandResult {
    if value.is_empty() {
        return command::CommandResult::ok("");
    }
    let aliases = match state.lock() {
        Ok(state) => state.command_aliases(),
        Err(_) => return command::CommandResult::err("server state poisoned\n"),
    };
    let command = command::replace_prompt_template(value, item_target, 1);
    let groups = match command::command_string_groups_with_aliases(&command, &aliases) {
        Ok(groups) => groups,
        Err(error) => return error,
    };
    let mut argv = Vec::new();
    for group in groups {
        if !argv.is_empty() {
            argv.push(";".to_string());
        }
        argv.extend(group);
    }
    command::run_with_context(&argv, state, &hub.snapshot().panes, context)
}

fn run_mode_edit(
    edit: &ModeEdit,
    value: &str,
    target: &str,
    state: &Arc<Mutex<ServerState>>,
    hub: &StatusHub,
    context: &command::ClientContext,
) -> command::CommandResult {
    match edit {
        ModeEdit::Option { name, .. } => {
            let args = vec![
                "set-option".to_string(),
                "-t".to_string(),
                target.to_string(),
                "--".to_string(),
                name.clone(),
                value.to_string(),
            ];
            let result = command::run_with_context(&args, state, &hub.snapshot().panes, context);
            if result.exit == 0 {
                if let Ok(mut state) = state.lock() {
                    let _ = state.mode_view_update_edit(target, edit, value);
                }
            }
            result
        }
        ModeEdit::BindingCommand {
            table,
            key,
            note,
            repeat,
            ..
        } => {
            if value.is_empty() {
                return command::CommandResult::ok("");
            }
            let aliases = match state.lock() {
                Ok(state) => state.command_aliases(),
                Err(_) => return command::CommandResult::err("server state poisoned\n"),
            };
            let groups = match command::command_string_groups_with_aliases(value, &aliases) {
                Ok(groups) => groups,
                Err(error) => return error,
            };
            let mut commands = Vec::new();
            for group in groups {
                if !commands.is_empty() {
                    commands.push(";".to_string());
                }
                commands.extend(group);
            }
            let Some(key_code) = parse_key_name(key) else {
                return command::CommandResult::err(format!("unknown key: {key}\n"));
            };
            let mut state = match state.lock() {
                Ok(state) => state,
                Err(_) => return command::CommandResult::err("server state poisoned\n"),
            };
            state.bind_key(table, key_code, commands.clone(), *repeat, note.clone());
            let display = command::display_command(&commands);
            let _ = state.mode_view_update_binding(
                target,
                ModeBindingUpdate {
                    table: table.clone(),
                    key: key.clone(),
                    command_text: display,
                    command: commands,
                    note: note.clone(),
                    repeat: *repeat,
                },
            );
            command::CommandResult::ok("")
        }
        ModeEdit::BindingNote {
            table,
            key,
            command: commands,
            repeat,
            ..
        } => {
            if value.is_empty() {
                return command::CommandResult::ok("");
            }
            let Some(key_code) = parse_key_name(key) else {
                return command::CommandResult::err(format!("unknown key: {key}\n"));
            };
            let mut state = match state.lock() {
                Ok(state) => state,
                Err(_) => return command::CommandResult::err("server state poisoned\n"),
            };
            state.bind_key(
                table,
                key_code,
                commands.clone(),
                *repeat,
                Some(value.to_string()),
            );
            let display = command::display_command(commands);
            let _ = state.mode_view_update_binding(
                target,
                ModeBindingUpdate {
                    table: table.clone(),
                    key: key.clone(),
                    command_text: display,
                    command: commands.clone(),
                    note: Some(value.to_string()),
                    repeat: *repeat,
                },
            );
            command::CommandResult::ok("")
        }
    }
}

fn render_prompt_input(input: &[char]) -> String {
    let mut rendered = String::new();
    for character in input {
        match *character as u32 {
            0x00..=0x1f => {
                rendered.push('^');
                rendered.push(char::from_u32((*character as u32) | 0x40).unwrap_or('?'));
            }
            0x7f => rendered.push_str("^?"),
            0x23 => rendered.push_str("##"),
            _ => rendered.push(*character),
        }
    }
    rendered
}

fn prompt_input_width(input: &[char]) -> usize {
    format::display_width(&render_prompt_input(input))
}

fn clip_prompt_display(value: &str, offset: usize, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut position = 0;
    let mut output = String::new();
    for (token, token_width) in format::display_tokens(value) {
        if token_width == 0 {
            output.push_str(token);
            continue;
        }
        let end = position + token_width;
        if end > offset && end <= offset + width {
            output.push_str(token);
        } else if end > offset + width {
            break;
        }
        position = end;
    }
    output
}

fn common_prompt_prefix(values: &[String]) -> Option<String> {
    let mut prefix = values.first()?.chars().collect::<Vec<_>>();
    for value in &values[1..] {
        let common = prefix
            .iter()
            .copied()
            .zip(value.chars())
            .take_while(|(left, right)| left == right)
            .count();
        prefix.truncate(common);
    }
    Some(prefix.into_iter().collect())
}

fn prompt_paste_text(data: &[u8]) -> String {
    let mut output = String::new();
    let mut offset = 0;
    while offset < data.len() {
        let first = data[offset];
        if first.is_ascii() {
            if first <= 0x1f || first == 0x7f {
                break;
            }
            output.push(first as char);
            offset += 1;
            continue;
        }
        let width = match first {
            0xc2..=0xdf => 2,
            0xe0..=0xef => 3,
            0xf0..=0xf4 => 4,
            _ => break,
        };
        let Some(bytes) = data.get(offset..offset + width) else {
            break;
        };
        let Ok(value) = std::str::from_utf8(bytes) else {
            break;
        };
        output.push_str(value);
        offset += width;
    }
    output
}

fn prompt_word_range(input: &[char], cursor: usize) -> Option<(usize, usize)> {
    if input.is_empty() {
        return Some((0, 0));
    }
    let index = cursor.saturating_sub(1).min(input.len() - 1);
    let mut first = index;
    while first > 0 && input[first] != ' ' {
        first -= 1;
    }
    while first < input.len() && input[first] == ' ' {
        first += 1;
    }
    let mut last = index;
    while last < input.len() && input[last] != ' ' {
        last += 1;
    }
    while last > 0 && last < input.len() && input[last] == ' ' {
        last -= 1;
    }
    if last < input.len() {
        last += 1;
    }
    (last >= first).then_some((first, last))
}

fn completion_menu(mut values: Vec<String>, prefix: &str) -> PromptCompletion {
    if values.len() > 10 {
        values.drain(..values.len() - 10);
    }
    PromptCompletion::Menu {
        items: values
            .into_iter()
            .map(|value| PromptCompletionItem {
                label: value.clone(),
                replacement: format!("{prefix}{value}"),
            })
            .collect(),
        replace_entire: false,
    }
}

fn command_prompt_completion(
    state: &ServerState,
    context: &command::ClientContext,
    prompt_type: &str,
    word: &str,
    at_start: bool,
) -> PromptCompletion {
    if !matches!(prompt_type, "target" | "window-target")
        && !word.starts_with("-t")
        && !word.starts_with("-s")
    {
        if word.is_empty() {
            return PromptCompletion::None;
        }
        let mut matches = BTreeSet::new();
        for candidate in super::registry::COMMAND_SPECS
            .iter()
            .flat_map(|spec| std::iter::once(spec.name).chain(spec.alias.iter().copied()))
        {
            if candidate.starts_with(word) {
                matches.insert(candidate.to_string());
            }
        }
        for (alias, _) in state.command_aliases() {
            if alias.starts_with(word) {
                matches.insert(alias);
            }
        }
        if !at_start {
            for candidate in
                super::options::option_names().chain(command::LAYOUT_NAMES.iter().copied())
            {
                if candidate.starts_with(word) {
                    matches.insert(candidate.to_string());
                }
            }
        }
        let matches = matches.into_iter().collect::<Vec<_>>();
        return match matches.as_slice() {
            [] => PromptCompletion::None,
            [only] => PromptCompletion::Replace(format!("{only} ")),
            _ => match common_prompt_prefix(&matches) {
                Some(prefix) if prefix != word => PromptCompletion::Replace(prefix),
                _ => completion_menu(matches, ""),
            },
        };
    }

    if prompt_type == "window-target" {
        let Some(session_index) = context
            .current_session_id
            .and_then(|id| state.sessions().iter().position(|session| session.id == id))
        else {
            return PromptCompletion::None;
        };
        let session = &state.sessions()[session_index];
        let mut items = session
            .windows
            .iter()
            .enumerate()
            .filter(|(_, link)| link.index.to_string().starts_with(word))
            .map(|(window_index, link)| PromptCompletionItem {
                label: format!(
                    "{} ({})",
                    link.index,
                    state.window(session_index, window_index).name
                ),
                replacement: link.index.to_string(),
            })
            .collect::<Vec<_>>();
        items.truncate(10);
        return match items.len() {
            0 => PromptCompletion::None,
            1 if items[0].replacement == word => PromptCompletion::None,
            1 => PromptCompletion::Replace(items[0].replacement.clone()),
            _ => PromptCompletion::Menu {
                items,
                replace_entire: true,
            },
        };
    }

    let (flag, target) = if let Some(target) = word.strip_prefix("-t") {
        ("-t", target)
    } else if let Some(target) = word.strip_prefix("-s") {
        ("-s", target)
    } else {
        ("", word)
    };
    let Some(colon) = target.find(':') else {
        let mut matches = state
            .sessions()
            .iter()
            .filter_map(|session| {
                if target.starts_with('$') {
                    let candidate = format!("${}:", session.id);
                    candidate.starts_with(target).then_some(candidate)
                } else {
                    let candidate = format!("{}:", session.name);
                    candidate.starts_with(target).then_some(candidate)
                }
            })
            .collect::<Vec<_>>();
        matches.sort();
        matches.dedup();
        return match matches.as_slice() {
            [] => PromptCompletion::None,
            _ => match common_prompt_prefix(&matches) {
                Some(prefix) if format!("{flag}{prefix}") != word => {
                    PromptCompletion::Replace(format!("{flag}{prefix}"))
                }
                _ => completion_menu(matches, flag),
            },
        };
    };
    if target[colon + 1..].contains('.') {
        return PromptCompletion::None;
    }
    let session_target = &target[..colon];
    let session_index = if session_target.is_empty() {
        context
            .current_session_id
            .and_then(|id| state.sessions().iter().position(|session| session.id == id))
    } else if let Some(id) = session_target
        .strip_prefix('$')
        .and_then(|id| id.parse::<u32>().ok())
    {
        state.sessions().iter().position(|session| session.id == id)
    } else {
        state
            .sessions()
            .iter()
            .position(|session| session.name == session_target)
    };
    let Some(session_index) = session_index else {
        return PromptCompletion::None;
    };
    let session = &state.sessions()[session_index];
    let window_prefix = &target[colon + 1..];
    let mut items = session
        .windows
        .iter()
        .enumerate()
        .filter(|(_, link)| link.index.to_string().starts_with(window_prefix))
        .map(|(window_index, link)| PromptCompletionItem {
            label: format!(
                "{}:{} ({})",
                session.name,
                link.index,
                state.window(session_index, window_index).name
            ),
            replacement: format!("{flag}{}:{}", session.name, link.index),
        })
        .collect::<Vec<_>>();
    items.truncate(10);
    match items.len() {
        0 => PromptCompletion::None,
        1 if items[0].replacement == word => PromptCompletion::None,
        1 => PromptCompletion::Replace(items[0].replacement.clone()),
        _ => PromptCompletion::Menu {
            items,
            replace_entire: false,
        },
    }
}

fn handle_command_prompt_key(
    prompt: &mut Option<CommandPrompt>,
    key: &str,
    state: &Arc<Mutex<ServerState>>,
    hub: &StatusHub,
    context: &command::ClientContext,
) {
    let Some(active) = prompt.as_mut() else {
        return;
    };
    match active.handle_key(key, state, hub, context) {
        CommandPromptInput::Continue => {}
        CommandPromptInput::Finish(result) => {
            let mut active = prompt.take().expect("command prompt checked");
            active.complete(&result, state, context);
        }
        CommandPromptInput::Cancel => {
            let mut active = prompt.take().expect("command prompt checked");
            active.cancel_external();
        }
    }
}

/// Resolve and execute one key from an attached client's active table.
///
/// This is the interactive counterpart to the command interpreter: where a
/// command client sends `new-window` over the imsg control plane, an attached
/// client presses `C-b c` and this maps it to the same [`ServerState`] mutation.
/// Default and user bindings share the same mutable table. Ordinary commands
/// run through the command interpreter in the attached client's context;
/// client-local operations such as detach, confirmation, and copy-mode entry
/// return an outcome for the attach loop to apply.
///
/// The destructive bindings `&` (kill-window) and `x` (kill-pane) do not mutate
/// state here: like real tmux they first raise a `confirm-before` `(y/n)` prompt
/// ([`PrefixOutcome::Confirm`]); the attach loop performs the kill only after the
/// user answers `y`. Unknown keys are a no-op (real tmux rings the bell).
///
/// On any layout change the now-active pane is resized to the client's current
/// geometry, mirroring tmux keeping every window in a session at the session
/// size — otherwise a freshly spawned window's pane would keep its 80×24 default.
fn dispatch_key_binding(
    table: &str,
    key: KeyCode,
    state: &Arc<Mutex<ServerState>>,
    target: &str,
    cols: u16,
    pane_rows: u16,
    hub: &StatusHub,
    context: &command::ClientContext,
    mouse: Option<MouseEvent>,
) -> PrefixOutcome {
    let binding = match state.lock() {
        Ok(st) => st.key_binding(table, key).cloned(),
        Err(_) => None,
    };
    let Some(binding) = binding else {
        return PrefixOutcome::Handled { changed: false };
    };
    let Some(command_name) = binding.command.first().map(String::as_str) else {
        return PrefixOutcome::Handled { changed: false };
    };

    match command_name {
        "detach-client" | "detach" => return PrefixOutcome::Detach,
        "send-prefix" => {
            let option = if binding.command.iter().any(|word| word == "-2") {
                "prefix2"
            } else {
                "prefix"
            };
            let bytes = state
                .lock()
                .ok()
                .and_then(|st| {
                    st.option_for_target(target, option)
                        .or_else(|| super::options::option_default(option))
                        .and_then(parse_key_name)
                })
                .and_then(basic_key_bytes)
                .unwrap_or_default();
            return PrefixOutcome::SendPrefix(bytes);
        }
        "copy-mode" if !binding.command.iter().any(|word| word == ";") => {
            return PrefixOutcome::CopyMode {
                page_up: binding.command.iter().any(|word| word == "-u"),
                page_down: binding.command.iter().any(|word| word == "-d"),
                slider: binding.command.iter().any(|word| word == "-S"),
                mouse,
                begin_selection: binding.command.iter().any(|word| word == "-M"),
            };
        }
        "command-prompt" => {
            return PrefixOutcome::Prompt {
                args: binding.command.clone(),
            };
        }
        "display-message"
            if !binding.command.iter().any(|word| word == ";")
                && !binding.command.iter().any(|word| word == "-p") =>
        {
            let mut command = binding.command.clone();
            command.insert(1, "-p".to_string());
            let agents = hub.snapshot().panes;
            let result = command::run_with_context(&command, state, &agents, context);
            if result.exit != 0 {
                return PrefixOutcome::Handled { changed: false };
            }
            let mut text = result
                .stdout
                .strip_suffix('\n')
                .unwrap_or(&result.stdout)
                .to_string();
            if binding.command.iter().any(|word| word == "-N") {
                text = text.replace('#', "##");
            }
            let explicit = binding
                .command
                .windows(2)
                .find(|words| words[0] == "-d")
                .and_then(|words| words[1].parse::<u64>().ok());
            let milliseconds = explicit
                .or_else(|| {
                    state.lock().ok().and_then(|state| {
                        state
                            .option_for_target(target, "display-time")
                            .and_then(|value| value.parse().ok())
                    })
                })
                .unwrap_or(750);
            return PrefixOutcome::Message {
                text,
                duration: Duration::from_millis(milliseconds),
            };
        }
        "confirm-before" | "confirm"
            if binding
                .command
                .last()
                .is_some_and(|word| word == "kill-window") =>
        {
            let name = state
                .lock()
                .ok()
                .and_then(|st| st.active_window_name(target))
                .unwrap_or_default();
            return PrefixOutcome::Confirm {
                prompt: format!("kill-window {name}? (y/n)"),
                action: ConfirmAction::KillWindow,
            };
        }
        "confirm-before" | "confirm"
            if binding
                .command
                .last()
                .is_some_and(|word| word == "kill-pane") =>
        {
            let idx = state
                .lock()
                .ok()
                .and_then(|st| st.active_pane_index(target))
                .unwrap_or(0);
            return PrefixOutcome::Confirm {
                prompt: format!("kill-pane {idx}? (y/n)"),
                action: ConfirmAction::KillPane,
            };
        }
        _ => {}
    }

    let agents = hub.snapshot().panes;
    let mut binding_context = context.clone();
    binding_context.key_event = Some(key);
    binding_context.mouse = mouse;
    let result = command::run_with_context(&binding.command, state, &agents, &binding_context);
    if result.exit == 0 && !result.stdout_data().is_empty() {
        return PrefixOutcome::ViewOutput(result.stdout_data().to_vec());
    }
    let changed = result.exit == 0;
    if changed {
        if let Ok(mut st) = state.lock() {
            let _ = st.resize_session(target, cols, pane_rows);
        }
    }
    PrefixOutcome::Handled { changed }
}

pub(super) fn dispatch_control_client_keys(
    keys: &[ClientKey],
    prefix_pending: &mut bool,
    state: &Arc<Mutex<ServerState>>,
    target: &str,
    hub: &StatusHub,
    context: &command::ClientContext,
) -> bool {
    for injected in keys {
        let bytes = &injected.bytes;
        let mut index = 0;
        while index < bytes.len() {
            let start = index;
            let (key, consumed) = decode_tty_key(&bytes[index..])
                .map(|(decoded, consumed)| (decoded.code, consumed))
                .unwrap_or_else(|| (Some(key_from_byte(bytes[index])), 1));
            index += consumed;
            let Some(key) = key else {
                continue;
            };
            if is_configured_prefix(state, target, key) {
                *prefix_pending = true;
                continue;
            }

            let from_prefix = std::mem::take(prefix_pending);
            let table = if from_prefix {
                "prefix".to_string()
            } else if state
                .lock()
                .ok()
                .is_some_and(|state| state.copy_mode_active(target))
            {
                copy_table_name(state, target).to_string()
            } else {
                client_key_table(state, target)
            };
            let bound = state
                .lock()
                .ok()
                .is_some_and(|state| state.key_binding(&table, key).is_some());
            if !bound {
                if !from_prefix && injected.forward_unbound {
                    if let Ok(state) = state.lock() {
                        let _ = state.input_to_active_pane(target, &bytes[start..index]);
                    }
                }
                continue;
            }

            match dispatch_key_binding(&table, key, state, target, 80, 24, hub, context, None) {
                PrefixOutcome::Detach => return true,
                PrefixOutcome::SendPrefix(bytes) => {
                    if let Ok(state) = state.lock() {
                        let _ = state.input_to_active_pane(target, &bytes);
                    }
                }
                PrefixOutcome::CopyMode {
                    page_up, page_down, ..
                } => {
                    set_copy_mode_state(state, target, true, page_up);
                    if page_down {
                        if let Ok(mut state) = state.lock() {
                            let vi = copy_mode_uses_vi_keys(&state, target);
                            let separators = state
                                .option_for_target(target, "word-separators")
                                .unwrap_or("")
                                .to_string();
                            let _ = state.copy_mode_command(target, "page-down", vi, &separators);
                        }
                    }
                }
                PrefixOutcome::Confirm { .. }
                | PrefixOutcome::Prompt { .. }
                | PrefixOutcome::Message { .. }
                | PrefixOutcome::ViewOutput(_)
                | PrefixOutcome::Handled { .. } => {}
            }
        }
    }
    false
}

#[cfg(test)]
fn dispatch_prefix_key(
    key: u8,
    state: &Arc<Mutex<ServerState>>,
    target: &str,
    cols: u16,
    pane_rows: u16,
) -> PrefixOutcome {
    let current_session_id = state.lock().ok().and_then(|st| st.session_id(target));
    let context = command::ClientContext {
        current_session_id,
        ..command::ClientContext::default()
    };
    dispatch_key_binding(
        "prefix",
        key_from_byte(key),
        state,
        target,
        cols,
        pane_rows,
        &StatusHub::new(),
        &context,
        None,
    )
}

/// A logical key parsed from the client's tty input, limited to what the attach
/// loop's prefix table and copy-mode navigation need to recognize. Every other
/// key stays an opaque byte ([`Key::Byte`]) and is forwarded to the pane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Key {
    /// A single literal byte (a command key like `c`, or plain input).
    Byte(u8),
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    /// A bare `Escape` (ESC with no CSI introducer following it this chunk).
    Escape,
    Enter,
}

/// Parse one logical [`Key`] from the front of `bytes` (non-empty), returning the
/// key and how many bytes it consumed.
///
/// Recognizes just the escape sequences copy-mode navigation needs — `PgUp`/
/// `PgDn` (`CSI 5~` / `CSI 6~`) and the up/down arrows (`CSI A` / `CSI B`) — plus
/// the single bytes the prefix table and copy mode use. A lone `ESC`, or any
/// other/partial CSI, resolves to a single-byte key so the caller can fall back
/// to a bell (in a key table) or verbatim forwarding (in passthrough).
///
/// This is only invoked while interpreting keys (after the prefix, or in copy
/// mode); ordinary passthrough forwards bytes untouched, so an app's own arrow
/// keys and UTF-8 are never reinterpreted.
fn read_key(bytes: &[u8]) -> (Key, usize) {
    let Some((decoded, consumed)) = decode_tty_key(bytes) else {
        return (Key::Byte(bytes[0]), 1);
    };
    let key = match decoded.code.map(|code| code.base) {
        Some(KeyBase::Special(SpecialKey::PageUp)) => Key::PageUp,
        Some(KeyBase::Special(SpecialKey::PageDown)) => Key::PageDown,
        Some(KeyBase::Special(SpecialKey::Up)) => Key::Up,
        Some(KeyBase::Special(SpecialKey::Down)) => Key::Down,
        Some(KeyBase::Special(SpecialKey::Left)) => Key::Left,
        Some(KeyBase::Special(SpecialKey::Right)) => Key::Right,
        Some(KeyBase::Char('\u{1b}')) => Key::Escape,
        Some(KeyBase::Char('\r' | '\n')) => Key::Enter,
        _ if consumed == 1 => Key::Byte(bytes[0]),
        _ => Key::Byte(0x1b),
    };
    (key, consumed)
}

fn is_configured_prefix(state: &Arc<Mutex<ServerState>>, target: &str, key: KeyCode) -> bool {
    state.lock().ok().is_some_and(|st| {
        ["prefix", "prefix2"].into_iter().any(|option| {
            st.option_for_target(target, option)
                .or_else(|| super::options::option_default(option))
                .and_then(parse_key_name)
                .is_some_and(|prefix| prefix == key)
        })
    })
}

fn client_key_table(state: &Arc<Mutex<ServerState>>, target: &str) -> String {
    state
        .lock()
        .ok()
        .and_then(|state| {
            state
                .option_for_target(target, "key-table")
                .map(str::to_string)
        })
        .filter(|table| !table.is_empty())
        .unwrap_or_else(|| "root".to_string())
}

fn plain_prompt_key(byte: u8) -> String {
    match byte {
        0 => "C-Space".to_string(),
        1..=8 => format!("C-{}", (b'a' + byte - 1) as char),
        9 => "Tab".to_string(),
        10..=12 => format!("C-{}", (b'a' + byte - 1) as char),
        13 => "Enter".to_string(),
        14..=26 => format!("C-{}", (b'a' + byte - 1) as char),
        27 => "Escape".to_string(),
        28 => "C-\\".to_string(),
        29 => "C-]".to_string(),
        30 => "C-^".to_string(),
        31 => "C-_".to_string(),
        32 => "Space".to_string(),
        33..=126 => (byte as char).to_string(),
        127 => "BSpace".to_string(),
        _ => format!("0x{byte:02x}"),
    }
}

fn meta_prompt_key(byte: u8) -> String {
    match byte {
        0 => "C-M-Space".to_string(),
        1..=8 => format!("C-M-{}", (b'a' + byte - 1) as char),
        9 => "M-Tab".to_string(),
        10..=12 => format!("C-M-{}", (b'a' + byte - 1) as char),
        13 => "M-Enter".to_string(),
        14..=26 => format!("C-M-{}", (b'a' + byte - 1) as char),
        27 => "M-Escape".to_string(),
        28 => "C-M-\\".to_string(),
        29 => "C-M-]".to_string(),
        30 => "C-M-^".to_string(),
        31 => "C-M-_".to_string(),
        32 => "M-Space".to_string(),
        33..=126 => format!("M-{}", byte as char),
        127 => "M-BSpace".to_string(),
        _ => format!("M-0x{byte:02x}"),
    }
}

fn function_key(number: &str) -> Option<&'static str> {
    match number {
        "11" => Some("F1"),
        "12" => Some("F2"),
        "13" => Some("F3"),
        "14" => Some("F4"),
        "15" => Some("F5"),
        "17" => Some("F6"),
        "18" => Some("F7"),
        "19" => Some("F8"),
        "20" => Some("F9"),
        "21" => Some("F10"),
        "23" => Some("F11"),
        "24" => Some("F12"),
        _ => None,
    }
}

fn shifted_function_key(number: &str) -> Option<String> {
    let index = match number {
        "25" => 3,
        "26" => 4,
        "28" => 5,
        "29" => 6,
        "31" => 7,
        "32" => 8,
        "33" => 9,
        "34" => 10,
        "23" => 11,
        "24" => 12,
        _ => return None,
    };
    Some(format!("S-F{index}"))
}

fn decode_ss3(final_byte: u8) -> Option<&'static str> {
    match final_byte {
        b'P' => Some("F1"),
        b'Q' => Some("F2"),
        b'R' => Some("F3"),
        b'S' => Some("F4"),
        b'M' => Some("KPEnter"),
        b'j' => Some("KP*"),
        b'k' => Some("KP+"),
        b'm' => Some("KP-"),
        b'n' => Some("KP."),
        b'o' => Some("KP/"),
        b'p' => Some("KP0"),
        b'q' => Some("KP1"),
        b'r' => Some("KP2"),
        b's' => Some("KP3"),
        b't' => Some("KP4"),
        b'u' => Some("KP5"),
        b'v' => Some("KP6"),
        b'w' => Some("KP7"),
        b'x' => Some("KP8"),
        b'y' => Some("KP9"),
        b'A' => Some("Up"),
        b'B' => Some("Down"),
        b'C' => Some("Right"),
        b'D' => Some("Left"),
        b'H' => Some("Home"),
        b'F' => Some("End"),
        b'a' => Some("C-Up"),
        b'b' => Some("C-Down"),
        b'c' => Some("C-Right"),
        b'd' => Some("C-Left"),
        _ => None,
    }
}

fn decode_csi(params: &str, final_byte: u8) -> Option<String> {
    let fixed = match (params, final_byte) {
        ("", b'A') => Some("Up"),
        ("", b'B') => Some("Down"),
        ("", b'C') => Some("Right"),
        ("", b'D') => Some("Left"),
        ("", b'H') => Some("Home"),
        ("", b'F') => Some("End"),
        ("", b'a') => Some("S-Up"),
        ("", b'b') => Some("S-Down"),
        ("", b'c') => Some("S-Right"),
        ("", b'd') => Some("S-Left"),
        ("", b'I') => Some("FocusIn"),
        ("", b'O') => Some("FocusOut"),
        ("", b'Z') => Some("BTab"),
        ("1;5", b'Z') => Some("C-S-Tab"),
        _ => None,
    };
    if let Some(name) = fixed {
        return Some(name.to_string());
    }
    if let Some((base, modifier)) = params.split_once(';') {
        let key = match (base, final_byte) {
            ("1", b'A') => "Up",
            ("1", b'B') => "Down",
            ("1", b'C') => "Right",
            ("1", b'D') => "Left",
            ("1", b'H') => "Home",
            ("1", b'F') => "End",
            _ => "",
        };
        let prefix = match modifier {
            "2" => "S-",
            "3" => "M-",
            "4" => "M-S-",
            "5" => "C-",
            "6" => "C-S-",
            "7" => "C-M-",
            "8" => "C-M-S-",
            _ => "",
        };
        if !key.is_empty() && !prefix.is_empty() {
            return Some(format!("{prefix}{key}"));
        }
    }
    match final_byte {
        b'~' => match params {
            "1" | "7" => Some("Home".to_string()),
            "2" => Some("IC".to_string()),
            "3" => Some("DC".to_string()),
            "4" | "8" => Some("End".to_string()),
            "5" => Some("PPage".to_string()),
            "6" => Some("NPage".to_string()),
            "200" => Some("PasteStart".to_string()),
            "201" => Some("PasteEnd".to_string()),
            number => function_key(number)
                .map(str::to_string)
                .or_else(|| shifted_function_key(number)),
        },
        b'$' => shifted_function_key(params),
        b'^' => function_key(params).map(|name| format!("C-{name}")),
        b'@' => function_key(params).map(|name| format!("C-S-{name}")),
        b'u' => {
            let (code, modifier) = params.split_once(';')?;
            let code = code.parse::<u8>().ok()?;
            match modifier {
                "2" => Some(format!("S-{}", plain_prompt_key(code))),
                "5" => Some(format!("C-{}", plain_prompt_key(code))),
                "7" => Some(format!("C-M-{}", plain_prompt_key(code))),
                _ => None,
            }
        }
        _ => None,
    }
}

#[derive(Clone, Debug)]
struct DecodedTtyKey {
    name: String,
    code: Option<KeyCode>,
    mouse: Option<MouseEvent>,
}

/// Decode one terminal key for every attached-client consumer.
///
/// The display name remains available for `command-prompt -k`; table dispatch
/// uses the semantic code. A few internal terminal events (focus and bracketed
/// paste boundaries) have display names but are not accepted by tmux's
/// control-plane key-name parser, so their code is intentionally `None`.
fn decode_tty_key(bytes: &[u8]) -> Option<(DecodedTtyKey, usize)> {
    let first = *bytes.first()?;
    if first != 0x1b {
        let (name, consumed) = if first >= 0x80 {
            let width = match first {
                0xc2..=0xdf => 2,
                0xe0..=0xef => 3,
                0xf0..=0xf4 => 4,
                _ => 1,
            };
            let text = std::str::from_utf8(bytes.get(..width)?).ok()?;
            (text.to_string(), width)
        } else {
            (plain_prompt_key(first), 1)
        };
        return Some((
            DecodedTtyKey {
                code: parse_key_name(&name),
                name,
                mouse: None,
            },
            consumed,
        ));
    }
    if bytes.len() == 1 {
        let name = "Escape".to_string();
        return Some((
            DecodedTtyKey {
                code: parse_key_name(&name),
                name,
                mouse: None,
            },
            1,
        ));
    }

    let (start, meta) = if bytes[1] == 0x1b {
        if bytes.len() == 2 {
            let name = "M-Escape".to_string();
            return Some((
                DecodedTtyKey {
                    code: parse_key_name(&name),
                    name,
                    mouse: None,
                },
                2,
            ));
        }
        (2, true)
    } else {
        (1, false)
    };

    if !meta && bytes.get(start..start + 2) == Some(b"[<") {
        let end = bytes
            .iter()
            .enumerate()
            .skip(start + 2)
            .find_map(|(index, &byte)| matches!(byte, b'M' | b'm').then_some(index))?;
        let fields = std::str::from_utf8(&bytes[start + 2..end]).ok()?;
        let mut fields = fields.split(';');
        let button = fields.next()?.parse::<u16>().ok()?;
        let x = fields.next()?.parse::<u16>().ok()?.checked_sub(1)?;
        let y = fields.next()?.parse::<u16>().ok()?.checked_sub(1)?;
        if fields.next().is_some() {
            return None;
        }
        let mouse = MouseEvent::from_terminal_report(
            MouseProtocol::Sgr,
            button,
            bytes[end] == b'm',
            MousePosition { x, y },
        );
        let code = mouse.key_code_for(super::key::MouseLocation::Pane);
        let name = code
            .map(|code| code.to_string())
            .unwrap_or_else(|| "Mouse".into());
        return Some((
            DecodedTtyKey {
                code,
                name,
                mouse: Some(mouse),
            },
            end + 1,
        ));
    }

    if !meta && bytes.get(start..start + 2) == Some(b"[M") {
        let button = u16::from(bytes.get(start + 2)?.checked_sub(32)?);
        let x = u16::from(bytes.get(start + 3)?.checked_sub(33)?);
        let y = u16::from(bytes.get(start + 4)?.checked_sub(33)?);
        let mouse = MouseEvent::from_terminal_report(
            MouseProtocol::Legacy,
            button,
            button & 0xc3 == 3,
            MousePosition { x, y },
        );
        let code = mouse.key_code_for(super::key::MouseLocation::Pane);
        let name = code
            .map(|code| code.to_string())
            .unwrap_or_else(|| "Mouse".into());
        return Some((
            DecodedTtyKey {
                code,
                name,
                mouse: Some(mouse),
            },
            start + 5,
        ));
    }

    let (name, consumed) = match bytes.get(start).copied()? {
        b'O' => {
            let final_byte = *bytes.get(start + 1)?;
            (decode_ss3(final_byte)?.to_string(), start + 2)
        }
        b'[' => {
            let mut end = start + 1;
            while end < bytes.len()
                && (bytes[end].is_ascii_digit() || matches!(bytes[end], b';' | b':'))
            {
                end += 1;
            }
            let final_byte = *bytes.get(end)?;
            let params = std::str::from_utf8(&bytes[start + 1..end]).ok()?;
            (decode_csi(params, final_byte)?, end + 1)
        }
        byte => (meta_prompt_key(byte), start + 1),
    };
    let name = if meta { format!("M-{name}") } else { name };
    Some((
        DecodedTtyKey {
            code: parse_key_name(&name),
            name,
            mouse: None,
        },
        consumed,
    ))
}

fn resolve_mouse_key(
    decoded: &mut DecodedTtyKey,
    input: &mut MouseInputState,
    state: &Arc<Mutex<ServerState>>,
    target: &str,
    cols: u16,
    rows: u16,
    status_cache: &mut status::RenderCache,
) {
    let Some(event) = decoded.mouse.as_mut() else {
        return;
    };
    if let Ok(state) = state.lock() {
        let rendered = status_cache.render(&state, target, cols, rows);
        mouse::resolve_event(&state, target, rows, rendered, event);
    }
    input.observe(event, Instant::now());
    decoded.code = event.key_code();
    if let Some(code) = decoded.code {
        decoded.name = code.to_string();
    } else {
        decoded.name = "Mouse".into();
    }
}

/// Decode the key syntax accepted by tmux's `command-prompt -k`.
fn decode_prompt_key(bytes: &[u8]) -> Option<(String, usize)> {
    decode_tty_key(bytes).map(|(key, consumed)| (key.name, consumed))
}

/// The ambiguity delay tmux applies to an incomplete terminal key.
///
/// `escape-time=0` still gets a one millisecond timer in tmux
/// (`tty_keys_next`), so preserve that lower bound here. Invalid stored values
/// fall back to the modeled tmux default rather than turning into an unbounded
/// poll.
fn prompt_escape_delay(state: &Arc<Mutex<ServerState>>) -> Duration {
    let milliseconds = state
        .lock()
        .ok()
        .and_then(|st| st.server_options().get("escape-time")?.parse::<u64>().ok())
        .unwrap_or(10)
        .min(i32::MAX as u64)
        .max(1);
    Duration::from_millis(milliseconds)
}

/// Wait for more tty input until an already-established key deadline.
///
/// The deadline is created when the first incomplete bytes arrive and is not
/// extended by later partial reads. This matches tmux's per-client key timer
/// and prevents a fragmented CSI sequence from receiving a fresh full
/// `escape-time` for every byte.
fn poll_input_until(fd: RawFd, deadline: Instant) -> bool {
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return false;
        }
        let timeout = remaining
            .as_millis()
            .saturating_add(u128::from(remaining.subsec_nanos() % 1_000_000 != 0))
            .min(i32::MAX as u128) as libc::c_int;
        let mut pfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let result = unsafe { libc::poll(&mut pfd, 1, timeout) };
        if result > 0 {
            return true;
        }
        if result == 0 {
            return false;
        }
        if io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
            return false;
        }
    }
}

fn copy_mode_active(state: &Arc<Mutex<ServerState>>, target: &str) -> bool {
    state
        .lock()
        .ok()
        .is_some_and(|st| st.active_copy_state(target).is_some())
}

fn append_view_output(state: &Arc<Mutex<ServerState>>, target: &str, output: &[u8]) {
    if let Ok(mut state) = state.lock() {
        let _ = state.append_view_output(target, output);
    }
}

fn set_copy_mode_state(state: &Arc<Mutex<ServerState>>, target: &str, active: bool, page_up: bool) {
    if let Ok(mut st) = state.lock() {
        let _ = st.set_pane_mode(target, active.then_some("copy-mode"));
        if active && page_up {
            let vi = match st.option_for_target(target, "mode-keys") {
                Some(mode) => mode == "vi",
                None => super::options::mode_keys_default() == "vi",
            };
            let separators = st
                .option_for_target(target, "word-separators")
                .unwrap_or(" !\"#$%&'()*+,-./:;<=>?@[\\]^`{|}~")
                .to_string();
            let _ = st.copy_mode_command(target, "page-up", vi, &separators);
        }
    }
}

fn copy_table_name(state: &Arc<Mutex<ServerState>>, target: &str) -> &'static str {
    let mode = state.lock().ok().and_then(|st| {
        st.option_for_target(target, "mode-keys")
            .map(str::to_string)
    });
    let vi = match mode.as_deref() {
        Some(mode) => mode == "vi",
        None => super::options::mode_keys_default() == "vi",
    };
    if vi {
        "copy-mode-vi"
    } else {
        "copy-mode"
    }
}

/// Wait until either side of an attached client, its active pane, or its agent
/// status subscription has work.
/// Tty readiness needs no flag because the non-blocking input drain runs next.
fn wait_for_attach_events(
    imsg_fd: RawFd,
    input_fd: RawFd,
    output_fd: RawFd,
    prompt_fd: RawFd,
    render_fd: RawFd,
    status_fd: RawFd,
    timeout: i32,
) -> io::Result<(bool, bool, bool, bool, bool)> {
    let mut fds = [
        libc::pollfd {
            fd: imsg_fd,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: input_fd,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: output_fd,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: prompt_fd,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: render_fd,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: status_fd,
            events: libc::POLLIN,
            revents: 0,
        },
    ];
    let result = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, timeout) };
    if result >= 0 {
        return Ok((
            fds[0].revents != 0,
            fds[2].revents != 0,
            fds[3].revents != 0,
            fds[4].revents != 0,
            fds[5].revents != 0,
        ));
    }
    let error = io::Error::last_os_error();
    if error.kind() == io::ErrorKind::Interrupted {
        // Re-evaluate the absolute status deadline rather than restarting
        // the full relative poll timeout after every signal.
        return Ok((false, false, false, false, false));
    }
    Err(error)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ActiveWindowOutputKey {
    panes: Vec<(u32, u64)>,
    active: usize,
}

/// Return the active window's stable pane set and one notification subscription
/// shared by all panes the compositor displays.
fn active_window_output_subscription(
    state: &ServerState,
    session: &str,
) -> io::Result<(ActiveWindowOutputKey, OutputSubscription)> {
    let (panes, active) = state
        .active_window_pane_identities(session)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("no active window for session: {session}"),
            )
        })?;
    let subscription = state.subscribe_active_window_output(session)?;
    Ok((ActiveWindowOutputKey { panes, active }, subscription))
}

/// Replace `subscription` when another command client or pane reaping changed
/// the active window's pane set or selection. Returns true when the caller must
/// redraw and ignore readiness reported for the old platform wakeup.
fn refresh_active_window_output_subscription(
    state: &Arc<Mutex<ServerState>>,
    session: &str,
    subscribed_window: &mut ActiveWindowOutputKey,
    subscription: &mut OutputSubscription,
) -> io::Result<bool> {
    let st = state
        .lock()
        .map_err(|_| io::Error::other("state poisoned"))?;
    let (panes, active) = st.active_window_pane_identities(session).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("no active window for session: {session}"),
        )
    })?;
    let current = ActiveWindowOutputKey { panes, active };
    if current == *subscribed_window {
        return Ok(false);
    }
    let new_subscription = st.subscribe_active_window_output(session)?;
    *subscribed_window = current;
    *subscription = new_subscription;
    Ok(true)
}

/// The client's tty fds captured during identify.
#[derive(Debug)]
pub struct ClientTty {
    /// Stdin fd (typically the same underlying tty as stdout).
    pub stdin: Option<OwnedFd>,
    /// Stdout fd (the fd we render to).
    pub stdout: Option<OwnedFd>,
    /// The TERM the client advertised, if any.
    pub term: Option<String>,
    /// Typed terminfo entries sent by the tmux client during identify.
    pub terminfo: Vec<String>,
    /// Named terminal-feature bits sent by the tmux client during identify.
    pub features: u32,
    /// Client flags sent during identification, including UTF-8 support.
    pub flags: i64,
    /// The tty path advertised during client identification.
    pub tty_name: Option<String>,
    /// The client process id advertised during identification.
    pub client_pid: Option<i32>,
}

impl ClientTty {
    pub fn new() -> Self {
        ClientTty {
            stdin: None,
            stdout: None,
            term: None,
            terminfo: Vec::new(),
            features: 0,
            flags: 0,
            tty_name: None,
            client_pid: None,
        }
    }

    /// The fd we should use for rendering / reading input. Prefers stdout,
    /// falls back to stdin, matching tmux's `c->fd` which is typically stdout.
    pub fn render_fd(&self) -> Option<BorrowedFd<'_>> {
        if let Some(ref fd) = self.stdout {
            Some(fd.as_fd())
        } else if let Some(ref fd) = self.stdin {
            Some(fd.as_fd())
        } else {
            None
        }
    }

    pub fn input_fd(&self) -> Option<BorrowedFd<'_>> {
        if let Some(ref fd) = self.stdin {
            Some(fd.as_fd())
        } else if let Some(ref fd) = self.stdout {
            Some(fd.as_fd())
        } else {
            None
        }
    }
}

/// Return only a target supplied by the client. An omitted target must be
/// resolved with server state, not confused with an explicit `-t 0`.
fn explicit_target_session(args: &[String]) -> Option<String> {
    // Look for `-t` flag.
    if let Some(idx) = args.iter().position(|a| a == "-t") {
        if let Some(name) = args.get(idx + 1) {
            return Some(name.clone());
        }
    }
    // Positional after command name, skipping flags that take values.
    // For attach, only `-t` takes a value; other flags like `-d`, `-r` are
    // boolean and should be skipped without consuming next arg.
    let value_flags = ["-t"];
    let mut i = 1; // skip command name
    while i < args.len() {
        let a = &args[i];
        if a.starts_with('-') {
            if value_flags.contains(&a.as_str()) {
                i += 2;
                continue;
            } else {
                i += 1;
                continue;
            }
        } else {
            return Some(a.clone());
        }
    }
    None
}

fn is_tty(fd: RawFd) -> bool {
    unsafe { libc::isatty(fd) == 1 }
}

fn get_winsize(fd: RawFd) -> io::Result<(u16, u16)> {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws as *mut _) };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    let cols = if ws.ws_col == 0 { 80 } else { ws.ws_col };
    let rows = if ws.ws_row == 0 { 24 } else { ws.ws_row };
    Ok((cols, rows))
}

fn tty_start_sequence(terminal: &ResolvedTerm) -> Vec<u8> {
    let mut output = Vec::new();
    for name in ["smcup", "smkx", "clear", "cnorm"] {
        if let Some(value) = term::string_capability(terminal, name) {
            output.extend_from_slice(value);
        }
    }
    if terminal.capability("kmous").is_some() {
        output.extend_from_slice(b"\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?1005l");
    }
    if let Some(value) = term::string_capability(terminal, "Enbp") {
        output.extend_from_slice(value);
    }
    output
}

fn tty_stop_sequence(terminal: &ResolvedTerm, rows: u16) -> Vec<u8> {
    let mut output = term::expand_capability(
        terminal,
        "csr",
        &[
            term::CapabilityParameter::Number(0),
            term::CapabilityParameter::Number(i32::from(rows.saturating_sub(1))),
        ],
    )
    .unwrap_or_default();
    for name in ["sgr0", "rmkx", "clear", "cnorm"] {
        if let Some(value) = term::string_capability(terminal, name) {
            output.extend_from_slice(value);
        }
    }
    if terminal.capability("kmous").is_some() {
        output.extend_from_slice(b"\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?1005l");
    }
    for name in ["Dsbp", "Dsfcs", "Dseks", "Dsmg", "rmcup"] {
        if let Some(value) = term::string_capability(terminal, name) {
            output.extend_from_slice(value);
        }
    }
    output
}

/// Put `fd`'s terminal into raw mode, returning the previous `termios` so the
/// caller can restore it on detach.
///
/// This is the native counterpart to tmux's `tty_start_tty` (`tty-term.c` /
/// `tty.c`), which the tmux *server* runs on the client's tty before it starts
/// driving it. Without it the client's pty stays in its default canonical,
/// line-buffered mode: multi-byte input is only delivered on a newline, so a
/// bare prefix key like `C-b c` (no newline) sits in the line-discipline buffer
/// forever and the server never sees it. Stock tmux raws the tty, so raw mode is
/// required for the native engine
/// to observe keystrokes at all, prefix table included.
fn make_raw(fd: RawFd) -> io::Result<libc::termios> {
    // SAFETY: `termios` is a plain C struct; tcgetattr fills it fully.
    let mut old: libc::termios = unsafe { std::mem::zeroed() };
    if unsafe { libc::tcgetattr(fd, &mut old) } < 0 {
        return Err(io::Error::last_os_error());
    }
    let mut raw = old;
    // SAFETY: cfmakeraw only writes into `raw`.
    unsafe { libc::cfmakeraw(&mut raw) };
    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(old)
}

/// Restore a previously saved `termios` (best effort).
fn restore_termios(fd: RawFd, old: &libc::termios) {
    // SAFETY: `old` came from a prior tcgetattr on a still-open fd.
    unsafe {
        libc::tcsetattr(fd, libc::TCSANOW, old);
    }
}

/// RAII guard that restores the client tty to its pre-`make_raw` `termios` when
/// the attach scope unwinds by *any* path — a normal loop break, an early `?`
/// return (e.g. a failed `set_nonblock`), or a panic.
///
/// The interactive loop leaves the tty in raw mode (`cfmakeraw`, which clears
/// `OPOST`) for its whole lifetime, so a scope exit that skips the restore hands
/// the user back a terminal with no output post-processing: `\n` no longer maps
/// to CR-LF and every command staircases. The restore used to be a plain
/// statement after the loop, reachable only on the clean paths; this guard makes
/// it unconditional. The clean paths still restore explicitly (so the tty is
/// cooked *before* the detach/exit handshake writes to it) and then [`disarm`]
/// this guard so it doesn't fire twice.
///
/// [`disarm`]: TermiosGuard::disarm
struct TermiosGuard {
    fd: RawFd,
    saved: Option<libc::termios>,
}

impl TermiosGuard {
    /// Mark the terminal as already restored by the normal path, so `drop` is a
    /// no-op.
    fn disarm(&mut self) {
        self.saved = None;
    }
}

impl Drop for TermiosGuard {
    fn drop(&mut self) {
        if let Some(saved) = self.saved {
            restore_termios(self.fd, &saved);
        }
    }
}

fn set_nonblock(fd: RawFd) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    let rc = unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn set_blocking(fd: RawFd) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    let rc = unsafe { libc::fcntl(fd, libc::F_SETFL, flags & !libc::O_NONBLOCK) };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Send an error result back via the file protocol, matching how tmux reports
/// command errors to a command client. This is used when attach fails early
/// (e.g. "can't find session", "not a terminal") so the conformance harness
/// sees a recognized command that failed for a tty reason, not an "unknown
/// command" rejection.
fn send_error_and_exit<R, W>(
    reader: &mut R,
    writer: &mut W,
    msg: &str,
    exit_code: i32,
) -> io::Result<()>
where
    R: FrameReader,
    W: FrameWriter,
{
    // Reuse the file-protocol helper from protocol.rs logic: open stream 2
    // (stderr), wait for ready, write, close, then MSG_EXIT.
    const FD_STDERR: i32 = 2;
    writer.send(Frame::new(Message::WriteOpen {
        stream: 2,
        fd: FD_STDERR,
        flags: 0,
        path: Vec::new(),
    }))?;

    // Wait for WriteReady with a timeout so we don't hang if client is gone.
    // We use the same blocking recv but with SO_RCVTIMEO set by caller if needed.
    loop {
        let frame = match reader.recv() {
            Ok(f) => f,
            Err(e)
                if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut =>
            {
                // Client didn't ack in time; give up and just send exit.
                break;
            }
            Err(_) => break,
        };
        if let Message::WriteReady { stream: 2, .. } = frame.msg {
            break;
        }
    }

    writer.send(Frame::new(Message::Write {
        stream: 2,
        data: msg.as_bytes().to_vec(),
    }))?;
    writer.send(Frame::new(Message::WriteClose { stream: 2 }))?;
    writer.send(Frame::new(Message::Exit(Some(exit_code))))?;
    Ok(())
}

/// The main attach entry point, called from `protocol::handle` when it sees
/// `attach-session` or `attach`.
///
/// `client_tty` must contain the fds captured during identify. If those fds
/// are not ttys (e.g. the harness passes `/dev/null`), we return a tty-failure
/// error via the file protocol so the capability matrix counts this as
/// `Supported` ("recognized; failed for lack of a tty") rather than
/// `Unsupported`.
pub fn handle_attach<R, W>(
    args: &[String],
    client_tty: ClientTty,
    state: &Arc<Mutex<ServerState>>,
    hub: &StatusHub,
    context: &command::ClientContext,
    reader: &mut R,
    writer: &mut W,
) -> io::Result<()>
where
    R: AttachFrameReader,
    W: FrameWriter,
{
    let supplied_target = explicit_target_session(args);
    let target = {
        let mut st = state
            .lock()
            .map_err(|_| io::Error::other("state poisoned"))?;
        match attach_target(supplied_target, &mut st, context) {
            Ok(target) => target,
            Err(message) => {
                drop(st);
                return send_error_and_exit(reader, writer, &message, 1);
            }
        }
    };

    // Check session existence first, before tty checks, to match tmux ordering:
    // "can't find session" takes precedence over "not a terminal".
    {
        let st = state
            .lock()
            .map_err(|_| io::Error::other("state poisoned"))?;
        if st.find(&target).is_none() {
            let msg = format!("can't find session: {target}\n");
            return send_error_and_exit(reader, writer, &msg, 1);
        }
    }

    run_attach(&target, client_tty, state, hub, context, reader, writer)
}

pub(crate) fn attach_target(
    supplied_target: Option<String>,
    state: &mut ServerState,
    context: &command::ClientContext,
) -> Result<String, String> {
    if let Some(target) = supplied_target {
        return Ok(target);
    }
    if let Some(session) = state.sessions().last() {
        return Ok(session.name.clone());
    }
    if state.initial_attach_pending() {
        return command::new_session_for_attach(&[], state, context);
    }
    Err("no sessions\n".to_string())
}

/// The interactive `new-session` (and bare-`tmux`) path: create — or, with `-A`,
/// find-or-create — the session, then attach to it. This is what makes
/// `tmux -S hmux.sock` (no command) start a shell in a window under the native
/// engine, matching stock tmux.
///
/// Session creation reuses [`command::new_session_for_attach`] so an interactive
/// create is identical to a `new-session -d` create; only the follow-up differs
/// (attach the client instead of exiting). A creation error (e.g. a duplicate
/// name without `-A`) is reported over the file protocol just like a command
/// client would see it.
pub fn handle_new_session<R, W>(
    args: &[String],
    client_tty: ClientTty,
    state: &Arc<Mutex<ServerState>>,
    hub: &StatusHub,
    context: &command::ClientContext,
    reader: &mut R,
    writer: &mut W,
) -> io::Result<()>
where
    R: AttachFrameReader,
    W: FrameWriter,
{
    let target = {
        let mut st = state
            .lock()
            .map_err(|_| io::Error::other("state poisoned"))?;
        match command::new_session_for_attach(args, &mut st, context) {
            Ok(name) => name,
            Err(msg) => {
                drop(st);
                return send_error_and_exit(reader, writer, &msg, 1);
            }
        }
    };
    run_attach(&target, client_tty, state, hub, context, reader, writer)
}

/// Drive an interactive attach to an already-resolved, existing `target`
/// session: validate the client's tty fds, then run the compositor loop
/// (render active pane → tty, forward keystrokes → pane, handle resize/detach).
/// Shared by [`handle_attach`] and [`handle_new_session`].
fn run_attach<R, W>(
    target: &str,
    client_tty: ClientTty,
    state: &Arc<Mutex<ServerState>>,
    hub: &StatusHub,
    context: &command::ClientContext,
    reader: &mut R,
    writer: &mut W,
) -> io::Result<()>
where
    R: AttachFrameReader,
    W: FrameWriter,
{
    // If we have no tty fds at all, or they are not ttys, report the same error
    // tmux does: "open terminal failed: not a terminal". This is what makes the
    // conformance matrix show OK for native even when the harness uses /dev/null.
    let render_fd_borrowed = client_tty.render_fd();
    let input_fd_borrowed = client_tty.input_fd();

    let (render_raw, input_raw) = match (render_fd_borrowed, input_fd_borrowed) {
        (Some(r), Some(i)) => (r.as_raw_fd(), i.as_raw_fd()),
        (Some(r), None) => (r.as_raw_fd(), r.as_raw_fd()),
        (None, Some(i)) => (i.as_raw_fd(), i.as_raw_fd()),
        (None, None) => {
            let msg = "open terminal failed: not a terminal\n";
            return send_error_and_exit(reader, writer, msg, 1);
        }
    };

    // Dup the fds so we own them for the duration of attach, independent of
    // ClientTty's OwnedFds which will be dropped after this function if we move.
    // We need owned copies because we will set non-blocking and use them in the
    // loop.
    let render_fd_owned = unsafe {
        let dup = libc::dup(render_raw);
        if dup < 0 {
            let msg = "open terminal failed: not a terminal\n";
            return send_error_and_exit(reader, writer, msg, 1);
        }
        OwnedFd::from_raw_fd(dup)
    };
    let input_fd_owned = if input_raw == render_raw {
        // Same underlying fd, dup again for input side to keep lifetimes simple.
        unsafe {
            let dup = libc::dup(input_raw);
            if dup < 0 {
                return send_error_and_exit(
                    reader,
                    writer,
                    "open terminal failed: not a terminal\n",
                    1,
                );
            }
            OwnedFd::from_raw_fd(dup)
        }
    } else {
        unsafe {
            let dup = libc::dup(input_raw);
            if dup < 0 {
                return send_error_and_exit(
                    reader,
                    writer,
                    "open terminal failed: not a terminal\n",
                    1,
                );
            }
            OwnedFd::from_raw_fd(dup)
        }
    };

    if !is_tty(render_fd_owned.as_raw_fd()) || !is_tty(input_fd_owned.as_raw_fd()) {
        let msg = "open terminal failed: not a terminal\n";
        return send_error_and_exit(reader, writer, msg, 1);
    }

    // From here on we have a real tty. Enter interactive attach.
    let (mut cols, mut rows) = get_winsize(render_fd_owned.as_raw_fd()).unwrap_or((80, 24));
    let (prompt_registry, render_registry, mut session_id) = {
        let st = state
            .lock()
            .map_err(|_| io::Error::other("state poisoned"))?;
        let session_id = st.session_id(target).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {target}"),
            )
        })?;
        (
            st.client_prompt_registry(),
            st.client_render_registry(),
            session_id,
        )
    };
    let prompt_attachment = prompt_registry.attach(
        client_tty.tty_name.clone().unwrap_or_default(),
        client_tty.client_pid,
        session_id,
    )?;
    let render_name = client_tty
        .tty_name
        .clone()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| format!("client-{}", client_tty.client_pid.unwrap_or_default()));
    let render_attachment = render_registry.attach_with_details(
        session_id,
        render_name,
        client_tty.term.clone().unwrap_or_default(),
        client_tty.client_pid,
        cols,
        rows,
        String::new(),
        false,
        false,
    )?;
    // Keep the attach anchored to the session's stable identity. Session names
    // are mutable (`rename-session`), but tmux clients remain attached across a
    // rename and subsequent prefix commands must still target the same session.
    let mut stable_target = format!("${session_id}");
    let target = stable_target.as_str();
    let mut attached_context = context.clone();
    attached_context.current_session_id = Some(session_id);
    attached_context.wait_for_interactions = false;

    // Get initial window size from the tty, then reserve rows for the status
    // line and resize the session/pane into what's left — exactly as tmux sizes
    // a window to `client rows - status lines`.
    let terminal_identity = TerminalIdentity::new(
        client_tty.term.clone().unwrap_or_default(),
        client_tty.terminfo.clone(),
        client_tty.features,
        context.env("COLORTERM").map(str::to_string),
    )
    .with_utf8(client_tty.flags & 0x10000 != 0);
    let (mut status_h, status_interval, mut terminal) = {
        let st = state
            .lock()
            .map_err(|_| io::Error::other("state poisoned"))?;
        (
            status::height(&st, target),
            status::interval(&st, target),
            ResolvedTerm::resolve(terminal_identity, st.server_options().iter_effective()),
        )
    };
    if let Some(cause) = terminal.validation_error() {
        return send_error_and_exit(reader, writer, &format!("{cause}\n"), 1);
    }
    render_attachment.update_terminal(&terminal);
    // tmux sends MSG_READY only after the outer terminal has been validated.
    writer.send(Frame::new(Message::Ready))?;
    let mut status_timer = StatusTimer::new(status_interval, Instant::now());
    let mut status_cache = status::RenderCache::for_client(status::ClientContext {
        term: (!terminal.name().is_empty()).then(|| terminal.name().to_string()),
        tty: client_tty.tty_name.clone(),
        pid: client_tty.client_pid,
        cwd: context.cwd.clone(),
        environment: context.environment.clone(),
        ..status::ClientContext::default()
    });
    let agent_status_subscription = hub.subscribe()?;
    status_cache.update_agents(hub.snapshot());
    let mut pane_rows = rows.saturating_sub(status_h).max(1);
    {
        let mut st = state
            .lock()
            .map_err(|_| io::Error::other("state poisoned"))?;
        let _ = st.resize_session(target, cols, pane_rows);
    }

    // Put the client's tty into raw mode (as tmux's server does before driving a
    // client's terminal). Both dup'd fds share one open terminal device, so a
    // single tcsetattr covers input and render. Restored on detach below.
    let saved_termios = make_raw(input_fd_owned.as_raw_fd()).ok();
    // Guarantee the tty is put back even if we leave this function early (a `?`
    // below, a panic, or an abrupt client disconnect that breaks the loop): a
    // scope exit that skips the restore leaves the terminal raw (no `OPOST`) and
    // the user's shell staircases. The clean paths disarm this after restoring
    // explicitly. See [`TermiosGuard`].
    let mut termios_guard = TermiosGuard {
        fd: input_fd_owned.as_raw_fd(),
        saved: saved_termios,
    };

    // Set tty fds non-blocking for our poll loop.
    set_nonblock(input_fd_owned.as_raw_fd())?;
    set_nonblock(render_fd_owned.as_raw_fd())?;

    let imsg_fd = reader.as_raw_fd();

    // Initialize the outer tty from its resolved terminfo profile.
    let tty_start = tty_start_sequence(&terminal);
    let _ = write_all(render_fd_owned.as_raw_fd(), &tty_start);
    if state
        .lock()
        .ok()
        .is_some_and(|st| st.option_for_target(target, "mouse") == Some("on"))
    {
        let _ = write_all(
            render_fd_owned.as_raw_fd(),
            b"\x1b[?1000h\x1b[?1002h\x1b[?1006h",
        );
    }
    let mut last_render: Vec<u8> = Vec::new();
    let (mut subscribed_window, mut output_subscription) = {
        let st = state
            .lock()
            .map_err(|_| io::Error::other("state poisoned"))?;
        active_window_output_subscription(&st, target)?
    };
    // Track what DECTCEM state we have actually sent to the outer terminal.
    // Synchronized terminals can omit a frame's defensive hide/show pair
    // because intermediate cursor movement is never presented. Without
    // synchronized output, the pair must remain around a repaint or the
    // hardware cursor visibly walks through each row as it is redrawn.
    let mut output_cursor_visible: Option<bool> = None;
    // tmux remembers the last expanded title per client and only writes the
    // outer terminal's title capabilities when that value changes.
    let mut last_title: Option<String> = None;
    // Frames are drawn in place (no per-frame `\x1b[2J`) to avoid flicker; a
    // one-shot full clear is prepended only when the whole screen must be reset:
    // the first paint, a resize, or a layout change that swaps the active pane.
    // Otherwise stale cells from a shrunk/replaced screen could linger.
    let mut force_clear = true;
    let mut prefix_pending = false;
    let mut mouse_input = MouseInputState::default();
    // A pending `confirm-before` prompt (`C-b x` / `C-b &`), client-local. While
    // set, the status line shows `prompt` and every key answers it (`y`/`Y` runs
    // `action`, anything else cancels) instead of reaching the pane — mirroring
    // tmux's `status_prompt` confirm flow.
    let mut confirm: Option<ActiveConfirm> = None;
    let mut command_prompt: Option<CommandPrompt> = None;
    let mut active_overlay: Option<ActiveOverlay> = None;
    let mut status_message: Option<(String, Instant)> = None;
    let mut should_exit = false;
    // Set when the loop exits because the *user* asked to detach (`C-b d`) rather
    // than because the client/connection went away. Only a user detach runs the
    // graceful MSG_DETACH handshake at the end.
    let mut detach_requested = false;
    // Set when the attached session disappeared because its last pane exited.
    // This uses tmux's normal client-exit handshake and lets `exit-empty`
    // terminate the outer server.
    let mut session_ended = false;
    let mut locked = false;
    let mut suspended = false;
    let mut key_prompt_buf = Vec::new();
    let mut key_prompt_deadline = None;
    let mut switch_to = None;
    let mut injected_input = VecDeque::new();

    // Optional keystroke→screen latency probe (off unless HMUX_LATENCY is set).
    // It times the wholly in-daemon path so the user can tell hmux-side latency
    // apart from network lag; see `latmon`.
    let mut latmon = LatMon::new(format!("sess={target}"));

    // Main attach loop.
    loop {
        if let Some(new_session_id) = switch_to.take() {
            session_id = new_session_id;
            stable_target = format!("${session_id}");
            attached_context.current_session_id = Some(session_id);
            let target = stable_target.as_str();
            if let Ok(mut st) = state.lock() {
                status_h = status::height(&st, target);
                pane_rows = rows.saturating_sub(status_h).max(1);
                let _ = st.resize_session(target, cols, pane_rows);
                status_timer.configure(status::interval(&st, target), Instant::now());
            }
            status_cache.invalidate();
            last_render.clear();
            force_clear = true;
        }
        let target = stable_target.as_str();
        if should_exit {
            break;
        }

        // Reap pty children before waiting for more client traffic. With
        // remain-on-exit off (the default), an exited last pane removes its
        // window and session; an empty server also requests listener shutdown.
        let target_exists = match state.lock() {
            Ok(mut st) => {
                if st.reap_exited_panes() {
                    let _ = st.resize_session(target, cols, pane_rows);
                    last_render.clear();
                    force_clear = true;
                    status_cache.invalidate();
                }
                st.find(target).is_some()
            }
            Err(_) => false,
        };
        if !target_exists {
            session_ended = true;
            break;
        }

        // Reaping an exited pane can select a survivor without going through
        // the prefix-key path. Refresh before blocking so the next output wake
        // always belongs to the pane we are about to compose.
        if refresh_active_window_output_subscription(
            state,
            target,
            &mut subscribed_window,
            &mut output_subscription,
        )? {
            last_render.clear();
            force_clear = true;
            status_cache.invalidate();
        }

        let now = Instant::now();
        let poll_timeout = minimum_poll_timeout(
            status_timer.poll_timeout(now),
            deadline_poll_timeout(status_message.as_ref().map(|(_, deadline)| *deadline), now),
        );
        let poll_timeout = minimum_poll_timeout(
            poll_timeout,
            active_overlay
                .as_ref()
                .map(|overlay| overlay.poll_timeout(now))
                .unwrap_or_else(|| {
                    if state.lock().ok().is_some_and(|st| {
                        st.active_mode_view(target)
                            .is_some_and(|view| view.kind == ModeKind::Clock)
                    }) {
                        1000
                    } else {
                        -1
                    }
                }),
        );
        let (control_ready, mut output_ready, prompt_ready, render_ready, agent_status_ready) =
            if reader.has_buffered_frame() {
                (true, false, false, false, false)
            } else {
                wait_for_attach_events(
                    imsg_fd,
                    if locked || suspended {
                        -1
                    } else {
                        input_fd_owned.as_raw_fd()
                    },
                    if locked || suspended {
                        -1
                    } else {
                        output_subscription.as_raw_fd()
                    },
                    if locked || suspended {
                        -1
                    } else {
                        prompt_attachment.as_raw_fd()
                    },
                    render_attachment.as_raw_fd(),
                    agent_status_subscription.as_raw_fd(),
                    poll_timeout,
                )?
            };
        let now = Instant::now();
        let agent_status_changed = if agent_status_ready {
            agent_status_subscription.drain();
            status_cache.update_agents(hub.snapshot())
        } else {
            false
        };
        let status_timer_ready = status_timer.take_expired(now);
        let overlay_tick = active_overlay.is_some();
        let mut overlay_exit = 0;
        let overlay_expired = match active_overlay.as_mut() {
            Some(ActiveOverlay::DisplayPanes { deadline, .. }) => *deadline <= now,
            Some(ActiveOverlay::Popup {
                request,
                pane,
                exit_status,
                ..
            }) => {
                if pane.has_exited() {
                    if exit_status.is_none() {
                        *exit_status = pane.try_wait();
                    }
                    if let Some(exit) = *exit_status {
                        overlay_exit = exit;
                        request.close_on_exit || (request.close_on_success && exit == 0)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        };
        if overlay_expired {
            if let Some(mut overlay) = active_overlay.take() {
                let mut result = command::CommandResult::ok("");
                result.exit = overlay_exit;
                overlay.complete(result, false);
            }
            last_render.clear();
            force_clear = true;
        }
        let message_expired = status_message
            .as_ref()
            .is_some_and(|(_, deadline)| *deadline <= now);
        if message_expired {
            status_message = None;
            last_render.clear();
        }
        if status_timer_ready {
            status_cache.invalidate();
        }
        let render_invalidation = if render_ready {
            render_attachment.take()
        } else {
            super::state::RenderInvalidation::default()
        };
        if render_ready {
            for message in render_attachment.take_messages() {
                status_message = Some((
                    message.text,
                    Instant::now() + Duration::from_millis(message.duration_ms),
                ));
                confirm = None;
                last_render.clear();
                force_clear = true;
            }
            if let Some(action) = render_attachment.take_action() {
                match action {
                    ClientAction::Lock(command) if !locked => {
                        let stop = tty_stop_sequence(&terminal, rows);
                        let _ = write_all(render_fd_owned.as_raw_fd(), &stop);
                        output_cursor_visible = None;
                        if let Some(ref saved) = saved_termios {
                            restore_termios(input_fd_owned.as_raw_fd(), saved);
                        }
                        writer.send(Frame::new(Message::Lock(command)))?;
                        locked = true;
                        last_render.clear();
                        force_clear = true;
                    }
                    ClientAction::Suspend if !suspended => {
                        let stop = tty_stop_sequence(&terminal, rows);
                        let _ = write_all(render_fd_owned.as_raw_fd(), &stop);
                        output_cursor_visible = None;
                        if let Some(ref saved) = saved_termios {
                            restore_termios(input_fd_owned.as_raw_fd(), saved);
                        }
                        writer.send(Frame::new(Message::Suspend))?;
                        suspended = true;
                        last_render.clear();
                        force_clear = true;
                    }
                    ClientAction::Detach => {
                        detach_requested = true;
                        break;
                    }
                    ClientAction::Switch(new_session_id) => {
                        switch_to = Some(new_session_id);
                        continue;
                    }
                    ClientAction::Keys(keys) if !locked && !suspended => {
                        injected_input.extend(keys);
                    }
                    ClientAction::SetSelection(data) => {
                        let encoded = base64_encode(&data);
                        if let Some(sequence) = term::expand_capability(
                            &terminal,
                            "Ms",
                            &[
                                term::CapabilityParameter::String(""),
                                term::CapabilityParameter::String(&encoded),
                            ],
                        ) {
                            let _ = write_all(render_fd_owned.as_raw_fd(), &sequence);
                        }
                    }
                    ClientAction::Overlay { request, reply } => {
                        if matches!(request, OverlayRequest::Clear) {
                            if let Some(mut overlay) = active_overlay.take() {
                                overlay.complete(command::CommandResult::ok(""), false);
                            }
                            if let Some(reply) = reply {
                                let _ = reply.send(super::state::PromptCompletion {
                                    stdout: String::new(),
                                    stderr: String::new(),
                                    exit: 0,
                                    inserted: false,
                                });
                            }
                        } else if active_overlay.is_some() {
                            if let Some(reply) = reply {
                                let _ = reply.send(super::state::PromptCompletion {
                                    stdout: String::new(),
                                    stderr: String::new(),
                                    exit: 0,
                                    inserted: false,
                                });
                            }
                        } else {
                            active_overlay =
                                ActiveOverlay::from_request(request, reply, cols, rows)
                                    .ok()
                                    .flatten();
                        }
                        last_render.clear();
                        force_clear = true;
                    }
                    ClientAction::Confirm {
                        prompt,
                        command,
                        confirm_key,
                        default_yes,
                        reply,
                    } => {
                        confirm = Some(ActiveConfirm {
                            prompt,
                            action: ConfirmAction::Command(command),
                            confirm_key,
                            default_yes,
                            reply,
                        });
                        last_render.clear();
                        force_clear = true;
                    }
                    ClientAction::Lock(_) => {}
                    ClientAction::Suspend => {}
                    ClientAction::Keys(_) => {}
                }
            }
        }
        if render_invalidation.contains(super::state::RenderInvalidation::SESSION_GONE) {
            session_ended = true;
            break;
        }
        if !render_invalidation.is_empty() {
            status_cache.invalidate();
        }
        if render_invalidation.contains(super::state::RenderInvalidation::RESET_MODE)
            || render_invalidation.contains(super::state::RenderInvalidation::MODE)
        {
            last_render.clear();
        }
        if render_invalidation.contains(super::state::RenderInvalidation::STATUS) {
            let mut st = state
                .lock()
                .map_err(|_| io::Error::other("state poisoned"))?;
            if render_invalidation.contains(super::state::RenderInvalidation::TERMINAL) {
                terminal.refresh(st.server_options().iter_effective());
                render_attachment.update_terminal(&terminal);
            }
            status_timer.configure(status::interval(&st, target), Instant::now());
            let new_status_h = status::height(&st, target);
            if new_status_h != status_h {
                status_h = new_status_h;
                pane_rows = rows.saturating_sub(status_h).max(1);
                let _ = st.resize_session(target, cols, pane_rows);
                last_render.clear();
                force_clear = true;
            }
        }
        if prompt_ready && command_prompt.is_none() {
            if let Some(external) = prompt_attachment.take_command_prompt() {
                let args = external.args().to_vec();
                match CommandPrompt::new(args, Some(external), state, hub, &attached_context) {
                    Ok(mut prompt) => {
                        if !prompt.spec.no_freeze {
                            prompt.frozen_frame = Some(last_render.clone());
                        }
                        prompt.initial_incremental(state, hub, &attached_context);
                        command_prompt = Some(prompt);
                    }
                    Err(_) => {}
                }
                last_render.clear();
            }
        }
        // An external command connection may switch windows while this thread
        // is blocked in poll. Tty input or an old-pane notification wakes us;
        // replace the stale subscription before attributing output or sending
        // that input to the newly active pane.
        if refresh_active_window_output_subscription(
            state,
            target,
            &mut subscribed_window,
            &mut output_subscription,
        )? {
            output_ready = false;
            last_render.clear();
            force_clear = true;
            status_cache.invalidate();
        }
        if output_ready {
            output_subscription.drain();
            status_cache.invalidate();
            // If this wake came from the active pane, mark its latest output so
            // the upcoming compose is timed against the keystroke that caused
            // it. Background-pane wakes have no newer active timestamp.
            latmon.on_output(output_subscription.last_output_at());
        }

        // 1. Handle imsg control messages (resize, detach) when poll says that
        // reading cannot block.
        if control_ready {
            match reader.recv() {
                Ok(frame) => {
                    if frame.version != PROTOCOL_VERSION {
                        let _ = writer.send(Frame::new(Message::Version));
                        break;
                    }
                    match frame.msg {
                        Message::Resize => {
                            if let Ok((new_cols, new_rows)) =
                                get_winsize(render_fd_owned.as_raw_fd())
                            {
                                if new_cols != cols || new_rows != rows {
                                    cols = new_cols;
                                    rows = new_rows;
                                    if let Some(overlay) = active_overlay.as_mut() {
                                        overlay.resize(cols, rows);
                                    }
                                    render_attachment.update_size(cols, rows);
                                    if let Ok(mut st) = state.lock() {
                                        status_h = status::height(&st, target);
                                        pane_rows = rows.saturating_sub(status_h).max(1);
                                        let _ = st.resize_session(target, cols, pane_rows);
                                    }
                                    // Force a full re-render on resize: dimensions
                                    // changed, so clear once to drop any stale cells.
                                    last_render.clear();
                                    force_clear = true;
                                    status_cache.invalidate();
                                }
                            }
                        }
                        Message::Unlock if locked => {
                            let _ = make_raw(input_fd_owned.as_raw_fd());
                            let start = tty_start_sequence(&terminal);
                            let _ = write_all(render_fd_owned.as_raw_fd(), &start);
                            output_cursor_visible = None;
                            if state.lock().ok().is_some_and(|st| {
                                st.option_for_target(target, "mouse") == Some("on")
                            }) {
                                let _ = write_all(
                                    render_fd_owned.as_raw_fd(),
                                    b"\x1b[?1000h\x1b[?1002h\x1b[?1006h",
                                );
                            }
                            locked = false;
                            last_render.clear();
                            force_clear = true;
                            status_cache.invalidate();
                        }
                        Message::Wakeup if suspended => {
                            let _ = make_raw(input_fd_owned.as_raw_fd());
                            let start = tty_start_sequence(&terminal);
                            let _ = write_all(render_fd_owned.as_raw_fd(), &start);
                            output_cursor_visible = None;
                            if state.lock().ok().is_some_and(|st| {
                                st.option_for_target(target, "mouse") == Some("on")
                            }) {
                                let _ = write_all(
                                    render_fd_owned.as_raw_fd(),
                                    b"\x1b[?1000h\x1b[?1002h\x1b[?1006h",
                                );
                            }
                            suspended = false;
                            last_render.clear();
                            force_clear = true;
                            status_cache.invalidate();
                        }
                        Message::Detach(_) | Message::DetachKill(_) => {
                            // A server-driven detach (rare on the inbound path): run
                            // the graceful handshake below, like a `C-b d` detach.
                            detach_requested = true;
                            break;
                        }
                        Message::Exit(_) | Message::Shutdown => {
                            break;
                        }
                        _ => {
                            // Ignore other control frames while attached.
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(_) => {
                    // Treat as detach on error.
                    break;
                }
            }
        }

        if locked || suspended {
            continue;
        }

        // 2. Relay terminal queries which Ghostty consumed from pane output.
        //    The outer terminal's reply is read immediately below and forwarded
        //    through the ordinary pane-input path. In particular, Neovim sends
        //    an OSC 11 default-background request followed by a CSI 5n status
        //    request; it needs both the RGB and CSI 0n replies.
        let terminal_queries = state
            .lock()
            .ok()
            .and_then(|st| st.take_active_pane_terminal_queries(target).ok())
            .unwrap_or_default();
        for query in terminal_queries {
            let _ = write_all(render_fd_owned.as_raw_fd(), &query);
        }

        // 3. Read input from client tty, interpreting tmux's prefix key table
        //    and forwarding everything else to the active pane.
        let mut input_buf = [0u8; 1024];
        let mut force_render = false;
        // Bytes forwarded to the pane's pty this iteration (real keystrokes, not
        // prefix-table navigation), used to stamp keystroke latency below.
        let mut forwarded = PaneInputStats::default();
        let mut first_forward_at = None;
        // Keep plain bytes across immediately adjacent tty reads. Besides
        // reducing PTY writes, this preserves compound terminal replies such as
        // OSC 11 followed by CSI 0n when they straddle read boundaries.
        let mut forward_buf: Vec<u8> = Vec::with_capacity(input_buf.len());
        // A key prompt may consume only the front logical key from a tty read.
        // Replay its suffix through this same loop so prefix/copy/passthrough
        // handling remains identical to input received by a later read.
        let mut replay_input = Vec::new();
        let mut replay_forward_unbound = true;
        let mut waited_for_terminal_reply_tail = false;
        loop {
            let (replayed, forward_unbound) = if replay_input.is_empty() {
                if let Some(key) = injected_input.pop_front() {
                    (key.bytes, key.forward_unbound)
                } else {
                    (Vec::new(), true)
                }
            } else {
                (std::mem::take(&mut replay_input), replay_forward_unbound)
            };
            let n = if replayed.is_empty() {
                unsafe {
                    libc::read(
                        input_fd_owned.as_raw_fd(),
                        input_buf.as_mut_ptr() as *mut libc::c_void,
                        input_buf.len(),
                    )
                }
            } else {
                replayed.len() as isize
            };
            if n < 0 {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::WouldBlock {
                    if command_prompt
                        .as_ref()
                        .is_some_and(|prompt| prompt.spec.key)
                        && !key_prompt_buf.is_empty()
                    {
                        let decoded = decode_prompt_key(&key_prompt_buf);
                        let could_be_terminal_key = decoded.is_none()
                            && key_prompt_buf.starts_with(b"\x1b")
                            && matches!(key_prompt_buf.get(1), Some(b'[' | b'O'));
                        if could_be_terminal_key {
                            let deadline = *key_prompt_deadline
                                .get_or_insert_with(|| Instant::now() + prompt_escape_delay(state));
                            if poll_input_until(input_fd_owned.as_raw_fd(), deadline) {
                                continue;
                            }
                        }
                        let decoded = decoded.or_else(|| {
                            (key_prompt_buf.len() >= 2 && key_prompt_buf[0] == 0x1b)
                                .then(|| (meta_prompt_key(key_prompt_buf[1]), 2))
                        });
                        if let Some((key, consumed)) = decoded {
                            handle_command_prompt_key(
                                &mut command_prompt,
                                &key,
                                state,
                                hub,
                                &attached_context,
                            );
                            replay_input.extend_from_slice(&key_prompt_buf[consumed..]);
                            replay_forward_unbound = forward_unbound;
                            key_prompt_buf.clear();
                            key_prompt_deadline = None;
                            force_render = true;
                            if !replay_input.is_empty() {
                                continue;
                            }
                        }
                    }
                    let is_partial_terminal_reply = forward_buf.starts_with(b"\x1b]")
                        && !forward_buf.windows(4).any(|bytes| bytes == b"\x1b[0n");
                    if is_partial_terminal_reply && !waited_for_terminal_reply_tail {
                        waited_for_terminal_reply_tail = true;
                        let mut pfd = libc::pollfd {
                            fd: input_fd_owned.as_raw_fd(),
                            events: libc::POLLIN,
                            revents: 0,
                        };
                        if unsafe { libc::poll(&mut pfd, 1, 2) } > 0 {
                            continue;
                        }
                    }
                    break;
                } else {
                    should_exit = true;
                    break;
                }
            } else if n == 0 {
                // EOF on tty: client closed.
                should_exit = true;
                break;
            }

            // Feed the chunk through the prefix state machine byte by byte. Plain
            // bytes are buffered and flushed to the active pane in order; a prefix
            // (`Ctrl-b`) consumes the next byte as a key-table command. The
            // `prefix_pending` flag lives outside the read loop, so a prefix at
            // the end of one chunk pairs with the command key in the next one —
            // exactly how a user types `C-b` then `c`.
            let read_data = if replayed.is_empty() {
                &input_buf[..n as usize]
            } else {
                replayed.as_slice()
            };
            prompt_attachment.note_activity();
            let mut prompt_tail = None;
            if command_prompt
                .as_ref()
                .is_some_and(|prompt| prompt.spec.key)
            {
                key_prompt_buf.extend_from_slice(read_data);
                if let Some((key, consumed)) = decode_prompt_key(&key_prompt_buf) {
                    handle_command_prompt_key(
                        &mut command_prompt,
                        &key,
                        state,
                        hub,
                        &attached_context,
                    );
                    prompt_tail = Some(key_prompt_buf[consumed..].to_vec());
                    key_prompt_buf.clear();
                    key_prompt_deadline = None;
                    force_render = true;
                } else if key_prompt_buf.starts_with(b"\x1b")
                    && matches!(key_prompt_buf.get(1), Some(b'[' | b'O'))
                    && key_prompt_deadline.is_none()
                {
                    key_prompt_deadline = Some(Instant::now() + prompt_escape_delay(state));
                }
                if prompt_tail.as_ref().is_none_or(Vec::is_empty) {
                    continue;
                }
            }
            let data = prompt_tail.as_deref().unwrap_or(read_data);
            key_prompt_buf.clear();
            key_prompt_deadline = None;
            let mut i = 0;
            while i < data.len() {
                if active_overlay.is_some() {
                    let start = i;
                    let (decoded, consumed) = decode_tty_key(&data[i..]).unwrap_or_else(|| {
                        (
                            DecodedTtyKey {
                                name: plain_prompt_key(data[i]),
                                code: Some(key_from_byte(data[i])),
                                mouse: None,
                            },
                            1,
                        )
                    });
                    i += consumed;
                    let mut close = false;
                    let mut close_exit = 0;
                    let mut selected_command = None;
                    match active_overlay.as_mut().expect("overlay checked") {
                        ActiveOverlay::Menu {
                            request, selected, ..
                        } => match decoded.name.as_str() {
                            "q" | "Escape" | "C-c" => close = true,
                            "Up" | "k" => *selected = selected.saturating_sub(1),
                            "Down" | "j" => {
                                *selected =
                                    (*selected + 1).min(request.items.len().saturating_sub(1))
                            }
                            "Enter" => {
                                selected_command = request
                                    .items
                                    .get(*selected)
                                    .map(|item| item.command.clone());
                                close = true;
                            }
                            key => {
                                if let Some(item) =
                                    request.items.iter().find(|item| item.key == key)
                                {
                                    selected_command = Some(item.command.clone());
                                    close = true;
                                }
                            }
                        },
                        ActiveOverlay::Popup {
                            request,
                            pane,
                            exit_status,
                            ..
                        } => {
                            if exit_status.is_some()
                                || request.close_on_key
                                || ((decoded.name == "Escape" || decoded.name == "C-c")
                                    && !request.close_on_exit
                                    && !request.close_on_success)
                            {
                                close = true;
                                close_exit = (*exit_status).unwrap_or(129);
                            } else {
                                let _ = pane.input(&data[start..i]);
                            }
                        }
                        ActiveOverlay::DisplayPanes {
                            command,
                            accept_input,
                            ..
                        } => {
                            if !*accept_input {
                                close = true;
                            } else if matches!(decoded.name.as_str(), "Escape" | "q" | "C-c") {
                                close = true;
                            } else if let Some(index) = decoded
                                .name
                                .chars()
                                .next()
                                .filter(|_| decoded.name.chars().count() == 1)
                                .and_then(|value| value.to_digit(10))
                            {
                                let pane_id = state.lock().ok().and_then(|st| {
                                    st.active_window_panes(target)
                                        .ok()
                                        .and_then(|(window, _)| window.panes.get(index as usize))
                                        .map(|pane| pane.id)
                                });
                                if let Some(pane_id) = pane_id {
                                    let source = if command.is_empty() {
                                        vec![
                                            "select-pane".to_string(),
                                            "-t".to_string(),
                                            format!("%{pane_id}"),
                                        ]
                                    } else {
                                        command
                                            .iter()
                                            .map(|word| word.replace("%%", &format!("%{pane_id}")))
                                            .collect()
                                    };
                                    selected_command = Some(source);
                                    close = true;
                                }
                            }
                        }
                    }
                    let inserted = selected_command
                        .as_ref()
                        .is_some_and(|command| !command.is_empty());
                    let result = if let Some(command) =
                        selected_command.filter(|command| !command.is_empty())
                    {
                        let agents = hub.snapshot().panes;
                        Some(command::run_with_context(
                            &command,
                            state,
                            &agents,
                            &attached_context,
                        ))
                    } else if close {
                        Some(if close_exit == 0 {
                            command::CommandResult::ok("")
                        } else {
                            let mut result = command::CommandResult::err("");
                            result.exit = close_exit;
                            result
                        })
                    } else {
                        None
                    };
                    if close {
                        if let Some(mut overlay) = active_overlay.take() {
                            overlay.complete(
                                result.unwrap_or_else(|| command::CommandResult::ok("")),
                                inserted,
                            );
                        }
                    }
                    force_render = true;
                    continue;
                }
                if let Some(prompt) = command_prompt.as_mut() {
                    let (decoded, consumed) = decode_tty_key(&data[i..])
                        .map(|(key, consumed)| (key.name, consumed))
                        .unwrap_or_else(|| (plain_prompt_key(data[i]), 1));
                    i += consumed;
                    match prompt.handle_key(&decoded, state, hub, &attached_context) {
                        CommandPromptInput::Continue => {}
                        CommandPromptInput::Finish(result) => {
                            let mut prompt = command_prompt.take().expect("command prompt checked");
                            prompt.complete(&result, state, &attached_context);
                        }
                        CommandPromptInput::Cancel => {
                            let mut prompt = command_prompt.take().expect("command prompt checked");
                            prompt.cancel_external();
                        }
                    }
                    force_render = true;
                    continue;
                }
                if let Some(active) = confirm.take() {
                    // A `confirm-before` prompt is up: this key answers it and is
                    // consumed whole (so a multi-byte escape can't leak to the
                    // pane). `y`/`Y` runs the guarded command; every other key
                    // cancels, exactly like tmux's confirm callback.
                    let (key, consumed) = read_key(&data[i..]);
                    i += consumed;
                    force_render = true;
                    let accepted = matches!(key, Key::Byte(value) if value == active.confirm_key)
                        || (key == Key::Enter && active.default_yes);
                    let result = if accepted {
                        match active.action {
                            ConfirmAction::Command(command) => {
                                let agents = hub.snapshot().panes;
                                command::run_with_context(
                                    &command,
                                    state,
                                    &agents,
                                    &attached_context,
                                )
                            }
                            action @ (ConfirmAction::KillPane | ConfirmAction::KillWindow) => {
                                let killed = if let Ok(mut st) = state.lock() {
                                    let killed = match action {
                                        ConfirmAction::KillPane => st.kill_pane(target).is_ok(),
                                        ConfirmAction::KillWindow => st.kill_window(target).is_ok(),
                                        ConfirmAction::Command(_) => unreachable!(),
                                    };
                                    // A survivor window/pane inherits the client viewport,
                                    // just like a layout-changing prefix key.
                                    if killed && st.find(target).is_some() {
                                        let _ = st.resize_session(target, cols, pane_rows);
                                    }
                                    killed
                                } else {
                                    false
                                };
                                if killed {
                                    command::CommandResult::ok("")
                                } else {
                                    command::CommandResult::err("")
                                }
                            }
                        }
                    } else {
                        command::CommandResult::err("")
                    };
                    if let Some(reply) = active.reply {
                        let _ = reply.send(super::state::PromptCompletion {
                            stdout: result.stdout,
                            stderr: result.stderr,
                            exit: result.exit,
                            inserted: accepted,
                        });
                    }
                    continue;
                }
                if state
                    .lock()
                    .ok()
                    .is_some_and(|st| st.mode_view_active(target))
                {
                    let (decoded, consumed) = decode_tty_key(&data[i..]).unwrap_or_else(|| {
                        (
                            DecodedTtyKey {
                                name: plain_prompt_key(data[i]),
                                code: Some(key_from_byte(data[i])),
                                mouse: None,
                            },
                            1,
                        )
                    });
                    i += consumed;
                    let outcome = state
                        .lock()
                        .ok()
                        .and_then(|mut st| {
                            st.mode_view_key(target, &decoded.name, pane_rows as usize)
                                .ok()
                        })
                        .unwrap_or(ModeViewKeyResult::None);
                    match outcome {
                        ModeViewKeyResult::Command(command) if !command.is_empty() => {
                            let agents = hub.snapshot().panes;
                            let _ = command::run_with_context(
                                &command,
                                state,
                                &agents,
                                &attached_context,
                            );
                        }
                        ModeViewKeyResult::Prompt(request) => {
                            if let Ok(mut prompt) = CommandPrompt::for_mode(
                                request,
                                target,
                                state,
                                hub,
                                &attached_context,
                            ) {
                                if !prompt.spec.no_freeze {
                                    prompt.frozen_frame = Some(last_render.clone());
                                }
                                prompt.initial_incremental(state, hub, &attached_context);
                                command_prompt = Some(prompt);
                            }
                        }
                        ModeViewKeyResult::None | ModeViewKeyResult::Command(_) => {}
                    }
                    force_render = true;
                    continue;
                }
                if prefix_pending {
                    prefix_pending = false;
                    // Flush any keystrokes typed before this command so the pane
                    // sees them in order relative to a possible send-prefix byte.
                    if !forward_buf.is_empty() {
                        first_forward_at.get_or_insert_with(Instant::now);
                        if let Ok(stats) = forward_input(state, target, &forward_buf) {
                            add_input_stats(&mut forwarded, stats);
                        }
                        forward_buf.clear();
                    }
                    // The command key can be a multi-byte escape (e.g. PgUp), so
                    // parse a logical key rather than taking one raw byte.
                    let (key, mouse, consumed) = match decode_tty_key(&data[i..]) {
                        Some((mut decoded, consumed)) => {
                            resolve_mouse_key(
                                &mut decoded,
                                &mut mouse_input,
                                state,
                                target,
                                cols,
                                rows,
                                &mut status_cache,
                            );
                            (decoded.code, decoded.mouse, consumed)
                        }
                        None => (Some(key_from_byte(data[i])), None, 1),
                    };
                    i += consumed;
                    let Some(key) = key else {
                        continue;
                    };
                    match dispatch_key_binding(
                        "prefix",
                        key,
                        state,
                        target,
                        cols,
                        pane_rows,
                        hub,
                        &attached_context,
                        mouse,
                    ) {
                        PrefixOutcome::Detach => {
                            detach_requested = true;
                            should_exit = true;
                            break;
                        }
                        PrefixOutcome::SendPrefix(bytes) => forward_buf.extend(bytes),
                        PrefixOutcome::CopyMode {
                            page_up,
                            page_down,
                            slider,
                            mouse,
                            begin_selection,
                        } => {
                            set_copy_mode_state(state, target, true, page_up);
                            if let Some(mouse) = mouse {
                                if let Ok(mut st) = state.lock() {
                                    let vi = copy_mode_uses_vi_keys(&st, target);
                                    let position = mouse.pane_position();
                                    let _ = st.position_copy_cursor_from_mouse(
                                        target, position.x, position.y, vi,
                                    );
                                    if slider {
                                        let _ = st.set_copy_scroll_from_mouse(
                                            target, position.y, pane_rows, vi,
                                        );
                                    }
                                    if begin_selection {
                                        let separators = st
                                            .option_for_target(target, "word-separators")
                                            .unwrap_or("")
                                            .to_string();
                                        let _ = st.copy_mode_command(
                                            target,
                                            "begin-selection",
                                            vi,
                                            &separators,
                                        );
                                    }
                                }
                            }
                            if page_down {
                                if let Ok(mut st) = state.lock() {
                                    let vi = copy_mode_uses_vi_keys(&st, target);
                                    let separators = st
                                        .option_for_target(target, "word-separators")
                                        .unwrap_or("")
                                        .to_string();
                                    let _ =
                                        st.copy_mode_command(target, "page-down", vi, &separators);
                                }
                            }
                            force_render = true;
                        }
                        PrefixOutcome::Confirm { prompt, action } => {
                            confirm = Some(ActiveConfirm {
                                prompt,
                                action,
                                confirm_key: b'y',
                                default_yes: false,
                                reply: None,
                            });
                            force_render = true;
                        }
                        PrefixOutcome::Prompt { args } => {
                            if let Ok(mut prompt) =
                                CommandPrompt::new(args, None, state, hub, &attached_context)
                            {
                                if !prompt.spec.no_freeze {
                                    prompt.frozen_frame = Some(last_render.clone());
                                }
                                prompt.initial_incremental(state, hub, &attached_context);
                                command_prompt = Some(prompt);
                            }
                            force_render = true;
                        }
                        PrefixOutcome::Message { text, duration } => {
                            confirm = None;
                            status_message = Some((
                                text,
                                Instant::now()
                                    .checked_add(duration)
                                    .unwrap_or_else(Instant::now),
                            ));
                            force_render = true;
                        }
                        PrefixOutcome::ViewOutput(bytes) => {
                            append_view_output(state, target, &bytes);
                            force_render = true;
                        }
                        PrefixOutcome::Handled { changed } => {
                            if changed {
                                force_render = true;
                            }
                        }
                    }
                    continue;
                }
                if copy_mode_active(state, target) {
                    let (key, mouse, consumed) = match decode_tty_key(&data[i..]) {
                        Some((mut decoded, consumed)) => {
                            resolve_mouse_key(
                                &mut decoded,
                                &mut mouse_input,
                                state,
                                target,
                                cols,
                                rows,
                                &mut status_cache,
                            );
                            (decoded.code, decoded.mouse, consumed)
                        }
                        None => (Some(key_from_byte(data[i])), None, 1),
                    };
                    i += consumed;
                    let Some(key) = key else {
                        continue;
                    };
                    if is_configured_prefix(state, target, key) {
                        prefix_pending = true;
                        continue;
                    }
                    let copy_table = copy_table_name(state, target);
                    let table = state
                        .lock()
                        .ok()
                        .filter(|st| st.key_binding(copy_table, key).is_none())
                        .and_then(|st| st.key_binding("root", key).map(|_| "root"))
                        .unwrap_or(copy_table);
                    match dispatch_key_binding(
                        table,
                        key,
                        state,
                        target,
                        cols,
                        pane_rows,
                        hub,
                        &attached_context,
                        mouse,
                    ) {
                        PrefixOutcome::Detach => {
                            detach_requested = true;
                            should_exit = true;
                            break;
                        }
                        PrefixOutcome::SendPrefix(bytes) => forward_buf.extend(bytes),
                        PrefixOutcome::CopyMode {
                            page_up,
                            page_down: _,
                            slider: _,
                            mouse: _,
                            begin_selection: _,
                        } => {
                            set_copy_mode_state(state, target, true, page_up);
                            force_render = true;
                        }
                        PrefixOutcome::Confirm { prompt, action } => {
                            confirm = Some(ActiveConfirm {
                                prompt,
                                action,
                                confirm_key: b'y',
                                default_yes: false,
                                reply: None,
                            });
                            force_render = true;
                        }
                        PrefixOutcome::Prompt { args } => {
                            if let Ok(mut prompt) =
                                CommandPrompt::new(args, None, state, hub, &attached_context)
                            {
                                if !prompt.spec.no_freeze {
                                    prompt.frozen_frame = Some(last_render.clone());
                                }
                                prompt.initial_incremental(state, hub, &attached_context);
                                command_prompt = Some(prompt);
                            }
                            force_render = true;
                        }
                        PrefixOutcome::Message { text, duration } => {
                            confirm = None;
                            status_message = Some((
                                text,
                                Instant::now()
                                    .checked_add(duration)
                                    .unwrap_or_else(Instant::now),
                            ));
                            force_render = true;
                        }
                        PrefixOutcome::ViewOutput(bytes) => {
                            append_view_output(state, target, &bytes);
                            force_render = true;
                        }
                        PrefixOutcome::Handled { changed } => {
                            if changed {
                                force_render = true;
                            }
                        }
                    }
                    continue;
                }
                // Normal passthrough: forward bytes verbatim (arrow keys, UTF-8,
                // pastes, …), intercepting only the prefix key.
                let start = i;
                let (key, mouse, consumed) = match decode_tty_key(&data[i..]) {
                    Some((mut decoded, consumed)) => {
                        resolve_mouse_key(
                            &mut decoded,
                            &mut mouse_input,
                            state,
                            target,
                            cols,
                            rows,
                            &mut status_cache,
                        );
                        (decoded.code, decoded.mouse, consumed)
                    }
                    None => (Some(key_from_byte(data[i])), None, 1),
                };
                i += consumed;
                if key.is_some_and(|key| is_configured_prefix(state, target, key)) {
                    // Flush what preceded the prefix, then await the command key.
                    if !forward_buf.is_empty() {
                        first_forward_at.get_or_insert_with(Instant::now);
                        if let Ok(stats) = forward_input(state, target, &forward_buf) {
                            add_input_stats(&mut forwarded, stats);
                        }
                        forward_buf.clear();
                    }
                    prefix_pending = true;
                } else if key.is_some_and(|key| {
                    let table = client_key_table(state, target);
                    state
                        .lock()
                        .ok()
                        .is_some_and(|st| st.key_binding(&table, key).is_some())
                }) {
                    if !forward_buf.is_empty() {
                        first_forward_at.get_or_insert_with(Instant::now);
                        if let Ok(stats) = forward_input(state, target, &forward_buf) {
                            add_input_stats(&mut forwarded, stats);
                        }
                        forward_buf.clear();
                    }
                    let table = client_key_table(state, target);
                    match dispatch_key_binding(
                        &table,
                        key.expect("checked root binding"),
                        state,
                        target,
                        cols,
                        pane_rows,
                        hub,
                        &attached_context,
                        mouse,
                    ) {
                        PrefixOutcome::Detach => {
                            detach_requested = true;
                            should_exit = true;
                            break;
                        }
                        PrefixOutcome::SendPrefix(bytes) => forward_buf.extend(bytes),
                        PrefixOutcome::CopyMode {
                            page_up,
                            page_down,
                            slider,
                            mouse,
                            begin_selection,
                        } => {
                            set_copy_mode_state(state, target, true, page_up);
                            if let Some(mouse) = mouse {
                                if let Ok(mut st) = state.lock() {
                                    let vi = copy_mode_uses_vi_keys(&st, target);
                                    let position = mouse.pane_position();
                                    let _ = st.position_copy_cursor_from_mouse(
                                        target, position.x, position.y, vi,
                                    );
                                    if slider {
                                        let _ = st.set_copy_scroll_from_mouse(
                                            target, position.y, pane_rows, vi,
                                        );
                                    }
                                    if begin_selection {
                                        let separators = st
                                            .option_for_target(target, "word-separators")
                                            .unwrap_or("")
                                            .to_string();
                                        let _ = st.copy_mode_command(
                                            target,
                                            "begin-selection",
                                            vi,
                                            &separators,
                                        );
                                    }
                                }
                            }
                            if page_down {
                                if let Ok(mut st) = state.lock() {
                                    let vi = copy_mode_uses_vi_keys(&st, target);
                                    let separators = st
                                        .option_for_target(target, "word-separators")
                                        .unwrap_or("")
                                        .to_string();
                                    let _ =
                                        st.copy_mode_command(target, "page-down", vi, &separators);
                                }
                            }
                            force_render = true;
                        }
                        PrefixOutcome::Confirm { prompt, action } => {
                            confirm = Some(ActiveConfirm {
                                prompt,
                                action,
                                confirm_key: b'y',
                                default_yes: false,
                                reply: None,
                            });
                            force_render = true;
                        }
                        PrefixOutcome::Prompt { args } => {
                            if let Ok(mut prompt) =
                                CommandPrompt::new(args, None, state, hub, &attached_context)
                            {
                                if !prompt.spec.no_freeze {
                                    prompt.frozen_frame = Some(last_render.clone());
                                }
                                prompt.initial_incremental(state, hub, &attached_context);
                                command_prompt = Some(prompt);
                            }
                            force_render = true;
                        }
                        PrefixOutcome::Message { text, duration } => {
                            confirm = None;
                            status_message = Some((
                                text,
                                Instant::now()
                                    .checked_add(duration)
                                    .unwrap_or_else(Instant::now),
                            ));
                            force_render = true;
                        }
                        PrefixOutcome::ViewOutput(bytes) => {
                            append_view_output(state, target, &bytes);
                            force_render = true;
                        }
                        PrefixOutcome::Handled { changed } => {
                            if changed {
                                force_render = true;
                            }
                        }
                    }
                } else if forward_unbound {
                    forward_buf.extend_from_slice(&data[start..i]);
                }
            }
            if should_exit {
                break;
            }
        }
        if !forward_buf.is_empty() {
            first_forward_at.get_or_insert_with(Instant::now);
            if let Ok(stats) = forward_input(state, target, &forward_buf) {
                add_input_stats(&mut forwarded, stats);
            }
        }
        // Start (or extend) the latency clock after offering this keystroke burst
        // to the pane. The counters retain whether bytes reached the PTY now,
        // remained queued, or were dropped; the output/render hooks close it out.
        if forwarded.accepted() > 0 || forwarded.dropped > 0 {
            latmon.on_input(
                first_forward_at.unwrap_or_else(Instant::now),
                forwarded.accepted(),
                forwarded.queued,
                forwarded.dropped,
            );
        }
        if should_exit {
            break;
        }

        // A prefix command changed the window/pane layout: drop the cached frame
        // and force a full clear so the (possibly smaller) new active pane can't
        // leave the previous pane's cells behind.
        if force_render {
            last_render.clear();
            force_clear = true;
            status_cache.invalidate();
            let st = state
                .lock()
                .map_err(|_| io::Error::other("state poisoned"))?;
            (subscribed_window, output_subscription) =
                active_window_output_subscription(&st, target)?;
        }

        // 4. Render only when pane output or a layout/input action requests it.
        let should_render = output_ready
            || status_timer_ready
            || agent_status_changed
            || overlay_tick
            || message_expired
            || !render_invalidation.is_empty()
            || last_render.is_empty();

        if should_render {
            let mut wrote_frame = false;
            if let Ok(st) = state.lock() {
                let title = terminal_title_update(
                    &st,
                    target,
                    cols,
                    rows,
                    &mut status_cache,
                    &terminal,
                    &mut last_title,
                );
                if !title.is_empty() {
                    let _ = write_all(render_fd_owned.as_raw_fd(), &title);
                }
            }
            let frame = command_prompt
                .as_ref()
                .and_then(|prompt| prompt.frozen_frame.clone())
                .map(Ok)
                .unwrap_or_else(|| {
                    let st = state.lock();
                    match st {
                        Ok(g) => compose_frame_cached(
                            &g,
                            target,
                            cols,
                            rows,
                            status_h,
                            0,
                            &mut status_cache,
                            &terminal,
                        ),
                        Err(_) => Err(io::Error::other("state poisoned")),
                    }
                });
            if let Ok(mut frame) = frame {
                if let Some(overlay) = active_overlay.as_ref() {
                    if let Ok(st) = state.lock() {
                        frame.extend_from_slice(&render_active_overlay(
                            overlay, &st, target, cols, rows, &terminal,
                        ));
                    }
                }
                // Overlay a client prompt on the message line (tmux's last
                // row). It is appended to the frame so the diff below repaints
                // it, and its absence after completion redraws the status bar
                // underneath.
                if let Some(prompt) = command_prompt.as_ref() {
                    if prompt.completion.is_some() {
                        if let Ok(state) = state.lock() {
                            frame.extend_from_slice(&render_prompt_completion(
                                prompt, &state, target, cols, rows, status_h, &terminal,
                            ));
                        }
                    }
                    let (display, cursor, row, style, fill) = state
                        .lock()
                        .ok()
                        .map(|st| {
                            let (display, cursor) =
                                prompt.formatted_display(&st, target, usize::from(cols));
                            let line = st
                                .option_for_target(target, "message-line")
                                .and_then(|value| value.parse::<u16>().ok())
                                .unwrap_or(0)
                                .min(status_h.saturating_sub(1));
                            let row =
                                if st.option_for_target(target, "status-position") == Some("top") {
                                    line + 1
                                } else {
                                    rows.saturating_sub(status_h).saturating_add(line) + 1
                                };
                            let (style_option, style_fallback) = if prompt.vi_command {
                                ("message-command-style", "bg=black,fg=yellow,fill=black")
                            } else {
                                ("message-style", "bg=yellow,fg=black,fill=yellow")
                            };
                            let style_value = st
                                .option_for_target(target, style_option)
                                .unwrap_or(style_fallback);
                            (
                                display,
                                cursor,
                                row,
                                style_value.to_string(),
                                style_value
                                    .split(',')
                                    .any(|part| part.trim().starts_with("fill=")),
                            )
                        })
                        .unwrap_or_else(|| {
                            (
                                prompt.display(),
                                prompt.display_cursor(),
                                rows,
                                "bg=yellow,fg=black,fill=yellow".to_string(),
                                true,
                            )
                        });
                    let writable_cols = term::writable_width(&terminal, row, cols, rows) as u16;
                    frame.extend_from_slice(&render_status_prompt_styled_at_row(
                        &display,
                        cursor,
                        cols,
                        writable_cols,
                        row,
                        &style,
                        fill,
                        &terminal,
                    ));
                } else if let Some(active) = &confirm {
                    let prompt = &active.prompt;
                    let (row, style, fill) = state
                        .lock()
                        .ok()
                        .map(|st| {
                            let visible_lines = status_h.max(1);
                            let line = st
                                .option_for_target(target, "message-line")
                                .and_then(|value| value.parse::<u16>().ok())
                                .unwrap_or(0)
                                .min(visible_lines.saturating_sub(1));
                            let row = if status_h == 0 {
                                rows
                            } else if status::at_top(&st, target) {
                                line + 1
                            } else {
                                rows.saturating_sub(status_h).saturating_add(line) + 1
                            };
                            let value = st
                                .option_for_target(target, "message-style")
                                .unwrap_or("bg=yellow,fg=black,fill=yellow");
                            (
                                row,
                                value.to_string(),
                                value
                                    .split(',')
                                    .any(|part| part.trim().starts_with("fill=")),
                            )
                        })
                        .unwrap_or_else(|| {
                            (rows, "bg=yellow,fg=black,fill=yellow".to_string(), true)
                        });
                    let writable_cols = term::writable_width(&terminal, row, cols, rows) as u16;
                    frame.extend_from_slice(&render_status_prompt_styled_at_row(
                        prompt,
                        prompt.chars().count(),
                        cols,
                        writable_cols,
                        row,
                        &style,
                        fill,
                        &terminal,
                    ));
                } else if let Some((message, _)) = status_message.as_ref() {
                    let (row, rendered) = state
                        .lock()
                        .ok()
                        .map(|st| {
                            let visible_lines = status_h.max(1);
                            let line = st
                                .option_for_target(target, "message-line")
                                .and_then(|value| value.parse::<u16>().ok())
                                .unwrap_or(0)
                                .min(visible_lines.saturating_sub(1));
                            let row = if status_h == 0 {
                                rows
                            } else if status::at_top(&st, target) {
                                line + 1
                            } else {
                                rows.saturating_sub(status_h).saturating_add(line) + 1
                            };
                            let writable = term::writable_width(&terminal, row, cols, rows);
                            (
                                row,
                                status_cache.message_row(
                                    &st, target, message, cols, rows, writable, &terminal,
                                ),
                            )
                        })
                        .unwrap_or_else(|| (rows, Vec::new()));
                    frame.extend_from_slice(&render_status_message_row_at(
                        row, &rendered, &terminal,
                    ));
                }
                if frame != last_render {
                    let (mut repaint, direct_cursor_safe) = if last_render.is_empty() {
                        (frame.clone(), false)
                    } else {
                        let delta = diff_rendered_frame(&last_render, &frame);
                        let direct_cursor_safe = delta.direct_cursor_safe();
                        (delta.into_frame(), direct_cursor_safe)
                    };
                    // A first paint, resize, or layout change still needs one
                    // full clear. Keep that control sequence out of `last_render`
                    // so subsequent frames compare canonical compositor output.
                    if force_clear {
                        let mut cleared = Vec::with_capacity(repaint.len() + 8);
                        cleared.extend_from_slice(b"\x1b[H\x1b[2J");
                        cleared.extend_from_slice(&repaint);
                        repaint = cleared;
                    }

                    // Commit multi-row repaint deltas atomically when possible.
                    // Cursor-only changes and a bounded update ending on the
                    // cursor row can be sent directly: they never march the
                    // hardware cursor across unrelated rows. Larger
                    // unsynchronized deltas get an
                    // immediate hide/restore pair around only the dirty rows.
                    let sync_start = term::expand_capability(
                        &terminal,
                        "Sync",
                        &[term::CapabilityParameter::Number(1)],
                    );
                    let sync_end = term::expand_capability(
                        &terminal,
                        "Sync",
                        &[term::CapabilityParameter::Number(2)],
                    );
                    if let (Some(sync_start), Some(sync_end)) = (sync_start, sync_end) {
                        let output = suppress_redundant_cursor_visibility(
                            &repaint,
                            &mut output_cursor_visible,
                        );
                        let mut atomic_output =
                            Vec::with_capacity(sync_start.len() + output.len() + sync_end.len());
                        atomic_output.extend_from_slice(&sync_start);
                        atomic_output.extend_from_slice(&output);
                        atomic_output.extend_from_slice(&sync_end);
                        let _ = write_all(render_fd_owned.as_raw_fd(), &atomic_output);
                    } else if direct_cursor_safe && !force_clear {
                        let output = suppress_redundant_cursor_visibility(
                            &repaint,
                            &mut output_cursor_visible,
                        );
                        let _ = write_all(render_fd_owned.as_raw_fd(), &output);
                    } else {
                        let output =
                            guard_cursor_during_repaint(&repaint, &mut output_cursor_visible);
                        let _ = write_all(render_fd_owned.as_raw_fd(), &output);
                    }
                    last_render = frame;
                    force_clear = false;
                    wrote_frame = true;
                }
            }
            // Close the latency sample: a written frame is the keystroke's echo
            // reaching the screen; an unchanged frame means this input drew
            // nothing, so drop it rather than blame a later frame.
            if wrote_frame {
                latmon.on_render();
            } else {
                latmon.discard();
            }
        }
    }

    // Restore blocking mode and leave the outer tty using its capabilities.
    let _ = set_blocking(input_fd_owned.as_raw_fd());
    let _ = set_blocking(render_fd_owned.as_raw_fd());
    let tty_stop = tty_stop_sequence(&terminal, rows);
    let _ = write_all(render_fd_owned.as_raw_fd(), &tty_stop);
    // Restore the client's original terminal mode (undo `make_raw`) *before* any
    // detach/exit handshake, so those bytes reach a cooked tty. Then disarm the
    // guard: the terminal is already restored, so its `drop` must not run again.
    if let Some(ref t) = saved_termios {
        restore_termios(input_fd_owned.as_raw_fd(), t);
    }
    termios_guard.disarm();

    // If the user asked to detach (`C-b d`), run tmux's detach handshake so the
    // client exits cleanly (status 0) and prints "[detached (from session
    // <name>)]". We do this *after* the tty restore above so those sequences hit
    // the terminal before the client's detach message. For any other exit (client
    // EOF, read error, server shutdown) the connection is already tearing down,
    // so there is nothing to hand shake — returning drops the socket and the
    // client sees EOF, matching a lost/closed client.
    if detach_requested {
        let session_name = state
            .lock()
            .ok()
            .and_then(|st| {
                st.sessions()
                    .iter()
                    .find(|session| session.id == session_id)
                    .map(|session| session.name.clone())
            })
            .unwrap_or_else(|| stable_target.clone());
        let _ = detach_handshake(reader, writer, &session_name);
    } else if session_ended {
        let _ = exit_handshake(reader, writer);
    }

    Ok(())
}

/// Return the outer-terminal sequence for a changed configured title.
/// Capability lookup is deliberately all-or-nothing, matching tty_set_title:
/// terminals need both `tsl` (start title) and `fsl` (finish title).
fn terminal_title_update(
    state: &ServerState,
    session: &str,
    cols: u16,
    rows: u16,
    status_cache: &mut status::RenderCache,
    terminal: &dyn TerminalCapabilities,
    last_title: &mut Option<String>,
) -> Vec<u8> {
    if state.option_for_target(session, "set-titles") != Some("on") {
        return Vec::new();
    }
    let template = state
        .option_for_target(session, "set-titles-string")
        .or_else(|| super::options::option_default("set-titles-string"))
        .unwrap_or_default();
    let Some(title) = status_cache.expand_format(state, session, template, cols, rows) else {
        return Vec::new();
    };
    if last_title.as_deref() == Some(title.as_str()) {
        return Vec::new();
    }
    *last_title = Some(title.clone());

    let (Some(start), Some(finish)) = (
        term::string_capability(terminal, "tsl"),
        term::string_capability(terminal, "fsl"),
    ) else {
        return Vec::new();
    };
    let mut output = Vec::with_capacity(start.len() + title.len() + finish.len());
    output.extend_from_slice(start);
    output.extend_from_slice(title.as_bytes());
    output.extend_from_slice(finish);
    output
}

/// Complete a normal attached-client exit after its last pane is gone.
///
/// Real tmux sends MSG_EXIT(0), waits for MSG_EXITING, then sends MSG_EXITED.
/// A plain EOF is not equivalent: the client reports "lost server" or remains
/// blocked behind hmux's bidirectional pairing.
fn exit_handshake<R, W>(reader: &mut R, writer: &mut W) -> io::Result<()>
where
    R: FrameReader,
    W: FrameWriter,
{
    writer.send(Frame::new(Message::Exit(Some(0))))?;

    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        match reader.recv() {
            Ok(frame) => match frame.msg {
                Message::Exiting | Message::Exit(_) => break,
                _ => continue,
            },
            Err(e)
                if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut =>
            {
                continue;
            }
            Err(_) => break,
        }
    }

    writer.send(Frame::new(Message::Exited))?;
    Ok(())
}

/// tmux's detach handshake (`server_client_check_exit`, `CLIENT_EXIT_DETACH`).
///
/// Sends `MSG_DETACH` carrying the session name, waits (bounded) for the client's
/// `MSG_EXITING` acknowledgement, then sends `MSG_EXITED`. On receiving these the
/// client leaves its event loop, prints `[detached (from session <name>)]`, and
/// exits with status 0.
///
/// This is what makes native detach match stock tmux. Without it, the native
/// server just returned, dropping only its half of the socketpair; hmux's
/// pairing (`serve::pump`) keeps the *client* socket open on the other direction,
/// so the client neither sees EOF nor a detach — it hangs attached. Even a clean
/// EOF would only get the client to "lost server" (exit 1), never the detach
/// message. The explicit handshake is required.
fn detach_handshake<R, W>(reader: &mut R, writer: &mut W, session: &str) -> io::Result<()>
where
    R: FrameReader,
    W: FrameWriter,
{
    writer.send(Frame::new(Message::Detach(Some(session.to_string()))))?;

    // Wait for the client's MSG_EXITING before MSG_EXITED, as real tmux does. The
    // imsg reader carries a short SO_RCVTIMEO (set for the attach loop), so poll
    // until a deadline, tolerating timeouts; give up on EOF/error (client gone).
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        match reader.recv() {
            Ok(frame) => match frame.msg {
                Message::Exiting | Message::Exit(_) => break,
                _ => continue, // late resize/other frames: ignore, keep waiting
            },
            Err(e)
                if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut =>
            {
                continue;
            }
            Err(_) => break,
        }
    }

    writer.send(Frame::new(Message::Exited))?;
    Ok(())
}

/// Composite one full frame for the attached client: the active pane's grid in
/// the top region and the status bar (if enabled) in the reserved bottom rows.
///
/// **In-place, no full-screen clear.** Each physical row is positioned
/// absolutely, its content rewritten, and cleared to end-of-line (`EL`) to drop
/// any stale cells from a previous, longer frame. This is the key difference
/// from a `\x1b[2J`-per-frame repaint: erasing the entire screen before every
/// redraw blanks the terminal for an instant, so a routine burst of frames (a
/// shell echoing a keystroke, or zsh's multi-write prompt redraw with a slow
/// precmd) reads as flicker — text appears then is overwritten. Real tmux never
/// clears the whole screen for ordinary output; it overwrites in place. We do
/// the same, so redrawing a cell with identical content is invisible and only
/// genuine changes are seen. A one-shot full clear on first paint / resize /
/// layout change is handled by the caller (see `force_clear`).
///
/// Order: draw the pane rows, erase any pane rows now below the content,
/// constrain scrolling to the pane region (DECSTBM — otherwise a line feed at
/// the bottom would scroll the status bar away), draw the status bar last, then
/// restore the pane's own cursor position (peeled from the VT dump) so the
/// terminal is left where the user expects.
/// Render a prompt onto the client's message line (tmux's last row), styled
/// like tmux's default `message-style` (black on yellow). The prompt fills the
/// row and the cursor is parked just after the text.
fn render_status_prompt_styled_at_row(
    prompt: &str,
    cursor: usize,
    cols: u16,
    writable_cols: u16,
    row: u16,
    style: &str,
    fill: bool,
    terminal: &dyn TerminalCapabilities,
) -> Vec<u8> {
    let cols = cols as usize;
    let writable_cols = writable_cols as usize;
    let text = clip_prompt_display(prompt, 0, writable_cols);
    let shown = cursor.min(cols);
    let text =
        status::render_overlay_text_for_terminal(&text, style, writable_cols, fill, terminal);

    let mut out = Vec::with_capacity(cols + 32);
    // Show the cursor so it sits at the prompt, then position to the last row.
    out.extend_from_slice(b"\x1b[?25h");
    out.extend_from_slice(format!("\x1b[{row};1H").as_bytes());
    out.extend_from_slice(&text);
    append_terminal_style_reset(&mut out, terminal);
    // Park the cursor right after the prompt text.
    out.extend_from_slice(format!("\x1b[{};{}H", row, shown + 1).as_bytes());
    out
}

fn render_status_message_row_at(
    row: u16,
    rendered: &[u8],
    terminal: &dyn TerminalCapabilities,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(rendered.len() + 24);
    out.extend_from_slice(b"\x1b[?25l");
    out.extend_from_slice(format!("\x1b[{row};1H").as_bytes());
    out.extend_from_slice(rendered);
    append_terminal_style_reset(&mut out, terminal);
    out
}

fn append_terminal_style_reset(out: &mut Vec<u8>, terminal: &dyn TerminalCapabilities) {
    if let Some(reset) = term::string_capability(terminal, "sgr0") {
        out.extend_from_slice(reset);
    }
}

fn overlay_position(value: Option<&str>, available: u16, size: u16) -> u16 {
    match value {
        Some("C" | "M" | "P" | "W" | "S") | None => available.saturating_sub(size) / 2,
        Some(value) if value.ends_with('%') => value
            .trim_end_matches('%')
            .parse::<u32>()
            .ok()
            .map(|percent| (u32::from(available.saturating_sub(size)) * percent / 100) as u16)
            .unwrap_or(0),
        Some(value) => value
            .parse::<u16>()
            .unwrap_or(0)
            .min(available.saturating_sub(size)),
    }
}

fn draw_overlay_box(out: &mut Vec<u8>, top: u16, left: u16, width: u16, height: u16, title: &str) {
    if width < 2 || height < 2 {
        return;
    }
    let inner = width.saturating_sub(2) as usize;
    let mut top_line = format!("┌{}┐", "─".repeat(inner));
    if !title.is_empty() && inner > 2 {
        let shown = clip_mode_line(title, inner.saturating_sub(2));
        let replacement = format!(" {} ", shown);
        let mut chars = top_line.chars().collect::<Vec<_>>();
        for (index, character) in replacement.chars().enumerate() {
            if index + 1 < chars.len().saturating_sub(1) {
                chars[index + 1] = character;
            }
        }
        top_line = chars.into_iter().collect();
    }
    out.extend_from_slice(format!("\x1b[{};{}H{}", top + 1, left + 1, top_line).as_bytes());
    for row in 1..height.saturating_sub(1) {
        out.extend_from_slice(
            format!(
                "\x1b[{};{}H│\x1b[{};{}H│",
                top + row + 1,
                left + 1,
                top + row + 1,
                left + width
            )
            .as_bytes(),
        );
    }
    out.extend_from_slice(
        format!("\x1b[{};{}H└{}┘", top + height, left + 1, "─".repeat(inner)).as_bytes(),
    );
}

fn render_active_overlay(
    overlay: &ActiveOverlay,
    st: &ServerState,
    target: &str,
    cols: u16,
    rows: u16,
    terminal: &dyn TerminalCapabilities,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"\x1b[?25l");
    append_terminal_style_reset(&mut out, terminal);
    match overlay {
        ActiveOverlay::Menu {
            request, selected, ..
        } => {
            let content_width = request
                .items
                .iter()
                .map(|item| {
                    format::display_width(&item.label)
                        + if item.key.is_empty() {
                            0
                        } else {
                            format::display_width(&item.key) + 3
                        }
                })
                .max()
                .unwrap_or(1)
                .max(format::display_width(&request.title));
            let width = (content_width + 4).min(cols as usize).max(3) as u16;
            let height = (request.items.len() + 2).min(rows as usize).max(3) as u16;
            let left = overlay_position(request.x.as_deref(), cols, width);
            let top = overlay_position(request.y.as_deref(), rows, height);
            draw_overlay_box(&mut out, top, left, width, height, &request.title);
            for (index, item) in request
                .items
                .iter()
                .take(height.saturating_sub(2) as usize)
                .enumerate()
            {
                if item.label.is_empty() {
                    out.extend_from_slice(
                        format!(
                            "\x1b[{};{}H{}",
                            top + index as u16 + 2,
                            left + 2,
                            "─".repeat(width.saturating_sub(2) as usize)
                        )
                        .as_bytes(),
                    );
                    continue;
                }
                let key = if item.key.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", item.key)
                };
                let text = clip_mode_line(
                    &format!("{}{}", item.label, key),
                    width.saturating_sub(4) as usize,
                );
                out.extend_from_slice(
                    format!("\x1b[{};{}H", top + index as u16 + 2, left + 3).as_bytes(),
                );
                if index == *selected {
                    out.extend_from_slice(b"\x1b[7m");
                }
                out.extend_from_slice(text.as_bytes());
                if index == *selected {
                    append_terminal_style_reset(&mut out, terminal);
                }
            }
        }
        ActiveOverlay::Popup { request, pane, .. } => {
            let width = overlay_dimension(request.width.as_deref(), cols, 50)
                .max(3)
                .min(cols.max(3));
            let height = overlay_dimension(request.height.as_deref(), rows, 50)
                .max(3)
                .min(rows.max(3));
            let left = overlay_position(request.x.as_deref(), cols, width);
            let top = overlay_position(request.y.as_deref(), rows, height);
            let inset = u16::from(request.border);
            if request.border {
                draw_overlay_box(&mut out, top, left, width, height, &request.title);
            }
            if let Ok(vt) = pane.dump_vt() {
                let (popup_rows, cursor) = split_pane_vt(&vt);
                let visible_height = height.saturating_sub(inset * 2);
                let visible_width = width.saturating_sub(inset * 2);
                for (row, content) in popup_rows
                    .iter()
                    .rev()
                    .take(visible_height as usize)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .enumerate()
                {
                    out.extend_from_slice(
                        format!(
                            "\x1b[{};{}H\x1b[{}X\x1b[{};{}H",
                            top + inset + row as u16 + 1,
                            left + inset + 1,
                            visible_width,
                            top + inset + row as u16 + 1,
                            left + inset + 1
                        )
                        .as_bytes(),
                    );
                    out.extend_from_slice(content);
                }
                if !pane.has_exited() {
                    if let Some((cursor_row, cursor_col)) = parse_cup(cursor) {
                        out.extend_from_slice(
                            format!(
                                "\x1b[{};{}H\x1b[?25h",
                                top + inset + cursor_row,
                                left + inset + cursor_col
                            )
                            .as_bytes(),
                        );
                    }
                }
            }
        }
        ActiveOverlay::DisplayPanes { .. } => {
            if let Ok((window, _)) = st.active_window_panes(target) {
                for (index, pane) in window.panes.iter().enumerate() {
                    let rect = window.pane_rect(pane.id).unwrap_or_default();
                    let label = index.to_string();
                    let row = rect.top + rect.height / 2 + 1;
                    let col = rect.left + rect.width.saturating_sub(label.len() as u16) / 2 + 1;
                    out.extend_from_slice(
                        format!("\x1b[{row};{col}H\x1b[30;43m{label}").as_bytes(),
                    );
                    append_terminal_style_reset(&mut out, terminal);
                }
            }
        }
    }
    out
}

fn render_prompt_completion(
    prompt: &CommandPrompt,
    state: &ServerState,
    target: &str,
    cols: u16,
    rows: u16,
    status_height: u16,
    terminal: &dyn TerminalCapabilities,
) -> Vec<u8> {
    let Some(completion) = prompt.completion.as_ref() else {
        return Vec::new();
    };
    let label_width = completion
        .items
        .iter()
        .map(|item| format::display_width(&item.label))
        .max()
        .unwrap_or(0);
    let items = completion
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| MenuItem {
            label: format!(
                "{}{}",
                item.label,
                " ".repeat(label_width.saturating_sub(format::display_width(&item.label)))
            ),
            key: char::from(b'0' + index.min(9) as u8).to_string(),
            command: Vec::new(),
        })
        .collect::<Vec<_>>();
    let height = (items.len() + 2).min(rows as usize).max(3) as u16;
    let left = prompt_input_width(&prompt.input[..completion.start])
        .saturating_add(format::display_width(prompt.label()))
        .saturating_sub(2)
        .min(usize::from(cols.saturating_sub(1)));
    let top = if status::at_top(state, target) {
        status_height
    } else {
        rows.saturating_sub(height.saturating_add(status_height))
    };
    let overlay = ActiveOverlay::Menu {
        request: MenuRequest {
            title: String::new(),
            items,
            selected: completion.selected,
            x: Some(left.to_string()),
            y: Some(top.to_string()),
        },
        selected: completion.selected,
        reply: None,
    };
    render_active_overlay(&overlay, state, target, cols, rows, terminal)
}

fn render_mode_rows(view: &ModeView, width: usize, height: usize) -> Vec<Vec<u8>> {
    if view.kind == ModeKind::Clock {
        return render_clock_rows(width, height);
    }
    let mut rows = Vec::with_capacity(height);
    rows.push(clip_mode_line(&format!("[{}]", view.title), width).into_bytes());
    for index in view.scroll..view.items.len() {
        if rows.len() >= height {
            break;
        }
        let label = clip_mode_line(&view.items[index].label, width.saturating_sub(2));
        let mut row = Vec::new();
        if index == view.selected {
            row.extend_from_slice(b"\x1b[7m> ");
            row.extend_from_slice(label.as_bytes());
            row.extend_from_slice(b"\x1b[0m");
        } else {
            row.extend_from_slice(b"  ");
            row.extend_from_slice(label.as_bytes());
        }
        rows.push(row);
    }
    rows
}

fn clip_mode_line(value: &str, width: usize) -> String {
    let mut output = String::new();
    let mut used = 0;
    for (token, token_width) in format::display_tokens(value) {
        if used + token_width > width {
            break;
        }
        output.push_str(token);
        used += token_width;
    }
    output
}

fn render_clock_rows(width: usize, height: usize) -> Vec<Vec<u8>> {
    let mut time = [0 as libc::c_char; 16];
    let now = unsafe { libc::time(std::ptr::null_mut()) };
    // SAFETY: localtime returns a process-owned tm pointer valid until the next
    // libc time conversion on this thread; strftime copies it immediately.
    let length = unsafe {
        let local = libc::localtime(&now);
        if local.is_null() {
            0
        } else {
            libc::strftime(time.as_mut_ptr(), time.len(), c"%H:%M".as_ptr(), local)
        }
    };
    let text = if length == 0 {
        "00:00".to_string()
    } else {
        String::from_utf8_lossy(
            &time[..length]
                .iter()
                .map(|value| *value as u8)
                .collect::<Vec<_>>(),
        )
        .into_owned()
    };
    const DIGITS: [[&str; 5]; 10] = [
        ["###", "# #", "# #", "# #", "###"],
        ["  #", "  #", "  #", "  #", "  #"],
        ["###", "  #", "###", "#  ", "###"],
        ["###", "  #", "###", "  #", "###"],
        ["# #", "# #", "###", "  #", "  #"],
        ["###", "#  ", "###", "  #", "###"],
        ["###", "#  ", "###", "# #", "###"],
        ["###", "  #", "  #", "  #", "  #"],
        ["###", "# #", "###", "# #", "###"],
        ["###", "# #", "###", "  #", "###"],
    ];
    let mut clock = vec![String::new(); 5];
    for character in text.chars() {
        for row in 0..5 {
            if !clock[row].is_empty() {
                clock[row].push(' ');
            }
            if let Some(digit) = character.to_digit(10) {
                clock[row].push_str(DIGITS[digit as usize][row]);
            } else {
                clock[row].push_str(if matches!(row, 1 | 3) { " # " } else { "   " });
            }
        }
    }
    let clock_width = clock.first().map_or(0, String::len);
    let top = height.saturating_sub(5) / 2;
    let left = width.saturating_sub(clock_width) / 2;
    let mut rows = vec![Vec::new(); height];
    for (offset, line) in clock.into_iter().enumerate() {
        if let Some(row) = rows.get_mut(top + offset) {
            row.extend(std::iter::repeat_n(b' ', left));
            row.extend_from_slice(line.as_bytes());
        }
    }
    rows
}

#[cfg(test)]
fn compose_frame(
    st: &ServerState,
    target: &str,
    cols: u16,
    rows: u16,
    status_h: u16,
    scroll_offset: usize,
) -> io::Result<Vec<u8>> {
    let mut status_cache = status::RenderCache::default();
    let terminal = ResolvedTerm::resolve(
        TerminalIdentity::new(
            "test",
            vec![
                "am=1".into(),
                "colors=256".into(),
                "sgr0=\x1b[m".into(),
                "setaf=\x1b[3%p1%dm".into(),
                "setab=\x1b[4%p1%dm".into(),
                "dim=\x1b[2m".into(),
                "bold=\x1b[1m".into(),
            ],
            0,
            None,
        ),
        [],
    );
    compose_frame_cached(
        st,
        target,
        cols,
        rows,
        status_h,
        scroll_offset,
        &mut status_cache,
        &terminal,
    )
}

fn compose_frame_cached(
    st: &ServerState,
    target: &str,
    cols: u16,
    rows: u16,
    status_h: u16,
    scroll_offset: usize,
    status_cache: &mut status::RenderCache,
    terminal: &dyn TerminalCapabilities,
) -> io::Result<Vec<u8>> {
    if st.active_window_panes(target)?.0.panes.len() > 1 {
        return compose_split_frame(st, target, cols, rows, status_h, status_cache, terminal);
    }
    let pane_height = if status_h > 0 {
        rows.saturating_sub(status_h).max(1)
    } else {
        rows
    };
    let pane_top = if status_h > 0 && status::at_top(st, target) {
        status_h
    } else {
        0
    };
    let active_mode = st.active_mode_view(target);
    let (all_rows, mut cursor, cursor_visible, restore_cursor, frame_capacity) =
        if let Some(view) = active_mode {
            (
                render_mode_rows(view, cols as usize, pane_height as usize),
                Vec::new(),
                false,
                false,
                usize::from(cols) * usize::from(pane_height) + 256,
            )
        } else if let Some(copy) = st.active_copy_state(target) {
            let top = copy.grid.scrollback_rows.saturating_sub(copy.scroll);
            let pane_rows = copy
                .vt_rows()
                .skip(top)
                .take(pane_height as usize)
                .map(<[u8]>::to_vec)
                .collect::<Vec<_>>();
            let cursor_row = copy
                .cursor
                .row
                .saturating_sub(top)
                .min(pane_height.saturating_sub(1) as usize)
                + 1;
            let cursor_col = copy.cursor.col.min(cols.saturating_sub(1) as usize) + 1;
            (
                pane_rows,
                format!("\x1b[{cursor_row};{cursor_col}H").into_bytes(),
                true,
                true,
                copy.vt.len() + 256,
            )
        } else {
            let (vt, scroll) =
                st.dump_active_pane_viewport_vt(target, scroll_offset, pane_height as usize)?;
            let (pane_rows, cursor) = split_pane_vt(&vt);
            (
                pane_rows
                    .into_iter()
                    .map(<[u8]>::to_vec)
                    .collect::<Vec<_>>(),
                cursor.to_vec(),
                scroll == 0 && st.active_pane_cursor_visible(target).unwrap_or(true),
                scroll == 0,
                vt.len() + 256,
            )
        };
    // The terminal formatter receives a selection containing only this
    // viewport. Hidden history is never serialized or scanned here.
    let pane_rows = &all_rows[..];
    // The pane's DECTCEM state. The VT dump carries the cursor *position* but not
    // its *visibility*, so we query it and mirror it below. A TUI that hides the
    // cursor and paints its own (e.g. claude-code) must not leave the client's
    // real cursor lit on top — that is the "double cursor" bug this fixes. While
    // scrolled back into history the pane's cursor belongs to the live viewport,
    // not to what we are painting, so keep it hidden and don't reposition it.
    let cursor_shape = st.active_pane_cursor_shape(target).unwrap_or(0);
    let mut out = Vec::with_capacity(frame_capacity);
    let active_copy = active_mode
        .is_none()
        .then(|| st.active_copy_state(target))
        .flatten();
    let line_number_width = active_copy
        .map(|copy| copy_line_number_width(st, target, copy))
        .unwrap_or(0)
        .min(cols.saturating_sub(1) as usize);
    if let Some(copy) = active_copy {
        let top = copy.grid.scrollback_rows.saturating_sub(copy.scroll);
        let row = copy
            .cursor
            .row
            .saturating_sub(top)
            .min(pane_height.saturating_sub(1) as usize)
            + 1;
        let col = line_number_width
            .saturating_add(copy.cursor.col)
            .min(cols.saturating_sub(1) as usize)
            + 1;
        cursor = format!("\x1b[{row};{col}H").into_bytes();
    }

    // Hide the cursor for the duration of the repaint so it doesn't visibly
    // travel across every row we position to; it is restored (if the pane wants
    // it shown) at the very end, at the pane's real cursor location.
    out.extend_from_slice(b"\x1b[?25l");
    // Start from a known-default SGR so the first row's erase-to-EOL doesn't
    // inherit a stray background color from a prior frame.
    append_terminal_style_reset(&mut out, terminal);

    // Draw each pane row in place: position, rewrite content, erase to EOL.
    for i in 0..pane_height as usize {
        out.extend_from_slice(format!("\x1b[{};1H", usize::from(pane_top) + i + 1).as_bytes());
        if let Some(copy) = active_copy {
            let physical_row = copy
                .grid
                .scrollback_rows
                .saturating_sub(copy.scroll)
                .saturating_add(i);
            render_copy_line_number(
                &mut out,
                st,
                target,
                copy,
                physical_row,
                physical_row == copy.cursor.row,
                line_number_width,
                terminal,
            );
        }
        if line_number_width > 0 {
            out.extend_from_slice(b"\x1b[?7l");
        }
        if let Some(row) = pane_rows.get(i) {
            out.extend_from_slice(row);
        }
        if line_number_width > 0 {
            out.extend_from_slice(b"\x1b[?7h");
        }
        append_terminal_style_reset(&mut out, terminal);
        out.extend_from_slice(b"\x1b[K");
    }
    if let Some(copy) = active_copy {
        render_copy_search(
            &mut out,
            copy,
            copy_mode_uses_vi_keys(st, target),
            &copy_style_escape(
                st,
                target,
                "copy-mode-match-style",
                "bg=cyan,fg=black",
                terminal,
            ),
            &copy_style_escape(
                st,
                target,
                "copy-mode-current-match-style",
                "bg=magenta,fg=black",
                terminal,
            ),
            pane_top,
            line_number_width as u16,
            pane_height,
            cols.saturating_sub(line_number_width as u16),
            terminal,
        );
        render_copy_selection(
            &mut out,
            copy,
            copy_mode_uses_vi_keys(st, target),
            &copy_style_escape(
                st,
                target,
                "copy-mode-selection-style",
                "bg=yellow,fg=black",
                terminal,
            ),
            pane_top,
            line_number_width as u16,
            pane_height,
            cols.saturating_sub(line_number_width as u16),
            terminal,
        );
        render_copy_mark_and_position(
            &mut out,
            st,
            target,
            copy,
            pane_top,
            line_number_width as u16,
            pane_height,
            cols.saturating_sub(line_number_width as u16),
            terminal,
        );
    }
    // Erase any pane rows the previous, taller frame left behind. Clear them one
    // at a time (not `\x1b[J`) so the status region below is never touched.
    if status_h > 0 {
        out.extend_from_slice(
            format!("\x1b[{};{}r", pane_top + 1, pane_top + pane_height).as_bytes(),
        );

        let status = status_cache.render(st, target, cols, rows);
        let first = if pane_top == 0 {
            rows.saturating_sub(status_h) + 1
        } else {
            1
        };
        for r in 0..status_h {
            let physical_row = first + r;
            let writable = term::writable_width(terminal, physical_row, cols, rows);
            out.extend_from_slice(format!("\x1b[{physical_row};1H").as_bytes());
            status.append_row_for_terminal(
                &mut out,
                usize::from(r),
                writable < usize::from(cols),
                terminal,
            );
            // Status rows are already padded to the terminal width. EL while
            // the terminal has a pending wrap at the final column can erase
            // that final cell on some terminals.
            append_terminal_style_reset(&mut out, terminal);
        }
    } else {
        // A prior frame may have enabled a pane-only region before the status
        // option changed. Restore full-screen scrolling when no rows are
        // reserved.
        out.extend_from_slice(b"\x1b[r");
    }

    // Restore the pane's own cursor position (peeled from the VT dump), which is
    // where the user's cursor must sit — but only at the live bottom. While
    // scrolled back the CUP would drop the cursor into the middle of history, so
    // leave it homed (and hidden, per `cursor_visible` above).
    if restore_cursor {
        out.extend_from_slice(&offset_cup_row(&cursor, pane_top));
        out.extend_from_slice(format!("\x1b[{cursor_shape} q").as_bytes());
    }
    // Mirror the pane's cursor visibility. Only re-show it when the pane's app
    // wants it shown; a TUI that hid it (painting its own) keeps the client's
    // real cursor off, so there is exactly one cursor on screen.
    if cursor_visible {
        out.extend_from_slice(b"\x1b[?25h");
    }
    Ok(out)
}

const HIDE_CURSOR: &[u8] = b"\x1b[?25l";
const SHOW_CURSOR: &[u8] = b"\x1b[?25h";

fn strip_cursor_visibility(frame: &[u8]) -> (Vec<u8>, Option<bool>) {
    let mut content = Vec::with_capacity(frame.len());
    let mut requested = None;
    let mut i = 0;
    while i < frame.len() {
        let rest = &frame[i..];
        if rest.starts_with(HIDE_CURSOR) {
            requested = Some(false);
            i += HIDE_CURSOR.len();
        } else if rest.starts_with(SHOW_CURSOR) {
            requested = Some(true);
            i += SHOW_CURSOR.len();
        } else {
            content.push(frame[i]);
            i += 1;
        }
    }
    (content, requested)
}

#[derive(Debug)]
struct FrameDelta {
    bytes: Vec<u8>,
    requested_visibility: Option<bool>,
    paint_rows: BTreeSet<u16>,
    cursor_row: Option<u16>,
}

impl FrameDelta {
    fn direct_cursor_safe(&self) -> bool {
        if self.paint_rows.is_empty() {
            return true;
        }
        let Some(cursor_row) = self.cursor_row else {
            return false;
        };
        if self.paint_rows.len() == 1 {
            return self.paint_rows.contains(&cursor_row);
        }
        let (Some(&first), Some(&last)) = (self.paint_rows.first(), self.paint_rows.last()) else {
            return true;
        };
        self.paint_rows.len() <= 2
            && last == cursor_row
            && usize::from(last.saturating_sub(first)) + 1 == self.paint_rows.len()
    }

    fn into_frame(mut self) -> Vec<u8> {
        if let Some(visible) = self.requested_visibility {
            self.bytes
                .extend_from_slice(if visible { SHOW_CURSOR } else { HIDE_CURSOR });
        }
        self.bytes
    }
}

#[derive(Debug)]
struct PositionedSection<'a> {
    row: u16,
    bytes: &'a [u8],
}

#[derive(Debug)]
struct ParsedFrame<'a> {
    prefix: &'a [u8],
    paint: Vec<PositionedSection<'a>>,
    cursor: Option<PositionedSection<'a>>,
}

fn cup_prefix(bytes: &[u8]) -> Option<(u16, usize)> {
    let rest = bytes.strip_prefix(b"\x1b[")?;
    let final_index = rest.iter().position(|byte| (0x40..=0x7e).contains(byte))?;
    if !matches!(rest[final_index], b'H' | b'f') {
        return None;
    }
    let parameters = &rest[..final_index];
    let row = parameters
        .split(|byte| *byte == b';')
        .next()
        .filter(|value| !value.is_empty())
        .and_then(|value| std::str::from_utf8(value).ok())
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(1)
        .max(1);
    Some((row, final_index + 3))
}

fn is_cursor_tail(section: &[u8]) -> bool {
    let Some((_, cup_len)) = cup_prefix(section) else {
        return false;
    };
    let tail = &section[cup_len..];
    if tail.is_empty() {
        return true;
    }
    let Some(parameters) = tail
        .strip_prefix(b"\x1b[")
        .and_then(|tail| tail.strip_suffix(b" q"))
    else {
        return false;
    };
    !parameters.is_empty() && parameters.iter().all(u8::is_ascii_digit)
}

fn parse_positioned_frame(frame: &[u8]) -> Option<ParsedFrame<'_>> {
    let mut positions = Vec::new();
    let mut index = 0;
    while index < frame.len() {
        if let Some((row, len)) = cup_prefix(&frame[index..]) {
            positions.push((index, row));
            index += len;
        } else {
            index += 1;
        }
    }
    let &(first, _) = positions.first()?;
    let mut sections = positions
        .iter()
        .enumerate()
        .map(|(index, &(start, row))| {
            let end = positions
                .get(index + 1)
                .map(|(start, _)| *start)
                .unwrap_or(frame.len());
            PositionedSection {
                row,
                bytes: &frame[start..end],
            }
        })
        .collect::<Vec<_>>();
    let cursor = sections
        .last()
        .is_some_and(|section| is_cursor_tail(section.bytes))
        .then(|| sections.pop())
        .flatten();
    Some(ParsedFrame {
        prefix: &frame[..first],
        paint: sections,
        cursor,
    })
}

fn row_payloads(frame: &ParsedFrame<'_>) -> BTreeMap<u16, Vec<u8>> {
    let mut rows = BTreeMap::<u16, Vec<u8>>::new();
    for section in &frame.paint {
        rows.entry(section.row)
            .or_default()
            .extend_from_slice(section.bytes);
    }
    rows
}

fn diff_rendered_frame(previous: &[u8], current: &[u8]) -> FrameDelta {
    let (previous, _) = strip_cursor_visibility(previous);
    let (current, requested_visibility) = strip_cursor_visibility(current);
    let (Some(previous_frame), Some(current_frame)) = (
        parse_positioned_frame(&previous),
        parse_positioned_frame(&current),
    ) else {
        return FrameDelta {
            bytes: current,
            requested_visibility,
            paint_rows: BTreeSet::from([0, 1]),
            cursor_row: None,
        };
    };

    let previous_rows = row_payloads(&previous_frame);
    let current_rows = row_payloads(&current_frame);
    let mut paint_rows = previous_rows
        .keys()
        .chain(current_rows.keys())
        .copied()
        .filter(|row| previous_rows.get(row) != current_rows.get(row))
        .collect::<BTreeSet<_>>();

    if previous_frame.prefix != current_frame.prefix {
        paint_rows.extend(current_rows.keys().copied());
    }

    let cursor_changed = previous_frame.cursor.as_ref().map(|section| section.bytes)
        != current_frame.cursor.as_ref().map(|section| section.bytes);
    let mut bytes = Vec::new();
    if !paint_rows.is_empty() {
        bytes.extend_from_slice(current_frame.prefix);
        for section in &current_frame.paint {
            if paint_rows.contains(&section.row) {
                bytes.extend_from_slice(section.bytes);
            }
        }
    }
    if !paint_rows.is_empty() || cursor_changed {
        if let Some(cursor) = current_frame.cursor.as_ref() {
            bytes.extend_from_slice(cursor.bytes);
        }
    }
    FrameDelta {
        bytes,
        requested_visibility,
        paint_rows,
        cursor_row: current_frame.cursor.as_ref().map(|section| section.row),
    }
}

/// Remove compositor-internal DECTCEM churn from a frame before it reaches the
/// client tty.
///
/// Frames are built independently, so each one hides the cursor while painting
/// and then optionally shows it.  The outer terminal is stateful: repeating
/// that pair for every application update makes its hardware cursor disappear
/// and reappear at up to 30 Hz.  Preserve the frame's final requested state,
/// but emit a visibility command only when that state differs from the last one
/// sent.  Cursor positioning and all cell/style output remain untouched.
fn suppress_redundant_cursor_visibility(frame: &[u8], visible: &mut Option<bool>) -> Vec<u8> {
    let (mut out, requested) = strip_cursor_visibility(frame);
    if let Some(requested) = requested {
        if *visible != Some(requested) {
            out.extend_from_slice(if requested { SHOW_CURSOR } else { HIDE_CURSOR });
            *visible = Some(requested);
        }
    }
    out
}

/// Keep the hardware cursor hidden while an unsynchronized frame is painted.
///
/// Unlike synchronized output, an ordinary tty may display the result of every
/// cursor-positioning command as soon as it arrives. Strip the compositor's
/// internal DECTCEM commands, hide before any other frame bytes, and restore the
/// requested state immediately after the final cursor position.
fn guard_cursor_during_repaint(frame: &[u8], visible: &mut Option<bool>) -> Vec<u8> {
    let (content, requested) = strip_cursor_visibility(frame);
    let Some(requested) = requested else {
        return content;
    };

    let was_hidden = *visible == Some(false);
    let mut out = Vec::with_capacity(
        content.len()
            + if was_hidden { 0 } else { HIDE_CURSOR.len() }
            + if requested { SHOW_CURSOR.len() } else { 0 },
    );
    if !was_hidden {
        out.extend_from_slice(HIDE_CURSOR);
    }
    out.extend_from_slice(&content);
    if requested {
        out.extend_from_slice(SHOW_CURSOR);
    }
    *visible = Some(requested);
    out
}

/// Paint every pane in the active window from the preserved split tree.
fn compose_split_frame(
    st: &ServerState,
    target: &str,
    cols: u16,
    rows: u16,
    status_h: u16,
    status_cache: &mut status::RenderCache,
    terminal: &dyn TerminalCapabilities,
) -> io::Result<Vec<u8>> {
    let (win, active) = st.active_window_panes(target)?;
    let available_rows = rows.saturating_sub(status_h).max(1);
    let pane_top = if status_h > 0 && status::at_top(st, target) {
        status_h
    } else {
        0
    };
    let mut out = Vec::with_capacity((cols as usize) * (rows as usize) + 512);
    let mut active_cursor = (0, 0);
    let active_id = win.panes.get(active).map(|pane| pane.id);
    let mut owners = vec![None; cols as usize * available_rows as usize];
    out.extend_from_slice(b"\x1b[?25l");
    append_terminal_style_reset(&mut out, terminal);

    for (index, node) in win.panes.iter().enumerate() {
        let rect = win.pane_rect(node.id).unwrap_or_default();
        let top = rect.top.min(available_rows);
        let left = rect.left.min(cols);
        let height = rect.height.min(available_rows.saturating_sub(top));
        let width = rect.width.min(cols.saturating_sub(left));
        for y in top..top.saturating_add(height) {
            for x in left..left.saturating_add(width) {
                owners[y as usize * cols as usize + x as usize] = Some(node.id);
            }
        }
        let (pane_rows, cursor, row_start) = if let Some(view) = node.mode_view.as_ref() {
            (
                render_mode_rows(view, width as usize, height as usize),
                Vec::new(),
                0,
            )
        } else if let Some(copy) = node.copy.as_ref() {
            let top = copy.grid.scrollback_rows.saturating_sub(copy.scroll);
            let cursor_row = copy.cursor.row.saturating_sub(top) as u16 + 1;
            let cursor_col = copy.cursor.col as u16 + 1;
            (
                copy.vt_rows()
                    .skip(top)
                    .take(height as usize)
                    .map(<[u8]>::to_vec)
                    .collect::<Vec<_>>(),
                format!("\x1b[{cursor_row};{cursor_col}H").into_bytes(),
                0,
            )
        } else {
            let vt = node.pane.dump_vt()?;
            let (pane_rows, cursor) = split_pane_vt(&vt);
            (
                pane_rows.into_iter().map(<[u8]>::to_vec).collect(),
                cursor.to_vec(),
                node.pane.scrollback_rows().unwrap_or(0),
            )
        };
        if index == active {
            let (cursor_row, cursor_col) = parse_cup(&cursor).unwrap_or((1, 1));
            active_cursor = (
                top + cursor_row.min(height.max(1)) - 1,
                left + cursor_col.min(width.max(1)) - 1,
            );
        }
        let line_number_width = node
            .copy
            .as_ref()
            .map(|copy| copy_line_number_width(st, target, copy))
            .unwrap_or(0)
            .min(width.saturating_sub(1) as usize);
        if index == active && node.copy.is_some() {
            active_cursor.1 = active_cursor
                .1
                .saturating_add(line_number_width as u16)
                .min(left + width.saturating_sub(1));
        }
        for row in 0..height {
            out.extend_from_slice(
                format!("\x1b[{};{}H", pane_top + top + row + 1, left + 1).as_bytes(),
            );
            append_terminal_style_reset(&mut out, terminal);
            out.extend_from_slice(format!("\x1b[{}X", width).as_bytes());
            out.extend_from_slice(
                format!("\x1b[{};{}H", pane_top + top + row + 1, left + 1).as_bytes(),
            );
            if let Some(copy) = node.copy.as_ref() {
                let physical_row = copy
                    .grid
                    .scrollback_rows
                    .saturating_sub(copy.scroll)
                    .saturating_add(row as usize);
                render_copy_line_number(
                    &mut out,
                    st,
                    target,
                    copy,
                    physical_row,
                    physical_row == copy.cursor.row,
                    line_number_width,
                    terminal,
                );
            }
            if let Some(content) = pane_rows.get(row_start + row as usize) {
                if line_number_width > 0 {
                    out.extend_from_slice(b"\x1b[?7l");
                }
                out.extend_from_slice(content);
                if line_number_width > 0 {
                    out.extend_from_slice(b"\x1b[?7h");
                }
            }
        }
        if let Some(copy) = node.copy.as_ref() {
            render_copy_search(
                &mut out,
                copy,
                copy_mode_uses_vi_keys(st, target),
                &copy_style_escape(
                    st,
                    target,
                    "copy-mode-match-style",
                    "bg=cyan,fg=black",
                    terminal,
                ),
                &copy_style_escape(
                    st,
                    target,
                    "copy-mode-current-match-style",
                    "bg=magenta,fg=black",
                    terminal,
                ),
                pane_top + top,
                left + line_number_width as u16,
                height,
                width.saturating_sub(line_number_width as u16),
                terminal,
            );
            render_copy_selection(
                &mut out,
                copy,
                copy_mode_uses_vi_keys(st, target),
                &copy_style_escape(
                    st,
                    target,
                    "copy-mode-selection-style",
                    "bg=yellow,fg=black",
                    terminal,
                ),
                pane_top + top,
                left + line_number_width as u16,
                height,
                width.saturating_sub(line_number_width as u16),
                terminal,
            );
            render_copy_mark_and_position(
                &mut out,
                st,
                target,
                copy,
                pane_top + top,
                left + line_number_width as u16,
                height,
                width.saturating_sub(line_number_width as u16),
                terminal,
            );
        }
    }

    let indicators = st.option_for_target(target, "pane-border-indicators") == Some("both");
    let owner = |x: u16, y: u16| -> Option<u32> {
        (x < cols && y < available_rows)
            .then(|| owners[y as usize * cols as usize + x as usize])
            .flatten()
    };
    for y in 0..available_rows {
        for x in 0..cols {
            if owner(x, y).is_some() {
                continue;
            }
            let left_owner = x.checked_sub(1).and_then(|px| owner(px, y));
            let right_owner = x.checked_add(1).and_then(|px| owner(px, y));
            let above_owner = y.checked_sub(1).and_then(|py| owner(x, py));
            let below_owner = y.checked_add(1).and_then(|py| owner(x, py));
            let vertical =
                left_owner.is_some() && right_owner.is_some() && left_owner != right_owner;
            let horizontal =
                above_owner.is_some() && below_owner.is_some() && above_owner != below_owner;

            // A separator can terminate into another separator, leaving the
            // junction cell with no pane owner directly above or below it.
            // The four surrounding pane quadrants reveal those side arms.
            let separates = |first: Option<u32>, second: Option<u32>| {
                first.is_some() && second.is_some() && first != second
            };
            let upper_left = x
                .checked_sub(1)
                .and_then(|px| y.checked_sub(1).and_then(|py| owner(px, py)));
            let upper_right = x
                .checked_add(1)
                .and_then(|px| y.checked_sub(1).and_then(|py| owner(px, py)));
            let lower_left = x
                .checked_sub(1)
                .and_then(|px| y.checked_add(1).and_then(|py| owner(px, py)));
            let lower_right = x
                .checked_add(1)
                .and_then(|px| y.checked_add(1).and_then(|py| owner(px, py)));
            let up = vertical || separates(upper_left, upper_right);
            let right = horizontal || separates(upper_right, lower_right);
            let down = vertical || separates(lower_left, lower_right);
            let left = horizontal || separates(upper_left, lower_left);
            let mut cell = match (up, right, down, left) {
                (true, false, true, false) => "│",
                (false, true, false, true) => "─",
                (true, true, true, false) => "├",
                (true, false, true, true) => "┤",
                (false, true, true, true) => "┬",
                (true, true, false, true) => "┴",
                (true, true, true, true) => "┼",
                _ => continue,
            };
            if indicators && vertical {
                let pair = (left_owner, right_owner);
                let mut segment_top = y;
                while segment_top > 0 {
                    let py = segment_top - 1;
                    let previous = (
                        x.checked_sub(1).and_then(|px| owner(px, py)),
                        x.checked_add(1).and_then(|px| owner(px, py)),
                    );
                    if previous != pair {
                        break;
                    }
                    segment_top = py;
                }
                if y == segment_top.saturating_add(1) {
                    if left_owner == active_id {
                        cell = "←";
                    } else if right_owner == active_id {
                        cell = "→";
                    }
                }
            }
            if indicators && horizontal {
                let pair = (above_owner, below_owner);
                let mut segment_left = x;
                while segment_left > 0 {
                    let px = segment_left - 1;
                    let previous = (
                        y.checked_sub(1).and_then(|py| owner(px, py)),
                        y.checked_add(1).and_then(|py| owner(px, py)),
                    );
                    if previous != pair {
                        break;
                    }
                    segment_left = px;
                }
                if x == segment_left.saturating_add(1) {
                    if above_owner == active_id {
                        cell = "↑";
                    } else if below_owner == active_id {
                        cell = "↓";
                    }
                }
            }
            out.extend_from_slice(format!("\x1b[{};{}H{cell}", pane_top + y + 1, x + 1).as_bytes());
        }
    }

    if status_h > 0 {
        out.extend_from_slice(
            format!("\x1b[{};{}r", pane_top + 1, pane_top + available_rows).as_bytes(),
        );
        let status = status_cache.render(st, target, cols, rows);
        let first = if pane_top == 0 {
            rows.saturating_sub(status_h) + 1
        } else {
            1
        };
        for index in 0..status_h {
            let row = first + index;
            let writable = term::writable_width(terminal, row, cols, rows);
            out.extend_from_slice(format!("\x1b[{row};1H").as_bytes());
            status.append_row_for_terminal(
                &mut out,
                usize::from(index),
                writable < usize::from(cols),
                terminal,
            );
            append_terminal_style_reset(&mut out, terminal);
        }
    } else {
        out.extend_from_slice(b"\x1b[r");
    }
    out.extend_from_slice(
        format!(
            "\x1b[{};{}H",
            pane_top + active_cursor.0 + 1,
            active_cursor.1 + 1
        )
        .as_bytes(),
    );
    out.extend_from_slice(format!("\x1b[{} q", win.panes[active].pane.cursor_shape()).as_bytes());
    if win.panes[active].mode_view.is_none()
        && (win.panes[active].copy.is_some()
            || win.panes[active].pane.cursor_visible().unwrap_or(true))
    {
        out.extend_from_slice(b"\x1b[?25h");
    }
    Ok(out)
}

fn parse_cup(cursor: &[u8]) -> Option<(u16, u16)> {
    let body = cursor.strip_prefix(b"\x1b[")?.strip_suffix(b"H")?;
    let semi = body.iter().position(|&b| b == b';')?;
    let row = std::str::from_utf8(&body[..semi]).ok()?.parse().ok()?;
    let col = std::str::from_utf8(&body[semi + 1..]).ok()?.parse().ok()?;
    Some((row, col))
}

fn offset_cup_row(cursor: &[u8], offset: u16) -> Vec<u8> {
    if offset == 0 {
        return cursor.to_vec();
    }
    parse_cup(cursor)
        .map(|(row, column)| {
            format!("\x1b[{};{}H", row.saturating_add(offset), column).into_bytes()
        })
        .unwrap_or_else(|| cursor.to_vec())
}

fn copy_mode_uses_vi_keys(st: &ServerState, target: &str) -> bool {
    match st.option_for_target(target, "mode-keys") {
        Some(mode) => mode == "vi",
        None => super::options::mode_keys_default() == "vi",
    }
}

fn copy_style_escape(
    st: &ServerState,
    target: &str,
    option: &str,
    fallback: &str,
    terminal: &dyn TerminalCapabilities,
) -> Vec<u8> {
    status::option_style_escape_for(st, target, option, fallback, terminal)
}

fn render_copy_mark_and_position(
    out: &mut Vec<u8>,
    st: &ServerState,
    target: &str,
    copy: &CopyState,
    screen_top: u16,
    screen_left: u16,
    height: u16,
    width: u16,
    terminal: &dyn TerminalCapabilities,
) {
    let view_top = copy.grid.scrollback_rows.saturating_sub(copy.scroll);
    if let Some((row, _)) = copy.mark {
        if row >= view_top && row < view_top.saturating_add(height as usize) {
            let style = copy_style_escape(
                st,
                target,
                "copy-mode-mark-style",
                "bg=red,fg=black",
                terminal,
            );
            out.extend_from_slice(
                format!(
                    "\x1b[{};{}H",
                    screen_top + (row - view_top) as u16 + 1,
                    screen_left + 1
                )
                .as_bytes(),
            );
            out.extend_from_slice(&style);
            for cell in copy.grid.rows[row].cells.iter().take(width as usize) {
                if !matches!(
                    cell.width,
                    ghostty_sys::GridCellWidth::SpacerTail | ghostty_sys::GridCellWidth::SpacerHead
                ) {
                    if cell.text.is_empty() {
                        out.push(b' ');
                    } else {
                        out.extend_from_slice(cell.text.as_bytes());
                    }
                }
            }
            append_terminal_style_reset(out, terminal);
        }
    }
    if copy.hide_position || width == 0 || height == 0 {
        return;
    }
    let mut vars = format::Vars::new();
    vars.set("copy_position", copy.scroll.to_string())
        .set("copy_position_limit", copy.grid.scrollback_rows.to_string())
        .set(
            "search_count",
            copy.search_count
                .map(|count| count.to_string())
                .unwrap_or_default(),
        )
        .set("copy_cursor_x", copy.cursor.col.to_string())
        .set("copy_cursor_y", copy.cursor.row.to_string());
    let configured = st
        .option_for_target(target, "copy-mode-position-format")
        .filter(|value| !value.is_empty());
    let source = configured.unwrap_or("[#{copy_position}/#{copy_position_limit}]");
    let align_right = configured.is_none() || source.contains("#[align=right]");
    let text = format::expand(source, &vars).replace("#[align=right]", "");
    let text = format::trim_right(&text, width as usize);
    let col = if align_right {
        screen_left + width.saturating_sub(format::display_width(&text) as u16) + 1
    } else {
        screen_left + 1
    };
    out.extend_from_slice(format!("\x1b[{};{}H", screen_top + 1, col).as_bytes());
    out.extend_from_slice(&copy_style_escape(
        st,
        target,
        "copy-mode-position-style",
        "bg=yellow,fg=black",
        terminal,
    ));
    out.extend_from_slice(text.as_bytes());
    append_terminal_style_reset(out, terminal);
}

fn copy_line_number_width(st: &ServerState, target: &str, copy: &CopyState) -> usize {
    if st
        .option_for_target(target, "copy-mode-line-numbers")
        .unwrap_or("off")
        == "off"
    {
        0
    } else {
        let lines = copy
            .grid
            .scrollback_rows
            .saturating_add(copy.grid.viewport_rows as usize)
            .saturating_add(1);
        (lines.max(1).ilog10() as usize + 2).max(4)
    }
}

fn render_copy_line_number(
    out: &mut Vec<u8>,
    st: &ServerState,
    target: &str,
    copy: &CopyState,
    physical_row: usize,
    current: bool,
    width: usize,
    terminal: &dyn TerminalCapabilities,
) {
    if width == 0 {
        return;
    }
    let mode = st
        .option_for_target(target, "copy-mode-line-numbers")
        .unwrap_or("off");
    let absolute = physical_row + 1;
    let relative = physical_row.abs_diff(copy.cursor.row);
    let number = match mode {
        "absolute" => absolute,
        "hybrid" if current => absolute,
        "relative" | "hybrid" => relative,
        _ => copy.grid.scrollback_rows.abs_diff(physical_row),
    };
    let style = if current {
        copy_style_escape(
            st,
            target,
            "copy-mode-current-line-number-style",
            "fg=yellow",
            terminal,
        )
    } else {
        copy_style_escape(
            st,
            target,
            "copy-mode-line-number-style",
            "fg=white,dim",
            terminal,
        )
    };
    out.extend_from_slice(&style);
    out.extend_from_slice(format!("{number:>w$} ", w = width - 1).as_bytes());
    append_terminal_style_reset(out, terminal);
}

fn render_copy_selection(
    out: &mut Vec<u8>,
    copy: &CopyState,
    vi: bool,
    style: &[u8],
    screen_top: u16,
    screen_left: u16,
    height: u16,
    width: u16,
    terminal: &dyn TerminalCapabilities,
) {
    if height == 0 || width == 0 {
        return;
    }
    let view_top = copy.grid.scrollback_rows.saturating_sub(copy.scroll);
    let view_bottom = view_top.saturating_add(height as usize);
    for (row, from, to) in copy_selection_segments(copy, vi) {
        if row < view_top || row >= view_bottom {
            continue;
        }
        let from = from.min(width as usize);
        let to = to.min(width as usize);
        if from >= to {
            continue;
        }
        out.extend_from_slice(
            format!(
                "\x1b[{};{}H",
                screen_top + (row - view_top) as u16 + 1,
                screen_left + from as u16 + 1,
            )
            .as_bytes(),
        );
        out.extend_from_slice(style);
        for cell in &copy.grid.rows[row].cells[from..to] {
            if matches!(
                cell.width,
                ghostty_sys::GridCellWidth::SpacerTail | ghostty_sys::GridCellWidth::SpacerHead
            ) {
                continue;
            }
            if cell.text.is_empty() {
                out.push(b' ');
            } else {
                out.extend_from_slice(cell.text.as_bytes());
            }
        }
        append_terminal_style_reset(out, terminal);
    }
}

fn render_copy_search(
    out: &mut Vec<u8>,
    copy: &CopyState,
    vi: bool,
    other_style: &[u8],
    current_style: &[u8],
    screen_top: u16,
    screen_left: u16,
    height: u16,
    width: u16,
    terminal: &dyn TerminalCapabilities,
) {
    if height == 0 || width == 0 {
        return;
    }
    let view_top = copy.grid.scrollback_rows.saturating_sub(copy.scroll);
    let view_bottom = view_top.saturating_add(height as usize);
    for (row, from, to, current) in copy_search_segments(copy, vi) {
        if row < view_top || row >= view_bottom {
            continue;
        }
        let from = from.min(width as usize);
        let to = to.min(width as usize);
        if from >= to {
            continue;
        }
        out.extend_from_slice(
            format!(
                "\x1b[{};{}H",
                screen_top + (row - view_top) as u16 + 1,
                screen_left + from as u16 + 1,
            )
            .as_bytes(),
        );
        out.extend_from_slice(if current { current_style } else { other_style });
        for cell in &copy.grid.rows[row].cells[from..to] {
            if matches!(
                cell.width,
                ghostty_sys::GridCellWidth::SpacerTail | ghostty_sys::GridCellWidth::SpacerHead
            ) {
                continue;
            }
            if cell.text.is_empty() {
                out.push(b' ');
            } else {
                out.extend_from_slice(cell.text.as_bytes());
            }
        }
        append_terminal_style_reset(out, terminal);
    }
}

/// Split a pane VT dump into per-row content plus the trailing cursor-restore.
///
/// `Terminal::dump_vt` emits `row0\r\nrow1\r\n…rowN` (styles flow inline across
/// rows) followed by a final cursor-position escape (`CSI r ; c H`). The
/// compositor needs the rows individually so it can position and erase each one
/// in place, and the cursor escape to reissue after drawing. If no trailing CUP
/// is found (empty grid), the cursor slice is empty and the whole input is
/// treated as content.
fn split_pane_vt(vt: &[u8]) -> (Vec<&[u8]>, &[u8]) {
    let (content, cursor) = split_trailing_cup(vt);
    let rows = content.split(|&b| b == b'\n').map(strip_cr).collect();
    (rows, cursor)
}

/// Peel a trailing `CSI … H` (cursor position) off the end of a VT dump,
/// returning `(content_before, cursor_seq)`. Returns `(vt, &[])` when the dump
/// does not end in a CUP.
fn split_trailing_cup(vt: &[u8]) -> (&[u8], &[u8]) {
    if vt.last() != Some(&b'H') {
        return (vt, &[]);
    }
    // Walk back over the CSI parameter bytes (digits and `;`).
    let mut j = vt.len() - 1;
    while j > 0 {
        let c = vt[j - 1];
        if c == b';' || c.is_ascii_digit() {
            j -= 1;
        } else {
            break;
        }
    }
    // Expect the `CSI` introducer (`ESC [`) immediately before the parameters.
    if j >= 2 && vt[j - 1] == b'[' && vt[j - 2] == 0x1b {
        (&vt[..j - 2], &vt[j - 2..])
    } else {
        (vt, &[])
    }
}

/// Drop a single trailing carriage return so rows split on `\n` keep no `\r`.
fn strip_cr(row: &[u8]) -> &[u8] {
    match row.split_last() {
        Some((&b'\r', rest)) => rest,
        _ => row,
    }
}

fn add_input_stats(total: &mut PaneInputStats, batch: PaneInputStats) {
    total.written += batch.written;
    total.queued += batch.queued;
    total.dropped += batch.dropped;
}

fn forward_input(
    state: &Arc<Mutex<ServerState>>,
    session: &str,
    bytes: &[u8],
) -> io::Result<PaneInputStats> {
    let st = state
        .lock()
        .map_err(|_| io::Error::other("state poisoned"))?;
    st.input_to_active_pane_with_stats(session, bytes)
}

fn write_all(fd: RawFd, mut data: &[u8]) -> io::Result<()> {
    while !data.is_empty() {
        let n = unsafe { libc::write(fd, data.as_ptr() as *const libc::c_void, data.len()) };
        if n < 0 {
            let e = io::Error::last_os_error();
            if e.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            if e.kind() == io::ErrorKind::WouldBlock {
                // For non-blocking tty, WouldBlock means buffer full; wait briefly.
                std::thread::sleep(Duration::from_millis(1));
                continue;
            }
            return Err(e);
        }
        data = &data[n as usize..];
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::pane::Pane;
    use super::super::state::PaneSpec;
    use super::*;

    #[test]
    fn prompt_clipping_ignores_style_directive_width() {
        assert_eq!(clip_prompt_display("#[]long: 12345", 0, 9), "#[]long: 123");
    }

    #[test]
    fn prompt_input_escapes_hashes_for_the_status_parser() {
        assert_eq!(render_prompt_input(&['#', '[', ']']), "##[]");
        assert_eq!(prompt_input_width(&['#', '[', ']']), 3);
    }

    #[test]
    fn sgr_mouse_decoding_preserves_full_event() {
        let (drag, drag_len) = decode_tty_key(b"\x1b[<32;2;3M").expect("drag event");
        assert_eq!(drag.name, "MouseDrag1Pane");
        assert_eq!(drag_len, 10);
        let drag = drag.mouse.expect("mouse");
        assert_eq!(drag.kind, MouseEventKind::Drag);
        assert_eq!(drag.button, Some(MouseButton::One));
        assert_eq!(drag.position, MousePosition { x: 1, y: 2 });
        assert_eq!(drag.protocol, MouseProtocol::Sgr);

        let (release, release_len) = decode_tty_key(b"\x1b[<0;5;7m").expect("drag release event");
        assert_eq!(release.name, "MouseUp1Pane");
        assert_eq!(release_len, 9);
        let release = release.mouse.expect("mouse");
        assert_eq!(release.kind, MouseEventKind::Up);
        assert_eq!(release.button, Some(MouseButton::One));
        assert_eq!(release.position, MousePosition { x: 4, y: 6 });
    }

    #[test]
    fn legacy_mouse_decoding_preserves_button_modifiers_and_position() {
        let sequence = [0x1b, b'[', b'M', 32 + 1 + 4 + 16, 33 + 7, 33 + 9];
        let (decoded, consumed) = decode_tty_key(&sequence).expect("legacy event");
        assert_eq!(decoded.name, "C-S-MouseDown2Pane");
        assert_eq!(consumed, sequence.len());
        let mouse = decoded.mouse.expect("mouse");
        assert_eq!(mouse.button, Some(MouseButton::Two));
        assert!(mouse.modifiers.shift);
        assert!(mouse.modifiers.control);
        assert_eq!(mouse.position, MousePosition { x: 7, y: 9 });
        assert_eq!(mouse.protocol, MouseProtocol::Legacy);
    }

    #[test]
    fn status_timer_repeats_and_preserves_deadline_for_same_interval() {
        let start = Instant::now();
        let two_seconds = Some(Duration::from_secs(2));
        let mut timer = StatusTimer::new(two_seconds, start);

        assert_eq!(timer.poll_timeout(start), 2_000);
        assert!(!timer.take_expired(start + Duration::from_secs(1)));

        timer.configure(two_seconds, start + Duration::from_secs(1));
        assert!(
            timer.take_expired(start + Duration::from_secs(2)),
            "an unrelated status invalidation must not postpone the deadline"
        );
        assert_eq!(
            timer.poll_timeout(start + Duration::from_secs(2)),
            2_000,
            "the repeating timer is scheduled from the callback"
        );
    }

    #[test]
    fn agent_status_subscription_wakes_attach_poll_without_a_timer() {
        use crate::integration::status::AgentStatus;
        use crate::integration::AgentState;
        use crate::observability::v1::PaneId;

        let hub = StatusHub::new();
        let subscription = hub.subscribe().expect("subscribe");
        subscription.drain();
        assert_eq!(
            wait_for_attach_events(-1, -1, -1, -1, -1, subscription.as_raw_fd(), 0).expect("poll"),
            (false, false, false, false, false)
        );

        hub.publish(
            PaneId(1),
            AgentStatus {
                agent: "codex",
                pid: Some(42),
                session_id: None,
                state: AgentState::Working,
            },
        );
        assert_eq!(
            wait_for_attach_events(-1, -1, -1, -1, -1, subscription.as_raw_fd(), 100)
                .expect("poll"),
            (false, false, false, false, true)
        );
    }

    #[test]
    fn status_timer_restarts_when_interval_changes_and_disables_at_zero() {
        let start = Instant::now();
        let mut timer = StatusTimer::new(Some(Duration::from_secs(2)), start);

        timer.configure(Some(Duration::from_secs(5)), start + Duration::from_secs(1));
        assert!(!timer.take_expired(start + Duration::from_secs(2)));
        assert!(timer.take_expired(start + Duration::from_secs(6)));

        timer.configure(None, start + Duration::from_secs(6));
        assert_eq!(timer.poll_timeout(start + Duration::from_secs(100)), -1);
        assert!(!timer.take_expired(start + Duration::from_secs(100)));
    }

    // ---- compositor: in-place redraw, no full-screen clear -----------------
    //
    // Regression for the "zsh Enter clutter" bug (see report.md). The compositor
    // used to prefix every frame with `\x1b[2J`, blanking the whole screen before
    // each redraw; a routine burst of frames then read as flicker. It now draws
    // each row in place and erases to end-of-line, clearing the whole screen only
    // on a forced full redraw (first paint / resize / layout change).

    fn contains_seq(haystack: &[u8], needle: &[u8]) -> bool {
        haystack.windows(needle.len()).any(|w| w == needle)
    }

    #[test]
    fn split_trailing_cup_peels_cursor() {
        let (content, cur) = split_trailing_cup(b"row0\r\nrow1\x1b[0m\x1b[2;13H");
        assert_eq!(content, b"row0\r\nrow1\x1b[0m");
        assert_eq!(cur, b"\x1b[2;13H");

        // No trailing CUP: the whole dump is content, cursor slice empty.
        let (content, cur) = split_trailing_cup(b"abc");
        assert_eq!(content, b"abc");
        assert!(cur.is_empty());
    }

    #[test]
    fn split_pane_vt_splits_rows_and_cursor() {
        let (rows, cur) =
            split_pane_vt(b"\x1b[38;5;1mRED\x1b[0m one\r\ntwo\r\n\r\nlast\x1b[0m\x1b[4;5H");
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0], b"\x1b[38;5;1mRED\x1b[0m one");
        assert_eq!(rows[1], b"two");
        assert_eq!(rows[2], b"");
        assert_eq!(rows[3], b"last\x1b[0m");
        assert_eq!(cur, b"\x1b[4;5H");
    }

    #[test]
    fn compose_frame_never_full_clears_and_erases_each_row() {
        let state = fresh_state();
        let st = state.lock().unwrap();
        let status_h = status::height(&st, "0");
        let frame = compose_frame(&st, "0", 80, 24, status_h, 0).expect("compose");
        assert!(
            !contains_seq(&frame, b"\x1b[2J"),
            "compose_frame must never emit a full-screen clear (flicker source)"
        );
        assert!(
            contains_seq(&frame, b"\x1b[K"),
            "compose_frame must erase each row to end-of-line so stale cells drop"
        );
    }

    // ---- compositor: cursor visibility mirroring (double-cursor bug) -------
    //
    // Regression for the "two cursors" bug (see report.md). A full-screen TUI
    // (claude-code) hides the hardware cursor (`CSI ? 25 l`) and paints its own.
    // The compositor used to always leave the client's real cursor lit, so two
    // cursors appeared — the app's painted one plus the client's, 3 rows up at
    // ghostty's tracked position. The compositor now mirrors the pane's DECTCEM
    // state: it hides the cursor during the repaint and only re-shows it (at the
    // pane's position) when the pane's app wants it shown.

    #[test]
    fn compose_frame_hides_cursor_when_pane_hid_it() {
        let state = fresh_state();
        // Pane app hides its cursor and paints content (like a TUI).
        {
            let st = state.lock().unwrap();
            st.active_pane("0")
                .unwrap()
                .feed(b"\x1b[?25l\x1b[H\x1b[2Jpainted");
        }
        let st = state.lock().unwrap();
        let status_h = status::height(&st, "0");
        let frame = compose_frame(&st, "0", 80, 24, status_h, 0).expect("compose");
        assert!(
            contains_seq(&frame, b"\x1b[?25l"),
            "cursor must be hidden during composite"
        );
        assert!(
            !contains_seq(&frame, b"\x1b[?25h"),
            "must NOT re-show the client cursor while the pane keeps it hidden \
             (that is the double cursor)"
        );
    }

    #[test]
    fn compose_frame_shows_cursor_when_pane_shows_it() {
        let state = fresh_state();
        {
            let st = state.lock().unwrap();
            // Default is visible; be explicit and paint something.
            st.active_pane("0").unwrap().feed(b"\x1b[?25h\x1b[Hshell");
        }
        let st = state.lock().unwrap();
        let status_h = status::height(&st, "0");
        let frame = compose_frame(&st, "0", 80, 24, status_h, 0).expect("compose");
        assert!(
            contains_seq(&frame, b"\x1b[?25h"),
            "a visible-cursor pane must re-show the client cursor"
        );
        // And the show must come *after* the final cursor position, so the one
        // visible cursor lands where the pane put it.
        assert!(
            frame.ends_with(b"\x1b[?25h"),
            "the show should be the last thing emitted, {:?}",
            String::from_utf8_lossy(&frame)
        );
    }

    #[test]
    fn compose_frame_restores_pane_cursor_shape() {
        let state = fresh_state();
        {
            let st = state.lock().unwrap();
            // Neovim uses DECSCUSR 6 for its steady bar insert-mode cursor.
            st.active_pane("0").unwrap().feed(b"\x1b[6 qeditor");
        }
        let st = state.lock().unwrap();
        let status_h = status::height(&st, "0");
        let frame = compose_frame(&st, "0", 80, 24, status_h, 0).expect("compose");
        assert!(
            contains_seq(&frame, b"\x1b[6 q"),
            "compositor must restore the pane's DECSCUSR shape"
        );
        let shape = frame
            .windows(5)
            .position(|window| window == b"\x1b[6 q")
            .unwrap();
        let show = frame
            .windows(6)
            .position(|window| window == b"\x1b[?25h")
            .unwrap();
        assert!(
            shape < show,
            "shape must be selected before cursor is shown"
        );
    }

    #[test]
    fn repeated_visible_frames_do_not_toggle_outer_cursor() {
        let mut visible = None;
        let first = suppress_redundant_cursor_visibility(
            b"\x1b[?25lpaint-one\x1b[3;4H\x1b[?25h",
            &mut visible,
        );
        assert_eq!(first, b"paint-one\x1b[3;4H\x1b[?25h");
        assert_eq!(visible, Some(true));

        let next = suppress_redundant_cursor_visibility(
            b"\x1b[?25lpaint-two\x1b[3;5H\x1b[?25h",
            &mut visible,
        );
        assert_eq!(next, b"paint-two\x1b[3;5H");
        assert_eq!(visible, Some(true));
    }

    #[test]
    fn actual_cursor_visibility_changes_are_preserved() {
        let mut visible = Some(true);
        let hidden = suppress_redundant_cursor_visibility(b"\x1b[?25lpaint-hidden", &mut visible);
        assert_eq!(hidden, b"paint-hidden\x1b[?25l");
        assert_eq!(visible, Some(false));

        let shown = suppress_redundant_cursor_visibility(
            b"\x1b[?25lpaint-visible\x1b[9;2H\x1b[?25h",
            &mut visible,
        );
        assert_eq!(shown, b"paint-visible\x1b[9;2H\x1b[?25h");
        assert_eq!(visible, Some(true));
    }

    #[test]
    fn unsynchronized_repaint_hides_cursor_before_any_cursor_movement() {
        let mut visible = Some(true);
        let frame = b"\x1b[H\x1b[2J\x1b[?25lpaint\x1b[3;5Hmore\x1b[9;2H\x1b[?25h";

        let output = guard_cursor_during_repaint(frame, &mut visible);

        assert_eq!(
            output,
            b"\x1b[?25l\x1b[H\x1b[2Jpaint\x1b[3;5Hmore\x1b[9;2H\x1b[?25h"
        );
        assert_eq!(visible, Some(true));
    }

    #[test]
    fn unsynchronized_hidden_cursor_stays_hidden_without_visibility_churn() {
        let mut visible = Some(false);

        let output = guard_cursor_during_repaint(b"\x1b[?25lpaint\x1b[3;5Hmore", &mut visible);

        assert_eq!(output, b"paint\x1b[3;5Hmore");
        assert_eq!(visible, Some(false));
    }

    #[test]
    fn cursor_only_delta_moves_directly_without_repainting_rows() {
        let previous = b"\x1b[?25l\x1b[m\x1b[1;1Hone\x1b[K\x1b[2;1Htwo\x1b[K\
            \x1b[1;2H\x1b[0 q\x1b[?25h";
        let current = b"\x1b[?25l\x1b[m\x1b[1;1Hone\x1b[K\x1b[2;1Htwo\x1b[K\
            \x1b[2;3H\x1b[0 q\x1b[?25h";

        let delta = diff_rendered_frame(previous, current);

        assert!(delta.paint_rows.is_empty());
        assert!(delta.direct_cursor_safe());
        assert_eq!(delta.into_frame(), b"\x1b[2;3H\x1b[0 q\x1b[?25h");
    }

    #[test]
    fn single_dirty_cursor_row_does_not_repaint_the_viewport() {
        let previous = b"\x1b[?25l\x1b[m\x1b[1;1Hsame\x1b[K\x1b[2;1Hold\x1b[K\
            \x1b[2;4H\x1b[0 q\x1b[?25h";
        let current = b"\x1b[?25l\x1b[m\x1b[1;1Hsame\x1b[K\x1b[2;1Hnew\x1b[K\
            \x1b[2;4H\x1b[0 q\x1b[?25h";

        let delta = diff_rendered_frame(previous, current);

        assert_eq!(delta.paint_rows, BTreeSet::from([2]));
        assert!(delta.direct_cursor_safe());
        let output = delta.into_frame();
        assert!(!contains_seq(&output, b"\x1b[1;1Hsame"));
        assert!(contains_seq(&output, b"\x1b[2;1Hnew"));
        assert!(output.ends_with(b"\x1b[2;4H\x1b[0 q\x1b[?25h"));
    }

    #[test]
    fn multi_row_delta_requires_cursor_guard_without_sync() {
        let previous = b"\x1b[?25l\x1b[m\x1b[1;1Hone\x1b[K\x1b[2;1Htwo\x1b[K\
            \x1b[3;1Hthree\x1b[K\x1b[3;4H\x1b[0 q\x1b[?25h";
        let current = b"\x1b[?25l\x1b[m\x1b[1;1HONE\x1b[K\x1b[2;1HTWO\x1b[K\
            \x1b[3;1HTHREE\x1b[K\x1b[3;4H\x1b[0 q\x1b[?25h";

        let delta = diff_rendered_frame(previous, current);

        assert_eq!(delta.paint_rows, BTreeSet::from([1, 2, 3]));
        assert!(!delta.direct_cursor_safe());
    }

    #[test]
    fn coalesced_forward_output_can_paint_new_rows_directly() {
        let previous = b"\x1b[?25l\x1b[m\x1b[1;1Hone\x1b[K\
            \x1b[2;1Htwo\x1b[K\x1b[2;4H\x1b[0 q\x1b[?25h";
        let current = b"\x1b[?25l\x1b[m\x1b[1;1Hone\x1b[K\
            \x1b[2;1Htwo\x1b[K\x1b[3;1Hthree\x1b[K\
            \x1b[4;1Hfour\x1b[K\x1b[4;4H\x1b[0 q\x1b[?25h";

        let delta = diff_rendered_frame(previous, current);

        assert_eq!(delta.paint_rows, BTreeSet::from([3, 4]));
        assert!(delta.direct_cursor_safe());
    }

    #[test]
    fn carriage_return_line_update_is_a_single_cursor_row_delta() {
        let state = fresh_state();
        let mut st = state.lock().unwrap();
        replace_active_pane_with_inert(&mut st);
        let status_h = status::height(&st, "0");
        let _ = st.resize_session("0", 80, 24 - status_h);
        st.active_pane("0").unwrap().feed(b"\rCURSOR-FRAME-01");
        let previous = compose_frame(&st, "0", 80, 24, status_h, 0).expect("first frame");
        st.active_pane("0").unwrap().feed(b"\rCURSOR-FRAME-02");
        let current = compose_frame(&st, "0", 80, 24, status_h, 0).expect("second frame");

        let delta = diff_rendered_frame(&previous, &current);

        assert!(
            delta.direct_cursor_safe(),
            "line update unexpectedly dirtied rows {:?} with cursor on {:?}",
            delta.paint_rows,
            delta.cursor_row
        );
    }

    // ---- compositor: render the visible screen, not oldest scrollback -----
    //
    // Regression for the "long output corrupts the screen and never recovers"
    // bug (see report.md). The pane's VT dump carries the whole grid — scrollback
    // history first, then the visible viewport. The compositor used to paint the
    // dump's *first* `pane_bottom` rows, i.e. the OLDEST history, so once any
    // output scrolled the pane the client froze showing the top of history, and
    // `clear` (which empties the viewport but keeps history) could not recover.
    // The compositor now skips `scrollback_rows` leading history rows and paints
    // the on-screen tail.

    #[test]
    fn compose_frame_renders_visible_tail_after_scroll_not_oldest_history() {
        let state = fresh_state();
        let mut st = state.lock().unwrap();
        // Size the pane exactly as run_attach does: client rows minus status.
        let status_h = status::height(&st, "0");
        let _ = st.resize_session("0", 80, 24 - status_h);
        // Feed far more lines than the viewport holds so the pane scrolls;
        // unique sentinels mark the oldest (scrolled-off) and newest (visible) rows.
        let mut feed = b"HEAD_OLDEST\r\n".to_vec();
        for i in 1..=60 {
            feed.extend_from_slice(format!("filler{i}\r\n").as_bytes());
        }
        feed.extend_from_slice(b"TAIL_NEWEST");
        st.active_pane("0").unwrap().feed(&feed);
        assert!(
            st.active_pane_scrollback_rows("0").unwrap() > 0,
            "precondition: the pane must have scrolled"
        );

        let frame = compose_frame(&st, "0", 80, 24, status_h, 0).expect("compose");
        let text = String::from_utf8_lossy(&frame);
        assert!(
            text.contains("TAIL_NEWEST"),
            "must paint the visible tail, got:\n{text}"
        );
        assert!(
            !text.contains("HEAD_OLDEST"),
            "must NOT paint the scrolled-off top of history, got:\n{text}"
        );
    }

    // ---- compositor: copy-mode scrollback view -----------------------------
    //
    // Regression for the "cannot scroll up with C-b PgUp" bug (see report.md).
    // The compositor always painted the live tail (skip == scrollback_rows), so
    // the client could never see history no matter what keys it pressed. It now
    // takes a `scroll_offset` — how many rows above the live bottom to view — and
    // slides the painted window up into scrollback, which is what `C-b PgUp`
    // drives in the attach loop.

    #[test]
    fn compose_frame_scrolls_back_into_history_with_offset() {
        let state = fresh_state();
        let mut st = state.lock().unwrap();
        let status_h = status::height(&st, "0");
        let _ = st.resize_session("0", 80, 24 - status_h);
        let mut feed = b"HEAD_OLDEST\r\n".to_vec();
        for i in 1..=60 {
            feed.extend_from_slice(format!("filler{i}\r\n").as_bytes());
        }
        feed.extend_from_slice(b"TAIL_NEWEST");
        st.active_pane("0").unwrap().feed(&feed);
        let scrollback = st.active_pane_scrollback_rows("0").unwrap();
        assert!(scrollback > 0, "precondition: the pane must have scrolled");

        // Live view (offset 0): the newest tail, never the scrolled-off top.
        let live = compose_frame(&st, "0", 80, 24, status_h, 0).expect("compose");
        let live = String::from_utf8_lossy(&live);
        assert!(
            live.contains("TAIL_NEWEST"),
            "live view shows the tail:\n{live}"
        );
        assert!(
            !live.contains("HEAD_OLDEST"),
            "live view must not show the scrolled-off top:\n{live}"
        );

        // Scrolled to the very top of history: the oldest line becomes visible.
        let top = compose_frame(&st, "0", 80, 24, status_h, scrollback).expect("compose");
        let top = String::from_utf8_lossy(&top);
        assert!(
            top.contains("HEAD_OLDEST"),
            "scrolling up by the full history must reveal the oldest line:\n{top}"
        );

        // An out-of-range offset clamps to the top rather than panicking or
        // painting past the start of history.
        let clamped = compose_frame(&st, "0", 80, 24, status_h, scrollback + 999).expect("compose");
        assert_eq!(
            top,
            String::from_utf8_lossy(&clamped),
            "an offset past the top of history clamps to the top"
        );
    }

    #[test]
    fn compose_frame_hides_cursor_while_scrolled_back() {
        let state = fresh_state();
        let mut st = state.lock().unwrap();
        let status_h = status::height(&st, "0");
        let _ = st.resize_session("0", 80, 24 - status_h);
        let mut feed = Vec::new();
        for i in 1..=60 {
            feed.extend_from_slice(format!("line{i}\r\n").as_bytes());
        }
        st.active_pane("0").unwrap().feed(&feed);

        // While scrolled into history the cursor belongs to the (hidden) live
        // viewport: it must not be re-shown down in the scrollback view.
        let frame = compose_frame(&st, "0", 80, 24, status_h, 5).expect("compose");
        assert!(
            !contains_seq(&frame, b"\x1b[?25h"),
            "a scrolled-back frame must not re-show the cursor"
        );
    }

    #[test]
    fn compose_frame_uses_the_frozen_styled_copy_snapshot() {
        let state = fresh_state();
        let mut st = state.lock().unwrap();
        let status_h = status::height(&st, "0");
        let _ = st.resize_session("0", 20, 6);
        st.active_pane("0")
            .unwrap()
            .feed(b"\x1b[38;5;1mBEFORE\x1b[m");
        st.set_pane_mode("0", Some("copy-mode"))
            .expect("enter copy mode");
        st.active_pane("0")
            .unwrap()
            .feed(b"\r\x1b[K\x1b[38;5;2mAFTER\x1b[m");

        let frame = compose_frame(&st, "0", 20, 6 + status_h, status_h, 0)
            .expect("compose frozen copy mode");
        assert!(
            contains_seq(&frame, b"\x1b[38;5;1mBEFORE"),
            "copy mode must retain the snapshot's styled cells: {frame:?}"
        );
        assert!(
            !contains_seq(&frame, b"AFTER"),
            "live output after copy-mode entry must remain hidden: {frame:?}"
        );
    }

    #[test]
    fn read_key_parses_escapes_and_bytes() {
        // Page keys and arrows.
        assert_eq!(read_key(b"\x1b[5~"), (Key::PageUp, 4));
        assert_eq!(read_key(b"\x1b[6~"), (Key::PageDown, 4));
        assert_eq!(read_key(b"\x1b[A"), (Key::Up, 3));
        assert_eq!(read_key(b"\x1b[B"), (Key::Down, 3));
        // A PgUp at the front of a chunk consumes only its own bytes.
        assert_eq!(read_key(b"\x1b[5~\x1b[5~"), (Key::PageUp, 4));
        // Prefix, Enter, a lone Escape, and plain bytes.
        assert_eq!(read_key(&[PREFIX]), (Key::Byte(PREFIX), 1));
        assert_eq!(read_key(b"\r"), (Key::Enter, 1));
        assert_eq!(read_key(b"\x1b"), (Key::Escape, 1));
        assert_eq!(read_key(b"q"), (Key::Byte(b'q'), 1));
        // A recognized key unused by copy mode still consumes one whole
        // logical key; unbound passthrough retains the original bytes.
        assert_eq!(read_key(b"\x1b[H"), (Key::Byte(0x1b), 3));
    }

    #[test]
    fn compose_frame_after_clear_shows_blank_not_stale_scrollback() {
        let state = fresh_state();
        let mut st = state.lock().unwrap();
        let status_h = status::height(&st, "0");
        let _ = st.resize_session("0", 80, 24 - status_h);
        let mut feed = Vec::new();
        for i in 1..=60 {
            feed.extend_from_slice(format!("line{i}\r\n").as_bytes());
        }
        st.active_pane("0").unwrap().feed(&feed);
        // `clear` empties the viewport (2J) and homes the cursor; scrollback still
        // holds the lines, but the visible screen is now blank.
        st.active_pane("0").unwrap().feed(b"\x1b[H\x1b[2J");

        let frame = compose_frame(&st, "0", 80, 24, status_h, 0).expect("compose");
        let text = String::from_utf8_lossy(&frame);
        assert!(
            !text.contains("line60") && !text.contains("line1"),
            "clear must recover to a blank screen, not stale scrollback, got:\n{text}"
        );
    }

    #[test]
    fn parse_target_with_t_flag() {
        let args = vec!["attach-session".into(), "-t".into(), "work".into()];
        assert_eq!(explicit_target_session(&args).as_deref(), Some("work"));
    }

    #[test]
    fn parse_target_positional() {
        let args = vec!["attach-session".into(), "my Sess".into()];
        assert_eq!(explicit_target_session(&args).as_deref(), Some("my Sess"));
    }

    #[test]
    fn parse_target_is_none_when_omitted() {
        let args = vec!["attach-session".into()];
        assert_eq!(explicit_target_session(&args), None);
    }

    #[test]
    fn parse_target_skips_boolean_flags() {
        let args = vec![
            "attach-session".into(),
            "-d".into(),
            "-t".into(),
            "foo".into(),
        ];
        assert_eq!(explicit_target_session(&args).as_deref(), Some("foo"));
    }

    #[test]
    fn pending_initial_attach_creates_an_ordinarily_named_session() {
        let mut state = ServerState::empty();
        let target = attach_target(None, &mut state, &command::ClientContext::default())
            .expect("initial attach target");

        assert_eq!(target, "0");
        assert!(!state.initial_attach_pending());
        assert_eq!(state.sessions().len(), 1);
        assert!(!state.window(0, 0).name.is_empty());
    }

    #[test]
    fn established_empty_server_does_not_bootstrap_again() {
        let mut state = ServerState::empty();
        let target = attach_target(None, &mut state, &command::ClientContext::default())
            .expect("initial attach target");
        assert!(state.kill_session(&target));

        assert_eq!(
            attach_target(None, &mut state, &command::ClientContext::default()),
            Err("no sessions\n".to_string())
        );
        assert!(state.sessions().is_empty());
    }

    #[test]
    fn explicit_missing_target_does_not_consume_pending_state() {
        let mut state = ServerState::empty();
        assert_eq!(
            attach_target(
                Some("missing".to_string()),
                &mut state,
                &command::ClientContext::default(),
            ),
            Ok("missing".to_string())
        );
        assert!(state.initial_attach_pending());
        assert!(state.sessions().is_empty());
    }

    // ---- prefix key table -------------------------------------------------
    //
    // These pin `dispatch_prefix_key` — the interactive counterpart to the
    // command interpreter — without a tty or a `tmux` binary (the `e2e_keys`
    // suite covers the same bindings end to end against real tmux). The default
    // session is "0" with one window (index 0) holding one pane.

    fn fresh_state() -> Arc<Mutex<ServerState>> {
        Arc::new(Mutex::new(
            ServerState::with_test_session().expect("build state"),
        ))
    }

    fn replace_active_pane_with_inert(st: &mut ServerState) {
        let pane = st.active_pane_mut("0").expect("active pane");
        let (cols, rows) = pane.size();
        *pane = Pane::inert(cols, rows).expect("replace live pane with inert fixture");
    }

    fn subscribed_active_identity(key: &ActiveWindowOutputKey) -> (u32, u64) {
        key.panes[key.active]
    }

    #[test]
    fn output_subscription_follows_external_active_window_change() {
        let state = fresh_state();
        let (mut subscribed_window, mut subscription) = {
            let st = state.lock().unwrap();
            active_window_output_subscription(&st, "0").expect("initial subscription")
        };
        let original_pane_id = subscribed_active_identity(&subscribed_window).0;

        // Model a separate tmux command connection selecting a newly-created
        // window while this attach loop is blocked on the original pane.
        state
            .lock()
            .unwrap()
            .new_window("0", None, true)
            .expect("external new-window");

        assert!(
            refresh_active_window_output_subscription(
                &state,
                "0",
                &mut subscribed_window,
                &mut subscription,
            )
            .expect("refresh subscription"),
            "the attach loop must replace a subscription to the old pane"
        );
        assert_ne!(
            subscribed_active_identity(&subscribed_window).0,
            original_pane_id
        );
        assert!(
            !refresh_active_window_output_subscription(
                &state,
                "0",
                &mut subscribed_window,
                &mut subscription,
            )
            .expect("stable subscription"),
            "an unchanged active-window pane set must keep its existing subscription"
        );

        // Closing the selected window chooses the survivor. This is the other
        // half of the live regression: the temporary window made typing smooth,
        // then closing it left the client subscribed to the removed pane.
        state
            .lock()
            .unwrap()
            .kill_window("0:1")
            .expect("close selected window");
        assert!(
            refresh_active_window_output_subscription(
                &state,
                "0",
                &mut subscribed_window,
                &mut subscription,
            )
            .expect("refresh survivor subscription"),
            "closing the selected window must subscribe to its survivor"
        );
        assert_eq!(
            subscribed_active_identity(&subscribed_window).0,
            original_pane_id
        );
    }

    #[test]
    fn output_subscription_follows_respawned_runtime_with_same_pane_id() {
        let state = fresh_state();
        let (mut subscribed_window, mut subscription) = {
            let st = state.lock().unwrap();
            active_window_output_subscription(&st, "0").expect("initial subscription")
        };
        let (original_pane_id, original_runtime_id) =
            subscribed_active_identity(&subscribed_window);
        replace_active_pane_with_inert(&mut state.lock().unwrap());

        assert!(refresh_active_window_output_subscription(
            &state,
            "0",
            &mut subscribed_window,
            &mut subscription,
        )
        .expect("refresh respawned runtime"));
        let (pane_id, runtime_id) = subscribed_active_identity(&subscribed_window);
        assert_eq!(pane_id, original_pane_id);
        assert_ne!(runtime_id, original_runtime_id);
    }

    #[test]
    fn output_subscription_wakes_for_visible_inactive_pane() {
        let state = fresh_state();
        let (mut subscribed_window, mut subscription) = {
            let st = state.lock().unwrap();
            active_window_output_subscription(&st, "0").expect("initial subscription")
        };
        state
            .lock()
            .unwrap()
            .split_window_direction_with_spec(
                "0",
                false,
                false,
                super::super::state::SplitDirection::TopBottom,
                PaneSpec::Inert,
            )
            .expect("split inactive pane");

        assert!(refresh_active_window_output_subscription(
            &state,
            "0",
            &mut subscribed_window,
            &mut subscription,
        )
        .expect("refresh split subscription"));
        assert_eq!(subscribed_window.panes.len(), 2);
        subscription.drain();

        {
            let st = state.lock().unwrap();
            let (window, active) = st.active_window_panes("0").expect("active window");
            let inactive = window
                .panes
                .iter()
                .enumerate()
                .find(|(index, _)| *index != active)
                .map(|(_, pane)| pane)
                .expect("inactive pane");
            inactive.pane.feed(b"BACKGROUND");
        }

        let mut pollfd = libc::pollfd {
            fd: subscription.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        assert_eq!(
            unsafe { libc::poll(&mut pollfd, 1, 100) },
            1,
            "inactive pane output must wake the shared compositor subscription"
        );
    }

    /// (window count, active window index, active window's pane count) for
    /// session "0".
    fn snapshot(state: &Arc<Mutex<ServerState>>) -> (usize, u32, usize) {
        let st = state.lock().unwrap();
        let s = st.find("0").expect("session 0");
        let link = &s.windows[s.active];
        let window = st.window_for_link(link);
        (s.windows.len(), link.index, window.panes.len())
    }

    fn is_changed(o: &PrefixOutcome) -> bool {
        matches!(o, PrefixOutcome::Handled { changed: true })
    }

    #[test]
    fn prefix_c_creates_window() {
        let state = fresh_state();
        let out = dispatch_prefix_key(b'c', &state, "0", 80, 24);
        assert!(is_changed(&out));
        assert_eq!(snapshot(&state).0, 2, "one new window");
    }

    #[test]
    fn prefix_c_twice_creates_two_windows() {
        let state = fresh_state();
        dispatch_prefix_key(b'c', &state, "0", 80, 24);
        dispatch_prefix_key(b'c', &state, "0", 80, 24);
        assert_eq!(snapshot(&state).0, 3);
    }

    #[test]
    fn prefix_new_window_is_active_and_sized() {
        let state = fresh_state();
        dispatch_prefix_key(b'c', &state, "0", 100, 40);
        let st = state.lock().unwrap();
        let s = st.find("0").unwrap();
        // The new window (index 1) is the active one.
        assert_eq!(s.windows[s.active].index, 1);
        // Its pane was resized to the client viewport, not left at 80×24.
        let pane = &st.session_window(s, s.active).panes[0].pane;
        assert_eq!((pane.cols(), pane.rows()), (100, 40));
    }

    #[test]
    fn prefix_split_adds_pane() {
        for key in [b'"', b'%'] {
            let state = fresh_state();
            let out = dispatch_prefix_key(key, &state, "0", 80, 24);
            assert!(is_changed(&out));
            assert_eq!(snapshot(&state).2, 2, "split adds a pane (key {key:?})");
        }
    }

    #[test]
    fn prefix_arrow_parser_recognizes_all_directions() {
        for (bytes, expected) in [
            (&b"\x1b[A"[..], Key::Up),
            (&b"\x1b[B"[..], Key::Down),
            (&b"\x1b[C"[..], Key::Right),
            (&b"\x1b[D"[..], Key::Left),
        ] {
            assert_eq!(read_key(bytes), (expected, 3));
        }
    }

    /// Visual conformance regression: a split must tile both pane rectangles;
    /// merely creating/selecting pane 1 recreates the reported fullscreen bug.
    #[test]
    fn prefix_splits_compose_two_visible_rectangles() {
        for (key, separator) in [(b'"', "─"), (b'%', "│")] {
            let state = fresh_state();
            dispatch_prefix_key(key, &state, "0", 80, 24);
            let st = state.lock().unwrap();
            let session = st.find("0").unwrap();
            let win = st.session_window(session, 0);
            assert_eq!(win.panes.len(), 2);
            assert!(win
                .panes
                .iter()
                .all(|p| p.pane.cols() < 80 || p.pane.rows() < 24));
            let frame = compose_frame(&st, "0", 80, 25, 1, 0).expect("compose split");
            assert!(
                frame
                    .windows(separator.len())
                    .any(|w| w == separator.as_bytes()),
                "split key {key:?} must draw a pane boundary"
            );
        }
    }

    #[test]
    fn mixed_splits_compose_a_t_junction() {
        let state = fresh_state();
        dispatch_prefix_key(b'%', &state, "0", 20, 8);
        dispatch_prefix_key(b'"', &state, "0", 20, 8);

        let st = state.lock().unwrap();
        let frame = compose_frame(&st, "0", 20, 9, 1, 0).expect("compose mixed splits");

        assert!(
            contains_seq(&frame, "\x1b[5;11H├".as_bytes()),
            "mixed split boundary must join at a T: {frame:?}"
        );
    }

    #[test]
    fn split_compositor_restores_active_pane_relative_cursor() {
        let state = fresh_state();
        dispatch_prefix_key(b'"', &state, "0", 80, 23);
        let mut st = state.lock().unwrap();
        replace_active_pane_with_inert(&mut st);
        st.active_pane("0").unwrap().feed(b"\x1b[3;5H");
        let frame = compose_frame(&st, "0", 80, 24, 1, 0).expect("compose split");
        assert!(
            contains_seq(&frame, b"\x1b[15;5H\x1b[0 q\x1b[?25h"),
            "active pane cursor must be offset into the lower pane: {frame:?}"
        );
    }

    #[test]
    fn split_compositor_renders_the_active_panes_copy_snapshot() {
        let state = fresh_state();
        dispatch_prefix_key(b'"', &state, "0", 40, 12);
        let mut st = state.lock().unwrap();
        replace_active_pane_with_inert(&mut st);
        let mut output = Vec::new();
        for index in 1..=30 {
            output.extend_from_slice(format!("SPLIT_COPY_{index:02}\r\n").as_bytes());
        }
        st.active_pane("0").unwrap().feed(&output);
        st.set_global_option("mode-keys", "vi");
        st.set_pane_mode("0", Some("copy-mode"))
            .expect("enter copy mode in split");
        st.copy_mode_command("0", "history-top", true, "")
            .expect("move split copy cursor");
        for command in ["begin-selection", "cursor-right", "cursor-right"] {
            st.copy_mode_command("0", command, true, "")
                .unwrap_or_else(|error| panic!("run {command}: {error}"));
        }
        st.active_pane("0")
            .unwrap()
            .feed(b"\r\x1b[KSPLIT_LIVE_AFTER");

        let frame = compose_frame(&st, "0", 40, 13, 1, 0).expect("compose split copy mode");
        assert!(
            contains_seq(&frame, b"SPLIT_COPY_01"),
            "split compositor must paint the top of the frozen snapshot: {frame:?}"
        );
        assert!(
            !contains_seq(&frame, b"SPLIT_LIVE_AFTER"),
            "split compositor must not leak live output over copy mode: {frame:?}"
        );
        assert!(
            contains_seq(&frame, b"\x1b[m\x1b[30m\x1b[43mSPL\x1b[m"),
            "split compositor must overlay the shared selection: {frame:?}"
        );
    }

    #[test]
    fn compositor_highlights_a_stopped_shared_copy_selection() {
        let state = fresh_state();
        let mut st = state.lock().unwrap();
        st.set_global_option("mode-keys", "vi");
        st.active_pane("0").unwrap().feed(b"abcdef");
        st.set_pane_mode("0", Some("copy-mode"))
            .expect("enter copy mode");
        for command in [
            "history-top",
            "start-of-line",
            "cursor-right",
            "begin-selection",
            "cursor-right",
            "cursor-right",
            "stop-selection",
            "cursor-right",
            "cursor-right",
        ] {
            st.copy_mode_command("0", command, true, "")
                .unwrap_or_else(|error| panic!("run {command}: {error}"));
        }

        let frame = compose_frame(&st, "0", 80, 24, 0, 0).expect("compose selection");
        assert!(
            contains_seq(&frame, b"\x1b[1;2H\x1b[m\x1b[30m\x1b[43mbcd\x1b[m",),
            "the retained selection must use the default copy-mode style: {frame:?}"
        );
        assert!(
            contains_seq(&frame, b"\x1b[1;6H\x1b[0 q\x1b[?25h"),
            "moving after stop-selection must move only the copy cursor: {frame:?}"
        );
    }

    #[test]
    fn compositor_distinguishes_current_and_other_literal_search_matches() {
        let state = fresh_state();
        let mut st = state.lock().unwrap();
        st.set_global_option("mode-keys", "vi");
        st.active_pane("0").unwrap().feed(b"alpha one alpha two");
        st.set_pane_mode("0", Some("copy-mode"))
            .expect("enter copy mode");
        for command in ["history-top", "start-of-line"] {
            st.copy_mode_command("0", command, true, "")
                .unwrap_or_else(|error| panic!("run {command}: {error}"));
        }
        st.copy_mode_command_with_argument("0", "search-forward-text", Some("alpha"), true, "")
            .expect("search frozen copy snapshot");

        let frame = compose_frame(&st, "0", 80, 24, 0, 0).expect("compose search");
        assert!(
            contains_seq(&frame, b"\x1b[1;1H\x1b[m\x1b[30m\x1b[46malpha\x1b[m",),
            "other matches must use the default copy-mode match style: {frame:?}"
        );
        assert!(
            contains_seq(&frame, b"\x1b[1;11H\x1b[m\x1b[30m\x1b[45malpha\x1b[m",),
            "the current match must use the default current-match style: {frame:?}"
        );
        assert!(
            contains_seq(&frame, b"\x1b[1;11H\x1b[0 q\x1b[?25h"),
            "vi search must leave the cursor at the current match start: {frame:?}"
        );

        for _ in 0..5 {
            st.copy_mode_command("0", "cursor-right", true, "")
                .expect("move beyond current match");
        }
        let moved = compose_frame(&st, "0", 80, 24, 0, 0).expect("compose moved search");
        assert!(
            !contains_seq(&moved, b"\x1b[30m\x1b[45m"),
            "no match is current after the cursor moves beyond it: {moved:?}"
        );
        assert!(
            contains_seq(&moved, b"\x1b[1;11H\x1b[m\x1b[30m\x1b[46malpha\x1b[m",),
            "vi movement must retain non-current search marks: {moved:?}"
        );
    }

    #[test]
    fn prefix_digit_selects_window() {
        let state = fresh_state();
        dispatch_prefix_key(b'c', &state, "0", 80, 24); // create window 1 (active)
        assert_eq!(snapshot(&state).1, 1);
        dispatch_prefix_key(b'0', &state, "0", 80, 24); // back to window 0
        assert_eq!(snapshot(&state).1, 0);
    }

    #[test]
    fn prefix_n_p_navigate_windows() {
        let state = fresh_state();
        dispatch_prefix_key(b'c', &state, "0", 80, 24); // 0,1 active=1
        dispatch_prefix_key(b'c', &state, "0", 80, 24); // 0,1,2 active=2
        dispatch_prefix_key(b'n', &state, "0", 80, 24); // wraps to 0
        assert_eq!(snapshot(&state).1, 0);
        dispatch_prefix_key(b'p', &state, "0", 80, 24); // wraps back to 2
        assert_eq!(snapshot(&state).1, 2);
    }

    #[test]
    fn prefix_l_last_window() {
        let state = fresh_state();
        dispatch_prefix_key(b'c', &state, "0", 80, 24); // active 1 (last 0)
        dispatch_prefix_key(b'0', &state, "0", 80, 24); // active 0 (last 1)
        dispatch_prefix_key(b'l', &state, "0", 80, 24); // back to 1
        assert_eq!(snapshot(&state).1, 1);
    }

    #[test]
    fn prefix_next_window_on_single_window_is_bell() {
        let state = fresh_state();
        // Only one window: next-window fails (tmux's bell) — no change, no redraw.
        let out = dispatch_prefix_key(b'n', &state, "0", 80, 24);
        assert!(!is_changed(&out));
        assert_eq!(snapshot(&state).0, 1);
    }

    #[test]
    fn prefix_d_detaches() {
        let state = fresh_state();
        assert!(matches!(
            dispatch_prefix_key(b'd', &state, "0", 80, 24),
            PrefixOutcome::Detach
        ));
    }

    #[test]
    fn prefix_prefix_sends_literal() {
        let state = fresh_state();
        assert!(matches!(
            dispatch_prefix_key(PREFIX, &state, "0", 80, 24),
            PrefixOutcome::SendPrefix(_)
        ));
    }

    #[test]
    fn prefix_unknown_key_is_noop() {
        let state = fresh_state();
        let out = dispatch_prefix_key(b'Z', &state, "0", 80, 24);
        assert!(!is_changed(&out));
        assert_eq!(snapshot(&state), (1, 0, 1));
    }
}

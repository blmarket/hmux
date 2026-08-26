//! Per-client server-side state: what each attached client is showing, what
//! it has been told, and what it has been asked.
//!
//! [`ClientRenderRegistry`] holds one slot per attached client — its size,
//! flags, and pending invalidations, and it is where a wakeup is published to
//! one session's clients or to every control client.
//! [`ClientPromptRegistry`] is the parallel table for
//! `command-prompt`, `confirm-before`, and the other interactive requests a
//! command can park on a client.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io;
use std::os::fd::{AsFd, AsRawFd, RawFd};
use std::rc::Rc;
use std::time::{Duration, Instant};

use bytes::Bytes;

use super::{
    ClientAction, ClientActionResult, ClientKey, ClientMessage, ClientMessageResult,
    DEFAULT_KEY_TABLE, MessageLogEntry, OverlayRequest, ServerState, now_epoch, now_micros,
};
use crate::platform::{CurrentPlatform, OutputWakeup, Platform};
use crate::server::options::SessionHook;
use crate::server::pane::NativePaneObservation;
use crate::server::term::ResolvedTerm;
use crate::sync::{Completion, CompletionSender, completion_pair};

/// Client-scoped `command-prompt -k` routing. This is an internal server
/// capability, deliberately kept out of the public server trait.
pub(crate) struct ClientPromptRegistry {
    inner: RefCell<ClientPromptRegistryState>,
    activity: Cell<u64>,
}

#[derive(Default)]
struct ClientPromptRegistryState {
    next_id: u64,
    clients: BTreeMap<u64, ClientPromptEntry>,
}

struct ClientPromptEntry {
    name: String,
    tty_name: String,
    slot: Rc<ClientPromptSlot>,
    activity: Rc<Cell<u64>>,
}

struct ClientPromptSlot {
    inner: RefCell<ClientPromptSlotState>,
    wakeup: <CurrentPlatform as Platform>::OutputWakeup,
}

#[derive(Default)]
struct ClientPromptSlotState {
    queued_command: Option<QueuedCommandPrompt>,
    active: bool,
}

struct QueuedCommandPrompt {
    args: Vec<String>,
    reply: PromptReply,
}

/// Registration owned by one interactive attach loop.
pub(crate) struct ClientPromptAttachment {
    registry: Rc<ClientPromptRegistry>,
    id: u64,
    slot: Rc<ClientPromptSlot>,
    activity: Rc<Cell<u64>>,
}

pub(crate) struct ActiveCommandPrompt {
    slot: Rc<ClientPromptSlot>,
    args: Vec<String>,
    reply: Option<PromptReply>,
}

#[derive(Clone, Debug)]
pub(crate) struct PromptCompletion {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) exit: i32,
    pub(crate) inserted: bool,
}

/// The answering end of an interactive prompt, menu, popup or confirmation.
///
/// One command queue waits for one answer, so the reply is a one-shot: only the
/// first [`send`](Self::send) is delivered. It is clonable because the client
/// context carrying it is cloned into every nested command that might answer,
/// and dropping the last clone unanswered reports `None` — the disconnect the
/// blocking receive this replaces saw when its sender went away.
#[derive(Clone)]
pub(crate) struct PromptReply {
    sender: Rc<RefCell<Option<CompletionSender<Option<PromptCompletion>>>>>,
}

impl PromptReply {
    pub(crate) fn new() -> io::Result<(Self, Completion<Option<PromptCompletion>>)> {
        let (completion, sender) = completion_pair()?;
        Ok((
            Self {
                sender: Rc::new(RefCell::new(Some(sender))),
            },
            completion,
        ))
    }

    pub(crate) fn send(&self, completion: Option<PromptCompletion>) {
        // Taken out from under the borrow: completing writes to a descriptor
        // whose reader may answer again on the same turn.
        let sender = self.sender.borrow_mut().take();
        if let Some(sender) = sender {
            sender.complete(completion);
        }
    }
}

impl std::fmt::Debug for PromptReply {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PromptReply")
            .field(
                "answered",
                &self
                    .sender
                    .try_borrow()
                    .is_ok_and(|sender| sender.is_none()),
            )
            .finish()
    }
}

impl Drop for PromptReply {
    fn drop(&mut self) {
        if Rc::strong_count(&self.sender) == 1 {
            self.send(None);
        }
    }
}

pub(crate) enum CommandPromptRequestResult {
    /// The prompt is up and `-w` asked to wait for the answer.
    Waiting(Completion<Option<PromptCompletion>>),
    Queued,
    NoCurrentClient,
    TargetNotFound,
    Busy,
}

/// Coalesced reasons an attached client's compositor must inspect server state.
///
/// Pane output keeps its dedicated notification path; these bits cover state
/// changes made by command clients while an attach loop may be blocked waiting
/// on a pane that is no longer active.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RenderInvalidation(u8);

impl RenderInvalidation {
    pub(crate) const STATUS: Self = Self(1 << 0);
    pub(crate) const LAYOUT: Self = Self(1 << 1);
    pub(crate) const SESSION_GONE: Self = Self(1 << 2);
    pub(crate) const RESET_MODE: Self = Self(1 << 3);
    pub(crate) const MODE: Self = Self(1 << 4);
    pub(crate) const TERMINAL: Self = Self(1 << 5);
    /// A control notification was fired. Published to every control client,
    /// whichever session it is on: the event stream is server-wide, and an
    /// event in another session is exactly what a per-session wake misses.
    pub(crate) const NOTIFY: Self = Self(1 << 6);

    fn bits(self) -> u8 {
        self.0
    }

    pub(crate) fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub(crate) fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

impl std::ops::BitOr for RenderInvalidation {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// Per-client state-change notifications, scoped by stable session id.
pub(crate) struct ClientRenderRegistry {
    inner: RefCell<ClientRenderRegistryState>,
    /// Bumped whenever the set of clients, or which session one is on,
    /// changes. Lets the server loop skip the lifecycle sweep when nothing
    /// the policies depend on has moved.
    generation: Cell<u64>,
}

#[derive(Default)]
struct ClientRenderRegistryState {
    next_id: u64,
    /// Ticked every time a client declares a terminal size. Ordering these is
    /// what lets `window-size latest` name the client that sized a window most
    /// recently, without every resize path having to carry a client identity.
    next_size_seq: u64,
    clients: BTreeMap<u64, ClientRenderEntry>,
}

pub(super) struct ClientRenderEntry {
    pub(super) session_id: u32,
    pub(super) name: String,
    pub(super) term: String,
    pid: Option<i32>,
    /// The uid of the process on the other end of this client's socket, and
    /// the login name it maps to — tmux's `proc_get_peer_uid` and the `c->user`
    /// it caches from it. Empty until the attach path reports them.
    uid: Option<u32>,
    user: String,
    /// The environment the client attached with — tmux's `c->environ`, which
    /// `switch-client` copies `update-environment` names out of.
    environment: Vec<String>,
    pub(super) cols: u16,
    pub(super) rows: u16,
    /// The client terminal's cell size in pixels — tmux's `tty->xpixel` and
    /// `tty->ypixel` — or zero while the terminal has reported none.
    pub(super) xpixel: u16,
    pub(super) ypixel: u16,
    pub(super) control_mode: bool,
    size_changed: bool,
    /// When this client last declared its terminal size, on the registry's
    /// monotonic sequence — tmux's `w->latest` ordering.
    pub(super) size_seq: u64,
    /// The window this client's `refresh-client` pan applies to, and the pan
    /// itself — tmux's `c->pan_window`/`c->pan_ox`/`c->pan_oy`. A pan is
    /// abandoned as soon as the client shows a different window or the window
    /// shrinks to fit, which is what makes `refresh-client -c` a reset.
    pan_window: Option<u32>,
    pan_ox: u16,
    pan_oy: u16,
    /// Identity bits the client sent at handshake time (`CLIENT_UTF8`, …),
    /// needed to rebuild the display flag string server-side.
    identified: i64,
    /// Whether the client's terminal currently has focus.
    pub(super) focused: bool,
    /// The theme the client's terminal last reported (`dark`/`light`), empty
    /// until it says.
    theme: String,
    /// The key table the client is currently in — tmux's `c->keytable`, which
    /// `#{client_key_table}` and `#{client_prefix}` report.
    key_table: String,
    /// When this client last sent a key, in microseconds since the epoch —
    /// tmux's `c->activity_time`, which orders `cmd_find_best_client`.
    activity_micros: i64,
    /// When this client attached, in microseconds since the epoch — tmux's
    /// `c->creation_time`, which `#{client_created}` reports.
    created_micros: i64,
    /// The session this client showed before its last switch — tmux's
    /// `c->last_session`, cleared when a switch lands on the same session.
    last_session_id: Option<u32>,
    /// Bytes written to this client's terminal — tmux's `c->written`.
    written: u64,
    /// tmux's `CLIENT_SUSPENDED`: set while the client is stopped by
    /// `suspend-client` and cleared when it wakes back up.
    suspended: bool,
    /// When a clipboard query was last sent to this client's terminal.
    clipboard_query_at: Option<Instant>,
    flag_state: ClientFlagState,
    pub(super) terminal: Option<ResolvedTerm>,
    /// What the terminal called itself in its XTVERSION reply, empty until it
    /// answers — `#{client_termtype}`.
    pub(super) term_type: String,
    slot: Rc<ClientRenderSlot>,
    /// This client's `#()` job tree — tmux's `c->jobs`, shared by every format
    /// the client expands (its status line and the commands it runs). Dropped
    /// with the client, which kills the jobs still running in it.
    format_jobs: Rc<crate::server::status::FormatJobRegistry>,
}

impl ClientRenderEntry {
    /// Apply one `refresh-client -f` value.
    fn apply_flag_value(&mut self, value: &str) {
        self.flag_state.apply_flags(value);
    }

    /// This client's `#{client_flags}` string — tmux's
    /// `server_client_get_flags`, rendered from the flag set plus the handshake
    /// bits and the focus this entry is holding.
    fn display_flags(&self) -> String {
        self.flag_state
            .display_flags_full(self.identified, self.control_mode, self.focused)
    }

    /// Whether this client refuses input — tmux's `CLIENT_READONLY`.
    pub(super) fn read_only(&self) -> bool {
        self.flag_state.read_only
    }

    /// Whether this client's terminal size is excluded from window sizing —
    /// tmux's `CLIENT_IGNORESIZE`.
    pub(super) fn ignore_size(&self) -> bool {
        self.flag_state.ignore_size
    }

    /// Whether this client's terminal size constrains window sizing: a
    /// control client contributes nothing until it declares a size.
    pub(super) fn counts_for_sizing(&self) -> bool {
        !self.control_mode || self.size_changed
    }

    fn viewport(&self) -> ViewportClient {
        ViewportClient {
            session_id: self.session_id,
            control_mode: self.control_mode,
            cols: self.cols,
            rows: self.rows,
            pan_window: self.pan_window,
            pan_ox: self.pan_ox,
            pan_oy: self.pan_oy,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// A value snapshot of one client for the format and chooser consumers
/// outside this module (`#{client_*}` vars, `list-clients`, `choose-client`).
/// Server-internal paths read the registry entries directly instead of
/// materializing these.
pub(crate) struct ClientSnapshot {
    pub(crate) session_id: u32,
    pub(crate) name: String,
    pub(crate) term: String,
    pub(crate) pid: Option<i32>,
    /// The client's peer uid and the login name it maps to — `#{client_uid}`
    /// and `#{client_user}`.
    pub(crate) uid: Option<u32>,
    pub(crate) user: String,
    pub(crate) cols: u16,
    pub(crate) rows: u16,
    /// The terminal's cell size in pixels, zero until it reports one —
    /// `#{client_cell_width}` and `#{client_cell_height}`.
    pub(crate) xpixel: u16,
    pub(crate) ypixel: u16,
    pub(crate) flags: String,
    pub(crate) read_only: bool,
    pub(crate) control_mode: bool,
    /// The window this client's pan applies to, and the pan itself.
    pub(crate) pan_window: Option<u32>,
    pub(crate) pan_ox: u16,
    pub(crate) pan_oy: u16,
    /// The theme this client's terminal reported (`dark`/`light`), empty until
    /// it says — tmux's `#{client_theme}`.
    pub(crate) theme: String,
    /// The key table the client is in — tmux's `#{client_key_table}`.
    pub(crate) key_table: String,
    /// When this client last sent a key, in microseconds since the epoch.
    pub(crate) activity_micros: i64,
    /// When this client attached, in microseconds since the epoch.
    pub(crate) created_micros: i64,
    /// The session this client showed before its last switch, if any.
    pub(crate) last_session_id: Option<u32>,
    /// The features resolved for this client's terminal, in tmux's
    /// feature-bit order — `#{client_termfeatures}`.
    pub(crate) termfeatures: String,
    /// The name the terminal gave for itself — `#{client_termtype}`.
    pub(crate) termtype: String,
    /// Bytes written to this client's terminal — `#{client_written}`.
    pub(crate) written: u64,
    /// Whether the client is stopped on `suspend-client`, which is what tmux's
    /// `sort_get_clients` skips a `CLIENT_SUSPENDED` client for.
    pub(crate) suspended: bool,
}

/// A question a pane asked that only the attached terminal can answer, waiting
/// for the answer to come back through that client's input — tmux's
/// `input_request`.
#[derive(Clone, Debug)]
pub(crate) struct TerminalRequest {
    /// The client the question went to. Its input is the only place the answer
    /// can arrive, so an answer from any other client is not this one's.
    pub(super) client: String,
    /// The pane that asked, which is where the answer is written. It is not
    /// necessarily the client's active pane — that is the whole point of
    /// remembering the request.
    pub(super) pane_id: u32,
    pub(super) kind: TerminalRequestKind,
    pub(super) at: Instant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TerminalRequestKind {
    /// OSC 4: the palette entry asked about, and whether the pane's question
    /// ended with BEL, which is how its answer ends too.
    Palette { index: u8, bel: bool },
    /// OSC 52: a clipboard read. `bel` again mirrors the pane's question.
    Clipboard { bel: bool },
}

/// What an attached terminal answered, as the client's input scanner recovered
/// it.
pub(crate) use hmux_vt::TerminalAnswer as TerminalReply;

/// The client-side inputs of the oversized-window viewport calculation.
#[derive(Clone, Copy)]
pub(crate) struct ViewportClient {
    pub(crate) session_id: u32,
    pub(crate) control_mode: bool,
    pub(crate) cols: u16,
    pub(crate) rows: u16,
    pub(crate) pan_window: Option<u32>,
    pub(crate) pan_ox: u16,
    pub(crate) pan_oy: u16,
}

impl ClientSnapshot {
    pub(crate) fn viewport(&self) -> ViewportClient {
        ViewportClient {
            session_id: self.session_id,
            control_mode: self.control_mode,
            cols: self.cols,
            rows: self.rows,
            pan_window: self.pan_window,
            pan_ox: self.pan_ox,
            pan_oy: self.pan_oy,
        }
    }
}

#[derive(Clone)]
pub(crate) struct ControlPaneSnapshot {
    pub(crate) id: u32,
    pub(crate) runtime_id: u64,
    pub(crate) observation: Rc<NativePaneObservation>,
}

#[derive(Clone)]
pub(crate) struct ControlWindowSnapshot {
    pub(crate) id: u32,
    pub(crate) index: u32,
    pub(crate) panes: Vec<ControlPaneSnapshot>,
}

/// What a control client reads its own session out of: the panes it streams
/// output from, and the layouts `refresh-client -C` dumps.
///
/// Notifications are not derived from this — they arrive on the server-wide
/// notification log, resolved where the event fired — so nothing here exists
/// to be compared against a previous copy of itself.
#[derive(Clone)]
pub(crate) struct ControlStateSnapshot {
    pub(crate) session_id: u32,
    pub(crate) session_name: String,
    pub(crate) windows: BTreeMap<u32, ControlWindowSnapshot>,
}

const CLIENT_READONLY: i64 = 0x800;

const CLIENT_UTF8: i64 = 0x10000;

const CLIENT_IGNORESIZE: i64 = 0x20000;

const CLIENT_CONTROL_NOOUTPUT: i64 = 0x4000000;

const CLIENT_ACTIVEPANE: i64 = 0x80000000;

const CLIENT_CONTROL_PAUSEAFTER: i64 = 0x100000000;

const CLIENT_CONTROL_WAITEXIT: i64 = 0x200000000;

const CLIENT_NO_DETACH_ON_DESTROY: i64 = 0x8000000000;

/// The `refresh-client -f` flag set of one client.
///
/// Both the client that owns the terminal and the registry entry other clients
/// see keep one of these, so a `refresh-client -t other -f …` is visible to
/// `#{client_flags}` and to the destroy policy without waiting for the target
/// client to wake up.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ClientFlagState {
    pub(crate) pause_after: Option<Duration>,
    pub(crate) no_output: bool,
    pub(crate) wait_exit: bool,
    pub(crate) read_only: bool,
    pub(crate) ignore_size: bool,
    pub(crate) active_pane: bool,
    pub(crate) no_detach_on_destroy: bool,
}

impl ClientFlagState {
    /// The flags an attach command's `-f` carries, in either of the spellings
    /// the option parser accepts. tmux hands the value straight to
    /// `server_client_set_flags` as the client attaches.
    pub(crate) fn from_attach_args(args: &[String]) -> Self {
        let mut flags = Self::default();
        let mut index = 0;
        while index < args.len() {
            let value = if args[index] == "-f" {
                index += 1;
                args.get(index).map(String::as_str)
            } else {
                args[index]
                    .strip_prefix("-f")
                    .filter(|value| !value.is_empty())
            };
            if let Some(value) = value {
                flags.apply_flags(value);
            }
            index += 1;
        }
        if crate::server::command::command_flag("attach-session", args, 'r') {
            flags.read_only = true;
            flags.ignore_size = true;
        }
        flags
    }

    pub(crate) fn apply_flags(&mut self, value: &str) {
        for flag in value.split(',') {
            let (clear, flag) = flag
                .strip_prefix('!')
                .map_or((false, flag), |flag| (true, flag));
            if flag == "pause-after" || flag.starts_with("pause-after=") {
                if clear {
                    self.pause_after = None;
                } else {
                    let seconds = flag
                        .strip_prefix("pause-after=")
                        .and_then(|value| value.parse::<u64>().ok())
                        .unwrap_or(0);
                    self.pause_after = Some(Duration::from_secs(seconds));
                }
            } else {
                let enabled = !clear;
                match flag {
                    "no-output" => self.no_output = enabled,
                    "wait-exit" => self.wait_exit = enabled,
                    // An established read-only client cannot clear itself with
                    // refresh-client; switch-client -r is the owner escape.
                    "read-only" if enabled || !self.read_only => self.read_only = enabled,
                    "ignore-size" => self.ignore_size = enabled,
                    "active-pane" => self.active_pane = enabled,
                    "no-detach-on-destroy" => self.no_detach_on_destroy = enabled,
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn client_flags(&self, identified: i64) -> i64 {
        let mut flags = identified;
        for (enabled, flag) in [
            (self.no_output, CLIENT_CONTROL_NOOUTPUT),
            (self.wait_exit, CLIENT_CONTROL_WAITEXIT),
            (self.pause_after.is_some(), CLIENT_CONTROL_PAUSEAFTER),
            (self.read_only, CLIENT_READONLY),
            (self.ignore_size, CLIENT_IGNORESIZE),
            (self.active_pane, CLIENT_ACTIVEPANE),
            (self.no_detach_on_destroy, CLIENT_NO_DETACH_ON_DESTROY),
        ] {
            if enabled {
                flags |= flag;
            } else {
                flags &= !flag;
            }
        }
        flags
    }

    pub(crate) fn display_flags(&self, identified: i64) -> String {
        self.display_flags_with(identified, true)
    }

    /// tmux's `server_client_get_flags` ordering. `control_mode` selects the
    /// `control-mode` marker an interactive attach does not carry.
    pub(crate) fn display_flags_with(&self, identified: i64, control_mode: bool) -> String {
        self.display_flags_full(identified, control_mode, true)
    }

    pub(crate) fn display_flags_full(
        &self,
        identified: i64,
        control_mode: bool,
        focused: bool,
    ) -> String {
        let mut flags = vec!["attached"];
        if focused {
            flags.push("focused");
        }
        if control_mode {
            flags.push("control-mode");
        }
        if self.ignore_size {
            flags.push("ignore-size");
        }
        if self.no_detach_on_destroy {
            flags.push("no-detach-on-destroy");
        }
        if self.no_output {
            flags.push("no-output");
        }
        if self.wait_exit {
            flags.push("wait-exit");
        }
        let pause_after = self
            .pause_after
            .map(|duration| format!("pause-after={}", duration.as_secs()));
        if let Some(pause_after) = pause_after.as_deref() {
            flags.push(pause_after);
        }
        if self.read_only {
            flags.push("read-only");
        }
        if self.active_pane {
            flags.push("active-pane");
        }
        if identified & CLIENT_UTF8 != 0 {
            flags.push("UTF-8");
        }
        flags.join(",")
    }
}

struct ClientRenderSlot {
    pending: Cell<u8>,
    action: RefCell<Option<ClientAction>>,
    messages: RefCell<VecDeque<ClientMessage>>,
    /// Bytes an application asked to have written to this client's terminal
    /// verbatim. Kept out of `action` for the same reason as `flag_updates`,
    /// and ordered because a payload is meaningless out of sequence.
    client_output: RefCell<VecDeque<Bytes>>,
    /// `refresh-client -f` values aimed at this client by *another* client.
    /// Kept out of `action` because flag updates must not displace a queued
    /// switch or detach, and several may arrive before the client next runs.
    flag_updates: RefCell<Vec<String>>,
    wakeup: <CurrentPlatform as Platform>::OutputWakeup,
}

impl ClientRenderSlot {
    /// Add `reason` to what this client must re-examine, and wake it.
    fn publish(&self, reason: RenderInvalidation) {
        self.pending.set(self.pending.get() | reason.bits());
        let _ = self.wakeup.wake();
    }
}

/// Registration owned by one interactive attach loop.
pub(crate) struct ClientRenderAttachment {
    registry: Rc<ClientRenderRegistry>,
    id: u64,
    slot: Rc<ClientRenderSlot>,
    format_jobs: Rc<crate::server::status::FormatJobRegistry>,
}

impl ClientRenderRegistry {
    pub(super) fn new() -> Self {
        Self {
            inner: RefCell::new(ClientRenderRegistryState::default()),
            generation: Cell::new(0),
        }
    }

    fn bump_generation(&self) {
        self.generation.set(self.generation.get().wrapping_add(1));
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation.get()
    }

    #[cfg(test)]
    pub(crate) fn attach(
        self: &Rc<Self>,
        session_id: u32,
        name: String,
    ) -> io::Result<ClientRenderAttachment> {
        self.attach_with_details(
            session_id,
            name,
            String::new(),
            None,
            80,
            24,
            0,
            ClientFlagState::default(),
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn attach_with_details(
        self: &Rc<Self>,
        session_id: u32,
        name: String,
        term: String,
        pid: Option<i32>,
        cols: u16,
        rows: u16,
        identified: i64,
        flag_state: ClientFlagState,
        control_mode: bool,
    ) -> io::Result<ClientRenderAttachment> {
        let wakeup = CurrentPlatform::new_output_wakeup()?;
        wakeup.clear()?;
        let slot = Rc::new(ClientRenderSlot {
            pending: Cell::new(0),
            action: RefCell::new(None),
            messages: RefCell::new(VecDeque::new()),
            client_output: RefCell::new(VecDeque::new()),
            flag_updates: RefCell::new(Vec::new()),
            wakeup,
        });
        let format_jobs = Rc::new(crate::server::status::FormatJobRegistry::new(self));
        let mut inner = self.inner.borrow_mut();
        let id = inner.next_id;
        inner.next_id = inner.next_id.wrapping_add(1);
        inner.next_size_seq = inner.next_size_seq.wrapping_add(1);
        let size_seq = inner.next_size_seq;
        inner.clients.insert(
            id,
            ClientRenderEntry {
                session_id,
                name,
                term,
                pid,
                uid: None,
                user: String::new(),
                environment: Vec::new(),
                cols,
                rows,
                // A client reports its pixel size after the handshake, if at
                // all; until then the window falls back to tmux's defaults.
                xpixel: 0,
                ypixel: 0,
                control_mode,
                size_changed: !control_mode,
                size_seq,
                pan_window: None,
                pan_ox: 0,
                pan_oy: 0,
                identified,
                focused: true,
                theme: String::new(),
                key_table: DEFAULT_KEY_TABLE.to_string(),
                activity_micros: now_micros(),
                created_micros: now_micros(),
                last_session_id: None,
                written: 0,
                suspended: false,
                clipboard_query_at: None,
                flag_state,
                terminal: None,
                term_type: String::new(),
                slot: Rc::clone(&slot),
                format_jobs: Rc::clone(&format_jobs),
            },
        );
        let peers = inner
            .clients
            .iter()
            .filter(|(client_id, _)| **client_id != id)
            .map(|(_, entry)| Rc::clone(&entry.slot))
            .collect::<Vec<_>>();
        drop(inner);
        self.bump_generation();
        for peer in peers {
            peer.publish(RenderInvalidation::STATUS);
        }
        Ok(ClientRenderAttachment {
            registry: Rc::clone(self),
            id,
            slot,
            format_jobs,
        })
    }

    pub(super) fn snapshots(&self) -> Vec<ClientSnapshot> {
        let inner = self.inner.borrow();
        inner
            .clients
            .values()
            .map(|entry| ClientSnapshot {
                session_id: entry.session_id,
                name: entry.name.clone(),
                term: entry.term.clone(),
                pid: entry.pid,
                uid: entry.uid,
                user: entry.user.clone(),
                cols: entry.cols,
                rows: entry.rows,
                xpixel: entry.xpixel,
                ypixel: entry.ypixel,
                flags: entry.display_flags(),
                read_only: entry.read_only(),
                control_mode: entry.control_mode,
                pan_window: entry.pan_window,
                pan_ox: entry.pan_ox,
                pan_oy: entry.pan_oy,
                theme: entry.theme.clone(),
                key_table: entry.key_table.clone(),
                activity_micros: entry.activity_micros,
                created_micros: entry.created_micros,
                last_session_id: entry.last_session_id,
                termfeatures: entry
                    .terminal
                    .as_ref()
                    .map(|terminal| terminal.feature_list())
                    .unwrap_or_default(),
                termtype: entry.term_type.clone(),
                written: entry.written,
                suspended: entry.suspended,
            })
            .collect()
    }

    /// Run `f` over the live registry entries without materializing
    /// snapshots. `f` must not touch the registry itself: the entries are
    /// handed out under the registry borrow.
    pub(super) fn with_entries<R>(
        &self,
        f: impl FnOnce(std::collections::btree_map::Values<'_, u64, ClientRenderEntry>) -> R,
    ) -> R {
        let inner = self.inner.borrow();
        f(inner.clients.values())
    }

    /// The viewport inputs of the client called `name`.
    pub(super) fn viewport_client(&self, name: &str) -> Option<ViewportClient> {
        self.with_entries(|mut entries| {
            entries
                .find(|entry| entry.name == name)
                .map(ClientRenderEntry::viewport)
        })
    }

    /// The job tree and current session of the client called `name`, which is
    /// how a command reaches the tree of the client that ran it.
    pub(super) fn client_format_jobs(
        &self,
        name: &str,
    ) -> Option<(Rc<crate::server::status::FormatJobRegistry>, u32)> {
        let inner = self.inner.borrow();
        inner
            .clients
            .values()
            .find(|entry| entry.name == name)
            .map(|entry| (Rc::clone(&entry.format_jobs), entry.session_id))
    }

    pub(crate) fn all_format_jobs(&self) -> Vec<Rc<crate::server::status::FormatJobRegistry>> {
        let inner = self.inner.borrow();
        inner
            .clients
            .values()
            .map(|entry| Rc::clone(&entry.format_jobs))
            .collect()
    }

    pub(super) fn names_for_sessions(&self, session_ids: &BTreeSet<u32>) -> Vec<String> {
        let inner = self.inner.borrow();
        inner
            .clients
            .values()
            .filter(|entry| session_ids.contains(&entry.session_id))
            .map(|entry| entry.name.clone())
            .collect()
    }

    /// Deliver one window alert to every non-control client of `session_id`.
    pub(super) fn announce_alert(
        &self,
        session_id: u32,
        bell: bool,
        text: Option<String>,
        duration_ms: u64,
    ) {
        let inner = self.inner.borrow();
        for entry in inner
            .clients
            .values()
            // tmux's `alerts_set_message` skips control clients outright.
            .filter(|entry| entry.session_id == session_id && !entry.control_mode)
        {
            let mut messages = entry.slot.messages.borrow_mut();
            messages.push_back(ClientMessage {
                text: text.clone().unwrap_or_default(),
                duration_ms,
                bell,
            });
            let _ = entry.slot.wakeup.wake();
        }
    }

    /// Queue bytes an application wrote for the terminals of every non-control
    /// client of `sessions`. tmux's `tty_client_ready` skips a client with no
    /// terminal of its own, which is what leaves control clients out.
    /// Write straight to one named client's terminal, for the questions a pane
    /// asks that only that client can answer.
    pub(super) fn write_client_output_named(&self, client: &str, bytes: Bytes) {
        let inner = self.inner.borrow();
        let Some(entry) = inner
            .clients
            .values()
            .find(|entry| entry.name == client && !entry.control_mode)
        else {
            return;
        };
        {
            let mut queued = entry.slot.client_output.borrow_mut();
            queued.push_back(bytes);
        }
        let _ = entry.slot.wakeup.wake();
    }

    /// Whether a `refresh-client -l` question to this client is still waiting
    /// for its answer — tmux's `TTY_OSC52QUERY`.
    pub(super) fn clipboard_query_outstanding(&self, client: &str) -> bool {
        self.inner
            .borrow()
            .clients
            .values()
            .any(|entry| entry.name == client && entry.clipboard_query_at.is_some())
    }

    /// Clear that flag, reporting whether it was set, so the answer is turned
    /// into a paste buffer exactly once.
    pub(super) fn take_clipboard_query(&self, client: &str) -> bool {
        let mut inner = self.inner.borrow_mut();
        inner
            .clients
            .values_mut()
            .find(|entry| entry.name == client)
            .is_some_and(|entry| entry.clipboard_query_at.take().is_some())
    }

    pub(super) fn write_client_output(&self, sessions: &BTreeSet<u32>, bytes: Bytes) {
        let inner = self.inner.borrow();
        for entry in inner
            .clients
            .values()
            .filter(|entry| sessions.contains(&entry.session_id) && !entry.control_mode)
        {
            let mut queued = entry.slot.client_output.borrow_mut();
            queued.push_back(bytes.clone());
            let _ = entry.slot.wakeup.wake();
        }
    }

    pub(crate) fn publish_session(&self, session_id: u32, reason: RenderInvalidation) {
        let inner = self.inner.borrow();
        for entry in inner
            .clients
            .values()
            .filter(|entry| entry.session_id == session_id)
        {
            entry.slot.publish(reason);
        }
    }

    /// Wake every control-mode client, which is the audience of the
    /// server-wide notification log.
    pub(super) fn publish_control(&self, reason: RenderInvalidation) {
        let inner = self.inner.borrow();
        for entry in inner.clients.values().filter(|entry| entry.control_mode) {
            entry.slot.publish(reason);
        }
    }

    pub(super) fn publish_all(&self, reason: RenderInvalidation) {
        let inner = self.inner.borrow();
        for entry in inner.clients.values() {
            entry.slot.publish(reason);
        }
    }

    fn queue_lock(entry: &ClientRenderEntry, command: &str) {
        // tmux's `server_lock_client` refuses to send an empty MSG_LOCK, so an
        // empty `lock-command` leaves the client running rather than suspending
        // it on a command that would do nothing.
        if command.is_empty() {
            return;
        }
        {
            let mut action = entry.slot.action.borrow_mut();
            *action = Some(ClientAction::Lock(command.to_string()));
            let _ = entry.slot.wakeup.wake();
        }
    }

    pub(super) fn lock_all(&self, commands: &BTreeMap<u32, String>) {
        let inner = self.inner.borrow();
        for entry in inner.clients.values() {
            if let Some(command) = commands.get(&entry.session_id) {
                Self::queue_lock(entry, command);
            }
        }
    }

    pub(super) fn lock_session(&self, session_id: u32, command: &str) {
        let inner = self.inner.borrow();
        for entry in inner
            .clients
            .values()
            .filter(|entry| entry.session_id == session_id)
        {
            Self::queue_lock(entry, command);
        }
    }

    pub(super) fn lock_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        commands: &BTreeMap<u32, String>,
    ) -> ClientActionResult {
        let inner = self.inner.borrow();
        let explicit = target.map(|target| target.strip_suffix(':').unwrap_or(target));
        let selected = if let Some(target) = explicit {
            inner.clients.values().find(|entry| {
                entry.name == target
                    || entry
                        .name
                        .strip_prefix("/dev/")
                        .is_some_and(|tty| tty == target)
            })
        } else {
            invoking_tty.and_then(|tty| inner.clients.values().find(|entry| entry.name == tty))
        };
        let Some(entry) = selected else {
            return if explicit.is_some() {
                ClientActionResult::TargetNotFound
            } else {
                ClientActionResult::NoCurrentClient
            };
        };
        if let Some(command) = commands.get(&entry.session_id) {
            Self::queue_lock(entry, command);
        }
        ClientActionResult::Queued
    }

    fn client_entry<'a>(
        inner: &'a ClientRenderRegistryState,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> Result<&'a ClientRenderEntry, ClientActionResult> {
        let explicit = target.map(|target| target.strip_suffix(':').unwrap_or(target));
        let selected = if let Some(target) = explicit {
            inner.clients.values().find(|entry| {
                entry.name == target
                    || entry
                        .name
                        .strip_prefix("/dev/")
                        .is_some_and(|tty| tty == target)
            })
        } else {
            invoking_tty.and_then(|tty| inner.clients.values().find(|entry| entry.name == tty))
        };
        selected.ok_or(if explicit.is_some() {
            ClientActionResult::TargetNotFound
        } else {
            ClientActionResult::NoCurrentClient
        })
    }

    fn client_id_for(
        inner: &ClientRenderRegistryState,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> Result<u64, ClientActionResult> {
        let explicit = target.map(|target| target.strip_suffix(':').unwrap_or(target));
        let selected = if let Some(target) = explicit {
            inner.clients.iter().find(|(_, entry)| {
                entry.name == target
                    || entry
                        .name
                        .strip_prefix("/dev/")
                        .is_some_and(|tty| tty == target)
            })
        } else {
            invoking_tty.and_then(|tty| inner.clients.iter().find(|(_, entry)| entry.name == tty))
        };
        selected.map(|(id, _)| *id).ok_or(if explicit.is_some() {
            ClientActionResult::TargetNotFound
        } else {
            ClientActionResult::NoCurrentClient
        })
    }

    /// Queue `refresh-client -f` values for a client other than the one running
    /// the command. The client applies them to its own flag state on its next
    /// turn; the registry copy is what `#{client_flags}` and the destroy policy
    /// read in the meantime.
    pub(super) fn refresh_client_flags(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        values: &[String],
    ) -> ClientActionResult {
        let mut inner = self.inner.borrow_mut();
        let id = match Self::client_id_for(&inner, target, invoking_tty) {
            Ok(id) => id,
            Err(result) => return result,
        };
        let entry = inner.clients.get_mut(&id).expect("selected client present");
        for value in values {
            entry.apply_flag_value(value);
        }
        {
            let mut pending = entry.slot.flag_updates.borrow_mut();
            pending.extend(values.iter().cloned());
        }
        let _ = entry.slot.wakeup.wake();
        ClientActionResult::Queued
    }

    /// tmux's `switch-client -r`: flip the target client's read-only bit,
    /// carrying `ignore-size` with it. This is the path that can clear a
    /// read-only client, so it moves the flag state directly instead of going
    /// through a `refresh-client -f` value, which refuses to clear it.
    ///
    /// Returns the client the toggle landed on, so the session change that
    /// follows moves the same client.
    pub(super) fn toggle_client_read_only(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> Result<String, ClientActionResult> {
        let mut inner = self.inner.borrow_mut();
        let id = match Self::client_id_for(&inner, target, invoking_tty) {
            Ok(id) => id,
            // A plain command client owns no terminal of its own; tmux's
            // `cmd_find_current_client` resolves that to the sole attached
            // client, the same fallback the selection path takes.
            Err(ClientActionResult::NoCurrentClient) if inner.clients.len() == 1 => {
                *inner.clients.keys().next().expect("one client present")
            }
            Err(result) => return Err(result),
        };
        let entry = inner.clients.get_mut(&id).expect("selected client present");
        let enable = !entry.flag_state.read_only;
        entry.flag_state.read_only = enable;
        entry.flag_state.ignore_size = enable;
        {
            // The attached client re-reads the registry view rather than the
            // values themselves; a control client, which keeps its own flag
            // state, applies them the way `refresh-client -f` does.
            let mut pending = entry.slot.flag_updates.borrow_mut();
            pending.push(if enable {
                "read-only,ignore-size".to_owned()
            } else {
                "!read-only,!ignore-size".to_owned()
            });
        }
        let _ = entry.slot.wakeup.wake();
        Ok(entry.name.clone())
    }

    /// tmux's `server_acl_user_allow_write` / `_deny_write` client sweep: every
    /// connected client whose peer uid matches follows its user's new access.
    ///
    /// The flag moves on the registry entry directly rather than through a
    /// `refresh-client -f` value, because such a value cannot clear read-only;
    /// the queued value is what tells the client to re-read the entry.
    pub(super) fn set_read_only_for_uid(&self, uid: u32, read_only: bool) {
        let mut inner = self.inner.borrow_mut();
        for entry in inner.clients.values_mut() {
            if entry.uid != Some(uid) || entry.flag_state.read_only == read_only {
                continue;
            }
            entry.flag_state.read_only = read_only;
            let value = if read_only { "read-only" } else { "!read-only" };
            entry.slot.flag_updates.borrow_mut().push(value.to_owned());
            let _ = entry.slot.wakeup.wake();
        }
    }

    /// End every connected client belonging to `uid`, each reporting `message`
    /// as why it stopped — the sweep `server-access -d` runs over `clients`
    /// before dropping the entry.
    pub(super) fn evict_clients_with_uid(&self, uid: u32, message: &str) {
        let inner = self.inner.borrow();
        for entry in inner.clients.values() {
            if entry.uid != Some(uid) {
                continue;
            }
            let mut action = entry.slot.action.borrow_mut();
            *action = Some(ClientAction::Evict {
                message: message.to_owned(),
            });
            let _ = entry.slot.wakeup.wake();
        }
    }

    pub(super) fn send_message(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        session_id: u32,
        message: ClientMessage,
    ) -> ClientMessageResult {
        let inner = self.inner.borrow();
        let explicit = target.map(|target| target.strip_suffix(':').unwrap_or(target));
        let selected = if let Some(target) = explicit {
            inner.clients.values().find(|entry| {
                entry.name == target
                    || entry
                        .name
                        .strip_prefix("/dev/")
                        .is_some_and(|tty| tty == target)
            })
        } else {
            invoking_tty
                .and_then(|tty| inner.clients.values().find(|entry| entry.name == tty))
                .or_else(|| {
                    inner
                        .clients
                        .values()
                        .find(|entry| entry.session_id == session_id)
                })
        };
        let Some(entry) = selected else {
            return if explicit.is_some() {
                ClientMessageResult::TargetNotFound
            } else {
                ClientMessageResult::NoClient
            };
        };
        if entry.control_mode && invoking_tty.is_some_and(|invoking_tty| entry.name == invoking_tty)
        {
            return ClientMessageResult::CurrentControl;
        }
        let mut messages = entry.slot.messages.borrow_mut();
        messages.push_back(message);
        let _ = entry.slot.wakeup.wake();
        ClientMessageResult::Queued
    }

    pub(super) fn detach_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        exec: Option<&str>,
        hangup: bool,
    ) -> ClientActionResult {
        let inner = self.inner.borrow();
        let entry = match Self::client_entry(&inner, target, invoking_tty) {
            Ok(entry) => entry,
            Err(result) => return result,
        };
        {
            let mut action = entry.slot.action.borrow_mut();
            *action = Some(ClientAction::Detach {
                exec: exec.map(str::to_owned),
                hangup,
            });
            let _ = entry.slot.wakeup.wake();
        }
        ClientActionResult::Queued
    }

    pub(super) fn detach_all_other_clients(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        exec: Option<&str>,
        hangup: bool,
    ) -> ClientActionResult {
        let inner = self.inner.borrow();
        let target_entry = match Self::client_entry(&inner, target, invoking_tty) {
            Ok(entry) => entry,
            Err(result) => return result,
        };
        let target_name = target_entry.name.clone();
        for entry in inner.clients.values() {
            if entry.name != target_name {
                let mut action = entry.slot.action.borrow_mut();
                *action = Some(ClientAction::Detach {
                    exec: exec.map(str::to_owned),
                    hangup,
                });
                let _ = entry.slot.wakeup.wake();
            }
        }
        ClientActionResult::Queued
    }

    pub(super) fn detach_session_clients(&self, session_id: u32, exec: Option<&str>, hangup: bool) {
        let inner = self.inner.borrow();
        for entry in inner.clients.values() {
            if entry.session_id == session_id {
                let mut action = entry.slot.action.borrow_mut();
                *action = Some(ClientAction::Detach {
                    exec: exec.map(str::to_owned),
                    hangup,
                });
                let _ = entry.slot.wakeup.wake();
            }
        }
    }

    pub(super) fn suspend_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> ClientActionResult {
        let inner = self.inner.borrow();
        let entry = match Self::client_entry(&inner, target, invoking_tty) {
            Ok(entry) => entry,
            Err(result) => return result,
        };
        {
            let mut action = entry.slot.action.borrow_mut();
            *action = Some(ClientAction::Suspend);
            let _ = entry.slot.wakeup.wake();
        }
        ClientActionResult::Queued
    }

    pub(super) fn refresh_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> ClientActionResult {
        let inner = self.inner.borrow();
        let entry = match Self::client_entry(&inner, target, invoking_tty) {
            Ok(entry) => entry,
            Err(result) => return result,
        };
        let reason = RenderInvalidation::STATUS | RenderInvalidation::LAYOUT;
        entry.slot.publish(reason);
        ClientActionResult::Queued
    }

    pub(super) fn client_read_only(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> Option<bool> {
        let inner = self.inner.borrow();
        Self::client_entry(&inner, target, invoking_tty)
            .ok()
            .map(|entry| entry.read_only())
    }

    pub(super) fn send_client_keys(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        keys: Vec<ClientKey>,
    ) -> ClientActionResult {
        let inner = self.inner.borrow();
        let entry = match Self::client_entry(&inner, target, invoking_tty) {
            Ok(entry) => entry,
            Err(result) => return result,
        };
        {
            let mut action = entry.slot.action.borrow_mut();
            match action.as_mut() {
                Some(ClientAction::Keys(queued)) => queued.extend(keys),
                _ => *action = Some(ClientAction::Keys(keys)),
            }
            let _ = entry.slot.wakeup.wake();
        }
        ClientActionResult::Queued
    }

    /// Write, or with `None` query, the terminal selection of one client.
    ///
    /// An untargeted request run from a client with no terminal of its own —
    /// an ordinary command client — falls back to the sole attached client,
    /// which is what tmux's `cmd_find_current_client` resolves to.
    pub(super) fn set_client_selection(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        data: Option<Vec<u8>>,
    ) -> ClientActionResult {
        let inner = self.inner.borrow();
        let entry = match Self::client_entry(&inner, target, invoking_tty) {
            Ok(entry) => entry,
            Err(ClientActionResult::NoCurrentClient) if inner.clients.len() == 1 => {
                inner.clients.values().next().expect("one client present")
            }
            Err(result) => return result,
        };
        {
            let mut action = entry.slot.action.borrow_mut();
            *action = Some(ClientAction::SetSelection(data));
            let _ = entry.slot.wakeup.wake();
        }
        ClientActionResult::Queued
    }

    /// Set a client's terminal focus, reporting whether it changed.
    /// Store a client's `refresh-client` pan. `None` clears it (`-c`).
    pub(super) fn set_client_pan(&self, client: &str, pan: Option<(u32, u16, u16)>) -> bool {
        let mut inner = self.inner.borrow_mut();
        let Some(entry) = inner
            .clients
            .values_mut()
            .find(|entry| entry.name == client)
        else {
            return false;
        };
        match pan {
            Some((window, ox, oy)) => {
                entry.pan_window = Some(window);
                entry.pan_ox = ox;
                entry.pan_oy = oy;
            }
            None => {
                entry.pan_window = None;
                entry.pan_ox = 0;
                entry.pan_oy = 0;
            }
        }
        true
    }

    /// Store the focus state a terminal reported, reporting whether the client
    /// is still known. tmux's `tty-keys` focus branch stores the flag for every
    /// report rather than only for a transition, so the caller can notify on
    /// each one.
    pub(super) fn set_client_focused(&self, client: &str, focused: bool) -> bool {
        let mut inner = self.inner.borrow_mut();
        let Some(entry) = inner
            .clients
            .values_mut()
            .find(|entry| entry.name == client)
        else {
            return false;
        };
        entry.focused = focused;
        true
    }

    /// The recency stamp a client currently holds, which is how a window names
    /// its latest client.
    pub(super) fn client_size_seq(&self, client: &str) -> Option<u64> {
        let inner = self.inner.borrow();
        inner
            .clients
            .values()
            .find(|entry| entry.name == client)
            .map(|entry| entry.size_seq)
    }

    /// Stamp a client as the most recent one, returning its new stamp.
    pub(super) fn promote_latest_client(&self, client: &str) -> Option<u64> {
        let stamped = {
            let mut inner = self.inner.borrow_mut();
            inner.next_size_seq = inner.next_size_seq.wrapping_add(1);
            let size_seq = inner.next_size_seq;
            inner
                .clients
                .values_mut()
                .find(|entry| entry.name == client)
                .map(|entry| {
                    entry.size_seq = size_seq;
                    size_seq
                })
        };
        if stamped.is_some() {
            self.bump_generation();
        }
        stamped
    }

    /// Record the theme a client's terminal reported, reporting whether it
    /// changed.
    pub(super) fn set_client_theme(&self, client: &str, theme: &str) -> bool {
        let mut inner = self.inner.borrow_mut();
        let Some(entry) = inner
            .clients
            .values_mut()
            .find(|entry| entry.name == client)
        else {
            return false;
        };
        if entry.theme == theme {
            return false;
        }
        entry.theme = theme.to_string();
        true
    }

    /// Stamp a client's activity time, tmux's `c->activity_time`.
    pub(super) fn touch_client_activity(&self, client: &str, at: i64) {
        let mut inner = self.inner.borrow_mut();
        if let Some(entry) = inner
            .clients
            .values_mut()
            .find(|entry| entry.name == client)
        {
            entry.activity_micros = at;
        }
    }

    /// The environment a named client attached with.
    pub(super) fn client_environment(&self, client: &str) -> Vec<String> {
        self.inner
            .borrow()
            .clients
            .values()
            .find(|entry| entry.name == client)
            .map(|entry| entry.environment.clone())
            .unwrap_or_default()
    }

    /// Record the key table a client moved into.
    pub(super) fn set_client_key_table(&self, client: &str, table: &str) {
        let mut inner = self.inner.borrow_mut();
        let Some(entry) = inner
            .clients
            .values_mut()
            .find(|entry| entry.name == client)
        else {
            return;
        };
        if entry.key_table != table {
            entry.key_table = table.to_string();
        }
    }

    pub(super) fn begin_clipboard_query(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        timeout: Duration,
    ) -> bool {
        let mut inner = self.inner.borrow_mut();
        let id = match Self::client_id_for(&inner, target, invoking_tty) {
            Ok(id) => id,
            Err(ClientActionResult::NoCurrentClient) if inner.clients.len() == 1 => {
                *inner.clients.keys().next().expect("one client present")
            }
            Err(_) => return false,
        };
        let now = Instant::now();
        let entry = inner.clients.get_mut(&id).expect("selected client present");
        if entry
            .clipboard_query_at
            .is_some_and(|at| now.saturating_duration_since(at) < timeout)
        {
            return false;
        }
        entry.clipboard_query_at = Some(now);
        true
    }

    pub(super) fn confirm_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        prompt: String,
        command: Vec<String>,
        confirm_key: u8,
        default_yes: bool,
        reply: Option<PromptReply>,
    ) -> ClientActionResult {
        let inner = self.inner.borrow();
        let entry = match Self::client_entry(&inner, target, invoking_tty) {
            Ok(entry) => entry,
            Err(result) => return result,
        };
        {
            let mut action = entry.slot.action.borrow_mut();
            *action = Some(ClientAction::Confirm {
                prompt,
                command,
                confirm_key,
                default_yes,
                reply,
            });
            let _ = entry.slot.wakeup.wake();
        }
        ClientActionResult::Queued
    }

    /// Move a client to another session, reporting the client's name so the
    /// caller can raise `client-session-changed` for it.
    pub(super) fn switch_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        session_id: u32,
    ) -> Result<String, ClientActionResult> {
        let mut inner = self.inner.borrow_mut();
        let explicit = target.map(|target| target.strip_suffix(':').unwrap_or(target));
        let id = if let Some(target) = explicit {
            inner.clients.iter().find_map(|(id, entry)| {
                (entry.name == target
                    || entry
                        .name
                        .strip_prefix("/dev/")
                        .is_some_and(|tty| tty == target))
                .then_some(*id)
            })
        } else {
            invoking_tty.and_then(|tty| {
                inner
                    .clients
                    .iter()
                    .find_map(|(id, entry)| (entry.name == tty).then_some(*id))
            })
        };
        let Some(id) = id else {
            return Err(if explicit.is_some() {
                ClientActionResult::TargetNotFound
            } else {
                ClientActionResult::NoCurrentClient
            });
        };
        let entry = inner
            .clients
            .get_mut(&id)
            .expect("selected client disappeared");
        let name = entry.name.clone();
        // tmux's `server_client_set_session`: a switch that lands on a
        // different session remembers the one it left; anything else clears it.
        entry.last_session_id = (entry.session_id != session_id).then_some(entry.session_id);
        entry.session_id = session_id;
        {
            let mut action = entry.slot.action.borrow_mut();
            *action = Some(ClientAction::Switch { session_id });
            let _ = entry.slot.wakeup.wake();
        }
        drop(inner);
        self.bump_generation();
        Ok(name)
    }

    /// tmux's `server_destroy_session` client fan-out: every client on
    /// `from_session_id` moves to `to`, except that a client carrying
    /// `no-detach-on-destroy` falls back to `no_detach_to` when `to` is `None`.
    /// A client with no destination is left alone and exits on its own.
    pub(super) fn reassign_session(
        &self,
        from_session_id: u32,
        to: Option<u32>,
        no_detach_to: Option<u32>,
    ) {
        let mut inner = self.inner.borrow_mut();
        for entry in inner
            .clients
            .values_mut()
            .filter(|entry| entry.session_id == from_session_id)
        {
            let Some(target) = to.or_else(|| {
                entry
                    .flag_state
                    .no_detach_on_destroy
                    .then_some(no_detach_to)
                    .flatten()
            }) else {
                continue;
            };
            entry.last_session_id = (entry.session_id != target).then_some(entry.session_id);
            entry.session_id = target;
            {
                let mut action = entry.slot.action.borrow_mut();
                *action = Some(ClientAction::Switch { session_id: target });
                let _ = entry.slot.wakeup.wake();
            }
        }
        drop(inner);
        self.bump_generation();
    }

    pub(super) fn overlay_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        request: OverlayRequest,
        reply: Option<PromptReply>,
    ) -> ClientActionResult {
        let inner = self.inner.borrow();
        let entry = match Self::client_entry(&inner, target, invoking_tty) {
            Ok(entry) => entry,
            Err(result) => return result,
        };
        {
            let mut action = entry.slot.action.borrow_mut();
            *action = Some(ClientAction::Overlay { request, reply });
            entry.slot.publish(RenderInvalidation::LAYOUT);
        }
        ClientActionResult::Queued
    }
}

impl ClientRenderAttachment {
    pub(crate) fn as_raw_fd(&self) -> RawFd {
        self.slot.wakeup.as_fd().as_raw_fd()
    }

    /// Publish the running total of bytes the attach loop has written to this
    /// client's terminal, backing `#{client_written}`.
    pub(crate) fn publish_written(&self, total: u64) {
        let mut inner = self.registry.inner.borrow_mut();
        if let Some(entry) = inner.clients.get_mut(&self.id) {
            entry.written = total;
        }
    }

    /// This client's `#()` job tree, so its status renderer and the commands it
    /// runs share one cache — as they do in tmux, where both reach `c->jobs`.
    pub(crate) fn format_jobs(&self) -> Rc<crate::server::status::FormatJobRegistry> {
        Rc::clone(&self.format_jobs)
    }

    /// Clear readiness and take every reason published so far.
    ///
    /// Clearing before the atomic swap cannot lose a concurrent publication:
    /// a publication after the clear leaves the fd readable (at worst causing
    /// one harmless coalesced follow-up poll).
    pub(crate) fn take(&self) -> RenderInvalidation {
        let _ = self.slot.wakeup.clear();
        RenderInvalidation(self.slot.pending.replace(0))
    }

    pub(crate) fn take_action(&self) -> Option<ClientAction> {
        self.slot.action.borrow_mut().take()
    }

    /// Take the pending action only when it is a session switch.
    ///
    /// A `switch-client` has to take effect within the command that asked for
    /// it, so the command path claims that one action as it finishes rather
    /// than leaving it for the next event-loop pass. Every other action stays
    /// in the slot for the loop to handle.
    pub(crate) fn take_switch(&self) -> Option<u32> {
        let mut action = self.slot.action.borrow_mut();
        let Some(ClientAction::Switch { session_id }) = *action else {
            return None;
        };
        *action = None;
        Some(session_id)
    }

    pub(crate) fn take_messages(&self) -> Vec<ClientMessage> {
        self.slot.messages.borrow_mut().drain(..).collect()
    }

    /// Publish the client terminal's new cell count.
    ///
    /// Which client is *latest* is not settled here: tmux promotes the resizing
    /// client from `MSG_RESIZE` through `server_client_update_latest`, which
    /// also raises `client-active` and skips control clients entirely. The
    /// attached client's resize path calls
    /// [`ServerState::update_latest_client`] for that.
    pub(crate) fn update_size(&self, cols: u16, rows: u16) {
        {
            let mut inner = self.registry.inner.borrow_mut();
            if let Some(entry) = inner.clients.get_mut(&self.id) {
                entry.cols = cols;
                entry.rows = rows;
                entry.size_changed = true;
            }
        }
        self.registry.bump_generation();
    }

    /// Publish the client terminal's cell size in pixels, which the window's
    /// own cell size is aggregated from. Separate from [`Self::update_size`]
    /// because a font change moves it without moving the cell count.
    pub(crate) fn update_cell_pixels(&self, xpixel: u16, ypixel: u16) {
        {
            let mut inner = self.registry.inner.borrow_mut();
            if let Some(entry) = inner.clients.get_mut(&self.id) {
                entry.xpixel = xpixel;
                entry.ypixel = ypixel;
            }
        }
        self.registry.bump_generation();
    }

    pub(crate) fn mark_size_changed(&self) {
        {
            let mut inner = self.registry.inner.borrow_mut();
            if let Some(entry) = inner.clients.get_mut(&self.id) {
                entry.size_changed = true;
            }
        }
    }

    /// Record that the client is stopped on `suspend-client`, or has woken
    /// back up. tmux's `sort_get_clients` skips a suspended client, so the
    /// listings built on it lose the row until it resumes.
    pub(crate) fn set_suspended(&self, suspended: bool) {
        {
            let mut inner = self.registry.inner.borrow_mut();
            if let Some(entry) = inner.clients.get_mut(&self.id) {
                entry.suspended = suspended;
            }
        }
        self.registry.bump_generation();
    }

    /// Publish the flag set a control client keeps of its own, so the registry
    /// view other clients read (`#{client_flags}`, the destroy policy) follows
    /// its `refresh-client -f`.
    pub(crate) fn update_control_flags(&self, flag_state: &ClientFlagState) {
        {
            let mut inner = self.registry.inner.borrow_mut();
            if let Some(entry) = inner.clients.get_mut(&self.id) {
                entry.flag_state = flag_state.clone();
            }
        }
    }

    /// This client's registry name, as `#{client_name}` reports it.
    pub(crate) fn client_name(&self) -> String {
        self.registry
            .inner
            .borrow()
            .clients
            .get(&self.id)
            .map(|entry| entry.name.clone())
            .unwrap_or_default()
    }

    /// Bytes an application asked to have written to this client's terminal.
    pub(crate) fn take_client_output(&self) -> Vec<Bytes> {
        self.slot.client_output.borrow_mut().drain(..).collect()
    }

    /// `refresh-client -f` values another client aimed at this one.
    pub(crate) fn take_flag_updates(&self) -> Vec<String> {
        std::mem::take(&mut *self.slot.flag_updates.borrow_mut())
    }

    /// The registry's current flag view for this client: the rendered
    /// `#{client_flags}` string and the read-only bit. What a client that
    /// doesn't own a `ClientFlagState` of its own reads back after
    /// [`Self::take_flag_updates`].
    pub(crate) fn client_flags_view(&self) -> (String, bool) {
        self.registry
            .inner
            .borrow()
            .clients
            .get(&self.id)
            .map(|entry| (entry.display_flags(), entry.read_only()))
            .unwrap_or_default()
    }

    /// Record the name the client's terminal gave for itself in its XTVERSION
    /// reply — `#{client_termtype}`.
    pub(crate) fn update_term_type(&self, term_type: &str) {
        let mut inner = self.registry.inner.borrow_mut();
        if let Some(entry) = inner.clients.get_mut(&self.id) {
            entry.term_type = term_type.to_owned();
        }
    }

    pub(crate) fn update_terminal(&self, terminal: &ResolvedTerm) {
        {
            let mut inner = self.registry.inner.borrow_mut();
            if let Some(entry) = inner.clients.get_mut(&self.id) {
                entry.terminal = Some(terminal.clone());
            }
        }
    }

    /// Record the peer identity of the client this attachment renders for.
    /// The login name is resolved by the caller, which already pays the NSS
    /// lookup once for its own format context.
    pub(crate) fn set_peer_identity(&self, uid: Option<u32>, user: String) {
        let mut inner = self.registry.inner.borrow_mut();
        if let Some(entry) = inner.clients.get_mut(&self.id) {
            entry.uid = uid;
            entry.user = user;
        }
    }

    /// Record the environment this client attached with, which outlives the
    /// attach command: `switch-client` copies out of it later, from whatever
    /// other client happens to run it.
    pub(crate) fn set_environment(&self, environment: Vec<String>) {
        let mut inner = self.registry.inner.borrow_mut();
        if let Some(entry) = inner.clients.get_mut(&self.id) {
            entry.environment = environment;
        }
    }

    pub(crate) fn update_session(&self, session_id: u32) {
        {
            let mut inner = self.registry.inner.borrow_mut();
            if let Some(entry) = inner.clients.get_mut(&self.id) {
                entry.session_id = session_id;
            }
        }
        self.registry.bump_generation();
    }
}

impl Drop for ClientRenderAttachment {
    fn drop(&mut self) {
        {
            let mut inner = self.registry.inner.borrow_mut();
            inner.clients.remove(&self.id);
            let peers = inner
                .clients
                .values()
                .map(|entry| Rc::clone(&entry.slot))
                .collect::<Vec<_>>();
            drop(inner);
            self.registry.bump_generation();
            for peer in peers {
                peer.publish(RenderInvalidation::STATUS);
            }
        }
    }
}

impl ClientPromptRegistry {
    pub(super) fn new() -> Self {
        Self {
            inner: RefCell::new(ClientPromptRegistryState::default()),
            activity: Cell::new(0),
        }
    }

    pub(crate) fn attach(
        self: &Rc<Self>,
        tty_name: String,
        client_pid: Option<i32>,
        _session_id: u32,
    ) -> io::Result<ClientPromptAttachment> {
        let wakeup = CurrentPlatform::new_output_wakeup()?;
        wakeup.clear()?;
        let slot = Rc::new(ClientPromptSlot {
            inner: RefCell::new(ClientPromptSlotState::default()),
            wakeup,
        });
        let activity = Rc::new(Cell::new(self.next_activity()));
        let mut inner = self.inner.borrow_mut();
        let id = inner.next_id;
        inner.next_id = inner.next_id.wrapping_add(1);
        let name = if tty_name.is_empty() {
            format!("client-{}", client_pid.unwrap_or_default())
        } else {
            tty_name.clone()
        };
        inner.clients.insert(
            id,
            ClientPromptEntry {
                name,
                tty_name,
                slot: Rc::clone(&slot),
                activity: Rc::clone(&activity),
            },
        );
        Ok(ClientPromptAttachment {
            registry: Rc::clone(self),
            id,
            slot,
            activity,
        })
    }

    fn next_activity(&self) -> u64 {
        let issued = self.activity.get();
        self.activity.set(issued.wrapping_add(1));
        issued
    }

    pub(crate) fn request_command(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        args: Vec<String>,
        wait: bool,
    ) -> CommandPromptRequestResult {
        let completed = {
            let inner = self.inner.borrow();
            let explicit = target.map(|target| target.strip_suffix(':').unwrap_or(target));
            let selected = if let Some(target) = explicit {
                inner.clients.values().find(|entry| {
                    entry.name == target
                        || entry.tty_name == target
                        || entry
                            .tty_name
                            .strip_prefix("/dev/")
                            .is_some_and(|tty| tty == target)
                })
            } else {
                invoking_tty
                    .and_then(|tty| inner.clients.values().find(|entry| entry.tty_name == tty))
                    .or_else(|| {
                        inner
                            .clients
                            .values()
                            .max_by_key(|entry| entry.activity.get())
                    })
            };
            let Some(entry) = selected else {
                return if explicit.is_some() {
                    CommandPromptRequestResult::TargetNotFound
                } else {
                    CommandPromptRequestResult::NoCurrentClient
                };
            };
            let slot = Rc::clone(&entry.slot);
            let Ok((reply, completed)) = PromptReply::new() else {
                return CommandPromptRequestResult::NoCurrentClient;
            };
            let mut state = slot.inner.borrow_mut();
            if state.active || state.queued_command.is_some() {
                return CommandPromptRequestResult::Busy;
            }
            state.queued_command = Some(QueuedCommandPrompt { args, reply });
            if slot.wakeup.wake().is_err() {
                state.queued_command = None;
                return CommandPromptRequestResult::NoCurrentClient;
            }
            completed
        };
        if !wait {
            return CommandPromptRequestResult::Queued;
        }
        CommandPromptRequestResult::Waiting(completed)
    }
}

impl ClientPromptAttachment {
    pub(crate) fn as_raw_fd(&self) -> RawFd {
        self.slot.wakeup.as_fd().as_raw_fd()
    }

    pub(crate) fn take_command_prompt(&self) -> Option<ActiveCommandPrompt> {
        let _ = self.slot.wakeup.clear();
        let mut state = self.slot.inner.borrow_mut();
        let queued = state.queued_command.take()?;
        state.active = true;
        Some(ActiveCommandPrompt {
            slot: Rc::clone(&self.slot),
            args: queued.args,
            reply: Some(queued.reply),
        })
    }

    pub(crate) fn note_activity(&self) {
        self.activity.set(self.registry.next_activity());
    }
}

impl Drop for ClientPromptAttachment {
    fn drop(&mut self) {
        {
            let mut inner = self.registry.inner.borrow_mut();
            inner.clients.remove(&self.id);
        }
        {
            let mut state = self.slot.inner.borrow_mut();
            if let Some(queued) = state.queued_command.take() {
                queued.reply.send(None);
            }
        }
    }
}

impl ActiveCommandPrompt {
    pub(crate) fn args(&self) -> &[String] {
        &self.args
    }

    pub(crate) fn complete(mut self, result: PromptCompletion) {
        {
            let mut state = self.slot.inner.borrow_mut();
            state.active = false;
        }
        if let Some(reply) = self.reply.take() {
            reply.send(Some(result));
        }
    }

    pub(crate) fn cancel(mut self) {
        {
            let mut state = self.slot.inner.borrow_mut();
            state.active = false;
        }
        if let Some(reply) = self.reply.take() {
            reply.send(None);
        }
    }
}

impl Drop for ActiveCommandPrompt {
    fn drop(&mut self) {
        {
            let mut state = self.slot.inner.borrow_mut();
            state.active = false;
        }
    }
}

impl ServerState {
    pub(crate) fn client_prompt_registry(&self) -> Rc<ClientPromptRegistry> {
        Rc::clone(&self.client_prompts)
    }

    pub(crate) fn client_render_registry(&self) -> Rc<ClientRenderRegistry> {
        Rc::clone(&self.client_renders)
    }

    pub(crate) fn client_snapshots(&self) -> Vec<ClientSnapshot> {
        self.client_renders.snapshots()
    }

    /// Each attached client's resolved terminal description, for
    /// `show-messages -T`: `(name, TERM, resolved)`. The only path that pays
    /// for a `ResolvedTerm` clone.
    pub(crate) fn client_terminals(&self) -> Vec<(String, String, ResolvedTerm)> {
        self.client_renders.with_entries(|entries| {
            entries
                .filter_map(|entry| {
                    let terminal = entry.terminal.clone()?;
                    Some((entry.name.clone(), entry.term.clone(), terminal))
                })
                .collect()
        })
    }

    /// Whether a client other than `client_name` holds the session's size:
    /// tmux's `ignore_client_size` — an `ignore-size` client is ignored only
    /// while some counted client is unflagged.
    pub(crate) fn other_client_constrains_size(&self, session_id: u32, client_name: &str) -> bool {
        self.client_renders.with_entries(|mut entries| {
            entries.any(|entry| {
                entry.session_id == session_id
                    && entry.name != client_name
                    && !entry.ignore_size()
                    && entry.counts_for_sizing()
            })
        })
    }

    pub(crate) fn send_client_message(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        session_id: u32,
        message: ClientMessage,
    ) -> ClientMessageResult {
        self.client_renders
            .send_message(target, invoking_tty, session_id, message)
    }

    pub(crate) fn set_client_selection(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        data: Option<Vec<u8>>,
    ) -> ClientActionResult {
        self.client_renders
            .set_client_selection(target, invoking_tty, data)
    }

    /// Record a terminal focus report. The client's `focused` flag follows it
    /// whatever `focus-events` says — the option only decides whether tmux asks
    /// the terminal for reports at all — and the active pane's focus moves with
    /// the client's.
    ///
    /// The hook is raised per *report*, not per transition: tmux notifies from
    /// the key decoder without comparing against the flag it already holds, so
    /// a terminal that repeats a focus report raises the hook again.
    pub(crate) fn set_client_focus(
        &mut self,
        client: &str,
        session_id: u32,
        focused: bool,
        target: &str,
    ) {
        if !self.client_renders.set_client_focused(client, focused) {
            return;
        }
        self.notify_client(
            if focused {
                SessionHook::ClientFocusIn
            } else {
                SessionHook::ClientFocusOut
            },
            client,
            Some(session_id),
        );
        let _ = target;
        if let Some(window_id) = self.current_window_of_session(session_id) {
            self.update_window_focus(window_id);
        }
    }

    /// Record the theme a client's terminal reported, firing the matching hook
    /// when it changes.
    pub(crate) fn set_client_theme(&mut self, client: &str, session_id: u32, theme: &str) {
        if !self.client_renders.set_client_theme(client, theme) {
            return;
        }
        self.notify_client(
            if theme == "dark" {
                SessionHook::ClientDarkTheme
            } else {
                SessionHook::ClientLightTheme
            },
            client,
            Some(session_id),
        );
    }

    /// Whether a clipboard query may be sent to this client now. tmux keeps one
    /// outstanding request per terminal for `CLIPBOARD_QUERY_TIMEOUT`, so a
    /// repeat inside the window is dropped instead of queued.
    pub(crate) fn begin_clipboard_query(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> bool {
        const CLIPBOARD_QUERY_TIMEOUT: Duration = Duration::from_secs(5);
        self.client_renders
            .begin_clipboard_query(target, invoking_tty, CLIPBOARD_QUERY_TIMEOUT)
    }

    pub(crate) fn overlay_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        request: OverlayRequest,
        reply: Option<PromptReply>,
    ) -> ClientActionResult {
        self.client_renders
            .overlay_client(target, invoking_tty, request, reply)
    }

    pub(crate) fn confirm_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        prompt: String,
        command: Vec<String>,
        confirm_key: u8,
        default_yes: bool,
        reply: Option<PromptReply>,
    ) -> ClientActionResult {
        self.client_renders.confirm_client(
            target,
            invoking_tty,
            prompt,
            command,
            confirm_key,
            default_yes,
            reply,
        )
    }

    pub(crate) fn add_message(&mut self, text: String) {
        let limit = self
            .server_options()
            .get("message-limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(1000);
        self.message_log.push(MessageLogEntry {
            time: now_epoch(),
            text,
        });
        if self.message_log.len() > limit {
            self.message_log.drain(..self.message_log.len() - limit);
        }
    }

    pub(crate) fn messages(&self) -> &[MessageLogEntry] {
        &self.message_log
    }

    pub(crate) fn lock_all_clients(&self) {
        self.client_renders.lock_all(&self.lock_commands());
    }

    pub(crate) fn lock_session_clients(&self, target: &str) -> io::Result<()> {
        let session = self.resolve_session(target).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("can't find session: {target}"),
            )
        })?;
        let command = session
            .options(&self.global_options)
            .get("lock-command")
            .unwrap_or("lock -np");
        self.client_renders.lock_session(session.id, command);
        Ok(())
    }

    pub(crate) fn lock_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> ClientActionResult {
        self.client_renders
            .lock_client(target, invoking_tty, &self.lock_commands())
    }

    pub(crate) fn detach_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        exec: Option<&str>,
        hangup: bool,
    ) -> ClientActionResult {
        self.client_renders
            .detach_client(target, invoking_tty, exec, hangup)
    }

    pub(crate) fn detach_all_other_clients(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        exec: Option<&str>,
        hangup: bool,
    ) -> ClientActionResult {
        self.client_renders
            .detach_all_other_clients(target, invoking_tty, exec, hangup)
    }

    pub(crate) fn detach_session_clients(&self, session_id: u32, exec: Option<&str>, hangup: bool) {
        self.client_renders
            .detach_session_clients(session_id, exec, hangup);
    }

    pub(crate) fn suspend_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> ClientActionResult {
        self.client_renders.suspend_client(target, invoking_tty)
    }

    pub(crate) fn refresh_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> ClientActionResult {
        self.client_renders.refresh_client(target, invoking_tty)
    }

    /// Record `refresh-client -f` values on the target client and hand them to
    /// it so its own flag state follows.
    pub(crate) fn refresh_client_flags(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        values: &[String],
    ) -> ClientActionResult {
        self.client_renders
            .refresh_client_flags(target, invoking_tty, values)
    }

    pub(crate) fn client_read_only(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> Option<bool> {
        self.client_renders.client_read_only(target, invoking_tty)
    }

    pub(crate) fn send_client_keys(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        keys: Vec<ClientKey>,
    ) -> ClientActionResult {
        self.client_renders
            .send_client_keys(target, invoking_tty, keys)
    }

    pub(crate) fn switch_client(
        &mut self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
        session_id: u32,
    ) -> ClientActionResult {
        match self
            .client_renders
            .switch_client(target, invoking_tty, session_id)
        {
            Ok(name) => {
                self.announce_client_session(&name, session_id);
                ClientActionResult::Queued
            }
            Err(result) => result,
        }
    }

    /// tmux's `server_client_update_latest`: a key from a client — or its
    /// terminal resizing — makes it the latest client of the window its session
    /// is showing, and `client-active` announces the move.
    ///
    /// A window sized `latest` follows its latest client, so promoting one is
    /// also a resize trigger.
    pub(crate) fn update_latest_client(&mut self, client: &str, session_id: u32) {
        let Some(window_id) = self.current_window_of_session(session_id) else {
            return;
        };
        let Some(seq) = self.client_renders.client_size_seq(client) else {
            return;
        };
        if self
            .windows
            .get(&window_id)
            .is_some_and(|window| window.latest_client == Some(seq))
        {
            return;
        }
        let Some(seq) = self.client_renders.promote_latest_client(client) else {
            return;
        };
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.latest_client = Some(seq);
        }
        let follows_latest = self.windows.get(&window_id).is_some_and(|window| {
            window.options(&self.global_options).get("window-size") == Some("latest")
        });
        if follows_latest {
            let _ = self.recalculate_sizes();
        }
        self.notify_client(SessionHook::ClientActive, client, Some(session_id));
    }

    /// Raise `client-session-changed` for a client that has just been given a
    /// session.
    ///
    /// tmux notifies from `server_client_set_session` for every switch, so a
    /// client sent to the session it is already on is announced again. The
    /// client-layer snapshot is moved along with it, because that diff is what
    /// reports the session moves nothing calls this for.
    fn announce_client_session(&mut self, client: &str, session_id: u32) {
        self.notify_client(SessionHook::ClientSessionChanged, client, Some(session_id));
        self.take_session_for_client(session_id);
        if let Some(known) = self.known_clients.get_mut(client) {
            known.0 = session_id;
        }
    }

    /// Raise `client-resized` for a client that has just reported its terminal
    /// size.
    ///
    /// tmux notifies from the `MSG_RESIZE` arm of `server_client_dispatch`,
    /// which is the message itself rather than a change in what it carries: a
    /// client that reports the size it already had is announced again. The
    /// client-layer snapshot is moved along with it, so the size diff that
    /// reports the resizes no client announces does not report this one twice.
    pub(crate) fn announce_client_resize(
        &mut self,
        client: &str,
        session_id: u32,
        cols: u16,
        rows: u16,
    ) {
        self.notify_client(SessionHook::ClientResized, client, Some(session_id));
        if let Some(known) = self.known_clients.get_mut(client) {
            known.1 = cols;
            known.2 = rows;
        }
    }

    /// Toggle `switch-client -r` on the target client, reporting its name.
    pub(crate) fn toggle_client_read_only(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> Result<String, ClientActionResult> {
        self.client_renders
            .toggle_client_read_only(target, invoking_tty)
    }

    /// The environment a named client attached with — tmux's `c->environ`.
    pub(crate) fn client_environment(&self, client: &str) -> Vec<String> {
        self.client_renders.client_environment(client)
    }

    /// The client a `-c` target names, resolved the way tmux's
    /// `cmd_find_client` does it: the named client, else the one on the
    /// invoking terminal, else — for a command client that owns no terminal —
    /// the sole attached client.
    pub(crate) fn resolve_target_client(
        &self,
        target: Option<&str>,
        invoking_tty: Option<&str>,
    ) -> Result<ClientSnapshot, ClientActionResult> {
        let clients = self.client_snapshots();
        let matches = |entry: &ClientSnapshot, name: &str| {
            entry.name == name
                || entry
                    .name
                    .strip_prefix("/dev/")
                    .is_some_and(|tty| tty == name)
        };
        if let Some(target) = target.map(|target| target.strip_suffix(':').unwrap_or(target)) {
            return clients
                .into_iter()
                .find(|entry| matches(entry, target))
                .ok_or(ClientActionResult::TargetNotFound);
        }
        if let Some(entry) = invoking_tty
            .and_then(|tty| clients.iter().find(|entry| matches(entry, tty)))
            .cloned()
        {
            return Ok(entry);
        }
        match clients.len() {
            1 => Ok(clients.into_iter().next().expect("one client present")),
            _ => Err(ClientActionResult::NoCurrentClient),
        }
    }

    /// tmux stamps `c->activity_time` alongside the session's whenever a key
    /// arrives; it is what orders `cmd_find_best_client`.
    pub(crate) fn touch_client_activity(&mut self, client: &str, session_id: u32) {
        self.touch_session_activity(session_id, false);
        self.client_renders
            .touch_client_activity(client, now_micros());
    }

    /// tmux's `session_lock_timer`: lock every client of a session whose
    /// clients have gone `lock-after-time` seconds without touching a key.
    ///
    /// tmux arms the timer from `session_update_activity` and lets libevent
    /// fire it once; hmux polls instead, so `locked_at_activity_micros` is what
    /// keeps an idle session from being locked again on every server loop.
    /// Returns how long the loop may sleep before the next session is due.
    pub(crate) fn refresh_lock_timers(&mut self) -> Option<Duration> {
        let attached = self.attached_session_ids();
        if attached.is_empty() {
            return None;
        }
        let now = now_micros();
        let mut due = Vec::new();
        let mut next = None;
        for session in self
            .sessions
            .iter()
            .filter(|session| attached.contains(&session.id))
        {
            let seconds = session
                .options(&self.global_options)
                .get("lock-after-time")
                .and_then(|value| value.parse::<i64>().ok())
                .unwrap_or(0);
            if seconds <= 0 || session.locked_at_activity_micros == Some(session.activity_micros) {
                continue;
            }
            let deadline = session.activity_micros + seconds * 1_000_000;
            if deadline <= now {
                due.push(session.id);
            } else {
                let remaining = Duration::from_micros((deadline - now) as u64);
                next = Some(next.map_or(remaining, |next: Duration| next.min(remaining)));
            }
        }
        let commands = self.lock_commands();
        for session_id in due {
            if let Some(session) = self
                .sessions
                .iter_mut()
                .find(|session| session.id == session_id)
            {
                session.locked_at_activity_micros = Some(session.activity_micros);
            }
            if let Some(command) = commands.get(&session_id) {
                self.client_renders.lock_session(session_id, command);
            }
        }
        next
    }

    pub(crate) fn control_snapshot(&self, session_name: &str) -> Option<ControlStateSnapshot> {
        let session_pos = self.session_index(session_name)?;
        let session = &self.sessions[session_pos];
        let mut windows = BTreeMap::new();
        for link in session.windows.iter() {
            let window = self.windows.get(&link.id)?;
            windows.insert(
                window.id,
                ControlWindowSnapshot {
                    id: window.id,
                    index: link.index,
                    panes: window
                        .panes
                        .iter()
                        .map(|pane| ControlPaneSnapshot {
                            id: pane.id,
                            runtime_id: pane.pane.runtime_id(),
                            observation: pane.pane.observation_state(),
                        })
                        .collect(),
                },
            );
        }
        Some(ControlStateSnapshot {
            session_id: session.id,
            session_name: session.name.clone(),
            windows,
        })
    }
}

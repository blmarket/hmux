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
//! The native compatibility path uses pane reader threads and polls attach
//! sources directly. The event-loop adapter drives pane PTYs and delivers attach
//! sources as readiness turns from its central reactor. Grid changes are
//! detected by diffing the last rendered VT. The compositor itself remains
//! single-threaded and both tty fds are non-blocking.

mod actions;
mod copy_mode;
mod input;
mod keys;
mod overlay;
mod prompt;
mod render;
mod session;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::integration::status::StatusHub;
use crate::tmux::message::{Frame, Message, PROTOCOL_VERSION};
use crate::tmux::traits::{FrameReader, FrameWriter};

use super::cmd_send_keys::base64_encode;
use super::input_keys::PaneKey;
use super::command;
use super::format;
use super::key::{key_from_byte, parse_key_name, KeyBase, KeyCode, SpecialKey};
use super::latmon::LatMon;
use super::mouse::{
    self, MouseEvent, MouseEventKind, MouseInputState, MousePosition, MouseProtocol,
};
#[cfg(test)]
use super::mouse::MouseButton;
use super::pane::{OutputSubscription, Pane, PaneInputStats, PaneIo, PaneIoMode};
use super::state::{
    ClientAction, ClientKey, MenuRequest, ModeKind, ModeView, ModeViewKeyResult, OverlayRequest,
    PopupRequest, ServerState,
};
use super::status;
use super::term::{self, ResolvedTerm, TerminalCapabilities, TerminalIdentity};
pub(super) use actions::dispatch_control_client_keys;
#[cfg(test)]
use actions::dispatch_prefix_key;
use actions::{
    dispatch_key_binding, ActiveConfirm, ConfirmAction, ConfirmResolution, PrefixOutcome,
};
use copy_mode::CopyModeView;
use keys::{ClientKeyState, KeyResolution, ServerKeyTables};
pub(crate) use overlay::ActiveOverlay;
use prompt::{
    clip_prompt_display, render_prompt_completion, take_deferred_attach_command, CommandPrompt,
    CommandPromptInput,
};
#[cfg(test)]
use prompt::{prompt_input_width, render_prompt_input};

#[cfg(test)]
const PREFIX: u8 = 0x02;
const TTY_OUTPUT_LIMIT: usize = 4 * 1024 * 1024;

/// Internal capability needed by the event-driven attach loop. This stays
/// separate from the public `FrameReader` compatibility contract.
pub(crate) trait AttachFrameReader: FrameReader + AsRawFd {
    fn try_recv(&mut self) -> io::Result<Frame>;
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
        if self.deadline.is_none_or(|deadline| now < deadline) {
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

/// Client-local compositor data that must survive from one readiness turn to
/// the next. Keeping it explicit lets the event-loop adapter eventually own
/// this state without an executor task or a second attach implementation.
struct AttachTargetState {
    session_id: u32,
    stable_target: String,
    context: command::ClientContext,
}

struct AttachRenderState {
    last_render: Vec<u8>,
    seen_large_scroll: BTreeMap<u32, u64>,
    output_cursor_visible: Option<bool>,
    last_title: Option<String>,
    force_clear: bool,
}

struct StatusMessage {
    text: String,
    deadline: Instant,
}

struct AttachUiState {
    confirm: Option<ActiveConfirm>,
    command_prompt: Option<CommandPrompt>,
    active_overlay: Option<ActiveOverlay>,
    status_message: Option<StatusMessage>,
}

#[derive(Default)]
enum KeyPromptState {
    #[default]
    Idle,
    Pending {
        bytes: Vec<u8>,
        deadline: Option<Instant>,
    },
}

struct PendingTerminalReply {
    bytes: Vec<u8>,
    deadline: Instant,
}

struct AttachInputState {
    keys: ClientKeyState,
    /// The key table last published to the server, so `#{client_key_table}`
    /// and the status line only churn when it actually changes.
    published_key_table: String,
    mouse: MouseInputState,
    key_prompt: KeyPromptState,
    terminal_reply: Option<PendingTerminalReply>,
    injected: VecDeque<ClientKey>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ClientIoState {
    Active,
    Locked,
    Suspended,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AttachFinishReason {
    ConnectionClosed,
    Detached,
    SessionEnded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AttachTransition {
    SwitchSession(u32),
    Finish(AttachFinishReason),
}

impl AttachTargetState {
    fn switch_session(&mut self, session_id: u32) {
        self.session_id = session_id;
        self.stable_target = format!("${session_id}");
        self.context.current_session_id = Some(session_id);
    }
}

impl KeyPromptState {
    fn deadline(&self) -> Option<Instant> {
        match self {
            Self::Idle => None,
            Self::Pending { deadline, .. } => *deadline,
        }
    }

    fn bytes(&self) -> &[u8] {
        match self {
            Self::Idle => &[],
            Self::Pending { bytes, .. } => bytes,
        }
    }

    fn extend(&mut self, input: &[u8]) {
        match self {
            Self::Idle => {
                *self = Self::Pending {
                    bytes: input.to_vec(),
                    deadline: None,
                };
            }
            Self::Pending { bytes, .. } => bytes.extend_from_slice(input),
        }
    }

    fn clear(&mut self) {
        *self = Self::Idle;
    }

    fn deadline_or_insert(&mut self, deadline: Instant) -> Instant {
        match self {
            Self::Idle => unreachable!("idle key prompt has no deadline"),
            Self::Pending {
                deadline: current, ..
            } => *current.get_or_insert(deadline),
        }
    }

    fn set_deadline_if_none(&mut self, deadline: Instant) {
        let Self::Pending {
            deadline: current, ..
        } = self
        else {
            unreachable!("idle key prompt has no deadline");
        };
        current.get_or_insert(deadline);
    }
}

struct AttachCompositorState {
    target: AttachTargetState,
    render: AttachRenderState,
    ui: AttachUiState,
    input: AttachInputState,
    io_state: ClientIoState,
    transition: Option<AttachTransition>,
}

/// All native attach state that must remain alive while readiness is owned by
/// either the compatibility poller or the server event loop.
pub(crate) struct AttachSession {
    tty: AttachTty,
    attachments: AttachAttachments,
    viewport: AttachViewport,
    status: AttachStatus,
    pane_io: AttachPaneIo,
    commands: AttachCommands,
    compositor: AttachCompositorState,
    finish: AttachFinishState,
}

struct AttachTty {
    // Restore the tty before the owned descriptors below are closed if a turn
    // exits early. The normal finish path disarms this guard explicitly.
    termios_guard: TermiosGuard,
    input_fd: OwnedFd,
    render_fd: OwnedFd,
    terminal: ResolvedTerm,
    output: TtyOutput,
}

struct AttachAttachments {
    prompt_attachment: super::state::ClientPromptAttachment,
    render_attachment: super::state::ClientRenderAttachment,
    agent_status_subscription: crate::integration::status::StatusSubscription,
    output_subscription: OutputSubscription,
    subscribed_window: ActiveWindowOutputKey,
    output_generation: u64,
}

struct AttachViewport {
    cols: u16,
    rows: u16,
    pane_rows: u16,
    status_height: u16,
}

struct AttachStatus {
    status_timer: StatusTimer,
    status_cache: status::RenderCache,
}

struct AttachPaneIo {
    mode: PaneIoMode,
    latmon: LatMon,
}

struct AttachCommands {
    /// Commands a key binding deferred, in the order their keys arrived.
    ///
    /// A queue rather than one slot: a burst of mouse reports in a single read
    /// defers one command per report, and each has to run — replacing the slot
    /// would silently drop every binding but the last.
    pending: VecDeque<AttachCommandRequest>,
    deferred_prompts: VecDeque<AttachCommandRequest>,
}

pub(crate) struct AttachCommandRequest {
    pub(crate) source: command::DeferredCommand,
    pub(crate) context: command::ClientContext,
    pub(crate) continuation: AttachCommandContinuation,
}

pub(crate) enum AttachCommandContinuation {
    PrefixBinding {
        target: String,
        cols: u16,
        pane_rows: u16,
    },
    Overlay {
        overlay: Box<ActiveOverlay>,
        inserted: bool,
    },
    Confirm {
        reply: Option<std::sync::mpsc::Sender<super::state::PromptCompletion>>,
        inserted: bool,
    },
    Prompt {
        prompt: Box<CommandPrompt>,
    },
    Message {
        target: String,
        escape_hashes: bool,
        explicit_duration: Option<u64>,
    },
    Ignore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AttachFinishState {
    Running,
    DrainingTty { reason: AttachFinishReason },
    WaitingForAck { deadline: Instant },
    Done,
}

struct AttachRenderTriggers {
    output_ready: bool,
    status_timer_ready: bool,
    agent_status_changed: bool,
    overlay_tick: bool,
    message_expired: bool,
    render_invalidation: super::state::RenderInvalidation,
}

enum AttachNotificationOutcome {
    Continue(AttachRenderTriggers),
    Return(AttachDrive),
}

pub(crate) enum AttachPrepared {
    Ready(AttachWaitReady),
    Wait {
        sources: AttachWaitSources,
        timeout: i32,
    },
    Finished,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AttachDrive {
    Continue,
    Finished,
}

pub(crate) enum AttachStartFailure {
    Client(String),
    Io(io::Error),
}

impl AttachStartFailure {
    pub(crate) fn into_message(self) -> String {
        match self {
            Self::Client(message) => message,
            Self::Io(error) => format!("{error}\n"),
        }
    }
}

impl From<io::Error> for AttachStartFailure {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

struct TtyOutput {
    bytes: Vec<u8>,
    written: usize,
}

impl TtyOutput {
    fn new() -> TtyOutput {
        TtyOutput {
            bytes: Vec::new(),
            written: 0,
        }
    }

    fn has_pending(&self) -> bool {
        self.written < self.bytes.len()
    }

    fn queue(&mut self, fd: RawFd, bytes: &[u8]) -> io::Result<()> {
        if bytes.len() > TTY_OUTPUT_LIMIT.saturating_sub(self.bytes.len() - self.written) {
            return Err(io::Error::other("attach tty output limit exceeded"));
        }
        if self.written != 0 {
            self.bytes.drain(..self.written);
            self.written = 0;
        }
        self.bytes.extend_from_slice(bytes);
        self.flush(fd)
    }

    fn flush(&mut self, fd: RawFd) -> io::Result<()> {
        while self.has_pending() {
            let remaining = &self.bytes[self.written..];
            let written = unsafe {
                libc::write(
                    fd,
                    remaining.as_ptr() as *const libc::c_void,
                    remaining.len(),
                )
            };
            if written > 0 {
                self.written += written as usize;
                continue;
            }
            if written == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "attach tty write returned zero",
                ));
            }
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            if error.kind() == io::ErrorKind::WouldBlock {
                return Ok(());
            }
            return Err(error);
        }
        self.bytes.clear();
        self.written = 0;
        Ok(())
    }
}

impl AttachCompositorState {
    fn new(
        session_id: u32,
        context: command::ClientContext,
        target: String,
    ) -> AttachCompositorState {
        AttachCompositorState {
            target: AttachTargetState {
                session_id,
                stable_target: target,
                context,
            },
            render: AttachRenderState {
                last_render: Vec::new(),
                seen_large_scroll: BTreeMap::new(),
                output_cursor_visible: None,
                last_title: None,
                force_clear: true,
            },
            ui: AttachUiState {
                confirm: None,
                command_prompt: None,
                active_overlay: None,
                status_message: None,
            },
            input: AttachInputState {
                keys: ClientKeyState::new(Instant::now()),
                published_key_table: super::state::DEFAULT_KEY_TABLE.to_string(),
                mouse: MouseInputState::default(),
                key_prompt: KeyPromptState::Idle,
                terminal_reply: None,
                injected: VecDeque::new(),
            },
            io_state: ClientIoState::Active,
            transition: None,
        }
    }
}

fn handle_command_prompt_key(
    prompt: &mut Option<CommandPrompt>,
    key: &str,
    state: &Arc<Mutex<ServerState>>,
    hub: &StatusHub,
    context: &command::ClientContext,
) -> Option<AttachCommandRequest> {
    let Some(active) = prompt.as_mut() else {
        return None;
    };
    match active.handle_key(key, state, hub, context) {
        CommandPromptInput::Continue => {}
        CommandPromptInput::Finish(mut result) => {
            let mut active = prompt.take().expect("command prompt checked");
            if let Some(source) = take_deferred_attach_command(&mut result) {
                return Some(AttachCommandRequest {
                    source,
                    context: context.clone(),
                    continuation: AttachCommandContinuation::Prompt {
                        prompt: Box::new(active),
                    },
                });
            }
            active.complete(&result, state, context);
        }
        CommandPromptInput::Cancel => {
            let mut active = prompt.take().expect("command prompt checked");
            active.cancel_external();
        }
    }
    None
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
        .map(|state| state.session_key_table(target))
        .unwrap_or_else(|| super::state::DEFAULT_KEY_TABLE.to_string())
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
    /// How the client spelled this key, which the pane encoder needs on top of
    /// the semantic identity. tmux keeps the same distinctions in its
    /// `tty_keys` table.
    flags: TtyKeyFlags,
}

/// The terminal-shaped part of a decoded key: which application-mode form it
/// arrived as, and whether its meta modifier was spelled inside the sequence
/// rather than as a leading `ESC`.
#[derive(Clone, Copy, Debug, Default)]
struct TtyKeyFlags {
    cursor: bool,
    keypad: bool,
    implied_meta: bool,
}

impl TtyKeyFlags {
    fn pane_key(self, code: KeyCode) -> PaneKey {
        PaneKey {
            code,
            cursor: self.cursor,
            keypad: self.keypad,
            implied_meta: self.implied_meta,
        }
    }
}

/// The flags an `SS3` sequence carries: both keypad and cursor keys have an
/// application form, and tmux tracks which one the client actually sent.
fn ss3_flags(final_byte: u8) -> TtyKeyFlags {
    TtyKeyFlags {
        cursor: matches!(final_byte, b'A' | b'B' | b'C' | b'D'),
        keypad: matches!(
            final_byte,
            b'M' | b'j' | b'k' | b'm' | b'n' | b'o' | b'p'..=b'y'
        ),
        implied_meta: false,
    }
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
                flags: TtyKeyFlags::default(),
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
                flags: TtyKeyFlags::default(),
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
                    flags: TtyKeyFlags::default(),
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
                flags: TtyKeyFlags::default(),
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
                flags: TtyKeyFlags::default(),
            },
            start + 5,
        ));
    }

    let (name, consumed, mut flags) = match bytes.get(start).copied()? {
        b'O' => {
            let final_byte = *bytes.get(start + 1)?;
            (
                decode_ss3(final_byte)?.to_string(),
                start + 2,
                ss3_flags(final_byte),
            )
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
            let name = decode_csi(params, final_byte)?;
            let flags = TtyKeyFlags {
                // The normal cursor form is an application cursor key too: a
                // pane in DECCKM is told `SS3 A` whichever one the client sent.
                cursor: params.is_empty() && matches!(final_byte, b'A' | b'B' | b'C' | b'D'),
                keypad: false,
                // A modifier spelled in the parameters carries its own meta.
                implied_meta: name.contains("M-"),
            };
            (name, end + 1, flags)
        }
        byte => (meta_prompt_key(byte), start + 1, TtyKeyFlags::default()),
    };
    let name = if meta {
        // A leading `ESC` is meta the pane has to be told about the same way,
        // so it stays a prefix — except for Home and End, which tmux lists with
        // the modifier already implied.
        flags.implied_meta = matches!(name.as_str(), "Home" | "End");
        format!("M-{name}")
    } else {
        name
    };
    Some((
        DecodedTtyKey {
            code: parse_key_name(&name),
            name,
            mouse: None,
            flags,
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
    // The opening report of a drag resolves where the button went down, not
    // where the pointer has already moved to — that is how a press on a border
    // followed by motion is a border drag rather than a drag inside a pane.
    // Only the *location* comes from there: what the pane is told, and what
    // `#{mouse_x}`/`#{mouse_y}` report, is still where the pointer is now.
    let reported_position = event.position;
    if let Some(position) = input.drag_start_position(event) {
        event.position = position;
    }
    let resolved_position = event.position;
    if let Ok(state) = state.lock() {
        let rendered = status_cache.render(&state, target, cols, rows);
        mouse::resolve_event(&state, target, rows, rendered, event);
    }
    input.observe(event, Instant::now());
    event.position = reported_position;
    if resolved_position != reported_position {
        if let Some(local) = event
            .target
            .as_mut()
            .and_then(|target| target.local_position.as_mut())
        {
            local.x = shift_coordinate(local.x, resolved_position.x, reported_position.x);
            local.y = shift_coordinate(local.y, resolved_position.y, reported_position.y);
        }
    }
    apply_focus_follows_mouse(state, target, event);
    decoded.code = event.key_code();
    if let Some(code) = decoded.code {
        decoded.name = code.to_string();
    } else {
        decoded.name = "Mouse".into();
    }
}

/// Move a pane-local coordinate by the same amount its screen coordinate moved.
fn shift_coordinate(local: u16, from: u16, to: u16) -> u16 {
    if to >= from {
        local.saturating_add(to - from)
    } else {
        local.saturating_sub(from - to)
    }
}

/// `focus-follows-mouse`: bare motion over an inactive pane selects it.
///
/// tmux does this inside `server_client_check_mouse` rather than through a
/// binding, so it happens even though `MouseMovePane` cannot be bound at all.
fn apply_focus_follows_mouse(
    state: &Arc<Mutex<ServerState>>,
    target: &str,
    event: &MouseEvent,
) {
    if event.kind != MouseEventKind::Move {
        return;
    }
    let Some(pane_id) = event
        .target
        .as_ref()
        .filter(|resolved| resolved.location == super::key::MouseLocation::Pane)
        .and_then(|resolved| resolved.pane_id)
    else {
        return;
    };
    let Ok(mut st) = state.lock() else {
        return;
    };
    if st.option_for_target(target, "focus-follows-mouse") != Some("on") {
        return;
    }
    let _ = st.select_pane(&format!("%{pane_id}"));
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

fn append_view_output(state: &Arc<Mutex<ServerState>>, target: &str, output: &[u8]) {
    if let Ok(mut state) = state.lock() {
        let _ = state.append_view_output(target, output);
    }
}

/// Wait until either side of an attached client, its active pane, or its agent
/// status subscription has work.
/// Tty readiness needs no flag because the non-blocking input drain runs next.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AttachWaitSources {
    pub(crate) control: RawFd,
    pub(crate) input: RawFd,
    pub(crate) tty_output: RawFd,
    pub(crate) output: RawFd,
    pub(crate) output_generation: u64,
    pub(crate) prompt: RawFd,
    pub(crate) render: RawFd,
    pub(crate) status: RawFd,
    pub(crate) popup_read: RawFd,
    pub(crate) popup_write: RawFd,
}

/// Readiness delivered for one attach compositor turn.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AttachWaitReady {
    pub(crate) control: bool,
    pub(crate) tty_output: bool,
    pub(crate) output: bool,
    pub(crate) prompt: bool,
    pub(crate) render: bool,
    pub(crate) status: bool,
    pub(crate) popup_read: bool,
    pub(crate) popup_write: bool,
}

/// Internal wait operation used by the turn-based attach compositor.
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

/// Observe pane-local large-scroll hints for this client and decide whether the
/// client's terminal needs a software repaint. The revisions are retained per
/// attach loop so one client cannot consume another client's notification.
fn take_large_scroll_repaint(
    state: &ServerState,
    session: &str,
    cols: u16,
    terminal: &dyn TerminalCapabilities,
    seen: &mut BTreeMap<u32, u64>,
) -> bool {
    let Ok((window, _)) = state.active_window_panes(session) else {
        return false;
    };
    let has_scroll_region = term::string_capability(terminal, "csr").is_some();
    let has_margins = term::string_capability(terminal, "Cmg").is_some()
        && term::string_capability(terminal, "Clmg").is_some();
    let mut repaint = false;
    for node in &window.panes {
        let revision = node.pane.observation_state().large_scroll_revision();
        let previous = seen.insert(node.id, revision).unwrap_or(0);
        if revision == 0 || revision == previous {
            continue;
        }
        let partial_width = window
            .pane_rect(node.id)
            .is_some_and(|rect| rect.width < cols);
        if !has_scroll_region || (partial_width && !has_margins) {
            repaint = true;
        }
    }
    repaint
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
        self.stdout
            .as_ref()
            .or(self.stdin.as_ref())
            .map(AsFd::as_fd)
    }

    pub fn input_fd(&self) -> Option<BorrowedFd<'_>> {
        self.stdin
            .as_ref()
            .or(self.stdout.as_ref())
            .map(AsFd::as_fd)
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

fn tty_start_sequence(terminal: &ResolvedTerm, focus_events: bool) -> Vec<u8> {
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
    if terminal.is_vt100_like() {
        // Subscribe to theme changes and ask for the current one, as tmux's
        // `tty_start_tty` does.
        output.extend_from_slice(b"\x1b[?2031h\x1b[?996n");
    }
    // tmux enables focus reporting from `tty_update_features`, gated on the
    // `focus-events` server option.
    if focus_events {
        if let Some(value) = term::string_capability(terminal, "Enfcs") {
            output.extend_from_slice(value);
        }
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
    if terminal.is_vt100_like() {
        output.extend_from_slice(b"\x1b[?2031l");
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
/// cooked *before* the detach/exit handshake writes to it) and disarm this
/// guard so it doesn't fire twice.
struct TermiosGuard {
    fd: RawFd,
    saved: Option<libc::termios>,
}

impl TermiosGuard {
    fn restore(&self) {
        if let Some(saved) = self.saved.as_ref() {
            restore_termios(self.fd, saved);
        }
    }

    fn restore_and_disarm(&mut self) {
        self.restore();
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

pub(crate) fn start_attach_session<W>(
    args: &[String],
    client_tty: ClientTty,
    state: &Arc<Mutex<ServerState>>,
    hub: &StatusHub,
    context: &command::ClientContext,
    writer: &mut W,
    pane_io_mode: PaneIoMode,
) -> Result<AttachSession, AttachStartFailure>
where
    W: FrameWriter + ?Sized,
{
    let target = match command::classify(args) {
        command::Intent::Attach => {
            let supplied_target = explicit_target_session(args);
            let mut st = state
                .lock()
                .map_err(|_| io::Error::other("state poisoned"))?;
            let target = attach_target(supplied_target, &mut st, context)
                .map_err(AttachStartFailure::Client)?;
            if st.find(&target).is_none() {
                return Err(AttachStartFailure::Client(format!(
                    "can't find session: {target}\n"
                )));
            }
            target
        }
        command::Intent::NewAttach => {
            // tmux opens the terminal before creating the session, so a client
            // that cannot attach leaves nothing behind.
            AttachSession::check_terminal(&client_tty)?;
            let mut st = state
                .lock()
                .map_err(|_| io::Error::other("state poisoned"))?;
            command::new_session_for_attach(args, &mut st, context)
                .map_err(AttachStartFailure::Client)?
        }
        command::Intent::Command => {
            return Err(AttachStartFailure::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "not an attach command",
            )));
        }
    };
    AttachSession::start_in_mode(
        &target,
        client_tty,
        state,
        hub,
        context,
        writer,
        pane_io_mode,
    )
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
    let active_copy = active_mode
        .is_none()
        .then(|| st.active_copy_state(target))
        .flatten();
    let copy_view = active_copy.map(|copy| CopyModeView::new(st, target, copy, cols, terminal));
    let (all_rows, cursor, cursor_visible, restore_cursor, frame_capacity) =
        if let Some(view) = active_mode {
            (
                render_mode_rows(view, cols as usize, pane_height as usize),
                Vec::new(),
                false,
                false,
                usize::from(cols) * usize::from(pane_height) + 256,
            )
        } else if let Some(copy_view) = copy_view.as_ref() {
            (
                copy_view.rows(pane_height),
                copy_view.cursor(pane_height, cols),
                true,
                true,
                copy_view.serialized_len() + 256,
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
    let line_number_width = copy_view
        .as_ref()
        .map(CopyModeView::line_number_width)
        .unwrap_or(0)
        .min(cols.saturating_sub(1) as usize);

    // Hide the cursor for the duration of the repaint so it doesn't visibly
    // travel across every row we position to; it is restored (if the pane wants
    // it shown) at the very end, at the pane's real cursor location.
    out.extend_from_slice(b"\x1b[?25l");
    // Start from a known-default SGR so the first row's erase-to-EOL doesn't
    // inherit a stray background color from a prior frame.
    append_terminal_style_reset(&mut out, terminal);

    // Draw each pane row in place: position, erase the row, rewrite content.
    //
    // The erase precedes the content because a row that fills the last column
    // leaves the terminal in pending wrap with the cursor still *on* that
    // column, so an erase-to-EOL afterwards would wipe the character just
    // written there. The split-pane path avoids the same hazard by erasing
    // (`ECH`) before it paints.
    for i in 0..pane_height as usize {
        out.extend_from_slice(format!("\x1b[{};1H", usize::from(pane_top) + i + 1).as_bytes());
        out.extend_from_slice(b"\x1b[K");
        if let Some(copy_view) = copy_view.as_ref() {
            copy_view.render_line_number(&mut out, i);
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
    }
    if let Some(copy_view) = copy_view.as_ref() {
        copy_view.render_overlays(&mut out, pane_top, 0, pane_height, cols);
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
        let copy_view = node
            .copy
            .as_ref()
            .map(|copy| CopyModeView::new(st, target, copy, width, terminal));
        let (pane_rows, cursor, row_start) = if let Some(view) = node.mode_view.as_ref() {
            (
                render_mode_rows(view, width as usize, height as usize),
                Vec::new(),
                0,
            )
        } else if let Some(copy_view) = copy_view.as_ref() {
            (copy_view.rows(height), copy_view.cursor(height, width), 0)
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
        let line_number_width = copy_view
            .as_ref()
            .map(CopyModeView::line_number_width)
            .unwrap_or(0)
            .min(width.saturating_sub(1) as usize);
        for row in 0..height {
            out.extend_from_slice(
                format!("\x1b[{};{}H", pane_top + top + row + 1, left + 1).as_bytes(),
            );
            append_terminal_style_reset(&mut out, terminal);
            out.extend_from_slice(format!("\x1b[{}X", width).as_bytes());
            out.extend_from_slice(
                format!("\x1b[{};{}H", pane_top + top + row + 1, left + 1).as_bytes(),
            );
            if let Some(copy_view) = copy_view.as_ref() {
                copy_view.render_line_number(&mut out, row as usize);
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
        if let Some(copy_view) = copy_view.as_ref() {
            copy_view.render_overlays(&mut out, pane_top + top, left, height, width);
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

#[cfg(test)]
mod tests {
    use super::super::pane::Pane;
    use super::super::state::PaneSpec;
    use super::*;
    use std::io::Read as _;
    use std::os::unix::net::UnixStream;

    #[test]
    fn tty_output_retains_a_bounded_suffix_until_writable() {
        let (writer, mut reader) = UnixStream::pair().expect("socket pair");
        writer.set_nonblocking(true).expect("nonblocking writer");
        reader.set_nonblocking(true).expect("nonblocking reader");
        let fd = writer.as_raw_fd();
        let filler = [b'x'; 16 * 1024];
        loop {
            let written =
                unsafe { libc::write(fd, filler.as_ptr() as *const libc::c_void, filler.len()) };
            if written > 0 {
                continue;
            }
            let error = io::Error::last_os_error();
            assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
            break;
        }

        let mut output = TtyOutput::new();
        output.queue(fd, b"tail").expect("queue tail");
        assert!(output.has_pending());

        let mut scratch = [0u8; 64 * 1024];
        loop {
            match reader.read(&mut scratch) {
                Ok(0) => panic!("socket closed while draining filler"),
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) => panic!("failed to drain filler: {error}"),
            }
        }
        output.flush(fd).expect("flush tail");
        assert!(!output.has_pending());
        assert_eq!(reader.read(&mut scratch).expect("read tail"), 4);
        assert_eq!(&scratch[..4], b"tail");
    }

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
        // These take a pane target: a bare `0` is pane index 0, which is the
        // *inactive* half of the split. `0:` is the session's active pane, the
        // one the frame is fed through.
        st.set_pane_mode("0:", Some("copy-mode"))
            .expect("enter copy mode in split");
        st.copy_mode_command("0:", "history-top", true, "")
            .expect("move split copy cursor");
        for command in ["begin-selection", "cursor-right", "cursor-right"] {
            st.copy_mode_command("0:", command, true, "")
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

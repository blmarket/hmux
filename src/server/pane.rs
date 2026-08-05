//! A pane: a child process on a PTY, its output parsed by a libghostty-vt
//! [`Terminal`].
//!
//! This is where the "clone" earns its name — instead of proxying to a backing
//! tmux, hmux owns the pty/child and maintains the screen itself. tmux keeps this
//! state in `window_pane` + `screen`/`input.c`; here the grid lives in libghostty
//! and the master fd is drained by the central event loop.
//!
//! Only a text-emulation slice is implemented: spawn, feed output → grid, send
//! input, resize, dump. Compositing multiple panes onto an attached client's tty
//! is the next milestone (see the module docs).

use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, VecDeque};
use std::ffi::CString;
use std::io::{self, Read, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd};
use std::os::raw::{c_int, c_void};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::ptr;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

use libc::pid_t;

use crate::ghostty::Terminal;
use crate::observability::v1::{PaneObservability, PaneProcess, ScreenSource, ScreenTail};
use crate::server::input_keys::ExtendedKeys;
use crate::platform::{CurrentPlatform, ForkOutcome, OutputWakeup, Platform};
use crate::server::task::{Coroutine, FdInterest, ReadySet, TaskPoll, WaitRequest, WaitToken};


/// A single pane. Holds the emulated screen and, if live, the child on its pty.
pub struct Pane {
    /// Read-only state shared with observation handles. Keeping this separate
    /// from the PTY owner lets consumers inspect a resolved pane without
    /// retaining the native server's global state lock.
    observation: Rc<NativePaneObservation>,
    /// Terminal queries emitted by the child which must be relayed to an
    /// attached outer terminal. Ghostty consumes OSC sequences while updating
    /// the grid, so they need a separate side channel to reach the compositor.
    terminal_queries: Rc<RefCell<VecDeque<Vec<u8>>>>,
    /// The running child + pty, or `None` for an inert (process-less) pane.
    child: Option<Child>,
    /// Bytes queued to be written to the child's pty (keystrokes and terminal-
    /// query replies) that have not yet been accepted by the child. The pty
    /// master is non-blocking, so a child that stops reading its stdin (a stalled
    /// full-screen app) can never block a server thread writing to it — pending
    /// bytes wait here and are flushed by the active I/O driver on writability. The
    /// buffer is bounded; once full, further input is dropped rather than allowed
    /// to stall the shared server, matching how tmux tolerates an unresponsive
    /// pane. Shared with the active I/O driver.
    pending_input: Rc<RefCell<VecDeque<u8>>>,
    /// Original process specification retained for command-less respawns.
    spawn_spec: Option<PaneSpawnSpec>,
    /// Buffer the PTY reader appends to when `pipe-pane -O` is active.
    pipe_output: Rc<RefCell<PanePipeOutbound>>,
    pipe_output_active: Rc<Cell<bool>>,
    pipe: Option<PanePipe>,
    /// Pipe children the loop has not adopted yet.
    new_pipes: Vec<PanePipeIo>,
    event_io: Option<PaneIo>,
    runtime_id: u64,
    cols: u16,
    rows: u16,
}

#[derive(Clone, Debug)]
pub(crate) struct PaneSpawnSpec {
    pub(crate) argv: Vec<String>,
    pub(crate) cwd: Option<PathBuf>,
}

/// How much pane output may wait for a `pipe-pane` child that is not reading.
///
/// The old writer thread queued without bound, so a wedged pipe command grew
/// the server's memory for as long as the pane produced output. The loop keeps
/// the newest bytes instead: a pipe is a diagnostic tap, and stalling the pane
/// to preserve one would be worse than dropping from a tap nobody is draining.
const PANE_PIPE_OUTBOUND_CAP: usize = 4 * 1024 * 1024;

/// Pane output waiting to be written to an open `pipe-pane` child.
#[derive(Default)]
pub(crate) struct PanePipeOutbound {
    queue: VecDeque<u8>,
    /// Set when the pane stops piping, so the job closes the child's stdin.
    closed: bool,
}

impl PanePipeOutbound {
    fn push(&mut self, bytes: &[u8]) {
        self.queue.extend(bytes);
        let overflow = self.queue.len().saturating_sub(PANE_PIPE_OUTBOUND_CAP);
        if overflow != 0 {
            self.queue.drain(..overflow);
        }
    }
}

struct PanePipe {
    pid: u32,
    alive: Rc<Cell<bool>>,
    outbound: Rc<RefCell<PanePipeOutbound>>,
}

impl PanePipe {
    /// Tell the job to close the child's stdin, which is what a `pipe-pane`
    /// command sees as end of input.
    fn close_outbound(&self) {
        {
            let mut outbound = self.outbound.borrow_mut();
            outbound.closed = true;
        }
    }
}

impl Drop for PanePipe {
    fn drop(&mut self) {
        self.close_outbound();
        if self.alive.get() {
            // Closing a pipe is asynchronous. SIGHUP mirrors the lifetime of a
            // tmux job without making the command path wait for the child.
            unsafe {
                libc::kill(self.pid as pid_t, libc::SIGHUP);
            }
        }
    }
}

/// The loop-owned half of an open `pipe-pane` child.
///
/// Both directions and the child's exit are driven as one job: the pane's
/// buffered output is written to the child's stdin as it accepts it, and the
/// child's stdout is fed back into the pane's input queue.
pub(crate) struct PanePipeIo {
    child: std::process::Child,
    stdin: Option<std::process::ChildStdin>,
    stdout: Option<std::process::ChildStdout>,
    /// The pty master pipe input is written to, when the pipe reads back.
    master: Option<OwnedFd>,
    pending_input: Rc<RefCell<VecDeque<u8>>>,
    outbound: Rc<RefCell<PanePipeOutbound>>,
    alive: Rc<Cell<bool>>,
}

impl PanePipeIo {
    const STDIN: WaitToken = WaitToken::new(0);
    const STDOUT: WaitToken = WaitToken::new(1);

    /// A write-only pipe job: `payload` is handed to the child's stdin as it
    /// accepts it, and the child is reaped once it exits.
    ///
    /// `copy-pipe` uses this to give a selection to a filter. Writing it
    /// inline would stall the loop for as long as a slow — or simply
    /// uninterested — consumer took to drain more than a pipe buffer.
    pub(crate) fn for_write(
        child: std::process::Child,
        stdin: std::process::ChildStdin,
        payload: &[u8],
    ) -> io::Result<Self> {
        set_nonblocking(stdin.as_raw_fd())?;
        Ok(Self {
            child,
            stdin: Some(stdin),
            stdout: None,
            master: None,
            pending_input: Rc::new(RefCell::new(VecDeque::new())),
            // The whole payload is queued up front and the write end closes
            // after it, which is the end of input the child waits for. The
            // pane cap does not apply: dropping from a selection would hand
            // the filter a silently truncated one.
            outbound: Rc::new(RefCell::new(PanePipeOutbound {
                queue: payload.iter().copied().collect(),
                closed: true,
            })),
            alive: Rc::new(Cell::new(true)),
        })
    }

    /// Write what the child will take without blocking. The buffer keeps what
    /// it will not.
    fn write_outbound(&mut self) {
        let Some(stdin) = self.stdin.as_mut() else {
            return;
        };
        let mut outbound = self.outbound.borrow_mut();
        while !outbound.queue.is_empty() {
            let (front, _) = outbound.queue.as_slices();
            match stdin.write(front) {
                Ok(0) => break,
                Ok(count) => {
                    outbound.queue.drain(..count);
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return,
                Err(_) => {
                    // The child stopped reading; nothing more will reach it.
                    outbound.queue.clear();
                    outbound.closed = true;
                    break;
                }
            }
        }
        if outbound.closed && outbound.queue.is_empty() {
            // Dropping the write end reports end of input to the child.
            self.stdin = None;
        }
    }

    /// Feed whatever the child has written back into the pane.
    fn read_inbound(&mut self) {
        let Some(stdout) = self.stdout.as_mut() else {
            return;
        };
        let Some(master) = self.master.as_ref() else {
            self.stdout = None;
            return;
        };
        let mut bytes = [0u8; 4096];
        loop {
            match stdout.read(&mut bytes) {
                Ok(0) => break,
                Ok(count) => {
                    enqueue_pane_input(master.as_raw_fd(), &self.pending_input, &bytes[..count]);
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return,
                Err(_) => break,
            }
        }
        self.stdout = None;
    }
}

impl Coroutine for PanePipeIo {
    type Output = ();

    fn wait(&self) -> WaitRequest<'_> {
        let mut sources = Vec::new();
        // Writable interest only while there is something to write; an idle
        // pipe would otherwise report ready on every poll.
        let outbound = self.outbound.borrow();
        let has_outbound = !outbound.queue.is_empty() || outbound.closed;
        drop(outbound);
        if let Some(stdin) = self.stdin.as_ref().filter(|_| has_outbound) {
            sources.push(FdInterest::writable(Self::STDIN, stdin.as_fd()));
        }
        if let Some(stdout) = self.stdout.as_ref() {
            sources.push(FdInterest::readable(Self::STDOUT, stdout.as_fd()));
        }
        // With neither end left the job is only waiting for the child to exit,
        // which nothing portable makes readable.
        let deadline = (sources.is_empty()).then(|| Instant::now() + PIPE_REAP_RETRY);
        WaitRequest::new(sources, deadline)
    }

    fn resume(&mut self, _ready: &ReadySet) -> TaskPoll<Self::Output> {
        self.write_outbound();
        self.read_inbound();
        if self.stdin.is_some() || self.stdout.is_some() {
            return TaskPoll::Pending;
        }
        match self.child.try_wait() {
            Ok(None) => TaskPoll::Pending,
            Ok(Some(_)) | Err(_) => {
                self.alive.set(false);
                TaskPoll::Ready(())
            }
        }
    }
}

/// How often a pipe job that has closed both ends asks whether its child has
/// exited. Nothing portable makes a child's exit readable.
const PIPE_REAP_RETRY: Duration = Duration::from_millis(50);

static NEXT_PANE_RUNTIME_ID: AtomicU64 = AtomicU64::new(1);

/// The live half of a pane: the child pid and the pty master.
struct Child {
    pid: pid_t,
    master: OwnedFd,
    alive: Rc<Cell<bool>>,
    reaped: bool,
    termination_requested: bool,
    exit_code: Option<i32>,
    /// How the child ended, once it has been waited for: tmux's `wp->status`
    /// and `wp->dead_time`, which is what `#{pane_dead_*}` reports.
    death: Option<PaneDeath>,
}

/// How a pane's child ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PaneDeath {
    /// The exit status, for a child that exited on its own.
    pub(crate) status: Option<i32>,
    /// The signal number, for a child that was killed. tmux prints the name
    /// where the platform has `sys_signame` and the number otherwise; the
    /// pinned build prints the number.
    pub(crate) signal: Option<i32>,
    /// When it was reaped, as `#{pane_dead_time}` reports it.
    pub(crate) at: SystemTime,
}

struct ObservedChild {
    pid: u32,
    alive: Rc<Cell<bool>>,
}

/// State which remains valid for the lifetime of a resolved observation
/// handle, even if the pane is subsequently removed from the server tree.
pub(crate) struct NativePaneObservation {
    term: Rc<RefCell<Terminal>>,
    revision: Cell<u64>,
    /// Revision of the latest scroll operation whose vertical region is large
    /// enough for tmux to prefer one deferred repaint over immediate row draws.
    /// This is monotonic so each attached client can observe it independently.
    large_scroll_revision: Cell<u64>,
    redraw_detector: RefCell<ScrollRedrawDetector>,
    control_output: RefCell<ControlOutputJournal>,
    /// Last DECSCUSR parameter emitted by the pane (0..=6). The VT formatter
    /// restores cursor position but does not serialize this terminal state.
    cursor_shape: Cell<u8>,
    /// The pane's reportable VT modes, republished as one snapshot at the end
    /// of each output batch. Ghostty owns the emulation; these are tracked
    /// beside it because it does not expose them.
    modes: Cell<PaneModeSnapshot>,
    /// Set when the pane sent DSR ?996 and is waiting for an answer.
    theme_query: Cell<bool>,
    background: RefCell<String>,
    /// The rest of the OSC-set pane state, behind `#{pane_fg}`,
    /// `#{cursor_colour}` and `#{pane_path}`.
    foreground: RefCell<String>,
    cursor_colour: RefCell<String>,
    path: RefCell<String>,
    /// The OSC 9;4 progress bar, behind `#{pane_pb_state}` (0..=4) and
    /// `#{pane_pb_progress}`.
    progress_state: Cell<u8>,
    progress_value: Cell<u8>,
    /// The pane's tab stops, or `None` while they are still the default every
    /// eight columns. A resize puts them back, as tmux's `screen_reset_tabs`
    /// does.
    tab_stops: RefCell<Option<BTreeSet<u16>>>,
    /// The pane's width, which is what the default tab stops are laid out
    /// against. Held here so a tab edit never has to lock the terminal.
    columns: Cell<u16>,
    /// Whether the pane is on its alternate screen (`#{alternate_on}`), and the
    /// cursor DECSET 1049 saved on the way in. tmux leaves the saved position
    /// behind on the way out, and starts it at `UINT_MAX` to mean "never set" —
    /// which is the value `#{alternate_saved_x}` reports.
    alternate_on: Cell<bool>,
    alternate_saved_x: Cell<u32>,
    alternate_saved_y: Cell<u32>,
    child: Option<ObservedChild>,
    output_waiters: RefCell<Vec<Weak<OutputEvent>>>,
    output_timing: Option<Rc<OutputTiming>>,
    last_output_at: RefCell<Option<Instant>>,
    bell_count: Cell<u64>,
    /// OSC 52 sequences the pane emitted, waiting for the server to apply the
    /// `set-clipboard`/`get-clipboard` policy to them.
    clipboard_events: RefCell<VecDeque<PaneClipboardEvent>>,
    /// `DCS tmux;` payloads the pane emitted, waiting for the server to put them
    /// on the client ttys they are allowed to reach.
    passthrough: RefCell<VecDeque<PanePassthrough>>,
    /// The title the pane last set for itself, tmux's `screen->title`. Tracked
    /// here rather than read back from Ghostty because tmux's limit on it is
    /// `input-buffer-size`, and Ghostty's is its own.
    announced_title: RefCell<Option<String>>,
    /// The options that decide what the pane's own output is allowed to do.
    output_policy: PaneOutputPolicyCell,
}

/// The options a pane's *own output* has to be parsed against.
///
/// tmux reads these from `wp->options` inside `input_parse`, which runs with the
/// whole server in reach. hmux parses a pane's bytes away from the server state,
/// so the resolved values are pushed to the pane instead and re-pushed whenever
/// they can have changed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PaneOutputPolicy {
    /// `alternate-screen`: whether smcup and rmcup switch screens at all.
    pub(crate) alternate_screen: bool,
    /// `allow-set-title`: whether the pane may retitle itself.
    pub(crate) allow_set_title: bool,
    /// `allow-passthrough`: how far a `DCS tmux;` payload reaches.
    pub(crate) passthrough: PassthroughPolicy,
    /// `input-buffer-size`: how long a terminal string may grow before the
    /// parser abandons it.
    pub(crate) input_buffer_size: u32,
    /// `pane-colours`, packed as `0xrrggbb` by index — the palette a query
    /// falls back to when the pane has set nothing itself.
    pub(crate) palette: Vec<Option<u32>>,
}

/// `allow-passthrough`: whether a pane may write to a client's terminal
/// directly, and whether it has to be the pane on screen to do so.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum PassthroughPolicy {
    #[default]
    Off,
    /// `on`: only clients whose current window holds the pane.
    Visible,
    /// `all`: also clients that merely have the window linked, which is tmux's
    /// `TTY_CTX_INVISIBLE_PANES`.
    Always,
}

/// [`PaneOutputPolicy`] as the pane's parser reads it: shared with the server,
/// so each field is an atomic rather than behind the state lock.
struct PaneOutputPolicyCell {
    alternate_screen: Cell<bool>,
    allow_set_title: Cell<bool>,
    passthrough: Cell<u8>,
    input_buffer_size: Cell<u32>,
    /// Read only where a pane's bytes are parsed, so a plain mutex is enough.
    palette: RefCell<Vec<Option<u32>>>,
}

impl PaneOutputPolicyCell {
    fn load(&self) -> PaneOutputPolicy {
        PaneOutputPolicy {
            alternate_screen: self.alternate_screen.get(),
            allow_set_title: self.allow_set_title.get(),
            passthrough: match self.passthrough.get() {
                1 => PassthroughPolicy::Visible,
                2 => PassthroughPolicy::Always,
                _ => PassthroughPolicy::Off,
            },
            input_buffer_size: self.input_buffer_size.get(),
            palette: self.palette.borrow().clone(),
        }
    }

    fn store(&self, policy: PaneOutputPolicy) {
        self.alternate_screen
            .set(policy.alternate_screen);
        self.allow_set_title
            .set(policy.allow_set_title);
        self.passthrough.set(match policy.passthrough {
                PassthroughPolicy::Off => 0,
                PassthroughPolicy::Visible => 1,
                PassthroughPolicy::Always => 2,
            });
        self.input_buffer_size
            .set(policy.input_buffer_size);
        {
            let mut palette = self.palette.borrow_mut();
            *palette = policy.palette;
        }
    }
}

/// The options' own defaults, which a pane parses against until the server's
/// first refresh reaches it.
impl Default for PaneOutputPolicyCell {
    fn default() -> Self {
        Self {
            alternate_screen: Cell::new(true),
            allow_set_title: Cell::new(true),
            passthrough: Cell::new(0),
            input_buffer_size: Cell::new(INPUT_BUFFER_DEFAULT_SIZE),
            palette: RefCell::new(Vec::new()),
        }
    }
}

/// One `DCS tmux; … ST` payload seen in a pane's output, already stripped of
/// its prefix and terminator, as `screen_write_rawstring` receives it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PanePassthrough {
    pub(crate) data: Vec<u8>,
    /// `allow-passthrough` was `all` when the sequence completed, so a client
    /// that merely has the window linked gets the payload too — tmux's
    /// `TTY_CTX_INVISIBLE_PANES`.
    pub(crate) invisible_panes: bool,
}

/// One OSC 52 sequence seen in a pane's output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PaneClipboardEvent {
    /// `OSC 52 ; <selection> ; <base64>` — an application setting the
    /// clipboard, already decoded.
    Set { data: Vec<u8> },
    /// `OSC 52 ; <selection> ; ?` — an application asking for it.
    Query {
        /// The selection character echoed back in the reply; empty when the
        /// request named none that tmux recognises.
        selection: String,
        /// Whether the request ended with ST rather than BEL, which the reply
        /// mirrors.
        string_terminator: bool,
    },
}

struct OutputEvent {
    wakeup: <CurrentPlatform as Platform>::OutputWakeup,
}

#[derive(Clone, Copy)]
struct ScrollRegion {
    top: u16,
    bottom: u16,
}

#[derive(Clone, Copy)]
enum ScrollEdge {
    Top,
    Bottom,
    Inside,
    Any,
}

#[derive(Clone, Copy)]
struct ScrollAction {
    byte_index: usize,
    region: ScrollRegion,
    edge: ScrollEdge,
    rows: u16,
}

impl ScrollAction {
    fn needs_large_redraw(self, cursor_y: u16) -> bool {
        if self.region.bottom.saturating_sub(self.region.top) < self.rows / 2 {
            return false;
        }
        match self.edge {
            ScrollEdge::Top => cursor_y == self.region.top,
            ScrollEdge::Bottom => cursor_y == self.region.bottom,
            ScrollEdge::Inside => (self.region.top..=self.region.bottom).contains(&cursor_y),
            ScrollEdge::Any => true,
        }
    }
}

#[derive(Default)]
struct CsiState {
    params: [u16; 2],
    present: [bool; 2],
    index: usize,
    private: bool,
    intermediate: Option<u8>,
}

impl CsiState {
    fn parameter(&self, index: usize, default: u16) -> u16 {
        self.present[index]
            .then_some(self.params[index])
            .filter(|value| *value != 0)
            .unwrap_or(default)
    }
}

#[derive(Default)]
enum RedrawParserState {
    #[default]
    Ground,
    Escape,
    Csi(CsiState),
    String,
    StringEscape,
}

/// Minimal streaming VT parser for operations which can scroll a vertical
/// region. Ghostty remains the terminal parser and source of truth; this only
/// retains enough metadata to choose between row and pane repainting.
struct ScrollRedrawDetector {
    rows: u16,
    explicit_region: Option<ScrollRegion>,
    state: RedrawParserState,
}

impl ScrollRedrawDetector {
    fn new(rows: u16) -> Self {
        Self {
            rows: rows.max(1),
            explicit_region: None,
            state: RedrawParserState::Ground,
        }
    }

    fn resize(&mut self, rows: u16) {
        self.rows = rows.max(1);
        self.explicit_region = None;
    }

    fn region(&self) -> ScrollRegion {
        self.explicit_region.unwrap_or(ScrollRegion {
            top: 0,
            bottom: self.rows.saturating_sub(1),
        })
    }

    fn scan(&mut self, bytes: &[u8]) -> Vec<ScrollAction> {
        let mut actions = Vec::new();
        for (byte_index, &byte) in bytes.iter().enumerate() {
            if let Some(edge) = self.feed_byte(byte) {
                actions.push(ScrollAction {
                    byte_index,
                    region: self.region(),
                    edge,
                    rows: self.rows,
                });
            }
        }
        actions
    }

    fn feed_byte(&mut self, byte: u8) -> Option<ScrollEdge> {
        let state = std::mem::take(&mut self.state);
        match state {
            RedrawParserState::Ground => match byte {
                b'\n' | 0x0b | 0x0c => Some(ScrollEdge::Bottom),
                0x1b => {
                    self.state = RedrawParserState::Escape;
                    None
                }
                0x9b => {
                    self.state = RedrawParserState::Csi(CsiState::default());
                    None
                }
                0x90 | 0x98 | 0x9d | 0x9e | 0x9f => {
                    self.state = RedrawParserState::String;
                    None
                }
                _ => None,
            },
            RedrawParserState::Escape => match byte {
                b'[' => {
                    self.state = RedrawParserState::Csi(CsiState::default());
                    None
                }
                b'P' | b'X' | b']' | b'^' | b'_' => {
                    self.state = RedrawParserState::String;
                    None
                }
                b'D' => Some(ScrollEdge::Bottom),
                b'M' => Some(ScrollEdge::Top),
                b'c' => {
                    self.explicit_region = None;
                    None
                }
                0x1b => {
                    self.state = RedrawParserState::Escape;
                    None
                }
                _ => None,
            },
            RedrawParserState::Csi(mut csi) => match byte {
                b'0'..=b'9' => {
                    if csi.index < csi.params.len() {
                        csi.present[csi.index] = true;
                        csi.params[csi.index] = csi.params[csi.index]
                            .saturating_mul(10)
                            .saturating_add(u16::from(byte - b'0'));
                    }
                    self.state = RedrawParserState::Csi(csi);
                    None
                }
                b';' => {
                    csi.index = csi.index.saturating_add(1);
                    self.state = RedrawParserState::Csi(csi);
                    None
                }
                0x20..=0x2f => {
                    csi.intermediate = Some(byte);
                    self.state = RedrawParserState::Csi(csi);
                    None
                }
                0x3c..=0x3f => {
                    csi.private = true;
                    self.state = RedrawParserState::Csi(csi);
                    None
                }
                0x40..=0x7e => self.finish_csi(byte, &csi),
                b'\n' | 0x0b | 0x0c => {
                    self.state = RedrawParserState::Csi(csi);
                    Some(ScrollEdge::Bottom)
                }
                0x1b => {
                    self.state = RedrawParserState::Escape;
                    None
                }
                _ => {
                    self.state = RedrawParserState::Csi(csi);
                    None
                }
            },
            RedrawParserState::String => match byte {
                0x07 | 0x9c => None,
                0x1b => {
                    self.state = RedrawParserState::StringEscape;
                    None
                }
                _ => {
                    self.state = RedrawParserState::String;
                    None
                }
            },
            RedrawParserState::StringEscape => {
                if byte != b'\\' && byte != 0x9c {
                    self.state = if byte == 0x1b {
                        RedrawParserState::StringEscape
                    } else {
                        RedrawParserState::String
                    };
                }
                None
            }
        }
    }

    fn finish_csi(&mut self, final_byte: u8, csi: &CsiState) -> Option<ScrollEdge> {
        if final_byte == b'p' && csi.intermediate == Some(b'!') {
            self.explicit_region = None;
            return None;
        }
        if csi.private || csi.intermediate.is_some() {
            return None;
        }
        match final_byte {
            b'r' => {
                let top = csi.parameter(0, 1);
                let bottom = csi.parameter(1, self.rows);
                if top < bottom && bottom <= self.rows {
                    self.explicit_region = Some(ScrollRegion {
                        top: top - 1,
                        bottom: bottom - 1,
                    });
                }
                None
            }
            b'S' | b'T' => Some(ScrollEdge::Any),
            b'L' | b'M' => Some(ScrollEdge::Inside),
            _ => None,
        }
    }
}

const CONTROL_OUTPUT_LIMIT: usize = 1024 * 1024;

#[derive(Default)]
struct ControlOutputJournal {
    bytes: VecDeque<u8>,
    end: u64,
}

impl ControlOutputJournal {
    fn append(&mut self, bytes: &[u8]) {
        self.bytes.extend(bytes);
        self.end = self.end.saturating_add(bytes.len() as u64);
        if self.bytes.len() > CONTROL_OUTPUT_LIMIT {
            self.bytes
                .drain(..self.bytes.len().saturating_sub(CONTROL_OUTPUT_LIMIT));
        }
    }

    #[cfg(test)]
    fn since(&self, offset: u64) -> (u64, Vec<u8>) {
        let start = self.end.saturating_sub(self.bytes.len() as u64);
        let offset = offset.clamp(start, self.end);
        let skip = (offset - start) as usize;
        (self.end, self.bytes.iter().skip(skip).copied().collect())
    }

    fn chunk(&self, offset: u64, limit: usize) -> (u64, u64, Vec<u8>) {
        let start = self.end.saturating_sub(self.bytes.len() as u64);
        let offset = offset.clamp(start, self.end);
        let skip = (offset - start) as usize;
        let bytes = self
            .bytes
            .iter()
            .skip(skip)
            .take(limit)
            .copied()
            .collect::<Vec<_>>();
        (offset.saturating_add(bytes.len() as u64), self.end, bytes)
    }
}

/// Restore the signal dispositions expected by a freshly executed pane.
///
/// The daemon changes process-wide signal handling for its own control flow.
/// A forked pane must not inherit those changes: unlike an interactive shell,
/// a directly executed program may not repair them before doing useful work.
/// Keep this list aligned with tmux's `proc_clear_signals`.
unsafe fn reset_child_signal_dispositions() {
    let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
    action.sa_sigaction = libc::SIG_DFL;
    unsafe {
        libc::sigemptyset(&mut action.sa_mask);
    }

    for signal in [
        libc::SIGPIPE,
        libc::SIGTSTP,
        libc::SIGINT,
        libc::SIGQUIT,
        libc::SIGHUP,
        libc::SIGCHLD,
        libc::SIGCONT,
        libc::SIGTERM,
        libc::SIGUSR1,
        libc::SIGUSR2,
        libc::SIGWINCH,
    ] {
        unsafe {
            libc::sigaction(signal, &action, ptr::null_mut());
        }
    }
}

/// Remove the daemon thread's blocked-signal mask before executing a pane.
unsafe fn clear_child_signal_mask() {
    let mut empty: libc::sigset_t = unsafe { std::mem::zeroed() };
    unsafe {
        libc::sigemptyset(&mut empty);
        libc::sigprocmask(libc::SIG_SETMASK, &empty, ptr::null_mut());
    }
}

/// Per-consumer pane-output notification. Each attached client gets its own
/// wakeup so one client cannot consume another client's notification.
pub(crate) struct OutputSubscription {
    event: Rc<OutputEvent>,
    output_timing: Option<Rc<OutputTiming>>,
}

/// Timestamp side channel used only by the opt-in attach latency monitor.
/// Keeping it beside the wakeup lets the attach thread distinguish time spent
/// waiting for the pane from time spent waiting to be scheduled after output.
struct OutputTiming {
    last_at: RefCell<Option<Instant>>,
}

/// What happened to one batch of bytes offered to a pane's non-blocking PTY.
///
/// Kept crate-private because this is attach-loop instrumentation, not part of
/// the pane API. `queued` bytes were accepted by hmux but had not reached the
/// PTY when the call returned; `dropped` bytes did not fit in the bounded queue.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PaneInputStats {
    pub(crate) written: usize,
    pub(crate) queued: usize,
    pub(crate) dropped: usize,
}

impl PaneInputStats {
    pub(crate) fn accepted(self) -> usize {
        self.written + self.queued
    }
}

impl OutputSubscription {
    /// Subscribe one consumer wakeup to every pane currently displayed in a
    /// window. The active pane supplies the optional latency timestamp, while
    /// output from any pane wakes the compositor.
    pub(crate) fn for_panes<'a>(
        panes: impl IntoIterator<Item = &'a Pane>,
        active_pane: &Pane,
    ) -> io::Result<Self> {
        let event = Rc::new(OutputEvent {
            wakeup: CurrentPlatform::new_output_wakeup()?,
        });
        for pane in panes {
            pane.observation.register_output_event(&event)?;
        }
        Ok(Self {
            event,
            output_timing: active_pane
                .observation
                .output_timing
                .as_ref()
                .map(Rc::clone),
        })
    }

    pub(crate) fn as_raw_fd(&self) -> c_int {
        self.event.wakeup.as_fd().as_raw_fd()
    }

    pub(crate) fn as_fd(&self) -> BorrowedFd<'_> {
        self.event.wakeup.as_fd()
    }

    pub(crate) fn drain(&self) {
        let _ = self.event.wakeup.clear();
    }

    pub(crate) fn last_output_at(&self) -> Option<Instant> {
        *self.output_timing.as_ref()?.last_at.borrow()
    }
}

impl NativePaneObservation {
    fn new(
        term: Rc<RefCell<Terminal>>,
        child: Option<ObservedChild>,
        cols: u16,
        rows: u16,
    ) -> Self {
        let latency_enabled = matches!(
            std::env::var("HMUX_LATENCY"),
            Ok(value) if !value.is_empty() && value != "0"
        );
        Self {
            term,
            revision: Cell::new(0),
            large_scroll_revision: Cell::new(0),
            redraw_detector: RefCell::new(ScrollRedrawDetector::new(rows)),
            control_output: RefCell::new(ControlOutputJournal::default()),
            cursor_shape: Cell::new(0),
            modes: Cell::new(PaneModeSnapshot::default()),
            theme_query: Cell::new(false),
            background: RefCell::new("default".to_string()),
            foreground: RefCell::new("default".to_string()),
            cursor_colour: RefCell::new("none".to_string()),
            path: RefCell::new(String::new()),
            progress_state: Cell::new(0),
            progress_value: Cell::new(0),
            tab_stops: RefCell::new(None),
            columns: Cell::new(cols),
            alternate_on: Cell::new(false),
            alternate_saved_x: Cell::new(u32::MAX),
            alternate_saved_y: Cell::new(u32::MAX),
            child,
            output_waiters: RefCell::new(Vec::new()),
            output_timing: latency_enabled.then(|| {
                Rc::new(OutputTiming {
                    last_at: RefCell::new(None),
                })
            }),
            last_output_at: RefCell::new(None),
            bell_count: Cell::new(0),
            clipboard_events: RefCell::new(VecDeque::new()),
            passthrough: RefCell::new(VecDeque::new()),
            announced_title: RefCell::new(None),
            output_policy: PaneOutputPolicyCell::default(),
        }
    }

    /// The options the pane's own output is currently parsed against.
    fn output_policy(&self) -> PaneOutputPolicy {
        self.output_policy.load()
    }

    /// The pane's own key-output modes. The `extended-keys` option still has to
    /// be applied to `extended_request`, which is why this is not already a
    /// `PaneKeyModes`.
    pub(crate) fn key_state(&self) -> PaneKeyState {
        let modes = self.modes.get();
        PaneKeyState {
            cursor_keys: modes.cursor_keys,
            application_keypad: modes.application_keypad,
            bracketed_paste: modes.bracketed_paste,
            extended_request: modes.extended_keys_request,
        }
    }

    fn osc_state(&self) -> PaneOscState {
        let read = |slot: &RefCell<String>| slot.borrow().clone();
        PaneOscState {
            foreground: read(&self.foreground),
            cursor_colour: read(&self.cursor_colour),
            path: read(&self.path),
            progress_state: match self.progress_state.get() {
                1 => "normal",
                2 => "error",
                3 => "indeterminate",
                4 => "paused",
                _ => "hidden",
            },
            progress_value: self.progress_value.get(),
        }
    }

    /// Apply `edit` to the pane's tab stops, materializing the defaults first
    /// if the pane has not changed them yet.
    fn update_tab_stops(&self, edit: impl FnOnce(&mut BTreeSet<u16>)) {
        let columns = self.columns.get();
        {
            let mut stops = self.tab_stops.borrow_mut();
            edit(stops.get_or_insert_with(|| default_tab_stops(columns)));
        }
    }

    /// The pane's tab stops, as `#{pane_tabs}` lists them.
    fn tab_stops(&self) -> Vec<u16> {
        self.tab_stops
            .borrow()
            .as_ref()
            .map(|stops| stops.iter().copied().collect())
            .unwrap_or_else(|| {
                default_tab_stops(self.columns.get())
                    .into_iter()
                    .collect()
            })
    }

    /// The pane's scroll region, as `#{scroll_region_upper}`/`#{lower}` report
    /// it: the DECSTBM region when one is set, else the whole screen.
    fn scroll_region(&self) -> (u16, u16) {
        let region = self.redraw_detector.borrow().region();
        (region.top, region.bottom)
    }

    fn terminal_modes(&self) -> PaneTerminalModes {
        let modes = self.modes.get();
        PaneTerminalModes {
            insert: modes.insert_mode,
            origin: modes.origin_mode,
            wrap: modes.wrap_mode,
            cursor_visible: modes.cursor_visible,
            cursor_blinking: modes.cursor_blinking,
            keypad: modes.application_keypad,
            cursor_keys: modes.cursor_keys,
            cursor_shape: PaneCursorShape::from_parameter(self.cursor_shape.get()),
            synchronized_output: modes.synchronized_output,
            bracketed_paste: modes.bracketed_paste,
        }
    }

    pub(crate) fn mouse_modes(&self) -> PaneMouseModes {
        let modes = self.modes.get();
        PaneMouseModes {
            tracking: modes.mouse_tracking,
            utf8: modes.mouse_utf8,
            sgr: modes.mouse_sgr,
        }
    }

    fn note_clipboard_event(&self, event: PaneClipboardEvent) {
        {
            let mut events = self.clipboard_events.borrow_mut();
            // A runaway application must not grow this without bound; tmux
            // likewise answers only what it can keep up with.
            if events.len() < 16 {
                events.push_back(event);
            }
        }
    }

    fn note_passthrough(&self, event: PanePassthrough) {
        {
            let mut queued = self.passthrough.borrow_mut();
            // As with the clipboard queue, an application that outruns the
            // server loop loses the excess rather than growing the server.
            if queued.len() < 16 {
                queued.push_back(event);
            }
        }
    }

    /// The title the pane last set for itself.
    fn announced_title(&self) -> Option<String> {
        self.announced_title.borrow().clone()
    }

    pub(crate) fn take_passthrough(&self) -> Vec<PanePassthrough> {
        self.passthrough.borrow_mut().drain(..).collect()
    }

    pub(crate) fn take_clipboard_events(&self) -> Vec<PaneClipboardEvent> {
        self.clipboard_events.borrow_mut().drain(..).collect()
    }

    fn record_output(&self, bytes: &[u8], large_scroll: bool) {
        self.append_control_output(bytes);
        let mut detector = BellDetector::default();
        self.note_bells(bytes.iter().filter(|byte| detector.feed(**byte)).count() as u64);
        self.record_change(large_scroll);
    }

    fn note_bells(&self, count: u64) {
        if count != 0 {
            self.bell_count.set(self.bell_count.get().wrapping_add(count));
        }
    }

    fn append_control_output(&self, bytes: &[u8]) {
        if !bytes.is_empty() {
            {
                let mut output = self.control_output.borrow_mut();
                output.append(bytes);
            }
        }
    }

    fn record_change(&self, large_scroll: bool) {
        {
            let mut at = self.last_output_at.borrow_mut();
            *at = Some(Instant::now());
        }
        if let Some(timing) = self.output_timing.as_ref() {
            {
                let mut at = timing.last_at.borrow_mut();
                *at = Some(Instant::now());
            }
        }
        let revision = self.revision.get().wrapping_add(1);
        self.revision.set(revision);
        if large_scroll {
            self.large_scroll_revision
                .set(revision);
        }
        self.notify_output();
    }

    fn write_terminal(&self, terminal: &mut Terminal, bytes: &[u8]) -> bool {
        let actions = self.redraw_detector.borrow_mut().scan(bytes);
        if actions.is_empty() {
            terminal.write(bytes);
            return false;
        }

        let mut large_scroll = false;
        let mut start = 0;
        for action in actions {
            terminal.write(&bytes[start..action.byte_index]);
            if terminal
                .cursor_position()
                .ok()
                .is_some_and(|(_, y)| action.needs_large_redraw(y))
            {
                large_scroll = true;
            }
            terminal.write(&bytes[action.byte_index..=action.byte_index]);
            start = action.byte_index + 1;
        }
        terminal.write(&bytes[start..]);
        large_scroll
    }

    fn notify_output(&self) {
        let mut waiters = self.output_waiters.borrow_mut();
        waiters.retain(|waiter| {
            let Some(event) = waiter.upgrade() else {
                return false;
            };
            let _ = event.wakeup.wake();
            true
        });
    }

    fn register_output_event(&self, event: &Rc<OutputEvent>) -> io::Result<()> {
        self.output_waiters.borrow_mut().push(Rc::downgrade(event));
        Ok(())
    }

    pub(crate) fn subscribe_output(&self) -> io::Result<OutputSubscription> {
        // Start signalled so a subscriber performs one state scan and cannot
        // miss output or a terminal query queued just before registration.
        let event = Rc::new(OutputEvent {
            wakeup: CurrentPlatform::new_output_wakeup()?,
        });
        self.register_output_event(&event)?;
        Ok(OutputSubscription {
            event,
            output_timing: self.output_timing.as_ref().map(Rc::clone),
        })
    }

    pub(crate) fn contract_process(&self) -> (Option<u32>, bool) {
        match &self.child {
            Some(child) => (Some(child.pid), !child.alive.get()),
            None => (None, false),
        }
    }

    pub(crate) fn contract_revision(&self) -> u64 {
        self.revision.get()
    }

    pub(crate) fn large_scroll_revision(&self) -> u64 {
        self.large_scroll_revision.get()
    }

    pub(crate) fn alert_snapshot(&self) -> (u64, u64, Option<Instant>) {
        (
            self.revision.get(),
            self.bell_count.get(),
            *self.last_output_at.borrow(),
        )
    }

    pub(crate) fn control_output_end(&self) -> u64 {
        self.control_output.borrow().end
    }

    pub(crate) fn control_output_chunk(&self, offset: u64, limit: usize) -> (u64, u64, Vec<u8>) {
        self.control_output.borrow().chunk(offset, limit)
    }

    #[allow(dead_code)]
    pub(crate) fn contract_title(&self) -> io::Result<Option<String>> {
        Ok(self.announced_title())
    }

    /// Return a bounded tail of the terminal screen for the native observation
    /// boundary.
    #[allow(dead_code)]
    pub(crate) fn contract_terminal_tail(&self, max_rows: usize) -> io::Result<String> {
        let terminal = self.term.borrow_mut();
        Ok(trailing_lines(
            &terminal.dump_plain().map_err(ghostty_err)?,
            max_rows,
        ))
    }
}

impl PaneObservability for NativePaneObservation {
    fn process(&self) -> io::Result<PaneProcess> {
        Ok(match &self.child {
            Some(child) => PaneProcess {
                child_pid: Some(child.pid),
                exited: !child.alive.get(),
            },
            None => PaneProcess {
                child_pid: None,
                exited: false,
            },
        })
    }

    fn output_revision(&self) -> io::Result<u64> {
        Ok(self.revision.get())
    }

    fn screen(&self, source: ScreenSource, lines: usize) -> io::Result<ScreenTail> {
        let term = self.term.borrow_mut();
        let text = match source {
            ScreenSource::Recent => term.dump_plain().map_err(ghostty_err)?,
            ScreenSource::RecentUnwrapped => term.dump_plain_unwrapped().map_err(ghostty_err)?,
            ScreenSource::Visible => {
                // The plain dump is history-first; the viewport is the tail after
                // the scrollback rows. Drop history so only the on-screen rows
                // remain (see report.md).
                let dump = term.dump_plain().map_err(ghostty_err)?;
                let history = term.scrollback_rows().map_err(ghostty_err)?;
                drop_leading_lines(&dump, history)
            }
        };
        // Writers advance the revision while holding the same terminal lock,
        // so the formatted text, cursor state, and revision form one coherent
        // snapshot.
        let revision = self.revision.get();
        let cursor_visible = term.cursor_visible().map_err(ghostty_err)?;
        let cursor_shape = self.cursor_shape.get();
        Ok(ScreenTail {
            revision,
            text: trailing_lines(&text, lines),
            cursor_visible,
            cursor_shape,
        })
    }

    fn scrollback_rows(&self) -> io::Result<usize> {
        self.term
            .borrow_mut()
            .scrollback_rows()
            .map_err(ghostty_err)
    }

    fn title(&self) -> io::Result<Option<String>> {
        Ok(self.announced_title())
    }
}

impl Pane {
    /// A pane with a screen but no process. Useful as a lightweight session
    /// placeholder and for feeding synthetic bytes in tests.
    pub fn inert(cols: u16, rows: u16) -> io::Result<Pane> {
        let term = Terminal::new(cols, rows).map_err(ghostty_err)?;
        Ok(Pane {
            observation: Rc::new(NativePaneObservation::new(
                Rc::new(RefCell::new(term)),
                None,
                cols,
                rows,
            )),
            terminal_queries: Rc::new(RefCell::new(VecDeque::new())),
            child: None,
            pending_input: Rc::new(RefCell::new(VecDeque::new())),
            spawn_spec: None,
            pipe_output: Rc::new(RefCell::new(PanePipeOutbound::default())),
            pipe_output_active: Rc::new(Cell::new(false)),
            pipe: None,
            new_pipes: Vec::new(),
            event_io: None,
            runtime_id: NEXT_PANE_RUNTIME_ID.fetch_add(1, Ordering::Relaxed),
            cols,
            rows,
        })
    }

    pub(crate) fn spawn(
        argv: &[&str],
        cwd: Option<&Path>,
        cols: u16,
        rows: u16,
    ) -> io::Result<Pane> {
        assert!(!argv.is_empty(), "argv must have at least the program");

        let term = Terminal::new(cols, rows).map_err(ghostty_err)?;
        let term = Rc::new(RefCell::new(term));
        let terminal_queries = Rc::new(RefCell::new(VecDeque::new()));

        // Build the C argv *before* forking: allocation is not async-signal-safe,
        // so between fork and exec the child may only call execvp/_exit.
        let c_args: Vec<CString> = argv
            .iter()
            .map(|a| CString::new(*a))
            .collect::<Result<_, _>>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let mut c_ptrs: Vec<*const libc::c_char> = c_args.iter().map(|c| c.as_ptr()).collect();
        c_ptrs.push(ptr::null());
        let c_cwd = cwd
            .map(|path| CString::new(path.as_os_str().as_encoded_bytes()))
            .transpose()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        let ws = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        // SAFETY: the child branch below performs only post-fork-safe calls and
        // always terminates with exec or _exit.
        let (pid, master) = match unsafe { CurrentPlatform::fork_pty(ws)? } {
            ForkOutcome::Parent { pid, master } => (pid, master),
            ForkOutcome::Child => {
                // Child: exec the program. c_ptrs is NULL-terminated and its
                // strings outlive the call in the forked address space.
                unsafe {
                    if let Some(cwd) = &c_cwd {
                        if libc::chdir(cwd.as_ptr()) != 0 {
                            libc::_exit(127);
                        }
                    }
                    reset_child_signal_dispositions();
                    // Drop every inherited fd above stdio (the PTY fork wired the
                    // pty slave onto 0/1/2). Otherwise the exec'd shell inherits the
                    // listener/connection sockets *and* any client fd passed via
                    // SCM_RIGHTS — e.g. the stdout pipe of a `tmux new-window` command
                    // client — and holds them open for its whole life, so a client
                    // using command substitution (`$(tmux …)`) never sees EOF and
                    // hangs. Real tmux avoids this by marking those fds close-on-exec.
                    CurrentPlatform::close_fds_from(3);
                    clear_child_signal_mask();
                    libc::execvp(c_ptrs[0], c_ptrs.as_ptr());
                    libc::_exit(127); // exec failed
                }
            }
        };

        // Parent.
        // Make the master non-blocking so no server thread can ever block writing
        // to a child that has stopped reading its stdin. Both the reader thread's
        // dup and `Pane::input` share this open file description, so reads (which
        // now poll first) and writes (which drop on `EAGAIN`) are both guarded.
        set_nonblocking(master.as_raw_fd())?;
        let alive = Rc::new(Cell::new(true));
        let pending_input = Rc::new(RefCell::new(VecDeque::new()));
        let pipe_output = Rc::new(RefCell::new(PanePipeOutbound::default()));
        let pipe_output_active = Rc::new(Cell::new(false));
        let observation = Rc::new(NativePaneObservation::new(
            term,
            Some(ObservedChild {
                pid: pid as u32,
                alive: Rc::clone(&alive),
            }),
            cols,
            rows,
        ));
        let pane_io = PaneIo::new(
            &master,
            Rc::clone(&observation),
            Rc::clone(&terminal_queries),
            Rc::clone(&pending_input),
            Rc::clone(&pipe_output),
            Rc::clone(&pipe_output_active),
            Rc::clone(&alive),
        )?;
        let event_io = Some(pane_io);

        Ok(Pane {
            observation,
            terminal_queries,
            child: Some(Child {
                pid,
                master,
                alive,
                reaped: false,
                termination_requested: false,
                exit_code: None,
                death: None,
            }),
            pending_input,
            spawn_spec: Some(PaneSpawnSpec {
                argv: argv.iter().map(|arg| (*arg).to_string()).collect(),
                cwd: cwd.map(Path::to_path_buf),
            }),
            pipe_output,
            pipe_output_active,
            pipe: None,
            new_pipes: Vec::new(),
            event_io,
            runtime_id: NEXT_PANE_RUNTIME_ID.fetch_add(1, Ordering::Relaxed),
            cols,
            rows,
        })
    }

    pub(crate) fn spawn_spec(&self) -> Option<PaneSpawnSpec> {
        self.spawn_spec.clone()
    }

    pub(crate) fn spawn_from_spec(
        spec: &PaneSpawnSpec,
        cols: u16,
        rows: u16,
    ) -> io::Result<Pane> {
        let argv = spec.argv.iter().map(String::as_str).collect::<Vec<_>>();
        Self::spawn(&argv, spec.cwd.as_deref(), cols, rows)
    }

    pub(crate) fn runtime_id(&self) -> u64 {
        self.runtime_id
    }

    pub(crate) fn take_event_io(&mut self) -> Option<PaneIo> {
        self.event_io.take()
    }

    pub(crate) fn pipe_active(&self) -> bool {
        self.pipe
            .as_ref()
            .is_some_and(|pipe| pipe.alive.get())
    }

    /// The pid of the process the pane forked for its pty (`#{pane_pid}`).
    pub(crate) fn child_pid(&self) -> Option<pid_t> {
        self.child.as_ref().map(|child| child.pid)
    }

    /// The `pipe-pane` child's pid (`#{pane_pipe_pid}`), while the pipe is open.
    pub(crate) fn pipe_pid(&self) -> Option<u32> {
        self.pipe
            .as_ref()
            .filter(|pipe| pipe.alive.get())
            .map(|pipe| pipe.pid)
    }

    pub(crate) fn close_pipe(&mut self) {
        self.pipe_output_active.set(false);
        // The job owns the child's stdin; marking the buffer closed is what
        // makes it drop that end, and `PanePipe`'s own drop hangs up the child.
        if let Some(pipe) = self.pipe.take() {
            pipe.close_outbound();
        }
    }

    /// Pipe children opened since the last call, for the loop to drive.
    pub(crate) fn take_new_pipes(&mut self) -> Vec<PanePipeIo> {
        std::mem::take(&mut self.new_pipes)
    }

    /// Start a shell command connected to pane output (`output`) and/or pane
    /// input (`input`).
    ///
    /// Only the spawn happens here; the pipe I/O itself is handed to the server
    /// loop, which reads the child's stdout straight into the same pane-input
    /// queue the loop's own PTY writer drains — so pipe input and client
    /// keystrokes can no longer interleave mid-write.
    pub(crate) fn open_pipe(&mut self, command: &str, input: bool, output: bool) -> io::Result<()> {
        self.close_pipe();
        let mut process = Command::new("/bin/sh");
        process
            .arg("-c")
            .arg(command)
            .stdin(if output {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(if input { Stdio::piped() } else { Stdio::null() })
            .stderr(Stdio::null());
        unsafe {
            process.pre_exec(|| {
                reset_child_signal_dispositions();
                CurrentPlatform::close_fds_from(3);
                clear_child_signal_mask();
                Ok(())
            });
        }
        let mut process = process.spawn()?;
        let pid = process.id();
        let alive = Rc::new(Cell::new(true));

        let stdin = if output {
            let stdin = process
                .stdin
                .take()
                .ok_or_else(|| io::Error::other("pipe child has no stdin"))?;
            set_nonblocking(stdin.as_raw_fd())?;
            {
                let mut outbound = self.pipe_output.borrow_mut();
                *outbound = PanePipeOutbound::default();
            }
            self.pipe_output_active.set(true);
            Some(stdin)
        } else {
            None
        };

        let stdout = if input {
            let stdout = process
                .stdout
                .take()
                .ok_or_else(|| io::Error::other("pipe child has no stdout"))?;
            set_nonblocking(stdout.as_raw_fd())?;
            Some(stdout)
        } else {
            None
        };

        let master = match self.child.as_ref() {
            Some(child) => Some(child.master.as_fd().try_clone_to_owned()?),
            None if stdout.is_some() => {
                return Err(io::Error::other("pane has no child"));
            }
            None => None,
        };

        self.new_pipes.push(PanePipeIo {
            child: process,
            stdin,
            stdout,
            master,
            pending_input: Rc::clone(&self.pending_input),
            outbound: Rc::clone(&self.pipe_output),
            alive: Rc::clone(&alive),
        });
        self.pipe = Some(PanePipe {
            pid,
            alive,
            outbound: Rc::clone(&self.pipe_output),
        });
        Ok(())
    }

    /// Feed synthetic bytes directly into the screen (bypassing any pty). Used
    /// for inert panes and tests.
    pub fn feed(&self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        {
            let mut t = self.observation.term.borrow_mut();
            let large_scroll = self.observation.write_terminal(&mut t, bytes);
            let mut detector = CursorShapeDetector::default();
            for &byte in bytes {
                if let Some(shape) = detector.feed_byte(byte) {
                    self.observation
                        .cursor_shape
                        .set(shape);
                }
            }
            self.observation.record_output(bytes, large_scroll);
        }
    }

    /// Return the stable read-only handle associated with this pane.
    pub(crate) fn observation(&self) -> Rc<dyn PaneObservability> {
        Rc::clone(&self.observation) as Rc<dyn PaneObservability>
    }

    /// Concrete shared state used by the crate-private native observation
    /// handle. Public consumers continue to use `PaneObservability`.
    pub(crate) fn observation_state(&self) -> Rc<NativePaneObservation> {
        Rc::clone(&self.observation)
    }


    #[cfg(test)]
    pub(crate) fn subscribe_output(&self) -> io::Result<OutputSubscription> {
        self.observation.subscribe_output()
    }

    /// Send input bytes (keystrokes) to the child via the pty master. No-op for
    /// an inert pane.
    ///
    /// This never blocks, even when the child has stopped reading its stdin: the
    /// master is non-blocking, so bytes that the child cannot yet accept are held
    /// in the bounded `pending_input` buffer and flushed by the loop when the
    /// master reports writable. This is what keeps a stalled full-screen app from
    /// wedging the server lock held by the caller: `forward_input` holds the
    /// state mutex, and a blocking write here would hang every command.
    pub fn input(&self, bytes: &[u8]) -> io::Result<()> {
        self.input_with_stats(bytes).map(|_| ())
    }

    pub(crate) fn encode_mouse(&self, event: ghostty_sys::MouseEvent) -> io::Result<Vec<u8>> {
        self.observation
            .term
            .borrow_mut()
            .encode_mouse(event)
            .map_err(ghostty_err)
    }

    /// Reset the emulated terminal state without sending bytes to the child.
    pub(crate) fn reset_terminal(&self) -> io::Result<()> {
        let mut terminal = self.observation.term.borrow_mut();
        self.observation.write_terminal(&mut terminal, b"\x1bc");
        self.observation.record_change(false);
        Ok(())
    }

    /// Instrumented form of [`Self::input`] used by the attach latency probe.
    /// The ordinary public method deliberately retains its existing signature.
    pub(crate) fn input_with_stats(&self, bytes: &[u8]) -> io::Result<PaneInputStats> {
        let Some(child) = &self.child else {
            return Ok(PaneInputStats::default());
        };
        let stats = enqueue_pane_input(child.master.as_raw_fd(), &self.pending_input, bytes);
        if stats.queued != 0 {
        }
        Ok(stats)
    }

    /// The pane's live working directory (`#{pane_current_path}`), or `None` for
    /// an inert pane or one whose child cwd can't be read (e.g. it has exited).
    ///
    /// This mirrors real tmux's `osdep_get_cwd`: read the cwd of the pane's
    /// *foreground* process group — the group `tcgetpgrp` reports for the pty
    /// master. Reading it live (rather than caching the spawn directory) is what
    /// makes `#{pane_current_path}` follow a shell as it `cd`s, which is the
    /// behavior real tmux exposes.
    pub fn current_path(&self) -> Option<String> {
        self.process_probe()?.current_path()
    }

    /// Capture what a format callback needs to resolve
    /// `#{pane_current_path}` / `#{pane_current_command}` later without
    /// touching the pane again: the pty's foreground and session pids (two
    /// ioctls, read now) and the in-memory spawn command fallback. The
    /// expensive part — the `/proc` (or `libproc`) walk — is deferred to the
    /// probe's accessors, so a format that never names these variables never
    /// pays for it. `None` for a pane with no live child.
    pub(crate) fn process_probe(&self) -> Option<PaneProcessProbe> {
        let child = self.child.as_ref()?;
        let fd = child.master.as_raw_fd();
        // SAFETY: querying the foreground group / session of an owned pty
        // master fd.
        let foreground = unsafe { libc::tcgetpgrp(fd) };
        let session_leader = unsafe { libc::tcgetsid(fd) };
        Some(PaneProcessProbe {
            foreground: (foreground > 0).then_some(foreground),
            session_leader: (session_leader > 0).then_some(session_leader),
            fallback_command: self
                .spawn_spec
                .as_ref()
                .map(|spec| stringify_argv(&spec.argv)),
        })
    }

    /// The program occupying the pane's foreground process group.
    ///
    /// This mirrors tmux's `format_cb_current_command`, which tries the
    /// foreground group's `argv[0]` and then falls back to the pane's own
    /// command line. The fallback is what answers for a *leaderless* group: a
    /// process group id is only a pid while the group's leader lives, so a
    /// pipeline whose first member exits ahead of the others leaves the
    /// terminal owned by a group that names no process at all.
    pub fn current_command(&self) -> Option<String> {
        self.process_probe()?.current_command()
    }

    /// Drain terminal queries emitted by the child since the previous call.
    ///
    /// These bytes are written to the attached client's terminal. Its response
    /// then comes back through the normal client-input path and reaches
    /// [`Pane::input`], completing the same exchange real tmux provides.
    pub fn take_terminal_queries(&self) -> Vec<Vec<u8>> {
        self.terminal_queries.borrow_mut().drain(..).collect()
    }

    /// Publish the options the pane's output is parsed against. The server
    /// re-pushes these whenever they can have changed, since the parse itself
    /// has no view of the option tables.
    pub(crate) fn set_output_policy(&self, policy: PaneOutputPolicy) {
        self.observation.output_policy.store(policy);
    }

    /// The pane's current column count (`#{pane_width}`).
    pub fn cols(&self) -> u16 {
        self.cols
    }

    /// The pane's current row count (`#{pane_height}`).
    pub fn rows(&self) -> u16 {
        self.rows
    }

    /// Resize the screen and, if live, the pty (so the child gets SIGWINCH).
    pub fn resize(&mut self, cols: u16, rows: u16) -> io::Result<()> {
        self.cols = cols;
        self.rows = rows;
        {
            let mut t = self.observation.term.borrow_mut();
            t.resize(cols, rows).map_err(ghostty_err)?;
            {
                let mut detector = self.observation.redraw_detector.borrow_mut();
                detector.resize(rows);
            }
        }
        // tmux's screen_resize lays the default tab stops out afresh, dropping
        // whatever the pane had set.
        self.observation
            .columns
            .set(cols);
        {
            let mut stops = self.observation.tab_stops.borrow_mut();
            *stops = None;
        }
        if let Some(child) = &self.child {
            let ws = libc::winsize {
                ws_row: rows,
                ws_col: cols,
                ws_xpixel: 0,
                ws_ypixel: 0,
            };
            // SAFETY: TIOCSWINSZ takes a *const winsize on a valid fd.
            let r = unsafe { libc::ioctl(child.master.as_raw_fd(), libc::TIOCSWINSZ, &ws) };
            if r < 0 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    /// The current screen as plain text.
    pub fn dump(&self) -> io::Result<String> {
        self.observation
            .term
            .borrow_mut()
            .dump_plain()
            .map_err(ghostty_err)
    }

    /// The visible screen as plain text, without scrollback — what tmux's
    /// `window_pane_search` walks.
    pub(crate) fn visible_screen(&self) -> io::Result<String> {
        Ok(self
            .observation
            .screen(ScreenSource::Visible, usize::from(self.rows))
            .map_err(|error| io::Error::other(error.to_string()))?
            .text)
    }

    pub(crate) fn cursor_position(&self) -> io::Result<(u16, u16)> {
        self.observation
            .term
            .borrow_mut()
            .cursor_position()
            .map_err(ghostty_err)
    }

    pub(crate) fn copy_snapshot(
        &self,
    ) -> io::Result<(ghostty_sys::GridSnapshot, Vec<u8>, (u16, u16))> {
        let terminal = self.observation.term.borrow_mut();
        let grid = terminal.grid_snapshot().map_err(ghostty_err)?;
        let vt = terminal.dump_vt().map_err(ghostty_err)?;
        let cursor = terminal.cursor_position().map_err(ghostty_err)?;
        Ok((grid, vt, cursor))
    }

    /// Row geometry of the grid without the per-cell snapshot walk.
    pub(crate) fn grid_dims(&self) -> io::Result<ghostty_sys::GridDims> {
        self.observation
            .term
            .borrow_mut()
            .grid_dims()
            .map_err(ghostty_err)
    }

    /// Snapshot only physical rows `[start, start + count)`; see
    /// [`ghostty_sys::Terminal::grid_snapshot_range`].
    pub(crate) fn grid_snapshot_range(
        &self,
        start: usize,
        count: usize,
    ) -> io::Result<ghostty_sys::GridSnapshot> {
        self.observation
            .term
            .borrow_mut()
            .grid_snapshot_range(start, count)
            .map_err(ghostty_err)
    }

    pub(crate) fn background_color(&self) -> String {
        self.observation.background.borrow().clone()
    }

    /// Latest title advertised by the child. Ghostty handles OSC titles; the
    /// screen/tmux `ESC k ... ST` form is consumed before reaching Ghostty, so
    /// recover that form from the bounded raw-output journal.
    pub(crate) fn title(&self) -> Option<String> {
        let legacy = latest_screen_title(
            self.observation
                .control_output
                .borrow()
                .bytes
                .iter()
                .copied(),
        );
        legacy.or_else(|| self.observation.announced_title())
    }

    /// The current screen as VT escape sequences, suitable for writing to a
    /// client tty. This is the compositor primitive: the pane's grid is
    /// formatted as VT and sent to the attached client's terminal.
    pub fn dump_vt(&self) -> io::Result<Vec<u8>> {
        self.observation
            .term
            .borrow_mut()
            .dump_vt()
            .map_err(ghostty_err)
    }

    pub(crate) fn dump_rows_vt(&self, start: usize, rows: usize) -> io::Result<Vec<u8>> {
        self.observation
            .term
            .borrow_mut()
            .dump_vt_rows(start, rows, self.cols)
            .map_err(ghostty_err)
    }

    /// One physical row as trimmed plain text, without formatting the rest of
    /// the grid.
    pub(crate) fn dump_plain_row(&self, row: usize) -> io::Result<String> {
        self.observation
            .term
            .borrow_mut()
            .dump_plain_rows(row, 1, self.cols)
            .map_err(ghostty_err)
    }

    /// Format only the rows visible at a copy-mode scroll offset. Returning
    /// the clamped offset lets the compositor decide whether the live cursor
    /// belongs in the selected viewport.
    pub fn dump_viewport_vt(
        &self,
        scroll_offset: usize,
        visible_rows: usize,
    ) -> io::Result<(Vec<u8>, usize)> {
        let terminal = self.observation.term.borrow_mut();
        let scrollback = terminal.scrollback_rows().map_err(ghostty_err)?;
        let scroll = scroll_offset.min(scrollback);
        let start = scrollback - scroll;
        // A client whose terminal is taller than this pane asks for rows the
        // grid does not have, and reaching past the last one used to fail the
        // whole dump — which left such a client with a frame-less, permanently
        // blank screen. Serve the rows that exist and let the compositor erase
        // the rest, so the window is drawn at the top as tmux draws it. tmux
        // pads the remainder with the pane-border fill rather than blanks.
        let available = scroll.saturating_add(usize::from(self.rows));
        let vt = terminal
            .dump_vt_rows(start, visible_rows.min(available), self.cols)
            .map_err(ghostty_err)?;
        Ok((vt, scroll))
    }

    /// How many scrollback (history) rows the grid holds above the visible
    /// viewport. Consumers that render only the on-screen rows (the compositor,
    /// `capture-pane -p`) skip this many leading rows of a dump.
    pub fn scrollback_rows(&self) -> io::Result<usize> {
        self.observation
            .term
            .borrow_mut()
            .scrollback_rows()
            .map_err(ghostty_err)
    }

    /// Clear scrollback while preserving the visible viewport.
    ///
    /// CSI 3 J is Ghostty's own erase-scrollback operation, so this keeps the
    /// chosen terminal engine authoritative instead of reconstructing its grid
    /// in hmux.
    pub fn clear_history(&self) -> io::Result<()> {
        let mut terminal = self.observation.term.borrow_mut();
        self.observation.write_terminal(&mut terminal, b"\x1b[3J");
        self.observation.record_change(false);
        Ok(())
    }

    /// Whether the pane's cursor is visible (DEC mode 25). The compositor
    /// mirrors this onto the client tty so a TUI that hides the cursor and
    /// paints its own doesn't leave the client's real cursor lit on top.
    pub fn cursor_visible(&self) -> io::Result<bool> {
        self.observation
            .term
            .borrow_mut()
            .cursor_visible()
            .map_err(ghostty_err)
    }

    /// Current DECSCUSR parameter (0/default, 1..=6 block/underline/bar with
    /// blinking encoded by odd values), for mirroring onto the attached tty.
    pub fn cursor_shape(&self) -> u8 {
        self.observation.cursor_shape.get()
    }

    pub(crate) fn bracketed_paste_enabled(&self) -> bool {
        self.observation.modes.get().bracketed_paste
    }

    /// The pane's DECCKM, DECKPAM and `modifyOtherKeys` state, which decide how
    /// a key reaching it is spelled.
    pub(crate) fn key_state(&self) -> PaneKeyState {
        self.observation.key_state()
    }

    /// Whether the pane asked to be told when focus moves (DECSET 1004).
    pub(crate) fn focus_reporting_enabled(&self) -> bool {
        self.observation.modes.get().focus_reporting
    }

    /// Whether the pane asked to be told when the theme changes (DECSET 2031).
    pub(crate) fn theme_updates_enabled(&self) -> bool {
        self.observation.modes.get().theme_updates
    }

    /// The pane program's DECSET mouse reporting state, which decides both
    /// which reports reach it and how the default bindings treat a click.
    pub(crate) fn mouse_modes(&self) -> PaneMouseModes {
        self.observation.mouse_modes()
    }

    /// The pane terminal modes behind `#{insert_flag}` and its neighbours.
    pub(crate) fn terminal_modes(&self) -> PaneTerminalModes {
        self.observation.terminal_modes()
    }

    /// The pane's DECSTBM scroll region, as `#{scroll_region_upper}` and
    /// `#{scroll_region_lower}` report it.
    pub(crate) fn scroll_region(&self) -> (u16, u16) {
        self.observation.scroll_region()
    }

    /// The pane's tab stops, as `#{pane_tabs}` lists them.
    pub(crate) fn tab_stops(&self) -> Vec<u16> {
        self.observation.tab_stops()
    }

    /// Whether the pane is on its alternate screen, and the cursor DECSET 1049
    /// saved on the way in — `u32::MAX` in each axis while none has been.
    pub(crate) fn alternate_screen(&self) -> (bool, u32, u32) {
        (
            self.observation.alternate_on.get(),
            self.observation.alternate_saved_x.get(),
            self.observation.alternate_saved_y.get(),
        )
    }

    /// The pane's OSC-set colours and path: `#{cursor_colour}`, `#{pane_fg}`
    /// and `#{pane_path}`.
    pub(crate) fn osc_state(&self) -> PaneOscState {
        self.observation.osc_state()
    }

    /// Take a pending DSR ?996 theme question, if the pane asked one.
    pub(crate) fn take_theme_query(&self) -> bool {
        self.observation.theme_query.replace(false)
    }

    pub fn size(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    pub fn is_live(&self) -> bool {
        self.child
            .as_ref()
            .is_some_and(|child| child.alive.get())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.child.is_none()
    }

    /// Whether this pane had a child and that child has closed its pty.
    ///
    /// Inert fixture panes are deliberately not considered exited. The native
    /// server uses this distinction to apply tmux's `remain-on-exit=off`
    /// lifecycle without deleting deterministic panes used by conformance
    /// fixtures.
    pub fn has_exited(&self) -> bool {
        self.child
            .as_ref()
            .is_some_and(|child| !child.alive.get())
    }

    /// Reap an exited child without blocking and return its shell-style status.
    pub(crate) fn try_wait(&mut self) -> Option<i32> {
        let child = self.child.as_mut()?;
        if child.reaped {
            return child.exit_code;
        }
        let mut status = 0;
        let waited = unsafe { libc::waitpid(child.pid, &mut status, libc::WNOHANG) };
        if waited != child.pid {
            return None;
        }
        child.reaped = true;
        let code = if libc::WIFEXITED(status) {
            child.death = Some(PaneDeath {
                status: Some(libc::WEXITSTATUS(status)),
                signal: None,
                at: SystemTime::now(),
            });
            libc::WEXITSTATUS(status)
        } else if libc::WIFSIGNALED(status) {
            child.death = Some(PaneDeath {
                status: None,
                signal: Some(libc::WTERMSIG(status)),
                at: SystemTime::now(),
            });
            128 + libc::WTERMSIG(status)
        } else {
            return None;
        };
        child.exit_code = Some(code);
        Some(code)
    }

    pub(crate) fn collect_exited_child(&mut self) -> bool {
        if !self.has_exited() {
            return false;
        }
        if self.try_wait().is_some() {
            return true;
        }
        let Some(child) = self.child.as_mut() else {
            return true;
        };
        if !child.termination_requested {
            unsafe {
                libc::kill(child.pid, libc::SIGKILL);
            }
            child.termination_requested = true;
        }
        child.reaped
    }

    /// How this pane's child ended, or `None` while it is still running or has
    /// not been waited for yet — tmux's `PANE_STATUSREADY`.
    pub(crate) fn death(&self) -> Option<PaneDeath> {
        self.child.as_ref().and_then(|child| child.death)
    }

    pub(crate) fn child_reaped(&self) -> bool {
        self.child.as_ref().is_none_or(|child| child.reaped)
    }

}

fn latest_screen_title(bytes: impl Iterator<Item = u8>) -> Option<String> {
    let bytes = bytes.collect::<Vec<_>>();
    let mut latest = None;
    let mut index = 0;
    while index + 1 < bytes.len() {
        if bytes[index] != 0x1b || bytes[index + 1] != b'k' {
            index += 1;
            continue;
        }
        let start = index + 2;
        let mut end = start;
        while end < bytes.len() {
            if bytes[end] == 0x07 {
                latest = String::from_utf8(bytes[start..end].to_vec()).ok();
                index = end + 1;
                break;
            }
            if end + 1 < bytes.len() && bytes[end] == 0x1b && bytes[end + 1] == b'\\' {
                latest = String::from_utf8(bytes[start..end].to_vec()).ok();
                index = end + 2;
                break;
            }
            end += 1;
        }
        if end == bytes.len() {
            break;
        }
    }
    latest
}

impl Drop for Child {
    fn drop(&mut self) {
        if self.reaped {
            return;
        }
        // Kill the child so its pty slave closes, ending the drain.
        // SAFETY: sending a signal to our own child pid.
        if !self.reaped {
            unsafe {
                libc::kill(self.pid, libc::SIGKILL);
            }
        }

        // Reap the child off the caller's turn.
        //
        // `waitpid(pid, 0)` waits for the signalled child to actually die —
        // usually instant, but a child stuck in an uninterruptible syscall
        // delays it for an unbounded time.
        //
        // Crucially, `Child::drop` runs inside `kill-window` / `kill-pane`, which
        // the attach loop invokes while it holds the server state. Blocking here
        // froze the whole compositor: answering the `confirm-before` `(y/n)`
        // prompt cleared it instantly (client-local state) but the post-kill
        // redraw was stuck behind this teardown, so pressing `y` appeared to lag
        // by a second or more — intermittently, depending on whether the pane's
        // process tree still held the pty. Handing the pid to the orphan list
        // lets the kill return at once so the compositor redraws immediately;
        // the child is still reaped (no zombie) on the `SIGCHLD` the kill itself
        // delivers.
        //
        // The `master` OwnedFd field is dropped as this `Child` drops, closing
        // the parent's handle.
        if !self.reaped {
            register_orphan(self.pid);
        }
    }
}

thread_local! {
    /// Children killed by a pane teardown, waiting to be reaped.
    ///
    /// `waitpid` on a signalled child can block for an unbounded time — long
    /// enough to stall the loop that asked for the kill — so the pid is parked
    /// here and collected without waiting on the next `SIGCHLD`.
    ///
    /// The panes and the reaping pass both belong to the server loop, so this
    /// list belongs to that thread rather than to the process.
    static ORPHANED_CHILDREN: RefCell<Vec<pid_t>> = const { RefCell::new(Vec::new()) };
}

fn register_orphan(pid: pid_t) {
    ORPHANED_CHILDREN.with_borrow_mut(|orphans| orphans.push(pid));
}

/// Reap every orphan that has exited, keeping the ones still running.
pub(crate) fn reap_orphans() {
    ORPHANED_CHILDREN.with_borrow_mut(reap_orphan_list);
}

fn reap_orphan_list(orphans: &mut Vec<pid_t>) {
    orphans.retain(|pid| {
        let mut status = 0;
        // SAFETY: reaping our own child pid. No other code waits on it, so
        // there is no competing consumer and PID reuse is not a hazard.
        let reaped = unsafe { libc::waitpid(*pid, &mut status, libc::WNOHANG) };
        // Still running (0) is the only reason to ask again; an error means the
        // pid is not ours to wait for any more.
        reaped == 0
    });
}

/// Upper bound on how much currently-readable pane output is applied in one grid
/// transition. Draining without waiting batches an already-queued burst like an
/// event loop, without adding a timer or delaying interactive echo.
const OUTPUT_COALESCE_MAX_BYTES: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PaneIoReadResult {
    pub(crate) continuation: bool,
    pub(crate) closed: bool,
}

/// Owned nonblocking PTY state, driven by the central reactor one readiness
/// turn at a time.
pub(crate) struct PaneIo {
    fd: OwnedFd,
    observation: Rc<NativePaneObservation>,
    terminal_queries: Rc<RefCell<VecDeque<Vec<u8>>>>,
    pending_input: Rc<RefCell<VecDeque<u8>>>,
    pipe_output: Rc<RefCell<PanePipeOutbound>>,
    pipe_output_active: Rc<Cell<bool>>,
    alive: Rc<Cell<bool>>,
    query_detector: BackgroundColorQueryDetector,
    dsr_detector: DeviceStatusReportQueryDetector,
    cursor_report_detector: CursorPositionReportQueryDetector,
    cursor_shape_detector: CursorShapeDetector,
    mode_query_detector: ModeQueryDetector,
    osc_detector: OscStateDetector,
    decrqss_detector: DecrqssDetector,
    tab_stop_detector: TabStopDetector,
    alternate_detector: AlternateScreenDetector,
    alternate_stripper: AlternateScreenStripper,
    title_filter: SetTitleFilter,
    passthrough_detector: PassthroughDetector,
    clipboard_detector: Osc52Detector,
    utf8_sanitizer: Utf8Sanitizer,
    title_stripper: ScreenTitleStripper,
    bell_detector: BellDetector,
    closed: bool,
}

impl PaneIo {
    pub(crate) fn new(
        master: &OwnedFd,
        observation: Rc<NativePaneObservation>,
        terminal_queries: Rc<RefCell<VecDeque<Vec<u8>>>>,
        pending_input: Rc<RefCell<VecDeque<u8>>>,
        pipe_output: Rc<RefCell<PanePipeOutbound>>,
        pipe_output_active: Rc<Cell<bool>>,
        alive: Rc<Cell<bool>>,
    ) -> io::Result<Self> {
        Ok(Self {
            fd: master.as_fd().try_clone_to_owned()?,
            observation,
            terminal_queries,
            pending_input,
            pipe_output,
            pipe_output_active,
            alive,
            query_detector: BackgroundColorQueryDetector::default(),
            dsr_detector: DeviceStatusReportQueryDetector::default(),
            cursor_report_detector: CursorPositionReportQueryDetector::default(),
            cursor_shape_detector: CursorShapeDetector::default(),
            mode_query_detector: ModeQueryDetector::default(),
            osc_detector: OscStateDetector::default(),
            decrqss_detector: DecrqssDetector::default(),
            tab_stop_detector: TabStopDetector::default(),
            alternate_detector: AlternateScreenDetector::default(),
            alternate_stripper: AlternateScreenStripper::default(),
            title_filter: SetTitleFilter::default(),
            passthrough_detector: PassthroughDetector::default(),
            clipboard_detector: Osc52Detector::default(),
            utf8_sanitizer: Utf8Sanitizer::default(),
            title_stripper: ScreenTitleStripper::default(),
            bell_detector: BellDetector::default(),
            closed: false,
        })
    }

    pub(crate) fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.fd.as_fd()
    }

    pub(crate) fn wants_write(&self) -> bool {
        !self.pending_input.borrow().is_empty()
    }

    pub(crate) fn drive_writable(&mut self) {
        if self.closed {
            return;
        }
        {
            let mut queued = self.pending_input.borrow_mut();
            flush_pane_input(self.fd.as_raw_fd(), &mut queued);
        }
    }

    pub(crate) fn drive_readable(&mut self) -> io::Result<PaneIoReadResult> {
        if self.closed {
            return Ok(PaneIoReadResult {
                closed: true,
                ..PaneIoReadResult::default()
            });
        }

        let mut buffer = [0u8; 4096];
        let mut pending = Vec::new();
        let mut reached_eof = false;
        while pending.len() < OUTPUT_COALESCE_MAX_BYTES {
            let read = unsafe {
                libc::read(
                    self.fd.as_raw_fd(),
                    buffer.as_mut_ptr() as *mut c_void,
                    buffer.len(),
                )
            };
            if read > 0 {
                pending.extend_from_slice(&buffer[..read as usize]);
                continue;
            }
            if read == 0 {
                reached_eof = true;
                break;
            }
            let error = io::Error::last_os_error();
            match error.kind() {
                io::ErrorKind::Interrupted => continue,
                io::ErrorKind::WouldBlock => break,
                _ => {
                    reached_eof = true;
                    break;
                }
            }
        }

        let continuation = pending_capacity_reached(&pending);
        if !pending.is_empty() {
            self.process_output(pending);
        }
        if reached_eof {
            self.close();
        }
        Ok(PaneIoReadResult {
            continuation: !self.closed && continuation,
            closed: self.closed,
        })
    }

    fn process_output(&mut self, pending: Vec<u8>) {
        if self.pipe_output_active.get() {
            let mut outbound = self.pipe_output.borrow_mut();
            if outbound.closed {
                drop(outbound);
                self.pipe_output_active.set(false);
            } else {
                outbound.push(&pending);
            }
        }

        self.observation.append_control_output(&pending);
        let policy = self.observation.output_policy();
        for (index, colour) in policy.palette.iter().enumerate().take(256) {
            self.osc_detector.option_palette[index] = *colour;
        }
        let sanitized = self.utf8_sanitizer.filter(&pending);
        let filtered = self.title_stripper.filter(&sanitized);
        let filtered = self
            .alternate_stripper
            .filter(&filtered, policy.alternate_screen);
        let filtered = self.title_filter.filter(
            &filtered,
            policy.allow_set_title,
            input_buffer_capacity(policy.input_buffer_size),
        );
        if let Some(title) = self.title_filter.accepted.pop() {
            self.title_filter.accepted.clear();
            {
                let mut announced = self.observation.announced_title.borrow_mut();
                *announced = Some(title);
            }
        }
        let bytes = &filtered[..];
        self.observation.note_bells(
            bytes
                .iter()
                .filter(|byte| self.bell_detector.feed(**byte))
                .count() as u64,
        );
        let mut queries = Vec::new();
        let mut cursor_events: Vec<(usize, PaneCursorEvent)> = Vec::new();
        let mut mode_replies = Vec::new();
        for (index, &byte) in bytes.iter().enumerate() {
            if self.query_detector.feed_byte(byte) {
                queries.push(BACKGROUND_COLOR_QUERY);
            }
            if self.dsr_detector.feed_byte(byte) {
                queries.push(DEVICE_STATUS_REPORT_QUERY);
            }
            if self.cursor_report_detector.feed_byte(byte) {
                cursor_events.push((index + 1, PaneCursorEvent::PositionReport));
            }
            if let Some(event) = self.tab_stop_detector.feed_byte(byte) {
                cursor_events.push((index + 1, event));
            }
            if let Some(change) = self.alternate_detector.feed_byte(byte) {
                self.observation.alternate_on.set(!matches!(change, AlternateScreenChange::Leave));
                if let AlternateScreenChange::EnterSavingCursor { sequence_len } = change {
                    // The save has to happen before the sequence runs, since
                    // switching screens is what moves the cursor away. A
                    // sequence split across two reads saturates to 0, which is
                    // still before it — nothing after its ESC has been applied.
                    cursor_events.push((
                        (index + 1).saturating_sub(sequence_len),
                        PaneCursorEvent::SaveCursor,
                    ));
                }
            }
            if let Some(shape) = self.cursor_shape_detector.feed_byte(byte) {
                self.observation
                    .cursor_shape
                    .set(shape);
                // tmux's screen_set_cursor_style: every style but the default
                // one also decides whether the cursor blinks, odd styles
                // blinking and even ones steady.
                if shape != 0 {
                    self.mode_query_detector.modes.cursor_blinking = !shape.is_multiple_of(2);
                }
            }
            if let Some(reply) = self.mode_query_detector.feed_byte(byte) {
                mode_replies.push(reply);
            }
            if let Some(request) = self.decrqss_detector.feed_byte(byte) {
                mode_replies.push(decrqss_reply(
                    &request,
                    PaneCursorShape::from_parameter(
                        self.observation.cursor_shape.get(),
                    ),
                    self.mode_query_detector.modes.cursor_blinking,
                ));
            }
            if let Some(update) = self.osc_detector.feed_byte(byte) {
                let slot_value = match update {
                    PaneOscUpdate::Background(color) => Some((&self.observation.background, color)),
                    PaneOscUpdate::Foreground(color) => Some((&self.observation.foreground, color)),
                    PaneOscUpdate::CursorColour(color) => {
                        Some((&self.observation.cursor_colour, color))
                    }
                    PaneOscUpdate::Path(path) => Some((&self.observation.path, path)),
                    PaneOscUpdate::Reply(reply) => {
                        mode_replies.push(reply);
                        None
                    }
                    PaneOscUpdate::ProgressBar { state, progress } => {
                        self.observation
                            .progress_state
                            .set(state);
                        if let Some(progress) = progress {
                            self.observation
                                .progress_value
                                .set(progress);
                        }
                        None
                    }
                };
                if let Some((slot, value)) = slot_value {
                    {
                        let mut current = slot.borrow_mut();
                        *current = value;
                    }
                }
            }
            if let Some(event) = self.clipboard_detector.feed_byte(byte) {
                self.observation.note_clipboard_event(event);
            }
            // tmux reads `allow-passthrough` when the string terminator
            // arrives and drops the payload where it is off.
            if let Some(data) = self.passthrough_detector.feed_byte(byte) {
                match policy.passthrough {
                    PassthroughPolicy::Off => {}
                    reach => self.observation.note_passthrough(PanePassthrough {
                        data,
                        invisible_panes: reach == PassthroughPolicy::Always,
                    }),
                }
            }
        }
        self.observation.modes.set(self.mode_query_detector.modes);
        if std::mem::take(&mut self.mode_query_detector.theme_query) {
            self.observation.theme_query.set(true);
        }
        if !queries.is_empty() {
            {
                let mut queued = self.terminal_queries.borrow_mut();
                for query in queries {
                    if queued.len() == 16 {
                        break;
                    }
                    queued.push_back(query.to_vec());
                }
            }
        }

        let mut cursor_replies = Vec::new();
        {
            let mut terminal = self.observation.term.borrow_mut();
            let mut segment_start = 0usize;
            let mut large_scroll = false;
            // A save-cursor event splits ahead of its own sequence, so the list
            // is not necessarily in stream order; sorting keeps every split
            // point ascending, which is what the segment walk below assumes.
            cursor_events.sort_by_key(|(split_at, _)| *split_at);
            for (split_at, event) in cursor_events {
                large_scroll |= self
                    .observation
                    .write_terminal(&mut terminal, &bytes[segment_start..split_at]);
                // The cursor now reflects exactly the bytes this event needs to
                // have been applied, and none of the ones it does not.
                match event {
                    PaneCursorEvent::PositionReport => {
                        if let Some(response) = cursor_position_report(&terminal) {
                            cursor_replies.push(response);
                        }
                    }
                    PaneCursorEvent::SaveCursor => {
                        if let Ok((x, y)) = terminal.cursor_position() {
                            self.observation
                                .alternate_saved_x
                                .set(u32::from(x));
                            self.observation
                                .alternate_saved_y
                                .set(u32::from(y));
                        }
                    }
                    PaneCursorEvent::SetTabStop => {
                        if let Ok((x, _)) = terminal.cursor_position() {
                            self.observation.update_tab_stops(|stops| {
                                stops.insert(x);
                            });
                        }
                    }
                    PaneCursorEvent::ClearTabStop => {
                        if let Ok((x, _)) = terminal.cursor_position() {
                            self.observation.update_tab_stops(|stops| {
                                stops.remove(&x);
                            });
                        }
                    }
                    PaneCursorEvent::ClearAllTabStops => {
                        self.observation.update_tab_stops(BTreeSet::clear);
                    }
                }
                segment_start = split_at;
            }
            if segment_start < bytes.len() {
                large_scroll |= self
                    .observation
                    .write_terminal(&mut terminal, &bytes[segment_start..]);
            }
            self.observation.record_change(large_scroll);
        }
        for reply in cursor_replies {
            enqueue_pane_input(self.fd.as_raw_fd(), &self.pending_input, &reply);
        }
        for reply in mode_replies {
            enqueue_pane_input(self.fd.as_raw_fd(), &self.pending_input, &reply);
        }
    }

    fn close(&mut self) {
        if self.closed {
            return;
        }
        self.closed = true;
        self.alive.set(false);
        self.observation.notify_output();
    }
}

impl Drop for PaneIo {
    fn drop(&mut self) {
        self.close();
    }
}

fn pending_capacity_reached(pending: &[u8]) -> bool {
    pending.len() >= OUTPUT_COALESCE_MAX_BYTES
}

fn trailing_lines(text: &str, lines: usize) -> String {
    if lines == 0 || text.is_empty() {
        return String::new();
    }
    let rows: Vec<&str> = text.lines().collect();
    rows[rows.len().saturating_sub(lines)..].join("\n")
}

/// Drop the first `n` lines of `text` (used to strip scrollback history from a
/// history-first dump, leaving the visible viewport).
fn drop_leading_lines(text: &str, n: usize) -> String {
    if n == 0 {
        return text.to_string();
    }
    let rows: Vec<&str> = text.lines().collect();
    rows[n.min(rows.len())..].join("\n")
}

/// Neovim's default-background request. OSC allows either BEL or ST as its
/// terminator; queries relayed outward are normalized to ST.
const BACKGROUND_COLOR_QUERY_PREFIX: &[u8] = b"\x1b]11;?";
const BACKGROUND_COLOR_QUERY: &[u8] = b"\x1b]11;?\x1b\\";
const DEVICE_STATUS_REPORT_QUERY: &[u8] = b"\x1b[5n";

#[derive(Default)]
struct Utf8Sanitizer {
    pending: Vec<u8>,
}

impl Utf8Sanitizer {
    fn filter(&mut self, input: &[u8]) -> Vec<u8> {
        self.pending.extend_from_slice(input);
        let mut out = Vec::with_capacity(self.pending.len());
        let mut index = 0usize;
        while index < self.pending.len() {
            let first = self.pending[index];
            if first < 0x80 {
                // DEL is a control character in tmux, not a printable glyph.
                if first != 0x7f {
                    out.push(first);
                }
                index += 1;
                continue;
            }
            let width = match first {
                0xc2..=0xdf => 2,
                0xe0..=0xef => 3,
                0xf0..=0xf4 => 4,
                _ => {
                    out.extend_from_slice("\u{fffd}".as_bytes());
                    index += 1;
                    continue;
                }
            };
            let available = self.pending.len() - index;
            let continuation_count = self.pending[index + 1..]
                .iter()
                .take_while(|byte| (0x80..=0xbf).contains(*byte))
                .take(width - 1)
                .count();
            if available < width && continuation_count == available - 1 {
                break;
            }
            let consumed = 1 + continuation_count;
            if consumed == width && std::str::from_utf8(&self.pending[index..index + width]).is_ok()
            {
                out.extend_from_slice(&self.pending[index..index + width]);
            } else {
                // Collapse one malformed legacy sequence to the single
                // replacement cell tmux records, instead of one cell per byte.
                out.extend_from_slice("\u{fffd}".as_bytes());
            }
            index += consumed;
        }
        self.pending.drain(..index);
        out
    }
}

/// Streaming detector for OSC 11 queries. PTY reads can split an escape
/// sequence at any byte, so matching each read independently is insufficient.
#[derive(Default)]
struct BackgroundColorQueryDetector {
    /// Bytes matched in `BACKGROUND_COLOR_QUERY_PREFIX`, followed by one extra
    /// state when an ESC terminator has begun.
    matched: usize,
}

/// Collects the OSC sequences whose effect tmux publishes as a format variable
/// rather than as grid content: the pane's colours and its reported path.
struct OscStateDetector {
    sequence: Vec<u8>,
    in_osc: bool,
    escaped: bool,
    /// The pane's `OSC 4` palette, as packed `0xrrggbb`. tmux keeps one per
    /// pane so a query is answered from what that pane set, not the client's.
    palette: Box<[Option<u32>; 256]>,
    /// The palette `pane-colours` seeds, which a query falls back to — tmux's
    /// `colour_palette_from_option`. Pushed by the server, since the option is
    /// out of reach where a pane's bytes are parsed.
    option_palette: Box<[Option<u32>; 256]>,
}

impl Default for OscStateDetector {
    fn default() -> Self {
        Self {
            sequence: Vec::new(),
            in_osc: false,
            escaped: false,
            palette: Box::new([None; 256]),
            option_palette: Box::new([None; 256]),
        }
    }
}

/// One OSC sequence that changed a pane's reported state.
#[derive(Clone, Debug, Eq, PartialEq)]
enum PaneOscUpdate {
    /// OSC 11 / 111: `#{pane_bg}`.
    Background(String),
    /// OSC 10 / 110: `#{pane_fg}`.
    Foreground(String),
    /// OSC 12 / 112: `#{cursor_colour}`.
    CursorColour(String),
    /// OSC 7: `#{pane_path}`, kept verbatim as tmux keeps it.
    Path(String),
    /// Bytes to write back to the querying pane.
    Reply(Vec<u8>),
    /// OSC 9;4: `#{pane_pb_state}` and `#{pane_pb_progress}`. The progress is
    /// absent when the report named only a state, which leaves the old value.
    ProgressBar { state: u8, progress: Option<u8> },
}

impl OscStateDetector {
    fn feed_byte(&mut self, byte: u8) -> Option<PaneOscUpdate> {
        if !self.in_osc {
            self.sequence.push(byte);
            if self.sequence.ends_with(b"\x1b]") {
                self.sequence.clear();
                self.in_osc = true;
            } else if self.sequence.len() > 2 {
                self.sequence.remove(0);
            }
            return None;
        }
        if self.escaped {
            self.escaped = false;
            if byte == b'\\' {
                return self.finish(true);
            }
            self.sequence.push(0x1b);
        }
        match byte {
            0x07 => self.finish(false),
            0x9c => self.finish(true),
            0x1b => {
                self.escaped = true;
                None
            }
            _ => {
                if self.sequence.len() < 256 {
                    self.sequence.push(byte);
                }
                None
            }
        }
    }

    fn finish(&mut self, string_terminator: bool) -> Option<PaneOscUpdate> {
        self.in_osc = false;
        self.escaped = false;
        let sequence = std::mem::take(&mut self.sequence);
        let text = std::str::from_utf8(&sequence).ok()?;
        // The reset forms. tmux spells an unset cursor colour `none` but an
        // unset foreground or background `default`.
        match text {
            "110" => return Some(PaneOscUpdate::Foreground("default".to_string())),
            "111" => return Some(PaneOscUpdate::Background("default".to_string())),
            "112" => return Some(PaneOscUpdate::CursorColour("none".to_string())),
            // OSC 104 with no index clears the whole palette.
            "104" => {
                *self.palette = [None; 256];
                return None;
            }
            _ => {}
        }
        let (number, payload) = text.split_once(';')?;
        // OSC 7 carries a URL, not a colour, and tmux stores it unparsed.
        if number == "7" {
            return Some(PaneOscUpdate::Path(payload.to_string()));
        }
        if number == "4" {
            return self.palette_request(payload, string_terminator);
        }
        if number == "9" {
            return progress_bar_report(payload);
        }
        // OSC 104 with indices clears just those entries.
        if number == "104" {
            for index in payload.split(';') {
                match index.parse::<u8>() {
                    Ok(index) => self.palette[usize::from(index)] = None,
                    // tmux stops at the first index it cannot read.
                    Err(_) => break,
                }
            }
            return None;
        }
        // A `?` is the application asking rather than setting.
        if payload == "?" {
            return None;
        }
        let colour = parse_background_color(payload)?;
        match number {
            "10" => Some(PaneOscUpdate::Foreground(colour)),
            "11" => Some(PaneOscUpdate::Background(colour)),
            "12" => Some(PaneOscUpdate::CursorColour(colour)),
            _ => None,
        }
    }

    /// Apply one `OSC 4` body, which is a run of `index ; value` pairs.
    ///
    /// A `?` value asks for the entry back. tmux answers from its own palette
    /// when the entry has been set and otherwise forwards the question to the
    /// client's terminal; with nothing stored here the question is dropped,
    /// which is what keeps an unset entry silent.
    fn palette_request(&mut self, body: &str, string_terminator: bool) -> Option<PaneOscUpdate> {
        let mut reply = Vec::new();
        let mut rest = body;
        while !rest.is_empty() {
            let (index, tail) = rest.split_once(';')?;
            // tmux stops at the first unparseable index rather than skipping it.
            let index = index.parse::<u8>().ok()?;
            let (value, tail) = match tail.split_once(';') {
                Some((value, tail)) => (value, tail),
                None => (tail, ""),
            };
            if value == "?" {
                if let Some(colour) = self.palette[usize::from(index)]
                    .or(self.option_palette[usize::from(index)])
                {
                    let (r, g, b) = (
                        (colour >> 16) as u8,
                        (colour >> 8) as u8,
                        colour as u8,
                    );
                    let end = if string_terminator { "\x1b\\" } else { "\x07" };
                    // tmux answers in 16-bit components, each byte doubled.
                    reply.extend_from_slice(
                        format!("\x1b]4;{index};rgb:{r:02x}{r:02x}/{g:02x}{g:02x}/{b:02x}{b:02x}{end}")
                            .as_bytes(),
                    );
                }
            } else if let Some(packed) = parse_packed_colour(value) {
                self.palette[usize::from(index)] = Some(packed);
            }
            rest = tail;
        }
        (!reply.is_empty()).then_some(PaneOscUpdate::Reply(reply))
    }
}

/// Parse an `OSC 9` body as tmux's `input_osc_9` does.
///
/// Only the `4` subcommand — the ConEmu progress report — means anything to
/// tmux. A report naming just a state leaves the previous progress in place,
/// which is why the value is optional.
fn progress_bar_report(body: &str) -> Option<PaneOscUpdate> {
    let rest = body.strip_prefix('4')?;
    // `9;4` and `9;4;` carry no state and are dropped rather than reset.
    let rest = rest.strip_prefix(';').filter(|rest| !rest.is_empty())?;
    let (state, rest) = rest.split_at_checked(1)?;
    let state = state.parse::<u8>().ok().filter(|state| *state <= 4)?;
    let Some(progress) = rest.strip_prefix(';').filter(|rest| !rest.is_empty()) else {
        // Anything other than a clean end here is malformed, not a bare state.
        return rest
            .is_empty()
            .then_some(PaneOscUpdate::ProgressBar {
                state,
                progress: None,
            });
    };
    let progress = progress.parse::<u8>().ok().filter(|value| *value <= 100)?;
    Some(PaneOscUpdate::ProgressBar {
        state,
        progress: Some(progress),
    })
}

/// tmux's `screen_reset_tabs`: a stop every eight columns, skipping column 0.
fn default_tab_stops(columns: u16) -> BTreeSet<u16> {
    (1..)
        .map(|multiple| multiple * 8)
        .take_while(|stop| *stop < columns)
        .collect()
}

/// An X11 colour payload as a packed `0xrrggbb`, for the palette store.
pub(crate) fn parse_packed_colour(value: &str) -> Option<u32> {
    let text = parse_background_color(value)?;
    u32::from_str_radix(text.strip_prefix('#')?, 16).ok()
}

/// Collects `OSC 52 ; selection ; payload` sequences out of a pane's output.
///
/// The grid parser consumes the sequence too, but hmux needs the payload at the
/// server layer to apply `set-clipboard`/`get-clipboard`, so it is recognised
/// here rather than read back out of the terminal.
#[derive(Default)]
struct Osc52Detector {
    prefix: Vec<u8>,
    body: Vec<u8>,
    in_osc: bool,
    escaped: bool,
}

impl Osc52Detector {
    fn feed_byte(&mut self, byte: u8) -> Option<PaneClipboardEvent> {
        if !self.in_osc {
            self.prefix.push(byte);
            if self.prefix.ends_with(b"\x1b]52;") {
                self.prefix.clear();
                self.body.clear();
                self.in_osc = true;
            } else if self.prefix.len() > 5 {
                self.prefix.remove(0);
            }
            return None;
        }
        if self.escaped {
            self.escaped = false;
            if byte == b'\\' {
                return self.finish(true);
            }
            self.body.push(0x1b);
        }
        match byte {
            0x07 | 0x9c => self.finish(false),
            0x1b => {
                self.escaped = true;
                None
            }
            _ => {
                // A payload longer than this is not a clipboard update worth
                // buffering; drop the sequence rather than grow without bound.
                if self.body.len() < 1024 * 1024 {
                    self.body.push(byte);
                    None
                } else {
                    self.in_osc = false;
                    self.body.clear();
                    None
                }
            }
        }
    }

    fn finish(&mut self, string_terminator: bool) -> Option<PaneClipboardEvent> {
        self.in_osc = false;
        self.escaped = false;
        let body = std::mem::take(&mut self.body);
        let text = String::from_utf8(body).ok()?;
        // A sequence with no `;` at all names no payload and is dropped.
        let (selection, payload) = text.split_once(';')?;
        if payload == "?" {
            return Some(PaneClipboardEvent::Query {
                selection: osc52_reply_selection(selection),
                string_terminator,
            });
        }
        // Empty or undecodable data is dropped, as tmux drops it.
        if payload.is_empty() {
            return None;
        }
        base64_decode_strict(payload).map(|data| PaneClipboardEvent::Set { data })
    }
}

/// The selection tmux echoes back in an OSC 52 reply: the first character it
/// recognises, or nothing when the request named none.
fn osc52_reply_selection(selection: &str) -> String {
    selection
        .chars()
        .find(|character| "cpqs01234567".contains(*character))
        .map(String::from)
        .unwrap_or_default()
}

/// Decode standard base64, rejecting anything that is not fully padded — which
/// is what makes tmux drop `aGk` where it accepts `aGk=`.
fn base64_decode_strict(text: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = text.as_bytes();
    if bytes.is_empty() || !bytes.len().is_multiple_of(4) {
        return None;
    }
    let padding = bytes.iter().rev().take_while(|byte| **byte == b'=').count();
    if padding > 2 {
        return None;
    }
    let data = &bytes[..bytes.len() - padding];
    let mut out = Vec::with_capacity(data.len() / 4 * 3);
    let (mut accumulator, mut bits) = (0u32, 0u32);
    for byte in data {
        let position = ALPHABET.iter().position(|candidate| candidate == byte)?;
        accumulator = (accumulator << 6) | position as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((accumulator >> bits) & 0xff) as u8);
        }
    }
    Some(out)
}

fn parse_background_color(value: &str) -> Option<String> {
    let value = value.trim();
    let rgb = if let Some(parts) = value.strip_prefix("rgb:") {
        let parts: Vec<_> = parts.split('/').collect();
        if parts.len() != 3 {
            return None;
        }
        (
            scale_hex(parts[0])?,
            scale_hex(parts[1])?,
            scale_hex(parts[2])?,
        )
    } else if let Some(parts) = value.strip_prefix("cmy:") {
        let parts: Vec<_> = parts.split('/').collect();
        if parts.len() != 3 {
            return None;
        }
        (
            ((1.0 - parts[0].parse::<f64>().ok()?) * 255.0) as u8,
            ((1.0 - parts[1].parse::<f64>().ok()?) * 255.0) as u8,
            ((1.0 - parts[2].parse::<f64>().ok()?) * 255.0) as u8,
        )
    } else if let Some(parts) = value.strip_prefix("cmyk:") {
        let parts: Vec<_> = parts.split('/').collect();
        if parts.len() != 4 {
            return None;
        }
        let k = parts[3].parse::<f64>().ok()?;
        (
            ((1.0 - parts[0].parse::<f64>().ok()?) * (1.0 - k) * 255.0) as u8,
            ((1.0 - parts[1].parse::<f64>().ok()?) * (1.0 - k) * 255.0) as u8,
            ((1.0 - parts[2].parse::<f64>().ok()?) * (1.0 - k) * 255.0) as u8,
        )
    } else if let Some(hex) = value.strip_prefix('#') {
        match hex.len() {
            6 => (
                u8::from_str_radix(&hex[0..2], 16).ok()?,
                u8::from_str_radix(&hex[2..4], 16).ok()?,
                u8::from_str_radix(&hex[4..6], 16).ok()?,
            ),
            12 => (
                scale_hex(&hex[0..4])?,
                scale_hex(&hex[4..8])?,
                scale_hex(&hex[8..12])?,
            ),
            _ => return None,
        }
    } else if value.contains(',') {
        let parts: Vec<_> = value.split(',').collect();
        if parts.len() != 3 {
            return None;
        }
        (
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        )
    } else {
        // Anything left is a name, which tmux resolves against X11's table
        // rather than the terminal palette — so `red` is #ff0000, not colour 1.
        let packed = super::x11_colour::colour_by_name(value)?;
        (
            (packed >> 16) as u8,
            (packed >> 8) as u8,
            packed as u8,
        )
    };
    Some(format!("#{:02x}{:02x}{:02x}", rgb.0, rgb.1, rgb.2))
}

fn scale_hex(component: &str) -> Option<u8> {
    if component.is_empty() || component.len() > 4 {
        return None;
    }
    let value = u32::from_str_radix(component, 16).ok()?;
    if component.len() == 4 {
        return Some((value >> 8) as u8);
    }
    let maximum = 16u32.pow(component.len() as u32) - 1;
    Some(((value * 255 + maximum / 2) / maximum) as u8)
}

impl BackgroundColorQueryDetector {
    fn feed_byte(&mut self, byte: u8) -> bool {
        let prefix_len = BACKGROUND_COLOR_QUERY_PREFIX.len();
        if self.matched < prefix_len {
            if byte == BACKGROUND_COLOR_QUERY_PREFIX[self.matched] {
                self.matched += 1;
            } else {
                self.matched = usize::from(byte == BACKGROUND_COLOR_QUERY_PREFIX[0]);
            }
            return false;
        }

        if self.matched == prefix_len {
            match byte {
                b'\x07' | b'\x9c' => {
                    self.matched = 0;
                    return true;
                }
                b'\x1b' => self.matched += 1,
                _ => self.matched = 0,
            }
            return false;
        }

        // `matched == prefix_len + 1`: ESC must be followed by `\` for ST.
        if byte == b'\\' {
            self.matched = 0;
            true
        } else {
            if byte != b'\x1b' {
                self.matched = 0;
            }
            false
        }
    }
}

/// Streaming detector for `CSI 5 n`, the DSR synchronization request Neovim
/// appends to terminal capability queries. The corresponding outer-terminal
/// response (`CSI 0 n`) tells Neovim that all preceding replies have arrived.
#[derive(Default)]
struct DeviceStatusReportQueryDetector {
    matched: usize,
}

impl DeviceStatusReportQueryDetector {
    fn feed_byte(&mut self, byte: u8) -> bool {
        if byte == DEVICE_STATUS_REPORT_QUERY[self.matched] {
            self.matched += 1;
            if self.matched == DEVICE_STATUS_REPORT_QUERY.len() {
                self.matched = 0;
                return true;
            }
        } else {
            self.matched = usize::from(byte == DEVICE_STATUS_REPORT_QUERY[0]);
        }
        false
    }
}

/// Something in a pane's output that can only be handled at an exact point in
/// the byte stream, because it reads the cursor the surrounding bytes move.
///
/// Each is paired with the number of bytes to write to the terminal first.
/// Most sit *after* their own sequence; [`PaneCursorEvent::SaveCursor`] sits
/// before it, since the sequence it belongs to moves the cursor it must save.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PaneCursorEvent {
    /// DSR 6n: report where the cursor is.
    PositionReport,
    /// DECSET 1049: remember the cursor for the eventual return to the primary
    /// screen.
    SaveCursor,
    /// HTS: set a tab stop in the cursor's column.
    SetTabStop,
    /// TBC 0: clear the tab stop in the cursor's column.
    ClearTabStop,
    /// TBC 3: clear every tab stop. Listed here so it stays ordered with the
    /// others even though it does not read the cursor.
    ClearAllTabStops,
}

/// Recognizes the DEC modes that switch a pane to and from the alternate
/// screen, which is what `#{alternate_on}` reports.
#[derive(Default)]
struct AlternateScreenDetector {
    tail: VecDeque<u8>,
}

/// A switch between a pane's primary and alternate screens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AlternateScreenChange {
    /// DECSET 47 or 1047, which do not save the cursor.
    Enter,
    /// DECSET 1049, which saves the cursor before switching. Carries the
    /// sequence's own length so the save can be placed ahead of it.
    EnterSavingCursor { sequence_len: usize },
    /// DECRST 47, 1047 or 1049.
    Leave,
}

impl AlternateScreenDetector {
    fn feed_byte(&mut self, byte: u8) -> Option<AlternateScreenChange> {
        if self.tail.len() == 8 {
            self.tail.pop_front();
        }
        self.tail.push_back(byte);
        let tail: Vec<u8> = self.tail.iter().copied().collect();

        if tail.ends_with(b"\x1b[?1049h") {
            return Some(AlternateScreenChange::EnterSavingCursor { sequence_len: 8 });
        }
        if tail.ends_with(b"\x1b[?47h") || tail.ends_with(b"\x1b[?1047h") {
            return Some(AlternateScreenChange::Enter);
        }
        if tail.ends_with(b"\x1b[?47l")
            || tail.ends_with(b"\x1b[?1047l")
            || tail.ends_with(b"\x1b[?1049l")
        {
            return Some(AlternateScreenChange::Leave);
        }
        None
    }
}

/// The DEC modes that switch a pane between its primary and alternate screens.
const ALTERNATE_SCREEN_SWITCHES: [&[u8]; 6] = [
    b"\x1b[?47h",
    b"\x1b[?47l",
    b"\x1b[?1047h",
    b"\x1b[?1047l",
    b"\x1b[?1049h",
    b"\x1b[?1049l",
];

/// Drops those switches from a pane's output while `alternate-screen` is off.
///
/// tmux parses the sequence and then returns early from
/// `screen_write_alternateon`/`_alternateoff`, so the switch has no effect at
/// all — including on the cursor 1049 would have saved. Removing the bytes
/// before anything sees them is the same observable, and keeps the option out of
/// every detector downstream.
#[derive(Default)]
struct AlternateScreenStripper {
    /// Bytes held back because they are a prefix of one of the switches. A
    /// partial sequence at the end of a read is no more applied than it would be
    /// inside a terminal parser, so holding it until the rest arrives is safe.
    pending: Vec<u8>,
}

impl AlternateScreenStripper {
    fn filter(&mut self, input: &[u8], allowed: bool) -> Vec<u8> {
        // Whatever is held was never a complete switch, so re-enabling the
        // option releases it unchanged.
        if allowed {
            let mut out = std::mem::take(&mut self.pending);
            out.extend_from_slice(input);
            return out;
        }
        let mut out = Vec::with_capacity(self.pending.len() + input.len());
        for &byte in input {
            self.pending.push(byte);
            if ALTERNATE_SCREEN_SWITCHES
                .iter()
                .any(|switch| self.pending == *switch)
            {
                self.pending.clear();
                continue;
            }
            // Not a switch: give back the longest head that cannot start one,
            // which leaves any genuine prefix still pending.
            while !self.pending.is_empty()
                && !ALTERNATE_SCREEN_SWITCHES
                    .iter()
                    .any(|switch| switch.starts_with(&self.pending))
            {
                out.push(self.pending.remove(0));
            }
        }
        out
    }
}

/// Recognizes HTS (`ESC H`) and TBC (`CSI Ps g`), which together decide what
/// `#{pane_tabs}` reports.
#[derive(Default)]
struct TabStopDetector {
    state: TabStopState,
    parameter: u16,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum TabStopState {
    #[default]
    Ground,
    Esc,
    Csi,
}

impl TabStopDetector {
    fn feed_byte(&mut self, byte: u8) -> Option<PaneCursorEvent> {
        use TabStopState::{Csi, Esc, Ground};

        self.state = match (self.state, byte) {
            (Ground, b'\x1b') => Esc,
            (Ground, _) => Ground,
            (Esc, b'H') => {
                self.state = Ground;
                return Some(PaneCursorEvent::SetTabStop);
            }
            (Esc, b'[') => {
                self.parameter = 0;
                Csi
            }
            (Esc, b'\x1b') => Esc,
            (Esc, _) => Ground,
            (Csi, b'0'..=b'9') => {
                self.parameter = self
                    .parameter
                    .saturating_mul(10)
                    .saturating_add(u16::from(byte - b'0'));
                Csi
            }
            (Csi, b'g') => {
                self.state = Ground;
                // tmux honours only the clear-here and clear-all forms.
                return match self.parameter {
                    0 => Some(PaneCursorEvent::ClearTabStop),
                    3 => Some(PaneCursorEvent::ClearAllTabStops),
                    _ => None,
                };
            }
            (Csi, b'\x1b') => Esc,
            (Csi, _) => Ground,
        };
        None
    }
}

#[derive(Default)]
struct CursorPositionReportQueryDetector {
    state: CursorPositionReportState,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum CursorPositionReportState {
    #[default]
    Ground,
    Esc,
    Csi,
    SawSix,
}

/// Streaming recognizer for DECSCUSR (`CSI Ps SP q`). Applications such as
/// Neovim use 2 for a steady block and 6 for a steady bar. PTY reads may split
/// the sequence, so the state is retained across output bursts.
#[derive(Default)]
struct CursorShapeDetector {
    state: CursorShapeState,
    parameter: u8,
}

/// Track the DEC modes tmux answers locally and recognize DECRQM queries.
/// The reportable VT modes a pane's byte stream has set, in their own types.
/// The detector mutates them in place; the observation republishes the whole
/// snapshot at the end of each output batch.
#[derive(Clone, Copy)]
struct PaneModeSnapshot {
    cursor_visible: bool,
    /// DECSET 2004: the pane wants the paste markers.
    bracketed_paste: bool,
    /// DECSET 1004: the pane asked to be told when focus moves.
    focus_reporting: bool,
    /// DECSET 2031: the pane asked to be told when the theme changes.
    theme_updates: bool,
    /// The pane program's mouse reporting mode, if any. tmux keeps 1000/1002/
    /// 1003 mutually exclusive — each one clears the others — and tracks the
    /// two encoding modes independently.
    mouse_tracking: Option<MouseTrackingMode>,
    /// DECSET 1005: UTF-8 coordinate encoding.
    mouse_utf8: bool,
    /// DECSET 1006: SGR encoding.
    mouse_sgr: bool,
    /// IRM (`CSI 4 h`): typed cells shift the rest of the line right.
    insert_mode: bool,
    /// DECOM (DECSET 6): cursor addressing is relative to the scroll region.
    origin_mode: bool,
    /// DECAWM (DECSET 7): text wraps at the right margin. On by default.
    wrap_mode: bool,
    /// tmux's `MODE_CURSOR_BLINKING`, written both by DECSET/DECRST 12 and, as
    /// a side effect, by every DECSCUSR style except the default one.
    cursor_blinking: bool,
    /// DECCKM (DECSET 1): the pane wants the application cursor key forms.
    cursor_keys: bool,
    /// DECKPAM (`ESC =`): the pane wants the application keypad forms.
    application_keypad: bool,
    /// The `modifyOtherKeys` level the pane asked for with `CSI > 4 ; n m`.
    /// What it *gets* also depends on the `extended-keys` option, which is
    /// applied where the key is encoded rather than here.
    extended_keys_request: ExtendedKeys,
    /// DECSET 2026, tmux's `MODE_SYNC`: the pane asked for its output to be
    /// held back until it says the frame is done.
    synchronized_output: bool,
}

impl Default for PaneModeSnapshot {
    /// A terminal that has been sent none of these sequences: tmux starts a
    /// screen with the cursor shown and wrapping on.
    fn default() -> Self {
        Self {
            cursor_visible: true,
            bracketed_paste: false,
            focus_reporting: false,
            theme_updates: false,
            mouse_tracking: None,
            mouse_utf8: false,
            mouse_sgr: false,
            insert_mode: false,
            origin_mode: false,
            wrap_mode: true,
            cursor_blinking: false,
            cursor_keys: false,
            application_keypad: false,
            extended_keys_request: ExtendedKeys::Off,
            synchronized_output: false,
        }
    }
}

struct ModeQueryDetector {
    tail: VecDeque<u8>,
    /// Whether the pane asked which theme it is under (DSR ?996) and has
    /// not been answered yet.
    theme_query: bool,
    modes: PaneModeSnapshot,
}

/// A pane's own key-output modes, before the `extended-keys` option decides how
/// much of the extended request is honoured.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PaneKeyState {
    pub(crate) cursor_keys: bool,
    pub(crate) application_keypad: bool,
    /// DECSET 2004: whether the pane wants the paste markers.
    pub(crate) bracketed_paste: bool,
    pub(crate) extended_request: ExtendedKeys,
}

/// The pane terminal state tmux keeps in `screen->mode` and publishes as format
/// variables. Ghostty owns the emulation but does not expose these, so they are
/// tracked from the pane's byte stream beside it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PaneTerminalModes {
    /// IRM, as `#{insert_flag}`.
    pub(crate) insert: bool,
    /// DECOM, as `#{origin_flag}`.
    pub(crate) origin: bool,
    /// DECAWM, as `#{wrap_flag}`.
    pub(crate) wrap: bool,
    /// DECTCEM, as `#{cursor_flag}`.
    pub(crate) cursor_visible: bool,
    /// `#{cursor_blinking}`, set by DECSET 12 and by DECSCUSR alike.
    pub(crate) cursor_blinking: bool,
    /// DECKPAM, as `#{keypad_flag}`.
    pub(crate) keypad: bool,
    /// DECCKM, as `#{keypad_cursor_flag}`.
    pub(crate) cursor_keys: bool,
    /// The DECSCUSR style, as `#{cursor_shape}`.
    pub(crate) cursor_shape: PaneCursorShape,
    /// DECSET 2026, as `#{synchronized_output_flag}`.
    pub(crate) synchronized_output: bool,
    /// DECSET 2004, as `#{bracket_paste_flag}`.
    pub(crate) bracketed_paste: bool,
}

impl Default for PaneTerminalModes {
    /// A terminal that has been sent none of these sequences: tmux starts a
    /// screen with the cursor shown and wrapping on.
    fn default() -> Self {
        Self {
            insert: false,
            origin: false,
            wrap: true,
            cursor_visible: true,
            cursor_blinking: false,
            keypad: false,
            cursor_keys: false,
            cursor_shape: PaneCursorShape::Default,
            synchronized_output: false,
            bracketed_paste: false,
        }
    }
}

/// The pane state an OSC sequence set, as the formats report it. `#{pane_bg}`
/// is not here because it already has its own accessor.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PaneOscState {
    /// OSC 10, defaulting to `default`.
    pub(crate) foreground: String,
    /// OSC 12, defaulting to `none`.
    pub(crate) cursor_colour: String,
    /// OSC 7, defaulting to empty.
    pub(crate) path: String,
    /// OSC 9;4, as `#{pane_pb_state}` names it.
    pub(crate) progress_state: &'static str,
    /// The last progress percentage the pane reported, which survives a state
    /// change that does not carry one.
    pub(crate) progress_value: u8,
}

/// The cursor style a pane asked for with DECSCUSR.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum PaneCursorShape {
    #[default]
    Default,
    Block,
    Underline,
    Bar,
}

impl PaneCursorShape {
    /// tmux's `screen_set_cursor_style` mapping of a DECSCUSR parameter.
    fn from_parameter(parameter: u8) -> Self {
        match parameter {
            1 | 2 => Self::Block,
            3 | 4 => Self::Underline,
            5 | 6 => Self::Bar,
            _ => Self::Default,
        }
    }

    /// The name `#{cursor_shape}` reports.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Block => "block",
            Self::Underline => "underline",
            Self::Bar => "bar",
        }
    }
}

/// A pane's DECSET mouse state, as `#{mouse_*_flag}` reports it.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PaneMouseModes {
    pub(crate) tracking: Option<MouseTrackingMode>,
    pub(crate) utf8: bool,
    pub(crate) sgr: bool,
}

impl PaneMouseModes {
    /// tmux's `ALL_MOUSE_MODES`: the pane asked for mouse reports at all.
    pub(crate) fn any(self) -> bool {
        self.tracking.is_some()
    }
}

/// Which DECSET mouse-reporting mode a pane's program asked for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MouseTrackingMode {
    /// DECSET 1000: presses and releases only.
    Standard,
    /// DECSET 1002: adds motion while a button is held.
    Button,
    /// DECSET 1003: adds button-less motion.
    All,
}

impl Default for ModeQueryDetector {
    fn default() -> Self {
        Self {
            tail: VecDeque::with_capacity(16),
            theme_query: false,
            modes: PaneModeSnapshot::default(),
        }
    }
}

impl ModeQueryDetector {
    fn mouse_mode_status(&self, mode: MouseTrackingMode) -> u8 {
        if self.modes.mouse_tracking == Some(mode) {
            1
        } else {
            2
        }
    }

    fn feed_byte(&mut self, byte: u8) -> Option<Vec<u8>> {
        if self.tail.len() == 16 {
            self.tail.pop_front();
        }
        self.tail.push_back(byte);
        let tail: Vec<u8> = self.tail.iter().copied().collect();

        if tail.ends_with(b"\x1b[?2026h") {
            self.modes.synchronized_output = true;
        } else if tail.ends_with(b"\x1b[?2026l") {
            self.modes.synchronized_output = false;
        } else if tail.ends_with(b"\x1b[?25h") {
            self.modes.cursor_visible = true;
        } else if tail.ends_with(b"\x1b[?25l") {
            self.modes.cursor_visible = false;
        } else if tail.ends_with(b"\x1b[?2004h") {
            self.modes.bracketed_paste = true;
        } else if tail.ends_with(b"\x1b[?2004l") {
            self.modes.bracketed_paste = false;
        } else if tail.ends_with(b"\x1b[?1004h") {
            self.modes.focus_reporting = true;
        } else if tail.ends_with(b"\x1b[?1004l") {
            self.modes.focus_reporting = false;
        } else if tail.ends_with(b"\x1b[?2031h") {
            self.modes.theme_updates = true;
        } else if tail.ends_with(b"\x1b[?2031l") {
            self.modes.theme_updates = false;
        } else if tail.ends_with(b"\x1b[?1000h") {
            self.modes.mouse_tracking = Some(MouseTrackingMode::Standard);
        } else if tail.ends_with(b"\x1b[?1002h") {
            self.modes.mouse_tracking = Some(MouseTrackingMode::Button);
        } else if tail.ends_with(b"\x1b[?1003h") {
            self.modes.mouse_tracking = Some(MouseTrackingMode::All);
        } else if tail.ends_with(b"\x1b[?1000l")
            || tail.ends_with(b"\x1b[?1001l")
            || tail.ends_with(b"\x1b[?1002l")
            || tail.ends_with(b"\x1b[?1003l")
        {
            self.modes.mouse_tracking = None;
        } else if tail.ends_with(b"\x1b[?1005h") {
            self.modes.mouse_utf8 = true;
        } else if tail.ends_with(b"\x1b[?1005l") {
            self.modes.mouse_utf8 = false;
        } else if tail.ends_with(b"\x1b[?1006h") {
            self.modes.mouse_sgr = true;
        } else if tail.ends_with(b"\x1b[?1006l") {
            self.modes.mouse_sgr = false;
        } else if tail.ends_with(b"\x1b[4h") {
            self.modes.insert_mode = true;
        } else if tail.ends_with(b"\x1b[4l") {
            self.modes.insert_mode = false;
        } else if tail.ends_with(b"\x1b[?6h") {
            self.modes.origin_mode = true;
        } else if tail.ends_with(b"\x1b[?6l") {
            self.modes.origin_mode = false;
        } else if tail.ends_with(b"\x1b[?7h") {
            self.modes.wrap_mode = true;
        } else if tail.ends_with(b"\x1b[?7l") {
            self.modes.wrap_mode = false;
        } else if tail.ends_with(b"\x1b[?12h") {
            self.modes.cursor_blinking = true;
        } else if tail.ends_with(b"\x1b[?12l") {
            self.modes.cursor_blinking = false;
        } else if tail.ends_with(b"\x1b[?1h") {
            self.modes.cursor_keys = true;
        } else if tail.ends_with(b"\x1b[?1l") {
            self.modes.cursor_keys = false;
        } else if tail.ends_with(b"\x1b=") {
            self.modes.application_keypad = true;
        } else if tail.ends_with(b"\x1b>") {
            self.modes.application_keypad = false;
        } else if tail.ends_with(b"\x1b[>4;2m") {
            self.modes.extended_keys_request = ExtendedKeys::All;
        } else if tail.ends_with(b"\x1b[>4;1m") {
            self.modes.extended_keys_request = ExtendedKeys::Standard;
        } else if tail.ends_with(b"\x1b[>4;0m")
            || tail.ends_with(b"\x1b[>4m")
            || tail.ends_with(b"\x1b[>4n")
        {
            // `CSI > 4 m` with no level, and `CSI > 4 n`, both put the keyboard
            // back to the standard forms.
            self.modes.extended_keys_request = ExtendedKeys::Off;
        } else if tail.ends_with(b"\x1b[?996n") {
            // DSR ?996: the pane is asking which theme it is running under.
            self.theme_query = true;
        }

        if let Some(reply) = device_attributes_reply(&tail) {
            return Some(reply);
        }

        let private = tail
            .windows(3)
            .rposition(|window| window == b"\x1b[?")
            .and_then(|start| tail.get(start + 3..));
        if let Some(body) = private {
            if body.last() == Some(&b'p') && body.get(body.len().saturating_sub(2)) == Some(&b'$') {
                let digits = &body[..body.len() - 2];
                if let Ok(mode) = std::str::from_utf8(digits).unwrap_or("").parse::<u32>() {
                    let status = match mode {
                        2026 => {
                            if self.modes.synchronized_output {
                                1
                            } else {
                                2
                            }
                        }
                        25 => {
                            if self.modes.cursor_visible {
                                1
                            } else {
                                2
                            }
                        }
                        1004 => {
                            if self.modes.focus_reporting {
                                1
                            } else {
                                2
                            }
                        }
                        2031 => {
                            if self.modes.theme_updates {
                                1
                            } else {
                                2
                            }
                        }
                        1000 => self.mouse_mode_status(MouseTrackingMode::Standard),
                        1002 => self.mouse_mode_status(MouseTrackingMode::Button),
                        1003 => self.mouse_mode_status(MouseTrackingMode::All),
                        1005 => {
                            if self.modes.mouse_utf8 {
                                1
                            } else {
                                2
                            }
                        }
                        1006 => {
                            if self.modes.mouse_sgr {
                                1
                            } else {
                                2
                            }
                        }
                        _ => 0,
                    };
                    return Some(format!("\x1b[?{mode};{status}$y").into_bytes());
                }
            }
        }
        None
    }
}

/// Collects `DCS $ q Pt ST` (DECRQSS), the request for a setting's current
/// value.
///
/// tmux recognizes only the cursor-style request and answers everything else
/// with the "invalid request" form, so that is all this reproduces.
#[derive(Default)]
struct DecrqssDetector {
    prefix: Vec<u8>,
    body: Vec<u8>,
    in_dcs: bool,
    escaped: bool,
}

impl DecrqssDetector {
    /// Returns the DECRQSS payload once a complete request has been read.
    fn feed_byte(&mut self, byte: u8) -> Option<Vec<u8>> {
        if !self.in_dcs {
            self.prefix.push(byte);
            if self.prefix.ends_with(b"\x1bP$q") {
                self.prefix.clear();
                self.body.clear();
                self.in_dcs = true;
            } else if self.prefix.len() > 4 {
                self.prefix.remove(0);
            }
            return None;
        }
        if self.escaped {
            self.escaped = false;
            if byte == b'\\' {
                return self.finish();
            }
            self.body.push(0x1b);
        }
        match byte {
            0x9c => self.finish(),
            0x1b => {
                self.escaped = true;
                None
            }
            _ => {
                if self.body.len() < 64 {
                    self.body.push(byte);
                    None
                } else {
                    self.in_dcs = false;
                    self.body.clear();
                    None
                }
            }
        }
    }

    fn finish(&mut self) -> Option<Vec<u8>> {
        self.in_dcs = false;
        self.escaped = false;
        Some(std::mem::take(&mut self.body))
    }
}

/// tmux's `input_handle_decrqss` reply. The only setting it reports is the
/// cursor style; anything else gets `DCS 0 $ r ST`, its "invalid request" form.
///
/// Divergence: with no DECSCUSR applied tmux falls back to the `cursor-style`
/// option, which the pane's reader has no view of, so hmux always reports the
/// default 0 there. Both agree once a pane has set a style itself.
fn decrqss_reply(request: &[u8], shape: PaneCursorShape, blinking: bool) -> Vec<u8> {
    if request != b" q" {
        return b"\x1bP0$r\x1b\\".to_vec();
    }
    let ps = match shape {
        PaneCursorShape::Default => 0,
        PaneCursorShape::Block if blinking => 1,
        PaneCursorShape::Block => 2,
        PaneCursorShape::Underline if blinking => 3,
        PaneCursorShape::Underline => 4,
        PaneCursorShape::Bar if blinking => 5,
        PaneCursorShape::Bar => 6,
    };
    format!("\x1bP1$r q{ps} q\x1b\\").into_bytes()
}

/// The version XTVERSION reports. hmux presents a tmux-compatible surface, and
/// an application that special-cases a terminal by name has to see the same
/// answer the daemon's command language claims to implement.
const XTVERSION_NAME: &str = "tmux";

/// The device queries tmux answers out of its own state rather than the grid.
///
/// Each is only answered with parameter 0 or none, as tmux's `input_get`
/// default does; any other parameter is silently ignored there and here.
fn device_attributes_reply(tail: &[u8]) -> Option<Vec<u8>> {
    // Primary DA. The pinned tmux is built with sixel support, which is what
    // puts the `4` in its answer.
    if tail.ends_with(b"\x1b[c") || tail.ends_with(b"\x1b[0c") {
        return Some(b"\x1b[?1;2;4c".to_vec());
    }
    // Secondary DA. 84 is `T`, tmux's terminal identifier.
    if tail.ends_with(b"\x1b[>c") || tail.ends_with(b"\x1b[>0c") {
        return Some(b"\x1b[>84;0;0c".to_vec());
    }
    if tail.ends_with(b"\x1b[>q") || tail.ends_with(b"\x1b[>0q") {
        return Some(
            format!(
                "\x1bP>|{XTVERSION_NAME} {}\x1b\\",
                crate::server::TMUX_VERSION
            )
            .into_bytes(),
        );
    }
    // DSR 5n: the terminal is operating with no malfunction.
    if tail.ends_with(b"\x1b[5n") {
        return Some(b"\x1b[0n".to_vec());
    }
    None
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum CursorShapeState {
    #[default]
    Ground,
    Esc,
    Csi,
    Parameter,
    Space,
}

impl CursorShapeDetector {
    fn feed_byte(&mut self, byte: u8) -> Option<u8> {
        use CursorShapeState::*;
        self.state = match (self.state, byte) {
            (Ground, b'\x1b') => Esc,
            (Ground, _) => Ground,
            (Esc, b'[') => {
                self.parameter = 0;
                Csi
            }
            (Esc, b'\x1b') => Esc,
            (Esc, _) => Ground,
            (Csi | Parameter, b'0'..=b'9') => {
                self.parameter = self
                    .parameter
                    .saturating_mul(10)
                    .saturating_add(byte - b'0');
                Parameter
            }
            (Csi | Parameter, b' ') => Space,
            (Csi | Parameter, b'\x1b') => Esc,
            (Csi | Parameter, _) => Ground,
            (Space, b'q') if self.parameter <= 6 => {
                let shape = self.parameter;
                self.state = Ground;
                return Some(shape);
            }
            (Space, b'\x1b') => Esc,
            (Space, _) => Ground,
        };
        None
    }
}

impl CursorPositionReportQueryDetector {
    /// Recognize DSR 6n. The private form `CSI ? 6 n` (DECXCPR) is deliberately
    /// not recognized: tmux's private-DSR handler answers only the theme
    /// question, so a reply hmux sent there would be a divergence of its own.
    fn feed_byte(&mut self, byte: u8) -> bool {
        use CursorPositionReportState::{Csi, Esc, Ground, SawSix};

        let next = match (self.state, byte) {
            (Ground, b'\x1b') => Esc,
            (Ground, _) => Ground,
            (Esc, b'[') => Csi,
            (Esc, b'\x1b') => Esc,
            (Esc, _) => Ground,
            (Csi, b'6') => SawSix,
            (Csi, b'\x1b') => Esc,
            (Csi, _) => Ground,
            (SawSix, b'n') => {
                self.state = Ground;
                return true;
            }
            (SawSix, b'\x1b') => Esc,
            (SawSix, _) => Ground,
        };
        self.state = next;
        false
    }
}

#[derive(Default)]
struct BellDetector {
    state: BellState,
}

#[derive(Clone, Copy, Default)]
enum BellState {
    #[default]
    Ground,
    Escape,
    Osc,
    OscEscape,
}

impl BellDetector {
    fn feed(&mut self, byte: u8) -> bool {
        use BellState::{Escape, Ground, Osc, OscEscape};
        match self.state {
            Ground if byte == 0x1b => self.state = Escape,
            Ground => return byte == 0x07,
            Escape if byte == b']' => self.state = Osc,
            Escape if byte == 0x1b => self.state = Escape,
            Escape => {
                self.state = Ground;
                return byte == 0x07;
            }
            Osc if byte == 0x07 => self.state = Ground,
            Osc if byte == 0x1b => self.state = OscEscape,
            Osc => {}
            OscEscape if byte == b'\\' || byte == 0x07 => self.state = Ground,
            OscEscape => self.state = Osc,
        }
        false
    }
}

/// Streaming remover for the screen/tmux window-title control, `ESC k <title>
/// ST`, where the string terminator is either `ESC \` (ST) or a bare `BEL`.
///
/// This is terminfo's `tsl`/`fsl` pair for the `screen`/`tmux` terminal types.
/// Shells and prompt frameworks emit it to advertise the running command in the
/// window title; under a `screen`-family `$TERM` that is what zsh's default
/// `precmd`/`preexec` title hooks use. libghostty-vt (the native grid) does not
/// recognize `ESC k`, so its bytes would otherwise be printed literally into the
/// screen. Real tmux parses and consumes the sequence, so to match it we drop
/// the whole thing from the byte stream before it reaches the grid.
///
/// State is retained across calls so a sequence split across PTY reads is still
/// removed. A held `ESC` at end of input (state [`ScreenTitleState::Esc`]) is
/// carried into the next call and only emitted once we know it does not begin
/// `ESC k` — this delays a lone `ESC` by at most one burst, harmless for a grid
/// that is repainted after each burst anyway.
#[derive(Default)]
struct ScreenTitleStripper {
    state: ScreenTitleState,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ScreenTitleState {
    /// Passing bytes through.
    #[default]
    Ground,
    /// Saw an `ESC` in ground state; it is held pending the next byte.
    Esc,
    /// Inside `ESC k …`; dropping the title text until the terminator.
    Title,
    /// Inside the title and saw an `ESC`; a following `\` ends it (ST).
    TitleEsc,
}

impl ScreenTitleStripper {
    /// Return `input` with any `ESC k … ST/BEL` sequences removed, resuming from
    /// the state left by the previous call.
    fn filter(&mut self, input: &[u8]) -> Vec<u8> {
        use ScreenTitleState::{Esc, Ground, Title, TitleEsc};
        let mut out = Vec::with_capacity(input.len());
        for &byte in input {
            match self.state {
                Ground => {
                    if byte == 0x1b {
                        self.state = Esc; // hold the ESC
                    } else {
                        out.push(byte);
                    }
                }
                Esc => match byte {
                    b'k' => self.state = Title, // ESC k → begin title, drop both
                    0x1b => out.push(0x1b),     // ESC ESC → emit one, hold the new
                    _ => {
                        // Some other escape (`ESC [`, `ESC ]`, …): pass through
                        // unchanged so the query detectors still see it.
                        out.push(0x1b);
                        out.push(byte);
                        self.state = Ground;
                    }
                },
                Title => match byte {
                    0x07 => self.state = Ground, // BEL terminator
                    0x1b => self.state = TitleEsc,
                    _ => {} // title text: drop
                },
                TitleEsc => match byte {
                    b'\\' => self.state = Ground, // ST terminator, drop both
                    0x1b => {}                    // another ESC: stay pending
                    _ => self.state = Title,      // not a terminator: still title
                },
            }
        }
        out
    }
}

/// tmux's `INPUT_BUF_DEFAULT_SIZE`, the `input-buffer-size` default.
const INPUT_BUFFER_DEFAULT_SIZE: u32 = 1_048_576;

/// tmux's `INPUT_BUF_START`: the parser's string buffer starts here and doubles.
const INPUT_BUFFER_START: usize = 32;

/// How long a terminal string may actually grow under `input-buffer-size`.
///
/// tmux's `input_input` doubles the buffer from [`INPUT_BUFFER_START`] and
/// abandons the string once the next doubling would pass the option, so the
/// usable capacity is the largest such power of two that fits — and a string
/// is discarded as soon as it would fill it.
fn input_buffer_capacity(limit: u32) -> usize {
    let limit = limit as usize;
    let mut capacity = INPUT_BUFFER_START;
    while capacity.saturating_mul(2) <= limit {
        capacity *= 2;
    }
    capacity
}

/// The only DCS tmux forwards: everything after this prefix and before the
/// string terminator is what `allow-passthrough` puts on a client's tty.
const PASSTHROUGH_INTRO: &[u8] = b"\x1bPtmux;";

/// How much of one payload is kept. tmux abandons an input string that outgrows
/// `INPUT_BUF_LIMIT`; the cap here is smaller because a payload is held in the
/// server until the loop comes round rather than written straight out.
const PASSTHROUGH_LIMIT: usize = 64 * 1024;

/// Collects the payloads of `DCS tmux; … ST` out of a pane's output.
///
/// Ghostty already consumes the sequence, so nothing here has to remove it —
/// only to read it, as tmux's `input_dcs_dispatch` does. Inside the string an
/// `ESC` that is not the terminator is dropped and the byte after it kept,
/// which is why applications double the escapes in a passthrough payload.
#[derive(Default)]
struct PassthroughDetector {
    state: PassthroughState,
    /// How much of [`PASSTHROUGH_INTRO`] has matched so far.
    matched: usize,
    payload: Vec<u8>,
    overflowed: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum PassthroughState {
    #[default]
    Ground,
    /// Part-way through the introducer.
    Intro,
    /// Inside the payload.
    Payload,
    /// Saw an `ESC` inside the payload; a following `\` ends it (ST).
    PayloadEsc,
}

impl PassthroughDetector {
    /// The completed payload, once the string terminator arrives.
    fn feed_byte(&mut self, byte: u8) -> Option<Vec<u8>> {
        use PassthroughState::{Ground, Intro, Payload, PayloadEsc};
        match self.state {
            Ground | Intro => {
                if byte == PASSTHROUGH_INTRO[self.matched] {
                    self.matched += 1;
                    if self.matched == PASSTHROUGH_INTRO.len() {
                        self.matched = 0;
                        self.payload.clear();
                        self.overflowed = false;
                        self.state = Payload;
                    } else {
                        self.state = Intro;
                    }
                } else {
                    // A mismatch can itself be the start of the next attempt.
                    self.matched = usize::from(byte == PASSTHROUGH_INTRO[0]);
                    self.state = if self.matched == 0 { Ground } else { Intro };
                }
                None
            }
            Payload => {
                if byte == 0x1b {
                    self.state = PayloadEsc;
                } else {
                    self.push(byte);
                }
                None
            }
            PayloadEsc => {
                self.state = Payload;
                if byte != b'\\' {
                    self.push(byte);
                    return None;
                }
                self.state = Ground;
                let payload = std::mem::take(&mut self.payload);
                (!std::mem::take(&mut self.overflowed)).then_some(payload)
            }
        }
    }

    fn push(&mut self, byte: u8) {
        if self.payload.len() == PASSTHROUGH_LIMIT {
            self.overflowed = true;
            return;
        }
        self.payload.push(byte);
    }
}

/// Applies `allow-set-title` to the sequences that retitle a pane, and rewrites
/// an APC title as the OSC 2 that means the same thing.
///
/// tmux gates OSC 0, OSC 2 and APC on the one option and hands all three to the
/// same `screen_set_title`. Ghostty owns the title here and does not recognize
/// APC, so an allowed OSC passes through untouched, an allowed APC passes
/// through as its OSC 2 equivalent — which keeps the two in stream order, since
/// the last one to arrive is the one that wins — and a refused sequence is
/// removed before the emulator can apply it.
///
/// Like [`ScreenTitleStripper`], state is retained across calls so a sequence
/// split across PTY reads is still recognized.
#[derive(Default)]
struct SetTitleFilter {
    /// The titles accepted since the last drain, in stream order.
    accepted: Vec<String>,
    state: SetTitleState,
    /// The escape bytes seen so far: the prefix of a sequence that may yet turn
    /// out not to be a title, or an APC payload waiting for its terminator.
    held: Vec<u8>,
    /// Set when the string outgrew the `input-buffer-size` capacity.
    overflowed: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum SetTitleState {
    #[default]
    Ground,
    /// Saw an `ESC` in ground state; it is held pending the next byte.
    Esc,
    /// Inside `ESC ]`, reading the OSC number.
    OscNumber,
    /// Inside an allowed OSC 0/2, collecting the title so its length can be
    /// held against `input-buffer-size` before the emulator sees it.
    OscTitle,
    /// Saw an `ESC` while collecting; a following `\` ends it (ST).
    OscTitleEsc,
    /// Inside a refused OSC 0/2, dropping the rest of the string.
    OscDrop,
    /// Saw an `ESC` while dropping; a following `\` ends the string (ST).
    OscDropEsc,
    /// Inside `ESC _`, collecting the APC payload.
    Apc,
    /// Saw an `ESC` inside the payload; a following `\` ends it (ST).
    ApcEsc,
}

impl SetTitleFilter {
    fn filter(&mut self, input: &[u8], allowed: bool, capacity: usize) -> Vec<u8> {
        use SetTitleState::{
            Apc, ApcEsc, Esc, Ground, OscDrop, OscDropEsc, OscNumber, OscTitle, OscTitleEsc,
        };
        let mut out = Vec::with_capacity(input.len());
        for &byte in input {
            match self.state {
                Ground => {
                    if byte == 0x1b {
                        self.held.clear();
                        self.held.push(byte);
                        self.state = Esc;
                    } else {
                        out.push(byte);
                    }
                }
                Esc => match byte {
                    b']' => {
                        self.held.push(byte);
                        self.state = OscNumber;
                    }
                    b'_' => {
                        self.held.clear();
                        self.overflowed = false;
                        self.state = Apc;
                    }
                    0x1b => out.push(0x1b), // ESC ESC → emit one, hold the new
                    _ => {
                        out.append(&mut self.held);
                        out.push(byte);
                        self.state = Ground;
                    }
                },
                // Only `0` and `2` are titles; every other OSC — including the
                // ones with a digit in common, like 04 or 52 — is released as
                // soon as it can no longer be one.
                OscNumber => match byte {
                    b'0' | b'2' if self.held.len() == 2 => self.held.push(byte),
                    b';' if self.held.len() == 3 => {
                        if allowed {
                            self.held.push(byte);
                            self.overflowed = false;
                            self.state = OscTitle;
                        } else {
                            self.held.clear();
                            self.state = OscDrop;
                        }
                    }
                    _ => {
                        out.append(&mut self.held);
                        out.push(byte);
                        self.state = Ground;
                    }
                },
                OscTitle => match byte {
                    0x07 => {
                        out.extend_from_slice(&self.finish_osc_title(&[0x07], capacity));
                        self.state = Ground;
                    }
                    0x1b => self.state = OscTitleEsc,
                    _ => self.push(byte, capacity),
                },
                OscTitleEsc => {
                    if byte == b'\\' {
                        out.extend_from_slice(&self.finish_osc_title(b"\x1b\\", capacity));
                        self.state = Ground;
                    } else {
                        self.push(0x1b, capacity);
                        self.push(byte, capacity);
                        self.state = OscTitle;
                    }
                }
                OscDrop => match byte {
                    0x07 => self.state = Ground, // BEL terminator
                    0x1b => self.state = OscDropEsc,
                    _ => {} // title text: drop
                },
                OscDropEsc => match byte {
                    b'\\' => self.state = Ground, // ST terminator
                    0x1b => {}                    // another ESC: stay pending
                    _ => self.state = OscDrop,
                },
                Apc => match byte {
                    0x07 => {
                        out.extend_from_slice(&self.finish_apc(allowed));
                        self.state = Ground;
                    }
                    0x1b => self.state = ApcEsc,
                    _ => self.push(byte, capacity),
                },
                ApcEsc => match byte {
                    b'\\' => {
                        out.extend_from_slice(&self.finish_apc(allowed));
                        self.state = Ground;
                    }
                    _ => {
                        // Not a terminator, so the ESC was payload after all.
                        self.push(0x1b, capacity);
                        self.push(byte, capacity);
                        self.state = Apc;
                    }
                },
            }
        }
        out
    }

    /// Collect one byte of the string, abandoning it once it would fill the
    /// parser's buffer — tmux's `input_input`.
    fn push(&mut self, byte: u8, capacity: usize) {
        if self.held.len() + 1 >= capacity {
            self.overflowed = true;
            return;
        }
        self.held.push(byte);
    }

    /// The OSC title as the emulator should see it: the whole sequence, or
    /// nothing when it outgrew the parser's buffer.
    fn finish_osc_title(&mut self, terminator: &[u8], capacity: usize) -> Vec<u8> {
        let mut sequence = std::mem::take(&mut self.held);
        // The four introducer bytes are not part of tmux's string buffer.
        let overflowed = std::mem::take(&mut self.overflowed)
            || sequence.len().saturating_sub(4) + 1 >= capacity;
        if overflowed {
            return Vec::new();
        }
        self.note_title(&sequence[4..]);
        sequence.extend_from_slice(terminator);
        sequence
    }

    /// Record an accepted title, as tmux's `screen_set_title` does.
    fn note_title(&mut self, title: &[u8]) {
        self.accepted
            .push(String::from_utf8_lossy(title).into_owned());
    }

    /// The bytes an APC title leaves behind: the OSC 2 it is equivalent to, or
    /// nothing when the option refuses it or the payload ran away.
    fn finish_apc(&mut self, allowed: bool) -> Vec<u8> {
        let payload = std::mem::take(&mut self.held);
        if !allowed || std::mem::take(&mut self.overflowed) {
            return Vec::new();
        }
        self.note_title(&payload);
        let mut osc = b"\x1b]2;".to_vec();
        osc.extend_from_slice(&payload);
        osc.extend_from_slice(b"\x1b\\");
        osc
    }
}

fn cursor_position_report(terminal: &Terminal) -> Option<Vec<u8>> {
    let (x, y) = terminal.cursor_position().ok()?;
    Some(format!("\x1b[{};{}R", y.saturating_add(1), x.saturating_add(1)).into_bytes())
}

/// Upper bound on bytes buffered for a child that is not reading its stdin. A
/// pane's queued input never grows past this; once full, new input is dropped
/// rather than allowed to stall the server. Real tmux likewise caps a pane's
/// input buffer and drops when a child will not drain it. 32 KiB comfortably
/// absorbs a paste or a burst of query replies (each ~8 bytes) while keeping the
/// idle footprint of a once-backed-up pane small.
const PANE_INPUT_CAP: usize = 32 * 1024;

/// Set `fd` non-blocking. Used on the pty master so neither reads nor writes on
/// it can ever block a server thread.
fn set_nonblocking(fd: c_int) -> io::Result<()> {
    // SAFETY: F_GETFL/F_SETFL on an owned fd.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Append `bytes` to a pane's pending-input queue (bounded) and flush as much as
/// the child will currently accept. Never blocks: the pty master is non-blocking,
/// so bytes the child cannot take yet stay queued for the reader thread to flush
/// on the next writability. Excess beyond [`PANE_INPUT_CAP`] is dropped.
fn enqueue_pane_input(fd: c_int, pending: &RefCell<VecDeque<u8>>, bytes: &[u8]) -> PaneInputStats {
    let mut queued = pending.borrow_mut();
    let room = PANE_INPUT_CAP.saturating_sub(queued.len());
    let take = bytes.len().min(room);
    queued.extend(&bytes[..take]);
    flush_pane_input(fd, &mut queued);
    // New input is appended behind any old backlog. Therefore any bytes left
    // at the tail, up to `take`, are from this batch; the rest reached the PTY
    // before this call returned.
    let new_queued = queued.len().min(take);
    PaneInputStats {
        written: take - new_queued,
        queued: new_queued,
        dropped: bytes.len() - take,
    }
}

/// Write as much of `queued` to `fd` as the child will accept without blocking,
/// removing what was written. Stops on `EAGAIN` (child's input buffer full); the
/// remainder is flushed later when the master polls writable. The queue lock is
/// held across the writes, but every write is non-blocking, so this can never
/// stall a caller.
fn flush_pane_input(fd: c_int, queued: &mut VecDeque<u8>) {
    while !queued.is_empty() {
        let (front, _) = queued.as_slices();
        // SAFETY: writing a valid slice to the non-blocking pty master fd.
        let n = unsafe { libc::write(fd, front.as_ptr() as *const c_void, front.len()) };
        if n > 0 {
            queued.drain(..n as usize);
            continue;
        }
        if n < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            // WouldBlock (buffer full) or a write error: keep the remainder queued
            // and stop; the reader thread retries on the next POLLOUT.
            break;
        }
        break; // n == 0
    }
}

fn ghostty_err(e: crate::ghostty::Error) -> io::Error {
    io::Error::other(format!("ghostty: {e}"))
}

/// A pane's foreground-process identity, captured while the pane was at hand.
///
/// Holds only owned data (pids and the spawn command line), so a deferred
/// format callback can resolve the process-derived variables long after the
/// pane borrow ended. A pid whose process has exited simply fails its
/// `/proc` read and falls through, exactly as the live path always has.
#[derive(Clone)]
pub(crate) struct PaneProcessProbe {
    foreground: Option<pid_t>,
    session_leader: Option<pid_t>,
    fallback_command: Option<String>,
}

impl PaneProcessProbe {
    /// The working directory of the pane's foreground process group
    /// (`#{pane_current_path}`).
    ///
    /// Mirrors tmux's `osdep_get_cwd`: prefer the foreground group, then fall
    /// back to the session leader — the group id is only a pid while the
    /// group's leader lives, and a shell pipeline whose first member exited
    /// leaves a group that names no process.
    pub(crate) fn current_path(&self) -> Option<String> {
        [self.foreground, self.session_leader]
            .into_iter()
            .flatten()
            .find_map(|pid| CurrentPlatform::process_cwd(pid as u32))
            .map(|path| path.to_string_lossy().into_owned())
    }

    /// The program occupying the pane's foreground process group
    /// (`#{pane_current_command}`).
    ///
    /// Mirrors tmux's `format_cb_current_command`, which tries the foreground
    /// group's `argv[0]` and then falls back to the pane's own command line —
    /// the fallback is what answers for a leaderless group.
    pub(crate) fn current_command(&self) -> Option<String> {
        // tmux's Linux osdep_get_name reads argv[0] from /proc/PID/cmdline.
        // Keep the executable-name candidates as a fallback for platforms or
        // processes where the argument vector is unavailable.
        let foreground = self
            .foreground
            .and_then(|pid| {
                CurrentPlatform::process_arguments(pid as u32)
                    .into_iter()
                    .next()
                    .or_else(|| {
                        CurrentPlatform::process_programs(pid as u32)
                            .into_iter()
                            .next()
                    })
            })
            .map(|program| program.to_string_lossy().into_owned());

        foreground
            .filter(|program| !program.is_empty())
            .or_else(|| self.fallback_command.clone())
            .map(|command| parse_window_name(&command))
            .filter(|name| !name.is_empty())
    }
}

/// Render a pane's argument vector the way tmux's `cmd_stringify_argv` does,
/// for the callers that reduce the result with [`parse_window_name`].
///
/// tmux escapes each argument; that step is skipped because it cannot change
/// what the caller sees. `parse_window_name` cuts at the first space, and it
/// does so after resolving quotes, so both a quoted and an unquoted argument
/// reduce to the same leading word either way.
fn stringify_argv(argv: &[String]) -> String {
    argv.join(" ")
}

/// Reduce a command line to the program name tmux displays for it, following
/// tmux's `parse_window_name`: take the first quoted or whitespace-delimited
/// word, drop an `exec` prefix and any leading dashes (a login shell's `-bash`
/// is reported as `bash`), and keep only the last component of an absolute
/// path.
///
/// tmux additionally runs the result through `clean_name`, which escapes
/// non-printable bytes for display. That step is not reproduced: every name
/// reaching this function is a program name, and the trailing-byte trim below
/// already removes the control characters `clean_name` would have escaped.
fn parse_window_name(input: &str) -> String {
    let mut name = input.strip_prefix('"').unwrap_or(input);
    if let Some(quote) = name.find('"') {
        name = &name[..quote];
    }
    name = name.strip_prefix("exec ").unwrap_or(name);
    name = name.trim_start_matches([' ', '-']);
    if let Some(space) = name.find(' ') {
        name = &name[..space];
    }
    // tmux keeps trailing bytes only while they are alphanumeric or
    // punctuation, which together are exactly the printable ASCII characters.
    let trimmed = name.trim_end_matches(|ch: char| !ch.is_ascii_graphic());
    if trimmed.starts_with('/') {
        return Path::new(trimmed).file_name().map_or_else(
            || trimmed.to_string(),
            |base| base.to_string_lossy().into_owned(),
        );
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_shape_detector_handles_split_decscusr() {
        let mut detector = CursorShapeDetector::default();
        for &byte in b"noise\x1b[" {
            assert_eq!(detector.feed_byte(byte), None);
        }
        assert_eq!(detector.feed_byte(b'6'), None);
        assert_eq!(detector.feed_byte(b' '), None);
        assert_eq!(detector.feed_byte(b'q'), Some(6));
        assert_eq!(detector.feed_byte(b'x'), None);
    }

    #[test]
    fn inert_pane_feeds_grid() {
        let pane = Pane::inert(20, 4).expect("inert pane");
        pane.feed(b"synthetic\r\noutput");
        let dump = pane.dump().expect("dump");
        assert!(dump.contains("synthetic"), "got {dump:?}");
        assert!(dump.contains("output"), "got {dump:?}");
        assert!(!pane.is_live());
    }

    #[test]
    fn observation_screen_includes_cursor_visibility_and_shape() {
        let pane = Pane::inert(20, 4).expect("inert pane");
        pane.feed(b"\x1b[6 q\x1b[?25h");

        let visible = pane.observation().last_lines(1).expect("screen tail");
        assert!(visible.cursor_visible);
        assert_eq!(visible.cursor_shape, 6);

        pane.feed(b"\x1b[?25l");
        let hidden = pane.observation().last_lines(1).expect("screen tail");
        assert!(!hidden.cursor_visible);
        assert_eq!(hidden.cursor_shape, 6);
    }

    #[test]
    fn observation_screen_sources_expose_viewport_and_scrollback() {
        // A 4-row pane; write 8 lines so 4 scroll into history.
        let pane = Pane::inert(20, 4).expect("inert pane");
        pane.feed(b"L1\r\nL2\r\nL3\r\nL4\r\nL5\r\nL6\r\nL7\r\nL8");
        let obs = pane.observation();

        assert_eq!(obs.scrollback_rows().expect("scrollback"), 4);

        // Visible: only the on-screen viewport (the last 4 rows), no history.
        let visible = obs
            .screen(ScreenSource::Visible, 100)
            .expect("visible")
            .text;
        assert!(
            visible.contains("L5") && visible.contains("L8"),
            "{visible:?}"
        );
        assert!(
            !visible.contains("L1"),
            "history must be excluded: {visible:?}"
        );

        // Recent: reads back into scrollback history.
        let recent = obs.screen(ScreenSource::Recent, 100).expect("recent").text;
        assert!(recent.contains("L1") && recent.contains("L8"), "{recent:?}");

        // last_lines is the Recent shim.
        assert_eq!(obs.last_lines(100).expect("last_lines").text, recent);
    }

    #[test]
    fn large_scroll_revision_only_advances_for_large_regions() {
        let pane = Pane::inert(20, 24).expect("pane");
        pane.feed(b"\x1b[1;6r\x1b[6;1H");
        pane.feed(b"\n");
        assert_eq!(pane.observation_state().large_scroll_revision(), 0);

        pane.feed(b"\x1b[1;18r\x1b[18;1H");
        pane.feed(b"\x1b]2;title\ncontinuation\x07");
        assert_eq!(
            pane.observation_state().large_scroll_revision(),
            0,
            "control-string payloads are not terminal linefeeds"
        );
        pane.feed(b"\n");
        assert!(
            pane.observation_state().large_scroll_revision() > 0,
            "a linefeed at the bottom of a three-quarter-pane region must \
             publish a large-scroll redraw hint"
        );
    }

    #[test]
    fn clear_history_preserves_the_visible_viewport() {
        let pane = Pane::inert(10, 4).expect("pane");
        pane.feed(b"1\r\n2\r\n3\r\n4\r\n5\r\n6\r\n7");
        assert!(pane.scrollback_rows().expect("history before") > 0);
        let visible_before = pane.dump_viewport_vt(0, 4).expect("visible before").0;

        pane.clear_history().expect("clear history");

        assert_eq!(pane.scrollback_rows().expect("history after"), 0);
        assert_eq!(
            pane.dump_viewport_vt(0, 4).expect("visible after").0,
            visible_before
        );
    }

    #[test]
    fn observation_unwrapped_source_rejoins_soft_wraps() {
        let pane = Pane::inert(10, 4).expect("inert pane");
        pane.feed(b"abcdefghijKLMNOP"); // soft-wraps across two rows
        let obs = pane.observation();

        let recent = obs.screen(ScreenSource::Recent, 100).expect("recent").text;
        assert_eq!(recent.lines().count(), 2, "wrapped: {recent:?}");

        let unwrapped = obs
            .screen(ScreenSource::RecentUnwrapped, 100)
            .expect("unwrapped")
            .text;
        assert_eq!(unwrapped.trim_end(), "abcdefghijKLMNOP", "{unwrapped:?}");
    }

    #[test]
    fn inert_pane_has_no_current_path() {
        // An inert pane has no child, so there is no live cwd to read; the
        // format layer falls back to the server cwd for these.
        let pane = Pane::inert(20, 4).expect("inert pane");
        assert_eq!(pane.current_path(), None);
    }

    #[test]
    fn parse_window_name_reduces_a_command_line_to_its_program() {
        // The plain cases: a bare name survives, an absolute path loses its
        // directories, and arguments are dropped.
        assert_eq!(parse_window_name("bash"), "bash");
        assert_eq!(parse_window_name("/usr/bin/sleep"), "sleep");
        assert_eq!(parse_window_name("sleep 30"), "sleep");
        // A relative path keeps its directories; tmux only takes the basename
        // of a name that starts at the root.
        assert_eq!(parse_window_name("bin/sleep"), "bin/sleep");
        // A login shell announces itself with a leading dash.
        assert_eq!(parse_window_name("-zsh"), "zsh");
        assert_eq!(parse_window_name("exec vim file"), "vim");
        // The stringified argv of the leaderless-pipeline fixture.
        assert_eq!(parse_window_name(r#"bash -mc "echo x | sleep 30""#), "bash");
        // Quotes are resolved before the cut at the first space, so a quoted
        // argv[0] containing one is still cut there.
        assert_eq!(parse_window_name(r#""my program" -x"#), "my");
        assert_eq!(parse_window_name("sleep\r\n"), "sleep");
        assert_eq!(parse_window_name(""), "");
    }

    #[test]
    fn stringify_argv_reduces_to_the_program_the_pane_was_given() {
        let argv = ["bash", "-mc", "echo x | sleep 30"].map(String::from);
        assert_eq!(parse_window_name(&stringify_argv(&argv)), "bash");
        // A pane spawned with no command carries just the shell.
        let shell = ["/bin/zsh"].map(String::from);
        assert_eq!(parse_window_name(&stringify_argv(&shell)), "zsh");
    }

    /// A `PaneIo` over a placeholder descriptor, for driving the output
    /// pipeline without a child. `Pane::feed` writes straight to the emulator,
    /// so it deliberately skips the filters this exercises.
    fn test_pane_io(pane: &Pane) -> PaneIo {
        let null = std::fs::File::open("/dev/null").expect("/dev/null");
        PaneIo::new(
            &OwnedFd::from(null),
            pane.observation_state(),
            Rc::clone(&pane.terminal_queries),
            Rc::clone(&pane.pending_input),
            Rc::clone(&pane.pipe_output),
            Rc::clone(&pane.pipe_output_active),
            Rc::new(Cell::new(true)),
        )
        .expect("pane io")
    }

    #[test]
    fn observation_reports_osc_title() {
        let pane = Pane::inert(40, 5).expect("inert pane");
        let observation = pane.observation();
        // No title set yet.
        assert_eq!(observation.title().expect("title"), None);
        // Codex reports its live status in the window title via OSC 2. The
        // title is recorded where the child's output is filtered, since that
        // is what holds it to `input-buffer-size`.
        test_pane_io(&pane).process_output(b"\x1b]2;Working (5s)\x07".to_vec());
        assert_eq!(
            observation.title().expect("title").as_deref(),
            Some("Working (5s)")
        );
    }

    #[test]
    fn output_notification_is_broadcast_to_each_subscriber() {
        let pane = Pane::inert(20, 4).expect("inert pane");
        let first = pane.subscribe_output().expect("first subscription");
        let second = pane.subscribe_output().expect("second subscription");
        pane.feed(b"ready");

        for subscription in [&first, &second] {
            let mut pfd = libc::pollfd {
                fd: subscription.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            assert_eq!(unsafe { libc::poll(&mut pfd, 1, 0) }, 1);
            subscription.drain();
        }
    }

    #[test]
    fn control_output_journal_tracks_offsets_and_bounds_history() {
        let mut output = ControlOutputJournal::default();
        output.append(b"abc");
        assert_eq!(output.since(0), (3, b"abc".to_vec()));
        assert_eq!(output.since(2), (3, b"c".to_vec()));

        output.append(&vec![b'x'; CONTROL_OUTPUT_LIMIT + 5]);
        let (end, retained) = output.since(0);
        assert_eq!(end, CONTROL_OUTPUT_LIMIT as u64 + 8);
        assert_eq!(retained.len(), CONTROL_OUTPUT_LIMIT);
        assert!(retained.iter().all(|byte| *byte == b'x'));
    }

    #[test]
    fn control_output_journal_chunks_advance_only_by_returned_bytes() {
        let mut output = ControlOutputJournal::default();
        output.append(b"abcdef");
        assert_eq!(output.chunk(0, 2), (2, 6, b"ab".to_vec()));
        assert_eq!(output.chunk(2, 3), (5, 6, b"cde".to_vec()));
        assert_eq!(output.chunk(5, 3), (6, 6, b"f".to_vec()));
    }

    #[test]
    fn bell_detector_ignores_osc_terminators() {
        let mut detector = BellDetector::default();
        let bytes = b"before\x07\x1b]0;title\x07after\x1b]1;title\x1b\\\x07";
        assert_eq!(bytes.iter().filter(|byte| detector.feed(**byte)).count(), 2);
    }

    #[test]
    fn terminal_query_detectors_handle_pty_boundaries_and_terminators() {
        let mut background = BackgroundColorQueryDetector::default();
        let found = b"before\x1b]11;?\x1b\\after\x1b]11;?\x07\x1b]11;?\x9c"
            .iter()
            .filter(|&&byte| background.feed_byte(byte))
            .count();
        assert_eq!(found, 3);

        let mut dsr = DeviceStatusReportQueryDetector::default();
        assert!(!b"before\x1b[5".iter().any(|&byte| dsr.feed_byte(byte)));
        assert!(dsr.feed_byte(b'n'));

        let mut cpr = CursorPositionReportQueryDetector::default();
        assert!(!b"before\x1b[6".iter().any(|&byte| cpr.feed_byte(byte)));
        assert!(cpr.feed_byte(b'n'));

        // DECXCPR is not a cursor report tmux answers, so the private form must
        // pass through without one.
        let mut private_cpr = CursorPositionReportQueryDetector::default();
        assert!(!b"before\x1b[?6n"
            .iter()
            .any(|&byte| private_cpr.feed_byte(byte)));
    }
}

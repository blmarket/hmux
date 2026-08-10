//! A pane: a child process on a PTY, its output parsed onto an hmux-vt
//! [`PaneScreen`].
//!
//! This is where the "clone" earns its name — instead of proxying to a backing
//! tmux, hmux owns the pty/child and maintains the screen itself. tmux keeps this
//! state in `window_pane` + `screen`/`input.c`; here the grid lives in hmux-vt
//! and the master fd is drained by the central event loop.
//!
//! Only a text-emulation slice is implemented: spawn, feed output → grid, send
//! input, resize, dump. Compositing multiple panes onto an attached client's tty
//! is the next milestone (see the module docs).

use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, VecDeque};
use std::ffi::{CStr, CString};
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

use crate::observability::v1::{PaneObservability, PaneProcess, ScreenSource, ScreenTail};
use crate::platform::{CurrentPlatform, ForkOutcome, OutputWakeup, Platform};
use crate::server::input_keys::ExtendedKeys;
use crate::server::task::{Coroutine, FdInterest, ReadySet, TaskPoll, WaitRequest, WaitToken};
use hmux_vt::observer::{
    decrqss_reply, Event as VtEvent, Observer, OscUpdate, BACKGROUND_COLOR_QUERY,
};
use hmux_vt::parser::{Param, StringEnd, Token, TokenKind};

use hmux_vt::input::MouseEvent;
pub(crate) use hmux_vt::observer::parse_packed_colour;
pub(crate) use hmux_vt::observer::{
    ClipboardEvent as PaneClipboardEvent, CursorShape as PaneCursorShape,
    OutputPolicy as PaneOutputPolicy, PassthroughPolicy,
};
use hmux_vt::screen::{mode, CaptureExtent, Grid, GridDims, ScreenImage, ScreenOptions};
use hmux_vt::PaneScreen;

/// A single pane. Holds the emulated screen and, if live, the child on its pty.
pub struct Pane {
    /// Read-only state shared with observation handles. Keeping this separate
    /// from the PTY owner lets consumers inspect a resolved pane without
    /// retaining the native server's global state lock.
    observation: Rc<NativePaneObservation>,
    /// The running child + pty, or `None` for an inert (process-less) pane.
    child: Option<Child>,
    /// Bytes pending to be written to the child's pty (keystrokes and terminal-
    /// query replies).
    pending_input: Rc<RefCell<VecDeque<u8>>>,
    /// Messages queried from child
    terminal_queries: Rc<RefCell<VecDeque<Vec<u8>>>>,
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

/// How deep `CSI 22 t` may stack pane titles, tmux's limit in
/// `screen_push_title`.
const TITLE_STACK_LIMIT: usize = 10;

static NEXT_PANE_RUNTIME_ID: AtomicU64 = AtomicU64::new(1);

/// The live half of a pane: the child pid and the pty master.
struct Child {
    pid: pid_t,
    master: OwnedFd,
    /// Original process specification retained for command-less respawns.
    spawn_spec: PaneSpawnSpec,
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
    term: Rc<RefCell<PaneScreen>>,
    revision: Cell<u64>,
    /// Revision of the latest scroll operation whose vertical region is large
    /// enough for tmux to prefer one deferred repaint over immediate row draws.
    /// This is monotonic so each attached client can observe it independently.
    large_scroll_revision: Cell<u64>,
    redraw_detector: RefCell<ScrollRedraw>,
    control_output: RefCell<ControlOutputJournal>,
    /// Last DECSCUSR parameter emitted by the pane (0..=6). The VT formatter
    /// restores cursor position but does not serialize this terminal state.
    cursor_shape: Cell<u8>,
    /// The pane's reportable VT modes, republished as one snapshot read back
    /// from the screen's mode word at the end of each output batch.
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
    /// here from the observer's title events; the screen keeps no copy of it.
    announced_title: RefCell<Option<String>>,
    /// tmux's `screen->titles`, the stack `CSI 22 t` pushes onto and
    /// `CSI 23 t` pops from. It lives beside the title rather than in the
    /// screen because that is where the title itself lives.
    title_stack: RefCell<Vec<Option<String>>>,
    /// OSC 4 questions about palette entries the pane does not hold, waiting
    /// for the server to put them to an attached terminal. Each carries the
    /// terminator the pane asked with, because that is what its answer uses.
    palette_queries: RefCell<VecDeque<(u8, bool)>>,
    /// tmux's `PANE_CHANGED`: something happened in this pane that could change
    /// the name `automatic-rename` derives from it — output arrived, or the
    /// pane became the active one. Cleared by the pass that re-derives the name.
    changed: Cell<bool>,
    /// `ESC k … ST` renames the pane's window emitted, waiting for the server
    /// to apply the `allow-rename` policy to them. Queued like the clipboard
    /// events so the option decides each rename once, as tmux's
    /// `input_exit_rename` does, instead of being consulted afresh every time a
    /// format asks for the window's name.
    renames: RefCell<VecDeque<String>>,
    /// The options that decide what the pane's own output is allowed to do.
    output_policy: PaneOutputPolicyCell,
    /// The pane's one tokenizer. Every query reply, OSC state change, mode
    /// change, bell, clipboard event, passthrough payload and title comes out
    /// of this single parse of the byte stream.
    observer: RefCell<Observer>,
}

/// [`PaneOutputPolicy`] as the pane's parser reads it: shared with the server,
/// so each field is an atomic rather than behind the state lock.
struct PaneOutputPolicyCell {
    alternate_screen: Cell<bool>,
    allow_set_title: Cell<bool>,
    passthrough: Cell<u8>,
    input_buffer_size: Cell<u32>,
    cursor_style: Cell<u8>,
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
            cursor_style: self.cursor_style.get(),
            palette: self.palette.borrow().clone(),
        }
    }

    fn store(&self, policy: PaneOutputPolicy) {
        self.alternate_screen.set(policy.alternate_screen);
        self.allow_set_title.set(policy.allow_set_title);
        self.passthrough.set(match policy.passthrough {
            PassthroughPolicy::Off => 0,
            PassthroughPolicy::Visible => 1,
            PassthroughPolicy::Always => 2,
        });
        self.input_buffer_size.set(policy.input_buffer_size);
        self.cursor_style.set(policy.cursor_style);
        {
            let mut palette = self.palette.borrow_mut();
            *palette = policy.palette;
        }
    }
}

/// The options' own defaults, which a pane parses against until the server's
/// first refresh reaches it. The values themselves live in
/// [`PaneOutputPolicy`]'s `Default`.
impl Default for PaneOutputPolicyCell {
    fn default() -> Self {
        let cell = Self {
            alternate_screen: Cell::default(),
            allow_set_title: Cell::default(),
            passthrough: Cell::default(),
            input_buffer_size: Cell::default(),
            cursor_style: Cell::default(),
            palette: RefCell::default(),
        };
        cell.store(PaneOutputPolicy::default());
        cell
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

struct OutputEvent {
    wakeup: <CurrentPlatform as Platform>::OutputWakeup,
}

#[derive(Clone, Copy)]
struct ScrollRegion {
    top: u16,
    bottom: u16,
}

/// Where in its scrolling region an operation moves rows, which decides whether
/// the cursor has to be inside it for the move to be visible.
#[derive(Clone, Copy)]
enum ScrollEdge {
    /// Reverse index: only at the region's top row.
    Top,
    /// Line feed and index: only at the region's bottom row.
    Bottom,
    /// Insert/delete line: anywhere inside the region.
    Inside,
    /// Scroll up/down: the whole region moves regardless of the cursor.
    Any,
}

#[derive(Clone, Copy)]
struct ScrollAction {
    region: ScrollRegion,
    edge: ScrollEdge,
    rows: u16,
}

impl ScrollAction {
    /// Whether tmux would prefer one deferred repaint of the pane over drawing
    /// the moved rows: the region has to be at least half the pane, and the
    /// cursor has to be somewhere the operation actually moves rows.
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

/// Recognizes the operations that scroll a vertical region, so the compositor
/// can choose between repainting rows and repainting the pane.
///
/// This reads the pane's tokens rather than its bytes: the framing is already
/// settled by the time it sees them, so all that is left is the scrolling
/// region, which is screen state the tokens change.
struct ScrollRedraw {
    rows: u16,
    explicit_region: Option<ScrollRegion>,
}

impl ScrollRedraw {
    fn new(rows: u16) -> Self {
        Self {
            rows: rows.max(1),
            explicit_region: None,
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

    /// Classify one token, applying any change it makes to the region first.
    fn scan(&mut self, token: &Token) -> Option<ScrollAction> {
        let edge = match &token.kind {
            TokenKind::Control(0x0a..=0x0c) => ScrollEdge::Bottom,
            TokenKind::Esc {
                intermediates,
                final_byte,
            } if intermediates.is_empty() => match final_byte {
                b'D' => ScrollEdge::Bottom,
                b'M' => ScrollEdge::Top,
                // RIS puts the region back to the whole screen.
                b'c' => {
                    self.explicit_region = None;
                    return None;
                }
                _ => return None,
            },
            TokenKind::Csi {
                private: None,
                params,
                intermediates,
                final_byte,
            } => match (intermediates.as_slice(), final_byte) {
                // DECSTR, which also resets the region.
                ([b'!'], b'p') => {
                    self.explicit_region = None;
                    return None;
                }
                ([], b'r') => {
                    self.set_region(params);
                    return None;
                }
                ([], b'S' | b'T') => ScrollEdge::Any,
                ([], b'L' | b'M') => ScrollEdge::Inside,
                _ => return None,
            },
            _ => return None,
        };
        Some(ScrollAction {
            region: self.region(),
            edge,
            rows: self.rows,
        })
    }

    /// DECSTBM. A region that is empty or runs off the screen is refused, as
    /// tmux refuses it.
    fn set_region(&mut self, params: &[Param]) {
        let read = |index: usize, default: u16| -> u16 {
            params
                .get(index)
                .and_then(|param: &Param| param.value)
                .and_then(|value| u16::try_from(value).ok())
                .filter(|value| *value != 0)
                .unwrap_or(default)
        };
        let top = read(0, 1);
        let bottom = read(1, self.rows);
        if top < bottom && bottom <= self.rows {
            self.explicit_region = Some(ScrollRegion {
                top: top - 1,
                bottom: bottom - 1,
            });
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
    /// `#{history_bytes}` and `#{history_all_bytes}`: the byte totals of the C
    /// structures tmux would allocate for this grid. The struct sizes are
    /// tmux's on LP64 — `grid_line` (40), packed `grid_cell_entry` (5) and
    /// packed `grid_extd_entry` (23) — because the variables report those
    /// allocations, not hmux's own storage.
    pub(crate) fn history_byte_formats(&self) -> io::Result<(String, String)> {
        const GRID_LINE_BYTES: usize = 40;
        const GRID_CELL_ENTRY_BYTES: usize = 5;
        const GRID_EXTD_ENTRY_BYTES: usize = 23;
        let term = self.term.borrow_mut();
        let dims = term.grid_dims();
        let grid = term.grid_snapshot_range(0, dims.total_rows);
        let lines = dims.total_rows;
        let cells: usize = grid.rows.iter().map(|row| row.size).sum();
        let extended: usize = grid.rows.iter().map(|row| row.extd).sum();
        let total = lines * GRID_LINE_BYTES
            + cells * GRID_CELL_ENTRY_BYTES
            + extended * GRID_EXTD_ENTRY_BYTES;
        Ok((
            total.to_string(),
            format!(
                "{lines},{},{cells},{},{extended},{}",
                lines * GRID_LINE_BYTES,
                cells * GRID_CELL_ENTRY_BYTES,
                extended * GRID_EXTD_ENTRY_BYTES,
            ),
        ))
    }

    fn new(
        term: Rc<RefCell<PaneScreen>>,
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
            redraw_detector: RefCell::new(ScrollRedraw::new(rows)),
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
            title_stack: RefCell::new(Vec::new()),
            palette_queries: RefCell::new(VecDeque::new()),
            // A pane that has produced nothing yet still needs naming once.
            changed: Cell::new(true),
            renames: RefCell::new(VecDeque::new()),
            observer: RefCell::new(Observer::default()),
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
            .unwrap_or_else(|| default_tab_stops(self.columns.get()).into_iter().collect())
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

    pub(crate) fn take_renames(&self) -> Vec<String> {
        self.renames.borrow_mut().drain(..).collect()
    }

    /// The palette questions this pane has for the attached terminal, as
    /// `(index, answer with BEL)` pairs.
    pub(crate) fn take_palette_queries(&self) -> Vec<(u8, bool)> {
        self.palette_queries.borrow_mut().drain(..).collect()
    }

    /// tmux's `wp->flags |= PANE_CHANGED`, for the changes the pane itself does
    /// not see: it became the active pane, or entered a mode.
    pub(crate) fn note_changed(&self) {
        self.changed.set(true);
    }

    /// Whether anything has happened in this pane since the last automatic
    /// rename. Peeked rather than taken, because a rename the interval defers
    /// still has to happen on a later pass.
    pub(crate) fn changed(&self) -> bool {
        self.changed.get()
    }

    pub(crate) fn clear_changed(&self) {
        self.changed.set(false);
    }

    /// Feed one chunk of the pane's output through its tokenizer and apply
    /// everything the parse reports.
    ///
    /// Returns what only the caller can deliver: bytes to write back to the
    /// pane's own input, and questions to forward to the client's terminal.
    ///
    /// Each event is applied at exactly its point in the byte stream, so an
    /// event that reads the cursor — a DSR reply, a tab stop, the cursor DECSET
    /// 1049 saves — sees the screen as it stands there and not as it ends up.
    fn observe_output(&self, pending: &[u8]) -> (Vec<Vec<u8>>, Vec<&'static [u8]>) {
        self.append_control_output(pending);
        let policy = self.output_policy();
        let observed = self.observer.borrow_mut().feed(pending, &policy);
        let tokens = &observed.screen[..];

        let mut replies: Vec<Vec<u8>> = Vec::new();
        let mut queries: Vec<&'static [u8]> = Vec::new();
        {
            let mut terminal = self.term.borrow_mut();
            let mut segment_start = 0usize;
            let mut large_scroll = false;
            for (split_at, event) in observed.events {
                while segment_start < split_at {
                    large_scroll |= self.write_terminal(&mut terminal, &tokens[segment_start]);
                    segment_start += 1;
                }
                self.apply_event(event, &terminal, &policy, &mut replies, &mut queries);
            }
            for token in &tokens[segment_start..] {
                large_scroll |= self.write_terminal(&mut terminal, token);
            }
            self.record_change(large_scroll);
            // The modes are republished from the screen once the whole batch
            // has reached it. Reading them before would report what the pane
            // asked for a read ago, and there is nothing else holding them:
            // the screen's mode word is the only copy.
            self.modes.set(PaneModeSnapshot::of(terminal.modes()));
        }
        (replies, queries)
    }

    /// Whether the pane is part-way through a string sequence, which is when
    /// tmux's five-second ground timer runs.
    fn awaiting_terminator(&self) -> bool {
        self.observer.borrow().awaiting_terminator()
    }

    /// Give up on a string sequence whose terminator never arrived, as tmux's
    /// ground timer does, and report whether there was one.
    ///
    /// `input_ground_timer_callback` reaches `input_reset(ictx, 0)`, which is
    /// more than the tokenizer's half: it also returns the pending cell and the
    /// charset designations to their defaults. Those are the screen's, so they
    /// are reset the way the rest of this file resets screen state — by
    /// synthesizing the sequences that say it. What `input_reset` does to the
    /// DECSC save is not reachable that way and is left alone; nothing the
    /// server reports exposes it.
    fn expire_ground(&self) -> bool {
        if !self.observer.borrow_mut().expire() {
            return false;
        }
        let mut terminal = self.term.borrow_mut();
        for token in hmux_vt::parser::tokenize(b"\x1b[m\x0f\x1b(B\x1b)B") {
            terminal.apply(&token);
        }
        drop(terminal);
        self.record_change(false);
        true
    }

    /// Apply one parse event against the screen as it stands at that point in
    /// the stream. Anything the caller has to deliver is collected instead.
    fn apply_event(
        &self,
        event: VtEvent,
        terminal: &PaneScreen,
        policy: &PaneOutputPolicy,
        replies: &mut Vec<Vec<u8>>,
        queries: &mut Vec<&'static [u8]>,
    ) {
        match event {
            VtEvent::Bell => self.note_bells(1),
            VtEvent::Title(title) => {
                let mut announced = self.announced_title.borrow_mut();
                *announced = Some(title);
            }
            VtEvent::TitlePush => {
                let title = self.announced_title.borrow().clone();
                let mut stack = self.title_stack.borrow_mut();
                // tmux's `screen_push_title` keeps ten and evicts the oldest to
                // make room, so a pane that pushes without popping loses the
                // bottom of its stack rather than the top.
                while stack.len() >= TITLE_STACK_LIMIT {
                    stack.remove(0);
                }
                stack.push(title);
            }
            // `screen_pop_title` on an empty stack leaves the title alone.
            VtEvent::TitlePop => {
                if let Some(title) = self.title_stack.borrow_mut().pop() {
                    *self.announced_title.borrow_mut() = title;
                }
            }
            // `ESC k` renames the window rather than retitling the pane, so it
            // is queued for the server instead of touching `announced_title`.
            VtEvent::Rename(title) => {
                let mut renames = self.renames.borrow_mut();
                // As with the clipboard queue, an application that outruns the
                // server loop loses the excess rather than growing the server.
                // Only the last rename is observable anyway.
                if renames.len() < 16 {
                    renames.push_back(title);
                }
            }
            VtEvent::AlternateScreen(on) => self.alternate_on.set(on),
            VtEvent::SaveAlternateCursor => {
                let (x, y) = terminal.cursor_position();
                self.alternate_saved_x.set(u32::from(x));
                self.alternate_saved_y.set(u32::from(y));
            }
            VtEvent::CursorPositionReport => replies.push(cursor_position_report(terminal)),
            VtEvent::WindowSizeReport(operation) => {
                if let Some(reply) = window_size_report(terminal, operation) {
                    replies.push(reply);
                }
            }
            VtEvent::DecPrivateModeReport(mode) => {
                let status = dec_mode_status(
                    terminal.modes(),
                    self.alternate_on.get(),
                    PaneCursorShape::from_parameter(self.cursor_shape.get()),
                    policy.cursor_style,
                    mode,
                );
                replies.push(format!("\x1b[?{mode};{status}$y").into_bytes());
            }
            VtEvent::DecModeReport(mode) => {
                let status = ansi_mode_status(terminal.modes(), mode);
                replies.push(format!("\x1b[{mode};{status}$y").into_bytes());
            }
            VtEvent::StatusReport(request) => replies.push(decrqss_reply(
                &request,
                PaneCursorShape::from_parameter(self.cursor_shape.get()),
                terminal.modes() & mode::CURSOR_BLINKING != 0,
                policy.cursor_style,
            )),
            VtEvent::SetTabStop => {
                let (x, _) = terminal.cursor_position();
                self.update_tab_stops(|stops| {
                    stops.insert(x);
                });
            }
            VtEvent::ClearTabStop => {
                let (x, _) = terminal.cursor_position();
                self.update_tab_stops(|stops| {
                    stops.remove(&x);
                });
            }
            VtEvent::ClearAllTabStops => self.update_tab_stops(BTreeSet::clear),
            VtEvent::CursorShape(shape) => self.cursor_shape.set(shape),
            VtEvent::Reply(reply) => replies.push(reply),
            VtEvent::ColourQuery { number, end } => {
                let stored = match number {
                    10 => &self.foreground,
                    11 => &self.background,
                    _ => &self.cursor_colour,
                };
                match colour_query_reply(number, &stored.borrow(), end) {
                    Some(reply) => replies.push(reply),
                    // With no colour of its own, tmux answers a background
                    // question from the attached terminal's; ask it. An unset
                    // foreground or cursor colour has no such fallback here, so
                    // the question goes unanswered as tmux leaves it when no
                    // client offers one.
                    None if number == 11 => queries.push(BACKGROUND_COLOR_QUERY),
                    None => {}
                }
            }
            VtEvent::ForwardQuery(query) => queries.push(query),
            VtEvent::PaletteQuery { index, end } => {
                let mut queued = self.palette_queries.borrow_mut();
                // Bounded like the clipboard queue: a pane that outruns the
                // server loop loses the excess rather than growing the server.
                if queued.len() < 16 {
                    queued.push_back((index, end == StringEnd::Bell));
                }
            }
            VtEvent::Osc(update) => {
                let slot_value = match update {
                    OscUpdate::Background(colour) => Some((&self.background, colour)),
                    OscUpdate::Foreground(colour) => Some((&self.foreground, colour)),
                    OscUpdate::CursorColour(colour) => Some((&self.cursor_colour, colour)),
                    OscUpdate::Path(path) => Some((&self.path, path)),
                    OscUpdate::ProgressBar { state, progress } => {
                        self.progress_state.set(state);
                        if let Some(progress) = progress {
                            self.progress_value.set(progress);
                        }
                        None
                    }
                };
                if let Some((slot, value)) = slot_value {
                    let mut current = slot.borrow_mut();
                    *current = value;
                }
            }
            VtEvent::Clipboard(event) => self.note_clipboard_event(event),
            VtEvent::Passthrough(data) => self.note_passthrough(PanePassthrough {
                data,
                invisible_panes: policy.passthrough == PassthroughPolicy::Always,
            }),
            VtEvent::ThemeQuery => self.theme_query.set(true),
        }
    }

    fn note_bells(&self, count: u64) {
        if count != 0 {
            self.bell_count
                .set(self.bell_count.get().wrapping_add(count));
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
        self.changed.set(true);
        if let Some(timing) = self.output_timing.as_ref() {
            {
                let mut at = timing.last_at.borrow_mut();
                *at = Some(Instant::now());
            }
        }
        let revision = self.revision.get().wrapping_add(1);
        self.revision.set(revision);
        if large_scroll {
            self.large_scroll_revision.set(revision);
        }
        self.notify_output();
    }

    /// Apply one token to the screen, reporting whether it scrolled enough of
    /// the pane that the compositor should repaint the whole thing.
    ///
    /// The cursor is read *before* the token applies, because a scroll is only
    /// visible where the cursor already was.
    fn write_terminal(&self, terminal: &mut PaneScreen, token: &Token) -> bool {
        let action = self.redraw_detector.borrow_mut().scan(token);
        let large_scroll = action
            .is_some_and(|action| action.needs_large_redraw(terminal.cursor_position().1));
        terminal.apply(token);
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
        Ok(trailing_lines(&plain_dump(&terminal, false), max_rows))
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
            ScreenSource::Recent => plain_dump(&term, false),
            ScreenSource::RecentUnwrapped => plain_dump(&term, true),
            ScreenSource::Visible => {
                // The plain dump is history-first; the viewport is the tail after
                // the scrollback rows. Drop history so only the on-screen rows
                // remain.
                let dims = term.grid_dims();
                let dump = term.dump_plain_rows(0, dims.total_rows, false);
                drop_leading_lines(&dump, dims.scrollback_rows)
            }
        };
        // Writers advance the revision while holding the same terminal lock,
        // so the formatted text, cursor state, and revision form one coherent
        // snapshot.
        let revision = self.revision.get();
        let cursor_visible = term.cursor_visible();
        let cursor_shape = self.cursor_shape.get();
        Ok(ScreenTail {
            revision,
            text: trailing_lines(&text, lines),
            cursor_visible,
            cursor_shape,
        })
    }

    fn scrollback_rows(&self) -> io::Result<usize> {
        Ok(self.term.borrow_mut().grid_dims().scrollback_rows)
    }

    fn title(&self) -> io::Result<Option<String>> {
        Ok(self.announced_title())
    }
}

impl Pane {
    /// A pane with a screen but no process. Useful as a lightweight session
    /// placeholder and for feeding synthetic bytes in tests.
    pub fn inert(cols: u16, rows: u16) -> io::Result<Pane> {
        let term = PaneScreen::new(cols, rows);
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

        let term = PaneScreen::new(cols, rows);
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
                spawn_spec: PaneSpawnSpec {
                    argv: argv.iter().map(|arg| (*arg).to_string()).collect(),
                    cwd: cwd.map(Path::to_path_buf),
                },
                alive,
                reaped: false,
                termination_requested: false,
                exit_code: None,
                death: None,
            }),
            pending_input,
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
        self.child.as_ref().map(|child| child.spawn_spec.clone())
    }

    /// The directory the pane was spawned in (`#{pane_start_path}`), when one
    /// was chosen explicitly. `None` means the pane inherited the server's own
    /// working directory, which is what the caller reports instead.
    pub(crate) fn start_path(&self) -> Option<&Path> {
        self.child.as_ref()?.spawn_spec.cwd.as_deref()
    }

    pub(crate) fn spawn_from_spec(spec: &PaneSpawnSpec, cols: u16, rows: u16) -> io::Result<Pane> {
        let argv = spec.argv.iter().map(String::as_str).collect::<Vec<_>>();
        Self::spawn(&argv, spec.cwd.as_deref(), cols, rows)
    }

    pub(crate) fn runtime_id(&self) -> u64 {
        self.runtime_id
    }

    pub(crate) fn take_event_io(&mut self) -> Option<PaneIo> {
        self.event_io.take()
    }

    /// Give back the loop registration [`Self::take_event_io`] handed out, for
    /// a pane that changes owner without its child restarting.
    pub(crate) fn restore_event_io(&mut self, io: PaneIo) {
        self.event_io = Some(io);
    }

    pub(crate) fn pipe_active(&self) -> bool {
        self.pipe.as_ref().is_some_and(|pipe| pipe.alive.get())
    }

    /// The pid of the process the pane forked for its pty (`#{pane_pid}`).
    pub(crate) fn child_pid(&self) -> Option<pid_t> {
        self.child.as_ref().map(|child| child.pid)
    }

    /// The slave device for the pane's live PTY (`#{pane_tty}`).
    pub(crate) fn tty_name(&self) -> Option<String> {
        let master = self.child.as_ref()?.master.as_raw_fd();
        let name = unsafe { libc::ptsname(master) };
        if name.is_null() {
            return None;
        }
        Some(
            unsafe { CStr::from_ptr(name) }
                .to_string_lossy()
                .into_owned(),
        )
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
        // An inert pane has no child to answer, so the replies and forwarded
        // questions the parse produces have nowhere to go.
        let _ = self.observation.observe_output(bytes);
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

    pub(crate) fn encode_mouse(&self, event: MouseEvent) -> Vec<u8> {
        self.observation.term.borrow_mut().encode_mouse(event)
    }

    /// Reset the emulated terminal state without sending bytes to the child.
    pub(crate) fn reset_terminal(&self) -> io::Result<()> {
        let mut terminal = self.observation.term.borrow_mut();
        for token in hmux_vt::parser::tokenize(b"\x1bc") {
            self.observation.write_terminal(&mut terminal, &token);
        }
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
        if stats.queued != 0 {}
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
            fallback_command: Some(stringify_argv(&child.spawn_spec.argv)),
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

    /// Publish the options the pane's *screen* consults; see [`ScreenOptions`].
    /// Like the output policy, these are pushed rather than looked up, and for
    /// the same reason: the screen has no view of the option tables.
    pub(crate) fn set_screen_options(&self, options: ScreenOptions) {
        self.observation.term.borrow_mut().set_options(options);
    }

    /// Apply the session's history cap to the pane's primary scrollback.
    pub(crate) fn set_history_limit(&self, limit: usize) {
        self.observation.term.borrow_mut().set_history_limit(limit);
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
            t.resize(cols, rows);
            {
                let mut detector = self.observation.redraw_detector.borrow_mut();
                detector.resize(rows);
            }
        }
        // tmux's screen_resize lays the default tab stops out afresh, dropping
        // whatever the pane had set.
        self.observation.columns.set(cols);
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
    pub fn dump(&self) -> String {
        plain_dump(&self.observation.term.borrow_mut(), false)
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

    pub(crate) fn cursor_position(&self) -> (u16, u16) {
        self.observation.term.borrow_mut().cursor_position()
    }

    pub(crate) fn copy_snapshot(&self) -> (Grid, Vec<u8>, (u16, u16)) {
        let terminal = self.observation.term.borrow_mut();
        let total = terminal.grid_dims().total_rows;
        (
            terminal.grid_snapshot_range(0, total),
            terminal.dump_vt_rows(0, total),
            terminal.cursor_position(),
        )
    }

    /// Row geometry of the grid without the per-cell snapshot walk.
    pub(crate) fn grid_dims(&self) -> GridDims {
        self.observation.term.borrow_mut().grid_dims()
    }

    /// Snapshot only physical rows `[start, start + count)`; see
    /// [`PaneScreen::grid_snapshot_range`].
    pub(crate) fn grid_snapshot_range(&self, start: usize, count: usize) -> Grid {
        self.observation
            .term
            .borrow_mut()
            .grid_snapshot_range(start, count)
    }

    /// `resize-pane -T`; see [`PaneScreen::trim_history_below_cursor`].
    pub(crate) fn trim_history_below_cursor(&self) {
        self.observation
            .term
            .borrow_mut()
            .trim_history_below_cursor();
    }

    /// The screen the alternate-screen switch displaced, which
    /// `capture-pane -a` reads, or `None` when the pane is not on an alternate
    /// screen. The VT half is the `-e` serialization of the same rows.
    pub(crate) fn inactive_snapshot(&self) -> Option<(Grid, Vec<u8>)> {
        self.observation.term.borrow_mut().inactive_snapshot()
    }

    /// Bytes of an escape sequence the pane's tokenizer has not finished, which
    /// `capture-pane -P` returns. This is the pane's one parser, so the answer
    /// is the same framing everything else on this pane was decided by.
    pub(crate) fn pending_input(&self) -> Vec<u8> {
        self.observation.observer.borrow().pending().to_vec()
    }

    pub(crate) fn background_color(&self) -> String {
        self.observation.background.borrow().clone()
    }

    /// The current screen as VT escape sequences, suitable for writing to a
    /// client tty. This is the compositor primitive: the pane's grid is
    /// formatted as VT and sent to the attached client's terminal.
    pub fn dump_vt(&self) -> Vec<u8> {
        let terminal = self.observation.term.borrow_mut();
        let total = terminal.grid_dims().total_rows;
        terminal.dump_vt_rows(0, total)
    }

    /// Rows as `capture-pane -e` wants them, which is not the same read as the
    /// compositor's: see [`PaneScreen::dump_vt_capture_rows`].
    pub(crate) fn dump_rows_vt(&self, start: usize, rows: usize, extent: CaptureExtent) -> Vec<u8> {
        self.observation
            .term
            .borrow_mut()
            .dump_vt_capture_rows(start, rows, extent)
    }

    /// One physical row as trimmed plain text, without formatting the rest of
    /// the grid.
    pub(crate) fn dump_plain_row(&self, row: usize) -> String {
        self.observation
            .term
            .borrow_mut()
            .dump_plain_rows(row, 1, false)
    }

    /// Format only the rows visible at a copy-mode scroll offset. Returning
    /// the clamped offset lets the compositor decide whether the live cursor
    /// belongs in the selected viewport.
    pub fn dump_viewport_vt(&self, scroll_offset: usize, visible_rows: usize) -> (Vec<u8>, usize) {
        let terminal = self.observation.term.borrow_mut();
        let scrollback = terminal.grid_dims().scrollback_rows;
        let scroll = scroll_offset.min(scrollback);
        let start = scrollback - scroll;
        // A client whose terminal is taller than this pane asks for rows the
        // grid does not have, and reaching past the last one used to fail the
        // whole dump — which left such a client with a frame-less, permanently
        // blank screen. Serve the rows that exist and let the compositor erase
        // the rest, so the window is drawn at the top as tmux draws it. tmux
        // pads the remainder with the pane-border fill rather than blanks.
        let available = scroll.saturating_add(usize::from(self.rows));
        let vt = terminal.dump_vt_rows(start, visible_rows.min(available));
        (vt, scroll)
    }

    /// The images anchored to this pane's visible screen, oldest first.
    ///
    /// Separate from [`Self::dump_viewport_vt`] because an image is drawn
    /// *over* the cells rather than being some of them: tmux paints the pane's
    /// text first and then walks the same list in `tty_draw_images`.
    pub(crate) fn images(&self) -> Vec<ScreenImage> {
        self.observation.term.borrow_mut().images()
    }

    /// How many scrollback (history) rows the grid holds above the visible
    /// viewport. Consumers that render only the on-screen rows (the compositor,
    /// `capture-pane -p`) skip this many leading rows of a dump.
    pub fn scrollback_rows(&self) -> usize {
        self.observation.term.borrow_mut().grid_dims().scrollback_rows
    }

    /// Clear scrollback while preserving the visible viewport.
    ///
    /// CSI 3 J is the emulator's own erase-scrollback operation, so this keeps
    /// the terminal engine authoritative instead of reconstructing its grid
    /// in hmux.
    pub fn clear_history(&self) -> io::Result<()> {
        let mut terminal = self.observation.term.borrow_mut();
        for token in hmux_vt::parser::tokenize(b"\x1b[3J") {
            self.observation.write_terminal(&mut terminal, &token);
        }
        self.observation.record_change(false);
        Ok(())
    }

    /// Whether the pane's cursor is visible (DEC mode 25). The compositor
    /// mirrors this onto the client tty so a TUI that hides the cursor and
    /// paints its own doesn't leave the client's real cursor lit on top.
    pub fn cursor_visible(&self) -> bool {
        self.observation.term.borrow_mut().cursor_visible()
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
        self.child.as_ref().is_some_and(|child| child.alive.get())
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
        self.child.as_ref().is_some_and(|child| !child.alive.get())
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

/// Upper bound on how much currently-readable pane output is consumed in one
/// readiness turn. Draining without waiting batches an already-queued burst
/// like an event loop, without adding a timer or delaying interactive echo.
/// Each read chunk is parsed as it arrives, so a reply to a query is written
/// back at its chunk boundary rather than held for the rest of the turn.
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
            closed: false,
        })
    }

    /// Whether the pane's parser is waiting for a string terminator, which is
    /// when tmux arms its five-second ground timer.
    pub(crate) fn awaiting_terminator(&self) -> bool {
        self.observation.awaiting_terminator()
    }

    /// The ground timer fired: abandon the sequence whose terminator never came
    /// so the output that follows reaches the screen.
    pub(crate) fn expire_ground(&self) {
        self.observation.expire_ground();
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
        let mut consumed = 0usize;
        let mut reached_eof = false;
        while consumed < OUTPUT_COALESCE_MAX_BYTES {
            let read = unsafe {
                libc::read(
                    self.fd.as_raw_fd(),
                    buffer.as_mut_ptr() as *mut c_void,
                    buffer.len(),
                )
            };
            if read > 0 {
                consumed += read as usize;
                // Parsed chunk by chunk, not batched for the turn: a reply this
                // chunk provokes must reach the pane before the pane can stop
                // repainting over its own echo of it.
                self.process_output(&buffer[..read as usize]);
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

        let continuation = consumed >= OUTPUT_COALESCE_MAX_BYTES;
        if reached_eof {
            self.close();
        }
        Ok(PaneIoReadResult {
            continuation: !self.closed && continuation,
            closed: self.closed,
        })
    }

    fn process_output(&mut self, pending: &[u8]) {
        if self.pipe_output_active.get() {
            let mut outbound = self.pipe_output.borrow_mut();
            if outbound.closed {
                drop(outbound);
                self.pipe_output_active.set(false);
            } else {
                outbound.push(pending);
            }
        }

        let (replies, queries) = self.observation.observe_output(pending);

        if !queries.is_empty() {
            let mut queued = self.terminal_queries.borrow_mut();
            for query in queries {
                if queued.len() == 16 {
                    break;
                }
                queued.push_back(query.to_vec());
            }
        }
        for reply in replies {
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

/// The whole grid as plain text — the full row range, which is how every
/// whole-screen reader spells it now that the screen takes one.
fn plain_dump(terminal: &PaneScreen, join_wraps: bool) -> String {
    terminal.dump_plain_rows(0, terminal.grid_dims().total_rows, join_wraps)
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

/// tmux's `screen_reset_tabs`: a stop every eight columns, skipping column 0.
fn default_tab_stops(columns: u16) -> BTreeSet<u16> {
    (1..)
        .map(|multiple| multiple * 8)
        .take_while(|stop| *stop < columns)
        .collect()
}

/// tmux's DECRQM answer for a private mode: 1 set, 2 reset, 0 unrecognized,
/// or 4 permanently reset.
///
/// The alternate-screen aliases are state rather than bits in the screen's
/// mode word, so their status comes from the pane's screen-tracking state.
///
/// Mode 12 is the one answer that is not a mode-word read. tmux reports the
/// blink the pane asked for only once the pane has spoken — a DECSCUSR that
/// moved the cursor off its default style, or a DECSET/DECRST 12 — and
/// otherwise answers from the `cursor-style` option, whose blinking choices
/// are the odd parameters.
fn dec_mode_status(
    modes: u32,
    alternate_on: bool,
    cursor_shape: PaneCursorShape,
    cursor_style: u8,
    mode: u32,
) -> u8 {
    let reports = |bit: u32| Some(modes & bit != 0);
    let set = match mode {
        // DECCOLM is recognized by tmux but permanently reset: hmux has no
        // 132-column screen mode to enable either.
        3 => return 4,
        1 => reports(mode::KCURSOR),
        6 => reports(mode::ORIGIN),
        7 => reports(mode::WRAP),
        12 => Some(
            if cursor_shape != PaneCursorShape::Default
                || modes & mode::CURSOR_BLINKING_SET != 0
            {
                modes & mode::CURSOR_BLINKING != 0
            } else {
                matches!(cursor_style, 1 | 3 | 5)
            },
        ),
        25 => reports(mode::CURSOR),
        47 | 1047 | 1049 => Some(alternate_on),
        1000 => reports(mode::MOUSE_STANDARD),
        1002 => reports(mode::MOUSE_BUTTON),
        1003 => reports(mode::MOUSE_ALL),
        1004 => reports(mode::FOCUSON),
        1005 => reports(mode::MOUSE_UTF8),
        1006 => reports(mode::MOUSE_SGR),
        2004 => reports(mode::BRACKETPASTE),
        2026 => reports(mode::SYNC),
        2031 => reports(mode::THEME_UPDATES),
        _ => None,
    };
    match set {
        Some(true) => 1,
        Some(false) => 2,
        None => 0,
    }
}

/// tmux's DECRQM answer for a standard mode: 1 set, 2 reset, 0 unrecognized.
fn ansi_mode_status(modes: u32, mode: u32) -> u8 {
    match mode {
        4 if modes & mode::INSERT != 0 => 1,
        4 => 2,
        _ => 0,
    }
}

/// The reportable VT modes a pane's byte stream has set, in their own types.
///
/// This is a *reading* of the screen's mode word, not a second copy of it: the
/// screen applies every DECSET the pane sends and this projects the bits the
/// server cares about into the shapes it wants them in. The observation
/// republishes it at the end of each output batch.
#[derive(Clone, Copy)]
pub(crate) struct PaneModeSnapshot {
    pub(crate) cursor_visible: bool,
    /// DECSET 2004: the pane wants the paste markers.
    pub(crate) bracketed_paste: bool,
    /// DECSET 1004: the pane asked to be told when focus moves.
    pub(crate) focus_reporting: bool,
    /// DECSET 2031: the pane asked to be told when the theme changes.
    pub(crate) theme_updates: bool,
    /// The pane program's mouse reporting mode, if any. tmux keeps 1000/1002/
    /// 1003 mutually exclusive — each one clears the others — and tracks the
    /// two encoding modes independently.
    pub(crate) mouse_tracking: Option<MouseTrackingMode>,
    /// DECSET 1005: UTF-8 coordinate encoding.
    pub(crate) mouse_utf8: bool,
    /// DECSET 1006: SGR encoding.
    pub(crate) mouse_sgr: bool,
    /// IRM (`CSI 4 h`): typed cells shift the rest of the line right.
    pub(crate) insert_mode: bool,
    /// DECOM (DECSET 6): cursor addressing is relative to the scroll region.
    pub(crate) origin_mode: bool,
    /// DECAWM (DECSET 7): text wraps at the right margin. On by default.
    pub(crate) wrap_mode: bool,
    /// tmux's `MODE_CURSOR_BLINKING`, written both by DECSET/DECRST 12 and, as
    /// a side effect, by every DECSCUSR style except the default one.
    pub(crate) cursor_blinking: bool,
    /// DECCKM (DECSET 1): the pane wants the application cursor key forms.
    pub(crate) cursor_keys: bool,
    /// DECKPAM (`ESC =`): the pane wants the application keypad forms.
    pub(crate) application_keypad: bool,
    /// The `modifyOtherKeys` level the pane asked for with `CSI > 4 ; n m`.
    /// What it *gets* also depends on the `extended-keys` option, which is
    /// applied where the key is encoded rather than here.
    pub(crate) extended_keys_request: ExtendedKeys,
    /// DECSET 2026, tmux's `MODE_SYNC`: the pane asked for its output to be
    /// held back until it says the frame is done.
    pub(crate) synchronized_output: bool,
}

impl PaneModeSnapshot {
    /// Read a screen's mode word.
    pub(crate) fn of(modes: u32) -> PaneModeSnapshot {
        let on = |bit: u32| modes & bit != 0;
        PaneModeSnapshot {
            cursor_visible: on(mode::CURSOR),
            bracketed_paste: on(mode::BRACKETPASTE),
            focus_reporting: on(mode::FOCUSON),
            theme_updates: on(mode::THEME_UPDATES),
            // The three tracking modes are mutually exclusive in the word, so
            // at most one of these matches.
            mouse_tracking: if on(mode::MOUSE_ALL) {
                Some(MouseTrackingMode::All)
            } else if on(mode::MOUSE_BUTTON) {
                Some(MouseTrackingMode::Button)
            } else if on(mode::MOUSE_STANDARD) {
                Some(MouseTrackingMode::Standard)
            } else {
                None
            },
            mouse_utf8: on(mode::MOUSE_UTF8),
            mouse_sgr: on(mode::MOUSE_SGR),
            insert_mode: on(mode::INSERT),
            origin_mode: on(mode::ORIGIN),
            wrap_mode: on(mode::WRAP),
            cursor_blinking: on(mode::CURSOR_BLINKING),
            cursor_keys: on(mode::KCURSOR),
            application_keypad: on(mode::KKEYPAD),
            extended_keys_request: if on(mode::KEYS_EXTENDED_2) {
                ExtendedKeys::All
            } else if on(mode::KEYS_EXTENDED) {
                ExtendedKeys::Standard
            } else {
                ExtendedKeys::Off
            },
            synchronized_output: on(mode::SYNC),
        }
    }
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
/// variables, tracked from the pane's byte stream beside the screen.
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

fn cursor_position_report(terminal: &PaneScreen) -> Vec<u8> {
    let (x, y) = terminal.cursor_position();
    format!("\x1b[{};{}R", y.saturating_add(1), x.saturating_add(1)).into_bytes()
}

/// tmux's default window pixel geometry when no attached client reports one.
const DEFAULT_CELL_WIDTH: u32 = 16;
const DEFAULT_CELL_HEIGHT: u32 = 32;

fn window_size_report(terminal: &PaneScreen, operation: u32) -> Option<Vec<u8>> {
    let dims = terminal.grid_dims();
    let columns = u32::from(dims.cols);
    let rows = u32::from(dims.viewport_rows);
    let (report, height, width) = match operation {
        14 => (4, rows * DEFAULT_CELL_HEIGHT, columns * DEFAULT_CELL_WIDTH),
        15 => (5, rows * DEFAULT_CELL_HEIGHT, columns * DEFAULT_CELL_WIDTH),
        16 => (6, DEFAULT_CELL_HEIGHT, DEFAULT_CELL_WIDTH),
        18 => (8, rows, columns),
        19 => (9, rows, columns),
        _ => return None,
    };
    Some(format!("\x1b[{report};{height};{width}t").into_bytes())
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

    /// Whether the pane's own shell holds the terminal, meaning it is sitting
    /// at a prompt with nothing running in front of it.
    ///
    /// The session leader is the process the pane forked; while it is also the
    /// foreground group, nothing it launched has taken the terminal from it.
    /// Running a command moves the foreground group to that command, and it
    /// moves back when the command finishes.
    ///
    /// That test alone is not enough, because a pane launched straight into a
    /// program (`new-window -- tail -f log`) has that program as leader *and*
    /// foreground group, and a prompt is the one thing it is not. So the
    /// program has to be a shell, and a shell handed a command to run
    /// (`sh -c 'while :; do :; done'`) is working rather than prompting.
    ///
    /// What this cannot do is ask the shell whether it is currently at its
    /// prompt: bash and dash sit in a `poll` loop there, which is
    /// indistinguishable from any other wait. Reading the invocation is what
    /// is left, and it is right for the panes that matter — a login shell has
    /// no `-c`, and a shell given one never returns to a prompt.
    fn shell_at_prompt(&self) -> bool {
        let holds_terminal = match (self.foreground, self.session_leader) {
            (Some(foreground), Some(leader)) => foreground == leader,
            _ => false,
        };
        holds_terminal
            && self
                .current_command()
                .is_some_and(|command| is_shell(&command))
            && !self.runs_a_command_string()
    }

    /// Whether the foreground process was invoked with `-c`, i.e. handed a
    /// command to run rather than started interactively.
    ///
    /// An unreadable argument vector reads as no `-c`, leaving the shell to be
    /// treated as interactive — which is what a pane's shell usually is.
    fn runs_a_command_string(&self) -> bool {
        let Some(pid) = self.foreground else {
            return false;
        };
        let arguments = CurrentPlatform::process_arguments(pid as u32);
        invoked_with_command_string(arguments.iter().filter_map(|argument| argument.to_str()))
    }

    /// Whether the foreground command is parked in a terminal read.
    ///
    /// The foreground process *group* id is a pid only while the group's
    /// leader lives, so this answers for the leader and reads a leaderless
    /// group — a pipeline outliving its first member — as not waiting. Which
    /// is the conservative direction: it never claims a pane wants you.
    fn waiting_for_tty(&self) -> bool {
        self.foreground
            .and_then(|pid| CurrentPlatform::process_waiting_for_tty(pid as u32))
            .unwrap_or(false)
    }
}

/// Whether a shell's argument vector carries `-c`.
///
/// Only the leading option arguments are scanned: everything from the first
/// non-option onwards is the command string and its own arguments, which may
/// contain anything at all. `--` ends the options.
///
/// Within that, only single-dash arguments are short-option bundles, so `-lc`
/// counts while a long option merely spelled with a `c` in it — `--norc` —
/// does not.
fn invoked_with_command_string<'a>(arguments: impl Iterator<Item = &'a str>) -> bool {
    arguments
        .skip(1)
        .take_while(|argument| argument.starts_with('-') && *argument != "--")
        .filter(|argument| !argument.starts_with("--"))
        .any(|argument| argument.contains('c'))
}

/// Whether a program name — as [`parse_window_name`] reduces it, so already
/// stripped of its path and of the leading `-` a login shell carries — is an
/// interactive shell.
///
/// Used only to decide whether a pane holding its own terminal is at a prompt.
/// An unlisted shell reads as a foreground program instead, which is the same
/// answer the pane would give while running anything else.
fn is_shell(command: &str) -> bool {
    const SHELLS: [&str; 13] = [
        "sh", "bash", "zsh", "fish", "dash", "ash", "ksh", "mksh", "csh", "tcsh", "elvish", "nu",
        "xonsh",
    ];
    SHELLS.contains(&command)
}

/// What a pane is doing, as the compact `#{pane_state_emoji}` label reports it.
///
/// This is the non-agent half of that variable: a pane running a recognized
/// agent reports the agent's lifecycle state instead. Every other pane lands in
/// exactly one of these, which is what keeps the variable from ever being
/// empty.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaneClass {
    /// No live child: reaped with `remain-on-exit` holding the pane open, or
    /// never started.
    Dead,
    /// The pane is on its alternate screen, which is what a full-screen
    /// application switches to and an ordinary command line never does.
    Tui,
    /// The pane's shell is at its prompt, waiting for you.
    ShellPrompt,
    /// A foreground command has stopped to read from the terminal, waiting for
    /// you.
    WaitingForTty,
    /// A foreground command is working.
    Running,
}

impl PaneClass {
    /// Classify a pane from what it is running.
    ///
    /// Order matters. A dead pane is dead whatever its last screen was. The
    /// alternate screen then outranks the shell/command split, because a pane
    /// launched straight into a full-screen application (`new-window htop`)
    /// has that application as *both* its session leader and its foreground
    /// group, and would otherwise read as a shell sitting at a prompt.
    pub(crate) fn classify(
        probe: Option<&PaneProcessProbe>,
        alternate_on: bool,
        dead: bool,
    ) -> Self {
        let Some(probe) = probe.filter(|_| !dead) else {
            return PaneClass::Dead;
        };
        if alternate_on {
            PaneClass::Tui
        } else if probe.shell_at_prompt() {
            PaneClass::ShellPrompt
        } else if probe.waiting_for_tty() {
            PaneClass::WaitingForTty
        } else {
            PaneClass::Running
        }
    }

    /// The status-bar glyph for this class.
    ///
    /// Every glyph here — and every one [`AgentState::emoji`] returns — is a
    /// single codepoint two columns wide. That is a hard requirement, not a
    /// preference: the status renderer measures per codepoint, so a glyph
    /// needing U+FE0F to reach emoji presentation would be counted as one
    /// column while the terminal drew two, and the whole status line would
    /// drift. Check any replacement against `codepoint_width` first.
    ///
    /// [`AgentState::emoji`]: crate::integration::AgentState::emoji
    pub(crate) fn emoji(self) -> &'static str {
        match self {
            PaneClass::Dead => "🛑",
            PaneClass::Tui => "🪟",
            PaneClass::ShellPrompt => "💲",
            PaneClass::WaitingForTty => "⌛",
            PaneClass::Running => "🔧",
        }
    }
}

/// Build tmux's OSC 10 / 11 / 12 reply from the pane-local colour state. An
/// unset colour — spelled `default` for the foreground and background, `none`
/// for the cursor — parses as no colour and so has no reply, which is where
/// tmux's own fallbacks take over.
fn colour_query_reply(number: u32, colour: &str, end: StringEnd) -> Option<Vec<u8>> {
    let packed = parse_packed_colour(colour)?;
    let (r, g, b) = ((packed >> 16) as u8, (packed >> 8) as u8, packed as u8);
    let terminator = match end {
        StringEnd::StringTerminator => "\x1b\\",
        StringEnd::Bell => "\x07",
    };
    Some(
        format!("\x1b]{number};rgb:{r:02x}{r:02x}/{g:02x}{g:02x}/{b:02x}{b:02x}{terminator}")
            .into_bytes(),
    )
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

    /// Every glyph the status bar can put in a pane's slot has to occupy the
    /// same two columns, or window entries stop lining up as panes change
    /// state. Two things can break that, and this covers both: a glyph the
    /// width table calls one column, and a glyph that only reaches emoji
    /// presentation through U+FE0F — which the status renderer measures as
    /// zero, leaving the cell one column wide while the terminal draws two.
    #[test]
    fn every_pane_state_glyph_is_exactly_two_columns() {
        use crate::integration::AgentState;
        use hmux_vt::width::codepoint_width;

        let classes = [
            PaneClass::Dead,
            PaneClass::Tui,
            PaneClass::ShellPrompt,
            PaneClass::WaitingForTty,
            PaneClass::Running,
        ]
        .map(PaneClass::emoji);
        let agents = [
            AgentState::Idle,
            AgentState::Working,
            AgentState::Blocked,
            AgentState::Exited,
        ]
        .map(AgentState::emoji);

        for emoji in classes.iter().chain(agents.iter()) {
            let mut codepoints = emoji.chars();
            let first = codepoints.next().expect("a glyph");
            assert_eq!(
                codepoints.next(),
                None,
                "{emoji:?} is more than one codepoint"
            );
            assert_eq!(codepoint_width(first as u32), 2, "{emoji:?} is not 2 columns");
        }

        // Unknown stays empty on purpose: it is what lets the state emoji fall
        // through to the pane's own class instead of labelling it as an agent.
        assert_eq!(AgentState::Unknown.emoji(), "");
    }

    /// A pane with no live child has no foreground group to compare, so the
    /// classes that describe a running process cannot apply to it.
    #[test]
    fn a_pane_with_no_process_is_dead_whatever_its_screen_says() {
        assert_eq!(PaneClass::classify(None, false, false), PaneClass::Dead);
        assert_eq!(PaneClass::classify(None, true, false), PaneClass::Dead);
    }

    #[test]
    fn a_probed_pane_is_classified_by_what_holds_its_terminal() {
        // The pids are deliberately ones `/proc` cannot answer for, so the
        // program name comes from the spawn command the way it does for a
        // leaderless group — and the tty probe reads as "cannot tell", which
        // is also what every non-Linux platform reports.
        let probe = |foreground, session_leader, command: &str| PaneProcessProbe {
            foreground,
            session_leader,
            fallback_command: Some(command.to_string()),
        };

        // The shell still owns the terminal: nothing runs in front of it.
        assert_eq!(
            PaneClass::classify(Some(&probe(Some(0), Some(0), "bash")), false, false),
            PaneClass::ShellPrompt
        );
        // A command took the terminal away from the shell.
        assert_eq!(
            PaneClass::classify(Some(&probe(Some(0), Some(1), "bash")), false, false),
            PaneClass::Running
        );
        // A pane launched straight into a program is its own session leader,
        // so the group comparison alone would call this a prompt.
        assert_eq!(
            PaneClass::classify(Some(&probe(Some(0), Some(0), "tail -f log")), false, false),
            PaneClass::Running
        );
        // The alternate screen outranks the rest: a full-screen program is
        // likewise both leader and foreground group.
        assert_eq!(
            PaneClass::classify(Some(&probe(Some(0), Some(0), "bash")), true, false),
            PaneClass::Tui
        );
        // Death outranks everything, including the screen it died on.
        assert_eq!(
            PaneClass::classify(Some(&probe(Some(0), Some(0), "bash")), true, true),
            PaneClass::Dead
        );
    }

    /// A shell handed a command is working, not prompting — and the scan has
    /// to stop before the command string, which can hold anything.
    #[test]
    fn a_shell_given_a_command_string_is_not_at_a_prompt() {
        let invoked = |arguments: &[&str]| invoked_with_command_string(arguments.iter().copied());

        assert!(invoked(&["sh", "-c", "while :; do :; done"]));
        assert!(invoked(&["bash", "-lc", "make"]));
        assert!(!invoked(&["bash", "--norc", "-i"]));
        assert!(!invoked(&["-bash"]));
        assert!(!invoked(&["zsh"]));
        // The command string is not an option, so a `c` inside it is not a
        // `-c`, and neither is anything after `--`.
        assert!(!invoked(&["sh", "-i", "printf c"]));
        assert!(!invoked(&["sh", "--", "-c"]));
        // Only single-dash arguments bundle short options, so a long option
        // that merely contains a `c` is not one.
        assert!(!invoked(&["bash", "--noprofile", "--norc"]));
        assert!(invoked(&["bash", "--norc", "-c", "make"]));
    }

    #[test]
    fn a_login_shells_argv0_still_reads_as_a_shell() {
        // A login shell is spelled `-bash`, and an absolute path is common in
        // `default-shell`; both reduce to the bare program name.
        assert!(is_shell(&parse_window_name("-bash")));
        assert!(is_shell(&parse_window_name("/usr/bin/zsh")));
        assert!(is_shell(&parse_window_name("fish")));
        assert!(!is_shell(&parse_window_name("/usr/bin/tail -f log")));
        assert!(!is_shell(&parse_window_name("htop")));
    }

    /// A pty read can end anywhere, including in the middle of a sequence. The
    /// tokenizer retains its state across reads, so the pane still sees one
    /// DECSCUSR rather than two fragments of one.
    #[test]
    fn a_decscusr_split_across_reads_still_sets_the_shape() {
        let pane = Pane::inert(20, 4).expect("inert pane");
        pane.feed(b"noise\x1b[");
        assert_eq!(pane.cursor_shape(), 0);
        pane.feed(b"6 q");
        assert_eq!(pane.cursor_shape(), 6);
    }

    #[test]
    fn inert_pane_feeds_grid() {
        let pane = Pane::inert(20, 4).expect("inert pane");
        pane.feed(b"synthetic\r\noutput");
        let dump = pane.dump();
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
        assert!(pane.scrollback_rows() > 0);
        let visible_before = pane.dump_viewport_vt(0, 4).0;

        pane.clear_history().expect("clear history");

        assert_eq!(pane.scrollback_rows(), 0);
        assert_eq!(
            pane.dump_viewport_vt(0, 4).0,
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
        test_pane_io(&pane).process_output(b"\x1b]2;Working (5s)\x07");
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

    /// A BEL that terminates an OSC string is a terminator, not a bell.
    fn inert_observation() -> NativePaneObservation {
        let term = PaneScreen::new(20, 4);
        NativePaneObservation::new(Rc::new(RefCell::new(term)), None, 20, 4)
    }

    #[test]
    fn only_bells_outside_a_string_are_counted() {
        let pane = Pane::inert(20, 4).expect("inert pane");
        pane.feed(b"before\x07\x1b]0;title\x07after\x1b]1;title\x1b\\\x07");
        assert_eq!(pane.observation.bell_count.get(), 2);
    }

    /// The questions only the client's terminal can answer are recognized
    /// whatever terminator they use and wherever the read boundaries fall.
    #[test]
    fn forwarded_queries_survive_pty_boundaries_and_every_terminator() {
        let observation = inert_observation();
        let mut forwarded = 0;
        for chunk in [
            &b"before\x1b]11;?\x1b"[..],
            b"\\after\x1b]11;?\x07",
            b"\x1b[5",
            b"n",
        ] {
            forwarded += observation.observe_output(chunk).1.len();
        }
        assert_eq!(forwarded, 3, "two OSC 11 questions and one DSR");
    }

    /// DECXCPR is not a cursor report tmux answers, so the private form must
    /// pass through without one.
    #[test]
    fn the_private_cursor_report_is_not_answered() {
        let observation = inert_observation();
        let (replies, _) = observation.observe_output(b"before\x1b[?6n");
        assert!(replies.is_empty(), "got {replies:?}");
        let (replies, _) = observation.observe_output(b"\x1b[6n");
        assert_eq!(replies, vec![b"\x1b[1;7R".to_vec()]);
    }

    #[test]
    fn pane_cursor_colour_query_uses_the_stored_colour_and_terminator() {
        let observation = inert_observation();
        let (replies, queries) = observation.observe_output(b"\x1b]12;#00ff00\x07\x1b]12;?\x07");
        assert!(queries.is_empty(), "got forwarded queries: {queries:?}");
        assert_eq!(replies, vec![b"\x1b]12;rgb:0000/ffff/0000\x07".to_vec()]);
    }

    #[test]
    fn pane_colour_queries_answer_from_the_stored_foreground_and_background() {
        let observation = inert_observation();
        let (replies, queries) = observation
            .observe_output(b"\x1b]10;#ff0000\x07\x1b]11;#104e8b\x07\x1b]10;?\x07\x1b]11;?\x1b\\");
        assert!(queries.is_empty(), "got forwarded queries: {queries:?}");
        assert_eq!(
            replies,
            vec![
                b"\x1b]10;rgb:ffff/0000/0000\x07".to_vec(),
                b"\x1b]11;rgb:1010/4e4e/8b8b\x1b\\".to_vec(),
            ]
        );
    }

    #[test]
    fn an_unset_background_question_still_goes_out_to_the_terminal() {
        let observation = inert_observation();
        let (replies, queries) = observation.observe_output(b"\x1b]10;?\x07\x1b]11;?\x07");
        assert!(replies.is_empty(), "got pane replies: {replies:?}");
        assert_eq!(queries, vec![BACKGROUND_COLOR_QUERY]);
    }
}

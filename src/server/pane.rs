//! A pane: a child process on a PTY, its output parsed by a libghostty-vt
//! [`Terminal`].
//!
//! This is where the "clone" earns its name — instead of proxying to a backing
//! tmux, hmux owns the pty/child and maintains the screen itself. tmux keeps this
//! state in `window_pane` + `screen`/`input.c`; here the grid lives in libghostty
//! and the master fd is drained by the central event loop (or a dedicated
//! reader thread in unit tests).
//!
//! Only a text-emulation slice is implemented: spawn, feed output → grid, send
//! input, resize, dump. Compositing multiple panes onto an attached client's tty
//! is the next milestone (see the module docs).

use std::collections::VecDeque;
use std::ffi::CString;
use std::io::{self, Read, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd};
use std::os::raw::{c_int, c_void};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex, Weak};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use libc::pid_t;

use crate::ghostty::Terminal;
use crate::observability::v1::{PaneObservability, PaneProcess, ScreenSource, ScreenTail};
use crate::platform::{CurrentPlatform, ForkOutcome, OutputWakeup, Platform};


/// A single pane. Holds the emulated screen and, if live, the child on its pty.
pub struct Pane {
    /// Read-only state shared with observation handles. Keeping this separate
    /// from the PTY owner lets consumers inspect a resolved pane without
    /// retaining the native server's global state lock.
    observation: Arc<NativePaneObservation>,
    /// Terminal queries emitted by the child which must be relayed to an
    /// attached outer terminal. Ghostty consumes OSC sequences while updating
    /// the grid, so they need a separate side channel to reach the compositor.
    terminal_queries: Arc<Mutex<VecDeque<Vec<u8>>>>,
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
    pending_input: Arc<Mutex<VecDeque<u8>>>,
    /// Original process specification retained for command-less respawns.
    spawn_spec: Option<PaneSpawnSpec>,
    /// Sender observed by the PTY reader when `pipe-pane -O` is active.
    pipe_output: Arc<Mutex<Option<Sender<Vec<u8>>>>>,
    pipe_output_active: Arc<AtomicBool>,
    pipe: Option<PanePipe>,
    event_io: Option<PaneIo>,
    runtime_id: u64,
    cols: u16,
    rows: u16,
}

#[cfg(test)]
pub(crate) type PaneReaderSpawner = fn(PaneIo) -> JoinHandle<()>;

#[derive(Clone, Copy, Debug)]
pub(crate) enum PaneIoMode {
    /// Drain the pane on a dedicated thread. Unit-test scaffolding only; the
    /// server runtime owns pane I/O through the event loop.
    #[cfg(test)]
    Threaded(PaneReaderSpawner),
    EventLoop,
}

/// Threaded driver for shared nonblocking pane I/O. Unit-test scaffolding for
/// exercising panes without an event loop.
#[cfg(test)]
pub(crate) fn spawn_reader(mut pane_io: PaneIo) -> JoinHandle<()> {
    thread::spawn(move || loop {
        let mut wait = libc::pollfd {
            fd: pane_io.as_fd().as_raw_fd(),
            events: libc::POLLIN
                | if pane_io.wants_write() {
                    libc::POLLOUT
                } else {
                    0
                },
            revents: 0,
        };
        let result = unsafe { libc::poll(&mut wait, 1, -1) };
        if result < 0 {
            if io::Error::last_os_error().kind() == io::ErrorKind::Interrupted {
                continue;
            }
            break;
        }
        if wait.revents & libc::POLLOUT != 0 {
            pane_io.drive_writable();
        }
        if wait.revents & (libc::POLLIN | libc::POLLHUP | libc::POLLERR) == 0 {
            continue;
        }
        match pane_io.drive_readable() {
            Ok(result) if result.closed => break,
            Ok(result) if result.continuation => continue,
            Ok(_) => {}
            Err(_) => break,
        }
    })
}

#[derive(Clone, Debug)]
pub(crate) struct PaneSpawnSpec {
    pub(crate) argv: Vec<String>,
    pub(crate) cwd: Option<PathBuf>,
}

struct PanePipe {
    pid: u32,
    alive: Arc<AtomicBool>,
}

impl Drop for PanePipe {
    fn drop(&mut self) {
        if self.alive.load(Ordering::Acquire) {
            // Closing a pipe is asynchronous. SIGHUP mirrors the lifetime of a
            // tmux job without making the command path wait for the child.
            unsafe {
                libc::kill(self.pid as pid_t, libc::SIGHUP);
            }
        }
    }
}

static NEXT_PANE_RUNTIME_ID: AtomicU64 = AtomicU64::new(1);

/// The live half of a pane: the child pid, the pty master, and the drain thread.
struct Child {
    pid: pid_t,
    master: OwnedFd,
    reader: Option<JoinHandle<()>>,
    alive: Arc<AtomicBool>,
    reaped: bool,
    termination_requested: bool,
    exit_code: Option<i32>,
}

struct ObservedChild {
    pid: u32,
    alive: Arc<AtomicBool>,
}

/// State which remains valid for the lifetime of a resolved observation
/// handle, even if the pane is subsequently removed from the server tree.
pub(crate) struct NativePaneObservation {
    term: Arc<Mutex<Terminal>>,
    revision: AtomicU64,
    /// Revision of the latest scroll operation whose vertical region is large
    /// enough for tmux to prefer one deferred repaint over immediate row draws.
    /// This is monotonic so each attached client can observe it independently.
    large_scroll_revision: AtomicU64,
    redraw_detector: Mutex<ScrollRedrawDetector>,
    control_output: Mutex<ControlOutputJournal>,
    /// Last DECSCUSR parameter emitted by the pane (0..=6). The VT formatter
    /// restores cursor position but does not serialize this terminal state.
    cursor_shape: AtomicU8,
    bracketed_paste: AtomicBool,
    /// Whether the pane asked for focus reporting (DECSET 1004).
    focus_reporting: AtomicBool,
    /// Whether the pane asked for theme updates (DECSET 2031).
    theme_updates: AtomicBool,
    /// The pane's DECSET mouse modes: 0 for none, else 1000/1002/1003, with
    /// 1005 and 1006 as separate flags.
    mouse_tracking_mode: AtomicU8,
    mouse_utf8: AtomicBool,
    mouse_sgr: AtomicBool,
    /// Set when the pane sent DSR ?996 and is waiting for an answer.
    theme_query: AtomicBool,
    background: Mutex<String>,
    child: Option<ObservedChild>,
    output_waiters: Mutex<Vec<Weak<OutputEvent>>>,
    output_timing: Option<Arc<OutputTiming>>,
    last_output_at: Mutex<Option<Instant>>,
    bell_count: AtomicU64,
    /// OSC 52 sequences the pane emitted, waiting for the server to apply the
    /// `set-clipboard`/`get-clipboard` policy to them.
    clipboard_events: Mutex<VecDeque<PaneClipboardEvent>>,
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
    event: Arc<OutputEvent>,
    output_timing: Option<Arc<OutputTiming>>,
}

/// Timestamp side channel used only by the opt-in attach latency monitor.
/// Keeping it beside the wakeup lets the attach thread distinguish time spent
/// waiting for the pane from time spent waiting to be scheduled after output.
struct OutputTiming {
    last_at: Mutex<Option<Instant>>,
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
        let event = Arc::new(OutputEvent {
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
                .map(Arc::clone),
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
        self.output_timing
            .as_ref()?
            .last_at
            .lock()
            .ok()
            .and_then(|at| *at)
    }
}

impl NativePaneObservation {
    fn new(term: Arc<Mutex<Terminal>>, child: Option<ObservedChild>, rows: u16) -> Self {
        let latency_enabled = matches!(
            std::env::var("HMUX_LATENCY"),
            Ok(value) if !value.is_empty() && value != "0"
        );
        Self {
            term,
            revision: AtomicU64::new(0),
            large_scroll_revision: AtomicU64::new(0),
            redraw_detector: Mutex::new(ScrollRedrawDetector::new(rows)),
            control_output: Mutex::new(ControlOutputJournal::default()),
            cursor_shape: AtomicU8::new(0),
            bracketed_paste: AtomicBool::new(false),
            focus_reporting: AtomicBool::new(false),
            theme_updates: AtomicBool::new(false),
            theme_query: AtomicBool::new(false),
            mouse_tracking_mode: AtomicU8::new(0),
            mouse_utf8: AtomicBool::new(false),
            mouse_sgr: AtomicBool::new(false),
            background: Mutex::new("default".to_string()),
            child,
            output_waiters: Mutex::new(Vec::new()),
            output_timing: latency_enabled.then(|| {
                Arc::new(OutputTiming {
                    last_at: Mutex::new(None),
                })
            }),
            last_output_at: Mutex::new(None),
            bell_count: AtomicU64::new(0),
            clipboard_events: Mutex::new(VecDeque::new()),
        }
    }

    pub(crate) fn mouse_modes(&self) -> PaneMouseModes {
        PaneMouseModes {
            tracking: match self.mouse_tracking_mode.load(Ordering::Acquire) {
                1 => Some(MouseTrackingMode::Standard),
                2 => Some(MouseTrackingMode::Button),
                3 => Some(MouseTrackingMode::All),
                _ => None,
            },
            utf8: self.mouse_utf8.load(Ordering::Acquire),
            sgr: self.mouse_sgr.load(Ordering::Acquire),
        }
    }

    fn note_clipboard_event(&self, event: PaneClipboardEvent) {
        if let Ok(mut events) = self.clipboard_events.lock() {
            // A runaway application must not grow this without bound; tmux
            // likewise answers only what it can keep up with.
            if events.len() < 16 {
                events.push_back(event);
            }
        }
    }

    pub(crate) fn take_clipboard_events(&self) -> Vec<PaneClipboardEvent> {
        self.clipboard_events
            .lock()
            .map(|mut events| events.drain(..).collect())
            .unwrap_or_default()
    }

    fn record_output(&self, bytes: &[u8], large_scroll: bool) {
        self.append_control_output(bytes);
        let mut detector = BellDetector::default();
        self.note_bells(bytes.iter().filter(|byte| detector.feed(**byte)).count() as u64);
        self.record_change(large_scroll);
    }

    fn note_bells(&self, count: u64) {
        if count != 0 {
            self.bell_count.fetch_add(count, Ordering::Release);
        }
    }

    fn append_control_output(&self, bytes: &[u8]) {
        if !bytes.is_empty() {
            if let Ok(mut output) = self.control_output.lock() {
                output.append(bytes);
            }
        }
    }

    fn record_change(&self, large_scroll: bool) {
        if let Ok(mut at) = self.last_output_at.lock() {
            *at = Some(Instant::now());
        }
        if let Some(timing) = self.output_timing.as_ref() {
            if let Ok(mut at) = timing.last_at.lock() {
                *at = Some(Instant::now());
            }
        }
        let revision = self.revision.fetch_add(1, Ordering::AcqRel).wrapping_add(1);
        if large_scroll {
            self.large_scroll_revision
                .store(revision, Ordering::Release);
        }
        self.notify_output();
    }

    fn write_terminal(&self, terminal: &mut Terminal, bytes: &[u8]) -> bool {
        let actions = self
            .redraw_detector
            .lock()
            .map(|mut detector| detector.scan(bytes))
            .unwrap_or_default();
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
        let Ok(mut waiters) = self.output_waiters.lock() else {
            return;
        };
        waiters.retain(|waiter| {
            let Some(event) = waiter.upgrade() else {
                return false;
            };
            let _ = event.wakeup.wake();
            true
        });
    }

    fn register_output_event(&self, event: &Arc<OutputEvent>) -> io::Result<()> {
        self.output_waiters
            .lock()
            .map_err(|_| io::Error::other("pane output waiters mutex poisoned"))?
            .push(Arc::downgrade(event));
        Ok(())
    }

    pub(crate) fn subscribe_output(&self) -> io::Result<OutputSubscription> {
        // Start signalled so a subscriber performs one state scan and cannot
        // miss output or a terminal query queued just before registration.
        let event = Arc::new(OutputEvent {
            wakeup: CurrentPlatform::new_output_wakeup()?,
        });
        self.register_output_event(&event)?;
        Ok(OutputSubscription {
            event,
            output_timing: self.output_timing.as_ref().map(Arc::clone),
        })
    }

    pub(crate) fn contract_process(&self) -> (Option<u32>, bool) {
        match &self.child {
            Some(child) => (Some(child.pid), !child.alive.load(Ordering::Acquire)),
            None => (None, false),
        }
    }

    pub(crate) fn contract_revision(&self) -> u64 {
        self.revision.load(Ordering::Acquire)
    }

    pub(crate) fn large_scroll_revision(&self) -> u64 {
        self.large_scroll_revision.load(Ordering::Acquire)
    }

    pub(crate) fn alert_snapshot(&self) -> (u64, u64, Option<Instant>) {
        (
            self.revision.load(Ordering::Acquire),
            self.bell_count.load(Ordering::Acquire),
            self.last_output_at.lock().ok().and_then(|at| *at),
        )
    }

    pub(crate) fn control_output_end(&self) -> u64 {
        self.control_output
            .lock()
            .map(|output| output.end)
            .unwrap_or_default()
    }

    pub(crate) fn control_output_chunk(&self, offset: u64, limit: usize) -> (u64, u64, Vec<u8>) {
        self.control_output
            .lock()
            .map(|output| output.chunk(offset, limit))
            .unwrap_or((offset, offset, Vec::new()))
    }

    #[allow(dead_code)]
    pub(crate) fn contract_title(&self) -> io::Result<Option<String>> {
        self.term
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?
            .title()
            .map_err(ghostty_err)
    }

    /// Return the terminal facts needed by the native observation boundary in
    /// one terminal-lock critical section. Keeping this operation here avoids
    /// a title read and a tail read observing different VT states.
    #[allow(dead_code)]
    pub(crate) fn contract_terminal_tail(
        &self,
        max_rows: usize,
    ) -> io::Result<(Option<String>, String)> {
        let terminal = self
            .term
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?;
        let title = terminal.title().map_err(ghostty_err)?;
        let text = trailing_lines(&terminal.dump_plain().map_err(ghostty_err)?, max_rows);
        Ok((title, text))
    }
}

impl PaneObservability for NativePaneObservation {
    fn process(&self) -> io::Result<PaneProcess> {
        Ok(match &self.child {
            Some(child) => PaneProcess {
                child_pid: Some(child.pid),
                exited: !child.alive.load(Ordering::Acquire),
            },
            None => PaneProcess {
                child_pid: None,
                exited: false,
            },
        })
    }

    fn output_revision(&self) -> io::Result<u64> {
        Ok(self.revision.load(Ordering::Acquire))
    }

    fn screen(&self, source: ScreenSource, lines: usize) -> io::Result<ScreenTail> {
        let term = self
            .term
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?;
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
        let revision = self.revision.load(Ordering::Acquire);
        let cursor_visible = term.cursor_visible().map_err(ghostty_err)?;
        let cursor_shape = self.cursor_shape.load(Ordering::Acquire);
        Ok(ScreenTail {
            revision,
            text: trailing_lines(&text, lines),
            cursor_visible,
            cursor_shape,
        })
    }

    fn scrollback_rows(&self) -> io::Result<usize> {
        self.term
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?
            .scrollback_rows()
            .map_err(ghostty_err)
    }

    fn title(&self) -> io::Result<Option<String>> {
        self.term
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?
            .title()
            .map_err(ghostty_err)
    }
}

impl Pane {
    /// A pane with a screen but no process. Useful as a lightweight session
    /// placeholder and for feeding synthetic bytes in tests.
    pub fn inert(cols: u16, rows: u16) -> io::Result<Pane> {
        let term = Terminal::new(cols, rows).map_err(ghostty_err)?;
        Ok(Pane {
            observation: Arc::new(NativePaneObservation::new(
                Arc::new(Mutex::new(term)),
                None,
                rows,
            )),
            terminal_queries: Arc::new(Mutex::new(VecDeque::new())),
            child: None,
            pending_input: Arc::new(Mutex::new(VecDeque::new())),
            spawn_spec: None,
            pipe_output: Arc::new(Mutex::new(None)),
            pipe_output_active: Arc::new(AtomicBool::new(false)),
            pipe: None,
            event_io: None,
            runtime_id: NEXT_PANE_RUNTIME_ID.fetch_add(1, Ordering::Relaxed),
            cols,
            rows,
        })
    }

    pub(crate) fn spawn_in_mode(
        argv: &[&str],
        cwd: Option<&Path>,
        cols: u16,
        rows: u16,
        io_mode: PaneIoMode,
    ) -> io::Result<Pane> {
        assert!(!argv.is_empty(), "argv must have at least the program");

        let term = Terminal::new(cols, rows).map_err(ghostty_err)?;
        let term = Arc::new(Mutex::new(term));
        let terminal_queries = Arc::new(Mutex::new(VecDeque::new()));

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
        let alive = Arc::new(AtomicBool::new(true));
        let pending_input = Arc::new(Mutex::new(VecDeque::new()));
        let pipe_output = Arc::new(Mutex::new(None));
        let pipe_output_active = Arc::new(AtomicBool::new(false));
        let observation = Arc::new(NativePaneObservation::new(
            term,
            Some(ObservedChild {
                pid: pid as u32,
                alive: Arc::clone(&alive),
            }),
            rows,
        ));
        let pane_io = PaneIo::new(
            &master,
            Arc::clone(&observation),
            Arc::clone(&terminal_queries),
            Arc::clone(&pending_input),
            Arc::clone(&pipe_output),
            Arc::clone(&pipe_output_active),
            Arc::clone(&alive),
        )?;
        let (reader, event_io) = match io_mode {
            #[cfg(test)]
            PaneIoMode::Threaded(spawn) => (Some(spawn(pane_io)), None),
            PaneIoMode::EventLoop => (None, Some(pane_io)),
        };

        Ok(Pane {
            observation,
            terminal_queries,
            child: Some(Child {
                pid,
                master,
                reader,
                alive,
                reaped: false,
                termination_requested: false,
                exit_code: None,
            }),
            pending_input,
            spawn_spec: Some(PaneSpawnSpec {
                argv: argv.iter().map(|arg| (*arg).to_string()).collect(),
                cwd: cwd.map(Path::to_path_buf),
            }),
            pipe_output,
            pipe_output_active,
            pipe: None,
            event_io,
            runtime_id: NEXT_PANE_RUNTIME_ID.fetch_add(1, Ordering::Relaxed),
            cols,
            rows,
        })
    }

    /// Spawn `argv` on a fresh pty with a dedicated reader thread.
    #[cfg(test)]
    pub(crate) fn spawn(argv: &[&str], cols: u16, rows: u16) -> io::Result<Pane> {
        Self::spawn_in_mode(argv, None, cols, rows, PaneIoMode::Threaded(spawn_reader))
    }

    pub(crate) fn spawn_spec(&self) -> Option<PaneSpawnSpec> {
        self.spawn_spec.clone()
    }

    pub(crate) fn spawn_from_spec_mode(
        spec: &PaneSpawnSpec,
        cols: u16,
        rows: u16,
        io_mode: PaneIoMode,
    ) -> io::Result<Pane> {
        let argv = spec.argv.iter().map(String::as_str).collect::<Vec<_>>();
        Self::spawn_in_mode(&argv, spec.cwd.as_deref(), cols, rows, io_mode)
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
            .is_some_and(|pipe| pipe.alive.load(Ordering::Acquire))
    }

    pub(crate) fn close_pipe(&mut self) {
        self.pipe_output_active.store(false, Ordering::Release);
        if let Ok(mut output) = self.pipe_output.lock() {
            *output = None;
        }
        self.pipe = None;
    }

    /// Start a shell command connected to pane output (`output`) and/or pane
    /// input (`input`). Worker threads own all blocking pipe I/O.
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
        let alive = Arc::new(AtomicBool::new(true));

        if output {
            let mut stdin = process
                .stdin
                .take()
                .ok_or_else(|| io::Error::other("pipe child has no stdin"))?;
            let (sender, receiver) = channel::<Vec<u8>>();
            *self
                .pipe_output
                .lock()
                .map_err(|_| io::Error::other("pane pipe mutex poisoned"))? = Some(sender);
            self.pipe_output_active.store(true, Ordering::Release);
            thread::spawn(move || {
                while let Ok(bytes) = receiver.recv() {
                    if stdin.write_all(&bytes).is_err() {
                        break;
                    }
                }
            });
        }

        if input {
            let child = self
                .child
                .as_ref()
                .ok_or_else(|| io::Error::other("pane has no child"))?;
            let mut stdout = process
                .stdout
                .take()
                .ok_or_else(|| io::Error::other("pipe child has no stdout"))?;
            let master = child.master.as_fd().try_clone_to_owned()?;
            let pending_input = Arc::clone(&self.pending_input);
            thread::spawn(move || {
                let mut bytes = [0u8; 4096];
                loop {
                    match stdout.read(&mut bytes) {
                        Ok(0) | Err(_) => break,
                        Ok(count) => {
                            enqueue_pane_input(
                                master.as_raw_fd(),
                                &pending_input,
                                &bytes[..count],
                            );
                        }
                    }
                }
            });
        }

        let reaper_alive = Arc::clone(&alive);
        thread::spawn(move || {
            let _ = process.wait();
            reaper_alive.store(false, Ordering::Release);
        });
        self.pipe = Some(PanePipe { pid, alive });
        Ok(())
    }

    /// Feed synthetic bytes directly into the screen (bypassing any pty). Used
    /// for inert panes and tests.
    pub fn feed(&self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        if let Ok(mut t) = self.observation.term.lock() {
            let large_scroll = self.observation.write_terminal(&mut t, bytes);
            let mut detector = CursorShapeDetector::default();
            for &byte in bytes {
                if let Some(shape) = detector.feed_byte(byte) {
                    self.observation
                        .cursor_shape
                        .store(shape, Ordering::Release);
                }
            }
            self.observation.record_output(bytes, large_scroll);
        }
    }

    /// Return the stable read-only handle associated with this pane.
    pub(crate) fn observation(&self) -> Arc<dyn PaneObservability> {
        Arc::clone(&self.observation) as Arc<dyn PaneObservability>
    }

    /// Concrete shared state used by the crate-private native observation
    /// handle. Public consumers continue to use `PaneObservability`.
    pub(crate) fn observation_state(&self) -> Arc<NativePaneObservation> {
        Arc::clone(&self.observation)
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
    /// in the bounded `pending_input` buffer and flushed by the reader thread on
    /// writability. This is what keeps a stalled full-screen app from wedging the
    /// server lock held by the caller (`forward_input` holds the state mutex; a
    /// blocking write here used to hang every command — see `report.md`).
    pub fn input(&self, bytes: &[u8]) -> io::Result<()> {
        self.input_with_stats(bytes).map(|_| ())
    }

    pub(crate) fn encode_key(&self, event: ghostty_sys::KeyEvent<'_>) -> io::Result<Vec<u8>> {
        self.observation
            .term
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?
            .encode_key(event)
            .map_err(ghostty_err)
    }

    pub(crate) fn encode_mouse(&self, event: ghostty_sys::MouseEvent) -> io::Result<Vec<u8>> {
        self.observation
            .term
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?
            .encode_mouse(event)
            .map_err(ghostty_err)
    }

    /// Reset the emulated terminal state without sending bytes to the child.
    pub(crate) fn reset_terminal(&self) -> io::Result<()> {
        let mut terminal = self
            .observation
            .term
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?;
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
        let child = self.child.as_ref()?;
        CurrentPlatform::pane_cwd(child.master.as_fd())
            .map(|path| path.to_string_lossy().into_owned())
    }

    /// The program occupying the pane's foreground process group.
    pub fn current_command(&self) -> Option<String> {
        let child = self.child.as_ref()?;
        // SAFETY: querying the foreground group of an owned pty master fd.
        let pid = unsafe { libc::tcgetpgrp(child.master.as_raw_fd()) };
        if pid <= 0 {
            return None;
        }
        // tmux's Linux osdep_get_name reads argv[0] from /proc/PID/cmdline.
        // Keep the executable-name candidates as a fallback for platforms or
        // processes where the argument vector is unavailable.
        CurrentPlatform::process_arguments(pid as u32)
            .into_iter()
            .next()
            .or_else(|| {
                CurrentPlatform::process_programs(pid as u32)
                    .into_iter()
                    .next()
            })
            .and_then(|program| {
                Path::new(&program)
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
    }

    /// Drain terminal queries emitted by the child since the previous call.
    ///
    /// These bytes are written to the attached client's terminal. Its response
    /// then comes back through the normal client-input path and reaches
    /// [`Pane::input`], completing the same exchange real tmux provides.
    pub fn take_terminal_queries(&self) -> Vec<Vec<u8>> {
        self.terminal_queries
            .lock()
            .map(|mut queries| queries.drain(..).collect())
            .unwrap_or_default()
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
        if let Ok(mut t) = self.observation.term.lock() {
            t.resize(cols, rows).map_err(ghostty_err)?;
            if let Ok(mut detector) = self.observation.redraw_detector.lock() {
                detector.resize(rows);
            }
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
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?
            .dump_plain()
            .map_err(ghostty_err)
    }

    pub(crate) fn cursor_position(&self) -> io::Result<(u16, u16)> {
        self.observation
            .term
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?
            .cursor_position()
            .map_err(ghostty_err)
    }

    pub(crate) fn copy_snapshot(
        &self,
    ) -> io::Result<(ghostty_sys::GridSnapshot, Vec<u8>, (u16, u16))> {
        let terminal = self
            .observation
            .term
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?;
        let grid = terminal.grid_snapshot().map_err(ghostty_err)?;
        let vt = terminal.dump_vt().map_err(ghostty_err)?;
        let cursor = terminal.cursor_position().map_err(ghostty_err)?;
        Ok((grid, vt, cursor))
    }

    /// Snapshot the active Ghostty grid for row-oriented consumers such as
    /// `capture-pane`. This deliberately exposes Ghostty's physical rows and
    /// soft-wrap metadata rather than reconstructing them from a text dump.
    pub(crate) fn grid_snapshot(&self) -> io::Result<ghostty_sys::GridSnapshot> {
        self.observation
            .term
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?
            .grid_snapshot()
            .map_err(ghostty_err)
    }

    pub(crate) fn background_color(&self) -> String {
        self.observation
            .background
            .lock()
            .map(|color| color.clone())
            .unwrap_or_else(|_| "default".to_string())
    }

    /// Latest title advertised by the child. Ghostty handles OSC titles; the
    /// screen/tmux `ESC k ... ST` form is consumed before reaching Ghostty, so
    /// recover that form from the bounded raw-output journal.
    pub(crate) fn title(&self) -> Option<String> {
        let legacy = self
            .observation
            .control_output
            .lock()
            .ok()
            .and_then(|output| latest_screen_title(output.bytes.iter().copied()));
        legacy.or_else(|| {
            self.observation
                .term
                .lock()
                .ok()
                .and_then(|terminal| terminal.title().ok().flatten())
        })
    }

    /// The current screen as VT escape sequences, suitable for writing to a
    /// client tty. This is the compositor primitive: the pane's grid is
    /// formatted as VT and sent to the attached client's terminal.
    pub fn dump_vt(&self) -> io::Result<Vec<u8>> {
        self.observation
            .term
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?
            .dump_vt()
            .map_err(ghostty_err)
    }

    pub(crate) fn dump_rows_vt(&self, start: usize, rows: usize) -> io::Result<Vec<u8>> {
        self.observation
            .term
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?
            .dump_vt_rows(start, rows, self.cols)
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
        let terminal = self
            .observation
            .term
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?;
        let scrollback = terminal.scrollback_rows().map_err(ghostty_err)?;
        let scroll = scroll_offset.min(scrollback);
        let start = scrollback - scroll;
        let vt = terminal
            .dump_vt_rows(start, visible_rows, self.cols)
            .map_err(ghostty_err)?;
        Ok((vt, scroll))
    }

    /// How many scrollback (history) rows the grid holds above the visible
    /// viewport. Consumers that render only the on-screen rows (the compositor,
    /// `capture-pane -p`) skip this many leading rows of a dump.
    pub fn scrollback_rows(&self) -> io::Result<usize> {
        self.observation
            .term
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?
            .scrollback_rows()
            .map_err(ghostty_err)
    }

    /// Clear scrollback while preserving the visible viewport.
    ///
    /// CSI 3 J is Ghostty's own erase-scrollback operation, so this keeps the
    /// chosen terminal engine authoritative instead of reconstructing its grid
    /// in hmux.
    pub fn clear_history(&self) -> io::Result<()> {
        let mut terminal = self
            .observation
            .term
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?;
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
            .lock()
            .map_err(|_| io::Error::other("pane terminal mutex poisoned"))?
            .cursor_visible()
            .map_err(ghostty_err)
    }

    /// Current DECSCUSR parameter (0/default, 1..=6 block/underline/bar with
    /// blinking encoded by odd values), for mirroring onto the attached tty.
    pub fn cursor_shape(&self) -> u8 {
        self.observation.cursor_shape.load(Ordering::Acquire)
    }

    pub(crate) fn bracketed_paste_enabled(&self) -> bool {
        self.observation.bracketed_paste.load(Ordering::Acquire)
    }

    /// Whether the pane asked to be told when focus moves (DECSET 1004).
    pub(crate) fn focus_reporting_enabled(&self) -> bool {
        self.observation.focus_reporting.load(Ordering::Acquire)
    }

    /// Whether the pane asked to be told when the theme changes (DECSET 2031).
    pub(crate) fn theme_updates_enabled(&self) -> bool {
        self.observation.theme_updates.load(Ordering::Acquire)
    }

    /// The pane program's DECSET mouse reporting state, which decides both
    /// which reports reach it and how the default bindings treat a click.
    pub(crate) fn mouse_modes(&self) -> PaneMouseModes {
        self.observation.mouse_modes()
    }

    /// Take a pending DSR ?996 theme question, if the pane asked one.
    pub(crate) fn take_theme_query(&self) -> bool {
        self.observation.theme_query.swap(false, Ordering::AcqRel)
    }

    pub fn size(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    pub fn is_live(&self) -> bool {
        self.child
            .as_ref()
            .is_some_and(|child| child.alive.load(Ordering::Acquire))
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
            .is_some_and(|child| !child.alive.load(Ordering::Acquire))
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
            libc::WEXITSTATUS(status)
        } else if libc::WIFSIGNALED(status) {
            128 + libc::WTERMSIG(status)
        } else {
            return None;
        };
        child.exit_code = Some(code);
        Some(code)
    }

    pub(crate) fn collect_exited_child(&mut self, terminate_if_running: bool) -> bool {
        if !self.has_exited() {
            return false;
        }
        if self.try_wait().is_some() {
            return true;
        }
        let Some(child) = self.child.as_mut() else {
            return true;
        };
        if terminate_if_running && !child.termination_requested {
            unsafe {
                libc::kill(child.pid, libc::SIGKILL);
            }
            child.termination_requested = true;
        }
        child.reaped
    }

    pub(crate) fn child_reaped(&self) -> bool {
        self.child.as_ref().is_none_or(|child| child.reaped)
    }

    /// Block until the child exits and all its output has been drained into the
    /// screen (the reader thread reaches EOF and finishes). Test helper.
    pub fn wait_drained(&mut self) {
        if let Some(child) = &mut self.child {
            if let Some(handle) = child.reader.take() {
                let _ = handle.join();
            }
        }
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
        if self.reaped && self.reader.is_none() {
            return;
        }
        // Kill the child so its pty slave closes, unblocking the reader's read().
        // SAFETY: sending a signal to our own child pid.
        if !self.reaped {
            unsafe {
                libc::kill(self.pid, libc::SIGKILL);
            }
        }

        // Reap the child and join the reader thread OFF the caller's thread.
        //
        // Both steps can block for an unbounded time. `waitpid(pid, 0)` waits for
        // the signalled child to actually die (usually instant, but a child stuck
        // in an uninterruptible syscall delays it). The reader thread sits in
        // `poll(master, -1)` until the pty master hangs up, which only happens
        // once *every* slave fd is closed — a killed shell's still-running
        // subprocess (a background agent) keeps the slave open, so the master
        // never hangs up and `join()` waits for as long as that grandchild lives.
        //
        // Crucially, `Child::drop` runs inside `kill-window` / `kill-pane`, which
        // the attach loop invokes while holding the global server-state mutex.
        // Blocking here froze the whole compositor: answering the `confirm-before`
        // `(y/n)` prompt cleared it instantly (client-local state) but the
        // post-kill redraw was stuck behind this teardown, so pressing `y`
        // appeared to lag by a second or more — intermittently, depending on
        // whether the pane's process tree still held the pty. Handing the
        // teardown to a detached thread lets the kill return at once so the
        // compositor redraws immediately; the child is still reaped (no zombie)
        // and the reader thread still exits once the pty finally closes.
        //
        // The `master` OwnedFd field is dropped as this `Child` drops, closing the
        // parent's handle; the reader thread owns a separate dup, so its lifetime
        // is unaffected by that close.
        let pid = (!self.reaped).then_some(self.pid);
        let reader = self.reader.take();
        thread::spawn(move || {
            // SAFETY: reaping our own child pid. No other code waits on it, so
            // there is no competing consumer and PID reuse is not a hazard.
            if let Some(pid) = pid {
                unsafe {
                    let mut status = 0;
                    libc::waitpid(pid, &mut status, 0);
                }
            }
            if let Some(handle) = reader {
                let _ = handle.join();
            }
        });
    }
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

/// Owned nonblocking PTY state. A compatibility thread or the central reactor
/// may drive the same parser one readiness turn at a time.
pub(crate) struct PaneIo {
    fd: OwnedFd,
    observation: Arc<NativePaneObservation>,
    terminal_queries: Arc<Mutex<VecDeque<Vec<u8>>>>,
    pending_input: Arc<Mutex<VecDeque<u8>>>,
    pipe_output: Arc<Mutex<Option<Sender<Vec<u8>>>>>,
    pipe_output_active: Arc<AtomicBool>,
    alive: Arc<AtomicBool>,
    query_detector: BackgroundColorQueryDetector,
    dsr_detector: DeviceStatusReportQueryDetector,
    cursor_report_detector: CursorPositionReportQueryDetector,
    cursor_shape_detector: CursorShapeDetector,
    mode_query_detector: ModeQueryDetector,
    background_detector: BackgroundColorDetector,
    clipboard_detector: Osc52Detector,
    utf8_sanitizer: Utf8Sanitizer,
    title_stripper: ScreenTitleStripper,
    bell_detector: BellDetector,
    closed: bool,
}

impl PaneIo {
    pub(crate) fn new(
        master: &OwnedFd,
        observation: Arc<NativePaneObservation>,
        terminal_queries: Arc<Mutex<VecDeque<Vec<u8>>>>,
        pending_input: Arc<Mutex<VecDeque<u8>>>,
        pipe_output: Arc<Mutex<Option<Sender<Vec<u8>>>>>,
        pipe_output_active: Arc<AtomicBool>,
        alive: Arc<AtomicBool>,
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
            background_detector: BackgroundColorDetector::default(),
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
        self.pending_input
            .lock()
            .map(|queued| !queued.is_empty())
            .unwrap_or(false)
    }

    pub(crate) fn drive_writable(&mut self) {
        if self.closed {
            return;
        }
        if let Ok(mut queued) = self.pending_input.lock() {
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
        if self.pipe_output_active.load(Ordering::Acquire) {
            let pipe_sender = self
                .pipe_output
                .lock()
                .ok()
                .and_then(|sender| sender.clone());
            if let Some(sender) = pipe_sender {
                if sender.send(pending.clone()).is_err() {
                    self.pipe_output_active.store(false, Ordering::Release);
                    if let Ok(mut current) = self.pipe_output.lock() {
                        *current = None;
                    }
                }
            }
        }

        self.observation.append_control_output(&pending);
        let sanitized = self.utf8_sanitizer.filter(&pending);
        let filtered = self.title_stripper.filter(&sanitized);
        let bytes = &filtered[..];
        self.observation.note_bells(
            bytes
                .iter()
                .filter(|byte| self.bell_detector.feed(**byte))
                .count() as u64,
        );
        let mut queries = Vec::new();
        let mut cursor_report_queries = Vec::new();
        let mut mode_replies = Vec::new();
        for (index, &byte) in bytes.iter().enumerate() {
            if self.query_detector.feed_byte(byte) {
                queries.push(BACKGROUND_COLOR_QUERY);
            }
            if self.dsr_detector.feed_byte(byte) {
                queries.push(DEVICE_STATUS_REPORT_QUERY);
            }
            if let Some(kind) = self.cursor_report_detector.feed_byte(byte) {
                cursor_report_queries.push((index, kind));
            }
            if let Some(shape) = self.cursor_shape_detector.feed_byte(byte) {
                self.observation
                    .cursor_shape
                    .store(shape, Ordering::Release);
            }
            if let Some(reply) = self.mode_query_detector.feed_byte(byte) {
                mode_replies.push(reply);
            }
            if let Some(color) = self.background_detector.feed_byte(byte) {
                if let Ok(mut background) = self.observation.background.lock() {
                    *background = color;
                }
            }
            if let Some(event) = self.clipboard_detector.feed_byte(byte) {
                self.observation.note_clipboard_event(event);
            }
        }
        self.observation
            .bracketed_paste
            .store(self.mode_query_detector.bracketed_paste, Ordering::Release);
        self.observation
            .focus_reporting
            .store(self.mode_query_detector.focus_reporting, Ordering::Release);
        self.observation
            .theme_updates
            .store(self.mode_query_detector.theme_updates, Ordering::Release);
        self.observation.mouse_tracking_mode.store(
            match self.mode_query_detector.mouse_tracking {
                None => 0,
                Some(MouseTrackingMode::Standard) => 1,
                Some(MouseTrackingMode::Button) => 2,
                Some(MouseTrackingMode::All) => 3,
            },
            Ordering::Release,
        );
        self.observation
            .mouse_utf8
            .store(self.mode_query_detector.mouse_utf8, Ordering::Release);
        self.observation
            .mouse_sgr
            .store(self.mode_query_detector.mouse_sgr, Ordering::Release);
        if std::mem::take(&mut self.mode_query_detector.theme_query) {
            self.observation.theme_query.store(true, Ordering::Release);
        }
        if !queries.is_empty() {
            if let Ok(mut queued) = self.terminal_queries.lock() {
                for query in queries {
                    if queued.len() == 16 {
                        break;
                    }
                    queued.push_back(query.to_vec());
                }
            }
        }

        let mut cursor_replies = Vec::new();
        if let Ok(mut terminal) = self.observation.term.lock() {
            let mut segment_start = 0usize;
            let mut large_scroll = false;
            for (query_end, kind) in cursor_report_queries {
                large_scroll |= self
                    .observation
                    .write_terminal(&mut terminal, &bytes[segment_start..=query_end]);
                if let Some(response) = cursor_position_report(&terminal, kind) {
                    cursor_replies.push(response);
                }
                segment_start = query_end + 1;
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
        self.alive.store(false, Ordering::Release);
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

#[derive(Default)]
struct BackgroundColorDetector {
    sequence: Vec<u8>,
    in_osc: bool,
    escaped: bool,
}

impl BackgroundColorDetector {
    fn feed_byte(&mut self, byte: u8) -> Option<String> {
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
                return self.finish();
            }
            self.sequence.push(0x1b);
        }
        match byte {
            0x07 | 0x9c => self.finish(),
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

    fn finish(&mut self) -> Option<String> {
        self.in_osc = false;
        self.escaped = false;
        let sequence = std::mem::take(&mut self.sequence);
        let text = std::str::from_utf8(&sequence).ok()?;
        if text == "111" {
            return Some("default".to_string());
        }
        let payload = text.strip_prefix("11;")?;
        (payload != "?")
            .then(|| parse_background_color(payload))
            .flatten()
    }
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
    } else if value.eq_ignore_ascii_case("DodgerBlue4") {
        (0x10, 0x4e, 0x8b)
    } else if value.eq_ignore_ascii_case("grey") || value.eq_ignore_ascii_case("gray") {
        (0xbe, 0xbe, 0xbe)
    } else {
        let lower = value.to_ascii_lowercase();
        let percentage = lower
            .strip_prefix("grey")
            .or_else(|| lower.strip_prefix("gray"))?
            .parse::<u16>()
            .ok()?;
        if percentage > 100 {
            return None;
        }
        let component = (percentage as f64 * 2.55).round() as u8;
        (component, component, component)
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CursorPositionReportKind {
    Standard,
    Private,
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
    Private,
    SawSix(CursorPositionReportKind),
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
struct ModeQueryDetector {
    tail: VecDeque<u8>,
    synchronized_output: bool,
    cursor_visible: bool,
    bracketed_paste: bool,
    /// DECSET 1004: the pane asked to be told when focus moves.
    focus_reporting: bool,
    /// Whether the pane asked which theme it is under (DSR ?996) and has
    /// not been answered yet.
    theme_query: bool,
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
            synchronized_output: false,
            cursor_visible: true,
            bracketed_paste: false,
            focus_reporting: false,
            theme_updates: false,
            theme_query: false,
            mouse_tracking: None,
            mouse_utf8: false,
            mouse_sgr: false,
        }
    }
}

impl ModeQueryDetector {
    fn mouse_mode_status(&self, mode: MouseTrackingMode) -> u8 {
        if self.mouse_tracking == Some(mode) {
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
            self.synchronized_output = true;
        } else if tail.ends_with(b"\x1b[?2026l") {
            self.synchronized_output = false;
        } else if tail.ends_with(b"\x1b[?25h") {
            self.cursor_visible = true;
        } else if tail.ends_with(b"\x1b[?25l") {
            self.cursor_visible = false;
        } else if tail.ends_with(b"\x1b[?2004h") {
            self.bracketed_paste = true;
        } else if tail.ends_with(b"\x1b[?2004l") {
            self.bracketed_paste = false;
        } else if tail.ends_with(b"\x1b[?1004h") {
            self.focus_reporting = true;
        } else if tail.ends_with(b"\x1b[?1004l") {
            self.focus_reporting = false;
        } else if tail.ends_with(b"\x1b[?2031h") {
            self.theme_updates = true;
        } else if tail.ends_with(b"\x1b[?2031l") {
            self.theme_updates = false;
        } else if tail.ends_with(b"\x1b[?1000h") {
            self.mouse_tracking = Some(MouseTrackingMode::Standard);
        } else if tail.ends_with(b"\x1b[?1002h") {
            self.mouse_tracking = Some(MouseTrackingMode::Button);
        } else if tail.ends_with(b"\x1b[?1003h") {
            self.mouse_tracking = Some(MouseTrackingMode::All);
        } else if tail.ends_with(b"\x1b[?1000l")
            || tail.ends_with(b"\x1b[?1001l")
            || tail.ends_with(b"\x1b[?1002l")
            || tail.ends_with(b"\x1b[?1003l")
        {
            self.mouse_tracking = None;
        } else if tail.ends_with(b"\x1b[?1005h") {
            self.mouse_utf8 = true;
        } else if tail.ends_with(b"\x1b[?1005l") {
            self.mouse_utf8 = false;
        } else if tail.ends_with(b"\x1b[?1006h") {
            self.mouse_sgr = true;
        } else if tail.ends_with(b"\x1b[?1006l") {
            self.mouse_sgr = false;
        } else if tail.ends_with(b"\x1b[?996n") {
            // DSR ?996: the pane is asking which theme it is running under.
            self.theme_query = true;
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
                            if self.synchronized_output {
                                1
                            } else {
                                2
                            }
                        }
                        25 => {
                            if self.cursor_visible {
                                1
                            } else {
                                2
                            }
                        }
                        1004 => {
                            if self.focus_reporting {
                                1
                            } else {
                                2
                            }
                        }
                        2031 => {
                            if self.theme_updates {
                                1
                            } else {
                                2
                            }
                        }
                        1000 => self.mouse_mode_status(MouseTrackingMode::Standard),
                        1002 => self.mouse_mode_status(MouseTrackingMode::Button),
                        1003 => self.mouse_mode_status(MouseTrackingMode::All),
                        1005 => {
                            if self.mouse_utf8 {
                                1
                            } else {
                                2
                            }
                        }
                        1006 => {
                            if self.mouse_sgr {
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
    fn feed_byte(&mut self, byte: u8) -> Option<CursorPositionReportKind> {
        use CursorPositionReportKind::{Private, Standard};
        use CursorPositionReportState::{Csi, Esc, Ground, SawSix};

        let next = match (self.state, byte) {
            (Ground, b'\x1b') => Esc,
            (Ground, _) => Ground,
            (Esc, b'[') => Csi,
            (Esc, b'\x1b') => Esc,
            (Esc, _) => Ground,
            (Csi, b'6') => SawSix(Standard),
            (Csi, b'?') => CursorPositionReportState::Private,
            (Csi, b'\x1b') => Esc,
            (Csi, _) => Ground,
            (CursorPositionReportState::Private, b'6') => SawSix(Private),
            (CursorPositionReportState::Private, b'\x1b') => Esc,
            (CursorPositionReportState::Private, _) => Ground,
            (SawSix(kind), b'n') => {
                self.state = Ground;
                return Some(kind);
            }
            (SawSix(_), b'\x1b') => Esc,
            (SawSix(_), _) => Ground,
        };
        self.state = next;
        None
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

fn cursor_position_report(terminal: &Terminal, kind: CursorPositionReportKind) -> Option<Vec<u8>> {
    let (x, y) = terminal.cursor_position().ok()?;
    let prefix = match kind {
        CursorPositionReportKind::Standard => "\x1b[",
        CursorPositionReportKind::Private => "\x1b[?",
    };
    Some(format!("{prefix}{};{}R", y.saturating_add(1), x.saturating_add(1)).into_bytes())
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
fn enqueue_pane_input(fd: c_int, pending: &Mutex<VecDeque<u8>>, bytes: &[u8]) -> PaneInputStats {
    let Ok(mut queued) = pending.lock() else {
        return PaneInputStats {
            dropped: bytes.len(),
            ..PaneInputStats::default()
        };
    };
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

    /// Writing to a child that has stopped reading its stdin must never block the
    /// caller: the pty master is non-blocking and `Pane::input` buffers/drops
    /// instead of stalling. Before the fix a blocking `write(2)` here (or in the
    /// reader thread's cursor-report reply) wedged the terminal lock and hung
    /// `capture-pane` (see `report.md`). Run on a worker thread with a deadline so
    /// a regression fails the suite instead of hanging it.
    #[test]
    fn input_to_a_child_that_never_reads_stdin_does_not_block() {
        use std::sync::mpsc;
        use std::time::Duration;

        // Raw mode (so a full input buffer blocks the writer, like a TUI), then
        // sleep without ever reading stdin.
        let pane = Pane::spawn(
            &[
                "/bin/sh",
                "-c",
                "stty -icanon -echo min 1 time 0; exec sleep 30",
            ],
            80,
            24,
        )
        .expect("spawn");
        // Let the child apply raw mode before we flood its stdin.
        std::thread::sleep(Duration::from_millis(300));

        let (tx, rx) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            // Far more than any pty input buffer, never drained by the child.
            let big = vec![b'x'; 1024 * 1024];
            for _ in 0..4 {
                let _ = pane.input(&big);
            }
            // Prove the terminal lock was never wedged by the writes.
            let dumped = pane.dump().is_ok();
            let _ = tx.send(dumped);
        });

        match rx.recv_timeout(Duration::from_secs(3)) {
            Ok(dumped) => assert!(dumped, "pane screen must stay dumpable"),
            Err(_) => panic!("Pane::input blocked on a child that will not read its stdin"),
        }
        let _ = worker.join();
    }

    #[test]
    fn spawned_child_output_lands_in_grid() {
        // A short-lived, deterministic command: prints and exits.
        let mut pane =
            Pane::spawn(&["/bin/sh", "-c", "printf 'ghostty-pane-works'"], 40, 5).expect("spawn");
        assert!(pane.is_live());
        pane.wait_drained(); // child exits → reader drains → thread ends
        let dump = pane.dump().expect("dump");
        assert!(
            dump.contains("ghostty-pane-works"),
            "pty output should reach the grid, got {dump:?}"
        );
    }

    #[test]
    fn observation_reports_spawned_child_lifecycle() {
        let mut pane = Pane::spawn(&["/bin/sh", "-c", "printf observed"], 40, 5).expect("spawn");
        let observation = pane.observation();
        assert!(observation.process().expect("process").child_pid.is_some());

        pane.wait_drained();

        assert!(observation.process().expect("process").exited);
        assert!(observation.output_revision().expect("revision") > 0);
        assert!(observation
            .last_lines(1)
            .expect("screen tail")
            .text
            .contains("observed"));
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
    fn dropping_killed_pane_does_not_block_on_lingering_grandchild() {
        // Repro for the intermittent `C-b x` / `C-b &` kill lag. Answering the
        // confirm prompt with `y` calls kill-pane / kill-window, which removes
        // the window and drops its `Pane`. `Child::drop` used to `waitpid(pid,
        // 0)` and then `join()` the reader thread *synchronously*, and the
        // reader sits in `poll(master, -1)` until the pty master hangs up — which
        // only happens once *every* slave fd is closed. A killed shell's still
        // running subprocess (exactly the "3-4 agents" workload in note.md) keeps
        // the slave open, so the join blocked for as long as that grandchild
        // lived. Because the kill runs on the attach loop's thread while it holds
        // the global server-state mutex, that block froze the whole compositor:
        // the prompt cleared instantly (client-local state) but the post-kill
        // redraw was stuck behind the teardown, so pressing `y` appeared to lag by
        // a second or more. The drop must hand the blocking teardown off and
        // return promptly on the caller's thread.
        //
        // `sh -c 'setsid sleep 10 & wait'` leaves a `setsid` grandchild in its own
        // session: it survives the SIGKILL delivered to the tracked shell (it is
        // not in the shell's process group, so it gets no SIGHUP) yet keeps the
        // pty slave open via its inherited stdout/stderr. The master therefore
        // never hangs up until the grandchild exits, reproducing a `poll(master,
        // -1)` in the reader thread that the old blocking `join()` waited on.
        let pane = Pane::spawn(&["/bin/sh", "-c", "setsid sleep 10 & wait"], 40, 5).expect("spawn");
        // Let the shell background the grandchild so it holds the pty slave open
        // before we tear the pane down.
        std::thread::sleep(std::time::Duration::from_millis(200));

        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let start = Instant::now();
            drop(pane);
            let _ = tx.send(start.elapsed());
        });

        match rx.recv_timeout(std::time::Duration::from_secs(4)) {
            Ok(elapsed) => assert!(
                elapsed < std::time::Duration::from_secs(2),
                "dropping a killed pane must return promptly, took {elapsed:?}"
            ),
            Err(_) => panic!(
                "dropping a killed pane blocked >4s waiting for a lingering \
                 grandchild to release the pty — the kill path stalls the \
                 compositor (this is the `C-b x`/`y` redraw lag)"
            ),
        }
    }

    #[test]
    fn spawned_pane_reports_inherited_cwd() {
        // A freshly spawned pane inherits the server's cwd (no chdir), so its
        // live current path is exactly that directory — read from the child, not
        // assumed.
        let cwd = std::env::current_dir()
            .expect("cwd")
            .to_string_lossy()
            .into_owned();
        let pane = Pane::spawn(&["/bin/sh", "-c", "sleep 30"], 40, 5).expect("spawn");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            if pane.current_path().as_deref() == Some(cwd.as_str()) {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "pane_current_path should report the inherited cwd {cwd:?}, got {:?}",
                pane.current_path()
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    #[test]
    fn current_path_tracks_child_directory_change() {
        // The reported bug, in miniature: after the pane's shell `cd`s,
        // #{pane_current_path} (Pane::current_path) must reflect the new
        // directory, not the hmux server's launch directory. Real tmux reads the
        // child's live cwd; the native engine must too. Before the fix
        // this returned std::env::current_dir() (the server cwd) forever, so this
        // test would hang until the deadline and fail.
        //
        // The process temporary directory is available both in ordinary test
        // runs and in pure build sandboxes. The platform lookup returns a
        // canonical path, so compare against the canonicalized target.
        let target = std::fs::canonicalize(std::env::temp_dir())
            .expect("temporary directory")
            .to_string_lossy()
            .into_owned();
        let pane = Pane::spawn(&["/bin/sh"], 40, 5).expect("spawn sh");
        pane.input(b"cd \"${TMPDIR:-/tmp}\"\n").expect("send cd");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if pane.current_path().as_deref() == Some(target.as_str()) {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "pane_current_path never followed the cd to {target:?}; got {:?}",
                pane.current_path()
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    #[test]
    fn current_command_tracks_foreground_program() {
        let pane = Pane::spawn(&["/bin/sh"], 40, 5).expect("spawn sh");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            if pane.current_command().as_deref() == Some("sh") {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "pane_current_command should report sh, got {:?}",
                pane.current_command()
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        pane.input(b"sleep 30\n").expect("start foreground app");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            if pane.current_command().as_deref() == Some("sleep") {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "pane_current_command should follow the foreground app, got {:?}",
                pane.current_command()
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    #[test]
    fn observation_reports_osc_title() {
        let pane = Pane::inert(40, 5).expect("inert pane");
        let observation = pane.observation();
        // No title set yet.
        assert_eq!(observation.title().expect("title"), None);
        // Codex reports its live status in the window title via OSC 2.
        pane.feed(b"\x1b]2;Working (5s)\x07");
        assert_eq!(
            observation.title().expect("title").as_deref(),
            Some("Working (5s)")
        );
    }

    #[test]
    fn spawned_child_does_not_inherit_extra_fds() {
        // Regression: a pane's child (and the shell it exec's) must not inherit
        // fds above stdio. If it does, a `tmux new-window` command client whose
        // stdout is a pipe (`$(tmux …)`) never sees EOF and hangs, because the
        // long-lived child keeps the write end open. We model that fd with a
        // pipe: keep the write end open across the spawn, then close our copy —
        // if the child leaked it, the pipe still has a writer.
        let mut fds = [0 as c_int; 2];
        assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0, "pipe");
        let (rd, wr) = (fds[0], fds[1]);

        // Long-lived child so a leaked write end would stay open.
        let _pane = Pane::spawn(&["/bin/sh", "-c", "sleep 30"], 20, 4).expect("spawn");

        // Drop the parent's write end; now only a leaking child could hold one.
        assert_eq!(unsafe { libc::close(wr) }, 0, "close wr");

        // Non-blocking read: EOF (0) means no writers remain (child didn't leak);
        // EAGAIN means a writer is still open (the child inherited it). Poll
        // briefly to absorb the fork→descriptor cleanup→exec startup window.
        unsafe {
            let flags = libc::fcntl(rd, libc::F_GETFL);
            libc::fcntl(rd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }
        let mut buf = [0u8; 1];
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        let mut got_eof = false;
        while std::time::Instant::now() < deadline {
            let n = unsafe { libc::read(rd, buf.as_mut_ptr() as *mut c_void, 1) };
            if n == 0 {
                got_eof = true; // no writers → child did not inherit the fd
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        unsafe {
            libc::close(rd);
        }
        assert!(
            got_eof,
            "spawned child leaked an inherited fd (pipe still has a writer)"
        );
    }

    #[test]
    fn reader_drains_an_immediately_available_burst() {
        // Adjacent output is consumed without a settling timer. Depending on
        // scheduler timing this may be one or two readiness events, but neither
        // path delays the bytes while waiting for hypothetical future output.
        let script = "printf 'FIRST'; printf 'SECOND'";
        let mut pane = Pane::spawn(&["/bin/sh", "-c", script], 40, 5).expect("spawn");
        pane.wait_drained();
        assert!(pane.dump().unwrap().contains("FIRSTSECOND"));
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
    fn screen_style_title_sequence_is_not_leaked_into_grid() {
        // Under a screen/tmux `$TERM`, zsh sets the window title to the running
        // command using the legacy `ESC k <title> ESC \` sequence (terminfo
        // tsl/fsl). A terminal must *consume* it, not render it — otherwise the
        // title text leaks in front of the command's real output. This is the
        // "ls -> lsAGENTS.md" bug: libghostty-vt does not implement `ESC k`, so
        // the running-command title (`ls`) was printed literally at the start of
        // the output row. Real tmux consumes the sequence; the native engine must
        // match by stripping it before the bytes reach the grid.
        //
        // `printf '\033kTITLE\033\\VISIBLE'` emits: set-title("TITLE") + "VISIBLE".
        let mut pane = Pane::spawn(
            &["/bin/sh", "-c", "printf '\\033kTITLE\\033\\\\VISIBLE'"],
            40,
            5,
        )
        .expect("spawn");
        pane.wait_drained();
        let dump = pane.dump().expect("dump");
        assert!(
            dump.contains("VISIBLE"),
            "the command's real output must render, got {dump:?}"
        );
        assert!(
            !dump.contains("TITLE"),
            "the screen-style title must not leak into the grid, got {dump:?}"
        );
    }

    #[test]
    fn screen_title_sequence_split_across_pty_reads_is_still_stripped() {
        // A PTY read can split the escape at any byte, so the stripper must be a
        // streaming state machine (like the query detectors). Emit the title with
        // a sleep in the middle of the sequence so the two halves land in
        // separate reader bursts, then confirm nothing leaks.
        let script = "printf '\\033kTIT'; sleep 0.05; printf 'LE\\033\\\\SHOWN'";
        let mut pane = Pane::spawn(&["/bin/sh", "-c", script], 40, 5).expect("spawn");
        pane.wait_drained();
        let dump = pane.dump().expect("dump");
        assert!(dump.contains("SHOWN"), "output must render, got {dump:?}");
        assert!(
            !dump.contains("TIT"),
            "split title must not leak, got {dump:?}"
        );
    }

    #[test]
    fn bel_terminated_screen_title_is_stripped() {
        // Some emitters terminate the title with BEL instead of ST (`ESC \`).
        let mut pane =
            Pane::spawn(&["/bin/sh", "-c", "printf '\\033kNAME\\007OUT'"], 40, 5).expect("spawn");
        pane.wait_drained();
        let dump = pane.dump().expect("dump");
        assert!(dump.contains("OUT"), "output must render, got {dump:?}");
        assert!(
            !dump.contains("NAME"),
            "BEL-terminated title must not leak, got {dump:?}"
        );
    }

    #[test]
    fn input_reaches_child_and_echoes_back() {
        // `cat` echoes stdin back to the pty; the echo lands in the grid.
        let pane = Pane::spawn(&["/bin/cat"], 40, 5).expect("spawn cat");
        pane.input(b"roundtrip\r").expect("send input");
        // Give the echo a moment to arrive, then dump.
        std::thread::sleep(std::time::Duration::from_millis(200));
        let dump = pane.dump().expect("dump");
        assert!(
            dump.contains("roundtrip"),
            "echo should appear, got {dump:?}"
        );
        // Pane drops here → child killed, reader joined.
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
        assert!(!b"before\x1b[6"
            .iter()
            .any(|&byte| cpr.feed_byte(byte).is_some()));
        assert_eq!(
            cpr.feed_byte(b'n'),
            Some(CursorPositionReportKind::Standard)
        );

        let mut private_cpr = CursorPositionReportQueryDetector::default();
        assert!(!b"before\x1b[?6"
            .iter()
            .any(|&byte| private_cpr.feed_byte(byte).is_some()));
        assert_eq!(
            private_cpr.feed_byte(b'n'),
            Some(CursorPositionReportKind::Private)
        );
    }

    #[test]
    fn pane_answers_cursor_position_report_from_inner_grid() {
        let script = concat!(
            "stty raw -echo min 0 time 20; ",
            "printf 'abc\\033[6n'; ",
            "r=$(dd bs=32 count=1 2>/dev/null | od -An -tx1 | tr -d ' \\n'); ",
            "stty sane; ",
            "printf '\\r\\nCPR:%s\\r\\n' \"$r\""
        );
        let mut pane = Pane::spawn(&["/bin/sh", "-c", script], 40, 5).expect("spawn");
        pane.wait_drained();
        let dump = pane.dump().expect("dump");
        assert!(
            dump.contains("CPR:1b5b313b3452"),
            "expected ESC[1;4R after three printed cells, got {dump:?}"
        );
    }
}

//! Native tmux server built on libghostty-vt.
//!
//! The native path owns sessions/windows/panes, spawns pty
//! children, and maintains each pane's screen with a libghostty-vt terminal
//! ([`crate::ghostty`]). It implements the same [`crate::tmux::TmuxServer`]
//! trait, so the conformance suite runs against it unchanged — real tmux stays
//! the ground truth.
//!
//! The interactive attach path — compositing panes onto the client's tty — is
//! implemented in `attach.rs`: on attach-identify the server takes the client's
//! tty fd and drives it directly via libghostty's VT formatter, with input
//! forwarding and resize/detach handling.
//!
//! The only public server contract remains [`TmuxServer`]. The observation
//! hook below is crate-private: the native runtime emits unclassified pane
//! lifecycle/output events to first-party consumers, while those consumers
//! own process-tree walking, agent detection, classification, and status
//! publication. The worker calls hooks without holding the server or terminal
//! locks; pane handles retain shared terminal state and remain readable after
//! removal.

pub mod attach;
pub mod pane;
pub mod protocol;

use std::collections::HashMap;
use std::io;
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

use crate::integration::status::StatusHub;
use crate::observability::v1::{PaneId as PublicPaneId, PaneObservability, ServerObservability};
use crate::tmux::codec::{split_stream, ImsgReader, ImsgWriter};
use crate::tmux::traits::TmuxServer;

use crate::server::state::ServerState;
use pane::{PaneIo, PaneIoMode};

type EventPaneSnapshot = (Vec<(u64, PaneIo)>, Vec<u64>);

/// A tmux server implemented natively on libghostty-vt (no backing tmux).
///
/// State is shared across connections behind a mutex; each [`TmuxServer::connect`]
/// spins up an in-process client/server socket pair and a handler thread, which
/// is exactly the shape a real listener uses (see [`crate::serve`]) minus the
/// accept loop — so the same handler serves conformance connections and, later,
/// real `tmux attach` clients.
#[derive(Clone)]
pub struct NativeServer {
    state: Arc<Mutex<ServerState>>,
    /// Shared per-pane agent status, written by the [`AgentObserver`] and read by
    /// the format layer (`#{pane_agent*}`). A sibling of `state`, not part of it,
    /// so `ServerState` and the observability traits are untouched.
    ///
    /// [`AgentObserver`]: crate::integration::AgentObserver
    status: StatusHub,
    /// The observation worker owns the private event boundary. Keeping it in
    /// an `Arc` makes cloned servers share one worker and lets the worker stop
    /// when the final server handle is dropped.
    observation: Arc<ObservationRuntime>,
}

impl ServerObservability for NativeServer {
    fn pane_ids(&self) -> io::Result<Vec<PublicPaneId>> {
        let state = self
            .state
            .lock()
            .map_err(|_| io::Error::other("native server state mutex poisoned"))?;
        let mut ids = state
            .all_windows()
            .flat_map(|window| &window.panes)
            .map(|pane| PublicPaneId(pane.id))
            .collect::<Vec<_>>();
        ids.sort_unstable_by_key(|id| id.0);
        Ok(ids)
    }

    fn resolve_pane(&self, id: PublicPaneId) -> io::Result<Option<Arc<dyn PaneObservability>>> {
        let state = self
            .state
            .lock()
            .map_err(|_| io::Error::other("native server state mutex poisoned"))?;
        let pane = state
            .all_windows()
            .flat_map(|window| &window.panes)
            .find(|pane| pane.id == id.0)
            .map(|pane| pane.pane.observation());
        Ok(pane)
    }
}

impl NativeServer {
    fn from_state(state: ServerState, hook: Arc<dyn ObservationHook>) -> io::Result<NativeServer> {
        let observation_signal = state.observation_signal();
        let state = Arc::new(Mutex::new(state));
        let observation = Arc::new(ObservationRuntime::start(
            Arc::clone(&state),
            hook,
            observation_signal,
        )?);
        Ok(NativeServer {
            state,
            status: StatusHub::new(),
            observation,
        })
    }

    /// Build a server awaiting its first client. The first untargeted attach
    /// creates session 0 through the ordinary new-session path.
    pub fn new() -> io::Result<NativeServer> {
        Self::from_state(ServerState::empty(), Arc::new(NoopObservationHook))
    }

    /// Construct a native server with a first-party, unclassified observation
    /// sink. This is crate-private because the observation boundary is an
    /// internal integration seam; ordinary users use the tmux control plane.
    #[allow(dead_code)]
    pub(crate) fn with_observation_hook(
        state: ServerState,
        hook: Arc<dyn ObservationHook>,
    ) -> io::Result<NativeServer> {
        Self::from_state(state, hook)
    }

    /// Shared state handle (for embedding hmux in a larger app, e.g. querying
    /// sessions for agent detection).
    pub fn state(&self) -> Arc<Mutex<ServerState>> {
        Arc::clone(&self.state)
    }

    /// A clone of this server's status hub, so the [`AgentObserver`] can publish
    /// into the same hub the connection handlers read from.
    ///
    /// [`AgentObserver`]: crate::integration::AgentObserver
    pub fn status_hub(&self) -> StatusHub {
        self.status.clone()
    }

    pub(crate) fn enable_event_loop_pane_io(&self) -> io::Result<()> {
        self.state
            .lock()
            .map_err(|_| io::Error::other("native server state mutex poisoned"))?
            .set_pane_io_mode(PaneIoMode::EventLoop);
        self.observation.stop_worker();
        Ok(())
    }

    pub(crate) fn reconcile_event_observations(&self) -> io::Result<()> {
        self.observation.reconcile_once(&self.state)
    }

    pub(crate) fn try_event_pane_snapshot(&self) -> io::Result<Option<EventPaneSnapshot>> {
        let mut state = match self.state.try_lock() {
            Ok(state) => state,
            Err(std::sync::TryLockError::WouldBlock) => return Ok(None),
            Err(std::sync::TryLockError::Poisoned(_)) => {
                return Err(io::Error::other("native server state mutex poisoned"));
            }
        };
        let runtimes = state.take_event_pane_ios();
        let active = state.pane_runtime_ids().into_iter().collect();
        Ok(Some((runtimes, active)))
    }

    /// Whether the native server's lifecycle rules allow its event loop to end.
    ///
    /// This is deliberately separate from [`TmuxServer`], which only models a
    /// client protocol connection. The outer native runtime supplies this as
    /// its loop callback, matching tmux's `proc_loop(server_proc, server_loop)`.
    pub fn shutdown_requested(&self) -> bool {
        self.state
            .lock()
            .map(|mut state| {
                state.reap_exited_panes();
                state.shutdown_requested()
            })
            .unwrap_or(true)
    }

    /// Nonblocking lifecycle check for the readiness-loop thread.
    ///
    /// Protocol workers may hold the state mutex while awaiting a client
    /// response. The event loop must keep forwarding that response instead of
    /// waiting for the same mutex; a busy state therefore defers the shutdown
    /// decision to a later turn.
    pub(crate) fn event_loop_shutdown_requested(&self) -> bool {
        match self.state.try_lock() {
            Ok(mut state) => {
                state.reap_exited_panes();
                state.shutdown_requested()
            }
            Err(std::sync::TryLockError::WouldBlock) => false,
            Err(std::sync::TryLockError::Poisoned(_)) => true,
        }
    }

    pub(crate) fn try_reap_event_children(&self) -> bool {
        match self.state.try_lock() {
            Ok(mut state) => {
                state.reap_exited_panes();
                true
            }
            Err(std::sync::TryLockError::WouldBlock) => false,
            Err(std::sync::TryLockError::Poisoned(_)) => true,
        }
    }

    fn spawn_protocol_connection(&self) -> io::Result<UnixStream> {
        let (client, server) = UnixStream::pair()?;
        let (server_reader, server_writer) = split_stream(server)?;

        let state = Arc::clone(&self.state);
        let hub = self.status.clone();
        let observation = Arc::clone(&self.observation);
        thread::spawn(move || {
            // A dropped/broken client tears down only this handler.
            let _observation = observation;
            let _ = protocol::handle(server_reader, server_writer, state, hub);
        });

        Ok(client)
    }
}

impl TmuxServer for NativeServer {
    type Reader = ImsgReader;
    type Writer = ImsgWriter;

    fn connect(&self) -> io::Result<(Self::Reader, Self::Writer)> {
        // SCM_RIGHTS fd passing works across the socketpair, so the identify
        // handshake behaves as over a real listener connection.
        split_stream(self.spawn_protocol_connection()?)
    }
}

/// Stable identity of a pane for this native server lifetime.
///
/// This is intentionally distinct from the public v1 observability identity:
/// the event boundary is crate-private and is not an additional compatibility
/// contract.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct PaneId(pub(crate) u32);

/// An unclassified change in a pane owned by the native runtime.
#[allow(dead_code)]
#[derive(Clone)]
pub(crate) enum PaneEvent {
    /// A pane became observable. The handle is also the consumer's cacheable
    /// access path to the pane's current state.
    Added(PaneHandle),
    /// One or more PTY reads have been applied to the terminal state.
    Output(PaneId),
    /// The root child process has exited.
    Exited(PaneId),
    /// The pane was removed from the server tree.
    Removed(PaneId),
}

/// First-party sink for the native runtime's ordered pane events.
pub(crate) trait ObservationHook: Send + Sync {
    fn on_event(&self, event: PaneEvent) -> io::Result<()>;
}

struct NoopObservationHook;

impl ObservationHook for NoopObservationHook {
    fn on_event(&self, _event: PaneEvent) -> io::Result<()> {
        Ok(())
    }
}

/// The concrete lazy pane handle supplied in [`PaneEvent::Added`]. It owns an
/// `Arc` to the pane's terminal/process state, so it remains readable after
/// the pane is removed from the session tree.
#[derive(Clone)]
pub(crate) struct PaneHandle {
    id: PaneId,
    observation: Arc<pane::NativePaneObservation>,
}

#[allow(dead_code)]
impl PaneHandle {
    fn from_pane(id: PaneId, pane: &pane::Pane) -> PaneHandle {
        PaneHandle {
            id,
            observation: pane.observation_state(),
        }
    }

    pub(crate) fn id(&self) -> PaneId {
        self.id
    }

    /// Process facts are read from atomics and do not walk descendants or
    /// perform any agent-specific work.
    pub(crate) fn process(&self) -> io::Result<PaneProcess> {
        let (child_pid, exited) = self.observation.contract_process();
        Ok(PaneProcess { child_pid, exited })
    }

    /// Read the title only; this does not format the terminal screen.
    pub(crate) fn title(&self) -> io::Result<Option<String>> {
        self.observation.contract_title()
    }

    /// Read title and a bounded live-bottom tail under one terminal lock.
    pub(crate) fn terminal_tail(&self, max_rows: u16) -> io::Result<TerminalTail> {
        let (title, text) = self.observation.contract_terminal_tail(max_rows as usize)?;
        Ok(TerminalTail { title, text })
    }

    fn output_revision(&self) -> u64 {
        self.observation.contract_revision()
    }
}

/// Root process facts known by owning a pane's PTY child.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PaneProcess {
    pub(crate) child_pid: Option<u32>,
    pub(crate) exited: bool,
}

/// Coherent title and screen inputs for a screen-aware observer.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TerminalTail {
    pub(crate) title: Option<String>,
    pub(crate) text: String,
}

struct ObservationRuntime {
    stop: Arc<AtomicBool>,
    worker: Mutex<Option<JoinHandle<()>>>,
    signal: Arc<ObservationSignal>,
    hook: Arc<dyn ObservationHook>,
    previous: Arc<Mutex<HashMap<PaneId, ObservedPane>>>,
}

#[derive(Default)]
pub(crate) struct ObservationSignal {
    revision: Mutex<u64>,
    changed: Condvar,
}

impl ObservationSignal {
    pub(crate) fn notify(&self) {
        let mut revision = self
            .revision
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *revision = revision.wrapping_add(1);
        drop(revision);
        self.changed.notify_all();
    }

    fn revision(&self) -> u64 {
        *self
            .revision
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn wait_after(&self, revision: u64, stop: &AtomicBool) -> u64 {
        let current = self
            .revision
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let current = self
            .changed
            .wait_while(current, |current| {
                *current == revision && !stop.load(Ordering::Acquire)
            })
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *current
    }
}

impl ObservationRuntime {
    fn start(
        state: Arc<Mutex<ServerState>>,
        hook: Arc<dyn ObservationHook>,
        signal: Arc<ObservationSignal>,
    ) -> io::Result<ObservationRuntime> {
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker_signal = Arc::clone(&signal);
        let previous = Arc::new(Mutex::new(HashMap::new()));
        let worker_previous = Arc::clone(&previous);
        let worker_hook = Arc::clone(&hook);
        let worker = thread::Builder::new()
            .name("hmux-pane-observer".to_string())
            .spawn(move || {
                observe(
                    state,
                    worker_hook,
                    worker_stop,
                    worker_signal,
                    worker_previous,
                )
            })?;
        Ok(ObservationRuntime {
            stop,
            worker: Mutex::new(Some(worker)),
            signal,
            hook,
            previous,
        })
    }

    fn stop_worker(&self) {
        self.stop.store(true, Ordering::Release);
        self.signal.notify();
        if let Ok(mut worker) = self.worker.lock() {
            if let Some(worker) = worker.take() {
                let _ = worker.join();
            }
        }
    }

    fn reconcile_once(&self, state: &Arc<Mutex<ServerState>>) -> io::Result<()> {
        let current = pane_snapshot(state)?;
        let mut previous = self
            .previous
            .lock()
            .map_err(|_| io::Error::other("pane observation state mutex poisoned"))?;
        reconcile(&mut previous, current, self.hook.as_ref());
        Ok(())
    }
}

impl Drop for ObservationRuntime {
    fn drop(&mut self) {
        self.stop_worker();
    }
}

struct ObservedPane {
    handle: PaneHandle,
    revision: u64,
    exited: bool,
}

fn observe(
    state: Arc<Mutex<ServerState>>,
    hook: Arc<dyn ObservationHook>,
    stop: Arc<AtomicBool>,
    signal: Arc<ObservationSignal>,
    previous: Arc<Mutex<HashMap<PaneId, ObservedPane>>>,
) {
    let mut revision = signal.revision();
    while !stop.load(Ordering::Acquire) {
        match pane_snapshot(&state) {
            Ok(current) => {
                if let Ok(mut previous) = previous.lock() {
                    reconcile(&mut previous, current, hook.as_ref());
                }
            }
            Err(error) => tracing::warn!(target: "hmux::native", %error, "pane observation failed"),
        }
        revision = signal.wait_after(revision, &stop);
    }
}

fn pane_snapshot(state: &Arc<Mutex<ServerState>>) -> io::Result<HashMap<PaneId, ObservedPane>> {
    let state = state
        .lock()
        .map_err(|_| io::Error::other("native server state mutex poisoned"))?;
    let mut snapshot = HashMap::new();
    for window in state.all_windows() {
        for node in &window.panes {
            let handle = PaneHandle::from_pane(PaneId(node.id), &node.pane);
            let process = handle.process()?;
            snapshot.insert(
                handle.id(),
                ObservedPane {
                    revision: handle.output_revision(),
                    exited: process.exited,
                    handle,
                },
            );
        }
    }
    Ok(snapshot)
}

fn reconcile(
    previous: &mut HashMap<PaneId, ObservedPane>,
    current: HashMap<PaneId, ObservedPane>,
    hook: &dyn ObservationHook,
) {
    let mut ids: Vec<PaneId> = current.keys().copied().collect();
    ids.sort_unstable();
    for id in ids {
        let now = current.get(&id).expect("snapshot id present");
        match previous.get(&id) {
            None => {
                deliver(hook, PaneEvent::Added(now.handle.clone()));
                if now.revision != 0 {
                    deliver(hook, PaneEvent::Output(id));
                }
                if now.exited {
                    deliver(hook, PaneEvent::Exited(id));
                }
            }
            Some(was) => {
                if now.revision > was.revision {
                    deliver(hook, PaneEvent::Output(id));
                }
                if now.exited && !was.exited {
                    deliver(hook, PaneEvent::Exited(id));
                }
            }
        }
    }

    let mut removed: Vec<PaneId> = previous
        .keys()
        .filter(|id| !current.contains_key(id))
        .copied()
        .collect();
    removed.sort_unstable();
    for id in removed {
        deliver(hook, PaneEvent::Removed(id));
    }
    *previous = current;
}

fn deliver(hook: &dyn ObservationHook, event: PaneEvent) {
    if let Err(error) = hook.on_event(event) {
        tracing::warn!(target: "hmux::native", %error, "pane observation hook rejected event");
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::Mutex as StdMutex;
    use std::time::Duration;

    use crate::observability::v1::{PaneId, ServerObservability};

    use super::{NativeServer, NoopObservationHook, ObservationHook, PaneEvent, ServerState};

    fn server_with_test_session() -> NativeServer {
        NativeServer::from_state(
            ServerState::with_test_session().expect("test state"),
            std::sync::Arc::new(NoopObservationHook),
        )
        .expect("native server")
    }

    #[derive(Default)]
    struct RecordingHook(StdMutex<Vec<PaneEvent>>);

    impl ObservationHook for RecordingHook {
        fn on_event(&self, event: PaneEvent) -> io::Result<()> {
            self.0.lock().unwrap().push(event);
            Ok(())
        }
    }

    fn wait_until(mut predicate: impl FnMut() -> bool) {
        for _ in 0..100 {
            if predicate() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(predicate(), "observation event did not arrive");
    }

    #[test]
    fn private_observation_boundary_orders_events_and_keeps_handle_readable() {
        let hook = std::sync::Arc::new(RecordingHook::default());
        let server = NativeServer::with_observation_hook(
            ServerState::with_test_session().expect("default state"),
            hook.clone(),
        )
        .expect("native server");

        wait_until(|| {
            hook.0
                .lock()
                .unwrap()
                .iter()
                .any(|event| matches!(event, PaneEvent::Added(_)))
        });

        let handle = hook
            .0
            .lock()
            .unwrap()
            .iter()
            .find_map(|event| match event {
                PaneEvent::Added(handle) => Some(handle.clone()),
                _ => None,
            })
            .expect("added handle");
        assert_eq!(handle.id().0, 0);
        assert_eq!(handle.process().expect("process").child_pid, None);
        assert_eq!(handle.title().expect("title"), None);

        {
            let state = server.state();
            let state = state.lock().unwrap();
            state.window(0, 0).panes[0].pane.feed(b"first\r\nsecond");
        }
        wait_until(|| {
            hook.0
                .lock()
                .unwrap()
                .iter()
                .any(|event| matches!(event, PaneEvent::Output(id) if id.0 == 0))
        });
        let tail = handle.terminal_tail(1).expect("tail");
        assert_eq!(tail.text, "second");

        assert!(server.state().lock().unwrap().kill_session("0"));
        wait_until(|| {
            hook.0
                .lock()
                .unwrap()
                .iter()
                .any(|event| matches!(event, PaneEvent::Removed(id) if id.0 == 0))
        });

        let events = hook.0.lock().unwrap();
        let positions = events
            .iter()
            .enumerate()
            .filter_map(|(index, event)| match event {
                PaneEvent::Added(_) | PaneEvent::Output(_) | PaneEvent::Removed(_) => Some(index),
                PaneEvent::Exited(_) => None,
            })
            .collect::<Vec<_>>();
        assert!(matches!(events[positions[0]], PaneEvent::Added(_)));
        assert!(matches!(events[positions[1]], PaneEvent::Output(_)));
        assert!(matches!(
            events[*positions.last().unwrap()],
            PaneEvent::Removed(_)
        ));
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(
            handle.terminal_tail(1).expect("retained tail").text,
            "second"
        );
    }

    #[test]
    fn native_server_exposes_stable_pane_observation_handles() {
        let server = server_with_test_session();
        assert_eq!(server.pane_ids().expect("pane ids"), vec![PaneId(0)]);

        let pane = server
            .resolve_pane(PaneId(0))
            .expect("resolve pane")
            .expect("pane exists");
        assert_eq!(pane.process().expect("process").child_pid, None);
        assert_eq!(pane.output_revision().expect("revision"), 0);

        {
            let state = server.state();
            let state = state.lock().expect("server state");
            state.window(0, 0).panes[0]
                .pane
                .feed(b"one\r\ntwo\r\nthree");
        }

        assert_eq!(pane.output_revision().expect("revision"), 1);
        let tail = pane.last_lines(2).expect("screen tail");
        assert_eq!(tail.revision, 1);
        assert_eq!(tail.text, "two\nthree");

        let state = server.state();
        assert!(state.lock().expect("server state").kill_session("0"));
        assert!(server
            .resolve_pane(PaneId(0))
            .expect("resolve removed pane")
            .is_none());
        assert_eq!(pane.last_lines(1).expect("cached handle").text, "three");
    }

    #[test]
    fn unknown_pane_does_not_resolve() {
        let server = NativeServer::new().expect("native server");
        assert!(server.pane_ids().expect("pane ids").is_empty());
        assert!(server
            .resolve_pane(PaneId(99))
            .expect("resolve pane")
            .is_none());
    }
}

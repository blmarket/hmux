//! Shared tmux-compatible server engine built on libghostty-vt.
//!
//! This module owns sessions, windows, panes, command behavior, attach state,
//! and terminal emulation. The [`crate::event_loop`] runtime supplies
//! connection scheduling and readiness handling.
//!
//! The interactive attach path — compositing panes onto the client's tty — is
//! implemented in the `attach` module: on attach-identify the server takes the
//! client's tty fd and drives it directly via libghostty's VT formatter, with
//! input forwarding and resize/detach handling.
//!
//! The observation hook below is crate-private: the server emits unclassified
//! pane lifecycle/output events to first-party consumers, while those consumers
//! own process-tree walking, agent detection, classification, and status
//! publication. Hooks are called without holding the server or terminal locks;
//! pane handles retain shared terminal state and remain readable after removal.

pub mod attach;
#[path = "cmd-send-keys.rs"]
pub(crate) mod cmd_send_keys;
pub mod command;
pub(crate) mod control;
pub mod format;
#[path = "input-keys.rs"]
pub(crate) mod input_keys;
pub(crate) mod key;
pub mod latmon;
pub(crate) mod mouse;
pub mod options;
pub mod pane;
pub mod registry;
pub mod state;
pub mod status;
pub(crate) mod style;
pub(crate) mod task;
pub(crate) mod term;

use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};

use crate::integration::status::StatusHub;
use crate::observability::v1::{PaneId as PublicPaneId, PaneObservability, ServerObservability};

use pane::{PaneIo, PaneIoMode};
use state::ServerState;

type EventPaneSnapshot = (Vec<(u64, PaneIo)>, Vec<u64>);

/// Runtime-independent tmux server state and behavior.
///
/// State is shared across runtime adapters behind a mutex. A runtime chooses
/// how protocol clients and pane descriptors are scheduled without owning the
/// command or session implementation.
#[derive(Clone)]
pub struct Server {
    state: Arc<Mutex<ServerState>>,
    /// Shared per-pane agent status, written by the [`AgentObserver`] and read by
    /// format renderers (`#{pane_agent*}`). A sibling of
    /// `state`, not part of it, so `ServerState` and the observability traits are
    /// untouched.
    ///
    /// [`AgentObserver`]: crate::integration::AgentObserver
    status: StatusHub,
    observation: Arc<ObservationState>,
}

impl ServerObservability for Server {
    fn pane_ids(&self) -> io::Result<Vec<PublicPaneId>> {
        let state = self
            .state
            .lock()
            .map_err(|_| io::Error::other("server state mutex poisoned"))?;
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
            .map_err(|_| io::Error::other("server state mutex poisoned"))?;
        let pane = state
            .all_windows()
            .flat_map(|window| &window.panes)
            .find(|pane| pane.id == id.0)
            .map(|pane| pane.pane.observation());
        Ok(pane)
    }
}

impl Server {
    fn from_state(state: ServerState, hook: Arc<dyn ObservationHook>) -> io::Result<Server> {
        let state = Arc::new(Mutex::new(state));
        Ok(Server {
            state,
            status: StatusHub::new(),
            observation: Arc::new(ObservationState::new(hook)),
        })
    }

    /// Build a server awaiting its first client. The first untargeted attach
    /// creates session 0 through the ordinary new-session path.
    pub fn new() -> io::Result<Server> {
        let mut state = ServerState::empty();
        state.set_pane_io_mode(PaneIoMode::EventLoop);
        Self::from_state(state, Arc::new(NoopObservationHook))
    }

    /// Construct a server with a first-party, unclassified observation
    /// sink. This is crate-private because the observation boundary is an
    /// internal integration seam; ordinary users use the tmux control plane.
    #[allow(dead_code)]
    pub(crate) fn with_observation_hook(
        state: ServerState,
        hook: Arc<dyn ObservationHook>,
    ) -> io::Result<Server> {
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
            .map_err(|_| io::Error::other("server state mutex poisoned"))?
            .set_pane_io_mode(PaneIoMode::EventLoop);
        Ok(())
    }

    pub(crate) fn reconcile_event_observations(&self) -> io::Result<()> {
        self.observation.reconcile_once(&self.state)
    }

    /// Every `pipe-pane` child opened since the last call.
    pub(crate) fn take_new_pane_pipes(&self) -> Vec<crate::server::pane::PanePipeIo> {
        match self.state.try_lock() {
            Ok(mut state) => state.take_new_pane_pipes(),
            Err(_) => Vec::new(),
        }
    }

    /// Every `#()` job a format expansion has launched since the last call.
    pub(crate) fn take_pending_format_jobs(&self) -> Vec<crate::server::status::FormatJob> {
        self.state
            .lock()
            .map(|state| state.take_pending_format_jobs())
            .unwrap_or_default()
    }

    /// Recheck monitored windows for bell, activity, and silence alerts.
    ///
    /// tmux runs this check in its server event loop (`alerts_callback`), so a
    /// window is flagged whether or not any client is watching, and a client on
    /// a tty sees the flag as soon as it is raised. Driving it from the runtime
    /// loop rather than from a client keeps that property here: pane output
    /// wakes the loop, and the returned interval is how long the loop may sleep
    /// before a pending `monitor-silence` timer must be reconsidered.
    ///
    /// Recording a control checkpoint on change is what lets control clients
    /// report the new flags; the render invalidation raised by the check itself
    /// wakes attached clients to repaint their status line.
    ///
    /// A control client rechecks alerts on its own turns as well. The check
    /// only consumes pane observation deltas it has not seen, so whichever
    /// caller reaches a delta first raises the flags and the checkpoint.
    pub(crate) fn refresh_alerts(&self) -> io::Result<Option<std::time::Duration>> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| io::Error::other("server state mutex poisoned"))?;
        if state.refresh_alerts(std::time::Instant::now()) {
            state.record_control_checkpoint();
        }
        Ok(state.alert_poll_timeout())
    }

    /// tmux's `session_lock_timer`, polled once per server loop: lock the
    /// clients of any session idle past its `lock-after-time`. Returns when the
    /// next session falls due, so the loop can sleep until then.
    pub(crate) fn refresh_lock_timers(&self) -> io::Result<Option<std::time::Duration>> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| io::Error::other("server state mutex poisoned"))?;
        Ok(state.refresh_lock_timers())
    }

    /// tmux's per-loop `server_check_unattached` plus the `server_loop` exit
    /// test, run once the current batch of client events has been applied.
    pub(crate) fn enforce_lifecycle_policies(&self) -> io::Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| io::Error::other("server state mutex poisoned"))?;
        state.enforce_lifecycle_policies();
        // tmux's per-loop `recalculate_sizes`. Most size changes are applied by
        // whichever path caused them, but a client going away has no such path
        // — the window it sized has to be re-derived from the clients left.
        let _ = state.recalculate_sizes();
        state.process_pane_clipboard();
        state.process_pane_themes();
        command::run_deferred_notification_hooks(&mut state);
        Ok(())
    }


    pub(crate) fn try_event_pane_snapshot(&self) -> io::Result<Option<EventPaneSnapshot>> {
        let mut state = match self.state.try_lock() {
            Ok(state) => state,
            Err(std::sync::TryLockError::WouldBlock) => return Ok(None),
            Err(std::sync::TryLockError::Poisoned(_)) => {
                return Err(io::Error::other("server state mutex poisoned"));
            }
        };
        let runtimes = state.take_event_pane_ios();
        let active = state.pane_runtime_ids().into_iter().collect();
        Ok(Some((runtimes, active)))
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
        crate::server::pane::reap_orphans();
        match self.state.try_lock() {
            Ok(mut state) => {
                state.reap_exited_panes();
                true
            }
            Err(std::sync::TryLockError::WouldBlock) => false,
            Err(std::sync::TryLockError::Poisoned(_)) => true,
        }
    }
}

/// Stable identity of a pane for this server lifetime.
///
/// This is intentionally distinct from the public v1 observability identity:
/// the event boundary is crate-private and is not an additional compatibility
/// contract.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct PaneId(pub(crate) u32);

/// An unclassified change in a pane owned by the server engine.
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

/// First-party sink for the server engine's ordered pane events.
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

struct ObservationState {
    hook: Arc<dyn ObservationHook>,
    previous: Mutex<HashMap<PaneId, ObservedPane>>,
}

impl ObservationState {
    fn new(hook: Arc<dyn ObservationHook>) -> Self {
        Self {
            hook,
            previous: Mutex::new(HashMap::new()),
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

struct ObservedPane {
    handle: PaneHandle,
    revision: u64,
    exited: bool,
}

fn pane_snapshot(state: &Arc<Mutex<ServerState>>) -> io::Result<HashMap<PaneId, ObservedPane>> {
    let state = state
        .lock()
        .map_err(|_| io::Error::other("server state mutex poisoned"))?;
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
        tracing::warn!(target: "hmux::server", %error, "pane observation hook rejected event");
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::Mutex as StdMutex;

    use crate::observability::v1::{PaneId, ServerObservability};

    use super::{NoopObservationHook, ObservationHook, PaneEvent, Server, ServerState};

    fn server_with_test_session() -> Server {
        Server::from_state(
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

    #[test]
    fn private_observation_boundary_orders_events_and_keeps_handle_readable() {
        let hook = std::sync::Arc::new(RecordingHook::default());
        let server = Server::with_observation_hook(
            ServerState::with_test_session().expect("default state"),
            hook.clone(),
        )
        .expect("native server");

        server.reconcile_event_observations().unwrap();

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
        server.reconcile_event_observations().unwrap();
        let tail = handle.terminal_tail(1).expect("tail");
        assert_eq!(tail.text, "second");

        assert!(server.state().lock().unwrap().kill_session("0"));
        server.reconcile_event_observations().unwrap();

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
        let server = Server::new().expect("native server");
        assert!(server.pane_ids().expect("pane ids").is_empty());
        assert!(server
            .resolve_pane(PaneId(99))
            .expect("resolve pane")
            .is_none());
    }
}

//! Shared server-side registry of per-pane agent status.
//!
//! The [`AgentObserver`](super::AgentObserver) writes each pane's classified
//! [`AgentStatus`] here; readers — the format-variable layer (`#{pane_agent*}`)
//! and the long-poll push handler — read it back. A monotonic `revision` mirrors
//! the pane `output_revision` pattern and lets a long-poll client block for
//! "something newer than N" without busy-polling.
//!
//! This hub is a **sibling** of the native server's `ServerState`, not a part of
//! it, so neither `ServerState`'s shape nor the `observability::v1` traits change.
//! It is engine-agnostic state that the native runtime happens to own.

use std::collections::HashMap;
use std::io;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};
use std::sync::Weak;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

use crate::observability::v1::PaneId;
use crate::platform::{CurrentPlatform, OutputWakeup, Platform};

use super::AgentState;

/// Per-pane agent attribution: which agent (if any) runs in a pane and the
/// lifecycle state last classified for it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentStatus {
    /// Detector label: `"codex"`, `"claude"`, `"pi"`, or `""` when no agent
    /// is present.
    pub agent: &'static str,
    /// OS pid of the matched agent process, when process attribution found one.
    pub pid: Option<u32>,
    /// The agent's session id, resolved from its process metadata when the
    /// detector defines a source (for example a cwd-scoped Claude/Pi transcript
    /// or an active Codex rollout). `None` when unknown.
    pub session_id: Option<String>,
    /// The pane's classified lifecycle state.
    pub state: AgentState,
}

/// A map of pane id → agent status, as carried in a [`StatusSnapshot`] and used
/// by the format layer to expand `#{pane_agent*}`.
pub type PaneAgents = HashMap<PaneId, AgentStatus>;

/// A point-in-time copy of the hub: the `revision` it represents plus every
/// pane's status at that revision.
#[derive(Clone, Debug)]
pub struct StatusSnapshot {
    /// Monotonic counter, bumped on every observable change. Starts at 1 so a
    /// fresh client's `wait_after(0, …)` returns the current snapshot at once.
    pub revision: u64,
    /// Per-pane agent status at `revision`.
    pub panes: PaneAgents,
}

/// The shared registry. Cloneable; every clone refers to the same underlying
/// state (observer and connection handlers all hold clones).
#[derive(Clone)]
pub struct StatusHub(Arc<Inner>);

struct Inner {
    snapshot: Mutex<StatusSnapshot>,
    /// Signalled whenever `revision` advances, to wake parked long-poll waiters.
    changed: Condvar,
    /// Per-consumer pollable wakeups for attached status renderers.
    waiters: Mutex<Vec<Weak<StatusEvent>>>,
}

struct StatusEvent {
    wakeup: <CurrentPlatform as Platform>::OutputWakeup,
}

/// A coalescing, pollable notification for agent-status revision changes.
/// Each attached client owns one so draining cannot consume another client's
/// wakeup.
pub(crate) struct StatusSubscription {
    event: Arc<StatusEvent>,
}

impl StatusSubscription {
    pub(crate) fn as_fd(&self) -> BorrowedFd<'_> {
        self.event.wakeup.as_fd()
    }

    pub(crate) fn as_raw_fd(&self) -> RawFd {
        self.as_fd().as_raw_fd()
    }

    pub(crate) fn drain(&self) {
        let _ = self.event.wakeup.clear();
    }
}

impl StatusHub {
    /// Build an empty hub at revision 1.
    pub fn new() -> Self {
        StatusHub(Arc::new(Inner {
            snapshot: Mutex::new(StatusSnapshot {
                revision: 1,
                panes: PaneAgents::new(),
            }),
            changed: Condvar::new(),
            waiters: Mutex::new(Vec::new()),
        }))
    }

    /// Upsert a pane's status. Bumps the revision and wakes waiters only when the
    /// stored value actually changes, so a heartbeat re-publish of an unchanged
    /// status doesn't churn `revision`.
    pub fn publish(&self, pane: PaneId, status: AgentStatus) {
        let mut snap = self.lock();
        if snap.panes.get(&pane) == Some(&status) {
            return;
        }
        snap.panes.insert(pane, status);
        snap.revision += 1;
        drop(snap);
        self.notify_changed();
    }

    /// Drop a pane that no longer exists. Bumps the revision and wakes waiters
    /// only when a pane was actually present.
    pub fn remove(&self, pane: PaneId) {
        let mut snap = self.lock();
        if snap.panes.remove(&pane).is_none() {
            return;
        }
        snap.revision += 1;
        drop(snap);
        self.notify_changed();
    }

    /// Clone the current snapshot under the lock.
    pub fn snapshot(&self) -> StatusSnapshot {
        self.lock().clone()
    }

    /// Subscribe to revision changes with a pollable, initially signalled
    /// descriptor. Initial readiness closes the race between registration and
    /// the subscriber's first snapshot.
    pub(crate) fn subscribe(&self) -> io::Result<StatusSubscription> {
        let event = Arc::new(StatusEvent {
            wakeup: CurrentPlatform::new_output_wakeup()?,
        });
        self.0
            .waiters
            .lock()
            .map_err(|_| io::Error::other("agent status waiters mutex poisoned"))?
            .push(Arc::downgrade(&event));
        Ok(StatusSubscription { event })
    }

    /// Park until the revision advances past `since`, or `timeout` elapses, then
    /// return the current snapshot. Returns immediately when `revision > since`
    /// already; on timeout the returned snapshot's `revision` may still equal
    /// `since` (a liveness heartbeat). The hub mutex is released while parked, so
    /// a blocked long-poll never stalls other holders of a hub clone.
    pub fn wait_after(&self, since: u64, timeout: Duration) -> StatusSnapshot {
        let guard = self.lock();
        let (guard, _timed_out) = self
            .0
            .changed
            .wait_timeout_while(guard, timeout, |snap| snap.revision <= since)
            .unwrap_or_else(|poison| poison.into_inner());
        guard.clone()
    }

    /// Lock the snapshot, recovering from a poisoned mutex rather than panicking
    /// the observer or a connection thread.
    fn lock(&self) -> MutexGuard<'_, StatusSnapshot> {
        self.0.snapshot.lock().unwrap_or_else(|p| p.into_inner())
    }

    fn notify_changed(&self) {
        self.0.changed.notify_all();
        let Ok(mut waiters) = self.0.waiters.lock() else {
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
}

impl Default for StatusHub {
    fn default() -> Self {
        StatusHub::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::fd::RawFd;
    use std::thread;
    use std::time::Instant;

    fn status(agent: &'static str, state: AgentState) -> AgentStatus {
        AgentStatus {
            agent,
            pid: None,
            session_id: None,
            state,
        }
    }

    #[test]
    fn publish_and_remove_bump_revision() {
        let hub = StatusHub::new();
        assert_eq!(hub.snapshot().revision, 1);

        hub.publish(PaneId(0), status("claude", AgentState::Working));
        let snap = hub.snapshot();
        assert_eq!(snap.revision, 2);
        assert_eq!(
            snap.panes[&PaneId(0)],
            status("claude", AgentState::Working)
        );

        // Re-publishing the identical status is a no-op for the revision.
        hub.publish(PaneId(0), status("claude", AgentState::Working));
        assert_eq!(hub.snapshot().revision, 2);

        // A real change bumps it.
        hub.publish(PaneId(0), status("claude", AgentState::Idle));
        assert_eq!(hub.snapshot().revision, 3);

        hub.remove(PaneId(0));
        let snap = hub.snapshot();
        assert_eq!(snap.revision, 4);
        assert!(snap.panes.is_empty());

        // Removing an absent pane is a no-op.
        hub.remove(PaneId(0));
        assert_eq!(hub.snapshot().revision, 4);
    }

    #[test]
    fn wait_after_returns_immediately_when_ahead() {
        let hub = StatusHub::new();
        // revision starts at 1, so a fresh since=0 wait returns at once.
        let snap = hub.wait_after(0, Duration::from_secs(30));
        assert_eq!(snap.revision, 1);
    }

    #[test]
    fn wait_after_times_out_as_heartbeat() {
        let hub = StatusHub::new();
        let start = Instant::now();
        let snap = hub.wait_after(1, Duration::from_millis(50));
        assert!(start.elapsed() >= Duration::from_millis(40));
        // Unchanged: the revision still equals `since` — a liveness heartbeat.
        assert_eq!(snap.revision, 1);
    }

    #[test]
    fn wait_after_wakes_on_change() {
        let hub = StatusHub::new();
        let writer = hub.clone();
        let waiter = thread::spawn(move || hub.wait_after(1, Duration::from_secs(5)));
        // Let the waiter park, then publish.
        thread::sleep(Duration::from_millis(50));
        writer.publish(PaneId(7), status("codex", AgentState::Blocked));
        let snap = waiter.join().expect("waiter joined");
        assert_eq!(snap.revision, 2);
        assert_eq!(snap.panes[&PaneId(7)], status("codex", AgentState::Blocked));
    }

    fn is_readable(fd: RawFd) -> bool {
        let mut pollfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        unsafe { libc::poll(&mut pollfd, 1, 0) == 1 }
    }

    #[test]
    fn subscription_wakes_only_for_observable_changes() {
        let hub = StatusHub::new();
        let subscription = hub.subscribe().expect("subscribe");
        assert!(is_readable(subscription.as_raw_fd()));
        subscription.drain();
        assert!(!is_readable(subscription.as_raw_fd()));

        hub.publish(PaneId(1), status("codex", AgentState::Working));
        assert!(is_readable(subscription.as_raw_fd()));
        subscription.drain();

        hub.publish(PaneId(1), status("codex", AgentState::Working));
        assert!(!is_readable(subscription.as_raw_fd()));

        hub.publish(PaneId(1), status("codex", AgentState::Blocked));
        assert!(is_readable(subscription.as_raw_fd()));
    }
}

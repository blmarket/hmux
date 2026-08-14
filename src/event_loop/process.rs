//! Task-driven child-exit notification.

use std::io;

use libc::SIGCHLD;

use crate::server::Server;

use crate::sync::yield_now;
use hmux_rt::{JoinHandle, Signals, TaskHandle};

/// The loop's app-lifetime child-signal task.
///
/// A singleton, not a `Handle`: unlike the dynamically owned `PaneHandle`
/// there is no cancel-on-drop. The loop tears the task down with an explicit
/// [`shutdown`] and otherwise only drops this when the runtime itself is
/// about to go, taking the detached task with it.
///
/// [`shutdown`]: Self::shutdown
pub(crate) struct ChildSignalSingleton {
    task: JoinHandle<()>,
}

impl ChildSignalSingleton {
    /// Ask the task to stop; it finishes on its next turn.
    pub(crate) fn shutdown(&self) {
        self.task.cancel();
    }

    pub(crate) fn is_alive(&self) -> bool {
        !self.task.is_finished()
    }
}

/// Watch for `SIGCHLD` on the loop and reap exited children as it arrives.
pub(crate) fn spawn(tasks: &TaskHandle, server: Server) -> io::Result<ChildSignalSingleton> {
    let mut signals = Signals::new(tasks, &[SIGCHLD])?;
    let task = tasks.spawn_join(async move {
        while signals.recv().await.is_ok() {
            // The reap can lose to a concurrent borrow of the server's state;
            // give the loop a turn and try again, the same retry the actor's
            // `Retry` event carried.
            while !server.try_reap_event_children() {
                yield_now().await;
            }
        }
    });
    Ok(ChildSignalSingleton { task })
}

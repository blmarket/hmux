//! Task-driven `SIGINT`/`SIGTERM` teardown.
//!
//! The runtime's [`Signals`] source delivers both on the loop, so the
//! shutdown decision is made like every other event rather than by a thread
//! parked in `sigwait`.

use std::io;

use libc::{SIGINT, SIGTERM};
use tracing::info;

use hmux_rt::{Signals, TaskHandle};

/// Watch for `SIGINT`/`SIGTERM` on the loop. The task lives as long as the
/// process does; the first signal ends it.
pub(crate) fn spawn(tasks: &TaskHandle) -> io::Result<()> {
    let mut signals = Signals::new(tasks, &[SIGINT, SIGTERM])?;
    tasks.spawn(async move {
        if signals.recv().await.is_err() {
            return;
        }
        info!("shutting down");
        // Exactly what the `sigwait` teardown did, and what tmux does:
        // leave the socket pathname in place — it is only ever unlinked
        // when a new server binds it.
        std::process::exit(0);
    });
    Ok(())
}

//! Task-driven `SIGINT`/`SIGTERM` teardown.
//!
//! The runtime's [`Signals`] source delivers both on the loop, so the
//! shutdown decision is made like every other event rather than by a thread
//! parked in `sigwait`.

use std::io;

use libc::{SIGINT, SIGTERM};
use tracing::info;

use hmux_rt::{Signals, TaskHandle};

use crate::server::Server;

/// Watch for `SIGINT`/`SIGTERM` on the loop. The task lives as long as the
/// process does; the first signal ends it.
pub(crate) fn spawn(tasks: &TaskHandle, server: Server) -> io::Result<()> {
    let mut signals = Signals::new(tasks, &[SIGINT, SIGTERM])?;
    tasks.spawn(async move {
        if signals.recv().await.is_err() {
            return;
        }
        info!("shutting down");
        // tmux's `server_signal`: the signal asks the server to exit rather
        // than ending the process where it stands, so every client is told
        // and the loop leaves through its own drain. The socket pathname
        // stays in place — it is only ever unlinked when a new server binds
        // it.
        server.request_signal_shutdown();
    });
    Ok(())
}

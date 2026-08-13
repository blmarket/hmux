//! Task-driven `SIGINT`/`SIGTERM` teardown.
//!
//! Both signals write into a self-pipe a task waits on, so the shutdown
//! decision is made on the loop like every other event rather than by a thread
//! parked in `sigwait`. The signals stay deliverable process-wide: nothing
//! blocks them, so a child that never clears its mask is unaffected.

use std::io::{self, Read};
use std::os::fd::AsFd;
use std::os::unix::net::UnixStream;

use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::low_level::{pipe, unregister};
use signal_hook::SigId;
use tracing::info;

use hmux_rt::{AsyncFd, Interest, TaskHandle};

/// The self-pipe both signals write into, and its registrations' lifetime.
struct TermSignalSource {
    reader: UnixStream,
    registrations: Vec<SigId>,
}

impl TermSignalSource {
    fn new() -> io::Result<Self> {
        let (reader, writer) = UnixStream::pair()?;
        reader.set_nonblocking(true)?;
        let mut registrations = Vec::new();
        for signal in [SIGINT, SIGTERM] {
            let writer = writer.try_clone()?;
            match pipe::register(signal, writer) {
                Ok(registration) => registrations.push(registration),
                Err(error) => {
                    for registration in registrations {
                        unregister(registration);
                    }
                    return Err(error);
                }
            }
        }
        Ok(Self {
            reader,
            registrations,
        })
    }

    /// Whether a signal actually arrived, as opposed to a spurious wakeup.
    fn drain(&mut self) -> bool {
        let mut bytes = [0u8; 64];
        let mut delivered = false;
        loop {
            match self.reader.read(&mut bytes) {
                Ok(0) => return delivered,
                Ok(_) => delivered = true,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => return delivered,
            }
        }
    }
}

impl Drop for TermSignalSource {
    fn drop(&mut self) {
        for registration in std::mem::take(&mut self.registrations) {
            unregister(registration);
        }
    }
}

/// Watch for `SIGINT`/`SIGTERM` on the loop. The task lives as long as the
/// process does; the first signal ends it.
pub(crate) fn spawn(tasks: &TaskHandle) -> io::Result<()> {
    let mut source = TermSignalSource::new()?;
    let handle = tasks.clone();
    tasks.spawn(async move {
        let Ok(readiness) = AsyncFd::new(&handle, source.reader.as_fd(), Interest::READABLE)
        else {
            return;
        };
        loop {
            readiness.readiness().await;
            if source.drain() {
                info!("shutting down");
                // Exactly what the `sigwait` teardown did, and what tmux does:
                // leave the socket pathname in place — it is only ever unlinked
                // when a new server binds it.
                std::process::exit(0);
            }
        }
    });
    Ok(())
}

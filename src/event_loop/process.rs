//! Task-driven child-exit notification.

use std::io::{self, Read};
use std::os::fd::AsFd;
use std::os::unix::net::UnixStream;

use signal_hook::consts::signal::SIGCHLD;
use signal_hook::low_level::{pipe, unregister};
use signal_hook::SigId;

use crate::server::Server;

use hmux_rt::{yield_now, AsyncFd, Interest, JoinHandle, TaskHandle};

/// The self-pipe `SIGCHLD` writes into, and its registration's lifetime.
struct ChildSignalSource {
    reader: UnixStream,
    registration: SigId,
}

impl ChildSignalSource {
    fn new() -> io::Result<Self> {
        let (reader, writer) = UnixStream::pair()?;
        reader.set_nonblocking(true)?;
        let registration = pipe::register(SIGCHLD, writer)?;
        Ok(Self {
            reader,
            registration,
        })
    }

    fn drain(&mut self) {
        let mut bytes = [0u8; 64];
        loop {
            match self.reader.read(&mut bytes) {
                Ok(0) => break,
                Ok(_) => continue,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }
    }
}

impl Drop for ChildSignalSource {
    fn drop(&mut self) {
        unregister(self.registration);
    }
}

/// The loop's handle to the child-signal task.
pub(crate) struct ChildSignalHandle {
    task: JoinHandle<()>,
}

impl ChildSignalHandle {
    /// Ask the task to stop; it finishes on its next turn.
    pub(crate) fn shutdown(&self) {
        self.task.cancel();
    }

    pub(crate) fn is_alive(&self) -> bool {
        !self.task.is_finished()
    }
}

impl Drop for ChildSignalHandle {
    fn drop(&mut self) {
        self.task.cancel();
    }
}

/// Watch for `SIGCHLD` on the loop and reap exited children as it arrives.
pub(crate) fn spawn(tasks: &TaskHandle, server: Server) -> io::Result<ChildSignalHandle> {
    let mut source = ChildSignalSource::new()?;
    let handle = tasks.clone();
    let task = tasks.spawn_join(async move {
        let Ok(readiness) = AsyncFd::new(&handle, source.reader.as_fd(), Interest::READABLE)
        else {
            return;
        };
        loop {
            readiness.readiness().await;
            source.drain();
            // The reap can lose to a concurrent borrow of the server's state;
            // give the loop a turn and try again, the same retry the actor's
            // `Retry` event carried.
            while !server.try_reap_event_children() {
                yield_now().await;
            }
        }
    });
    Ok(ChildSignalHandle { task })
}

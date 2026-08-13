//! Task-driven child-exit notification.

use std::cell::Cell;
use std::io::{self, Read};
use std::os::fd::AsFd;
use std::os::unix::net::UnixStream;
use std::rc::Rc;

use signal_hook::consts::signal::SIGCHLD;
use signal_hook::low_level::{pipe, unregister};
use signal_hook::SigId;

use crate::server::Server;

use hmux_rt::{select, yield_now, AsyncFd, Either, Interest, Notify, TaskHandle};

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
    shutdown: Notify,
    done: Rc<Cell<bool>>,
}

impl ChildSignalHandle {
    /// Ask the task to stop; it finishes on its next turn.
    pub(crate) fn shutdown(&self) {
        self.shutdown.notify();
    }

    pub(crate) fn is_alive(&self) -> bool {
        !self.done.get()
    }
}

/// Watch for `SIGCHLD` on the loop and reap exited children as it arrives.
pub(crate) fn spawn(tasks: &TaskHandle, server: Server) -> io::Result<ChildSignalHandle> {
    let mut source = ChildSignalSource::new()?;
    let shutdown = Notify::new();
    let done = Rc::new(Cell::new(false));
    let task_shutdown = shutdown.clone();
    let task_done = Rc::clone(&done);
    let handle = tasks.clone();
    tasks.spawn(async move {
        let Ok(readiness) = AsyncFd::new(&handle, source.reader.as_fd(), Interest::READABLE)
        else {
            task_done.set(true);
            return;
        };
        loop {
            match select(readiness.readiness(), task_shutdown.notified()).await {
                Either::First(_) => {
                    source.drain();
                    // The reap can lose to a concurrent borrow of the server's
                    // state; give the loop a turn and try again, the same
                    // retry the actor's `Retry` event carried.
                    while !server.try_reap_event_children() {
                        yield_now().await;
                    }
                }
                Either::Second(()) => break,
            }
        }
        task_done.set(true);
    });
    Ok(ChildSignalHandle { shutdown, done })
}

//! Task-driven Unix listener.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::io;
use std::os::fd::AsFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::rc::Rc;

use tracing::warn;

use hmux_rt::{select, yield_now, AsyncFd, Either, Interest, Notify, TaskHandle};

/// Accepted sockets waiting for the server adapter to create client actors.
#[derive(Clone, Default)]
pub(crate) struct AcceptedClients {
    inner: Rc<RefCell<VecDeque<UnixStream>>>,
}

impl AcceptedClients {
    pub(crate) fn pop_front(&self) -> Option<UnixStream> {
        self.inner.borrow_mut().pop_front()
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.inner.borrow().len()
    }

    fn push_back(&self, stream: UnixStream) {
        self.inner.borrow_mut().push_back(stream);
    }
}

/// The loop's handle to the listener task.
pub(crate) struct ListenerHandle {
    accepted: AcceptedClients,
    shutdown: Notify,
    done: Rc<Cell<bool>>,
}

impl ListenerHandle {
    pub(crate) fn pop_accepted(&self) -> Option<UnixStream> {
        self.accepted.pop_front()
    }

    #[cfg(test)]
    pub(crate) fn accepted_len(&self) -> usize {
        self.accepted.len()
    }

    /// Stop accepting; the task finishes on its next turn.
    pub(crate) fn shutdown(&self) {
        self.shutdown.notify();
    }

    pub(crate) fn is_alive(&self) -> bool {
        !self.done.get()
    }
}

/// Accept clients on the loop, `accept_budget` per turn.
pub(crate) fn spawn(
    tasks: &TaskHandle,
    listener: UnixListener,
    accept_budget: usize,
) -> io::Result<ListenerHandle> {
    assert!(accept_budget > 0, "listener accept budget must be nonzero");
    listener.set_nonblocking(true)?;
    let accepted = AcceptedClients::default();
    let shutdown = Notify::new();
    let done = Rc::new(Cell::new(false));
    let task_accepted = accepted.clone();
    let task_shutdown = shutdown.clone();
    let task_done = Rc::clone(&done);
    let handle = tasks.clone();
    tasks.spawn(async move {
        let Ok(readiness) = AsyncFd::new(&handle, listener.as_fd(), Interest::READABLE) else {
            task_done.set(true);
            return;
        };
        'run: loop {
            match select(task_shutdown.notified(), readiness.readiness()).await {
                Either::First(()) => break,
                Either::Second(_) => {
                    // Edge-style: accept until `WouldBlock` before waiting
                    // again, yielding after every budget's worth so a
                    // connection flood cannot starve the rest of the loop.
                    'draining: loop {
                        let mut accepted = 0;
                        while accepted < accept_budget {
                            match listener.accept() {
                                Ok((stream, _)) => {
                                    task_accepted.push_back(stream);
                                    accepted += 1;
                                }
                                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                                    break 'draining;
                                }
                                Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                                    continue;
                                }
                                Err(error) => {
                                    warn!(error = %error, "accept failed");
                                    break 'draining;
                                }
                            }
                        }
                        // A yield that a shutdown can still win: the backlog
                        // does not hold the listener past its stop.
                        match select(task_shutdown.notified(), yield_now()).await {
                            Either::First(()) => break 'run,
                            Either::Second(()) => {}
                        }
                    }
                }
            }
        }
        task_done.set(true);
    });
    Ok(ListenerHandle {
        accepted,
        shutdown,
        done,
    })
}

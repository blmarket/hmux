//! Task-driven Unix listener.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::os::fd::AsFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::rc::Rc;

use tracing::warn;

use crate::sync::yield_now;
use hmux_rt::{AsyncFd, Interest, JoinHandle, TaskHandle};

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
    task: JoinHandle<()>,
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
        self.task.cancel();
    }

    pub(crate) fn is_alive(&self) -> bool {
        !self.task.is_finished()
    }
}

impl Drop for ListenerHandle {
    fn drop(&mut self) {
        self.task.cancel();
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
    let task_accepted = accepted.clone();
    let handle = tasks.clone();
    let task = tasks.spawn_join(async move {
        let Ok(readiness) = AsyncFd::new(&handle, listener.as_fd(), Interest::READABLE) else {
            return;
        };
        loop {
            readiness.readiness().await;
            // Edge-style: accept until `WouldBlock` before waiting again,
            // yielding after every budget's worth so a connection flood
            // cannot starve the rest of the loop.
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
                        Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                        Err(error) => {
                            warn!(error = %error, "accept failed");
                            break 'draining;
                        }
                    }
                }
                yield_now().await;
            }
        }
    });
    Ok(ListenerHandle { accepted, task })
}

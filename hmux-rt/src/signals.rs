//! Task-driven Unix signal delivery.
//!
//! Registered signals write into a self-pipe a task waits on, so the reaction
//! runs on the loop like every other event rather than inside a handler or a
//! thread parked in `sigwait`. The signals stay deliverable process-wide:
//! nothing blocks them, so a child that never clears its mask is unaffected.

use std::ffi::c_int;
use std::io::{self, Read};
use std::os::fd::AsFd;
use std::os::unix::net::UnixStream;

use signal_hook::low_level::{pipe, unregister};
use signal_hook::SigId;

use crate::reactor::Interest;
use crate::tasks::{AsyncFd, TaskHandle};

/// Async source of one or more Unix signals: the self-pipe their handlers
/// write into, and its registrations' lifetime.
///
/// Dropping the source unregisters the handlers.
pub struct Signals {
    handle: TaskHandle,
    reader: UnixStream,
    registrations: Vec<SigId>,
    /// Created on the first [`recv`], the earliest moment a task is running
    /// to own the registration.
    ///
    /// [`recv`]: Self::recv
    fd: Option<AsyncFd>,
}

impl Signals {
    /// Register `signals` to write into a fresh self-pipe.
    ///
    /// Callable from anywhere; the descriptor joins the loop on the first
    /// [`recv`].
    ///
    /// [`recv`]: Self::recv
    pub fn new(handle: &TaskHandle, signals: &[c_int]) -> io::Result<Self> {
        let (reader, writer) = UnixStream::pair()?;
        reader.set_nonblocking(true)?;
        let mut registrations = Vec::new();
        for &signal in signals {
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
            handle: handle.clone(),
            reader,
            registrations,
            fd: None,
        })
    }

    /// Wait until at least one registered signal has been delivered since the
    /// last call; spurious wakeups are absorbed here.
    ///
    /// Must be awaited from inside a task, like any descriptor wait.
    pub async fn recv(&mut self) -> io::Result<()> {
        if self.fd.is_none() {
            self.fd = Some(AsyncFd::new(
                &self.handle,
                self.reader.as_fd(),
                Interest::READABLE,
            )?);
        }
        let fd = self.fd.as_ref().expect("created above");
        loop {
            fd.readiness().await;
            if drain(&mut self.reader) {
                return Ok(());
            }
        }
    }
}

impl Drop for Signals {
    fn drop(&mut self) {
        for registration in std::mem::take(&mut self.registrations) {
            unregister(registration);
        }
    }
}

/// Whether a signal actually arrived, as opposed to a spurious wakeup.
fn drain(reader: &mut UnixStream) -> bool {
    let mut bytes = [0u8; 64];
    let mut delivered = false;
    loop {
        match reader.read(&mut bytes) {
            Ok(0) => return delivered,
            Ok(_) => delivered = true,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return delivered,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::TaskRuntime;

    #[test]
    fn recv_resolves_on_delivery() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let mut signals = Signals::new(&runtime.handle(), &[libc::SIGUSR2]).expect("register");
        // Delivered before the descriptor ever joins the loop: the first
        // `recv` still has to see it.
        unsafe { libc::raise(libc::SIGUSR2) };
        runtime.block_on(async move {
            signals.recv().await.expect("recv");
        });
    }

    #[test]
    fn recv_resolves_on_later_delivery() {
        let mut runtime = TaskRuntime::new().expect("runtime");
        let mut signals = Signals::new(&runtime.handle(), &[libc::SIGUSR1]).expect("register");
        // Delivered while the loop is parked on the descriptor: the handler's
        // pipe write is what wakes it.
        let raiser = std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(50));
            unsafe { libc::raise(libc::SIGUSR1) };
        });
        runtime.block_on(async move {
            signals.recv().await.expect("recv");
        });
        raiser.join().expect("raiser");
    }
}

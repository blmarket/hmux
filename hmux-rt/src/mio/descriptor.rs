//! Readiness bridge for synchronous, nonblocking descriptor consumers.
//!
//! This adapter supports legacy accept/recvmsg/tty call sites without running
//! callbacks inside the driver. It uses the same waiter and cancellation machinery
//! as byte operations. The consumer must bound its syscalls after each wait.
use std::future::{Future, poll_fn};
use std::io;
use std::os::fd::{AsRawFd, OwnedFd};
use std::pin::pin;
use std::rc::{Rc, Weak};
use std::task::Poll;

use super::{Handle, Io};
use crate::Handle as _;

/// A leased descriptor for a synchronous nonblocking consumer.
/// Regular files are always ready and may perform blocking disk I/O.
pub struct Descriptor {
    core: Weak<super::runtime::Core>,
    fd: Rc<OwnedFd>,
    io: Option<Io>,
}

impl Descriptor {
    /// Register a descriptor. Regular files bypass the kernel readiness poller.
    pub(super) fn new(handle: &Handle, fd: Rc<OwnedFd>) -> io::Result<Self> {
        handle.core.check()?;
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: live descriptor and valid output storage.
        if unsafe { libc::fstat(fd.as_raw_fd(), stat.as_mut_ptr()) } < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: fstat initialized stat.
        let regular = unsafe { stat.assume_init() }.st_mode & libc::S_IFMT == libc::S_IFREG;
        let io = if regular {
            None
        } else {
            match handle.io(fd.clone()) {
                Ok(io) => Some(io),
                // Devices such as /dev/null support immediate I/O but epoll
                // rejects them. The zero-time poll below verifies readiness.
                Err(e) if e.kind() == io::ErrorKind::Unsupported => None,
                Err(e) => return Err(e),
            }
        };
        Ok(Self {
            core: Rc::downgrade(&handle.core),
            fd,
            io,
        })
    }

    fn probe(&self, read: bool, write: bool) -> io::Result<(bool, bool)> {
        self.core
            .upgrade()
            .ok_or_else(super::runtime::invalid)?
            .check()?;
        probe_fd(self.fd.as_raw_fd(), read, write)
    }
}

impl crate::AsyncFd for Descriptor {
    /// Wait for requested directions. The returned pair is (readable, writable).
    /// Cancellation consumes no bytes. One waiter per direction is permitted.
    async fn ready(&self, read: bool, write: bool) -> io::Result<(bool, bool)> {
        if !read && !write {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        match self.probe(read, write) {
            Err(e)
                if e.kind() == io::ErrorKind::WouldBlock
                    || e.kind() == io::ErrorKind::Interrupted => {}
            result => return result,
        }
        let source = self
            .io
            .as_ref()
            .ok_or_else(|| io::Error::other("file not ready"))?;
        let mut reader = pin!(source.read_with(|| self.probe(true, false)));
        let mut writer = pin!(source.write_with(|| self.probe(false, true)));
        poll_fn(|cx| {
            if read && let Poll::Ready(result) = reader.as_mut().poll(cx) {
                return Poll::Ready(result);
            }
            if write && let Poll::Ready(result) = writer.as_mut().poll(cx) {
                return Poll::Ready(result);
            }
            Poll::Pending
        })
        .await
    }
}

fn probe_fd(raw: std::os::fd::RawFd, read: bool, write: bool) -> io::Result<(bool, bool)> {
    let mut pfd = libc::pollfd {
        fd: raw,
        events: (if read { libc::POLLIN } else { 0 }) | (if write { libc::POLLOUT } else { 0 }),
        revents: 0,
    };
    // SAFETY: one initialized pollfd; zero timeout never blocks.
    let result = unsafe { libc::poll(&mut pfd, 1, 0) };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    if pfd.revents & libc::POLLNVAL != 0 {
        return Err(io::Error::from_raw_os_error(libc::EBADF));
    }
    let terminal = pfd.revents & (libc::POLLERR | libc::POLLHUP) != 0;
    let ready = (
        read && (terminal || pfd.revents & libc::POLLIN != 0),
        write && (terminal || pfd.revents & libc::POLLOUT != 0),
    );
    if ready.0 || ready.1 {
        Ok(ready)
    } else {
        Err(io::ErrorKind::WouldBlock.into())
    }
}

impl Io {
    fn probe(&self, read: bool, write: bool) -> io::Result<(bool, bool)> {
        self.state.check()?;
        probe_fd(self.state.raw(), read, write)
    }
}

impl crate::AsyncFd for Io {
    /// Wait for requested directions. The returned pair is (readable, writable).
    /// Cancellation consumes no bytes. One waiter per direction is permitted.
    async fn ready(&self, read: bool, write: bool) -> io::Result<(bool, bool)> {
        if !read && !write {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        match self.probe(read, write) {
            Err(e)
                if e.kind() == io::ErrorKind::WouldBlock
                    || e.kind() == io::ErrorKind::Interrupted => {}
            result => return result,
        }
        let mut reader = pin!(self.read_with(|| self.probe(true, false)));
        let mut writer = pin!(self.write_with(|| self.probe(false, true)));
        poll_fn(|cx| {
            if read && let Poll::Ready(result) = reader.as_mut().poll(cx) {
                return Poll::Ready(result);
            }
            if write && let Poll::Ready(result) = writer.as_mut().poll(cx) {
                return Poll::Ready(result);
            }
            Poll::Pending
        })
        .await
    }
}

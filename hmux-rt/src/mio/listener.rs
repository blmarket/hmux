use std::io;
use std::os::fd::{FromRawFd, OwnedFd};

use super::Io;

/// An owned Unix stream listener. Readiness and retries are internal to accept.
pub struct Listener {
    source: Io,
}

impl Listener {
    pub(super) fn new(source: Io) -> Self {
        Self { source }
    }
}

impl crate::AsyncAccept for Listener {
    async fn accept(&self) -> io::Result<OwnedFd> {
        self.source
            .read_with(|| {
                // SAFETY: the operation validates the owned listener before
                // calling us. No peer address is requested. Accept never blocks.
                #[cfg(any(target_os = "linux", target_os = "android", target_os = "freebsd"))]
                let raw = unsafe {
                    libc::accept4(
                        self.source.state.raw(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC,
                    )
                };
                #[cfg(not(any(
                    target_os = "linux",
                    target_os = "android",
                    target_os = "freebsd"
                )))]
                let raw = unsafe {
                    libc::accept(
                        self.source.state.raw(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    )
                };
                if raw < 0 {
                    return Err(io::Error::last_os_error());
                }
                // SAFETY: accept transferred a new descriptor to this caller.
                let fd = unsafe { OwnedFd::from_raw_fd(raw) };
                #[cfg(not(any(
                    target_os = "linux",
                    target_os = "android",
                    target_os = "freebsd"
                )))]
                {
                    // SAFETY: fd is live; these calls only query/set its flags.
                    let flags = unsafe { libc::fcntl(raw, libc::F_GETFL) };
                    if flags < 0
                        || unsafe { libc::fcntl(raw, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
                        || unsafe { libc::fcntl(raw, libc::F_SETFD, libc::FD_CLOEXEC) } < 0
                    {
                        return Err(io::Error::last_os_error());
                    }
                }
                Ok(fd)
            })
            .await
    }
}

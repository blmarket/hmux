//! Reusable callback adapters, independent of any application process globals.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum WatchInterest {
    Read,
    Write,
    ReadWrite,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum WatchMode {
    Once,
    Persistent,
}

/// Register a raw descriptor through a checked duplicate. Invalid raw numbers
/// produce an OS error instead of constructing an invalid BorrowedFd.
pub(crate) fn register_fd(
    handle: &crate::TaskHandle,
    fd: std::os::fd::RawFd,
    interest: crate::Interest,
) -> std::io::Result<crate::AsyncFd> {
    use std::os::fd::{AsFd, FromRawFd, OwnedFd};
    let copy = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 0) };
    if copy < 0 {
        return Err(std::io::Error::last_os_error());
    }
    let copy = unsafe { OwnedFd::from_raw_fd(copy) };
    crate::AsyncFd::new(handle, copy.as_fd(), interest)
}

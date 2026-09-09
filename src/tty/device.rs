use std::ffi::{CStr, CString};
use std::io;
use std::os::fd::RawFd;

pub(crate) fn terminal_device_name(fd: RawFd) -> io::Result<CString> {
    read_device_name(fd, vec![0; 64])
}

fn read_device_name(fd: RawFd, mut bytes: Vec<u8>) -> io::Result<CString> {
    loop {
        let error = unsafe { libc::ttyname_r(fd, bytes.as_mut_ptr().cast(), bytes.len()) };
        match error {
            0 => {
                return CStr::from_bytes_until_nul(&bytes)
                    .map(CStr::to_owned)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
            }
            libc::ERANGE => {
                let capacity = bytes
                    .len()
                    .checked_mul(2)
                    .ok_or_else(|| io::Error::from_raw_os_error(libc::ERANGE))?;
                bytes.resize(capacity, 0);
            }
            error => return Err(io::Error::from_raw_os_error(error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

    fn terminal() -> (OwnedFd, OwnedFd) {
        let mut master = -1;
        let mut slave = -1;
        assert_eq!(
            unsafe {
                libc::openpty(
                    &raw mut master,
                    &raw mut slave,
                    core::ptr::null_mut(),
                    core::ptr::null(),
                    core::ptr::null(),
                )
            },
            0
        );
        unsafe { (OwnedFd::from_raw_fd(master), OwnedFd::from_raw_fd(slave)) }
    }

    #[test]
    fn names_survive_other_lookups_and_small_storage_grows() {
        let (_master, slave) = terminal();
        let (_other_master, other_slave) = terminal();
        let first = terminal_device_name(slave.as_raw_fd()).unwrap();
        let second = terminal_device_name(other_slave.as_raw_fd()).unwrap();
        assert_ne!(first, second);
        assert_eq!(
            read_device_name(slave.as_raw_fd(), vec![0; 1]).unwrap(),
            first
        );
        drop(slave);
        assert_eq!(
            terminal_device_name(other_slave.as_raw_fd()).unwrap(),
            second
        );
        assert!(!first.is_empty());
    }

    #[test]
    fn invalid_and_nonterminal_descriptors_return_errors() {
        assert_eq!(
            terminal_device_name(-1).unwrap_err().raw_os_error(),
            Some(libc::EBADF)
        );
        let file = std::fs::File::open("/dev/null").unwrap();
        assert_eq!(
            terminal_device_name(file.as_raw_fd())
                .unwrap_err()
                .raw_os_error(),
            Some(libc::ENOTTY)
        );
    }
}

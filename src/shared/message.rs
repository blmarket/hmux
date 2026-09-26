//! Authoritative client/server message identifiers.

use super::abi::{pid_t, size_t, uint32_t};
use std::collections::VecDeque;
use std::os::fd::OwnedFd;
use std::ptr::NonNull;
pub type msgtype = ::core::ffi::c_uint;

pub const MSG_WRITE_DONE: msgtype = 308;
pub const MSG_READ_CANCEL: msgtype = 307;
pub const MSG_WRITE_CLOSE: msgtype = 306;
pub const MSG_WRITE_READY: msgtype = 305;
pub const MSG_WRITE: msgtype = 304;
pub const MSG_WRITE_OPEN: msgtype = 303;
pub const MSG_READ_DONE: msgtype = 302;
pub const MSG_READ: msgtype = 301;
pub const MSG_READ_OPEN: msgtype = 300;
pub const MSG_FLAGS: msgtype = 218;
pub const MSG_EXEC: msgtype = 217;
pub const MSG_WAKEUP: msgtype = 216;
pub const MSG_UNLOCK: msgtype = 215;
pub const MSG_SUSPEND: msgtype = 214;
pub const MSG_OLDSTDOUT: msgtype = 213;
pub const MSG_OLDSTDIN: msgtype = 212;
pub const MSG_OLDSTDERR: msgtype = 211;
pub const MSG_SHUTDOWN: msgtype = 210;
pub const MSG_SHELL: msgtype = 209;
pub const MSG_RESIZE: msgtype = 208;
pub const MSG_READY: msgtype = 207;
pub const MSG_LOCK: msgtype = 206;
pub const MSG_EXITING: msgtype = 205;
pub const MSG_EXITED: msgtype = 204;
pub const MSG_EXIT: msgtype = 203;
pub const MSG_DETACHKILL: msgtype = 202;
pub const MSG_DETACH: msgtype = 201;
pub const MSG_COMMAND: msgtype = 200;
pub const MSG_IDENTIFY_TERMINFO: msgtype = 112;
pub const MSG_IDENTIFY_LONGFLAGS: msgtype = 111;
pub const MSG_IDENTIFY_STDOUT: msgtype = 110;
pub const MSG_IDENTIFY_FEATURES: msgtype = 109;
pub const MSG_IDENTIFY_CWD: msgtype = 108;
pub const MSG_IDENTIFY_CLIENTPID: msgtype = 107;
pub const MSG_IDENTIFY_DONE: msgtype = 106;
pub const MSG_IDENTIFY_ENVIRON: msgtype = 105;
pub const MSG_IDENTIFY_STDIN: msgtype = 104;
pub const MSG_IDENTIFY_OLDCWD: msgtype = 103;
pub const MSG_IDENTIFY_TTYNAME: msgtype = 102;
pub const MSG_IDENTIFY_TERM: msgtype = 101;
pub const MSG_IDENTIFY_FLAGS: msgtype = 100;
pub const MSG_VERSION: msgtype = 12;

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::mem::{align_of, size_of};

    #[test]
    fn message_domain_matches_translated_c_baseline() {
        assert_eq!(size_of::<msgtype>(), 4);
        assert_eq!(align_of::<msgtype>(), 4);
        assert_eq!(MSG_VERSION, 12);
        assert_eq!(MSG_IDENTIFY_FLAGS, 100);
        assert_eq!(MSG_COMMAND, 200);
        assert_eq!(MSG_WRITE_DONE, 308);
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct imsg_hdr {
    pub type_0: uint32_t,
    pub len: uint32_t,
    pub peerid: uint32_t,
    pub pid: uint32_t,
}

pub const IMSG_HEADER_SIZE: usize = ::core::mem::size_of::<imsg_hdr>();

pub const MAX_IMSGSIZE: ::core::ffi::c_int = 16384 as ::core::ffi::c_int;

pub const PROTOCOL_VERSION: ::core::ffi::c_int = 8 as ::core::ffi::c_int;

#[repr(C)]
pub struct ibuf {
    pub buf: *mut ::core::ffi::c_uchar,
    pub size: size_t,
    pub max: size_t,
    pub wpos: size_t,
    pub rpos: size_t,
    pub fd: ::core::ffi::c_int,
    pub(crate) storage: Option<Box<[u8]>>,
}

impl Drop for ibuf {
    fn drop(&mut self) {
        unsafe {
            let saved_errno = *crate::src::ffi::libc::__errno_location();
            if self.fd >= 0 {
                crate::src::ffi::libc::close(self.fd);
                self.fd = -1;
            }
            if let Some(mut bytes) = self.storage.take() {
                bytes.fill(0);
                drop(bytes);
            }
            self.buf = ::core::ptr::null_mut();
            *crate::src::ffi::libc::__errno_location() = saved_errno;
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct imsg {
    pub hdr: imsg_hdr,
    pub data: *mut ::core::ffi::c_void,
    pub buf: *mut ibuf,
}

#[repr(C)]
pub struct ibufqueue {
    pub bufs: ibufqueue_bufs,
}

#[repr(C)]
pub struct msgbuf {
    pub bufs: ibufqueue,
    pub rbufs: ibufqueue,
    pub rbuf: *mut ::core::ffi::c_char,
    pub rpmsg: *mut ibuf,
    pub readhdr:
        Option<Box<dyn FnMut(&[u8], Option<OwnedFd>) -> (Option<NonNull<ibuf>>, Option<OwnedFd>)>>,
    pub roff: size_t,
    pub hdrsize: size_t,
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct imsgbuf {
    pub w: *mut msgbuf,
    pub pid: pid_t,
    pub maxsize: uint32_t,
    pub fd: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
}

pub struct ibufqueue_bufs {
    entries: VecDeque<Box<ibuf>>,
}

impl ibufqueue_bufs {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = *mut ibuf> + '_ {
        self.entries
            .iter()
            .map(|buf| (&**buf as *const ibuf).cast_mut())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn push_back_raw(&mut self, buf: *mut ibuf) {
        self.entries.push_back(unsafe { Box::from_raw(buf) });
    }

    pub(crate) fn push_front_raw(&mut self, buf: *mut ibuf) {
        self.entries.push_front(unsafe { Box::from_raw(buf) });
    }

    pub(crate) fn pop_front_raw(&mut self) -> Option<*mut ibuf> {
        self.entries.pop_front().map(Box::into_raw)
    }

    pub(crate) fn pop_front_owned(&mut self) -> Option<Box<ibuf>> {
        self.entries.pop_front()
    }

    pub(crate) fn front(&self) -> Option<*mut ibuf> {
        self.entries
            .front()
            .map(|buf| (&**buf as *const ibuf).cast_mut())
    }

    pub(crate) fn append(&mut self, other: &mut Self) {
        self.entries.append(&mut other.entries);
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }
}

impl ibufqueue {
    pub fn new() -> Self {
        Self {
            bufs: ibufqueue_bufs::new(),
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_command {
    pub argc: ::core::ffi::c_int,
}

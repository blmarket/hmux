//! Authoritative client/server message identifiers.

use super::abi::{pid_t, size_t, uint32_t};
use std::collections::VecDeque;
use std::os::fd::OwnedFd;
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

pub(crate) enum IbufStorage<'a> {
    Owned(Box<[u8]>),
    Borrowed(&'a [u8]),
}

impl IbufStorage<'_> {
    pub fn as_slice(&self) -> &[u8] {
        match self {
            Self::Owned(bytes) => bytes,
            Self::Borrowed(bytes) => bytes,
        }
    }

    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }

    pub(crate) fn as_mut_slice(&mut self) -> Option<&mut [u8]> {
        match self {
            Self::Owned(bytes) => Some(bytes),
            Self::Borrowed(_) => None,
        }
    }
}

/// An opaque ibuf record. Use the accessors rather than relying on a C layout.
/// The lifetime parameter keeps borrowed byte storage tied to its source.
pub struct ibuf<'a> {
    pub(crate) storage: IbufStorage<'a>,
    pub(crate) max: size_t,
    pub(crate) wpos: size_t,
    pub(crate) rpos: size_t,
    pub(crate) fd: ::core::ffi::c_int,
}

impl<'a> ibuf<'a> {
    pub fn borrowed(bytes: &'a [u8]) -> Self {
        Self {
            storage: IbufStorage::Borrowed(bytes),
            max: 0,
            wpos: bytes.len(),
            rpos: 0,
            fd: -1,
        }
    }

    pub fn from_ibuf(from: &'a ibuf<'_>) -> Self {
        let start = from.rpos;
        let end = from.wpos;
        let bytes = from.storage.as_slice().get(start..end).unwrap_or(&[]);
        Self::borrowed(bytes)
    }

    pub fn take_view(&mut self, len: usize) -> Option<ibuf<'_>> {
        let start = self.rpos;
        let end = start.checked_add(len)?;
        if end > self.wpos || end > self.storage_len() {
            return None;
        }
        self.rpos = end;
        let bytes = self.storage.as_slice().get(start..end)?;
        Some(ibuf::borrowed(bytes))
    }

    pub fn is_owned(&self) -> bool {
        self.storage.is_owned()
    }

    pub fn size(&self) -> usize {
        self.wpos.saturating_sub(self.rpos)
    }

    pub fn unread(&self) -> &[u8] {
        let bytes = self.storage.as_slice();
        let end = self.wpos.min(bytes.len());
        let start = self.rpos.min(end);
        &bytes[start..end]
    }

    pub fn skip(&mut self, len: usize) -> bool {
        let Some(end) = self.rpos.checked_add(len) else {
            return false;
        };
        if end > self.wpos {
            return false;
        }
        self.rpos = end;
        true
    }

    pub fn rewind(&mut self) {
        self.rpos = 0;
    }

    pub fn storage_len(&self) -> usize {
        self.storage.as_slice().len()
    }

    pub(crate) fn replace_owned(&mut self, bytes: Box<[u8]>) {
        if let IbufStorage::Owned(mut old) =
            ::core::mem::replace(&mut self.storage, IbufStorage::Owned(bytes))
        {
            old.fill(0);
        }
    }
}

impl ibuf<'static> {
    /// Reclaim a raw buffer only after checking that its byte storage is owned.
    ///
    /// # Safety
    /// A non-null `buf` must point to a uniquely owned heap allocation that
    /// was created as an `ibuf<'static>` and has not already been reclaimed.
    pub(crate) unsafe fn from_raw_owned(buf: *mut Self) -> Option<Box<Self>> {
        if buf.is_null() || !(*buf).is_owned() {
            return None;
        }
        Some(Box::from_raw(buf))
    }
}

impl Drop for ibuf<'_> {
    fn drop(&mut self) {
        unsafe {
            let saved_errno = *crate::src::ffi::libc::__errno_location();
            if self.fd >= 0 {
                crate::src::ffi::libc::close(self.fd);
                self.fd = -1;
            }
            let storage = ::core::mem::replace(&mut self.storage, IbufStorage::Borrowed(&[]));
            if let IbufStorage::Owned(mut bytes) = storage {
                bytes.fill(0);
                drop(bytes);
            }
            *crate::src::ffi::libc::__errno_location() = saved_errno;
        }
    }
}

/// Raw compatibility spelling for `ibuf<'static>`.
///
/// The static lifetime does not prove that the storage is owned; use checked
/// ownership boundaries before reclaiming or queueing its raw pointer.
pub type OwnedIbuf = ibuf<'static>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct imsg {
    pub hdr: imsg_hdr,
    pub data: *mut ::core::ffi::c_void,
    pub buf: *mut OwnedIbuf,
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
    pub rpmsg: *mut OwnedIbuf,
    pub readhdr:
        Option<Box<dyn FnMut(&[u8], Option<OwnedFd>) -> (Option<Box<OwnedIbuf>>, Option<OwnedFd>)>>,
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
    entries: VecDeque<Box<OwnedIbuf>>,
}

impl ibufqueue_bufs {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = *mut OwnedIbuf> + '_ {
        self.entries
            .iter()
            .map(|buf| (&**buf as *const OwnedIbuf).cast_mut())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) unsafe fn push_back_raw(&mut self, buf: *mut OwnedIbuf) -> bool {
        let Some(buf) = OwnedIbuf::from_raw_owned(buf) else {
            return false;
        };
        self.entries.push_back(buf);
        true
    }

    pub(crate) unsafe fn push_front_raw(&mut self, buf: *mut OwnedIbuf) -> bool {
        let Some(buf) = OwnedIbuf::from_raw_owned(buf) else {
            return false;
        };
        self.entries.push_front(buf);
        true
    }

    pub(crate) fn pop_front_raw(&mut self) -> Option<*mut OwnedIbuf> {
        self.entries.pop_front().map(Box::into_raw)
    }

    pub(crate) fn pop_front_owned(&mut self) -> Option<Box<OwnedIbuf>> {
        self.entries.pop_front()
    }

    pub(crate) fn front(&self) -> Option<*mut OwnedIbuf> {
        self.entries
            .front()
            .map(|buf| (&**buf as *const OwnedIbuf).cast_mut())
    }

    pub(crate) fn append(&mut self, other: &mut Self) {
        self.entries.append(&mut other.entries);
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Make a read-only view whose lifetime is tied to the source slice.
///
/// A view cannot outlive its source:
///
/// ```compile_fail
/// use hmux2::src::shared::message::ibuf_from_buffer;
/// let bytes = vec![1, 2, 3];
/// let view = ibuf_from_buffer(&bytes);
/// drop(bytes);
/// assert_eq!(view.unread(), &[1, 2, 3]);
/// ```
pub fn ibuf_from_buffer(data: &[u8]) -> ibuf<'_> {
    ibuf::borrowed(data)
}

/// Make a read-only view whose lifetime is tied to the source buffer borrow.
pub fn ibuf_from_ibuf<'a>(from: &'a ibuf<'_>) -> ibuf<'a> {
    ibuf::from_ibuf(from)
}

/// Advance the source cursor and return a view of the consumed bytes.
///
/// The mutable borrow stays active for as long as the subview exists:
///
/// ```compile_fail
/// use hmux2::src::shared::message::{ibuf_from_buffer, ibuf_get_ibuf};
/// let bytes = [1, 2, 3];
/// let mut source = ibuf_from_buffer(&bytes);
/// let view = ibuf_get_ibuf(&mut source, 1).unwrap();
/// source.rewind();
/// assert_eq!(view.unread(), &[1]);
/// ```
pub fn ibuf_get_ibuf<'a>(from: &'a mut ibuf<'_>, len: usize) -> Option<ibuf<'a>> {
    from.take_view(len)
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

mod imsg_buffer;
mod message;

pub use message::{
    imsg, imsg_hdr, msgtype, MSG_COMMAND, MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT,
    MSG_EXITED, MSG_EXITING, MSG_FLAGS, MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD,
    MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON, MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_LONGFLAGS,
    MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO,
    MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_READ, MSG_READY, MSG_READ_CANCEL, MSG_READ_DONE,
    MSG_READ_OPEN, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN, MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION,
    MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_DONE, MSG_WRITE_OPEN, MSG_WRITE_READY,
};
pub(crate) use message::{imsgbuf, msg_command, IMSG_HEADER_SIZE, MAX_IMSGSIZE, PROTOCOL_VERSION};
use message::{msgbuf, OwnedIbuf};

use crate::src::ffi::libc::getpid;
use crate::src::shared::abi::uint32_t;
use crate::src::shared::abi::*;
use crate::src::shared::errno::{EBADMSG, EINVAL, ENOMEM, ERANGE};
use imsg_buffer::{
    ibuf_add, ibuf_close, ibuf_data, ibuf_dynamic, ibuf_fd_avail, ibuf_fd_get, ibuf_fd_set,
    ibuf_open, ibuf_read, ibuf_set_h32, ibuf_size, ibuf_write, msgbuf_get, msgbuf_new_reader_owned,
    msgbuf_queuelen, msgbuf_read, msgbuf_write,
};
use std::os::fd::OwnedFd;

const IMSG_ALLOW_FDPASS: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
const IMSG_FD_MARK: ::core::ffi::c_uint = 0x80000000 as ::core::ffi::c_uint;
pub(crate) fn imsgbuf_init(
    imsgbuf: &mut imsgbuf,
    fd: ::core::ffi::c_int,
) -> Result<(), ::core::ffi::c_int> {
    let maxsize = MAX_IMSGSIZE as uint32_t;
    let msgbuf = msgbuf_new_reader_owned(IMSG_HEADER_SIZE, move |header, fd| {
        imsg_parse_hdr(header, maxsize, fd)
    })?;
    imsgbuf.w = Some(msgbuf);
    imsgbuf.pid = unsafe { getpid() } as pid_t;
    imsgbuf.maxsize = maxsize;
    imsgbuf.fd = fd;
    imsgbuf.flags = 0;
    Ok(())
}
pub(crate) fn imsgbuf_allow_fdpass(imsgbuf: &mut imsgbuf) {
    imsgbuf.flags |= IMSG_ALLOW_FDPASS;
}
pub(crate) fn imsgbuf_read(
    imsgbuf: &mut imsgbuf,
) -> Result<::core::ffi::c_int, ::core::ffi::c_int> {
    let Some(msgbuf) = imsgbuf.w.as_deref_mut() else {
        return Err(EINVAL);
    };
    if imsgbuf.flags & IMSG_ALLOW_FDPASS != 0 {
        msgbuf_read(imsgbuf.fd, msgbuf)
    } else {
        ibuf_read(imsgbuf.fd, msgbuf)
    }
}
pub(crate) fn imsgbuf_write(imsgbuf: &mut imsgbuf) -> Result<(), ::core::ffi::c_int> {
    let Some(msgbuf) = imsgbuf.w.as_deref_mut() else {
        return Err(EINVAL);
    };
    if imsgbuf.flags & IMSG_ALLOW_FDPASS != 0 {
        msgbuf_write(imsgbuf.fd, msgbuf)
    } else {
        ibuf_write(imsgbuf.fd, msgbuf)
    }
}
pub(crate) fn imsgbuf_flush(imsgbuf: &mut imsgbuf) -> Result<(), ::core::ffi::c_int> {
    while imsgbuf_queuelen(imsgbuf) > 0 {
        imsgbuf_write(imsgbuf)?;
    }
    Ok(())
}
pub(crate) fn imsgbuf_clear(imsgbuf: &mut imsgbuf) {
    imsgbuf.w = None;
}
pub(crate) fn imsgbuf_queuelen(imsgbuf: &imsgbuf) -> uint32_t {
    imsgbuf.w.as_deref().map_or(0, msgbuf_queuelen)
}
pub(crate) fn imsgbuf_get(imsgbuf: &mut imsgbuf) -> Result<Option<imsg>, ::core::ffi::c_int> {
    let Some(msgbuf) = imsgbuf.w.as_deref_mut() else {
        return Ok(None);
    };
    let Some(mut buf) = msgbuf_get(msgbuf) else {
        return Ok(None);
    };
    let Some(mut hdr) = decode_imsg_hdr(ibuf_data(&buf)) else {
        return Err(EBADMSG);
    };
    if !buf.skip(IMSG_HEADER_SIZE) {
        return Err(EBADMSG);
    }
    hdr.len = (hdr.len & !IMSG_FD_MARK) as uint32_t;
    let mut data = Vec::new();
    data.try_reserve_exact(buf.size()).map_err(|_| ENOMEM)?;
    data.extend_from_slice(buf.unread());
    let fd = ibuf_fd_get(&mut buf);
    Ok(Some(imsg { hdr, data, fd }))
}
pub(crate) fn imsg_get_fd(imsg: &mut imsg) -> Option<OwnedFd> {
    imsg.fd.take()
}
pub(crate) fn imsg_compose(
    imsgbuf: &mut imsgbuf,
    type_0: uint32_t,
    id: uint32_t,
    pid: pid_t,
    fd: Option<OwnedFd>,
    data: &[u8],
) -> Result<(), ::core::ffi::c_int> {
    let mut wbuf = imsg_create(imsgbuf, type_0, id, pid, data.len())?;
    ibuf_add(&mut wbuf, data)?;
    ibuf_fd_set(&mut wbuf, fd)?;
    imsg_close(imsgbuf, wbuf)
}
fn imsg_create(
    imsgbuf: &imsgbuf,
    type_0: uint32_t,
    id: uint32_t,
    pid: pid_t,
    datalen: usize,
) -> Result<Box<OwnedIbuf>, ::core::ffi::c_int> {
    let mut hdr: imsg_hdr = imsg_hdr {
        type_0: 0,
        len: 0,
        peerid: 0,
        pid: 0,
    };
    let Some(datalen) = datalen.checked_add(IMSG_HEADER_SIZE) else {
        return Err(ERANGE);
    };
    if datalen > imsgbuf.maxsize as usize {
        return Err(ERANGE);
    }
    hdr.len = 0 as uint32_t;
    hdr.type_0 = type_0;
    hdr.peerid = id;
    hdr.pid = pid as uint32_t;
    if hdr.pid == 0 as uint32_t {
        hdr.pid = imsgbuf.pid as uint32_t;
    }
    let mut wbuf = ibuf_dynamic(datalen, imsgbuf.maxsize as usize)?;
    ibuf_add(&mut wbuf, &encode_imsg_hdr(hdr))?;
    Ok(wbuf)
}
fn imsg_close(imsgbuf: &mut imsgbuf, mut msg: Box<OwnedIbuf>) -> Result<(), ::core::ffi::c_int> {
    let mut len = ibuf_size(&msg) as uint32_t;
    if ibuf_fd_avail(&msg) {
        len |= IMSG_FD_MARK;
    }
    ibuf_set_h32(&mut msg, 4, len as uint64_t)?;
    let Some(msgbuf) = imsgbuf.w.as_deref_mut() else {
        return Err(EINVAL);
    };
    ibuf_close(msgbuf, msg)
}
fn imsg_parse_hdr(
    header: &[u8],
    maxsize: uint32_t,
    fd: Option<OwnedFd>,
) -> Result<(Box<OwnedIbuf>, Option<OwnedFd>), ::core::ffi::c_int> {
    let Some(hdr) = decode_imsg_hdr(header) else {
        return Err(EBADMSG);
    };
    let len = hdr.len & !IMSG_FD_MARK;
    if (len as usize) < IMSG_HEADER_SIZE || len > maxsize {
        return Err(ERANGE);
    }
    let mut b = ibuf_open(len as size_t)?;
    if hdr.len & IMSG_FD_MARK != 0 {
        ibuf_fd_set(&mut b, fd)?;
        Ok((b, None))
    } else {
        Ok((b, fd))
    }
}

fn decode_imsg_hdr(bytes: &[u8]) -> Option<imsg_hdr> {
    if bytes.len() < IMSG_HEADER_SIZE {
        return None;
    }
    let read_u32 = |offset: usize| -> Option<u32> {
        let bytes: [u8; 4] = bytes.get(offset..offset + 4)?.try_into().ok()?;
        Some(u32::from_ne_bytes(bytes))
    };
    Some(imsg_hdr {
        type_0: read_u32(0)?,
        len: read_u32(4)?,
        peerid: read_u32(8)?,
        pid: read_u32(12)?,
    })
}

fn encode_imsg_hdr(hdr: imsg_hdr) -> [u8; IMSG_HEADER_SIZE] {
    let mut bytes = [0; IMSG_HEADER_SIZE];
    bytes[0..4].copy_from_slice(&hdr.type_0.to_ne_bytes());
    bytes[4..8].copy_from_slice(&hdr.len.to_ne_bytes());
    bytes[8..12].copy_from_slice(&hdr.peerid.to_ne_bytes());
    bytes[12..16].copy_from_slice(&hdr.pid.to_ne_bytes());
    bytes
}

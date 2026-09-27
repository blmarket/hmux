//! C-call-site compatibility for runtime-owned byte storage.
use crate::src::format::bytes::write_cstr;
use crate::src::shared::abi::{size_t, ssize_t};
pub use hmux_buffer::SegmentedBuf as evbuffer;
use hmux_buffer::{Buf, BufMut, Buffer, LineEnding, SegmentedBuf as ByteBuffer};
use std::ffi::{c_char, c_int, c_void, CStr};

pub unsafe fn evbuffer_new() -> *mut ByteBuffer {
    Box::into_raw(Box::new(ByteBuffer::default()))
}
pub unsafe fn evbuffer_free(b: *mut ByteBuffer) {
    if !b.is_null() {
        drop(Box::from_raw(b));
    }
}
pub fn evbuffer_get_length(b: &ByteBuffer) -> size_t {
    b.remaining() as size_t
}
pub unsafe fn evbuffer_add(b: *mut ByteBuffer, data: *const c_void, len: size_t) -> c_int {
    if len != 0 {
        (*b).put_slice(std::slice::from_raw_parts(data.cast(), len));
    }
    super::wake_buffer(b);
    0
}
pub unsafe fn evbuffer_drain(b: *mut ByteBuffer, len: size_t) -> c_int {
    let count = len.min((*b).remaining());
    (*b).advance(count);
    super::wake_buffer(b);
    0
}
pub unsafe fn evbuffer_pullup(b: *mut ByteBuffer, size: ssize_t) -> *mut u8 {
    if !(*b).has_remaining() || (size >= 0 && size as usize > (*b).remaining()) {
        std::ptr::null_mut()
    } else {
        let count = if size < 0 {
            (*b).remaining()
        } else {
            size as usize
        };
        (*b).pullup(count).unwrap().as_mut_ptr()
    }
}
pub unsafe fn evbuffer_read(b: *mut ByteBuffer, fd: c_int, limit: c_int) -> c_int {
    let count = if limit < 0 {
        65536
    } else {
        (limit as usize).min(65536)
    };
    let mut bytes = Vec::<u8>::with_capacity(count);
    let n = libc::read(fd, bytes.as_mut_ptr().cast(), count);
    if n > 0 {
        // read initialized exactly n bytes of the allocation.
        bytes.set_len(n as usize);
        (*b).put(ByteBuffer::from(bytes));
        super::wake_buffer(b);
    }
    n as c_int
}
pub unsafe fn evbuffer_write(b: *mut ByteBuffer, fd: c_int) -> c_int {
    let mut chunks = [libc::iovec {
        iov_base: std::ptr::null_mut(),
        iov_len: 0,
    }; 64];
    let mut count = 0;
    let mut remaining = 65536;
    for chunk in (*b).chunks().take(64) {
        let len = chunk.len().min(remaining);
        chunks[count] = libc::iovec {
            iov_base: chunk.as_ptr() as *mut c_void,
            iov_len: len,
        };
        count += 1;
        remaining -= len;
        if remaining == 0 {
            break;
        }
    }
    let n = libc::writev(fd, chunks.as_ptr(), count as c_int);
    if n > 0 {
        (*b).advance(n as usize);
        super::wake_buffer(b);
    }
    n as c_int
}

unsafe fn read_line(b: *mut ByteBuffer, ending: LineEnding) -> Option<Vec<u8>> {
    let Some(mut line) = (*b).read_line(ending) else {
        return None;
    };
    line.push(0);

    super::wake_buffer(b);
    Some(line)
}
pub unsafe fn evbuffer_readln(b: *mut ByteBuffer) -> Option<Vec<u8>> {
    read_line(b, LineEnding::Lf)
}

pub unsafe fn evbuffer_readline(b: *mut ByteBuffer) -> Option<Vec<u8>> {
    read_line(b, LineEnding::Legacy)
}
pub unsafe fn evbuffer_add_formatted(
    b: *mut ByteBuffer,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) -> c_int {
    let Some(mut formatted) = format_buffer(write) else {
        return -1;
    };
    let count = formatted.remaining();
    (*b).append(&mut formatted);
    super::wake_buffer(b);
    count as c_int
}

/// Keep a trailing NUL in spare capacity, excluded from the readable payload.
/// Moving this segment with append preserves its allocation and capacity.
fn format_buffer(
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) -> Option<ByteBuffer> {
    let mut bytes = crate::src::format::bytes::format_bytes_with(write).ok()?;
    if bytes.len() > c_int::MAX as usize {
        return None;
    }
    bytes.try_reserve(1).ok()?;
    bytes.push(0);
    bytes.pop();
    Some(ByteBuffer::from(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    unsafe fn append_formatted(
        destination: *mut ByteBuffer,
        write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
    ) -> c_int {
        let Some(mut formatted) = format_buffer(write) else {
            return -1;
        };
        let count = formatted.remaining();
        let pointer = formatted.chunk().as_ptr();
        let destination = unsafe { &mut *destination };
        destination.append(&mut formatted);
        assert_eq!(formatted.remaining(), 0);
        if count != 0 {
            assert_eq!(destination.chunks().next_back().unwrap().as_ptr(), pointer);
        }
        count as c_int
    }

    #[test]
    fn formatted_storage_survives_append_and_terminator_commit() {
        for size in [1, 1023, 1024, 4095, 4096, 4097, 40076, 100_000] {
            let text = CString::new("x".repeat(size)).unwrap();
            let mut buffer = ByteBuffer::default();
            assert_eq!(
                unsafe {
                    append_formatted(&mut buffer, |out| {
                        write_cstr(out, text.as_ptr())?;
                        write!(out, ":{}:{}", (-7i32) as i32, (42usize) as usize)
                    })
                },
                (size + 6) as c_int
            );
            assert_eq!(buffer.chunks().len(), 1);
            let length = buffer.remaining();
            let pointer = buffer.pullup(length).unwrap().as_ptr();
            assert_eq!(
                unsafe { std::ffi::CStr::from_ptr(pointer.cast()) }.to_bytes(),
                buffer.chunk()
            );
            assert_eq!(
                buffer.chunk(),
                format!("{}:-7:42", "x".repeat(size)).as_bytes()
            );
            buffer.put_slice(b"\0");
            assert_eq!(buffer.chunks().len(), 1);
            assert_eq!(buffer.chunk().as_ptr(), pointer);
            assert_eq!(buffer.remaining(), length + 1);
            assert_eq!(buffer.chunk()[length], 0);
        }
    }

    #[test]
    fn formatting_preserves_prefix_empty_output_and_embedded_nul() {
        let mut buffer = ByteBuffer::from(b"prefix".to_vec());
        unsafe {
            assert_eq!(
                append_formatted(&mut buffer, |out| { write_cstr(out, c"".as_ptr()) }),
                0
            );
            assert_eq!(buffer.remaining(), 6);
            assert_eq!(
                append_formatted(&mut buffer, |out| {
                    out.write_all(&[(0i32) as u8])?;
                    write!(out, ":{}", (7i32) as i32)
                }),
                3
            );
        }
        assert_eq!(
            buffer.chunks().flatten().copied().collect::<Vec<_>>(),
            b"prefix\0:7"
        );
    }
}

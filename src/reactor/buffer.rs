//! C-call-site compatibility for runtime-owned byte storage.
use crate::src::shared::abi::{size_t, ssize_t};
pub use hmux_buffer::SegmentedBuf as evbuffer;
use hmux_buffer::{Buf, BufMut, Buffer, LineEnding, SegmentedBuf as ByteBuffer};
use std::ffi::{c_char, c_int, c_void, VaList};

pub unsafe fn evbuffer_new() -> *mut ByteBuffer {
    Box::into_raw(Box::new(ByteBuffer::default()))
}
pub unsafe fn evbuffer_free(b: *mut ByteBuffer) {
    if !b.is_null() {
        drop(Box::from_raw(b));
    }
}
pub unsafe fn evbuffer_get_length(b: *const ByteBuffer) -> size_t {
    (*b).remaining() as size_t
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

unsafe fn read_line(b: *mut ByteBuffer, out: *mut size_t, ending: LineEnding) -> *mut c_char {
    if !out.is_null() {
        *out = 0;
    }
    let Some(line) = (*b).read_line(ending) else {
        return std::ptr::null_mut();
    };
    let result = libc::malloc(line.len() + 1).cast::<u8>();
    if result.is_null() {
        std::process::abort();
    }
    std::ptr::copy_nonoverlapping(line.as_ptr(), result, line.len());
    *result.add(line.len()) = 0;
    if !out.is_null() {
        *out = line.len() as size_t;
    }
    super::wake_buffer(b);
    result.cast()
}
pub unsafe fn evbuffer_readln(b: *mut ByteBuffer, out: *mut size_t, style: u32) -> *mut c_char {
    let ending = match style {
        0 => LineEnding::Any,
        1 => LineEnding::CrLf,
        2 => LineEnding::CrLfStrict,
        3 => LineEnding::Lf,
        4 => LineEnding::Nul,
        _ => return std::ptr::null_mut(),
    };
    read_line(b, out, ending)
}
pub unsafe fn evbuffer_readline(b: *mut ByteBuffer) -> *mut c_char {
    read_line(b, std::ptr::null_mut(), LineEnding::Legacy)
}
pub unsafe extern "C" fn evbuffer_add_printf(
    b: *mut ByteBuffer,
    fmt: *const c_char,
    args: ...
) -> c_int {
    evbuffer_add_vprintf(b, fmt, args.clone())
}
pub unsafe extern "C" fn evbuffer_add_vprintf(
    b: *mut ByteBuffer,
    fmt: *const c_char,
    args: VaList,
) -> c_int {
    let mut data = std::ptr::null_mut();
    let count = crate::src::ffi::libc::vasprintf(&mut data, fmt, args);
    if count < 0 {
        return -1;
    }
    evbuffer_add(b, data.cast(), count as size_t);
    libc::free(data.cast());
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn c_formats_and_malloc_lines() {
        unsafe {
            let b = evbuffer_new();
            let long = std::ffi::CString::new("x".repeat(100_000)).unwrap();
            assert_eq!(
                evbuffer_add_printf(b, c"%s:%d:%zu\n".as_ptr(), long.as_ptr(), -7i32, 42usize),
                100_007
            );
            let mut len = 0;
            let line = evbuffer_readln(b, &mut len, 3);
            assert_eq!(len, 100_006);
            assert!(std::ffi::CStr::from_ptr(line)
                .to_bytes()
                .ends_with(b":-7:42"));
            libc::free(line.cast());
            assert_eq!(evbuffer_get_length(b), 0);
            evbuffer_free(b);
        }
    }
}

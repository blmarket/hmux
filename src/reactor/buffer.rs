//! C-call-site compatibility for runtime-owned byte storage.
use crate::src::shared::abi::{size_t, ssize_t};
pub use hmux_rt::ByteBuffer as evbuffer;
use hmux_rt::{ByteBuffer, LineEnding};
use std::ffi::{c_char, c_int, c_void, VaList};

pub unsafe fn evbuffer_new() -> *mut ByteBuffer {
    Box::into_raw(Box::new(ByteBuffer::new()))
}
pub unsafe fn evbuffer_free(b: *mut ByteBuffer) {
    if !b.is_null() {
        drop(Box::from_raw(b));
    }
}
pub unsafe fn evbuffer_get_length(b: *const ByteBuffer) -> size_t {
    (*b).len() as size_t
}
pub unsafe fn evbuffer_add(b: *mut ByteBuffer, data: *const c_void, len: size_t) -> c_int {
    if len != 0 {
        (*b).append(std::slice::from_raw_parts(data.cast(), len));
    }
    super::wake_buffer(b);
    0
}
pub unsafe fn evbuffer_drain(b: *mut ByteBuffer, len: size_t) -> c_int {
    (*b).drain(len);
    super::wake_buffer(b);
    0
}
pub unsafe fn evbuffer_pullup(b: *mut ByteBuffer, size: ssize_t) -> *mut u8 {
    if (*b).is_empty() || (size >= 0 && size as usize > (*b).len()) {
        std::ptr::null_mut()
    } else {
        (*b).as_mut_slice().as_mut_ptr()
    }
}
pub unsafe fn evbuffer_read(b: *mut ByteBuffer, fd: c_int, limit: c_int) -> c_int {
    match (*b).read_from_fd(fd, if limit < 0 { 65536 } else { limit as usize }) {
        Ok(n) => {
            super::wake_buffer(b);
            n as c_int
        }
        Err(e) => {
            *libc::__errno_location() = e.raw_os_error().unwrap_or(libc::EIO);
            -1
        }
    }
}
pub unsafe fn evbuffer_write(b: *mut ByteBuffer, fd: c_int) -> c_int {
    match (*b).write_to_fd(fd) {
        Ok(n) => {
            super::wake_buffer(b);
            n as c_int
        }
        Err(e) => {
            *libc::__errno_location() = e.raw_os_error().unwrap_or(libc::EIO);
            -1
        }
    }
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

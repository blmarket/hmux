//! Arguments for the escaped debug-log format.
//!
//! Escape byte inputs here, before interpolation. The debug sink must not run
//! `strvis` again: doing so would double the backslashes in these arguments.
use crate::src::compat::vis::vis;
use crate::src::shared::vis::{VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB};
use std::ffi::{c_char, c_int, c_void, CStr};
use std::fmt;

/// Display bytes using the logger's existing escaping, stopping at the first NUL.
pub fn log_bytes(bytes: &[u8]) -> impl fmt::Display + '_ {
    LogBytes(bytes)
}

struct LogBytes<'a>(&'a [u8]);

impl fmt::Display for LogBytes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.0.iter().take_while(|&&byte| byte != 0) {
            escaped_byte(byte, f)?;
        }
        Ok(())
    }
}

fn escaped_byte(byte: u8, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut escaped = [0 as c_char; 5];
    // With these flags, lookahead only affects NUL. String arguments stop
    // before NUL, and log_byte handles it separately.
    let end = unsafe {
        vis(
            escaped.as_mut_ptr(),
            byte.into(),
            VIS_CSTYLE | VIS_NL | VIS_OCTAL | VIS_TAB,
            0,
        )
    };
    let len = unsafe { end.offset_from(escaped.as_ptr()) as usize };
    let bytes = unsafe { std::slice::from_raw_parts(escaped.as_ptr().cast(), len) };
    f.write_str(std::str::from_utf8(bytes).expect("vis produces ASCII"))
}

/// A deferred read of a C string, optionally bounded like printf's `%.*s`.
pub struct LogCStr {
    ptr: *const c_char,
    precision: Option<c_int>,
    width: c_int,
}

/// Escape a C string only when the log message is formatted.
///
/// # Safety
/// A non-null pointer must remain a readable NUL-terminated string until the
/// returned argument has been formatted or dropped. Null prints `(null)`.
pub unsafe fn log_cstr(ptr: *const c_char) -> LogCStr {
    LogCStr {
        ptr,
        precision: None,
        width: 0,
    }
}

/// Like `log_cstr`, with printf's byte precision (negative means unlimited).
///
/// # Safety
/// A non-null pointer must be readable up to the first NUL or `precision` bytes,
/// whichever comes first, until formatting completes. Negative precision
/// requires NUL termination.
pub unsafe fn log_cstr_n(ptr: *const c_char, precision: c_int) -> LogCStr {
    LogCStr {
        ptr,
        precision: Some(precision),
        width: 0,
    }
}

/// Like `log_cstr`, padding to a byte width before escaping.
/// Negative widths select left alignment, as with printf's `%*s`.
///
/// # Safety
/// The pointer must satisfy the same requirements as `log_cstr`.
pub unsafe fn log_cstr_width(ptr: *const c_char, width: c_int) -> LogCStr {
    LogCStr {
        ptr,
        precision: None,
        width,
    }
}

impl fmt::Display for LogCStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let precision = self.precision.filter(|&n| n >= 0).map(|n| n as usize);
        let bytes = if self.ptr.is_null() {
            // Match glibc's precision handling of a null %s argument.
            if precision.is_some_and(|n| n < 6) {
                b"".as_slice()
            } else {
                b"(null)"
            }
        } else {
            unsafe {
                let len = match precision {
                    Some(limit) => libc::strnlen(self.ptr, limit),
                    None => CStr::from_ptr(self.ptr).to_bytes().len(),
                };
                std::slice::from_raw_parts(self.ptr.cast(), len)
            }
        };
        let padding = (self.width.unsigned_abs() as usize).saturating_sub(bytes.len());
        if self.width >= 0 {
            for _ in 0..padding {
                f.write_str(" ")?;
            }
        }
        fmt::Display::fmt(&LogBytes(bytes), f)?;
        if self.width < 0 {
            for _ in 0..padding {
                f.write_str(" ")?;
            }
        }
        Ok(())
    }
}

/// Escape one printf-style byte. NUL terminates the assembled log message.
pub fn log_byte(byte: u8) -> impl fmt::Display {
    struct Byte(u8);
    impl fmt::Display for Byte {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.0 == 0 {
                f.write_str("\0")
            } else {
                escaped_byte(self.0, f)
            }
        }
    }
    Byte(byte)
}

/// Preserve libc's `(nil)` spelling for a null pointer.
pub fn log_pointer(ptr: *const c_void) -> impl fmt::Display {
    struct Pointer(*const c_void);
    impl fmt::Display for Pointer {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.0.is_null() {
                f.write_str("(nil)")
            } else {
                write!(f, "{:p}", self.0)
            }
        }
    }
    Pointer(ptr)
}

/// Preserve printf's `%#x`/`%#llx` spelling: zero has no `0x` prefix.
pub fn log_hex(value: u64) -> impl fmt::Display {
    struct Hex(u64);
    impl fmt::Display for Hex {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.0 == 0 {
                f.write_str("0")
            } else {
                write!(f, "{:#x}", self.0)
            }
        }
    }
    Hex(value)
}

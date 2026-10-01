//! Rust formatting into owned bytes and bounded C-character slices
//!
//! ```
//! use hmux::src::format::bytes::format_bytes;
//! use std::ffi::CString;
//! use std::io::Write;
//!
//! let name = CString::new(vec![0xff, b'x']).unwrap();
//! let mut bytes = format_bytes(format_args!("client {}: ", 7));
//! bytes.extend_from_slice(name.as_bytes());
//! write!(&mut bytes, " ({:04x})", 42).unwrap();
//! let value = CString::new(bytes).unwrap();
//! assert_eq!(value.as_bytes(), b"client 7: \xffx (002a)");
//! ```

use std::ffi::{c_char, c_int, CStr, CString, NulError};
use std::fmt;
use std::io::{self, Write};

/// Format into bytes. Embedded NUL bytes are preserved.
pub fn format_bytes(args: fmt::Arguments<'_>) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes
        .write_fmt(args)
        .expect("formatting into a byte vector failed");
    bytes
}

/// Format into a NUL-terminated string, rejecting embedded NUL bytes.
pub fn format_cstring(args: fmt::Arguments<'_>) -> Result<CString, NulError> {
    CString::new(format_bytes(args))
}

/// Assemble raw bytes and typed formatting, preserving embedded NUL bytes.
pub fn format_bytes_with(
    write: impl FnOnce(&mut dyn Write) -> io::Result<()>,
) -> io::Result<Vec<u8>> {
    struct Writer(Vec<u8>);
    impl Write for Writer {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0
                .try_reserve(bytes.len())
                .map_err(|_| io::ErrorKind::OutOfMemory)?;
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
        fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> io::Result<()> {
            fmt::write(self, args).map_err(|_| io::ErrorKind::Other.into())
        }
    }
    impl fmt::Write for Writer {
        fn write_str(&mut self, text: &str) -> fmt::Result {
            self.write_all(text.as_bytes()).map_err(|_| fmt::Error)
        }
    }
    let mut out = Writer(Vec::new());
    write(&mut out)?;
    Ok(out.0)
}

/// Assemble a C-string message; as with C-string consumers, the first NUL ends it.
pub fn try_format_message_with(
    write: impl FnOnce(&mut dyn Write) -> io::Result<()>,
) -> Option<CString> {
    let mut bytes = format_bytes_with(write).ok()?;
    if let Some(end) = bytes.iter().position(|&byte| byte == 0) {
        bytes.truncate(end);
    }
    Some(CString::new(bytes).expect("message was truncated before its first NUL"))
}

pub fn format_message_with(write: impl FnOnce(&mut dyn Write) -> io::Result<()>) -> CString {
    try_format_message_with(write).unwrap_or_else(|| unsafe {
        crate::src::log::fatalx(|out| out.write_all(b"message formatting failed"))
    })
}

/// Write an unescaped C string. A null pointer retains the legacy `(null)` text.
///
/// # Safety
/// A non-null pointer must reference a readable NUL-terminated string.
pub unsafe fn write_cstr(out: &mut dyn Write, value: *const c_char) -> io::Result<()> {
    write_cstr_n(out, value, -1)
}

/// Write at most `precision` bytes, stopping at NUL; negative means unlimited.
///
/// # Safety
/// A non-null pointer must be readable through its NUL or the precision limit.
pub unsafe fn write_cstr_n(
    out: &mut dyn Write,
    value: *const c_char,
    precision: c_int,
) -> io::Result<()> {
    let bytes = if value.is_null() {
        if (0..6).contains(&precision) {
            b"".as_slice()
        } else {
            b"(null)".as_slice()
        }
    } else if precision < 0 {
        CStr::from_ptr(value).to_bytes()
    } else {
        let len = libc::strnlen(value, precision as usize);
        std::slice::from_raw_parts(value.cast(), len)
    };
    out.write_all(bytes)
}

/// Format into an existing C-character buffer without allocating.
///
/// The slice includes room for the trailing NUL. Returns the payload byte count,
/// excluding that terminator. Embedded NUL bytes are preserved and counted.
/// Empty destinations, insufficient capacity, and formatter failures return an
/// error. On failure, a nonempty destination contains a NUL-terminated prefix;
/// bytes beyond the terminator are unspecified. Limits count bytes, not columns.
pub fn format_cstr_into(dst: &mut [c_char], args: fmt::Arguments<'_>) -> Result<usize, fmt::Error> {
    format_cstr_with(dst, |out| out.write_fmt(args)).map_err(|_| fmt::Error)
}

struct CStrWriter<'a> {
    dst: &'a mut [c_char],
    written: usize,
}

impl Write for CStrWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.dst.len() - 1 - self.written {
            return Err(io::ErrorKind::WriteZero.into());
        }
        let end = self.written + bytes.len();
        for (slot, &byte) in self.dst[self.written..end].iter_mut().zip(bytes) {
            *slot = byte as c_char;
        }
        self.written = end;
        self.dst[end] = 0;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> io::Result<()> {
        fmt::write(self, args).map_err(|_| io::ErrorKind::Other.into())
    }
}

impl fmt::Write for CStrWriter<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.write_all(text.as_bytes()).map_err(|_| fmt::Error)
    }
}

/// Write raw bytes and formatted text into a bounded C-character slice.
///
/// This has the same length and NUL-termination contract as `format_cstr_into`.
/// Use `write_all` for arbitrary C-string bytes and single-byte characters, and
/// `write!` for Rust formatting. Propagate write errors from the closure.
pub fn format_cstr_with(
    dst: &mut [c_char],
    write: impl FnOnce(&mut dyn Write) -> io::Result<()>,
) -> io::Result<usize> {
    *dst.first_mut().ok_or(io::ErrorKind::WriteZero)? = 0;
    let mut writer = CStrWriter { dst, written: 0 };
    write(&mut writer)?;
    Ok(writer.written)
}

/// Format into a bounded C-character slice, exiting on overflow or format errors.
pub fn xformat(dst: &mut [c_char], args: fmt::Arguments<'_>) -> c_int {
    xformat_with(dst, |out| out.write_fmt(args))
}

/// Write bytes and formatted text, exiting on overflow or write errors.
/// Returns the payload byte count, excluding the trailing NUL.
pub fn xformat_with(
    dst: &mut [c_char],
    write: impl FnOnce(&mut dyn Write) -> io::Result<()>,
) -> c_int {
    if dst.len() > c_int::MAX as usize {
        unsafe { crate::src::log::fatalx(|out| out.write_all(b"xformat: len > INT_MAX")) };
    }
    match format_cstr_with(dst, write) {
        Ok(written) => written as c_int,
        Err(_) => unsafe {
            crate::src::log::fatalx(|out| out.write_all(b"xformat: formatting failed or overflow"))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_output_preserves_nul_while_cstring_reports_it() {
        assert_eq!(format_bytes(format_args!("a\0{}", 7)), b"a\x007");
        let error = format_cstring(format_args!("a\0{}", 7)).unwrap_err();
        assert_eq!(error.nul_position(), 1);
        assert_eq!(error.into_vec(), b"a\x007");
    }

    #[test]
    fn bounded_format_reserves_terminator_and_reuses_storage() {
        let mut dst = [42 as c_char; 32];
        let text = "x".repeat(31);
        assert_eq!(format_cstr_into(&mut dst, format_args!("{text}")), Ok(31));
        assert_eq!(dst[31], 0);
        assert_eq!(format_cstr_into(&mut dst, format_args!("{}", 12345)), Ok(5));
        assert_eq!(
            unsafe { std::ffi::CStr::from_ptr(dst.as_ptr()) }.to_bytes(),
            b"12345"
        );
        assert_eq!(
            format_cstr_into(&mut dst, format_args!("{}", "x".repeat(32))),
            Err(fmt::Error)
        );
        assert!(dst.contains(&0));
    }

    #[test]
    fn bounded_format_handles_empty_and_offset_slices() {
        assert_eq!(format_cstr_into(&mut [], format_args!("")), Err(fmt::Error));
        let mut empty = [42 as c_char];
        assert_eq!(format_cstr_into(&mut empty, format_args!("")), Ok(0));
        assert_eq!(empty, [0]);
        assert_eq!(
            format_cstr_into(&mut empty, format_args!("x")),
            Err(fmt::Error)
        );
        let mut dst = [42 as c_char; 8];
        let n = format_cstr_into(&mut dst[..4], format_args!("abc")).unwrap();
        assert_eq!(dst[4], 42);
        let m = format_cstr_into(&mut dst[n..], format_args!("{}", 1234)).unwrap();
        assert_eq!(n + m, 7);
        assert_eq!(
            unsafe { std::ffi::CStr::from_ptr(dst.as_ptr()) }.to_bytes(),
            b"abc1234"
        );
    }

    #[test]
    fn bounded_format_counts_utf8_and_embedded_nul_bytes() {
        let mut dst = [0 as c_char; 6];
        assert_eq!(format_cstr_into(&mut dst, format_args!("é\0{}", 42)), Ok(5));
        assert_eq!(dst.map(|c| c as u8), *b"\xc3\xa9\x0042\0");
        assert_eq!(
            format_cstr_into(&mut dst[..2], format_args!("é")),
            Err(fmt::Error)
        );
    }

    #[test]
    fn bounded_format_propagates_formatter_errors() {
        struct Broken;
        impl fmt::Display for Broken {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("partial")?;
                Err(fmt::Error)
            }
        }
        let mut dst = [42 as c_char; 16];
        assert_eq!(
            format_cstr_into(&mut dst, format_args!("{Broken}")),
            Err(fmt::Error)
        );
        assert_eq!(
            unsafe { std::ffi::CStr::from_ptr(dst.as_ptr()) }.to_bytes(),
            b"partial"
        );
    }

    #[test]
    fn mixed_output_matches_c_bytes_characters_and_widths() {
        let value = CString::new(b"name\xff".as_slice()).unwrap();
        for byte in [0u8, b'M', 0x80, 0xff] {
            let mut expected = [0 as c_char; 64];
            let count = unsafe {
                crate::src::ffi::libc::snprintf(
                    expected.as_mut_ptr(),
                    expected.len(),
                    c"%4u: %s:%c:%u".as_ptr(),
                    7u32,
                    value.as_ptr(),
                    byte as c_int,
                    u32::MAX,
                )
            };
            let mut actual = [42 as c_char; 64];
            let written = xformat_with(&mut actual, |out| {
                write!(out, "{:4}: ", 7u32)?;
                out.write_all(value.as_bytes())?;
                out.write_all(&[b':', byte, b':'])?;
                write!(out, "{}", u32::MAX)
            });
            assert_eq!(written, count);
            assert_eq!(&actual[..=count as usize], &expected[..=count as usize]);
        }
    }

    #[test]
    fn mixed_output_checks_capacity_and_propagates_write_errors() {
        let mut dst = [42 as c_char; 6];
        let n = format_cstr_with(&mut dst[..3], |out| out.write_all(b"\xffx")).unwrap();
        assert_eq!(n, 2);
        assert_eq!(dst[2], 0);
        assert_eq!(dst[3], 42);
        let n = format_cstr_with(&mut dst[n..], |out| {
            out.write_all(&[0])?;
            write!(out, "{}", 12)
        })
        .unwrap();
        assert_eq!(n, 3);
        assert_eq!(dst.map(|c| c as u8), *b"\xffx\x0012\0");

        let error = format_cstr_with(&mut dst[..3], |out| {
            out.write_all(b"a")?;
            out.write_all(b"bc")
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::WriteZero);
        assert_eq!(&dst[..2], &[b'a' as c_char, 0]);
        let error = format_cstr_with(&mut dst, |out| {
            out.write_all(b"ok")?;
            Err(io::ErrorKind::InvalidData.into())
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(dst[2], 0);
    }

    #[test]
    fn owned_messages_and_byte_buffers_keep_their_distinct_nul_contracts() {
        let bytes = format_bytes_with(|out| out.write_all(b"\xffbefore\0after")).unwrap();
        assert_eq!(bytes, b"\xffbefore\0after");
        let message = format_message_with(|out| out.write_all(&bytes));
        assert_eq!(message.as_bytes(), b"\xffbefore");
        assert!(try_format_message_with(|_| Err(io::ErrorKind::InvalidData.into())).is_none());
    }

    #[test]
    fn raw_c_strings_match_libc_for_nulls_and_byte_precision() {
        unsafe {
            let raw = [0xffu8, b'x', b'y', 0];
            for value in [raw.as_ptr().cast::<c_char>(), std::ptr::null()] {
                for precision in [-1, 0, 1, 2, 3, 5, 6, 8] {
                    let actual =
                        format_bytes_with(|out| write_cstr_n(out, value, precision)).unwrap();
                    let mut expected = [0 as c_char; 32];
                    let count = crate::src::ffi::libc::snprintf(
                        expected.as_mut_ptr(),
                        expected.len(),
                        c"%.*s".as_ptr(),
                        precision,
                        value,
                    );
                    assert!(count >= 0);
                    assert_eq!(actual, CStr::from_ptr(expected.as_ptr()).to_bytes());
                }
            }
            // This buffer has no NUL: the precision must bound the read.
            let raw = [0xffu8, b'x'];
            let actual =
                format_bytes_with(|out| write_cstr_n(out, raw.as_ptr().cast(), 2)).unwrap();
            assert_eq!(actual, raw);
        }
    }
}

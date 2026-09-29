use crate::src::compat::stdio::CFile;
use crate::src::compat::vis::strvis;
use crate::src::ffi::libc::{
    __errno_location, exit, fflush, fopen, fprintf, getpid, gettimeofday, setvbuf, snprintf,
    strerror,
};
use crate::src::format::bytes::format_bytes;
use crate::src::format::bytes::try_format_message_with;
use crate::src::shared::abi::*;
use crate::src::shared::stdio::FILE;
use crate::src::shared::vis::{VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB};
use std::ffi::{CStr, CString};
use std::fmt;

mod format;
pub use format::{log_byte, log_bytes, log_cstr, log_cstr_n, log_cstr_width, log_hex, log_pointer};

pub const _IOLBF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

static mut log_file: Option<CFile> = None;
static mut log_level: ::core::ffi::c_int = 0;

unsafe fn log_file_ptr() -> *mut FILE {
    unsafe {
        (*(&raw const log_file))
            .as_ref()
            .map_or(::core::ptr::null_mut::<FILE>(), CFile::as_ptr)
    }
}
pub unsafe fn log_add_level() {
    unsafe {
        log_level += 1;
    }
}
pub unsafe fn log_get_level() -> ::core::ffi::c_int {
    unsafe {
        return log_level;
    }
}
pub unsafe fn log_open(mut name: *const ::core::ffi::c_char) {
    unsafe {
        if log_level == 0 as ::core::ffi::c_int {
            return;
        }
        log_close();
        let pid = (getpid() as ::core::ffi::c_long).to_string();
        let mut path = b"tmux-".to_vec();
        path.extend_from_slice(CStr::from_ptr(name).to_bytes());
        path.push(b'-');
        path.extend_from_slice(pid.as_bytes());
        path.extend_from_slice(b".log");
        let path = CString::new(path).expect("log filename components contain no interior NUL");
        let file = fopen(
            path.as_ptr(),
            b"a\0" as *const u8 as *const ::core::ffi::c_char,
        ) as *mut FILE;
        let Some(file) = CFile::from_raw(file) else {
            return;
        };
        log_file = Some(file);
        setvbuf(
            log_file_ptr(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            _IOLBF,
            0 as size_t,
        );
    }
}
pub unsafe fn log_toggle(mut name: *const ::core::ffi::c_char) {
    unsafe {
        if log_level == 0 as ::core::ffi::c_int {
            log_level = 1 as ::core::ffi::c_int;
            log_open(name);
            log_debug(format_args!("log opened"));
        } else {
            log_debug(format_args!("log closed"));
            log_level = 0 as ::core::ffi::c_int;
            log_close();
        };
    }
}
pub unsafe fn log_close() {
    unsafe {
        log_file = None;
    }
}
unsafe fn log_message(
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
    mut prefix: *const ::core::ffi::c_char,
) {
    unsafe {
        let file = log_file_ptr();
        if file.is_null() {
            return;
        }
        let Some(s) = try_format_message_with(write) else {
            return;
        };
        // strvis writes at most four bytes per input byte plus the terminator.
        let Some(capacity) = s
            .as_bytes()
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_mul(4))
        else {
            return;
        };
        let mut out = Vec::<u8>::new();
        if out.try_reserve_exact(capacity).is_err() {
            return;
        }
        out.resize(capacity, 0);
        strvis(
            out.as_mut_ptr().cast(),
            s.as_ptr(),
            VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL,
        );
        drop(s);
        log_write_escaped(CStr::from_ptr(out.as_ptr().cast()), CStr::from_ptr(prefix));
    }
}

unsafe fn log_write_escaped(message: &CStr, prefix: &CStr) {
    unsafe {
        let file = log_file_ptr();
        if file.is_null() {
            return;
        }
        let mut tv: timeval = timeval {
            tv_sec: 0,
            tv_usec: 0,
        };
        gettimeofday(&raw mut tv, NULL);
        if fprintf(
            file,
            b"%lld.%06d %s%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            tv.tv_sec as ::core::ffi::c_longlong,
            tv.tv_usec as ::core::ffi::c_int,
            prefix.as_ptr(),
            message.as_ptr(),
        ) != -(1 as ::core::ffi::c_int)
        {
            fflush(file);
        }
    }
}
/// Write a debug message, formatting only while logging is enabled.
///
/// Literal text and numeric arguments are written as supplied. Use `log_bytes`
/// or `log_cstr` for data that needs the logger's byte escaping (including
/// backslashes, newlines and non-UTF-8 bytes). Arguments are escaped once,
/// before interpolation. As before, an embedded NUL ends the message.
pub unsafe fn log_debug(args: fmt::Arguments<'_>) {
    unsafe {
        if log_file_ptr().is_null() {
            return;
        }
        let mut message = format_bytes(args);
        message.push(0);
        log_write_escaped(CStr::from_bytes_until_nul(&message).unwrap(), c"");
    }
}
pub unsafe fn fatal(write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>) -> ! {
    unsafe {
        let mut tmp: [::core::ffi::c_char; 256] = [0; 256];
        if snprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
            b"fatal: %s: \0" as *const u8 as *const ::core::ffi::c_char,
            strerror(*__errno_location()),
        ) < 0 as ::core::ffi::c_int
        {
            exit(1 as ::core::ffi::c_int);
        }
        log_message(write, &raw mut tmp as *mut ::core::ffi::c_char);
        exit(1 as ::core::ffi::c_int);
    }
}
pub unsafe fn fatalx(write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>) -> ! {
    unsafe {
        log_message(
            write,
            b"fatal: \0" as *const u8 as *const ::core::ffi::c_char,
        );
        exit(1 as ::core::ffi::c_int);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Seek, SeekFrom};
    use std::os::fd::FromRawFd;

    // Keep the comparison against libc without defining a variadic Rust function.
    macro_rules! legacy_message {
        ($format:expr_2021 $(, $arg:expr_2021)* $(,)?) => {{
            let count = snprintf(std::ptr::null_mut(), 0, $format $(, $arg)*);
            assert!(count >= 0);
            let mut raw = vec![0u8; count as usize + 1];
            assert_eq!(snprintf(raw.as_mut_ptr().cast(), raw.len(), $format $(, $arg)*), count);
            let raw = CStr::from_ptr(raw.as_ptr().cast());
            let mut escaped = vec![0u8; raw.to_bytes().len() * 4 + 1];
            strvis(escaped.as_mut_ptr().cast(), raw.as_ptr(), VIS_CSTYLE | VIS_NL | VIS_OCTAL | VIS_TAB);
            escaped.truncate(escaped.iter().position(|&byte| byte == 0).unwrap());
            escaped
        }};
    }

    fn message(args: fmt::Arguments<'_>) -> Vec<u8> {
        let mut bytes = format_bytes(args);
        if let Some(end) = bytes.iter().position(|&byte| byte == 0) {
            bytes.truncate(end);
        }
        bytes
    }

    #[test]
    fn debug_arguments_match_legacy_escaping_for_every_byte() {
        unsafe {
            for byte in 0..=255u8 {
                let bytes = [byte, b'7', b'\\', b'\n', 0];
                assert_eq!(
                    message(format_args!(
                        "before {} after",
                        log_cstr(bytes.as_ptr().cast())
                    )),
                    legacy_message!(c"before %s after".as_ptr(), bytes.as_ptr()),
                    "C string byte {byte}"
                );
                assert_eq!(
                    message(format_args!("before {}7 after", log_byte(byte))),
                    legacy_message!(c"before %c7 after".as_ptr(), byte as i32),
                    "character byte {byte}"
                );
                assert_eq!(
                    message(format_args!("before {} after", log_bytes(&bytes))),
                    legacy_message!(c"before %s after".as_ptr(), bytes.as_ptr()),
                    "slice byte {byte}"
                );
            }
        }
    }

    #[test]
    fn bounded_strings_and_padding_match_printf_before_escaping() {
        unsafe {
            // No NUL terminator: precision must bound the actual memory read.
            let bytes = [0xffu8, b'\\', b'\n', 0xc3];
            for precision in 0..=bytes.len() as i32 {
                assert_eq!(
                    message(format_args!(
                        "[{}]",
                        log_cstr_n(bytes.as_ptr().cast(), precision)
                    )),
                    legacy_message!(c"[%.*s]".as_ptr(), precision, bytes.as_ptr())
                );
            }
            let bytes = c"\xff\\\n";
            for precision in [-1, 0, 1, 2, 3, 20] {
                assert_eq!(
                    message(format_args!("[{}]", log_cstr_n(bytes.as_ptr(), precision))),
                    legacy_message!(c"[%.*s]".as_ptr(), precision, bytes.as_ptr())
                );
            }
            for width in [-8, -1, 0, 1, 8] {
                assert_eq!(
                    message(format_args!("[{}]", log_cstr_width(bytes.as_ptr(), width))),
                    legacy_message!(c"[%*s]".as_ptr(), width, bytes.as_ptr())
                );
            }
            for precision in [-1, 0, 1, 5, 6, 8] {
                assert_eq!(
                    message(format_args!(
                        "[{}]",
                        log_cstr_n(std::ptr::null(), precision)
                    )),
                    legacy_message!(
                        c"[%.*s]".as_ptr(),
                        precision,
                        std::ptr::null::<::core::ffi::c_char>()
                    )
                );
            }
        }
    }

    #[test]
    fn pointer_and_hex_spelling_match_printf() {
        unsafe {
            let byte = 0u8;
            for ptr in [
                std::ptr::null(),
                (&raw const byte).cast::<::core::ffi::c_void>(),
            ] {
                assert_eq!(
                    message(format_args!("{}", log_pointer(ptr))),
                    legacy_message!(c"%p".as_ptr(), ptr)
                );
            }
            for value in [0u64, 1, 0xff, u64::MAX] {
                assert_eq!(
                    message(format_args!("{}", log_hex(value))),
                    legacy_message!(c"%#llx".as_ptr(), value)
                );
            }
        }
    }

    struct RestoreLog(Option<CFile>);
    impl Drop for RestoreLog {
        fn drop(&mut self) {
            unsafe {
                log_file = self.0.take();
            }
        }
    }

    #[test]
    fn disabled_debug_does_not_format_arguments() {
        struct MustNotFormat;
        impl fmt::Display for MustNotFormat {
            fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
                panic!("disabled logger formatted an argument")
            }
        }
        unsafe {
            let _restore = RestoreLog((&raw mut log_file).replace(None));
            log_debug(format_args!("{}", MustNotFormat));
            log_message(
                |_| panic!("disabled logger invoked the message writer"),
                c"fatal: ".as_ptr(),
            );
        }
    }

    #[test]
    fn debug_sink_writes_one_escaped_line_and_preserves_nul_truncation() {
        unsafe {
            let file = libc::tmpfile();
            assert!(!file.is_null());
            let fd = libc::dup(libc::fileno(file));
            assert!(fd >= 0);
            let mut reader = std::fs::File::from_raw_fd(fd);
            let _restore = RestoreLog((&raw mut log_file).replace(CFile::from_raw(file.cast())));
            log_debug(format_args!("packet: {}", log_bytes(b"\x80\n\\")));
            log_debug(format_args!("stop{}ignored", log_byte(0)));
            reader.seek(SeekFrom::Start(0)).unwrap();
            let mut output = Vec::new();
            reader.read_to_end(&mut output).unwrap();
            let lines: Vec<_> = output.split(|&byte| byte == b'\n').collect();
            assert_eq!(lines.len(), 3);
            assert!(lines[0].ends_with(b" packet: \\200\\n\\\\"));
            assert!(lines[1].ends_with(b" stop"));
            assert!(lines[2].is_empty());
        }
    }
}

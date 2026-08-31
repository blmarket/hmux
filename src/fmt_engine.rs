//! A printf(3) engine that reads its arguments from an explicit slice.
//!
//! The tree still carries the C format strings tmux was written with, and its
//! output — status lines, control-mode messages, log lines — is a byte-level
//! compatibility surface. This module keeps the format strings and replaces
//! only the variadic argument list: a caller passes [`FmtArg`] values in a
//! slice, so no Rust function needs a C calling convention to accept them.
//!
//! Everything except the floating-point conversions is formatted here.
//! `e E f F g G a A` are handed to libc `snprintf` one conversion at a time,
//! with the specifier rebuilt from what was parsed: the four `%.*f` sites in
//! the tree print user-visible arithmetic, and glibc's rounding is the
//! behaviour they have today.
//!
//! Arguments are consumed strictly left to right as the format reaches them,
//! so `%.*s` takes the precision first and the pointer second, exactly as the
//! C call passed them. The value's width comes from the argument, not from the
//! default promotions, but a length modifier still truncates — `%hhx` of an
//! `int` prints one byte, as it does in C.

use ::core::ffi::{c_char, c_int};
use ::std::ffi::CString;

use crate::ffi::snprintf;
use crate::reactor::Buf;

/// One argument to a conversion.
///
/// The variants are the four shapes a C vararg reaches printf in, not the
/// Rust types the call site holds: an integer conversion accepts any of the
/// integral variants and truncates per its length modifier, and `%s` and `%p`
/// both read [`FmtArg::Ptr`].
#[derive(Clone, Copy, Debug)]
pub enum FmtArg {
    Int(i64),
    UInt(u64),
    Ptr(*const u8),
    Flt(f64),
}

macro_rules! fmt_arg_from_signed {
    ($($t:ty),*) => {$(
        impl From<$t> for FmtArg {
            fn from(v: $t) -> Self {
                FmtArg::Int(v as i64)
            }
        }
    )*};
}

macro_rules! fmt_arg_from_unsigned {
    ($($t:ty),*) => {$(
        impl From<$t> for FmtArg {
            fn from(v: $t) -> Self {
                FmtArg::UInt(v as u64)
            }
        }
    )*};
}

macro_rules! fmt_arg_from_float {
    ($($t:ty),*) => {$(
        impl From<$t> for FmtArg {
            fn from(v: $t) -> Self {
                FmtArg::Flt(v as f64)
            }
        }
    )*};
}

fmt_arg_from_signed!(i8, i16, i32, i64, isize);
fmt_arg_from_unsigned!(u8, u16, u32, u64, usize);
fmt_arg_from_float!(f32, f64);

impl<T> From<*const T> for FmtArg {
    fn from(v: *const T) -> Self {
        FmtArg::Ptr(v as *const u8)
    }
}

impl<T> From<*mut T> for FmtArg {
    fn from(v: *mut T) -> Self {
        FmtArg::Ptr(v as *const u8)
    }
}

/// A borrowed C string prints as the `%s` it already is.
impl From<&::core::ffi::CStr> for FmtArg {
    fn from(v: &::core::ffi::CStr) -> Self {
        FmtArg::Ptr(v.as_ptr() as *const u8)
    }
}

/// A missing C string prints as the null `%s` the C call would have passed.
impl From<Option<&::core::ffi::CStr>> for FmtArg {
    fn from(v: Option<&::core::ffi::CStr>) -> Self {
        FmtArg::Ptr(v.map_or(::core::ptr::null(), |s| s.as_ptr() as *const u8))
    }
}

/// Builds the argument slice a format-taking function wants.
///
/// ```ignore
/// cmdq_print(item, c"%s: %u".as_ptr(), fmt_args![name, count]);
/// ```
#[macro_export]
macro_rules! fmt_args {
    () => {
        &[] as &[$crate::fmt_engine::FmtArg]
    };
    ($($arg:expr),+ $(,)?) => {
        &[$($crate::fmt_engine::FmtArg::from($arg)),+] as &[$crate::fmt_engine::FmtArg]
    };
}

#[derive(Clone, Copy, PartialEq, Default)]
enum Len {
    #[default]
    Default,
    Char,
    Short,
    Long,
    LongLong,
    Size,
    IntMax,
    PtrDiff,
    LongDouble,
}

#[derive(Default)]
struct Spec {
    minus: bool,
    plus: bool,
    space: bool,
    hash: bool,
    zero: bool,
    width: usize,
    prec: Option<usize>,
    len: Len,
    conv: u8,
}

/// Reads the next argument, or `None` past the end of the slice.
///
/// Running out is a call-site bug, not something a format string can do, so
/// tests trip an assertion on it while a release build formats what it can.
fn next(args: &[FmtArg], at: &mut usize) -> Option<FmtArg> {
    let arg = args.get(*at).copied();
    debug_assert!(
        arg.is_some(),
        "printf format wants more arguments than given"
    );
    *at += 1;
    arg
}

fn next_signed(args: &[FmtArg], at: &mut usize) -> i64 {
    match next(args, at) {
        Some(FmtArg::Int(v)) => v,
        Some(FmtArg::UInt(v)) => v as i64,
        Some(FmtArg::Ptr(v)) => v as i64,
        Some(FmtArg::Flt(v)) => v as i64,
        None => 0,
    }
}

fn next_unsigned(args: &[FmtArg], at: &mut usize) -> u64 {
    match next(args, at) {
        Some(FmtArg::Int(v)) => v as u64,
        Some(FmtArg::UInt(v)) => v,
        Some(FmtArg::Ptr(v)) => v as u64,
        Some(FmtArg::Flt(v)) => v as u64,
        None => 0,
    }
}

fn next_pointer(args: &[FmtArg], at: &mut usize) -> *const u8 {
    match next(args, at) {
        Some(FmtArg::Ptr(v)) => v,
        Some(FmtArg::Int(v)) => v as *const u8,
        Some(FmtArg::UInt(v)) => v as *const u8,
        Some(FmtArg::Flt(_)) | None => ::core::ptr::null(),
    }
}

fn next_float(args: &[FmtArg], at: &mut usize) -> f64 {
    match next(args, at) {
        Some(FmtArg::Flt(v)) => v,
        Some(FmtArg::Int(v)) => v as f64,
        Some(FmtArg::UInt(v)) => v as f64,
        Some(FmtArg::Ptr(_)) | None => 0.0,
    }
}

fn truncate_signed(v: i64, len: Len) -> i64 {
    match len {
        Len::Char => v as i8 as i64,
        Len::Short => v as i16 as i64,
        Len::Default => v as i32 as i64,
        _ => v,
    }
}

fn truncate_unsigned(v: u64, len: Len) -> u64 {
    match len {
        Len::Char => v as u8 as u64,
        Len::Short => v as u16 as u64,
        Len::Default => v as u32 as u64,
        _ => v,
    }
}

fn digits(mut v: u64, base: u64, upper: bool) -> Vec<u8> {
    if v == 0 {
        return vec![b'0'];
    }
    let table: &[u8] = if upper {
        b"0123456789ABCDEF"
    } else {
        b"0123456789abcdef"
    };
    let mut out = Vec::new();
    while v != 0 {
        out.push(table[(v % base) as usize]);
        v /= base;
    }
    out.reverse();
    out
}

/// Lays out sign, prefix, precision zeros and field padding around `body`.
fn emit_number(out: &mut Vec<u8>, spec: &Spec, sign: &[u8], prefix: &[u8], body: &[u8]) {
    let zeros = spec.prec.map_or(0, |p| p.saturating_sub(body.len()));
    let carried = sign.len() + prefix.len() + zeros + body.len();
    let pad = spec.width.saturating_sub(carried);
    let (zero_pad, space_pad) = if spec.minus {
        (0, pad)
    } else if spec.zero && spec.prec.is_none() {
        (pad, 0)
    } else {
        (0, pad)
    };

    if !spec.minus {
        out.extend(::core::iter::repeat_n(b' ', space_pad));
    }
    out.extend_from_slice(sign);
    out.extend_from_slice(prefix);
    out.extend(::core::iter::repeat_n(b'0', zero_pad + zeros));
    out.extend_from_slice(body);
    if spec.minus {
        out.extend(::core::iter::repeat_n(b' ', space_pad));
    }
}

fn emit_string(out: &mut Vec<u8>, spec: &Spec, body: &[u8]) {
    let pad = spec.width.saturating_sub(body.len());
    if !spec.minus {
        out.extend(::core::iter::repeat_n(b' ', pad));
    }
    out.extend_from_slice(body);
    if spec.minus {
        out.extend(::core::iter::repeat_n(b' ', pad));
    }
}

fn emit_signed(out: &mut Vec<u8>, spec: &Spec, v: i64) {
    let v = truncate_signed(v, spec.len);
    let sign: &[u8] = if v < 0 {
        b"-"
    } else if spec.plus {
        b"+"
    } else if spec.space {
        b" "
    } else {
        b""
    };
    let body = if v == 0 && spec.prec == Some(0) {
        Vec::new()
    } else {
        digits(v.unsigned_abs(), 10, false)
    };
    emit_number(out, spec, sign, b"", &body);
}

fn emit_unsigned(out: &mut Vec<u8>, spec: &Spec, v: u64) {
    let v = truncate_unsigned(v, spec.len);
    let (base, upper) = match spec.conv {
        b'o' => (8, false),
        b'x' => (16, false),
        b'X' => (16, true),
        _ => (10, false),
    };
    let body = if v == 0 && spec.prec == Some(0) {
        Vec::new()
    } else {
        digits(v, base, upper)
    };
    let zeros = spec.prec.map_or(0, |p| p.saturating_sub(body.len()));
    let prefix: &[u8] = if !spec.hash {
        b""
    } else if base == 8 && zeros == 0 && body.first() != Some(&b'0') {
        b"0"
    } else if base == 16 && v != 0 {
        if upper { b"0X" } else { b"0x" }
    } else {
        b""
    };
    emit_number(out, spec, b"", prefix, &body);
}

/// The bytes `p` points at, stopping at a NUL or at `prec` bytes, whichever
/// comes first. A null pointer reads as glibc's `(null)`, which glibc drops
/// altogether rather than truncating when the precision cannot hold it.
unsafe fn read_string(p: *const u8, prec: Option<usize>) -> Vec<u8> {
    if p.is_null() {
        let null: &[u8] = b"(null)";
        return if prec.is_none_or(|n| n >= null.len()) {
            null.to_vec()
        } else {
            Vec::new()
        };
    }
    let mut out = Vec::new();
    let mut i = 0usize;
    unsafe {
        while prec.is_none_or(|n| i < n) {
            let c = *p.add(i);
            if c == 0 {
                break;
            }
            out.push(c);
            i += 1;
        }
    }
    out
}

/// Formats one floating-point conversion through libc, rebuilding the
/// specifier with its `*` width and precision already resolved.
fn emit_float(out: &mut Vec<u8>, spec: &Spec, v: f64) {
    let mut c_spec: Vec<u8> = vec![b'%'];
    for (flag, byte) in [
        (spec.minus, b'-'),
        (spec.plus, b'+'),
        (spec.space, b' '),
        (spec.hash, b'#'),
        (spec.zero, b'0'),
    ] {
        if flag {
            c_spec.push(byte);
        }
    }
    if spec.width != 0 {
        c_spec.extend_from_slice(spec.width.to_string().as_bytes());
    }
    if let Some(p) = spec.prec {
        c_spec.push(b'.');
        c_spec.extend_from_slice(p.to_string().as_bytes());
    }
    c_spec.push(spec.conv);
    c_spec.push(0);

    unsafe {
        let fmt = c_spec.as_ptr() as *const c_char;
        let n = snprintf(::core::ptr::null_mut(), 0, fmt, v);
        if n <= 0 {
            return;
        }
        let mut buf = vec![0u8; n as usize + 1];
        snprintf(buf.as_mut_ptr() as *mut c_char, n as usize + 1, fmt, v);
        out.extend_from_slice(&buf[..n as usize]);
    }
}

/// Appends `fmt` expanded over `args` to `out`.
pub unsafe fn format_append(out: &mut Vec<u8>, fmt: *const c_char, args: &[FmtArg]) {
    let mut p = fmt as *const u8;
    let mut at = 0usize;

    unsafe {
        loop {
            let c = *p;
            if c == 0 {
                break;
            }
            if c != b'%' {
                out.push(c);
                p = p.add(1);
                continue;
            }

            let start = p;
            p = p.add(1);
            let mut spec = Spec::default();

            loop {
                match *p {
                    b'-' => spec.minus = true,
                    b'+' => spec.plus = true,
                    b' ' => spec.space = true,
                    b'#' => spec.hash = true,
                    b'0' => spec.zero = true,
                    _ => break,
                }
                p = p.add(1);
            }

            if *p == b'*' {
                p = p.add(1);
                let w = next_signed(args, &mut at) as i32;
                if w < 0 {
                    spec.minus = true;
                    spec.width = w.unsigned_abs() as usize;
                } else {
                    spec.width = w as usize;
                }
            } else {
                while (*p).is_ascii_digit() {
                    spec.width = spec.width * 10 + (*p - b'0') as usize;
                    p = p.add(1);
                }
            }

            if *p == b'.' {
                p = p.add(1);
                if *p == b'*' {
                    p = p.add(1);
                    let n = next_signed(args, &mut at) as i32;
                    spec.prec = if n < 0 { None } else { Some(n as usize) };
                } else {
                    let mut n = 0usize;
                    while (*p).is_ascii_digit() {
                        n = n * 10 + (*p - b'0') as usize;
                        p = p.add(1);
                    }
                    spec.prec = Some(n);
                }
            }

            spec.len = match *p {
                b'h' => {
                    p = p.add(1);
                    if *p == b'h' {
                        p = p.add(1);
                        Len::Char
                    } else {
                        Len::Short
                    }
                }
                b'l' => {
                    p = p.add(1);
                    if *p == b'l' {
                        p = p.add(1);
                        Len::LongLong
                    } else {
                        Len::Long
                    }
                }
                b'z' => {
                    p = p.add(1);
                    Len::Size
                }
                b'j' => {
                    p = p.add(1);
                    Len::IntMax
                }
                b't' => {
                    p = p.add(1);
                    Len::PtrDiff
                }
                b'L' => {
                    p = p.add(1);
                    Len::LongDouble
                }
                _ => Len::Default,
            };

            spec.conv = *p;
            if spec.conv == 0 {
                out.extend_from_slice(::core::slice::from_raw_parts(
                    start,
                    p.offset_from(start) as usize,
                ));
                break;
            }
            p = p.add(1);

            match spec.conv {
                b'%' => out.push(b'%'),
                b'd' | b'i' => {
                    let v = next_signed(args, &mut at);
                    emit_signed(out, &spec, v);
                }
                b'u' | b'o' | b'x' | b'X' => {
                    let v = next_unsigned(args, &mut at);
                    emit_unsigned(out, &spec, v);
                }
                b'c' => {
                    let v = next_signed(args, &mut at);
                    emit_string(out, &spec, &[v as u8]);
                }
                b's' => {
                    let v = next_pointer(args, &mut at);
                    let body = read_string(v, spec.prec);
                    let spec = Spec { prec: None, ..spec };
                    emit_string(out, &spec, &body);
                }
                b'p' => {
                    let v = next_pointer(args, &mut at);
                    let body = if v.is_null() {
                        b"(nil)".to_vec()
                    } else {
                        let mut body = b"0x".to_vec();
                        body.extend(digits(v as u64, 16, false));
                        body
                    };
                    let spec = Spec {
                        prec: None,
                        zero: false,
                        ..spec
                    };
                    emit_string(out, &spec, &body);
                }
                b'e' | b'E' | b'f' | b'F' | b'g' | b'G' | b'a' | b'A' => {
                    let v = next_float(args, &mut at);
                    emit_float(out, &spec, v);
                }
                _ => {
                    debug_assert!(false, "unsupported printf conversion");
                    out.extend_from_slice(::core::slice::from_raw_parts(
                        start,
                        p.offset_from(start) as usize,
                    ));
                }
            }
        }
    }
}

/// `fmt` expanded over `args`, with no trailing NUL.
pub unsafe fn format_bytes(fmt: *const c_char, args: &[FmtArg]) -> Vec<u8> {
    let mut out = Vec::new();
    unsafe { format_append(&mut out, fmt, args) };
    out
}

/// The length `fmt` expands to, the answer `vsnprintf(NULL, 0, ...)` gives.
pub unsafe fn format_len(fmt: *const c_char, args: &[FmtArg]) -> usize {
    unsafe { format_bytes(fmt, args).len() }
}

/// Writes into `str` under `snprintf` rules: at most `len` bytes including the
/// NUL, and the length the whole expansion would have had is returned.
pub unsafe fn format_into(
    str: *mut c_char,
    len: usize,
    fmt: *const c_char,
    args: &[FmtArg],
) -> c_int {
    let body = unsafe { format_bytes(fmt, args) };
    if len != 0 {
        let n = body.len().min(len - 1);
        unsafe {
            ::core::ptr::copy_nonoverlapping(body.as_ptr(), str as *mut u8, n);
            *str.add(n) = 0;
        }
    }
    body.len() as c_int
}

/// Appends the expansion to a byte buffer.
pub unsafe fn format_buf(buffer: &mut Buf, fmt: *const c_char, args: &[FmtArg]) -> c_int {
    let body = unsafe { format_bytes(fmt, args) };
    buffer.append(&body);
    0
}

/// A freshly allocated NUL-terminated expansion, as `vasprintf` leaves behind:
/// the caller owns it and frees it with libc `free`.
pub unsafe fn format_alloc(fmt: *const c_char, args: &[FmtArg]) -> CString {
    let body = unsafe { format_bytes(fmt, args) };
    CString::new(body).expect("a formatted string carries no NUL")
}

#[cfg(test)]
#[path = "tests/test_fmt_engine.rs"]
mod tests;

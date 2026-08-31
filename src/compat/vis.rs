use crate::ffi::__ctype_b_loc;
pub use crate::types::*;
pub type ctype_mask = ::core::ffi::c_uint;
pub const _IScntrl: ctype_mask = 2;
pub const _ISgraph: ctype_mask = 32768;
pub const UCHAR_MAX: ::core::ffi::c_int =
    __SCHAR_MAX__ * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const VIS_OCTAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VIS_CSTYLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VIS_SP: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const VIS_TAB: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const VIS_NL: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const VIS_SAFE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const VIS_DQ: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const VIS_ALL: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const VIS_NOSLASH: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const VIS_GLOB: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;

/// One source byte as the C code sees it: `char` is signed on this target, so
/// a byte with the high bit set reaches the encoder sign-extended, and the
/// checks that compare against a whole `int` see the negative value.
fn ch(b: u8) -> ::core::ffi::c_int {
    b as ::core::ffi::c_char as ::core::ffi::c_int
}

/// Ask the C library whether `b` is in `mask`'s character class. The encoder
/// keeps consulting the locale table the C code used, so `isgraph` and
/// `iscntrl` stay exactly as locale-sensitive as they were.
fn ctype_is(b: u8, mask: ctype_mask) -> bool {
    unsafe { *(*__ctype_b_loc()).add(b as usize) as ctype_mask & mask != 0 }
}

/// The four characters a shell would expand, which `VIS_GLOB` escapes.
fn is_glob(c: ::core::ffi::c_int) -> bool {
    c == '*' as ::core::ffi::c_int
        || c == '?' as ::core::ffi::c_int
        || c == '[' as ::core::ffi::c_int
        || c == '#' as ::core::ffi::c_int
}

/// The `isvisible` predicate: whether `c` is written as itself rather than as
/// an escape. Note that `VIS_SAFE` re-admits every printable character, so it
/// outranks `VIS_GLOB`.
fn is_visible(c: ::core::ffi::c_int, flag: ::core::ffi::c_int) -> bool {
    if c != '\\' as ::core::ffi::c_int && flag & VIS_ALL != 0 {
        return false;
    }
    let b = c as u8;
    let printable_ascii = c as ::core::ffi::c_uint <= UCHAR_MAX as ::core::ffi::c_uint
        && b & !0x7f == 0
        && (!is_glob(c) || flag & VIS_GLOB == 0)
        && ctype_is(b, _ISgraph);
    printable_ascii
        || (flag & VIS_SP == 0 && c == ' ' as ::core::ffi::c_int)
        || (flag & VIS_TAB == 0 && c == '\t' as ::core::ffi::c_int)
        || (flag & VIS_NL == 0 && c == '\n' as ::core::ffi::c_int)
        || (flag & VIS_SAFE != 0
            && (c == 0x08 || c == 0x07 || c == '\r' as ::core::ffi::c_int || ctype_is(b, _ISgraph)))
}

/// The bytes one character encodes to, before the terminator. The longest
/// forms — `\377`, `\M-a`, `\M^?`, `\000` — are four bytes.
struct Escaped {
    buf: [u8; 4],
    len: usize,
}

impl Escaped {
    fn new() -> Self {
        Escaped {
            buf: [0; 4],
            len: 0,
        }
    }

    fn push(&mut self, b: u8) {
        self.buf[self.len] = b;
        self.len += 1;
    }

    fn bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

/// Encode one character. `nextc` is the character that follows it, which only
/// a `VIS_CSTYLE` NUL cares about: it pads itself to three digits so that an
/// octal digit coming after cannot be read as part of the escape.
fn escape(c: ::core::ffi::c_int, flag: ::core::ffi::c_int, nextc: ::core::ffi::c_int) -> Escaped {
    let mut out = Escaped::new();
    if is_visible(c, flag) {
        if c == '"' as ::core::ffi::c_int && flag & VIS_DQ != 0
            || c == '\\' as ::core::ffi::c_int && flag & VIS_NOSLASH == 0
        {
            out.push(b'\\');
        }
        out.push(c as u8);
        return out;
    }
    if flag & VIS_CSTYLE != 0 {
        let named = match c {
            0x0a => Some(b'n'),
            0x0d => Some(b'r'),
            0x08 => Some(b'b'),
            0x07 => Some(b'a'),
            0x0b => Some(b'v'),
            0x09 => Some(b't'),
            0x0c => Some(b'f'),
            0x20 => Some(b's'),
            _ => None,
        };
        if let Some(named) = named {
            out.push(b'\\');
            out.push(named);
            return out;
        }
        if c == 0 {
            out.push(b'\\');
            out.push(b'0');
            if (nextc as u8) >= b'0' && (nextc as u8) <= b'7' {
                out.push(b'0');
                out.push(b'0');
            }
            return out;
        }
    }
    if c & 0o177 == ' ' as ::core::ffi::c_int
        || flag & VIS_OCTAL != 0
        || flag & VIS_GLOB != 0 && is_glob(c)
    {
        let b = c as u8;
        out.push(b'\\');
        out.push((b >> 6 & 0o7) + b'0');
        out.push((b >> 3 & 0o7) + b'0');
        out.push((b & 0o7) + b'0');
        return out;
    }
    if flag & VIS_NOSLASH == 0 {
        out.push(b'\\');
    }
    let mut c = c;
    if c & 0o200 != 0 {
        c &= 0o177;
        out.push(b'M');
    }
    if ctype_is(c as u8, _IScntrl) {
        out.push(b'^');
        if c == 0o177 {
            out.push(b'?');
        } else {
            out.push((c + '@' as ::core::ffi::c_int) as u8);
        }
    } else {
        out.push(b'-');
        out.push(c as u8);
    }
    out
}

/// Encode every byte of `src`, each one seeing its successor as `nextc` and
/// the last one seeing a NUL.
fn encode(src: &[u8], flag: ::core::ffi::c_int) -> Vec<u8> {
    let mut out = Vec::with_capacity(src.len());
    for (i, &b) in src.iter().enumerate() {
        let nextc = ch(src.get(i + 1).copied().unwrap_or(0));
        out.extend_from_slice(escape(ch(b), flag, nextc).bytes());
    }
    out
}

/// `strnvis`'s pass over `src`: the bytes that fit in a `siz`-byte buffer
/// alongside a terminator, and the length the whole encoding would have had.
///
/// A character is only written when its escape fits entirely, and a visible
/// character that needs a leading backslash needs both bytes or neither. The
/// returned length is the truncated output plus the encoding of everything the
/// pass did not reach — but, as in C, only when the last character considered
/// would have run past the end.
fn encode_bounded(
    src: &[u8],
    siz: usize,
    flag: ::core::ffi::c_int,
) -> (Vec<u8>, ::core::ffi::c_int) {
    let end = siz as isize - 1;
    let mut out: Vec<u8> = Vec::new();
    let mut last: isize = 0;
    let mut i = 0;
    while i < src.len() && (out.len() as isize) < end {
        let c = ch(src[i]);
        if is_visible(c, flag) {
            if c == '"' as ::core::ffi::c_int && flag & VIS_DQ != 0
                || c == '\\' as ::core::ffi::c_int && flag & VIS_NOSLASH == 0
            {
                if out.len() as isize + 1 >= end {
                    last = 2;
                    break;
                }
                out.push(b'\\');
            }
            last = 1;
            out.push(c as u8);
            i += 1;
        } else {
            let escaped = escape(c, flag, ch(src.get(i + 1).copied().unwrap_or(0)));
            last = escaped.len as isize;
            if out.len() as isize + last > end {
                break;
            }
            out.extend_from_slice(escaped.bytes());
            i += 1;
        }
    }
    let mut len = out.len();
    if out.len() as isize + last > end {
        len += encode(&src[i..], flag).len();
    }
    (out, len as ::core::ffi::c_int)
}

/// Write `out` and a terminator at `dst`, which must have room for both.
unsafe fn store(dst: *mut ::core::ffi::c_char, out: &[u8]) {
    unsafe {
        for (i, &b) in out.iter().enumerate() {
            *dst.add(i) = b as ::core::ffi::c_char;
        }
        *dst.add(out.len()) = 0;
    }
}

pub unsafe fn vis(
    dst: *mut ::core::ffi::c_char,
    c: ::core::ffi::c_int,
    flag: ::core::ffi::c_int,
    nextc: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let out = escape(c, flag, nextc);
        store(dst, out.bytes());
        dst.add(out.len)
    }
}

pub unsafe fn strvis(
    dst: *mut ::core::ffi::c_char,
    src: *const ::core::ffi::c_char,
    flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let out = encode(::core::ffi::CStr::from_ptr(src).to_bytes(), flag);
        store(dst, &out);
        out.len() as ::core::ffi::c_int
    }
}

pub unsafe fn strnvis(
    dst: *mut ::core::ffi::c_char,
    src: *const ::core::ffi::c_char,
    siz: size_t,
    flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let (out, len) = encode_bounded(::core::ffi::CStr::from_ptr(src).to_bytes(), siz, flag);
        for (i, &b) in out.iter().enumerate() {
            *dst.add(i) = b as ::core::ffi::c_char;
        }
        if siz > 0 as size_t {
            *dst.add(out.len()) = 0;
        }
        len
    }
}

/// Encode `src` into a string of its own. The C took a buffer from the
/// allocator and left the caller to free it; the string owns its bytes here,
/// so the out-of-memory return the C had is gone with the manual free.
pub unsafe fn stravis(
    src: *const ::core::ffi::c_char,
    flag: ::core::ffi::c_int,
) -> ::std::ffi::CString {
    unsafe {
        let out = encode(::core::ffi::CStr::from_ptr(src).to_bytes(), flag);
        ::std::ffi::CString::new(out).expect("an encoded string holds no nul")
    }
}

pub unsafe fn strvisx(
    dst: *mut ::core::ffi::c_char,
    src: *const ::core::ffi::c_char,
    len: size_t,
    flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let out = encode(::core::slice::from_raw_parts(src as *const u8, len), flag);
        store(dst, &out);
        out.len() as ::core::ffi::c_int
    }
}

pub const __SCHAR_MAX__: ::core::ffi::c_int = 127 as ::core::ffi::c_int;

#[cfg(test)]
#[path = "../tests/test_compat_vis.rs"]
mod tests;

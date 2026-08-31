//! The three `wchar_t` functions tmux uses in place of the C library's own,
//! answered by utf8proc instead of the locale: a width, a character read out
//! of bytes and a character written back to them.
//!
//! Each is a thin shim over one or two utf8proc calls, and each keeps the C
//! library's answer for the pointer it was given nothing at — zero, meaning
//! nothing read or written — because that is how `mbtowc` and `wctomb` are
//! asked whether they hold any state between calls.
//!
//! Coverage exemptions: none.
pub use crate::types::*;
use ::core::ffi::{c_char, c_int, c_uint};

unsafe extern "C" {
    fn utf8proc_iterate(
        str: *const utf8proc_uint8_t,
        strlen: utf8proc_ssize_t,
        codepoint_ref: *mut utf8proc_int32_t,
    ) -> utf8proc_ssize_t;
    fn utf8proc_codepoint_valid(codepoint: utf8proc_int32_t) -> utf8proc_bool;
    fn utf8proc_encode_char(
        codepoint: utf8proc_int32_t,
        dst: *mut utf8proc_uint8_t,
    ) -> utf8proc_ssize_t;
    fn utf8proc_charwidth(codepoint: utf8proc_int32_t) -> c_int;
    fn utf8proc_category(codepoint: utf8proc_int32_t) -> utf8proc_category_t;
}

type utf8proc_uint8_t = u8;
type utf8proc_int32_t = i32;
type utf8proc_ssize_t = isize;
type utf8proc_bool = bool;
type utf8proc_category_t = c_uint;

/// The private-use category, the one category whose width utf8proc is not
/// asked about.
const UTF8PROC_CATEGORY_CO: utf8proc_category_t = 29;

/// The number of columns `wc` takes up.
///
/// Powerline and the fonts like it put their glyphs in the private use area,
/// where the width is formally ambiguous and utf8proc answers none; tmux gives
/// those one column instead.
pub fn utf8proc_wcwidth(wc: wchar_t) -> c_int {
    unsafe {
        if utf8proc_category(wc as utf8proc_int32_t) == UTF8PROC_CATEGORY_CO {
            return 1;
        }
        utf8proc_charwidth(wc as utf8proc_int32_t)
    }
}

/// Reads the character the first `n` bytes of `s` spell, writes it to `pwc`
/// and answers how many bytes it took; -1 for bytes that spell none, and zero
/// for no string at all.
///
/// utf8proc writes -1 to `pwc` for a codepoint that is not one, which is a
/// separate answer from the negative length it gives for bytes it could not
/// read at all; both are the same refusal here.
pub unsafe fn utf8proc_mbtowc(pwc: *mut wchar_t, s: *const c_char, n: size_t) -> c_int {
    unsafe {
        if s.is_null() {
            return 0;
        }
        let read = utf8proc_iterate(
            s as *const utf8proc_uint8_t,
            n as utf8proc_ssize_t,
            pwc as *mut utf8proc_int32_t,
        );
        if *pwc == -1 as wchar_t || read < 0 {
            return -1;
        }
        read as c_int
    }
}

/// Writes `wc` to `s` as its bytes and answers how many it took; -1 for a
/// codepoint that stands for no character, and zero for nowhere to write it.
pub unsafe fn utf8proc_wctomb(s: *mut c_char, wc: wchar_t) -> c_int {
    unsafe {
        if s.is_null() {
            return 0;
        }
        if !utf8proc_codepoint_valid(wc as utf8proc_int32_t) {
            return -1;
        }
        utf8proc_encode_char(wc as utf8proc_int32_t, s as *mut utf8proc_uint8_t) as c_int
    }
}

#[cfg(test)]
#[path = "../tests/test_compat_utf8proc.rs"]
mod tests;

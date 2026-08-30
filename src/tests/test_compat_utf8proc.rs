use super::*;
use ::core::ffi::c_char;
use ::core::ptr::{null, null_mut};

/// The codepoint and the length `utf8proc_mbtowc` reads out of `s`.
fn read(s: &[u8]) -> (::core::ffi::c_int, wchar_t) {
    unsafe {
        let mut wc: wchar_t = 0;
        let n = utf8proc_mbtowc(&raw mut wc, s.as_ptr() as *const c_char, s.len() as size_t);
        (n, wc)
    }
}

/// A private-use codepoint has no width of its own, so the compatibility
/// wrapper gives it one column rather than the zero utf8proc answers.
#[test]
fn a_width_is_one_column_for_anything_in_the_private_use_area() {
    {
        assert_eq!(utf8proc_wcwidth(0xe000), 1);
        assert_eq!(utf8proc_wcwidth(0xf8ff), 1);
    }
}

#[test]
fn a_width_is_what_utf8proc_says_for_everything_else() {
    {
        assert_eq!(utf8proc_wcwidth('a' as wchar_t), 1);
        assert_eq!(utf8proc_wcwidth(0x4e00), 2);
        assert_eq!(utf8proc_wcwidth(0x0301), 0);
    }
}

#[test]
fn a_character_is_read_back_with_the_bytes_it_took() {
    {
        assert_eq!(read(b"a"), (1, 'a' as wchar_t));
        assert_eq!(read("\u{4e00}".as_bytes()), (3, 0x4e00));
        assert_eq!(read("\u{e9}".as_bytes()), (2, 0xe9));
    }
}

#[test]
fn a_byte_that_starts_no_character_is_read_as_an_error() {
    {
        assert_eq!(read(b"\xff").0, -1);
        assert_eq!(read(b"\xc3").0, -1);
        assert_eq!(read(b"").0, -1);
    }
}

/// Reading from nothing at all answers nothing read, which is what the C
/// library's own `mbtowc` answers for a null pointer.
#[test]
fn reading_from_no_string_answers_nothing_read() {
    unsafe {
        let mut wc: wchar_t = 0;
        assert_eq!(utf8proc_mbtowc(&raw mut wc, null::<c_char>(), 1), 0);
    }
}

#[test]
fn a_codepoint_is_written_out_as_its_bytes() {
    unsafe {
        let mut out = [0u8; 8];
        let n = utf8proc_wctomb(out.as_mut_ptr() as *mut c_char, 'a' as wchar_t);
        assert_eq!((n, out[0]), (1, b'a'));
        let n = utf8proc_wctomb(out.as_mut_ptr() as *mut c_char, 0x4e00);
        assert_eq!(n, 3);
        assert_eq!(&out[..3], "\u{4e00}".as_bytes());
    }
}

#[test]
fn a_codepoint_that_is_no_character_is_not_written_out() {
    unsafe {
        let mut out = [0u8; 8];
        let s = out.as_mut_ptr() as *mut c_char;
        assert_eq!(utf8proc_wctomb(s, 0xd800), -1);
        assert_eq!(utf8proc_wctomb(s, 0x110000), -1);
        assert_eq!(utf8proc_wctomb(s, -1), -1);
    }
}

/// Writing to nothing at all answers nothing written, which is how the C
/// library's own `wctomb` is asked whether it holds any state.
#[test]
fn writing_to_no_string_answers_nothing_written() {
    unsafe {
        assert_eq!(utf8proc_wctomb(null_mut::<c_char>(), 'a' as wchar_t), 0);
    }
}

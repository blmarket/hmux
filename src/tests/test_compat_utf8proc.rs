use super::*;

/// The codepoint and the length `utf8proc_mbtowc` reads out of `s`.
fn read(s: &[u8]) -> (c_int, wchar_t) {
    let mut wc: wchar_t = 0;
    let n = utf8proc_mbtowc(Some(&mut wc), Some(s));
    (n, wc)
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
    let mut wc: wchar_t = 0;
    assert_eq!(utf8proc_mbtowc(Some(&mut wc), None), 0);
}

#[test]
fn a_codepoint_is_written_out_as_its_bytes() {
    let mut out = [0u8; 4];
    let n = utf8proc_wctomb(Some(&mut out), 'a' as wchar_t);
    assert_eq!((n, out[0]), (1, b'a'));
    let n = utf8proc_wctomb(Some(&mut out), 0x4e00);
    assert_eq!(n, 3);
    assert_eq!(&out[..3], "\u{4e00}".as_bytes());
}

#[test]
fn a_codepoint_that_is_no_character_is_not_written_out() {
    let mut out = [0u8; 4];
    assert_eq!(utf8proc_wctomb(Some(&mut out), 0xd800), -1);
    assert_eq!(utf8proc_wctomb(Some(&mut out), 0x110000), -1);
    assert_eq!(utf8proc_wctomb(Some(&mut out), -1), -1);
}

/// Writing to nothing at all answers nothing written, which is how the C
/// library's own `wctomb` is asked whether it holds any state.
#[test]
fn writing_to_no_string_answers_nothing_written() {
    assert_eq!(utf8proc_wctomb(None, 'a' as wchar_t), 0);
}

use super::*;
use ::core::ffi::{CStr, c_int};

/// One `vis` call into a guarded buffer, returning the bytes it wrote. The
/// returned pointer must land on the terminator it also writes.
fn vis_call(c: c_int, flag: c_int, nextc: c_int) -> Vec<u8> {
    vis(c, flag, nextc)
}

fn strvis_call(src: &CStr, flag: c_int) -> (c_int, Vec<u8>) {
    let bytes = strvis(src, flag);
    (bytes.len() as c_int, bytes)
}

fn strvisx_call(src: &[u8], flag: c_int) -> (c_int, Vec<u8>) {
    let bytes = strvisx(src, flag);
    (bytes.len() as c_int, bytes)
}

/// `strnvis` into a `siz`-byte window with guard bytes on both sides; the
/// second element is the whole window, so a short write shows up as the
/// `0xaa` filler it left behind.
fn strnvis_call(src: &CStr, siz: usize, flag: c_int) -> (c_int, Vec<u8>) {
    let result = strnvis(src, siz as size_t, flag);
    let mut written = vec![0xaa; siz];
    written[..result.output.len()].copy_from_slice(&result.output);
    if siz > 0 {
        written[result.output.len()] = 0;
    }
    (result.len, written)
}

fn stravis_call(src: &CStr, flag: c_int) -> (c_int, Vec<u8>) {
    let out = stravis(src, flag);
    let bytes = out.as_bytes().to_vec();
    (bytes.len() as c_int, bytes)
}

#[test]
fn printable_characters_are_written_as_themselves() {
    assert_eq!(vis_call('a' as c_int, 0, 0), b"a");
    assert_eq!(vis_call('~' as c_int, 0, 0), b"~");
    assert_eq!(vis_call(' ' as c_int, 0, 0), b" ");
    assert_eq!(vis_call('\t' as c_int, 0, 0), b"\t");
    assert_eq!(vis_call('\n' as c_int, 0, 0), b"\n");
}

#[test]
fn quotes_and_backslashes_are_doubled_on_request() {
    assert_eq!(vis_call('"' as c_int, VIS_DQ, 0), b"\\\"");
    assert_eq!(vis_call('"' as c_int, 0, 0), b"\"");
    assert_eq!(vis_call('\\' as c_int, 0, 0), b"\\\\");
    assert_eq!(vis_call('\\' as c_int, VIS_NOSLASH, 0), b"\\");
}

#[test]
fn vis_all_escapes_everything_but_a_backslash() {
    assert_eq!(vis_call('a' as c_int, VIS_ALL, 0), b"\\-a");
    assert_eq!(vis_call('a' as c_int, VIS_ALL | VIS_OCTAL, 0), b"\\141");
    assert_eq!(vis_call('\\' as c_int, VIS_ALL, 0), b"\\\\");
}

#[test]
fn the_whitespace_flags_turn_their_character_into_an_escape() {
    assert_eq!(vis_call(' ' as c_int, VIS_SP, 0), b"\\040");
    assert_eq!(vis_call('\t' as c_int, VIS_TAB, 0), b"\\^I");
    assert_eq!(vis_call('\n' as c_int, VIS_NL, 0), b"\\^J");
}

#[test]
fn vis_safe_keeps_the_harmless_control_characters() {
    assert_eq!(vis_call(0x08, VIS_SAFE, 0), b"\x08");
    assert_eq!(vis_call(0x07, VIS_SAFE, 0), b"\x07");
    assert_eq!(vis_call('\r' as c_int, VIS_SAFE, 0), b"\r");
    assert_eq!(vis_call(0x01, VIS_SAFE, 0), b"\\^A");
}

#[test]
fn vis_glob_makes_the_glob_metacharacters_octal() {
    assert_eq!(vis_call('*' as c_int, VIS_GLOB, 0), b"\\052");
    assert_eq!(vis_call('?' as c_int, VIS_GLOB, 0), b"\\077");
    assert_eq!(vis_call('[' as c_int, VIS_GLOB, 0), b"\\133");
    assert_eq!(vis_call('#' as c_int, VIS_GLOB, 0), b"\\043");
    assert_eq!(vis_call('*' as c_int, 0, 0), b"*");
    assert_eq!(vis_call('*' as c_int, VIS_GLOB | VIS_SAFE, 0), b"*");
}

#[test]
fn vis_cstyle_names_the_familiar_escapes() {
    assert_eq!(vis_call('\n' as c_int, VIS_CSTYLE | VIS_NL, 0), b"\\n");
    assert_eq!(vis_call('\r' as c_int, VIS_CSTYLE, 0), b"\\r");
    assert_eq!(vis_call(0x08, VIS_CSTYLE, 0), b"\\b");
    assert_eq!(vis_call(0x07, VIS_CSTYLE, 0), b"\\a");
    assert_eq!(vis_call(0x0b, VIS_CSTYLE, 0), b"\\v");
    assert_eq!(vis_call('\t' as c_int, VIS_CSTYLE | VIS_TAB, 0), b"\\t");
    assert_eq!(vis_call(0x0c, VIS_CSTYLE, 0), b"\\f");
    assert_eq!(vis_call(' ' as c_int, VIS_CSTYLE | VIS_SP, 0), b"\\s");
}

#[test]
fn vis_cstyle_pads_a_nul_that_an_octal_digit_follows() {
    assert_eq!(vis_call(0, VIS_CSTYLE, 'x' as c_int), b"\\0");
    assert_eq!(vis_call(0, VIS_CSTYLE, '8' as c_int), b"\\0");
    assert_eq!(vis_call(0, VIS_CSTYLE, '0' as c_int), b"\\000");
    assert_eq!(vis_call(0, VIS_CSTYLE, '7' as c_int), b"\\000");
    assert_eq!(vis_call(0, 0, 0), b"\\^@");
}

#[test]
fn vis_cstyle_falls_back_to_the_generic_forms() {
    assert_eq!(vis_call(0x01, VIS_CSTYLE, 0), b"\\^A");
    assert_eq!(vis_call(0xe1, VIS_CSTYLE, 0), b"\\M-a");
}

#[test]
fn control_characters_become_caret_escapes() {
    assert_eq!(vis_call(0x01, 0, 0), b"\\^A");
    assert_eq!(vis_call(0x7f, 0, 0), b"\\^?");
    assert_eq!(vis_call(0x01, VIS_NOSLASH, 0), b"^A");
}

#[test]
fn high_bit_characters_get_a_meta_prefix() {
    assert_eq!(vis_call(0x80, 0, 0), b"\\M^@");
    assert_eq!(vis_call(0xe1, 0, 0), b"\\M-a");
    assert_eq!(vis_call(0xff, 0, 0), b"\\M^?");
    assert_eq!(vis_call(0xe1, VIS_NOSLASH, 0), b"M-a");
}

#[test]
fn a_sign_extended_character_is_treated_as_its_low_byte() {
    assert_eq!(vis_call(-31, 0, 0), vis_call(0xe1, 0, 0));
    assert_eq!(vis_call(-1, 0, 0), vis_call(0xff, 0, 0));
    assert_eq!(vis_call(-96, VIS_SAFE, 0), b"\\240");
}

#[test]
fn vis_octal_numbers_every_escape() {
    assert_eq!(vis_call(0x01, VIS_OCTAL, 0), b"\\001");
    assert_eq!(vis_call(0xff, VIS_OCTAL, 0), b"\\377");
    assert_eq!(vis_call(0xa0, 0, 0), b"\\240");
}

#[test]
fn strvis_encodes_a_whole_string() {
    assert_eq!(strvis_call(c"", 0), (0, b"".to_vec()));
    assert_eq!(strvis_call(c"ab", 0), (2, b"ab".to_vec()));
    assert_eq!(
        strvis_call(c"a\tb", VIS_TAB | VIS_CSTYLE),
        (4, b"a\\tb".to_vec())
    );
    assert_eq!(strvis_call(c"\xe1", 0), (4, b"\\M-a".to_vec()));
}

#[test]
fn strvisx_encodes_a_counted_string_including_nuls() {
    assert_eq!(strvisx_call(b"", 0), (0, b"".to_vec()));
    assert_eq!(strvisx_call(b"a", 0), (1, b"a".to_vec()));
    assert_eq!(strvisx_call(b"ab", 0), (2, b"ab".to_vec()));
    assert_eq!(strvisx_call(b"a\0b", VIS_CSTYLE), (4, b"a\\0b".to_vec()));
    assert_eq!(strvisx_call(b"a\0 b", VIS_CSTYLE), (5, b"a\\0 b".to_vec()));
    assert_eq!(strvisx_call(b"\x000", VIS_CSTYLE), (5, b"\\0000".to_vec()));
}

#[test]
fn strnvis_encodes_within_the_buffer() {
    assert_eq!(
        strnvis_call(c"ab", 8, 0),
        (2, b"ab\0\xaa\xaa\xaa\xaa\xaa".to_vec())
    );
    assert_eq!(strnvis_call(c"ab", 3, 0), (2, b"ab\0".to_vec()));
    assert_eq!(
        strnvis_call(c"a\x01b", 8, 0),
        (5, b"a\\^Ab\0\xaa\xaa".to_vec())
    );
    assert_eq!(
        strnvis_call(c"\"", 8, VIS_DQ),
        (2, b"\\\"\0\xaa\xaa\xaa\xaa\xaa".to_vec())
    );
    assert_eq!(
        strnvis_call(c"\x07*", 8, VIS_SAFE | VIS_GLOB),
        (2, b"\x07*\0\xaa\xaa\xaa\xaa\xaa".to_vec())
    );
}

#[test]
fn strnvis_returns_the_full_length_when_it_truncates() {
    assert_eq!(strnvis_call(c"abcd", 3, 0), (4, b"ab\0".to_vec()));
    assert_eq!(strnvis_call(c"a\\b", 3, 0), (4, b"a\0\xaa".to_vec()));
    assert_eq!(strnvis_call(c"a\x01b", 4, 0), (5, b"a\0\xaa\xaa".to_vec()));
}

#[test]
fn strnvis_writes_nothing_when_there_is_no_room() {
    assert_eq!(strnvis_call(c"ab", 0, 0), (2, Vec::new()));
}

#[test]
fn stravis_allocates_the_encoded_string() {
    assert_eq!(stravis_call(c"", 0), (0, b"".to_vec()));
    assert_eq!(stravis_call(c"ab", 0), (2, b"ab".to_vec()));
    assert_eq!(
        stravis_call(c"a\x01b", VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL),
        (6, b"a\\001b".to_vec())
    );
}

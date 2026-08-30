//! Extra coverage for [`crate::compat`] – edge paths not hit by the
//! in-module suite. Exercises `VIS_NOSLASH`+`VIS_ALL` interactions, the
//! `encode_bounded` backslash-split guard, NUL padding via `strvisx`,
//! and high-bit `VIS_OCTAL` forcing.

use crate::compat::{
    VIS_ALL, VIS_CSTYLE, VIS_DQ, VIS_GLOB, VIS_NL, VIS_NOSLASH, VIS_OCTAL, VIS_SAFE, VIS_SP,
    VIS_TAB, stravis, strnvis, strvis, strvisx, vis,
};
use ::core::ffi::{CStr, c_char, c_int};

// ---------------------------------------------------------------------------
// helpers – mirror the in-module suite but kept local so the file is self-
// contained and does not rely on private helpers inside vis.rs.
// ---------------------------------------------------------------------------

fn vis_call(c: c_int, flag: c_int, nextc: c_int) -> Vec<u8> {
    let mut buf = [0xaau8 as c_char; 16];
    let end = unsafe { vis(buf.as_mut_ptr(), c, flag, nextc) };
    let n = unsafe { end.offset_from(buf.as_ptr()) } as usize;
    assert_eq!(buf[n], 0, "missing terminator for {c:#x}/{flag:#x}");
    buf[..n].iter().map(|&b| b as u8).collect()
}

fn strvis_call(src: &CStr, flag: c_int) -> (c_int, Vec<u8>) {
    let mut dst = vec![0xaau8 as c_char; src.count_bytes() * 4 + 8];
    let n = unsafe { strvis(dst.as_mut_ptr(), src.as_ptr(), flag) };
    let bytes = unsafe { CStr::from_ptr(dst.as_ptr()).to_bytes().to_vec() };
    (n, bytes)
}

fn strnvis_call(src: &CStr, siz: usize, flag: c_int) -> (c_int, Vec<u8>) {
    let mut buf = vec![0xaau8 as c_char; siz + 16];
    let dst = unsafe { buf.as_mut_ptr().add(8) };
    let n = unsafe { strnvis(dst, src.as_ptr(), siz as crate::types::size_t, flag) };
    let written = buf[8..8 + siz].iter().map(|&b| b as u8).collect();
    (n, written)
}

fn strvisx_call(src: &[u8], len: usize, flag: c_int) -> (c_int, Vec<u8>) {
    let mut owned: Vec<c_char> = src.iter().map(|&b| b as c_char).collect();
    owned.push(0);
    let mut dst = vec![0xaau8 as c_char; src.len() * 4 + 8];
    let n = unsafe {
        strvisx(
            dst.as_mut_ptr(),
            owned.as_ptr(),
            len as crate::types::size_t,
            flag,
        )
    };
    let bytes = unsafe { CStr::from_ptr(dst.as_ptr()).to_bytes().to_vec() };
    (n, bytes)
}

fn stravis_call(src: &CStr, flag: c_int) -> (c_int, Vec<u8>) {
    let bytes = unsafe { stravis(src.as_ptr(), flag) }.into_bytes();
    (bytes.len() as c_int, bytes)
}

// ---------------------------------------------------------------------------
// edge tests
// ---------------------------------------------------------------------------

#[test]
fn vis_noslash_removes_backslash_from_meta_and_caret_forms() {
    // caret form without the leading '\'
    assert_eq!(vis_call(0x01, VIS_NOSLASH, 0), b"^A");
    assert_eq!(vis_call(0x7f, VIS_NOSLASH, 0), b"^?");
    // meta form without the leading '\' – the M- prefix stays but '\' is stripped
    assert_eq!(vis_call(0xe1, VIS_NOSLASH, 0), b"M-a");
    assert_eq!(vis_call(0x80, VIS_NOSLASH, 0), b"M^@");
    assert_eq!(vis_call(0xff, VIS_NOSLASH, 0), b"M^?");
    // printable high-bit with VIS_SAFE is still escaped via octal, but NOSLASH not involved
    // cstyle path is unaffected by NOSLASH except for the backslash itself
    assert_eq!(vis_call(0x01, VIS_CSTYLE | VIS_NOSLASH, 0), b"^A");
}

#[test]
fn vis_all_with_octal_and_noslash_combinations() {
    // VIS_ALL forces octal-like via the \M- / \^ path; VIS_OCTAL forces 3-digit octal
    assert_eq!(vis_call('A' as c_int, VIS_ALL, 0), b"\\-A");
    assert_eq!(vis_call('A' as c_int, VIS_ALL | VIS_OCTAL, 0), b"\\101");
    assert_eq!(vis_call('A' as c_int, VIS_ALL | VIS_NOSLASH, 0), b"-A");
    // backslash itself is always doubled even under VIS_ALL (no -A dance)
    assert_eq!(vis_call('\\' as c_int, VIS_ALL, 0), b"\\\\");
    assert_eq!(vis_call('\\' as c_int, VIS_ALL | VIS_NOSLASH, 0), b"\\");
    // glob interaction: VIS_SAFE wins over VIS_GLOB
    assert_eq!(vis_call('#' as c_int, VIS_GLOB | VIS_SAFE, 0), b"#");
    assert_eq!(vis_call('#' as c_int, VIS_GLOB | VIS_OCTAL, 0), b"\\043");
}

#[test]
fn vis_cstyle_octal_vs_generic_for_high_bit_and_controls() {
    // cstyle NUL with and without octal follower
    assert_eq!(vis_call(0, VIS_CSTYLE, '0' as c_int), b"\\000");
    assert_eq!(vis_call(0, VIS_CSTYLE, '9' as c_int), b"\\0");
    // high-bit under VIS_OCTAL is 3-digit octal; 0xa0 low 7 bits is space so always octal
    assert_eq!(vis_call(0xa0, VIS_OCTAL, 0), b"\\240");
    assert_eq!(vis_call(0xa0, 0, 0), b"\\240");
    assert_eq!(vis_call(0xa0, VIS_CSTYLE, 0), b"\\240");
    // control via cstyle still uses caret when not named
    assert_eq!(vis_call(0x1f, VIS_CSTYLE, 0), b"\\^_");
    assert_eq!(vis_call(0x1f, VIS_OCTAL, 0), b"\\037");
}

#[test]
fn strnvis_exact_fit_and_backslash_split_guard() {
    // fits exactly in sized buffer (2 chars + NUL in siz=3)
    assert_eq!(strnvis_call(c"ab", 3, 0), (2, b"ab\0".to_vec()));
    // needs 4 bytes "a\"b" via "\" escaping backslash: quoted DQ needs 2 bytes
    // dst siz=3 cannot fit the 2-byte "\"\"" sequence, so it writes only "a"
    assert_eq!(strnvis_call(c"a\\b", 3, 0), (4, b"a\0\xaa".to_vec()));
    // siz=1 can hold only the terminator – last==0 so no full-length report
    assert_eq!(strnvis_call(c"ab", 1, 0), (0, b"\0".to_vec()));
    // siz=2 truncates after one char and reports full length
    assert_eq!(strnvis_call(c"abcd", 3, 0), (4, b"ab\0".to_vec()));
    // visible DQ that needs backslash also hits the guard
    assert_eq!(strnvis_call(c"\"", 2, VIS_DQ), (2, b"\0\xaa".to_vec()));
    assert_eq!(strnvis_call(c"\"", 3, VIS_DQ), (2, b"\\\"\0".to_vec()));
}

#[test]
fn strnvis_truncation_reports_full_length_with_mixed_escapes() {
    // "a\x01b" encodes as "a\^Ab" (5 bytes) – siz=4 cannot fit ^A escape
    assert_eq!(strnvis_call(c"a\x01b", 4, 0), (5, b"a\0\xaa\xaa".to_vec()));
    // VIS_TAB forces caret form for tab (\^I = 3 bytes, total 5)
    assert_eq!(
        strnvis_call(c"a\tb", 8, VIS_TAB),
        (5, b"a\\^Ib\0\xaa\xaa".to_vec())
    );
    // VIS_CSTYLE changes the escape shape but still counts correctly
    assert_eq!(
        strnvis_call(c"a\tb", 8, VIS_TAB | VIS_CSTYLE),
        (4, b"a\\tb\0\xaa\xaa\xaa".to_vec())
    );
}

#[test]
fn strvisx_handles_embedded_nuls_and_octal_padding() {
    // empty counted string
    assert_eq!(strvisx_call(b"", 0, 0), (0, b"".to_vec()));
    // single NUL via cstyle without follower
    assert_eq!(strvisx_call(b"\0", 1, VIS_CSTYLE), (2, b"\\0".to_vec()));
    // NUL followed by octal digit pads to 3 digits
    assert_eq!(
        strvisx_call(b"\x000", 2, VIS_CSTYLE),
        (5, b"\\0000".to_vec())
    );
    // NUL followed by non-octal does not pad
    assert_eq!(strvisx_call(b"\0a", 2, VIS_CSTYLE), (3, b"\\0a".to_vec()));
    // raw NUL without cstyle is caret form
    assert_eq!(strvisx_call(b"\0", 1, 0), (3, b"\\^@".to_vec()));
    // counted string with interior NUL + visible tail
    assert_eq!(strvisx_call(b"a\0b", 3, VIS_CSTYLE), (4, b"a\\0b".to_vec()));
}

#[test]
fn stravis_and_strvis_roundtrip_with_mixed_flags() {
    // stravis allocates the same bytes strvis would write
    let (n1, b1) = stravis_call(c"hello", 0);
    let (n2, b2) = strvis_call(c"hello", 0);
    assert_eq!(n1, n2);
    assert_eq!(b1, b2);

    // meta-heavy string with VIS_OCTAL
    assert_eq!(
        strvis_call(c"\xe1\xff", VIS_OCTAL),
        (8, b"\\341\\377".to_vec())
    );
    assert_eq!(
        stravis_call(c"\xe1\xff", VIS_OCTAL),
        (8, b"\\341\\377".to_vec())
    );

    // VIS_SP and VIS_NL mix: space and newline become named under CSTYLE
    assert_eq!(
        strvis_call(c"a b\nc", VIS_SP | VIS_NL | VIS_CSTYLE),
        (7, b"a\\sb\\nc".to_vec())
    );
}

#[test]
fn vis_whitespace_and_safe_interactions() {
    // VIS_SP|VIS_NL|VIS_TAB each force escaping
    assert_eq!(vis_call(' ' as c_int, VIS_SP | VIS_CSTYLE, 0), b"\\s");
    assert_eq!(vis_call(' ' as c_int, VIS_SP, 0), b"\\040");
    assert_eq!(vis_call('\n' as c_int, VIS_NL | VIS_CSTYLE, 0), b"\\n");
    assert_eq!(vis_call('\t' as c_int, VIS_TAB | VIS_CSTYLE, 0), b"\\t");
    // VIS_SAFE keeps \b \a \r visible, otherwise they are caret escapes
    assert_eq!(vis_call(0x08, VIS_SAFE, 0), b"\x08");
    assert_eq!(vis_call(0x08, 0, 0), b"\\^H");
    assert_eq!(vis_call(0x07, VIS_SAFE, 0), b"\x07");
    assert_eq!(vis_call(0x07, 0, 0), b"\\^G");
    // VIS_SAFE does not keep generic controls
    assert_eq!(vis_call(0x01, VIS_SAFE, 0), b"\\^A");
    assert_eq!(vis_call(0x1f, VIS_SAFE, 0), b"\\^_");
}

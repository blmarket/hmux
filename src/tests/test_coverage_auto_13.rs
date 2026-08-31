//! Coverage for [`crate::style`] — pure string helpers.
//!
//! `colour.rs` is dominated by table lookups and `strtonum`/`sscanf` parsing
//! that needs no live server. These tests exercise the deterministic string
//! surface — `colour_tostring`, `colour_fromstring`, `colour_byname` and
//! `colour_parseX11` — pinning hex formatting, case-insensitivity, delimiter
//! handling and grey-percentage rounding without touching the option trees.

use crate::style::{
    COLOUR_FLAG_256, COLOUR_FLAG_RGB, colour_byname, colour_fromstring, colour_parseX11,
    colour_tostring,
};
use ::core::ffi::CStr;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn tostring(c: ::core::ffi::c_int) -> String {
    colour_tostring(c).to_str().unwrap().to_owned()
}

fn fromstring(s: &CStr) -> ::core::ffi::c_int {
    unsafe { colour_fromstring(s.as_ptr()) }
}

fn byname(s: &CStr) -> ::core::ffi::c_int {
    unsafe { colour_byname(s.as_ptr()) }
}

fn parse_x11(s: &CStr) -> ::core::ffi::c_int {
    unsafe { colour_parseX11(s.as_ptr()) }
}

// ---------------------------------------------------------------------------
// flag constants
// ---------------------------------------------------------------------------

#[test]
fn colour_flag_constants_are_distinct_powers_of_two() {
    assert_eq!(COLOUR_FLAG_256, 0x1000000);
    assert_eq!(COLOUR_FLAG_RGB, 0x2000000);
    assert_ne!(COLOUR_FLAG_256, COLOUR_FLAG_RGB);
    assert_eq!(COLOUR_FLAG_256 & COLOUR_FLAG_RGB, 0);
    assert!(COLOUR_FLAG_256 > 0xff);
    assert!(COLOUR_FLAG_RGB > COLOUR_FLAG_256);
}

// ---------------------------------------------------------------------------
// tostring
// ---------------------------------------------------------------------------

#[test]
fn tostring_renders_none_for_minus_one() {
    assert_eq!(tostring(-1), "none");
    // -1 is the only sentinel that renders as "none"
    assert_ne!(tostring(0), "none");
}

#[test]
fn tostring_rgb_is_lowercase_zero_padded_hex() {
    assert_eq!(tostring(COLOUR_FLAG_RGB), "#000000");
    assert_eq!(tostring(0xffffff | COLOUR_FLAG_RGB), "#ffffff");
    assert_eq!(tostring(0x0a1b2c | COLOUR_FLAG_RGB), "#0a1b2c");
    assert_eq!(tostring(0x010203 | COLOUR_FLAG_RGB), "#010203");
    // single-digit components are padded
    assert_eq!(tostring(0x00000f | COLOUR_FLAG_RGB), "#00000f");
}

#[test]
fn tostring_256_masks_to_low_byte() {
    assert_eq!(tostring(COLOUR_FLAG_256), "colour0");
    assert_eq!(tostring(255 | COLOUR_FLAG_256), "colour255");
    // high bits are masked: 300 & 0xff == 44
    assert_eq!(tostring(300 | COLOUR_FLAG_256), "colour44");
}

#[test]
fn tostring_basic_names_are_stable() {
    for (c, name) in [
        (0, "black"),
        (7, "white"),
        (8, "default"),
        (90, "brightblack"),
        (97, "brightwhite"),
    ] {
        assert_eq!(tostring(c), name);
    }
    // outside basic range without flags is "invalid"
    assert_eq!(tostring(10), "invalid");
    assert_eq!(tostring(100), "invalid");
}

// ---------------------------------------------------------------------------
// fromstring
// ---------------------------------------------------------------------------

#[test]
fn fromstring_hash_requires_seven_chars_and_hex_digits() {
    assert_eq!(fromstring(c"#000000"), COLOUR_FLAG_RGB);
    assert_eq!(fromstring(c"#ffffff"), 0xffffff | COLOUR_FLAG_RGB);
    assert_eq!(fromstring(c"#FF00FF"), 0xff00ff | COLOUR_FLAG_RGB);
    // too short / too long / non-hex all fail
    assert_eq!(fromstring(c"#12345"), -1);
    assert_eq!(fromstring(c"#1234567"), -1);
    assert_eq!(fromstring(c"#12345g"), -1);
    assert_eq!(fromstring(c"#zzzzzz"), -1);
    assert_eq!(fromstring(c"123456"), -1);
    assert_eq!(fromstring(c""), -1);
}

#[test]
fn fromstring_colour_numbers_accept_0_through_255_only() {
    assert_eq!(fromstring(c"colour0"), COLOUR_FLAG_256);
    assert_eq!(fromstring(c"colour255"), 255 | COLOUR_FLAG_256);
    assert_eq!(fromstring(c"color0"), COLOUR_FLAG_256);
    assert_eq!(fromstring(c"COLOR255"), 255 | COLOUR_FLAG_256);
    assert_eq!(fromstring(c"Colour128"), 128 | COLOUR_FLAG_256);
    assert_eq!(fromstring(c"colour256"), -1);
    assert_eq!(fromstring(c"colour-1"), -1);
    assert_eq!(fromstring(c"colour"), -1);
    assert_eq!(fromstring(c"colourx"), -1);
}

#[test]
fn fromstring_basic_names_and_numbers_are_case_insensitive() {
    assert_eq!(fromstring(c"black"), 0);
    assert_eq!(fromstring(c"BLACK"), 0);
    assert_eq!(fromstring(c"0"), 0);
    assert_eq!(fromstring(c"red"), 1);
    assert_eq!(fromstring(c"RED"), 1);
    assert_eq!(fromstring(c"1"), 1);
    assert_eq!(fromstring(c"brightwhite"), 97);
    assert_eq!(fromstring(c"BRIGHTWHITE"), 97);
    assert_eq!(fromstring(c"97"), 97);
    assert_eq!(fromstring(c"default"), 8);
    assert_eq!(fromstring(c"DEFAULT"), 8);
    assert_eq!(fromstring(c"terminal"), 9);
    assert_eq!(fromstring(c"nosuch"), -1);
}

#[test]
fn byname_grey_scales_percentage_with_rounding() {
    assert_eq!(byname(c"grey"), 0xbebebe | COLOUR_FLAG_RGB);
    assert_eq!(byname(c"gray"), 0xbebebe | COLOUR_FLAG_RGB);
    assert_eq!(byname(c"GREY"), 0xbebebe | COLOUR_FLAG_RGB);
    assert_eq!(byname(c"grey0"), COLOUR_FLAG_RGB);
    assert_eq!(byname(c"grey100"), 0xffffff | COLOUR_FLAG_RGB);
    // 2.55*50 rounds to 0x7f with the binary representation tmux sees
    assert_eq!(byname(c"grey50"), 0x7f7f7f | COLOUR_FLAG_RGB);
    assert_eq!(byname(c"gray50"), 0x7f7f7f | COLOUR_FLAG_RGB);
    assert_eq!(byname(c"grey101"), -1);
    assert_eq!(byname(c"greyx"), -1);
}

#[test]
fn parse_x11_accepts_all_documented_spellings() {
    // rgb:xx/xx/xx and #rrggbb
    assert_eq!(parse_x11(c"rgb:11/22/33"), 0x112233 | COLOUR_FLAG_RGB);
    assert_eq!(parse_x11(c"#aabbcc"), 0xaabbcc | COLOUR_FLAG_RGB);
    // decimal triple
    assert_eq!(parse_x11(c"1,2,3"), 0x010203 | COLOUR_FLAG_RGB);
    // 4-digit expanded form (high byte kept)
    assert_eq!(parse_x11(c"rgb:1111/2222/3333"), 0x112233 | COLOUR_FLAG_RGB);
    assert_eq!(parse_x11(c"#111122223333"), 0x112233 | COLOUR_FLAG_RGB);
    // cmyk / cmy
    assert_eq!(parse_x11(c"cmyk:0/0/0/0"), 0xffffff | COLOUR_FLAG_RGB);
    assert_eq!(parse_x11(c"cmyk:1/1/1/1"), COLOUR_FLAG_RGB);
    assert_eq!(parse_x11(c"cmy:0/0/0"), 0xffffff | COLOUR_FLAG_RGB);
    // out-of-range cmyk rejected
    assert_eq!(parse_x11(c"cmyk:2/0/0/0"), -1);
    // named colour after trimming
    assert_eq!(parse_x11(c"  AliceBlue  "), 0xf0f8ff | COLOUR_FLAG_RGB);
    assert_eq!(parse_x11(c"nosuch"), -1);
    assert_eq!(parse_x11(c""), -1);
}

//! Extra edge coverage for [`crate::style`] and [`crate::style`].
//! Complements `style::tests` (99% line coverage) by pinning the few
//! branches that the in-module suite does not force: width-percentage length
//! guard, control-range tostring name reuse, case-insensitive prefixes, and
//! attribute alias / delimiter handling.

use crate::style::{GRID_ATTR_BRIGHT, GRID_ATTR_DIM, attributes_fromstring, attributes_tostring};
use crate::style::{
    GRID_ATTR_NOATTR, STYLE_ALIGN_CENTRE, STYLE_LIST_ON, STYLE_LIST_RIGHT_MARKER,
    STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_RIGHT, style_parse, style_tostring,
};
use ::core::ffi::{CStr, c_char};

fn blank_style() -> Box<crate::types::style> {
    let mut sy = Box::new(crate::types::style::default());
    crate::style::style_set(&mut sy, &crate::grid::grid_default_cell);
    sy
}

fn base_cell() -> crate::types::grid_cell {
    let mut gc = unsafe { crate::grid::grid_default_cell };
    gc.fg = 1;
    gc.bg = 2;
    gc.us = 3;
    gc.attr = 0x10;
    gc
}

fn parse(s: &CStr) -> (Box<crate::types::style>, ::core::ffi::c_int) {
    let mut sy = blank_style();
    let gc = base_cell();
    let rc = unsafe { style_parse(&mut sy, &gc, s.to_bytes()) };
    (sy, rc)
}

fn parsed(s: &CStr) -> Box<crate::types::style> {
    let (sy, rc) = parse(s);
    assert_eq!(rc, 0, "{s:?} did not parse");
    sy
}

fn tostring(sy: &crate::types::style) -> String {
    unsafe { style_tostring(sy).to_string_lossy().into_owned() }
}

fn range_string(sy: &crate::types::style) -> String {
    unsafe { crate::tests::test_fixtures::seen(&raw const sy.range_string as *const c_char) }
}

fn attr_tostring(attr: ::core::ffi::c_int) -> String {
    attributes_tostring(attr).to_string_lossy().into_owned()
}

fn attr_fromstring(s: &CStr) -> ::core::ffi::c_int {
    attributes_fromstring(s)
}

// ---------------------------------------------------------------------------
// attributes edges
// ---------------------------------------------------------------------------

#[test]
fn attributes_bold_is_alias_for_bright_and_case_insensitive() {
    assert_eq!(attr_fromstring(c"bold"), GRID_ATTR_BRIGHT);
    assert_eq!(attr_fromstring(c"BOLD"), GRID_ATTR_BRIGHT);
    assert_eq!(attr_fromstring(c"Bold"), GRID_ATTR_BRIGHT);
    assert_eq!(attr_fromstring(c"BRIGHT"), GRID_ATTR_BRIGHT);
    // combined with other attrs across delimiters
    assert_eq!(
        attr_fromstring(c"bold,dim|italics"),
        GRID_ATTR_BRIGHT | GRID_ATTR_DIM | 0x40
    );
}

#[test]
fn attributes_noattr_prints_but_does_not_parse() {
    // tostring prints noattr when that bit is set
    assert_eq!(attr_tostring(GRID_ATTR_NOATTR), "noattr");
    assert_eq!(
        attr_tostring(GRID_ATTR_BRIGHT | GRID_ATTR_NOATTR),
        "bright,noattr"
    );
    // fromstring rejects noattr — it is not in PARSED
    assert_eq!(attr_fromstring(c"noattr"), -1);
    assert_eq!(attr_fromstring(c"NOATTR"), -1);
    assert_eq!(attr_fromstring(c"bright,noattr"), -1);
}

#[test]
fn attributes_double_underscore_variants_round_trip() {
    for (name, bit) in [
        ("double-underscore", 0x200),
        ("curly-underscore", 0x400),
        ("dotted-underscore", 0x800),
        ("dashed-underscore", 0x1000),
        ("overline", 0x2000),
    ] {
        let cs = std::ffi::CString::new(name).unwrap();
        assert_eq!(attr_fromstring(&cs), bit, "{name}");
        let upper = std::ffi::CString::new(name.to_uppercase()).unwrap();
        assert_eq!(attr_fromstring(&upper), bit, "{name} upper");
        assert_eq!(attr_tostring(bit), name);
    }
}

// ---------------------------------------------------------------------------
// style width percentage length guard
// ---------------------------------------------------------------------------

#[test]
fn style_width_percentage_requires_more_than_seven_chars_in_word() {
    // "width=1%" is 8 bytes -> w.len()>7 true, so percentage branch taken
    let sy = parsed(c"width=1%");
    assert_eq!(sy.width, 1);
    assert_eq!(sy.width_percentage, 1);
    assert_eq!(tostring(&sy), "width=1%");

    // "width=%" is 7 bytes -> guard false, falls to non-percentage branch
    // style_number("%", ...) fails -> refused and style unchanged
    let (sy, rc) = parse(c"width=%");
    assert_eq!(rc, -1);
    assert_eq!(tostring(&sy), "default");

    // boundary: "width=0%" also 8 -> valid
    let sy = parsed(c"width=0%");
    assert_eq!((sy.width, sy.width_percentage), (0, 1));

    // "width=100%" is 10 bytes, 100 is within 0..100
    let sy = parsed(c"width=100%");
    assert_eq!((sy.width, sy.width_percentage), (100, 1));

    // 101% exceeds upper bound -> refused
    let (_sy, rc) = parse(c"width=101%");
    assert_eq!(rc, -1);

    // without % the same digits are a plain count
    let sy = parsed(c"width=101");
    assert_eq!((sy.width, sy.width_percentage), (101, 0));
}

#[test]
fn style_width_and_pad_are_case_insensitive_prefixes() {
    // style_after uses eq_ignore_ascii_case
    let sy = parsed(c"WIDTH=5");
    assert_eq!(sy.width, 5);
    let sy = parsed(c"Width=6");
    assert_eq!(sy.width, 6);
    let sy = parsed(c"PAD=7");
    assert_eq!(sy.pad, 7);
    let sy = parsed(c"Pad=8");
    assert_eq!(sy.pad, 8);
    // percentage with upper case prefix
    let sy = parsed(c"WIDTH=20%");
    assert_eq!((sy.width, sy.width_percentage), (20, 1));
}

// ---------------------------------------------------------------------------
// style range and list interactions
// ---------------------------------------------------------------------------

#[test]
fn style_range_case_insensitive_and_control_boundary() {
    assert_eq!(parsed(c"RANGE=LEFT").range_type, STYLE_RANGE_LEFT);
    assert_eq!(parsed(c"Range=Right").range_type, STYLE_RANGE_RIGHT);
    assert_eq!(parsed(c"range=CONTROL|0").range_type, STYLE_RANGE_CONTROL);
    assert_eq!(parsed(c"range=control|9").range_argument, 9);
    // control 10 is out of 0..9 -> refused
    let (_sy, rc) = parse(c"range=control|10");
    assert_eq!(rc, -1);
    let (_sy, rc) = parse(c"range=Control|0");
    assert_eq!(rc, 0);
}

#[test]
fn style_range_user_truncation_and_tostring_round_trip() {
    // exactly RANGE_STRING_SIZE-1 = 15 chars fits
    let sy = parsed(c"range=user|123456789012345");
    assert_eq!(range_string(&sy), "123456789012345");
    assert_eq!(tostring(&sy), "range=user|123456789012345");
    // 16 chars truncated to 15, matching style_set_range_string strlcpy
    let sy = parsed(c"range=user|1234567890123456");
    assert_eq!(range_string(&sy), "123456789012345");
    // user without argument is refused
    let (_sy, rc) = parse(c"range=user");
    assert_eq!(rc, -1);
}

#[test]
fn style_tostring_control_range_reuses_previous_tmp_name() {
    // style_tostring keeps `tmp` across blocks; a control range has no arm
    // and leaves whatever the previous block wrote there.
    // without preceding list, tmp is empty -> "range="
    let sy = parsed(c"range=control|3");
    assert_eq!(sy.range_type, STYLE_RANGE_CONTROL);
    assert_eq!(tostring(&sy), "range=");
    // with list=on before it, tmp is "on" -> "range=on" is not printed, but list+range
    let sy = parsed(c"list=on,range=control|3");
    assert_eq!(tostring(&sy), "list=on,range=on");
    // list=focus gives "range=focus"
    let sy = parsed(c"list=focus,range=control|3");
    assert_eq!(tostring(&sy), "list=focus,range=focus");
}

#[test]
fn style_parse_mixed_case_and_multiple_words() {
    // fg/bg/us and align/list prefixes are case-insensitive
    let sy = parsed(c"FG=red,BG=blue,US=green");
    assert_eq!((sy.gc.fg, sy.gc.bg, sy.gc.us), (1, 4, 2));
    assert_eq!(parsed(c"ALIGN=CENTRE").align, STYLE_ALIGN_CENTRE);
    assert_eq!(parsed(c"List=On").list, STYLE_LIST_ON);
    assert_eq!(parsed(c"LIST=RIGHT-MARKER").list, STYLE_LIST_RIGHT_MARKER);
    // delimiters mix and case-insensitive "default" word
    let sy = parsed(c"DEFAULT");
    assert_eq!((sy.gc.fg, sy.gc.bg), (1, 2));
    // duplicate attribute words are idempotent, mixed case
    let sy = parsed(c"Bright,BRIGHT");
    assert_eq!(sy.gc.attr, 0x1);
}

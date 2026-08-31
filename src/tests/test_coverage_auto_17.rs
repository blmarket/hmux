//! Extra coverage for [`crate::text`] – complements the in-module suite (88%)
//! by exercising `utf8_isvalid`, `utf8_sanitize`, `utf8_cstrwidth`,
//! `utf8_padcstr`/`utf8_rpadcstr` and the width cache populated by
//! `utf8_update_width_cache` and the `codepoint-widths` option.

use crate::ffi::free;
use crate::options::{options_array_set, options_codepoint_widths, options_get_ptr};
use crate::tests::test_fixtures::globals;
use crate::tmux::global_options;
use crate::text::{
    utf8_cstrwidth, utf8_isvalid, utf8_padcstr, utf8_rpadcstr, utf8_sanitize,
    utf8_update_width_cache,
};
use ::core::ffi::{CStr, c_char, c_void};

unsafe fn taken(p: *mut c_char) -> Vec<u8> {
    unsafe {
        let v = CStr::from_ptr(p).to_bytes().to_vec();
        free(p as *mut c_void);
        v
    }
}

fn c(s: &CStr) -> *const c_char {
    s.as_ptr()
}

// ---------------------------------------------------------------------------
// utf8_isvalid
// ---------------------------------------------------------------------------

#[test]
fn utf8_isvalid_accepts_printable_ascii_and_valid_multibyte() {
    let _g = globals();
    unsafe {
        utf8_update_width_cache(options_codepoint_widths(global_options));
        assert_eq!(utf8_isvalid(c"".as_ptr()), 1);
        assert_eq!(utf8_isvalid(c"abc".as_ptr()), 1);
        assert_eq!(utf8_isvalid(c" ".as_ptr()), 1, "space 0x20 is printable");
        assert_eq!(utf8_isvalid(c"~".as_ptr()), 1, "tilde 0x7E is printable");
        assert_eq!(utf8_isvalid(c"a b~c".as_ptr()), 1);
        // valid 2-, 3- and 4-byte sequences are accepted
        assert_eq!(utf8_isvalid(c"a\xc3\xa9b".as_ptr()), 1, "é U+00E9");
        assert_eq!(utf8_isvalid(c"\xe2\x82\xac".as_ptr()), 1, "€ U+20AC");
        assert_eq!(utf8_isvalid(c"\xf0\x9f\x98\x80".as_ptr()), 1, "😀 U+1F600");
        // string that is only valid multibyte
        assert_eq!(
            utf8_isvalid(c"\xc3\xa9\xe2\x82\xac\xf0\x9f\x98\x80".as_ptr()),
            1
        );
    }
}

#[test]
fn utf8_isvalid_rejects_controls_and_broken_utf8() {
    let _g = globals();
    unsafe {
        utf8_update_width_cache(options_codepoint_widths(global_options));
        // DEL and C0 controls are not printable
        assert_eq!(utf8_isvalid(c"a\x7f".as_ptr()), 0, "DEL 0x7F invalid");
        assert_eq!(utf8_isvalid(c"a\x01b".as_ptr()), 0, "SOH 0x01 invalid");
        assert_eq!(utf8_isvalid(c"\x1f".as_ptr()), 0, "0x1F invalid");
        assert_eq!(utf8_isvalid(c"\t".as_ptr()), 0, "tab invalid");
        assert_eq!(utf8_isvalid(c"\n".as_ptr()), 0, "newline invalid");
        // lone continuation / illegal starters
        assert_eq!(utf8_isvalid(c"\x80".as_ptr()), 0);
        assert_eq!(utf8_isvalid(c"\xff".as_ptr()), 0);
        assert_eq!(utf8_isvalid(c"\xfe".as_ptr()), 0);
        // truncated sequences – leading byte without enough continuations
        assert_eq!(utf8_isvalid(c"\xc3".as_ptr()), 0);
        assert_eq!(utf8_isvalid(c"\xe2\x82".as_ptr()), 0);
        assert_eq!(utf8_isvalid(c"\xf0\x9f\x98".as_ptr()), 0);
        // bad continuation byte
        assert_eq!(utf8_isvalid(c"\xc3\x28".as_ptr()), 0);
        assert_eq!(utf8_isvalid(c"\xe2\x28\xa1".as_ptr()), 0);
        // overlong / surrogate range that utf8_take refuses
        assert_eq!(utf8_isvalid(c"\xc0\x80".as_ptr()), 0);
        assert_eq!(utf8_isvalid(c"\xed\xa0\x80".as_ptr()), 0);
    }
}

// ---------------------------------------------------------------------------
// utf8_sanitize
// ---------------------------------------------------------------------------

#[test]
fn utf8_sanitize_keeps_printable_ascii_replaces_controls() {
    let _g = globals();
    unsafe {
        utf8_update_width_cache(options_codepoint_widths(global_options));
        assert_eq!(utf8_sanitize(c"abc".as_ptr()).as_bytes(), b"abc");
        assert_eq!(utf8_sanitize(c"".as_ptr()).as_bytes(), b"");
        assert_eq!(utf8_sanitize(c" ".as_ptr()).as_bytes(), b" ");
        assert_eq!(utf8_sanitize(c"~".as_ptr()).as_bytes(), b"~");
        // controls replaced: SOH, DEL, tab, newline
        assert_eq!(utf8_sanitize(c"a\x01b".as_ptr()).as_bytes(), b"a_b");
        assert_eq!(utf8_sanitize(c"a\x7f".as_ptr()).as_bytes(), b"a_");
        assert_eq!(utf8_sanitize(c"a\x7fb".as_ptr()).as_bytes(), b"a_b");
        assert_eq!(utf8_sanitize(c"a\tb".as_ptr()).as_bytes(), b"a_b");
        assert_eq!(utf8_sanitize(c"a\nb".as_ptr()).as_bytes(), b"a_b");
        assert_eq!(utf8_sanitize(c"a\x01b\x7f".as_ptr()).as_bytes(), b"a_b_");
        // lone invalid byte replaced
        assert_eq!(utf8_sanitize(c"\xff".as_ptr()).as_bytes(), b"_");
        assert_eq!(utf8_sanitize(c"\x80".as_ptr()).as_bytes(), b"_");
        // truncated leading byte replaced
        assert_eq!(utf8_sanitize(c"\xc3".as_ptr()).as_bytes(), b"_");
        assert_eq!(utf8_sanitize(c"a\xc3".as_ptr()).as_bytes(), b"a_");
    }
}

#[test]
fn utf8_sanitize_replaces_each_character_by_its_display_width() {
    let _g = globals();
    unsafe {
        utf8_update_width_cache(options_codepoint_widths(global_options));
        // width 1 multibyte -> one underscore
        assert_eq!(utf8_sanitize(c"a\xc3\xa9".as_ptr()).as_bytes(), b"a_");
        assert_eq!(utf8_sanitize(c"\xc3\xa9".as_ptr()).as_bytes(), b"_");
        assert_eq!(
            utf8_sanitize(c"\xe2\x82\xac".as_ptr()).as_bytes(),
            b"_",
            "€ width 1"
        );
        // width 2 emoji -> two underscores
        assert_eq!(
            utf8_sanitize(c"\xf0\x9f\x98\x80".as_ptr()).as_bytes(),
            b"__",
            "😀 width 2"
        );
        assert_eq!(
            utf8_sanitize(c"a\xf0\x9f\x98\x80b".as_ptr()).as_bytes(),
            b"a__b"
        );
        // mixed widths - length equals sum of widths / 1 per ASCII
        assert_eq!(
            utf8_sanitize(c"a\xc3\xa9\xf0\x9f\x98\x80".as_ptr()).as_bytes(),
            b"a___",
            "1 + 1 + 2"
        );
        // empty stays empty
        assert_eq!(utf8_sanitize(c"".as_ptr()).as_bytes(), b"");
    }
}

// ---------------------------------------------------------------------------
// utf8_cstrwidth / padding - width cache observable behaviour
// ---------------------------------------------------------------------------

#[test]
fn utf8_cstrwidth_counts_display_width_skipping_controls() {
    let _g = globals();
    unsafe {
        utf8_update_width_cache(options_codepoint_widths(global_options));
        assert_eq!(utf8_cstrwidth(c"".as_ptr()), 0);
        assert_eq!(utf8_cstrwidth(c"abc".as_ptr()), 3);
        assert_eq!(utf8_cstrwidth(c" ".as_ptr()), 1);
        assert_eq!(utf8_cstrwidth(c"~".as_ptr()), 1);
        // multibyte widths
        assert_eq!(utf8_cstrwidth(c"a\xc3\xa9".as_ptr()), 2, "a + é");
        assert_eq!(
            utf8_cstrwidth(c"a\xc3\xa9\xf0\x9f\x98\x80".as_ptr()),
            4,
            "a(1)+é(1)+😀(2)"
        );
        // controls contribute 0 width
        assert_eq!(utf8_cstrwidth(c"a\x01b".as_ptr()), 2);
        assert_eq!(utf8_cstrwidth(c"a\x7f".as_ptr()), 1);
        assert_eq!(utf8_cstrwidth(c"a\x01b\x7f".as_ptr()), 2);
        // truncated leading byte contributes 0
        assert_eq!(utf8_cstrwidth(c"\xc3".as_ptr()), 0);
        assert_eq!(utf8_cstrwidth(c"\xff".as_ptr()), 0);
    }
}

#[test]
fn utf8_padcstr_uses_display_width_for_padding() {
    let _g = globals();
    unsafe { utf8_update_width_cache(options_codepoint_widths(global_options)) };
    // ASCII: width==len, padding adds spaces on the right / left
    assert_eq!(utf8_padcstr(c"ab", 4).into_bytes(), b"ab  ");
    assert_eq!(utf8_rpadcstr(c"ab", 4).into_bytes(), b"  ab");
    // already wide enough -> no padding
    assert_eq!(utf8_padcstr(c"ab", 2).into_bytes(), b"ab");
    assert_eq!(utf8_padcstr(c"ab", 1).into_bytes(), b"ab");
    assert_eq!(utf8_rpadcstr(c"ab", 2).into_bytes(), b"ab");
    // wide char counts as 2: "a😀" width 3, pad to 5 adds two spaces
    assert_eq!(
        utf8_padcstr(c"a\xf0\x9f\x98\x80", 5).into_bytes(),
        b"a\xf0\x9f\x98\x80  "
    );
    assert_eq!(
        utf8_rpadcstr(c"a\xf0\x9f\x98\x80", 5).into_bytes(),
        b"  a\xf0\x9f\x98\x80"
    );
    // controls not counted: "a\x01b" width 2, pad to 4 adds two spaces
    assert_eq!(utf8_padcstr(c"a\x01b", 4).into_bytes(), b"a\x01b  ");
    // empty string padded
    assert_eq!(utf8_padcstr(c"", 3).into_bytes(), b"   ");
    assert_eq!(utf8_rpadcstr(c"", 2).into_bytes(), b"  ");
}

// ---------------------------------------------------------------------------
// width cache - observable via cstrwidth / sanitize after update
// ---------------------------------------------------------------------------

#[test]
fn width_cache_defaults_are_applied_after_update() {
    let _g = globals();
    unsafe {
        // ensure a clean rebuild from defaults
        let o = options_get_ptr(global_options, c"codepoint-widths".as_ptr());
        // clear any previous entries
        options_array_set(o, 0, c"".as_ptr(), 0, &mut None);
        utf8_update_width_cache(options_codepoint_widths(global_options));

        // 0x261D (☝) is in UTF8_DEFAULT_WIDTHS with width 2
        assert_eq!(
            utf8_cstrwidth(c"\xe2\x98\x9d".as_ptr()),
            2,
            "U+261D default 2"
        );
        // 0x1F1E6 (🇦 regional indicator) is default 1
        assert_eq!(
            utf8_cstrwidth(c"\xf0\x9f\x87\xa6".as_ptr()),
            1,
            "U+1F1E6 default 1"
        );
        // 0x1FAF8 is also in the table as 2
        assert_eq!(
            utf8_cstrwidth(c"\xf0\x9f\xab\xb8".as_ptr()),
            2,
            "U+1FAF8 default 2"
        );

        // sanitize reflects the same widths
        assert_eq!(utf8_sanitize(c"\xe2\x98\x9d".as_ptr()).as_bytes(), b"__");
        assert_eq!(utf8_sanitize(c"\xf0\x9f\x87\xa6".as_ptr()).as_bytes(), b"_");

        options_array_set(o, 0, c"".as_ptr(), 0, &mut None);
        utf8_update_width_cache(options_codepoint_widths(global_options));
    }
}

#[test]
fn width_cache_codepoint_widths_option_overrides_and_range() {
    let _g = globals();
    unsafe {
        let o = options_get_ptr(global_options, c"codepoint-widths".as_ptr());
        // single multibyte codepoint: é (U+E9) normally width 1, force to 2
        options_array_set(o, 0, c"U+E9=2".as_ptr(), 0, &mut None);
        utf8_update_width_cache(options_codepoint_widths(global_options));
        assert_eq!(utf8_cstrwidth(c"\xc3\xa9".as_ptr()), 2);
        assert_eq!(utf8_sanitize(c"\xc3\xa9".as_ptr()).as_bytes(), b"__");

        // clear and check it returns to 1
        options_array_set(o, 0, c"".as_ptr(), 0, &mut None);
        utf8_update_width_cache(options_codepoint_widths(global_options));
        assert_eq!(utf8_cstrwidth(c"\xc3\xa9".as_ptr()), 1);

        // range: U+E9..U+EB (é, ê, ë) to 2
        options_array_set(o, 0, c"U+E9-U+EB=2".as_ptr(), 0, &mut None);
        utf8_update_width_cache(options_codepoint_widths(global_options));
        assert_eq!(utf8_cstrwidth(c"\xc3\xa9".as_ptr()), 2, "U+E9");
        assert_eq!(utf8_cstrwidth(c"\xc3\xaa".as_ptr()), 2, "U+EA");
        assert_eq!(utf8_cstrwidth(c"\xc3\xab".as_ptr()), 2, "U+EB");
        assert_eq!(
            utf8_cstrwidth(c"\xc3\xac".as_ptr()),
            1,
            "outside range stays 1"
        );

        // another single: € (U+20AC) forced to 0
        options_array_set(o, 0, c"U+20AC=0".as_ptr(), 0, &mut None);
        utf8_update_width_cache(options_codepoint_widths(global_options));
        assert_eq!(utf8_cstrwidth(c"\xe2\x82\xac".as_ptr()), 0);
        assert_eq!(utf8_sanitize(c"\xe2\x82\xac".as_ptr()).as_bytes(), b"");

        // invalid width 3 is rejected - width should be default again
        options_array_set(o, 0, c"U+E9=3".as_ptr(), 0, &mut None);
        utf8_update_width_cache(options_codepoint_widths(global_options));
        assert_eq!(utf8_cstrwidth(c"\xc3\xa9".as_ptr()), 1);

        // cleanup
        options_array_set(o, 0, c"".as_ptr(), 0, &mut None);
        utf8_update_width_cache(options_codepoint_widths(global_options));
        assert_eq!(utf8_cstrwidth(c"\xc3\xa9".as_ptr()), 1);
        assert_eq!(utf8_cstrwidth(c"\xe2\x82\xac".as_ptr()), 1);
        assert_eq!(utf8_cstrwidth(c"\xc3\xaa".as_ptr()), 1);
    }
}

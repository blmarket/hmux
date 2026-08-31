use super::*;
use ::core::ffi::CStr;
use ::std::sync::Mutex;

const UNKNOWN: key_code = KEYC_UNKNOWN as key_code;

/// `key_string_lookup_key` answers out of one static buffer, so only one
/// test may hold its answer at a time.
static OUT: Mutex<()> = Mutex::new(());

fn lookup(string: &CStr) -> key_code {
    // A character of more than three bytes is interned into the process-wide
    // UTF-8 table, so this changes shared state and takes the guard.
    let _globals = crate::tests::test_fixtures::globals();
    unsafe { key_string_lookup_string(string.as_ptr()) }
}

fn name_of(key: key_code, with_flags: ::core::ffi::c_int) -> Vec<u8> {
    OUT.clear_poison();
    let _guard = OUT.lock().expect("just cleared any poison");
    let _globals = crate::tests::test_fixtures::globals();
    unsafe {
        CStr::from_ptr(key_string_lookup_key(key, with_flags))
            .to_bytes()
            .to_vec()
    }
}

fn name(key: key_code) -> Vec<u8> {
    name_of(key, 0 as ::core::ffi::c_int)
}

fn name_with_flags(key: key_code) -> Vec<u8> {
    name_of(key, 1 as ::core::ffi::c_int)
}

#[test]
fn none_and_any_are_matched_whatever_their_case() {
    assert_eq!(lookup(c"None"), KEYC_NONE as key_code);
    assert_eq!(lookup(c"none"), KEYC_NONE as key_code);
    assert_eq!(lookup(c"Any"), KEYC_ANY as key_code);
    assert_eq!(lookup(c"ANY"), KEYC_ANY as key_code);
}

#[test]
fn a_hexadecimal_key_is_the_character_it_numbers() {
    assert_eq!(lookup(c"0x1"), 1);
    assert_eq!(lookup(c"0x1f"), 0x1f);
    assert_eq!(lookup(c"0x20"), 0x41000020);
    assert_eq!(lookup(c"0x41"), 0x41000041);
    assert_eq!(lookup(c"0x7f"), 0x4100007f);
}

#[test]
fn a_hexadecimal_key_the_locale_cannot_encode_is_unknown() {
    assert_eq!(lookup(c"0xe9"), UNKNOWN);
    assert_eq!(lookup(c"0x"), UNKNOWN);
    assert_eq!(lookup(c"0xzz"), UNKNOWN);
}

#[test]
fn a_caret_makes_a_control_key_of_the_character_after_it() {
    assert_eq!(lookup(c"^A"), 'a' as key_code | KEYC_CTRL);
    assert_eq!(lookup(c"^a"), 'a' as key_code | KEYC_CTRL);
    assert_eq!(lookup(c"^?"), '?' as key_code | KEYC_CTRL);
    assert_eq!(lookup(c"^"), '^' as key_code);
    assert_eq!(lookup(c"^AB"), UNKNOWN);
    assert_eq!(lookup(c"^F1"), KEYC_F1 as key_code | KEYC_CTRL);
}

#[test]
fn the_modifier_prefixes_stack_in_any_case() {
    assert_eq!(lookup(c"C-a"), 'a' as key_code | KEYC_CTRL);
    assert_eq!(lookup(c"c-a"), 'a' as key_code | KEYC_CTRL);
    assert_eq!(lookup(c"M-a"), 'a' as key_code | KEYC_META);
    assert_eq!(lookup(c"m-a"), 'a' as key_code | KEYC_META);
    assert_eq!(lookup(c"S-a"), 'a' as key_code | KEYC_SHIFT);
    assert_eq!(lookup(c"s-a"), 'a' as key_code | KEYC_SHIFT);
    assert_eq!(
        lookup(c"C-M-S-a"),
        'a' as key_code | KEYC_CTRL | KEYC_META | KEYC_SHIFT
    );
    assert_eq!(lookup(c"X-a"), UNKNOWN);
    assert_eq!(lookup(c"C-"), UNKNOWN);
}

#[test]
fn a_lone_printable_character_is_its_own_key() {
    assert_eq!(lookup(c"a"), 'a' as key_code);
    assert_eq!(lookup(c"~"), '~' as key_code);
    assert_eq!(lookup(c""), UNKNOWN);
    assert_eq!(lookup(c"\x01"), UNKNOWN);
    assert_eq!(lookup(c"ab"), UNKNOWN);
}

#[test]
fn a_utf8_character_becomes_a_wide_key() {
    assert_eq!(lookup(c"\xc3\xa9"), 0x4200a9c3);
    assert_eq!(lookup(c"\xe2\x82\xac"), 0x43ac82e2);
    // A character of more than three bytes does not fit in the key, so it is
    // interned and the key carries the index it was given. That index counts
    // up across the process, so what is fixed here is everything above it:
    // the four-byte size and the two-column width.
    let interned = lookup(c"\xf0\x9f\x98\x80");
    assert_eq!(interned & !0x00ff_ffff, 0x6400_0000, "size and width bits");
    assert_eq!(
        lookup(c"\xf0\x9f\x98\x80"),
        interned,
        "the same character interns to the same key"
    );
    assert_eq!(lookup(c"\xc3"), UNKNOWN);
    assert_eq!(lookup(c"\xc3\x41"), UNKNOWN);
    assert_eq!(lookup(c"M-\xc3\xa9"), 0x4200a9c3 | KEYC_META);
}

#[test]
fn table_names_are_matched_without_case() {
    assert_eq!(lookup(c"F1"), 0x200000008);
    assert_eq!(lookup(c"f1"), 0x200000008);
    assert_eq!(lookup(c"Space"), ' ' as key_code);
    assert_eq!(lookup(c"Enter"), '\r' as key_code);
    assert_eq!(lookup(c"[NUL]"), 0);
    assert_eq!(lookup(c"NotAKey"), UNKNOWN);
}

#[test]
fn a_table_key_keeps_its_implied_meta_only_behind_a_meta_modifier() {
    assert_eq!(lookup(c"F1") & KEYC_IMPLIED_META, 0);
    assert_eq!(lookup(c"C-F1") & KEYC_IMPLIED_META, 0);
    assert_eq!(lookup(c"M-F1") & KEYC_IMPLIED_META, KEYC_IMPLIED_META);
    assert_eq!(lookup(c"M-F1"), 0x200000008 | KEYC_META | KEYC_IMPLIED_META);
}

#[test]
fn user_keys_are_numbered_up_to_the_limit() {
    assert_eq!(lookup(c"User0"), KEYC_USER as key_code);
    assert_eq!(lookup(c"User3"), KEYC_USER as key_code + 3);
    assert_eq!(
        lookup(c"User1000"),
        KEYC_USER as key_code + KEYC_NUSER as key_code
    );
    assert_eq!(lookup(c"User1001"), UNKNOWN);
    assert_eq!(lookup(c"user5"), UNKNOWN);
    assert_eq!(lookup(c"User"), UNKNOWN);
    assert_eq!(lookup(c"UserX"), UNKNOWN);
}

#[test]
fn the_keys_with_a_name_of_their_own_come_back_by_name() {
    for (key, want) in [
        (KEYC_NONE, b"None".as_slice()),
        (KEYC_UNKNOWN, b"Unknown"),
        (KEYC_ANY, b"Any"),
        (KEYC_FOCUS_IN, b"FocusIn"),
        (KEYC_FOCUS_OUT, b"FocusOut"),
        (KEYC_PASTE_START, b"PasteStart"),
        (KEYC_PASTE_END, b"PasteEnd"),
        (KEYC_REPORT_DARK_THEME, b"ReportDarkTheme"),
        (KEYC_REPORT_LIGHT_THEME, b"ReportLightTheme"),
        (KEYC_MOUSE, b"Mouse"),
        (KEYC_DRAGGING, b"Dragging"),
        (KEYC_MOUSEMOVE_PANE, b"MouseMovePane"),
        (KEYC_MOUSEMOVE_STATUS, b"MouseMoveStatus"),
        (KEYC_MOUSEMOVE_STATUS_LEFT, b"MouseMoveStatusLeft"),
        (KEYC_MOUSEMOVE_STATUS_RIGHT, b"MouseMoveStatusRight"),
        (KEYC_MOUSEMOVE_BORDER, b"MouseMoveBorder"),
    ] {
        assert_eq!(name(key as key_code), want, "{key:#x}");
    }
}

#[test]
fn modifiers_are_written_back_as_prefixes() {
    assert_eq!(name('a' as key_code), b"a");
    assert_eq!(name('a' as key_code | KEYC_CTRL), b"C-a");
    assert_eq!(name('a' as key_code | KEYC_META), b"M-a");
    assert_eq!(name('a' as key_code | KEYC_SHIFT), b"S-a");
    assert_eq!(
        name('a' as key_code | KEYC_CTRL | KEYC_META | KEYC_SHIFT),
        b"C-M-S-a"
    );
}

#[test]
fn a_literal_key_is_written_as_its_own_byte() {
    assert_eq!(name(KEYC_LITERAL | 0x41), b"A");
    assert_eq!(name(KEYC_LITERAL | 0x141), b"A");
    assert_eq!(name(KEYC_LITERAL | KEYC_CTRL | 0x41), b"A");
    assert_eq!(name(KEYC_LITERAL), b"");
    assert_eq!(name_with_flags(KEYC_LITERAL), b"[L]");
}

#[test]
fn a_user_key_is_numbered_and_truncated_to_seven_characters() {
    assert_eq!(name(KEYC_USER as key_code), b"User0");
    assert_eq!(name(KEYC_USER as key_code + 3), b"User3");
    assert_eq!(name(KEYC_USER as key_code + 999), b"User999");
    assert_eq!(name(KEYC_USER as key_code + 1000), b"User100");
}

#[test]
fn a_table_key_comes_back_as_its_table_name() {
    assert_eq!(name(0x200000008), b"F1");
    assert_eq!(name(' ' as key_code), b"Space");
    assert_eq!(name('\r' as key_code), b"Enter");
    assert_eq!(name(0), b"[NUL]");
    assert_eq!(name(1), b"[SOH]");
}

#[test]
fn a_printable_ascii_key_is_itself() {
    assert_eq!(name('a' as key_code), b"a");
    assert_eq!(name('!' as key_code), b"!");
    assert_eq!(name('~' as key_code), b"~");
    assert_eq!(name(0x7f), b"C-?");
}

#[test]
fn a_unicode_key_comes_back_as_its_character() {
    assert_eq!(name(0x41000041), b"A");
    assert_eq!(name(lookup(c"\xc3\xa9")), b"\xc3\xa9");
    assert_eq!(name(lookup(c"\xe2\x82\xac")), b"\xe2\x82\xac");
    assert_eq!(name(0xc8), b"");
}

#[test]
fn a_key_that_is_nothing_else_is_reported_as_invalid() {
    assert_eq!(name(0x200000999), b"Invalid#200000999");
    assert_eq!(name(0x200000999 | KEYC_SHIFT), b"Invalid#400200000999");
}

#[test]
fn the_flag_suffix_lists_the_flags_that_are_set() {
    assert_eq!(name(0x200000008 | KEYC_IMPLIED_META), b"F1");
    assert_eq!(name_with_flags(0x200000008 | KEYC_IMPLIED_META), b"F1[I]");
    assert_eq!(name_with_flags(0x200000008), b"F1");
    assert_eq!(
        name_with_flags(
            'a' as key_code | KEYC_KEYPAD | KEYC_CURSOR | KEYC_BUILD_MODIFIERS | KEYC_SENT
        ),
        b"a[KCBS]"
    );
    assert_eq!(
        name_with_flags('a' as key_code | KEYC_LITERAL | KEYC_KEYPAD),
        b"a[LK]"
    );
}

#[test]
fn a_string_is_only_a_key_when_it_holds_exactly_one_character() {
    let mut buf = [0 as ::core::ffi::c_char; 8];
    assert_eq!(unsafe { key_from_cstr(buf.as_mut_ptr()) }, None);
    for (i, &b) in b"ab\0".iter().enumerate() {
        buf[i] = b as ::core::ffi::c_char;
    }
    assert_eq!(unsafe { key_from_cstr(buf.as_mut_ptr()) }, None);
    buf[1] = 0;
    assert_eq!(unsafe { key_from_cstr(buf.as_mut_ptr()) }, Some(0x41000061));
}

#[test]
fn a_character_too_long_to_pack_is_not_a_key() {
    let mut ud = utf8_data::default();
    ud.data[0] = b'a';
    ud.have = 1;
    ud.size = 1;
    ud.width = 1;
    assert_eq!(unsafe { key_from_data(&raw mut ud) }, Some(0x41000061));
    ud.size = 33;
    assert_eq!(unsafe { key_from_data(&raw mut ud) }, None);
}

#[test]
fn every_table_name_survives_a_round_trip() {
    for i in 0..1314 {
        let entry = key_string_table[i];
        assert_eq!(
            lookup(entry.string) & KEYC_MASK_KEY,
            entry.key & KEYC_MASK_KEY
        );
    }
}

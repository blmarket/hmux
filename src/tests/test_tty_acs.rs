use super::*;
use crate::tests::test_fixtures::{Tty, globals, seen};
use ::core::ffi::CStr;

/// What `tty_acs_get` answers for `ch`, or `None` if it has no translation.
unsafe fn get(tty: *mut tty, ch: u8) -> Option<String> {
    unsafe { tty_acs_get(tty, ch).map(|s| seen(s.as_ptr())) }
}

/// What `tty_acs_reverse_get` answers for the whole of `s`.
fn reverse(s: &CStr) -> ::core::ffi::c_int {
    tty_acs_reverse_get(s.to_bytes())
}

#[test]
fn no_terminal_at_all_needs_no_acs() {
    assert_eq!(unsafe { tty_acs_needed(::core::ptr::null_mut::<tty>()) }, 0);
}

#[test]
fn a_terminal_without_the_u8_capability_follows_the_client_flag() {
    let _guard = globals();
    let mut t = Tty::new();
    assert_eq!(unsafe { tty_acs_needed(t.ptr()) }, 1);
    t.set_client_flags(CLIENT_UTF8 as u64);
    assert_eq!(unsafe { tty_acs_needed(t.ptr()) }, 0);
}

#[test]
fn a_u8_capability_of_zero_forces_acs_whatever_the_client_says() {
    let _guard = globals();
    let mut t = Tty::new();
    t.set_number(TTYC_U8, 0);
    t.set_client_flags(CLIENT_UTF8 as u64);
    assert_eq!(unsafe { tty_acs_needed(t.ptr()) }, 1);
}

#[test]
fn a_nonzero_u8_capability_leaves_the_client_flag_in_charge() {
    let _guard = globals();
    let mut t = Tty::new();
    t.set_number(TTYC_U8, 1);
    assert_eq!(unsafe { tty_acs_needed(t.ptr()) }, 1);
    t.set_client_flags(CLIENT_UTF8 as u64);
    assert_eq!(unsafe { tty_acs_needed(t.ptr()) }, 0);
}

#[test]
fn a_utf8_client_reads_the_table_and_unknown_keys_have_nothing() {
    let _guard = globals();
    let mut t = Tty::new();
    t.set_client_flags(CLIENT_UTF8 as u64);
    unsafe {
        assert_eq!(get(t.ptr(), b'q'), Some("\u{2500}".to_string()));
        assert_eq!(get(t.ptr(), b'x'), Some("\u{2502}".to_string()));
        assert_eq!(get(t.ptr(), b'+'), Some("\u{2192}".to_string()));
        assert_eq!(get(t.ptr(), b'~'), Some("\u{00b7}".to_string()));
        assert_eq!(get(t.ptr(), b'f'), Some("\u{00b0}".to_string()));
        assert_eq!(get(t.ptr(), b'*'), None);
        assert_eq!(get(t.ptr(), b'/'), None);
        assert_eq!(get(t.ptr(), 0), None);
        assert_eq!(get(t.ptr(), 255), None);
    }
}

#[test]
fn every_key_in_the_table_is_found_and_the_table_is_sorted() {
    let _guard = globals();
    let mut t = Tty::new();
    t.set_client_flags(CLIENT_UTF8 as u64);
    unsafe {
        let mut last = 0u8;
        for entry in &tty_acs_table {
            assert!(entry.key > last, "the table is not sorted");
            last = entry.key;
            assert_eq!(get(t.ptr(), entry.key), Some(seen(entry.string.as_ptr())));
        }
    }
}

#[test]
fn a_terminal_that_needs_acs_reads_its_own_translations() {
    let _guard = globals();
    let mut t = Tty::new();
    t.set_acs(b'q', "-");
    unsafe {
        assert_eq!(get(t.ptr(), b'q'), Some("-".to_string()));
        assert_eq!(get(t.ptr(), b'x'), None);
        assert_eq!(
            tty_acs_get(t.ptr(), b'q').map(CStr::as_ptr),
            Some(&raw const t.term().acs[b'q' as usize][0])
        );
    }
}

#[test]
fn reverse_lookup_only_answers_for_two_and_three_byte_strings() {
    {
        assert_eq!(reverse(c""), -1);
        assert_eq!(reverse(c"-"), -1);
        assert_eq!(reverse(c"abcd"), -1);
        assert_eq!(reverse(c"\u{00b7}"), b'~' as ::core::ffi::c_int);
        assert_eq!(reverse(c"\u{00b0}"), -1);
        assert_eq!(reverse(c"\u{2500}"), b'q' as ::core::ffi::c_int);
        assert_eq!(reverse(c"\u{2503}"), b'x' as ::core::ffi::c_int);
        assert_eq!(reverse(c"\u{256c}"), b'n' as ::core::ffi::c_int);
        assert_eq!(reverse(c"\u{2592}"), -1);
    }
}

#[test]
fn every_reverse_entry_is_found_and_both_tables_are_sorted() {
    {
        for table in [&tty_acs_reverse2[..], &tty_acs_reverse3[..]] {
            let mut last: &[u8] = b"";
            for entry in table {
                assert!(entry.string.to_bytes() > last, "the table is not sorted");
                last = entry.string.to_bytes();
                assert_eq!(reverse(entry.string), entry.key as ::core::ffi::c_int);
            }
        }
    }
}

#[test]
fn a_border_cell_carries_the_character_and_nothing_else() {
    let empty = border("");
    assert_eq!(empty.size, 0);
    assert_eq!(empty.width, 0);
    assert_eq!(empty.have, 0);
    assert_eq!(empty.data, [0; 32]);

    let one = border("\u{2551}");
    assert_eq!(one.size, 3);
    assert_eq!(one.width, 1);
    assert_eq!(one.have, 0);
    assert_eq!(&one.data[..4], b"\xe2\x95\x91\0");
    assert_eq!(&one.data[4..], &[0u8; 28]);
    assert_eq!(one.data, tty_acs_double_borders_list[1].data);
}

#[test]
fn the_border_tables_answer_one_character_per_cell_type() {
    unsafe {
        assert_eq!(
            seen(tty_acs_double_borders(0).data.as_ptr() as *const ::core::ffi::c_char),
            ""
        );
        assert_eq!(tty_acs_double_borders(0).size, 0);
        assert_eq!(
            seen(tty_acs_double_borders(1).data.as_ptr() as *const ::core::ffi::c_char),
            "\u{2551}"
        );
        assert_eq!(
            seen(tty_acs_double_borders(12).data.as_ptr() as *const ::core::ffi::c_char),
            "\u{00b7}"
        );
        assert_eq!(tty_acs_double_borders(12).size, 2);
        assert_eq!(
            seen(tty_acs_heavy_borders(1).data.as_ptr() as *const ::core::ffi::c_char),
            "\u{2503}"
        );
        assert_eq!(
            seen(tty_acs_heavy_borders(12).data.as_ptr() as *const ::core::ffi::c_char),
            "\u{00b7}"
        );
        assert_eq!(
            seen(tty_acs_rounded_borders(3).data.as_ptr() as *const ::core::ffi::c_char),
            "\u{256d}"
        );
        assert_eq!(
            seen(tty_acs_rounded_borders(12).data.as_ptr() as *const ::core::ffi::c_char),
            "\u{00b7}"
        );
        for i in 1..13 {
            assert_eq!(tty_acs_double_borders(i).width, 1);
            assert_eq!(tty_acs_heavy_borders(i).width, 1);
            assert_eq!(tty_acs_rounded_borders(i).width, 1);
        }
    }
}

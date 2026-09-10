use super::*;
use crate::compat::{VIS_CSTYLE, VIS_NL, VIS_OCTAL};
use crate::options::{OptionsEngine, OptionsRef, RustOptionsEngine};

use crate::tests::test_fixtures::globals;
use crate::tmux::global_options;
use ::core::ffi::{CStr, c_int};

/// Serializes tests that reset the shared character store and process options.
/// Each test also starts and ends with an empty thread-local width cache.
struct Globals {
    _guard: crate::tests::test_fixtures::GlobalsGuard,
}

impl Drop for Globals {
    fn drop(&mut self) {
        forget_everything();
    }
}

fn exclusive() -> Globals {
    let guard = globals();
    forget_everything();
    Globals { _guard: guard }
}

/// Resets the shared character store and width cache between isolated tests.
fn forget_everything() {
    let UTF8_STORE = UTF8_STORE_FIELD.get();

    {
        *UTF8_STORE.borrow_mut() = Utf8Store::new();
        UTF8_WIDTH_CACHE.with_borrow_mut(|cache| cache.clear());
        UTF8_NO_WIDTH.set(false);
    }
}

/// The `utf8_data` a character of `bytes` makes.
fn filled(bytes: &[u8], width: u_char) -> utf8_data {
    let mut ud = utf8_data::default();
    ud.data[..bytes.len()].copy_from_slice(bytes);
    ud.have = bytes.len() as u_char;
    ud.size = ud.have;
    ud.width = width;
    ud
}

/// The bytes a `utf8_data` holds.
fn bytes_of(ud: &utf8_data) -> Vec<u8> {
    ud.data[..ud.size as usize].to_vec()
}

/// Runs a character through `utf8_open` and `utf8_append`, the way the
/// terminal input parser feeds it one byte at a time.
fn open_append(bytes: &[u8]) -> (utf8_state, utf8_data) {
    unsafe {
        let mut ud = utf8_data::default();
        let mut state = utf8_open(&mut ud, bytes[0]);
        for &b in &bytes[1..] {
            if state != UTF8_MORE {
                break;
            }
            state = utf8_append(&mut ud, b);
        }
        (state, ud)
    }
}

#[test]
fn only_a_lead_byte_opens_a_character() {
    let mut ud = utf8_data::default();
    assert_eq!(utf8_open(&mut ud, b'A'), UTF8_ERROR);
    assert_eq!(utf8_open(&mut ud, 0x80), UTF8_ERROR);
    assert_eq!(utf8_open(&mut ud, 0xc1), UTF8_ERROR);
    assert_eq!(utf8_open(&mut ud, 0xf5), UTF8_ERROR);
    assert_eq!(utf8_open(&mut ud, 0xff), UTF8_ERROR);
}

#[test]
fn a_lead_byte_says_how_many_bytes_are_still_to_come() {
    for (lead, size) in [
        (0xc2u8, 2 as u_char),
        (0xdf, 2),
        (0xe0, 3),
        (0xef, 3),
        (0xf0, 4),
        (0xf4, 4),
    ] {
        let mut ud = utf8_data::default();
        assert_eq!(utf8_open(&mut ud, lead), UTF8_MORE);
        assert_eq!(ud.size, size);
        assert_eq!(ud.have, 1);
        assert_eq!(ud.data[0], lead);
    }
}

#[test]
fn a_finished_character_is_measured() {
    let _guard = exclusive();
    {
        let (state, ud) = open_append("é".as_bytes());
        assert_eq!(state, UTF8_DONE);
        assert_eq!((ud.size, ud.have, ud.width), (2, 2, 1));

        let (state, ud) = open_append("€".as_bytes());
        assert_eq!(state, UTF8_DONE);
        assert_eq!((ud.size, ud.have, ud.width), (3, 3, 1));

        let (state, ud) = open_append("😀".as_bytes());
        assert_eq!(state, UTF8_DONE);
        assert_eq!((ud.size, ud.have, ud.width), (4, 4, 2));
    }
}

#[test]
fn a_byte_that_is_not_a_continuation_spoils_the_character() {
    let _guard = exclusive();
    {
        let (state, ud) = open_append(b"\xc3A");
        assert_eq!(state, UTF8_ERROR);
        assert_eq!(ud.width, 0xff);

        let (state, _) = open_append(b"\xe2\x82A");
        assert_eq!(state, UTF8_ERROR);
    }
}

#[test]
fn a_character_that_no_codepoint_answers_to_is_an_error() {
    let _guard = exclusive();
    {
        let (state, _) = open_append(b"\xed\xa0\x80");
        assert_eq!(state, UTF8_ERROR);
        let (state, _) = open_append(b"\xc0\x80");
        assert_eq!(state, UTF8_ERROR);
    }
}

#[test]
fn the_width_is_not_worked_out_while_the_cache_is_being_filled() {
    let _guard = exclusive();
    {
        UTF8_NO_WIDTH.set(true);
        let (state, ud) = open_append(b"\xed\xa0\x80");
        assert_eq!(state, UTF8_DONE);
        assert_eq!(ud.width, 0);
        UTF8_NO_WIDTH.set(false);
    }
}

#[test]
fn a_character_and_its_codepoint_convert_both_ways() {
    let _guard = exclusive();
    unsafe {
        let ud = filled("é".as_bytes(), 1);
        let wc = utf8_towc(&ud).expect("é is a codepoint");
        assert_eq!(wc, 0xe9);

        let mut back = utf8_data::default();
        assert_eq!(utf8_fromwc(wc, &mut back), UTF8_DONE);
        assert_eq!(bytes_of(&back), "é".as_bytes());
        assert_eq!((back.have, back.size, back.width), (2, 2, 1));
    }
}

#[test]
fn a_character_that_is_not_utf8_has_no_codepoint() {
    let _guard = exclusive();
    unsafe {
        let ud = filled(b"\xff\xfe", 1);
        assert_eq!(utf8_towc(&ud), None);

        let nul = utf8_data {
            size: 1,
            ..utf8_data::default()
        };
        assert_eq!(utf8_towc(&nul), Some(0));
    }
}

#[test]
fn a_codepoint_that_has_no_character_or_no_width_is_an_error() {
    let _guard = exclusive();
    unsafe {
        let mut ud = utf8_data::default();
        assert_eq!(utf8_fromwc(0x110000, &mut ud), UTF8_ERROR);
        assert_eq!(utf8_fromwc(0xd800, &mut ud), UTF8_ERROR);
        assert_eq!(utf8_fromwc(0, &mut ud), UTF8_DONE);
        assert_eq!((ud.have, ud.size, ud.width), (1, 1, 0));
    }
}

#[test]
fn a_cached_width_is_used_in_place_of_the_measured_one() {
    let _guard = exclusive();
    unsafe {
        let ud = filled("é".as_bytes(), 0);
        assert_eq!(utf8_width(&ud), Ok(1));

        utf8_insert_width_cache(0xe9, 2);
        assert_eq!(utf8_width(&ud), Ok(2));

        utf8_insert_width_cache(0xe9, 0);
        assert_eq!(utf8_width(&ud), Ok(0));

        let bad = filled(b"\xff", 0);
        assert_eq!(utf8_width(&bad), Err(UTF8_ERROR));
    }
}

#[test]
fn a_width_given_again_takes_the_place_of_the_one_before_it() {
    let _guard = exclusive();
    {
        for wc in 1..=15 {
            utf8_insert_width_cache(wc, 1);
        }
        for wc in [4, 6, 1, 15, 8, 12, 2, 3, 5, 7, 9, 10, 11, 13, 14] {
            utf8_insert_width_cache(wc, 2);
            assert_eq!(utf8_find_in_width_cache(wc).unwrap(), 2, "U+{wc:X}");
        }
        for wc in 1..=15 {
            assert_eq!(utf8_find_in_width_cache(wc).unwrap(), 2, "U+{wc:X}");
        }
    }
}

#[test]
fn the_width_cache_takes_codepoints_and_ranges_of_them() {
    let _guard = exclusive();
    unsafe {
        utf8_add_to_width_cache(c"U+41=2");
        assert_eq!(utf8_find_in_width_cache(0x41).unwrap(), 2);

        utf8_add_to_width_cache(c"U+61-U+63=0");
        for wc in 0x61..=0x63 {
            assert_eq!(utf8_find_in_width_cache(wc).unwrap(), 0);
        }
        assert!(utf8_find_in_width_cache(0x64).is_none());

        utf8_add_to_width_cache(c"\xc3\xa9=2");
        assert_eq!(utf8_find_in_width_cache(0xe9).unwrap(), 2);
    }
}

#[test]
fn the_width_cache_turns_down_what_it_cannot_read() {
    let _guard = exclusive();
    unsafe {
        for spec in [
            c"U+41",
            c"U+41=x",
            c"U+41=3",
            c"U+=1",
            c"U+0=1",
            c"U+FFFFFFFF=1",
            c"U+FFFFFFFFFFFFFFFFFF=1",
            c"U+41x=1",
            c"U+41-x=1",
            c"U+41-U+40=1",
            c"U+41-U+42x=1",
            c"U+41-U+0=1",
            c"=1",
            c"ab=1",
            c"\xed\xa0\x80=1",
        ] {
            utf8_add_to_width_cache(spec);
            assert!(
                UTF8_WIDTH_CACHE.with_borrow(|cache| cache.is_empty()),
                "{spec:?} went into the cache"
            );
        }
    }
}

#[test]
fn the_width_cache_is_rebuilt_from_the_defaults_and_the_option() {
    let _guard = exclusive();
    unsafe {
        utf8_insert_width_cache(0x41, 2);
        let store = global_options
            .get()
            .expect("global options are initialized");
        store.with_entry_mut(c"codepoint-widths", false, |entry| {
            RustOptionsEngine.array_set(entry.unwrap(), 0, Some(c"U+42=2"), 0, &mut None)
        });

        utf8_update_width_cache(
            (global_options
                .get()
                .as_ref()
                .expect("global options are initialized"))
            .codepoint_widths(),
        );

        assert!(utf8_find_in_width_cache(0x41).is_none());
        assert_eq!(utf8_find_in_width_cache(0x42).unwrap(), 2);
        assert_eq!(utf8_find_in_width_cache(0x261d).unwrap(), 2);
        assert_eq!(utf8_find_in_width_cache(0x1faf8).unwrap(), 2);

        utf8_update_width_cache(
            (global_options
                .get()
                .as_ref()
                .expect("global options are initialized"))
            .codepoint_widths(),
        );
        assert_eq!(utf8_find_in_width_cache(0x42).unwrap(), 2);

        store.with_entry_mut(c"codepoint-widths", false, |entry| {
            RustOptionsEngine.array_set(entry.unwrap(), 0, Some(c""), 0, &mut None)
        });
    }
}

#[test]
fn a_short_character_is_carried_in_the_utf8_char_itself() {
    let UTF8_STORE = UTF8_STORE_FIELD.get();

    let _guard = exclusive();
    {
        let ud = filled(b"\xc3\xa9", 1);
        let (state, uc) = utf8_from_data(&ud);
        assert_eq!((state, uc), (UTF8_DONE, 0x4200a9c3));
        assert!(UTF8_STORE.borrow_mut().by_data.is_empty());

        let mut back = utf8_data::default();
        utf8_to_data(uc, &mut back);
        assert_eq!(bytes_of(&back), b"\xc3\xa9");
        assert_eq!((back.have, back.size, back.width), (2, 2, 1));
    }
}

#[test]
fn a_long_character_is_kept_in_the_trees_under_an_index() {
    let _guard = exclusive();
    {
        let ud = filled("😀".as_bytes(), 2);
        let (state, uc) = utf8_from_data(&ud);
        assert_eq!((state, uc), (UTF8_DONE, 0x64000000));

        assert_eq!(utf8_from_data(&ud), (UTF8_DONE, uc));

        let other = filled("😁".as_bytes(), 2);
        assert_eq!(utf8_from_data(&other), (UTF8_DONE, 0x64000001));

        let mut back = utf8_data::default();
        utf8_to_data(uc, &mut back);
        assert_eq!(bytes_of(&back), "😀".as_bytes());

        utf8_to_data(0x64000009, &mut back);
        assert_eq!(bytes_of(&back), b"    ");
    }
}

#[test]
fn characters_are_kept_in_order_of_length_and_then_of_bytes() {
    let _guard = exclusive();
    {
        let words: [&[u8]; 5] = [
            b"123456",
            b"12345",
            b"1234",
            "\u{1f600}".as_bytes(),
            "\u{1f601}".as_bytes(),
        ];
        let mut chars = Vec::new();
        for bytes in words {
            let ud = filled(bytes, 1);
            let (state, uc) = utf8_from_data(&ud);
            assert_eq!(state, UTF8_DONE);
            chars.push(uc);
        }
        assert_eq!(
            chars.iter().map(|uc| uc & 0xffffff).collect::<Vec<u_int>>(),
            [0, 1, 2, 3, 4]
        );
        for (uc, bytes) in chars.iter().zip(words) {
            let ud = filled(bytes, 1);
            assert_eq!(utf8_from_data(&ud), (UTF8_DONE, *uc));

            let mut back = utf8_data::default();
            utf8_to_data(*uc, &mut back);
            assert_eq!(bytes_of(&back), bytes);
        }
    }
}

#[test]
fn a_character_that_will_not_fit_comes_back_as_spaces() {
    let _guard = exclusive();
    {
        let mut ud = filled(b"x", 0);
        ud.size = 33;
        assert_eq!(utf8_from_data(&ud), (UTF8_ERROR, 0x20000000));

        ud.width = 1;
        assert_eq!(utf8_from_data(&ud), (UTF8_ERROR, 0x41000020));

        ud.width = 2;
        assert_eq!(utf8_from_data(&ud), (UTF8_ERROR, 0x41002020));
    }
}

#[test]
fn the_last_index_is_where_the_trees_stop_taking_characters() {
    let UTF8_STORE = UTF8_STORE_FIELD.get();

    let _guard = exclusive();
    {
        UTF8_STORE.borrow_mut().next_index = UTF8_INDEX_END;
        let ud = filled("😀".as_bytes(), 2);
        assert_eq!(utf8_from_data(&ud), (UTF8_ERROR, 0x41002020));
    }
}

#[test]
fn one_byte_makes_its_own_character() {
    assert_eq!(utf8_build_one(b'A'), 0x41000041);
    let mut ud = utf8_data::default();
    ud.data[5] = b'x';
    utf8_set(&mut ud, b'A');
    assert_eq!((ud.have, ud.size, ud.width), (1, 1, 1));
    assert_eq!(ud.data[0], b'A');
    assert_eq!(ud.data[5], 0);
}

#[test]
fn copying_a_character_clears_what_is_past_its_end() {
    let from = filled(b"\xc3\xa9", 1);
    let mut to = utf8_data {
        data: [b'x'; 32],
        ..utf8_data::default()
    };
    utf8_copy(&mut to, &from);
    assert_eq!(to.data, from.data);
    assert_eq!((to.have, to.size, to.width), (2, 2, 1));
}

#[test]
fn a_string_is_taken_apart_into_characters_and_put_back_together() {
    let _guard = exclusive();
    let ud = utf8_fromcstr(c"a\xc3\xa9\xff\xf0\x9f\x98\x80");
    assert_eq!(utf8_vec_strlen(&ud), 4);
    assert_eq!(bytes_of(&ud[0]), b"a");
    assert_eq!(bytes_of(&ud[1]), b"\xc3\xa9");
    assert_eq!(bytes_of(&ud[2]), b"\xff");
    assert_eq!(bytes_of(&ud[3]), "😀".as_bytes());
    assert_eq!(utf8_vec_strwidth(&ud, -1), 5);
    assert_eq!(utf8_vec_strwidth(&ud, 2), 2);
    assert_eq!(utf8_vec_strwidth(&ud, 0), 0);

    let s = utf8_vec_tocstr(&ud);
    assert_eq!(s.as_bytes(), b"a\xc3\xa9\xff\xf0\x9f\x98\x80");
}

#[test]
fn a_half_finished_character_at_the_end_is_taken_one_byte_at_a_time() {
    let _guard = exclusive();
    let ud = utf8_fromcstr(c"\xc3");
    assert_eq!(utf8_vec_strlen(&ud), 1);
    assert_eq!(bytes_of(&ud[0]), b"\xc3");

    let ud = utf8_fromcstr(c"");
    assert_eq!(utf8_vec_strlen(&ud), 0);
}

#[test]
fn the_width_of_a_string_counts_its_printable_bytes() {
    let _guard = exclusive();
    assert_eq!(utf8_cstrwidth(c"abc"), 3);
    assert_eq!(utf8_cstrwidth(c"a\xc3\xa9\xf0\x9f\x98\x80"), 4);
    assert_eq!(utf8_cstrwidth(c"a\x01b\x7f"), 2);
    assert_eq!(utf8_cstrwidth(c"\xc3"), 0);
    assert_eq!(utf8_cstrwidth(c""), 0);
}

#[test]
fn a_string_is_padded_to_a_width_on_either_side() {
    let _guard = exclusive();
    assert_eq!(utf8_padcstr(c"ab", 4).into_bytes(), b"ab  ");
    assert_eq!(utf8_padcstr(c"ab", 2).into_bytes(), b"ab");
    assert_eq!(utf8_padcstr(c"ab", 1).into_bytes(), b"ab");
    assert_eq!(utf8_rpadcstr(c"ab", 4).into_bytes(), b"  ab");
    assert_eq!(utf8_rpadcstr(c"ab", 2).into_bytes(), b"ab");
}

#[test]
fn a_string_is_searched_for_a_character() {
    let _guard = exclusive();
    let e = filled(b"\xc3\xa9", 1);
    let a = filled(b"a", 1);
    assert_eq!(utf8_cstrhas(c"a\xc3\xa9b", &e), 1);
    assert_eq!(utf8_cstrhas(c"a\xc3\xa9b", &a), 1);
    assert_eq!(utf8_cstrhas(c"abc", &e), 0);
    assert_eq!(utf8_cstrhas(c"", &a), 0);
}

#[test]
fn a_string_is_valid_when_it_is_utf8_and_printable() {
    let _guard = exclusive();
    assert_eq!(utf8_isvalid(c"abc"), 1);
    assert_eq!(utf8_isvalid(c"a\xc3\xa9b"), 1);
    assert_eq!(utf8_isvalid(c""), 1);
    assert_eq!(utf8_isvalid(c"a\x01b"), 0);
    assert_eq!(utf8_isvalid(c"a\x7f"), 0);
    assert_eq!(utf8_isvalid(c"\xff"), 0);
    assert_eq!(utf8_isvalid(c"\xc3"), 0);
}

#[test]
fn sanitizing_replaces_everything_that_is_not_plain_ascii() {
    let _guard = exclusive();
    assert_eq!(utf8_sanitize(c"abc").as_bytes(), b"abc");
    assert_eq!(utf8_sanitize(c"a\x01b\x7f").as_bytes(), b"a_b_");
    assert_eq!(utf8_sanitize(c"a\xc3\xa9").as_bytes(), b"a_");
    assert_eq!(utf8_sanitize(c"\xf0\x9f\x98\x80").as_bytes(), b"__");
    assert_eq!(utf8_sanitize(c"\xc3").as_bytes(), b"_");
    assert_eq!(utf8_sanitize(c"").as_bytes(), b"");
}

/// The visible form produced by `utf8_strvis`.
fn strvis(src: &[u8], flag: c_int) -> Vec<u8> {
    utf8_strvis(src, flag)
}

#[test]
fn visible_form_keeps_utf8_characters_and_escapes_the_rest() {
    let _guard = exclusive();
    {
        assert_eq!(strvis(b"abc", 0), b"abc");
        assert_eq!(strvis("é".as_bytes(), 0), "é".as_bytes());
        assert_eq!(strvis(b"\xff", VIS_OCTAL), b"\\377");
        assert_eq!(strvis(b"a\nb", VIS_CSTYLE | VIS_NL), b"a\\nb");
        assert_eq!(strvis(b"a\nb", VIS_CSTYLE), b"a\nb");
        assert_eq!(strvis(b"\xc3", VIS_OCTAL), b"\\303");
        assert_eq!(strvis(b"\xc3z", VIS_OCTAL), b"\\303z");
    }
}

#[test]
fn a_dollar_is_escaped_inside_double_quotes_when_it_would_expand() {
    let _guard = exclusive();
    {
        assert_eq!(strvis(b"$a", VIS_DQ), b"\\$a");
        assert_eq!(strvis(b"$_", VIS_DQ), b"\\$_");
        assert_eq!(strvis(b"${", VIS_DQ), b"\\${");
        assert_eq!(strvis(b"$1", VIS_DQ), b"$1");
        assert_eq!(strvis(b"$", VIS_DQ), b"$");
        assert_eq!(strvis(b"$a", 0), b"$a");
    }
}

#[test]
fn the_visible_form_is_allocated_to_fit() {
    let _guard = exclusive();
    let dst = utf8_stravis(c"a\xff", VIS_OCTAL);
    assert_eq!(dst.as_bytes(), b"a\\377");

    let dst = utf8_stravisx(b"a", VIS_OCTAL);
    assert_eq!(dst.as_bytes(), b"a");
}

#[test]
fn visual_decode_returns_bytes_and_rejects_bad_syntax() {
    assert_eq!(RustUtf8VisModel.decode(c"a\\nb"), Some(b"a\nb".to_vec()));
    assert_eq!(RustUtf8VisModel.decode(c"a\\0b"), Some(b"a\0b".to_vec()));
    assert_eq!(RustUtf8VisModel.decode(c"a\\z"), None);
}

#[test]
fn character_store_exhaustion_keeps_existing_identities_and_both_indexes() {
    let mut store = Utf8Store::new();
    store.next_index = UTF8_INDEX_END - 1;
    let last = utf8_stored_of("😀".as_bytes());
    let rejected = utf8_stored_of("😁".as_bytes());
    assert_eq!(store.intern(last), Some((UTF8_INDEX_END - 1, true)));
    assert_eq!(store.intern(rejected), None);
    assert_eq!(store.intern(rejected), None);
    assert_eq!(store.intern(last), Some((UTF8_INDEX_END - 1, false)));
    assert_eq!(store.by_data.len(), 1);
    assert_eq!(store.by_index.get(&(UTF8_INDEX_END - 1)).map(std::rc::Rc::as_ref), Some(&last));
}

#[test]
fn character_store_reuses_ids_and_decodes_every_interned_character() {
    let _guard = exclusive();
    let mut packed = Vec::new();
    for offset in 0..8 {
        for n in 0..128 {
            let character = char::from_u32(0x1f600 + (n + offset) % 32).unwrap();
            let mut bytes = [0; 4];
            let text = character.encode_utf8(&mut bytes);
            let (state, id) = utf8_from_data(&filled(text.as_bytes(), 2));
            assert_eq!(state, UTF8_DONE);
            packed.push((id, bytes));
        }
    }
    for (id, bytes) in packed {
        let mut decoded = utf8_data::default();
        utf8_to_data(id, &mut decoded);
        assert_eq!(&decoded.data[..4], &bytes);
    }
    UTF8_STORE_FIELD.with(|store| {
        let store = store.borrow();
        assert_eq!(store.by_data.len(), 32);
        assert_eq!(store.by_index.len(), 32);
        for (data, index) in &store.by_data {
            assert!(std::rc::Rc::ptr_eq(store.by_index.get(index).unwrap(), data));
        }
    });
}

#[test]
fn width_configuration_and_suppression_are_thread_local() {
    let _guard = exclusive();
    utf8_update_width_cache([c"U+1F600=1".to_owned()]);
    without_width(|| {
        std::thread::spawn(|| {
            assert!(!UTF8_NO_WIDTH.get());
            assert_eq!(utf8_find_in_width_cache(0x1f600), None);
            utf8_update_width_cache([c"U+1F600=2".to_owned()]);
            assert_eq!(utf8_find_in_width_cache(0x1f600), Some(2));
        })
        .join()
        .unwrap();
        assert!(UTF8_NO_WIDTH.get());
        assert_eq!(utf8_find_in_width_cache(0x1f600), Some(1));
    });
    assert!(!UTF8_NO_WIDTH.get());
}

#[test]
fn width_suppression_restores_nested_and_unwound_state() {
    std::thread::spawn(|| {
        assert!(!UTF8_NO_WIDTH.get());
        without_width(|| {
            assert!(UTF8_NO_WIDTH.get());
            without_width(|| assert!(UTF8_NO_WIDTH.get()));
            assert!(UTF8_NO_WIDTH.get());
            let failed = std::panic::catch_unwind(|| without_width(|| panic!("parser failure")));
            assert!(failed.is_err());
            assert!(UTF8_NO_WIDTH.get());
        });
        assert!(!UTF8_NO_WIDTH.get());
        let failed = std::panic::catch_unwind(|| without_width(|| panic!("parser failure")));
        assert!(failed.is_err());
        assert!(!UTF8_NO_WIDTH.get());
    })
    .join()
    .unwrap();
}

#[test]
fn codepoint_parser_borrows_the_suffix_and_preserves_c_numeric_forms() {
    let text = c"U+ +0x41-U+42";
    let (first, rest) = utf8_parse_codepoint(text).unwrap();
    assert_eq!(first, 0x41);
    assert_eq!(rest, c"-U+42");
    assert_eq!(
        utf8_parse_codepoint(c"U+\t0042trailing"),
        Some((0x42, c"trailing"))
    );
    assert_eq!(utf8_parse_codepoint(c"U+42"), Some((0x42, c"")));
    assert_eq!(utf8_parse_codepoint(c"U+0"), None);
    assert_eq!(utf8_parse_codepoint(c"U+-1"), None);
    assert_eq!(utf8_parse_codepoint(c"U+FFFFFFFFFFFFFFFFFFFFFFFF"), None);
}

/// The visible form of `src` up to its terminator.
pub(crate) fn utf8_stravis(src: &CStr, flag: core::ffi::c_int) -> CString {
    utf8_stravisx(src.to_bytes(), flag)
}

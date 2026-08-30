use super::*;
use crate::tests::test_fixtures::{globals, zeroed};
use crate::types::u_char;

const ZWJ: &[u8] = &[0xe2, 0x80, 0x8d];
const VS16: &[u8] = &[0xef, 0xb8, 0x8f];
const FILLER: &[u8] = &[0xe3, 0x85, 0xa4];
/// U+1F1FA and U+1F1F8, the regional indicators that spell a flag.
const RI_U: &[u8] = &[0xf0, 0x9f, 0x87, 0xba];
const RI_S: &[u8] = &[0xf0, 0x9f, 0x87, 0xb8];
/// U+1F3FB, the lightest skin tone.
const TONE: &[u8] = &[0xf0, 0x9f, 0x8f, 0xbb];
/// U+1F44B, a waving hand, which takes a skin tone.
const HAND: &[u8] = &[0xf0, 0x9f, 0x91, 0x8b];
/// U+1F600, a grinning face, which does not.
const FACE: &[u8] = &[0xf0, 0x9f, 0x98, 0x80];

/// The character `bytes` spell, as the UTF-8 parser would hand it over.
fn ud(bytes: &[u8]) -> utf8_data {
    let mut u = *zeroed::<utf8_data>();
    u.data[..bytes.len()].copy_from_slice(bytes);
    u.have = bytes.len() as u_char;
    u.size = bytes.len() as u_char;
    u.width = 1;
    u
}

/// The same, from two characters written one after the other.
fn both(first: &[u8], second: &[u8]) -> utf8_data {
    ud(&[first, second].concat())
}

#[test]
fn a_joiner_at_the_end_is_found() {
    unsafe {
        assert_eq!(utf8_has_zwj(&ud(b"ab")), 0);
        assert_eq!(utf8_has_zwj(&ud(ZWJ)), 1);
        assert_eq!(utf8_has_zwj(&both(HAND, ZWJ)), 1);
        assert_eq!(utf8_has_zwj(&both(ZWJ, HAND)), 0);
    }
}

#[test]
fn a_character_that_is_only_a_joiner_is_found() {
    unsafe {
        assert_eq!(utf8_is_zwj(&ud(ZWJ)), 1);
        assert_eq!(utf8_is_zwj(&both(HAND, ZWJ)), 0);
        assert_eq!(utf8_is_zwj(&ud(FILLER)), 0);
    }
}

#[test]
fn a_variation_selector_is_found() {
    unsafe {
        assert_eq!(utf8_is_vs(&ud(VS16)), 1);
        assert_eq!(utf8_is_vs(&ud(b"ab")), 0);
        assert_eq!(utf8_is_vs(&ud(ZWJ)), 0);
    }
}

#[test]
fn a_hangul_filler_is_found() {
    unsafe {
        assert_eq!(utf8_is_hangul_filler(&ud(FILLER)), 1);
        assert_eq!(utf8_is_hangul_filler(&ud(b"ab")), 0);
        assert_eq!(utf8_is_hangul_filler(&ud(ZWJ)), 0);
    }
}

#[test]
fn a_character_that_does_not_decode_never_combines() {
    let _guard = globals();
    unsafe {
        assert_eq!(utf8_should_combine(&ud(&[0x80]), &ud(HAND)), 0);
        assert_eq!(utf8_should_combine(&ud(TONE), &ud(&[0x80])), 0);
    }
}

#[test]
fn two_regional_indicators_make_a_flag() {
    let _guard = globals();
    unsafe {
        assert_eq!(utf8_should_combine(&ud(RI_U), &ud(RI_S)), 1);
    }
}

/// A flag is two indicators and no more: neither side may already be one.
#[test]
fn a_finished_flag_takes_no_more_indicators() {
    let _guard = globals();
    unsafe {
        assert_eq!(utf8_should_combine(&both(RI_U, RI_S), &ud(RI_S)), 0);
        assert_eq!(utf8_should_combine(&ud(RI_U), &both(RI_S, RI_U)), 0);
    }
}

/// The emoji list is read against what is being *added*, and the skin tone
/// against what is already there, so a tone written in front of a hand
/// combines while the same pair the other way round does not.
#[test]
fn a_skin_tone_in_front_of_a_hand_combines() {
    let _guard = globals();
    unsafe {
        assert_eq!(utf8_should_combine(&ud(TONE), &ud(HAND)), 1);
        assert_eq!(utf8_should_combine(&ud(HAND), &ud(TONE)), 0);
        assert_eq!(utf8_should_combine(&ud(TONE), &ud(FACE)), 0);
        assert_eq!(utf8_should_combine(&ud(HAND), &ud(HAND)), 0);
    }
}

/// What `hanguljamo_check_state` makes of `ud` written after `p_ud`.
fn state(previous: &[u8], next: &[u8]) -> hanguljamo_state {
    unsafe { hanguljamo_check_state(&ud(previous), &ud(next)) }
}

#[test]
fn only_a_three_byte_character_can_be_a_jamo() {
    assert_eq!(state(b"", b"ab"), HANGULJAMO_STATE_NOT_HANGULJAMO);
    assert_eq!(state(b"", FILLER), HANGULJAMO_STATE_NOT_HANGULJAMO);
}

#[test]
fn a_leading_consonant_starts_a_syllable() {
    for lead in [
        &[0xe1, 0x84, 0x80],
        &[0xe1, 0x84, 0x92],
        &[0xe1, 0x84, 0x93],
        &[0xe1, 0x84, 0xbf],
        &[0xe1, 0x85, 0x9f],
        &[0xe1, 0x85, 0x80],
        &[0xe1, 0x85, 0x9e],
        &[0xea, 0xa5, 0xa0],
        &[0xea, 0xa5, 0xbc],
    ] {
        assert_eq!(
            state(b"", lead),
            HANGULJAMO_STATE_CHOSEONG,
            "{lead:02x?} starts a syllable"
        );
    }
}

#[test]
fn a_vowel_composes_only_after_a_leading_consonant() {
    let lead: &[u8] = &[0xe1, 0x84, 0x80];
    for vowel in [
        &[0xe1, 0x85, 0xa0],
        &[0xe1, 0x85, 0xa1],
        &[0xe1, 0x85, 0xb5],
        &[0xe1, 0x85, 0xb6],
        &[0xe1, 0x85, 0xbf],
        &[0xe1, 0x86, 0x80],
        &[0xe1, 0x86, 0xa7],
        &[0xed, 0x9e, 0xb0],
        &[0xed, 0x9e, 0xbf],
        &[0xed, 0x9f, 0x80],
        &[0xed, 0x9f, 0x86],
    ] {
        assert_eq!(
            state(lead, vowel),
            HANGULJAMO_STATE_COMPOSABLE,
            "{vowel:02x?} follows a leading consonant"
        );
        assert_eq!(
            state(b"", vowel),
            HANGULJAMO_STATE_NOT_COMPOSABLE,
            "{vowel:02x?} has nothing in front of it"
        );
        assert_eq!(
            state(FILLER, vowel),
            HANGULJAMO_STATE_NOT_COMPOSABLE,
            "{vowel:02x?} follows something that is no jamo"
        );
    }
}

#[test]
fn a_trailing_consonant_composes_only_after_a_vowel() {
    let vowel: &[u8] = &[0xe1, 0x85, 0xa1];
    for tail in [
        &[0xe1, 0x86, 0xa8],
        &[0xe1, 0x86, 0xbf],
        &[0xe1, 0x87, 0x80],
        &[0xe1, 0x87, 0x82],
        &[0xe1, 0x87, 0x83],
        &[0xe1, 0x87, 0xbf],
        &[0xed, 0x9f, 0x8b],
        &[0xed, 0x9f, 0xbb],
    ] {
        assert_eq!(
            state(vowel, tail),
            HANGULJAMO_STATE_COMPOSABLE,
            "{tail:02x?} follows a vowel"
        );
        assert_eq!(
            state(b"", tail),
            HANGULJAMO_STATE_NOT_COMPOSABLE,
            "{tail:02x?} has nothing in front of it"
        );
        assert_eq!(
            state(&[0xe1, 0x84, 0x80], tail),
            HANGULJAMO_STATE_NOT_COMPOSABLE,
            "{tail:02x?} follows a leading consonant"
        );
    }
}

/// A jamo is read out of the *last* three bytes of what is already there,
/// so a combined character ending in a leading consonant still composes.
#[test]
fn the_last_three_bytes_of_what_is_there_are_what_is_read() {
    assert_eq!(
        unsafe {
            hanguljamo_check_state(&both(HAND, &[0xe1, 0x84, 0x80]), &ud(&[0xe1, 0x85, 0xa1]))
        },
        HANGULJAMO_STATE_COMPOSABLE
    );
}

#[test]
fn a_three_byte_character_that_is_no_jamo_is_no_jamo() {
    for other in [
        &[0xe1, 0x88, 0x80],
        &[0xe1, 0x84, 0x00],
        &[0xe1, 0x85, 0x00],
        &[0xe1, 0x86, 0x00],
        &[0xe1, 0x87, 0x00],
        &[0xea, 0xa6, 0xa0],
        &[0xea, 0xa5, 0x9f],
        &[0xed, 0x9e, 0xaf],
        &[0xed, 0x9d, 0xb0],
        &[0xed, 0x9f, 0x87],
        &[0xed, 0x9f, 0xbc],
        &[0xe2, 0x80, 0x8d],
    ] {
        assert_eq!(
            state(b"", other),
            HANGULJAMO_STATE_NOT_HANGULJAMO,
            "{other:02x?} is no jamo"
        );
    }
}

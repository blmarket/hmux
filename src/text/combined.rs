use crate::types::wchar_t;
use super::utf8::{utf8_data, utf8_towc};

/// How far a hangul jamo sequence has been read.
pub type hanguljamo_state = ::core::ffi::c_uint;

pub const HANGULJAMO_STATE_NOT_COMPOSABLE: hanguljamo_state = 3;
pub const HANGULJAMO_STATE_COMPOSABLE: hanguljamo_state = 2;
pub const HANGULJAMO_STATE_CHOSEONG: hanguljamo_state = 1;
pub const HANGULJAMO_STATE_NOT_HANGULJAMO: hanguljamo_state = 0;

/// The zero-width joiner, U+200D.
const ZWJ: &[u8] = &[0xe2, 0x80, 0x8d];
/// The emoji variation selector, U+FE0F.
const VARIATION_SELECTOR: &[u8] = &[0xef, 0xb8, 0x8f];
/// The Hangul filler, U+3164.
const HANGUL_FILLER: &[u8] = &[0xe3, 0x85, 0xa4];
/// The regional indicators, U+1F1E6 to U+1F1FF: two of them spell a flag.
const REGIONAL: ::core::ops::RangeInclusive<wchar_t> = 0x1f1e6..=0x1f1ff;
/// The skin tone modifiers, U+1F3FB to U+1F3FF.
const SKIN_TONE: ::core::ops::RangeInclusive<wchar_t> = 0x1f3fb..=0x1f3ff;

/// The bytes a character is written as. A character never holds more than the
/// 32 bytes of its own array, so the slice is always in range.
fn bytes(ud: &utf8_data) -> &[u8] {
    &ud.data[..ud.size as usize]
}

/// The codepoint a character stands for, or nothing when it is not one.
fn towc(ud: &utf8_data) -> Option<wchar_t> {
    unsafe { utf8_towc(ud) }
}

/// Whether the character ends with a zero-width joiner.
pub fn utf8_has_zwj(ud: &utf8_data) -> bool {
    bytes(ud).ends_with(ZWJ)
}

/// Whether the character is nothing but a zero-width joiner.
pub fn utf8_is_zwj(ud: &utf8_data) -> bool {
    bytes(ud) == ZWJ
}

/// Whether the character is nothing but a variation selector.
pub fn utf8_is_vs(ud: &utf8_data) -> bool {
    bytes(ud) == VARIATION_SELECTOR
}

/// Whether the character is nothing but a Hangul filler.
pub fn utf8_is_hangul_filler(ud: &utf8_data) -> bool {
    bytes(ud) == HANGUL_FILLER
}

/// How many regional indicators the character holds. Every byte offset is
/// looked at rather than every character, which is what the C did; an
/// indicator is four bytes and none of them overlaps another, so the count is
/// the same.
fn regional_count(ud: &utf8_data) -> usize {
    bytes(ud)
        .windows(4)
        .filter(|w| w[0] == 0xf0 && w[1] == 0x9f && w[2] == 0x87 && (0xa6..=0xbf).contains(&w[3]))
        .count()
}

/// Whether the emoji takes a skin tone.
fn takes_skin_tone(a: wchar_t) -> bool {
    matches!(
        a,
        128075
            | 128076
            | 128077
            | 128078
            | 128079
            | 128080
            | 128102
            | 128103
            | 128104
            | 128105
            | 128110
            | 128112
            | 128113
            | 128114
            | 128115
            | 128116
            | 128117
            | 128118
            | 128119
            | 128120
            | 128124
            | 128129
            | 128130
            | 128131
            | 128133
            | 128134
            | 128135
            | 128170
            | 128373
            | 128378
            | 128400
            | 128405
            | 128406
            | 128581
            | 128582
            | 128583
            | 128587
            | 128588
            | 128589
            | 128590
            | 128591
            | 128692
            | 128693
            | 128694
            | 129318
            | 129335
            | 129336
            | 129337
            | 129341
            | 129342
            | 129461
            | 129462
            | 129464
            | 129465
            | 129485
            | 129486
            | 129487
            | 129489
            | 129490
            | 129491
            | 129492
            | 129493
            | 129494
            | 129495
            | 129496
            | 129497
            | 129498
            | 129499
            | 129500
            | 129501
            | 129502
            | 129503
    )
}

/// Whether `add` written after `with` makes one character rather than two.
///
/// Two things join: a pair of regional indicators, so long as neither side is
/// already a finished flag, and an emoji that takes a skin tone written after
/// the tone itself. The list is read against what is being *added* and the
/// tone against what is already there, so the pair only joins in that order.
pub fn utf8_should_combine(with: &utf8_data, add: &utf8_data) -> bool {
    let Some(w) = towc(with) else {
        return false;
    };
    let Some(a) = towc(add) else {
        return false;
    };
    if REGIONAL.contains(&a) && REGIONAL.contains(&w) {
        return regional_count(with) == 1 && regional_count(add) == 1;
    }
    takes_skin_tone(a) && SKIN_TONE.contains(&w)
}

/// Which part of a Hangul syllable a jamo is.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Jamo {
    /// A leading consonant, which starts a syllable.
    Choseong,
    /// A vowel, which follows a leading consonant.
    Jungseong,
    /// A trailing consonant, which follows a vowel.
    Jongseong,
}

/// Which part of a syllable the three bytes `s` spell, or nothing when they
/// spell no jamo at all.
///
/// The C answered one of eleven subclasses — modern, old, and the extended
/// blocks — and then folded them onto these three parts; nothing else read the
/// subclass, so the fold is done here. The ranges below are those subclasses
/// run together: U+1100 to U+115F are leading consonants (the modern ones up
/// to U+1112, the old ones and the choseong filler after it), U+1160 to
/// U+11A7 vowels, U+11A8 to U+11FF trailing consonants, U+A960 to U+A97C
/// extended leading consonants, U+D7B0 to U+D7C6 extended vowels and U+D7CB to
/// U+D7FB extended trailing consonants.
fn jamo(s: &[u8]) -> Option<Jamo> {
    match (s[0], s[1], s[2]) {
        (0xe1, 0x84, 0x80..=0xbf) | (0xe1, 0x85, 0x80..=0x9f) | (0xea, 0xa5, 0xa0..=0xbc) => {
            Some(Jamo::Choseong)
        }
        (0xe1, 0x85, 0xa0..=0xbf)
        | (0xe1, 0x86, 0x80..=0xa7)
        | (0xed, 0x9e, 0xb0..=0xbf)
        | (0xed, 0x9f, 0x80..=0x86) => Some(Jamo::Jungseong),
        (0xe1, 0x86, 0xa8..=0xbf) | (0xe1, 0x87, 0x80..=0xbf) | (0xed, 0x9f, 0x8b..=0xbb) => {
            Some(Jamo::Jongseong)
        }
        _ => None,
    }
}

/// Where `ud` stands in a Hangul syllable that ends with `p_ud`: it starts
/// one, it joins the one already there, it is a jamo that joins nothing, or it
/// is no jamo at all. The jamo already there is read out of the *last* three
/// bytes of `p_ud`, so a character built up from several joins still
/// composes.
pub fn hanguljamo_check_state(p_ud: &utf8_data, ud: &utf8_data) -> hanguljamo_state {
    let s = bytes(ud);
    if s.len() != 3 {
        return HANGULJAMO_STATE_NOT_HANGULJAMO;
    }
    let wanted = match jamo(s) {
        None => return HANGULJAMO_STATE_NOT_HANGULJAMO,
        Some(Jamo::Choseong) => return HANGULJAMO_STATE_CHOSEONG,
        Some(Jamo::Jungseong) => Jamo::Choseong,
        Some(Jamo::Jongseong) => Jamo::Jungseong,
    };
    let p = bytes(p_ud);
    if p.len() < 3 {
        return HANGULJAMO_STATE_NOT_COMPOSABLE;
    }
    if jamo(&p[p.len() - 3..]) == Some(wanted) {
        return HANGULJAMO_STATE_COMPOSABLE;
    }
    HANGULJAMO_STATE_NOT_COMPOSABLE
}

#[cfg(test)]
#[path = "../tests/test_utf8_combined.rs"]
mod tests;

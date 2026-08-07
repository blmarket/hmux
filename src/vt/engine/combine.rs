//! Which characters join the cell to their left, and which are dropped.
//!
//! This is tmux's `utf8-combined.c`. Terminals have to decide, without a
//! shaping engine, when a codepoint is a continuation of the grapheme already
//! on screen rather than a new one: a zero-width joiner, a variation selector,
//! an emoji skin-tone modifier, a flag's second regional indicator, or a Hangul
//! jamo that syllable-composes with the jamo before it. tmux answers with the
//! rules below, and the answer is observable through `capture-pane` and the
//! cursor column, so we answer the same way.
//!
//! tmux's predicates read the raw UTF-8 of a cell's cluster, and the Hangul
//! ones deliberately look at the *last three bytes* of the previous cluster
//! rather than at its last codepoint. That distinction is preserved here: the
//! classification below is byte-level, exactly as `hanguljamo_get_subclass` is.

/// U+200D ZERO WIDTH JOINER.
const ZWJ: &str = "\u{200d}";
/// U+FE0F VARIATION SELECTOR-16.
const VS16: &str = "\u{fe0f}";
/// U+3164 HANGUL FILLER.
const HANGUL_FILLER: &str = "\u{3164}";

/// tmux's `utf8_is_zwj`.
pub(super) fn is_zwj(cluster: &str) -> bool {
    cluster == ZWJ
}

/// tmux's `utf8_is_vs`.
pub(super) fn is_vs(cluster: &str) -> bool {
    cluster == VS16
}

/// tmux's `utf8_is_hangul_filler`. tmux drops this codepoint on the floor
/// rather than giving it a cell.
pub(super) fn is_hangul_filler(cluster: &str) -> bool {
    cluster == HANGUL_FILLER
}

/// tmux's `utf8_has_zwj`: does the cluster end with a joiner, and so expect
/// whatever comes next to join it?
pub(super) fn has_zwj(cluster: &str) -> bool {
    cluster.as_bytes().ends_with(ZWJ.as_bytes())
}

/// tmux's `utf8_regional_count`: how many regional indicators the cluster's
/// bytes contain.
fn regional_count(cluster: &str) -> usize {
    let bytes = cluster.as_bytes();
    bytes
        .windows(4)
        .filter(|window| {
            window[0] == 0xf0
                && window[1] == 0x9f
                && window[2] == 0x87
                && (0xa6..=0xbf).contains(&window[3])
        })
        .count()
}

/// The first codepoint of a cluster, which is what tmux's `utf8_towc` yields
/// for a multi-codepoint one.
fn first_codepoint(cluster: &str) -> Option<u32> {
    cluster.chars().next().map(u32::from)
}

/// Emoji that take a skin-tone modifier, tmux's list in `utf8_should_combine`.
const SKIN_TONE_BASES: &[u32] = &[
    0x1F44B, 0x1F44C, 0x1F44D, 0x1F44E, 0x1F44F, 0x1F450, 0x1F466, 0x1F467, 0x1F468, 0x1F469,
    0x1F46E, 0x1F470, 0x1F471, 0x1F472, 0x1F473, 0x1F474, 0x1F475, 0x1F476, 0x1F477, 0x1F478,
    0x1F47C, 0x1F481, 0x1F482, 0x1F483, 0x1F485, 0x1F486, 0x1F487, 0x1F4AA, 0x1F575, 0x1F57A,
    0x1F590, 0x1F595, 0x1F596, 0x1F645, 0x1F646, 0x1F647, 0x1F64B, 0x1F64C, 0x1F64D, 0x1F64E,
    0x1F64F, 0x1F6B4, 0x1F6B5, 0x1F6B6, 0x1F926, 0x1F937, 0x1F938, 0x1F939, 0x1F93D, 0x1F93E,
    0x1F9B5, 0x1F9B6, 0x1F9B8, 0x1F9B9, 0x1F9CD, 0x1F9CE, 0x1F9CF, 0x1F9D1, 0x1F9D2, 0x1F9D3,
    0x1F9D4, 0x1F9D5, 0x1F9D6, 0x1F9D7, 0x1F9D8, 0x1F9D9, 0x1F9DA, 0x1F9DB, 0x1F9DC, 0x1F9DD,
    0x1F9DE, 0x1F9DF,
];

/// tmux's `utf8_should_combine`.
///
/// Note the argument order tmux actually uses: `with` supplies the modifier and
/// `add` the emoji it modifies. `screen_write_combine` tries the pair both ways
/// round, which is how a skin tone combines whether it arrives before or after
/// the person it applies to.
pub(super) fn should_combine(with: &str, add: &str) -> bool {
    let (Some(w), Some(a)) = (first_codepoint(with), first_codepoint(add)) else {
        return false;
    };

    // Regional indicators: two of them make one flag, but only if neither side
    // is already a finished pair.
    if (0x1F1E6..=0x1F1FF).contains(&a) && (0x1F1E6..=0x1F1FF).contains(&w) {
        return regional_count(with) == 1 && regional_count(add) == 1;
    }

    // Emoji skin tone modifiers.
    if SKIN_TONE_BASES.contains(&a) {
        return (0x1F3FB..=0x1F3FF).contains(&w);
    }
    false
}

/// The three-way Hangul jamo classification tmux's `hanguljamo_get_class`
/// produces, with the subclasses folded in as tmux folds them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum JamoClass {
    NotHangulJamo,
    /// Leading consonant, U+1100–U+115F and U+A960–U+A97C.
    Choseong,
    /// Medial vowel, U+1160–U+11A7 and U+D7B0–U+D7C6.
    Jungseong,
    /// Trailing consonant, U+11A8–U+11FF and U+D7CB–U+D7FB.
    Jongseong,
}

/// tmux's `hanguljamo_get_subclass` and `hanguljamo_get_class` in one step,
/// classifying exactly three bytes of UTF-8.
fn jamo_class(bytes: &[u8]) -> JamoClass {
    if bytes.len() < 3 {
        return JamoClass::NotHangulJamo;
    }
    match (bytes[0], bytes[1], bytes[2]) {
        // U+1100–U+115F: choseong, old choseong and the choseong filler.
        (0xE1, 0x84, _) => JamoClass::Choseong,
        (0xE1, 0x85, third) if third <= 0x9F => JamoClass::Choseong,
        // U+1160–U+11A7: the jungseong filler, jungseong and old jungseong.
        (0xE1, 0x85, _) => JamoClass::Jungseong,
        (0xE1, 0x86, third) if third <= 0xA7 => JamoClass::Jungseong,
        // U+11A8–U+11FF: jongseong and old jongseong.
        (0xE1, 0x86, _) => JamoClass::Jongseong,
        (0xE1, 0x87, _) => JamoClass::Jongseong,
        // U+A960–U+A97C: extended old choseong.
        (0xEA, 0xA5, third) if (0xA0..=0xBC).contains(&third) => JamoClass::Choseong,
        // U+D7B0–U+D7C6: extended old jungseong.
        (0xED, 0x9E, third) if third >= 0xB0 => JamoClass::Jungseong,
        (0xED, 0x9F, third) if third <= 0x86 => JamoClass::Jungseong,
        // U+D7CB–U+D7FB: extended old jongseong.
        (0xED, 0x9F, third) if (0x8B..=0xBB).contains(&third) => JamoClass::Jongseong,
        _ => JamoClass::NotHangulJamo,
    }
}

/// tmux's `enum hanguljamo_state`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum JamoState {
    /// Neither character is a jamo; fall through to the other combining rules.
    NotHangulJamo,
    /// A leading consonant, which starts a new syllable in its own cell.
    Choseong,
    /// This jamo continues the syllable in the cell to the left.
    Composable,
    /// A jamo that cannot follow what is to its left. tmux discards it.
    NotComposable,
}

/// tmux's `hanguljamo_check_state`.
pub(super) fn jamo_state(previous: &str, incoming: &str) -> JamoState {
    // tmux only classifies a three-byte incoming character.
    if incoming.len() != 3 {
        return JamoState::NotHangulJamo;
    }
    let previous_tail = |previous: &str| {
        if previous.len() < 3 {
            None
        } else {
            Some(jamo_class(&previous.as_bytes()[previous.len() - 3..]))
        }
    };
    match jamo_class(incoming.as_bytes()) {
        JamoClass::Choseong => JamoState::Choseong,
        JamoClass::Jungseong => match previous_tail(previous) {
            Some(JamoClass::Choseong) => JamoState::Composable,
            Some(_) => JamoState::NotComposable,
            None => JamoState::NotComposable,
        },
        JamoClass::Jongseong => match previous_tail(previous) {
            Some(JamoClass::Jungseong) => JamoState::Composable,
            Some(_) => JamoState::NotComposable,
            None => JamoState::NotComposable,
        },
        JamoClass::NotHangulJamo => JamoState::NotHangulJamo,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_single_codepoint_predicates_only_match_themselves() {
        assert!(is_zwj("\u{200d}"));
        assert!(!is_zwj("a\u{200d}"));
        assert!(is_vs("\u{fe0f}"));
        assert!(!is_vs("\u{fe0e}"));
        assert!(is_hangul_filler("\u{3164}"));
    }

    #[test]
    fn a_trailing_joiner_is_what_invites_the_next_character_in() {
        assert!(has_zwj("\u{1f468}\u{200d}"));
        assert!(has_zwj("\u{200d}"));
        assert!(!has_zwj("\u{1f468}"));
        assert!(!has_zwj("a"));
    }

    #[test]
    fn two_regional_indicators_make_a_flag_but_a_third_does_not_join() {
        let first = "\u{1f1ec}"; // G
        let second = "\u{1f1e7}"; // B
        assert!(should_combine(first, second));
        // Once the pair is complete the cluster holds two indicators, and tmux
        // refuses to add a third, so the next flag starts its own cell.
        let pair = "\u{1f1ec}\u{1f1e7}";
        assert!(!should_combine(pair, first));
    }

    #[test]
    fn skin_tones_combine_in_either_arrival_order() {
        let person = "\u{1f469}";
        let tone = "\u{1f3fd}";
        // tmux's own argument order: the modifier is `with`.
        assert!(should_combine(tone, person));
        // The predicate itself is one-directional, which is precisely why
        // `screen_write_combine` calls it with the pair both ways round.
        assert!(!should_combine(person, tone));
        assert!(!should_combine(person, "a"));
    }

    #[test]
    fn jamo_compose_in_syllable_order() {
        let choseong = "\u{1100}"; // ᄀ
        let jungseong = "\u{1161}"; // ᅡ
        let jongseong = "\u{11a8}"; // ᆨ
        assert_eq!(jamo_state("", choseong), JamoState::Choseong);
        assert_eq!(jamo_state(choseong, jungseong), JamoState::Composable);
        assert_eq!(
            jamo_state(&format!("{choseong}{jungseong}"), jongseong),
            JamoState::Composable
        );
        // A vowel with no leading consonant to attach to is discarded.
        assert_eq!(jamo_state("a", jungseong), JamoState::NotComposable);
        assert_eq!(jamo_state("", jungseong), JamoState::NotComposable);
        // A trailing consonant needs a vowel, not another consonant.
        assert_eq!(jamo_state(choseong, jongseong), JamoState::NotComposable);
        assert_eq!(jamo_state("a", "b"), JamoState::NotHangulJamo);
    }

    #[test]
    fn the_jamo_class_boundaries_match_tmuxs_byte_ranges() {
        let class = |ch: char| {
            let mut buffer = [0u8; 4];
            jamo_class(ch.encode_utf8(&mut buffer).as_bytes())
        };
        assert_eq!(class('\u{1100}'), JamoClass::Choseong);
        assert_eq!(class('\u{115f}'), JamoClass::Choseong);
        assert_eq!(class('\u{1160}'), JamoClass::Jungseong);
        assert_eq!(class('\u{11a7}'), JamoClass::Jungseong);
        assert_eq!(class('\u{11a8}'), JamoClass::Jongseong);
        assert_eq!(class('\u{11ff}'), JamoClass::Jongseong);
        assert_eq!(class('\u{a960}'), JamoClass::Choseong);
        assert_eq!(class('\u{a97c}'), JamoClass::Choseong);
        assert_eq!(class('\u{a97d}'), JamoClass::NotHangulJamo);
        assert_eq!(class('\u{d7b0}'), JamoClass::Jungseong);
        assert_eq!(class('\u{d7c6}'), JamoClass::Jungseong);
        assert_eq!(class('\u{d7c7}'), JamoClass::NotHangulJamo);
        assert_eq!(class('\u{d7cb}'), JamoClass::Jongseong);
        assert_eq!(class('\u{d7fb}'), JamoClass::Jongseong);
        assert_eq!(class('\u{ac00}'), JamoClass::NotHangulJamo);
    }
}

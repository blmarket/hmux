use super::*;
use ::core::ffi::{CStr, c_int};

const REG_EXTENDED: c_int = 1;
const REG_ICASE: c_int = 1 << 1;

/// What `regsub` makes of `text`, or `None` when the pattern does not
/// compile.
fn sub(pattern: &CStr, with: &CStr, text: &CStr, flags: c_int) -> Option<String> {
    unsafe {
        regsub(pattern.as_ptr(), with.as_ptr(), text.as_ptr(), flags)
            .map(|value| value.to_string_lossy().into_owned())
    }
}

/// The same, for the extended syntax every caller asks for.
fn ere(pattern: &CStr, with: &CStr, text: &CStr) -> String {
    sub(pattern, with, text, REG_EXTENDED).expect("the pattern compiles")
}

#[test]
fn empty_text_answers_an_empty_string() {
    assert_eq!(ere(c"a", c"X", c""), "");
}

#[test]
fn an_empty_pattern_hands_the_text_back() {
    assert_eq!(ere(c"", c"X", c"abc"), "abc");
}

#[test]
fn a_pattern_that_does_not_compile_answers_nothing() {
    assert!(sub(c"(", c"X", c"abc", REG_EXTENDED).is_none());
}

#[test]
fn text_with_no_match_comes_back_unchanged() {
    assert_eq!(ere(c"z", c"X", c"abc"), "abc");
}

#[test]
fn every_match_is_replaced() {
    assert_eq!(ere(c"a", c"X", c"banana"), "bXnXnX");
    assert_eq!(ere(c"an", c"", c"banana"), "ba");
}

#[test]
fn a_match_at_the_end_leaves_nothing_behind() {
    assert_eq!(ere(c"c", c"X", c"abc"), "abX");
}

#[test]
fn case_is_ignored_when_asked() {
    assert_eq!(ere(c"a", c"X", c"AbA"), "AbA");
    assert_eq!(
        sub(c"a", c"X", c"AbA", REG_EXTENDED | REG_ICASE).expect("the pattern compiles"),
        "XbX"
    );
}

#[test]
fn a_backslashed_digit_stands_for_the_group_it_names() {
    assert_eq!(ere(c"(a)(b)", c"\\2\\1", c"zabz"), "zbaz");
    assert_eq!(ere(c"a(b)c", c"[\\0]", c"xabcx"), "x[abc]x");
}

#[test]
fn a_group_that_matched_nothing_leaves_its_digit_behind() {
    assert_eq!(ere(c"(a)|(b)", c"<\\2>", c"a"), "<2>");
    assert_eq!(ere(c"a", c"<\\9>", c"a"), "<9>");
}

#[test]
fn a_backslash_in_front_of_anything_else_is_dropped() {
    assert_eq!(ere(c"a", c"\\n", c"a"), "n");
    assert_eq!(ere(c"a", c"\\\\", c"a"), "\\");
}

#[test]
fn a_trailing_backslash_is_kept() {
    assert_eq!(ere(c"a", c"x\\", c"a"), "x\\");
}

/// The first empty match at a position is skipped rather than replaced:
/// the run that finds one moves on a byte with `empty` set, and only the
/// *second* look at that byte expands the replacement. So a pattern that
/// matches the empty string writes nothing in front of the first
/// character, and once at the end of the text.
#[test]
fn a_pattern_that_matches_nothing_at_all_writes_between_the_characters() {
    assert_eq!(ere(c"x*", c"-", c"abc"), "a-b-c-");
}

#[test]
fn an_empty_match_beside_a_real_one_still_replaces_the_real_one() {
    assert_eq!(ere(c"b*", c"-", c"ab"), "a-");
    assert_eq!(ere(c"b*", c"-", c"abab"), "a-a-");
}

/// A pattern anchored at the start is applied once; the rest of the text
/// is copied over as it stands, even where it would match again.
#[test]
fn an_anchored_pattern_is_replaced_once() {
    assert_eq!(ere(c"^a", c"X", c"aaa"), "Xaa");
    assert_eq!(ere(c"^z", c"X", c"aaa"), "aaa");
}

/// The tail an anchored pattern copies over starts where the *next* look
/// would, not where the match ended, so an anchored pattern whose first
/// match is empty eats the character behind it: the empty match steps a
/// byte on before the copy, and nothing writes that byte out.
#[test]
fn an_anchored_pattern_that_matches_nothing_eats_a_character() {
    assert_eq!(ere(c"^x*", c"-", c"abc"), "bc");
}

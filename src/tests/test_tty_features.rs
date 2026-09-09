use super::*;
use crate::terminfo::TerminalCapabilities;
use crate::tests::test_fixtures::{Tty, globals};
use ::core::ffi::{CStr, c_int};

/// The bit `tty_add_features` sets for each feature name, in the order the
/// table lists them.
const NAMES: [&CStr; 21] = [
    c"256",
    c"bpaste",
    c"ccolour",
    c"clipboard",
    c"hyperlinks",
    c"cstyle",
    c"extkeys",
    c"focus",
    c"ignorefkeys",
    c"margins",
    c"mouse",
    c"osc7",
    c"overline",
    c"progressbar",
    c"rectfill",
    c"RGB",
    c"sixel",
    c"strikethrough",
    c"sync",
    c"title",
    c"usstyle",
];

/// The feature bits `s` adds to `start`, read with `separators`.
fn add(start: c_int, s: &CStr, separators: &CStr) -> c_int {
    let mut feat = start;
    tty_add_features(&mut feat, s, separators);
    feat
}

/// The feature bits of a comma-separated list.
fn feat(s: &CStr) -> c_int {
    add(0, s, c",")
}

/// What `tty_get_features` names those bits.
fn names(feat: c_int) -> String {
    tty_get_features(feat).to_string_lossy().into_owned()
}

fn code(t: &Tty, name: &CStr) -> tty_code_code {
    t.term()
        .find_capability(name)
        .unwrap_or_else(|| panic!("terminal capability {name:?} exists"))
}

#[test]
fn each_feature_name_has_a_bit_of_its_own() {
    for (i, name) in NAMES.iter().enumerate() {
        assert_eq!(feat(name), 1 << i, "{name:?}");
    }
}

#[test]
fn a_feature_name_is_matched_without_regard_to_case() {
    assert_eq!(feat(c"rgb"), feat(c"RGB"));
    assert_eq!(feat(c"TiTlE"), feat(c"title"));
}

#[test]
fn a_list_adds_every_feature_it_names() {
    assert_eq!(
        feat(c"256,RGB,title"),
        feat(c"256") | feat(c"RGB") | feat(c"title")
    );
}

#[test]
fn a_feature_already_there_leaves_the_bits_alone() {
    let once = feat(c"mouse");
    assert_eq!(add(once, c"mouse", c","), once);
    assert_eq!(feat(c"mouse,mouse"), once);
}

#[test]
fn an_unknown_feature_stops_the_rest_of_the_list() {
    assert_eq!(feat(c"256,bogus,title"), feat(c"256"));
    assert_eq!(feat(c"bogus,title"), 0);
    assert_eq!(feat(c""), 0);
    assert_eq!(feat(c"256,,title"), feat(c"256"));
}

#[test]
fn any_of_the_separators_ends_a_name() {
    assert_eq!(add(0, c"256 RGB\ttitle", c" \t"), feat(c"256,RGB,title"));
    assert_eq!(add(0, c"256,RGB", c" "), 0);
}

#[test]
fn no_features_are_named_by_an_empty_set_of_bits() {
    let _guard = globals();
    assert_eq!(names(0), "");
}

#[test]
fn the_named_features_come_back_in_table_order() {
    let _guard = globals();
    assert_eq!(names(feat(c"title")), "title");
    assert_eq!(names(feat(c"title,256,RGB")), "256,RGB,title");
    let all = NAMES
        .iter()
        .map(|name| name.to_str().expect("ASCII"))
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(names(!0), all);
}

#[test]
fn no_features_at_all_leave_the_terminal_alone() {
    let _guard = globals();
    let mut t = Tty::new();
    assert!(!t.term_mut().apply_features(0));
    assert_eq!(t.term().flags(), 0);
}

#[test]
fn applying_a_feature_adds_its_capabilities_and_flags() {
    let _guard = globals();
    let mut t = Tty::new();
    assert!(t.term_mut().apply_features(feat(c"title")));
    assert!(t.term().has(code(&t, c"tsl")));
    assert_eq!(t.term().string(code(&t, c"tsl")), c"\u{1b}]0;");
    assert_eq!(t.term().string(code(&t, c"fsl")), c"\u{7}");
    assert!(!t.term_mut().apply_features(feat(c"title")));
    assert_eq!(t.term().flags(), 0);
}

#[test]
fn a_feature_carrying_terminal_flags_hands_them_to_the_terminal() {
    let _guard = globals();
    let mut t = Tty::new();
    assert!(t.term_mut().apply_features(feat(c"RGB,sixel")));
    assert_eq!(
        t.term().flags(),
        TERM_256COLOURS | TERM_RGBCOLOURS | TERM_SIXEL
    );
    assert!(t.term().has(code(&t, c"setrgbf")));
}

#[test]
fn a_capability_ending_in_an_at_sign_takes_the_capability_away() {
    let _guard = globals();
    let mut t = Tty::new();
    t.term_mut().apply_features(feat(c"mouse"));
    assert!(t.term().has(code(&t, c"kmous")));
    let kf1 = code(&t, c"kf1");
    t.set_string(kf1, c"kf1");
    t.term_mut().apply_features(feat(c"ignorefkeys"));
    assert!(!t.term().has(kf1));
}

#[test]
fn a_feature_the_terminal_already_has_is_not_applied_again() {
    let _guard = globals();
    let mut t = Tty::new();
    assert!(t.term_mut().apply_features(feat(c"overline")));
    let smol = code(&t, c"Smol");
    t.clear_code(smol);
    assert!(!t.term_mut().apply_features(feat(c"overline")));
    assert!(!t.term().has(smol));
    assert!(t.term_mut().apply_features(feat(c"overline,clipboard")));
    assert!(!t.term().has(smol));
    assert!(t.term().has(code(&t, c"Ms")));
}

#[test]
fn a_terminal_the_table_names_gets_the_features_listed_for_it() {
    let _guard = globals();
    let mut got = 0;
    tty_default_features(&mut got, c"tmux", 0);
    assert_eq!(
        names(got),
        "256,bpaste,ccolour,clipboard,hyperlinks,cstyle,extkeys,focus,mouse,overline,progressbar,RGB,strikethrough,title,usstyle"
    );
}

#[test]
fn every_terminal_in_the_table_names_features_that_exist() {
    for name in [
        c"mintty",
        c"tmux",
        c"rxvt-unicode",
        c"iTerm2",
        c"foot",
        c"WezTerm",
        c"XTerm",
    ] {
        let mut got = 0;
        tty_default_features(&mut got, name, 1);
        assert_ne!(got, 0, "{name:?}");
        assert_eq!(got & !((1 << NAMES.len()) - 1), 0, "{name:?}");
    }
}

#[test]
fn a_terminal_the_table_does_not_name_gets_nothing() {
    let mut got = 0;
    tty_default_features(&mut got, c"dumb", 0);
    assert_eq!(got, 0);
    tty_default_features(&mut got, c"TMUX", 0);
    assert_eq!(got, 0);
}

//! Isolated option-name regressions for set-option/show-options parsing.

use hmux2::src::ffi::libc::free;
use hmux2::src::options::{options_match, options_match_command};
use std::ffi::{CStr, CString};

#[derive(Debug, PartialEq, Eq)]
struct MatchOutcome {
    name: Option<Vec<u8>>,
    key: Option<Vec<u8>>,
    ambiguous: i32,
}

unsafe fn run_match(input: &[u8], abi_adapter: bool) -> MatchOutcome {
    let input = CString::new(input).expect("test input has no embedded NUL");
    let mut key = std::ptr::null_mut();
    let mut ambiguous = 0;
    let name = if abi_adapter {
        options_match(input.as_ptr(), &mut key, &mut ambiguous)
    } else {
        options_match_command(input.as_ptr(), &mut key, &mut ambiguous)
    };
    let outcome = MatchOutcome {
        name: (!name.is_null()).then(|| CStr::from_ptr(name).to_bytes().to_vec()),
        key: (!key.is_null()).then(|| CStr::from_ptr(key).to_bytes().to_vec()),
        ambiguous,
    };
    if !name.is_null() {
        free(name.cast());
    }
    if !key.is_null() {
        free(key.cast());
    }
    outcome
}

const CASES: &[(&[u8], Option<&[u8]>, Option<&[u8]>, i32)] = &[
    (b"status", Some(b"status"), None, 0),
    (b"mous", Some(b"mouse"), None, 0),
    (b"pane-colors", Some(b"pane-colours"), None, 0),
    (
        b"status-format[0007]",
        Some(b"status-format"),
        Some(b"7"),
        0,
    ),
    (b"@test[hook]", Some(b"@test"), Some(b"hook"), 0),
    (b"[key]", None, None, 1),
    (b"status-", None, None, 1),
    (b"status[]", None, None, 0),
    (b"status-format[4294967296]", None, None, 0),
    (b"status-format[0]tail", None, None, 0),
    (b"not-an-option", None, None, 0),
];

fn assert_cases(abi_adapter: bool) {
    for &(input, name, key, ambiguous) in CASES {
        let actual = unsafe { run_match(input, abi_adapter) };
        assert_eq!(
            actual,
            MatchOutcome {
                name: name.map(<[u8]>::to_vec),
                key: key.map(<[u8]>::to_vec),
                ambiguous,
            },
            "input={input:?}"
        );
    }
}

#[test]
fn set_option_command_adapter_cases() {
    assert_cases(false);
}

#[test]
fn show_options_command_adapter_cases() {
    assert_cases(false);
}

#[test]
fn options_match_abi_adapter_matches_command_adapter() {
    for &(input, ..) in CASES {
        assert_eq!(
            unsafe { run_match(input, false) },
            unsafe { run_match(input, true) },
            "input={input:?}"
        );
    }
}

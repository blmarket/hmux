//! Coverage for [`crate::cmd`] – constants and pure helpers.

use crate::cmd::{
    CMD_PARSE_COMMANDS, CMD_PARSE_ERROR, CMD_PARSE_MAX_ENVIRON_LEN, CMD_PARSE_NOALIAS,
    CMD_PARSE_ONEGROUP, CMD_PARSE_PARSED_COMMANDS, CMD_PARSE_PARSEONLY, CMD_PARSE_STRING,
    CMD_PARSE_SUCCESS, CMD_PARSE_VERBOSE, DOUBLE_QUOTES, NONE, SINGLE_QUOTES, START,
    cmd_parse_state,
};

// ---------------------------------------------------------------------------
// constants: status / args / argument types
// ---------------------------------------------------------------------------

#[test]
fn cmd_parse_status_constants_are_distinct() {
    assert_eq!(CMD_PARSE_ERROR, 0);
    assert_eq!(CMD_PARSE_SUCCESS, 1);
    assert_ne!(CMD_PARSE_SUCCESS, CMD_PARSE_ERROR);
}

#[test]
fn cmd_parse_argument_type_constants_match_expected() {
    assert_eq!(CMD_PARSE_STRING, 0);
    assert_eq!(CMD_PARSE_COMMANDS, 1);
    assert_eq!(CMD_PARSE_PARSED_COMMANDS, 2);
    // pairwise distinct
    assert_ne!(CMD_PARSE_STRING, CMD_PARSE_COMMANDS);
    assert_ne!(CMD_PARSE_COMMANDS, CMD_PARSE_PARSED_COMMANDS);
}

// ---------------------------------------------------------------------------
// flags: distinct power-of-two bits
// ---------------------------------------------------------------------------

#[test]
fn cmd_parse_flags_are_distinct_bits() {
    assert_eq!(CMD_PARSE_PARSEONLY, 0x2);
    assert_eq!(CMD_PARSE_NOALIAS, 0x4);
    assert_eq!(CMD_PARSE_VERBOSE, 0x8);
    assert_eq!(CMD_PARSE_ONEGROUP, 0x10);
    let flags = [
        CMD_PARSE_PARSEONLY,
        CMD_PARSE_NOALIAS,
        CMD_PARSE_VERBOSE,
        CMD_PARSE_ONEGROUP,
    ];
    for i in 0..flags.len() {
        for j in (i + 1)..flags.len() {
            assert_eq!(
                flags[i] & flags[j],
                0,
                "flags overlap {:#x} & {:#x}",
                flags[i],
                flags[j]
            );
        }
    }
    assert_eq!(
        CMD_PARSE_PARSEONLY | CMD_PARSE_NOALIAS | CMD_PARSE_VERBOSE | CMD_PARSE_ONEGROUP,
        0x1e
    );
    // combining does not collapse
    assert_ne!(CMD_PARSE_PARSEONLY | CMD_PARSE_VERBOSE, CMD_PARSE_PARSEONLY);
}

#[test]
fn max_environ_len_has_expected_value() {
    assert_eq!(CMD_PARSE_MAX_ENVIRON_LEN, 16384);
    assert!(CMD_PARSE_MAX_ENVIRON_LEN > 0);
}

// ---------------------------------------------------------------------------
// token state constants
// ---------------------------------------------------------------------------

#[test]
fn token_state_constants_ordered() {
    assert_eq!(START, 0);
    assert_eq!(NONE, 1);
    assert_eq!(DOUBLE_QUOTES, 2);
    assert_eq!(SINGLE_QUOTES, 3);
    assert!(START < NONE);
    assert!(NONE < DOUBLE_QUOTES);
    assert!(DOUBLE_QUOTES < SINGLE_QUOTES);
}

// ---------------------------------------------------------------------------
// pure helper: Default for cmd_parse_state
// ---------------------------------------------------------------------------

#[test]
fn cmd_parse_state_default_is_zeroed() {
    let st = cmd_parse_state::default();
    assert!(st.f.is_none());
    assert!(st.buf.is_null());
    assert_eq!(st.len, 0);
    assert_eq!(st.off, 0);
    assert_eq!(st.condition, 0);
    assert_eq!(st.eol, 0);
    assert_eq!(st.eof, 0);
    assert!(st.input.is_null());
    assert_eq!(st.escapes, 0);
    assert!(st.error.is_none());
}

#[test]
fn cmd_parse_state_default_is_cloneable() {
    let a = cmd_parse_state::default();
    let b = a.clone();
    let c = b.clone();
    assert_eq!(a.len, c.len);
    assert_eq!(a.off, c.off);
    assert!(c.f.is_none());
}

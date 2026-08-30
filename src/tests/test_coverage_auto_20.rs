//! Coverage for [`crate::format`] — constants and pure helpers.
//!
//! `format.rs` exposes a long ladder of `pub const` flag and limit values plus
//! the [`format_expand_state`] default that can be exercised without a live
//! server. All tests are deterministic and stay clear of the `fatal` paths.

use crate::format::{
    FORMAT_BASENAME, FORMAT_CHARACTER, FORMAT_CLIENTS, FORMAT_COLOUR, FORMAT_DIRNAME,
    FORMAT_EXPAND, FORMAT_EXPAND_NOJOBS, FORMAT_EXPAND_TIME, FORMAT_EXPANDTIME, FORMAT_FORCE,
    FORMAT_LAST, FORMAT_LENGTH, FORMAT_LITERAL, FORMAT_LOOP_LIMIT, FORMAT_MAX_PRECISION,
    FORMAT_MAX_REPEAT, FORMAT_MAX_WIDTH, FORMAT_NOJOBS, FORMAT_NONE, FORMAT_NOT, FORMAT_NOT_NOT,
    FORMAT_PANE, FORMAT_PANES, FORMAT_PRETTY, FORMAT_QUOTE_ARGUMENTS, FORMAT_QUOTE_SHELL,
    FORMAT_QUOTE_STYLE, FORMAT_REPEAT, FORMAT_SESSION_NAME, FORMAT_SESSIONS, FORMAT_STATUS,
    FORMAT_TIME_LIMIT, FORMAT_TIMESTRING, FORMAT_TYPE_PANE, FORMAT_TYPE_SESSION,
    FORMAT_TYPE_UNKNOWN, FORMAT_TYPE_WINDOW, FORMAT_VERBOSE, FORMAT_WIDTH, FORMAT_WINDOW,
    FORMAT_WINDOW_NAME, FORMAT_WINDOWS, SORT_ACTIVITY, SORT_CREATION, SORT_END, SORT_INDEX,
    format_expand_state,
};

// ---------------------------------------------------------------------------
// format type constants
// ---------------------------------------------------------------------------

#[test]
fn format_type_constants_are_ordered_and_distinct() {
    assert_eq!(FORMAT_TYPE_UNKNOWN, 0);
    assert_eq!(FORMAT_TYPE_SESSION, 1);
    assert_eq!(FORMAT_TYPE_WINDOW, 2);
    assert_eq!(FORMAT_TYPE_PANE, 3);
    assert!(FORMAT_TYPE_UNKNOWN < FORMAT_TYPE_SESSION);
    assert!(FORMAT_TYPE_SESSION < FORMAT_TYPE_WINDOW);
    assert!(FORMAT_TYPE_WINDOW < FORMAT_TYPE_PANE);
    assert_ne!(FORMAT_TYPE_PANE, FORMAT_TYPE_WINDOW);
}

// ---------------------------------------------------------------------------
// status / control flags — distinct power-of-two bits
// ---------------------------------------------------------------------------

#[test]
fn format_status_flags_are_distinct_bits() {
    assert_eq!(FORMAT_NONE, 0);
    assert_eq!(FORMAT_STATUS, 0x1);
    assert_eq!(FORMAT_FORCE, 0x2);
    assert_eq!(FORMAT_NOJOBS, 0x4);
    assert_eq!(FORMAT_VERBOSE, 0x8);
    assert_eq!(FORMAT_LAST, 0x10);
    let flags = [
        FORMAT_STATUS,
        FORMAT_FORCE,
        FORMAT_NOJOBS,
        FORMAT_VERBOSE,
        FORMAT_LAST,
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
        FORMAT_STATUS | FORMAT_FORCE | FORMAT_NOJOBS | FORMAT_VERBOSE | FORMAT_LAST,
        0x1f
    );
}

#[test]
fn format_pane_window_high_bits_are_distinct() {
    assert_eq!(FORMAT_WINDOW, 0x40000000);
    assert_eq!(FORMAT_PANE, 0x80000000);
    assert_ne!(FORMAT_WINDOW, FORMAT_PANE);
    assert_eq!(FORMAT_WINDOW & FORMAT_PANE, 0);
    // they occupy the two high bits of a 32-bit word
    assert!(FORMAT_PANE > FORMAT_WINDOW);
}

// ---------------------------------------------------------------------------
// max limits
// ---------------------------------------------------------------------------

#[test]
fn format_max_limits_have_expected_values() {
    assert_eq!(FORMAT_MAX_WIDTH, 10000);
    assert_eq!(FORMAT_MAX_REPEAT, 10000);
    assert_eq!(FORMAT_MAX_PRECISION, 100);
    assert!(FORMAT_MAX_WIDTH > FORMAT_MAX_PRECISION);
    assert_eq!(FORMAT_MAX_WIDTH, FORMAT_MAX_REPEAT);
    assert!(FORMAT_LOOP_LIMIT > 0);
    assert!(FORMAT_TIME_LIMIT > 0);
    assert_eq!(FORMAT_LOOP_LIMIT, 100);
    assert_eq!(FORMAT_TIME_LIMIT, 100);
}

// ---------------------------------------------------------------------------
// modifier flags — low word 0x1..0x200 etc, each a single bit
// ---------------------------------------------------------------------------

#[test]
fn format_modifier_low_flags_are_distinct_bits() {
    assert_eq!(FORMAT_TIMESTRING, 0x1);
    assert_eq!(FORMAT_BASENAME, 0x2);
    assert_eq!(FORMAT_DIRNAME, 0x4);
    assert_eq!(FORMAT_QUOTE_SHELL, 0x8);
    assert_eq!(FORMAT_LITERAL, 0x10);
    assert_eq!(FORMAT_EXPAND, 0x20);
    assert_eq!(FORMAT_EXPANDTIME, 0x40);
    assert_eq!(FORMAT_SESSIONS, 0x80);
    assert_eq!(FORMAT_WINDOWS, 0x100);
    assert_eq!(FORMAT_PANES, 0x200);
    let low = [
        FORMAT_TIMESTRING,
        FORMAT_BASENAME,
        FORMAT_DIRNAME,
        FORMAT_QUOTE_SHELL,
        FORMAT_LITERAL,
        FORMAT_EXPAND,
        FORMAT_EXPANDTIME,
        FORMAT_SESSIONS,
        FORMAT_WINDOWS,
        FORMAT_PANES,
    ];
    for i in 0..low.len() {
        for j in (i + 1)..low.len() {
            assert_eq!(
                low[i] & low[j],
                0,
                "low flags overlap {:#x} & {:#x}",
                low[i],
                low[j]
            );
        }
    }
}

#[test]
fn format_modifier_high_flags_are_distinct_bits() {
    assert_eq!(FORMAT_PRETTY, 0x400);
    assert_eq!(FORMAT_LENGTH, 0x800);
    assert_eq!(FORMAT_WIDTH, 0x1000);
    assert_eq!(FORMAT_QUOTE_STYLE, 0x2000);
    assert_eq!(FORMAT_WINDOW_NAME, 0x4000);
    assert_eq!(FORMAT_SESSION_NAME, 0x8000);
    assert_eq!(FORMAT_CHARACTER, 0x10000);
    assert_eq!(FORMAT_COLOUR, 0x20000);
    assert_eq!(FORMAT_CLIENTS, 0x40000);
    assert_eq!(FORMAT_NOT, 0x80000);
    assert_eq!(FORMAT_NOT_NOT, 0x100000);
    assert_eq!(FORMAT_REPEAT, 0x200000);
    assert_eq!(FORMAT_QUOTE_ARGUMENTS, 0x400000);
    let high = [
        FORMAT_PRETTY,
        FORMAT_LENGTH,
        FORMAT_WIDTH,
        FORMAT_QUOTE_STYLE,
        FORMAT_WINDOW_NAME,
        FORMAT_SESSION_NAME,
        FORMAT_CHARACTER,
        FORMAT_COLOUR,
        FORMAT_CLIENTS,
        FORMAT_NOT,
        FORMAT_NOT_NOT,
        FORMAT_REPEAT,
        FORMAT_QUOTE_ARGUMENTS,
    ];
    for i in 0..high.len() {
        for j in (i + 1)..high.len() {
            assert_eq!(
                high[i] & high[j],
                0,
                "high flags overlap {:#x} & {:#x}",
                high[i],
                high[j]
            );
        }
    }
    // high and low words do not overlap
    assert_eq!(FORMAT_TIMESTRING & FORMAT_PRETTY, 0);
    assert_eq!(FORMAT_PANES & FORMAT_PRETTY, 0);
}

#[test]
fn format_expand_flags_are_distinct_bits() {
    assert_eq!(FORMAT_EXPAND_TIME, 0x1);
    assert_eq!(FORMAT_EXPAND_NOJOBS, 0x2);
    assert_eq!(FORMAT_EXPAND_TIME & FORMAT_EXPAND_NOJOBS, 0);
    assert_eq!(FORMAT_EXPAND_TIME | FORMAT_EXPAND_NOJOBS, 0x3);
}

// ---------------------------------------------------------------------------
// sort order constants
// ---------------------------------------------------------------------------

#[test]
fn sort_order_constants_are_consecutive_from_zero() {
    assert_eq!(SORT_ACTIVITY, 0);
    assert_eq!(SORT_CREATION, 1);
    assert_eq!(SORT_INDEX, 2);
    assert_eq!(SORT_END, 8);
    assert!(SORT_ACTIVITY < SORT_CREATION);
    assert!(SORT_CREATION < SORT_INDEX);
    assert!(SORT_INDEX < SORT_END);
}

// ---------------------------------------------------------------------------
// pure helper: Default for format_expand_state
// ---------------------------------------------------------------------------

#[test]
fn format_expand_state_default_is_zeroed() {
    let st = format_expand_state::default();
    assert!(st.ft.is_null());
    assert_eq!(st.loop_0, 0);
    assert_eq!(st.start_time, 0);
    assert_eq!(st.flags, 0);
    assert_eq!(st.time, 0);
    assert_eq!(st.tm.tm_sec, 0);
    assert_eq!(st.tm.tm_min, 0);
    assert_eq!(st.tm.tm_hour, 0);
}

#[test]
fn format_expand_state_default_is_copy_and_clone() {
    let a = format_expand_state::default();
    let b = a;
    let c = b;
    assert_eq!(a.loop_0, c.loop_0);
    assert_eq!(a.flags, c.flags);
    assert!(c.ft.is_null());
    assert_eq!(c.start_time, 0);
}

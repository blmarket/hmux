//! Coverage for [`crate::compat`] — edge probes beyond the
//! inline suite in `strtonum.rs`.
//!
//! `strtonum` is already at high coverage; these tests target residual
//! branches: whitespace-only / sign-only inputs, hex-like prefixes,
//! single-valued ranges, exact boundary adjacency, leading-zero handling,
//! trailing whitespace / newline rejection, and `errno` preservation on
//! the success path versus `EINVAL`/`ERANGE` on the error path. They
//! also exercise the `errstrp == NULL` path that the library allows.
//! A lighter `getopt_long` constant check is included to document that
//! `BADCH`/`INORDER`/`FLAG_*` values stayed stable, without duplicating
//! the heavy `getopt_internal` harness already present in that file.

use crate::compat::{EINVAL, ERANGE, LLONG_MAX, LLONG_MIN, strtonum};
use crate::ffi::__errno_location;
use ::core::ffi::CStr;

struct Outcome {
    value: i64,
    errstr: Option<String>,
    errno: i32,
}

fn call(numstr: &CStr, minval: i64, maxval: i64, preset: i32) -> Outcome {
    unsafe {
        *__errno_location() = preset;
        let result = strtonum(numstr.as_ptr(), minval, maxval);
        Outcome {
            value: result.unwrap_or(0),
            errstr: result
                .err()
                .map(|errstr| errstr.to_string_lossy().into_owned()),
            errno: *__errno_location(),
        }
    }
}

#[test]
fn whitespace_only_and_sign_only_are_invalid() {
    for s in [c" ", c"   ", c"\t", c"+", c"-", c"  +", c"  -"] {
        let r = call(s, -100, 100, 7);
        assert_eq!(r.value, 0, "input {s:?} should fail");
        assert_eq!(r.errstr.as_deref(), Some("invalid"), "input {s:?}");
        assert_eq!(r.errno, EINVAL, "input {s:?}");
    }
}

#[test]
fn hex_prefix_and_trailing_whitespace_are_invalid() {
    // strtonum uses base 10 only; "0x…" must not consume as 0
    for s in [c"0x10", c"0X10", c"42 ", c"42\t", c"42\n", c" 42\n"] {
        let r = call(s, 0, 1000, 7);
        assert_eq!(r.value, 0, "input {s:?} should be invalid");
        assert_eq!(r.errstr.as_deref(), Some("invalid"), "input {s:?}");
        assert_eq!(r.errno, EINVAL, "input {s:?}");
    }
}

#[test]
fn leading_zeros_and_negative_zero_are_valid() {
    let r = call(c"00042", 0, 100, 99);
    assert_eq!(r.value, 42);
    assert_eq!(r.errstr, None);
    assert_eq!(r.errno, 99);

    let r = call(c"  00000", 0, 0, 5);
    assert_eq!(r.value, 0);
    assert_eq!(r.errstr, None);
    assert_eq!(r.errno, 5);

    let r = call(c"-0", -10, 10, 11);
    assert_eq!(r.value, 0);
    assert_eq!(r.errstr, None);
    assert_eq!(r.errno, 11);

    let r = call(c"+0", -10, 10, 11);
    assert_eq!(r.value, 0);
    assert_eq!(r.errstr, None);
    assert_eq!(r.errno, 11);
}

#[test]
fn single_valued_range_accepts_only_exact() {
    let ok = call(c"42", 42, 42, 7);
    assert_eq!(ok.value, 42);
    assert_eq!(ok.errstr, None);
    assert_eq!(ok.errno, 7);

    let low = call(c"41", 42, 42, 7);
    assert_eq!(low.value, 0);
    assert_eq!(low.errstr.as_deref(), Some("too small"));
    assert_eq!(low.errno, ERANGE);

    let high = call(c"43", 42, 42, 7);
    assert_eq!(high.value, 0);
    assert_eq!(high.errstr.as_deref(), Some("too large"));
    assert_eq!(high.errno, ERANGE);

    // single-valued range at LLONG_MIN
    let edge = call(c"-9223372036854775808", LLONG_MIN, LLONG_MIN, 9);
    assert_eq!(edge.value, LLONG_MIN);
    assert_eq!(edge.errstr, None);
    assert_eq!(edge.errno, 9);
}

#[test]
fn one_beyond_minmax_reports_out_of_range() {
    let r = call(c"99", 100, 200, 7);
    assert_eq!(r.errstr.as_deref(), Some("too small"));
    assert_eq!(r.errno, ERANGE);

    let r = call(c"201", 100, 200, 7);
    assert_eq!(r.errstr.as_deref(), Some("too large"));
    assert_eq!(r.errno, ERANGE);

    // just inside the window is fine and preserves errno
    let r = call(c"100", 100, 200, 7);
    assert_eq!(r.value, 100);
    assert_eq!(r.errstr, None);
    assert_eq!(r.errno, 7);
    let r = call(c"200", 100, 200, 7);
    assert_eq!(r.value, 200);
    assert_eq!(r.errstr, None);
    assert_eq!(r.errno, 7);
}

#[test]
fn errno_preserved_on_success_and_set_on_failure() {
    // success preserves preset
    for preset in [0, 7, 22, 99] {
        let r = call(c"10", 0, 100, preset);
        assert_eq!(r.value, 10);
        assert_eq!(r.errstr, None);
        assert_eq!(r.errno, preset);
    }
    // invalid sets EINVAL, too small/large set ERANGE regardless of preset
    let r = call(c"nope", 0, 100, 99);
    assert_eq!(r.errno, EINVAL);
    let r = call(c"-5", 0, 100, 99);
    assert_eq!(r.errno, ERANGE);
    let r = call(c"500", 0, 100, 99);
    assert_eq!(r.errno, ERANGE);

    // a discarded message still leaves errno set
    unsafe {
        *__errno_location() = 55;
        let v = strtonum(c"nope".as_ptr(), 0, 100).unwrap_or(0);
        assert_eq!(v, 0);
        assert_eq!(*__errno_location(), EINVAL);
        *__errno_location() = 55;
        let v = strtonum(c"10".as_ptr(), 0, 100).unwrap_or(0);
        assert_eq!(v, 10);
        assert_eq!(*__errno_location(), 55);
    }
}

#[test]
fn partial_consume_with_embedded_garbage_is_invalid() {
    for s in [c"12x", c"12.0", c"1,000", c"--1", c"++1", c"1-", c" 1 1"] {
        let r = call(s, -1000, 1000, 7);
        assert_eq!(r.errstr.as_deref(), Some("invalid"), "input {s:?}");
        assert_eq!(r.errno, EINVAL, "input {s:?}");
    }
}

#[test]
fn adjacent_to_llong_limits_and_overflow() {
    // one beyond the exact limits is out of the caller range, not strtoll overflow;
    // caller sees "too small"/"too large" based on min/max comparison.
    let r = call(c"-9223372036854775809", LLONG_MIN, LLONG_MAX, 7);
    assert_eq!(r.errstr.as_deref(), Some("too small"));
    assert_eq!(r.errno, ERANGE);

    let r = call(c"9223372036854775808", LLONG_MIN, LLONG_MAX, 7);
    assert_eq!(r.errstr.as_deref(), Some("too large"));
    assert_eq!(r.errno, ERANGE);

    // far overflow still reports correctly
    let r = call(c"999999999999999999999999", -100, 100, 7);
    assert_eq!(r.errstr.as_deref(), Some("too large"));
    assert_eq!(r.errno, ERANGE);

    let r = call(c"-999999999999999999999999", -100, 100, 7);
    assert_eq!(r.errstr.as_deref(), Some("too small"));
    assert_eq!(r.errno, ERANGE);

    // narrow window that excludes the overflow value entirely
    let r = call(c"9223372036854775807", 0, 9223372036854775806, 7);
    assert_eq!(r.errstr.as_deref(), Some("too large"));
}

#[test]
fn getopt_long_constants_are_stable() {
    // Cheap smoke to show the companion pure helper kept its values;
    // the heavy getopt_internal suite lives in compat::getopt_long::tests.
    use crate::compat::{BADCH, FLAG_ALLARGS, FLAG_LONGONLY, FLAG_PERMUTE, INORDER};
    assert_eq!(BADCH, '?' as i32);
    assert_eq!(INORDER, 1);
    assert_eq!(FLAG_PERMUTE, 0x1);
    assert_eq!(FLAG_ALLARGS, 0x2);
    assert_eq!(FLAG_LONGONLY, 0x4);
    assert_ne!(FLAG_PERMUTE, FLAG_ALLARGS);
    assert_eq!(FLAG_PERMUTE | FLAG_ALLARGS | FLAG_LONGONLY, 0x7);
}

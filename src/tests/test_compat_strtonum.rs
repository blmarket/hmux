use super::*;
use ::core::ffi::CStr;

struct Outcome {
    value: ::core::ffi::c_longlong,
    errstr: Option<String>,
    errno: ::core::ffi::c_int,
}

fn call(numstr: &CStr, minval: i64, maxval: i64, preset: ::core::ffi::c_int) -> Outcome {
    unsafe {
        *__errno_location() = preset;
        let result = strtonum(numstr.as_ptr(), minval, maxval);
        Outcome {
            value: result.unwrap_or(0),
            errstr: result
                .err()
                .map(|errstr| errstr.to_str().unwrap().to_owned()),
            errno: *__errno_location(),
        }
    }
}

#[test]
fn parses_a_number_in_range_and_restores_errno() {
    let r = call(c"42", 0, 100, 7);
    assert_eq!(r.value, 42);
    assert_eq!(r.errstr, None);
    assert_eq!(r.errno, 7);
}

#[test]
fn accepts_leading_whitespace_and_a_sign() {
    assert_eq!(call(c"  +42", 0, 100, 0).value, 42);
    assert_eq!(call(c"-42", -100, 100, 0).value, -42);
}

#[test]
fn inverted_bounds_are_invalid() {
    let r = call(c"42", 100, 0, 7);
    assert_eq!(r.value, 0);
    assert_eq!(r.errstr.as_deref(), Some("invalid"));
    assert_eq!(r.errno, EINVAL);
}

#[test]
fn an_empty_string_is_invalid() {
    let r = call(c"", 0, 100, 7);
    assert_eq!(r.value, 0);
    assert_eq!(r.errstr.as_deref(), Some("invalid"));
    assert_eq!(r.errno, EINVAL);
}

#[test]
fn trailing_garbage_is_invalid() {
    let r = call(c"12x", 0, 100, 7);
    assert_eq!(r.value, 0);
    assert_eq!(r.errstr.as_deref(), Some("invalid"));
    assert_eq!(r.errno, EINVAL);
}

#[test]
fn a_value_below_the_minimum_is_too_small() {
    let r = call(c"-5", 0, 100, 7);
    assert_eq!(r.value, 0);
    assert_eq!(r.errstr.as_deref(), Some("too small"));
    assert_eq!(r.errno, ERANGE);
}

#[test]
fn a_value_above_the_maximum_is_too_large() {
    let r = call(c"200", 0, 100, 7);
    assert_eq!(r.value, 0);
    assert_eq!(r.errstr.as_deref(), Some("too large"));
    assert_eq!(r.errno, ERANGE);
}

#[test]
fn underflowing_the_long_long_range_is_too_small() {
    let r = call(c"-99999999999999999999999", LLONG_MIN, LLONG_MAX, 7);
    assert_eq!(r.value, 0);
    assert_eq!(r.errstr.as_deref(), Some("too small"));
    assert_eq!(r.errno, ERANGE);
}

#[test]
fn overflowing_the_long_long_range_is_too_large() {
    let r = call(c"99999999999999999999999", LLONG_MIN, LLONG_MAX, 7);
    assert_eq!(r.value, 0);
    assert_eq!(r.errstr.as_deref(), Some("too large"));
    assert_eq!(r.errno, ERANGE);
}

#[test]
fn the_exact_range_limits_parse_without_error() {
    let low = call(c"-9223372036854775808", LLONG_MIN, LLONG_MAX, 7);
    assert_eq!(low.value, LLONG_MIN);
    assert_eq!(low.errstr, None);
    assert_eq!(low.errno, 7);
    let high = call(c"9223372036854775807", LLONG_MIN, LLONG_MAX, 7);
    assert_eq!(high.value, LLONG_MAX);
    assert_eq!(high.errstr, None);
    assert_eq!(high.errno, 7);
}

/// A caller that only wants the number still leaves errno the way the
/// underlying function does.
#[test]
fn a_discarded_message_still_leaves_errno_set() {
    unsafe {
        *__errno_location() = 7;
        assert_eq!(strtonum(c"42".as_ptr(), 0, 100).unwrap_or(0), 42);
        assert_eq!(*__errno_location(), 7);
        assert_eq!(strtonum(c"nope".as_ptr(), 0, 100).unwrap_or(0), 0);
        assert_eq!(*__errno_location(), EINVAL);
    }
}

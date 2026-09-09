use super::*;
use crate::fmt_args;
use ::core::ffi::c_int;

#[test]
fn xasprintf_allocates_the_formatted_string() {
    let s = xasprintf(c"%s-%d", fmt_args![c"a".as_ptr(), 42 as c_int]);
    assert_eq!(s.as_bytes(), b"a-42");
}

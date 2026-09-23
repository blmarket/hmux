//! Exercise the && and || operand scratch lifetime through public expansion.
use hmux2::src::format::{format_create, format_expand, format_free};
use std::ffi::CStr;

#[test]
fn boolean_operands_expand_in_order_and_preserve_bytes() {
    unsafe {
        let tree = format_create(core::ptr::null_mut(), core::ptr::null_mut(), 0, 0);
        for (expression, expected) in [
            (b"#{&&:1,1,0}\0".as_slice(), b"0".as_slice()),
            (b"#{||:0,0,1}\0".as_slice(), b"1".as_slice()),
            (b"#{&&:1,#{||:0,1}}\0".as_slice(), b"1".as_slice()),
            (b"#{&&:1,}\0".as_slice(), b"0".as_slice()),
            (b"#{||:0,\xff}\0".as_slice(), b"1".as_slice()),
        ] {
            let expression = CStr::from_bytes_with_nul(expression).unwrap();
            let result = format_expand(tree, expression.as_ptr());
            assert_eq!(
                CStr::from_ptr(result).to_bytes(),
                expected,
                "{expression:?}"
            );
            libc::free(result.cast());
        }
        format_free(tree);
    }
}

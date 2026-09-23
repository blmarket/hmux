//! Exercise the expanded operands returned by format_choose through public formats.
use hmux2::src::format::{format_create, format_expand, format_free};
use hmux2::src::options::{options_create, options_free};
use hmux2::src::tmux::{global_options, global_s_options, global_w_options};
use std::ffi::CStr;

#[test]
fn compare_repeat_and_arithmetic_expand_split_operands() {
    unsafe {
        let saved_options = global_options;
        let saved_s_options = global_s_options;
        let saved_w_options = global_w_options;
        let options = options_create(std::ptr::null_mut());
        let s_options = options_create(std::ptr::null_mut());
        let w_options = options_create(std::ptr::null_mut());
        global_options = options;
        global_s_options = s_options;
        global_w_options = w_options;

        let tree = format_create(std::ptr::null_mut(), std::ptr::null_mut(), 0, 0);
        for (expression, expected) in [
            (b"#{==:a#,b,a#,b}\0".as_slice(), b"1".as_slice()),
            (b"#{==:#{e|+:1,2},3}\0", b"1"),
            (b"#{==:\xff,\xff}\0", b"1"),
            (b"#{R:a,3}\0", b"aaa"),
            (b"#{e|+:1,2}\0", b"3"),
            (b"prefix#{==:missing-comma}tail\0", b"prefix"),
            (b"prefix#{R:missing-comma}tail\0", b"prefix"),
            (b"#{R:a,0}\0", b""),
            (b"#{R:a,invalid}\0", b""),
            (b"#{e|+:missing-comma}\0", b""),
            (b"#{e|+:invalid,2}\0", b""),
            (b"#{e|+:1,invalid}\0", b""),
            (b"#{e|+:1,}\0", b"1"),
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
        global_options = saved_options;
        global_s_options = saved_s_options;
        global_w_options = saved_w_options;
        options_free(options);
        options_free(s_options);
        options_free(w_options);
    }
}

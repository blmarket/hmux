//! Exercise conditional scratch strings through public format expansion.
use hmux2::src::environ::{environ_create};
use hmux2::src::format::bytes::write_cstr;
use hmux2::src::format::{format_add, format_create, format_expand_cstring, format_free};
use hmux2::src::options::{options_create, options_free};
use hmux2::src::tmux::{global_environ, global_options, global_s_options, global_w_options};
use std::ffi::CStr;

#[test]
fn conditionals_expand_true_false_fallback_and_nested_branches() {
    unsafe {
        let saved_options = global_options;
        let saved_s_options = global_s_options;
        let saved_w_options = global_w_options;
        let saved_environ = global_environ.take();
        let mut options_owner = options_create(std::ptr::null_mut());
        let options = &raw mut *options_owner;
        let mut s_options_owner = options_create(std::ptr::null_mut());
        let s_options = &raw mut *s_options_owner;
        let mut w_options_owner = options_create(std::ptr::null_mut());
        let w_options = &raw mut *w_options_owner;
        global_options = options;
        global_s_options = s_options;
        global_w_options = w_options;
        let environ = environ_create();
        global_environ = Some(environ);

        let mut tree_owner = format_create(None, None, 0, 0);
        let tree = &raw mut *tree_owner;
        format_add(tree, c"yes".as_ptr(), |out| write_cstr(out, c"1".as_ptr()));
        format_add(tree, c"no".as_ptr(), |out| write_cstr(out, c"0".as_ptr()));

        for (expression, expected) in [
            (
                b"#{?yes,chosen,fallback}\0".as_slice(),
                b"chosen".as_slice(),
            ),
            (b"#{?no,wrong,fallback}\0", b"fallback"),
            (b"#{?no,wrong,yes,chosen,fallback}\0", b"chosen"),
            (b"#{?missing,wrong,fallback}\0", b"fallback"),
            (b"#{?no,wrong}\0", b""),
            (b"#{?yes,left#,right,fallback}\0", b"left,right"),
            (b"#{?yes,\xff,fallback}\0", b"\xff"),
            (b"#{?#{?yes,1,0},#{?no,wrong,nested},fallback}\0", b"nested"),
        ] {
            let expression = CStr::from_bytes_with_nul(expression).unwrap();
            let result = format_expand_cstring(tree, expression.as_ptr());
            assert_eq!(result.as_bytes(), expected, "{expression:?}");
        }

        format_free(tree_owner);
        global_options = saved_options;
        global_s_options = saved_s_options;
        global_w_options = saved_w_options;
        global_environ = saved_environ;
        options_free(options_owner);
        options_free(s_options_owner);
        options_free(w_options_owner);
    }
}

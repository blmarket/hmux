//! Environment iteration supports nested expansion and last-entry detection.
use hmux2::src::environ::{environ_create, environ_set};
use hmux2::src::format::bytes::write_cstr;
use hmux2::src::format::{format_create, format_expand_cstring, format_free};
use hmux2::src::options::{options_create, options_free};
use hmux2::src::tmux::{global_environ, global_options, global_s_options, global_w_options};

#[test]
fn environment_loops_support_nested_reads_and_last_entry_flags() {
    unsafe {
        let saved_options = global_options;
        let saved_s_options = global_s_options;
        let saved_w_options = global_w_options;
        let saved_environ = global_environ.take();
        let options = options_create(std::ptr::null_mut());
        let s_options = options_create(std::ptr::null_mut());
        let w_options = options_create(std::ptr::null_mut());
        global_options = options;
        global_s_options = s_options;
        global_w_options = w_options;
        let environ = environ_create();
        global_environ = Some(environ);

        let tree = format_create(None, None, 0, 0);
        for (name, value) in [(c"a", c"one"), (c"b", c"two")] {
            environ_set(
                global_environ.as_deref_mut().expect("test environment"),
                name.as_ptr(),
                0,
                |out| write_cstr(out, value.as_ptr()),
            );
        }
        for (expression, expected) in [
            (
                c"#{V/g:#{environ_name}=#{environ_value}:#{loop_index}:#{loop_last_flag};}",
                "a=one:0:0;b=two:1:1;",
            ),
            (
                c"#{V/g:#{environ_name}[#{V/g:#{environ_name}}]#{a};}",
                "a[ab]one;b[ab]one;",
            ),
        ] {
            let result = format_expand_cstring(tree, expression.as_ptr());
            assert_eq!(result.to_bytes(), expected.as_bytes(), "{expression:?}");
        }

        format_free(Box::from_raw(tree));
        global_options = saved_options;
        global_s_options = saved_s_options;
        global_w_options = saved_w_options;
        global_environ = saved_environ;
        options_free(options);
        options_free(s_options);
        options_free(w_options);
    }
}

//! Exercise lazy format-entry caching through the owned expansion path.
use hmux2::src::format::{
    format_add, format_add_owned_cb, format_create, format_expand_cstring, format_free,
    format_tree,
};
use hmux2::src::options::{options_create, options_free};
use hmux2::src::tmux::{global_options, global_s_options, global_w_options};
use std::sync::atomic::{AtomicUsize, Ordering};

static CALLBACK_CALLS: AtomicUsize = AtomicUsize::new(0);

fn callback(_ft: *mut format_tree) -> Option<std::ffi::CString> {
    CALLBACK_CALLS.fetch_add(1, Ordering::SeqCst);
    Some(std::ffi::CString::new(b"cached\xff".to_vec()).unwrap())
}

#[test]
fn expansion_caches_callback_then_replaces_the_same_entry() {
    unsafe {
        let saved_options = global_options;
        let saved_w_options = global_w_options;
        let saved_s_options = global_s_options;
        let options = options_create(core::ptr::null_mut());
        let w_options = options_create(core::ptr::null_mut());
        let s_options = options_create(core::ptr::null_mut());
        global_options = options;
        global_w_options = w_options;
        global_s_options = s_options;

        let ft = format_create(core::ptr::null_mut(), core::ptr::null_mut(), 0, 0);
        let key = b"zz_test_format_value\0".as_ptr().cast();
        let expression = b"#{zz_test_format_value}\0".as_ptr().cast();
        format_add_owned_cb(ft, std::ffi::CStr::from_ptr(key), callback);

        for _ in 0..2 {
            let expanded = format_expand_cstring(ft, expression);
            assert_eq!(expanded.as_bytes(), b"cached\xff");
        }
        assert_eq!(CALLBACK_CALLS.load(Ordering::SeqCst), 1);

        format_add(
            ft,
            key,
            b"%s\0".as_ptr() as *const core::ffi::c_char,
            b"replacement\0".as_ptr() as *const core::ffi::c_char,
        );
        let expanded = format_expand_cstring(ft, expression);
        assert_eq!(expanded.as_bytes(), b"replacement");
        assert_eq!(CALLBACK_CALLS.load(Ordering::SeqCst), 1);

        format_free(ft);
        global_options = saved_options;
        global_w_options = saved_w_options;
        global_s_options = saved_s_options;
        options_free(options);
        options_free(w_options);
        options_free(s_options);
    }
}

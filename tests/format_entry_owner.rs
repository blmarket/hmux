//! Exercise lazy format-entry caching through the owned expansion path.
use hmux::src::format::bytes::write_cstr;
use hmux::src::format::{
    format_add, format_add_owned_cb, format_create, format_expand_cstring, format_free, format_tree,
};
use hmux::src::options::{options_create, options_free};
use hmux::src::tmux::{global_options, global_s_options, global_w_options};
use std::sync::atomic::{AtomicUsize, Ordering};

static CALLBACK_CALLS: AtomicUsize = AtomicUsize::new(0);

fn callback(_ft: std::ptr::NonNull<format_tree>) -> Option<std::ffi::CString> {
    CALLBACK_CALLS.fetch_add(1, Ordering::SeqCst);
    Some(std::ffi::CString::new(b"cached\xff".to_vec()).unwrap())
}

#[test]
fn expansion_caches_callback_then_replaces_the_same_entry() {
    unsafe {
        let saved_options = global_options;
        let saved_w_options = global_w_options;
        let saved_s_options = global_s_options;
        let mut options_owner = options_create(None);
        let options = &raw mut *options_owner;
        let mut w_options_owner = options_create(None);
        let w_options = &raw mut *w_options_owner;
        let mut s_options_owner = options_create(None);
        let s_options = &raw mut *s_options_owner;
        global_options = options;
        global_w_options = w_options;
        global_s_options = s_options;

        let mut ft_owner = format_create(None, None, 0, 0);
        let ft = &raw mut *ft_owner;
        let key = c"zz_test_format_value";
        let expression = c"#{zz_test_format_value}".as_ptr();
        format_add_owned_cb(ft, key, callback);

        for _ in 0..2 {
            let expanded = format_expand_cstring(ft, expression);
            assert_eq!(expanded.as_bytes(), b"cached\xff");
        }
        assert_eq!(CALLBACK_CALLS.load(Ordering::SeqCst), 1);

        format_add(ft, key, |out| write_cstr(out, c"replacement".as_ptr()));
        let expanded = format_expand_cstring(ft, expression);
        assert_eq!(expanded.as_bytes(), b"replacement");
        assert_eq!(CALLBACK_CALLS.load(Ordering::SeqCst), 1);

        format_free(ft_owner);
        global_options = saved_options;
        global_w_options = saved_w_options;
        global_s_options = saved_s_options;
        options_free(options_owner);
        options_free(w_options_owner);
        options_free(s_options_owner);
    }
}

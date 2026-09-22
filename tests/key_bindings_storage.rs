use hmux2::src::{cmd::cmd_list_new, key_bindings::*};
use std::{ffi::CStr, ptr::null};

#[test]
fn named_tables_defaults_replacement_and_retained_table_lifetime() {
    unsafe {
        let z = key_bindings_get_table(c"z".as_ptr(), 1);
        let a = key_bindings_get_table(c"a".as_ptr(), 1);
        assert_eq!(key_bindings_get_table(c"z".as_ptr(), 0), z);
        assert_eq!(key_bindings_first_table(), a);
        assert_eq!(key_bindings_next_table(a), z);
        assert!(key_bindings_next_table(z).is_null());
        for key in [99, 1, 42] {
            key_bindings_add(c"a".as_ptr(), key, c"original".as_ptr(), 0, cmd_list_new());
        }
        let first = key_bindings_first(a);
        assert_eq!((*first).key, 1);
        assert_eq!((*key_bindings_next(a, first)).key, 42);
        key_bindings_add(
            c"a".as_ptr(),
            42,
            c"replacement".as_ptr(),
            1,
            cmd_list_new(),
        );
        let binding = key_bindings_get(a, 42);
        assert_eq!(CStr::from_ptr((*binding).note), c"replacement");
        assert_ne!((*binding).flags & KEY_BINDING_REPEAT, 0);
        // Active and default bindings use independent indexes with identical keys.
        let default = libc::calloc(1, std::mem::size_of::<key_binding>()) as *mut key_binding;
        (*default).key = 42;
        (*default).cmdlist = cmd_list_new();
        key_bindings_index_insert(&mut (*a).default_key_bindings, default);
        assert_eq!(key_bindings_get_default(a, 42), default);
        key_bindings_reset(c"a".as_ptr(), 42);
        assert_eq!((*binding).cmdlist, (*default).cmdlist);
        assert!((*binding).note.is_null());
        key_bindings_add(
            c"a".as_ptr(),
            42,
            c"annotation".as_ptr(),
            0,
            std::ptr::null_mut(),
        );
        assert_eq!(CStr::from_ptr((*binding).note), c"annotation");
        (*a).references += 1;
        key_bindings_remove_table(c"a".as_ptr());
        assert!(key_bindings_get_table(c"a".as_ptr(), 0).is_null());
        assert_eq!(key_bindings_get(a, 42), binding);
        key_bindings_unref_table(a);
        key_bindings_add(c"z".as_ptr(), 1, null(), 0, cmd_list_new());
        key_bindings_remove(c"z".as_ptr(), 1);
        assert!(key_bindings_first_table().is_null());
    }
}

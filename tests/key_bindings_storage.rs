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
        assert_eq!(
            CStr::from_ptr(
                ((*binding).note)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
            ),
            c"replacement"
        );
        assert_ne!((*binding).flags & KEY_BINDING_REPEAT, 0);
        // A note-only update may borrow the current note. Replacement must
        // copy it before releasing the previous owner.
        key_bindings_add(
            c"a".as_ptr(),
            42,
            ((*binding).note)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            0,
            std::ptr::null_mut(),
        );
        assert_eq!(
            CStr::from_ptr(
                ((*binding).note)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
            ),
            c"replacement"
        );
        key_bindings_add(
            c"a".as_ptr(),
            42,
            b"\xff\0".as_ptr().cast(),
            0,
            std::ptr::null_mut(),
        );
        assert_eq!(
            CStr::from_ptr(
                ((*binding).note)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
            )
            .to_bytes(),
            b"\xff"
        );
        // Active and default bindings use independent indexes with identical keys.
        let default = key_bindings_add_default(a, 42, cmd_list_new(), null(), 0);
        assert_eq!(key_bindings_get_default(a, 42), default);
        assert!((*default).tablename.is_null());
        let source = key_bindings_get(a, 99);
        let copied = key_bindings_add_default(
            a,
            99,
            cmd_list_new(),
            ((*source).note)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            0,
        );
        key_bindings_add(
            c"a".as_ptr(),
            99,
            c"changed".as_ptr(),
            0,
            std::ptr::null_mut(),
        );
        assert_eq!(
            CStr::from_ptr(
                ((*copied).note)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
            ),
            c"original"
        );
        key_bindings_reset(c"a".as_ptr(), 42);
        assert_eq!((*binding).cmdlist, (*default).cmdlist);
        assert!((*binding).note.is_none());
        key_bindings_add(
            c"a".as_ptr(),
            42,
            c"annotation".as_ptr(),
            0,
            std::ptr::null_mut(),
        );
        assert_eq!(
            CStr::from_ptr(
                ((*binding).note)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
            ),
            c"annotation"
        );
        (*a).references += 1;
        key_bindings_remove_table(c"a".as_ptr());
        assert!(key_bindings_get_table(c"a".as_ptr(), 0).is_null());
        assert_eq!(key_bindings_get(a, 42), binding);
        key_bindings_unref_table(a);
        key_bindings_add(c"z".as_ptr(), 1, null(), 0, cmd_list_new());
        key_bindings_remove(c"z".as_ptr(), 1);
        let raw_name = b"\xff\0".as_ptr().cast();
        let raw_table = key_bindings_get_table(raw_name, 1);
        assert_eq!(
            CStr::from_ptr(((*raw_table).name).as_ptr().cast_mut()).to_bytes(),
            b"\xff"
        );
        key_bindings_remove_table(raw_name);
        assert!(key_bindings_first_table().is_null());
    }
}

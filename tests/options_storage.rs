//! Behavior of the Rust-owned option-name index through the existing API.
use hmux2::src::format::bytes::write_cstr;
use hmux2::src::options::*;
use std::ffi::{CStr, CString};
use std::ptr::{null, null_mut};

#[test]
fn ordered_names_survive_updates_and_removal() {
    unsafe {
        let oo = options_create(null_mut());
        assert!(options_iter(&*oo).next().is_none());
        let mut names: Vec<Vec<u8>> = (0..128)
            .rev()
            .map(|i| format!("@key-{i:03}").into_bytes())
            .collect();
        names.extend([b"@\xff".to_vec(), b"@\x80".to_vec(), b"@".to_vec()]);
        let mut pointers = Vec::new();
        for name in &names {
            let name = CString::new(name.as_slice()).unwrap();
            let entry = options_set_string(oo, name.as_ptr(), 0, |out| out.write_all(b"value"));
            pointers.push(entry);
        }
        for (name, &entry) in names.iter().zip(&pointers) {
            let name = CString::new(name.as_slice()).unwrap();
            assert_eq!(hmux2::src::options::options_get_only_mut(&mut *(oo), std::ffi::CStr::from_ptr(name.as_ptr())).map_or(std::ptr::null_mut(), |entry| entry), entry);
            assert_eq!(options_owner(entry), oo);
            assert_eq!(
                options_set_string(oo, name.as_ptr(), 0, |out| { out.write_all(b"updated") }),
                entry
            );
        }
        names.sort();
        let keys: Vec<_> = options_iter(&*oo).map(|entry| entry.name.clone()).collect();
        for (name, key) in names.iter().zip(keys) {
            let entry = std::ptr::from_mut(options_get_only_mut(&mut *oo, &key).unwrap());
            assert!(!entry.is_null());
            assert_eq!(CStr::from_ptr(options_name(&*(entry)).as_ptr()).to_bytes(), name);
            assert_eq!(CStr::from_ptr((*entry).value.string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut())), c"updated");
            assert_eq!(options_remove_or_default(entry, null(), null_mut()), 0);
        }
        assert!(options_iter(&*oo).next().is_none());
        assert!(hmux2::src::options::options_get_only_mut(&mut *(oo), std::ffi::CStr::from_ptr(c"@key-000".as_ptr())).map_or(std::ptr::null_mut(), |entry| entry).is_null());
        // Repopulate so destruction also exercises nonempty storage.
        options_set_string(oo, c"@again".as_ptr(), 0, |out| out.write_all(b"value"));
        options_free(oo);
    }
}

#[test]
fn aliases_parent_fallback_and_shadowing() {
    unsafe {
        let parent = options_create(null_mut());
        let child = options_create(parent);
        let inherited = options_set_string(parent, c"@shared".as_ptr(), 0, |out| {
            out.write_all(b"parent")
        });
        assert!(hmux2::src::options::options_get_only_mut(&mut *(child), std::ffi::CStr::from_ptr(c"@shared".as_ptr())).map_or(std::ptr::null_mut(), |entry| entry).is_null());
        assert_eq!(options_get(child, c"@shared".as_ptr()), inherited);
        let local =
            options_set_string(child, c"@shared".as_ptr(), 0, |out| out.write_all(b"child"));
        assert_ne!(local, inherited);
        assert_eq!(options_get(child, c"@shared".as_ptr()), local);
        options_remove_or_default(local, null(), null_mut());
        assert_eq!(options_get(child, c"@shared".as_ptr()), inherited);

        // Use an actual option-table entry so alias lookup and value cleanup
        // follow the same path as built-in options.
        let table = &raw const hmux2::src::options_table::options_table;
        let definition = (*table)
            .iter()
            .find(|oe| oe.name == Some(c"display-panes-colour"))
            .unwrap();
        let canonical = options_default(parent, definition);
        assert_eq!(
            hmux2::src::options::options_get_only_mut(&mut *(parent), std::ffi::CStr::from_ptr(c"display-panes-color".as_ptr())).map_or(std::ptr::null_mut(), |entry| entry),
            canonical
        );
        assert_eq!(
            options_get(child, c"display-panes-color".as_ptr()),
            canonical
        );
        options_set_string(child, c"display-panes-color".as_ptr(), 0, |out| {
            out.write_all(b"red")
        });
        assert_eq!(
            CStr::from_ptr(options_get_string(child, c"display-panes-colour".as_ptr())),
            c"red"
        );
        options_free(child);
        assert_eq!(hmux2::src::options::options_get_only_mut(&mut *(parent), std::ffi::CStr::from_ptr(c"@shared".as_ptr())).map_or(std::ptr::null_mut(), |entry| entry), inherited);
        options_free(parent);
    }
}

#[test]
fn scalar_string_replacement_append_and_default_keep_stable_entry() {
    unsafe {
        let oo = options_create(null_mut());
        let entry = options_set_string(oo, c"@bytes".as_ptr(), 0, |out| {
            write_cstr(out, c"\xff".as_ptr())
        });
        let name = options_name(&*(entry)).as_ptr();
        let previous = (*entry).value.string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut());
        assert_eq!(
            options_set_string(oo, name, 1, |out| { write_cstr(out, previous) }),
            entry
        );
        assert_eq!(
            CStr::from_ptr((*entry).value.string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut())).to_bytes(),
            b"\xff\xff"
        );
        let previous = (*entry).value.string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut());
        assert_eq!(
            options_set_string(oo, name, 0, |out| { write_cstr(out, previous) }),
            entry
        );
        assert_eq!(
            CStr::from_ptr((*entry).value.string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut())).to_bytes(),
            b"\xff\xff"
        );
        options_set_string(oo, name, 0, |out| out.write_all(b""));
        assert_eq!(CStr::from_ptr((*entry).value.string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut())), c"");

        let table = &raw const hmux2::src::options_table::options_table;
        let definition = (*table)
            .iter()
            .find(|oe| oe.name == Some(c"status-left"))
            .unwrap();
        let default = options_default(oo, definition);
        assert_eq!(
            CStr::from_ptr((*default).value.string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut())),
            definition.default_str.unwrap()
        );
        assert_eq!(hmux2::src::options::options_get_only_mut(&mut *(oo), std::ffi::CStr::from_ptr(definition.name_ptr())).map_or(std::ptr::null_mut(), |entry| entry), default);

        let empty_definition = (*table)
            .iter()
            .find(|oe| oe.name == Some(c"status-right"))
            .unwrap();
        let empty = options_empty(oo, empty_definition);
        assert!((*empty).value.string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut()).is_null());
        options_set_string(oo, empty_definition.name_ptr(), 1, |out| {
            write_cstr(out, c"tail".as_ptr())
        });
        assert_eq!(CStr::from_ptr((*empty).value.string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut())), c"(null)tail");
        options_free(oo);
    }
}

#[test]
fn array_keys_order_normalize_and_keep_stable_items() {
    unsafe {
        let oo = options_create(null_mut());
        let table = &raw const hmux2::src::options_table::options_table;
        let definition = (*table)
            .iter()
            .find(|oe| oe.name == Some(c"update-environment"))
            .unwrap();
        let array = options_empty(oo, definition);
        assert!(options_array_iter_mut(&mut *(array)).next().map_or(std::ptr::null_mut(), |item| item).is_null());
        let keys: &[&[u8]] = &[
            b"\xff",
            b"10",
            b"alpha",
            b"002",
            b"0",
            b"4294967295",
            b"\x80",
            b"-1",
        ];
        for key in keys {
            let key = CString::new(*key).unwrap();
            assert_eq!(
                options_array_set(array, key.as_ptr(), c"value".as_ptr(), 0, null_mut()),
                0
            );
        }
        let first = options_array_iter_mut(&mut *(array)).next().map_or(std::ptr::null_mut(), |item| item);
        let two = options_array_iter_mut(&mut *array).nth(1).map_or(std::ptr::null_mut(), |item| item);
        assert_eq!(CStr::from_ptr(options_array_item_key(&*(two)).as_ptr()), c"2");
        assert_eq!(
            options_array_set(array, c"0002".as_ptr(), c"updated".as_ptr(), 0, null_mut()),
            0
        );
        assert_eq!(options_array_iter_mut(&mut *array).nth(1).map_or(std::ptr::null_mut(), |item| item), two);
        assert_eq!(
            hmux2::src::options::options_array_get_mut(&mut *(array), std::ffi::CStr::from_ptr(c"02".as_ptr())).map_or(std::ptr::null_mut(), |value| value),
            (hmux2::src::options::options_array_item_value_mut(&mut *(two)) as *mut hmux2::src::shared::options::options_value)
        );
        assert_eq!(
            hmux2::src::options::options_array_get_index_mut(&mut *(array), 2).map_or(std::ptr::null_mut(), |value| value),
            (hmux2::src::options::options_array_item_value_mut(&mut *(two)) as *mut hmux2::src::shared::options::options_value)
        );
        for (index, key) in [(0, c"0"), (u32::MAX, c"4294967295")] {
            let value = hmux2::src::options::options_array_get_index_mut(&mut *(array), index).map_or(std::ptr::null_mut(), |value| value);
            assert!(!value.is_null());
            assert_eq!(value, hmux2::src::options::options_array_get_mut(&mut *(array), std::ffi::CStr::from_ptr(key.as_ptr())).map_or(std::ptr::null_mut(), |value| value));
        }
        assert!(hmux2::src::options::options_array_get_index_mut(&mut *(array), 3).map_or(std::ptr::null_mut(), |value| value).is_null());
        let non_utf8 = c"\xff";
        let non_utf8_value = hmux2::src::options::options_array_get_mut(&mut *(array), std::ffi::CStr::from_ptr(non_utf8.as_ptr())).map_or(std::ptr::null_mut(), |value| value);
        assert!(!non_utf8_value.is_null());
        assert!(hmux2::src::options::options_array_get_mut(&mut *(array), std::ffi::CStr::from_ptr(c"".as_ptr())).map_or(std::ptr::null_mut(), |value| value).is_null());
        assert_eq!(
            CStr::from_ptr((*(hmux2::src::options::options_array_item_value_mut(&mut *(two)) as *mut hmux2::src::shared::options::options_value)).string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut())),
            c"updated"
        );
        for (key, expected) in [
            (c"0002".as_ptr(), b"updated".as_slice()),
            (non_utf8.as_ptr(), b"value".as_slice()),
            (c"4294967296".as_ptr(), b"".as_slice()),
        ] {
            let output = options_to_cstring(array, key, 0);
            assert_eq!(output.as_bytes(), expected);
        }
        for invalid in [c"", c"4294967296"] {
            assert_eq!(
                options_array_set(array, invalid.as_ptr(), c"bad".as_ptr(), 0, null_mut()),
                -1
            );
            assert!(hmux2::src::options::options_array_get_mut(&mut *(array), std::ffi::CStr::from_ptr(invalid.as_ptr())).map_or(std::ptr::null_mut(), |value| value).is_null());
        }
        let expected: &[&[u8]] = &[
            b"0",
            b"2",
            b"10",
            b"4294967295",
            b"-1",
            b"alpha",
            b"\x80",
            b"\xff",
        ];
        let keys: Vec<_> = options_array_iter(&*array).map(|item| item.key.clone()).collect();
        for (key, saved_key) in expected.iter().zip(keys) {
            let item = options_array_iter_mut(&mut *array).find(|item| item.key == saved_key).map_or(std::ptr::null_mut(), |item| item);
            assert!(!item.is_null());
            assert_eq!(
                CStr::from_ptr(options_array_item_key(&*(item)).as_ptr()).to_bytes(),
                *key
            );
            let key = CString::new(*key).unwrap();
            assert_eq!(
                options_array_set(array, key.as_ptr(), null(), 0, null_mut()),
                0
            );
        }
        assert!(options_array_iter_mut(&mut *(array)).next().map_or(std::ptr::null_mut(), |item| item).is_null());
        // Clearing keeps the storage reusable; destroying a populated option
        // releases both the Rust index and C-allocated values.
        for _ in 0..3 {
            for index in (0..128).rev() {
                let key = CString::new(index.to_string()).unwrap();
                assert_eq!(
                    options_array_set(array, key.as_ptr(), c"again".as_ptr(), 0, null_mut()),
                    0
                );
            }
            options_array_clear(array);
            assert!(options_array_iter_mut(&mut *(array)).next().map_or(std::ptr::null_mut(), |item| item).is_null());
        }
        assert_eq!(
            options_array_set(array, c"7".as_ptr(), c"last".as_ptr(), 0, null_mut()),
            0
        );
        // Replacing the option also destroys populated array storage.
        let replacement = options_default(oo, definition);
        assert!(!options_array_iter_mut(&mut *(replacement)).next().map_or(std::ptr::null_mut(), |item| item).is_null());
        options_free(oo);
    }
}

#[test]
fn array_assign_copies_split_tokens_and_keeps_partial_result_on_error() {
    unsafe {
        let oo = options_create(null_mut());
        let table = &raw const hmux2::src::options_table::options_table;
        let definition = (*table)
            .iter()
            .find(|oe| oe.name == Some(c"update-environment"))
            .unwrap();
        let strings = options_empty(oo, definition);
        let mut input = b"  ONE,\xff TWO,,\0ignored".to_vec();
        assert_eq!(
            options_array_assign(strings, input.as_ptr().cast(), null_mut()),
            0
        );
        input.fill(b'x');
        for (index, expected) in [b"ONE".as_slice(), b"\xff", b"TWO"].iter().enumerate() {
            let key = CString::new(index.to_string()).unwrap();
            let value = hmux2::src::options::options_array_get_mut(&mut *(strings), std::ffi::CStr::from_ptr(key.as_ptr())).map_or(std::ptr::null_mut(), |value| value);
            assert!(!value.is_null());
            assert_eq!(CStr::from_ptr((*value).string_ptr().map_or(std::ptr::null_mut(), |value| value.as_ptr().cast_mut())).to_bytes(), *expected);
        }
        assert!(hmux2::src::options::options_array_get_mut(&mut *(strings), std::ffi::CStr::from_ptr(c"3".as_ptr())).map_or(std::ptr::null_mut(), |value| value).is_null());

        let colour_definition = (*table)
            .iter()
            .find(|oe| oe.name == Some(c"pane-colours"))
            .unwrap();
        let colours = options_empty(oo, colour_definition);
        let mut cause: Option<CString> = None;
        assert_eq!(
            options_array_assign(colours, c"red,,invalid-colour".as_ptr(), &mut cause),
            -1
        );
        assert!(!hmux2::src::options::options_array_get_mut(&mut *(colours), std::ffi::CStr::from_ptr(c"0".as_ptr())).map_or(std::ptr::null_mut(), |value| value).is_null());
        assert!(hmux2::src::options::options_array_get_mut(&mut *(colours), std::ffi::CStr::from_ptr(c"1".as_ptr())).map_or(std::ptr::null_mut(), |value| value).is_null());
        assert_eq!(
            CStr::from_ptr(cause.as_ref().unwrap().as_ptr()),
            c"bad colour: invalid-colour"
        );
        options_free(oo);
    }
}

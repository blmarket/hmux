//! Behavior of the Rust-owned option-name index through the existing API.
use hmux2::src::format::bytes::write_cstr;
use hmux2::src::options::*;
use std::ffi::{CStr, CString};
use std::ptr::{null, null_mut};

#[test]
fn ordered_names_survive_updates_and_removal() {
    unsafe {
        let oo = options_create(null_mut());
        assert!(options_first(oo).is_null());
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
            assert_eq!(options_get_only(oo, name.as_ptr()), entry);
            assert_eq!(options_owner(entry), oo);
            assert_eq!(
                options_set_string(oo, name.as_ptr(), 0, |out| { out.write_all(b"updated") }),
                entry
            );
        }
        names.sort();
        let mut entry = options_first(oo);
        for name in &names {
            assert!(!entry.is_null());
            assert_eq!(CStr::from_ptr(options_name(entry)).to_bytes(), name);
            assert_eq!(CStr::from_ptr((*entry).value.string_ptr()), c"updated");
            let next = options_next(entry);
            assert_eq!(options_remove_or_default(entry, null(), null_mut()), 0);
            entry = next;
        }
        assert!(entry.is_null());
        assert!(options_first(oo).is_null());
        assert!(options_get_only(oo, c"@key-000".as_ptr()).is_null());
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
        assert!(options_get_only(child, c"@shared".as_ptr()).is_null());
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
            .find(|oe| !oe.name.is_null() && CStr::from_ptr(oe.name) == c"display-panes-colour")
            .unwrap();
        let canonical = options_default(parent, definition);
        assert_eq!(
            options_get_only(parent, c"display-panes-color".as_ptr()),
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
        assert_eq!(options_get_only(parent, c"@shared".as_ptr()), inherited);
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
        let name = options_name(entry);
        let previous = (*entry).value.string_ptr();
        assert_eq!(
            options_set_string(oo, name, 1, |out| { write_cstr(out, previous) }),
            entry
        );
        assert_eq!(
            CStr::from_ptr((*entry).value.string_ptr()).to_bytes(),
            b"\xff\xff"
        );
        let previous = (*entry).value.string_ptr();
        assert_eq!(
            options_set_string(oo, name, 0, |out| { write_cstr(out, previous) }),
            entry
        );
        assert_eq!(
            CStr::from_ptr((*entry).value.string_ptr()).to_bytes(),
            b"\xff\xff"
        );
        options_set_string(oo, name, 0, |out| out.write_all(b""));
        assert_eq!(CStr::from_ptr((*entry).value.string_ptr()), c"");

        let table = &raw const hmux2::src::options_table::options_table;
        let definition = (*table)
            .iter()
            .find(|oe| !oe.name.is_null() && CStr::from_ptr(oe.name) == c"status-left")
            .unwrap();
        let default = options_default(oo, definition);
        assert_eq!(
            CStr::from_ptr((*default).value.string_ptr()),
            CStr::from_ptr(definition.default_str)
        );
        assert_eq!(options_get_only(oo, definition.name), default);

        let empty_definition = (*table)
            .iter()
            .find(|oe| !oe.name.is_null() && CStr::from_ptr(oe.name) == c"status-right")
            .unwrap();
        let empty = options_empty(oo, empty_definition);
        assert!((*empty).value.string_ptr().is_null());
        options_set_string(oo, empty_definition.name, 1, |out| {
            write_cstr(out, c"tail".as_ptr())
        });
        assert_eq!(CStr::from_ptr((*empty).value.string_ptr()), c"(null)tail");
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
            .find(|oe| !oe.name.is_null() && CStr::from_ptr(oe.name) == c"update-environment")
            .unwrap();
        let array = options_empty(oo, definition);
        assert!(options_array_first(array).is_null());
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
        let first = options_array_first(array);
        let two = options_array_next(first);
        assert_eq!(CStr::from_ptr(options_array_item_key(two)), c"2");
        assert_eq!(
            options_array_set(array, c"0002".as_ptr(), c"updated".as_ptr(), 0, null_mut()),
            0
        );
        assert_eq!(options_array_next(first), two);
        assert_eq!(
            options_array_get(array, c"02".as_ptr()),
            options_array_item_value(two)
        );
        assert_eq!(
            options_array_get_index(array, 2),
            options_array_item_value(two)
        );
        for (index, key) in [(0, c"0"), (u32::MAX, c"4294967295")] {
            let value = options_array_get_index(array, index);
            assert!(!value.is_null());
            assert_eq!(value, options_array_get(array, key.as_ptr()));
        }
        assert!(options_array_get_index(array, 3).is_null());
        let non_utf8 = c"\xff";
        let non_utf8_value = options_array_get(array, non_utf8.as_ptr());
        assert!(!non_utf8_value.is_null());
        assert!(options_array_get(array, c"".as_ptr()).is_null());
        assert_eq!(
            CStr::from_ptr((*options_array_item_value(two)).string_ptr()),
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
            assert!(options_array_get(array, invalid.as_ptr()).is_null());
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
        let mut item = first;
        for key in expected {
            assert!(!item.is_null());
            assert_eq!(
                CStr::from_ptr(options_array_item_key(item)).to_bytes(),
                *key
            );
            let next = options_array_next(item);
            let key = CString::new(*key).unwrap();
            assert_eq!(
                options_array_set(array, key.as_ptr(), null(), 0, null_mut()),
                0
            );
            item = next;
        }
        assert!(item.is_null());
        assert!(options_array_first(array).is_null());
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
            assert!(options_array_first(array).is_null());
        }
        assert_eq!(
            options_array_set(array, c"7".as_ptr(), c"last".as_ptr(), 0, null_mut()),
            0
        );
        // Replacing the option also destroys populated array storage.
        let replacement = options_default(oo, definition);
        assert!(!options_array_first(replacement).is_null());
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
            .find(|oe| !oe.name.is_null() && CStr::from_ptr(oe.name) == c"update-environment")
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
            let value = options_array_get(strings, key.as_ptr());
            assert!(!value.is_null());
            assert_eq!(CStr::from_ptr((*value).string_ptr()).to_bytes(), *expected);
        }
        assert!(options_array_get(strings, c"3".as_ptr()).is_null());

        let colour_definition = (*table)
            .iter()
            .find(|oe| !oe.name.is_null() && CStr::from_ptr(oe.name) == c"pane-colours")
            .unwrap();
        let colours = options_empty(oo, colour_definition);
        let mut cause: Option<CString> = None;
        assert_eq!(
            options_array_assign(colours, c"red,,invalid-colour".as_ptr(), &mut cause),
            -1
        );
        assert!(!options_array_get(colours, c"0".as_ptr()).is_null());
        assert!(options_array_get(colours, c"1".as_ptr()).is_null());
        assert_eq!(
            CStr::from_ptr(cause.as_ref().unwrap().as_ptr()),
            c"bad colour: invalid-colour"
        );
        options_free(oo);
    }
}

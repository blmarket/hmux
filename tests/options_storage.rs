//! Behavior of the Rust-owned option-name index through the existing API.
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
            let entry = options_set_string(oo, name.as_ptr(), 0, c"value".as_ptr());
            pointers.push(entry);
        }
        for (name, &entry) in names.iter().zip(&pointers) {
            let name = CString::new(name.as_slice()).unwrap();
            assert_eq!(options_get_only(oo, name.as_ptr()), entry);
            assert_eq!(options_owner(entry), oo);
            assert_eq!(
                options_set_string(oo, name.as_ptr(), 0, c"updated".as_ptr()),
                entry
            );
        }
        names.sort();
        let mut entry = options_first(oo);
        for name in &names {
            assert!(!entry.is_null());
            assert_eq!(CStr::from_ptr(options_name(entry)).to_bytes(), name);
            assert_eq!(CStr::from_ptr((*entry).value.string), c"updated");
            let next = options_next(entry);
            assert_eq!(options_remove_or_default(entry, null(), null_mut()), 0);
            entry = next;
        }
        assert!(entry.is_null());
        assert!(options_first(oo).is_null());
        assert!(options_get_only(oo, c"@key-000".as_ptr()).is_null());
        // Repopulate so destruction also exercises nonempty storage.
        options_set_string(oo, c"@again".as_ptr(), 0, c"value".as_ptr());
        options_free(oo);
    }
}

#[test]
fn aliases_parent_fallback_and_shadowing() {
    unsafe {
        let parent = options_create(null_mut());
        let child = options_create(parent);
        let inherited = options_set_string(parent, c"@shared".as_ptr(), 0, c"parent".as_ptr());
        assert!(options_get_only(child, c"@shared".as_ptr()).is_null());
        assert_eq!(options_get(child, c"@shared".as_ptr()), inherited);
        let local = options_set_string(child, c"@shared".as_ptr(), 0, c"child".as_ptr());
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
        options_set_string(child, c"display-panes-color".as_ptr(), 0, c"red".as_ptr());
        assert_eq!(
            CStr::from_ptr(options_get_string(child, c"display-panes-colour".as_ptr())),
            c"red"
        );
        options_free(child);
        assert_eq!(options_get_only(parent, c"@shared".as_ptr()), inherited);
        options_free(parent);
    }
}

use crate::src::log::log_cstr;
// Private tree-storage implementation.  It owns the format-entry tree and
// format-tree CRUD operations.
// Shared C-layout types, allocator/FFI helpers, and callback-table symbols
// remain supplied by the parent facade.
use super::*;
use std::ffi::{CStr, CString};

fn format_entry_new(key: &CStr) -> *mut format_entry {
    let key = key.to_owned();
    let entry = format_entry {
        owned_cb: None,
        key: key.clone(),
        value: Default::default(),
        time: 0,
    };
    let owner = Box::new(format_entry {
        key: key,
        value: None,
        owned_cb: None,
        ..entry
    });
    Box::into_raw(owner) as *mut format_entry
}

fn format_entry_set_value(fe: &mut format_entry, value: Option<CString>) {
    fe.value = value;
}

/// Evaluate without retaining an owner borrow: callbacks may add or replace
/// entries in this tree. As with the C API, callbacks must not destroy the tree.
pub(super) unsafe fn format_entry_ensure_value(ft: *mut format_tree, fe: *mut format_entry) {
    if !(*fe).value.is_none() {
        return;
    }
    let key = (*fe).key.clone();
    let Some(mut callback) = (*fe).owned_cb.take() else {
        return;
    };
    let value =
        callback(std::ptr::NonNull::new(ft).expect("format tree is non-null")).unwrap_or_default();
    let current = format_entry_tree_find_key(&mut (*ft).tree, key.as_c_str());
    if current == fe
        && !current.is_null()
        && (*current).value.is_none()
        && (*current).owned_cb.is_none()
    {
        format_entry_set_value(&mut *current, Some(value));
        if (*current).owned_cb.is_none() {
            (*current).owned_cb = Some(callback);
        }
    }
}

fn format_entry_tree_key(elm: &format_entry) -> Vec<u8> {
    elm.key.as_bytes().to_vec()
}

unsafe fn format_entry_tree_find_key(
    head: *mut format_entry_tree,
    key: &CStr,
) -> *mut format_entry {
    if head.is_null() {
        return ::core::ptr::null_mut();
    }
    (*head)
        .entries
        .get(key.to_bytes())
        .copied()
        .unwrap_or(::core::ptr::null_mut())
}

pub(super) unsafe fn format_entry_tree_find(
    head: *mut format_entry_tree,
    elm: *mut format_entry,
) -> *mut format_entry {
    if head.is_null() || elm.is_null() {
        return ::core::ptr::null_mut::<format_entry>();
    }
    (*head)
        .entries
        .get(&format_entry_tree_key(&*elm))
        .copied()
        .unwrap_or(::core::ptr::null_mut::<format_entry>())
}

unsafe fn format_entry_tree_insert(
    head: *mut format_entry_tree,
    elm: *mut format_entry,
) -> *mut format_entry {
    if head.is_null() || elm.is_null() {
        return ::core::ptr::null_mut::<format_entry>();
    }
    match (*head).entries.entry(format_entry_tree_key(&*elm)) {
        std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
            ::core::ptr::null_mut::<format_entry>()
        }
    }
}

unsafe fn format_entry_tree_remove(
    head: *mut format_entry_tree,
    elm: *mut format_entry,
) -> *mut format_entry {
    if head.is_null() || elm.is_null() {
        return ::core::ptr::null_mut::<format_entry>();
    }
    let key = format_entry_tree_key(&*elm);
    if (*head).entries.get(&key).copied() != Some(elm) {
        return ::core::ptr::null_mut::<format_entry>();
    }
    (*head)
        .entries
        .remove(&key)
        .unwrap_or(::core::ptr::null_mut::<format_entry>())
}

unsafe fn format_entry_tree_minmax(head: *mut format_entry_tree) -> *mut format_entry {
    if head.is_null() {
        return ::core::ptr::null_mut::<format_entry>();
    }
    let item = (*head).entries.values().next();
    item.copied()
        .unwrap_or(::core::ptr::null_mut::<format_entry>())
}

unsafe fn format_entry_tree_next(
    head: *mut format_entry_tree,
    elm: *mut format_entry,
) -> *mut format_entry {
    if head.is_null() || elm.is_null() {
        return ::core::ptr::null_mut::<format_entry>();
    }
    (*head)
        .entries
        .range((
            std::ops::Bound::Excluded(format_entry_tree_key(&*elm)),
            std::ops::Bound::Unbounded,
        ))
        .next()
        .map(|(_, item)| *item)
        .unwrap_or(::core::ptr::null_mut::<format_entry>())
}
pub unsafe fn format_merge(mut ft: *mut format_tree, mut from: *mut format_tree) {
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    fe = format_entry_tree_minmax(&raw mut (*from).tree);
    while !fe.is_null() {
        if !(*fe).value.is_none() {
            format_add(
                ft,
                ((*fe).key).as_ptr().cast_mut(),
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                ((*fe).value)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            );
        }
        fe = format_entry_tree_next(&raw mut (*from).tree, fe);
    }
}
pub unsafe fn format_get_pane(mut ft: *mut format_tree) -> *mut window_pane {
    return (*ft).wp;
}
pub(super) unsafe fn format_create_add_item(mut ft: *mut format_tree, mut item: *mut cmdq_item) {
    let mut event: *mut key_event = cmdq_get_event(item);
    let mut m: *mut mouse_event = &raw mut (*event).m;
    cmdq_merge_formats(item, ft);
    memcpy(
        &raw mut (*ft).m as *mut ::core::ffi::c_void,
        m as *const ::core::ffi::c_void,
        ::core::mem::size_of::<mouse_event>() as size_t,
    );
}
pub unsafe fn format_create(
    mut c: *mut client,
    mut item: *mut cmdq_item,
    mut tag: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> *mut format_tree {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    ft = Box::into_raw(Box::new(format_tree::default()));
    if !c.is_null() {
        (*ft).client = c;
        crate::src::shared::rc::retain((*ft).client);
    }
    (*ft).item = item;
    (*ft).tag = tag as u_int;
    (*ft).flags = flags;
    if !item.is_null() {
        format_create_add_item(ft, item);
    }
    return ft;
}
pub unsafe fn format_free(mut ft: *mut format_tree) {
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut fe1: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    fe = format_entry_tree_minmax(&raw mut (*ft).tree);
    while !fe.is_null() && {
        fe1 = format_entry_tree_next(&raw mut (*ft).tree, fe);
        1 as ::core::ffi::c_int != 0
    } {
        format_entry_tree_remove(&raw mut (*ft).tree, fe);
        drop(Box::from_raw(fe as *mut format_entry));
        fe = fe1;
    }
    if !(*ft).client.is_null() {
        server_client_unref((*ft).client);
    }
    drop(Box::from_raw(ft));
}
pub unsafe fn format_log_debug(mut ft: *mut format_tree, mut prefix: *const ::core::ffi::c_char) {
    if log_get_level() == 0 as ::core::ffi::c_int {
        return;
    }
    format_each(ft, |key, value| {
        log_debug(format_args!(
            "{}: {}={}",
            log_cstr((prefix) as *const _),
            log_cstr((key.as_ptr()) as *const _),
            log_cstr((value.as_ptr()) as *const _)
        ));
    });
}
pub unsafe fn format_each(ft: *mut format_tree, mut cb: impl FnMut(&CStr, &CStr)) {
    let mut fe: *mut format_entry;
    let mut s: [::core::ffi::c_char; 64] = [0; 64];
    for entry in &FORMAT_TABLE {
        let Some(value) = entry.get(ft) else { continue };
        let value = match value {
            FormatValue::String(value) => value,
            FormatValue::Time(value) => {
                CString::new(value.to_string()).expect("timestamp contains no NUL")
            }
        };
        cb(entry.key, value.as_c_str());
    }
    let mut values = Vec::new();
    fe = format_entry_tree_minmax(&raw mut (*ft).tree);
    let mut keys = Vec::new();
    while !fe.is_null() {
        keys.push((*fe).key.clone());
        fe = format_entry_tree_next(&raw mut (*ft).tree, fe);
    }
    for key in keys {
        fe = format_entry_tree_find_key(&raw mut (*ft).tree, key.as_c_str());
        if fe.is_null() {
            continue;
        }
        if (*fe).time != 0 as time_t {
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                (*fe).time as ::core::ffi::c_longlong,
            );
            values.push((key, CStr::from_ptr((&raw const s).cast()).to_owned()));
        } else {
            format_entry_ensure_value(ft, fe);
            fe = format_entry_tree_find_key(&raw mut (*ft).tree, key.as_c_str());
            if !fe.is_null() {
                if let Some(value) = (*fe).value.as_ref() {
                    values.push(((*fe).key.clone(), value.clone()));
                }
            }
        }
    }
    for (key, value) in values {
        cb(key.as_c_str(), value.as_c_str());
    }
}
pub unsafe extern "C" fn format_add(
    mut ft: *mut format_tree,
    mut key: *const ::core::ffi::c_char,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let value = xvasprintf_cstring(fmt, args.clone());
    format_add_value(ft, CStr::from_ptr(key), value);
}

/// Insert literal text, copying the value into the format tree.
pub unsafe fn format_add_cstr(ft: *mut format_tree, key: &CStr, value: &CStr) {
    format_add_value(ft, key, value.to_owned());
}

unsafe fn format_add_value(ft: *mut format_tree, key: &CStr, value: CString) {
    let mut fe = format_entry_new(key);
    let fe_now = format_entry_tree_insert(&raw mut (*ft).tree, fe);
    if !fe_now.is_null() {
        drop(Box::from_raw(fe));
        fe = fe_now;
    }
    (*fe).owned_cb = None;
    (*fe).time = 0;
    format_entry_set_value(&mut *fe, Some(value));
}
pub unsafe fn format_add_tv(
    mut ft: *mut format_tree,
    mut key: *const ::core::ffi::c_char,
    mut tv: *mut timeval,
) {
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut fe_now: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    fe = format_entry_new(CStr::from_ptr(key));
    fe_now = format_entry_tree_insert(&raw mut (*ft).tree, fe);
    if !fe_now.is_null() {
        drop(Box::from_raw(fe as *mut format_entry));
        fe = fe_now;
    }
    (*(fe as *mut format_entry)).owned_cb = None;
    (*fe).time = (*tv).tv_sec as time_t;
    format_entry_set_value(&mut *fe, None);
}
/// Registers a lazy Rust callback that returns an owned cache value.
pub unsafe fn format_add_owned_cb(
    ft: *mut format_tree,
    key: &CStr,
    cb: impl FnMut(std::ptr::NonNull<format_tree>) -> Option<CString> + 'static,
) {
    let new = format_entry_new(key);
    let existing = format_entry_tree_insert(&raw mut (*ft).tree, new);
    let fe = if existing.is_null() {
        new
    } else {
        drop(Box::from_raw(new as *mut format_entry));
        existing
    };
    (*(fe as *mut format_entry)).owned_cb = Some(Box::new(cb));
    (*fe).time = 0;
    format_entry_set_value(&mut *fe, None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};

    fn cached_test_value(_ft: std::ptr::NonNull<format_tree>) -> Option<CString> {
        Some(CString::new(b"cached\xff".to_vec()).unwrap())
    }

    #[test]
    fn owned_callbacks_cache_absence_and_allow_reentrant_replacement() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        fn replace_during_callback(ft: std::ptr::NonNull<format_tree>) -> Option<CString> {
            CALLS.fetch_add(1, Ordering::Relaxed);
            unsafe { format_add(ft.as_ptr(), c"owned".as_ptr(), c"temporary".as_ptr()) };
            Some(CString::new(b"owned\xff".to_vec()).unwrap())
        }
        fn absent(_ft: std::ptr::NonNull<format_tree>) -> Option<CString> {
            CALLS.fetch_add(1, Ordering::Relaxed);
            None
        }
        unsafe {
            CALLS.store(0, Ordering::Relaxed);
            let ft = format_create(std::ptr::null_mut(), std::ptr::null_mut(), 0, 0);
            let mut probe: format_entry = format_entry::empty();
            probe.key = ::std::ffi::CStr::from_ptr(c"owned".as_ptr().cast_mut()).to_owned();
            format_add_owned_cb(ft, c"owned", replace_during_callback);
            let entry = format_entry_tree_find(&raw mut (*ft).tree, &raw mut probe);
            assert!((*entry).value.is_none());
            assert_eq!(CALLS.load(Ordering::Relaxed), 0);
            format_entry_ensure_value(ft, entry);
            format_entry_ensure_value(ft, entry);
            assert_eq!(CALLS.load(Ordering::Relaxed), 1);
            assert_eq!(
                CStr::from_ptr(
                    ((*entry).value)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                b"temporary"
            );
            assert!((*entry).owned_cb.is_none());

            format_add_owned_cb(ft, c"owned", absent);
            format_entry_ensure_value(ft, entry);
            format_entry_ensure_value(ft, entry);
            assert_eq!(CALLS.load(Ordering::Relaxed), 2);
            assert_eq!(
                CStr::from_ptr(
                    ((*entry).value)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                ),
                c""
            );

            format_add_owned_cb(ft, c"owned", replace_during_callback);
            format_add(ft, c"owned".as_ptr(), c"literal".as_ptr());
            format_entry_ensure_value(ft, entry);
            assert_eq!(
                CStr::from_ptr(
                    ((*entry).value)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                ),
                c"literal"
            );
            assert!((*(entry as *mut format_entry)).owned_cb.is_none());

            format_add_owned_cb(ft, c"owned", replace_during_callback);
            let mut tv = timeval {
                tv_sec: 123,
                tv_usec: 0,
            };
            format_add_tv(ft, c"owned".as_ptr(), &raw mut tv);
            assert_eq!((*entry).time, 123);
            assert!((*(entry as *mut format_entry)).owned_cb.is_none());

            format_add_owned_cb(ft, c"owned", replace_during_callback);
            format_add_owned_cb(ft, c"owned", cached_test_value);
            format_entry_ensure_value(ft, entry);
            assert_eq!(
                CStr::from_ptr(
                    ((*entry).value)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                b"cached\xff"
            );
            assert_eq!(CALLS.load(Ordering::Relaxed), 2);
            format_free(ft);
        }
    }

    #[test]
    fn format_entry_owns_replacements_and_cached_callback_values() {
        unsafe {
            let ft = format_create(::core::ptr::null_mut(), ::core::ptr::null_mut(), 0, 0);
            let key = b"custom\xff\0".as_ptr() as *const ::core::ffi::c_char;
            let format = b"%s\0".as_ptr() as *const ::core::ffi::c_char;
            let first = b"first\xff\0".as_ptr() as *const ::core::ffi::c_char;
            format_add(ft, key, format, first);
            let mut probe = format_entry {
                owned_cb: None,
                key: ::std::ffi::CStr::from_ptr(key as *mut _).to_owned(),
                value: Default::default(),
                time: 0,
            };
            let entry = format_entry_tree_find(&raw mut (*ft).tree, &raw mut probe);
            assert!(!entry.is_null());
            assert_eq!(
                CStr::from_ptr(
                    ((*entry).value)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                b"first\xff"
            );

            // The old value remains alive while vasprintf reads its argument.
            format_add(
                ft,
                key,
                b"%s-next\0".as_ptr() as *const _,
                ((*entry).value)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            );
            assert_eq!(
                format_entry_tree_find(&raw mut (*ft).tree, &raw mut probe),
                entry
            );
            assert_eq!(
                CStr::from_ptr(
                    ((*entry).value)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                b"first\xff-next"
            );

            format_add_owned_cb(ft, CStr::from_ptr(key), cached_test_value);
            assert_eq!(
                format_entry_tree_find(&raw mut (*ft).tree, &raw mut probe),
                entry
            );
            assert!((*entry).value.is_none());
            format_entry_ensure_value(ft, entry);
            assert_eq!(
                CStr::from_ptr(
                    ((*entry).value)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                b"cached\xff"
            );

            let mut tv = timeval {
                tv_sec: 123,
                tv_usec: 0,
            };
            format_add_tv(ft, key, &raw mut tv);
            assert_eq!(
                format_entry_tree_find(&raw mut (*ft).tree, &raw mut probe),
                entry
            );
            assert!((*entry).value.is_none());
            assert_eq!((*entry).time, 123);

            format_add(
                ft,
                key,
                format,
                b"final\0".as_ptr() as *const ::core::ffi::c_char,
            );
            assert_eq!(
                format_entry_tree_find(&raw mut (*ft).tree, &raw mut probe),
                entry
            );
            assert_eq!(
                CStr::from_ptr(
                    ((*entry).value)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut())
                )
                .to_bytes(),
                b"final"
            );
            format_free(ft);
        }
    }
}

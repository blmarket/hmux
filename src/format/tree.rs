// Private tree-storage implementation.  It owns the format-entry tree and
// format-tree CRUD operations.
// Shared C-layout types, allocator/FFI helpers, and callback-table symbols
// remain supplied by the parent facade.
use super::*;
use std::ffi::{CStr, CString};

/// The C-layout prefix is borrowed by format lookups and callbacks. This box
/// owns both strings until `format_free`; replacing a value does not move the
/// prefix or its key.
#[repr(C)]
struct FormatEntryOwner {
    entry: format_entry,
    key: CString,
    value: Option<CString>,
}
const _: () = assert!(::core::mem::offset_of!(FormatEntryOwner, entry) == 0);

unsafe fn format_entry_new(key: *const ::core::ffi::c_char) -> *mut format_entry {
    let key = CStr::from_ptr(key).to_owned();
    let entry = format_entry {
        key: key.as_ptr() as *mut ::core::ffi::c_char,
        value: ::core::ptr::null_mut(),
        time: 0,
        cb: None,
        entry: format_entry_entry {
            rbe_left: ::core::ptr::null_mut(),
            rbe_right: ::core::ptr::null_mut(),
            rbe_parent: ::core::ptr::null_mut(),
            rbe_color: 0,
        },
    };
    let owner = Box::new(FormatEntryOwner {
        entry,
        key,
        value: None,
    });
    Box::into_raw(owner) as *mut format_entry
}

unsafe fn format_entry_set_value(fe: *mut format_entry, value: Option<CString>) {
    let owner = &mut *(fe as *mut FormatEntryOwner);
    owner.value = value;
    owner.entry.value = owner
        .value
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr() as *mut _);
}

/// Callback results have the existing malloc/free contract. Copy their C
/// string view into the entry owner, then release the callback allocation.
pub(super) unsafe fn format_entry_cache_callback(
    fe: *mut format_entry,
    value: *mut ::core::ffi::c_char,
) {
    let owned = if value.is_null() {
        CString::default()
    } else {
        let owned = CStr::from_ptr(value).to_owned();
        free(value as *mut ::core::ffi::c_void);
        owned
    };
    format_entry_set_value(fe, Some(owned));
}

unsafe fn format_entry_tree_key(elm: *mut format_entry) -> Vec<u8> {
    std::ffi::CStr::from_ptr((*elm).key).to_bytes().to_vec()
}

pub(super) unsafe fn format_entry_tree_find(
    head: *mut format_entry_tree,
    elm: *mut format_entry,
) -> *mut format_entry {
    if head.is_null() || (*head).entries.is_null() || elm.is_null() || (*elm).key.is_null() {
        return ::core::ptr::null_mut::<format_entry>();
    }
    (*(*head).entries)
        .entries
        .get(&format_entry_tree_key(elm))
        .copied()
        .unwrap_or(::core::ptr::null_mut::<format_entry>())
}

unsafe fn format_entry_tree_insert(
    head: *mut format_entry_tree,
    elm: *mut format_entry,
) -> *mut format_entry {
    if head.is_null() || elm.is_null() || (*elm).key.is_null() {
        return ::core::ptr::null_mut::<format_entry>();
    }
    if (*head).entries.is_null() {
        (*head).entries = Box::into_raw(Box::new(format_entry_tree_storage::default()));
    }
    match (*(*head).entries).entries.entry(format_entry_tree_key(elm)) {
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
    if head.is_null() || (*head).entries.is_null() || elm.is_null() || (*elm).key.is_null() {
        return ::core::ptr::null_mut::<format_entry>();
    }
    let key = format_entry_tree_key(elm);
    if (*(*head).entries).entries.get(&key).copied() != Some(elm) {
        return ::core::ptr::null_mut::<format_entry>();
    }
    (*(*head).entries)
        .entries
        .remove(&key)
        .unwrap_or(::core::ptr::null_mut::<format_entry>())
}

unsafe fn format_entry_tree_minmax(
    head: *mut format_entry_tree,
    val: ::core::ffi::c_int,
) -> *mut format_entry {
    if head.is_null() || (*head).entries.is_null() {
        return ::core::ptr::null_mut::<format_entry>();
    }
    let item = if val < 0 {
        (*(*head).entries).entries.values().next()
    } else {
        (*(*head).entries).entries.values().next_back()
    };
    item.copied()
        .unwrap_or(::core::ptr::null_mut::<format_entry>())
}

unsafe fn format_entry_tree_next(
    head: *mut format_entry_tree,
    elm: *mut format_entry,
) -> *mut format_entry {
    if head.is_null() || (*head).entries.is_null() || elm.is_null() || (*elm).key.is_null() {
        return ::core::ptr::null_mut::<format_entry>();
    }
    (*(*head).entries)
        .entries
        .range((
            std::ops::Bound::Excluded(format_entry_tree_key(elm)),
            std::ops::Bound::Unbounded,
        ))
        .next()
        .map(|(_, item)| *item)
        .unwrap_or(::core::ptr::null_mut::<format_entry>())
}

#[no_mangle]
pub unsafe extern "C" fn format_merge(mut ft: *mut format_tree, mut from: *mut format_tree) {
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    fe = format_entry_tree_minmax(&raw mut (*from).tree, RB_NEGINF);
    while !fe.is_null() {
        if !(*fe).value.is_null() {
            format_add(
                ft,
                (*fe).key,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*fe).value,
            );
        }
        fe = format_entry_tree_next(&raw mut (*from).tree, fe);
    }
}
#[no_mangle]
pub unsafe extern "C" fn format_get_pane(mut ft: *mut format_tree) -> *mut window_pane {
    return (*ft).wp;
}
pub(super) unsafe extern "C" fn format_create_add_item(
    mut ft: *mut format_tree,
    mut item: *mut cmdq_item,
) {
    let mut event: *mut key_event = cmdq_get_event(item);
    let mut m: *mut mouse_event = &raw mut (*event).m;
    cmdq_merge_formats(item, ft);
    memcpy(
        &raw mut (*ft).m as *mut ::core::ffi::c_void,
        m as *const ::core::ffi::c_void,
        ::core::mem::size_of::<mouse_event>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn format_create(
    mut c: *mut client,
    mut item: *mut cmdq_item,
    mut tag: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> *mut format_tree {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    ft = Box::into_raw(Box::new(::core::mem::zeroed::<format_tree>()));
    (*ft).tree.entries = Box::into_raw(Box::new(format_entry_tree_storage::default()));
    if !c.is_null() {
        (*ft).client = c;
        (*(*ft).client).references += 1;
    }
    (*ft).item = item;
    (*ft).tag = tag as u_int;
    (*ft).flags = flags;
    if !item.is_null() {
        format_create_add_item(ft, item);
    }
    return ft;
}
#[no_mangle]
pub unsafe extern "C" fn format_free(mut ft: *mut format_tree) {
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut fe1: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    fe = format_entry_tree_minmax(&raw mut (*ft).tree, RB_NEGINF);
    while !fe.is_null() && {
        fe1 = format_entry_tree_next(&raw mut (*ft).tree, fe);
        1 as ::core::ffi::c_int != 0
    } {
        format_entry_tree_remove(&raw mut (*ft).tree, fe);
        drop(Box::from_raw(fe as *mut FormatEntryOwner));
        fe = fe1;
    }
    if !(*ft).tree.entries.is_null() {
        drop(Box::from_raw((*ft).tree.entries));
        (*ft).tree.entries = ::core::ptr::null_mut::<format_entry_tree_storage>();
    }
    if !(*ft).client.is_null() {
        server_client_unref((*ft).client);
    }
    drop(Box::from_raw(ft));
}
pub(super) unsafe extern "C" fn format_log_debug_cb(
    mut key: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut prefix: *const ::core::ffi::c_char = arg as *const ::core::ffi::c_char;
    log_debug(
        b"%s: %s=%s\0" as *const u8 as *const ::core::ffi::c_char,
        prefix,
        key,
        value,
    );
}
#[no_mangle]
pub unsafe extern "C" fn format_log_debug(
    mut ft: *mut format_tree,
    mut prefix: *const ::core::ffi::c_char,
) {
    if log_get_level() == 0 as ::core::ffi::c_int {
        return;
    }
    format_each(
        ft,
        Some(
            format_log_debug_cb
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_char,
                    *const ::core::ffi::c_char,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        prefix as *mut ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn format_each(
    mut ft: *mut format_tree,
    mut cb: Option<
        unsafe extern "C" fn(
            *const ::core::ffi::c_char,
            *const ::core::ffi::c_char,
            *mut ::core::ffi::c_void,
        ) -> (),
    >,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut fte: *const format_table_entry = ::core::ptr::null::<format_table_entry>();
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut i: u_int = 0;
    let mut s: [::core::ffi::c_char; 64] = [0; 64];
    let mut value: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut tv: *mut timeval = ::core::ptr::null_mut::<timeval>();
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[format_table_entry; 214]>() as usize)
            .wrapping_div(::core::mem::size_of::<format_table_entry>() as usize)
    {
        fte = (&raw const format_table as *const format_table_entry).offset(i as isize)
            as *const format_table_entry;
        value = (*fte).cb.expect("non-null function pointer")(ft);
        if !value.is_null() {
            if (*fte).type_0 as ::core::ffi::c_uint
                == FORMAT_TABLE_TIME as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                tv = value as *mut timeval;
                xsnprintf(
                    &raw mut s as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                    b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                    (*tv).tv_sec as ::core::ffi::c_longlong,
                );
                cb.expect("non-null function pointer")(
                    (*fte).key,
                    &raw mut s as *mut ::core::ffi::c_char,
                    arg,
                );
            } else {
                cb.expect("non-null function pointer")(
                    (*fte).key,
                    value as *const ::core::ffi::c_char,
                    arg,
                );
                free(value);
            }
        }
        i = i.wrapping_add(1);
    }
    fe = format_entry_tree_minmax(&raw mut (*ft).tree, RB_NEGINF);
    while !fe.is_null() {
        if (*fe).time != 0 as time_t {
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                (*fe).time as ::core::ffi::c_longlong,
            );
            cb.expect("non-null function pointer")(
                (*fe).key,
                &raw mut s as *mut ::core::ffi::c_char,
                arg,
            );
        } else {
            if (*fe).value.is_null() && (*fe).cb.is_some() {
                let value =
                    (*fe).cb.expect("non-null function pointer")(ft) as *mut ::core::ffi::c_char;
                format_entry_cache_callback(fe, value);
            }
            cb.expect("non-null function pointer")((*fe).key, (*fe).value, arg);
        }
        fe = format_entry_tree_next(&raw mut (*ft).tree, fe);
    }
}
#[no_mangle]
pub unsafe extern "C" fn format_add(
    mut ft: *mut format_tree,
    mut key: *const ::core::ffi::c_char,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut fe_now: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut ap: ::core::ffi::VaList;
    fe = format_entry_new(key);
    fe_now = format_entry_tree_insert(&raw mut (*ft).tree, fe);
    if !fe_now.is_null() {
        drop(Box::from_raw(fe as *mut FormatEntryOwner));
        fe = fe_now;
    }
    (*fe).cb = None;
    (*fe).time = 0 as time_t;
    ap = args.clone();
    format_entry_set_value(fe, Some(xvasprintf_cstring(fmt, ap)));
}
#[no_mangle]
pub unsafe extern "C" fn format_add_tv(
    mut ft: *mut format_tree,
    mut key: *const ::core::ffi::c_char,
    mut tv: *mut timeval,
) {
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut fe_now: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    fe = format_entry_new(key);
    fe_now = format_entry_tree_insert(&raw mut (*ft).tree, fe);
    if !fe_now.is_null() {
        drop(Box::from_raw(fe as *mut FormatEntryOwner));
        fe = fe_now;
    }
    (*fe).cb = None;
    (*fe).time = (*tv).tv_sec as time_t;
    format_entry_set_value(fe, None);
}
#[no_mangle]
pub unsafe extern "C" fn format_add_cb(
    mut ft: *mut format_tree,
    mut key: *const ::core::ffi::c_char,
    mut cb: format_cb,
) {
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut fe_now: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    fe = format_entry_new(key);
    fe_now = format_entry_tree_insert(&raw mut (*ft).tree, fe);
    if !fe_now.is_null() {
        drop(Box::from_raw(fe as *mut FormatEntryOwner));
        fe = fe_now;
    }
    (*fe).cb = cb;
    (*fe).time = 0 as time_t;
    format_entry_set_value(fe, None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};

    unsafe extern "C" fn cached_test_value(_ft: *mut format_tree) -> *mut ::core::ffi::c_void {
        xstrdup(b"cached\xff\0".as_ptr() as *const ::core::ffi::c_char) as *mut ::core::ffi::c_void
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
                key: key as *mut _,
                value: ::core::ptr::null_mut(),
                time: 0,
                cb: None,
                entry: format_entry_entry {
                    rbe_left: ::core::ptr::null_mut(),
                    rbe_right: ::core::ptr::null_mut(),
                    rbe_parent: ::core::ptr::null_mut(),
                    rbe_color: 0,
                },
            };
            let entry = format_entry_tree_find(&raw mut (*ft).tree, &raw mut probe);
            assert!(!entry.is_null());
            assert_eq!(CStr::from_ptr((*entry).value).to_bytes(), b"first\xff");

            // The old value remains alive while vasprintf reads its argument.
            format_add(ft, key, b"%s-next\0".as_ptr() as *const _, (*entry).value);
            assert_eq!(
                format_entry_tree_find(&raw mut (*ft).tree, &raw mut probe),
                entry
            );
            assert_eq!(CStr::from_ptr((*entry).value).to_bytes(), b"first\xff-next");

            format_add_cb(ft, key, Some(cached_test_value));
            assert_eq!(
                format_entry_tree_find(&raw mut (*ft).tree, &raw mut probe),
                entry
            );
            assert!((*entry).value.is_null());
            let raw = (*entry).cb.expect("test callback is present")(ft) as *mut _;
            format_entry_cache_callback(entry, raw);
            assert_eq!(CStr::from_ptr((*entry).value).to_bytes(), b"cached\xff");
            format_entry_cache_callback(entry, ::core::ptr::null_mut());
            assert_eq!(CStr::from_ptr((*entry).value).to_bytes(), b"");

            let mut tv = timeval {
                tv_sec: 123,
                tv_usec: 0,
            };
            format_add_tv(ft, key, &raw mut tv);
            assert_eq!(
                format_entry_tree_find(&raw mut (*ft).tree, &raw mut probe),
                entry
            );
            assert!((*entry).value.is_null());
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
            assert_eq!(CStr::from_ptr((*entry).value).to_bytes(), b"final");
            format_free(ft);
        }
    }

    unsafe fn new_entry(key: &CString) -> *mut format_entry {
        Box::into_raw(Box::new(format_entry {
            key: key.as_ptr() as *mut ::core::ffi::c_char,
            value: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            time: 0,
            cb: None,
            entry: format_entry_entry {
                rbe_left: ::core::ptr::null_mut::<format_entry>(),
                rbe_right: ::core::ptr::null_mut::<format_entry>(),
                rbe_parent: ::core::ptr::null_mut::<format_entry>(),
                rbe_color: 0,
            },
        }))
    }

    unsafe fn free_tree(head: &mut format_entry_tree) {
        let mut item = format_entry_tree_minmax(head, RB_NEGINF);
        while !item.is_null() {
            let next = format_entry_tree_next(head, item);
            assert_eq!(format_entry_tree_remove(head, item), item);
            drop(Box::from_raw(item));
            item = next;
        }
        drop(Box::from_raw(head.entries));
        head.entries = ::core::ptr::null_mut::<format_entry_tree_storage>();
    }

    #[test]
    fn format_entry_tree_preserves_strcmp_order_and_duplicate_keys() {
        unsafe {
            let names: &[&[u8]] = &[b"zeta", b"alpha", b"alpha-2", b"\x80high", b"alpha\x01"];
            let keys: Vec<CString> = names
                .iter()
                .map(|name| CString::new(*name).expect("test key has no NUL"))
                .collect();
            assert!(crate::src::ffi::libc::strcmp(keys[1].as_ptr(), keys[2].as_ptr()) < 0);
            assert!(crate::src::ffi::libc::strcmp(keys[3].as_ptr(), keys[0].as_ptr()) > 0);
            let mut head = format_entry_tree {
                entries: Box::into_raw(Box::new(format_entry_tree_storage::default())),
            };
            let mut items = Vec::new();
            for key in &keys {
                let item = new_entry(key);
                assert!(format_entry_tree_insert(&mut head, item).is_null());
                items.push(item);
            }

            let duplicate_key = CString::new(b"alpha".as_slice()).unwrap();
            let duplicate = new_entry(&duplicate_key);
            assert_eq!(format_entry_tree_insert(&mut head, duplicate), items[1]);
            drop(Box::from_raw(duplicate));

            let probe_key = CString::new(b"alpha".as_slice()).unwrap();
            let probe = new_entry(&probe_key);
            assert_eq!(format_entry_tree_find(&mut head, probe), items[1]);
            drop(Box::from_raw(probe));

            assert_eq!(
                CStr::from_ptr((*format_entry_tree_minmax(&mut head, RB_NEGINF)).key).to_bytes(),
                b"alpha"
            );
            assert_eq!(
                CStr::from_ptr((*format_entry_tree_minmax(&mut head, RB_INF)).key).to_bytes(),
                b"\x80high"
            );

            let mut ordered = Vec::new();
            let mut item = format_entry_tree_minmax(&mut head, RB_NEGINF);
            while !item.is_null() {
                ordered.push(CStr::from_ptr((*item).key).to_bytes().to_vec());
                item = format_entry_tree_next(&mut head, item);
            }
            assert_eq!(
                ordered,
                vec![
                    b"alpha".to_vec(),
                    b"alpha\x01".to_vec(),
                    b"alpha-2".to_vec(),
                    b"zeta".to_vec(),
                    b"\x80high".to_vec(),
                ]
            );

            let removed = format_entry_tree_remove(&mut head, items[2]);
            assert_eq!(removed, items[2]);
            let removed_probe = new_entry(&keys[2]);
            assert!(format_entry_tree_find(&mut head, removed_probe).is_null());
            drop(Box::from_raw(removed_probe));
            drop(Box::from_raw(removed));
            free_tree(&mut head);
        }
    }
}

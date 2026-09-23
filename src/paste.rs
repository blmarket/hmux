use crate::src::events::events_fire;
use crate::src::events_payload::{event_payload_create, event_payload_set_string};
use crate::src::ffi::libc::{free, strlcpy, time};
use crate::src::options::options_get_number;
use crate::src::shared::abi::*;
pub use crate::src::shared::events::event_payload;
pub use crate::src::shared::options::options;
pub use crate::src::shared::paste::{
    paste_buffer, paste_buffer_name_entry, paste_buffer_time_entry,
};
pub use crate::src::shared::tree::{RB_BLACK, RB_INF, RB_NEGINF, RB_RED};
pub use crate::src::shared::vis::{VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB};
use crate::src::tmux::{clean_name, global_options};
use crate::src::utf8::utf8_strvis;
use crate::src::xmalloc::{xasprintf, xreallocarray, xstrdup};

#[derive(Default)]
pub struct paste_time_tree {
    entries: std::collections::BTreeMap<std::cmp::Reverse<u_int>, *mut paste_buffer>,
}
#[derive(Default)]
pub struct paste_name_tree {
    entries: std::collections::BTreeMap<Vec<u8>, *mut paste_buffer>,
}

static mut paste_next_index: u_int = 0;
static mut paste_next_order: u_int = 0;
static mut paste_num_automatic: u_int = 0;
static mut paste_by_name: paste_name_tree = paste_name_tree {
    entries: std::collections::BTreeMap::new(),
};
static mut paste_by_time: paste_time_tree = paste_time_tree {
    entries: std::collections::BTreeMap::new(),
};
unsafe fn paste_name_key(name: *const ::core::ffi::c_char) -> Vec<u8> {
    std::ffi::CStr::from_ptr(name).to_bytes().to_vec()
}

unsafe fn paste_name_tree_find(
    head: *mut paste_name_tree,
    name: *const ::core::ffi::c_char,
) -> *mut paste_buffer {
    (*head)
        .entries
        .get(&paste_name_key(name))
        .copied()
        .unwrap_or(::core::ptr::null_mut::<paste_buffer>())
}

unsafe fn paste_name_tree_insert(
    head: *mut paste_name_tree,
    elm: *mut paste_buffer,
) -> *mut paste_buffer {
    match (*head).entries.entry(paste_name_key((*elm).name)) {
        std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
            ::core::ptr::null_mut::<paste_buffer>()
        }
    }
}

unsafe fn paste_name_tree_remove(
    head: *mut paste_name_tree,
    elm: *mut paste_buffer,
) -> *mut paste_buffer {
    (*head)
        .entries
        .remove(&paste_name_key((*elm).name))
        .unwrap_or(::core::ptr::null_mut::<paste_buffer>())
}
// The original comparator puts larger order values first and treats equal
// orders as duplicates. Reverse gives BTreeMap the same ordering and Entry
// preserves RB_INSERT's existing-item result for duplicate keys.
unsafe fn paste_time_key(pb: *const paste_buffer) -> std::cmp::Reverse<u_int> {
    std::cmp::Reverse((*pb).order)
}

unsafe fn paste_time_tree_insert(
    head: *mut paste_time_tree,
    elm: *mut paste_buffer,
) -> *mut paste_buffer {
    match (*head).entries.entry(paste_time_key(elm)) {
        std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
            ::core::ptr::null_mut::<paste_buffer>()
        }
    }
}

unsafe fn paste_time_tree_minmax(
    head: *mut paste_time_tree,
    val: ::core::ffi::c_int,
) -> *mut paste_buffer {
    let entry = if val < 0 {
        (*head).entries.values().next()
    } else {
        (*head).entries.values().next_back()
    };
    entry
        .copied()
        .unwrap_or(::core::ptr::null_mut::<paste_buffer>())
}

unsafe fn paste_time_tree_next(
    head: *mut paste_time_tree,
    elm: *mut paste_buffer,
) -> *mut paste_buffer {
    (*head)
        .entries
        .range((
            std::ops::Bound::Excluded(paste_time_key(elm)),
            std::ops::Bound::Unbounded,
        ))
        .next()
        .map(|(_, entry)| *entry)
        .unwrap_or(::core::ptr::null_mut::<paste_buffer>())
}

unsafe fn paste_time_tree_prev(
    head: *mut paste_time_tree,
    elm: *mut paste_buffer,
) -> *mut paste_buffer {
    (*head)
        .entries
        .range((
            std::ops::Bound::Unbounded,
            std::ops::Bound::Excluded(paste_time_key(elm)),
        ))
        .next_back()
        .map(|(_, entry)| *entry)
        .unwrap_or(::core::ptr::null_mut::<paste_buffer>())
}

unsafe fn paste_time_tree_remove(
    head: *mut paste_time_tree,
    elm: *mut paste_buffer,
) -> *mut paste_buffer {
    (*head)
        .entries
        .remove(&paste_time_key(elm))
        .unwrap_or(::core::ptr::null_mut::<paste_buffer>())
}
unsafe extern "C" fn paste_fire_event(
    mut name: *const ::core::ffi::c_char,
    mut pbname: *const ::core::ffi::c_char,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    ep = event_payload_create();
    event_payload_set_string(
        ep,
        b"paste_buffer\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        pbname,
    );
    events_fire(name, ep);
}
#[no_mangle]
pub unsafe extern "C" fn paste_buffer_name(
    mut pb: *mut paste_buffer,
) -> *const ::core::ffi::c_char {
    return (*pb).name;
}
#[no_mangle]
pub unsafe extern "C" fn paste_buffer_order(mut pb: *mut paste_buffer) -> u_int {
    return (*pb).order;
}
#[no_mangle]
pub unsafe extern "C" fn paste_buffer_created(mut pb: *mut paste_buffer) -> time_t {
    return (*pb).created;
}
#[no_mangle]
pub unsafe extern "C" fn paste_buffer_data(
    mut pb: *mut paste_buffer,
    mut size: *mut size_t,
) -> *const ::core::ffi::c_char {
    if !size.is_null() {
        *size = (*pb).size;
    }
    return (*pb).data;
}
#[no_mangle]
pub unsafe extern "C" fn paste_walk(mut pb: *mut paste_buffer) -> *mut paste_buffer {
    if pb.is_null() {
        return paste_time_tree_minmax(&raw mut paste_by_time, RB_NEGINF);
    }
    return paste_time_tree_next(&raw mut paste_by_time, pb);
}
#[no_mangle]
pub unsafe extern "C" fn paste_is_empty() -> ::core::ffi::c_int {
    return paste_by_time.entries.is_empty() as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn paste_get_top(
    mut name: *mut *mut ::core::ffi::c_char,
) -> *mut paste_buffer {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    pb = paste_time_tree_minmax(&raw mut paste_by_time, RB_NEGINF);
    while !pb.is_null() && (*pb).automatic == 0 {
        pb = paste_time_tree_next(&raw mut paste_by_time, pb);
    }
    if pb.is_null() {
        return ::core::ptr::null_mut::<paste_buffer>();
    }
    if !name.is_null() {
        *name = xstrdup((*pb).name);
    }
    return pb;
}
#[no_mangle]
pub unsafe extern "C" fn paste_get_name(mut name: *const ::core::ffi::c_char) -> *mut paste_buffer {
    if name.is_null() || *name as ::core::ffi::c_int == '\0' as i32 {
        return ::core::ptr::null_mut::<paste_buffer>();
    }
    return paste_name_tree_find(&raw mut paste_by_name, name);
}
#[no_mangle]
pub unsafe extern "C" fn paste_free(mut pb: *mut paste_buffer) {
    paste_fire_event(
        b"paste-buffer-deleted\0" as *const u8 as *const ::core::ffi::c_char,
        (*pb).name,
    );
    paste_name_tree_remove(&raw mut paste_by_name, pb);
    paste_time_tree_remove(&raw mut paste_by_time, pb);
    if (*pb).automatic != 0 {
        paste_num_automatic = paste_num_automatic.wrapping_sub(1);
    }
    free((*pb).data as *mut ::core::ffi::c_void);
    free((*pb).name as *mut ::core::ffi::c_void);
    drop(Box::from_raw(pb));
}
#[no_mangle]
pub unsafe extern "C" fn paste_add(
    mut prefix: *const ::core::ffi::c_char,
    mut data: *mut ::core::ffi::c_char,
    mut size: size_t,
) {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut pb1: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut limit: u_int = 0;
    if prefix.is_null() {
        prefix = b"buffer\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if size == 0 as size_t {
        free(data as *mut ::core::ffi::c_void);
        return;
    }
    limit = options_get_number(
        global_options,
        b"buffer-limit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    pb = paste_time_tree_minmax(&raw mut paste_by_time, RB_INF);
    while !pb.is_null() && {
        pb1 = paste_time_tree_prev(&raw mut paste_by_time, pb);
        1 as ::core::ffi::c_int != 0
    } {
        if paste_num_automatic < limit {
            break;
        }
        if (*pb).automatic != 0 {
            paste_free(pb);
        }
        pb = pb1;
    }
    pb = Box::into_raw(Box::new(::core::mem::zeroed::<paste_buffer>()));
    (*pb).name = ::core::ptr::null_mut::<::core::ffi::c_char>();
    loop {
        free((*pb).name as *mut ::core::ffi::c_void);
        xasprintf(
            &raw mut (*pb).name,
            b"%s%u\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
            paste_next_index,
        );
        paste_next_index = paste_next_index.wrapping_add(1);
        if paste_get_name((*pb).name).is_null() {
            break;
        }
    }
    (*pb).data = data;
    (*pb).size = size;
    (*pb).automatic = 1 as ::core::ffi::c_int;
    paste_num_automatic = paste_num_automatic.wrapping_add(1);
    (*pb).created = time(::core::ptr::null_mut::<time_t>());
    let fresh0 = paste_next_order;
    paste_next_order = paste_next_order.wrapping_add(1);
    (*pb).order = fresh0;
    paste_name_tree_insert(&raw mut paste_by_name, pb);
    paste_time_tree_insert(&raw mut paste_by_time, pb);
    paste_fire_event(
        b"paste-buffer-changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*pb).name,
    );
}
#[no_mangle]
pub unsafe extern "C" fn paste_rename(
    mut oldname: *const ::core::ffi::c_char,
    mut newname: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut pb_new: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !cause.is_null() {
        *cause = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if oldname.is_null() || *oldname as ::core::ffi::c_int == '\0' as i32 {
        if !cause.is_null() {
            *cause = xstrdup(b"no buffer\0" as *const u8 as *const ::core::ffi::c_char);
        }
        return -(1 as ::core::ffi::c_int);
    }
    if newname.is_null() || *newname as ::core::ffi::c_int == '\0' as i32 {
        if !cause.is_null() {
            *cause = xstrdup(b"new name is empty\0" as *const u8 as *const ::core::ffi::c_char);
        }
        return -(1 as ::core::ffi::c_int);
    }
    name = clean_name(newname, 0 as ::core::ffi::c_int);
    if name.is_null() {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"invalid buffer name: %s\0" as *const u8 as *const ::core::ffi::c_char,
                newname,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    pb = paste_get_name(oldname);
    if pb.is_null() {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"no buffer %s\0" as *const u8 as *const ::core::ffi::c_char,
                oldname,
            );
        }
        free(name as *mut ::core::ffi::c_void);
        return -(1 as ::core::ffi::c_int);
    }
    pb_new = paste_get_name(name);
    if pb_new == pb {
        free(name as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if !pb_new.is_null() {
        paste_free(pb_new);
    }
    paste_name_tree_remove(&raw mut paste_by_name, pb);
    free((*pb).name as *mut ::core::ffi::c_void);
    (*pb).name = name;
    if (*pb).automatic != 0 {
        paste_num_automatic = paste_num_automatic.wrapping_sub(1);
    }
    (*pb).automatic = 0 as ::core::ffi::c_int;
    paste_name_tree_insert(&raw mut paste_by_name, pb);
    paste_fire_event(
        b"paste-buffer-deleted\0" as *const u8 as *const ::core::ffi::c_char,
        oldname,
    );
    paste_fire_event(
        b"paste-buffer-changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*pb).name,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn paste_set(
    mut data: *mut ::core::ffi::c_char,
    mut size: size_t,
    mut name: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut old: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut newname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !cause.is_null() {
        *cause = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if size == 0 as size_t {
        free(data as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if name.is_null() {
        paste_add(::core::ptr::null::<::core::ffi::c_char>(), data, size);
        return 0 as ::core::ffi::c_int;
    }
    if *name as ::core::ffi::c_int == '\0' as i32 {
        if !cause.is_null() {
            *cause = xstrdup(b"empty buffer name\0" as *const u8 as *const ::core::ffi::c_char);
        }
        return -(1 as ::core::ffi::c_int);
    }
    newname = clean_name(name, 0 as ::core::ffi::c_int);
    if newname.is_null() {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"invalid buffer name: %s\0" as *const u8 as *const ::core::ffi::c_char,
                name,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    pb = Box::into_raw(Box::new(::core::mem::zeroed::<paste_buffer>()));
    (*pb).name = newname;
    (*pb).data = data;
    (*pb).size = size;
    (*pb).automatic = 0 as ::core::ffi::c_int;
    let fresh1 = paste_next_order;
    paste_next_order = paste_next_order.wrapping_add(1);
    (*pb).order = fresh1;
    (*pb).created = time(::core::ptr::null_mut::<time_t>());
    old = paste_get_name((*pb).name);
    if !old.is_null() {
        paste_free(old);
    }
    paste_name_tree_insert(&raw mut paste_by_name, pb);
    paste_time_tree_insert(&raw mut paste_by_time, pb);
    paste_fire_event(
        b"paste-buffer-changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*pb).name,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn paste_replace(
    mut pb: *mut paste_buffer,
    mut data: *mut ::core::ffi::c_char,
    mut size: size_t,
) {
    free((*pb).data as *mut ::core::ffi::c_void);
    (*pb).data = data;
    (*pb).size = size;
    paste_fire_event(
        b"paste-buffer-changed\0" as *const u8 as *const ::core::ffi::c_char,
        (*pb).name,
    );
}
#[no_mangle]
pub unsafe extern "C" fn paste_make_sample(mut pb: *mut paste_buffer) -> *mut ::core::ffi::c_char {
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut used: size_t = 0;
    let flags: ::core::ffi::c_int = VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL;
    let width: size_t = 200 as size_t;
    len = (*pb).size;
    if len > width {
        len = width;
    }
    buf = xreallocarray(
        NULL,
        len,
        (4 as ::core::ffi::c_int + 4 as ::core::ffi::c_int) as size_t,
    ) as *mut ::core::ffi::c_char;
    used = utf8_strvis(buf, (*pb).data, len, flags);
    if (*pb).size > width || used > width {
        strlcpy(
            buf.offset(width as isize),
            b"...\0" as *const u8 as *const ::core::ffi::c_char,
            4 as size_t,
        );
    }
    return buf;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn named_buffer(name: &CString) -> Box<paste_buffer> {
        Box::new(paste_buffer {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
            name: name.as_ptr() as *mut ::core::ffi::c_char,
            created: 0,
            automatic: 0,
            order: 0,
            name_entry: paste_buffer_name_entry {
                rbe_left: ::core::ptr::null_mut::<paste_buffer>(),
                rbe_right: ::core::ptr::null_mut::<paste_buffer>(),
                rbe_parent: ::core::ptr::null_mut::<paste_buffer>(),
                rbe_color: 0,
            },
            time_entry: paste_buffer_time_entry {
                rbe_left: ::core::ptr::null_mut::<paste_buffer>(),
                rbe_right: ::core::ptr::null_mut::<paste_buffer>(),
                rbe_parent: ::core::ptr::null_mut::<paste_buffer>(),
                rbe_color: 0,
            },
        })
    }

    fn ordered_buffer(name: &CString, order: u_int) -> Box<paste_buffer> {
        let mut buffer = named_buffer(name);
        buffer.order = order;
        buffer
    }

    #[test]
    fn paste_name_tree_matches_strcmp_order_and_duplicate_semantics() {
        let names = [
            CString::new("z").unwrap(),
            CString::new("a").unwrap(),
            CString::new(vec![b'a', 0xff]).unwrap(),
            CString::new("a0").unwrap(),
        ];
        let mut items = names.iter().map(named_buffer).collect::<Vec<_>>();
        let duplicate_name = CString::new("a").unwrap();
        let mut duplicate = named_buffer(&duplicate_name);
        let mut tree = paste_name_tree::default();

        unsafe {
            let z = items[0].as_mut() as *mut paste_buffer;
            let a = items[1].as_mut() as *mut paste_buffer;
            let a_high = items[2].as_mut() as *mut paste_buffer;
            let a0 = items[3].as_mut() as *mut paste_buffer;
            let duplicate = duplicate.as_mut() as *mut paste_buffer;

            assert!(paste_name_tree_insert(&raw mut tree, z).is_null());
            assert!(paste_name_tree_insert(&raw mut tree, a_high).is_null());
            assert!(paste_name_tree_insert(&raw mut tree, a).is_null());
            assert!(paste_name_tree_insert(&raw mut tree, a0).is_null());
            assert_eq!(
                paste_name_tree_insert(&raw mut tree, duplicate),
                a,
                "duplicate names keep the original item"
            );

            let ordered = tree.entries.keys().map(Vec::as_slice).collect::<Vec<_>>();
            assert_eq!(
                ordered,
                vec![&b"a"[..], &b"a0"[..], &b"a\xff"[..], &b"z"[..]]
            );
            assert_eq!(
                paste_name_tree_find(&raw mut tree, names[2].as_ptr()),
                a_high
            );
            assert!(paste_name_tree_find(&raw mut tree, b"missing\0".as_ptr().cast()).is_null());

            assert_eq!(paste_name_tree_remove(&raw mut tree, a_high), a_high);
            assert!(paste_name_tree_find(&raw mut tree, names[2].as_ptr()).is_null());
        }
    }

    #[test]
    fn paste_time_tree_matches_reverse_order_and_duplicate_semantics() {
        let names = [
            CString::new("oldest").unwrap(),
            CString::new("middle").unwrap(),
            CString::new("newest").unwrap(),
        ];
        let mut items = [
            ordered_buffer(&names[0], 4),
            ordered_buffer(&names[1], 7),
            ordered_buffer(&names[2], 12),
        ];
        let duplicate_name = CString::new("duplicate").unwrap();
        let mut duplicate = ordered_buffer(&duplicate_name, 7);
        let mut tree = paste_time_tree::default();

        unsafe {
            let oldest = items[0].as_mut() as *mut paste_buffer;
            let middle = items[1].as_mut() as *mut paste_buffer;
            let newest = items[2].as_mut() as *mut paste_buffer;
            let duplicate = duplicate.as_mut() as *mut paste_buffer;

            assert!(paste_time_tree_insert(&raw mut tree, oldest).is_null());
            assert!(paste_time_tree_insert(&raw mut tree, newest).is_null());
            assert!(paste_time_tree_insert(&raw mut tree, middle).is_null());
            assert_eq!(paste_time_tree_insert(&raw mut tree, duplicate), middle);

            assert_eq!(
                (*paste_time_tree_minmax(&raw mut tree, RB_NEGINF)).order,
                12
            );
            assert_eq!((*paste_time_tree_next(&raw mut tree, newest)).order, 7);
            assert_eq!((*paste_time_tree_next(&raw mut tree, middle)).order, 4);
            assert!(paste_time_tree_next(&raw mut tree, oldest).is_null());

            assert_eq!((*paste_time_tree_minmax(&raw mut tree, RB_INF)).order, 4);
            assert_eq!((*paste_time_tree_prev(&raw mut tree, oldest)).order, 7);
            assert_eq!((*paste_time_tree_prev(&raw mut tree, middle)).order, 12);
            assert!(paste_time_tree_prev(&raw mut tree, newest).is_null());

            assert_eq!(paste_time_tree_remove(&raw mut tree, middle), middle);
            assert_eq!((*paste_time_tree_next(&raw mut tree, newest)).order, 4);
        }
    }
}

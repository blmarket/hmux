use std::cell::RefCell;
use crate::src::events::events_fire;
use crate::src::events_payload::{event_payload_create, event_payload_set_string};
use crate::src::ffi::libc::{free, time};
use crate::src::options::options_get_number;
use crate::src::shared::abi::*;
use crate::src::shared::events::event_payload;
use crate::src::shared::options::options;
use crate::src::shared::paste::{
    paste_buffer, paste_buffer_name_entry, paste_buffer_time_entry,
};
use crate::src::shared::tree::{RB_BLACK, RB_INF, RB_NEGINF, RB_RED};
use crate::src::shared::vis::{VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB};
use crate::src::tmux::{clean_name_cstring, global_options};
use crate::src::text::utf8::utf8_strvis;
use std::ffi::{CStr, CString};

macro_rules! set_paste_cause {
    ($cause:expr, $value:expr) => {{
        let cause = $cause;
        if !cause.is_null() {
            *cause = Some($value);
        }
    }};
}

unsafe fn paste_name_cause(
    cause: *mut Option<CString>,
    prefix: &[u8],
    name: *const ::core::ffi::c_char,
) {
    if cause.is_null() {
        return;
    }
    let mut message = prefix.to_vec();
    message.extend_from_slice(CStr::from_ptr(name).to_bytes());
    *cause = Some(CString::new(message).expect("paste diagnostic contains no NUL"));
}

#[derive(Default)]
pub struct paste_time_tree {
    entries: std::collections::BTreeMap<std::cmp::Reverse<u_int>, *mut paste_buffer>,
}
#[derive(Default)]
pub struct paste_name_tree {
    entries: std::collections::BTreeMap<Vec<u8>, *mut paste_buffer>,
}

thread_local! {
    static paste_next_index: RefCell<u_int> = RefCell::new(0);
    static paste_next_order: RefCell<u_int> = RefCell::new(0);
    static paste_num_automatic: RefCell<u_int> = RefCell::new(0);
    static paste_by_name: RefCell<paste_name_tree> = RefCell::new(paste_name_tree {
        entries: std::collections::BTreeMap::new(),
    });
    static paste_by_time: RefCell<paste_time_tree> = RefCell::new(paste_time_tree {
        entries: std::collections::BTreeMap::new(),
    });
}

unsafe fn paste_new_owned(name: CString) -> *mut paste_buffer {
    let mut owner = Box::new(paste_buffer {
        name: name,
        data: None,
        ..paste_buffer::empty()
    });

    Box::into_raw(owner).cast::<paste_buffer>()
}

/// Accepted producer allocations are copied into the owner and released here.
/// Copy before replacing the old owner data so source bytes remain readable.
unsafe fn paste_take_data(pb: *mut paste_buffer, data: *mut ::core::ffi::c_char, size: size_t) {
    let owned = if size == 0 {
        None
    } else {
        Some(std::slice::from_raw_parts(data.cast::<u8>(), size).into())
    };
    paste_store_data(&mut *pb, owned);
    free(data.cast());
}

fn paste_store_data(pb: &mut paste_buffer, data: Option<Box<[u8]>>) {
    let size = data.as_ref().map_or(0, |bytes| bytes.len());
    pb.data = data;

    pb.size = size;
}

fn paste_replace_name(pb: &mut paste_buffer, name: CString) -> CString {
    std::mem::replace(&mut pb.name, name)
}

fn paste_name_key(name: &CStr) -> Vec<u8> {
    name.to_bytes().to_vec()
}

fn paste_name_tree_find(head: &paste_name_tree, name: &CStr) -> *mut paste_buffer {
    head
        .entries
        .get(&paste_name_key(name))
        .copied()
        .unwrap_or(::core::ptr::null_mut::<paste_buffer>())
}

unsafe fn paste_name_tree_insert(
    head: *mut paste_name_tree,
    elm: *mut paste_buffer,
) -> *mut paste_buffer {
    match (*head).entries.entry(paste_name_key((*elm).name.as_c_str())) {
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
        .remove(&paste_name_key((*elm).name.as_c_str()))
        .unwrap_or(::core::ptr::null_mut::<paste_buffer>())
}
// The original comparator puts larger order values first and treats equal
// orders as duplicates. Reverse gives BTreeMap the same ordering and Entry
// preserves RB_INSERT's existing-item result for duplicate keys.
fn paste_time_key(pb: &paste_buffer) -> std::cmp::Reverse<u_int> {
    std::cmp::Reverse(pb.order)
}

unsafe fn paste_time_tree_insert(
    head: *mut paste_time_tree,
    elm: *mut paste_buffer,
) -> *mut paste_buffer {
    match (*head).entries.entry(paste_time_key(&*elm)) {
        std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
            ::core::ptr::null_mut::<paste_buffer>()
        }
    }
}

fn paste_time_tree_minmax(head: &paste_time_tree, val: ::core::ffi::c_int) -> *mut paste_buffer {
    let entry = if val < 0 {
        head.entries.values().next()
    } else {
        head.entries.values().next_back()
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
            std::ops::Bound::Excluded(paste_time_key(&*elm)),
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
            std::ops::Bound::Excluded(paste_time_key(&*elm)),
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
        .remove(&paste_time_key(&*elm))
        .unwrap_or(::core::ptr::null_mut::<paste_buffer>())
}

unsafe fn paste_name_tree_find_local(name: &CStr) -> *mut paste_buffer {
    paste_by_name.with(|head| {
        let head = head.borrow();
        paste_name_tree_find(&head, name)
    })
}

unsafe fn paste_name_tree_insert_local(elm: *mut paste_buffer) -> *mut paste_buffer {
    paste_by_name.with(|head| {
        let mut head = head.borrow_mut();
        paste_name_tree_insert(&mut *head, elm)
    })
}

unsafe fn paste_name_tree_remove_local(elm: *mut paste_buffer) -> *mut paste_buffer {
    paste_by_name.with(|head| {
        let mut head = head.borrow_mut();
        paste_name_tree_remove(&mut *head, elm)
    })
}

unsafe fn paste_time_tree_insert_local(elm: *mut paste_buffer) -> *mut paste_buffer {
    paste_by_time.with(|head| {
        let mut head = head.borrow_mut();
        paste_time_tree_insert(&mut *head, elm)
    })
}

unsafe fn paste_time_tree_minmax_local(val: ::core::ffi::c_int) -> *mut paste_buffer {
    paste_by_time.with(|head| {
        let head = head.borrow();
        paste_time_tree_minmax(&head, val)
    })
}

unsafe fn paste_time_tree_next_local(elm: *mut paste_buffer) -> *mut paste_buffer {
    paste_by_time.with(|head| {
        let mut head = head.borrow_mut();
        paste_time_tree_next(&mut *head, elm)
    })
}

unsafe fn paste_time_tree_prev_local(elm: *mut paste_buffer) -> *mut paste_buffer {
    paste_by_time.with(|head| {
        let mut head = head.borrow_mut();
        paste_time_tree_prev(&mut *head, elm)
    })
}

unsafe fn paste_time_tree_remove_local(elm: *mut paste_buffer) -> *mut paste_buffer {
    paste_by_time.with(|head| {
        let mut head = head.borrow_mut();
        paste_time_tree_remove(&mut *head, elm)
    })
}

fn paste_time_tree_is_empty_local() -> bool {
    paste_by_time.with(|head| head.borrow().entries.is_empty())
}

fn paste_automatic_count() -> u_int {
    paste_num_automatic.with(|count| *count.borrow())
}

fn paste_automatic_count_increment() {
    paste_num_automatic.with(|count| {
        let mut count = count.borrow_mut();
        *count = (*count).wrapping_add(1);
    });
}

fn paste_automatic_count_decrement() {
    paste_num_automatic.with(|count| {
        let mut count = count.borrow_mut();
        *count = (*count).wrapping_sub(1);
    });
}

fn paste_next_index_take() -> u_int {
    paste_next_index.with(|index| {
        let mut index = index.borrow_mut();
        let current = *index;
        *index = (*index).wrapping_add(1);
        current
    })
}

fn paste_next_order_take() -> u_int {
    paste_next_order.with(|order| {
        let mut order = order.borrow_mut();
        let current = *order;
        *order = (*order).wrapping_add(1);
        current
    })
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
    return ((*pb).name).as_ptr().cast_mut();
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
    return ((*pb).data)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| {
            value.as_ptr().cast_mut().cast::<::core::ffi::c_char>()
        });
}
#[no_mangle]
pub unsafe extern "C" fn paste_walk(mut pb: *mut paste_buffer) -> *mut paste_buffer {
    if pb.is_null() {
        return paste_time_tree_minmax_local(RB_NEGINF);
    }
    return paste_time_tree_next_local(pb);
}
#[no_mangle]
pub unsafe extern "C" fn paste_is_empty() -> ::core::ffi::c_int {
    return paste_time_tree_is_empty_local() as ::core::ffi::c_int;
}
pub(crate) unsafe fn paste_get_top(name: Option<&mut Option<CString>>) -> *mut paste_buffer {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    pb = paste_time_tree_minmax_local(RB_NEGINF);
    while !pb.is_null() && (*pb).automatic == 0 {
        pb = paste_time_tree_next_local(pb);
    }
    if pb.is_null() {
        return ::core::ptr::null_mut::<paste_buffer>();
    }
    if let Some(name) = name {
        *name = Some((*pb).name.clone());
    }
    return pb;
}
#[no_mangle]
pub unsafe extern "C" fn paste_get_name(mut name: *const ::core::ffi::c_char) -> *mut paste_buffer {
    if name.is_null() || *name as ::core::ffi::c_int == '\0' as i32 {
        return ::core::ptr::null_mut::<paste_buffer>();
    }
    return paste_name_tree_find_local(CStr::from_ptr(name));
}
#[no_mangle]
pub unsafe extern "C" fn paste_free(mut pb: *mut paste_buffer) {
    paste_fire_event(
        b"paste-buffer-deleted\0" as *const u8 as *const ::core::ffi::c_char,
        ((*pb).name).as_ptr().cast_mut(),
    );
    paste_name_tree_remove_local(pb);
    paste_time_tree_remove_local(pb);
    if (*pb).automatic != 0 {
        paste_automatic_count_decrement();
    }
    drop(Box::from_raw(pb));
}
#[no_mangle]
pub unsafe extern "C" fn paste_add(
    prefix: *const ::core::ffi::c_char,
    data: *mut ::core::ffi::c_char,
    size: size_t,
) {
    if size == 0 {
        free(data.cast());
        return;
    }
    let owned = std::slice::from_raw_parts(data.cast::<u8>(), size).into();
    let prefix = if prefix.is_null() {
        None
    } else {
        Some(CStr::from_ptr(prefix).to_owned())
    };
    paste_add_owned(prefix, owned);
    free(data.cast());
}

pub(crate) unsafe fn paste_add_owned(prefix: Option<CString>, data: Box<[u8]>) {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut pb1: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut limit: u_int = 0;
    if data.is_empty() {
        return;
    }
    let prefix_bytes = prefix
        .as_ref()
        .map_or(b"buffer".as_slice(), CString::as_bytes);
    limit = options_get_number(
        global_options,
        b"buffer-limit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    pb = paste_time_tree_minmax_local(RB_INF);
    while !pb.is_null() && {
        pb1 = paste_time_tree_prev_local(pb);
        1 as ::core::ffi::c_int != 0
    } {
        if paste_automatic_count() < limit {
            break;
        }
        if (*pb).automatic != 0 {
            paste_free(pb);
        }
        pb = pb1;
    }
    loop {
        let mut bytes = Vec::with_capacity(prefix_bytes.len() + 10);
        bytes.extend_from_slice(&prefix_bytes);
        let index = paste_next_index_take();
        bytes.extend_from_slice(index.to_string().as_bytes());
        let name = CString::new(bytes).expect("generated buffer name has no NUL");
        if paste_get_name(name.as_ptr()).is_null() {
            pb = paste_new_owned(name);
            break;
        }
    }
    paste_store_data(&mut *pb, Some(data));
    (*pb).automatic = 1 as ::core::ffi::c_int;
    paste_automatic_count_increment();
    (*pb).created = time(::core::ptr::null_mut::<time_t>());
    let fresh0 = paste_next_order_take();
    (*pb).order = fresh0;
    paste_name_tree_insert_local(pb);
    paste_time_tree_insert_local(pb);
    paste_fire_event(
        b"paste-buffer-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ((*pb).name).as_ptr().cast_mut(),
    );
}
pub unsafe fn paste_rename(
    mut oldname: *const ::core::ffi::c_char,
    mut newname: *const ::core::ffi::c_char,
    mut cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut pb_new: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    if !cause.is_null() {
        *cause = None;
    }
    if oldname.is_null() || *oldname as ::core::ffi::c_int == '\0' as i32 {
        if !cause.is_null() {
            set_paste_cause!(cause, CString::new(b"no buffer".to_vec()).unwrap());
        }
        return -(1 as ::core::ffi::c_int);
    }
    if newname.is_null() || *newname as ::core::ffi::c_int == '\0' as i32 {
        if !cause.is_null() {
            set_paste_cause!(cause, CString::new(b"new name is empty".to_vec()).unwrap());
        }
        return -(1 as ::core::ffi::c_int);
    }
    let Some(name) = clean_name_cstring(CStr::from_ptr(newname), 0) else {
        if !cause.is_null() {
            paste_name_cause(cause, b"invalid buffer name: ", newname);
        }
        return -(1 as ::core::ffi::c_int);
    };
    pb = paste_get_name(oldname);
    if pb.is_null() {
        if !cause.is_null() {
            paste_name_cause(cause, b"no buffer ", oldname);
        }
        return -(1 as ::core::ffi::c_int);
    }
    pb_new = paste_get_name(name.as_ptr());
    if pb_new == pb {
        return 0 as ::core::ffi::c_int;
    }
    if !pb_new.is_null() {
        paste_free(pb_new);
    }
    paste_name_tree_remove_local(pb);
    let previous = paste_replace_name(&mut *pb, name);
    if (*pb).automatic != 0 {
        paste_automatic_count_decrement();
    }
    (*pb).automatic = 0 as ::core::ffi::c_int;
    paste_name_tree_insert_local(pb);
    paste_fire_event(
        b"paste-buffer-deleted\0" as *const u8 as *const ::core::ffi::c_char,
        previous.as_ptr(),
    );
    paste_fire_event(
        b"paste-buffer-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ((*pb).name).as_ptr().cast_mut(),
    );
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn paste_set(
    data: *mut ::core::ffi::c_char,
    size: size_t,
    name: *const ::core::ffi::c_char,
    cause: *mut Option<CString>,
) -> ::core::ffi::c_int {
    let owned = if size == 0 {
        Box::<[u8]>::default()
    } else {
        std::slice::from_raw_parts(data.cast::<u8>(), size).into()
    };
    paste_set_inner(owned, name, cause, data.cast())
}

pub(crate) unsafe fn paste_set_owned(
    data: Box<[u8]>,
    name: *const ::core::ffi::c_char,
    cause: Option<&mut Option<CString>>,
) -> ::core::ffi::c_int {
    let cause = cause.map_or_else(::core::ptr::null_mut, |cause| &raw mut *cause);
    paste_set_inner(data, name, cause, ::core::ptr::null_mut())
}

unsafe fn paste_set_inner(
    data: Box<[u8]>,
    name: *const ::core::ffi::c_char,
    cause: *mut Option<CString>,
    // Null for Rust callers; on a name error the C caller still owns this.
    c_producer: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut old: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    if !cause.is_null() {
        *cause = None;
    }
    if data.is_empty() {
        free(c_producer);
        return 0 as ::core::ffi::c_int;
    }
    if name.is_null() {
        paste_add_owned(None, data);
        free(c_producer);
        return 0 as ::core::ffi::c_int;
    }
    if *name as ::core::ffi::c_int == '\0' as i32 {
        if !cause.is_null() {
            set_paste_cause!(cause, CString::new(b"empty buffer name".to_vec()).unwrap());
        }
        return -(1 as ::core::ffi::c_int);
    }
    let Some(newname) = clean_name_cstring(CStr::from_ptr(name), 0) else {
        if !cause.is_null() {
            paste_name_cause(cause, b"invalid buffer name: ", name);
        }
        return -(1 as ::core::ffi::c_int);
    };
    pb = paste_new_owned(newname);
    paste_store_data(&mut *pb, Some(data));
    free(c_producer);
    (*pb).automatic = 0 as ::core::ffi::c_int;
    let fresh1 = paste_next_order_take();
    (*pb).order = fresh1;
    (*pb).created = time(::core::ptr::null_mut::<time_t>());
    old = paste_get_name(((*pb).name).as_ptr().cast_mut());
    if !old.is_null() {
        paste_free(old);
    }
    paste_name_tree_insert_local(pb);
    paste_time_tree_insert_local(pb);
    paste_fire_event(
        b"paste-buffer-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ((*pb).name).as_ptr().cast_mut(),
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn paste_replace(
    mut pb: *mut paste_buffer,
    mut data: *mut ::core::ffi::c_char,
    mut size: size_t,
) {
    paste_take_data(pb, data, size);
    paste_fire_event(
        b"paste-buffer-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ((*pb).name).as_ptr().cast_mut(),
    );
}
/// Replace a paste buffer with Rust-owned bytes from an internal caller.
pub(crate) unsafe fn paste_replace_owned(pb: &mut paste_buffer, data: Box<[u8]>) {
    paste_store_data(pb, Some(data));
    paste_fire_event(
        b"paste-buffer-changed\0" as *const u8 as *const ::core::ffi::c_char,
        pb.name.as_ptr().cast_mut(),
    );
}
pub(crate) unsafe fn paste_make_sample_cstring(pb: &paste_buffer) -> CString {
    let flags = VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL;
    let width = 200;
    let len = pb.size.min(width);
    let mut buffer = vec![0u8; len * 8 + 4];
    let used = utf8_strvis(
        buffer.as_mut_ptr().cast(),
        pb.data
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| {
                value.as_ptr().cast_mut().cast::<::core::ffi::c_char>()
            }),
        len,
        flags,
    );
    if pb.size > width || used > width {
        buffer[width..width + 4].copy_from_slice(b"...\0");
    }
    let length = CStr::from_ptr(buffer.as_ptr().cast())
        .to_bytes_with_nul()
        .len();
    buffer.truncate(length);
    CString::from_vec_with_nul(buffer).expect("sample contains one terminating NUL")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::src::ffi::libc::{malloc, strdup};
    use crate::src::options::{
        options_create, options_default, options_free, options_search, options_set_number,
    };
    use crate::src::tmux::global_options;
    use std::ffi::CString;

    unsafe fn allocated_bytes(bytes: &[u8]) -> *mut ::core::ffi::c_char {
        let data = malloc(bytes.len()).cast::<u8>();
        assert!(!data.is_null());
        ::core::ptr::copy_nonoverlapping(bytes.as_ptr(), data, bytes.len());
        data.cast()
    }

    #[test]
    fn owned_data_preserves_binary_bytes_across_replacement() {
        unsafe {
            let mut cause: Option<CString> = None;
            let name = c"owner-binary-data";
            let first = b"A\0B\xff";
            assert_eq!(
                paste_set(
                    allocated_bytes(first),
                    first.len(),
                    name.as_ptr(),
                    &raw mut cause
                ),
                0
            );
            assert!(cause.is_none());
            let pb = paste_get_name(name.as_ptr());
            let mut len = 0;
            let data = paste_buffer_data(pb, &raw mut len);
            assert_eq!(std::slice::from_raw_parts(data.cast::<u8>(), len), first);

            let second = b"\0\x80new";
            paste_replace(pb, allocated_bytes(second), second.len());
            let data = paste_buffer_data(pb, &raw mut len);
            assert_eq!(std::slice::from_raw_parts(data.cast::<u8>(), len), second);
            paste_free(pb);
        }
    }

    #[test]
    fn rename_accepts_borrowed_current_name() {
        unsafe {
            let mut cause: Option<CString> = None;
            assert_eq!(
                paste_set(
                    strdup(c"payload".as_ptr()),
                    7,
                    c"owner-rename-alias".as_ptr(),
                    &raw mut cause,
                ),
                0
            );
            assert!(cause.is_none());
            let pb = paste_get_name(c"owner-rename-alias".as_ptr());
            assert!(!pb.is_null());
            assert_eq!(
                paste_rename(((*pb).name).as_ptr().cast_mut(), c"owner-renamed".as_ptr(), &raw mut cause),
                0
            );
            assert!(cause.is_none());
            assert!(paste_get_name(c"owner-rename-alias".as_ptr()).is_null());
            assert_eq!(paste_get_name(c"owner-renamed".as_ptr()), pb);
            assert_eq!(
                CStr::from_ptr(((*pb).name).as_ptr().cast_mut()),
                c"owner-renamed"
            );
            paste_free(pb);
        }
    }

    #[test]
    fn invalid_names_return_owned_diagnostics() {
        unsafe {
            let mut cause: Option<CString> = Some(c"stale".to_owned());
            assert_eq!(
                paste_set_owned(
                    b"payload".to_vec().into_boxed_slice(),
                    c"".as_ptr(),
                    Some(&mut cause),
                ),
                -1
            );
            assert_eq!(
                CStr::from_ptr(cause.as_ref().unwrap().as_ptr()),
                c"empty buffer name"
            );

            cause = Some(c"stale".to_owned());
            let name = c"owned-cause-success";
            assert_eq!(
                paste_set_owned(
                    b"payload".to_vec().into_boxed_slice(),
                    name.as_ptr(),
                    Some(&mut cause),
                ),
                0
            );
            assert!(cause.is_none());
            paste_free(paste_get_name(name.as_ptr()));

            cause = Some(c"stale".to_owned());
            assert_eq!(
                paste_set_owned(
                    Box::<[u8]>::default(),
                    ::core::ptr::null(),
                    Some(&mut cause),
                ),
                0
            );
            assert!(cause.is_none(), "empty-data success clears the cause");

            assert_eq!(
                paste_set_owned(b"payload".to_vec().into_boxed_slice(), c"".as_ptr(), None,),
                -1,
                "failure can discard its diagnostic"
            );
            assert_eq!(
                paste_set_owned(
                    b"payload".to_vec().into_boxed_slice(),
                    c"owned-cause-none".as_ptr(),
                    None,
                ),
                0,
                "success can discard its diagnostic"
            );
            paste_free(paste_get_name(c"owned-cause-none".as_ptr()));

            cause = Some(c"stale".to_owned());
            assert_eq!(
                paste_rename(
                    ::core::ptr::null(),
                    c"renamed".as_ptr(),
                    &raw mut cause,
                ),
                -1
            );
            assert_eq!(
                CStr::from_ptr(cause.as_ref().unwrap().as_ptr()),
                c"no buffer"
            );
        }
    }

    #[test]
    fn raw_add_snapshots_optional_prefix_before_pruning() {
        unsafe {
            let previous_global_options = global_options;
            global_options = options_create(::core::ptr::null_mut());
            let buffer_limit = options_search(c"buffer-limit".as_ptr());
            assert!(!buffer_limit.is_null());
            options_default(global_options, buffer_limit);
            options_set_number(global_options, c"buffer-limit".as_ptr(), 50);

            // Empty data must return before attempting to read this invalid prefix.
            paste_add(
                1usize as *const ::core::ffi::c_char,
                ::core::ptr::null_mut(),
                0,
            );

            paste_add(::core::ptr::null(), allocated_bytes(b"x"), 1);
            let default_name = CStr::from_ptr(paste_buffer_name(paste_get_top(None)))
                .to_bytes()
                .to_vec();
            assert!(default_name.starts_with(b"buffer"));

            paste_add(c"".as_ptr(), allocated_bytes(b"x"), 1);
            let empty_name = CStr::from_ptr(paste_buffer_name(paste_get_top(None))).to_bytes();
            assert!(!empty_name.is_empty() && empty_name.iter().all(u8::is_ascii_digit));

            paste_add(c"custom-".as_ptr(), allocated_bytes(b"x"), 1);
            let custom_name = CStr::from_ptr(paste_buffer_name(paste_get_top(None))).to_bytes();
            assert!(custom_name.starts_with(b"custom-"));

            let binary_prefix = CString::new(b"binary-\xff".to_vec()).unwrap();
            paste_add(binary_prefix.as_ptr(), allocated_bytes(b"x"), 1);
            let binary_name = CStr::from_ptr(paste_buffer_name(paste_get_top(None))).to_bytes();
            assert!(binary_name.starts_with(b"binary-\xff"));

            options_set_number(global_options, c"buffer-limit".as_ptr(), 1);
            while paste_is_empty() == 0 {
                let pb = paste_time_tree_minmax_local(RB_NEGINF);
                assert!(!pb.is_null());
                paste_free(pb);
            }

            paste_add(c"prune-prefix-".as_ptr(), allocated_bytes(b"x"), 1);
            let old = paste_get_top(None);
            assert!(!old.is_null());
            let old_name = (*old).name.clone();
            paste_add(paste_buffer_name(old), allocated_bytes(b"x"), 1);

            assert!(paste_get_name(old_name.as_ptr()).is_null());
            let new_name = CStr::from_ptr(paste_buffer_name(paste_get_top(None))).to_bytes();
            assert!(new_name.starts_with(old_name.as_bytes()));

            while paste_is_empty() == 0 {
                let pb = paste_time_tree_minmax_local(RB_NEGINF);
                assert!(!pb.is_null());
                paste_free(pb);
            }
            options_free(global_options);
            global_options = previous_global_options;
        }
    }

    fn named_buffer(name: &CString) -> Box<paste_buffer> {
        Box::new(paste_buffer {
            data: Default::default(),
            size: 0,
            name: name.clone(),
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
                paste_name_tree_find(&tree, names[2].as_c_str()),
                a_high
            );
            assert!(paste_name_tree_find(
                &tree,
                CStr::from_bytes_with_nul(b"missing\0").unwrap()
            )
            .is_null());

            assert_eq!(paste_name_tree_remove(&raw mut tree, a_high), a_high);
            assert!(paste_name_tree_find(&tree, names[2].as_c_str()).is_null());
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
                (*paste_time_tree_minmax(&tree, RB_NEGINF)).order,
                12
            );
            assert_eq!((*paste_time_tree_next(&raw mut tree, newest)).order, 7);
            assert_eq!((*paste_time_tree_next(&raw mut tree, middle)).order, 4);
            assert!(paste_time_tree_next(&raw mut tree, oldest).is_null());

            assert_eq!((*paste_time_tree_minmax(&tree, RB_INF)).order, 4);
            assert_eq!((*paste_time_tree_prev(&raw mut tree, oldest)).order, 7);
            assert_eq!((*paste_time_tree_prev(&raw mut tree, middle)).order, 12);
            assert!(paste_time_tree_prev(&raw mut tree, newest).is_null());

            assert_eq!(paste_time_tree_remove(&raw mut tree, middle), middle);
            assert_eq!((*paste_time_tree_next(&raw mut tree, newest)).order, 4);
        }
    }
}

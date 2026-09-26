use crate::src::events::events_fire;
use crate::src::events_payload::{event_payload_create, event_payload_set_string};
use crate::src::ffi::libc::time;
use crate::src::options::options_get_number;
use crate::src::shared::abi::*;
use crate::src::shared::events::event_payload;
use crate::src::shared::paste::paste_buffer;
use crate::src::shared::tree::{RB_INF, RB_NEGINF};
use crate::src::shared::vis::{VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB};
use crate::src::text::utf8::utf8_strvis;
use crate::src::tmux::{clean_name_cstring, global_options};
use std::cell::RefCell;
use std::ffi::{CStr, CString};

macro_rules! set_paste_cause {
    ($cause:expr, $value:expr) => {{
        let cause = $cause;
        if !cause.is_null() {
            *cause = Some($value);
        }
    }};
}

unsafe fn paste_name_cause(cause: Option<&mut Option<CString>>, prefix: &[u8], name: &CStr) {
    let Some(cause) = cause else {
        return;
    };
    let mut message = prefix.to_vec();
    message.extend_from_slice(name.to_bytes());
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
    head.entries
        .get(&paste_name_key(name))
        .copied()
        .unwrap_or(::core::ptr::null_mut::<paste_buffer>())
}

unsafe fn paste_name_tree_insert(
    head: &mut paste_name_tree,
    elm: *mut paste_buffer,
) -> *mut paste_buffer {
    match head.entries.entry(paste_name_key((*elm).name.as_c_str())) {
        std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
            ::core::ptr::null_mut::<paste_buffer>()
        }
    }
}

fn paste_name_tree_remove(head: &mut paste_name_tree, elm: &paste_buffer) -> *mut paste_buffer {
    head.entries
        .remove(&paste_name_key(elm.name.as_c_str()))
        .unwrap_or(::core::ptr::null_mut::<paste_buffer>())
}
// The original comparator puts larger order values first and treats equal
// orders as duplicates. Reverse gives BTreeMap the same ordering and Entry
// preserves RB_INSERT's existing-item result for duplicate keys.
fn paste_time_key(pb: &paste_buffer) -> std::cmp::Reverse<u_int> {
    std::cmp::Reverse(pb.order)
}

unsafe fn paste_time_tree_insert(
    head: &mut paste_time_tree,
    elm: *mut paste_buffer,
) -> *mut paste_buffer {
    match head.entries.entry(paste_time_key(&*elm)) {
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

fn paste_time_tree_next(head: &paste_time_tree, elm: &paste_buffer) -> *mut paste_buffer {
    head.entries
        .range((
            std::ops::Bound::Excluded(paste_time_key(elm)),
            std::ops::Bound::Unbounded,
        ))
        .next()
        .map(|(_, entry)| *entry)
        .unwrap_or(::core::ptr::null_mut::<paste_buffer>())
}

fn paste_time_tree_prev(head: &paste_time_tree, elm: &paste_buffer) -> *mut paste_buffer {
    head.entries
        .range((
            std::ops::Bound::Unbounded,
            std::ops::Bound::Excluded(paste_time_key(elm)),
        ))
        .next_back()
        .map(|(_, entry)| *entry)
        .unwrap_or(::core::ptr::null_mut::<paste_buffer>())
}

fn paste_time_tree_remove(head: &mut paste_time_tree, elm: &paste_buffer) -> *mut paste_buffer {
    head.entries
        .remove(&paste_time_key(elm))
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

fn paste_name_tree_remove_local(elm: &paste_buffer) -> *mut paste_buffer {
    paste_by_name.with(|head| {
        let mut head = head.borrow_mut();
        paste_name_tree_remove(&mut head, elm)
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

fn paste_time_tree_next_local(elm: &paste_buffer) -> *mut paste_buffer {
    paste_by_time.with(|head| {
        let head = head.borrow();
        paste_time_tree_next(&head, elm)
    })
}

fn paste_time_tree_prev_local(elm: &paste_buffer) -> *mut paste_buffer {
    paste_by_time.with(|head| {
        let head = head.borrow();
        paste_time_tree_prev(&head, elm)
    })
}

fn paste_time_tree_remove_local(elm: &paste_buffer) -> *mut paste_buffer {
    paste_by_time.with(|head| {
        let mut head = head.borrow_mut();
        paste_time_tree_remove(&mut head, elm)
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

unsafe fn paste_fire_event(
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
pub unsafe fn paste_buffer_name(mut pb: *mut paste_buffer) -> *const ::core::ffi::c_char {
    return ((*pb).name).as_ptr().cast_mut();
}
pub unsafe fn paste_buffer_order(mut pb: *mut paste_buffer) -> u_int {
    return (*pb).order;
}
pub unsafe fn paste_buffer_created(mut pb: *mut paste_buffer) -> time_t {
    return (*pb).created;
}
pub unsafe fn paste_buffer_data(
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
pub unsafe fn paste_walk(mut pb: *mut paste_buffer) -> *mut paste_buffer {
    if pb.is_null() {
        return paste_time_tree_minmax_local(RB_NEGINF);
    }
    return paste_time_tree_next_local(&*pb);
}
pub unsafe fn paste_is_empty() -> ::core::ffi::c_int {
    return paste_time_tree_is_empty_local() as ::core::ffi::c_int;
}
pub(crate) unsafe fn paste_get_top(name: Option<&mut Option<CString>>) -> *mut paste_buffer {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    pb = paste_time_tree_minmax_local(RB_NEGINF);
    while !pb.is_null() && (*pb).automatic == 0 {
        pb = paste_time_tree_next_local(&*pb);
    }
    if pb.is_null() {
        return ::core::ptr::null_mut::<paste_buffer>();
    }
    if let Some(name) = name {
        *name = Some((*pb).name.clone());
    }
    return pb;
}
pub unsafe fn paste_get_name(mut name: *const ::core::ffi::c_char) -> *mut paste_buffer {
    if name.is_null() || *name as ::core::ffi::c_int == '\0' as i32 {
        return ::core::ptr::null_mut::<paste_buffer>();
    }
    return paste_name_tree_find_local(CStr::from_ptr(name));
}
pub unsafe fn paste_free(mut pb: *mut paste_buffer) {
    paste_fire_event(
        b"paste-buffer-deleted\0" as *const u8 as *const ::core::ffi::c_char,
        ((*pb).name).as_ptr().cast_mut(),
    );
    paste_name_tree_remove_local(&*pb);
    paste_time_tree_remove_local(&*pb);
    if (*pb).automatic != 0 {
        paste_automatic_count_decrement();
    }
    drop(Box::from_raw(pb));
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
        pb1 = paste_time_tree_prev_local(&*pb);
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
            paste_name_cause(
                cause.as_mut(),
                b"invalid buffer name: ",
                CStr::from_ptr(newname),
            );
        }
        return -(1 as ::core::ffi::c_int);
    };
    pb = paste_get_name(oldname);
    if pb.is_null() {
        if !cause.is_null() {
            paste_name_cause(cause.as_mut(), b"no buffer ", CStr::from_ptr(oldname));
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
    paste_name_tree_remove_local(&*pb);
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
pub(crate) unsafe fn paste_set_owned(
    data: Box<[u8]>,
    name: *const ::core::ffi::c_char,
    cause: Option<&mut Option<CString>>,
) -> ::core::ffi::c_int {
    let cause = cause.map_or_else(::core::ptr::null_mut, |cause| &raw mut *cause);
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut old: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    if !cause.is_null() {
        *cause = None;
    }
    if data.is_empty() {
        return 0 as ::core::ffi::c_int;
    }
    if name.is_null() {
        paste_add_owned(None, data);
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
            paste_name_cause(
                cause.as_mut(),
                b"invalid buffer name: ",
                CStr::from_ptr(name),
            );
        }
        return -(1 as ::core::ffi::c_int);
    };
    pb = paste_new_owned(newname);
    paste_store_data(&mut *pb, Some(data));
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
        pb.data.as_ref().map_or(::core::ptr::null_mut(), |value| {
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
    use std::ffi::CString;

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
                paste_rename(::core::ptr::null(), c"renamed".as_ptr(), &raw mut cause,),
                -1
            );
            assert_eq!(
                CStr::from_ptr(cause.as_ref().unwrap().as_ptr()),
                c"no buffer"
            );
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

            assert!(paste_name_tree_insert(&mut tree, z).is_null());
            assert!(paste_name_tree_insert(&mut tree, a_high).is_null());
            assert!(paste_name_tree_insert(&mut tree, a).is_null());
            assert!(paste_name_tree_insert(&mut tree, a0).is_null());
            assert_eq!(
                paste_name_tree_insert(&mut tree, duplicate),
                a,
                "duplicate names keep the original item"
            );

            let ordered = tree.entries.keys().map(Vec::as_slice).collect::<Vec<_>>();
            assert_eq!(
                ordered,
                vec![&b"a"[..], &b"a0"[..], &b"a\xff"[..], &b"z"[..]]
            );
            assert_eq!(paste_name_tree_find(&tree, names[2].as_c_str()), a_high);
            assert!(
                paste_name_tree_find(&tree, CStr::from_bytes_with_nul(b"missing\0").unwrap())
                    .is_null()
            );

            assert_eq!(paste_name_tree_remove(&mut tree, &*a_high), a_high);
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

            assert!(paste_time_tree_insert(&mut tree, oldest).is_null());
            assert!(paste_time_tree_insert(&mut tree, newest).is_null());
            assert!(paste_time_tree_insert(&mut tree, middle).is_null());
            assert_eq!(paste_time_tree_insert(&mut tree, duplicate), middle);

            assert_eq!((*paste_time_tree_minmax(&tree, RB_NEGINF)).order, 12);
            assert_eq!((*paste_time_tree_next(&tree, &*newest)).order, 7);
            assert_eq!((*paste_time_tree_next(&tree, &*middle)).order, 4);
            assert!(paste_time_tree_next(&tree, &*oldest).is_null());

            assert_eq!((*paste_time_tree_minmax(&tree, RB_INF)).order, 4);
            assert_eq!((*paste_time_tree_prev(&tree, &*oldest)).order, 7);
            assert_eq!((*paste_time_tree_prev(&tree, &*middle)).order, 12);
            assert!(paste_time_tree_prev(&tree, &*newest).is_null());

            assert_eq!(paste_time_tree_remove(&mut tree, &*middle), middle);
            assert_eq!((*paste_time_tree_next(&tree, &*newest)).order, 4);
        }
    }
}

use crate::src::cmd::find::{
    cmd_find_clear_state, cmd_find_from_nothing, cmd_find_from_pane, cmd_find_from_session,
    cmd_find_from_session_window, cmd_find_from_winlink, cmd_find_from_winlink_pane,
    cmd_find_valid_state,
};
use crate::src::format::format_add;
use crate::src::log::{fatalx, log_debug};
use crate::src::reactor::{
    evbuffer_add_printf, evbuffer_free, evbuffer_get_length, evbuffer_new, evbuffer_pullup,
};
use crate::src::server_client::server_client_unref;
use crate::src::session::{session_add_ref, session_alive, session_remove_ref};
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::command::cmd_find_state;
use crate::src::shared::event::*;
use crate::src::shared::events::{
    event_payload, event_payload_free_cb, event_payload_item, event_payload_print_cb,
    event_payload_tree, event_payload_tree_storage, event_payload_type,
};
use crate::src::shared::format::format_tree;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::window::{window, winlink};
use crate::src::window::{
    window_add_ref, window_has_pane, window_pane_add_ref, window_pane_remove_ref,
    window_remove_ref, winlink_find_by_index,
};
use crate::src::xmalloc::xvasprintf_cstring;
use std::ffi::{CStr, CString};

pub const EVENT_PAYLOAD_POINTER: event_payload_type = 8;
pub const EVENT_PAYLOAD_PANE: event_payload_type = 7;
pub const EVENT_PAYLOAD_WINDOW: event_payload_type = 6;
pub const EVENT_PAYLOAD_SESSION: event_payload_type = 5;
pub const EVENT_PAYLOAD_CLIENT: event_payload_type = 4;
pub const EVENT_PAYLOAD_UINT: event_payload_type = 3;
pub const EVENT_PAYLOAD_INT: event_payload_type = 2;
pub const EVENT_PAYLOAD_TIME: event_payload_type = 1;
pub const EVENT_PAYLOAD_STRING: event_payload_type = 0;

fn event_payload_name_key(name: &CStr) -> Vec<u8> {
    name.to_bytes().to_vec()
}

unsafe fn event_payload_tree_find(
    head: *mut event_payload_tree,
    name: &CStr,
) -> *mut event_payload_item {
    if head.is_null() {
        return ::core::ptr::null_mut::<event_payload_item>();
    }
    (*head)
        .entries
        .try_borrow_mut()
        .expect("event payload tree already borrowed")
        .entries
        .get(&event_payload_name_key(name))
        .copied()
        .unwrap_or(::core::ptr::null_mut::<event_payload_item>())
}

unsafe fn event_payload_tree_insert(
    head: *mut event_payload_tree,
    elm: *mut event_payload_item,
) -> *mut event_payload_item {
    if head.is_null() {
        return ::core::ptr::null_mut::<event_payload_item>();
    }
    let observer = (*head).entries.downgrade();
    let mut storage = (*head)
        .entries
        .try_borrow_mut()
        .expect("event payload tree already borrowed");
    match storage.entries.entry(event_payload_name_key(
        (*elm)
            .name
            .as_deref()
            .expect("event payload item has a name"),
    )) {
        std::collections::btree_map::Entry::Occupied(entry) => *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            // event_payload_next receives only an item pointer, so retain a
            // weak handle to the storage that owns the ordering map.
            (*elm).owner = Some(observer);
            entry.insert(elm);
            ::core::ptr::null_mut::<event_payload_item>()
        }
    }
}

unsafe fn event_payload_tree_remove(
    head: *mut event_payload_tree,
    elm: *mut event_payload_item,
) -> *mut event_payload_item {
    if head.is_null() || elm.is_null() {
        return ::core::ptr::null_mut::<event_payload_item>();
    }
    let key = event_payload_name_key(
        (*elm)
            .name
            .as_deref()
            .expect("event payload item has a name"),
    );
    let mut storage = (*head)
        .entries
        .try_borrow_mut()
        .expect("event payload tree already borrowed");
    if storage.entries.get(&key).copied() != Some(elm) {
        return ::core::ptr::null_mut();
    }
    let removed = storage.entries.remove(&key).unwrap_or(std::ptr::null_mut());
    (*elm).owner = None;
    removed
}

unsafe fn event_payload_tree_minmax(
    head: *mut event_payload_tree,
    val: ::core::ffi::c_int,
) -> *mut event_payload_item {
    if head.is_null() {
        return ::core::ptr::null_mut::<event_payload_item>();
    }
    let storage = (*head)
        .entries
        .try_borrow_mut()
        .expect("event payload tree already borrowed");
    let item = if val < 0 {
        storage.entries.values().next()
    } else {
        storage.entries.values().next_back()
    };
    item.copied()
        .unwrap_or(::core::ptr::null_mut::<event_payload_item>())
}

unsafe fn event_payload_tree_next(elm: *mut event_payload_item) -> *mut event_payload_item {
    if elm.is_null() {
        return ::core::ptr::null_mut::<event_payload_item>();
    }
    let Some(owner) = (*elm).owner.as_ref() else {
        return ::core::ptr::null_mut::<event_payload_item>();
    };
    let storage = match owner.try_borrow_mut() {
        Ok(storage) => storage,
        Err(refbox::BorrowError::Dropped) => return ::core::ptr::null_mut(),
        Err(refbox::BorrowError::Borrowed) => panic!("event payload tree already borrowed"),
    };
    storage
        .entries
        .range((
            std::ops::Bound::Excluded(event_payload_name_key(
                (*elm)
                    .name
                    .as_deref()
                    .expect("event payload item has a name"),
            )),
            std::ops::Bound::Unbounded,
        ))
        .next()
        .map(|(_, item)| *item)
        .unwrap_or(::core::ptr::null_mut::<event_payload_item>())
}

unsafe extern "C" fn event_payload_find(
    ep: *mut event_payload,
    name: *const ::core::ffi::c_char,
) -> *mut event_payload_item {
    if name.is_null() {
        return ::core::ptr::null_mut::<event_payload_item>();
    }
    event_payload_tree_find(&raw mut (*ep).items, CStr::from_ptr(name))
}
unsafe extern "C" fn event_payload_free_target(mut ep: *mut event_payload) {
    let mut target: *mut cmd_find_state = &raw mut (*ep).target;
    if !(*target).s.is_null() {
        session_remove_ref(
            (*target).s,
            b"event_payload_free_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !(*target).w.is_null() {
        window_remove_ref(
            (*target).w,
            b"event_payload_free_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !(*target).wp.is_null() {
        window_pane_remove_ref(
            (*target).wp,
            b"event_payload_free_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    cmd_find_clear_state(target, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn event_payload_free_value(mut epi: *mut event_payload_item) {
    match (*epi).type_0 as ::core::ffi::c_uint {
        0 => {}
        4 => {
            server_client_unref((*epi).c2rust_unnamed.client);
        }
        5 => {
            session_remove_ref(
                (*epi).c2rust_unnamed.session,
                b"event_payload_free_value\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        6 => {
            window_remove_ref(
                (*epi).c2rust_unnamed.window,
                b"event_payload_free_value\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        7 => {
            window_pane_remove_ref(
                (*epi).c2rust_unnamed.pane,
                b"event_payload_free_value\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        8 => {
            if (*epi).c2rust_unnamed.pointer.free_cb.is_some() {
                (*epi)
                    .c2rust_unnamed
                    .pointer
                    .free_cb
                    .expect("non-null function pointer")(
                    (*epi).c2rust_unnamed.pointer.ptr
                );
            }
        }
        2 | 3 | 1 | _ => {}
    };
}
unsafe fn event_payload_free_item(epi: *mut event_payload_item) {
    event_payload_free_value(epi);
    drop(Box::from_raw(epi));
}

unsafe fn event_payload_new_item() -> *mut event_payload_item {
    // Every field is C-style pointer or integer storage, including the union.
    Box::into_raw(Box::new(event_payload_item {
        name: None,
        string: None,
        ..event_payload_item::empty()
    }))
    .cast()
}
unsafe extern "C" fn event_payload_set_item(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut new: *mut event_payload_item,
) {
    let mut old: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    // `name` may borrow the item being replaced, so copy before its callback.
    let owner = &mut *new;
    owner.name = Some(CStr::from_ptr(name).to_owned());

    old = event_payload_tree_insert(&raw mut (*ep).items, new);
    if !old.is_null() {
        event_payload_tree_remove(&raw mut (*ep).items, old);
        event_payload_free_item(old);
        event_payload_tree_insert(&raw mut (*ep).items, new);
    }
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_create() -> *mut event_payload {
    let ep = Box::into_raw(Box::new(event_payload {
        items: event_payload_tree::default(),
        target: ::core::mem::zeroed::<cmd_find_state>(),
    }));
    cmd_find_clear_state(&raw mut (*ep).target, 0 as ::core::ffi::c_int);
    return ep;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_free(mut ep: *mut event_payload) {
    if !ep.is_null() {
        let items: Vec<*mut event_payload_item> = (*ep)
            .items
            .entries
            .try_borrow_mut()
            .expect("event payload tree already borrowed")
            .entries
            .values()
            .copied()
            .collect();
        for epi in items {
            event_payload_tree_remove(&raw mut (*ep).items, epi);
            event_payload_free_item(epi);
        }
        event_payload_free_target(ep);
        drop(Box::from_raw(ep));
    }
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_target(
    mut ep: *mut event_payload,
    mut fs: *mut cmd_find_state,
) {
    let mut target: *mut cmd_find_state = &raw mut (*ep).target;
    event_payload_free_target(ep);
    if !(*fs).s.is_null() {
        session_add_ref(
            (*fs).s,
            b"event_payload_set_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*target).s = (*fs).s;
    }
    if !(*fs).wl.is_null() {
        (*target).idx = (*(*fs).wl).idx;
        if (*target).s.is_null() {
            session_add_ref(
                (*(*fs).wl).session,
                b"event_payload_set_target\0" as *const u8 as *const ::core::ffi::c_char,
            );
            (*target).s = (*(*fs).wl).session;
        }
    } else {
        (*target).idx = -(1 as ::core::ffi::c_int);
    }
    if !(*fs).w.is_null() {
        window_add_ref(
            (*fs).w,
            b"event_payload_set_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*target).w = (*fs).w;
    } else if !(*fs).wl.is_null() {
        window_add_ref(
            (*(*fs).wl).window,
            b"event_payload_set_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*target).w = (*(*fs).wl).window;
    }
    if !(*fs).wp.is_null() {
        window_pane_add_ref(
            (*fs).wp,
            b"event_payload_set_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*target).wp = (*fs).wp;
    }
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_target(
    mut ep: *mut event_payload,
    mut fs: *mut cmd_find_state,
) -> ::core::ffi::c_int {
    let mut t: *mut cmd_find_state = &raw mut (*ep).target;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut flags: ::core::ffi::c_int = (*fs).flags;
    if (*t).idx != -(1 as ::core::ffi::c_int)
        && !(*t).s.is_null()
        && !(*t).w.is_null()
        && session_alive((*t).s) != 0
    {
        wl = winlink_find_by_index(&raw mut (*(*t).s).windows, (*t).idx);
        if !wl.is_null() && (*wl).window != (*t).w {
            wl = ::core::ptr::null_mut::<winlink>();
        }
    }
    cmd_find_clear_state(fs, flags);
    (*fs).s = (*t).s;
    (*fs).w = (*t).w;
    (*fs).wp = (*t).wp;
    (*fs).wl = wl;
    (*fs).idx = if !wl.is_null() {
        (*wl).idx
    } else {
        -(1 as ::core::ffi::c_int)
    };
    if cmd_find_valid_state(fs) != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if !wl.is_null() && !(*t).wp.is_null() && window_has_pane((*wl).window, (*t).wp) != 0 {
        cmd_find_from_winlink_pane(fs, wl, (*t).wp, flags);
        if cmd_find_valid_state(fs) != 0 {
            return 1 as ::core::ffi::c_int;
        }
    }
    if !(*t).wp.is_null()
        && cmd_find_from_pane(fs, (*t).wp, flags) == 0 as ::core::ffi::c_int
        && cmd_find_valid_state(fs) != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if !wl.is_null() {
        cmd_find_from_winlink(fs, wl, flags);
        if cmd_find_valid_state(fs) != 0 {
            return 1 as ::core::ffi::c_int;
        }
    }
    if !(*t).s.is_null()
        && !(*t).w.is_null()
        && session_alive((*t).s) != 0
        && cmd_find_from_session_window(fs, (*t).s, (*t).w, flags) == 0 as ::core::ffi::c_int
        && cmd_find_valid_state(fs) != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if !(*t).s.is_null() && session_alive((*t).s) != 0 {
        cmd_find_from_session(fs, (*t).s, flags);
        if cmd_find_valid_state(fs) != 0 {
            return 1 as ::core::ffi::c_int;
        }
    }
    if cmd_find_from_nothing(fs, flags) == 0 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    cmd_find_clear_state(fs, flags);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_string(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    let string = xvasprintf_cstring(fmt, ap);
    let epi = event_payload_new_item();
    (*epi).type_0 = EVENT_PAYLOAD_STRING;
    let owner = &mut *epi;
    owner.string = Some(string);
    owner.c2rust_unnamed.string = owner.string.as_ref().unwrap().as_ptr().cast_mut();
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_time(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: time_t,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_new_item();
    (*epi).type_0 = EVENT_PAYLOAD_TIME;
    (*epi).c2rust_unnamed.time = value;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_int(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: ::core::ffi::c_int,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_new_item();
    (*epi).type_0 = EVENT_PAYLOAD_INT;
    (*epi).c2rust_unnamed.number = value;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_uint(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: u_int,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_new_item();
    (*epi).type_0 = EVENT_PAYLOAD_UINT;
    (*epi).c2rust_unnamed.unsigned_number = value;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_client(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut c: *mut client,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    (*c).references += 1;
    epi = event_payload_new_item();
    (*epi).type_0 = EVENT_PAYLOAD_CLIENT;
    (*epi).c2rust_unnamed.client = c;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_session(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut s: *mut session,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    session_add_ref(
        s,
        b"event_payload_set_session\0" as *const u8 as *const ::core::ffi::c_char,
    );
    epi = event_payload_new_item();
    (*epi).type_0 = EVENT_PAYLOAD_SESSION;
    (*epi).c2rust_unnamed.session = s;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_window(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut w: *mut window,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    window_add_ref(
        w,
        b"event_payload_set_window\0" as *const u8 as *const ::core::ffi::c_char,
    );
    epi = event_payload_new_item();
    (*epi).type_0 = EVENT_PAYLOAD_WINDOW;
    (*epi).c2rust_unnamed.window = w;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_pane(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut wp: *mut window_pane,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    window_pane_add_ref(
        wp,
        b"event_payload_set_pane\0" as *const u8 as *const ::core::ffi::c_char,
    );
    epi = event_payload_new_item();
    (*epi).type_0 = EVENT_PAYLOAD_PANE;
    (*epi).c2rust_unnamed.pane = wp;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_pointer(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut ptr: *mut ::core::ffi::c_void,
    mut free_cb: event_payload_free_cb,
    mut print_cb: event_payload_print_cb,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_new_item();
    (*epi).type_0 = EVENT_PAYLOAD_POINTER;
    (*epi).c2rust_unnamed.pointer.ptr = ptr;
    (*epi).c2rust_unnamed.pointer.free_cb = free_cb;
    (*epi).c2rust_unnamed.pointer.print_cb = print_cb;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_string(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return (*epi).c2rust_unnamed.string;
}
unsafe extern "C" fn event_payload_add_item(
    mut epi: *mut event_payload_item,
    mut evb: *mut evbuffer,
) {
    match (*epi).type_0 as ::core::ffi::c_uint {
        0 => {
            evbuffer_add_printf(
                evb,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*epi).c2rust_unnamed.string,
            );
        }
        1 => {
            evbuffer_add_printf(
                evb,
                b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                (*epi).c2rust_unnamed.time as ::core::ffi::c_longlong,
            );
        }
        2 => {
            evbuffer_add_printf(
                evb,
                b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                (*epi).c2rust_unnamed.number,
            );
        }
        3 => {
            evbuffer_add_printf(
                evb,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*epi).c2rust_unnamed.unsigned_number,
            );
        }
        4 => {
            evbuffer_add_printf(
                evb,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                ((*(*epi).c2rust_unnamed.client).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            );
        }
        5 => {
            evbuffer_add_printf(
                evb,
                b"$%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*(*epi).c2rust_unnamed.session).id,
            );
        }
        6 => {
            evbuffer_add_printf(
                evb,
                b"@%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*(*epi).c2rust_unnamed.window).id,
            );
        }
        7 => {
            evbuffer_add_printf(
                evb,
                b"%%%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*(*epi).c2rust_unnamed.pane).id,
            );
        }
        8 => {
            if (*epi).c2rust_unnamed.pointer.print_cb.is_some() {
                (*epi)
                    .c2rust_unnamed
                    .pointer
                    .print_cb
                    .expect("non-null function pointer")(
                    (*epi).c2rust_unnamed.pointer.ptr, evb
                );
            } else {
                evbuffer_add_printf(
                    evb,
                    b"%p\0" as *const u8 as *const ::core::ffi::c_char,
                    (*epi).c2rust_unnamed.pointer.ptr,
                );
            }
        }
        _ => {}
    };
}
/// Printed payload bytes with one trailing NUL for synchronous C consumers.
/// The bytes before that terminator may themselves contain NULs.
pub(crate) unsafe fn event_payload_item_print_owned(epi: *mut event_payload_item) -> Vec<u8> {
    let mut evb: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: size_t = 0;
    evb = evbuffer_new();
    if evb.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    event_payload_add_item(epi, evb);
    size = evbuffer_get_length(&*(evb));
    let mut value = Vec::with_capacity(size + 1);
    if size != 0 as size_t {
        let bytes = evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t) as *const u8;
        value.extend_from_slice(std::slice::from_raw_parts(bytes, size));
    }
    value.push(0);
    evbuffer_free(evb);
    value
}

pub(crate) unsafe fn event_payload_print_owned(
    ep: *mut event_payload,
    name: *const ::core::ffi::c_char,
) -> Option<Vec<u8>> {
    let epi = event_payload_find(ep, name);
    (!epi.is_null()).then(|| event_payload_item_print_owned(epi))
}

#[no_mangle]
pub unsafe extern "C" fn event_payload_add_formats(
    mut ep: *mut event_payload,
    mut ft: *mut format_tree,
    mut prefix: *const ::core::ffi::c_char,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    if prefix.is_null() {
        prefix = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    let prefix = CStr::from_ptr(prefix).to_bytes();
    epi = event_payload_tree_minmax(&raw mut (*ep).items, -1);
    while !epi.is_null() {
        let key = (*epi).name.as_ref().unwrap().as_ptr();
        if !(*key as ::core::ffi::c_int == '_' as i32) {
            let value = event_payload_item_print_owned(epi);
            let key_bytes = CStr::from_ptr(key).to_bytes();
            let mut name_bytes = Vec::with_capacity(prefix.len() + key_bytes.len());
            name_bytes.extend_from_slice(prefix);
            name_bytes.extend_from_slice(key_bytes);
            let name = CString::new(name_bytes).expect("C string parts contain no NUL");
            // format_add copies the key into its format entry before returning.
            format_add(
                ft,
                name.as_ptr(),
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                value.as_ptr().cast::<::core::ffi::c_char>(),
            );
            let named = if (*epi).type_0 as ::core::ffi::c_uint
                == EVENT_PAYLOAD_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                Some((*(*epi).c2rust_unnamed.session).name.as_ptr().cast_mut())
            } else if (*epi).type_0 as ::core::ffi::c_uint
                == EVENT_PAYLOAD_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                Some((*(*epi).c2rust_unnamed.window).name)
            } else {
                None
            };
            if let Some(named) = named {
                let mut suffixed = name.as_bytes().to_vec();
                suffixed.extend_from_slice(b"_name");
                let suffixed = CString::new(suffixed).expect("C string parts contain no NUL");
                format_add(
                    ft,
                    suffixed.as_ptr(),
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    named,
                );
            }
        }
        epi = event_payload_tree_next(epi);
    }
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_first(
    mut ep: *mut event_payload,
) -> *mut event_payload_item {
    return event_payload_tree_minmax(&raw mut (*ep).items, -1);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_next(
    mut epi: *mut event_payload_item,
) -> *mut event_payload_item {
    return event_payload_tree_next(epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_item_name(
    mut epi: *mut event_payload_item,
) -> *const ::core::ffi::c_char {
    return ((*epi).name)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_item_type(
    mut epi: *mut event_payload_item,
) -> event_payload_type {
    return (*epi).type_0;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_log(
    mut ep: *mut event_payload,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut evb: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    let prefix = xvasprintf_cstring(fmt, ap);
    evb = evbuffer_new();
    if evb.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if !ep.is_null() {
        epi = event_payload_tree_minmax(&raw mut (*ep).items, -1);
        while !epi.is_null() {
            if evbuffer_get_length(&*(evb)) != 0 as size_t {
                evbuffer_add_printf(evb, b", \0" as *const u8 as *const ::core::ffi::c_char);
            }
            evbuffer_add_printf(
                evb,
                b"%s=\0" as *const u8 as *const ::core::ffi::c_char,
                ((*epi).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            );
            event_payload_add_item(epi, evb);
            epi = event_payload_tree_next(epi);
        }
    }
    log_debug(
        b"%s%.*s\0" as *const u8 as *const ::core::ffi::c_char,
        prefix.as_ptr(),
        evbuffer_get_length(&*(evb)) as ::core::ffi::c_int,
        evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t) as *mut ::core::ffi::c_char,
    );
    evbuffer_free(evb);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_time(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> time_t {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_TIME as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as time_t;
    }
    return (*epi).c2rust_unnamed.time;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_int(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_INT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    *value = (*epi).c2rust_unnamed.number;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_uint(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: *mut u_int,
) -> ::core::ffi::c_int {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_UINT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    *value = (*epi).c2rust_unnamed.unsigned_number;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_client(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut client {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_CLIENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<client>();
    }
    return (*epi).c2rust_unnamed.client;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_session(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut session {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<session>();
    }
    return (*epi).c2rust_unnamed.session;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_window(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut window {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<window>();
    }
    return (*epi).c2rust_unnamed.window;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_pane(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut window_pane {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<window_pane>();
    }
    return (*epi).c2rust_unnamed.pane;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_pointer(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_void {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_POINTER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return (*epi).c2rust_unnamed.pointer.ptr;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::src::reactor::evbuffer_add;
    use crate::src::shared::events::event_payload_item_c2rust_unnamed;
    use std::ffi::{CStr, CString};

    unsafe extern "C" fn count_pointer_release(ptr: *mut ::core::ffi::c_void) {
        *(ptr as *mut usize) += 1;
    }

    unsafe extern "C" fn print_binary_pointer(_ptr: *mut ::core::ffi::c_void, evb: *mut evbuffer) {
        let bytes = b"A\0B";
        evbuffer_add(evb, bytes.as_ptr().cast(), bytes.len());
    }

    #[test]
    fn printed_pointer_bytes_keep_an_interior_nul() {
        unsafe {
            let ep = event_payload_create();
            event_payload_set_pointer(
                ep,
                c"binary".as_ptr(),
                ::core::ptr::null_mut(),
                None,
                Some(print_binary_pointer),
            );
            let item = event_payload_first(ep);
            assert_eq!(event_payload_item_print_owned(item), b"A\0B\0");
            assert_eq!(
                event_payload_print_owned(ep, c"binary".as_ptr()),
                Some(b"A\0B\0".to_vec())
            );
            assert!(event_payload_print_owned(ep, c"missing".as_ptr()).is_none());
            event_payload_free(ep);
        }
    }

    #[test]
    fn string_items_survive_lookup_and_release_on_both_replacement_and_payload_free() {
        unsafe {
            let ep = event_payload_create();
            let name = c"value";
            let first = CString::new(vec![b'a', 0xff]).unwrap();
            event_payload_set_string(ep, name.as_ptr(), c"%s".as_ptr(), first.as_ptr());
            let first_item = event_payload_first(ep);
            assert_eq!((*first_item).type_0, EVENT_PAYLOAD_STRING);
            assert_eq!(
                CStr::from_ptr(event_payload_get_string(ep, name.as_ptr())).to_bytes(),
                first.to_bytes()
            );
            assert_eq!(
                (*first_item).c2rust_unnamed.string,
                event_payload_get_string(ep, name.as_ptr()).cast_mut()
            );

            event_payload_set_string(ep, name.as_ptr(), c"%s".as_ptr(), c"second".as_ptr());
            let second_item = event_payload_first(ep);
            assert_ne!(second_item, first_item);
            assert_eq!(
                CStr::from_ptr(event_payload_get_string(ep, name.as_ptr())),
                c"second"
            );

            event_payload_set_int(ep, name.as_ptr(), 42);
            assert!(event_payload_get_string(ep, name.as_ptr()).is_null());
            assert_eq!((*event_payload_first(ep)).type_0, EVENT_PAYLOAD_INT);

            let mut releases = 0usize;
            event_payload_set_pointer(
                ep,
                name.as_ptr(),
                (&raw mut releases).cast(),
                Some(count_pointer_release),
                None,
            );
            event_payload_set_string(ep, name.as_ptr(), c"%s".as_ptr(), c"".as_ptr());
            assert_eq!(releases, 1);
            assert_eq!(
                CStr::from_ptr(event_payload_get_string(ep, name.as_ptr())),
                c""
            );
            event_payload_free(ep);
        }
    }

    #[test]
    fn replacement_accepts_the_previous_items_borrowed_name() {
        unsafe {
            let ep = event_payload_create();
            let name = CString::new(vec![b'k', 0xff]).unwrap();
            event_payload_set_string(ep, name.as_ptr(), c"%s".as_ptr(), c"old".as_ptr());
            let old_name = event_payload_item_name(event_payload_first(ep));
            event_payload_set_int(ep, old_name, 42);

            let replacement = event_payload_first(ep);
            assert_eq!(
                CStr::from_ptr(event_payload_item_name(replacement)).to_bytes(),
                name.as_bytes()
            );
            assert_eq!((*replacement).c2rust_unnamed.number, 42);
            assert_eq!(event_payload_next(replacement), ::core::ptr::null_mut());
            event_payload_free(ep);
        }
    }

    #[test]
    fn item_observers_follow_tree_moves_and_clear_on_removal() {
        unsafe {
            fn item(name: &str) -> *mut event_payload_item {
                let mut item = Box::new(event_payload_item::empty());
                item.name = Some(CString::new(name).unwrap());
                Box::into_raw(item)
            }

            let mut payload = event_payload {
                items: event_payload_tree::default(),
                target: ::core::mem::zeroed(),
            };
            let mut other = event_payload_tree::default();
            let first = item("alpha");
            let second = item("beta");
            let duplicate = item("alpha");
            assert!(event_payload_tree_insert(&mut payload.items, first).is_null());
            assert!(event_payload_tree_insert(&mut payload.items, second).is_null());
            let storage_observer = (*first).owner.as_ref().unwrap().clone();
            assert_eq!(
                event_payload_tree_insert(&mut payload.items, duplicate),
                first
            );
            assert!((*duplicate).owner.is_none());
            assert!(event_payload_tree_remove(&mut other, first).is_null());
            assert!((*first).owner.is_some());

            let mut moved = payload;
            assert_eq!(event_payload_tree_minmax(&raw mut moved.items, -1), first);
            assert_eq!(event_payload_tree_next(first), second);
            assert_eq!(
                event_payload_tree_remove(&raw mut moved.items, first),
                first
            );
            assert!((*first).owner.is_none());
            assert!(event_payload_tree_next(first).is_null());
            assert_eq!(
                event_payload_tree_remove(&raw mut moved.items, second),
                second
            );

            event_payload_free_item(first);
            event_payload_free_item(second);
            event_payload_free_item(duplicate);
            drop(moved);
            assert!(matches!(
                storage_observer.try_borrow_mut(),
                Err(refbox::BorrowError::Dropped)
            ));
        }
    }
}

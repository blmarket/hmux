use crate::src::cmd::find::{cmd_find_from_session, cmd_find_from_winlink};
use crate::src::compat::strtonum::strtonum;
use crate::src::environ::environ_free;
use crate::src::events::{events_fire, events_fire_session, events_fire_winlink};
use crate::src::events_payload::{
    event_payload_create, event_payload_set_int, event_payload_set_session,
    event_payload_set_string, event_payload_set_target, event_payload_set_uint,
    event_payload_set_window,
};
use crate::src::ffi::libc::{free, gettimeofday, memcpy, strcmp};
use crate::src::grid::grid_collect_history;
use crate::src::log::{fatal, fatalx, log_debug};
use crate::src::options::{options_free, options_get_number};
use crate::src::reactor::{event_add, event_del, event_initialized, event_once, event_set};
use crate::src::resize::recalculate_sizes;
use crate::src::server::{marked_pane, server_clear_marked};
use crate::src::server_fn::server_lock_session;
use crate::src::shared::abi::*;
use crate::src::shared::command::cmd_find_state;
use crate::src::shared::environment::environ;
use crate::src::shared::event::EV_TIMEOUT;
use crate::src::shared::event::*;
use crate::src::shared::events::event_payload;
use crate::src::shared::grid::*;
use crate::src::shared::limits::UINT_MAX;
use crate::src::shared::options::options;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_THEMECHANGED;
use crate::src::shared::session::{session, session_entry};
use crate::src::shared::session::{session_group, session_group_entry, session_groups, sessions};
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::terminal::*;
use crate::src::shared::tree::{RB_INF, RB_NEGINF};
use crate::src::shared::window::{window, winlink, winlink_stack, winlinks};
use crate::src::shared::window::{WINLINK_ALERTFLAGS, WINLINK_VISITED};
use crate::src::sort::sort_get_sessions;
use crate::src::status::status_update_cache;
use crate::src::tmux::global_options;
use crate::src::tty::tty_update_window_offset;
use crate::src::window::{
    window_pane_first, window_pane_next, window_update_activity, window_update_focus,
    window_winlinks_first, window_winlinks_next, winlink_add, winlink_clear_flags,
    winlink_find_by_index, winlink_find_by_window, winlink_find_by_window_id, winlink_next,
    winlink_previous, winlink_remove, winlink_set_window, winlink_stack_push, winlink_stack_remove,
    winlinks_minmax, winlinks_next,
};
use std::ffi::{CStr, CString};

#[no_mangle]
pub static mut sessions: sessions = sessions { storage: None };
#[no_mangle]
pub static mut next_session_id: u_int = 0;
#[no_mangle]
pub static mut session_groups: session_groups = session_groups { storage: None };

/// `session.cwd` borrows this value until replacement or early destruction.
pub(crate) fn session_set_cwd(s: &mut session, cwd: Option<CString>) {
    s.cwd = cwd;
}

/// C producers return libc-owned strings, not CString-owned allocations.
pub(crate) unsafe fn session_set_cwd_from_c_owned(
    s: *mut session,
    raw_cwd: *mut ::core::ffi::c_char,
) {
    let cwd = CStr::from_ptr(raw_cwd).to_owned();
    free(raw_cwd.cast());
    session_set_cwd(&mut *s, Some(cwd));
}

/// Replace the borrowed public name after callers remove the old map key.
pub(crate) fn session_replace_name(s: &mut session, name: CString) -> CString {
    std::mem::replace(&mut s.name, name)
}
#[no_mangle]
pub unsafe extern "C" fn session_cmp(
    mut s1: *mut session,
    mut s2: *mut session,
) -> ::core::ffi::c_int {
    return strcmp(
        ((*s1).name).as_ptr().cast_mut(),
        ((*s2).name).as_ptr().cast_mut(),
    );
}
pub(crate) fn sessions_key(elm: &session) -> Vec<u8> {
    elm.name.as_bytes().to_vec()
}
pub fn sessions_find(head: &sessions, elm: &session) -> *mut session {
    let Some(owner) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = owner
        .try_borrow_mut()
        .expect("session index already borrowed");
    let key = elm.name.as_c_str().to_bytes();
    map.get(key).copied().unwrap_or(std::ptr::null_mut())
}
pub fn sessions_nfind(head: &sessions, elm: &session) -> *mut session {
    let Some(owner) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = owner
        .try_borrow_mut()
        .expect("session index already borrowed");
    let key = elm.name.as_c_str().to_bytes();
    map.range::<[u8], _>((std::ops::Bound::Included(key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
pub unsafe fn sessions_insert(head: *mut sessions, elm: *mut session) -> *mut session {
    let key = std::ffi::CStr::from_ptr(((*elm).name).as_ptr().cast_mut()).to_bytes();
    let owner = (*head).storage.get_or_insert_with(refbox::RefBox::default);
    let observer = owner.downgrade();
    let mut map = owner
        .try_borrow_mut()
        .expect("session index already borrowed");
    match map.entry(key.to_vec()) {
        std::collections::btree_map::Entry::Occupied(entry) => return *entry.get(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
            (*elm).entry.owner = Some(observer);
        }
    }
    std::ptr::null_mut()
}
pub unsafe fn sessions_remove(head: *mut sessions, elm: *mut session) -> *mut session {
    if elm.is_null() {
        return std::ptr::null_mut();
    }
    let key = std::ffi::CStr::from_ptr(((*elm).name).as_ptr().cast_mut()).to_bytes();
    let Some(owner) = (*head).storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let empty = {
        let mut map = owner
            .try_borrow_mut()
            .expect("session index already borrowed");
        if map.get(key).copied() != Some(elm) {
            return std::ptr::null_mut();
        }
        map.remove(key);
        map.is_empty()
    };
    (*elm).entry.owner = None;
    if empty {
        (*head).storage = None;
    }
    elm
}
pub fn sessions_minmax(head: &sessions, direction: ::core::ffi::c_int) -> *mut session {
    let Some(owner) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = owner
        .try_borrow_mut()
        .expect("session index already borrowed");
    let pair = if direction < 0 {
        map.first_key_value()
    } else {
        map.last_key_value()
    };
    pair.map_or(std::ptr::null_mut(), |(_, node)| *node)
}
/// Resume a potentially destructive walk using a saved name and the live index.
/// The named session and any of its successors may already have been removed.
pub fn sessions_after(head: &sessions, name: &[u8]) -> *mut session {
    let Some(owner) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = owner
        .try_borrow_mut()
        .expect("session index already borrowed");
    map.range::<[u8], _>((std::ops::Bound::Excluded(name), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, &node)| node)
}

/// The session must still belong to its index. Destructive walks use sessions_after.
pub unsafe fn sessions_next(elm: &session) -> *mut session {
    let Some(owner) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = match owner.try_borrow_mut() {
        Ok(map) => map,
        Err(refbox::BorrowError::Dropped) => return std::ptr::null_mut(),
        Err(refbox::BorrowError::Borrowed) => panic!("session index already borrowed"),
    };
    let key = std::ffi::CStr::from_ptr(elm.name.as_ptr().cast_mut()).to_bytes();
    map.range::<[u8], _>((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}
/// The session must still belong to its index.
pub unsafe fn sessions_prev(elm: &session) -> *mut session {
    let Some(owner) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = match owner.try_borrow_mut() {
        Ok(map) => map,
        Err(refbox::BorrowError::Dropped) => return std::ptr::null_mut(),
        Err(refbox::BorrowError::Borrowed) => panic!("session index already borrowed"),
    };
    let key = std::ffi::CStr::from_ptr(elm.name.as_ptr().cast_mut()).to_bytes();
    map.range::<[u8], _>((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(key)))
        .next_back()
        .map_or(std::ptr::null_mut(), |(_, node)| *node)
}

#[no_mangle]
pub unsafe extern "C" fn session_group_cmp(
    mut s1: *mut session_group,
    mut s2: *mut session_group,
) -> ::core::ffi::c_int {
    return strcmp(
        ((*s1).name).as_ptr().cast_mut(),
        ((*s2).name).as_ptr().cast_mut(),
    );
}
pub fn session_groups_find(head: &session_groups, elm: &session_group) -> *mut session_group {
    let Some(owner) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = owner
        .try_borrow_mut()
        .expect("session group index already borrowed");
    let key = elm.name.as_c_str().to_bytes();
    map.get(key)
        .map_or(std::ptr::null_mut(), |owner| owner.node_ptr())
}
pub fn session_groups_nfind(head: &session_groups, elm: &session_group) -> *mut session_group {
    let Some(owner) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = owner
        .try_borrow_mut()
        .expect("session group index already borrowed");
    let key = elm.name.as_c_str().to_bytes();
    map.range::<[u8], _>((std::ops::Bound::Included(key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, owner)| owner.node_ptr())
}
impl session_group {
    pub fn new(name: &std::ffi::CStr) -> Box<Self> {
        let mut owner = Box::new(session_group {
            name: name.to_owned(),
            entry: session_group_entry { owner: None },
            members: Vec::new(),
        });

        owner
    }

    pub fn node_ptr(&self) -> *mut session_group {
        std::ptr::addr_of!(*self).cast_mut()
    }
}

/// Consumes the new owner. On a duplicate name, the old node is returned and
/// the incoming owner is dropped without entering the index.
pub unsafe fn session_groups_insert(
    head: *mut session_groups,
    owner: Box<session_group>,
) -> *mut session_group {
    let key = owner.name.to_bytes();
    let storage = (*head).storage.get_or_insert_with(refbox::RefBox::default);
    let observer = storage.downgrade();
    let mut map = storage
        .try_borrow_mut()
        .expect("session group index already borrowed");
    match map.entry(key.to_vec()) {
        std::collections::btree_map::Entry::Occupied(entry) => return entry.get().node_ptr(),
        std::collections::btree_map::Entry::Vacant(entry) => {
            let elm = owner.node_ptr();
            entry.insert(owner);
            (*elm).entry.owner = Some(observer);
        }
    }
    std::ptr::null_mut()
}
/// Removes and drops the indexed owner. A same-name node outside the index is
/// not adopted or removed.
pub unsafe fn session_groups_remove(head: *mut session_groups, elm: *mut session_group) -> bool {
    if elm.is_null() {
        return false;
    }
    let key = std::ffi::CStr::from_ptr(((*elm).name).as_ptr().cast_mut()).to_bytes();
    let Some(storage) = (*head).storage.as_ref() else {
        return false;
    };
    // End the map borrow before dropping the group and its name.
    let (removed, empty) = {
        let mut map = storage
            .try_borrow_mut()
            .expect("session group index already borrowed");
        if map.get(key).map(|owner| owner.node_ptr()) != Some(elm) {
            return false;
        }
        (*elm).entry.owner = None;
        (
            map.remove(key).expect("indexed session group disappeared"),
            map.is_empty(),
        )
    };
    if empty {
        (*head).storage = None;
    }
    drop(removed);
    true
}
pub fn session_groups_minmax(
    head: &session_groups,
    direction: ::core::ffi::c_int,
) -> *mut session_group {
    let Some(storage) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = storage
        .try_borrow_mut()
        .expect("session group index already borrowed");
    let pair = if direction < 0 {
        map.first_key_value()
    } else {
        map.last_key_value()
    };
    pair.map_or(std::ptr::null_mut(), |(_, owner)| owner.node_ptr())
}
pub unsafe fn session_groups_next(elm: &session_group) -> *mut session_group {
    let Some(owner) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = match owner.try_borrow_mut() {
        Ok(map) => map,
        Err(refbox::BorrowError::Dropped) => return std::ptr::null_mut(),
        Err(refbox::BorrowError::Borrowed) => panic!("session group index already borrowed"),
    };
    let key = std::ffi::CStr::from_ptr(elm.name.as_ptr().cast_mut()).to_bytes();
    map.range::<[u8], _>((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, owner)| owner.node_ptr())
}
pub unsafe fn session_groups_prev(elm: &session_group) -> *mut session_group {
    let Some(owner) = elm.entry.owner.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = match owner.try_borrow_mut() {
        Ok(map) => map,
        Err(refbox::BorrowError::Dropped) => return std::ptr::null_mut(),
        Err(refbox::BorrowError::Borrowed) => panic!("session group index already borrowed"),
    };
    let key = std::ffi::CStr::from_ptr(elm.name.as_ptr().cast_mut()).to_bytes();
    map.range::<[u8], _>((std::ops::Bound::Unbounded, std::ops::Bound::Excluded(key)))
        .next_back()
        .map_or(std::ptr::null_mut(), |(_, owner)| owner.node_ptr())
}

#[no_mangle]
pub unsafe extern "C" fn session_alive(mut s: *mut session) -> ::core::ffi::c_int {
    let mut s_loop: *mut session = ::core::ptr::null_mut::<session>();
    s_loop = sessions_minmax(&*std::ptr::addr_of!(sessions), RB_NEGINF);
    while !s_loop.is_null() {
        if s_loop == s {
            return 1 as ::core::ffi::c_int;
        }
        s_loop = sessions_next(&*s_loop);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn session_find(mut name: *const ::core::ffi::c_char) -> *mut session {
    let mut s: session = session {
        id: 0,
        name: Default::default(),
        cwd: Default::default(),
        creation_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        last_attached_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        activity_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        last_activity_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        lock_timer: event::default(),
        curw: ::core::ptr::null_mut::<winlink>(),
        lastw: winlink_stack {
            storage: None,
            reserved: std::ptr::null_mut(),
        },
        windows: winlinks { storage: None },
        statusat: 0,
        statuslines: 0,
        options: ::core::ptr::null_mut::<options>(),
        flags: 0,
        attached: 0,
        tio: None,
        environ: ::core::ptr::null_mut::<environ>(),
        references: 0,
        entry: session_entry { owner: None },
    };
    s.name = ::std::ffi::CStr::from_ptr(name as *mut ::core::ffi::c_char).to_owned();
    return sessions_find(&*std::ptr::addr_of!(sessions), &s);
}
#[no_mangle]
pub unsafe extern "C" fn session_find_by_id_str(mut s: *const ::core::ffi::c_char) -> *mut session {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: u_int = 0;
    if *s as ::core::ffi::c_int != '$' as i32 {
        return ::core::ptr::null_mut::<session>();
    }
    id = strtonum(
        s.offset(1 as ::core::ffi::c_int as isize),
        0 as ::core::ffi::c_longlong,
        UINT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as u_int;
    if !errstr.is_null() {
        return ::core::ptr::null_mut::<session>();
    }
    return session_find_by_id(id);
}
#[no_mangle]
pub unsafe extern "C" fn session_find_by_id(mut id: u_int) -> *mut session {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    s = sessions_minmax(&*std::ptr::addr_of!(sessions), RB_NEGINF);
    while !s.is_null() {
        if (*s).id == id {
            return s;
        }
        s = sessions_next(&*s);
    }
    return ::core::ptr::null_mut::<session>();
}
#[no_mangle]
pub unsafe extern "C" fn session_create(
    mut prefix: *const ::core::ffi::c_char,
    mut name: *const ::core::ffi::c_char,
    mut cwd: *const ::core::ffi::c_char,
    mut env: *mut environ,
    mut oo: *mut options,
    mut tio: *mut termios,
) -> *mut session {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut owner = Box::new(session {
        tio: if tio.is_null() {
            None
        } else {
            Some(Box::new(*tio))
        },
        cwd: Some(CStr::from_ptr(cwd).to_owned()),
        name: CString::new("").expect("empty session name has no NUL"),
        ..session::empty()
    });

    s = Box::into_raw(owner).cast::<session>();
    (*s).references = 1 as ::core::ffi::c_int;
    (*s).flags = 0 as ::core::ffi::c_int;
    (*s).lastw.storage = None;
    (*s).lastw.reserved = std::ptr::null_mut();
    (*s).windows.storage = None;
    (*s).environ = env;
    (*s).options = oo;
    status_update_cache(s);
    if !name.is_null() {
        drop(session_replace_name(
            &mut *s,
            CStr::from_ptr(name).to_owned(),
        ));
        let fresh0 = next_session_id;
        next_session_id = next_session_id.wrapping_add(1);
        (*s).id = fresh0;
    } else {
        loop {
            let fresh1 = next_session_id;
            next_session_id = next_session_id.wrapping_add(1);
            (*s).id = fresh1;
            let mut generated = if prefix.is_null() {
                Vec::new()
            } else {
                let mut bytes = CStr::from_ptr(prefix).to_bytes().to_vec();
                bytes.push(b'-');
                bytes
            };
            generated.extend_from_slice((*s).id.to_string().as_bytes());
            drop(session_replace_name(
                &mut *s,
                CString::new(generated).expect("generated name has no NUL"),
            ));
            if sessions_find(&*std::ptr::addr_of!(sessions), &*s).is_null() {
                break;
            }
        }
    }
    sessions_insert(&raw mut sessions, s);
    log_debug(
        b"new session %s $%u\0" as *const u8 as *const ::core::ffi::c_char,
        ((*s).name).as_ptr().cast_mut(),
        (*s).id,
    );
    if gettimeofday(&raw mut (*s).creation_time, NULL) != 0 as ::core::ffi::c_int {
        fatal(b"gettimeofday failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    session_update_activity(s, &raw mut (*s).creation_time);
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn session_add_ref(
    mut s: *mut session,
    mut from: *const ::core::ffi::c_char,
) {
    (*s).references += 1;
    log_debug(
        b"%s: %s %s, now %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"session_add_ref\0" as *const u8 as *const ::core::ffi::c_char,
        ((*s).name).as_ptr().cast_mut(),
        from,
        (*s).references,
    );
}
#[no_mangle]
pub unsafe extern "C" fn session_remove_ref(
    mut s: *mut session,
    mut from: *const ::core::ffi::c_char,
) {
    (*s).references -= 1;
    log_debug(
        b"%s: %s %s, now %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"session_remove_ref\0" as *const u8 as *const ::core::ffi::c_char,
        ((*s).name).as_ptr().cast_mut(),
        from,
        (*s).references,
    );
    if (*s).references == 0 as ::core::ffi::c_int {
        event_once(
            -(1 as ::core::ffi::c_int),
            EV_TIMEOUT as ::core::ffi::c_short,
            move |fd, flags| unsafe { session_free(fd, flags, s as *mut ::core::ffi::c_void) },
            ::core::ptr::null::<timeval>(),
        );
    }
}
unsafe fn session_free(
    _fd: ::core::ffi::c_int,
    _events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut s: *mut session = arg as *mut session;
    log_debug(
        b"session %s freed (%d references)\0" as *const u8 as *const ::core::ffi::c_char,
        ((*s).name).as_ptr().cast_mut(),
        (*s).references,
    );
    if (*s).references == 0 as ::core::ffi::c_int {
        environ_free((*s).environ);
        options_free((*s).options);
        crate::src::window::winlink_stack_clear(&mut (*s).lastw);
        drop(Box::from_raw(s));
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_destroy(
    mut s: *mut session,
    mut notify: ::core::ffi::c_int,
    mut from: *const ::core::ffi::c_char,
) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    log_debug(
        b"session %s destroyed (%s)\0" as *const u8 as *const ::core::ffi::c_char,
        ((*s).name).as_ptr().cast_mut(),
        from,
    );
    if (*s).curw.is_null() {
        return;
    }
    (*s).curw = ::core::ptr::null_mut::<winlink>();
    sessions_remove(&raw mut sessions, s);
    if notify != 0 {
        events_fire_session(
            b"session-closed\0" as *const u8 as *const ::core::ffi::c_char,
            s,
        );
    }
    (*s).tio = None;
    (*s).tio = None;
    if event_initialized(&(*s).lock_timer) != 0 {
        event_del(&raw mut (*s).lock_timer);
    }
    session_group_remove(s);
    while !crate::src::window::winlink_stack_first(&(*s).lastw, &raw mut (*s).windows).is_null() {
        let first = crate::src::window::winlink_stack_first(&(*s).lastw, &raw mut (*s).windows);
        winlink_stack_remove(&raw mut (*s).lastw, first);
    }
    crate::src::window::winlink_stack_clear(&mut (*s).lastw);
    while (*s).windows.storage.is_some() {
        wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
        events_fire_winlink(
            b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
            wl,
        );
        winlink_remove(&raw mut (*s).windows, wl);
    }
    session_set_cwd(&mut *s, None);
    session_remove_ref(
        s,
        b"session_destroy\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
unsafe fn session_lock_timer(
    _fd: ::core::ffi::c_int,
    _events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut s: *mut session = arg as *mut session;
    if (*s).attached == 0 as u_int {
        return;
    }
    log_debug(
        b"session %s locked, activity time %lld\0" as *const u8 as *const ::core::ffi::c_char,
        ((*s).name).as_ptr().cast_mut(),
        (*s).activity_time.tv_sec as ::core::ffi::c_longlong,
    );
    server_lock_session(s);
    recalculate_sizes();
}
#[no_mangle]
pub unsafe extern "C" fn session_update_activity(mut s: *mut session, mut from: *mut timeval) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if from.is_null() {
        gettimeofday(&raw mut (*s).activity_time, NULL);
    } else {
        memcpy(
            &raw mut (*s).activity_time as *mut ::core::ffi::c_void,
            from as *const ::core::ffi::c_void,
            ::core::mem::size_of::<timeval>() as size_t,
        );
    }
    log_debug(
        b"session $%u %s activity %lld.%06d\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).id,
        ((*s).name).as_ptr().cast_mut(),
        (*s).activity_time.tv_sec as ::core::ffi::c_longlong,
        (*s).activity_time.tv_usec as ::core::ffi::c_int,
    );
    if event_initialized(&(*s).lock_timer) != 0 {
        event_del(&raw mut (*s).lock_timer);
    } else {
        event_set(
            &raw mut (*s).lock_timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            move |fd, flags| unsafe { session_lock_timer(fd, flags, s as *mut ::core::ffi::c_void) },
        );
    }
    if (*s).attached != 0 as u_int {
        tv.tv_usec = 0 as __suseconds_t;
        tv.tv_sec = tv.tv_usec as __time_t;
        tv.tv_sec = options_get_number(
            (*s).options,
            b"lock-after-time\0" as *const u8 as *const ::core::ffi::c_char,
        ) as __time_t;
        if tv.tv_sec != 0 as __time_t {
            event_add(&raw mut (*s).lock_timer, &raw mut tv);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_next_session(
    mut s: *mut session,
    mut sort_crit: *mut sort_criteria,
) -> *mut session {
    let mut i: u_int = 0;
    if sessions.storage.is_none() || session_alive(s) == 0 {
        return ::core::ptr::null_mut::<session>();
    }
    let l = sort_get_sessions(sort_crit);
    let n = u_int::try_from(l.len()).expect("too many sessions to switch");
    i = 0 as u_int;
    while i < n {
        if l[i as usize] == s {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i == n {
        fatalx(
            b"session %s not found in sorted list\0" as *const u8 as *const ::core::ffi::c_char,
            ((*s).name).as_ptr().cast_mut(),
        );
    }
    i = i.wrapping_add(1);
    if i == n {
        i = 0 as u_int;
    }
    return l[i as usize];
}
#[no_mangle]
pub unsafe extern "C" fn session_previous_session(
    mut s: *mut session,
    mut sort_crit: *mut sort_criteria,
) -> *mut session {
    let mut i: u_int = 0;
    if sessions.storage.is_none() || session_alive(s) == 0 {
        return ::core::ptr::null_mut::<session>();
    }
    let l = sort_get_sessions(sort_crit);
    let n = u_int::try_from(l.len()).expect("too many sessions to switch");
    i = 0 as u_int;
    while i < n {
        if l[i as usize] == s {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i == n {
        fatalx(
            b"session %s not found in sorted list\0" as *const u8 as *const ::core::ffi::c_char,
            ((*s).name).as_ptr().cast_mut(),
        );
    }
    if i == 0 as u_int {
        i = n;
    }
    i = i.wrapping_sub(1);
    return l[i as usize];
}
pub unsafe fn session_attach(
    mut s: *mut session,
    mut w: *mut window,
    mut idx: ::core::ffi::c_int,
) -> Result<*mut winlink, std::ffi::CString> {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = winlink_add(&raw mut (*s).windows, idx);
    if wl.is_null() {
        return Err(std::ffi::CString::new(format!("index in use: {idx}"))
            .expect("numeric diagnostic contains no NUL"));
    }
    (*wl).session = s;
    winlink_set_window(wl, w);
    events_fire_winlink(
        b"window-linked\0" as *const u8 as *const ::core::ffi::c_char,
        wl,
    );
    session_group_synchronize_from(s);
    Ok(wl)
}
#[no_mangle]
pub unsafe extern "C" fn session_detach(
    mut s: *mut session,
    mut wl: *mut winlink,
) -> ::core::ffi::c_int {
    if winlinks_minmax(&(*s).windows, RB_NEGINF) == wl
        && winlinks_minmax(&(*s).windows, RB_INF) == wl
    {
        return 1 as ::core::ffi::c_int;
    }
    if (*s).curw == wl
        && session_last(s) != 0 as ::core::ffi::c_int
        && session_previous(s, 0 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int
    {
        session_next(s, 0 as ::core::ffi::c_int);
    }
    (*wl).flags &= !WINLINK_ALERTFLAGS;
    events_fire_winlink(
        b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
        wl,
    );
    winlink_stack_remove(&raw mut (*s).lastw, wl);
    winlink_remove(&raw mut (*s).windows, wl);
    session_group_synchronize_from(s);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn session_has(
    mut s: *mut session,
    mut w: *mut window,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = window_winlinks_first(w);
    while !wl.is_null() {
        if (*wl).session == s {
            return 1 as ::core::ffi::c_int;
        }
        wl = window_winlinks_next(w, wl);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn session_is_linked(
    mut s: *mut session,
    mut w: *mut window,
) -> ::core::ffi::c_int {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains(s);
    if !sg.is_null() {
        return ((*w).references != session_group_count(sg)) as ::core::ffi::c_int;
    }
    return ((*w).references != 1 as u_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn session_next_alert(mut wl: *mut winlink) -> *mut winlink {
    while !wl.is_null() {
        if (*wl).flags & WINLINK_ALERTFLAGS != 0 {
            break;
        }
        wl = winlink_next(wl);
    }
    return wl;
}
#[no_mangle]
pub unsafe extern "C" fn session_next(
    mut s: *mut session,
    mut alert: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if (*s).curw.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    wl = winlink_next((*s).curw);
    if alert != 0 {
        wl = session_next_alert(wl);
    }
    if wl.is_null() {
        wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
        if alert != 0 && {
            wl = session_next_alert(wl);
            wl.is_null()
        } {
            return -(1 as ::core::ffi::c_int);
        }
    }
    return session_set_current(s, wl);
}
unsafe extern "C" fn session_previous_alert(mut wl: *mut winlink) -> *mut winlink {
    while !wl.is_null() {
        if (*wl).flags & WINLINK_ALERTFLAGS != 0 {
            break;
        }
        wl = winlink_previous(wl);
    }
    return wl;
}
#[no_mangle]
pub unsafe extern "C" fn session_previous(
    mut s: *mut session,
    mut alert: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if (*s).curw.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    wl = winlink_previous((*s).curw);
    if alert != 0 {
        wl = session_previous_alert(wl);
    }
    if wl.is_null() {
        wl = winlinks_minmax(&(*s).windows, RB_INF);
        if alert != 0 && {
            wl = session_previous_alert(wl);
            wl.is_null()
        } {
            return -(1 as ::core::ffi::c_int);
        }
    }
    return session_set_current(s, wl);
}
#[no_mangle]
pub unsafe extern "C" fn session_select(
    mut s: *mut session,
    mut idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = winlink_find_by_index(&raw mut (*s).windows, idx);
    return session_set_current(s, wl);
}
#[no_mangle]
pub unsafe extern "C" fn session_last(mut s: *mut session) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = crate::src::window::winlink_stack_first(&(*s).lastw, &raw mut (*s).windows);
    if wl.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if wl == (*s).curw {
        return 1 as ::core::ffi::c_int;
    }
    return session_set_current(s, wl);
}
unsafe extern "C" fn session_fire_window_changed(
    mut s: *mut session,
    mut wl: *mut winlink,
    mut old: *mut winlink,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_winlink(&raw mut fs, wl, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_session(
        ep,
        b"session\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).window,
    );
    event_payload_set_window(
        ep,
        b"new_window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).window,
    );
    event_payload_set_int(
        ep,
        b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).idx,
    );
    event_payload_set_int(
        ep,
        b"new_window_index\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).idx,
    );
    if !old.is_null() {
        event_payload_set_window(
            ep,
            b"old_window\0" as *const u8 as *const ::core::ffi::c_char,
            (*old).window,
        );
        event_payload_set_int(
            ep,
            b"old_window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*old).idx,
        );
    }
    events_fire(
        b"session-window-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
#[no_mangle]
pub unsafe extern "C" fn session_set_current(
    mut s: *mut session,
    mut wl: *mut winlink,
) -> ::core::ffi::c_int {
    let mut old: *mut winlink = (*s).curw;
    if wl.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if wl == (*s).curw {
        return 1 as ::core::ffi::c_int;
    }
    winlink_stack_remove(&raw mut (*s).lastw, wl);
    winlink_stack_push(&raw mut (*s).lastw, (*s).curw);
    (*s).curw = wl;
    if options_get_number(
        global_options,
        b"focus-events\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        if !old.is_null() {
            window_update_focus((*old).window);
        }
        window_update_focus((*wl).window);
    }
    winlink_clear_flags(wl);
    window_update_activity((*wl).window);
    tty_update_window_offset((*wl).window);
    session_fire_window_changed(s, wl, old);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn session_group_contains(mut target: *mut session) -> *mut session_group {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_groups_minmax(&*std::ptr::addr_of!(session_groups), RB_NEGINF);
    while !sg.is_null() {
        if (*sg).members.contains(&target) {
            return sg;
        }
        sg = session_groups_next(&*sg);
    }
    return ::core::ptr::null_mut::<session_group>();
}
#[no_mangle]
pub unsafe extern "C" fn session_group_find(
    mut name: *const ::core::ffi::c_char,
) -> *mut session_group {
    let mut sg: session_group = session_group {
        name: Default::default(),
        entry: session_group_entry { owner: None },
        ..session_group::empty()
    };
    sg.name = ::std::ffi::CStr::from_ptr(name).to_owned();
    return session_groups_find(&*std::ptr::addr_of!(session_groups), &sg);
}
#[no_mangle]
pub unsafe extern "C" fn session_group_new(
    mut name: *const ::core::ffi::c_char,
) -> *mut session_group {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_find(name);
    if !sg.is_null() {
        return sg;
    }
    let owner = session_group::new(std::ffi::CStr::from_ptr(name));
    sg = owner.node_ptr();
    assert!(session_groups_insert(&raw mut session_groups, owner).is_null());
    return sg;
}
unsafe extern "C" fn session_group_fire(
    mut name: *const ::core::ffi::c_char,
    mut sg: *mut session_group,
    mut s: *mut session,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    ep = event_payload_create();
    if session_alive(s) != 0 {
        cmd_find_from_session(&raw mut fs, s, 0 as ::core::ffi::c_int);
        event_payload_set_target(ep, &raw mut fs);
    }
    event_payload_set_session(
        ep,
        b"session\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    event_payload_set_string(
        ep,
        b"group\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        ((*sg).name).as_ptr().cast_mut(),
    );
    event_payload_set_uint(
        ep,
        b"group_size\0" as *const u8 as *const ::core::ffi::c_char,
        session_group_count(sg),
    );
    events_fire(name, ep);
}
#[no_mangle]
pub unsafe extern "C" fn session_group_add(mut sg: *mut session_group, mut s: *mut session) {
    if session_group_contains(s).is_null() {
        (*sg).members.push(s);
        session_group_fire(
            b"session-added-to-group\0" as *const u8 as *const ::core::ffi::c_char,
            sg,
            s,
        );
    }
}
unsafe extern "C" fn session_group_remove(mut s: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains(s);
    if sg.is_null() {
        return;
    }
    session_group_fire(
        b"session-removed-from-group\0" as *const u8 as *const ::core::ffi::c_char,
        sg,
        s,
    );
    let members = &mut (*sg).members;
    let index = members
        .iter()
        .position(|member| *member == s)
        .expect("session group membership disappeared");
    members.remove(index);
    if members.is_empty() {
        assert!(session_groups_remove(&raw mut session_groups, sg));
    }
}
/// Returns the group's members in their preserved insertion order.
pub unsafe fn session_group_members(sg: *mut session_group) -> Vec<*mut session> {
    if sg.is_null() {
        return Vec::new();
    }
    (*sg).members.clone()
}
#[no_mangle]
pub unsafe extern "C" fn session_group_count(mut sg: *mut session_group) -> u_int {
    return u_int::try_from((*sg).members.len()).expect("session group has too many members");
}
#[no_mangle]
pub unsafe extern "C" fn session_group_attached_count(mut sg: *mut session_group) -> u_int {
    (*sg)
        .members
        .iter()
        .fold(0, |count, member| count.wrapping_add((**member).attached))
}
#[no_mangle]
pub unsafe extern "C" fn session_group_synchronize_to(mut s: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains(s);
    if sg.is_null() {
        return;
    }
    let target = (*sg)
        .members
        .iter()
        .copied()
        .find(|target| *target != s)
        .unwrap_or(std::ptr::null_mut());
    if !target.is_null() {
        session_group_synchronize1(target, s);
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_group_synchronize_from(mut target: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains(target);
    if sg.is_null() {
        return;
    }
    for s in session_group_members(sg) {
        if s != target {
            session_group_synchronize1(target, s);
        }
    }
}
unsafe extern "C" fn session_group_synchronize1(mut target: *mut session, mut s: *mut session) {
    let mut ww: *mut winlinks = ::core::ptr::null_mut::<winlinks>();
    let mut old_windows: winlinks;
    let mut old_lastw: winlink_stack;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wl2: *mut winlink = ::core::ptr::null_mut::<winlink>();
    ww = &raw mut (*target).windows;
    if (*ww).storage.is_none() {
        return;
    }
    if !(*s).curw.is_null()
        && winlink_find_by_index(ww, (*(*s).curw).idx).is_null()
        && session_last(s) != 0 as ::core::ffi::c_int
        && session_previous(s, 0 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int
    {
        session_next(s, 0 as ::core::ffi::c_int);
    }
    old_windows = std::ptr::replace(&mut (*s).windows, winlinks { storage: None });
    wl = winlinks_minmax(&*ww, RB_NEGINF);
    while !wl.is_null() {
        wl2 = winlink_add(&raw mut (*s).windows, (*wl).idx);
        (*wl2).session = s;
        winlink_set_window(wl2, (*wl).window);
        events_fire_winlink(
            b"window-linked\0" as *const u8 as *const ::core::ffi::c_char,
            wl2,
        );
        (*wl2).flags |= (*wl).flags & WINLINK_ALERTFLAGS;
        wl = winlinks_next(&*wl);
    }
    if !(*s).curw.is_null() {
        (*s).curw = winlink_find_by_index(&raw mut (*s).windows, (*(*s).curw).idx);
    } else if !(*target).curw.is_null() {
        (*s).curw = winlink_find_by_index(&raw mut (*s).windows, (*(*target).curw).idx);
    }
    if (*s).curw.is_null() {
        (*s).curw = winlinks_minmax(&(*s).windows, RB_NEGINF);
    }
    old_lastw = std::ptr::replace(
        &raw mut (*s).lastw,
        winlink_stack {
            storage: None,
            reserved: std::ptr::null_mut(),
        },
    );
    for old_idx in crate::src::window::winlink_stack_indices(&old_lastw) {
        wl2 = winlink_find_by_index(&raw mut (*s).windows, old_idx);
        if !wl2.is_null() {
            crate::src::window::winlink_stack_append(&mut (*s).lastw, wl2);
        }
    }
    crate::src::window::winlink_stack_clear(&mut old_lastw);
    while old_windows.storage.is_some() {
        wl = winlinks_minmax(&old_windows, RB_NEGINF);
        wl2 = winlink_find_by_window_id(&raw mut (*s).windows, (*(*wl).window).id);
        if wl2.is_null() {
            events_fire_winlink(
                b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
                wl,
            );
        }
        winlink_remove(&raw mut old_windows, wl);
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_renumber_windows(mut s: *mut session) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wl1: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wl_new: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut old_wins: winlinks = std::ptr::replace(&mut (*s).windows, winlinks { storage: None });
    let mut old_lastw: winlink_stack;
    let mut new_idx: ::core::ffi::c_int = 0;
    let mut new_curw_idx: ::core::ffi::c_int = 0;
    let mut marked_idx: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    new_idx = options_get_number(
        (*s).options,
        b"base-index\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    new_curw_idx = 0 as ::core::ffi::c_int;
    wl = winlinks_minmax(&old_wins, RB_NEGINF);
    while !wl.is_null() {
        wl_new = winlink_add(&raw mut (*s).windows, new_idx);
        (*wl_new).session = s;
        winlink_set_window(wl_new, (*wl).window);
        (*wl_new).flags |= (*wl).flags & WINLINK_ALERTFLAGS;
        if wl == marked_pane.wl {
            marked_idx = (*wl_new).idx;
        }
        if wl == (*s).curw {
            new_curw_idx = (*wl_new).idx;
        }
        new_idx += 1;
        wl = winlinks_next(&*wl);
    }
    old_lastw = std::ptr::replace(
        &raw mut (*s).lastw,
        winlink_stack {
            storage: None,
            reserved: std::ptr::null_mut(),
        },
    );
    for old_idx in crate::src::window::winlink_stack_indices(&old_lastw) {
        wl = winlink_find_by_index(&raw mut old_wins, old_idx);
        if wl.is_null() {
            continue;
        }
        (*wl).flags &= !WINLINK_VISITED;
        wl_new = winlink_find_by_window(&raw mut (*s).windows, (*wl).window);
        if !wl_new.is_null() {
            crate::src::window::winlink_stack_append(&mut (*s).lastw, wl_new);
        }
    }
    crate::src::window::winlink_stack_clear(&mut old_lastw);
    if marked_idx != -(1 as ::core::ffi::c_int) {
        marked_pane.wl = winlink_find_by_index(&raw mut (*s).windows, marked_idx);
        if marked_pane.wl.is_null() {
            server_clear_marked();
        }
    }
    (*s).curw = winlink_find_by_index(&raw mut (*s).windows, new_curw_idx);
    wl = winlinks_minmax(&old_wins, RB_NEGINF);
    while !wl.is_null() && {
        wl1 = winlinks_next(&*wl);
        1 as ::core::ffi::c_int != 0
    } {
        winlink_remove(&raw mut old_wins, wl);
        wl = wl1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_theme_changed(mut s: *mut session) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !s.is_null() {
        wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
        while !wl.is_null() {
            wp = window_pane_first((*wl).window);
            while !wp.is_null() {
                (*wp).flags |= PANE_THEMECHANGED;
                wp = window_pane_next(wp);
            }
            wl = winlinks_next(&*wl);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_update_history(mut s: *mut session) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut limit: u_int = 0;
    let mut osize: u_int = 0;
    limit = options_get_number(
        (*s).options,
        b"history-limit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
    while !wl.is_null() {
        wp = window_pane_first((*wl).window);
        while !wp.is_null() {
            gd = (*wp).base.grid;
            osize = (*gd).hsize;
            (*gd).hlimit = limit;
            grid_collect_history(gd, 1 as ::core::ffi::c_int);
            if (*gd).hsize != osize {
                log_debug(
                    b"%s: %%%u %u -> %u\0" as *const u8 as *const ::core::ffi::c_char,
                    b"session_update_history\0" as *const u8 as *const ::core::ffi::c_char,
                    (*wp).id,
                    osize,
                    (*gd).hsize,
                );
            }
            wp = window_pane_next(wp);
        }
        wl = winlinks_next(&*wl);
    }
}

#[cfg(test)]
mod session_index_tests {
    use super::*;

    fn session(name: &str) -> *mut session {
        let mut session = Box::new(session::empty());
        session.name = std::ffi::CString::new(name).unwrap();
        Box::into_raw(session)
    }

    #[test]
    fn session_index_observers_follow_move_duplicate_and_removal() {
        unsafe {
            let mut head = sessions { storage: None };
            let mut other = sessions { storage: None };
            let first = session("alpha");
            let second = session("beta");
            let duplicate = session("alpha");

            assert!(sessions_insert(&mut head, first).is_null());
            assert!(sessions_insert(&mut head, second).is_null());
            let index_observer = (*first).entry.owner.as_ref().unwrap().clone();
            assert_eq!(sessions_insert(&mut head, duplicate), first);
            assert!((*duplicate).entry.owner.is_none());
            assert!(sessions_remove(&mut other, first).is_null());
            assert!((*first).entry.owner.is_some());

            let mut moved = head;
            assert_eq!(sessions_minmax(&moved, -1), first);
            assert_eq!(sessions_next(&*first), second);
            assert_eq!(sessions_remove(&mut moved, first), first);
            assert!((*first).entry.owner.is_none());
            assert!(sessions_next(&*first).is_null());
            assert_eq!(sessions_remove(&mut moved, second), second);

            drop(Box::from_raw(duplicate));
            drop(Box::from_raw(first));
            drop(Box::from_raw(second));
            drop(moved);
            assert!(matches!(
                index_observer.try_borrow_mut(),
                Err(refbox::BorrowError::Dropped)
            ));
        }
    }

    #[test]
    fn session_group_index_drops_nodes_after_releasing_the_map_borrow() {
        unsafe {
            let mut head = session_groups { storage: None };
            let mut other = session_groups { storage: None };
            let first_owner = session_group::new(c"alpha");
            let first = first_owner.node_ptr();
            let second_owner = session_group::new(c"beta");
            let second = second_owner.node_ptr();
            assert!(session_groups_insert(&mut head, first_owner).is_null());
            assert!(session_groups_insert(&mut head, second_owner).is_null());

            let index_observer = (*second).entry.owner.as_ref().unwrap().clone();
            let duplicate = session_group::new(c"alpha");
            assert_eq!(session_groups_insert(&mut head, duplicate), first);
            assert!((*second).entry.owner.is_some());
            assert!(!session_groups_remove(&mut other, second));
            assert!((*second).entry.owner.is_some());

            let mut moved = head;
            assert_eq!(session_groups_next(&*first), second);
            assert!(session_groups_remove(&mut moved, first));
            assert!(index_observer.try_borrow_mut().is_ok());
            assert!(session_groups_remove(&mut moved, second));
            drop(moved);
            assert!(matches!(
                index_observer.try_borrow_mut(),
                Err(refbox::BorrowError::Dropped)
            ));
        }
    }
}

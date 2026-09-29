use crate::src::options::options_owner_ptr;
use crate::src::cmd::find::{cmd_find_from_session, cmd_find_from_winlink};
use crate::src::compat::strtonum::strtonum;
use crate::src::events::{events_fire, events_fire_session, events_fire_winlink};
use crate::src::events_payload::{
    event_payload_create, event_payload_set_int, event_payload_set_session,
    event_payload_set_string, event_payload_set_target, event_payload_set_uint,
    event_payload_set_window,
};
use crate::src::ffi::libc::{gettimeofday, memcpy, strcmp};
use crate::src::format::bytes::write_cstr;
use crate::src::grid::grid_collect_history;
use crate::src::log::{fatal, fatalx, log_bytes, log_cstr, log_debug};
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
use std::cell::UnsafeCell;
use std::ffi::{CStr, CString};
use std::rc::Rc;
pub static mut sessions: sessions = sessions { storage: None };
pub static mut next_session_id: u_int = 0;
pub static mut session_groups: session_groups = session_groups { storage: None };

/// `session.cwd` borrows this value until replacement or early destruction.
pub(crate) fn session_set_cwd(s: &mut session, cwd: Option<CString>) {
    s.cwd = cwd;
}

/// Replace the borrowed public name after callers remove the old map key.
pub(crate) fn session_replace_name(s: &mut session, name: CString) -> CString {
    std::mem::replace(&mut s.name, name)
}
pub(crate) fn sessions_key(elm: &session) -> Vec<u8> {
    elm.name.as_bytes().to_vec()
}
pub fn sessions_find(head: &sessions, elm: &session) -> Option<Rc<UnsafeCell<session>>> {
    let owner = head.storage.as_ref()?;
    let map = owner.try_borrow_mut().expect("session index already borrowed");
    map.get(elm.name.as_bytes()).cloned()
}
pub unsafe fn sessions_insert(
    head: &mut sessions,
    session: Rc<UnsafeCell<session>>,
) -> Option<Rc<UnsafeCell<session>>> {
    let elm = session.get();
    let key = (*elm).name.as_bytes();
    let owner = head.storage.get_or_insert_with(refbox::RefBox::default);
    let observer = owner.downgrade();
    let mut map = owner.try_borrow_mut().expect("session index already borrowed");
    match map.entry(key.to_vec()) {
        std::collections::btree_map::Entry::Occupied(entry) => Some(entry.get().clone()),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(session);
            (*elm).entry.owner = observer;
            None
        }
    }
}
pub unsafe fn sessions_remove(
    head: &mut sessions,
    elm: &Rc<UnsafeCell<session>>,
) -> Option<Rc<UnsafeCell<session>>> {
    let key = (*elm.get()).name.as_bytes();
    let owner = head.storage.as_ref()?;
    let (session, empty) = {
        let mut map = owner.try_borrow_mut().expect("session index already borrowed");
        if !map.get(key).is_some_and(|candidate| Rc::ptr_eq(candidate, elm)) {
            return None;
        }
        (map.remove(key).expect("matching session"), map.is_empty())
    };
    (*session.get()).entry.owner = refbox::Weak::new();
    if empty {
        head.storage = None;
    }
    Some(session)
}
pub fn sessions_minmax(head: &sessions) -> Option<Rc<UnsafeCell<session>>> {
    let Some(owner) = head.storage.as_ref() else {
        return None;
    };
    let map = owner
        .try_borrow_mut()
        .expect("session index already borrowed");
    let pair = map.first_key_value();
    pair.map(|(_, node)| Rc::clone(node))
}
/// Resume a potentially destructive walk using a saved name and the live index.
/// The named session and any of its successors may already have been removed.
pub fn sessions_after(head: &sessions, name: &[u8]) -> Option<Rc<UnsafeCell<session>>> {
    let Some(owner) = head.storage.as_ref() else {
        return None;
    };
    let map = owner
        .try_borrow_mut()
        .expect("session index already borrowed");
    map.range::<[u8], _>((std::ops::Bound::Excluded(name), std::ops::Bound::Unbounded))
        .next()
        .map(|(_, node)| Rc::clone(node))
}

/// The session must still belong to its index. Destructive walks use sessions_after.
pub unsafe fn sessions_next(elm: &session) -> Option<Rc<UnsafeCell<session>>> {
    let owner = &elm.entry.owner;
    let map = match owner.try_borrow_mut() {
        Ok(map) => map,
        Err(refbox::BorrowError::Dropped) => return None,
        Err(refbox::BorrowError::Borrowed) => panic!("session index already borrowed"),
    };
    let key = elm.name.as_bytes();
    map.range::<[u8], _>((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
        .next()
        .map(|(_, node)| Rc::clone(node))
}
pub fn session_groups_find(head: &session_groups, elm: &session_group) -> *mut session_group {
    let Some(owner) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = owner
        .try_borrow_mut()
        .expect("session group index already borrowed");
    let key = elm.name.as_bytes();
    map.get(key)
        .map_or(std::ptr::null_mut(), |owner| owner.node_ptr())
}
impl session_group {
    pub fn new(name: &std::ffi::CStr) -> Box<Self> {
        let mut owner = Box::new(session_group {
            name: name.to_owned(),
            entry: session_group_entry { owner: refbox::Weak::new() },
            members: Vec::new(),
        });

        owner
    }

    pub fn node_ptr(&self) -> *mut session_group {
        (self as *const Self).cast_mut()
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
            (*elm).entry.owner = observer;
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
    let key = (*elm).name.as_bytes();
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
        (*elm).entry.owner = refbox::Weak::new();
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
pub fn session_groups_minmax(head: &session_groups) -> *mut session_group {
    let Some(storage) = head.storage.as_ref() else {
        return std::ptr::null_mut();
    };
    let map = storage
        .try_borrow_mut()
        .expect("session group index already borrowed");
    let pair = map.first_key_value();
    pair.map_or(std::ptr::null_mut(), |(_, owner)| owner.node_ptr())
}
pub unsafe fn session_groups_next(elm: &session_group) -> *mut session_group {
    let owner = &elm.entry.owner;
    let map = match owner.try_borrow_mut() {
        Ok(map) => map,
        Err(refbox::BorrowError::Dropped) => return std::ptr::null_mut(),
        Err(refbox::BorrowError::Borrowed) => panic!("session group index already borrowed"),
    };
    let key = elm.name.as_bytes();
    map.range::<[u8], _>((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, owner)| owner.node_ptr())
}
/// Resolve an observer only while its session remains in this index.
/// A removed session can still be retained by queued events or other owners.
pub fn sessions_resolve(
    head: &sessions,
    observer: &std::rc::Weak<UnsafeCell<session>>,
) -> Option<Rc<UnsafeCell<session>>> {
    let owner = observer.upgrade()?;
    let index = head.storage.as_ref()?;
    let map = index.try_borrow_mut().expect("session index already borrowed");
    map.values().any(|candidate| Rc::ptr_eq(candidate, &owner)).then_some(owner)
}

pub unsafe fn session_alive(s: Option<&session>) -> ::core::ffi::c_int {
    s.is_some_and(|s| sessions_resolve(&sessions, &s.observer).is_some()) as ::core::ffi::c_int
}
pub unsafe fn session_find(name: &CStr) -> Option<Rc<UnsafeCell<session>>> {
    let index = sessions.storage.as_ref()?;
    let map = index.try_borrow_mut().expect("session index already borrowed");
    map.get(name.to_bytes()).cloned()
}
pub unsafe fn session_find_by_id_str(s: &CStr) -> Option<Rc<UnsafeCell<session>>> {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: u_int = 0;
    if s.to_bytes().first() != Some(&b'$') {
        return None;
    }
    id = strtonum(
        s.as_ptr().add(1),
        0 as ::core::ffi::c_longlong,
        UINT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as u_int;
    if !errstr.is_null() {
        return None;
    }
    return session_find_by_id(id);
}
pub unsafe fn session_find_by_id(mut id: u_int) -> Option<Rc<UnsafeCell<session>>> {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut s_owner = sessions_minmax(&sessions);
    s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    while !s.is_null() {
        if (*s).id == id {
            return (*s).observer.upgrade();
        }
        s_owner = sessions_next(&*s);
        s = s_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    }
    return None;
}
pub unsafe fn session_create(
    prefix: Option<&CStr>,
    name: Option<&CStr>,
    cwd: &CStr,
    env: Box<environ>,
    oo: Option<Box<options>>,
    tio: Option<&termios>,
) -> Rc<UnsafeCell<session>> {
    let owner = session::new();
    let s = crate::src::shared::rc::as_ptr(&owner);
    (*s).tio = tio.copied().map(Box::new);
    (*s).cwd = Some(cwd.to_owned());

    (*s).flags = 0 as ::core::ffi::c_int;
    (*s).lastw.storage.clear();
    (*s).windows.storage = None;
    (*s).environ = Some(env);
    (*s).options = oo;
    status_update_cache(&mut *(s));
    if let Some(name) = name {
        drop(session_replace_name(
            &mut *s,
            name.to_owned(),
        ));
        let fresh0 = next_session_id;
        next_session_id = next_session_id.wrapping_add(1);
        (*s).id = fresh0;
    } else {
        loop {
            let fresh1 = next_session_id;
            next_session_id = next_session_id.wrapping_add(1);
            (*s).id = fresh1;
            let mut generated = prefix.map_or_else(Vec::new, |prefix| {
                let mut bytes = prefix.to_bytes().to_vec();
                bytes.push(b'-');
                bytes
            });
            generated.extend_from_slice((*s).id.to_string().as_bytes());
            drop(session_replace_name(
                &mut *s,
                CString::new(generated).expect("generated name has no NUL"),
            ));
            if sessions_find(&sessions, &*s).is_none() {
                break;
            }
        }
    }
    sessions_insert(&mut sessions, Rc::clone(&owner));
    log_debug(format_args!(
        "new session {} ${}",
        log_bytes((*s).name.as_bytes()),
        ((*s).id) as u32
    ));
    if gettimeofday(&raw mut (*s).creation_time, NULL) != 0 as ::core::ffi::c_int {
        fatal(|out| out.write_all(b"gettimeofday failed"));
    }
    let created = (*s).creation_time;
    session_update_activity(&mut *s, Some(created));
    owner
}
pub unsafe fn session_add_ref(s: &session, from: *const ::core::ffi::c_char) -> Rc<UnsafeCell<session>> {
    let owner = s.observer.upgrade().expect("live Rc session");
    log_debug(format_args!(
        "{}: {} {}, now {}",
        "session_add_ref",
        log_bytes(s.name.as_bytes()),
        log_cstr((from) as *const _),
        s.observer.strong_count() as ::core::ffi::c_int
    ));
    owner
}
/// Consume one session owner and defer its release until the event loop runs.
pub unsafe fn session_remove_ref(s: Rc<UnsafeCell<session>>, from: &CStr) {
    log_debug(format_args!(
        "release session {} ({})",
        log_bytes((*s.get()).name.as_bytes()),
        log_bytes(from.to_bytes())
    ));
    crate::src::shared::rc::release_later(s);
}
unsafe fn session_free(s: &mut session) {
    log_debug(format_args!(
        "session {} freed",
        log_bytes(s.name.as_bytes())
    ));
    drop(s.environ.take());
    drop(s.options.take());
    crate::src::window::winlink_stack_clear(&mut s.lastw);
}
pub unsafe fn session_destroy(
    s_owner: &Rc<UnsafeCell<session>>,
    mut notify: ::core::ffi::c_int,
    mut from: *const ::core::ffi::c_char,
) {
    let s = s_owner.get();

    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    log_debug(format_args!(
        "session {} destroyed ({})",
        log_bytes((*s).name.as_bytes()),
        log_cstr((from) as *const _)
    ));
    // This field also marks explicit session teardown. An expired observer
    // still needs the normal destruction path if the index owner remains.
    if (*s).curw.is_empty() {
        return;
    }
    (*s).set_curw((refbox::Weak::new()).clone());
    let owner = sessions_remove(&mut sessions, s_owner).expect("registered session owner");
    if notify != 0 {
        events_fire_session(
            b"session-closed\0" as *const u8 as *const ::core::ffi::c_char,
            (*(s)).observer.upgrade().expect("live session"),
        );
    }
    (*s).tio = None;
    (*s).tio = None;
    if event_initialized(&(*s).lock_timer) != 0 {
        event_del(&raw mut (*s).lock_timer);
    }
    session_group_remove(s_owner);
    while crate::src::window::winlink_stack_first(&(*s).lastw).is_alive() {
        let first = crate::src::window::winlink_stack_first(&(*s).lastw);
        winlink_stack_remove(&raw mut (*s).lastw, (first).clone());
    }
    crate::src::window::winlink_stack_clear(&mut (*s).lastw);
    while (*s).windows.storage.is_some() {
        wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
        events_fire_winlink(
            b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
            wl.clone(),
        );
        winlink_remove(&raw mut (*s).windows, wl.clone());
    }
    session_set_cwd(&mut *s, None);
    session_remove_ref(owner, c"session_destroy");
}
unsafe fn session_lock_timer(owner: &Rc<UnsafeCell<session>>) {
    let session = &*owner.get();
    if session_alive(Some(session)) == 0 || session.attached == 0 {
        return;
    }
    log_debug(format_args!(
        "session {} locked, activity time {}",
        log_bytes(session.name.as_bytes()),
        session.activity_time.tv_sec as ::core::ffi::c_longlong
    ));
    server_lock_session(owner);
    recalculate_sizes();
}
pub unsafe fn session_update_activity(session: &mut session, from: Option<timeval>) {
    if let Some(from) = from {
        session.activity_time = from;
    } else {
        gettimeofday(&raw mut session.activity_time, NULL);
    }
    log_debug(format_args!(
        "session ${} {} activity {}.{:06}",
        session.id,
        log_bytes(session.name.as_bytes()),
        session.activity_time.tv_sec as ::core::ffi::c_longlong,
        session.activity_time.tv_usec as ::core::ffi::c_int
    ));
    if event_initialized(&session.lock_timer) != 0 {
        event_del(&raw mut session.lock_timer);
    } else {
        let observer = session.observer.clone();
        event_set(
            &raw mut session.lock_timer,
            -1,
            0,
            move |_, _| unsafe {
                if let Some(owner) = observer.upgrade() {
                    session_lock_timer(&owner);
                }
            },
        );
    }
    if session.attached != 0 {
        let mut timeout = timeval {
            tv_sec: crate::src::options::options_get_number_ref(
                session.options.as_deref().expect("session options"), c"lock-after-time",
            ) as __time_t,
            tv_usec: 0,
        };
        if timeout.tv_sec != 0 {
            event_add(&raw mut session.lock_timer, &raw mut timeout);
        }
    }
}
pub unsafe fn session_next_session(
    s: Option<&session>,
    sort_crit: &sort_criteria,
) -> Option<Rc<UnsafeCell<session>>> {
    session_adjacent(s, sort_crit, false)
}
pub unsafe fn session_previous_session(
    s: Option<&session>,
    sort_crit: &sort_criteria,
) -> Option<Rc<UnsafeCell<session>>> {
    session_adjacent(s, sort_crit, true)
}
unsafe fn session_adjacent(
    s: Option<&session>,
    sort_crit: &sort_criteria,
    previous: bool,
) -> Option<Rc<UnsafeCell<session>>> {
    let s = s?;
    let sorted = sort_get_sessions(sort_crit);
    let index = sorted.iter().position(|owner| Rc::downgrade(owner).ptr_eq(&s.observer))?;
    let selected = if previous {
        if index == 0 { sorted.len() - 1 } else { index - 1 }
    } else {
        (index + 1) % sorted.len()
    };
    Some(sorted[selected].clone())
}
pub unsafe fn session_attach(
    s_owner: &Rc<UnsafeCell<session>>,
    window_owner: &Rc<UnsafeCell<window>>,
    mut idx: ::core::ffi::c_int,
) -> Result<refbox::Weak<winlink>, std::ffi::CString> {
    let s = s_owner.get();

    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    wl = winlink_add(&raw mut (*s).windows, idx);
    if !wl.is_alive() {
        return Err(std::ffi::CString::new(format!("index in use: {idx}"))
            .expect("numeric diagnostic contains no NUL"));
    }
    wl.get_mut_unchecked().session = (*s).observer.clone();
    winlink_set_window(wl.clone(), &window_owner);
    events_fire_winlink(
        b"window-linked\0" as *const u8 as *const ::core::ffi::c_char,
        wl.clone(),
    );
    session_group_synchronize_from(s_owner);
    Ok(wl)
}
pub unsafe fn session_detach(s_owner: &Rc<UnsafeCell<session>>, mut wl: refbox::Weak<winlink>) -> ::core::ffi::c_int {
    let s = s_owner.get();

    if winlinks_minmax(&(*s).windows, RB_NEGINF) == wl
        && winlinks_minmax(&(*s).windows, RB_INF) == wl
    {
        return 1 as ::core::ffi::c_int;
    }
    if (*s).current_winlink() == wl
        && session_last(s_owner) != 0 as ::core::ffi::c_int
        && session_previous(s_owner, 0 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int
    {
        session_next(s_owner, 0 as ::core::ffi::c_int);
    }
    wl.get_mut_unchecked().flags &= !WINLINK_ALERTFLAGS;
    events_fire_winlink(
        b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
        wl.clone(),
    );
    winlink_stack_remove(&raw mut (*s).lastw, wl.clone());
    winlink_remove(&raw mut (*s).windows, wl.clone());
    session_group_synchronize_from(s_owner);
    return 0 as ::core::ffi::c_int;
}
pub fn session_has(s: &session, w: &window) -> ::core::ffi::c_int {
    if s.observer.strong_count() == 0 { return 0; }
    let links = &w.winlinks.storage;
    for observer in links.iter() {
        let link = match observer.try_borrow_mut() {
            Ok(link) => link,
            Err(refbox::BorrowError::Dropped) => continue,
            Err(refbox::BorrowError::Borrowed) => panic!("winlink already borrowed during session membership check"),
        };
        if link.session.ptr_eq(&s.observer) {
            return 1;
        }
    }
    0
}
pub unsafe fn session_is_linked(s: Option<&session>, w: &window) -> ::core::ffi::c_int {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains(s);
    if !sg.is_null() {
        return (w.observer.strong_count() != session_group_count(sg) as usize)
            as ::core::ffi::c_int;
    }
    return (w.observer.strong_count() != 1) as ::core::ffi::c_int;
}
unsafe fn session_next_alert(mut wl: refbox::Weak<winlink>) -> refbox::Weak<winlink> {
    while wl.is_alive() {
        if wl.get_unchecked().flags & WINLINK_ALERTFLAGS != 0 {
            break;
        }
        wl = winlink_next(wl.clone());
    }
    return wl;
}
pub unsafe fn session_next(
    s_owner: &Rc<UnsafeCell<session>>,
    mut alert: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let s = s_owner.get();

    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    if !(*s).current_winlink().is_alive() {
        return -(1 as ::core::ffi::c_int);
    }
    wl = winlink_next(((*s).current_winlink()).clone());
    if alert != 0 {
        wl = session_next_alert(wl.clone());
    }
    if !wl.is_alive() {
        wl = winlinks_minmax(&(*s).windows, RB_NEGINF);
        if alert != 0 && {
            wl = session_next_alert(wl.clone());
            !wl.is_alive()
        } {
            return -(1 as ::core::ffi::c_int);
        }
    }
    return session_set_current(s_owner, wl.clone());
}
unsafe fn session_previous_alert(mut wl: refbox::Weak<winlink>) -> refbox::Weak<winlink> {
    while wl.is_alive() {
        if wl.get_unchecked().flags & WINLINK_ALERTFLAGS != 0 {
            break;
        }
        wl = winlink_previous(wl.clone());
    }
    return wl;
}
pub unsafe fn session_previous(
    s_owner: &Rc<UnsafeCell<session>>,
    mut alert: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let s = s_owner.get();

    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    if !(*s).current_winlink().is_alive() {
        return -(1 as ::core::ffi::c_int);
    }
    wl = winlink_previous(((*s).current_winlink()).clone());
    if alert != 0 {
        wl = session_previous_alert(wl.clone());
    }
    if !wl.is_alive() {
        wl = winlinks_minmax(&(*s).windows, RB_INF);
        if alert != 0 && {
            wl = session_previous_alert(wl.clone());
            !wl.is_alive()
        } {
            return -(1 as ::core::ffi::c_int);
        }
    }
    return session_set_current(s_owner, wl.clone());
}
pub unsafe fn session_select(
    s_owner: &Rc<UnsafeCell<session>>,
    mut idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let s = s_owner.get();

    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    wl = winlink_find_by_index(&raw mut (*s).windows, idx);
    return session_set_current(s_owner, wl.clone());
}
pub unsafe fn session_last(s_owner: &Rc<UnsafeCell<session>>) -> ::core::ffi::c_int {
    let s = s_owner.get();

    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    wl = crate::src::window::winlink_stack_first(&(*s).lastw);
    if !wl.is_alive() {
        return -(1 as ::core::ffi::c_int);
    }
    if wl == (*s).current_winlink() {
        return 1 as ::core::ffi::c_int;
    }
    return session_set_current(s_owner, wl.clone());
}
unsafe fn session_fire_window_changed(
    s_owner: &Rc<UnsafeCell<session>>,
    mut wl: refbox::Weak<winlink>,
    mut old: refbox::Weak<winlink>,
) {
    let s = s_owner.get();

    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        s: std::rc::Weak::new(),
        wl: refbox::Weak::new(),
        w: std::rc::Weak::new(),
        wp: std::rc::Weak::new(),
        idx: 0,
    };
    let mut ep = event_payload_create();
    cmd_find_from_winlink(&raw mut fs, wl.clone(), 0 as ::core::ffi::c_int);
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_session(
        &mut *ep,
        b"session\0" as *const u8 as *const ::core::ffi::c_char,
        (*(s)).observer.upgrade().expect("live session"),
    );
    event_payload_set_window(
        &mut *ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*(wl.get_unchecked().window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()))).observer.upgrade().expect("live window"),
    );
    event_payload_set_window(
        &mut *ep,
        b"new_window\0" as *const u8 as *const ::core::ffi::c_char,
        (*(wl.get_unchecked().window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()))).observer.upgrade().expect("live window"),
    );
    event_payload_set_int(
        &mut *ep,
        b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
        wl.get_unchecked().idx,
    );
    event_payload_set_int(
        &mut *ep,
        b"new_window_index\0" as *const u8 as *const ::core::ffi::c_char,
        wl.get_unchecked().idx,
    );
    if old.is_alive() {
        event_payload_set_window(
            &mut *ep,
            b"old_window\0" as *const u8 as *const ::core::ffi::c_char,
            (*(old.get_unchecked().window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()))).observer.upgrade().expect("live window"),
        );
        event_payload_set_int(
            &mut *ep,
            b"old_window_index\0" as *const u8 as *const ::core::ffi::c_char,
            old.get_unchecked().idx,
        );
    }
    events_fire(
        b"session-window-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
pub unsafe fn session_set_current(s_owner: &Rc<UnsafeCell<session>>, mut wl: refbox::Weak<winlink>) -> ::core::ffi::c_int {
    let s = s_owner.get();

    let mut old: refbox::Weak<winlink> = (*s).current_winlink();
    if !wl.is_alive() {
        return -(1 as ::core::ffi::c_int);
    }
    if wl == (*s).current_winlink() {
        return 1 as ::core::ffi::c_int;
    }
    winlink_stack_remove(&raw mut (*s).lastw, wl.clone());
    winlink_stack_push(&raw mut (*s).lastw, ((*s).current_winlink()).clone());
    (*s).set_curw(wl.clone());
    if options_get_number(
        global_options,
        b"focus-events\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        if old.is_alive() {
            window_update_focus((old.get_unchecked().window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).as_ref().and_then(|model| model.observer.upgrade()).as_ref());
        }
        window_update_focus((wl.get_unchecked().window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).as_ref().and_then(|model| model.observer.upgrade()).as_ref());
    }
    winlink_clear_flags(wl.clone());
    window_update_activity(&(*(wl.get_unchecked().window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()))).observer.upgrade().expect("live window"));
    tty_update_window_offset(&(*(wl.get_unchecked().window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()))).observer.upgrade().expect("live window"));
    session_fire_window_changed(s_owner, wl.clone(), (old).clone());
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn session_group_contains(target: Option<&session>) -> *mut session_group {
    let Some(target) = target else { return std::ptr::null_mut(); };
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_groups_minmax(&session_groups);
    while !sg.is_null() {
        if (*sg).members.iter().any(|member| member.ptr_eq(&target.observer)) {
            return sg;
        }
        sg = session_groups_next(&*sg);
    }
    return ::core::ptr::null_mut::<session_group>();
}
pub unsafe fn session_group_find(mut name: *const ::core::ffi::c_char) -> *mut session_group {
    let mut sg: session_group = session_group {
        name: Default::default(),
        entry: session_group_entry { owner: refbox::Weak::new() },
        ..session_group::empty()
    };
    sg.name = ::std::ffi::CStr::from_ptr(name).to_owned();
    return session_groups_find(&session_groups, &sg);
}
pub unsafe fn session_group_new(mut name: *const ::core::ffi::c_char) -> *mut session_group {
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
unsafe fn session_group_fire(
    mut name: *const ::core::ffi::c_char,
    mut sg: *mut session_group,
    owner: &Rc<UnsafeCell<session>>,
) {
    let s = owner.get();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        s: std::rc::Weak::new(),
        wl: refbox::Weak::new(),
        w: std::rc::Weak::new(),
        wp: std::rc::Weak::new(),
        idx: 0,
    };
    let mut ep = event_payload_create();
    if session_alive(s.as_ref()) != 0 {
        cmd_find_from_session(&raw mut fs, &(*(s)).observer.upgrade().expect("live session"), 0 as ::core::ffi::c_int);
        event_payload_set_target(&mut *ep, &fs);
    }
    event_payload_set_session(
        &mut *ep,
        b"session\0" as *const u8 as *const ::core::ffi::c_char,
        (*(s)).observer.upgrade().expect("live session"),
    );
    event_payload_set_string(
        &mut *ep,
        b"group\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, ((*sg).name).as_ptr().cast_mut()),
    );
    event_payload_set_uint(
        &mut *ep,
        b"group_size\0" as *const u8 as *const ::core::ffi::c_char,
        session_group_count(sg),
    );
    events_fire(name, ep);
}
pub unsafe fn session_group_add(sg: *mut session_group, owner: &Rc<UnsafeCell<session>>) {
    let s = owner.get();
    if session_group_contains((s).as_ref()).is_null() {
        (*sg).members.push(Rc::downgrade(owner));
        session_group_fire(
            b"session-added-to-group\0" as *const u8 as *const ::core::ffi::c_char,
            sg,
            owner,
        );
    }
}
unsafe fn session_group_remove(s_owner: &Rc<UnsafeCell<session>>) {
    let s = s_owner.get();

    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains((s).as_ref());
    if sg.is_null() {
        return;
    }
    session_group_fire(
        b"session-removed-from-group\0" as *const u8 as *const ::core::ffi::c_char,
        sg,
        s_owner,
    );
    let members = &mut (*sg).members;
    let index = members
        .iter()
        .position(|member| member.as_ptr().cast::<session>() == s)
        .expect("session group membership disappeared");
    members.remove(index);
    if members.is_empty() {
        assert!(session_groups_remove(&raw mut session_groups, sg));
    }
}
/// Retain live members in insertion order for the entire caller operation.
/// Callbacks may remove membership or destroy the group while this snapshot exists.
pub unsafe fn session_group_members(sg: *mut session_group) -> Vec<Rc<UnsafeCell<session>>> {
    if sg.is_null() {
        return Vec::new();
    }
    (*sg).members.iter().filter_map(std::rc::Weak::upgrade).collect()
}
pub unsafe fn session_group_count(mut sg: *mut session_group) -> u_int {
    return u_int::try_from((*sg).members.iter().filter(|member| member.strong_count() != 0).count()).expect("session group has too many members");
}
pub unsafe fn session_group_attached_count(mut sg: *mut session_group) -> u_int {
    session_group_members(sg)
        .iter()
        .fold(0, |count, member| count.wrapping_add((*member.get()).attached))
}
pub unsafe fn session_group_synchronize_to(s_owner: &Rc<UnsafeCell<session>>) {
    let s = s_owner.get();

    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains((s).as_ref());
    if sg.is_null() {
        return;
    }
    let target = session_group_members(sg).into_iter().find(|target| target.get() != s);
    if let Some(target) = target {
        session_group_synchronize1(&target, s_owner);
    }
}
pub unsafe fn session_group_synchronize_from(target_owner: &Rc<UnsafeCell<session>>) {
    let target = target_owner.get();

    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains((target).as_ref());
    if sg.is_null() {
        return;
    }
    for owner in session_group_members(sg) {
        if owner.get() != target {
            session_group_synchronize1(target_owner, &owner);
        }
    }
}
unsafe fn session_group_synchronize1(target_owner: &Rc<UnsafeCell<session>>, s_owner: &Rc<UnsafeCell<session>>) {
    let target = target_owner.get();
    let s = s_owner.get();

    let mut ww: *mut winlinks = ::core::ptr::null_mut::<winlinks>();
    let mut old_windows: winlinks;
    let mut old_lastw: winlink_stack;
    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut wl2: refbox::Weak<winlink> = refbox::Weak::new();
    ww = &raw mut (*target).windows;
    if (*ww).storage.is_none() {
        return;
    }
    if (*s).current_winlink().is_alive()
        && !winlink_find_by_index(ww, ((*s).current_winlink()).get_unchecked().idx).is_alive()
        && session_last(s_owner) != 0 as ::core::ffi::c_int
        && session_previous(s_owner, 0 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int
    {
        session_next(s_owner, 0 as ::core::ffi::c_int);
    }
    old_windows = std::ptr::replace(&mut (*s).windows, winlinks { storage: None });
    wl = winlinks_minmax(&*ww, RB_NEGINF);
    while wl.is_alive() {
        wl2 = winlink_add(&raw mut (*s).windows, wl.get_unchecked().idx);
        wl2.get_mut_unchecked().session = (*s).observer.clone();
        winlink_set_window((wl2).clone(), &(*(wl.get_unchecked().window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()))).observer.upgrade().expect("live window"));
        events_fire_winlink(
            b"window-linked\0" as *const u8 as *const ::core::ffi::c_char,
            (wl2).clone(),
        );
        wl2.get_mut_unchecked().flags |= wl.get_unchecked().flags & WINLINK_ALERTFLAGS;
        wl = winlinks_next(wl.get_unchecked());
    }
    if (*s).current_winlink().is_alive() {
        (*s).set_curw((winlink_find_by_index(&raw mut (*s).windows, ((*s).current_winlink()).get_unchecked().idx)).clone());
    } else if (*target).current_winlink().is_alive() {
        (*s).set_curw((winlink_find_by_index(&raw mut (*s).windows, ((*target).current_winlink()).get_unchecked().idx)).clone());
    }
    if !(*s).current_winlink().is_alive() {
        (*s).set_curw((winlinks_minmax(&(*s).windows, RB_NEGINF)).clone());
    }
    old_lastw = std::ptr::replace(
        &raw mut (*s).lastw,
        winlink_stack { storage: Default::default() },
    );
    for old_idx in crate::src::window::winlink_stack_indices(&old_lastw) {
        wl2 = winlink_find_by_index(&raw mut (*s).windows, old_idx);
        if wl2.is_alive() {
            crate::src::window::winlink_stack_append(&mut (*s).lastw, (wl2).clone());
        }
    }
    crate::src::window::winlink_stack_clear(&mut old_lastw);
    while old_windows.storage.is_some() {
        wl = winlinks_minmax(&old_windows, RB_NEGINF);
        wl2 = winlink_find_by_window_id(&raw mut (*s).windows, (*wl.get_unchecked().window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).id);
        if !wl2.is_alive() {
            events_fire_winlink(
                b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
                wl.clone(),
            );
        }
        winlink_remove(&raw mut old_windows, wl.clone());
    }
}
pub unsafe fn session_renumber_windows(s_owner: &Rc<UnsafeCell<session>>) {
    let s = s_owner.get();

    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut wl1: refbox::Weak<winlink> = refbox::Weak::new();
    let mut wl_new: refbox::Weak<winlink> = refbox::Weak::new();
    let mut old_wins: winlinks = std::ptr::replace(&mut (*s).windows, winlinks { storage: None });
    let mut old_lastw: winlink_stack;
    let mut new_idx: ::core::ffi::c_int = 0;
    let mut new_curw_idx: ::core::ffi::c_int = 0;
    let mut marked_idx: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    new_idx = options_get_number(
        options_owner_ptr(&mut (*s).options).map_or(std::ptr::null_mut(), |options| options),
        b"base-index\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    new_curw_idx = 0 as ::core::ffi::c_int;
    wl = winlinks_minmax(&old_wins, RB_NEGINF);
    while wl.is_alive() {
        wl_new = winlink_add(&raw mut (*s).windows, new_idx);
        wl_new.get_mut_unchecked().session = (*s).observer.clone();
        winlink_set_window((wl_new).clone(), &(*(wl.get_unchecked().window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()))).observer.upgrade().expect("live window"));
        wl_new.get_mut_unchecked().flags |= wl.get_unchecked().flags & WINLINK_ALERTFLAGS;
        if wl == marked_pane.winlink_handle() {
            marked_idx = wl_new.get_unchecked().idx;
        }
        if wl == (*s).current_winlink() {
            new_curw_idx = wl_new.get_unchecked().idx;
        }
        wl = winlinks_next(wl.get_unchecked());
        if wl.is_alive() {
            // The final window may use i32::MAX; no next index is needed.
            new_idx += 1;
        }
    }
    old_lastw = std::ptr::replace(
        &raw mut (*s).lastw,
        winlink_stack { storage: Default::default() },
    );
    for old_idx in crate::src::window::winlink_stack_indices(&old_lastw) {
        wl = winlink_find_by_index(&raw mut old_wins, old_idx);
        if !wl.is_alive() {
            continue;
        }
        wl.get_mut_unchecked().flags &= !WINLINK_VISITED;
        wl_new = winlink_find_by_window(&raw mut (*s).windows, &(*(wl.get_unchecked().window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()))).observer.upgrade().expect("live window"));
        if wl_new.is_alive() {
            crate::src::window::winlink_stack_append(&mut (*s).lastw, (wl_new).clone());
        }
    }
    crate::src::window::winlink_stack_clear(&mut old_lastw);
    if marked_idx != -(1 as ::core::ffi::c_int) {
        marked_pane.set_wl((winlink_find_by_index(&raw mut (*s).windows, marked_idx)).clone());
        if !marked_pane.winlink_handle().is_alive() {
            server_clear_marked();
        }
    }
    (*s).set_curw((winlink_find_by_index(&raw mut (*s).windows, new_curw_idx)).clone());
    wl = winlinks_minmax(&old_wins, RB_NEGINF);
    while wl.is_alive() && {
        wl1 = winlinks_next(wl.get_unchecked());
        1 as ::core::ffi::c_int != 0
    } {
        winlink_remove(&raw mut old_wins, wl.clone());
        wl = wl1;
    }
}
pub unsafe fn session_theme_changed(session: Option<&session>) {
    let Some(session) = session else { return; };
    let mut link = winlinks_minmax(&session.windows, RB_NEGINF);
    while link.is_alive() {
        let wl = link.get_unchecked();
        let mut next = window_pane_first(wl.window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()).as_ref());
        while let Some(owner) = next {
            let pane = &mut *owner.get();
            pane.flags |= PANE_THEMECHANGED;
            next = window_pane_next(Some(pane));
        }
        link = winlinks_next(wl);
    }
}
pub unsafe fn session_update_history(session: &session) {
    let limit = crate::src::options::options_get_number_ref(
        session.options.as_deref().expect("session options"), c"history-limit",
    ) as u_int;
    let mut link = winlinks_minmax(&session.windows, RB_NEGINF);
    while link.is_alive() {
        let wl = link.get_unchecked();
        let mut next = window_pane_first(wl.window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()).as_ref());
        while let Some(owner) = next {
            let pane = &mut *owner.get();
            let id = pane.id;
            let grid = pane.base.grid_mut();
            let old_size = grid.hsize;
            grid.hlimit = limit;
            grid_collect_history(grid, 1);
            if grid.hsize != old_size {
                log_debug(format_args!(
                    "session_update_history: %{} {} -> {}",
                    id, old_size, grid.hsize,
                ));
            }
            next = window_pane_next(Some(pane));
        }
        link = winlinks_next(wl);
    }
}

#[cfg(test)]
mod session_index_tests {
    use super::*;

    #[test]
    fn current_winlink_observer_expires_when_index_removes_it() {
        unsafe {
            let owner = session::new();
            let session = &mut *owner.get();
            let link = winlink_add(&raw mut session.windows, 4);
            session.set_curw(link.clone());
            assert_eq!(session.current_winlink(), link);
            winlink_remove(&raw mut session.windows, link.clone());
            assert!(!session.current_winlink().is_alive());
            assert!(!session.curw.is_empty());
            session.set_curw(refbox::Weak::new());
            assert!(session.curw.is_empty());
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

            let index_observer = (*second).entry.owner.clone();
            let duplicate = session_group::new(c"alpha");
            assert_eq!(session_groups_insert(&mut head, duplicate), first);
            assert!(!(*second).entry.owner.is_empty());
            assert!(!session_groups_remove(&mut other, second));
            assert!(!(*second).entry.owner.is_empty());

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

impl Drop for session {
    fn drop(&mut self) {
        unsafe { session_free(self) }
    }
}

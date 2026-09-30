mod alerts;
mod api;
pub(crate) use alerts::alerts_check_all;
#[cfg(test)]
pub(crate) use api::replace_test_environment;
mod model;
mod size;
mod sort;
mod spawn;
pub use model::session;
mod format;
use crate::src::session_group::session_group_remove;
pub use crate::src::session_group::{
    session_group_add, session_group_attached_count, session_group_count, session_group_find,
    session_group_for, session_group_members, session_group_new, session_group_synchronize_from,
    session_group_synchronize_to, session_groups, session_groups_find, session_groups_insert,
    session_groups_minmax, session_groups_next, session_groups_remove,
};
pub use api::Session;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::src::cmd::find::{cmd_find_from_session, cmd_find_from_winlink};
use crate::src::compat::strtonum::strtonum;
use crate::src::events::{events_fire, events_fire_session, events_fire_winlink};
use crate::src::events_payload::{
    event_payload_create, event_payload_set_int, event_payload_set_session,
    event_payload_set_string, event_payload_set_target, event_payload_set_uint,
    event_payload_set_window,
};
use crate::src::ffi::libc::{memcpy, strcmp};
use crate::src::format::bytes::write_cstr;
use crate::src::grid::grid_collect_history;
use crate::src::log::{fatal, fatalx, log_bytes, log_cstr, log_debug};
use crate::src::options::options_owner_ptr;
use crate::src::options::{options_free, options_get_number};
use crate::src::resize::recalculate_sizes;
use crate::src::server::{marked_pane, server_clear_marked};
use crate::src::server_fn::server_lock_session;
use crate::src::shared::abi::*;
use crate::src::shared::command::cmd_find_state;
use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
use crate::src::shared::events::event_payload;
use crate::src::shared::grid::*;
use crate::src::shared::limits::UINT_MAX;
use crate::src::shared::options::options;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::PANE_THEMECHANGED;
use crate::src::shared::session::{session_group, session_groups, sessions};
use crate::src::shared::session::{SessionRef, SessionWeak};
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::terminal::*;
use crate::src::shared::tree::{RB_INF, RB_NEGINF};
use crate::src::shared::window::WindowRef;
use crate::src::shared::window::{window, winlink, winlink_stack, winlinks};
use crate::src::shared::window::{WINLINK_ALERTFLAGS, WINLINK_VISITED};
use crate::src::tmux::global_options;
use crate::src::tty::tty_update_window_offset;
use crate::src::window::Window as _;
use crate::src::window::{
    window_pane_next, window_update_activity, window_update_focus, winlink_add,
    winlink_clear_flags, winlink_find_by_index, winlink_find_by_window, winlink_find_by_window_id,
    winlink_next, winlink_previous, winlink_remove, winlink_set_window, winlink_stack_push,
    winlink_stack_remove, winlinks_minmax, winlinks_next,
};
pub(crate) use size::recalculate_size_state;
use size::status_update_cache;
pub use sort::sort_get_sessions;
use std::cell::UnsafeCell;
use std::ffi::{CStr, CString};
use std::rc::Rc;
pub static mut sessions: sessions = sessions { storage: None };
pub static mut next_session_id: u_int = 0;

/// `session.cwd` borrows this value until replacement or early destruction.
fn session_set_cwd(s: &mut session, cwd: Option<CString>) {
    s.cwd = cwd;
}

/// Replace the borrowed public name after callers remove the old map key.
fn session_replace_name(s: &mut session, name: CString) -> CString {
    std::mem::replace(&mut s.name, name)
}
fn sessions_key(elm: &session) -> Vec<u8> {
    elm.name.as_bytes().to_vec()
}
fn sessions_find(head: &sessions, elm: &session) -> Option<SessionRef> {
    let owner = head.storage.as_ref()?;
    let map = owner
        .try_borrow_mut()
        .expect("session index already borrowed");
    map.get(elm.name.as_bytes()).cloned()
}
pub unsafe fn sessions_insert(head: &mut sessions, session: SessionRef) -> Option<SessionRef> {
    let elm = session.get();
    let key = (*elm).name.as_bytes();
    let owner = head.storage.get_or_insert_with(refbox::RefBox::default);
    let observer = owner.downgrade();
    let mut map = owner
        .try_borrow_mut()
        .expect("session index already borrowed");
    match map.entry(key.to_vec()) {
        std::collections::btree_map::Entry::Occupied(entry) => Some(entry.get().clone()),
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(session);
            (*elm).owner = observer;
            None
        }
    }
}
pub unsafe fn sessions_remove(head: &mut sessions, elm: &SessionRef) -> Option<SessionRef> {
    let key = (*elm.get()).name.as_bytes();
    let owner = head.storage.as_ref()?;
    let (session, empty) = {
        let mut map = owner
            .try_borrow_mut()
            .expect("session index already borrowed");
        if !map
            .get(key)
            .is_some_and(|candidate| Rc::ptr_eq(candidate, elm))
        {
            return None;
        }
        (map.remove(key).expect("matching session"), map.is_empty())
    };
    (*session.get()).owner = refbox::Weak::new();
    if empty {
        head.storage = None;
    }
    Some(session)
}
pub fn sessions_minmax(head: &sessions) -> Option<SessionRef> {
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
pub fn sessions_after(head: &sessions, name: &[u8]) -> Option<SessionRef> {
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
unsafe fn sessions_next(elm: &session) -> Option<SessionRef> {
    let owner = &elm.owner;
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
/// Resolve an observer only while its session remains in this index.
/// A removed session can still be retained by queued events or other owners.
pub fn sessions_resolve(head: &sessions, observer: &SessionWeak) -> Option<SessionRef> {
    let owner = observer.upgrade()?;
    let index = head.storage.as_ref()?;
    let map = index
        .try_borrow_mut()
        .expect("session index already borrowed");
    map.values()
        .any(|candidate| Rc::ptr_eq(candidate, &owner))
        .then_some(owner)
}

unsafe fn session_alive(s: Option<&session>) -> ::core::ffi::c_int {
    s.is_some_and(|s| sessions_resolve(&sessions, &s.observer).is_some()) as ::core::ffi::c_int
}
pub unsafe fn session_find(name: &CStr) -> Option<SessionRef> {
    let index = sessions.storage.as_ref()?;
    let map = index
        .try_borrow_mut()
        .expect("session index already borrowed");
    map.get(name.to_bytes()).cloned()
}
pub unsafe fn session_find_by_id_str(s: &CStr) -> Option<SessionRef> {
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
pub unsafe fn session_find_by_id(mut id: u_int) -> Option<SessionRef> {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut s_owner = sessions_minmax(&sessions);
    s = s_owner
        .as_ref()
        .map_or(std::ptr::null_mut(), |owner| owner.get());
    while !s.is_null() {
        if (*s).id == id {
            return (*s).observer.upgrade();
        }
        s_owner = sessions_next(&*s);
        s = s_owner
            .as_ref()
            .map_or(std::ptr::null_mut(), |owner| owner.get());
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
) -> SessionRef {
    let owner = session::new();
    let s = owner.get();
    (*s).tio = tio.copied().map(Box::new);
    (*s).cwd = Some(cwd.to_owned());

    (*s).flags = 0 as ::core::ffi::c_int;
    (*s).lastw.clear();
    (*s).windows = None;
    (*s).environ = Some(env);
    (*s).options = oo;
    status_update_cache(&mut *(s));
    if let Some(name) = name {
        drop(session_replace_name(&mut *s, name.to_owned()));
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
    (*s).creation_time = SystemTime::now();
    let created = (*s).creation_time;
    session_update_activity(&mut *s, Some(created));
    owner
}
unsafe fn session_add_ref(s: &session, from: *const ::core::ffi::c_char) -> SessionRef {
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
pub unsafe fn session_remove_ref(s: SessionRef, from: &CStr) {
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
    s_owner: &SessionRef,
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
    if (*s).lock_timer.is_initialized() {
        (*s).lock_timer.cancel();
    }
    session_group_remove(s_owner);
    while crate::src::window::winlink_stack_first(&(*s).lastw).is_alive() {
        let first = crate::src::window::winlink_stack_first(&(*s).lastw);
        winlink_stack_remove(&raw mut (*s).lastw, (first).clone());
    }
    crate::src::window::winlink_stack_clear(&mut (*s).lastw);
    while (*s).windows.is_some() {
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
unsafe fn session_lock_timer(owner: &SessionRef) {
    let session = &*owner.get();
    if session_alive(Some(session)) == 0 || session.attached == 0 {
        return;
    }
    log_debug(format_args!(
        "session {} locked, activity time {}",
        log_bytes(session.name.as_bytes()),
        crate::src::shared::time::unix_seconds(session.activity_time) as ::core::ffi::c_longlong
    ));
    server_lock_session(owner);
    recalculate_sizes();
}
unsafe fn session_update_activity(session: &mut session, from: Option<SystemTime>) {
    if let Some(from) = from {
        session.activity_time = from;
    } else {
        session.activity_time = SystemTime::now();
    }
    log_debug(format_args!(
        "session ${} {} activity {}.{:06}",
        session.id,
        log_bytes(session.name.as_bytes()),
        crate::src::shared::time::unix_seconds(session.activity_time) as ::core::ffi::c_longlong,
        session
            .activity_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_micros() as ::core::ffi::c_int
    ));
    if session.lock_timer.is_initialized() {
        session.lock_timer.cancel();
    } else {
        let observer = session.observer.clone();
        session.lock_timer.set(move || unsafe {
            if let Some(owner) = observer.upgrade() {
                session_lock_timer(&owner);
            }
        });
    }
    if session.attached != 0 {
        let timeout = Duration::from_secs(
            (crate::src::options::options_get_number_ref(
                session.options.as_deref().expect("session options"),
                c"lock-after-time",
            )) as u64,
        );
        if timeout.as_secs() != 0 {
            session.lock_timer.arm(timeout).expect("arm timer");
        }
    }
}
pub unsafe fn session_next_session(
    s: Option<&SessionRef>,
    sort_crit: &sort_criteria,
) -> Option<SessionRef> {
    session_adjacent(s, sort_crit, false)
}
pub unsafe fn session_previous_session(
    s: Option<&SessionRef>,
    sort_crit: &sort_criteria,
) -> Option<SessionRef> {
    session_adjacent(s, sort_crit, true)
}
unsafe fn session_adjacent(
    s: Option<&SessionRef>,
    sort_crit: &sort_criteria,
    previous: bool,
) -> Option<SessionRef> {
    let s = s?;
    let sorted = sort_get_sessions(sort_crit);
    let index = sorted.iter().position(|owner| Rc::ptr_eq(owner, s))?;
    let selected = if previous {
        if index == 0 {
            sorted.len() - 1
        } else {
            index - 1
        }
    } else {
        (index + 1) % sorted.len()
    };
    Some(sorted[selected].clone())
}
pub unsafe fn session_attach(
    s_owner: &SessionRef,
    window_owner: &WindowRef,
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
pub unsafe fn session_detach(
    s_owner: &SessionRef,
    mut wl: refbox::Weak<winlink>,
) -> ::core::ffi::c_int {
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
pub unsafe fn session_is_linked(s: Option<&SessionRef>, w: &WindowRef) -> ::core::ffi::c_int {
    let group = session_group_for(&s.map_or_else(std::rc::Weak::new, Rc::downgrade));
    let members = if group.is_null() {
        1
    } else {
        session_group_count(group) as usize
    };
    w.is_linked_outside_group(members) as ::core::ffi::c_int
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
    s_owner: &SessionRef,
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
    s_owner: &SessionRef,
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
    s_owner: &SessionRef,
    mut idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let s = s_owner.get();

    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    wl = winlink_find_by_index(&(*s).windows, idx);
    return session_set_current(s_owner, wl.clone());
}
pub unsafe fn session_last(s_owner: &SessionRef) -> ::core::ffi::c_int {
    let wl = s_owner.last_winlink();
    if !wl.is_alive() {
        return -(1 as ::core::ffi::c_int);
    }
    if wl == s_owner.current_winlink() {
        return 1 as ::core::ffi::c_int;
    }
    s_owner.select_winlink(wl)
}
unsafe fn session_fire_window_changed(
    s_owner: &SessionRef,
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
        std::rc::Rc::clone(&((wl.get_unchecked().window_handle().as_ref()).expect("live window"))),
    );
    event_payload_set_window(
        &mut *ep,
        b"new_window\0" as *const u8 as *const ::core::ffi::c_char,
        std::rc::Rc::clone(&((wl.get_unchecked().window_handle().as_ref()).expect("live window"))),
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
            std::rc::Rc::clone(
                &((old.get_unchecked().window_handle().as_ref()).expect("live window")),
            ),
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
pub unsafe fn session_set_current(
    s_owner: &SessionRef,
    mut wl: refbox::Weak<winlink>,
) -> ::core::ffi::c_int {
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
            window_update_focus(old.get_unchecked().window_handle().cloned().as_ref());
        }
        window_update_focus(wl.get_unchecked().window_handle().cloned().as_ref());
    }
    winlink_clear_flags(wl.clone());
    window_update_activity(&std::rc::Rc::clone(
        &((wl.get_unchecked().window_handle().as_ref()).expect("live window")),
    ));
    tty_update_window_offset(&std::rc::Rc::clone(
        &((wl.get_unchecked().window_handle().as_ref()).expect("live window")),
    ));
    session_fire_window_changed(s_owner, wl.clone(), (old).clone());
    return 0 as ::core::ffi::c_int;
}
unsafe fn session_group_contains(target: Option<&session>) -> *mut session_group {
    let Some(target) = target else {
        return std::ptr::null_mut();
    };
    session_group_for(&target.observer)
}

/// Rebuild Session-owned associations. Group membership and group traversal are
/// deliberately outside this operation. Each model loan ends before notification.
unsafe fn session_synchronize_windows(source: &SessionRef, destination: &SessionRef) {
    assert!(
        !Rc::ptr_eq(source, destination),
        "synchronized sessions must be distinct"
    );
    if source.with_winlinks(Option::is_none) {
        return;
    }
    let current = destination.current_winlink();
    if current.is_alive()
        && !source
            .with_winlinks(|links| winlink_find_by_index(links, current.get_unchecked().idx))
            .is_alive()
        && session_last(destination) != 0
        && session_previous(destination, 0) != 0
    {
        session_next(destination, 0);
    }

    // Retain the old index locally. Window cleanup may reenter either Session,
    // so it cannot run through a pointer into the destination's model storage.
    let mut old_windows = (&mut *destination.get()).windows.take();
    let mut link = source.with_winlinks(|links| winlinks_minmax(links, RB_NEGINF));
    while link.is_alive() {
        let index = link.get_unchecked().idx;
        let mut replacement = {
            let state = &mut *destination.get();
            winlink_add(&mut state.windows, index)
        };
        replacement.get_mut_unchecked().session = Rc::downgrade(destination);
        let window = link
            .get_unchecked()
            .window_handle()
            .expect("source window")
            .clone();
        winlink_set_window(replacement.clone(), &window);
        // Both source and newly published links retain this window now.
        drop(window);
        events_fire_winlink(c"window-linked".as_ptr(), replacement.clone());
        // Notification may change alerts or add a later source link. Observe
        // both at the original post-notification point in the live traversal.
        replacement.get_mut_unchecked().flags |= link.get_unchecked().flags & WINLINK_ALERTFLAGS;
        link = winlinks_next(link.get_unchecked());
    }

    let current = destination.current_winlink();
    if current.is_alive() {
        let index = current.get_unchecked().idx;
        let replacement = destination.with_winlinks(|links| winlink_find_by_index(links, index));
        (*destination.get()).curw = replacement;
    } else {
        let current = source.current_winlink();
        if current.is_alive() {
            let index = current.get_unchecked().idx;
            let replacement =
                destination.with_winlinks(|links| winlink_find_by_index(links, index));
            (*destination.get()).curw = replacement;
        }
    }
    if !destination.current_winlink().is_alive() {
        let first = destination.with_winlinks(|links| winlinks_minmax(links, RB_NEGINF));
        (*destination.get()).curw = first;
    }

    let mut old_history = std::mem::take(&mut (*destination.get()).lastw);
    for index in crate::src::window::winlink_stack_indices(&old_history) {
        let replacement = destination.with_winlinks(|links| winlink_find_by_index(links, index));
        if replacement.is_alive() {
            crate::src::window::winlink_stack_append(&mut (*destination.get()).lastw, replacement);
        }
    }
    crate::src::window::winlink_stack_clear(&mut old_history);
    while old_windows.is_some() {
        let old = winlinks_minmax(&old_windows, RB_NEGINF);
        let id = old
            .get_unchecked()
            .window_handle()
            .expect("old window")
            .id();
        let replacement = destination.with_winlinks(|links| winlink_find_by_window_id(links, id));
        if !replacement.is_alive() {
            events_fire_winlink(c"window-unlinked".as_ptr(), old.clone());
        }
        winlink_remove(&mut old_windows, old);
    }
}

pub unsafe fn session_renumber_windows(s_owner: &SessionRef) {
    let s = s_owner.get();

    let mut wl: refbox::Weak<winlink> = refbox::Weak::new();
    let mut wl1: refbox::Weak<winlink> = refbox::Weak::new();
    let mut wl_new: refbox::Weak<winlink> = refbox::Weak::new();
    let mut old_wins: winlinks = std::ptr::replace(&mut (*s).windows, None);
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
        winlink_set_window(
            (wl_new).clone(),
            &std::rc::Rc::clone(
                &((wl.get_unchecked().window_handle().as_ref()).expect("live window")),
            ),
        );
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
    old_lastw = std::ptr::replace(&raw mut (*s).lastw, Default::default());
    for old_idx in crate::src::window::winlink_stack_indices(&old_lastw) {
        wl = winlink_find_by_index(&old_wins, old_idx);
        if !wl.is_alive() {
            continue;
        }
        wl.get_mut_unchecked().flags &= !WINLINK_VISITED;
        wl_new = winlink_find_by_window(
            &(*s).windows,
            &std::rc::Rc::clone(
                &((wl.get_unchecked().window_handle().as_ref()).expect("live window")),
            ),
        );
        if wl_new.is_alive() {
            crate::src::window::winlink_stack_append(&mut (*s).lastw, (wl_new).clone());
        }
    }
    crate::src::window::winlink_stack_clear(&mut old_lastw);
    if marked_idx != -(1 as ::core::ffi::c_int) {
        marked_pane.set_wl((winlink_find_by_index(&(*s).windows, marked_idx)).clone());
        if !marked_pane.winlink_handle().is_alive() {
            server_clear_marked();
        }
    }
    (*s).set_curw((winlink_find_by_index(&(*s).windows, new_curw_idx)).clone());
    wl = winlinks_minmax(&old_wins, RB_NEGINF);
    while wl.is_alive() && {
        wl1 = winlinks_next(wl.get_unchecked());
        1 as ::core::ffi::c_int != 0
    } {
        winlink_remove(&raw mut old_wins, wl.clone());
        wl = wl1;
    }
}
unsafe fn session_theme_changed(session: Option<&session>) {
    let Some(session) = session else {
        return;
    };
    let mut link = winlinks_minmax(&session.windows, RB_NEGINF);
    while link.is_alive() {
        let wl = link.get_unchecked();
        let mut next = wl.window_handle().and_then(|window| window.next_pane(None));
        while let Some(owner) = next {
            let pane = &mut *owner.get();
            pane.flags |= PANE_THEMECHANGED;
            next = window_pane_next(Some(pane));
        }
        link = winlinks_next(wl);
    }
}
unsafe fn session_update_history(session: &session) {
    let limit = crate::src::options::options_get_number_ref(
        session.options.as_deref().expect("session options"),
        c"history-limit",
    ) as u_int;
    let mut link = winlinks_minmax(&session.windows, RB_NEGINF);
    while link.is_alive() {
        let wl = link.get_unchecked();
        let mut next = wl.window_handle().and_then(|window| window.next_pane(None));
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

            let index_observer = (*second).owner.clone();
            let duplicate = session_group::new(c"alpha");
            assert_eq!(session_groups_insert(&mut head, duplicate), first);
            assert!(!(*second).owner.is_empty());
            assert!(!session_groups_remove(&mut other, second));
            assert!(!(*second).owner.is_empty());

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

#[cfg(test)]
mod sorted_session_owners_tests;

#[cfg(test)]
mod winlink_storage_tests;

#[cfg(test)]
mod session_lastw_queue_tests;

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod storage_tests;

#[cfg(test)]
mod group_synchronization_tests {
    use super::*;
    use crate::src::events::{events_add_sink, events_remove_sink};
    use crate::src::shared::events::events_callback;
    use crate::src::shared::window::{WINLINK_ACTIVITY, WINLINK_BELL, WINLINK_SILENCE};
    use std::cell::RefCell;

    unsafe fn link(session: &SessionRef, index: i32, window: &WindowRef) -> refbox::Weak<winlink> {
        let link = test_support::add_link(session, index);
        winlink_set_window(link.clone(), window);
        link
    }

    unsafe fn indexes(session: &SessionRef) -> Vec<i32> {
        session.with_winlinks(|links| {
            let mut result = Vec::new();
            let mut link = winlinks_minmax(links, RB_NEGINF);
            while link.is_alive() {
                result.push(link.get_unchecked().idx);
                link = winlinks_next(link.get_unchecked());
            }
            result
        })
    }

    unsafe fn clean(session: SessionRef) {
        (*session.get()).curw = refbox::Weak::new();
        (*session.get()).lastw.clear();
        let mut links = (*session.get()).windows.take();
        while links.is_some() {
            let link = winlinks_minmax(&links, RB_NEGINF);
            winlink_remove(&mut links, link);
        }
        drop(session);
    }

    #[test]
    fn synchronization_keeps_live_source_order_and_publishes_history_before_unlink() {
        unsafe {
            let source = session::with_options_for_test(crate::src::options::options_create(None));
            let destination =
                session::with_options_for_test(crate::src::options::options_create(None));
            let windows = (1..=4).map(window::with_id_for_test).collect::<Vec<_>>();
            let mut first = link(&source, 2, &windows[0]);
            let mut second = link(&source, 7, &windows[1]);
            first.get_mut_unchecked().flags |= WINLINK_BELL;
            second.get_mut_unchecked().flags |= WINLINK_ACTIVITY;
            (*source.get()).curw = second.clone();
            let old_current = link(&destination, 2, &windows[0]);
            let old_last = link(&destination, 9, &windows[2]);
            (*destination.get()).curw = old_current.clone();
            winlink_stack_push(&mut (*destination.get()).lastw, old_current.clone());
            winlink_stack_push(&mut (*destination.get()).lastw, old_last.clone());

            let order = Rc::new(RefCell::new(Vec::new()));
            let observed = order.clone();
            let source_for_callback = source.clone();
            let destination_for_callback = destination.clone();
            let old_current_for_callback = old_current.clone();
            let old_last_for_callback = old_last.clone();
            let inserted_window = windows[3].clone();
            let linked = events_add_sink(
                c"window-linked",
                events_callback(move |_, payload| {
                    let index = payload.target.idx;
                    observed.borrow_mut().push(("linked", index));
                    let source = &source_for_callback;
                    let destination = &destination_for_callback;
                    assert_eq!(destination.current_winlink(), old_current_for_callback);
                    assert_eq!(destination.last_winlink(), old_last_for_callback);
                    assert_eq!(
                        indexes(destination),
                        match index {
                            2 => vec![2],
                            5 => vec![2, 5],
                            7 => vec![2, 5, 7],
                            _ => unreachable!(),
                        }
                    );
                    let replacement =
                        destination.with_winlinks(|links| winlink_find_by_index(links, index));
                    assert_eq!(
                        replacement.get_unchecked().flags & WINLINK_ALERTFLAGS,
                        0,
                        "source alerts are copied after linked callbacks"
                    );
                    // Reenter component operations in both Sessions. The operation
                    // retains owners and old links, never a Session component loan.
                    source.with_options_mut(|table| {
                        assert!(table.parent.is_none());
                    });
                    destination.with_options_mut(|table| {
                        assert!(table.parent.is_none());
                    });
                    if index == 2 {
                        let mut original =
                            source.with_winlinks(|links| winlink_find_by_index(links, 2));
                        original.get_mut_unchecked().flags = WINLINK_SILENCE;
                        link(source, 5, &inserted_window);
                    }
                }),
            );
            let observed = order.clone();
            let source_for_callback = source.clone();
            let destination_for_callback = destination.clone();
            let old_current_for_callback = old_current.clone();
            let old_last_for_callback = old_last.clone();
            let unlinked = events_add_sink(
                c"window-unlinked",
                events_callback(move |_, payload| {
                    observed.borrow_mut().push(("unlinked", payload.target.idx));
                    assert_eq!(
                        payload.target.idx, 9,
                        "shared old windows have no unlink event"
                    );
                    let current = destination_for_callback.current_winlink();
                    assert_eq!(current.get_unchecked().idx, 2);
                    assert_ne!(current, old_current_for_callback);
                    assert_eq!(destination_for_callback.last_winlink(), current);
                    assert!(
                        !old_current_for_callback.is_alive(),
                        "shared old link was released first"
                    );
                    assert!(
                        old_last_for_callback.is_alive(),
                        "unlink notification precedes old-link release"
                    );
                    assert_eq!(indexes(&source_for_callback), [2, 5, 7]);
                    assert_eq!(indexes(&destination_for_callback), [2, 5, 7]);
                }),
            );

            destination.synchronize_windows_from(&source);
            events_remove_sink(linked);
            events_remove_sink(unlinked);
            assert_eq!(
                *order.borrow(),
                [("linked", 2), ("linked", 5), ("linked", 7), ("unlinked", 9)]
            );
            assert!(!old_current.is_alive());
            assert!(!old_last.is_alive());
            assert_eq!(
                destination.current_winlink().get_unchecked().flags & WINLINK_ALERTFLAGS,
                WINLINK_SILENCE
            );
            assert_eq!(
                crate::src::window::winlink_stack_indices(&(*destination.get()).lastw),
                [2]
            );
            assert_eq!(source.current_winlink(), second);
            clean(source);
            clean(destination);
            for window in windows {
                window.release(c"group synchronization test");
            }
            crate::src::reactor::shutdown_runtime();
        }
    }
}

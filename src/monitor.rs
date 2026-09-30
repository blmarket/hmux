use crate::src::ffi::libc::sscanf;
use crate::src::format::{
    format_create, format_defaults, format_expand_cstring, format_free, format_true,
};
use crate::src::log::{log_cstr, log_debug};
use crate::src::reactor::Timer;
use crate::src::server::current_time;
use crate::src::server_client::Client as _;
#[cfg(test)]
use crate::src::session::SessionFixture as _;
use crate::src::session::SessionIndex as _;
use crate::src::session::{sessions, Session as _};
use crate::src::shared::abi::*;
use crate::src::shared::client::{ClientRef, ClientWeak};
use crate::src::shared::format::{format_tree, FORMAT_NOJOBS};
use crate::src::shared::monitor::{
    monitor_cb, monitor_change, monitor_item, monitor_items, monitor_pane, monitor_panes,
    monitor_window, monitor_windows,
};
pub use crate::src::shared::monitor::{
    monitor_type, MONITOR_ALL_PANES, MONITOR_ALL_WINDOWS, MONITOR_NOTIFY_INITIAL,
    MONITOR_NOTIFY_TRUE, MONITOR_PANE, MONITOR_SESSION, MONITOR_WINDOW,
};
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::session::SessionRef;
use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::window::winlink;
use crate::src::window::Window as _;
use crate::src::window::WindowIndex as _;
use crate::src::window::{winlinks_minmax, winlinks_next, Window as _};
use crate::src::window_pane::WindowPane as _;
use std::cell::UnsafeCell;
use std::ffi::{CStr, CString};
use std::rc::{Rc, Weak};
use std::time::Duration;

/// Owned by one control client or hook, which calls monitor_destroy on removal.
/// Operations and timers use weak observers and do not prolong this ownership.
pub struct monitor_set {
    client: ClientWeak,
    session: Option<SessionRef>,
    callback: Option<monitor_cb>,
    items: monitor_items,
    timer: Option<Timer>,
    generation: u_int,
    next_item_identity: u64,
}

// Release this borrow before model queries, formatting, callbacks or Session
// release. A callback may destroy the sole owner; the next borrow then expires.
fn monitor_borrow(observer: &refbox::Weak<monitor_set>) -> Option<refbox::Borrow<'_, monitor_set>> {
    match observer.try_borrow_mut() {
        Ok(state) => Some(state),
        Err(refbox::BorrowError::Dropped) => None,
        Err(refbox::BorrowError::Borrowed) => panic!("monitor state already borrowed"),
    }
}

fn monitor_first_item(observer: &refbox::Weak<monitor_set>) -> Option<MonitorItemIdentity> {
    monitor_borrow(observer)?
        .items
        .first_key_value()
        .map(|(_, item)| MonitorItemIdentity::of(item))
}
fn monitor_next_item(
    observer: &refbox::Weak<monitor_set>,
    after: &CStr,
) -> Option<MonitorItemIdentity> {
    monitor_borrow(observer)?
        .items
        .range((
            std::ops::Bound::Excluded(after.to_bytes().to_vec()),
            std::ops::Bound::Unbounded,
        ))
        .next()
        .map(|(_, item)| MonitorItemIdentity::of(item))
}
fn monitor_with_item<R>(
    observer: &refbox::Weak<monitor_set>,
    id: &MonitorItemIdentity,
    operation: impl FnOnce(&mut monitor_item) -> R,
) -> Option<R> {
    let mut state = monitor_borrow(observer)?;
    let item = state.items.get_mut(id.name.to_bytes())?;
    (item.identity == id.identity).then(|| operation(item))
}
unsafe fn monitor_expand(
    monitor: &refbox::Weak<monitor_set>,
    identity: &MonitorItemIdentity,
    formats: &mut format_tree,
) -> Option<CString> {
    let expression = monitor_with_item(monitor, identity, |item| item.format.clone())?;
    Some(format_expand_cstring(formats, expression.as_ptr()))
}
fn monitor_next_generation(observer: &refbox::Weak<monitor_set>) -> Option<u32> {
    let mut state = monitor_borrow(observer)?;
    state.generation = state.generation.wrapping_add(1).max(1);
    Some(state.generation)
}

#[derive(Clone)]
struct MonitorItemIdentity {
    name: CString,
    identity: u64,
}
impl MonitorItemIdentity {
    fn of(item: &monitor_item) -> Self {
        Self {
            name: item.name.clone(),
            identity: item.identity,
        }
    }
}
// Preserve the original one-successor-ahead traversal: additions before the
// prefetched successor wait for a later pass; later additions can be observed.
fn monitor_visit(
    monitor: &refbox::Weak<monitor_set>,
    mut visit: impl FnMut(MonitorItemIdentity, monitor_type),
) {
    let mut cursor = monitor_first_item(monitor);
    while let Some(identity) = cursor {
        if !monitor.is_alive() {
            break;
        }
        let next = monitor_next_item(monitor, &identity.name);
        if let Some(kind) = monitor_with_item(monitor, &identity, |item| item.type_0) {
            visit(identity, kind);
        }
        cursor = next;
    }
}

fn monitor_pane_new() -> Box<monitor_pane> {
    Box::new(monitor_pane::empty())
}
fn monitor_window_new() -> Box<monitor_window> {
    Box::new(monitor_window::empty())
}
unsafe fn monitor_has_client(monitor: &refbox::Weak<monitor_set>) -> bool {
    monitor_borrow(monitor).is_some_and(|state| !state.client.ptr_eq(&Weak::new()))
}
unsafe fn monitor_client(monitor: &refbox::Weak<monitor_set>) -> Option<ClientRef> {
    let observer = monitor_borrow(monitor)?.client.clone();
    let client = observer.upgrade()?;
    (!client.is_dead()).then_some(client)
}
unsafe fn monitor_get_session(
    monitor: &refbox::Weak<monitor_set>,
    client: Option<&ClientRef>,
) -> Option<SessionRef> {
    let (has_client, session) = {
        let state = monitor_borrow(monitor)?;
        (!state.client.ptr_eq(&Weak::new()), state.session.clone())
    };
    if has_client {
        return client?.attached_session().upgrade();
    }
    let Some(session) = session else {
        return sessions.first();
    };
    let indexed = crate::src::shared::session::SessionRef::find_by_id(session.id())?;
    Rc::ptr_eq(&session, &indexed).then_some(indexed)
}
unsafe fn monitor_context(
    monitor: &refbox::Weak<monitor_set>,
) -> Option<(Option<ClientRef>, SessionRef)> {
    if !monitor.is_alive() {
        return None;
    }
    let client = monitor_client(monitor);
    if monitor_has_client(monitor) && client.is_none() {
        return None;
    }
    let session = monitor_get_session(monitor, client.as_ref())?;
    Some((client, session))
}
unsafe fn monitor_create_formats(
    client: Option<&ClientRef>,
    session: Option<&SessionRef>,
    link: refbox::Weak<winlink>,
    pane: Option<&Rc<UnsafeCell<window_pane>>>,
) -> Box<format_tree> {
    let mut formats = format_create(None, None, 0, FORMAT_NOJOBS);
    format_defaults(&mut *formats, client, session, link, pane);
    formats
}

// No externally callable function lends an item, child record or last-value
// field. These keys select the destination only during the commit below.
#[derive(Clone, Copy)]
enum MonitorValueTarget {
    Session,
    Pane(u32, u32),
    Window(u32, u32),
}
unsafe fn monitor_check_value(
    monitor: &refbox::Weak<monitor_set>,
    identity: &MonitorItemIdentity,
    session: Option<&SessionRef>,
    link: refbox::Weak<winlink>,
    pane: Option<&Rc<UnsafeCell<window_pane>>>,
    value: &CStr,
    target: MonitorValueTarget,
    generation: Option<u32>,
) {
    let truth = format_true(value.as_ptr()) != 0;
    // Formatting may reenter a scan. Preserve the live generation used by the
    // original record commit, rather than the generation from before expansion.
    let Some(state) = monitor_borrow(monitor) else {
        return;
    };
    let generation = generation.map(|_| state.generation);
    drop(state);
    let record = monitor_with_item(monitor, identity, |item| {
        let last = match target {
            MonitorValueTarget::Session => &mut item.last,
            MonitorValueTarget::Pane(pane, index) => {
                let find = monitor_pane {
                    pane,
                    idx: index,
                    ..monitor_pane::empty()
                };
                let mut record = monitor_panes_find(&mut item.panes, &find);
                if record.is_null() {
                    assert!(monitor_panes_insert(&mut item.panes, Box::new(find)).is_ok());
                    record = monitor_panes_find(
                        &mut item.panes,
                        &monitor_pane {
                            pane,
                            idx: index,
                            ..monitor_pane::empty()
                        },
                    );
                }
                if let Some(generation) = generation {
                    (*record).generation = generation;
                }
                &mut (*record).last
            }
            MonitorValueTarget::Window(window, index) => {
                let find = monitor_window {
                    window,
                    idx: index,
                    ..monitor_window::empty()
                };
                let mut record = monitor_windows_find(&mut item.windows, &find);
                if record.is_null() {
                    assert!(monitor_windows_insert(&mut item.windows, Box::new(find)).is_ok());
                    record = monitor_windows_find(
                        &mut item.windows,
                        &monitor_window {
                            window,
                            idx: index,
                            ..monitor_window::empty()
                        },
                    );
                }
                if let Some(generation) = generation {
                    (*record).generation = generation;
                }
                &mut (*record).last
            }
        };
        if last.as_deref() == Some(value) {
            return None;
        }
        let previous = last.replace(value.to_owned());
        if (previous.is_none() && item.flags & MONITOR_NOTIFY_INITIAL == 0)
            || (item.flags & MONITOR_NOTIFY_TRUE != 0 && !truth)
        {
            return None;
        }
        item.fire_count = item.fire_count.wrapping_add(1);
        item.fire_time = current_time;
        Some((item.name.clone(), previous))
    })
    .flatten();
    let Some((name, previous)) = record else {
        return;
    };
    let (callback, client) = {
        let Some(state) = monitor_borrow(monitor) else {
            return;
        };
        (
            state
                .callback
                .as_ref()
                .expect("live monitor callback")
                .clone(),
            state.client.clone(),
        )
    };
    log_debug(format_args!(
        "monitor_report: {} changed to {}",
        log_cstr(name.as_ptr()),
        log_cstr(value.as_ptr())
    ));
    let change = monitor_change {
        name: &name,
        value,
        last: previous.as_deref(),
        c: client,
        s: session.map_or_else(Weak::new, Rc::downgrade),
        wl: link,
        wp: pane.map_or_else(Weak::new, Rc::downgrade),
    };
    callback(&change);
}
unsafe fn monitor_check_session(
    monitor: &refbox::Weak<monitor_set>,
    identity: &MonitorItemIdentity,
    formats: &mut format_tree,
) {
    let Some((_client, session)) = monitor_context(monitor) else {
        return;
    };
    let Some(value) = monitor_expand(monitor, identity, formats) else {
        return;
    };
    monitor_check_value(
        monitor,
        identity,
        Some(&session),
        refbox::Weak::new(),
        None,
        &value,
        MonitorValueTarget::Session,
        None,
    );
}
unsafe fn monitor_check_pane(monitor: &refbox::Weak<monitor_set>, identity: &MonitorItemIdentity) {
    let Some(target) = monitor_with_item(monitor, identity, |item| item.id) else {
        return;
    };
    let Some((client, session)) = monitor_context(monitor) else {
        return;
    };
    let Some(pane) = Rc::<UnsafeCell<window_pane>>::find_by_id(target) else {
        return;
    };
    if !pane.has_tty() {
        return;
    }
    let Some(window) = pane.window_observer().upgrade() else {
        return;
    };
    let mut link = window.next_winlink(None);
    while link.is_alive() && monitor_with_item(monitor, identity, |_| ()).is_some() {
        let matches = link
            .get_unchecked()
            .session
            .ptr_eq(&Rc::downgrade(&session));
        if matches {
            let mut formats =
                monitor_create_formats(client.as_ref(), Some(&session), link.clone(), Some(&pane));
            let value = monitor_expand(monitor, identity, &mut *formats);
            format_free(formats);
            let Some(value) = value else { break };
            if link.is_alive() {
                let index = link.get_unchecked().idx as u32;
                monitor_check_value(
                    monitor,
                    identity,
                    Some(&session),
                    link.clone(),
                    Some(&pane),
                    &value,
                    MonitorValueTarget::Pane(pane.id(), index),
                    None,
                );
            }
        }
        // A removed link has no valid successor in the old list. The next timer
        // pass will rescan; never recover one through a freed winlink reference.
        if !link.is_alive() {
            break;
        }
        link = window.next_winlink(Some(link));
    }
    window.release(c"monitor_check_pane");
}
unsafe fn monitor_check_window(
    monitor: &refbox::Weak<monitor_set>,
    identity: &MonitorItemIdentity,
) {
    let Some(target) = monitor_with_item(monitor, identity, |item| item.id) else {
        return;
    };
    let Some((client, session)) = monitor_context(monitor) else {
        return;
    };
    let Some(window) = crate::src::shared::window::WindowRef::find_by_id(target) else {
        return;
    };
    let mut link = window.next_winlink(None);
    while link.is_alive() && monitor_with_item(monitor, identity, |_| ()).is_some() {
        let matches = link
            .get_unchecked()
            .session
            .ptr_eq(&Rc::downgrade(&session));
        if matches {
            let mut formats =
                monitor_create_formats(client.as_ref(), Some(&session), link.clone(), None);
            let value = monitor_expand(monitor, identity, &mut *formats);
            format_free(formats);
            let Some(value) = value else { break };
            if link.is_alive() {
                let index = link.get_unchecked().idx as u32;
                monitor_check_value(
                    monitor,
                    identity,
                    Some(&session),
                    link.clone(),
                    None,
                    &value,
                    MonitorValueTarget::Window(window.id(), index),
                    None,
                );
            }
        }
        if !link.is_alive() {
            break;
        }
        link = window.next_winlink(Some(link));
    }
    window.release(c"monitor_check_window");
}
unsafe fn monitor_check_all_panes_one(
    monitor: &refbox::Weak<monitor_set>,
    identity: &MonitorItemIdentity,
    formats: &mut format_tree,
    link: refbox::Weak<winlink>,
    pane: &Rc<UnsafeCell<window_pane>>,
    generation: u32,
) {
    let Some((_client, session)) = monitor_context(monitor) else {
        return;
    };
    let Some(value) = monitor_expand(monitor, identity, formats) else {
        return;
    };
    if !link.is_alive() {
        return;
    }
    let index = link.get_unchecked().idx as u32;
    monitor_check_value(
        monitor,
        identity,
        Some(&session),
        link,
        Some(pane),
        &value,
        MonitorValueTarget::Pane(pane.id(), index),
        Some(generation),
    );
}
unsafe fn monitor_check_all_windows_one(
    monitor: &refbox::Weak<monitor_set>,
    identity: &MonitorItemIdentity,
    formats: &mut format_tree,
    link: refbox::Weak<winlink>,
    generation: u32,
) {
    let Some((_client, session)) = monitor_context(monitor) else {
        return;
    };
    let Some(value) = monitor_expand(monitor, identity, formats) else {
        return;
    };
    if !link.is_alive() {
        return;
    }
    let (index, window) = {
        let target = link.get_unchecked();
        (
            target.idx as u32,
            target.window_handle().expect("monitor link window").id(),
        )
    };
    monitor_check_value(
        monitor,
        identity,
        Some(&session),
        link,
        None,
        &value,
        MonitorValueTarget::Window(window, index),
        Some(generation),
    );
}
unsafe fn monitor_sweep_all_panes(item: &mut monitor_item, generation: u32) {
    let mut record = monitor_panes_minmax(&mut item.panes);
    while !record.is_null() {
        let next = monitor_panes_next(&mut item.panes, record);
        if (*record).generation != generation {
            drop(monitor_panes_remove(&mut item.panes, record));
        }
        record = next;
    }
}
unsafe fn monitor_sweep_all_windows(item: &mut monitor_item, generation: u32) {
    let mut record = monitor_windows_minmax(&mut item.windows);
    while !record.is_null() {
        let next = monitor_windows_next(&mut item.windows, record);
        if (*record).generation != generation {
            drop(monitor_windows_remove(&mut item.windows, record));
        }
        record = next;
    }
}
unsafe fn monitor_check_sessions(monitor: &refbox::Weak<monitor_set>) {
    let Some((client, session)) = monitor_context(monitor) else {
        return;
    };
    let mut formats =
        monitor_create_formats(client.as_ref(), Some(&session), refbox::Weak::new(), None);
    monitor_visit(monitor, |identity, kind| {
        if kind == MONITOR_SESSION {
            monitor_check_session(monitor, &identity, &mut *formats);
        }
    });
    format_free(formats);
}
unsafe fn monitor_check_panes_windows(monitor: &refbox::Weak<monitor_set>) {
    monitor_visit(monitor, |identity, kind| match kind {
        MONITOR_PANE => monitor_check_pane(monitor, &identity),
        MONITOR_WINDOW => monitor_check_window(monitor, &identity),
        _ => {}
    });
}
unsafe fn monitor_check_all_panes(monitor: &refbox::Weak<monitor_set>) {
    let Some((client, session)) = monitor_context(monitor) else {
        return;
    };
    let Some(generation) = monitor_next_generation(monitor) else {
        return;
    };
    let mut link = session.with_winlinks(|links| winlinks_minmax(links, RB_NEGINF));
    while link.is_alive() && monitor.is_alive() {
        let window = link.get_unchecked().window_handle().cloned();
        let mut cursor = window.as_ref().and_then(|window| window.next_pane(None));
        while let Some(pane) = cursor {
            if !link.is_alive() || !monitor.is_alive() {
                break;
            }
            let mut formats =
                monitor_create_formats(client.as_ref(), Some(&session), link.clone(), Some(&pane));
            monitor_visit(monitor, |identity, kind| {
                if kind == MONITOR_ALL_PANES && link.is_alive() {
                    monitor_check_all_panes_one(
                        monitor,
                        &identity,
                        &mut *formats,
                        link.clone(),
                        &pane,
                        generation,
                    );
                }
            });
            format_free(formats);
            // Keep the legacy live parent lookup after callbacks that may move a pane.
            let next_window = pane.window_observer().upgrade();
            cursor = next_window
                .as_ref()
                .and_then(|window| window.next_pane(Some(&pane)));
            if let Some(window) = next_window {
                window.release(c"monitor pane successor");
            }
        }
        if let Some(window) = window {
            window.release(c"monitor all panes window");
        }
        if !link.is_alive() {
            break;
        }
        link = winlinks_next(link.get_unchecked());
    }
    monitor_visit(monitor, |identity, kind| {
        if kind == MONITOR_ALL_PANES {
            let generation = monitor_borrow(monitor)
                .expect("live monitor traversal")
                .generation;
            monitor_with_item(monitor, &identity, |item| {
                monitor_sweep_all_panes(item, generation)
            });
        }
    });
}
unsafe fn monitor_check_all_windows(monitor: &refbox::Weak<monitor_set>) {
    let Some((client, session)) = monitor_context(monitor) else {
        return;
    };
    let Some(generation) = monitor_next_generation(monitor) else {
        return;
    };
    let mut link = session.with_winlinks(|links| winlinks_minmax(links, RB_NEGINF));
    while link.is_alive() && monitor.is_alive() {
        let mut formats =
            monitor_create_formats(client.as_ref(), Some(&session), link.clone(), None);
        monitor_visit(monitor, |identity, kind| {
            if kind == MONITOR_ALL_WINDOWS && link.is_alive() {
                monitor_check_all_windows_one(
                    monitor,
                    &identity,
                    &mut *formats,
                    link.clone(),
                    generation,
                );
            }
        });
        format_free(formats);
        if !link.is_alive() {
            break;
        }
        link = winlinks_next(link.get_unchecked());
    }
    monitor_visit(monitor, |identity, kind| {
        if kind == MONITOR_ALL_WINDOWS {
            let generation = monitor_borrow(monitor)
                .expect("live monitor traversal")
                .generation;
            monitor_with_item(monitor, &identity, |item| {
                monitor_sweep_all_windows(item, generation)
            });
        }
    });
}
unsafe fn monitor_timer(monitor: &refbox::Weak<monitor_set>) {
    let client = monitor_client(monitor);
    if monitor_has_client(monitor) && client.is_none() {
        return;
    }
    if !monitor.is_alive() {
        return;
    }
    log_debug(format_args!("monitor_timer: timer fired"));
    {
        let Some(mut state) = monitor_borrow(monitor) else {
            return;
        };
        let timeout = Duration::from_secs(1);
        let observer = monitor.clone();
        state.timer = Some(
            Timer::new(timeout, move || {
                monitor_timer(&observer);
            })
            .expect("arm timer"),
        );
    }
    let Some(_session) = monitor_get_session(monitor, client.as_ref()) else {
        return;
    };
    let mut have_session = false;
    let mut have_all_panes = false;
    let mut have_all_windows = false;
    monitor_visit(monitor, |_, kind| match kind {
        MONITOR_SESSION => have_session = true,
        MONITOR_ALL_PANES => have_all_panes = true,
        MONITOR_ALL_WINDOWS => have_all_windows = true,
        _ => {}
    });
    if have_session {
        monitor_check_sessions(monitor);
    }
    if !monitor.is_alive() {
        return;
    }
    monitor_check_panes_windows(monitor);
    if !monitor.is_alive() {
        return;
    }
    if have_all_panes {
        monitor_check_all_panes(monitor);
    }
    if !monitor.is_alive() {
        return;
    }
    if have_all_windows {
        monitor_check_all_windows(monitor);
    }
}
fn monitor_create(callback: monitor_cb) -> refbox::RefBox<monitor_set> {
    refbox::RefBox::new(monitor_set {
        client: Weak::new(),
        session: None,
        callback: Some(callback),
        items: Default::default(),
        timer: None,
        generation: 0,
        next_item_identity: 1,
    })
}
pub unsafe fn monitor_create_client(
    client: Option<&ClientRef>,
    callback: monitor_cb,
) -> refbox::RefBox<monitor_set> {
    let owner = monitor_create(callback);
    owner.try_borrow_mut().expect("new monitor").client =
        client.map_or_else(Weak::new, Rc::downgrade);
    owner
}
pub unsafe fn monitor_create_session(
    session: Option<&SessionRef>,
    callback: monitor_cb,
) -> refbox::RefBox<monitor_set> {
    let owner = monitor_create(callback);
    owner.try_borrow_mut().expect("new monitor").session = session.cloned();
    owner
}
unsafe fn monitor_free_item(state: &mut monitor_set, item: *mut monitor_item) {
    let mut pane = monitor_panes_minmax(&mut (*item).panes);
    while !pane.is_null() {
        let next = monitor_panes_next(&mut (*item).panes, pane);
        drop(monitor_panes_remove(&mut (*item).panes, pane));
        pane = next;
    }
    let mut window = monitor_windows_minmax(&mut (*item).windows);
    while !window.is_null() {
        let next = monitor_windows_next(&mut (*item).windows, window);
        drop(monitor_windows_remove(&mut (*item).windows, window));
        window = next;
    }
    drop(monitor_items_remove(&mut state.items, item).expect("indexed monitor record"));
}
pub unsafe fn monitor_destroy(owner: refbox::RefBox<monitor_set>) {
    let (session, callback) = {
        let mut state = owner
            .try_borrow_mut()
            .expect("monitor destruction outside a borrow");
        drop(state.timer.take());
        let mut item = monitor_items_minmax(&mut state.items);
        while !item.is_null() {
            let next = monitor_items_next(&mut state.items, item);
            monitor_free_item(&mut state, item);
            item = next;
        }
        (state.session.take(), state.callback.take())
    };
    // Expire every observer before releasing resources that may reenter.
    drop(owner);
    if let Some(session) = session {
        session.release(c"monitor_clear");
    }
    drop(callback);
}
pub unsafe fn monitor_create_client_owned(
    client: Option<&ClientRef>,
    callback: monitor_cb,
) -> refbox::RefBox<monitor_set> {
    monitor_create_client(client, callback)
}
pub unsafe fn monitor_create_session_owned(
    session: Option<&SessionRef>,
    callback: monitor_cb,
) -> refbox::RefBox<monitor_set> {
    monitor_create_session(session, callback)
}

pub struct ParsedMonitor {
    pub name: CString,
    pub type_0: monitor_type,
    pub id: ::core::ffi::c_int,
    pub format: CString,
}

unsafe fn monitor_parse_parts(value: &CStr) -> Option<ParsedMonitor> {
    let value_bytes = value.to_bytes();
    let mut type_0;
    let mut id = -1;
    let Some(first_colon) = value_bytes.iter().position(|&byte| byte == b':') else {
        return None;
    };
    let target_start = first_colon + 1;
    let target_bytes = &value_bytes[target_start..];
    let Some(second_colon) = target_bytes.iter().position(|&byte| byte == b':') else {
        return None;
    };
    let target_bytes = &target_bytes[..second_colon];
    let format_start = target_start + second_colon + 1;

    if target_bytes == b"%*" {
        type_0 = MONITOR_ALL_PANES;
    } else if target_bytes == b"@*" {
        type_0 = MONITOR_ALL_WINDOWS;
    } else if target_bytes.is_empty() {
        type_0 = MONITOR_SESSION;
    } else {
        let target = CString::new(target_bytes).expect("monitor target contains no NUL");
        if sscanf(
            target.as_ptr(),
            b"%%%d\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut id,
        ) == 1 as ::core::ffi::c_int
            && id >= 0 as ::core::ffi::c_int
        {
            type_0 = MONITOR_PANE;
        } else if sscanf(
            target.as_ptr(),
            b"@%d\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut id,
        ) == 1 as ::core::ffi::c_int
            && id >= 0 as ::core::ffi::c_int
        {
            type_0 = MONITOR_WINDOW;
        } else {
            return None;
        }
    }

    Some(ParsedMonitor {
        name: CString::new(&value_bytes[..first_colon]).expect("monitor name contains no NUL"),
        type_0,
        id,
        format: CString::new(&value_bytes[format_start..]).expect("monitor format contains no NUL"),
    })
}

pub fn monitor_parse_owned(value: &CStr) -> Option<ParsedMonitor> {
    unsafe { monitor_parse_parts(value) }
}

pub unsafe fn monitor_add(
    monitor: &refbox::Weak<monitor_set>,
    name: *const ::core::ffi::c_char,
    type_0: monitor_type,
    id: i32,
    format: *const ::core::ffi::c_char,
    flags: i32,
) {
    let name = CStr::from_ptr(name).to_owned();
    let format = CStr::from_ptr(format).to_owned();
    let observer = monitor.clone();
    {
        let Some(mut state) = monitor_borrow(monitor) else {
            return;
        };
        let old = monitor_items_find(
            &mut state.items,
            &monitor_item {
                name: name.clone(),
                ..monitor_item::empty()
            },
        );
        if !old.is_null() {
            monitor_free_item(&mut state, old);
        }
        let identity = state.next_item_identity;
        state.next_item_identity = identity
            .checked_add(1)
            .expect("monitor item identity exhausted");
        let item = Box::new(monitor_item {
            name,
            format,
            type_0,
            id: id as u32,
            flags,
            identity,
            ..monitor_item::empty()
        });
        assert!(monitor_items_insert(&mut state.items, item).is_ok());
        if !state.timer.as_ref().is_some_and(Timer::is_pending) {
            let timeout = Duration::from_secs(1);
            state.timer = Some(
                Timer::new(timeout, move || {
                    monitor_timer(&observer);
                })
                .expect("arm timer"),
            );
        }
    }
}
pub unsafe fn monitor_remove(
    monitor: &refbox::Weak<monitor_set>,
    name: *const ::core::ffi::c_char,
) {
    let name = CStr::from_ptr(name).to_owned();
    {
        let Some(mut state) = monitor_borrow(monitor) else {
            return;
        };
        let item = monitor_items_find(
            &mut state.items,
            &monitor_item {
                name,
                ..monitor_item::empty()
            },
        );
        if !item.is_null() {
            monitor_free_item(&mut state, item);
        }
        if state.items.is_empty() {
            drop(state.timer.take());
        }
    }
}
pub unsafe fn monitor_get_fire_count(
    monitor: &refbox::Weak<monitor_set>,
    name: *const ::core::ffi::c_char,
) -> u_int {
    let name = CStr::from_ptr(name);
    monitor_borrow(monitor)
        .and_then(|state| state.items.get(name.to_bytes()).map(|item| item.fire_count))
        .unwrap_or(0)
}
pub unsafe fn monitor_get_fire_time(
    monitor: &refbox::Weak<monitor_set>,
    name: *const ::core::ffi::c_char,
) -> time_t {
    let name = CStr::from_ptr(name);
    monitor_borrow(monitor)
        .and_then(|state| state.items.get(name.to_bytes()).map(|item| item.fire_time))
        .unwrap_or(0)
}

// Parent indexes own stable Box records; no independent index allocation.
unsafe fn monitor_items_find(head: &mut monitor_items, elm: &monitor_item) -> *mut monitor_item {
    let key = elm.name.as_bytes().to_vec();
    head.get_mut(&key)
        .map_or(std::ptr::null_mut(), |node| &raw mut **node)
}
unsafe fn monitor_items_insert(
    head: &mut monitor_items,
    elm: Box<monitor_item>,
) -> Result<(), (*mut monitor_item, Box<monitor_item>)> {
    let key = elm.name.as_bytes().to_vec();
    match head.entry(key) {
        std::collections::btree_map::Entry::Occupied(mut entry) => {
            Err((&raw mut **entry.get_mut(), elm))
        }
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
            Ok(())
        }
    }
}
unsafe fn monitor_items_remove(
    head: &mut monitor_items,
    elm: *mut monitor_item,
) -> Option<Box<monitor_item>> {
    let elm = elm.as_ref()?;
    let key = elm.name.as_bytes().to_vec();
    if head
        .get(&key)
        .is_none_or(|node| !std::ptr::eq(&**node, elm))
    {
        return None;
    }
    head.remove(&key)
}
unsafe fn monitor_items_minmax(head: &mut monitor_items) -> *mut monitor_item {
    head.first_entry()
        .map_or(std::ptr::null_mut(), |mut entry| &raw mut **entry.get_mut())
}
unsafe fn monitor_items_next(
    head: &mut monitor_items,
    elm: *const monitor_item,
) -> *mut monitor_item {
    let elm = &*elm;
    let key = elm.name.as_bytes().to_vec();
    if head
        .get(&key)
        .is_none_or(|node| !std::ptr::eq(&**node, elm))
    {
        return std::ptr::null_mut();
    }
    head.range_mut((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| &raw mut **node)
}

// Parent indexes own stable Box records; no independent index allocation.
unsafe fn monitor_panes_find(head: &mut monitor_panes, elm: &monitor_pane) -> *mut monitor_pane {
    let key = (elm.pane, elm.idx);
    head.get_mut(&key)
        .map_or(std::ptr::null_mut(), |node| &raw mut **node)
}
unsafe fn monitor_panes_insert(
    head: &mut monitor_panes,
    elm: Box<monitor_pane>,
) -> Result<(), (*mut monitor_pane, Box<monitor_pane>)> {
    let key = (elm.pane, elm.idx);
    match head.entry(key) {
        std::collections::btree_map::Entry::Occupied(mut entry) => {
            Err((&raw mut **entry.get_mut(), elm))
        }
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
            Ok(())
        }
    }
}
unsafe fn monitor_panes_remove(
    head: &mut monitor_panes,
    elm: *mut monitor_pane,
) -> Option<Box<monitor_pane>> {
    let elm = elm.as_ref()?;
    let key = (elm.pane, elm.idx);
    if head
        .get(&key)
        .is_none_or(|node| !std::ptr::eq(&**node, elm))
    {
        return None;
    }
    head.remove(&key)
}
unsafe fn monitor_panes_minmax(head: &mut monitor_panes) -> *mut monitor_pane {
    head.first_entry()
        .map_or(std::ptr::null_mut(), |mut entry| &raw mut **entry.get_mut())
}
unsafe fn monitor_panes_next(
    head: &mut monitor_panes,
    elm: *const monitor_pane,
) -> *mut monitor_pane {
    let elm = &*elm;
    let key = (elm.pane, elm.idx);
    if head
        .get(&key)
        .is_none_or(|node| !std::ptr::eq(&**node, elm))
    {
        return std::ptr::null_mut();
    }
    head.range_mut((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| &raw mut **node)
}

// Parent indexes own stable Box records; no independent index allocation.
unsafe fn monitor_windows_find(
    head: &mut monitor_windows,
    elm: &monitor_window,
) -> *mut monitor_window {
    let key = (elm.window, elm.idx);
    head.get_mut(&key)
        .map_or(std::ptr::null_mut(), |node| &raw mut **node)
}
unsafe fn monitor_windows_insert(
    head: &mut monitor_windows,
    elm: Box<monitor_window>,
) -> Result<(), (*mut monitor_window, Box<monitor_window>)> {
    let key = (elm.window, elm.idx);
    match head.entry(key) {
        std::collections::btree_map::Entry::Occupied(mut entry) => {
            Err((&raw mut **entry.get_mut(), elm))
        }
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(elm);
            Ok(())
        }
    }
}
unsafe fn monitor_windows_remove(
    head: &mut monitor_windows,
    elm: *mut monitor_window,
) -> Option<Box<monitor_window>> {
    let elm = elm.as_ref()?;
    let key = (elm.window, elm.idx);
    if head
        .get(&key)
        .is_none_or(|node| !std::ptr::eq(&**node, elm))
    {
        return None;
    }
    head.remove(&key)
}
unsafe fn monitor_windows_minmax(head: &mut monitor_windows) -> *mut monitor_window {
    head.first_entry()
        .map_or(std::ptr::null_mut(), |mut entry| &raw mut **entry.get_mut())
}
unsafe fn monitor_windows_next(
    head: &mut monitor_windows,
    elm: *const monitor_window,
) -> *mut monitor_window {
    let elm = &*elm;
    let key = (elm.window, elm.idx);
    if head
        .get(&key)
        .is_none_or(|node| !std::ptr::eq(&**node, elm))
    {
        return std::ptr::null_mut();
    }
    head.range_mut((std::ops::Bound::Excluded(key), std::ops::Bound::Unbounded))
        .next()
        .map_or(std::ptr::null_mut(), |(_, node)| &raw mut **node)
}

#[cfg(test)]
mod ownership_tests {
    use super::*;
    use crate::src::shared::client::{client, CLIENT_DEAD};

    #[test]
    fn destroy_expires_observers_before_releasing_callback_captures() {
        use std::cell::{Cell, RefCell};

        struct Capture {
            observer: Rc<RefCell<refbox::Weak<monitor_set>>>,
            drops: Rc<Cell<usize>>,
        }
        impl Drop for Capture {
            fn drop(&mut self) {
                let observer = self.observer.borrow();
                assert!(matches!(
                    observer.try_borrow_mut(),
                    Err(refbox::BorrowError::Dropped)
                ));
                self.drops.set(self.drops.get() + 1);
            }
        }

        unsafe {
            let observer = Rc::new(RefCell::new(refbox::Weak::new()));
            let drops = Rc::new(Cell::new(0));
            let capture = Capture {
                observer: observer.clone(),
                drops: drops.clone(),
            };
            let owner = monitor_create(Rc::new(move |_| {
                let _ = &capture;
            }));
            *observer.borrow_mut() = owner.downgrade();
            monitor_add(
                &owner.downgrade(),
                c"watched".as_ptr(),
                MONITOR_SESSION,
                -1,
                c"".as_ptr(),
                0,
            );
            monitor_destroy(owner);
            assert_eq!(drops.get(), 1);
            // An outstanding observer and the cancelled timer do not keep captures alive.
            crate::src::reactor::poll_runtime();
            assert_eq!(drops.get(), 1);
            crate::src::reactor::shutdown_runtime();
        }
    }

    #[test]
    fn traversal_preserves_prefetched_successor_order_and_rejects_replacements() {
        unsafe {
            let owner = monitor_create(Rc::new(|_| {}));
            for name in [c"a", c"c", c"e"] {
                monitor_add(
                    &owner.downgrade(),
                    name.as_ptr(),
                    MONITOR_SESSION,
                    -1,
                    c"".as_ptr(),
                    0,
                );
            }
            let mut visited = Vec::new();
            monitor_visit(&owner.downgrade(), |identity, _| {
                visited.push(identity.name.to_bytes().to_vec());
                if identity.name.as_c_str() == c"a" {
                    // b precedes the already-prefetched c; d is discovered later.
                    for name in [c"b", c"d"] {
                        monitor_add(
                            &owner.downgrade(),
                            name.as_ptr(),
                            MONITOR_SESSION,
                            -1,
                            c"".as_ptr(),
                            0,
                        );
                    }
                    // The prefetched c identity must not dispatch its replacement.
                    monitor_add(
                        &owner.downgrade(),
                        c"c".as_ptr(),
                        MONITOR_SESSION,
                        -1,
                        c"new".as_ptr(),
                        0,
                    );
                    monitor_remove(&owner.downgrade(), c"a".as_ptr());
                }
                if identity.name.as_c_str() == c"d" {
                    monitor_remove(&owner.downgrade(), c"e".as_ptr());
                }
            });
            assert_eq!(visited, [b"a".to_vec(), b"d".to_vec()]);
            assert_eq!(
                monitor_first_item(&owner.downgrade())
                    .unwrap()
                    .name
                    .as_c_str(),
                c"b"
            );
            monitor_destroy(owner);
        }
    }

    #[test]
    fn callback_destroy_cancels_timer_and_stops_the_remaining_traversal() {
        use std::cell::{Cell, RefCell};
        unsafe {
            let owner_slot = Rc::new(RefCell::new(None::<refbox::RefBox<monitor_set>>));
            let callback_owner = owner_slot.clone();
            let calls = Rc::new(Cell::new(0));
            let callback_calls = calls.clone();
            let owner = monitor_create(Rc::new(move |change| {
                let owner = callback_owner.borrow_mut().take().unwrap();
                // Count, time and last are committed before user callbacks.
                assert_eq!(
                    monitor_get_fire_count(&owner.downgrade(), change.name.as_ptr()),
                    1
                );
                let observer = owner.downgrade();
                monitor_destroy(owner);
                assert!(!observer.is_alive());
                assert_eq!(change.name, c"a");
                assert_eq!(change.value, c"new");
                assert_eq!(change.last, None);
                callback_calls.set(callback_calls.get() + 1);
            }));
            for name in [c"a", c"b"] {
                monitor_add(
                    &owner.downgrade(),
                    name.as_ptr(),
                    MONITOR_SESSION,
                    -1,
                    c"".as_ptr(),
                    MONITOR_NOTIFY_INITIAL,
                );
            }
            let observer = owner.downgrade();
            let active_dispatch = owner.downgrade();
            *owner_slot.borrow_mut() = Some(owner);
            let mut visited = 0;
            monitor_visit(&active_dispatch, |identity, _| {
                visited += 1;
                monitor_check_value(
                    &active_dispatch,
                    &identity,
                    None,
                    refbox::Weak::new(),
                    None,
                    c"new",
                    MonitorValueTarget::Session,
                    None,
                );
            });
            assert_eq!(visited, 1);
            assert_eq!(calls.get(), 1);
            assert!(!observer.is_alive());
            assert!(matches!(
                active_dispatch.try_borrow_mut(),
                Err(refbox::BorrowError::Dropped)
            ));
            // A captured weak timer cannot prolong ownership or revive a dead set.
            drop(active_dispatch);
            assert!(matches!(
                observer.try_borrow_mut(),
                Err(refbox::BorrowError::Dropped)
            ));
            crate::src::reactor::poll_runtime();
            assert_eq!(calls.get(), 1);
            crate::src::reactor::shutdown_runtime();
        }
    }

    #[test]
    fn timer_destruction_in_session_phase_prevents_later_phases_and_rearming() {
        use std::cell::{Cell, RefCell};
        unsafe {
            let saved_sessions = std::ptr::replace(
                &raw mut sessions,
                crate::src::shared::session::sessions::default(),
            );
            let session = crate::src::shared::session::SessionRef::allocate();
            (&session).fixture_metadata(Some(c"monitor-timer-destroy".to_owned()), None, None);
            let session_observer = Rc::downgrade(&session);
            (&mut sessions).insert(session);
            let owner_slot = Rc::new(RefCell::new(None::<refbox::RefBox<monitor_set>>));
            let callback_owner = owner_slot.clone();
            let calls = Rc::new(Cell::new(0));
            let callback_calls = calls.clone();
            let owner = monitor_create_session(
                session_observer.upgrade().as_ref(),
                Rc::new(move |change| {
                    assert_eq!(change.name, c"a-session");
                    assert_eq!(change.value, c"constant");
                    monitor_destroy(callback_owner.borrow_mut().take().unwrap());
                    // The dispatched strings remain valid after logical destruction.
                    assert_eq!(change.value, c"constant");
                    callback_calls.set(callback_calls.get() + 1);
                }),
            );
            for (name, kind) in [
                (c"a-session", MONITOR_SESSION),
                (c"b-session", MONITOR_SESSION),
                (c"c-pane", MONITOR_PANE),
                (c"d-window", MONITOR_WINDOW),
                (c"e-all-panes", MONITOR_ALL_PANES),
                (c"f-all-windows", MONITOR_ALL_WINDOWS),
            ] {
                monitor_add(
                    &owner.downgrade(),
                    name.as_ptr(),
                    kind,
                    -1,
                    c"constant".as_ptr(),
                    MONITOR_NOTIFY_INITIAL,
                );
            }
            let observer = owner.downgrade();
            let dispatch = owner.downgrade();
            *owner_slot.borrow_mut() = Some(owner);
            monitor_timer(&dispatch);
            assert_eq!(calls.get(), 1);
            assert!(!observer.is_alive());
            assert!(matches!(
                dispatch.try_borrow_mut(),
                Err(refbox::BorrowError::Dropped)
            ));
            // Monitor teardown releases its owner before returning; only the
            // session registry remains after the dispatch view is released.
            assert_eq!(session_observer.strong_count(), 1);
            (&mut sessions).remove(&session_observer.upgrade().unwrap());
            assert_eq!(session_observer.strong_count(), 0);
            drop(dispatch);
            assert!(matches!(
                observer.try_borrow_mut(),
                Err(refbox::BorrowError::Dropped)
            ));
            crate::src::reactor::poll_runtime();
            assert_eq!(calls.get(), 1);
            assert!(session_observer.upgrade().is_none());
            crate::src::reactor::shutdown_runtime();
            sessions = saved_sessions;
        }
    }

    #[test]
    fn stale_format_result_does_not_update_same_name_replacement() {
        use std::cell::Cell;
        unsafe {
            let calls = Rc::new(Cell::new(0));
            let callback_calls = calls.clone();
            let owner = monitor_create(Rc::new(move |_| {
                callback_calls.set(callback_calls.get() + 1)
            }));
            monitor_add(
                &owner.downgrade(),
                c"name".as_ptr(),
                MONITOR_SESSION,
                -1,
                c"old".as_ptr(),
                MONITOR_NOTIFY_INITIAL,
            );
            let old = monitor_first_item(&owner.downgrade()).unwrap();
            monitor_add(
                &owner.downgrade(),
                c"name".as_ptr(),
                MONITOR_SESSION,
                -1,
                c"new".as_ptr(),
                MONITOR_NOTIFY_INITIAL,
            );
            monitor_check_value(
                &owner.downgrade(),
                &old,
                None,
                refbox::Weak::new(),
                None,
                c"stale",
                MonitorValueTarget::Session,
                None,
            );
            assert_eq!(calls.get(), 0);
            let replacement = monitor_first_item(&owner.downgrade()).unwrap();
            assert_ne!(old.identity, replacement.identity);
            assert_eq!(
                monitor_with_item(&owner.downgrade(), &replacement, |item| item.last.clone()),
                Some(None)
            );
            monitor_destroy(owner);
        }
    }

    #[test]
    fn notify_true_updates_last_without_firing_until_truth_changes() {
        use std::cell::RefCell;
        unsafe {
            let changes = Rc::new(RefCell::new(Vec::new()));
            let callback_changes = changes.clone();
            let owner = monitor_create(Rc::new(move |change| {
                callback_changes
                    .borrow_mut()
                    .push((change.value.to_owned(), change.last.map(CStr::to_owned)))
            }));
            monitor_add(
                &owner.downgrade(),
                c"name".as_ptr(),
                MONITOR_SESSION,
                -1,
                c"".as_ptr(),
                MONITOR_NOTIFY_INITIAL | MONITOR_NOTIFY_TRUE,
            );
            let item = monitor_first_item(&owner.downgrade()).unwrap();
            for value in [c"0", c"0", c"yes", c"yes", c"0", c"next"] {
                monitor_check_value(
                    &owner.downgrade(),
                    &item,
                    None,
                    refbox::Weak::new(),
                    None,
                    value,
                    MonitorValueTarget::Session,
                    None,
                );
            }
            assert_eq!(
                &*changes.borrow(),
                &[
                    (c"yes".to_owned(), Some(c"0".to_owned())),
                    (c"next".to_owned(), Some(c"0".to_owned()))
                ]
            );
            assert_eq!(
                monitor_get_fire_count(&owner.downgrade(), c"name".as_ptr()),
                2
            );
            assert_eq!(
                monitor_get_fire_time(&owner.downgrade(), c"name".as_ptr()),
                current_time
            );
            monitor_destroy(owner);
        }
    }

    #[test]
    fn client_scan_guards_release_immediately_on_missing_session_and_dead_client() {
        use crate::src::reactor::{poll_runtime, shutdown_runtime};

        unsafe {
            for cancel in [false, true] {
                let client = ClientRef::allocate();
                let observer = Rc::downgrade(&client);
                let set_owner = monitor_create_client(Some(&client), Rc::new(|_| {}));
                let set_observer = set_owner.downgrade();
                let set = &set_observer;
                assert!(monitor_has_client(set));

                // The client exists, but the missing session ends the scan early.
                monitor_check_sessions(set);
                assert_eq!(observer.strong_count(), 1);
                observer
                    .upgrade()
                    .unwrap()
                    .update_flags(CLIENT_DEAD as uint64_t, 0);
                assert!(monitor_client(set).is_none());
                assert_eq!(observer.strong_count(), 1);

                drop(client);
                assert!(observer.upgrade().is_none());
                if cancel {
                    shutdown_runtime();
                } else {
                    poll_runtime();
                }
                assert!(observer.upgrade().is_none());
                assert!(
                    monitor_has_client(set),
                    "expired explicit client remains selected"
                );
                assert!(monitor_client(set).is_none());
                monitor_check_sessions(set);
                monitor_destroy(set_owner);
                shutdown_runtime();
            }
            let global_owner = monitor_create(Rc::new(|_| {}));
            let global_observer = global_owner.downgrade();
            let global = &global_observer;
            assert!(!monitor_has_client(global));
            monitor_destroy(global_owner);
        }
    }

    #[test]
    fn session_scan_guards_release_immediately_and_monitor_owner_releases_on_teardown() {
        use crate::src::reactor::{poll_runtime, shutdown_runtime};

        unsafe {
            let saved = std::ptr::replace(
                &raw mut sessions,
                crate::src::shared::session::sessions::default(),
            );
            let owner = crate::src::shared::session::SessionRef::allocate();
            (&owner).fixture_metadata(Some(c"monitor-release-test".to_owned()), None, None);
            let observer = std::rc::Rc::downgrade(&owner);
            (&mut sessions).insert(owner);
            let set_owner =
                monitor_create_session(observer.upgrade().as_ref(), std::rc::Rc::new(|_| {}));
            let set_observer = set_owner.downgrade();
            let set = &set_observer;
            monitor_add(set, c"missing".as_ptr(), MONITOR_PANE, -1, c"".as_ptr(), 0);
            let identity = monitor_first_item(set).expect("registered monitor item");

            // Missing pane and window both return after acquiring a guard.
            monitor_check_pane(set, &identity);
            monitor_check_window(set, &identity);
            // Empty scans exercise the normal exit paths.
            monitor_check_all_panes(set);
            monitor_check_all_windows(set);
            assert_eq!(observer.strong_count(), 2);
            poll_runtime();
            assert_eq!(observer.strong_count(), 2);

            (&mut sessions).remove(&observer.upgrade().expect("indexed session"));
            monitor_destroy(set_owner);
            assert_eq!(observer.strong_count(), 0);
            shutdown_runtime();
            assert!(observer.upgrade().is_none());
            sessions = saved;
        }
    }

    struct Capture {
        set: refbox::Weak<monitor_set>,
        name: Vec<u8>,
        value: Vec<u8>,
        last: Vec<u8>,
    }

    #[test]
    fn changed_value_survives_reentrant_item_removal() {
        unsafe {
            let capture = std::rc::Rc::new(std::cell::RefCell::new(Capture {
                set: refbox::Weak::new(),
                name: Vec::new(),
                value: Vec::new(),
                last: Vec::new(),
            }));
            let callback_capture = capture.clone();
            let set_owner = monitor_create_client(
                None,
                crate::src::shared::monitor::monitor_callback(move |change| {
                    let mut capture = callback_capture.borrow_mut();
                    capture.value = change.value.to_bytes().to_vec();
                    capture.last = change.last.expect("previous value").to_bytes().to_vec();
                    monitor_remove(&capture.set, change.name.as_ptr());
                    capture.name = change.name.to_bytes().to_vec();
                }),
            );
            let set_observer = set_owner.downgrade();
            let set = &set_observer;
            capture.borrow_mut().set = set.clone();
            monitor_add(
                set,
                c"reentrant-last".as_ptr(),
                MONITOR_SESSION,
                -1,
                c"value".as_ptr(),
                0,
            );
            let item = monitor_first_item(set).unwrap();
            let first = CString::from_vec_with_nul(b"\xffold\0".to_vec()).unwrap();
            monitor_check_value(
                set,
                &item,
                None,
                refbox::Weak::new(),
                None,
                &first,
                MonitorValueTarget::Session,
                None,
            );
            assert_eq!(
                monitor_with_item(set, &item, |item| item.last.clone())
                    .unwrap()
                    .as_deref()
                    .expect("string is present")
                    .to_bytes(),
                first.to_bytes()
            );
            monitor_check_value(
                set,
                &item,
                None,
                refbox::Weak::new(),
                None,
                &first,
                MonitorValueTarget::Session,
                None,
            );
            assert!(capture.borrow().value.is_empty());

            let second = CString::from_vec_with_nul(b"\xfeneW\0".to_vec()).unwrap();
            monitor_check_value(
                set,
                &item,
                None,
                refbox::Weak::new(),
                None,
                &second,
                MonitorValueTarget::Session,
                None,
            );
            assert_eq!(capture.borrow().value.as_slice(), second.to_bytes());
            assert_eq!(capture.borrow().last.as_slice(), first.to_bytes());
            assert_eq!(capture.borrow().name.as_slice(), b"reentrant-last");
            assert!(monitor_first_item(set).is_none());
            monitor_destroy(set_owner);
        }
    }

    #[test]
    fn monitor_item_indexes_preserve_identity_move_duplicate_and_removal() {
        unsafe {
            fn item(name: &str) -> Box<monitor_item> {
                let mut item = Box::new(monitor_item::empty());
                item.name = CString::new(name).unwrap();
                item
            }

            let mut head = monitor_items::new();
            let mut other = monitor_items::new();
            let mut first_owner = item("alpha");
            let first = &raw mut *first_owner;
            let mut second_owner = item("beta");
            let second = &raw mut *second_owner;
            let mut duplicate_owner = item("alpha");
            let duplicate = &raw mut *duplicate_owner;
            assert!(monitor_items_insert(&mut head, first_owner).is_ok());
            assert!(monitor_items_insert(&mut head, second_owner).is_ok());
            let (existing, duplicate_owner) = monitor_items_insert(&mut head, duplicate_owner)
                .err()
                .expect("duplicate returned to caller");
            assert_eq!(existing, first);
            assert!(monitor_items_remove(&mut other, first).is_none());

            let mut moved = head;
            assert_eq!(monitor_items_minmax(&mut moved), first);
            assert_eq!(monitor_items_next(&mut moved, first), second);
            let mut first_owner = monitor_items_remove(&mut moved, first).expect("indexed record");
            assert_eq!(&raw mut *first_owner, first);
            drop(first_owner);
            let mut second_owner =
                monitor_items_remove(&mut moved, second).expect("indexed record");
            assert_eq!(&raw mut *second_owner, second);
            drop(second_owner);
            drop(duplicate_owner);
            assert!(moved.is_empty());
            drop(moved);
        }
    }

    #[test]
    fn monitor_child_indexes_transfer_owners_and_keep_order() {
        unsafe {
            let mut panes = monitor_panes::new();
            let mut other_panes = monitor_panes::new();
            let mut pane1 = Box::new(monitor_pane::empty());
            pane1.pane = 3;
            pane1.idx = 1;
            let mut pane1_owner = pane1;
            let pane1 = &raw mut *pane1_owner;
            let mut pane2 = Box::new(monitor_pane::empty());
            pane2.pane = 3;
            pane2.idx = 2;
            let mut pane2_owner = pane2;
            let pane2 = &raw mut *pane2_owner;
            assert!(monitor_panes_insert(&mut panes, pane1_owner).is_ok());
            assert!(monitor_panes_insert(&mut panes, pane2_owner).is_ok());
            assert!(monitor_panes_remove(&mut other_panes, pane1).is_none());
            assert_eq!(monitor_panes_next(&mut panes, pane1), pane2);
            let mut pane1_owner = monitor_panes_remove(&mut panes, pane1).expect("indexed record");
            assert_eq!(&raw mut *pane1_owner, pane1);
            drop(pane1_owner);
            let mut pane2_owner = monitor_panes_remove(&mut panes, pane2).expect("indexed record");
            assert_eq!(&raw mut *pane2_owner, pane2);
            drop(pane2_owner);
            assert!(panes.is_empty());

            let mut windows = monitor_windows::new();
            let mut other_windows = monitor_windows::new();
            let mut window1 = Box::new(monitor_window::empty());
            window1.window = 8;
            window1.idx = 1;
            let mut window1_owner = window1;
            let window1 = &raw mut *window1_owner;
            let mut window2 = Box::new(monitor_window::empty());
            window2.window = 8;
            window2.idx = 2;
            let mut window2_owner = window2;
            let window2 = &raw mut *window2_owner;
            assert!(monitor_windows_insert(&mut windows, window1_owner).is_ok());
            assert!(monitor_windows_insert(&mut windows, window2_owner).is_ok());
            assert!(monitor_windows_remove(&mut other_windows, window1).is_none());
            assert_eq!(monitor_windows_next(&mut windows, window1), window2);
            let mut window1_owner =
                monitor_windows_remove(&mut windows, window1).expect("indexed record");
            assert_eq!(&raw mut *window1_owner, window1);
            drop(window1_owner);
            let mut window2_owner =
                monitor_windows_remove(&mut windows, window2).expect("indexed record");
            assert_eq!(&raw mut *window2_owner, window2);
            drop(window2_owner);
            assert!(windows.is_empty());
        }
    }
}

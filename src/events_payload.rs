use crate::src::cmd::find::{
    cmd_find_clear_state, cmd_find_from_nothing, cmd_find_from_pane, cmd_find_from_session,
    cmd_find_from_session_window, cmd_find_from_winlink, cmd_find_from_winlink_pane,
    cmd_find_valid_state,
};
use crate::src::format::bytes::format_message_with;
use crate::src::format::bytes::write_cstr;
use crate::src::format::format_add;
use crate::src::log::{fatalx, log_cstr, log_cstr_n, log_debug};
use crate::src::reactor::{
    evbuffer_add, evbuffer_add_formatted, evbuffer_get_length, evbuffer_new, evbuffer_pullup,
};
use crate::src::server_client::server_client_unref_owned;
use crate::src::session::{session_add_ref, session_alive, session_remove_ref};
use crate::src::shared::abi::ssize_t;
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::command::cmd_find_state;
use crate::src::shared::event::*;
use crate::src::shared::events::{
    event_payload, event_payload_item, event_payload_tree, event_payload_tree_storage,
    event_payload_type, EventPayloadIdentity, EventPayloadValue,
};
use crate::src::shared::format::format_tree;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::window::{window, winlink};
use crate::src::window::{
    window_add_ref, window_has_pane, window_pane_add_ref, window_pane_remove_ref,
    window_remove_ref, winlink_find_by_index,
};
use std::ffi::{CStr, CString};

pub const EVENT_PAYLOAD_POINTER: event_payload_type = 8;
pub const EVENT_PAYLOAD_PANE: event_payload_type = 7;
pub const EVENT_PAYLOAD_WINDOW: event_payload_type = 6;
pub const EVENT_PAYLOAD_SESSION: event_payload_type = 5;
pub const EVENT_PAYLOAD_CLIENT: event_payload_type = 4;
pub const EVENT_PAYLOAD_STRING: event_payload_type = 0;

fn event_payload_find<'a>(ep: &'a event_payload, name: &CStr) -> Option<&'a event_payload_item> {
    ep.items
        .entries
        .entries
        .get(name.to_bytes())
        .map(Box::as_ref)
}

pub fn event_payload_items(ep: &event_payload) -> impl Iterator<Item = &event_payload_item> {
    ep.items.entries.entries.values().map(Box::as_ref)
}

unsafe fn event_payload_free_target(ep: &mut event_payload) {
    let target = &mut ep.target;
    if let Some(session) = ep.target_session.take() {
        session_remove_ref(
            session,
            c"event_payload_free_target",
        );
    }
    if let Some(window) = ep.target_window.take() {
        window_remove_ref(window.get(), b"event_payload_free_target\0" as *const u8 as *const ::core::ffi::c_char, || window);
    }
    if let Some(pane) = ep.target_pane.take() {
        window_pane_remove_ref(
            pane,
            b"event_payload_free_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    cmd_find_clear_state(target, 0 as ::core::ffi::c_int);
}
impl Drop for event_payload_item {
    fn drop(&mut self) {
        // Release retained models while the item's name is still alive.
        let value = std::mem::replace(
            &mut self.value,
            EventPayloadValue::String(Default::default()),
        );
        unsafe {
            match value {
                EventPayloadValue::Client(client) => server_client_unref_owned(client),
                EventPayloadValue::Session(session) => {
                    session_remove_ref(session, c"event_payload_free_value")
                }
                EventPayloadValue::Window(window) => {
                    window_remove_ref(window.get(), c"event_payload_free_value".as_ptr(), || window)
                }
                EventPayloadValue::Pane(pane) => {
                    window_pane_remove_ref(pane, c"event_payload_free_value".as_ptr())
                }
                _ => {}
            }
        }
    }
}

unsafe fn event_payload_set_item(
    ep: &mut event_payload,
    name: *const ::core::ffi::c_char,
    value: EventPayloadValue,
) {
    // A legacy caller may borrow the old item's name. Copy it before removing
    // that owner, and finish removal before releasing references can dispatch.
    let name = CStr::from_ptr(name).to_owned();
    let key = name.to_bytes().to_vec();
    let new = Box::new(event_payload_item { name, value });
    let old = ep.items.entries.entries.remove(&key);
    drop(old);
    ep.items.entries.entries.insert(key, new);
}

pub fn event_payload_create() -> Box<event_payload> {
    Box::new(event_payload {
        target_pane: None,
        target_window: None,
        target_session: None,
        items: event_payload_tree::default(),
        target: cmd_find_state {
            idx: -1,
            ..Default::default()
        },
    })
}

impl Drop for event_payload {
    fn drop(&mut self) {
        // Match tmux: remove each item in key order before releasing its models,
        // then release the target after all item-triggered dispatch has finished.
        while let Some((_, item)) = self.items.entries.entries.pop_first() {
            drop(item);
        }
        unsafe { event_payload_free_target(self) };
    }
}

pub unsafe fn event_payload_set_target(ep: &mut event_payload, fs: &cmd_find_state) {
    event_payload_free_target(ep);
    let target = &mut ep.target;
    ep.target_session = fs.s.upgrade();
    if ep.target_session.is_some() { target.s = fs.s.clone(); }
    ep.target_window = fs.w.upgrade();
    if ep.target_window.is_some() { target.w = fs.w.clone(); }
    let link = match fs.wl.try_borrow_mut() {
        Ok(link) => Some(link),
        Err(refbox::BorrowError::Dropped) => None,
        Err(refbox::BorrowError::Borrowed) => panic!("event target winlink already borrowed"),
    };
    if let Some(link) = link {
        target.idx = link.idx;
        if ep.target_session.is_none() {
            ep.target_session = link.session.upgrade();
            target.s = ep.target_session.as_ref().map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        }
        if ep.target_window.is_none() {
            ep.target_window = link.window_owner.clone();
            target.w = ep.target_window.as_ref().map_or_else(std::rc::Weak::new, std::rc::Rc::downgrade);
        }
    } else {
        target.idx = -1;
    }
    ep.target_pane = fs.wp.upgrade();
    if ep.target_pane.is_some() { target.wp = fs.wp.clone(); }

}
pub unsafe fn event_payload_get_target(
    ep: &event_payload,
    fs: &mut cmd_find_state,
) -> ::core::ffi::c_int {
    let t = &ep.target;
    let session_owner = t.s.upgrade();
    let window_owner = t.w.upgrade();
    let pane_owner = t.wp.upgrade();
    let s = session_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    let w = window_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    let wp = pane_owner.as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut flags: ::core::ffi::c_int = fs.flags;
    if t.idx != -(1 as ::core::ffi::c_int)
        && !s.is_null()
        && !w.is_null()
        && session_alive(s.as_ref()) != 0
    {
        wl = winlink_find_by_index(&raw mut (*s).windows, t.idx);
        if !wl.is_null() && (*wl).window_ptr() != w {
            wl = ::core::ptr::null_mut::<winlink>();
        }
    }
    cmd_find_clear_state(fs, flags);
    fs.s = t.s.clone();
    fs.w = t.w.clone();
    fs.wp = t.wp.clone();
    fs.set_wl(wl);
    fs.idx = if !wl.is_null() {
        (*wl).idx
    } else {
        -(1 as ::core::ffi::c_int)
    };
    if cmd_find_valid_state(&*fs) != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if !wl.is_null() && !wp.is_null() && window_has_pane(&*(*wl).window_ptr(), &t.wp) {
        cmd_find_from_winlink_pane(fs, wl, wp, flags);
        if cmd_find_valid_state(&*fs) != 0 {
            return 1 as ::core::ffi::c_int;
        }
    }
    if !wp.is_null()
        && cmd_find_from_pane(fs, wp, flags) == 0 as ::core::ffi::c_int
        && cmd_find_valid_state(&*fs) != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if !wl.is_null() {
        cmd_find_from_winlink(fs, wl, flags);
        if cmd_find_valid_state(&*fs) != 0 {
            return 1 as ::core::ffi::c_int;
        }
    }
    if !s.is_null()
        && !w.is_null()
        && session_alive(s.as_ref()) != 0
        && cmd_find_from_session_window(fs, s, w, flags) == 0 as ::core::ffi::c_int
        && cmd_find_valid_state(&*fs) != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if !s.is_null() && session_alive(s.as_ref()) != 0 {
        cmd_find_from_session(fs, s, flags);
        if cmd_find_valid_state(&*fs) != 0 {
            return 1 as ::core::ffi::c_int;
        }
    }
    if cmd_find_from_nothing(fs, flags) == 0 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    cmd_find_clear_state(fs, flags);
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn event_payload_set_string(
    ep: &mut event_payload,
    mut name: *const ::core::ffi::c_char,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    let string = format_message_with(write);
    event_payload_set_item(&mut *ep, name, EventPayloadValue::String(string));
}
pub unsafe fn event_payload_set_time(
    ep: &mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: time_t,
) {
    event_payload_set_item(&mut *ep, name, EventPayloadValue::Time(value));
}
pub unsafe fn event_payload_set_int(
    ep: &mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: ::core::ffi::c_int,
) {
    event_payload_set_item(&mut *ep, name, EventPayloadValue::Int(value));
}
pub unsafe fn event_payload_set_uint(
    ep: &mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: u_int,
) {
    event_payload_set_item(&mut *ep, name, EventPayloadValue::Uint(value));
}
pub unsafe fn event_payload_set_client(ep: &mut event_payload, mut c: *mut client) {
    let mut name: *const ::core::ffi::c_char =
        b"client\0" as *const u8 as *const ::core::ffi::c_char;
    let client = (*c).observer.upgrade().expect("live Rc client");
    event_payload_set_item(&mut *ep, name, EventPayloadValue::Client(client));
}
pub unsafe fn event_payload_set_session(
    ep: &mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut s: *mut session,
) {
    let session = session_add_ref(
        s,
        b"event_payload_set_session\0" as *const u8 as *const ::core::ffi::c_char,
    );
    event_payload_set_item(&mut *ep, name, EventPayloadValue::Session(session));
}
pub unsafe fn event_payload_set_window(
    ep: &mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut w: *mut window,
) {
    let window = window_add_ref(
        w,
        b"event_payload_set_window\0" as *const u8 as *const ::core::ffi::c_char,
    );
    event_payload_set_item(&mut *ep, name, EventPayloadValue::Window(window));
}
pub unsafe fn event_payload_set_pane(
    ep: &mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut wp: *mut window_pane,
) {
    let pane = window_pane_add_ref(
        wp,
        b"event_payload_set_pane\0" as *const u8 as *const ::core::ffi::c_char,
    );
    event_payload_set_item(&mut *ep, name, EventPayloadValue::Pane(pane));
}
pub unsafe fn event_payload_set_identity(
    ep: &mut event_payload,
    mut name: *const ::core::ffi::c_char,
    identity: EventPayloadIdentity,
) {
    event_payload_set_item(&mut *ep, name, EventPayloadValue::Identity(identity));
}
pub fn event_payload_get_string(ep: &event_payload) -> Option<&CStr> {
    match event_payload_find(ep, c"paste_buffer").map(|item| &item.value) {
        Some(EventPayloadValue::String(value)) => Some(value),
        _ => None,
    }
}
unsafe fn event_payload_add_item(epi: &event_payload_item, evb: &mut evbuffer) {
    match epi.type_0() as ::core::ffi::c_uint {
        0 => {
            evbuffer_add_formatted(evb, |out| write_cstr(out, epi.value.string()));
        }
        1 => {
            evbuffer_add_formatted(evb, |out| {
                write!(
                    out,
                    "{}",
                    (epi.value.time() as ::core::ffi::c_longlong) as i64
                )
            });
        }
        2 => {
            evbuffer_add_formatted(evb, |out| write!(out, "{}", (epi.value.number()) as i32));
        }
        3 => {
            evbuffer_add_formatted(evb, |out| {
                write!(out, "{}", (epi.value.unsigned_number()) as u32)
            });
        }
        4 => {
            evbuffer_add_formatted(evb, |out| {
                write_cstr(
                    out,
                    ((*epi.value.client().get()).name)
                        .as_ref()
                        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                )
            });
        }
        5 => {
            evbuffer_add_formatted(evb, |out| {
                write!(out, "${}", ((*epi.value.session().get()).id) as u32)
            });
        }
        6 => {
            evbuffer_add_formatted(evb, |out| {
                write!(out, "@{}", ((*epi.value.window().get()).id) as u32)
            });
        }
        7 => {
            evbuffer_add_formatted(evb, |out| {
                write!(out, "%{}", ((*epi.value.pane().get()).id) as u32)
            });
        }
        8 => {
            evbuffer_add_formatted(evb, |out| {
                let address = epi.value.identity().address();
                if address == 0 {
                    out.write_all(b"(nil)")
                } else {
                    write!(out, "{:p}", address as *const ())
                }
            });
        }
        _ => {}
    };
}
/// Printed payload bytes with one trailing NUL for synchronous C consumers.
/// The bytes before that terminator may themselves contain NULs.
pub(crate) unsafe fn event_payload_item_print_owned(epi: &event_payload_item) -> Vec<u8> {
    let mut size: size_t = 0;
    let mut evb = evbuffer_new();
    event_payload_add_item(epi, &mut *evb);
    size = evbuffer_get_length(&evb);
    let mut value = Vec::with_capacity(size + 1);
    value.extend_from_slice(evbuffer_pullup(&mut evb, -1).unwrap_or_default());
    value.push(0);
    value
}

pub(crate) unsafe fn event_payload_print_owned(ep: &event_payload) -> Option<Vec<u8>> {
    event_payload_find(ep, c"pane").map(|item| event_payload_item_print_owned(item))
}
pub unsafe fn event_payload_add_formats(
    ep: &event_payload,
    mut ft: *mut format_tree,
    mut prefix: *const ::core::ffi::c_char,
) {
    if prefix.is_null() {
        prefix = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    let prefix = CStr::from_ptr(prefix).to_bytes();
    for epi in event_payload_items(&*ep) {
        let key = epi.name.as_ptr();
        if !(*key as ::core::ffi::c_int == '_' as i32) {
            let value = event_payload_item_print_owned(epi);
            let key_bytes = CStr::from_ptr(key).to_bytes();
            let mut name_bytes = Vec::with_capacity(prefix.len() + key_bytes.len());
            name_bytes.extend_from_slice(prefix);
            name_bytes.extend_from_slice(key_bytes);
            let name = CString::new(name_bytes).expect("C string parts contain no NUL");
            // format_add copies the key into its format entry before returning.
            format_add(ft, name.as_ptr(), |out| {
                write_cstr(out, value.as_ptr().cast::<::core::ffi::c_char>())
            });
            let named = if epi.type_0() as ::core::ffi::c_uint
                == EVENT_PAYLOAD_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                Some((*epi.value.session().get()).name.as_ptr().cast_mut())
            } else if epi.type_0() as ::core::ffi::c_uint
                == EVENT_PAYLOAD_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                Some((*epi.value.window().get()).name.as_ptr().cast_mut())
            } else {
                None
            };
            if let Some(named) = named {
                let mut suffixed = name.as_bytes().to_vec();
                suffixed.extend_from_slice(b"_name");
                let suffixed = CString::new(suffixed).expect("C string parts contain no NUL");
                format_add(ft, suffixed.as_ptr(), |out| write_cstr(out, named));
            }
        }
    }
}
pub fn event_payload_item_name(epi: &event_payload_item) -> &CStr {
    &epi.name
}

pub unsafe fn event_payload_log(
    ep: &event_payload,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    let prefix = format_message_with(write);
    let mut evb = evbuffer_new();
    for epi in event_payload_items(ep) {
        if evbuffer_get_length(&evb) != 0 as size_t {
            evbuffer_add_formatted(&mut *evb, |out| out.write_all(b", "));
        }
        evbuffer_add_formatted(&mut *evb, |out| {
            write_cstr(out, epi.name.as_ptr())?;
            out.write_all(b"=")
        });
        event_payload_add_item(epi, &mut *evb);
    }

    log_debug(format_args!(
        "{}{}",
        log_cstr((prefix.as_ptr()) as *const _),
        log_cstr_n(
            (evbuffer_pullup(&mut *evb, -1).map_or(std::ptr::null_mut(), |bytes| bytes.as_mut_ptr())
                as *mut ::core::ffi::c_char) as *const _,
            evbuffer_get_length(&evb) as ::core::ffi::c_int
        )
    ));
}
pub unsafe fn event_payload_get_client(ep: &event_payload) -> *mut client {
    match event_payload_find(ep, c"client").map(|item| &item.value) {
        Some(EventPayloadValue::Client(value)) => crate::src::shared::rc::as_ptr(value),
        _ => std::ptr::null_mut(),
    }
}
pub unsafe fn event_payload_get_session(ep: &event_payload) -> *mut session {
    match event_payload_find(ep, c"session").map(|item| &item.value) {
        Some(EventPayloadValue::Session(value)) => crate::src::shared::rc::as_ptr(value),
        _ => std::ptr::null_mut(),
    }
}
pub unsafe fn event_payload_get_window(ep: &event_payload) -> *mut window {
    match event_payload_find(ep, c"window").map(|item| &item.value) {
        Some(EventPayloadValue::Window(value)) => crate::src::shared::rc::as_ptr(value),
        _ => std::ptr::null_mut(),
    }
}
pub unsafe fn event_payload_get_pane(ep: &event_payload) -> *mut window_pane {
    match event_payload_find(ep, c"pane").map(|item| &item.value) {
        Some(EventPayloadValue::Pane(value)) => crate::src::shared::rc::as_ptr(value),
        _ => std::ptr::null_mut(),
    }
}
pub unsafe fn event_payload_get_identity(
    ep: &event_payload,
    name: *const ::core::ffi::c_char,
) -> Option<&EventPayloadIdentity> {
    if name.is_null() {
        return None;
    }
    match event_payload_find(ep, CStr::from_ptr(name)).map(|item| &item.value) {
        Some(EventPayloadValue::Identity(value)) => Some(value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::src::cmd::queue::{cmdq_free_detached, cmdq_get_callback_owned};
    use crate::src::reactor::evbuffer_add;
    use std::ffi::{CStr, CString};

    #[test]
    fn typed_event_identities_keep_pointer_format_without_owning_queue_items() {
        unsafe {
            let item = cmdq_get_callback_owned(c"event identity".as_ptr(), None);
            let observer = (*item).observer.clone();
            let mut payload = event_payload_create();
            event_payload_set_identity(&mut payload, c"_cmdq_item".as_ptr(),
                EventPayloadIdentity::QueueItem(observer.clone()));
            let identity = event_payload_get_identity(&payload, c"_cmdq_item".as_ptr()).unwrap();
            assert!(matches!(identity, EventPayloadIdentity::QueueItem(weak) if weak.ptr_eq(&observer)));
            assert_eq!(observer.strong_count(), 1);
            let printed = event_payload_item_print_owned(event_payload_find(&payload, c"_cmdq_item").unwrap());
            assert_eq!(&printed[..printed.len() - 1], format!("{item:p}").as_bytes());
            cmdq_free_detached(item);
            assert!(observer.upgrade().is_none());

            event_payload_set_identity(&mut payload, c"_hooks_monitor".as_ptr(),
                EventPayloadIdentity::HookMonitor(0x1234));
            assert!(matches!(event_payload_get_identity(&payload, c"_hooks_monitor".as_ptr()),
                Some(EventPayloadIdentity::HookMonitor(0x1234))));
        }
    }

    #[test]
    fn payload_handle_and_target_retain_pane_after_source_release() {
        unsafe {
            let pane = window_pane::new();
            let observed = std::rc::Rc::downgrade(&pane);
            let value = EventPayloadValue::Pane(pane.clone());
            let retained = value.pane().clone();
            let state = cmd_find_state { wp: observed.clone(), ..Default::default() };
            let mut payload = event_payload_create();
            event_payload_set_target(&mut payload, &state);
            assert!(std::rc::Rc::ptr_eq(payload.target_pane.as_ref().unwrap(), &pane));
            drop(pane);
            drop(value);
            drop(payload);
            assert!(observed.upgrade().is_some());
            drop(retained);
            assert!(observed.upgrade().is_none());
            let mut expired_payload = event_payload_create();
            event_payload_set_target(&mut expired_payload, &state);
            assert!(expired_payload.target_pane.is_none());
        }
    }

    #[test]
    fn replacement_accepts_the_previous_items_borrowed_name() {
        unsafe {
            let mut ep = event_payload_create();
            let name = CString::new(vec![b'k', 0xff]).unwrap();
            event_payload_set_string(&mut *ep, name.as_ptr(), |out| {
                write_cstr(out, c"old".as_ptr())
            });
            let old_name = event_payload_item_name(event_payload_items(&*ep).next().unwrap()).as_ptr();
            event_payload_set_int(&mut *ep, old_name, 42);

            let replacement = event_payload_items(&*ep).next().unwrap();
            assert_eq!(
                event_payload_item_name(replacement).to_bytes(),
                name.as_bytes()
            );
            assert_eq!((*replacement).value.number(), 42);
            assert_eq!(event_payload_items(&*ep).count(), 1);
            drop(ep);
        }
    }

    #[test]
    fn boxed_item_drop_releases_models_and_allows_nested_event_dispatch() {
        use crate::src::events::{events_add_sink, events_fire, events_remove_sink};
        use crate::src::shared::events::events_callback;
        use crate::src::shared::rc;
        use std::cell::RefCell;
        use std::rc::Rc;
        unsafe {
            let first_owner = window::new();
        let first = rc::as_ptr(&first_owner);
            (*first).id = 11;
            let second_owner = window::new();
        let second = rc::as_ptr(&second_owner);
            (*second).id = 22;
            let first_observer = (*first).observer.clone();
            let second_observer = (*second).observer.clone();
            let closed = Rc::new(RefCell::new(Vec::new()));
            let observed = closed.clone();
            let sink = events_add_sink(
                c"window-closed",
                events_callback(move |_, payload| {
                    let window = event_payload_get_window(payload);
                    observed.borrow_mut().push((*window).id);
                    events_fire(c"payload-nested-cleanup".as_ptr(), event_payload_create());
                }),
            );
            let mut payload = event_payload {
                target_pane: None,
        target_window: None,
        target_session: None,
                items: event_payload_tree::default(),
                target: Default::default(),
            };
            event_payload_set_window(&mut payload, c"alpha".as_ptr(), first);
            event_payload_set_window(&mut payload, c"beta".as_ptr(), second);
            window_remove_ref(first_owner.get(), c"test initial owner".as_ptr(), || first_owner);
            window_remove_ref(second_owner.get(), c"test initial owner".as_ptr(), || second_owner);
            event_payload_set_int(&mut payload, c"alpha".as_ptr(), 7);
            assert!(first_observer.upgrade().is_none());
            assert!(second_observer.upgrade().is_some());
            assert_eq!(*closed.borrow(), [11]);
            drop(payload);
            assert!(second_observer.upgrade().is_none());
            assert_eq!(*closed.borrow(), [11, 22]);
            events_remove_sink(sink);
        }
    }

    #[test]
    fn boxed_items_preserve_order_and_addresses_when_the_payload_moves() {
        unsafe {
            let mut payload = event_payload {
                target_pane: None,
        target_window: None,
        target_session: None,
                items: event_payload_tree::default(),
                target: Default::default(),
            };
            event_payload_set_int(&mut payload, c"beta".as_ptr(), 2);
            event_payload_set_int(&mut payload, c"alpha".as_ptr(), 1);
            let address = event_payload_find(&payload, c"alpha").unwrap()
                as *const event_payload_item as usize;
            let mut moved = payload;
            for i in 0..128 {
                let key = CString::new(format!("extra-{i}")).unwrap();
                event_payload_set_int(&mut moved, key.as_ptr(), i);
            }
            assert_eq!(
                event_payload_items(&moved).next().unwrap() as *const event_payload_item as usize,
                address
            );
            assert_eq!(
                event_payload_items(&moved)
                    .take(2)
                    .map(|item| item.name.to_bytes())
                    .collect::<Vec<_>>(),
                [b"alpha".as_slice(), b"beta".as_slice()]
            );
            let item = moved
                .items
                .entries
                .entries
                .remove(b"alpha".as_slice())
                .unwrap();
            assert_eq!(item.value.number(), 1);
            assert!(event_payload_find(&moved, c"alpha").is_none());
            drop(moved);
            assert_eq!(item.name.as_bytes(), b"alpha");
        }
    }

    #[test]
    fn payload_drop_releases_items_in_key_order_before_the_target() {
        use crate::src::events::{events_add_sink, events_remove_sink};
        use crate::src::shared::events::events_callback;
        use crate::src::shared::rc;
        use std::cell::RefCell;
        use std::rc::Rc;

        unsafe {
            let first_owner = window::new();
        let first = rc::as_ptr(&first_owner);
            (*first).id = 11;
            let second_owner = window::new();
        let second = rc::as_ptr(&second_owner);
            (*second).id = 22;
            let target_owner = window::new();
        let target = rc::as_ptr(&target_owner);
            (*target).id = 33;
            let first_observer = (*first).observer.clone();
            let second_observer = (*second).observer.clone();
            let target_observer = (*target).observer.clone();
            let target_during_cleanup = target_observer.clone();
            let closed = Rc::new(RefCell::new(Vec::new()));
            let observed = closed.clone();
            let sink = events_add_sink(
                c"window-closed",
                events_callback(move |_, payload| {
                    let id = (*event_payload_get_window(payload)).id;
                    // Reentrant item cleanup still has the payload's target.
                    if id != 33 {
                        assert!(target_during_cleanup.upgrade().is_some());
                    }
                    observed.borrow_mut().push(id);
                }),
            );

            let mut payload = event_payload_create();
            let fs = cmd_find_state {
                w: (*target).observer.clone(),
                ..Default::default()
            };
            event_payload_set_target(&mut payload, &fs);
            event_payload_set_window(&mut payload, c"beta".as_ptr(), first);
            event_payload_set_window(&mut payload, c"alpha".as_ptr(), second);
            window_remove_ref(first_owner.get(), c"test initial owner".as_ptr(), || first_owner);
            window_remove_ref(second_owner.get(), c"test initial owner".as_ptr(), || second_owner);
            window_remove_ref(target_owner.get(), c"test initial owner".as_ptr(), || target_owner);
            drop(payload);

            assert_eq!(*closed.borrow(), [22, 11, 33]);
            assert!(first_observer.upgrade().is_none());
            assert!(second_observer.upgrade().is_none());
            assert!(target_observer.upgrade().is_none());
            events_remove_sink(sink);
        }
    }

    #[test]
    fn dispatch_keeps_payload_models_alive_until_every_sink_finishes() {
        use crate::src::events::{events_add_sink, events_fire, events_remove_sink};
        use crate::src::shared::events::events_callback;
        use crate::src::shared::rc;
        use std::cell::RefCell;
        use std::rc::Rc;

        unsafe {
            let window_owner = window::new();
        let window = rc::as_ptr(&window_owner);
            (*window).id = 44;
            let observer = (*window).observer.clone();
            let observed = Rc::new(RefCell::new(Vec::new()));
            let mut sinks = Vec::new();
            for _ in 0..2 {
                let observer = observer.clone();
                let observed = observed.clone();
                sinks.push(events_add_sink(
                    c"payload-owner-test",
                    events_callback(move |_, payload| {
                        assert!(observer.upgrade().is_some());
                        observed
                            .borrow_mut()
                            .push((*event_payload_get_window(payload)).id);
                    }),
                ));
            }
            let mut payload = event_payload_create();
            event_payload_set_window(&mut payload, c"window".as_ptr(), window);
            window_remove_ref(window_owner.get(), c"test initial owner".as_ptr(), || window_owner);
            events_fire(c"payload-owner-test".as_ptr(), payload);

            assert_eq!(*observed.borrow(), [44, 44]);
            assert!(observer.upgrade().is_none());
            for sink in sinks {
                events_remove_sink(sink);
            }
        }
    }
}

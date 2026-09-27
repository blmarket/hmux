use crate::src::cmd::find::{
    cmd_find_from_client, cmd_find_from_pane, cmd_find_from_session, cmd_find_from_window,
    cmd_find_from_winlink,
};
use crate::src::events_payload::{
    event_payload_create, event_payload_log, event_payload_set_client, event_payload_set_int,
    event_payload_set_pane, event_payload_set_session, event_payload_set_string,
    event_payload_set_target, event_payload_set_window,
};
use crate::src::ffi::libc::strcmp;
use crate::src::format::bytes::write_cstr;
use crate::src::log::log_get_level;
use crate::src::session::session_alive;
use crate::src::shared::abi::*;
use crate::src::shared::client::client;
use crate::src::shared::command::cmd_find_state;
use crate::src::shared::events::{event_payload, events_cb, events_sink};
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::window::{window, winlink};
use std::ffi::CStr;

static mut events_sinks: Vec<Box<events_sink>> = Vec::new();
static mut events_dispatching: u_int = 0;
static mut events_generation: u_int = 0;

unsafe fn events_free_sink(mut es: *mut events_sink) {
    let sinks = &mut *(&raw mut events_sinks);
    if let Some(index) = sinks
        .iter()
        .position(|sink| (&**sink as *const events_sink).cast_mut() == es)
    {
        drop(sinks.remove(index));
    }
}
unsafe fn events_free_dead() {
    let sinks = &mut *(&raw mut events_sinks);
    let mut index = 0;
    while index < sinks.len() {
        if sinks[index].dead != 0 {
            drop(sinks.remove(index));
        } else {
            index += 1;
        }
    }
}
pub unsafe fn events_add_sink(name: &CStr, cb: events_cb) -> *mut events_sink {
    events_generation = events_generation.wrapping_add(1);
    let mut owner = Box::new(events_sink {
        name: name.to_owned(),
        cb: cb,
        dead: 0,
        generation: events_generation,
    });

    let es = &raw mut *owner;
    (&mut *(&raw mut events_sinks)).push(owner);
    es
}
pub unsafe fn events_remove_sink(mut es: *mut events_sink) {
    if !es.is_null() && (*es).dead == 0 {
        if events_dispatching != 0 as u_int {
            (*es).dead = 1 as ::core::ffi::c_int;
        } else {
            events_free_sink(es);
        }
    }
}
pub unsafe fn events_fire(mut name: *const ::core::ffi::c_char, mut ep: Box<event_payload>) {
    let mut es: *mut events_sink = ::core::ptr::null_mut::<events_sink>();
    let mut generation: u_int = events_generation;
    event_payload_set_string(
        &mut *ep,
        b"event\0" as *const u8 as *const ::core::ffi::c_char,
        |out| write_cstr(out, name),
    );
    if log_get_level() != 0 as ::core::ffi::c_int {
        event_payload_log(&ep, |out| {
            write_cstr(
                out,
                b"events_fire\0" as *const u8 as *const ::core::ffi::c_char,
            )?;
            out.write_all(b": ")?;
            write_cstr(out, name)?;
            out.write_all(b": ")
        });
    }
    events_dispatching = events_dispatching.wrapping_add(1);
    let mut index = 0;
    while index < (&*(&raw const events_sinks)).len() {
        es = {
            let sinks = &mut *(&raw mut events_sinks);
            &raw mut *sinks[index]
        };
        index += 1;
        if !((*es).dead != 0 || (*es).generation > generation) {
            if strcmp(((*es).name).as_ptr().cast_mut(), name) == 0 as ::core::ffi::c_int {
                let callback = (*es).cb.clone();
                callback(CStr::from_ptr(name), &mut *ep);
            }
        }
    }
    events_dispatching = events_dispatching.wrapping_sub(1);
    if events_dispatching == 0 as u_int {
        events_free_dead();
    }
    drop(ep);
}
pub unsafe fn events_fire_client(mut name: *const ::core::ffi::c_char, mut c: *mut client) {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut ep = event_payload_create();
    cmd_find_from_client(&raw mut fs, c, 0 as ::core::ffi::c_int);
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_client(&mut *ep, c);
    if !fs.s.is_null() {
        event_payload_set_session(
            &mut *ep,
            b"session\0" as *const u8 as *const ::core::ffi::c_char,
            fs.s,
        );
    }
    if !fs.w.is_null() {
        event_payload_set_window(
            &mut *ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            fs.w,
        );
    }
    if !fs.wl.is_null() {
        event_payload_set_int(
            &mut *ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*fs.wl).idx,
        );
    } else if fs.idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            &mut *ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            fs.idx,
        );
    }
    if !fs.wp.is_null() {
        event_payload_set_pane(
            &mut *ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            fs.wp,
        );
    }
    events_fire(name, ep);
}
pub unsafe fn events_fire_session(mut name: *const ::core::ffi::c_char, mut s: *mut session) {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut ep = event_payload_create();
    if session_alive(s) != 0 {
        cmd_find_from_session(&raw mut fs, s, 0 as ::core::ffi::c_int);
        event_payload_set_target(&mut *ep, &fs);
    }
    event_payload_set_session(
        &mut *ep,
        b"session\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    events_fire(name, ep);
}
pub unsafe fn events_fire_window(mut name: *const ::core::ffi::c_char, mut w: *mut window) {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut ep = event_payload_create();
    cmd_find_from_window(&raw mut fs, w, 0 as ::core::ffi::c_int);
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_window(
        &mut *ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    events_fire(name, ep);
}
pub unsafe fn events_fire_pane(mut name: *const ::core::ffi::c_char, mut wp: *mut window_pane) {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_pane(
        &mut *ep,
        b"pane\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
    );
    event_payload_set_window(
        &mut *ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    events_fire(name, ep);
}
pub unsafe fn events_fire_winlink(mut name: *const ::core::ffi::c_char, mut wl: *mut winlink) {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut ep = event_payload_create();
    cmd_find_from_winlink(&raw mut fs, wl, 0 as ::core::ffi::c_int);
    event_payload_set_target(&mut *ep, &fs);
    event_payload_set_session(
        &mut *ep,
        b"session\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).session,
    );
    event_payload_set_window(
        &mut *ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).window_ptr(),
    );
    event_payload_set_int(
        &mut *ep,
        b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).idx,
    );
    events_fire(name, ep);
}

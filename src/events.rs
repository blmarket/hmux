use crate::src::cmd_find::{
    cmd_find_from_client, cmd_find_from_pane, cmd_find_from_session, cmd_find_from_window,
    cmd_find_from_winlink,
};
use crate::src::events_payload::{
    event_payload_create, event_payload_free, event_payload_log, event_payload_set_client,
    event_payload_set_int, event_payload_set_pane, event_payload_set_session,
    event_payload_set_string, event_payload_set_target, event_payload_set_window,
};
use crate::src::ffi::libc::strcmp;
use crate::src::log::log_get_level;
use crate::src::session::session_alive;
use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::events::{event_payload, events_cb, events_sink};
pub use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::{
    window_pane, window_pane_modes, window_pane_prompt, window_pane_tree_entry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

static mut events_sinks: Vec<Box<events_sink>> = Vec::new();
static mut events_dispatching: u_int = 0;
static mut events_generation: u_int = 0;

unsafe extern "C" fn events_free_sink(mut es: *mut events_sink) {
    let sinks = &mut *(&raw mut events_sinks);
    if let Some(index) = sinks
        .iter()
        .position(|sink| (&**sink as *const events_sink).cast_mut() == es)
    {
        drop(sinks.remove(index));
    }
}
unsafe extern "C" fn events_free_dead() {
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
#[no_mangle]
pub unsafe extern "C" fn events_add_sink(
    mut name: *const ::core::ffi::c_char,
    mut cb: events_cb,
    mut data: *mut ::core::ffi::c_void,
) -> *mut events_sink {
    events_generation = events_generation.wrapping_add(1);
    let mut owner = Box::new(events_sink {
        name: CStr::from_ptr(name).to_owned(),
        cb: cb,
        data: data,
        dead: 0,
        generation: events_generation,
    });

    let es = &raw mut *owner;
    (&mut *(&raw mut events_sinks)).push(owner);
    es
}
#[no_mangle]
pub unsafe extern "C" fn events_remove_sink(mut es: *mut events_sink) {
    if !es.is_null() && (*es).dead == 0 {
        if events_dispatching != 0 as u_int {
            (*es).dead = 1 as ::core::ffi::c_int;
        } else {
            events_free_sink(es);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn events_fire(
    mut name: *const ::core::ffi::c_char,
    mut ep: *mut event_payload,
) {
    let mut es: *mut events_sink = ::core::ptr::null_mut::<events_sink>();
    let mut generation: u_int = events_generation;
    event_payload_set_string(
        ep,
        b"event\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    if log_get_level() != 0 as ::core::ffi::c_int {
        event_payload_log(
            ep,
            b"%s: %s: \0" as *const u8 as *const ::core::ffi::c_char,
            b"events_fire\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
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
                (*es).cb.expect("non-null function pointer")(name, ep, (*es).data);
            }
        }
    }
    events_dispatching = events_dispatching.wrapping_sub(1);
    if events_dispatching == 0 as u_int {
        events_free_dead();
    }
    event_payload_free(ep);
}
#[no_mangle]
pub unsafe extern "C" fn events_fire_client(
    mut name: *const ::core::ffi::c_char,
    mut c: *mut client,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_client(&raw mut fs, c, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_client(
        ep,
        b"client\0" as *const u8 as *const ::core::ffi::c_char,
        c,
    );
    if !fs.s.is_null() {
        event_payload_set_session(
            ep,
            b"session\0" as *const u8 as *const ::core::ffi::c_char,
            fs.s,
        );
    }
    if !fs.w.is_null() {
        event_payload_set_window(
            ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            fs.w,
        );
    }
    if !fs.wl.is_null() {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*fs.wl).idx,
        );
    } else if fs.idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            fs.idx,
        );
    }
    if !fs.wp.is_null() {
        event_payload_set_pane(
            ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            fs.wp,
        );
    }
    events_fire(name, ep);
}
#[no_mangle]
pub unsafe extern "C" fn events_fire_session(
    mut name: *const ::core::ffi::c_char,
    mut s: *mut session,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
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
    events_fire(name, ep);
}
#[no_mangle]
pub unsafe extern "C" fn events_fire_window(
    mut name: *const ::core::ffi::c_char,
    mut w: *mut window,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_window(&raw mut fs, w, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    events_fire(name, ep);
}
#[no_mangle]
pub unsafe extern "C" fn events_fire_pane(
    mut name: *const ::core::ffi::c_char,
    mut wp: *mut window_pane,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    events_fire(name, ep);
}
#[no_mangle]
pub unsafe extern "C" fn events_fire_winlink(
    mut name: *const ::core::ffi::c_char,
    mut wl: *mut winlink,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_winlink(&raw mut fs, wl, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_session(
        ep,
        b"session\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).session,
    );
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).window,
    );
    event_payload_set_int(
        ep,
        b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).idx,
    );
    events_fire(name, ep);
}

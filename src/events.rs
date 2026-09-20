pub use crate::src::shared::events::{event_payload, events_cb, events_sink, events_sink_entry};
pub use crate::src::shared::arguments::{args};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_data};
pub use crate::src::shared::options::{options};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::process::{tmuxpeer};
pub use crate::src::shared::prompt::{prompt};
pub use crate::src::shared::redraw::{redraw_scene};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::{spawn_editor_state};
pub use crate::src::shared::status::{status_line};
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::environment::{environ};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;
extern "C" {

    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn event_payload_create() -> *mut event_payload;
    fn event_payload_free(_: *mut event_payload);
    fn event_payload_log(_: *mut event_payload, _: *const ::core::ffi::c_char, ...);
    fn event_payload_set_target(_: *mut event_payload, _: *mut cmd_find_state);
    fn event_payload_set_string(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn event_payload_set_int(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    );
    fn event_payload_set_client(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut client,
    );
    fn event_payload_set_session(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut session,
    );
    fn event_payload_set_window(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut window,
    );
    fn event_payload_set_pane(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut window_pane,
    );
    fn cmd_find_from_session(_: *mut cmd_find_state, _: *mut session, _: ::core::ffi::c_int);
    fn cmd_find_from_winlink(_: *mut cmd_find_state, _: *mut winlink, _: ::core::ffi::c_int);
    fn cmd_find_from_window(
        _: *mut cmd_find_state,
        _: *mut window,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_find_from_pane(
        _: *mut cmd_find_state,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_find_from_client(
        _: *mut cmd_find_state,
        _: *mut client,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn session_alive(_: *mut session) -> ::core::ffi::c_int;
    fn log_get_level() -> ::core::ffi::c_int;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_12 {
    pub offset: u_int,
    pub data: C2RustUnnamed_13,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_13 {
    pub attr: u_char,
    pub fg: u_char,
    pub bg: u_char,
    pub data: u_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct events_sinks {
    pub tqh_first: *mut events_sink,
    pub tqh_last: *mut *mut events_sink,
}
static mut events_sinks: events_sinks = events_sinks {
    tqh_first: ::core::ptr::null::<events_sink>() as *mut events_sink,
    tqh_last: ::core::ptr::null::<*mut events_sink>() as *mut *mut events_sink,
};
static mut events_dispatching: u_int = 0;
static mut events_generation: u_int = 0;
unsafe extern "C" fn events_free_sink(mut es: *mut events_sink) {
    if !(*es).entry.tqe_next.is_null() {
        (*(*es).entry.tqe_next).entry.tqe_prev = (*es).entry.tqe_prev;
    } else {
        events_sinks.tqh_last = (*es).entry.tqe_prev;
    }
    *(*es).entry.tqe_prev = (*es).entry.tqe_next;
    free((*es).name as *mut ::core::ffi::c_void);
    free(es as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn events_free_dead() {
    let mut es: *mut events_sink = ::core::ptr::null_mut::<events_sink>();
    let mut es1: *mut events_sink = ::core::ptr::null_mut::<events_sink>();
    es = events_sinks.tqh_first;
    while !es.is_null() && {
        es1 = (*es).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if (*es).dead != 0 {
            events_free_sink(es);
        }
        es = es1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn events_add_sink(
    mut name: *const ::core::ffi::c_char,
    mut cb: events_cb,
    mut data: *mut ::core::ffi::c_void,
) -> *mut events_sink {
    let mut es: *mut events_sink = ::core::ptr::null_mut::<events_sink>();
    es = xcalloc(1 as size_t, ::core::mem::size_of::<events_sink>() as size_t) as *mut events_sink;
    (*es).name = xstrdup(name);
    (*es).cb = cb;
    (*es).data = data;
    events_generation = events_generation.wrapping_add(1);
    (*es).generation = events_generation;
    (*es).entry.tqe_next = ::core::ptr::null_mut::<events_sink>();
    (*es).entry.tqe_prev = events_sinks.tqh_last;
    *events_sinks.tqh_last = es;
    events_sinks.tqh_last = &raw mut (*es).entry.tqe_next;
    return es;
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
    es = events_sinks.tqh_first;
    while !es.is_null() {
        if !((*es).dead != 0 || (*es).generation > generation) {
            if strcmp((*es).name, name) == 0 as ::core::ffi::c_int {
                (*es).cb.expect("non-null function pointer")(name, ep, (*es).data);
            }
        }
        es = (*es).entry.tqe_next;
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
unsafe extern "C" fn run_static_initializers() {
    events_sinks = events_sinks {
        tqh_first: ::core::ptr::null_mut::<events_sink>(),
        tqh_last: &raw mut events_sinks.tqh_first,
    };
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [run_static_initializers];

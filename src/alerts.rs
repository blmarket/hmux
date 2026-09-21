use crate::src::events::events_fire_winlink;
use crate::src::log::log_debug;
use crate::src::options::options_get_number;
use crate::src::reactor::{event_add, event_del, event_initialized, event_once, event_set};
pub use crate::src::server::clients;
use crate::src::server_fn::server_status_session;
use crate::src::shared::abi::*;
pub use crate::src::shared::alerts::{
    ALERT_ANY, ALERT_CURRENT, ALERT_OTHER, VISUAL_BOTH, VISUAL_OFF,
};
pub use crate::src::shared::arguments::args;
pub use crate::src::shared::client::CLIENT_CONTROL;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
pub use crate::src::shared::event::EV_TIMEOUT;
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
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
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
pub use crate::src::shared::tree::RB_NEGINF;
use crate::src::shared::tty::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::window::{
    WINDOW_ACTIVITY, WINDOW_ALERTFLAGS, WINDOW_BELL, WINDOW_SILENCE, WINLINK_ACTIVITY,
    WINLINK_BELL, WINLINK_SILENCE,
};
use crate::src::status::status_message_set;
use crate::src::tty::tty_putcode;
pub use crate::src::window::windows;
use crate::src::window::{
    window_add_ref, window_remove_ref, windows_RB_MINMAX, windows_RB_NEXT, winlinks_RB_MINMAX,
    winlinks_RB_NEXT,
};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub tqh_first: *mut window,
    pub tqh_last: *mut *mut window,
}

pub const SESSION_ALERTED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
static mut alerts_fired: ::core::ffi::c_int = 0;
static mut alerts_list: C2RustUnnamed_35 = C2RustUnnamed_35 {
    tqh_first: ::core::ptr::null::<window>() as *mut window,
    tqh_last: ::core::ptr::null::<*mut window>() as *mut *mut window,
};
unsafe extern "C" fn alerts_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut w: *mut window = arg as *mut window;
    log_debug(
        b"@%u alerts timer expired\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
    );
    alerts_queue(w, WINDOW_SILENCE);
}
unsafe extern "C" fn alerts_callback(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut w1: *mut window = ::core::ptr::null_mut::<window>();
    let mut alerts: ::core::ffi::c_int = 0;
    w = alerts_list.tqh_first;
    while !w.is_null() && {
        w1 = (*w).alerts_entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        alerts = alerts_check_all(w);
        log_debug(
            b"@%u alerts check, alerts %#x\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
            alerts,
        );
        (*w).alerts_queued = 0 as ::core::ffi::c_int;
        if !(*w).alerts_entry.tqe_next.is_null() {
            (*(*w).alerts_entry.tqe_next).alerts_entry.tqe_prev = (*w).alerts_entry.tqe_prev;
        } else {
            alerts_list.tqh_last = (*w).alerts_entry.tqe_prev;
        }
        *(*w).alerts_entry.tqe_prev = (*w).alerts_entry.tqe_next;
        (*w).flags &= !WINDOW_ALERTFLAGS;
        window_remove_ref(
            w,
            b"alerts_callback\0" as *const u8 as *const ::core::ffi::c_char,
        );
        w = w1;
    }
    alerts_fired = 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn alerts_action_applies(
    mut wl: *mut winlink,
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut action: ::core::ffi::c_int = 0;
    action = options_get_number((*(*wl).session).options, name) as ::core::ffi::c_int;
    if action == ALERT_ANY {
        return 1 as ::core::ffi::c_int;
    }
    if action == ALERT_CURRENT {
        return (wl == (*(*wl).session).curw) as ::core::ffi::c_int;
    }
    if action == ALERT_OTHER {
        return (wl != (*(*wl).session).curw) as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn alerts_check_all(mut w: *mut window) -> ::core::ffi::c_int {
    let mut alerts: ::core::ffi::c_int = 0;
    alerts = alerts_check_bell(w);
    alerts |= alerts_check_activity(w);
    alerts |= alerts_check_silence(w);
    return alerts;
}
#[no_mangle]
pub unsafe extern "C" fn alerts_check_session(mut s: *mut session) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        alerts_check_all((*wl).window);
        wl = winlinks_RB_NEXT(wl);
    }
}
unsafe extern "C" fn alerts_enabled(
    mut w: *mut window,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if flags & WINDOW_BELL != 0 {
        if options_get_number(
            (*w).options,
            b"monitor-bell\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
        {
            return 1 as ::core::ffi::c_int;
        }
    }
    if flags & WINDOW_ACTIVITY != 0 {
        if options_get_number(
            (*w).options,
            b"monitor-activity\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
        {
            return 1 as ::core::ffi::c_int;
        }
    }
    if flags & WINDOW_SILENCE != 0 {
        if options_get_number(
            (*w).options,
            b"monitor-silence\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0 as ::core::ffi::c_longlong
        {
            return 1 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn alerts_reset_all() {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
    while !w.is_null() {
        alerts_reset(w);
        w = windows_RB_NEXT(w);
    }
}
unsafe extern "C" fn alerts_reset(mut w: *mut window) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if event_initialized(&raw mut (*w).alerts_timer) == 0 {
        event_set(
            &raw mut (*w).alerts_timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            Some(
                alerts_timer
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            w as *mut ::core::ffi::c_void,
        );
    }
    (*w).flags &= !WINDOW_SILENCE;
    event_del(&raw mut (*w).alerts_timer);
    tv.tv_usec = 0 as __suseconds_t;
    tv.tv_sec = tv.tv_usec as __time_t;
    tv.tv_sec = options_get_number(
        (*w).options,
        b"monitor-silence\0" as *const u8 as *const ::core::ffi::c_char,
    ) as __time_t;
    log_debug(
        b"@%u alerts timer reset %u\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        tv.tv_sec as u_int,
    );
    if tv.tv_sec != 0 as __time_t {
        event_add(&raw mut (*w).alerts_timer, &raw mut tv);
    }
}
#[no_mangle]
pub unsafe extern "C" fn alerts_queue(mut w: *mut window, mut flags: ::core::ffi::c_int) {
    alerts_reset(w);
    if (*w).flags & flags != flags {
        (*w).flags |= flags;
        log_debug(
            b"@%u alerts flags added %#x\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
            flags,
        );
    }
    if alerts_enabled(w, flags) != 0 {
        if (*w).alerts_queued == 0 {
            (*w).alerts_queued = 1 as ::core::ffi::c_int;
            (*w).alerts_entry.tqe_next = ::core::ptr::null_mut::<window>();
            (*w).alerts_entry.tqe_prev = alerts_list.tqh_last;
            *alerts_list.tqh_last = w;
            alerts_list.tqh_last = &raw mut (*w).alerts_entry.tqe_next;
            window_add_ref(
                w,
                b"alerts_queue\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if alerts_fired == 0 {
            log_debug(
                b"alerts check queued (by @%u)\0" as *const u8 as *const ::core::ffi::c_char,
                (*w).id,
            );
            event_once(
                -(1 as ::core::ffi::c_int),
                EV_TIMEOUT as ::core::ffi::c_short,
                Some(
                    alerts_callback
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            ::core::ffi::c_short,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
                NULL,
                ::core::ptr::null::<timeval>(),
            );
            alerts_fired = 1 as ::core::ffi::c_int;
        }
    }
}
unsafe extern "C" fn alerts_check_bell(mut w: *mut window) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    if !(*w).flags & WINDOW_BELL != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if options_get_number(
        (*w).options,
        b"monitor-bell\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        (*(*wl).session).flags &= !SESSION_ALERTED;
        wl = (*wl).wentry.tqe_next;
    }
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        s = (*wl).session;
        if (*s).curw != wl || (*s).attached == 0 as u_int {
            (*wl).flags |= WINLINK_BELL;
            server_status_session(s);
        }
        if !(alerts_action_applies(
            wl,
            b"bell-action\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0)
        {
            events_fire_winlink(
                b"alert-bell\0" as *const u8 as *const ::core::ffi::c_char,
                wl,
            );
            if !((*s).flags & SESSION_ALERTED != 0) {
                (*s).flags |= SESSION_ALERTED;
                alerts_set_message(
                    wl,
                    b"Bell\0" as *const u8 as *const ::core::ffi::c_char,
                    b"visual-bell\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        wl = (*wl).wentry.tqe_next;
    }
    return 0x1 as ::core::ffi::c_int;
}
unsafe extern "C" fn alerts_check_activity(mut w: *mut window) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    if !(*w).flags & WINDOW_ACTIVITY != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if options_get_number(
        (*w).options,
        b"monitor-activity\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        (*(*wl).session).flags &= !SESSION_ALERTED;
        wl = (*wl).wentry.tqe_next;
    }
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if !((*wl).flags & WINLINK_ACTIVITY != 0) {
            s = (*wl).session;
            if (*s).curw != wl || (*s).attached == 0 as u_int {
                (*wl).flags |= WINLINK_ACTIVITY;
                server_status_session(s);
            }
            if !(alerts_action_applies(
                wl,
                b"activity-action\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0)
            {
                events_fire_winlink(
                    b"alert-activity\0" as *const u8 as *const ::core::ffi::c_char,
                    wl,
                );
                if !((*s).flags & SESSION_ALERTED != 0) {
                    (*s).flags |= SESSION_ALERTED;
                    alerts_set_message(
                        wl,
                        b"Activity\0" as *const u8 as *const ::core::ffi::c_char,
                        b"visual-activity\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
            }
        }
        wl = (*wl).wentry.tqe_next;
    }
    return 0x2 as ::core::ffi::c_int;
}
unsafe extern "C" fn alerts_check_silence(mut w: *mut window) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    if !(*w).flags & WINDOW_SILENCE != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if options_get_number(
        (*w).options,
        b"monitor-silence\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_longlong
    {
        return 0 as ::core::ffi::c_int;
    }
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        (*(*wl).session).flags &= !SESSION_ALERTED;
        wl = (*wl).wentry.tqe_next;
    }
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if !((*wl).flags & WINLINK_SILENCE != 0) {
            s = (*wl).session;
            if (*s).curw != wl || (*s).attached == 0 as u_int {
                (*wl).flags |= WINLINK_SILENCE;
                server_status_session(s);
            }
            if !(alerts_action_applies(
                wl,
                b"silence-action\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0)
            {
                events_fire_winlink(
                    b"alert-silence\0" as *const u8 as *const ::core::ffi::c_char,
                    wl,
                );
                if !((*s).flags & SESSION_ALERTED != 0) {
                    (*s).flags |= SESSION_ALERTED;
                    alerts_set_message(
                        wl,
                        b"Silence\0" as *const u8 as *const ::core::ffi::c_char,
                        b"visual-silence\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
            }
        }
        wl = (*wl).wentry.tqe_next;
    }
    return 0x4 as ::core::ffi::c_int;
}
unsafe extern "C" fn alerts_set_message(
    mut wl: *mut winlink,
    mut type_0: *const ::core::ffi::c_char,
    mut option: *const ::core::ffi::c_char,
) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut visual: ::core::ffi::c_int = 0;
    visual = options_get_number((*(*wl).session).options, option) as ::core::ffi::c_int;
    c = clients.tqh_first;
    while !c.is_null() {
        if !((*c).session != (*wl).session || (*c).flags & CLIENT_CONTROL as uint64_t != 0) {
            if visual == VISUAL_OFF || visual == VISUAL_BOTH {
                tty_putcode(&raw mut (*c).tty, TTYC_BEL);
            }
            if !(visual == VISUAL_OFF) {
                if (*(*c).session).curw == wl {
                    status_message_set(
                        c,
                        -(1 as ::core::ffi::c_int),
                        1 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        b"%s in current window\0" as *const u8 as *const ::core::ffi::c_char,
                        type_0,
                    );
                } else {
                    status_message_set(
                        c,
                        -(1 as ::core::ffi::c_int),
                        1 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        b"%s in window %d\0" as *const u8 as *const ::core::ffi::c_char,
                        type_0,
                        (*wl).idx,
                    );
                }
            }
        }
        c = (*c).entry.tqe_next;
    }
}
unsafe extern "C" fn run_static_initializers() {
    alerts_list = C2RustUnnamed_35 {
        tqh_first: ::core::ptr::null_mut::<window>(),
        tqh_last: &raw mut alerts_list.tqh_first,
    };
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [run_static_initializers];

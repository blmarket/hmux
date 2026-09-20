use crate::src::cmd_find::cmd_find_from_window;
use crate::src::control::control_get_window_size;
use crate::src::events::{events_fire, events_fire_window};
use crate::src::events_payload::{
    event_payload_create, event_payload_set_target, event_payload_set_uint,
    event_payload_set_window,
};
use crate::src::ffi::libc::sscanf;
use crate::src::layout::layout_resize;
use crate::src::log::log_debug;
use crate::src::options::{options_get_number, options_get_string};
pub use crate::src::server::clients;
use crate::src::server_fn::server_redraw_window;
use crate::src::session::{session_has, sessions_RB_MINMAX, sessions_RB_NEXT};
pub use crate::src::session::sessions;
use crate::src::status::{status_line_size, status_update_cache};
use crate::src::tmux::global_w_options;
use crate::src::tty::tty_update_window_offset;
use crate::src::window::{
    window_has_pane, window_resize, window_unzoom, window_zoom, window_zoomed_pane,
    windows_RB_MINMAX, windows_RB_NEXT,
};
pub use crate::src::window::windows;
pub use crate::src::shared::events::{event_payload};

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
pub use crate::src::shared::window::{window, window_alerts_entry, window_entry, window_mode, window_mode_entry, window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry, winlink_stack, winlink_wentry, winlinks};
pub use crate::src::shared::environment::{environ};
pub use crate::src::shared::limits::{__INT_MAX__, UINT_MAX};
pub use crate::src::shared::window::{
    WINDOW_MAXIMUM, WINDOW_MINIMUM, WINDOW_RESIZE, WINDOW_SIZE_LARGEST, WINDOW_SIZE_LATEST,
    WINDOW_SIZE_MANUAL,
};
pub use crate::src::shared::pane::{
    PANE_MINIMUM, window_pane_offset, window_pane_resize, window_pane_resize_entry,
    window_pane_resizes,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::tree::{RB_NEGINF};
pub use crate::src::shared::client::{
    CLIENT_CONTROL, CLIENT_DEAD, CLIENT_EXIT, CLIENT_IGNORESIZE, CLIENT_NOSIZEFLAGS,
    CLIENT_SIZECHANGED, CLIENT_STATUSOFF, CLIENT_SUSPENDED, CLIENT_UNATTACHEDFLAGS,
    CLIENT_WINDOWSIZECHANGED,
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

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

unsafe extern "C" fn resize_fire_window_resized(
    mut w: *mut window,
    mut old_sx: u_int,
    mut old_sy: u_int,
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
    event_payload_set_uint(
        ep,
        b"width\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).sx,
    );
    event_payload_set_uint(
        ep,
        b"height\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).sy,
    );
    event_payload_set_uint(
        ep,
        b"old_width\0" as *const u8 as *const ::core::ffi::c_char,
        old_sx,
    );
    event_payload_set_uint(
        ep,
        b"old_height\0" as *const u8 as *const ::core::ffi::c_char,
        old_sy,
    );
    events_fire(
        b"window-resized\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
#[no_mangle]
pub unsafe extern "C" fn resize_window(
    mut w: *mut window,
    mut sx: u_int,
    mut sy: u_int,
    mut xpixel: ::core::ffi::c_int,
    mut ypixel: ::core::ffi::c_int,
) {
    let mut zwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut old_sx: u_int = (*w).sx;
    let mut old_sy: u_int = (*w).sy;
    if sx < WINDOW_MINIMUM as u_int {
        sx = WINDOW_MINIMUM as u_int;
    }
    if sx > WINDOW_MAXIMUM as u_int {
        sx = WINDOW_MAXIMUM as u_int;
    }
    if sy < WINDOW_MINIMUM as u_int {
        sy = WINDOW_MINIMUM as u_int;
    }
    if sy > WINDOW_MAXIMUM as u_int {
        sy = WINDOW_MAXIMUM as u_int;
    }
    zwp = window_zoomed_pane(w);
    if !zwp.is_null() {
        window_unzoom(w, 1 as ::core::ffi::c_int);
    }
    layout_resize(w, sx, sy);
    if sx < (*(*w).layout_root).g.sx {
        sx = (*(*w).layout_root).g.sx;
    }
    if sy < (*(*w).layout_root).g.sy {
        sy = (*(*w).layout_root).g.sy;
    }
    window_resize(w, sx, sy, xpixel, ypixel);
    log_debug(
        b"%s: @%u resized to %ux%u; layout %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"resize_window\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        sx,
        sy,
        (*(*w).layout_root).g.sx,
        (*(*w).layout_root).g.sy,
    );
    if !zwp.is_null() && window_has_pane(w, zwp) != 0 {
        window_zoom(zwp);
    }
    tty_update_window_offset(w);
    server_redraw_window(w);
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    resize_fire_window_resized(w, old_sx, old_sy);
    (*w).flags &= !WINDOW_RESIZE;
}
unsafe extern "C" fn ignore_client_size(mut c: *mut client) -> ::core::ffi::c_int {
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    if (*c).session.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    if (*c).flags & CLIENT_NOSIZEFLAGS as uint64_t != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if (*c).flags & CLIENT_IGNORESIZE as uint64_t != 0 {
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            if !(*loop_0).session.is_null() {
                if !((*loop_0).flags & CLIENT_NOSIZEFLAGS as uint64_t != 0) {
                    if !(*loop_0).flags & CLIENT_IGNORESIZE as uint64_t != 0 {
                        return 1 as ::core::ffi::c_int;
                    }
                }
            }
            loop_0 = (*loop_0).entry.tqe_next;
        }
    }
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0
        && !(*c).flags & CLIENT_SIZECHANGED as uint64_t != 0
        && !(*c).flags as ::core::ffi::c_ulonglong & CLIENT_WINDOWSIZECHANGED != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn clients_with_window(mut w: *mut window) -> u_int {
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut n: u_int = 0 as u_int;
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        if !(ignore_client_size(loop_0) != 0 || session_has((*loop_0).session, w) == 0) {
            n = n.wrapping_add(1);
            if n > 1 as u_int {
                break;
            }
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    return n;
}
unsafe extern "C" fn clients_calculate_size(
    mut type_0: ::core::ffi::c_int,
    mut current: ::core::ffi::c_int,
    mut c: *mut client,
    mut s: *mut session,
    mut w: *mut window,
    mut skip_client: Option<
        unsafe extern "C" fn(
            *mut client,
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            *mut session,
            *mut window,
        ) -> ::core::ffi::c_int,
    >,
    mut sx: *mut u_int,
    mut sy: *mut u_int,
    mut xpixel: *mut u_int,
    mut ypixel: *mut u_int,
) -> ::core::ffi::c_int {
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut n: u_int = 0 as u_int;
    if type_0 == WINDOW_SIZE_LARGEST {
        *sx = 0 as u_int;
        *sy = 0 as u_int;
    } else if !w.is_null() && type_0 == WINDOW_SIZE_MANUAL {
        *sx = (*w).manual_sx;
        *sy = (*w).manual_sy;
        log_debug(
            b"%s: manual size %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
            *sx,
            *sy,
        );
    } else {
        *sx = UINT_MAX as u_int;
        *sy = UINT_MAX as u_int;
    }
    *ypixel = 0 as u_int;
    *xpixel = *ypixel;
    if type_0 == WINDOW_SIZE_LATEST && !w.is_null() {
        n = clients_with_window(w);
    }
    if !(type_0 == WINDOW_SIZE_MANUAL) {
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            if loop_0 != c && ignore_client_size(loop_0) != 0 {
                log_debug(
                    b"%s: ignoring %s (1)\0" as *const u8 as *const ::core::ffi::c_char,
                    b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
                    (*loop_0).name,
                );
            } else if loop_0 != c
                && skip_client.expect("non-null function pointer")(loop_0, type_0, current, s, w)
                    != 0
            {
                log_debug(
                    b"%s: skipping %s (1)\0" as *const u8 as *const ::core::ffi::c_char,
                    b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
                    (*loop_0).name,
                );
            } else if type_0 == WINDOW_SIZE_LATEST
                && n > 1 as u_int
                && loop_0 != (*w).latest as *mut client
            {
                log_debug(
                    b"%s: %s is not latest\0" as *const u8 as *const ::core::ffi::c_char,
                    b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
                    (*loop_0).name,
                );
            } else {
                if w.is_null()
                    || control_get_window_size(loop_0, (*w).id, &raw mut cx, &raw mut cy) == 0
                    || cx == 0 as u_int
                    || cy == 0 as u_int
                {
                    cx = (*loop_0).tty.sx;
                    cy = (*loop_0).tty.sy.wrapping_sub(status_line_size(loop_0));
                }
                if type_0 == WINDOW_SIZE_LARGEST {
                    if cx > *sx {
                        *sx = cx;
                    }
                    if cy > *sy {
                        *sy = cy;
                    }
                } else {
                    if cx < *sx {
                        *sx = cx;
                    }
                    if cy < *sy {
                        *sy = cy;
                    }
                }
                if (*loop_0).tty.xpixel > *xpixel && (*loop_0).tty.ypixel > *ypixel {
                    *xpixel = (*loop_0).tty.xpixel;
                    *ypixel = (*loop_0).tty.ypixel;
                }
                log_debug(
                    b"%s: after %s (%ux%u), size is %ux%u\0" as *const u8
                        as *const ::core::ffi::c_char,
                    b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
                    (*loop_0).name,
                    cx,
                    cy,
                    *sx,
                    *sy,
                );
            }
            loop_0 = (*loop_0).entry.tqe_next;
        }
        if *sx != UINT_MAX && *sy != UINT_MAX {
            log_debug(
                b"%s: calculated size %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
                b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
                *sx,
                *sy,
            );
        } else {
            log_debug(
                b"%s: no calculated size\0" as *const u8 as *const ::core::ffi::c_char,
                b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    }
    if !w.is_null() {
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            if !(loop_0 != c && ignore_client_size(loop_0) != 0) {
                if !(loop_0 != c
                    && skip_client.expect("non-null function pointer")(
                        loop_0, type_0, current, s, w,
                    ) != 0)
                {
                    if !(!(*loop_0).flags as ::core::ffi::c_ulonglong & CLIENT_WINDOWSIZECHANGED
                        != 0)
                    {
                        if !(control_get_window_size(loop_0, (*w).id, &raw mut cx, &raw mut cy)
                            == 0)
                        {
                            log_debug(
                                b"%s: %s size for @%u is %ux%u\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                b"clients_calculate_size\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                (*loop_0).name,
                                (*w).id,
                                cx,
                                cy,
                            );
                            if cx != 0 as u_int && *sx > cx {
                                *sx = cx;
                            }
                            if cy != 0 as u_int && *sy > cy {
                                *sy = cy;
                            }
                        }
                    }
                }
            }
            loop_0 = (*loop_0).entry.tqe_next;
        }
    }
    if *sx != UINT_MAX && *sy != UINT_MAX {
        log_debug(
            b"%s: calculated size %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
            *sx,
            *sy,
        );
    } else {
        log_debug(
            b"%s: no calculated size\0" as *const u8 as *const ::core::ffi::c_char,
            b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if type_0 == WINDOW_SIZE_MANUAL {
        log_debug(
            b"%s: type is manual\0" as *const u8 as *const ::core::ffi::c_char,
            b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return (w != NULL as *mut window) as ::core::ffi::c_int;
    }
    if type_0 == WINDOW_SIZE_LARGEST {
        log_debug(
            b"%s: type is largest\0" as *const u8 as *const ::core::ffi::c_char,
            b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return (*sx != 0 as u_int && *sy != 0 as u_int) as ::core::ffi::c_int;
    }
    if type_0 == WINDOW_SIZE_LATEST {
        log_debug(
            b"%s: type is latest\0" as *const u8 as *const ::core::ffi::c_char,
            b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        log_debug(
            b"%s: type is smallest\0" as *const u8 as *const ::core::ffi::c_char,
            b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    return (*sx != UINT_MAX && *sy != UINT_MAX) as ::core::ffi::c_int;
}
unsafe extern "C" fn default_window_size_skip_client(
    mut loop_0: *mut client,
    mut type_0: ::core::ffi::c_int,
    mut current: ::core::ffi::c_int,
    mut s: *mut session,
    mut w: *mut window,
) -> ::core::ffi::c_int {
    if !w.is_null() && session_has((*loop_0).session, w) == 0 {
        return 1 as ::core::ffi::c_int;
    }
    if w.is_null() && (*loop_0).session != s {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn default_window_size(
    mut c: *mut client,
    mut s: *mut session,
    mut w: *mut window,
    mut sx: *mut u_int,
    mut sy: *mut u_int,
    mut xpixel: *mut u_int,
    mut ypixel: *mut u_int,
    mut type_0: ::core::ffi::c_int,
) {
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if type_0 == -(1 as ::core::ffi::c_int) {
        type_0 = options_get_number(
            global_w_options,
            b"window-size\0" as *const u8 as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int;
    }
    if type_0 == WINDOW_SIZE_LATEST && !c.is_null() && ignore_client_size(c) == 0 {
        *sx = (*c).tty.sx;
        *sy = (*c).tty.sy.wrapping_sub(status_line_size(c));
        *xpixel = (*c).tty.xpixel;
        *ypixel = (*c).tty.ypixel;
        log_debug(
            b"%s: using %ux%u from %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"default_window_size\0" as *const u8 as *const ::core::ffi::c_char,
            *sx,
            *sy,
            (*c).name,
        );
    } else {
        if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            c = ::core::ptr::null_mut::<client>();
        }
        if clients_calculate_size(
            type_0,
            0 as ::core::ffi::c_int,
            c,
            s,
            w,
            Some(
                default_window_size_skip_client
                    as unsafe extern "C" fn(
                        *mut client,
                        ::core::ffi::c_int,
                        ::core::ffi::c_int,
                        *mut session,
                        *mut window,
                    ) -> ::core::ffi::c_int,
            ),
            sx,
            sy,
            xpixel,
            ypixel,
        ) == 0
        {
            value = options_get_string(
                (*s).options,
                b"default-size\0" as *const u8 as *const ::core::ffi::c_char,
            );
            if sscanf(
                value,
                b"%ux%u\0" as *const u8 as *const ::core::ffi::c_char,
                sx,
                sy,
            ) != 2 as ::core::ffi::c_int
            {
                *sx = 80 as u_int;
                *sy = 24 as u_int;
            }
            log_debug(
                b"%s: using %ux%u from default-size\0" as *const u8 as *const ::core::ffi::c_char,
                b"default_window_size\0" as *const u8 as *const ::core::ffi::c_char,
                *sx,
                *sy,
            );
        }
    }
    if *sx < WINDOW_MINIMUM as u_int {
        *sx = WINDOW_MINIMUM as u_int;
    }
    if *sx > WINDOW_MAXIMUM as u_int {
        *sx = WINDOW_MAXIMUM as u_int;
    }
    if *sy < WINDOW_MINIMUM as u_int {
        *sy = WINDOW_MINIMUM as u_int;
    }
    if *sy > WINDOW_MAXIMUM as u_int {
        *sy = WINDOW_MAXIMUM as u_int;
    }
    log_debug(
        b"%s: resulting size is %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"default_window_size\0" as *const u8 as *const ::core::ffi::c_char,
        *sx,
        *sy,
    );
}
unsafe extern "C" fn recalculate_size_skip_client(
    mut loop_0: *mut client,
    mut type_0: ::core::ffi::c_int,
    mut current: ::core::ffi::c_int,
    mut s: *mut session,
    mut w: *mut window,
) -> ::core::ffi::c_int {
    if (*(*loop_0).session).curw.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    if current != 0 {
        return ((*(*(*loop_0).session).curw).window != w) as ::core::ffi::c_int;
    }
    return (session_has((*loop_0).session, w) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn recalculate_size(mut w: *mut window, mut now: ::core::ffi::c_int) {
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xpixel: u_int = 0 as u_int;
    let mut ypixel: u_int = 0 as u_int;
    let mut type_0: ::core::ffi::c_int = 0;
    let mut current: ::core::ffi::c_int = 0;
    let mut changed: ::core::ffi::c_int = 0;
    if (*w).active.is_null() {
        return;
    }
    log_debug(
        b"%s: @%u is %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"recalculate_size\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        (*w).sx,
        (*w).sy,
    );
    type_0 = options_get_number(
        (*w).options,
        b"window-size\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    current = options_get_number(
        (*w).options,
        b"aggressive-resize\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    changed = clients_calculate_size(
        type_0,
        current,
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        w,
        Some(
            recalculate_size_skip_client
                as unsafe extern "C" fn(
                    *mut client,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                    *mut session,
                    *mut window,
                ) -> ::core::ffi::c_int,
        ),
        &raw mut sx,
        &raw mut sy,
        &raw mut xpixel,
        &raw mut ypixel,
    );
    if (*w).flags & WINDOW_RESIZE != 0 {
        if now == 0 && changed != 0 && (*w).new_sx == sx && (*w).new_sy == sy {
            changed = 0 as ::core::ffi::c_int;
        }
    } else if now == 0 && changed != 0 && (*w).sx == sx && (*w).sy == sy {
        changed = 0 as ::core::ffi::c_int;
    }
    if changed == 0 {
        log_debug(
            b"%s: @%u no size change\0" as *const u8 as *const ::core::ffi::c_char,
            b"recalculate_size\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
        );
        tty_update_window_offset(w);
        return;
    }
    log_debug(
        b"%s: @%u new size %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"recalculate_size\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        sx,
        sy,
    );
    if now != 0 || type_0 == WINDOW_SIZE_MANUAL {
        resize_window(
            w,
            sx,
            sy,
            xpixel as ::core::ffi::c_int,
            ypixel as ::core::ffi::c_int,
        );
    } else {
        (*w).new_sx = sx;
        (*w).new_sy = sy;
        (*w).new_xpixel = xpixel;
        (*w).new_ypixel = ypixel;
        (*w).flags |= WINDOW_RESIZE;
        tty_update_window_offset(w);
    };
}
#[no_mangle]
pub unsafe extern "C" fn recalculate_sizes() {
    recalculate_sizes_now(0 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn recalculate_sizes_now(mut now: ::core::ffi::c_int) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        (*s).attached = 0 as u_int;
        status_update_cache(s);
        s = sessions_RB_NEXT(s);
    }
    c = clients.tqh_first;
    while !c.is_null() {
        s = (*c).session;
        if !s.is_null() && (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t == 0 {
            (*s).attached = (*s).attached.wrapping_add(1);
        }
        if !(ignore_client_size(c) != 0) {
            if (*c).tty.sy <= (*s).statuslines || (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                (*c).flags |= CLIENT_STATUSOFF as uint64_t;
            } else {
                (*c).flags &= !CLIENT_STATUSOFF as uint64_t;
            }
        }
        c = (*c).entry.tqe_next;
    }
    w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
    while !w.is_null() {
        recalculate_size(w, now);
        w = windows_RB_NEXT(w);
    }
}

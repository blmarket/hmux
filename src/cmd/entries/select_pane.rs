use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::{cmd_get_args, cmd_get_entry};
use crate::src::cmd_find::{cmd_find_from_pane, cmd_find_from_winlink, cmd_find_from_winlink_pane};
use crate::src::cmd_queue::{
    cmdq_error, cmdq_get_current, cmdq_get_target, cmdq_insert_hook, cmdq_print,
};
use crate::src::events::events_fire;
use crate::src::events_payload::{
    event_payload_create, event_payload_set_int, event_payload_set_pane, event_payload_set_string,
    event_payload_set_target, event_payload_set_window,
};
use crate::src::format::format_single_from_target_cstring;
use crate::src::options::{options_get_string, options_set_string};
use crate::src::screen::screen_set_title;
pub use crate::src::server::clients;
use crate::src::server::{
    marked_pane, server_check_marked, server_clear_marked, server_is_marked, server_set_marked,
};
use crate::src::server_fn::{
    server_redraw_client, server_redraw_window, server_redraw_window_borders, server_status_window,
};
use crate::src::session::session_has;
pub use crate::src::shared::events::event_payload;
use crate::src::tty::tty_window_bigger;
use crate::src::window::{
    window_count_panes, window_pane_find_down, window_pane_find_left, window_pane_find_right,
    window_pane_find_up, window_pane_is_floating, window_pane_is_visible, window_pane_next,
    window_pane_previous, window_pane_stack_first, window_pop_zoom, window_push_zoom,
    window_redraw_active_switch,
    window_set_active_pane,
};

use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
pub use crate::src::shared::client::{CLIENT_CONTROL, CLIENT_REDRAWBORDERS, CLIENT_REDRAWSTATUS};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
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
pub use crate::src::shared::options::{options, options_entry};
pub use crate::src::shared::pane::{
    window_pane, window_pane_modes, window_pane_prompt, window_pane_tree_entry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resizes, PANE_INPUTOFF, PANE_REDRAW,
    PANE_STYLECHANGED, PANE_THEMECHANGED,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_entry, window_mode, window_mode_entry, window_winlinks, winlink, winlink_entry,
    winlink_stack, winlinks,
};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[no_mangle]
pub static mut cmd_select_pane_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"select-pane\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"selectp\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"DdegLlMmP:RT:t:UZ\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-DdeLlMmRUZ] [-T title] [-t target-pane]\0" as *const u8
            as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_select_pane_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_last_pane_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"last-pane\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"lastp\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"det:Z\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-deZ] [-t target-window]\0" as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_select_pane_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_select_pane_redraw(mut w: *mut window) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.first();
    while !c.is_null() {
        if !((*c).session.is_null() || (*c).flags & CLIENT_CONTROL as uint64_t != 0) {
            if (*(*(*c).session).curw).window == w && tty_window_bigger(&raw mut (*c).tty) != 0 {
                server_redraw_client(c);
            } else {
                if (*(*(*c).session).curw).window == w {
                    (*c).flags |= CLIENT_REDRAWBORDERS as uint64_t;
                }
                if session_has((*c).session, w) != 0 {
                    (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
                }
            }
        }
        c = clients.next(c);
    }
}
unsafe extern "C" fn cmd_select_pane_marked_pane(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
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
    let mut wl: *mut winlink = (*target).wl;
    let mut wp: *mut window_pane = (*target).wp;
    let mut lwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut mwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut s: *mut session = (*target).s;
    if args_has(args, 'm' as i32 as u_char) != 0 && window_pane_is_visible(wp) == 0 {
        return CMD_RETURN_NORMAL;
    }
    if server_check_marked() != 0 {
        lwp = marked_pane.wp;
    }
    if args_has(args, 'M' as i32 as u_char) != 0 || server_is_marked(s, wl, wp) != 0 {
        server_clear_marked();
    } else {
        server_set_marked(s, wl, wp);
    }
    mwp = marked_pane.wp;
    ep = event_payload_create();
    if !mwp.is_null() {
        cmd_find_from_pane(&raw mut fs, mwp, 0 as ::core::ffi::c_int);
    } else if !lwp.is_null() {
        cmd_find_from_pane(&raw mut fs, lwp, 0 as ::core::ffi::c_int);
    } else {
        cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    }
    event_payload_set_target(ep, &raw mut fs);
    if !mwp.is_null() {
        event_payload_set_pane(
            ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            mwp,
        );
        event_payload_set_pane(
            ep,
            b"new_pane\0" as *const u8 as *const ::core::ffi::c_char,
            mwp,
        );
        event_payload_set_window(
            ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            (*mwp).window as *mut window,
        );
    } else if !lwp.is_null() {
        event_payload_set_pane(
            ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            lwp,
        );
        event_payload_set_window(
            ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            (*lwp).window as *mut window,
        );
    } else {
        event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
        event_payload_set_window(
            ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).window as *mut window,
        );
    }
    if !lwp.is_null() {
        event_payload_set_pane(
            ep,
            b"old_pane\0" as *const u8 as *const ::core::ffi::c_char,
            lwp,
        );
    }
    event_payload_set_int(
        ep,
        b"marked\0" as *const u8 as *const ::core::ffi::c_char,
        (mwp != NULL as *mut window_pane) as ::core::ffi::c_int,
    );
    events_fire(
        b"marked-pane-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
    if !lwp.is_null() {
        (*lwp).flags |= PANE_REDRAW | PANE_STYLECHANGED | PANE_THEMECHANGED;
        server_redraw_window_borders((*lwp).window as *mut window);
        server_status_window((*lwp).window as *mut window);
    }
    if !mwp.is_null() {
        (*mwp).flags |= PANE_REDRAW | PANE_STYLECHANGED | PANE_THEMECHANGED;
        server_redraw_window_borders((*mwp).window as *mut window);
        server_status_window((*mwp).window as *mut window);
    }
    if window_pane_is_floating(wp) != 0 {
        window_redraw_active_switch((*wp).window as *mut window, wp);
        window_set_active_pane((*wp).window as *mut window, wp, 1 as ::core::ffi::c_int);
    }
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_select_pane_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut entry: *const cmd_entry = cmd_get_entry(self_0);
    let mut current: *mut cmd_find_state = cmdq_get_current(item);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
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
    let mut wl: *mut winlink = (*target).wl;
    let mut w: *mut window = (*wl).window;
    let mut s: *mut session = (*target).s;
    let mut wp: *mut window_pane = (*target).wp;
    let mut lastwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut oo: *mut options = (*wp).options;
    let mut style: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut visible: ::core::ffi::c_int = 0;
    let mut Zflag: ::core::ffi::c_int = args_has(args, 'Z' as i32 as u_char);
    if entry == &raw const cmd_last_pane_entry || args_has(args, 'l' as i32 as u_char) != 0 {
        lastwp = window_pane_stack_first(w);
        if lastwp.is_null() && window_count_panes(w, 1 as ::core::ffi::c_int) == 2 as u_int {
            lastwp = window_pane_previous((*w).active);
            if lastwp.is_null() {
                lastwp = window_pane_next((*w).active);
            }
        }
        if lastwp.is_null() {
            cmdq_error(
                item,
                b"no last pane\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
        if args_has(args, 'e' as i32 as u_char) != 0 {
            (*lastwp).flags &= !PANE_INPUTOFF;
            server_redraw_window_borders((*lastwp).window as *mut window);
            server_status_window((*lastwp).window as *mut window);
        } else if args_has(args, 'd' as i32 as u_char) != 0 {
            (*lastwp).flags |= PANE_INPUTOFF;
            server_redraw_window_borders((*lastwp).window as *mut window);
            server_status_window((*lastwp).window as *mut window);
        } else {
            if !(*w).modal.is_null() && lastwp != (*w).modal {
                visible = 1 as ::core::ffi::c_int;
            } else {
                visible = window_pane_is_visible(lastwp);
            }
            if visible == 0 && window_push_zoom(w, 0 as ::core::ffi::c_int, Zflag) != 0 {
                server_redraw_window(w);
            }
            window_redraw_active_switch(w, lastwp);
            if window_set_active_pane(w, lastwp, 1 as ::core::ffi::c_int) != 0 {
                cmd_find_from_winlink(current, wl, 0 as ::core::ffi::c_int);
                cmd_select_pane_redraw(w);
            }
            if visible == 0 && window_pop_zoom(w) != 0 {
                server_redraw_window(w);
            }
        }
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'm' as i32 as u_char) != 0 || args_has(args, 'M' as i32 as u_char) != 0 {
        return cmd_select_pane_marked_pane(self_0, item);
    }
    style = args_get(args, 'P' as i32 as u_char);
    if !style.is_null() {
        o = options_set_string(
            oo,
            b"window-style\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            style,
        );
        if o.is_null() {
            cmdq_error(
                item,
                b"bad style: %s\0" as *const u8 as *const ::core::ffi::c_char,
                style,
            );
            return CMD_RETURN_ERROR;
        }
        options_set_string(
            oo,
            b"window-active-style\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            style,
        );
        (*wp).flags |= PANE_REDRAW | PANE_STYLECHANGED | PANE_THEMECHANGED;
    }
    if args_has(args, 'g' as i32 as u_char) != 0 {
        cmdq_print(
            item,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            options_get_string(
                oo,
                b"window-style\0" as *const u8 as *const ::core::ffi::c_char,
            ),
        );
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'L' as i32 as u_char) != 0 {
        window_push_zoom(w, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        wp = window_pane_find_left(wp);
        window_pop_zoom(w);
    } else if args_has(args, 'R' as i32 as u_char) != 0 {
        window_push_zoom(w, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        wp = window_pane_find_right(wp);
        window_pop_zoom(w);
    } else if args_has(args, 'U' as i32 as u_char) != 0 {
        window_push_zoom(w, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        wp = window_pane_find_up(wp);
        window_pop_zoom(w);
    } else if args_has(args, 'D' as i32 as u_char) != 0 {
        window_push_zoom(w, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        wp = window_pane_find_down(wp);
        window_pop_zoom(w);
    }
    if wp.is_null() {
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'e' as i32 as u_char) != 0 {
        (*wp).flags &= !PANE_INPUTOFF;
        server_redraw_window_borders((*wp).window as *mut window);
        server_status_window((*wp).window as *mut window);
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'd' as i32 as u_char) != 0 {
        (*wp).flags |= PANE_INPUTOFF;
        server_redraw_window_borders((*wp).window as *mut window);
        server_status_window((*wp).window as *mut window);
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'T' as i32 as u_char) != 0 {
        let title = format_single_from_target_cstring(item, args_get(args, 'T' as i32 as u_char));
        if screen_set_title(&raw mut (*wp).base, title.as_ptr(), 0 as ::core::ffi::c_int) != 0 {
            ep = event_payload_create();
            cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
            event_payload_set_target(ep, &raw mut fs);
            event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
            event_payload_set_window(
                ep,
                b"window\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).window as *mut window,
            );
            event_payload_set_string(
                ep,
                b"new_title\0" as *const u8 as *const ::core::ffi::c_char,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                title.as_ptr(),
            );
            events_fire(
                b"pane-title-changed\0" as *const u8 as *const ::core::ffi::c_char,
                ep,
            );
            server_redraw_window_borders((*wp).window as *mut window);
            server_status_window((*wp).window as *mut window);
        }
        return CMD_RETURN_NORMAL;
    }
    if wp == (*w).active {
        return CMD_RETURN_NORMAL;
    }
    if !(*w).modal.is_null() && wp != (*w).modal {
        visible = 1 as ::core::ffi::c_int;
    } else {
        visible = window_pane_is_visible(wp);
    }
    if visible == 0 && window_push_zoom(w, 0 as ::core::ffi::c_int, Zflag) != 0 {
        server_redraw_window(w);
    }
    window_redraw_active_switch(w, wp);
    if window_set_active_pane(w, wp, 1 as ::core::ffi::c_int) != 0 {
        cmd_find_from_winlink_pane(current, wl, wp, 0 as ::core::ffi::c_int);
    }
    cmdq_insert_hook(
        s,
        item,
        current,
        b"after-select-pane\0" as *const u8 as *const ::core::ffi::c_char,
    );
    cmd_select_pane_redraw(w);
    if visible == 0 && window_pop_zoom(w) != 0 {
        server_redraw_window(w);
    }
    return CMD_RETURN_NORMAL;
}

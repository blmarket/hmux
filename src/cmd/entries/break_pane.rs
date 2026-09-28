use crate::src::options::options_owner_ptr;
use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::find::cmd_find_from_session;
use crate::src::cmd::queue::{
    cmdq_error, cmdq_get_state_owned, cmdq_get_source, cmdq_get_target, cmdq_get_target_client,
    cmdq_print,
};
use crate::src::events::events_fire_window;
use crate::src::format::bytes::write_cstr;
use crate::src::format::format_single_cstring;
use crate::src::layout::{
    layout_close_pane, layout_fix_offsets, layout_fix_panes, layout_floating_args_parse,
    layout_init, layout_remove_tile, layout_set_size,
};
use crate::src::names::default_window_name_cstring;
use crate::src::options::{options_get_number, options_set_number, options_set_parent};
use crate::src::server_client::server_client_remove_pane;
use crate::src::server_fn::{
    server_link_window, server_redraw_session, server_redraw_window, server_status_session_group,
    server_unlink_window, server_unzoom_window,
};
use crate::src::session::{session_attach, session_select};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::CMD_FIND_WINDOW_INDEX;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::layout::layout_cell;
use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{PANE_CHANGED, PANE_STYLECHANGED, PANE_THEMECHANGED};
use crate::src::shared::session::session;
use crate::src::shared::window::WINDOW_ZOOMED;
use crate::src::shared::window::{window, winlink};
use crate::src::style::colour::colour_palette_from_option;
use crate::src::tmux::{check_name, clean_name_cstring};
use crate::src::window::{
    window_add_ref, window_count_panes, window_create, window_fire_pane_moved,
    window_get_pane_lines, window_lost_pane, window_pane_is_floating,
    window_pane_list_insert_front, window_pane_list_remove, window_pane_z_insert_front,
    window_pane_z_remove, window_remove_ref, window_replace_name, window_set_active_pane,
    window_set_name, winlink_find_by_index, winlink_find_by_window, winlink_shuffle_up,
};
use crate::src::window_border::window_set_fill_cells;

pub const BREAK_PANE_TEMPLATE: [::core::ffi::c_char; 46] = unsafe {
    ::core::mem::transmute::<[u8; 46], [::core::ffi::c_char; 46]>(
        *b"#{session_name}:#{window_index}.#{pane_index}\0",
    )
};
use std::ffi::CStr;

pub static cmd_break_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"break-pane",
        alias: Some(c"breakp"),
        args: args_parse {
            template: c"abdPF:n:s:t:Wx:X:y:Y:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-abdPW] [-F format] [-n window-name] [-s src-pane] [-t dst-window] [-x width] [-y height] [-X x-position] [-Y y-position]",
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: CMD_FIND_WINDOW_INDEX,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_break_pane_exec
                as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe fn cmd_break_pane_float(
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut w: *mut window,
    mut wp: *mut window_pane,
) -> cmd_retval {
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    let mut lines: pane_lines = window_get_pane_lines(w);
    let mut fg: *mut layout_geometry = &raw mut (*lc).fg;
    if window_pane_is_floating(&*wp) != 0 {
        cmdq_error(item, |out| out.write_all(b"pane is already floating"));
        return CMD_RETURN_ERROR;
    }
    if (*w).flags & WINDOW_ZOOMED != 0 {
        cmdq_error(item, |out| {
            out.write_all(b"can't float a pane while window is zoomed")
        });
        return CMD_RETURN_ERROR;
    }
    if let Err(cause) = layout_floating_args_parse(item, args, lines, w, fg) {
        cmdq_error(item, |out| {
            out.write_all(b"failed to float pane: ")?;
            write_cstr(out, cause.as_ptr())
        });
        return CMD_RETURN_ERROR;
    }
    layout_remove_tile(w, lc);
    layout_set_size(lc, (*fg).sx, (*fg).sy, (*fg).xoff, (*fg).yoff);
    (*lc).flags |= LAYOUT_CELL_FLOATING;
    window_pane_z_remove(w, wp);
    window_pane_z_insert_front(w, wp);
    if args_has(args, 'd' as i32 as u_char) == 0 {
        window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
    }
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
    return CMD_RETURN_NORMAL;
}
unsafe fn cmd_break_pane_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let current = cmdq_get_state_owned(item);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut source: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_source_mut(&mut *item);
    let tc_owner = cmdq_get_target_client(item);
    let mut tc: *mut client = tc_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let mut wl: *mut winlink = (*source).wl_ptr();
    let mut src_s: *mut session = (*source).s_ptr();
    let mut dst_s: *mut session = (*target).s_ptr();
    let mut wp: *mut window_pane = (*source).wp_ptr();
    let mut w: *mut window = (*wl).window_ptr();
    let mut old_w: *mut window = w;
    let _cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = (*target).idx;
    let mut before: ::core::ffi::c_int = 0;
    let mut old_idx: ::core::ffi::c_int = (*wl).idx;
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = args_get(&*(args), 'n' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
    if (*w).modal.ptr_eq(&(*wp).observer) {
        cmdq_error(item, |out| out.write_all(b"pane is modal"));
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'W' as i32 as u_char) != 0 {
        return cmd_break_pane_float(item, args, w, wp);
    }
    if !name.is_null() && !check_name(CStr::from_ptr(name)) {
        cmdq_error(item, |out| {
            out.write_all(b"invalid window name: ")?;
            write_cstr(out, name)
        });
        return CMD_RETURN_ERROR;
    }
    before = args_has(args, 'b' as i32 as u_char);
    if args_has(args, 'a' as i32 as u_char) != 0 || before != 0 {
        if !(*target).wl_ptr().is_null() {
            idx = winlink_shuffle_up(dst_s, (*target).wl_ptr(), before);
        } else {
            idx = winlink_shuffle_up(dst_s, (*dst_s).curw, before);
        }
        if idx == -(1 as ::core::ffi::c_int) {
            return CMD_RETURN_ERROR;
        }
    }
    server_unzoom_window(w);
    if window_count_panes(&*w, 1 as ::core::ffi::c_int) == 1 as u_int {
        if let Err(link_error) = server_link_window(
            src_s,
            wl,
            dst_s,
            idx,
            0 as ::core::ffi::c_int,
            (args_has(args, 'd' as i32 as u_char) == 0) as ::core::ffi::c_int,
        ) {
            cmdq_error(item, |out| write_cstr(out, link_error.as_ptr()));
            return CMD_RETURN_ERROR;
        }
        if !name.is_null() {
            window_set_name(w, name, 0 as ::core::ffi::c_int);
            options_set_number(
                options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
                b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_longlong,
            );
        }
        server_unlink_window(src_s, wl);
        wl = winlink_find_by_window(&raw mut (*dst_s).windows, w);
        if wl.is_null() {
            return CMD_RETURN_ERROR;
        }
        window_fire_pane_moved(wp, old_w, old_idx, w, (*wl).idx);
    } else {
        if idx != -(1 as ::core::ffi::c_int)
            && !winlink_find_by_index(&raw mut (*dst_s).windows, idx).is_null()
        {
            cmdq_error(item, |out| write!(out, "index in use: {}", (idx) as i32));
            return CMD_RETURN_ERROR;
        }
        server_client_remove_pane(wp);
        // Select a replacement while the departing pane still has neighbors.
        window_lost_pane(w, wp);
        window_pane_list_remove(w, wp);
        window_pane_z_remove(w, wp);
        layout_close_pane(wp);
        let window = window_create((*w).sx, (*w).sy, (*w).xpixel, (*w).ypixel);
        (*wp).window = window.as_ptr();
        w = (*wp).window as *mut window;
        // window_create supplied the temporary reference released after attach.
        options_set_parent(options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options), options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options));
        (*wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
        window_pane_list_insert_front(w, wp);
        window_pane_z_insert_front(w, wp);
        (*w).active = wp;
        (*w).latest = tc.as_ref().map_or_else(std::rc::Weak::new, |client| client.observer.clone());
        if name.is_null() {
            drop(window_replace_name(w, default_window_name_cstring(&*w)));
        } else {
            let cleaned = clean_name_cstring(std::ffi::CStr::from_ptr(name), 0)
                .expect("check_name validated the explicit window name");
            drop(window_replace_name(w, cleaned));
            options_set_number(
                options_owner_ptr(&mut (*w).options).map_or(std::ptr::null_mut(), |options| options),
                b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_longlong,
            );
        }
        window_set_fill_cells(w);
        if idx == -(1 as ::core::ffi::c_int) {
            idx = (-(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong
                - options_get_number(
                    options_owner_ptr(&mut (*dst_s).options).map_or(std::ptr::null_mut(), |options| options),
                    b"base-index\0" as *const u8 as *const ::core::ffi::c_char,
                )) as ::core::ffi::c_int;
        }
        wl = match session_attach(dst_s, w, idx) {
            Ok(wl) => wl,
            Err(error) => {
                cmdq_error(item, |out| write_cstr(out, error.as_ptr()));
                return CMD_RETURN_ERROR;
            }
        };
        layout_init(w, wp);
        (*wp).flags |= PANE_CHANGED;
        colour_palette_from_option(Some(&mut (*wp).palette), options_owner_ptr(&mut (*wp).options).map_or(std::ptr::null_mut(), |options| options));
        drop(window);
        events_fire_window(
            b"window-created\0" as *const u8 as *const ::core::ffi::c_char,
            w,
        );
        window_fire_pane_moved(wp, old_w, old_idx, w, (*wl).idx);
        if args_has(args, 'd' as i32 as u_char) == 0 {
            session_select(dst_s, (*wl).idx);
            cmd_find_from_session(&mut *current.current.borrow_mut(), dst_s, 0 as ::core::ffi::c_int);
        }
        server_redraw_session(src_s);
        if src_s != dst_s {
            server_redraw_session(dst_s);
        }
        server_status_session_group(src_s);
        if src_s != dst_s {
            server_status_session_group(dst_s);
        }
    }
    if args_has(args, 'P' as i32 as u_char) != 0 {
        template = args_get(&*(args), 'F' as i32 as u_char).map_or(std::ptr::null(), |value| value.as_ptr());
        if template.is_null() {
            template = BREAK_PANE_TEMPLATE.as_ptr();
        }
        let cp = format_single_cstring(item, template, tc, dst_s, wl, wp);
        cmdq_print(item, |out| write_cstr(out, cp.as_ptr()));
    }
    return CMD_RETURN_NORMAL;
}

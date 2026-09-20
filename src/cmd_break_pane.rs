pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_data};
pub use crate::src::shared::options::{options, options_entry};
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
pub use crate::src::shared::window::{WINDOW_ZOOMED};
pub use crate::src::shared::pane::{
    PANE_CHANGED, PANE_STYLECHANGED, PANE_THEMECHANGED, window_pane_offset, window_pane_resize,
    window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::command::{CMD_FIND_WINDOW_INDEX};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::layout::*;
use crate::src::shared::arguments::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;
extern "C" {

    fn free(__ptr: *mut ::core::ffi::c_void);
    fn clean_name(_: *const ::core::ffi::c_char, _: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn check_name(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn format_single(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut ::core::ffi::c_char;
    fn events_fire_window(_: *const ::core::ffi::c_char, _: *mut window);
    fn options_set_parent(_: *mut options, _: *mut options);
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn options_set_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
    ) -> *mut options_entry;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn cmd_find_from_session(_: *mut cmd_find_state, _: *mut session, _: ::core::ffi::c_int);
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_source(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_current(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_print(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn server_client_remove_pane(_: *mut window_pane);
    fn server_redraw_session(_: *mut session);
    fn server_status_session_group(_: *mut session);
    fn server_redraw_window(_: *mut window);
    fn server_link_window(
        _: *mut session,
        _: *mut winlink,
        _: *mut session,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn server_unlink_window(_: *mut session, _: *mut winlink);
    fn server_unzoom_window(_: *mut window);
    fn colour_palette_from_option(_: *mut colour_palette, _: *mut options);
    fn winlink_find_by_index(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn winlink_find_by_window(_: *mut winlinks, _: *mut window) -> *mut winlink;
    fn window_create(_: u_int, _: u_int, _: u_int, _: u_int) -> *mut window;
    fn window_set_active_pane(
        _: *mut window,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn window_fire_pane_moved(
        _: *mut window_pane,
        _: *mut window,
        _: ::core::ffi::c_int,
        _: *mut window,
        _: ::core::ffi::c_int,
    );
    fn window_lost_pane(_: *mut window, _: *mut window_pane);
    fn window_count_panes(_: *mut window, _: ::core::ffi::c_int) -> u_int;
    fn window_set_name(_: *mut window, _: *const ::core::ffi::c_char, _: ::core::ffi::c_int);
    fn window_add_ref(_: *mut window, _: *const ::core::ffi::c_char);
    fn window_remove_ref(_: *mut window, _: *const ::core::ffi::c_char);
    fn winlink_shuffle_up(
        _: *mut session,
        _: *mut winlink,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn window_get_pane_lines(_: *mut window) -> pane_lines;
    fn window_pane_is_floating(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_set_fill_cells(_: *mut window);
    fn layout_set_size(
        _: *mut layout_cell,
        _: u_int,
        _: u_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn layout_fix_offsets(_: *mut window);
    fn layout_fix_panes(_: *mut window, _: *mut window_pane);
    fn layout_init(_: *mut window, _: *mut window_pane);
    fn layout_close_pane(_: *mut window_pane);
    fn layout_floating_args_parse(
        _: *mut cmdq_item,
        _: *mut args,
        _: pane_lines,
        _: *mut window,
        _: *mut layout_geometry,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn layout_remove_tile(_: *mut window, _: *mut layout_cell) -> ::core::ffi::c_int;
    fn default_window_name(_: *mut window) -> *mut ::core::ffi::c_char;
    fn session_attach(
        _: *mut session,
        _: *mut window,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut winlink;
    fn session_select(_: *mut session, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
}

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

pub const BREAK_PANE_TEMPLATE: [::core::ffi::c_char; 46] = unsafe {
    ::core::mem::transmute::<[u8; 46], [::core::ffi::c_char; 46]>(
        *b"#{session_name}:#{window_index}.#{pane_index}\0",
    )
};
#[no_mangle]
pub static mut cmd_break_pane_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"break-pane\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"breakp\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"abdPF:n:s:t:Wx:X:y:Y:\0" as *const u8
                as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-abdPW] [-F format] [-n window-name] [-s src-pane] [-t dst-window] [-x width] [-y height] [-X x-position] [-Y y-position]\0"
            as *const u8 as *const ::core::ffi::c_char,
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
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_break_pane_float(
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut w: *mut window,
    mut wp: *mut window_pane,
) -> cmd_retval {
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut lines: pane_lines = window_get_pane_lines(w);
    let mut fg: *mut layout_geometry = &raw mut (*lc).fg;
    if window_pane_is_floating(wp) != 0 {
        cmdq_error(
            item,
            b"pane is already floating\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if (*w).flags & WINDOW_ZOOMED != 0 {
        cmdq_error(
            item,
            b"can't float a pane while window is zoomed\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if layout_floating_args_parse(item, args, lines, w, fg, &raw mut cause)
        != 0 as ::core::ffi::c_int
    {
        cmdq_error(
            item,
            b"failed to float pane: %s\0" as *const u8 as *const ::core::ffi::c_char,
            cause,
        );
        free(cause as *mut ::core::ffi::c_void);
        return CMD_RETURN_ERROR;
    }
    layout_remove_tile(w, lc);
    layout_set_size(lc, (*fg).sx, (*fg).sy, (*fg).xoff, (*fg).yoff);
    (*lc).flags |= LAYOUT_CELL_FLOATING;
    if !(*wp).zentry.tqe_next.is_null() {
        (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
    } else {
        (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
    }
    *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
    (*wp).zentry.tqe_next = (*w).z_index.tqh_first;
    if !(*wp).zentry.tqe_next.is_null() {
        (*(*w).z_index.tqh_first).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
    } else {
        (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
    }
    (*w).z_index.tqh_first = wp;
    (*wp).zentry.tqe_prev = &raw mut (*w).z_index.tqh_first;
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
unsafe extern "C" fn cmd_break_pane_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut current: *mut cmd_find_state = cmdq_get_current(item);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut source: *mut cmd_find_state = cmdq_get_source(item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut wl: *mut winlink = (*source).wl;
    let mut src_s: *mut session = (*source).s;
    let mut dst_s: *mut session = (*target).s;
    let mut wp: *mut window_pane = (*source).wp;
    let mut w: *mut window = (*wl).window;
    let mut old_w: *mut window = w;
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = (*target).idx;
    let mut before: ::core::ffi::c_int = 0;
    let mut old_idx: ::core::ffi::c_int = (*wl).idx;
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = args_get(args, 'n' as i32 as u_char);
    if wp == (*w).modal {
        cmdq_error(
            item,
            b"pane is modal\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'W' as i32 as u_char) != 0 {
        return cmd_break_pane_float(item, args, w, wp);
    }
    if !name.is_null() && check_name(name) == 0 {
        cmdq_error(
            item,
            b"invalid window name: %s\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
        return CMD_RETURN_ERROR;
    }
    before = args_has(args, 'b' as i32 as u_char);
    if args_has(args, 'a' as i32 as u_char) != 0 || before != 0 {
        if !(*target).wl.is_null() {
            idx = winlink_shuffle_up(dst_s, (*target).wl, before);
        } else {
            idx = winlink_shuffle_up(dst_s, (*dst_s).curw, before);
        }
        if idx == -(1 as ::core::ffi::c_int) {
            return CMD_RETURN_ERROR;
        }
    }
    server_unzoom_window(w);
    if window_count_panes(w, 1 as ::core::ffi::c_int) == 1 as u_int {
        if server_link_window(
            src_s,
            wl,
            dst_s,
            idx,
            0 as ::core::ffi::c_int,
            (args_has(args, 'd' as i32 as u_char) == 0) as ::core::ffi::c_int,
            &raw mut cause,
        ) != 0 as ::core::ffi::c_int
        {
            cmdq_error(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
        if !name.is_null() {
            window_set_name(w, name, 0 as ::core::ffi::c_int);
            options_set_number(
                (*w).options,
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
            cmdq_error(
                item,
                b"index in use: %d\0" as *const u8 as *const ::core::ffi::c_char,
                idx,
            );
            return CMD_RETURN_ERROR;
        }
        if !(*wp).entry.tqe_next.is_null() {
            (*(*wp).entry.tqe_next).entry.tqe_prev = (*wp).entry.tqe_prev;
        } else {
            (*w).panes.tqh_last = (*wp).entry.tqe_prev;
        }
        *(*wp).entry.tqe_prev = (*wp).entry.tqe_next;
        if !(*wp).zentry.tqe_next.is_null() {
            (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
        } else {
            (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
        }
        *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
        server_client_remove_pane(wp);
        window_lost_pane(w, wp);
        layout_close_pane(wp);
        (*wp).window = window_create((*w).sx, (*w).sy, (*w).xpixel, (*w).ypixel) as *mut window;
        w = (*wp).window as *mut window;
        window_add_ref(
            w,
            b"cmd_break_pane_exec\0" as *const u8 as *const ::core::ffi::c_char,
        );
        options_set_parent((*wp).options, (*w).options);
        (*wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
        (*wp).entry.tqe_next = (*w).panes.tqh_first;
        if !(*wp).entry.tqe_next.is_null() {
            (*(*w).panes.tqh_first).entry.tqe_prev = &raw mut (*wp).entry.tqe_next;
        } else {
            (*w).panes.tqh_last = &raw mut (*wp).entry.tqe_next;
        }
        (*w).panes.tqh_first = wp;
        (*wp).entry.tqe_prev = &raw mut (*w).panes.tqh_first;
        (*wp).zentry.tqe_next = (*w).z_index.tqh_first;
        if !(*wp).zentry.tqe_next.is_null() {
            (*(*w).z_index.tqh_first).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
        } else {
            (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
        }
        (*w).z_index.tqh_first = wp;
        (*wp).zentry.tqe_prev = &raw mut (*w).z_index.tqh_first;
        (*w).active = wp;
        (*w).latest = tc as *mut ::core::ffi::c_void;
        free((*w).name as *mut ::core::ffi::c_void);
        if name.is_null() {
            (*w).name = default_window_name(w);
        } else {
            (*w).name = clean_name(name, 0 as ::core::ffi::c_int);
            options_set_number(
                (*w).options,
                b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_longlong,
            );
        }
        window_set_fill_cells(w);
        if idx == -(1 as ::core::ffi::c_int) {
            idx = (-(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong
                - options_get_number(
                    (*dst_s).options,
                    b"base-index\0" as *const u8 as *const ::core::ffi::c_char,
                )) as ::core::ffi::c_int;
        }
        wl = session_attach(dst_s, w, idx, &raw mut cause);
        layout_init(w, wp);
        (*wp).flags |= PANE_CHANGED;
        colour_palette_from_option(&raw mut (*wp).palette, (*wp).options);
        window_remove_ref(
            w,
            b"cmd_break_pane_exec\0" as *const u8 as *const ::core::ffi::c_char,
        );
        events_fire_window(
            b"window-created\0" as *const u8 as *const ::core::ffi::c_char,
            w,
        );
        window_fire_pane_moved(wp, old_w, old_idx, w, (*wl).idx);
        if args_has(args, 'd' as i32 as u_char) == 0 {
            session_select(dst_s, (*wl).idx);
            cmd_find_from_session(current, dst_s, 0 as ::core::ffi::c_int);
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
        template = args_get(args, 'F' as i32 as u_char);
        if template.is_null() {
            template = BREAK_PANE_TEMPLATE.as_ptr();
        }
        cp = format_single(item, template, tc, dst_s, wl, wp);
        cmdq_print(item, b"%s\0" as *const u8 as *const ::core::ffi::c_char, cp);
        free(cp as *mut ::core::ffi::c_void);
    }
    return CMD_RETURN_NORMAL;
}

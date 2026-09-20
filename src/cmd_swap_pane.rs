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
pub use crate::src::shared::pane::{
    PANE_STYLECHANGED, PANE_THEMECHANGED, window_pane_offset, window_pane_resize,
    window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::command::{CMD_FIND_DEFAULT_MARKED};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
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

    fn events_fire_window(_: *const ::core::ffi::c_char, _: *mut window);
    fn options_set_parent(_: *mut options, _: *mut options);
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_source(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn server_client_remove_pane(_: *mut window_pane);
    fn server_redraw_window(_: *mut window);
    fn colour_palette_from_option(_: *mut colour_palette, _: *mut options);
    fn redraw_invalidate_scene(_: *mut window);
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
    fn window_push_zoom(
        _: *mut window,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn window_pop_zoom(_: *mut window) -> ::core::ffi::c_int;
    fn window_pane_resize(_: *mut window_pane, _: u_int, _: u_int);
    fn window_pane_stack_remove(_: *mut window_panes, _: *mut window_pane);
    fn window_pane_is_floating(_: *mut window_pane) -> ::core::ffi::c_int;
    fn layout_cell_is_tiled(_: *mut layout_cell) -> ::core::ffi::c_int;
    fn layout_fix_panes(_: *mut window, _: *mut window_pane);
}

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

#[no_mangle]
pub static mut cmd_swap_pane_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"swap-pane\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"swapp\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"dDs:t:UZ\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-dDUZ] [-s src-pane] [-t dst-pane]\0" as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_DEFAULT_MARKED,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_swap_pane_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_swap_pane_next_tiled_pane(mut wp: *mut window_pane) -> *mut window_pane {
    while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
        wp = (*wp).entry.tqe_next;
    }
    return wp;
}
unsafe extern "C" fn cmd_swap_pane_prev_tiled_pane(mut wp: *mut window_pane) -> *mut window_pane {
    while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
        wp = *(*((*wp).entry.tqe_prev as *mut window_panes)).tqh_last;
    }
    return wp;
}
unsafe extern "C" fn cmd_swap_pane_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut source: *mut cmd_find_state = cmdq_get_source(item);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut src_w: *mut window = ::core::ptr::null_mut::<window>();
    let mut dst_w: *mut window = ::core::ptr::null_mut::<window>();
    let mut tmp_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut src_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut dst_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut src_lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut dst_lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xoff: u_int = 0;
    let mut yoff: u_int = 0;
    let mut src_idx: ::core::ffi::c_int = 0;
    let mut dst_idx: ::core::ffi::c_int = 0;
    dst_w = (*(*target).wl).window;
    dst_wp = (*target).wp;
    dst_idx = (*(*target).wl).idx;
    src_w = (*(*source).wl).window;
    src_wp = (*source).wp;
    src_idx = (*(*source).wl).idx;
    if src_wp == (*src_w).modal || dst_wp == (*dst_w).modal {
        cmdq_error(
            item,
            b"pane is modal\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if window_push_zoom(
        dst_w,
        0 as ::core::ffi::c_int,
        args_has(args, 'Z' as i32 as u_char),
    ) != 0
    {
        server_redraw_window(dst_w);
    }
    if args_has(args, 'D' as i32 as u_char) != 0 {
        if window_pane_is_floating(dst_wp) != 0 {
            cmdq_error(
                item,
                b"cannot swap down on floating pane\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
        src_w = dst_w;
        src_wp = (*dst_wp).entry.tqe_next;
        src_wp = cmd_swap_pane_next_tiled_pane(src_wp);
        if src_wp.is_null() {
            src_wp = (*dst_w).panes.tqh_first;
            src_wp = cmd_swap_pane_next_tiled_pane(src_wp);
        }
    } else if args_has(args, 'U' as i32 as u_char) != 0 {
        if window_pane_is_floating(dst_wp) != 0 {
            cmdq_error(
                item,
                b"cannot swap up on floating pane\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
        src_w = dst_w;
        src_wp = *(*((*dst_wp).entry.tqe_prev as *mut window_panes)).tqh_last;
        src_wp = cmd_swap_pane_prev_tiled_pane(src_wp);
        if src_wp.is_null() {
            src_wp = *(*((*dst_w).panes.tqh_last as *mut window_panes)).tqh_last;
            src_wp = cmd_swap_pane_prev_tiled_pane(src_wp);
        }
    }
    if src_w != dst_w
        && window_push_zoom(
            src_w,
            0 as ::core::ffi::c_int,
            args_has(args, 'Z' as i32 as u_char),
        ) != 0
    {
        server_redraw_window(src_w);
    }
    if !(src_wp == dst_wp) {
        server_client_remove_pane(src_wp);
        server_client_remove_pane(dst_wp);
        tmp_wp = *(*((*dst_wp).entry.tqe_prev as *mut window_panes)).tqh_last;
        if !(*dst_wp).entry.tqe_next.is_null() {
            (*(*dst_wp).entry.tqe_next).entry.tqe_prev = (*dst_wp).entry.tqe_prev;
        } else {
            (*dst_w).panes.tqh_last = (*dst_wp).entry.tqe_prev;
        }
        *(*dst_wp).entry.tqe_prev = (*dst_wp).entry.tqe_next;
        (*dst_wp).entry.tqe_next = (*src_wp).entry.tqe_next;
        if !(*dst_wp).entry.tqe_next.is_null() {
            (*(*dst_wp).entry.tqe_next).entry.tqe_prev = &raw mut (*dst_wp).entry.tqe_next;
        } else {
            (*src_w).panes.tqh_last = &raw mut (*dst_wp).entry.tqe_next;
        }
        (*dst_wp).entry.tqe_prev = (*src_wp).entry.tqe_prev;
        *(*dst_wp).entry.tqe_prev = dst_wp;
        if tmp_wp == src_wp {
            tmp_wp = dst_wp;
        }
        if tmp_wp.is_null() {
            (*src_wp).entry.tqe_next = (*dst_w).panes.tqh_first;
            if !(*src_wp).entry.tqe_next.is_null() {
                (*(*dst_w).panes.tqh_first).entry.tqe_prev = &raw mut (*src_wp).entry.tqe_next;
            } else {
                (*dst_w).panes.tqh_last = &raw mut (*src_wp).entry.tqe_next;
            }
            (*dst_w).panes.tqh_first = src_wp;
            (*src_wp).entry.tqe_prev = &raw mut (*dst_w).panes.tqh_first;
        } else {
            (*src_wp).entry.tqe_next = (*tmp_wp).entry.tqe_next;
            if !(*src_wp).entry.tqe_next.is_null() {
                (*(*src_wp).entry.tqe_next).entry.tqe_prev = &raw mut (*src_wp).entry.tqe_next;
            } else {
                (*dst_w).panes.tqh_last = &raw mut (*src_wp).entry.tqe_next;
            }
            (*tmp_wp).entry.tqe_next = src_wp;
            (*src_wp).entry.tqe_prev = &raw mut (*tmp_wp).entry.tqe_next;
        }
        tmp_wp = *(*((*dst_wp).zentry.tqe_prev as *mut window_panes)).tqh_last;
        if !(*dst_wp).zentry.tqe_next.is_null() {
            (*(*dst_wp).zentry.tqe_next).zentry.tqe_prev = (*dst_wp).zentry.tqe_prev;
        } else {
            (*dst_w).z_index.tqh_last = (*dst_wp).zentry.tqe_prev;
        }
        *(*dst_wp).zentry.tqe_prev = (*dst_wp).zentry.tqe_next;
        (*dst_wp).zentry.tqe_next = (*src_wp).zentry.tqe_next;
        if !(*dst_wp).zentry.tqe_next.is_null() {
            (*(*dst_wp).zentry.tqe_next).zentry.tqe_prev = &raw mut (*dst_wp).zentry.tqe_next;
        } else {
            (*src_w).z_index.tqh_last = &raw mut (*dst_wp).zentry.tqe_next;
        }
        (*dst_wp).zentry.tqe_prev = (*src_wp).zentry.tqe_prev;
        *(*dst_wp).zentry.tqe_prev = dst_wp;
        if tmp_wp == src_wp {
            tmp_wp = dst_wp;
        }
        if tmp_wp.is_null() {
            (*src_wp).zentry.tqe_next = (*dst_w).z_index.tqh_first;
            if !(*src_wp).zentry.tqe_next.is_null() {
                (*(*dst_w).z_index.tqh_first).zentry.tqe_prev = &raw mut (*src_wp).zentry.tqe_next;
            } else {
                (*dst_w).z_index.tqh_last = &raw mut (*src_wp).zentry.tqe_next;
            }
            (*dst_w).z_index.tqh_first = src_wp;
            (*src_wp).zentry.tqe_prev = &raw mut (*dst_w).z_index.tqh_first;
        } else {
            (*src_wp).zentry.tqe_next = (*tmp_wp).zentry.tqe_next;
            if !(*src_wp).zentry.tqe_next.is_null() {
                (*(*src_wp).zentry.tqe_next).zentry.tqe_prev = &raw mut (*src_wp).zentry.tqe_next;
            } else {
                (*dst_w).z_index.tqh_last = &raw mut (*src_wp).zentry.tqe_next;
            }
            (*tmp_wp).zentry.tqe_next = src_wp;
            (*src_wp).zentry.tqe_prev = &raw mut (*tmp_wp).zentry.tqe_next;
        }
        src_lc = (*src_wp).layout_cell as *mut layout_cell;
        dst_lc = (*dst_wp).layout_cell as *mut layout_cell;
        (*src_lc).wp = dst_wp;
        (*dst_wp).layout_cell = src_lc as *mut layout_cell;
        (*dst_lc).wp = src_wp;
        (*src_wp).layout_cell = dst_lc as *mut layout_cell;
        (*src_wp).window = dst_w as *mut window;
        options_set_parent((*src_wp).options, (*dst_w).options);
        (*src_wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
        (*dst_wp).window = src_w as *mut window;
        options_set_parent((*dst_wp).options, (*src_w).options);
        (*dst_wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
        sx = (*src_wp).sx;
        sy = (*src_wp).sy;
        xoff = (*src_wp).xoff as u_int;
        yoff = (*src_wp).yoff as u_int;
        (*src_wp).xoff = (*dst_wp).xoff;
        (*src_wp).yoff = (*dst_wp).yoff;
        window_pane_resize(src_wp, (*dst_wp).sx, (*dst_wp).sy);
        (*dst_wp).xoff = xoff as ::core::ffi::c_int;
        (*dst_wp).yoff = yoff as ::core::ffi::c_int;
        window_pane_resize(dst_wp, sx, sy);
        if args_has(args, 'd' as i32 as u_char) == 0 {
            if src_w != dst_w {
                window_set_active_pane(src_w, dst_wp, 1 as ::core::ffi::c_int);
                window_set_active_pane(dst_w, src_wp, 1 as ::core::ffi::c_int);
            } else {
                tmp_wp = dst_wp;
                window_set_active_pane(src_w, tmp_wp, 1 as ::core::ffi::c_int);
            }
        } else {
            if (*src_w).active == src_wp {
                window_set_active_pane(src_w, dst_wp, 1 as ::core::ffi::c_int);
            }
            if (*dst_w).active == dst_wp {
                window_set_active_pane(dst_w, src_wp, 1 as ::core::ffi::c_int);
            }
        }
        if src_w != dst_w {
            window_pane_stack_remove(&raw mut (*src_w).last_panes, src_wp);
            window_pane_stack_remove(&raw mut (*dst_w).last_panes, dst_wp);
            colour_palette_from_option(&raw mut (*src_wp).palette, (*src_wp).options);
            colour_palette_from_option(&raw mut (*dst_wp).palette, (*dst_wp).options);
            layout_fix_panes(src_w, ::core::ptr::null_mut::<window_pane>());
            redraw_invalidate_scene(src_w);
            server_redraw_window(src_w);
        }
        layout_fix_panes(dst_w, ::core::ptr::null_mut::<window_pane>());
        redraw_invalidate_scene(dst_w);
        server_redraw_window(dst_w);
        if src_w != dst_w {
            window_fire_pane_moved(src_wp, src_w, src_idx, dst_w, dst_idx);
            window_fire_pane_moved(dst_wp, dst_w, dst_idx, src_w, src_idx);
        }
        events_fire_window(
            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
            src_w,
        );
        if src_w != dst_w {
            events_fire_window(
                b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
                dst_w,
            );
        }
    }
    if window_pop_zoom(src_w) != 0 {
        server_redraw_window(src_w);
    }
    if src_w != dst_w && window_pop_zoom(dst_w) != 0 {
        server_redraw_window(dst_w);
    }
    return CMD_RETURN_NORMAL;
}

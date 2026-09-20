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

    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn cmd_find_from_winlink_pane(
        _: *mut cmd_find_state,
        _: *mut winlink,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    );
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_current(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn server_redraw_window(_: *mut window);
    fn redraw_invalidate_scene(_: *mut window);
    fn window_set_active_pane(
        _: *mut window,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn window_push_zoom(
        _: *mut window,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn window_pop_zoom(_: *mut window) -> ::core::ffi::c_int;
    fn window_pane_resize(_: *mut window_pane, _: u_int, _: u_int);
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

#[no_mangle]
pub static mut cmd_rotate_window_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"rotate-window\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"rotatew\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"Dt:UZ\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-DUZ] [-t target-window]\0" as *const u8 as *const ::core::ffi::c_char,
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
            cmd_rotate_window_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_rotate_window_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut current: *mut cmd_find_state = cmdq_get_current(item);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut wl: *mut winlink = (*target).wl;
    let mut w: *mut window = (*wl).window;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wp2: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xoff: u_int = 0;
    let mut yoff: u_int = 0;
    window_push_zoom(
        w,
        0 as ::core::ffi::c_int,
        args_has(args, 'Z' as i32 as u_char),
    );
    if args_has(args, 'D' as i32 as u_char) != 0 {
        wp = *(*((*w).panes.tqh_last as *mut window_panes)).tqh_last;
        if !(*wp).entry.tqe_next.is_null() {
            (*(*wp).entry.tqe_next).entry.tqe_prev = (*wp).entry.tqe_prev;
        } else {
            (*w).panes.tqh_last = (*wp).entry.tqe_prev;
        }
        *(*wp).entry.tqe_prev = (*wp).entry.tqe_next;
        (*wp).entry.tqe_next = (*w).panes.tqh_first;
        if !(*wp).entry.tqe_next.is_null() {
            (*(*w).panes.tqh_first).entry.tqe_prev = &raw mut (*wp).entry.tqe_next;
        } else {
            (*w).panes.tqh_last = &raw mut (*wp).entry.tqe_next;
        }
        (*w).panes.tqh_first = wp;
        (*wp).entry.tqe_prev = &raw mut (*w).panes.tqh_first;
        lc = (*wp).layout_cell as *mut layout_cell;
        xoff = (*wp).xoff as u_int;
        yoff = (*wp).yoff as u_int;
        sx = (*wp).sx;
        sy = (*wp).sy;
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            wp2 = (*wp).entry.tqe_next;
            if wp2.is_null() {
                break;
            }
            (*wp).layout_cell = (*wp2).layout_cell;
            if !(*wp).layout_cell.is_null() {
                (*(*wp).layout_cell).wp = wp;
            }
            (*wp).xoff = (*wp2).xoff;
            (*wp).yoff = (*wp2).yoff;
            window_pane_resize(wp, (*wp2).sx, (*wp2).sy);
            wp = (*wp).entry.tqe_next;
        }
        (*wp).layout_cell = lc as *mut layout_cell;
        if !(*wp).layout_cell.is_null() {
            (*(*wp).layout_cell).wp = wp;
        }
        (*wp).xoff = xoff as ::core::ffi::c_int;
        (*wp).yoff = yoff as ::core::ffi::c_int;
        window_pane_resize(wp, sx, sy);
        wp = *(*((*(*w).active).entry.tqe_prev as *mut window_panes)).tqh_last;
        if wp.is_null() {
            wp = *(*((*w).panes.tqh_last as *mut window_panes)).tqh_last;
        }
    } else {
        wp = (*w).panes.tqh_first;
        if !(*wp).entry.tqe_next.is_null() {
            (*(*wp).entry.tqe_next).entry.tqe_prev = (*wp).entry.tqe_prev;
        } else {
            (*w).panes.tqh_last = (*wp).entry.tqe_prev;
        }
        *(*wp).entry.tqe_prev = (*wp).entry.tqe_next;
        (*wp).entry.tqe_next = ::core::ptr::null_mut::<window_pane>();
        (*wp).entry.tqe_prev = (*w).panes.tqh_last;
        *(*w).panes.tqh_last = wp;
        (*w).panes.tqh_last = &raw mut (*wp).entry.tqe_next;
        lc = (*wp).layout_cell as *mut layout_cell;
        xoff = (*wp).xoff as u_int;
        yoff = (*wp).yoff as u_int;
        sx = (*wp).sx;
        sy = (*wp).sy;
        wp = *(*((*w).panes.tqh_last as *mut window_panes)).tqh_last;
        while !wp.is_null() {
            wp2 = *(*((*wp).entry.tqe_prev as *mut window_panes)).tqh_last;
            if wp2.is_null() {
                break;
            }
            (*wp).layout_cell = (*wp2).layout_cell;
            if !(*wp).layout_cell.is_null() {
                (*(*wp).layout_cell).wp = wp;
            }
            (*wp).xoff = (*wp2).xoff;
            (*wp).yoff = (*wp2).yoff;
            window_pane_resize(wp, (*wp2).sx, (*wp2).sy);
            wp = *(*((*wp).entry.tqe_prev as *mut window_panes)).tqh_last;
        }
        (*wp).layout_cell = lc as *mut layout_cell;
        if !(*wp).layout_cell.is_null() {
            (*(*wp).layout_cell).wp = wp;
        }
        (*wp).xoff = xoff as ::core::ffi::c_int;
        (*wp).yoff = yoff as ::core::ffi::c_int;
        window_pane_resize(wp, sx, sy);
        wp = (*(*w).active).entry.tqe_next;
        if wp.is_null() {
            wp = (*w).panes.tqh_first;
        }
    }
    window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
    cmd_find_from_winlink_pane(current, wl, wp, 0 as ::core::ffi::c_int);
    window_pop_zoom(w);
    redraw_invalidate_scene(w);
    server_redraw_window(w);
    return CMD_RETURN_NORMAL;
}

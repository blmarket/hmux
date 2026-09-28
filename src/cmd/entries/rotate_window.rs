use crate::src::arguments::args_has;
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::find::cmd_find_from_winlink_pane;
use crate::src::cmd::queue::{cmdq_get_state_owned, cmdq_get_target};
use crate::src::screen_redraw::redraw_invalidate_scene;
use crate::src::server_fn::server_redraw_window;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::layout::layout_cell;
use crate::src::shared::window::{WindowOwner, winlink};
use crate::src::window::window_pane_resize;
use crate::src::window::{
    window_pane_first, window_pane_last, window_pane_list_insert_back,
    window_pane_list_insert_front, window_pane_list_remove, window_pane_next, window_pane_previous,
    window_pop_zoom, window_push_zoom, window_set_active_pane,
};
pub static cmd_rotate_window_entry: cmd_entry = {
    cmd_entry {
        name: c"rotate-window",
        alias: Some(c"rotatew"),
        args: args_parse {
            template: c"Dt:UZ",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-DUZ] [-t target-window]",
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
        exec: Some(cmd_rotate_window_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_rotate_window_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let current = cmdq_get_state_owned(item);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut wl: *mut winlink = (*target).wl_ptr();
    let window_owner = WindowOwner::adopt((*target).w.upgrade().expect("live rotation window"));
    let w = window_owner.as_ptr();
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
    let selected_pane = if args_has(args, 'D' as i32 as u_char) != 0 {
        let moved_owner = window_pane_last(Some(&*w)).expect("rotation window has panes");
        let wp = moved_owner.get();
        window_pane_list_remove(&mut *w, &*wp);
        window_pane_list_insert_front(&mut *w, &*wp);
        lc = (*wp).layout_cell as *mut layout_cell;
        xoff = (*wp).xoff as u_int;
        yoff = (*wp).yoff as u_int;
        sx = (*wp).sx;
        sy = (*wp).sy;
        let mut cursor = window_pane_first(Some(&*w)).expect("rotation window has panes");
        loop {
            let wp = cursor.get();
            let Some(neighbor) = window_pane_next(Some(&*wp)) else {
                break;
            };
            let wp2 = neighbor.get();
            (*wp).layout_cell = (*wp2).layout_cell;
            if !(*wp).layout_cell.is_null() {
                (*(*wp).layout_cell).wp = (*wp).observer.clone();
            }
            (*wp).xoff = (*wp2).xoff;
            (*wp).yoff = (*wp2).yoff;
            window_pane_resize(&cursor, (*wp2).sx, (*wp2).sy);
            cursor = window_pane_next(Some(&*wp)).expect("rotation neighbor remains in order");
        }
        let wp = cursor.get();
        (*wp).layout_cell = lc as *mut layout_cell;
        if !(*wp).layout_cell.is_null() {
            (*(*wp).layout_cell).wp = (*wp).observer.clone();
        }
        (*wp).xoff = xoff as ::core::ffi::c_int;
        (*wp).yoff = yoff as ::core::ffi::c_int;
        window_pane_resize(&cursor, sx, sy);
        window_pane_previous(((*w).active).as_ref())
            .or_else(|| window_pane_last(Some(&*w)))
    } else {
        let moved_owner = window_pane_first(Some(&*w)).expect("rotation window has panes");
        let wp = moved_owner.get();
        window_pane_list_remove(&mut *w, &*wp);
        window_pane_list_insert_back(&mut *w, &*wp);
        lc = (*wp).layout_cell as *mut layout_cell;
        xoff = (*wp).xoff as u_int;
        yoff = (*wp).yoff as u_int;
        sx = (*wp).sx;
        sy = (*wp).sy;
        let mut cursor = window_pane_last(Some(&*w)).expect("rotation window has panes");
        loop {
            let wp = cursor.get();
            let Some(neighbor) = window_pane_previous(Some(&*wp)) else {
                break;
            };
            let wp2 = neighbor.get();
            (*wp).layout_cell = (*wp2).layout_cell;
            if !(*wp).layout_cell.is_null() {
                (*(*wp).layout_cell).wp = (*wp).observer.clone();
            }
            (*wp).xoff = (*wp2).xoff;
            (*wp).yoff = (*wp2).yoff;
            window_pane_resize(&cursor, (*wp2).sx, (*wp2).sy);
            cursor = window_pane_previous(Some(&*wp)).expect("rotation neighbor remains in order");
        }
        let wp = cursor.get();
        (*wp).layout_cell = lc as *mut layout_cell;
        if !(*wp).layout_cell.is_null() {
            (*(*wp).layout_cell).wp = (*wp).observer.clone();
        }
        (*wp).xoff = xoff as ::core::ffi::c_int;
        (*wp).yoff = yoff as ::core::ffi::c_int;
        window_pane_resize(&cursor, sx, sy);
        window_pane_next(((*w).active).as_ref())
            .or_else(|| window_pane_first(Some(&*w)))
    }.expect("rotation window has an active candidate");
    let wp = selected_pane.get();
    window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
    cmd_find_from_winlink_pane(&mut *current.current.borrow_mut(), wl, wp, 0 as ::core::ffi::c_int);
    window_pop_zoom(w);
    redraw_invalidate_scene(w);
    server_redraw_window(w);
    return CMD_RETURN_NORMAL;
}

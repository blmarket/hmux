use std::cell::UnsafeCell;
use std::rc::Rc;
use crate::src::shared::window::WindowOwner;
use crate::src::options::options_owner_ptr;
use crate::src::arguments::args_has;
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_source, cmdq_get_target};
use crate::src::events::events_fire_window;
use crate::src::layout::{layout_cell_is_tiled, layout_fix_panes};
use crate::src::options::options_set_parent;
use crate::src::screen_redraw::redraw_invalidate_scene;
use crate::src::server_client::server_client_remove_pane;
use crate::src::server_fn::server_redraw_window;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::CMD_FIND_DEFAULT_MARKED;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::layout::layout_cell;
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{PANE_STYLECHANGED, PANE_THEMECHANGED};
use crate::src::shared::window::window;
use crate::src::style::colour::colour_palette_from_option;
use crate::src::window::window_pane_resize;
use crate::src::window::{
    window_fire_pane_moved, window_pane_first, window_pane_is_floating, window_pane_last,
    window_pane_next, window_pane_previous, window_pane_stack_remove, window_pane_swap_order,
    window_pane_z_swap_order, window_pop_zoom, window_push_zoom, window_set_active_pane,
};
pub static cmd_swap_pane_entry: cmd_entry = {
    cmd_entry {
        name: c"swap-pane",
        alias: Some(c"swapp"),
        args: args_parse {
            template: c"dDs:t:UZ",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-dDUZ] [-s src-pane] [-t dst-pane]",
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
        exec: Some(cmd_swap_pane_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_swap_pane_next_tiled_pane(
    mut pane: Option<Rc<UnsafeCell<window_pane>>>,
) -> Option<Rc<UnsafeCell<window_pane>>> {
    while let Some(owner) = pane.as_ref() {
        let wp = &*owner.get();
        if layout_cell_is_tiled(wp.layout_cell) != 0 {
            break;
        }
        pane = window_pane_next(Some(wp));
    }
    pane
}
unsafe fn cmd_swap_pane_prev_tiled_pane(
    mut pane: Option<Rc<UnsafeCell<window_pane>>>,
) -> Option<Rc<UnsafeCell<window_pane>>> {
    while let Some(owner) = pane.as_ref() {
        let wp = &*owner.get();
        if layout_cell_is_tiled(wp.layout_cell) != 0 {
            break;
        }
        pane = window_pane_previous(Some(wp));
    }
    pane
}
unsafe fn cmd_swap_pane_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut source: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_source_mut(&mut *item);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut src_lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut dst_lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xoff: u_int = 0;
    let mut yoff: u_int = 0;
    let mut src_idx: ::core::ffi::c_int = 0;
    let mut dst_idx: ::core::ffi::c_int = 0;
    let dst_window_owner = WindowOwner::adopt((*target).w.upgrade().expect("live swap target window"));
    let dst_pane_owner = (*target).wp.upgrade().expect("live swap target pane");
    let dst_w = dst_window_owner.as_ptr();
    let dst_wp = dst_pane_owner.get();
    dst_idx = (*(*target).wl_ptr()).idx;
    let mut src_window_owner = WindowOwner::adopt((*source).w.upgrade().expect("live swap source window"));
    let mut src_pane_owner = (*source).wp.upgrade().expect("live swap source pane");
    let mut src_w = src_window_owner.as_ptr();
    let src_wp = src_pane_owner.get();
    src_idx = (*(*source).wl_ptr()).idx;
    if (*src_w).modal.ptr_eq(&(*src_wp).observer) || (*dst_w).modal.ptr_eq(&(*dst_wp).observer) {
        cmdq_error(item, |out| out.write_all(b"pane is modal"));
        return CMD_RETURN_ERROR;
    }
    if window_push_zoom(
        dst_w,
        0 as ::core::ffi::c_int,
        args_has(args, 'Z' as i32 as u_char),
    ) != 0
    {
        server_redraw_window(&*(dst_w));
    }
    if args_has(args, 'D' as i32 as u_char) != 0 {
        if window_pane_is_floating(&*dst_wp) != 0 {
            cmdq_error(item, |out| {
                out.write_all(b"cannot swap down on floating pane")
            });
            return CMD_RETURN_ERROR;
        }
        src_window_owner = WindowOwner::adopt(dst_window_owner.as_rc().clone());
        src_w = src_window_owner.as_ptr();
        src_pane_owner = cmd_swap_pane_next_tiled_pane(window_pane_next(Some(&*dst_wp)))
            .or_else(|| cmd_swap_pane_next_tiled_pane(window_pane_first(Some(&*dst_w))))
            .expect("tiled swap target remains in its window");
    } else if args_has(args, 'U' as i32 as u_char) != 0 {
        if window_pane_is_floating(&*dst_wp) != 0 {
            cmdq_error(item, |out| {
                out.write_all(b"cannot swap up on floating pane")
            });
            return CMD_RETURN_ERROR;
        }
        src_window_owner = WindowOwner::adopt(dst_window_owner.as_rc().clone());
        src_w = src_window_owner.as_ptr();
        src_pane_owner = cmd_swap_pane_prev_tiled_pane(window_pane_previous(Some(&*dst_wp)))
            .or_else(|| cmd_swap_pane_prev_tiled_pane(window_pane_last(Some(&*dst_w))))
            .expect("tiled swap target remains in its window");
    }
    let src_wp = src_pane_owner.get();
    if src_w != dst_w
        && window_push_zoom(
            src_w,
            0 as ::core::ffi::c_int,
            args_has(args, 'Z' as i32 as u_char),
        ) != 0
    {
        server_redraw_window(&*(src_w));
    }
    if !Rc::ptr_eq(&src_pane_owner, &dst_pane_owner) {
        server_client_remove_pane(src_wp);
        server_client_remove_pane(dst_wp);
        window_pane_swap_order(
            &mut *dst_w,
            &*dst_wp,
            if src_w == dst_w { None } else { Some(&mut *src_w) },
            &*src_wp,
        );
        window_pane_z_swap_order(
            &mut *dst_w,
            &*dst_wp,
            if src_w == dst_w { None } else { Some(&mut *src_w) },
            &*src_wp,
        );
        src_lc = (*src_wp).layout_cell as *mut layout_cell;
        dst_lc = (*dst_wp).layout_cell as *mut layout_cell;
        (*src_lc).wp = (*dst_wp).observer.clone();
        (*dst_wp).layout_cell = src_lc as *mut layout_cell;
        (*dst_lc).wp = (*src_wp).observer.clone();
        (*src_wp).layout_cell = dst_lc as *mut layout_cell;
        (*src_wp).window = dst_w as *mut window;
        options_set_parent(options_owner_ptr(&mut (*src_wp).options).map_or(std::ptr::null_mut(), |options| options), options_owner_ptr(&mut (*dst_w).options).map_or(std::ptr::null_mut(), |options| options));
        (*src_wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
        (*dst_wp).window = src_w as *mut window;
        options_set_parent(options_owner_ptr(&mut (*dst_wp).options).map_or(std::ptr::null_mut(), |options| options), options_owner_ptr(&mut (*src_w).options).map_or(std::ptr::null_mut(), |options| options));
        (*dst_wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
        sx = (*src_wp).sx;
        sy = (*src_wp).sy;
        xoff = (*src_wp).xoff as u_int;
        yoff = (*src_wp).yoff as u_int;
        (*src_wp).xoff = (*dst_wp).xoff;
        (*src_wp).yoff = (*dst_wp).yoff;
        window_pane_resize(&src_pane_owner, (*dst_wp).sx, (*dst_wp).sy);
        (*dst_wp).xoff = xoff as ::core::ffi::c_int;
        (*dst_wp).yoff = yoff as ::core::ffi::c_int;
        window_pane_resize(&dst_pane_owner, sx, sy);
        if args_has(args, 'd' as i32 as u_char) == 0 {
            if src_w != dst_w {
                window_set_active_pane(src_w, dst_wp, 1 as ::core::ffi::c_int);
                window_set_active_pane(dst_w, src_wp, 1 as ::core::ffi::c_int);
            } else {
                window_set_active_pane(src_w, dst_wp, 1 as ::core::ffi::c_int);
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
            colour_palette_from_option(Some(&mut (*src_wp).palette), options_owner_ptr(&mut (*src_wp).options).map_or(std::ptr::null_mut(), |options| options));
            colour_palette_from_option(Some(&mut (*dst_wp).palette), options_owner_ptr(&mut (*dst_wp).options).map_or(std::ptr::null_mut(), |options| options));
            layout_fix_panes(src_w, ::core::ptr::null_mut::<window_pane>());
            redraw_invalidate_scene(src_w);
            server_redraw_window(&*(src_w));
        }
        layout_fix_panes(dst_w, ::core::ptr::null_mut::<window_pane>());
        redraw_invalidate_scene(dst_w);
        server_redraw_window(&*(dst_w));
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
        server_redraw_window(&*(src_w));
    }
    if src_w != dst_w && window_pop_zoom(dst_w) != 0 {
        server_redraw_window(&*(dst_w));
    }
    return CMD_RETURN_NORMAL;
}

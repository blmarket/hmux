use crate::src::arguments::args_has;
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_source, cmdq_get_target};
use crate::src::events::events_fire_window;
use crate::src::layout::layout_fix_panes;
use crate::src::options::options_owner_ptr;
use crate::src::options::options_set_parent;
use crate::src::server_client::server_client_remove_pane;
use crate::src::server_fn::server_redraw_window;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::CMD_FIND_DEFAULT_MARKED;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{PANE_STYLECHANGED, PANE_THEMECHANGED};
use crate::src::shared::window::window;
use crate::src::style::colour::colour_palette_from_option;
use crate::src::window::window_pane_resize;
use crate::src::window::Window as _;
use crate::src::window::{
    window_fire_pane_moved, window_pane_is_floating, window_pane_next, window_pane_previous,
    window_pop_zoom, window_push_zoom, window_set_active_pane,
};
use crate::src::window_pane::WindowPane as _;
use std::cell::UnsafeCell;
use std::rc::Rc;
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
        exec: Some(cmd_swap_pane_exec),
    }
};
unsafe fn cmd_swap_pane_next_tiled_pane(
    mut pane: Option<Rc<UnsafeCell<window_pane>>>,
) -> Option<Rc<UnsafeCell<window_pane>>> {
    while let Some(owner) = pane.as_ref() {
        if !owner.is_floating() {
            break;
        }
        pane = window_pane_next(Some(&*owner.get()));
    }
    pane
}
unsafe fn cmd_swap_pane_prev_tiled_pane(
    mut pane: Option<Rc<UnsafeCell<window_pane>>>,
) -> Option<Rc<UnsafeCell<window_pane>>> {
    while let Some(owner) = pane.as_ref() {
        if !owner.is_floating() {
            break;
        }
        pane = window_pane_previous(Some(&*owner.get()));
    }
    pane
}
unsafe fn cmd_swap_pane_exec(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    let item = item_handle.get();
    let mut args: *mut args =
        cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
    let mut source: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_source_mut(&mut *item);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xoff: u_int = 0;
    let mut yoff: u_int = 0;
    let mut src_idx: ::core::ffi::c_int = 0;
    let mut dst_idx: ::core::ffi::c_int = 0;
    let dst_window_owner = (*target).w.upgrade().expect("live swap target window");
    let dst_pane_owner = (*target).wp.upgrade().expect("live swap target pane");

    let dst_wp = dst_pane_owner.get();
    dst_idx = ((*target).winlink_handle()).get_unchecked().idx;
    let mut src_window_owner = (*source).w.upgrade().expect("live swap source window");
    let result = (|| {
        let mut src_pane_owner = (*source).wp.upgrade().expect("live swap source pane");

        let src_wp = src_pane_owner.get();
        src_idx = ((*source).winlink_handle()).get_unchecked().idx;
        if src_window_owner
            .modal_pane()
            .is_some_and(|pane| Rc::ptr_eq(&pane, &src_pane_owner))
            || dst_window_owner
                .modal_pane()
                .is_some_and(|pane| Rc::ptr_eq(&pane, &dst_pane_owner))
        {
            cmdq_error(item_handle, |out| out.write_all(b"pane is modal"));
            return CMD_RETURN_ERROR;
        }
        if window_push_zoom(
            &std::rc::Rc::clone(&(dst_window_owner)),
            0 as ::core::ffi::c_int,
            args_has(args, 'Z' as i32 as u_char),
        ) != 0
        {
            server_redraw_window(&(dst_window_owner));
        }
        if args_has(args, 'D' as i32 as u_char) != 0 {
            if window_pane_is_floating(&*dst_wp) != 0 {
                cmdq_error(item_handle, |out| {
                    out.write_all(b"cannot swap down on floating pane")
                });
                return CMD_RETURN_ERROR;
            }
            let previous = std::mem::replace(&mut src_window_owner, dst_window_owner.clone());
            crate::src::window::window_remove_ref(previous, c"cmd_swap_pane_exec".as_ptr());

            src_pane_owner = cmd_swap_pane_next_tiled_pane(window_pane_next(Some(&*dst_wp)))
                .or_else(|| {
                    cmd_swap_pane_next_tiled_pane(dst_window_owner.step_pane(
                        crate::src::window::PaneOrder::Index,
                        None,
                        false,
                    ))
                })
                .expect("tiled swap target remains in its window");
        } else if args_has(args, 'U' as i32 as u_char) != 0 {
            if window_pane_is_floating(&*dst_wp) != 0 {
                cmdq_error(item_handle, |out| {
                    out.write_all(b"cannot swap up on floating pane")
                });
                return CMD_RETURN_ERROR;
            }
            let previous = std::mem::replace(&mut src_window_owner, dst_window_owner.clone());
            crate::src::window::window_remove_ref(previous, c"cmd_swap_pane_exec".as_ptr());

            src_pane_owner = cmd_swap_pane_prev_tiled_pane(window_pane_previous(Some(&*dst_wp)))
                .or_else(|| {
                    cmd_swap_pane_prev_tiled_pane(dst_window_owner.step_pane(
                        crate::src::window::PaneOrder::Index,
                        None,
                        true,
                    ))
                })
                .expect("tiled swap target remains in its window");
        }
        let src_wp = src_pane_owner.get();
        if !Rc::ptr_eq(&src_window_owner, &dst_window_owner)
            && window_push_zoom(
                &src_window_owner,
                0 as ::core::ffi::c_int,
                args_has(args, 'Z' as i32 as u_char),
            ) != 0
        {
            server_redraw_window(&src_window_owner);
        }
        if !Rc::ptr_eq(&src_pane_owner, &dst_pane_owner) {
            server_client_remove_pane(&(*(src_wp)).observer.upgrade().expect("live window_pane"));
            server_client_remove_pane(&(*(dst_wp)).observer.upgrade().expect("live window_pane"));
            for order in [
                crate::src::window::PaneOrder::Index,
                crate::src::window::PaneOrder::Stacking,
            ] {
                dst_window_owner.swap_pane_order(
                    order,
                    &Rc::downgrade(&dst_pane_owner),
                    &src_window_owner,
                    &Rc::downgrade(&src_pane_owner),
                );
            }
            let src_lc = (*src_wp).layout_cell.expect("swap source layout cell");
            let dst_lc = (*dst_wp).layout_cell.expect("swap target layout cell");
            {
                let mut cell = src_window_owner
                    .borrow_layout_cell_mut(src_lc)
                    .expect("swap source cell belongs to its original window");
                cell.wp = Rc::downgrade(&dst_pane_owner);
            }
            (*dst_wp).layout_cell = Some(src_lc);
            {
                let mut cell = dst_window_owner
                    .borrow_layout_cell_mut(dst_lc)
                    .expect("swap target cell belongs to its original window");
                cell.wp = Rc::downgrade(&src_pane_owner);
            }
            (*src_wp).layout_cell = Some(dst_lc);
            src_pane_owner.reparent(&dst_window_owner);
            dst_pane_owner.reparent(&src_window_owner);
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
                if !Rc::ptr_eq(&src_window_owner, &dst_window_owner) {
                    window_set_active_pane(
                        &src_window_owner,
                        &(*(dst_wp)).observer.upgrade().expect("live window_pane"),
                        1 as ::core::ffi::c_int,
                    );
                    window_set_active_pane(
                        &std::rc::Rc::clone(&(dst_window_owner)),
                        &(*(src_wp)).observer.upgrade().expect("live window_pane"),
                        1 as ::core::ffi::c_int,
                    );
                } else {
                    window_set_active_pane(
                        &src_window_owner,
                        &(*(dst_wp)).observer.upgrade().expect("live window_pane"),
                        1 as ::core::ffi::c_int,
                    );
                }
            } else {
                if src_window_owner
                    .active_pane()
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get())
                    == src_wp
                {
                    window_set_active_pane(
                        &src_window_owner,
                        &(*(dst_wp)).observer.upgrade().expect("live window_pane"),
                        1 as ::core::ffi::c_int,
                    );
                }
                if (dst_window_owner)
                    .active_pane()
                    .as_ref()
                    .map_or(std::ptr::null_mut(), |owner| owner.get())
                    == dst_wp
                {
                    window_set_active_pane(
                        &std::rc::Rc::clone(&(dst_window_owner)),
                        &(*(src_wp)).observer.upgrade().expect("live window_pane"),
                        1 as ::core::ffi::c_int,
                    );
                }
            }
            if !Rc::ptr_eq(&src_window_owner, &dst_window_owner) {
                crate::src::shared::pane::pane_history_remove(
                    &mut src_window_owner.borrow_pane_history_mut(),
                    &Rc::downgrade(&src_pane_owner),
                );
                crate::src::shared::pane::pane_history_remove(
                    &mut dst_window_owner.borrow_pane_history_mut(),
                    &Rc::downgrade(&dst_pane_owner),
                );
                colour_palette_from_option(
                    Some(&mut (*src_wp).palette),
                    options_owner_ptr(&mut (*src_wp).options)
                        .map_or(std::ptr::null_mut(), |options| options),
                );
                colour_palette_from_option(
                    Some(&mut (*dst_wp).palette),
                    options_owner_ptr(&mut (*dst_wp).options)
                        .map_or(std::ptr::null_mut(), |options| options),
                );
                layout_fix_panes(&src_window_owner, None);
                src_window_owner.invalidate_scene();
                server_redraw_window(&src_window_owner);
            }
            layout_fix_panes(&std::rc::Rc::clone(&(dst_window_owner)), None);
            dst_window_owner.invalidate_scene();
            server_redraw_window(&(dst_window_owner));
            if !Rc::ptr_eq(&src_window_owner, &dst_window_owner) {
                window_fire_pane_moved(
                    &(*(src_wp)).observer.upgrade().expect("live window_pane"),
                    &src_window_owner,
                    src_idx,
                    &std::rc::Rc::clone(&(dst_window_owner)),
                    dst_idx,
                );
                window_fire_pane_moved(
                    &(*(dst_wp)).observer.upgrade().expect("live window_pane"),
                    &std::rc::Rc::clone(&(dst_window_owner)),
                    dst_idx,
                    &src_window_owner,
                    src_idx,
                );
            }
            events_fire_window(
                b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
                Rc::clone(&src_window_owner),
            );
            if !Rc::ptr_eq(&src_window_owner, &dst_window_owner) {
                events_fire_window(
                    b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
                    std::rc::Rc::clone(&(dst_window_owner)),
                );
            }
        }
        if window_pop_zoom(&src_window_owner) != 0 {
            server_redraw_window(&src_window_owner);
        }
        if !Rc::ptr_eq(&src_window_owner, &dst_window_owner)
            && window_pop_zoom(&std::rc::Rc::clone(&(dst_window_owner))) != 0
        {
            server_redraw_window(&(dst_window_owner));
        }
        return CMD_RETURN_NORMAL;
    })();
    crate::src::window::window_remove_ref(src_window_owner, c"cmd_swap_pane_exec".as_ptr());
    crate::src::window::window_remove_ref(dst_window_owner, c"cmd_swap_pane_exec".as_ptr());
    result
}

use crate::src::arguments::args_has;
use crate::src::cmd::cmd_get_args_mut;
use crate::src::cmd::find::cmd_find_from_winlink_pane;
use crate::src::cmd::queue::{cmdq_get_state_owned, cmdq_get_target};
use crate::src::server_fn::server_redraw_window;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::layout::layout_cell;
use crate::src::shared::pane::window_pane;
use crate::src::shared::window::{winlink, WindowRef};
use crate::src::window::Window as _;
use crate::src::window_pane::WindowPane as _;
use std::cell::UnsafeCell;
use std::rc::Rc;
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
        exec: Some(cmd_rotate_window_exec),
    }
};

unsafe fn cmd_rotate_window_assign_cell(
    window: &WindowRef,
    pane: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    cell: Option<*mut layout_cell>,
) {
    if let Some(id) = cell {
        let Some(mut cell) = window.borrow_layout_cell_mut(id) else {
            // A resize callback may have replaced the original tree. Keep the
            // pane's replacement association instead of restoring an expired ID.
            return;
        };
        cell.wp = std::rc::Rc::downgrade(pane);
    }
    if let Some(cell) = cell {
        pane.place_in_layout(cell);
    } else if let Some(previous) = pane.layout_identity() {
        pane.detach_layout(previous);
    }
}

// The resize operation may dispatch mode callbacks. Only copied geometry and
// cell identities survive each call; the following pane is looked up live.
unsafe fn cmd_rotate_window_panes(
    window: &WindowRef,
    down: bool,
    mut resize: impl FnMut(&Rc<UnsafeCell<window_pane>>, u32, u32),
) -> Rc<UnsafeCell<window_pane>> {
    let moved = window
        .step_pane(None, down)
        .expect("rotation window has panes");
    {
        let mut order = window.borrow_pane_order_mut();
        let observer = Rc::downgrade(&moved);
        assert!(order.remove(&observer), "pane is not in its window order");
        if down {
            order.push_front(observer);
        } else {
            order.push_back(observer);
        }
    }
    let saved_cell = moved.layout_identity();
    let (saved_sx, saved_sy, saved_x, saved_y) = moved.geometry();
    let mut cursor = window
        .step_pane(None, !down)
        .expect("rotation window has panes");
    let next =
        |pane: &Rc<UnsafeCell<window_pane>>| window.step_pane(Some(&Rc::downgrade(pane)), !down);
    while let Some(neighbor) = next(&cursor) {
        let cell = neighbor.layout_identity();
        let (sx, sy, x, y) = neighbor.geometry();
        cmd_rotate_window_assign_cell(window, &cursor, cell);
        cursor.set_layout_offset(x, y);
        resize(&cursor, sx, sy);
        cursor = next(&cursor).expect("rotation neighbor remains in order");
    }
    cmd_rotate_window_assign_cell(window, &cursor, saved_cell);
    cursor.set_layout_offset(saved_x, saved_y);
    resize(&cursor, saved_sx, saved_sy);
    let active = window.active_pane();
    let selected = active
        .as_ref()
        .and_then(|pane| window.step_pane(Some(&Rc::downgrade(pane)), down));
    selected
        .or_else(|| window.step_pane(None, down))
        .expect("rotation window has an active candidate")
}

unsafe fn cmd_rotate_window_exec(
    mut self_0: refbox::Weak<cmd>,
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
) -> cmd_retval {
    let item = item_handle.get();
    let mut args: *mut args =
        cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
    let current = cmdq_get_state_owned(&*(item));
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut wl: refbox::Weak<winlink> = (*target).winlink_handle();
    let window_owner = (*target).w.upgrade().expect("live rotation window");
    let result = {
        let selected_pane =
            cmd_rotate_window_panes(&window_owner, args_has(args, b'D') != 0, |pane, sx, sy| {
                pane.resize(sx, sy)
            });
        std::rc::Rc::clone(&(window_owner)).select_pane(&selected_pane, true);
        cmd_find_from_winlink_pane(
            &mut *current.current.borrow_mut(),
            wl.clone(),
            &selected_pane,
            0 as ::core::ffi::c_int,
        );
        window_owner.invalidate_scene();
        server_redraw_window(&(window_owner));
        CMD_RETURN_NORMAL
    };
    window_owner.release(c"cmd_rotate_window_exec");
    result
}

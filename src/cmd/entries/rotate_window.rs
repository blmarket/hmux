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
#[cfg(test)]
use crate::src::window_pane::PaneFixture as _;

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
    } else if let Some(previous) = pane.layout_identity(false) {
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
        .step_pane(crate::src::window::PaneOrder::Index, None, down)
        .expect("rotation window has panes");
    {
        let mut order = window.borrow_pane_order_mut(crate::src::window::PaneOrder::Index);
        let observer = Rc::downgrade(&moved);
        assert!(order.remove(&observer), "pane is not in its window order");
        if down {
            order.push_front(observer);
        } else {
            order.push_back(observer);
        }
    }
    let saved_cell = moved.layout_identity(false);
    let (saved_sx, saved_sy, saved_x, saved_y) = moved.geometry();
    let mut cursor = window
        .step_pane(crate::src::window::PaneOrder::Index, None, !down)
        .expect("rotation window has panes");
    let next = |pane: &Rc<UnsafeCell<window_pane>>| {
        window.step_pane(
            crate::src::window::PaneOrder::Index,
            Some(&Rc::downgrade(pane)),
            !down,
        )
    };
    while let Some(neighbor) = next(&cursor) {
        let cell = neighbor.layout_identity(false);
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
    let selected = active.as_ref().and_then(|pane| {
        window.step_pane(
            crate::src::window::PaneOrder::Index,
            Some(&Rc::downgrade(pane)),
            down,
        )
    });
    selected
        .or_else(|| window.step_pane(crate::src::window::PaneOrder::Index, None, down))
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
    let result = (|| {
        window_owner.push_zoom(false, (args_has(args, 'Z' as u_char)) != 0);
        let selected_pane = cmd_rotate_window_panes(
            &window_owner,
            args_has(args, 'D' as u_char) != 0,
            |pane, sx, sy| pane.resize(sx, sy),
        );
        (&std::rc::Rc::clone(&(window_owner))).select_pane(&selected_pane, true);
        cmd_find_from_winlink_pane(
            &mut *current.current.borrow_mut(),
            wl.clone(),
            &selected_pane,
            0 as ::core::ffi::c_int,
        );
        (&std::rc::Rc::clone(&(window_owner))).pop_zoom();
        window_owner.invalidate_scene();
        server_redraw_window(&(window_owner));
        return CMD_RETURN_NORMAL;
    })();
    window_owner.release(c"cmd_rotate_window_exec");
    result
}

#[cfg(test)]
mod layout_identity_tests {
    use super::*;
    use crate::src::layout::{layout_make_leaf, layout_make_node};
    use crate::src::shared::layout::{layout_cell, layout_cells_push_back, LAYOUT_LEFTRIGHT};
    use crate::src::shared::window::window;
    use crate::src::window::{LayoutView, PaneOrder};

    unsafe fn fixture(count: u32) -> (WindowRef, Vec<Rc<UnsafeCell<window_pane>>>) {
        let window = crate::src::shared::window::WindowRef::empty();
        let panes: Vec<_> = (0..count)
            .map(|index| {
                let pane = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::allocate();
                pane.fixture_id(index);
                pane.fixture_parent(Some(&window));
                pane.fixture_geometry(
                    (10 + index, 20 + index),
                    (3 * index as i32, -(index as i32)),
                );
                pane
            })
            .collect();
        window.initialize_pane(&panes[0], None);
        for pane in &panes[1..] {
            for order in [PaneOrder::Index, PaneOrder::Stacking] {
                window
                    .borrow_pane_order_mut(order)
                    .push_back(Rc::downgrade(pane));
            }
        }
        install_tree(&window, &panes);
        (window, panes)
    }

    unsafe fn install_tree(window: &WindowRef, panes: &[Rc<UnsafeCell<window_pane>>]) {
        let mut root = layout_cell::new();
        layout_make_node(&mut *root, LAYOUT_LEFTRIGHT);
        for pane in panes {
            let mut leaf = layout_cell::new();
            layout_make_leaf(&mut *leaf, pane);
            layout_cells_push_back(&mut *root, leaf);
        }
        let previous = window.borrow_layout_root_mut().replace(root);
        drop(previous);
    }

    unsafe fn finish(window: WindowRef, panes: Vec<Rc<UnsafeCell<window_pane>>>) {
        for order in [PaneOrder::Index, PaneOrder::Stacking] {
            window.borrow_pane_order_mut(order).storage.clear();
        }
        window.release(c"rotation layout identity fixture");
        for pane in panes {
            pane.release(c"rotation layout identity fixture");
        }
    }

    #[test]
    fn both_directions_preserve_geometry_cell_assignment_and_resize_order() {
        unsafe {
            for down in [false, true] {
                let (window, panes) = fixture(3);
                let old_ids: Vec<_> = panes
                    .iter()
                    .map(|pane| pane.layout_identity(false))
                    .collect();
                let old_geometry: Vec<_> = panes.iter().map(|pane| pane.geometry()).collect();
                let mut resized = Vec::new();
                let selected = cmd_rotate_window_panes(&window, down, |pane, sx, sy| {
                    // Reenter the Window during the real algorithm's resize phase.
                    assert_eq!(window.pane_snapshot().len(), 3);
                    resized.push(pane.id());
                    let (_, _, x, y) = pane.geometry();
                    pane.fixture_geometry((sx, sy), (x, y));
                });
                let sources = if down { [1, 2, 0] } else { [2, 0, 1] };
                for (pane, source) in panes.iter().zip(sources) {
                    assert_eq!(pane.layout_identity(false), old_ids[source]);
                    assert_eq!(pane.geometry(), old_geometry[source]);
                    let root = window.borrow_layout_root(LayoutView::Visible).unwrap();
                    let cell = root.find(old_ids[source].unwrap()).unwrap();
                    assert!(cell.wp.ptr_eq(&Rc::downgrade(pane)));
                }
                assert_eq!(resized, if down { vec![2, 0, 1] } else { vec![0, 2, 1] });
                assert_eq!(selected.id(), if down { 2 } else { 1 });
                drop(selected);
                finish(window, panes);
            }
        }
    }

    #[test]
    fn resize_replacement_does_not_restore_a_cell_id_from_the_old_tree() {
        unsafe {
            for down in [false, true] {
                let (window, panes) = fixture(2);
                let old_ids: Vec<_> = panes
                    .iter()
                    .map(|pane| pane.layout_identity(false))
                    .collect();
                let mut new_ids = Vec::new();
                let mut resized = Vec::new();
                let selected = cmd_rotate_window_panes(&window, down, |pane, _, _| {
                    resized.push(pane.id());
                    if resized.len() == 1 {
                        install_tree(&window, &panes);
                        new_ids = panes
                            .iter()
                            .map(|pane| pane.layout_identity(false))
                            .collect();
                    }
                });
                assert_eq!(resized.len(), 2, "rotation still resizes both panes");
                for (index, pane) in panes.iter().enumerate() {
                    assert_eq!(pane.layout_identity(false), new_ids[index]);
                    assert_ne!(new_ids[index], old_ids[index]);
                    let root = window.borrow_layout_root(LayoutView::Visible).unwrap();
                    assert!(root.find(old_ids[index].unwrap()).is_none());
                    assert!(root
                        .find(new_ids[index].unwrap())
                        .unwrap()
                        .wp
                        .ptr_eq(&Rc::downgrade(pane)));
                }
                drop(selected);
                finish(window, panes);
            }
        }
    }
}

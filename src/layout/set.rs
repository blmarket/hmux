use super::core::{
    layout_resize_adjust_with_limits, layout_resize_limits, layout_spread_cell_with_limits,
};
use crate::src::arguments::args_string_percentage_result;
use crate::src::events::events_fire_window;
use crate::src::ffi::libc::{strcmp, strlen, strncmp};
use crate::src::layout::{
    layout_count_cells, layout_create_cell, layout_fix_offsets, layout_fix_panes, layout_make_node,
    layout_print_cell, layout_set_size, layout_take_leaf, layout_take_leaves,
};
use crate::src::options::{options_get_number, options_get_string};
use crate::src::server_fn::server_redraw_window;
use crate::src::shared::abi::*;
use crate::src::shared::layout::layout_cell;
use crate::src::shared::layout::*;
use crate::src::shared::pane::PANE_MINIMUM;
use crate::src::shared::window::window;
use crate::src::shared::window::WindowRef;
use crate::src::window::Window as _;

use crate::src::window_pane::WindowPane as _;
use std::ffi::CStr;

/// A preset arrangement. `arrange` rebuilds the tree for a `window_size`
/// window and fixes offsets, without resizing panes, firing events or
/// redrawing. It returns false when it leaves the tree alone.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub name: &'static CStr,
    pub arrange: unsafe fn(&WindowRef, (u_int, u_int)) -> bool,
}
static layout_sets: [C2RustUnnamed_35; 7] = {
    [
        C2RustUnnamed_35 {
            name: c"even-horizontal",
            arrange: layout_set_even_h,
        },
        C2RustUnnamed_35 {
            name: c"even-vertical",
            arrange: layout_set_even_v,
        },
        C2RustUnnamed_35 {
            name: c"main-horizontal",
            arrange: layout_set_main_h,
        },
        C2RustUnnamed_35 {
            name: c"main-horizontal-mirrored",
            arrange: layout_set_main_h_mirrored,
        },
        C2RustUnnamed_35 {
            name: c"main-vertical",
            arrange: layout_set_main_v,
        },
        C2RustUnnamed_35 {
            name: c"main-vertical-mirrored",
            arrange: layout_set_main_v_mirrored,
        },
        C2RustUnnamed_35 {
            name: c"tiled",
            arrange: layout_set_tiled,
        },
    ]
};
pub unsafe fn layout_set_lookup(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    let mut matched: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    i = 0 as u_int;
    while (i as usize) < layout_sets.len() {
        if strcmp(layout_sets[i as usize].name.as_ptr(), name) == 0 as ::core::ffi::c_int {
            return i as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    i = 0 as u_int;
    while (i as usize) < layout_sets.len() {
        if strncmp(layout_sets[i as usize].name.as_ptr(), name, strlen(name))
            == 0 as ::core::ffi::c_int
        {
            if matched != -(1 as ::core::ffi::c_int) {
                return -(1 as ::core::ffi::c_int);
            }
            matched = i as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    matched
}
pub unsafe fn layout_set_select(w_owner: &WindowRef, mut layout: u_int) -> u_int {
    if layout as usize > layout_sets.len().wrapping_sub(1_usize) {
        layout = layout_sets.len().wrapping_sub(1_usize) as u_int;
    }
    if (layout_sets[layout as usize].arrange)(w_owner, w_owner.size()) {
        layout_fix_panes(w_owner, None);
        events_fire_window(
            c"window-layout-changed".as_ptr(),
            std::rc::Rc::clone(w_owner),
        );
        server_redraw_window(w_owner);
    }
    w_owner.remember_layout_preset(layout as i32);
    layout
}
pub unsafe fn layout_set_next(w_owner: &WindowRef) -> u_int {
    let mut layout: u_int = 0;
    if w_owner.last_layout_preset() == -(1 as ::core::ffi::c_int) {
        layout = 0 as u_int;
    } else {
        layout = (w_owner.last_layout_preset() + 1 as ::core::ffi::c_int) as u_int;
        if layout as usize > layout_sets.len().wrapping_sub(1_usize) {
            layout = 0 as u_int;
        }
    }
    layout_set_select(w_owner, layout)
}
pub unsafe fn layout_set_previous(w_owner: &WindowRef) -> u_int {
    let mut layout: u_int = 0;
    if w_owner.last_layout_preset() == -(1 as ::core::ffi::c_int) {
        layout = layout_sets.len().wrapping_sub(1_usize) as u_int;
    } else {
        layout = w_owner.last_layout_preset() as u_int;
        if layout == 0 as u_int {
            layout = layout_sets.len().wrapping_sub(1_usize) as u_int;
        } else {
            layout = layout.wrapping_sub(1);
        }
    }
    layout_set_select(w_owner, layout)
}
/// Arrange the window's sticky preset, if any, for an `sx` by `sy` window.
/// Returns whether it arranged; panes are not resized and nothing is notified.
pub unsafe fn layout_set_arrange_sticky(w_owner: &WindowRef, sx: u_int, sy: u_int) -> bool {
    if w_owner.is_zoomed() {
        return false;
    }
    let Some(layout) = w_owner.sticky_layout() else {
        return false;
    };
    // A cell reserved for a pane not yet assigned has nothing to arrange; the
    // assignment arranges instead.
    let leaves = w_owner
        .borrow_layout_root(crate::src::window::LayoutView::Visible)
        .map_or(0, |root| layout_count_cells(&*root as *const _ as *mut _));
    if leaves as usize != layout_set_cells(w_owner).len() {
        return false;
    }
    (layout_sets[layout as usize].arrange)(w_owner, (sx, sy))
}
/// The cells of the panes placed in the layout, in pane order. A pane whose
/// cell was just closed is still in the pane list and is skipped.
unsafe fn layout_set_cells(w_owner: &WindowRef) -> Vec<*mut layout_cell> {
    let panes = w_owner.pane_snapshot();
    let Some(root) = w_owner.borrow_layout_root(crate::src::window::LayoutView::Visible) else {
        return Vec::new();
    };
    panes
        .iter()
        .filter_map(|pane| {
            let id = pane.layout_identity(false)?;
            assert!(
                root.find(id).is_some(),
                "placed pane belongs to visible layout"
            );
            Some(id)
        })
        .collect()
}
// Detached leaves remain owned throughout preset reconstruction. Resolve their
// IDs here rather than reaching through Pane into the Window's old tree.
fn layout_set_leaf(leaves: &mut [Box<layout_cell>], id: *mut layout_cell) -> &mut layout_cell {
    leaves
        .iter_mut()
        .find(|cell| cell.id() == id)
        .expect("detached pane cell")
}

unsafe fn layout_set_even(
    w_owner: &WindowRef,
    window_size: (u_int, u_int),
    mut type_0: layout_type,
) -> bool {
    {
        let cells = layout_set_cells(w_owner);

        let mut wp = None;
        let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut n: u_int = 0;
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        {
            let mut tree = w_owner.borrow_layout_root_mut();
            layout_print_cell(
                tree.as_deref_mut()
                    .map_or(std::ptr::null_mut(), |root| root),
                c"layout_set_even".as_ptr(),
                1,
            );
        }
        n = cells.len() as u_int;
        if n <= 1 as u_int {
            return false;
        }
        if type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            sx = n
                .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
                .wrapping_sub(1 as u_int);
            if sx < window_size.0 {
                sx = window_size.0;
            }
            sy = window_size.1;
        } else {
            sy = n
                .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
                .wrapping_sub(1 as u_int);
            if sy < window_size.1 {
                sy = window_size.1;
            }
            sx = window_size.0;
        }
        let (pane_status, horizontal_minimum) = layout_resize_limits(w_owner);
        let mut tree = w_owner.borrow_layout_root_mut();
        let mut leaves = layout_take_leaves(tree.take());
        *tree = Some(layout_create_cell());
        lcroot = tree.as_deref_mut().expect("preset layout root");
        layout_set_size(
            lcroot,
            sx,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_make_node(lcroot, type_0);
        let mut pane_iter = cells.iter();
        wp = pane_iter.next();
        while wp.is_some() {
            lcchild = layout_set_leaf(&mut leaves, *wp.expect("pane")) as *mut layout_cell;
            layout_cells_push_back(lcroot, layout_take_leaf(&mut leaves, (*lcchild).id()));
            (*lcchild).parent = lcroot;
            (*lcchild).g.sx = window_size.0;
            (*lcchild).g.sy = window_size.1;
            wp = pane_iter.next();
        }
        layout_spread_cell_with_limits(lcroot, pane_status, horizontal_minimum, lcroot);
        assert!(leaves.is_empty(), "all detached pane cells were reinserted");
    } // All tree pointers and the component guard end before pane callbacks.
    layout_fix_offsets(w_owner);
    {
        let mut tree = w_owner.borrow_layout_root_mut();
        let root = tree.as_deref_mut().expect("preset layout root");
        layout_print_cell(root, c"layout_set_even".as_ptr(), 1);
    }
    true
}
unsafe fn layout_set_even_h(w_owner: &WindowRef, window_size: (u_int, u_int)) -> bool {
    layout_set_even(w_owner, window_size, LAYOUT_LEFTRIGHT)
}
unsafe fn layout_set_even_v(w_owner: &WindowRef, window_size: (u_int, u_int)) -> bool {
    layout_set_even(w_owner, window_size, LAYOUT_TOPBOTTOM)
}
unsafe fn layout_set_main_h(w_owner: &WindowRef, window_size: (u_int, u_int)) -> bool {
    {
        let cells = layout_set_cells(w_owner);

        let mut wp = None;
        let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcmain: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut n: u_int = 0;
        let mut mainh: u_int = 0;
        let mut otherh: u_int = 0;
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        {
            let mut tree = w_owner.borrow_layout_root_mut();
            layout_print_cell(
                tree.as_deref_mut()
                    .map_or(std::ptr::null_mut(), |root| root),
                c"layout_set_main_h".as_ptr(),
                1,
            );
        }
        n = cells.len() as u_int;
        if n <= 1 as u_int {
            return false;
        }
        n = n.wrapping_sub(1);
        sy = window_size.1.wrapping_sub(1 as u_int);
        let size_option =
            w_owner.with_options_mut(|options| options_get_string(options, c"main-pane-height"));
        s = size_option.as_ptr();
        mainh = match args_string_percentage_result(
            (!s.is_null()).then(|| ::std::ffi::CStr::from_ptr(s)),
            sy as ::core::ffi::c_longlong,
            sy as ::core::ffi::c_longlong,
        ) {
            Ok(value) => value as u_int,
            Err(_) => 24 as u_int,
        };
        if mainh.wrapping_add(PANE_MINIMUM as u_int) >= sy {
            if sy <= (PANE_MINIMUM + PANE_MINIMUM) as u_int {
                mainh = PANE_MINIMUM as u_int;
            } else {
                mainh = sy.wrapping_sub(PANE_MINIMUM as u_int);
            }
            otherh = PANE_MINIMUM as u_int;
        } else {
            let size_option = w_owner
                .with_options_mut(|options| options_get_string(options, c"other-pane-height"));
            s = size_option.as_ptr();
            otherh = match args_string_percentage_result(
                (!s.is_null()).then(|| ::std::ffi::CStr::from_ptr(s)),
                sy as ::core::ffi::c_longlong,
                sy as ::core::ffi::c_longlong,
            ) {
                Ok(value) => value as u_int,
                Err(_) => 0 as u_int,
            };
            if otherh == 0 as u_int || otherh > sy || sy.wrapping_sub(otherh) < mainh {
                otherh = sy.wrapping_sub(mainh);
            } else {
                mainh = sy.wrapping_sub(otherh);
            }
        }
        sx = n
            .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
            .wrapping_sub(1 as u_int);
        if sx < window_size.0 {
            sx = window_size.0;
        }
        let (pane_status, horizontal_minimum) = layout_resize_limits(w_owner);
        let mut tree = w_owner.borrow_layout_root_mut();
        let mut leaves = layout_take_leaves(tree.take());
        *tree = Some(layout_create_cell());
        lcroot = tree.as_deref_mut().expect("preset layout root");
        layout_set_size(
            lcroot,
            sx,
            mainh.wrapping_add(otherh).wrapping_add(1 as u_int),
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_make_node(lcroot, LAYOUT_TOPBOTTOM);
        let main_id = *cells.first().expect("main pane");
        lcmain = layout_set_leaf(&mut leaves, main_id) as *mut layout_cell;
        (*lcmain).parent = lcroot;
        layout_set_size(
            lcmain,
            sx,
            mainh,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_cells_push_back(lcroot, layout_take_leaf(&mut leaves, (*lcmain).id()));
        if n == 1 as u_int {
            let mut pane_iter = cells.iter().skip_while(|id| **id != main_id).skip(1);
            wp = pane_iter.next();
            let mut secondary = layout_take_leaf(&mut leaves, *wp.expect("pane"));
            lcchild = &mut *secondary;
            layout_cells_push_back(lcroot, secondary);
            layout_set_size(
                lcchild,
                sx,
                otherh,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        } else {
            let mut lcother_owner = layout_create_cell();
            lcother = &mut *lcother_owner;
            layout_set_size(
                lcother,
                sx,
                otherh,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            layout_make_node(lcother, LAYOUT_LEFTRIGHT);
            layout_cells_push_back(lcroot, lcother_owner);
            let mut pane_iter = cells.iter();
            wp = pane_iter.next();
            while wp.is_some() {
                if *wp.expect("pane") != main_id {
                    lcchild = layout_set_leaf(&mut leaves, *wp.expect("pane")) as *mut layout_cell;
                    layout_cells_push_back(lcother, layout_take_leaf(&mut leaves, (*lcchild).id()));
                    (*lcchild).parent = lcother;
                    layout_set_size(
                        lcchild,
                        PANE_MINIMUM as u_int,
                        otherh,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
                wp = pane_iter.next();
            }
            layout_spread_cell_with_limits(lcroot, pane_status, horizontal_minimum, lcother);
        }
        assert!(leaves.is_empty(), "all detached pane cells were reinserted");
    } // All tree pointers and the component guard end before pane callbacks.
    layout_fix_offsets(w_owner);
    {
        let mut tree = w_owner.borrow_layout_root_mut();
        let root = tree.as_deref_mut().expect("preset layout root");
        layout_print_cell(root, c"layout_set_main_h".as_ptr(), 1);
    }
    true
}
unsafe fn layout_set_main_h_mirrored(w_owner: &WindowRef, window_size: (u_int, u_int)) -> bool {
    {
        let cells = layout_set_cells(w_owner);

        let mut wp = None;
        let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcmain: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut n: u_int = 0;
        let mut mainh: u_int = 0;
        let mut otherh: u_int = 0;
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        {
            let mut tree = w_owner.borrow_layout_root_mut();
            layout_print_cell(
                tree.as_deref_mut()
                    .map_or(std::ptr::null_mut(), |root| root),
                c"layout_set_main_h_mirrored".as_ptr(),
                1,
            );
        }
        n = cells.len() as u_int;
        if n <= 1 as u_int {
            return false;
        }
        n = n.wrapping_sub(1);
        sy = window_size.1.wrapping_sub(1 as u_int);
        let size_option =
            w_owner.with_options_mut(|options| options_get_string(options, c"main-pane-height"));
        s = size_option.as_ptr();
        mainh = match args_string_percentage_result(
            (!s.is_null()).then(|| ::std::ffi::CStr::from_ptr(s)),
            sy as ::core::ffi::c_longlong,
            sy as ::core::ffi::c_longlong,
        ) {
            Ok(value) => value as u_int,
            Err(_) => 24 as u_int,
        };
        if mainh.wrapping_add(PANE_MINIMUM as u_int) >= sy {
            if sy <= (PANE_MINIMUM + PANE_MINIMUM) as u_int {
                mainh = PANE_MINIMUM as u_int;
            } else {
                mainh = sy.wrapping_sub(PANE_MINIMUM as u_int);
            }
            otherh = PANE_MINIMUM as u_int;
        } else {
            let size_option = w_owner
                .with_options_mut(|options| options_get_string(options, c"other-pane-height"));
            s = size_option.as_ptr();
            otherh = match args_string_percentage_result(
                (!s.is_null()).then(|| ::std::ffi::CStr::from_ptr(s)),
                sy as ::core::ffi::c_longlong,
                sy as ::core::ffi::c_longlong,
            ) {
                Ok(value) => value as u_int,
                Err(_) => 0 as u_int,
            };
            if otherh == 0 as u_int || otherh > sy || sy.wrapping_sub(otherh) < mainh {
                otherh = sy.wrapping_sub(mainh);
            } else {
                mainh = sy.wrapping_sub(otherh);
            }
        }
        sx = n
            .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
            .wrapping_sub(1 as u_int);
        if sx < window_size.0 {
            sx = window_size.0;
        }
        let (pane_status, horizontal_minimum) = layout_resize_limits(w_owner);
        let mut tree = w_owner.borrow_layout_root_mut();
        let mut leaves = layout_take_leaves(tree.take());
        *tree = Some(layout_create_cell());
        lcroot = tree.as_deref_mut().expect("preset layout root");
        layout_set_size(
            lcroot,
            sx,
            mainh.wrapping_add(otherh).wrapping_add(1 as u_int),
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_make_node(lcroot, LAYOUT_TOPBOTTOM);
        let main_id = *cells.first().expect("main pane");
        lcmain = layout_set_leaf(&mut leaves, main_id) as *mut layout_cell;
        (*lcmain).parent = lcroot;
        layout_set_size(
            lcmain,
            sx,
            mainh,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_cells_push_back(lcroot, layout_take_leaf(&mut leaves, (*lcmain).id()));
        if n == 1 as u_int {
            let mut pane_iter = cells.iter().skip_while(|id| **id != main_id).skip(1);
            wp = pane_iter.next();
            let mut secondary = layout_take_leaf(&mut leaves, *wp.expect("pane"));
            lcchild = &mut *secondary;
            layout_cells_push_front(lcroot, secondary);
            layout_set_size(
                lcchild,
                sx,
                otherh,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        } else {
            let mut lcother_owner = layout_create_cell();
            lcother = &mut *lcother_owner;
            layout_set_size(
                lcother,
                sx,
                otherh,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            layout_make_node(lcother, LAYOUT_LEFTRIGHT);
            layout_cells_push_front(lcroot, lcother_owner);
            let mut pane_iter = cells.iter();
            wp = pane_iter.next();
            while wp.is_some() {
                if *wp.expect("pane") != main_id {
                    lcchild = layout_set_leaf(&mut leaves, *wp.expect("pane")) as *mut layout_cell;
                    layout_cells_push_back(lcother, layout_take_leaf(&mut leaves, (*lcchild).id()));
                    (*lcchild).parent = lcother;
                    layout_set_size(
                        lcchild,
                        PANE_MINIMUM as u_int,
                        otherh,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
                wp = pane_iter.next();
            }
            layout_spread_cell_with_limits(lcroot, pane_status, horizontal_minimum, lcother);
        }
        assert!(leaves.is_empty(), "all detached pane cells were reinserted");
    } // All tree pointers and the component guard end before pane callbacks.
    layout_fix_offsets(w_owner);
    {
        let mut tree = w_owner.borrow_layout_root_mut();
        let root = tree.as_deref_mut().expect("preset layout root");
        layout_print_cell(root, c"layout_set_main_h_mirrored".as_ptr(), 1);
    }
    true
}
unsafe fn layout_set_main_v(w_owner: &WindowRef, window_size: (u_int, u_int)) -> bool {
    {
        let cells = layout_set_cells(w_owner);

        let mut wp = None;
        let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcmain: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut n: u_int = 0;
        let mut mainw: u_int = 0;
        let mut otherw: u_int = 0;
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        {
            let mut tree = w_owner.borrow_layout_root_mut();
            layout_print_cell(
                tree.as_deref_mut()
                    .map_or(std::ptr::null_mut(), |root| root),
                c"layout_set_main_v".as_ptr(),
                1,
            );
        }
        n = cells.len() as u_int;
        if n <= 1 as u_int {
            return false;
        }
        n = n.wrapping_sub(1);
        sx = window_size.0.wrapping_sub(1 as u_int);
        let size_option =
            w_owner.with_options_mut(|options| options_get_string(options, c"main-pane-width"));
        s = size_option.as_ptr();
        mainw = match args_string_percentage_result(
            (!s.is_null()).then(|| ::std::ffi::CStr::from_ptr(s)),
            sx as ::core::ffi::c_longlong,
            sx as ::core::ffi::c_longlong,
        ) {
            Ok(value) => value as u_int,
            Err(_) => 80 as u_int,
        };
        if mainw.wrapping_add(PANE_MINIMUM as u_int) >= sx {
            if sx <= (PANE_MINIMUM + PANE_MINIMUM) as u_int {
                mainw = PANE_MINIMUM as u_int;
            } else {
                mainw = sx.wrapping_sub(PANE_MINIMUM as u_int);
            }
            otherw = PANE_MINIMUM as u_int;
        } else {
            let size_option = w_owner
                .with_options_mut(|options| options_get_string(options, c"other-pane-width"));
            s = size_option.as_ptr();
            otherw = match args_string_percentage_result(
                (!s.is_null()).then(|| ::std::ffi::CStr::from_ptr(s)),
                sx as ::core::ffi::c_longlong,
                sx as ::core::ffi::c_longlong,
            ) {
                Ok(value) => value as u_int,
                Err(_) => 0 as u_int,
            };
            if otherw == 0 as u_int || otherw > sx || sx.wrapping_sub(otherw) < mainw {
                otherw = sx.wrapping_sub(mainw);
            } else {
                mainw = sx.wrapping_sub(otherw);
            }
        }
        sy = n
            .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
            .wrapping_sub(1 as u_int);
        if sy < window_size.1 {
            sy = window_size.1;
        }
        let (pane_status, horizontal_minimum) = layout_resize_limits(w_owner);
        let mut tree = w_owner.borrow_layout_root_mut();
        let mut leaves = layout_take_leaves(tree.take());
        *tree = Some(layout_create_cell());
        lcroot = tree.as_deref_mut().expect("preset layout root");
        layout_set_size(
            lcroot,
            mainw.wrapping_add(otherw).wrapping_add(1 as u_int),
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_make_node(lcroot, LAYOUT_LEFTRIGHT);
        let main_id = *cells.first().expect("main pane");
        lcmain = layout_set_leaf(&mut leaves, main_id) as *mut layout_cell;
        (*lcmain).parent = lcroot;
        layout_set_size(
            lcmain,
            mainw,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_cells_push_back(lcroot, layout_take_leaf(&mut leaves, (*lcmain).id()));
        if n == 1 as u_int {
            let mut pane_iter = cells.iter().skip_while(|id| **id != main_id).skip(1);
            wp = pane_iter.next();
            let mut secondary = layout_take_leaf(&mut leaves, *wp.expect("pane"));
            lcchild = &mut *secondary;
            layout_cells_push_back(lcroot, secondary);
            layout_set_size(
                lcchild,
                otherw,
                sy,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        } else {
            let mut lcother_owner = layout_create_cell();
            lcother = &mut *lcother_owner;
            layout_make_node(lcother, LAYOUT_TOPBOTTOM);
            layout_set_size(
                lcother,
                otherw,
                sy,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            layout_cells_push_back(lcroot, lcother_owner);
            let mut pane_iter = cells.iter();
            wp = pane_iter.next();
            while wp.is_some() {
                if *wp.expect("pane") != main_id {
                    lcchild = layout_set_leaf(&mut leaves, *wp.expect("pane")) as *mut layout_cell;
                    layout_cells_push_back(lcother, layout_take_leaf(&mut leaves, (*lcchild).id()));
                    (*lcchild).parent = lcother;
                    layout_set_size(
                        lcchild,
                        otherw,
                        PANE_MINIMUM as u_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
                wp = pane_iter.next();
            }
            layout_spread_cell_with_limits(lcroot, pane_status, horizontal_minimum, lcother);
        }
        assert!(leaves.is_empty(), "all detached pane cells were reinserted");
    } // All tree pointers and the component guard end before pane callbacks.
    layout_fix_offsets(w_owner);
    {
        let mut tree = w_owner.borrow_layout_root_mut();
        let root = tree.as_deref_mut().expect("preset layout root");
        layout_print_cell(root, c"layout_set_main_v".as_ptr(), 1);
    }
    true
}
unsafe fn layout_set_main_v_mirrored(w_owner: &WindowRef, window_size: (u_int, u_int)) -> bool {
    {
        let cells = layout_set_cells(w_owner);

        let mut wp = None;
        let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcmain: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut n: u_int = 0;
        let mut mainw: u_int = 0;
        let mut otherw: u_int = 0;
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        {
            let mut tree = w_owner.borrow_layout_root_mut();
            layout_print_cell(
                tree.as_deref_mut()
                    .map_or(std::ptr::null_mut(), |root| root),
                c"layout_set_main_v_mirrored".as_ptr(),
                1,
            );
        }
        n = cells.len() as u_int;
        if n <= 1 as u_int {
            return false;
        }
        n = n.wrapping_sub(1);
        sx = window_size.0.wrapping_sub(1 as u_int);
        let size_option =
            w_owner.with_options_mut(|options| options_get_string(options, c"main-pane-width"));
        s = size_option.as_ptr();
        mainw = match args_string_percentage_result(
            (!s.is_null()).then(|| ::std::ffi::CStr::from_ptr(s)),
            sx as ::core::ffi::c_longlong,
            sx as ::core::ffi::c_longlong,
        ) {
            Ok(value) => value as u_int,
            Err(_) => 80 as u_int,
        };
        if mainw.wrapping_add(PANE_MINIMUM as u_int) >= sx {
            if sx <= (PANE_MINIMUM + PANE_MINIMUM) as u_int {
                mainw = PANE_MINIMUM as u_int;
            } else {
                mainw = sx.wrapping_sub(PANE_MINIMUM as u_int);
            }
            otherw = PANE_MINIMUM as u_int;
        } else {
            let size_option = w_owner
                .with_options_mut(|options| options_get_string(options, c"other-pane-width"));
            s = size_option.as_ptr();
            otherw = match args_string_percentage_result(
                (!s.is_null()).then(|| ::std::ffi::CStr::from_ptr(s)),
                sx as ::core::ffi::c_longlong,
                sx as ::core::ffi::c_longlong,
            ) {
                Ok(value) => value as u_int,
                Err(_) => 0 as u_int,
            };
            if otherw == 0 as u_int || otherw > sx || sx.wrapping_sub(otherw) < mainw {
                otherw = sx.wrapping_sub(mainw);
            } else {
                mainw = sx.wrapping_sub(otherw);
            }
        }
        sy = n
            .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
            .wrapping_sub(1 as u_int);
        if sy < window_size.1 {
            sy = window_size.1;
        }
        let (pane_status, horizontal_minimum) = layout_resize_limits(w_owner);
        let mut tree = w_owner.borrow_layout_root_mut();
        let mut leaves = layout_take_leaves(tree.take());
        *tree = Some(layout_create_cell());
        lcroot = tree.as_deref_mut().expect("preset layout root");
        layout_set_size(
            lcroot,
            mainw.wrapping_add(otherw).wrapping_add(1 as u_int),
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_make_node(lcroot, LAYOUT_LEFTRIGHT);
        let main_id = *cells.first().expect("main pane");
        lcmain = layout_set_leaf(&mut leaves, main_id) as *mut layout_cell;
        (*lcmain).parent = lcroot;
        layout_set_size(
            lcmain,
            mainw,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_cells_push_back(lcroot, layout_take_leaf(&mut leaves, (*lcmain).id()));
        if n == 1 as u_int {
            let mut pane_iter = cells.iter().skip_while(|id| **id != main_id).skip(1);
            wp = pane_iter.next();
            let mut secondary = layout_take_leaf(&mut leaves, *wp.expect("pane"));
            lcchild = &mut *secondary;
            layout_cells_push_front(lcroot, secondary);
            layout_set_size(
                lcchild,
                otherw,
                sy,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        } else {
            let mut lcother_owner = layout_create_cell();
            lcother = &mut *lcother_owner;
            layout_make_node(lcother, LAYOUT_TOPBOTTOM);
            layout_set_size(
                lcother,
                otherw,
                sy,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            layout_cells_push_front(lcroot, lcother_owner);
            let mut pane_iter = cells.iter();
            wp = pane_iter.next();
            while wp.is_some() {
                if *wp.expect("pane") != main_id {
                    lcchild = layout_set_leaf(&mut leaves, *wp.expect("pane")) as *mut layout_cell;
                    layout_cells_push_back(lcother, layout_take_leaf(&mut leaves, (*lcchild).id()));
                    (*lcchild).parent = lcother;
                    layout_set_size(
                        lcchild,
                        otherw,
                        PANE_MINIMUM as u_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
                wp = pane_iter.next();
            }
            layout_spread_cell_with_limits(lcroot, pane_status, horizontal_minimum, lcother);
        }
        assert!(leaves.is_empty(), "all detached pane cells were reinserted");
    } // All tree pointers and the component guard end before pane callbacks.
    layout_fix_offsets(w_owner);
    {
        let mut tree = w_owner.borrow_layout_root_mut();
        let root = tree.as_deref_mut().expect("preset layout root");
        layout_print_cell(root, c"layout_set_main_v_mirrored".as_ptr(), 1);
    }
    true
}
unsafe fn layout_set_tiled(w_owner: &WindowRef, window_size: (u_int, u_int)) -> bool {
    {
        let cells = layout_set_cells(w_owner);

        let mut wp = None;
        let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcrow: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut n: u_int = 0;
        let mut width: u_int = 0;
        let mut height: u_int = 0;
        let mut used: u_int = 0;
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut i: u_int = 0;
        let mut j: u_int = 0;
        let mut columns: u_int = 0;
        let mut rows: u_int = 0;
        let mut max_columns: u_int = 0;
        {
            let mut tree = w_owner.borrow_layout_root_mut();
            layout_print_cell(
                tree.as_deref_mut()
                    .map_or(std::ptr::null_mut(), |root| root),
                c"layout_set_tiled".as_ptr(),
                1,
            );
        }
        n = cells.len() as u_int;
        if n <= 1 as u_int {
            return false;
        }
        max_columns = w_owner
            .with_options_mut(|options| options_get_number(options, c"tiled-layout-max-columns"))
            as u_int;
        columns = 1 as u_int;
        rows = columns;
        while rows.wrapping_mul(columns) < n {
            rows = rows.wrapping_add(1);
            if rows.wrapping_mul(columns) < n
                && (max_columns == 0 as u_int || columns < max_columns)
            {
                columns = columns.wrapping_add(1);
            }
        }
        width = window_size
            .0
            .wrapping_sub(columns.wrapping_sub(1 as u_int))
            .wrapping_div(columns);
        if width < PANE_MINIMUM as u_int {
            width = PANE_MINIMUM as u_int;
        }
        height = window_size
            .1
            .wrapping_sub(rows.wrapping_sub(1 as u_int))
            .wrapping_div(rows);
        if height < PANE_MINIMUM as u_int {
            height = PANE_MINIMUM as u_int;
        }
        sx = width
            .wrapping_add(1 as u_int)
            .wrapping_mul(columns)
            .wrapping_sub(1 as u_int);
        if sx < window_size.0 {
            sx = window_size.0;
        }
        sy = height
            .wrapping_add(1 as u_int)
            .wrapping_mul(rows)
            .wrapping_sub(1 as u_int);
        if sy < window_size.1 {
            sy = window_size.1;
        }
        let (pane_status, horizontal_minimum) = layout_resize_limits(w_owner);
        let mut tree = w_owner.borrow_layout_root_mut();
        let mut leaves = layout_take_leaves(tree.take());
        *tree = Some(layout_create_cell());
        lcroot = tree.as_deref_mut().expect("preset layout root");
        layout_set_size(
            lcroot,
            sx,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_make_node(lcroot, LAYOUT_TOPBOTTOM);
        let mut pane_iter = cells.iter();
        wp = pane_iter.next();
        j = 0 as u_int;
        while j < rows {
            if wp.is_none() {
                break;
            }
            lcchild = layout_set_leaf(&mut leaves, *wp.expect("pane")) as *mut layout_cell;
            if n.wrapping_sub(j.wrapping_mul(columns)) == 1 as u_int || columns == 1 as u_int {
                (*lcchild).parent = lcroot;
                layout_cells_push_back(lcroot, layout_take_leaf(&mut leaves, (*lcchild).id()));
                layout_set_size(
                    lcchild,
                    window_size.0,
                    height,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                wp = pane_iter.next();
            } else {
                let mut lcrow_owner = layout_create_cell();
                lcrow = &mut *lcrow_owner;
                layout_make_node(lcrow, LAYOUT_LEFTRIGHT);
                layout_set_size(
                    lcrow,
                    window_size.0,
                    height,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                layout_cells_push_back(lcroot, lcrow_owner);
                i = 0 as u_int;
                while i < columns {
                    (*lcchild).parent = lcrow;
                    layout_cells_push_back(lcrow, layout_take_leaf(&mut leaves, (*lcchild).id()));
                    layout_set_size(
                        lcchild,
                        width,
                        height,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    wp = pane_iter.next();
                    if wp.is_none() {
                        break;
                    }
                    lcchild = layout_set_leaf(&mut leaves, *wp.expect("pane")) as *mut layout_cell;
                    i = i.wrapping_add(1);
                }
                if i == columns {
                    i = i.wrapping_sub(1);
                }
                used = i
                    .wrapping_add(1 as u_int)
                    .wrapping_mul(width.wrapping_add(1 as u_int))
                    .wrapping_sub(1 as u_int);
                if !(window_size.0 <= used) {
                    lcchild = layout_cells_last(&*lcrow);
                    layout_resize_adjust_with_limits(
                        lcroot,
                        pane_status,
                        horizontal_minimum,
                        lcchild,
                        LAYOUT_LEFTRIGHT,
                        window_size.0.wrapping_sub(used) as ::core::ffi::c_int,
                    );
                }
            }
            j = j.wrapping_add(1);
        }
        used = rows
            .wrapping_mul(height)
            .wrapping_add(rows)
            .wrapping_sub(1 as u_int);
        if window_size.1 > used {
            lcrow = layout_cells_last(&*lcroot);
            layout_resize_adjust_with_limits(
                lcroot,
                pane_status,
                horizontal_minimum,
                lcrow,
                LAYOUT_TOPBOTTOM,
                window_size.1.wrapping_sub(used) as ::core::ffi::c_int,
            );
        }
        assert!(leaves.is_empty(), "all detached pane cells were reinserted");
    } // All tree pointers and the component guard end before pane callbacks.
    layout_fix_offsets(w_owner);
    {
        let mut tree = w_owner.borrow_layout_root_mut();
        let root = tree.as_deref_mut().expect("preset layout root");
        layout_print_cell(root, c"layout_set_tiled".as_ptr(), 1);
    }
    true
}

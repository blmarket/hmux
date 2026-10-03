use crate::src::arguments::{
    args_has, args_percentage_and_expand_result, args_strtonum_and_expand_result,
};
use crate::src::events::events_fire_window;
use crate::src::log::{fatalx, log_cstr, log_cstr_width, log_debug, log_pointer};
use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args;
pub use crate::src::shared::command::cmdq_item;
pub use crate::src::shared::environment::environ;
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::key::key_event;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cells};
pub use crate::src::shared::limits::{INT_MAX, UINT_MAX};
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::window_pane;
pub use crate::src::shared::pane::{
    PANE_MINIMUM, PANE_REDRAWSCROLLBAR, PANE_SCROLLBARS_ALWAYS, PANE_SCROLLBARS_LEFT,
    PANE_STATUS_BOTTOM, PANE_STATUS_TOP,
};
pub use crate::src::shared::spawn::{SPAWN_BEFORE, SPAWN_FULLSIZE, SPAWN_HORIZONTAL, SPAWN_ZOOM};
use crate::src::shared::style::*;
pub use crate::src::shared::tty::tty_term;
pub use crate::src::shared::window::window;
use crate::src::shared::window::WindowRef;
use crate::src::window::Window as _;
use crate::src::window_pane::WindowPane as _;

pub fn layout_create_cell() -> Box<layout_cell> {
    layout_cell::new()
}

/// Free internal nodes while keeping detached pane cells alive for a new layout.
pub fn layout_take_leaves(root: Option<Box<layout_cell>>) -> Vec<Box<layout_cell>> {
    fn collect(mut cell: Box<layout_cell>, leaves: &mut Vec<Box<layout_cell>>) {
        cell.parent = std::ptr::null_mut();
        cell.sibling_index = 0;
        if cell.type_0 == LAYOUT_WINDOWPANE {
            leaves.push(cell);
        } else {
            for child in std::mem::take(&mut cell.cells) {
                collect(child, leaves);
            }
        }
    }
    let mut leaves = Vec::new();
    if let Some(root) = root {
        collect(root, &mut leaves);
    }
    leaves
}

pub fn layout_take_leaf(
    leaves: &mut Vec<Box<layout_cell>>,
    id: *mut layout_cell,
) -> Box<layout_cell> {
    let index = leaves
        .iter()
        .position(|leaf| leaf.id() == id)
        .expect("detached layout leaf is owned");
    leaves.remove(index)
}

pub unsafe fn layout_print_cell(
    mut lc: *mut layout_cell,
    mut hdr: *const ::core::ffi::c_char,
    mut n: u_int,
) {
    let _lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut type_0: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if lc.is_null() {
        return;
    }
    match (*lc).type_0 as ::core::ffi::c_uint {
        0 => {
            type_0 = c"LEFTRIGHT".as_ptr();
        }
        1 => {
            type_0 = c"TOPBOTTOM".as_ptr();
        }
        2 => {
            type_0 = c"WINDOWPANE".as_ptr();
        }
        _ => {
            type_0 = c"UNKNOWN".as_ptr();
        }
    }
    log_debug(format_args!(
        "{}:{}{} type {} [parent {}] wp={} [{},{} {}x{}]",
        log_cstr((hdr) as *const _),
        log_cstr_width((c" ".as_ptr()) as *const _, n as i32),
        log_pointer((lc) as *const ::core::ffi::c_void),
        log_cstr((type_0) as *const _),
        log_pointer(((*lc).parent) as *const ::core::ffi::c_void),
        log_pointer((*lc).wp.as_ptr().cast()),
        { (*lc).g.xoff },
        { (*lc).g.yoff },
        { (*lc).g.sx },
        { (*lc).g.sy }
    ));
    match (*lc).type_0 as ::core::ffi::c_uint {
        0 | 1 => {
            let children = (*lc)
                .cells
                .iter()
                .map(|child| &**child as *const layout_cell as *mut layout_cell)
                .collect::<Vec<_>>();
            for lcchild in children {
                layout_print_cell(lcchild, hdr, n.wrapping_add(1 as u_int));
            }
        }
        _ => {}
    };
}
pub unsafe fn layout_search_by_border(
    mut lc: *mut layout_cell,
    mut x: u_int,
    mut y: u_int,
) -> *mut layout_cell {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut last: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    lcchild = layout_cells_first(&*lc);
    while !lcchild.is_null() {
        if x as ::core::ffi::c_int >= (*lcchild).g.xoff
            && (x as ::core::ffi::c_int) < (*lcchild).g.xoff + (*lcchild).g.sx as ::core::ffi::c_int
            && y as ::core::ffi::c_int >= (*lcchild).g.yoff
            && (y as ::core::ffi::c_int) < (*lcchild).g.yoff + (*lcchild).g.sy as ::core::ffi::c_int
        {
            return layout_search_by_border(lcchild, x, y);
        }
        if last.is_null() {
            last = lcchild;
        } else {
            match (*lc).type_0 as ::core::ffi::c_uint {
                0 => {
                    if (x as ::core::ffi::c_int) < (*lcchild).g.xoff
                        && x as ::core::ffi::c_int
                            >= (*last).g.xoff + (*last).g.sx as ::core::ffi::c_int
                    {
                        return last;
                    }
                }
                1 if (y as ::core::ffi::c_int) < (*lcchild).g.yoff
                    && y as ::core::ffi::c_int
                        >= (*last).g.yoff + (*last).g.sy as ::core::ffi::c_int =>
                {
                    return last;
                }
                _ => {}
            }
            last = lcchild;
        }
        lcchild = layout_cell_next(lcchild);
    }
    ::core::ptr::null_mut::<layout_cell>()
}
pub unsafe fn layout_set_size(
    mut lc: *mut layout_cell,
    mut sx: u_int,
    mut sy: u_int,
    mut xoff: ::core::ffi::c_int,
    mut yoff: ::core::ffi::c_int,
) {
    (*lc).g.sx = sx;
    (*lc).g.sy = sy;
    (*lc).g.xoff = xoff;
    (*lc).g.yoff = yoff;
}
pub unsafe fn layout_make_leaf(
    lc: *mut layout_cell,
    owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
) {
    (*lc).type_0 = LAYOUT_WINDOWPANE;
    layout_cells_require_empty(&*lc);
    owner.place_in_layout((*lc).id());
    (*lc).wp = std::rc::Rc::downgrade(owner);
}
pub unsafe fn layout_make_node(mut lc: *mut layout_cell, mut type_0: layout_type) {
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fatalx(|out| out.write_all(b"bad layout type"));
    }
    (*lc).type_0 = type_0;
    layout_cells_require_empty(&*lc);
    if let Some(owner) = (*lc).wp.upgrade() {
        owner.detach_layout((*lc).id());
    }
    (*lc).wp = std::rc::Weak::new();
}
unsafe fn layout_fix_offsets1(mut lc: *mut layout_cell) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        xoff = (*lc).g.xoff;
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            (*lcchild).g.xoff = xoff;
            (*lcchild).g.yoff = (*lc).g.yoff;
            if (*lcchild).type_0 as ::core::ffi::c_uint
                != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                layout_fix_offsets1(lcchild);
            }
            xoff = (xoff as u_int).wrapping_add((*lcchild).g.sx.wrapping_add(1 as u_int))
                as ::core::ffi::c_int as ::core::ffi::c_int;
            lcchild = layout_cell_next(lcchild);
        }
    } else {
        yoff = (*lc).g.yoff;
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            (*lcchild).g.xoff = (*lc).g.xoff;
            (*lcchild).g.yoff = yoff;
            if (*lcchild).type_0 as ::core::ffi::c_uint
                != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                layout_fix_offsets1(lcchild);
            }
            yoff = (yoff as u_int).wrapping_add((*lcchild).g.sy.wrapping_add(1 as u_int))
                as ::core::ffi::c_int as ::core::ffi::c_int;
            lcchild = layout_cell_next(lcchild);
        }
    };
}
pub unsafe fn layout_fix_offsets(w_owner: &WindowRef) {
    let mut tree = w_owner.borrow_layout_root_mut();
    let lc = tree.as_deref_mut().expect("layout root") as *mut layout_cell;
    (*lc).g.xoff = 0 as ::core::ffi::c_int;
    (*lc).g.yoff = 0 as ::core::ffi::c_int;
    layout_fix_offsets1(lc);
}
unsafe fn layout_cell_is_top(
    mut root: *mut layout_cell,
    mut lc: *mut layout_cell,
) -> ::core::ffi::c_int {
    let mut next: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    while lc != root {
        next = (*lc).parent;
        if next.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        if (*next).type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            && lc != layout_cells_first(&*next)
        {
            return 0 as ::core::ffi::c_int;
        }
        lc = next;
    }
    1 as ::core::ffi::c_int
}
unsafe fn layout_cell_is_bottom(
    mut root: *mut layout_cell,
    mut lc: *mut layout_cell,
) -> ::core::ffi::c_int {
    let mut next: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    while lc != root {
        next = (*lc).parent;
        if next.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        if (*next).type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            && lc != layout_cells_last(&*next)
        {
            return 0 as ::core::ffi::c_int;
        }
        lc = next;
    }
    1 as ::core::ffi::c_int
}
pub unsafe fn layout_add_horizontal_border(
    mut root: *mut layout_cell,
    mut lc: *mut layout_cell,
    mut status: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if status == PANE_STATUS_TOP {
        return layout_cell_is_top(root, lc);
    }
    if status == PANE_STATUS_BOTTOM {
        return layout_cell_is_bottom(root, lc);
    }
    0 as ::core::ffi::c_int
}
/// Apply cell geometry to the panes; returns whether any pane changed.
pub unsafe fn layout_fix_panes(
    window: &WindowRef,
    skip: Option<&std::rc::Rc<std::cell::UnsafeCell<window_pane>>>,
) -> bool {
    let mut changed = false;
    let mut cursor = window.next_pane(None);
    while let Some(pane) = cursor {
        if pane.layout_identity(false).is_some()
            && !skip.is_some_and(|skip| std::rc::Rc::ptr_eq(&pane, skip))
        {
            // Read placement before resizing; a callback may replace the tree
            // before the next pane is visited.
            let geometry = window
                .pane_layout_cell(
                    &std::rc::Rc::downgrade(&pane),
                    crate::src::window::LayoutView::Visible,
                )
                .expect("placed pane belongs to visible layout");
            changed |= pane.apply_layout(geometry, window);
        }
        cursor = window.next_pane(Some(&pane));
    }
    if changed {
        window.invalidate_scene();
    }
    changed
}
pub unsafe fn layout_count_cells(mut lc: *mut layout_cell) -> u_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut count: u_int = 0 as u_int;
    match (*lc).type_0 as ::core::ffi::c_uint {
        2 => 1 as u_int,
        0 | 1 => {
            lcchild = layout_cells_first(&*lc);
            while !lcchild.is_null() {
                count = count.wrapping_add(layout_count_cells(lcchild));
                lcchild = layout_cell_next(lcchild);
            }
            count
        }
        _ => {
            fatalx(|out| out.write_all(b"bad layout type"));
        }
    }
}
pub(super) unsafe fn layout_resize_limits(owner: &WindowRef) -> (i32, u32) {
    let pane_status = owner.pane_border_status();
    let horizontal_minimum = if owner.scrollbar_mode() == PANE_SCROLLBARS_ALWAYS {
        owner
            .active_pane()
            .expect("active layout pane")
            .minimum_layout_width(true)
    } else {
        PANE_MINIMUM as u32
    };
    (pane_status, horizontal_minimum)
}
pub(super) unsafe fn layout_resize_check_with_limits(
    root: *mut layout_cell,
    pane_status: i32,
    horizontal_minimum: u32,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
) -> u_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut();
    let mut available: u_int = 0;
    let mut minimum: u_int = 0;
    let mut status: ::core::ffi::c_int = 0;
    status = pane_status;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            available = (*lc).g.sx;
            minimum = horizontal_minimum;
        } else {
            available = (*lc).g.sy;
            if layout_add_horizontal_border(root, lc, status) != 0 {
                minimum = (PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int;
            } else {
                minimum = PANE_MINIMUM as u_int;
            }
        }
        if available > minimum {
            available = available.wrapping_sub(minimum);
        } else {
            available = 0 as u_int;
        }
    } else if (*lc).type_0 as ::core::ffi::c_uint == type_0 as ::core::ffi::c_uint {
        available = 0 as u_int;
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            available = available.wrapping_add(layout_resize_check_with_limits(
                root,
                pane_status,
                horizontal_minimum,
                lcchild,
                type_0,
            ));
            lcchild = layout_cell_next(lcchild);
        }
    } else {
        minimum = UINT_MAX as u_int;
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            available = layout_resize_check_with_limits(
                root,
                pane_status,
                horizontal_minimum,
                lcchild,
                type_0,
            );
            if available < minimum {
                minimum = available;
            }
            lcchild = layout_cell_next(lcchild);
        }
        available = minimum;
    }
    available
}
pub(super) unsafe fn layout_resize_adjust_with_limits(
    root: *mut layout_cell,
    pane_status: i32,
    horizontal_minimum: u32,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut change: ::core::ffi::c_int,
) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut changed: ::core::ffi::c_int = 0;
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*lc).g.sx = (*lc).g.sx.wrapping_add(change as u_int);
    } else {
        (*lc).g.sy = (*lc).g.sy.wrapping_add(change as u_int);
    }
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    if (*lc).type_0 as ::core::ffi::c_uint != type_0 as ::core::ffi::c_uint {
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            layout_resize_adjust_with_limits(
                root,
                pane_status,
                horizontal_minimum,
                lcchild,
                type_0,
                change,
            );
            lcchild = layout_cell_next(lcchild);
        }
        return;
    }
    while change != 0 as ::core::ffi::c_int {
        changed = 0 as ::core::ffi::c_int;
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            if change == 0 as ::core::ffi::c_int {
                break;
            }
            if change > 0 as ::core::ffi::c_int {
                layout_resize_adjust_with_limits(
                    root,
                    pane_status,
                    horizontal_minimum,
                    lcchild,
                    type_0,
                    1 as ::core::ffi::c_int,
                );
                change -= 1;
                changed = 1 as ::core::ffi::c_int;
            } else if layout_resize_check_with_limits(
                root,
                pane_status,
                horizontal_minimum,
                lcchild,
                type_0,
            ) > 0 as u_int
            {
                layout_resize_adjust_with_limits(
                    root,
                    pane_status,
                    horizontal_minimum,
                    lcchild,
                    type_0,
                    -(1 as ::core::ffi::c_int),
                );
                change += 1;
                changed = 1 as ::core::ffi::c_int;
            }
            lcchild = layout_cell_next(lcchild);
        }
        if changed == 0 {
            break;
        }
    }
}
pub unsafe fn layout_cell_get_neighbour(lc: *mut layout_cell) -> *mut layout_cell {
    let lcparent: *mut layout_cell = (*lc).parent;
    if lcparent.is_null() {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    if lc == layout_cells_last(&*lcparent) {
        layout_cell_prev(lc)
    } else {
        layout_cell_next(lc)
    }
}
/// Removal from a detached parser-owned tree. Live Window removal uses the
/// pure helper with policy copied before its root is borrowed.
pub unsafe fn layout_destroy_cell(
    window: Option<&WindowRef>,
    cell: *mut layout_cell,
    root: &mut Option<Box<layout_cell>>,
) {
    let (pane_status, horizontal_minimum) = if !(*cell).parent.is_null() {
        layout_resize_limits(window.expect("cell removal requires a live window"))
    } else {
        (0, 0)
    };
    layout_destroy_cell_with_limits(pane_status, horizontal_minimum, cell, root);
}
unsafe fn layout_destroy_cell_with_limits(
    pane_status: i32,
    horizontal_minimum: u32,
    lc: *mut layout_cell,
    root: &mut Option<Box<layout_cell>>,
) {
    let parent = (*lc).parent;
    if parent.is_null() {
        assert!(root.as_deref().is_some_and(|owner| std::ptr::eq(owner, lc)));
        drop(root.take());
        return;
    }
    let other = layout_cell_get_neighbour(lc);
    if !other.is_null() {
        let change = if (*parent).type_0 == LAYOUT_LEFTRIGHT {
            (*lc).g.sx.wrapping_add(1)
        } else {
            (*lc).g.sy.wrapping_add(1)
        };
        layout_resize_adjust_with_limits(
            root.as_deref_mut().unwrap(),
            pane_status,
            horizontal_minimum,
            other,
            (*parent).type_0,
            change as i32,
        );
    }
    drop(layout_cells_remove(parent, lc).expect("removed cell is owned"));
    if (*parent).cells.len() == 1 {
        let child = layout_cells_first(&*parent);
        let mut child_owner = layout_cells_remove(parent, child).expect("remaining child is owned");
        let grandparent = (*parent).parent;
        if grandparent.is_null() {
            child_owner.g.xoff = 0;
            child_owner.g.yoff = 0;
            *root = Some(child_owner);
        } else {
            drop(layout_cells_replace(grandparent, parent, child_owner));
        }
    }
}

/// Area the arranged window occupies: per dimension, the larger of the
/// window size and the visible tiled root. Clients clip and pan across it.
pub unsafe fn logical_size(w_owner: &WindowRef) -> (u32, u32) {
    let (sx, sy) = w_owner.size();
    let Some(root) = w_owner.borrow_layout_root(crate::src::window::LayoutView::Visible) else {
        return (sx, sy);
    };
    (sx.max(root.g.sx), sy.max(root.g.sy))
}

pub unsafe fn layout_init(
    w_owner: &WindowRef,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
) {
    let (sx, sy) = w_owner.size();
    {
        let mut tree = w_owner.borrow_layout_root_mut();
        *tree = Some(layout_create_cell());
        let root = tree.as_deref_mut().expect("initial layout root");
        layout_set_size(root, sx, sy, 0, 0);
        layout_make_leaf(root, wp_owner);
    }
    layout_fix_panes(w_owner, None);
}
pub unsafe fn layout_free(w_owner: &WindowRef) {
    let detached = w_owner.borrow_layout_root_mut().take();
    drop(detached);
}
/// Fit the layout to an `sx` by `sy` window: arrange the sticky preset, or
/// adjust the tree proportionally when there is none or it declines. Returns
/// whether the tree or any pane moved.
pub unsafe fn layout_resize(w_owner: &WindowRef, mut sx: u_int, mut sy: u_int) -> bool {
    if super::set::layout_set_arrange_sticky(w_owner, sx, sy) {
        return layout_fix_panes(w_owner, None);
    }
    let (pane_status, horizontal_minimum) = layout_resize_limits(w_owner);
    let adjusted;
    {
        let mut tree = w_owner.borrow_layout_root_mut();
        let lc = tree.as_deref_mut().expect("layout root") as *mut layout_cell;
        let mut xlimit: i32;
        let mut ylimit: i32;
        let mut xchange: i32;
        let mut ychange: i32;
        xchange = sx.wrapping_sub((*lc).g.sx) as ::core::ffi::c_int;
        xlimit = layout_resize_check_with_limits(
            lc,
            pane_status,
            horizontal_minimum,
            lc,
            LAYOUT_LEFTRIGHT,
        ) as ::core::ffi::c_int;
        if xchange < 0 as ::core::ffi::c_int && xchange < -xlimit {
            xchange = -xlimit;
        }
        if xlimit == 0 as ::core::ffi::c_int {
            if sx <= (*lc).g.sx {
                xchange = 0 as ::core::ffi::c_int;
            } else {
                xchange = sx.wrapping_sub((*lc).g.sx) as ::core::ffi::c_int;
            }
        }
        if xchange != 0 as ::core::ffi::c_int {
            layout_resize_adjust_with_limits(
                lc,
                pane_status,
                horizontal_minimum,
                lc,
                LAYOUT_LEFTRIGHT,
                xchange,
            );
        }
        ychange = sy.wrapping_sub((*lc).g.sy) as ::core::ffi::c_int;
        ylimit = layout_resize_check_with_limits(
            lc,
            pane_status,
            horizontal_minimum,
            lc,
            LAYOUT_TOPBOTTOM,
        ) as ::core::ffi::c_int;
        if ychange < 0 as ::core::ffi::c_int && ychange < -ylimit {
            ychange = -ylimit;
        }
        if ylimit == 0 as ::core::ffi::c_int {
            if sy <= (*lc).g.sy {
                ychange = 0 as ::core::ffi::c_int;
            } else {
                ychange = sy.wrapping_sub((*lc).g.sy) as ::core::ffi::c_int;
            }
        }
        if ychange != 0 as ::core::ffi::c_int {
            layout_resize_adjust_with_limits(
                lc,
                pane_status,
                horizontal_minimum,
                lc,
                LAYOUT_TOPBOTTOM,
                ychange,
            );
        }
        adjusted = xchange != 0 || ychange != 0;
    }
    layout_fix_offsets(w_owner);
    layout_fix_panes(w_owner, None) || adjusted
}
pub unsafe fn layout_resize_pane_to(
    pane: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    direction: layout_type,
    new_size: u_int,
) {
    let window = pane
        .window_observer()
        .upgrade()
        .expect("resized pane window");
    let id = pane.layout_identity(false).expect("resized pane layout");
    let change = (|| {
        let mut guard = window
            .borrow_layout_cell_mut(id)
            .expect("resized pane belongs to layout");
        let mut cell = &mut *guard as *mut layout_cell;
        let mut parent = (*cell).parent;
        while !parent.is_null() && (*parent).type_0 != direction {
            cell = parent;
            parent = (*cell).parent;
        }
        if parent.is_null() {
            return None;
        }
        let size = if direction == LAYOUT_LEFTRIGHT {
            (*cell).g.sx
        } else {
            (*cell).g.sy
        };
        Some(if cell == layout_cells_last(&*parent) {
            size.wrapping_sub(new_size) as i32
        } else {
            new_size.wrapping_sub(size) as i32
        })
    })();
    window.release(c"resize pane to size");
    if let Some(change) = change {
        layout_resize_pane(pane, direction, change);
    }
}

#[cfg(test)]
mod layout_cell_collection_tests {
    use super::*;

    #[test]
    fn parent_owned_children_preserve_order_and_neighbor_navigation() {
        unsafe {
            let mut root_owner = layout_create_cell();
            let root: *mut layout_cell = &mut *root_owner;
            layout_make_node(root, LAYOUT_LEFTRIGHT);

            let mut first_owner = layout_create_cell();
            let first: *mut layout_cell = &mut *first_owner;
            let mut second_owner = layout_create_cell();
            let second: *mut layout_cell = &mut *second_owner;
            let mut third_owner = layout_create_cell();
            let third: *mut layout_cell = &mut *third_owner;
            let mut fourth_owner = layout_create_cell();
            let fourth: *mut layout_cell = &mut *fourth_owner;
            let mut fifth_owner = layout_create_cell();
            let fifth: *mut layout_cell = &mut *fifth_owner;
            layout_cells_push_back(root, first_owner);
            layout_cells_push_back(root, second_owner);
            layout_cells_push_back(root, third_owner);
            layout_cells_push_back(root, fourth_owner);
            layout_cells_push_back(root, fifth_owner);

            assert_eq!(layout_cells_first(&*root), first);
            assert_eq!(layout_cells_last(&*root), fifth);
            assert_eq!(layout_cell_next(first), second);
            assert_eq!(layout_cell_prev(fifth), fourth);
            assert_eq!((*first).sibling_index, 0);
            assert_eq!((*second).sibling_index, 1);
            assert_eq!((*third).sibling_index, 2);
            assert_eq!(layout_cell_get_neighbour(first), second);
            assert_eq!(layout_cell_get_neighbour(fifth), fourth);

            let mut inserted_owner = layout_create_cell();
            let inserted: *mut layout_cell = &mut *inserted_owner;
            layout_cells_insert_before(root, second, inserted_owner);
            assert_eq!(layout_cell_next(first), inserted);
            assert_eq!(layout_cell_prev(second), inserted);
            assert_eq!((*inserted).sibling_index, 1);
            assert_eq!((*second).sibling_index, 2);
            assert_eq!((*third).sibling_index, 3);
            assert_eq!((*fifth).sibling_index, 5);
            assert_eq!(layout_cell_get_neighbour(inserted), second);

            let inserted_owner = layout_cells_remove(root, inserted).unwrap();
            assert!((*inserted).parent.is_null());
            drop(inserted_owner);
            assert_eq!(layout_cell_next(first), second);
            assert_eq!((*second).sibling_index, 1);
            assert_eq!((*fifth).sibling_index, 4);
            assert_eq!(layout_cell_get_neighbour(fifth), fourth);

            drop(root_owner);
        }
    }
}

/// Resize a previously observed border. A callback may have removed it since
/// observation; a stale identity is ignored, never resolved through an old address.
pub unsafe fn layout_resize_layout(
    window: &WindowRef,
    id: *mut layout_cell,
    direction: layout_type,
    change: i32,
    opposite: i32,
) -> bool {
    let exists = {
        let tree = window.borrow_layout_root(crate::src::window::LayoutView::Visible);
        tree.is_some_and(|root| root.find(id).is_some())
    };
    if !exists {
        return false;
    }
    let (pane_status, horizontal_minimum) = if change != 0 {
        layout_resize_limits(window)
    } else {
        (0, 0)
    };
    {
        let mut tree = window.borrow_layout_root_mut();
        let Some(root) = tree.as_deref_mut() else {
            return false;
        };
        let root = root as *mut layout_cell;
        let Some(cell) = (*root).find_mut(id) else {
            return false;
        };
        let cell = cell as *mut layout_cell;
        let mut needed = change;
        while needed != 0 {
            let size = if change > 0 {
                let size = layout_resize_pane_grow(
                    root,
                    pane_status,
                    horizontal_minimum,
                    cell,
                    direction,
                    needed,
                    opposite,
                );
                needed -= size;
                size
            } else {
                let size = layout_resize_pane_shrink(
                    root,
                    pane_status,
                    horizontal_minimum,
                    cell,
                    direction,
                    needed,
                );
                needed += size;
                size
            };
            if size == 0 {
                break;
            }
        }
    }
    layout_fix_offsets(window);
    layout_fix_panes(window, None);
    events_fire_window(c"window-layout-changed".as_ptr(), window.clone());
    true
}
pub unsafe fn layout_resize_pane(
    pane: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    direction: layout_type,
    change: i32,
) {
    let window = pane
        .window_observer()
        .upgrade()
        .expect("resized pane window");
    let target = (|| {
        let mut tree = window.borrow_layout_root_mut();
        let mut cell =
            tree.as_deref_mut()?
                .find_pane_mut(&std::rc::Rc::downgrade(pane))? as *mut layout_cell;
        let mut parent = (*cell).parent;
        while !parent.is_null() && (*parent).type_0 != direction {
            cell = parent;
            parent = (*cell).parent;
        }
        if parent.is_null() {
            return None;
        }
        if cell == layout_cells_last(&*parent) {
            cell = layout_cell_prev(cell);
            if cell.is_null() {
                return None;
            }
        }
        Some((*cell).id())
    })();
    if let Some(id) = target {
        layout_resize_layout(&window, id, direction, change, 1);
    }
    window.release(c"resize pane layout");
}
unsafe fn layout_resize_pane_grow(
    root: *mut layout_cell,
    pane_status: i32,
    horizontal_minimum: u32,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut needed: ::core::ffi::c_int,
    mut opposite: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lcadd: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcremove: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut size: u_int = 0 as u_int;
    lcadd = lc;
    lcremove = layout_cell_next(lc);
    while !lcremove.is_null() {
        size = layout_resize_check_with_limits(
            root,
            pane_status,
            horizontal_minimum,
            lcremove,
            type_0,
        );
        if size > 0 as u_int {
            break;
        }
        lcremove = layout_cell_next(lcremove);
    }
    if opposite != 0 && lcremove.is_null() {
        lcremove = layout_cell_prev(lc);
        while !lcremove.is_null() {
            size = layout_resize_check_with_limits(
                root,
                pane_status,
                horizontal_minimum,
                lcremove,
                type_0,
            );
            if size > 0 as u_int {
                break;
            }
            lcremove = layout_cell_prev(lcremove);
        }
    }
    if lcremove.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if size > needed as u_int {
        size = needed as u_int;
    }
    layout_resize_adjust_with_limits(
        root,
        pane_status,
        horizontal_minimum,
        lcadd,
        type_0,
        size as ::core::ffi::c_int,
    );
    layout_resize_adjust_with_limits(
        root,
        pane_status,
        horizontal_minimum,
        lcremove,
        type_0,
        size.wrapping_neg() as ::core::ffi::c_int,
    );
    size as ::core::ffi::c_int
}
unsafe fn layout_resize_pane_shrink(
    root: *mut layout_cell,
    pane_status: i32,
    horizontal_minimum: u32,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut needed: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lcadd: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcremove: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut size: u_int = 0;
    lcremove = lc;
    loop {
        size = layout_resize_check_with_limits(
            root,
            pane_status,
            horizontal_minimum,
            lcremove,
            type_0,
        );
        if size != 0 as u_int {
            break;
        }
        lcremove = layout_cell_prev(lcremove);
        if lcremove.is_null() {
            break;
        }
    }
    if lcremove.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    lcadd = layout_cell_next(lc);
    if lcadd.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    // The magnitude of i32::MIN fits in the unsigned layout size.
    if size > needed.unsigned_abs() {
        size = needed.unsigned_abs();
    }
    layout_resize_adjust_with_limits(
        root,
        pane_status,
        horizontal_minimum,
        lcadd,
        type_0,
        size as ::core::ffi::c_int,
    );
    layout_resize_adjust_with_limits(
        root,
        pane_status,
        horizontal_minimum,
        lcremove,
        type_0,
        size.wrapping_neg() as ::core::ffi::c_int,
    );
    size as ::core::ffi::c_int
}
pub unsafe fn layout_assign_pane(
    window: &WindowRef,
    id: *mut layout_cell,
    pane: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    do_not_resize: i32,
) {
    {
        let mut cell = window
            .borrow_layout_cell_mut(id)
            .expect("reserved layout cell still belongs to window");
        layout_make_leaf(&mut *cell, pane);
    }
    let (sx, sy) = window.size();
    super::set::layout_set_arrange_sticky(window, sx, sy);
    layout_fix_panes(window, if do_not_resize != 0 { Some(pane) } else { None });
}
unsafe fn layout_new_pane_size(
    root: *mut layout_cell,
    pane_status: i32,
    horizontal_minimum: u32,
    mut previous: u_int,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut size: u_int,
    mut count_left: u_int,
    mut size_left: u_int,
) -> u_int {
    let mut new_size: u_int = 0;
    let mut min: u_int = 0;
    let mut max: u_int = 0;
    let mut available: u_int = 0;
    if count_left == 1 as u_int {
        return size_left;
    }
    available = layout_resize_check_with_limits(root, pane_status, horizontal_minimum, lc, type_0);
    min = ((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
        .wrapping_mul(count_left.wrapping_sub(1 as u_int));
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*lc).g.sx.wrapping_sub(available) > min {
            min = (*lc).g.sx.wrapping_sub(available);
        }
        new_size = (*lc).g.sx.wrapping_mul(size).wrapping_div(previous);
    } else {
        if (*lc).g.sy.wrapping_sub(available) > min {
            min = (*lc).g.sy.wrapping_sub(available);
        }
        new_size = (*lc).g.sy.wrapping_mul(size).wrapping_div(previous);
    }
    max = size_left.wrapping_sub(min);
    if new_size > max {
        new_size = max;
    }
    if new_size < PANE_MINIMUM as u_int {
        new_size = PANE_MINIMUM as u_int;
    }
    new_size
}
unsafe fn layout_set_size_check(
    root: *mut layout_cell,
    pane_status: i32,
    horizontal_minimum: u32,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut size: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut new_size: u_int = 0;
    let mut available: u_int = 0;
    let mut previous: u_int = 0;
    let mut count: u_int = 0;
    let mut idx: u_int = 0;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return (size >= PANE_MINIMUM) as ::core::ffi::c_int;
    }
    available = size as u_int;
    count = 0 as u_int;
    lcchild = layout_cells_first(&*lc);
    while !lcchild.is_null() {
        count = count.wrapping_add(1);
        lcchild = layout_cell_next(lcchild);
    }
    if (*lc).type_0 as ::core::ffi::c_uint == type_0 as ::core::ffi::c_uint {
        if available < count.wrapping_mul(2 as u_int).wrapping_sub(1 as u_int) {
            return 0 as ::core::ffi::c_int;
        }
        if type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            previous = (*lc).g.sx;
        } else {
            previous = (*lc).g.sy;
        }
        idx = 0 as u_int;
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            new_size = layout_new_pane_size(
                root,
                pane_status,
                horizontal_minimum,
                previous,
                lcchild,
                type_0,
                size as u_int,
                count.wrapping_sub(idx),
                available,
            );
            if idx == count.wrapping_sub(1 as u_int) {
                if new_size > available {
                    return 0 as ::core::ffi::c_int;
                }
                available = available.wrapping_sub(new_size);
            } else {
                if new_size.wrapping_add(1 as u_int) > available {
                    return 0 as ::core::ffi::c_int;
                }
                available = available.wrapping_sub(new_size.wrapping_add(1 as u_int));
            }
            if layout_set_size_check(
                root,
                pane_status,
                horizontal_minimum,
                lcchild,
                type_0,
                new_size as ::core::ffi::c_int,
            ) == 0
            {
                return 0 as ::core::ffi::c_int;
            }
            idx = idx.wrapping_add(1);
            lcchild = layout_cell_next(lcchild);
        }
    } else {
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            if !((*lcchild).type_0 as ::core::ffi::c_uint
                == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint)
                && layout_set_size_check(
                    root,
                    pane_status,
                    horizontal_minimum,
                    lcchild,
                    type_0,
                    size,
                ) == 0
            {
                return 0 as ::core::ffi::c_int;
            }
            lcchild = layout_cell_next(lcchild);
        }
    }
    1 as ::core::ffi::c_int
}
unsafe fn layout_resize_child_cells(
    root: *mut layout_cell,
    pane_status: i32,
    horizontal_minimum: u32,
    mut lc: *mut layout_cell,
) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut prev: u_int = 0;
    let mut available: u_int = 0;
    let mut count: u_int = 0;
    let mut idx: u_int = 0;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    count = 0 as u_int;
    prev = 0 as u_int;
    lcchild = layout_cells_first(&*lc);
    while !lcchild.is_null() {
        count = count.wrapping_add(1);
        if (*lc).type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            prev = prev.wrapping_add((*lcchild).g.sx);
        } else if (*lc).type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            prev = prev.wrapping_add((*lcchild).g.sy);
        }
        lcchild = layout_cell_next(lcchild);
    }
    prev = prev.wrapping_add(count.wrapping_sub(1 as u_int));
    available = 0 as u_int;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        available = (*lc).g.sx;
    } else if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        available = (*lc).g.sy;
    }
    idx = 0 as u_int;
    lcchild = layout_cells_first(&*lc);
    while !lcchild.is_null() {
        if (*lc).type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*lcchild).g.sx = (*lc).g.sx;
            (*lcchild).g.xoff = (*lc).g.xoff;
        } else {
            (*lcchild).g.sx = layout_new_pane_size(
                root,
                pane_status,
                horizontal_minimum,
                prev,
                lcchild,
                (*lc).type_0,
                (*lc).g.sx,
                count.wrapping_sub(idx),
                available,
            );
            available = available.wrapping_sub((*lcchild).g.sx.wrapping_add(1 as u_int));
        }
        if (*lc).type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*lcchild).g.sy = (*lc).g.sy;
            (*lcchild).g.yoff = (*lc).g.yoff;
        } else {
            (*lcchild).g.sy = layout_new_pane_size(
                root,
                pane_status,
                horizontal_minimum,
                prev,
                lcchild,
                (*lc).type_0,
                (*lc).g.sy,
                count.wrapping_sub(idx),
                available,
            );
            available = available.wrapping_sub((*lcchild).g.sy.wrapping_add(1 as u_int));
        }
        layout_resize_child_cells(root, pane_status, horizontal_minimum, lcchild);
        idx = idx.wrapping_add(1);
        lcchild = layout_cell_next(lcchild);
    }
}
unsafe fn layout_replace_with_node(
    tree: &mut Option<Box<layout_cell>>,
    lc: *mut layout_cell,
    type_0: layout_type,
) -> *mut layout_cell {
    let mut node = layout_create_cell();
    let parent: *mut layout_cell = &mut *node;
    layout_make_node(parent, type_0);
    layout_set_size(parent, (*lc).g.sx, (*lc).g.sy, (*lc).g.xoff, (*lc).g.yoff);
    let detached = if (*lc).parent.is_null() {
        tree.replace(node).expect("replaced root is owned")
    } else {
        layout_cells_replace((*lc).parent, lc, node)
    };
    layout_cells_push_front(parent, detached);
    parent
}

unsafe fn layout_split_check_space_with_limits(
    root: *mut layout_cell,
    cell: *mut layout_cell,
    direction: layout_type,
    status: i32,
    horizontal_minimum: u32,
) -> i32 {
    match direction {
        LAYOUT_LEFTRIGHT => ((*cell).g.sx >= horizontal_minimum) as i32,
        LAYOUT_TOPBOTTOM => {
            let minimum = (PANE_MINIMUM * 2
                + 1
                + (layout_add_horizontal_border(root, cell, status) != 0) as i32)
                as u32;
            ((*cell).g.sy >= minimum) as i32
        }
        _ => fatalx(|out| out.write_all(b"bad layout type")),
    }
}
pub unsafe fn layout_split_sizes(
    mut lc: *mut layout_cell,
    mut size: ::core::ffi::c_int,
    mut before: ::core::ffi::c_int,
    mut type_0: layout_type,
    mut size1: *mut u_int,
    mut size2: *mut u_int,
    mut saved_size: *mut u_int,
) {
    let mut s1: u_int = 0;
    let mut s2: u_int = 0;
    let mut ss: u_int = 0;
    let mut sx: u_int = (*lc).g.sx;
    let mut sy: u_int = (*lc).g.sy;
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        ss = sx;
    } else {
        ss = sy;
    }
    if size < 0 as ::core::ffi::c_int {
        s2 = ss
            .wrapping_add(1 as u_int)
            .wrapping_div(2 as u_int)
            .wrapping_sub(1 as u_int);
    } else if before != 0 {
        s2 = ss.wrapping_sub(size as u_int).wrapping_sub(1 as u_int);
    } else {
        s2 = size as u_int;
    }
    if s2 < PANE_MINIMUM as u_int {
        s2 = PANE_MINIMUM as u_int;
    } else if s2 > ss.wrapping_sub(2 as u_int) {
        s2 = ss.wrapping_sub(2 as u_int);
    }
    s1 = ss.wrapping_sub(1 as u_int).wrapping_sub(s2);
    *size1 = s1;
    *size2 = s2;
    *saved_size = ss;
}
pub unsafe fn layout_split_pane(
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut type_0: layout_type,
    mut size: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> Option<*mut layout_cell> {
    let window = wp_owner.window_observer().upgrade().expect("split window");
    let status = window.pane_border_status();
    let split_horizontal_minimum =
        wp_owner.split_minimum_width(window.scrollbar_mode() == PANE_SCROLLBARS_ALWAYS);
    let (pane_status, horizontal_minimum) = if flags & SPAWN_FULLSIZE != 0 {
        layout_resize_limits(&window)
    } else {
        (status, 0)
    };
    let result = (|| {
        let mut tree = window.borrow_layout_root_mut();
        let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lcnew: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lc1: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut lc2: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut xoff: u_int = 0;
        let mut yoff: u_int = 0;
        let mut size1: u_int = 0;
        let mut size2: u_int = 0;
        let mut new_size: u_int = 0;
        let mut saved_size: u_int = 0;
        let mut resize_first: u_int = 0 as u_int;
        let mut full_size: ::core::ffi::c_int = flags & SPAWN_FULLSIZE;
        let mut before: ::core::ffi::c_int = flags & SPAWN_BEFORE;
        lc = if full_size != 0 {
            tree.as_deref_mut().expect("split root")
        } else {
            tree.as_deref_mut()
                .expect("split root")
                .find_pane_mut(&std::rc::Rc::downgrade(wp_owner))
                .expect("split pane belongs to tree")
        };
        sx = (*lc).g.sx;
        sy = (*lc).g.sy;
        xoff = (*lc).g.xoff as u_int;
        yoff = (*lc).g.yoff as u_int;
        if layout_split_check_space_with_limits(
            tree.as_deref_mut().unwrap(),
            lc,
            type_0,
            status,
            split_horizontal_minimum,
        ) == 0
        {
            return None;
        }
        layout_split_sizes(
            lc,
            size,
            before,
            type_0,
            &raw mut size1,
            &raw mut size2,
            &raw mut saved_size,
        );
        if flags & SPAWN_BEFORE != 0 {
            new_size = size2;
        } else {
            new_size = size1;
        }
        if full_size != 0
            && layout_set_size_check(
                tree.as_deref_mut().unwrap(),
                pane_status,
                horizontal_minimum,
                lc,
                type_0,
                new_size as ::core::ffi::c_int,
            ) == 0
        {
            return None;
        }
        if !(*lc).parent.is_null()
            && (*(*lc).parent).type_0 as ::core::ffi::c_uint == type_0 as ::core::ffi::c_uint
        {
            lcparent = (*lc).parent;
            let mut new_owner = layout_create_cell();
            lcnew = &mut *new_owner;
            if flags & SPAWN_BEFORE != 0 {
                layout_cells_insert_before(lcparent, lc, new_owner);
            } else {
                layout_cells_insert_after(lcparent, lc, new_owner);
            }
        } else if full_size != 0
            && (*lc).parent.is_null()
            && (*lc).type_0 as ::core::ffi::c_uint == type_0 as ::core::ffi::c_uint
        {
            if (*lc).type_0 as ::core::ffi::c_uint
                == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*lc).g.sx = new_size;
                layout_resize_child_cells(
                    tree.as_deref_mut().unwrap(),
                    pane_status,
                    horizontal_minimum,
                    lc,
                );
                (*lc).g.sx = saved_size;
            } else if (*lc).type_0 as ::core::ffi::c_uint
                == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*lc).g.sy = new_size;
                layout_resize_child_cells(
                    tree.as_deref_mut().unwrap(),
                    pane_status,
                    horizontal_minimum,
                    lc,
                );
                (*lc).g.sy = saved_size;
            }
            resize_first = 1 as u_int;
            let mut new_owner = layout_create_cell();
            lcnew = &mut *new_owner;
            size = saved_size.wrapping_sub(1 as u_int).wrapping_sub(new_size) as ::core::ffi::c_int;
            if (*lc).type_0 as ::core::ffi::c_uint
                == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                layout_set_size(
                    lcnew,
                    size as u_int,
                    sy,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
            } else if (*lc).type_0 as ::core::ffi::c_uint
                == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                layout_set_size(
                    lcnew,
                    sx,
                    size as u_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
            }
            if flags & SPAWN_BEFORE != 0 {
                layout_cells_push_front(lc, new_owner);
            } else {
                layout_cells_push_back(lc, new_owner);
            }
        } else {
            lcparent = layout_replace_with_node(tree, lc, type_0);
            let mut new_owner = layout_create_cell();
            lcnew = &mut *new_owner;
            if flags & SPAWN_BEFORE != 0 {
                layout_cells_push_front(lcparent, new_owner);
            } else {
                layout_cells_push_back(lcparent, new_owner);
            }
        }
        if flags & SPAWN_BEFORE != 0 {
            lc1 = lcnew;
            lc2 = lc;
        } else {
            lc1 = lc;
            lc2 = lcnew;
        }
        if resize_first == 0
            && type_0 as ::core::ffi::c_uint
                == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            layout_set_size(
                lc1,
                size1,
                sy,
                xoff as ::core::ffi::c_int,
                yoff as ::core::ffi::c_int,
            );
            layout_set_size(
                lc2,
                size2,
                sy,
                xoff.wrapping_add((*lc1).g.sx).wrapping_add(1 as u_int) as ::core::ffi::c_int,
                yoff as ::core::ffi::c_int,
            );
        } else if resize_first == 0
            && type_0 as ::core::ffi::c_uint
                == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            layout_set_size(
                lc1,
                sx,
                size1,
                xoff as ::core::ffi::c_int,
                yoff as ::core::ffi::c_int,
            );
            layout_set_size(
                lc2,
                sx,
                size2,
                xoff as ::core::ffi::c_int,
                yoff.wrapping_add((*lc1).g.sy).wrapping_add(1 as u_int) as ::core::ffi::c_int,
            );
        }
        if full_size != 0 {
            if resize_first == 0 {
                layout_resize_child_cells(
                    tree.as_deref_mut().unwrap(),
                    pane_status,
                    horizontal_minimum,
                    lc,
                );
            }
        } else {
            layout_make_leaf(lc, wp_owner);
        }
        Some((*lcnew).id())
    })();
    if result.is_some() && flags & SPAWN_FULLSIZE != 0 {
        layout_fix_offsets(&window);
    }
    window.release(c"split layout");
    result
}
pub unsafe fn layout_close_pane(pane: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>) {
    // A logically destroyed pane may already have lost its parent Window. Keep
    // the legacy early return for a cleared Pane cell link.
    if pane.layout_identity(false).is_none() {
        return;
    }
    let window = pane
        .window_observer()
        .upgrade()
        .expect("closing pane window");
    let observer = std::rc::Rc::downgrade(pane);
    let needs_policy = {
        let mut tree = window.borrow_layout_root_mut();
        let cell = tree
            .as_deref_mut()
            .and_then(|root| root.find_pane_mut(&observer))
            .expect("closing pane belongs to visible layout");
        !cell.parent.is_null()
    };
    let (pane_status, horizontal_minimum) = if needs_policy {
        layout_resize_limits(&window)
    } else {
        (0, 0)
    };
    let has_root = {
        let mut tree = window.borrow_layout_root_mut();
        let cell = tree
            .as_deref_mut()
            .and_then(|root| root.find_pane_mut(&observer))
            .unwrap() as *mut layout_cell;
        let id = (*cell).id();
        layout_destroy_cell_with_limits(pane_status, horizontal_minimum, cell, tree);
        pane.detach_layout(id);
        tree.is_some()
    };
    if has_root {
        let (sx, sy) = window.size();
        super::set::layout_set_arrange_sticky(&window, sx, sy);
        layout_fix_offsets(&window);
        layout_fix_panes(&window, None);
    }
    // Resize callbacks can reparent the pane. Preserve the original notification
    // target lookup, which occurs after resizing rather than at function entry.
    let notify_window = pane
        .window_observer()
        .upgrade()
        .expect("pane notification window");
    events_fire_window(c"window-layout-changed".as_ptr(), notify_window.clone());
    notify_window.release(c"close pane layout notification");
    window.release(c"close pane layout");
}
pub(super) unsafe fn layout_spread_cell_with_limits(
    root: *mut layout_cell,
    pane_status: i32,
    horizontal_minimum: u32,
    mut parent: *mut layout_cell,
) -> ::core::ffi::c_int {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut number: u_int = 0;
    let mut each: u_int = 0;
    let mut size: u_int = 0;
    let mut this: u_int = 0;
    let mut remainder: u_int = 0;
    let mut change: ::core::ffi::c_int = 0;
    let mut changed: ::core::ffi::c_int = 0;
    let mut status: ::core::ffi::c_int = 0;
    number = (*parent).cells.len() as u_int;
    if number <= 1 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    status = pane_status;
    if (*parent).type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        size = (*parent).g.sx;
    } else if (*parent).type_0 as ::core::ffi::c_uint
        == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if layout_add_horizontal_border(root, parent, status) != 0 {
            size = (*parent).g.sy.wrapping_sub(1 as u_int);
        } else {
            size = (*parent).g.sy;
        }
    } else {
        return 0 as ::core::ffi::c_int;
    }
    if size < number.wrapping_sub(1 as u_int) {
        return 0 as ::core::ffi::c_int;
    }
    each = size
        .wrapping_sub(number.wrapping_sub(1 as u_int))
        .wrapping_div(number);
    if each == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    remainder = size
        .wrapping_sub(number.wrapping_mul(each.wrapping_add(1 as u_int)))
        .wrapping_add(1 as u_int);
    changed = 0 as ::core::ffi::c_int;
    lc = layout_cells_first(&*parent);
    while !lc.is_null() {
        change = 0 as ::core::ffi::c_int;
        if (*parent).type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            change =
                each.wrapping_sub((*lc).g.sx as ::core::ffi::c_int as u_int) as ::core::ffi::c_int;
            if remainder > 0 as u_int {
                change += 1;
                remainder = remainder.wrapping_sub(1);
            }
            layout_resize_adjust_with_limits(
                root,
                pane_status,
                horizontal_minimum,
                lc,
                LAYOUT_LEFTRIGHT,
                change,
            );
        } else if (*parent).type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if layout_add_horizontal_border(root, lc, status) != 0 {
                this = each.wrapping_add(1 as u_int);
            } else {
                this = each;
            }
            if remainder > 0 as u_int {
                this = this.wrapping_add(1);
                remainder = remainder.wrapping_sub(1);
            }
            change =
                this.wrapping_sub((*lc).g.sy as ::core::ffi::c_int as u_int) as ::core::ffi::c_int;
            layout_resize_adjust_with_limits(
                root,
                pane_status,
                horizontal_minimum,
                lc,
                LAYOUT_TOPBOTTOM,
                change,
            );
        }
        if change != 0 as ::core::ffi::c_int {
            changed = 1 as ::core::ffi::c_int;
        }
        lc = layout_cell_next(lc);
    }
    changed
}
pub unsafe fn layout_spread_out(wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>) {
    let window = wp_owner
        .window_observer()
        .upgrade()
        .expect("spread pane window");
    let pane = std::rc::Rc::downgrade(wp_owner);
    let can_spread = {
        let mut tree = window.borrow_layout_root_mut();
        !tree
            .as_deref_mut()
            .expect("spread root")
            .find_pane_mut(&pane)
            .expect("spread pane cell")
            .parent
            .is_null()
    };
    if can_spread {
        let (pane_status, horizontal_minimum) = layout_resize_limits(&window);
        let changed = {
            let mut tree = window.borrow_layout_root_mut();
            let root = tree.as_deref_mut().expect("spread root") as *mut layout_cell;
            let mut parent = (*root)
                .find_pane_mut(&pane)
                .expect("spread pane cell")
                .parent;
            loop {
                if parent.is_null() {
                    break false;
                }
                if layout_spread_cell_with_limits(root, pane_status, horizontal_minimum, parent)
                    != 0
                {
                    break true;
                }
                parent = (*parent).parent;
            }
        };
        // Resizing panes can reenter the Window and replace its layout.
        if changed {
            layout_fix_offsets(&window);
            layout_fix_panes(&window, None);
        }
    }
    window.release(c"spread layout");
}
pub unsafe fn layout_get_tiled_cell(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    mut args: *mut args,
    w_owner: &WindowRef,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut flags: ::core::ffi::c_int,
) -> Result<*mut layout_cell, std::ffi::CString> {
    let mut type_0: layout_type = LAYOUT_TOPBOTTOM;
    let mut curval: u_int = 0;
    let mut size: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    if flags & SPAWN_HORIZONTAL != 0 {
        type_0 = LAYOUT_LEFTRIGHT;
    }
    if args_has(args, 'l' as i32 as u_char) != 0 || args_has(args, 'p' as i32 as u_char) != 0 {
        if flags & SPAWN_FULLSIZE != 0 {
            if type_0 as ::core::ffi::c_uint
                == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                curval = (w_owner).size().1;
            } else {
                curval = (w_owner).size().0;
            }
        } else if type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            curval = wp_owner.geometry().1;
        } else {
            curval = wp_owner.geometry().0;
        }
    }
    if args_has(args, 'l' as i32 as u_char) != 0 {
        size = match args_percentage_and_expand_result(
            args,
            'l' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            INT_MAX as ::core::ffi::c_longlong,
            curval as ::core::ffi::c_longlong,
            Some(item_handle),
        ) {
            Ok(value) => value as ::core::ffi::c_int,
            Err(error) => {
                let mut message = b"invalid tiled geometry ".to_vec();
                message.extend_from_slice(error.message().to_bytes());
                return Err(std::ffi::CString::new(message).expect("diagnostic contains no NUL"));
            }
        };
    } else if args_has(args, 'p' as i32 as u_char) != 0 {
        size = match args_strtonum_and_expand_result(
            args,
            'p' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            100 as ::core::ffi::c_longlong,
            Some(item_handle),
        ) {
            Ok(value) => curval
                .wrapping_mul(value as u_int)
                .wrapping_div(100 as u_int) as ::core::ffi::c_int,
            Err(error) => {
                let mut message = b"invalid tiled geometry ".to_vec();
                message.extend_from_slice(error.message().to_bytes());
                return Err(std::ffi::CString::new(message).expect("diagnostic contains no NUL"));
            }
        };
    }
    w_owner.push_zoom(true, (flags & SPAWN_ZOOM) != 0);
    layout_split_pane(wp_owner, type_0, size, flags)
        .ok_or_else(|| c"no space for a new pane".to_owned())
}
#[cfg(test)]
mod geometry_policy_tests {
    use super::*;

    #[test]
    fn tiled_removal_reclaims_space_and_keeps_surviving_cell_identity() {
        unsafe {
            let mut node = layout_create_cell();
            layout_make_node(&mut *node, LAYOUT_LEFTRIGHT);
            layout_set_size(&mut *node, 17, 5, 0, 0);
            for x in [0, 9] {
                let mut cell = layout_create_cell();
                layout_set_size(&mut *cell, 8, 5, x, 0);
                layout_cells_push_back(&mut *node, cell);
            }
            let removed = layout_cells_first(&node);
            let survivor = layout_cell_next(removed);
            let survivor_id = (*survivor).id();
            let mut tree = Some(node);
            layout_destroy_cell_with_limits(0, 1, removed, &mut tree);
            let root = tree.as_deref().unwrap();
            assert_eq!(root.id(), survivor_id);
            assert!(std::ptr::eq(root, survivor));
            assert!(root.parent.is_null());
            assert_eq!(
                (root.g.sx, root.g.sy, root.g.xoff, root.g.yoff),
                (17, 5, 0, 0)
            );
        }
    }

    #[test]
    fn border_resize_transfers_space_between_tiles() {
        unsafe {
            let mut root = layout_create_cell();
            layout_make_node(&mut *root, LAYOUT_LEFTRIGHT);
            layout_set_size(&mut *root, 17, 5, 0, 0);
            for x in [0, 9] {
                let mut cell = layout_create_cell();
                layout_set_size(&mut *cell, 8, 5, x, 0);
                layout_cells_push_back(&mut *root, cell);
            }
            let (pane_status, horizontal_minimum) = (0, 1);
            let root_ptr = &mut *root as *mut layout_cell;
            let left = layout_cells_first(&root);
            assert_eq!(
                layout_resize_pane_grow(
                    root_ptr,
                    pane_status,
                    horizontal_minimum,
                    left,
                    LAYOUT_LEFTRIGHT,
                    3,
                    0
                ),
                3
            );
            assert_eq!((root.cells[0].g.sx, root.cells[1].g.sx), (11, 5));
            assert_eq!(
                layout_resize_pane_shrink(
                    root_ptr,
                    pane_status,
                    horizontal_minimum,
                    left,
                    LAYOUT_LEFTRIGHT,
                    -2
                ),
                2
            );
            assert_eq!((root.cells[0].g.sx, root.cells[1].g.sx), (9, 7));
        }
    }

    #[test]
    fn split_space_respects_pane_scrollbar_width_and_outer_status_edges() {
        unsafe {
            let mut root = layout_create_cell();
            layout_make_node(&mut *root, LAYOUT_TOPBOTTOM);
            layout_set_size(&mut *root, 10, 7, 0, 0);
            for y in [0, 4] {
                let mut cell = layout_create_cell();
                layout_set_size(&mut *cell, 10, 3, 0, y);
                layout_cells_push_back(&mut *root, cell);
            }
            let root_ptr = &mut *root as *mut layout_cell;
            let top = layout_cells_first(&root);
            let bottom = layout_cell_next(top);
            // At this height, the status row blocks only the outer edge pane.
            assert_eq!(
                layout_split_check_space_with_limits(
                    root_ptr,
                    top,
                    LAYOUT_TOPBOTTOM,
                    PANE_STATUS_TOP,
                    0
                ),
                0
            );
            assert_eq!(
                layout_split_check_space_with_limits(
                    root_ptr,
                    bottom,
                    LAYOUT_TOPBOTTOM,
                    PANE_STATUS_TOP,
                    0
                ),
                1
            );
            assert_eq!(
                layout_split_check_space_with_limits(
                    root_ptr,
                    top,
                    LAYOUT_TOPBOTTOM,
                    PANE_STATUS_BOTTOM,
                    0
                ),
                1
            );
            assert_eq!(
                layout_split_check_space_with_limits(
                    root_ptr,
                    bottom,
                    LAYOUT_TOPBOTTOM,
                    PANE_STATUS_BOTTOM,
                    0
                ),
                0
            );
            assert_eq!(
                layout_split_check_space_with_limits(root_ptr, top, LAYOUT_TOPBOTTOM, 0, 0),
                1
            );
            (*top).g.sy = 4;
            assert_eq!(
                layout_split_check_space_with_limits(
                    root_ptr,
                    top,
                    LAYOUT_TOPBOTTOM,
                    PANE_STATUS_TOP,
                    0
                ),
                1
            );
            // Use the split pane's own scrollbar width, not the active pane's.
            assert_eq!(
                layout_split_check_space_with_limits(root_ptr, top, LAYOUT_LEFTRIGHT, 0, 10),
                1
            );
            assert_eq!(
                layout_split_check_space_with_limits(root_ptr, top, LAYOUT_LEFTRIGHT, 0, 11),
                0
            );
        }
    }

    #[test]
    fn spread_and_shrink_preserve_scrollbar_minimum() {
        unsafe {
            let mut root = layout_create_cell();
            layout_make_node(&mut *root, LAYOUT_LEFTRIGHT);
            layout_set_size(&mut *root, 19, 8, 0, 0);
            for _ in 0..3 {
                let mut cell = layout_create_cell();
                layout_set_size(&mut *cell, 1, 8, 0, 0);
                layout_cells_push_back(&mut *root, cell);
            }
            let (pane_status, horizontal_minimum) = (0, 3);
            let root_ptr = &mut *root as *mut layout_cell;
            assert_eq!(
                layout_spread_cell_with_limits(root_ptr, pane_status, horizontal_minimum, root_ptr),
                1
            );
            assert_eq!(
                root.cells.iter().map(|c| c.g.sx).collect::<Vec<_>>(),
                [6, 6, 5]
            );
            let available = layout_resize_check_with_limits(
                root_ptr,
                pane_status,
                horizontal_minimum,
                root_ptr,
                LAYOUT_LEFTRIGHT,
            );
            assert_eq!(available, 8);
            layout_resize_adjust_with_limits(
                root_ptr,
                pane_status,
                horizontal_minimum,
                root_ptr,
                LAYOUT_LEFTRIGHT,
                -(available as i32),
            );
            assert_eq!(root.g.sx, 11);
            assert_eq!(
                root.cells.iter().map(|c| c.g.sx).collect::<Vec<_>>(),
                [3, 3, 3]
            );
            assert_eq!(
                layout_resize_check_with_limits(
                    root_ptr,
                    pane_status,
                    horizontal_minimum,
                    root_ptr,
                    LAYOUT_LEFTRIGHT
                ),
                0
            );
        }
    }
}

use crate::src::arguments::{
    args_has, args_percentage_and_expand_result, args_strtonum_and_expand_result,
};
use crate::src::events::events_fire_window;
use crate::src::ffi::libc::memcpy;
use crate::src::log::{fatalx, log_cstr, log_cstr_width, log_debug, log_pointer};
use crate::src::shared::abi::*;
pub use crate::src::shared::arguments::args;
pub use crate::src::shared::command::cmdq_item;
pub use crate::src::shared::environment::environ;
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::key::key_event;
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cells};
pub use crate::src::shared::limits::{INT_MAX, UINT_MAX};
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::window_pane;
pub use crate::src::shared::pane::{
    PANE_MAXIMUM, PANE_MINIMUM, PANE_REDRAWSCROLLBAR, PANE_SCROLLBARS_ALWAYS, PANE_SCROLLBARS_LEFT,
    PANE_STATUS_BOTTOM, PANE_STATUS_TOP,
};
pub use crate::src::shared::spawn::{
    SPAWN_BEFORE, SPAWN_FLOATOVERZOOM, SPAWN_FULLSIZE, SPAWN_HORIZONTAL, SPAWN_SPLIT, SPAWN_ZOOM,
};
use crate::src::shared::style::*;
pub use crate::src::shared::tty::tty_term;
pub use crate::src::shared::window::window;
use crate::src::shared::window::WindowRef;
use crate::src::window::Window as _;
use crate::src::window_pane::WindowPane as _;
use std::ffi::{CStr, CString};

unsafe fn layout_geometry_init(mut lg: *mut layout_geometry) {
    (*lg).sx = UINT_MAX as u_int;
    (*lg).sy = UINT_MAX as u_int;
    (*lg).xoff = INT_MAX;
    (*lg).yoff = INT_MAX;
}
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
pub unsafe fn layout_cell_is_tiled(mut lc: *mut layout_cell) -> ::core::ffi::c_int {
    let mut is_leaf: ::core::ffi::c_int = ((*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
    let mut is_floating: ::core::ffi::c_int = (*lc).flags & LAYOUT_CELL_FLOATING;
    (is_leaf != 0 && is_floating == 0) as ::core::ffi::c_int
}
pub unsafe fn layout_cell_has_tiled_child(mut lc: *mut layout_cell) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    lcchild = layout_cells_first(&*lc);
    while !lcchild.is_null() {
        if layout_cell_is_tiled(lcchild) != 0 || layout_cell_has_tiled_child(lcchild) != 0 {
            return 1 as ::core::ffi::c_int;
        }
        lcchild = layout_cell_next(lcchild);
    }
    0 as ::core::ffi::c_int
}
unsafe fn layout_cell_is_first_tiled(mut lc: *mut layout_cell) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = (*lc).parent;
    if lcparent.is_null() {
        return layout_cell_is_tiled(lc);
    }
    lcchild = layout_cells_first(&*lcparent);
    while !lcchild.is_null() {
        if layout_cell_is_tiled(lcchild) != 0 || layout_cell_has_tiled_child(lcchild) != 0 {
            break;
        }
        lcchild = layout_cell_next(lcchild);
    }
    (lcchild == lc) as ::core::ffi::c_int
}
unsafe fn layout_cell_get_first_tiled(mut lc: *mut layout_cell) -> *mut layout_cell {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild2: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if layout_cell_is_tiled(lc) != 0 {
        return lc;
    }
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    lcchild = layout_cells_first(&*lc);
    while !lcchild.is_null() {
        if layout_cell_is_tiled(lcchild) != 0 {
            return lcchild;
        }
        if (*lcchild).type_0 as ::core::ffi::c_uint
            != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            lcchild2 = layout_cell_get_first_tiled(lcchild);
            if !lcchild2.is_null() {
                return lcchild2;
            }
        }
        lcchild = layout_cell_next(lcchild);
    }
    ::core::ptr::null_mut::<layout_cell>()
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
            if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
                (*lcchild).g.xoff = xoff;
                (*lcchild).g.yoff = (*lc).g.yoff;
                if (*lcchild).type_0 as ::core::ffi::c_uint
                    != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    layout_fix_offsets1(lcchild);
                }
                xoff = (xoff as u_int).wrapping_add((*lcchild).g.sx.wrapping_add(1 as u_int))
                    as ::core::ffi::c_int as ::core::ffi::c_int;
            }
            lcchild = layout_cell_next(lcchild);
        }
    } else {
        yoff = (*lc).g.yoff;
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
                (*lcchild).g.xoff = (*lc).g.xoff;
                (*lcchild).g.yoff = yoff;
                if (*lcchild).type_0 as ::core::ffi::c_uint
                    != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    layout_fix_offsets1(lcchild);
                }
                yoff = (yoff as u_int).wrapping_add((*lcchild).g.sy.wrapping_add(1 as u_int))
                    as ::core::ffi::c_int as ::core::ffi::c_int;
            }
            lcchild = layout_cell_next(lcchild);
        }
    };
}
pub unsafe fn layout_fix_offsets(w_owner: &WindowRef) {
    let mut tree = w_owner.borrow_layout_root_mut();
    let lc = tree.as_deref_mut().expect("layout root") as *mut layout_cell;
    if (*lc).flags & LAYOUT_CELL_FLOATING != 0 {
        return;
    }
    (*lc).g.xoff = 0 as ::core::ffi::c_int;
    (*lc).g.yoff = 0 as ::core::ffi::c_int;
    layout_fix_offsets1(lc);
}
unsafe fn layout_cell_is_last_tiled(mut lc: *mut layout_cell) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = (*lc).parent;
    if lcparent.is_null() {
        return layout_cell_is_tiled(lc);
    }
    lcchild = layout_cells_last(&*lcparent);
    while !lcchild.is_null() {
        if layout_cell_is_tiled(lcchild) != 0 || layout_cell_has_tiled_child(lcchild) != 0 {
            break;
        }
        lcchild = layout_cell_prev(lcchild);
    }
    (lcchild == lc) as ::core::ffi::c_int
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
            && layout_cell_is_first_tiled(lc) == 0
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
            && layout_cell_is_last_tiled(lc) == 0
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
pub unsafe fn layout_fix_panes(
    window: &WindowRef,
    skip: Option<&std::rc::Rc<std::cell::UnsafeCell<window_pane>>>,
) {
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
}
pub unsafe fn layout_count_cells(
    mut lc: *mut layout_cell,
    mut with_floating: ::core::ffi::c_int,
) -> u_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut count: u_int = 0 as u_int;
    match (*lc).type_0 as ::core::ffi::c_uint {
        2 => {
            if (*lc).flags & LAYOUT_CELL_FLOATING != 0 && with_floating == 0 {
                return 0 as u_int;
            }
            1 as u_int
        }
        0 | 1 => {
            lcchild = layout_cells_first(&*lc);
            while !lcchild.is_null() {
                count = count.wrapping_add(layout_count_cells(lcchild, with_floating));
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
    if layout_cell_is_tiled(lc) == 0 && layout_cell_has_tiled_child(lc) == 0 {
        return 0 as u_int;
    }
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
            if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
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
            if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
                layout_resize_adjust_with_limits(
                    root,
                    pane_status,
                    horizontal_minimum,
                    lcchild,
                    type_0,
                    change,
                );
            }
            lcchild = layout_cell_next(lcchild);
        }
        return;
    }
    if layout_cell_has_tiled_child(lc) == 0 {
        return;
    }
    while change != 0 as ::core::ffi::c_int {
        changed = 0 as ::core::ffi::c_int;
        lcchild = layout_cells_first(&*lc);
        while !lcchild.is_null() {
            if change == 0 as ::core::ffi::c_int {
                break;
            }
            if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
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
            }
            lcchild = layout_cell_next(lcchild);
        }
        if changed == 0 {
            break;
        }
    }
}
unsafe fn layout_resize_set_size_with_limits(
    root: *mut layout_cell,
    pane_status: i32,
    horizontal_minimum: u32,
    cell: *mut layout_cell,
    direction: layout_type,
    size: u32,
) {
    let previous = if direction == LAYOUT_LEFTRIGHT {
        (*cell).g.sx
    } else {
        (*cell).g.sy
    };
    layout_resize_adjust_with_limits(
        root,
        pane_status,
        horizontal_minimum,
        cell,
        direction,
        size.wrapping_sub(previous) as i32,
    );
}
unsafe fn layout_cell_get_neighbour_dir(
    mut lc: *mut layout_cell,
    mut direction: ::core::ffi::c_int,
) -> *mut layout_cell {
    let mut lcn: *mut layout_cell = lc;
    loop {
        if direction != 0 {
            lcn = layout_cell_next(lcn);
        } else {
            lcn = layout_cell_prev(lcn);
        }
        if lcn.is_null() || layout_cell_is_tiled(lcn) != 0 || layout_cell_has_tiled_child(lcn) != 0
        {
            return lcn;
        }
    }
}
pub unsafe fn layout_cell_get_neighbour(mut lc: *mut layout_cell) -> *mut layout_cell {
    let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = (*lc).parent;
    let mut direction: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if lcparent.is_null() {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    if lc == layout_cells_last(&*lcparent) {
        direction = (direction == 0) as ::core::ffi::c_int;
    }
    lcother = layout_cell_get_neighbour_dir(lc, direction);
    if lcother.is_null() {
        lcother = layout_cell_get_neighbour_dir(lc, (direction == 0) as ::core::ffi::c_int);
    }
    lcother
}
/// Removal from a detached parser-owned tree. Live Window removal uses the
/// pure helper with policy copied before its root is borrowed.
pub unsafe fn layout_destroy_cell(
    window: Option<&WindowRef>,
    cell: *mut layout_cell,
    root: &mut Option<Box<layout_cell>>,
) {
    let (pane_status, horizontal_minimum) =
        if !(*cell).parent.is_null() && layout_cell_is_tiled(cell) != 0 {
            layout_resize_limits(window.expect("tiled cell removal requires a live window"))
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
    if layout_cell_is_tiled(lc) != 0 {
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
        } else {
            layout_remove_tile_with_limits(
                root.as_deref_mut().unwrap(),
                pane_status,
                horizontal_minimum,
                parent,
            );
        }
    }
    drop(layout_cells_remove(parent, lc).expect("removed cell is owned"));
    if (*parent).cells.len() == 1 {
        let child = layout_cells_first(&*parent);
        let mut child_owner = layout_cells_remove(parent, child).expect("remaining child is owned");
        let grandparent = (*parent).parent;
        if grandparent.is_null() {
            if layout_cell_is_tiled(child) != 0 {
                child_owner.g.xoff = 0;
                child_owner.g.yoff = 0;
            }
            *root = Some(child_owner);
        } else {
            drop(layout_cells_replace(grandparent, parent, child_owner));
        }
    }
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
unsafe fn layout_clamp_floating_panes(window: &WindowRef, sx: u_int, sy: u_int) {
    let mut cursor = window.step_pane(crate::src::window::PaneOrder::Stacking, None, false);
    while let Some(pane) = cursor {
        let id = pane.layout_identity(false);
        if let Some(id) = id {
            let floating = {
                window
                    .borrow_layout_cell(id)
                    .is_some_and(|cell| cell.flags & LAYOUT_CELL_FLOATING != 0)
            };
            if floating {
                let pad = (pane.pane_lines() != PANE_LINES_NONE) as u32;
                let mut cell = window
                    .borrow_layout_cell_mut(id)
                    .expect("floating pane belongs to layout");
                layout_clamp_floating_cell(cell, sx, sy, pad);
            }
        }
        cursor = window.step_pane(
            crate::src::window::PaneOrder::Stacking,
            Some(&std::rc::Rc::downgrade(&pane)),
            false,
        );
    }
}

fn layout_clamp_floating_cell(cell: &mut layout_cell, sx: u32, sy: u32, pad: u32) {
    let x_available = sx.saturating_sub(2 * pad);
    if cell.g.sx > x_available {
        cell.g.sx = x_available.max(PANE_MINIMUM as u32);
    }
    let y_available = sy.saturating_sub(2 * pad);
    if cell.g.sy > y_available {
        cell.g.sy = y_available.max(PANE_MINIMUM as u32);
    }
    if (cell.g.xoff as u32)
        .wrapping_add(cell.g.sx)
        .wrapping_add(pad)
        > sx
    {
        cell.g.xoff = if cell.g.sx.wrapping_add(2 * pad) >= sx {
            pad as i32
        } else {
            sx.wrapping_sub(cell.g.sx).wrapping_sub(pad) as i32
        };
    }
    if (cell.g.yoff as u32)
        .wrapping_add(cell.g.sy)
        .wrapping_add(pad)
        > sy
    {
        cell.g.yoff = if cell.g.sy.wrapping_add(2 * pad) >= sy {
            pad as i32
        } else {
            sy.wrapping_sub(cell.g.sy).wrapping_sub(pad) as i32
        };
    }
}

#[cfg(test)]
mod floating_clamp_tests {
    use super::*;

    #[test]
    fn clamping_keeps_minimum_size_and_accounts_for_optional_borders() {
        for (width, height, pad, expected) in [
            (20, 12, 0, (20, 12, 0, 0)),
            (20, 12, 1, (18, 10, 1, 1)),
            (1, 1, 1, (1, 1, 1, 1)),
        ] {
            let mut cell = layout_create_cell();
            cell.g = layout_geometry {
                sx: 30,
                sy: 22,
                xoff: 25,
                yoff: 17,
            };
            layout_clamp_floating_cell(&mut cell, width, height, pad);
            assert_eq!((cell.g.sx, cell.g.sy, cell.g.xoff, cell.g.yoff), expected);
        }
        let mut cell = layout_create_cell();
        cell.g = layout_geometry {
            sx: 5,
            sy: 3,
            xoff: 18,
            yoff: 10,
        };
        layout_clamp_floating_cell(&mut cell, 20, 12, 1);
        assert_eq!(
            (cell.g.sx, cell.g.sy, cell.g.xoff, cell.g.yoff),
            (5, 3, 14, 8)
        );
    }
}
pub unsafe fn layout_resize(w_owner: &WindowRef, mut sx: u_int, mut sy: u_int) {
    let floating_root = {
        let mut tree = w_owner.borrow_layout_root_mut();
        let root = tree.as_deref_mut().expect("layout root");
        root.type_0 == LAYOUT_WINDOWPANE && root.flags & LAYOUT_CELL_FLOATING != 0
    };
    if floating_root {
        layout_clamp_floating_panes(w_owner, sx, sy);
        layout_fix_panes(w_owner, None);
        return;
    }
    let (pane_status, horizontal_minimum) = layout_resize_limits(w_owner);
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
    }
    layout_fix_offsets(w_owner);
    layout_clamp_floating_panes(w_owner, sx, sy);
    layout_fix_panes(w_owner, None);
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
        Some(if layout_cell_is_last_tiled(cell) != 0 {
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

pub unsafe fn layout_resize_floating_pane_to(
    pane: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    direction: layout_type,
    mut size: u_int,
) -> Result<(), std::ffi::CString> {
    let window = pane
        .window_observer()
        .upgrade()
        .expect("resized pane window");
    let result = (|| {
        let id = pane.layout_identity(false).expect("floating pane layout");
        let floating = {
            window
                .borrow_layout_cell(id)
                .expect("floating pane belongs to layout")
                .flags
                & LAYOUT_CELL_FLOATING
                != 0
        };
        if !floating {
            return Err(c"pane is not floating".to_owned());
        }
        if pane.pane_lines() != PANE_LINES_NONE && size >= (PANE_MINIMUM + 2) as u32 {
            size -= 2;
        }
        if size < PANE_MINIMUM as u32 || size > PANE_MAXIMUM as u32 {
            return Err(c"size is too big or too small".to_owned());
        }
        let changed = {
            let mut guard = window
                .borrow_layout_cell_mut(id)
                .expect("floating pane belongs to layout");
            let cell = &mut *guard;
            let current = if direction == LAYOUT_TOPBOTTOM {
                &mut cell.g.sy
            } else {
                &mut cell.g.sx
            };
            let changed = *current != size;
            *current = size;
            changed
        };
        if changed {
            window.invalidate_scene();
        }
        Ok(())
    })();
    window.release(c"resize floating pane to size");
    result
}
pub unsafe fn layout_resize_floating_pane(
    pane: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    direction: layout_type,
    change: ::core::ffi::c_int,
    opposite: ::core::ffi::c_int,
) -> Result<(), std::ffi::CString> {
    let window = pane
        .window_observer()
        .upgrade()
        .expect("resized pane window");
    let result = (|| {
        let id = pane.layout_identity(false).expect("floating pane layout");
        {
            let mut guard = window
                .borrow_layout_cell_mut(id)
                .expect("floating pane belongs to layout");
            let cell = &mut *guard;
            if cell.flags & LAYOUT_CELL_FLOATING == 0 {
                return Err(c"pane is not floating".to_owned());
            }
            if change == 0 {
                return Ok(());
            }
            let (current, offset) = if direction == LAYOUT_TOPBOTTOM {
                (&mut cell.g.sy, &mut cell.g.yoff)
            } else {
                (&mut cell.g.sx, &mut cell.g.xoff)
            };
            let size = current.wrapping_add(change as u32);
            if size < PANE_MINIMUM as u32 || size > PANE_MAXIMUM as u32 {
                return Err(c"change is too big or too small".to_owned());
            }
            *current = size;
            if opposite != 0 {
                *offset -= change;
            }
        }
        window.invalidate_scene();
        Ok(())
    })();
    window.release(c"resize floating pane");
    result
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
        if layout_cell_is_last_tiled(cell) != 0 {
            cell = layout_cell_get_neighbour_dir(cell, 0);
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
    lcremove = layout_cell_get_neighbour_dir(lc, 1 as ::core::ffi::c_int);
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
        lcremove = layout_cell_get_neighbour_dir(lcremove, 1 as ::core::ffi::c_int);
    }
    if opposite != 0 && lcremove.is_null() {
        lcremove = layout_cell_get_neighbour_dir(lc, 0 as ::core::ffi::c_int);
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
            lcremove = layout_cell_get_neighbour_dir(lcremove, 0 as ::core::ffi::c_int);
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
        lcremove = layout_cell_get_neighbour_dir(lcremove, 0 as ::core::ffi::c_int);
        if lcremove.is_null() {
            break;
        }
    }
    if lcremove.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    lcadd = layout_cell_get_neighbour_dir(lc, 1 as ::core::ffi::c_int);
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
        if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
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
        if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
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
        }
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
        if (*lc).flags & LAYOUT_CELL_FLOATING != 0 {
            fatalx(|out| out.write_all(b"floating cells cannot be split"));
        }
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
pub unsafe fn layout_floating_pane(
    window: &WindowRef,
    pane: Option<&std::rc::Rc<std::cell::UnsafeCell<window_pane>>>,
    geometry: *mut layout_geometry,
) -> *mut layout_cell {
    let mut tree = window.borrow_layout_root_mut();
    let root = tree.as_deref_mut().expect("floating layout root");
    let cell = match pane {
        Some(pane) => root
            .find_pane_mut(&std::rc::Rc::downgrade(pane))
            .expect("floating anchor belongs to tree"),
        None => root,
    } as *mut layout_cell;
    let mut parent = (*cell).parent;
    if parent.is_null() {
        parent = layout_replace_with_node(tree, cell, LAYOUT_TOPBOTTOM);
    }
    let mut new_cell = layout_create_cell();
    new_cell.flags |= LAYOUT_CELL_FLOATING;
    new_cell.g = *geometry;
    let id = new_cell.id();
    layout_cells_insert_after(parent, cell, new_cell);
    id
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
        !cell.parent.is_null() && layout_cell_is_tiled(cell) != 0
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
    number = 0 as u_int;
    lc = layout_cells_first(&*parent);
    while !lc.is_null() {
        if layout_cell_is_tiled(lc) != 0 {
            number = number.wrapping_add(1);
        }
        lc = layout_cell_next(lc);
    }
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
        if !(layout_cell_is_tiled(lc) == 0) {
            change = 0 as ::core::ffi::c_int;
            if (*parent).type_0 as ::core::ffi::c_uint
                == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                change = each.wrapping_sub((*lc).g.sx as ::core::ffi::c_int as u_int)
                    as ::core::ffi::c_int;
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
                change = this.wrapping_sub((*lc).g.sy as ::core::ffi::c_int as u_int)
                    as ::core::ffi::c_int;
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
        let cell = tree
            .as_deref_mut()
            .expect("spread root")
            .find_pane_mut(&pane)
            .expect("spread pane cell");
        let mut parent = cell.parent;
        loop {
            if parent.is_null() {
                break false;
            }
            if (*parent)
                .cells
                .iter()
                .filter(|cell| {
                    layout_cell_is_tiled(&***cell as *const layout_cell as *mut layout_cell) != 0
                })
                .count()
                > 1
            {
                break true;
            }
            parent = (*parent).parent;
        }
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
    if wp_owner.is_floating() {
        return Err(c"can't split a floating pane".to_owned());
    }
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
    if std::rc::Rc::clone(w_owner).active_pane_over_zoom() != 0 {
        std::rc::Rc::clone(w_owner).push_zoom(false, true);
    } else {
        std::rc::Rc::clone(w_owner).push_zoom(true, (flags & SPAWN_ZOOM) != 0);
    }
    layout_split_pane(wp_owner, type_0, size, flags)
        .ok_or_else(|| c"no space for a new pane".to_owned())
}
pub unsafe fn layout_get_floating_cell(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    mut args: *mut args,
    mut lines: pane_lines,
    w_owner: &WindowRef,
    wp_owner: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    mut flags: ::core::ffi::c_int,
) -> Result<*mut layout_cell, CString> {
    let mut fg: layout_geometry = layout_geometry {
        sx: 0,
        sy: 0,
        xoff: 0,
        yoff: 0,
    };
    layout_geometry_init(&raw mut fg);
    if flags & SPAWN_SPLIT != 0 {
        let size = w_owner.size();
        let mut tree = w_owner.borrow_layout_root_mut();
        let cell = tree
            .as_deref_mut()
            .expect("floating split root")
            .find_pane_mut(&std::rc::Rc::downgrade(wp_owner))
            .expect("floating split pane belongs to tree");
        layout_split_floating_cell(cell, size, &raw mut fg, lines, flags)?;
    } else {
        layout_floating_args_parse(item_handle, args, lines, w_owner, &mut fg)?;
    }
    let (unzoom, remember) =
        if flags & SPAWN_FLOATOVERZOOM != 0 || w_owner.active_pane_over_zoom() != 0 {
            (0, 1)
        } else {
            (1, flags & SPAWN_ZOOM)
        };
    let parent = wp_owner
        .window_observer()
        .upgrade()
        .expect("floating pane window");
    parent.push_zoom((unzoom) != 0, (remember) != 0);
    parent.release(c"prepare floating layout");
    Ok(layout_floating_pane(w_owner, Some(wp_owner), &raw mut fg))
}
fn layout_position_error(message: &CStr) -> CString {
    let mut bytes = b"position ".to_vec();
    bytes.extend_from_slice(message.to_bytes());
    CString::new(bytes).expect("diagnostic contains no NUL")
}

pub unsafe fn layout_floating_args_parse(
    item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>,
    mut args: *mut args,
    mut lines: pane_lines,
    w_owner: &WindowRef,
    lg: &mut layout_geometry,
) -> Result<(), CString> {
    let mut sx: ::core::ffi::c_int = 0;
    let mut sy: ::core::ffi::c_int = 0;
    let mut ox: ::core::ffi::c_int = 0;
    let mut oy: ::core::ffi::c_int = 0;
    sx = (if lg.sx == UINT_MAX {
        (w_owner).size().0.wrapping_div(2 as u_int)
    } else {
        lg.sx
    }) as ::core::ffi::c_int;
    sy = (if lg.sy == UINT_MAX {
        (w_owner).size().1.wrapping_div(4 as u_int)
    } else {
        lg.sy
    }) as ::core::ffi::c_int;
    ox = lg.xoff;
    oy = lg.yoff;
    if args_has(args, 'x' as i32 as u_char) != 0 {
        sx = match args_percentage_and_expand_result(
            args,
            'x' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            PANE_MAXIMUM as ::core::ffi::c_longlong,
            (w_owner).size().0 as ::core::ffi::c_longlong,
            Some(item_handle),
        ) {
            Ok(value) => value as ::core::ffi::c_int,
            Err(error) => {
                return Err(layout_position_error(error.message()));
            }
        };
        if lines as ::core::ffi::c_uint
            != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            sx -= 2 as ::core::ffi::c_int;
        }
    }
    if args_has(args, 'y' as i32 as u_char) != 0 {
        sy = match args_percentage_and_expand_result(
            args,
            'y' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            PANE_MAXIMUM as ::core::ffi::c_longlong,
            (w_owner).size().1 as ::core::ffi::c_longlong,
            Some(item_handle),
        ) {
            Ok(value) => value as ::core::ffi::c_int,
            Err(error) => {
                return Err(layout_position_error(error.message()));
            }
        };
        if lines as ::core::ffi::c_uint
            != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            sy -= 2 as ::core::ffi::c_int;
        }
    }
    if args_has(args, 'X' as i32 as u_char) != 0 {
        ox = match args_percentage_and_expand_result(
            args,
            'X' as i32 as u_char,
            -sx as ::core::ffi::c_longlong,
            (w_owner).size().0 as ::core::ffi::c_longlong,
            (w_owner).size().0 as ::core::ffi::c_longlong,
            Some(item_handle),
        ) {
            Ok(value) => value as ::core::ffi::c_int,
            Err(error) => {
                return Err(layout_position_error(error.message()));
            }
        };
    }
    if args_has(args, 'Y' as i32 as u_char) != 0 {
        oy = match args_percentage_and_expand_result(
            args,
            'Y' as i32 as u_char,
            -sy as ::core::ffi::c_longlong,
            (w_owner).size().1 as ::core::ffi::c_longlong,
            (w_owner).size().1 as ::core::ffi::c_longlong,
            Some(item_handle),
        ) {
            Ok(value) => value as ::core::ffi::c_int,
            Err(error) => {
                return Err(layout_position_error(error.message()));
            }
        };
    }
    let default_x = ox == INT_MAX;
    let default_y = oy == INT_MAX;
    (ox, oy) =
        w_owner.resolve_floating_position((!default_x).then_some(ox), (!default_y).then_some(oy));
    if !default_x && args_has(args, b'X') != 0 && lines != PANE_LINES_NONE {
        ox += 1;
    }
    if !default_y && args_has(args, b'Y') != 0 && lines != PANE_LINES_NONE {
        oy += 1;
    }
    if !(PANE_MINIMUM..=PANE_MAXIMUM).contains(&sx) {
        return Err(c"invalid width".to_owned());
    }
    if !(PANE_MINIMUM..=PANE_MAXIMUM).contains(&sy) {
        return Err(c"invalid height".to_owned());
    }
    lg.sx = sx as u_int;
    lg.sy = sy as u_int;
    lg.xoff = ox;
    lg.yoff = oy;
    Ok(())
}
unsafe fn layout_split_floating_cell(
    mut lc: *mut layout_cell,
    window_size: (u32, u32),
    mut out: *mut layout_geometry,
    mut lines: pane_lines,
    mut flags: ::core::ffi::c_int,
) -> Result<(), CString> {
    let mut old: layout_geometry = layout_geometry {
        sx: 0,
        sy: 0,
        xoff: 0,
        yoff: 0,
    };
    let mut new: layout_geometry = layout_geometry {
        sx: 0,
        sy: 0,
        xoff: 0,
        yoff: 0,
    };
    let mut tborder: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut bborder: ::core::ffi::c_int =
        window_size.1.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    let mut lborder: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
    let mut rborder: ::core::ffi::c_int =
        window_size.0.wrapping_sub(3 as u_int) as ::core::ffi::c_int;
    let mut border: ::core::ffi::c_int = if lines as ::core::ffi::c_uint
        != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    let mut size: ::core::ffi::c_int = 0;
    let mut space: ::core::ffi::c_int = 0;
    memcpy(
        &raw mut old as *mut ::core::ffi::c_void,
        &raw mut (*lc).g as *const ::core::ffi::c_void,
        ::core::mem::size_of::<layout_geometry>() as size_t,
    );
    if lborder > old.xoff - border {
        old.xoff = lborder + border;
    }
    if rborder < old.xoff + old.sx as ::core::ffi::c_int + border {
        old.xoff = rborder - old.sx as ::core::ffi::c_int - border;
    }
    if tborder > old.yoff - border {
        old.yoff = tborder + border;
    }
    if bborder < old.yoff + old.sy as ::core::ffi::c_int + border {
        old.yoff = bborder - old.sy as ::core::ffi::c_int - border;
    }
    memcpy(
        &raw mut new as *mut ::core::ffi::c_void,
        &raw mut old as *const ::core::ffi::c_void,
        ::core::mem::size_of::<layout_geometry>() as size_t,
    );
    if flags & SPAWN_HORIZONTAL != 0 {
        if flags & SPAWN_BEFORE != 0 {
            new.xoff = (new.xoff as u_int).wrapping_sub(
                old.sx
                    .wrapping_add((2 as ::core::ffi::c_int * border) as u_int),
            ) as ::core::ffi::c_int as ::core::ffi::c_int;
        } else {
            new.xoff = (new.xoff as u_int).wrapping_add(
                old.sx
                    .wrapping_add((2 as ::core::ffi::c_int * border) as u_int),
            ) as ::core::ffi::c_int as ::core::ffi::c_int;
        }
    } else if flags & SPAWN_BEFORE != 0 {
        new.yoff = (new.yoff as u_int).wrapping_sub(
            old.sy
                .wrapping_add((2 as ::core::ffi::c_int * border) as u_int),
        ) as ::core::ffi::c_int as ::core::ffi::c_int;
    } else {
        new.yoff = (new.yoff as u_int).wrapping_add(
            old.sy
                .wrapping_add((2 as ::core::ffi::c_int * border) as u_int),
        ) as ::core::ffi::c_int as ::core::ffi::c_int;
    }
    if lborder > new.xoff - border {
        space = (old.xoff as u_int)
            .wrapping_add(old.sx)
            .wrapping_sub(lborder as u_int)
            .wrapping_sub((3 as ::core::ffi::c_int * border) as u_int)
            .wrapping_add(1 as u_int) as ::core::ffi::c_int;
        size = space / 2 as ::core::ffi::c_int;
        new.sx = size as u_int;
        old.sx = size as u_int;
        new.xoff = lborder + border;
        old.xoff = (new.xoff as u_int)
            .wrapping_add(new.sx)
            .wrapping_add((2 as ::core::ffi::c_int * border) as u_int)
            as ::core::ffi::c_int;
        if space % 2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            old.sx = old.sx.wrapping_sub(1 as u_int);
        }
    } else if rborder < new.xoff + new.sx as ::core::ffi::c_int + border {
        space = rborder - old.xoff - 3 as ::core::ffi::c_int * border + 1 as ::core::ffi::c_int;
        size = space / 2 as ::core::ffi::c_int;
        new.sx = size as u_int;
        old.sx = size as u_int;
        new.xoff = (old.xoff as u_int)
            .wrapping_add(old.sx)
            .wrapping_add((2 as ::core::ffi::c_int * border) as u_int)
            as ::core::ffi::c_int;
        if space % 2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            new.sx = new.sx.wrapping_sub(1 as u_int);
        }
    } else if tborder > new.yoff - border {
        space = old
            .sy
            .wrapping_add(old.yoff as u_int)
            .wrapping_sub(tborder as u_int)
            .wrapping_sub((3 as ::core::ffi::c_int * border) as u_int)
            .wrapping_add(1 as u_int) as ::core::ffi::c_int;
        size = space / 2 as ::core::ffi::c_int;
        new.sy = size as u_int;
        old.sy = size as u_int;
        new.yoff = tborder + border;
        old.yoff = (new.yoff as u_int)
            .wrapping_add(new.sy)
            .wrapping_add((2 as ::core::ffi::c_int * border) as u_int)
            as ::core::ffi::c_int;
        if space % 2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            old.sy = old.sy.wrapping_sub(1 as u_int);
        }
    } else if bborder < new.yoff + new.sy as ::core::ffi::c_int + border {
        space = bborder - old.yoff - 3 as ::core::ffi::c_int * border + 1 as ::core::ffi::c_int;
        size = space / 2 as ::core::ffi::c_int;
        new.sy = size as u_int;
        old.sy = size as u_int;
        new.yoff = (old.yoff as u_int)
            .wrapping_add(old.sy)
            .wrapping_add((2 as ::core::ffi::c_int * border) as u_int)
            as ::core::ffi::c_int;
        if space % 2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            new.sy = new.sy.wrapping_sub(1 as u_int);
        }
    }
    if flags & SPAWN_FULLSIZE != 0 {
        if flags & SPAWN_HORIZONTAL != 0 {
            new.yoff = tborder + border;
            new.sy = (bborder - tborder - 2 as ::core::ffi::c_int * border) as u_int;
            if flags & SPAWN_BEFORE != 0 {
                new.xoff = lborder + border;
                new.sx = (old.xoff - new.xoff - 2 as ::core::ffi::c_int * border) as u_int;
            } else {
                new.sx = (rborder - new.xoff - border) as u_int;
            }
        } else {
            new.xoff = lborder + border;
            new.sx = (rborder - lborder - 2 as ::core::ffi::c_int * border) as u_int;
            if flags & SPAWN_BEFORE != 0 {
                new.yoff = tborder + border;
                new.sy = (old.yoff - new.yoff - 2 as ::core::ffi::c_int * border) as u_int;
            } else {
                new.sy = (bborder - new.yoff - border) as u_int;
            }
        }
    }
    if new.sx < PANE_MINIMUM as u_int
        || new.sy < PANE_MINIMUM as u_int
        || old.sx < PANE_MINIMUM as u_int
        || old.sy < PANE_MINIMUM as u_int
    {
        return Err(c"no space for a new pane".to_owned());
    }
    layout_set_size(lc, old.sx, old.sy, old.xoff, old.yoff);
    memcpy(
        out as *mut ::core::ffi::c_void,
        &raw mut new as *const ::core::ffi::c_void,
        ::core::mem::size_of::<layout_geometry>() as size_t,
    );
    Ok(())
}
/// Convert the pane's current tile after argument expansion has finished.
/// Neither the caller nor pane callbacks retain a pointer into the tree.
pub unsafe fn layout_float_pane(
    window: &WindowRef,
    pane: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
    geometry: layout_geometry,
) {
    let (pane_status, horizontal_minimum) = layout_resize_limits(window);
    let mut tree = window.borrow_layout_root_mut();
    let root = tree.as_deref_mut().expect("tile root") as *mut layout_cell;
    let cell = (*root)
        .find_pane_mut(&std::rc::Rc::downgrade(pane))
        .expect("floating pane cell") as *mut layout_cell;
    (*cell).fg = geometry;
    layout_remove_tile_with_limits(root, pane_status, horizontal_minimum, cell);
    layout_set_size(cell, geometry.sx, geometry.sy, geometry.xoff, geometry.yoff);
    (*cell).flags |= LAYOUT_CELL_FLOATING;
}
unsafe fn layout_remove_tile_with_limits(
    root: *mut layout_cell,
    pane_status: i32,
    horizontal_minimum: u32,
    lc: *mut layout_cell,
) -> i32 {
    let mut lcneighbour: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut type_0: layout_type = LAYOUT_LEFTRIGHT;
    let mut change: ::core::ffi::c_int = 0;
    if (*lc).flags & LAYOUT_CELL_FLOATING != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    lcneighbour = layout_cell_get_neighbour(lc);
    if lcneighbour.is_null() {
        if !(*lc).parent.is_null() {
            layout_remove_tile_with_limits(root, pane_status, horizontal_minimum, (*lc).parent);
        }
    } else {
        lcparent = (*lcneighbour).parent;
        if !lcparent.is_null() {
            type_0 = (*lcparent).type_0;
            if type_0 as ::core::ffi::c_uint
                == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                change = (*lc).g.sy.wrapping_add(1 as u_int) as ::core::ffi::c_int;
            } else {
                change = (*lc).g.sx.wrapping_add(1 as u_int) as ::core::ffi::c_int;
            }
            layout_resize_adjust_with_limits(
                root,
                pane_status,
                horizontal_minimum,
                lcneighbour,
                type_0,
                change,
            );
        }
    }
    if !(*lc).parent.is_null() {
        layout_set_size(
            lc,
            0 as u_int,
            0 as u_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    0 as ::core::ffi::c_int
}
/// Convert a floating pane using a fresh tree borrow. The saved floating
/// geometry is updated even if its tiled neighbour has insufficient space.
pub unsafe fn layout_tile_pane(
    window: &WindowRef,
    pane: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>,
) -> bool {
    let (id, split_pane) = {
        let mut tree = window.borrow_layout_root_mut();
        let cell = tree
            .as_deref_mut()
            .expect("tiling root")
            .find_pane_mut(&std::rc::Rc::downgrade(pane))
            .expect("tiling pane belongs to tree") as *mut layout_cell;
        (*cell).fg = (*cell).g;
        if layout_cell_is_tiled(cell) != 0 {
            return false;
        }
        // An all-floating branch inherits its eventual tiled size from its
        // ancestors. Only the first ancestor with a tiled neighbour splits it.
        let mut candidate = cell;
        let split_pane = loop {
            let parent = (*candidate).parent;
            if parent.is_null() {
                break None;
            }
            let neighbour = layout_cell_get_neighbour(candidate);
            if !neighbour.is_null() {
                let tiled = layout_cell_get_first_tiled(neighbour);
                break Some((
                    (*tiled).wp.clone(),
                    (*parent).type_0,
                    (*neighbour).id(),
                    (*neighbour).flags & LAYOUT_CELL_FLOATING != 0,
                ));
            }
            candidate = parent;
        };
        ((*cell).id(), split_pane)
    };
    let window_size = window.size();
    let (mut pane_status, mut horizontal_minimum) = (0, 0);
    let mut split_pane_status = 0;
    let mut split_horizontal_minimum = 0;
    let mut can_split = false;
    if let Some((observer, direction, neighbour_id, floating_neighbour)) = split_pane {
        if let Some(neighbour) = observer.upgrade() {
            if floating_neighbour {
                fatalx(|out| out.write_all(b"floating cells cannot be split"));
            }
            // A split's horizontal threshold uses the neighbour's scrollbar;
            // recursive resizing uses the active pane's policy as before.
            let parent = neighbour
                .window_observer()
                .upgrade()
                .expect("split pane window");
            split_pane_status = parent.pane_border_status();
            split_horizontal_minimum = if direction == LAYOUT_LEFTRIGHT {
                neighbour.split_minimum_width(parent.scrollbar_mode() == PANE_SCROLLBARS_ALWAYS)
            } else {
                0
            };
            parent.release(c"tile split policy");
            neighbour.release(c"tile split policy");
            can_split = {
                let mut tree = window.borrow_layout_root_mut();
                let root = tree.as_deref_mut().expect("tiling root") as *mut layout_cell;
                let neighbour = (*root)
                    .find_mut(neighbour_id)
                    .expect("tiling neighbour belongs to tree")
                    as *mut layout_cell;
                layout_split_check_space_with_limits(
                    root,
                    neighbour,
                    direction,
                    split_pane_status,
                    split_horizontal_minimum,
                ) != 0
            };
            if can_split {
                (pane_status, horizontal_minimum) = layout_resize_limits(window);
            }
        }
    }
    let mut tree = window.borrow_layout_root_mut();
    let root = tree.as_deref_mut().expect("tiling root") as *mut layout_cell;
    let cell = (*root).find_mut(id).expect("tiling cell belongs to tree") as *mut layout_cell;
    if layout_insert_tile_with_limits(
        root,
        window_size,
        pane_status,
        horizontal_minimum,
        split_pane_status,
        split_horizontal_minimum,
        can_split,
        cell,
    ) != 0
    {
        return false;
    }
    (*cell).flags &= !LAYOUT_CELL_FLOATING;
    true
}

unsafe fn layout_insert_tile_with_limits(
    root: *mut layout_cell,
    window_size: (u32, u32),
    pane_status: i32,
    horizontal_minimum: u32,
    split_pane_status: i32,
    split_horizontal_minimum: u32,
    can_split: bool,
    lc: *mut layout_cell,
) -> i32 {
    let mut lcneighbour: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut type_0: layout_type = LAYOUT_LEFTRIGHT;
    let mut size1: u_int = 0;
    let mut size2: u_int = 0;
    let mut saved_size: u_int = 0;
    if layout_cell_is_tiled(lc) != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    lcparent = (*lc).parent;
    if lcparent.is_null() {
        layout_set_size(
            lc,
            window_size.0,
            window_size.1,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        return 0 as ::core::ffi::c_int;
    }
    type_0 = (*lcparent).type_0;
    lcneighbour = layout_cell_get_neighbour(lc);
    if lcneighbour.is_null() {
        layout_insert_tile_with_limits(
            root,
            window_size,
            pane_status,
            horizontal_minimum,
            split_pane_status,
            split_horizontal_minimum,
            can_split,
            lcparent,
        );
        if type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            size1 = (*lcparent).g.sx;
        } else {
            size1 = (*lcparent).g.sy;
        }
        layout_resize_set_size_with_limits(
            root,
            pane_status,
            horizontal_minimum,
            lc,
            type_0,
            size1,
        );
    } else {
        if !can_split
            || layout_split_check_space_with_limits(
                root,
                lcneighbour,
                type_0,
                split_pane_status,
                split_horizontal_minimum,
            ) == 0
        {
            return -(1 as ::core::ffi::c_int);
        }
        layout_split_sizes(
            lcneighbour,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
            type_0,
            &raw mut size1,
            &raw mut size2,
            &raw mut saved_size,
        );
        layout_resize_set_size_with_limits(
            root,
            pane_status,
            horizontal_minimum,
            lc,
            type_0,
            size1,
        );
        layout_resize_set_size_with_limits(
            root,
            pane_status,
            horizontal_minimum,
            lcneighbour,
            type_0,
            size2,
        );
    }
    if (*lcparent).type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        size1 = (*lcparent).g.sy;
        type_0 = LAYOUT_TOPBOTTOM;
    } else {
        size1 = (*lcparent).g.sx;
        type_0 = LAYOUT_LEFTRIGHT;
    }
    layout_resize_set_size_with_limits(root, pane_status, horizontal_minimum, lc, type_0, size1);
    0 as ::core::ffi::c_int
}

#[cfg(test)]
mod geometry_policy_tests {
    use super::*;

    #[test]
    fn tiling_splits_the_neighbor_and_preserves_cell_identity() {
        unsafe {
            let mut root = layout_create_cell();
            layout_make_node(&mut *root, LAYOUT_LEFTRIGHT);
            layout_set_size(&mut *root, 17, 5, 0, 0);
            let mut floating = layout_create_cell();
            floating.flags = LAYOUT_CELL_FLOATING;
            layout_set_size(&mut *floating, 30, 10, -2, 4);
            let id = floating.id();
            layout_cells_push_back(&mut *root, floating);
            let mut tiled = layout_create_cell();
            layout_set_size(&mut *tiled, 17, 5, 0, 0);
            layout_cells_push_back(&mut *root, tiled);
            let root_ptr = &mut *root as *mut layout_cell;
            let cell = layout_cells_first(&root);
            assert_eq!(
                layout_insert_tile_with_limits(
                    root_ptr,
                    (17, 5),
                    0,
                    PANE_MINIMUM as u32,
                    0,
                    (PANE_MINIMUM * 2 + 1) as u32,
                    true,
                    cell
                ),
                0
            );
            assert_eq!(root.cells[0].id(), id);
            assert_eq!((root.cells[0].g.sx, root.cells[0].g.sy), (8, 5));
            assert_eq!((root.cells[1].g.sx, root.cells[1].g.sy), (8, 5));
            // The outer operation changes the flag after successful insertion.
            assert_ne!(root.cells[0].flags & LAYOUT_CELL_FLOATING, 0);
        }
    }

    #[test]
    fn tiling_an_all_floating_branch_splits_its_ancestors_neighbor() {
        unsafe {
            let mut root = layout_create_cell();
            layout_make_node(&mut *root, LAYOUT_LEFTRIGHT);
            layout_set_size(&mut *root, 17, 5, 0, 0);
            let mut branch = layout_create_cell();
            layout_make_node(&mut *branch, LAYOUT_TOPBOTTOM);
            layout_set_size(&mut *branch, 0, 0, 0, 0);
            let branch_id = branch.id();
            let mut floating = layout_create_cell();
            floating.flags = LAYOUT_CELL_FLOATING;
            layout_set_size(&mut *floating, 30, 10, -2, 4);
            let id = floating.id();
            layout_cells_push_back(&mut *branch, floating);
            let mut other = layout_create_cell();
            other.flags = LAYOUT_CELL_FLOATING;
            layout_set_size(&mut *other, 9, 4, 8, 6);
            let other_id = other.id();
            layout_cells_push_back(&mut *branch, other);
            layout_cells_push_back(&mut *root, branch);
            let mut tiled = layout_create_cell();
            layout_set_size(&mut *tiled, 17, 5, 0, 0);
            layout_cells_push_back(&mut *root, tiled);
            let root_ptr = &mut *root as *mut layout_cell;
            let cell = root.find_mut(id).unwrap() as *mut layout_cell;
            assert_eq!(
                layout_insert_tile_with_limits(
                    root_ptr,
                    (17, 5),
                    0,
                    PANE_MINIMUM as u32,
                    0,
                    (PANE_MINIMUM * 2 + 1) as u32,
                    true,
                    cell
                ),
                0
            );
            let cell = root.find(id).unwrap();
            let branch = root.find(branch_id).unwrap();
            let other = root.find(other_id).unwrap();
            assert_eq!((cell.g.sx, cell.g.sy), (8, 5));
            assert_eq!((branch.g.sx, branch.g.sy), (8, 5));
            assert_eq!((root.cells[1].g.sx, root.cells[1].g.sy), (8, 5));
            assert_eq!(
                (other.g.sx, other.g.sy, other.g.xoff, other.g.yoff),
                (9, 4, 8, 6)
            );
        }
    }

    #[test]
    fn failed_tile_split_keeps_floating_and_neighbor_geometry() {
        unsafe {
            let mut root = layout_create_cell();
            layout_make_node(&mut *root, LAYOUT_LEFTRIGHT);
            layout_set_size(&mut *root, 2, 5, 0, 0);
            let mut floating = layout_create_cell();
            floating.flags = LAYOUT_CELL_FLOATING;
            layout_set_size(&mut *floating, 30, 10, -2, 4);
            layout_cells_push_back(&mut *root, floating);
            let mut tiled = layout_create_cell();
            layout_set_size(&mut *tiled, 2, 5, 0, 0);
            layout_cells_push_back(&mut *root, tiled);
            let root_ptr = &mut *root as *mut layout_cell;
            let cell = layout_cells_first(&root);
            assert_eq!(
                layout_insert_tile_with_limits(
                    root_ptr,
                    (2, 5),
                    0,
                    PANE_MINIMUM as u32,
                    0,
                    (PANE_MINIMUM * 2 + 1) as u32,
                    true,
                    cell
                ),
                -1
            );
            let cell = &root.cells[0];
            assert_eq!(
                (cell.g.sx, cell.g.sy, cell.g.xoff, cell.g.yoff),
                (30, 10, -2, 4)
            );
            assert_ne!(cell.flags & LAYOUT_CELL_FLOATING, 0);
            assert_eq!((root.cells[1].g.sx, root.cells[1].g.sy), (2, 5));
        }
    }

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
    fn removing_last_tile_preserves_remaining_floating_geometry() {
        unsafe {
            let mut node = layout_create_cell();
            layout_make_node(&mut *node, LAYOUT_LEFTRIGHT);
            layout_set_size(&mut *node, 8, 5, 0, 0);
            let mut tiled = layout_create_cell();
            layout_set_size(&mut *tiled, 8, 5, 0, 0);
            let removed = &mut *tiled as *mut layout_cell;
            layout_cells_push_back(&mut *node, tiled);
            let mut floating = layout_create_cell();
            floating.flags = LAYOUT_CELL_FLOATING;
            layout_set_size(&mut *floating, 30, 10, 5, 3);
            let id = floating.id();
            layout_cells_push_back(&mut *node, floating);
            let mut tree = Some(node);
            layout_destroy_cell_with_limits(0, 1, removed, &mut tree);
            let root = tree.as_deref().unwrap();
            assert_eq!(root.id(), id);
            assert_eq!(
                (root.g.sx, root.g.sy, root.g.xoff, root.g.yoff),
                (30, 10, 5, 3)
            );
            assert!(root.parent.is_null());
        }
    }

    #[test]
    fn border_resize_transfers_space_between_tiles_and_preserves_floating_cells() {
        unsafe {
            let mut root = layout_create_cell();
            layout_make_node(&mut *root, LAYOUT_LEFTRIGHT);
            layout_set_size(&mut *root, 17, 5, 0, 0);
            for x in [0, 9] {
                let mut cell = layout_create_cell();
                layout_set_size(&mut *cell, 8, 5, x, 0);
                layout_cells_push_back(&mut *root, cell);
            }
            let mut floating = layout_create_cell();
            floating.flags = LAYOUT_CELL_FLOATING;
            layout_set_size(&mut *floating, 30, 10, 5, 3);
            layout_cells_push_back(&mut *root, floating);
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
            assert_eq!((root.cells[2].g.sx, root.cells[2].g.sy), (30, 10));
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
    fn spread_and_shrink_preserve_floating_geometry_and_scrollbar_minimum() {
        unsafe {
            let mut root = layout_create_cell();
            layout_make_node(&mut *root, LAYOUT_LEFTRIGHT);
            layout_set_size(&mut *root, 19, 8, 0, 0);
            for _ in 0..3 {
                let mut cell = layout_create_cell();
                layout_set_size(&mut *cell, 1, 8, 0, 0);
                layout_cells_push_back(&mut *root, cell);
            }
            let mut floating = layout_create_cell();
            floating.flags = LAYOUT_CELL_FLOATING;
            layout_set_size(&mut *floating, 88, 9, 4, 2);
            layout_cells_push_back(&mut *root, floating);
            let (pane_status, horizontal_minimum) = (0, 3);
            let root_ptr = &mut *root as *mut layout_cell;
            assert_eq!(
                layout_spread_cell_with_limits(root_ptr, pane_status, horizontal_minimum, root_ptr),
                1
            );
            assert_eq!(
                root.cells.iter().map(|c| c.g.sx).collect::<Vec<_>>(),
                [6, 6, 5, 88]
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
                [3, 3, 3, 88]
            );
            assert_eq!(
                (
                    root.cells[3].g.xoff,
                    root.cells[3].g.yoff,
                    root.cells[3].g.sy
                ),
                (4, 2, 9)
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

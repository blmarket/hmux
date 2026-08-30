use crate::arguments::{args_has, args_percentage_and_expand, args_strtonum_and_expand};
use crate::fmt_args;
use crate::list::foreach_owned;
use crate::log::{fatalx, log_debug};
use crate::notify::notify_window;
use crate::options::{options_get_number, options_ptr};
pub use crate::types::*;
use crate::window::window_get_active;
use crate::window::window_pane_of_id;
use crate::window::{
    window_pane_is_floating, window_pane_resize, window_pane_show_scrollbar,
    window_pane_zindex_insert_tail, window_panes_first, window_panes_next, window_push_zoom,
};
use ::core::ffi::{c_char, c_int};
use ::core::ptr::null_mut;
use ::std::ffi::CString;

pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
pub const INT_MAX: c_int = c_int::MAX;
pub const UINT_MAX: ::core::ffi::c_uint = ::core::ffi::c_uint::MAX;
pub const PANE_MINIMUM: c_int = 1 as c_int;
pub const PANE_REDRAWSCROLLBAR: c_int = 0x8000 as c_int;
pub const PANE_STATUS_TOP: c_int = 1 as c_int;
pub const PANE_STATUS_BOTTOM: c_int = 2 as c_int;
pub const PANE_SCROLLBARS_OFF: c_int = 0 as c_int;
pub const PANE_SCROLLBARS_LEFT: c_int = 1 as c_int;
pub const LAYOUT_CELL_FLOATING: c_int = 0x1 as c_int;
pub const SPAWN_BEFORE: c_int = 0x8 as c_int;
pub const SPAWN_FULLSIZE: c_int = 0x20 as c_int;

/// The cells directly under `lc`, in order, walked the way the C's
/// `TAILQ_FOREACH` walked them.
unsafe fn children(lc: *mut layout_cell) -> impl Iterator<Item = *mut layout_cell> {
    unsafe { foreach_owned(&raw mut (*lc).cells) }
}

/// A window's root cell, as the borrowed view every walk over the layout
/// takes, or null for a window that has none.
pub fn layout_root_ptr(root: &Option<Box<layout_cell>>) -> *mut layout_cell {
    root.as_ref()
        .map(|lc| &raw const **lc as *mut layout_cell)
        .unwrap_or(null_mut::<layout_cell>())
}

/// Where `lc` sits in `head`, which is the list it hangs in.
unsafe fn cell_position(head: *mut layout_cells, lc: *mut layout_cell) -> Option<usize> {
    unsafe {
        (*head)
            .iter()
            .position(|cell| std::ptr::eq(&raw const **cell, lc))
    }
}

/// The list `lc` hangs in, which is its parent's children, or null when it is
/// a root.
unsafe fn siblings(lc: *mut layout_cell) -> *mut layout_cells {
    unsafe {
        match (*lc).parent.is_null() {
            true => null_mut::<layout_cells>(),
            false => &raw mut (*(*lc).parent).cells,
        }
    }
}

/// Where `lc` sits among its siblings.
unsafe fn position(lc: *mut layout_cell) -> Option<usize> {
    unsafe {
        let head = siblings(lc);
        (!head.is_null()).then(|| cell_position(head, lc)).flatten()
    }
}

/// The sibling after `lc`, null if it is the last.
unsafe fn next(lc: *mut layout_cell) -> *mut layout_cell {
    unsafe {
        let head = siblings(lc);
        position(lc)
            .and_then(|at| (*head).get(at + 1))
            .map(|cell| &raw const **cell as *mut layout_cell)
            .unwrap_or(null_mut::<layout_cell>())
    }
}

/// The sibling before `lc`, null if it is the first.
unsafe fn prev(lc: *mut layout_cell) -> *mut layout_cell {
    unsafe {
        let head = siblings(lc);
        position(lc)
            .filter(|&at| at > 0)
            .map(|at| &raw const *(*head)[at - 1] as *mut layout_cell)
            .unwrap_or(null_mut::<layout_cell>())
    }
}

/// The first cell under `lc`, null if it has none.
unsafe fn first(lc: *mut layout_cell) -> *mut layout_cell {
    unsafe {
        (*lc)
            .cells
            .first()
            .map(|cell| &raw const **cell as *mut layout_cell)
            .unwrap_or(null_mut::<layout_cell>())
    }
}

/// The last cell under `lc`, null if it has none.
pub(crate) unsafe fn last(lc: *mut layout_cell) -> *mut layout_cell {
    unsafe {
        (*lc)
            .cells
            .last()
            .map(|cell| &raw const **cell as *mut layout_cell)
            .unwrap_or(null_mut::<layout_cell>())
    }
}

/// Empties `lc`'s list of children.
unsafe fn init_children(lc: *mut layout_cell) {
    unsafe { (*lc).cells.clear() }
}

/// Takes `lc` out of the list it hangs in, which is `head`, and hands it to
/// the caller, who hangs it somewhere else or gives it up by dropping it.
unsafe fn unlink(head: *mut layout_cells, lc: *mut layout_cell) -> Option<Box<layout_cell>> {
    unsafe {
        let at = cell_position(head, lc)?;
        Some((*head).remove(at))
    }
}

/// Puts `lcnew` where `lc` sits in `head` and hands `lc` back to the caller.
unsafe fn replace(
    head: *mut layout_cells,
    lc: *mut layout_cell,
    lcnew: Box<layout_cell>,
) -> Box<layout_cell> {
    unsafe {
        let at = cell_position(head, lc).expect("the cell is in this list");

        ::core::mem::replace(&mut (*head)[at], lcnew)
    }
}

/// Puts `lc` at the end of `head`.
pub(crate) unsafe fn insert_tail(head: *mut layout_cells, lc: Box<layout_cell>) {
    unsafe { (*head).push(lc) }
}

/// A new cell under `lcparent`, hung at the end of `head`, as the borrowed
/// pointer the sizing calls take.
pub(crate) unsafe fn insert_new_tail(
    head: *mut layout_cells,
    lcparent: *mut layout_cell,
) -> *mut layout_cell {
    unsafe {
        let lc = layout_create_cell(lcparent);
        let ptr = &raw const *lc as *mut layout_cell;
        insert_tail(head, lc);
        ptr
    }
}

/// Puts `lc` at the front of `head`.
unsafe fn insert_head(head: *mut layout_cells, lc: Box<layout_cell>) {
    unsafe { (*head).insert(0, lc) }
}

/// Puts `lc` in front of `mark`, which is already in `head`.
unsafe fn insert_before(head: *mut layout_cells, mark: *mut layout_cell, lc: Box<layout_cell>) {
    unsafe {
        let at = cell_position(head, mark).expect("the mark is in this list");
        (*head).insert(at, lc);
    }
}

/// Puts `lc` after `mark`, which is already in `head`.
unsafe fn insert_after(head: *mut layout_cells, mark: *mut layout_cell, lc: Box<layout_cell>) {
    unsafe {
        let at = cell_position(head, mark).expect("the mark is in this list");
        (*head).insert(at + 1, lc);
    }
}

/// Whether `lc` is one of the floating cells, which sit over the layout rather
/// than inside it and are left out of every sum.
unsafe fn floating(lc: *mut layout_cell) -> bool {
    unsafe { (*lc).flags & LAYOUT_CELL_FLOATING != 0 }
}

/// A new cell under `lcparent`, as large as a cell can be until somebody sizes
/// it.
pub unsafe fn layout_create_cell(lcparent: *mut layout_cell) -> Box<layout_cell> {
    Box::new(layout_cell {
        type_0: LAYOUT_WINDOWPANE,
        flags: 0,
        parent: lcparent,
        sx: UINT_MAX,
        sy: UINT_MAX,
        xoff: INT_MAX,
        yoff: INT_MAX,
        wp_id: None,
        cells: layout_cells::new(),
    })
}

/// Gives up `lc` and the cells under it. A leaf tells the pane it drew that
/// its cell is gone; dropping the box is what frees the tree.
pub unsafe fn layout_free_cell(w: *mut window, lc: Option<Box<layout_cell>>) {
    unsafe {
        let Some(lc) = lc else {
            return;
        };
        layout_forget_panes(w, &raw const *lc as *mut layout_cell);
    }
}

/// Tells every pane under `lc` that the cell it drew in is going.
unsafe fn layout_forget_panes(w: *mut window, lc: *mut layout_cell) {
    unsafe {
        match (*lc).type_0 {
            LAYOUT_LEFTRIGHT | LAYOUT_TOPBOTTOM => {
                for lcchild in children(lc) {
                    layout_forget_panes(w, lcchild);
                }
            }
            LAYOUT_WINDOWPANE => {
                let wp = layout_cell_pane(w, lc);
                if !wp.is_null() && !(*wp).layout_cell.is_null() {
                    (*(*wp).layout_cell).parent = null_mut::<layout_cell>();
                    (*wp).layout_cell = null_mut::<layout_cell>();
                }
            }
            _ => {}
        }
    }
}

/// Writes the tree under `lc` to the debug log, `n` levels in.
pub unsafe fn layout_print_cell(lc: *mut layout_cell, hdr: *const c_char, n: u_int) {
    unsafe {
        if lc.is_null() {
            return;
        }
        let type_0 = match (*lc).type_0 {
            LAYOUT_LEFTRIGHT => c"LEFTRIGHT",
            LAYOUT_TOPBOTTOM => c"TOPBOTTOM",
            LAYOUT_WINDOWPANE => c"WINDOWPANE",
            _ => c"UNKNOWN",
        };
        log_debug(
            c"%s:%*s%p type %s [parent %p] wp=%%%u [%d,%d %ux%u]".as_ptr(),
            fmt_args![
                hdr,
                n,
                c" ".as_ptr(),
                lc,
                type_0.as_ptr(),
                (*lc).parent,
                (*lc).wp_id.unwrap_or(u_int::MAX),
                (*lc).xoff,
                (*lc).yoff,
                (*lc).sx,
                (*lc).sy
            ],
        );
        if (*lc).type_0 == LAYOUT_LEFTRIGHT || (*lc).type_0 == LAYOUT_TOPBOTTOM {
            for lcchild in children(lc) {
                layout_print_cell(lcchild, hdr, n.wrapping_add(1));
            }
        }
    }
}

/// The cell whose border a click at (x, y) landed on, or null if the point is
/// inside a pane rather than between two of them.
pub unsafe fn layout_search_by_border(
    lc: *mut layout_cell,
    x: u_int,
    y: u_int,
) -> *mut layout_cell {
    unsafe {
        let mut last: *mut layout_cell = null_mut::<layout_cell>();
        for lcchild in children(lc) {
            if x as c_int >= (*lcchild).xoff
                && (x as c_int) < (*lcchild).xoff + (*lcchild).sx as c_int
                && y as c_int >= (*lcchild).yoff
                && (y as c_int) < (*lcchild).yoff + (*lcchild).sy as c_int
            {
                return layout_search_by_border(lcchild, x, y);
            }
            if !last.is_null() {
                match (*lc).type_0 {
                    LAYOUT_LEFTRIGHT => {
                        if (x as c_int) < (*lcchild).xoff
                            && x as c_int >= (*last).xoff + (*last).sx as c_int
                        {
                            return last;
                        }
                    }
                    LAYOUT_TOPBOTTOM
                        if (y as c_int) < (*lcchild).yoff
                            && y as c_int >= (*last).yoff + (*last).sy as c_int =>
                    {
                        return last;
                    }
                    _ => {}
                }
            }
            last = lcchild;
        }
        null_mut::<layout_cell>()
    }
}

pub unsafe fn layout_set_size(
    lc: *mut layout_cell,
    sx: u_int,
    sy: u_int,
    xoff: c_int,
    yoff: c_int,
) {
    unsafe {
        (*lc).sx = sx;
        (*lc).sy = sy;
        (*lc).xoff = xoff;
        (*lc).yoff = yoff;
    }
}

/// The pane of `w` the cell holds, or null for a cell that holds other cells.
pub unsafe fn layout_cell_pane(w: *mut window, lc: *mut layout_cell) -> *mut window_pane {
    unsafe {
        match (*lc).wp_id {
            Some(id) if !w.is_null() => window_pane_of_id(w, id),
            _ => null_mut(),
        }
    }
}

/// Makes `lc` the cell that holds `wp`.
pub unsafe fn layout_cell_set_pane(lc: *mut layout_cell, wp: *mut window_pane) {
    unsafe {
        (*lc).wp_id = wp.as_ref().map(|wp| wp.id);
    }
}

pub unsafe fn layout_make_leaf(lc: *mut layout_cell, wp: *mut window_pane) {
    unsafe {
        (*lc).type_0 = LAYOUT_WINDOWPANE;
        init_children(lc);
        (*wp).layout_cell = lc;
        (*lc).wp_id = Some((*wp).id);
    }
}

pub unsafe fn layout_make_node(w: *mut window, lc: *mut layout_cell, type_0: layout_type) {
    unsafe {
        if type_0 == LAYOUT_WINDOWPANE {
            fatalx(c"bad layout type".as_ptr(), fmt_args![]);
        }
        (*lc).type_0 = type_0;
        init_children(lc);
        let wp = layout_cell_pane(w, lc);
        if !wp.is_null() {
            (*wp).layout_cell = null_mut::<layout_cell>();
        }
        (*lc).wp_id = None;
    }
}

/// Puts the window's panes on its z-index list in the order the tree holds
/// them, left to right and top to bottom.
pub unsafe fn layout_fix_zindexes(w: *mut window, lc: *mut layout_cell) {
    unsafe {
        if lc.is_null() {
            return;
        }
        match (*lc).type_0 {
            LAYOUT_WINDOWPANE => {
                let wp = layout_cell_pane(w, lc);
                if !wp.is_null() {
                    window_pane_zindex_insert_tail(w, wp);
                }
            }
            LAYOUT_LEFTRIGHT | LAYOUT_TOPBOTTOM => {
                for lcchild in children(lc) {
                    layout_fix_zindexes(w, lcchild);
                }
            }
            _ => {
                fatalx(c"bad layout type".as_ptr(), fmt_args![]);
            }
        }
    }
}

/// Gives every cell under `lc` the offset its size and its siblings' put it at.
unsafe fn layout_fix_offsets1(lc: *mut layout_cell) {
    unsafe {
        let leftright = (*lc).type_0 == LAYOUT_LEFTRIGHT;
        let mut off = if leftright { (*lc).xoff } else { (*lc).yoff };
        for lcchild in children(lc) {
            if floating(lcchild) {
                continue;
            }
            if leftright {
                (*lcchild).xoff = off;
                (*lcchild).yoff = (*lc).yoff;
            } else {
                (*lcchild).xoff = (*lc).xoff;
                (*lcchild).yoff = off;
            }
            if (*lcchild).type_0 != LAYOUT_WINDOWPANE {
                layout_fix_offsets1(lcchild);
            }
            let size = if leftright {
                (*lcchild).sx
            } else {
                (*lcchild).sy
            };
            off = (off as u_int).wrapping_add(size.wrapping_add(1)) as c_int;
        }
    }
}

pub unsafe fn layout_fix_offsets(w: *mut window) {
    unsafe {
        let lc = layout_root_ptr(&(*w).layout_root);
        if floating(lc) {
            return;
        }
        (*lc).xoff = 0;
        (*lc).yoff = 0;
        layout_fix_offsets1(lc);
    }
}

/// Whether `lc` is at the very top of the window: the first cell of every
/// top-to-bottom node above it, skipping any floating cells at the edge.
unsafe fn layout_cell_is_top(w: *mut window, mut lc: *mut layout_cell) -> c_int {
    unsafe {
        while lc != layout_root_ptr(&(*w).layout_root) {
            let next = (*lc).parent;
            if next.is_null() {
                return 0;
            }
            if (*next).type_0 == LAYOUT_TOPBOTTOM {
                let edge = children(next).find(|lc| !floating(*lc));
                if lc != edge.unwrap_or(null_mut::<layout_cell>()) {
                    return 0;
                }
            }
            lc = next;
        }
        1
    }
}

/// Whether `lc` is at the very bottom of the window. See [`layout_cell_is_top`].
unsafe fn layout_cell_is_bottom(w: *mut window, mut lc: *mut layout_cell) -> c_int {
    unsafe {
        while lc != layout_root_ptr(&(*w).layout_root) {
            let next = (*lc).parent;
            if next.is_null() {
                return 0;
            }
            if (*next).type_0 == LAYOUT_TOPBOTTOM {
                let mut edge = last(next);
                while !edge.is_null() {
                    if !floating(edge) {
                        break;
                    }
                    edge = prev(edge);
                }
                if lc != edge {
                    return 0;
                }
            }
            lc = next;
        }
        1
    }
}

/// Whether `lc` gives up a row to the pane border status line.
unsafe fn layout_add_horizontal_border(
    w: *mut window,
    lc: *mut layout_cell,
    status: c_int,
) -> c_int {
    unsafe {
        if status == PANE_STATUS_TOP {
            return layout_cell_is_top(w, lc);
        }
        if status == PANE_STATUS_BOTTOM {
            return layout_cell_is_bottom(w, lc);
        }
        0
    }
}

/// Gives every pane the size and place its cell now has, leaving `skip` alone.
pub unsafe fn layout_fix_panes(w: *mut window, skip: *mut window_pane) {
    unsafe {
        let status =
            options_get_number(options_ptr(&(*w).options), c"pane-border-status".as_ptr()) as c_int;
        let mut wp = window_panes_first(w);
        while !wp.is_null() {
            let lc = (*wp).layout_cell;
            if !lc.is_null() && wp != skip {
                (*wp).xoff = (*lc).xoff;
                (*wp).yoff = (*lc).yoff;
                let mut sx = (*lc).sx;
                let mut sy = (*lc).sy;

                if window_pane_is_floating(wp) == 0
                    && layout_add_horizontal_border(w, lc, status) != 0
                {
                    if status == PANE_STATUS_TOP {
                        (*wp).yoff += 1;
                    }
                    sy = sy.wrapping_sub(1);
                }

                if window_pane_show_scrollbar(wp) != 0 {
                    let sb_w = ::core::cmp::max((*wp).scrollbar_style.width, 1);
                    let sb_pad = ::core::cmp::max((*wp).scrollbar_style.pad, 0);
                    if (*w).sb_pos == PANE_SCROLLBARS_LEFT {
                        if sx as c_int - sb_w < PANE_MINIMUM {
                            (*wp).xoff = (*wp).xoff + sx as c_int - PANE_MINIMUM;
                            sx = PANE_MINIMUM as u_int;
                        } else {
                            sx = sx.wrapping_sub(sb_w as u_int).wrapping_sub(sb_pad as u_int);
                            (*wp).xoff = (*wp).xoff + sb_w + sb_pad;
                        }
                    } else if sx as c_int - sb_w - sb_pad < PANE_MINIMUM {
                        sx = PANE_MINIMUM as u_int;
                    } else {
                        sx = sx.wrapping_sub(sb_w as u_int).wrapping_sub(sb_pad as u_int);
                    }
                    (*wp).flags |= PANE_REDRAWSCROLLBAR;
                }

                window_pane_resize(wp, sx, sy);
            }
            wp = window_panes_next(w, wp);
        }
    }
}

pub unsafe fn layout_count_cells(lc: *mut layout_cell) -> u_int {
    unsafe {
        match (*lc).type_0 {
            LAYOUT_WINDOWPANE => 1,
            LAYOUT_LEFTRIGHT | LAYOUT_TOPBOTTOM => {
                let mut count: u_int = 0;
                for lcchild in children(lc) {
                    count = count.wrapping_add(layout_count_cells(lcchild));
                }
                count
            }
            _ => {
                fatalx(c"bad layout type".as_ptr(), fmt_args![]);
            }
        }
    }
}

/// How much `lc` could give up in `type_0`'s direction and still leave every
/// pane under it at least one row or column, plus whatever a scrollbar or a
/// border status line needs.
unsafe fn layout_resize_check(w: *mut window, lc: *mut layout_cell, type_0: layout_type) -> u_int {
    unsafe {
        let sb_style = &raw mut (*window_get_active(w)).scrollbar_style;
        let status =
            options_get_number(options_ptr(&(*w).options), c"pane-border-status".as_ptr()) as c_int;

        if (*lc).type_0 == LAYOUT_WINDOWPANE {
            let (available, minimum) = if type_0 == LAYOUT_LEFTRIGHT {
                let minimum = if (*w).sb != PANE_SCROLLBARS_OFF {
                    (PANE_MINIMUM + (*sb_style).width + (*sb_style).pad) as u_int
                } else {
                    PANE_MINIMUM as u_int
                };
                ((*lc).sx, minimum)
            } else {
                let minimum = if layout_add_horizontal_border(w, lc, status) != 0 {
                    (PANE_MINIMUM + 1) as u_int
                } else {
                    PANE_MINIMUM as u_int
                };
                ((*lc).sy, minimum)
            };
            available.saturating_sub(minimum)
        } else if (*lc).type_0 == type_0 {
            let mut available: u_int = 0;
            for lcchild in children(lc) {
                available = available.wrapping_add(layout_resize_check(w, lcchild, type_0));
            }
            available
        } else {
            let mut minimum = UINT_MAX;
            for lcchild in children(lc) {
                let available = layout_resize_check(w, lcchild, type_0);
                if available < minimum {
                    minimum = available;
                }
            }
            minimum
        }
    }
}

/// Adds `change` to `lc`'s size in `type_0`'s direction, sharing it out among
/// the cells under it one row or column at a time.
pub unsafe fn layout_resize_adjust(
    w: *mut window,
    lc: *mut layout_cell,
    type_0: layout_type,
    mut change: c_int,
) {
    unsafe {
        if type_0 == LAYOUT_LEFTRIGHT {
            (*lc).sx = (*lc).sx.wrapping_add(change as u_int);
        } else {
            (*lc).sy = (*lc).sy.wrapping_add(change as u_int);
        }
        if type_0 == LAYOUT_WINDOWPANE {
            return;
        }
        if (*lc).type_0 != type_0 {
            for lcchild in children(lc) {
                if !floating(lcchild) {
                    layout_resize_adjust(w, lcchild, type_0, change);
                }
            }
            return;
        }
        while change != 0 {
            for lcchild in children(lc) {
                if change == 0 {
                    break;
                }
                if floating(lcchild) {
                    continue;
                }
                if change > 0 {
                    layout_resize_adjust(w, lcchild, type_0, 1);
                    change -= 1;
                } else if layout_resize_check(w, lcchild, type_0) > 0 {
                    layout_resize_adjust(w, lcchild, type_0, -1);
                    change += 1;
                }
            }
        }
    }
}

/// Takes `lc` out of the tree, giving its room to a neighbour and folding away
/// a node that is left with one child. `lcroot` is the window's root cell,
/// which is written when the tree loses or gains a root.
pub unsafe fn layout_destroy_cell(
    w: *mut window,
    lc: *mut layout_cell,
    lcroot: &mut Option<Box<layout_cell>>,
) {
    unsafe {
        let lcparent = (*lc).parent;
        if lcparent.is_null() {
            layout_free_cell(w, lcroot.take());
            return;
        }

        let mut lcother: *mut layout_cell = null_mut::<layout_cell>();
        if !floating(lc) {
            lcother = if lc == first(lcparent) {
                next(lc)
            } else {
                prev(lc)
            };
        }
        if !lcother.is_null() && !floating(lcother) {
            if (*lcparent).type_0 == LAYOUT_LEFTRIGHT {
                layout_resize_adjust(
                    w,
                    lcother,
                    (*lcparent).type_0,
                    (*lc).sx.wrapping_add(1) as c_int,
                );
            } else {
                layout_resize_adjust(
                    w,
                    lcother,
                    (*lcparent).type_0,
                    (*lc).sy.wrapping_add(1) as c_int,
                );
            }
        }

        layout_free_cell(w, unlink(&raw mut (*lcparent).cells, lc));

        let lc = first(lcparent);
        if !lc.is_null() && next(lc).is_null() {
            let only =
                unlink(&raw mut (*lcparent).cells, lc).expect("the cell is under its parent");
            (*lc).parent = (*lcparent).parent;
            if (*lc).parent.is_null() {
                if !floating(lc) {
                    (*lc).xoff = 0;
                    (*lc).yoff = 0;
                }
                // The parent is the root the cell takes the place of, so
                // giving the root up is what frees it.
                layout_free_cell(w, lcroot.replace(only));
            } else {
                layout_free_cell(
                    w,
                    Some(replace(&raw mut (*(*lc).parent).cells, lcparent, only)),
                );
            }
        }
    }
}

pub unsafe fn layout_init(w: *mut window, wp: *mut window_pane) {
    unsafe {
        (*w).layout_root = Some(layout_create_cell(null_mut::<layout_cell>()));
        let lc = layout_root_ptr(&(*w).layout_root);
        layout_set_size(lc, (*w).sx, (*w).sy, 0, 0);
        layout_make_leaf(lc, wp);
        layout_fix_panes(w, null_mut::<window_pane>());
    }
}

pub unsafe fn layout_free(w: *mut window) {
    unsafe { layout_free_cell(w, (*w).layout_root.take()) }
}

/// Resizes the window to `sx` by `sy`, shrinking only as far as its panes
/// allow.
pub unsafe fn layout_resize(w: *mut window, sx: u_int, sy: u_int) {
    unsafe {
        let lc = layout_root_ptr(&(*w).layout_root);
        if (*lc).type_0 == LAYOUT_WINDOWPANE && floating(lc) {
            return;
        }

        let mut xchange = sx.wrapping_sub((*lc).sx) as c_int;
        let xlimit = layout_resize_check(w, lc, LAYOUT_LEFTRIGHT) as c_int;
        if xchange < 0 && xchange < -xlimit {
            xchange = -xlimit;
        }
        if xlimit == 0 {
            xchange = if sx <= (*lc).sx {
                0
            } else {
                sx.wrapping_sub((*lc).sx) as c_int
            };
        }
        if xchange != 0 {
            layout_resize_adjust(w, lc, LAYOUT_LEFTRIGHT, xchange);
        }

        let mut ychange = sy.wrapping_sub((*lc).sy) as c_int;
        let ylimit = layout_resize_check(w, lc, LAYOUT_TOPBOTTOM) as c_int;
        if ychange < 0 && ychange < -ylimit {
            ychange = -ylimit;
        }
        if ylimit == 0 {
            ychange = if sy <= (*lc).sy {
                0
            } else {
                sy.wrapping_sub((*lc).sy) as c_int
            };
        }
        if ychange != 0 {
            layout_resize_adjust(w, lc, LAYOUT_TOPBOTTOM, ychange);
        }

        layout_fix_offsets(w);
        layout_fix_panes(w, null_mut::<window_pane>());
    }
}

/// The cell `wp` sits in that is a child of the nearest node of `type_0`, and
/// that node, or nothing when there is none above the pane.
unsafe fn layout_pane_cell(
    wp: *mut window_pane,
    type_0: layout_type,
) -> Option<(*mut layout_cell, *mut layout_cell)> {
    unsafe {
        let mut lc = (*wp).layout_cell;
        let mut lcparent = (*lc).parent;
        while !lcparent.is_null() && (*lcparent).type_0 != type_0 {
            lc = lcparent;
            lcparent = (*lc).parent;
        }
        if lcparent.is_null() {
            return None;
        }
        Some((lc, lcparent))
    }
}

/// Resizes the pane to `new_size` in `type_0`'s direction by moving the border
/// after it, or the one before it when it is the last.
pub unsafe fn layout_resize_pane_to(wp: *mut window_pane, type_0: layout_type, new_size: u_int) {
    unsafe {
        let Some((lc, lcparent)) = layout_pane_cell(wp, type_0) else {
            return;
        };
        let size = if type_0 == LAYOUT_LEFTRIGHT {
            (*lc).sx as c_int
        } else {
            (*lc).sy as c_int
        };
        let change = if lc == last(lcparent) {
            (size as u_int).wrapping_sub(new_size) as c_int
        } else {
            new_size.wrapping_sub(size as u_int) as c_int
        };
        layout_resize_pane(wp, type_0, change, 1);
    }
}

/// Moves the border after `lc` by `change`, taking the room from or giving it
/// to whichever neighbour has it. `opposite` allows taking it from the cells
/// before `lc` when the ones after have none.
pub unsafe fn layout_resize_layout(
    w: *mut window,
    lc: *mut layout_cell,
    type_0: layout_type,
    change: c_int,
    opposite: c_int,
) {
    unsafe {
        let mut needed = change;
        while needed != 0 {
            let size = if change > 0 {
                let size = layout_resize_pane_grow(w, lc, type_0, needed, opposite);
                needed -= size;
                size
            } else {
                let size = layout_resize_pane_shrink(w, lc, type_0, needed);
                needed += size;
                size
            };
            if size == 0 {
                break;
            }
        }
        layout_fix_offsets(w);
        layout_fix_panes(w, null_mut::<window_pane>());
        notify_window(c"window-layout-changed".as_ptr(), w);
    }
}

pub unsafe fn layout_resize_pane(
    wp: *mut window_pane,
    type_0: layout_type,
    change: c_int,
    opposite: c_int,
) {
    unsafe {
        let Some((mut lc, lcparent)) = layout_pane_cell(wp, type_0) else {
            return;
        };
        if lc == last(lcparent) {
            lc = prev(lc);
        }
        layout_resize_layout((*wp).window, lc, type_0, change, opposite);
    }
}

/// Grows `lc` by up to `needed`, taking the room from the first cell after it
/// that has some — or, with `opposite`, from the first before it.
unsafe fn layout_resize_pane_grow(
    w: *mut window,
    lc: *mut layout_cell,
    type_0: layout_type,
    needed: c_int,
    opposite: c_int,
) -> c_int {
    unsafe {
        let mut size: u_int = 0;
        let mut lcremove = next(lc);
        while !lcremove.is_null() {
            size = layout_resize_check(w, lcremove, type_0);
            if size > 0 {
                break;
            }
            lcremove = next(lcremove);
        }
        if opposite != 0 && lcremove.is_null() {
            lcremove = prev(lc);
            while !lcremove.is_null() {
                size = layout_resize_check(w, lcremove, type_0);
                if size > 0 {
                    break;
                }
                lcremove = prev(lcremove);
            }
        }
        if lcremove.is_null() {
            return 0;
        }
        if size > needed as u_int {
            size = needed as u_int;
        }
        layout_resize_adjust(w, lc, type_0, size as c_int);
        layout_resize_adjust(w, lcremove, type_0, size.wrapping_neg() as c_int);
        size as c_int
    }
}

/// Shrinks `lc` — or the first cell before it that has room to give — by up to
/// `needed`, handing what it gives up to the cell after `lc`.
unsafe fn layout_resize_pane_shrink(
    w: *mut window,
    lc: *mut layout_cell,
    type_0: layout_type,
    needed: c_int,
) -> c_int {
    unsafe {
        let mut lcremove = lc;
        let mut size;
        loop {
            size = layout_resize_check(w, lcremove, type_0);
            if size != 0 {
                break;
            }
            lcremove = prev(lcremove);
            if lcremove.is_null() {
                break;
            }
        }
        if lcremove.is_null() {
            return 0;
        }
        let lcadd = next(lc);
        if lcadd.is_null() {
            return 0;
        }
        if size > -needed as u_int {
            size = -needed as u_int;
        }
        layout_resize_adjust(w, lcadd, type_0, size as c_int);
        layout_resize_adjust(w, lcremove, type_0, size.wrapping_neg() as c_int);
        size as c_int
    }
}

/// Gives `lc` to `wp`, then sizes the window's panes to the tree — leaving
/// `wp` itself alone when `do_not_resize` is set, since the caller has already
/// sized it.
pub unsafe fn layout_assign_pane(lc: *mut layout_cell, wp: *mut window_pane, do_not_resize: c_int) {
    unsafe {
        layout_make_leaf(lc, wp);
        if do_not_resize != 0 {
            layout_fix_panes((*wp).window, wp);
        } else {
            layout_fix_panes((*wp).window, null_mut::<window_pane>());
        }
    }
}

/// The size cell `lc` should take when its parent goes from `previous` to
/// `size`, with `count_left` cells still to place in `size_left`.
unsafe fn layout_new_pane_size(
    w: *mut window,
    previous: u_int,
    lc: *mut layout_cell,
    type_0: layout_type,
    size: u_int,
    count_left: u_int,
    size_left: u_int,
) -> u_int {
    unsafe {
        if count_left == 1 {
            return size_left;
        }

        let available = layout_resize_check(w, lc, type_0);
        let mut min = ((PANE_MINIMUM + 1) as u_int).wrapping_mul(count_left.wrapping_sub(1));
        let mut new_size;
        if type_0 == LAYOUT_LEFTRIGHT {
            if (*lc).sx.wrapping_sub(available) > min {
                min = (*lc).sx.wrapping_sub(available);
            }
            new_size = (*lc).sx.wrapping_mul(size).wrapping_div(previous);
        } else {
            if (*lc).sy.wrapping_sub(available) > min {
                min = (*lc).sy.wrapping_sub(available);
            }
            new_size = (*lc).sy.wrapping_mul(size).wrapping_div(previous);
        }

        let max = size_left.wrapping_sub(min);
        if new_size > max {
            new_size = max;
        }
        if new_size < PANE_MINIMUM as u_int {
            new_size = PANE_MINIMUM as u_int;
        }
        new_size
    }
}

/// Whether the tree under `lc` still fits when its size in `type_0`'s
/// direction becomes `size`.
///
/// The three "does not fit" guards inside the loop are kept as the C wrote
/// them, but no test reaches them: [`layout_new_pane_size`] clamps its answer
/// to what is left less the room the cells after it need, so a child is never
/// handed more than `available`, and a leaf is never handed less than one row
/// or column.
unsafe fn layout_set_size_check(
    w: *mut window,
    lc: *mut layout_cell,
    type_0: layout_type,
    size: c_int,
) -> c_int {
    unsafe {
        if (*lc).type_0 == LAYOUT_WINDOWPANE {
            return (size >= PANE_MINIMUM) as c_int;
        }

        let mut available = size as u_int;
        let count = children(lc).count() as u_int;

        if (*lc).type_0 == type_0 {
            if available < count.wrapping_mul(2).wrapping_sub(1) {
                return 0;
            }
            let previous = if type_0 == LAYOUT_LEFTRIGHT {
                (*lc).sx
            } else {
                (*lc).sy
            };
            for (idx, lcchild) in children(lc).enumerate() {
                let idx = idx as u_int;
                let new_size = layout_new_pane_size(
                    w,
                    previous,
                    lcchild,
                    type_0,
                    size as u_int,
                    count.wrapping_sub(idx),
                    available,
                );
                if idx == count.wrapping_sub(1) {
                    if new_size > available {
                        return 0;
                    }
                    available = available.wrapping_sub(new_size);
                } else {
                    if new_size.wrapping_add(1) > available {
                        return 0;
                    }
                    available = available.wrapping_sub(new_size.wrapping_add(1));
                }
                if layout_set_size_check(w, lcchild, type_0, new_size as c_int) == 0 {
                    return 0;
                }
            }
        } else {
            for lcchild in children(lc) {
                if (*lcchild).type_0 != LAYOUT_WINDOWPANE
                    && layout_set_size_check(w, lcchild, type_0, size) == 0
                {
                    return 0;
                }
            }
        }
        1
    }
}

/// Shares `lc`'s size out among the cells under it, in proportion to what they
/// had before.
unsafe fn layout_resize_child_cells(w: *mut window, lc: *mut layout_cell) {
    unsafe {
        if (*lc).type_0 == LAYOUT_WINDOWPANE {
            return;
        }

        let leftright = (*lc).type_0 == LAYOUT_LEFTRIGHT;
        let mut count: u_int = 0;
        let mut previous: u_int = 0;
        for lcchild in children(lc) {
            if floating(lcchild) {
                continue;
            }
            count = count.wrapping_add(1);
            previous = previous.wrapping_add(if leftright {
                (*lcchild).sx
            } else {
                (*lcchild).sy
            });
        }
        previous = previous.wrapping_add(count.wrapping_sub(1));
        let mut available = if leftright { (*lc).sx } else { (*lc).sy };

        let mut idx: u_int = 0;
        for lcchild in children(lc) {
            if floating(lcchild) {
                continue;
            }
            if !leftright {
                (*lcchild).sx = (*lc).sx;
                (*lcchild).xoff = (*lc).xoff;
            } else {
                (*lcchild).sx = layout_new_pane_size(
                    w,
                    previous,
                    lcchild,
                    (*lc).type_0,
                    (*lc).sx,
                    count.wrapping_sub(idx),
                    available,
                );
                available = available.wrapping_sub((*lcchild).sx.wrapping_add(1));
            }
            if leftright {
                (*lcchild).sy = (*lc).sy;
                (*lcchild).yoff = (*lc).yoff;
            } else {
                (*lcchild).sy = layout_new_pane_size(
                    w,
                    previous,
                    lcchild,
                    (*lc).type_0,
                    (*lc).sy,
                    count.wrapping_sub(idx),
                    available,
                );
                available = available.wrapping_sub((*lcchild).sy.wrapping_add(1));
            }
            layout_resize_child_cells(w, lcchild);
            idx = idx.wrapping_add(1);
        }
    }
}

/// Splits the pane's cell in two and answers the empty half, or null when
/// there is no room. `size` is the new half's size, or -1 for half of what
/// there is; `SPAWN_BEFORE` puts the new half first and `SPAWN_FULLSIZE`
/// splits the whole window rather than the one pane.
pub unsafe fn layout_split_pane(
    wp: *mut window_pane,
    type_0: layout_type,
    mut size: c_int,
    flags: c_int,
) -> *mut layout_cell {
    unsafe {
        let w = (*wp).window;
        let sb_style = &raw mut (*wp).scrollbar_style;
        let full_size = flags & SPAWN_FULLSIZE;
        let before = flags & SPAWN_BEFORE != 0;
        let mut resize_first = false;

        let lc = if full_size != 0 {
            layout_root_ptr(&(*w).layout_root)
        } else {
            (*wp).layout_cell
        };
        let status =
            options_get_number(options_ptr(&(*w).options), c"pane-border-status".as_ptr()) as c_int;

        let sx = (*lc).sx;
        let sy = (*lc).sy;
        let xoff = (*lc).xoff as u_int;
        let yoff = (*lc).yoff as u_int;

        match type_0 {
            LAYOUT_LEFTRIGHT => {
                let minimum = if (*w).sb != PANE_SCROLLBARS_OFF {
                    (PANE_MINIMUM * 2 + (*sb_style).width + (*sb_style).pad) as u_int
                } else {
                    (PANE_MINIMUM * 2 + 1) as u_int
                };
                if sx < minimum {
                    return null_mut::<layout_cell>();
                }
            }
            LAYOUT_TOPBOTTOM => {
                let minimum = if layout_add_horizontal_border(w, lc, status) != 0 {
                    (PANE_MINIMUM * 2 + 2) as u_int
                } else {
                    (PANE_MINIMUM * 2 + 1) as u_int
                };
                if sy < minimum {
                    return null_mut::<layout_cell>();
                }
            }
            _ => {
                fatalx(c"bad layout type".as_ptr(), fmt_args![]);
            }
        }

        let saved_size = if type_0 == LAYOUT_LEFTRIGHT { sx } else { sy };
        let mut size2 = if size < 0 {
            saved_size.wrapping_add(1).wrapping_div(2).wrapping_sub(1)
        } else if before {
            saved_size.wrapping_sub(size as u_int).wrapping_sub(1)
        } else {
            size as u_int
        };
        if size2 < PANE_MINIMUM as u_int {
            size2 = PANE_MINIMUM as u_int;
        } else if size2 > saved_size.wrapping_sub(2) {
            size2 = saved_size.wrapping_sub(2);
        }
        let size1 = saved_size.wrapping_sub(1).wrapping_sub(size2);
        let new_size = if before { size2 } else { size1 };

        if full_size != 0 && layout_set_size_check(w, lc, type_0, new_size as c_int) == 0 {
            return null_mut::<layout_cell>();
        }

        let lcnew;
        if !(*lc).parent.is_null() && (*(*lc).parent).type_0 == type_0 {
            let lcparent = (*lc).parent;
            let new = layout_create_cell(lcparent);
            lcnew = &raw const *new as *mut layout_cell;
            if before {
                insert_before(&raw mut (*lcparent).cells, lc, new);
            } else {
                insert_after(&raw mut (*lcparent).cells, lc, new);
            }
        } else if full_size != 0 && (*lc).parent.is_null() && (*lc).type_0 == type_0 {
            if (*lc).type_0 == LAYOUT_LEFTRIGHT {
                (*lc).sx = new_size;
                layout_resize_child_cells(w, lc);
                (*lc).sx = saved_size;
            } else {
                (*lc).sy = new_size;
                layout_resize_child_cells(w, lc);
                (*lc).sy = saved_size;
            }
            resize_first = true;

            let new = layout_create_cell(lc);
            lcnew = &raw const *new as *mut layout_cell;
            size = saved_size.wrapping_sub(1).wrapping_sub(new_size) as c_int;
            if (*lc).type_0 == LAYOUT_LEFTRIGHT {
                layout_set_size(lcnew, size as u_int, sy, 0, 0);
            } else {
                layout_set_size(lcnew, sx, size as u_int, 0, 0);
            }
            if before {
                insert_head(&raw mut (*lc).cells, new);
            } else {
                insert_tail(&raw mut (*lc).cells, new);
            }
        } else {
            let mut node = layout_create_cell((*lc).parent);
            let lcparent = &raw mut *node;
            layout_make_node(w, lcparent, type_0);
            layout_set_size(lcparent, sx, sy, xoff as c_int, yoff as c_int);
            // The new node takes the cell's place, and the cell goes under it.
            let only = if (*lc).parent.is_null() {
                (*w).layout_root
                    .replace(node)
                    .expect("the cell is the root")
            } else {
                replace(&raw mut (*(*lc).parent).cells, lc, node)
            };
            (*lc).parent = lcparent;
            insert_head(&raw mut (*lcparent).cells, only);
            let new = layout_create_cell(lcparent);
            lcnew = &raw const *new as *mut layout_cell;
            if before {
                insert_head(&raw mut (*lcparent).cells, new);
            } else {
                insert_tail(&raw mut (*lcparent).cells, new);
            }
        }

        let (lc1, lc2) = if before { (lcnew, lc) } else { (lc, lcnew) };
        if !resize_first && type_0 == LAYOUT_LEFTRIGHT {
            layout_set_size(lc1, size1, sy, xoff as c_int, yoff as c_int);
            layout_set_size(
                lc2,
                size2,
                sy,
                xoff.wrapping_add((*lc1).sx).wrapping_add(1) as c_int,
                yoff as c_int,
            );
        } else if !resize_first && type_0 == LAYOUT_TOPBOTTOM {
            layout_set_size(lc1, sx, size1, xoff as c_int, yoff as c_int);
            layout_set_size(
                lc2,
                sx,
                size2,
                xoff as c_int,
                yoff.wrapping_add((*lc1).sy).wrapping_add(1) as c_int,
            );
        }

        if full_size != 0 {
            if !resize_first {
                layout_resize_child_cells(w, lc);
            }
            layout_fix_offsets(w);
        } else {
            layout_make_leaf(lc, wp);
        }

        lcnew
    }
}

/// A cell that floats over the layout at (ox, oy), under a node made for the
/// purpose when the window is still one pane.
pub unsafe fn layout_floating_pane(
    w: *mut window,
    sx: u_int,
    sy: u_int,
    ox: c_int,
    oy: c_int,
) -> *mut layout_cell {
    unsafe {
        let lc = layout_root_ptr(&(*w).layout_root);
        let lcparent = if (*lc).type_0 == LAYOUT_WINDOWPANE {
            let mut node = layout_create_cell(null_mut::<layout_cell>());
            let lcparent = &raw mut *node;
            layout_make_node(w, lcparent, LAYOUT_TOPBOTTOM);
            layout_set_size(lcparent, (*w).sx, (*w).sy, 0, 0);
            let only = (*w)
                .layout_root
                .replace(node)
                .expect("the window has a root");
            (*lc).parent = lcparent;
            insert_head(&raw mut (*lcparent).cells, only);
            lcparent
        } else {
            layout_root_ptr(&(*w).layout_root)
        };

        let new = layout_create_cell(lcparent);
        let lcnew = &raw const *new as *mut layout_cell;
        insert_tail(&raw mut (*lcparent).cells, new);
        (*lcnew).flags |= LAYOUT_CELL_FLOATING;
        layout_set_size(lcnew, sx, sy, ox, oy);
        lcnew
    }
}

pub unsafe fn layout_close_pane(wp: *mut window_pane) {
    unsafe {
        let w = (*wp).window;
        if (*wp).layout_cell.is_null() {
            return;
        }
        layout_destroy_cell(w, (*wp).layout_cell, &mut (*w).layout_root);
        (*wp).layout_cell = null_mut::<layout_cell>();
        if !layout_root_ptr(&(*w).layout_root).is_null() {
            layout_fix_offsets(w);
            layout_fix_panes(w, null_mut::<window_pane>());
        }
        notify_window(c"window-layout-changed".as_ptr(), w);
    }
}

/// Gives every cell under `parent` the same share of its size, and answers
/// whether anything moved.
pub unsafe fn layout_spread_cell(w: *mut window, parent: *mut layout_cell) -> c_int {
    unsafe {
        let number = children(parent).count() as u_int;
        if number <= 1 {
            return 0;
        }
        let leftright = match (*parent).type_0 {
            LAYOUT_LEFTRIGHT => true,
            LAYOUT_TOPBOTTOM => false,
            _ => return 0,
        };
        let status =
            options_get_number(options_ptr(&(*w).options), c"pane-border-status".as_ptr()) as c_int;

        let size = if leftright {
            (*parent).sx
        } else if layout_add_horizontal_border(w, parent, status) != 0 {
            (*parent).sy.wrapping_sub(1)
        } else {
            (*parent).sy
        };

        if size < number.wrapping_sub(1) {
            return 0;
        }
        let each = size
            .wrapping_sub(number.wrapping_sub(1))
            .wrapping_div(number);
        if each == 0 {
            return 0;
        }
        let mut remainder = size
            .wrapping_sub(number.wrapping_mul(each.wrapping_add(1)))
            .wrapping_add(1);

        let mut changed = 0;
        for lc in children(parent) {
            let change = if leftright {
                let mut change = each.wrapping_sub((*lc).sx) as c_int;
                if remainder > 0 {
                    change += 1;
                    remainder = remainder.wrapping_sub(1);
                }
                layout_resize_adjust(w, lc, LAYOUT_LEFTRIGHT, change);
                change
            } else {
                let mut this = if layout_add_horizontal_border(w, lc, status) != 0 {
                    each.wrapping_add(1)
                } else {
                    each
                };
                if remainder > 0 {
                    this = this.wrapping_add(1);
                    remainder = remainder.wrapping_sub(1);
                }
                let change = this.wrapping_sub((*lc).sy) as c_int;
                layout_resize_adjust(w, lc, LAYOUT_TOPBOTTOM, change);
                change
            };
            if change != 0 {
                changed = 1;
            }
        }
        changed
    }
}

/// Shares out the nearest node above `wp` that has room to share.
pub unsafe fn layout_spread_out(wp: *mut window_pane) {
    unsafe {
        let w = (*wp).window;
        let mut parent = (*(*wp).layout_cell).parent;
        while !parent.is_null() {
            if layout_spread_cell(w, parent) != 0 {
                layout_fix_offsets(w);
                layout_fix_panes(w, null_mut::<window_pane>());
                break;
            }
            parent = (*parent).parent;
        }
    }
}

/// The cell a `split-window` should fill, read out of its arguments.
pub unsafe fn layout_get_tiled_cell(
    item: *mut cmdq_item,
    args: &args,
    w: *mut window,
    wp: *mut window_pane,
    mut flags: c_int,
    cause: &mut CString,
) -> *mut layout_cell {
    unsafe {
        *cause = CString::default();
        if window_pane_is_floating(wp) != 0 {
            *cause = c"can't split a floating pane".to_owned();
            return null_mut::<layout_cell>();
        }

        let type_0 = if args_has(args, b'h') != 0 {
            LAYOUT_LEFTRIGHT
        } else {
            LAYOUT_TOPBOTTOM
        };

        let mut curval: u_int = 0;
        if args_has(args, b'l') != 0 || args_has(args, b'p') != 0 {
            curval = if args_has(args, b'f') != 0 {
                if type_0 == LAYOUT_TOPBOTTOM {
                    (*w).sy
                } else {
                    (*w).sx
                }
            } else if type_0 == LAYOUT_TOPBOTTOM {
                (*wp).sy
            } else {
                (*wp).sx
            };
        }

        let mut size = -1;
        let mut parser_cause = None;
        if args_has(args, b'l') != 0 {
            size = args_percentage_and_expand(
                args,
                b'l',
                0,
                INT_MAX as ::core::ffi::c_longlong,
                curval as ::core::ffi::c_longlong,
                item,
                &mut parser_cause,
            ) as c_int;
        } else if args_has(args, b'p') != 0 {
            size = args_strtonum_and_expand(args, b'p', 0, 100, item, &mut parser_cause) as c_int;
            if parser_cause.is_none() {
                size = curval.wrapping_mul(size as u_int).wrapping_div(100) as c_int;
            }
        }
        if parser_cause.is_some() {
            *cause = c"invalid tiled geometry".to_owned();
            return null_mut::<layout_cell>();
        }

        if args_has(args, b'b') != 0 {
            flags |= SPAWN_BEFORE;
        }
        if args_has(args, b'f') != 0 {
            flags |= SPAWN_FULLSIZE;
        }

        window_push_zoom((*wp).window, 1, args_has(args, b'Z'));
        let lc = layout_split_pane(wp, type_0, size, flags);
        if lc.is_null() {
            *cause = c"no space for a new pane".to_owned();
        }
        lc
    }
}

/// The floating cell a `new-pane` should fill, read out of its arguments.
/// Without a place of its own each new pane steps four columns and two rows on
/// from the last, starting over once that walks off the window.
pub unsafe fn layout_get_floating_cell(
    item: *mut cmdq_item,
    args: &args,
    w: *mut window,
    _wp: *mut window_pane,
    cause: &mut Option<CString>,
) -> *mut layout_cell {
    unsafe {
        let mut sx = (*w).sx.wrapping_div(2) as c_int;
        let mut sy = (*w).sy.wrapping_div(4) as c_int;
        let mut ox = INT_MAX;
        let mut oy = INT_MAX;

        if args_has(args, b'x') != 0 {
            sx = args_percentage_and_expand(
                args,
                b'x',
                0,
                (*w).sx.wrapping_sub(1) as ::core::ffi::c_longlong,
                (*w).sx as ::core::ffi::c_longlong,
                item,
                cause,
            ) as c_int;
            if cause.is_some() {
                return null_mut::<layout_cell>();
            }
        }
        if args_has(args, b'y') != 0 {
            sy = args_percentage_and_expand(
                args,
                b'y',
                0,
                (*w).sy.wrapping_sub(1) as ::core::ffi::c_longlong,
                (*w).sy as ::core::ffi::c_longlong,
                item,
                cause,
            ) as c_int;
            if cause.is_some() {
                return null_mut::<layout_cell>();
            }
        }
        if args_has(args, b'X') != 0 {
            ox = args_percentage_and_expand(
                args,
                b'X',
                -sx as ::core::ffi::c_longlong,
                (*w).sx as ::core::ffi::c_longlong,
                (*w).sx as ::core::ffi::c_longlong,
                item,
                cause,
            ) as c_int;
            if cause.is_some() {
                return null_mut::<layout_cell>();
            }
        }
        if args_has(args, b'Y') != 0 {
            oy = args_percentage_and_expand(
                args,
                b'Y',
                -sy as ::core::ffi::c_longlong,
                (*w).sy as ::core::ffi::c_longlong,
                (*w).sy as ::core::ffi::c_longlong,
                item,
                cause,
            ) as c_int;
            if cause.is_some() {
                return null_mut::<layout_cell>();
            }
        }

        if ox == INT_MAX {
            ox = if (*w).last_new_pane_x == 0 {
                4
            } else if (*w).last_new_pane_x > (*w).sx {
                4
            } else {
                (*w).last_new_pane_x.wrapping_add(4) as c_int
            };
            (*w).last_new_pane_x = ox as u_int;
        }
        if oy == INT_MAX {
            oy = if (*w).last_new_pane_y == 0 {
                2
            } else if (*w).last_new_pane_y > (*w).sy {
                2
            } else {
                (*w).last_new_pane_y.wrapping_add(2) as c_int
            };
            (*w).last_new_pane_y = oy as u_int;
        }

        layout_floating_pane(w, sx as u_int, sy as u_int, ox, oy)
    }
}

#[cfg(test)]
#[path = "../tests/test_layout.rs"]
mod tests;

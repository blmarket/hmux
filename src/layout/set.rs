use super::cells::{
    LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, PANE_MINIMUM, insert_new_tail, last, layout_create_cell,
    layout_fix_offsets, layout_fix_panes, layout_free, layout_make_leaf, layout_make_node,
    layout_print_cell, layout_resize_adjust, layout_set_size, layout_spread_cell,
};
use crate::arguments::args_string_percentage;
use crate::notify::notify_window;
use crate::options::{options_get_number, options_get_string};
use crate::server::server_redraw_window;
pub use crate::types::*;
use crate::window::{
    window_count_panes, window_pane_is_floating, window_panes_first, window_panes_next,
    window_resize,
};
use ::core::ffi::{CStr, c_int, c_longlong};
use ::core::ptr::null_mut;

/// The narrowest a pane may be, as the sizes here count.
const MINIMUM: u_int = PANE_MINIMUM as u_int;

/// One of the layouts `select-layout` knows, and what arranges a window into
/// it. Every entry has one, so the "is there one" test the C wrote in front of
/// each call is gone.
struct layout_set_entry {
    name: &'static CStr,
    arrange: unsafe fn(*mut window),
}

static layout_sets: [layout_set_entry; 7] = [
    layout_set_entry {
        name: c"even-horizontal",
        arrange: layout_set_even_h,
    },
    layout_set_entry {
        name: c"even-vertical",
        arrange: layout_set_even_v,
    },
    layout_set_entry {
        name: c"main-horizontal",
        arrange: layout_set_main_h,
    },
    layout_set_entry {
        name: c"main-horizontal-mirrored",
        arrange: layout_set_main_h_mirrored,
    },
    layout_set_entry {
        name: c"main-vertical",
        arrange: layout_set_main_v,
    },
    layout_set_entry {
        name: c"main-vertical-mirrored",
        arrange: layout_set_main_v_mirrored,
    },
    layout_set_entry {
        name: c"tiled",
        arrange: layout_set_tiled,
    },
];

/// The panes of a window, in the order they were made.
struct Panes(*mut window, *mut window_pane);

impl Iterator for Panes {
    type Item = *mut window_pane;

    fn next(&mut self) -> Option<*mut window_pane> {
        if self.1.is_null() {
            return None;
        }
        let wp = self.1;
        self.1 = unsafe { window_panes_next(self.0, wp) };
        Some(wp)
    }
}

/// The panes of `w`, in list order.
unsafe fn panes(w: *mut window) -> Panes {
    unsafe { Panes(w, window_panes_first(w)) }
}

/// The panes of `w` that are not floating.
///
/// Every walk here runs after `layout_free`, which takes each pane's own
/// pointer to its cell away, so nothing looks floating by the time it is
/// asked and this is the whole pane list. The test is kept as upstream wrote
/// it; what it costs is that a floating pane is left out of the count that
/// decides whether there is anything to arrange, and then arranged with the
/// rest anyway.
unsafe fn tiled(w: *mut window) -> impl Iterator<Item = *mut window_pane> {
    unsafe { panes(w) }.filter(|wp| unsafe { window_pane_is_floating(*wp) } == 0)
}

/// The first pane of `w` that is not floating. See [`tiled`].
unsafe fn layout_first_tiled(w: *mut window) -> *mut window_pane {
    unsafe { tiled(w).next().unwrap_or(null_mut::<window_pane>()) }
}

/// The next pane after `wp` that is not floating. See [`tiled`].
unsafe fn next_tiled(wp: *mut window_pane) -> *mut window_pane {
    unsafe {
        let w: *mut window = (*wp).window;
        Panes(w, window_panes_next(w, wp))
            .find(|p| { window_pane_is_floating(*p) } == 0)
            .unwrap_or(null_mut::<window_pane>())
    }
}

/// The layout number `name` names, or -1 when it names none or more than one.
/// A whole name is looked for first, so a name that is also the front of a
/// longer one is that layout rather than an ambiguity.
pub fn layout_set_lookup(name: &CStr) -> c_int {
    let name = name.to_bytes();
    if let Some(i) = layout_sets.iter().position(|e| e.name.to_bytes() == name) {
        return i as c_int;
    }
    let mut matched = -1;
    for (i, entry) in layout_sets.iter().enumerate() {
        if entry.name.to_bytes().starts_with(name) {
            if matched != -1 {
                return -1;
            }
            matched = i as c_int;
        }
    }
    matched
}

/// The last layout there is.
const LAST: u_int = layout_sets.len() as u_int - 1;

/// Arranges `w` into layout number `layout`, or into the last layout when
/// there is no such number.
pub unsafe fn layout_set_select(w: *mut window, layout: u_int) -> u_int {
    unsafe {
        let layout = layout.min(LAST);
        (layout_sets[layout as usize].arrange)(w);
        (*w).lastlayout = layout as c_int;
        layout
    }
}

/// Arranges `w` into the layout after the one it is in, starting again at the
/// first once past the last.
pub unsafe fn layout_set_next(w: *mut window) -> u_int {
    unsafe {
        let layout = match (*w).lastlayout {
            -1 => 0,
            was if was as u_int + 1 > LAST => 0,
            was => was as u_int + 1,
        };
        (layout_sets[layout as usize].arrange)(w);
        (*w).lastlayout = layout as c_int;
        layout
    }
}

/// Arranges `w` into the layout before the one it is in, starting again at the
/// last once past the first.
pub unsafe fn layout_set_previous(w: *mut window) -> u_int {
    unsafe {
        let layout = match (*w).lastlayout {
            -1 | 0 => LAST,
            was => was as u_int - 1,
        };
        (layout_sets[layout as usize].arrange)(w);
        (*w).lastlayout = layout as c_int;
        layout
    }
}

/// The tail every arrangement ends with: give the tree its offsets, give the
/// panes their sizes, log what came out, resize the window to what the root
/// cell became and tell everybody.
unsafe fn finish(w: *mut window, lc: *mut layout_cell, name: &CStr) {
    unsafe {
        layout_fix_offsets(w);
        layout_fix_panes(w, null_mut::<window_pane>());
        layout_print_cell((*w).layout_root_ptr(), name.as_ptr(), 1);
        window_resize(w, (*lc).sx, (*lc).sy, -1, -1);
        notify_window(c"window-layout-changed".as_ptr(), w);
        server_redraw_window(w);
    }
}

/// Shares the window out evenly between its panes, along `type_0`. Each pane
/// keeps at least one column or line, so a window with less room than that
/// grows to fit rather than squeezing them.
unsafe fn layout_set_even(w: *mut window, type_0: layout_type) {
    unsafe {
        layout_print_cell((*w).layout_root_ptr(), c"layout_set_even".as_ptr(), 1);
        let n = window_count_panes(w, 0);
        if n <= 1 {
            return;
        }
        layout_free(w);
        (*w).layout_root = Some(layout_create_cell(null_mut::<layout_cell>()));
        let lc = (*w).layout_root_ptr();
        let needed = n.wrapping_mul(MINIMUM + 1).wrapping_sub(1);
        let (sx, sy) = if type_0 == LAYOUT_LEFTRIGHT {
            (needed.max((*w).sx), (*w).sy)
        } else {
            ((*w).sx, needed.max((*w).sy))
        };
        layout_set_size(lc, sx, sy, 0, 0);
        layout_make_node(w, lc, type_0);
        for wp in tiled(w) {
            let lcnew = insert_new_tail(&mut (*lc).cells, lc);
            layout_make_leaf(lcnew, wp);
            (*lcnew).sx = (*w).sx;
            (*lcnew).sy = (*w).sy;
        }
        layout_spread_cell(w, lc);
        finish(w, lc, c"layout_set_even");
    }
}

unsafe fn layout_set_even_h(w: *mut window) {
    unsafe { layout_set_even(w, LAYOUT_LEFTRIGHT) }
}

unsafe fn layout_set_even_v(w: *mut window) {
    unsafe { layout_set_even(w, LAYOUT_TOPBOTTOM) }
}

/// Which way round a `main-*` layout is built. One axis is divided between the
/// main pane and the rest; the other is the width or height they all share.
struct Axis {
    /// What the root node divides along.
    root: layout_type,
    /// What the other panes are shared out along.
    others: layout_type,
    /// The option holding the main pane's size on the divided axis.
    main_option: &'static CStr,
    /// The option holding the others' size on the divided axis.
    other_option: &'static CStr,
    /// What the main pane takes when its option is no size at all.
    fallback: u_int,
}

impl Axis {
    /// A width and height from a size `along` the divided axis and one
    /// `across` it.
    fn size(&self, along: u_int, across: u_int) -> (u_int, u_int) {
        if self.root == LAYOUT_TOPBOTTOM {
            (across, along)
        } else {
            (along, across)
        }
    }
}

static HORIZONTAL: Axis = Axis {
    root: LAYOUT_TOPBOTTOM,
    others: LAYOUT_LEFTRIGHT,
    main_option: c"main-pane-height",
    other_option: c"other-pane-height",
    fallback: 24,
};

static VERTICAL: Axis = Axis {
    root: LAYOUT_LEFTRIGHT,
    others: LAYOUT_TOPBOTTOM,
    main_option: c"main-pane-width",
    other_option: c"other-pane-width",
    fallback: 80,
};

/// How much of the divided axis, which is `along` long, the main pane and the
/// other panes each take.
///
/// The main pane's size comes from the axis's own option, falling back to a
/// fixed size when that is no size at all. A main pane that would leave the
/// others no room is cut down, and they get one column or line each. Otherwise
/// the others' option decides: a size of zero, one that will not read and one
/// bigger than the axis all leave the others whatever the main pane did not
/// take, a size that would leave the main pane less than it asked for does the
/// same, and anything else is taken as written with the main pane keeping the
/// rest.
unsafe fn main_size(w: *mut window, axis: &Axis, along: u_int) -> (u_int, u_int) {
    unsafe {
        let mut cause = None;
        let s = options_get_string((*w).options_ptr(), axis.main_option.as_ptr());
        let mut main =
            args_string_percentage(s, 0, along as c_longlong, along as c_longlong, &mut cause)
                as u_int;
        if cause.is_some() {
            main = axis.fallback;
        }
        if main.wrapping_add(MINIMUM) >= along {
            main = if along <= MINIMUM + MINIMUM {
                MINIMUM
            } else {
                along.wrapping_sub(MINIMUM)
            };
            return (main, MINIMUM);
        }
        let s = options_get_string((*w).options_ptr(), axis.other_option.as_ptr());
        let mut other =
            args_string_percentage(s, 0, along as c_longlong, along as c_longlong, &mut cause)
                as u_int;
        if cause.is_some() || other == 0 {
            other = along.wrapping_sub(main);
        } else if other > along || along.wrapping_sub(other) < main {
            other = along.wrapping_sub(main);
        } else {
            main = along.wrapping_sub(other);
        }
        (main, other)
    }
}

/// Hangs the main pane's cell at the end of `lc`.
unsafe fn main_cell(w: *mut window, lc: *mut layout_cell, axis: &Axis, main: u_int, across: u_int) {
    unsafe {
        let lcmain = insert_new_tail(&mut (*lc).cells, lc);
        let (sx, sy) = axis.size(main, across);
        layout_set_size(lcmain, sx, sy, 0, 0);
        layout_make_leaf(lcmain, layout_first_tiled(w));
    }
}

/// Hangs the other `n` panes at the end of `lc`: one cell when there is only
/// one of them, otherwise a node holding one cell each, shared out evenly.
unsafe fn other_cells(
    w: *mut window,
    lc: *mut layout_cell,
    axis: &Axis,
    n: u_int,
    other: u_int,
    across: u_int,
) {
    unsafe {
        let lcother = insert_new_tail(&mut (*lc).cells, lc);
        let (sx, sy) = axis.size(other, across);
        layout_set_size(lcother, sx, sy, 0, 0);
        let first = layout_first_tiled(w);
        if n == 1 {
            layout_make_leaf(lcother, next_tiled(first));
            return;
        }
        layout_make_node(w, lcother, axis.others);
        for wp in tiled(w) {
            if wp == first {
                continue;
            }
            let lcchild = insert_new_tail(&mut (*lcother).cells, lcother);
            let (sx, sy) = axis.size(other, MINIMUM);
            layout_set_size(lcchild, sx, sy, 0, 0);
            layout_make_leaf(lcchild, wp);
        }
        layout_spread_cell(w, lcother);
    }
}

/// Gives one pane the room and shares what is left between the rest. The
/// mirrored form is the same layout with the other panes written first.
unsafe fn layout_set_main(w: *mut window, axis: &Axis, mirrored: bool, name: &CStr) {
    unsafe {
        layout_print_cell((*w).layout_root_ptr(), name.as_ptr(), 1);
        let n = window_count_panes(w, 0);
        if n <= 1 {
            return;
        }
        let n = n.wrapping_sub(1);
        let (along, across) = if axis.root == LAYOUT_TOPBOTTOM {
            ((*w).sy, (*w).sx)
        } else {
            ((*w).sx, (*w).sy)
        };
        let along = along.wrapping_sub(1);
        let (main, other) = main_size(w, axis, along);
        let across = n.wrapping_mul(MINIMUM + 1).wrapping_sub(1).max(across);

        layout_free(w);
        (*w).layout_root = Some(layout_create_cell(null_mut::<layout_cell>()));
        let lc = (*w).layout_root_ptr();
        let (sx, sy) = axis.size(main.wrapping_add(other).wrapping_add(1), across);
        layout_set_size(lc, sx, sy, 0, 0);
        layout_make_node(w, lc, axis.root);
        if mirrored {
            other_cells(w, lc, axis, n, other, across);
            main_cell(w, lc, axis, main, across);
        } else {
            main_cell(w, lc, axis, main, across);
            other_cells(w, lc, axis, n, other, across);
        }
        finish(w, lc, name);
    }
}

unsafe fn layout_set_main_h(w: *mut window) {
    unsafe { layout_set_main(w, &HORIZONTAL, false, c"layout_set_main_h") }
}

unsafe fn layout_set_main_h_mirrored(w: *mut window) {
    unsafe { layout_set_main(w, &HORIZONTAL, true, c"layout_set_main_h_mirrored") }
}

unsafe fn layout_set_main_v(w: *mut window) {
    unsafe { layout_set_main(w, &VERTICAL, false, c"layout_set_main_v") }
}

unsafe fn layout_set_main_v_mirrored(w: *mut window) {
    unsafe { layout_set_main(w, &VERTICAL, true, c"layout_set_main_v_mirrored") }
}

/// Fills the window with rows of equally sized panes, at most
/// `tiled-layout-max-columns` to a row.
///
/// The rows and columns are chosen as the smallest grid that holds every pane,
/// so the grid is never a whole row bigger than it needs: the row loop's
/// "nothing left" guard cannot fire at the top of a row, only after the last
/// pane of the last row.
unsafe fn layout_set_tiled(w: *mut window) {
    unsafe {
        layout_print_cell((*w).layout_root_ptr(), c"layout_set_tiled".as_ptr(), 1);
        let n = window_count_panes(w, 0);
        if n <= 1 {
            return;
        }
        let max_columns =
            options_get_number((*w).options_ptr(), c"tiled-layout-max-columns".as_ptr()) as u_int;
        let mut columns: u_int = 1;
        let mut rows: u_int = 1;
        while rows.wrapping_mul(columns) < n {
            rows += 1;
            if rows.wrapping_mul(columns) < n && (max_columns == 0 || columns < max_columns) {
                columns += 1;
            }
        }
        let width = (*w)
            .sx
            .wrapping_sub(columns.wrapping_sub(1))
            .wrapping_div(columns)
            .max(MINIMUM);
        let height = (*w)
            .sy
            .wrapping_sub(rows.wrapping_sub(1))
            .wrapping_div(rows)
            .max(MINIMUM);

        layout_free(w);
        (*w).layout_root = Some(layout_create_cell(null_mut::<layout_cell>()));
        let lc = (*w).layout_root_ptr();
        let sx = width
            .wrapping_add(1)
            .wrapping_mul(columns)
            .wrapping_sub(1)
            .max((*w).sx);
        let sy = height
            .wrapping_add(1)
            .wrapping_mul(rows)
            .wrapping_sub(1)
            .max((*w).sy);
        layout_set_size(lc, sx, sy, 0, 0);
        layout_make_node(w, lc, LAYOUT_TOPBOTTOM);

        let mut wp = layout_first_tiled(w);
        for j in 0..rows {
            if wp.is_null() {
                break;
            }
            let lcrow = insert_new_tail(&mut (*lc).cells, lc);
            layout_set_size(lcrow, (*w).sx, height, 0, 0);
            if n.wrapping_sub(j.wrapping_mul(columns)) == 1 || columns == 1 {
                layout_make_leaf(lcrow, wp);
                wp = next_tiled(wp);
                continue;
            }
            layout_make_node(w, lcrow, LAYOUT_LEFTRIGHT);
            let mut i = 0;
            while i < columns {
                let lcchild = insert_new_tail(&mut (*lcrow).cells, lcrow);
                layout_set_size(lcchild, width, height, 0, 0);
                layout_make_leaf(lcchild, wp);
                wp = next_tiled(wp);
                if wp.is_null() {
                    break;
                }
                i += 1;
            }
            if i == columns {
                i -= 1;
            }
            let used = i
                .wrapping_add(1)
                .wrapping_mul(width.wrapping_add(1))
                .wrapping_sub(1);
            if (*w).sx > used {
                layout_resize_adjust(
                    w,
                    last(lcrow),
                    LAYOUT_LEFTRIGHT,
                    (*w).sx.wrapping_sub(used) as c_int,
                );
            }
        }
        let used = rows.wrapping_mul(height).wrapping_add(rows).wrapping_sub(1);
        if (*w).sy > used {
            layout_resize_adjust(
                w,
                last(lc),
                LAYOUT_TOPBOTTOM,
                (*w).sy.wrapping_sub(used) as c_int,
            );
        }
        finish(w, lc, c"layout_set_tiled");
    }
}

#[cfg(test)]
#[path = "../tests/test_layout_set.rs"]
mod tests;

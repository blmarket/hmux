use crate::{
    args::argument_text::{ArgumentTextCodec as _, RustArgumentTextCodec},
    window_dimensions::WindowDimensionsState,
};

use super::cells::{
    LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LayoutCellPath, PANE_MINIMUM, insert_new_tail,
    layout_create_cell, layout_make_leaf, layout_print_cell, layout_set_size,
};
use crate::notify::notify_window;

pub use crate::types::*;
use crate::window::{window_count_panes, window_pane_is_floating};
use ::core::ffi::{CStr, c_int, c_longlong};

/// The narrowest a pane may be, as the sizes here count.
const MINIMUM: u_int = PANE_MINIMUM as u_int;

/// One of the layouts `select-layout` knows, and what arranges a window into
/// it. Every entry has one, so the "is there one" test the C wrote in front of
/// each call is gone.
struct layout_set_entry {
    name: &'static CStr,
    arrange: unsafe fn(&WindowRef),
}

static layout_sets: [layout_set_entry; 7] = [
    layout_set_entry {
        name: c"even-horizontal",
        arrange: WindowRef::layout_set_even_h,
    },
    layout_set_entry {
        name: c"even-vertical",
        arrange: WindowRef::layout_set_even_v,
    },
    layout_set_entry {
        name: c"main-horizontal",
        arrange: WindowRef::layout_set_main_h,
    },
    layout_set_entry {
        name: c"main-horizontal-mirrored",
        arrange: WindowRef::layout_set_main_h_mirrored,
    },
    layout_set_entry {
        name: c"main-vertical",
        arrange: WindowRef::layout_set_main_v,
    },
    layout_set_entry {
        name: c"main-vertical-mirrored",
        arrange: WindowRef::layout_set_main_v_mirrored,
    },
    layout_set_entry {
        name: c"tiled",
        arrange: WindowRef::layout_set_tiled,
    },
];

/// The IDs of panes to arrange, in pane-index order. Arrangement calls this
/// after `layout_free`, which clears the cells used to identify floating
/// panes, preserving the existing inclusion of those panes in the layout.
fn tiled(owner: &WindowRef) -> Vec<u_int> {
    let payload = owner.as_window();
    let w = &*payload;
    w.panes
        .iter()
        .filter(|pane| window_pane_is_floating(w, &pane.downgrade()) == 0)
        .map(|pane| pane.pane_id())
        .collect()
}

fn pane_for_layout(panes: &mut [RustWindowPaneRef], id: u_int) -> &mut (impl crate::WindowPane + ?Sized) {
    panes
        .iter_mut()
        .find(|pane| pane.pane_id() == id)
        .map(|pane| unsafe { pane.as_pane_mut() })
        .expect("layout arrangement keeps its panes")
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

/// The tail every arrangement ends with: give the tree its offsets, give the
/// panes their sizes, log what came out, resize the window to what the root
/// cell became and tell everybody.
unsafe fn finish(owner: &WindowRef, name: &CStr) {
    unsafe {
        owner.fix_layout_offsets();
        owner.fix_layout_panes(None);
        let mut payload = owner.as_window_mut();
        let w = &mut *payload;
        layout_print_cell(w.layout_root.as_deref(), name, 1);
        let root = w
            .layout_root
            .as_deref()
            .expect("an arranged window has a layout");
        let size = (root.sx, root.sy);
        drop(payload);
        owner.resize(size.0, size.1, -1, -1);
        notify_window(c"window-layout-changed", Some(owner));
        owner.redraw();
    }
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
fn main_size(owner: &WindowRef, axis: &Axis, along: u_int) -> (u_int, u_int) {
    {
        let mut cause = None;
        let s = owner.options().string_ref(axis.main_option);
        let mut main = RustArgumentTextCodec.percentage(
            &s,
            0,
            along as c_longlong,
            along as c_longlong,
            &mut cause,
        ) as u_int;
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
        let s = owner.options().string_ref(axis.other_option);
        let mut other = RustArgumentTextCodec.percentage(
            &s,
            0,
            along as c_longlong,
            along as c_longlong,
            &mut cause,
        ) as u_int;
        if cause.is_some() || other == 0 || other > along || along.wrapping_sub(other) < main {
            other = along.wrapping_sub(main);
        } else {
            main = along.wrapping_sub(other);
        }
        (main, other)
    }
}

/// Hangs the main pane's cell at the end of the layout root.
fn main_cell(owner: &WindowRef, axis: &Axis, main: u_int, across: u_int) {
    {
        let id = tiled(owner)
            .into_iter()
            .next()
            .expect("the main pane exists");
        let mut payload = owner.as_window_mut();
        let w = &mut *payload;
        let root = w
            .layout_root
            .as_deref_mut()
            .expect("the main layout has a root");
        let lcmain = insert_new_tail(root);
        let (sx, sy) = axis.size(main, across);
        layout_set_size(lcmain, sx, sy, 0, 0);
        layout_make_leaf(lcmain, pane_for_layout(&mut w.panes, id));
    }
}

/// Hangs the other `n` panes under the layout root: one cell when there is only
/// one of them, otherwise a node holding one cell each, shared out evenly.
unsafe fn other_cells(owner: &WindowRef, axis: &Axis, n: u_int, other: u_int, across: u_int) {
    unsafe {
        let panes = tiled(owner);
        let mut payload = owner.as_window_mut();
        let w = &mut *payload;
        let root = w
            .layout_root
            .as_deref_mut()
            .expect("the main layout has a root");
        let path = LayoutCellPath::root().child(root.cells.len());
        let lcother = insert_new_tail(root);
        let (sx, sy) = axis.size(other, across);
        layout_set_size(lcother, sx, sy, 0, 0);
        if n == 1 {
            layout_make_leaf(lcother, pane_for_layout(&mut w.panes, panes[1]));
            return;
        }
        lcother.type_0 = axis.others;
        for id in panes.into_iter().skip(1) {
            let lcchild = insert_new_tail(lcother);
            let (sx, sy) = axis.size(other, MINIMUM);
            layout_set_size(lcchild, sx, sy, 0, 0);
            layout_make_leaf(lcchild, pane_for_layout(&mut w.panes, id));
        }
        drop(payload);
        owner.spread_layout_cell(&path);
    }
}

#[cfg(test)]
#[path = "../tests/test_layout_set.rs"]
mod tests;

impl WindowRef {
    /// Arranges `w` into layout number `layout`, or into the last layout when
    /// there is no such number.
    pub unsafe fn select_layout(&self, layout: u_int) -> u_int {
        let owner = self;

        unsafe {
            let layout = layout.min(LAST);
            (layout_sets[layout as usize].arrange)(owner);
            owner.remember_layout(layout as c_int);
            layout
        }
    }
    /// Arranges `w` into the layout after the one it is in, starting again at the
    /// first once past the last.
    pub unsafe fn select_next_layout(&self) -> u_int {
        let owner = self;

        unsafe {
            let layout = match owner.previous_layout() {
                None => 0,
                Some(was) if was as u_int + 1 > LAST => 0,
                Some(was) => was as u_int + 1,
            };
            (layout_sets[layout as usize].arrange)(owner);
            owner.remember_layout(layout as c_int);
            layout
        }
    }
    /// Arranges `w` into the layout before the one it is in, starting again at the
    /// last once past the first.
    pub unsafe fn select_previous_layout(&self) -> u_int {
        let owner = self;

        unsafe {
            let layout = match owner.previous_layout() {
                None | Some(0) => LAST,
                Some(was) => was as u_int - 1,
            };
            (layout_sets[layout as usize].arrange)(owner);
            owner.remember_layout(layout as c_int);
            layout
        }
    }
    /// Shares the window out evenly between its panes, along `type_0`. Each pane
    /// keeps at least one column or line, so a window with less room than that
    /// grows to fit rather than squeezing them.
    unsafe fn layout_set_even(&self, type_0: layout_type) {
        let owner = self;

        unsafe {
            let payload = owner.as_window();
            let w = &*payload;
            layout_print_cell(w.layout_root.as_deref(), c"layout_set_even", 1);
            let n = window_count_panes(w, 0);
            if n <= 1 {
                return;
            }
            drop(payload);
            owner.free_layout();
            let panes = tiled(owner);
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            let needed = n.wrapping_mul(MINIMUM + 1).wrapping_sub(1);
            let (sx, sy) = if type_0 == LAYOUT_LEFTRIGHT {
                (
                    needed.max(w.dimensions().size.width),
                    w.dimensions().size.height,
                )
            } else {
                (
                    w.dimensions().size.width,
                    needed.max(w.dimensions().size.height),
                )
            };
            let (wsx, wsy) = (w.dimensions().size.width, w.dimensions().size.height);
            let root = w.layout_root.insert(layout_create_cell(None));
            layout_set_size(root, sx, sy, 0, 0);
            root.type_0 = type_0;
            for id in panes {
                let lcnew = insert_new_tail(root);
                layout_make_leaf(lcnew, pane_for_layout(&mut w.panes, id));
                lcnew.sx = wsx;
                lcnew.sy = wsy;
            }
            drop(payload);
            owner.spread_layout_cell(&LayoutCellPath::root());
            finish(owner, c"layout_set_even");
        }
    }
    unsafe fn layout_set_even_h(&self) {
        let owner = self;

        unsafe { owner.layout_set_even(LAYOUT_LEFTRIGHT) }
    }
    unsafe fn layout_set_even_v(&self) {
        let owner = self;

        unsafe { owner.layout_set_even(LAYOUT_TOPBOTTOM) }
    }
    /// Gives one pane the room and shares what is left between the rest. The
    /// mirrored form is the same layout with the other panes written first.
    unsafe fn layout_set_main(&self, axis: &Axis, mirrored: bool, name: &CStr) {
        let owner = self;

        unsafe {
            let payload = owner.as_window();
            let w = &*payload;
            layout_print_cell(w.layout_root.as_deref(), name, 1);
            let n = window_count_panes(w, 0);
            if n <= 1 {
                return;
            }
            let n = n.wrapping_sub(1);
            let (along, across) = if axis.root == LAYOUT_TOPBOTTOM {
                (w.dimensions().size.height, w.dimensions().size.width)
            } else {
                (w.dimensions().size.width, w.dimensions().size.height)
            };
            let along = along.wrapping_sub(1);
            let (main, other) = main_size(owner, axis, along);
            let across = n.wrapping_mul(MINIMUM + 1).wrapping_sub(1).max(across);

            drop(payload);
            owner.free_layout();
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            let root = w.layout_root.insert(layout_create_cell(None));
            let (sx, sy) = axis.size(main.wrapping_add(other).wrapping_add(1), across);
            layout_set_size(root, sx, sy, 0, 0);
            root.type_0 = axis.root;
            drop(payload);
            if mirrored {
                other_cells(owner, axis, n, other, across);
                main_cell(owner, axis, main, across);
            } else {
                main_cell(owner, axis, main, across);
                other_cells(owner, axis, n, other, across);
            }
            finish(owner, name);
        }
    }
    unsafe fn layout_set_main_h(&self) {
        let owner = self;

        unsafe { owner.layout_set_main(&HORIZONTAL, false, c"layout_set_main_h") }
    }
    unsafe fn layout_set_main_h_mirrored(&self) {
        let owner = self;

        unsafe { owner.layout_set_main(&HORIZONTAL, true, c"layout_set_main_h_mirrored") }
    }
    unsafe fn layout_set_main_v(&self) {
        let owner = self;

        unsafe { owner.layout_set_main(&VERTICAL, false, c"layout_set_main_v") }
    }
    unsafe fn layout_set_main_v_mirrored(&self) {
        let owner = self;

        unsafe { owner.layout_set_main(&VERTICAL, true, c"layout_set_main_v_mirrored") }
    }
    /// Fills the window with rows of equally sized panes, at most
    /// `tiled-layout-max-columns` to a row.
    ///
    /// The rows and columns are chosen as the smallest grid that holds every pane,
    /// so the grid is never a whole row bigger than it needs: the row loop's
    /// "nothing left" guard cannot fire at the top of a row, only after the last
    /// pane of the last row.
    unsafe fn layout_set_tiled(&self) {
        let owner = self;

        unsafe {
            let payload = owner.as_window();
            let w = &*payload;
            layout_print_cell(w.layout_root.as_deref(), c"layout_set_tiled", 1);
            let n = window_count_panes(w, 0);
            if n <= 1 {
                return;
            }
            let max_columns = w.options_ref().number(c"tiled-layout-max-columns") as u_int;
            let mut columns: u_int = 1;
            let mut rows: u_int = 1;
            while rows.wrapping_mul(columns) < n {
                rows += 1;
                if rows.wrapping_mul(columns) < n && (max_columns == 0 || columns < max_columns) {
                    columns += 1;
                }
            }
            let width = w
                .dimensions()
                .size
                .width
                .wrapping_sub(columns.wrapping_sub(1))
                .wrapping_div(columns)
                .max(MINIMUM);
            let height = w
                .dimensions()
                .size
                .height
                .wrapping_sub(rows.wrapping_sub(1))
                .wrapping_div(rows)
                .max(MINIMUM);

            drop(payload);
            owner.free_layout();
            let mut panes = tiled(owner).into_iter().peekable();
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            let size = w.dimensions().size;
            let sx = width
                .wrapping_add(1)
                .wrapping_mul(columns)
                .wrapping_sub(1)
                .max(w.dimensions().size.width);
            let sy = height
                .wrapping_add(1)
                .wrapping_mul(rows)
                .wrapping_sub(1)
                .max(w.dimensions().size.height);
            let root = w.layout_root.insert(layout_create_cell(None));
            layout_set_size(root, sx, sy, 0, 0);
            root.type_0 = LAYOUT_TOPBOTTOM;

            drop(payload);
            for j in 0..rows {
                let mut payload = owner.as_window_mut();
                let w = &mut *payload;
                if panes.peek().is_none() {
                    break;
                }
                let root = w
                    .layout_root
                    .as_deref_mut()
                    .expect("the tiled layout has a root");
                let row_path = LayoutCellPath::root().child(root.cells.len());
                let lcrow = insert_new_tail(root);
                layout_set_size(lcrow, size.width, height, 0, 0);
                if n.wrapping_sub(j.wrapping_mul(columns)) == 1 || columns == 1 {
                    let id = panes.next().expect("the row has a pane");
                    layout_make_leaf(lcrow, pane_for_layout(&mut w.panes, id));
                    continue;
                }
                lcrow.type_0 = LAYOUT_LEFTRIGHT;
                let mut i = 0;
                while i < columns {
                    let lcchild = insert_new_tail(lcrow);
                    layout_set_size(lcchild, width, height, 0, 0);
                    let id = panes.next().expect("the row has another pane");
                    layout_make_leaf(lcchild, pane_for_layout(&mut w.panes, id));
                    if panes.peek().is_none() {
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
                if size.width > used {
                    let last = row_path.child(lcrow.cells.len() - 1);
                    drop(payload);
                    owner.layout_resize_adjust_path(
                        &last,
                        LAYOUT_LEFTRIGHT,
                        size.width.wrapping_sub(used) as c_int,
                    );
                }
            }
            let used = rows.wrapping_mul(height).wrapping_add(rows).wrapping_sub(1);
            if size.height > used {
                let payload = owner.as_window();
                let w = &*payload;
                let root = w
                    .layout_root
                    .as_deref()
                    .expect("the tiled layout has a root");
                let last = LayoutCellPath::root().child(root.cells.len() - 1);
                drop(payload);
                owner.layout_resize_adjust_path(
                    &last,
                    LAYOUT_TOPBOTTOM,
                    size.height.wrapping_sub(used) as c_int,
                );
            }
            finish(owner, c"layout_set_tiled");
        }
    }
}

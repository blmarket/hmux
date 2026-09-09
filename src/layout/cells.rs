use crate::args::RustArguments;
use crate::cmd::cmdq_item;
use crate::WindowPane;
use crate::window_scrollbar::WindowScrollbarState;

use crate::window_dimensions::WindowDimensionsState;

use crate::args::{args_has, args_percentage_and_expand, args_strtonum_and_expand};
use crate::fmt_args;
use crate::log::{fatalx, log_debug};
use crate::notify::notify_window;

use crate::pane_geometry::PaneGeometryState;
use crate::pane_scrollbar_style::PaneScrollbarStyleState;
pub use crate::types::*;
use crate::window::{window_pane_resize, window_pane_show_scrollbar};
use ::core::ffi::{CStr, c_int};
use ::std::ffi::CString;

pub use crate::consts::{
    INT_MAX, LAYOUT_CELL_FLOATING, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE,
    PANE_MINIMUM, PANE_REDRAWSCROLLBAR, PANE_SCROLLBARS_LEFT, PANE_SCROLLBARS_OFF,
    PANE_STATUS_BOTTOM, PANE_STATUS_TOP, SPAWN_BEFORE, SPAWN_FULLSIZE, UINT_MAX,
};

/// The cells directly under `lc`, in their stored order, as scoped mutable
/// borrows.
fn children(lc: &mut layout_cell) -> impl Iterator<Item = &mut layout_cell> {
    lc.cells.iter_mut().map(Box::as_mut)
}

/// Puts `lc` at the end of `head`.
pub(crate) fn insert_tail(head: &mut layout_cells, lc: Box<layout_cell>) {
    head.push(lc)
}

/// Inserts a new child and borrows it for sizing and pane assignment.
pub(crate) fn insert_new_tail(lcparent: &mut layout_cell) -> &mut layout_cell {
    let lc = layout_create_cell(Some(lcparent));
    insert_tail(&mut lcparent.cells, lc);
    lcparent.cells.last_mut().expect("the child was inserted")
}

/// Whether `lc` is one of the floating cells, which sit over the layout rather
/// than inside it and are left out of every sum.
fn floating(lc: &layout_cell) -> bool {
    lc.flags & LAYOUT_CELL_FLOATING != 0
}

/// A new cell under `lcparent`, as large as a cell can be until somebody sizes
/// it.
pub fn layout_create_cell(lcparent: Option<&mut layout_cell>) -> Box<layout_cell> {
    Box::new(layout_cell {
        type_0: LAYOUT_WINDOWPANE,
        flags: 0,
        has_parent: lcparent.is_some(),
        sx: UINT_MAX,
        sy: UINT_MAX,
        xoff: INT_MAX,
        yoff: INT_MAX,
        wp_ref: None,
        cells: layout_cells::new(),
    })
}

/// Drops the owned cell and its descendants.
pub fn layout_free_cell(lc: Option<Box<layout_cell>>) {
    drop(lc);
}

/// Writes the tree under `lc` to the debug log, `n` levels in.
pub unsafe fn layout_print_cell(lc: Option<&layout_cell>, hdr: &CStr, n: u_int) {
    unsafe {
        let Some(lc) = lc else {
            return;
        };
        let type_0 = match lc.type_0 {
            LAYOUT_LEFTRIGHT => c"LEFTRIGHT",
            LAYOUT_TOPBOTTOM => c"TOPBOTTOM",
            LAYOUT_WINDOWPANE => c"WINDOWPANE",
            _ => c"UNKNOWN",
        };
        log_debug(
            c"%s:%*stype %s wp=%%%u [%d,%d %ux%u]",
            fmt_args![
                hdr,
                n,
                c" ",
                type_0,
                lc.wp_ref.as_ref().map_or(u_int::MAX, |pane| pane.id()),
                lc.xoff,
                lc.yoff,
                lc.sx,
                lc.sy
            ],
        );
        if lc.type_0 == LAYOUT_LEFTRIGHT || lc.type_0 == LAYOUT_TOPBOTTOM {
            for lcchild in &lc.cells {
                layout_print_cell(Some(lcchild), hdr, n.wrapping_add(1));
            }
        }
    }
}

/// A position in one layout tree, valid while its child ordering is unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutCellPath(Vec<usize>);

impl LayoutCellPath {
    pub fn root() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn child(&self, index: usize) -> Self {
        let mut path = self.clone();
        path.0.push(index);
        path
    }

    pub fn get<'a>(&self, root: &'a layout_cell) -> Option<&'a layout_cell> {
        self.0
            .iter()
            .try_fold(root, |cell, &index| cell.cells.get(index).map(Box::as_ref))
    }

    pub(crate) fn get_mut<'a>(&self, root: &'a mut layout_cell) -> Option<&'a mut layout_cell> {
        self.0.iter().try_fold(root, |cell, &index| {
            cell.cells.get_mut(index).map(Box::as_mut)
        })
    }

    pub(crate) fn parent(&self) -> Option<Self> {
        let mut path = self.clone();
        path.0.pop()?;
        Some(path)
    }

    pub(crate) fn for_pane(root: &layout_cell, pane: &RustWindowPaneWeak) -> Option<Self> {
        if root.wp_ref.as_ref() == Some(pane) {
            return Some(Self(Vec::new()));
        }
        root.cells.iter().enumerate().find_map(|(index, child)| {
            let mut path = Self::for_pane(child, pane)?;
            path.0.insert(0, index);
            Some(path)
        })
    }
}

/// The path to the cell whose border contains (x, y), or `None` inside a pane.
pub fn layout_search_by_border(lc: &layout_cell, x: u_int, y: u_int) -> Option<LayoutCellPath> {
    for (index, child) in lc.cells.iter().enumerate() {
        if x as c_int >= child.xoff
            && (x as c_int) < child.xoff + child.sx as c_int
            && y as c_int >= child.yoff
            && (y as c_int) < child.yoff + child.sy as c_int
        {
            let mut path = layout_search_by_border(child, x, y)?;
            path.0.insert(0, index);
            return Some(path);
        }
        if index != 0 {
            let last = &lc.cells[index - 1];
            let between = match lc.type_0 {
                LAYOUT_LEFTRIGHT => {
                    (x as c_int) < child.xoff && x as c_int >= last.xoff + last.sx as c_int
                }
                LAYOUT_TOPBOTTOM => {
                    (y as c_int) < child.yoff && y as c_int >= last.yoff + last.sy as c_int
                }
                _ => false,
            };
            if between {
                return Some(LayoutCellPath(vec![index - 1]));
            }
        }
    }
    None
}

pub unsafe fn layout_set_size(
    lc: &mut layout_cell,
    sx: u_int,
    sy: u_int,
    xoff: c_int,
    yoff: c_int,
) {
    {
        lc.sx = sx;
        lc.sy = sy;
        lc.xoff = xoff;
        lc.yoff = yoff;
    }
}

/// Observes the pane allocation held by the cell while it remains alive.
#[cfg(test)]
pub fn layout_cell_pane(lc: &layout_cell) -> Option<RustWindowPaneWeak> {
    lc.wp_ref
        .clone()
        .filter(|pane| unsafe { pane.get().is_some() })
}

/// Records the pane allocation held by the cell.
pub fn layout_cell_set_pane(lc: &mut layout_cell, pane: Option<RustWindowPaneWeak>) {
    lc.wp_ref = pane;
}

pub fn layout_make_leaf(lc: &mut layout_cell, wp: &impl crate::WindowPane) {
    lc.type_0 = LAYOUT_WINDOWPANE;
    lc.cells.clear();
    lc.wp_ref = unsafe { crate::window::window_pane_ref_of(wp) };
}

pub fn layout_make_node(lc: &mut layout_cell, type_0: layout_type) {
    if type_0 == LAYOUT_WINDOWPANE {
        unsafe { fatalx(c"bad layout type", fmt_args![]) };
    }
    lc.type_0 = type_0;
    lc.cells.clear();
    lc.wp_ref = None;
}

/// Gives every cell under `lc` the offset its size and its siblings' put it at.
unsafe fn layout_fix_offsets1(lc: &mut layout_cell) {
    unsafe {
        let leftright = lc.type_0 == LAYOUT_LEFTRIGHT;
        let mut off = if leftright { lc.xoff } else { lc.yoff };
        let (parent_xoff, parent_yoff) = (lc.xoff, lc.yoff);
        for lcchild in children(lc) {
            if floating(lcchild) {
                continue;
            }
            if leftright {
                lcchild.xoff = off;
                lcchild.yoff = parent_yoff;
            } else {
                lcchild.xoff = parent_xoff;
                lcchild.yoff = off;
            }
            if lcchild.type_0 != LAYOUT_WINDOWPANE {
                layout_fix_offsets1(lcchild);
            }
            let size = if leftright { lcchild.sx } else { lcchild.sy };
            off = (off as u_int).wrapping_add(size.wrapping_add(1)) as c_int;
        }
    }
}

fn layout_cell_at_edge(root: Option<&layout_cell>, target: &layout_cell, bottom: bool) -> bool {
    let mut pending: Vec<_> = root.into_iter().collect();
    while let Some(cell) = pending.pop() {
        if core::ptr::eq(cell, target) {
            return true;
        }
        if cell.type_0 == LAYOUT_TOPBOTTOM {
            let mut children = cell
                .cells
                .iter()
                .map(Box::as_ref)
                .filter(|child| !floating(child));
            pending.extend(if bottom {
                children.next_back()
            } else {
                children.next()
            });
        } else {
            pending.extend(cell.cells.iter().map(Box::as_ref));
        }
    }
    false
}

/// Whether `lc` gives up a row to the pane border status line.
fn layout_add_horizontal_border(
    root: Option<&layout_cell>,
    lc: &layout_cell,
    status: c_int,
) -> c_int {
    match status {
        PANE_STATUS_TOP => layout_cell_at_edge(root, lc, false) as c_int,
        PANE_STATUS_BOTTOM => layout_cell_at_edge(root, lc, true) as c_int,
        _ => 0,
    }
}

/// Finds a pane's cell and its owning parent in the current layout tree.
pub(crate) fn layout_cell_for_pane<'a>(
    root: Option<&'a layout_cell>,
    pane: &RustWindowPaneWeak,
) -> Option<(&'a layout_cell, Option<&'a layout_cell>)> {
    let mut pending = vec![(root?, None)];
    while let Some((cell, parent)) = pending.pop() {
        if cell.wp_ref.as_ref() == Some(pane) {
            return Some((cell, parent));
        }
        pending.extend(
            cell.cells
                .iter()
                .rev()
                .map(|child| (child.as_ref(), Some(cell))),
        );
    }
    None
}

/// Assigns a pane identity to an owned layout slot when present.
pub(crate) fn layout_bind_pane(
    owner: &WindowRef,
    path: Option<&LayoutCellPath>,
    pane: &RustWindowPaneWeak,
) {
    let mut payload = owner.as_window_mut();
    let w = &mut *payload;
    assert!(
        w.panes
            .iter()
            .any(|candidate| candidate.downgrade() == *pane),
        "the layout pane belongs to its window"
    );
    if let Some(path) = path {
        let cell = w
            .layout_root
            .as_deref_mut()
            .and_then(|root| path.get_mut(root))
            .expect("the layout slot exists");
        layout_cell_set_pane(cell, Some(pane.clone()));
    }
}

/// Gives every pane the size and place its cell now has, leaving `skip` alone.
struct PaneLayout {
    pane: RustWindowPaneWeak,
    x: c_int,
    y: c_int,
    sx: u_int,
    sy: u_int,
    redraw_scrollbar: bool,
}

impl PaneLayout {
    unsafe fn apply(mut self) {
        unsafe {
            if let Some(pane) = self.pane.get_mut() {
                if self.redraw_scrollbar {
                    *pane.flags_mut() |= PANE_REDRAWSCROLLBAR;
                }
                pane.set_position(self.x, self.y);
                window_pane_resize(pane, self.sx, self.sy);
            }
        }
    }
}

unsafe fn layout_pane_geometry(
    root: Option<&layout_cell>,
    scrollbar: crate::WindowScrollbarSettings,
    reference: &RustWindowPaneWeak,
    status: c_int,
) -> Option<PaneLayout> {
    unsafe {
        let pane = reference.get()?;
        let (cell, _) = layout_cell_for_pane(root, reference)?;
        let (mut x, mut y, mut sx, mut sy) = (cell.xoff, cell.yoff, cell.sx, cell.sy);
        if !floating(cell) && layout_add_horizontal_border(root, cell, status) != 0 {
            if status == PANE_STATUS_TOP {
                y += 1;
            }
            sy = sy.wrapping_sub(1);
        }
        let redraw_scrollbar = window_pane_show_scrollbar(pane, scrollbar.mode) != 0;
        if redraw_scrollbar {
            let style = pane.scrollbar_style();
            let sb_w = core::cmp::max(style.width, 1);
            let sb_pad = core::cmp::max(style.padding, 0);
            if scrollbar.position == PANE_SCROLLBARS_LEFT {
                if sx as c_int - sb_w < PANE_MINIMUM {
                    x = x + sx as c_int - PANE_MINIMUM;
                    sx = PANE_MINIMUM as u_int;
                } else {
                    sx = sx.wrapping_sub(sb_w as u_int).wrapping_sub(sb_pad as u_int);
                    x = x + sb_w + sb_pad;
                }
            } else if sx as c_int - sb_w - sb_pad < PANE_MINIMUM {
                sx = PANE_MINIMUM as u_int;
            } else {
                sx = sx.wrapping_sub(sb_w as u_int).wrapping_sub(sb_pad as u_int);
            }
        }
        Some(PaneLayout {
            pane: reference.clone(),
            x,
            y,
            sx,
            sy,
            redraw_scrollbar,
        })
    }
}

/// Restores pane geometry during final window destruction, after weak window
/// observations can no longer upgrade to a live owner.
pub(crate) unsafe fn layout_fix_panes_on_drop(
    root: Option<&layout_cell>,
    scrollbar: crate::WindowScrollbarSettings,
    panes: &[RustWindowPaneRef],
    status: c_int,
) {
    unsafe {
        for pane in panes {
            if let Some(geometry) = layout_pane_geometry(root, scrollbar, &pane.downgrade(), status)
            {
                geometry.apply();
            }
        }
    }
}

pub fn layout_count_cells(lc: &layout_cell) -> u_int {
    match lc.type_0 {
        LAYOUT_WINDOWPANE => 1,
        LAYOUT_LEFTRIGHT | LAYOUT_TOPBOTTOM => lc.cells.iter().fold(0 as u_int, |count, child| {
            count.wrapping_add(layout_count_cells(child))
        }),
        _ => unsafe { fatalx(c"bad layout type", fmt_args![]) },
    }
}

struct LayoutResizeLimits {
    minimum_width: u_int,
    minimum_height: u_int,
    children: Vec<Self>,
}

impl LayoutResizeLimits {
    fn collect(
        root: Option<&layout_cell>,
        cell: &layout_cell,
        minimum_width: u_int,
        status: c_int,
    ) -> Self {
        Self {
            minimum_width,
            minimum_height: (PANE_MINIMUM + layout_add_horizontal_border(root, cell, status))
                as u_int,
            children: cell
                .cells
                .iter()
                .map(|child| Self::collect(root, child, minimum_width, status))
                .collect(),
        }
    }

    unsafe fn for_adjustment(
        owner: &WindowRef,
        cell: &layout_cell,
        axis: layout_type,
        change: c_int,
    ) -> Self {
        fn needs_limits(cell: &layout_cell, axis: layout_type) -> bool {
            cell.cells
                .iter()
                .filter(|child| !floating(child))
                .any(|child| cell.type_0 == axis || needs_limits(child, axis))
        }
        if change < 0 && axis != LAYOUT_WINDOWPANE && needs_limits(cell, axis) {
            unsafe { Self::new(owner, cell) }
        } else {
            Self::collect(None, cell, 0, 0)
        }
    }

    unsafe fn new(owner: &WindowRef, cell: &layout_cell) -> Self {
        let payload = owner.as_window();
        let w = &*payload;
        unsafe {
            let active = w
                .panes
                .iter()
                .find(|pane| Some(pane.pane_id()) == w.active_pane_id())
                .expect("a resized layout has an active pane");
            let style = active.as_pane().scrollbar_style();
            let minimum_width = if w.scrollbar_settings().mode != PANE_SCROLLBARS_OFF {
                (PANE_MINIMUM + style.width + style.padding) as u_int
            } else {
                PANE_MINIMUM as u_int
            };
            let status = w.options_ref().number(c"pane-border-status") as c_int;
            Self::collect(w.layout_root.as_deref(), cell, minimum_width, status)
        }
    }

    fn available(&self, cell: &layout_cell, axis: layout_type) -> u_int {
        if cell.type_0 == LAYOUT_WINDOWPANE {
            if axis == LAYOUT_LEFTRIGHT {
                cell.sx.saturating_sub(self.minimum_width)
            } else {
                cell.sy.saturating_sub(self.minimum_height)
            }
        } else if cell.type_0 == axis {
            cell.cells
                .iter()
                .zip(&self.children)
                .fold(0_u32, |sum, (child, limits)| {
                    sum.wrapping_add(limits.available(child, axis))
                })
        } else {
            cell.cells
                .iter()
                .zip(&self.children)
                .map(|(child, limits)| limits.available(child, axis))
                .min()
                .unwrap_or(UINT_MAX)
        }
    }

    fn adjust(&self, cell: &mut layout_cell, axis: layout_type, mut change: c_int) {
        if axis == LAYOUT_LEFTRIGHT {
            cell.sx = cell.sx.wrapping_add(change as u_int);
        } else {
            cell.sy = cell.sy.wrapping_add(change as u_int);
        }
        if axis == LAYOUT_WINDOWPANE {
            return;
        }
        if cell.type_0 != axis {
            for (child, limits) in cell.cells.iter_mut().zip(&self.children) {
                if !floating(child) {
                    limits.adjust(child, axis, change);
                }
            }
            return;
        }
        while change != 0 {
            for (child, limits) in cell.cells.iter_mut().zip(&self.children) {
                if change == 0 {
                    break;
                }
                if floating(child) {
                    continue;
                }
                if change > 0 {
                    limits.adjust(child, axis, 1);
                    change -= 1;
                } else if limits.available(child, axis) > 0 {
                    limits.adjust(child, axis, -1);
                    change += 1;
                }
            }
        }
    }
}

struct LayoutCellRemoval {
    path: LayoutCellPath,
    resize: Option<(usize, layout_type, c_int, LayoutResizeLimits)>,
}

impl LayoutCellRemoval {
    unsafe fn new(owner: &WindowRef, root: &layout_cell, path: &LayoutCellPath) -> Option<Self> {
        let cell = path.get(root)?;
        let resize = if let Some(parent_path) = path.parent() {
            let parent = parent_path.get(root)?;
            let index = *path.0.last()?;
            let neighbor = if index == 0 { 1 } else { index - 1 };
            parent
                .cells
                .get(neighbor)
                .filter(|other| !floating(cell) && !floating(other))
                .map(|other| {
                    let axis = parent.type_0;
                    let change = if axis == LAYOUT_LEFTRIGHT {
                        cell.sx
                    } else {
                        cell.sy
                    }
                    .wrapping_add(1) as c_int;
                    let limits =
                        unsafe { LayoutResizeLimits::for_adjustment(owner, other, axis, change) };
                    (neighbor, axis, change, limits)
                })
        } else {
            None
        };
        Some(Self {
            path: path.clone(),
            resize,
        })
    }

    fn apply(
        self,
        root: &mut Option<Box<layout_cell>>,
        mut slot: Option<&mut LayoutCellPath>,
    ) -> impl IntoIterator<Item = Box<layout_cell>> {
        if let Some(slot) = slot.as_ref() {
            assert!(
                !slot.0.starts_with(&self.path.0),
                "the tracked slot survives removal"
            );
        }
        let Some(parent_path) = self.path.parent() else {
            return root.take().into_iter().collect::<Vec<_>>();
        };
        let parent = root
            .as_deref_mut()
            .and_then(|root| parent_path.get_mut(root))
            .expect("the removal parent is unchanged");
        if let Some((neighbor, axis, change, limits)) = self.resize {
            limits.adjust(&mut parent.cells[neighbor], axis, change);
        }
        let index = *self.path.0.last().expect("the removed cell has a parent");
        if let Some(slot) = slot.as_deref_mut()
            && slot.0.starts_with(&parent_path.0)
            && let Some(slot_index) = slot.0.get_mut(parent_path.0.len())
            && *slot_index > index
        {
            *slot_index -= 1;
        }
        let mut removed = vec![parent.cells.remove(index)];
        if parent.cells.len() != 1 {
            return removed;
        }
        if let Some(slot) = slot
            && slot.0.starts_with(&parent_path.0)
            && slot.0.len() > parent_path.0.len()
        {
            slot.0.remove(parent_path.0.len());
        }
        let mut only = parent.cells.pop().expect("the parent has one child");
        if let Some(grandparent_path) = parent_path.parent() {
            let grandparent = root
                .as_deref_mut()
                .and_then(|root| grandparent_path.get_mut(root))
                .expect("the grandparent is unchanged");
            only.has_parent = true;
            let index = *parent_path.0.last().expect("the parent has a grandparent");
            removed.push(core::mem::replace(&mut grandparent.cells[index], only));
        } else {
            only.has_parent = false;
            if !floating(&only) {
                only.xoff = 0;
                only.yoff = 0;
            }
            removed.extend(root.replace(only));
        }
        removed
    }
}

/// The path toward `pane` directly under its nearest split of `type_0`.
fn layout_pane_cell(
    root: Option<&layout_cell>,
    pane: &RustWindowPaneWeak,
    type_0: layout_type,
) -> Option<LayoutCellPath> {
    let root = root?;
    let mut path = LayoutCellPath::for_pane(root, pane)?;
    while let Some(parent) = path.parent() {
        if parent.get(root)?.type_0 == type_0 {
            return Some(path);
        }
        path = parent;
    }
    None
}

fn layout_calculate_pane_size(
    available: u_int,
    previous: u_int,
    lc: &layout_cell,
    type_0: layout_type,
    size: u_int,
    count_left: u_int,
    size_left: u_int,
) -> u_int {
    if count_left == 1 {
        return size_left;
    }
    let mut min = ((PANE_MINIMUM + 1) as u_int).wrapping_mul(count_left.wrapping_sub(1));
    let mut new_size;
    if type_0 == LAYOUT_LEFTRIGHT {
        if lc.sx.wrapping_sub(available) > min {
            min = lc.sx.wrapping_sub(available);
        }
        new_size = lc.sx.wrapping_mul(size).wrapping_div(previous);
    } else {
        if lc.sy.wrapping_sub(available) > min {
            min = lc.sy.wrapping_sub(available);
        }
        new_size = lc.sy.wrapping_mul(size).wrapping_div(previous);
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

fn layout_resize_child_cells_with_limits(limits: &LayoutResizeLimits, lc: &mut layout_cell) {
    if lc.type_0 == LAYOUT_WINDOWPANE {
        return;
    }

    let leftright = lc.type_0 == LAYOUT_LEFTRIGHT;
    let mut count: u_int = 0;
    let mut previous: u_int = 0;
    for lcchild in &lc.cells {
        if floating(lcchild) {
            continue;
        }
        count = count.wrapping_add(1);
        previous = previous.wrapping_add(if leftright { lcchild.sx } else { lcchild.sy });
    }
    previous = previous.wrapping_add(count.wrapping_sub(1));
    let mut available = if leftright { lc.sx } else { lc.sy };

    let mut idx: u_int = 0;
    let (parent_sx, parent_sy, parent_xoff, parent_yoff, parent_type) =
        (lc.sx, lc.sy, lc.xoff, lc.yoff, lc.type_0);
    for (lcchild, child_limits) in lc.cells.iter_mut().zip(&limits.children) {
        if floating(&*lcchild) {
            continue;
        }
        if !leftright {
            lcchild.sx = parent_sx;
            lcchild.xoff = parent_xoff;
        } else {
            lcchild.sx = layout_calculate_pane_size(
                child_limits.available(lcchild, parent_type),
                previous,
                lcchild,
                parent_type,
                parent_sx,
                count.wrapping_sub(idx),
                available,
            );
            available = available.wrapping_sub(lcchild.sx.wrapping_add(1));
        }
        if leftright {
            lcchild.sy = parent_sy;
            lcchild.yoff = parent_yoff;
        } else {
            lcchild.sy = layout_calculate_pane_size(
                child_limits.available(lcchild, parent_type),
                previous,
                lcchild,
                parent_type,
                parent_sy,
                count.wrapping_sub(idx),
                available,
            );
            available = available.wrapping_sub(lcchild.sy.wrapping_add(1));
        }
        layout_resize_child_cells_with_limits(child_limits, lcchild);
        idx = idx.wrapping_add(1);
    }
}

#[cfg(test)]
#[path = "../tests/test_layout.rs"]
mod tests;

impl WindowRef {
    /// Puts the window's panes on its z-index list in the order the tree holds
    /// them, left to right and top to bottom.
    pub fn fix_layout_zindexes(&self) {
        let owner = self;

        let mut payload = owner.as_window_mut();
        let w = &mut *payload;
        fn collect(cell: &layout_cell, order: &mut Vec<RustWindowPaneWeak>) {
            match cell.type_0 {
                LAYOUT_WINDOWPANE => order.extend(cell.wp_ref.clone()),
                LAYOUT_LEFTRIGHT | LAYOUT_TOPBOTTOM => {
                    for child in &cell.cells {
                        collect(child, order);
                    }
                }
                _ => unsafe { fatalx(c"bad layout type", fmt_args![]) },
            }
        }
        let mut order = Vec::new();
        if let Some(root) = w.layout_root.as_deref() {
            collect(root, &mut order);
        }
        w.z_index.extend(order);
    }
    pub unsafe fn fix_layout_offsets(&self) {
        let owner = self;

        unsafe {
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            let Some(root) = w.layout_root.as_deref_mut() else {
                return;
            };
            if floating(root) {
                return;
            }
            root.xoff = 0;
            root.yoff = 0;
            layout_fix_offsets1(root);
        }
    }
    pub unsafe fn fix_layout_panes(&self, skip: Option<&RustWindowPaneWeak>) {
        let owner = self;

        unsafe {
            let w = owner.as_window();
            let status = w.options_ref().number(c"pane-border-status") as c_int;
            let panes: Vec<_> = w.panes.iter().map(|pane| pane.downgrade()).collect();
            drop(w);
            for pane in panes {
                if skip == Some(&pane) {
                    continue;
                }
                let geometry = {
                    let w = owner.as_window();
                    layout_pane_geometry(
                        w.layout_root.as_deref(),
                        w.scrollbar_settings(),
                        &pane,
                        status,
                    )
                };
                if let Some(geometry) = geometry {
                    geometry.apply();
                }
            }
        }
    }
    /// How much `lc` could give up while leaving its panes their minimum size.
    unsafe fn layout_resize_check(&self, lc: &layout_cell, axis: layout_type) -> u_int {
        let owner = self;

        unsafe { LayoutResizeLimits::new(owner, lc).available(lc, axis) }
    }
    /// Removes the cell at `path` from a separately owned tree, giving its room
    /// to a neighbor and folding away a parent left with one child.
    pub unsafe fn destroy_layout_cell(
        &self,
        root: &mut Option<Box<layout_cell>>,
        path: &LayoutCellPath,
    ) {
        let owner = self;

        unsafe {
            let Some(removal) = root
                .as_deref()
                .and_then(|root| LayoutCellRemoval::new(owner, root, path))
            else {
                return;
            };
            for cell in removal.apply(root, None) {
                layout_free_cell(Some(cell));
            }
        }
    }
    pub unsafe fn init_layout(&self, pane: &RustWindowPaneWeak) {
        let owner = self;

        unsafe {
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            let size = w.dimensions().size;
            let root = w.layout_root.insert(layout_create_cell(None));
            layout_set_size(root, size.width, size.height, 0, 0);
            layout_make_leaf(root, pane.as_pane());
            drop(payload);
            owner.fix_layout_panes(None);
        }
    }
    pub fn free_layout(&self) {
        let owner = self;

        let mut payload = owner.as_window_mut();
        let w = &mut *payload;
        layout_free_cell(w.layout_root.take());
    }
    /// Resizes the window to `sx` by `sy`, shrinking only as far as its panes
    /// allow.
    pub unsafe fn resize_layout(&self, sx: u_int, sy: u_int) {
        let owner = self;

        unsafe {
            let payload = owner.as_window();
            let w = &*payload;
            let Some(root) = w.layout_root.as_deref() else {
                return;
            };
            if root.type_0 == LAYOUT_WINDOWPANE && floating(root) {
                return;
            }
            let limits = LayoutResizeLimits::new(owner, root);
            drop(payload);
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            let root = w
                .layout_root
                .as_deref_mut()
                .expect("the resized root is unchanged");
            for (axis, target) in [(LAYOUT_LEFTRIGHT, sx), (LAYOUT_TOPBOTTOM, sy)] {
                let current = if axis == LAYOUT_LEFTRIGHT {
                    root.sx
                } else {
                    root.sy
                };
                let mut change = target.wrapping_sub(current) as c_int;
                let limit = limits.available(root, axis) as c_int;
                if change < 0 && change < -limit {
                    change = -limit;
                }
                if limit == 0 {
                    change = if target <= current {
                        0
                    } else {
                        target.wrapping_sub(current) as c_int
                    };
                }
                if change != 0 {
                    limits.adjust(root, axis, change);
                }
            }
            drop(payload);
            owner.fix_layout_offsets();
            owner.fix_layout_panes(None);
        }
    }
    /// Resizes the pane to `new_size` in `type_0`'s direction by moving the border
    /// after it, or the one before it when it is the last.
    pub unsafe fn resize_pane_to(
        &self,
        pane: &RustWindowPaneWeak,
        type_0: layout_type,
        new_size: u_int,
    ) {
        let owner = self;

        unsafe {
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            let Some(path) = layout_pane_cell(w.layout_root.as_deref(), pane, type_0) else {
                return;
            };
            let root = w.layout_root.as_deref().expect("the pane path has a root");
            let cell = path.get(root).expect("the pane path was just found");
            let parent = path
                .parent()
                .and_then(|parent| parent.get(root))
                .expect("the pane path has a split parent");
            let size = if type_0 == LAYOUT_LEFTRIGHT {
                cell.sx
            } else {
                cell.sy
            };
            let change = if path.0.last() == Some(&(parent.cells.len() - 1)) {
                size.wrapping_sub(new_size) as c_int
            } else {
                new_size.wrapping_sub(size) as c_int
            };
            drop(payload);
            owner.resize_pane(pane, type_0, change, 1);
        }
    }
    /// Moves the border after `lc` by `change`, taking the room from or giving it
    /// to whichever neighbour has it. `opposite` allows taking it from the cells
    /// before `lc` when the ones after have none.
    pub unsafe fn resize_layout_cell(
        &self,
        path: &LayoutCellPath,
        type_0: layout_type,
        change: c_int,
        opposite: c_int,
    ) {
        let owner = self;

        unsafe {
            let payload = owner.as_window();
            let w = &*payload;
            if w.layout_root
                .as_deref()
                .and_then(|root| path.get(root))
                .is_none()
            {
                return;
            }
            drop(payload);
            let mut needed = change;
            while needed != 0 {
                let size = if change > 0 {
                    let size = owner.layout_resize_pane_grow(path, type_0, needed, opposite);
                    needed -= size;
                    size
                } else {
                    let size = owner.layout_resize_pane_shrink(path, type_0, needed);
                    needed += size;
                    size
                };
                if size == 0 {
                    break;
                }
            }
            owner.fix_layout_offsets();
            owner.fix_layout_panes(None);
            notify_window(c"window-layout-changed", Some(owner));
        }
    }
    pub unsafe fn resize_pane(
        &self,
        pane: &RustWindowPaneWeak,
        type_0: layout_type,
        change: c_int,
        opposite: c_int,
    ) {
        let owner = self;

        unsafe {
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            let Some(mut path) = layout_pane_cell(w.layout_root.as_deref(), pane, type_0) else {
                return;
            };
            let root = w.layout_root.as_deref().expect("the pane path has a root");
            let parent = path
                .parent()
                .and_then(|parent| parent.get(root))
                .expect("the pane path has a split parent");
            let index = path.0.last_mut().expect("the pane path is under a split");
            if *index == parent.cells.len() - 1 {
                let Some(previous) = index.checked_sub(1) else {
                    return;
                };
                *index = previous;
            }
            drop(payload);
            owner.resize_layout_cell(&path, type_0, change, opposite);
        }
    }
    pub(crate) unsafe fn layout_resize_adjust_path(
        &self,
        path: &LayoutCellPath,
        type_0: layout_type,
        change: c_int,
    ) {
        let owner = self;

        unsafe {
            let payload = owner.as_window();
            let w = &*payload;
            let Some(cell) = w.layout_root.as_deref().and_then(|root| path.get(root)) else {
                return;
            };
            let limits = LayoutResizeLimits::for_adjustment(owner, cell, type_0, change);
            drop(payload);
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            let cell = w
                .layout_root
                .as_deref_mut()
                .and_then(|root| path.get_mut(root))
                .expect("the resize path is unchanged");
            limits.adjust(cell, type_0, change);
        }
    }
    /// Grows the cell at `path`, taking room from the first sibling after it with
    /// room to give, or from the first before it when `opposite` allows that.
    unsafe fn layout_resize_pane_grow(
        &self,
        path: &LayoutCellPath,
        type_0: layout_type,
        needed: c_int,
        opposite: c_int,
    ) -> c_int {
        let owner = self;

        unsafe {
            let payload = owner.as_window();
            let w = &*payload;
            let Some(parent_path) = path.parent() else {
                return 0;
            };
            let Some(parent) = w
                .layout_root
                .as_deref()
                .and_then(|root| parent_path.get(root))
            else {
                return 0;
            };
            let index = *path.0.last().expect("the path has a parent");
            let candidates =
                (index + 1..parent.cells.len()).chain((0..index).rev().filter(|_| opposite != 0));
            let Some((source, size)) = candidates
                .filter_map(|index| {
                    let cell = parent.cells.get(index)?;
                    let available = owner.layout_resize_check(cell, type_0);
                    (available != 0).then_some((index, available.min(needed as u_int)))
                })
                .next()
            else {
                return 0;
            };
            let mut source_path = path.clone();
            *source_path.0.last_mut().unwrap() = source;
            drop(payload);
            owner.layout_resize_adjust_path(path, type_0, size as c_int);
            owner.layout_resize_adjust_path(&source_path, type_0, size.wrapping_neg() as c_int);
            size as c_int
        }
    }
    /// Shrinks the cell at `path`, or the first preceding sibling with room to
    /// give, handing that room to the cell after `path`.
    unsafe fn layout_resize_pane_shrink(
        &self,
        path: &LayoutCellPath,
        type_0: layout_type,
        needed: c_int,
    ) -> c_int {
        let owner = self;

        unsafe {
            let payload = owner.as_window();
            let w = &*payload;
            let Some(parent_path) = path.parent() else {
                return 0;
            };
            let Some(parent) = w
                .layout_root
                .as_deref()
                .and_then(|root| parent_path.get(root))
            else {
                return 0;
            };
            let index = *path.0.last().expect("the path has a parent");
            if parent.cells.get(index + 1).is_none() {
                return 0;
            }
            let Some((source, size)) = (0..=index)
                .rev()
                .filter_map(|index| {
                    let cell = parent.cells.get(index)?;
                    let available = owner.layout_resize_check(cell, type_0);
                    (available != 0).then_some((index, available.min(-needed as u_int)))
                })
                .next()
            else {
                return 0;
            };
            let mut source_path = path.clone();
            *source_path.0.last_mut().unwrap() = source;
            let mut target_path = path.clone();
            *target_path.0.last_mut().unwrap() = index + 1;
            drop(payload);
            owner.layout_resize_adjust_path(&target_path, type_0, size as c_int);
            owner.layout_resize_adjust_path(&source_path, type_0, size.wrapping_neg() as c_int);
            size as c_int
        }
    }
    /// Assigns an owned layout slot to a pane, then applies the layout geometry.
    /// `do_not_resize` preserves the assigned pane's existing size.
    pub unsafe fn assign_pane_layout(
        &self,
        path: &LayoutCellPath,
        pane: &RustWindowPaneWeak,
        do_not_resize: c_int,
    ) {
        let owner = self;

        unsafe {
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            let cell = w
                .layout_root
                .as_deref_mut()
                .and_then(|root| path.get_mut(root))
                .expect("the assigned slot is in the owned layout");
            layout_make_leaf(cell, pane.as_pane());
            drop(payload);
            owner.fix_layout_panes((do_not_resize != 0).then_some(pane));
        }
    }
    /// The size cell `lc` should take when its parent goes from `previous` to
    /// `size`, with `count_left` cells still to place in `size_left`.
    unsafe fn layout_new_pane_size(
        &self,
        previous: u_int,
        lc: &layout_cell,
        type_0: layout_type,
        size: u_int,
        count_left: u_int,
        size_left: u_int,
    ) -> u_int {
        let owner = self;

        if count_left == 1 {
            return size_left;
        }
        let available = unsafe { owner.layout_resize_check(lc, type_0) };
        layout_calculate_pane_size(available, previous, lc, type_0, size, count_left, size_left)
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
        &self,
        lc: &layout_cell,
        type_0: layout_type,
        size: c_int,
    ) -> c_int {
        let owner = self;

        unsafe {
            if lc.type_0 == LAYOUT_WINDOWPANE {
                return (size >= PANE_MINIMUM) as c_int;
            }

            let mut available = size as u_int;
            let count = lc.cells.len() as u_int;

            if lc.type_0 == type_0 {
                if available < count.wrapping_mul(2).wrapping_sub(1) {
                    return 0;
                }
                let previous = if type_0 == LAYOUT_LEFTRIGHT {
                    lc.sx
                } else {
                    lc.sy
                };
                for (idx, lcchild) in lc.cells.iter().enumerate() {
                    let idx = idx as u_int;
                    let new_size = owner.layout_new_pane_size(
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
                    if owner.layout_set_size_check(lcchild, type_0, new_size as c_int) == 0 {
                        return 0;
                    }
                }
            } else {
                for lcchild in &lc.cells {
                    if lcchild.type_0 != LAYOUT_WINDOWPANE
                        && owner.layout_set_size_check(lcchild, type_0, size) == 0
                    {
                        return 0;
                    }
                }
            }
            1
        }
    }
    /// Shares the cell at `path` among its children, in proportion to what they
    /// had before.
    unsafe fn layout_resize_child_cells(&self, path: &LayoutCellPath) {
        let owner = self;

        let payload = owner.as_window();
        let w = &*payload;
        fn needs_limits(cell: &layout_cell) -> bool {
            if cell.type_0 == LAYOUT_WINDOWPANE {
                return false;
            }
            let mut children = cell.cells.iter().filter(|child| !floating(child));
            children.clone().count() > 1 || children.any(|child| needs_limits(child))
        }
        let Some(cell) = w.layout_root.as_deref().and_then(|root| path.get(root)) else {
            return;
        };
        let limits = if needs_limits(cell) {
            unsafe { LayoutResizeLimits::new(owner, cell) }
        } else {
            LayoutResizeLimits::collect(None, cell, 0, 0)
        };
        drop(payload);
        let mut payload = owner.as_window_mut();
        let w = &mut *payload;
        let cell = w
            .layout_root
            .as_deref_mut()
            .and_then(|root| path.get_mut(root))
            .expect("resizing preserves the cell path");
        layout_resize_child_cells_with_limits(&limits, cell);
    }
    /// Splits the pane's cell in two and answers the empty half's path, or `None` when
    /// there is no room. `size` is the new half's size, or -1 for half of what
    /// there is; `SPAWN_BEFORE` puts the new half first and `SPAWN_FULLSIZE`
    /// splits the whole window rather than the one pane.
    pub unsafe fn split_pane_layout(
        &self,
        pane: &RustWindowPaneWeak,
        type_0: layout_type,
        size: c_int,
        flags: c_int,
    ) -> Option<LayoutCellPath> {
        let owner = self;

        unsafe {
            let payload = owner.as_window();
            let w = &*payload;
            let pane_index = w
                .panes
                .iter()
                .position(|candidate| candidate.downgrade() == *pane)?;
            let sb_style = w.panes[pane_index].as_pane().scrollbar_style();
            let full_size = flags & SPAWN_FULLSIZE != 0;
            let before = flags & SPAWN_BEFORE != 0;
            let root = w.layout_root.as_deref()?;
            let mut path = (if full_size {
                Some(LayoutCellPath::root())
            } else {
                LayoutCellPath::for_pane(root, pane)
            })?;
            let cell = path.get(root).expect("the split cell is in its tree");
            let status = w.options_ref().number(c"pane-border-status") as c_int;
            let (sx, sy, xoff, yoff) = (cell.sx, cell.sy, cell.xoff as u_int, cell.yoff as u_int);
            let minimum = match type_0 {
                LAYOUT_LEFTRIGHT => {
                    if w.scrollbar_settings().mode != PANE_SCROLLBARS_OFF {
                        (PANE_MINIMUM * 2 + sb_style.width + sb_style.padding) as u_int
                    } else {
                        (PANE_MINIMUM * 2 + 1) as u_int
                    }
                }
                LAYOUT_TOPBOTTOM => {
                    if layout_add_horizontal_border(w.layout_root.as_deref(), cell, status) != 0 {
                        (PANE_MINIMUM * 2 + 2) as u_int
                    } else {
                        (PANE_MINIMUM * 2 + 1) as u_int
                    }
                }
                _ => fatalx(c"bad layout type", fmt_args![]),
            };
            let saved_size = if type_0 == LAYOUT_LEFTRIGHT { sx } else { sy };
            if saved_size < minimum {
                return None;
            }
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
            if full_size && owner.layout_set_size_check(cell, type_0, new_size as c_int) == 0 {
                return None;
            }

            let parent_path = path.parent();
            let matching_parent = parent_path
                .as_ref()
                .and_then(|path| path.get(root))
                .is_some_and(|parent| parent.type_0 == type_0);
            let resize_first = full_size && parent_path.is_none() && cell.type_0 == type_0;
            drop(payload);
            let mut payload = owner.as_window_mut();
            let mut w = &mut *payload;
            let new_path;
            if matching_parent {
                let parent_path = parent_path.expect("the matching parent exists");
                let parent = parent_path
                    .get_mut(w.layout_root.as_deref_mut().unwrap())
                    .unwrap();
                let index = *path.0.last().unwrap();
                let new_index = index + usize::from(!before);
                let new = layout_create_cell(Some(parent));
                parent.cells.insert(new_index, new);
                path = parent_path.child(index + usize::from(before));
                new_path = parent_path.child(new_index);
            } else if resize_first {
                let cell = path.get_mut(w.layout_root.as_deref_mut().unwrap()).unwrap();
                if type_0 == LAYOUT_LEFTRIGHT {
                    cell.sx = new_size;
                } else {
                    cell.sy = new_size;
                }
                drop(payload);
                owner.layout_resize_child_cells(&path);
                payload = owner.as_window_mut();
                w = &mut *payload;
                let cell = path.get_mut(w.layout_root.as_deref_mut().unwrap()).unwrap();
                if type_0 == LAYOUT_LEFTRIGHT {
                    cell.sx = saved_size;
                } else {
                    cell.sy = saved_size;
                }
                let mut new = layout_create_cell(Some(cell));
                let size = saved_size.wrapping_sub(1).wrapping_sub(new_size);
                if type_0 == LAYOUT_LEFTRIGHT {
                    layout_set_size(&mut new, size, sy, 0, 0);
                } else {
                    layout_set_size(&mut new, sx, size, 0, 0);
                }
                let index = if before { 0 } else { cell.cells.len() };
                cell.cells.insert(index, new);
                new_path = path.child(index);
            } else {
                let mut node = layout_create_cell(None);
                node.type_0 = type_0;
                layout_set_size(&mut node, sx, sy, xoff as c_int, yoff as c_int);
                let mut only = if let Some(parent_path) = parent_path {
                    let parent = parent_path
                        .get_mut(w.layout_root.as_deref_mut().unwrap())
                        .unwrap();
                    node.has_parent = true;
                    let index = *path.0.last().unwrap();
                    core::mem::replace(&mut parent.cells[index], node)
                } else {
                    w.layout_root
                        .replace(node)
                        .expect("the split cell is the root")
                };
                let node = path.get_mut(w.layout_root.as_deref_mut().unwrap()).unwrap();
                only.has_parent = true;
                node.cells.push(only);
                let new = layout_create_cell(Some(node));
                node.cells.insert(usize::from(!before), new);
                new_path = path.child(usize::from(!before));
                path = path.child(usize::from(before));
            }

            if !resize_first {
                let (first, second) = if before {
                    (&new_path, &path)
                } else {
                    (&path, &new_path)
                };
                let cell = first
                    .get_mut(w.layout_root.as_deref_mut().unwrap())
                    .unwrap();
                if type_0 == LAYOUT_LEFTRIGHT {
                    layout_set_size(cell, size1, sy, xoff as c_int, yoff as c_int);
                    let cell = second
                        .get_mut(w.layout_root.as_deref_mut().unwrap())
                        .unwrap();
                    layout_set_size(
                        cell,
                        size2,
                        sy,
                        xoff.wrapping_add(size1).wrapping_add(1) as c_int,
                        yoff as c_int,
                    );
                } else {
                    layout_set_size(cell, sx, size1, xoff as c_int, yoff as c_int);
                    let cell = second
                        .get_mut(w.layout_root.as_deref_mut().unwrap())
                        .unwrap();
                    layout_set_size(
                        cell,
                        sx,
                        size2,
                        xoff as c_int,
                        yoff.wrapping_add(size1).wrapping_add(1) as c_int,
                    );
                }
            }
            if full_size {
                drop(payload);
                if !resize_first {
                    owner.layout_resize_child_cells(&path);
                }
                owner.fix_layout_offsets();
            } else {
                let cell = path.get_mut(w.layout_root.as_deref_mut().unwrap()).unwrap();
                layout_make_leaf(cell, w.panes[pane_index].as_pane_mut());
            }
            Some(new_path)
        }
    }
    /// A cell that floats over the layout at (ox, oy), under a node made for the
    /// purpose when the window is still one pane.
    pub unsafe fn float_pane_layout(
        &self,
        sx: u_int,
        sy: u_int,
        ox: c_int,
        oy: c_int,
    ) -> LayoutCellPath {
        let owner = self;

        unsafe {
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            if w.layout_root
                .as_deref()
                .expect("a floating pane needs a layout")
                .type_0
                == LAYOUT_WINDOWPANE
            {
                let mut node = layout_create_cell(None);
                layout_make_node(&mut node, LAYOUT_TOPBOTTOM);
                layout_set_size(
                    &mut node,
                    w.dimensions().size.width,
                    w.dimensions().size.height,
                    0,
                    0,
                );
                let mut only = w.layout_root.take().expect("the window has a root");
                only.has_parent = true;
                node.cells.push(only);
                w.layout_root = Some(node);
            }
            let parent = w
                .layout_root
                .as_deref_mut()
                .expect("the floating pane has a parent");
            let cell = insert_new_tail(parent);
            cell.flags |= LAYOUT_CELL_FLOATING;
            layout_set_size(cell, sx, sy, ox, oy);
            LayoutCellPath::root().child(parent.cells.len() - 1)
        }
    }
    pub unsafe fn close_pane_layout(&self, pane: &RustWindowPaneWeak) {
        let owner = self;

        unsafe { owner.layout_close_pane_with_slot(pane, None) };
    }
    /// Removes a pane's cell while updating a surviving slot through tree changes.
    pub(crate) unsafe fn layout_close_pane_with_slot(
        &self,
        pane: &RustWindowPaneWeak,
        slot: Option<&mut LayoutCellPath>,
    ) {
        let owner = self;

        unsafe {
            let payload = owner.as_window();
            let w = &*payload;
            let Some(removal) = w.layout_root.as_deref().and_then(|root| {
                let path = LayoutCellPath::for_pane(root, pane)?;
                LayoutCellRemoval::new(owner, root, &path)
            }) else {
                return;
            };
            drop(payload);
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            for cell in removal.apply(&mut w.layout_root, slot) {
                layout_free_cell(Some(cell));
            }
            let has_layout = w.layout_root.is_some();
            drop(payload);
            if has_layout {
                owner.fix_layout_offsets();
            }
            if has_layout {
                owner.fix_layout_panes(None);
            }
            notify_window(c"window-layout-changed", Some(owner));
        }
    }
    /// Gives every child at `path` the same share of its parent's size, and answers
    /// whether anything moved.
    pub unsafe fn spread_layout_cell(&self, path: &LayoutCellPath) -> c_int {
        let owner = self;

        unsafe {
            let payload = owner.as_window();
            let w = &*payload;
            let Some(parent) = w.layout_root.as_deref().and_then(|root| path.get(root)) else {
                return 0;
            };
            let number = parent.cells.len() as u_int;
            if number <= 1 {
                return 0;
            }
            let leftright = match parent.type_0 {
                LAYOUT_LEFTRIGHT => true,
                LAYOUT_TOPBOTTOM => false,
                _ => return 0,
            };
            let status = ((*w).options_ref()).number(c"pane-border-status") as c_int;

            let size = if leftright {
                parent.sx
            } else if layout_add_horizontal_border(w.layout_root.as_deref(), parent, status) != 0 {
                parent.sy.wrapping_sub(1)
            } else {
                parent.sy
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

            let axis = if leftright {
                LAYOUT_LEFTRIGHT
            } else {
                LAYOUT_TOPBOTTOM
            };
            drop(payload);
            let mut changed = 0;
            for index in 0..number as usize {
                let payload = owner.as_window();
                let w = &*payload;
                let child_path = path.child(index);
                let cell = child_path
                    .get(w.layout_root.as_deref().expect("the spread root exists"))
                    .expect("spreading preserves child positions");
                let change =
                    if leftright {
                        let mut change = each.wrapping_sub(cell.sx) as c_int;
                        if remainder > 0 {
                            change += 1;
                            remainder = remainder.wrapping_sub(1);
                        }
                        change
                    } else {
                        let mut this =
                            if layout_add_horizontal_border(w.layout_root.as_deref(), cell, status)
                                != 0
                            {
                                each.wrapping_add(1)
                            } else {
                                each
                            };
                        if remainder > 0 {
                            this = this.wrapping_add(1);
                            remainder = remainder.wrapping_sub(1);
                        }
                        this.wrapping_sub(cell.sy) as c_int
                    };
                let limits = LayoutResizeLimits::for_adjustment(owner, cell, axis, change);
                drop(payload);
                let mut payload = owner.as_window_mut();
                let w = &mut *payload;
                let cell = child_path
                    .get_mut(
                        w.layout_root
                            .as_deref_mut()
                            .expect("the spread root exists"),
                    )
                    .expect("spreading preserves child positions");
                limits.adjust(cell, axis, change);
                if change != 0 {
                    changed = 1;
                }
            }
            changed
        }
    }
    /// Shares out the nearest node above the pane that has room to share.
    pub unsafe fn spread_pane_layout(&self, pane: &RustWindowPaneWeak) {
        let owner = self;

        unsafe {
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            let mut parent = w
                .layout_root
                .as_deref()
                .and_then(|root| LayoutCellPath::for_pane(root, pane))
                .and_then(|path| path.parent());
            drop(payload);
            while let Some(path) = parent {
                if owner.spread_layout_cell(&path) != 0 {
                    owner.fix_layout_offsets();
                    owner.fix_layout_panes(None);
                    return;
                }
                parent = path.parent();
            }
        }
    }
    /// The cell a `split-window` should fill, read out of its arguments.
    pub unsafe fn tiled_layout_cell(
        &self,
        item: &cmdq_item,
        args: &RustArguments,
        pane: &RustWindowPaneWeak,
        mut flags: c_int,
        cause: &mut CString,
    ) -> Option<LayoutCellPath> {
        let owner = self;

        unsafe {
            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            *cause = CString::default();
            if layout_cell_for_pane(w.layout_root.as_deref(), pane)
                .is_some_and(|(cell, _)| floating(cell))
            {
                *cause = c"can't split a floating pane".to_owned();
                return None;
            }

            let Some(geometry) = w
                .panes
                .iter()
                .find(|candidate| candidate.downgrade() == *pane)
                .map(|pane| pane.as_pane().geometry())
            else {
                *cause = c"pane no longer exists".to_owned();
                return None;
            };
            let type_0 = if args_has(args, b'h') != 0 {
                LAYOUT_LEFTRIGHT
            } else {
                LAYOUT_TOPBOTTOM
            };

            let mut curval: u_int = 0;
            if args_has(args, b'l') != 0 || args_has(args, b'p') != 0 {
                curval = if args_has(args, b'f') != 0 {
                    if type_0 == LAYOUT_TOPBOTTOM {
                        w.dimensions().size.height
                    } else {
                        w.dimensions().size.width
                    }
                } else if type_0 == LAYOUT_TOPBOTTOM {
                    geometry.height
                } else {
                    geometry.width
                };
            }

            drop(payload);
            let mut size = -1;
            let mut parser_cause = None;
            if args_has(args, b'l') != 0 {
                size = args_percentage_and_expand(
                    args,
                    b'l',
                    0,
                    INT_MAX as core::ffi::c_longlong,
                    curval as core::ffi::c_longlong,
                    item,
                    &mut parser_cause,
                ) as c_int;
            } else if args_has(args, b'p') != 0 {
                size =
                    args_strtonum_and_expand(args, b'p', 0, 100, item, &mut parser_cause) as c_int;
                if parser_cause.is_none() {
                    size = curval.wrapping_mul(size as u_int).wrapping_div(100) as c_int;
                }
            }
            if parser_cause.is_some() {
                *cause = c"invalid tiled geometry".to_owned();
                return None;
            }

            if args_has(args, b'b') != 0 {
                flags |= SPAWN_BEFORE;
            }
            if args_has(args, b'f') != 0 {
                flags |= SPAWN_FULLSIZE;
            }

            owner.push_zoom(1, args_has(args, b'Z'));
            let lc = owner.split_pane_layout(pane, type_0, size, flags);
            if lc.is_none() {
                *cause = c"no space for a new pane".to_owned();
            }
            lc
        }
    }
    /// The floating cell a `new-pane` should fill, read out of its arguments.
    /// Without a place of its own each new pane steps four columns and two rows on
    /// from the last, starting over once that walks off the window.
    pub unsafe fn floating_layout_cell(
        &self,
        item: &cmdq_item,
        args: &RustArguments,
        cause: &mut Option<CString>,
    ) -> Option<LayoutCellPath> {
        let owner = self;

        unsafe {
            let size = owner.dimensions().size;
            let mut sx = size.width.wrapping_div(2) as c_int;
            let mut sy = size.height.wrapping_div(4) as c_int;
            let mut ox = INT_MAX;
            let mut oy = INT_MAX;

            if args_has(args, b'x') != 0 {
                sx = args_percentage_and_expand(
                    args,
                    b'x',
                    0,
                    size.width.wrapping_sub(1) as core::ffi::c_longlong,
                    size.width as core::ffi::c_longlong,
                    item,
                    cause,
                ) as c_int;
                if cause.is_some() {
                    return None;
                }
            }
            if args_has(args, b'y') != 0 {
                sy = args_percentage_and_expand(
                    args,
                    b'y',
                    0,
                    size.height.wrapping_sub(1) as core::ffi::c_longlong,
                    size.height as core::ffi::c_longlong,
                    item,
                    cause,
                ) as c_int;
                if cause.is_some() {
                    return None;
                }
            }
            if args_has(args, b'X') != 0 {
                ox = args_percentage_and_expand(
                    args,
                    b'X',
                    -sx as core::ffi::c_longlong,
                    size.width as core::ffi::c_longlong,
                    size.width as core::ffi::c_longlong,
                    item,
                    cause,
                ) as c_int;
                if cause.is_some() {
                    return None;
                }
            }
            if args_has(args, b'Y') != 0 {
                oy = args_percentage_and_expand(
                    args,
                    b'Y',
                    -sy as core::ffi::c_longlong,
                    size.height as core::ffi::c_longlong,
                    size.height as core::ffi::c_longlong,
                    item,
                    cause,
                ) as c_int;
                if cause.is_some() {
                    return None;
                }
            }

            let mut payload = owner.as_window_mut();
            let w = &mut *payload;
            if ox == INT_MAX {
                ox = if w.dimensions().last_new_pane.x == 0
                    || w.dimensions().last_new_pane.x > size.width
                {
                    4
                } else {
                    w.dimensions().last_new_pane.x.wrapping_add(4) as c_int
                };
                let mut position = w.dimensions().last_new_pane;
                position.x = ox as u_int;
                w.set_last_new_pane(position);
            }
            if oy == INT_MAX {
                oy = if w.dimensions().last_new_pane.y == 0
                    || w.dimensions().last_new_pane.y > size.height
                {
                    2
                } else {
                    w.dimensions().last_new_pane.y.wrapping_add(2) as c_int
                };
                let mut position = w.dimensions().last_new_pane;
                position.y = oy as u_int;
                w.set_last_new_pane(position);
            }

            drop(payload);
            Some(owner.float_pane_layout(sx as u_int, sy as u_int, ox, oy))
        }
    }
}

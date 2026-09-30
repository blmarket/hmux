//! Authoritative layout direction values.

#[cfg(test)]
use crate::src::window_pane::WindowPane as _;
use super::abi::u_int;
use super::pane::window_pane;
use std::{cell::UnsafeCell, rc::Weak};
pub type layout_type = ::core::ffi::c_uint;
pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
pub const LAYOUT_CELL_FLOATING: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const LAYOUT_CUSTOM_OLD_FORMAT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const LAYOUT_V1_MAX_DEPTH: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;

pub type box_lines = ::core::ffi::c_int;
pub const BOX_LINES_DEFAULT: box_lines = -1;
pub const BOX_LINES_NONE: box_lines = 6;
pub const BOX_LINES_SINGLE: box_lines = 0;

pub type pane_lines = ::core::ffi::c_uint;
pub const PANE_LINES_NONE: pane_lines = 6;
pub const PANE_LINES_ROUNDED: pane_lines = 7;
pub const PANE_LINES_SINGLE: pane_lines = 0;

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct layout_geometry {
    pub sx: u_int,
    pub sy: u_int,
    pub xoff: ::core::ffi::c_int,
    pub yoff: ::core::ffi::c_int,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::mem::{align_of, size_of};

    #[test]
    fn dropping_old_leaf_preserves_replacement_link_and_parent() {
        unsafe {
            let pane = std::rc::Rc::<std::cell::UnsafeCell<window_pane>>::allocate();
            let mut old = layout_cell::new();
            old.wp = std::rc::Rc::downgrade(&pane);
            pane.place_in_layout(old.id());
            let mut parent = layout_cell::new();
            parent.type_0 = LAYOUT_TOPBOTTOM;
            let mut replacement = layout_cell::new();
            replacement.wp = std::rc::Rc::downgrade(&pane);
            let replacement_id = replacement.id();
            pane.place_in_layout(replacement_id);
            layout_cells_push_back(&mut *parent, replacement);
            drop(old);
            assert_eq!(pane.layout_identity(false), Some(replacement_id));
            let parent_pointer = &mut *parent as *mut layout_cell;
            assert_eq!(parent.cells[0].parent, parent_pointer);
            drop(parent);
            assert_eq!(pane.layout_identity(false), None);
        }
    }

    #[test]
    fn cell_identity_survives_transfer_but_not_removal_or_replacement() {
        let mut first = layout_cell::new();
        first.type_0 = LAYOUT_TOPBOTTOM;
        let child = layout_cell::new();
        let id = child.id();
        unsafe {
            layout_cells_push_back(&mut *first, child);
        }
        assert!(first.find_mut(id).is_some());
        let mut second = layout_cell::new();
        second.type_0 = LAYOUT_TOPBOTTOM;
        let detached =
            unsafe { layout_cells_remove(&mut *first, layout_cells_first(&first)).unwrap() };
        assert!(first.find_mut(id).is_none());
        unsafe {
            layout_cells_push_back(&mut *second, detached);
        }
        assert_eq!(second.find_mut(id).unwrap().id(), id);
        let removed =
            unsafe { layout_cells_remove(&mut *second, layout_cells_first(&second)).unwrap() };
        drop(removed);
        assert!(second.find_mut(id).is_none());
        // A fresh allocation cannot make a stale ID resolve, even if the allocator
        // reuses a previous cell's address.
        for _ in 0..64 {
            let replacement = layout_cell::new();
            assert_ne!(replacement.id(), id);
            unsafe {
                layout_cells_push_back(&mut *second, replacement);
            }
        }
        assert!(second.find_mut(id).is_none());
    }

    #[test]
    fn layout_domain_matches_translated_c_baseline() {
        assert_eq!(size_of::<layout_type>(), 4);
        assert_eq!(align_of::<layout_type>(), 4);
        assert_eq!(LAYOUT_LEFTRIGHT, 0);
        assert_eq!(LAYOUT_TOPBOTTOM, 1);
        assert_eq!(LAYOUT_WINDOWPANE, 2);
        assert_eq!(size_of::<box_lines>(), 4);
        assert_eq!(align_of::<box_lines>(), 4);
        assert_eq!(size_of::<pane_lines>(), 4);
        assert_eq!(align_of::<pane_lines>(), 4);
        assert_eq!(BOX_LINES_DEFAULT, -1);
        assert_eq!(PANE_LINES_ROUNDED, 7);
    }
}

/// Nonowning identity for a cell across callbacks. It is never an address and
/// does not keep a removed cell alive. Resolve it under the owning tree borrow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutCellId(u64);

/// Cells are Box-owned by their parent, a window root, or an explicit detached owner.
/// Child addresses remain stable as the parent `Vec` moves the boxes.
pub struct layout_cell {
    id: LayoutCellId,
    pub type_0: layout_type,
    pub flags: ::core::ffi::c_int,
    pub parent: *mut layout_cell,
    /// Index in `parent.cells` for constant-time neighbor steps.
    pub sibling_index: usize,
    pub g: layout_geometry,
    pub fg: layout_geometry,
    /// Nonowning pane association; upgrade before accessing the pane.
    pub wp: Weak<UnsafeCell<window_pane>>,
    pub cells: layout_cells,
}

impl layout_cell {
    pub fn new() -> Box<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_ID
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("layout cell identity exhausted");
        let geometry = layout_geometry {
            sx: u32::MAX,
            sy: u32::MAX,
            xoff: i32::MAX,
            yoff: i32::MAX,
        };
        Box::new(Self {
            id: LayoutCellId(id),
            type_0: LAYOUT_WINDOWPANE,
            flags: 0,
            parent: std::ptr::null_mut(),
            sibling_index: 0,
            g: geometry,
            fg: geometry,
            wp: Weak::new(),
            cells: Vec::new(),
        })
    }
    pub fn id(&self) -> LayoutCellId {
        self.id
    }
    pub fn find(&self, id: LayoutCellId) -> Option<&Self> {
        if self.id == id {
            return Some(self);
        }
        self.cells.iter().find_map(|child| child.find(id))
    }
    pub fn find_mut(&mut self, id: LayoutCellId) -> Option<&mut Self> {
        if self.id == id {
            return Some(self);
        }
        self.cells.iter_mut().find_map(|child| child.find_mut(id))
    }
    pub fn find_pane_mut(&mut self, pane: &Weak<UnsafeCell<window_pane>>) -> Option<&mut Self> {
        if self.wp.ptr_eq(pane) {
            return Some(self);
        }
        self.cells
            .iter_mut()
            .find_map(|child| child.find_pane_mut(pane))
    }
}

pub type layout_cells = Vec<Box<layout_cell>>;

#[inline]
fn layout_cell_ptr(cell: &Box<layout_cell>) -> *mut layout_cell {
    &**cell as *const layout_cell as *mut layout_cell
}

#[inline]
pub fn layout_cells_first(parent: &layout_cell) -> *mut layout_cell {
    parent
        .cells
        .first()
        .map(layout_cell_ptr)
        .unwrap_or(std::ptr::null_mut())
}

#[inline]
pub fn layout_cells_last(parent: &layout_cell) -> *mut layout_cell {
    parent
        .cells
        .last()
        .map(layout_cell_ptr)
        .unwrap_or(std::ptr::null_mut())
}

#[inline]
pub unsafe fn layout_cell_next(cell: *mut layout_cell) -> *mut layout_cell {
    if cell.is_null() || (*cell).parent.is_null() {
        return std::ptr::null_mut();
    }
    let children = &(*(*cell).parent).cells;
    let index = (*cell).sibling_index.min(children.len().saturating_sub(1));
    let index = if children.get(index).map(layout_cell_ptr) == Some(cell) {
        Some(index)
    } else {
        children
            .iter()
            .position(|child| layout_cell_ptr(child) == cell)
    };
    index
        .and_then(|index| children.get(index + 1).map(layout_cell_ptr))
        .unwrap_or(std::ptr::null_mut())
}

#[inline]
pub unsafe fn layout_cell_prev(cell: *mut layout_cell) -> *mut layout_cell {
    if cell.is_null() || (*cell).parent.is_null() {
        return std::ptr::null_mut();
    }
    let children = &(*(*cell).parent).cells;
    let index = (*cell).sibling_index.min(children.len().saturating_sub(1));
    let index = if children.get(index).map(layout_cell_ptr) == Some(cell) {
        Some(index)
    } else {
        children
            .iter()
            .position(|child| layout_cell_ptr(child) == cell)
    };
    index
        .and_then(|index| {
            index
                .checked_sub(1)
                .and_then(|prev| children.get(prev).map(layout_cell_ptr))
        })
        .unwrap_or(std::ptr::null_mut())
}

#[inline]
pub unsafe fn layout_cells_remove(
    parent: *mut layout_cell,
    child: *mut layout_cell,
) -> Option<Box<layout_cell>> {
    let children = &mut (*parent).cells;
    let hinted = (*child).sibling_index;
    let index = if children.get(hinted).map(layout_cell_ptr) == Some(child) {
        Some(hinted)
    } else {
        children
            .iter()
            .position(|item| layout_cell_ptr(item) == child)
    }?;
    let mut removed = children.remove(index);
    removed.sibling_index = 0;
    removed.parent = std::ptr::null_mut();
    for (sibling_index, sibling) in children.iter_mut().enumerate().skip(index) {
        sibling.sibling_index = sibling_index;
    }
    Some(removed)
}

#[inline]
pub unsafe fn layout_cells_push_back(parent: *mut layout_cell, mut child: Box<layout_cell>) {
    child.parent = parent;
    child.sibling_index = (*parent).cells.len();
    (*parent).cells.push(child);
}

#[inline]
pub unsafe fn layout_cells_push_front(parent: *mut layout_cell, mut child: Box<layout_cell>) {
    child.parent = parent;
    (*parent).cells.insert(0, child);
    for (index, sibling) in (*parent).cells.iter_mut().enumerate() {
        sibling.sibling_index = index;
    }
}

#[inline]
pub unsafe fn layout_cells_insert_before(
    parent: *mut layout_cell,
    reference: *mut layout_cell,
    mut child: Box<layout_cell>,
) {
    child.parent = parent;
    let children = &mut (*parent).cells;
    let index = children
        .iter()
        .position(|item| layout_cell_ptr(item) == reference)
        .expect("layout cell reference must be a child of its parent");
    children.insert(index, child);
    for (sibling_index, sibling) in children.iter_mut().enumerate().skip(index) {
        sibling.sibling_index = sibling_index;
    }
}

#[inline]
pub unsafe fn layout_cells_insert_after(
    parent: *mut layout_cell,
    reference: *mut layout_cell,
    mut child: Box<layout_cell>,
) {
    child.parent = parent;
    let children = &mut (*parent).cells;
    let index = children
        .iter()
        .position(|item| layout_cell_ptr(item) == reference)
        .expect("layout cell reference must be a child of its parent");
    children.insert(index + 1, child);
    for (sibling_index, sibling) in children.iter_mut().enumerate().skip(index + 1) {
        sibling.sibling_index = sibling_index;
    }
}

#[inline]
pub unsafe fn layout_cells_replace(
    parent: *mut layout_cell,
    old: *mut layout_cell,
    mut new: Box<layout_cell>,
) -> Box<layout_cell> {
    let children = &mut (*parent).cells;
    let index = children
        .iter()
        .position(|item| layout_cell_ptr(item) == old)
        .expect("layout cell to replace must be a child of its parent");
    new.parent = parent;
    new.sibling_index = index;
    let mut detached = std::mem::replace(&mut children[index], new);
    detached.parent = std::ptr::null_mut();
    detached.sibling_index = 0;
    detached
}

impl Drop for layout_cell {
    fn drop(&mut self) {
        if self.type_0 == LAYOUT_WINDOWPANE {
            if let Some(owner) = self.wp.upgrade() {
                unsafe {
                    let pane = &mut *owner.get();
                    if pane.layout_cell == Some(self.id) {
                        pane.layout_cell = None;
                    }
                }
            }
        }
    }
}

/// Confirm that a cell has no owned children before changing its role.
///
/// The children collection owns its boxes. Callers must first remove and free
/// or move every child; silently draining the collection would leak them.
///
#[inline]
pub fn layout_cells_require_empty(parent: &layout_cell) {
    assert!(
        parent.cells.is_empty(),
        "layout cell children must be detached before clearing"
    );
}

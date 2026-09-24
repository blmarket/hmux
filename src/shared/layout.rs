//! Authoritative layout direction values.

use super::abi::u_int;
use super::pane::window_pane;
pub type layout_type = ::core::ffi::c_uint;
pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
pub const LAYOUT_CELL_FLOATING: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const LAYOUT_CUSTOM_OLD_FORMAT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const LAYOUT_V1_MAX_DEPTH: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;

pub type box_lines = ::core::ffi::c_int;
pub const BOX_LINES_DEFAULT: box_lines = -1;
pub const BOX_LINES_DOUBLE: box_lines = 1;
pub const BOX_LINES_HEAVY: box_lines = 2;
pub const BOX_LINES_NONE: box_lines = 6;
pub const BOX_LINES_PADDED: box_lines = 5;
pub const BOX_LINES_ROUNDED: box_lines = 4;
pub const BOX_LINES_SIMPLE: box_lines = 3;
pub const BOX_LINES_SINGLE: box_lines = 0;

pub type pane_lines = ::core::ffi::c_uint;
pub const PANE_LINES_DOUBLE: pane_lines = 1;
pub const PANE_LINES_HEAVY: pane_lines = 2;
pub const PANE_LINES_NONE: pane_lines = 6;
pub const PANE_LINES_NUMBER: pane_lines = 4;
pub const PANE_LINES_ROUNDED: pane_lines = 7;
pub const PANE_LINES_SIMPLE: pane_lines = 3;
pub const PANE_LINES_SINGLE: pane_lines = 0;
pub const PANE_LINES_SPACES: pane_lines = 5;

#[derive(Copy, Clone)]
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

/// Each cell is Box-owned by its parent collection until the tree is freed.
/// Child addresses remain stable as the parent `Vec` moves the boxes.
pub struct layout_cell {
    pub type_0: layout_type,
    pub flags: ::core::ffi::c_int,
    pub parent: *mut layout_cell,
    /// Index in `parent.cells.children` for constant-time neighbor steps.
    pub sibling_index: usize,
    pub g: layout_geometry,
    pub fg: layout_geometry,
    pub wp: *mut window_pane,
    pub cells: layout_cells,
}

/// Compatibility marker kept for existing module re-exports. Layout cells no
/// longer embed an intrusive TAILQ entry.
pub struct layout_cell_entry;

pub struct layout_cells {
    pub children: Vec<Box<layout_cell>>,
}

#[inline]
fn layout_cell_ptr(cell: &Box<layout_cell>) -> *mut layout_cell {
    &**cell as *const layout_cell as *mut layout_cell
}

#[inline]
pub unsafe fn layout_cells_first(parent: *mut layout_cell) -> *mut layout_cell {
    (*parent)
        .cells
        .children
        .first()
        .map(layout_cell_ptr)
        .unwrap_or(std::ptr::null_mut())
}

#[inline]
pub unsafe fn layout_cells_last(parent: *mut layout_cell) -> *mut layout_cell {
    (*parent)
        .cells
        .children
        .last()
        .map(layout_cell_ptr)
        .unwrap_or(std::ptr::null_mut())
}

#[inline]
pub unsafe fn layout_cell_next(cell: *mut layout_cell) -> *mut layout_cell {
    if cell.is_null() || (*cell).parent.is_null() {
        return std::ptr::null_mut();
    }
    let children = &(*(*cell).parent).cells.children;
    let index = (*cell).sibling_index.min(children.len().saturating_sub(1));
    let index = if children.get(index).map(layout_cell_ptr) == Some(cell) {
        Some(index)
    } else {
        children.iter().position(|child| layout_cell_ptr(child) == cell)
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
    let children = &(*(*cell).parent).cells.children;
    let index = (*cell).sibling_index.min(children.len().saturating_sub(1));
    let index = if children.get(index).map(layout_cell_ptr) == Some(cell) {
        Some(index)
    } else {
        children.iter().position(|child| layout_cell_ptr(child) == cell)
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
pub unsafe fn layout_cells_remove(parent: *mut layout_cell, child: *mut layout_cell) -> bool {
    let children = &mut (*parent).cells.children;
    let hinted = (*child).sibling_index;
    let index = if children.get(hinted).map(layout_cell_ptr) == Some(child) {
        Some(hinted)
    } else {
        children.iter().position(|item| layout_cell_ptr(item) == child)
    };
    if let Some(index) = index {
        let boxed = children.remove(index);
        let removed = Box::into_raw(boxed);
        debug_assert_eq!(removed, child);
        (*child).sibling_index = 0;
        (*child).parent = std::ptr::null_mut();
        for (sibling_index, sibling) in children.iter_mut().enumerate().skip(index) {
            sibling.sibling_index = sibling_index;
        }
        true
    } else {
        false
    }
}

#[inline]
unsafe fn layout_cells_prepare_insert(parent: *mut layout_cell, child: *mut layout_cell) {
    let old_parent = (*child).parent;
    if !old_parent.is_null() {
        layout_cells_remove(old_parent, child);
    }
    (*child).parent = parent;
}

#[inline]
pub unsafe fn layout_cells_push_back(parent: *mut layout_cell, child: *mut layout_cell) {
    layout_cells_prepare_insert(parent, child);
    (*child).sibling_index = (*parent).cells.children.len();
    (*parent).cells.children.push(Box::from_raw(child));
}

#[inline]
pub unsafe fn layout_cells_push_front(parent: *mut layout_cell, child: *mut layout_cell) {
    layout_cells_prepare_insert(parent, child);
    (*parent).cells.children.insert(0, Box::from_raw(child));
    for (index, sibling) in (*parent).cells.children.iter_mut().enumerate() {
        sibling.sibling_index = index;
    }
}

#[inline]
pub unsafe fn layout_cells_insert_before(
    parent: *mut layout_cell,
    reference: *mut layout_cell,
    child: *mut layout_cell,
) {
    layout_cells_prepare_insert(parent, child);
    let children = &mut (*parent).cells.children;
    let index = children
        .iter()
        .position(|item| layout_cell_ptr(item) == reference)
        .expect("layout cell reference must be a child of its parent");
    children.insert(index, Box::from_raw(child));
    for (sibling_index, sibling) in children.iter_mut().enumerate().skip(index) {
        sibling.sibling_index = sibling_index;
    }
}

#[inline]
pub unsafe fn layout_cells_insert_after(
    parent: *mut layout_cell,
    reference: *mut layout_cell,
    child: *mut layout_cell,
) {
    layout_cells_prepare_insert(parent, child);
    let children = &mut (*parent).cells.children;
    let index = children
        .iter()
        .position(|item| layout_cell_ptr(item) == reference)
        .expect("layout cell reference must be a child of its parent");
    children.insert(index + 1, Box::from_raw(child));
    for (sibling_index, sibling) in children.iter_mut().enumerate().skip(index + 1) {
        sibling.sibling_index = sibling_index;
    }
}

#[inline]
pub unsafe fn layout_cells_replace(
    parent: *mut layout_cell,
    old: *mut layout_cell,
    new: *mut layout_cell,
) {
    layout_cells_prepare_insert(parent, new);
    let children = &mut (*parent).cells.children;
    let index = children
        .iter()
        .position(|item| layout_cell_ptr(item) == old)
        .expect("layout cell to replace must be a child of its parent");
    let old_box = std::mem::replace(&mut children[index], Box::from_raw(new));
    let detached_old = Box::into_raw(old_box);
    debug_assert_eq!(detached_old, old);
    (*old).parent = std::ptr::null_mut();
    (*old).sibling_index = 0;
    (*new).parent = parent;
    (*new).sibling_index = index;
}

#[inline]
pub unsafe fn layout_cells_clear(parent: *mut layout_cell) {
    for child in (*parent).cells.children.drain(..) {
        let child = Box::into_raw(child);
        (*child).parent = std::ptr::null_mut();
        (*child).sibling_index = 0;
    }
}

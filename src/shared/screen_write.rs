//! Authoritative screen_write declarations, shared by the C translation units.

use super::abi::u_int;
use super::grid::{grid_cell, GridArray};
use super::pane::window_pane;
use super::screen::screen;
use super::tty::tty_ctx;
use std::collections::VecDeque;

/// Element of the screen-owned boxed write-list slice.
#[derive(Default)]
pub struct screen_write_cline {
    /// Text bytes owned by this row; scrolling moves the collection.
    pub data: GridArray<::core::ffi::c_char>,
    pub items: screen_write_items,
}
#[derive(Default)]
pub struct screen_write_citem {
    pub x: u_int,
    pub wrapped: ::core::ffi::c_int,
    pub type_0: screen_write_item_type,
    pub used: u_int,
    pub bg: u_int,
    pub gc: grid_cell,
}

/// Row-owned collection. Boxes keep item addresses stable as the deque grows,
/// inserts, removes, and moves between rows. Pointer-based lookups and middle
/// deque edits are O(n), so repeated arbitrary edits can be O(n²). Cursor-based
/// traversal avoids rescanning for each neighbor.
#[derive(Default)]
pub struct screen_write_items {
    items: VecDeque<Box<screen_write_citem>>,
}

pub type screen_write_item_type = ::core::ffi::c_uint;
pub const CLEAR: screen_write_item_type = 1;

#[derive(Default)]
pub struct screen_write_ctx {
    pub wp: *mut window_pane,
    pub s: *mut screen,
    pub flags: ::core::ffi::c_int,
    pub init_ctx_cb: screen_write_init_ctx_cb,
    pub item: Option<Box<screen_write_citem>>,
    pub scrolled: u_int,
    pub bg: u_int,
}

pub type screen_write_init_ctx_cb = Option<Box<dyn FnMut(&mut tty_ctx)>>;

impl screen_write_items {
    pub(crate) fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub(crate) fn first_ptr(&mut self) -> *mut screen_write_citem {
        self.items
            .front_mut()
            .map_or(std::ptr::null_mut(), |node| &raw mut **node)
    }
    pub(crate) fn get_ptr(&mut self, index: usize) -> *mut screen_write_citem {
        self.items
            .get_mut(index)
            .map_or(std::ptr::null_mut(), |node| &raw mut **node)
    }
    fn index_of(&self, node: *mut screen_write_citem) -> usize {
        self.items
            .iter()
            .position(|item| std::ptr::eq::<screen_write_citem>(&**item, node))
            .expect("linked write command")
    }
    pub(crate) fn push_back(&mut self, node: Box<screen_write_citem>) {
        self.items.push_back(node);
    }
    pub(crate) fn remove(&mut self, node: *mut screen_write_citem) -> Box<screen_write_citem> {
        self.items
            .remove(self.index_of(node))
            .expect("linked write command")
    }
    pub(crate) fn remove_at(&mut self, index: usize) -> Box<screen_write_citem> {
        self.items.remove(index).expect("linked write command")
    }
    pub(crate) fn insert_before(
        &mut self,
        before: *mut screen_write_citem,
        node: Box<screen_write_citem>,
    ) {
        if before.is_null() {
            self.push_back(node);
            return;
        }
        let index = self.index_of(before);
        self.insert_at(index, node);
    }
    pub(crate) fn insert_at(&mut self, index: usize, node: Box<screen_write_citem>) {
        assert!(index <= self.items.len());
        self.items.insert(index, node);
    }
    pub(crate) fn pop_front(&mut self) -> Option<Box<screen_write_citem>> {
        self.items.pop_front()
    }
    pub(crate) fn append(&mut self, other: &mut Self) {
        self.items.append(&mut other.items);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_operations_preserve_item_addresses_and_order() {
        let mut dst = screen_write_items::default();
        let mut src = screen_write_items::default();
        let make_item = || Box::new(Default::default());

        let first = make_item();
        let first_ptr = &*first as *const _ as *mut _;
        dst.push_back(first);
        let second = make_item();
        let second_ptr = &*second as *const _ as *mut _;
        dst.push_back(second);

        let inserted = make_item();
        let inserted_ptr = &*inserted as *const _ as *mut _;
        dst.insert_before(second_ptr, inserted);
        assert_eq!(dst.get_ptr(0), first_ptr);
        assert_eq!(dst.get_ptr(1), inserted_ptr);
        assert_eq!(dst.get_ptr(2), second_ptr);

        let mut src_last = std::ptr::null_mut();
        for _ in 0..64 {
            let item = make_item();
            src_last = &*item as *const _ as *mut _;
            src.push_back(item);
        }
        let src_first = src.first_ptr();
        dst.append(&mut src);
        assert!(src.is_empty());
        assert_eq!(dst.get_ptr(2), second_ptr);
        assert_eq!(dst.get_ptr(3), src_first);
        assert_eq!(dst.get_ptr(66), src_last);
        assert!(dst.get_ptr(67).is_null());
        assert_eq!(dst.get_ptr(1), inserted_ptr);
        dst.append(&mut src);
        assert_eq!(dst.first_ptr(), first_ptr);

        let tail = make_item();
        let tail_ptr = &*tail as *const _ as *mut _;
        dst.insert_before(std::ptr::null_mut(), tail);
        assert_eq!(dst.get_ptr(66), src_last);
        assert_eq!(dst.get_ptr(67), tail_ptr);

        let removed = dst.remove(inserted_ptr);
        assert_eq!(&*removed as *const _ as *mut _, inserted_ptr);
        assert_eq!(dst.get_ptr(0), first_ptr);
        assert_eq!(dst.get_ptr(1), second_ptr);
        assert_eq!(dst.first_ptr(), first_ptr);
    }
}

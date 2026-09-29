//! Authoritative screen_write declarations, shared by the C translation units.

use super::abi::u_int;
use super::grid::{GridArray, grid_cell};
use super::pane::window_pane;
use super::screen::screen;
use super::tty::tty_ctx;
use std::collections::VecDeque;

/// Element of the screen-owned boxed write-list slice.
#[derive(Default)]
pub struct screen_write_cline {
    /// Text bytes owned by this row; scrolling moves the collection.
    pub data: GridArray<u8>,
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
/// inserts, removes, and moves between rows. Indexed lookups are O(1); middle
/// deque edits are O(n), so repeated arbitrary edits can be O(n²).
#[derive(Default)]
pub struct screen_write_items {
    items: VecDeque<Box<screen_write_citem>>,
}

pub type screen_write_item_type = ::core::ffi::c_uint;
pub const CLEAR: screen_write_item_type = 1;

#[derive(Default)]
pub struct screen_write_ctx {
    /// Optional pane observer for redraw and terminal output.
    pub wp: std::rc::Weak<std::cell::UnsafeCell<window_pane>>,
    pub(crate) target: ScreenWriteTarget,
    pub flags: ::core::ffi::c_int,
    pub init_ctx_cb: screen_write_init_ctx_cb,
    pub item: Option<Box<screen_write_citem>>,
    pub scrolled: u_int,
    pub bg: u_int,
}

/// The active write target. A borrowed target is valid only between start and
/// stop; a pane base target is resolved from its owner on each access.
#[derive(Default)]
pub(crate) struct ScreenWriteTarget(ScreenWriteTargetKind);

#[derive(Default)]
enum ScreenWriteTargetKind {
    #[default]
    None,
    PaneBase(std::rc::Weak<std::cell::UnsafeCell<window_pane>>),
    Borrowed(std::ptr::NonNull<screen>),
}

impl screen_write_ctx {
    pub fn screen_ptr(&self) -> *mut screen {
        match &self.target.0 {
            ScreenWriteTargetKind::None => std::ptr::null_mut(),
            ScreenWriteTargetKind::PaneBase(observer) => {
                let owner = observer
                    .upgrade()
                    .expect("active pane base write has an owner");
                unsafe { &raw mut (*owner.get()).base }
            }
            ScreenWriteTargetKind::Borrowed(screen) => screen.as_ptr(),
        }
    }

    pub fn use_pane_base(&mut self, pane: &std::rc::Rc<std::cell::UnsafeCell<window_pane>>) {
        self.target = ScreenWriteTarget(ScreenWriteTargetKind::PaneBase(std::rc::Rc::downgrade(
            pane,
        )));
    }

    /// # Safety
    /// The screen must remain allocated until this context stops or changes target.
    pub unsafe fn borrow_screen(&mut self, screen: *mut screen) {
        self.target = ScreenWriteTarget(ScreenWriteTargetKind::Borrowed(
            std::ptr::NonNull::new(screen).expect("screen write target"),
        ));
    }
}

impl ScreenWriteTarget {
    pub(crate) const fn none() -> Self {
        Self(ScreenWriteTargetKind::None)
    }
}

pub type screen_write_init_ctx_cb = Option<Box<dyn FnMut(&mut tty_ctx)>>;

impl screen_write_items {
    pub(crate) fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub(crate) fn get(&self, index: usize) -> Option<&screen_write_citem> {
        self.items.get(index).map(Box::as_ref)
    }
    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut screen_write_citem> {
        self.items.get_mut(index).map(Box::as_mut)
    }
    pub(crate) fn push_back(&mut self, node: Box<screen_write_citem>) {
        self.items.push_back(node);
    }
    pub(crate) fn remove_at(&mut self, index: usize) -> Box<screen_write_citem> {
        self.items.remove(index).expect("linked write command")
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

    fn address(item: &screen_write_citem) -> usize {
        item as *const screen_write_citem as usize
    }

    #[test]
    fn owner_operations_preserve_item_addresses_and_order() {
        let mut dst = screen_write_items::default();
        let mut src = screen_write_items::default();
        let make_item = || Box::new(Default::default());

        let first = make_item();
        let first_ptr = address(&first);
        dst.push_back(first);
        let second = make_item();
        let second_ptr = address(&second);
        dst.push_back(second);

        let inserted = make_item();
        let inserted_ptr = address(&inserted);
        dst.insert_at(1, inserted);
        assert_eq!(address(dst.get(0).unwrap()), first_ptr);
        assert_eq!(address(dst.get(1).unwrap()), inserted_ptr);
        assert_eq!(address(dst.get(2).unwrap()), second_ptr);

        let mut src_last = 0;
        for _ in 0..64 {
            let item = make_item();
            src_last = address(&item);
            src.push_back(item);
        }
        let src_first = address(src.get(0).unwrap());
        dst.append(&mut src);
        assert!(src.is_empty());
        assert_eq!(address(dst.get(2).unwrap()), second_ptr);
        assert_eq!(address(dst.get(3).unwrap()), src_first);
        assert_eq!(address(dst.get(66).unwrap()), src_last);
        assert!(dst.get(67).is_none());
        assert_eq!(address(dst.get(1).unwrap()), inserted_ptr);
        dst.append(&mut src);
        assert_eq!(address(dst.get(0).unwrap()), first_ptr);

        let tail = make_item();
        let tail_ptr = address(&tail);
        dst.push_back(tail);
        assert_eq!(address(dst.get(66).unwrap()), src_last);
        assert_eq!(address(dst.get(67).unwrap()), tail_ptr);

        let removed = dst.remove_at(1);
        assert_eq!(address(&removed), inserted_ptr);
        assert_eq!(address(dst.get(0).unwrap()), first_ptr);
        assert_eq!(address(dst.get(1).unwrap()), second_ptr);
        assert_eq!(address(dst.get(0).unwrap()), first_ptr);
    }
}

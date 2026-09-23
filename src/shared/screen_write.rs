//! Authoritative screen_write declarations, shared by the C translation units.

use super::abi::u_int;
use super::grid::{grid_cell, GridArray};
use super::pane::window_pane;
use super::screen::screen;
use super::tty::tty_ctx;
#[repr(C)]
/// Element of the screen-owned boxed write-list slice. Each `items` tail
/// borrows the final address of its own `tqh_first` field.
pub struct screen_write_cline {
    /// Text bytes owned by this row; scrolling moves the collection.
    pub data: GridArray<::core::ffi::c_char>,
    pub items: screen_write_items,
}
#[repr(C)]
pub struct screen_write_citem {
    pub x: u_int,
    pub wrapped: ::core::ffi::c_int,
    pub type_0: screen_write_item_type,
    pub used: u_int,
    pub bg: u_int,
    pub gc: grid_cell,
    pub entry: screen_write_item_link,
}
#[repr(C)]
pub struct screen_write_items {
    pub tqh_first: Option<Box<screen_write_citem>>,
    pub tqh_last: *mut Option<Box<screen_write_citem>>,
}
#[repr(C)]
pub struct screen_write_item_link {
    pub tqe_next: Option<Box<screen_write_citem>>,
    pub tqe_prev: *mut Option<Box<screen_write_citem>>,
}
pub type screen_write_item_type = ::core::ffi::c_uint;
pub const CLEAR: screen_write_item_type = 1;
pub const TEXT: screen_write_item_type = 0;

#[repr(C)]
pub struct screen_write_ctx {
    pub wp: *mut window_pane,
    pub s: *mut screen,
    pub flags: ::core::ffi::c_int,
    pub init_ctx_cb: screen_write_init_ctx_cb,
    pub arg: *mut ::core::ffi::c_void,
    pub item: Option<Box<screen_write_citem>>,
    pub scrolled: u_int,
    pub bg: u_int,
}

pub type screen_write_init_ctx_cb =
    Option<unsafe extern "C" fn(*mut screen_write_ctx, *mut tty_ctx) -> ()>;

impl screen_write_citem {
    pub(crate) fn next_ptr(&mut self) -> *mut Self {
        self.entry
            .tqe_next
            .as_mut()
            .map_or(std::ptr::null_mut(), |node| &raw mut **node)
    }
}

// The queue owns its forward chain. Back links borrow the owning slots and
// are used only while a node is linked. Rows stay in their boxed slice.
impl screen_write_items {
    pub(crate) fn first_ptr(&mut self) -> *mut screen_write_citem {
        self.tqh_first
            .as_mut()
            .map_or(std::ptr::null_mut(), |node| &raw mut **node)
    }
    pub(crate) unsafe fn push_back(&mut self, mut node: Box<screen_write_citem>) {
        assert!(node.entry.tqe_next.is_none());
        let slot = if self.tqh_first.is_none() {
            &raw mut self.tqh_first
        } else {
            self.tqh_last
        };
        node.entry.tqe_prev = slot;
        self.tqh_last = &raw mut node.entry.tqe_next;
        *slot = Some(node);
    }
    pub(crate) unsafe fn remove(
        &mut self,
        node: *mut screen_write_citem,
    ) -> Box<screen_write_citem> {
        let slot = (*node).entry.tqe_prev;
        let mut owner = (*slot).take().expect("linked write command");
        *slot = owner.entry.tqe_next.take();
        if let Some(next) = (*slot).as_mut() {
            next.entry.tqe_prev = slot;
        } else {
            self.tqh_last = slot;
        }
        owner.entry.tqe_prev = std::ptr::null_mut();
        owner
    }
    pub(crate) unsafe fn insert_before(
        &mut self,
        before: *mut screen_write_citem,
        mut node: Box<screen_write_citem>,
    ) {
        if before.is_null() {
            self.push_back(node);
            return;
        }
        assert!(node.entry.tqe_next.is_none());
        let slot = (*before).entry.tqe_prev;
        node.entry.tqe_prev = slot;
        node.entry.tqe_next = (*slot).take();
        node.entry.tqe_next.as_mut().unwrap().entry.tqe_prev = &raw mut node.entry.tqe_next;
        *slot = Some(node);
    }
    pub(crate) unsafe fn append(&mut self, other: &mut Self) {
        let Some(mut head) = other.tqh_first.take() else {
            return;
        };
        let slot = if self.tqh_first.is_none() {
            &raw mut self.tqh_first
        } else {
            self.tqh_last
        };
        head.entry.tqe_prev = slot;
        *slot = Some(head);
        self.tqh_last = other.tqh_last;
        other.tqh_last = std::ptr::null_mut();
    }
}
impl Drop for screen_write_items {
    fn drop(&mut self) {
        // Iterative destruction avoids a recursive drop for a long command list.
        let mut next = self.tqh_first.take();
        while let Some(mut node) = next {
            next = node.entry.tqe_next.take();
        }
    }
}

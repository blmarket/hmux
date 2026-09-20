//! Authoritative screen_write declarations, shared by the C translation units.

use super::abi::u_int;
use super::grid::grid_cell;
use super::pane::window_pane;
use super::screen::screen;
use super::tty::tty_ctx;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct screen_write_cline {
    pub data: *mut ::core::ffi::c_char,
    pub items: screen_write_items,
}
#[derive(Copy, Clone)]
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct screen_write_items {
    pub tqh_first: *mut screen_write_citem,
    pub tqh_last: *mut *mut screen_write_citem,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct screen_write_item_link {
    pub tqe_next: *mut screen_write_citem,
    pub tqe_prev: *mut *mut screen_write_citem,
}
pub type screen_write_item_type = ::core::ffi::c_uint;
pub const CLEAR: screen_write_item_type = 1;
pub const TEXT: screen_write_item_type = 0;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct screen_write_ctx {
    pub wp: *mut window_pane,
    pub s: *mut screen,
    pub flags: ::core::ffi::c_int,
    pub init_ctx_cb: screen_write_init_ctx_cb,
    pub arg: *mut ::core::ffi::c_void,
    pub item: *mut screen_write_citem,
    pub scrolled: u_int,
    pub bg: u_int,
}

pub type screen_write_init_ctx_cb =
    Option<unsafe extern "C" fn(*mut screen_write_ctx, *mut tty_ctx) -> ()>;

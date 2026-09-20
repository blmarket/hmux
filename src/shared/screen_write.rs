//! Authoritative screen_write declarations, shared by the C translation units.

use super::{abi::u_int, grid::grid_cell};
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

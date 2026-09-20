//! Authoritative menu declarations, shared by the C translation units.

use super::{abi::u_int, key::key_code};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct menu_item {
    pub name: *const ::core::ffi::c_char,
    pub key: key_code,
    pub command: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct menu {
    pub title: *const ::core::ffi::c_char,
    pub items: *mut menu_item,
    pub count: u_int,
    pub width: u_int,
}
pub const MENU_NOMOUSE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MENU_STAYOPEN: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MENU_TAB: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;

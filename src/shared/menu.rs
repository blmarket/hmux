//! Authoritative menu declarations, shared by the C translation units.

use super::abi::u_int;
use super::command::cmd_find_state;
use super::grid::grid_cell;
use super::key::key_code;
use super::layout::box_lines;
use super::mouse::mouse_event;
use super::screen::screen;
use super::style::style;
use super::window::window;
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

#[repr(C)]
pub struct menu_data {
    pub w: *mut window,
    pub flags: ::core::ffi::c_int,
    pub style: *mut ::core::ffi::c_char,
    pub border_style: *mut ::core::ffi::c_char,
    pub selected_style: *mut ::core::ffi::c_char,
    pub style_gc: grid_cell,
    pub border_style_gc: grid_cell,
    pub selected_style_gc: grid_cell,
    pub border_lines: box_lines,
    pub fs: cmd_find_state,
    pub key: key_code,
    pub m: mouse_event,
    pub s: screen,
    pub px: u_int,
    pub py: u_int,
    pub menu: *mut menu,
    pub choice: ::core::ffi::c_int,
    pub cb: menu_choice_cb,
    pub data: *mut ::core::ffi::c_void,
}

pub type menu_choice_cb =
    Option<unsafe extern "C" fn(*mut menu, u_int, key_code, *mut ::core::ffi::c_void) -> ()>;

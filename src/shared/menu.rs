//! Authoritative menu declarations, shared by the C translation units.

use super::abi::u_int;
use super::command::cmd_find_state;
use super::grid::grid_cell;
use super::key::key_code;
use super::layout::box_lines;
use super::mouse::mouse_event;
use super::screen::screen;
use super::window::window;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct menu_item {
    pub name: *const ::core::ffi::c_char,
    pub key: key_code,
    pub command: *const ::core::ffi::c_char,
}
/// A runtime menu row owns its expanded text. Static definitions use `menu_item`.
#[derive(Default)]
pub struct MenuRow {
    pub name: Option<std::ffi::CString>,
    pub key: key_code,
    pub command: Option<std::ffi::CString>,
}

impl MenuRow {
    pub fn name_ptr(&self) -> *const ::core::ffi::c_char {
        self.name
            .as_ref()
            .map_or(std::ptr::null(), |name| name.as_ptr())
    }
}

#[repr(C)]
pub struct menu {
    pub title: std::ffi::CString,
    pub items: Vec<MenuRow>,
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
    pub style: Option<std::ffi::CString>,
    pub border_style: Option<std::ffi::CString>,
    pub selected_style: Option<std::ffi::CString>,
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
}

impl menu_data {
    pub fn empty() -> Self {
        Self {
            w: Default::default(),
            flags: Default::default(),
            style: Default::default(),
            border_style: Default::default(),
            selected_style: Default::default(),
            style_gc: Default::default(),
            border_style_gc: Default::default(),
            selected_style_gc: Default::default(),
            border_lines: Default::default(),
            fs: Default::default(),
            key: Default::default(),
            m: Default::default(),
            s: screen::empty(),
            px: Default::default(),
            py: Default::default(),
            menu: Default::default(),
            choice: Default::default(),
            cb: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MenuSelection {
    Selected { index: u_int, key: key_code },
    Cancelled,
}

pub type menu_choice_cb = Option<Box<dyn FnOnce(MenuSelection)>>;

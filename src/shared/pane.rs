//! Authoritative pane declarations, shared by the C translation units.

use super::abi::{size_t, u_int};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_pane_offset {
    pub used: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_pane_resizes {
    pub tqh_first: *mut window_pane_resize,
    pub tqh_last: *mut *mut window_pane_resize,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_pane_resize {
    pub sx: u_int,
    pub sy: u_int,
    pub osx: u_int,
    pub osy: u_int,
    pub entry: window_pane_resize_entry,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_pane_resize_entry {
    pub tqe_next: *mut window_pane_resize,
    pub tqe_prev: *mut *mut window_pane_resize,
}

pub const PANE_CHANGED: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const PANE_STYLECHANGED: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const PANE_THEMECHANGED: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const PANE_INPUTOFF: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const PANE_MINIMUM: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_MAXIMUM: ::core::ffi::c_int = 10000 as ::core::ffi::c_int;
pub const PANE_REDRAW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_STATUS_TOP: ::core::ffi::c_int = 1;
pub const PANE_STATUS_BOTTOM: ::core::ffi::c_int = 2;
pub const PANE_SCROLLBARS_RIGHT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_LEFT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_CLOSEONCLICK: ::core::ffi::c_int = 0x80000 as ::core::ffi::c_int;
pub const PANE_CAPTUREALLKEYS: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;
pub const PANE_CLOSEONCANCEL: ::core::ffi::c_int = 0x400000 as ::core::ffi::c_int;
pub const PANE_ZOOMED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const PANE_STATUSREADY: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const PANE_STATUSDRAWN: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const PANE_UNSEENCHANGES: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const PANE_CMDRUNNING: ::core::ffi::c_int = 0x20000 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_ALWAYS: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PANE_ACTIVITY: ::core::ffi::c_int = 0x40000 as ::core::ffi::c_int;
pub const PANE_REDRAWSCROLLBAR: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
pub const PANE_BORDER_COLOUR: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_STATUS_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_BORDER_ARROWS: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PANE_BORDER_BOTH: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const PANE_NEWSTATUS: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const PANE_DROP: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const PANE_EXITED: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_MODAL: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_AUTOHIDE: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const PANE_FLOATOVERZOOM: ::core::ffi::c_int = 0x200000 as ::core::ffi::c_int;
pub const PANE_EMPTY: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_DEFAULT_PADDING: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_DEFAULT_WIDTH: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_CHARACTER: ::core::ffi::c_int = ' ' as i32;
pub const PANE_FOCUSED: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const PANE_VISITED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const PANE_DESTROYED: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const PANE_STATUS_TOP_FLOATING: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const PANE_STATUS_BOTTOM_FLOATING: ::core::ffi::c_int = 4 as ::core::ffi::c_int;

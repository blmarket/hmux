//! Authoritative screen declarations, shared by the C translation units.

use super::{
    abi::{bitstr_t, u_int},
    display::{progress_bar, screen_cursor_style},
    grid::{grid, grid_cell},
    hyperlinks::hyperlinks,
    screen_write::screen_write_cline,
};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct screen_sel {
    pub hidden: ::core::ffi::c_int,
    pub rectangle: ::core::ffi::c_int,
    pub modekeys: ::core::ffi::c_int,
    pub sx: u_int,
    pub sy: u_int,
    pub ex: u_int,
    pub ey: u_int,
    pub clipx: u_int,
    pub cell: grid_cell,
}
#[derive(Copy, Clone)]
#[repr(C)]
/// Box-owned by the screen once the first title is pushed. An empty stack's
/// tail link points into this header, so its address stays fixed until cleanup.
pub struct screen_titles {
    pub tqh_first: *mut screen_title_entry,
    pub tqh_last: *mut *mut screen_title_entry,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct screen {
    pub title: *mut ::core::ffi::c_char,
    pub path: *mut ::core::ffi::c_char,
    pub titles: *mut screen_titles,
    pub ntitles: u_int,
    pub grid: *mut grid,
    pub cx: u_int,
    pub cy: u_int,
    pub cstyle: screen_cursor_style,
    pub default_cstyle: screen_cursor_style,
    pub ccolour: ::core::ffi::c_int,
    pub default_ccolour: ::core::ffi::c_int,
    pub rupper: u_int,
    pub rlower: u_int,
    pub mode: ::core::ffi::c_int,
    pub default_mode: ::core::ffi::c_int,
    pub saved_cx: u_int,
    pub saved_cy: u_int,
    pub saved_grid: *mut grid,
    pub saved_cell: grid_cell,
    pub saved_flags: ::core::ffi::c_int,
    pub tabs: *mut bitstr_t,
    /// Box-owned by screen_set_selection; invalid after clear or screen_free.
    pub sel: *mut screen_sel,
    pub write_list: *mut screen_write_cline,
    pub hyperlinks: *mut hyperlinks,
    pub progress_bar: progress_bar,
}
#[derive(Copy, Clone)]
#[repr(C)]
/// Box-owned while linked in a screen's title stack. Pop transfers `text` to
/// the screen; eviction and stack cleanup free it before dropping the record.
pub struct screen_title_entry {
    pub text: *mut ::core::ffi::c_char,
    pub entry: screen_title_link,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct screen_title_link {
    pub tqe_next: *mut screen_title_entry,
    pub tqe_prev: *mut *mut screen_title_entry,
}

pub const MODE_BRACKETPASTE: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const MODE_CURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MODE_INSERT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MODE_KCURSOR: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MODE_KKEYPAD: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const MODE_WRAP: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const MODE_MOUSE_STANDARD: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const MODE_MOUSE_BUTTON: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const MODE_CURSOR_BLINKING: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const MODE_MOUSE_UTF8: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const MODE_MOUSE_SGR: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const MODE_FOCUSON: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const MODE_MOUSE_ALL: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const MODE_ORIGIN: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const MODE_KEYS_EXTENDED: ::core::ffi::c_int = 32768;
pub const MODE_CURSOR_VERY_VISIBLE: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const MODE_CURSOR_BLINKING_SET: ::core::ffi::c_int = 0x20000 as ::core::ffi::c_int;
pub const MODE_KEYS_EXTENDED_2: ::core::ffi::c_int = 262144;
pub const MODE_THEME_UPDATES: ::core::ffi::c_int = 0x80000 as ::core::ffi::c_int;
pub const MODE_SYNC: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;
pub const ALL_MOUSE_MODES: ::core::ffi::c_int =
    MODE_MOUSE_STANDARD | MODE_MOUSE_BUTTON | MODE_MOUSE_ALL;
pub const EXTENDED_KEY_MODES: ::core::ffi::c_int = MODE_KEYS_EXTENDED | MODE_KEYS_EXTENDED_2;
pub const MODE_CRLF: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const ALL_MODES: ::core::ffi::c_int = 0xffffff as ::core::ffi::c_int;
pub const CURSOR_MODES: ::core::ffi::c_int =
    MODE_CURSOR | MODE_CURSOR_BLINKING | MODE_CURSOR_VERY_VISIBLE;

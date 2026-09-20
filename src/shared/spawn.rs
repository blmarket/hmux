//! Authoritative spawn declarations, shared by the C translation units.

use super::abi::{pid_t, size_t};
use super::client::client;
use super::command::cmdq_item;
use super::environment::environ;
use super::layout::layout_cell;
use super::pane::window_pane;
use super::session::session;
use super::window::winlink;
pub const SPAWN_BEFORE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const SPAWN_FULLSIZE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const SPAWN_HORIZONTAL: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const SPAWN_KILL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const SPAWN_DETACHED: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const SPAWN_EMPTY: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const SPAWN_RESPAWN: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const SPAWN_ZOOM: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const SPAWN_FLOATING: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const SPAWN_SPLIT: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const SPAWN_MODAL: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const SPAWN_FLOATOVERZOOM: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const SPAWN_NONOTIFY: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct spawn_editor_state {
    pub path: *mut ::core::ffi::c_char,
    pub pid: pid_t,
    pub cb: spawn_finish_edit_cb,
    pub arg: *mut ::core::ffi::c_void,
}

pub type spawn_finish_edit_cb =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_char, size_t, *mut ::core::ffi::c_void) -> ()>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct spawn_context {
    pub item: *mut cmdq_item,
    pub s: *mut session,
    pub wl: *mut winlink,
    pub tc: *mut client,
    pub wp0: *mut window_pane,
    pub lc: *mut layout_cell,
    pub name: *const ::core::ffi::c_char,
    pub argv: *mut *mut ::core::ffi::c_char,
    pub argc: ::core::ffi::c_int,
    pub environ: *mut environ,
    pub idx: ::core::ffi::c_int,
    pub cwd: *const ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
}

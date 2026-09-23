//! Authoritative monitor declarations, shared by the C translation units.

use super::abi::{time_t, u_int};
use super::client::client;
use super::event::event;
use super::pane::window_pane;
use super::session::session;
use super::window::{window, windows, winlink};
pub type monitor_type = ::core::ffi::c_uint;
pub const MONITOR_ALL_WINDOWS: monitor_type = 4;
pub const MONITOR_WINDOW: monitor_type = 3;
pub const MONITOR_ALL_PANES: monitor_type = 2;
pub const MONITOR_PANE: monitor_type = 1;
pub const MONITOR_SESSION: monitor_type = 0;
pub const MONITOR_NOTIFY_TRUE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MONITOR_NOTIFY_INITIAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

/// Box-owned by monitor_create; callback and owner pointers expire on destroy.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_set {
    pub client: *mut client,
    pub session: *mut session,
    pub cb: monitor_cb,
    pub data: *mut ::core::ffi::c_void,
    pub items: monitor_items,
    pub timer: event,
    pub generation: u_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_change {
    pub name: *const ::core::ffi::c_char,
    pub value: *const ::core::ffi::c_char,
    pub last: *const ::core::ffi::c_char,
    pub c: *mut client,
    pub s: *mut session,
    pub wl: *mut winlink,
    pub wp: *mut window_pane,
}

pub type monitor_cb =
    Option<unsafe extern "C" fn(*mut monitor_change, *mut ::core::ffi::c_void) -> ()>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_items {
    pub storage: *mut std::collections::BTreeMap<Vec<u8>, *mut monitor_item>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_item {
    pub name: *mut ::core::ffi::c_char,
    pub format: *mut ::core::ffi::c_char,
    pub type_0: monitor_type,
    pub id: u_int,
    pub flags: ::core::ffi::c_int,
    pub last: *mut ::core::ffi::c_char,
    pub panes: monitor_panes,
    pub windows: monitor_windows,
    pub fire_count: u_int,
    pub fire_time: time_t,
    pub entry: monitor_item_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_item_entry {
    /// Stable Rust index used by entry-only traversal; not an owning pointer.
    pub owner: *mut std::collections::BTreeMap<Vec<u8>, *mut monitor_item>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_windows {
    pub storage: *mut std::collections::BTreeMap<(u32, u32), *mut monitor_window>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_window {
    pub window: u_int,
    pub idx: u_int,
    pub last: *mut ::core::ffi::c_char,
    pub generation: u_int,
    pub entry: monitor_window_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_window_entry {
    /// Stable Rust index used by entry-only traversal; not an owning pointer.
    pub owner: *mut std::collections::BTreeMap<(u32, u32), *mut monitor_window>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_panes {
    pub storage: *mut std::collections::BTreeMap<(u32, u32), *mut monitor_pane>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_pane {
    pub pane: u_int,
    pub idx: u_int,
    pub last: *mut ::core::ffi::c_char,
    pub generation: u_int,
    pub entry: monitor_pane_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_pane_entry {
    /// Stable Rust index used by entry-only traversal; not an owning pointer.
    pub owner: *mut std::collections::BTreeMap<(u32, u32), *mut monitor_pane>,
}

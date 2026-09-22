//! Authoritative control model declarations.

use super::abi::{size_t, u_int, uint64_t};
use super::event::bufferevent;
use super::monitor::monitor_set;
use super::pane::window_pane_offset;
use super::window::{window, windows};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_state {
    pub panes: control_panes,
    pub windows: control_windows,
    pub pending_list: control_state_pending_list,
    pub pending_count: u_int,
    pub all_blocks: control_state_all_blocks,
    pub queued_reply_bytes: size_t,
    pub read_event: *mut bufferevent,
    pub write_event: *mut bufferevent,
    pub subs: *mut monitor_set,
    pub guard_depth: ::core::ffi::c_int,
    pub deferred: control_state_deferred,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_state_deferred {
    pub tqh_first: *mut control_line,
    pub tqh_last: *mut *mut control_line,
}

#[repr(C)]
pub struct control_line {
    pub line: *mut ::core::ffi::c_char,
    pub entry: control_line_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_line_entry {
    pub tqe_next: *mut control_line,
    pub tqe_prev: *mut *mut control_line,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_state_all_blocks {
    pub tqh_first: *mut control_block,
    pub tqh_last: *mut *mut control_block,
}

#[repr(C)]
pub struct control_block {
    pub size: size_t,
    pub line: *mut ::core::ffi::c_char,
    pub t: uint64_t,
    pub entry: control_block_entry,
    pub all_entry: control_block_all_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_block_all_entry {
    pub tqe_next: *mut control_block,
    pub tqe_prev: *mut *mut control_block,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_block_entry {
    pub tqe_next: *mut control_block,
    pub tqe_prev: *mut *mut control_block,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_state_pending_list {
    pub tqh_first: *mut control_pane,
    pub tqh_last: *mut *mut control_pane,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_pane {
    pub pane: u_int,
    pub offset: window_pane_offset,
    pub queued: window_pane_offset,
    pub flags: ::core::ffi::c_int,
    pub pending_flag: ::core::ffi::c_int,
    pub pending_entry: control_pane_pending_entry,
    pub blocks: control_pane_blocks,
    pub entry: control_pane_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_pane_entry {
    /// Stable Rust index used by entry-only traversal; not an owning pointer.
    pub owner: *mut std::collections::BTreeMap<u32, *mut control_pane>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_pane_blocks {
    pub tqh_first: *mut control_block,
    pub tqh_last: *mut *mut control_block,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_pane_pending_entry {
    pub tqe_next: *mut control_pane,
    pub tqe_prev: *mut *mut control_pane,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_windows {
    pub storage: *mut std::collections::BTreeMap<u32, *mut control_window>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_window {
    pub window: u_int,
    pub sx: u_int,
    pub sy: u_int,
    pub entry: control_window_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_window_entry {
    /// Stable Rust index used by entry-only traversal; not an owning pointer.
    pub owner: *mut std::collections::BTreeMap<u32, *mut control_window>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_panes {
    pub storage: *mut std::collections::BTreeMap<u32, *mut control_pane>,
}

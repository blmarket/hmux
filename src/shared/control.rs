//! Authoritative control model declarations.

use super::abi::{size_t, u_int, uint64_t};
use super::event::bufferevent;
use super::monitor::monitor_set;
use super::pane::window_pane_offset;
use std::collections::VecDeque;

#[repr(C)]
pub struct control_state {
    pub panes: control_panes,
    pub windows: control_windows,
    pub pending_count: u_int,
    pub queued_reply_bytes: size_t,
    pub read_event: *mut bufferevent,
    pub write_event: *mut bufferevent,
    pub subs: *mut monitor_set,
    pub guard_depth: ::core::ffi::c_int,
    pub(crate) deferred: VecDeque<std::ffi::CString>,
    pub(crate) all_blocks: VecDeque<Box<control_block>>,
    pub(crate) pending_panes: VecDeque<*mut control_pane>,
}

impl control_state {
    pub fn empty() -> Self {
        Self {
            panes: unsafe { ::core::mem::zeroed() },
            windows: unsafe { ::core::mem::zeroed() },
            pending_count: unsafe { ::core::mem::zeroed() },
            queued_reply_bytes: unsafe { ::core::mem::zeroed() },
            read_event: unsafe { ::core::mem::zeroed() },
            write_event: unsafe { ::core::mem::zeroed() },
            subs: unsafe { ::core::mem::zeroed() },
            guard_depth: unsafe { ::core::mem::zeroed() },
            deferred: Default::default(),
            all_blocks: Default::default(),
            pending_panes: Default::default(),
        }
    }
}

#[repr(C)]
pub struct control_block {
    pub size: size_t,
    pub line: Option<std::ffi::CString>,
    pub t: uint64_t,
}

impl control_block {
    pub fn empty() -> Self {
        Self {
            size: unsafe { ::core::mem::zeroed() },
            line: Default::default(),
            t: unsafe { ::core::mem::zeroed() },
        }
    }
}

#[repr(C)]
/// Owned by the pane-ID index until `control_reset_offsets` removes it.
/// The pending queue borrows its stable address.
pub struct control_pane {
    pub pane: u_int,
    pub offset: window_pane_offset,
    pub queued: window_pane_offset,
    pub flags: ::core::ffi::c_int,
    pub pending_flag: ::core::ffi::c_int,
    /// Ordered non-owning handles into the state-owned block collection.
    pub blocks: VecDeque<*mut control_block>,
    pub entry: control_pane_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_pane_entry {
    /// Stable Rust index used by entry-only traversal; not an owning pointer.
    pub owner: *mut std::collections::BTreeMap<u32, Box<control_pane>>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_windows {
    pub storage: *mut std::collections::BTreeMap<u32, *mut control_window>,
}

#[derive(Copy, Clone)]
#[repr(C)]
/// Box-owned by the control client; the window-ID index borrows its address.
/// Unlink before `control_clear_window_size` or `control_stop` consumes it.
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
    /// Points to a map that owns each stable pane box.
    pub storage: *mut std::collections::BTreeMap<u32, Box<control_pane>>,
}

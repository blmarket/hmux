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
            panes: control_panes { storage: None },
            windows: control_windows { storage: None },
            pending_count: Default::default(),
            queued_reply_bytes: Default::default(),
            read_event: Default::default(),
            write_event: Default::default(),
            subs: Default::default(),
            guard_depth: Default::default(),
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

#[repr(C)]
pub struct control_pane_entry {
    /// Weak traversal handle into the pane ID index.
    pub owner: Option<refbox::Weak<std::collections::BTreeMap<u32, Box<control_pane>>>>,
}

#[repr(C)]
pub struct control_windows {
    pub storage: Option<Box<std::collections::BTreeMap<u32, Box<control_window>>>>,
}

#[repr(C)]
/// Owned by the control client's window-ID index.
pub struct control_window {
    pub window: u_int,
    pub sx: u_int,
    pub sy: u_int,
}

#[repr(C)]
pub struct control_panes {
    /// The index owns each stable pane box.
    pub storage: Option<refbox::RefBox<std::collections::BTreeMap<u32, Box<control_pane>>>>,
}

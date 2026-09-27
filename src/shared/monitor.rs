//! Authoritative monitor declarations, shared by the C translation units.

use super::abi::{time_t, u_int};
use super::client::client;
use super::event::event;
use super::pane::window_pane;
use super::session::session;
use super::window::winlink;
pub type monitor_type = ::core::ffi::c_uint;
pub const MONITOR_ALL_WINDOWS: monitor_type = 4;
pub const MONITOR_WINDOW: monitor_type = 3;
pub const MONITOR_ALL_PANES: monitor_type = 2;
pub const MONITOR_PANE: monitor_type = 1;
pub const MONITOR_SESSION: monitor_type = 0;
pub const MONITOR_NOTIFY_TRUE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MONITOR_NOTIFY_INITIAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

/// Box-owned by monitor_create; its callback and weak observers expire on destroy.
#[repr(C)]
pub struct monitor_set {
    pub client: *mut client,
    pub session: *mut session,
    pub cb: monitor_cb,
    pub items: monitor_items,
    pub timer: event,
    pub generation: u_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_change<'a> {
    pub name: &'a std::ffi::CStr,
    pub value: &'a std::ffi::CStr,
    pub last: Option<&'a std::ffi::CStr>,
    pub c: *mut client,
    pub s: *mut session,
    pub wl: *mut winlink,
    pub wp: *mut window_pane,
}

pub type monitor_cb = std::rc::Rc<dyn Fn(&monitor_change)>;

pub fn monitor_callback(callback: impl Fn(&monitor_change) + 'static) -> monitor_cb {
    std::rc::Rc::new(callback)
}

#[repr(C)]
pub struct monitor_items {
    pub storage: Option<refbox::RefBox<std::collections::BTreeMap<Vec<u8>, Box<monitor_item>>>>,
}

#[repr(C)]
pub struct monitor_item {
    pub name: std::ffi::CString,
    pub format: std::ffi::CString,
    pub type_0: monitor_type,
    pub id: u_int,
    pub flags: ::core::ffi::c_int,
    pub last: Option<std::ffi::CString>,
    pub panes: monitor_panes,
    pub windows: monitor_windows,
    pub fire_count: u_int,
    pub fire_time: time_t,
    pub entry: monitor_item_entry,
}

impl monitor_item {
    pub fn empty() -> Self {
        Self {
            name: Default::default(),
            format: Default::default(),
            type_0: Default::default(),
            id: Default::default(),
            flags: Default::default(),
            last: Default::default(),
            panes: monitor_panes { storage: None },
            windows: monitor_windows { storage: None },
            fire_count: Default::default(),
            fire_time: Default::default(),
            entry: monitor_item_entry { owner: None },
        }
    }
}

#[repr(C)]
pub struct monitor_item_entry {
    /// Weak traversal handle into the monitor item index.
    pub owner: Option<refbox::Weak<std::collections::BTreeMap<Vec<u8>, Box<monitor_item>>>>,
}

#[repr(C)]
pub struct monitor_windows {
    pub storage:
        Option<refbox::RefBox<std::collections::BTreeMap<(u32, u32), Box<monitor_window>>>>,
}

#[repr(C)]
pub struct monitor_window {
    pub window: u_int,
    pub idx: u_int,
    pub last: Option<std::ffi::CString>,
    pub generation: u_int,
    pub entry: monitor_window_entry,
}

impl monitor_window {
    pub fn empty() -> Self {
        Self {
            window: Default::default(),
            idx: Default::default(),
            last: Default::default(),
            generation: Default::default(),
            entry: monitor_window_entry { owner: None },
        }
    }
}

#[repr(C)]
pub struct monitor_window_entry {
    /// Weak traversal handle into the monitor window index.
    pub owner: Option<refbox::Weak<std::collections::BTreeMap<(u32, u32), Box<monitor_window>>>>,
}

#[repr(C)]
pub struct monitor_panes {
    pub storage: Option<refbox::RefBox<std::collections::BTreeMap<(u32, u32), Box<monitor_pane>>>>,
}

#[repr(C)]
pub struct monitor_pane {
    pub pane: u_int,
    pub idx: u_int,
    pub last: Option<std::ffi::CString>,
    pub generation: u_int,
    pub entry: monitor_pane_entry,
}

impl monitor_pane {
    pub fn empty() -> Self {
        Self {
            pane: Default::default(),
            idx: Default::default(),
            last: Default::default(),
            generation: Default::default(),
            entry: monitor_pane_entry { owner: None },
        }
    }
}

#[repr(C)]
pub struct monitor_pane_entry {
    /// Weak traversal handle into the monitor pane index.
    pub owner: Option<refbox::Weak<std::collections::BTreeMap<(u32, u32), Box<monitor_pane>>>>,
}

/// Sole monitor-set owner. Detach the allocation before destroying callbacks,
/// which can reenter the enclosing client or option.
pub struct MonitorOwner(pub(crate) Option<Box<monitor_set>>);

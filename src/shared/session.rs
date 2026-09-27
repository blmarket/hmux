//! Authoritative session model declarations.

use super::abi::{timeval, u_int};
use super::environment::environ;
use super::event::event;
use super::options::options;
use super::terminal::termios;
use super::window::{winlink, winlink_stack, winlinks};

#[repr(C)]
pub struct session {
    pub id: u_int,
    pub name: std::ffi::CString,
    pub cwd: Option<std::ffi::CString>,
    pub creation_time: timeval,
    pub last_attached_time: timeval,
    pub activity_time: timeval,
    pub last_activity_time: timeval,
    pub lock_timer: event,
    pub curw: *mut winlink,
    pub lastw: winlink_stack,
    pub windows: winlinks,
    pub statusat: ::core::ffi::c_int,
    pub statuslines: u_int,
    pub options: *mut options,
    pub flags: ::core::ffi::c_int,
    pub attached: u_int,
    pub tio: Option<Box<termios>>,
    pub environ: Option<Box<environ>>,
    pub entry: session_entry,
}

impl session {
    pub fn empty() -> Self {
        Self {
            id: Default::default(),
            name: Default::default(),
            cwd: Default::default(),
            creation_time: Default::default(),
            last_attached_time: Default::default(),
            activity_time: Default::default(),
            last_activity_time: Default::default(),
            lock_timer: Default::default(),
            curw: Default::default(),
            lastw: winlink_stack {
                storage: None,
                reserved: std::ptr::null_mut(),
            },
            windows: winlinks { storage: None },
            statusat: Default::default(),
            statuslines: Default::default(),
            options: Default::default(),
            flags: Default::default(),
            attached: Default::default(),
            tio: Default::default(),
            environ: Default::default(),
            entry: session_entry { owner: None },
        }
    }
}

#[repr(C)]
pub struct session_entry {
    /// Weak traversal handle into the session index.
    pub owner: Option<refbox::Weak<std::collections::BTreeMap<Vec<u8>, *mut session>>>,
}

#[repr(C)]
pub struct sessions {
    pub storage: Option<refbox::RefBox<std::collections::BTreeMap<Vec<u8>, *mut session>>>,
}

#[repr(C)]
pub struct session_group {
    pub name: std::ffi::CString,
    pub entry: session_group_entry,
    pub(crate) members: Vec<*mut session>,
}

impl session_group {
    pub fn empty() -> Self {
        Self {
            name: Default::default(),
            entry: session_group_entry { owner: None },
            members: Default::default(),
        }
    }
}

#[repr(C)]
pub struct session_group_entry {
    /// Weak traversal handle into the session group index.
    pub owner: Option<refbox::Weak<std::collections::BTreeMap<Vec<u8>, Box<session_group>>>>,
}

#[repr(C)]
pub struct session_groups {
    pub storage: Option<refbox::RefBox<std::collections::BTreeMap<Vec<u8>, Box<session_group>>>>,
}

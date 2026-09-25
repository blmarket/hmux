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
    pub environ: *mut environ,
    pub references: ::core::ffi::c_int,
    pub entry: session_entry,
}

impl session {
    pub fn empty() -> Self {
        Self {
            id: unsafe { ::core::mem::zeroed() },
            name: Default::default(),
            cwd: Default::default(),
            creation_time: unsafe { ::core::mem::zeroed() },
            last_attached_time: unsafe { ::core::mem::zeroed() },
            activity_time: unsafe { ::core::mem::zeroed() },
            last_activity_time: unsafe { ::core::mem::zeroed() },
            lock_timer: unsafe { ::core::mem::zeroed() },
            curw: unsafe { ::core::mem::zeroed() },
            lastw: unsafe { ::core::mem::zeroed() },
            windows: unsafe { ::core::mem::zeroed() },
            statusat: unsafe { ::core::mem::zeroed() },
            statuslines: unsafe { ::core::mem::zeroed() },
            options: unsafe { ::core::mem::zeroed() },
            flags: unsafe { ::core::mem::zeroed() },
            attached: unsafe { ::core::mem::zeroed() },
            tio: Default::default(),
            environ: unsafe { ::core::mem::zeroed() },
            references: unsafe { ::core::mem::zeroed() },
            entry: unsafe { ::core::mem::zeroed() },
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct session_entry {
    /// Stable Rust index used by entry-only traversal; not an owning pointer.
    pub owner: *mut std::collections::BTreeMap<Vec<u8>, *mut session>,
}

#[repr(C)]
pub struct sessions {
    pub storage: Option<Box<std::collections::BTreeMap<Vec<u8>, *mut session>>>,
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
            entry: unsafe { ::core::mem::zeroed() },
            members: Default::default(),
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct session_group_entry {
    /// Stable Rust index used by entry-only traversal; not an owning pointer.
    pub owner: *mut std::collections::BTreeMap<Vec<u8>, Box<session_group>>,
}

#[repr(C)]
pub struct session_groups {
    pub storage: Option<Box<std::collections::BTreeMap<Vec<u8>, Box<session_group>>>>,
}

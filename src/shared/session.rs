//! Authoritative session model declarations.

use super::abi::{timeval, u_int};
use super::environment::environ;
use super::event::event;
use super::options::options;
use super::terminal::termios;
use super::window::{windows, winlink, winlink_stack, winlinks};

#[repr(C)]
pub struct session {
    pub id: u_int,
    pub name: *mut ::core::ffi::c_char,
    pub cwd: *const ::core::ffi::c_char,
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
    pub tio: *mut termios,
    pub environ: *mut environ,
    pub references: ::core::ffi::c_int,
    pub gentry: session_gentry,
    pub entry: session_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct session_entry {
    /// Stable Rust index used by entry-only traversal; not an owning pointer.
    pub owner: *mut std::collections::BTreeMap<Vec<u8>, *mut session>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct session_gentry {
    pub tqe_next: *mut session,
    pub tqe_prev: *mut *mut session,
}

#[repr(C)]
pub struct sessions {
    pub storage: Option<Box<std::collections::BTreeMap<Vec<u8>, *mut session>>>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct session_group {
    pub name: *const ::core::ffi::c_char,
    pub sessions: session_group_sessions,
    pub entry: session_group_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct session_group_entry {
    /// Stable Rust index used by entry-only traversal; not an owning pointer.
    pub owner: *mut std::collections::BTreeMap<Vec<u8>, Box<SessionGroupOwner>>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct session_group_sessions {
    pub tqh_first: *mut session,
    pub tqh_last: *mut *mut session,
}

#[repr(C)]
pub struct session_groups {
    pub storage: Option<Box<std::collections::BTreeMap<Vec<u8>, Box<SessionGroupOwner>>>>,
}

/// Owns a stable C-compatible group node and its byte-preserving name. The
/// session-groups index is the sole owner; `session_group` pointers are borrowed.
pub struct SessionGroupOwner {
    pub(crate) node: session_group,
    pub(crate) name: std::ffi::CString,
}

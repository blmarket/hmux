//! Authoritative process model declarations.

use super::abi::{gid_t, uid_t};
use super::event::event;
use crate::src::compat::imsg::{imsg, imsgbuf};

pub enum PeerMessage<'a> {
    Disconnected,
    Message(&'a mut imsg),
}

/// Owned by the parent process's peer list until `proc_remove_peer`.
pub struct tmuxpeer {
    pub parent: *mut tmuxproc,
    pub ibuf: imsgbuf,
    pub event: event,
    pub uid: uid_t,
    pub gid: gid_t,
    pub flags: ::core::ffi::c_int,
    pub dispatchcb: Option<Box<dyn for<'a> FnMut(PeerMessage<'a>)>>,
}

impl Default for tmuxpeer {
    fn default() -> Self {
        Self {
            parent: std::ptr::null_mut(),
            ibuf: imsgbuf::default(),
            event: event::default(),
            uid: 0,
            gid: 0,
            flags: 0,
            dispatchcb: None,
        }
    }
}

#[repr(C)]
pub struct tmuxproc {
    pub name: std::ffi::CString,
    pub exit: ::core::ffi::c_int,
    pub signalcb: Option<Box<dyn FnMut(super::signal::ProcessSignal)>>,
    pub ev_sigint: event,
    pub ev_sighup: event,
    pub ev_sigchld: event,
    pub ev_sigcont: event,
    pub ev_sigterm: event,
    pub ev_sigusr1: event,
    pub ev_sigusr2: event,
    pub ev_sigwinch: event,
    /// Owns stable peer allocations until `proc_remove_peer` removes them.
    pub peers: Vec<Box<tmuxpeer>>,
}

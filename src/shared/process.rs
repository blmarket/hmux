//! Authoritative process model declarations.

use super::abi::{gid_t, uid_t};
use super::event::event;
use super::message::{ibuf, imsg, imsgbuf};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tmuxpeer {
    pub parent: *mut tmuxproc,
    pub ibuf: imsgbuf,
    pub event: event,
    pub uid: uid_t,
    pub gid: gid_t,
    pub flags: ::core::ffi::c_int,
    pub dispatchcb: Option<unsafe extern "C" fn(*mut imsg, *mut ::core::ffi::c_void) -> ()>,
    pub arg: *mut ::core::ffi::c_void,
    pub entry: tmuxpeer_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tmuxproc {
    pub name: *const ::core::ffi::c_char,
    pub exit: ::core::ffi::c_int,
    pub signalcb: Option<unsafe extern "C" fn(::core::ffi::c_int) -> ()>,
    pub ev_sigint: event,
    pub ev_sighup: event,
    pub ev_sigchld: event,
    pub ev_sigcont: event,
    pub ev_sigterm: event,
    pub ev_sigusr1: event,
    pub ev_sigusr2: event,
    pub ev_sigwinch: event,
    pub peers: tmuxproc_peers,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tmuxpeer_entry {
    pub tqe_next: *mut tmuxpeer,
    pub tqe_prev: *mut *mut tmuxpeer,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tmuxproc_peers {
    pub tqh_first: *mut tmuxpeer,
    pub tqh_last: *mut *mut tmuxpeer,
}

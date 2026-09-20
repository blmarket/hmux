//! Authoritative job declarations.

use super::abi::pid_t;
use super::command::cmd;
use super::event::{bufferevent, event};
use super::tty::tty;
pub const JOB_NOWAIT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

pub const JOB_SHOWSTDERR: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;

pub const JOB_KEEPWRITE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;

pub const JOB_PTY: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;

pub const JOB_DEFAULTSHELL: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct job {
    pub state: job_state,
    pub flags: ::core::ffi::c_int,
    pub cmd: *mut ::core::ffi::c_char,
    pub pid: pid_t,
    pub tty: [::core::ffi::c_char; 32],
    pub status: ::core::ffi::c_int,
    pub fd: ::core::ffi::c_int,
    pub event: *mut bufferevent,
    pub updatecb: job_update_cb,
    pub completecb: job_complete_cb,
    pub freecb: job_free_cb,
    pub data: *mut ::core::ffi::c_void,
    pub entry: job_entry,
}

pub type job_update_cb = Option<unsafe extern "C" fn(*mut job) -> ()>;

pub type job_complete_cb = Option<unsafe extern "C" fn(*mut job) -> ()>;

pub type job_free_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct job_entry {
    pub le_next: *mut job,
    pub le_prev: *mut *mut job,
}

pub type job_state = ::core::ffi::c_uint;

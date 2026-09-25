//! Application callback handles for the Rust reactor.
//!
//! Scheduling and registration ownership live in the reactor's Rust collections.
//! Embedded handles own no resources and remain valid when zero initialized by
//! translated callers; their layout is no longer tied to libevent.
use super::abi::*;
pub use crate::src::reactor::{bufferevent_ops, evbuffer, event_base};

pub type EventCallback = Option<
    unsafe extern "C" fn(::core::ffi::c_int, ::core::ffi::c_short, *mut ::core::ffi::c_void),
>;

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct event {
    pub(crate) initialized: bool,
    pub(crate) fd: ::core::ffi::c_int,
    pub(crate) flags: ::core::ffi::c_short,
    pub(crate) callback: EventCallback,
    pub(crate) arg: *mut ::core::ffi::c_void,
}

impl event {
    /// Const initialization for embedded events in static owners.
    pub const fn new() -> Self {
        Self {
            initialized: false,
            fd: 0,
            flags: 0,
            callback: None,
            arg: ::core::ptr::null_mut(),
        }
    }
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct bufferevent {
    pub ev_base: *mut event_base,
    pub be_ops: *const bufferevent_ops,
    pub ev_read: event,
    pub ev_write: event,
    pub input: *mut evbuffer,
    pub output: *mut evbuffer,
    pub wm_read: event_watermark,
    pub wm_write: event_watermark,
    pub readcb: bufferevent_data_cb,
    pub writecb: bufferevent_data_cb,
    pub errorcb: bufferevent_event_cb,
    pub cbarg: *mut ::core::ffi::c_void,
    pub timeout_read: timeval,
    pub timeout_write: timeval,
    pub enabled: ::core::ffi::c_short,
}

pub type bufferevent_event_cb = Option<
    unsafe extern "C" fn(*mut bufferevent, ::core::ffi::c_short, *mut ::core::ffi::c_void) -> (),
>;
pub type bufferevent_data_cb =
    Option<unsafe extern "C" fn(*mut bufferevent, *mut ::core::ffi::c_void) -> ()>;

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct event_watermark {
    pub low: size_t,
    pub high: size_t,
}

pub const EV_TIMEOUT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const EV_READ: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const EV_WRITE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const EV_SIGNAL: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const EV_PERSIST: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;

pub type evbuffer_eol_style = ::core::ffi::c_uint;
pub const EVBUFFER_EOL_NUL: evbuffer_eol_style = 4;
pub const EVBUFFER_EOL_LF: evbuffer_eol_style = 3;
pub const EVBUFFER_EOL_CRLF_STRICT: evbuffer_eol_style = 2;
pub const EVBUFFER_EOL_CRLF: evbuffer_eol_style = 1;
pub const EVBUFFER_EOL_ANY: evbuffer_eol_style = 0;

//! Application callback handles for the Rust reactor.
//!
//! Scheduling and registration ownership live in the reactor's Rust collections.
//! Event handles can be zero initialized by translated callers. Streams own
//! their buffers; neither layout is tied to libevent.
use super::abi::*;
pub use crate::src::reactor::{bufferevent_ops, evbuffer, event_base};

pub type EventCallback = Option<
    std::rc::Rc<std::cell::RefCell<Box<dyn FnMut(::core::ffi::c_int, ::core::ffi::c_short)>>>,
>;

#[derive(Default)]
#[repr(C)]
pub struct event {
    pub(crate) initialized: bool,
    pub(crate) fd: ::core::ffi::c_int,
    pub(crate) flags: ::core::ffi::c_short,
    pub(crate) callback: EventCallback,
}

impl event {
    /// Const initialization for embedded events in static owners.
    pub const fn new() -> Self {
        Self {
            initialized: false,
            fd: 0,
            flags: 0,
            callback: None,
        }
    }
}

#[derive(Default)]
#[repr(C)]
pub struct bufferevent {
    pub ev_read: event,
    pub ev_write: event,
    // Keep buffer addresses stable for reactor registrations and borrowed pointers.
    pub input: Box<evbuffer>,
    pub output: Box<evbuffer>,
    pub wm_read: event_watermark,
    pub wm_write: event_watermark,
    pub readcb: bufferevent_data_cb,
    pub writecb: bufferevent_data_cb,
    pub errorcb: bufferevent_event_cb,
    pub timeout_read: timeval,
    pub timeout_write: timeval,
    pub enabled: ::core::ffi::c_short,
}

pub type bufferevent_event_cb = Option<
    std::rc::Rc<
        std::cell::RefCell<Box<dyn FnMut(std::ptr::NonNull<bufferevent>, ::core::ffi::c_short)>>,
    >,
>;
pub type bufferevent_data_cb =
    Option<std::rc::Rc<std::cell::RefCell<Box<dyn FnMut(std::ptr::NonNull<bufferevent>)>>>>;

pub fn bufferevent_data_callback(
    callback: impl FnMut(std::ptr::NonNull<bufferevent>) + 'static,
) -> bufferevent_data_cb {
    Some(std::rc::Rc::new(std::cell::RefCell::new(Box::new(
        callback,
    ))))
}

pub fn bufferevent_event_callback(
    callback: impl FnMut(std::ptr::NonNull<bufferevent>, ::core::ffi::c_short) + 'static,
) -> bufferevent_event_cb {
    Some(std::rc::Rc::new(std::cell::RefCell::new(Box::new(
        callback,
    ))))
}

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

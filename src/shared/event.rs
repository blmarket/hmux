//! Authoritative libevent/bufferevent ABI declarations.
//!
//! The generated units gave each anonymous C union/queue member a local
//! `C2RustUnnamed_*` name.  These names are not identities: their suffixes
//! vary with the declarations present in a translation unit.  The names
//! below describe the C subjects and are backed by layout tests.

use super::abi::*;

extern "C" {
    pub type event_base;
    pub type evbuffer;
    pub type bufferevent_ops;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct event {
    pub ev_evcallback: event_callback,
    pub ev_timeout_pos: event_timeout_pos,
    pub ev_fd: ::core::ffi::c_int,
    pub ev_base: *mut event_base,
    pub ev_: event_io_or_signal,
    pub ev_events: ::core::ffi::c_short,
    pub ev_res: ::core::ffi::c_short,
    pub ev_timeout: timeval,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union event_io_or_signal {
    pub ev_io: event_io,
    pub ev_signal: event_signal,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_signal {
    pub ev_signal_next: event_signal_entry,
    pub ev_ncalls: ::core::ffi::c_short,
    pub ev_pncalls: *mut ::core::ffi::c_short,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_signal_entry {
    pub le_next: *mut event,
    pub le_prev: *mut *mut event,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_io {
    pub ev_io_next: event_io_entry,
    pub ev_timeout: timeval,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_io_entry {
    pub le_next: *mut event,
    pub le_prev: *mut *mut event,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union event_timeout_pos {
    pub ev_next_with_common_timeout: event_timeout_entry,
    pub min_heap_idx: ::core::ffi::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_timeout_entry {
    pub tqe_next: *mut event,
    pub tqe_prev: *mut *mut event,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_callback {
    pub evcb_active_next: event_callback_entry,
    pub evcb_flags: ::core::ffi::c_short,
    pub evcb_pri: uint8_t,
    pub evcb_closure: uint8_t,
    pub evcb_cb_union: event_callback_union,
    pub evcb_arg: *mut ::core::ffi::c_void,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union event_callback_union {
    pub evcb_callback: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            ::core::ffi::c_short,
            *mut ::core::ffi::c_void,
        ) -> (),
    >,
    pub evcb_selfcb:
        Option<unsafe extern "C" fn(*mut event_callback, *mut ::core::ffi::c_void) -> ()>,
    pub evcb_evfinalize: Option<unsafe extern "C" fn(*mut event, *mut ::core::ffi::c_void) -> ()>,
    pub evcb_cbfinalize:
        Option<unsafe extern "C" fn(*mut event_callback, *mut ::core::ffi::c_void) -> ()>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_callback_entry {
    pub tqe_next: *mut event_callback,
    pub tqe_prev: *mut *mut event_callback,
}

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_watermark {
    pub low: size_t,
    pub high: size_t,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::core::mem::{align_of, offset_of, size_of};

    #[test]
    fn event_layout_matches_translated_c_baseline() {
        assert_eq!(size_of::<event_callback_entry>(), 16);
        assert_eq!(align_of::<event_callback_entry>(), 8);
        assert_eq!(size_of::<event_callback_union>(), 8);
        assert_eq!(size_of::<event_callback>(), 40);
        assert_eq!(offset_of!(event_callback, evcb_cb_union), 24);
        assert_eq!(offset_of!(event_callback, evcb_arg), 32);

        assert_eq!(size_of::<event_signal_entry>(), 16);
        assert_eq!(size_of::<event_io>(), 32);
        assert_eq!(size_of::<event_io_or_signal>(), 32);
        assert_eq!(size_of::<event_timeout_pos>(), 16);
        assert_eq!(size_of::<event>(), 128);
        assert_eq!(align_of::<event>(), 8);
        assert_eq!(offset_of!(event, ev_timeout_pos), 40);
        assert_eq!(offset_of!(event, ev_base), 64);
        assert_eq!(offset_of!(event, ev_), 72);
        assert_eq!(offset_of!(event, ev_timeout), 112);

        assert_eq!(size_of::<event_watermark>(), 16);
        assert_eq!(size_of::<bufferevent>(), 392);
        assert_eq!(align_of::<bufferevent>(), 8);
        assert_eq!(offset_of!(bufferevent, ev_read), 16);
        assert_eq!(offset_of!(bufferevent, wm_read), 288);
        assert_eq!(offset_of!(bufferevent, readcb), 320);
        assert_eq!(offset_of!(bufferevent, timeout_read), 352);
        assert_eq!(offset_of!(bufferevent, enabled), 384);
    }
}

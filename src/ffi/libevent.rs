//! Foreign declarations supplied by libevent.

use crate::src::shared::abi::{size_t, ssize_t, timeval};
use crate::src::shared::event::{
    bufferevent, bufferevent_data_cb, bufferevent_event_cb, evbuffer_eol_style, event,
};
pub type event_log_cb =
    Option<unsafe extern "C" fn(::core::ffi::c_int, *const ::core::ffi::c_char) -> ()>;

extern "C" {
    pub fn bufferevent_disable(
        bufev: *mut bufferevent,
        event: ::core::ffi::c_short,
    ) -> ::core::ffi::c_int;
    pub fn bufferevent_enable(
        bufev: *mut bufferevent,
        event: ::core::ffi::c_short,
    ) -> ::core::ffi::c_int;
    pub fn bufferevent_free(bufev: *mut bufferevent);
    pub fn bufferevent_get_output(bufev: *mut bufferevent) -> *mut evbuffer;
    pub fn bufferevent_new(
        fd: ::core::ffi::c_int,
        readcb: bufferevent_data_cb,
        writecb: bufferevent_data_cb,
        errorcb: bufferevent_event_cb,
        cbarg: *mut ::core::ffi::c_void,
    ) -> *mut bufferevent;
    pub type bufferevent_ops;
    pub fn bufferevent_setwatermark(
        bufev: *mut bufferevent,
        events: ::core::ffi::c_short,
        lowmark: size_t,
        highmark: size_t,
    );
    pub fn bufferevent_write(
        bufev: *mut bufferevent,
        data: *const ::core::ffi::c_void,
        size: size_t,
    ) -> ::core::ffi::c_int;
    pub fn bufferevent_write_buffer(
        bufev: *mut bufferevent,
        buf: *mut evbuffer,
    ) -> ::core::ffi::c_int;
    pub type evbuffer;
    pub fn evbuffer_add(
        buf: *mut evbuffer,
        data: *const ::core::ffi::c_void,
        datlen: size_t,
    ) -> ::core::ffi::c_int;
    pub fn evbuffer_add_printf(
        buf: *mut evbuffer,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    pub fn evbuffer_add_vprintf(
        buf: *mut evbuffer,
        fmt: *const ::core::ffi::c_char,
        ap: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    pub fn evbuffer_drain(buf: *mut evbuffer, len: size_t) -> ::core::ffi::c_int;
    pub fn evbuffer_free(buf: *mut evbuffer);
    pub fn evbuffer_get_length(buf: *const evbuffer) -> size_t;
    pub fn evbuffer_new() -> *mut evbuffer;
    pub fn evbuffer_pullup(buf: *mut evbuffer, size: ssize_t) -> *mut ::core::ffi::c_uchar;
    pub fn evbuffer_read(
        buffer: *mut evbuffer,
        fd: ::core::ffi::c_int,
        howmuch: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    pub fn evbuffer_readline(buffer: *mut evbuffer) -> *mut ::core::ffi::c_char;
    pub fn evbuffer_readln(
        buffer: *mut evbuffer,
        n_read_out: *mut size_t,
        eol_style: evbuffer_eol_style,
    ) -> *mut ::core::ffi::c_char;
    pub fn evbuffer_write(buffer: *mut evbuffer, fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn event_active(ev: *mut event, res: ::core::ffi::c_int, ncalls: ::core::ffi::c_short);
    pub fn event_add(ev: *mut event, timeout: *const timeval) -> ::core::ffi::c_int;
    pub type event_base;
    pub fn event_del(_: *mut event) -> ::core::ffi::c_int;
    pub fn event_get_method() -> *const ::core::ffi::c_char;
    pub fn event_get_version() -> *const ::core::ffi::c_char;
    pub fn event_init() -> *mut event_base;
    pub fn event_initialized(ev: *const event) -> ::core::ffi::c_int;
    pub fn event_loop(_: ::core::ffi::c_int) -> ::core::ffi::c_int;
    pub fn event_once(
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_short,
        _: Option<
            unsafe extern "C" fn(
                ::core::ffi::c_int,
                ::core::ffi::c_short,
                *mut ::core::ffi::c_void,
            ) -> (),
        >,
        _: *mut ::core::ffi::c_void,
        _: *const timeval,
    ) -> ::core::ffi::c_int;
    pub fn event_pending(
        ev: *const event,
        events: ::core::ffi::c_short,
        tv: *mut timeval,
    ) -> ::core::ffi::c_int;
    pub fn event_reinit(base: *mut event_base) -> ::core::ffi::c_int;
    pub fn event_set(
        _: *mut event,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_short,
        _: Option<
            unsafe extern "C" fn(
                ::core::ffi::c_int,
                ::core::ffi::c_short,
                *mut ::core::ffi::c_void,
            ) -> (),
        >,
        _: *mut ::core::ffi::c_void,
    );
    pub fn event_set_log_callback(cb: event_log_cb);
}

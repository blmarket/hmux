//! Authoritative input declarations.

use super::abi::{size_t, u_char, u_int, uint64_t};
use super::client::client;
use super::colour::colour_palette;
use super::event::{bufferevent, evbuffer, event};
use super::grid::{grid_cell, utf8_data};
use super::pane::window_pane;
use super::screen_write::screen_write_ctx;
pub const INPUT_REQUEST_QUEUE: input_request_type = 2;

pub const INPUT_REQUEST_CLIPBOARD: input_request_type = 1;

pub const INPUT_REQUEST_PALETTE: input_request_type = 0;

pub const INPUT_BUF_DEFAULT_SIZE: ::core::ffi::c_int = 1048576 as ::core::ffi::c_int;

pub type input_request_type = ::core::ffi::c_uint;

#[repr(C)]
pub struct input_ctx {
    pub wp: *mut window_pane,
    pub event: *mut bufferevent,
    pub ctx: screen_write_ctx,
    pub palette: *mut colour_palette,
    pub c: *mut client,
    pub cell: input_cell,
    pub old_cell: input_cell,
    pub old_cx: u_int,
    pub old_cy: u_int,
    pub old_mode: ::core::ffi::c_int,
    pub interm_buf: [u_char; 4],
    pub interm_len: size_t,
    pub param_buf: [u_char; 64],
    pub param_len: size_t,
    pub input_buf: *mut u_char,
    pub input_len: size_t,
    pub input_space: size_t,
    pub input_end: input_end_type,
    pub param_list: [input_param; 24],
    pub param_list_len: u_int,
    pub utf8data: utf8_data,
    pub utf8started: ::core::ffi::c_int,
    pub ch: ::core::ffi::c_int,
    pub last: utf8_data,
    pub state: *const input_state,
    pub flags: ::core::ffi::c_int,
    // Opaque handle to the request collection owned by InputCtxOwner. The
    // second pointer preserves the translated C record's size and alignment.
    pub requests: input_requests,
    pub request_count: u_int,
    pub request_timer: event,
    pub since_ground: *mut evbuffer,
    pub ground_timer: event,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_request {
    pub c: *mut client,
    pub ictx: *mut input_ctx,
    pub type_0: input_request_type,
    pub t: uint64_t,
    pub end: input_end_type,
    pub idx: ::core::ffi::c_int,
    pub data: *mut ::core::ffi::c_void,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_requests {
    /// Points at the collection held by the input or client owner.
    pub collection: *mut ::core::ffi::c_void,
    /// Retained to preserve the translated record's size and alignment.
    pub reserved: *mut ::core::ffi::c_void,
}

pub type input_end_type = ::core::ffi::c_uint;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_state {
    pub name: *const ::core::ffi::c_char,
    pub enter: Option<unsafe extern "C" fn(*mut input_ctx) -> ()>,
    pub exit: Option<unsafe extern "C" fn(*mut input_ctx) -> ()>,
    pub transitions: *const input_transition,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_transition {
    pub first: ::core::ffi::c_int,
    pub last: ::core::ffi::c_int,
    pub handler: Option<unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int>,
    pub state: *const input_state,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_param {
    pub type_0: input_param_type_0,
    pub c2rust_unnamed: input_param_c2rust_unnamed,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union input_param_c2rust_unnamed {
    pub num: ::core::ffi::c_int,
    pub str_0: *mut ::core::ffi::c_char,
}

pub type input_param_type_0 = ::core::ffi::c_uint;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_cell {
    pub cell: grid_cell,
    pub set: ::core::ffi::c_int,
    pub g0set: ::core::ffi::c_int,
    pub g1set: ::core::ffi::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_request_palette_data {
    pub idx: ::core::ffi::c_int,
    pub c: ::core::ffi::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_request_clipboard_data {
    pub buf: *mut ::core::ffi::c_char,
    pub len: size_t,
    pub clip: ::core::ffi::c_char,
}

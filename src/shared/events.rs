//! Authoritative events model declarations.

use super::abi::{time_t, u_int};
use super::client::client;
use super::command::cmd_find_state;
use super::event::evbuffer;
use super::pane::window_pane;
use super::session::session;
use super::window::window;
use std::collections::BTreeMap;

#[derive(Copy, Clone)]
#[repr(C)]
/// Box-owned by its creator until `event_payload_free`. `events_fire` lends
/// this stable address to callbacks, then frees items and target references.
pub struct event_payload {
    pub items: event_payload_tree,
    pub target: cmd_find_state,
}

pub type event_payload_free_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;

pub type event_payload_print_cb =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut evbuffer) -> ()>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_payload_item {
    pub name: *mut ::core::ffi::c_char,
    pub type_0: event_payload_type,
    pub c2rust_unnamed: event_payload_item_c2rust_unnamed,
    pub entry: event_payload_item_entry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct events_sink {
    pub name: *mut ::core::ffi::c_char,
    pub cb: events_cb,
    pub data: *mut ::core::ffi::c_void,
    pub dead: ::core::ffi::c_int,
    pub generation: u_int,
    pub entry: events_sink_entry,
}

pub type events_cb = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_char,
        *mut event_payload,
        *mut ::core::ffi::c_void,
    ) -> (),
>;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct events_sink_entry {
    pub tqe_next: *mut events_sink,
    pub tqe_prev: *mut *mut events_sink,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_payload_tree {
    pub entries: *mut event_payload_tree_storage,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_payload_item_entry {
    pub rbe_left: *mut event_payload_item,
    pub rbe_right: *mut event_payload_item,
    pub rbe_parent: *mut event_payload_item,
    pub rbe_color: ::core::ffi::c_int,
}

/// Rust-owned ordering storage for an event payload's C-allocated items.
/// Keys retain the original C string bytes so ordering matches `strcmp`.
#[derive(Default)]
pub struct event_payload_tree_storage {
    pub(crate) entries: BTreeMap<Vec<u8>, *mut event_payload_item>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union event_payload_item_c2rust_unnamed {
    pub string: *mut ::core::ffi::c_char,
    pub time: time_t,
    pub number: ::core::ffi::c_int,
    pub unsigned_number: u_int,
    pub client: *mut client,
    pub session: *mut session,
    pub window: *mut window,
    pub pane: *mut window_pane,
    pub pointer: event_payload_item_c2rust_unnamed_pointer,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_payload_item_c2rust_unnamed_pointer {
    pub ptr: *mut ::core::ffi::c_void,
    pub free_cb: event_payload_free_cb,
    pub print_cb: event_payload_print_cb,
}

pub type event_payload_type = ::core::ffi::c_uint;

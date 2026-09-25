//! Authoritative events model declarations.

use super::abi::{time_t, u_int};
use super::client::client;
use super::command::cmd_find_state;
use super::event::evbuffer;
use super::pane::window_pane;
use super::session::session;
use super::window::window;
use std::collections::BTreeMap;

/// Box-owned by its creator until `event_payload_free`. `events_fire` lends
/// this stable address to callbacks, then frees items and target references.
pub struct event_payload {
    pub items: event_payload_tree,
    pub target: cmd_find_state,
}

pub type event_payload_free_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;

pub type event_payload_print_cb =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut evbuffer) -> ()>;

pub struct event_payload_item {
    pub name: Option<std::ffi::CString>,
    pub type_0: event_payload_type,
    pub c2rust_unnamed: event_payload_item_c2rust_unnamed,
    /// Weak traversal handle into the payload's ordered item map.
    pub(crate) owner: Option<refbox::Weak<event_payload_tree_storage>>,
    // The Copy payload union also stores borrowed object pointers and callbacks.
    // Its string variant borrows this allocation until this item is freed.
    pub(crate) string: Option<std::ffi::CString>,
}

impl event_payload_item {
    pub fn empty() -> Self {
        Self {
            name: Default::default(),
            type_0: unsafe { ::core::mem::zeroed() },
            c2rust_unnamed: unsafe { ::core::mem::zeroed() },
            owner: None,
            string: Default::default(),
        }
    }
}

pub struct events_sink {
    pub name: std::ffi::CString,
    pub cb: events_cb,
    pub data: *mut ::core::ffi::c_void,
    pub dead: ::core::ffi::c_int,
    pub generation: u_int,
}

impl events_sink {
    pub fn empty() -> Self {
        Self {
            name: Default::default(),
            cb: unsafe { ::core::mem::zeroed() },
            data: unsafe { ::core::mem::zeroed() },
            dead: unsafe { ::core::mem::zeroed() },
            generation: unsafe { ::core::mem::zeroed() },
        }
    }
}

pub type events_cb = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_char,
        *mut event_payload,
        *mut ::core::ffi::c_void,
    ) -> (),
>;

pub struct event_payload_tree {
    pub entries: refbox::RefBox<event_payload_tree_storage>,
}

impl Default for event_payload_tree {
    fn default() -> Self {
        Self {
            entries: refbox::RefBox::default(),
        }
    }
}

/// Rust-owned ordering storage for an event payload's Box-owned items.
/// Keys retain the original C string bytes so ordering matches `strcmp`.
#[derive(Default)]
pub struct event_payload_tree_storage {
    pub(crate) entries: BTreeMap<Vec<u8>, *mut event_payload_item>,
}

#[derive(Copy, Clone)]
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
pub struct event_payload_item_c2rust_unnamed_pointer {
    pub ptr: *mut ::core::ffi::c_void,
    pub free_cb: event_payload_free_cb,
    pub print_cb: event_payload_print_cb,
}

pub type event_payload_type = ::core::ffi::c_uint;

//! Authoritative events model declarations.

use super::abi::{time_t, u_int};
use super::client::client;
use super::command::cmd_find_state;
use super::event::evbuffer;
use super::pane::window_pane;
use super::session::session;
use super::window::window;
use std::collections::BTreeMap;

#[repr(C)]
/// Box-owned by its creator until `event_payload_free`. `events_fire` lends
/// this stable address to callbacks, then frees items and target references.
pub struct event_payload {
    pub items: event_payload_tree,
    pub target: cmd_find_state,
}

pub type event_payload_free_cb = Option<Box<dyn FnOnce()>>;

pub type event_payload_print_cb = Option<Box<dyn FnMut() -> Vec<u8>>>;

pub struct event_payload_item {
    pub name: Option<std::ffi::CString>,
    pub type_0: event_payload_type,
    pub c2rust_unnamed: event_payload_item_c2rust_unnamed,
    /// Weak traversal handle into the payload's ordered item map.
    pub(crate) owner: Option<refbox::Weak<event_payload_tree_storage>>,
    // The payload union's string variant borrows this allocation until freed.
    pub(crate) string: Option<std::ffi::CString>,
}

impl event_payload_item {
    pub fn empty() -> Self {
        Self {
            name: Default::default(),
            type_0: Default::default(),
            c2rust_unnamed: Default::default(),
            owner: None,
            string: Default::default(),
        }
    }
}

pub struct events_sink {
    pub name: std::ffi::CString,
    pub cb: events_cb,
    pub dead: ::core::ffi::c_int,
    pub generation: u_int,
}

impl events_sink {
    pub fn empty() -> Self {
        Self {
            name: Default::default(),
            cb: events_callback(|_, _| {}),
            dead: Default::default(),
            generation: Default::default(),
        }
    }
}

pub type events_cb = std::rc::Rc<dyn Fn(&std::ffi::CStr, &mut event_payload)>;

pub fn events_callback(
    callback: impl Fn(&std::ffi::CStr, &mut event_payload) + 'static,
) -> events_cb {
    std::rc::Rc::new(callback)
}

#[repr(C)]
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

pub union event_payload_item_c2rust_unnamed {
    pub string: *mut ::core::ffi::c_char,
    pub time: time_t,
    pub number: ::core::ffi::c_int,
    pub unsigned_number: u_int,
    pub client: *mut client,
    pub session: *mut session,
    pub window: *mut window,
    pub pane: *mut window_pane,
    pub pointer: std::mem::ManuallyDrop<event_payload_item_c2rust_unnamed_pointer>,
}

impl Default for event_payload_item_c2rust_unnamed {
    fn default() -> Self {
        Self { number: 0 }
    }
}

pub enum event_payload_item_c2rust_unnamed_pointer {
    Raw(*mut ::core::ffi::c_void),
    Owned {
        ptr: *mut ::core::ffi::c_void,
        free_cb: event_payload_free_cb,
        print_cb: event_payload_print_cb,
    },
}

impl event_payload_item_c2rust_unnamed_pointer {
    pub fn ptr(&self) -> *mut ::core::ffi::c_void {
        match self {
            Self::Raw(ptr) | Self::Owned { ptr, .. } => *ptr,
        }
    }

    pub fn print(&mut self) -> Option<Vec<u8>> {
        match self {
            Self::Raw(_) => None,
            Self::Owned { print_cb, .. } => print_cb.as_mut().map(|callback| callback()),
        }
    }

    pub fn free(self) {
        if let Self::Owned {
            mut free_cb, ..
        } = self
        {
            if let Some(callback) = free_cb.take() {
                callback();
            }
        }
    }
}

pub type event_payload_type = ::core::ffi::c_uint;

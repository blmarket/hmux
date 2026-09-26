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

pub struct event_payload_item {
    pub name: Option<std::ffi::CString>,
    pub value: EventPayloadValue,
    /// Weak traversal handle into the payload's ordered item map.
    pub(crate) owner: Option<refbox::Weak<event_payload_tree_storage>>,
}

impl event_payload_item {
    pub fn type_0(&self) -> event_payload_type {
        self.value.kind()
    }
    pub fn empty() -> Self {
        Self {
            name: Default::default(),
            value: EventPayloadValue::String(Default::default()),
            owner: None,
        }
    }
}

pub struct events_sink {
    pub name: std::ffi::CString,
    pub cb: events_cb,
    pub dead: ::core::ffi::c_int,
    pub generation: u_int,
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

pub enum EventPayloadValue {
    String(std::ffi::CString),
    Time(time_t),
    Int(::core::ffi::c_int),
    Uint(u_int),
    Client(*mut client),
    Session(*mut session),
    Window(*mut window),
    Pane(*mut window_pane),
    Pointer(EventPayloadPointer),
}
impl EventPayloadValue {
    pub fn kind(&self) -> event_payload_type {
        match self {
            Self::String(_) => 0,
            Self::Time(_) => 1,
            Self::Int(_) => 2,
            Self::Uint(_) => 3,
            Self::Client(_) => 4,
            Self::Session(_) => 5,
            Self::Window(_) => 6,
            Self::Pane(_) => 7,
            Self::Pointer(_) => 8,
        }
    }
    pub fn string(&self) -> *const ::core::ffi::c_char {
        let Self::String(value) = self else {
            panic!("incorrect event payload type")
        };
        value.as_ptr()
    }
    pub fn time(&self) -> time_t {
        let Self::Time(value) = self else {
            panic!("incorrect event payload type")
        };
        *value
    }
    pub fn number(&self) -> ::core::ffi::c_int {
        let Self::Int(value) = self else {
            panic!("incorrect event payload type")
        };
        *value
    }
    pub fn unsigned_number(&self) -> u_int {
        let Self::Uint(value) = self else {
            panic!("incorrect event payload type")
        };
        *value
    }
    pub fn client(&self) -> *mut client {
        let Self::Client(value) = self else {
            panic!("incorrect event payload type")
        };
        *value
    }
    pub fn session(&self) -> *mut session {
        let Self::Session(value) = self else {
            panic!("incorrect event payload type")
        };
        *value
    }
    pub fn window(&self) -> *mut window {
        let Self::Window(value) = self else {
            panic!("incorrect event payload type")
        };
        *value
    }
    pub fn pane(&self) -> *mut window_pane {
        let Self::Pane(value) = self else {
            panic!("incorrect event payload type")
        };
        *value
    }
    pub fn pointer(&self) -> &EventPayloadPointer {
        let Self::Pointer(value) = self else {
            panic!("incorrect event payload type")
        };
        value
    }
}

pub enum EventPayloadPointer {
    Raw(*mut ::core::ffi::c_void),
}

impl EventPayloadPointer {
    pub fn ptr(&self) -> *mut ::core::ffi::c_void {
        match self {
            Self::Raw(ptr) => *ptr,
        }
    }

}

pub type event_payload_type = ::core::ffi::c_uint;

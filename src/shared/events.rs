//! Authoritative events model declarations.

use super::abi::{time_t, u_int};
use super::client::client;
use super::command::cmd_find_state;
use super::event::evbuffer;
use super::pane::window_pane;
use super::session::session;
use super::window::window;
use std::collections::BTreeMap;

/// Box-owned until `events_fire` consumes it. Sinks borrow the payload during
/// dispatch; Drop releases items in key order before the target references.
pub struct event_payload {
    pub items: event_payload_tree,
    pub target: cmd_find_state,
    pub target_pane: Option<std::rc::Rc<std::cell::UnsafeCell<window_pane>>>,
    pub target_window: Option<std::rc::Rc<std::cell::UnsafeCell<window>>>,
    pub target_session: Option<std::rc::Rc<std::cell::UnsafeCell<session>>>,
}

pub struct event_payload_item {
    pub name: std::ffi::CString,
    pub value: EventPayloadValue,
}

impl event_payload_item {
    pub fn type_0(&self) -> event_payload_type {
        self.value.kind()
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
    pub entries: Box<event_payload_tree_storage>,
}

impl Default for event_payload_tree {
    fn default() -> Self {
        Self {
            entries: Box::default(),
        }
    }
}

/// Rust-owned ordering storage for an event payload's Box-owned items.
/// Keys retain the original C string bytes so ordering matches `strcmp`.
#[derive(Default)]
pub struct event_payload_tree_storage {
    pub(crate) entries: BTreeMap<Vec<u8>, Box<event_payload_item>>,
}

pub enum EventPayloadValue {
    String(std::ffi::CString),
    Time(time_t),
    Int(::core::ffi::c_int),
    Uint(u_int),
    Client(std::rc::Rc<std::cell::UnsafeCell<client>>),
    Session(std::rc::Rc<std::cell::UnsafeCell<session>>),
    Window(std::rc::Rc<std::cell::UnsafeCell<window>>),
    Pane(std::rc::Rc<std::cell::UnsafeCell<window_pane>>),
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
        super::rc::as_ptr(value)
    }
    pub fn session(&self) -> *mut session {
        let Self::Session(value) = self else {
            panic!("incorrect event payload type")
        };
        super::rc::as_ptr(value)
    }
    pub fn window(&self) -> *mut window {
        let Self::Window(value) = self else {
            panic!("incorrect event payload type")
        };
        super::rc::as_ptr(value)
    }
    pub fn pane(&self) -> *mut window_pane {
        let Self::Pane(value) = self else {
            panic!("incorrect event payload type")
        };
        super::rc::as_ptr(value)
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

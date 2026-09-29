//! Session indexes and groups; Session state belongs to its owner module.

pub use crate::src::session::session;

#[repr(C)]
pub struct session_entry {
    /// Weak traversal handle into the session index.
    pub owner: refbox::Weak<std::collections::BTreeMap<Vec<u8>, std::rc::Rc<std::cell::UnsafeCell<session>>>>,
}

#[repr(C)]
pub struct sessions {
    pub storage: Option<refbox::RefBox<std::collections::BTreeMap<Vec<u8>, std::rc::Rc<std::cell::UnsafeCell<session>>>>>,
}

#[repr(C)]
pub struct session_group {
    pub name: std::ffi::CString,
    pub entry: session_group_entry,
    /// Membership observes globally owned sessions without retaining them.
    pub(crate) members: Vec<std::rc::Weak<std::cell::UnsafeCell<session>>>,
}

impl session_group {
    pub fn empty() -> Self {
        Self {
            name: Default::default(),
            entry: session_group_entry { owner: refbox::Weak::new() },
            members: Default::default(),
        }
    }
}

#[repr(C)]
pub struct session_group_entry {
    /// Weak traversal handle into the session group index.
    pub owner: refbox::Weak<std::collections::BTreeMap<Vec<u8>, Box<session_group>>>,
}

#[repr(C)]
pub struct session_groups {
    pub storage: Option<refbox::RefBox<std::collections::BTreeMap<Vec<u8>, Box<session_group>>>>,
}

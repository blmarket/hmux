//! Authoritative session model declarations.

use super::abi::{timeval, u_int};
use super::environment::environ;
use super::event::event;
use super::options::options;
use super::terminal::termios;
use super::window::{winlink, winlink_stack, winlinks};

#[repr(C)]
pub struct session {
    pub id: u_int,
    pub name: std::ffi::CString,
    pub cwd: Option<std::ffi::CString>,
    pub creation_time: timeval,
    pub last_attached_time: timeval,
    pub activity_time: timeval,
    pub last_activity_time: timeval,
    pub lock_timer: event,
    pub curw: *mut winlink,
    pub lastw: winlink_stack,
    pub windows: winlinks,
    pub statusat: ::core::ffi::c_int,
    pub statuslines: u_int,
    pub options: Option<Box<options>>,
    pub flags: ::core::ffi::c_int,
    pub attached: u_int,
    pub tio: Option<Box<termios>>,
    pub environ: Option<Box<environ>>,
    pub entry: session_entry,
}

impl session {
    pub fn empty() -> Self {
        Self {
            id: Default::default(),
            name: Default::default(),
            cwd: Default::default(),
            creation_time: Default::default(),
            last_attached_time: Default::default(),
            activity_time: Default::default(),
            last_activity_time: Default::default(),
            lock_timer: Default::default(),
            curw: Default::default(),
            lastw: winlink_stack { storage: None },
            windows: winlinks { storage: None },
            statusat: Default::default(),
            statuslines: Default::default(),
            options: Default::default(),
            flags: Default::default(),
            attached: Default::default(),
            tio: Default::default(),
            environ: Default::default(),
            entry: session_entry { owner: None },
        }
    }
}

#[repr(C)]
pub struct session_entry {
    /// Weak traversal handle into the session index.
    pub owner: Option<refbox::Weak<std::collections::BTreeMap<Vec<u8>, *mut session>>>,
}

#[repr(C)]
pub struct sessions {
    pub storage: Option<refbox::RefBox<std::collections::BTreeMap<Vec<u8>, *mut session>>>,
}

#[repr(C)]
pub struct session_group {
    pub name: std::ffi::CString,
    pub entry: session_group_entry,
    pub(crate) members: Vec<*mut session>,
}

impl session_group {
    pub fn empty() -> Self {
        Self {
            name: Default::default(),
            entry: session_group_entry { owner: None },
            members: Default::default(),
        }
    }
}

#[repr(C)]
pub struct session_group_entry {
    /// Weak traversal handle into the session group index.
    pub owner: Option<refbox::Weak<std::collections::BTreeMap<Vec<u8>, Box<session_group>>>>,
}

#[repr(C)]
pub struct session_groups {
    pub storage: Option<refbox::RefBox<std::collections::BTreeMap<Vec<u8>, Box<session_group>>>>,
}

#[cfg(test)]
mod retained_session_tests {
    use super::*;
    use crate::src::{reactor, shared::rc};

    #[test]
    fn removing_typed_reference_preserves_other_owners() {
        use std::cell::UnsafeCell;
        use std::rc::Rc;

        unsafe {
            for cancel in [false, true] {
                let owner = Rc::new(UnsafeCell::new(session::empty()));
                let observer = Rc::downgrade(&owner);
                let from = c"typed-owner-test".to_owned();
                crate::src::session::session_remove_ref(owner.clone(), &from);
                drop(from);
                assert_eq!(Rc::strong_count(&owner), 2);
                if cancel {
                    reactor::shutdown_runtime();
                } else {
                    reactor::event_loop();
                }
                assert_eq!(Rc::strong_count(&owner), 1);
                drop(owner);
                assert!(observer.upgrade().is_none());
                reactor::shutdown_runtime();
            }
        }
    }

    #[test]
    fn final_reference_release_defers_cleanup_until_dispatch_or_cancellation() {
        unsafe {
            for cancel in [false, true] {
                let ptr = rc::new(session::empty());
                let observer = rc::downgrade(ptr);
                let owner = observer.upgrade().unwrap();
                rc::release(ptr);
                assert_eq!(rc::as_ptr(&owner), ptr);
                crate::src::session::session_remove_ref(owner, c"owner-test");
                assert_eq!(observer.strong_count(), 1);
                if cancel {
                    reactor::shutdown_runtime();
                } else {
                    reactor::event_loop();
                }
                assert_eq!(observer.strong_count(), 0);
                reactor::shutdown_runtime();
            }
        }
    }
}

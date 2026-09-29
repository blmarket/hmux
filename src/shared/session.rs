//! Authoritative session model declarations.

use super::abi::{timeval, u_int};
use super::environment::environ;
use super::event::event;
use super::options::options;
use super::terminal::termios;
use super::window::{winlink, winlink_stack, winlinks};

#[repr(C)]
pub struct session {
    /// Nonowning allocation observer for callbacks receiving borrowed pointers.
    pub(crate) observer: std::rc::Weak<std::cell::UnsafeCell<session>>,
    pub id: u_int,
    pub name: std::ffi::CString,
    pub cwd: Option<std::ffi::CString>,
    pub creation_time: timeval,
    pub last_attached_time: timeval,
    pub activity_time: timeval,
    pub last_activity_time: timeval,
    pub lock_timer: event,
    pub curw: refbox::Weak<winlink>,
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
    /// Observe the current session-index winlink.
    pub fn current_winlink(&self) -> refbox::Weak<winlink> {
        self.curw.clone()
    }

    /// The supplied winlink must be live in a session index.
    pub fn set_curw(&mut self, wl: refbox::Weak<winlink>) {
        self.curw = wl;
    }

    pub fn new() -> std::rc::Rc<std::cell::UnsafeCell<Self>> {
        std::rc::Rc::new_cyclic(|observer| {
            let mut value = Self::empty();
            value.observer = observer.clone();
            std::cell::UnsafeCell::new(value)
        })
    }

    pub fn empty() -> Self {
        Self {
            observer: std::rc::Weak::new(),
            id: Default::default(),
            name: Default::default(),
            cwd: Default::default(),
            creation_time: Default::default(),
            last_attached_time: Default::default(),
            activity_time: Default::default(),
            last_activity_time: Default::default(),
            lock_timer: Default::default(),
            curw: Default::default(),
            lastw: winlink_stack { storage: Default::default() },
            windows: winlinks { storage: None },
            statusat: Default::default(),
            statuslines: Default::default(),
            options: Default::default(),
            flags: Default::default(),
            attached: Default::default(),
            tio: Default::default(),
            environ: Default::default(),
            entry: session_entry { owner: refbox::Weak::new() },
        }
    }
}

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

#[cfg(test)]
mod retained_session_tests {
    use super::*;
    use crate::src::{reactor, shared::rc};

    #[test]
    fn group_membership_is_weak_but_traversal_retains_live_sessions() {
        use crate::src::session::{session_group_attached_count, session_group_count, session_group_members};
        use std::rc::Rc;

        unsafe {
            let first = session::new();
            let expired = session::new();
            let last = session::new();
            (*first.get()).attached = 2;
            (*last.get()).attached = 3;
            let first_weak = Rc::downgrade(&first);
            let last_weak = Rc::downgrade(&last);
            let mut group = session_group::empty();
            group.members = vec![first_weak.clone(), Rc::downgrade(&expired), last_weak.clone()];
            assert_eq!(Rc::strong_count(&first), 1);
            drop(expired);
            assert_eq!(session_group_count(&mut group), 2);
            assert_eq!(session_group_attached_count(&mut group), 5);
            let snapshot = session_group_members(&mut group);
            assert_eq!(snapshot.len(), 2);
            assert!(Rc::ptr_eq(&snapshot[0], &first));
            assert!(Rc::ptr_eq(&snapshot[1], &last));
            drop(first);
            drop(last);
            drop(group);
            assert!(first_weak.upgrade().is_some());
            assert!(last_weak.upgrade().is_some());
            drop(snapshot);
            assert!(first_weak.upgrade().is_none());
            assert!(last_weak.upgrade().is_none());
        }
    }

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
                let initial = session::new();
                let ptr = rc::as_ptr(&initial);
                let observer = (*ptr).observer.clone();
                let owner = observer.upgrade().unwrap();
                drop(initial);
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

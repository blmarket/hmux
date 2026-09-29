//! Session state is private to its implementation. Other models use Session.
use crate::src::shared::abi::{timeval, u_int};
use crate::src::shared::environment::environ;
use crate::src::shared::event::event;
use crate::src::shared::options::options;
use crate::src::shared::session::session_group;
use crate::src::shared::terminal::termios;
use crate::src::shared::window::{winlink, winlink_stack, winlinks};

#[repr(C)]
pub struct session {
    /// Nonowning allocation observer for callbacks receiving borrowed pointers.
    pub(super) observer: std::rc::Weak<std::cell::UnsafeCell<session>>,
    pub(super) id: u_int,
    pub(super) name: std::ffi::CString,
    pub(super) cwd: Option<std::ffi::CString>,
    pub(super) creation_time: timeval,
    pub(super) last_attached_time: timeval,
    pub(super) activity_time: timeval,
    pub(super) last_activity_time: timeval,
    pub(super) lock_timer: event,
    pub(super) curw: refbox::Weak<winlink>,
    pub(super) lastw: winlink_stack,
    pub(super) windows: winlinks,
    pub(super) statusat: ::core::ffi::c_int,
    pub(super) statuslines: u_int,
    // Transitional: options-scope and customization callbacks still store this pointer.
    pub(crate) options: Option<Box<options>>,
    pub(super) flags: ::core::ffi::c_int,
    pub(super) attached: u_int,
    pub(super) tio: Option<Box<termios>>,
    pub(super) environ: Option<Box<environ>>,
    /// Weak traversal handle into the containing index.
    pub(super) owner: refbox::Weak<
        std::collections::BTreeMap<Vec<u8>, std::rc::Rc<std::cell::UnsafeCell<session>>>,
    >,
}

impl session {
    /// Observe the current session-index winlink.
    pub(super) fn current_winlink(&self) -> refbox::Weak<winlink> {
        self.curw.clone()
    }

    /// The supplied winlink must be live in a session index.
    pub(super) fn set_curw(&mut self, wl: refbox::Weak<winlink>) {
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
            lastw: Default::default(),
            windows: None,
            statusat: Default::default(),
            statuslines: Default::default(),
            options: Default::default(),
            flags: Default::default(),
            attached: Default::default(),
            tio: Default::default(),
            environ: Default::default(),
            owner: refbox::Weak::new(),
        }
    }
}

#[cfg(test)]
mod retained_session_tests {
    use super::*;
    use crate::src::{reactor, shared::rc};

    #[test]
    fn group_membership_is_weak_but_traversal_retains_live_sessions() {
        use crate::src::session::{
            session_group_attached_count, session_group_count, session_group_members,
        };
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
            group.members = vec![
                first_weak.clone(),
                Rc::downgrade(&expired),
                last_weak.clone(),
            ];
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

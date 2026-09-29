//! Borrowed model views and deferred cleanup for ordinary Rc owners.
//!
//! Ownership is kept in Rc values. Callback observers use Weak handles created
//! with the allocation; borrowed raw addresses never represent strong references.

use std::cell::UnsafeCell;
use std::rc::Rc;

/// Borrow the model pointer without deriving it from a reference to the value.
pub fn as_ptr<T>(owner: &Rc<UnsafeCell<T>>) -> *mut T {
    owner.get()
}

/// Compare optional retained handles by allocation identity, including absent targets.
pub fn same<T>(a: Option<&Rc<T>>, b: Option<&Rc<T>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

/// Defer one ordinary Rc drop. Cancellation or a scheduling failure drops the
/// capture normally, so cleanup still happens even without callback dispatch.
pub fn release_later<T: 'static>(owner: Rc<T>) {
    let mut owner = Some(owner);
    unsafe {
        crate::src::reactor::event_once(move |_, _| drop(owner.take()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct Value(Rc<Cell<usize>>);

    impl Drop for Value {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    #[test]
    fn final_release_cleans_once_and_expires_weak_observers() {
        let calls = Rc::new(Cell::new(0));
        let owner = Rc::new(UnsafeCell::new(Value(calls.clone())));
        let ptr = as_ptr(&owner);
        let weak = Rc::downgrade(&owner);
        let retained = owner.clone();
        drop(owner);
        assert_eq!(calls.get(), 0);
        assert_eq!(as_ptr(&weak.upgrade().unwrap()), ptr);
        drop(retained);
        assert_eq!(calls.get(), 1);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn deferred_release_survives_until_dispatch_or_cancellation() {
        unsafe {
            for cancel in [false, true] {
                let calls = Rc::new(Cell::new(0));
                let owner = Rc::new(Value(calls.clone()));
                release_later(owner);
                assert_eq!(calls.get(), 0);
                if cancel {
                    crate::src::reactor::shutdown_runtime();
                } else {
                    crate::src::reactor::event_loop();
                }
                assert_eq!(calls.get(), 1);
                crate::src::reactor::shutdown_runtime();
            }
        }
    }
}

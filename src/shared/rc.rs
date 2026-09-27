//! Adapters for the remaining legacy raw-reference interfaces.
//!
//! Model ownership and transfers use ordinary Rc and Weak handles.
//!
//! Each retained raw pointer represents one strong reference. Borrowed pointers
//! do not. T owns final cleanup through Drop, so dropping the last ordinary Rc
//! (also when a deferred callback is cancelled) runs cleanup exactly once.

use std::cell::UnsafeCell;
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak};

/// Borrow the model pointer without deriving it from a reference to the value.
pub fn as_ptr<T>(owner: &Rc<UnsafeCell<T>>) -> *mut T {
    Rc::as_ptr(owner).cast_mut().cast()
}

/// Allocate a model value and return its initial retained raw reference.
/// Final release drops T, including its model-specific cleanup.
pub fn new<T>(value: T) -> *mut T {
    Rc::into_raw(Rc::new(UnsafeCell::new(value)))
        .cast_mut()
        .cast()
}

/// Add a strong reference without changing the model address.
///
/// # Safety
/// `ptr` must refer to a live `Rc<UnsafeCell<T>>` allocation for exactly this T.
/// Each retain must be matched by one release or transfer through `take`.
pub unsafe fn retain<T>(ptr: *mut T) {
    Rc::increment_strong_count(ptr.cast::<UnsafeCell<T>>());
}

/// Consume one retained raw reference, without changing the count yet.
///
/// # Safety
/// The caller must own one unreleased raw reference from `new` or `retain`.
/// This transfers it to the returned Rc; it must not be released again as raw.
unsafe fn take<T>(ptr: *mut T) -> Rc<UnsafeCell<T>> {
    Rc::from_raw(ptr.cast::<UnsafeCell<T>>())
}

/// Consume one retained raw reference.
///
/// # Safety
/// The ownership requirements of `take` apply. Final release may run cleanup
/// and invalidate all borrowed pointers.
pub unsafe fn release<T>(ptr: *mut T) {
    drop(take(ptr));
}

/// Inspect the count, including references pending deferred release.
///
/// # Safety
/// `ptr` must refer to a live `Rc<UnsafeCell<T>>` allocation with a strong owner.
pub unsafe fn strong_count<T>(ptr: *mut T) -> usize {
    let owner = ManuallyDrop::new(take(ptr));
    Rc::strong_count(&owner)
}

/// Observe an allocation without retaining its value.
///
/// # Safety
/// `ptr` must refer to a live `Rc<UnsafeCell<T>>` allocation with a strong owner.
pub unsafe fn downgrade<T>(ptr: *mut T) -> Weak<UnsafeCell<T>> {
    let owner = ManuallyDrop::new(take(ptr));
    Rc::downgrade(&owner)
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
        unsafe {
            let calls = Rc::new(Cell::new(0));
            let ptr = new(Value(calls.clone()));
            let weak = downgrade(ptr);
            retain(ptr);
            release(ptr);
            assert_eq!(calls.get(), 0);
            assert_eq!(as_ptr(&weak.upgrade().unwrap()), ptr);
            release(ptr);
            assert_eq!(calls.get(), 1);
            assert!(weak.upgrade().is_none());
        }
    }

    #[test]
    fn deferred_release_survives_until_dispatch_or_cancellation() {
        unsafe {
            for cancel in [false, true] {
                let calls = Rc::new(Cell::new(0));
                let ptr = new(Value(calls.clone()));
                release_later(take(ptr));
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

//! Rc ownership behind the model's existing raw-pointer interfaces.
//!
//! Each retained raw pointer represents one strong reference. Borrowed pointers
//! do not. The allocation owns final cleanup, so dropping an ordinary Rc (also
//! when a deferred callback is cancelled) runs cleanup exactly once.

use std::cell::UnsafeCell;
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak};

#[repr(C)]
pub struct Allocation<T> {
    // First field: a model pointer has the allocation's address. UnsafeCell is
    // transparent, and mutations must obey the existing raw-pointer contract.
    value: UnsafeCell<T>,
    cleanup: unsafe fn(*mut T),
}

/// Borrow the model pointer without deriving it from a reference to the value.
/// Retain/release must preserve the provenance of Rc's complete allocation.
pub fn as_ptr<T>(owner: &Rc<Allocation<T>>) -> *mut T {
    Rc::as_ptr(owner).cast_mut().cast()
}

impl<T> Drop for Allocation<T> {
    fn drop(&mut self) {
        unsafe { (self.cleanup)(self.value.get()) }
    }
}

/// Allocate a model value and return its initial retained raw reference.
///
/// # Safety
/// `cleanup` must accept this value at final release. It may release external
/// resources, but must not free/drop the value itself or resurrect its Rc.
/// Rust drops the value's fields after cleanup. Required logical shutdown must
/// happen before final release, including release caused by cancellation.
pub unsafe fn new<T>(value: T, cleanup: unsafe fn(*mut T)) -> *mut T {
    Rc::into_raw(Rc::new(Allocation {
        value: UnsafeCell::new(value),
        cleanup,
    })) as *mut T
}

/// Add a strong reference without changing the model address.
///
/// # Safety
/// `ptr` must refer to a live allocation made by `new` for exactly this T.
/// Each retain must be matched by one release or transfer through `take`.
pub unsafe fn retain<T>(ptr: *mut T) {
    Rc::increment_strong_count(ptr.cast::<Allocation<T>>());
}

/// Consume one retained raw reference, without changing the count yet.
///
/// # Safety
/// The caller must own one unreleased raw reference from `new` or `retain`.
/// This transfers it to the returned Rc; it must not be released again as raw.
pub unsafe fn take<T>(ptr: *mut T) -> Rc<Allocation<T>> {
    Rc::from_raw(ptr.cast::<Allocation<T>>())
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
/// `ptr` must refer to a live `new` allocation with at least one strong owner.
pub unsafe fn strong_count<T>(ptr: *mut T) -> usize {
    let owner = ManuallyDrop::new(take(ptr));
    Rc::strong_count(&owner)
}

/// Observe an allocation without retaining its value.
///
/// # Safety
/// `ptr` must refer to a live `new` allocation with at least one strong owner.
pub unsafe fn downgrade<T>(ptr: *mut T) -> Weak<Allocation<T>> {
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

    unsafe fn cleanup(ptr: *mut Value) {
        (*ptr).0.set((*ptr).0.get() + 1);
    }

    #[test]
    fn final_release_cleans_once_and_expires_weak_observers() {
        unsafe {
            let calls = Rc::new(Cell::new(0));
            let ptr = new(Value(calls.clone()), cleanup);
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
                let ptr = new(Value(calls.clone()), cleanup);
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

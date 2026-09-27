use refbox::{BorrowError, RefBox, Weak, coerce_weak};
use std::cell::Cell;
use std::mem::size_of;

#[test]
fn empty_state_and_error_callbacks() {
    struct NoDefault;
    const EMPTY: Weak<NoDefault> = Weak::new();
    let empty = EMPTY;
    assert!(empty.is_empty());
    assert!(!empty.is_alive());
    assert!(!empty.is_borrowed());
    assert_eq!(empty.weak_count(), 0);
    assert_eq!(empty, Weak::default());
    assert_eq!(empty, empty.clone());
    assert_eq!(empty, empty.try_clone().unwrap());
    assert!(matches!(empty.try_borrow_mut(), Err(BorrowError::Dropped)));
    assert_eq!(
        empty.try_access_mut(|_| panic!("must not access")),
        Err(BorrowError::Dropped)
    );
    assert!(matches!(
        empty.try_borrow_mut_or_else(|| panic!("not borrowed"), || "empty"),
        Err("empty")
    ));
    assert!(format!("{empty:?}").starts_with("Weak("));
    assert_eq!(size_of::<Weak<NoDefault>>(), size_of::<*const NoDefault>());
    assert_eq!(
        size_of::<Option<Weak<NoDefault>>>(),
        size_of::<Weak<NoDefault>>()
    );
}

#[test]
fn empty_live_and_expired_have_distinct_identity() {
    let owner = RefBox::new(7);
    let empty = Weak::new();
    let weak = owner.downgrade();
    assert_ne!(empty, weak);
    assert_ne!(empty, owner);
    assert_ne!(owner, empty);
    assert!(!empty.is(&owner));
    let mut slot = empty.clone();
    slot.clone_from(&weak);
    assert_eq!(owner.weak_count(), 2);
    slot.clone_from(&empty);
    assert!(slot.is_empty());
    assert_eq!(owner.weak_count(), 1);
    drop(owner);
    assert!(!weak.is_empty());
    assert!(!weak.is_alive());
    let expired_clone = weak.try_clone().unwrap();
    assert_eq!(weak.weak_count(), 2);
    assert_eq!(weak, expired_clone);
    assert_ne!(weak, empty);
}

#[test]
fn raw_roundtrips_and_unsized_empty_handles() {
    let empty = Weak::<u64>::new();
    let sentinel = empty.as_ptr();
    assert!(!sentinel.is_null());
    let raw = empty.into_raw();
    let restored = unsafe { Weak::from_raw(raw) };
    assert!(restored.is_empty());
    assert_eq!(sentinel, restored.as_ptr());
    let cast = unsafe { restored.cast::<i64>() };
    assert!(cast.is_empty());

    let dynamic = coerce_weak!(Weak::<u64>::new() => dyn std::fmt::Debug);
    assert!(dynamic.is_empty());
    assert_eq!(dynamic.as_ptr().cast::<()>().addr(), sentinel.addr());
    assert_eq!(dynamic, dynamic.clone());
    assert!(matches!(
        dynamic.try_borrow_mut(),
        Err(BorrowError::Dropped)
    ));
    let dynamic = unsafe { Weak::from_raw(dynamic.into_raw()) };
    assert!(dynamic.try_clone().unwrap().is_empty());

    let slice = coerce_weak!(Weak::<[u8; 3]>::new() => [u8]);
    assert!(slice.is_empty());
    assert_eq!(slice.as_ptr().len(), 3);
    assert_eq!(slice, slice.clone());
    assert!(matches!(slice.try_borrow_mut(), Err(BorrowError::Dropped)));
    let slice = unsafe { Weak::from_raw(slice.into_raw()) };
    assert_eq!(slice.as_ptr().len(), 3);
}

#[test]
fn live_raw_roundtrip_and_deferred_destruction() {
    struct CountDrop<'a>(&'a Cell<usize>);
    impl Drop for CountDrop<'_> {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let drops = Cell::new(0);
    let owner = RefBox::new(CountDrop(&drops));
    let weak = owner.downgrade();
    let ptr = weak.as_ptr();
    let weak = unsafe { Weak::from_raw(weak.into_raw()) };
    assert_eq!(ptr, weak.as_ptr());
    assert_eq!(weak.weak_count(), 1);
    let borrow = weak.try_borrow_mut().unwrap();
    assert!(weak.is_borrowed());
    assert!(matches!(weak.try_borrow_mut(), Err(BorrowError::Borrowed)));
    drop(owner);
    assert!(!weak.is_alive());
    assert!(!weak.is_empty());
    assert_eq!(drops.get(), 0);
    drop(borrow);
    assert_eq!(drops.get(), 1);
    assert_eq!(ptr, weak.as_ptr());
    drop(weak);
    assert_eq!(drops.get(), 1);
}

#[test]
#[cfg(any(feature = "cyclic", feature = "cyclic_stable"))]
fn cyclic_construction_and_unwinding_preserve_weak_identity() {
    struct Node {
        parent: Weak<Node>,
    }
    let node = RefBox::new_cyclic(|this| {
        assert!(!this.is_empty());
        assert!(!this.is_alive());
        Node {
            parent: this.clone(),
        }
    });
    assert!(node.try_borrow_mut().unwrap().parent.is(&node));
    drop(node);

    let mut escaped = Weak::<u32>::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        RefBox::new_cyclic(|this| {
            escaped = this.clone();
            panic!("construction failed");
        })
    }));
    assert!(result.is_err());
    assert!(!escaped.is_empty());
    assert!(!escaped.is_alive());
    assert_eq!(escaped.weak_count(), 1);
    assert!(matches!(
        escaped.try_borrow_mut(),
        Err(BorrowError::Dropped)
    ));
}

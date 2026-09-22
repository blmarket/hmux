//! Environment-owner compatibility and lifetime checks.
use hmux2::src::environ::{environ, EnvironOwner, ENVIRON_HIDDEN};

#[test]
fn owner_preserves_missing_valueless_flags_order_and_bytes() {
    let mut env = EnvironOwner::new();
    env.set(b"b", 0, b"value").unwrap();
    env.set(b"a\xff", ENVIRON_HIDDEN, b"\xfe\xfd").unwrap();
    env.set(b"a", 0, b"ascii").unwrap();
    env.clear(b"empty").unwrap();
    env.set(b"\xff", 0x40, b"high-byte").unwrap();

    assert!(env.find_bytes(b"missing").unwrap().is_none());

    let empty = env.find_bytes(b"empty").unwrap().unwrap();
    assert_eq!(empty.name_bytes(), b"empty");
    assert_eq!(empty.value_bytes(), None);
    assert_eq!(empty.flags(), 0);

    let hidden = env.find_bytes(b"a\xff").unwrap().unwrap();
    assert_eq!(hidden.value_bytes(), Some(&b"\xfe\xfd"[..]));
    assert_eq!(hidden.flags(), ENVIRON_HIDDEN);

    let names: Vec<Vec<u8>> = env
        .borrow()
        .entries()
        .map(|entry| entry.name_bytes().to_vec())
        .collect();
    assert_eq!(
        names,
        vec![
            b"a".to_vec(),
            b"a\xff".to_vec(),
            b"b".to_vec(),
            b"empty".to_vec(),
            b"\xff".to_vec()
        ]
    );

    assert!(env.set(b"bad\0name", 0, b"value").is_err());
    assert!(env.find_bytes(b"bad\0name").is_err());
}

#[test]
fn borrowed_copy_and_entry_views_follow_their_owner() {
    let mut source = EnvironOwner::new();
    source.set(b"copied", 7, b"bytes\xff").unwrap();
    let mut destination = EnvironOwner::new();
    destination.copy_from(source.borrow());

    let entry = destination.find_bytes(b"copied").unwrap().unwrap();
    assert_eq!(entry.value_bytes(), Some(&b"bytes\xff"[..]));
    assert_eq!(entry.flags(), 7);
    assert!(!entry.as_ptr().is_null());
}

#[test]
fn owner_transfer_reclaims_the_same_c_tree_once() {
    fn assert_copy<T: Copy>() {}

    assert_copy::<environ>();
    assert!(std::mem::needs_drop::<EnvironOwner>());
    assert!(!std::mem::needs_drop::<environ>());

    let mut original = EnvironOwner::new();
    original.set(b"transferred", 0, b"owned").unwrap();
    let raw = original.transfer();
    assert!(!raw.is_null());

    // Adoption after transfer restores one and only one Rust Drop owner.
    let adopted = unsafe { EnvironOwner::from_raw(raw) };
    assert_eq!(
        adopted
            .find_bytes(b"transferred")
            .unwrap()
            .unwrap()
            .value_bytes(),
        Some(&b"owned"[..])
    );
    drop(adopted);

    assert!(unsafe { EnvironOwner::from_raw_owned(std::ptr::null_mut()).is_none() });
}

#[test]
fn updates_preserve_entry_addresses_and_removal_preserves_saved_successors() {
    use hmux2::src::environ::{environ_first, environ_next};
    let mut env = EnvironOwner::new();
    env.set(b"middle", 7, b"old").unwrap();
    let middle = env.find_bytes(b"middle").unwrap().unwrap().as_ptr();
    // Grow the index in reverse order to exercise map rebalancing.
    for i in (0..128).rev() {
        env.set(format!("key-{i:03}").as_bytes(), 0, b"value")
            .unwrap();
    }
    env.set(b"middle", ENVIRON_HIDDEN, b"new").unwrap();
    assert_eq!(env.find_bytes(b"middle").unwrap().unwrap().as_ptr(), middle);
    env.clear(b"middle").unwrap();
    let cleared = env.find_bytes(b"middle").unwrap().unwrap();
    assert_eq!(cleared.as_ptr(), middle);
    assert_eq!(cleared.value_bytes(), None);
    assert_eq!(cleared.flags(), ENVIRON_HIDDEN);

    unsafe {
        let mut current = environ_first(env.as_ptr());
        let mut removed = 0;
        while !current.is_null() {
            let next = environ_next(current);
            let name = std::ffi::CStr::from_ptr((*current).name).to_owned();
            env.unset_cstr(&name);
            current = next;
            removed += 1;
        }
        assert_eq!(removed, 129);
        assert!(environ_first(env.as_ptr()).is_null());
    }
    env.set(b"reinserted", 0, b"ok").unwrap();
    assert_eq!(env.borrow().entries().count(), 1);
}

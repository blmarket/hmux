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

#[test]
fn variadic_updates_can_read_the_previous_value_before_replacement() {
    use hmux2::src::environ::environ_set;
    use std::ffi::CStr;

    let mut env = EnvironOwner::new();
    env.set(b"VAR", ENVIRON_HIDDEN, b"old\xff").unwrap();
    let entry = env.find_bytes(b"VAR").unwrap().unwrap().as_ptr();
    unsafe {
        environ_set(
            env.as_ptr(),
            (*entry).name,
            0x40,
            c"%s-new".as_ptr(),
            (*entry).value,
        );
        assert_eq!(entry, env.find_bytes(b"VAR").unwrap().unwrap().as_ptr());
        assert_eq!(CStr::from_ptr((*entry).value).to_bytes(), b"old\xff-new");
        assert_eq!((*entry).flags, 0x40);
    }
    env.clear(b"VAR").unwrap();
    let cleared = env.find_bytes(b"VAR").unwrap().unwrap();
    assert_eq!(cleared.as_ptr(), entry);
    assert_eq!(cleared.value_bytes(), None);
    assert_eq!(cleared.flags(), 0x40);
    env.set(b"VAR", 0, b"").unwrap();
    assert_eq!(
        env.find_bytes(b"VAR").unwrap().unwrap().value_bytes(),
        Some(&b""[..])
    );
}

#[test]
fn put_splits_first_equals_and_preserves_c_string_bytes() {
    use hmux2::src::environ::environ_put;

    let env = EnvironOwner::new();
    unsafe {
        environ_put(env.as_ptr(), b"plain=one=two\0".as_ptr().cast(), 7);
        environ_put(env.as_ptr(), b"\xff=\xfe\0".as_ptr().cast(), 8);
        environ_put(env.as_ptr(), b"=empty-name\0".as_ptr().cast(), 9);
        environ_put(env.as_ptr(), b"no-equals\0".as_ptr().cast(), 10);
        environ_put(
            env.as_ptr(),
            b"first=visible\0later=hidden\0".as_ptr().cast(),
            11,
        );
    }
    assert_eq!(
        env.find_bytes(b"plain").unwrap().unwrap().value_bytes(),
        Some(&b"one=two"[..])
    );
    assert_eq!(
        env.find_bytes(b"\xff").unwrap().unwrap().value_bytes(),
        Some(&b"\xfe"[..])
    );
    assert_eq!(
        env.find_bytes(b"").unwrap().unwrap().value_bytes(),
        Some(&b"empty-name"[..])
    );
    assert!(env.find_bytes(b"no-equals").unwrap().is_none());
    assert_eq!(
        env.find_bytes(b"first").unwrap().unwrap().value_bytes(),
        Some(&b"visible"[..])
    );
    assert!(env.find_bytes(b"later").unwrap().is_none());
}

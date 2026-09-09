//! File lookup, client printing, and ownership tests.

use crate::file::{file_can_print, file_find_ref};
use crate::tests::test_fixtures::{ensure_reactor, globals, zeroed_client};
use crate::types::*;

#[test]
fn file_find_ref_returns_none_for_missing_stream_and_entry_for_present() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        let files = std::rc::Rc::new(std::cell::RefCell::new(client_files_t::new()));
        assert!(file_find_ref(&files.borrow(), 5).is_none());
        let cf = ClientFileRef::create_with_peer(
            None,
            &FileOwner::Shared(std::rc::Rc::downgrade(&files)),
            5,
            None,
            ClientFileData::None,
        );

        assert!(file_find_ref(&files.borrow(), 5).unwrap().ptr_eq(&cf));
        assert!(file_find_ref(&files.borrow(), 6).is_none());
        cf.close();
        assert!(file_find_ref(&files.borrow(), 5).is_none());
    }
}

#[test]
fn file_can_print_answers_for_client_flags() {
    let _guard = globals();
    {
        assert_eq!(file_can_print(None), 0);
        let mut c = zeroed_client();
        // attached
        *unsafe { c.flags_mut() } = crate::file::CLIENT_ATTACHED as u64;
        assert_eq!(file_can_print(Some(&*(unsafe { c.as_client() }))), 0);
        // control
        *unsafe { c.flags_mut() } = crate::file::CLIENT_CONTROL as u64;
        assert_eq!(file_can_print(Some(&*(unsafe { c.as_client() }))), 0);
        // neither
        *unsafe { c.flags_mut() } = 0;
        assert_eq!(file_can_print(Some(&*(unsafe { c.as_client() }))), 1);
    }
}

#[test]
fn file_create_with_client_for_attached_client_becomes_detached() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        let mut c = zeroed_client();
        *c.flags_mut() = crate::file::CLIENT_ATTACHED as u64;
        let cf = ClientFileRef::create_with_client(
            Some(c.as_client_mut()),
            99,
            None,
            ClientFileData::None,
        );

        assert!(
            cf.borrow().client().is_none(),
            "attached client is detached"
        );
        assert_eq!(cf.borrow().stream, 99);
        cf.close();
    }
}

#[test]
fn file_create_with_client_keeps_the_client_tree_entry_until_free() {
    let _guard = globals();
    unsafe {
        ensure_reactor();
        let mut c = zeroed_client();
        *c.flags_mut() = 0;
        c.as_client_mut().peer = None;
        // need a valid files map inside client
        c.as_client_mut().files = std::collections::BTreeMap::new();
        let cf = ClientFileRef::create_with_client(
            Some(c.as_client_mut()),
            11,
            None,
            ClientFileData::None,
        );

        assert!(cf.borrow().client().is_some_and(|held| held.ptr_eq(&c)));
        cf.close();
        // file map entry removed
        assert!(file_find_ref(&c.as_client().files, 11).is_none());
    }
}

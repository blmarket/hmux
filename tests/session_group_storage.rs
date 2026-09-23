use hmux2::src::session::*;
use std::ffi::{CStr, CString};

#[test]
fn byte_order_duplicates_neighbors_and_owner_removal() {
    unsafe {
        let mut head = session_groups { storage: None };
        assert!(session_groups_minmax(&mut head, -1).is_null());
        let names = [c"z", c"\xff", c"a", c""];
        let mut pointers = Vec::new();
        for name in names {
            let owner = SessionGroupOwner::new(name);
            let pointer = owner.node_ptr();
            assert!(session_groups_insert(&mut head, owner).is_null());
            pointers.push(pointer);
        }
        // Moving the tree head must preserve entry-only traversal.
        let mut head = Box::new(head);
        let duplicate = SessionGroupOwner::new(c"a");
        assert_eq!(session_groups_insert(&mut *head, duplicate), pointers[2]);
        let outsider = SessionGroupOwner::new(c"a");
        assert!(!session_groups_remove(&mut *head, outsider.node_ptr()));
        let mut lookup = session_group {
            name: c"a".as_ptr(),
            sessions: session_group_sessions {
                tqh_first: std::ptr::null_mut(),
                tqh_last: std::ptr::null_mut(),
            },
            entry: session_group_entry {
                owner: std::ptr::null_mut(),
            },
        };
        assert_eq!(session_groups_find(&mut *head, &mut lookup), pointers[2]);
        lookup.name = c"b".as_ptr();
        assert!(session_groups_find(&mut *head, &mut lookup).is_null());
        assert_eq!(session_groups_nfind(&mut *head, &mut lookup), pointers[0]);
        let mut node = session_groups_minmax(&mut *head, -1);
        for name in [c"", c"a", c"z", c"\xff"] {
            assert_eq!(CStr::from_ptr((*node).name), name);
            node = session_groups_next(node);
        }
        assert!(node.is_null());
        node = session_groups_minmax(&mut *head, 1);
        for name in [c"\xff", c"z", c"a", c""] {
            assert_eq!(CStr::from_ptr((*node).name), name);
            node = session_groups_prev(node);
        }
        assert!(node.is_null());
        // Cache the successor before the index drops the current owner.
        node = session_groups_minmax(&mut *head, -1);
        while !node.is_null() {
            let next = session_groups_next(node);
            assert!(session_groups_remove(&mut *head, node));
            node = next;
        }
        assert!(head.storage.is_none());
        let replacement = SessionGroupOwner::new(c"a");
        let replacement_ptr = replacement.node_ptr();
        assert!(session_groups_insert(&mut *head, replacement).is_null());
        assert_eq!(session_groups_minmax(&mut *head, -1), replacement_ptr);
        assert!(session_groups_remove(&mut *head, replacement_ptr));
        assert!(head.storage.is_none());
    }
}

#[test]
fn named_group_owns_borrowed_name_and_reuses_stable_allocation() {
    unsafe {
        let head = &raw mut session_groups;
        assert!((*head).storage.is_none());
        let borrowed_name = CString::new(b"gr\xffup".to_vec()).unwrap();
        let first = session_group_new(borrowed_name.as_ptr());
        drop(borrowed_name);
        assert_eq!(CStr::from_ptr((*first).name).to_bytes(), b"gr\xffup");
        assert_eq!(session_group_new(c"gr\xffup".as_ptr()), first);
        assert_eq!(session_group_find(c"gr\xffup".as_ptr()), first);
        assert!(session_group_find(c"missing".as_ptr()).is_null());
        assert_eq!(
            (*first).sessions.tqh_last,
            &raw mut (*first).sessions.tqh_first
        );
        assert!(session_groups_remove(head, first));
        assert!((*head).storage.is_none());
    }
}

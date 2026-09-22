use hmux2::src::session::*;
use std::{ffi::CStr, ptr::null_mut};

fn node(name: &CStr) -> Box<session> {
    let mut node: Box<session> = Box::new(unsafe { std::mem::zeroed() });
    node.name = name.as_ptr().cast_mut();
    node
}

#[test]
fn byte_order_duplicates_neighbors_and_removal() {
    unsafe {
        let mut head = sessions {
            storage: null_mut(),
        };
        assert!(sessions_minmax(&mut head, -1).is_null());
        let mut nodes: Vec<_> = [c"z", c"\xff", c"a", c""].into_iter().map(node).collect();
        for node in &mut nodes {
            assert!(sessions_insert(&mut head, &mut **node).is_null());
        }
        // Moving the tree head must preserve entry-only traversal.
        let mut head = Box::new(head);
        let mut duplicate = node(c"a");
        let a = &mut *nodes[2] as *mut session;
        assert_eq!(sessions_insert(&mut *head, &mut *duplicate), a);
        assert!(duplicate.entry.owner.is_null());
        assert!(sessions_remove(&mut *head, &mut *duplicate).is_null());
        assert_eq!(sessions_find(&mut *head, &mut *duplicate), a);
        let mut missing = node(c"b");
        assert!(sessions_find(&mut *head, &mut *missing).is_null());
        assert_eq!(
            sessions_nfind(&mut *head, &mut *missing),
            &mut *nodes[0] as *mut session
        );
        let mut node = sessions_minmax(&mut *head, -1);
        for name in [c"", c"a", c"z", c"\xff"] {
            assert_eq!(CStr::from_ptr((*node).name), name);
            node = sessions_next(node);
        }
        assert!(node.is_null());
        node = sessions_minmax(&mut *head, 1);
        for name in [c"\xff", c"z", c"a", c""] {
            assert_eq!(CStr::from_ptr((*node).name), name);
            node = sessions_prev(node);
        }
        assert!(node.is_null());
        // Cache the successor before removing the current record, as callers do.
        node = sessions_minmax(&mut *head, -1);
        while !node.is_null() {
            let next = sessions_next(node);
            assert_eq!(sessions_remove(&mut *head, node), node);
            assert!((*node).entry.owner.is_null());
            node = next;
        }
        assert!(head.storage.is_null());
        assert!(sessions_insert(&mut *head, a).is_null());
        assert_eq!(sessions_remove(&mut *head, a), a);
        assert!(head.storage.is_null());
    }
}

#[test]
fn rename_preserves_identity_and_updates_name_order() {
    unsafe {
        let head = &raw mut sessions;
        assert!((*head).storage.is_null());
        let mut first = node(c"a");
        let mut second = node(c"m");
        first.id = 42;
        let first_ptr = &mut *first as *mut session;
        let second_ptr = &mut *second as *mut session;
        assert!(sessions_insert(head, first_ptr).is_null());
        assert!(sessions_insert(head, second_ptr).is_null());
        assert_eq!(session_find(c"a".as_ptr()), first_ptr);
        assert_eq!(session_find_by_id(42), first_ptr);
        assert_eq!(sessions_next(first_ptr), second_ptr);
        // Same remove/change/reinsert sequence used by rename-session.
        assert_eq!(sessions_remove(head, first_ptr), first_ptr);
        first.name = c"z".as_ptr().cast_mut();
        assert!(sessions_insert(head, first_ptr).is_null());
        assert!(session_find(c"a".as_ptr()).is_null());
        assert_eq!(session_find(c"z".as_ptr()), first_ptr);
        assert_eq!(session_find_by_id(42), first_ptr);
        assert_eq!(sessions_minmax(head, -1), second_ptr);
        assert_eq!(sessions_next(second_ptr), first_ptr);
        assert_eq!(sessions_prev(first_ptr), second_ptr);
        assert_eq!(sessions_remove(head, first_ptr), first_ptr);
        assert_eq!(sessions_remove(head, second_ptr), second_ptr);
        assert!((*head).storage.is_null());
    }
}

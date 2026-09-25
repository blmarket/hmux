use hmux2::src::session::*;
use hmux2::src::shared::session::{session, sessions};
use std::ffi::CStr;

fn node(name: &CStr) -> Box<session> {
    let mut node: Box<session> = Box::new(session::empty());
    node.name = name.to_owned();
    node
}

#[test]
fn byte_order_duplicates_neighbors_and_removal() {
    unsafe {
        let mut head = sessions { storage: None };
        assert!(sessions_minmax(&head, -1).is_null());
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
        assert_eq!(sessions_find(&*head, &*duplicate), a);
        let mut missing = node(c"b");
        assert!(sessions_find(&*head, &*missing).is_null());
        assert_eq!(
            sessions_nfind(&*head, &*missing),
            &mut *nodes[0] as *mut session
        );
        let mut node = sessions_minmax(&*head, -1);
        for name in [c"", c"a", c"z", c"\xff"] {
            assert_eq!(CStr::from_ptr(((*node).name).as_ptr().cast_mut()), name);
            node = sessions_next(&*node);
        }
        assert!(node.is_null());
        node = sessions_minmax(&*head, 1);
        for name in [c"\xff", c"z", c"a", c""] {
            assert_eq!(CStr::from_ptr(((*node).name).as_ptr().cast_mut()), name);
            node = sessions_prev(&*node);
        }
        assert!(node.is_null());
        // Cache the successor before removing the current record, as callers do.
        node = sessions_minmax(&*head, -1);
        while !node.is_null() {
            let next = sessions_next(&*node);
            assert_eq!(sessions_remove(&mut *head, node), node);
            assert!((*node).entry.owner.is_null());
            node = next;
        }
        assert!(head.storage.is_none());
        assert!(sessions_insert(&mut *head, a).is_null());
        assert_eq!(sessions_remove(&mut *head, a), a);
        assert!(head.storage.is_none());
    }
}

#[test]
fn rename_preserves_identity_and_updates_name_order() {
    unsafe {
        let head = &raw mut sessions;
        assert!((*head).storage.is_none());
        let mut first = node(c"a");
        let mut second = node(c"m");
        first.id = 42;
        let first_ptr = &mut *first as *mut session;
        let second_ptr = &mut *second as *mut session;
        assert!(sessions_insert(head, first_ptr).is_null());
        assert!(sessions_insert(head, second_ptr).is_null());
        assert_eq!(session_find(c"a".as_ptr()), first_ptr);
        assert_eq!(session_find_by_id(42), first_ptr);
        assert_eq!(sessions_next(&*first_ptr), second_ptr);
        // Same remove/change/reinsert sequence used by rename-session.
        assert_eq!(sessions_remove(head, first_ptr), first_ptr);
        first.name = ::std::ffi::CStr::from_ptr(c"z".as_ptr().cast_mut()).to_owned();
        assert!(sessions_insert(head, first_ptr).is_null());
        assert!(session_find(c"a".as_ptr()).is_null());
        assert_eq!(session_find(c"z".as_ptr()), first_ptr);
        assert_eq!(session_find_by_id(42), first_ptr);
        assert_eq!(sessions_minmax(&*head, -1), second_ptr);
        assert_eq!(sessions_next(&*second_ptr), first_ptr);
        assert_eq!(sessions_prev(&*first_ptr), second_ptr);
        assert_eq!(sessions_remove(head, first_ptr), first_ptr);
        assert_eq!(sessions_remove(head, second_ptr), second_ptr);
        assert!((*head).storage.is_none());
    }
}

#[test]
fn saved_name_survives_removal_of_current_successor_and_entire_index() {
    unsafe {
        let mut head = sessions { storage: None };
        let mut first = node(c"a1");
        let mut second = node(c"a2");
        let mut last = node(c"b1");
        sessions_insert(&mut head, &mut *first);
        sessions_insert(&mut head, &mut *second);
        sessions_insert(&mut head, &mut *last);
        let name = CStr::from_ptr((first.name).as_ptr().cast_mut())
            .to_bytes()
            .to_vec();
        assert_eq!(sessions_next(&mut *first), &mut *second as *mut _);

        // Group destruction removes the current session AND its cached successor.
        sessions_remove(&mut head, &mut *first);
        sessions_remove(&mut head, &mut *second);
        drop(first);
        drop(second);
        assert_eq!(sessions_after(&head, &name), &mut *last as *mut _);

        let last_name = CStr::from_ptr((last.name).as_ptr().cast_mut())
            .to_bytes()
            .to_vec();
        sessions_remove(&mut head, &mut *last);
        drop(last);
        assert!(head.storage.is_none());
        assert!(sessions_after(&head, &last_name).is_null());

        // Resuming reads the head afresh even after its previous map was freed.
        let mut replacement = node(c"z1");
        sessions_insert(&mut head, &mut *replacement);
        assert_eq!(
            sessions_after(&head, &last_name),
            &mut *replacement as *mut _
        );
        sessions_remove(&mut head, &mut *replacement);
    }
}

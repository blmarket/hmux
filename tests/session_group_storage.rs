use hmux2::src::session::*;
use std::{ffi::CStr, ptr::null_mut};

fn group(name: &CStr) -> Box<session_group> {
    Box::new(session_group {
        name: name.as_ptr(),
        sessions: session_group_sessions {
            tqh_first: null_mut(),
            tqh_last: null_mut(),
        },
        entry: session_group_entry { owner: null_mut() },
    })
}

#[test]
fn byte_order_duplicates_neighbors_and_removal() {
    unsafe {
        let mut head = session_groups { storage: None };
        assert!(session_groups_minmax(&mut head, -1).is_null());
        let mut nodes: Vec<_> = [c"z", c"\xff", c"a", c""].into_iter().map(group).collect();
        for node in &mut nodes {
            assert!(session_groups_insert(&mut head, &mut **node).is_null());
        }
        // Moving the tree head must preserve entry-only traversal.
        let mut head = Box::new(head);
        let mut duplicate = group(c"a");
        let a = &mut *nodes[2] as *mut session_group;
        assert_eq!(session_groups_insert(&mut *head, &mut *duplicate), a);
        assert!(duplicate.entry.owner.is_null());
        assert!(session_groups_remove(&mut *head, &mut *duplicate).is_null());
        assert_eq!(session_groups_find(&mut *head, &mut *duplicate), a);
        let mut missing = group(c"b");
        assert!(session_groups_find(&mut *head, &mut *missing).is_null());
        assert_eq!(
            session_groups_nfind(&mut *head, &mut *missing),
            &mut *nodes[0] as *mut session_group
        );
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
        // Cache the successor before removing the current record, as callers do.
        node = session_groups_minmax(&mut *head, -1);
        while !node.is_null() {
            let next = session_groups_next(node);
            assert_eq!(session_groups_remove(&mut *head, node), node);
            assert!((*node).entry.owner.is_null());
            node = next;
        }
        assert!(head.storage.is_none());
        assert!(session_groups_insert(&mut *head, a).is_null());
        assert_eq!(session_groups_remove(&mut *head, a), a);
        assert!(head.storage.is_none());
    }
}

#[test]
fn named_group_reuses_stable_allocation() {
    unsafe {
        let head = &raw mut session_groups;
        assert!((*head).storage.is_none());
        let first = session_group_new(c"group".as_ptr());
        assert_eq!(session_group_new(c"group".as_ptr()), first);
        assert_eq!(session_group_find(c"group".as_ptr()), first);
        assert!(session_group_find(c"missing".as_ptr()).is_null());
        assert_eq!(
            (*first).sessions.tqh_last,
            &raw mut (*first).sessions.tqh_first
        );
        assert_eq!(session_groups_remove(head, first), first);
        libc::free((*first).name as *mut libc::c_void);
        libc::free(first.cast());
        assert!((*head).storage.is_none());
    }
}

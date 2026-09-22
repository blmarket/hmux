use hmux2::src::session as s;
use std::ffi::{CStr, CString};

#[test]
fn session_name_order_rename_and_group_lifetime() {
    unsafe {
        let names: Vec<_> = [b"z".as_slice(), b"a", b"\xff", b"\x80"]
            .into_iter()
            .map(|name| CString::new(name).unwrap())
            .collect();
        let mut nodes: Vec<s::session> = (0..names.len()).map(|_| std::mem::zeroed()).collect();
        for (id, (node, name)) in nodes.iter_mut().zip(&names).enumerate() {
            node.id = id as u32;
            node.name = name.as_ptr().cast_mut();
            assert!(s::sessions_insert(&raw mut s::sessions, node).is_null());
            assert_eq!(s::session_find(name.as_ptr()), node as *mut _);
            assert_eq!(s::session_find_by_id(id as u32), node as *mut _);
        }
        let mut next = s::sessions_minmax(&raw mut s::sessions, -1);
        for expected in [b"a".as_slice(), b"z", b"\x80", b"\xff"] {
            assert_eq!(CStr::from_ptr((*next).name).to_bytes(), expected);
            next = s::sessions_next(next);
        }
        assert!(next.is_null());
        let mut probe: s::session = std::mem::zeroed();
        probe.name = c"a".as_ptr().cast_mut();
        assert_eq!(
            s::sessions_insert(&raw mut s::sessions, &mut probe),
            &mut nodes[1] as *mut _
        );
        assert!(probe.entry.owner.is_null());
        // Rename follows the command path: remove before changing the indexed name.
        let renamed = &mut nodes[0] as *mut s::session;
        s::sessions_remove(&raw mut s::sessions, renamed);
        (*renamed).name = c"b".as_ptr().cast_mut();
        s::sessions_insert(&raw mut s::sessions, renamed);
        assert!(s::session_find(c"z".as_ptr()).is_null());
        assert_eq!(s::session_find(c"b".as_ptr()), renamed);
        assert_eq!(s::sessions_next(&mut nodes[1]), renamed);
        assert_eq!(s::sessions_prev(renamed), &mut nodes[1] as *mut _);
        for node in &mut nodes {
            s::sessions_remove(&raw mut s::sessions, node);
        }
        assert!(s::sessions.storage.is_null());
        assert_eq!(s::session_alive(renamed), 0);
        let z = s::session_group_new(c"z".as_ptr());
        let a = s::session_group_new(c"a".as_ptr());
        assert_eq!(s::session_group_new(c"a".as_ptr()), a);
        assert_eq!(s::session_group_find(c"z".as_ptr()), z);
        assert_eq!(s::session_groups_minmax(&raw mut s::session_groups, -1), a);
        assert_eq!(s::session_groups_next(a), z);
        for group in [a, z] {
            s::session_groups_remove(&raw mut s::session_groups, group);
            libc::free((*group).name.cast_mut().cast());
            libc::free(group.cast());
        }
        assert!(s::session_groups.storage.is_null());
    }
}

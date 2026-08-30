use super::*;
use crate::session::session_name;
use crate::session::{
    session_group_add, session_group_new, session_group_remove, session_groups, sessions,
};
use crate::tests::test_fixtures::{Args, Registry, Session, Window, globals, link, unlink};

/// An empty session group sitting in the server's group tree for the
/// length of a test, so that [`session_group_contains`] finds it. It is made
/// and joined through the real `session_group_*` calls.
struct Group(*mut session_group);

impl Group {
    fn new() -> Group {
        assert!(
            session_groups.map().is_empty(),
            "the group tree is not empty"
        );
        Group(unsafe { session_group_new(c"kill-session-group".as_ptr()) })
    }

    fn ptr(&mut self) -> *mut session_group {
        self.0
    }
}

impl Drop for Group {
    fn drop(&mut self) {
        session_groups.map().remove(c"kill-session-group");
    }
}

#[test]
fn windows_of_hands_over_the_sessions_winlinks_in_index_order() {
    let _guard = globals();
    let mut s = Session::new(50, "walked");
    let mut first = Window::new(51, "first", 80, 24);
    let mut second = Window::new(52, "second", 80, 24);
    let mut third = Window::new(53, "third", 80, 24);
    let wl2 = link(&mut s, &mut third, 2);
    let wl0 = link(&mut s, &mut first, 0);
    let wl1 = link(&mut s, &mut second, 1);

    assert_eq!(
        windows_of(s.ptr()).collect::<Vec<_>>(),
        vec![wl0, wl1, wl2],
        "the tree is walked by index, not by the order they were linked"
    );

    for wl in [wl0, wl1, wl2] {
        unlink(&mut s, wl);
    }
    assert_eq!(
        windows_of(s.ptr()).count(),
        0,
        "an empty session walks to nothing"
    );
}

#[test]
fn each_session_hands_over_every_session_and_survives_one_leaving_mid_walk() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut ay = Session::new(60, "a");
    let mut bee = Session::new(61, "b");
    let mut cee = Session::new(62, "c");
    registry.add_session(&mut bee);
    registry.add_session(&mut cee);
    registry.add_session(&mut ay);
    unsafe {
        let mut walked = Vec::new();
        for s in each_session() {
            walked.push(s);
            sessions
                .map()
                .remove(::core::ffi::CStr::from_ptr(session_name(s)));
        }

        assert_eq!(
            walked,
            vec![ay.ptr(), bee.ptr(), cee.ptr()],
            "the tree is keyed by name, and taking each session out as it \
             arrives loses none of the ones behind it"
        );
        assert!(sessions.map().is_empty());
    }
}

#[test]
fn members_of_hands_over_every_member_and_survives_one_leaving_mid_walk() {
    let _guard = globals();
    let mut one = Session::new(63, "one");
    let mut two = Session::new(64, "two");
    let mut three = Session::new(65, "three");
    let mut group = Group::new();
    unsafe {
        session_group_add(group.ptr(), one.ptr());
        session_group_add(group.ptr(), two.ptr());
        session_group_add(group.ptr(), three.ptr());

        let mut walked = Vec::new();
        for s in members_of(group.ptr()) {
            walked.push(s);
            session_group_remove(s);
        }

        assert_eq!(
            walked,
            vec![one.ptr(), two.ptr(), three.ptr()],
            "members arrive in the order they joined, and taking one out of \
             the group as it arrives loses none of the ones behind it"
        );
    }
}

#[test]
fn asked_group_answers_only_under_g_and_only_for_a_session_in_one() {
    let _guard = globals();
    let mut joined = Session::new(66, "joined");
    let mut solo = Session::new(67, "solo");
    let mut group = Group::new();
    unsafe {
        session_group_add(group.ptr(), joined.ptr());

        let plain = Args::parse(c"kill-session");
        let flagged = Args::parse(c"kill-session -g");

        assert_eq!(
            asked_group(&*plain.ptr(), joined.ptr()),
            None,
            "without -g the group is never looked for"
        );
        assert_eq!(
            asked_group(&*flagged.ptr(), joined.ptr()),
            Some(group.ptr())
        );
        assert_eq!(
            asked_group(&*flagged.ptr(), solo.ptr()),
            None,
            "-g on a session in no group falls through to the plain kill"
        );
    }
}

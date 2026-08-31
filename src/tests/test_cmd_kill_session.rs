use super::*;
use crate::session::{
    session_group_add, session_group_new, session_group_remove, session_groups,
    session_registry_remove, sessions_empty,
};
use crate::tests::test_fixtures::{Args, Registry, Session, globals};

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
fn each_session_hands_over_every_session_and_survives_one_leaving_mid_walk() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut ay = Session::new(60, "a");
    let mut bee = Session::new(61, "b");
    let mut cee = Session::new(62, "c");
    registry.add_session(&mut bee);
    registry.add_session(&mut cee);
    registry.add_session(&mut ay);
    let mut walked = Vec::new();
    for s in each_session() {
        walked.push(s.as_ptr());
        session_registry_remove(s.as_ptr());
    }

    assert_eq!(
        walked,
        vec![ay.ptr(), bee.ptr(), cee.ptr()],
        "the tree is keyed by name, and taking each session out as it \
         arrives loses none of the ones behind it"
    );
    assert!(sessions_empty());
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

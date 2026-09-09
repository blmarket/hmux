use super::*;
use crate::session::{
    session_group_ensure, session_group_registry_remove, session_groups_empty,
    session_registry_remove, sessions_empty,
};
use crate::tests::test_fixtures::{Args, Registry, Session, globals};

/// An empty session group held in the registry for the length of a test.
struct Group;

impl Group {
    fn new() -> Group {
        assert!(session_groups_empty(), "the group tree is not empty");
        session_group_ensure(c"kill-session-group");
        Group
    }

    fn add(&self, session: &session) {
        if let Some(session) = crate::session::session_ref_of(session) {
            session.join_group(c"kill-session-group");
        };
    }
}

impl Drop for Group {
    fn drop(&mut self) {
        session_group_registry_remove(c"kill-session-group");
    }
}

#[test]
fn session_registry_walk_survives_one_leaving_mid_walk() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut ay = Session::new(60, "a");
    let mut bee = Session::new(61, "b");
    let mut cee = Session::new(62, "c");
    registry.add_session(&mut bee);
    registry.add_session(&mut cee);
    registry.add_session(&mut ay);
    let mut walked = Vec::new();
    for s in SESSIONS.walk_safe() {
        session_registry_remove(unsafe { s.as_session() });
        walked.push(s);
    }

    assert_eq!(walked.len(), 3);
    for (member, expected) in walked
        .iter()
        .zip([ay.reference(), bee.reference(), cee.reference()])
    {
        assert!(
            member.ptr_eq(&expected),
            "sessions remain in name order after removal"
        );
    }
    assert!(sessions_empty());
}

#[test]
fn group_members_survive_one_leaving_mid_walk() {
    let _guard = globals();
    let one = Session::new(63, "one");
    let two = Session::new(64, "two");
    let three = Session::new(65, "three");
    let group = Group::new();
    let one = one.reference();
    let two = two.reference();
    let three = three.reference();
    unsafe {
        group.add(one.as_session());
        group.add(two.as_session());
        group.add(three.as_session());

        let mut walked = Vec::new();
        for s in one.group_walk_safe().unwrap() {
            s.leave_group();
            walked.push(s);
        }

        assert_eq!(walked.len(), 3);
        for (member, expected) in walked.iter().zip([one, two, three]) {
            assert!(
                member.ptr_eq(&expected),
                "members remain in join order after removal"
            );
        }
    }
}

#[test]
fn asked_group_answers_only_under_g_and_only_for_a_session_in_one() {
    let _guard = globals();
    let joined = Session::new(66, "joined");
    let solo = Session::new(67, "solo");
    let group = Group::new();
    let joined = joined.reference();
    let solo = solo.reference();
    unsafe {
        group.add(joined.as_session());

        let plain = Args::parse(c"kill-session");
        let flagged = Args::parse(c"kill-session -g");

        assert!(
            asked_group(&plain.borrow(), &joined).is_none(),
            "without -g the group is never looked for"
        );
        let members: Vec<_> = asked_group(&flagged.borrow(), &joined).unwrap().collect();
        assert_eq!(members.len(), 1);
        assert!(members[0].ptr_eq(&joined));
        assert!(
            asked_group(&flagged.borrow(), &solo).is_none(),
            "-g on a session in no group falls through to the plain kill"
        );
    }
}

use hmux2::src::session::{
    session_next_session, session_previous_session, sessions, sessions_insert, sessions_remove,
};
use hmux2::src::shared::session::session;
use hmux2::src::shared::sort::{SORT_NAME, sort_criteria};
use hmux2::src::sort::sort_get_sessions;
use std::rc::Rc;

#[test]
fn sorted_and_adjacent_results_survive_session_removal() {
    unsafe {
        let head = &raw mut sessions;
        let first = session::new();
        let last = session::new();
        (*first.get()).name = c"a".to_owned();
        (*last.get()).name = c"z".to_owned();
        let first_weak = Rc::downgrade(&first);
        let last_weak = Rc::downgrade(&last);
        sessions_insert(&mut *head, last.clone());
        sessions_insert(&mut *head, first.clone());
        let criteria = sort_criteria {
            order: SORT_NAME,
            reversed: 0,
            order_seq: &[],
        };
        let sorted = sort_get_sessions(&criteria);
        assert!(Rc::ptr_eq(&sorted[0], &first));
        assert!(Rc::ptr_eq(&sorted[1], &last));
        let next = session_next_session(Some(&*last.get()), &criteria).unwrap();
        let previous = session_previous_session(Some(&*first.get()), &criteria).unwrap();
        assert!(Rc::ptr_eq(&next, &first));
        assert!(Rc::ptr_eq(&previous, &last));
        sessions_remove(&mut *head, &first);
        assert!(session_next_session(Some(&*first.get()), &criteria).is_none());
        assert!(Rc::ptr_eq(
            &session_previous_session(Some(&*last.get()), &criteria).unwrap(),
            &last,
        ));
        sessions_remove(&mut *head, &last);
        drop(first);
        drop(last);
        assert_eq!((*sorted[0].get()).name.as_c_str(), c"a");
        drop(sorted);
        assert!(first_weak.upgrade().is_some());
        assert!(last_weak.upgrade().is_some());
        drop(next);
        drop(previous);
        assert!(first_weak.upgrade().is_none());
        assert!(last_weak.upgrade().is_none());
        assert!(session_next_session(None, &criteria).is_none());
    }
}

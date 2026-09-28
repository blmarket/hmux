use hmux2::src::window::*;
use std::rc::Rc;

#[test]
fn global_index_observes_windows_and_lookups_retain_them() {
    unsafe {
        let head = &raw mut windows;
        assert!((*head).storage.is_none());
        let first = window::new();
        let second = window::new();
        (*first.get()).id = 123;
        (*second.get()).id = 125;
        let first_weak = Rc::downgrade(&first);
        let second_weak = Rc::downgrade(&second);
        assert!(windows_insert(head, &first).is_none());
        assert!(windows_insert(head, &second).is_none());
        assert_eq!(Rc::strong_count(&first), 1, "index must not own windows");
        let retained = window_find_by_id(123).unwrap();
        assert_eq!(retained.get(), first.get());
        assert!(window_find_by_id(124).is_none());
        let minimum = windows_minmax(&*head).unwrap();
        assert_eq!(minimum.get(), first.get());
        window_remove_ref(minimum.get(), c"index minimum".as_ptr(), || minimum);
        let next = windows_next(&*first.get()).unwrap();
        assert_eq!(next.get(), second.get());
        window_remove_ref(next.get(), c"index successor".as_ptr(), || next);
        assert!(windows_next(&*second.get()).is_none());

        let duplicate = window::new();
        (*duplicate.get()).id = 123;
        let existing = windows_insert(head, &duplicate).unwrap();
        assert_eq!(existing.get(), first.get());
        window_remove_ref(existing.get(), c"duplicate lookup".as_ptr(), || existing);
        assert!((*duplicate.get()).entry.owner.is_empty());
        assert!(windows_remove(head, duplicate.get()).is_null());
        window_remove_ref(duplicate.get(), c"duplicate window".as_ptr(), || duplicate);

        let mut other = hmux2::src::shared::window::windows { storage: None };
        assert!(windows_remove(&mut other, first.get()).is_null());
        assert_eq!(windows_remove(head, second.get()), second.get());
        assert!(window_find_by_id(125).is_none());
        assert!(windows_next(&*first.get()).is_none());
        assert!(
            second_weak.upgrade().is_some(),
            "removal must not destroy window"
        );
        window_remove_ref(second.get(), c"removed window".as_ptr(), || second);
        assert!(second_weak.upgrade().is_none());

        window_remove_ref(first.get(), c"indexed window".as_ptr(), || first);
        assert!(first_weak.upgrade().is_some(), "lookup must retain window");
        window_remove_ref(retained.get(), c"retained lookup".as_ptr(), || retained);
        assert!(first_weak.upgrade().is_none());
        assert!(
            (*head).storage.is_none(),
            "final drop must unlink expired Weak"
        );
        assert!(window_find_by_id(123).is_none());
    }
}

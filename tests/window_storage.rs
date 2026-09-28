use hmux2::src::shared::window::WindowOwner;
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
        assert_eq!(retained.as_ptr(), first.get());
        assert!(window_find_by_id(124).is_none());
        assert_eq!(windows_minmax(&*head).unwrap().as_ptr(), first.get());
        assert_eq!(windows_next(&*first.get()).unwrap().as_ptr(), second.get());
        assert!(windows_next(&*second.get()).is_none());

        let duplicate = window::new();
        (*duplicate.get()).id = 123;
        assert_eq!(
            windows_insert(head, &duplicate).unwrap().as_ptr(),
            first.get()
        );
        assert!((*duplicate.get()).entry.owner.is_empty());
        assert!(windows_remove(head, duplicate.get()).is_null());
        drop(duplicate);

        let mut other = hmux2::src::shared::window::windows { storage: None };
        assert!(windows_remove(&mut other, first.get()).is_null());
        assert_eq!(windows_remove(head, second.get()), second.get());
        assert!(window_find_by_id(125).is_none());
        assert!(windows_next(&*first.get()).is_none());
        assert!(
            second_weak.upgrade().is_some(),
            "removal must not destroy window"
        );
        drop(WindowOwner::adopt(second));
        assert!(second_weak.upgrade().is_none());

        drop(first);
        assert!(first_weak.upgrade().is_some(), "lookup must retain window");
        drop(retained);
        assert!(first_weak.upgrade().is_none());
        assert!(
            (*head).storage.is_none(),
            "final drop must unlink expired Weak"
        );
        assert!(window_find_by_id(123).is_none());
    }
}

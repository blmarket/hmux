#[path = "support/window_fixture.rs"]
mod window_fixture;
use hmux2::src::window::*;
use std::rc::Rc;
use window_fixture::WindowOptions;

#[test]
fn global_index_observes_windows_and_lookups_retain_them() {
    unsafe {
        let head = &raw mut windows;
        assert!((*head).storage.is_none());
        let options = WindowOptions::new();
        let first = options.create(80, 24);
        let second = options.create(80, 24);
        let first_id = first.id();
        let second_id = second.id();
        let first_weak = Rc::downgrade(&first);
        let second_weak = Rc::downgrade(&second);
        let existing = windows_insert(head, &first).unwrap();
        assert!(Rc::ptr_eq(&existing, &first));
        existing.release(c"existing index entry");
        assert_eq!(Rc::strong_count(&first), 1, "index must not own windows");
        let retained = window_find_by_id(first_id).unwrap();
        assert!(Rc::ptr_eq(&retained, &first));
        assert!(window_find_by_id(second_id + 1).is_none());
        let minimum = windows_minmax(&*head).unwrap();
        assert!(Rc::ptr_eq(&minimum, &first));
        window_remove_ref(minimum, c"index minimum".as_ptr());
        let next = first.next_window().unwrap();
        assert!(Rc::ptr_eq(&next, &second));
        window_remove_ref(next, c"index successor".as_ptr());
        assert!(second.next_window().is_none());

        let duplicate = window::new();
        assert_eq!(
            duplicate.id(),
            first_id,
            "the first allocated ID collides with an unregistered default"
        );
        let existing = windows_insert(head, &duplicate).unwrap();
        assert!(Rc::ptr_eq(&existing, &first));
        window_remove_ref(existing, c"duplicate lookup".as_ptr());
        assert!(duplicate.next_window().is_none());
        assert!(!windows_remove(&mut *head, &duplicate));
        window_remove_ref(duplicate, c"duplicate window".as_ptr());

        let mut other = hmux2::src::shared::window::windows { storage: None };
        assert!(!windows_remove(&mut other, &first));
        assert!(windows_remove(&mut *head, &second));
        assert_eq!(
            Rc::strong_count(&second),
            1,
            "removal must not retain window"
        );
        assert!(window_find_by_id(second_id).is_none());
        assert!(first.next_window().is_none());
        assert!(
            second_weak.upgrade().is_some(),
            "removal must not destroy window"
        );
        window_remove_ref(second, c"removed window".as_ptr());
        assert!(second_weak.upgrade().is_none());

        window_remove_ref(first, c"indexed window".as_ptr());
        assert!(first_weak.upgrade().is_some(), "lookup must retain window");
        window_remove_ref(retained, c"retained lookup".as_ptr());
        assert!(first_weak.upgrade().is_none());
        assert!(
            (*head).storage.is_none(),
            "final explicit release must unlink the window"
        );
        assert!(window_find_by_id(first_id).is_none());
        options.free();
    }
}

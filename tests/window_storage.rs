use hmux2::src::window::Window as _;
use hmux2::src::window::WindowIndex as _;
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
        let existing = (&mut *head).insert(&first).unwrap();
        assert!(Rc::ptr_eq(&existing, &first));
        existing.release(c"existing index entry");
        assert_eq!(Rc::strong_count(&first), 1, "index must not own windows");
        let retained = hmux2::src::shared::window::WindowRef::find_by_id(first_id).unwrap();
        assert!(Rc::ptr_eq(&retained, &first));
        assert!(hmux2::src::shared::window::WindowRef::find_by_id(second_id + 1).is_none());
        let minimum = (&*head).first().unwrap();
        assert!(Rc::ptr_eq(&minimum, &first));
        minimum.release(c"index minimum");
        let next = first.next_window().unwrap();
        assert!(Rc::ptr_eq(&next, &second));
        next.release(c"index successor");
        assert!(second.next_window().is_none());

        let duplicate = hmux2::src::shared::window::WindowRef::empty();
        assert_eq!(
            duplicate.id(),
            first_id,
            "the first allocated ID collides with an unregistered default"
        );
        let existing = (&mut *head).insert(&duplicate).unwrap();
        assert!(Rc::ptr_eq(&existing, &first));
        existing.release(c"duplicate lookup");
        assert!(duplicate.next_window().is_none());
        assert!(!(&mut *head).remove(&duplicate));
        duplicate.release(c"duplicate window");

        let mut other = hmux2::src::shared::window::windows { storage: None };
        assert!(!(&mut other).remove(&first));
        assert!((&mut *head).remove(&second));
        assert_eq!(
            Rc::strong_count(&second),
            1,
            "removal must not retain window"
        );
        assert!(hmux2::src::shared::window::WindowRef::find_by_id(second_id).is_none());
        assert!(first.next_window().is_none());
        assert!(
            second_weak.upgrade().is_some(),
            "removal must not destroy window"
        );
        second.release(c"removed window");
        assert!(second_weak.upgrade().is_none());

        first.release(c"indexed window");
        assert!(first_weak.upgrade().is_some(), "lookup must retain window");
        retained.release(c"retained lookup");
        assert!(first_weak.upgrade().is_none());
        assert!(
            (*head).storage.is_none(),
            "final explicit release must unlink the window"
        );
        assert!(hmux2::src::shared::window::WindowRef::find_by_id(first_id).is_none());
        options.free();
    }
}

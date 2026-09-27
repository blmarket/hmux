use hmux2::src::session::*;
use hmux2::src::shared::session::{session, sessions};
use std::ffi::CStr;

fn node(name: &CStr) -> std::rc::Rc<std::cell::UnsafeCell<session>> {
    let node = session::new();
    unsafe { (*node.get()).name = name.to_owned(); }
    node
}

#[test]
fn saved_name_survives_removal_of_current_successor_and_entire_index() {
    unsafe {
        let mut head = sessions { storage: None };
        let first = node(c"a1");
        let second = node(c"a2");
        let last = node(c"b1");
        sessions_insert(&mut head, first.clone());
        sessions_insert(&mut head, second.clone());
        sessions_insert(&mut head, last.clone());
        let name = CStr::from_ptr(((*first.get()).name).as_ptr().cast_mut())
            .to_bytes()
            .to_vec();
        assert_eq!(sessions_next(&mut *first.get()).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()), &mut *second.get() as *mut _);

        // Group destruction removes the current session AND its cached successor.
        sessions_remove(&mut head, &mut *first.get());
        sessions_remove(&mut head, &mut *second.get());
        drop(first);
        drop(second);
        assert_eq!(sessions_after(&head, &name).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()), &mut *last.get() as *mut _);

        let last_name = CStr::from_ptr(((*last.get()).name).as_ptr().cast_mut())
            .to_bytes()
            .to_vec();
        sessions_remove(&mut head, &mut *last.get());
        drop(last);
        assert!(head.storage.is_none());
        assert!(sessions_after(&head, &last_name).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()).is_null());

        // Resuming reads the head afresh even after its previous map was freed.
        let replacement = node(c"z1");
        sessions_insert(&mut head, replacement.clone());
        assert_eq!(
            sessions_after(&head, &last_name).as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()),
            &mut *replacement.get() as *mut _
        );
        sessions_remove(&mut head, &mut *replacement.get());
    }
}

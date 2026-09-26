use hmux2::src::session::*;
use hmux2::src::shared::session::{session, sessions};
use std::ffi::CStr;

fn node(name: &CStr) -> Box<session> {
    let mut node: Box<session> = Box::new(session::empty());
    node.name = name.to_owned();
    node
}

#[test]
fn saved_name_survives_removal_of_current_successor_and_entire_index() {
    unsafe {
        let mut head = sessions { storage: None };
        let mut first = node(c"a1");
        let mut second = node(c"a2");
        let mut last = node(c"b1");
        sessions_insert(&mut head, &mut *first);
        sessions_insert(&mut head, &mut *second);
        sessions_insert(&mut head, &mut *last);
        let name = CStr::from_ptr((first.name).as_ptr().cast_mut())
            .to_bytes()
            .to_vec();
        assert_eq!(sessions_next(&mut *first), &mut *second as *mut _);

        // Group destruction removes the current session AND its cached successor.
        sessions_remove(&mut head, &mut *first);
        sessions_remove(&mut head, &mut *second);
        drop(first);
        drop(second);
        assert_eq!(sessions_after(&head, &name), &mut *last as *mut _);

        let last_name = CStr::from_ptr((last.name).as_ptr().cast_mut())
            .to_bytes()
            .to_vec();
        sessions_remove(&mut head, &mut *last);
        drop(last);
        assert!(head.storage.is_none());
        assert!(sessions_after(&head, &last_name).is_null());

        // Resuming reads the head afresh even after its previous map was freed.
        let mut replacement = node(c"z1");
        sessions_insert(&mut head, &mut *replacement);
        assert_eq!(
            sessions_after(&head, &last_name),
            &mut *replacement as *mut _
        );
        sessions_remove(&mut head, &mut *replacement);
    }
}

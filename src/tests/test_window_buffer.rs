use super::*;
use crate::paste::{paste_free, paste_set, paste_walk};
use crate::tests::test_fixtures::globals;
use ::std::sync::MutexGuard;

/// A turn at the paste store with no buffers in it, since the store is a
/// global the tests share.
fn store() -> MutexGuard<'static, ()> {
    let guard = globals();
    unsafe {
        let mut pb = paste_walk(::core::ptr::null_mut::<paste_buffer>());
        while !pb.is_null() {
            let next = paste_walk(pb);
            paste_free(pb);
            pb = next;
        }
    }
    guard
}

/// What the buffer called `name` holds, or nothing when there is none.
unsafe fn contents(name: &::core::ffi::CStr) -> Option<Vec<u8>> {
    unsafe {
        let pb = paste_get_name(name.as_ptr());
        if pb.is_null() {
            return None;
        }
        Some(paste_buffer_data(&*pb).to_vec())
    }
}

/// The editor state a close would carry for the buffer called `name`, which
/// must be there.
unsafe fn editing(name: &::core::ffi::CStr) -> Box<window_buffer_editdata> {
    unsafe {
        let pb = paste_get_name(name.as_ptr());
        assert!(!pb.is_null(), "the buffer is there to be edited");
        Box::new(window_buffer_editdata {
            wp_id: u_int::MAX,
            name: Some(name.to_owned()),
            order: paste_buffer_order(&*pb),
        })
    }
}

#[test]
fn what_the_editor_wrote_goes_into_the_buffer_it_was_opened_on() {
    let _guard = store();
    unsafe {
        paste_set(b"original\n".to_vec(), c"edited".as_ptr()).expect("the buffer is set");
        let ed = editing(c"edited");

        window_buffer_edit_close_cb(b"what the editor wrote\n".to_vec(), ed);

        assert_eq!(
            contents(c"edited"),
            Some(b"what the editor wrote\n".to_vec())
        );
    }
}

#[test]
fn a_buffer_replaced_under_the_same_name_keeps_what_replaced_it() {
    let _guard = store();
    unsafe {
        paste_set(b"original\n".to_vec(), c"edited".as_ptr()).expect("the buffer is set");
        let ed = editing(c"edited");
        paste_set(b"somebody else's\n".to_vec(), c"edited".as_ptr()).expect("the buffer is set");

        window_buffer_edit_close_cb(b"what the editor wrote\n".to_vec(), ed);

        assert_eq!(contents(c"edited"), Some(b"somebody else's\n".to_vec()));
    }
}

#[test]
fn a_buffer_that_has_gone_takes_nothing_from_the_editor() {
    let _guard = store();
    unsafe {
        paste_set(b"original\n".to_vec(), c"edited".as_ptr()).expect("the buffer is set");
        let ed = editing(c"edited");
        paste_free(paste_get_name(c"edited".as_ptr()));

        window_buffer_edit_close_cb(b"what the editor wrote\n".to_vec(), ed);

        assert_eq!(contents(c"edited"), None);
    }
}

#[test]
fn an_editor_that_wrote_nothing_leaves_the_buffer_alone() {
    let _guard = store();
    unsafe {
        paste_set(b"original\n".to_vec(), c"edited".as_ptr()).expect("the buffer is set");
        let ed = editing(c"edited");

        window_buffer_edit_close_cb(Vec::new(), ed);

        assert_eq!(contents(c"edited"), Some(b"original\n".to_vec()));
    }
}

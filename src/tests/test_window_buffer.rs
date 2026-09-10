use super::*;
use crate::paste::{PasteBufferStore, with_paste_buffers, with_paste_buffers_mut};
use crate::tests::test_fixtures::globals;

/// A turn at the paste store with no buffers in it, since the store is a
/// global the tests share.
fn store() -> crate::tests::test_fixtures::GlobalsGuard {
    let guard = globals();
    let names = with_paste_buffers(|buffers| {
        buffers
            .buffers()
            .map(|buffer| buffer.name.to_owned())
            .collect::<Vec<_>>()
    });
    for name in names {
        with_paste_buffers_mut(|buffers| buffers.remove(name.as_c_str()));
    }
    guard
}

/// What the buffer called `name` holds, or nothing when there is none.
fn contents(name: &CStr) -> Option<Vec<u8>> {
    with_paste_buffers(|buffers| buffers.get(name).map(|buffer| buffer.data.to_vec()))
}

/// The editor state a close would carry for the buffer called `name`, which
/// must be there.
fn editing(name: &CStr) -> Box<window_buffer_editdata> {
    let order = with_paste_buffers(|buffers| buffers.get(name).map(|buffer| buffer.order))
        .expect("the buffer is there to be edited");
    Box::new(window_buffer_editdata {
        wp_id: u_int::MAX,
        name: Some(name.to_owned()),
        order,
    })
}

#[test]
fn what_the_editor_wrote_goes_into_the_buffer_it_was_opened_on() {
    let _guard = store();
    unsafe {
        with_paste_buffers_mut(|buffers| buffers.set_named(c"edited", b"original\n".to_vec()))
            .expect("the buffer is set");
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
        with_paste_buffers_mut(|buffers| buffers.set_named(c"edited", b"original\n".to_vec()))
            .expect("the buffer is set");
        let ed = editing(c"edited");
        with_paste_buffers_mut(|buffers| {
            buffers.set_named(c"edited", b"somebody else's\n".to_vec())
        })
        .expect("the buffer is set");

        window_buffer_edit_close_cb(b"what the editor wrote\n".to_vec(), ed);

        assert_eq!(contents(c"edited"), Some(b"somebody else's\n".to_vec()));
    }
}

#[test]
fn a_buffer_that_has_gone_takes_nothing_from_the_editor() {
    let _guard = store();
    unsafe {
        with_paste_buffers_mut(|buffers| buffers.set_named(c"edited", b"original\n".to_vec()))
            .expect("the buffer is set");
        let ed = editing(c"edited");
        with_paste_buffers_mut(|buffers| buffers.remove(c"edited"));

        window_buffer_edit_close_cb(b"what the editor wrote\n".to_vec(), ed);

        assert_eq!(contents(c"edited"), None);
    }
}

#[test]
fn an_editor_that_wrote_nothing_leaves_the_buffer_alone() {
    let _guard = store();
    unsafe {
        with_paste_buffers_mut(|buffers| buffers.set_named(c"edited", b"original\n".to_vec()))
            .expect("the buffer is set");
        let ed = editing(c"edited");

        window_buffer_edit_close_cb(Vec::new(), ed);

        assert_eq!(contents(c"edited"), Some(b"original\n".to_vec()));
    }
}

#[test]
fn editor_completion_after_buffer_mode_closes_updates_the_buffer() {
    let _guard = store();
    unsafe {
        let mut target = crate::tests::test_fixtures::Target::new(40, 12);
        let fs = target.state();
        let pane = &mut *target.pane(0);
        with_paste_buffers_mut(|buffers| buffers.set_named(c"edited", b"original\n".to_vec()))
            .expect("the buffer is set");
        assert_eq!(
            crate::window::window_pane_set_mode(pane, None, WindowMode::Buffer, Some(&fs), None),
            0,
        );
        let mut ed = editing(c"edited");
        ed.wp_id = pane.pane_id();
        window_pane_reset_mode(pane);
        assert!(window_pane_current_mode(pane).is_none());

        window_buffer_edit_close_cb(b"updated\n".to_vec(), ed);

        assert_eq!(contents(c"edited"), Some(b"updated\n".to_vec()));
        assert!(window_pane_current_mode(pane).is_none());
        with_paste_buffers_mut(|buffers| buffers.remove(c"edited"));
    }
}

#[test]
fn retained_rows_survive_rebuild_and_close_and_resolve_replaced_buffers_by_name() {
    let _guard = store();
    unsafe {
        let mut target = crate::tests::test_fixtures::Target::new(40, 12);
        let fs = target.state();
        let mut pane = fs
            .window()
            .unwrap()
            .pane_by_id(fs.wp_ref.as_ref().map(|pane| pane.id()).unwrap())
            .unwrap();
        with_paste_buffers_mut(|buffers| buffers.set_named(c"retained", b"original".to_vec()))
            .unwrap();
        assert_eq!(
            crate::window::window_pane_set_mode(
                pane.get_mut().unwrap(),
                None,
                WindowMode::Buffer,
                Some(&fs),
                None
            ),
            0,
        );
        let owner = window_pane_current_mode(pane.get().unwrap())
            .unwrap()
            .state
            .buffer()
            .unwrap();
        let tree = owner.borrow().tree_ref();
        let original = tree.current_item().buffer().unwrap();
        let weak = std::rc::Rc::downgrade(&original);
        with_paste_buffers_mut(|buffers| buffers.set_named(c"retained", b"replacement".to_vec()))
            .unwrap();
        tree.build();
        assert!(!std::rc::Rc::ptr_eq(
            &original,
            &tree.current_item().buffer().unwrap()
        ));
        window_pane_reset_mode(pane.get_mut().unwrap());
        drop(tree);
        drop(owner);
        assert_eq!(original.name(), Some(c"retained"));
        assert_eq!(
            window_buffer_search(
                ModeTreeItemData::Buffer(original.clone()),
                c"replacement",
                0
            ),
            1
        );
        assert_eq!(
            window_buffer_search(ModeTreeItemData::Buffer(original.clone()), c"original", 0),
            0
        );
        with_paste_buffers_mut(|buffers| buffers.remove(c"retained"));
        assert_eq!(
            window_buffer_search(ModeTreeItemData::Buffer(original.clone()), c"retained", 0),
            0
        );
        drop(original);
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn editor_and_menu_callbacks_handle_a_removed_pane() {
    let _guard = store();
    unsafe {
        let mut target = crate::tests::test_fixtures::Target::new(40, 12);
        let fs = target.state();
        let mut pane = fs.pane_ref().unwrap();
        with_paste_buffers_mut(|buffers| buffers.set_named(c"edited", b"original\n".to_vec()))
            .unwrap();
        assert_eq!(
            crate::window::window_pane_set_mode(
                pane.get_mut().unwrap(),
                None,
                WindowMode::Buffer,
                Some(&fs),
                None
            ),
            0
        );
        let data = window_pane_current_mode(pane.get().unwrap())
            .unwrap()
            .state
            .buffer()
            .unwrap();
        let mut editor = editing(c"edited");
        editor.wp_id = pane.id();
        drop(target);
        assert!(pane.get().is_none());
        assert!(data.borrow().pane().is_none());
        let mut client = crate::tests::test_fixtures::zeroed_client();
        data.menu(client.as_client_mut(), b'd' as key_code);
        window_buffer_edit_close_cb(b"updated\n".to_vec(), editor);
        assert_eq!(contents(c"edited"), Some(b"updated\n".to_vec()));
        with_paste_buffers_mut(|buffers| buffers.remove(c"edited"));
    }
}

#[test]
fn row_and_key_formats_use_the_resolved_target_and_drop_invalid_context() {
    let _guard = store();
    unsafe {
        let mut target = crate::tests::test_fixtures::Target::new(40, 12);
        target.add_window(7, 40, 12);
        let mut pane = window_pane_find_by_id(1).unwrap();
        let mut fs = cmd_find_state::default();
        crate::cmd::cmd_find_from_pane(&mut fs, pane.get().unwrap(), 0);
        with_paste_buffers_mut(|buffers| buffers.set_named(c"context", b"text".to_vec())).unwrap();
        assert_eq!(
            crate::window::window_pane_set_mode(
                pane.get_mut().unwrap(),
                None,
                WindowMode::Buffer,
                Some(&fs),
                None
            ),
            0
        );
        let owner = window_pane_current_mode(pane.get().unwrap())
            .unwrap()
            .state
            .buffer()
            .unwrap();
        let tree = owner.borrow().tree_ref();
        {
            let mut data = owner.borrow_mut();
            data.format =
                Some(c"#{session_name}:#{window_index}:#{pane_id}:#{buffer_name}".to_owned());
            data.key_format = Some(c"#{?window_index,#{window_index},b}".to_owned());
        }
        tree.build();
        assert_eq!(
            tree.borrow().children[0].text.as_deref(),
            Some(c"0:7:%1:context")
        );
        assert_eq!(
            (owner.clone()).get_key(tree.current_item(), 0),
            b'7' as key_code
        );
        owner.borrow_mut().fs.wl_idx = Some(99);
        tree.build();
        assert_eq!(
            tree.borrow().children[0].text.as_deref(),
            Some(c":::context")
        );
        assert_eq!(owner.get_key(tree.current_item(), 0), b'b' as key_code);
        with_paste_buffers_mut(|buffers| buffers.remove(c"context"));
    }
}

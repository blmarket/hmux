use crate::grid::Grid as _;
use super::*;
use crate::WindowPane;
use crate::screen::Screen;
use crate::tests::test_fixtures::{Pane, Window, globals, zeroed_client};
use crate::window::window_pane_find_by_id;

fn enable_rebuild(tree: &ModeTreeDataRef) {
    {
        let weak = tree.downgrade();
        let owner = tree.clone();
        owner.borrow_mut().buildcb = Some(std::rc::Rc::new(move |_, _, _| {
            let Some(tree) = weak.upgrade() else {
                return;
            };
            let parent = tree
                .add_item(None, ModeTreeItemData::None, 50, c"parent", None, 0)
                .unwrap();
            tree.add_item(Some(&parent), ModeTreeItemData::None, 51, c"child", None, 0)
                .unwrap();
            for n in 0..8 {
                tree.add_item(None, ModeTreeItemData::None, 100 + n, c"row", None, 0)
                    .unwrap();
            }
        }));
    }
}
unsafe fn tree() -> (Window, ModeTreeDataRef) {
    unsafe { tree_for_id(701) }
}

unsafe fn tree_for_id(id: u_int) -> (Window, ModeTreeDataRef) {
    unsafe {
        let mut window = Window::new(id, "widget", 80, 24);
        let mut pane = Pane::new(id, 80, 24, 100);
        window.add_pane(&mut pane);
        let tree = ModeTreeDataRef::start(
            &mut *pane.ptr(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            WindowModeData::None,
            &[],
        );
        (window, tree)
    }
}

unsafe fn rebuild_lines(tree: &ModeTreeDataRef) {
    unsafe {
        tree.build_lines();
        tree.clone().borrow_mut().height = 3;
    }
}

#[test]
fn item_handles_retain_the_tree_and_reject_removed_or_foreign_parents() {
    let _guard = globals();
    unsafe {
        let (_window, tree) = tree();
        let weak = tree.downgrade();
        let root = tree
            .add_item(None, ModeTreeItemData::None, 1, c"root", None, 1)
            .unwrap();
        let child = tree
            .add_item(Some(&root), ModeTreeItemData::None, 2, c"child", None, 1)
            .unwrap();
        let (_other_window, other) = tree_for_id(702);
        assert!(
            other
                .add_item(Some(&root), ModeTreeItemData::None, 3, c"foreign", None, 1)
                .is_none()
        );
        drop(tree);
        assert!(weak.upgrade().is_some());
        assert_eq!(child.get().unwrap().parent, Some(root.id()));
        root.remove();
        assert!(root.get().is_none());
        assert!(child.get().is_none());
        assert!(root.draw_as_parent().is_none());
        assert!(root.no_tag().is_none());
        assert!(child.set_align(1).is_none());
        assert!(
            root.tree
                .add_item(Some(&root), ModeTreeItemData::None, 4, c"stale", None, 1)
                .is_none()
        );
        drop(root);
        assert!(weak.upgrade().is_some());
        drop(child);
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn borrowed_items_prevent_removal_while_the_tree_screen_remains_accessible() {
    let _guard = globals();
    unsafe {
        let (_window, tree) = tree();
        let item = tree
            .add_item(None, ModeTreeItemData::None, 1, c"retained", None, 0)
            .unwrap();
        let retained = item.get().unwrap();
        let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            item.remove();
        }));
        assert!(attempt.is_err());
        assert_eq!(retained.name.as_deref(), Some(c"retained"));
        drop(retained);
        let mut retained = item.get_mut().unwrap();
        retained.tagged = 1;
        let mut writer = RustScreenWriteCtx::on_shared_screen(tree.screen_handle());
        writer.putc(&grid_default_cell, b'x');
        writer.finish();
        assert_eq!(tree.screen_handle().borrow().cursor(), (1, 0));
        assert_eq!(retained.tagged, 1);
        drop(retained);
        item.remove();
        assert!(item.get().is_none());
    }
}

#[test]
fn rebuilt_items_restore_saved_flags_without_reviving_old_handles() {
    let _guard = globals();
    unsafe {
        let (_window, tree) = tree();
        let original = tree
            .add_item(None, ModeTreeItemData::None, 1, c"root", None, 1)
            .unwrap();
        original.get_mut().unwrap().tagged = 1;
        original.get_mut().unwrap().expanded = 0;
        let mut data = tree.borrow_mut();
        data.saved = core::mem::take(&mut data.children);
        drop(data);
        let replacement = tree
            .add_item(None, ModeTreeItemData::None, 1, c"root", None, 1)
            .unwrap();
        assert_ne!(original.id(), replacement.id());
        assert_eq!(replacement.get().unwrap().tagged, 1);
        assert_eq!(replacement.get().unwrap().expanded, 0);
        assert!(original.get().is_some());
        tree.borrow_mut().saved.clear();
        assert!(original.get().is_none());
        assert!(replacement.get().is_some());
    }
}

#[test]
fn retained_tree_stops_observing_a_destroyed_pane() {
    let _guard = globals();
    unsafe {
        let (window, tree) = tree();
        let observed = tree.borrow().pane().unwrap();
        tree.borrow_mut().buildcb = Some(std::rc::Rc::new(|_, _, _| {
            panic!("a dead pane cannot rebuild")
        }));
        drop(window);
        assert!(observed.get().is_none());
        assert!(tree.borrow().pane().is_none());
        tree.build();
        tree.draw();
        tree.resize(20, 4);
        tree.close();
        let mut client = zeroed_client();
        tree.filter_input(client.as_client_mut(), Some(c"filter"), 1);
        tree.search_input(client.as_client_mut(), Some(c"search"), 1);
        let mut key = b'j' as key_code;
        assert_eq!(tree.key(client.as_client_mut(), &mut key, None), (1, 0, 0));
        assert_eq!(key, KEYC_NONE);
    }
}

#[test]
fn closing_a_mode_detaches_a_retained_tree_from_a_live_pane() {
    let _guard = globals();
    unsafe {
        let (_window, tree) = tree();
        let mut pane = tree.borrow().pane().unwrap();
        let id = pane.id();
        tree.close();
        assert_eq!(window_pane_find_by_id(id).unwrap().id(), id);
        assert!(tree.borrow().pane().is_none());
        *pane.get_mut().unwrap().flags_mut() = 0;
        tree.borrow_mut().buildcb = Some(std::rc::Rc::new(|_, _, _| {
            panic!("a closed mode cannot rebuild")
        }));
        tree.build();
        tree.resize(20, 4);
        tree.draw();
        tree.borrow().redraw_pane();
        assert_eq!(*pane.get_mut().unwrap().flags(), 0);
    }
}
#[test]
fn character_case_helpers_cover_bounds_and_detection() {
    assert_eq!(tolower(b'A'), b'a');
    assert_eq!(toupper(b'z'), b'Z');
    assert_eq!(tolower(0), 0);
    assert_eq!(toupper(0xff), 0xff);
    assert_eq!(mode_tree_is_lowercase(c"alpha 1"), 1);
    assert_eq!(mode_tree_is_lowercase(c"Alpha"), 0);
    let mut error = CString::new("problem").unwrap();
    uppercase_first_byte(&mut error);
    assert_eq!(error.as_bytes(), b"Problem");
    let mut empty = CString::new("").unwrap();
    uppercase_first_byte(&mut empty);
    assert!(empty.is_empty());
}

#[test]
fn append_obeys_strlcat_capacity() {
    let mut dst = b"ab".to_vec();
    mode_tree_append(&mut dst, b"cdef", 5);
    assert_eq!(dst, b"abcd");
    mode_tree_append(&mut dst, b"z", 4);
    assert_eq!(dst, b"abcd");
    mode_tree_append(&mut Vec::new(), b"x", 0);
}

#[test]
fn hierarchy_lookup_and_removal_are_isolated() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        let first = tree
            .add_item(None, ModeTreeItemData::None, 10, c"first", None, 1)
            .unwrap();
        let second = tree
            .add_item(
                None,
                ModeTreeItemData::None,
                20,
                c"second",
                Some(c"text"),
                0,
            )
            .unwrap();
        let child = tree
            .add_item(Some(&first), ModeTreeItemData::None, 11, c"child", None, 0)
            .unwrap();
        rebuild_lines(&tree);
        assert_eq!(tree.borrow().line_list.len(), 3);
        assert_eq!(child.get().unwrap().parent, Some(first.id()));
        assert!(first.get().unwrap().parent.is_none());
        assert_eq!(
            mode_tree_find_item(&tree.borrow().children, 11).map(|item| item.id),
            Some(child.id())
        );
        assert!(mode_tree_find_item(&tree.borrow().children, 999).is_none());
        assert_eq!(
            mode_tree_item_ref(&tree.borrow(), second.id())
                .unwrap()
                .text
                .as_deref(),
            Some(c"text")
        );
        assert_eq!(line_item_ref(&tree.borrow(), 1).unwrap().id, child.id());
        child.remove();
        assert!(mode_tree_item_ref(&tree.borrow(), child.id()).is_none());
    }
}

#[test]
fn line_generation_assigns_depth_shape_and_all_key_classes() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        let parent = tree
            .add_item(None, ModeTreeItemData::None, 1, c"parent", None, 1)
            .unwrap();
        tree.add_item(Some(&parent), ModeTreeItemData::None, 2, c"nested", None, 0)
            .unwrap();
        for n in 2..39u64 {
            let name = CString::new(format!("item-{n}")).unwrap();
            tree.add_item(None, ModeTreeItemData::None, n + 1, &name, None, 0)
                .unwrap();
        }
        rebuild_lines(&tree);
        let state = tree.borrow();
        assert_eq!(state.line_list.len(), 39);
        assert_eq!(state.line_list[1].depth, 1);
        assert_eq!(state.line_list[0].flat, 0);
        assert_eq!(line_item_ref(&state, 0).unwrap().key, b'0' as key_code);
        assert_ne!(
            line_item_ref(&state, 10).unwrap().key & KEYC_META as key_code,
            0
        );
        assert_eq!(line_item_ref(&state, 36).unwrap().key, KEYC_NONE);
        assert_eq!(state.maxdepth, 1);
    }
}

#[test]
fn navigation_selection_and_tag_count_cover_edges() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        let rows: Vec<_> = (0..6)
            .map(|n| {
                tree.add_item(None, ModeTreeItemData::None, 100 + n, c"row", None, 0)
                    .unwrap()
            })
            .collect();
        rebuild_lines(&tree);
        assert_eq!(tree.current_name().as_deref(), Some(c"row"));
        tree.up(1);
        assert_eq!(tree.borrow().current, 5);
        tree.down(1);
        assert_eq!(tree.borrow().current, 0);
        assert_eq!(tree.down(0), 1);
        assert_eq!(tree.set_current(105), 1);
        assert_eq!(tree.borrow().offset, 3);
        assert_eq!(tree.set_current(999), 0);
        rows[1].get_mut().unwrap().tagged = 1;
        rows[4].get_mut().unwrap().tagged = 1;
        assert_eq!(tree.count_tagged(), 2);
        mode_tree_clear_tagged(&mut tree.borrow_mut().children);
        assert_eq!(tree.count_tagged(), 0);
    }
}
#[test]
fn item_flags_and_search_walk_both_directions() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        let alpha = tree
            .add_item(None, ModeTreeItemData::None, 1, c"Alpha", None, 0)
            .unwrap();
        let beta = tree
            .add_item(None, ModeTreeItemData::None, 2, c"beta target", None, 0)
            .unwrap();
        let gamma = tree
            .add_item(None, ModeTreeItemData::None, 3, c"Gamma", None, 0)
            .unwrap();
        alpha.draw_as_parent().unwrap();
        beta.no_tag().unwrap();
        gamma.set_align(7).unwrap();
        assert_eq!(
            (
                alpha.get().unwrap().draw_as_parent,
                beta.get().unwrap().no_tag,
                gamma.get().unwrap().align
            ),
            (1, 1, 7)
        );
        rebuild_lines(&tree);
        tree.borrow_mut().search = Some(c"target".to_owned());
        for direction in [MODE_TREE_SEARCH_FORWARD, MODE_TREE_SEARCH_BACKWARD] {
            assert_eq!(
                tree.search(direction).map(|item| item.id()),
                Some(beta.id())
            );
        }
        tree.borrow_mut().search = Some(c"missing".to_owned());
        assert!(tree.search(MODE_TREE_SEARCH_FORWARD).is_none());
        tree.borrow_mut().search = Some(c"gamma".to_owned());
        tree.borrow_mut().search_icase = 1;
        assert_eq!(
            tree.search(MODE_TREE_SEARCH_FORWARD).map(|item| item.id()),
            Some(gamma.id())
        );
        tree.swap(-1);
        tree.swap(1);
    }
}

#[test]
fn search_wraps_through_collapsed_descendants_and_excludes_the_current_item() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        assert!(tree.search(MODE_TREE_SEARCH_FORWARD).is_none());
        let root = tree
            .add_item(None, ModeTreeItemData::None, 1, c"hit root", None, 0)
            .unwrap();
        let child = tree
            .add_item(
                Some(&root),
                ModeTreeItemData::None,
                2,
                c"hit child",
                None,
                0,
            )
            .unwrap();
        let grandchild = tree
            .add_item(
                Some(&child),
                ModeTreeItemData::None,
                3,
                c"hit grandchild",
                None,
                0,
            )
            .unwrap();
        let sibling = tree
            .add_item(
                Some(&root),
                ModeTreeItemData::None,
                4,
                c"hit sibling",
                None,
                0,
            )
            .unwrap();
        let tail = tree
            .add_item(None, ModeTreeItemData::None, 5, c"hit tail", None, 0)
            .unwrap();
        rebuild_lines(&tree);
        tree.borrow_mut().search = Some(c"hit".to_owned());
        assert_eq!(tree.borrow().line_list.len(), 2);
        assert_eq!(
            tree.search(MODE_TREE_SEARCH_FORWARD).unwrap().id(),
            child.id()
        );
        tree.set_current(5);
        assert_eq!(
            tree.search(MODE_TREE_SEARCH_BACKWARD).unwrap().id(),
            sibling.id()
        );
        root.get_mut().unwrap().expanded = 1;
        child.get_mut().unwrap().expanded = 1;
        rebuild_lines(&tree);
        let rows = [&root, &child, &grandchild, &sibling, &tail];
        for (at, row) in rows.iter().enumerate() {
            let tag = row.get().unwrap().tag;
            tree.set_current(tag);
            assert_eq!(
                tree.search(MODE_TREE_SEARCH_FORWARD).unwrap().id(),
                rows[(at + 1) % rows.len()].id()
            );
            assert_eq!(
                tree.search(MODE_TREE_SEARCH_BACKWARD).unwrap().id(),
                rows[(at + rows.len() - 1) % rows.len()].id()
            );
        }
        tree.borrow_mut().search = Some(c"tail".to_owned());
        for direction in [MODE_TREE_SEARCH_FORWARD, MODE_TREE_SEARCH_BACKWARD] {
            assert!(tree.search(direction).is_none());
        }
    }
}

#[test]
fn search_callbacks_can_remove_items_and_new_items_wait_for_the_next_search() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        let root = tree
            .add_item(None, ModeTreeItemData::None, 1, c"root", None, 0)
            .unwrap();
        let removed = tree
            .add_item(None, ModeTreeItemData::None, 2, c"removed", None, 0)
            .unwrap();
        let survivor = tree
            .add_item(None, ModeTreeItemData::None, 3, c"survivor", None, 0)
            .unwrap();
        rebuild_lines(&tree);
        tree.borrow_mut().search = Some(c"match".to_owned());
        let weak = tree.downgrade();
        let root_id = root.id();
        let removed_id = removed.id();
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let seen = calls.clone();
        tree.borrow_mut().searchcb = Some(std::rc::Rc::new(move |_, _, _| {
            seen.set(seen.get() + 1);
            if seen.get() == 1 {
                let tree = weak.upgrade().unwrap();
                (ModeTreeItemRef {
                    tree: tree.clone(),
                    id: root_id,
                })
                .remove();
                (ModeTreeItemRef {
                    tree: tree.clone(),
                    id: removed_id,
                })
                .remove();
                tree.add_item(None, ModeTreeItemData::None, 4, c"new", None, 0)
                    .unwrap();
                return 1;
            }
            0
        }));
        assert!(tree.search(MODE_TREE_SEARCH_FORWARD).is_none());
        assert_eq!(calls.get(), 2);
        assert!(root.get().is_none());
        assert!(removed.get().is_none());
        assert!(survivor.get().is_some());
        rebuild_lines(&tree);
        tree.borrow_mut().current = 0;
        calls.set(0);
        tree.borrow_mut().searchcb = Some(std::rc::Rc::new(|_, _, _| 1));
        assert_eq!(
            tree.search(MODE_TREE_SEARCH_FORWARD)
                .unwrap()
                .get()
                .unwrap()
                .name
                .as_deref(),
            Some(c"new")
        );
    }
}
#[test]
fn expand_collapse_and_tag_lookup_cover_present_missing_and_empty_trees() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        let parent = tree
            .add_item(None, ModeTreeItemData::None, 50, c"parent", None, 0)
            .expect("a widget row can be inserted");
        tree.add_item(Some(&parent), ModeTreeItemData::None, 51, c"child", None, 0)
            .expect("a widget row can be inserted");
        rebuild_lines(&tree);
        enable_rebuild(&tree);
        assert_eq!(line_count(&tree.borrow()), 1);
        tree.expand_current();
        assert_eq!(line_count(&tree.borrow()), 10);
        tree.expand_current();
        tree.collapse_current();
        assert_eq!(line_count(&tree.borrow()), 9);
        tree.collapse_current();
        tree.expand(50);
        assert_eq!(line_count(&tree.borrow()), 10);
        tree.expand(999);
        assert_eq!(tree.set_current(51), 1);
        assert_eq!(tree.borrow().current, 1);

        mode_tree_clear_lines(&mut tree.borrow_mut());
        tree.expand_current();
        tree.collapse_current();
        tree.expand(50);
        assert!(matches!(tree.current_item(), ModeTreeItemData::None));
        assert_eq!(tree.set_current(999), 0);
        tree.up(1);
        assert_eq!(tree.down(1), 0);
    }
}

const EACH_COUNT: crate::server_state::LocalField<std::cell::Cell<usize>> =
    crate::server_state::LocalField::new(|state| &state.widget_test_each_count);

fn count_each(_modedata: WindowModeData, _itemdata: ModeTreeItemData) {
    EACH_COUNT.set(EACH_COUNT.get() + 1);
}

#[test]
fn each_tagged_visits_tags_or_current_exactly_as_requested() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        let rows: Vec<_> = (0..4)
            .map(|n| {
                tree.add_item(None, ModeTreeItemData::None, n, c"row", None, 0)
                    .unwrap()
            })
            .collect();
        rebuild_lines(&tree);
        enable_rebuild(&tree);
        rows[0].get_mut().unwrap().tagged = 1;
        rows[3].get_mut().unwrap().tagged = 1;
        EACH_COUNT.set(0);
        tree.each_tagged(|modedata, itemdata| count_each(modedata, itemdata), 1);
        assert_eq!(EACH_COUNT.get(), 2);
        mode_tree_clear_tagged(&mut tree.borrow_mut().children);
        tree.each_tagged(|modedata, itemdata| count_each(modedata, itemdata), 1);
        assert_eq!(EACH_COUNT.get(), 3);
        tree.each_tagged(|modedata, itemdata| count_each(modedata, itemdata), 0);
        assert_eq!(EACH_COUNT.get(), 3);
    }
}
#[test]
fn key_navigation_covers_pages_edges_choice_tags_and_hierarchy() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        let parent = tree
            .add_item(None, ModeTreeItemData::None, 1, c"parent", None, 1)
            .expect("a widget row can be inserted");
        tree.add_item(Some(&parent), ModeTreeItemData::None, 2, c"child", None, 0)
            .expect("a widget row can be inserted");
        for n in 3..12u64 {
            tree.add_item(None, ModeTreeItemData::None, n, c"row", None, 0)
                .expect("a widget row can be inserted");
        }
        rebuild_lines(&tree);
        enable_rebuild(&tree);
        tree.borrow_mut().height = 3;
        tree.borrow_mut().width = 40;
        let mut client = zeroed_client();
        for input in [
            KEYC_DOWN,
            KEYC_UP,
            KEYC_NPAGE,
            KEYC_PPAGE,
            KEYC_END,
            KEYC_HOME,
            KEYC_RIGHT,
            KEYC_LEFT,
            b't' as key_code,
            b'T' as key_code,
            b'1' as key_code,
        ] {
            let mut key = input;
            let (finished, _, _) = tree.key(client.as_client_mut(), &mut key, None);
            assert_eq!(finished, 0);
        }
        assert!(tree.borrow().current < line_count(&tree.borrow()));
        let mut enter = b'\r' as key_code;
        assert!(tree.key(client.as_client_mut(), &mut enter, None).0 <= 1);
        let mut escape = 27 as key_code;
        assert_eq!(tree.key(client.as_client_mut(), &mut escape, None).0, 1);
    }
}

#[test]
fn resize_height_selection_and_free_cover_clamps_and_saved_children() {
    unsafe {
        let _guard = globals();
        let mut window = Window::new(702, "widget", 80, 24);
        let mut pane = Pane::new(703, 80, 24, 100);
        window.add_pane(&mut pane);
        let tree = ModeTreeDataRef::start(
            &mut *pane.ptr(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            WindowModeData::None,
            &[],
        );
        for n in 0..8u64 {
            tree.add_item(None, ModeTreeItemData::None, n, c"row", Some(c"preview"), 0)
                .expect("a widget row can be inserted");
        }
        rebuild_lines(&tree);
        enable_rebuild(&tree);
        tree.borrow_mut().preview = MODE_TREE_PREVIEW_OFF as core::ffi::c_int;
        tree.borrow_mut().current = 7;
        tree.resize(20, 4);
        assert_eq!(RustScreen::grid(&tree.screen_handle().borrow()).width(), 20);
        assert_eq!(RustScreen::grid(&tree.screen_handle().borrow()).height(), 4);
        assert!(tree.borrow().offset <= tree.borrow().current);
        tree.resize(1, 1);
        tree.close();
        assert!(!tree.borrow().children.is_empty());
        assert_ne!(*(*pane.ptr()).flags() & PANE_REDRAW, 0);
    }
}

#[test]
fn current_row_name_outlives_item_removal_and_tree_release() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        assert!(tree.current_name().is_none());
        let item = tree
            .add_item(None, ModeTreeItemData::None, 1, c"retained name", None, 0)
            .unwrap();
        rebuild_lines(&tree);
        let name = tree.current_name().unwrap();
        item.remove();
        assert!(tree.current_name().is_none());
        assert!(matches!(tree.current_item(), ModeTreeItemData::None));
        assert_eq!(tree.count_tagged(), 0);
        drop(item);
        drop(tree);
        assert_eq!(name.as_c_str(), c"retained name");
    }
}

#[test]
fn tagged_callbacks_can_rebuild_the_tree_and_empty_current_is_skipped() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        let mut first = tree
            .add_item(None, ModeTreeItemData::None, 1, c"first", None, 0)
            .unwrap();
        let mut removed = tree
            .add_item(None, ModeTreeItemData::None, 2, c"removed", None, 0)
            .unwrap();
        let mut untagged = tree
            .add_item(None, ModeTreeItemData::None, 3, c"untagged", None, 0)
            .unwrap();
        for item in [&mut first, &mut removed, &mut untagged] {
            item.get_mut().unwrap().tagged = 1;
        }
        rebuild_lines(&tree);
        let mut calls = 0;
        tree.each_tagged(
            |_, _| {
                calls += 1;
                if calls == 1 {
                    removed.remove();
                    untagged.get_mut().unwrap().tagged = 0;
                    let added = tree
                        .add_item(None, ModeTreeItemData::None, 4, c"added", None, 0)
                        .unwrap();
                    added.get_mut().unwrap().tagged = 1;
                    rebuild_lines(&tree);
                }
            },
            1,
        );
        assert_eq!(calls, 2);
        tree.borrow_mut().line_list.clear();
        tree.each_tagged(|_, _| panic!("an empty tree has no current item"), 1);
    }
}

#[test]
fn swaps_skip_descendants_and_borrow_no_tree_state_during_callbacks() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        let payload = |session| {
            ModeTreeItemData::Tree(crate::modes::tree::window_tree_itemdata {
                session,
                ..Default::default()
            })
        };
        let first = tree
            .add_item(None, payload(1), 1, c"first", None, 1)
            .unwrap();
        tree.add_item(Some(&first), payload(2), 2, c"child", None, 0)
            .unwrap();
        tree.add_item(None, payload(3), 3, c"last", None, 0)
            .unwrap();
        rebuild_lines(&tree);
        let weak = tree.downgrade();
        let calls = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let recorded = calls.clone();
        tree.borrow_mut().swapcb = Some(std::rc::Rc::new(move |first, second, criteria| {
            recorded.borrow_mut().push((
                first.tree().unwrap().session,
                second.tree().unwrap().session,
            ));
            let reversed = criteria.reversed();
            weak.upgrade()
                .unwrap()
                .borrow_mut()
                .sort_crit
                .set_reversed(!reversed);
            assert_eq!(criteria.reversed(), reversed);
            0
        }));
        tree.swap(1);
        tree.borrow_mut().current = 1;
        tree.swap(1);
        tree.swap(-1);
        tree.borrow_mut().current = 2;
        tree.swap(-1);
        tree.swap(1);
        assert_eq!(*calls.borrow(), [(1, 3), (3, 1)]);
        tree.borrow_mut().line_list.clear();
        tree.swap(1);
        assert_eq!(calls.borrow().len(), 2);
    }
}

#[test]
fn line_key_callbacks_keep_postorder_and_can_remove_their_item() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        let payload = |session| {
            ModeTreeItemData::Tree(crate::modes::tree::window_tree_itemdata {
                session,
                ..Default::default()
            })
        };
        let root = tree
            .add_item(None, payload(1), 1, c"root", None, 1)
            .unwrap();
        let child = tree
            .add_item(Some(&root), payload(2), 2, c"child", None, 0)
            .unwrap();
        let tail = tree
            .add_item(None, payload(3), 3, c"tail", None, 0)
            .unwrap();
        let child_id = child.id();
        let weak = tree.downgrade();
        let calls = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let recorded = calls.clone();
        tree.borrow_mut().keycb = Some(std::rc::Rc::new(move |item, line| {
            let row = item.tree().unwrap().session;
            recorded.borrow_mut().push((row, line));
            assert_eq!(
                weak.upgrade().unwrap().borrow().line_list[line as usize].last,
                if row == 1 { 0 } else { 1 }
            );
            if row == 2 {
                (ModeTreeItemRef {
                    tree: weak.upgrade().unwrap(),
                    id: child_id,
                })
                .remove();
            }
            if row == 3 {
                KEYC_UNKNOWN
            } else {
                b'r' as key_code
            }
        }));
        rebuild_lines(&tree);
        assert_eq!(*calls.borrow(), [(2, 1), (1, 0), (3, 2)]);
        assert!(child.get().is_none());
        assert_eq!(
            tree.borrow()
                .line_list
                .iter()
                .map(|line| line.item)
                .collect::<Vec<_>>(),
            [root.id(), tail.id()]
        );
        assert_eq!(root.get().unwrap().line, 0);
        assert_eq!(tail.get().unwrap().line, 1);
        assert_eq!(root.get().unwrap().keystr.as_deref(), Some(c"r"));
        assert_eq!(tail.get().unwrap().key, KEYC_NONE);
        assert!(tail.get().unwrap().keystr.is_none());
    }
}

#[test]
fn rebuild_callbacks_own_inputs_and_keep_the_unfiltered_fallback() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        tree.borrow_mut().filter = Some(c"missing".to_owned());
        tree.borrow_mut().sortcb = Some(std::rc::Rc::new(|sort| sort.set_order(SORT_NAME)));
        let weak = tree.downgrade();
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let recorded = calls.clone();
        tree.borrow_mut().buildcb = Some(std::rc::Rc::new(move |sort, tag, filter| {
            recorded.set(recorded.get() + 1);
            let tree = weak.upgrade().unwrap();
            if filter.is_some() {
                tree.borrow_mut().filter = Some(c"replacement".to_owned());
                tree.borrow_mut().sort_crit.set_order(SORT_ACTIVITY);
                *tree.screen_handle().borrow_mut() = RustScreen::new_with_server_options(40, 12, 0);
                assert_eq!(filter, Some(c"missing"));
                assert_eq!(sort.order(), SORT_NAME);
            } else {
                assert_eq!(sort.order(), SORT_ACTIVITY);
                tree.add_item(None, ModeTreeItemData::None, 7, c"fallback", None, 0)
                    .unwrap();
                *tag = 7;
            }
        }));
        let weak = tree.downgrade();
        tree.borrow_mut().heightcb = Some(std::rc::Rc::new(move || {
            let tree = weak.upgrade().unwrap();
            assert_eq!(tree.borrow().width, 40);
            assert_eq!(tree.borrow().children.len(), 1);
            *tree.screen_handle().borrow_mut() = RustScreen::new_with_server_options(40, 16, 0);
            4
        }));
        tree.build();
        assert_eq!(calls.get(), 2);
        assert_eq!(tree.current_name().as_deref(), Some(c"fallback"));
        let state = tree.borrow();
        assert_eq!(state.no_matches, 1);
        assert_eq!(state.filter.as_deref(), Some(c"replacement"));
        assert_eq!((state.width, state.height), (40, 12));
        assert!(state.saved.is_empty());
    }
}

#[test]
fn menu_callbacks_revalidate_owners_and_can_close_the_selected_mode() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        for tag in 1..=2 {
            tree.add_item(None, ModeTreeItemData::None, tag, c"row", None, 0)
                .unwrap();
        }
        rebuild_lines(&tree);
        let client = zeroed_client();
        let weak = tree.downgrade();
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let recorded = calls.clone();
        tree.borrow_mut().menucb = Some(std::rc::Rc::new(move |_, key| {
            let tree = weak.upgrade().unwrap();
            assert_eq!(key, b't' as key_code);
            assert_eq!(tree.borrow().current, 1);
            tree.borrow_mut().filter = Some(c"from menu".to_owned());
            tree.close();
            recorded.set(recorded.get() + 1);
        }));
        let menu = |line| {
            Box::new(mode_tree_menu {
                data: tree.downgrade(),
                c: client.downgrade(),
                line,
            })
        };
        mode_tree_menu_callback(0, KEYC_NONE, menu(0));
        mode_tree_menu_callback(0, b't' as key_code, menu(2));
        assert_eq!(calls.get(), 0);
        mode_tree_menu_callback(0, b't' as key_code, menu(1));
        mode_tree_menu_callback(0, b't' as key_code, menu(0));
        assert_eq!(calls.get(), 1);
        assert_eq!(tree.borrow().filter.as_deref(), Some(c"from menu"));
        let expired_client = menu(0);
        drop(client);
        mode_tree_menu_callback(0, b't' as key_code, expired_client);
        assert_eq!(calls.get(), 1);
    }
}

#[test]
fn tag_changes_preserve_ancestor_exclusion_and_visible_row_rules() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        let root = tree
            .add_item(None, ModeTreeItemData::None, 1, c"root", None, 1)
            .unwrap();
        let child = tree
            .add_item(Some(&root), ModeTreeItemData::None, 2, c"child", None, 1)
            .unwrap();
        let grandchild = tree
            .add_item(
                Some(&child),
                ModeTreeItemData::None,
                3,
                c"grandchild",
                None,
                0,
            )
            .unwrap();
        let group = tree
            .add_item(None, ModeTreeItemData::None, 4, c"group", None, 1)
            .unwrap();
        let entry = tree
            .add_item(Some(&group), ModeTreeItemData::None, 5, c"entry", None, 0)
            .unwrap();
        group.get_mut().unwrap().no_tag = 1;
        entry.get_mut().unwrap().no_tag = 1;
        root.get_mut().unwrap().tagged = 1;
        grandchild.get_mut().unwrap().tagged = 1;
        assert!(mode_tree_toggle_tag(&mut tree.borrow_mut(), child.id()));
        assert_eq!(
            (
                root.get().unwrap().tagged,
                child.get().unwrap().tagged,
                grandchild.get().unwrap().tagged
            ),
            (0, 1, 0)
        );
        assert!(mode_tree_toggle_tag(&mut tree.borrow_mut(), child.id()));
        assert_eq!(child.get().unwrap().tagged, 0);
        assert!(!mode_tree_toggle_tag(&mut tree.borrow_mut(), entry.id()));
        assert!(!mode_tree_toggle_tag(&mut tree.borrow_mut(), u_int::MAX));
        rebuild_lines(&tree);
        mode_tree_set_all_tagged(&mut tree.borrow_mut(), true);
        assert_eq!(
            [&root, &child, &grandchild, &group, &entry].map(|item| item.get().unwrap().tagged),
            [1, 0, 0, 0, 1]
        );
        root.get_mut().unwrap().expanded = 0;
        grandchild.get_mut().unwrap().tagged = 1;
        rebuild_lines(&tree);
        mode_tree_set_all_tagged(&mut tree.borrow_mut(), false);
        assert_eq!(tree.count_tagged(), 0);
        assert_eq!(grandchild.get().unwrap().tagged, 1);
    }
}

#[test]
fn hierarchy_keys_follow_rows_recreated_by_rebuilds() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        enable_rebuild(&tree);
        tree.build();
        let original = ModeTreeItemRef {
            tree: tree.clone(),
            id: tree.borrow().line_list[0].item,
        };
        let mut client = zeroed_client();
        let mut right = KEYC_RIGHT;
        assert_eq!(tree.key(client.as_client_mut(), &mut right, None).0, 0);
        assert!(original.get().is_none());
        assert_eq!(tree.borrow().line_list.len(), 10);
        assert_eq!(tree.current_name().as_deref(), Some(c"parent"));
        assert_eq!(tree.key(client.as_client_mut(), &mut right, None).0, 0);
        assert_eq!(tree.current_name().as_deref(), Some(c"child"));
        let mut left = KEYC_LEFT;
        assert_eq!(tree.key(client.as_client_mut(), &mut left, None).0, 0);
        assert_eq!(tree.current_name().as_deref(), Some(c"parent"));
        assert_eq!(tree.borrow().line_list.len(), 9);
        assert_eq!(tree.borrow().current, 0);
    }
}

#[test]
fn a_key_rebuild_can_remove_all_items_and_detach_the_mode() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        let item = tree
            .add_item(None, ModeTreeItemData::None, 1, c"item", None, 0)
            .unwrap();
        rebuild_lines(&tree);
        let weak = tree.downgrade();
        tree.borrow_mut().buildcb = Some(std::rc::Rc::new(move |_, _, _| {
            let tree = weak.upgrade().unwrap();
            tree.borrow_mut().saved.clear();
            tree.close();
        }));
        let mut client = zeroed_client();
        let mut preview = b'v' as key_code;
        assert_eq!(tree.key(client.as_client_mut(), &mut preview, None).0, 0);
        assert!(item.get().is_none());
        assert!(tree.borrow().pane().is_none());
        let mut down = KEYC_DOWN;
        assert_eq!(tree.key(client.as_client_mut(), &mut down, None), (1, 0, 0));
        assert_eq!(down, KEYC_NONE);
    }
}

#[test]
fn widget_screens_are_retained_independently_of_tree_state() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        let screen = tree.screen_handle().clone();
        let weak_screen = screen.downgrade();
        let weak_tree = tree.downgrade();
        {
            let mut drawing = screen.borrow_mut();
            tree.borrow_mut().filter = Some(c"independent".to_owned());
            screen_resize(&mut drawing, 17, 9, 0);
        }
        drop(tree);
        assert!(weak_tree.upgrade().is_none());
        assert_eq!(
            (
                RustScreen::grid(&screen.borrow()).width(),
                RustScreen::grid(&screen.borrow()).height()
            ),
            (17, 9)
        );
        drop(screen);
        assert!(weak_screen.upgrade().is_none());
    }
}

#[test]
fn drawing_preserves_nested_prefixes_alignment_and_tag_labels() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        let mut root = tree
            .add_item(None, ModeTreeItemData::None, 1, c"root", None, 1)
            .unwrap();
        let mut child = tree
            .add_item(Some(&root), ModeTreeItemData::None, 2, c"child", None, 1)
            .unwrap();
        let mut grandchild = tree
            .add_item(Some(&child), ModeTreeItemData::None, 3, c"grand", None, 1)
            .unwrap();
        let mut leaf = tree
            .add_item(
                Some(&grandchild),
                ModeTreeItemData::None,
                4,
                c"leaf",
                None,
                0,
            )
            .unwrap();
        let mut peer = tree
            .add_item(Some(&root), ModeTreeItemData::None, 5, c"peer", None, 0)
            .unwrap();
        let mut tail = tree
            .add_item(
                None,
                ModeTreeItemData::None,
                6,
                c"tail",
                Some(c"details"),
                0,
            )
            .unwrap();
        for item in [
            &mut root,
            &mut child,
            &mut grandchild,
            &mut leaf,
            &mut peer,
            &mut tail,
        ] {
            item.get_mut().unwrap().align = 1;
        }
        leaf.get_mut().unwrap().tagged = 1;
        rebuild_lines(&tree);
        tree.borrow_mut().width = 50;
        tree.borrow_mut().height = 6;
        tree.borrow_mut().current = 3;
        tree.borrow_mut().preview = MODE_TREE_PREVIEW_OFF as core::ffi::c_int;
        tree.draw();
        let screen = tree.screen_handle().borrow();
        let grid = RustScreen::grid(&screen);
        let rows: Vec<_> = (0..6)
            .map(|y| {
                (grid).string_cells(0, grid.history_size() + y, grid.width(), None, 0, None)
                    .to_string_lossy()
                    .trim_end()
                    .to_owned()
            })
            .collect();
        assert_eq!(
            rows,
            [
                "(0) - root",
                "(1) tq> - child",
                "(2) x   mq> - grand",
                "(3)         mq> leaf*",
                "(4) mq>    peer",
                "(5)   tail: details",
            ]
        );
    }
}

#[test]
fn preview_callbacks_can_read_the_screen_and_release_tree_items() {
    unsafe {
        let _guard = globals();
        let (_pane, tree) = tree();
        let item = tree
            .add_item(None, ModeTreeItemData::None, 1, c"preview", None, 0)
            .unwrap();
        rebuild_lines(&tree);
        tree.borrow_mut().width = 50;
        tree.borrow_mut().height = 6;
        let weak = tree.downgrade();
        tree.borrow_mut().drawcb = Some(std::rc::Rc::new(move |_, writer, width, height| {
            assert_eq!((width, height), (46, 16));
            let tree = weak.upgrade().unwrap();
            assert_eq!(
                tree.screen_handle().borrow().cursor(),
                writer.cursor_position()
            );
            tree.borrow_mut().children.clear();
            tree.borrow_mut().line_list.clear();
            tree.borrow_mut().filter = Some(c"changed during preview".to_owned());
            tree.close();
            writer.puts(&grid_default_cell, c"retained preview", fmt_args![]);
        }));
        tree.draw();
        assert!(item.get().is_none());
        assert!(tree.borrow().pane().is_none());
        assert_eq!(
            tree.borrow().filter.as_deref(),
            Some(c"changed during preview")
        );
        let screen = tree.screen_handle().borrow();
        let grid = RustScreen::grid(&screen);
        assert_eq!(
            (grid).string_cells(2, grid.history_size() + 7, 16, None, 0, None).as_c_str(),
            c"retained preview"
        );
    }
}

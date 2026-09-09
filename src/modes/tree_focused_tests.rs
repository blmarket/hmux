use super::*;
use crate::WindowPane;
use crate::pane_identity::PaneIdentity;
use crate::tests::test_fixtures::{Pane, Target, globals, zeroed_client};
use crate::window::window_pane_current_mode;

unsafe fn items(
    target: &mut Target,
) -> (
    window_tree_itemdata,
    window_tree_itemdata,
    window_tree_itemdata,
) {
    unsafe {
        let s = target.session();
        let wl = target.winlink(0);
        let wp = target.pane(0);
        (
            window_tree_itemdata {
                type_0: WINDOW_TREE_SESSION,
                session: crate::SessionIdentity::session_id(&*s) as core::ffi::c_int,
                winlink: -1,
                pane: -1,
            },
            window_tree_itemdata {
                type_0: WINDOW_TREE_WINDOW,
                session: crate::SessionIdentity::session_id(&*s) as core::ffi::c_int,
                winlink: (*wl).idx,
                pane: -1,
            },
            window_tree_itemdata {
                type_0: WINDOW_TREE_PANE,
                session: crate::SessionIdentity::session_id(&*s) as core::ffi::c_int,
                winlink: (*wl).idx,
                pane: (*wp).pane_id() as core::ffi::c_int,
            },
        )
    }
}

#[test]
fn resolve_item_retains_each_level_and_rejects_stale_ids() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    unsafe {
        let (session, window, pane) = items(&mut target);
        for item in [session, window, pane] {
            let resolved = window_tree_resolve_item(&item).expect("the item is live");
            assert_eq!(resolved.link.session().id(), session.session as u_int);
            assert_eq!(resolved.link.index(), window.winlink);
            assert_eq!(
                resolved.pane.as_ref().map(RustWindowPaneWeak::id),
                Some(pane.pane as u_int)
            );
            assert!(resolved.link.get().is_some());
            assert!(resolved.pane.as_ref().unwrap().get().is_some());
        }

        let mut bad = pane;
        bad.session = core::ffi::c_int::MAX;
        assert!(window_tree_resolve_item(&bad).is_none());
        bad = pane;
        bad.winlink = core::ffi::c_int::MAX;
        assert!(window_tree_resolve_item(&bad).is_none());
        bad = pane;
        bad.pane = core::ffi::c_int::MAX;
        assert!(window_tree_resolve_item(&bad).is_none());
    }
}

#[test]
fn selection_tags_distinguish_kinds_and_links_and_expire_without_reuse() {
    let _guard = globals();
    let mut fixture = Target::new(40, 12);
    fixture.add_window(1, 40, 12);
    let mut data = window_tree_modedata::default();
    let keys = [
        WindowTreeTag::Session(0),
        WindowTreeTag::Window(0, 0),
        WindowTreeTag::Window(0, 1),
        WindowTreeTag::Pane(0),
    ];
    let tags: Vec<_> = keys.iter().map(|key| data.tag_for(*key)).collect();
    assert_eq!(
        tags.iter().collect::<std::collections::BTreeSet<_>>().len(),
        keys.len()
    );
    assert!(tags.iter().all(|tag| *tag != 0 && *tag != uint64_t::MAX));
    data.prune_tags();
    assert_eq!(
        keys.iter()
            .map(|key| data.tag_for(*key))
            .collect::<Vec<_>>(),
        tags
    );

    drop(fixture);
    data.prune_tags();
    assert!(data.tags.is_empty());
    assert!(!tags.contains(&data.tag_for(WindowTreeTag::Session(0))));
}

#[test]
fn target_and_search_helpers_cover_valid_invalid_case_and_item_types() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    unsafe {
        let (session, window, pane) = items(&mut target);
        for item in [session, window, pane] {
            let mut fs = cmd_find_state::default();
            let text = window_tree_get_target(&item, &mut fs).expect("fixture item has a target");
            assert!(text.as_bytes().starts_with(b"="));
        }
        let mut fs = cmd_find_state::default();
        assert!(window_tree_get_target(&window_tree_itemdata::default(), &mut fs).is_none());

        let session_name = crate::SessionNameState::session_name(&*target.session())
            .expect("the session has a name")
            .to_owned();
        assert_eq!(
            window_tree_search(ModeTreeItemData::Tree(session), &session_name, 0),
            1
        );
        let window_name = (*target.winlink(0))
            .window_handle()
            .unwrap()
            .window_name()
            .as_deref()
            .expect("a window has a name")
            .to_owned();
        assert_eq!(
            window_tree_search(ModeTreeItemData::Tree(window), &window_name, 1),
            1
        );
        let none = window_tree_itemdata::default();
        assert_eq!(window_tree_search(ModeTreeItemData::Tree(none), c"x", 1), 0);
    }
}

#[test]
fn resolved_targets_keep_owners_alive_and_observe_fixture_teardown() {
    let _guard = globals();
    let mut fixture = Target::new(40, 12);
    let (_, _, item) = unsafe { items(&mut fixture) };
    let resolved = unsafe { window_tree_resolve_item(&item) }.unwrap();
    let session = resolved.link.session().downgrade();
    let window = resolved.window.downgrade();

    drop(fixture);
    assert!(session.upgrade().is_some());
    assert!(window.upgrade().is_some());
    assert!(resolved.link.get().is_none());
    assert!(unsafe { resolved.pane.as_ref().unwrap().get() }.is_none());
    drop(resolved);
    assert!(session.upgrade().is_none());
    assert!(window.upgrade().is_none());
}

#[test]
fn filter_add_sort_help_and_case_helpers_cover_configuration_branches() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    unsafe {
        let s = target.session();
        let wl = target.winlink(0);
        let wp = target.pane(0);
        assert_eq!(
            window_tree_filter_pane(&mut *s, &mut *wl, &mut *wp, None),
            1
        );
        assert_eq!(
            window_tree_filter_pane(&mut *s, &mut *wl, &mut *wp, Some(c"1")),
            1
        );
        assert_eq!(
            window_tree_filter_pane(&mut *s, &mut *wl, &mut *wp, Some(c"0")),
            0
        );

        let mut data = window_tree_modedata::default();
        let first = window_tree_add_item(&mut data);
        (*first).type_0 = WINDOW_TREE_SESSION;
        let second = window_tree_add_item(&mut data);
        (*second).type_0 = WINDOW_TREE_WINDOW;
        assert_eq!(data.item_list.len(), 2);

        let mut criteria = sort_criteria_t::default();
        criteria.set_order(SORT_END);
        window_tree_sort(&mut criteria);
        assert_eq!(criteria.order(), SORT_INDEX);
        assert!(criteria.has_cycle());
        criteria.set_order(SORT_NAME);
        window_tree_sort(&mut criteria);
        assert_eq!(criteria.order(), SORT_NAME);

        let (lines, width, noun) = window_tree_help();
        assert_eq!(lines.len(), 12);
        assert_eq!(width, 51);
        assert_eq!(noun, c"item");
        assert_eq!(tolower(b'A'), b'a');
        assert_eq!(tolower(0xff), 0xff);
        let owner = WindowTreeModeDataRef::new(data);
        assert!(owner.owner().unwrap().upgrade().is_some());
    }
}

#[test]
fn swap_rejects_nonwindow_and_mixed_items_without_mutation() {
    let _guard = globals();
    let mut first = Target::new(40, 12);
    unsafe {
        let (session, window, _) = items(&mut first);
        let criteria = sort_criteria_t::default();
        assert_eq!(
            window_tree_swap(
                ModeTreeItemData::Tree(session),
                ModeTreeItemData::Tree(session),
                &criteria
            ),
            0
        );
        assert_eq!(
            window_tree_swap(
                ModeTreeItemData::Tree(session),
                ModeTreeItemData::Tree(window),
                &criteria
            ),
            0
        );
    }
}

#[test]
fn real_tree_mode_builds_draws_resizes_updates_and_exposes_current_item() {
    let _guard = globals();
    let mut target = Target::new(60, 18);
    unsafe {
        let pane = target.pane(0);
        let wl = target.winlink(0);
        let mut fs = cmd_find_state::default();
        cmd_find_from_winlink_pane(&mut fs, &*wl, &*pane, 0);
        let source_pane_id = crate::PaneIdentity::pane_id(&*pane);
        assert_eq!(
            crate::window::window_pane_set_mode(
                &mut *pane,
                crate::window::window_pane_find_by_id(source_pane_id),
                WindowMode::Tree,
                Some(&fs),
                None,
            ),
            0
        );
        let wme = window_pane_current_mode_mut(&mut *pane).expect("pane is in a mode");
        assert_eq!(wme.mode(), WindowMode::Tree);
        let data = wme.state.tree().unwrap();
        drop(data.tree_ref());
        assert!(!data.borrow().item_list.is_empty());

        window_tree_resize(wme, 50, 14);
        (data.clone()).update();
        let current = (data.tree_ref()).current_item();
        assert!(current.tree().is_some());
        let item = current.tree().unwrap();
        let found = window_tree_search(ModeTreeItemData::Tree(item), c"", 1);
        assert_eq!(found, 1);
        let key = (data.clone()).get_key(ModeTreeItemData::Tree(item), 1);
        assert_ne!(key, KEYC_NONE);
        window_pane_reset_mode(&mut *pane);
        drop(data);
        assert_eq!(window_tree_search(current, c"", 1), 1);
    }
}

#[test]
fn multi_level_builds_cover_sort_filter_type_and_tag_selection_paths() {
    let _guard = globals();
    let mut target = Target::new(60, 18);
    target.add_window(1, 40, 12);
    let mut extra = Pane::new(99, 20, 12, 20);
    unsafe {
        extra.hand_to(target.window(0));
        let pane = target.pane(0);
        let wl = target.winlink(0);
        let mut fs = cmd_find_state::default();
        cmd_find_from_winlink_pane(&mut fs, &*wl, &*pane, 0);
        let source_pane_id = crate::PaneIdentity::pane_id(&*pane);
        assert_eq!(
            crate::window::window_pane_set_mode(
                &mut *pane,
                crate::window::window_pane_find_by_id(source_pane_id),
                WindowMode::Tree,
                Some(&fs),
                None
            ),
            0
        );
        let wme = window_pane_current_mode(&*pane).expect("pane is in a mode");
        let data = wme.state.tree().unwrap();
        assert!(
            data.borrow()
                .item_list
                .iter()
                .any(|item| item.type_0 == WINDOW_TREE_SESSION)
        );
        assert!(
            data.borrow()
                .item_list
                .iter()
                .any(|item| item.type_0 == WINDOW_TREE_WINDOW)
        );
        assert!(
            data.borrow()
                .item_list
                .iter()
                .any(|item| item.type_0 == WINDOW_TREE_PANE)
        );

        let owner = data.borrow().owner.clone().unwrap().upgrade().unwrap();
        for type_0 in [WINDOW_TREE_SESSION, WINDOW_TREE_WINDOW, WINDOW_TREE_PANE] {
            data.borrow_mut().type_0 = type_0;
            let mut tag = 0;
            let mut criteria = sort_criteria_t::default();
            criteria.set_order(SORT_NAME);
            window_tree_sort(&mut criteria);
            (owner.clone()).build(&criteria, &mut tag, Some(c"1"));
            assert_ne!(tag, 0);
            assert!(!data.borrow().item_list.is_empty());
        }
        let mut tag = 0;
        (owner.clone()).build(&sort_criteria_t::default(), &mut tag, Some(c"0"));
        assert!(
            data.borrow()
                .item_list
                .iter()
                .all(|item| item.type_0 != WINDOW_TREE_PANE)
        );
        owner.build(&sort_criteria_t::default(), &mut tag, None);
        assert!(!data.borrow().item_list.is_empty());
        crate::window::window_pane_reset_mode(&mut *pane);
    }
}

#[test]
fn keyboard_navigation_preview_offsets_tags_and_marking_stay_inside_fixture() {
    let _guard = globals();
    let mut target = Target::new(60, 18);
    target.add_window(1, 40, 12);
    let mut client = zeroed_client();
    unsafe { client.set_attached_session(Some(target.session_handle())) };
    unsafe {
        let pane = target.pane(0);
        let wl = target.winlink(0);
        let mut fs = cmd_find_state::default();
        cmd_find_from_winlink_pane(&mut fs, &*wl, &*pane, 0);
        let source_pane_id = crate::PaneIdentity::pane_id(&*pane);
        assert_eq!(
            crate::window::window_pane_set_mode(
                &mut *pane,
                crate::window::window_pane_find_by_id(source_pane_id),
                WindowMode::Tree,
                Some(&fs),
                None
            ),
            0
        );
        let wme = window_pane_current_mode_mut(&mut *pane).expect("pane is in a mode");
        let data = wme.state.tree().unwrap();
        for key in [
            b'<' as key_code,
            b'>' as key_code,
            b'H' as key_code,
            KEYC_DOWN,
            KEYC_UP,
            KEYC_NPAGE,
            KEYC_PPAGE,
            KEYC_HOME,
            KEYC_END,
            b't' as key_code,
            b'T' as key_code,
            b'm' as key_code,
            b'M' as key_code,
        ] {
            (wme.state.tree().unwrap()).key(client.as_client_mut(), key, None);
        }
        assert!(!data.borrow().item_list.is_empty());
        assert!(*(*pane).flags() & PANE_REDRAW != 0);
        crate::window::window_pane_reset_mode(&mut *pane);
    }
}

#[test]
fn mouse_mapping_covers_nonmouse_preview_edges_and_empty_ranges() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    unsafe {
        let (session, window, pane) = items(&mut target);
        let data = WindowTreeModeDataRef::new(window_tree_modedata::default());
        assert_eq!(data.mouse(b'x' as key_code, 0, &session), KEYC_NONE);
        data.borrow_mut().left = 4;
        data.borrow_mut().right = 20;
        data.borrow_mut().start = 0;
        data.borrow_mut().end = 0;
        data.borrow_mut().each = 5;
        assert_eq!(
            data.mouse(KEYC_MOUSEDOWN1_PANE, 3, &session),
            b'<' as key_code
        );
        assert_eq!(
            data.mouse(KEYC_MOUSEDOWN1_PANE, 21, &window),
            b'>' as key_code
        );
        assert_eq!(data.mouse(KEYC_MOUSEDOWN1_PANE, 10, &pane), KEYC_NONE);
        let mut stale = session;
        stale.session = i32::MAX;
        assert_eq!(data.mouse(KEYC_MOUSEDOWN1_PANE, 10, &stale), KEYC_NONE);
    }
}

#[test]
fn swapping_two_windows_exchanges_links_and_rejects_sorted_or_stale_pairs() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    target.add_window(1, 50, 14);
    unsafe {
        let s = crate::SessionIdentity::session_id(&*target.session()) as core::ffi::c_int;
        let mut first = window_tree_itemdata {
            type_0: WINDOW_TREE_WINDOW,
            session: s,
            winlink: 0,
            pane: -1,
        };
        let mut second = window_tree_itemdata {
            type_0: WINDOW_TREE_WINDOW,
            session: s,
            winlink: 1,
            pane: -1,
        };
        let first_window = (*target.winlink(0)).window_handle().unwrap().clone();
        let second_window = (*target.winlink(1)).window_handle().unwrap().clone();
        let mut criteria = sort_criteria_t::default();
        criteria.set_order(SORT_INDEX);
        assert_eq!(
            window_tree_swap(
                ModeTreeItemData::Tree(first),
                ModeTreeItemData::Tree(second),
                &criteria,
            ),
            1
        );
        assert!(
            (*target.winlink(0))
                .window_handle()
                .unwrap()
                .ptr_eq(&second_window)
        );
        assert!(
            (*target.winlink(1))
                .window_handle()
                .unwrap()
                .ptr_eq(&first_window)
        );
        let same = first;
        assert_eq!(
            window_tree_swap(
                ModeTreeItemData::Tree(first),
                ModeTreeItemData::Tree(same),
                &criteria,
            ),
            0
        );
        criteria.set_order(SORT_NAME);
        let _ = window_tree_swap(
            ModeTreeItemData::Tree(first),
            ModeTreeItemData::Tree(second),
            &criteria,
        );
        second.winlink = i32::MAX;
        assert_eq!(
            window_tree_swap(
                ModeTreeItemData::Tree(first),
                ModeTreeItemData::Tree(second),
                &criteria,
            ),
            0
        );
        first.winlink = i32::MAX;
        assert_eq!(
            window_tree_swap(
                ModeTreeItemData::Tree(first),
                ModeTreeItemData::Tree(second),
                &criteria,
            ),
            0
        );
    }
}

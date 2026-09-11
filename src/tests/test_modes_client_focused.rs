use crate::screen::Screen as _;
use crate::grid::Grid as _;
use super::*;
use crate::WindowPane;
use crate::server::CLIENT_ATTACHED;
use crate::tests::test_fixtures::{
    Clients, Pane, Screen, Session, Target, Window, globals, zeroed_client,
};
use crate::window::window_pane_find_by_id;

fn mode_data(key_format: &'static CStr) -> WindowClientModeDataRef {
    WindowClientModeDataRef::new(window_client_modedata {
        wp: None,
        data: None,
        format: Some(c"#{client_name}".to_owned()),
        key_format: Some(key_format.to_owned()),
        command: Some(c"detach-client -t '%%'".to_owned()),
        item_list: Vec::new(),
        owner: None,
    })
}

#[test]
fn item_sort_help_and_key_helpers_cover_client_mode_boundaries() {
    unsafe {
        let _guard = globals();
        let mut client = zeroed_client();
        client.as_client_mut().name = Some(c"alpha".to_owned());
        let data = mode_data(c"#{line}");
        let item = window_client_add_item(&mut data.borrow_mut(), client.clone());
        assert_eq!(data.borrow().item_list.len(), 1);
        assert!(item.client().ptr_eq(&client));

        let key = (data.clone()).get_key(ModeTreeItemData::Client(item), 7);
        assert_eq!(key, b'7' as key_code);

        let mut sort = sort_criteria_t::default();
        sort.set_order(SORT_END);
        window_client_sort(&mut sort);
        assert_eq!(sort.order(), SORT_NAME);
        assert!(sort.has_cycle());
        sort.set_order(SORT_ACTIVITY);
        window_client_sort(&mut sort);
        assert_eq!(sort.order(), SORT_ACTIVITY);

        let (lines, width, noun) = window_client_help();
        assert_eq!(lines.len(), 8);
        assert_eq!(width, 0);
        assert_eq!(noun, c"client");
    }
}

#[test]
fn build_selects_attached_clients_and_applies_filters() {
    unsafe {
        let _guard = globals();
        let mut window = Window::new(901, "clients", 80, 24);
        let mut pane = Pane::new(901, 80, 24, 100);
        window.add_pane(&mut pane);
        let data = mode_data(c"#{line}");
        let build_data = data.downgrade();
        let menu_data = data.downgrade();
        let key_data = data.downgrade();
        let tree = ModeTreeDataRef::start(
            &mut *pane.ptr(),
            None,
            Some(std::rc::Rc::new(move |sort, tag, filter| {
                if let Some(data) = build_data.upgrade() {
                    data.build(sort, tag, filter);
                }
            })),
            Some(std::rc::Rc::new(|itemdata, writer, sx, sy| {
                window_client_draw(itemdata, writer, sx, sy)
            })),
            None,
            Some(std::rc::Rc::new(move |c, key| {
                if let Some(data) = menu_data.upgrade() {
                    data.menu(c, key);
                }
            })),
            None,
            Some(std::rc::Rc::new(move |itemdata, line| {
                key_data
                    .upgrade()
                    .map_or(KEYC_NONE, |data| data.get_key(itemdata, line))
            })),
            None,
            Some(std::rc::Rc::new(window_client_sort)),
            Some(window_client_help()),
            WindowModeData::Client(data.downgrade()),
            &window_client_menu_items,
        );
        data.borrow_mut().data = Some(tree.clone());

        let session = Session::new(902, "client-mode");
        let mut clients = Clients::new();
        let first = clients.add("alpha", 80, 24);
        (*first).set_attached_session(Some(session.handle()));
        (*first).flags |= CLIENT_ATTACHED as uint64_t;
        let second = clients.add("beta", 100, 30);
        (*second).set_attached_session(Some(session.handle()));
        (*second).flags |= CLIENT_ATTACHED as uint64_t;
        let detached = clients.add("detached", 40, 10);
        (*detached).set_attached_session(Some(session.handle()));
        (*detached).flags |= (CLIENT_ATTACHED | CLIENT_UNATTACHEDFLAGS) as uint64_t;
        clients.add("no-session", 20, 5);

        let named_filter = c"#{m/r:^(alpha|beta)$,#{client_name}}";
        tree.borrow_mut().filter = Some(named_filter.to_owned());
        tree.build();
        assert_eq!(data.borrow().item_list.len(), 2);
        assert_eq!(tree.borrow().children.len(), 2);
        let original = tree.current_item().client().unwrap();
        let original_weak = std::rc::Rc::downgrade(&original);
        let original_client = original.client();

        tree.borrow_mut().children.clear();
        let mut tag = 0;
        let sort = tree.borrow().sort_crit.clone();
        (data.clone()).build(&sort, &mut tag, Some(c"0"));
        assert!(tree.borrow().children.is_empty());
        tree.borrow_mut().line_list.clear();

        tree.borrow_mut().filter = Some(named_filter.to_owned());
        tree.build();
        assert_eq!(tree.borrow().no_matches, 0);
        assert_eq!(tree.borrow().children.len(), 2);
        let current = tree.current_item().client().unwrap();
        assert!(!std::rc::Rc::ptr_eq(&original, &current));
        assert!(original.client().ptr_eq(&original_client));
        assert!(current.client().ptr_eq(&original_client));
        window_client_do_detach(
            WindowModeData::Client(data.downgrade()),
            ModeTreeItemData::Client(original.clone()),
            KEYC_NONE,
        );
        assert_eq!(tree.borrow().current, 0);
        window_client_do_detach(
            WindowModeData::Client(data.downgrade()),
            ModeTreeItemData::Client(current),
            KEYC_NONE,
        );
        assert_eq!(tree.borrow().current, 1);
        drop(original);
        assert!(original_weak.upgrade().is_none());
    }
}

#[test]
fn draw_returns_for_missing_or_unattached_session() {
    unsafe {
        let _guard = globals();
        let data = mode_data(c"#{line}");
        let mut client = zeroed_client();
        let item = window_client_add_item(&mut data.borrow_mut(), client.clone());
        let mut output = Screen::new(20, 5, 0);
        let mut writer = crate::screen::screen_write_ctx_on_screen(&mut output);
        window_client_draw(ModeTreeItemData::Client(item.clone()), &mut writer, 20, 5);
        *client.flags_mut() |= CLIENT_UNATTACHEDFLAGS as uint64_t;
        window_client_draw(ModeTreeItemData::Client(item.clone()), &mut writer, 20, 5);
    }
}

#[test]
fn lifecycle_initializes_resizes_updates_and_frees_an_empty_mode() {
    unsafe {
        let _guard = globals();
        let mut target = Target::new(40, 12);
        let pane = target.pane(0);
        let mut entry = window_mode_entry {
            wp: Some(
                crate::window::window_pane_find_by_id((*pane).pane_id())
                    .unwrap()
                    .clone(),
            ),
            swp: None,
            state: WindowModeState::None,
            screen: None,
            prefix: 0,
            };
        WindowMode::Client.init(
            &mut entry,
            crate::window::window_pane_find_by_id((*pane).pane_id()).unwrap(),
            None,
            None,
        );
        assert!(entry.screen.is_some());
        assert!(matches!(entry.state, WindowModeState::Client(_)));
        window_client_resize(&mut entry, 32, 8);
        {
            let screen = entry
                .screen.as_ref().unwrap().shared().unwrap()
                .borrow();
            assert_eq!(RustScreen::grid(&screen).width(), 32);
            assert_eq!(RustScreen::grid(&screen).height(), 8);
        }
        (entry.state.client().unwrap()).update();
        assert_ne!(*(*pane).flags() & PANE_REDRAW, 0);
        window_client_free(&mut entry);
    }
}

#[test]
fn menu_callbacks_stop_observing_a_removed_pane() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    let data = mode_data(c"#{line}");
    data.borrow_mut().wp = target.state().pane_ref();
    let observed = data.borrow().pane().unwrap();
    drop(target);
    unsafe {
        assert!(observed.get().is_none());
        assert!(data.borrow().pane().is_none());
        let mut client = zeroed_client();
        data.menu(client.as_client_mut(), b'd' as key_code);
    }
}

#[test]
fn preview_uses_the_clients_current_window_and_skips_missing_active_panes() {
    let _guard = globals();
    let mut target = Target::new(40, 12);
    target.add_window(7, 40, 12);
    unsafe {
        for (id, text) in [(0, c"A"), (1, c"B")] {
            let mut pane = window_pane_find_by_id(id).unwrap();
            let mut screen = pane.get_mut().unwrap().base_mut();
            let mut writer = crate::screen::RustScreenWriteCtx::on_borrowed_screen(&mut screen);
            writer.nputs(1, &crate::grid::grid_default_cell, c"%s", fmt_args![text]);
        }
        let mut session = target.state().session().unwrap();
        session.as_session_mut().statuslines = 0;
        let mut client = zeroed_client();
        client.set_attached_session(Some(&session));
        client.as_client_mut().status.screen = RustScreen::new_with_server_options(40, 1, 0);
        let metadata = ModeTreeItemData::Client(std::rc::Rc::new(window_client_itemdata {
            c: client.clone(),
        }));
        for (index, expected) in [(0, b'A'), (7, b'B')] {
            session.as_session_mut().curw = Some(index);
            let mut output = Screen::new(40, 12, 0);
            {
                let mut writer = crate::screen::screen_write_ctx_on_screen(&mut output);
                window_client_draw(metadata.clone(), &mut writer, 40, 12);
            }
            assert_eq!(
                (RustScreen::grid(&output)).cell(0, 0)
                    .data
                    .data[0],
                expected
            );
        }
        let window = session.curw().unwrap().window().unwrap().clone();
        window.as_window_mut().active = None;
        for current in [Some(7), None] {
            session.as_session_mut().curw = current;
            let mut output = Screen::new(40, 12, 0);
            {
                let mut writer = crate::screen::screen_write_ctx_on_screen(&mut output);
                window_client_draw(metadata.clone(), &mut writer, 40, 12);
            }
            let grid = RustScreen::grid(&output);
            assert!(
                (grid).string_cells(0, 0, grid.width(), None, 0, None)
                    .to_string_lossy()
                    .trim()
                    .is_empty()
            );
        }
        session.as_session_mut().curw = Some(0);
        client.set_attached_session(None);
    }
}

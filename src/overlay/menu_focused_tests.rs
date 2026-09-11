use crate::grid::Grid as _;
use super::*;
use crate::reactor::ByteBuffer;
use crate::tests::test_fixtures::{Target, globals, zeroed_client, zeroed_term};
use core::ffi::c_int;

const CALLBACK_CHOICE: crate::server_state::LocalField<std::cell::Cell<usize>> =
    crate::server_state::LocalField::new(|state| &state.menu_test_choice);

fn record_choice(choice: u_int, _key: key_code) {
    CALLBACK_CHOICE.set(choice as usize);
}

struct Fixture {
    _target: Target,
    client: ClientRef,
}

impl Fixture {
    fn new(sx: u_int, sy: u_int) -> Self {
        let target = Target::new(sx, sy);
        let mut client = zeroed_client();
        unsafe { client.as_tty_mut() }.sx = sx;
        unsafe { client.as_tty_mut() }.sy = sy;
        unsafe { client.set_attached_session(Some(target.session_handle())) };
        unsafe { client.as_tty_mut() }.term = Some(zeroed_term());
        unsafe { client.as_tty_mut() }.out = Some(Box::new(ByteBuffer::new()));
        unsafe { client.as_tty_mut() }.client = Some(client.downgrade());
        Self {
            _target: target,
            client,
        }
    }

    fn client(&mut self) -> &mut client {
        unsafe { self.client.as_client_mut() }
    }

    fn menu(&mut self, flags: c_int, start: c_int) -> MenuDataRef {
        unsafe {
            menu_prepare(
                populated_menu(),
                flags,
                start,
                None,
                70,
                20,
                self.client.as_client_mut(),
                BOX_LINES_SIMPLE,
                None,
                None,
                None,
                None,
                None,
            )
            .expect("menu fits")
        }
    }
}

fn entry(name: Option<&CStr>, key: key_code) -> menu_entry {
    menu_entry {
        name: name.map(CStr::to_owned),
        key,
        command: Some(c"display-message chosen".to_owned()),
    }
}

fn populated_menu() -> Box<menu> {
    let mut menu = menu_create(c"navigation");
    menu.width = 18;
    menu.items = vec![
        entry(Some(c"zero"), b'0' as key_code),
        entry(None, KEYC_NONE),
        entry(Some(c"-disabled"), KEYC_NONE),
        entry(Some(c"three"), b'3' as key_code),
        entry(Some(c"four"), b'4' as key_code),
        entry(Some(c"five"), b'5' as key_code),
        entry(Some(c"six"), b'6' as key_code),
        entry(Some(c"seven"), b'7' as key_code),
        entry(Some(c"eight"), b'8' as key_code),
        entry(Some(c"nine"), b'9' as key_code),
    ];
    menu
}

fn key(code: key_code) -> key_event {
    key_event {
        key: code,
        ..Default::default()
    }
}

#[test]
fn prepare_clamps_geometry_and_selects_real_entries() {
    let _guard = globals();
    let mut fixture = Fixture::new(40, 16);
    unsafe {
        let md = fixture.menu(MENU_NOMOUSE, 1);
        assert_eq!((md.borrow().px, md.borrow().py), (18, 4));
        assert_eq!(md.borrow().choice, 3);
        assert_eq!(
            md.borrow().s.borrow().mode() & (MODE_MOUSE_ALL | MODE_MOUSE_BUTTON | MODE_CURSOR),
            0
        );
        let (s, x, y) = menu_mode_cb(&md.borrow());
        assert_eq!(s, md.borrow().s.borrow().mode_state());
        assert_eq!((x, y), (20, 8));

        let wrapped = fixture.menu(MENU_NOMOUSE, 99);
        assert_eq!(wrapped.borrow().choice, 9);
        menu_resize_cb(fixture.client(), &mut wrapped.borrow_mut());
        assert_eq!((wrapped.borrow().px, wrapped.borrow().py), (18, 4));

        fixture.client.as_tty_mut().sx = 15;
        fixture.client.as_tty_mut().sy = 8;
        menu_resize_cb(fixture.client(), &mut wrapped.borrow_mut());
        assert_eq!((wrapped.borrow().px, wrapped.borrow().py), (0, 0));
    }
}

#[test]
fn prepare_rejects_oversize_and_enables_mouse_mode() {
    let _guard = globals();
    let mut fixture = Fixture::new(21, 11);
    unsafe {
        assert!(
            menu_prepare(
                populated_menu(),
                0,
                0,
                None,
                0,
                0,
                fixture.client(),
                BOX_LINES_DEFAULT,
                Some(c"fg=red"),
                Some(c"bg=blue"),
                Some(c"fg=green"),
                None,
                None,
            )
            .is_none()
        );

        fixture.client.as_tty_mut().sx = 30;
        fixture.client.as_tty_mut().sy = 20;
        let md = fixture.menu(0, 4);
        assert_eq!(md.borrow().choice, -1);
        assert_ne!(md.borrow().s.borrow().mode() & MODE_MOUSE_ALL, 0);
        assert_ne!(md.borrow().s.borrow().mode() & MODE_MOUSE_BUTTON, 0);
        assert_eq!(md.borrow().s.borrow().mode() & MODE_CURSOR, 0);
    }
}

#[test]
fn keyboard_navigation_exercises_every_dispatch_family() {
    let _guard = globals();
    let mut fixture = Fixture::new(100, 40);
    let md = fixture.menu(MENU_NOMOUSE, 0);
    let cases = [
        (KEYC_UP, 9),
        (KEYC_BTAB, 8),
        (9, 8),
        (KEYC_DOWN, 9),
        (b'j' as key_code, 0),
        (KEYC_PPAGE, 0),
        (KEYC_NPAGE, 7),
        (KEYC_HOME, 0),
        (b'g' as key_code, 0),
        (KEYC_END, 9),
        (b'G' as key_code, 9),
    ];
    unsafe {
        for (code, expected) in cases {
            let mut event = key(code);
            assert_eq!(md.key(fixture.client(), &mut event), 0);
            assert_eq!(md.borrow().choice, expected, "key {code}");
        }
        assert_ne!(fixture.client.flags() & CLIENT_REDRAWOVERLAY as u64, 0);
    }
}

#[test]
fn tab_and_escape_obey_menu_flags() {
    let _guard = globals();
    let mut fixture = Fixture::new(100, 40);
    unsafe {
        let md = fixture.menu(MENU_NOMOUSE | MENU_TAB, 9);
        let mut tab = key(9);
        assert_eq!(md.key(fixture.client(), &mut tab), 1);
        md.borrow_mut().choice = 0;
        let mut backtab = key(KEYC_BTAB);
        assert_eq!(md.key(fixture.client(), &mut backtab), 0);
        assert_eq!(md.borrow().choice, 9);
        let mut escape = key(27);
        assert_eq!(md.key(fixture.client(), &mut escape), 1);
        let mut q = key(b'q' as key_code);
        assert_eq!(md.key(fixture.client(), &mut q), 1);
    }
}

#[test]
fn accelerator_and_enter_deliver_exactly_one_choice() {
    let _guard = globals();
    let mut fixture = Fixture::new(100, 40);
    unsafe {
        CALLBACK_CHOICE.set(usize::MAX);
        let md = fixture.menu(MENU_NOMOUSE, -1);
        md.borrow_mut().cb = Some(Box::new(record_choice));
        let mut accelerator = key(b'7' as key_code);
        assert_eq!(md.key(fixture.client(), &mut accelerator), 1);
        assert_eq!(CALLBACK_CHOICE.get(), 7);
        assert!(md.borrow().cb.is_none());

        let md = fixture.menu(MENU_NOMOUSE | MENU_STAYOPEN, 1);
        md.borrow_mut().choice = 1;
        let mut enter = key(13);
        assert_eq!(md.key(fixture.client(), &mut enter), 0);
        md.borrow_mut().flags &= !MENU_STAYOPEN;
        assert_eq!(md.key(fixture.client(), &mut enter), 1);
    }
}

#[test]
fn mouse_motion_selection_and_outside_clicks_cover_lifecycle() {
    let _guard = globals();
    let mut fixture = Fixture::new(100, 40);
    let md = fixture.menu(0, -1);
    unsafe {
        let mut inside = key(KEYC_MOUSE);
        inside.m.x = md.borrow().px + 2;
        inside.m.y = md.borrow().py + 5;
        inside.m.b = MOUSE_WHEEL_DOWN as u_int;
        assert_eq!(md.key(fixture.client(), &mut inside), 0);
        assert_eq!(md.borrow().choice, 4);

        let mut outside_motion = key(KEYC_MOUSE);
        outside_motion.m.x = 0;
        outside_motion.m.y = 0;
        outside_motion.m.b = MOUSE_WHEEL_UP as u_int;
        assert_eq!(md.key(fixture.client(), &mut outside_motion), 0);
        assert_eq!(md.borrow().choice, -1);

        outside_motion.m.b = 3;
        assert_eq!(md.key(fixture.client(), &mut outside_motion), 1);

        let no_mouse = fixture.menu(MENU_NOMOUSE, 0);
        inside.m.b = MOUSE_BUTTON_1 as u_int;
        assert_eq!(no_mouse.key(fixture.client(), &mut inside), 0);
        inside.m.b = 3;
        assert_eq!(no_mouse.key(fixture.client(), &mut inside), 1);
    }
}

#[test]
fn overlay_range_and_free_callback_are_self_contained() {
    let _guard = globals();
    let mut fixture = Fixture::new(100, 40);
    {
        CALLBACK_CHOICE.set(0);
        let md = fixture.menu(0, -1);
        let py = md.borrow().py;
        let mut state = md.borrow_mut();
        let ranges = menu_check_cb(&mut state, 0, py, 100);
        assert!(ranges.used > 0);
        drop(ranges);
        drop(state);
        md.borrow_mut().cb = Some(Box::new(record_choice));
        md.close(fixture.client());
        assert_eq!(CALLBACK_CHOICE.get(), UINT_MAX as usize);
    }
}

#[test]
fn adding_items_handles_separators_keys_trimming_and_commands() {
    let _guard = globals();
    let mut fixture = Fixture::new(28, 20);
    unsafe {
        let mut menu = menu_create(c"title");
        menu_add_item(&mut menu, None, None, fixture.client(), None);
        assert!(menu.items.is_empty());

        menu_add_item(
            &mut menu,
            Some(&menu_item {
                name: Some(c"short"),
                key: b's' as key_code,
                command: Some(c"display-message short"),
            }),
            None,
            fixture.client(),
            None,
        );
        menu_add_item(&mut menu, None, None, fixture.client(), None);
        menu_add_item(&mut menu, None, None, fixture.client(), None);
        menu_add_items(
            &mut menu,
            &[
                menu_item {
                    name: Some(c"a very long menu item that must be clipped"),
                    key: KEYC_UNKNOWN,
                    command: None,
                },
                menu_item {
                    name: Some(c"-disabled"),
                    key: KEYC_NONE,
                    command: Some(c""),
                },
            ],
            None,
            fixture.client(),
            None,
        );
        assert_eq!(menu.items.len(), 4);
        assert!(
            menu.items[0]
                .name()
                .unwrap()
                .to_bytes()
                .windows(3)
                .any(|w| w == b"(s)")
        );
        assert!(menu.items[1].name().is_none());
        assert!(menu.items[2].name().unwrap().to_bytes().ends_with(b">"));
        assert_eq!(menu.items[3].name().unwrap().to_bytes(), b"-disabled");
        assert!(menu.width <= fixture.client.as_tty().sx - 4);
    }
}

#[test]
fn drawing_builds_the_menu_screen_for_each_border_mode() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    let mut fixture = Fixture::new(100, 40);
    unsafe {
        for lines in [BOX_LINES_NONE, BOX_LINES_SIMPLE] {
            let md = fixture.menu(MENU_NOMOUSE, 3);
            md.borrow_mut().border_lines = lines;
            md.borrow_mut().style = Some(c"fg=red,bg=blue".to_owned());
            md.borrow_mut().selected_style = Some(c"fg=green".to_owned());
            md.borrow_mut().border_style = Some(c"fg=yellow".to_owned());
            md.draw(fixture.client(), &mut screen_redraw_ctx::default());
            assert_eq!(
                RustScreen::grid(&md.borrow().s.borrow()).width(),
                md.borrow().menu.width + 4
            );
            assert!(
                fixture
                    .client
                    .as_tty()
                    .out
                    .as_ref()
                    .is_some_and(|out| !out.is_empty())
            );
        }
    }
}

#[test]
fn stayopen_mouse_release_drag_wheel_and_motion_take_distinct_paths() {
    let _guard = globals();
    let mut fixture = Fixture::new(100, 40);
    unsafe {
        let md = fixture.menu(MENU_STAYOPEN, 0);
        CALLBACK_CHOICE.set(usize::MAX);
        md.borrow_mut().cb = Some(Box::new(record_choice));
        let mut event = key(KEYC_MOUSE);
        event.m.x = md.borrow().px + 2;
        event.m.y = md.borrow().py + 4;
        event.m.b = MOUSE_WHEEL_DOWN as u_int;
        assert_eq!(md.key(fixture.client(), &mut event), 0);
        assert_eq!(md.borrow().choice, 3);

        event.m.b = MOUSE_MASK_DRAG as u_int | MOUSE_BUTTON_1 as u_int;
        event.m.y = md.borrow().py + 6;
        assert_eq!(md.key(fixture.client(), &mut event), 0);
        assert_eq!(md.borrow().choice, 5);

        event.m.b = MOUSE_BUTTON_1 as u_int;
        assert_eq!(md.key(fixture.client(), &mut event), 1);
        assert_eq!(CALLBACK_CHOICE.get(), 5);

        event.m.x = 0;
        event.m.y = 0;
        event.m.b = MOUSE_BUTTON_1 as u_int;
        assert_eq!(md.key(fixture.client(), &mut event), 1);

        let mut motion = key((KEYC_TYPE_MOUSEMOVE as key_code) << 32);
        motion.m.x = md.borrow().px + 2;
        motion.m.y = md.borrow().py + 8;
        motion.m.b = MOUSE_MASK_DRAG as u_int;
        assert_eq!(md.key(fixture.client(), &mut motion), 0);
    }
}

#[test]
fn disabled_entries_unknown_keys_and_flagged_accelerators_do_not_misfire() {
    let _guard = globals();
    let mut fixture = Fixture::new(100, 40);
    unsafe {
        CALLBACK_CHOICE.set(usize::MAX);
        let md = fixture.menu(MENU_NOMOUSE | MENU_STAYOPEN, 2);
        md.borrow_mut().cb = Some(Box::new(record_choice));
        md.borrow_mut().choice = 2;
        let mut enter = key(13);
        assert_eq!(md.key(fixture.client(), &mut enter), 0);
        assert_eq!(CALLBACK_CHOICE.get(), usize::MAX);

        let mut unknown = key(b'x' as key_code);
        assert_eq!(md.key(fixture.client(), &mut unknown), 0);
        assert_eq!(CALLBACK_CHOICE.get(), usize::MAX);

        let mut flagged = key(b'4' as key_code | KEYC_CTRL as key_code);
        assert_eq!(md.key(fixture.client(), &mut flagged), 0);
        assert_eq!(CALLBACK_CHOICE.get(), usize::MAX);
        assert!(md.borrow().cb.is_some());
    }
}

#[test]
fn selection_callback_can_reborrow_the_retained_menu() {
    let _guard = globals();
    let mut fixture = Fixture::new(100, 40);
    let owner = fixture.menu(MENU_NOMOUSE, 0);
    let weak = std::rc::Rc::downgrade(&owner.0);
    owner.borrow_mut().cb = Some(Box::new(move |choice, key| {
        assert_eq!((choice, key), (7, b'7' as key_code));
        let state = weak.upgrade().unwrap();
        assert!(state.borrow().cb.is_none());
        state.borrow_mut().choice = 3;
    }));
    unsafe {
        assert_eq!(owner.key(fixture.client(), &mut key(b'7' as key_code)), 1);
    }
    assert_eq!(owner.borrow().choice, 3);
    assert!(owner.borrow().cb.is_none());
}

#[test]
fn clearing_an_overlay_cancels_once_and_keeps_retained_menu_state_alive() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    let mut fixture = Fixture::new(100, 40);
    let owner = fixture.menu(MENU_NOMOUSE, 0);
    let weak = std::rc::Rc::downgrade(&owner.0);
    let callback_owner = weak.clone();
    let calls = std::rc::Rc::new(std::cell::Cell::new(0));
    let callback_calls = calls.clone();
    owner.borrow_mut().cb = Some(Box::new(move |choice, key| {
        assert_eq!((choice, key), (UINT_MAX, KEYC_NONE));
        let state = callback_owner.upgrade().unwrap();
        state.borrow_mut().choice = 9;
        callback_calls.set(callback_calls.get() + 1);
    }));
    unsafe {
        server_client_set_overlay(
            fixture.client(),
            0,
            Overlay::Menu,
            OverlayState::Menu(owner),
        );
        let retained = fixture.client().overlay_data().menu();
        retained.draw(fixture.client(), &mut screen_redraw_ctx::default());
        let screen = retained.borrow().s.clone();
        crate::server::server_client_clear_overlay(fixture.client());
        assert_eq!(calls.get(), 1);
        assert_eq!(retained.borrow().choice, 9);
        retained.close(fixture.client());
        assert_eq!(calls.get(), 1);
        assert!(weak.upgrade().is_none());
        assert_eq!(screen.borrow().size(), (22, 12));
    }
}

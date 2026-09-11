use crate::grid::Grid as _;
use super::*;
use crate::reactor::ByteBuffer;
use crate::server::server_client_clear_overlay;
use crate::tests::test_fixtures::{
    Clients, Session, Window, ensure_reactor, globals, link, unlink_all, zeroed_term,
};

struct Fixture {
    session: Session,
    window: Window,
    _clients: Clients,
    client: *mut client,
    _guard: crate::tests::test_fixtures::GlobalsGuard,
}

impl Fixture {
    fn new(name: &str, sx: u_int, sy: u_int, lines: box_lines) -> Self {
        let guard = globals();
        ensure_reactor();
        let mut session = Session::new(0, name);
        let mut window = Window::new(0, "popup", 80, 24);
        link(&mut session, &mut window, 0);
        let mut clients = Clients::new();
        let client = clients.add(name, 80, 24);
        unsafe {
            (*client).set_attached_session(Some(session.handle()));
            assert_eq!(
                popup_display(
                    POPUP_NOJOB,
                    lines,
                    None,
                    4,
                    3,
                    sx,
                    sy,
                    None,
                    None,
                    &[],
                    None,
                    Some(c"title"),
                    &mut *client,
                    Some(&*session.ptr()),
                    None,
                    None,
                    None,
                ),
                0
            );
        }
        Self {
            session,
            window,
            _clients: clients,
            client,
            _guard: guard,
        }
    }

    fn pd(&self) -> PopupDataRef {
        unsafe { (*self.client).overlay_data().popup() }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        unsafe {
            if !(*self.client).overlay_data().is_none() {
                server_client_clear_overlay(&mut *self.client);
            }
        }
        unlink_all(&mut self.session);
    }
}

#[test]
fn mode_and_context_callbacks_account_for_border_and_client_identity() {
    let f = Fixture::new("mode", 10, 6, BOX_LINES_SINGLE);
    unsafe {
        let pd = f.pd();
        pd.borrow().s.borrow_mut().set_cursor(2, 1);
        let (screen, x, y) = popup_mode_cb(&pd.borrow());
        assert_eq!(screen, pd.borrow().s.borrow().mode_state());
        assert_eq!((x, y), (7, 5));

        let mut ctx = tty_ctx::default();
        popup_init_ctx_cb(&pd.borrow(), &mut ctx);
        let palette = ctx.palette.as_ref().expect("a popup carries a palette");
        assert_eq!(
            (palette.fg, palette.bg),
            (pd.borrow().palette.fg, pd.borrow().palette.bg)
        );
        assert_eq!(ctx.flags & TTY_CTX_WINDOW_BIGGER, 0);
        (*f.client).flags &= !(CLIENT_REDRAWOVERLAY as u64);
        assert_eq!(popup_set_client_cb(&mut ctx, &mut *f.client), 1);
        assert_eq!((ctx.xoff, ctx.yoff), (5, 4));
        let mut other = client::default();
        assert_eq!(popup_set_client_cb(&mut ctx, &mut other), 0);
        (*f.client).flags |= CLIENT_REDRAWOVERLAY as u64;
        assert_eq!(popup_set_client_cb(&mut ctx, &mut *f.client), 0);
        (*f.client).flags &= !(CLIENT_REDRAWOVERLAY as u64);

        pd.borrow_mut().border_lines = BOX_LINES_NONE;
        let (_, x, y) = popup_mode_cb(&pd.borrow());
        assert_eq!((x, y), (6, 4));
        assert_eq!(popup_set_client_cb(&mut ctx, &mut *f.client), 1);
        assert_eq!((ctx.xoff, ctx.yoff), (4, 3));
    }
}

#[test]
fn range_callback_splits_lines_before_inside_and_after_popup() {
    let f = Fixture::new("ranges", 10, 6, BOX_LINES_SINGLE);
    {
        let pd = f.pd();
        let ranges = popup_ranges(&pd, 0, 0, 20);
        let r = ranges.borrow();
        assert_eq!(r.used, 1);
        assert_eq!((r.ranges[0].px, r.ranges[0].nx), (0, 20));
        drop(r);

        let py = pd.borrow().py;
        let ranges = popup_ranges(&pd, 0, py, 20);
        let r = ranges.borrow();
        assert_eq!(r.used, 2);
        assert_eq!((r.ranges[0].px, r.ranges[0].nx), (0, 4));
        assert_eq!((r.ranges[1].px, r.ranges[1].nx), (14, 6));
        drop(r);

        let ranges = popup_ranges(&pd, 6, py + 1, 3);
        let r = ranges.borrow();
        assert_eq!(
            r.ranges[..r.used as usize]
                .iter()
                .map(|one| one.nx)
                .sum::<u_int>(),
            0
        );
    }
}

#[test]
fn resize_clamps_geometry_and_resizes_bordered_and_borderless_screens() {
    let f = Fixture::new("resize", 20, 10, BOX_LINES_SINGLE);
    unsafe {
        let pd = f.pd();
        (*f.client).tty.sx = 12;
        (*f.client).tty.sy = 7;
        ((*f.client).overlay_data().popup()).resize(&mut *f.client);
        assert_eq!(
            (
                pd.borrow().px,
                pd.borrow().py,
                pd.borrow().sx,
                pd.borrow().sy
            ),
            (0, 0, 12, 7)
        );
        assert_eq!(
            (
                RustScreen::grid(&pd.borrow().s.borrow()).width(),
                RustScreen::grid(&pd.borrow().s.borrow()).height()
            ),
            (10, 5)
        );

        pd.borrow_mut().border_lines = BOX_LINES_NONE;
        pd.borrow_mut().psx = 8;
        pd.borrow_mut().psy = 4;
        pd.borrow_mut().ppx = 2;
        pd.borrow_mut().ppy = 1;
        ((*f.client).overlay_data().popup()).resize(&mut *f.client);
        assert_eq!(
            (
                pd.borrow().px,
                pd.borrow().py,
                pd.borrow().sx,
                pd.borrow().sy
            ),
            (2, 1, 8, 4)
        );
        assert_eq!(
            (
                RustScreen::grid(&pd.borrow().s.borrow()).width(),
                RustScreen::grid(&pd.borrow().s.borrow()).height()
            ),
            (8, 4)
        );
    }
}

#[test]
fn drag_handler_moves_clamps_resizes_and_stops_on_release() {
    let f = Fixture::new("drag", 10, 6, BOX_LINES_SINGLE);
    unsafe {
        let pd = f.pd();
        pd.borrow_mut().dragging = MOVE;
        pd.borrow_mut().dx = 2;
        pd.borrow_mut().dy = 1;
        let mut m = mouse_event {
            x: 30,
            y: 20,
            b: MOUSE_MASK_DRAG as u_int,
            ..Default::default()
        };
        popup_handle_drag(&mut *f.client, &mut pd.borrow_mut(), &m);
        assert_eq!((pd.borrow().px, pd.borrow().py), (28, 18));
        m.x = 1;
        m.y = 0;
        popup_handle_drag(&mut *f.client, &mut pd.borrow_mut(), &m);
        assert_eq!((pd.borrow().px, pd.borrow().py), (0, 0));
        m.b = 0;
        popup_handle_drag(&mut *f.client, &mut pd.borrow_mut(), &m);
        assert_eq!(pd.borrow().dragging, OFF);

        pd.borrow_mut().dragging = SIZE;
        pd.borrow_mut().border_lines = BOX_LINES_SINGLE;
        m.b = MOUSE_MASK_DRAG as u_int;
        m.x = pd.borrow().px + 2;
        m.y = pd.borrow().py + 2;
        popup_handle_drag(&mut *f.client, &mut pd.borrow_mut(), &m);
        assert_eq!((pd.borrow().sx, pd.borrow().sy), (10, 6));
        m.x = pd.borrow().px + 16;
        m.y = pd.borrow().py + 9;
        popup_handle_drag(&mut *f.client, &mut pd.borrow_mut(), &m);
        assert_eq!((pd.borrow().sx, pd.borrow().sy), (16, 9));
        assert_eq!(
            (
                RustScreen::grid(&pd.borrow().s.borrow()).width(),
                RustScreen::grid(&pd.borrow().s.borrow()).height()
            ),
            (14, 7)
        );
    }
}

fn popup_ranges(owner: &PopupDataRef, px: u_int, py: u_int, nx: u_int) -> VisibleRangesRef {
    let mut pd = owner.borrow_mut();
    popup_check_cb(&mut pd, px, py, nx);
    pd.r.clone()
}

fn event(key: key_code) -> key_event {
    key_event {
        key,
        ..Default::default()
    }
}

#[test]
fn key_callback_handles_escape_close_any_key_and_outside_mouse() {
    let f = Fixture::new("keys", 10, 6, BOX_LINES_SINGLE);
    unsafe {
        let pd = f.pd();
        let mut escape = event(27);
        assert_eq!(
            ((*f.client).overlay_data().popup()).key(&mut *f.client, &mut escape),
            1
        );
        let mut control_c = event(b'c' as key_code | KEYC_CTRL);
        assert_eq!(
            ((*f.client).overlay_data().popup()).key(&mut *f.client, &mut control_c),
            1
        );
        pd.borrow_mut().flags |= POPUP_CLOSEANYKEY;
        let mut ordinary = event(b'x' as key_code);
        assert_eq!(
            ((*f.client).overlay_data().popup()).key(&mut *f.client, &mut ordinary),
            1
        );

        let mut mouse = event(KEYC_MOUSE as key_code);
        mouse.m.x = 79;
        mouse.m.y = 23;
        mouse.m.b = MOUSE_BUTTON_1 as u_int;
        assert_eq!(
            ((*f.client).overlay_data().popup()).key(&mut *f.client, &mut mouse),
            0
        );
        assert_eq!((pd.borrow().lx, pd.borrow().ly), (0, 0));
    }
}

#[test]
fn mouse_drag_sequences_select_move_and_size_modes() {
    let f = Fixture::new("mouse", 10, 6, BOX_LINES_SINGLE);
    unsafe {
        let pd = f.pd();
        let mut move_event = event(KEYC_MOUSE as key_code);
        move_event.m.x = pd.borrow().px;
        move_event.m.y = pd.borrow().py + 2;
        move_event.m.lx = pd.borrow().px;
        move_event.m.ly = pd.borrow().py + 2;
        move_event.m.lb = MOUSE_BUTTON_1 as u_int;
        move_event.m.b = (MOUSE_BUTTON_1 | MOUSE_MASK_DRAG) as u_int;
        assert_eq!(
            ((*f.client).overlay_data().popup()).key(&mut *f.client, &mut move_event),
            0
        );
        assert_eq!(pd.borrow().dragging, MOVE);

        let mut release = move_event;
        release.m.b = MOUSE_BUTTON_1 as u_int;
        assert_eq!(
            ((*f.client).overlay_data().popup()).key(&mut *f.client, &mut release),
            0
        );
        assert_eq!(pd.borrow().dragging, OFF);

        let mut size_event = event(KEYC_MOUSE as key_code);
        size_event.m.x = pd.borrow().px + pd.borrow().sx - 1;
        size_event.m.y = pd.borrow().py + pd.borrow().sy - 1;
        size_event.m.lx = size_event.m.x;
        size_event.m.ly = size_event.m.y;
        size_event.m.lb = MOUSE_BUTTON_3 as u_int;
        size_event.m.b = (MOUSE_BUTTON_3 | MOUSE_MASK_DRAG) as u_int;
        assert_eq!(
            ((*f.client).overlay_data().popup()).key(&mut *f.client, &mut size_event),
            0
        );
        assert_eq!(pd.borrow().dragging, SIZE);
    }
}

#[test]
fn menu_actions_fill_center_close_and_ignore_unknown_keys() {
    let f = Fixture::new("actions", 10, 6, BOX_LINES_SINGLE);
    unsafe {
        let pd = f.pd();
        popup_menu_done(0, b'F' as key_code, pd.downgrade());
        assert_eq!(
            (
                pd.borrow().px,
                pd.borrow().py,
                pd.borrow().sx,
                pd.borrow().sy
            ),
            (0, 0, 80, 24)
        );
        pd.borrow_mut().sx = 10;
        pd.borrow_mut().sy = 6;
        popup_menu_done(0, b'C' as key_code, pd.downgrade());
        assert_eq!((pd.borrow().px, pd.borrow().py), (35, 9));
        popup_menu_done(0, b'x' as key_code, pd.downgrade());
        assert_eq!(pd.borrow().close, 0);
        popup_menu_done(0, b'q' as key_code, pd.downgrade());
        assert_eq!(pd.borrow().close, 1);
    }
}

#[test]
fn redraw_and_set_client_callbacks_mark_only_the_popup_client() {
    let f = Fixture::new("redraw", 10, 6, BOX_LINES_NONE);
    unsafe {
        let pd = f.pd();
        (*f.client).flags &= !(CLIENT_REDRAWOVERLAY as u64);
        let mut ctx = tty_ctx {
            arg: TtyCtxArg::Popup(pd.downgrade()),
            ..Default::default()
        };
        popup_redraw_cb(&ctx);
        assert_ne!((*f.client).flags & CLIENT_REDRAWOVERLAY as u64, 0);
        ctx.arg = TtyCtxArg::None;
        (*f.client).flags &= !(CLIENT_REDRAWOVERLAY as u64);
        popup_redraw_cb(&ctx);
        assert_eq!((*f.client).flags & CLIENT_REDRAWOVERLAY as u64, 0);
    }
}

#[test]
fn draw_callback_renders_border_title_and_body_to_buffered_terminal() {
    if crate::test_process::run() {
        return;
    }
    let f = Fixture::new("draw", 10, 6, BOX_LINES_SINGLE);
    unsafe {
        (*f.client).tty.term = Some(zeroed_term());
        (*f.client).tty.out = Some(Box::new(ByteBuffer::new()));
        (*f.client).tty.client = crate::server::client_ref_of(&*f.client).map(|c| c.downgrade());
        (*f.client).tty.sx = 80;
        (*f.client).tty.sy = 24;
        let pd = f.pd();
        let (ictx, screen) = {
            let pd = pd.borrow();
            (pd.ictx.clone(), pd.s.clone())
        };
        ictx.as_ref()
            .expect("an input owner has a parser")
            .parse_shared_screen(&screen, None, ByteBuffer::from(b"body".to_vec()));
        let mut rctx = screen_redraw_ctx::default();
        ((*f.client).overlay_data().popup()).draw(&mut *f.client, &mut rctx);
        assert_eq!((*f.client).overlay(), Overlay::Popup);
        assert!(!(*f.client).tty.out.as_mut().unwrap().is_empty());
    }
}

#[test]
fn style_reapplication_covers_explicit_valid_invalid_and_detached_session_paths() {
    let f = Fixture::new("styles", 10, 6, BOX_LINES_SINGLE);
    unsafe {
        let pd = f.pd();
        pd.borrow_mut().style = Some(c"fg=red,bg=blue,bold".to_owned());
        pd.borrow_mut().border_style = Some(c"fg=green,bg=yellow".to_owned());
        popup_reapply_styles(&mut pd.borrow_mut());
        assert_ne!(pd.borrow().defaults.fg, 8);
        assert_ne!(pd.borrow().border_cell.fg, 8);
        assert_eq!(pd.borrow().defaults.attr, 0);
        assert_eq!(pd.borrow().border_cell.attr, 0);

        pd.borrow_mut().style = Some(c"not-a-style".to_owned());
        pd.borrow_mut().border_style = Some(c"still-not-a-style".to_owned());
        popup_reapply_styles(&mut pd.borrow_mut());
        let session = (*f.client).attached_session();
        (*f.client).set_attached_session(None);
        popup_reapply_styles(&mut pd.borrow_mut());
        (*f.client).set_attached_session(session.as_ref());
    }
}

#[test]
fn borderless_and_tiny_resize_drag_paths_clamp_without_jobs() {
    let f = Fixture::new("tiny", 10, 6, BOX_LINES_SINGLE);
    unsafe {
        let pd = f.pd();
        (*f.client).tty.sx = 2;
        (*f.client).tty.sy = 2;
        pd.borrow_mut().psx = 1;
        pd.borrow_mut().psy = 2;
        pd.borrow_mut().ppx = 9;
        pd.borrow_mut().ppy = 9;
        ((*f.client).overlay_data().popup()).resize(&mut *f.client);
        assert_eq!((pd.borrow().sx, pd.borrow().sy), (1, 2));
        assert_eq!((pd.borrow().px, pd.borrow().py), (1, 0));

        pd.borrow_mut().border_lines = BOX_LINES_NONE;
        pd.borrow_mut().dragging = SIZE;
        pd.borrow_mut().px = 1;
        pd.borrow_mut().py = 1;
        let mut m = mouse_event {
            x: 1,
            y: 5,
            b: MOUSE_MASK_DRAG as u_int,
            ..Default::default()
        };
        popup_handle_drag(&mut *f.client, &mut pd.borrow_mut(), &m);
        assert_eq!(pd.borrow().sx, 1);
        m.x = 6;
        m.y = 1;
        popup_handle_drag(&mut *f.client, &mut pd.borrow_mut(), &m);
        assert_eq!(pd.borrow().sy, 2);
        m.x = 7;
        m.y = 8;
        popup_handle_drag(&mut *f.client, &mut pd.borrow_mut(), &m);
        assert_eq!((pd.borrow().sx, pd.borrow().sy), (6, 7));
        assert_eq!(
            (
                RustScreen::grid(&pd.borrow().s.borrow()).width(),
                RustScreen::grid(&pd.borrow().s.borrow()).height()
            ),
            (6, 7)
        );
    }
}

#[test]
fn mouse_context_menu_and_paste_markers_cover_special_key_routing() {
    let f = Fixture::new("context", 12, 8, BOX_LINES_SINGLE);
    unsafe {
        let pd = f.pd();
        let mut right = event(KEYC_MOUSE as key_code);
        right.m.x = 70;
        right.m.y = 20;
        right.m.b = MOUSE_BUTTON_3 as u_int;
        assert_eq!(
            ((*f.client).overlay_data().popup()).key(&mut *f.client, &mut right),
            0
        );
        assert!(pd.borrow().md.is_some());
        ((*f.client).overlay_data().popup()).resize(&mut *f.client);
        assert!(pd.borrow().md.is_none());

        pd.borrow_mut().flags |= POPUP_INTERNAL;
        right.m.x = pd.borrow().px;
        right.m.y = pd.borrow().py;
        assert_eq!(
            ((*f.client).overlay_data().popup()).key(&mut *f.client, &mut right),
            0
        );
        assert!(pd.borrow().md.is_some());
        ((*f.client).overlay_data().popup()).resize(&mut *f.client);

        pd.borrow_mut().flags |= POPUP_CLOSEANYKEY;
        for key in [KEYC_PASTE_START, KEYC_PASTE_END] {
            let mut paste = event(key);
            assert_eq!(
                ((*f.client).overlay_data().popup()).key(&mut *f.client, &mut paste),
                0
            );
        }
        let mut modified = event(b'x' as key_code | crate::tty::KEYC_META);
        assert_eq!(
            ((*f.client).overlay_data().popup()).key(&mut *f.client, &mut modified),
            1
        );
    }
}

#[test]
fn range_callback_covers_partial_overlap_left_right_and_zero_width() {
    let f = Fixture::new("range-edges", 10, 6, BOX_LINES_SINGLE);
    {
        let pd = f.pd();
        let cases = [
            (0, pd.borrow().py, 4),
            (2, pd.borrow().py, 5),
            (pd.borrow().px + pd.borrow().sx - 2, pd.borrow().py, 8),
            (pd.borrow().px, pd.borrow().py - 1, 12),
            (pd.borrow().px, pd.borrow().py + pd.borrow().sy, 12),
            (pd.borrow().px, pd.borrow().py + 1, 0),
        ];
        for (px, py, nx) in cases {
            let ranges = popup_ranges(&pd, px, py, nx);
            let r = ranges.borrow();
            assert!(r.used <= 2);
        }
    }
}

const CLOSE_STATUS: crate::server_state::LocalField<std::cell::Cell<i32>> =
    crate::server_state::LocalField::new(|state| &state.popup_test_close_status);

#[test]
fn overlay_clear_releases_popup_and_invokes_close_callback_once() {
    let f = Fixture::new("close", 10, 6, BOX_LINES_NONE);
    unsafe {
        CLOSE_STATUS.set(-1);
        let pd = f.pd();
        pd.borrow_mut().status = 37;
        pd.borrow_mut().cb = Some(Box::new(|status| CLOSE_STATUS.set(status)));
        assert!(pd.downgrade().upgrade().is_some());
        server_client_clear_overlay(&mut *f.client);
        assert_eq!(CLOSE_STATUS.get(), 37);
        assert!((*f.client).overlay_data().is_none());
    }
}

#[test]
fn menu_range_checks_retain_the_stored_popup_spans_and_count() {
    let fixture = Fixture::new("menu ranges", 10, 6, BOX_LINES_SINGLE);
    unsafe {
        let popup = fixture.pd();
        let mut right = event(KEYC_MOUSE as key_code);
        right.m.x = 70;
        right.m.y = 20;
        right.m.b = MOUSE_BUTTON_3 as u_int;
        assert_eq!(
            ((*fixture.client).overlay_data().popup()).key(&mut *fixture.client, &mut right),
            0
        );
        {
            let state = popup.borrow_mut();
            let py = state.py;
            let mut menu = state
                .md
                .as_ref()
                .expect("a right click opens a menu")
                .borrow_mut();
            menu.px = 10;
            menu.py = py;
            let mut ranges = state.r.borrow_mut();
            ranges.ensure_capacity(2);
            ranges.ranges[0] = visible_range { px: 0, nx: 4 };
            ranges.ranges[1] = visible_range { px: 14, nx: 16 };
            ranges.used = 1;
        }
        let py = popup.borrow().py;
        let owner = popup_ranges(&popup, 0, py, 30);
        let ranges = owner.borrow();
        assert_eq!(ranges.used, 1);
        assert_eq!((ranges.ranges[0].px, ranges.ranges[0].nx), (0, 4));
        assert_eq!((ranges.ranges[1].px, ranges.ranges[1].nx), (14, 16));
    }
}

#[test]
fn terminal_callbacks_ignore_closed_and_destroyed_popups() {
    let fixture = Fixture::new("stale callback", 10, 6, BOX_LINES_NONE);
    let mut ctx = tty_ctx::default();
    unsafe {
        let popup = fixture.pd();
        RustColourEngine.set_palette(Some(&mut popup.borrow_mut().palette), 3, 123);
        let owner = popup.downgrade();
        let retained = owner.upgrade().expect("the popup is active");
        popup_init_ctx_cb(&popup.borrow(), &mut ctx);
        server_client_clear_overlay(&mut *fixture.client);
        (*fixture.client).flags &= !(CLIENT_REDRAWOVERLAY as u64);
        popup_redraw_cb(&ctx);
        assert_eq!(popup_set_client_cb(&mut ctx, &mut *fixture.client), 0);
        assert_eq!((*fixture.client).flags & CLIENT_REDRAWOVERLAY as u64, 0);
        drop(popup);
        drop(retained);
        assert!(owner.upgrade().is_none());
        assert_eq!(RustColourEngine.get_palette(ctx.palette.as_ref(), 3), 123);
        popup_redraw_cb(&ctx);
        assert_eq!(popup_set_client_cb(&mut ctx, &mut *fixture.client), 0);
        assert_eq!((*fixture.client).flags & CLIENT_REDRAWOVERLAY as u64, 0);
    }
}

#[test]
fn range_handles_observe_nested_checks_and_outlive_the_popup() {
    let fixture = Fixture::new("retained ranges", 10, 6, BOX_LINES_SINGLE);
    unsafe {
        let owner = fixture.pd().downgrade();
        (*fixture.client).tty.client = Some(
            crate::server::client_ref_of(&*fixture.client)
                .unwrap()
                .downgrade(),
        );
        let ranges = crate::tty::tty_check_overlay_range(&mut (*fixture.client).tty, 0, 3, 20);
        let left = ranges.range_at(0).unwrap();
        let right = ranges.range_at(1).unwrap();
        assert_eq!((left.px, left.nx), (0, 4));
        assert_eq!((right.px, right.nx), (14, 6));

        let nested = crate::tty::tty_check_overlay_range(&mut (*fixture.client).tty, 0, 0, 30);
        let updated = ranges.range_at(0).unwrap();
        assert_eq!((updated.px, updated.nx), (0, 30));
        assert!(ranges.range_at(1).is_none());
        nested.borrow_mut().ranges[0].nx = 7;

        server_client_clear_overlay(&mut *fixture.client);
        assert!(owner.upgrade().is_none());
        let retained = ranges.range_at(0).unwrap();
        assert_eq!((retained.px, retained.nx), (0, 7));
    }
}

#[test]
fn a_menu_view_resolves_the_menu_currently_owned_by_the_popup() {
    let fixture = Fixture::new("menu view", 10, 6, BOX_LINES_SINGLE);
    unsafe {
        let mut right = event(KEYC_MOUSE as key_code);
        right.m.x = 70;
        right.m.y = 20;
        right.m.b = MOUSE_BUTTON_3 as u_int;
        assert_eq!(
            ((*fixture.client).overlay_data().popup()).key(&mut *fixture.client, &mut right),
            0
        );
        (*fixture.client).set_overlay_view(OverlayView::Menu);
        let original = (*fixture.client).current_overlay_data();
        let previous = fixture.pd().borrow_mut().md.take().unwrap();

        assert_eq!(
            ((*fixture.client).overlay_data().popup()).key(&mut *fixture.client, &mut right),
            0
        );
        let current = (*fixture.client).current_overlay_data();
        assert_ne!(current, original);
        assert_eq!(
            current.menu(),
            fixture.pd().borrow().md.as_ref().unwrap().clone()
        );
        previous.close(&mut *fixture.client);

        server_client_clear_overlay(&mut *fixture.client);
        assert!((*fixture.client).current_overlay_data().is_none());
    }
}

#[test]
fn popup_options_follow_the_explicit_session_before_the_clients_session() {
    let f = Fixture::new("option-context", 10, 6, BOX_LINES_SINGLE);
    let mut explicit = Session::new(99, "explicit");
    let mut window = Window::new(99, "explicit", 80, 24);
    link(&mut explicit, &mut window, 0);
    unsafe {
        let mut client = client_ref_of(&*f.client).unwrap();
        server_client_clear_overlay(client.as_client_mut());
        f.window
            .options()
            .set_string(c"popup-style", 0, c"fg=red", fmt_args![]);
        window
            .options()
            .set_string(c"popup-style", 0, c"fg=blue", fmt_args![]);
        let show = |c: &mut client, s: Option<&session>| {
            popup_display(
                POPUP_NOJOB,
                BOX_LINES_SINGLE,
                None,
                0,
                0,
                10,
                6,
                None,
                None,
                &[],
                None,
                None,
                c,
                s,
                None,
                None,
                None,
            )
        };
        let mut session = explicit.handle().clone();
        assert_eq!(show(client.as_client_mut(), Some(session.as_session())), 0);
        assert_eq!(f.pd().borrow().defaults.fg, 4);
        server_client_clear_overlay(client.as_client_mut());
        assert_eq!(show(client.as_client_mut(), None), 0);
        assert_eq!(f.pd().borrow().defaults.fg, 1);
        server_client_clear_overlay(client.as_client_mut());
        let current = session.as_session_mut().curw.take();
        assert_eq!(show(client.as_client_mut(), Some(session.as_session())), -1);
        assert!(client.overlay().is_none());
        session.as_session_mut().curw = current;
        client.set_attached_session(None);
        assert_eq!(show(client.as_client_mut(), Some(session.as_session())), 0);
        assert_eq!(f.pd().borrow().defaults.fg, 4);
        server_client_clear_overlay(client.as_client_mut());
        assert_eq!(show(client.as_client_mut(), None), -1);
        assert!(client.overlay().is_none());
    }
    unlink_all(&mut explicit);
}

#[test]
fn overlay_dispatch_data_retains_popup_storage_after_overlay_removal() {
    let fixture = Fixture::new("retained-overlay", 10, 6, BOX_LINES_SINGLE);
    unsafe {
        let data = (*fixture.client).current_overlay_data();
        let weak = data.clone().popup().downgrade();
        server_client_clear_overlay(&mut *fixture.client);
        assert!((*fixture.client).current_overlay_data().is_none());
        let retained = weak.upgrade().expect("dispatch still retains the popup");
        assert!(retained.borrow().client().is_none());
        drop(retained);
        drop(data);
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn popup_screen_remains_owned_after_the_overlay_releases_it() {
    let fixture = Fixture::new("retained-screen", 10, 6, BOX_LINES_NONE);
    unsafe {
        let screen = fixture.pd().borrow().s.clone();
        let weak = screen.downgrade();
        popup_write(&mut *fixture.client, b"retained");
        server_client_clear_overlay(&mut *fixture.client);
        assert!(weak.upgrade().is_some());
        {
            let borrowed = screen.borrow();
            let cell = crate::grid::grid_get_cell(RustScreen::grid(&borrowed), 0, 0);
            assert_eq!(cell.data.data[0], b'r');
            assert_eq!(borrowed.cursor().0, 8);
        }
        drop(screen);
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn popup_teardown_releases_metadata_before_menu_and_close_callbacks() {
    let fixture = Fixture::new("reentrant-close", 12, 8, BOX_LINES_SINGLE);
    unsafe {
        let owner = (*fixture.client).overlay_data().popup();
        let mut right = event(KEYC_MOUSE as key_code);
        right.m.x = 70;
        right.m.y = 20;
        right.m.b = MOUSE_BUTTON_3 as u_int;
        assert_eq!(owner.key(&mut *fixture.client, &mut right), 0);
        let menu_owner = owner.downgrade();
        owner.borrow().md.as_ref().unwrap().borrow_mut().cb = Some(Box::new(move |_, _| {
            let owner = menu_owner.upgrade().unwrap();
            assert!(owner.borrow().md.is_none());
            owner.borrow_mut().status = 17;
        }));
        let close_owner = owner.downgrade();
        owner.borrow_mut().cb = Some(Box::new(move |status| {
            assert_eq!(status, 17);
            let owner = close_owner.upgrade().unwrap();
            owner.borrow_mut().status = 23;
        }));
        server_client_clear_overlay(&mut *fixture.client);
        assert_eq!(owner.borrow().status, 23);
        assert!(owner.borrow().client().is_none());
    }
}

#[test]
fn popup_output_parsing_borrows_palette_metadata_separately_from_the_screen() {
    let fixture = Fixture::new("palette-borrow", 12, 8, BOX_LINES_NONE);
    unsafe {
        let owner = (*fixture.client).overlay_data().popup();
        let before = RustColourEngine.get_palette(Some(&owner.borrow().palette), 2);
        popup_write(&mut *fixture.client, b"\x1b]4;2;#112233\x07A");
        assert_ne!(
            RustColourEngine.get_palette(Some(&owner.borrow().palette), 2),
            before
        );
        let screen = owner.borrow().s.clone();
        let screen = screen.borrow();
        let cell = crate::grid::grid_get_cell(RustScreen::grid(&screen), 0, 0);
        assert_eq!(cell.data.data[0], b'A');
    }
}

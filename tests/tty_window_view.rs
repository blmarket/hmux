use hmux2::src::shared::client::{
    client, CLIENT_REDRAWSTATUS, CLIENT_REDRAWWINDOW, CLIENT_STATUSOFF, CLIENT_TERMINAL,
};
use hmux2::src::shared::pane::window_pane;
use hmux2::src::shared::screen::{screen, MODE_CURSOR};
use hmux2::src::shared::session::session;
use hmux2::src::shared::window::{window, winlink};
use hmux2::src::tty::{tty_update_client_offset, tty_window_offset};

#[test]
fn viewport_matches_tmux_cursor_panning_and_redraw_behavior() {
    let mut c = client::empty();
    let mut session = session::empty();
    let mut link = winlink::default();
    let mut window = window::default();
    let mut pane = window_pane::empty();
    let mut screen = screen::empty();
    c.session = &raw mut session;
    c.tty.client = &raw mut c;
    session.curw = &raw mut link;
    session.statuslines = 1;
    link.window = &raw mut window;
    window.active = &raw mut pane;
    pane.screen = &raw mut screen;
    (c.tty.sx, c.tty.sy) = (80, 24);
    let redraw = (CLIENT_REDRAWWINDOW | CLIENT_REDRAWSTATUS) as u64;

    // Golden results from the pinned tmux e880cf63 tty_window_offset1:
    // window size, cursor, cursor visible, manual pan, returned x/y/width/height.
    for (size, cursor, visible, pan, expected) in [
        ((60, 20), (59, 19), true, None, (0, 0, 60, 20)),
        ((80, 23), (79, 22), true, None, (0, 0, 80, 23)),
        ((200, 80), (0, 0), true, None, (0, 0, 80, 23)),
        ((200, 80), (80, 23), true, None, (40, 1, 80, 23)),
        ((200, 80), (100, 40), true, None, (60, 18, 80, 23)),
        ((200, 80), (120, 57), true, None, (80, 35, 80, 23)),
        ((200, 80), (121, 58), true, None, (120, 57, 80, 23)),
        ((200, 80), (199, 79), false, None, (0, 0, 80, 23)),
        ((200, 80), (0, 0), true, Some((190, 79)), (120, 57, 80, 23)),
        ((200, 80), (0, 0), false, Some((7, 9)), (7, 9, 80, 23)),
        ((100, 10), (99, 9), true, None, (20, 0, 80, 23)),
        ((60, 40), (59, 39), true, None, (0, 17, 80, 23)),
        ((100, 10), (0, 0), true, Some((99, 99)), (20, 0, 80, 23)),
        ((60, 40), (0, 0), true, Some((99, 99)), (0, 17, 80, 23)),
        ((60, 20), (0, 0), true, Some((99, 99)), (0, 0, 60, 20)),
    ] {
        (window.sx, window.sy) = size;
        (screen.cx, screen.cy) = cursor;
        screen.mode = if visible { MODE_CURSOR } else { 0 };
        c.pan_window = if pan.is_some() {
            (&raw mut window).cast()
        } else {
            std::ptr::null_mut()
        };
        (c.pan_ox, c.pan_oy) = pan.unwrap_or((0, 0));
        c.flags = CLIENT_TERMINAL as u64;
        (c.tty.oox, c.tty.ooy, c.tty.osx, c.tty.osy) = (u32::MAX, 0, 0, 0);
        unsafe { tty_update_client_offset(&raw mut c) };
        let view = tty_window_offset(&c.tty);
        assert_eq!(
            (view.ox, view.oy, view.sx, view.sy),
            expected,
            "size={size:?}, cursor={cursor:?}, pan={pan:?}"
        );
        let bigger = size.0 > 80 || size.1 > 23;
        assert_eq!(view.bigger, bigger);
        assert_eq!(!c.pan_window.is_null(), bigger && pan.is_some());
        if bigger && pan.is_some() {
            assert_eq!((c.pan_ox, c.pan_oy), (view.ox, view.oy));
        }
        assert_eq!(c.flags & redraw, redraw);

        // The bigger flag updates even when identical geometry needs no redraw.
        c.flags &= !redraw;
        c.tty.oflag = !bigger as i32;
        unsafe { tty_update_client_offset(&raw mut c) };
        assert_eq!(tty_window_offset(&c.tty), view);
        assert_eq!(c.flags & redraw, 0);
    }

    // Hiding status increases the drawable height by the configured line count.
    (window.sx, window.sy) = (80, 24);
    c.flags = CLIENT_TERMINAL as u64;
    unsafe { tty_update_client_offset(&raw mut c) };
    assert!(tty_window_offset(&c.tty).bigger);
    assert_eq!(tty_window_offset(&c.tty).sy, 23);
    c.flags = (CLIENT_TERMINAL | CLIENT_STATUSOFF) as u64;
    unsafe { tty_update_client_offset(&raw mut c) };
    assert!(!tty_window_offset(&c.tty).bigger);
    assert_eq!(tty_window_offset(&c.tty).sy, 24);
    assert_eq!(c.flags & redraw, redraw);

    // A client without a terminal retains its cached geometry and pan state.
    let cached = tty_window_offset(&c.tty);
    c.flags = 0;
    c.pan_window = (&raw mut window).cast();
    (window.sx, window.sy) = (300, 200);
    unsafe { tty_update_client_offset(&raw mut c) };
    assert_eq!(tty_window_offset(&c.tty), cached);
    assert_eq!(c.pan_window, (&raw mut window).cast());
    assert_eq!(c.flags, 0);
}

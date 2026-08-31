//! Unit tests for [`crate::overlay`]: the constants the module defines,
//! [`popup_present`]'s answer for a plain client, the size checks at the top
//! of [`popup_display`], and one whole popup cycle — display without a job
//! behind it (`POPUP_NOJOB`, so no process is ever spawned), the state that
//! setup fills in, [`popup_write`] feeding the job stream into the popup's
//! own screen, [`popup_modify`]'s title, style and flag updates, replacement
//! by a second display, and teardown through
//! [`server_client_clear_overlay`], which runs the module's private free
//! path, and the popup menu's open and selection paths.
//!
//! What stays unexercised is everything needing more than this process: the
//! job-backed branch of `popup_display` spawns a shell through a pty, the
//! editor's happy path spawns an editor over a temporary file, and the
//! private draw and resize callbacks want a live terminal attached to their
//! client. The private free path is reached here both directly and after a
//! popup menu selection.

use crate::environ::environ_t;
use crate::fmt_args;
use crate::grid::{grid_default_cell, grid_get_cell};
use crate::modes::window_buffer_editdata;
use crate::options::options_set_string;
use crate::overlay::{
    _PATH_BSHELL, _PATH_TMP, BOTTOM, BOX_LINES_DEFAULT, BOX_LINES_DOUBLE, BOX_LINES_HEAVY,
    BOX_LINES_NONE, BOX_LINES_PADDED, BOX_LINES_ROUNDED, BOX_LINES_SIMPLE, BOX_LINES_SINGLE,
    CLIENT_REDRAWOVERLAY, JOB_DEFAULTSHELL, JOB_KEEPWRITE, JOB_NOWAIT, JOB_PTY, KEYC_CTRL,
    KEYC_MASK_KEY, KEYC_MASK_TYPE, KEYC_MOUSE, LEFT, MOUSE_BUTTON_1, MOUSE_BUTTON_3,
    MOUSE_MASK_BUTTONS, MOUSE_MASK_CTRL, MOUSE_MASK_META, MOUSE_MASK_MODIFIERS, MOUSE_MASK_SHIFT,
    MOVE, NONE, OFF, PANE_CHANGED, POPUP_CLOSEANYKEY, POPUP_CLOSEEXIT, POPUP_CLOSEEXITZERO,
    POPUP_INTERNAL, POPUP_NOJOB, RIGHT, SIGHUP, SIZE, TOP, TTY_CTX_WINDOW_BIGGER, popup_data,
    popup_display, popup_editor, popup_key_cb, popup_modify, popup_present, popup_write,
};
use crate::screen::screen_grid;
use crate::screen::screen_grid_ptr;
use crate::server::server_client_clear_overlay;
use crate::tests::test_fixtures::{
    Clients, Session, Window, ensure_reactor, globals, link, seen, unlink_all,
};
use crate::tmux::global_options;
use crate::types::*;
use ::core::ffi::{c_char, c_int};
use ::core::ptr::{null, null_mut};

#[test]
fn the_popup_flags_are_single_bits_and_the_states_are_ladders() {
    assert_eq!(POPUP_CLOSEEXIT, 0x1);
    assert_eq!(POPUP_CLOSEEXITZERO, 0x2);
    assert_eq!(POPUP_INTERNAL, 0x4);
    assert_eq!(POPUP_CLOSEANYKEY, 0x8);
    assert_eq!(POPUP_NOJOB, 0x10);
    let all =
        POPUP_CLOSEEXIT | POPUP_CLOSEEXITZERO | POPUP_INTERNAL | POPUP_CLOSEANYKEY | POPUP_NOJOB;
    assert_eq!(all.count_ones(), 5);

    assert_eq!(OFF, 0);
    assert_eq!(MOVE, 1);
    assert_eq!(SIZE, 2);

    assert_eq!(NONE, 0);
    assert_eq!(LEFT, 1);
    assert_eq!(RIGHT, 2);
    assert_eq!(TOP, 3);
    assert_eq!(BOTTOM, 4);

    assert_eq!(BOX_LINES_DEFAULT, -1);
    for (smaller, larger) in [
        (BOX_LINES_SINGLE, BOX_LINES_DOUBLE),
        (BOX_LINES_DOUBLE, BOX_LINES_HEAVY),
        (BOX_LINES_HEAVY, BOX_LINES_SIMPLE),
        (BOX_LINES_SIMPLE, BOX_LINES_ROUNDED),
        (BOX_LINES_ROUNDED, BOX_LINES_PADDED),
        (BOX_LINES_PADDED, BOX_LINES_NONE),
    ] {
        assert_ne!(smaller, larger);
    }
}

#[test]
fn the_popup_paths_and_masks_read_as_the_c_headers_define_them() {
    unsafe {
        assert_eq!(seen(_PATH_BSHELL.as_ptr()), "/bin/sh");
        assert_eq!(seen(_PATH_TMP.as_ptr()), "/tmp/");
    }
    assert_eq!(
        MOUSE_MASK_MODIFIERS,
        MOUSE_MASK_SHIFT | MOUSE_MASK_META | MOUSE_MASK_CTRL
    );
    assert_ne!(MOUSE_BUTTON_1, MOUSE_BUTTON_3);
    assert_eq!(MOUSE_MASK_MODIFIERS & MOUSE_MASK_BUTTONS, 0);
    assert_eq!(KEYC_CTRL & KEYC_MASK_KEY, 0);
    assert_eq!(KEYC_CTRL & KEYC_MASK_TYPE, 0);
    let jobs = JOB_NOWAIT | JOB_KEEPWRITE | JOB_PTY | JOB_DEFAULTSHELL;
    assert_eq!(jobs.count_ones(), 4);
    assert_eq!(PANE_CHANGED, 0x80);
    assert_eq!(TTY_CTX_WINDOW_BIGGER, 0x4);
}

/// A client reporting 80 by 24, attached to an unregistered session whose
/// current window is linked — the chain a displayed popup walks for its
/// window options and its redraw focus. Everything is taken down when this
/// goes out of scope; a live popup must be cleared first. The guard is
/// declared last so that it is dropped last, keeping the process-global
/// state held until the fixtures have given back what they took.
struct Popup {
    session: Session,
    window: Window,
    clients: Clients,
    c: *mut client,
    _guard: ::std::sync::MutexGuard<'static, ()>,
}

impl Drop for Popup {
    fn drop(&mut self) {
        unlink_all(&mut self.session);
    }
}

impl Popup {
    /// Takes the process-global guard, then builds the client–session–window
    /// chain under it.
    fn new(name: &str) -> Popup {
        let guard = globals();
        ensure_reactor();
        let mut session = Session::new(0, name);
        let mut window = Window::new(0, "popup", 80, 24);
        link(&mut session, &mut window, 0);
        let mut clients = Clients::new();
        let c = clients.add(name, 80, 24);
        unsafe { (*c).session = session.ptr() };
        Popup {
            session,
            window,
            clients,
            c,
            _guard: guard,
        }
    }

    fn s(&mut self) -> *mut session {
        self.session.ptr()
    }

    /// The live popup behind the client, once one has been displayed.
    fn pd(&self) -> *mut popup_data {
        unsafe { (*self.c).overlay_data().popup() }
    }

    /// Asks for a popup of `sx` by `sy` at (2, 1) titled "fixture", never
    /// running a job for it.
    fn show(&mut self, flags: c_int, lines: box_lines, sx: u_int, sy: u_int) -> c_int {
        let s = self.session.ptr();
        unsafe {
            popup_display(
                flags,
                lines,
                null_mut::<cmdq_item>(),
                2,
                1,
                sx,
                sy,
                null_mut::<environ_t>(),
                null::<c_char>(),
                &[],
                null::<c_char>(),
                c"fixture".as_ptr(),
                self.c,
                s,
                null::<c_char>(),
                null::<c_char>(),
                None,
            )
        }
    }

    /// A 10 by 6 popup with the borders the window options ask for.
    fn display(&mut self, flags: c_int) -> c_int {
        self.show(flags, BOX_LINES_DEFAULT, 10, 6)
    }
}

#[test]
fn a_client_without_an_overlay_is_not_a_popup_and_popup_write_leaves_it_alone() {
    let _guard = globals();
    let mut clients = Clients::new();
    let c = clients.add("plain", 80, 24);
    unsafe {
        assert_eq!(popup_present(c), 0);

        let flags_before = (*c).flags;
        popup_write(c, b"ignored".as_ptr() as *const c_char, 7);
        assert_eq!((*c).flags, flags_before);
        assert!((*c).overlay_check().is_none());
        assert!((*c).overlay().is_none());
        assert!((*c).overlay_data().is_none());
        assert_eq!(popup_present(c), 0);

        server_client_clear_overlay(c);
        assert_eq!(popup_present(c), 0);
    }
}

#[test]
fn popup_display_refuses_sizes_that_cannot_hold_the_borders_or_the_terminal() {
    let mut p = Popup::new("sized");
    let refused: [(box_lines, u_int, u_int); 7] = [
        (BOX_LINES_NONE, 0, 6),
        (BOX_LINES_NONE, 10, 0),
        (BOX_LINES_SINGLE, 2, 6),
        (BOX_LINES_SINGLE, 10, 2),
        (BOX_LINES_DEFAULT, 2, 6),
        (BOX_LINES_SINGLE, 81, 6),
        (BOX_LINES_SINGLE, 10, 25),
    ];
    unsafe {
        for (lines, sx, sy) in refused {
            assert_eq!(
                p.show(POPUP_NOJOB, lines, sx, sy),
                -1,
                "a {sx}x{sy} popup with border lines {lines} should have been refused"
            );
            assert_eq!(popup_present(p.c), 0, "{lines} {sx}x{sy}");
            assert!((*p.c).overlay_data().is_none(), "{lines} {sx}x{sy}");
        }
    }
}

#[test]
fn a_jobless_popup_wires_the_overlay_and_answers_present() {
    let mut p = Popup::new("shown");
    unsafe {
        assert_eq!(p.display(POPUP_NOJOB), 0);

        let pd = p.pd();
        assert!(!pd.is_null());
        assert_eq!(popup_present(p.c), 1);

        assert_eq!((*pd).flags, POPUP_NOJOB);
        assert_eq!((*pd).border_lines, BOX_LINES_SINGLE);
        assert_eq!(seen((*pd).title_ptr()), "fixture");
        assert_eq!((*pd).status, 128 + SIGHUP);
        assert_eq!((*pd).px, 2);
        assert_eq!((*pd).py, 1);
        assert_eq!((*pd).sx, 10);
        assert_eq!((*pd).sy, 6);
        assert_eq!((*pd).ppx, (*pd).px);
        assert_eq!((*pd).ppy, (*pd).py);
        assert_eq!((*pd).psx, (*pd).sx);
        assert_eq!((*pd).psy, (*pd).sy);
        assert_eq!((*pd).dragging, OFF);
        assert_eq!((*pd).close, 0);
        assert!((*pd).job_id.is_none());
        assert!((*pd).ictx.is_some());
        assert!((*pd).item.is_none());
        assert!((*pd).close_cb.is_none());
        assert_eq!((*pd).palette.fg, 8);
        assert_eq!((*pd).palette.bg, 8);

        assert_eq!((*screen_grid_ptr(&mut (*pd).s)).sx, 8);
        assert_eq!((*screen_grid_ptr(&mut (*pd).s)).sy, 4);

        assert_eq!((*p.c).overlay_check(), OverlayCheck::Popup);
        assert_eq!((*p.c).overlay(), Overlay::Popup);

        server_client_clear_overlay(p.c);
        assert_eq!(popup_present(p.c), 0);
        assert!((*p.c).overlay().is_none());
        assert!((*p.c).overlay_data().is_none());
    }
}

#[test]
fn popup_write_feeds_the_job_stream_into_the_popup_screen_while_it_is_up() {
    let mut p = Popup::new("stream");
    unsafe {
        assert_eq!(p.display(POPUP_NOJOB), 0);
        let pd = p.pd();

        (*p.c).set_overlay_view(OverlayView::Nothing);

        popup_write(p.c, b"hi".as_ptr() as *const c_char, 2);

        assert!((*p.c).overlay_check().is_some());
        assert_eq!((*p.c).overlay_data().data(), OverlayData::Popup(pd));

        let mut gc = grid_default_cell;
        gc = grid_get_cell(screen_grid(&(*pd).s), 0, 0);
        assert_eq!(gc.data.data[0], b'h');
        gc = grid_get_cell(screen_grid(&(*pd).s), 1, 0);
        assert_eq!(gc.data.data[0], b'i');

        server_client_clear_overlay(p.c);
    }
}

#[test]
fn popup_modify_updates_the_title_styles_and_flags_of_the_live_popup() {
    let mut p = Popup::new("styled");
    unsafe {
        assert_eq!(p.display(POPUP_NOJOB), 0);
        let pd = p.pd();

        assert_eq!(
            popup_modify(p.c, null(), null(), null(), BOX_LINES_DEFAULT, -1),
            0
        );
        assert_eq!((*pd).flags, POPUP_NOJOB);

        assert_eq!(
            popup_modify(
                p.c,
                c"renamed".as_ptr(),
                null(),
                null(),
                BOX_LINES_DEFAULT,
                -1
            ),
            0
        );
        assert_eq!(seen((*pd).title_ptr()), "renamed");
        assert_eq!(
            popup_modify(
                p.c,
                c"again".as_ptr(),
                null(),
                null(),
                BOX_LINES_DEFAULT,
                -1
            ),
            0
        );
        assert_eq!(seen((*pd).title_ptr()), "again");

        assert_eq!(
            popup_modify(
                p.c,
                null(),
                c"bg=red".as_ptr(),
                null(),
                BOX_LINES_DEFAULT,
                -1
            ),
            0
        );
        assert_eq!((*pd).defaults.bg, 1);

        assert_eq!(
            popup_modify(
                p.c,
                null(),
                null(),
                c"fg=blue".as_ptr(),
                BOX_LINES_DEFAULT,
                -1
            ),
            0
        );
        assert_eq!((*pd).border_cell.fg, 4);

        (*p.c).flags &= !(CLIENT_REDRAWOVERLAY as u64);
        assert_eq!(
            popup_modify(
                p.c,
                null(),
                null(),
                null(),
                BOX_LINES_DEFAULT,
                POPUP_CLOSEEXIT | POPUP_CLOSEEXITZERO
            ),
            0
        );
        assert_eq!((*pd).flags, POPUP_CLOSEEXIT | POPUP_CLOSEEXITZERO);
        assert_ne!((*p.c).flags & CLIENT_REDRAWOVERLAY as u64, 0);

        server_client_clear_overlay(p.c);
    }
}

#[test]
fn displaying_over_a_live_popup_replaces_it_with_a_fresh_one() {
    let mut p = Popup::new("replaced");
    unsafe {
        assert_eq!(p.display(POPUP_NOJOB), 0);
        let first = p.pd();
        assert!(!first.is_null());

        assert_eq!(p.display(POPUP_NOJOB | POPUP_INTERNAL), 0);
        let second = p.pd();
        assert_ne!(second, first);
        assert_eq!(popup_present(p.c), 1);
        assert_eq!((*second).flags, POPUP_NOJOB | POPUP_INTERNAL);

        server_client_clear_overlay(p.c);
        assert_eq!(popup_present(p.c), 0);
    }
}

#[test]
fn popup_editor_declines_when_no_editor_is_configured() {
    let _guard = globals();
    unsafe {
        options_set_string(
            global_options,
            c"editor".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"".as_ptr()],
        );
        let rv = popup_editor(
            null_mut::<client>(),
            null(),
            0,
            None,
            Box::new(window_buffer_editdata {
                wp_id: 0,
                name: None,
                order: 0,
            }),
        );
        options_set_string(
            global_options,
            c"editor".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"/usr/bin/vi".as_ptr()],
        );
        assert_eq!(rv, -1);
    }
}

unsafe fn open_popup_menu(p: &mut Popup) {
    unsafe {
        let pd = p.pd();
        let mut event = key_event::default();
        event.key = KEYC_MOUSE as key_code;
        event.m.x = (*pd).px;
        event.m.y = (*pd).py;
        event.m.b = MOUSE_BUTTON_3 as u_int;
        assert_eq!(popup_key_cb(p.c, pd, &raw mut event), 0);
        assert!((*pd).md.is_some());
    }
}

/// Clearing a popup while its menu is open lets the popup free path reclaim
/// both the menu data and the menu itself.
#[test]
fn clearing_a_popup_with_an_open_menu_releases_the_menu_data() {
    let mut p = Popup::new("menu-clear");
    unsafe {
        assert_eq!(p.display(POPUP_NOJOB), 0);
        open_popup_menu(&mut p);
        server_client_clear_overlay(p.c);
        assert!((*p.c).overlay_data().is_none());
    }
}

/// Choosing Close frees the menu and then takes the overlay down from inside
/// the key callback, so the popup data is already gone when it returns and a
/// further clear finds nothing left.
#[test]
fn choosing_close_from_a_popup_menu_gives_up_the_menu_data() {
    let mut p = Popup::new("menu-choice");
    unsafe {
        assert_eq!(p.display(POPUP_NOJOB), 0);
        open_popup_menu(&mut p);
        let pd = p.pd();
        let mut event = key_event::default();
        event.key = b'q' as key_code;
        assert_eq!(popup_key_cb(p.c, pd, &raw mut event), 0);
        assert!((*p.c).overlay_data().is_none());
        server_client_clear_overlay(p.c);
        assert!((*p.c).overlay_data().is_none());
    }
}

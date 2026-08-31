//! Unit tests for [`crate::tty`] — the constants the transpiler
//! re-declared alongside `tty_draw_line`, the state-name table the drawing
//! loop logs, and the deterministic branches of the line drawer itself over a
//! terminal that reports no capabilities until a test gives it one.
//!
//! The drawer is driven directly, the way [`crate::screen`] calls
//! it: a fixture terminal whose output lands in a plain ensure_reactor buffer (the
//! shape the display-panes overlay tests stand up), a real screen holding the
//! source grid, and the default cell as defaults. With every capability
//! missing, attribute and cursor sequences expand to nothing, so the only
//! bytes a draw produces are what the clear fallback puts — literal spaces,
//! or whichever erase sequence a test hands the terminal — plus the text
//! itself, which lets every expectation be exact. The guard clauses, the
//! width clamp, the padding prefix of a wrapped cell, the wrap flag carried
//! between lines, a blocked terminal and the codeset fallback round out the
//! branches the fixtures can reach. Nothing here opens a descriptor, touches
//! a live terminal or runs the event loop.

use crate::grid::{grid_default_cell, grid_get_line};
use crate::grid::{grid_view_set_cell, grid_view_set_padding};
use crate::reactor::Buf;
use crate::terminfo::TtyCode;
use crate::tests::test_fixtures::{Screen, ascii, globals, zeroed_client, zeroed_term, zeroed_tty};
use crate::tty::TTY_BLOCK;
use crate::tty::{
    GRID_ATTR_CHARSET, GRID_FLAG_CLEARED, GRID_FLAG_PADDING, GRID_FLAG_SELECTED, GRID_FLAG_TAB,
    GRID_LINE_WRAPPED, MSG_COMMAND, MSG_FLAGS, MSG_READ_CANCEL, MSG_READ_OPEN, MSG_VERSION,
    TTY_DRAW_LINE_DONE, TTY_DRAW_LINE_EMPTY, TTY_DRAW_LINE_FIRST, TTY_DRAW_LINE_FLUSH,
    TTY_DRAW_LINE_NEW1, TTY_DRAW_LINE_NEW2, TTY_DRAW_LINE_SAME, TTY_NOCURSOR, TTYC_ACSC, TTYC_BCE,
    TTYC_ECH, TTYC_EL, TTYC_EL1, TTYC_XT, tty_draw_line,
};
use crate::types::*;
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;

/// A cell holding `bytes` as one character taking `width` columns. Nothing
/// checks that the two agree, which is what lets a test drive the codeset
/// fallback with a character a real grid would carry unchanged.
fn cell(bytes: &[u8], width: u8) -> grid_cell {
    let mut gc = unsafe { grid_default_cell };
    gc.data.data[..bytes.len()].copy_from_slice(bytes);
    gc.data.have = bytes.len() as u_char;
    gc.data.size = bytes.len() as u_char;
    gc.data.width = width;
    gc
}

/// Puts `text` on `screen`, one ASCII cell per byte from `px`.
fn write_text(screen: &Screen, px: u_int, py: u_int, text: &str) {
    for (i, &byte) in text.as_bytes().iter().enumerate() {
        let gc = ascii(byte);
        unsafe { grid_view_set_cell(&mut *screen.grid(), px + i as u_int, py, &gc) };
    }
}

/// A terminal that reports no capabilities until a test hands it one, whose
/// output lands in a buffer instead of a descriptor: a zeroed `tty` over a
/// zeroed `tty_term` and `client`. The term's code table is full-length and
/// missing, so cursor and attribute sequences expand to nothing
/// and the only bytes produced are the ones `tty_draw_line` puts itself.
struct Drawer {
    tty: Box<tty>,
    client: ClientRef,
    seen: usize,
}

impl Drawer {
    fn new(sx: u_int, sy: u_int) -> Drawer {
        let mut d = Drawer {
            tty: zeroed_tty(),
            client: zeroed_client(),
            seen: 0,
        };
        d.tty.term = Some(zeroed_term());
        d.tty.owner = crate::server::client_ref_from_ptr(&raw mut *d.client).map(|c| c.downgrade());
        d.tty.out = Some(Box::new(Buf::new()));
        d.tty.sx = sx;
        d.tty.sy = sy;
        d
    }

    fn ptr(&mut self) -> *mut tty {
        &raw mut *self.tty
    }

    fn term_mut(&mut self) -> &mut tty_term {
        self.tty.term.as_mut().expect("the fixture built a term")
    }

    /// Gives `code` a string value, as a terminfo entry carrying that
    /// capability would.
    fn set_string(&mut self, code: tty_code_code, s: &'static CStr) {
        self.term_mut().codes[code as usize] = TtyCode::String(s.to_owned());
    }

    /// Gives the ACS key `ch` the one-byte drawing character `to`.
    fn set_acs(&mut self, ch: u8, to: u8) {
        self.term_mut().acs[ch as usize] = [to as c_char, 0 as c_char];
    }

    /// What has been written since this was last asked.
    fn written(&mut self) -> Vec<u8> {
        let out = self.tty.out.as_mut().unwrap();
        let len = out.len();
        if len <= self.seen {
            return Vec::new();
        }
        let fresh = out.as_slice()[self.seen..].to_vec();
        self.seen = len;
        fresh
    }
}

/// Draws `nx` columns of screen row `py` at terminal position (`atx`, `aty`),
/// with the default cell as defaults and no palette, the way the redraw code
/// draws a plain pane line.
fn draw(d: &mut Drawer, s: &mut Screen, py: u_int, nx: u_int, atx: u_int, aty: u_int) {
    unsafe {
        tty_draw_line(
            &mut *d.ptr(),
            s.ptr(),
            0,
            py,
            nx,
            atx,
            aty,
            &grid_default_cell,
            null_mut::<colour_palette>(),
        );
    }
}

#[test]
fn the_reexported_constants_keep_their_values() {
    for (constant, value) in [
        (TTY_DRAW_LINE_FIRST, 0),
        (TTY_DRAW_LINE_FLUSH, 1),
        (TTY_DRAW_LINE_NEW1, 2),
        (TTY_DRAW_LINE_NEW2, 3),
        (TTY_DRAW_LINE_EMPTY, 4),
        (TTY_DRAW_LINE_SAME, 5),
        (TTY_DRAW_LINE_DONE, 6),
    ] {
        assert_eq!(constant, value);
    }
    for (constant, value) in [
        (GRID_LINE_WRAPPED, 0x1),
        (GRID_FLAG_PADDING, 0x4),
        (GRID_FLAG_SELECTED, 0x10),
        (GRID_FLAG_CLEARED, 0x40),
        (GRID_FLAG_TAB, 0x80),
        (GRID_ATTR_CHARSET, 0x80),
        (TTY_NOCURSOR, 0x1),
    ] {
        assert_eq!(constant, value);
    }
    for (constant, value) in [
        (MSG_VERSION, 12),
        (MSG_COMMAND, 200),
        (MSG_FLAGS, 218),
        (MSG_READ_OPEN, 300),
        (MSG_READ_CANCEL, 307),
        (TTYC_ACSC, 0),
        (TTYC_BCE, 3),
        (TTYC_ECH, 37),
        (TTYC_EL, 39),
        (TTYC_EL1, 40),
        (TTYC_XT, 232),
    ] {
        assert_eq!(constant, value);
    }
}

#[test]
fn drawing_outside_the_terminal_returns_before_anything_is_written() {
    let _guard = globals();
    let mut d = Drawer::new(80, 24);
    let mut s = Screen::new(80, 24, 100);
    unsafe {
        tty_draw_line(
            &mut *d.ptr(),
            s.ptr(),
            0,
            0,
            10,
            80,
            0,
            &grid_default_cell,
            null_mut::<colour_palette>(),
        );
        assert!(d.written().is_empty(), "an off-edge draw wrote bytes");
        assert_eq!((*d.ptr()).cx, 0);

        tty_draw_line(
            &mut *d.ptr(),
            s.ptr(),
            0,
            0,
            0,
            0,
            0,
            &grid_default_cell,
            null_mut::<colour_palette>(),
        );
        assert!(d.written().is_empty(), "a zero-width draw wrote bytes");
        assert_eq!((*d.ptr()).cx, 0);
        assert_eq!((*d.ptr()).cy, 0);
    }
}

#[test]
fn a_width_past_the_terminal_edge_is_clamped_and_cleared_with_spaces() {
    let _guard = globals();
    let mut d = Drawer::new(8, 24);
    let mut s = Screen::new(8, 24, 100);
    draw(&mut d, &mut s, 0, 100, 0, 0);
    assert_eq!(d.written(), b"        ", "the clamp did not fit the clear");
    unsafe { assert_eq!((*d.ptr()).cx, 8) };
}

#[test]
fn text_runs_are_flushed_and_the_background_cleared_behind_them() {
    let _guard = globals();
    let mut d = Drawer::new(10, 24);
    let mut s = Screen::new(10, 24, 100);
    write_text(&s, 0, 0, "abc");
    unsafe { (*d.ptr()).flags |= TTY_NOCURSOR };
    draw(&mut d, &mut s, 0, 10, 0, 0);
    assert_eq!(d.written(), b"abc       ");
    unsafe {
        assert_eq!((*d.ptr()).cx, 10, "the clear did not run off the end");
        assert_eq!(
            (*d.ptr()).flags & TTY_NOCURSOR,
            TTY_NOCURSOR,
            "the saved flags were not restored"
        );
    }
}

#[test]
fn an_el_capable_terminal_clears_to_the_end_of_the_line() {
    let _guard = globals();
    let mut d = Drawer::new(16, 24);
    d.set_string(TTYC_EL, c"\x1b[K");
    let mut s = Screen::new(16, 24, 100);
    draw(&mut d, &mut s, 0, 16, 0, 0);
    assert_eq!(d.written(), b"\x1b[K", "EL did not replace the spaces");
    unsafe { assert_eq!((*d.ptr()).cx, 0, "the fast path moved no cells") };
}

#[test]
fn el1_clears_a_run_that_starts_at_home_when_only_it_exists() {
    let _guard = globals();
    let mut d = Drawer::new(16, 24);
    d.set_string(TTYC_EL1, c"\x1b[1K");
    let mut s = Screen::new(16, 24, 100);
    draw(&mut d, &mut s, 0, 16, 0, 0);
    assert_eq!(d.written(), b"\x1b[1K");
    unsafe { assert_eq!((*d.ptr()).cx, 15, "the cursor did not reach the run") };
}

#[test]
fn ech_erases_a_counted_run_in_place() {
    let _guard = globals();
    let mut d = Drawer::new(20, 24);
    d.set_string(TTYC_ECH, c"\x1b[%dX");
    let mut s = Screen::new(20, 24, 100);
    draw(&mut d, &mut s, 0, 20, 0, 0);
    assert_eq!(d.written(), b"\x1b[20X");
    unsafe { assert_eq!((*d.ptr()).cx, 0, "the erasure moved the cursor") };
}

#[test]
fn leading_padding_cells_are_cleared_before_what_follows_them() {
    let _guard = globals();
    let mut d = Drawer::new(8, 24);
    let mut s = Screen::new(8, 24, 100);
    for px in 0..3 {
        unsafe { grid_view_set_padding(&mut *s.grid(), px, 0) };
    }
    write_text(&s, 3, 0, "xy");
    draw(&mut d, &mut s, 0, 8, 0, 0);
    assert_eq!(d.written(), b"   xy   ");
    unsafe { assert_eq!((*d.ptr()).cx, 8) };
}

#[test]
fn a_wrapped_previous_line_clears_without_moving_the_cursor() {
    let _guard = globals();

    let mut wrapped = Drawer::new(16, 24);
    let mut ws = Screen::new(16, 4, 100);
    unsafe {
        grid_get_line(&mut *ws.grid(), (*ws.grid()).hsize).flags |= GRID_LINE_WRAPPED;
        (*wrapped.ptr()).cx = 16;
        (*wrapped.ptr()).cy = 3;
    }
    draw(&mut wrapped, &mut ws, 1, 16, 0, 0);
    assert_eq!(wrapped.written(), b"                ");
    unsafe {
        assert_eq!(
            (*wrapped.ptr()).cy,
            4,
            "the cursor was moved before the wrap-aware clear"
        );
    }

    let mut plain = Drawer::new(16, 24);
    let mut ps = Screen::new(16, 4, 100);
    unsafe {
        (*plain.ptr()).cx = 16;
        (*plain.ptr()).cy = 3;
    }
    draw(&mut plain, &mut ps, 1, 16, 0, 0);
    assert_eq!(plain.written(), b"                ");
    unsafe {
        assert_eq!(
            (*plain.ptr()).cy,
            0,
            "the unwrapped clear did not start from home"
        );
    }
}

#[test]
fn a_blocked_terminal_counts_what_it_discards() {
    let _guard = globals();
    let mut d = Drawer::new(10, 24);
    let mut s = Screen::new(10, 24, 100);
    write_text(&s, 0, 0, "abc");
    unsafe { (*d.ptr()).flags |= TTY_BLOCK };
    draw(&mut d, &mut s, 0, 10, 0, 0);
    assert!(d.written().is_empty(), "a blocked terminal wrote bytes");
    unsafe {
        assert_eq!((*d.ptr()).discarded, 10, "the discard count is wrong");
        assert_eq!((*d.ptr()).flags & TTY_BLOCK, TTY_BLOCK);
    }
}

#[test]
fn characters_outside_the_codeset_become_acs_keys_or_underscores() {
    let _guard = globals();
    let mut d = Drawer::new(8, 24);
    let mut s = Screen::new(8, 24, 100);
    let gc = cell(&[0xe2, 0x94, 0x80], 1);
    unsafe { grid_view_set_cell(&mut *s.grid(), 0, 0, &gc) };

    draw(&mut d, &mut s, 0, 8, 0, 0);
    assert_eq!(d.written(), b"q       ", "the ACS key was not drawn");

    d.set_acs(b'q', b'A');
    draw(&mut d, &mut s, 0, 8, 0, 0);
    assert_eq!(d.written(), b"A       ", "the translation was not applied");

    let latin = cell(&[0xc3, 0xa9], 1);
    unsafe { grid_view_set_cell(&mut *s.grid(), 0, 0, &latin) };
    draw(&mut d, &mut s, 0, 8, 0, 0);
    assert_eq!(
        d.written(),
        b"_       ",
        "the untranslatable cell was not an underscore"
    );
}

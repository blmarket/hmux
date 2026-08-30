//! Characterization tests for the corners of [`crate::screen`] that
//! the module's own suite does not reach, kept in a file of their own so that
//! parallel efforts to widen coverage stay out of each other's way.
//!
//! Four groups of them. The first stands a *client* up — a zeroed terminal over
//! the shared [`Tty`] fixture's term, a session holding the window, and a
//! ensure_reactor buffer event over a socket pair for the bytes to land in — which is
//! what makes `tty_write` consult the pane's client callback instead of
//! answering at once; that callback is the module's one entirely unexercised
//! function. The second drives the *collect* path: the trimming of items that
//! already sit on a line, the give-up a synchronised update makes, and the
//! per-line flush against a visible range that a floating pane has cut in two.
//! The third drives the *combining* path with characters built by hand, since
//! the width and size a `utf8_data` carries are what decide every branch there.
//! The fourth pins the remaining single lines: clamps, wraparounds and the
//! guards in front of them.
//!
//! Everything here asserts what the transpiled code *does*. Where that is an
//! upstream oddity — a freed collected item still being read for its wrapped
//! flag, a pane sitting outside its own window making a width subtraction wrap
//! — the oddity is pinned, not fixed.

use crate::session::session_set_curw;
use crate::tests::test_fixtures::zeroed_term;
use crate::types::*;
use crate::window::window_set_active;

use crate::fmt_args;
use crate::grid::{grid_default_cell, grid_get_line, grid_string_cells};
use crate::grid::{grid_view_get_cell, grid_view_set_cell, grid_view_set_padding};
use crate::layout::LAYOUT_CELL_FLOATING;
use crate::screen::{
    GRID_FLAG_PADDING, GRID_FLAG_SELECTED, MODE_SYNC, MODE_WRAP, PANE_REDRAW, PANE_REDRAWSCROLLBAR,
    SCREEN_WRITE_CHECKED_IF_OBSCURED, TTY_CTX_INVISIBLE_PANES, TTY_CTX_SYNC, TTY_CTX_WINDOW_BIGGER,
    screen_write_cell, screen_write_clearcharacter, screen_write_clearendofline,
    screen_write_collect_add, screen_write_collect_end, screen_write_cursormove,
    screen_write_deletecharacter, screen_write_fast_copy, screen_write_fullredraw,
    screen_write_linefeed, screen_write_mode_clear, screen_write_mode_set, screen_write_preview,
    screen_write_puts, screen_write_rawstring, screen_write_scrollup, screen_write_start,
    screen_write_start_callback, screen_write_start_pane, screen_write_stop, screen_write_text,
    screen_write_vline, test_hooks,
};
use crate::screen::{screen_clear_selection, screen_grid_ptr, screen_set_selection};
use crate::tests::test_fixtures::{
    Clients, Pane, Screen, Session, Tty, Window, ascii, globals, link, unlink, unlink_all,
};
use ::core::ffi::{c_char, c_int};
use ::core::ptr::null_mut;

/// The flag a client carries while the whole of its window is waiting to be
/// redrawn, which the pane callback answers "later" to.
const CLIENT_REDRAWPANES: u64 = 0x20000000;
/// The flag `tty_sync_start` leaves on a terminal it has told to synchronise.
const TTY_SYNCING: c_int = 0x400;

/// A character built by hand: `bytes` is what it is written as and `width` how
/// many columns it takes. Nothing checks that the two agree, which is what lets
/// a test drive a branch that a real terminal would only reach with a real
/// character.
fn cell(bytes: &[u8], width: u8) -> grid_cell {
    let mut gc = unsafe { grid_default_cell };
    gc.data.data[..bytes.len()].copy_from_slice(bytes);
    gc.data.have = bytes.len() as u_char;
    gc.data.size = bytes.len() as u_char;
    gc.data.width = width;
    gc
}

/// U+0301, a combining acute accent: two bytes and no width of its own.
const ACUTE: &[u8] = &[0xcc, 0x81];
/// U+3042, a full-width Japanese letter.
const WIDE: &[u8] = &[0xe3, 0x81, 0x82];
/// U+1100, a leading Hangul consonant, which starts a syllable.
const CHOSEONG: &[u8] = &[0xe1, 0x84, 0x80];
/// U+1161, a Hangul vowel, which joins a leading consonant and nothing else.
const JUNGSEONG: &[u8] = &[0xe1, 0x85, 0xa1];
/// U+1F1FA and U+1F1F8, the regional indicators that spell a flag together.
const RI_U: &[u8] = &[0xf0, 0x9f, 0x87, 0xba];
const RI_S: &[u8] = &[0xf0, 0x9f, 0x87, 0xb8];
/// U+1F44B, a waving hand, and U+1F3FB, the lightest skin tone.
const HAND: &[u8] = &[0xf0, 0x9f, 0x91, 0x8b];
const TONE: &[u8] = &[0xf0, 0x9f, 0x8f, 0xbb];

/// A screen with a writing context over it and no pane behind it: `tty_write`
/// answers at once when nothing has set a client callback, so everything here
/// only touches the screen and its grid.
struct Writer {
    screen: Screen,
    ctx: Box<screen_write_ctx>,
}

impl Writer {
    fn new(sx: u_int, sy: u_int) -> Writer {
        let mut w = Writer {
            screen: Screen::new(sx, sy, 100),
            ctx: Box::new(screen_write_ctx::default()),
        };
        let s = w.screen.ptr();
        unsafe { screen_write_start(&mut w.ctx, s) };
        w
    }

    fn ptr(&mut self) -> *mut screen_write_ctx {
        &raw mut *self.ctx
    }

    fn s(&mut self) -> *mut screen {
        self.screen.ptr()
    }

    fn grid(&self) -> *mut grid {
        self.screen.grid()
    }

    fn flush(&mut self) {
        unsafe {
            screen_write_collect_end(&mut *self.ptr());
            test_hooks::collect_flush(&mut *self.ptr(), 0, c"test".as_ptr());
        }
    }

    fn lines(&mut self) -> Vec<String> {
        self.flush();
        lines_of(self.grid())
    }

    fn cursor(&mut self) -> (u_int, u_int) {
        unsafe { ((*self.s()).cx, (*self.s()).cy) }
    }

    fn move_to(&mut self, px: u_int, py: u_int) {
        unsafe { screen_write_cursormove(&mut *self.ptr(), px as c_int, py as c_int, 0) };
    }

    fn puts(&mut self, text: &str) {
        for byte in text.bytes() {
            let gc = ascii(byte);
            unsafe { screen_write_cell(&mut *self.ptr(), &raw const gc) };
        }
    }

    fn collect(&mut self, text: &str) {
        for byte in text.bytes() {
            let gc = ascii(byte);
            unsafe { screen_write_collect_add(&mut *self.ptr(), &raw const gc) };
        }
    }

    fn cell_at(&mut self, px: u_int, py: u_int) -> grid_cell {
        let mut gc = unsafe { grid_default_cell };
        unsafe { gc = grid_view_get_cell(&*self.grid(), px, py) };
        gc
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        unsafe { screen_write_stop(&mut self.ctx) };
    }
}

/// The visible screen, one string per line with trailing blanks cut.
fn lines_of(gd: *mut grid) -> Vec<String> {
    unsafe {
        (0..(*gd).sy)
            .map(|y| {
                let p = grid_string_cells(&*gd, 0, (*gd).hsize + y, (*gd).sx, None, 0, null_mut());
                p.to_string_lossy().trim_end().to_string()
            })
            .collect()
    }
}

/// A writer over a pane's own screen. The pane and its window are the
/// server-free fixtures; unless a test adds a client there are none, so
/// `tty_write` still answers at once and what runs is the pane-facing half of
/// the drawing code.
struct PaneWriter {
    window: Window,
    pane: Pane,
    ctx: Box<screen_write_ctx>,
}

impl PaneWriter {
    fn sized(sx: u_int, sy: u_int, wsx: u_int, wsy: u_int) -> PaneWriter {
        let mut w = PaneWriter {
            window: Window::new(1, "writer", wsx, wsy),
            pane: Pane::new(1, sx, sy, 100),
            ctx: Box::new(screen_write_ctx::default()),
        };
        w.window.add_pane(&mut w.pane);
        let (wp, screen) = (w.pane.ptr(), w.pane.screen());
        unsafe { screen_write_start_pane(&mut w.ctx, wp, screen) };
        w
    }

    fn new(sx: u_int, sy: u_int) -> PaneWriter {
        PaneWriter::sized(sx, sy, sx, sy)
    }

    fn ptr(&mut self) -> *mut screen_write_ctx {
        &raw mut *self.ctx
    }

    fn wp(&mut self) -> *mut window_pane {
        self.pane.ptr()
    }

    fn w(&mut self) -> *mut window {
        self.window.ptr()
    }

    fn grid(&mut self) -> *mut grid {
        unsafe { screen_grid_ptr(self.pane.screen()) }
    }

    fn move_to(&mut self, px: u_int, py: u_int) {
        unsafe { screen_write_cursormove(&mut *self.ptr(), px as c_int, py as c_int, 0) };
    }

    fn puts(&mut self, text: &str) {
        for byte in text.bytes() {
            let gc = ascii(byte);
            unsafe { screen_write_cell(&mut *self.ptr(), &raw const gc) };
        }
    }

    fn collect(&mut self, text: &str) {
        for byte in text.bytes() {
            let gc = ascii(byte);
            unsafe { screen_write_collect_add(&mut *self.ptr(), &raw const gc) };
        }
    }

    fn flush(&mut self) {
        unsafe {
            screen_write_collect_end(&mut *self.ptr());
            test_hooks::collect_flush(&mut *self.ptr(), 0, c"test".as_ptr());
        }
    }

    fn lines(&mut self) -> Vec<String> {
        self.flush();
        lines_of(self.grid())
    }
}

impl Drop for PaneWriter {
    fn drop(&mut self) {
        unsafe { screen_write_stop(&mut self.ctx) };
    }
}

/// A floating pane laid over `under`, in front of it on the z-index list — the
/// walk that looks for one goes backwards, so a pane only counts as being over
/// another when it sits *earlier* in that list. It takes itself back off that
/// list before its pane is freed, so it must not outlive the window it was laid
/// over.
struct Floating {
    pane: Pane,
    cell: Box<layout_cell>,
    window: *mut window,
}

impl Floating {
    fn over(
        under: *mut window_pane,
        w: *mut window,
        xoff: c_int,
        yoff: c_int,
        sx: u_int,
        sy: u_int,
    ) -> Floating {
        let mut f = Floating {
            pane: Pane::new(2, sx.max(1), sy.max(1), 20),
            cell: Box::new(layout_cell::default()),
            window: w,
        };
        f.cell.flags = LAYOUT_CELL_FLOATING;
        unsafe {
            let above = f.pane.hand_to(w);
            (*w).z_index.retain(|id| *id != (*above).id);
            (*above).layout_cell = &raw mut *f.cell;
            (*above).xoff = xoff;
            (*above).yoff = yoff;
            (*above).sx = sx;
            (*above).sy = sy;
            let at = (*w)
                .z_index
                .iter()
                .position(|id| *id == (*under).id)
                .unwrap();
            (*w).z_index.insert(at, (*above).id);
        }
        f
    }
}

impl Drop for Floating {
    fn drop(&mut self) {
        unsafe {
            let above = self.pane.ptr();
            (*self.window).z_index.retain(|id| *id != (*above).id);
        }
    }
}

/// A client the drawing code will write to: a session holding the window, a
/// terminal over a zeroed term whose capabilities are all missing, and an
/// output buffer for the bytes.
struct Attached {
    clients: Clients,
    session: Session,
    other: Session,
    tty: Tty,
    client: *mut client,
}

impl Drop for Attached {
    fn drop(&mut self) {
        unlink_all(&mut self.session);
    }
}

impl Attached {
    fn new(window: &mut Window, sx: u_int, sy: u_int) -> Attached {
        let mut a = Attached {
            clients: Clients::new(),
            session: Session::new(1, "attached"),
            other: Session::new(2, "elsewhere"),
            tty: Tty::new(),
            client: null_mut(),
        };
        link(&mut a.session, window, 0);
        a.client = a.clients.add("client", sx, sy);
        unsafe {
            let c = a.client;
            (*c).session = a.session.ptr();
            (*c).tty.owner = crate::server::client_ref_from_ptr(c).map(|c| c.downgrade());
            (*c).tty.term = Some(zeroed_term());
            (*c).tty.out = Some(Box::new(Buf::new()));
        }
        a
    }

    fn c(&mut self) -> *mut client {
        self.client
    }
}

/// The client callback answers for every client the terminal writing code walks
/// over, and everything it is asked stops at the first "no": a client looking at
/// another window, a pane with no cell in the layout, a pane already waiting to
/// be redrawn and a client whose panes are all waiting are each turned down
/// before the offsets are worked out.
#[test]
fn the_client_callback_turns_a_client_down_before_working_out_offsets() {
    let _guard = globals();
    let mut w = PaneWriter::new(6, 3);
    let mut a = Attached::new(&mut w.window, 20, 6);
    let mut layout = Box::new(layout_cell::default());
    let mut elsewhere = Window::new(2, "elsewhere", 6, 3);
    let cb = test_hooks::set_client_cb();
    unsafe {
        let (wp, c) = (w.wp(), a.c());
        let mut ttyctx = Box::new(tty_ctx::default());
        assert_eq!(ttyctx.arg, TtyCtxArg::None);
        test_hooks::initctx(&mut *w.ptr(), &mut ttyctx, 0, 0);
        assert_eq!(ttyctx.arg, TtyCtxArg::Pane(wp));

        (*wp).layout_cell = null_mut::<layout_cell>();
        assert_eq!(cb(&mut ttyctx, c), 0);

        (*wp).layout_cell = &raw mut *layout;
        (*wp).flags |= PANE_REDRAW;
        assert_eq!(cb(&mut ttyctx, c), -1);
        (*wp).flags &= !PANE_REDRAW;

        (*c).flags |= CLIENT_REDRAWPANES;
        assert_eq!(cb(&mut ttyctx, c), -1);
        assert_eq!(
            (*wp).flags & (PANE_REDRAW | PANE_REDRAWSCROLLBAR),
            PANE_REDRAW | PANE_REDRAWSCROLLBAR
        );
        (*c).flags &= !CLIENT_REDRAWPANES;
        (*wp).flags &= !(PANE_REDRAW | PANE_REDRAWSCROLLBAR);

        let wl = link(&mut a.session, &mut elsewhere, 1);
        session_set_curw(a.session.ptr(), wl);
        assert_eq!(cb(&mut ttyctx, c), 0);
        session_set_curw(
            a.session.ptr(),
            crate::window::winlink_find_by_index(&raw mut (*a.session.ptr()).windows, 0),
        );
        unlink(&mut a.session, wl);
    }
}

/// Once a client passes every check the callback fills the offsets in: the
/// window offset the terminal has cached decides the "window is bigger" flag,
/// and a status line at the top pushes the pane's own offset down by as many
/// lines as it takes.
#[test]
fn the_client_callback_fills_in_the_offsets_it_answers_with() {
    let _guard = globals();
    let mut w = PaneWriter::new(6, 3);
    let mut a = Attached::new(&mut w.window, 20, 6);
    let mut layout = Box::new(layout_cell::default());
    let cb = test_hooks::set_client_cb();
    unsafe {
        let (wp, c) = (w.wp(), a.c());
        (*wp).layout_cell = &raw mut *layout;
        (*wp).xoff = 3;
        (*wp).yoff = 4;
        let mut ttyctx = Box::new(tty_ctx::default());
        test_hooks::initctx(&mut *w.ptr(), &mut ttyctx, 0, 0);

        (*c).tty.oflag = 1;
        (*c).tty.oox = 7;
        (*c).tty.ooy = 8;
        (*c).tty.osx = 9;
        (*c).tty.osy = 10;
        assert_eq!(cb(&mut ttyctx, c), 1);
        assert_eq!(ttyctx.flags & TTY_CTX_WINDOW_BIGGER, TTY_CTX_WINDOW_BIGGER);
        assert_eq!(
            (ttyctx.wox, ttyctx.woy, ttyctx.wsx, ttyctx.wsy),
            (7, 8, 9, 10)
        );
        assert_eq!((ttyctx.rxoff, ttyctx.xoff), (3, 3));
        assert_eq!((ttyctx.ryoff, ttyctx.yoff), (4, 4));

        (*c).tty.oflag = 0;
        (*a.session.ptr()).statusat = 0;
        (*a.session.ptr()).statuslines = 2;
        assert_eq!(cb(&mut ttyctx, c), 1);
        assert_eq!(ttyctx.flags & TTY_CTX_WINDOW_BIGGER, 0);
        assert_eq!((ttyctx.ryoff, ttyctx.yoff), (4, 6));
    }
}

/// A write that may go to a pane nobody can see asks only whether the client's
/// session holds the window at all, and nothing else.
#[test]
fn the_client_callback_only_asks_about_the_session_for_an_invisible_pane() {
    let _guard = globals();
    let mut w = PaneWriter::new(6, 3);
    let mut a = Attached::new(&mut w.window, 20, 6);
    let cb = test_hooks::set_client_cb();
    unsafe {
        let (wp, c) = (w.wp(), a.c());
        (*wp).layout_cell = null_mut::<layout_cell>();
        (*wp).flags |= PANE_REDRAW;
        let mut ttyctx = Box::new(tty_ctx::default());
        test_hooks::initctx(&mut *w.ptr(), &mut ttyctx, 0, 0);
        ttyctx.flags |= TTY_CTX_INVISIBLE_PANES;
        assert_eq!(cb(&mut ttyctx, c), 1);

        (*c).session = a.other.ptr();
        assert_eq!(cb(&mut ttyctx, c), 0);
        (*c).session = a.session.ptr();
        (*wp).flags &= !PANE_REDRAW;
    }
}

/// The same callback reached the way the terminal writing code reaches it: a
/// raw string written with invisible panes allowed lands in the client's own
/// output buffer, and a full redraw tells the client's terminal to start a
/// synchronised update.
#[test]
fn a_client_is_written_to_through_the_callback() {
    let _guard = globals();
    let mut w = PaneWriter::new(6, 3);
    let mut a = Attached::new(&mut w.window, 20, 6);
    let mut layout = Box::new(layout_cell::default());
    unsafe {
        let (wp, c) = (w.wp(), a.c());
        (*wp).layout_cell = &raw mut *layout;
        let mut text = *b"hello";
        screen_write_rawstring(&mut *w.ptr(), text.as_mut_ptr(), text.len() as u_int, 1);
        let mut out = (*c).tty.out.as_ref().unwrap().clone();
        assert_eq!(out.as_slice(), b"hello");

        (*c).tty.flags &= !TTY_SYNCING;
        (*w.ptr()).flags = 0;
        screen_write_fullredraw(&mut *w.ptr());
        assert_eq!((*c).tty.flags & TTY_SYNCING, TTY_SYNCING);
    }
}

/// A writing context whose callback leaves the palette alone has nothing to
/// take the default colours from, so what the callback wrote stands.
#[test]
fn a_writer_callback_that_leaves_the_palette_alone_keeps_its_own_colours() {
    let _guard = globals();
    unsafe fn init(_ctx: &mut screen_write_ctx, ttyctx: &mut tty_ctx) {
        unsafe {
            ttyctx.defaults.fg = 8;
            ttyctx.defaults.bg = 8;
        }
    }
    let mut screen = Screen::new(4, 2, 100);
    let mut ctx = Box::new(screen_write_ctx::default());
    let s = screen.ptr();
    unsafe {
        screen_write_start_callback(&mut ctx, s, Some(init), null_mut::<popup_data>());
        let mut ttyctx = Box::new(tty_ctx::default());
        test_hooks::initctx(&mut ctx, &mut ttyctx, 0, 0);
        assert!(ttyctx.palette.is_null());
        assert_eq!((ttyctx.defaults.fg, ttyctx.defaults.bg), (8, 8));
        screen_write_stop(&mut ctx);
    }
}

/// Drawing into a pane that is not the window's active one is always
/// synchronised, whatever the call itself asked for.
#[test]
fn drawing_into_an_inactive_pane_is_synchronised() {
    let _guard = globals();
    let mut w = PaneWriter::new(6, 3);
    let mut other = Pane::new(2, 6, 3, 20);
    unsafe {
        window_set_active(w.w(), other.ptr());
        let mut ttyctx = Box::new(tty_ctx::default());
        test_hooks::initctx(&mut *w.ptr(), &mut ttyctx, 0, 0);
        assert_eq!(ttyctx.flags & TTY_CTX_SYNC, TTY_CTX_SYNC);
        window_set_active(w.w(), w.wp());
    }
}

/// A floating pane that starts above and to the left of the pane being drawn
/// still covers it: the second half of each of the two overlap tests is what
/// answers then.
#[test]
fn a_floating_pane_that_starts_outside_still_obscures() {
    let _guard = globals();
    let mut w = PaneWriter::new(6, 3);
    let base = w.wp();
    let window = w.w();
    let _over = Floating::over(base, window, -1, -1, 2, 2);
    unsafe {
        let mut ttyctx = Box::new(tty_ctx::default());
        test_hooks::initctx(&mut *w.ptr(), &mut ttyctx, 0, 1);
        assert_eq!(
            ttyctx.flags & crate::screen::TTY_CTX_PANE_OBSCURED,
            crate::screen::TTY_CTX_PANE_OBSCURED
        );
    }
}

/// A vertical line longer than the screen draws every cell it can and leaves
/// the cursor where it started: the row it asks for below the last one is
/// clamped to the last.
#[test]
fn a_vertical_line_longer_than_the_screen_is_clamped_to_it() {
    let _guard = globals();
    let mut w = Writer::new(4, 3);
    w.move_to(1, 0);
    unsafe { screen_write_vline(&mut *w.ptr(), 10, 1, 1) };
    assert_eq!(w.cursor(), (1, 0));
    assert_eq!(w.lines(), [" w", " x", " v"]);
}

/// Text that runs out of room exactly at a space breaks there, and the space
/// itself is dropped rather than written at the start of the next line.
#[test]
fn wrapped_text_that_breaks_on_a_space_drops_it() {
    let _guard = globals();
    let mut w = Writer::new(4, 2);
    let gc = unsafe { grid_default_cell };
    let done = unsafe {
        screen_write_text(
            &mut *w.ptr(),
            0,
            2,
            2,
            0,
            &raw const gc,
            c"%s".as_ptr(),
            fmt_args![c"ab cd".as_ptr()],
        )
    };
    assert_eq!(done, 0);
    assert_eq!(w.lines(), ["ab", "cd"]);
}

/// Writing a string stops reading a character the moment its bytes do not spell
/// one, and carries on with the byte after it; a character that does spell one
/// is written whole and counted by its width.
#[test]
fn writing_a_string_steps_over_bytes_that_spell_nothing() {
    let _guard = globals();
    let mut w = Writer::new(6, 2);
    let gc = unsafe { grid_default_cell };
    let broken = *b"\xc3(\0";
    unsafe {
        screen_write_puts(
            &mut *w.ptr(),
            &raw const gc,
            c"%s".as_ptr(),
            fmt_args![broken.as_ptr() as *const c_char],
        )
    };
    assert_eq!(w.cursor(), (0, 0));

    let good = *b"a\xc3\xa9b\0";
    unsafe {
        screen_write_puts(
            &mut *w.ptr(),
            &raw const gc,
            c"%s".as_ptr(),
            fmt_args![good.as_ptr() as *const c_char],
        )
    };
    assert_eq!(w.cursor(), (3, 0));
    assert_eq!(w.lines(), ["a\u{e9}b", ""]);
}

/// A fast copy stops at the column where a character would not fit whole, so a
/// double-width character straddling the right edge is left out.
#[test]
fn a_fast_copy_stops_at_a_character_that_would_not_fit() {
    let _guard = globals();
    let mut src = Writer::new(4, 1);
    src.move_to(2, 0);
    let wide = cell(WIDE, 2);
    unsafe { screen_write_cell(&mut *src.ptr(), &raw const wide) };
    src.flush();

    let mut dst = Writer::new(4, 1);
    unsafe {
        let from = src.s();
        screen_write_fast_copy(
            &mut *dst.ptr(),
            from,
            0,
            (*screen_grid_ptr(from)).hsize,
            3,
            1,
        );
    }
    assert_eq!(dst.lines(), [""]);
    assert_eq!(dst.cell_at(2, 0).data.size, 1);
}

/// A preview of a screen whose cursor is near the bottom slides the window it
/// shows up so that it still fits.
#[test]
fn a_preview_slides_up_to_fit_a_cursor_near_the_bottom() {
    let _guard = globals();
    let mut src = Writer::new(4, 6);
    src.move_to(0, 5);
    src.puts("zz");
    src.flush();

    let mut dst = Writer::new(4, 3);
    unsafe {
        (*src.s()).cx = 0;
        (*src.s()).cy = 5;
        screen_write_preview(&mut *dst.ptr(), src.s(), 4, 3);
    }
    assert_eq!(dst.lines(), ["", "", "zz"]);

    let mut top = Writer::new(4, 3);
    unsafe {
        (*src.s()).cy = 0;
        screen_write_preview(&mut *top.ptr(), src.s(), 4, 3);
    }
    assert_eq!(top.lines(), ["", "", ""]);
}

/// A pane sitting past the right-hand edge of its own window makes the width
/// left for it a subtraction below zero, which wraps: the redraw then clamps
/// what it writes back to the pane's own width rather than following it.
#[test]
fn a_pane_placed_outside_its_window_wraps_the_width_left_for_it() {
    let _guard = globals();
    let mut w = PaneWriter::sized(4, 2, 4, 2);
    unsafe {
        (*w.wp()).xoff = 6;
        w.puts("ab");
        screen_write_clearcharacter(&mut *w.ptr(), 1, 8);
    }
    assert_eq!(w.lines(), ["ab", ""]);
}

/// A floating pane leaving one column of a line visible has that column redrawn
/// on its own rather than as a whole line — but only when what it holds is a
/// plain single-width character. Anything else is redrawn as a line, and a cell
/// the selection has taken over is handed on with the selection's colours mixed
/// in.
#[test]
fn a_single_visible_column_is_redrawn_by_what_the_cell_holds() {
    let _guard = globals();
    let mut w = PaneWriter::new(4, 2);
    let (base, window) = (w.wp(), w.w());
    let _over = Floating::over(base, window, 2, 0, 2, 1);
    unsafe {
        let two_bytes = cell(&[0xc3, 0xa9], 1);
        grid_view_set_cell(&mut *w.grid(), 0, 0, &two_bytes);
        w.move_to(3, 0);
        screen_write_clearcharacter(&mut *w.ptr(), 1, 8);
        let mut gc = grid_default_cell;
        gc = grid_view_get_cell(&*w.grid(), 0, 0);
        assert_eq!(gc.data.size, 2);

        let mut chosen = ascii(b'a');
        chosen.flags |= GRID_FLAG_SELECTED as u_char;
        grid_view_set_cell(&mut *w.grid(), 0, 0, &chosen);
        let mut sel = grid_default_cell;
        screen_set_selection(w.pane.screen(), 0, 0, 0, 0, 0, 0, 0, &raw mut sel);
        screen_write_clearcharacter(&mut *w.ptr(), 1, 8);
        screen_clear_selection(w.pane.screen());
        gc = grid_view_get_cell(&*w.grid(), 0, 0);
        assert_eq!(gc.flags as c_int & GRID_FLAG_SELECTED, GRID_FLAG_SELECTED);

        let mut plain = ascii(b'a');
        plain.flags = 0;
        grid_view_set_cell(&mut *w.grid(), 0, 0, &plain);
        screen_write_clearcharacter(&mut *w.ptr(), 1, 8);
    }
}

/// A count of nothing is read as one wherever a count is asked for.
#[test]
fn a_count_of_nothing_is_read_as_one() {
    let _guard = globals();
    let mut w = Writer::new(4, 2);
    w.puts("abcd");
    w.move_to(0, 0);
    unsafe {
        screen_write_deletecharacter(&mut *w.ptr(), 0, 8);
        assert_eq!(w.lines(), ["bcd", ""]);
        screen_write_clearcharacter(&mut *w.ptr(), 0, 8);
    }
    assert_eq!(w.lines(), [" cd", ""]);
}

/// Scrolling more lines than the region holds is counted as the whole region,
/// and the pane's scrollbar is marked for redrawing once the scroll has been
/// written out.
#[test]
fn a_scroll_longer_than_the_region_is_cut_to_it() {
    let _guard = globals();
    let mut w = Writer::new(4, 4);
    w.puts("ab");
    unsafe {
        screen_write_scrollup(&mut *w.ptr(), 3, 8);
        screen_write_scrollup(&mut *w.ptr(), 3, 8);
    }
    assert_eq!(w.lines(), ["", "", "", ""]);

    let mut p = PaneWriter::new(4, 3);
    unsafe {
        p.move_to(0, 2);
        screen_write_linefeed(&mut *p.ptr(), 0, 8);
        assert_eq!((*p.wp()).flags & PANE_REDRAWSCROLLBAR, 0);
        p.flush();
        assert_eq!((*p.wp()).flags & PANE_REDRAWSCROLLBAR, PANE_REDRAWSCROLLBAR);
    }
}

/// A pane that has already been found not to be obscured keeps that answer even
/// once its window has shrunk under it; the scroll it then writes out has the
/// bottom of its region pulled back by however far the pane hangs off.
#[test]
fn a_scroll_from_a_pane_hanging_off_its_window_pulls_the_region_back() {
    let _guard = globals();
    let mut w = PaneWriter::new(4, 4);
    unsafe {
        screen_write_clearcharacter(&mut *w.ptr(), 1, 8);
        assert_eq!(
            (*w.ptr()).flags & SCREEN_WRITE_CHECKED_IF_OBSCURED,
            SCREEN_WRITE_CHECKED_IF_OBSCURED
        );
        (*w.w()).sy = 2;
        w.move_to(0, 3);
        screen_write_linefeed(&mut *w.ptr(), 0, 8);
        w.flush();
        assert_eq!((*w.wp()).flags & PANE_REDRAWSCROLLBAR, PANE_REDRAWSCROLLBAR);
        (*w.w()).sy = 4;
    }
}

/// A floating pane cuts the line being flushed in two. What is collected inside
/// each half is written from where that half starts; what falls into the gap
/// between them is written nowhere and stays collected.
#[test]
fn a_flush_writes_only_what_falls_inside_a_visible_range() {
    let _guard = globals();
    let mut w = PaneWriter::new(8, 2);
    let (base, window) = (w.wp(), w.w());
    let _over = Floating::over(base, window, 3, 0, 2, 1);
    w.collect("abcdefgh");
    assert_eq!(w.lines(), ["abcdefgh", ""]);

    let mut hidden = PaneWriter::new(8, 2);
    let (base, window) = (hidden.wp(), hidden.w());
    let _over = Floating::over(base, window, 3, 0, 2, 1);
    hidden.move_to(3, 0);
    hidden.collect("xy");
    unsafe {
        screen_write_collect_end(&mut *hidden.ptr());
        test_hooks::collect_flush(&mut *hidden.ptr(), 0, c"test".as_ptr());
        let cl = &(*(*hidden.pane.screen()).write_list.as_ptr().add(0)).items;
        assert!(
            !cl.is_empty(),
            "an item nothing could write stays collected"
        );
    }
}

/// A floating pane covering the whole of a line leaves a range of no width at
/// all, which every walk over the ranges steps past.
#[test]
fn a_line_covered_end_to_end_leaves_a_range_of_no_width() {
    let _guard = globals();
    let mut w = PaneWriter::new(4, 2);
    let (base, window) = (w.wp(), w.w());
    let _over = Floating::over(base, window, 0, 0, 4, 1);
    w.collect("ab");
    unsafe { screen_write_clearcharacter(&mut *w.ptr(), 1, 8) };
    assert_eq!(w.lines(), ["ab", ""]);
}

/// A screen in a synchronised update gives up everything collected rather than
/// writing it, including the items behind the first one on a line.
#[test]
fn a_synchronised_update_gives_up_every_collected_item() {
    let _guard = globals();
    let mut w = Writer::new(8, 2);
    w.collect("ab");
    unsafe {
        screen_write_collect_end(&mut *w.ptr());
        w.move_to(6, 0);
        screen_write_clearendofline(&mut *w.ptr(), 1);
        let cl = &(*(*w.s()).write_list.as_ptr().add(0)).items;
        assert_eq!(cl.len(), 2, "two items on the line");

        screen_write_mode_set(&mut *w.ptr(), MODE_SYNC);
        test_hooks::collect_flush(&mut *w.ptr(), 0, c"test".as_ptr());
        assert!((*(*w.s()).write_list.as_ptr().add(0)).items.is_empty());
        screen_write_mode_clear(&mut *w.ptr(), MODE_SYNC);
    }
}

/// Collecting over an item that is already there takes it out of the list, and
/// the wrapped flag of the item that went is carried onto the one that replaces
/// it — read out of the item after it has been handed back to the free list,
/// which is where the C read it too.
#[test]
fn a_collected_item_that_is_written_over_hands_on_its_wrapped_flag() {
    let _guard = globals();
    let mut w = Writer::new(4, 3);
    w.collect("abcde");
    unsafe {
        screen_write_collect_end(&mut *w.ptr());
        let wrapped = (*(*w.s()).write_list.as_ptr().add(1)).items[0];
        assert_eq!(crate::screen::citem(wrapped).wrapped, 1);

        w.move_to(0, 1);
        w.collect("xy");
        screen_write_collect_end(&mut *w.ptr());
        let now = (*(*w.s()).write_list.as_ptr().add(1)).items[0];
        assert_eq!(
            (
                crate::screen::citem(now).x,
                crate::screen::citem(now).used,
                crate::screen::citem(now).wrapped
            ),
            (0, 2, 1)
        );
    }
    assert_eq!(w.lines(), ["abcd", "xy", ""]);
}

/// An item written over in the middle is split in two around what replaced it,
/// and the item that came after it in the list is still found from the second
/// half.
#[test]
fn an_item_written_over_in_the_middle_is_split_around_it() {
    let _guard = globals();
    let mut w = Writer::new(8, 2);
    w.collect("abcd");
    unsafe {
        screen_write_collect_end(&mut *w.ptr());
        w.move_to(6, 0);
        screen_write_clearendofline(&mut *w.ptr(), 1);

        w.move_to(1, 0);
        w.collect("xy");
        screen_write_collect_end(&mut *w.ptr());

        let xs: Vec<(u_int, u_int)> = (*(*w.s()).write_list.as_ptr().add(0))
            .items
            .iter()
            .map(|&ci| (crate::screen::citem(ci).x, crate::screen::citem(ci).used))
            .collect();
        assert_eq!(xs, [(0, 1), (1, 2), (3, 1), (6, 2)]);
    }
    assert_eq!(w.lines(), ["axyd", ""]);
}

/// An item taken out of the list because it lies wholly inside what replaced it
/// is unlinked from the item after it as well.
#[test]
fn an_item_wholly_written_over_is_unlinked_from_the_one_after_it() {
    let _guard = globals();
    let mut w = Writer::new(8, 2);
    w.collect("ab");
    unsafe {
        screen_write_collect_end(&mut *w.ptr());
        w.move_to(4, 0);
        screen_write_clearendofline(&mut *w.ptr(), 1);

        w.move_to(0, 0);
        w.collect("xy");
        screen_write_collect_end(&mut *w.ptr());

        let items = &(*(*w.s()).write_list.as_ptr().add(0)).items;
        assert_eq!(items.len(), 2);
        let head = items[0];
        assert_eq!(
            (
                crate::screen::citem(head).x,
                crate::screen::citem(head).used
            ),
            (0, 2)
        );
        let next = items[1];
        assert_eq!(
            (
                crate::screen::citem(next).x,
                crate::screen::citem(next).used
            ),
            (4, 4)
        );
    }
}

/// Padding left in front of where text is collected is erased back to the
/// character that made it; a character of one column in front of that padding is
/// looked at for padding of its own and left alone.
#[test]
fn padding_in_front_of_collected_text_is_erased_back_to_a_plain_cell() {
    let _guard = globals();
    let mut w = Writer::new(6, 2);
    w.puts("ab");
    w.flush();
    unsafe {
        grid_view_set_padding(&mut *w.grid(), 2, 0);
        grid_view_set_padding(&mut *w.grid(), 3, 0);
    }
    w.move_to(3, 0);
    w.collect("z");
    unsafe { screen_write_collect_end(&mut *w.ptr()) };
    assert_eq!(w.lines(), ["ab z", ""]);
    assert_eq!(w.cell_at(1, 0).data.data[0], b'b');
}

/// A padding cell handed to the writing code is dropped: nothing is written and
/// the cursor stays where it was.
#[test]
fn a_padding_cell_is_not_written_at_all() {
    let _guard = globals();
    let mut w = Writer::new(4, 2);
    let mut gc = ascii(b'a');
    gc.flags |= GRID_FLAG_PADDING as u_char;
    unsafe { screen_write_cell(&mut *w.ptr(), &raw const gc) };
    assert_eq!(w.cursor(), (0, 0));
    assert_eq!(w.lines(), ["", ""]);
}

/// With wrapping off a double-width character that would run off the right-hand
/// edge is dropped, and so is anything written once the cursor has come to rest
/// past the last column.
#[test]
fn with_wrapping_off_a_character_that_does_not_fit_is_dropped() {
    let _guard = globals();
    let mut w = Writer::new(4, 2);
    unsafe { screen_write_mode_clear(&mut *w.ptr(), MODE_WRAP) };
    w.move_to(3, 0);
    let wide = cell(WIDE, 2);
    unsafe { screen_write_cell(&mut *w.ptr(), &raw const wide) };
    assert_eq!(w.cursor(), (3, 0));
    assert_eq!(w.lines(), ["", ""]);

    let mut past = Writer::new(4, 2);
    past.puts("abcd");
    assert_eq!(past.cursor(), (4, 0));
    unsafe { screen_write_mode_clear(&mut *past.ptr(), MODE_WRAP) };
    past.puts("e");
    assert_eq!(past.cursor(), (4, 0));
    assert_eq!(past.lines(), ["abcd", ""]);
}

/// Writing a cell that matches what is already there is skipped, and each of
/// the things that can differ is looked at in turn: the stored cell being an
/// extended one, its colours, and the width or size of the character being
/// written.
#[test]
fn a_cell_is_only_skipped_when_every_part_of_it_matches() {
    let _guard = globals();
    let mut w = Writer::new(6, 2);
    let accented = cell(&[b'a', 0xcc, 0x81], 1);
    unsafe { screen_write_cell(&mut *w.ptr(), &raw const accented) };
    w.move_to(0, 0);
    w.puts("a");
    assert_eq!(w.cell_at(0, 0).data.size, 1);

    let mut plain = Writer::new(6, 2);
    plain.puts("aaaa");
    unsafe {
        plain.move_to(0, 0);
        let mut fg = ascii(b'a');
        fg.fg = 1;
        screen_write_cell(&mut *plain.ptr(), &raw const fg);
        assert_eq!(plain.cell_at(0, 0).fg, 1);

        plain.move_to(1, 0);
        let mut bg = ascii(b'a');
        bg.bg = 2;
        screen_write_cell(&mut *plain.ptr(), &raw const bg);
        assert_eq!(plain.cell_at(1, 0).bg, 2);

        plain.move_to(2, 0);
        let two_bytes = cell(&[0xc3, 0xa9], 1);
        screen_write_cell(&mut *plain.ptr(), &raw const two_bytes);
        assert_eq!(plain.cell_at(2, 0).data.size, 2);
    }
}

/// A character carrying the selection flag written where there is no selection
/// has the flag taken off again before it is stored.
#[test]
fn a_selected_character_written_outside_a_selection_loses_the_flag() {
    let _guard = globals();
    let mut w = Writer::new(4, 2);
    let mut gc = ascii(b'a');
    gc.flags |= GRID_FLAG_SELECTED as u_char;
    unsafe { screen_write_cell(&mut *w.ptr(), &raw const gc) };
    assert_eq!(w.cell_at(0, 0).flags as c_int & GRID_FLAG_SELECTED, 0);
    assert_eq!(w.lines(), ["a", ""]);
}

/// Writing over a double-width character in a pane erases the padding it left
/// and redraws the whole line rather than the one cell.
#[test]
fn writing_over_a_wide_character_in_a_pane_redraws_the_line() {
    let _guard = globals();
    let mut w = PaneWriter::new(6, 2);
    let wide = cell(WIDE, 2);
    unsafe {
        screen_write_cell(&mut *w.ptr(), &raw const wide);
        w.move_to(0, 0);
        w.puts("x");
    }
    assert_eq!(w.lines(), ["x", ""]);
}

/// A combining character only joins the character in front of it when that one
/// is as wide as the step back taken to find it, so one written onto the second
/// half of a double-width character joins nothing and is dropped.
#[test]
fn a_combining_character_does_not_join_a_character_of_another_width() {
    let _guard = globals();
    let mut w = Writer::new(4, 2);
    let wide = cell(WIDE, 2);
    let acute = cell(ACUTE, 0);
    unsafe {
        screen_write_cell(&mut *w.ptr(), &raw const wide);
        w.move_to(1, 0);
        screen_write_cell(&mut *w.ptr(), &raw const acute);
    }
    assert_eq!(w.cell_at(0, 0).data.size, 3);
    assert_eq!(w.cursor(), (1, 0));
}

/// A Hangul jamo is looked at against the one in front of it: a leading
/// consonant starts a syllable of its own and is written, while a vowel with no
/// leading consonant in front of it composes nothing and is dropped.
#[test]
fn a_hangul_jamo_is_written_or_dropped_by_what_stands_in_front_of_it() {
    let _guard = globals();
    let mut w = Writer::new(4, 2);
    w.puts("a");
    let vowel = cell(JUNGSEONG, 1);
    unsafe { screen_write_cell(&mut *w.ptr(), &raw const vowel) };
    assert_eq!(w.cursor(), (1, 0));
    assert_eq!(w.cell_at(1, 0).data.size, 1);

    let lead = cell(CHOSEONG, 1);
    unsafe { screen_write_cell(&mut *w.ptr(), &raw const lead) };
    assert_eq!(w.cursor(), (2, 0));
    assert_eq!(w.cell_at(1, 0).data.size, 3);
}

/// Two characters that make one wider character together are joined and the
/// pair is made two columns wide, whichever way round the rule is read: a pair
/// of regional indicators spells a flag, and a skin tone written before an emoji
/// that takes one joins it as well.
#[test]
fn characters_that_make_one_wider_character_are_joined() {
    let _guard = globals();
    let mut flag = Writer::new(6, 2);
    unsafe {
        let first = cell(RI_U, 1);
        let second = cell(RI_S, 1);
        screen_write_cell(&mut *flag.ptr(), &raw const first);
        screen_write_cell(&mut *flag.ptr(), &raw const second);
    }
    assert_eq!(flag.cell_at(0, 0).data.size, 8);
    assert_eq!(flag.cell_at(0, 0).data.width, 2);
    assert_eq!(flag.cursor(), (2, 0));

    let mut toned = Writer::new(6, 2);
    unsafe {
        let hand = cell(HAND, 1);
        let tone = cell(TONE, 1);
        screen_write_cell(&mut *toned.ptr(), &raw const hand);
        screen_write_cell(&mut *toned.ptr(), &raw const tone);
    }
    assert_eq!(toned.cell_at(0, 0).data.size, 8);
    assert_eq!(toned.cell_at(0, 0).data.width, 2);
}

/// A character that will not fit beside the one in front of it is written on
/// its own instead, and a character of no width at all reaching that point is
/// stored as it stands — the width the cell already has is what is looked at
/// afterwards.
#[test]
fn a_character_that_will_not_fit_beside_the_last_is_written_on_its_own() {
    let _guard = globals();
    let mut w = Writer::new(6, 2);
    let long = cell(&[b'x'; 31], 1);
    unsafe {
        screen_write_cell(&mut *w.ptr(), &raw const long);
        w.puts("b");
        w.move_to(1, 0);
        let acute = cell(ACUTE, 0);
        screen_write_cell(&mut *w.ptr(), &raw const acute);
    }
    assert_eq!(w.cell_at(0, 0).data.size, 31);
    assert_eq!(w.cell_at(1, 0).data.size, 2);
    assert_eq!(w.cell_at(1, 0).data.width, 0);
}

/// A character combined onto the one before it in a pane is only written out to
/// the terminal when the columns it takes are visible; a row past the bottom of
/// the window has none, so the join is made on the grid and nothing else.
#[test]
fn a_join_in_a_row_nobody_can_see_is_made_but_not_written() {
    let _guard = globals();
    let mut w = PaneWriter::sized(6, 4, 6, 2);
    w.move_to(0, 3);
    w.puts("a");
    let acute = cell(ACUTE, 0);
    unsafe { screen_write_cell(&mut *w.ptr(), &raw const acute) };
    unsafe {
        let mut gc = grid_default_cell;
        gc = grid_view_get_cell(&*w.grid(), 0, 3);
        assert_eq!(gc.data.size, 3);
    }
}

/// Writing over the last column of a line whose padding runs to the edge walks
/// off the end of the line rather than finding a character to stop at.
#[test]
fn erasing_padding_that_runs_to_the_edge_stops_at_the_edge() {
    let _guard = globals();
    let mut w = Writer::new(4, 2);
    w.move_to(2, 0);
    let wide = cell(WIDE, 2);
    unsafe {
        screen_write_cell(&mut *w.ptr(), &raw const wide);
        w.move_to(3, 0);
        w.puts("x");
    }
    assert_eq!(w.cell_at(3, 0).data.data[0], b'x');
}

/// The line the cursor sits on is what a linefeed at the bottom of the region
/// scrolls, and the collected items move up with it.
#[test]
fn a_collected_line_moves_up_with_the_scroll() {
    let _guard = globals();
    let mut w = Writer::new(4, 3);
    w.collect("ab");
    unsafe {
        screen_write_collect_end(&mut *w.ptr());
        w.move_to(0, 2);
        screen_write_linefeed(&mut *w.ptr(), 0, 8);
        let gl = grid_get_line(&mut *w.grid(), (*w.grid()).hsize);
        assert_eq!(gl.flags & crate::screen::GRID_LINE_WRAPPED, 0);
    }
    assert_eq!(w.lines(), ["", "", ""]);
}

use crate::options::OptionsRef;
use crate::pane_geometry::PaneGeometryState;

use super::*;
use crate::cmd::{CmdqListOps, cmdq_next};

use crate::reactor::ByteBuffer;
use crate::server::client_ref_of;

use crate::terminfo::TerminalCapabilities;
use crate::tests::test_fixtures::{Pane, Target, globals, zeroed, zeroed_client, zeroed_term};
use crate::window::WINDOW_ZOOMED;
use ::core::ffi::{CStr, c_int, c_longlong};
use ::std::ffi::CString;

/// The only capability the fixture terminal has: a cursor move, written so
/// that the row and the column it was asked for can be read straight back
/// out of the output. Everything else — colours, attributes, clearing —
/// stays missing, so the bytes the terminal is handed are the cursor moves
/// and the cells this module puts, and nothing else.
const CUP: &CStr = c"<%p1%d,%p2%d>";

/// A client with a terminal whose bytes land in a buffer, looking at the
/// current window of a registered session. Everything the draw and the key
/// callbacks reach is here: `c->session->curw->window` and its panes, the
/// session options the colours come from, and the window options the pane
/// numbering comes from.
///
/// Panes added with [`Overlay::add_pane`] are kept in a field declared
/// ahead of the target, so they are dropped before the window they were
/// spliced into.
struct Overlay {
    extra: Vec<Pane>,
    t: Target,
    c: ClientRef,
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl Overlay {
    fn new(sx: u_int, sy: u_int) -> Overlay {
        let guard = globals();
        let mut t = Target::new(sx, sy);
        let mut term = zeroed_term();
        term.borrow_mut().apply_overrides(c"cup=<%p1%d,%p2%d>");
        let mut c = zeroed_client();
        (unsafe { c.as_client_mut() }).name = Some(c"draw-fixture".to_owned());
        unsafe { c.set_attached_session(Some(t.session_handle())) };
        unsafe { c.as_tty_mut() }.sx = sx;
        unsafe { c.as_tty_mut() }.sy = sy;
        unsafe { c.as_tty_mut() }.term = Some(term);
        unsafe { c.as_tty_mut() }.out = Some(Box::new(ByteBuffer::new()));
        unsafe { c.as_tty_mut() }.owner =
            crate::server::client_ref_of(&*(unsafe { c.as_client() })).map(|c| c.downgrade());
        Overlay {
            extra: Vec::new(),
            t,
            c,
            _guard: guard,
        }
    }

    fn client(&mut self) -> *mut client {
        unsafe { self.c.as_client_mut() }
    }

    fn pane(&mut self) -> *mut window_pane {
        self.t.pane(0)
    }

    /// Splices one more pane onto the end of the window's pane list, the
    /// way `window_add_pane` would. It is not the active one.
    fn add_pane(&mut self, sx: u_int, sy: u_int) {
        let id = 1 + self.extra.len() as u_int;
        self.extra.push(Pane::new(id, sx, sy, 100));
        let w = self.t.window(0);
        self.extra.last_mut().expect("just pushed").hand_to(w);
    }

    /// Numbers the window's panes from `base`, as `pane-base-index` does.
    fn base_index(&mut self, base: c_longlong) {
        let wo = unsafe { (*self.t.window(0)).options_ref() };
        unsafe { (*wo).set_number(c"pane-base-index", base) };
    }

    /// A redraw context covering the whole terminal, which a test moves and
    /// shrinks to put the panes outside it.
    fn ctx(&mut self) -> Box<screen_redraw_ctx> {
        let mut ctx = Box::new(screen_redraw_ctx::default());
        ctx.c = client_ref_of(unsafe { self.c.as_client() });
        ctx.sx = unsafe { self.c.as_tty() }.sx;
        ctx.sy = unsafe { self.c.as_tty() }.sy;
        ctx
    }

    /// Runs the overlay draw callback over `ctx`, exactly as the redraw
    /// code calls it.
    fn draw(&mut self, ctx: &mut screen_redraw_ctx) {
        let c = { &mut *(unsafe { self.c.as_client_mut() }) };
        unsafe { cmd_display_panes_draw(&mut crate::server::client_ref_of(c).unwrap(), ctx) };
    }

    /// Everything the terminal has been handed so far: the cursor moves
    /// [`CUP`] expands to and the cells that were put.
    fn written(&self) -> String {
        let mut out = unsafe { self.c.as_tty() }.out.as_ref().unwrap().clone();
        String::from_utf8(out.as_slice().to_vec()).expect("the fixture terminal is given ASCII")
    }
}

/// The colour a session option names, as the draw code reads it.
fn colour_option(f: &mut Overlay, name: &CStr) -> c_int {
    let session = { f.c.attached_session() }.unwrap();
    let oo = { session.options() };
    unsafe { oo.number(name) as c_int }
}

/// A window filling its terminal draws its one pane's number as clock cells
/// in the middle and its size in the top right corner, and leaves the
/// cursor at home. Each row of the zero takes one cursor move; within a row
/// the cells follow each other, so no further move is needed.
#[test]
fn a_pane_filling_the_context_draws_its_number_and_size() {
    let mut f = Overlay::new(80, 24);
    let mut ctx = f.ctx();
    f.draw(&mut ctx);
    assert_eq!(
        f.written(),
        "<10,37>     <11,37> <11,41> <12,37> <12,41> <13,37> <13,41> <14,37>     \
         <0,75>80x24<0,0>"
    );
    unsafe {
        assert_eq!((*f.client()).tty.cx, 0, "the cursor was not left at home");
        assert_eq!((*f.client()).tty.cy, 0);
    }
}

/// The active pane is drawn in `display-panes-active-colour` and every
/// other pane in `display-panes-colour`; the last attributes set are the
/// ones the terminal is left holding.
#[test]
fn the_active_pane_and_the_rest_take_their_own_colours() {
    let mut f = Overlay::new(80, 24);
    let active = colour_option(&mut f, c"display-panes-active-colour");
    let plain = colour_option(&mut f, c"display-panes-colour");
    assert_ne!(active, plain, "the two default colours are the same");

    let mut ctx = f.ctx();
    f.draw(&mut ctx);
    unsafe { assert_eq!((*f.client()).tty.cell.fg, active) };

    f.add_pane(80, 24);
    f.draw(&mut ctx);
    unsafe { assert_eq!((*f.client()).tty.cell.fg, plain) };
}

/// A pane hanging off the left and the top of the redraw context is drawn
/// from the context's own origin, with the width and the height that are
/// left.
#[test]
fn a_pane_off_the_left_and_top_keeps_only_what_is_inside() {
    let mut f = Overlay::new(80, 24);
    let mut ctx = f.ctx();
    ctx.ox = 10;
    ctx.oy = 4;
    f.draw(&mut ctx);
    assert_eq!(
        f.written(),
        "<8,32>     <9,32> <9,36> <10,32> <10,36> <11,32> <11,36> <12,32>     \
         <0,65>80x24\r"
    );
}

/// A pane wider and taller than the redraw context is drawn as if it were
/// exactly the context's size.
#[test]
fn a_pane_larger_than_the_context_is_drawn_at_the_context_size() {
    let mut f = Overlay::new(80, 24);
    let mut ctx = f.ctx();
    ctx.ox = 10;
    ctx.sx = 20;
    ctx.oy = 4;
    ctx.sy = 10;
    f.draw(&mut ctx);
    assert_eq!(
        f.written(),
        "<3,7>     <4,7> <4,11> <5,7> <5,11> <6,7> <6,11> <7,7>     \
         <0,15>80x24\r"
    );
}

/// A pane starting inside the context and running off its right and bottom
/// keeps its own size beyond the context's edges, which is where the C
/// stops clipping.
#[test]
fn a_pane_running_off_the_right_and_bottom_keeps_its_own_size() {
    let mut f = Overlay::new(80, 24);
    unsafe {
        (*f.pane()).set_position(10, 4);
    }
    let mut ctx = f.ctx();
    ctx.sx = 40;
    ctx.sy = 10;
    f.draw(&mut ctx);
    assert_eq!(
        f.written(),
        "<12,42>     <13,42> <13,46> <14,42> <14,46> <15,42> <15,46> <16,42>     \
         <4,75>80x24<0,0>"
    );
}

/// A status line at the top pushes every row of the drawing down by as many
/// lines as it has.
#[test]
fn a_top_status_line_pushes_the_drawing_down() {
    let mut f = Overlay::new(80, 24);
    let mut ctx = f.ctx();
    ctx.statustop = 1;
    ctx.statuslines = 2;
    f.draw(&mut ctx);
    assert_eq!(
        f.written(),
        "<12,37>     <13,37> <13,41> <14,37> <14,41> <15,37> <15,41> <16,37>     \
         <2,75>80x24<0,0>"
    );
}

/// A pane with fewer columns than its number has digits is skipped
/// outright — before the colours are read, before anything is drawn, and
/// without even putting the cursor back.
#[test]
fn a_pane_narrower_than_its_number_draws_nothing() {
    let mut f = Overlay::new(80, 24);
    f.base_index(10);
    let mut ctx = f.ctx();
    ctx.ox = 79;
    f.draw(&mut ctx);
    assert_eq!(f.written(), "");
}

/// A pane too small for the clock digits falls back to writing the number
/// itself on one line, followed by a space and the pane's letter.
#[test]
fn a_small_pane_writes_its_number_beside_its_letter() {
    let mut f = Overlay::new(80, 24);
    f.base_index(10);
    let mut ctx = f.ctx();
    ctx.oy = 20;
    ctx.sy = 4;
    f.draw(&mut ctx);
    assert_eq!(f.written(), "<2,38>10\u{0}\u{0} a<0,0>");
}

/// When even the number, the space and the letter will not fit, only the
/// number goes out, and its own length is what is used.
#[test]
fn a_pane_one_column_wide_writes_only_its_number() {
    let mut f = Overlay::new(80, 24);
    let mut ctx = f.ctx();
    ctx.ox = 79;
    f.draw(&mut ctx);
    assert_eq!(f.written(), "<12,0>0<0,0>");
}

/// A pane exactly tall enough for the clock digits but no taller drops the
/// size and the letter: they need a seventh row.
#[test]
fn a_pane_six_rows_tall_drops_its_size_and_letter() {
    let mut f = Overlay::new(80, 6);
    f.base_index(10);
    let mut ctx = f.ctx();
    f.draw(&mut ctx);
    let written = f.written();
    assert!(!written.contains("80x6"), "the size was drawn: {written}");
    assert!(written.ends_with("<0,0>"), "no cursor reset: {written}");
}

/// Panes past the ninth get a letter as well as a number: `a` for the
/// eleventh, written under the right-hand end of the digits.
#[test]
fn a_pane_past_the_ninth_gets_a_letter_under_its_number() {
    let mut f = Overlay::new(80, 24);
    f.base_index(10);
    let mut ctx = f.ctx();
    f.draw(&mut ctx);
    let written = f.written();
    assert!(
        written.ends_with("<0,75>80x24<15,44>a<0,0>"),
        "the size and the letter are missing: {written}"
    );
}

/// A zoomed window shows only its active pane, so that is the only pane
/// the numbers go over.
#[test]
fn a_zoomed_window_numbers_only_its_active_pane() {
    let mut f = Overlay::new(80, 24);
    f.add_pane(80, 24);
    unsafe { (*f.t.window(0)).flags |= WINDOW_ZOOMED };
    let mut ctx = f.ctx();
    f.draw(&mut ctx);
    assert_eq!(
        f.written(),
        "<10,37>     <11,37> <11,41> <12,37> <12,41> <13,37> <13,41> <14,37>     \
         <0,75>80x24<0,0>"
    );
}

/// The size in the corner is dropped when the pane's own size needs more
/// columns than the pane has left inside the redraw context — the size is
/// the pane's, the room for it is the visible part's.
#[test]
fn a_size_wider_than_the_visible_pane_is_dropped() {
    let mut f = Overlay::new(1000, 24);
    let mut ctx = f.ctx();
    ctx.ox = 994;
    f.draw(&mut ctx);
    assert_eq!(
        f.written(),
        "<10,0>     \r\n <11,4> \r\n <12,4> \r\n <13,4> \r\n     <0,0>"
    );
}

/// A template that does not parse is reported rather than run: the parser's
/// message goes onto the client's own queue as an error item, and the key
/// is still swallowed.
#[test]
fn a_template_that_will_not_parse_queues_its_error_on_the_client() {
    let mut f = Overlay::new(80, 24);
    let mut state = zeroed::<args_command_state>();
    let mut cdata = zeroed::<cmd_display_panes_data>();
    let mut event = key_event::default();
    unsafe {
        let queue =
            f.c.as_client_mut()
                .queue
                .insert(CmdqListRef::empty())
                .clone();
        state.cmd = Some(CString::new("not-a-command").unwrap());
        cdata.state = Some(state);
        event.key = '0' as i32 as key_code;

        assert_eq!((DisplayPanesRef::new(*cdata)).key(&f.c, &event), 1);

        assert!(!queue.is_empty(), "the error was not queued");
        let head = queue.item_at(0).unwrap().as_ptr();
        assert!((*head).client().is_some_and(|client| client.ptr_eq(&f.c)));
        assert_eq!(cmdq_next(Some(&f.c)), 1);
        assert_eq!(
            f.c.as_client()
                .message_string
                .as_deref()
                .expect("client text")
                .to_string_lossy()
                .into_owned(),
            "Unknown command: not-a-command"
        );

        f.c.as_client_mut().queue = None;
    }
}

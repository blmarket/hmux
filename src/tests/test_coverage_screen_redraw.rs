//! Characterization tests for [`crate::screen`], kept in a file of
//! their own so that parallel efforts to widen coverage stay out of each
//! other's way.
//!
//! Three groups of them. The first pins the metadata the module publishes: the
//! rings of border and cell constants, the glyph tables those constants index,
//! the client redraw bits that fold into `CLIENT_ALLREDRAWFLAGS`, and the bidi
//! isolate markers wrapped around a border line on a UTF-8 terminal. The
//! second drives the two public helpers behind every pane draw — asking
//! whether one column falls inside a set of visible ranges, and turning a
//! wanted stretch of a row into the ranges that survive the panes stacked over
//! it. The third drives the geometry under the border drawing itself, which
//! cannot be reached without standing a whole redraw up over a real terminal:
//! the dresser that gives a border cell its character for each
//! `pane-border-lines` style, the walk that answers whether a window is
//! exactly two panes, the classifier that says which edge of a pane a
//! coordinate sits on, and the neighbour probe and bit table that turn edges
//! into one of the thirteen junction cells.
//!
//! Everything here asserts what the transpiled code *does*. Where that is an
//! upstream shape rather than a tidy one — a fully interior cell classifying as
//! outside, a window's bottom row reading as vertical line — the shape is
//! pinned, not fixed. Nothing here opens a terminal: the helpers dress
//! caller-owned cells and fill caller-owned range buffers, and the client a
//! probe needs carries its session chain but never draws.

use crate::grid::grid_default_cell;
use crate::layout::LAYOUT_CELL_FLOATING;
use crate::options::options_set_number;
use crate::screen::{
    BORDER_MARKERS, CELL_BORDERS, CELL_BOTTOMJOIN, CELL_BOTTOMLEFT, CELL_BOTTOMRIGHT, CELL_INSIDE,
    CELL_JOIN, CELL_LEFTJOIN, CELL_LEFTRIGHT, CELL_OUTSIDE, CELL_RIGHTJOIN, CELL_SCROLLBAR,
    CELL_TOPBOTTOM, CELL_TOPJOIN, CELL_TOPLEFT, CELL_TOPRIGHT, CLIENT_ALLREDRAWFLAGS,
    CLIENT_REDRAWBORDERS, CLIENT_REDRAWOVERLAY, CLIENT_REDRAWPANES, CLIENT_REDRAWSCROLLBARS,
    CLIENT_REDRAWSTATUS, CLIENT_REDRAWSTATUSALWAYS, CLIENT_REDRAWWINDOW, CLIENT_SUSPENDED,
    CLIENT_UTF8, END_ISOLATE, GRID_ATTR_CHARSET, GRID_ATTR_REVERSE, LAYOUT_LEFTRIGHT,
    LAYOUT_TOPBOTTOM, PANE_LINES_DOUBLE, PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE,
    PANE_LINES_SINGLE, PANE_LINES_SPACES, PANE_SCROLLBARS_LEFT, PANE_SCROLLBARS_MODAL,
    PANE_SCROLLBARS_OFF, PANE_SCROLLBARS_RIGHT, PANE_STATUS_BOTTOM, PANE_STATUS_OFF,
    PANE_STATUS_TOP, SCREEN_REDRAW_BORDER_BOTTOM, SCREEN_REDRAW_BORDER_LEFT,
    SCREEN_REDRAW_BORDER_RIGHT, SCREEN_REDRAW_BORDER_TOP, SCREEN_REDRAW_INSIDE,
    SCREEN_REDRAW_OUTSIDE, SIMPLE_BORDERS, START_ISOLATE, screen_redraw_border_set,
    screen_redraw_cell_border, screen_redraw_check_is, screen_redraw_clip_visible_ranges,
    screen_redraw_is_visible, screen_redraw_pane_border, screen_redraw_two_panes,
    screen_redraw_type_of_cell,
};
use crate::terminfo::{tty_acs_double_borders, tty_acs_heavy_borders};
use crate::tests::test_fixtures::{Pane, Session, Window, globals, link, unlink, zeroed_client};
use crate::types::{
    ClientRef, client, grid_cell, layout_cell, layout_type, screen_redraw_ctx, u_char, u_int,
    u_short, utf8_data, visible_range, visible_ranges, window, window_pane, winlink,
};
use ::core::ffi::c_int;
use ::core::ptr::null_mut;

/// A character of one column spelling the bytes `s`.
fn glyph(s: &[u8]) -> utf8_data {
    let mut ud = utf8_data {
        data: [0; 32],
        have: s.len() as u_char,
        size: s.len() as u_char,
        width: 1,
    };
    ud.data[..s.len()].copy_from_slice(s);
    ud
}

/// A default cell whose attribute starts at `attr` and whose character is
/// deliberately dirty, so a dresser that leaves either alone shows itself.
fn undressed(attr: u_short) -> grid_cell {
    let mut gc = unsafe { grid_default_cell };
    gc.attr = attr;
    gc.data = glyph(b"?");
    gc
}

/// Asserts the cell holds exactly the character `want`.
fn expect_dressed_as(gc: &grid_cell, want: utf8_data) {
    assert_eq!(
        (gc.data.data, gc.data.have, gc.data.size, gc.data.width),
        (want.data, want.have, want.size, want.width)
    );
}

/// A window with hand-placed panes and nothing registered anywhere. Panes are
/// kept ahead of the window they were spliced into, so they go back off it
/// before the window itself is dropped.
struct Pair {
    panes: Vec<Pane>,
    window: Window,
}

impl Pair {
    fn new(wsx: u_int, wsy: u_int) -> Pair {
        Pair {
            panes: Vec::new(),
            window: Window::new(3, "pair", wsx, wsy),
        }
    }

    /// Adds a pane of `sx` by `sy` sitting at (`xoff`, `yoff`) in the window,
    /// answering its position.
    fn add(&mut self, xoff: c_int, yoff: c_int, sx: u_int, sy: u_int) -> usize {
        let id = 1 + self.panes.len() as u_int;
        let mut pane = Pane::new(id, sx, sy, 20);
        unsafe {
            (*pane.ptr()).xoff = xoff;
            (*pane.ptr()).yoff = yoff;
        }
        self.window.add_pane(&mut pane);
        self.panes.push(pane);
        self.panes.len() - 1
    }

    fn w(&mut self) -> *mut window {
        self.window.ptr()
    }

    fn wp(&mut self, i: usize) -> *mut window_pane {
        self.panes[i].ptr()
    }
}

/// A [`Pair`] looked at by a client through a session holding the window,
/// which is the chain the neighbour probes read out of the redraw context. The
/// client itself never draws: nothing here reaches the terminal.
struct Viewed {
    client: ClientRef,
    pair: Pair,
    session: Session,
    wl: *mut winlink,
}

impl Viewed {
    /// A window of `wsx` by `wsy` holding one pane that fills it.
    fn new(wsx: u_int, wsy: u_int) -> Viewed {
        let mut v = Viewed::empty(wsx, wsy);
        v.add(0, 0, wsx, wsy);
        v
    }

    /// A window with no panes yet, for tests that place their own.
    fn empty(wsx: u_int, wsy: u_int) -> Viewed {
        let mut v = Viewed {
            client: zeroed_client(),
            pair: Pair::new(wsx, wsy),
            session: Session::new(4, "viewed"),
            wl: null_mut(),
        };
        v.wl = link(&mut v.session, &mut v.pair.window, 0);
        unsafe { (*v.c()).session = v.session.ptr() };
        v
    }

    fn add(&mut self, xoff: c_int, yoff: c_int, sx: u_int, sy: u_int) -> usize {
        self.pair.add(xoff, yoff, sx, sy)
    }

    fn c(&mut self) -> *mut client {
        &raw mut *self.client
    }

    fn w(&mut self) -> *mut window {
        self.pair.w()
    }

    fn wp(&mut self, i: usize) -> *mut window_pane {
        self.pair.wp(i)
    }

    /// A redraw context over this client carrying only what the geometry
    /// helpers read: the client chain and the pane status line's position.
    fn ctx(&mut self, pane_status: c_int) -> Box<screen_redraw_ctx> {
        let mut ctx = Box::new(screen_redraw_ctx::default());
        ctx.c = self.c();
        ctx.pane_status = pane_status;
        ctx
    }
}

impl Drop for Viewed {
    fn drop(&mut self) {
        if !self.wl.is_null() {
            unlink(&mut self.session, self.wl);
            self.wl = null_mut();
        }
    }
}

/// Caller-owned storage behind a [`visible_ranges`], sized so that a split
/// never asks for more room.
struct Ranges {
    r: visible_ranges,
}

impl Ranges {
    fn new(filled: &[visible_range]) -> Ranges {
        let mut ranges = filled.to_vec();
        ranges.resize(4, visible_range { px: 0, nx: 0 });
        Ranges {
            r: visible_ranges {
                ranges,
                used: filled.len() as u_int,
            },
        }
    }
}

/// The border and cell constants name the twelve points of the redraw ring in
/// order, and the pane status and scrollbar choices sit beside them.
#[test]
fn the_constants_name_the_border_ring_in_order() {
    assert_eq!(SCREEN_REDRAW_OUTSIDE, 0);
    assert_eq!(SCREEN_REDRAW_INSIDE, 1);
    assert_eq!(SCREEN_REDRAW_BORDER_LEFT, 2);
    assert_eq!(SCREEN_REDRAW_BORDER_RIGHT, 3);
    assert_eq!(SCREEN_REDRAW_BORDER_TOP, 4);
    assert_eq!(SCREEN_REDRAW_BORDER_BOTTOM, 5);

    assert_eq!(CELL_INSIDE, 0);
    assert_eq!(CELL_TOPBOTTOM, 1);
    assert_eq!(CELL_LEFTRIGHT, 2);
    assert_eq!(CELL_TOPLEFT, 3);
    assert_eq!(CELL_TOPRIGHT, 4);
    assert_eq!(CELL_BOTTOMLEFT, 5);
    assert_eq!(CELL_BOTTOMRIGHT, 6);
    assert_eq!(CELL_TOPJOIN, 7);
    assert_eq!(CELL_BOTTOMJOIN, 8);
    assert_eq!(CELL_LEFTJOIN, 9);
    assert_eq!(CELL_RIGHTJOIN, 10);
    assert_eq!(CELL_JOIN, 11);
    assert_eq!(CELL_OUTSIDE, 12);
    assert_eq!(CELL_SCROLLBAR, 13);

    assert_eq!(PANE_LINES_SINGLE, 0);
    assert_eq!(PANE_LINES_DOUBLE, 1);
    assert_eq!(PANE_LINES_HEAVY, 2);
    assert_eq!(PANE_LINES_SIMPLE, 3);
    assert_eq!(PANE_LINES_NUMBER, 4);
    assert_eq!(PANE_LINES_SPACES, 5);

    assert_eq!(PANE_STATUS_OFF, 0);
    assert_eq!(PANE_STATUS_TOP, 1);
    assert_eq!(PANE_STATUS_BOTTOM, 2);

    assert_eq!(PANE_SCROLLBARS_OFF, 0);
    assert_eq!(PANE_SCROLLBARS_MODAL, 1);
    assert_eq!(PANE_SCROLLBARS_RIGHT, 0);
    assert_eq!(PANE_SCROLLBARS_LEFT, 1);
}

/// The three glyph tables are spelled to be indexed by the constants: the ACS
/// set for plain lines, the ASCII stand-ins for simple ones, and the arrow
/// markers for the four edges.
#[test]
fn the_glyph_tables_are_indexed_by_the_cells_they_draw() {
    for (i, ch) in b" xqlkmjwvtun~".iter().enumerate() {
        assert_eq!(CELL_BORDERS[i], *ch);
        assert_eq!(SIMPLE_BORDERS[i], b" |-+++++++++."[i]);
    }
    for (i, ch) in b"  +,.-".iter().enumerate() {
        assert_eq!(BORDER_MARKERS[i], *ch);
    }
    assert_eq!(CELL_BORDERS[CELL_OUTSIDE as usize], b'~');
    assert_eq!(CELL_BORDERS[CELL_INSIDE as usize], b' ');
    assert_eq!(CELL_BORDERS[CELL_JOIN as usize], b'n');
    assert_eq!(SIMPLE_BORDERS[CELL_TOPBOTTOM as usize], b'|');
    assert_eq!(SIMPLE_BORDERS[CELL_LEFTRIGHT as usize], b'-');
    assert_eq!(BORDER_MARKERS[SCREEN_REDRAW_BORDER_TOP as usize], b'.');
    assert_eq!(BORDER_MARKERS[SCREEN_REDRAW_BORDER_BOTTOM as usize], b'-');
}

/// Every individual redraw bit the drawing code looks for lands in
/// `CLIENT_ALLREDRAWFLAGS`, and the suspended and UTF-8 bits stay out of it.
#[test]
fn every_client_redraw_bit_lands_in_the_all_flag() {
    assert_eq!(CLIENT_REDRAWWINDOW, 0x8);
    assert_eq!(CLIENT_REDRAWSTATUS, 0x10);
    assert_eq!(CLIENT_SUSPENDED, 0x40);
    assert_eq!(CLIENT_REDRAWBORDERS, 0x400);
    assert_eq!(CLIENT_UTF8, 0x10000);
    assert_eq!(CLIENT_REDRAWSTATUSALWAYS, 0x1000000);
    assert_eq!(CLIENT_REDRAWOVERLAY, 0x2000000);
    assert_eq!(CLIENT_REDRAWPANES, 0x20000000);
    assert_eq!(CLIENT_REDRAWSCROLLBARS, 0x4000000000u64);
    assert_eq!(
        CLIENT_ALLREDRAWFLAGS,
        (CLIENT_REDRAWWINDOW
            | CLIENT_REDRAWSTATUS
            | CLIENT_REDRAWSTATUSALWAYS
            | CLIENT_REDRAWBORDERS
            | CLIENT_REDRAWOVERLAY
            | CLIENT_REDRAWPANES) as u64
            | CLIENT_REDRAWSCROLLBARS
    );
    assert_eq!(CLIENT_ALLREDRAWFLAGS & CLIENT_SUSPENDED as u64, 0);
    assert_eq!(CLIENT_ALLREDRAWFLAGS & CLIENT_UTF8 as u64, 0);
}

/// The isolate markers are the left-to-right and right-to-left directional
/// formatting characters, handed over NUL-terminated in four bytes each.
#[test]
fn the_isolate_markers_wrap_a_line_in_direction_codes() {
    for (i, byte) in [0xe2u8, 0x81, 0xa6].iter().enumerate() {
        assert_eq!(START_ISOLATE[i] as u8, *byte);
    }
    for (i, byte) in [0xe2u8, 0x81, 0xa9].iter().enumerate() {
        assert_eq!(END_ISOLATE[i] as u8, *byte);
    }
    assert_eq!(START_ISOLATE[3], 0);
    assert_eq!(END_ISOLATE[3], 0);
}

/// No ranges at all means nothing can hide a column, and a column is visible
/// only where some range of real width covers it; an entry of no width is
/// stepped over however close the column sits to it.
#[test]
fn a_column_is_visible_only_where_a_range_of_width_covers_it() {
    assert!(screen_redraw_is_visible(None, 0));

    let rs = Ranges::new(&[
        visible_range { px: 2, nx: 3 },
        visible_range { px: 9, nx: 0 },
    ]);
    assert!(!screen_redraw_is_visible(Some(&rs.r), 0));
    assert!(!screen_redraw_is_visible(Some(&rs.r), 1));
    assert!(screen_redraw_is_visible(Some(&rs.r), 2));
    assert!(screen_redraw_is_visible(Some(&rs.r), 4));
    assert!(!screen_redraw_is_visible(Some(&rs.r), 5));
    assert!(!screen_redraw_is_visible(Some(&rs.r), 8));
    assert!(!screen_redraw_is_visible(Some(&rs.r), 9));
}

/// A request covering nothing — a row above the screen, no width at all, or a
/// negative start running past the width — leaves the caller's own ranges
/// emptied rather than filled.
#[test]
fn a_request_that_covers_nothing_empties_the_caller_ranges() {
    let _guard = globals();
    let mut rs = Ranges::new(&[
        visible_range { px: 1, nx: 1 },
        visible_range { px: 5, nx: 2 },
    ]);
    unsafe {
        screen_redraw_clip_visible_ranges(null_mut(), 0, -1, 8, &mut rs.r);
        assert_eq!(rs.r.used, 0);

        rs.r.used = 2;
        screen_redraw_clip_visible_ranges(null_mut(), 0, 0, 0, &mut rs.r);
        assert_eq!(rs.r.used, 0);

        rs.r.used = 2;
        screen_redraw_clip_visible_ranges(null_mut(), -4, 0, 4, &mut rs.r);
        assert_eq!(rs.r.used, 0);
    }
}

/// Whatever ranges the caller hands in come back exactly as they went in when
/// no pane stands over them — even a negative start that fits inside the width
/// leaves them alone — and a row past the bottom of the window empties them.
#[test]
fn ranges_survive_unchanged_when_nothing_stands_over_them() {
    let _guard = globals();
    let mut p = Pair::new(30, 5);
    p.add(2, 1, 6, 3);
    let mut rs = Ranges::new(&[
        visible_range { px: 3, nx: 5 },
        visible_range { px: 40, nx: 1 },
    ]);
    unsafe {
        screen_redraw_clip_visible_ranges(null_mut(), -1, 2, 6, &mut rs.r);
        assert_eq!(rs.r.used, 2);
        assert_eq!((rs.r.ranges[0].px, rs.r.ranges[0].nx), (3, 5));
        assert_eq!((rs.r.ranges[1].px, rs.r.ranges[1].nx), (40, 1));

        screen_redraw_clip_visible_ranges(p.wp(0), 2, 5, 6, &mut rs.r);
        assert_eq!(rs.r.used, 0);
    }
}

/// A single pane never cuts the range asked for: it is found by the backwards
/// walk, marks itself as found, and the answer comes back as handed in.
#[test]
fn a_lone_pane_leaves_the_whole_range_alone() {
    let _guard = globals();
    let mut p = Pair::new(30, 5);
    p.add(4, 1, 8, 3);
    let mut rs = Ranges::new(&[visible_range { px: 1, nx: 20 }]);
    unsafe {
        screen_redraw_clip_visible_ranges(p.wp(0), 1, 2, 20, &mut rs.r);
        assert_eq!(rs.r.used, 1);
        assert_eq!((rs.r.ranges[0].px, rs.r.ranges[0].nx), (1, 20));
    }
}

/// A pane earlier in the z order sits *in front* of a later one, and the
/// backwards walk only weighs panes once it has passed the pane asked about.
/// A row crossing such a pane comes back cut short where that pane's body and
/// the column its left border occupies begin.
#[test]
fn a_pane_in_front_cuts_the_range_short_at_its_left_edge() {
    let _guard = globals();
    let mut p = Pair::new(30, 5);
    p.add(10, 0, 4, 4);
    let base = p.add(2, 1, 6, 3);
    let mut rs = Ranges::new(&[visible_range { px: 0, nx: 12 }]);
    unsafe {
        screen_redraw_clip_visible_ranges(p.wp(base), 0, 2, 12, &mut rs.r);
        assert_eq!(rs.r.used, 1);
        assert_eq!((rs.r.ranges[0].px, rs.r.ranges[0].nx), (0, 9));
    }
}

/// When the pane in front stops partway across the range the row is split in
/// two around its right border column, keeping the front piece and opening a
/// second range for what lies beyond it.
#[test]
fn a_pane_in_front_splits_the_range_in_two() {
    let _guard = globals();
    let mut p = Pair::new(30, 5);
    p.add(10, 0, 6, 4);
    let base = p.add(2, 1, 6, 3);
    let mut rs = Ranges::new(&[visible_range { px: 0, nx: 20 }]);
    unsafe {
        screen_redraw_clip_visible_ranges(p.wp(base), 0, 2, 20, &mut rs.r);
        assert_eq!(rs.r.used, 2);
        assert_eq!((rs.r.ranges[0].px, rs.r.ranges[0].nx), (0, 9));
        assert_eq!((rs.r.ranges[1].px, rs.r.ranges[1].nx), (17, 3));
    }
}

/// Outside cells take the window's fill character verbatim, and the dresser
/// returns before touching anything else about the cell.
#[test]
fn a_fill_character_takes_over_outside_cells_untouched() {
    let _guard = globals();
    let mut p = Pair::new(8, 4);
    p.add(0, 0, 8, 4);
    let fill = Box::new(glyph(b"#"));
    unsafe {
        (*p.w()).fill_character = Some(fill);
        let mut gc = undressed(GRID_ATTR_CHARSET as u_short);
        screen_redraw_border_set(p.w(), p.wp(0), PANE_LINES_SINGLE, CELL_OUTSIDE, &raw mut gc);
        expect_dressed_as(&gc, glyph(b"#"));
        assert_eq!(gc.attr as c_int & GRID_ATTR_CHARSET, GRID_ATTR_CHARSET);
    }
}

/// Plain lines dress every junction from the ACS table and mark the cell as
/// wanting the character set; simple lines dress it from the ASCII table, and
/// both leave any other attribute the cell was carrying alone.
#[test]
fn plain_and_simple_lines_dress_cells_from_their_own_tables() {
    let _guard = globals();
    let mut p = Pair::new(8, 4);
    p.add(0, 0, 8, 4);
    let kinds = [
        CELL_TOPBOTTOM,
        CELL_LEFTRIGHT,
        CELL_TOPLEFT,
        CELL_TOPRIGHT,
        CELL_BOTTOMLEFT,
        CELL_BOTTOMRIGHT,
        CELL_TOPJOIN,
        CELL_BOTTOMJOIN,
        CELL_LEFTJOIN,
        CELL_RIGHTJOIN,
        CELL_JOIN,
        CELL_OUTSIDE,
    ];
    for t in kinds {
        let mut plain = undressed(GRID_ATTR_REVERSE as u_short);
        unsafe { screen_redraw_border_set(p.w(), p.wp(0), PANE_LINES_SINGLE, t, &raw mut plain) };
        assert_eq!(plain.attr as c_int & GRID_ATTR_CHARSET, GRID_ATTR_CHARSET);
        assert_eq!(plain.attr as c_int & GRID_ATTR_REVERSE, GRID_ATTR_REVERSE);
        assert_eq!(plain.data.data[0], CELL_BORDERS[t as usize]);
        assert_eq!((plain.data.size, plain.data.width), (1, 1));

        let mut simple = undressed(GRID_ATTR_CHARSET as u_short);
        unsafe { screen_redraw_border_set(p.w(), p.wp(0), PANE_LINES_SIMPLE, t, &raw mut simple) };
        assert_eq!(simple.attr as c_int & GRID_ATTR_CHARSET, 0);
        assert_eq!(simple.data.data[0], SIMPLE_BORDERS[t as usize]);
    }
}

/// Double and heavy lines copy their whole UTF-8 characters straight out of
/// the ACS border tables, character width included, and spaces blank the cell
/// altogether; each turns the character-set attribute back off.
#[test]
fn double_heavy_and_space_lines_take_their_characters_whole() {
    let _guard = globals();
    let mut p = Pair::new(8, 4);
    p.add(0, 0, 8, 4);
    let kinds = [
        CELL_TOPBOTTOM,
        CELL_LEFTRIGHT,
        CELL_TOPLEFT,
        CELL_TOPRIGHT,
        CELL_BOTTOMLEFT,
        CELL_BOTTOMRIGHT,
        CELL_TOPJOIN,
        CELL_BOTTOMJOIN,
        CELL_LEFTJOIN,
        CELL_RIGHTJOIN,
        CELL_JOIN,
        CELL_OUTSIDE,
    ];
    for t in kinds {
        let mut doubled = undressed(GRID_ATTR_CHARSET as u_short);
        unsafe { screen_redraw_border_set(p.w(), p.wp(0), PANE_LINES_DOUBLE, t, &raw mut doubled) };
        assert_eq!(doubled.attr as c_int & GRID_ATTR_CHARSET, 0);
        let want = *tty_acs_double_borders(t);
        expect_dressed_as(&doubled, want);

        let mut heavy = undressed(GRID_ATTR_CHARSET as u_short);
        unsafe { screen_redraw_border_set(p.w(), p.wp(0), PANE_LINES_HEAVY, t, &raw mut heavy) };
        assert_eq!(heavy.attr as c_int & GRID_ATTR_CHARSET, 0);
        let want = *tty_acs_heavy_borders(t);
        expect_dressed_as(&heavy, want);

        let mut spaced = undressed(GRID_ATTR_CHARSET as u_short);
        unsafe { screen_redraw_border_set(p.w(), p.wp(0), PANE_LINES_SPACES, t, &raw mut spaced) };
        assert_eq!(spaced.attr as c_int & GRID_ATTR_CHARSET, 0);
        assert_eq!(spaced.data.data[0], b' ');
        assert_eq!((spaced.data.size, spaced.data.width), (1, 1));
    }
}

/// Numbered lines show a pane's position in the window counting from
/// `pane-base-index`, and a star when there is no pane to name.
#[test]
fn numbered_lines_show_the_pane_index_or_a_star() {
    let _guard = globals();
    let mut p = Pair::new(16, 4);
    p.add(0, 0, 8, 4);
    p.add(8, 0, 8, 4);
    unsafe {
        let mut second = undressed(GRID_ATTR_CHARSET as u_short);
        screen_redraw_border_set(
            p.w(),
            p.wp(1),
            PANE_LINES_NUMBER,
            CELL_TOPLEFT,
            &raw mut second,
        );
        assert_eq!(second.data.data[0], b'1');
        assert_eq!(second.attr as c_int & GRID_ATTR_CHARSET, 0);

        let mut star = undressed(0);
        screen_redraw_border_set(
            p.w(),
            null_mut(),
            PANE_LINES_NUMBER,
            CELL_TOPLEFT,
            &raw mut star,
        );
        assert_eq!(star.data.data[0], b'*');

        options_set_number((*p.w()).options_ptr(), c"pane-base-index".as_ptr(), 9);
        let mut rebased = undressed(0);
        screen_redraw_border_set(
            p.w(),
            p.wp(0),
            PANE_LINES_NUMBER,
            CELL_TOPLEFT,
            &raw mut rebased,
        );
        assert_eq!(rebased.data.data[0], b'9');
    }
}

/// The two-pane walk counts only panes hanging off a shared parent cell: two
/// such panes answer with the split direction, a third breaks it, a floating
/// partner or a missing parent leaves it at one, and panes without cells are
/// not counted at all.
#[test]
fn two_panes_answers_only_for_exactly_two_sharing_a_parent() {
    let _guard = globals();
    let mut bare = Pair::new(12, 6);
    bare.add(0, 0, 6, 6);
    bare.add(6, 0, 6, 6);
    unsafe {
        let mut kind: layout_type = LAYOUT_TOPBOTTOM;
        assert_eq!(screen_redraw_two_panes(bare.w(), &raw mut kind), 0);
    }

    let mut split = Pair::new(12, 6);
    split.add(0, 0, 6, 6);
    split.add(6, 0, 6, 6);
    let mut parent = Box::new(layout_cell::default());
    let mut left = Box::new(layout_cell::default());
    let mut right = Box::new(layout_cell::default());
    unsafe {
        left.parent = &raw mut *parent;
        right.parent = &raw mut *parent;
        (*split.wp(0)).layout_cell = &raw mut *left;
        (*split.wp(1)).layout_cell = &raw mut *right;

        parent.type_0 = LAYOUT_LEFTRIGHT;
        let mut kind: layout_type = LAYOUT_TOPBOTTOM;
        assert_eq!(screen_redraw_two_panes(split.w(), &raw mut kind), 1);
        assert_eq!(kind, LAYOUT_LEFTRIGHT);

        parent.type_0 = LAYOUT_TOPBOTTOM;
        assert_eq!(screen_redraw_two_panes(split.w(), &raw mut kind), 1);
        assert_eq!(kind, LAYOUT_TOPBOTTOM);
    }

    let mut triple = Pair::new(18, 6);
    triple.add(0, 0, 6, 6);
    triple.add(6, 0, 6, 6);
    triple.add(12, 0, 6, 6);
    let mut parent3 = Box::new(layout_cell::default());
    let mut cells3 = [
        Box::new(layout_cell::default()),
        Box::new(layout_cell::default()),
        Box::new(layout_cell::default()),
    ];
    unsafe {
        for (i, cell) in cells3.iter_mut().enumerate() {
            cell.parent = &raw mut *parent3;
            (*triple.wp(i)).layout_cell = &raw mut **cell;
        }
        let mut kind: layout_type = LAYOUT_TOPBOTTOM;
        assert_eq!(screen_redraw_two_panes(triple.w(), &raw mut kind), 0);
    }

    let mut floaty = Pair::new(12, 6);
    floaty.add(0, 0, 6, 6);
    floaty.add(6, 0, 6, 6);
    let mut parent4 = Box::new(layout_cell::default());
    let mut cell4 = Box::new(layout_cell::default());
    let mut hover4 = Box::new(layout_cell::default());
    unsafe {
        cell4.parent = &raw mut *parent4;
        hover4.parent = &raw mut *parent4;
        hover4.flags |= LAYOUT_CELL_FLOATING;
        (*floaty.wp(0)).layout_cell = &raw mut *cell4;
        (*floaty.wp(1)).layout_cell = &raw mut *hover4;
        let mut kind: layout_type = LAYOUT_LEFTRIGHT;
        assert_eq!(screen_redraw_two_panes(floaty.w(), &raw mut kind), 0);
        assert_eq!(kind, LAYOUT_LEFTRIGHT);
    }

    let mut orphan = Pair::new(12, 6);
    orphan.add(0, 0, 6, 6);
    orphan.add(6, 0, 6, 6);
    let mut left5 = Box::new(layout_cell::default());
    let mut right5 = Box::new(layout_cell::default());
    unsafe {
        (*orphan.wp(0)).layout_cell = &raw mut *left5;
        (*orphan.wp(1)).layout_cell = &raw mut *right5;
        let mut kind: layout_type = LAYOUT_LEFTRIGHT;
        assert_eq!(screen_redraw_two_panes(orphan.w(), &raw mut kind), 0);
    }

    let mut lone = Pair::new(6, 6);
    lone.add(0, 0, 6, 6);
    let mut parent6 = Box::new(layout_cell::default());
    let mut cell6 = Box::new(layout_cell::default());
    unsafe {
        cell6.parent = &raw mut *parent6;
        (*lone.wp(0)).layout_cell = &raw mut *cell6;
        let mut kind: layout_type = LAYOUT_LEFTRIGHT;
        assert_eq!(screen_redraw_two_panes(lone.w(), &raw mut kind), 0);
    }
}

/// Whether a cell counts as sitting on the marked pane's border is answered
/// straight from the edge classifier: nowhere at all for a missing pane, no on
/// the inside or clear outside, and yes on a true edge.
#[test]
fn check_is_answers_yes_only_on_a_true_edge() {
    let _guard = globals();
    let mut p = Pair::new(12, 8);
    p.add(4, 2, 6, 3);
    let mut ctx = Box::new(screen_redraw_ctx::default());
    unsafe {
        assert_eq!(
            screen_redraw_check_is(&mut ctx, 5, 3, null_mut::<window_pane>()) as c_int,
            0
        );
        assert_eq!(screen_redraw_check_is(&mut ctx, 5, 3, p.wp(0)) as c_int, 0);
        assert_eq!(screen_redraw_check_is(&mut ctx, 11, 6, p.wp(0)) as c_int, 0);
        assert_eq!(screen_redraw_check_is(&mut ctx, 10, 4, p.wp(0)) as c_int, 1);
        assert_eq!(screen_redraw_check_is(&mut ctx, 5, 1, p.wp(0)) as c_int, 1);
    }
}

/// For a pane placed away from the window's own edges the classifier puts the
/// left border one column before it, the right one on its last inclusive
/// column, and its top and bottom rows on the rows just outside it; the status
/// line position suppresses whichever horizontal edge it is drawn over.
#[test]
fn a_placed_pane_wears_its_borders_around_itself() {
    let _guard = globals();
    let mut p = Pair::new(14, 9);
    let wp = p.add(4, 2, 6, 3);
    unsafe {
        let mut ctx = Box::new(screen_redraw_ctx::default());
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, p.wp(wp), 5, 3),
            SCREEN_REDRAW_INSIDE
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, p.wp(wp), 3, 4),
            SCREEN_REDRAW_BORDER_LEFT
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, p.wp(wp), 10, 4),
            SCREEN_REDRAW_BORDER_RIGHT
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, p.wp(wp), 5, 1),
            SCREEN_REDRAW_BORDER_TOP
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, p.wp(wp), 5, 5),
            SCREEN_REDRAW_BORDER_BOTTOM
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, p.wp(wp), 11, 3),
            SCREEN_REDRAW_OUTSIDE
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, p.wp(wp), 3, 6),
            SCREEN_REDRAW_OUTSIDE
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, p.wp(wp), 0, 0),
            SCREEN_REDRAW_OUTSIDE
        );

        let mut topctx = Box::new(screen_redraw_ctx::default());
        topctx.pane_status = PANE_STATUS_TOP;
        assert_eq!(
            screen_redraw_pane_border(&mut topctx, p.wp(wp), 5, 1),
            SCREEN_REDRAW_BORDER_TOP
        );
        assert_eq!(
            screen_redraw_pane_border(&mut topctx, p.wp(wp), 5, 5),
            SCREEN_REDRAW_OUTSIDE
        );

        let mut botctx = Box::new(screen_redraw_ctx::default());
        botctx.pane_status = PANE_STATUS_BOTTOM;
        assert_eq!(
            screen_redraw_pane_border(&mut botctx, p.wp(wp), 5, 1),
            SCREEN_REDRAW_OUTSIDE
        );
        assert_eq!(
            screen_redraw_pane_border(&mut botctx, p.wp(wp), 5, 5),
            SCREEN_REDRAW_BORDER_BOTTOM
        );
    }
}

/// A floating pane wears its border one column out from its body on every
/// side — wider than a tiled pane's, since it hangs over whatever is under
/// it — and the inside of its body still wins over the ring around it.
#[test]
fn a_floating_pane_wears_its_borders_one_column_out() {
    let _guard = globals();
    let mut p = Pair::new(10, 6);
    let wp = p.add(2, 1, 4, 2);
    let mut fcell = Box::new(layout_cell::default());
    fcell.flags |= LAYOUT_CELL_FLOATING;
    unsafe { (*p.wp(wp)).layout_cell = &raw mut *fcell };
    unsafe {
        let mut ctx = Box::new(screen_redraw_ctx::default());
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, p.wp(wp), 3, 1),
            SCREEN_REDRAW_INSIDE
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, p.wp(wp), 1, 2),
            SCREEN_REDRAW_BORDER_LEFT
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, p.wp(wp), 6, 2),
            SCREEN_REDRAW_BORDER_RIGHT
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, p.wp(wp), 3, 0),
            SCREEN_REDRAW_BORDER_TOP
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, p.wp(wp), 3, 3),
            SCREEN_REDRAW_BORDER_BOTTOM
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, p.wp(wp), 7, 2),
            SCREEN_REDRAW_OUTSIDE
        );
        assert_eq!(
            screen_redraw_pane_border(&mut ctx, p.wp(wp), 0, 0),
            SCREEN_REDRAW_OUTSIDE
        );
    }
}

/// The neighbour probe answers no for anything past the window's right or
/// bottom edge, yes on the last column and row, and otherwise hands the
/// question to the panes in z order: the first pane that can answer wins, a
/// border column of either pane answers yes, and an inside answer from the
/// second pane counts even after the first turned the same point down as
/// outside.
#[test]
fn the_neighbour_probe_walks_the_z_order_until_someone_answers() {
    let _guard = globals();
    let mut full = Viewed::new(10, 5);
    let mut ctx = full.ctx(PANE_STATUS_OFF);
    unsafe {
        assert_eq!(screen_redraw_cell_border(&mut ctx, full.wp(0), 5, 3), 0);
        assert_eq!(screen_redraw_cell_border(&mut ctx, full.wp(0), 10, 3), 1);
        assert_eq!(screen_redraw_cell_border(&mut ctx, full.wp(0), 11, 3), 0);
    }

    let mut halves = Viewed::empty(10, 5);
    halves.add(0, 0, 4, 5);
    halves.add(6, 0, 4, 5);
    let mut ctx = halves.ctx(PANE_STATUS_OFF);
    unsafe {
        assert_eq!(screen_redraw_cell_border(&mut ctx, halves.wp(0), 3, 3), 0);
        assert_eq!(screen_redraw_cell_border(&mut ctx, halves.wp(0), 4, 3), 1);
        assert_eq!(screen_redraw_cell_border(&mut ctx, halves.wp(0), 5, 3), 1);
        assert_eq!(screen_redraw_cell_border(&mut ctx, halves.wp(0), 7, 3), 0);
    }

    let mut hovering = Viewed::empty(10, 6);
    hovering.add(0, 0, 10, 6);
    let floater = hovering.add(2, 1, 4, 2);
    let mut fcell = Box::new(layout_cell::default());
    fcell.flags |= LAYOUT_CELL_FLOATING;
    unsafe { (*hovering.wp(floater)).layout_cell = &raw mut *fcell };
    let mut ctx = hovering.ctx(PANE_STATUS_OFF);
    unsafe {
        assert_eq!(
            screen_redraw_cell_border(&mut ctx, hovering.wp(floater), 3, 1),
            0
        );
        assert_eq!(
            screen_redraw_cell_border(&mut ctx, hovering.wp(floater), 1, 1),
            1
        );
        assert_eq!(
            screen_redraw_cell_border(&mut ctx, hovering.wp(floater), 0, 0),
            0
        );
    }
}

/// The junction table reads the neighbour answers as bits: a cell with no
/// bordering neighbours classifies as outside even mid-pane, a column of
/// border on both sides reads as a vertical line, the window's bottom row
/// likewise reads as vertical because its neighbours along the row answer yes,
/// the top right corner reads as a top right corner, and the bottom right
/// corner joins a left border with a line above it.
#[test]
fn the_junction_table_turns_neighbour_bits_into_cells() {
    let _guard = globals();
    let mut v = Viewed::new(10, 5);
    let mut ctx = v.ctx(PANE_STATUS_OFF);
    unsafe {
        assert_eq!(
            screen_redraw_type_of_cell(&mut ctx, v.wp(0), 4, 2),
            CELL_OUTSIDE
        );
        assert_eq!(
            screen_redraw_type_of_cell(&mut ctx, v.wp(0), 10, 0),
            CELL_TOPBOTTOM
        );
        assert_eq!(
            screen_redraw_type_of_cell(&mut ctx, v.wp(0), 10, 3),
            CELL_TOPBOTTOM
        );
        assert_eq!(
            screen_redraw_type_of_cell(&mut ctx, v.wp(0), 5, 5),
            CELL_LEFTRIGHT
        );
        assert_eq!(
            screen_redraw_type_of_cell(&mut ctx, v.wp(0), 10, 5),
            CELL_BOTTOMRIGHT
        );
    }
}

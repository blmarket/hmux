use super::*;
use crate::grid::hyperlinks_put;
use crate::screen::screen_free;
use crate::tests::test_fixtures::globals;
use ::core::ffi::{CStr, c_int};
use ::core::ptr::null_mut;

/// A grid that frees itself at the end of the test.
struct Grid(Box<grid>);

impl Grid {
    fn new(sx: u_int, sy: u_int, hlimit: u_int) -> Grid {
        Grid(grid_create(sx, sy, hlimit))
    }

    fn ptr(&self) -> *mut grid {
        self.0.as_ref() as *const grid as *mut grid
    }

    fn line(&self, py: u_int) -> grid_line {
        unsafe { (*self.ptr()).linedata[py as usize].clone() }
    }

    fn entry(&self, px: u_int, py: u_int) -> grid_cell_entry {
        self.line(py).celldata()[px as usize]
    }

    fn cell(&self, px: u_int, py: u_int) -> grid_cell {
        let mut gc = grid_cell::default();
        unsafe { gc = grid_get_cell(&*self.ptr(), px, py) };
        gc
    }

    /// Writes `s` one ASCII cell per byte from (px, py).
    fn write(&self, px: u_int, py: u_int, s: &str) {
        for (i, ch) in s.bytes().enumerate() {
            let gc = ascii(ch);
            unsafe { grid_set_cell(&mut *self.ptr(), px + i as u_int, py, &gc) };
        }
    }

    /// The text of a line the way `grid_string_cells` renders it.
    fn text(&self, py: u_int) -> String {
        self.render(py, 0, None, null_mut())
    }

    fn render(
        &self,
        py: u_int,
        flags: c_int,
        lastgc: Option<&mut grid_cell>,
        sc: *mut screen,
    ) -> String {
        unsafe {
            let p = grid_string_cells(&*self.ptr(), 0, py, 1000, lastgc, flags, sc);

            p.to_string_lossy().into_owned()
        }
    }

    /// Every line as text, with the flag bits that reflow moves around.
    fn dump(&self) -> Vec<(String, c_int)> {
        unsafe {
            (0..(*self.ptr()).hsize + (*self.ptr()).sy)
                .map(|py| (self.text(py), self.line(py).flags))
                .collect()
        }
    }
}

impl ::core::ops::Deref for Grid {
    type Target = grid;

    fn deref(&self) -> &grid {
        &self.0
    }
}

/// The default cell holding one ASCII byte.
fn ascii(ch: u8) -> grid_cell {
    let mut gc = unsafe { grid_default_cell };
    unsafe { utf8_set(&mut gc.data, ch) };
    gc
}

/// The default cell holding one character, which may be more than one
/// byte wide.
fn wide(s: &str, width: u_char) -> grid_cell {
    let mut gc = unsafe { grid_default_cell };
    gc.data.data[..s.len()].copy_from_slice(s.as_bytes());
    gc.data.have = s.len() as u_char;
    gc.data.size = gc.data.have;
    gc.data.width = width;
    gc
}

fn text_of(gc: &grid_cell) -> String {
    String::from_utf8_lossy(&gc.data.data[..gc.data.size as usize]).into_owned()
}

#[test]
fn a_new_grid_has_no_history_and_empty_lines() {
    let _guard = globals();
    let g = Grid::new(10, 3, 0);
    assert_eq!(g.sx, 10);
    assert_eq!(g.sy, 3);
    assert_eq!(g.hsize, 0);
    assert_eq!(g.hscrolled, 0);
    assert_eq!(g.hlimit, 0);
    assert_eq!(g.flags, 0);
    assert_eq!(g.line(0).cellsize() as usize, 0);
    assert!(g.line(0).celldata().is_empty());

    let h = Grid::new(10, 3, 100);
    assert_eq!(h.flags, GRID_HISTORY);
    assert_eq!(h.hlimit, 100);

    let empty = Grid::new(10, 0, 0);
    assert!(empty.linedata.is_empty());
}

#[test]
fn an_unwritten_cell_reads_as_the_default_cell() {
    let _guard = globals();
    let g = Grid::new(10, 3, 0);
    let gc = g.cell(0, 0);
    assert_eq!(text_of(&gc), " ");
    assert_eq!(gc.fg, 8);
    assert_eq!(gc.bg, 8);
    assert_eq!(gc.us, 8);
    assert_eq!(gc.flags, 0);
    assert_eq!(gc.attr, 0);
    assert_eq!(gc.link, 0);
}

#[test]
fn a_cell_beyond_the_end_of_the_grid_reads_as_the_default_cell() {
    let _guard = globals();
    let g = Grid::new(10, 3, 0);
    g.write(0, 0, "abc");
    assert_eq!(text_of(&g.cell(9, 3)), " ");
    assert_eq!(text_of(&g.cell(9, 300)), " ");
    assert_eq!(g.line(0).cellsize() as usize, 5);
    assert_eq!(text_of(&g.cell(7, 0)), " ");
}

#[test]
fn a_plain_cell_is_packed_into_the_entry() {
    let _guard = globals();
    let g = Grid::new(10, 3, 0);
    let mut gc = ascii(b'x');
    gc.fg = 4;
    gc.bg = 2;
    gc.attr = GRID_ATTR_BRIGHT as u_short;
    gc.flags = GRID_FLAG_CLEARED as u_char;
    unsafe { grid_set_cell(&mut *g.ptr(), 1, 0, &gc) };

    let gce = g.entry(1, 0);
    assert_eq!(gce.flags, 0);
    unsafe {
        assert_eq!(gce.c2rust_unnamed.data.fg, 4);
        assert_eq!(gce.c2rust_unnamed.data.bg, 2);
        assert_eq!(gce.c2rust_unnamed.data.attr, GRID_ATTR_BRIGHT as u_char);
        assert_eq!(gce.c2rust_unnamed.data.data, b'x');
    }

    let read = g.cell(1, 0);
    assert_eq!(text_of(&read), "x");
    assert_eq!(read.fg, 4);
    assert_eq!(read.bg, 2);
    assert_eq!(read.us, 8);
    assert_eq!(read.attr, GRID_ATTR_BRIGHT as u_short);
    assert_eq!(read.flags, 0);
    assert_eq!(g.line(0).cellused, 2);
}

#[test]
fn the_256_colour_flags_live_in_the_entry_flags() {
    let _guard = globals();
    let g = Grid::new(10, 3, 0);
    let mut gc = ascii(b'y');
    gc.fg = 200 | COLOUR_FLAG_256;
    gc.bg = 100 | COLOUR_FLAG_256;
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };

    let gce = g.entry(0, 0);
    assert_eq!(gce.flags as c_int, GRID_FLAG_FG256 | GRID_FLAG_BG256);
    unsafe {
        assert_eq!(gce.c2rust_unnamed.data.fg, 200);
        assert_eq!(gce.c2rust_unnamed.data.bg, 100);
    }
    let read = g.cell(0, 0);
    assert_eq!(read.fg, 200 | COLOUR_FLAG_256);
    assert_eq!(read.bg, 100 | COLOUR_FLAG_256);
}

#[test]
fn each_reason_for_an_extended_cell() {
    let _guard = globals();
    let plain = ascii(b'a');
    let mut extended = |change: &dyn Fn(&mut grid_cell)| {
        let g = Grid::new(10, 1, 0);
        let mut gc = ascii(b'a');
        change(&mut gc);
        unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };

        g.entry(0, 0).flags as c_int & GRID_FLAG_EXTENDED != 0
    };

    assert!(!extended(&|_| {}));
    assert!(extended(&|gc| gc.attr = GRID_ATTR_UNDERSCORE_2 as u_short));
    assert!(extended(&|gc| gc.data.size = 2));
    assert!(extended(&|gc| gc.data.width = 2));
    assert!(extended(&|gc| gc.fg = 0x111111 | COLOUR_FLAG_RGB));
    assert!(extended(&|gc| gc.bg = 0x111111 | COLOUR_FLAG_RGB));
    assert!(extended(&|gc| gc.us = 4));
    assert!(extended(&|gc| gc.link = 1));
    assert!(extended(&|gc| gc.flags = GRID_FLAG_TAB as u_char));

    // An entry that is already extended stays extended.
    let g = Grid::new(10, 1, 0);
    let mut gc = ascii(b'a');
    gc.us = 4;
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &plain) };
    assert!(g.entry(0, 0).flags as c_int & GRID_FLAG_EXTENDED != 0);
    assert_eq!(g.line(0).extdsize() as usize, 1);
    assert_eq!(g.cell(0, 0).us, 8);
}

#[test]
fn an_extended_cell_keeps_everything_the_packed_entry_cannot() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    let mut gc = wide("\u{4e2d}", 2);
    gc.fg = 0x102030 | COLOUR_FLAG_RGB;
    gc.bg = 0x405060 | COLOUR_FLAG_RGB;
    gc.us = 3 | COLOUR_FLAG_256;
    gc.attr = GRID_ATTR_UNDERSCORE_3 as u_short;
    gc.link = 7;
    gc.flags = GRID_FLAG_CLEARED as u_char;
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };

    assert_eq!(g.line(0).flags, GRID_LINE_EXTENDED | GRID_LINE_HYPERLINK);
    let read = g.cell(0, 0);
    assert_eq!(text_of(&read), "\u{4e2d}");
    assert_eq!(read.data.width, 2);
    assert_eq!(read.fg, 0x102030 | COLOUR_FLAG_RGB);
    assert_eq!(read.bg, 0x405060 | COLOUR_FLAG_RGB);
    assert_eq!(read.us, 3 | COLOUR_FLAG_256);
    assert_eq!(read.attr, GRID_ATTR_UNDERSCORE_3 as u_short);
    assert_eq!(read.link, 7);
    assert_eq!(read.flags, 0, "the cleared flag is not stored");
}

#[test]
fn a_tab_cell_stores_its_width_and_comes_back_as_spaces() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    let mut gc = ascii(b'\t');
    unsafe { grid_set_tab(&mut gc, 4) };
    assert_eq!(gc.flags as c_int, GRID_FLAG_TAB);
    assert_eq!(gc.data.width, 4);
    assert_eq!(gc.data.size, 4);
    assert_eq!(gc.data.have, 4);
    assert_eq!(text_of(&gc), "    ");

    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };
    let read = g.cell(0, 0);
    assert_eq!(read.flags as c_int, GRID_FLAG_TAB);
    assert_eq!(read.data.width, 4);
    assert_eq!(text_of(&read), "    ");
}

#[test]
fn setting_a_tab_clears_the_padding_flag() {
    let mut gc = ascii(b' ');
    gc.flags = GRID_FLAG_PADDING as u_char;
    gc.data.data[5] = b'z';
    unsafe { grid_set_tab(&mut gc, 2) };
    assert_eq!(gc.flags as c_int, GRID_FLAG_TAB);
    assert_eq!(gc.data.data[5], 0, "the old data is wiped");
}

#[test]
fn a_padding_cell_is_stored_packed_and_reads_back_one_cell_wide() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    unsafe { grid_set_padding(&mut *g.ptr(), 1, 0) };
    assert_eq!(
        g.entry(1, 0).flags as c_int,
        GRID_FLAG_PADDING,
        "nothing in the padding cell asks for an extended entry"
    );
    let read = g.cell(1, 0);
    assert_eq!(read.flags as c_int, GRID_FLAG_PADDING);
    assert_eq!(text_of(&read), "!");
    assert_eq!(read.data.width, 1, "the stored zero width does not survive");
    assert_eq!(read.data.size, 1);
}

#[test]
fn an_extended_entry_pointing_outside_the_extended_data_reads_as_default() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    let mut gc = ascii(b'a');
    gc.us = 4;
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };
    unsafe {
        let gl = &mut (*g.ptr()).linedata[0];
        gl.celldata_mut()[0].c2rust_unnamed.offset = 99;
    }
    let read = g.cell(0, 0);
    assert_eq!(text_of(&read), " ");
    assert_eq!(read.fg, 8);
}

#[test]
fn cells_look_equal_when_their_style_matches() {
    let one = ascii(b'a');
    let look = |change: &dyn Fn(&mut grid_cell)| {
        let mut two = ascii(b'a');
        change(&mut two);
        unsafe { grid_cells_look_equal(&one, &two) }
    };
    assert_eq!(look(&|_| {}), 1);
    assert_eq!(look(&|gc| gc.data.data[0] = b'b'), 1, "text is not style");
    assert_eq!(
        look(&|gc| gc.flags = GRID_FLAG_CLEARED as u_char),
        1,
        "the cleared flag is not style"
    );
    assert_eq!(look(&|gc| gc.fg = 1), 0);
    assert_eq!(look(&|gc| gc.bg = 1), 0);
    assert_eq!(look(&|gc| gc.attr = 1), 0);
    assert_eq!(look(&|gc| gc.flags = GRID_FLAG_PADDING as u_char), 0);
    assert_eq!(look(&|gc| gc.link = 1), 0);
}

#[test]
fn cells_are_equal_when_their_style_and_text_match() {
    let one = ascii(b'a');
    let equal = |change: &dyn Fn(&mut grid_cell)| {
        let mut two = ascii(b'a');
        change(&mut two);
        unsafe { grid_cells_equal(&one, &two) }
    };
    assert_eq!(equal(&|_| {}), 1);
    assert_eq!(equal(&|gc| gc.fg = 1), 0);
    assert_eq!(equal(&|gc| gc.data.width = 2), 0);
    assert_eq!(equal(&|gc| gc.data.size = 2), 0);
    assert_eq!(equal(&|gc| gc.data.data[0] = b'b'), 0);
}

#[test]
fn a_line_grows_in_the_steps_the_expansion_rule_names() {
    let _guard = globals();
    let g = Grid::new(80, 1, 0);
    g.write(0, 0, "a");
    assert_eq!(g.line(0).cellsize() as usize, 20, "a quarter of the width");
    g.write(25, 0, "a");
    assert_eq!(g.line(0).cellsize() as usize, 40, "half the width");
    g.write(50, 0, "a");
    assert_eq!(g.line(0).cellsize() as usize, 80, "the whole width");
    g.write(100, 0, "a");
    assert_eq!(
        g.line(0).cellsize() as usize,
        101,
        "past the width, exactly as asked"
    );
    g.write(0, 0, "b");
    assert_eq!(g.line(0).cellsize() as usize, 101, "no shrinking");
    assert_eq!(g.line(0).cellused, 101);
}

#[test]
fn an_emptied_line_keeps_no_cells_unless_the_background_is_set() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    g.write(0, 0, "abc");
    unsafe { grid_empty_line(&mut *g.ptr(), 0, 8) };
    assert_eq!(g.line(0).cellsize() as usize, 0);
    assert_eq!(g.line(0).cellused, 0);

    unsafe { grid_empty_line(&mut *g.ptr(), 1, 9) };
    assert_eq!(
        g.line(1).cellsize() as usize,
        0,
        "9 is also a default background"
    );

    unsafe { grid_empty_line(&mut *g.ptr(), 1, 2) };
    assert_eq!(g.line(1).cellsize() as usize, 10);
    assert_eq!(g.cell(3, 1).bg, 2);
}

#[test]
fn peeking_past_the_end_of_the_grid_gives_nothing() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    assert!(unsafe { grid_peek_line(&*g.ptr(), 1) }.is_some());
    assert!(unsafe { grid_peek_line(&*g.ptr(), 2) }.is_none());
    assert!(::core::ptr::eq(
        unsafe { grid_get_line(&mut *g.ptr(), 1) },
        unsafe { &(*g.ptr()).linedata[1] }
    ));
}

#[test]
fn setting_a_cell_past_the_end_of_the_grid_does_nothing() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    let gc = ascii(b'a');
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 2, &gc) };
    unsafe { grid_set_cells(&mut *g.ptr(), 0, 2, &gc, b"abc") };
    assert_eq!(g.line(0).cellsize() as usize, 0);
    assert_eq!(g.line(1).cellsize() as usize, 0);
}

#[test]
fn a_run_of_cells_shares_one_style() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    let mut gc = ascii(b'?');
    gc.fg = 3;
    unsafe { grid_set_cells(&mut *g.ptr(), 2, 0, &gc, b"abc") };
    assert_eq!(g.line(0).cellused, 5);
    assert_eq!(text_of(&g.cell(2, 0)), "a");
    assert_eq!(text_of(&g.cell(4, 0)), "c");
    assert_eq!(g.cell(3, 0).fg, 3);
    assert_eq!(g.text(0), "  abc");
}

#[test]
fn a_run_of_extended_cells_keeps_one_byte_each() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    let mut gc = ascii(b'?');
    gc.us = 2;
    unsafe { grid_set_cells(&mut *g.ptr(), 0, 0, &gc, b"xyz") };
    assert_eq!(g.text(0), "xyz");
    assert_eq!(g.cell(1, 0).us, 2);
    assert_eq!(g.line(0).extdsize() as usize, 3);
}

#[test]
fn clearing_nothing_does_nothing() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    g.write(0, 0, "abcde");
    unsafe {
        grid_clear(&mut *g.ptr(), 0, 0, 0, 1, 8);
        grid_clear(&mut *g.ptr(), 0, 0, 1, 0, 8);
        grid_clear_lines(&mut *g.ptr(), 0, 0, 8);
    }
    assert_eq!(g.text(0), "abcde");
}

#[test]
fn clearing_a_whole_width_clears_the_lines() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    g.write(0, 0, "abcde");
    unsafe { grid_clear(&mut *g.ptr(), 0, 0, 10, 1, 8) };
    assert_eq!(
        g.line(0).cellsize() as usize,
        0,
        "the line was freed, not painted"
    );
}

#[test]
fn clearing_past_the_end_of_the_grid_does_nothing() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    g.write(0, 1, "abcde");
    unsafe {
        grid_clear(&mut *g.ptr(), 1, 2, 3, 1, 8);
        grid_clear(&mut *g.ptr(), 1, 1, 3, 5, 8);
        grid_clear_lines(&mut *g.ptr(), 2, 1, 8);
        grid_clear_lines(&mut *g.ptr(), 1, 5, 8);
    }
    assert_eq!(g.text(1), "abcde");
}

#[test]
fn a_count_that_has_gone_round_zero_clears_and_moves_nothing() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    g.write(0, 0, "abcde");
    g.write(0, 1, "fghij");
    // The grid view works its counts out from screen sizes and hands
    // over what it gets, which is a very large number when the
    // subtraction went below zero.
    unsafe {
        grid_clear(&mut *g.ptr(), 1, 0, 3, UINT_MAX, 8);
        grid_clear_lines(&mut *g.ptr(), 0, UINT_MAX, 8);
        grid_move_lines(&mut *g.ptr(), 0, 1, UINT_MAX, 8);
    }
    assert_eq!(g.text(0), "abcde");
    assert_eq!(g.text(1), "fghij");
}

#[test]
fn a_default_background_clear_stops_at_the_cells_the_line_has() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    g.write(0, 0, "abcdefghij");
    assert_eq!(g.line(0).cellsize() as usize, 10);
    unsafe { grid_clear(&mut *g.ptr(), 8, 0, 5, 1, 8) };
    assert_eq!(g.line(0).cellsize() as usize, 10, "the line did not grow");
    assert_eq!(g.text(0), "abcdefgh  ", "the cleared cells are still used");

    // A line with no cells at all is skipped entirely.
    unsafe { grid_clear(&mut *g.ptr(), 3, 1, 2, 1, 8) };
    assert_eq!(g.line(1).cellsize() as usize, 0);
}

#[test]
fn a_coloured_clear_grows_the_line_to_reach_the_cells() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    unsafe { grid_clear(&mut *g.ptr(), 3, 1, 2, 1, 4) };
    assert_eq!(
        g.line(1).cellsize() as usize,
        10,
        "grown by the expansion rule"
    );
    assert_eq!(g.cell(3, 1).bg, 4);
    assert_eq!(g.cell(4, 1).bg, 4);
    assert_eq!(g.cell(2, 1).bg, 8);
}

#[test]
fn a_cleared_cell_takes_the_background_in_the_form_the_colour_needs() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    unsafe {
        grid_clear(&mut *g.ptr(), 0, 0, 1, 1, 2);
        grid_clear(&mut *g.ptr(), 1, 0, 1, 1, 3 | COLOUR_FLAG_256 as u_int);
        grid_clear(
            &mut *g.ptr(),
            2,
            0,
            1,
            1,
            0x334455 | COLOUR_FLAG_RGB as u_int,
        );
    }
    assert_eq!(g.cell(0, 0).bg, 2);
    assert_eq!(g.entry(0, 0).flags as c_int, GRID_FLAG_CLEARED);
    assert_eq!(g.cell(1, 0).bg, 3 | COLOUR_FLAG_256);
    assert_eq!(
        g.entry(1, 0).flags as c_int,
        GRID_FLAG_CLEARED | GRID_FLAG_BG256
    );
    assert_eq!(g.cell(2, 0).bg, 0x334455 | COLOUR_FLAG_RGB);
    assert!(g.entry(2, 0).flags as c_int & GRID_FLAG_EXTENDED != 0);
}

#[test]
fn clearing_an_extended_cell_keeps_its_slot() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    let mut gc = ascii(b'a');
    gc.us = 4;
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };
    unsafe { grid_set_cell(&mut *g.ptr(), 1, 0, &gc) };
    assert_eq!(g.line(0).extdsize() as usize, 2);

    unsafe { grid_clear(&mut *g.ptr(), 0, 0, 1, 1, 8) };
    assert_eq!(g.line(0).extdsize() as usize, 2, "the slot was reused");
    assert_eq!(text_of(&g.cell(0, 0)), " ");
    assert_eq!(g.cell(0, 0).bg, 8);

    unsafe { grid_clear(&mut *g.ptr(), 1, 0, 1, 1, 5) };
    assert_eq!(g.line(0).extdsize() as usize, 2);
    assert_eq!(g.cell(1, 0).bg, 5);
}

#[test]
fn clearing_lines_takes_the_wrap_flag_off_the_line_above() {
    let _guard = globals();
    let g = Grid::new(10, 3, 0);
    g.write(0, 0, "abc");
    g.write(0, 1, "def");
    unsafe {
        (*g.ptr()).linedata[0].flags |= GRID_LINE_WRAPPED;
        grid_clear_lines(&mut *g.ptr(), 1, 1, 8);
    }
    assert_eq!(g.line(0).flags, 0);
    assert_eq!(g.text(1), "");
    assert_eq!(g.text(0), "abc");
}

#[test]
fn cleared_lines_keep_a_coloured_background() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    g.write(0, 0, "abc");
    unsafe { grid_clear_lines(&mut *g.ptr(), 0, 1, 6) };
    assert_eq!(g.line(0).cellsize() as usize, 10);
    assert_eq!(g.cell(0, 0).bg, 6);
}

#[test]
fn moving_no_lines_or_onto_themselves_does_nothing() {
    let _guard = globals();
    let g = Grid::new(10, 3, 0);
    g.write(0, 0, "abc");
    unsafe {
        grid_move_lines(&mut *g.ptr(), 1, 0, 0, 8);
        grid_move_lines(&mut *g.ptr(), 0, 0, 1, 8);
    }
    assert_eq!(g.text(0), "abc");
}

#[test]
fn moving_lines_past_the_end_of_the_grid_does_nothing() {
    let _guard = globals();
    let g = Grid::new(10, 3, 0);
    g.write(0, 0, "abc");
    unsafe {
        grid_move_lines(&mut *g.ptr(), 0, 3, 1, 8);
        grid_move_lines(&mut *g.ptr(), 0, 1, 5, 8);
        grid_move_lines(&mut *g.ptr(), 3, 0, 1, 8);
        grid_move_lines(&mut *g.ptr(), 2, 0, 2, 8);
    }
    assert_eq!(g.text(0), "abc");
}

#[test]
fn moved_lines_leave_empty_lines_behind() {
    let _guard = globals();
    let g = Grid::new(10, 4, 0);
    g.write(0, 0, "one");
    g.write(0, 1, "two");
    g.write(0, 2, "three");
    unsafe {
        (*g.ptr()).linedata[2].flags |= GRID_LINE_WRAPPED;
        grid_move_lines(&mut *g.ptr(), 0, 1, 2, 8);
    }
    assert_eq!(g.text(0), "two");
    assert_eq!(g.text(1), "three");
    assert_eq!(g.text(2), "", "the source line was emptied");
    assert_eq!(g.line(1).flags & GRID_LINE_WRAPPED, GRID_LINE_WRAPPED);
}

#[test]
fn a_line_moved_up_takes_the_wrap_flag_off_the_line_above_it() {
    let _guard = globals();
    let g = Grid::new(10, 4, 0);
    g.write(0, 1, "two");
    g.write(0, 2, "three");
    unsafe {
        (*g.ptr()).linedata[1].flags |= GRID_LINE_WRAPPED;
        grid_move_lines(&mut *g.ptr(), 0, 2, 1, 8);
    }
    assert_eq!(g.text(0), "three");
    assert_eq!(g.text(2), "");
    assert_eq!(
        g.line(1).flags & GRID_LINE_WRAPPED,
        0,
        "the line above the source no longer continues onto it"
    );
}

#[test]
fn overlapping_line_moves_keep_the_lines_they_still_need() {
    let _guard = globals();
    let g = Grid::new(10, 4, 0);
    g.write(0, 0, "one");
    g.write(0, 1, "two");
    g.write(0, 2, "three");
    unsafe {
        (*g.ptr()).linedata[0].flags |= GRID_LINE_WRAPPED;
        (*g.ptr()).linedata[1].flags |= GRID_LINE_WRAPPED;
        grid_move_lines(&mut *g.ptr(), 1, 0, 2, 8);
    }
    assert_eq!(g.text(1), "one");
    assert_eq!(g.text(2), "two");
    assert_eq!(g.text(0), "", "the line the move left behind");
    assert_eq!(g.line(0).flags, 0);
    assert_eq!(
        g.line(1).flags & GRID_LINE_WRAPPED,
        0,
        "the wrap flag is taken off the line above the destination \
         before the move, so the moved line loses it too"
    );
    assert_eq!(g.line(2).flags & GRID_LINE_WRAPPED, GRID_LINE_WRAPPED);
}

#[test]
fn moving_no_cells_or_onto_themselves_does_nothing() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    g.write(0, 0, "abc");
    unsafe {
        grid_move_cells(&mut *g.ptr(), 1, 0, 0, 0, 8);
        grid_move_cells(&mut *g.ptr(), 0, 0, 0, 3, 8);
        grid_move_cells(&mut *g.ptr(), 1, 0, 2, 3, 8);
    }
    assert_eq!(g.text(0), "abc");
}

#[test]
fn moved_cells_leave_cleared_cells_behind() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    g.write(0, 0, "abcdef");
    unsafe { grid_move_cells(&mut *g.ptr(), 0, 3, 0, 3, 8) };
    assert_eq!(g.text(0), "def   ");
    assert_eq!(g.line(0).cellused, 6);

    g.write(0, 1, "abcdef");
    unsafe { grid_move_cells(&mut *g.ptr(), 1, 0, 1, 3, 2) };
    assert_eq!(g.text(1), " abcef");
    assert_eq!(g.cell(0, 1).bg, 2, "only the cell outside the move");
    assert_eq!(g.cell(1, 1).bg, 8);
}

#[test]
fn moving_cells_over_an_extended_cell_drops_its_slot() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    let mut gc = ascii(b'a');
    gc.us = 4;
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };
    g.write(1, 0, "bc");
    assert_eq!(g.line(0).extdsize() as usize, 1);
    unsafe { grid_move_cells(&mut *g.ptr(), 3, 0, 0, 1, 8) };
    assert_eq!(g.cell(3, 0).us, 4);
    assert_eq!(
        g.entry(0, 0).flags as c_int,
        GRID_FLAG_CLEARED,
        "the moved-from cell forgets the extended slot"
    );
    assert_eq!(
        g.line(0).extdsize() as usize,
        1,
        "the slot itself is still there"
    );
}

#[test]
fn history_grows_by_one_line_at_a_time() {
    let _guard = globals();
    let g = Grid::new(10, 2, 100);
    g.write(0, 0, "one");
    g.write(0, 1, "two");
    unsafe { grid_scroll_history(&mut *g.ptr(), 8) };
    assert_eq!(g.hsize, 1);
    assert_eq!(g.hscrolled, 1);
    assert_eq!(g.text(0), "one");
    assert_eq!(g.text(1), "two");
    assert_eq!(g.text(2), "");
    assert_eq!(g.line(0).time, unsafe { current_time });
}

#[test]
fn a_scrolled_line_gives_back_the_extended_slots_it_stopped_using() {
    let _guard = globals();
    let g = Grid::new(10, 2, 100);
    let mut gc = ascii(b'a');
    gc.us = 4;
    unsafe {
        grid_set_cell(&mut *g.ptr(), 0, 0, &gc);
        grid_set_cell(&mut *g.ptr(), 2, 0, &gc);
    }
    g.write(1, 0, "b");
    assert_eq!(g.line(0).extdsize() as usize, 2);

    // Move the first cell away: its entry stops being extended, so the
    // slot is dead but still allocated.
    unsafe { grid_move_cells(&mut *g.ptr(), 5, 0, 0, 1, 8) };
    assert_eq!(g.line(0).extdsize() as usize, 2);

    unsafe { grid_scroll_history(&mut *g.ptr(), 8) };
    assert_eq!(
        g.line(0).extdsize() as usize,
        2,
        "one for each cell still extended"
    );
    assert_eq!(g.cell(2, 0).us, 4);
    assert_eq!(g.cell(5, 0).us, 4);
}

#[test]
fn a_scrolled_line_with_no_extended_cells_left_frees_them_all() {
    let _guard = globals();
    let g = Grid::new(10, 2, 100);
    let mut gc = ascii(b'a');
    gc.us = 4;
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };
    assert_eq!(g.line(0).extdsize() as usize, 1);
    unsafe { grid_move_cells(&mut *g.ptr(), 5, 0, 0, 1, 8) };
    unsafe {
        let gl = &mut (*g.ptr()).linedata[0];
        gl.celldata_mut()[5].flags = GRID_FLAG_CLEARED as u_char;
    }
    unsafe { grid_scroll_history(&mut *g.ptr(), 8) };
    assert_eq!(g.line(0).extdsize() as usize, 0);
    assert!(g.line(0).extddata().is_empty());
}

#[test]
fn a_scrolled_line_with_no_extended_data_is_left_alone() {
    let _guard = globals();
    let g = Grid::new(10, 2, 100);
    g.write(0, 0, "abc");
    unsafe { grid_scroll_history(&mut *g.ptr(), 8) };
    assert_eq!(g.line(0).extdsize() as usize, 0);
    assert_eq!(g.text(0), "abc");
}

#[test]
fn a_scrolled_line_keeps_the_background_it_was_given() {
    let _guard = globals();
    let g = Grid::new(10, 2, 100);
    unsafe { grid_scroll_history(&mut *g.ptr(), 3) };
    assert_eq!(g.line(2).cellsize() as usize, 10);
    assert_eq!(g.cell(0, 2).bg, 3);
}

#[test]
fn history_is_collected_in_tenths_once_it_is_full() {
    let _guard = globals();
    let g = Grid::new(10, 1, 20);
    for i in 0..20 {
        g.write(0, g.hsize, &format!("{i}"));
        unsafe { grid_scroll_history(&mut *g.ptr(), 8) };
    }
    assert_eq!(g.hsize, 20);
    unsafe { grid_collect_history(&mut *g.ptr(), 0) };
    assert_eq!(g.hsize, 18, "a tenth of the limit");
    assert_eq!(g.hscrolled, 18);
    assert_eq!(g.text(0), "2");
}

#[test]
fn collecting_all_of_the_history_leaves_the_limit() {
    let _guard = globals();
    let g = Grid::new(10, 1, 5);
    for i in 0..8 {
        g.write(0, g.hsize, &format!("{i}"));
        unsafe { grid_scroll_history(&mut *g.ptr(), 8) };
    }
    assert_eq!(g.hsize, 8);
    unsafe { grid_collect_history(&mut *g.ptr(), 1) };
    assert_eq!(g.hsize, 5);
    assert_eq!(g.text(0), "3");
}

#[test]
fn collecting_history_that_is_not_full_does_nothing() {
    let _guard = globals();
    let g = Grid::new(10, 1, 100);
    unsafe { grid_collect_history(&mut *g.ptr(), 0) };
    assert_eq!(g.hsize, 0);
    g.write(0, 1, "a");
    unsafe {
        grid_scroll_history(&mut *g.ptr(), 8);
        grid_collect_history(&mut *g.ptr(), 0);
    }
    assert_eq!(g.hsize, 1);
}

#[test]
fn a_collection_always_takes_at_least_one_line() {
    let _guard = globals();
    let g = Grid::new(10, 1, 5);
    for i in 0..5 {
        g.write(0, g.hsize, &format!("{i}"));
        unsafe { grid_scroll_history(&mut *g.ptr(), 8) };
    }
    assert_eq!(g.hsize, 5);
    unsafe { grid_collect_history(&mut *g.ptr(), 0) };
    assert_eq!(g.hsize, 4, "a tenth of five is none, so one line goes");
    unsafe {
        grid_scroll_history(&mut *g.ptr(), 8);
        grid_collect_history(&mut *g.ptr(), 1);
    }
    assert_eq!(g.hsize, 4, "with nothing over the limit, one line goes");
}

#[test]
fn collecting_history_pulls_the_scroll_position_back() {
    let _guard = globals();
    let g = Grid::new(10, 1, 5);
    for _ in 0..8 {
        unsafe { grid_scroll_history(&mut *g.ptr(), 8) };
    }
    assert_eq!(g.hscrolled, 8);
    unsafe { grid_collect_history(&mut *g.ptr(), 1) };
    assert_eq!(g.hsize, 5);
    assert_eq!(g.hscrolled, 5);
}

#[test]
fn history_can_be_removed_from_the_bottom() {
    let _guard = globals();
    let g = Grid::new(10, 1, 100);
    for i in 0..3 {
        g.write(0, g.hsize, &format!("{i}"));
        unsafe { grid_scroll_history(&mut *g.ptr(), 8) };
    }
    unsafe { grid_remove_history(&mut *g.ptr(), 5) };
    assert_eq!(g.hsize, 3, "more than there is does nothing");
    unsafe { grid_remove_history(&mut *g.ptr(), 2) };
    assert_eq!(g.hsize, 1);
    assert_eq!(g.text(0), "0");
    assert_eq!(
        g.text(1),
        "1",
        "the bottom lines go, so the screen now sits on history"
    );
}

#[test]
fn clearing_the_history_leaves_the_screen() {
    let _guard = globals();
    let g = Grid::new(10, 2, 100);
    unsafe {
        grid_scroll_history(&mut *g.ptr(), 8);
        grid_scroll_history(&mut *g.ptr(), 8);
    }
    assert_eq!(g.hsize, 2);
    g.write(0, 2, "keep");
    unsafe { grid_clear_history(&mut *g.ptr()) };
    assert_eq!(g.hsize, 0);
    assert_eq!(g.hscrolled, 0);
    assert_eq!(g.text(0), "keep");
    assert_eq!(g.text(1), "");
}

#[test]
fn scrolling_a_region_moves_its_top_line_into_the_history() {
    let _guard = globals();
    let g = Grid::new(10, 4, 100);
    g.write(0, 0, "one");
    g.write(0, 1, "two");
    g.write(0, 2, "three");
    g.write(0, 3, "four");
    unsafe { grid_scroll_history_region(&mut *g.ptr(), 0, 2, 8) };
    assert_eq!(g.hsize, 1);
    assert_eq!(g.hscrolled, 1);
    assert_eq!(g.text(0), "one", "the line that left the region");
    assert_eq!(g.text(1), "two");
    assert_eq!(g.text(2), "three");
    assert_eq!(g.text(3), "", "the bottom of the region is empty now");
    assert_eq!(g.text(4), "four");
    assert_eq!(g.line(0).time, unsafe { current_time });
}

#[test]
fn a_scrolled_region_can_take_a_background() {
    let _guard = globals();
    let g = Grid::new(10, 3, 100);
    unsafe { grid_scroll_history_region(&mut *g.ptr(), 0, 1, 4) };
    assert_eq!(g.cell(0, 2).bg, 4);
}

#[test]
fn lines_can_be_made_room_for_and_emptied() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    g.write(0, 0, "keep");
    unsafe {
        grid_adjust_lines(&mut *g.ptr(), 4);
        grid_empty_line(&mut *g.ptr(), 2, 8);
        grid_empty_line(&mut *g.ptr(), 3, 8);
        (*g.ptr()).sy = 4;
    }
    assert_eq!(g.text(0), "keep");
    assert_eq!(g.line(3).cellsize() as usize, 0);
    g.write(0, 3, "new");
    assert_eq!(g.text(3), "new");
}

#[test]
fn grids_compare_by_size_and_by_every_cell_of_the_screen() {
    let _guard = globals();
    let a = Grid::new(10, 2, 0);
    let b = Grid::new(10, 2, 0);
    a.write(0, 0, "abc");
    b.write(0, 0, "abc");
    assert_eq!(unsafe { grid_compare(&*a.ptr(), &*b.ptr()) }, 0);

    let wider = Grid::new(11, 2, 0);
    assert_eq!(unsafe { grid_compare(&*a.ptr(), &*wider.ptr()) }, 1);
    let taller = Grid::new(10, 3, 0);
    assert_eq!(unsafe { grid_compare(&*a.ptr(), &*taller.ptr()) }, 1);

    b.write(8, 0, "d");
    assert_eq!(
        unsafe { grid_compare(&*a.ptr(), &*b.ptr()) },
        1,
        "different cell counts"
    );
    b.write(0, 0, "abc");
    unsafe { grid_empty_line(&mut *b.ptr(), 0, 8) };
    b.write(0, 0, "abc");
    assert_eq!(unsafe { grid_compare(&*a.ptr(), &*b.ptr()) }, 0);
    b.write(3, 0, "d");
    a.write(3, 0, "e");
    assert_eq!(
        unsafe { grid_compare(&*a.ptr(), &*b.ptr()) },
        1,
        "different cells"
    );
    a.write(3, 0, "d");
    assert_eq!(unsafe { grid_compare(&*a.ptr(), &*b.ptr()) }, 0);
}

/// A screen with nothing but the hyperlink table `grid_string_cells`
/// reads.
struct Screen(Box<screen>);

impl Screen {
    fn new() -> Screen {
        Screen(Box::new(crate::types::screen::new(1, 1, 0)))
    }

    fn put(&mut self, uri: &CStr, id: &CStr) -> u_int {
        unsafe {
            hyperlinks_put(
                self.0.hyperlinks_ref().expect("a hyperlink store"),
                uri,
                Some(id),
            )
        }
    }

    fn ptr(&mut self) -> *mut screen {
        &raw mut *self.0
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        unsafe { screen_free(&mut *self.0) };
    }
}

/// Renders a line with the escape sequences, starting from a fresh last
/// cell.
fn coded(g: &Grid, py: u_int, flags: c_int, sc: *mut screen) -> String {
    let mut last: grid_cell = grid_default_cell;
    g.render(py, flags | GRID_STRING_WITH_SEQUENCES, Some(&mut last), sc)
}

/// One line holding cells the caller has styled, rendered with sequences.
fn styled(cells: &[grid_cell], flags: c_int) -> String {
    let g = Grid::new(80, 1, 0);
    for (i, gc) in cells.iter().enumerate() {
        unsafe { grid_set_cell(&mut *g.ptr(), i as u_int, 0, gc) };
    }
    coded(&g, 0, flags, null_mut())
}

#[test]
fn a_line_past_the_end_of_the_grid_renders_as_nothing() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    assert_eq!(g.text(2), "");
}

#[test]
fn padding_cells_are_left_out_of_the_text() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    g.write(0, 0, "a");
    unsafe { grid_set_padding(&mut *g.ptr(), 1, 0) };
    g.write(2, 0, "c");
    assert_eq!(g.text(0), "ac");
}

#[test]
fn a_tab_cell_renders_as_one_tab() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    let mut gc = ascii(b' ');
    unsafe { grid_set_tab(&mut gc, 4) };
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };
    g.write(1, 0, "x");
    assert_eq!(g.text(0), "\tx");
}

#[test]
fn trailing_spaces_can_be_trimmed() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    g.write(0, 0, "ab   ");
    assert_eq!(g.text(0), "ab   ");
    assert_eq!(g.render(0, GRID_STRING_TRIM_SPACES, None, null_mut()), "ab");
}

#[test]
fn empty_cells_can_be_asked_for() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    g.write(0, 0, "abc");
    assert_eq!(g.line(0).cellused, 3);
    assert_eq!(g.line(0).cellsize() as usize, 5);
    assert_eq!(g.text(0), "abc");
    assert_eq!(
        g.render(0, GRID_STRING_EMPTY_CELLS, None, null_mut()),
        "abc  "
    );
}

#[test]
fn a_backslash_is_doubled_when_the_sequences_are_escaped() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    g.write(0, 0, "a\\b");
    assert_eq!(g.text(0), "a\\b");
    assert_eq!(
        g.render(0, GRID_STRING_ESCAPE_SEQUENCES, None, null_mut()),
        "a\\\\b"
    );
}

#[test]
fn a_long_line_grows_the_answer_as_it_goes() {
    let _guard = globals();
    let g = Grid::new(400, 1, 0);
    let text: String = (0..300).map(|i| (b'a' + (i % 26) as u8) as char).collect();
    g.write(0, 0, &text);
    assert_eq!(g.text(0), text);
}

#[test]
fn colours_only_appear_when_they_change() {
    let _guard = globals();
    let mut one = ascii(b'a');
    one.fg = 1;
    let mut two = ascii(b'b');
    two.fg = 1;
    let three = ascii(b'c');
    assert_eq!(
        styled(&[one, two, three], 0),
        "\u{1b}[31ma b\u{1b}[39mc".replace(' ', "")
    );
}

#[test]
fn each_way_of_naming_a_foreground_colour() {
    let _guard = globals();
    let of = |fg: c_int| {
        let mut gc = ascii(b'x');
        gc.fg = fg;
        styled(&[gc], 0)
    };
    assert_eq!(of(3), "\u{1b}[33mx");
    assert_eq!(of(8), "x", "the default needs no code");
    assert_eq!(of(93), "\u{1b}[93mx");
    assert_eq!(of(200 | COLOUR_FLAG_256), "\u{1b}[38;5;200mx");
    assert_eq!(of(0x102030 | COLOUR_FLAG_RGB), "\u{1b}[38;2;16;32;48mx");
    assert_eq!(of(50), "x", "a colour with no sequence says nothing");
}

#[test]
fn each_way_of_naming_a_background_colour() {
    let _guard = globals();
    let of = |bg: c_int| {
        let mut gc = ascii(b'x');
        gc.bg = bg;
        styled(&[gc], 0)
    };
    assert_eq!(of(3), "\u{1b}[43mx");
    assert_eq!(of(8), "x");
    assert_eq!(of(93), "\u{1b}[103mx");
    assert_eq!(of(200 | COLOUR_FLAG_256), "\u{1b}[48;5;200mx");
    assert_eq!(of(0x102030 | COLOUR_FLAG_RGB), "\u{1b}[48;2;16;32;48mx");
    assert_eq!(of(50), "x");
}

#[test]
fn an_underscore_colour_is_only_named_when_it_has_one() {
    let _guard = globals();
    let of = |us: c_int| {
        let mut gc = ascii(b'x');
        gc.us = us;
        styled(&[gc], 0)
    };
    assert_eq!(of(3 | COLOUR_FLAG_256), "\u{1b}[58;5;3mx");
    assert_eq!(of(0x102030 | COLOUR_FLAG_RGB), "\u{1b}[58;2;16;32;48mx");
    assert_eq!(of(3), "x", "a plain colour has no underscore sequence");
}

#[test]
fn a_new_attribute_is_named_and_a_removed_one_resets_everything() {
    let _guard = globals();
    let attr = |bits: c_int, ch: u8| {
        let mut gc = ascii(ch);
        gc.attr = bits as u_short;
        gc
    };
    assert_eq!(styled(&[attr(GRID_ATTR_BRIGHT, b'a')], 0), "\u{1b}[1ma");
    assert_eq!(
        styled(&[attr(GRID_ATTR_UNDERSCORE_2, b'a')], 0),
        "\u{1b}[4:2ma",
        "the two digit codes are written as a colon pair"
    );
    assert_eq!(
        styled(
            &[
                attr(GRID_ATTR_BRIGHT | GRID_ATTR_DIM, b'a'),
                attr(GRID_ATTR_BRIGHT, b'b')
            ],
            0
        ),
        "\u{1b}[1;2ma\u{1b}[0;1mb",
        "the reset and the attributes that stay share one sequence"
    );
}

#[test]
fn a_reset_writes_the_colours_again_unless_they_are_the_default() {
    let _guard = globals();
    let mut one = ascii(b'a');
    one.attr = GRID_ATTR_BRIGHT as u_short;
    one.fg = 1;
    let mut two = ascii(b'b');
    two.fg = 1;
    assert_eq!(
        styled(&[one, two], 0),
        "\u{1b}[1m\u{1b}[31ma\u{1b}[0m\u{1b}[31mb",
        "the attributes and the colours are separate sequences"
    );

    let mut three = ascii(b'a');
    three.attr = GRID_ATTR_BRIGHT as u_short;
    let four = ascii(b'b');
    assert_eq!(
        styled(&[three, four], 0),
        "\u{1b}[1ma\u{1b}[0mb",
        "a default colour after a reset is left out"
    );
}

#[test]
fn losing_an_underscore_colour_resets_everything_too() {
    let _guard = globals();
    let mut one = ascii(b'a');
    one.us = 2 | COLOUR_FLAG_256;
    let two = ascii(b'b');
    assert_eq!(styled(&[one, two], 0), "\u{1b}[58;5;2ma\u{1b}[0mb");
}

#[test]
fn the_alternate_character_set_is_switched_in_and_out() {
    let _guard = globals();
    let mut one = ascii(b'a');
    one.attr = GRID_ATTR_CHARSET as u_short;
    let two = ascii(b'b');
    assert_eq!(styled(&[one, two], 0), "\u{e}a\u{f}b");
}

#[test]
fn escaped_sequences_are_written_with_backslashes() {
    let _guard = globals();
    let mut one = ascii(b'a');
    one.attr = (GRID_ATTR_CHARSET | GRID_ATTR_BRIGHT) as u_short;
    one.fg = 1;
    let two = ascii(b'b');
    assert_eq!(
        styled(&[one, two], GRID_STRING_ESCAPE_SEQUENCES),
        "\\033[1m\\033[31m\\016a\\033[0m\\017b"
    );
}

#[test]
fn an_escaped_hyperlink_is_written_with_backslashes() {
    let _guard = globals();
    let mut sc = Screen::new();
    let link = sc.put(c"http://one", c"id1");
    let g = Grid::new(10, 1, 0);
    let mut gc = ascii(b'a');
    gc.link = link;
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };
    g.write(1, 0, "b");

    assert_eq!(
        coded(&g, 0, GRID_STRING_ESCAPE_SEQUENCES, sc.ptr()),
        "\\033]8;id=id1;http://one\\033\\\\a\\033]8;;\\033\\\\b"
    );
}

#[test]
fn a_line_that_ends_with_a_hyperlink_grows_the_answer_for_it() {
    let _guard = globals();
    let mut sc = Screen::new();
    let uri = ::std::ffi::CString::new(format!("http://{}", "u".repeat(200))).unwrap();
    let link = sc.put(&uri, c"");
    let g = Grid::new(10, 1, 0);
    let mut gc = ascii(b'a');
    gc.link = link;
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };

    let out = coded(&g, 0, 0, sc.ptr());
    assert_eq!(out.matches("http://").count(), 2);
    assert!(out.ends_with("\u{1b}]8;;\u{1b}\\"), "{out}");
}

#[test]
fn a_hyperlink_is_opened_and_closed_around_the_cells_that_have_one() {
    let _guard = globals();
    let mut sc = Screen::new();
    let link = sc.put(c"http://one", c"id1");
    let g = Grid::new(10, 1, 0);
    let mut gc = ascii(b'a');
    gc.link = link;
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };
    g.write(1, 0, "b");

    assert_eq!(
        coded(&g, 0, 0, sc.ptr()),
        "\u{1b}]8;id=id1;http://one\u{1b}\\a\u{1b}]8;;\u{1b}\\b"
    );
}

#[test]
fn a_hyperlink_still_open_at_the_end_of_the_line_is_closed() {
    let _guard = globals();
    let mut sc = Screen::new();
    let link = sc.put(c"http://one", c"");
    let g = Grid::new(10, 1, 0);
    let mut gc = ascii(b'a');
    gc.link = link;
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };

    assert_eq!(
        coded(&g, 0, 0, sc.ptr()),
        "\u{1b}]8;;http://one\u{1b}\\a\u{1b}]8;;http://one\u{1b}\\\u{1b}]8;;\u{1b}\\",
        "an empty id is written as no id at all, and the closing sequence \
         is appended to whatever the last cell left in the code buffer"
    );
}

#[test]
fn a_hyperlink_too_long_for_the_buffer_is_left_out() {
    let _guard = globals();
    let mut sc = Screen::new();
    let long = ::std::ffi::CString::new("h".repeat(8200)).unwrap();
    let link = sc.put(&long, c"");
    let g = Grid::new(10, 1, 0);
    let mut gc = ascii(b'a');
    gc.link = link;
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };
    assert_eq!(coded(&g, 0, 0, sc.ptr()), "a");
}

#[test]
fn a_screen_without_hyperlinks_never_asks_for_one() {
    let _guard = globals();
    let mut sc = Screen::new();
    sc.0.hyperlinks = None;
    let g = Grid::new(10, 1, 0);
    let mut gc = ascii(b'a');
    gc.link = 1;
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };
    assert_eq!(coded(&g, 0, 0, sc.ptr()), "a");
}

#[test]
fn lines_can_be_duplicated_between_grids() {
    let _guard = globals();
    let src = Grid::new(10, 3, 0);
    let dst = Grid::new(10, 3, 0);
    src.write(0, 0, "one");
    let mut gc = ascii(b'x');
    gc.us = 4;
    unsafe { grid_set_cell(&mut *src.ptr(), 0, 1, &gc) };
    dst.write(0, 2, "gone");

    unsafe { grid_duplicate_lines(&mut *dst.ptr(), 0, &*src.ptr(), 0, 3) };
    assert_eq!(dst.text(0), "one");
    assert_eq!(dst.cell(0, 1).us, 4);
    assert_eq!(dst.text(2), "");
    assert!(dst.line(2).celldata().is_empty());
    assert!(dst.line(0).extddata().is_empty());
    assert_ne!(
        dst.line(1).extddata().as_ptr(),
        src.line(1).extddata().as_ptr()
    );

    src.write(0, 0, "two");
    assert_eq!(dst.text(0), "one", "the copy is its own");
}

#[test]
fn duplication_stops_at_the_end_of_either_grid() {
    let _guard = globals();
    let src = Grid::new(10, 3, 0);
    let dst = Grid::new(10, 2, 0);
    src.write(0, 0, "one");
    src.write(0, 1, "two");
    src.write(0, 2, "three");

    unsafe { grid_duplicate_lines(&mut *dst.ptr(), 1, &*src.ptr(), 0, 3) };
    assert_eq!(dst.text(1), "one");

    unsafe { grid_duplicate_lines(&mut *dst.ptr(), 0, &*src.ptr(), 2, 2) };
    assert_eq!(dst.text(0), "three");
    assert_eq!(dst.text(1), "one", "the second line was not reached");
}

#[test]
fn wrapped_lines_add_up_to_one_position() {
    let _guard = globals();
    let g = Grid::new(10, 3, 0);
    g.write(0, 0, "abcde");
    g.write(0, 1, "fgh");
    g.write(0, 2, "ij");
    unsafe {
        (*g.ptr()).linedata[0].flags |= GRID_LINE_WRAPPED;
        (*g.ptr()).linedata[1].flags |= GRID_LINE_WRAPPED;
    }

    let (mut wx, mut wy) = (0, 0);
    (wx, wy) = unsafe { grid_wrap_position(&*g.ptr(), 1, 2) };
    assert_eq!((wx, wy), (9, 0), "five and three cells came before it");

    (wx, wy) = unsafe { grid_wrap_position(&*g.ptr(), 5, 2) };
    assert_eq!(wx, UINT_MAX, "past the end of the line");

    let (mut px, mut py) = (0, 0);
    (px, py) = unsafe { grid_unwrap_position(&*g.ptr(), 9, 0) };
    assert_eq!((px, py), (1, 2));

    (px, py) = unsafe { grid_unwrap_position(&*g.ptr(), UINT_MAX, 0) };
    assert_eq!((px, py), (2, 2), "the end of the wrapped run");

    (px, py) = unsafe { grid_unwrap_position(&*g.ptr(), 3, 0) };
    assert_eq!((px, py), (3, 0), "still on the first line of the run");
}

#[test]
fn an_unwrapped_line_is_its_own_position() {
    let _guard = globals();
    let g = Grid::new(10, 3, 0);
    g.write(0, 0, "abc");
    g.write(0, 1, "def");
    g.write(0, 2, "ghi");

    let (mut wx, mut wy) = (0, 0);
    (wx, wy) = unsafe { grid_wrap_position(&*g.ptr(), 2, 2) };
    assert_eq!((wx, wy), (2, 2));

    let (mut px, mut py) = (0, 0);
    (px, py) = unsafe { grid_unwrap_position(&*g.ptr(), 2, 2) };
    assert_eq!((px, py), (2, 2));
}

#[test]
fn a_line_is_as_long_as_its_last_non_space_cell() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    assert_eq!(unsafe { grid_line_length(&*g.ptr(), 0) }, 0);
    g.write(0, 0, "ab   ");
    assert_eq!(unsafe { grid_line_length(&*g.ptr(), 0) }, 2);
    g.write(0, 1, "abcdefghijklmno");
    assert_eq!(
        unsafe { grid_line_length(&*g.ptr(), 1) },
        10,
        "never more than the width of the grid"
    );
}

#[test]
fn a_line_ending_in_a_wide_cell_or_padding_is_as_long_as_that_cell() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    let gc = wide("\u{4e2d}", 2);
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };
    unsafe { grid_set_padding(&mut *g.ptr(), 1, 0) };
    assert_eq!(unsafe { grid_line_length(&*g.ptr(), 0) }, 2);
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 1, &gc) };
    assert_eq!(unsafe { grid_line_length(&*g.ptr(), 1) }, 1);
}

#[test]
fn a_cell_can_be_looked_for_in_a_set_of_characters() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    g.write(0, 0, "ab");
    assert_eq!(unsafe { grid_in_set(&*g.ptr(), 0, 0, c"xa") }, 1);
    assert_eq!(unsafe { grid_in_set(&*g.ptr(), 1, 0, c"xa") }, 0);
    unsafe { grid_set_padding(&mut *g.ptr(), 2, 0) };
    assert_eq!(unsafe { grid_in_set(&*g.ptr(), 2, 0, c"xa") }, 0);
}

#[test]
fn a_tab_in_the_set_matches_the_rest_of_a_tab_cell() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    let mut gc = ascii(b' ');
    unsafe { grid_set_tab(&mut gc, 4) };
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 0, &gc) };
    for px in 1..4 {
        unsafe { grid_set_padding(&mut *g.ptr(), px, 0) };
    }
    assert_eq!(unsafe { grid_in_set(&*g.ptr(), 0, 0, c" \t") }, 4);
    assert_eq!(
        unsafe { grid_in_set(&*g.ptr(), 2, 0, c" \t") },
        2,
        "two of the four columns are still to come"
    );
    assert_eq!(
        unsafe { grid_in_set(&*g.ptr(), 1, 0, c" ") },
        0,
        "without a tab in the set the padding is just padding"
    );
}

#[test]
fn padding_with_no_tab_in_front_of_it_is_not_a_tab() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    g.write(0, 0, "a");
    unsafe { grid_set_padding(&mut *g.ptr(), 1, 0) };
    assert_eq!(unsafe { grid_in_set(&*g.ptr(), 1, 0, c" \t") }, 0);
    assert_eq!(
        unsafe { grid_in_set(&*g.ptr(), 0, 0, c" \t") },
        0,
        "the cell itself is not a tab either"
    );
}

/// Marks a line as continuing onto the next one.
fn wrap(g: &Grid, py: u_int) {
    unsafe { (*g.ptr()).linedata[py as usize].flags |= GRID_LINE_WRAPPED };
}

/// The lines of a grid as (text, wrapped) pairs.
fn lines(g: &Grid) -> Vec<(String, bool)> {
    g.dump()
        .into_iter()
        .map(|(text, flags)| (text, flags & GRID_LINE_WRAPPED != 0))
        .collect()
}

#[test]
fn reflowing_to_the_same_width_moves_the_lines_across_unchanged() {
    let _guard = globals();
    let g = Grid::new(5, 2, 0);
    g.write(0, 0, "abcde");
    g.write(0, 1, "fg");
    unsafe { grid_reflow(&mut *g.ptr(), 5) };
    assert_eq!(g.hsize, 0);
    assert_eq!(lines(&g), [("abcde".into(), false), ("fg".into(), false)]);
}

#[test]
fn a_line_too_long_for_the_new_width_is_split() {
    let _guard = globals();
    let g = Grid::new(10, 2, 100);
    g.write(0, 0, "abcdefgh");
    g.write(0, 1, "xy");
    unsafe { grid_reflow(&mut *g.ptr(), 5) };
    assert_eq!(g.hsize, 1, "the split line pushed one line into history");
    assert_eq!(g.hscrolled, 1);
    assert_eq!(
        lines(&g),
        [
            ("abcde".into(), true),
            ("fgh".into(), false),
            ("xy".into(), false)
        ]
    );
}

#[test]
fn a_wrapped_line_takes_back_what_fits_when_the_grid_gets_wider() {
    let _guard = globals();
    let g = Grid::new(5, 2, 0);
    g.write(0, 0, "abcde");
    wrap(&g, 0);
    g.write(0, 1, "fgh");
    unsafe { grid_reflow(&mut *g.ptr(), 10) };
    assert_eq!(g.hsize, 0);
    assert_eq!(lines(&g), [("abcdefgh".into(), false), ("".into(), false)]);
}

#[test]
fn a_join_that_fills_the_line_leaves_the_rest_where_it_was() {
    let _guard = globals();
    let g = Grid::new(5, 2, 0);
    g.write(0, 0, "abc");
    wrap(&g, 0);
    g.write(0, 1, "defgh");
    unsafe { grid_reflow(&mut *g.ptr(), 5) };
    assert_eq!(lines(&g), [("abcde".into(), true), ("fgh".into(), false)]);
}

#[test]
fn a_join_walks_over_the_empty_lines_of_a_wrapped_run() {
    let _guard = globals();
    let g = Grid::new(5, 3, 0);
    g.write(0, 0, "abc");
    wrap(&g, 0);
    wrap(&g, 1);
    g.write(0, 2, "de");
    unsafe { grid_reflow(&mut *g.ptr(), 5) };
    assert_eq!(
        lines(&g),
        [
            ("abcde".into(), false),
            ("".into(), false),
            ("".into(), false)
        ]
    );
}

#[test]
fn a_join_stops_at_an_empty_line_that_is_not_part_of_the_run() {
    let _guard = globals();
    let g = Grid::new(5, 2, 0);
    g.write(0, 0, "abc");
    wrap(&g, 0);
    unsafe { grid_reflow(&mut *g.ptr(), 5) };
    assert_eq!(
        lines(&g),
        [("abc".into(), true), ("".into(), false)],
        "nothing was joined, so the wrap flag stays"
    );
}

#[test]
fn a_wrapped_line_at_the_bottom_of_the_grid_has_nothing_to_join() {
    let _guard = globals();
    let g = Grid::new(5, 1, 0);
    g.write(0, 0, "abc");
    wrap(&g, 0);
    unsafe { grid_reflow(&mut *g.ptr(), 5) };
    assert_eq!(lines(&g), [("abc".into(), true)]);
}

#[test]
fn a_join_stops_when_the_next_character_no_longer_fits() {
    let _guard = globals();
    let g = Grid::new(5, 2, 0);
    g.write(0, 0, "abcde");
    wrap(&g, 0);
    g.write(0, 1, "fg");
    unsafe { grid_reflow(&mut *g.ptr(), 5) };
    assert_eq!(lines(&g), [("abcde".into(), true), ("fg".into(), false)]);
}

#[test]
fn a_split_line_joins_the_next_one_onto_its_tail() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    g.write(0, 0, "abcdefg");
    wrap(&g, 0);
    g.write(0, 1, "hi");
    unsafe { grid_reflow(&mut *g.ptr(), 5) };
    assert_eq!(g.hsize, 0);
    assert_eq!(g.hscrolled, 0);
    assert_eq!(lines(&g), [("abcde".into(), true), ("fghi".into(), false)]);
}

#[test]
fn a_split_keeps_the_wrap_flag_on_its_last_line() {
    let _guard = globals();
    let g = Grid::new(20, 2, 100);
    g.write(0, 0, "abcdefghij");
    wrap(&g, 0);
    g.write(0, 1, "klmnopqrst");
    unsafe { grid_reflow(&mut *g.ptr(), 4) };
    assert_eq!(
        lines(&g),
        [
            ("abcd".into(), true),
            ("efgh".into(), true),
            ("ijkl".into(), true),
            ("mnop".into(), true),
            ("qrst".into(), false)
        ]
    );
}

#[test]
fn dead_lines_are_skipped_and_the_screen_is_padded_back_out() {
    let _guard = globals();
    let g = Grid::new(5, 4, 0);
    g.write(0, 0, "ab");
    wrap(&g, 0);
    g.write(0, 1, "cd");
    wrap(&g, 1);
    g.write(0, 2, "ef");
    unsafe { grid_reflow(&mut *g.ptr(), 10) };
    assert_eq!(g.hsize, 0);
    assert_eq!(
        lines(&g),
        [
            ("abcdef".into(), false),
            ("".into(), false),
            ("".into(), false),
            ("".into(), false)
        ]
    );
}

#[test]
fn reflow_counts_the_width_of_wide_characters() {
    let _guard = globals();
    let g = Grid::new(10, 2, 100);
    let gc = wide("\u{4e2d}", 2);
    for px in 0..4 {
        unsafe { grid_set_cell(&mut *g.ptr(), px, 0, &gc) };
    }
    assert_eq!(g.line(0).flags & GRID_LINE_EXTENDED, GRID_LINE_EXTENDED);
    unsafe { grid_reflow(&mut *g.ptr(), 4) };
    assert_eq!(g.hsize, 1);
    assert_eq!(
        lines(&g),
        [
            ("\u{4e2d}\u{4e2d}".into(), true),
            ("\u{4e2d}\u{4e2d}".into(), false),
            ("".into(), false)
        ]
    );
}

#[test]
fn a_split_of_wide_characters_counts_out_every_line_it_needs() {
    let _guard = globals();
    let g = Grid::new(20, 1, 100);
    let gc = wide("\u{4e2d}", 2);
    for px in 0..6 {
        unsafe { grid_set_cell(&mut *g.ptr(), px, 0, &gc) };
    }
    unsafe { grid_reflow(&mut *g.ptr(), 4) };
    assert_eq!(g.hsize, 2);
    assert_eq!(
        lines(&g),
        [
            ("\u{4e2d}\u{4e2d}".into(), true),
            ("\u{4e2d}\u{4e2d}".into(), true),
            ("\u{4e2d}\u{4e2d}".into(), false)
        ]
    );
}

#[test]
fn a_join_stops_when_the_first_character_of_the_next_line_does_not_fit() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    g.write(0, 0, "abcd");
    wrap(&g, 0);
    let gc = wide("\u{4e2d}", 2);
    unsafe { grid_set_cell(&mut *g.ptr(), 0, 1, &gc) };
    unsafe { grid_reflow(&mut *g.ptr(), 5) };
    assert_eq!(
        lines(&g),
        [("abcd".into(), true), ("\u{4e2d}".into(), false)]
    );
}

#[test]
fn a_line_of_wide_characters_that_ends_exactly_on_the_width_is_moved() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    let gc = wide("\u{4e2d}", 2);
    for px in 0..2 {
        unsafe { grid_set_cell(&mut *g.ptr(), px, 0, &gc) };
    }
    unsafe { grid_reflow(&mut *g.ptr(), 4) };
    assert_eq!(lines(&g), [("\u{4e2d}\u{4e2d}".into(), false)]);
}

#[test]
fn reflow_pulls_the_scroll_position_back_over_the_lines_it_took() {
    let _guard = globals();
    let g = Grid::new(5, 1, 100);
    for text in ["abc", "de", "fg"] {
        g.write(0, g.hsize, text);
        unsafe { grid_scroll_history(&mut *g.ptr(), 8) };
    }
    wrap(&g, 0);
    assert_eq!((g.hsize, g.hscrolled), (3, 3));

    unsafe { grid_reflow(&mut *g.ptr(), 5) };
    assert_eq!(g.hsize, 2, "two lines became one");
    assert_eq!(g.hscrolled, 2);
    assert_eq!(
        lines(&g),
        [
            ("abcde".into(), false),
            ("fg".into(), false),
            ("".into(), false)
        ]
    );
}

#[test]
fn reflow_clamps_the_scroll_position_to_the_line_it_joined_into() {
    let _guard = globals();
    let g = Grid::new(5, 1, 100);
    for text in ["abc", "de", "fg"] {
        g.write(0, g.hsize, text);
        unsafe { grid_scroll_history(&mut *g.ptr(), 8) };
    }
    wrap(&g, 0);
    unsafe { (*g.ptr()).hscrolled = 1 };
    unsafe { grid_reflow(&mut *g.ptr(), 5) };
    assert_eq!(g.hsize, 2);
    assert_eq!(g.hscrolled, 0);
}

#[test]
fn padding_at_the_start_of_a_line_walks_off_the_front() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    unsafe { grid_set_padding(&mut *g.ptr(), 0, 0) };
    assert_eq!(unsafe { grid_in_set(&*g.ptr(), 0, 0, c" \t") }, 0);
}

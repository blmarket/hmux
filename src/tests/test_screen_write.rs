use super::*;
use crate::WindowPane;
use crate::fmt_args;
use crate::grid::grid_view_get_cell;
use crate::grid::{grid_get_line, grid_string_cells};
use crate::options::OptionsRef;

use crate::pane_identity::PaneIdentity;
use crate::screen::Screen as ScreenBoundary;
use crate::tests::test_fixtures::{Pane, Screen, Window, ascii, globals};
use crate::window::{
    screen_write_start_sync, screen_write_stop_sync, screen_write_sync_callback_for_test,
};
use ::core::ffi::c_int;

/// A screen with a writing context over it and no pane behind it, which is
/// what keeps the terminal out of the way: `tty_write` answers at once when
/// nothing has set a client callback, so every drawing call here only
/// touches the screen and its grid.
struct Writer {
    screen: Screen,
    state: screen_write_state,
}

impl Writer {
    fn new(sx: u_int, sy: u_int, hlimit: u_int) -> Writer {
        let mut w = Writer {
            screen: Screen::new(sx, sy, hlimit),
            state: screen_write_state::default(),
        };
        w.state = unsafe { screen_write_start(&mut w.screen) };
        w
    }

    fn ctx(&mut self) -> screen_write_ctx<'_> {
        screen_write_ctx::new(&mut self.state, &mut self.screen)
    }

    fn screen(&mut self) -> &mut RustScreen {
        &mut self.screen
    }

    fn grid(&self) -> &grid {
        self.screen.grid()
    }

    fn grid_mut(&mut self) -> &mut grid {
        self.screen.grid_mut()
    }

    /// Everything written so far, made visible: the collected text is
    /// flushed onto the grid first.
    fn flush(&mut self) {
        unsafe {
            screen_write_collect_end(&mut self.ctx());
            screen_write_collect_flush(&mut self.ctx(), 0, c"test");
        }
    }

    /// The visible screen, one string per line with trailing blanks cut.
    fn lines(&mut self) -> Vec<String> {
        self.flush();
        {
            let gd = self.grid();
            (0..(*gd).sy)
                .map(|y| {
                    let p = grid_string_cells(&*gd, 0, (*gd).hsize + y, (*gd).sx, None, 0, None);
                    p.to_string_lossy().trim_end().to_string()
                })
                .collect()
        }
    }

    /// The scrollback, oldest line first.
    fn history(&mut self) -> Vec<String> {
        self.flush();
        {
            let gd = self.grid();
            (0..(*gd).hsize)
                .map(|y| {
                    let p = grid_string_cells(&*gd, 0, y, (*gd).sx, None, 0, None);
                    p.to_string_lossy().trim_end().to_string()
                })
                .collect()
        }
    }

    /// One cell of the visible screen.
    fn cell(&mut self, px: u_int, py: u_int) -> grid_cell {
        self.flush();
        grid_view_get_cell(&*self.grid(), px, py)
    }

    /// The background colour each cell of a line carries, which is what a
    /// clear with a colour leaves behind.
    fn backgrounds(&mut self, py: u_int) -> Vec<c_int> {
        self.flush();
        {
            let gd = self.grid();
            (0..(*gd).sx)
                .map(|x| {
                    let gc = grid_view_get_cell(&*gd, x, py);
                    gc.bg
                })
                .collect()
        }
    }

    fn cursor(&mut self) -> (u_int, u_int) {
        self.screen.cursor()
    }

    fn move_to(&mut self, px: u_int, py: u_int) {
        unsafe { screen_write_cursormove(&mut self.ctx(), px as c_int, py as c_int, 0) };
    }

    /// Writes `text` one ASCII cell at a time, as the input parser would.
    fn puts(&mut self, text: &str) {
        for byte in text.bytes() {
            let gc = ascii(byte);
            unsafe { screen_write_cell(&mut self.ctx(), &gc) };
        }
    }

    /// Writes `text` through the collecting path, which is what the input
    /// parser uses for plain characters.
    fn collect(&mut self, text: &str) {
        for byte in text.bytes() {
            let gc = ascii(byte);
            unsafe { screen_write_collect_add(&mut self.ctx(), &gc) };
        }
    }

    /// The visible screen as it stands, without flushing what is still
    /// collected.
    fn peek(&mut self) -> Vec<String> {
        {
            let gd = self.grid();
            (0..(*gd).sy)
                .map(|y| {
                    let p = grid_string_cells(&*gd, 0, (*gd).hsize + y, (*gd).sx, None, 0, None);
                    p.to_string_lossy().trim_end().to_string()
                })
                .collect()
        }
    }

    /// Writes `text` at (px, py).
    fn write_at(&mut self, px: u_int, py: u_int, text: &str) {
        self.move_to(px, py);
        self.puts(text);
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        unsafe { screen_write_stop(&mut self.ctx()) };
    }
}

#[test]
fn a_writer_starts_at_the_top_left_of_an_empty_screen() {
    let _guard = globals();
    let mut w = Writer::new(10, 3, 100);
    assert_eq!(w.cursor(), (0, 0));
    assert_eq!(w.lines(), ["", "", ""]);
    assert!(!{ w.screen().write_list().is_empty() });
}

#[test]
fn text_is_written_where_the_cursor_is() {
    let _guard = globals();
    let mut w = Writer::new(10, 3, 100);
    w.puts("hello");
    assert_eq!(w.cursor(), (5, 0));
    w.write_at(2, 1, "abc");
    assert_eq!(w.lines(), ["hello", "  abc", ""]);
    assert_eq!(w.cell(0, 0).data.data[0], b'h');
}

#[test]
fn the_cursor_moves_and_stops_at_the_edges() {
    let _guard = globals();
    let mut w = Writer::new(10, 5, 100);
    w.move_to(4, 3);
    assert_eq!(w.cursor(), (4, 3));
    unsafe {
        screen_write_cursorup(&mut w.ctx(), 1);
        assert_eq!(w.cursor(), (4, 2));
        screen_write_cursorup(&mut w.ctx(), 99);
        assert_eq!(w.cursor(), (4, 0));
        screen_write_cursordown(&mut w.ctx(), 2);
        assert_eq!(w.cursor(), (4, 2));
        screen_write_cursordown(&mut w.ctx(), 99);
        assert_eq!(w.cursor(), (4, 4));
        screen_write_cursorleft(&mut w.ctx(), 2);
        assert_eq!(w.cursor(), (2, 4));
        screen_write_cursorleft(&mut w.ctx(), 99);
        assert_eq!(w.cursor(), (0, 4));
        screen_write_cursorright(&mut w.ctx(), 3);
        assert_eq!(w.cursor(), (3, 4));
        screen_write_cursorright(&mut w.ctx(), 99);
        assert_eq!(w.cursor(), (9, 4));
    }
}

/// Moving up or down stops at the scrolling region's edge when the cursor
/// is already inside it, and at the screen's edge when it is not.
#[test]
fn the_cursor_stops_at_the_scrolling_region() {
    let _guard = globals();
    let mut w = Writer::new(10, 6, 100);
    unsafe { screen_write_scrollregion(&mut w.ctx(), 2, 4) };
    w.move_to(0, 3);
    unsafe {
        screen_write_cursorup(&mut w.ctx(), 99);
        assert_eq!(w.cursor(), (0, 2));
        screen_write_cursordown(&mut w.ctx(), 99);
        assert_eq!(w.cursor(), (0, 4));
    }
    w.move_to(0, 5);
    unsafe {
        screen_write_cursorup(&mut w.ctx(), 99);
        assert_eq!(w.cursor(), (0, 2));
    }
    w.move_to(0, 1);
    unsafe {
        screen_write_cursorup(&mut w.ctx(), 99);
        assert_eq!(w.cursor(), (0, 0));
        screen_write_cursordown(&mut w.ctx(), 99);
        assert_eq!(w.cursor(), (0, 4));
    }
    w.move_to(0, 5);
    unsafe {
        screen_write_cursordown(&mut w.ctx(), 99);
        assert_eq!(w.cursor(), (0, 5));
    }
}

/// Backspacing off the front of a line only steps up to the line above
/// when that line was wrapped; otherwise the cursor stays where it is.
#[test]
fn backspace_steps_back_and_over_a_wrapped_line() {
    let _guard = globals();
    let mut w = Writer::new(10, 3, 100);
    w.move_to(3, 1);
    unsafe { screen_write_backspace(&mut w.ctx()) };
    assert_eq!(w.cursor(), (2, 1));
    w.move_to(0, 1);
    unsafe { screen_write_backspace(&mut w.ctx()) };
    assert_eq!(w.cursor(), (0, 1));
    unsafe {
        let gd = w.grid_mut();
        let hsize = gd.hsize;
        grid_get_line(gd, hsize).flags |= GRID_LINE_WRAPPED;
        screen_write_backspace(&mut w.ctx());
    }
    assert_eq!(w.cursor(), (9, 0));
    w.move_to(0, 0);
    unsafe { screen_write_backspace(&mut w.ctx()) };
    assert_eq!(w.cursor(), (0, 0));
}

#[test]
fn a_carriage_return_goes_to_the_start_of_the_line() {
    let _guard = globals();
    let mut w = Writer::new(10, 3, 100);
    w.write_at(4, 1, "ab");
    unsafe { screen_write_carriagereturn(&mut w.ctx()) };
    assert_eq!(w.cursor(), (0, 1));
}

/// Moving with `origin` set counts from the top of the scrolling region.
#[test]
fn the_cursor_may_be_moved_from_the_regions_own_top() {
    let _guard = globals();
    let mut w = Writer::new(10, 6, 100);
    unsafe {
        screen_write_scrollregion(&mut w.ctx(), 2, 4);
        screen_write_cursormove(&mut w.ctx(), 1, 1, 1);
        assert_eq!(w.cursor(), (1, 1));
        screen_write_mode_set(&mut w.ctx(), MODE_ORIGIN);
        screen_write_cursormove(&mut w.ctx(), 1, 1, 1);
        assert_eq!(w.cursor(), (1, 3));
        screen_write_cursormove(&mut w.ctx(), 1, 99, 1);
        assert_eq!(w.cursor(), (1, 4));
        screen_write_mode_clear(&mut w.ctx(), MODE_ORIGIN);
        screen_write_cursormove(&mut w.ctx(), -1, -1, 0);
        assert_eq!(w.cursor(), (1, 4));
        screen_write_cursormove(&mut w.ctx(), 99, 99, 0);
        assert_eq!(w.cursor(), (9, 5));
    }
}

/// A region of one line is refused, as is one that does not fit.
#[test]
fn a_scrolling_region_must_be_more_than_one_line_and_fit() {
    let _guard = globals();
    let mut w = Writer::new(10, 6, 100);
    unsafe {
        screen_write_scrollregion(&mut w.ctx(), 3, 3);
        assert_eq!(w.screen().region(), (0, 5));
        screen_write_scrollregion(&mut w.ctx(), 99, 1);
        assert_eq!(w.screen().region(), (0, 5));
        screen_write_scrollregion(&mut w.ctx(), 1, 99);
        assert_eq!(w.screen().region(), (1, 5));
        screen_write_scrollregion(&mut w.ctx(), 1, 4);
        assert_eq!(w.screen().region(), (1, 4));
    }
}

#[test]
fn a_mode_is_set_and_cleared() {
    let _guard = globals();
    let mut w = Writer::new(10, 3, 100);
    unsafe {
        let before = w.screen().mode();
        screen_write_mode_set(&mut w.ctx(), MODE_INSERT);
        assert_eq!(w.screen().mode(), before | MODE_INSERT);
        screen_write_mode_clear(&mut w.ctx(), MODE_INSERT);
        assert_eq!(w.screen().mode(), before);
    }
}

#[test]
fn a_line_is_cleared_whole_or_from_the_cursor() {
    let _guard = globals();
    let mut w = Writer::new(6, 3, 100);
    w.write_at(0, 0, "abcdef");
    w.write_at(0, 1, "abcdef");
    w.write_at(0, 2, "abcdef");
    w.move_to(3, 0);
    unsafe { screen_write_clearendofline(&mut w.ctx(), 8) };
    w.move_to(3, 1);
    unsafe { screen_write_clearstartofline(&mut w.ctx(), 8) };
    w.move_to(3, 2);
    unsafe { screen_write_clearline(&mut w.ctx(), 8) };
    assert_eq!(w.lines(), ["abc", "    ef", ""]);
}

#[test]
fn a_clear_keeps_the_colour_it_was_given() {
    let _guard = globals();
    let mut w = Writer::new(4, 2, 100);
    w.write_at(0, 0, "abcd");
    unsafe { screen_write_clearline(&mut w.ctx(), 2) };
    assert_eq!(w.backgrounds(0), [2, 2, 2, 2]);
}

#[test]
fn characters_are_inserted_and_deleted_on_a_line() {
    let _guard = globals();
    let mut w = Writer::new(6, 2, 100);
    w.write_at(0, 0, "abcdef");
    w.move_to(2, 0);
    unsafe { screen_write_insertcharacter(&mut w.ctx(), 2, 8) };
    assert_eq!(w.lines()[0], "ab  cd");
    w.move_to(2, 0);
    unsafe { screen_write_deletecharacter(&mut w.ctx(), 2, 8) };
    assert_eq!(w.lines()[0], "abcd");
    w.move_to(1, 0);
    unsafe { screen_write_clearcharacter(&mut w.ctx(), 2, 8) };
    assert_eq!(w.lines()[0], "a  d");
}

/// A count bigger than what is left of the line is cut down to it, and a
/// count of nothing is read as one.
#[test]
fn an_insert_or_delete_count_is_cut_down_to_the_line() {
    let _guard = globals();
    let mut w = Writer::new(6, 2, 100);
    w.write_at(0, 0, "abcdef");
    w.move_to(4, 0);
    unsafe { screen_write_deletecharacter(&mut w.ctx(), 99, 8) };
    assert_eq!(w.lines()[0], "abcd");
    w.write_at(0, 0, "abcdef");
    w.move_to(4, 0);
    unsafe { screen_write_insertcharacter(&mut w.ctx(), 0, 8) };
    assert_eq!(w.lines()[0], "abcd e");
    w.write_at(0, 0, "abcdef");
    w.move_to(4, 0);
    unsafe { screen_write_clearcharacter(&mut w.ctx(), 99, 8) };
    assert_eq!(w.lines()[0], "abcd");
}

/// A cursor outside the scrolling region moves the whole screen rather
/// than the region, and the count is cut down to what is left below the
/// cursor instead of to the region's end. Characters are not tested
/// against the region at all.
///
/// Inserting as many lines as there are left below the cursor does
/// nothing at all: there is then nothing to move, and the lines that
/// should have been left blank are never cleared. Deleting that many does
/// clear them, because the delete clears what it moved off by itself.
#[test]
fn lines_moved_from_outside_the_region_move_the_whole_screen() {
    let _guard = globals();
    let mut w = Writer::new(6, 4, 100);
    let fill = |w: &mut Writer| {
        for y in 0..4 {
            w.write_at(0, y, &format!("l{y}"));
        }
    };
    fill(&mut w);
    unsafe { screen_write_scrollregion(&mut w.ctx(), 1, 2) };
    w.move_to(2, 0);
    unsafe {
        screen_write_insertline(&mut w.ctx(), 1, 8);
        assert_eq!(w.lines(), ["", "l0", "l1", "l2"]);
        screen_write_deleteline(&mut w.ctx(), 1, 8);
        assert_eq!(w.lines(), ["l0", "l1", "l2", ""]);
        screen_write_insertline(&mut w.ctx(), 99, 8);
        assert_eq!(w.lines(), ["l0", "l1", "l2", ""]);
        screen_write_deleteline(&mut w.ctx(), 99, 8);
        assert_eq!(w.lines(), ["", "", "", ""]);
        w.write_at(2, 0, "abcd");
        w.move_to(2, 0);
        screen_write_insertcharacter(&mut w.ctx(), 2, 8);
        assert_eq!(w.lines()[0], "    ab");
        screen_write_deletecharacter(&mut w.ctx(), 2, 8);
        assert_eq!(w.lines()[0], "  ab");
    }
    w.move_to(0, 3);
    unsafe {
        screen_write_insertline(&mut w.ctx(), 0, 8);
        assert_eq!(w.lines()[3], "");
        screen_write_deleteline(&mut w.ctx(), 0, 8);
        assert_eq!(w.lines()[3], "");
    }
}

#[test]
fn lines_are_inserted_and_deleted() {
    let _guard = globals();
    let mut w = Writer::new(4, 4, 100);
    for y in 0..4 {
        w.write_at(0, y, &format!("l{y}"));
    }
    w.move_to(0, 1);
    unsafe { screen_write_insertline(&mut w.ctx(), 1, 8) };
    assert_eq!(w.lines(), ["l0", "", "l1", "l2"]);
    w.move_to(0, 1);
    unsafe { screen_write_deleteline(&mut w.ctx(), 1, 8) };
    assert_eq!(w.lines(), ["l0", "l1", "l2", ""]);
}

/// Inserting or deleting lines inside a region only moves that region.
#[test]
fn lines_are_inserted_and_deleted_inside_a_region() {
    let _guard = globals();
    let mut w = Writer::new(4, 5, 100);
    for y in 0..5 {
        w.write_at(0, y, &format!("l{y}"));
    }
    unsafe { screen_write_scrollregion(&mut w.ctx(), 1, 3) };
    w.move_to(0, 1);
    unsafe { screen_write_insertline(&mut w.ctx(), 1, 8) };
    assert_eq!(w.lines(), ["l0", "", "l1", "l2", "l4"]);
    w.move_to(0, 1);
    unsafe { screen_write_deleteline(&mut w.ctx(), 99, 8) };
    assert_eq!(w.lines(), ["l0", "", "", "", "l4"]);
}

#[test]
fn the_screen_is_cleared_whole_or_from_the_cursor() {
    let _guard = globals();
    let mut w = Writer::new(4, 3, 100);
    let fill = |w: &mut Writer| {
        for y in 0..3 {
            w.write_at(0, y, "abcd");
        }
    };
    fill(&mut w);
    w.move_to(2, 1);
    unsafe { screen_write_clearendofscreen(&mut w.ctx(), 8) };
    assert_eq!(w.lines(), ["abcd", "ab", ""]);
    fill(&mut w);
    w.move_to(2, 1);
    unsafe { screen_write_clearstartofscreen(&mut w.ctx(), 8) };
    assert_eq!(w.lines(), ["", "   d", "abcd"]);
    fill(&mut w);
    unsafe { screen_write_clearscreen(&mut w.ctx(), 8) };
    assert_eq!(w.lines(), ["", "", ""]);
}

#[test]
fn a_line_feed_scrolls_the_screen_and_keeps_the_history() {
    let _guard = globals();
    let mut w = Writer::new(4, 2, 100);
    w.write_at(0, 0, "aa");
    w.write_at(0, 1, "bb");
    w.move_to(0, 1);
    unsafe { screen_write_linefeed(&mut w.ctx(), 0, 8) };
    assert_eq!(w.lines(), ["bb", ""]);
    assert_eq!(w.history(), ["aa"]);
}

#[test]
fn scrolling_up_and_down_moves_the_lines() {
    let _guard = globals();
    let mut w = Writer::new(4, 4, 100);
    for y in 0..4 {
        w.write_at(0, y, &format!("l{y}"));
    }
    unsafe { screen_write_scrollup(&mut w.ctx(), 2, 8) };
    assert_eq!(w.lines(), ["l2", "l3", "", ""]);
    assert_eq!(w.history(), ["l0", "l1"]);
    unsafe { screen_write_scrolldown(&mut w.ctx(), 1, 8) };
    assert_eq!(w.lines(), ["", "l2", "l3", ""]);
}

#[test]
fn a_reverse_index_moves_up_and_scrolls_at_the_top() {
    let _guard = globals();
    let mut w = Writer::new(4, 3, 100);
    for y in 0..3 {
        w.write_at(0, y, &format!("l{y}"));
    }
    w.move_to(0, 1);
    unsafe { screen_write_reverseindex(&mut w.ctx(), 8) };
    assert_eq!(w.cursor(), (0, 0));
    assert_eq!(w.lines(), ["l0", "l1", "l2"]);
    unsafe { screen_write_reverseindex(&mut w.ctx(), 8) };
    assert_eq!(w.lines(), ["", "l0", "l1"]);
}

#[test]
fn the_history_is_thrown_away_when_asked() {
    let _guard = globals();
    let mut w = Writer::new(4, 2, 100);
    w.write_at(0, 0, "aa");
    w.move_to(0, 1);
    unsafe { screen_write_linefeed(&mut w.ctx(), 0, 8) };
    assert_eq!(w.history(), ["aa"]);
    unsafe { screen_write_clearhistory(&mut w.ctx()) };
    assert!(w.history().is_empty());
}

#[test]
fn the_alignment_test_fills_the_screen_with_letters() {
    let _guard = globals();
    let mut w = Writer::new(4, 2, 100);
    unsafe { screen_write_alignmenttest(&mut w.ctx()) };
    assert_eq!(w.lines(), ["EEEE", "EEEE"]);
    assert_eq!(w.cursor(), (0, 0));
}

#[test]
fn a_reset_clears_the_screen_and_the_region() {
    let _guard = globals();
    let mut w = Writer::new(4, 3, 100);
    w.write_at(0, 0, "abcd");
    unsafe {
        screen_write_scrollregion(&mut w.ctx(), 1, 2);
        screen_write_reset(&mut w.ctx());
        assert_eq!(w.screen().region(), (0, 2));
        assert_eq!(w.screen().mode() & MODE_CURSOR, MODE_CURSOR);
    }
    assert_eq!(w.lines(), ["", "", ""]);
    assert_eq!(w.cursor(), (0, 0));
}

/// One cell holding the UTF-8 character `text`, as the input parser builds
/// one.
fn utf8(text: &str) -> grid_cell {
    let mut gc = { grid_default_cell };
    let bytes = text.as_bytes();
    assert!(bytes.len() > 1, "a one-byte character is not opened");
    unsafe {
        assert_eq!(utf8_open(&mut gc.data, bytes[0]), UTF8_MORE);
        for byte in &bytes[1..] {
            utf8_append(&mut gc.data, *byte);
        }
    }
    gc
}

#[test]
fn the_width_of_a_string_counts_what_would_be_drawn() {
    let _guard = globals();
    unsafe {
        assert_eq!(screen_write_strlen(c"%s", fmt_args![c"abc".as_ptr()]), 3);
        assert_eq!(screen_write_strlen(c"%s", fmt_args![c"a\tb".as_ptr()]), 3);
        assert_eq!(screen_write_strlen(c"%s", fmt_args![c"a\x07b".as_ptr()]), 2);
        assert_eq!(
            screen_write_strlen(c"%s", fmt_args![c"a\u{4e00}b".as_ptr()]),
            4
        );
        assert_eq!(
            screen_write_strlen(c"%s", fmt_args![c"a\u{e9}b".as_ptr()]),
            3
        );
        assert_eq!(
            screen_write_strlen(c"%s", fmt_args![c"a\xe4\xb8".as_ptr()]),
            1
        );
        assert_eq!(
            screen_write_strlen(c"%s", fmt_args![c"a\xff\xffb".as_ptr()]),
            2
        );
    }
}

#[test]
fn a_formatted_string_is_written_cell_by_cell() {
    let _guard = globals();
    let mut w = Writer::new(10, 3, 100);
    let gc = { grid_default_cell };
    unsafe {
        screen_write_puts(&mut w.ctx(), &gc, c"%s-%d", fmt_args![c"ab".as_ptr(), 7]);
    }
    assert_eq!(w.lines()[0], "ab-7");
}

/// A newline in the text feeds the line and returns the cursor, a byte of
/// one turns the character set over, and anything else below a space is
/// left out.
#[test]
fn a_written_string_reads_its_own_control_bytes() {
    let _guard = globals();
    let mut w = Writer::new(10, 3, 100);
    let gc = { grid_default_cell };
    unsafe {
        screen_write_puts(&mut w.ctx(), &gc, c"%s", fmt_args![c"ab\ncd".as_ptr()]);
    }
    assert_eq!(w.lines(), ["ab", "cd", ""]);
    let mut w = Writer::new(10, 3, 100);
    unsafe {
        screen_write_puts(
            &mut w.ctx(),
            &gc,
            c"%s",
            fmt_args![c"a\x01b\x01c\x07d".as_ptr()],
        );
    }
    assert_eq!(w.lines()[0], "abcd");
    assert_eq!(w.cell(0, 0).attr as c_int & GRID_ATTR_CHARSET, 0);
    assert_eq!(
        w.cell(1, 0).attr as c_int & GRID_ATTR_CHARSET,
        GRID_ATTR_CHARSET
    );
    assert_eq!(w.cell(2, 0).attr as c_int & GRID_ATTR_CHARSET, 0);
}

#[test]
fn a_string_may_be_cut_to_a_width() {
    let _guard = globals();
    let mut w = Writer::new(10, 2, 100);
    let gc = { grid_default_cell };
    unsafe {
        screen_write_nputs(&mut w.ctx(), 3, &gc, c"%s", fmt_args![c"abcdef".as_ptr()]);
    }
    assert_eq!(w.lines()[0], "abc");
    let mut w = Writer::new(10, 2, 100);
    unsafe {
        screen_write_nputs(
            &mut w.ctx(),
            3,
            &gc,
            c"%s",
            fmt_args![c"ab\u{4e00}c".as_ptr()],
        );
    }
    assert_eq!(w.lines()[0], "ab");
    let mut w = Writer::new(10, 2, 100);
    unsafe {
        screen_write_nputs(
            &mut w.ctx(),
            -1,
            &gc,
            c"%s",
            fmt_args![c"a\xe4\xb8".as_ptr()],
        );
    }
    assert_eq!(w.lines()[0], "a");
    let mut w = Writer::new(10, 2, 100);
    unsafe {
        screen_write_nputs(
            &mut w.ctx(),
            -1,
            &gc,
            c"%s",
            fmt_args![c"a\xff\xffb".as_ptr()],
        );
    }
    assert_eq!(w.lines()[0], "ab");
}

/// Text is wrapped on the last space that fits, and answers whether it all
/// went in.
#[test]
fn text_is_wrapped_over_the_lines_it_is_given() {
    let _guard = globals();
    let gc = { grid_default_cell };
    let mut w = Writer::new(12, 4, 100);
    let all = unsafe {
        screen_write_text(
            &mut w.ctx(),
            0,
            6,
            3,
            0,
            &gc,
            c"%s",
            fmt_args![c"one two three".as_ptr()],
        )
    };
    assert_eq!(w.lines(), ["one", "two", "three", ""]);
    assert_eq!(all, 0);
    let mut w = Writer::new(12, 4, 100);
    let all = unsafe {
        screen_write_text(
            &mut w.ctx(),
            0,
            6,
            4,
            0,
            &gc,
            c"%s",
            fmt_args![c"one two".as_ptr()],
        )
    };
    assert_eq!(w.lines(), ["one", "two", "", ""]);
    assert_eq!(all, 1);
    let mut w = Writer::new(12, 4, 100);
    let all = unsafe {
        screen_write_text(
            &mut w.ctx(),
            0,
            6,
            2,
            0,
            &gc,
            c"%s",
            fmt_args![c"one two three".as_ptr()],
        )
    };
    assert_eq!(w.lines(), ["one", "two", "", ""]);
    assert_eq!(all, 0);
    let mut w = Writer::new(12, 4, 100);
    unsafe {
        screen_write_text(
            &mut w.ctx(),
            0,
            6,
            3,
            0,
            &gc,
            c"%s",
            fmt_args![c"a\nbb\nccc".as_ptr()],
        );
    }
    assert_eq!(w.lines(), ["a", "bb", "ccc", ""]);
    let mut w = Writer::new(12, 4, 100);
    unsafe {
        screen_write_text(
            &mut w.ctx(),
            0,
            4,
            3,
            0,
            &gc,
            c"%s",
            fmt_args![c"abcdefgh".as_ptr()],
        );
    }
    assert_eq!(w.lines(), ["abcd", "efgh", "", ""]);
}

/// With `more` set the cursor is only moved on once the line is full.
#[test]
fn text_that_carries_on_leaves_the_cursor_where_it_stopped() {
    let _guard = globals();
    let gc = { grid_default_cell };
    let mut w = Writer::new(12, 4, 100);
    let all = unsafe {
        screen_write_text(
            &mut w.ctx(),
            0,
            6,
            3,
            1,
            &gc,
            c"%s",
            fmt_args![c"ab".as_ptr()],
        )
    };
    assert_eq!(all, 1);
    assert_eq!(w.cursor(), (2, 0));
    let all = unsafe {
        screen_write_text(
            &mut w.ctx(),
            0,
            6,
            3,
            1,
            &gc,
            c"%s",
            fmt_args![c"cdef".as_ptr()],
        )
    };
    assert_eq!(all, 1);
    assert_eq!(w.cursor(), (0, 1));
    assert_eq!(w.lines()[0], "abcdef");
}

#[test]
fn a_screen_is_copied_from_another_one() {
    let _guard = globals();
    let mut src = Writer::new(6, 3, 100);
    src.write_at(0, 0, "abcdef");
    src.write_at(0, 1, "ghijkl");
    src.flush();
    let mut w = Writer::new(6, 3, 100);
    w.move_to(1, 1);
    unsafe { screen_write_fast_copy(&mut w.ctx(), src.screen(), 0, 0, 3, 2) };
    assert_eq!(w.lines(), ["", " abc", " ghi"]);
}

/// A copy that runs off the edge of either screen stops there.
#[test]
fn a_copy_stops_at_the_edges() {
    let _guard = globals();
    let mut src = Writer::new(4, 2, 100);
    src.write_at(0, 0, "abcd");
    src.flush();
    let mut w = Writer::new(4, 2, 100);
    w.move_to(2, 0);
    unsafe { screen_write_fast_copy(&mut w.ctx(), src.screen(), 0, 0, 4, 4) };
    assert_eq!(w.lines(), ["  ab", ""]);
}

#[test]
fn a_horizontal_line_is_drawn_with_its_ends() {
    let _guard = globals();
    let mut w = Writer::new(6, 2, 100);
    unsafe { screen_write_hline(&mut w.ctx(), 4, 1, 1, BOX_LINES_SIMPLE, None) };
    assert_eq!(w.lines()[0], "+--+");
    assert_eq!(w.cursor(), (0, 0));
    let mut w = Writer::new(6, 2, 100);
    unsafe { screen_write_hline(&mut w.ctx(), 4, 0, 0, BOX_LINES_SIMPLE, None) };
    assert_eq!(w.lines()[0], "----");
}

#[test]
fn a_vertical_line_is_drawn_with_its_ends() {
    let _guard = globals();
    let mut w = Writer::new(4, 4, 100);
    unsafe { screen_write_vline(&mut w.ctx(), 4, 1, 1) };
    assert_eq!(w.lines(), ["w", "x", "x", "v"]);
    assert_eq!(
        w.cell(0, 1).attr as c_int & GRID_ATTR_CHARSET,
        GRID_ATTR_CHARSET
    );
    assert_eq!(w.cursor(), (0, 0));
}

#[test]
fn a_box_is_drawn_round_the_cursor() {
    let _guard = globals();
    let mut w = Writer::new(6, 4, 100);
    unsafe { screen_write_box(&mut w.ctx(), 6, 3, BOX_LINES_SIMPLE, None, None) };
    assert_eq!(w.lines(), ["+----+", "|    |", "+----+", ""]);
    let mut w = Writer::new(8, 4, 100);
    unsafe {
        screen_write_box(&mut w.ctx(), 8, 3, BOX_LINES_SIMPLE, None, Some(c"hi"));
    }
    assert_eq!(w.lines()[0], "+-hi---+");
}

#[test]
fn a_preview_shows_the_top_left_of_another_screen() {
    let _guard = globals();
    let mut src = Writer::new(6, 4, 100);
    for y in 0..4 {
        src.write_at(0, y, &format!("l{y}"));
    }
    src.flush();
    let mut w = Writer::new(6, 4, 100);
    unsafe { screen_write_preview(&mut w.ctx(), src.screen(), 4, 2) };
    assert_eq!(w.lines(), ["2", "3", "", ""]);
    let mut w = Writer::new(6, 4, 100);
    unsafe {
        let mode = src.screen().mode();
        src.screen().set_mode(mode & !MODE_CURSOR);
        screen_write_preview(&mut w.ctx(), src.screen(), 4, 2);
        let mode = src.screen().mode();
        src.screen().set_mode(mode | MODE_CURSOR);
    }
    assert_eq!(w.lines(), ["l0", "l1", "", ""]);
    let mut w = Writer::new(6, 4, 100);
    unsafe { screen_write_preview(&mut w.ctx(), src.screen(), 9, 9) };
    assert_eq!(w.lines(), ["l0", "l1", "l2", "l3"]);
}

#[test]
fn the_selection_and_a_raw_string_are_handed_to_the_terminal() {
    let _guard = globals();
    let mut w = Writer::new(6, 2, 100);
    unsafe {
        screen_write_setselection(&mut w.ctx(), c"clipboard", b"hi");
        screen_write_rawstring(&mut w.ctx(), b"hi", 0);
        screen_write_rawstring(&mut w.ctx(), b"hi", 1);
    }
    assert_eq!(w.lines(), ["", ""]);
}

#[test]
fn the_alternate_screen_is_swapped_in_and_out() {
    let _guard = globals();
    let mut w = Writer::new(4, 2, 100);
    w.write_at(0, 0, "main");
    w.flush();
    let mut gc = { grid_default_cell };
    unsafe { screen_write_alternateon(&mut w.ctx(), &mut gc, 1) };
    assert_eq!(w.lines(), ["", ""]);
    w.write_at(0, 0, "alt");
    unsafe { screen_write_alternateoff(&mut w.ctx(), &mut gc, 1) };
    assert_eq!(w.lines(), ["main", ""]);
}

/// Turning the alternate screen off when it was never on does nothing.
#[test]
fn the_alternate_screen_is_only_swapped_out_when_it_was_in() {
    let _guard = globals();
    let mut w = Writer::new(4, 2, 100);
    w.write_at(0, 0, "main");
    let mut gc = { grid_default_cell };
    unsafe { screen_write_alternateoff(&mut w.ctx(), &mut gc, 0) };
    assert_eq!(w.lines(), ["main", ""]);
}

#[test]
fn a_wide_character_takes_two_cells_and_leaves_padding() {
    let _guard = globals();
    let mut w = Writer::new(6, 2, 100);
    let gc = utf8("\u{4e00}");
    unsafe { screen_write_cell(&mut w.ctx(), &gc) };
    assert_eq!(w.cursor(), (2, 0));
    assert_eq!(w.cell(0, 0).data.size, 3);
    assert_eq!(
        w.cell(1, 0).flags as c_int & GRID_FLAG_PADDING,
        GRID_FLAG_PADDING
    );
    assert_eq!(w.lines()[0], "\u{4e00}");
}

/// A wide character with only one column left wraps to the next line.
#[test]
fn a_wide_character_wraps_rather_than_being_cut() {
    let _guard = globals();
    let mut w = Writer::new(3, 2, 100);
    w.puts("ab");
    let gc = utf8("\u{4e00}");
    unsafe { screen_write_cell(&mut w.ctx(), &gc) };
    assert_eq!(w.lines(), ["ab", "\u{4e00}"]);
}

/// Writing over the left half of a wide character clears the right half
/// too.
#[test]
fn writing_over_a_wide_character_clears_both_of_its_cells() {
    let _guard = globals();
    let mut w = Writer::new(6, 2, 100);
    let wide = utf8("\u{4e00}");
    unsafe { screen_write_cell(&mut w.ctx(), &wide) };
    w.flush();
    w.move_to(0, 0);
    w.puts("x");
    assert_eq!(w.lines()[0], "x");
    let mut w = Writer::new(6, 2, 100);
    unsafe { screen_write_cell(&mut w.ctx(), &wide) };
    w.flush();
    w.move_to(1, 0);
    w.puts("x");
    assert_eq!(w.lines()[0], " x");
}

/// A combining character joins the cell in front of it rather than taking
/// one of its own.
#[test]
fn a_combining_character_joins_the_cell_in_front_of_it() {
    let _guard = globals();
    let mut w = Writer::new(6, 2, 100);
    w.puts("e");
    let gc = utf8("\u{301}");
    unsafe { screen_write_cell(&mut w.ctx(), &gc) };
    assert_eq!(w.cursor(), (1, 0));
    assert_eq!(w.lines()[0], "e\u{301}");
}

#[test]
fn a_line_wraps_when_it_is_full() {
    let _guard = globals();
    let mut w = Writer::new(3, 3, 100);
    w.puts("abcdef");
    assert_eq!(w.lines(), ["abc", "def", ""]);
    assert_eq!(w.cursor(), (3, 1));
}

/// With wrapping turned off the last cell of the line is written over
/// again.
#[test]
fn a_line_that_may_not_wrap_keeps_writing_the_last_cell() {
    let _guard = globals();
    let mut w = Writer::new(3, 2, 100);
    unsafe { screen_write_mode_clear(&mut w.ctx(), MODE_WRAP) };
    w.puts("abcde");
    assert_eq!(w.lines(), ["abe", ""]);
}

/// In insert mode each cell pushes the rest of the line along.
#[test]
fn insert_mode_pushes_the_line_along() {
    let _guard = globals();
    let mut w = Writer::new(6, 2, 100);
    w.puts("abcd");
    w.flush();
    w.move_to(1, 0);
    unsafe { screen_write_mode_set(&mut w.ctx(), MODE_INSERT) };
    w.puts("x");
    assert_eq!(w.lines()[0], "axbcd");
}

#[test]
fn a_tab_is_kept_as_a_tab() {
    let _guard = globals();
    let mut w = Writer::new(10, 2, 100);
    let mut gc = ascii(b' ');
    gc.flags = (gc.flags as c_int | GRID_FLAG_TAB) as u_char;
    unsafe { screen_write_cell(&mut w.ctx(), &gc) };
    assert_eq!(w.cell(0, 0).flags as c_int & GRID_FLAG_TAB, GRID_FLAG_TAB);
}

#[test]
fn collected_text_is_only_written_when_it_is_flushed() {
    let _guard = globals();
    let mut w = Writer::new(10, 2, 100);
    w.collect("hello");
    assert_eq!(w.peek(), ["", ""]);
    assert_eq!(w.cursor(), (0, 0));
    assert_eq!(w.lines(), ["hello", ""]);
    assert_eq!(w.cursor(), (5, 0));
}

#[test]
fn collected_text_wraps_at_the_end_of_the_line() {
    let _guard = globals();
    let mut w = Writer::new(4, 3, 100);
    w.collect("abcdefghi");
    assert_eq!(w.lines(), ["abcd", "efgh", "i"]);
}

/// Text collected over text that is already waiting takes its place.
#[test]
fn collected_text_written_over_is_trimmed() {
    let _guard = globals();
    let mut w = Writer::new(8, 2, 100);
    w.collect("abcdef");
    unsafe { screen_write_collect_end(&mut w.ctx()) };
    w.move_to(2, 0);
    w.collect("XY");
    assert_eq!(w.lines()[0], "abXYef");
    let mut w = Writer::new(8, 2, 100);
    w.collect("abcdef");
    unsafe { screen_write_collect_end(&mut w.ctx()) };
    w.move_to(0, 0);
    w.collect("XYZWVU");
    assert_eq!(w.lines()[0], "XYZWVU");
    let mut w = Writer::new(8, 2, 100);
    w.collect("cd");
    unsafe { screen_write_collect_end(&mut w.ctx()) };
    w.move_to(4, 0);
    w.collect("gh");
    unsafe { screen_write_collect_end(&mut w.ctx()) };
    w.move_to(1, 0);
    w.collect("XYZWV");
    assert_eq!(w.lines()[0], "cXYZWV");
}

/// A cell the collector cannot hold — anything but a single printable
/// byte in the default character set, or any cell at all while the screen
/// is inserting, not wrapping or carrying a selection — is written
/// straight through instead.
#[test]
fn a_cell_the_collector_cannot_hold_goes_straight_through() {
    let _guard = globals();
    let mut w = Writer::new(8, 2, 100);
    let wide = utf8("\u{4e00}");
    w.collect("ab");
    unsafe { screen_write_collect_add(&mut w.ctx(), &wide) };
    assert_eq!(w.peek()[0], "ab\u{4e00}");

    let mut w = Writer::new(8, 2, 100);
    let mut gc = ascii(b'x');
    gc.attr = (gc.attr as c_int | GRID_ATTR_CHARSET) as u_short;
    unsafe { screen_write_collect_add(&mut w.ctx(), &gc) };
    assert_eq!(w.peek()[0], "x");

    let mut w = Writer::new(8, 2, 100);
    let mut gc = ascii(b' ');
    gc.flags = (gc.flags as c_int | GRID_FLAG_TAB) as u_char;
    unsafe { screen_write_collect_add(&mut w.ctx(), &gc) };
    assert_eq!(w.cursor(), (1, 0));

    let mut w = Writer::new(8, 2, 100);
    unsafe { screen_write_mode_clear(&mut w.ctx(), MODE_WRAP) };
    w.collect("ab");
    assert_eq!(w.peek()[0], "ab");

    let mut w = Writer::new(8, 2, 100);
    unsafe { screen_write_mode_set(&mut w.ctx(), MODE_INSERT) };
    w.collect("ab");
    assert_eq!(w.peek()[0], "ab");

    let mut w = Writer::new(8, 2, 100);
    let gc = { grid_default_cell };
    {
        w.screen().set_selection(0, 0, 2, 0, false, 0, 0, &gc);
    }
    w.collect("ab");
    assert_eq!(w.peek()[0], "ab");
    assert_eq!(
        w.cell(0, 0).flags as c_int & GRID_FLAG_SELECTED,
        GRID_FLAG_SELECTED
    );
    w.screen().clear_selection();
    w.move_to(0, 0);
    w.puts("cd");
    assert_eq!(w.cell(0, 0).flags as c_int & GRID_FLAG_SELECTED, 0);
}

#[test]
fn collected_text_survives_a_scroll() {
    let _guard = globals();
    let mut w = Writer::new(4, 2, 100);
    w.collect("ab");
    unsafe { screen_write_collect_end(&mut w.ctx()) };
    w.move_to(0, 1);
    w.collect("cd");
    unsafe { screen_write_collect_end(&mut w.ctx()) };
    w.move_to(0, 1);
    unsafe { screen_write_linefeed(&mut w.ctx(), 0, 8) };
    w.collect("ef");
    assert_eq!(w.lines(), ["cd", "ef"]);
    assert_eq!(w.history(), ["ab"]);
}

#[test]
fn a_box_may_be_drawn_in_any_of_its_line_styles() {
    let _guard = globals();
    let styles = [
        BOX_LINES_DEFAULT,
        BOX_LINES_SINGLE,
        BOX_LINES_DOUBLE,
        BOX_LINES_HEAVY,
        BOX_LINES_SIMPLE,
        BOX_LINES_ROUNDED,
        BOX_LINES_PADDED,
        BOX_LINES_NONE,
    ];
    let mut drawn = Vec::new();
    for lines in styles {
        let mut w = Writer::new(4, 3, 100);
        unsafe { screen_write_box(&mut w.ctx(), 4, 3, lines, None, None) };
        drawn.push(w.lines()[0].clone());
    }
    assert_eq!(
        drawn,
        [
            "lqqk",
            "lqqk",
            "\u{2554}\u{2550}\u{2550}\u{2557}",
            "\u{250f}\u{2501}\u{2501}\u{2513}",
            "+--+",
            "\u{256d}\u{2500}\u{2500}\u{256e}",
            "",
            ""
        ]
    );
}

#[test]
fn a_menu_is_drawn_with_a_border_and_a_choice() {
    let _guard = globals();
    let entry = |name: Option<&CStr>| menu_entry {
        name: name.map(CStr::to_owned),
        key: 0,
        command: None,
    };
    let m = menu {
        title: Some(c"title".to_owned()),
        items: vec![
            entry(Some(c"one")),
            entry(Some(c"-two")),
            entry(None),
            entry(Some(c"three")),
        ],
        width: 7,
    };
    let mut w = Writer::new(12, 8, 100);
    let gc = { grid_default_cell };
    unsafe {
        screen_write_menu(&mut w.ctx(), &m, 0, BOX_LINES_SIMPLE, &gc, &gc, &gc);
    }
    assert_eq!(
        w.lines(),
        [
            "+-title---+",
            "| one     |",
            "| two     |",
            "+---------+",
            "| three   |",
            "+---------+",
            "",
            ""
        ]
    );
    assert_eq!(w.cursor(), (0, 0));
}

/// The guards in front of the calls that build a debug message first only
/// run at a raised log level; the log file is never opened here, so
/// nothing is written.
#[test]
fn the_debug_paths_run_at_a_raised_log_level() {
    let _guard = globals();
    crate::log::log_with_level(1, || {
        let mut w = Writer::new(6, 3, 100);
        unsafe {
            screen_write_mode_set(&mut w.ctx(), MODE_INSERT);
            screen_write_mode_clear(&mut w.ctx(), MODE_INSERT);
        }
        w.collect("abcdefgh");
        w.puts("ij");
        unsafe {
            screen_write_clearline(&mut w.ctx(), 8);
            screen_write_clearendofscreen(&mut w.ctx(), 8);
            screen_write_linefeed(&mut w.ctx(), 0, 8);
        }
        assert_eq!(w.lines(), ["abcdef", "", ""]);
    });
}

/// A writer over a pane's own screen, which is what the input parser uses.
/// The pane and its window are the server-free fixtures and there are no
/// clients, so `tty_write` still answers at once; what does run is the
/// pane-facing half of the drawing code — the obscured check, the window's
/// offset timer and the per-line redraw. The timer is disarmed again when
/// the writer goes, since nothing here runs the event loop.
struct PaneWriter {
    window: Window,
    pane: Pane,
    state: screen_write_state,
}

impl PaneWriter {
    /// A pane filling its window, so that it is not obscured.
    fn new(sx: u_int, sy: u_int) -> PaneWriter {
        PaneWriter::sized(sx, sy, sx, sy)
    }

    /// A pane of `sx` by `sy` in a window of `wsx` by `wsy`; a pane bigger
    /// than its window is obscured.
    fn sized(sx: u_int, sy: u_int, wsx: u_int, wsy: u_int) -> PaneWriter {
        let mut w = PaneWriter {
            window: Window::new(1, "writer", wsx, wsy),
            pane: Pane::new(1, sx, sy, 100),
            state: screen_write_state::default(),
        };
        w.window.add_pane(&mut w.pane);
        let wp = w.pane.ptr();
        w.state = unsafe { screen_write_start_pane_base(&mut *wp) };
        w
    }

    fn ctx(&mut self) -> screen_write_ctx<'_> {
        screen_write_ctx::new(&mut self.state, self.pane.base_mut())
    }

    fn wp(&mut self) -> *mut window_pane {
        self.pane.ptr()
    }

    fn lines(&mut self) -> Vec<String> {
        unsafe {
            screen_write_collect_end(&mut self.ctx());
            screen_write_collect_flush(&mut self.ctx(), 0, c"test");
            let gd = RustScreen::grid_mut(self.pane.base_mut());
            (0..(*gd).sy)
                .map(|y| {
                    let p = grid_string_cells(&*gd, 0, (*gd).hsize + y, (*gd).sx, None, 0, None);
                    p.to_string_lossy().trim_end().to_string()
                })
                .collect()
        }
    }

    fn puts(&mut self, text: &str) {
        for byte in text.bytes() {
            let gc = ascii(byte);
            unsafe { screen_write_cell(&mut self.ctx(), &gc) };
        }
    }

    fn move_to(&mut self, px: u_int, py: u_int) {
        unsafe { screen_write_cursormove(&mut self.ctx(), px as c_int, py as c_int, 0) };
    }
}

impl Drop for PaneWriter {
    fn drop(&mut self) {
        unsafe {
            screen_write_stop(&mut self.ctx());
            (*self.window.ptr()).offset_timer.disarm();
        }
    }
}

#[test]
fn a_pane_is_drawn_into_the_screen_it_carries() {
    let _guard = globals();
    let mut w = PaneWriter::new(6, 3);
    w.puts("hello");
    assert_eq!(w.lines(), ["hello", "", ""]);
    assert!(
        unsafe { (*w.wp()).window_context() }
            .unwrap()
            .ptr_eq(w.window.handle())
    );
}

/// A pane that does not fit its window is obscured, and the drawing code
/// redraws its lines itself rather than telling the terminal to shift
/// them about.
#[test]
fn a_pane_that_does_not_fit_its_window_is_obscured() {
    let _guard = globals();
    let mut w = PaneWriter::sized(8, 4, 4, 2);
    w.puts("abcdef");
    w.move_to(1, 0);
    unsafe {
        screen_write_insertcharacter(&mut w.ctx(), 2, 8);
        assert_eq!(w.lines()[0], "a  bcdef");
        screen_write_deletecharacter(&mut w.ctx(), 2, 8);
        assert_eq!(w.lines()[0], "abcdef");
        screen_write_clearcharacter(&mut w.ctx(), 1, 8);
        screen_write_insertline(&mut w.ctx(), 1, 8);
        screen_write_deleteline(&mut w.ctx(), 1, 8);
        screen_write_clearline(&mut w.ctx(), 8);
        screen_write_clearendofline(&mut w.ctx(), 8);
        screen_write_clearstartofline(&mut w.ctx(), 8);
        screen_write_clearendofscreen(&mut w.ctx(), 8);
        screen_write_clearstartofscreen(&mut w.ctx(), 8);
        screen_write_clearscreen(&mut w.ctx(), 8);
        screen_write_reverseindex(&mut w.ctx(), 8);
        screen_write_scrollup(&mut w.ctx(), 1, 8);
        screen_write_scrolldown(&mut w.ctx(), 1, 8);
        screen_write_alignmenttest(&mut w.ctx());
    }
    assert_eq!(w.lines(), ["EEEEEEEE", "EEEEEEEE", "EEEEEEEE", "EEEEEEEE"]);
}

/// A floating pane lying over another one obscures it too. The walk is
/// backwards along the z-index list, so the floating pane has to sit in
/// front of the one being drawn for it to be seen at all.
#[test]
fn a_pane_under_a_floating_one_is_obscured() {
    let _guard = globals();
    let mut over = Pane::new(2, 6, 3, 100);
    let mut w = PaneWriter::new(6, 3);
    unsafe {
        let base = w.wp();
        let above = over.hand_to(w.window.ptr());
        crate::tests::test_fixtures::set_pane_floating(
            &mut *w.window.ptr(),
            (*above).pane_id(),
            true,
        );
        (*w.window.ptr())
            .z_index
            .retain(|pane| pane.id() != (*above).pane_id());
        let at = (*w.window.ptr())
            .z_index
            .iter()
            .position(|pane| pane.id() == (*base).pane_id())
            .unwrap();
        (*w.window.ptr()).z_index.insert(
            at,
            crate::window::window_pane_find_by_id((*above).pane_id()).unwrap(),
        );

        let mut ttyctx = Box::new(tty_ctx::default());
        screen_write_initctx(&mut w.ctx(), &mut ttyctx, 0, 1);
        assert_eq!(ttyctx.flags & TTY_CTX_PANE_OBSCURED, TTY_CTX_PANE_OBSCURED);

        w.ctx().flags &= !(SCREEN_WRITE_CHECKED_IF_OBSCURED | SCREEN_WRITE_OBSCURED);
        crate::tests::test_fixtures::set_pane_floating(
            &mut *w.window.ptr(),
            (*above).pane_id(),
            false,
        );
        let mut ttyctx = Box::new(tty_ctx::default());
        screen_write_initctx(&mut w.ctx(), &mut ttyctx, 0, 1);
        assert_eq!(ttyctx.flags & TTY_CTX_PANE_OBSCURED, 0);

        (*w.window.ptr())
            .z_index
            .retain(|pane| pane.id() != (*above).pane_id());
    }
}

/// The answer to "is this pane obscured" is worked out once and kept.
#[test]
fn the_obscured_answer_is_only_worked_out_once() {
    let _guard = globals();
    let mut w = PaneWriter::sized(8, 4, 4, 2);
    unsafe {
        let mut ttyctx = Box::new(tty_ctx::default());
        screen_write_initctx(&mut w.ctx(), &mut ttyctx, 0, 1);
        assert_eq!(ttyctx.flags & TTY_CTX_PANE_OBSCURED, TTY_CTX_PANE_OBSCURED);
        let mut ttyctx = Box::new(tty_ctx::default());
        screen_write_initctx(&mut w.ctx(), &mut ttyctx, 0, 1);
        assert_eq!(ttyctx.flags & TTY_CTX_PANE_OBSCURED, TTY_CTX_PANE_OBSCURED);
        w.ctx().flags &= !SCREEN_WRITE_OBSCURED;
        let mut ttyctx = Box::new(tty_ctx::default());
        screen_write_initctx(&mut w.ctx(), &mut ttyctx, 0, 1);
        assert_eq!(ttyctx.flags & TTY_CTX_PANE_OBSCURED, 0);
    }
}

/// Under synchronised updates the per-line redraw does nothing.
#[test]
fn a_synchronised_pane_is_not_redrawn_line_by_line() {
    let _guard = globals();
    let mut w = PaneWriter::sized(8, 4, 4, 2);
    unsafe {
        screen_write_start_sync(w.wp().as_mut());
        assert_eq!((*w.wp()).base().mode() & MODE_SYNC, MODE_SYNC);
        screen_write_mode_set(&mut w.ctx(), MODE_SYNC);
        w.puts("abc");
        screen_write_insertcharacter(&mut w.ctx(), 1, 8);
        screen_write_mode_clear(&mut w.ctx(), MODE_SYNC);
        screen_write_stop_sync(w.wp().as_mut());
        assert_eq!((*w.wp()).base().mode() & MODE_SYNC, 0);
        screen_write_start_sync(None::<&mut crate::types::window_pane>);
        screen_write_stop_sync(None::<&mut crate::types::window_pane>);
        screen_write_sync_callback_for_test(&mut *w.wp());
        (*w.wp())
            .base_mut()
            .set_mode((*w.wp()).base().mode() | MODE_SYNC);
        screen_write_sync_callback_for_test(&mut *w.wp());
        assert_eq!((*w.wp()).base().mode() & MODE_SYNC, 0);
        assert_eq!(*(*w.wp()).flags() & PANE_REDRAW, PANE_REDRAW);
    }
}

#[test]
fn a_full_redraw_asks_for_one() {
    let _guard = globals();
    let mut w = Writer::new(4, 2, 100);
    unsafe { screen_write_fullredraw(&mut w.ctx()) };
    let mut w = PaneWriter::new(4, 2);
    unsafe {
        *(*w.wp()).flags_mut() &= !PANE_REDRAW;
        screen_write_fullredraw(&mut w.ctx());
        assert_eq!(*(*w.wp()).flags() & PANE_REDRAW, PANE_REDRAW);
    }
}

/// The offset timer is armed when the cursor moves inside a pane, and the
/// callback it is armed with only updates the window's own offsets.
#[test]
fn a_cursor_move_inside_a_pane_arms_the_offset_timer() {
    let _guard = globals();
    let mut w = PaneWriter::new(6, 3);
    w.move_to(2, 1);
    unsafe {
        assert!((*w.window.ptr()).offset_timer.is_set());
        screen_write_offset_timer(w.window.reference());
    }
}

/// A writer may be started with a callback of its own instead of a pane,
/// which is what the overlay drawing does.
#[test]
fn a_writer_may_be_started_with_a_callback() {
    let _guard = globals();
    let mut palette = colour_palette::default();
    palette.fg = 4;
    palette.bg = 5;
    let init = std::rc::Rc::new(move |ttyctx: &mut tty_ctx| {
        ttyctx.defaults.fg = 8;
        ttyctx.defaults.bg = 8;
        ttyctx.palette = Some(palette.clone());
    });
    let mut screen = Screen::new(4, 2, 100);
    unsafe {
        let mut state = screen_write_start_callback(&mut screen, Some(init));
        let mut ctx = screen_write_ctx::new(&mut state, &mut screen);
        let mut ttyctx = Box::new(tty_ctx::default());
        screen_write_initctx(&mut ctx, &mut ttyctx, 0, 0);
        assert_eq!((ttyctx.defaults.fg, ttyctx.defaults.bg), (4, 5));
        screen_write_stop(&mut ctx);
    }
}

#[test]
fn a_cursor_count_of_nothing_is_read_as_one() {
    let _guard = globals();
    let mut w = Writer::new(6, 4, 100);
    w.move_to(3, 2);
    unsafe {
        screen_write_cursorup(&mut w.ctx(), 0);
        assert_eq!(w.cursor(), (3, 1));
        screen_write_cursordown(&mut w.ctx(), 0);
        assert_eq!(w.cursor(), (3, 2));
        screen_write_cursorleft(&mut w.ctx(), 0);
        assert_eq!(w.cursor(), (2, 2));
        screen_write_cursorright(&mut w.ctx(), 0);
        assert_eq!(w.cursor(), (3, 2));
    }
    w.move_to(5, 2);
    unsafe {
        screen_write_cursorright(&mut w.ctx(), 1);
        assert_eq!(w.cursor(), (5, 2));
    }
    w.move_to(0, 2);
    unsafe {
        screen_write_cursorleft(&mut w.ctx(), 1);
        assert_eq!(w.cursor(), (0, 2));
    }
}

/// A cursor sitting one past the last column — where a full line leaves it
/// when wrapping is on — steps back into the line before moving up or
/// down.
#[test]
fn a_cursor_past_the_last_column_steps_back_first() {
    let _guard = globals();
    let mut w = Writer::new(4, 3, 100);
    w.move_to(0, 1);
    w.puts("abcd");
    assert_eq!(w.cursor(), (4, 1));
    unsafe { screen_write_cursorup(&mut w.ctx(), 1) };
    assert_eq!(w.cursor(), (3, 0));
    w.move_to(0, 1);
    w.puts("abcd");
    unsafe { screen_write_cursordown(&mut w.ctx(), 1) };
    assert_eq!(w.cursor(), (3, 2));
}

#[test]
fn nothing_is_inserted_past_the_end_of_a_line() {
    let _guard = globals();
    let mut w = Writer::new(4, 2, 100);
    w.move_to(0, 0);
    w.puts("abcd");
    unsafe {
        screen_write_insertcharacter(&mut w.ctx(), 1, 8);
        screen_write_deletecharacter(&mut w.ctx(), 1, 8);
        screen_write_clearcharacter(&mut w.ctx(), 1, 8);
    }
    assert_eq!(w.lines()[0], "abcd");
}

/// Clearing to the start of the line from its last column clears the
/// whole line.
#[test]
fn clearing_to_the_start_from_the_last_column_clears_the_line() {
    let _guard = globals();
    let mut w = Writer::new(4, 2, 100);
    w.write_at(0, 0, "abcd");
    w.move_to(3, 0);
    unsafe { screen_write_clearstartofline(&mut w.ctx(), 8) };
    assert_eq!(w.lines()[0], "");
}

/// A scroll or line feed in a colour the writer is not already using
/// flushes what is waiting first. The parser always ends what it has
/// collected before it dispatches a control sequence, so nothing here
/// scrolls with an item still open.
#[test]
fn a_scroll_in_a_new_colour_flushes_first() {
    let _guard = globals();
    let mut w = Writer::new(4, 3, 100);
    w.collect("ab");
    unsafe {
        screen_write_collect_end(&mut w.ctx());
        screen_write_linefeed(&mut w.ctx(), 0, 2);
        assert_eq!(w.ctx().bg, 2);
    }
    assert_eq!(w.lines(), ["ab", "", ""]);
    let mut w = Writer::new(4, 3, 100);
    w.collect("ab");
    unsafe {
        screen_write_collect_end(&mut w.ctx());
        screen_write_scrollup(&mut w.ctx(), 0, 3);
        assert_eq!(w.ctx().bg, 3);
    }
    assert_eq!(w.history(), ["ab"]);
    let mut w = Writer::new(4, 3, 100);
    unsafe {
        screen_write_scrollup(&mut w.ctx(), 99, 8);
        screen_write_scrolldown(&mut w.ctx(), 0, 8);
        screen_write_scrolldown(&mut w.ctx(), 99, 8);
    }
    assert_eq!(w.lines(), ["", "", ""]);
}

/// Clearing to the end of the screen from its very top throws the whole
/// screen into the history instead, when the pane asks for that.
#[test]
fn clearing_from_the_top_of_a_pane_scrolls_it_into_the_history() {
    let _guard = globals();
    let mut w = PaneWriter::new(4, 2);
    unsafe {
        (*(*w.wp()).options_ref()).set_number(c"scroll-on-clear", 1);
    }
    w.puts("ab");
    w.move_to(0, 0);
    unsafe { screen_write_clearendofscreen(&mut w.ctx(), 8) };
    assert_eq!(w.lines(), ["", ""]);
    {
        let gd = RustScreen::grid_mut(w.pane.base_mut());
        assert_eq!((*gd).hsize, 1);
    }
}

#[test]
fn a_reset_turns_on_extended_keys_when_the_option_asks() {
    let _guard = globals();
    let mut w = Writer::new(4, 2, 100);
    unsafe {
        (global_options
            .as_ref()
            .expect("global options are initialized"))
        .set_number(c"extended-keys", 2);
        screen_write_reset(&mut w.ctx());
        assert_eq!(w.screen().mode() & MODE_KEYS_EXTENDED, MODE_KEYS_EXTENDED);
        (global_options
            .as_ref()
            .expect("global options are initialized"))
        .set_number(c"extended-keys", 0);
        screen_write_reset(&mut w.ctx());
        assert_eq!(w.screen().mode() & MODE_KEYS_EXTENDED, 0);
    }
}

/// A writer started on a pane with no screen of its own draws into the
/// pane's base screen.
#[test]
fn a_pane_writer_falls_back_to_the_panes_own_screen() {
    let _guard = globals();
    crate::log::log_with_level(1, || {
        let mut window = Window::new(1, "writer", 6, 3);
        let mut pane = Pane::new(1, 6, 3, 100);
        window.add_pane(&mut pane);
        let wp = pane.ptr();
        unsafe {
            use crate::screen::ScreenWriteCtx;
            *(*wp).shown_mut() = PaneScreen::Mode;
            let mut writer = crate::screen::RustScreenWriteCtx::on_pane(&mut *wp);
            writer.cell(&ascii(b'x'));
            writer.finish();
            assert_eq!((*wp).base().cursor(), (1, 0));
            assert_eq!(
                grid_view_get_cell((*wp).base().grid(), 0, 0).data.data[0],
                b'x'
            );
            (*window.ptr()).offset_timer.disarm();
        }
    });
}

/// A preview wider or taller than the screen it shows starts at its top
/// left; one that would run off the end is pulled back.
#[test]
fn a_preview_is_pulled_back_inside_the_screen_it_shows() {
    let _guard = globals();
    let mut src = Writer::new(10, 6, 100);
    for y in 0..6 {
        src.write_at(0, y, "abcdefghij");
    }
    src.move_to(9, 5);
    let mut w = Writer::new(10, 6, 100);
    unsafe { screen_write_preview(&mut w.ctx(), src.screen(), 4, 2) };
    assert_eq!(w.lines()[0], "ghij");
    let mut w = Writer::new(10, 6, 100);
    unsafe { screen_write_preview(&mut w.ctx(), src.screen(), 20, 20) };
    assert_eq!(w.lines()[0], "abcdefghij");
}

/// The copy walks as many rows as it is given, so the pane has to be at
/// least that tall: the row it writes is never checked against the
/// destination's own height.
#[test]
fn a_copy_into_a_pane_stops_at_the_panes_own_edges() {
    let _guard = globals();
    let mut src = Writer::new(6, 3, 100);
    src.write_at(0, 0, "abcdef");
    src.flush();
    let mut w = PaneWriter::new(4, 3);
    unsafe { screen_write_fast_copy(&mut w.ctx(), src.screen(), 0, 0, 6, 3) };
    assert_eq!(w.lines(), ["abcd", "", ""]);
}

/// A cell is single when it is one column of one printable byte that is
/// neither cleared, padding nor a tab. Its style is not looked at, so a
/// cell in the alternate character set or carrying a hyperlink is still
/// single.
#[test]
fn a_cell_is_single_only_when_it_is_one_plain_character() {
    let _guard = globals();
    let single = |gc: &grid_cell| unsafe { screen_write_cell_is_single(gc) };
    assert_eq!(single(&ascii(b'x')), 1);
    assert_eq!(single(&utf8("\u{4e00}")), 0);
    assert_eq!(single(&ascii(0x1f)), 0);
    assert_eq!(single(&ascii(0x7f)), 0);
    let mut gc = ascii(b'x');
    gc.attr = (gc.attr as c_int | GRID_ATTR_CHARSET) as u_short;
    assert_eq!(single(&gc), 1);
    let mut gc = ascii(b'x');
    gc.link = 1;
    assert_eq!(single(&gc), 1);
    for flag in [GRID_FLAG_CLEARED, GRID_FLAG_PADDING, GRID_FLAG_TAB] {
        let mut gc = ascii(b'x');
        gc.flags = (gc.flags as c_int | flag) as u_char;
        assert_eq!(single(&gc), 0, "{flag:#x}");
    }
    let mut gc = ascii(b'x');
    gc.data.size = 2;
    assert_eq!(single(&gc), 0);
}

/// A joiner, a variation selector and a zero-width character all join the
/// character in front of them; a Hangul filler is dropped outright. The
/// variation selector widens what it joins, because
/// `variation-selector-always-wide` defaults to on — verified against the
/// pinned tmux 3.7b, whose cursor also lands at 2 here.
#[test]
fn the_characters_that_join_what_is_in_front_of_them() {
    let _guard = globals();
    let filler = utf8("\u{3164}");
    let zwj = utf8("\u{200d}");
    let vs = utf8("\u{fe0f}");
    let mut w = Writer::new(8, 2, 100);
    w.puts("a");
    unsafe {
        screen_write_cell(&mut w.ctx(), &filler);
        assert_eq!(w.cursor(), (1, 0));
        screen_write_cell(&mut w.ctx(), &zwj);
        assert_eq!(w.cursor(), (1, 0));
        screen_write_cell(&mut w.ctx(), &vs);
        assert_eq!(w.cursor(), (2, 0));
    }
    assert_eq!(w.cell(0, 0).data.size, 7);
    assert_eq!(
        w.cell(1, 0).flags as c_int & GRID_FLAG_PADDING,
        GRID_FLAG_PADDING
    );

    let mut w = Writer::new(8, 2, 100);
    unsafe {
        screen_write_cell(&mut w.ctx(), &zwj);
        assert_eq!(w.cursor(), (0, 0));
    }
}

/// With `variation-selector-always-wide` off, a variation selector still
/// joins the character in front of it but leaves its width alone —
/// verified against the pinned tmux 3.7b with the option turned off.
#[test]
fn a_variation_selector_widens_nothing_when_asked_not_to() {
    let _guard = globals();
    let vs = utf8("\u{fe0f}");
    let mut w = Writer::new(8, 2, 100);
    unsafe {
        let name = c"variation-selector-always-wide".as_ptr();
        let before = (global_options
            .as_ref()
            .expect("global options are initialized"))
        .number(CStr::from_ptr(name));
        (global_options
            .as_ref()
            .expect("global options are initialized"))
        .set_number(CStr::from_ptr(name), 0);
        w.puts("a");
        screen_write_cell(&mut w.ctx(), &vs);
        (global_options
            .as_ref()
            .expect("global options are initialized"))
        .set_number(CStr::from_ptr(name), before);
    }
    assert_eq!(w.cursor(), (1, 0));
    assert_eq!(w.cell(0, 0).data.size, 4);
    assert_eq!(w.cell(1, 0).flags as c_int & GRID_FLAG_PADDING, 0);
}

/// Hangul jamo compose into one cell as they are written.
#[test]
fn hangul_jamo_compose_into_the_cell_in_front_of_them() {
    let _guard = globals();
    let lead = utf8("\u{1100}");
    let vowel = utf8("\u{1161}");
    let tail = utf8("\u{11a8}");
    let mut w = Writer::new(8, 2, 100);
    unsafe {
        screen_write_cell(&mut w.ctx(), &lead);
        screen_write_cell(&mut w.ctx(), &vowel);
        screen_write_cell(&mut w.ctx(), &tail);
    }
    assert_eq!(w.cursor(), (2, 0));
    assert_eq!(w.cell(0, 0).data.size, 9);
}

/// A character that would not fit in the cell's own thirty-two bytes is
/// written on its own instead.
#[test]
fn a_join_that_would_not_fit_is_written_on_its_own() {
    let _guard = globals();
    let zwj = utf8("\u{200d}");
    let hand = utf8("\u{1f44b}");
    let mut w = Writer::new(8, 2, 100);
    unsafe {
        screen_write_cell(&mut w.ctx(), &hand);
        for _ in 0..5 {
            screen_write_cell(&mut w.ctx(), &zwj);
            screen_write_cell(&mut w.ctx(), &hand);
        }
    }
    assert!(w.cell(0, 0).data.size <= 32);
    assert!(w.cursor().0 >= 2);
}

/// Writing over the padding of a wide character clears the whole of it.
#[test]
fn writing_over_padding_clears_the_character_it_belongs_to() {
    let _guard = globals();
    let wide = utf8("\u{4e00}");
    let mut w = Writer::new(6, 2, 100);
    unsafe {
        screen_write_cell(&mut w.ctx(), &wide);
        screen_write_cell(&mut w.ctx(), &wide);
    }
    w.flush();
    w.move_to(1, 0);
    let mut tab = ascii(b' ');
    tab.flags = (tab.flags as c_int | GRID_FLAG_TAB) as u_char;
    unsafe { screen_write_cell(&mut w.ctx(), &tab) };
    assert_eq!(w.lines()[0], " \t\u{4e00}");
}

#[test]
fn a_pane_may_refuse_the_alternate_screen() {
    let _guard = globals();
    let mut w = PaneWriter::new(4, 2);
    let mut gc = { grid_default_cell };
    w.puts("main");
    unsafe {
        (*(*w.wp()).options_ref()).set_number(c"alternate-screen", 0);
        screen_write_alternateon(&mut w.ctx(), &mut gc, 0);
        assert_eq!(w.lines()[0], "main");
        screen_write_alternateoff(&mut w.ctx(), &mut gc, 0);
        assert_eq!(w.lines()[0], "main");
        (*(*w.wp()).options_ref()).set_number(c"alternate-screen", 1);
        screen_write_alternateon(&mut w.ctx(), &mut gc, 0);
        assert_eq!(w.lines()[0], "");
        screen_write_alternateoff(&mut w.ctx(), &mut gc, 0);
        assert_eq!(w.lines()[0], "main");
    }
}

/// Text collected right after a wide character takes the padding cell's
/// place, and the character it belonged to is erased with it.
#[test]
fn collected_text_erases_the_padding_it_lands_on() {
    let _guard = globals();
    let wide = utf8("\u{4e00}");
    let mut w = Writer::new(8, 2, 100);
    unsafe {
        screen_write_cell(&mut w.ctx(), &wide);
        screen_write_cell(&mut w.ctx(), &wide);
    }
    w.flush();
    w.move_to(1, 0);
    w.collect("ab");
    assert_eq!(w.lines()[0], " ab");
    let mut w = Writer::new(8, 2, 100);
    unsafe {
        screen_write_cell(&mut w.ctx(), &wide);
        screen_write_cell(&mut w.ctx(), &wide);
    }
    w.flush();
    w.move_to(0, 0);
    w.collect("ab");
    assert_eq!(w.lines()[0], "ab\u{4e00}");
}

/// The first write after the writer starts opens a synchronised update.
#[test]
fn the_first_write_opens_a_synchronised_update() {
    let _guard = globals();
    let mut w = Writer::new(4, 3, 100);
    unsafe {
        assert_eq!(w.ctx().flags & SCREEN_WRITE_SYNC, 0);
        screen_write_insertline(&mut w.ctx(), 1, 8);
        assert_eq!(w.ctx().flags & SCREEN_WRITE_SYNC, SCREEN_WRITE_SYNC);
    }
}

/// An obscured pane redraws the lines it cleared itself, one visible
/// range at a time.
#[test]
fn an_obscured_pane_redraws_what_it_cleared() {
    let _guard = globals();
    let mut w = PaneWriter::sized(4, 4, 2, 2);
    w.puts("abcd");
    w.move_to(0, 2);
    w.puts("efgh");
    unsafe {
        screen_write_clearstartofscreen(&mut w.ctx(), 8);
        assert_eq!(w.lines(), ["", "", "", ""]);
    }
    w.move_to(0, 1);
    w.puts("abcd");
    unsafe {
        screen_write_clearstartofscreen(&mut w.ctx(), 8);
        screen_write_clearendofscreen(&mut w.ctx(), 8);
        screen_write_clearscreen(&mut w.ctx(), 8);
        assert_eq!(w.lines(), ["", "", "", ""]);
    }
}

/// An obscured pane redraws the whole of itself when lines move about
/// outside the scrolling region.
#[test]
fn an_obscured_pane_redraws_itself_when_lines_move() {
    let _guard = globals();
    let mut w = PaneWriter::sized(4, 4, 2, 2);
    unsafe {
        screen_write_scrollregion(&mut w.ctx(), 2, 3);
    }
    w.move_to(0, 0);
    w.puts("abcd");
    w.move_to(0, 0);
    unsafe {
        screen_write_insertline(&mut w.ctx(), 1, 8);
        assert_eq!(w.lines(), ["", "abcd", "", ""]);
        screen_write_deleteline(&mut w.ctx(), 1, 8);
        assert_eq!(w.lines(), ["abcd", "", "", ""]);
        screen_write_insertline(&mut w.ctx(), 0, 8);
        screen_write_deleteline(&mut w.ctx(), 0, 8);
    }
    w.move_to(0, 3);
    unsafe {
        screen_write_insertline(&mut w.ctx(), 99, 8);
        screen_write_deleteline(&mut w.ctx(), 99, 8);
    }
}

/// A pane one column wide is redrawn cell by cell rather than line by
/// line.
#[test]
fn a_pane_one_column_wide_is_redrawn_cell_by_cell() {
    let _guard = globals();
    let mut w = PaneWriter::sized(1, 3, 1, 1);
    w.puts("a");
    w.move_to(0, 0);
    unsafe {
        screen_write_clearcharacter(&mut w.ctx(), 1, 8);
        screen_write_insertcharacter(&mut w.ctx(), 1, 8);
    }
    assert_eq!(w.lines(), ["", "", ""]);
    let mut w = PaneWriter::sized(1, 3, 1, 1);
    let wide = utf8("\u{4e00}");
    unsafe {
        screen_write_cell(&mut w.ctx(), &wide);
        screen_write_insertcharacter(&mut w.ctx(), 1, 8);
    }
    let mut w = PaneWriter::sized(1, 3, 1, 1);
    let gc = { grid_default_cell };
    unsafe {
        w.pane
            .base_mut()
            .set_selection(0, 0, 1, 0, false, 0, 0, &gc);
        w.puts("a");
        screen_write_insertcharacter(&mut w.ctx(), 1, 8);
        w.pane.base_mut().clear_selection();
    }
}

/// Writing over the padding of a wide character clears the character it
/// belongs to, however far back it started.
#[test]
fn writing_over_padding_reaches_back_to_the_character() {
    let _guard = globals();
    let wide = utf8("\u{4e00}");
    let mut w = Writer::new(6, 2, 100);
    w.puts("a");
    unsafe { screen_write_cell(&mut w.ctx(), &wide) };
    w.flush();
    w.move_to(2, 0);
    w.puts("x");
    assert_eq!(w.lines()[0], "a x");
}

/// When a wide character is written over a tab, the cells it covers are
/// filled with tab cells rather than blanks — the fill copies whatever
/// was written over, and a tab is what that was.
#[test]
fn a_wide_character_over_a_tab_leaves_tab_cells() {
    let _guard = globals();
    let wide = utf8("\u{4e00}");
    let mut w = Writer::new(6, 2, 100);
    let mut tab = ascii(b' ');
    tab.flags = (tab.flags as c_int | GRID_FLAG_TAB) as u_char;
    unsafe {
        screen_write_cell(&mut w.ctx(), &tab);
        screen_write_cell(&mut w.ctx(), &wide);
    }
    w.flush();
    w.move_to(0, 0);
    unsafe { screen_write_cell(&mut w.ctx(), &wide) };
    assert_eq!(w.cell(2, 0).flags as c_int & GRID_FLAG_TAB, GRID_FLAG_TAB);
}

/// Clearing to the start of the screen from one past the last column
/// clears the whole line.
#[test]
fn clearing_to_the_start_of_the_screen_from_past_the_line_clears_it() {
    let _guard = globals();
    let mut w = Writer::new(4, 2, 100);
    w.move_to(0, 1);
    w.puts("abcd");
    assert_eq!(w.cursor(), (4, 1));
    unsafe { screen_write_clearstartofscreen(&mut w.ctx(), 8) };
    assert_eq!(w.lines(), ["", ""]);
}

#[test]
fn a_character_is_put_with_a_style_of_its_own() {
    let _guard = globals();
    let mut w = Writer::new(4, 2, 100);
    let mut gc = { grid_default_cell };
    gc.fg = 3;
    unsafe { screen_write_putc(&mut w.ctx(), &gc, b'x') };
    assert_eq!(w.cell(0, 0).data.data[0], b'x');
    assert_eq!(w.cell(0, 0).fg, 3);
}

#[test]
fn collected_item_pool_preserves_released_data_until_fifo_reuse() {
    let mut pool = CItemPool::new();
    let first = pool.allocate();
    let second = pool.allocate();
    pool.items[first as usize].wrapped = 1;
    pool.items[first as usize].used = 42;
    pool.free.extend([first, second]);
    assert_eq!(pool.items[first as usize].wrapped, 1);
    assert_eq!(pool.items[first as usize].used, 42);
    assert_eq!(pool.allocate(), first);
    assert_eq!(pool.items[first as usize].wrapped, 0);
    assert_eq!(pool.items[first as usize].used, 0);
    assert_eq!(pool.allocate(), second);
    assert_eq!(pool.items.len(), 2);
}

#[test]
fn collected_item_pool_never_allocates_the_sentinel_or_wraps() {
    assert_eq!(next_citem_index(CITEM_NONE as usize - 1), CITEM_NONE - 1);
    assert!(std::panic::catch_unwind(|| next_citem_index(CITEM_NONE as usize)).is_err());
    if let Some(overflow) = (CITEM_NONE as usize).checked_add(1) {
        assert!(std::panic::catch_unwind(|| next_citem_index(overflow)).is_err());
    }
}

#[test]
fn collected_item_pool_serializes_concurrent_allocations_and_keeps_snapshots() {
    let threads: Vec<_> = (0..8)
        .map(|owner| {
            std::thread::spawn(move || {
                let mut items = Vec::new();
                for sequence in 0..64 {
                    let ci = screen_write_get_citem();
                    with_citem(ci, |item| {
                        item.x = owner;
                        item.used = sequence;
                    });
                    items.push((ci, owner, sequence));
                }
                items
            })
        })
        .collect();
    let mut ids = std::collections::BTreeSet::new();
    let mut live = Vec::new();
    for thread in threads {
        for (ci, owner, sequence) in thread.join().unwrap() {
            assert!(ids.insert(ci));
            let snapshot = citem_snapshot(ci);
            assert_eq!((snapshot.x, snapshot.used), (owner, sequence));
            live.push(ci);
        }
    }
    citem_free_all(&mut live);
    assert!(live.is_empty());
}

#[test]
fn terminal_callbacks_ignore_a_pane_destroyed_after_context_creation() {
    let _guard = globals();
    let mut ttyctx = tty_ctx::default();
    {
        let mut writer = PaneWriter::new(6, 3);
        unsafe { screen_write_initctx(&mut writer.ctx(), &mut ttyctx, 0, 0) };
    }
    let TtyCtxArg::Pane(pane) = &ttyctx.arg else {
        panic!("a pane writer sets a pane callback argument");
    };
    assert!(!pane.is_alive());
    let mut clients = crate::tests::test_fixtures::Clients::new();
    let client = clients.add("client", 6, 3);
    unsafe {
        ttyctx.redraw_cb.expect("pane contexts can request redraw")(&ttyctx);
        assert_eq!(
            ttyctx.set_client_cb.expect("pane contexts select clients")(&mut ttyctx, &mut *client),
            0
        );
    }
}

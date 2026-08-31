use super::*;
use crate::grid::{
    grid_create, grid_default_cell, grid_get_line, grid_set_cell, grid_set_cells, grid_set_padding,
    grid_set_tab,
};
use crate::tests::test_fixtures::globals;
use ::core::ffi::{CStr, c_int};

const SEPARATORS: &CStr = c" -_@";

/// A grid holding a few lines, which frees itself at the end of the test.
struct Grid(Box<grid>);

impl Grid {
    /// A grid `sx` wide holding one line per string. A line whose string
    /// ends in `\\` is marked as wrapping onto the next one.
    fn of(sx: u_int, lines: &[&str]) -> Grid {
        let g = Grid(grid_create(sx, lines.len() as u_int, 0));
        let gc = grid_default_cell;
        for (py, line) in lines.iter().enumerate() {
            let (text, wrapped) = match line.strip_suffix('\\') {
                Some(text) => (text, true),
                None => (*line, false),
            };
            unsafe {
                grid_set_cells(&mut *g.ptr(), 0, py as u_int, &gc, text.as_bytes());
                if wrapped {
                    grid_get_line(&mut *g.ptr(), py as u_int).flags |= GRID_LINE_WRAPPED;
                }
            }
        }
        g
    }

    fn ptr(&self) -> *mut grid {
        self.0.as_ref() as *const grid as *mut grid
    }

    /// A reader on this grid at (cx, cy).
    fn reader(&self, cx: u_int, cy: u_int) -> grid_reader<'_> {
        grid_reader_start(&self.0, cx, cy)
    }

    /// Writes more text at (px, py), after what is there.
    fn write_after(&self, px: u_int, py: u_int, s: &str) {
        let gc = grid_default_cell;
        unsafe { grid_set_cells(&mut *self.ptr(), px, py, &gc, s.as_bytes()) };
    }

    /// Writes a wide character and its padding at (px, py).
    fn wide(&self, px: u_int, py: u_int) {
        let mut gc = grid_default_cell;
        gc.data.data[..3].copy_from_slice("\u{4e2d}".as_bytes());
        gc.data.have = 3;
        gc.data.size = 3;
        gc.data.width = 2;
        unsafe {
            grid_set_cell(&mut *self.ptr(), px, py, &gc);
            grid_set_padding(&mut *self.ptr(), px + 1, py);
        }
    }

    /// Writes a tab of `width` columns and its padding at (px, py).
    fn tab(&self, px: u_int, py: u_int, width: u_int) {
        let mut gc = grid_default_cell;
        unsafe {
            grid_set_tab(&raw mut gc, width);
            grid_set_cell(&mut *self.ptr(), px, py, &gc);
            for i in 1..width {
                grid_set_padding(&mut *self.ptr(), px + i, py);
            }
        }
    }
}

/// Where the reader's cursor is.
fn cursor(gr: &grid_reader<'_>) -> (u_int, u_int) {
    grid_reader_get_cursor(gr)
}

fn right(gr: &mut grid_reader<'_>, wrap: c_int, all: c_int, onemore: c_int) -> (u_int, u_int) {
    grid_reader_cursor_right(&mut *gr, wrap, all, onemore);
    cursor(gr)
}

fn left(gr: &mut grid_reader<'_>, wrap: c_int) -> (u_int, u_int) {
    grid_reader_cursor_left(&mut *gr, wrap);
    cursor(gr)
}

fn next_word(gr: &mut grid_reader<'_>, separators: &CStr) -> (u_int, u_int) {
    grid_reader_cursor_next_word(&mut *gr, separators);
    cursor(gr)
}

fn next_word_end(gr: &mut grid_reader<'_>, separators: &CStr) -> (u_int, u_int) {
    grid_reader_cursor_next_word_end(&mut *gr, separators);
    cursor(gr)
}

fn previous_word(
    gr: &mut grid_reader,
    separators: &CStr,
    already: c_int,
    stop_at_eol: c_int,
) -> (u_int, u_int) {
    grid_reader_cursor_previous_word(&mut *gr, separators, already, stop_at_eol);
    cursor(gr)
}

/// The one character `s` holds, as the jump commands take it.
fn one(s: &str) -> utf8_data {
    let mut ud = utf8_data::default();
    ud.data[..s.len()].copy_from_slice(s.as_bytes());
    ud.have = s.len() as u_char;
    ud.size = ud.have;
    ud.width = 1;
    ud
}

fn jump(gr: &mut grid_reader<'_>, s: &str) -> (c_int, (u_int, u_int)) {
    let jc = one(s);
    let found = grid_reader_cursor_jump(&mut *gr, &jc);
    (found, cursor(gr))
}

fn jump_back(gr: &mut grid_reader<'_>, s: &str) -> (c_int, (u_int, u_int)) {
    let jc = one(s);
    let found = grid_reader_cursor_jump_back(&mut *gr, &jc);
    (found, cursor(gr))
}

#[test]
fn a_reader_starts_where_it_is_told_to() {
    let _guard = globals();
    let g = Grid::of(10, &["abc", "de"]);
    let mut gr = g.reader(2, 1);
    assert!(::core::ptr::eq(gr.gd, &*g.0));
    assert_eq!(cursor(&gr), (2, 1));
    assert_eq!(grid_reader_line_length(&mut gr), 2);
}

#[test]
fn the_cursor_moves_right_up_to_the_last_character() {
    let _guard = globals();
    let g = Grid::of(10, &["abc", ""]);
    let mut gr = g.reader(0, 0);
    assert_eq!(right(&mut gr, 0, 0, 0), (1, 0));
    assert_eq!(right(&mut gr, 0, 0, 0), (2, 0));
    assert_eq!(right(&mut gr, 0, 0, 0), (2, 0), "the last character");
    assert_eq!(right(&mut gr, 0, 0, 1), (3, 0), "one more is past it");
    assert_eq!(right(&mut gr, 0, 1, 0), (4, 0), "all reaches the width");

    let mut empty = g.reader(0, 1);
    assert_eq!(
        right(&mut empty, 0, 0, 0),
        (0, 1),
        "an empty line is one cell"
    );
}

#[test]
fn the_cursor_moves_right_onto_the_next_line() {
    let _guard = globals();
    let g = Grid::of(10, &["abc", "de"]);
    let mut gr = g.reader(2, 0);
    assert_eq!(right(&mut gr, 1, 0, 0), (0, 1));
    let mut last = g.reader(1, 1);
    assert_eq!(
        right(&mut last, 1, 0, 0),
        (1, 1),
        "there is no line below the last one"
    );
}

#[test]
fn the_cursor_steps_over_padding_on_its_way_right() {
    let _guard = globals();
    let g = Grid::of(10, &["ab"]);
    g.wide(2, 0);
    g.write_after(4, 0, "c");
    let mut gr = g.reader(1, 0);
    assert_eq!(right(&mut gr, 0, 0, 0), (2, 0), "the wide character");
    assert_eq!(right(&mut gr, 0, 0, 0), (4, 0), "its padding is skipped");
}

#[test]
fn the_cursor_moves_left_and_over_padding() {
    let _guard = globals();
    let g = Grid::of(10, &["ab"]);
    g.wide(2, 0);
    g.write_after(4, 0, "c");
    let mut gr = g.reader(4, 0);
    assert_eq!(left(&mut gr, 0), (3, 0));
    assert_eq!(
        left(&mut gr, 0),
        (1, 0),
        "the padding is stepped over on the way"
    );
}

#[test]
fn the_cursor_moves_left_onto_the_line_above() {
    let _guard = globals();
    let g = Grid::of(10, &["abc\\", "de"]);
    let mut gr = g.reader(0, 1);
    assert_eq!(
        left(&mut gr, 0),
        (3, 0),
        "the line above wraps onto this one"
    );

    let plain = Grid::of(10, &["abc", "de"]);
    let mut stay = plain.reader(0, 1);
    assert_eq!(left(&mut stay, 0), (0, 1), "it does not");
    assert_eq!(left(&mut stay, 1), (3, 0), "unless it is asked to wrap");

    let mut top = plain.reader(0, 0);
    assert_eq!(
        left(&mut top, 1),
        (0, 0),
        "there is no line above the first"
    );
}

#[test]
fn the_cursor_moves_up_and_down_and_off_padding() {
    let _guard = globals();
    let g = Grid::of(10, &["abcd", "ab"]);
    g.wide(2, 1);
    let mut gr = g.reader(3, 0);
    assert_eq!(
        {
            grid_reader_cursor_down(&mut gr);
            cursor(&gr)
        },
        (2, 1),
        "the cursor came down onto padding and moved off it"
    );
    assert_eq!(
        {
            grid_reader_cursor_up(&mut gr);
            cursor(&gr)
        },
        (2, 0)
    );
    let mut top = g.reader(0, 0);
    grid_reader_cursor_up(&mut top);
    assert_eq!(cursor(&top), (0, 0));
    let mut bottom = g.reader(0, 1);
    grid_reader_cursor_down(&mut bottom);
    assert_eq!(cursor(&bottom), (0, 1));
}

#[test]
fn the_cursor_comes_off_padding_when_it_moves_up() {
    let _guard = globals();
    let g = Grid::of(10, &["a", "abcd"]);
    g.wide(1, 0);
    let mut gr = g.reader(2, 1);
    grid_reader_cursor_up(&mut gr);
    assert_eq!(cursor(&gr), (1, 0));
}

#[test]
fn the_ends_of_a_wrapped_line_are_the_ends_of_the_whole_run() {
    let _guard = globals();
    let g = Grid::of(10, &["abc\\", "de\\", "fg", "hi"]);
    let mut gr = g.reader(1, 1);
    grid_reader_cursor_start_of_line(&mut gr, 1);
    assert_eq!(cursor(&gr), (0, 0));
    grid_reader_cursor_end_of_line(&mut gr, 1, 0);
    assert_eq!(cursor(&gr), (2, 2));

    let mut plain = g.reader(1, 1);
    grid_reader_cursor_start_of_line(&mut plain, 0);
    assert_eq!(cursor(&plain), (0, 1));
    grid_reader_cursor_end_of_line(&mut plain, 0, 1);
    assert_eq!(cursor(&plain), (10, 1), "all is the width of the grid");
}

#[test]
fn the_cell_under_the_cursor_can_be_looked_for_in_a_set() {
    let _guard = globals();
    let g = Grid::of(10, &["ab"]);
    let mut gr = g.reader(1, 0);
    assert_eq!(grid_reader_in_set(&mut gr, c"b"), 1);
    assert_eq!(grid_reader_in_set(&mut gr, c"a"), 0);
}

#[test]
fn the_next_word_starts_after_the_one_the_cursor_is_in() {
    let _guard = globals();
    let g = Grid::of(20, &["abc def  ghi"]);
    let mut gr = g.reader(0, 0);
    assert_eq!(next_word(&mut gr, SEPARATORS), (4, 0));
    assert_eq!(next_word(&mut gr, SEPARATORS), (9, 0));
    assert_eq!(
        next_word(&mut gr, SEPARATORS),
        (13, 0),
        "the last word runs one past the end of the line"
    );
}

#[test]
fn a_run_of_separators_is_a_word_of_its_own() {
    let _guard = globals();
    let g = Grid::of(20, &["ab--cd"]);
    let mut gr = g.reader(0, 0);
    assert_eq!(
        next_word(&mut gr, SEPARATORS),
        (2, 0),
        "onto the separators"
    );
    assert_eq!(next_word(&mut gr, SEPARATORS), (4, 0), "past them");
}

#[test]
fn the_next_word_carries_on_over_a_wrapped_line() {
    let _guard = globals();
    let g = Grid::of(4, &["ab c\\", "def", "ghi"]);
    let mut gr = g.reader(0, 0);
    assert_eq!(next_word(&mut gr, SEPARATORS), (3, 0));
    assert_eq!(
        next_word(&mut gr, SEPARATORS),
        (0, 2),
        "the word wrapped onto the next line, and the one after it"
    );
}

#[test]
fn the_next_word_stops_at_the_end_of_the_grid() {
    let _guard = globals();
    let g = Grid::of(10, &["ab", "cd"]);
    let mut gr = g.reader(1, 1);
    assert_eq!(next_word(&mut gr, SEPARATORS), (3, 1));
    assert_eq!(next_word(&mut gr, SEPARATORS), (3, 1), "and stays there");
}

#[test]
fn a_word_ends_before_the_next_separator_or_space() {
    let _guard = globals();
    let g = Grid::of(20, &["ab  cd-ef"]);
    let mut gr = g.reader(0, 0);
    assert_eq!(next_word_end(&mut gr, SEPARATORS), (2, 0));
    assert_eq!(next_word_end(&mut gr, SEPARATORS), (6, 0));
    assert_eq!(next_word_end(&mut gr, SEPARATORS), (7, 0), "the separator");
    assert_eq!(next_word_end(&mut gr, SEPARATORS), (9, 0));
    assert_eq!(
        next_word_end(&mut gr, SEPARATORS),
        (10, 0),
        "one past the end of the line, where it stays"
    );
    assert_eq!(next_word_end(&mut gr, SEPARATORS), (10, 0));
}

#[test]
fn a_word_that_wraps_over_two_lines_is_walked_to_the_end_of_the_run() {
    let _guard = globals();
    let g = Grid::of(4, &["abcd\\", "efgh\\", "ij kl"]);
    let mut gr = g.reader(3, 0);
    assert_eq!(
        next_word(&mut gr, SEPARATORS),
        (3, 2),
        "the run of wrapped lines is one word"
    );

    let mut end = g.reader(0, 0);
    assert_eq!(next_word_end(&mut end, SEPARATORS), (2, 2));
}

#[test]
fn a_run_of_separators_ends_a_word_too() {
    let _guard = globals();
    let g = Grid::of(20, &["a--b"]);
    let mut gr = g.reader(1, 0);
    assert_eq!(next_word_end(&mut gr, SEPARATORS), (3, 0));
}

#[test]
fn the_previous_word_starts_before_the_one_the_cursor_is_in() {
    let _guard = globals();
    let g = Grid::of(20, &["abc def ghi"]);
    let mut gr = g.reader(9, 0);
    assert_eq!(previous_word(&mut gr, SEPARATORS, 0, 0), (8, 0));
    assert_eq!(
        previous_word(&mut gr, SEPARATORS, 0, 0),
        (8, 0),
        "already there"
    );
    assert_eq!(previous_word(&mut gr, SEPARATORS, 1, 0), (4, 0));
    assert_eq!(previous_word(&mut gr, SEPARATORS, 1, 0), (0, 0));
    assert_eq!(
        previous_word(&mut gr, SEPARATORS, 1, 0),
        (0, 0),
        "the start"
    );
}

#[test]
fn the_previous_word_walks_back_over_lines() {
    let _guard = globals();
    let g = Grid::of(6, &["abc de\\", "fgh"]);
    let mut gr = g.reader(1, 1);
    assert_eq!(
        previous_word(&mut gr, SEPARATORS, 0, 0),
        (4, 0),
        "the word carries on from the line above"
    );

    let plain = Grid::of(6, &["abc", "de"]);
    let mut over = plain.reader(0, 1);
    assert_eq!(
        previous_word(&mut over, SEPARATORS, 1, 0),
        (0, 0),
        "the line above is a word of its own"
    );
}

#[test]
fn the_previous_word_can_stop_at_the_end_of_a_line() {
    let _guard = globals();
    let g = Grid::of(10, &["ab", "def"]);
    g.tab(2, 0, 2);
    let mut gr = g.reader(0, 1);
    assert_eq!(
        previous_word(&mut gr, SEPARATORS, 1, 1),
        (4, 0),
        "the line above ends in whitespace, so that is where it stops"
    );
    assert_eq!(
        previous_word(&mut g.reader(0, 1), SEPARATORS, 1, 0),
        (0, 0),
        "without stopping it carries on to the word"
    );

    let letters = Grid::of(10, &["abc", "def"]);
    assert_eq!(
        previous_word(&mut letters.reader(0, 1), SEPARATORS, 1, 1),
        (0, 0),
        "a line that ends in a word is walked back over as usual"
    );
}

#[test]
fn there_is_no_word_before_the_first_line() {
    let _guard = globals();
    let g = Grid::of(10, &["  ab"]);
    let mut gr = g.reader(1, 0);
    assert_eq!(
        previous_word(&mut gr, SEPARATORS, 0, 0),
        (0, 0),
        "the walk back stops where it ran out of lines"
    );
}

#[test]
fn a_separator_before_the_cursor_is_a_word() {
    let _guard = globals();
    let g = Grid::of(10, &["ab-cd"]);
    let mut gr = g.reader(4, 0);
    assert_eq!(previous_word(&mut gr, SEPARATORS, 0, 0), (3, 0));
    assert_eq!(
        previous_word(&mut gr, SEPARATORS, 1, 0),
        (2, 0),
        "the separator"
    );
    assert_eq!(previous_word(&mut gr, SEPARATORS, 1, 0), (0, 0));
}

#[test]
fn the_cursor_jumps_to_a_character_and_back() {
    let _guard = globals();
    let g = Grid::of(10, &["abcabc", "abc"]);
    let mut gr = g.reader(0, 0);
    assert_eq!(jump(&mut gr, "b"), (1, (1, 0)));
    assert_eq!(jump(&mut gr, "b"), (1, (1, 0)), "it is already there");
    let mut from = g.reader(2, 0);
    assert_eq!(jump(&mut from, "b"), (1, (4, 0)));
    assert_eq!(jump(&mut from, "z"), (0, (4, 0)), "nothing to jump to");

    let mut back = g.reader(5, 0);
    assert_eq!(jump_back(&mut back, "b"), (1, (4, 0)));
    assert_eq!(
        jump_back(&mut back, "b"),
        (1, (4, 0)),
        "the cell under the cursor is looked at too"
    );
    assert_eq!(jump_back(&mut back, "z"), (0, (4, 0)));
}

#[test]
fn a_jump_carries_on_over_a_wrapped_line() {
    let _guard = globals();
    let g = Grid::of(4, &["abcd\\", "efg", "hz"]);
    let mut gr = g.reader(0, 0);
    assert_eq!(jump(&mut gr, "f"), (1, (1, 1)));
    assert_eq!(
        jump(&mut gr, "z"),
        (0, (1, 1)),
        "the run of wrapped lines ends first"
    );

    let mut back = g.reader(2, 1);
    assert_eq!(jump_back(&mut back, "b"), (1, (1, 0)));
    let mut over = g.reader(1, 2);
    assert_eq!(
        jump_back(&mut over, "a"),
        (0, (1, 2)),
        "the line above does not wrap onto this one"
    );
}

#[test]
fn a_jump_from_outside_the_grid_finds_nothing() {
    let _guard = globals();
    let g = Grid::of(10, &["abc"]);
    let mut past = g.reader(0, 5);
    assert_eq!(jump(&mut past, "a"), (0, (0, 5)));
    let mut before = g.reader(0, u_int::MAX);
    assert_eq!(jump_back(&mut before, "a"), (0, (0, u_int::MAX)));
}

#[test]
fn a_jump_matches_a_tab_and_never_padding() {
    let _guard = globals();
    let g = Grid::of(10, &["ab"]);
    g.tab(2, 0, 4);
    g.write_after(6, 0, "c");
    let mut gr = g.reader(0, 0);
    assert_eq!(jump(&mut gr, "\t"), (1, (2, 0)), "the tab cell itself");

    let wide = Grid::of(10, &["a"]);
    wide.wide(1, 0);
    let mut over = wide.reader(0, 0);
    assert_eq!(
        jump(&mut over, "\u{4e2d}"),
        (1, (1, 0)),
        "a wide character is matched by all of its bytes"
    );
    let mut none = wide.reader(2, 0);
    assert_eq!(
        jump(&mut none, "!"),
        (0, (2, 0)),
        "padding is never a match"
    );
}

#[test]
fn the_cursor_goes_back_to_the_first_thing_on_the_line() {
    let _guard = globals();
    let g = Grid::of(10, &["   abc", "      ", "  de\\", "fg"]);
    let mut gr = g.reader(5, 0);
    grid_reader_cursor_back_to_indentation(&mut gr);
    assert_eq!(cursor(&gr), (3, 0));

    let mut blank = g.reader(4, 1);
    grid_reader_cursor_back_to_indentation(&mut blank);
    assert_eq!(cursor(&blank), (4, 1), "a blank line leaves it alone");

    let mut wrapped = g.reader(1, 3);
    grid_reader_cursor_back_to_indentation(&mut wrapped);
    assert_eq!(
        cursor(&wrapped),
        (2, 2),
        "the run of wrapped lines starts on the line above"
    );
}

#[test]
fn indentation_can_run_onto_the_next_line_of_a_wrapped_run() {
    let _guard = globals();
    let g = Grid::of(4, &["   \\", "ab"]);
    let mut gr = g.reader(1, 1);
    grid_reader_cursor_back_to_indentation(&mut gr);
    assert_eq!(cursor(&gr), (0, 1));
}

#[test]
fn indentation_of_tabs_is_walked_over_too() {
    let _guard = globals();
    let g = Grid::of(10, &[""]);
    g.tab(0, 0, 4);
    g.write_after(4, 0, "ab");
    let mut gr = g.reader(5, 0);
    grid_reader_cursor_back_to_indentation(&mut gr);
    assert_eq!(cursor(&gr), (4, 0));
}

use super::*;
use crate::grid::{
    GRID_FLAG_PADDING, grid_create, grid_default_cell, grid_peek_line, grid_scroll_history,
};
use crate::tests::test_fixtures::globals;

/// A grid that frees itself at the end of the test.
struct Grid(Box<grid>);

impl Grid {
    fn new(sx: u_int, sy: u_int, hlimit: u_int) -> Grid {
        Grid(grid_create(sx, sy, hlimit))
    }

    fn ptr(&self) -> *mut grid {
        self.0.as_ref() as *const grid as *mut grid
    }

    /// Writes `s` from (px, py) of the screen, one cell per byte.
    fn write(&self, px: u_int, py: u_int, s: &str) {
        let gc = unsafe { grid_default_cell };
        unsafe { grid_view_set_cells(&mut *self.ptr(), px, py, &gc, s.as_bytes()) };
    }

    /// The text of one screen line.
    fn text(&self, py: u_int) -> String {
        unsafe {
            let p = grid_view_string_cells(&*self.ptr(), 0, py, 1000);

            p.to_string_lossy().into_owned()
        }
    }

    /// The text of one line of the whole grid, history included.
    fn history_text(&self, py: u_int) -> String {
        unsafe {
            let hsize = (*self.ptr()).hsize;
            let p = grid_view_string_cells(&*self.ptr(), 0, py.wrapping_sub(hsize), 1000);

            p.to_string_lossy().into_owned()
        }
    }

    /// The screen as text, one string per line.
    fn screen(&self) -> Vec<String> {
        unsafe { (0..(*self.ptr()).sy).map(|py| self.text(py)).collect() }
    }
}

impl ::core::ops::Deref for Grid {
    type Target = grid;

    fn deref(&self) -> &grid {
        &self.0
    }
}

/// A grid whose screen sits on top of `history` lines of history.
fn with_history(sx: u_int, sy: u_int, history: u_int) -> Grid {
    let g = Grid::new(sx, sy, 100);
    for i in 0..history {
        g.write(0, 0, &format!("h{i}"));
        unsafe { grid_scroll_history(&mut *g.ptr(), 8) };
    }
    g
}

#[test]
fn the_screen_starts_where_the_history_ends() {
    let _guard = globals();
    let g = with_history(10, 2, 3);
    assert_eq!(g.hsize, 3);
    g.write(0, 1, "abc");
    assert_eq!(g.text(1), "abc");
    assert_eq!(g.history_text(4), "abc");
    assert_eq!(g.history_text(0), "h0");

    let mut gc = unsafe { grid_default_cell };
    unsafe { gc = grid_view_get_cell(&*g.ptr(), 1, 1) };
    assert_eq!(gc.data.data[0], b'b');
}

#[test]
fn one_cell_and_one_padding_cell_can_be_set_through_the_view() {
    let _guard = globals();
    let g = with_history(10, 2, 2);
    let mut gc = unsafe { grid_default_cell };
    gc.data.data[0] = b'z';
    unsafe {
        grid_view_set_cell(&mut *g.ptr(), 0, 0, &gc);
        grid_view_set_padding(&mut *g.ptr(), 1, 0);
    }
    assert_eq!(g.text(0), "z", "the padding cell is left out");
    let mut read = unsafe { grid_default_cell };
    unsafe { read = grid_view_get_cell(&*g.ptr(), 1, 0) };
    assert_eq!(
        read.flags as ::core::ffi::c_int & GRID_FLAG_PADDING,
        GRID_FLAG_PADDING
    );
}

#[test]
fn clearing_the_view_leaves_the_history_alone() {
    let _guard = globals();
    let g = with_history(10, 2, 2);
    g.write(0, 0, "abc");
    g.write(0, 1, "def");
    unsafe { grid_view_clear(&mut *g.ptr(), 0, 0, 10, 1, 8) };
    assert_eq!(g.screen(), ["", "def"]);
    assert_eq!(g.history_text(0), "h0");
}

#[test]
fn clearing_the_history_scrolls_the_used_lines_into_it() {
    let _guard = globals();
    let g = Grid::new(10, 4, 100);
    g.write(0, 0, "one");
    g.write(0, 1, "two");
    unsafe { grid_view_clear_history(&mut *g.ptr(), 8) };
    assert_eq!(g.hsize, 2, "only the lines that had something in them");
    assert_eq!(g.hscrolled, 0);
    assert_eq!(g.history_text(0), "one");
    assert_eq!(g.history_text(1), "two");
    assert_eq!(g.screen(), ["", "", "", ""]);
}

#[test]
fn clearing_the_history_of_a_full_screen_leaves_nothing_to_clear() {
    let _guard = globals();
    let g = Grid::new(10, 2, 100);
    g.write(0, 0, "one");
    g.write(0, 1, "two");
    unsafe { grid_view_clear_history(&mut *g.ptr(), 8) };
    assert_eq!(g.hsize, 2);
    assert_eq!(g.screen(), ["", ""]);
}

#[test]
fn clearing_the_history_of_an_empty_screen_only_clears_it() {
    let _guard = globals();
    let g = with_history(10, 2, 2);
    unsafe { grid_view_clear_history(&mut *g.ptr(), 8) };
    assert_eq!(g.hsize, 2, "nothing was scrolled");
    assert_eq!(g.screen(), ["", ""]);
}

#[test]
fn a_full_region_scrolls_into_the_history() {
    let _guard = globals();
    let g = Grid::new(10, 3, 100);
    g.write(0, 0, "one");
    g.write(0, 1, "two");
    unsafe { grid_view_scroll_region_up(&mut *g.ptr(), 0, 2, 8) };
    assert_eq!(g.hsize, 1);
    assert_eq!(g.history_text(0), "one");
    assert_eq!(g.screen(), ["two", "", ""]);
}

#[test]
fn part_of_the_screen_scrolls_into_the_history_as_a_region() {
    let _guard = globals();
    let g = Grid::new(10, 3, 100);
    g.write(0, 0, "one");
    g.write(0, 1, "two");
    g.write(0, 2, "three");
    unsafe { grid_view_scroll_region_up(&mut *g.ptr(), 0, 1, 8) };
    assert_eq!(g.hsize, 1);
    assert_eq!(g.history_text(0), "one");
    assert_eq!(g.screen(), ["two", "", "three"]);
}

#[test]
fn a_grid_without_history_just_moves_its_lines_up() {
    let _guard = globals();
    let g = Grid::new(10, 3, 0);
    g.write(0, 0, "one");
    g.write(0, 1, "two");
    g.write(0, 2, "three");
    unsafe { grid_view_scroll_region_up(&mut *g.ptr(), 0, 2, 8) };
    assert_eq!(g.hsize, 0);
    assert_eq!(g.screen(), ["two", "three", ""]);
}

#[test]
fn a_region_scrolls_down_by_moving_its_lines() {
    let _guard = globals();
    let g = with_history(10, 3, 1);
    g.write(0, 0, "one");
    g.write(0, 1, "two");
    g.write(0, 2, "three");
    unsafe { grid_view_scroll_region_down(&mut *g.ptr(), 0, 2, 8) };
    assert_eq!(g.screen(), ["", "one", "two"]);
    assert_eq!(g.history_text(0), "h0");
}

#[test]
fn lines_are_inserted_by_pushing_the_ones_below_them_down() {
    let _guard = globals();
    let g = with_history(10, 3, 1);
    g.write(0, 0, "one");
    g.write(0, 1, "two");
    g.write(0, 2, "three");
    unsafe { grid_view_insert_lines(&mut *g.ptr(), 1, 1, 8) };
    assert_eq!(g.screen(), ["one", "", "two"]);
}

#[test]
fn lines_are_inserted_inside_a_region_without_touching_the_rest() {
    let _guard = globals();
    let g = with_history(10, 4, 1);
    for (py, text) in ["one", "two", "three", "four"].iter().enumerate() {
        g.write(0, py as u_int, text);
    }
    unsafe { grid_view_insert_lines_region(&mut *g.ptr(), 2, 1, 1, 8) };
    assert_eq!(g.screen(), ["one", "", "two", "four"]);
}

#[test]
fn inserting_more_lines_than_the_region_holds_clears_what_is_left() {
    let _guard = globals();
    let g = with_history(10, 4, 1);
    for (py, text) in ["one", "two", "three", "four"].iter().enumerate() {
        g.write(0, py as u_int, text);
    }
    unsafe { grid_view_insert_lines_region(&mut *g.ptr(), 2, 1, 2, 8) };
    assert_eq!(g.screen(), ["one", "", "", "four"]);
}

#[test]
fn lines_are_deleted_by_pulling_the_ones_below_them_up() {
    let _guard = globals();
    let g = with_history(10, 3, 1);
    g.write(0, 0, "one");
    g.write(0, 1, "two");
    g.write(0, 2, "three");
    unsafe { grid_view_delete_lines(&mut *g.ptr(), 0, 1, 8) };
    assert_eq!(g.screen(), ["two", "three", ""]);
}

#[test]
fn lines_are_deleted_inside_a_region_without_touching_the_rest() {
    let _guard = globals();
    let g = with_history(10, 4, 1);
    for (py, text) in ["one", "two", "three", "four"].iter().enumerate() {
        g.write(0, py as u_int, text);
    }
    unsafe { grid_view_delete_lines_region(&mut *g.ptr(), 2, 0, 1, 8) };
    assert_eq!(g.screen(), ["two", "three", "", "four"]);
}

#[test]
fn cells_are_inserted_by_pushing_the_rest_of_the_line_along() {
    let _guard = globals();
    let g = with_history(10, 2, 1);
    g.write(0, 0, "abcdef");
    unsafe { grid_view_insert_cells(&mut *g.ptr(), 2, 0, 2, 8) };
    assert_eq!(
        g.text(0),
        "ab  cdef  ",
        "the line was filled out to the width"
    );
}

#[test]
fn inserting_cells_at_the_end_of_a_line_only_clears_the_last_one() {
    let _guard = globals();
    let g = with_history(10, 2, 1);
    g.write(0, 0, "abcdefghij");
    unsafe { grid_view_insert_cells(&mut *g.ptr(), 9, 0, 2, 8) };
    assert_eq!(g.text(0), "abcdefghi ");
}

#[test]
fn cells_are_deleted_by_pulling_the_rest_of_the_line_back() {
    let _guard = globals();
    let g = with_history(10, 2, 1);
    g.write(0, 0, "abcdef");
    unsafe { grid_view_delete_cells(&mut *g.ptr(), 1, 0, 2, 8) };
    assert_eq!(g.text(0), "adef    ");
    assert_eq!(
        unsafe {
            grid_peek_line(&*g.ptr(), (*g.ptr()).hsize)
                .expect("the line is there")
                .cellused
        },
        8,
        "the cells that moved back leave the used width where it was"
    );
}

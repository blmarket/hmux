//! Coverage for [`crate::grid`] — edge cases around cell storage,
//! line movement, history bookkeeping, reflow and string export.
//!
//! `grid.rs` is relatively high coverage; these tests pin boundary
//! behaviour that is easy to regress — default-cell round-trips,
//! out-of-range reads, zero-size clears, overlapping moves, duplicate
//! lines, history counters and reflow — without spawning a server.
//! Every test builds a [`Grid`] or [`Screen`] fixture and holds
//! [`globals`] where global state might be touched.

use crate::grid::{
    GRID_FLAG_PADDING, GRID_HISTORY, GRID_STRING_EMPTY_CELLS, GRID_STRING_TRIM_SPACES,
    grid_cells_equal, grid_cells_look_equal, grid_clear, grid_clear_history, grid_clear_lines,
    grid_collect_history, grid_compare, grid_create, grid_default_cell, grid_destroy,
    grid_duplicate_lines, grid_get_cell, grid_move_cells, grid_move_lines, grid_reflow,
    grid_remove_history, grid_scroll_history, grid_set_cell, grid_set_cells, grid_set_padding,
    grid_string_cells,
};
use crate::tests::test_fixtures::{Grid, Screen, ascii, globals};
use ::core::ptr::null_mut;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

unsafe fn line_text(gd: *mut crate::types::grid, py: u32) -> String {
    unsafe {
        let p = grid_string_cells(&*gd, 0, py, 100, None, 0, null_mut());
        p.to_string_lossy().into_owned()
    }
}

unsafe fn text_with_flags(gd: *mut crate::types::grid, py: u32, nx: u32, flags: i32) -> String {
    unsafe {
        let p = grid_string_cells(&*gd, 0, py, nx, None, flags, null_mut());
        p.to_string_lossy().into_owned()
    }
}

// ---------------------------------------------------------------------------
// dimensions / default cell
// ---------------------------------------------------------------------------

#[test]
fn grid_create_dimensions_and_flags() {
    let _guard = globals();
    let g = Grid::new(10, 5, 100);
    unsafe {
        assert_eq!((*g.ptr()).sx, 10);
        assert_eq!((*g.ptr()).sy, 5);
        assert_eq!((*g.ptr()).hlimit, 100);
        assert_eq!((*g.ptr()).hsize, 0);
        assert_eq!((*g.ptr()).hscrolled, 0);
        assert_ne!((*g.ptr()).flags & GRID_HISTORY, 0);
        assert!(!(*g.ptr()).linedata.is_empty());
    }
    let plain = Grid::new(8, 4, 0);
    unsafe {
        assert_eq!((*plain.ptr()).flags & GRID_HISTORY, 0);
        assert_eq!((*plain.ptr()).hlimit, 0);
    }
    // default cell is a single space
    unsafe {
        let dgc = grid_default_cell;
        assert_eq!(dgc.data.data[0], b' ');
        assert_eq!(dgc.data.size, 1);
        assert_eq!(dgc.data.width, 1);
    }
}

#[test]
fn grid_get_cell_out_of_range_returns_default() {
    let _guard = globals();
    let g = Grid::new(4, 3, 0);
    g.write(0, 0, "hi");
    unsafe {
        let mut gc = grid_default_cell;
        gc = grid_get_cell(&*g.ptr(), 100, 0);
        assert_eq!(gc.data.data[0], b' ');
        // out-of-range y
        gc = grid_get_cell(&*g.ptr(), 0, 99);
        assert_eq!(gc.data.data[0], b' ');
        // unwritten cell in range still default
        gc = grid_get_cell(&*g.ptr(), 3, 0);
        assert_eq!(gc.data.data[0], b' ');
    }
}

// ---------------------------------------------------------------------------
// compare / equality
// ---------------------------------------------------------------------------

#[test]
fn grid_compare_equal_and_unequal() {
    let _guard = globals();
    let a = Grid::new(6, 2, 0);
    let b = Grid::new(6, 2, 0);
    unsafe {
        assert_eq!(grid_compare(&*a.ptr(), &*b.ptr()), 0);
        assert_eq!(grid_compare(&*b.ptr(), &*a.ptr()), 0);
    }
    a.write(0, 0, "hello");
    unsafe {
        assert_ne!(grid_compare(&*a.ptr(), &*b.ptr()), 0);
    }
    b.write(0, 0, "hello");
    unsafe {
        assert_eq!(grid_compare(&*a.ptr(), &*b.ptr()), 0);
    }
    b.write(0, 1, "x");
    unsafe {
        assert_ne!(grid_compare(&*a.ptr(), &*b.ptr()), 0);
    }
    // different dimensions compare as not equal
    let wide = Grid::new(7, 2, 0);
    unsafe {
        assert_ne!(grid_compare(&*a.ptr(), &*wide.ptr()), 0);
    }
    let tall = Grid::new(6, 3, 0);
    unsafe {
        assert_ne!(grid_compare(&*a.ptr(), &*tall.ptr()), 0);
    }
}

#[test]
fn grid_cells_equal_and_look_equal_distinguish_style_vs_content() {
    let _guard = globals();
    let mut gc1 = ascii(b'A');
    let mut gc2 = ascii(b'A');
    unsafe {
        assert_ne!(grid_cells_equal(&gc1, &gc2), 0);
        assert_ne!(grid_cells_look_equal(&gc1, &gc2), 0);
    }
    gc2.data.data[0] = b'B';
    unsafe {
        assert_eq!(grid_cells_equal(&gc1, &gc2), 0);
        // look-equal still true when fg/bg/attr equal despite byte change?
        // look_equal ignores size/width/content width but does compare bytes? No,
        // look_equal only compares fg/bg/attr/flags/link — so it stays 1.
        assert_ne!(grid_cells_look_equal(&gc1, &gc2), 0);
    }
    gc2 = ascii(b'A');
    gc2.fg = 1;
    unsafe {
        assert_eq!(grid_cells_look_equal(&gc1, &gc2), 0);
        assert_eq!(grid_cells_equal(&gc1, &gc2), 0);
    }
}

// ---------------------------------------------------------------------------
// clear / set_cells / padding
// ---------------------------------------------------------------------------

#[test]
fn grid_clear_and_clear_lines_edge_cases() {
    let _guard = globals();
    let g = Grid::new(10, 4, 0);
    g.write(0, 0, "abcdefghij");
    g.write(0, 1, "0123456789");
    unsafe {
        // zero-size clear is a no-op
        grid_clear(&mut *g.ptr(), 0, 0, 0, 1, 8);
        assert_eq!(line_text(g.ptr(), 0), "abcdefghij");
        grid_clear(&mut *g.ptr(), 0, 0, 5, 0, 8);
        assert_eq!(line_text(g.ptr(), 0), "abcdefghij");
        // clear middle of a line
        grid_clear(&mut *g.ptr(), 2, 0, 3, 1, 8);
        assert_eq!(line_text(g.ptr(), 0), "ab   fghij");
        // clear whole lines via fast path (px==0 && nx==sx)
        grid_clear(&mut *g.ptr(), 0, 1, 10, 1, 8);
        assert_eq!(line_text(g.ptr(), 1), "");
        // clear_lines directly
        g.write(0, 2, "xyz");
        assert_eq!(line_text(g.ptr(), 2), "xyz");
        grid_clear_lines(&mut *g.ptr(), 2, 1, 8);
        assert_eq!(line_text(g.ptr(), 2), "");
        // zero-line clear_lines is a no-op
        grid_clear_lines(&mut *g.ptr(), 0, 0, 8);
        assert_eq!(line_text(g.ptr(), 0), "ab   fghij");
    }
}

#[test]
fn grid_set_cells_and_padding_round_trip() {
    let _guard = globals();
    let g = Grid::new(10, 2, 0);
    unsafe {
        let mut gc = grid_default_cell;
        gc.fg = 2;
        // set_cells writes run of bytes sharing one style
        grid_set_cells(&mut *g.ptr(), 0, 0, &gc, b"hello");
        assert_eq!(line_text(g.ptr(), 0), "hello");
        let mut out = grid_default_cell;
        out = grid_get_cell(&*g.ptr(), 2, 0);
        assert_eq!(out.data.data[0], b'l');
        assert_eq!(out.fg, 2);
        // padding cell is a distinct flag
        grid_set_padding(&mut *g.ptr(), 5, 0);
        let mut pad = grid_default_cell;
        pad = grid_get_cell(&*g.ptr(), 5, 0);
        assert_ne!(pad.flags as i32 & GRID_FLAG_PADDING, 0);
        // grid_string_cells skips padding by default
        let s = line_text(g.ptr(), 0);
        assert_eq!(s, "hello", "padding is invisible to string_cells");
    }
}

// ---------------------------------------------------------------------------
// move_cells / move_lines / duplicate_lines
// ---------------------------------------------------------------------------

#[test]
fn grid_move_cells_and_move_lines() {
    let _guard = globals();
    let g = Grid::new(10, 6, 0);
    g.write(0, 0, "abcdefghij");
    g.write(0, 1, "111");
    g.write(0, 2, "222");
    g.write(0, 3, "333");
    unsafe {
        // move cells right within a line
        grid_move_cells(&mut *g.ptr(), 3, 0, 0, 3, 8);
        // 0..3 ("abc") moved to 3..6, source cleared to spaces
        let mut gc = grid_default_cell;
        gc = grid_get_cell(&*g.ptr(), 0, 0);
        assert_eq!(gc.data.data[0], b' ');
        gc = grid_get_cell(&*g.ptr(), 3, 0);
        assert_eq!(gc.data.data[0], b'a');
        gc = grid_get_cell(&*g.ptr(), 5, 0);
        assert_eq!(gc.data.data[0], b'c');

        // move lines down
        grid_move_lines(&mut *g.ptr(), 4, 1, 2, 8);
        assert_eq!(line_text(g.ptr(), 4), "111");
        assert_eq!(line_text(g.ptr(), 5), "222");
        // source lines were emptied
        assert_eq!(line_text(g.ptr(), 1), "");
        assert_eq!(line_text(g.ptr(), 2), "");

        // duplicate_lines copies without freeing source
        let dst = Grid::new(10, 6, 0);
        grid_duplicate_lines(&mut *dst.ptr(), 0, &*g.ptr(), 0, 6);
        assert_eq!(line_text(dst.ptr(), 4), "111");
        assert_eq!(line_text(g.ptr(), 4), "111");
    }
}

// ---------------------------------------------------------------------------
// history and reflow via Screen fixture
// ---------------------------------------------------------------------------

#[test]
fn grid_history_scroll_and_remove_and_collect() {
    let _guard = globals();
    let g = Grid::new(10, 2, 100);
    g.write(0, 0, "first");
    g.write(0, 1, "second");
    unsafe {
        assert_eq!((*g.ptr()).hsize, 0);
        grid_scroll_history(&mut *g.ptr(), 8);
        assert_eq!((*g.ptr()).hsize, 1);
        assert_eq!((*g.ptr()).hscrolled, 1);
        // after scroll, history line 0 holds the old line 0
        assert_eq!(line_text(g.ptr(), 0), "first");
        grid_remove_history(&mut *g.ptr(), 1);
        assert_eq!((*g.ptr()).hsize, 0);
        // ask to remove more than exists is a no-op
        grid_remove_history(&mut *g.ptr(), 99);
        assert_eq!((*g.ptr()).hsize, 0);
        // fill history to limit and collect
        for _ in 0..10 {
            grid_scroll_history(&mut *g.ptr(), 8);
        }
        assert!((*g.ptr()).hsize <= 10);
        grid_collect_history(&mut *g.ptr(), 0);
        // collect with all=1 drops everything over limit; here nothing over
        // so hsize may stay or shrink by ~hlimit/10 if at limit — just check no panic
        assert!((*g.ptr()).hsize <= 10);
        grid_clear_history(&mut *g.ptr());
        assert_eq!((*g.ptr()).hsize, 0);
        assert_eq!((*g.ptr()).hscrolled, 0);
    }
}

#[test]
fn grid_reflow_via_screen_fixture_preserves_content() {
    let _guard = globals();
    let mut s = Screen::new(10, 3, 100);
    unsafe {
        // write a long unwrapped line then reflow narrower
        let mut gc = grid_default_cell;
        for (i, b) in b"abcdefghij".iter().enumerate() {
            gc.data.data[0] = *b;
            grid_set_cell(&mut *s.grid(), i as u32, (*s.grid()).hsize, &gc);
        }
        // initial: one screen line used
        assert_eq!(line_text(s.grid(), (*s.grid()).hsize), "abcdefghij");
        grid_reflow(&mut *s.grid(), 5);
        // width reduced to 5: line should have split and be wrapped
        let total = (*s.grid()).hsize + (*s.grid()).sy;
        let mut all = String::new();
        for py in 0..total {
            all.push_str(&line_text(s.grid(), py));
        }
        let compact: String = all.chars().filter(|c| *c != ' ').collect();
        assert!(
            compact.contains("abcdefghij"),
            "reflow kept bytes, got {all:?} compact {compact:?} total {total} hsize {}",
            (*s.grid()).hsize
        );
        // reflow back wider should join again
        grid_reflow(&mut *s.grid(), 10);
        let total2 = (*s.grid()).hsize + (*s.grid()).sy;
        let mut all2 = String::new();
        for py in 0..total2 {
            all2.push_str(&line_text(s.grid(), py));
        }
        let compact2: String = all2.chars().filter(|c| *c != ' ').collect();
        assert!(
            compact2.contains("abcdefghij"),
            "re-join kept bytes, got {all2:?}"
        );
    }
}

#[test]
fn grid_string_cells_trim_and_empty_flags() {
    let _guard = globals();
    let g = Grid::new(10, 1, 0);
    g.write(0, 0, "hi  ");
    unsafe {
        // default includes trailing spaces as stored, but TRIM_SPACES drops them
        let full = text_with_flags(g.ptr(), 0, 10, 0);
        assert!(full.starts_with("hi"), "{full:?}");
        let trimmed = text_with_flags(g.ptr(), 0, 10, GRID_STRING_TRIM_SPACES);
        assert_eq!(trimmed, "hi");
        // EMPTY_CELLS pads to cellsize even beyond cellused
        let empty_padded = text_with_flags(g.ptr(), 0, 10, GRID_STRING_EMPTY_CELLS);
        assert!(empty_padded.len() >= full.len());
        let grid = grid_create(5, 1, 0);
        grid_destroy(grid);
    }
}

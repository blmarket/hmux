use super::store::{
    Grid, grid_create, grid_default_cell, grid_get_cell, grid_get_line, grid_in_set,
    grid_line_length, grid_peek_line, grid_set_cell, grid_set_cells, grid_set_padding,
    grid_set_tab,
};
pub use crate::consts::{GRID_FLAG_PADDING, GRID_FLAG_TAB, GRID_LINE_WRAPPED};
pub use crate::types::*;
use ::core::ffi::{CStr, c_int};

/// The characters the word commands count as space.
const WHITESPACE: &CStr = c"\t ";

/// Cursor navigation over a terminal grid.
pub trait GridReader {
    /// Return the current cursor position as `(column, line)`.
    fn cursor(&self) -> (u_int, u_int);

    /// Return the length of the line containing the cursor.
    fn line_length(&self) -> u_int;

    /// Return whether the cell under the cursor belongs to `set`.
    fn in_set(&self, set: &CStr) -> bool;

    /// Move right by one character, optionally crossing a line boundary.
    fn right(&mut self, wrap: bool, all: bool, one_more: bool);

    /// Move left by one character, optionally crossing a line boundary.
    fn left(&mut self, wrap: bool);

    /// Move down by one line.
    fn down(&mut self);

    /// Move up by one line.
    fn up(&mut self);

    /// Move to the start of this line or its wrapped run.
    fn start_of_line(&mut self, wrap: bool);

    /// Move to the end of this line or its wrapped run.
    fn end_of_line(&mut self, wrap: bool, all: bool);

    /// Move to the beginning of the next word.
    fn next_word(&mut self, separators: &CStr);

    /// Move to the end of the next word.
    fn next_word_end(&mut self, separators: &CStr);

    /// Move to the beginning of the previous word.
    fn previous_word(&mut self, separators: &CStr, already: bool, stop_at_eol: bool);

    /// Jump forward to `character`, returning whether it was found.
    fn jump(&mut self, character: &[u8]) -> bool;

    /// Jump backward to `character`, returning whether it was found.
    fn jump_back(&mut self, character: &[u8]) -> bool;

    /// Move to the first nonblank character in this wrapped line.
    fn back_to_indentation(&mut self);
}

/// The Rust grid-reader implementation.
pub struct RustGridReader<'a> {
    gd: &'a grid,
    cx: u_int,
    cy: u_int,
}

/// A grid fixture whose representation stays behind the reader boundary.
pub struct RustGrid(Box<grid>);

impl RustGrid {
    /// Builds an empty grid with the given visible size and history limit.
    pub fn new(sx: u_int, sy: u_int, hlimit: u_int) -> Self {
        Self(grid_create(sx, sy, hlimit))
    }

    /// Build a grid from lines, treating a trailing backslash as a wrap marker.
    pub fn from_lines(sx: u_int, lines: &[&str]) -> Self {
        let mut grid = Self::new(sx, lines.len() as u_int, 0);
        for (py, line) in lines.iter().enumerate() {
            let (text, wrapped) = match line.strip_suffix('\\') {
                Some(text) => (text, true),
                None => (*line, false),
            };
            grid.write(0, py as u_int, text);
            if wrapped {
                grid_get_line(&mut grid.0, py as u_int).flags |= GRID_LINE_WRAPPED;
            }
        }
        grid
    }

    /// Start a reader at `(cx, cy)`.
    pub fn reader(&self, cx: u_int, cy: u_int) -> RustGridReader<'_> {
        RustGridReader::start(&self.0, cx, cy)
    }

    /// Write text beginning at `(px, py)`.
    pub fn write(&mut self, px: u_int, py: u_int, text: &str) {
        grid_set_cells(&mut self.0, px, py, &grid_default_cell, text.as_bytes());
    }

    /// Write a two-column character and its padding beginning at `(px, py)`.
    pub fn write_wide(&mut self, px: u_int, py: u_int, character: char) {
        let mut cell = grid_default_cell;
        let mut bytes = [0; 4];
        let encoded = character.encode_utf8(&mut bytes).as_bytes();
        cell.data.data[..encoded.len()].copy_from_slice(encoded);
        cell.data.have = encoded.len() as u8;
        cell.data.size = cell.data.have;
        cell.data.width = 2;
        grid_set_cell(&mut self.0, px, py, &cell);
        grid_set_padding(&mut self.0, px + 1, py);
    }

    /// Write a tab and its padding beginning at `(px, py)`.
    pub fn write_tab(&mut self, px: u_int, py: u_int, width: u_int) {
        let mut cell = grid_default_cell;
        grid_set_tab(&mut cell, width);
        grid_set_cell(&mut self.0, px, py, &cell);
        for offset in 1..width {
            grid_set_padding(&mut self.0, px + offset, py);
        }
    }
}

impl Grid for RustGrid {
    type Cell = grid_cell;
    type Screen = crate::screen::RustScreen;

    fn dimensions(&self) -> (u_int, u_int, u_int, u_int) {
        self.0.dimensions()
    }

    fn content_eq(&self, other: &Self) -> bool {
        self.0.content_eq(&other.0)
    }

    fn cell(&self, px: u_int, py: u_int) -> Self::Cell {
        self.0.cell(px, py)
    }

    fn set_cell(&mut self, px: u_int, py: u_int, cell: &Self::Cell) {
        self.0.set_cell(px, py, cell)
    }

    fn set_padding(&mut self, px: u_int, py: u_int) {
        self.0.set_padding(px, py)
    }

    fn set_cells(&mut self, px: u_int, py: u_int, cell: &Self::Cell, text: &[u8]) {
        self.0.set_cells(px, py, cell, text)
    }

    fn clear(&mut self, px: u_int, py: u_int, nx: u_int, ny: u_int, background: u_int) {
        self.0.clear(px, py, nx, ny, background)
    }

    fn clear_lines(&mut self, py: u_int, ny: u_int, background: u_int) {
        self.0.clear_lines(py, ny, background)
    }

    fn move_lines(&mut self, dy: u_int, py: u_int, ny: u_int, background: u_int) {
        self.0.move_lines(dy, py, ny, background)
    }

    fn move_cells(&mut self, dx: u_int, px: u_int, py: u_int, nx: u_int, background: u_int) {
        self.0.move_cells(dx, px, py, nx, background)
    }

    fn collect_history(&mut self, all: bool) {
        self.0.collect_history(all)
    }

    fn remove_history(&mut self, lines: u_int) {
        self.0.remove_history(lines)
    }

    fn clear_history(&mut self) {
        self.0.clear_history()
    }

    fn scroll_history(&mut self, background: u_int) {
        self.0.scroll_history(background)
    }

    fn scroll_history_region(&mut self, upper: u_int, lower: u_int, background: u_int) {
        self.0.scroll_history_region(upper, lower, background)
    }

    fn duplicate_lines(&mut self, dy: u_int, source: &Self, sy: u_int, ny: u_int) {
        self.0.duplicate_lines(dy, &source.0, sy, ny)
    }

    fn reflow(&mut self, width: u_int) {
        self.0.reflow(width)
    }

    fn wrap_position(&self, px: u_int, py: u_int) -> (u_int, u_int) {
        self.0.wrap_position(px, py)
    }

    fn unwrap_position(&self, wx: u_int, wy: u_int) -> (u_int, u_int) {
        self.0.unwrap_position(wx, wy)
    }

    fn line_length(&self, py: u_int) -> u_int {
        self.0.line_length(py)
    }

    fn in_set(&self, px: u_int, py: u_int, set: &CStr) -> c_int {
        self.0.in_set(px, py, set)
    }

    fn string_cells(
        &self,
        px: u_int,
        py: u_int,
        nx: u_int,
        last_cell: Option<&mut Self::Cell>,
        flags: c_int,
        screen: Option<&Self::Screen>,
    ) -> std::ffi::CString {
        self.0.string_cells(px, py, nx, last_cell, flags, screen)
    }
}

impl<'a> RustGridReader<'a> {
    /// Start a reader at `(cx, cy)` in `grid`.
    pub(crate) fn start(grid: &'a grid, cx: u_int, cy: u_int) -> Self {
        Self { gd: grid, cx, cy }
    }
}

/// The last line of the grid.
fn bottom(gr: &RustGridReader<'_>) -> u_int {
    gr.gd.hsize.wrapping_add(gr.gd.sy).wrapping_sub(1)
}

/// Whether a line carries on onto the one below it.
fn wrapped(gr: &RustGridReader<'_>, py: u_int) -> bool {
    line_flags(gr.gd, py) & GRID_LINE_WRAPPED != 0
}

/// The flags of the line at `py`.
fn line_flags(gd: &grid, py: u_int) -> c_int {
    grid_peek_line(gd, py).map_or(0, |gl| gl.flags)
}

/// Whether a cell is the padding of a wider one in front of it.
fn padding(gr: &RustGridReader<'_>, px: u_int, py: u_int) -> bool {
    let gc = grid_get_cell(gr.gd, px, py);
    gc.flags as c_int & GRID_FLAG_PADDING != 0
}

/// How far along its line a walk can go: a line that wraps runs to the last
/// column of the grid, one that does not to the end of its text.
fn walk_end(gr: &RustGridReader<'_>) -> u_int {
    if wrapped(gr, gr.cy) {
        gr.gd.sx.wrapping_sub(1)
    } else {
        grid_reader_line_length(gr)
    }
}

/// What the set says about the cell under the cursor: zero when the cell is
/// not in it, and for a tab the number of its columns still to come.
fn in_set(gr: &RustGridReader<'_>, set: &CStr) -> c_int {
    grid_reader_in_set(gr, set)
}

/// Move the cursor back off any padding it is sitting on.
fn off_padding(gr: &mut RustGridReader<'_>) {
    while gr.cx > 0 && padding(gr, gr.cx, gr.cy) {
        gr.cx -= 1;
    }
}

fn grid_reader_get_cursor(gr: &RustGridReader<'_>) -> (u_int, u_int) {
    (gr.cx, gr.cy)
}

fn grid_reader_line_length(gr: &RustGridReader<'_>) -> u_int {
    grid_line_length(gr.gd, gr.cy)
}

/// Move the cursor right, onto the next line if it is at the end of this one
/// and asked to wrap.
fn grid_reader_cursor_right(gr: &mut RustGridReader<'_>, wrap: c_int, all: c_int, onemore: c_int) {
    let px = if all != 0 {
        gr.gd.sx
    } else {
        let length = grid_reader_line_length(gr);
        if onemore != 0 || length == 0 {
            length
        } else {
            length - 1
        }
    };
    if wrap != 0 && gr.cx >= px && gr.cy < bottom(gr) {
        grid_reader_cursor_start_of_line(gr, 0);
        grid_reader_cursor_down(gr);
    } else if gr.cx < px {
        gr.cx += 1;
        while gr.cx < px && padding(gr, gr.cx, gr.cy) {
            gr.cx += 1;
        }
    }
}

/// Move the cursor left, onto the line above when it is at the start of this
/// one and that line carries on onto it.
fn grid_reader_cursor_left(gr: &mut RustGridReader<'_>, wrap: c_int) {
    off_padding(gr);
    if gr.cx == 0 && gr.cy > 0 && (wrap != 0 || wrapped(gr, gr.cy - 1)) {
        grid_reader_cursor_up(gr);
        grid_reader_cursor_end_of_line(gr, 0, 0);
    } else if gr.cx > 0 {
        gr.cx -= 1;
    }
}

fn grid_reader_cursor_down(gr: &mut RustGridReader<'_>) {
    if gr.cy < bottom(gr) {
        gr.cy += 1;
    }
    off_padding(gr);
}

fn grid_reader_cursor_up(gr: &mut RustGridReader<'_>) {
    if gr.cy > 0 {
        gr.cy -= 1;
    }
    off_padding(gr);
}

/// Move to the start of the line, or of the whole run of lines it is wrapped
/// over.
fn grid_reader_cursor_start_of_line(gr: &mut RustGridReader<'_>, wrap: c_int) {
    if wrap != 0 {
        while gr.cy > 0 && wrapped(gr, gr.cy - 1) {
            gr.cy -= 1;
        }
    }
    gr.cx = 0;
}

/// Move to the end of the line, or of the whole run of lines it is wrapped
/// over.
fn grid_reader_cursor_end_of_line(gr: &mut RustGridReader<'_>, wrap: c_int, all: c_int) {
    if wrap != 0 {
        let yy = bottom(gr);
        while gr.cy < yy && wrapped(gr, gr.cy) {
            gr.cy += 1;
        }
    }
    gr.cx = if all != 0 {
        gr.gd.sx
    } else {
        grid_reader_line_length(gr)
    };
}

/// Make sure the cursor lies within the grid reader's bounding area, wrapping
/// to the next line as necessary. False if the cursor would wrap past the
/// bottom of the grid.
fn grid_reader_handle_wrap(gr: &mut RustGridReader<'_>, xx: &mut u_int, yy: u_int) -> bool {
    while gr.cx > *xx {
        if gr.cy == yy {
            return false;
        }
        grid_reader_cursor_start_of_line(gr, 0);
        grid_reader_cursor_down(gr);
        *xx = walk_end(gr);
    }
    true
}

fn grid_reader_in_set(gr: &RustGridReader<'_>, set: &CStr) -> c_int {
    grid_in_set(gr.gd, gr.cx, gr.cy, set)
}

/// Move the cursor to the start of the next word.
///
/// When navigating via spaces (for example with next-space) separators should
/// be empty.
///
/// If we started on a separator that is not whitespace, skip over subsequent
/// separators that are not whitespace. Otherwise, if we started on a
/// non-whitespace character, skip over subsequent characters that are neither
/// whitespace nor separators. Then, skip over whitespace (if any) until the
/// next non-whitespace character.
fn grid_reader_cursor_next_word(gr: &mut RustGridReader<'_>, separators: &CStr) {
    /* Do not break up wrapped words. */
    let mut xx = walk_end(gr);
    let yy = bottom(gr);

    if !grid_reader_handle_wrap(gr, &mut xx, yy) {
        return;
    }
    if in_set(gr, WHITESPACE) == 0 {
        skip_word(gr, separators, &mut xx, yy);
    }
    while grid_reader_handle_wrap(gr, &mut xx, yy) {
        let width = in_set(gr, WHITESPACE) as u_int;
        if width == 0 {
            break;
        }
        gr.cx += width;
    }
}

/// Walk the cursor off the end of the word it is in: off the run of
/// separators it starts on, or onto the first separator or space after the
/// characters it starts on.
fn skip_word(gr: &mut RustGridReader<'_>, separators: &CStr, xx: &mut u_int, yy: u_int) {
    let from_separator = in_set(gr, separators) != 0;
    gr.cx += 1;
    while grid_reader_handle_wrap(gr, xx, yy) {
        let ends = if from_separator {
            in_set(gr, separators) == 0 || in_set(gr, WHITESPACE) != 0
        } else {
            in_set(gr, separators) != 0 || in_set(gr, WHITESPACE) != 0
        };
        if ends {
            break;
        }
        gr.cx += 1;
    }
}

/// Move the cursor to the end of the next word.
///
/// When navigating via spaces (for example with next-space), separators should
/// be empty in both modes.
///
/// If we started on a whitespace, move until reaching the first non-whitespace
/// character. If that character is a separator, treat subsequent separators as
/// a word, and continue moving until the first non-separator. Otherwise,
/// continue moving until the first separator or whitespace.
fn grid_reader_cursor_next_word_end(gr: &mut RustGridReader<'_>, separators: &CStr) {
    /* Do not break up wrapped words. */
    let mut xx = walk_end(gr);
    let yy = bottom(gr);

    while grid_reader_handle_wrap(gr, &mut xx, yy) {
        if in_set(gr, WHITESPACE) != 0 {
            gr.cx += 1;
            continue;
        }
        skip_word(gr, separators, &mut xx, yy);
        return;
    }
}

/// Move the cursor to the previous place where a word begins.
fn grid_reader_cursor_previous_word(
    gr: &mut RustGridReader<'_>,
    separators: &CStr,
    already: c_int,
    stop_at_eol: c_int,
) {
    let word_is_letters;

    /* Move back to the previous word character. */
    if already != 0 || in_set(gr, WHITESPACE) != 0 {
        loop {
            if gr.cx > 0 {
                gr.cx -= 1;
                if in_set(gr, WHITESPACE) == 0 {
                    word_is_letters = (in_set(gr, separators) == 0) as c_int;
                    break;
                }
                continue;
            }
            if gr.cy == 0 {
                return;
            }
            grid_reader_cursor_up(gr);
            grid_reader_cursor_end_of_line(gr, 0, 0);

            /* Stop if separator at EOL. */
            if stop_at_eol != 0 && gr.cx > 0 {
                let oldx = gr.cx;
                gr.cx -= 1;
                let at_eol = in_set(gr, WHITESPACE) != 0;
                gr.cx = oldx;
                if at_eol {
                    word_is_letters = 0;
                    break;
                }
            }
        }
    } else {
        word_is_letters = (in_set(gr, separators) == 0) as c_int;
    }

    /* Move back to the beginning of this word. */
    let mut oldx;
    let mut oldy;
    loop {
        oldx = gr.cx;
        oldy = gr.cy;
        if gr.cx == 0 {
            if gr.cy == 0 || !wrapped(gr, gr.cy - 1) {
                break;
            }
            grid_reader_cursor_up(gr);
            grid_reader_cursor_end_of_line(gr, 0, 1);
        }
        if gr.cx > 0 {
            gr.cx -= 1;
        }
        if in_set(gr, WHITESPACE) != 0 || word_is_letters == in_set(gr, separators) {
            break;
        }
    }
    gr.cx = oldx;
    gr.cy = oldy;
}

/// Whether the character in a cell is the one being jumped to.
fn grid_reader_cell_equals_data(gc: &grid_cell, character: &[u8]) -> bool {
    if gc.flags as c_int & GRID_FLAG_PADDING != 0 {
        return false;
    }
    if gc.flags as c_int & GRID_FLAG_TAB != 0 && character == b"\t" {
        return true;
    }
    let size = gc.data.size as usize;
    size == character.len() && gc.data.data[..size] == *character
}

/// Jump forward to a character, over the run of lines this one wraps over.
fn grid_reader_cursor_jump(gr: &mut RustGridReader<'_>, jc: &[u8]) -> c_int {
    let yy = bottom(gr);
    let mut gc;
    let mut px = gr.cx;
    let mut py = gr.cy;
    while py <= yy {
        let xx = grid_line_length(gr.gd, py);
        while px < xx {
            gc = grid_get_cell(gr.gd, px, py);
            if grid_reader_cell_equals_data(&gc, jc) {
                gr.cx = px;
                gr.cy = py;
                return 1;
            }
            px += 1;
        }
        if py == yy || !wrapped(gr, py) {
            return 0;
        }
        px = 0;
        py += 1;
    }
    0
}

/// Jump back to a character, over the run of lines this one wraps over.
fn grid_reader_cursor_jump_back(gr: &mut RustGridReader<'_>, jc: &[u8]) -> c_int {
    let mut gc;
    let mut xx = gr.cx.wrapping_add(1);
    let mut py = gr.cy.wrapping_add(1);
    while py > 0 {
        let mut px = xx;
        while px > 0 {
            gc = grid_get_cell(gr.gd, px - 1, py - 1);
            if grid_reader_cell_equals_data(&gc, jc) {
                gr.cx = px - 1;
                gr.cy = py - 1;
                return 1;
            }
            px -= 1;
        }
        if py == 1 || !wrapped(gr, py - 2) {
            return 0;
        }
        xx = grid_line_length(gr.gd, py - 2);
        py -= 1;
    }
    0
}

/// Move the cursor to the first character of the line that is not a space,
/// looking over the run of lines it is wrapped over.
fn grid_reader_cursor_back_to_indentation(gr: &mut RustGridReader<'_>) {
    let yy = bottom(gr);
    let oldx = gr.cx;
    let oldy = gr.cy;
    let mut gc;

    grid_reader_cursor_start_of_line(gr, 1);
    let mut py = gr.cy;
    while py <= yy {
        let xx = grid_line_length(gr.gd, py);
        for px in 0..xx {
            gc = grid_get_cell(gr.gd, px, py);
            if (gc.data.size != 1 || gc.data.data[0] != b' ')
                && gc.flags as c_int & GRID_FLAG_TAB == 0
                && gc.flags as c_int & GRID_FLAG_PADDING == 0
            {
                gr.cx = px;
                gr.cy = py;
                return;
            }
        }
        if !wrapped(gr, py) {
            break;
        }
        py += 1;
    }
    gr.cx = oldx;
    gr.cy = oldy;
}

impl GridReader for RustGridReader<'_> {
    fn cursor(&self) -> (u_int, u_int) {
        grid_reader_get_cursor(self)
    }

    fn line_length(&self) -> u_int {
        grid_reader_line_length(self)
    }

    fn in_set(&self, set: &CStr) -> bool {
        grid_reader_in_set(self, set) != 0
    }

    fn right(&mut self, wrap: bool, all: bool, one_more: bool) {
        grid_reader_cursor_right(self, wrap as c_int, all as c_int, one_more as c_int)
    }

    fn left(&mut self, wrap: bool) {
        grid_reader_cursor_left(self, wrap as c_int)
    }

    fn down(&mut self) {
        grid_reader_cursor_down(self)
    }

    fn up(&mut self) {
        grid_reader_cursor_up(self)
    }

    fn start_of_line(&mut self, wrap: bool) {
        grid_reader_cursor_start_of_line(self, wrap as c_int)
    }

    fn end_of_line(&mut self, wrap: bool, all: bool) {
        grid_reader_cursor_end_of_line(self, wrap as c_int, all as c_int)
    }

    fn next_word(&mut self, separators: &CStr) {
        grid_reader_cursor_next_word(self, separators)
    }

    fn next_word_end(&mut self, separators: &CStr) {
        grid_reader_cursor_next_word_end(self, separators)
    }

    fn previous_word(&mut self, separators: &CStr, already: bool, stop_at_eol: bool) {
        grid_reader_cursor_previous_word(self, separators, already as c_int, stop_at_eol as c_int)
    }

    fn jump(&mut self, character: &[u8]) -> bool {
        grid_reader_cursor_jump(self, character) != 0
    }

    fn jump_back(&mut self, character: &[u8]) -> bool {
        grid_reader_cursor_jump_back(self, character) != 0
    }

    fn back_to_indentation(&mut self) {
        grid_reader_cursor_back_to_indentation(self)
    }
}

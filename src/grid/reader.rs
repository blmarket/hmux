use super::store::{grid_get_cell, grid_in_set, grid_line_length, grid_peek_line};
pub use crate::types::*;
use ::core::ffi::{CStr, c_char, c_int};
pub const GRID_FLAG_PADDING: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRID_FLAG_TAB: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const GRID_LINE_WRAPPED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

/// The characters the word commands count as space.
const WHITESPACE: &CStr = c"\t ";

/// The last line of the grid.
fn bottom(gr: &mut grid_reader<'_>) -> u_int {
    gr.gd.hsize.wrapping_add(gr.gd.sy).wrapping_sub(1)
}

/// Whether a line carries on onto the one below it.
fn wrapped(gr: &grid_reader<'_>, py: u_int) -> bool {
    line_flags(gr.gd, py) & GRID_LINE_WRAPPED != 0
}

/// The flags of the line at `py`.
fn line_flags(gd: &grid, py: u_int) -> c_int {
    grid_peek_line(gd, py).map_or(0, |gl| gl.flags)
}

/// Whether a cell is the padding of a wider one in front of it.
fn padding(gr: &mut grid_reader<'_>, px: u_int, py: u_int) -> bool {
    let mut gc = grid_cell::default();
    gc = grid_get_cell(gr.gd, px, py);
    gc.flags as c_int & GRID_FLAG_PADDING != 0
}

/// How far along its line a walk can go: a line that wraps runs to the last
/// column of the grid, one that does not to the end of its text.
fn walk_end(gr: &mut grid_reader<'_>) -> u_int {
    if wrapped(gr, gr.cy) {
        gr.gd.sx.wrapping_sub(1)
    } else {
        grid_reader_line_length(gr)
    }
}

/// What the set says about the cell under the cursor: zero when the cell is
/// not in it, and for a tab the number of its columns still to come.
fn in_set(gr: &mut grid_reader<'_>, set: &CStr) -> c_int {
    unsafe { grid_reader_in_set(gr, set.as_ptr()) }
}

/// Move the cursor back off any padding it is sitting on.
fn off_padding(gr: &mut grid_reader<'_>) {
    while gr.cx > 0 && padding(gr, gr.cx, gr.cy) {
        gr.cx -= 1;
    }
}

/// A reader placed at `cx`,`cy` of `gd`.
pub fn grid_reader_start(gd: &grid, cx: u_int, cy: u_int) -> grid_reader<'_> {
    grid_reader { gd, cx, cy }
}

/// Where the reader's cursor sits, as `(cx, cy)`.
pub fn grid_reader_get_cursor(gr: &grid_reader<'_>) -> (u_int, u_int) {
    (gr.cx, gr.cy)
}

pub fn grid_reader_line_length(gr: &mut grid_reader<'_>) -> u_int {
    grid_line_length(gr.gd, gr.cy)
}

/// Move the cursor right, onto the next line if it is at the end of this one
/// and asked to wrap.
pub fn grid_reader_cursor_right(gr: &mut grid_reader<'_>, wrap: c_int, all: c_int, onemore: c_int) {
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
pub fn grid_reader_cursor_left(gr: &mut grid_reader<'_>, wrap: c_int) {
    off_padding(gr);
    if gr.cx == 0 && gr.cy > 0 && (wrap != 0 || wrapped(gr, gr.cy - 1)) {
        grid_reader_cursor_up(gr);
        grid_reader_cursor_end_of_line(gr, 0, 0);
    } else if gr.cx > 0 {
        gr.cx -= 1;
    }
}

pub fn grid_reader_cursor_down(gr: &mut grid_reader<'_>) {
    if gr.cy < bottom(gr) {
        gr.cy += 1;
    }
    off_padding(gr);
}

pub fn grid_reader_cursor_up(gr: &mut grid_reader<'_>) {
    if gr.cy > 0 {
        gr.cy -= 1;
    }
    off_padding(gr);
}

/// Move to the start of the line, or of the whole run of lines it is wrapped
/// over.
pub fn grid_reader_cursor_start_of_line(gr: &mut grid_reader<'_>, wrap: c_int) {
    if wrap != 0 {
        while gr.cy > 0 && wrapped(gr, gr.cy - 1) {
            gr.cy -= 1;
        }
    }
    gr.cx = 0;
}

/// Move to the end of the line, or of the whole run of lines it is wrapped
/// over.
pub fn grid_reader_cursor_end_of_line(gr: &mut grid_reader<'_>, wrap: c_int, all: c_int) {
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
fn grid_reader_handle_wrap(gr: &mut grid_reader<'_>, xx: &mut u_int, yy: u_int) -> bool {
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

pub unsafe fn grid_reader_in_set(gr: &mut grid_reader<'_>, set: *const c_char) -> c_int {
    unsafe { grid_in_set(gr.gd, gr.cx, gr.cy, set) }
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
pub unsafe fn grid_reader_cursor_next_word(gr: &mut grid_reader<'_>, separators: *const c_char) {
    unsafe {
        let separators = CStr::from_ptr(separators);

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
}

/// Walk the cursor off the end of the word it is in: off the run of
/// separators it starts on, or onto the first separator or space after the
/// characters it starts on.
fn skip_word(gr: &mut grid_reader<'_>, separators: &CStr, xx: &mut u_int, yy: u_int) {
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
pub unsafe fn grid_reader_cursor_next_word_end(
    gr: &mut grid_reader<'_>,
    separators: *const c_char,
) {
    unsafe {
        let separators = CStr::from_ptr(separators);

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
}

/// Move the cursor to the previous place where a word begins.
pub unsafe fn grid_reader_cursor_previous_word(
    gr: &mut grid_reader<'_>,
    separators: *const c_char,
    already: c_int,
    stop_at_eol: c_int,
) {
    unsafe {
        let separators = CStr::from_ptr(separators);
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
}

/// Whether the character in a cell is the one being jumped to.
fn grid_reader_cell_equals_data(gc: &grid_cell, ud: &utf8_data) -> bool {
    if gc.flags as c_int & GRID_FLAG_PADDING != 0 {
        return false;
    }
    if gc.flags as c_int & GRID_FLAG_TAB != 0 && ud.size == 1 && ud.data[0] == b'\t' {
        return true;
    }
    let size = gc.data.size as usize;
    gc.data.size == ud.size && gc.data.data[..size] == ud.data[..size]
}

/// Jump forward to a character, over the run of lines this one wraps over.
pub fn grid_reader_cursor_jump(gr: &mut grid_reader<'_>, jc: &utf8_data) -> c_int {
    let yy = bottom(gr);
    let mut gc = grid_cell::default();
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
pub fn grid_reader_cursor_jump_back(gr: &mut grid_reader<'_>, jc: &utf8_data) -> c_int {
    let mut gc = grid_cell::default();
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
pub fn grid_reader_cursor_back_to_indentation(gr: &mut grid_reader<'_>) {
    let yy = bottom(gr);
    let oldx = gr.cx;
    let oldy = gr.cy;
    let mut gc = grid_cell::default();

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

#[cfg(test)]
#[path = "../tests/test_grid_reader.rs"]
mod tests;

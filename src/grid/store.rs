use super::links::hyperlinks_get;
use crate::fmt_args;
use crate::log::{fatalx, log_debug};
use crate::server::current_time;
use crate::style::colour_split_rgb;
use crate::text::{utf8_build_one, utf8_cstrhas, utf8_from_data, utf8_set, utf8_to_data};
pub use crate::types::*;
use ::core::ffi::{CStr, c_int};
use ::std::ffi::CString;
pub const UINT_MAX: ::core::ffi::c_uint = u_int::MAX;
pub const COLOUR_FLAG_256: ::core::ffi::c_int = 0x1000000 as ::core::ffi::c_int;
pub const COLOUR_FLAG_RGB: ::core::ffi::c_int = 0x2000000 as ::core::ffi::c_int;
pub const GRID_ATTR_BRIGHT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const GRID_ATTR_DIM: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRID_ATTR_BLINK: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const GRID_ATTR_REVERSE: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const GRID_ATTR_HIDDEN: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const GRID_ATTR_ITALICS: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const GRID_ATTR_CHARSET: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const GRID_ATTR_STRIKETHROUGH: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_2: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_3: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_4: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_5: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const GRID_ATTR_OVERLINE: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const GRID_FLAG_FG256: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const GRID_FLAG_BG256: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const GRID_FLAG_PADDING: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRID_FLAG_EXTENDED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const GRID_FLAG_CLEARED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const GRID_FLAG_TAB: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const GRID_LINE_WRAPPED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const GRID_LINE_EXTENDED: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const GRID_LINE_DEAD: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRID_LINE_HYPERLINK: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const GRID_STRING_WITH_SEQUENCES: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const GRID_STRING_ESCAPE_SEQUENCES: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const GRID_STRING_TRIM_SPACES: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRID_STRING_EMPTY_CELLS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const GRID_HISTORY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

/// The character of a cell that holds one byte.
const fn one_byte(byte: u_char) -> [u_char; 32] {
    let mut data = [0; 32];
    data[0] = byte;
    data
}

/// A cell holding one byte, in the size and width the grid records for it.
const fn one_byte_cell(byte: u_char, size: u_char, width: u_char, flags: c_int) -> grid_cell {
    grid_cell {
        data: utf8_data {
            data: one_byte(byte),
            have: 0,
            size,
            width,
        },
        attr: 0,
        flags: flags as u_char,
        fg: 8,
        bg: 8,
        us: 8,
        link: 0,
    }
}

pub static grid_default_cell: grid_cell = one_byte_cell(b' ', 1, 1, 0);

/// Padding cells are the only zero width cells the grid holds.
static grid_padding_cell: grid_cell = one_byte_cell(b'!', 0, 0, GRID_FLAG_PADDING);

static grid_cleared_cell: grid_cell = one_byte_cell(b' ', 1, 1, GRID_FLAG_CLEARED);

static grid_cleared_entry: grid_cell_entry = grid_cell_entry {
    c2rust_unnamed: grid_cell_entry_union {
        data: grid_cell_entry_data {
            attr: 0,
            fg: 8,
            bg: 8,
            data: b' ',
        },
    },
    flags: GRID_FLAG_CLEARED as u_char,
};

/// A line the reflow has taken the contents of.
fn dead_line() -> grid_line {
    let mut gl = grid_line::new();
    gl.flags = GRID_LINE_DEAD;
    gl
}

/// A cell to read into. Every reader fills in every field it goes on to look
/// at; this is the cell an unwritten position reads as.
fn scratch_cell() -> grid_cell {
    one_byte_cell(b' ', 1, 1, 0)
}

/// The line at `py`. The line array is indexed the way the C did, without a
/// bounds check of its own: every caller has either put `py` through
/// `grid_check_y` or derived it from a walk that stays inside the grid.
fn line_at(gd: &grid, py: u_int) -> &grid_line {
    &gd.linedata[py as usize]
}

/// The same line, to write to.
fn line_at_mut(gd: &mut grid, py: u_int) -> &mut grid_line {
    &mut gd.linedata[py as usize]
}

/// Whether a background colour is the terminal's own.
fn colour_is_default(bg: u_int) -> bool {
    bg == 8 || bg == 9
}

/// Store a cell in an entry, packed into the four bytes it has.
fn grid_store_cell(gce: &mut grid_cell_entry, gc: &grid_cell, c: u_char) {
    let mut flags = gc.flags as c_int & !GRID_FLAG_CLEARED;
    if gc.fg & COLOUR_FLAG_256 != 0 {
        flags |= GRID_FLAG_FG256;
    }
    if gc.bg & COLOUR_FLAG_256 != 0 {
        flags |= GRID_FLAG_BG256;
    }
    gce.flags = flags as u_char;
    gce.c2rust_unnamed.data = grid_cell_entry_data {
        attr: gc.attr as u_char,
        fg: (gc.fg & 0xff) as u_char,
        bg: (gc.bg & 0xff) as u_char,
        data: c,
    };
}

/// Whether a cell holds more than the packed entry can.
fn grid_need_extended_cell(gce: &grid_cell_entry, gc: &grid_cell) -> bool {
    gce.flags as c_int & GRID_FLAG_EXTENDED != 0
        || gc.attr as c_int > 0xff
        || gc.data.size > 1
        || gc.data.width > 1
        || gc.fg & COLOUR_FLAG_RGB != 0
        || gc.bg & COLOUR_FLAG_RGB != 0
        || gc.us != 8
        || gc.link != 0
        || gc.flags as c_int & GRID_FLAG_TAB != 0
}

/// Take a new extended cell for the entry at `px`, at the end of the line's.
fn grid_get_extended_cell(gl: &mut grid_line, px: u_int, flags: c_int) {
    let at = gl.push_extended();
    let gce = &mut gl.celldata_mut()[px as usize];
    gce.c2rust_unnamed.offset = at;
    gce.flags = (flags | GRID_FLAG_EXTENDED) as u_char;
}

/// Write a cell into the extended cell of the entry at `px`, taking one first
/// if the entry has none, and answer which extended cell that was.
fn grid_extended_cell(gl: &mut grid_line, px: u_int, gc: &grid_cell) -> u_int {
    unsafe {
        let flags = gc.flags as c_int & !GRID_FLAG_CLEARED;
        let gce = gl.celldata()[px as usize];
        if gce.flags as c_int & GRID_FLAG_EXTENDED == 0 {
            grid_get_extended_cell(gl, px, flags);
        } else if gce.c2rust_unnamed.offset >= gl.extdsize() {
            fatalx(c"offset too big".as_ptr(), fmt_args![]);
        }
        gl.flags |= GRID_LINE_EXTENDED;
        if gc.link != 0 {
            gl.flags |= GRID_LINE_HYPERLINK;
        }
        let data = if gc.flags as c_int & GRID_FLAG_TAB != 0 {
            gc.data.width as utf8_char
        } else {
            utf8_from_data(&gc.data).1
        };
        let at = gl.celldata()[px as usize].c2rust_unnamed.offset;
        gl.extddata_mut()[at as usize] = grid_extd_entry {
            data,
            attr: gc.attr,
            flags: flags as u_char,
            fg: gc.fg,
            bg: gc.bg,
            us: gc.us,
            link: gc.link,
        };
        at
    }
}

/// Give back the extended cells the line's entries no longer point at.
fn grid_compact_line(gl: &mut grid_line) {
    unsafe {
        if gl.extdsize() == 0 {
            return;
        }
        let wanted = gl
            .celldata()
            .iter()
            .filter(|gce| gce.flags as c_int & GRID_FLAG_EXTENDED != 0)
            .count();
        if wanted == 0 {
            gl.set_extended(&[]);
            return;
        }
        let mut new = Vec::with_capacity(wanted);
        let (cells, old) = gl.parts_mut();
        for gce in cells {
            if gce.flags as c_int & GRID_FLAG_EXTENDED != 0 {
                new.push(old[gce.c2rust_unnamed.offset as usize]);
                gce.c2rust_unnamed.offset = new.len() as u_int - 1;
            }
        }
        gl.set_extended(&new);
    }
}

/// The line at `line`, to write to.
pub fn grid_get_line(gd: &mut grid, line: u_int) -> &mut grid_line {
    line_at_mut(gd, line)
}

pub fn grid_adjust_lines(gd: &mut grid, lines: u_int) {
    gd.linedata.resize_with(lines as usize, grid_line::new);
}

/// Put the cleared cell at a position, in the background asked for. A cell
/// that has been moved rather than cleared in place gives up its extended
/// cell.
fn grid_clear_cell(gd: &mut grid, px: u_int, py: u_int, bg: u_int, moved: bool) {
    unsafe {
        let gl = line_at_mut(gd, py);
        let old = gl.celldata()[px as usize];
        let old_offset = old.c2rust_unnamed.offset;
        let had_extended = old.flags as c_int & GRID_FLAG_EXTENDED != 0;
        gl.celldata_mut()[px as usize] = grid_cleared_entry;
        if !moved && had_extended && old_offset < gl.extdsize() {
            let gce = &mut gl.celldata_mut()[px as usize];
            gce.flags = (gce.flags as c_int | GRID_FLAG_EXTENDED) as u_char;
            gce.c2rust_unnamed.offset = old_offset;
            let at = grid_extended_cell(gl, px, &grid_cleared_cell);
            if bg != 8 {
                gl.extddata_mut()[at as usize].bg = bg as c_int;
            }
        } else if bg != 8 {
            if bg & COLOUR_FLAG_RGB as u_int != 0 {
                let flags = gl.celldata()[px as usize].flags as c_int;
                grid_get_extended_cell(gl, px, flags);
                let at = grid_extended_cell(gl, px, &grid_cleared_cell);
                gl.extddata_mut()[at as usize].bg = bg as c_int;
            } else {
                let gce = &mut gl.celldata_mut()[px as usize];
                if bg & COLOUR_FLAG_256 as u_int != 0 {
                    gce.flags = (gce.flags as c_int | GRID_FLAG_BG256) as u_char;
                }
                gce.c2rust_unnamed.data.bg = bg as u_char;
            }
        }
    }
}

/// Whether `py` is a line of this grid, logging the caller if it is not.
fn grid_check_y(gd: &grid, from: &CStr, py: u_int) -> bool {
    unsafe {
        if py >= gd.hsize + gd.sy {
            log_debug(
                c"%s: y out of range: %u".as_ptr(),
                fmt_args![from.as_ptr(), py],
            );
            return false;
        }
        true
    }
}

/// Whether two cells are drawn the same way. The cleared flag is not part of
/// how a cell looks.
fn look_equal(gc1: &grid_cell, gc2: &grid_cell) -> bool {
    gc1.fg == gc2.fg
        && gc1.bg == gc2.bg
        && gc1.attr == gc2.attr
        && gc1.flags as c_int & !GRID_FLAG_CLEARED == gc2.flags as c_int & !GRID_FLAG_CLEARED
        && gc1.link == gc2.link
}

/// The bytes of the character a cell holds.
fn cell_bytes(gc: &grid_cell) -> &[u_char] {
    &gc.data.data[..gc.data.size as usize]
}

pub unsafe fn grid_cells_look_equal(gc1: &grid_cell, gc2: &grid_cell) -> c_int {
    look_equal(gc1, gc2) as c_int
}

pub unsafe fn grid_cells_equal(gc1: &grid_cell, gc2: &grid_cell) -> c_int {
    (look_equal(gc1, gc2)
        && gc1.data.width == gc2.data.width
        && gc1.data.size == gc2.data.size
        && cell_bytes(gc1) == cell_bytes(gc2)) as c_int
}

/// Turn a cell into a tab of `width` columns. The input parser is the only
/// caller that picks a width, and it turns down any width the character
/// buffer cannot hold.
fn set_tab(gc: &mut grid_cell, width: u_int) {
    gc.data.data.fill(0);
    gc.flags = (gc.flags as c_int | GRID_FLAG_TAB) as u_char;
    gc.flags = (gc.flags as c_int & !GRID_FLAG_PADDING) as u_char;
    gc.data.have = width as u_char;
    gc.data.size = gc.data.have;
    gc.data.width = gc.data.size;
    gc.data.data[..gc.data.size as usize].fill(b' ');
}

pub unsafe fn grid_set_tab(gc: &mut grid_cell, width: u_int) {
    set_tab(gc, width)
}

/// Free one line's cells and leave it empty.
fn grid_free_line(gd: &mut grid, py: u_int) {
    *line_at_mut(gd, py) = grid_line::new();
}

fn grid_free_lines(gd: &mut grid, py: u_int, ny: u_int) {
    {
        for yy in py..py.wrapping_add(ny) {
            grid_free_line(gd, yy);
        }
    }
}

pub fn grid_create(sx: u_int, sy: u_int, hlimit: u_int) -> Box<grid> {
    Box::new(grid {
        flags: if hlimit != 0 { GRID_HISTORY } else { 0 },
        sx,
        sy,
        hscrolled: 0,
        hsize: 0,
        hlimit,
        linedata: (0..sy).map(|_| grid_line::new()).collect(),
    })
}

pub fn grid_destroy(gd: Box<grid>) {
    drop(gd);
}

/// Whether two grids hold the same screen. Only the first `sy` lines are
/// looked at, which are the history when there is any.
pub fn grid_compare(ga: &grid, gb: &grid) -> c_int {
    if ga.sx != gb.sx || ga.sy != gb.sy {
        return 1;
    }
    let mut gca = scratch_cell();
    let mut gcb = scratch_cell();
    for yy in 0..ga.sy {
        let cellsize = line_at(ga, yy).cellsize();
        if cellsize != line_at(gb, yy).cellsize() {
            return 1;
        }
        for xx in 0..cellsize {
            gca = grid_get_cell(ga, xx, yy);
            gcb = grid_get_cell(gb, xx, yy);
            if unsafe { grid_cells_equal(&gca, &gcb) } == 0 {
                return 1;
            }
        }
    }
    0
}

/// Drop the first `ny` lines of the grid, moving the rest up over them.
fn grid_trim_history(gd: &mut grid, ny: u_int) {
    let mut lines = std::mem::take(&mut gd.linedata);
    let mut remaining = lines.split_off(ny as usize);
    remaining.extend((0..ny).map(|_| grid_line::new()));
    gd.linedata = remaining;
}

/// Collect the oldest lines of the history when it is at its limit: all of
/// what is over the limit, or a tenth of the limit. The count is never more
/// than the history holds — the history has reached the limit by the time
/// either form is worked out — so the clamp the C had against that is gone.
pub fn grid_collect_history(gd: &mut grid, all: c_int) {
    if gd.hsize == 0 || gd.hsize < gd.hlimit {
        return;
    }
    let mut ny = if all != 0 {
        gd.hsize - gd.hlimit
    } else {
        gd.hlimit / 10
    };
    if ny < 1 {
        ny = 1;
    }
    grid_trim_history(gd, ny);
    gd.hsize -= ny;
    if gd.hscrolled > gd.hsize {
        gd.hscrolled = gd.hsize;
    }
}

/// Take `ny` lines off the bottom of the grid and shrink the history by as
/// many, so the screen comes to sit on what was history.
pub fn grid_remove_history(gd: &mut grid, ny: u_int) {
    if ny > gd.hsize {
        return;
    }
    let start = gd.hsize + gd.sy - ny;
    for yy in 0..ny {
        grid_free_line(gd, start + yy);
    }
    gd.linedata[start as usize..].fill(grid_line::new());
    gd.hsize -= ny;
}

/// Scroll the whole screen, moving its top line into the history.
pub fn grid_scroll_history(gd: &mut grid, bg: u_int) {
    unsafe {
        let yy = gd.hsize + gd.sy;
        gd.linedata.push(grid_line::new());
        grid_empty_line(gd, yy, bg);
        gd.hscrolled += 1;
        let hsize = gd.hsize;
        let gl = line_at_mut(gd, hsize);
        grid_compact_line(gl);
        gl.time = current_time;
        gd.hsize += 1;
    }
}

pub fn grid_clear_history(gd: &mut grid) {
    grid_trim_history(gd, gd.hsize);
    gd.hscrolled = 0;
    gd.hsize = 0;
    gd.linedata.truncate(gd.sy as usize);
}

/// Scroll one region of the screen up, moving its top line into the history.
pub fn grid_scroll_history_region(gd: &mut grid, upper: u_int, lower: u_int, bg: u_int) {
    unsafe {
        /* Create a space for a new line. */
        gd.linedata.push(grid_line::new());

        /* Move the entire screen down to free a space for this line. */
        let history = gd.hsize as usize;
        gd.linedata[history..=history + gd.sy as usize].rotate_right(1);

        /* Adjust the region and find its start and end. */
        let upper = upper as usize + 1;
        let lower = lower as usize + 1;

        /* Move the line into the history. */
        let promoted = std::mem::replace(&mut gd.linedata[upper], grid_line::new());
        gd.linedata[history] = promoted;
        gd.linedata[history].time = current_time;

        /* Then move the region up and clear the bottom line. */
        gd.linedata[upper..=lower].rotate_left(1);
        grid_empty_line(gd, lower as u_int, bg);

        /* Move the history offset down over the line. */
        gd.hscrolled += 1;
        gd.hsize += 1;
    }
}

/// Give a line enough cells to reach `sx`, in steps of a quarter, a half and
/// the whole width of the grid.
fn grid_expand_line(gd: &mut grid, py: u_int, sx: u_int, bg: u_int) {
    if sx <= line_at(gd, py).cellsize() {
        return;
    }
    let sx = if sx < gd.sx / 4 {
        gd.sx / 4
    } else if sx < gd.sx / 2 {
        gd.sx / 2
    } else if gd.sx > sx {
        gd.sx
    } else {
        sx
    };
    let gl = line_at_mut(gd, py);
    let from = gl.cellsize();
    gl.resize_cells(sx);
    for xx in from..sx {
        grid_clear_cell(gd, xx, py, bg, false);
    }
}

/// Empty a line, giving it a background colour if it is not the default one.
pub fn grid_empty_line(gd: &mut grid, py: u_int, bg: u_int) {
    *line_at_mut(gd, py) = grid_line::new();
    if !colour_is_default(bg) {
        grid_expand_line(gd, py, gd.sx, bg);
    }
}

/// The line at `py`, or nothing when the grid does not reach it.
pub fn grid_peek_line(gd: &grid, py: u_int) -> Option<&grid_line> {
    if !grid_check_y(gd, c"grid_peek_line", py) {
        return None;
    }
    Some(line_at(gd, py))
}

/// Read the cell at `px` of a line, which the caller knows the line has.
fn grid_get_cell1(gl: &grid_line, px: u_int, gc: &mut grid_cell) {
    unsafe {
        let gce = gl.celldata()[px as usize];
        if gce.flags as c_int & GRID_FLAG_EXTENDED != 0 {
            let offset = gce.c2rust_unnamed.offset;
            if offset >= gl.extdsize() {
                *gc = grid_default_cell;
                return;
            }
            let gee = gl.extddata()[offset as usize];
            gc.flags = gee.flags;
            gc.attr = gee.attr;
            gc.fg = gee.fg;
            gc.bg = gee.bg;
            gc.us = gee.us;
            gc.link = gee.link;
            if gc.flags as c_int & GRID_FLAG_TAB != 0 {
                set_tab(gc, gee.data);
            } else {
                utf8_to_data(gee.data, &mut gc.data);
            }
            return;
        }
        let data = gce.c2rust_unnamed.data;
        gc.flags = (gce.flags as c_int & !(GRID_FLAG_FG256 | GRID_FLAG_BG256)) as u_char;
        gc.attr = data.attr as u_short;
        gc.fg = data.fg as c_int;
        if gce.flags as c_int & GRID_FLAG_FG256 != 0 {
            gc.fg |= COLOUR_FLAG_256;
        }
        gc.bg = data.bg as c_int;
        if gce.flags as c_int & GRID_FLAG_BG256 != 0 {
            gc.bg |= COLOUR_FLAG_256;
        }
        gc.us = 8;
        utf8_set(&mut gc.data, data.data);
        gc.link = 0;
    }
}

/// The cell at `px`,`py`, or the default one when nothing is written there.
pub fn grid_get_cell(gd: &grid, px: u_int, py: u_int) -> grid_cell {
    if !grid_check_y(gd, c"grid_get_cell", py) || px >= line_at(gd, py).cellsize() {
        return grid_default_cell;
    }
    let mut gc = grid_default_cell;
    grid_get_cell1(line_at(gd, py), px, &mut gc);
    gc
}

pub fn grid_set_cell(gd: &mut grid, px: u_int, py: u_int, gc: &grid_cell) {
    if !grid_check_y(gd, c"grid_set_cell", py) {
        return;
    }
    let used = px.wrapping_add(1);
    grid_expand_line(gd, py, used, 8);
    let gl = line_at_mut(gd, py);
    if used > gl.cellused {
        gl.cellused = used;
    }
    if grid_need_extended_cell(&gl.celldata()[px as usize], gc) {
        grid_extended_cell(gl, px, gc);
    } else {
        grid_store_cell(&mut gl.celldata_mut()[px as usize], gc, gc.data.data[0]);
    }
}

pub fn grid_set_padding(gd: &mut grid, px: u_int, py: u_int) {
    grid_set_cell(gd, px, py, &grid_padding_cell)
}

/// Write a run of cells that share one style, one byte of `s` each.
pub fn grid_set_cells(gd: &mut grid, px: u_int, py: u_int, gc: &grid_cell, s: &[u8]) {
    if !grid_check_y(gd, c"grid_set_cells", py) {
        return;
    }
    let used = px.wrapping_add(s.len() as u_int);
    grid_expand_line(gd, py, used, 8);
    let gl = line_at_mut(gd, py);
    if used > gl.cellused {
        gl.cellused = used;
    }
    for (i, &byte) in s.iter().enumerate() {
        let at = px + i as u_int;
        if grid_need_extended_cell(&gl.celldata()[at as usize], gc) {
            let extd = grid_extended_cell(gl, at, gc);
            gl.extddata_mut()[extd as usize].data = utf8_build_one(byte);
        } else {
            grid_store_cell(&mut gl.celldata_mut()[at as usize], gc, byte);
        }
    }
}

/// Clear a rectangle of the grid to a background colour.
pub fn grid_clear(gd: &mut grid, px: u_int, py: u_int, nx: u_int, ny: u_int, bg: u_int) {
    if nx == 0 || ny == 0 {
        return;
    }
    if px == 0 && nx == gd.sx {
        grid_clear_lines(gd, py, ny, bg);
        return;
    }
    /*
     * The counts come to the grid already worked out from a screen size
     * and can have gone round zero on the way, so they are added the way
     * the C added them: a count that has wrapped puts the last row out of
     * range and the whole call is dropped.
     */
    let bottom = py.wrapping_add(ny).wrapping_sub(1);
    if !grid_check_y(gd, c"grid_clear", py) || !grid_check_y(gd, c"grid_clear", bottom) {
        return;
    }
    for yy in py..py.wrapping_add(ny) {
        let mut sx = gd.sx;
        let cellsize = line_at(gd, yy).cellsize();
        if sx > cellsize {
            sx = cellsize;
        }
        let mut ox = nx;
        if colour_is_default(bg) {
            if px > sx {
                continue;
            }
            if px.wrapping_add(nx) > sx {
                ox = sx - px;
            }
        }
        grid_expand_line(gd, yy, px.wrapping_add(ox), 8); /* default bg first */
        for xx in px..px.wrapping_add(ox) {
            grid_clear_cell(gd, xx, yy, bg, false);
        }
    }
}

/// Clear whole lines, which is to free them rather than paint them.
pub fn grid_clear_lines(gd: &mut grid, py: u_int, ny: u_int, bg: u_int) {
    if ny == 0 {
        return;
    }
    let bottom = py.wrapping_add(ny).wrapping_sub(1);
    if !grid_check_y(gd, c"grid_clear_lines", py) || !grid_check_y(gd, c"grid_clear_lines", bottom)
    {
        return;
    }
    for yy in py..py.wrapping_add(ny) {
        grid_free_line(gd, yy);
        grid_empty_line(gd, yy, bg);
    }
    if py != 0 {
        line_at_mut(gd, py - 1).flags &= !GRID_LINE_WRAPPED;
    }
}

/// Move a group of lines, emptying the ones left behind.
pub fn grid_move_lines(gd: &mut grid, dy: u_int, py: u_int, ny: u_int, bg: u_int) {
    if ny == 0 || py == dy {
        return;
    }
    let from_bottom = py.wrapping_add(ny).wrapping_sub(1);
    let to_bottom = dy.wrapping_add(ny).wrapping_sub(1);
    if !grid_check_y(gd, c"grid_move_lines", py)
        || !grid_check_y(gd, c"grid_move_lines", from_bottom)
        || !grid_check_y(gd, c"grid_move_lines", dy)
        || !grid_check_y(gd, c"grid_move_lines", to_bottom)
    {
        return;
    }
    let (from_end, to_end) = (py.wrapping_add(ny), dy.wrapping_add(ny));

    if dy != 0 {
        line_at_mut(gd, dy - 1).flags &= !GRID_LINE_WRAPPED;
    }

    if dy > py {
        for offset in (0..ny as usize).rev() {
            gd.linedata.swap(py as usize + offset, dy as usize + offset);
        }
    } else {
        for offset in 0..ny as usize {
            gd.linedata.swap(py as usize + offset, dy as usize + offset);
        }
    }

    /*
     * Wipe any lines that have been moved (without freeing them - they
     * are still present).
     */
    for yy in py..from_end {
        if yy < dy || yy >= to_end {
            grid_empty_line(gd, yy, bg);
        }
    }

    if py != 0 && (py < dy || py >= to_end) {
        line_at_mut(gd, py - 1).flags &= !GRID_LINE_WRAPPED;
    }
}

/// Move a group of cells along a line, clearing the ones left behind.
pub fn grid_move_cells(gd: &mut grid, dx: u_int, px: u_int, py: u_int, nx: u_int, bg: u_int) {
    if nx == 0 || px == dx {
        return;
    }
    if !grid_check_y(gd, c"grid_move_cells", py) {
        return;
    }
    let (from_end, to_end) = (px.wrapping_add(nx), dx.wrapping_add(nx));
    grid_expand_line(gd, py, from_end, 8);
    grid_expand_line(gd, py, to_end, 8);
    let gl = line_at_mut(gd, py);
    gl.celldata_mut()
        .copy_within(px as usize..px as usize + nx as usize, dx as usize);
    if to_end > gl.cellused {
        gl.cellused = to_end;
    }

    /* Wipe any cells that have been moved. */
    for xx in px..from_end {
        if !(xx >= dx && xx < to_end) {
            grid_clear_cell(gd, xx, py, bg, true);
        }
    }
}

/// The numbers of one SGR colour sequence, at most six of them.
struct Values {
    items: [c_int; 6],
    len: usize,
}

impl Values {
    fn none() -> Values {
        Values {
            items: [0; 6],
            len: 0,
        }
    }

    fn of(items: &[c_int]) -> Values {
        let mut values = Values::none();
        values.items[..items.len()].copy_from_slice(items);
        values.len = items.len();
        values
    }

    /// The three parts of an RGB colour, after the code that introduces one.
    fn rgb(code: c_int, colour: c_int) -> Values {
        let (r, g, b) = colour_split_rgb(colour);
        Values::of(&[code, 2, r as c_int, g as c_int, b as c_int])
    }
}

impl ::core::ops::Deref for Values {
    type Target = [c_int];

    fn deref(&self) -> &[c_int] {
        &self.items[..self.len]
    }
}

/// The sequence that sets a foreground colour.
fn grid_string_cells_fg(gc: &grid_cell) -> Values {
    match gc.fg {
        fg if fg & COLOUR_FLAG_256 != 0 => Values::of(&[38, 5, fg & 0xff]),
        fg if fg & COLOUR_FLAG_RGB != 0 => Values::rgb(38, fg),
        fg @ 0..=7 => Values::of(&[fg + 30]),
        8 => Values::of(&[39]),
        fg @ 90..=97 => Values::of(&[fg]),
        _ => Values::none(),
    }
}

/// The sequence that sets a background colour.
fn grid_string_cells_bg(gc: &grid_cell) -> Values {
    match gc.bg {
        bg if bg & COLOUR_FLAG_256 != 0 => Values::of(&[48, 5, bg & 0xff]),
        bg if bg & COLOUR_FLAG_RGB != 0 => Values::rgb(48, bg),
        bg @ 0..=7 => Values::of(&[bg + 40]),
        8 => Values::of(&[49]),
        bg @ 90..=97 => Values::of(&[bg + 10]),
        _ => Values::none(),
    }
}

/// The sequence that sets an underscore colour, which only the palette and
/// RGB forms have.
fn grid_string_cells_us(gc: &grid_cell) -> Values {
    match gc.us {
        us if us & COLOUR_FLAG_256 != 0 => Values::of(&[58, 5, us & 0xff]),
        us if us & COLOUR_FLAG_RGB != 0 => Values::rgb(58, us),
        _ => Values::none(),
    }
}

/// How much room the escape sequences for one cell get.
const CODE_SIZE: usize = 8192;

/// The escape sequences for one cell, in a buffer of the size the C had,
/// with the same truncation once it is full.
struct Code {
    buf: [u8; CODE_SIZE],
    len: usize,
}

impl Code {
    fn new() -> Code {
        Code {
            buf: [0; CODE_SIZE],
            len: 0,
        }
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    /// Appends what still fits, keeping the room the terminating NUL took.
    fn push(&mut self, text: &[u8]) {
        let room = CODE_SIZE - 1 - self.len;
        let text = &text[..text.len().min(room)];
        self.buf[self.len..self.len + text.len()].copy_from_slice(text);
        self.len += text.len();
    }

    fn bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

/// The introducer of a control sequence, written out in full when the answer
/// is meant to be read rather than sent.
fn escape(flags: c_int) -> &'static [u8] {
    if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 {
        b"\\033"
    } else {
        b"\x1b"
    }
}

/// Add one colour sequence, unless the colour is unchanged or is the default
/// one just after a reset.
fn grid_string_cells_add_code(
    buf: &mut Code,
    s: &[u_int],
    newc: &Values,
    oldc: &Values,
    flags: c_int,
) {
    let reset = s.first() == Some(&0);
    if newc.is_empty() {
        return; /* no code to add */
    }
    if !reset && **newc == **oldc {
        return; /* no reset and colour unchanged */
    }
    if reset && (newc[0] == 49 || newc[0] == 39) {
        return; /* reset and colour default */
    }
    buf.push(escape(flags));
    buf.push(b"[");
    for (i, value) in newc.iter().enumerate() {
        if i + 1 < newc.len() {
            buf.push(format!("{value};").as_bytes());
        } else {
            buf.push(format!("{value}").as_bytes());
        }
    }
    buf.push(b"m");
}

/// Add the sequence that opens a hyperlink, or the one that closes the last
/// one when both parts are empty. Answers whether it fitted.
fn grid_string_cells_add_hyperlink(buf: &mut Code, id: &[u8], uri: &[u8], flags: c_int) -> bool {
    if uri.len() + id.len() + 17 >= CODE_SIZE {
        return false;
    }
    buf.push(escape(flags));
    buf.push(b"]8;");
    if !id.is_empty() {
        buf.push(b"id=");
        buf.push(id);
    }
    buf.push(b";");
    buf.push(uri);
    if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 {
        buf.push(b"\\033\\\\");
    } else {
        buf.push(b"\x1b\\");
    }
    true
}

/// The attribute bits, in the order they are written, with the SGR code of
/// each.
const ATTRS: [(u_int, u_int); 13] = [
    (GRID_ATTR_BRIGHT as u_int, 1),
    (GRID_ATTR_DIM as u_int, 2),
    (GRID_ATTR_ITALICS as u_int, 3),
    (GRID_ATTR_UNDERSCORE as u_int, 4),
    (GRID_ATTR_BLINK as u_int, 5),
    (GRID_ATTR_REVERSE as u_int, 7),
    (GRID_ATTR_HIDDEN as u_int, 8),
    (GRID_ATTR_STRIKETHROUGH as u_int, 9),
    (GRID_ATTR_UNDERSCORE_2 as u_int, 42),
    (GRID_ATTR_UNDERSCORE_3 as u_int, 43),
    (GRID_ATTR_UNDERSCORE_4 as u_int, 44),
    (GRID_ATTR_UNDERSCORE_5 as u_int, 45),
    (GRID_ATTR_OVERLINE as u_int, 53),
];

/// The sequences that turn the cell before this one into this one.
fn grid_string_cells_code(
    lastgc: &grid_cell,
    gc: &grid_cell,
    buf: &mut Code,
    flags: c_int,
    sc: *mut screen,
    has_link: &mut bool,
) {
    unsafe {
        let attr = gc.attr as u_int;
        let mut lastattr = lastgc.attr as u_int;
        let mut s: Vec<u_int> = Vec::new();

        /* If any attribute is removed, begin with 0. */
        let removed = ATTRS
            .iter()
            .any(|(mask, _)| !attr & mask != 0 && lastattr & mask != 0);
        if removed || (lastgc.us != 8 && gc.us == 8) {
            s.push(0);
            lastattr &= GRID_ATTR_CHARSET as u_int;
        }

        /* For each attribute that is newly set, add its code. */
        for (mask, code) in ATTRS {
            if attr & mask != 0 && lastattr & mask == 0 {
                s.push(code);
            }
        }

        /* Write the attributes. */
        buf.clear();
        if !s.is_empty() {
            buf.push(escape(flags));
            buf.push(b"[");
            for (i, value) in s.iter().enumerate() {
                if *value < 10 {
                    buf.push(format!("{value}").as_bytes());
                } else {
                    buf.push(format!("{}:{}", value / 10, value % 10).as_bytes());
                }
                if i + 1 < s.len() {
                    buf.push(b";");
                }
            }
            buf.push(b"m");
        }

        /* If a colour changed, write its parameters. */
        for (of, of_last) in [
            (grid_string_cells_fg(gc), grid_string_cells_fg(lastgc)),
            (grid_string_cells_bg(gc), grid_string_cells_bg(lastgc)),
            (grid_string_cells_us(gc), grid_string_cells_us(lastgc)),
        ] {
            grid_string_cells_add_code(buf, &s, &of, &of_last, flags);
        }

        /* Append shift in/shift out if needed. */
        let charset = GRID_ATTR_CHARSET as u_int;
        if attr & charset != 0 && lastattr & charset == 0 {
            if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 {
                buf.push(b"\\016"); /* SO */
            } else {
                buf.push(b"\x0e"); /* SO */
            }
        }
        if attr & charset == 0 && lastattr & charset != 0 {
            if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 {
                buf.push(b"\\017"); /* SI */
            } else {
                buf.push(b"\x0f"); /* SI */
            }
        }

        /* Add hyperlink if changed. */
        if !sc.is_null() && lastgc.link != gc.link {
            let hyperlinks = (*sc).hyperlinks_ptr();
            if !hyperlinks.is_null() {
                if let Some((uri, internal_id, _)) = hyperlinks_get(&*hyperlinks, gc.link) {
                    *has_link = grid_string_cells_add_hyperlink(
                        buf,
                        internal_id.to_bytes(),
                        uri.to_bytes(),
                        flags,
                    );
                } else if *has_link {
                    grid_string_cells_add_hyperlink(buf, b"", b"", flags);
                    *has_link = false;
                }
            }
        }
    }
}

/// An owned copy of `text`.
fn copy_of(text: &[u8]) -> CString {
    unsafe { CString::from_vec_unchecked(text.to_vec()) }
}

/// The text of `nx` cells of one line, with the escape sequences that draw
/// them when `lastgc` is given a cell to carry the style along in. The
/// caller owns that cell and seeds it with the default one.
pub fn grid_string_cells(
    gd: &grid,
    px: u_int,
    py: u_int,
    nx: u_int,
    mut lastgc: Option<&mut grid_cell>,
    flags: c_int,
    s: *mut screen,
) -> CString {
    let Some(gl) = grid_peek_line(gd, py) else {
        return copy_of(b"");
    };
    let end = if flags & GRID_STRING_EMPTY_CELLS != 0 {
        gl.cellsize()
    } else {
        gl.cellused
    };

    let mut out: Vec<u8> = Vec::new();
    let mut code = Code::new();
    let mut has_link = false;
    let mut gc = scratch_cell();
    for xx in px..px.wrapping_add(nx) {
        if xx >= end {
            break;
        }
        gc = grid_get_cell(gd, xx, py);
        if gc.flags as c_int & GRID_FLAG_PADDING != 0 {
            continue;
        }

        if let Some(last) = lastgc.as_deref_mut()
            && flags & GRID_STRING_WITH_SEQUENCES != 0
        {
            grid_string_cells_code(last, &gc, &mut code, flags, s, &mut has_link);
            out.extend_from_slice(code.bytes());
            *last = gc;
        }

        if gc.flags as c_int & GRID_FLAG_TAB != 0 {
            out.push(b'\t');
        } else if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 && cell_bytes(&gc) == b"\\" {
            out.extend_from_slice(b"\\\\");
        } else {
            out.extend_from_slice(cell_bytes(&gc));
        }
    }

    if has_link {
        /*
         * The closing sequence goes on the end of whatever the last cell
         * left in the code buffer, which is written out a second time
         * with it.
         */
        grid_string_cells_add_hyperlink(&mut code, b"", b"", flags);
        out.extend_from_slice(code.bytes());
    }

    if flags & GRID_STRING_TRIM_SPACES != 0 {
        while out.last() == Some(&b' ') {
            out.pop();
        }
    }
    copy_of(&out)
}

/// Copy a set of lines from one grid to another. Both are big enough.
pub fn grid_duplicate_lines(dst: &mut grid, dy: u_int, src: &grid, sy: u_int, ny: u_int) {
    let mut ny = ny;
    if dy.wrapping_add(ny) > dst.hsize + dst.sy {
        ny = (dst.hsize + dst.sy).wrapping_sub(dy);
    }
    if sy.wrapping_add(ny) > src.hsize + src.sy {
        ny = (src.hsize + src.sy).wrapping_sub(sy);
    }
    grid_free_lines(dst, dy, ny);

    for yy in 0..ny {
        let srcl = line_at(src, sy + yy).clone();
        *line_at_mut(dst, dy + yy) = srcl;
    }
}

/// Mark a line as one the reflow has taken the contents of.
fn grid_reflow_dead(gl: &mut grid_line) {
    *gl = dead_line();
}

/// Add `n` lines to the end of a grid.
fn grid_reflow_add(gd: &mut grid, n: u_int) {
    let sy = gd.sy + n;
    gd.linedata.resize_with(sy as usize, grid_line::new);
    gd.sy = sy;
}

/// Move a line to the end of a grid, leaving it dead where it was.
fn grid_reflow_move(gd: &mut grid, from: &mut grid_line) {
    let moved = std::mem::replace(from, dead_line());
    grid_reflow_add(gd, 1);
    gd.linedata[(gd.sy - 1) as usize] = moved;
}

/// Join the lines below `yy` onto it, as much as the new width takes.
fn grid_reflow_join(
    target: &mut grid,
    gd: &mut grid,
    sx: u_int,
    yy: u_int,
    width: u_int,
    already: bool,
) {
    let mut width = width;

    /*
     * Add a new target line.
     */
    let to = if already {
        target.sy - 1
    } else {
        let to = target.sy;
        grid_reflow_move(target, line_at_mut(gd, yy));
        to
    };
    let mut at = line_at(target, to).cellused;

    /*
     * Loop until no more to consume or the target line is full.
     */
    let mut lines = 0;
    let mut wrapped = true;
    let mut from = None;
    let mut want = 0;
    let mut gc = scratch_cell();
    loop {
        /*
         * If this is now the last line, there is nothing more to be
         * done.
         */
        if yy + 1 + lines == gd.hsize + gd.sy {
            break;
        }
        let line = yy + 1 + lines;

        /* If the next line is empty, skip it. */
        if line_at(gd, line).flags & GRID_LINE_WRAPPED == 0 {
            wrapped = false;
        }
        if line_at(gd, line).cellused == 0 {
            if !wrapped {
                break;
            }
            lines += 1;
            continue;
        }

        /*
         * Is the destination line now full? Copy the first character
         * separately because we need to leave "from" set to the last
         * line if this line is full.
         */
        grid_get_cell1(line_at(gd, line), 0, &mut gc);
        if width + gc.data.width as u_int > sx {
            break;
        }
        width += gc.data.width as u_int;
        grid_set_cell(target, at, to, &gc);
        at += 1;

        /* Join as much more as possible onto the current line. */
        from = Some(line);
        let cellused = line_at(gd, line).cellused;
        want = 1;
        while want < cellused {
            grid_get_cell1(line_at(gd, line), want, &mut gc);
            if width + gc.data.width as u_int > sx {
                break;
            }
            width += gc.data.width as u_int;

            grid_set_cell(target, at, to, &gc);
            at += 1;
            want += 1;
        }
        lines += 1;

        /*
         * If this line wasn't wrapped or we didn't consume the entire
         * line, don't try to join any further lines.
         */
        if !wrapped || want != cellused || width == sx {
            break;
        }
    }
    let Some(from) = from.filter(|_| lines != 0) else {
        return;
    };

    /*
     * If we didn't consume the entire final line, then remove what we did
     * consume. If we consumed the entire line and it wasn't wrapped,
     * remove the wrap flag from this line.
     */
    let left = line_at(gd, from).cellused - want;
    if left != 0 {
        /*
         * The line the cells are moved along is the one the count of
         * consumed lines names, which is the line "from" names unless
         * empty lines were skipped after it.
         */
        grid_move_cells(gd, 0, want, yy + lines, left, 8);
        let gl = line_at_mut(gd, from);
        gl.cellused = left;
        gl.resize_cells(left);
        lines -= 1;
    } else if !wrapped {
        line_at_mut(target, to).flags &= !GRID_LINE_WRAPPED;
    }

    /* Remove the lines that were completely consumed. */
    for i in yy + 1..yy + 1 + lines {
        let gl = line_at_mut(gd, i);
        grid_reflow_dead(gl);
    }

    /* Adjust scroll position. */
    if gd.hscrolled > to + lines {
        gd.hscrolled -= lines;
    } else if gd.hscrolled > to {
        gd.hscrolled = to;
    }
}

/// Split a line that is too long for the new width into several new ones.
fn grid_reflow_split(target: &mut grid, gd: &mut grid, sx: u_int, yy: u_int, at: u_int) {
    let used = line_at(gd, yy).cellused;
    let flags = line_at(gd, yy).flags;
    let mut gc = scratch_cell();

    /* How many lines do we need to insert? We know we need at least two. */
    let lines = if flags & GRID_LINE_EXTENDED == 0 {
        1 + (used - 1) / sx
    } else {
        let mut lines = 2;
        let mut width = 0;
        for i in at..used {
            grid_get_cell1(line_at(gd, yy), i, &mut gc);
            if width + gc.data.width as u_int > sx {
                lines += 1;
                width = 0;
            }
            width += gc.data.width as u_int;
        }
        lines
    };

    /* Insert new lines. */
    let mut line = target.sy + 1;
    grid_reflow_add(target, lines);
    let first = target.sy - lines;

    /* Copy sections from the original line. */
    let mut width = 0;
    let mut xx = 0;
    for i in at..used {
        grid_get_cell1(line_at(gd, yy), i, &mut gc);
        if width + gc.data.width as u_int > sx {
            line_at_mut(target, line).flags |= GRID_LINE_WRAPPED;

            line += 1;
            width = 0;
            xx = 0;
        }
        width += gc.data.width as u_int;
        grid_set_cell(target, xx, line, &gc);
        xx += 1;
    }
    if flags & GRID_LINE_WRAPPED != 0 {
        line_at_mut(target, line).flags |= GRID_LINE_WRAPPED;
    }

    /* Move the remainder of the original line. */
    let gl = line_at_mut(gd, yy);
    gl.resize_cells(at);
    gl.cellused = at;
    gl.flags |= GRID_LINE_WRAPPED;
    let moved = std::mem::replace(gl, dead_line());
    *line_at_mut(target, first) = moved;

    /* Adjust the scroll position. */
    if yy <= gd.hscrolled {
        gd.hscrolled += lines - 1;
    }

    /*
     * If the original line had the wrapped flag and there is still space
     * in the last new line, try to join with the next lines.
     */
    if width < sx && flags & GRID_LINE_WRAPPED != 0 {
        grid_reflow_join(target, gd, sx, yy, width, true);
    }
}

/// Reflow the lines of a grid to a new width.
pub fn grid_reflow(gd: &mut grid, sx: u_int) {
    /*
     * Create a destination grid. This is just used as a container for the
     * line data and may not be fully valid.
     */
    let mut target = grid {
        flags: 0,
        sx: gd.sx,
        sy: 0,
        hscrolled: 0,
        hsize: 0,
        hlimit: 0,
        linedata: Vec::new(),
    };

    /*
     * Loop over each source line.
     */
    let mut gc = scratch_cell();
    for yy in 0..gd.hsize + gd.sy {
        let gl = line_at(gd, yy);
        if gl.flags & GRID_LINE_DEAD != 0 {
            continue;
        }
        let flags = gl.flags;

        /*
         * Work out the width of this line. at is the point at which the
         * available width is hit, and width is the full line width.
         */
        let (at, width) = if gl.flags & GRID_LINE_EXTENDED == 0 {
            let width = gl.cellused;
            (if width > sx { sx } else { width }, width)
        } else {
            let (mut at, mut width) = (0, 0);
            for i in 0..gl.cellused {
                grid_get_cell1(gl, i, &mut gc);
                if at == 0 && width + gc.data.width as u_int > sx {
                    at = i;
                }
                width += gc.data.width as u_int;
            }
            (at, width)
        };

        /*
         * If the line is exactly right, just move it across unchanged.
         */
        if width == sx {
            grid_reflow_move(&mut target, line_at_mut(gd, yy));
            continue;
        }

        /*
         * If the line is too big, it needs to be split, whether or not it
         * was previously wrapped.
         */
        if width > sx {
            grid_reflow_split(&mut target, gd, sx, yy, at);
            continue;
        }

        /*
         * If the line was previously wrapped, join as much as possible of
         * the next line.
         */
        if flags & GRID_LINE_WRAPPED != 0 {
            grid_reflow_join(&mut target, gd, sx, yy, width, false);
        } else {
            grid_reflow_move(&mut target, line_at_mut(gd, yy));
        }
    }

    /*
     * Replace the old grid with the new.
     */
    if target.sy < gd.sy {
        let missing = gd.sy - target.sy;
        grid_reflow_add(&mut target, missing);
    }
    gd.hsize = target.sy - gd.sy;
    if gd.hscrolled > gd.hsize {
        gd.hscrolled = gd.hsize;
    }
    gd.linedata = target.linedata;
}

/// The position of a cell counted over wrapped lines, where a run of wrapped
/// lines counts as one.
pub fn grid_wrap_position(gd: &grid, px: u_int, py: u_int) -> (u_int, u_int) {
    let mut ax = 0;
    let mut ay = 0;
    for yy in 0..py {
        let gl = line_at(gd, yy);
        if gl.flags & GRID_LINE_WRAPPED != 0 {
            ax += gl.cellused;
        } else {
            ax = 0;
            ay += 1;
        }
    }
    let wx = if px >= line_at(gd, py).cellused {
        UINT_MAX
    } else {
        ax + px
    };
    (wx, ay)
}

/// The cell a wrapped position names.
pub fn grid_unwrap_position(gd: &grid, wx: u_int, wy: u_int) -> (u_int, u_int) {
    let mut wx = wx;
    let mut yy = 0;
    let mut ay = 0;
    while yy < (gd.hsize + gd.sy).wrapping_sub(1) {
        if ay == wy {
            break;
        }
        if line_at(gd, yy).flags & GRID_LINE_WRAPPED == 0 {
            ay += 1;
        }
        yy += 1;
    }

    /*
     * yy is now 0 on the unwrapped line which contains wx. Walk forwards
     * until we find the end or the line now containing wx.
     */
    if wx == UINT_MAX {
        while line_at(gd, yy).flags & GRID_LINE_WRAPPED != 0 {
            yy += 1;
        }
        wx = line_at(gd, yy).cellused;
    } else {
        while line_at(gd, yy).flags & GRID_LINE_WRAPPED != 0 {
            let cellused = line_at(gd, yy).cellused;
            if wx < cellused {
                break;
            }
            wx -= cellused;
            yy += 1;
        }
    }
    (wx, yy)
}

/// How many columns of a line hold something other than trailing spaces.
pub fn grid_line_length(gd: &grid, py: u_int) -> u_int {
    let mut px = line_at(gd, py).cellsize();
    if px > gd.sx {
        px = gd.sx;
    }
    let mut gc = scratch_cell();
    while px > 0 {
        gc = grid_get_cell(gd, px - 1, py);
        if gc.flags as c_int & GRID_FLAG_PADDING != 0 || cell_bytes(&gc) != b" " {
            break;
        }
        px -= 1;
    }
    px
}

/// Whether the character at a position is in a set, and for a tab how many of
/// its columns are still to come.
pub fn grid_in_set(gd: &grid, px: u_int, py: u_int, set: &CStr) -> c_int {
    let mut gc = scratch_cell();
    gc = grid_get_cell(gd, px, py);
    if set.to_bytes().contains(&b'\t') {
        if gc.flags as c_int & GRID_FLAG_PADDING != 0 {
            /*
             * Walk back to the cell the padding belongs to. Padding at
             * the start of a line walks off the front, where the read of
             * a cell that far out answers with the default cell.
             */
            let mut pxx = px;
            let mut tmp_gc = scratch_cell();
            loop {
                pxx = pxx.wrapping_sub(1);
                tmp_gc = grid_get_cell(gd, pxx, py);
                if !(pxx > 0 && tmp_gc.flags as c_int & GRID_FLAG_PADDING != 0) {
                    break;
                }
            }
            if tmp_gc.flags as c_int & GRID_FLAG_TAB != 0 {
                return (tmp_gc.data.width as u_int).wrapping_sub(px.wrapping_sub(pxx)) as c_int;
            }
        } else if gc.flags as c_int & GRID_FLAG_TAB != 0 {
            return gc.data.width as c_int;
        }
    }
    if gc.flags as c_int & GRID_FLAG_PADDING != 0 {
        return 0;
    }
    unsafe { utf8_cstrhas(set.as_ptr(), &gc.data) }
}

#[cfg(test)]
#[path = "../tests/test_grid.rs"]
mod tests;

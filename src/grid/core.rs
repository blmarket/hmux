use crate::src::ffi::libc::{memcmp, memcpy, memmove, memset, strchr, strlcat, strlen};
use crate::src::format::bytes::xformat;
use crate::src::hyperlinks::hyperlinks_get;
use crate::src::log::{fatalx, log_cstr, log_debug};
use crate::src::server::current_time;
use crate::src::shared::abi::*;
pub use crate::src::shared::colour::{COLOUR_FLAG_256, COLOUR_FLAG_RGB, COLOUR_FLAG_THEME};
use crate::src::shared::grid::*;
pub use crate::src::shared::limits::UINT_MAX;
pub use crate::src::shared::screen::screen;
use crate::src::style::colour::{colour_split_rgb, colour_theme_terminal_colour};
use crate::src::text::utf8::{
    utf8_build_one, utf8_cstrhas, utf8_from_data, utf8_has_whitespace, utf8_set, utf8_to_data,
};
use crate::src::tmux::start_time;
use std::ffi::{CStr, CString};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_1 {
    pub mask: u_int,
    pub code: u_int,
}
pub static grid_default_cell: grid_cell = grid_cell {
    data: utf8_data {
        data: [
            ' ' as i32 as u_char,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        have: 0 as u_char,
        size: 1 as u_char,
        width: 1 as u_char,
    },
    attr: 0 as u_short,
    flags: 0 as u_char,
    fg: 8 as ::core::ffi::c_int,
    bg: 8 as ::core::ffi::c_int,
    us: 8 as ::core::ffi::c_int,
    link: 0 as u_int,
};
static grid_padding_cell: grid_cell = grid_cell {
    data: utf8_data {
        data: [
            '!' as i32 as u_char,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        have: 0 as u_char,
        size: 0 as u_char,
        width: 0 as u_char,
    },
    attr: 0 as u_short,
    flags: GRID_FLAG_PADDING as u_char,
    fg: 8 as ::core::ffi::c_int,
    bg: 8 as ::core::ffi::c_int,
    us: 8 as ::core::ffi::c_int,
    link: 0 as u_int,
};
static grid_cleared_cell: grid_cell = grid_cell {
    data: utf8_data {
        data: [
            ' ' as i32 as u_char,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        have: 0 as u_char,
        size: 1 as u_char,
        width: 1 as u_char,
    },
    attr: 0 as u_short,
    flags: GRID_FLAG_CLEARED as u_char,
    fg: 8 as ::core::ffi::c_int,
    bg: 8 as ::core::ffi::c_int,
    us: 8 as ::core::ffi::c_int,
    link: 0 as u_int,
};
static grid_cleared_entry: grid_cell_entry = grid_cell_entry {
    c2rust_unnamed: grid_cell_entry_storage {
        data: grid_cell_entry_data {
            attr: 0 as u_char,
            fg: 8 as u_char,
            bg: 8 as u_char,
            data: ' ' as i32 as u_char,
        },
    },
    flags: GRID_FLAG_CLEARED as u_char,
};
pub unsafe fn grid_check_is_clear() {}
fn grid_store_cell(entry: &mut grid_cell_entry, cell: &grid_cell, byte: u_char) {
    entry.flags = (cell.flags as i32 & !GRID_FLAG_CLEARED) as u_char;
    if cell.fg & COLOUR_FLAG_256 != 0 {
        entry.flags |= GRID_FLAG_FG256 as u_char;
    }
    if cell.bg & COLOUR_FLAG_256 != 0 {
        entry.flags |= GRID_FLAG_BG256 as u_char;
    }
    entry.c2rust_unnamed.data = grid_cell_entry_data {
        fg: (cell.fg & 0xff) as u_char,
        bg: (cell.bg & 0xff) as u_char,
        attr: cell.attr as u_char,
        data: byte,
    };
}
fn grid_need_extended_cell(entry: &grid_cell_entry, cell: &grid_cell) -> bool {
    entry.flags as i32 & GRID_FLAG_EXTENDED != 0
        || cell.attr > 0xff
        || cell.data.size > 1
        || cell.data.width > 1
        || cell.fg & (COLOUR_FLAG_RGB | COLOUR_FLAG_THEME) != 0
        || cell.bg & (COLOUR_FLAG_RGB | COLOUR_FLAG_THEME) != 0
        || cell.us != 8
        || cell.link != 0
        || cell.flags as i32 & GRID_FLAG_TAB != 0
}
fn grid_get_extended_cell(line: &mut grid_line, column: usize, flags: i32) {
    let count = line.extdsize.wrapping_add(1);
    line.extddata
        .resize_with(count as usize, grid_extd_entry::default);
    line.extdsize = count;
    let entry = &mut line.celldata[column];
    entry.c2rust_unnamed.offset = count.wrapping_sub(1);
    entry.flags = (flags | GRID_FLAG_EXTENDED) as u_char;
}
unsafe fn grid_extended_cell<'a>(
    line: &'a mut grid_line,
    column: usize,
    cell: &grid_cell,
) -> &'a mut grid_extd_entry {
    let flags = cell.flags as i32 & !GRID_FLAG_CLEARED;
    let entry = &line.celldata[column];
    if entry.flags as i32 & GRID_FLAG_EXTENDED == 0 {
        grid_get_extended_cell(line, column, flags);
    } else if entry.c2rust_unnamed.offset >= line.extdsize {
        fatalx(|out| out.write_all(b"offset too big"));
    }
    line.flags |= GRID_LINE_EXTENDED as u_short;
    if cell.link != 0 {
        line.flags |= GRID_LINE_HYPERLINK as u_short;
    }
    let mut packed = 0;
    if cell.flags as i32 & GRID_FLAG_TAB != 0 {
        packed = cell.data.width as utf8_char;
    } else {
        utf8_from_data(&cell.data, &mut packed);
    }
    let offset = line.celldata[column].c2rust_unnamed.offset;
    let extended = &mut line.extddata[offset as usize];
    *extended = grid_extd_entry {
        data: packed,
        attr: cell.attr,
        flags: flags as u_char,
        fg: cell.fg,
        bg: cell.bg,
        us: cell.us,
        link: cell.link,
    };
    extended
}
unsafe fn grid_compact_line(gl: &mut grid_line) {
    if gl.extdsize == 0 {
        return;
    }
    let cells = &mut gl.celldata[..gl.cellsize as usize];
    let count = cells
        .iter()
        .filter(|cell| cell.flags as i32 & GRID_FLAG_EXTENDED != 0)
        .count();
    if count == 0 {
        gl.extddata.clear();
        gl.extdsize = 0;
        return;
    }
    let mut compact = GridArray::default();
    compact.resize_with(count, grid_extd_entry::default);
    let mut index = 0;
    for cell in cells {
        if cell.flags as i32 & GRID_FLAG_EXTENDED != 0 {
            compact[index] = gl.extddata[cell.c2rust_unnamed.offset as usize];
            cell.c2rust_unnamed.offset = index as u_int;
            index += 1;
        }
    }
    gl.extddata = compact;
    gl.extdsize = count as u_int;
}
pub unsafe fn grid_get_line(mut gd: *mut grid, mut line: u_int) -> *mut grid_line {
    return (*gd).linedata.as_mut_ptr().offset(line as isize) as *mut grid_line;
}
pub unsafe fn grid_line_time(gl: &grid_line) -> time_t {
    if gl.time == 0 {
        return 0;
    }
    start_time.tv_sec as time_t + gl.time as time_t - 1
}
unsafe fn grid_line_set_time(gl: &mut grid_line) {
    gl.time = if current_time == 0 {
        0
    } else {
        (current_time as __time_t - start_time.tv_sec + 1) as u_int
    };
}
pub fn grid_adjust_lines(gd: &mut grid, lines: u_int) {
    gd.linedata.resize_with(lines as usize, grid_line::default);
}
unsafe fn grid_clear_cell(line: &mut grid_line, px: u_int, bg: u_int, moved: bool) {
    let column = px as usize;
    let entry = &mut line.celldata[column];
    let old_offset = entry.c2rust_unnamed.offset;
    let had_extended = entry.flags as i32 & GRID_FLAG_EXTENDED != 0;
    *entry = grid_cleared_entry;
    if !moved && had_extended && old_offset < line.extdsize {
        entry.flags |= GRID_FLAG_EXTENDED as u_char;
        entry.c2rust_unnamed.offset = old_offset;
        let extended = grid_extended_cell(line, column, &grid_cleared_cell);
        if bg != 8 {
            extended.bg = bg as i32;
        }
    } else if bg != 8 {
        if bg & (COLOUR_FLAG_RGB | COLOUR_FLAG_THEME) as u_int != 0 {
            let flags = entry.flags as i32;
            grid_get_extended_cell(line, column, flags);
            grid_extended_cell(line, column, &grid_cleared_cell).bg = bg as i32;
        } else {
            if bg & COLOUR_FLAG_256 as u_int != 0 {
                entry.flags |= GRID_FLAG_BG256 as u_char;
            }
            entry.c2rust_unnamed.data.bg = bg as u_char;
        }
    }
}
unsafe fn grid_check_y(gd: &grid, from: &str, py: u_int) -> i32 {
    if py >= gd.hsize.wrapping_add(gd.sy) {
        log_debug(format_args!("{}: y out of range: {}", from, py));
        return -1;
    }
    0
}
pub fn grid_cells_look_equal(gc1: &grid_cell, gc2: &grid_cell) -> bool {
    gc1.fg == gc2.fg
        && gc1.bg == gc2.bg
        && gc1.attr == gc2.attr
        && (gc1.flags as i32 & !GRID_FLAG_CLEARED) == (gc2.flags as i32 & !GRID_FLAG_CLEARED)
        && gc1.link == gc2.link
}
pub fn grid_cells_equal(gc1: &grid_cell, gc2: &grid_cell) -> bool {
    grid_cells_look_equal(gc1, gc2)
        && gc1.data.width == gc2.data.width
        && gc1.data.size == gc2.data.size
        && gc1.data.data[..gc1.data.size as usize] == gc2.data.data[..gc2.data.size as usize]
}
pub fn grid_set_tab(gc: &mut grid_cell, width: u_int) {
    gc.data.data.fill(0);
    gc.flags |= GRID_FLAG_TAB as u_char;
    gc.flags &= !GRID_FLAG_PADDING as u_char;
    gc.data.have = width as u_char;
    gc.data.size = gc.data.have;
    gc.data.width = gc.data.size;
    gc.data.data[..gc.data.size as usize].fill(b' ');
}
fn grid_free_line(gd: &mut grid, py: u_int) {
    gd.linedata[py as usize] = grid_line::default();
}
pub fn grid_free_lines(gd: &mut grid, py: u_int, ny: u_int) {
    for row in py..py.wrapping_add(ny) {
        grid_free_line(gd, row);
    }
}
pub(crate) unsafe fn grid_create_box(sx: u_int, sy: u_int, hlimit: u_int) -> Box<grid> {
    let mut owner = Box::new(grid::default());
    owner.sx = sx;
    owner.sy = sy;
    if hlimit != 0 {
        owner.flags = GRID_HISTORY;
    }
    owner.hlimit = hlimit;
    owner.linedata.resize_with(sy as usize, grid_line::default);
    grid_check_is_clear();
    owner
}
pub unsafe fn grid_create(sx: u_int, sy: u_int, hlimit: u_int) -> *mut grid {
    Box::into_raw(grid_create_box(sx, sy, hlimit))
}
pub unsafe fn grid_destroy(gd: *mut grid) {
    drop(Box::from_raw(gd));
}
pub unsafe fn grid_compare(ga: &grid, gb: &grid) -> i32 {
    if ga.sx != gb.sx || ga.sy != gb.sy {
        return 1;
    }
    let mut a = grid_cell::default();
    let mut b = grid_cell::default();
    // The comparison deliberately starts at line zero, without a history offset.
    for row in 0..ga.sy {
        let a_line = &ga.linedata[row as usize];
        let b_line = &gb.linedata[row as usize];
        if a_line.cellsize != b_line.cellsize {
            return 1;
        }
        for column in 0..a_line.cellsize as u_int {
            grid_get_cell(ga, column, row, &mut a);
            grid_get_cell(gb, column, row, &mut b);
            if !grid_cells_equal(&a, &b) {
                return 1;
            }
        }
    }
    0
}

fn grid_trim_history(gd: &mut grid, ny: u_int) {
    grid_free_lines(gd, 0, ny);
    let live = gd.hsize.wrapping_add(gd.sy) as usize;
    gd.linedata[..live].rotate_left(ny as usize);
}
pub fn grid_collect_history(gd: &mut grid, all: i32) {
    if gd.hsize == 0 || gd.hsize < gd.hlimit {
        return;
    }
    let count = if all != 0 {
        gd.hsize.wrapping_sub(gd.hlimit)
    } else {
        gd.hlimit / 10
    }
    .max(1)
    .min(gd.hsize);
    grid_trim_history(gd, count);
    gd.hsize = gd.hsize.wrapping_sub(count);
    gd.scroll_collected = gd.scroll_collected.wrapping_add(count);
    gd.hscrolled = gd.hscrolled.min(gd.hsize);
}
pub fn grid_remove_history(gd: &mut grid, ny: u_int) {
    if ny > gd.hsize {
        return;
    }
    let start = gd.hsize.wrapping_add(gd.sy).wrapping_sub(ny);
    for offset in 0..ny {
        grid_free_line(gd, start.wrapping_add(offset));
    }
    gd.hsize = gd.hsize.wrapping_sub(ny);
}
pub unsafe fn grid_scroll_history(gd: &mut grid, bg: u_int) {
    let end = gd.hsize.wrapping_add(gd.sy);
    gd.linedata
        .resize_with(end.wrapping_add(1) as usize, grid_line::default);
    grid_empty_line(gd, end, bg);
    gd.hscrolled = gd.hscrolled.wrapping_add(1);
    let history = &mut gd.linedata[gd.hsize as usize];
    grid_compact_line(history);
    grid_line_set_time(history);
    gd.hsize = gd.hsize.wrapping_add(1);
    gd.scroll_added = gd.scroll_added.wrapping_add(1);
}
pub fn grid_clear_history(gd: &mut grid) {
    grid_trim_history(gd, gd.hsize);
    gd.hscrolled = 0;
    gd.hsize = 0;
    gd.scroll_generation = gd.scroll_generation.wrapping_add(1);
    gd.linedata.resize_with(gd.sy as usize, grid_line::default);
}
pub unsafe fn grid_scroll_history_region(gd: &mut grid, upper: u_int, lower: u_int, bg: u_int) {
    let end = gd.hsize.wrapping_add(gd.sy);
    gd.linedata
        .resize_with(end.wrapping_add(1) as usize, grid_line::default);
    let history = gd.hsize as usize;
    gd.linedata[history..=end as usize].rotate_right(1);
    let upper = upper.wrapping_add(1);
    let lower = lower.wrapping_add(1);
    gd.linedata.swap(history, upper as usize);
    grid_line_set_time(&mut gd.linedata[history]);
    gd.linedata[upper as usize..=lower as usize].rotate_left(1);
    grid_empty_line(gd, lower, bg);
    gd.hscrolled = gd.hscrolled.wrapping_add(1);
    gd.hsize = gd.hsize.wrapping_add(1);
    gd.scroll_added = gd.scroll_added.wrapping_add(1);
}
unsafe fn grid_expand_line(gd: &mut grid, py: u_int, mut width: u_int, bg: u_int) {
    let line = &mut gd.linedata[py as usize];
    if width <= line.cellsize as u_int {
        return;
    }
    if width < gd.sx / 4 {
        width = gd.sx / 4;
    } else if width < gd.sx / 2 {
        width = gd.sx / 2;
    } else if width < gd.sx {
        width = gd.sx;
    }
    line.celldata
        .resize_with(width as usize, grid_cell_entry::default);
    for column in line.cellsize as u_int..width {
        grid_clear_cell(line, column, bg, false);
    }
    line.cellsize = width as u_short;
}
pub unsafe fn grid_empty_line(gd: &mut grid, py: u_int, bg: u_int) {
    gd.linedata[py as usize] = grid_line::default();
    if bg != 8 && bg != 9 {
        grid_expand_line(gd, py, gd.sx, bg);
    }
}
pub unsafe fn grid_peek_line(gd: &grid, py: u_int) -> Option<&grid_line> {
    if grid_check_y(gd, "grid_peek_line", py) != 0 {
        return None;
    }
    Some(&gd.linedata[py as usize])
}
unsafe fn grid_get_cell1(gl: &grid_line, px: u_int, gc: &mut grid_cell) {
    let entry = &gl.celldata[px as usize];
    if entry.flags as i32 & GRID_FLAG_EXTENDED != 0 {
        let offset = entry.c2rust_unnamed.offset;
        if offset >= gl.extdsize {
            *gc = grid_default_cell;
        } else {
            let extended = &gl.extddata[offset as usize];
            gc.flags = extended.flags;
            gc.attr = extended.attr;
            gc.fg = extended.fg;
            gc.bg = extended.bg;
            gc.us = extended.us;
            gc.link = extended.link;
            if gc.flags as i32 & GRID_FLAG_TAB != 0 {
                grid_set_tab(gc, extended.data);
            } else {
                utf8_to_data(extended.data, &mut gc.data);
            }
        }
        return;
    }
    let data = entry.c2rust_unnamed.data;
    gc.flags = (entry.flags as i32 & !(GRID_FLAG_FG256 | GRID_FLAG_BG256)) as u_char;
    gc.attr = data.attr as u_short;
    gc.fg = data.fg as i32;
    if entry.flags as i32 & GRID_FLAG_FG256 != 0 {
        gc.fg |= COLOUR_FLAG_256;
    }
    gc.bg = data.bg as i32;
    if entry.flags as i32 & GRID_FLAG_BG256 != 0 {
        gc.bg |= COLOUR_FLAG_256;
    }
    gc.us = 8;
    utf8_set(&mut gc.data, data.data);
    gc.link = 0;
}
pub unsafe fn grid_get_cell(gd: &grid, px: u_int, py: u_int, gc: &mut grid_cell) {
    if grid_check_y(gd, "grid_get_cell", py) != 0
        || px >= gd.linedata[py as usize].cellsize as u_int
    {
        *gc = grid_default_cell;
    } else {
        grid_get_cell1(&gd.linedata[py as usize], px, gc);
    }
}
pub unsafe fn grid_set_cell(gd: &mut grid, px: u_int, py: u_int, gc: &grid_cell) {
    if grid_check_y(gd, "grid_set_cell", py) != 0 {
        return;
    }
    grid_expand_line(gd, py, px.wrapping_add(1), 8);
    let line = &mut gd.linedata[py as usize];
    if px.wrapping_add(1) > line.cellused as u_int {
        line.cellused = px.wrapping_add(1) as u_short;
    }
    let column = px as usize;
    if grid_need_extended_cell(&line.celldata[column], gc) {
        grid_extended_cell(line, column, gc);
    } else {
        grid_store_cell(&mut line.celldata[column], gc, gc.data.data[0]);
    }
}
pub unsafe fn grid_set_padding(gd: &mut grid, px: u_int, py: u_int, bg: i32) {
    let mut cell = grid_padding_cell;
    cell.bg = bg;
    grid_set_cell(gd, px, py, &cell);
}
pub unsafe fn grid_set_cells(
    gd: &mut grid,
    px: u_int,
    py: u_int,
    gc: &grid_cell,
    bytes: &[std::ffi::c_char],
) {
    if grid_check_y(gd, "grid_set_cells", py) != 0 {
        return;
    }
    let end = (px as usize).wrapping_add(bytes.len());
    grid_expand_line(gd, py, end as u_int, 8);
    let line = &mut gd.linedata[py as usize];
    if end > line.cellused as usize {
        line.cellused = end as u_short;
    }
    for (index, &byte) in bytes.iter().enumerate() {
        let column = px.wrapping_add(index as u_int) as usize;
        if grid_need_extended_cell(&line.celldata[column], gc) {
            grid_extended_cell(line, column, gc).data = utf8_build_one(byte as u_char);
        } else {
            grid_store_cell(&mut line.celldata[column], gc, byte as u_char);
        }
    }
}
pub unsafe fn grid_clear(gd: &mut grid, px: u_int, py: u_int, nx: u_int, ny: u_int, bg: u_int) {
    if nx == 0 || ny == 0 {
        return;
    }
    if px == 0 && nx == gd.sx {
        grid_clear_lines(gd, py, ny, bg);
        return;
    }
    if grid_check_y(gd, "grid_clear", py) != 0
        || grid_check_y(gd, "grid_clear", py.wrapping_add(ny).wrapping_sub(1)) != 0
    {
        return;
    }
    for row in py..py.wrapping_add(ny) {
        let width = gd.sx.min(gd.linedata[row as usize].cellsize as u_int);
        let mut count = nx;
        if bg == 8 || bg == 9 {
            if px > width {
                continue;
            }
            if px.wrapping_add(nx) > width {
                count = width.wrapping_sub(px);
            }
        }
        grid_expand_line(gd, row, px.wrapping_add(count), 8);
        let line = &mut gd.linedata[row as usize];
        for column in px..px.wrapping_add(count) {
            grid_clear_cell(line, column, bg, false);
        }
    }
}
pub unsafe fn grid_clear_lines(gd: &mut grid, py: u_int, ny: u_int, bg: u_int) {
    if ny == 0
        || grid_check_y(gd, "grid_clear_lines", py) != 0
        || grid_check_y(gd, "grid_clear_lines", py.wrapping_add(ny).wrapping_sub(1)) != 0
    {
        return;
    }
    for row in py..py.wrapping_add(ny) {
        grid_free_line(gd, row);
        grid_empty_line(gd, row, bg);
    }
    if py != 0 {
        gd.linedata[(py - 1) as usize].flags &= !GRID_LINE_WRAPPED as u_short;
    }
}
pub unsafe fn grid_move_lines(gd: &mut grid, dy: u_int, py: u_int, ny: u_int, bg: u_int) {
    if ny == 0 || py == dy {
        return;
    }
    let source_end = py.wrapping_add(ny);
    let destination_end = dy.wrapping_add(ny);
    for row in [
        py,
        source_end.wrapping_sub(1),
        dy,
        destination_end.wrapping_sub(1),
    ] {
        if grid_check_y(gd, "grid_move_lines", row) != 0 {
            return;
        }
    }
    for row in dy..destination_end {
        if row < py || row >= source_end {
            grid_free_line(gd, row);
        }
    }
    if dy != 0 {
        gd.linedata[(dy - 1) as usize].flags &= !GRID_LINE_WRAPPED as u_short;
    }
    let lines = &mut gd.linedata;
    if dy < py {
        for offset in 0..ny as usize {
            lines[dy as usize + offset] = std::mem::take(&mut lines[py as usize + offset]);
        }
    } else {
        for offset in (0..ny as usize).rev() {
            lines[dy as usize + offset] = std::mem::take(&mut lines[py as usize + offset]);
        }
    }
    for row in py..source_end {
        if row < dy || row >= destination_end {
            grid_empty_line(gd, row, bg);
        }
    }
    if py != 0 && (py < dy || py >= destination_end) {
        gd.linedata[(py - 1) as usize].flags &= !GRID_LINE_WRAPPED as u_short;
    }
}
pub unsafe fn grid_move_cells(
    gd: &mut grid,
    dx: u_int,
    px: u_int,
    py: u_int,
    nx: u_int,
    bg: u_int,
) {
    if nx == 0 || px == dx || grid_check_y(gd, "grid_move_cells", py) != 0 {
        return;
    }
    grid_expand_line(gd, py, px.wrapping_add(nx), 8);
    grid_expand_line(gd, py, dx.wrapping_add(nx), 8);
    let line = &mut gd.linedata[py as usize];
    line.celldata
        .copy_within(px as usize..px as usize + nx as usize, dx as usize);
    if dx.wrapping_add(nx) > line.cellused as u_int {
        line.cellused = dx.wrapping_add(nx) as u_short;
    }
    for column in px..px.wrapping_add(nx) {
        if column < dx || column >= dx.wrapping_add(nx) {
            grid_clear_cell(line, column, bg, true);
        }
    }
}
unsafe fn grid_string_cells_fg(
    mut gc: *const grid_cell,
    mut values: *mut ::core::ffi::c_int,
) -> size_t {
    let mut n: size_t = 0;
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    let mut c: ::core::ffi::c_int = 0;
    n = 0 as size_t;
    if (*gc).fg & COLOUR_FLAG_THEME != 0 {
        c = colour_theme_terminal_colour(((*gc).fg & 0xff as ::core::ffi::c_int) as u_int);
        if c == 8 as ::core::ffi::c_int {
            let fresh31 = n;
            n = n.wrapping_add(1);
            *values.offset(fresh31 as isize) = 39 as ::core::ffi::c_int;
        } else {
            let fresh32 = n;
            n = n.wrapping_add(1);
            *values.offset(fresh32 as isize) = c + 30 as ::core::ffi::c_int;
        }
    } else if (*gc).fg & COLOUR_FLAG_256 != 0 {
        let fresh33 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh33 as isize) = 38 as ::core::ffi::c_int;
        let fresh34 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh34 as isize) = 5 as ::core::ffi::c_int;
        let fresh35 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh35 as isize) = (*gc).fg & 0xff as ::core::ffi::c_int;
    } else if (*gc).fg & COLOUR_FLAG_RGB != 0 {
        let fresh36 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh36 as isize) = 38 as ::core::ffi::c_int;
        let fresh37 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh37 as isize) = 2 as ::core::ffi::c_int;
        colour_split_rgb((*gc).fg, &raw mut r, &raw mut g, &raw mut b);
        let fresh38 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh38 as isize) = r as ::core::ffi::c_int;
        let fresh39 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh39 as isize) = g as ::core::ffi::c_int;
        let fresh40 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh40 as isize) = b as ::core::ffi::c_int;
    } else {
        match (*gc).fg {
            0..=7 => {
                let fresh41 = n;
                n = n.wrapping_add(1);
                *values.offset(fresh41 as isize) = (*gc).fg + 30 as ::core::ffi::c_int;
            }
            8 => {
                let fresh42 = n;
                n = n.wrapping_add(1);
                *values.offset(fresh42 as isize) = 39 as ::core::ffi::c_int;
            }
            90..=97 => {
                let fresh43 = n;
                n = n.wrapping_add(1);
                *values.offset(fresh43 as isize) = (*gc).fg;
            }
            _ => {}
        }
    }
    return n;
}
unsafe fn grid_string_cells_bg(
    mut gc: *const grid_cell,
    mut values: *mut ::core::ffi::c_int,
) -> size_t {
    let mut n: size_t = 0;
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    let mut c: ::core::ffi::c_int = 0;
    n = 0 as size_t;
    if (*gc).bg & COLOUR_FLAG_THEME != 0 {
        c = colour_theme_terminal_colour(((*gc).bg & 0xff as ::core::ffi::c_int) as u_int);
        if c == 8 as ::core::ffi::c_int {
            let fresh18 = n;
            n = n.wrapping_add(1);
            *values.offset(fresh18 as isize) = 49 as ::core::ffi::c_int;
        } else {
            let fresh19 = n;
            n = n.wrapping_add(1);
            *values.offset(fresh19 as isize) = c + 40 as ::core::ffi::c_int;
        }
    } else if (*gc).bg & COLOUR_FLAG_256 != 0 {
        let fresh20 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh20 as isize) = 48 as ::core::ffi::c_int;
        let fresh21 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh21 as isize) = 5 as ::core::ffi::c_int;
        let fresh22 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh22 as isize) = (*gc).bg & 0xff as ::core::ffi::c_int;
    } else if (*gc).bg & COLOUR_FLAG_RGB != 0 {
        let fresh23 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh23 as isize) = 48 as ::core::ffi::c_int;
        let fresh24 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh24 as isize) = 2 as ::core::ffi::c_int;
        colour_split_rgb((*gc).bg, &raw mut r, &raw mut g, &raw mut b);
        let fresh25 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh25 as isize) = r as ::core::ffi::c_int;
        let fresh26 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh26 as isize) = g as ::core::ffi::c_int;
        let fresh27 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh27 as isize) = b as ::core::ffi::c_int;
    } else {
        match (*gc).bg {
            0..=7 => {
                let fresh28 = n;
                n = n.wrapping_add(1);
                *values.offset(fresh28 as isize) = (*gc).bg + 40 as ::core::ffi::c_int;
            }
            8 => {
                let fresh29 = n;
                n = n.wrapping_add(1);
                *values.offset(fresh29 as isize) = 49 as ::core::ffi::c_int;
            }
            90..=97 => {
                let fresh30 = n;
                n = n.wrapping_add(1);
                *values.offset(fresh30 as isize) = (*gc).bg + 10 as ::core::ffi::c_int;
            }
            _ => {}
        }
    }
    return n;
}
unsafe fn grid_string_cells_us(
    mut gc: *const grid_cell,
    mut values: *mut ::core::ffi::c_int,
) -> size_t {
    let mut n: size_t = 0;
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    let mut c: ::core::ffi::c_int = 0;
    n = 0 as size_t;
    if (*gc).us & COLOUR_FLAG_THEME != 0 {
        c = colour_theme_terminal_colour(((*gc).us & 0xff as ::core::ffi::c_int) as u_int);
        if c == 8 as ::core::ffi::c_int {
            let fresh6 = n;
            n = n.wrapping_add(1);
            *values.offset(fresh6 as isize) = 59 as ::core::ffi::c_int;
        } else {
            let fresh7 = n;
            n = n.wrapping_add(1);
            *values.offset(fresh7 as isize) = 58 as ::core::ffi::c_int;
            let fresh8 = n;
            n = n.wrapping_add(1);
            *values.offset(fresh8 as isize) = 5 as ::core::ffi::c_int;
            let fresh9 = n;
            n = n.wrapping_add(1);
            *values.offset(fresh9 as isize) = c;
        }
    } else if (*gc).us & COLOUR_FLAG_256 != 0 {
        let fresh10 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh10 as isize) = 58 as ::core::ffi::c_int;
        let fresh11 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh11 as isize) = 5 as ::core::ffi::c_int;
        let fresh12 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh12 as isize) = (*gc).us & 0xff as ::core::ffi::c_int;
    } else if (*gc).us & COLOUR_FLAG_RGB != 0 {
        let fresh13 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh13 as isize) = 58 as ::core::ffi::c_int;
        let fresh14 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh14 as isize) = 2 as ::core::ffi::c_int;
        colour_split_rgb((*gc).us, &raw mut r, &raw mut g, &raw mut b);
        let fresh15 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh15 as isize) = r as ::core::ffi::c_int;
        let fresh16 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh16 as isize) = g as ::core::ffi::c_int;
        let fresh17 = n;
        n = n.wrapping_add(1);
        *values.offset(fresh17 as isize) = b as ::core::ffi::c_int;
    }
    return n;
}
unsafe fn grid_string_cells_add_code(
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut n: u_int,
    mut s: *mut ::core::ffi::c_int,
    mut newc: *mut ::core::ffi::c_int,
    mut oldc: *mut ::core::ffi::c_int,
    mut nnewc: size_t,
    mut noldc: size_t,
    mut flags: ::core::ffi::c_int,
) {
    let mut i: u_int = 0;
    let mut tmp: [::core::ffi::c_char; 64] = [0; 64];
    let mut reset: ::core::ffi::c_int = (n != 0 as u_int
        && *s.offset(0 as ::core::ffi::c_int as isize) == 0 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    if nnewc == 0 as size_t {
        return;
    }
    if reset == 0
        && nnewc == noldc
        && memcmp(
            newc as *const ::core::ffi::c_void,
            oldc as *const ::core::ffi::c_void,
            nnewc.wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>() as size_t),
        ) == 0 as ::core::ffi::c_int
    {
        return;
    }
    if reset != 0
        && (*newc.offset(0 as ::core::ffi::c_int as isize) == 49 as ::core::ffi::c_int
            || *newc.offset(0 as ::core::ffi::c_int as isize) == 39 as ::core::ffi::c_int)
    {
        return;
    }
    if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 {
        strlcat(
            buf,
            b"\\033[\0" as *const u8 as *const ::core::ffi::c_char,
            len,
        );
    } else {
        strlcat(
            buf,
            b"\x1B[\0" as *const u8 as *const ::core::ffi::c_char,
            len,
        );
    }
    i = 0 as u_int;
    while (i as size_t) < nnewc {
        if (i.wrapping_add(1 as u_int) as size_t) < nnewc {
            xformat(
                &mut tmp,
                format_args!("{};", (*newc.offset(i as isize)) as i32),
            );
        } else {
            xformat(
                &mut tmp,
                format_args!("{}", (*newc.offset(i as isize)) as i32),
            );
        }
        strlcat(buf, &raw mut tmp as *mut ::core::ffi::c_char, len);
        i = i.wrapping_add(1);
    }
    strlcat(buf, b"m\0" as *const u8 as *const ::core::ffi::c_char, len);
}
unsafe fn grid_string_cells_add_hyperlink(
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut id: *const ::core::ffi::c_char,
    mut uri: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if strlen(uri)
        .wrapping_add(strlen(id))
        .wrapping_add(17 as size_t)
        >= len
    {
        return 0 as ::core::ffi::c_int;
    }
    if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 {
        strlcat(
            buf,
            b"\\033]8;\0" as *const u8 as *const ::core::ffi::c_char,
            len,
        );
    } else {
        strlcat(
            buf,
            b"\x1B]8;\0" as *const u8 as *const ::core::ffi::c_char,
            len,
        );
    }
    if *id as ::core::ffi::c_int != '\0' as i32 {
        let id_bytes = CStr::from_ptr(id).to_bytes();
        let mut bytes = Vec::with_capacity(id_bytes.len() + 4);
        bytes.extend_from_slice(b"id=");
        bytes.extend_from_slice(id_bytes);
        bytes.push(b';');
        let tmp = CString::new(bytes).expect("C string ID contains no interior NUL");
        strlcat(buf, tmp.as_ptr(), len);
    } else {
        strlcat(buf, b";\0" as *const u8 as *const ::core::ffi::c_char, len);
    }
    strlcat(buf, uri, len);
    if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 {
        strlcat(
            buf,
            b"\\033\\\\\0" as *const u8 as *const ::core::ffi::c_char,
            len,
        );
    } else {
        strlcat(
            buf,
            b"\x1B\\\0" as *const u8 as *const ::core::ffi::c_char,
            len,
        );
    }
    return 1 as ::core::ffi::c_int;
}
unsafe fn grid_string_cells_code(
    mut lastgc: *const grid_cell,
    mut gc: *const grid_cell,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut flags: ::core::ffi::c_int,
    mut sc: *mut screen,
    mut has_link: *mut ::core::ffi::c_int,
) {
    let mut oldc: [::core::ffi::c_int; 64] = [0; 64];
    let mut newc: [::core::ffi::c_int; 64] = [0; 64];
    let mut s: [::core::ffi::c_int; 128] = [0; 128];
    let mut noldc: size_t = 0;
    let mut nnewc: size_t = 0;
    let mut n: size_t = 0;
    let mut i: size_t = 0;
    let mut attr: u_int = (*gc).attr as u_int;
    let mut lastattr: u_int = (*lastgc).attr as u_int;
    let mut tmp: [::core::ffi::c_char; 64] = [0; 64];
    let mut uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    static mut attrs: [C2RustUnnamed_1; 13] = [
        C2RustUnnamed_1 {
            mask: GRID_ATTR_BRIGHT as u_int,
            code: 1 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_DIM as u_int,
            code: 2 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_ITALICS as u_int,
            code: 3 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_UNDERSCORE as u_int,
            code: 4 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_BLINK as u_int,
            code: 5 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_REVERSE as u_int,
            code: 7 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_HIDDEN as u_int,
            code: 8 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_STRIKETHROUGH as u_int,
            code: 9 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_UNDERSCORE_2 as u_int,
            code: 42 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_UNDERSCORE_3 as u_int,
            code: 43 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_UNDERSCORE_4 as u_int,
            code: 44 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_UNDERSCORE_5 as u_int,
            code: 45 as u_int,
        },
        C2RustUnnamed_1 {
            mask: GRID_ATTR_OVERLINE as u_int,
            code: 53 as u_int,
        },
    ];
    n = 0 as size_t;
    i = 0 as size_t;
    while i
        < (::core::mem::size_of::<[C2RustUnnamed_1; 13]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_1>() as usize)
    {
        if !attr & attrs[i as usize].mask != 0 && lastattr & attrs[i as usize].mask != 0
            || (*lastgc).us != 8 as ::core::ffi::c_int && (*gc).us == 8 as ::core::ffi::c_int
        {
            let fresh4 = n;
            n = n.wrapping_add(1);
            s[fresh4 as usize] = 0 as ::core::ffi::c_int;
            lastattr &= GRID_ATTR_CHARSET as u_int;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    i = 0 as size_t;
    while i
        < (::core::mem::size_of::<[C2RustUnnamed_1; 13]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_1>() as usize)
    {
        if attr & attrs[i as usize].mask != 0 && lastattr & attrs[i as usize].mask == 0 {
            let fresh5 = n;
            n = n.wrapping_add(1);
            s[fresh5 as usize] = attrs[i as usize].code as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    *buf = '\0' as i32 as ::core::ffi::c_char;
    if n > 0 as size_t {
        if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 {
            strlcat(
                buf,
                b"\\033[\0" as *const u8 as *const ::core::ffi::c_char,
                len,
            );
        } else {
            strlcat(
                buf,
                b"\x1B[\0" as *const u8 as *const ::core::ffi::c_char,
                len,
            );
        }
        i = 0 as size_t;
        while i < n {
            if s[i as usize] < 10 as ::core::ffi::c_int {
                xformat(&mut tmp, format_args!("{}", (s[i as usize]) as i32));
            } else {
                xformat(
                    &mut tmp,
                    format_args!(
                        "{}:{}",
                        (s[i as usize] / 10 as ::core::ffi::c_int) as i32,
                        (s[i as usize] % 10 as ::core::ffi::c_int) as i32
                    ),
                );
            }
            strlcat(buf, &raw mut tmp as *mut ::core::ffi::c_char, len);
            if i.wrapping_add(1 as size_t) < n {
                strlcat(buf, b";\0" as *const u8 as *const ::core::ffi::c_char, len);
            }
            i = i.wrapping_add(1);
        }
        strlcat(buf, b"m\0" as *const u8 as *const ::core::ffi::c_char, len);
    }
    nnewc = grid_string_cells_fg(gc, &raw mut newc as *mut ::core::ffi::c_int);
    noldc = grid_string_cells_fg(lastgc, &raw mut oldc as *mut ::core::ffi::c_int);
    grid_string_cells_add_code(
        buf,
        len,
        n as u_int,
        &raw mut s as *mut ::core::ffi::c_int,
        &raw mut newc as *mut ::core::ffi::c_int,
        &raw mut oldc as *mut ::core::ffi::c_int,
        nnewc,
        noldc,
        flags,
    );
    nnewc = grid_string_cells_bg(gc, &raw mut newc as *mut ::core::ffi::c_int);
    noldc = grid_string_cells_bg(lastgc, &raw mut oldc as *mut ::core::ffi::c_int);
    grid_string_cells_add_code(
        buf,
        len,
        n as u_int,
        &raw mut s as *mut ::core::ffi::c_int,
        &raw mut newc as *mut ::core::ffi::c_int,
        &raw mut oldc as *mut ::core::ffi::c_int,
        nnewc,
        noldc,
        flags,
    );
    nnewc = grid_string_cells_us(gc, &raw mut newc as *mut ::core::ffi::c_int);
    noldc = grid_string_cells_us(lastgc, &raw mut oldc as *mut ::core::ffi::c_int);
    grid_string_cells_add_code(
        buf,
        len,
        n as u_int,
        &raw mut s as *mut ::core::ffi::c_int,
        &raw mut newc as *mut ::core::ffi::c_int,
        &raw mut oldc as *mut ::core::ffi::c_int,
        nnewc,
        noldc,
        flags,
    );
    if attr & GRID_ATTR_CHARSET as u_int != 0 && lastattr & GRID_ATTR_CHARSET as u_int == 0 {
        if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 {
            strlcat(
                buf,
                b"\\016\0" as *const u8 as *const ::core::ffi::c_char,
                len,
            );
        } else {
            strlcat(
                buf,
                b"\x0E\0" as *const u8 as *const ::core::ffi::c_char,
                len,
            );
        }
    }
    if attr & GRID_ATTR_CHARSET as u_int == 0 && lastattr & GRID_ATTR_CHARSET as u_int != 0 {
        if flags & GRID_STRING_ESCAPE_SEQUENCES != 0 {
            strlcat(
                buf,
                b"\\017\0" as *const u8 as *const ::core::ffi::c_char,
                len,
            );
        } else {
            strlcat(
                buf,
                b"\x0F\0" as *const u8 as *const ::core::ffi::c_char,
                len,
            );
        }
    }
    if !sc.is_null() && !(*sc).hyperlinks.is_null() && (*lastgc).link != (*gc).link {
        if hyperlinks_get(
            (*sc).hyperlinks,
            (*gc).link,
            &raw mut uri,
            &raw mut id,
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
        ) != 0
        {
            *has_link = grid_string_cells_add_hyperlink(buf, len, id, uri, flags);
        } else if *has_link != 0 {
            grid_string_cells_add_hyperlink(
                buf,
                len,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
                flags,
            );
            *has_link = 0 as ::core::ffi::c_int;
        }
    }
}
/// The caller initializes `lastgc` to `grid_default_cell` once per capture
/// and retains it across lines so unchanged attributes are not emitted again.
pub unsafe fn grid_string_cells_bytes(
    gd: *mut grid,
    px: u_int,
    py: u_int,
    nx: u_int,
    mut lastgc: Option<&mut grid_cell>,
    flags: ::core::ffi::c_int,
    s: *mut screen,
) -> Vec<u8> {
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut data: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut buf = Vec::with_capacity(128);
    let mut code: [::core::ffi::c_char; 8192] = [0; 8192];
    let mut size: size_t = 0;
    let mut codelen: size_t = 0;
    let mut xx: u_int = 0;
    let mut end: u_int = 0;
    let mut has_link: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let Some(gl) = grid_peek_line(&*gd, py) else {
        return buf;
    };
    if flags & GRID_STRING_EMPTY_CELLS != 0 {
        end = gl.cellsize as u_int;
    } else {
        end = gl.cellused as u_int;
    }
    xx = px;
    while xx < px.wrapping_add(nx) {
        if xx >= end {
            break;
        }
        grid_get_cell(&*gd, xx, py, &mut gc);
        if !(gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0) {
            if let Some(lastgc) = lastgc
                .as_deref_mut()
                .filter(|_| flags & GRID_STRING_WITH_SEQUENCES != 0)
            {
                grid_string_cells_code(
                    lastgc,
                    &raw mut gc,
                    &raw mut code as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as size_t,
                    flags,
                    s,
                    &raw mut has_link,
                );
                codelen = strlen(&raw mut code as *mut ::core::ffi::c_char);
                *lastgc = gc;
            } else {
                codelen = 0 as size_t;
            }
            if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                data = b"\t\0" as *const u8 as *const ::core::ffi::c_char;
                size = 1 as size_t;
            } else {
                data = &raw mut gc.data.data as *mut u_char as *const ::core::ffi::c_char;
                size = gc.data.size as size_t;
                if flags & GRID_STRING_ESCAPE_SEQUENCES != 0
                    && size == 1 as size_t
                    && *data as ::core::ffi::c_int == '\\' as i32
                {
                    data = b"\\\\\0" as *const u8 as *const ::core::ffi::c_char;
                    size = 2 as size_t;
                }
            }
            if codelen != 0 as size_t {
                buf.extend_from_slice(std::slice::from_raw_parts(code.as_ptr().cast(), codelen));
            }
            buf.extend_from_slice(std::slice::from_raw_parts(data.cast(), size));
        }
        xx = xx.wrapping_add(1);
    }
    if has_link != 0 {
        grid_string_cells_add_hyperlink(
            &raw mut code as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as size_t,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
            flags,
        );
        codelen = strlen(&raw mut code as *mut ::core::ffi::c_char);
        buf.extend_from_slice(std::slice::from_raw_parts(code.as_ptr().cast(), codelen));
    }
    if flags & GRID_STRING_TRIM_SPACES != 0 {
        while buf.last() == Some(&b' ') {
            buf.pop();
        }
    }
    buf
}
pub fn grid_duplicate_lines(dst: &mut grid, dy: u_int, src: &grid, sy: u_int, mut ny: u_int) {
    if dy.wrapping_add(ny) > dst.hsize.wrapping_add(dst.sy) {
        ny = dst.hsize.wrapping_add(dst.sy).wrapping_sub(dy);
    }
    if sy.wrapping_add(ny) > src.hsize.wrapping_add(src.sy) {
        ny = src.hsize.wrapping_add(src.sy).wrapping_sub(sy);
    }
    grid_free_lines(dst, dy, ny);
    for offset in 0..ny {
        dst.linedata[dy.wrapping_add(offset) as usize] =
            src.linedata[sy.wrapping_add(offset) as usize].clone();
    }
}
unsafe fn grid_reflow_dead(mut gl: *mut grid_line) {
    *gl = grid_line::default();
    (*gl).flags = GRID_LINE_DEAD as u_short;
}
unsafe fn grid_reflow_add(mut gd: *mut grid, mut n: u_int) -> *mut grid_line {
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut sy: u_int = (*gd).sy.wrapping_add(n);
    (*gd).linedata.resize_with(sy as usize, grid_line::default);
    gl = (*gd).linedata.as_mut_ptr().offset((*gd).sy as isize) as *mut grid_line;
    (*gd).sy = sy;
    return gl;
}
unsafe fn grid_reflow_move(mut gd: *mut grid, mut from: *mut grid_line) -> *mut grid_line {
    let mut to: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    to = grid_reflow_add(gd, 1 as u_int);
    *to = std::mem::take(&mut *from);
    grid_reflow_dead(from);
    return to;
}
unsafe fn grid_reflow_join(
    mut target: *mut grid,
    mut gd: *mut grid,
    mut sx: u_int,
    mut yy: u_int,
    mut width: u_int,
    mut already: ::core::ffi::c_int,
) {
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut from: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut lines: u_int = 0;
    let mut left: u_int = 0;
    let mut i: u_int = 0;
    let mut to: u_int = 0;
    let mut line: u_int = 0;
    let mut want: u_int = 0 as u_int;
    let mut at: u_int = 0;
    let mut wrapped: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if already == 0 {
        to = (*target).sy;
        gl = grid_reflow_move(
            target,
            (*gd).linedata.as_mut_ptr().offset(yy as isize) as *mut grid_line,
        );
    } else {
        to = (*target).sy.wrapping_sub(1 as u_int);
        gl = (*target).linedata.as_mut_ptr().offset(to as isize) as *mut grid_line;
    }
    at = (*gl).cellused as u_int;
    lines = 0 as u_int;
    while !(yy.wrapping_add(1 as u_int).wrapping_add(lines) == (*gd).hsize.wrapping_add((*gd).sy)) {
        line = yy.wrapping_add(1 as u_int).wrapping_add(lines);
        if !((*(*gd).linedata.as_mut_ptr().offset(line as isize)).flags as ::core::ffi::c_int)
            & GRID_LINE_WRAPPED
            != 0
        {
            wrapped = 0 as ::core::ffi::c_int;
        }
        if (*(*gd).linedata.as_mut_ptr().offset(line as isize)).cellused as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            if wrapped == 0 {
                break;
            }
            lines = lines.wrapping_add(1);
        } else {
            grid_get_cell1(
                &(&(*gd).linedata)[line as usize],
                0 as u_int,
                &mut gc,
            );
            if width.wrapping_add(gc.data.width as u_int) > sx {
                break;
            }
            width = width.wrapping_add(gc.data.width as u_int);
            grid_set_cell(&mut *target, at, to, &gc);
            at = at.wrapping_add(1);
            from = (*gd).linedata.as_mut_ptr().offset(line as isize) as *mut grid_line;
            want = 1 as u_int;
            while want < (*from).cellused as u_int {
                grid_get_cell1(&*from, want, &mut gc);
                if width.wrapping_add(gc.data.width as u_int) > sx {
                    break;
                }
                width = width.wrapping_add(gc.data.width as u_int);
                grid_set_cell(&mut *target, at, to, &gc);
                at = at.wrapping_add(1);
                want = want.wrapping_add(1);
            }
            lines = lines.wrapping_add(1);
            if wrapped == 0 || want != (*from).cellused as u_int || width == sx {
                break;
            }
        }
    }
    if lines == 0 as u_int || from.is_null() {
        return;
    }
    left = ((*from).cellused as u_int).wrapping_sub(want);
    if left != 0 as u_int {
        grid_move_cells(
            &mut *gd,
            0 as u_int,
            want,
            yy.wrapping_add(lines),
            left,
            8 as u_int,
        );
        let from = &mut (&mut (*gd).linedata)[yy.wrapping_add(lines) as usize];
        from.cellused = left as u_short;
        from.cellsize = from.cellused;
        from.celldata.resize_with(from.cellsize as usize, grid_cell_entry::default);
        lines = lines.wrapping_sub(1);
    } else if wrapped == 0 {
        (&mut (*target).linedata)[to as usize].flags &= !GRID_LINE_WRAPPED as u_short;
    }
    i = yy.wrapping_add(1 as u_int);
    while i < yy.wrapping_add(1 as u_int).wrapping_add(lines) {
        grid_reflow_dead((*gd).linedata.as_mut_ptr().offset(i as isize) as *mut grid_line);
        i = i.wrapping_add(1);
    }
    if (*gd).hscrolled > to.wrapping_add(lines) {
        (*gd).hscrolled = (*gd).hscrolled.wrapping_sub(lines);
    } else if (*gd).hscrolled > to {
        (*gd).hscrolled = to;
    }
}
unsafe fn grid_reflow_split(
    mut target: *mut grid,
    mut gd: *mut grid,
    mut sx: u_int,
    mut yy: u_int,
    mut at: u_int,
) {
    let mut gl: *mut grid_line = (*gd).linedata.as_mut_ptr().offset(yy as isize) as *mut grid_line;
    let mut first: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut line: u_int = 0;
    let mut lines: u_int = 0;
    let mut width: u_int = 0;
    let mut i: u_int = 0;
    let mut xx: u_int = 0;
    let mut used: u_int = (*gl).cellused as u_int;
    let mut flags: ::core::ffi::c_int = (*gl).flags as ::core::ffi::c_int;
    if !((*gl).flags as ::core::ffi::c_int) & GRID_LINE_EXTENDED != 0 {
        lines = (1 as u_int).wrapping_add(
            (((*gl).cellused as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as u_int)
                .wrapping_div(sx),
        );
    } else {
        lines = 2 as u_int;
        width = 0 as u_int;
        i = at;
        while i < used {
            grid_get_cell1(&*gl, i, &mut gc);
            if width.wrapping_add(gc.data.width as u_int) > sx {
                lines = lines.wrapping_add(1);
                width = 0 as u_int;
            }
            width = width.wrapping_add(gc.data.width as u_int);
            i = i.wrapping_add(1);
        }
    }
    line = (*target).sy.wrapping_add(1 as u_int);
    first = grid_reflow_add(target, lines);
    width = 0 as u_int;
    xx = 0 as u_int;
    i = at;
    while i < used {
        grid_get_cell1(&*gl, i, &mut gc);
        if width.wrapping_add(gc.data.width as u_int) > sx {
            let ref mut fresh44 = (*(*target).linedata.as_mut_ptr().offset(line as isize)).flags;
            *fresh44 = (*fresh44 as ::core::ffi::c_int | GRID_LINE_WRAPPED) as u_short;
            line = line.wrapping_add(1);
            width = 0 as u_int;
            xx = 0 as u_int;
        }
        width = width.wrapping_add(gc.data.width as u_int);
        grid_set_cell(&mut *target, xx, line, &gc);
        xx = xx.wrapping_add(1);
        i = i.wrapping_add(1);
    }
    if flags & GRID_LINE_WRAPPED != 0 {
        let ref mut fresh45 = (*(*target).linedata.as_mut_ptr().offset(line as isize)).flags;
        *fresh45 = (*fresh45 as ::core::ffi::c_int | GRID_LINE_WRAPPED) as u_short;
    }
    (*gl).cellused = at as u_short;
    (*gl).cellsize = (*gl).cellused;
    (*gl)
        .celldata
        .resize_with((*gl).cellsize as usize, grid_cell_entry::default);
    (*gl).flags = ((*gl).flags as ::core::ffi::c_int | GRID_LINE_WRAPPED) as u_short;
    *first = std::mem::take(&mut *gl);
    grid_reflow_dead(gl);
    if yy <= (*gd).hscrolled {
        (*gd).hscrolled = (*gd).hscrolled.wrapping_add(lines.wrapping_sub(1 as u_int));
    }
    if width < sx && flags & GRID_LINE_WRAPPED != 0 {
        grid_reflow_join(target, gd, sx, yy, width, 1 as ::core::ffi::c_int);
    }
}
pub unsafe fn grid_reflow(mut gd: *mut grid, mut sx: u_int) {
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut yy: u_int = 0;
    let mut width: u_int = 0;
    let mut i: u_int = 0;
    let mut at: u_int = 0;
    let mut target_owner = grid_create_box((*gd).sx, 0, 0);
    let target = &raw mut *target_owner;
    yy = 0 as u_int;
    while yy < (*gd).hsize.wrapping_add((*gd).sy) {
        gl = (*gd).linedata.as_mut_ptr().offset(yy as isize) as *mut grid_line;
        if !((*gl).flags as ::core::ffi::c_int & GRID_LINE_DEAD != 0) {
            width = 0 as u_int;
            at = width;
            if !((*gl).flags as ::core::ffi::c_int) & GRID_LINE_EXTENDED != 0 {
                width = (*gl).cellused as u_int;
                if width > sx {
                    at = sx;
                } else {
                    at = width;
                }
            } else {
                i = 0 as u_int;
                while i < (*gl).cellused as u_int {
                    grid_get_cell1(&*gl, i, &mut gc);
                    if at == 0 as u_int && width.wrapping_add(gc.data.width as u_int) > sx {
                        at = i;
                    }
                    width = width.wrapping_add(gc.data.width as u_int);
                    i = i.wrapping_add(1);
                }
            }
            if width == sx {
                grid_reflow_move(target, gl);
            } else if width > sx {
                grid_reflow_split(target, gd, sx, yy, at);
            } else if (*gl).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED != 0 {
                grid_reflow_join(target, gd, sx, yy, width, 0 as ::core::ffi::c_int);
            } else {
                grid_reflow_move(target, gl);
            }
        }
        yy = yy.wrapping_add(1);
    }
    if (*target).sy < (*gd).sy {
        grid_reflow_add(target, (*gd).sy.wrapping_sub((*target).sy));
    }
    (*gd).hsize = (*target).sy.wrapping_sub((*gd).sy);
    if (*gd).hscrolled > (*gd).hsize {
        (*gd).hscrolled = (*gd).hsize;
    }
    (*gd).linedata = std::mem::take(&mut (*target).linedata);
    (*gd).scroll_generation = (*gd).scroll_generation.wrapping_add(1);
}
pub unsafe fn grid_wrap_position(
    mut gd: *mut grid,
    mut px: u_int,
    mut py: u_int,
    mut wx: *mut u_int,
    mut wy: *mut u_int,
) {
    let mut ax: u_int = 0 as u_int;
    let mut ay: u_int = 0 as u_int;
    let mut yy: u_int = 0;
    yy = 0 as u_int;
    while yy < py {
        if (*(*gd).linedata.as_mut_ptr().offset(yy as isize)).flags as ::core::ffi::c_int
            & GRID_LINE_WRAPPED
            != 0
        {
            ax = ax
                .wrapping_add((*(*gd).linedata.as_mut_ptr().offset(yy as isize)).cellused as u_int);
        } else {
            ax = 0 as u_int;
            ay = ay.wrapping_add(1);
        }
        yy = yy.wrapping_add(1);
    }
    if px >= (*(*gd).linedata.as_mut_ptr().offset(yy as isize)).cellused as u_int {
        ax = UINT_MAX as u_int;
    } else {
        ax = ax.wrapping_add(px);
    }
    *wx = ax;
    *wy = ay;
}
pub unsafe fn grid_unwrap_position(
    mut gd: *mut grid,
    mut px: *mut u_int,
    mut py: *mut u_int,
    mut wx: u_int,
    mut wy: u_int,
) {
    let mut yy: u_int = 0;
    let mut ay: u_int = 0 as u_int;
    let mut ey: u_int = (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int);
    yy = 0 as u_int;
    while yy < (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int) {
        if ay == wy {
            break;
        }
        if !((*(*gd).linedata.as_mut_ptr().offset(yy as isize)).flags as ::core::ffi::c_int)
            & GRID_LINE_WRAPPED
            != 0
        {
            ay = ay.wrapping_add(1);
        }
        yy = yy.wrapping_add(1);
    }
    if wx == UINT_MAX {
        while yy < ey
            && (*(*gd).linedata.as_mut_ptr().offset(yy as isize)).flags as ::core::ffi::c_int
                & GRID_LINE_WRAPPED
                != 0
        {
            yy = yy.wrapping_add(1);
        }
        wx = (*(*gd).linedata.as_mut_ptr().offset(yy as isize)).cellused as u_int;
    } else {
        while (*(*gd).linedata.as_mut_ptr().offset(yy as isize)).flags as ::core::ffi::c_int
            & GRID_LINE_WRAPPED
            != 0
        {
            if wx < (*(*gd).linedata.as_mut_ptr().offset(yy as isize)).cellused as u_int {
                break;
            }
            wx = wx
                .wrapping_sub((*(*gd).linedata.as_mut_ptr().offset(yy as isize)).cellused as u_int);
            yy = yy.wrapping_add(1);
        }
    }
    *px = wx;
    *py = yy;
}
pub unsafe fn grid_line_length(gd: &grid, mut py: u_int) -> u_int {
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut px: u_int = 0;
    px = gd.linedata[py as usize].cellsize as u_int;
    if px > gd.sx {
        px = gd.sx;
    }
    while px > 0 as u_int {
        grid_get_cell(gd, px.wrapping_sub(1 as u_int), py, &mut gc);
        if gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0
            || gc.data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
            || gc.data.data[0] != b' '
        {
            break;
        }
        px = px.wrapping_sub(1);
    }
    return px;
}
pub unsafe fn grid_line_limit(gd: &grid, mut py: u_int) -> u_int {
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut px: u_int = 0;
    px = grid_line_length(gd, py);
    if px == 0 as u_int {
        return 0 as u_int;
    }
    px = px.wrapping_sub(1);
    while px > 0 as u_int {
        grid_get_cell(gd, px, py, &mut gc);
        if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
            break;
        }
        px = px.wrapping_sub(1);
    }
    return px;
}
pub unsafe fn grid_in_set(
    gd: &grid,
    mut px: u_int,
    mut py: u_int,
    set: &CStr,
) -> ::core::ffi::c_int {
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut tmp_gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut pxx: u_int = 0;
    let mut has_tab: ::core::ffi::c_int = 0;
    let mut has_space: ::core::ffi::c_int = 0;
    has_tab = set.to_bytes().contains(&b'\t') as ::core::ffi::c_int;
    has_space = set.to_bytes().contains(&b' ') as ::core::ffi::c_int;
    grid_get_cell(gd, px, py, &mut gc);
    if gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        if has_tab == 0 && has_space == 0 {
            return 0 as ::core::ffi::c_int;
        }
        pxx = px;
        loop {
            pxx = pxx.wrapping_sub(1);
            grid_get_cell(gd, pxx, py, &mut tmp_gc);
            if !(pxx > 0 as u_int && tmp_gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0) {
                break;
            }
        }
        if (has_tab != 0 || has_space != 0)
            && tmp_gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0
            || has_space != 0 && utf8_has_whitespace(&tmp_gc.data) != 0
        {
            return (tmp_gc.data.width as u_int).wrapping_sub(px.wrapping_sub(pxx))
                as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
    if (has_tab != 0 || has_space != 0) && gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
        return gc.data.width as ::core::ffi::c_int;
    }
    if has_space != 0 && utf8_has_whitespace(&gc.data) != 0 {
        return if gc.data.width as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            1 as ::core::ffi::c_int
        } else {
            gc.data.width as ::core::ffi::c_int
        };
    }
    return utf8_cstrhas(set, &gc.data) as ::core::ffi::c_int;
}
pub unsafe fn grid_line_flags_string(mut flags: ::core::ffi::c_int) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 128] = [0; 128];
    *(&raw mut s as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if flags & GRID_LINE_WRAPPED != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"WRAPPED,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_LINE_EXTENDED != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"EXTENDED,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_LINE_DEAD != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"DEAD,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_LINE_START_PROMPT != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"START_PROMPT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_LINE_SECOND_PROMPT != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"SECOND_PROMPT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_LINE_START_COMMAND != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"START_COMMAND,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_LINE_START_OUTPUT != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"START_OUTPUT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_LINE_END_OUTPUT != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"END_OUTPUT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_LINE_HYPERLINK != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"HYPERLINK,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if *(&raw mut s as *mut ::core::ffi::c_char) as ::core::ffi::c_int == '\0' as i32 {
        return b"NONE\0" as *const u8 as *const ::core::ffi::c_char;
    }
    s[strlen(&raw mut s as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
        '\0' as i32 as ::core::ffi::c_char;
    return &raw mut s as *mut ::core::ffi::c_char;
}
pub unsafe fn grid_cell_flags_string(mut flags: ::core::ffi::c_int) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 128] = [0; 128];
    *(&raw mut s as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if flags & GRID_FLAG_FG256 != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"FG256,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_FLAG_BG256 != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"BG256,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_FLAG_PADDING != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"PADDING,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_FLAG_EXTENDED != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"EXTENDED,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_FLAG_SELECTED != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"SELECTED,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_FLAG_CLEARED != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"CLEARED,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_FLAG_TAB != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"TAB,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & GRID_FLAG_NOPALETTE != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"NOPALETTE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if *(&raw mut s as *mut ::core::ffi::c_char) as ::core::ffi::c_int == '\0' as i32 {
        return b"NONE\0" as *const u8 as *const ::core::ffi::c_char;
    }
    s[strlen(&raw mut s as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
        '\0' as i32 as ::core::ffi::c_char;
    return &raw mut s as *mut ::core::ffi::c_char;
}
pub unsafe fn grid_cell_attr_string(mut attr: ::core::ffi::c_int) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 256] = [0; 256];
    *(&raw mut s as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if attr & GRID_ATTR_CHARSET != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"CHARSET,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_BRIGHT != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"BRIGHT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_DIM != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"DIM,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_UNDERSCORE != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"UNDERSCORE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_BLINK != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"BLINK,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_REVERSE != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"REVERSE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_HIDDEN != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"HIDDEN,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_ITALICS != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"ITALICS,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_STRIKETHROUGH != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"STRIKETHROUGH,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_UNDERSCORE_2 != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"UNDERSCORE_2,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_UNDERSCORE_3 != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"UNDERSCORE_3,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_UNDERSCORE_4 != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"UNDERSCORE_4,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_UNDERSCORE_5 != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"UNDERSCORE_5,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if attr & GRID_ATTR_OVERLINE != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"OVERLINE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if *(&raw mut s as *mut ::core::ffi::c_char) as ::core::ffi::c_int == '\0' as i32 {
        return b"NONE\0" as *const u8 as *const ::core::ffi::c_char;
    }
    s[strlen(&raw mut s as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
        '\0' as i32 as ::core::ffi::c_char;
    return &raw mut s as *mut ::core::ffi::c_char;
}

#[cfg(test)]
mod storage_tests {
    use super::*;

    #[test]
    fn borrowed_bulk_writes_keep_embedded_nuls_bytes_and_empty_extent() {
        unsafe {
            let mut owner = grid_create_box(16, 3, 0);
            let bytes = [0, b'A' as std::ffi::c_char, 0xff_u8 as std::ffi::c_char];
            let mut cell = grid_default_cell;
            for row in 0..2 {
                if row == 1 {
                    cell.fg = COLOUR_FLAG_RGB | 0x123456;
                    cell.link = 7;
                }
                grid_set_cells(&mut owner, 1, row, &cell, &bytes);
                assert_eq!(owner.linedata[row as usize].cellsize, 8);
                assert_eq!(owner.linedata[row as usize].cellused, 4);
                for (index, byte) in bytes.iter().enumerate() {
                    let mut actual = grid_default_cell;
                    grid_get_cell(&owner, 1 + index as u_int, row, &mut actual);
                    assert_eq!(
                        (actual.data.size, actual.data.width, actual.data.data[0]),
                        (1, 1, *byte as u8)
                    );
                    assert_eq!((actual.fg, actual.link), (cell.fg, cell.link));
                }
            }
            grid_set_cells(&mut owner, 3, 2, &cell, &[]);
            assert_eq!(
                (owner.linedata[2].cellsize, owner.linedata[2].cellused),
                (4, 3)
            );
            assert_eq!(owner.linedata[2].extdsize, 0);
            grid_set_cells(&mut owner, 0, 3, &cell, &bytes);
            assert_eq!(owner.linedata.len(), 3);
        }
    }

    #[test]
    fn overlapping_cell_moves_keep_extended_colours_and_clear_source_only() {
        unsafe {
            let bg = (COLOUR_FLAG_RGB | 0xabcdef) as u_int;
            for (destination, source, expected) in [(0, 1, b"BCDE "), (1, 0, b" ABCD")] {
                let mut owner = grid_create_box(8, 1, 0);
                for (column, byte) in b"ABCDE".iter().enumerate() {
                    let mut cell = grid_default_cell;
                    cell.fg = COLOUR_FLAG_RGB | *byte as i32;
                    utf8_set(&mut cell.data, *byte);
                    grid_set_cell(&mut owner, column as u_int, 0, &cell);
                }
                grid_move_cells(&mut owner, destination, source, 0, 4, bg);
                for (column, byte) in expected.iter().enumerate() {
                    let mut actual = grid_default_cell;
                    grid_get_cell(&owner, column as u_int, 0, &mut actual);
                    assert_eq!(actual.data.data[0], *byte);
                    if *byte == b' ' {
                        assert_eq!(actual.bg, bg as i32);
                    } else {
                        assert_eq!(actual.fg, COLOUR_FLAG_RGB | *byte as i32);
                    }
                }
            }
            let mut owner = grid_create_box(16, 1, 0);
            grid_clear(&mut owner, 5, 0, 3, 1, 8);
            assert_eq!(owner.linedata[0].cellsize, 0);
            grid_clear(&mut owner, 5, 0, 3, 1, bg);
            assert_eq!(
                (owner.linedata[0].cellsize, owner.linedata[0].cellused),
                (16, 0)
            );
            let mut actual = grid_default_cell;
            grid_get_cell(&owner, 7, 0, &mut actual);
            assert_eq!(actual.bg, bg as i32);
            grid_get_cell(&owner, 8, 0, &mut actual);
            assert_eq!(actual.bg, 8);
        }
    }

    #[test]
    fn clearing_extended_cells_preserves_moved_entries_and_reuses_live_slots() {
        unsafe {
            let mut line = grid_line::default();
            line.cellsize = 2;
            line.celldata.resize_with(2, grid_cell_entry::default);
            let mut cell = grid_default_cell;
            cell.fg = COLOUR_FLAG_RGB | 0xabcdef;
            cell.link = 7;
            utf8_set(&mut cell.data, b'X');
            grid_extended_cell(&mut line, 0, &cell);
            line.celldata[1] = line.celldata[0];
            grid_clear_cell(&mut line, 0, 8, true);
            let mut actual = grid_default_cell;
            grid_get_cell1(&line, 1, &mut actual);
            assert_same_cell(&actual, &cell);
            assert_eq!(line.extdsize, 1);
            assert!(!grid_need_extended_cell(
                &line.celldata[0],
                &grid_default_cell
            ));

            grid_clear_cell(&mut line, 1, 9, false);
            grid_get_cell1(&line, 1, &mut actual);
            assert_eq!(
                (actual.data.data[0], actual.fg, actual.bg, actual.link),
                (b' ', 8, 9, 0)
            );
            assert_eq!(line.extdsize, 1);
            assert_ne!(line.flags as i32 & GRID_LINE_HYPERLINK, 0);

            let bg = (COLOUR_FLAG_RGB | 0x123456) as u_int;
            grid_clear_cell(&mut line, 0, bg, false);
            assert_eq!(line.extdsize, 2);
            grid_clear_cell(&mut line, 0, bg, false);
            assert_eq!(line.extdsize, 2);
            line.celldata[0].c2rust_unnamed.offset = u_int::MAX;
            grid_clear_cell(&mut line, 0, bg, false);
            assert_eq!(line.extdsize, 3);
            grid_get_cell1(&line, 0, &mut actual);
            assert_eq!((actual.data.data[0], actual.bg), (b' ', bg as i32));
        }
    }

    #[test]
    fn line_compaction_remaps_live_offsets_and_releases_unused_entries() {
        unsafe {
            let mut line = grid_line::default();
            line.cellsize = 5;
            line.extdsize = 6;
            line.flags = GRID_LINE_EXTENDED as u_short;
            line.celldata.resize_with(6, grid_cell_entry::default);
            line.extddata.resize_with(6, grid_extd_entry::default);
            for (index, extended) in line.extddata.iter_mut().enumerate() {
                extended.fg = 10 + index as i32;
            }
            for (column, offset) in [(0, 4), (2, 1), (3, 4), (5, 0)] {
                line.celldata[column].flags = GRID_FLAG_EXTENDED as u_char;
                line.celldata[column].c2rust_unnamed.offset = offset;
            }
            grid_compact_line(&mut line);
            assert_eq!(line.extdsize, 3);
            assert_eq!(
                line.extddata
                    .iter()
                    .map(|entry| entry.fg)
                    .collect::<Vec<_>>(),
                [14, 11, 14]
            );
            for (column, offset) in [(0, 0), (2, 1), (3, 2)] {
                let actual = line.celldata[column].c2rust_unnamed.offset;
                assert_eq!(actual, offset);
                line.celldata[column].flags = 0;
            }
            // The spare cell beyond cellsize does not keep the allocation alive.
            grid_compact_line(&mut line);
            assert_eq!(line.extdsize, 0);
            assert!(line.extddata.is_empty());
            assert_eq!(line.flags, GRID_LINE_EXTENDED as u_short);
        }
    }

    #[test]
    fn borrowed_history_operations_keep_thresholds_counts_and_spare_rows() {
        unsafe {
            let mut owner = labeled_grid();
            for _ in 0..4 {
                grid_scroll_history(&mut owner, 8);
            }
            assert_eq!(labels(&mut owner), b"ABCD    ");
            assert_eq!(
                (owner.hsize, owner.hscrolled, owner.scroll_added),
                (4, 4, 4)
            );
            owner.hlimit = 4;
            grid_collect_history(&mut owner, 1);
            assert_eq!(labels(&mut owner), b"BCD    ");
            assert_eq!(
                (owner.hsize, owner.hscrolled, owner.scroll_collected),
                (3, 3, 1)
            );
            owner.hlimit = 1;
            grid_collect_history(&mut owner, 1);
            assert_eq!(labels(&mut owner), b"D    ");
            assert_eq!(
                (owner.hsize, owner.hscrolled, owner.scroll_collected),
                (1, 1, 3)
            );
            grid_remove_history(&mut owner, 2);
            assert_eq!(owner.hsize, 1);
            grid_remove_history(&mut owner, 1);
            assert_eq!(labels(&mut owner), b"D   ");
            assert_eq!(owner.hscrolled, 1);
            assert_eq!(owner.linedata.len(), 8);
            grid_clear_history(&mut owner);
            assert_eq!(owner.linedata.len(), 4);
            assert_eq!(
                (owner.hsize, owner.hscrolled, owner.scroll_generation),
                (0, 0, 1)
            );
            assert_eq!((owner.scroll_added, owner.scroll_collected), (4, 3));
        }
    }

    #[test]
    fn borrowed_line_lookup_covers_history_and_rejects_spare_storage() {
        unsafe {
            let mut owner = grid_create_box(8, 2, 10);
            grid_scroll_history(&mut *owner, 8);
            grid_adjust_lines(&mut *owner, 4);
            owner.linedata[0].time = 1;
            owner.linedata[1].time = 9;
            owner.linedata[3].time = 77;
            let history = grid_peek_line(&owner, 0).unwrap();
            assert_eq!(history.time, 1);
            assert_eq!(grid_line_time(history), start_time.tv_sec);
            let visible = grid_peek_line(&owner, 1).unwrap();
            assert_eq!(grid_line_time(visible), start_time.tv_sec + 8);
            assert_eq!(grid_line_time(grid_peek_line(&owner, 2).unwrap()), 0);
            assert!(grid_peek_line(&owner, 3).is_none());
            assert!(grid_peek_line(&owner, u_int::MAX).is_none());
        }
    }

    #[test]
    fn cell_comparison_preserves_ignored_fields_and_exact_glyph_bytes() {
        let original = grid_default_cell;
        let mut changed = original;
        changed.flags |= GRID_FLAG_CLEARED as u_char;
        changed.us = 99;
        changed.data.have = 31;
        changed.data.data[31] = 0xff;
        assert!(grid_cells_equal(&original, &changed));
        changed.data.width = 2;
        assert!(grid_cells_look_equal(&original, &changed));
        assert!(!grid_cells_equal(&original, &changed));
        changed.data.width = original.data.width;
        changed.data.data[0] = b'x';
        assert!(grid_cells_look_equal(&original, &changed));
        assert!(!grid_cells_equal(&original, &changed));
        for mutate in [
            |cell: &mut grid_cell| cell.fg += 1,
            |cell: &mut grid_cell| cell.bg += 1,
            |cell: &mut grid_cell| cell.attr ^= 1,
            |cell: &mut grid_cell| cell.flags ^= GRID_FLAG_PADDING as u_char,
            |cell: &mut grid_cell| cell.link += 1,
        ] {
            let mut changed = original;
            mutate(&mut changed);
            assert!(!grid_cells_look_equal(&original, &changed));
            assert!(!grid_cells_equal(&original, &changed));
        }
    }

    #[test]
    fn grid_comparison_keeps_allocated_line_sizes_and_physical_row_origin() {
        unsafe {
            let mut a = grid_create_box(2, 1, 10);
            let mut b = grid_create_box(2, 1, 10);
            assert_eq!(grid_compare(&a, &b), 0);
            grid_set_cell(&mut *a, 0, 0, &grid_default_cell);
            assert_eq!(grid_compare(&a, &b), 1);
            grid_set_cell(&mut *b, 0, 0, &grid_default_cell);
            assert_eq!(grid_compare(&a, &b), 0);

            grid_scroll_history(&mut *a, 8);
            grid_scroll_history(&mut *b, 8);
            let mut cell = grid_default_cell;
            cell.data.data[0] = b'X';
            grid_set_cell(&mut *a, 0, 1, &cell);
            assert_eq!(grid_compare(&a, &b), 0);
            grid_set_cell(&mut *a, 0, 0, &cell);
            assert_eq!(grid_compare(&a, &b), 1);
            grid_set_cell(&mut *b, 0, 0, &cell);
            a.linedata[0].time = 17;
            a.linedata[0].flags ^= GRID_LINE_WRAPPED as u_short;
            assert_eq!(grid_compare(&a, &b), 0);
            b.sx = 3;
            assert_eq!(grid_compare(&a, &b), 1);
        }
    }

    fn assert_same_cell(actual: &grid_cell, expected: &grid_cell) {
        assert_eq!(actual.data.data, expected.data.data);
        assert_eq!(
            (actual.data.have, actual.data.size, actual.data.width),
            (expected.data.have, expected.data.size, expected.data.width),
        );
        assert_eq!(
            (
                actual.flags,
                actual.attr,
                actual.fg,
                actual.bg,
                actual.us,
                actual.link
            ),
            (
                expected.flags,
                expected.attr,
                expected.fg,
                expected.bg,
                expected.us,
                expected.link
            ),
        );
    }

    #[test]
    fn borrowed_cell_reads_cover_defaults_compact_extended_and_tab_storage() {
        unsafe {
            let mut owner = grid_create_box(16, 1, 0);
            let default = grid_default_cell;
            let mut actual = grid_cell::default();
            for (x, y) in [(0, 0), (u_int::MAX, 0), (0, u_int::MAX)] {
                grid_get_cell(&owner, x, y, &mut actual);
                assert_same_cell(&actual, &default);
            }

            let mut compact = default;
            utf8_set(&mut compact.data, b'A');
            compact.fg = COLOUR_FLAG_256 | 99;
            compact.bg = COLOUR_FLAG_256 | 17;
            compact.attr = 3;
            grid_set_cell(&mut *owner, 0, 0, &compact);
            grid_get_cell(&owner, 0, 0, &mut actual);
            assert_same_cell(&actual, &compact);

            let mut extended = default;
            extended.data = crate::src::text::utf8::utf8_fromcstr_vec(c"🦀")[0];
            extended.fg = COLOUR_FLAG_RGB | 0xabcdef;
            extended.us = COLOUR_FLAG_RGB | 0x123456;
            extended.link = 7;
            grid_set_cell(&mut *owner, 1, 0, &extended);
            grid_get_cell(&owner, 1, 0, &mut actual);
            assert_same_cell(&actual, &extended);

            let mut tab = default;
            grid_set_tab(&mut tab, 3);
            grid_set_cell(&mut *owner, 3, 0, &tab);
            grid_get_cell(&owner, 3, 0, &mut actual);
            assert_same_cell(&actual, &tab);

            owner.linedata[0].celldata[1].c2rust_unnamed.offset = u_int::MAX;
            grid_get_cell(&owner, 1, 0, &mut actual);
            assert_same_cell(&actual, &default);
        }
    }

    #[test]
    fn borrowed_line_queries_keep_wide_space_and_tab_padding_widths() {
        unsafe {
            let mut owner = grid_create_box(8, 1, 0);
            let mut cell = grid_default_cell;
            grid_set_tab(&mut cell, 3);
            grid_set_cell(&mut *owner, 2, 0, &cell);
            grid_set_padding(&mut *owner, 3, 0, 8);
            grid_set_padding(&mut *owner, 4, 0, 8);
            assert_eq!(grid_line_length(&owner, 0), 5);
            assert_eq!(grid_line_limit(&owner, 0), 2);
            for (x, expected) in [(2, 3), (3, 2), (4, 1)] {
                assert_eq!(grid_in_set(&owner, x, 0, c"\t"), expected);
                assert_eq!(grid_in_set(&owner, x, 0, c" "), expected);
                assert_eq!(grid_in_set(&owner, x, 0, c""), 0);
            }

            cell = grid_default_cell;
            cell.data = crate::src::text::utf8::utf8_fromcstr_vec(c"　")[0];
            grid_set_cell(&mut *owner, 5, 0, &cell);
            grid_set_padding(&mut *owner, 6, 0, 8);
            assert_eq!(grid_line_length(&owner, 0), 7);
            assert_eq!(grid_line_limit(&owner, 0), 5);
            assert_eq!(grid_in_set(&owner, 5, 0, c" "), 2);
            assert_eq!(grid_in_set(&owner, 6, 0, c" "), 1);
            assert_eq!(grid_in_set(&owner, 6, 0, c"\t"), 0);
            assert_eq!(grid_in_set(&owner, 5, 0, c"　"), 1);
        }
    }

    unsafe fn labeled_grid() -> Box<grid> {
        let mut owner = grid_create_box(8, 4, 1);
        for (y, byte) in b"ABCD".iter().enumerate() {
            let mut cell = grid_default_cell;
            cell.data.data[0] = *byte;
            cell.fg = COLOUR_FLAG_RGB | (*byte as i32);
            grid_set_cell(&mut *owner, 0, y as u_int, &cell);
        }
        owner
    }
    unsafe fn labels(owner: &mut grid) -> Vec<u8> {
        (0..owner.hsize + owner.sy)
            .map(|y| {
                let mut cell = grid_default_cell;
                grid_get_cell(&*owner, 0, y, &mut cell);
                cell.data.data[0]
            })
            .collect()
    }

    #[test]
    fn borrowed_line_moves_preserve_overlap_flags_and_fill_backgrounds() {
        unsafe {
            for bg in [8, (COLOUR_FLAG_RGB | 0x123456) as u_int] {
                for (destination, source, count, expected, wrapped) in [
                    (0, 1, 3, b"BCD ", [true, true, true, false]),
                    (1, 0, 3, b" ABC", [false, false, true, true]),
                    (2, 0, 1, b" BAD", [false, false, true, true]),
                    (0, 2, 1, b"CB D", [true, false, false, true]),
                    (4, 0, 1, b"ABCD", [true; 4]),
                    (0, 4, 1, b"ABCD", [true; 4]),
                    (0, 3, 2, b"ABCD", [true; 4]),
                    (0, 3, 0, b"ABCD", [true; 4]),
                ] {
                    let mut owner = labeled_grid();
                    for line in owner.linedata.iter_mut() {
                        line.flags |= GRID_LINE_WRAPPED as u_short;
                    }
                    grid_move_lines(&mut owner, destination, source, count, bg);
                    assert_eq!(labels(&mut owner), expected);
                    for (row, &is_wrapped) in wrapped.iter().enumerate() {
                        assert_eq!(
                            owner.linedata[row].flags as i32 & GRID_LINE_WRAPPED != 0,
                            is_wrapped,
                        );
                        if expected[row] == b' ' {
                            let mut cell = grid_default_cell;
                            grid_get_cell(&owner, 0, row as u_int, &mut cell);
                            assert_eq!(cell.bg, bg as i32);
                        }
                    }
                }
            }
            let mut owner = labeled_grid();
            for line in owner.linedata.iter_mut() {
                line.flags |= GRID_LINE_WRAPPED as u_short;
            }
            grid_clear_lines(&mut owner, 1, 2, 8);
            assert_eq!(labels(&mut owner), b"A  D");
            assert!(owner.linedata[..3]
                .iter()
                .all(|line| line.flags as i32 & GRID_LINE_WRAPPED == 0));
            assert_ne!(owner.linedata[3].flags as i32 & GRID_LINE_WRAPPED, 0);
        }
    }

    #[test]
    fn line_duplication_clips_both_grids_and_preserves_independent_metadata() {
        unsafe {
            let mut src = labeled_grid();
            let mut dst = labeled_grid();
            src.linedata[1].time = 77;
            src.linedata[1].flags |= GRID_LINE_WRAPPED as u_short | GRID_LINE_START_PROMPT as u_short;
            src.linedata[1].osc133_data.prompt_col = 3;
            grid_duplicate_lines(&mut dst, 3, &src, 1, 3);
            assert_eq!(labels(&mut dst), b"ABCB");
            assert_eq!(dst.linedata[3].time, 77);
            assert_eq!(dst.linedata[3].flags, src.linedata[1].flags);
            assert_eq!(dst.linedata[3].osc133_data.prompt_col, 3);
            src.linedata[1].extddata[0].fg = 8;
            let mut cell = grid_default_cell;
            grid_get_cell(&dst, 0, 3, &mut cell);
            assert_eq!(cell.fg, COLOUR_FLAG_RGB | b'B' as i32);
            grid_duplicate_lines(&mut dst, 0, &src, 3, 3);
            assert_eq!(labels(&mut dst), b"DBCB");
            grid_duplicate_lines(&mut dst, 4, &src, 4, 0);
            assert_eq!(labels(&mut dst), b"DBCB");
            drop(src);
            grid_get_cell(&dst, 0, 3, &mut cell);
            assert_eq!(cell.data.data[0], b'B');
        }
    }

    #[test]
    fn line_moves_history_rotation_and_deep_clones_keep_independent_cells() {
        unsafe {
            let mut owner = labeled_grid();
            let mut copy = grid_create_box(8, 4, 0);
            grid_duplicate_lines(&mut copy, 0, &owner, 0, 4);
            assert_ne!(
                owner.linedata[0].celldata.as_ptr(),
                copy.linedata[0].celldata.as_ptr()
            );
            assert_ne!(
                owner.linedata[0].extddata.as_ptr(),
                copy.linedata[0].extddata.as_ptr()
            );
            grid_move_lines(&mut *owner, 0, 1, 3, 8);
            assert_eq!(labels(&mut owner), b"BCD ");
            grid_move_lines(&mut *owner, 1, 0, 3, 8);
            assert_eq!(labels(&mut owner), b" BCD");
            assert_eq!(labels(&mut copy), b"ABCD");
            drop(owner);
            assert_eq!(labels(&mut copy), b"ABCD");

            let mut owner = labeled_grid();
            grid_scroll_history_region(&mut *owner, 1, 2, 8);
            assert_eq!(labels(&mut owner), b"BAC D");
            grid_collect_history(&mut *owner, 0);
            assert_eq!(labels(&mut owner), b"AC D");
            // Repeated extended-cell updates and history compaction must retain
            // the active entry while releasing unreachable extended entries.
            let mut cell = grid_default_cell;
            cell.fg = COLOUR_FLAG_RGB | 0x123456;
            cell.data.data[0] = b'X';
            for _ in 0..8 {
                grid_set_cell(&mut *owner, 0, 0, &cell);
            }
            grid_scroll_history(&mut *owner, 8);
            assert_eq!(labels(&mut owner), b"XC D ");
            assert_eq!(
                owner.linedata[0].extddata.len(),
                owner.linedata[0].extdsize as usize
            );
            grid_clear_history(&mut *owner);
            assert_eq!(labels(&mut owner), b"C D ");
            assert_eq!(owner.linedata.len(), 4);
        }
    }
}

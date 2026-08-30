use super::store::{
    grid_clear, grid_collect_history, grid_get_cell, grid_get_line, grid_move_cells,
    grid_move_lines, grid_scroll_history, grid_scroll_history_region, grid_set_cell,
    grid_set_cells, grid_set_padding, grid_string_cells,
};
pub use crate::types::*;
use ::core::ptr::null_mut;
use ::std::ffi::CString;
pub const GRID_HISTORY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

/*
 * The screen sits below the history, so every position the screen names is
 * that many lines further down the grid. The counts these functions work out
 * are added and subtracted the way the C did, going round zero rather than
 * stopping at it: the grid drops a call whose rows come out beyond its own.
 */

/// The line of the whole grid that a line of the screen sits on.
fn view_y(gd: &grid, py: u_int) -> u_int {
    gd.hsize.wrapping_add(py)
}

pub fn grid_view_get_cell(gd: &grid, px: u_int, py: u_int) -> grid_cell {
    grid_get_cell(gd, px, view_y(gd, py))
}

pub fn grid_view_set_cell(gd: &mut grid, px: u_int, py: u_int, gc: &grid_cell) {
    grid_set_cell(gd, px, view_y(gd, py), gc)
}

pub fn grid_view_set_padding(gd: &mut grid, px: u_int, py: u_int) {
    grid_set_padding(gd, px, view_y(gd, py))
}

pub unsafe fn grid_view_set_cells(
    gd: &mut grid,
    px: u_int,
    py: u_int,
    gc: *const grid_cell,
    s: *const ::core::ffi::c_char,
    slen: size_t,
) {
    unsafe { grid_set_cells(gd, px, view_y(gd, py), gc, s, slen) }
}

/// Scroll everything the screen holds into the history, then clear what is
/// left of the screen.
pub fn grid_view_clear_history(gd: &mut grid, bg: u_int) {
    let mut last = 0;
    for yy in 0..gd.sy {
        if grid_get_line(gd, view_y(gd, yy)).cellused != 0 {
            last = yy + 1;
        }
    }
    if last == 0 {
        grid_view_clear(gd, 0, 0, gd.sx, gd.sy, bg);
        return;
    }
    for _ in 0..last {
        grid_collect_history(gd, 0);
        grid_scroll_history(gd, bg);
    }
    if last < gd.sy {
        grid_view_clear(gd, 0, 0, gd.sx, gd.sy - last, bg);
    }
    gd.hscrolled = 0;
}

pub fn grid_view_clear(gd: &mut grid, px: u_int, py: u_int, nx: u_int, ny: u_int, bg: u_int) {
    grid_clear(gd, px, view_y(gd, py), nx, ny, bg)
}

/// Scroll a region of the screen up, into the history when there is one.
pub fn grid_view_scroll_region_up(gd: &mut grid, rupper: u_int, rlower: u_int, bg: u_int) {
    if gd.flags & GRID_HISTORY != 0 {
        grid_collect_history(gd, 0);
        if rupper == 0 && rlower == gd.sy.wrapping_sub(1) {
            grid_scroll_history(gd, bg);
        } else {
            grid_scroll_history_region(gd, view_y(gd, rupper), view_y(gd, rlower), bg);
        }
        return;
    }
    let (rupper, rlower) = (view_y(gd, rupper), view_y(gd, rlower));
    grid_move_lines(
        gd,
        rupper,
        rupper.wrapping_add(1),
        rlower.wrapping_sub(rupper),
        bg,
    );
}

/// Scroll a region of the screen down.
pub fn grid_view_scroll_region_down(gd: &mut grid, rupper: u_int, rlower: u_int, bg: u_int) {
    let (rupper, rlower) = (view_y(gd, rupper), view_y(gd, rlower));
    grid_move_lines(
        gd,
        rupper.wrapping_add(1),
        rupper,
        rlower.wrapping_sub(rupper),
        bg,
    );
}

/// Insert lines at a line of the screen, pushing the rest of it down.
pub fn grid_view_insert_lines(gd: &mut grid, py: u_int, ny: u_int, bg: u_int) {
    let py = view_y(gd, py);
    let sy = gd.hsize.wrapping_add(gd.sy);
    grid_move_lines(
        gd,
        py.wrapping_add(ny),
        py,
        sy.wrapping_sub(py).wrapping_sub(ny),
        bg,
    );
}

/// Insert lines inside a region, pushing the rest of the region down and off
/// its end.
pub fn grid_view_insert_lines_region(
    gd: &mut grid,
    rlower: u_int,
    py: u_int,
    ny: u_int,
    bg: u_int,
) {
    let rlower = view_y(gd, rlower);
    let py = view_y(gd, py);
    let ny2 = rlower.wrapping_add(1).wrapping_sub(py).wrapping_sub(ny);
    grid_move_lines(gd, rlower.wrapping_add(1).wrapping_sub(ny2), py, ny2, bg);
    grid_clear(gd, 0, py.wrapping_add(ny2), gd.sx, ny.wrapping_sub(ny2), bg);
}

/// Delete lines at a line of the screen, pulling the rest of it up.
pub fn grid_view_delete_lines(gd: &mut grid, py: u_int, ny: u_int, bg: u_int) {
    let py = view_y(gd, py);
    let sy = gd.hsize.wrapping_add(gd.sy);
    grid_move_lines(
        gd,
        py,
        py.wrapping_add(ny),
        sy.wrapping_sub(py).wrapping_sub(ny),
        bg,
    );
    grid_clear(gd, 0, sy.wrapping_sub(ny), gd.sx, ny, bg);
}

/// Delete lines inside a region, pulling the rest of the region up.
pub fn grid_view_delete_lines_region(
    gd: &mut grid,
    rlower: u_int,
    py: u_int,
    ny: u_int,
    bg: u_int,
) {
    let rlower = view_y(gd, rlower);
    let py = view_y(gd, py);
    let ny2 = rlower.wrapping_add(1).wrapping_sub(py).wrapping_sub(ny);
    grid_move_lines(gd, py, py.wrapping_add(ny), ny2, bg);
    grid_clear(gd, 0, py.wrapping_add(ny2), gd.sx, ny.wrapping_sub(ny2), bg);
}

/// Insert cells in a line, pushing the rest of it along and off its end.
pub fn grid_view_insert_cells(gd: &mut grid, px: u_int, py: u_int, nx: u_int, bg: u_int) {
    let py = view_y(gd, py);
    let sx = gd.sx;
    if px >= sx.wrapping_sub(1) {
        grid_clear(gd, px, py, 1, 1, bg);
    } else {
        grid_move_cells(
            gd,
            px.wrapping_add(nx),
            px,
            py,
            sx.wrapping_sub(px).wrapping_sub(nx),
            bg,
        );
    }
}

/// Delete cells in a line, pulling the rest of it back.
pub fn grid_view_delete_cells(gd: &mut grid, px: u_int, py: u_int, nx: u_int, bg: u_int) {
    let py = view_y(gd, py);
    let sx = gd.sx;
    grid_move_cells(
        gd,
        px,
        px.wrapping_add(nx),
        py,
        sx.wrapping_sub(px).wrapping_sub(nx),
        bg,
    );
    grid_clear(gd, sx.wrapping_sub(nx), py, nx, 1, bg);
}

/// The text of `nx` cells of one screen line.
pub fn grid_view_string_cells(gd: &grid, px: u_int, py: u_int, nx: u_int) -> CString {
    grid_string_cells(gd, px, view_y(gd, py), nx, None, 0, null_mut())
}

#[cfg(test)]
#[path = "../tests/test_grid_view.rs"]
mod tests;

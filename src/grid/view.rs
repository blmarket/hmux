use crate::src::grid::{
    grid_clear, grid_collect_history, grid_get_cell, grid_get_line, grid_move_cells,
    grid_move_lines, grid_scroll_history, grid_scroll_history_region, grid_set_cell,
    grid_set_cells, grid_set_padding, grid_string_cells_bytes,
};
use crate::src::shared::abi::*;
use crate::src::shared::grid::*;
pub unsafe fn grid_view_get_cell(gd: &grid, px: u_int, py: u_int, gc: &mut grid_cell) {
    grid_get_cell(gd, px, gd.hsize.wrapping_add(py), gc);
}
pub unsafe fn grid_view_set_cell(gd: &mut grid, mut px: u_int, mut py: u_int, gc: &grid_cell) {
    grid_set_cell(gd, px, gd.hsize.wrapping_add(py), gc);
}
pub unsafe fn grid_view_set_padding(
    gd: &mut grid,
    mut px: u_int,
    mut py: u_int,
    mut bg: ::core::ffi::c_int,
) {
    grid_set_padding(gd, px, gd.hsize.wrapping_add(py), bg);
}
pub unsafe fn grid_view_set_cells(
    gd: &mut grid,
    px: u_int,
    py: u_int,
    gc: &grid_cell,
    bytes: &[std::ffi::c_char],
) {
    grid_set_cells(gd, px, gd.hsize.wrapping_add(py), gc, bytes);
}
pub unsafe fn grid_view_clear_history(mut gd: *mut grid, mut bg: u_int) {
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut yy: u_int = 0;
    let mut last: u_int = 0;
    last = 0 as u_int;
    yy = 0 as u_int;
    while yy < (*gd).sy {
        gl = grid_get_line(gd, (*gd).hsize.wrapping_add(yy));
        if (*gl).cellused as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            last = yy.wrapping_add(1 as u_int);
        }
        yy = yy.wrapping_add(1);
    }
    if last == 0 as u_int {
        grid_view_clear(gd, 0 as u_int, 0 as u_int, (*gd).sx, (*gd).sy, bg);
        return;
    }
    yy = 0 as u_int;
    while yy < last {
        grid_collect_history(&mut *gd, 0 as ::core::ffi::c_int);
        grid_scroll_history(&mut *gd, bg);
        yy = yy.wrapping_add(1);
    }
    if last < (*gd).sy {
        grid_view_clear(
            gd,
            0 as u_int,
            0 as u_int,
            (*gd).sx,
            (*gd).sy.wrapping_sub(last),
            bg,
        );
    }
    (*gd).hscrolled = 0 as u_int;
}
pub unsafe fn grid_view_clear(
    mut gd: *mut grid,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut ny: u_int,
    mut bg: u_int,
) {
    px = px;
    py = (*gd).hsize.wrapping_add(py);
    grid_clear(&mut *gd, px, py, nx, ny, bg);
}
pub unsafe fn grid_view_scroll_region_up(
    mut gd: *mut grid,
    mut rupper: u_int,
    mut rlower: u_int,
    mut bg: u_int,
) {
    if (*gd).flags & GRID_HISTORY != 0 {
        grid_collect_history(&mut *gd, 0 as ::core::ffi::c_int);
        if rupper == 0 as u_int && rlower == (*gd).sy.wrapping_sub(1 as u_int) {
            grid_scroll_history(&mut *gd, bg);
        } else {
            rupper = (*gd).hsize.wrapping_add(rupper);
            rlower = (*gd).hsize.wrapping_add(rlower);
            grid_scroll_history_region(&mut *gd, rupper, rlower, bg);
        }
    } else {
        rupper = (*gd).hsize.wrapping_add(rupper);
        rlower = (*gd).hsize.wrapping_add(rlower);
        grid_move_lines(
            &mut *gd,
            rupper,
            rupper.wrapping_add(1 as u_int),
            rlower.wrapping_sub(rupper),
            bg,
        );
    };
}
pub unsafe fn grid_view_scroll_region_down(
    mut gd: *mut grid,
    mut rupper: u_int,
    mut rlower: u_int,
    mut bg: u_int,
) {
    rupper = (*gd).hsize.wrapping_add(rupper);
    rlower = (*gd).hsize.wrapping_add(rlower);
    grid_move_lines(
        &mut *gd,
        rupper.wrapping_add(1 as u_int),
        rupper,
        rlower.wrapping_sub(rupper),
        bg,
    );
}
pub unsafe fn grid_view_insert_lines(
    mut gd: *mut grid,
    mut py: u_int,
    mut ny: u_int,
    mut bg: u_int,
) {
    let mut sy: u_int = 0;
    py = (*gd).hsize.wrapping_add(py);
    sy = (*gd).hsize.wrapping_add((*gd).sy);
    grid_move_lines(
        &mut *gd,
        py.wrapping_add(ny),
        py,
        sy.wrapping_sub(py).wrapping_sub(ny),
        bg,
    );
}
pub unsafe fn grid_view_insert_lines_region(
    mut gd: *mut grid,
    mut rlower: u_int,
    mut py: u_int,
    mut ny: u_int,
    mut bg: u_int,
) {
    let mut ny2: u_int = 0;
    rlower = (*gd).hsize.wrapping_add(rlower);
    py = (*gd).hsize.wrapping_add(py);
    ny2 = rlower
        .wrapping_add(1 as u_int)
        .wrapping_sub(py)
        .wrapping_sub(ny);
    grid_move_lines(
        &mut *gd,
        rlower.wrapping_add(1 as u_int).wrapping_sub(ny2),
        py,
        ny2,
        bg,
    );
    let width = (*gd).sx;
    grid_clear(
        &mut *gd,
        0 as u_int,
        py.wrapping_add(ny2),
        width,
        ny.wrapping_sub(ny2),
        bg,
    );
}
pub unsafe fn grid_view_delete_lines(
    mut gd: *mut grid,
    mut py: u_int,
    mut ny: u_int,
    mut bg: u_int,
) {
    let mut sy: u_int = 0;
    py = (*gd).hsize.wrapping_add(py);
    sy = (*gd).hsize.wrapping_add((*gd).sy);
    grid_move_lines(
        &mut *gd,
        py,
        py.wrapping_add(ny),
        sy.wrapping_sub(py).wrapping_sub(ny),
        bg,
    );
    let width = (*gd).sx;
    grid_clear(&mut *gd, 0 as u_int, sy.wrapping_sub(ny), width, ny, bg);
}
pub unsafe fn grid_view_delete_lines_region(
    mut gd: *mut grid,
    mut rlower: u_int,
    mut py: u_int,
    mut ny: u_int,
    mut bg: u_int,
) {
    let mut ny2: u_int = 0;
    rlower = (*gd).hsize.wrapping_add(rlower);
    py = (*gd).hsize.wrapping_add(py);
    ny2 = rlower
        .wrapping_add(1 as u_int)
        .wrapping_sub(py)
        .wrapping_sub(ny);
    grid_move_lines(&mut *gd, py, py.wrapping_add(ny), ny2, bg);
    let width = (*gd).sx;
    grid_clear(
        &mut *gd,
        0 as u_int,
        py.wrapping_add(ny2),
        width,
        ny.wrapping_sub(ny2),
        bg,
    );
}
pub unsafe fn grid_view_insert_cells(
    mut gd: *mut grid,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    let mut sx: u_int = 0;
    px = px;
    py = (*gd).hsize.wrapping_add(py);
    sx = (*gd).sx;
    if px >= sx.wrapping_sub(1 as u_int) {
        grid_clear(&mut *gd, px, py, 1 as u_int, 1 as u_int, bg);
    } else {
        grid_move_cells(
            &mut *gd,
            px.wrapping_add(nx),
            px,
            py,
            sx.wrapping_sub(px).wrapping_sub(nx),
            bg,
        );
    };
}
pub unsafe fn grid_view_delete_cells(
    mut gd: *mut grid,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    let mut sx: u_int = 0;
    px = px;
    py = (*gd).hsize.wrapping_add(py);
    sx = (*gd).sx;
    grid_move_cells(
        &mut *gd,
        px,
        px.wrapping_add(nx),
        py,
        sx.wrapping_sub(px).wrapping_sub(nx),
        bg,
    );
    grid_clear(&mut *gd, sx.wrapping_sub(nx), py, nx, 1 as u_int, bg);
}
pub unsafe fn grid_view_string_cells_bytes(gd: *mut grid, py: u_int, nx: u_int) -> Vec<u8> {
    let px: u_int = 0 as u_int;
    grid_string_cells_bytes(
        gd,
        px,
        (*gd).hsize.wrapping_add(py),
        nx,
        None,
        0,
        ::core::ptr::null_mut(),
    )
}

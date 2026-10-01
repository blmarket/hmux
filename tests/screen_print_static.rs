use hmux::src::grid::{grid_create, grid_default_cell, grid_set_cell};
use hmux::src::screen::screen_print;
use hmux::src::shared::screen::screen;
use std::ffi::CStr;

#[test]
fn screen_print_reuses_its_static_result_across_filtered_lines() {
    unsafe {
        let mut grid = grid_create(4, 2, 0);
        let mut screen = screen::empty();
        let mut cell = grid_default_cell;
        for (column, byte) in [(0, b'A'), (1, b'B')] {
            cell.data.data[0] = byte;
            grid_set_cell(&mut *grid, column, 0, &cell);
        }

        screen.grid = Some(grid);
        let first = screen_print(&raw mut screen, 0);
        assert_eq!(CStr::from_ptr(first).to_bytes(), b"0000 \"AB\"\n");
        let second = screen_print(&raw mut screen, 1);
        assert_eq!(first, second);
        assert_eq!(CStr::from_ptr(second).to_bytes(), b"0001 \"\"\n");
        drop(screen.grid.take());
    }
}

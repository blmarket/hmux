use hmux2::src::grid::{grid_create, grid_default_cell, grid_destroy, grid_set_cell};
use hmux2::src::screen::screen_print;
use hmux2::src::shared::screen::screen;
use std::ffi::CStr;

#[test]
fn screen_print_reuses_its_static_result_across_filtered_lines() {
    unsafe {
        let grid = grid_create(4, 2, 0);
        let mut screen = screen::empty();
        screen.grid = grid;
        let mut cell = grid_default_cell;
        for (column, byte) in [(0, b'A'), (1, b'B')] {
            cell.data.data[0] = byte;
            grid_set_cell(grid, column, 0, &cell);
        }

        let first = screen_print(&raw mut screen, 0);
        assert_eq!(CStr::from_ptr(first).to_bytes(), b"0000 \"AB\"\n");
        let second = screen_print(&raw mut screen, 1);
        assert_eq!(first, second);
        assert_eq!(CStr::from_ptr(second).to_bytes(), b"0001 \"\"\n");
        grid_destroy(grid);
    }
}

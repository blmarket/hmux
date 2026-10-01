//! A grid line's temporary decoded cells must survive conversion to an owned
//! string, including tabs, wide-cell padding, and UTF-8 bytes.

use hmux::src::format::format_grid_line;
use hmux::src::grid::{grid_create, grid_default_cell, grid_set_cell, grid_set_padding};
use hmux::src::shared::grid::GRID_FLAG_TAB;

#[test]
fn grid_line_converts_cells_and_returns_an_owned_string() {
    unsafe {
        let mut gd = grid_create(8, 2, 0);
        assert!(format_grid_line(&*gd, 0).is_none());

        let mut cell = grid_default_cell;
        cell.data.data[0] = b'A';
        grid_set_cell(&mut *gd, 0, 0, &cell);

        cell.flags = GRID_FLAG_TAB as u8;
        grid_set_cell(&mut *gd, 1, 0, &cell);

        cell.flags = 0;
        cell.data.data[..3].copy_from_slice("漢".as_bytes());
        cell.data.size = 3;
        cell.data.have = 3;
        cell.data.width = 2;
        grid_set_cell(&mut *gd, 2, 0, &cell);
        grid_set_padding(&mut *gd, 3, 0, cell.bg);

        cell.data.data = [0; 32];
        cell.data.data[0] = b'Z';
        cell.data.size = 1;
        cell.data.have = 1;
        cell.data.width = 1;
        grid_set_cell(&mut *gd, 4, 0, &cell);

        let result = format_grid_line(&*gd, 0).unwrap();
        assert_eq!(result.as_bytes(), "A\t漢Z".as_bytes());
        drop(gd);
        // The returned allocation must remain valid after the grid is gone.
        assert_eq!(result.as_bytes(), "A\t漢Z".as_bytes());
    }
}

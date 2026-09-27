use hmux2::src::grid::{
    grid_create, grid_default_cell, grid_get_cell, grid_scroll_history, grid_set_cell,
};
use hmux2::src::screen::{screen_alternate_off, screen_alternate_on, screen_free, screen_resize};
use hmux2::src::shared::grid::{grid, grid_cell, GRID_HISTORY};
use hmux2::src::shared::screen::screen;

unsafe fn cell_at(grid: &grid, x: u32, y: u32) -> grid_cell {
    let mut cell = grid_default_cell;
    grid_get_cell(grid, x, y, &mut cell);
    cell
}

#[test]
fn alternate_snapshot_survives_mutation_and_resize_then_restores() {
    unsafe {
        let mut s = screen::empty();
        s.grid = Some(grid_create(8, 3, 10));
        let mut cell = grid_default_cell;
        cell.data.data[0] = b'H';
        grid_set_cell(s.grid_mut(), 0, 0, &cell);
        grid_scroll_history(s.grid_mut(), 8);
        cell.data.data[0] = b'A';
        cell.fg = 0x2123456;
        grid_set_cell(s.grid_mut(), 0, 1, &cell);
        s.cx = 3;
        s.cy = 1;

        assert_eq!(screen_alternate_on(&mut s, &cell, 1), 1);
        let snapshot = s.saved_grid.as_deref().unwrap();
        assert_eq!((snapshot.sx, snapshot.sy, snapshot.hsize), (8, 3, 0));
        assert_eq!(cell_at(snapshot, 0, 0).data.data[0], b'A');
        assert_eq!(cell_at(snapshot, 0, 0).fg, 0x2123456);
        assert_eq!(cell_at(s.grid(), 0, 0).data.data[0], b'H');
        assert_eq!(cell_at(s.grid(), 0, 1).data.data[0], b' ');
        assert_eq!(s.grid().flags & GRID_HISTORY, 0);

        cell.data.data[0] = b'B';
        cell.fg = 2;
        grid_set_cell(s.grid_mut(), 0, 1, &cell);
        s.cx = 1;
        s.cy = 0;
        assert_eq!(screen_alternate_on(&mut s, &cell, 1), 0);
        assert_eq!(
            cell_at(s.saved_grid.as_deref().unwrap(), 0, 0).data.data[0],
            b'A'
        );
        assert_eq!((s.saved_cx, s.saved_cy), (3, 1));

        screen_resize(&mut s, 12, 5, 0);
        assert_eq!(screen_alternate_off(&mut s, Some(&mut cell), 1), 1);
        assert!(s.saved_grid.is_none());
        assert_eq!((s.grid().sx, s.grid().sy, s.grid().hsize), (12, 5, 0));
        // Growing the restored screen pulls the history row into the viewport.
        // Reflow maps a cursor on an empty row to its first column.
        assert_eq!((s.cx, s.cy), (0, 2));
        assert_eq!(cell.fg, 0x2123456);
        assert_eq!(cell_at(s.grid(), 0, 0).data.data[0], b'H');
        assert_eq!(cell_at(s.grid(), 0, 1).data.data[0], b'A');
        assert_ne!(s.grid().flags & GRID_HISTORY, 0);

        // Reenter and free while the snapshot is still owned by the screen.
        assert_eq!(screen_alternate_on(&mut s, &cell, 0), 1);
        screen_free(&mut s);
        assert!(s.saved_grid.is_none());
        assert!(s.grid.is_none());

        // A moved screen record also owns both allocations through ordinary drop.
        s.grid = Some(grid_create(8, 3, 0));
        grid_set_cell(s.grid_mut(), 0, 0, &cell);
        assert_eq!(screen_alternate_on(&mut s, &cell, 0), 1);
        let moved = s;
        assert_eq!(
            cell_at(moved.saved_grid.as_deref().unwrap(), 0, 0).fg,
            0x2123456
        );
        drop(moved);
    }
}

#[test]
fn leaving_without_a_snapshot_still_restores_and_clamps_the_saved_cursor() {
    unsafe {
        let mut s = screen::empty();
        s.grid = Some(grid_create(8, 3, 0));
        s.saved_cx = 40;
        s.saved_cy = 20;
        s.saved_cell = grid_default_cell;
        s.saved_cell.fg = 4;
        let mut cell = grid_default_cell;
        assert_eq!(screen_alternate_off(&mut s, Some(&mut cell), 1), 0);
        assert_eq!((s.cx, s.cy), (7, 2));
        assert_eq!(cell.fg, 4);
        screen_free(&mut s);
    }
}

use hmux2::src::grid::view::{grid_view_get_cell, grid_view_set_cell, grid_view_set_padding};
use hmux2::src::grid::{grid_cells_equal, grid_create, grid_default_cell};
use hmux2::src::screen_write::{
    screen_write_fast_copy, screen_write_preview, screen_write_start, screen_write_stop,
};
use hmux2::src::shared::grid::{grid_cell, GRID_ATTR_REVERSE, GRID_FLAG_PADDING};
use hmux2::src::shared::screen::{screen, MODE_CURSOR, MODE_WRAP};
use hmux2::src::shared::screen_write::screen_write_ctx;

unsafe fn make_screen(width: u32, height: u32) -> screen {
    let mut s = screen::empty();
    s.grid = Some(grid_create(width, height, 0));
    s.mode = MODE_CURSOR | MODE_WRAP;
    s.rlower = height - 1;
    s
}

unsafe fn cell_at(s: &screen, x: u32, y: u32) -> grid_cell {
    let mut cell = grid_default_cell;
    grid_view_get_cell(s.grid(), x, y, &mut cell);
    cell
}

#[test]
fn copy_clips_wide_cells_and_restores_the_destination_cursor() {
    unsafe {
        for width in [2, 3] {
            let mut src = make_screen(4, 2);
            let mut dst = make_screen(8, 4);
            let mut cell = grid_default_cell;
            cell.data.data[0] = b'A';
            grid_view_set_cell(src.grid_mut(), 0, 1, &cell);
            cell.data.data[..3].copy_from_slice("漢".as_bytes());
            cell.data.size = 3;
            cell.data.width = 2;
            cell.bg = 42;
            grid_view_set_cell(src.grid_mut(), 1, 1, &cell);
            grid_view_set_padding(src.grid_mut(), 2, 1, cell.bg);
            src.cx = 3;
            src.cy = 1;
            let before: Vec<_> = (0..4).map(|x| cell_at(&src, x, 1)).collect();
            cell = grid_default_cell;
            cell.data.data[0] = b'-';
            for y in 0..4 {
                for x in 0..8 {
                    grid_view_set_cell(dst.grid_mut(), x, y, &cell);
                }
            }
            dst.cx = 2;
            dst.cy = 1;
            let mut ctx = screen_write_ctx::default();
            screen_write_start(&mut ctx, &mut dst);
            // The request extends beyond the source's last row.
            screen_write_fast_copy(&mut ctx, &src, 0, 1, width, 4);
            screen_write_stop(&mut ctx);
            assert_eq!((dst.cx, dst.cy), (2, 1));
            assert_eq!((src.cx, src.cy), (3, 1));
            for y in 0..4 {
                for x in 0..8 {
                    let expected = if y == 1 && x == 2 {
                        before[0]
                    } else if y == 1 && width == 3 && (3..=4).contains(&x) {
                        before[(x - 2) as usize]
                    } else {
                        cell
                    };
                    assert!(grid_cells_equal(&cell_at(&dst, x, y), &expected));
                }
            }
            assert_eq!(
                cell_at(&dst, 4, 1).flags as i32 & GRID_FLAG_PADDING != 0,
                width == 3
            );
            for x in 0..4 {
                assert!(grid_cells_equal(&cell_at(&src, x, 1), &before[x as usize]));
            }
        }
    }
}

#[test]
fn preview_tracks_and_highlights_the_source_cursor_without_changing_it() {
    unsafe {
        let mut src = make_screen(6, 3);
        let mut dst = make_screen(8, 4);
        for y in 0..3 {
            for x in 0..6 {
                let mut cell = grid_default_cell;
                cell.data.data[0] = b'A' + (y * 6 + x) as u8;
                grid_view_set_cell(src.grid_mut(), x, y, &cell);
            }
        }
        src.cx = 5;
        src.cy = 2;
        dst.cx = 1;
        dst.cy = 1;
        let mut ctx = screen_write_ctx::default();
        screen_write_start(&mut ctx, &mut dst);
        screen_write_preview(&mut ctx, &src, 3, 2);
        assert_eq!((src.cx, src.cy), (5, 2));
        assert_eq!((dst.cx, dst.cy), (4, 2));
        for (y, expected) in [(1, b"JKL"), (2, b"PQR")] {
            for (offset, byte) in expected.iter().enumerate() {
                let cell = cell_at(&dst, 1 + offset as u32, y);
                assert_eq!(cell.data.data[0], *byte);
                assert_eq!(
                    cell.attr as i32 & GRID_ATTR_REVERSE != 0,
                    y == 2 && offset == 2
                );
            }
        }
        assert_eq!(cell_at(&src, 5, 2).attr, 0);
        src.mode &= !MODE_CURSOR;
        dst.cx = 0;
        dst.cy = 0;
        screen_write_preview(&mut ctx, &src, 2, 1);
        assert_eq!((dst.cx, dst.cy), (0, 0));
        assert_eq!(cell_at(&dst, 0, 0).data.data[0], b'A');
        assert_eq!(cell_at(&dst, 1, 0).data.data[0], b'B');
        screen_write_stop(&mut ctx);
    }
}

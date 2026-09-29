use hmux2::src::grid::view::grid_view_get_cell;
use hmux2::src::grid::{grid_cells_equal, grid_create, grid_default_cell};
use hmux2::src::screen_write::{
    screen_write_box, screen_write_hline, screen_write_start, screen_write_stop, screen_write_vline,
};
use hmux2::src::shared::grid::{
    grid_cell, GRID_ATTR_BRIGHT, GRID_ATTR_CHARSET, GRID_FLAG_NOPALETTE,
};
use hmux2::src::shared::screen::{screen, MODE_WRAP};
use hmux2::src::shared::screen_write::screen_write_ctx;

unsafe fn canvas() -> screen {
    let mut s = screen::empty();
    s.grid = Some(grid_create(12, 8, 0));
    s.mode = MODE_WRAP;
    s.rlower = 7;
    s.cx = 2;
    s.cy = 1;
    s
}

unsafe fn cell_at(s: &screen, x: u32, y: u32) -> grid_cell {
    let mut cell = grid_default_cell;
    grid_view_get_cell(s.grid(), x, y, &mut cell);
    cell
}

#[test]
fn boxes_preserve_borrowed_styles_and_restore_the_cursor() {
    unsafe {
        for (lines, top, bottom, side, charset) in [
            (0, "lqqqqk", "mqqqqj", "x", true),
            (1, "╔════╗", "╚════╝", "║", false),
            (2, "┏━━━━┓", "┗━━━━┛", "┃", false),
            (3, "+----+", "+----+", "|", false),
            (4, "╭────╮", "╰────╯", "│", false),
            (5, "      ", "      ", " ", false),
        ] {
            let mut s = canvas();
            let mut ctx = screen_write_ctx::default();
            let mut style = grid_default_cell;
            style.attr = (GRID_ATTR_BRIGHT | GRID_ATTR_CHARSET) as u16;
            style.fg = 3;
            style.bg = 4;
            let before = style;
            screen_write_start(&mut ctx, &mut s);
            screen_write_box(&mut ctx, 6, 4, lines, Some(&style), None);
            screen_write_stop(&mut ctx);
            assert_eq!((s.cx, s.cy), (2, 1));
            assert!(grid_cells_equal(&style, &before));
            for y in 0..4 {
                for x in 0..6 {
                    let expected = if y == 0 {
                        top.chars().nth(x)
                    } else if y == 3 {
                        bottom.chars().nth(x)
                    } else if x == 0 || x == 5 {
                        side.chars().next()
                    } else {
                        None
                    };
                    let cell = cell_at(&s, x as u32 + 2, y + 1);
                    if let Some(expected) = expected {
                        let mut bytes = [0; 4];
                        assert_eq!(
                            &cell.data.data[..cell.data.size as usize],
                            expected.encode_utf8(&mut bytes).as_bytes()
                        );
                        assert_eq!((cell.fg, cell.bg), (3, 4));
                        assert_ne!(cell.flags as i32 & GRID_FLAG_NOPALETTE, 0);
                        assert_ne!(cell.attr as i32 & GRID_ATTR_BRIGHT, 0);
                        assert_eq!(cell.attr as i32 & GRID_ATTR_CHARSET != 0, charset);
                    } else {
                        assert!(grid_cells_equal(&cell, &grid_default_cell));
                    }
                }
            }
        }
    }
}

#[test]
fn lines_use_default_styles_and_keep_the_starting_cursor() {
    unsafe {
        let mut s = canvas();
        let mut ctx = screen_write_ctx::default();
        screen_write_start(&mut ctx, &mut s);
        screen_write_hline(&mut ctx, 5, 1, 1, 0, None);
        assert_eq!((s.cx, s.cy), (2, 1));
        for (offset, byte) in b"tqqqu".iter().enumerate() {
            let cell = cell_at(&s, offset as u32 + 2, 1);
            assert_eq!(cell.data.data[0], *byte);
            assert_eq!(
                (cell.fg, cell.bg),
                (grid_default_cell.fg, grid_default_cell.bg)
            );
        }
        s.cx = 9;
        screen_write_vline(&mut ctx, 4, None);
        assert_eq!((s.cx, s.cy), (9, 1));
        for y in 1..=4 {
            assert_eq!(cell_at(&s, 9, y).data.data[0], b'x');
        }
        screen_write_stop(&mut ctx);
    }
}

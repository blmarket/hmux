use hmux2::src::grid::view::grid_view_get_cell;
use hmux2::src::grid::{grid_cells_equal, grid_create, grid_default_cell};
use hmux2::src::screen_write::{screen_write_nputs, screen_write_start, screen_write_stop};
use hmux2::src::shared::grid::{GRID_ATTR_BRIGHT, GRID_ATTR_CHARSET, GRID_FLAG_PADDING};
use hmux2::src::shared::screen::{MODE_WRAP, screen};
use hmux2::src::shared::screen_write::screen_write_ctx;

#[test]
fn bounded_text_preserves_byte_scanning_width_limits_and_style_toggles() {
    unsafe {
        assert!(!libc::setlocale(libc::LC_CTYPE, c"C.UTF-8".as_ptr()).is_null());
        for (maxlen, input, expected, columns, cy) in [
            (0, &b"a\xe2(\xa1z"[..], &b"az"[..], 2, 0),
            (0, &b"ab\xe2\x82"[..], &b"ab"[..], 2, 0),
            (0, &b"ab\0hidden"[..], &b"ab"[..], 2, 0),
            (2, "a漢Z".as_bytes(), &b"a "[..], 2, 0),
            (3, "a漢Z".as_bytes(), "a漢".as_bytes(), 3, 0),
            (0, "a漢Z".as_bytes(), "a漢Z".as_bytes(), 4, 0),
            (-1, "a漢Z".as_bytes(), "a漢Z".as_bytes(), 4, 0),
            (0, "éZ".as_bytes(), "éZ".as_bytes(), 2, 0),
            (0, &b"\x01q\x01Z"[..], &b"qZ"[..], 2, 0),
            (0, &b"a\tb"[..], &b"a\tb"[..], 3, 0),
            (3, &b"ab\nc"[..], &b"c"[..], 1, 1),
            (2, &b"ab\nc"[..], &b"ab"[..], 2, 0),
        ] {
            let mut s = screen::empty();
            s.grid = Some(grid_create(20, 3, 0));
            s.mode = MODE_WRAP;
            s.rlower = 2;
            let mut ctx = screen_write_ctx::default();
            let mut style = grid_default_cell;
            style.attr = GRID_ATTR_BRIGHT as u16;
            style.fg = 2;
            style.bg = 3;
            let before = style;
            screen_write_start(&mut ctx, &mut s);
            screen_write_nputs(&mut ctx, maxlen, &style, |out| out.write_all(input));
            screen_write_stop(&mut ctx);
            assert!(grid_cells_equal(&style, &before));
            assert_eq!((s.cx, s.cy), (columns, cy), "{input:?}, limit {maxlen}");
            let mut actual = Vec::new();
            for x in 0..columns {
                let mut cell = grid_default_cell;
                grid_view_get_cell(s.grid(), x, cy, &mut cell);
                if cell.flags as i32 & GRID_FLAG_PADDING == 0 {
                    actual.extend_from_slice(&cell.data.data[..cell.data.size as usize]);
                    assert_eq!((cell.fg, cell.bg), (2, 3));
                    assert_ne!(cell.attr as i32 & GRID_ATTR_BRIGHT, 0);
                    assert_eq!(
                        cell.attr as i32 & GRID_ATTR_CHARSET != 0,
                        input[0] == 1 && x == 0
                    );
                }
            }
            assert_eq!(actual, expected, "{input:?}, limit {maxlen}");
        }
    }
}

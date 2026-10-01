use hmux::src::grid::grid_default_cell;
use hmux::src::screen::{
    screen_check_selection, screen_clear_selection, screen_hide_selection, screen_select_cell,
    screen_set_selection,
};
use hmux::src::shared::grid::{
    GRID_ATTR_BRIGHT, GRID_ATTR_CHARSET, GRID_ATTR_NOATTR, GRID_ATTR_UNDERSCORE, GRID_FLAG_SELECTED,
};
use hmux::src::shared::key::{MODEKEY_EMACS, MODEKEY_VI};
use hmux::src::shared::screen::{screen, screen_sel};

fn selected_rows(s: &screen) -> Vec<String> {
    (0..3)
        .map(|y| {
            (0..5)
                .map(|x| {
                    if screen_check_selection(s, x, y) != 0 {
                        '#'
                    } else {
                        '.'
                    }
                })
                .collect()
        })
        .collect()
}

#[test]
fn selection_endpoints_clipping_and_rectangles_keep_tmux_geometry() {
    let mut s = screen::empty();
    assert_eq!(selected_rows(&s), [".....", ".....", "....."]);
    for (mode, expected) in [
        (MODEKEY_VI, [".####", "####.", "....."]),
        (MODEKEY_EMACS, [".####", "###..", "....."]),
    ] {
        for (sx, sy, ex, ey) in [(1, 0, 3, 1), (3, 1, 1, 0)] {
            screen_set_selection(&mut s, sx, sy, ex, ey, 0, 0, mode, &grid_default_cell);
            assert_eq!(selected_rows(&s), expected);
            screen_set_selection(&mut s, sx, sy, ex, ey, 1, 0, mode, &grid_default_cell);
            assert_eq!(selected_rows(&s), [".###.", ".###.", "....."]);
        }
    }
    screen_set_selection(&mut s, 1, 0, 3, 1, 0, 2, MODEKEY_VI, &grid_default_cell);
    assert_eq!(selected_rows(&s), ["..###", "..##.", "....."]);
    screen_set_selection(&mut s, 0, 0, 0, 1, 0, 0, MODEKEY_EMACS, &grid_default_cell);
    assert_eq!(selected_rows(&s), ["#####", "#....", "....."]);
    screen_set_selection(&mut s, 0, 1, 0, 0, 0, 0, MODEKEY_EMACS, &grid_default_cell);
    assert_eq!(selected_rows(&s), ["#####", ".....", "....."]);
    screen_hide_selection(&mut s);
    assert_eq!(selected_rows(&s), [".....", ".....", "....."]);
    screen_clear_selection(&mut s);
    assert!(s.sel.is_none());
}

#[test]
fn selected_cell_owns_glyph_and_combines_style_without_mutating_source() {
    let mut s = screen::empty();
    let mut source = grid_default_cell;
    source.fg = 1;
    source.bg = 2;
    source.flags = GRID_FLAG_SELECTED as u8;
    source.attr = (GRID_ATTR_UNDERSCORE | GRID_ATTR_CHARSET) as u16;
    source.data.data.fill(0x7f);
    source.data.data[..3].copy_from_slice("漢".as_bytes());
    source.data.size = 3;
    source.data.have = 3;
    source.data.width = 2;
    assert!(screen_select_cell(&s, &source).is_none());

    let mut style = grid_default_cell;
    style.bg = 9;
    style.us = 7;
    style.link = 91;
    style.attr = GRID_ATTR_BRIGHT as u16;
    screen_set_selection(&mut s, 0, 0, 3, 1, 0, 0, MODEKEY_VI, &style);
    let allocation = s.sel.as_deref().unwrap() as *const screen_sel as usize;
    let selected = screen_select_cell(&s, &source).unwrap();
    assert_eq!(
        (selected.fg, selected.bg, selected.us, selected.link),
        (1, 2, 7, 91)
    );
    assert_eq!(
        selected.attr,
        (GRID_ATTR_BRIGHT | GRID_ATTR_UNDERSCORE | GRID_ATTR_CHARSET) as u16
    );
    assert_eq!(selected.flags, source.flags);
    assert_eq!(&selected.data.data[..3], "漢".as_bytes());
    assert_eq!(
        (selected.data.have, selected.data.size, selected.data.width),
        (3, 3, 2)
    );
    assert_eq!(&selected.data.data[3..], &[0; 29]);
    assert_eq!(source.data.data[3], 0x7f);

    style.attr = (GRID_ATTR_BRIGHT | GRID_ATTR_NOATTR) as u16;
    style.fg = 3;
    screen_set_selection(&mut s, 0, 0, 3, 1, 0, 0, MODEKEY_VI, &style);
    assert_eq!(
        s.sel.as_deref().unwrap() as *const screen_sel as usize,
        allocation
    );
    let selected = screen_select_cell(&s, &source).unwrap();
    assert_eq!(selected.fg, 3);
    assert_eq!(
        selected.attr,
        (GRID_ATTR_BRIGHT | GRID_ATTR_NOATTR | GRID_ATTR_CHARSET) as u16
    );
    screen_hide_selection(&mut s);
    assert!(screen_select_cell(&s, &source).is_none());
}

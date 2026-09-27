use hmux2::src::grid::{grid_cells_equal, grid_default_cell};
use hmux2::src::reactor::{evbuffer_new, evbuffer_pullup};
use hmux2::src::shared::client::{client, CLIENT_UTF8};
use hmux2::src::shared::grid::{grid_cell, GRID_FLAG_PADDING};
use hmux2::src::shared::tty::{
    tty, tty_code, tty_style_ctx, tty_term, TTYC_COLORS, TTYC_SETAB, TTYC_SETAF,
};
use hmux2::src::tty::{tty_cell, tty_default_attributes};
use hmux2::src::tty_term::tty_term_ncodes;

fn character(bytes: &[u8], width: u8) -> grid_cell {
    let mut cell = grid_default_cell;
    cell.data.data = [0; 32];
    cell.data.data[..bytes.len()].copy_from_slice(bytes);
    cell.data.size = bytes.len() as u8;
    cell.data.width = width;
    cell
}

#[test]
fn borrowed_cells_preserve_utf8_conversion_control_filtering_and_sources() {
    unsafe {
        for (utf8, text, width, padding, expected) in [
            (true, "漢".as_bytes(), 2, false, "漢".as_bytes()),
            (false, "漢".as_bytes(), 2, false, b"__".as_slice()),
            (false, "─".as_bytes(), 1, false, b"q".as_slice()),
            (true, b"A".as_slice(), 1, false, b"A".as_slice()),
            (true, b"\x1b".as_slice(), 1, false, b"".as_slice()),
            (true, b"\x7f".as_slice(), 1, false, b"".as_slice()),
            (true, b"A".as_slice(), 1, true, b"".as_slice()),
        ] {
            let mut client = client::empty();
            client.flags = if utf8 { CLIENT_UTF8 as u64 } else { 0 };
            let mut term = tty_term::empty();
            term.codes = vec![tty_code::None; tty_term_ncodes() as usize].into_boxed_slice();
            let mut terminal = tty {
                client: &raw mut client,
                term: &raw mut term,
                out: Some(evbuffer_new()),
                cell: grid_default_cell,
                last_cell: grid_default_cell,
                sx: 20,
                sy: 2,
                ..Default::default()
            };
            let mut source = character(text, width);
            if padding {
                source.flags |= GRID_FLAG_PADDING as u8;
            }
            let before = source;
            tty_cell(&raw mut terminal, &source, None);
            assert_eq!(
                evbuffer_pullup(terminal.out.as_deref_mut().unwrap(), -1).unwrap_or_default(),
                expected
            );
            assert_eq!(
                terminal.cx,
                if expected.is_empty() { 0 } else { width as u32 }
            );
            assert!(grid_cells_equal(&source, &before));
        }
    }
}

#[test]
fn style_defaults_apply_to_copies_and_default_background_can_override_them() {
    unsafe {
        let mut client = client::empty();
        let mut term = tty_term::empty();
        term.codes = vec![tty_code::None; tty_term_ncodes() as usize].into_boxed_slice();
        term.codes[TTYC_COLORS as usize] = tty_code::Number(8);
        term.codes[TTYC_SETAF as usize] = tty_code::String(c"F%p1%d".to_owned());
        term.codes[TTYC_SETAB as usize] = tty_code::String(c"B%p1%d".to_owned());
        let mut terminal = tty {
            client: &raw mut client,
            term: &raw mut term,
            out: Some(evbuffer_new()),
            cell: grid_default_cell,
            last_cell: grid_default_cell,
            sx: 20,
            sy: 2,
            ..Default::default()
        };
        let source = character(b"A", 1);
        let before = source;
        let defaults = grid_cell {
            fg: 2,
            bg: 4,
            ..grid_default_cell
        };
        let style = tty_style_ctx {
            defaults,
            ..Default::default()
        };
        tty_cell(&raw mut terminal, &source, Some(&style));
        assert_eq!((terminal.last_cell.fg, terminal.last_cell.bg), (2, 4));
        tty_default_attributes(&raw mut terminal, 3, Some(&style));
        assert_eq!((terminal.last_cell.fg, terminal.last_cell.bg), (2, 3));
        assert_eq!(
            evbuffer_pullup(terminal.out.as_deref_mut().unwrap(), -1).unwrap(),
            b"F2B4AB3"
        );
        assert!(grid_cells_equal(&source, &before));
        assert_eq!((defaults.fg, defaults.bg), (2, 4));
    }
}

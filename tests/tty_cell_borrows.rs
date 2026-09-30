use hmux2::src::grid::{grid_cells_equal, grid_create, grid_default_cell};
use hmux2::src::reactor::{evbuffer_new, evbuffer_pullup};
use hmux2::src::server_client::Client as _;
use hmux2::src::shared::client::{client, CLIENT_UTF8};
use hmux2::src::shared::grid::{grid_cell, GRID_FLAG_PADDING};
use hmux2::src::shared::screen::screen;
use hmux2::src::shared::tty::{
    tty, tty_code, tty_ctx, tty_style_ctx, tty_term, TTYC_COLORS, TTYC_SETAB, TTYC_SETAF,
};
use hmux2::src::tty::{tty_cell, tty_cmd_cell, tty_default_attributes};
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
            let client_owner = client::new();
            client_owner.update_flags(
                if utf8 { CLIENT_UTF8 as u64 } else { 0 },
                if utf8 { 0 } else { CLIENT_UTF8 as u64 },
            );
            let mut term = tty_term::empty();
            term.codes = vec![tty_code::None; tty_term_ncodes() as usize].into_boxed_slice();
            *client_owner.borrow_terminal_mut() = tty {
                client: std::rc::Rc::downgrade(&client_owner),
                term: Some(Box::new(term)),
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
            let mut screen = screen::empty();
            screen.grid = Some(grid_create(20, 2, 0));
            let ctx = tty_ctx {
                sx: 20,
                sy: 2,
                orlower: 1,
                style_ctx: tty_style_ctx {
                    defaults: grid_default_cell,
                    ..Default::default()
                },
                ..Default::default()
            };
            tty_cmd_cell(&client_owner, &ctx, &screen, &source);
            assert_eq!(
                evbuffer_pullup(
                    client_owner
                        .borrow_terminal_mut()
                        .out
                        .as_deref_mut()
                        .unwrap(),
                    -1
                )
                .unwrap_or_default(),
                expected
            );
            assert_eq!(
                client_owner.borrow_terminal().cx,
                if expected.is_empty() { 0 } else { width as u32 }
            );
            assert!(grid_cells_equal(&source, &before));
        }
    }
}

#[test]
fn style_defaults_apply_to_copies_and_default_background_can_override_them() {
    unsafe {
        let client_owner = client::new();
        let mut term = tty_term::empty();
        term.codes = vec![tty_code::None; tty_term_ncodes() as usize].into_boxed_slice();
        term.codes[TTYC_COLORS as usize] = tty_code::Number(8);
        term.codes[TTYC_SETAF as usize] = tty_code::String(c"F%p1%d".to_owned());
        term.codes[TTYC_SETAB as usize] = tty_code::String(c"B%p1%d".to_owned());
        *client_owner.borrow_terminal_mut() = tty {
            client: std::rc::Rc::downgrade(&client_owner),
            term: Some(Box::new(term)),
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
        tty_cell(&client_owner, &source, Some(&style));
        assert_eq!(
            (
                client_owner.borrow_terminal().last_cell.fg,
                client_owner.borrow_terminal().last_cell.bg
            ),
            (2, 4)
        );
        tty_default_attributes(&client_owner, 3, Some(&style));
        assert_eq!(
            (
                client_owner.borrow_terminal().last_cell.fg,
                client_owner.borrow_terminal().last_cell.bg
            ),
            (2, 3)
        );
        assert_eq!(
            evbuffer_pullup(
                client_owner
                    .borrow_terminal_mut()
                    .out
                    .as_deref_mut()
                    .unwrap(),
                -1
            )
            .unwrap(),
            b"F2B4AB3"
        );
        assert!(grid_cells_equal(&source, &before));
        assert_eq!((defaults.fg, defaults.bg), (2, 4));
    }
}

#[test]
fn line_rendering_preserves_source_cells_through_selection_conversion_and_clipping() {
    use hmux2::src::grid::{grid_get_cell, grid_set_cell};
    use hmux2::src::screen::screen_set_selection;
    use hmux2::src::shared::grid::GRID_FLAG_SELECTED;
    use hmux2::src::shared::key::MODEKEY_VI;
    use hmux2::src::tty_draw::tty_draw_line;

    unsafe {
        for (utf8, start, width, expected) in [
            (true, 0, 6, "漢AB界".as_bytes()),
            (false, 0, 6, b"__AB__".as_slice()),
            (true, 1, 4, b" AB ".as_slice()),
        ] {
            let client_owner = client::new();
            client_owner.update_flags(
                if utf8 { CLIENT_UTF8 as u64 } else { 0 },
                if utf8 { 0 } else { CLIENT_UTF8 as u64 },
            );
            let mut term = tty_term::empty();
            term.codes = vec![tty_code::None; tty_term_ncodes() as usize].into_boxed_slice();
            *client_owner.borrow_terminal_mut() = tty {
                client: std::rc::Rc::downgrade(&client_owner),
                term: Some(Box::new(term)),
                out: Some(evbuffer_new()),
                cell: grid_default_cell,
                last_cell: grid_default_cell,
                ccolour: -1,
                sx: 20,
                sy: 2,
                ..Default::default()
            };
            let mut screen = screen::empty();
            screen.grid = Some(grid_create(6, 2, 0));
            screen.ccolour = -1;
            screen.default_ccolour = -1;
            let padding = grid_cell {
                flags: GRID_FLAG_PADDING as u8,
                ..grid_default_cell
            };
            let selected = grid_cell {
                flags: GRID_FLAG_SELECTED as u8,
                ..character(b"B", 1)
            };
            let source = [
                character("漢".as_bytes(), 2),
                padding,
                character(b"A", 1),
                selected,
                character("界".as_bytes(), 2),
                padding,
            ];
            for (x, cell) in source.iter().enumerate() {
                grid_set_cell(screen.grid_mut(), x as u32, 0, cell);
            }
            let selection_style = grid_cell {
                fg: 2,
                bg: 4,
                ..grid_default_cell
            };
            screen_set_selection(&mut screen, 3, 0, 3, 0, 0, 6, MODEKEY_VI, &selection_style);
            tty_draw_line(&client_owner, &screen, start, 0, width, 0, 0, None);
            assert_eq!(
                evbuffer_pullup(
                    client_owner
                        .borrow_terminal_mut()
                        .out
                        .as_deref_mut()
                        .unwrap(),
                    -1
                )
                .unwrap(),
                expected
            );
            assert_eq!(client_owner.borrow_terminal().cx, width);
            for (x, before) in source.iter().enumerate() {
                let mut after = grid_default_cell;
                grid_get_cell(screen.grid(), x as u32, 0, &mut after);
                assert!(grid_cells_equal(before, &after), "cell {x} was changed");
            }
            assert!(grid_cells_equal(
                &screen.sel.as_ref().unwrap().cell,
                &selection_style
            ));
        }
    }
}

#[test]
fn optional_screen_cursor_style_preserves_defaults_and_explicit_overrides() {
    use hmux2::src::shared::display::{SCREEN_CURSOR_BAR, SCREEN_CURSOR_UNDERLINE};
    use hmux2::src::shared::screen::{MODE_CURSOR, MODE_CURSOR_BLINKING};
    use hmux2::src::shared::tty::{TTYC_CIVIS, TTYC_CNORM, TTYC_SS};
    use hmux2::src::tty::tty_update_mode;

    unsafe {
        let client_owner = client::new();
        let mut term = tty_term::empty();
        term.codes = vec![tty_code::None; tty_term_ncodes() as usize].into_boxed_slice();
        term.codes[TTYC_CNORM as usize] = tty_code::String(c"N".to_owned());
        term.codes[TTYC_CIVIS as usize] = tty_code::String(c"I".to_owned());
        term.codes[TTYC_SS as usize] = tty_code::String(c"S%p1%d".to_owned());
        *client_owner.borrow_terminal_mut() = tty {
            client: std::rc::Rc::downgrade(&client_owner),
            term: Some(Box::new(term)),
            out: Some(evbuffer_new()),
            ccolour: -1,
            ..Default::default()
        };
        let mut screen = screen::empty();
        screen.ccolour = -1;
        screen.default_ccolour = -1;
        screen.default_cstyle = SCREEN_CURSOR_UNDERLINE;
        screen.default_mode = MODE_CURSOR_BLINKING;
        tty_update_mode(&client_owner, MODE_CURSOR, Some((&screen).into()));
        assert_eq!(
            client_owner.borrow_terminal().cstyle,
            SCREEN_CURSOR_UNDERLINE
        );
        assert_eq!(
            client_owner.borrow_terminal().mode,
            MODE_CURSOR | MODE_CURSOR_BLINKING
        );

        // Without a screen, retain the previous style but use the requested blink mode.
        tty_update_mode(&client_owner, MODE_CURSOR, None);
        assert_eq!(
            client_owner.borrow_terminal().cstyle,
            SCREEN_CURSOR_UNDERLINE
        );
        assert_eq!(client_owner.borrow_terminal().mode, MODE_CURSOR);

        screen.cstyle = SCREEN_CURSOR_BAR;
        tty_update_mode(&client_owner, MODE_CURSOR, Some((&screen).into()));
        assert_eq!(client_owner.borrow_terminal().cstyle, SCREEN_CURSOR_BAR);
        assert_eq!(client_owner.borrow_terminal().mode, MODE_CURSOR);
        tty_update_mode(&client_owner, 0, None);
        assert_eq!(client_owner.borrow_terminal().mode, 0);
        assert_eq!(
            evbuffer_pullup(
                client_owner
                    .borrow_terminal_mut()
                    .out
                    .as_deref_mut()
                    .unwrap(),
                -1
            )
            .unwrap(),
            b"NS3NS4NS6I"
        );
    }
}

#[test]
fn palette_changes_apply_to_all_colour_channels_without_changing_the_source() {
    use hmux2::src::shared::colour::{colour_palette, COLOUR_FLAG_256};
    use hmux2::src::shared::tty::TTYC_SETULC1;
    use hmux2::src::style::colour::{colour_palette_init, colour_palette_set};

    unsafe {
        let client_owner = client::new();
        client_owner.update_flags(CLIENT_UTF8 as u64, 0);
        let mut term = tty_term::empty();
        term.codes = vec![tty_code::None; tty_term_ncodes() as usize].into_boxed_slice();
        term.codes[TTYC_COLORS as usize] = tty_code::Number(8);
        term.codes[TTYC_SETAF as usize] = tty_code::String(c"F%p1%d".to_owned());
        term.codes[TTYC_SETAB as usize] = tty_code::String(c"B%p1%d".to_owned());
        term.codes[TTYC_SETULC1 as usize] = tty_code::String(c"U%p1%d".to_owned());
        *client_owner.borrow_terminal_mut() = tty {
            client: std::rc::Rc::downgrade(&client_owner),
            term: Some(Box::new(term)),
            out: Some(evbuffer_new()),
            cell: grid_default_cell,
            last_cell: grid_default_cell,
            sx: 20,
            sy: 2,
            ..Default::default()
        };
        let mut palette = colour_palette::default();
        colour_palette_init(&mut palette);
        colour_palette_set(Some(&mut palette), 1, 2);
        colour_palette_set(Some(&mut palette), 4, 6);
        let palette = refbox::RefBox::new(palette);
        let style = tty_style_ctx {
            defaults: grid_cell {
                fg: 1,
                bg: 4,
                ..grid_default_cell
            },
            palette: hmux2::src::shared::tty::PaletteSource::Popup(palette.downgrade()),
            ..Default::default()
        };
        let source = grid_cell {
            us: COLOUR_FLAG_256 | 1,
            ..character(b"A", 1)
        };
        let before = source;
        tty_cell(&client_owner, &source, Some(&style));
        assert_eq!(
            (
                client_owner.borrow_terminal().last_cell.fg,
                client_owner.borrow_terminal().last_cell.bg,
                client_owner.borrow_terminal().last_cell.us
            ),
            (2, 6, 2)
        );
        colour_palette_set(Some(&mut palette.try_borrow_mut().unwrap()), 1, 5);
        tty_cell(&client_owner, &source, Some(&style));
        assert_eq!(
            (
                client_owner.borrow_terminal().last_cell.fg,
                client_owner.borrow_terminal().last_cell.bg,
                client_owner.borrow_terminal().last_cell.us
            ),
            (5, 6, 5)
        );
        assert_eq!(
            evbuffer_pullup(
                client_owner
                    .borrow_terminal_mut()
                    .out
                    .as_deref_mut()
                    .unwrap(),
                -1
            )
            .unwrap(),
            b"F2B6U2AF5U5A"
        );
        assert!(grid_cells_equal(&source, &before));
        assert_eq!((style.defaults.fg, style.defaults.bg), (1, 4));
    }
}

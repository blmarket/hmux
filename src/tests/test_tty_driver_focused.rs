use super::*;
use crate::WindowPane;
use crate::pane_style_cache::PaneStyleCache;
use crate::reactor::ByteBuffer;
use crate::terminfo::TerminalCapabilities;
use crate::tests::test_fixtures::{globals, zeroed_client, zeroed_term, zeroed_tty};
use core::ffi::c_int;
use std::ffi::CString;

struct Terminal {
    tty: Box<tty>,
    client: ClientRef,
    seen: usize,
}

impl Terminal {
    fn new(sx: u_int, sy: u_int) -> Self {
        let client = zeroed_client();
        let mut tty = zeroed_tty();
        tty.client = Some(client.downgrade());
        tty.term = Some(zeroed_term());
        tty.out = Some(Box::new(ByteBuffer::new()));
        tty.sx = sx;
        tty.sy = sy;
        tty.rlower = sy.saturating_sub(1);
        tty.rright = sx.saturating_sub(1);
        Self {
            tty,
            client,
            seen: 0,
        }
    }

    fn set(&mut self, code: tty_code_code, value: &'static core::ffi::CStr) {
        let mut terminal = self.tty.term.as_ref().unwrap().borrow_mut();
        let mut capability = terminal.capability_name(code).to_bytes().to_vec();
        capability.push(b'=');
        for &byte in value.to_bytes() {
            capability.push(byte);
            if byte == b':' {
                capability.push(byte);
            }
        }
        terminal.apply_overrides(&CString::new(capability).unwrap());
    }

    fn flag(&mut self, flag: tty_code_code, value: i32) {
        assert_ne!(value, 0);
        let mut terminal = self.tty.term.as_ref().unwrap().borrow_mut();
        let capability = terminal.capability_name(flag).to_owned();
        terminal.apply_overrides(&capability);
    }

    fn number(&mut self, code: tty_code_code, value: i32) {
        let mut terminal = self.tty.term.as_ref().unwrap().borrow_mut();
        let capability = CString::new(format!(
            "{}={value}",
            terminal.capability_name(code).to_string_lossy()
        ))
        .unwrap();
        terminal.apply_overrides(&capability);
    }

    fn output(&mut self) -> Vec<u8> {
        let out = self.tty.out.as_mut().unwrap().as_slice();
        let bytes = out[self.seen..].to_vec();
        self.seen = out.len();
        bytes
    }
}

#[test]
fn output_primitives_buffer_discard_wrap_and_translate_acs() {
    let _guard = globals();
    let mut t = Terminal::new(5, 2);
    unsafe {
        tty_puts(&mut t.tty, c"");
        assert!(t.output().is_empty());
        tty_puts(&mut t.tty, c"ab");
        tty_putc(&mut t.tty, b'c');
        tty_putn(&mut t.tty, b"de", 2);
        assert_eq!(t.output(), b"abcde");
        assert_eq!((t.tty.cx, t.client.as_client().written), (3, 5));

        t.tty.flags |= TTY_BLOCK;
        tty_putn(&mut t.tty, b"ignored", 7);
        assert!(t.output().is_empty());
        assert_eq!(t.tty.discarded, 7);
        t.tty.flags &= !TTY_BLOCK;

        t.tty.cx = 5;
        t.tty.cy = 0;
        tty_putc(&mut t.tty, b'x');
        assert_eq!(t.output(), b"x");
        assert_eq!((t.tty.cx, t.tty.cy), (1, 1));

        t.tty.term.as_ref().unwrap().borrow_mut().refresh_derived();
        t.tty.cx = 4;
        t.tty.cy = 1;
        tty_putc(&mut t.tty, b'z');
        assert!(t.output().is_empty());
        tty_putn(&mut t.tty, b"xyz", 3);
        assert!(t.output().is_empty());

        t.flag(TTYC_AM, 1);
        t.set(TTYC_ACSC, c"q-");
        t.tty.term.as_ref().unwrap().borrow_mut().refresh_derived();
        t.tty.cell.attr |= GRID_ATTR_CHARSET as u_short;
        t.tty.cx = 0;
        t.tty.cy = 0;
        tty_putc(&mut t.tty, b'q');
        assert_eq!(t.output(), b"-");
    }
}

#[test]
fn terminfo_emitters_expand_parameters_and_ignore_negative_numbers() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    let mut t = Terminal::new(80, 24);
    t.set(TTYC_CUF, c"R%p1%d");
    t.set(TTYC_CUP, c"P%p1%d,%p2%d");
    t.set(TTYC_CMG, c"M%p1%d,%p2%d");
    t.set(TTYC_HLS, c"H%p1%s:%p2%s");
    {
        tty_putcode_i(&mut t.tty, TTYC_CUF, -1);
        tty_putcode_ii(&mut t.tty, TTYC_CUP, 1, -1);
        tty_putcode_iii(&mut t.tty, TTYC_CMG, 1, 2, -1);
        assert!(t.output().is_empty());
        tty_putcode_i(&mut t.tty, TTYC_CUF, 7);
        tty_putcode_ii(&mut t.tty, TTYC_CUP, 2, 9);
        tty_putcode_ss(&mut t.tty, TTYC_HLS, c"id", c"uri");
        assert_eq!(t.output(), b"R7P2,9Hid:uri");
    }
}

#[test]
fn titles_paths_selection_and_progress_are_capability_gated() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    let mut t = Terminal::new(80, 24);
    unsafe {
        tty_set_title(&mut t.tty, c"title");
        tty_set_path(&mut t.tty, c"path");
        t.tty.flags |= TTY_STARTED;
        tty_set_selection(&mut t.tty, c"c", b"abc");
        t.tty.flags &= !TTY_STARTED;
        tty_set_progress_bar(&mut t.tty, &progress_bar::default());
        assert!(t.output().is_empty());
    }
    t.set(TTYC_TSL, c"<t>");
    t.set(TTYC_SWD, c"<p>");
    t.set(TTYC_FSL, c"</>");
    t.set(TTYC_MS, c"S%p1%s:%p2%s");
    t.set(TTYC_SPB, c"B%p1%d:%p2%d");
    unsafe {
        tty_set_title(&mut t.tty, c"title");
        tty_set_path(&mut t.tty, c"path");
        t.tty.flags |= TTY_STARTED;
        tty_set_selection(&mut t.tty, c"c", b"abc");
        t.tty.flags &= !TTY_STARTED;
        let pb = progress_bar {
            state: PROGRESS_BAR_NORMAL,
            progress: 42,
        };
        tty_set_progress_bar(&mut t.tty, &pb);
        assert_eq!(t.output(), b"<t>title</><p>path</>Sc:YWJjB1:42");
    }
}

#[test]
fn repeat_space_and_emulated_repeat_cover_native_and_fallback_paths() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    let mut t = Terminal::new(2000, 2);
    t.set(TTYC_CUF1, c">");
    unsafe {
        tty_emulate_repeat(&mut t.tty, TTYC_CUF, TTYC_CUF1, 3);
        assert_eq!(t.output(), b">>>");
        t.set(TTYC_CUF, c"R%p1%d");
        tty_emulate_repeat(&mut t.tty, TTYC_CUF, TTYC_CUF1, 4);
        assert_eq!(t.output(), b"R4");
        tty_repeat_space(&mut t.tty, 1200);
        assert_eq!(t.output(), vec![b' '; 1200]);
    }
}

#[test]
fn visibility_and_clamping_cover_inside_outside_and_each_edge() {
    let mut ctx = tty_ctx {
        sx: 20,
        sy: 10,
        ..Default::default()
    };
    assert_eq!(tty_is_visible(&ctx, 50, 50, 1, 1), 1);
    ctx.flags |= TTY_CTX_WINDOW_BIGGER;
    ctx.wox = 5;
    ctx.woy = 3;
    ctx.wsx = 10;
    ctx.wsy = 5;
    ctx.xoff = 0;
    ctx.yoff = 0;
    ctx.rxoff = 0;
    ctx.ryoff = 0;
    assert_eq!(tty_is_visible(&ctx, 0, 0, 2, 2), 0);
    assert_eq!(tty_is_visible(&ctx, 6, 4, 2, 2), 1);
    {
        assert_eq!(tty_clamp_line(&ctx, 6, 4, 2), Some((0, 1, 2, 1)));
        assert_eq!(tty_clamp_line(&ctx, 3, 4, 5), Some((2, 0, 3, 1)));
        assert_eq!(tty_clamp_line(&ctx, 13, 4, 5), Some((0, 8, 2, 1)));
        assert_eq!(tty_clamp_line(&ctx, 3, 4, 14), Some((5, 0, 10, 1)));
        assert!(tty_clamp_line(&ctx, 0, 0, 1).is_none());
        assert_eq!(tty_clamp_area(&ctx, 6, 4, 2, 2), Some((0, 0, 1, 1, 2, 2)));
        assert_eq!(tty_clamp_area(&ctx, 3, 1, 14, 9), Some((5, 3, 0, 0, 10, 5)));
        assert!(tty_clamp_area(&ctx, 30, 30, 1, 1).is_none());
    }
}

#[test]
fn bce_region_margin_invalidation_and_reset_update_cached_state() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    let mut t = Terminal::new(20, 10);
    let mut gc = { grid_default_cell };
    unsafe {
        assert_eq!(tty_fake_bce(&t.tty, &gc, 8), 0);
        gc.bg = 1;
        assert_eq!(tty_fake_bce(&t.tty, &gc, 8), 1);
        t.flag(TTYC_BCE, 1);
        assert_eq!(tty_fake_bce(&t.tty, &gc, 1), 0);

        tty_invalidate(&mut t.tty);
        assert_eq!((t.tty.cx, t.tty.cy), (UINT_MAX, UINT_MAX));
        assert_eq!(t.tty.mode, MODE_CURSOR);

        tty_region(&mut t.tty, 1, 8);
        assert_eq!((t.tty.rupper, t.tty.rlower), (UINT_MAX, UINT_MAX));
        t.set(TTYC_CSR, c"R%p1%d,%p2%d");
        tty_region(&mut t.tty, 1, 8);
        assert_eq!((t.tty.rupper, t.tty.rlower), (1, 8));
        assert_eq!(t.output(), b"R1,8");

        t.set(TTYC_CMG, c"M%p1%d,%p2%d");
        t.set(TTYC_CLMG, c"C");
        t.tty.term.as_ref().unwrap().borrow_mut().refresh_derived();
        tty_margin(&mut t.tty, 2, 17);
        assert_eq!((t.tty.rleft, t.tty.rright), (2, 17));
        assert_eq!(t.output(), b"R1,8M2,17");
        tty_margin_off(&mut t.tty);
        assert_eq!(t.output(), b"R1,8C");

        t.tty.cell.attr = GRID_ATTR_BRIGHT as u_short;
        t.set(TTYC_SGR0, c"0");
        tty_reset(&mut t.tty);
        assert_eq!(t.output(), b"0");
        assert_eq!(grid_cells_equal(&t.tty.cell, &grid_default_cell), 1);
    }
}

#[test]
fn cursor_chooses_home_relative_absolute_and_clamped_paths() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    let mut t = Terminal::new(20, 10);
    for (code, value) in [
        (TTYC_HOME, c"H"),
        (TTYC_CUB1, c"L"),
        (TTYC_CUF1, c"R"),
        (TTYC_CUU1, c"U"),
        (TTYC_CUD1, c"D"),
        (TTYC_CUP, c"P%p1%d,%p2%d"),
        (TTYC_CUB, c"l%p1%d"),
        (TTYC_CUF, c"r%p1%d"),
        (TTYC_CUU, c"u%p1%d"),
        (TTYC_CUD, c"d%p1%d"),
        (TTYC_HPA, c"x%p1%d"),
        (TTYC_VPA, c"y%p1%d"),
    ] {
        t.set(code, value);
    }
    unsafe {
        t.tty.cx = 5;
        t.tty.cy = 5;
        tty_cursor(&mut t.tty, 4, 5);
        tty_cursor(&mut t.tty, 5, 5);
        tty_cursor(&mut t.tty, 5, 4);
        tty_cursor(&mut t.tty, 5, 5);
        tty_cursor(&mut t.tty, 0, 0);
        tty_cursor(&mut t.tty, 100, 9);
        assert_eq!(t.output(), b"LRUDHP9,19");
        assert_eq!((t.tty.cx, t.tty.cy), (19, 9));

        t.tty.flags |= TTY_BLOCK;
        tty_cursor(&mut t.tty, 1, 1);
        assert_eq!((t.tty.cx, t.tty.cy), (19, 9));
    }
}

#[test]
fn mode_updates_emit_mouse_protocol_and_cursor_visibility() {
    let _guard = globals();
    let mut t = Terminal::new(80, 24);
    t.set(TTYC_KMOUS, c"m");
    t.set(TTYC_CIVIS, c"I");
    t.set(TTYC_CNORM, c"N");
    t.set(TTYC_CVVIS, c"V");
    unsafe {
        t.tty.mode = MODE_CURSOR;
        tty_update_mode(&mut t.tty, 0, None);
        assert_eq!(t.output(), b"I");
        tty_update_mode(&mut t.tty, MODE_CURSOR | MODE_MOUSE_STANDARD, None);
        let output = t.output();
        assert!(output.starts_with(b"N"));
        assert!(output.ends_with(b"\x1b[?1006h\x1b[?1000h"));
        tty_update_mode(&mut t.tty, MODE_CURSOR | MODE_MOUSE_ALL, None);
        assert!(t.output().ends_with(b"\x1b[?1000h\x1b[?1002h\x1b[?1003h"));
        t.tty.flags |= TTY_NOCURSOR;
        tty_update_mode(&mut t.tty, MODE_CURSOR, None);
        assert_eq!(t.tty.mode & MODE_CURSOR, 0);
    }
}

#[test]
fn colour_normalization_and_emitters_cover_basic_256_and_rgb_paths() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    let mut t = Terminal::new(80, 24);
    t.number(TTYC_COLORS, 8);
    let mut gc = { grid_default_cell };
    {
        gc.fg = 91;
        tty_check_fg(&mut t.tty, None, &mut gc);
        assert_eq!(gc.fg, 1);
        assert_ne!(gc.attr as c_int & GRID_ATTR_BRIGHT, 0);
        gc.bg = 94;
        tty_check_bg(&mut t.tty, None, &mut gc);
        assert_eq!(gc.bg, 4);

        gc.fg = COLOUR_FLAG_RGB | 0x336699;
        tty_check_fg(&mut t.tty, None, &mut gc);
        assert_eq!(gc.fg & COLOUR_FLAG_RGB, 0);
        t.tty
            .term
            .as_ref()
            .unwrap()
            .borrow_mut()
            .apply_features(1 << 15);
        gc.fg = COLOUR_FLAG_RGB | 0x123456;
        tty_check_fg(&mut t.tty, None, &mut gc);
        assert_eq!(gc.fg, COLOUR_FLAG_RGB | 0x123456);

        t.set(TTYC_SETAF, c"f%p1%d");
        t.set(TTYC_SETAB, c"b%p1%d");
        t.set(TTYC_SETRGBF, c"F%p1%d,%p2%d,%p3%d");
        t.set(TTYC_SETRGBB, c"B%p1%d,%p2%d,%p3%d");
        assert_eq!(tty_try_colour(&mut t.tty, COLOUR_FLAG_256 | 123, c"38"), 0);
        assert_eq!(tty_try_colour(&mut t.tty, COLOUR_FLAG_256 | 45, c"48"), 0);
        assert_eq!(
            tty_try_colour(&mut t.tty, COLOUR_FLAG_RGB | 0x123456, c"38"),
            0
        );
        assert_eq!(
            tty_try_colour(&mut t.tty, COLOUR_FLAG_RGB | 0xabcdef, c"48"),
            0
        );
        assert_eq!(tty_try_colour(&mut t.tty, 7, c"38"), -1);
        assert_eq!(t.output(), b"f123b45F18,52,86B171,205,239");
    }
}

#[test]
fn attributes_emit_each_supported_decoration_and_reset_removed_ones() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    let mut t = Terminal::new(80, 24);
    for (code, value) in [
        (TTYC_BOLD, c"A"),
        (TTYC_DIM, c"D"),
        (TTYC_SMSO, c"I"),
        (TTYC_SMUL, c"U"),
        (TTYC_BLINK, c"K"),
        (TTYC_REV, c"R"),
        (TTYC_INVIS, c"H"),
        (TTYC_SMXX, c"X"),
        (TTYC_SMOL, c"O"),
        (TTYC_SGR0, c"0"),
        (TTYC_SETAF, c"f%p1%d"),
        (TTYC_SETAB, c"b%p1%d"),
    ] {
        t.set(code, value);
    }
    let defaults = { grid_default_cell };
    let mut gc = defaults;
    gc.fg = 2;
    gc.bg = 3;
    gc.attr = (GRID_ATTR_BRIGHT
        | GRID_ATTR_DIM
        | GRID_ATTR_ITALICS
        | GRID_ATTR_UNDERSCORE
        | GRID_ATTR_BLINK
        | GRID_ATTR_REVERSE
        | GRID_ATTR_HIDDEN
        | GRID_ATTR_STRIKETHROUGH
        | GRID_ATTR_OVERLINE) as u_short;
    unsafe {
        tty_attributes(&mut t.tty, &gc, &defaults, None, None);
        let output = t.output();
        for marker in b"ADUIKRHXO" {
            assert!(
                output.contains(marker),
                "missing marker {marker:?} in {output:?}"
            );
        }
        assert_eq!(t.tty.last_cell.attr, gc.attr);
        tty_attributes(&mut t.tty, &defaults, &defaults, None, None);
        assert!(t.output().contains(&b'0'));
        tty_attributes(&mut t.tty, &defaults, &defaults, None, None);
        assert!(t.output().is_empty());
    }
}

#[test]
fn synchronized_output_is_idempotent_and_respects_blocking() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    let mut t = Terminal::new(80, 24);
    t.set(TTYC_SYNC, c"S%p1%d");
    unsafe {
        tty_sync_start(&mut t.tty);
        tty_sync_start(&mut t.tty);
        assert_eq!(t.output(), b"S1");
        assert_ne!(t.tty.flags & TTY_SYNCING, 0);
        tty_sync_end(&mut t.tty);
        tty_sync_end(&mut t.tty);
        assert_eq!(t.output(), b"S2");
        assert_eq!(t.tty.flags & TTY_SYNCING, 0);
        t.tty.flags |= TTY_BLOCK;
        tty_sync_start(&mut t.tty);
        assert_eq!(t.tty.flags & TTY_SYNCING, 0);
        assert!(t.output().is_empty());
    }
}

#[test]
fn cursor_colour_and_shape_cover_reset_and_each_style_code() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    let mut t = Terminal::new(80, 24);
    t.set(TTYC_CS, c"C%p1%s");
    t.set(TTYC_CR, c"R");
    t.set(TTYC_CNORM, c"N");
    t.set(TTYC_CIVIS, c"I");
    t.set(TTYC_CVVIS, c"V");
    t.set(TTYC_SS, c"S%p1%d");
    t.set(TTYC_SE, c"E");
    {
        tty_force_cursor_colour(&mut t.tty, COLOUR_FLAG_RGB | 0x123456);
        assert!(t.output().starts_with(b"Crgb:12/34/56"));
        tty_force_cursor_colour(&mut t.tty, COLOUR_FLAG_RGB | 0x123456);
        assert!(t.output().is_empty());
        tty_force_cursor_colour(&mut t.tty, -1);
        assert_eq!(t.output(), b"R");

        t.tty.mode = MODE_CURSOR;
        assert_eq!(tty_update_cursor(&mut t.tty, 0, None), 0);
        assert_eq!(t.output(), b"I");
        for (style, blinking, marker) in [
            (SCREEN_CURSOR_BLOCK, 0, b"S2".as_slice()),
            (SCREEN_CURSOR_BLOCK, MODE_CURSOR_BLINKING, b"S1"),
            (SCREEN_CURSOR_UNDERLINE, 0, b"S4"),
            (SCREEN_CURSOR_UNDERLINE, MODE_CURSOR_BLINKING, b"S3"),
            (SCREEN_CURSOR_BAR, 0, b"S6"),
            (SCREEN_CURSOR_BAR, MODE_CURSOR_BLINKING, b"S5"),
        ] {
            t.tty.cstyle = style;
            t.tty.mode = 0;
            tty_update_cursor(&mut t.tty, MODE_CURSOR | blinking, None);
            let out = t.output();
            assert!(out.ends_with(marker), "{out:?}");
        }
        t.tty.cstyle = SCREEN_CURSOR_BLOCK;
        t.tty.mode = 0;
        tty_update_cursor(&mut t.tty, MODE_CURSOR, None);
        t.output();
        t.tty.cstyle = SCREEN_CURSOR_DEFAULT;
        t.tty.mode = MODE_CURSOR;
        tty_update_cursor(&mut t.tty, MODE_CURSOR | MODE_CURSOR_VERY_VISIBLE, None);
        assert!(t.output().ends_with(b"V"));
    }
}

#[test]
fn colour_emitters_cover_defaults_bright_and_underline_variants() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    let mut t = Terminal::new(80, 24);
    for (code, value) in [
        (TTYC_SGR0, c"Z"),
        (TTYC_SETAF, c"F%p1%d"),
        (TTYC_SETAB, c"B%p1%d"),
        (TTYC_SETULC1, c"U%p1%d"),
        (TTYC_SETULC, c"X%p1%d"),
        (TTYC_OL, c"O"),
    ] {
        t.set(code, value);
    }
    unsafe {
        t.flag(TTYC_AX, 1);
        let mut gc = grid_default_cell;
        gc.fg = 8;
        gc.bg = 9;
        tty_colours(&mut t.tty, &gc);
        assert_eq!(t.output(), b"\x1b[39m\x1b[49mO");

        gc.fg = 92;
        gc.bg = 94;
        gc.us = 9;
        tty_colours(&mut t.tty, &gc);
        let out = t.output();
        assert!(!out.is_empty());
        assert!(out.contains(&b'O'));

        gc.us = COLOUR_FLAG_256 | 12;
        tty_colours_us(&mut t.tty, &gc);
        assert_eq!(t.output(), b"U12");
        gc.us = COLOUR_FLAG_RGB | 0x010203;
        tty_colours_us(&mut t.tty, &gc);
        assert_eq!(t.output(), b"X66051");
    }
}

#[test]
fn region_and_margin_skip_duplicates_and_restore_full_terminal() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    let mut t = Terminal::new(20, 10);
    t.set(TTYC_CSR, c"R%p1%d,%p2%d");
    t.set(TTYC_CMG, c"M%p1%d,%p2%d");
    t.set(TTYC_CLMG, c"C");
    t.tty.term.as_ref().unwrap().borrow_mut().refresh_derived();
    unsafe {
        tty_region(&mut t.tty, 2, 7);
        tty_region(&mut t.tty, 2, 7);
        assert_eq!(t.output(), b"R2,7");
        tty_region_off(&mut t.tty);
        assert_eq!(t.output(), b"R0,9");
        tty_region_off(&mut t.tty);
        assert!(t.output().is_empty());

        tty_margin(&mut t.tty, 3, 15);
        tty_margin(&mut t.tty, 3, 15);
        assert_eq!(t.output(), b"R0,9M3,15");
        tty_margin_off(&mut t.tty);
        assert_eq!(t.output(), b"R0,9C");
        tty_margin_off(&mut t.tty);
        assert!(t.output().is_empty());
    }
}

#[test]
fn a_cell_command_keeps_its_payload_after_the_source_scope_ends() {
    let _guard = globals();
    let mut terminal = Terminal::new(6, 3);
    let screen = crate::tests::test_fixtures::Screen::new(6, 3, 0);
    let ctx = {
        let cell = crate::tests::test_fixtures::ascii(b'A');
        tty_ctx {
            cell: Some(cell),
            sx: 6,
            sy: 3,
            defaults: grid_default_cell,
            ..Default::default()
        }
    };
    unsafe { tty_cmd_cell(&mut terminal.tty, &ctx, &screen) };
    assert_eq!(terminal.output(), b"A");
}

#[test]
fn cursor_mode_values_remain_usable_after_the_source_screen_is_dropped() {
    if crate::test_process::run() {
        return;
    }
    let _guard = globals();
    let mut terminal = Terminal::new(6, 3);
    terminal.set(TTYC_CNORM, c"N");
    terminal.set(TTYC_SS, c"S%p1%d");
    terminal.set(TTYC_CS, c"C%p1%s");
    let state = {
        let mut screen = crate::tests::test_fixtures::Screen::new(6, 3, 0);
        screen.set_mode(MODE_CURSOR);
        screen.set_cursor_style(SCREEN_CURSOR_DEFAULT);
        screen.set_default_cursor_style(5);
        screen.set_cursor_colour(-1);
        screen.set_default_cursor_colour(COLOUR_FLAG_RGB | 0x123456);
        screen.mode_state()
    };
    unsafe { tty_update_mode(&mut terminal.tty, state.mode, Some(state)) };
    assert_eq!(terminal.output(), b"Crgb:12/34/56NS5");
    assert_eq!(terminal.tty.cstyle, SCREEN_CURSOR_BAR);
    assert_ne!(terminal.tty.mode & MODE_CURSOR_BLINKING, 0);
}

#[test]
fn window_offset_updates_only_terminal_clients_viewing_that_window() {
    use crate::tests::test_fixtures::{Clients, Target};

    let _guard = globals();
    let mut target = Target::new(80, 24);
    let other = target.add_window(1, 80, 24);
    let mut attached = Clients::new();
    unsafe {
        let watching = &mut *attached.add("watching", 20, 6);
        watching.set_attached_session(Some(target.session_handle()));
        watching.flags = CLIENT_TERMINAL as uint64_t;
        watching.tty.osx = 999;
        let detached = &mut *attached.add("detached", 20, 6);
        detached.flags = CLIENT_TERMINAL as uint64_t;
        detached.tty.osx = 999;
        let nonterminal = &mut *attached.add("nonterminal", 20, 6);
        nonterminal.set_attached_session(Some(target.session_handle()));
        nonterminal.flags = 0;
        nonterminal.tty.osx = 999;
        ((*target.winlink(other)).window_handle().unwrap()).update_client_offsets();
        assert_eq!(watching.tty.osx, 999);
        ((*target.winlink(0)).window_handle().unwrap()).update_client_offsets();
        assert_eq!(watching.tty.osx, 20);
        assert_ne!(watching.flags & CLIENT_REDRAWWINDOW as uint64_t, 0);
        assert_eq!(detached.tty.osx, 999);
        assert_eq!(nonterminal.tty.osx, 999);
        (*target.session()).curw = None;
        watching.tty.osx = 999;
        ((*target.winlink(0)).window_handle().unwrap()).update_client_offsets();
        assert_eq!(watching.tty.osx, 999);
    }
}

#[test]
fn default_colours_follow_active_pane_identity_and_per_colour_fallbacks() {
    use crate::tests::test_fixtures::{Pane, Window};

    let _guard = globals();
    let mut first = Pane::new(900, 20, 6, 0);
    let mut second = Pane::new(901, 20, 6, 0);
    let mut window = Window::new(900, "colours", 20, 6);
    window.add_pane(&mut first);
    window.add_pane(&mut second);
    let owner = window.handle().clone();
    unsafe {
        let mut panes = owner.panes();
        for pane in &mut panes {
            let wp = pane.get_mut().unwrap();
            *wp.flags_mut() &= !PANE_STYLECHANGED;
            wp.configure_test(crate::window_pane::PaneTestSetup::Styles(PaneStyleCells {
                cached_gc: grid_cell {
                    fg: 1,
                    bg: 2,
                    ..grid_default_cell
                },
                cached_active_gc: grid_cell {
                    fg: 3,
                    bg: 8,
                    ..grid_default_cell
                },
            }));
        }
        let mut colours = grid_default_cell;
        tty_default_colours(&mut colours, panes[0].get_mut().unwrap());
        assert_eq!((colours.fg, colours.bg), (3, 2));
        tty_default_colours(&mut colours, panes[1].get_mut().unwrap());
        assert_eq!((colours.fg, colours.bg), (1, 2));
        {
            let mut payload = owner.as_window_mut();
            payload.active = payload
                .panes
                .iter()
                .find(|pane| pane.pane_id() == panes[1].id())
                .map(|pane| pane.downgrade());
        }
        panes[1].get_mut().unwrap().configure_test(crate::window_pane::PaneTestSetup::Styles(PaneStyleCells {
            cached_gc: grid_cell {
                fg: 1,
                bg: 2,
                ..grid_default_cell
            },
            cached_active_gc: grid_cell {
                fg: 8,
                bg: 4,
                ..grid_default_cell
            },
        }));
        tty_default_colours(&mut colours, panes[1].get_mut().unwrap());
        assert_eq!((colours.fg, colours.bg), (1, 4));
        tty_default_colours(&mut colours, panes[0].get_mut().unwrap());
        assert_eq!((colours.fg, colours.bg), (1, 2));
        {
            let mut payload = owner.as_window_mut();
            payload.active = payload
                .panes
                .iter()
                .find(|pane| pane.pane_id() == 999)
                .map(|pane| pane.downgrade());
        }
        tty_default_colours(&mut colours, panes[1].get_mut().unwrap());
        assert_eq!((colours.fg, colours.bg), (1, 2));
    }
}

#[test]
fn retained_capability_strings_survive_replacement_and_terminal_drop() {
    let _guard = globals();
    let mut terminal = Terminal::new(80, 24);
    terminal.set(TTYC_BEL, c"original");
    let value = tty_term_string_for(&terminal.tty, TTYC_BEL);
    terminal.set(TTYC_BEL, c"replacement");
    drop(terminal);
    assert_eq!(value, c"original");
}

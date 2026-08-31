//! Characterization tests for [`crate::input`], the terminal byte-stream
//! parser, kept in a file of their own so that parallel efforts to widen
//! coverage stay out of each other's way.
//!
//! A [`Parser`] stands up a server-free window and pane, gives the pane a real
//! input context with [`input_init`] and feeds bytes the way the pty would,
//! through [`input_parse_buffer`]; assertions read the pane's screen — its
//! grid, cursor, mode flags, title and palette. One variant hands the parser a
//! socket-pair buffer event so that replies to device queries land where
//! [`StreamBuffer::written`] can read them.
//!
//! The groups follow the parser's own dispatch tables: C0 controls, ESC
//! sequences, CSI sequences (cursor movement, erasing, insertion and deletion,
//! scroll regions, SGR attributes in every encoding, DECSET/DECRST modes),
//! OSC strings (titles, colours, palette, clipboard, prompt marks) and the
//! UTF-8 collector with its wide, combining and invalid inputs.
//!
//! Everything here asserts what the transpiled code *does* — including the
//! colour encodings it stores in cells and palettes: eight-colour attributes
//! as their raw ANSI numbers (`31` for bright cyan's foreground `91`), indexed
//! colours with [`COLOUR_FLAG_256`] and direct colours with
//! [`COLOUR_FLAG_RGB`].

use crate::types::*;

use crate::alerts::WINDOW_BELL;
use crate::format::{
    MODE_BRACKETPASTE, MODE_INSERT, MODE_KCURSOR, MODE_KKEYPAD, MODE_MOUSE_BUTTON, MODE_MOUSE_SGR,
    MODE_MOUSE_STANDARD, MODE_SYNC, MODE_WRAP,
};
use crate::grid::{
    GRID_ATTR_BRIGHT, GRID_ATTR_CHARSET, GRID_FLAG_PADDING, grid_default_cell, grid_get_cell,
    grid_get_line, grid_string_cells,
};
use crate::input::{
    GRID_LINE_START_OUTPUT, GRID_LINE_START_PROMPT, MODE_FOCUSON, input_init, input_parse_buffer,
    input_pending, input_reset, input_set_buffer_size,
};
use crate::options::options_set_number;
use crate::paste::{paste_buffer_data, paste_free, paste_get_top};
use crate::reactor::Stream;
use crate::screen::screen_grid_mut;
use crate::screen::screen_grid_ptr;
use crate::style::{
    COLOUR_FLAG_256, COLOUR_FLAG_RGB, colour_palette_free, colour_palette_get, colour_palette_init,
    colour_parseX11,
};
use crate::tests::test_fixtures::{Pane, StreamBuffer, Window, ensure_reactor, globals};
use crate::tmux::global_options;
use ::core::ffi::{CStr, c_int};
use ::core::ptr::null_mut;
use ::std::ffi::CString;

/// A window and pane carrying a live input parser over the pane's own base
/// screen. Nothing here touches the server's trees; the optional buffer event
/// exists only so that tests which expect a reply can read one back.
struct Parser {
    window: Window,
    pane: Pane,
    ictx: *mut crate::input::input_ctx,
    _bev: Option<StreamBuffer>,
    _globals: ::std::sync::MutexGuard<'static, ()>,
}

impl Parser {
    /// An 80x24 pane whose replies go nowhere.
    fn new() -> Parser {
        Parser::with_replies(false)
    }

    /// A pane whose replies are captured on a socket pair.
    fn answering() -> Parser {
        Parser::with_replies(true)
    }

    fn with_replies(replies: bool) -> Parser {
        let globals = globals();
        ensure_reactor();
        let bev = if replies {
            Some(StreamBuffer::new())
        } else {
            None
        };
        let mut window = Window::new(1, "input", 80, 24);
        let mut pane = Pane::new(0, 80, 24, 100);
        window.add_pane(&mut pane);
        let wp = pane.ptr();
        let ictx = unsafe {
            colour_palette_init(&mut (*wp).palette);
            (*wp).ictx = Some(input_init(
                crate::input::InputOwner::Pane((*wp).id),
                bev.as_ref().map_or(Stream::NONE, |b| b.ptr()),
            ));
            crate::input::ictx_opt(&(*wp).ictx).unwrap_or(null_mut())
        };
        Parser {
            window,
            pane,
            ictx,
            _bev: bev,
            _globals: globals,
        }
    }

    fn wp(&mut self) -> *mut window_pane {
        self.pane.ptr()
    }

    fn s(&mut self) -> *mut screen {
        self.pane.screen()
    }

    fn feed(&mut self, seq: &[u8]) {
        unsafe { input_parse_buffer(self.wp(), seq.as_ptr(), seq.len() as size_t) };
    }

    fn feed_str(&mut self, seq: &str) {
        self.feed(seq.as_bytes());
    }

    fn cursor(&mut self) -> (u_int, u_int) {
        unsafe { ((*self.s()).cx, (*self.s()).cy) }
    }

    fn mode(&mut self) -> c_int {
        unsafe { (*self.s()).mode }
    }

    fn title(&mut self) -> String {
        unsafe {
            CStr::from_ptr((*self.s()).title_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }

    fn lines(&mut self) -> Vec<String> {
        let gd = unsafe { screen_grid_ptr(&mut *self.s()) };
        unsafe {
            (0..(*gd).sy)
                .map(|y| {
                    let p =
                        grid_string_cells(&*gd, 0, (*gd).hsize + y, (*gd).sx, None, 0, null_mut());
                    p.to_string_lossy().trim_end().to_string()
                })
                .collect()
        }
    }

    fn cell(&mut self, px: u_int, py: u_int) -> grid_cell {
        let gd = unsafe { screen_grid_ptr(&mut *self.s()) };
        let mut gc = unsafe { grid_default_cell };
        unsafe { gc = grid_get_cell(&*gd, px, py) };
        gc
    }

    fn text_at(&mut self, px: u_int, py: u_int) -> u8 {
        self.cell(px, py).data.data[0]
    }

    /// What has been written to the buffer event since the last ask.
    fn replies(&mut self) -> Vec<u8> {
        self._bev
            .as_ref()
            .expect("a reply-capturing parser")
            .written()
    }
}

impl Drop for Parser {
    fn drop(&mut self) {
        unsafe {
            if let Some(ictx) = (*self.wp()).ictx.take() {
                crate::input::input_free_box(ictx);
            }
            colour_palette_free(Some(&mut (*self.wp()).palette));
        }
    }
}

#[test]
fn printable_text_lands_on_the_grid() {
    let mut p = Parser::new();
    p.feed_str("hello");
    assert_eq!(p.lines()[0], "hello");
    assert_eq!(p.cursor(), (5, 0));
}

#[test]
fn printing_past_the_right_edge_wraps_to_the_next_line() {
    let mut p = Parser::new();
    p.feed_str(&"0123456789".repeat(8));
    assert_eq!(p.lines()[0], "0123456789".repeat(8));
    p.feed_str("abc");
    assert_eq!(p.lines()[1], "abc");
}

#[test]
fn carriage_return_linefeed_and_backspace_move_the_cursor() {
    let mut p = Parser::new();
    p.feed_str("abc\rx");
    assert_eq!(p.lines()[0], "xbc");
    assert_eq!(p.cursor(), (1, 0));
    p.feed_str("\n");
    assert_eq!(p.cursor(), (1, 1));
    p.feed_str("\x08");
    assert_eq!(p.cursor(), (0, 1));
}

#[test]
fn tab_jumps_to_the_next_eight_column_stop() {
    let mut p = Parser::new();
    p.feed_str("\tX\tY");
    assert_eq!(p.cursor(), (17, 0));
    assert_eq!(p.text_at(8, 0), b'X');
    assert_eq!(p.text_at(16, 0), b'Y');
}

#[test]
fn tab_over_blank_space_writes_a_single_tab_cell() {
    let mut p = Parser::new();
    p.feed_str("ab\tc");
    assert_eq!(p.text_at(8, 0), b'c');
    assert_eq!(p.cursor(), (9, 0));
}

#[test]
fn bell_sets_the_window_alert_flag() {
    let mut p = Parser::new();
    unsafe { options_set_number((*p.window.ptr()).options_ptr(), c"monitor-bell".as_ptr(), 0) };
    p.feed_str("\x07");
    assert_ne!(unsafe { (*p.window.ptr()).flags } & WINDOW_BELL, 0);
}

#[test]
fn linefeeds_eventually_scroll_lines_into_history() {
    let mut p = Parser::new();
    for i in 0..23 {
        p.feed_str(&format!("{i}\r\n"));
    }
    assert_eq!(unsafe { (*screen_grid_ptr(&mut *p.s())).hsize }, 0);
    p.feed_str("23\n");
    assert_eq!(unsafe { (*screen_grid_ptr(&mut *p.s())).hsize }, 1);
    assert_eq!(p.lines()[0], "1");
    assert_eq!(p.lines()[21], "22");
    assert_eq!(p.lines()[22], "23");
    assert_eq!(p.lines()[23], "");
}

#[test]
fn shift_in_selects_graphics_and_shift_out_restores_ascii() {
    let mut p = Parser::new();
    p.feed_str("\x1b)0\x0eq\x0fq");
    let gc = p.cell(0, 0);
    assert_eq!(gc.data.data[0], b'q');
    assert_ne!(gc.attr as c_int & GRID_ATTR_CHARSET, 0);
    let gc2 = p.cell(1, 0);
    assert_eq!(gc2.data.data[0], b'q');
    assert_eq!(gc2.attr as c_int & GRID_ATTR_CHARSET, 0);
}

#[test]
fn save_and_restore_cursor_keeps_attributes_and_position() {
    let mut p = Parser::new();
    p.feed_str("\x1b[1;32mok\x1b7");
    p.feed_str("\x1b[0m\x1b[10;1H");
    p.feed_str("\x1b8Z");
    assert_eq!(p.cursor(), (3, 0));
    let gc = p.cell(2, 0);
    assert_ne!(gc.attr as c_int & GRID_ATTR_BRIGHT, 0);
    assert_eq!(gc.fg, 2);
}

#[test]
fn full_reset_empties_the_screen_and_the_palette() {
    let mut p = Parser::new();
    let red = unsafe { colour_parseX11(c"red".as_ptr()) };
    p.feed_str("junk\x1b]10;red\x07");
    assert_eq!(unsafe { (*p.wp()).palette.fg }, red);
    p.feed_str("\x1bc");
    assert_eq!(p.lines()[0], "");
    assert_eq!(p.cursor(), (0, 0));
    assert_eq!(unsafe { (*p.wp()).palette.fg }, 8);
}

#[test]
fn index_nel_and_reverse_index_move_within_the_screen() {
    let mut p = Parser::new();
    p.feed_str("a\r\nb");
    p.feed_str("\x1bDa");
    assert_eq!(p.lines()[2], " a");
    p.feed_str("\x1bEb");
    assert_eq!(p.lines()[3], "b");
    assert_eq!(p.cursor(), (1, 3));
    p.feed_str("\x1bM\x1bM");
    assert_eq!(p.cursor(), (1, 1));
}

#[test]
fn reverse_index_from_the_top_scrolls_the_screen_down() {
    let mut p = Parser::new();
    p.feed_str("top\r\nbottom");
    p.feed_str("\x1b[1;1H\x1bM");
    assert_eq!(p.lines()[0], "");
    assert_eq!(p.lines()[1], "top");
    assert_eq!(p.lines()[2], "bottom");
}

#[test]
fn horizontal_tab_set_and_clear_change_where_tab_stops() {
    let mut p = Parser::new();
    p.feed_str("\x1b[10G\x1bH\x1b[9G\x1b[g\x1b[H\tX");
    assert_eq!(p.text_at(9, 0), b'X');
    assert_eq!(p.cursor(), (10, 0));
}

#[test]
fn keypad_application_mode_is_a_screen_flag() {
    let mut p = Parser::new();
    p.feed_str("\x1b=");
    assert_ne!(p.mode() & MODE_KKEYPAD, 0);
    p.feed_str("\x1b>");
    assert_eq!(p.mode() & MODE_KKEYPAD, 0);
}

#[test]
fn alignment_test_fills_the_screen_with_e() {
    let mut p = Parser::new();
    p.feed_str("\x1b#8");
    for line in p.lines() {
        assert_eq!(line, "E".repeat(80));
    }
}

#[test]
fn cup_moves_to_row_and_column_counting_from_one() {
    let mut p = Parser::new();
    p.feed_str("\x1b[4;10HX");
    assert_eq!(p.text_at(9, 3), b'X');
    assert_eq!(p.cursor(), (10, 3));
    p.feed_str("\x1b[H");
    assert_eq!(p.cursor(), (0, 0));
}

#[test]
fn relative_cursor_movements_clamp_at_the_edges() {
    let mut p = Parser::new();
    p.feed_str("\x1b[3;5H");
    p.feed_str("\x1b[A\x1b[B\x1b[C\x1b[D");
    assert_eq!(p.cursor(), (4, 2));
    p.feed_str("\x1b[99A\x1b[99D");
    assert_eq!(p.cursor(), (0, 0));
    p.feed_str("\x1b[99B\x1b[99C");
    assert_eq!(p.cursor(), (79, 23));
}

#[test]
fn column_and_row_direct_addressing_use_one_parameter_each() {
    let mut p = Parser::new();
    p.feed_str("\x1b[20G\x1b[5dX");
    assert_eq!(p.text_at(19, 4), b'X');
}

#[test]
fn next_and_previous_line_combine_vertical_motion_with_carriage_return() {
    let mut p = Parser::new();
    p.feed_str("\x1b[1;10Ha\x1b[2E");
    assert_eq!(p.cursor(), (0, 2));
    p.feed_str("\x1b[1F");
    assert_eq!(p.cursor(), (0, 1));
}

#[test]
fn erase_display_and_erase_line_cover_every_variant() {
    let mut p = Parser::new();
    p.feed_str("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    p.feed_str("\x1b[1;15H");
    p.feed_str("\x1b[K");
    assert_eq!(p.lines()[0], "aaaaaaaaaaaaaa");
    p.feed_str("\x1b[1K");
    assert_eq!(p.lines()[0], "");
    p.feed_str("\x1b[2J\x1b[H");
    p.feed_str("zzzz");
    p.feed_str("\x1b[J");
    assert_eq!(p.lines()[0], "zzzz");
    p.feed_str("\x1b[3J");
    assert_eq!(unsafe { (*screen_grid_ptr(&mut *p.s())).hsize }, 0);
}

#[test]
fn insert_delete_and_erase_characters_edit_the_current_line() {
    let mut p = Parser::new();
    p.feed_str("abcdef\x1b[1;2H");
    p.feed_str("\x1b[1@X");
    assert_eq!(p.lines()[0], "aXbcdef");
    p.feed_str("\x1b[2P");
    assert_eq!(p.lines()[0], "aXdef");
    p.feed_str("\x1b[1;2H\x1b[3X");
    assert_eq!(p.lines()[0], "a   f");
}

#[test]
fn insert_and_delete_lines_work_inside_the_whole_screen() {
    let mut p = Parser::new();
    p.feed_str("one\r\ntwo\r\nthree");
    p.feed_str("\x1b[2;1H\x1b[1L");
    assert_eq!(p.lines()[1], "");
    assert_eq!(p.lines()[2], "two");
    p.feed_str("\x1b[1M");
    assert_eq!(p.lines()[1], "two");
    assert_eq!(p.lines()[2], "three");
}

#[test]
fn repeat_the_previous_character_n_times() {
    let mut p = Parser::new();
    p.feed_str("a\x1b[3b");
    assert_eq!(p.lines()[0], "aaaa");
}

#[test]
fn scroll_region_limits_where_scrolling_happens() {
    let mut p = Parser::new();
    p.feed_str("top\r\nsecond\r\nthird\r\nfourth");
    p.feed_str("\x1b[2;3r");
    assert_eq!(unsafe { (*p.s()).rupper }, 1);
    assert_eq!(unsafe { (*p.s()).rlower }, 2);
    p.feed_str("\x1b[3;1H\n");
    assert_eq!(p.lines()[1], "third");
    assert_eq!(p.lines()[2], "");
    assert_eq!(p.lines()[3], "fourth");
    p.feed_str("\x1b[r");
    assert_eq!(unsafe { (*p.s()).rupper }, 0);
    assert_eq!(unsafe { (*p.s()).rlower }, 23);
}

#[test]
fn scroll_up_and_down_shift_lines_on_the_screen() {
    let mut p = Parser::new();
    p.feed_str("aaa\r\nbbb\r\nccc");
    p.feed_str("\x1b[2S");
    assert_eq!(p.lines()[0], "ccc");
    p.feed_str("\x1b[T\x1b[T");
    assert_eq!(p.lines()[2], "ccc");
    assert_eq!(p.lines()[1], "");
}

#[test]
fn sgr_attributes_reach_the_cells_they_are_printed_with() {
    let mut p = Parser::new();
    p.feed_str("\x1b[1mb\x1b[22m\x1b[4mu\x1b[24m\x1b[7mr\x1b[27m\x1b[9ms\x1b[29m");
    let attrs: Vec<c_int> = (0..4).map(|i| p.cell(i, 0).attr as c_int).collect();
    assert_ne!(attrs[0] & GRID_ATTR_BRIGHT, 0);
    assert_eq!(attrs[1] & GRID_ATTR_BRIGHT, 0);
    assert_ne!(attrs[1] & 0x4, 0);
    assert_ne!(attrs[2] & 0x10, 0);
    assert_eq!(attrs[2] & 0x4, 0);
    assert_ne!(attrs[3] & 0x100, 0);
}

#[test]
fn sgr_colours_store_their_raw_ansi_numbers_for_bright_shades() {
    let mut p = Parser::new();
    p.feed_str("\x1b[31;44mc\x1b[39;49m\x1b[91;101md\x1b[me");
    let c = p.cell(0, 0);
    assert_eq!(c.fg, 1);
    assert_eq!(c.bg, 4);
    let d = p.cell(1, 0);
    assert_eq!(d.fg, 91);
    assert_eq!(d.bg, 91);
    let e = p.cell(2, 0);
    assert_eq!(e.fg, 8);
    assert_eq!(e.bg, 8);
}

#[test]
fn sgr_256_and_rgb_colours_arrive_in_both_semicolon_and_colon_forms() {
    let rgb = (10 << 16) | (20 << 8) | 30;
    let colon_rgb = (1 << 16) | (2 << 8) | 3;
    let mut p = Parser::new();
    p.feed_str("\x1b[38;5;200ma\x1b[0m");
    assert_eq!(p.cell(0, 0).fg, COLOUR_FLAG_256 | 200);
    p.feed_str("\x1b[38:5:100mb\x1b[0m");
    assert_eq!(p.cell(1, 0).fg, COLOUR_FLAG_256 | 100);
    p.feed_str("\x1b[48;2;10;20;30mc\x1b[0m");
    assert_eq!(p.cell(2, 0).bg, COLOUR_FLAG_RGB | rgb);
    p.feed_str("\x1b[48:2::1:2:3md\x1b[0m");
    assert_eq!(p.cell(3, 0).bg, COLOUR_FLAG_RGB | colon_rgb);
    p.feed_str("\x1b[58:5:40me\x1b[59mf\x1b[0m");
    assert_eq!(p.cell(4, 0).us, COLOUR_FLAG_256 | 40);
    assert_eq!(p.cell(5, 0).us, 8);
}

#[test]
fn device_attributes_reply_with_the_version_strings() {
    let mut p = Parser::answering();
    p.feed_str("\x1b[c");
    assert_eq!(p.replies(), b"\x1b[?1;2c");
    p.feed_str("\x1b[>0c");
    assert_eq!(p.replies(), b"\x1b[>84;0;0c");
}

#[test]
fn cursor_position_report_gives_row_and_column() {
    let mut p = Parser::answering();
    p.feed_str("\x1b[4;7H\x1b[6n");
    assert_eq!(p.replies(), b"\x1b[4;7R");
}

#[test]
fn dec_private_modes_set_and_clear_screen_flags() {
    let mut p = Parser::new();
    p.feed_str("\x1b[?2004;1004;1000;1006;1h");
    let m = p.mode();
    assert_ne!(m & MODE_BRACKETPASTE, 0);
    assert_ne!(m & MODE_FOCUSON, 0);
    assert_ne!(m & MODE_MOUSE_STANDARD, 0);
    assert_ne!(m & MODE_MOUSE_SGR, 0);
    assert_ne!(m & MODE_KCURSOR, 0);
    p.feed_str("\x1b[?2004;1004;1000;1006;1l");
    let m = p.mode();
    assert_eq!(
        m & (MODE_BRACKETPASTE
            | MODE_FOCUSON
            | MODE_MOUSE_STANDARD
            | MODE_MOUSE_SGR
            | MODE_KCURSOR),
        0
    );
    p.feed_str("\x1b[4h\x1b[34l");
    assert_ne!(p.mode() & MODE_INSERT, 0);
    p.feed_str("\x1b[?1002h");
    assert_ne!(p.mode() & MODE_MOUSE_BUTTON, 0);
}

#[test]
fn wrap_mode_off_pins_the_cursor_at_the_right_edge() {
    let mut p = Parser::new();
    p.feed_str("\x1b[?7l");
    assert_eq!(p.mode() & MODE_WRAP, 0);
    p.feed_str("\x1b[1;79Hab");
    assert_eq!(p.cursor(), (79, 0));
    assert_eq!(p.lines()[1], "");
    p.feed_str("\x1b[?7h");
    assert_ne!(p.mode() & MODE_WRAP, 0);
}

#[test]
fn origin_mode_addresses_rows_relative_to_the_scroll_region() {
    let mut p = Parser::new();
    p.feed_str("\x1b[5;10r\x1b[?6h\x1b[H");
    assert_eq!(p.cursor(), (0, 4));
    p.feed_str("X\x1b[?6l\x1b[HY");
    assert_eq!(p.text_at(0, 4), b'X');
    assert_eq!(p.text_at(0, 0), b'Y');
}

#[test]
fn alternate_screen_saves_and_restores_the_primary_grid() {
    let mut p = Parser::new();
    p.feed_str("primary");
    p.feed_str("\x1b[?1049h\x1b[H");
    assert_eq!(p.lines()[0], "");
    p.feed_str("alt");
    assert_eq!(p.lines()[0], "alt");
    p.feed_str("\x1b[?1049l");
    assert_eq!(p.lines()[0], "primary");
}

#[test]
fn sync_updates_track_the_screen_mode_flag() {
    let mut p = Parser::new();
    p.feed_str("\x1b[?2026h");
    assert_ne!(p.mode() & MODE_SYNC, 0);
    p.feed_str("\x1b[?2026l");
    assert_eq!(p.mode() & MODE_SYNC, 0);
}

#[test]
fn osc_titles_accept_both_bel_and_string_terminators() {
    let mut p = Parser::new();
    p.feed_str("\x1b]0;bel title\x07");
    assert_eq!(p.title(), "bel title");
    p.feed_str("\x1b]2;st title\x1b\\");
    assert_eq!(p.title(), "st title");
}

#[test]
fn osc_colour_queries_report_black_with_no_client_attached() {
    let mut p = Parser::answering();
    p.feed_str("\x1b]10;?\x1b\\");
    assert_eq!(p.replies(), b"\x1b]10;rgb:0000/0000/0000\x1b\\");
    p.feed_str("\x1b]11;?\x1b\\");
    assert_eq!(p.replies(), b"\x1b]11;rgb:0000/0000/0000\x1b\\");
}

#[test]
fn osc_colour_settings_land_in_the_pane_palette() {
    let mut p = Parser::new();
    let red = unsafe { colour_parseX11(c"red".as_ptr()) };
    let blue = unsafe { colour_parseX11(c"blue".as_ptr()) };
    p.feed_str("\x1b]10;red\x07\x1b]11;blue\x07");
    assert_eq!(unsafe { (*p.wp()).palette.fg }, red);
    assert_eq!(unsafe { (*p.wp()).palette.bg }, blue);
    p.feed_str("\x1b]110;\x07\x1b]111;\x07");
    assert_eq!(unsafe { (*p.wp()).palette.fg }, 8);
    assert_eq!(unsafe { (*p.wp()).palette.bg }, 8);
}

#[test]
fn osc_palette_entries_are_settable_queryable_and_resettable() {
    let mut p = Parser::answering();
    let red = unsafe { colour_parseX11(c"red".as_ptr()) };
    p.feed_str("\x1b]4;1;red\x07");
    assert_eq!(
        unsafe { colour_palette_get(Some(&(*p.wp()).palette), COLOUR_FLAG_256 | 1) },
        red
    );
    p.feed_str("\x1b]4;1;?\x1b\\");
    assert_eq!(p.replies(), b"\x1b]4;1;rgb:ffff/0000/0000\x1b\\");
    p.feed_str("\x1b]4;2;?\x1b\\");
    assert_eq!(p.replies(), b"");
    p.feed_str("\x1b]104;1\x07");
    assert_eq!(
        unsafe { colour_palette_get(Some(&(*p.wp()).palette), COLOUR_FLAG_256 | 1) },
        -1
    );
}

#[test]
fn osc_prompt_marks_tag_the_lines_they_arrive_on() {
    let mut p = Parser::new();
    p.feed_str("$ \x1b]133;A\x07output\x1b]133;C\x07");
    let flags = unsafe { grid_get_line(screen_grid_mut(&mut *p.s()), 0).flags };
    assert_ne!(flags & GRID_LINE_START_PROMPT, 0);
    assert_ne!(flags & GRID_LINE_START_OUTPUT, 0);
}

#[test]
fn osc_clipboard_write_stores_a_paste_buffer_when_allowed() {
    let mut p = Parser::new();
    unsafe { options_set_number(global_options, c"set-clipboard".as_ptr(), 2) };
    p.feed_str("\x1b]52;c;aGVsbG8=\x07");
    let mut name: Option<CString> = None;
    let top = unsafe { paste_get_top(Some(&mut name)) };
    assert!(!top.is_null());
    unsafe {
        assert_eq!(paste_buffer_data(&*top), b"hello");
        paste_free(top);
    };
}

#[test]
fn an_osc_52_query_is_refused_while_set_clipboard_is_off() {
    let mut p = Parser::answering();
    unsafe { options_set_number(global_options, c"set-clipboard".as_ptr(), 0) };
    p.feed_str("\x1b]52;c;?\x07");
    assert_eq!(p.replies(), b"");
}

#[test]
fn utf8_text_prints_accented_wide_and_invalid_bytes_predictably() {
    let mut p = Parser::new();
    p.feed(b"caf\xe9\x08");
    assert_eq!(p.lines()[0], "caf\u{fffd}".to_string());
    assert_eq!(p.cursor(), (3, 0));
    p.feed(b"\xe6\xbc\xa2");
    assert_eq!(p.cursor(), (5, 0));
    let pad = p.cell(4, 0);
    assert_ne!(pad.flags as c_int & GRID_FLAG_PADDING, 0);
    p.feed(b"e\xcc\x81");
    let gc = p.cell(5, 0);
    assert_eq!(gc.data.have, 3);
    assert_eq!(gc.data.width, 1);
    assert_eq!(&gc.data.data[..3], b"e\xcc\x81");
}

#[test]
fn lone_continuation_bytes_become_replacement_characters() {
    let mut p = Parser::new();
    p.feed(b"a\x84b\x9bz");
    assert_eq!(p.lines()[0], "a\u{fffd}b\u{fffd}z".to_string());
}

#[test]
fn control_bytes_interrupt_a_partial_utf8_sequence() {
    let mut p = Parser::new();
    p.feed(b"a\xe6\x08b");
    assert_eq!(p.lines()[0], "ab");
}

#[test]
fn an_unterminated_sequence_is_held_until_it_completes() {
    let mut p = Parser::new();
    p.feed_str("\x1b[");
    assert_eq!(unsafe { (*input_pending(&mut *p.ictx)).len() }, 2);
    p.feed_str("4mx");
    assert_eq!(unsafe { (*input_pending(&mut *p.ictx)).len() }, 0);
    assert_eq!(p.lines()[0], "x");
}

#[test]
fn resetting_the_parser_clears_the_screen_when_asked() {
    let mut p = Parser::new();
    p.feed_str("junk");
    unsafe { input_reset(&mut *p.ictx, 1) };
    assert_eq!(p.lines()[0], "");
    assert_eq!(p.cursor(), (0, 0));
}

#[test]
fn the_clipboard_size_limit_is_settable_without_observable_state() {
    input_set_buffer_size(1024);
}

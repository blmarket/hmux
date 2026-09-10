use super::*;
use crate::text::utf8_to_data;

const PRINT_SIZE: usize = 16384;
use crate::grid::{
    grid_default_cell, grid_scroll_history, grid_set_cell, grid_set_cells, grid_set_padding,
    grid_set_tab, grid_string_cells,
};
use crate::options::OptionsRef;

use crate::tests::test_fixtures::globals;
use crate::tmux::global_w_options;
use ::core::ffi::c_int;

impl RustScreen {
    pub(crate) fn clear_path(&mut self) {
        self.0.path = None;
    }

    pub(crate) fn write_list(&self) -> &[screen_write_cline] {
        &self.0.write_list
    }

    fn title_text(&self) -> String {
        self.0
            .title
            .as_deref()
            .expect("screen has a title")
            .to_string_lossy()
            .into_owned()
    }

    /// Writes `s` from (px, py) of the screen, one cell per byte.
    fn write(&mut self, px: u_int, py: u_int, text: &str) {
        let mut gc = { grid_default_cell };
        let py = self.grid().hsize + py;
        for (i, byte) in text.bytes().enumerate() {
            gc.data.data[0] = byte;
            gc.data.have = 1;
            gc.data.size = 1;
            gc.data.width = 1;
            grid_set_cell(self.grid_mut(), px + i as u_int, py, &gc);
        }
    }

    /// The text of one line of the whole grid, history included.
    fn text(&self, py: u_int) -> String {
        {
            let p = grid_string_cells(self.grid(), 0, py, 1000, None, 0, None);

            p.to_string_lossy().into_owned()
        }
    }

    /// Whether there is a tab stop at each column of the screen.
    fn tabs(&self) -> Vec<bool> {
        (0..self.grid().sx)
            .map(|i| self.0.tabs[(i >> 3) as usize] as c_int & (1 << (i & 0x7)) != 0)
            .collect()
    }
}

#[test]
fn a_new_screen_has_a_grid_and_nothing_else() {
    let _guard = globals();
    let s = RustScreen::new_with_server_options(10, 5, 100);
    assert_eq!(s.grid().sx, 10);
    assert_eq!(s.grid().sy, 5);
    assert_eq!(s.grid().hlimit, 100);
    assert_eq!(s.title_text(), "");
    assert_eq!(s.0.path, None);
    assert!(s.0.titles.is_none());
    assert!(s.0.saved_grid.is_none());
    assert!(s.0.sel.is_none());
    assert!(s.0.write_list.is_empty());
    assert!(s.0.hyperlinks.is_some());
    assert_eq!(s.0.cstyle, SCREEN_CURSOR_DEFAULT);
    assert_eq!(s.0.default_cstyle, SCREEN_CURSOR_DEFAULT);
    assert_eq!(s.0.ccolour, -1);
    assert_eq!(s.0.default_ccolour, -1);
    assert_eq!(s.0.default_mode, 0);
    assert_eq!(s.0.mode, MODE_CURSOR | MODE_WRAP);
    assert_eq!((s.0.cx, s.0.cy), (0, 0));
    assert_eq!((s.0.rupper, s.0.rlower), (0, 4));
    assert_eq!((s.0.saved_cx, s.0.saved_cy), (UINT_MAX, UINT_MAX));
}

#[test]
fn a_screen_is_reset_to_what_it_started_as() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 100);
    s.write(0, 0, "abc");
    s.0.cx = 3;
    s.0.cy = 2;
    s.0.mode = MODE_CRLF | MODE_INSERT;
    unsafe {
        s.push_title();
        let gc = grid_default_cell;
        s.set_selection(0, 0, 1, 1, false, 0, 0, &gc);
        s.set_progress_bar(PROGRESS_BAR_NORMAL, 50);
        screen_reinit(&mut s);
    }
    assert_eq!((s.0.cx, s.0.cy), (0, 0));
    assert_eq!(
        s.0.mode,
        MODE_CURSOR | MODE_WRAP | MODE_CRLF,
        "only the newline mode survives"
    );
    assert_eq!(s.text(0), "");
    assert!(s.0.sel.is_none());
    assert!(s.0.titles.is_none());
    assert_eq!(s.0.progress_bar.state, PROGRESS_BAR_HIDDEN);
    assert_eq!(s.0.progress_bar.progress, 0);
}

#[test]
fn a_reset_turns_extended_keys_on_when_the_option_asks_for_it() {
    let _guard = globals();
    unsafe {
        (global_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .set_number(c"extended-keys", 2)
    };
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    assert_eq!(s.mode() & MODE_KEYS_EXTENDED, MODE_KEYS_EXTENDED);
    let mut standalone = RustScreen::new(10, 5, 0);
    assert_eq!(standalone.mode() & MODE_KEYS_EXTENDED, 0);
    standalone.reinit();
    assert_eq!(standalone.mode() & MODE_KEYS_EXTENDED, 0);
    s.set_mode(0);
    unsafe { screen_reinit(&mut s) };
    assert_eq!(s.mode() & MODE_KEYS_EXTENDED, MODE_KEYS_EXTENDED);
    unsafe {
        (global_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .set_number(c"extended-keys", 0)
    };
    let plain = RustScreen::new_with_server_options(10, 5, 0);
    assert_eq!(plain.0.mode & MODE_KEYS_EXTENDED, 0);
}

#[test]
fn a_reset_comes_out_of_the_alternate_screen() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    s.write(0, 0, "main");
    let gc = { grid_default_cell };
    unsafe {
        screen_alternate_on(&mut s, &gc, 1);
        screen_reinit(&mut s);
    }
    assert!(s.0.saved_grid.is_none());
    assert_eq!(s.text(0), "", "the reset cleared the restored screen");
}

#[test]
fn the_hyperlinks_of_a_screen_are_made_once_and_then_emptied() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    let first = s.hyperlinks();
    let inner = first.put(c"https://example.com/", Some(c"id"));
    s.reset_hyperlinks();
    assert!(first.get(inner).is_none(), "the same table was emptied");
}

#[test]
fn tab_stops_are_every_eight_columns() {
    let _guard = globals();
    let s = RustScreen::new_with_server_options(20, 5, 0);
    let tabs = s.tabs();
    for (i, stop) in tabs.iter().enumerate() {
        assert_eq!(*stop, i != 0 && i % 8 == 0, "column {i}");
    }
}

#[test]
fn the_default_cursor_comes_from_the_options() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    unsafe {
        (global_w_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .set_number(c"cursor-colour", 4);
        (global_w_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .set_number(c"cursor-style", 3);
        s.set_default_cursor(
            global_w_options
                .get()
                .as_ref()
                .expect("global options are initialized"),
        );
    }
    assert_eq!(s.0.default_ccolour, 4);
    assert_eq!(s.0.default_cstyle, SCREEN_CURSOR_UNDERLINE);
    assert_eq!(s.0.default_mode, MODE_CURSOR_BLINKING);
    unsafe {
        (global_w_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .set_number(c"cursor-colour", -1);
        (global_w_options
            .get()
            .as_ref()
            .expect("global options are initialized"))
        .set_number(c"cursor-style", 0);
    }
}

#[test]
fn each_cursor_style_is_a_shape_and_a_blink() {
    let mut cstyle = SCREEN_CURSOR_BLOCK;
    let mut mode = MODE_CURSOR_BLINKING;
    let mut set = |style: u_int| {
        screen_set_cursor_style(style, &mut cstyle, &mut mode);
        (cstyle, mode & MODE_CURSOR_BLINKING != 0)
    };
    assert_eq!(set(1), (SCREEN_CURSOR_BLOCK, true));
    assert_eq!(set(2), (SCREEN_CURSOR_BLOCK, false));
    assert_eq!(set(3), (SCREEN_CURSOR_UNDERLINE, true));
    assert_eq!(set(4), (SCREEN_CURSOR_UNDERLINE, false));
    assert_eq!(set(5), (SCREEN_CURSOR_BAR, true));
    assert_eq!(set(6), (SCREEN_CURSOR_BAR, false));
    assert_eq!(set(0), (SCREEN_CURSOR_DEFAULT, false));
    assert_eq!(set(7), (SCREEN_CURSOR_DEFAULT, false), "nothing changes");
}

#[test]
fn the_cursor_colour_is_kept_as_it_is_given() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    s.set_cursor_colour(42);
    assert_eq!(s.0.ccolour, 42);
}

#[test]
fn a_title_and_a_path_are_cleaned_before_they_are_kept() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    assert_eq!(s.set_title(c"hello", 0), 1);
    assert_eq!(s.title_text(), "hello");
    assert_eq!(s.set_title(c"a#(b", 1), 1);
    assert_eq!(s.title_text(), "a_(b", "an untrusted format is defused");
    assert_eq!(
        s.set_title(c"\xc3\x28", 0),
        0,
        "invalid UTF-8 is turned down"
    );
    assert_eq!(s.title_text(), "a_(b");

    assert_eq!(s.set_path(c"/tmp", 0), 1);
    assert_eq!(s.0.path.as_deref().unwrap().to_str().unwrap(), "/tmp");
    assert_eq!(s.set_path(c"\xc3\x28", 0), 0);
}

#[test]
fn titles_are_pushed_and_popped_as_a_stack() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    s.set_title(c"one", 0);
    s.push_title();
    s.set_title(c"two", 0);
    s.push_title();
    s.set_title(c"three", 0);
    assert_eq!(s.0.ntitles, 2);

    s.pop_title();
    assert_eq!(s.title_text(), "two");
    assert_eq!(s.0.ntitles, 1);
    s.pop_title();
    assert_eq!(s.title_text(), "one");
    assert_eq!(s.0.ntitles, 0);
    s.pop_title();
    assert_eq!(
        s.title_text(),
        "one",
        "an empty stack leaves the title alone"
    );
}

#[test]
fn popping_a_title_that_was_never_pushed_does_nothing() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    {
        s.set_title(c"one", 0);
        s.pop_title();
    }
    assert_eq!(s.title_text(), "one");
    assert!(s.0.titles.is_none());
}

#[test]
fn the_title_stack_holds_ten() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    for i in 0..12 {
        let title = CString::new(format!("t{i}")).unwrap();
        {
            s.set_title(&title, 0);
            s.push_title();
        }
    }
    assert_eq!(s.0.ntitles, 10);
    for i in (2..12).rev() {
        s.pop_title();
        assert_eq!(
            s.title_text(),
            format!("t{i}"),
            "the oldest two were dropped"
        );
    }
    assert_eq!(s.0.ntitles, 0);
}

#[test]
fn a_progress_bar_keeps_its_progress_unless_it_has_none() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    s.set_progress_bar(PROGRESS_BAR_NORMAL, 30);
    assert_eq!(s.0.progress_bar.state, PROGRESS_BAR_NORMAL);
    assert_eq!(s.0.progress_bar.progress, 30);
    s.set_progress_bar(PROGRESS_BAR_ERROR, -1);
    assert_eq!(s.0.progress_bar.state, PROGRESS_BAR_ERROR);
    assert_eq!(
        s.0.progress_bar.progress, 30,
        "a negative progress is no news"
    );
    s.set_progress_bar(PROGRESS_BAR_INDETERMINATE, 70);
    assert_eq!(
        s.0.progress_bar.progress, 30,
        "an indeterminate bar has no progress to set"
    );
}

#[test]
fn a_screen_can_be_made_wider_and_narrower() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    s.write(0, 0, "abcdefghij");
    screen_resize(&mut s, 20, 5, 0);
    assert_eq!(s.grid().sx, 20);
    assert_eq!(s.tabs().len(), 20);
    assert_eq!(s.text(0), "abcdefghij");

    screen_resize(&mut s, 20, 5, 0);
    assert_eq!(s.grid().sx, 20, "the same width is no change");
}

#[test]
fn a_screen_is_never_smaller_than_one_cell() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    screen_resize(&mut s, 0, 0, 0);
    assert_eq!((s.grid().sx, s.grid().sy), (1, 1));
}

#[test]
fn a_taller_screen_gets_empty_lines_at_the_bottom() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 2, 0);
    s.write(0, 0, "one");
    s.write(0, 1, "two");
    screen_resize(&mut s, 10, 4, 0);
    assert_eq!(s.grid().sy, 4);
    assert_eq!(s.0.rlower, 3);
    assert_eq!([s.text(0), s.text(2)], ["one", ""]);
}

#[test]
fn a_taller_screen_takes_back_the_history_it_scrolled() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 2, 100);
    s.write(0, 0, "one");
    grid_scroll_history(s.grid_mut(), 8);
    assert_eq!((s.grid().hsize, s.grid().hscrolled), (1, 1));
    screen_resize(&mut s, 10, 3, 0);
    assert_eq!(
        (s.grid().hsize, s.grid().hscrolled),
        (0, 0),
        "the line came back out of the history"
    );
    assert_eq!(s.text(0), "one");
}

#[test]
fn a_shorter_screen_eats_the_empty_lines_below_the_cursor_first() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 4, 0);
    s.write(0, 0, "one");
    s.write(0, 1, "two");
    s.0.cy = 1;
    screen_resize(&mut s, 10, 2, 0);
    assert_eq!(s.grid().sy, 2);
    assert_eq!([s.text(0), s.text(1)], ["one", "two"]);
    assert_eq!(s.0.cy, 1);
}

#[test]
fn a_shorter_screen_without_history_drops_the_lines_above_the_cursor() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 4, 0);
    s.write(0, 0, "one");
    s.write(0, 1, "two");
    s.write(0, 2, "three");
    s.write(0, 3, "four");
    s.0.cy = 3;
    screen_resize(&mut s, 10, 2, 0);
    assert_eq!([s.text(0), s.text(1)], ["three", "four"]);
    assert_eq!(s.0.cy, 1);
}

#[test]
fn a_shorter_screen_with_history_pushes_the_lines_into_it() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 4, 100);
    s.write(0, 0, "one");
    s.write(0, 1, "two");
    s.write(0, 2, "three");
    s.write(0, 3, "four");
    s.0.cy = 3;
    screen_resize(&mut s, 10, 2, 0);
    assert_eq!(s.grid().hsize, 2);
    assert_eq!([s.text(0), s.text(2)], ["one", "three"]);
    assert_eq!(s.0.cy, 1);
}

#[test]
fn a_narrower_screen_reflows_its_lines_and_carries_the_cursor() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 3, 100);
    s.write(0, 0, "abcdefgh");
    s.0.cx = 7;
    screen_resize(&mut s, 5, 3, 1);
    assert_eq!(s.grid().hsize, 1);
    assert_eq!([s.text(0), s.text(1)], ["abcde", "fgh"]);
    assert_eq!(
        (s.0.cx, s.0.cy),
        (2, 0),
        "the cursor moved with its cell, which is now the first screen line"
    );
}

#[test]
fn a_reflow_that_is_not_asked_to_keep_the_cursor_puts_it_at_the_top() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 3, 100);
    s.write(0, 0, "abcdefgh");
    s.0.cx = 7;
    screen_resize_cursor(&mut s, 5, 3, 1, 1, 0);
    assert_eq!((s.0.cx, s.0.cy), (0, 0));
}

#[test]
fn a_selection_covers_the_cells_between_its_ends() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    let gc = { grid_default_cell };
    s.set_selection(2, 1, 4, 1, false, 0, 0, &gc);
    assert!(s.has_selection());
    let check = |px, py| s.check_selection(px, py) as c_int;
    assert_eq!(check(1, 1), 0);
    assert_eq!(check(2, 1), 1);
    assert_eq!(check(3, 1), 1);
    assert_eq!(check(4, 1), 0, "emacs keys leave the last cell out");
    assert_eq!(check(2, 0), 0);
    assert_eq!(check(2, 2), 0);
}

#[test]
fn a_hidden_or_missing_selection_covers_nothing() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    assert!(!s.check_selection(0, 0));
    let gc = { grid_default_cell };
    s.set_selection(0, 0, 9, 0, false, 0, 0, &gc);
    s.hide_selection();
    assert!(!s.check_selection(1, 0));
    s.clear_selection();
    assert!(!s.has_selection());
    s.hide_selection();
}

#[test]
fn a_selection_can_be_clipped_on_the_left() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    let gc = { grid_default_cell };
    s.set_selection(0, 0, 9, 0, false, 3, 0, &gc);
    assert!(!s.check_selection(2, 0));
    assert!(s.check_selection(3, 0));
}

#[test]
fn a_rectangular_selection_is_a_box() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    let gc = { grid_default_cell };
    s.set_selection(2, 1, 5, 3, true, 0, 0, &gc);
    let check = |px, py| s.check_selection(px, py) as c_int;
    assert_eq!(check(3, 2), 1);
    assert_eq!(check(2, 1), 1);
    assert_eq!(check(5, 3), 1, "a rectangle keeps its last column");
    assert_eq!(check(1, 2), 0);
    assert_eq!(check(6, 2), 0);
    assert_eq!(check(3, 0), 0);
    assert_eq!(check(3, 4), 0);
}

#[test]
fn a_rectangular_selection_can_be_drawn_in_any_direction() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    let gc = { grid_default_cell };
    s.set_selection(5, 3, 2, 1, true, 0, 0, &gc);
    let check = |px, py| s.check_selection(px, py) as c_int;
    assert_eq!(check(3, 2), 1);
    assert_eq!(check(1, 2), 0);
    assert_eq!(check(6, 2), 0);
    assert_eq!(check(3, 0), 0);
    assert_eq!(check(3, 4), 0);

    s.set_selection(2, 2, 5, 2, true, 0, 0, &gc);
    let flat = |px, py| s.check_selection(px, py) as c_int;
    assert_eq!(flat(3, 2), 1);
    assert_eq!(flat(3, 1), 0, "a rectangle of one line is that line");
}

#[test]
fn a_selection_drawn_upwards_covers_the_same_cells() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    let gc = { grid_default_cell };
    s.set_selection(4, 3, 2, 1, false, 0, 0, &gc);
    let check = |px, py| s.check_selection(px, py) as c_int;
    assert_eq!(check(1, 1), 0);
    assert_eq!(check(2, 1), 1);
    assert_eq!(check(9, 2), 1);
    assert_eq!(check(3, 3), 1);
    assert_eq!(check(4, 3), 0, "emacs keys leave the last cell out");
    assert_eq!(check(2, 0), 0);
    assert_eq!(check(2, 4), 0);
}

#[test]
fn a_selection_on_one_line_can_be_drawn_either_way() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    let gc = { grid_default_cell };
    s.set_selection(5, 1, 2, 1, false, 0, 0, &gc);
    let back = |px, py| s.check_selection(px, py) as c_int;
    assert_eq!(back(1, 1), 0);
    assert_eq!(back(2, 1), 1);
    assert_eq!(back(4, 1), 1);
    assert_eq!(back(5, 1), 0);
    assert_eq!(back(3, 0), 0);
}

#[test]
fn vi_keys_take_in_the_last_cell_of_a_selection() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    let gc = { grid_default_cell };
    s.set_selection(2, 1, 4, 1, false, 0, 1, &gc);
    assert!(s.check_selection(4, 1));

    s.set_selection(2, 1, 4, 3, false, 0, 1, &gc);
    assert!(s.check_selection(4, 3));

    s.set_selection(4, 3, 2, 1, false, 0, 1, &gc);
    assert!(s.check_selection(4, 3));

    s.set_selection(5, 1, 2, 1, false, 0, 1, &gc);
    assert!(s.check_selection(5, 1));
}

#[test]
fn a_selection_that_ends_where_it_starts_covers_one_cell_or_none() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    let gc = { grid_default_cell };
    s.set_selection(0, 1, 0, 1, false, 0, 0, &gc);
    assert!(
        s.check_selection(0, 1),
        "a selection of the first cell alone still covers it"
    );
    assert!(!s.check_selection(1, 1));

    s.set_selection(3, 1, 0, 1, false, 0, 0, &gc);
    assert!(s.check_selection(0, 1));
    assert!(s.check_selection(2, 1));
    assert!(!s.check_selection(3, 1));
}

#[test]
fn a_selection_drawn_upwards_to_the_first_cell_covers_nothing_there() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    let gc = { grid_default_cell };
    s.set_selection(0, 3, 2, 1, false, 0, 0, &gc);
    assert!(!s.check_selection(0, 3));
    assert!(s.check_selection(2, 1));
}

#[test]
fn a_selected_cell_takes_the_style_of_the_selection() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    let mut sel = { grid_default_cell };
    sel.attr = GRID_ATTR_NOATTR as u_short;
    let mut src = { grid_default_cell };
    src.fg = 3;
    src.bg = 4;
    src.data.data[0] = b'x';
    src.attr = (GRID_ATTR_CHARSET | 1) as u_short;
    src.flags = GRID_FLAG_TAB as u_char;
    let mut dst = { grid_default_cell };

    assert!(!s.select_cell(&mut dst, &src), "there is no selection");

    s.set_selection(0, 0, 9, 0, false, 0, 0, &sel);
    assert!(s.select_cell(&mut dst, &src));
    assert_eq!(dst.fg, 3, "the default colours come from the cell");
    assert_eq!(dst.bg, 4);
    assert_eq!(dst.data.data[0], b'x');
    assert_eq!(dst.flags, GRID_FLAG_TAB as u_char);
    assert_eq!(
        dst.attr as c_int,
        GRID_ATTR_NOATTR | GRID_ATTR_CHARSET,
        "no attributes but the character set"
    );

    sel.attr = 2;
    sel.fg = 1;
    sel.bg = 2;
    s.set_selection(0, 0, 9, 0, false, 0, 0, &sel);
    s.select_cell(&mut dst, &src);
    assert_eq!(dst.fg, 1, "the selection has its own colours");
    assert_eq!(dst.bg, 2);
    assert_eq!(dst.attr as c_int, 2 | GRID_ATTR_CHARSET | 1);

    s.hide_selection();
    assert!(!s.select_cell(&mut dst, &src));
}

#[test]
fn the_alternate_screen_puts_the_first_one_aside() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 3, 100);
    s.write(0, 0, "main");
    s.0.cx = 4;
    s.0.cy = 0;
    let mut gc = { grid_default_cell };
    gc.fg = 5;

    screen_alternate_on(&mut s, &gc, 1);
    assert!(s.0.saved_grid.is_some());
    assert_eq!(s.text(0), "", "the alternate screen starts empty");
    assert_eq!(s.grid().flags & GRID_HISTORY, 0);
    assert_eq!((s.0.saved_cx, s.0.saved_cy), (4, 0));

    s.write(0, 0, "alt");
    s.0.cx = 3;
    screen_alternate_on(&mut s, &gc, 1);
    assert_eq!(s.text(0), "alt", "a second call does nothing");

    let mut restored = { grid_default_cell };
    screen_alternate_off(&mut s, Some(&mut restored), 1);
    assert!(s.0.saved_grid.is_none());
    assert_eq!(s.text(0), "main");
    assert_eq!((s.0.cx, s.0.cy), (4, 0));
    assert_eq!(restored.fg, 5, "the cell came back with the screen");
    assert_eq!(s.grid().flags & GRID_HISTORY, GRID_HISTORY);
}

#[test]
fn restoring_a_resized_alternate_screen_keeps_main_contents_and_new_dimensions() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 3, 100);
    s.write(0, 0, "main");
    s.0.cx = 4;
    s.0.cy = 0;
    screen_alternate_on(&mut s, &grid_default_cell, 1);
    s.write(0, 0, "alternate");
    {
        screen_resize(&mut s, 20, 5, 0);
        screen_alternate_off(&mut s, None, 1);
    }
    assert_eq!(s.size(), (20, 5));
    assert_eq!(s.text(0), "main");
    assert_eq!((s.0.cx, s.0.cy), (4, 0));
    assert!(s.0.saved_grid.is_none());
    assert_eq!(s.grid().flags & GRID_HISTORY, GRID_HISTORY);
}

#[test]
fn leaving_an_alternate_screen_that_was_never_entered_only_clamps_the_cursor() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 3, 0);
    s.0.cx = 20;
    s.0.cy = 20;
    screen_alternate_off(&mut s, None, 1);
    assert_eq!((s.0.cx, s.0.cy), (9, 2));
}

#[test]
fn an_alternate_screen_can_be_left_without_taking_the_cursor_back() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 3, 0);
    let gc = { grid_default_cell };
    screen_alternate_on(&mut s, &gc, 0);
    assert_eq!((s.0.saved_cx, s.0.saved_cy), (UINT_MAX, UINT_MAX));
    s.0.cx = 2;
    s.0.cy = 1;
    screen_alternate_off(&mut s, None, 1);
    assert_eq!(
        (s.0.cx, s.0.cy),
        (2, 1),
        "there was no cursor to come back to"
    );
}

#[test]
fn a_screen_with_a_write_list_keeps_it_across_a_resize() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 3, 0);
    screen_write_make_list(&mut s);
    assert!(!s.0.write_list.is_empty());
    screen_resize(&mut s, 10, 5, 0);
    assert!(
        !s.0.write_list.is_empty(),
        "it was made again for the new size"
    );
}

#[test]
fn freeing_a_screen_frees_what_it_put_aside() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 3, 0);
    let gc = { grid_default_cell };
    {
        screen_write_make_list(&mut s);
        screen_alternate_on(&mut s, &gc, 1);
        s.set_title(c"one", 0);
        s.push_title();
        s.push_title();
        s.push_title();
    }
    assert!(s.0.saved_grid.is_some());
    assert_eq!(s.0.ntitles, 3);
}

#[test]
fn a_reset_frees_a_whole_stack_of_titles() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 3, 0);
    unsafe {
        s.push_title();
        s.push_title();
        s.push_title();
        screen_reinit(&mut s);
    }
    assert!(s.0.titles.is_none());
}

#[test]
fn a_cursor_left_above_the_screen_goes_back_to_the_top() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 4, 100);
    s.write(0, 0, "one");
    s.0.cx = 2;
    s.0.cy = 0;
    screen_resize_cursor(&mut s, 10, 2, 0, 0, 1);
    assert_eq!(s.grid().hsize, 2);
    assert_eq!(
        (s.0.cx, s.0.cy),
        (0, 0),
        "the cursor is now in the history, so it starts again"
    );
}

#[test]
fn a_shorter_screen_only_eats_as_many_lines_as_it_needs() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 6, 0);
    s.write(0, 0, "one");
    s.write(0, 5, "six");
    s.0.cy = 0;
    screen_resize(&mut s, 10, 4, 0);
    assert_eq!(s.grid().sy, 4);
    assert_eq!(s.text(0), "one");
    assert_eq!(s.text(3), "", "the two lines below the cursor went");
}

#[test]
fn a_taller_screen_only_takes_back_as_much_history_as_it_needs() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 2, 100);
    for text in ["one", "two", "three"] {
        s.write(0, 0, text);
        grid_scroll_history(s.grid_mut(), 8);
    }
    assert_eq!((s.grid().hsize, s.grid().hscrolled), (3, 3));
    screen_resize(&mut s, 10, 3, 0);
    assert_eq!(
        (s.grid().hsize, s.grid().hscrolled),
        (2, 2),
        "only the one line the screen grew by"
    );
}

#[test]
fn a_selection_over_several_lines_takes_in_the_lines_between() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    let gc = { grid_default_cell };
    s.set_selection(2, 1, 4, 3, false, 0, 0, &gc);
    let check = |px, py| s.check_selection(px, py) as c_int;
    assert_eq!(check(2, 0), 0, "above the selection");
    assert_eq!(check(2, 4), 0, "below it");
    assert_eq!(check(1, 1), 0, "before it on its first line");
    assert_eq!(check(2, 1), 1);
    assert_eq!(check(0, 2), 1, "a line in the middle is all selected");
    assert_eq!(check(3, 3), 1);
    assert_eq!(check(4, 3), 0, "emacs keys leave the last cell out");
}

#[test]
fn a_selection_that_ends_on_the_first_cell_of_a_line_still_takes_it() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 5, 0);
    let gc = { grid_default_cell };
    s.set_selection(2, 1, 0, 3, false, 0, 0, &gc);
    assert!(
        s.check_selection(0, 3),
        "there is no cell before the first one to leave out"
    );
    assert!(!s.check_selection(1, 3));
    assert!(s.check_selection(0, 2));
}

#[test]
fn the_cursor_is_clamped_when_the_alternate_screen_is_left() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 3, 0);
    let gc = { grid_default_cell };
    screen_alternate_on(&mut s, &gc, 0);
    s.0.cx = 20;
    s.0.cy = 20;
    screen_alternate_off(&mut s, None, 0);
    assert_eq!((s.0.cx, s.0.cy), (9, 2));
}

/// A screen holding one line of `n` cells, wide enough for all of them.
fn long_line(n: u_int) -> RustScreen {
    let mut s = RustScreen::new_with_server_options(n + 10, 2, 0);
    let gc = { grid_default_cell };
    let text = vec![b'a'; n as usize];
    grid_set_cells(s.grid_mut(), 0, 0, &gc, &text);
    s
}

fn printed(s: &RustScreen) -> String {
    unsafe { String::from_utf8_lossy(screen_print(&s, -1).to_bytes()).into_owned() }
}

#[test]
fn printing_stops_when_the_buffer_is_full() {
    let _guard = globals();

    let header = long_line(16370);
    assert_eq!(
        printed(&header).len(),
        16378,
        "there was no room for the next line's number"
    );

    let ending = long_line(16375);
    assert_eq!(
        printed(&ending).len(),
        16381,
        "there was no room for the closing quote"
    );

    let cells = long_line(16400);
    assert_eq!(printed(&cells).len(), 16382, "there was no room for a cell");
}

#[test]
fn printing_stops_when_a_tab_or_a_wide_character_no_longer_fits() {
    let _guard = globals();

    let mut tab = long_line(16376);
    let mut gc = { grid_default_cell };
    {
        grid_set_tab(&mut gc, 2);
        grid_set_cell(tab.grid_mut(), 16376, 0, &gc);
    }
    assert_eq!(printed(&tab).len(), 16382);

    let mut wide = long_line(16376);
    let mut gc = { grid_default_cell };
    gc.data.data[..3].copy_from_slice("\u{4e2d}".as_bytes());
    gc.data.have = 3;
    gc.data.size = 3;
    gc.data.width = 2;
    grid_set_cell(wide.grid_mut(), 16376, 0, &gc);
    assert_eq!(printed(&wide).len(), 16382);
}

#[test]
fn a_cell_with_no_character_at_all_prints_as_nothing() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 1, 0);
    let mut gc = { grid_default_cell };
    gc.us = 4;
    gc.data.have = 0;
    gc.data.size = 0;
    gc.data.width = 0;
    grid_set_cell(s.grid_mut(), 0, 0, &gc);
    assert_eq!(printed(&s), "0000 \"\"\n");
}

#[test]
fn the_modes_of_a_screen_have_names() {
    let _guard = globals();
    let name = |mode| screen_mode_to_string(mode).to_str().unwrap().to_owned();
    assert_eq!(name(0), "NONE");
    assert_eq!(name(ALL_MODES), "ALL");
    assert_eq!(name(MODE_CURSOR), "CURSOR");
    assert_eq!(name(MODE_CURSOR | MODE_WRAP), "CURSOR,WRAP");
    assert_eq!(
        name(
            MODE_INSERT
                | MODE_KCURSOR
                | MODE_KKEYPAD
                | MODE_MOUSE_STANDARD
                | MODE_MOUSE_BUTTON
                | MODE_CURSOR_BLINKING
                | MODE_CURSOR_VERY_VISIBLE
                | MODE_CURSOR_BLINKING_SET
                | MODE_MOUSE_UTF8
                | MODE_MOUSE_SGR
                | MODE_BRACKETPASTE
                | MODE_FOCUSON
                | MODE_MOUSE_ALL
                | MODE_ORIGIN
                | MODE_CRLF
                | MODE_KEYS_EXTENDED
                | MODE_KEYS_EXTENDED_2
                | MODE_THEME_UPDATES
                | MODE_SYNC
        ),
        "INSERT,KCURSOR,KKEYPAD,MOUSE_STANDARD,MOUSE_BUTTON,CURSOR_BLINKING,\
         CURSOR_VERY_VISIBLE,CURSOR_BLINKING_SET,MOUSE_UTF8,MOUSE_SGR,BRACKETPASTE,\
         FOCUSON,MOUSE_ALL,ORIGIN,CRLF,KEYS_EXTENDED,KEYS_EXTENDED_2,THEME_UPDATES,SYNC"
    );
    assert_eq!(name(0x200000), "", "a mode with no name says nothing");
}

#[test]
fn a_screen_prints_its_lines_in_quotes() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 3, 0);
    s.write(0, 0, "abc");
    s.write(0, 1, "de");
    let printed =
        |line| unsafe { String::from_utf8_lossy(screen_print(&s, line).to_bytes()).into_owned() };
    assert_eq!(printed(-1), "0000 \"abc\"\n0001 \"de\"\n0002 \"\"\n");
    assert_eq!(printed(1), "0001 \"de\"\n");
    assert_eq!(printed(9), "", "there is no such line");
}

#[test]
fn printing_leaves_out_padding_and_writes_tabs_and_wide_characters() {
    let _guard = globals();
    let mut s = RustScreen::new_with_server_options(10, 1, 0);
    let mut gc = { grid_default_cell };
    {
        grid_set_tab(&mut gc, 2);
        grid_set_cell(s.grid_mut(), 0, 0, &gc);
        grid_set_padding(s.grid_mut(), 1, 0);
    }
    let mut wide = { grid_default_cell };
    wide.data.data[..3].copy_from_slice("\u{4e2d}".as_bytes());
    wide.data.have = 3;
    wide.data.size = 3;
    wide.data.width = 2;
    {
        grid_set_cell(s.grid_mut(), 2, 0, &wide);
        grid_set_padding(s.grid_mut(), 3, 0);
    }
    let printed = unsafe { String::from_utf8_lossy(screen_print(&s, 0).to_bytes()).into_owned() };
    assert_eq!(printed, "0000 \"\t\u{4e2d}\"\n");
}

/// The lines of a screen written out one per line, as the caller's own
/// string, stopping at whatever [`PRINT_SIZE`] holds.
pub(crate) unsafe fn screen_print(s: &RustScreen, line: c_int) -> CString {
    unsafe {
        let mut buffer = [0u8; PRINT_SIZE];
        let buf = &mut buffer;

        let mut last = 0;
        let gd = RustScreen::grid(s);
        'out: for y in 0..gd.hsize + gd.sy {
            if line >= 0 && y != line as u_int {
                continue;
            }
            let header = format!("{y:04} \"");
            if header.len() >= PRINT_SIZE - last {
                break;
            }
            buf[last..last + header.len()].copy_from_slice(header.as_bytes());
            last += header.len();

            let gl = &gd.linedata[y as usize];
            for x in 0..gl.cellused {
                let gce = gl.celldata()[x as usize];
                if gce.flags as c_int & GRID_FLAG_PADDING != 0 {
                    continue;
                }

                if gce.flags as c_int & GRID_FLAG_EXTENDED == 0 {
                    if last + 2 >= PRINT_SIZE {
                        break 'out;
                    }
                    buf[last] = gce.value.data.data;
                    last += 1;
                } else if gce.flags as c_int & GRID_FLAG_TAB != 0 {
                    /*
                     * The arm for the alternate character set that came next
                     * in the C is gone: it tested the same bit as the tab
                     * above it, so it was never reached.
                     */
                    if last + 2 >= PRINT_SIZE {
                        break 'out;
                    }
                    buf[last] = b'\t';
                    last += 1;
                } else {
                    let mut ud = utf8_data::default();
                    utf8_to_data(
                        gl.extddata()[gce.value.offset as usize].data,
                        &mut ud,
                    );
                    let size = ud.size as usize;
                    if size > 0 {
                        if last + size + 1 >= PRINT_SIZE {
                            break 'out;
                        }
                        buf[last..last + size].copy_from_slice(&ud.data[..size]);
                        last += size;
                    }
                }
            }

            if last + 3 >= PRINT_SIZE {
                break;
            }
            buf[last] = b'"';
            buf[last + 1] = b'\n';
            last += 2;
        }
        // A cell holding a zero byte ended the C's answer where it stood, so
        // the printed lines stop there too.
        let printed = &buf[..last];
        let end = printed.iter().position(|&byte| byte == 0).unwrap_or(last);
        CString::new(&printed[..end]).expect("the bytes stop at the first zero")
    }
}

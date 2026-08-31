use super::*;
use crate::grid::{
    grid_default_cell, grid_scroll_history, grid_set_cell, grid_set_cells, grid_set_padding,
    grid_set_tab, grid_string_cells,
};
use crate::options::options_set_number;
use crate::tests::test_fixtures::globals;
use crate::tmux::global_w_options;
use ::core::ffi::{CStr, c_int};
use ::core::ptr::null_mut;

/// A screen that frees itself at the end of the test.
struct Screen(Box<screen>);

impl Screen {
    fn new(sx: u_int, sy: u_int, hlimit: u_int) -> Screen {
        Screen(Box::new(screen::new(sx, sy, hlimit)))
    }

    fn ptr(&mut self) -> *mut screen {
        &raw mut *self.0
    }

    fn grid(&self) -> *mut grid {
        screen_grid(&self.0) as *const grid as *mut grid
    }

    fn title(&self) -> String {
        unsafe {
            String::from_utf8_lossy(CStr::from_ptr(cstr_ptr(&self.0.title)).to_bytes()).into_owned()
        }
    }

    /// Writes `s` from (px, py) of the screen, one cell per byte.
    fn write(&mut self, px: u_int, py: u_int, text: &str) {
        let mut gc = unsafe { grid_default_cell };
        for (i, byte) in text.bytes().enumerate() {
            gc.data.data[0] = byte;
            gc.data.have = 1;
            gc.data.size = 1;
            gc.data.width = 1;
            unsafe {
                grid_set_cell(
                    &mut *self.grid(),
                    px + i as u_int,
                    (*self.grid()).hsize + py,
                    &gc,
                )
            };
        }
    }

    /// The text of one line of the whole grid, history included.
    fn text(&self, py: u_int) -> String {
        unsafe {
            let p = grid_string_cells(&*self.grid(), 0, py, 1000, None, 0, null_mut());

            p.to_string_lossy().into_owned()
        }
    }

    /// Whether there is a tab stop at each column of the screen.
    fn tabs(&self) -> Vec<bool> {
        unsafe {
            (0..(*self.grid()).sx)
                .map(|i| self.0.tabs[(i >> 3) as usize] as c_int & (1 << (i & 0x7)) != 0)
                .collect()
        }
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        unsafe { screen_free(&mut *self.0) };
    }
}

impl ::core::ops::Deref for Screen {
    type Target = screen;

    fn deref(&self) -> &screen {
        &self.0
    }
}

#[test]
fn a_new_screen_has_a_grid_and_nothing_else() {
    let _guard = globals();
    let s = Screen::new(10, 5, 100);
    assert_eq!(unsafe { (*s.grid()).sx }, 10);
    assert_eq!(unsafe { (*s.grid()).sy }, 5);
    assert_eq!(unsafe { (*s.grid()).hlimit }, 100);
    assert_eq!(s.title(), "");
    assert_eq!(s.path, None);
    assert!(s.titles.is_none());
    assert!(s.saved_grid.is_none());
    assert!(s.sel.is_none());
    assert!(s.write_list.is_empty());
    assert!(s.hyperlinks.is_some());
    assert_eq!(s.cstyle, SCREEN_CURSOR_DEFAULT);
    assert_eq!(s.default_cstyle, SCREEN_CURSOR_DEFAULT);
    assert_eq!(s.ccolour, -1);
    assert_eq!(s.default_ccolour, -1);
    assert_eq!(s.default_mode, 0);
    assert_eq!(s.mode, MODE_CURSOR | MODE_WRAP);
    assert_eq!((s.cx, s.cy), (0, 0));
    assert_eq!((s.rupper, s.rlower), (0, 4));
    assert_eq!((s.saved_cx, s.saved_cy), (UINT_MAX, UINT_MAX));
}

#[test]
fn a_screen_is_reset_to_what_it_started_as() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 100);
    s.write(0, 0, "abc");
    s.0.cx = 3;
    s.0.cy = 2;
    s.0.mode = MODE_CRLF | MODE_INSERT;
    unsafe {
        screen_push_title(s.ptr());
        let mut gc = grid_default_cell;
        screen_set_selection(s.ptr(), 0, 0, 1, 1, 0, 0, 0, &mut gc);
        screen_set_progress_bar(s.ptr(), PROGRESS_BAR_NORMAL, 50);
        screen_reinit(&mut *s.ptr());
    }
    assert_eq!((s.cx, s.cy), (0, 0));
    assert_eq!(
        s.mode,
        MODE_CURSOR | MODE_WRAP | MODE_CRLF,
        "only the newline mode survives"
    );
    assert_eq!(s.text(0), "");
    assert!(s.sel.is_none());
    assert!(s.titles.is_none());
    assert_eq!(s.progress_bar.state, PROGRESS_BAR_HIDDEN);
    assert_eq!(s.progress_bar.progress, 0);
}

#[test]
fn a_reset_turns_extended_keys_on_when_the_option_asks_for_it() {
    let _guard = globals();
    unsafe { options_set_number(global_options, c"extended-keys".as_ptr(), 2) };
    let s = Screen::new(10, 5, 0);
    assert_eq!(s.mode & MODE_KEYS_EXTENDED, MODE_KEYS_EXTENDED);
    unsafe { options_set_number(global_options, c"extended-keys".as_ptr(), 0) };
    let plain = Screen::new(10, 5, 0);
    assert_eq!(plain.mode & MODE_KEYS_EXTENDED, 0);
}

#[test]
fn a_reset_comes_out_of_the_alternate_screen() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    s.write(0, 0, "main");
    let mut gc = unsafe { grid_default_cell };
    unsafe {
        screen_alternate_on(s.ptr(), &gc, 1);
        screen_reinit(&mut *s.ptr());
    }
    assert!(s.saved_grid.is_none());
    assert_eq!(s.text(0), "", "the reset cleared the restored screen");
}

#[test]
fn the_hyperlinks_of_a_screen_are_made_once_and_then_emptied() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    let first = s.hyperlinks_ptr();
    unsafe { screen_reset_hyperlinks(s.ptr()) };
    assert_eq!(s.hyperlinks_ptr(), first, "the same table, emptied");
}

#[test]
fn tab_stops_are_every_eight_columns() {
    let _guard = globals();
    let s = Screen::new(20, 5, 0);
    let tabs = s.tabs();
    for (i, stop) in tabs.iter().enumerate() {
        assert_eq!(*stop, i != 0 && i % 8 == 0, "column {i}");
    }
}

#[test]
fn the_default_cursor_comes_from_the_options() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    unsafe {
        options_set_number(global_w_options, c"cursor-colour".as_ptr(), 4);
        options_set_number(global_w_options, c"cursor-style".as_ptr(), 3);
        screen_set_default_cursor(s.ptr(), global_w_options);
    }
    assert_eq!(s.default_ccolour, 4);
    assert_eq!(s.default_cstyle, SCREEN_CURSOR_UNDERLINE);
    assert_eq!(s.default_mode, MODE_CURSOR_BLINKING);
    unsafe {
        options_set_number(global_w_options, c"cursor-colour".as_ptr(), -1);
        options_set_number(global_w_options, c"cursor-style".as_ptr(), 0);
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
    let mut s = Screen::new(10, 5, 0);
    unsafe { screen_set_cursor_colour(s.ptr(), 42) };
    assert_eq!(s.ccolour, 42);
}

#[test]
fn a_title_and_a_path_are_cleaned_before_they_are_kept() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    assert_eq!(
        unsafe { screen_set_title(&mut *s.ptr(), c"hello".as_ptr(), 0) },
        1
    );
    assert_eq!(s.title(), "hello");
    assert_eq!(unsafe { screen_set_title(&mut *s.ptr(), c"a#(b".as_ptr(), 1) }, 1);
    assert_eq!(s.title(), "a_(b", "an untrusted format is defused");
    assert_eq!(
        unsafe { screen_set_title(&mut *s.ptr(), c"\xc3\x28".as_ptr(), 0) },
        0,
        "invalid UTF-8 is turned down"
    );
    assert_eq!(s.title(), "a_(b");

    assert_eq!(unsafe { screen_set_path(s.ptr(), c"/tmp".as_ptr(), 0) }, 1);
    assert_eq!(
        unsafe { CStr::from_ptr(cstr_ptr(&s.path)).to_str().unwrap() },
        "/tmp"
    );
    assert_eq!(
        unsafe { screen_set_path(s.ptr(), c"\xc3\x28".as_ptr(), 0) },
        0
    );
}

#[test]
fn titles_are_pushed_and_popped_as_a_stack() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    unsafe {
        screen_set_title(&mut *s.ptr(), c"one".as_ptr(), 0);
        screen_push_title(s.ptr());
        screen_set_title(&mut *s.ptr(), c"two".as_ptr(), 0);
        screen_push_title(s.ptr());
        screen_set_title(&mut *s.ptr(), c"three".as_ptr(), 0);
    }
    assert_eq!(s.ntitles, 2);

    unsafe { screen_pop_title(s.ptr()) };
    assert_eq!(s.title(), "two");
    assert_eq!(s.ntitles, 1);
    unsafe { screen_pop_title(s.ptr()) };
    assert_eq!(s.title(), "one");
    assert_eq!(s.ntitles, 0);
    unsafe { screen_pop_title(s.ptr()) };
    assert_eq!(s.title(), "one", "an empty stack leaves the title alone");
}

#[test]
fn popping_a_title_that_was_never_pushed_does_nothing() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    unsafe {
        screen_set_title(&mut *s.ptr(), c"one".as_ptr(), 0);
        screen_pop_title(s.ptr());
    }
    assert_eq!(s.title(), "one");
    assert!(s.titles.is_none());
}

#[test]
fn the_title_stack_holds_ten() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    for i in 0..12 {
        let title = ::std::ffi::CString::new(format!("t{i}")).unwrap();
        unsafe {
            screen_set_title(&mut *s.ptr(), title.as_ptr(), 0);
            screen_push_title(s.ptr());
        }
    }
    assert_eq!(s.ntitles, 10);
    for i in (2..12).rev() {
        unsafe { screen_pop_title(s.ptr()) };
        assert_eq!(s.title(), format!("t{i}"), "the oldest two were dropped");
    }
    assert_eq!(s.ntitles, 0);
}

#[test]
fn a_progress_bar_keeps_its_progress_unless_it_has_none() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    unsafe { screen_set_progress_bar(s.ptr(), PROGRESS_BAR_NORMAL, 30) };
    assert_eq!(s.progress_bar.state, PROGRESS_BAR_NORMAL);
    assert_eq!(s.progress_bar.progress, 30);
    unsafe { screen_set_progress_bar(s.ptr(), PROGRESS_BAR_ERROR, -1) };
    assert_eq!(s.progress_bar.state, PROGRESS_BAR_ERROR);
    assert_eq!(
        s.progress_bar.progress, 30,
        "a negative progress is no news"
    );
    unsafe { screen_set_progress_bar(s.ptr(), PROGRESS_BAR_INDETERMINATE, 70) };
    assert_eq!(
        s.progress_bar.progress, 30,
        "an indeterminate bar has no progress to set"
    );
}

#[test]
fn a_screen_can_be_made_wider_and_narrower() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    s.write(0, 0, "abcdefghij");
    unsafe { screen_resize(&mut *s.ptr(), 20, 5, 0) };
    assert_eq!(unsafe { (*s.grid()).sx }, 20);
    assert_eq!(s.tabs().len(), 20);
    assert_eq!(s.text(0), "abcdefghij");

    unsafe { screen_resize(&mut *s.ptr(), 20, 5, 0) };
    assert_eq!(unsafe { (*s.grid()).sx }, 20, "the same width is no change");
}

#[test]
fn a_screen_is_never_smaller_than_one_cell() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    unsafe { screen_resize(&mut *s.ptr(), 0, 0, 0) };
    assert_eq!(unsafe { ((*s.grid()).sx, (*s.grid()).sy) }, (1, 1));
}

#[test]
fn a_taller_screen_gets_empty_lines_at_the_bottom() {
    let _guard = globals();
    let mut s = Screen::new(10, 2, 0);
    s.write(0, 0, "one");
    s.write(0, 1, "two");
    unsafe { screen_resize(&mut *s.ptr(), 10, 4, 0) };
    assert_eq!(unsafe { (*s.grid()).sy }, 4);
    assert_eq!(s.rlower, 3);
    assert_eq!([s.text(0), s.text(2)], ["one", ""]);
}

#[test]
fn a_taller_screen_takes_back_the_history_it_scrolled() {
    let _guard = globals();
    let mut s = Screen::new(10, 2, 100);
    s.write(0, 0, "one");
    unsafe { grid_scroll_history(&mut *s.grid(), 8) };
    assert_eq!(
        unsafe { ((*s.grid()).hsize, (*s.grid()).hscrolled) },
        (1, 1)
    );
    unsafe { screen_resize(&mut *s.ptr(), 10, 3, 0) };
    assert_eq!(
        unsafe { ((*s.grid()).hsize, (*s.grid()).hscrolled) },
        (0, 0),
        "the line came back out of the history"
    );
    assert_eq!(s.text(0), "one");
}

#[test]
fn a_shorter_screen_eats_the_empty_lines_below_the_cursor_first() {
    let _guard = globals();
    let mut s = Screen::new(10, 4, 0);
    s.write(0, 0, "one");
    s.write(0, 1, "two");
    s.0.cy = 1;
    unsafe { screen_resize(&mut *s.ptr(), 10, 2, 0) };
    assert_eq!(unsafe { (*s.grid()).sy }, 2);
    assert_eq!([s.text(0), s.text(1)], ["one", "two"]);
    assert_eq!(s.cy, 1);
}

#[test]
fn a_shorter_screen_without_history_drops_the_lines_above_the_cursor() {
    let _guard = globals();
    let mut s = Screen::new(10, 4, 0);
    s.write(0, 0, "one");
    s.write(0, 1, "two");
    s.write(0, 2, "three");
    s.write(0, 3, "four");
    s.0.cy = 3;
    unsafe { screen_resize(&mut *s.ptr(), 10, 2, 0) };
    assert_eq!([s.text(0), s.text(1)], ["three", "four"]);
    assert_eq!(s.cy, 1);
}

#[test]
fn a_shorter_screen_with_history_pushes_the_lines_into_it() {
    let _guard = globals();
    let mut s = Screen::new(10, 4, 100);
    s.write(0, 0, "one");
    s.write(0, 1, "two");
    s.write(0, 2, "three");
    s.write(0, 3, "four");
    s.0.cy = 3;
    unsafe { screen_resize(&mut *s.ptr(), 10, 2, 0) };
    assert_eq!(unsafe { (*s.grid()).hsize }, 2);
    assert_eq!([s.text(0), s.text(2)], ["one", "three"]);
    assert_eq!(s.cy, 1);
}

#[test]
fn a_narrower_screen_reflows_its_lines_and_carries_the_cursor() {
    let _guard = globals();
    let mut s = Screen::new(10, 3, 100);
    s.write(0, 0, "abcdefgh");
    s.0.cx = 7;
    unsafe { screen_resize(&mut *s.ptr(), 5, 3, 1) };
    assert_eq!(unsafe { (*s.grid()).hsize }, 1);
    assert_eq!([s.text(0), s.text(1)], ["abcde", "fgh"]);
    assert_eq!(
        (s.cx, s.cy),
        (2, 0),
        "the cursor moved with its cell, which is now the first screen line"
    );
}

#[test]
fn a_reflow_that_is_not_asked_to_keep_the_cursor_puts_it_at_the_top() {
    let _guard = globals();
    let mut s = Screen::new(10, 3, 100);
    s.write(0, 0, "abcdefgh");
    s.0.cx = 7;
    unsafe { screen_resize_cursor(s.ptr(), 5, 3, 1, 1, 0) };
    assert_eq!((s.cx, s.cy), (0, 0));
}

#[test]
fn a_selection_covers_the_cells_between_its_ends() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    let mut gc = unsafe { grid_default_cell };
    unsafe { screen_set_selection(s.ptr(), 2, 1, 4, 1, 0, 0, 0, &mut gc) };
    assert!(s.sel.is_some());
    let mut check = |px, py| unsafe { screen_check_selection(&raw mut *s.0, px, py) };
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
    let mut s = Screen::new(10, 5, 0);
    assert_eq!(unsafe { screen_check_selection(s.ptr(), 0, 0) }, 0);
    let mut gc = unsafe { grid_default_cell };
    unsafe {
        screen_set_selection(s.ptr(), 0, 0, 9, 0, 0, 0, 0, &mut gc);
        screen_hide_selection(s.ptr());
    }
    assert_eq!(unsafe { screen_check_selection(s.ptr(), 1, 0) }, 0);
    unsafe { screen_clear_selection(&mut *s.ptr()) };
    assert!(s.sel.is_none());
    unsafe { screen_hide_selection(s.ptr()) };
}

#[test]
fn a_selection_can_be_clipped_on_the_left() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    let mut gc = unsafe { grid_default_cell };
    unsafe { screen_set_selection(s.ptr(), 0, 0, 9, 0, 0, 3, 0, &mut gc) };
    assert_eq!(unsafe { screen_check_selection(s.ptr(), 2, 0) }, 0);
    assert_eq!(unsafe { screen_check_selection(s.ptr(), 3, 0) }, 1);
}

#[test]
fn a_rectangular_selection_is_a_box() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    let mut gc = unsafe { grid_default_cell };
    unsafe { screen_set_selection(s.ptr(), 2, 1, 5, 3, 1, 0, 0, &mut gc) };
    let mut check = |px, py| unsafe { screen_check_selection(&raw mut *s.0, px, py) };
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
    let mut s = Screen::new(10, 5, 0);
    let mut gc = unsafe { grid_default_cell };
    unsafe { screen_set_selection(s.ptr(), 5, 3, 2, 1, 1, 0, 0, &mut gc) };
    let mut check = |px, py| unsafe { screen_check_selection(&raw mut *s.0, px, py) };
    assert_eq!(check(3, 2), 1);
    assert_eq!(check(1, 2), 0);
    assert_eq!(check(6, 2), 0);
    assert_eq!(check(3, 0), 0);
    assert_eq!(check(3, 4), 0);

    unsafe { screen_set_selection(s.ptr(), 2, 2, 5, 2, 1, 0, 0, &mut gc) };
    let mut flat = |px, py| unsafe { screen_check_selection(&raw mut *s.0, px, py) };
    assert_eq!(flat(3, 2), 1);
    assert_eq!(flat(3, 1), 0, "a rectangle of one line is that line");
}

#[test]
fn a_selection_drawn_upwards_covers_the_same_cells() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    let mut gc = unsafe { grid_default_cell };
    unsafe { screen_set_selection(s.ptr(), 4, 3, 2, 1, 0, 0, 0, &mut gc) };
    let mut check = |px, py| unsafe { screen_check_selection(&raw mut *s.0, px, py) };
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
    let mut s = Screen::new(10, 5, 0);
    let mut gc = unsafe { grid_default_cell };
    unsafe { screen_set_selection(s.ptr(), 5, 1, 2, 1, 0, 0, 0, &mut gc) };
    let mut back = |px, py| unsafe { screen_check_selection(&raw mut *s.0, px, py) };
    assert_eq!(back(1, 1), 0);
    assert_eq!(back(2, 1), 1);
    assert_eq!(back(4, 1), 1);
    assert_eq!(back(5, 1), 0);
    assert_eq!(back(3, 0), 0);
}

#[test]
fn vi_keys_take_in_the_last_cell_of_a_selection() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    let mut gc = unsafe { grid_default_cell };
    unsafe { screen_set_selection(s.ptr(), 2, 1, 4, 1, 0, 0, 1, &mut gc) };
    assert_eq!(unsafe { screen_check_selection(s.ptr(), 4, 1) }, 1);

    unsafe { screen_set_selection(s.ptr(), 2, 1, 4, 3, 0, 0, 1, &mut gc) };
    assert_eq!(unsafe { screen_check_selection(s.ptr(), 4, 3) }, 1);

    unsafe { screen_set_selection(s.ptr(), 4, 3, 2, 1, 0, 0, 1, &mut gc) };
    assert_eq!(unsafe { screen_check_selection(s.ptr(), 4, 3) }, 1);

    unsafe { screen_set_selection(s.ptr(), 5, 1, 2, 1, 0, 0, 1, &mut gc) };
    assert_eq!(unsafe { screen_check_selection(s.ptr(), 5, 1) }, 1);
}

#[test]
fn a_selection_that_ends_where_it_starts_covers_one_cell_or_none() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    let mut gc = unsafe { grid_default_cell };
    unsafe { screen_set_selection(s.ptr(), 0, 1, 0, 1, 0, 0, 0, &mut gc) };
    assert_eq!(
        unsafe { screen_check_selection(s.ptr(), 0, 1) },
        1,
        "a selection of the first cell alone still covers it"
    );
    assert_eq!(unsafe { screen_check_selection(s.ptr(), 1, 1) }, 0);

    unsafe { screen_set_selection(s.ptr(), 3, 1, 0, 1, 0, 0, 0, &mut gc) };
    assert_eq!(unsafe { screen_check_selection(s.ptr(), 0, 1) }, 1);
    assert_eq!(unsafe { screen_check_selection(s.ptr(), 2, 1) }, 1);
    assert_eq!(unsafe { screen_check_selection(s.ptr(), 3, 1) }, 0);
}

#[test]
fn a_selection_drawn_upwards_to_the_first_cell_covers_nothing_there() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    let mut gc = unsafe { grid_default_cell };
    unsafe { screen_set_selection(s.ptr(), 0, 3, 2, 1, 0, 0, 0, &mut gc) };
    assert_eq!(unsafe { screen_check_selection(s.ptr(), 0, 3) }, 0);
    assert_eq!(unsafe { screen_check_selection(s.ptr(), 2, 1) }, 1);
}

#[test]
fn a_selected_cell_takes_the_style_of_the_selection() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    let mut sel = unsafe { grid_default_cell };
    sel.attr = GRID_ATTR_NOATTR as u_short;
    let mut src = unsafe { grid_default_cell };
    src.fg = 3;
    src.bg = 4;
    src.data.data[0] = b'x';
    src.attr = (GRID_ATTR_CHARSET | 1) as u_short;
    src.flags = GRID_FLAG_TAB as u_char;
    let mut dst = unsafe { grid_default_cell };

    assert_eq!(
        unsafe { screen_select_cell(s.ptr(), &mut dst, &src) },
        0,
        "there is no selection"
    );

    unsafe { screen_set_selection(s.ptr(), 0, 0, 9, 0, 0, 0, 0, &mut sel) };
    assert_eq!(unsafe { screen_select_cell(s.ptr(), &mut dst, &src) }, 1);
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
    unsafe {
        screen_set_selection(s.ptr(), 0, 0, 9, 0, 0, 0, 0, &mut sel);
        screen_select_cell(s.ptr(), &mut dst, &src);
    }
    assert_eq!(dst.fg, 1, "the selection has its own colours");
    assert_eq!(dst.bg, 2);
    assert_eq!(dst.attr as c_int, 2 | GRID_ATTR_CHARSET | 1);

    unsafe { screen_hide_selection(s.ptr()) };
    assert_eq!(unsafe { screen_select_cell(s.ptr(), &mut dst, &src) }, 0);
}

#[test]
fn the_alternate_screen_puts_the_first_one_aside() {
    let _guard = globals();
    let mut s = Screen::new(10, 3, 100);
    s.write(0, 0, "main");
    s.0.cx = 4;
    s.0.cy = 0;
    let mut gc = unsafe { grid_default_cell };
    gc.fg = 5;

    unsafe { screen_alternate_on(s.ptr(), &gc, 1) };
    assert!(s.saved_grid.is_some());
    assert_eq!(s.text(0), "", "the alternate screen starts empty");
    assert_eq!(unsafe { (*s.grid()).flags } & GRID_HISTORY, 0);
    assert_eq!((s.saved_cx, s.saved_cy), (4, 0));

    s.write(0, 0, "alt");
    s.0.cx = 3;
    unsafe { screen_alternate_on(s.ptr(), &gc, 1) };
    assert_eq!(s.text(0), "alt", "a second call does nothing");

    let mut restored = unsafe { grid_default_cell };
    unsafe { screen_alternate_off(s.ptr(), Some(&mut restored), 1) };
    assert!(s.saved_grid.is_none());
    assert_eq!(s.text(0), "main");
    assert_eq!((s.cx, s.cy), (4, 0));
    assert_eq!(restored.fg, 5, "the cell came back with the screen");
    assert_eq!(unsafe { (*s.grid()).flags } & GRID_HISTORY, GRID_HISTORY);
}

#[test]
fn leaving_an_alternate_screen_that_was_never_entered_only_clamps_the_cursor() {
    let _guard = globals();
    let mut s = Screen::new(10, 3, 0);
    s.0.cx = 20;
    s.0.cy = 20;
    unsafe { screen_alternate_off(s.ptr(), None, 1) };
    assert_eq!((s.cx, s.cy), (9, 2));
}

#[test]
fn an_alternate_screen_can_be_left_without_taking_the_cursor_back() {
    let _guard = globals();
    let mut s = Screen::new(10, 3, 0);
    let mut gc = unsafe { grid_default_cell };
    unsafe { screen_alternate_on(s.ptr(), &gc, 0) };
    assert_eq!((s.saved_cx, s.saved_cy), (UINT_MAX, UINT_MAX));
    s.0.cx = 2;
    s.0.cy = 1;
    unsafe { screen_alternate_off(s.ptr(), None, 1) };
    assert_eq!((s.cx, s.cy), (2, 1), "there was no cursor to come back to");
}

#[test]
fn a_screen_with_a_write_list_keeps_it_across_a_resize() {
    let _guard = globals();
    let mut s = Screen::new(10, 3, 0);
    unsafe { screen_write_make_list(s.ptr()) };
    assert!(!s.write_list.is_empty());
    unsafe { screen_resize(&mut *s.ptr(), 10, 5, 0) };
    assert!(
        !s.write_list.is_empty(),
        "it was made again for the new size"
    );
}

#[test]
fn freeing_a_screen_frees_what_it_put_aside() {
    let _guard = globals();
    let mut s = Screen::new(10, 3, 0);
    let mut gc = unsafe { grid_default_cell };
    unsafe {
        screen_write_make_list(s.ptr());
        screen_alternate_on(s.ptr(), &gc, 1);
        screen_set_title(&mut *s.ptr(), c"one".as_ptr(), 0);
        screen_push_title(s.ptr());
        screen_push_title(s.ptr());
        screen_push_title(s.ptr());
    }
    assert!(s.saved_grid.is_some());
    assert_eq!(s.ntitles, 3);
}

#[test]
fn a_reset_frees_a_whole_stack_of_titles() {
    let _guard = globals();
    let mut s = Screen::new(10, 3, 0);
    unsafe {
        screen_push_title(s.ptr());
        screen_push_title(s.ptr());
        screen_push_title(s.ptr());
        screen_reinit(&mut *s.ptr());
    }
    assert!(s.titles.is_none());
}

#[test]
fn a_cursor_left_above_the_screen_goes_back_to_the_top() {
    let _guard = globals();
    let mut s = Screen::new(10, 4, 100);
    s.write(0, 0, "one");
    s.0.cx = 2;
    s.0.cy = 0;
    unsafe { screen_resize_cursor(s.ptr(), 10, 2, 0, 0, 1) };
    assert_eq!(unsafe { (*s.grid()).hsize }, 2);
    assert_eq!(
        (s.cx, s.cy),
        (0, 0),
        "the cursor is now in the history, so it starts again"
    );
}

#[test]
fn a_shorter_screen_only_eats_as_many_lines_as_it_needs() {
    let _guard = globals();
    let mut s = Screen::new(10, 6, 0);
    s.write(0, 0, "one");
    s.write(0, 5, "six");
    s.0.cy = 0;
    unsafe { screen_resize(&mut *s.ptr(), 10, 4, 0) };
    assert_eq!(unsafe { (*s.grid()).sy }, 4);
    assert_eq!(s.text(0), "one");
    assert_eq!(s.text(3), "", "the two lines below the cursor went");
}

#[test]
fn a_taller_screen_only_takes_back_as_much_history_as_it_needs() {
    let _guard = globals();
    let mut s = Screen::new(10, 2, 100);
    for text in ["one", "two", "three"] {
        s.write(0, 0, text);
        unsafe { grid_scroll_history(&mut *s.grid(), 8) };
    }
    assert_eq!(
        unsafe { ((*s.grid()).hsize, (*s.grid()).hscrolled) },
        (3, 3)
    );
    unsafe { screen_resize(&mut *s.ptr(), 10, 3, 0) };
    assert_eq!(
        unsafe { ((*s.grid()).hsize, (*s.grid()).hscrolled) },
        (2, 2),
        "only the one line the screen grew by"
    );
}

#[test]
fn a_selection_over_several_lines_takes_in_the_lines_between() {
    let _guard = globals();
    let mut s = Screen::new(10, 5, 0);
    let mut gc = unsafe { grid_default_cell };
    unsafe { screen_set_selection(s.ptr(), 2, 1, 4, 3, 0, 0, 0, &mut gc) };
    let mut check = |px, py| unsafe { screen_check_selection(&raw mut *s.0, px, py) };
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
    let mut s = Screen::new(10, 5, 0);
    let mut gc = unsafe { grid_default_cell };
    unsafe { screen_set_selection(s.ptr(), 2, 1, 0, 3, 0, 0, 0, &mut gc) };
    assert_eq!(
        unsafe { screen_check_selection(s.ptr(), 0, 3) },
        1,
        "there is no cell before the first one to leave out"
    );
    assert_eq!(unsafe { screen_check_selection(s.ptr(), 1, 3) }, 0);
    assert_eq!(unsafe { screen_check_selection(s.ptr(), 0, 2) }, 1);
}

#[test]
fn the_cursor_is_clamped_when_the_alternate_screen_is_left() {
    let _guard = globals();
    let mut s = Screen::new(10, 3, 0);
    let mut gc = unsafe { grid_default_cell };
    unsafe { screen_alternate_on(s.ptr(), &gc, 0) };
    s.0.cx = 20;
    s.0.cy = 20;
    unsafe { screen_alternate_off(s.ptr(), None, 0) };
    assert_eq!((s.cx, s.cy), (9, 2));
}

/// A screen holding one line of `n` cells, wide enough for all of them.
fn long_line(n: u_int) -> Screen {
    let mut s = Screen::new(n + 10, 2, 0);
    let gc = unsafe { grid_default_cell };
    let text = vec![b'a'; n as usize];
    unsafe { grid_set_cells(&mut *s.grid(), 0, 0, &gc, &text) };
    s
}

fn printed(s: &mut Screen) -> String {
    unsafe { String::from_utf8_lossy(screen_print(s.ptr(), -1).to_bytes()).into_owned() }
}

#[test]
fn printing_stops_when_the_buffer_is_full() {
    let _guard = globals();

    let mut header = long_line(16370);
    assert_eq!(
        printed(&mut header).len(),
        16378,
        "there was no room for the next line's number"
    );

    let mut ending = long_line(16375);
    assert_eq!(
        printed(&mut ending).len(),
        16381,
        "there was no room for the closing quote"
    );

    let mut cells = long_line(16400);
    assert_eq!(
        printed(&mut cells).len(),
        16382,
        "there was no room for a cell"
    );
}

#[test]
fn printing_stops_when_a_tab_or_a_wide_character_no_longer_fits() {
    let _guard = globals();

    let mut tab = long_line(16376);
    let mut gc = unsafe { grid_default_cell };
    unsafe {
        grid_set_tab(&mut gc, 2);
        grid_set_cell(&mut *tab.grid(), 16376, 0, &gc);
    }
    assert_eq!(printed(&mut tab).len(), 16382);

    let mut wide = long_line(16376);
    let mut gc = unsafe { grid_default_cell };
    gc.data.data[..3].copy_from_slice("\u{4e2d}".as_bytes());
    gc.data.have = 3;
    gc.data.size = 3;
    gc.data.width = 2;
    unsafe { grid_set_cell(&mut *wide.grid(), 16376, 0, &gc) };
    assert_eq!(printed(&mut wide).len(), 16382);
}

#[test]
fn a_cell_with_no_character_at_all_prints_as_nothing() {
    let _guard = globals();
    let mut s = Screen::new(10, 1, 0);
    let mut gc = unsafe { grid_default_cell };
    gc.us = 4;
    gc.data.have = 0;
    gc.data.size = 0;
    gc.data.width = 0;
    unsafe { grid_set_cell(&mut *s.grid(), 0, 0, &gc) };
    assert_eq!(printed(&mut s), "0000 \"\"\n");
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
    let mut s = Screen::new(10, 3, 0);
    s.write(0, 0, "abc");
    s.write(0, 1, "de");
    let mut printed = |line| unsafe {
        String::from_utf8_lossy(screen_print(&raw mut *s.0, line).to_bytes()).into_owned()
    };
    assert_eq!(printed(-1), "0000 \"abc\"\n0001 \"de\"\n0002 \"\"\n");
    assert_eq!(printed(1), "0001 \"de\"\n");
    assert_eq!(printed(9), "", "there is no such line");
}

#[test]
fn printing_leaves_out_padding_and_writes_tabs_and_wide_characters() {
    let _guard = globals();
    let mut s = Screen::new(10, 1, 0);
    let mut gc = unsafe { grid_default_cell };
    unsafe {
        grid_set_tab(&mut gc, 2);
        grid_set_cell(&mut *s.grid(), 0, 0, &gc);
        grid_set_padding(&mut *s.grid(), 1, 0);
    }
    let mut wide = unsafe { grid_default_cell };
    wide.data.data[..3].copy_from_slice("\u{4e2d}".as_bytes());
    wide.data.have = 3;
    wide.data.size = 3;
    wide.data.width = 2;
    unsafe {
        grid_set_cell(&mut *s.grid(), 2, 0, &wide);
        grid_set_padding(&mut *s.grid(), 3, 0);
    }
    let printed =
        unsafe { String::from_utf8_lossy(screen_print(&raw mut *s.0, 0).to_bytes()).into_owned() };
    assert_eq!(printed, "0000 \"\t\u{4e2d}\"\n");
}

use crate::WindowPane;
use crate::options::OptionsRef;
use crate::pane_geometry::PaneGeometryState;
use crate::pane_identity::PaneIdentity;
use crate::pane_scrollbar::{PaneScrollbarSlider};

use super::*;
use crate::grid::{grid_create, grid_get_cell, grid_line_info, grid_scroll_history, grid_set_cell};
use crate::tests::test_fixtures::{Args, Pane, Target, Window, ascii, globals};

unsafe fn open_copy(target: &mut Target) -> &mut window_mode_entry {
    unsafe {
        let pane = target.pane(0);
        for _ in 0..24 {
            grid_scroll_history(RustScreen::grid_mut((*pane).base_mut()), 8);
        }
        for y in 0..RustScreen::grid((*pane).base()).height() {
            for (x, byte) in b"one two three four".iter().copied().enumerate() {
                if x >= RustScreen::grid((*pane).base()).width() as usize {
                    break;
                }
                grid_set_cell(
                    RustScreen::grid_mut((*pane).base_mut()),
                    x as u_int,
                    RustScreen::grid((*pane).base()).history_size() + y,
                    &ascii(byte),
                );
            }
        }
        let source_pane_id = (*pane).pane_id();
        assert_eq!(
            (&mut *pane).set_mode(
                crate::window::window_pane_find_by_id(source_pane_id),
                WindowMode::Copy,
                None,
                None,
            ),
            0
        );
        (&mut *pane).active_mode_mut().map(|mode| mode.into_entry()).expect("pane is in copy mode")
    }
}

#[test]
fn cursor_format_callbacks_skip_an_absent_pane() {
    let tree = format_tree::default();
    unsafe {
        assert_eq!(window_copy_cursor_hyperlink_cb(&tree), None);
        assert_eq!(window_copy_cursor_word_cb(&tree), None);
        assert_eq!(window_copy_cursor_line_cb(&tree), None);
        assert_eq!(window_copy_search_match_cb(&tree), None);
    }
}

#[test]
fn refresh_keeps_the_snapshot_after_its_source_pane_is_destroyed() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let mut source = Window::new(100, "source", 18, 6);
        let mut source_pane = Pane::new(100, 18, 6, 100);
        source.add_pane(&mut source_pane);
        let source_ptr = source_pane.ptr();
        let source_id = (*source_ptr).pane_id();
        grid_set_cell(
            RustScreen::grid_mut((*source_ptr).base_mut()),
            0,
            0,
            &ascii(b'X'),
        );
        let pane = target.pane(0);
        assert_eq!(
            (&mut *pane).set_mode(
                crate::window::window_pane_find_by_id(source_id),
                WindowMode::Copy,
                None,
                None
            ),
            0
        );
        let entry = (&mut *pane).active_mode_mut().map(|mode| mode.into_entry()).expect("pane is in a mode");
        let backing_address = core::ptr::from_ref(window_copy_get_screen(entry).unwrap());
        drop(source);
        assert!(window_pane_find_by_id(source_id).is_none());
        assert!(entry.source_pane_ref().is_none());

        let parsed = Args::parse(c"send-keys -X refresh-from-pane");
        let mut state = window_copy_cmd_state {
            wme: entry,
            args: &*parsed.borrow(),
            wargs: &*parsed.borrow(),
            m: None,
            c: None,
            s: None,
            wl: None,
        };
        assert_eq!(
            window_copy_cmd_refresh_from_pane(&mut state),
            WINDOW_COPY_CMD_NOTHING
        );
        assert_eq!(
            core::ptr::from_ref(window_copy_get_screen(state.wme).unwrap()),
            backing_address
        );
        assert_eq!(
            grid_get_cell(
                RustScreen::grid(window_copy_get_screen(state.wme).unwrap()),
                0,
                0
            )
            .data
            .data[0],
            b'X'
        );
    }
}

#[test]
fn search_primitives_cover_plain_regex_wrapped_and_cell_position_paths() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(12, 4);
        let wme = open_copy(&mut target);
        let data = wme.state.copy_mode_data_mut().unwrap();
        let gd = RustScreen::grid_mut(
            &mut *(*data)
                .backing
                .as_deref_mut()
                .expect("copy mode has a backing screen"),
        );
        let mut needle = grid_create(3, 1, 0);
        for (x, byte) in b"two".iter().copied().enumerate() {
            grid_set_cell(&mut needle, x as u_int, 0, &ascii(byte));
        }
        assert_eq!(
            window_copy_search_lr(gd, &needle, gd.history_size(), 0, gd.width(), 0),
            Some(4)
        );
        assert_eq!(
            window_copy_search_rl(gd, &needle, gd.history_size(), 0, gd.width(), 0),
            Some(4)
        );
        assert!(window_copy_search_lr(gd, &needle, gd.history_size(), 5, gd.width(), 0).is_none());
        let mut upper = ascii(b'T');
        grid_set_cell(&mut needle, 0, 0, &upper);
        assert!(window_copy_search_lr(gd, &needle, gd.history_size(), 0, gd.width(), 1).is_none());
        upper.data.width = 2;
        grid_set_cell(&mut needle, 0, 0, &upper);
        assert!(window_copy_search_lr(gd, &needle, gd.history_size(), 0, gd.width(), 0).is_none());

        let sx = gd.width();
        assert_eq!(gd.cell_bytes(sx + 2, gd.history_size()).as_ref(), b" ");
        let mut bytes = vec![0];
        window_copy_stringify(gd, gd.history_size(), 0, gd.width(), &mut bytes);
        assert_eq!(
            CStr::from_bytes_with_nul(&bytes).unwrap().to_bytes(),
            b"one two thre"
        );
        let mut px = 0;
        let mut py = gd.history_size();
        window_copy_cstrtocellpos(gd, gd.width(), &mut px, &mut py, c"one two");
        assert_eq!((px, py), (0, gd.history_size() + 1));

        let reg = CompiledRegex::compile(c"t[a-z]+", REG_EXTENDED).unwrap();
        assert_eq!(
            window_copy_search_lr_regex(gd, gd.history_size(), 0, gd.width(), &reg),
            Some((4, 3))
        );
        assert_eq!(
            window_copy_search_rl_regex(gd, gd.history_size(), 0, gd.width(), &reg),
            Some((8, 4))
        );
    }
}

#[test]
fn command_handlers_exercise_copy_navigation_selection_and_search_states() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let wme = open_copy(&mut target);
        let parsed = Args::parse(c"display-message");
        let args = &*parsed.borrow();
        let mouse = mouse_event {
            valid: 0,
            ..Default::default()
        };
        for entry in &window_copy_cmd_table {
            let name = entry.command.to_bytes();
            if matches!(
                name,
                b"scroll-to-mouse"
                    | b"copy-mode"
                    | b"search-backward"
                    | b"search-forward"
                    | b"search-backward-incremental"
                    | b"search-forward-incremental"
                    | b"next-word"
                    | b"next-word-end"
                    | b"previous-word"
                    | b"select-word"
                    | b"selection-mode"
            ) {
                continue;
            }
            let Some(handler) = entry.f else { continue };
            wme.prefix = 2;
            let mut state = window_copy_cmd_state {
                wme,
                args,
                wargs: &*parsed.borrow(),
                m: Some(&mouse),
                c: None,
                s: None,
                wl: None,
            };
            let _ = handler(&mut state);
        }

        let data = wme.state.copy_mode_data_mut().unwrap();
        data.searchstr = Some(c"two".to_owned());
        data.searchtype = WINDOW_COPY_SEARCHDOWN as core::ffi::c_int;
        data.searchdirection = 1;
        assert!(window_copy_search_down(wme, 0) != 0);
        assert!(window_copy_search_up(wme, 0) != 0);
        window_copy_search_marks(wme, None, 0, 0);
        let data = wme.state.copy_mode_data_ref().unwrap();
        let _ = window_copy_match_at_cursor(data);
        window_copy_clear_marks(wme);
    }
}

#[test]
fn cursor_selection_scroll_and_mark_helpers_cover_boundary_states() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let wme = open_copy(&mut target);
        window_copy_update_cursor(wme, 4, 2);
        window_copy_start_selection(wme);
        window_copy_cursor_right(wme, 0);
        window_copy_cursor_left(wme);
        window_copy_cursor_down(wme, 0);
        window_copy_cursor_up(wme, 0);
        window_copy_cursor_end_of_line(wme);
        window_copy_cursor_start_of_line(wme);
        window_copy_cursor_back_to_indentation(wme);
        window_copy_cursor_next_word(wme, c" -_@");
        window_copy_cursor_previous_word(wme, c" -_@", 0);
        window_copy_cursor_next_word_end(wme, c" -_@", 0);
        let _ = window_copy_cursor_next_word_end_pos(wme, c" -_@");
        let _ = window_copy_cursor_previous_word_pos(wme, c" -_@");
        window_copy_rectangle_set(wme, 1);
        window_copy_rectangle_set(wme, 0);
        window_copy_other_end(wme);
        window_copy_scroll_up(wme, 3);
        window_copy_scroll_down(wme, 2);
        window_copy_scroll_to(wme, 3, 1, 1);
        let data = wme.state.copy_mode_data_mut().unwrap();
        data.mx = 2;
        data.my = 1;
        window_copy_jump_to_mark(wme);
        window_copy_clear_selection(wme);
        let data = wme.state.copy_mode_data_ref().unwrap();
        assert!(!data.screen.borrow().has_selection());
    }
}

#[test]
fn paragraph_line_word_and_offset_helpers_cover_modes_and_boundaries() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let pane = target.pane(0);
        let wme = open_copy(&mut target);

        window_copy_previous_paragraph(wme);
        window_copy_next_paragraph(wme);
        assert_eq!(
            window_copy_get_word(&mut *pane, 1, 0).unwrap().as_bytes(),
            b"one"
        );
        assert!(
            window_copy_get_line(&mut *pane, 0)
                .as_bytes()
                .starts_with(b"one two")
        );
        assert_eq!(window_copy_get_current_offset(&mut *pane), Some((24, 24)));

        assert_eq!(window_copy_line_number_width(wme), 0);
        window_copy_set_line_numbers(&mut *pane, 1);
        let data = wme.state.copy_mode_data_ref().unwrap();
        assert_eq!(data.line_numbers, 1);
        (*pane)
            .window_context()
            .unwrap()
            .options()
            .set_number(c"copy-mode-line-numbers", 1);
        assert!(window_copy_line_numbers_active(wme) != 0);
        let width = window_copy_line_number_width(wme);
        assert!(width >= 4);
        assert_eq!(window_copy_cursor_offset(wme, 0, 18), width);
        assert_eq!(window_copy_cursor_unoffset(wme, 0, 18), 0);
        assert_eq!(window_copy_cursor_unoffset(wme, width + 2, 18), 2);
        assert_eq!(window_copy_cursor_offset(wme, 99, 18), 17);
        window_copy_set_line_numbers(&mut *pane, 0);
        assert_eq!(window_copy_cursor_offset(wme, 3, 18), 3);
    }
}

#[test]
fn goto_line_and_page_helpers_cover_absolute_relative_and_clamped_positions() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let wme = open_copy(&mut target);

        window_copy_goto_line(wme, c"5");
        let data = wme.state.copy_mode_data_ref().unwrap();
        assert_eq!(data.oy, 5);
        window_copy_goto_line(wme, c"9999");
        let data = wme.state.copy_mode_data_ref().unwrap();
        assert_eq!(data.oy, 24);
        window_copy_goto_line(wme, c"bad");
        let data = wme.state.copy_mode_data_ref().unwrap();
        assert_eq!(data.oy, 24);

        window_copy_set_line_numbers(&mut *target.pane(0), 1);
        (*target.pane(0))
            .window_context()
            .unwrap()
            .options()
            .set_number(c"copy-mode-line-numbers", 2);
        let wme = (&mut *target.pane(0)).active_mode_mut().map(|mode| mode.into_entry()).expect("pane is in copy mode");
        window_copy_goto_line(wme, c"1");
        let data = wme.state.copy_mode_data_ref().unwrap();
        assert_eq!(data.oy, 24);
        window_copy_goto_line(wme, c"9999");
        let data = wme.state.copy_mode_data_ref().unwrap();
        assert_eq!(data.oy, 0);

        window_copy_pageup1(wme, 0);
        window_copy_pageup1(wme, 1);
        let _ = window_copy_pagedown1(wme, 0, 0);
        let _ = window_copy_pagedown1(wme, 1, 1);
    }
}

#[test]
fn search_motion_and_mark_helpers_cover_wrap_edges_and_match_extraction() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let wme = open_copy(&mut target);
        let data = wme.state.copy_mode_data_mut().unwrap();
        let backing = (*data)
            .backing
            .as_deref()
            .expect("copy mode has a backing screen");
        let gd = RustScreen::grid(backing);
        let last = gd.history_size() + gd.height() - 1;

        let (mut x, mut y) = (0, 0);
        window_copy_move_left(backing, &mut x, &mut y, 0);
        assert_eq!((x, y), (0, 0));
        window_copy_move_left(backing, &mut x, &mut y, 1);
        assert_eq!(y, last);
        window_copy_move_right(backing, &mut x, &mut y, 1);
        x = gd.width() - 1;
        y = last;
        window_copy_move_right(backing, &mut x, &mut y, 0);
        assert_eq!((x, y), (gd.width() - 1, last));

        let hsize = gd.history_size();
        data.searchstr = Some(c"two".to_owned());
        data.searchtype = WINDOW_COPY_SEARCHDOWN as core::ffi::c_int;
        assert_ne!(window_copy_search(wme, 1, 0), 0);
        let data = wme.state.copy_mode_data_ref().unwrap();
        let matched = window_copy_match_at_cursor(data).unwrap();
        assert_eq!(matched.as_bytes(), b"two");
        let at = window_copy_search_mark_at(data, data.cx, hsize - data.oy + data.cy).unwrap();
        let (start, end) = window_copy_match_start_end(data, at);
        assert!(start <= at && at <= end);
        window_copy_clear_marks(wme);
        let data = wme.state.copy_mode_data_ref().unwrap();
        assert!(window_copy_match_at_cursor(data).is_none());
    }
}

#[test]
fn clipping_case_and_position_helpers_cover_small_and_large_inputs() {
    assert_eq!(window_copy_clip_width(10, 0, 8, 2), 10);
    assert_eq!(window_copy_clip_width(2, 4, 8, 2), 2);
    assert_eq!(window_copy_clip_width(10, 7, 8, 2), 9);
    assert_eq!(window_copy_is_lowercase(c"lower-123"), 1);
    assert_eq!(window_copy_is_lowercase(c"Mixed"), 0);
}

#[test]
fn selection_extraction_covers_forward_reverse_rectangle_and_line_modes() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let wme = open_copy(&mut target);
        let data = wme.state.copy_mode_data_mut().unwrap();
        let absolute = RustScreen::grid(
            (*data)
                .backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size();
        data.oy = 0;

        window_copy_update_cursor(wme, 1, 0);
        window_copy_start_selection(wme);
        let data = wme.state.copy_mode_data_mut().unwrap();
        data.endselx = 7;
        data.endsely = absolute;
        assert!(window_copy_set_selection(wme, 0, 1) != 0);
        assert!(window_copy_get_selection(wme).is_some());

        let data = wme.state.copy_mode_data_mut().unwrap();
        data.selx = 8;
        data.sely = absolute + 2;
        data.endselx = 2;
        data.endsely = absolute;
        assert!(window_copy_set_selection(wme, 0, 1) != 0);
        let reverse = window_copy_get_selection(wme).unwrap();
        assert!(reverse.len() <= 18 * 3);

        let data = wme.state.copy_mode_data_mut().unwrap();
        data.rectflag = 1;
        data.cx = 6;
        data.cy = 2;
        data.selx = 2;
        data.sely = absolute;
        data.endselx = 6;
        data.endsely = absolute + 2;
        data.cursordrag = CURSORDRAG_ENDSEL;
        window_copy_set_selection(wme, 0, 1);
        assert!(!window_copy_get_selection(wme).unwrap().is_empty());
        let data = wme.state.copy_mode_data_mut().unwrap();
        data.cursordrag = CURSORDRAG_SEL;
        window_copy_set_selection(wme, 0, 1);
        assert!(!window_copy_get_selection(wme).unwrap().is_empty());

        let data = wme.state.copy_mode_data_mut().unwrap();
        data.rectflag = 0;
        data.lineflag = LINE_SEL_LEFT_RIGHT;
        data.selx = 0;
        data.sely = absolute;
        data.endselx = 17;
        data.endsely = absolute + 1;
        window_copy_set_selection(wme, 0, 1);
        assert!(!window_copy_get_selection(wme).unwrap().is_empty());

        let data = wme.state.copy_mode_data_mut().unwrap();
        data.sely = 0;
        data.endsely = 1;
        data.oy = 0;
        assert!(window_copy_set_selection(wme, 0, 1) <= 1);
        window_copy_clear_selection(wme);
        assert!(window_copy_get_selection(wme).is_none());
    }
}

#[test]
fn dynamic_backing_addition_covers_literal_parsed_and_history_growth_paths() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(20, 5);
        let pane = target.pane(0);
        let wme = open_copy(&mut target);
        let data = wme.state.copy_mode_data_ref().unwrap();
        let before = RustScreen::grid(
            (*data)
                .backing
                .as_deref()
                .expect("copy mode has a backing screen"),
        )
        .history_size();
        window_copy_add(
            &mut *pane,
            0,
            c"literal %s",
            crate::fmt_args![c"line".as_ptr()],
        );
        window_copy_add(&mut *pane, 0, c"second", crate::fmt_args![]);
        let data = wme.state.copy_mode_data_ref().unwrap();
        assert!(data.backing_written != 0);
        assert!(
            RustScreen::grid(
                (*data)
                    .backing
                    .as_deref()
                    .expect("copy mode has a backing screen")
            )
            .history_size()
                >= before
        );

        window_copy_resize(wme, 12, 3);
        window_copy_resize(wme, 30, 8);
        window_copy_style_changed(wme);
        let data = wme.state.copy_mode_data_ref().unwrap();
        assert_eq!(RustScreen::grid(&data.screen.borrow()).width(), 30);
    }
}

#[test]
fn scrollbar_motion_covers_slider_clamps_both_directions_and_selection_updates() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let pane = target.pane(0);
        let wme = open_copy(&mut target);
        (*pane).publish_slider(PaneScrollbarSlider { sb_slider_y: 0, sb_slider_h: 2 });
        let geometry = (*pane).geometry();
        (*pane).set_position(geometry.xoff, 1);

        window_copy_scroll(&mut *pane, 1, 0, 0, 0);
        window_copy_scroll(&mut *pane, 0, 100, 0, 0);
        window_copy_start_selection(wme);
        let data = wme.state.copy_mode_data_mut().unwrap();
        data.rectflag = 1;
        window_copy_scroll(&mut *pane, 1, 3, 1, 0);
        let data = wme.state.copy_mode_data_mut().unwrap();
        data.rectflag = 0;
        data.searchstr = Some(c"two".to_owned());
        window_copy_search_marks(wme, None, 0, 0);
        let data = wme.state.copy_mode_data_mut().unwrap();
        data.timeout = 0;
        window_copy_scroll(&mut *pane, 1, 2, 0, 0);
        let data = wme.state.copy_mode_data_ref().unwrap();
        assert!(
            data.oy
                <= RustScreen::grid(
                    (*data)
                        .backing
                        .as_deref()
                        .expect("copy mode has a backing screen")
                )
                .history_size()
        );
    }
}

#[test]
fn scrollbar_exit_restores_the_previous_mode_then_the_base_screen() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let pane = &mut *target.pane(0);
        assert!(window_copy_get_current_offset(pane).is_none());
        window_copy_scroll(pane, 0, 100, 0, 1);
        assert_eq!(
            (pane).set_mode( None, WindowMode::View, None, None),
            0
        );
        let wme = open_copy(&mut target);
        window_copy_goto_line(wme, c"10");

        let pane = &mut *target.pane(0);
        pane.publish_slider(PaneScrollbarSlider { sb_slider_y: 0, sb_slider_h: 1 });
        window_copy_scroll(pane, 0, 100, 0, 1);
        assert_eq!(
            (pane).active_mode().unwrap().mode(),
            WindowMode::View
        );
        assert_eq!(pane.showing_base(), false);

        window_copy_scroll(pane, 0, 100, 0, 1);
        assert!((pane).active_mode().is_none());
        assert!((pane).active_mode_mut().map(|mode| mode.into_entry()).is_none());
        assert!(window_copy_get_current_offset(pane).is_none());
        assert_eq!(pane.showing_base(), true);

        assert_eq!(
            (pane).set_mode( None, WindowMode::Clock, None, None),
            0
        );
        assert!(window_copy_get_current_offset(pane).is_none());
    }
}

#[test]
fn drag_scroll_timers_leave_covering_modes_and_their_state_untouched() {
    use crate::reactor::{Reactor, current};

    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let mut pane = target.state().pane_ref().unwrap();
        let mut source_window = Window::new(100, "source", 18, 6);
        let mut source_pane = Pane::new(100, 18, 6, 100);
        source_window.add_pane(&mut source_pane);
        let source = source_window.reference().pane_by_id(100).unwrap();
        for (under, over) in [
            (WindowMode::Copy, WindowMode::Clock),
            (WindowMode::Copy, WindowMode::View),
            (WindowMode::View, WindowMode::Copy),
        ] {
            if under == WindowMode::Copy {
                open_copy(&mut target);
            } else {
                (pane.get_mut().unwrap()).set_mode( None, under, None, None);
            }
            let timer = {
                let entry = (pane.get_mut().unwrap()).active_mode_mut().map(|mode| mode.into_entry()).unwrap();
                let data = entry.state.copy_mode_data_mut().unwrap();
                data.cy = 0;
                data.oy = 0;
                data.dragtimer.arm(timeval::from_secs(0));
                data.dragtimer
            };
            let source_pane = (over == WindowMode::Copy).then(|| source.id());
            (pane.get_mut().unwrap()).set_mode(
                source_pane.and_then(crate::window::window_pane_find_by_id),
                over,
                None,
                None,
            );
            current().run_once();
            assert!(!timer.is_armed());
            let wp = pane.get_mut().unwrap();
            assert_eq!(wp.active_mode().unwrap().mode(), over);
            assert_eq!(wp.find_mode_mut(under).map(|mode| mode.into_entry()).unwrap().mode(), under);
            let data = wp.find_mode_mut(under).map(|mode| mode.into_entry()).unwrap().state.copy_mode_data_ref().unwrap();
            assert_eq!((data.cy, data.oy), (0, 0));
            if over != WindowMode::Clock {
                let data = wp.active_mode().unwrap().state.copy_mode_data_ref().unwrap();
                assert!(!data.dragtimer.is_armed());
            }
            (pane.get_mut().unwrap()).reset_modes();
        }
    }
}

#[test]
fn drag_scroll_timer_moves_an_active_copy_mode_and_rearms() {
    use crate::reactor::{Reactor, current};

    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let entry = open_copy(&mut target);
        let data = entry.state.copy_mode_data_mut().unwrap();
        data.cy = 0;
        data.oy = 0;
        data.dragtimer.arm(timeval::from_secs(0));
        current().run_once();
        let mut pane = target.state().pane_ref().unwrap();
        let entry = (pane.get_mut().unwrap()).active_mode_mut().map(|mode| mode.into_entry()).unwrap();
        let data = entry.state.copy_mode_data_mut().unwrap();
        assert_eq!(data.oy, 1);
        assert!(data.dragtimer.is_armed());
        data.dragtimer.disarm();
    }
}

#[test]
fn mouse_drag_callbacks_release_the_timer_and_tolerate_a_closed_mode() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        open_copy(&mut target);
        let mut pane = target.state().pane_ref().unwrap();
        pane.window().unwrap().options().set_number(
            c"copy-mode-line-numbers",
            WINDOW_COPY_LINE_NUMBERS_ABSOLUTE as i64,
        );
        let mut client = crate::tests::test_fixtures::zeroed_client();
        let mut mouse = mouse_event {
            valid: 1,
            s: 0,
            w: -1,
            wp: -1,
            x: 6,
            y: 2,
            lx: 6,
            ly: 2,
            ..Default::default()
        };
        window_copy_start_drag(Some(client.as_client_mut()), &mouse);
        let update = client.as_tty().mouse_drag_update.clone().unwrap();
        let release = client.as_tty().mouse_drag_release.clone().unwrap();
        mouse.x = 10;
        mouse.y = 0;
        update(client.as_client_mut(), &mouse);
        {
            let entry = (pane.get_mut().unwrap()).active_mode_mut().map(|mode| mode.into_entry()).unwrap();
            let data = entry.state.copy_mode_data_ref().unwrap();
            assert!(data.screen.borrow().has_selection());
            assert!(data.dragtimer.is_armed());
            assert_eq!(data.oy, 1);
        }
        release(client.as_client_mut(), &mouse);
        let entry = (pane.get_mut().unwrap()).active_mode_mut().map(|mode| mode.into_entry()).unwrap();
        assert!(
            !entry
                .state
                .copy_mode_data_ref()
                .unwrap()
                .dragtimer
                .is_armed()
        );
        (pane.get_mut().unwrap()).reset_mode();
        update(client.as_client_mut(), &mouse);
        release(client.as_client_mut(), &mouse);
        drop(target);
        update(client.as_client_mut(), &mouse);
        release(client.as_client_mut(), &mouse);
    }
}

#[test]
fn search_commands_repeat_in_both_directions_after_format_expansion() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let wme = open_copy(&mut target);
        let flags = Args::parse(c"send-keys -F");
        let pattern = Args::parse(c"display-message '#{?pane_in_mode,two,missing}'");
        type SearchHandler = unsafe fn(&mut window_copy_cmd_state<'_>) -> window_copy_cmd_action;
        let handlers: [(SearchHandler, window_copy_search_type, i32, u32); 4] = [
            (window_copy_cmd_search_forward, WINDOW_COPY_SEARCHDOWN, 1, 3),
            (
                window_copy_cmd_search_forward_text,
                WINDOW_COPY_SEARCHDOWN,
                0,
                3,
            ),
            (window_copy_cmd_search_backward, WINDOW_COPY_SEARCHUP, 1, 0),
            (
                window_copy_cmd_search_backward_text,
                WINDOW_COPY_SEARCHUP,
                0,
                0,
            ),
        ];
        for (handler, direction, regex, expected_y) in handlers {
            wme.prefix = 2;
            let data = wme.state.copy_mode_data_mut().unwrap();
            data.cx = 0;
            data.cy = 2;
            data.oy = 0;
            data.timeout = 1;
            let mut state = window_copy_cmd_state {
                wme,
                args: &flags.borrow(),
                wargs: &pattern.borrow(),
                m: None,
                c: None,
                s: None,
                wl: None,
            };
            assert_eq!(handler(&mut state), WINDOW_COPY_CMD_NOTHING);
            let data = state.wme.state.copy_mode_data_ref().unwrap();
            assert_eq!(data.searchstr.as_deref(), Some(c"two"));
            assert_eq!(data.searchtype, direction as i32);
            assert_eq!(data.searchregex, regex);
            assert_eq!(data.timeout, 0);
            let expected_x = if direction == WINDOW_COPY_SEARCHDOWN {
                7
            } else {
                4
            };
            assert_eq!((data.cx, data.cy, data.oy), (expected_x, expected_y, 0));
        }
    }
}

#[test]
fn incremental_search_restarts_from_the_saved_row_when_the_pattern_changes() {
    let _guard = globals();
    unsafe {
        type SearchHandler = unsafe fn(&mut window_copy_cmd_state<'_>) -> window_copy_cmd_action;
        let handlers: [(SearchHandler, (u32, u32), (u32, u32)); 2] = [
            (window_copy_cmd_search_forward_incremental, (7, 2), (13, 3)),
            (window_copy_cmd_search_backward_incremental, (4, 1), (8, 2)),
        ];
        let flags = Args::parse(c"send-keys");
        for (handler, first_match, changed_match) in handlers {
            let mut target = Target::new(18, 6);
            let wme = open_copy(&mut target);
            let data = wme.state.copy_mode_data_mut().unwrap();
            data.cx = 0;
            data.cy = 2;
            data.oy = 0;
            for (command, expected, action) in [
                (
                    c"display-message '=two'",
                    Some(first_match),
                    WINDOW_COPY_CMD_NOTHING,
                ),
                (
                    c"display-message '=three'",
                    Some(changed_match),
                    WINDOW_COPY_CMD_REDRAW,
                ),
                (c"display-message ''", None, WINDOW_COPY_CMD_REDRAW),
            ] {
                let pattern = Args::parse(command);
                let mut state = window_copy_cmd_state {
                    wme,
                    args: &flags.borrow(),
                    wargs: &pattern.borrow(),
                    m: None,
                    c: None,
                    s: None,
                    wl: None,
                };
                assert_eq!(handler(&mut state), action);
                let data = state.wme.state.copy_mode_data_ref().unwrap();
                assert_eq!((data.searchx, data.searchy, data.searcho), (0, 2, 0));
                if let Some(expected) = expected {
                    assert_eq!((data.cx, data.cy), expected);
                    assert!(data.searchmark.iter().any(|mark| *mark != 0));
                } else {
                    assert!(data.searchmark.is_empty());
                    assert_eq!(data.searchstr.as_deref(), Some(c"three"));
                }
            }
        }
    }
}

#[test]
fn repositioning_the_viewport_keeps_the_absolute_cursor_line() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let wme = open_copy(&mut target);
        let data = wme.state.copy_mode_data_mut().unwrap();
        data.cy = 2;
        data.oy = 5;
        let absolute = RustScreen::grid(data.backing.as_deref().unwrap()).history_size() + data.cy - data.oy;
        let parsed = Args::parse(c"display-message");
        let mut state = window_copy_cmd_state {
            wme,
            args: &parsed.borrow(),
            wargs: &parsed.borrow(),
            m: None,
            c: None,
            s: None,
            wl: None,
        };
        for row in [5, 2, 0] {
            assert_eq!(
                window_copy_cmd_scroll_to(&mut state, row),
                WINDOW_COPY_CMD_REDRAW
            );
            let data = state.wme.state.copy_mode_data_ref().unwrap();
            assert_eq!(data.cy, row);
            assert_eq!(
                RustScreen::grid(data.backing.as_deref().unwrap()).history_size() + data.cy - data.oy,
                absolute
            );
        }
        let data = state.wme.state.copy_mode_data_mut().unwrap();
        data.cy = 2;
        data.oy = 0;
        window_copy_cmd_scroll_to(&mut state, 0);
        let data = state.wme.state.copy_mode_data_ref().unwrap();
        assert_eq!((data.cy, data.oy), (2, 0));
    }
}

#[test]
fn repeated_navigation_checks_cancellation_after_reaching_live_output() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let wme = open_copy(&mut target);
        let parsed = Args::parse(c"display-message");
        wme.prefix = 2;
        let data = wme.state.copy_mode_data_mut().unwrap();
        data.cy = 2;
        data.oy = 3;
        data.scroll_exit = 1;
        let mut state = window_copy_cmd_state {
            wme,
            args: &parsed.borrow(),
            wargs: &parsed.borrow(),
            m: None,
            c: None,
            s: None,
            wl: None,
        };
        assert_eq!(
            window_copy_cmd_scroll_down(&mut state),
            WINDOW_COPY_CMD_NOTHING
        );
        assert_eq!(state.wme.state.copy_mode_data_ref().unwrap().oy, 1);
        assert_eq!(
            window_copy_cmd_scroll_down(&mut state),
            WINDOW_COPY_CMD_CANCEL
        );
        assert_eq!(state.wme.state.copy_mode_data_ref().unwrap().oy, 0);
        state.wme.state.copy_mode_data_mut().unwrap().scroll_exit = 0;
        assert_eq!(
            window_copy_cmd_scroll_down(&mut state),
            WINDOW_COPY_CMD_NOTHING
        );
        assert_eq!(
            window_copy_cmd_scroll_down_and_cancel(&mut state),
            WINDOW_COPY_CMD_CANCEL
        );
        let data = state.wme.state.copy_mode_data_mut().unwrap();
        data.cy = 5;
        assert_eq!(
            window_copy_cmd_cursor_down_and_cancel(&mut state),
            WINDOW_COPY_CMD_CANCEL
        );
        for half_page in [false, true] {
            let data = state.wme.state.copy_mode_data_mut().unwrap();
            data.cy = 2;
            data.oy = 5;
            data.scroll_exit = 1;
            let action = if half_page {
                window_copy_cmd_halfpage_down(&mut state)
            } else {
                window_copy_cmd_page_down(&mut state)
            };
            assert_eq!(action, WINDOW_COPY_CMD_CANCEL);
            assert_eq!(state.wme.state.copy_mode_data_ref().unwrap().oy, 0);
        }
    }
}

#[test]
fn repeating_and_reversing_search_preserves_the_remembered_direction() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let wme = open_copy(&mut target);
        let parsed = Args::parse(c"display-message");
        for direction in [WINDOW_COPY_SEARCHUP, WINDOW_COPY_SEARCHDOWN] {
            for reverse in [false, true] {
                wme.prefix = 2;
                window_copy_clear_marks(wme);
                let data = wme.state.copy_mode_data_mut().unwrap();
                data.cx = 0;
                data.cy = 2;
                data.oy = 0;
                data.searchtype = direction as i32;
                data.searchregex = 0;
                data.searchstr = Some(c"two".to_owned());
                let mut state = window_copy_cmd_state {
                    wme,
                    args: &parsed.borrow(),
                    wargs: &parsed.borrow(),
                    m: None,
                    c: None,
                    s: None,
                    wl: None,
                };
                let action = if reverse {
                    window_copy_cmd_search_reverse(&mut state)
                } else {
                    window_copy_cmd_search_again(&mut state)
                };
                assert_eq!(action, WINDOW_COPY_CMD_NOTHING);
                let data = state.wme.state.copy_mode_data_ref().unwrap();
                let expected = if (direction == WINDOW_COPY_SEARCHDOWN) != reverse {
                    (7, 3)
                } else {
                    (4, 0)
                };
                assert_eq!((data.cx, data.cy), expected);
                assert_eq!(data.searchtype, direction as i32);
            }
        }
    }
}

fn replace_copy_text(wme: &mut window_mode_entry, lines: &[&[u8]]) {
    let data = wme.state.copy_mode_data_mut().unwrap();
    let (sx, sy) = data.screen.borrow().size();
    let mut backing = Box::new(RustScreen::new_with_server_options(sx, sy, 0));
    for (y, line) in lines.iter().enumerate() {
        for (x, byte) in line.iter().copied().enumerate() {
            grid_set_cell(
                RustScreen::grid_mut(&mut backing),
                x as u32,
                y as u32,
                &ascii(byte),
            );
        }
    }
    data.backing = Some(backing);
    data.cx = 0;
    data.cy = 0;
    data.oy = 0;
}

#[test]
fn bracket_commands_match_nested_pairs_across_lines() {
    let _guard = globals();
    unsafe {
        for keys in [MODEKEY_EMACS, MODEKEY_VI] {
            let mut target = Target::new(18, 6);
            target
                .state()
                .window()
                .unwrap()
                .options()
                .set_number(c"mode-keys", keys as i64);
            let wme = open_copy(&mut target);
            replace_copy_text(wme, &[b"{[(", b"x)]}"]);
            let parsed = Args::parse(c"display-message");
            wme.prefix = 1;
            for (next, start, expected) in [
                (true, (0, 0), (3, 1)),
                (true, (1, 0), (2, 1)),
                (true, (2, 0), (1, 1)),
                (false, (3, 1), (0, 0)),
                (false, (2, 1), (1, 0)),
                (false, (1, 1), (2, 0)),
            ] {
                let data = wme.state.copy_mode_data_mut().unwrap();
                (data.cx, data.cy) = start;
                let mut state = window_copy_cmd_state {
                    wme,
                    args: &parsed.borrow(),
                    wargs: &parsed.borrow(),
                    m: None,
                    c: None,
                    s: None,
                    wl: None,
                };
                let action = if next {
                    window_copy_cmd_next_matching_bracket(&mut state)
                } else {
                    window_copy_cmd_previous_matching_bracket(&mut state)
                };
                assert_eq!(action, WINDOW_COPY_CMD_NOTHING);
                let data = state.wme.state.copy_mode_data_ref().unwrap();
                assert_eq!((data.cx, data.cy), expected);
            }
        }
    }
}

#[test]
fn vi_bracket_scanning_crosses_only_wrapped_lines_and_reverses_on_a_closer() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        target
            .state()
            .window()
            .unwrap()
            .options()
            .set_number(c"mode-keys", MODEKEY_VI as i64);
        let wme = open_copy(&mut target);
        let parsed = Args::parse(c"display-message");
        for wrapped in [false, true] {
            replace_copy_text(wme, &[b"word", b"(x)"]);
            wme.prefix = 1;
            if wrapped {
                let data = wme.state.copy_mode_data_mut().unwrap();
                crate::grid::grid_mark_wrapped(
                    RustScreen::grid_mut(data.backing.as_deref_mut().unwrap()), 0,
                );
            }
            let mut state = window_copy_cmd_state {
                wme,
                args: &parsed.borrow(),
                wargs: &parsed.borrow(),
                m: None,
                c: None,
                s: None,
                wl: None,
            };
            window_copy_cmd_next_matching_bracket(&mut state);
            let data = state.wme.state.copy_mode_data_ref().unwrap();
            assert_eq!((data.cx, data.cy), if wrapped { (2, 1) } else { (0, 0) });
            if wrapped {
                window_copy_cmd_next_matching_bracket(&mut state);
                let data = state.wme.state.copy_mode_data_ref().unwrap();
                assert_eq!((data.cx, data.cy), (0, 1));
            }
        }
    }
}

#[test]
fn recentre_cycles_preserve_the_history_line_before_jumping_to_live_output() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let wme = open_copy(&mut target);
        let data = wme.state.copy_mode_data_mut().unwrap();
        data.cy = 1;
        data.oy = 8;
        let absolute = RustScreen::grid(data.backing.as_deref().unwrap()).history_size() + data.cy - data.oy;
        let parsed = Args::parse(c"display-message");
        let mut state = window_copy_cmd_state {
            wme,
            args: &parsed.borrow(),
            wargs: &parsed.borrow(),
            m: None,
            c: None,
            s: None,
            wl: None,
        };
        for row in [2, 0, 5, 2] {
            assert_eq!(
                window_copy_cmd_recentre_top_bottom(&mut state),
                WINDOW_COPY_CMD_REDRAW
            );
            let data = state.wme.state.copy_mode_data_ref().unwrap();
            assert_eq!(data.cy, row);
            assert_eq!(
                RustScreen::grid(data.backing.as_deref().unwrap()).history_size() + data.cy - data.oy,
                absolute
            );
        }
        assert_eq!(
            window_copy_cmd_history_bottom(&mut state),
            WINDOW_COPY_CMD_REDRAW
        );
        let data = state.wme.state.copy_mode_data_ref().unwrap();
        assert_eq!((data.cx, data.cy, data.oy), (18, 5, 0));
    }
}

#[test]
fn emacs_word_and_line_selection_preserves_single_cell_empty_spans() {
    let _guard = globals();
    unsafe {
        let cases: [(&[&[u8]], u32, u32, bool, &[u8]); 4] = [
            (&[b"alpha beta"], 7, 1, true, b"beta"),
            (&[b"alpha-beta"], 7, 1, true, b"beta"),
            (&[b"a b"], 2, 1, true, b""),
            (
                &[b"first", b"second", b"third"],
                3,
                2,
                false,
                b"first\nsecond",
            ),
        ];
        for (lines, x, prefix, word, expected) in cases {
            let mut target = Target::new(18, 6);
            let mut session = target.state().session().unwrap().clone();
            let wme = open_copy(&mut target);
            replace_copy_text(wme, lines);
            wme.prefix = prefix;
            wme.state.copy_mode_data_mut().unwrap().cx = x;
            let parsed = Args::parse(c"display-message");
            let mut state = window_copy_cmd_state {
                wme,
                args: &parsed.borrow(),
                wargs: &parsed.borrow(),
                m: None,
                c: None,
                s: Some(session.as_session_mut()),
                wl: None,
            };
            let action = if word {
                window_copy_cmd_select_word(&mut state)
            } else {
                window_copy_cmd_select_line(&mut state)
            };
            assert_eq!(action, WINDOW_COPY_CMD_REDRAW);
            assert_eq!(window_copy_get_selection(state.wme).unwrap(), expected);
        }
    }
}

#[test]
fn a_copy_display_owner_survives_mode_teardown_during_a_write() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let pane = target.pane(0);
        let entry = open_copy(&mut target);
        let display = entry.state.copy_mode_data_ref().unwrap().screen.clone();
        let weak = display.downgrade();
        let mut writer = RustScreenWriteCtx::on_shared_screen(&display);
        (&mut *pane).reset_mode();
        assert!((&mut *pane).active_mode_mut().map(|mode| mode.into_entry()).is_none());
        writer.cursormove(0, 0, 0);
        writer.puts(&grid_default_cell, c"retained display", &[]);
        writer.finish();
        assert_eq!(
            RustScreen::grid(&display.borrow()).cell(0, 0).data.data[0],
            b'r'
        );
        let base = RustScreen::grid((*pane).base());
        assert_eq!(base.cell(0, base.history_size()).data.data[0], b'o');
        drop(display);
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn pane_style_changes_can_read_copy_selection_formats_during_redraw() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let pane = target.pane(0);
        let entry = open_copy(&mut target);
        entry.state.copy_mode_data_mut().unwrap().line_numbers = 0;
        (*pane).options_ref().set_string(
            c"window-style",
            0,
            c"fg=#{?selection_present,red,blue}",
            &[],
        );
        *(*pane).flags_mut() |= crate::window::PANE_STYLECHANGED;
        window_copy_redraw_screen(entry);
        assert_eq!(*(*pane).flags() & crate::window::PANE_STYLECHANGED, 0);
        assert_eq!((*pane).styles().cached_gc.fg, 4);
        window_copy_start_selection(entry);
        window_copy_cursor_right(entry, 0);
        *(*pane).flags_mut() |= crate::window::PANE_STYLECHANGED;
        window_copy_redraw_screen(entry);
        assert_eq!((*pane).styles().cached_gc.fg, 1);
    }
}

#[test]
fn copy_search_and_word_motion_follow_the_panes_current_window() {
    let _guard = globals();
    unsafe {
        let mut target = Target::new(18, 6);
        let mut pane = target.state().pane_ref().unwrap();
        let original = pane.window().unwrap();
        original
            .options()
            .set_number(c"mode-keys", MODEKEY_EMACS as i64);
        let entry = open_copy(&mut target);
        replace_copy_text(entry, &[b"alpha beta"]);
        assert_eq!(window_copy_key_table(entry), c"copy-mode");
        let destination = Window::new(100, "destination", 18, 6);
        let destination_owner = destination.reference();
        let mut owned = crate::window::window_panes_take(
            &mut original.as_window_mut(),
            &crate::window::window_pane_find_by_id(pane.id()).expect("the pane exists"),
        )
        .unwrap();
        crate::window::window_pane_set_window_ref(owned.as_pane_mut(), Some(&destination_owner));
        crate::window::window_panes_insert_tail(&mut destination_owner.as_window_mut(), owned);

        for (keys, table, word_end, search_end) in [
            (MODEKEY_VI, c"copy-mode-vi", 4, 6),
            (MODEKEY_EMACS, c"copy-mode", 5, 10),
        ] {
            destination.options().set_number(c"mode-keys", keys as i64);
            let entry = (pane.get_mut().unwrap()).active_mode_mut().map(|mode| mode.into_entry()).unwrap();
            assert_eq!(window_copy_key_table(entry), table);
            let data = entry.state.copy_mode_data_mut().unwrap();
            (data.cx, data.cy, data.oy) = (0, 0, 0);
            window_copy_cursor_next_word_end(entry, c" ", 0);
            let data = entry.state.copy_mode_data_mut().unwrap();
            assert_eq!((data.cx, data.cy), (word_end, 0));
            (data.cx, data.cy, data.oy) = (0, 0, 0);
            data.searchstr = Some(c"beta".to_owned());
            assert_eq!(window_copy_search(entry, 1, 0), 1);
            let data = entry.state.copy_mode_data_ref().unwrap();
            assert_eq!((data.cx, data.cy), (search_end, 0));
            assert_eq!(pane.get().unwrap().query(), Some(c"beta"));
        }
        (pane.get_mut().unwrap()).reset_mode();
    }
}

#[test]
fn retiring_a_pane_resizes_and_releases_a_covered_copy_mode() {
    retire_covered_copy_mode(false);
}

#[test]
fn destroying_all_panes_releases_the_window_borrow_before_resizing_modes() {
    retire_covered_copy_mode(true);
}

fn retire_covered_copy_mode(all: bool) {
    let _guard = globals();
    unsafe {
        let mut window = Window::new(100, "retiring", 18, 6);
        let mut fixture = Pane::new(100, 18, 6, 100);
        window.add_pane(&mut fixture);
        let mut pane = window_pane_find_by_id(100).unwrap();
        window
            .options()
            .set_number(c"mode-keys", MODEKEY_EMACS as i64);
        (pane.get_mut().unwrap()).set_mode(
            crate::window::window_pane_find_by_id(100),
            WindowMode::Copy,
            None,
            None,
        );
        let entry = (pane.get_mut().unwrap()).active_mode_mut().map(|mode| mode.into_entry()).unwrap();
        replace_copy_text(entry, &[b"hello"]);
        let data = entry.state.copy_mode_data_mut().unwrap();
        data.cx = 10;
        data.line_numbers = 0;
        let display = data.screen.clone();
        let observer = display.downgrade();
        (pane.get_mut().unwrap()).set_mode( None, WindowMode::Clock, None, None);
        assert_eq!(pane.get().unwrap().mode_count(), 2);
        window.options().set_number(c"mode-keys", MODEKEY_VI as i64);
        window.options().set_string(
            c"copy-mode-position-format",
            0,
            c"#{copy_cursor_x}:#{window_name}",
            fmt_args![],
        );
        let window_owner = window.reference();
        if all {
            window_owner.destroy_panes();
        } else {
            window_owner
                .remove_pane(&crate::window::window_pane_find_by_id(100).expect("the pane exists"));
        }
        assert!(pane.get().is_none());
        assert!(window_pane_find_by_id(100).is_none());
        assert_eq!(display.borrow().cursor(), (4, 0));
        assert_eq!(display.borrow().size(), (18, 6));
        let text = crate::grid::grid_string_cells(display.borrow().grid(), 0, 0, 10, None, 0, None);
        assert_eq!(text.as_c_str(), c"4:retiring");
        drop(display);
        assert!(observer.upgrade().is_none());
    }
}

#[test]
fn copy_cursor_and_line_number_options_follow_the_owning_pane_window() {
    let _guard = globals();
    let mut target = Target::new(20, 6);
    let source_index = target.add_window(1, 20, 6);
    let state = target.state();
    let mut pane = state.pane_ref().unwrap();
    unsafe {
        let options = pane.window().unwrap().options();
        let source_options = (*target.window(source_index)).options_ref().clone();
        options.set_number(c"mode-keys", MODEKEY_VI as i64);
        options.set_number(c"copy-mode-line-numbers", 2);
        source_options.set_number(c"mode-keys", MODEKEY_EMACS as i64);
        source_options.set_number(c"copy-mode-line-numbers", 4);
        let source = &mut *target.pane(source_index);
        let mut writer = RustScreenWriteCtx::on_screen(source.base_mut());
        writer.puts(&grid_default_cell, c"abcd ef", fmt_args![]);
        writer.cursormove(0, 0, 0);
        writer.finish();
        let source_id = source.pane_id();
        assert_eq!(
            (pane.get_mut().unwrap()).set_mode(
                crate::window::window_pane_find_by_id(source_id),
                WindowMode::Copy,
                Some(&state),
                None,
            ),
            0
        );
        let entry = (pane.get_mut().unwrap()).active_mode_mut().map(|mode| mode.into_entry()).unwrap();
        let data = entry.state.copy_mode_data_mut().unwrap();
        data.line_numbers = 1;
        data.cx = 0;
        data.cy = 0;
        assert_eq!(window_copy_line_number_mode(entry), 2);
        assert_eq!(window_copy_cursor_limit(entry, 0, 0), 6);
        assert_eq!(window_copy_cursor_next_word_end_pos(entry, c" "), (3, 0));
        options.set_number(c"mode-keys", MODEKEY_EMACS as i64);
        options.set_number(c"copy-mode-line-numbers", 1);
        assert_eq!(window_copy_line_number_mode(entry), 1);
        assert_eq!(window_copy_cursor_limit(entry, 0, 0), 7);
        assert_eq!(window_copy_cursor_next_word_end_pos(entry, c" "), (4, 0));
    }
}

#[test]
fn a_copy_redraw_ignores_a_retired_pane_before_borrowing_its_screen() {
    let _guard = globals();
    let mut target = Target::new(20, 6);
    let state = target.state();
    let mut pane = state.pane_ref().unwrap();
    unsafe {
        let pane_id = pane.id();
        assert_eq!(
            (pane.get_mut().unwrap()).set_mode(
                crate::window::window_pane_find_by_id(pane_id),
                WindowMode::Copy,
                Some(&state),
                None,
            ),
            0
        );
        let mut entry = pane.get_mut().unwrap().take_test_mode().unwrap();
        let display = entry.state.copy_mode_data_ref().unwrap().screen.clone();
        drop(target);
        assert!(pane.get().is_none());
        let read = display.borrow();
        let cursor = read.cursor();
        window_copy_redraw_lines(&mut entry, 0, 1);
        assert_eq!(read.cursor(), cursor);
        drop(read);
        entry.mode().free(&mut entry);
    }
}

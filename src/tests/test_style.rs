use super::*;
use crate::options::options_set_string;
use crate::tests::test_fixtures::{Options, globals, seen};
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::null_mut;

/// A style holding the module's defaults, as `style_set` makes one.
fn blank() -> Box<style> {
    let mut sy = Box::new(style::default());
    style_set(&mut sy, &grid_default_cell);
    sy
}

/// A base cell with colours and attributes of its own, so that `default`
/// and a colour of `default` can be told apart from the style's own.
fn base() -> grid_cell {
    let mut gc = unsafe { grid_default_cell };
    gc.fg = 1;
    gc.bg = 2;
    gc.us = 3;
    gc.attr = 0x10;
    gc.flags = 0x4;
    gc
}

/// Parses `s` into a fresh style over [`base`], returning the style and
/// what the parser answered.
fn parse(s: &CStr) -> (Box<style>, c_int) {
    let mut sy = blank();
    let gc = base();
    let retval = unsafe { style_parse(&mut sy, &gc, s.to_bytes()) };
    (sy, retval)
}

/// The style `s` parses to, which must parse.
fn parsed(s: &CStr) -> Box<style> {
    let (sy, retval) = parse(s);
    assert_eq!(retval, 0, "{s:?} did not parse");
    sy
}

/// What the parser makes of `s`, which must not parse.
fn refused(s: &CStr) {
    let (sy, retval) = parse(s);
    assert_eq!(retval, -1, "{s:?} parsed");
    assert_eq!(tostring(&sy), "default", "{s:?} left the style changed");
}

fn tostring(sy: &style) -> String {
    unsafe { style_tostring(sy).to_string_lossy().into_owned() }
}

fn range_string(sy: &style) -> String {
    unsafe { seen(&raw const sy.range_string as *const c_char) }
}

#[test]
fn a_style_starts_from_the_defaults_and_the_cell_it_is_given() {
    let sy = blank();
    assert_eq!(sy.gc.fg, 8);
    assert_eq!(sy.gc.bg, 8);
    assert_eq!(sy.ignore, 0);
    assert_eq!(sy.fill, 8);
    assert_eq!(sy.align, STYLE_ALIGN_DEFAULT);
    assert_eq!(sy.list, STYLE_LIST_OFF);
    assert_eq!(sy.range_type, STYLE_RANGE_NONE);
    assert_eq!(sy.range_argument, 0);
    assert_eq!(range_string(&sy), "");
    assert_eq!(sy.width, STYLE_WIDTH_DEFAULT);
    assert_eq!(sy.width_percentage, 0);
    assert_eq!(sy.pad, STYLE_PAD_DEFAULT);
    assert_eq!(sy.default_type, STYLE_DEFAULT_BASE);

    let mut from_cell = Box::new(style::default());
    let gc = base();
    style_set(&mut from_cell, &gc);
    assert_eq!(from_cell.gc.fg, 1);
    assert_eq!(from_cell.gc.attr, 0x10);
    assert_eq!(from_cell.fill, 8);
}

#[test]
fn a_style_is_copied_whole() {
    let mut src = parsed(c"fg=red,align=centre,width=4");
    let mut dst = blank();
    style_copy(&mut dst, &src);
    assert_eq!(tostring(&dst), tostring(&src));
}

#[test]
fn an_empty_string_leaves_the_style_as_it_was() {
    let mut sy = parsed(c"fg=red");
    let gc = base();
    assert_eq!(unsafe { style_parse(&mut sy, &gc, b"") }, 0);
    assert_eq!(tostring(&sy), "fg=red");
}

#[test]
fn default_takes_the_colours_and_attributes_of_the_base_cell() {
    let sy = parsed(c"default");
    assert_eq!((sy.gc.fg, sy.gc.bg, sy.gc.us), (1, 2, 3));
    assert_eq!(sy.gc.attr, 0x10);
    assert_eq!(sy.gc.flags, 0x4);
}

#[test]
fn ignore_is_turned_on_and_off_again() {
    assert_eq!(parsed(c"ignore").ignore, 1);
    assert_eq!(parsed(c"ignore,noignore").ignore, 0);
}

#[test]
fn the_default_type_is_pushed_popped_or_set() {
    assert_eq!(parsed(c"push-default").default_type, STYLE_DEFAULT_PUSH);
    assert_eq!(parsed(c"pop-default").default_type, STYLE_DEFAULT_POP);
    assert_eq!(parsed(c"set-default").default_type, STYLE_DEFAULT_SET);
}

#[test]
fn the_list_takes_one_of_five_names() {
    assert_eq!(parsed(c"list=on").list, STYLE_LIST_ON);
    assert_eq!(parsed(c"list=focus").list, STYLE_LIST_FOCUS);
    assert_eq!(parsed(c"list=left-marker").list, STYLE_LIST_LEFT_MARKER);
    assert_eq!(parsed(c"list=right-marker").list, STYLE_LIST_RIGHT_MARKER);
    assert_eq!(parsed(c"list=on,nolist").list, STYLE_LIST_OFF);
    refused(c"list=other");
    refused(c"list=");
}

#[test]
fn the_alignment_takes_one_of_four_names() {
    assert_eq!(parsed(c"align=left").align, STYLE_ALIGN_LEFT);
    assert_eq!(parsed(c"align=centre").align, STYLE_ALIGN_CENTRE);
    assert_eq!(parsed(c"align=right").align, STYLE_ALIGN_RIGHT);
    assert_eq!(
        parsed(c"align=absolute-centre").align,
        STYLE_ALIGN_ABSOLUTE_CENTRE
    );
    assert_eq!(parsed(c"align=right,noalign").align, STYLE_ALIGN_DEFAULT);
    refused(c"align=other");
}

#[test]
fn a_range_names_what_it_covers_and_may_carry_an_argument() {
    let sy = parsed(c"range=left");
    assert_eq!(sy.range_type, STYLE_RANGE_LEFT);
    assert_eq!(sy.range_argument, 0);
    assert_eq!(range_string(&sy), "");

    assert_eq!(parsed(c"range=right").range_type, STYLE_RANGE_RIGHT);

    let sy = parsed(c"range=control|7");
    assert_eq!(sy.range_type, STYLE_RANGE_CONTROL);
    assert_eq!(sy.range_argument, 7);

    let sy = parsed(c"range=pane|%12");
    assert_eq!(sy.range_type, STYLE_RANGE_PANE);
    assert_eq!(sy.range_argument, 12);

    let sy = parsed(c"range=window|34");
    assert_eq!(sy.range_type, STYLE_RANGE_WINDOW);
    assert_eq!(sy.range_argument, 34);

    let sy = parsed(c"range=session|$56");
    assert_eq!(sy.range_type, STYLE_RANGE_SESSION);
    assert_eq!(sy.range_argument, 56);

    let sy = parsed(c"range=user|name");
    assert_eq!(sy.range_type, STYLE_RANGE_USER);
    assert_eq!(sy.range_argument, 0);
    assert_eq!(range_string(&sy), "name");

    let sy = parsed(c"range=left,norange");
    assert_eq!(sy.range_type, STYLE_RANGE_NONE);
    assert_eq!(sy.range_argument, 0);
    assert_eq!(range_string(&sy), "");
}

#[test]
fn a_range_argument_is_cut_to_the_room_the_style_keeps_for_it() {
    let sy = parsed(c"range=user|0123456789abcdefghij");
    assert_eq!(range_string(&sy), "0123456789abcde");
}

#[test]
fn a_range_is_refused_when_its_argument_does_not_fit_its_kind() {
    refused(c"range=left|1");
    refused(c"range=right|1");
    refused(c"range=control");
    refused(c"range=control|10");
    refused(c"range=pane");
    refused(c"range=pane|12");
    refused(c"range=pane|%");
    refused(c"range=pane|%x");
    refused(c"range=window");
    refused(c"range=window|x");
    refused(c"range=session");
    refused(c"range=session|12");
    refused(c"range=session|$");
    refused(c"range=session|$x");
    refused(c"range=user");
    refused(c"range=left|");
}

#[test]
fn a_range_of_an_unknown_kind_is_taken_and_ignored() {
    let sy = parsed(c"range=other|1");
    assert_eq!(sy.range_type, STYLE_RANGE_NONE);
    assert_eq!(tostring(&sy), "default");
}

#[test]
fn the_fill_takes_a_colour() {
    assert_eq!(parsed(c"fill=red").fill, 1);
    refused(c"fill=notacolour");
}

#[test]
fn the_three_colours_fall_back_to_the_base_cell_when_they_are_default() {
    let sy = parsed(c"fg=red,bg=blue,us=green");
    assert_eq!((sy.gc.fg, sy.gc.bg, sy.gc.us), (1, 4, 2));

    let sy = parsed(c"fg=default,bg=default,us=default");
    assert_eq!((sy.gc.fg, sy.gc.bg, sy.gc.us), (1, 2, 3));

    assert_eq!(parsed(c"FG=red").gc.fg, 1);
    assert_eq!(parsed(c"BG=red").gc.bg, 1);
    refused(c"xg=red");
    refused(c"fg=notacolour");
    refused(c"us=notacolour");
}

#[test]
fn attributes_are_added_taken_away_and_cleared() {
    assert_eq!(parsed(c"bright").gc.attr, 0x1);
    assert_eq!(parsed(c"bright,underscore").gc.attr, 0x1 | 0x4);
    assert_eq!(parsed(c"bright,nobright").gc.attr, 0);
    assert_eq!(parsed(c"bright,none").gc.attr, 0);
    assert_eq!(parsed(c"noattr").gc.attr as c_int, GRID_ATTR_NOATTR);
    refused(c"notanattribute");
    refused(c"nonsuch");
}

#[test]
fn the_width_is_a_count_or_a_percentage() {
    let sy = parsed(c"width=40");
    assert_eq!((sy.width, sy.width_percentage), (40, 0));

    let sy = parsed(c"width=40%");
    assert_eq!((sy.width, sy.width_percentage), (40, 1));

    let sy = parsed(c"width=8%");
    assert_eq!((sy.width, sy.width_percentage), (8, 1));

    refused(c"width=%");
    refused(c"width=x");
    refused(c"width=200%");
    refused(c"width=");
}

#[test]
fn the_padding_is_a_count() {
    assert_eq!(parsed(c"pad=3").pad, 3);
    refused(c"pad=x");
    refused(c"pad=");
}

#[test]
fn words_are_separated_by_spaces_commas_and_newlines() {
    let sy = parsed(c" ,\nfg=red bg=blue,\nalign=right ");
    assert_eq!((sy.gc.fg, sy.gc.bg), (1, 4));
    assert_eq!(sy.align, STYLE_ALIGN_RIGHT);

    assert_eq!(tostring(&parsed(c" ,\n")), "default");
}

#[test]
fn a_word_too_long_for_the_buffer_is_refused() {
    let long = ::std::ffi::CString::new("x".repeat(256)).unwrap();
    let (sy, retval) = parse(&long);
    assert_eq!(retval, -1);
    assert_eq!(tostring(&sy), "default");
}

#[test]
fn a_style_is_put_back_as_it_was_when_a_later_word_is_refused() {
    let mut sy = parsed(c"fg=red");
    let gc = base();
    assert_eq!(
        unsafe { style_parse(&mut sy, &gc, b"bg=blue,notanattribute") },
        -1
    );
    assert_eq!(tostring(&sy), "fg=red");
}

#[test]
fn a_default_style_prints_as_default() {
    assert_eq!(tostring(&blank()), "default");
}

#[test]
fn every_field_that_is_not_a_default_is_printed() {
    assert_eq!(tostring(&parsed(c"list=on")), "list=on");
    assert_eq!(tostring(&parsed(c"list=focus")), "list=focus");
    assert_eq!(tostring(&parsed(c"list=left-marker")), "list=left-marker");
    assert_eq!(tostring(&parsed(c"list=right-marker")), "list=right-marker");
    assert_eq!(tostring(&parsed(c"range=left")), "range=left");
    assert_eq!(tostring(&parsed(c"range=right")), "range=right");
    assert_eq!(tostring(&parsed(c"range=pane|%12")), "range=pane|%12");
    assert_eq!(tostring(&parsed(c"range=window|34")), "range=window|34");
    assert_eq!(tostring(&parsed(c"range=session|$56")), "range=session|$56");
    assert_eq!(tostring(&parsed(c"range=user|name")), "range=user|name");
    assert_eq!(tostring(&parsed(c"align=left")), "align=left");
    assert_eq!(tostring(&parsed(c"align=centre")), "align=centre");
    assert_eq!(tostring(&parsed(c"align=right")), "align=right");
    assert_eq!(
        tostring(&parsed(c"align=absolute-centre")),
        "align=absolute-centre"
    );
    assert_eq!(tostring(&parsed(c"push-default")), "push-default");
    assert_eq!(tostring(&parsed(c"pop-default")), "pop-default");
    assert_eq!(tostring(&parsed(c"set-default")), "set-default");
    assert_eq!(tostring(&parsed(c"fill=red")), "fill=red");
    assert_eq!(tostring(&parsed(c"fg=red")), "fg=red");
    assert_eq!(tostring(&parsed(c"bg=blue")), "bg=blue");
    assert_eq!(tostring(&parsed(c"us=green")), "us=green");
    assert_eq!(tostring(&parsed(c"bright")), "bright");
    assert_eq!(tostring(&parsed(c"width=4")), "width=4");
    assert_eq!(tostring(&parsed(c"width=40%")), "width=40%");
    assert_eq!(tostring(&parsed(c"pad=2")), "pad=2");
}

#[test]
fn the_printed_form_puts_every_field_in_order_behind_commas() {
    let sy = parsed(
        c"list=on,range=user|abc,align=centre,push-default,fill=red,fg=green,bg=blue,us=yellow,bright,width=3,pad=1",
    );
    assert_eq!(
        tostring(&sy),
        "list=on,range=user|abc,align=centre,push-default,fill=red,fg=green,bg=blue,us=yellow,bright,width=3,pad=1"
    );
}

#[test]
fn a_range_of_control_prints_under_whatever_name_came_before_it() {
    let sy = parsed(c"range=control|3");
    assert_eq!(sy.range_type, STYLE_RANGE_CONTROL);
    assert_eq!(tostring(&sy), "range=");

    let sy = parsed(c"list=focus,range=control|3");
    assert_eq!(tostring(&sy), "list=focus,range=focus");
}

#[test]
fn a_style_option_adds_its_colours_and_attributes_to_a_cell() {
    let _guard = globals();
    let oo = Options::empty(null_mut());
    unsafe {
        options_set_string(
            oo.ptr(),
            c"@c2rs-style".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"fg=red,bg=blue,us=green,bright".as_ptr()],
        );
        let mut gc = grid_default_cell;
        gc.fg = 9;
        style_add(&mut gc, oo.ptr(), c"@c2rs-style".as_ptr(), None);
        assert_eq!((gc.fg, gc.bg, gc.us), (1, 4, 2));
        assert_eq!(gc.attr, 0x1);

        let mut gc = grid_default_cell;
        gc.attr = 0x8;
        style_apply(&mut gc, oo.ptr(), c"@c2rs-style".as_ptr(), None);
        assert_eq!((gc.fg, gc.bg), (1, 4));
        assert_eq!(gc.attr, 0x1);
    }
}

#[test]
fn a_style_option_that_is_missing_leaves_the_cell_alone() {
    let _guard = globals();
    let oo = Options::empty(null_mut());
    unsafe {
        let mut gc = grid_default_cell;
        gc.fg = 9;
        style_add(&mut gc, oo.ptr(), c"@c2rs-missing".as_ptr(), None);
        assert_eq!((gc.fg, gc.bg, gc.us), (9, 8, 0));
        assert_eq!(gc.attr, 0);
    }
}

#[test]
fn a_style_option_holding_a_format_is_expanded_before_it_is_parsed() {
    let _guard = globals();
    let oo = Options::empty(null_mut());
    unsafe {
        options_set_string(
            oo.ptr(),
            c"@c2rs-format".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"fg=#{?1,red,blue}".as_ptr()],
        );
        let mut gc = grid_default_cell;
        style_add(&mut gc, oo.ptr(), c"@c2rs-format".as_ptr(), None);
        assert_eq!(gc.fg, 4);
    }
}

#[test]
fn the_scrollbar_style_falls_back_to_a_one_wide_unpadded_space() {
    let _guard = globals();
    let empty = Options::empty(null_mut());
    let mut sb = Box::new(style::default());
    unsafe {
        style_set_scrollbar_style_from_option(&mut sb, empty.ptr());
        assert_eq!(sb.width, PANE_SCROLLBARS_DEFAULT_WIDTH);
        assert_eq!(sb.pad, PANE_SCROLLBARS_DEFAULT_PADDING);
        assert_eq!(sb.gc.data.data[0], b' ');
        assert_eq!(sb.gc.fg, 8);
    }
}

#[test]
fn the_scrollbar_style_takes_the_option_and_floors_its_width_and_padding() {
    let _guard = globals();
    let oo = Options::window();
    let mut sb = Box::new(style::default());
    unsafe {
        options_set_string(
            oo.ptr(),
            c"pane-scrollbars-style".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"bg=red,width=3,pad=2".as_ptr()],
        );
        style_set_scrollbar_style_from_option(&mut sb, oo.ptr());
        assert_eq!((sb.width, sb.pad), (3, 2));
        assert_eq!(sb.gc.bg, 1);
        assert_eq!(sb.gc.data.data[0], b' ');

        options_set_string(
            oo.ptr(),
            c"pane-scrollbars-style".as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![c"bg=red".as_ptr()],
        );
        style_set_scrollbar_style_from_option(&mut sb, oo.ptr());
        assert_eq!(
            (sb.width, sb.pad),
            (
                PANE_SCROLLBARS_DEFAULT_WIDTH,
                PANE_SCROLLBARS_DEFAULT_PADDING
            )
        );
    }
}

#[test]
fn a_range_list_is_walked_by_column_and_freed_whole() {
    let mut srs = ::core::mem::MaybeUninit::<style_ranges>::uninit();
    unsafe {
        style_ranges_init(srs.as_mut_ptr());
        let srs = srs.assume_init_mut();
        assert!(srs.is_empty());
        assert!(style_ranges_get_range(srs, 0).is_null());

        for (start, end) in [(0, 4), (4, 10)] {
            srs.push(style_range {
                type_0: STYLE_RANGE_NONE as style_range_type,
                argument: 0,
                string: [0; 16],
                start,
                end,
            });
        }
        let added = [&raw mut srs[0], &raw mut srs[1]];

        assert_eq!(style_ranges_get_range(srs, 0), added[0]);
        assert_eq!(style_ranges_get_range(srs, 3), added[0]);
        assert_eq!(style_ranges_get_range(srs, 4), added[1]);
        assert_eq!(style_ranges_get_range(srs, 9), added[1]);
        assert!(style_ranges_get_range(srs, 10).is_null());

        style_ranges_free(srs);
        assert!(srs.is_empty());
    }
}

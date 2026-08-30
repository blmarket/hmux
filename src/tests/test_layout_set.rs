use super::*;
use crate::fmt_args;
use crate::layout::LAYOUT_CELL_FLOATING;
use crate::options::{options_set_number, options_set_string};
use crate::tests::test_fixtures::{Layout, globals};
use ::core::ffi::{CStr, c_int};
use ::std::ffi::CString;

/// A window of `sx` by `sy` holding `n` panes, none of them yet arranged
/// into anything but the single cell `layout_init` left.
fn build(n: usize, sx: u_int, sy: u_int) -> Layout {
    let mut l = Layout::new(sx, sy);
    for _ in 1..n {
        l.add_pane(sx, sy);
    }
    l
}

/// Arranges the window into the layout `name` and answers the tree.
fn set(l: &mut Layout, name: &CStr) -> String {
    unsafe {
        let i = layout_set_lookup(name.as_ptr());
        assert!(i >= 0, "{name:?} is a layout");
        assert_eq!(layout_set_select(l.w(), i as u_int), i as u_int);
        assert_eq!((*l.w()).lastlayout, i);
    }
    l.dump()
}

/// Gives one of the window's options a string value.
fn option(l: &mut Layout, name: &CStr, value: &str) {
    let value = CString::new(value).expect("no NUL");
    unsafe {
        options_set_string(
            options_ptr(&(*l.w()).options),
            name.as_ptr(),
            0,
            c"%s".as_ptr(),
            fmt_args![value.as_ptr()],
        );
    }
}

fn lookup(name: &CStr) -> c_int {
    unsafe { layout_set_lookup(name.as_ptr()) }
}

#[test]
fn every_layout_is_found_by_its_whole_name() {
    assert_eq!(lookup(c"even-horizontal"), 0);
    assert_eq!(lookup(c"even-vertical"), 1);
    assert_eq!(lookup(c"main-horizontal"), 2);
    assert_eq!(lookup(c"main-horizontal-mirrored"), 3);
    assert_eq!(lookup(c"main-vertical"), 4);
    assert_eq!(lookup(c"main-vertical-mirrored"), 5);
    assert_eq!(lookup(c"tiled"), 6);
}

/// A name may be shortened while it still fits only one layout. A whole
/// name is looked for first, so `main-vertical` is that layout even
/// though it is also the front of `main-vertical-mirrored`.
#[test]
fn a_layout_name_may_be_shortened() {
    assert_eq!(lookup(c"t"), 6);
    assert_eq!(lookup(c"even-h"), 0);
    assert_eq!(lookup(c"main-vertical"), 4);
    assert_eq!(lookup(c"main-vertical-m"), 5);
    assert_eq!(lookup(c"even-"), -1);
    assert_eq!(lookup(c"main-v"), -1);
    assert_eq!(lookup(c"m"), -1);
    assert_eq!(lookup(c""), -1);
    assert_eq!(lookup(c"zzz"), -1);
    assert_eq!(lookup(c"tiledd"), -1);
}

#[test]
fn an_index_past_the_last_layout_is_the_last_layout() {
    let _guard = globals();
    let mut l = build(2, 80, 24);
    unsafe {
        assert_eq!(layout_set_select(l.w(), 99), 6);
        assert_eq!((*l.w()).lastlayout, 6);
    }
}

#[test]
fn the_layouts_are_stepped_through_in_a_ring() {
    let _guard = globals();
    let mut l = build(2, 80, 24);
    unsafe {
        assert_eq!((*l.w()).lastlayout, -1);
        assert_eq!(layout_set_next(l.w()), 0);
        assert_eq!(layout_set_next(l.w()), 1);
        (*l.w()).lastlayout = 6;
        assert_eq!(layout_set_next(l.w()), 0);
        assert_eq!(layout_set_previous(l.w()), 6);
        assert_eq!(layout_set_previous(l.w()), 5);
        (*l.w()).lastlayout = -1;
        assert_eq!(layout_set_previous(l.w()), 6);
    }
}

#[test]
fn a_window_of_one_pane_is_left_alone() {
    let _guard = globals();
    for name in [
        c"even-horizontal",
        c"even-vertical",
        c"main-horizontal",
        c"main-horizontal-mirrored",
        c"main-vertical",
        c"main-vertical-mirrored",
        c"tiled",
    ] {
        let mut l = build(1, 80, 24);
        assert_eq!(set(&mut l, name), "%1 80x24+0+0", "{name:?}");
    }
}

#[test]
fn an_even_layout_shares_the_window_out() {
    let _guard = globals();
    let mut l = build(3, 80, 24);
    assert_eq!(
        set(&mut l, c"even-horizontal"),
        "LR 80x24+0+0 [%1 26x24+0+0 | %2 26x24+27+0 | %3 26x24+54+0]"
    );
    let mut l = build(3, 80, 24);
    assert_eq!(
        set(&mut l, c"even-vertical"),
        "TB 80x24+0+0 [%1 80x8+0+0 | %2 80x7+0+9 | %3 80x7+0+17]"
    );
    let mut l = build(5, 80, 24);
    assert_eq!(
        set(&mut l, c"even-horizontal"),
        "LR 80x24+0+0 [%1 16x24+0+0 | %2 15x24+17+0 | %3 15x24+33+0 | %4 15x24+49+0 | %5 15x24+65+0]"
    );
}

/// Every pane keeps at least one column, so a window too narrow to hold
/// them all grows rather than squeezing them.
#[test]
fn an_even_layout_grows_a_window_that_is_too_small() {
    let _guard = globals();
    let mut l = build(3, 4, 24);
    assert_eq!(
        set(&mut l, c"even-horizontal"),
        "LR 5x24+0+0 [%1 1x24+0+0 | %2 1x24+2+0 | %3 1x24+4+0]"
    );
    assert_eq!(unsafe { (*l.w()).sx }, 5);
    let mut l = build(3, 80, 4);
    assert_eq!(
        set(&mut l, c"even-vertical"),
        "TB 80x5+0+0 [%1 80x1+0+0 | %2 80x1+0+2 | %3 80x1+0+4]"
    );
    assert_eq!(unsafe { (*l.w()).sy }, 5);
}

#[test]
fn a_main_layout_gives_one_pane_the_room() {
    let _guard = globals();
    let mut l = build(2, 80, 24);
    assert_eq!(
        set(&mut l, c"main-horizontal"),
        "TB 80x24+0+0 [%1 80x22+0+0 | %2 80x1+0+23]"
    );
    let mut l = build(4, 80, 24);
    assert_eq!(
        set(&mut l, c"main-horizontal"),
        "TB 80x24+0+0 [%1 80x22+0+0 | LR 80x1+0+23 [%2 26x1+0+23 | %3 26x1+27+23 | %4 26x1+54+23]]"
    );
    let mut l = build(2, 80, 24);
    assert_eq!(
        set(&mut l, c"main-vertical"),
        "LR 80x24+0+0 [%1 78x24+0+0 | %2 1x24+79+0]"
    );
    let mut l = build(4, 80, 24);
    assert_eq!(
        set(&mut l, c"main-vertical"),
        "LR 80x24+0+0 [%1 78x24+0+0 | TB 1x24+79+0 [%2 1x8+79+0 | %3 1x7+79+9 | %4 1x7+79+17]]"
    );
}

/// The mirrored layouts put the other panes first; nothing else changes.
#[test]
fn a_mirrored_main_layout_puts_the_others_first() {
    let _guard = globals();
    let mut l = build(2, 80, 24);
    assert_eq!(
        set(&mut l, c"main-horizontal-mirrored"),
        "TB 80x24+0+0 [%2 80x1+0+0 | %1 80x22+0+2]"
    );
    let mut l = build(4, 80, 24);
    assert_eq!(
        set(&mut l, c"main-horizontal-mirrored"),
        "TB 80x24+0+0 [LR 80x1+0+0 [%2 26x1+0+0 | %3 26x1+27+0 | %4 26x1+54+0] | %1 80x22+0+2]"
    );
    let mut l = build(2, 80, 24);
    assert_eq!(
        set(&mut l, c"main-vertical-mirrored"),
        "LR 80x24+0+0 [%2 1x24+0+0 | %1 78x24+2+0]"
    );
    let mut l = build(4, 80, 24);
    assert_eq!(
        set(&mut l, c"main-vertical-mirrored"),
        "LR 80x24+0+0 [TB 1x24+0+0 [%2 1x8+0+0 | %3 1x7+0+9 | %4 1x7+0+17] | %1 78x24+2+0]"
    );
}

/// The size of the pane the layout is built around: the same whichever
/// way round the layout is written.
fn main_pane(l: &mut Layout) -> String {
    unsafe { format!("{}x{}", (*l.pane(0)).sx, (*l.pane(0)).sy) }
}

/// The main pane's size comes from `main-pane-height`, and the rest from
/// `other-pane-height`. A value that is no number falls back to 24 lines,
/// a zero or unreadable `other-pane-height` leaves the rest of the window
/// to the others, and an `other-pane-height` that would leave the main
/// pane less than it asked for is cut down instead.
#[test]
fn the_main_pane_height_is_read_from_the_options() {
    let _guard = globals();
    for name in [c"main-horizontal", c"main-horizontal-mirrored"] {
        let tall = |main: &str, other: Option<&str>| {
            let mut l = build(3, 80, 40);
            option(&mut l, c"main-pane-height", main);
            if let Some(other) = other {
                option(&mut l, c"other-pane-height", other);
            }
            set(&mut l, name);
            main_pane(&mut l)
        };
        assert_eq!(tall("24", None), "80x24", "{name:?}");
        assert_eq!(tall("24", Some("0")), "80x24", "{name:?}");
        assert_eq!(tall("24", Some("zzz")), "80x24", "{name:?}");
        assert_eq!(tall("zzz", None), "80x24", "{name:?}");
        assert_eq!(tall("50%", None), "80x19", "{name:?}");
        assert_eq!(tall("5", Some("100")), "80x5", "{name:?}");
        assert_eq!(tall("30", Some("15")), "80x30", "{name:?}");
        assert_eq!(tall("15", Some("10")), "80x29", "{name:?}");
    }
}

#[test]
fn the_main_pane_width_is_read_from_the_options() {
    let _guard = globals();
    for name in [c"main-vertical", c"main-vertical-mirrored"] {
        let wide = |main: &str, other: Option<&str>| {
            let mut l = build(3, 120, 24);
            option(&mut l, c"main-pane-width", main);
            if let Some(other) = other {
                option(&mut l, c"other-pane-width", other);
            }
            set(&mut l, name);
            main_pane(&mut l)
        };
        assert_eq!(wide("80", None), "80x24", "{name:?}");
        assert_eq!(wide("zzz", None), "80x24", "{name:?}");
        assert_eq!(wide("20", Some("200")), "20x24", "{name:?}");
        assert_eq!(wide("100", Some("50")), "100x24", "{name:?}");
        assert_eq!(wide("40", Some("30")), "89x24", "{name:?}");
    }
}

/// A window with no room for both keeps one line, or one column, each.
#[test]
fn a_main_layout_in_a_window_with_no_room_keeps_one_line_each() {
    let _guard = globals();
    for name in [c"main-horizontal", c"main-horizontal-mirrored"] {
        let mut l = build(3, 80, 3);
        set(&mut l, name);
        assert_eq!(main_pane(&mut l), "80x1", "{name:?}");
        let mut l = build(3, 20, 6);
        set(&mut l, name);
        assert_eq!(main_pane(&mut l), "20x4", "{name:?}");
    }
    for name in [c"main-vertical", c"main-vertical-mirrored"] {
        let mut l = build(3, 3, 80);
        set(&mut l, name);
        assert_eq!(main_pane(&mut l), "1x80", "{name:?}");
        let mut l = build(3, 6, 20);
        set(&mut l, name);
        assert_eq!(main_pane(&mut l), "4x20", "{name:?}");
    }
    let mut l = build(3, 80, 3);
    assert_eq!(
        set(&mut l, c"main-horizontal"),
        "TB 80x3+0+0 [%1 80x1+0+0 | LR 80x1+0+2 [%2 40x1+0+2 | %3 39x1+41+2]]"
    );
    let mut l = build(3, 3, 80);
    assert_eq!(
        set(&mut l, c"main-vertical"),
        "LR 3x80+0+0 [%1 1x80+0+0 | TB 1x80+2+0 [%2 1x40+2+0 | %3 1x39+2+41]]"
    );
}

#[test]
fn a_tiled_layout_fills_rows_and_columns() {
    let _guard = globals();
    let tiled = |n: usize, sx: u_int, sy: u_int| {
        let mut l = build(n, sx, sy);
        set(&mut l, c"tiled")
    };
    assert_eq!(
        tiled(2, 80, 24),
        "TB 80x24+0+0 [%1 80x11+0+0 | %2 80x12+0+12]"
    );
    assert_eq!(
        tiled(3, 80, 24),
        "TB 80x24+0+0 [LR 80x11+0+0 [%1 39x11+0+0 | %2 40x11+40+0] | %3 80x12+0+12]"
    );
    assert_eq!(
        tiled(4, 80, 24),
        "TB 80x24+0+0 [LR 80x11+0+0 [%1 39x11+0+0 | %2 40x11+40+0] | LR 80x12+0+12 [%3 39x12+0+12 | %4 40x12+40+12]]"
    );
    assert_eq!(
        tiled(7, 80, 24),
        "TB 80x24+0+0 [LR 80x7+0+0 [%1 26x7+0+0 | %2 26x7+27+0 | %3 26x7+54+0] | LR 80x7+0+8 [%4 26x7+0+8 | %5 26x7+27+8 | %6 26x7+54+8] | %7 80x8+0+16]"
    );
    assert_eq!(
        tiled(9, 80, 24),
        "TB 80x24+0+0 [LR 80x7+0+0 [%1 26x7+0+0 | %2 26x7+27+0 | %3 26x7+54+0] | LR 80x7+0+8 [%4 26x7+0+8 | %5 26x7+27+8 | %6 26x7+54+8] | LR 80x8+0+16 [%7 26x8+0+16 | %8 26x8+27+16 | %9 26x8+54+16]]"
    );
    assert_eq!(
        tiled(3, 4, 4),
        "TB 4x4+0+0 [LR 4x1+0+0 [%1 1x1+0+0 | %2 2x1+2+0] | %3 4x2+0+2]"
    );
}

/// Every tile keeps at least one column and one line, so a window with
/// less room than that grows instead.
#[test]
fn a_tiled_layout_grows_a_window_that_is_too_small() {
    let _guard = globals();
    let mut l = build(3, 2, 2);
    assert_eq!(
        set(&mut l, c"tiled"),
        "TB 3x3+0+0 [LR 2x1+0+0 [%1 1x1+0+0 | %2 1x1+2+0] | %3 2x1+0+2]"
    );
    assert_eq!(unsafe { ((*l.w()).sx, (*l.w()).sy) }, (3, 3));
}

#[test]
fn a_tiled_layout_keeps_to_the_column_limit() {
    let _guard = globals();
    let mut l = build(5, 80, 24);
    option(&mut l, c"main-pane-height", "24");
    unsafe {
        options_set_number(
            options_ptr(&(*l.w()).options),
            c"tiled-layout-max-columns".as_ptr(),
            2,
        )
    };
    assert_eq!(
        set(&mut l, c"tiled"),
        "TB 80x24+0+0 [LR 80x7+0+0 [%1 39x7+0+0 | %2 40x7+40+0] | LR 80x7+0+8 [%3 39x7+0+8 | %4 40x7+40+8] | %5 80x8+0+16]"
    );
    let mut l = build(3, 80, 24);
    unsafe {
        options_set_number(
            options_ptr(&(*l.w()).options),
            c"tiled-layout-max-columns".as_ptr(),
            1,
        )
    };
    assert_eq!(
        set(&mut l, c"tiled"),
        "TB 80x24+0+0 [%1 80x7+0+0 | %2 80x7+0+8 | %3 80x8+0+16]"
    );
}

/// Arranging a window frees the whole layout tree first, and freeing a
/// cell takes the pane's own pointer to it away — so a pane that was
/// floating stops being one before the panes are walked. It is left out
/// of the count that decides whether there is anything to do, and then
/// laid out with the rest anyway.
#[test]
fn a_floating_pane_is_counted_out_but_laid_out_anyway() {
    let _guard = globals();
    let mut l = build(3, 80, 24);
    set(&mut l, c"even-horizontal");
    unsafe { (*(*l.pane(1)).layout_cell).flags |= LAYOUT_CELL_FLOATING };
    assert_eq!(unsafe { crate::window::window_count_panes(l.w(), 0) }, 2);
    assert_eq!(
        set(&mut l, c"even-horizontal"),
        "LR 80x24+0+0 [%1 26x24+0+0 | %2 26x24+27+0 | %3 26x24+54+0]"
    );
}

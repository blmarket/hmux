//! Unit tests for [`crate::cmd::cmd_resize_pane`] — the `resize-pane` entry
//! (name, alias, argument template, usage, find flags and exec hook), the
//! block of message-protocol, style and command constants the file declares,
//! the argument parsing the template drives, and the exec branches that are
//! deterministic without touching a server: `-T` trimming the scrollback
//! below the cursor (a pure grid edit that returns before any redraw), `-M`
//! with no valid mouse event (the guard stops the mouse path at once), and
//! `-Z` on a one-pane window, where `window_zoom` refuses and the redraw walk
//! finds no clients. The layout-moving halves of `-x`/`-y`/`-L`/`-R`/`-U`/`-D`
//! each end in `notify_window`, which parks a notification on the global
//! command queue nothing drains, so no test drives them here.

use crate::arguments::{args_count, args_has, args_string};
use crate::cmd::cmd_get_args;
use crate::cmd::cmd_resize_pane::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, PANE_REDRAW, WINDOW_ZOOMED,
    cmd_resize_pane_entry,
};
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::grid::grid_scroll_history;
use crate::screen::screen_grid_ptr;
use crate::tests::test_fixtures::{Item, Target, ensure_reactor, globals, seen};
use crate::types::*;
use crate::window::PANE_ZOOMED;
use crate::window::window_get_active;
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;

/// The entry under test.
const ENTRY: *const cmd_entry = &raw const cmd_resize_pane_entry;

/// Where the tests' items claim to come from.
const FILE: &CStr = c"test-coverage-cmd-resize-pane.conf";

/// Runs the parsed command an item carries through the entry's exec hook, the
/// way the command queue calls it. The item must be running this entry.
unsafe fn exec_via(item: &mut Item) -> cmd_retval {
    unsafe {
        let exec = (*ENTRY).exec;
        exec(&*item.cmd(), item.ptr())
    }
}

#[test]
fn the_entry_describes_the_resize_pane_command() {
    let _guard = globals();
    unsafe {
        assert_eq!((*ENTRY).name.to_string_lossy(), "resize-pane");
        assert_eq!(
            (*ENTRY)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "resizep"
        );
        assert_eq!((*ENTRY).args.template.to_string_lossy(), "DLMRTt:Ux:y:Z");
        assert_eq!((*ENTRY).args.lower, 0);
        assert_eq!((*ENTRY).args.upper, 1);
        assert!(
            (*ENTRY).args.cb.is_none(),
            "resize-pane takes no args callback"
        );
        assert_eq!(
            (*ENTRY).usage.to_string_lossy(),
            "[-DLMRTUZ] [-x width] [-y height] [-t target-pane] [adjustment]"
        );

        assert_eq!((*ENTRY).source.flag, 0);
        assert_eq!((*ENTRY).source.type_0, CMD_FIND_PANE);
        assert_eq!((*ENTRY).source.flags, 0);
        assert_eq!((*ENTRY).target.flag, b't' as c_char);
        assert_eq!((*ENTRY).target.type_0, CMD_FIND_PANE);
        assert_eq!((*ENTRY).target.flags, 0);

        assert_eq!((*ENTRY).flags, CMD_AFTERHOOK);
    }
}

#[test]
fn the_parser_resolves_the_name_the_alias_and_a_prefix() {
    let _guard = globals();
    for (i, line) in [c"resize-pane -L", c"resizep -T", c"resize-p -R 5"]
        .into_iter()
        .enumerate()
    {
        let mut item = Item::new().from_file(FILE, i as u_int + 1).with_args(line);
        assert!(
            ::core::ptr::eq(unsafe { (*item.cmd()).entry }, ENTRY),
            "{line:?}"
        );
    }

    let mut flagged = Item::new()
        .from_file(FILE, 9)
        .with_args(c"resize-pane -DLMRTUZ -x 11 -y 22");
    assert!(::core::ptr::eq(unsafe { (*flagged.cmd()).entry }, ENTRY));
    unsafe {
        let args = cmd_get_args(&*flagged.cmd());
        for flag in *b"DLMRTUZxy" {
            assert_eq!(args_has(args, flag), 1, "{}", flag as char);
        }
        assert_eq!(args_count(args), 0, "every flag took its own argument");

        let mut adjusted = Item::new().from_file(FILE, 10).with_args(c"resize-pane 5");
        assert!(::core::ptr::eq((*adjusted.cmd()).entry, ENTRY));
        let args = cmd_get_args(&*adjusted.cmd());
        assert_eq!(args_count(args), 1);
        assert_eq!(seen(args_string(args, 0)), "5");
    }
}

#[test]
fn the_template_bounds_allow_at_most_one_adjustment() {
    let _guard = globals();
    unsafe {
        let mut none = cmd_parse_from_string(c"resize-pane".as_ptr(), null_mut());
        assert_eq!(none.status, CMD_PARSE_SUCCESS);
        let _ = none.cmdlist.take();

        let mut one = cmd_parse_from_string(c"resize-pane 3".as_ptr(), null_mut());
        assert_eq!(one.status, CMD_PARSE_SUCCESS);
        let _ = one.cmdlist.take();

        let mut two = cmd_parse_from_string(c"resize-pane 3 4".as_ptr(), null_mut());
        assert_eq!(two.status, CMD_PARSE_ERROR);
        let err = two.take_error();
        assert!(err.contains("resize-pane"), "{err}");
        assert!(err.contains("too many arguments"), "{err}");
    }
}

#[test]
fn trimming_scrollback_takes_the_history_below_the_cursor() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(10, 5);
    let wp = t.pane(0);
    let mut item = Item::new().targeting(&mut t).with_args(c"resize-pane -T");
    unsafe {
        let gd = screen_grid_ptr(&mut (*wp).base);
        for _ in 0..3 {
            grid_scroll_history(&mut *gd, 0);
        }
        (*wp).base.cy = 1;
        assert_eq!((*gd).hsize, 3);
        assert_eq!(args_has(cmd_get_args(&*item.cmd()), b'T'), 1);

        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);

        assert_eq!((*gd).hsize, 0, "the three history lines went");
        assert_eq!((*gd).hscrolled, 3, "only the history was removed");
        assert_eq!((*wp).base.cy, 4, "the cursor came down onto the old screen");
        assert_eq!((*wp).flags & PANE_REDRAW, PANE_REDRAW);
        assert!((*wp).modes.is_empty());
    }
}

#[test]
fn a_mouse_update_without_a_valid_event_stops_at_once() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    let (s, wl, w, wp) = (t.session(), t.winlink(0), t.window(0), t.pane(0));
    let mut item = Item::new().targeting(&mut t).with_args(c"resize-pane -M");
    unsafe {
        assert_eq!(args_has(cmd_get_args(&*item.cmd()), b'M'), 1);

        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);

        let target = crate::cmd::cmdq_get_target(item.ptr());
        assert_eq!((*target).session(), s);
        assert_eq!((*target).winlink(), wl);
        assert_eq!((*target).window(), w);
        assert_eq!((*target).pane(), wp);
        assert_eq!((*w).flags, 0, "nothing was redrawn or zoomed");
    }
}

#[test]
fn zooming_a_one_pane_window_is_refused_and_redraws_nothing() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    let (w, wp) = (t.window(0), t.pane(0));
    let mut item = Item::new().targeting(&mut t).with_args(c"resize-pane -Z");
    unsafe {
        assert_eq!(args_has(cmd_get_args(&*item.cmd()), b'Z'), 1);

        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);

        assert_eq!((*w).flags & WINDOW_ZOOMED, 0, "the window never zoomed");
        assert_eq!((*wp).flags & PANE_ZOOMED, 0);
        assert_eq!(window_get_active(w), wp);
        assert_eq!((*wp).sx, 80);
        assert_eq!((*wp).sy, 24, "the pane kept its size");
    }
}

#[test]
fn zooming_an_already_zoomed_window_unzooms_it() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    let (w, _wp) = (t.window(0), t.pane(0));
    unsafe {
        (*w).flags |= WINDOW_ZOOMED;
        let mut item = Item::new().targeting(&mut t).with_args(c"resize-pane -Z");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*w).flags & WINDOW_ZOOMED, 0);
    }
}

#[test]
fn resize_pane_invalid_adjustment_and_percentage_errors() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        let mut item_adj = Item::new()
            .targeting(&mut t)
            .with_args(c"resize-pane notanumber");
        assert_eq!(exec_via(&mut item_adj), CMD_RETURN_ERROR);

        let mut item_x = Item::new()
            .targeting(&mut t)
            .with_args(c"resize-pane -x invalid");
        assert_eq!(exec_via(&mut item_x), CMD_RETURN_ERROR);

        let mut item_y = Item::new()
            .targeting(&mut t)
            .with_args(c"resize-pane -y invalid");
        assert_eq!(exec_via(&mut item_y), CMD_RETURN_ERROR);
    }
}

#[test]
fn resize_pane_directions_and_dimensions() {
    let _guard = globals();
    ensure_reactor();
    let mut t = Target::new(80, 24);
    unsafe {
        crate::layout::layout_init(t.window(0), t.pane(0));

        let mut item_xy = Item::new()
            .targeting(&mut t)
            .with_args(c"resize-pane -x 40 -y 12");
        assert_eq!(exec_via(&mut item_xy), CMD_RETURN_NORMAL);

        let mut item_l = Item::new().targeting(&mut t).with_args(c"resize-pane -L 2");
        assert_eq!(exec_via(&mut item_l), CMD_RETURN_NORMAL);

        let mut item_r = Item::new().targeting(&mut t).with_args(c"resize-pane -R 2");
        assert_eq!(exec_via(&mut item_r), CMD_RETURN_NORMAL);

        let mut item_u = Item::new().targeting(&mut t).with_args(c"resize-pane -U 2");
        assert_eq!(exec_via(&mut item_u), CMD_RETURN_NORMAL);

        let mut item_d = Item::new().targeting(&mut t).with_args(c"resize-pane -D 2");
        assert_eq!(exec_via(&mut item_d), CMD_RETURN_NORMAL);

        crate::layout::layout_free(t.window(0));
    }
}

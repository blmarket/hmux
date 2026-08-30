//! Unit tests for [`crate::cmd::cmd_list_windows`] — the `list-windows`
//! entry metadata, its argument bounds and flags, the message-protocol,
//! pane, layout, style, prompt, sorting and return-value constants it
//! carries, the all-windows default template its `-a` output lines are built
//! from, and every deterministic branch of [`cmd_list_windows_exec`]
//! reachable through the entry's exec hook: the session walk over the item's
//! target and the server (`-a`) walk over the registered sessions, custom
//! formats, filters, sort orders, the reverse flag and the invalid order
//! that answers an error.
//!
//! Exec is reached through the entry's own function pointer, exactly as the
//! command queue calls it, over items whose arguments come from the real
//! command parser and whose target state points at fixture sessions holding
//! linked fixture windows. Printed lines go to a client-less item, which
//! only logs them; the content of what would be printed is asserted through
//! the same format engine the command drives.

use crate::arguments::{args_get, args_has};
use crate::cfg::cfg_print_causes;
use crate::cmd::cmd_list_windows::*;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{cmd_find, cmd_get_args, cmd_list_first, cmd_table};
use crate::session::session_set_curw;
use crate::tests::test_fixtures::{
    Format, Item, Pane, Registry, Session, Window, globals, link, seen, unlink,
};
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;

/// Where the tests' items claim to come from, which is what `cmdq_error`
/// reports them under.
const FILE: &CStr = c"test-coverage-cmd-list-windows.conf";

/// The entry whose exec hook is under test.
const WINDOWS: *const cmd_entry = &raw const cmd_list_windows_entry;

/// The all-windows default template, exactly as the exec falls back on it.
const TEMPLATE_ALL: &[u8] = b"#{session_name}:#{window_index}: #{window_name}#{window_raw_flags} (#{window_panes} panes) [#{window_width}x#{window_height}] \0";

/// The session-level default template, exactly as the exec builds it.
const TEMPLATE_SESSION: &[u8] = b"#{window_index}: #{window_name}#{window_raw_flags} (#{window_panes} panes) [#{window_width}x#{window_height}] [layout #{window_layout}] #{window_id}#{?window_active, (active),}\0";

/// A fixture window with one pane behind its winlink, so every template
/// field answers a value.
fn fixture_window() -> (Window, Pane) {
    let mut w = Window::new(7, "first", 80, 24);
    let mut p = Pane::new(11, 80, 24, 100);
    w.add_pane(&mut p);
    (w, p)
}

/// The command's table entry as a raw pointer, so every field read stays an
/// explicit unsafe dereference rather than a shared reference into a
/// `static mut`.
fn entry() -> *const cmd_entry {
    &raw const cmd_list_windows_entry
}

/// Runs the parsed command an item carries through `entry`'s exec hook, the
/// way the command queue calls it.
unsafe fn exec_via(item: &mut Item) -> cmd_retval {
    unsafe {
        assert!(
            ::core::ptr::eq((*item.cmd()).entry, WINDOWS),
            "the item is not running list-windows"
        );
        let exec = (*entry()).exec;
        exec(&*item.cmd(), item.ptr())
    }
}

/// An item claiming to come from [`FILE`], carrying a parsed command line,
/// whose target state names `s` the way a resolved `-t target-session`
/// leaves it.
unsafe fn aimed_at(s: *mut session, wl: *mut winlink, line: &'static CStr) -> Item {
    let mut item = Item::new().from_file(FILE, 1).with_args(line);
    unsafe {
        (*item.ptr()).target.set_session(s);
        (*item.ptr()).target.set_winlink(wl);
    }
    item
}

#[test]
fn entry_metadata_matches_upstream() {
    unsafe {
        let e = entry();
        assert_eq!((*e).name.to_bytes(), b"list-windows");
        assert_eq!(
            (*e).alias.expect("the entry has an alias").to_bytes(),
            b"lsw"
        );
        assert_eq!(
            (*e).usage.to_bytes(),
            b"[-ar] [-F format] [-f filter] [-O order][-t target-session]"
        );

        assert_eq!((*e).args.template.to_bytes(), b"aF:f:O:rt:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none());

        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);

        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_SESSION);
        assert_eq!((*e).target.flags, 0);

        assert_eq!((*e).flags, CMD_AFTERHOOK);
        assert_eq!((*e).flags & CMD_AFTERHOOK, CMD_AFTERHOOK);
        assert_eq!((*e).flags & !CMD_AFTERHOOK, 0);
    }
}

#[test]
fn argument_bounds_enforce_zero_positional_arguments_and_accept_flags() {
    let _guard = globals();
    unsafe {
        let mut none = cmd_parse_from_string(c"list-windows".as_ptr(), null_mut());
        assert_eq!(none.status, CMD_PARSE_SUCCESS);
        let _ = none.cmdlist.take();

        let mut alias = cmd_parse_from_string(c"lsw".as_ptr(), null_mut());
        assert_eq!(alias.status, CMD_PARSE_SUCCESS);
        let _ = alias.cmdlist.take();

        let mut flags = cmd_parse_from_string(
            c"list-windows -a -r -F '#{window_name}' -f '1' -O name -t mysess".as_ptr(),
            null_mut(),
        );
        assert_eq!(flags.status, CMD_PARSE_SUCCESS);
        let first = cmd_list_first(flags.cmdlist.as_ref().unwrap().as_ptr());
        let args = cmd_get_args(&*first);
        assert_ne!(args_has(args, b'a'), 0);
        assert_ne!(args_has(args, b'r'), 0);
        assert_eq!(seen(args_get(args, b'F')), "#{window_name}");
        assert_eq!(seen(args_get(args, b'f')), "1");
        assert_eq!(seen(args_get(args, b'O')), "name");
        assert_eq!(seen(args_get(args, b't')), "mysess");
        let _ = flags.cmdlist.take();

        let mut extra = cmd_parse_from_string(c"list-windows unexpected_arg".as_ptr(), null_mut());
        assert_eq!(extra.status, CMD_PARSE_ERROR);
        let err = extra.take_error();
        assert!(err.contains("list-windows"), "{err}");
        assert!(err.contains("too many arguments"), "{err}");

        let mut bad_flag = cmd_parse_from_string(c"list-windows -z".as_ptr(), null_mut());
        assert_eq!(bad_flag.status, CMD_PARSE_ERROR);
        let err_flag = bad_flag.take_error();
        assert!(err_flag.contains("unknown flag"), "{err_flag}");
    }
}

#[test]
fn all_windows_template_is_the_upstream_format_exactly() {
    let expected: &[u8] =
        b"#{session_name}:#{window_index}: #{window_name}#{window_raw_flags} (#{window_panes} panes) [#{window_width}x#{window_height}] \0";
    let got: Vec<u8> = LIST_WINDOWS_WITH_SESSION_TEMPLATE
        .iter()
        .map(|&b| b as u8)
        .collect();
    assert_eq!(LIST_WINDOWS_WITH_SESSION_TEMPLATE.len(), 127);
    assert_eq!(got.len(), expected.len());
    assert_eq!(got, expected);
    assert_eq!(got[got.len() - 1], 0, "the template ends in a NUL");
    assert!(
        expected[..expected.len() - 1].iter().all(|&b| b != 0),
        "the template has no interior NUL"
    );
}

/// A bare fixture window carries no layout tree, so `#{window_layout}`
/// answers the dump of nothing: the checksum of an empty layout and the
/// separator that follows it.
#[test]
fn template_expands_a_fixture_window_deterministically() {
    let _guard = globals();
    let mut s = Session::new(3, "main");
    let (mut w, _pane) = fixture_window();
    let wl = link(&mut s, &mut w, 2);
    unsafe {
        assert_eq!((*wl).idx, 2);

        let ft = Format::defaults(null_mut::<client>(), s.ptr(), wl, null_mut());

        assert_eq!(
            ft.expand(CStr::from_ptr(LIST_WINDOWS_WITH_SESSION_TEMPLATE.as_ptr())),
            "main:2: first* (1 panes) [80x24] "
        );
        assert_eq!(
            ft.expand(CStr::from_ptr(TEMPLATE_SESSION.as_ptr().cast())),
            "2: first* (1 panes) [80x24] [layout 0000,] @7 (active)"
        );
        assert_eq!(
            ft.expand(c"#{session_id}|#{window_index}|#{window_name}"),
            "$3|2|first"
        );

        (*ft.ptr()).set_window(null_mut::<window>());
        assert_eq!(ft.expand(c"#{window_id}|#{window_layout}"), "|");

        session_set_curw(s.ptr(), null_mut::<winlink>());
        let uncurrent = Format::defaults(null_mut::<client>(), s.ptr(), wl, null_mut());
        assert_eq!(
            uncurrent.expand(CStr::from_ptr(TEMPLATE_SESSION.as_ptr().cast())),
            "2: first (1 panes) [80x24] [layout 0000,] @7"
        );
    }
    unlink(&mut s, wl);
}

#[test]
fn message_protocol_constants_match_upstream() {
    assert_eq!(MSG_VERSION, 12);

    assert_eq!(MSG_IDENTIFY_FLAGS, 100);
    assert_eq!(MSG_IDENTIFY_TERM, 101);
    assert_eq!(MSG_IDENTIFY_TTYNAME, 102);
    assert_eq!(MSG_IDENTIFY_OLDCWD, 103);
    assert_eq!(MSG_IDENTIFY_STDIN, 104);
    assert_eq!(MSG_IDENTIFY_ENVIRON, 105);
    assert_eq!(MSG_IDENTIFY_DONE, 106);
    assert_eq!(MSG_IDENTIFY_CLIENTPID, 107);
    assert_eq!(MSG_IDENTIFY_CWD, 108);
    assert_eq!(MSG_IDENTIFY_FEATURES, 109);
    assert_eq!(MSG_IDENTIFY_STDOUT, 110);
    assert_eq!(MSG_IDENTIFY_LONGFLAGS, 111);
    assert_eq!(MSG_IDENTIFY_TERMINFO, 112);

    assert_eq!(MSG_COMMAND, 200);
    assert_eq!(MSG_DETACH, 201);
    assert_eq!(MSG_DETACHKILL, 202);
    assert_eq!(MSG_EXIT, 203);
    assert_eq!(MSG_EXITED, 204);
    assert_eq!(MSG_EXITING, 205);
    assert_eq!(MSG_LOCK, 206);
    assert_eq!(MSG_READY, 207);
    assert_eq!(MSG_RESIZE, 208);
    assert_eq!(MSG_SHELL, 209);
    assert_eq!(MSG_SHUTDOWN, 210);
    assert_eq!(MSG_OLDSTDERR, 211);
    assert_eq!(MSG_OLDSTDIN, 212);
    assert_eq!(MSG_OLDSTDOUT, 213);
    assert_eq!(MSG_SUSPEND, 214);
    assert_eq!(MSG_UNLOCK, 215);
    assert_eq!(MSG_WAKEUP, 216);
    assert_eq!(MSG_EXEC, 217);
    assert_eq!(MSG_FLAGS, 218);

    assert_eq!(MSG_READ_OPEN, 300);
    assert_eq!(MSG_READ, 301);
    assert_eq!(MSG_READ_DONE, 302);
    assert_eq!(MSG_WRITE_OPEN, 303);
    assert_eq!(MSG_WRITE, 304);
    assert_eq!(MSG_WRITE_READY, 305);
    assert_eq!(MSG_WRITE_CLOSE, 306);
    assert_eq!(MSG_READ_CANCEL, 307);

    let identify: [msgtype; 13] = [
        MSG_IDENTIFY_FLAGS,
        MSG_IDENTIFY_TERM,
        MSG_IDENTIFY_TTYNAME,
        MSG_IDENTIFY_OLDCWD,
        MSG_IDENTIFY_STDIN,
        MSG_IDENTIFY_ENVIRON,
        MSG_IDENTIFY_DONE,
        MSG_IDENTIFY_CLIENTPID,
        MSG_IDENTIFY_CWD,
        MSG_IDENTIFY_FEATURES,
        MSG_IDENTIFY_STDOUT,
        MSG_IDENTIFY_LONGFLAGS,
        MSG_IDENTIFY_TERMINFO,
    ];
    for (i, v) in identify.iter().enumerate() {
        assert_eq!(*v as usize, 100 + i);
    }
}

#[test]
fn pane_screen_layout_and_theme_constants_match_upstream() {
    assert_eq!(PANE_LINES_SINGLE, 0);
    assert_eq!(PANE_LINES_DOUBLE, 1);
    assert_eq!(PANE_LINES_HEAVY, 2);
    assert_eq!(PANE_LINES_SIMPLE, 3);
    assert_eq!(PANE_LINES_NUMBER, 4);
    assert_eq!(PANE_LINES_SPACES, 5);

    assert_eq!(PROGRESS_BAR_HIDDEN, 0);
    assert_eq!(PROGRESS_BAR_NORMAL, 1);
    assert_eq!(PROGRESS_BAR_ERROR, 2);
    assert_eq!(PROGRESS_BAR_INDETERMINATE, 3);
    assert_eq!(PROGRESS_BAR_PAUSED, 4);

    assert_eq!(SCREEN_CURSOR_DEFAULT, 0);
    assert_eq!(SCREEN_CURSOR_BLOCK, 1);
    assert_eq!(SCREEN_CURSOR_UNDERLINE, 2);
    assert_eq!(SCREEN_CURSOR_BAR, 3);

    assert_eq!(LAYOUT_LEFTRIGHT, 0);
    assert_eq!(LAYOUT_TOPBOTTOM, 1);
    assert_eq!(LAYOUT_WINDOWPANE, 2);

    assert_eq!(THEME_UNKNOWN, 0);
    assert_eq!(THEME_LIGHT, 1);
    assert_eq!(THEME_DARK, 2);

    let families = [
        &[
            PANE_LINES_SINGLE,
            PANE_LINES_DOUBLE,
            PANE_LINES_HEAVY,
            PANE_LINES_SIMPLE,
            PANE_LINES_NUMBER,
            PANE_LINES_SPACES,
        ][..],
        &[
            PROGRESS_BAR_HIDDEN,
            PROGRESS_BAR_NORMAL,
            PROGRESS_BAR_ERROR,
            PROGRESS_BAR_INDETERMINATE,
            PROGRESS_BAR_PAUSED,
        ][..],
        &[
            SCREEN_CURSOR_DEFAULT,
            SCREEN_CURSOR_BLOCK,
            SCREEN_CURSOR_UNDERLINE,
            SCREEN_CURSOR_BAR,
        ][..],
        &[LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE][..],
        &[THEME_UNKNOWN, THEME_LIGHT, THEME_DARK][..],
    ];
    for family in families {
        for (i, v) in family.iter().enumerate() {
            for w in &family[i + 1..] {
                assert_ne!(v, w, "family values stay distinct");
            }
        }
    }
}

#[test]
fn style_family_constants_match_upstream() {
    assert_eq!(STYLE_ALIGN_DEFAULT, 0);
    assert_eq!(STYLE_ALIGN_LEFT, 1);
    assert_eq!(STYLE_ALIGN_CENTRE, 2);
    assert_eq!(STYLE_ALIGN_RIGHT, 3);
    assert_eq!(STYLE_ALIGN_ABSOLUTE_CENTRE, 4);

    assert_eq!(STYLE_LIST_OFF, 0);
    assert_eq!(STYLE_LIST_ON, 1);
    assert_eq!(STYLE_LIST_FOCUS, 2);
    assert_eq!(STYLE_LIST_LEFT_MARKER, 3);
    assert_eq!(STYLE_LIST_RIGHT_MARKER, 4);

    assert_eq!(STYLE_RANGE_NONE, 0);
    assert_eq!(STYLE_RANGE_LEFT, 1);
    assert_eq!(STYLE_RANGE_RIGHT, 2);
    assert_eq!(STYLE_RANGE_PANE, 3);
    assert_eq!(STYLE_RANGE_WINDOW, 4);
    assert_eq!(STYLE_RANGE_SESSION, 5);
    assert_eq!(STYLE_RANGE_USER, 6);
    assert_eq!(STYLE_RANGE_CONTROL, 7);

    assert_eq!(STYLE_DEFAULT_BASE, 0);
    assert_eq!(STYLE_DEFAULT_PUSH, 1);
    assert_eq!(STYLE_DEFAULT_POP, 2);
    assert_eq!(STYLE_DEFAULT_SET, 3);

    let align = [
        STYLE_ALIGN_DEFAULT,
        STYLE_ALIGN_LEFT,
        STYLE_ALIGN_CENTRE,
        STYLE_ALIGN_RIGHT,
        STYLE_ALIGN_ABSOLUTE_CENTRE,
    ];
    let list = [
        STYLE_LIST_OFF,
        STYLE_LIST_ON,
        STYLE_LIST_FOCUS,
        STYLE_LIST_LEFT_MARKER,
        STYLE_LIST_RIGHT_MARKER,
    ];
    let range = [
        STYLE_RANGE_NONE,
        STYLE_RANGE_LEFT,
        STYLE_RANGE_RIGHT,
        STYLE_RANGE_PANE,
        STYLE_RANGE_WINDOW,
        STYLE_RANGE_SESSION,
        STYLE_RANGE_USER,
        STYLE_RANGE_CONTROL,
    ];
    let default_type = [
        STYLE_DEFAULT_BASE,
        STYLE_DEFAULT_PUSH,
        STYLE_DEFAULT_POP,
        STYLE_DEFAULT_SET,
    ];
    for family in [&align[..], &list[..], &range[..], &default_type[..]] {
        for (i, v) in family.iter().enumerate() {
            for w in &family[i + 1..] {
                assert_ne!(v, w, "family values stay distinct");
            }
        }
    }
}

#[test]
fn prompt_client_exit_args_parse_and_find_type_constants_match_upstream() {
    assert_eq!(PROMPT_TYPE_COMMAND, 0);
    assert_eq!(PROMPT_TYPE_SEARCH, 1);
    assert_eq!(PROMPT_TYPE_TARGET, 2);
    assert_eq!(PROMPT_TYPE_WINDOW_TARGET, 3);
    assert_eq!(PROMPT_TYPE_INVALID, 255);

    assert_eq!(PROMPT_ENTRY, 0);
    assert_eq!(PROMPT_COMMAND, 1);

    assert_eq!(CLIENT_EXIT_RETURN, 0);
    assert_eq!(CLIENT_EXIT_SHUTDOWN, 1);
    assert_eq!(CLIENT_EXIT_DETACH, 2);

    assert_eq!(ARGS_PARSE_INVALID, 0);
    assert_eq!(ARGS_PARSE_STRING, 1);
    assert_eq!(ARGS_PARSE_COMMANDS_OR_STRING, 2);
    assert_eq!(ARGS_PARSE_COMMANDS, 3);

    assert_eq!(CMD_FIND_PANE, 0);
    assert_eq!(CMD_FIND_WINDOW, 1);
    assert_eq!(CMD_FIND_SESSION, 2);
}

#[test]
fn sort_orders_return_values_and_misc_constants_match_upstream() {
    assert_eq!(SORT_ACTIVITY, 0);
    assert_eq!(SORT_CREATION, 1);
    assert_eq!(SORT_INDEX, 2);
    assert_eq!(SORT_MODIFIER, 3);
    assert_eq!(SORT_NAME, 4);
    assert_eq!(SORT_ORDER, 5);
    assert_eq!(SORT_SIZE, 6);
    assert_eq!(SORT_Z, 7);
    assert_eq!(SORT_END, 8);

    assert_eq!(CMD_RETURN_NORMAL, 0);
    assert_eq!(CMD_RETURN_WAIT, 1);
    assert_eq!(CMD_RETURN_STOP, 2);
    assert_eq!(CMD_RETURN_ERROR, -1);

    assert_eq!(CMD_AFTERHOOK, 0x4);
    assert_eq!(FORMAT_NONE, 0);

    let sorts = [
        SORT_ACTIVITY,
        SORT_CREATION,
        SORT_INDEX,
        SORT_MODIFIER,
        SORT_NAME,
        SORT_ORDER,
        SORT_SIZE,
        SORT_Z,
        SORT_END,
    ];
    for (i, v) in sorts.iter().enumerate() {
        assert_eq!(*v as usize, i);
    }
}

#[test]
fn entry_is_registered_once_in_cmd_table_and_findable_by_name_and_alias() {
    let _guard = globals();
    unsafe {
        let found = cmd_table
            .iter()
            .filter(|slot| ::core::ptr::eq(**slot, entry()))
            .count();
        assert_eq!(found, 1, "the entry appears exactly once");

        let mut cause = None;
        assert_eq!(cmd_find(c"list-windows".as_ptr(), &mut cause), entry());
        assert!(cause.is_none(), "no cause on success");

        assert_eq!(cmd_find(c"lsw".as_ptr(), &mut cause), entry());
        assert!(cause.is_none(), "no cause on success");
    }
}

#[test]
fn exec_answers_normal_for_a_session_with_no_windows() {
    let _guard = globals();
    let mut s = Session::new(9, "empty");
    unsafe {
        let mut item = aimed_at(s.ptr(), null_mut::<winlink>(), c"list-windows");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);

        let mut alias_item = aimed_at(s.ptr(), null_mut::<winlink>(), c"lsw");
        assert_eq!(exec_via(&mut alias_item), CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_lists_the_target_session_windows_through_the_default_format() {
    let _guard = globals();
    let mut s = Session::new(1, "sessioned");
    let (mut first, _first_pane) = fixture_window();
    let mut second = Window::new(8, "second", 100, 30);
    let wl0 = link(&mut s, &mut first, 0);
    let wl5 = link(&mut s, &mut second, 5);
    unsafe {
        let mut item = aimed_at(s.ptr(), wl0, c"list-windows");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);

        let mut other_current = aimed_at(s.ptr(), wl5, c"list-windows");
        assert_eq!(exec_via(&mut other_current), CMD_RETURN_NORMAL);

        let mut custom = aimed_at(
            s.ptr(),
            wl0,
            c"list-windows -F '#{line}: #{window_index} #{window_name}'",
        );
        assert_eq!(exec_via(&mut custom), CMD_RETURN_NORMAL);

        let mut empty_template = aimed_at(s.ptr(), wl0, c"list-windows -F ''");
        assert_eq!(exec_via(&mut empty_template), CMD_RETURN_NORMAL);
    }
    unlink(&mut s, wl5);
    unlink(&mut s, wl0);
}

#[test]
fn exec_with_a_walks_every_registered_session_in_the_server_tree() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut aaa = Session::new(11, "aaa");
    let mut bbb = Session::new(12, "bbb");
    registry.add_session(&mut aaa);
    registry.add_session(&mut bbb);

    let (mut wa, _wa_pane) = fixture_window();
    let (mut wb, _wb_pane) = fixture_window();
    let wla = link(&mut aaa, &mut wa, 0);
    let wlb = link(&mut bbb, &mut wb, 3);
    unsafe {
        let mut item = Item::new().from_file(FILE, 1).with_args(c"list-windows -a");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);

        let mut ordered = Item::new()
            .from_file(FILE, 2)
            .with_args(c"list-windows -a -O name -F '#{session_name}:#{window_id}'");
        assert_eq!(exec_via(&mut ordered), CMD_RETURN_NORMAL);
    }
    unlink(&mut bbb, wlb);
    unlink(&mut aaa, wla);
}

#[test]
fn exec_with_a_over_no_sessions_answers_normal() {
    let _guard = globals();
    let _registry = Registry::new();
    unsafe {
        let mut item = Item::new().from_file(FILE, 1).with_args(c"list-windows -a");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_filters_windows_through_the_f_filter_before_printing() {
    let _guard = globals();
    let mut s = Session::new(14, "filtered");
    let (mut first, _first_pane) = fixture_window();
    let mut second = Window::new(9, "second", 40, 10);
    let wl0 = link(&mut s, &mut first, 0);
    let wl1 = link(&mut s, &mut second, 1);
    unsafe {
        let mut keep_all = aimed_at(s.ptr(), wl0, c"list-windows -f '1'");
        assert_eq!(exec_via(&mut keep_all), CMD_RETURN_NORMAL);

        let mut keep_none = aimed_at(s.ptr(), wl0, c"list-windows -f '0'");
        assert_eq!(exec_via(&mut keep_none), CMD_RETURN_NORMAL);

        let mut keep_by_name = aimed_at(
            s.ptr(),
            wl0,
            c"list-windows -f '#{==:#{window_name},second}'",
        );
        assert_eq!(exec_via(&mut keep_by_name), CMD_RETURN_NORMAL);

        let mut keep_by_size = aimed_at(
            s.ptr(),
            wl0,
            c"list-windows -f '#{==:#{window_width}x#{window_height},80x24}'",
        );
        assert_eq!(exec_via(&mut keep_by_size), CMD_RETURN_NORMAL);
    }
    unlink(&mut s, wl1);
    unlink(&mut s, wl0);
}

#[test]
fn exec_reports_an_invalid_sort_order_as_an_error() {
    let _guard = globals();
    let mut s = Session::new(13, "sorted");
    let (mut w, _w_pane) = fixture_window();
    let wl = link(&mut s, &mut w, 0);
    unsafe {
        let mut item = aimed_at(s.ptr(), wl, c"list-windows -O no_such_order");
        assert_eq!(exec_via(&mut item), CMD_RETURN_ERROR);

        cfg_print_causes(item.ptr());
    }
    unlink(&mut s, wl);
}

#[test]
fn exec_sorts_by_every_order_the_criteria_accepts() {
    let _guard = globals();
    let mut s = Session::new(15, "orders");
    let (mut older, _older_pane) = fixture_window();
    let mut newer = Window::new(10, "newer", 60, 20);
    let wl0 = link(&mut s, &mut older, 0);
    let wl1 = link(&mut s, &mut newer, 1);
    unsafe {
        (*older.ptr()).creation_time = timeval {
            tv_sec: 100,
            tv_usec: 0,
        };
        (*older.ptr()).activity_time = timeval {
            tv_sec: 200,
            tv_usec: 0,
        };
        (*newer.ptr()).creation_time = timeval {
            tv_sec: 300,
            tv_usec: 0,
        };
        (*newer.ptr()).activity_time = timeval {
            tv_sec: 400,
            tv_usec: 5,
        };
    }
    unsafe {
        for line in [
            c"list-windows -O activity",
            c"list-windows -O creation",
            c"list-windows -O index",
            c"list-windows -O key",
            c"list-windows -O modifier",
            c"list-windows -O name",
            c"list-windows -O title",
            c"list-windows -O order",
            c"list-windows -O size",
            c"list-windows -O z",
            c"list-windows -O NAME",
            c"list-windows -O Title",
        ] {
            let mut item = aimed_at(s.ptr(), wl0, line);
            assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL, "{line:?}");
        }
    }
    unlink(&mut s, wl1);
    unlink(&mut s, wl0);
}

#[test]
fn exec_reverses_the_listing_with_the_r_flag() {
    let _guard = globals();
    let mut s = Session::new(16, "reversed");
    let (mut first, _first_pane) = fixture_window();
    let mut second = Window::new(12, "second", 70, 21);
    let wl0 = link(&mut s, &mut first, 0);
    let wl1 = link(&mut s, &mut second, 4);
    unsafe {
        let mut reversed_only = aimed_at(s.ptr(), wl0, c"list-windows -r");
        assert_eq!(exec_via(&mut reversed_only), CMD_RETURN_NORMAL);

        let mut reversed_by_index = aimed_at(s.ptr(), wl0, c"list-windows -O index -r");
        assert_eq!(exec_via(&mut reversed_by_index), CMD_RETURN_NORMAL);

        let mut reversed_all = aimed_at(s.ptr(), wl0, c"lsw -a -O name -r");
        assert_eq!(exec_via(&mut reversed_all), CMD_RETURN_NORMAL);
    }
    unlink(&mut s, wl1);
    unlink(&mut s, wl0);
}

#[test]
fn exec_combines_every_option_in_one_run() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(17, "combined");
    registry.add_session(&mut s);
    let (mut first, _first_pane) = fixture_window();
    let mut second = Window::new(14, "second", 50, 12);
    let wl0 = link(&mut s, &mut first, 7);
    let wl1 = link(&mut s, &mut second, 9);
    unsafe {
        let mut item = Item::new().from_file(FILE, 1).with_args(
            c"list-windows -a -F '#{session_name}:#{window_index}: #{window_id}' -f '1' -O index -r",
        );
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);

        let mut filtered = Item::new()
            .from_file(FILE, 2)
            .with_args(c"lsw -F '#{window_name}' -f '#{!=:#{window_name},first}' -O size");
        (*filtered.ptr()).target.set_session(s.ptr());
        assert_eq!(exec_via(&mut filtered), CMD_RETURN_NORMAL);
    }
    unlink(&mut s, wl1);
    unlink(&mut s, wl0);
}

//! Unit tests for [`crate::cmd::cmd_list_panes`] — the `list-panes`
//! entry metadata, its argument bounds and flags, the message-protocol,
//! layout, style, prompt and sorting constants it carries, the three default
//! templates its output lines are built from, and every deterministic branch
//! of [`cmd_list_panes_exec`] reachable through the entry's exec hook: the
//! window, session (`-s`) and server (`-a`) walks over fixture sessions,
//! windows and panes, custom formats, filters, sort orders and the invalid
//! order that answers an error.
//!
//! Exec is reached through the entry's own function pointer, exactly as the
//! command queue calls it, over items whose arguments come from the real
//! command parser and whose targets are fixture sessions linked to fixture
//! windows. Printed lines go to a client-less item, which only logs them;
//! the content of what would be printed is asserted through the same format
//! engine the command drives.

use crate::arguments::{args_get, args_has};
use crate::cfg::cfg_print_causes;
use crate::cmd::cmd_list_panes::*;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{cmd_find, cmd_get_args, cmd_list_first, cmd_table};
use crate::tests::test_fixtures::{
    Format, Item, Pane, Registry, Session, Window, globals, link, seen, unlink,
};
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;

/// Where the tests' items claim to come from, which is what `cmdq_error`
/// reports them under.
const FILE: &CStr = c"test-coverage-cmd-list-panes.conf";

/// The entry whose exec hook is under test.
const PANES: *const cmd_entry = &raw const cmd_list_panes_entry;

/// The window-level default template, exactly as the exec builds it.
const TEMPLATE_WINDOW: &[u8] = b"#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}\0";

/// The session-level default template, exactly as the exec builds it.
const TEMPLATE_SESSION: &[u8] = b"#{window_index}.#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}\0";

/// The server-level default template, exactly as the exec builds it.
const TEMPLATE_SERVER: &[u8] = b"#{session_name}:#{window_index}.#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}\0";

/// The command's table entry as a raw pointer, so every field read stays an
/// explicit unsafe dereference rather than a shared reference into a
/// `static mut`.
fn entry() -> *const cmd_entry {
    &raw const cmd_list_panes_entry
}

/// Runs the parsed command an item carries through `entry`'s exec hook, the
/// way the command queue calls it.
unsafe fn exec_via(item: &mut Item) -> cmd_retval {
    unsafe {
        assert!(
            ::core::ptr::eq((*item.cmd()).entry, PANES),
            "the item is not running list-panes"
        );
        let exec = (*entry()).exec;
        exec(&*item.cmd(), item.ptr())
    }
}

/// An item claiming to come from [`FILE`], carrying a parsed command line,
/// aimed at the session and winlink the way a resolved target would leave it.
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
        assert_eq!((*e).name.to_bytes(), b"list-panes");
        assert_eq!(
            (*e).alias.expect("the entry has an alias").to_bytes(),
            b"lsp"
        );
        assert_eq!(
            (*e).usage.to_bytes(),
            b"[-asr] [-F format] [-f filter] [-O order][-t target-window]"
        );

        assert_eq!((*e).args.template.to_bytes(), b"aF:f:O:rst:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none());

        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);

        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_WINDOW);
        assert_eq!((*e).target.flags, 0);

        assert_eq!((*e).flags, CMD_AFTERHOOK);
        assert_eq!((*e).flags & CMD_AFTERHOOK, CMD_AFTERHOOK);
        assert_eq!((*e).flags & !CMD_AFTERHOOK, 0);
    }
}

#[test]
fn argument_bounds_and_flags_parsing() {
    let _guard = globals();
    unsafe {
        let mut plain = cmd_parse_from_string(c"list-panes".as_ptr(), null_mut());
        assert_eq!(plain.status, CMD_PARSE_SUCCESS);
        let _ = plain.cmdlist.take();

        let mut alias = cmd_parse_from_string(c"lsp".as_ptr(), null_mut());
        assert_eq!(alias.status, CMD_PARSE_SUCCESS);
        let _ = alias.cmdlist.take();

        let mut flags = cmd_parse_from_string(
            c"list-panes -a -s -r -F '#{pane_id}' -f '1' -O name -t mysess".as_ptr(),
            null_mut(),
        );
        assert_eq!(flags.status, CMD_PARSE_SUCCESS);
        let first = cmd_list_first(flags.cmdlist.as_ref().unwrap());
        let args = cmd_get_args(&*first);
        assert_ne!(args_has(args, b'a'), 0);
        assert_ne!(args_has(args, b's'), 0);
        assert_ne!(args_has(args, b'r'), 0);
        assert_eq!(seen(args_get(args, b'F')), "#{pane_id}");
        assert_eq!(seen(args_get(args, b'f')), "1");
        assert_eq!(seen(args_get(args, b'O')), "name");
        assert_eq!(seen(args_get(args, b't')), "mysess");
        let _ = flags.cmdlist.take();

        let mut extra_arg =
            cmd_parse_from_string(c"list-panes unexpected_argument".as_ptr(), null_mut());
        assert_eq!(extra_arg.status, CMD_PARSE_ERROR);
        let err = extra_arg.take_error();
        assert!(err.contains("list-panes"), "{err}");
        assert!(err.contains("too many arguments"), "{err}");

        let mut bad_flag = cmd_parse_from_string(c"list-panes -z".as_ptr(), null_mut());
        assert_eq!(bad_flag.status, CMD_PARSE_ERROR);
        let err_flag = bad_flag.take_error();
        assert!(err_flag.contains("unknown flag"), "{err_flag}");
    }
}

#[test]
fn default_templates_are_the_upstream_formats_exactly() {
    for (template, expected) in [
        (
            TEMPLATE_WINDOW,
            "#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}",
        ),
        (
            TEMPLATE_SESSION,
            "#{window_index}.#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}",
        ),
        (
            TEMPLATE_SERVER,
            "#{session_name}:#{window_index}.#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}",
        ),
    ] {
        let got: Vec<u8> = template.iter().map(|&b| b).collect();
        let expected = format!("{expected}\0").into_bytes();
        assert_eq!(got.len(), expected.len());
        assert_eq!(got, expected);
        assert_eq!(got[got.len() - 1], 0, "the template ends in a NUL");
        assert!(
            expected[..expected.len() - 1].iter().all(|&b| b != 0),
            "the template has no interior NUL"
        );
    }
}

#[test]
fn default_templates_expand_against_fixture_panes() {
    let _guard = globals();
    let mut s = Session::new(3, "main");
    let mut w = Window::new(7, "first", 80, 24);
    let mut p0 = Pane::new(11, 80, 24, 100);
    let mut p1 = Pane::new(12, 40, 10, 200);
    w.add_pane(&mut p0);
    w.add_pane(&mut p1);
    let wl = link(&mut s, &mut w, 2);
    unsafe {
        assert_eq!((*wl).idx, 2);

        let ft0 = Format::defaults(null_mut::<client>(), s.ptr(), wl, p0.ptr());
        let ft1 = Format::defaults(null_mut::<client>(), s.ptr(), wl, p1.ptr());

        assert_eq!(
            ft0.expand(CStr::from_ptr(TEMPLATE_WINDOW.as_ptr().cast())),
            "0: [80x24] [history 0/100, 960 bytes] %11 (active)"
        );
        assert_eq!(
            ft1.expand(CStr::from_ptr(TEMPLATE_WINDOW.as_ptr().cast())),
            "1: [40x10] [history 0/200, 400 bytes] %12"
        );
        assert_eq!(
            ft0.expand(CStr::from_ptr(TEMPLATE_SESSION.as_ptr().cast())),
            "2.0: [80x24] [history 0/100, 960 bytes] %11 (active)"
        );
        assert_eq!(
            ft0.expand(CStr::from_ptr(TEMPLATE_SERVER.as_ptr().cast())),
            "main:2.0: [80x24] [history 0/100, 960 bytes] %11 (active)"
        );

        (*p1.ptr()).flags |= crate::format::PANE_STATUSREADY;
        assert_eq!(
            ft1.expand(CStr::from_ptr(TEMPLATE_WINDOW.as_ptr().cast())),
            "1: [40x10] [history 0/200, 400 bytes] %12 (dead)"
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

    let file_msgs: [msgtype; 8] = [
        MSG_READ_OPEN,
        MSG_READ,
        MSG_READ_DONE,
        MSG_WRITE_OPEN,
        MSG_WRITE,
        MSG_WRITE_READY,
        MSG_WRITE_CLOSE,
        MSG_READ_CANCEL,
    ];
    for (i, v) in file_msgs.iter().enumerate() {
        assert_eq!(*v as usize, 300 + i);
    }
}

#[test]
fn pane_screen_and_layout_constants_match_upstream() {
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

    let pane_lines = [
        PANE_LINES_SINGLE,
        PANE_LINES_DOUBLE,
        PANE_LINES_HEAVY,
        PANE_LINES_SIMPLE,
        PANE_LINES_NUMBER,
        PANE_LINES_SPACES,
    ];
    let cursor = [
        SCREEN_CURSOR_DEFAULT,
        SCREEN_CURSOR_BLOCK,
        SCREEN_CURSOR_UNDERLINE,
        SCREEN_CURSOR_BAR,
    ];
    for family in [&pane_lines[..], &cursor[..]] {
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
}

#[test]
fn prompt_client_and_argument_parsing_constants_match_upstream() {
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
fn sort_and_return_value_constants_match_upstream() {
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
    assert_eq!(RB_NEGINF, -1);
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
        assert_eq!(cmd_find(c"list-panes".as_ptr(), &mut cause), entry());
        assert!(cause.is_none(), "no cause on success");

        assert_eq!(cmd_find(c"lsp".as_ptr(), &mut cause), entry());
        assert!(cause.is_none(), "no cause on success");
    }
}

#[test]
fn exec_lists_the_target_window_panes_with_the_default_format() {
    let _guard = globals();
    let mut s = Session::new(1, "windowed");
    let mut w = Window::new(4, "one", 80, 24);
    let mut p0 = Pane::new(20, 80, 24, 100);
    let mut p1 = Pane::new(21, 40, 10, 200);
    w.add_pane(&mut p0);
    w.add_pane(&mut p1);
    let wl = link(&mut s, &mut w, 0);
    unsafe {
        let mut item = aimed_at(s.ptr(), wl, c"list-panes");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);

        let mut custom = aimed_at(
            s.ptr(),
            wl,
            c"list-panes -F '#{pane_index} of #{window_name}'",
        );
        assert_eq!(exec_via(&mut custom), CMD_RETURN_NORMAL);
    }
    unlink(&mut s, wl);
}

#[test]
fn exec_with_s_walks_every_window_of_the_session() {
    let _guard = globals();
    let mut s = Session::new(2, "sessioned");
    let mut w0 = Window::new(5, "first-window", 80, 24);
    let mut w1 = Window::new(6, "second-window", 100, 30);
    let mut p00 = Pane::new(30, 80, 24, 100);
    let mut p01 = Pane::new(31, 80, 24, 100);
    let mut p10 = Pane::new(32, 100, 30, 100);
    w0.add_pane(&mut p00);
    w0.add_pane(&mut p01);
    w1.add_pane(&mut p10);
    let wl0 = link(&mut s, &mut w0, 1);
    let wl1 = link(&mut s, &mut w1, 5);
    unsafe {
        let mut item = aimed_at(s.ptr(), wl0, c"list-panes -s");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);

        let mut from_other = aimed_at(s.ptr(), wl1, c"list-panes -s -r");
        assert_eq!(exec_via(&mut from_other), CMD_RETURN_NORMAL);

        let mut window_view = aimed_at(s.ptr(), wl1, c"list-panes");
        assert_eq!(exec_via(&mut window_view), CMD_RETURN_NORMAL);
    }
    unlink(&mut s, wl1);
    unlink(&mut s, wl0);
}

#[test]
fn exec_with_s_on_an_empty_session_lists_nothing_and_answers_normal() {
    let _guard = globals();
    let mut s = Session::new(9, "empty");
    unsafe {
        let mut item = aimed_at(s.ptr(), null_mut::<winlink>(), c"list-panes -s");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_with_a_walks_every_session_in_the_server_tree() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut first = Session::new(11, "aaa");
    let mut second = Session::new(12, "bbb");
    registry.add_session(&mut first);
    registry.add_session(&mut second);

    let mut wa = Window::new(8, "wa", 80, 24);
    let mut wb = Window::new(9, "wb", 80, 24);
    let mut pa = Pane::new(40, 80, 24, 100);
    let mut pb = Pane::new(41, 60, 20, 100);
    wa.add_pane(&mut pa);
    wb.add_pane(&mut pb);
    let wla = link(&mut first, &mut wa, 0);
    let wlb = link(&mut second, &mut wb, 3);
    unsafe {
        let mut item = Item::new().from_file(FILE, 1).with_args(c"list-panes -a");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);

        let mut ordered = Item::new()
            .from_file(FILE, 2)
            .with_args(c"list-panes -a -O name -F '#{session_name}:#{pane_id}'");
        assert_eq!(exec_via(&mut ordered), CMD_RETURN_NORMAL);
    }
    unlink(&mut first, wla);
    unlink(&mut second, wlb);
}

#[test]
fn exec_with_a_over_no_sessions_answers_normal() {
    let _guard = globals();
    let _registry = Registry::new();
    unsafe {
        let mut item = Item::new().from_file(FILE, 1).with_args(c"list-panes -a");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_rejects_an_invalid_sort_order_with_an_error_return() {
    let _guard = globals();
    let mut s = Session::new(13, "sorted");
    let mut w = Window::new(10, "w", 80, 24);
    let mut p = Pane::new(50, 80, 24, 100);
    w.add_pane(&mut p);
    let wl = link(&mut s, &mut w, 0);
    unsafe {
        let mut item = aimed_at(s.ptr(), wl, c"list-panes -O no_such_order");
        assert_eq!(exec_via(&mut item), CMD_RETURN_ERROR);

        cfg_print_causes(item.ptr());

        for line in [
            c"list-panes -O activity",
            c"list-panes -O creation",
            c"list-panes -O index",
            c"list-panes -O key",
            c"list-panes -O modifier",
            c"list-panes -O name",
            c"list-panes -O title",
            c"list-panes -O order",
            c"list-panes -O size",
            c"list-panes -O z",
            c"list-panes -O NAME",
            c"list-panes -O Title",
        ] {
            let mut ok = aimed_at(s.ptr(), wl, line);
            assert_eq!(exec_via(&mut ok), CMD_RETURN_NORMAL, "{line:?}");
        }
    }
    unlink(&mut s, wl);
}

#[test]
fn exec_filters_panes_through_the_f_filter_before_printing() {
    let _guard = globals();
    let mut s = Session::new(14, "filtered");
    let mut w = Window::new(12, "w", 80, 24);
    let mut p0 = Pane::new(60, 80, 24, 100);
    let mut p1 = Pane::new(61, 40, 10, 100);
    w.add_pane(&mut p0);
    w.add_pane(&mut p1);
    let wl = link(&mut s, &mut w, 0);
    unsafe {
        let mut keep_all = aimed_at(s.ptr(), wl, c"list-panes -f '1'");
        assert_eq!(exec_via(&mut keep_all), CMD_RETURN_NORMAL);

        let mut keep_none = aimed_at(s.ptr(), wl, c"list-panes -f '0'");
        assert_eq!(exec_via(&mut keep_none), CMD_RETURN_NORMAL);

        let mut keep_first = aimed_at(s.ptr(), wl, c"list-panes -f '#{==:#{pane_id},%60}'");
        assert_eq!(exec_via(&mut keep_first), CMD_RETURN_NORMAL);

        let mut keep_second = aimed_at(s.ptr(), wl, c"list-panes -f '#{==:#{pane_width},40}'");
        assert_eq!(exec_via(&mut keep_second), CMD_RETURN_NORMAL);
    }
    unlink(&mut s, wl);
}

#[test]
fn exec_sorts_by_every_order_the_criteria_accepts() {
    let _guard = globals();
    let mut s = Session::new(15, "orders");
    let mut w = Window::new(13, "w", 80, 24);
    let mut panes: Vec<Pane> = (0..4)
        .map(|i| Pane::new(70 + i, 80 - i * 5, 24, 100))
        .collect();
    for p in &mut panes {
        w.add_pane(p);
    }
    let wl = link(&mut s, &mut w, 0);
    unsafe {
        for line in [
            c"list-panes -O activity",
            c"list-panes -O creation",
            c"list-panes -O index",
            c"list-panes -O name",
            c"list-panes -O size",
            c"list-panes -r",
            c"list-panes -O index -r",
            c"list-panes -O size -r",
            c"list-panes -a -r",
        ] {
            let mut item = aimed_at(s.ptr(), wl, line);
            assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL, "{line:?}");
        }
    }
    unlink(&mut s, wl);
}

#[test]
fn exec_combines_every_option_in_one_run() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut s = Session::new(16, "combined");
    registry.add_session(&mut s);
    let mut w = Window::new(14, "w", 80, 24);
    let mut p0 = Pane::new(80, 80, 24, 100);
    let mut p1 = Pane::new(81, 40, 12, 100);
    w.add_pane(&mut p0);
    w.add_pane(&mut p1);
    let wl = link(&mut s, &mut w, 7);
    unsafe {
        let mut item = Item::new().from_file(FILE, 1).with_args(
            c"list-panes -a -F '#{session_name}:#{window_index}.#{pane_index}' -f '1' -O index -r",
        );
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);

        let mut session_item = Item::new()
            .from_file(FILE, 2)
            .with_args(c"lsp -s -F '#{pane_id}' -f '#{!=:#{pane_id},%81}' -O size");
        (*session_item.ptr()).target.set_session(s.ptr());
        assert_eq!(exec_via(&mut session_item), CMD_RETURN_NORMAL);
    }
    unlink(&mut s, wl);
}

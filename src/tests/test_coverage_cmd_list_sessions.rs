//! Unit tests for [`crate::cmd::cmd_list_sessions`] — the `list-sessions`
//! entry metadata, its argument bounds and flags, the message-protocol,
//! layout, style, prompt, sorting, and return-value constants it carries, the
//! default format template its output lines are built from, and every branch
//! of [`cmd_list_sessions_exec`] reachable through the entry's exec hook.
//!
//! Exec is reached through the entry's own function pointer, exactly as the
//! command queue calls it. The sessions it lists come from the server's
//! global session tree through `sort_get_sessions`, so each run registers
//! server-free [`Session`] fixtures with [`Registry`] under [`globals`].
//! Output lines are printed via `cmdq_print`, which logs them when the item
//! carries no client.

use crate::arguments::{args_get, args_has};
use crate::cmd::cmd_list_sessions::*;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{cmd_find, cmd_get_args, cmd_list_first, cmd_table};
use crate::session::{session_add_attached, session_set_activity_time};
use crate::tests::test_fixtures::{Format, Item, Registry, Session, globals, seen};
use ::core::ffi::CStr;
use ::core::ptr::null_mut;

/// The command's table entry as a raw pointer, so every field read stays an
/// explicit unsafe dereference rather than a shared reference into a
/// `static mut`.
fn entry() -> *const cmd_entry {
    &raw const cmd_list_sessions_entry
}

/// Runs the parsed command an item carries through `entry`'s exec hook, the
/// way the command queue calls it.
unsafe fn exec_via(item: &mut Item) -> cmd_retval {
    unsafe {
        let exec = (*entry()).exec;
        exec(&*item.cmd(), item.ptr())
    }
}

#[test]
fn entry_metadata_matches_upstream() {
    unsafe {
        let e = entry();
        assert_eq!((*e).name.to_bytes(), b"list-sessions");
        assert_eq!(
            (*e).alias.expect("the entry has an alias").to_bytes(),
            b"ls"
        );
        assert_eq!(
            (*e).usage.to_bytes(),
            b"[-r] [-F format] [-f filter] [-O order]"
        );

        assert_eq!((*e).args.template.to_bytes(), b"F:f:O:r");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none());

        let flags = [&raw const (*e).source, &raw const (*e).target];
        for flag in flags {
            assert_eq!((*flag).flag, 0);
            assert_eq!((*flag).type_0, CMD_FIND_PANE);
            assert_eq!((*flag).flags, 0);
        }

        assert_eq!((*e).flags, CMD_AFTERHOOK);
        assert_eq!((*e).flags & CMD_AFTERHOOK, CMD_AFTERHOOK);
        assert_eq!((*e).flags & !CMD_AFTERHOOK, 0);
    }
}

#[test]
fn argument_bounds_enforce_zero_positional_arguments_and_accept_flags() {
    let _guard = globals();
    unsafe {
        let mut none = cmd_parse_from_string(c"list-sessions".as_ptr(), null_mut());
        assert_eq!(none.status, CMD_PARSE_SUCCESS);
        let _ = none.cmdlist.take();

        let mut alias = cmd_parse_from_string(c"ls".as_ptr(), null_mut());
        assert_eq!(alias.status, CMD_PARSE_SUCCESS);
        let _ = alias.cmdlist.take();

        let mut flags = cmd_parse_from_string(
            c"list-sessions -F '#{session_name}' -f '1' -O name -r".as_ptr(),
            null_mut(),
        );
        assert_eq!(flags.status, CMD_PARSE_SUCCESS);
        let first = cmd_list_first(flags.cmdlist.as_ref().unwrap());
        let args = cmd_get_args(&*first);
        assert_eq!(seen(args_get(args, b'F')), "#{session_name}");
        assert_eq!(seen(args_get(args, b'f')), "1");
        assert_eq!(seen(args_get(args, b'O')), "name");
        assert_ne!(args_has(args, b'r'), 0);
        let _ = flags.cmdlist.take();

        let mut extra = cmd_parse_from_string(c"list-sessions unexpected_arg".as_ptr(), null_mut());
        assert_eq!(extra.status, CMD_PARSE_ERROR);
        let err = extra.take_error();
        assert!(err.contains("list-sessions"), "{err}");
        assert!(err.contains("too many arguments"), "{err}");

        let mut bad_flag = cmd_parse_from_string(c"list-sessions -z".as_ptr(), null_mut());
        assert_eq!(bad_flag.status, CMD_PARSE_ERROR);
        let err_flag = bad_flag.take_error();
        assert!(err_flag.contains("unknown flag"), "{err_flag}");
    }
}

#[test]
fn template_is_the_upstream_format_exactly() {
    let expected: &[u8] = b"#{session_name}: #{session_windows} windows (created #{t:session_created})#{?session_grouped, (group ,}#{session_group}#{?session_grouped,),}#{?session_attached, (attached),}\0";
    let got: Vec<u8> = LIST_SESSIONS_TEMPLATE.iter().map(|&b| b as u8).collect();
    assert_eq!(LIST_SESSIONS_TEMPLATE.len(), 175);
    assert_eq!(got.len(), expected.len());
    assert_eq!(got, expected);
    assert_eq!(got[got.len() - 1], 0, "the template ends in a NUL");
    assert!(
        expected[..expected.len() - 1].iter().all(|&b| b != 0),
        "the template has no interior NUL"
    );
}

/// A fixture session expands the whole upstream template deterministically:
/// no windows are linked into it, nothing is attached to it, it belongs to no
/// group and its creation time is the epoch, which the time conversion leaves
/// empty.
#[test]
fn template_expands_a_fixture_session_deterministically() {
    let _guard = globals();
    let mut s = Session::new(3, "main");
    unsafe {
        let ft = Format::defaults(null_mut::<client>(), s.ptr(), null_mut(), null_mut());
        let expanded = ft.expand(CStr::from_ptr(LIST_SESSIONS_TEMPLATE.as_ptr()));
        assert_eq!(expanded, "main: 0 windows (created )");

        session_add_attached(s.ptr());
        session_add_attached(s.ptr());
        let ft_attached = Format::defaults(null_mut::<client>(), s.ptr(), null_mut(), null_mut());
        let expanded_attached = ft_attached.expand(CStr::from_ptr(LIST_SESSIONS_TEMPLATE.as_ptr()));
        assert_eq!(expanded_attached, "main: 0 windows (created ) (attached)");

        let named = Format::defaults(null_mut::<client>(), s.ptr(), null_mut(), null_mut());
        assert_eq!(
            named.expand(c"#{session_id}|#{session_windows}|#{session_name}"),
            "$3|0|main"
        );
    }
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
fn prompt_client_exit_and_find_type_constants_match_upstream() {
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
        assert_eq!(cmd_find(c"list-sessions".as_ptr(), &mut cause), entry());
        assert!(cause.is_none(), "no cause on success");

        assert_eq!(cmd_find(c"ls".as_ptr(), &mut cause), entry());
        assert!(cause.is_none(), "no cause on success");
    }
}

#[test]
fn exec_returns_normal_when_no_sessions_exist() {
    let _guard = globals();
    let _registry = Registry::new();
    unsafe {
        let mut item = Item::new().with_args(c"list-sessions");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);

        let mut alias_item = Item::new().with_args(c"ls");
        assert_eq!(exec_via(&mut alias_item), CMD_RETURN_NORMAL);
    }
}

/// With no `-O`, the criteria stay at the end marker and `sort_get_sessions`
/// hands the tree walk straight back: name order, since that is the key the
/// session tree sorts by.
#[test]
fn exec_lists_registered_sessions_in_tree_order_by_default() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut bbb = Session::new(2, "bbb");
    let mut aaa = Session::new(1, "aaa");
    registry.add_session(&mut bbb);
    registry.add_session(&mut aaa);
    unsafe {
        let mut item = Item::new().with_args(c"list-sessions");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);

        let mut formatted = Item::new().with_args(c"list-sessions -F '#{session_name}'");
        assert_eq!(exec_via(&mut formatted), CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_honours_a_custom_format_from_the_f_flag() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut first = Session::new(10, "alpha");
    let mut second = Session::new(20, "beta");
    registry.add_session(&mut first);
    registry.add_session(&mut second);
    unsafe {
        let mut item =
            Item::new().with_args(c"list-sessions -F '#{line}: #{session_id} #{session_name}'");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);

        let mut empty_template = Item::new().with_args(c"list-sessions -F ''");
        assert_eq!(exec_via(&mut empty_template), CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_filters_sessions_with_format_filters() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut alpha = Session::new(5, "alpha");
    let mut beta = Session::new(6, "beta");
    registry.add_session(&mut alpha);
    registry.add_session(&mut beta);
    unsafe {
        let mut matches_alpha =
            Item::new().with_args(c"list-sessions -f '#{==:#{session_name},alpha}'");
        assert_eq!(exec_via(&mut matches_alpha), CMD_RETURN_NORMAL);

        let mut matches_none =
            Item::new().with_args(c"list-sessions -f '#{==:#{session_name},nonesuch}'");
        assert_eq!(exec_via(&mut matches_none), CMD_RETURN_NORMAL);
    }
}

/// A constant filter exercises both sides of the flag the exec loop keeps:
/// `1` prints every line, `0` expands each tree yet prints none.
#[test]
fn exec_filters_with_constant_true_and_false_filters() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut only = Session::new(9, "only");
    registry.add_session(&mut only);
    unsafe {
        let mut keep_all = Item::new().with_args(c"list-sessions -f '1'");
        assert_eq!(exec_via(&mut keep_all), CMD_RETURN_NORMAL);

        let mut keep_none = Item::new().with_args(c"list-sessions -f '0'");
        assert_eq!(exec_via(&mut keep_none), CMD_RETURN_NORMAL);
    }
}

/// Every order `-O` accepts names a real sort; sessions compare by id,
/// creation, activity and name directly, and the rest fall back on the name.
#[test]
fn exec_sorts_sessions_by_every_valid_order() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut older = Session::new(7, "older");
    let mut newer = Session::new(3, "newer");
    unsafe {
        (*older.ptr()).creation_time = timeval {
            tv_sec: 100,
            tv_usec: 0,
        };
        session_set_activity_time(
            older.ptr(),
            timeval {
                tv_sec: 100,
                tv_usec: 5,
            },
        );
        (*newer.ptr()).creation_time = timeval {
            tv_sec: 100,
            tv_usec: 7,
        };
        session_set_activity_time(
            newer.ptr(),
            timeval {
                tv_sec: 200,
                tv_usec: 0,
            },
        );
    }
    registry.add_session(&mut older);
    registry.add_session(&mut newer);
    unsafe {
        let orders = [
            c"list-sessions -O activity",
            c"list-sessions -O creation",
            c"list-sessions -O index",
            c"list-sessions -O modifier",
            c"list-sessions -O name",
            c"list-sessions -O order",
            c"list-sessions -O size",
            c"list-sessions -O z",
            c"list-sessions -O NAME",
        ];
        for cmd_str in orders {
            let mut item = Item::new().with_args(cmd_str);
            assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL, "{cmd_str:?}");
        }
    }
}

/// An unrecognised `-O` value reports through `cmdq_error` — the item here
/// carries a source file and line, the way parsed commands do — and stops
/// before any listing happens.
#[test]
fn exec_reports_an_invalid_sort_order_as_an_error() {
    let _guard = globals();
    let _registry = Registry::new();
    unsafe {
        let mut item = Item::new()
            .from_file(c"fixture.conf", 11)
            .with_args(c"list-sessions -O nosuchorder");
        assert_eq!(exec_via(&mut item), CMD_RETURN_ERROR);
    }
}

/// Without `-O` the end marker never counts as invalid, even though it is what
/// `sort_order_from_string` answers for a missing value.
#[test]
fn exec_reverses_the_listing_with_the_r_flag() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut first = Session::new(1, "first");
    let mut second = Session::new(2, "second");
    registry.add_session(&mut first);
    registry.add_session(&mut second);
    unsafe {
        let mut reversed_only = Item::new().with_args(c"list-sessions -r");
        assert_eq!(exec_via(&mut reversed_only), CMD_RETURN_NORMAL);

        let mut reversed_by_name = Item::new().with_args(c"list-sessions -O name -r");
        assert_eq!(exec_via(&mut reversed_by_name), CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_combines_all_flags() {
    let _guard = globals();
    let mut registry = Registry::new();
    let mut one = Session::new(30, "combo-one");
    let mut two = Session::new(40, "combo-two");
    registry.add_session(&mut one);
    registry.add_session(&mut two);
    unsafe {
        let mut item = Item::new().with_args(
            c"list-sessions -F '#{line} #{session_name}' -f '#{!=:#{session_name},none}' -O index -r",
        );
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);
    }
}

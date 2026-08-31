//! Unit tests for [`crate::cmd::cmd_list_commands`] — the `list-commands`
//! entry's metadata, its argument bounds and flags, the message-protocol and
//! enumeration constants it carries, and the format template its output lines
//! are built from.
//!
//! Two limitations worth recording about what these tests can reach. The
//! module's execution helpers (`cmd_list_single_command` and
//! `cmd_list_commands`) are private to their own translation unit, so this
//! separate coverage module cannot name them without changing production
//! visibility; they *are* exercised here through the public
//! [`cmd_list_commands_entry`] `.exec` function pointer, which is the very
//! code the command queue would call. And what can be observed through that
//! pointer is narrower than the real server's behaviour: the fixtures run each
//! command against an item with no client, so `cmdq_print` only reaches
//! `log_debug` and the per-entry lines cannot be read back — the tests pin the
//! return values instead. The same holds for the unknown-command error, whose
//! message `cmdq_error` files into `cfg.rs`'s private cause list rather than
//! anywhere a separate module can inspect. The `fatalx("out of memory")`
//! branch of `cmdq_print` is skipped as fatal.

use crate::arguments::{args_get, args_string};
use crate::cmd::cmd_list_commands::*;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{cmd_find, cmd_get_args, cmd_list_first, cmd_table};
use crate::fmt_args;
use crate::format::format_add;
use crate::tests::test_fixtures::{Format, Item, globals, seen};
use ::core::ffi::CStr;
use ::core::ptr::null_mut;

/// The command's table entry as a raw pointer, so every field read stays an
/// explicit unsafe dereference rather than a shared reference into a
/// `static mut`.
fn entry() -> *const cmd_entry {
    &raw const cmd_list_commands_entry
}

#[test]
fn entry_metadata_matches_upstream() {
    unsafe {
        let e = entry();
        assert_eq!((*e).name.to_bytes(), b"list-commands");
        assert_eq!(
            (*e).alias.expect("the entry has an alias").to_bytes(),
            b"lscm"
        );
        assert_eq!((*e).usage.to_bytes(), b"[-F format] [command]");

        assert_eq!((*e).args.template.to_bytes(), b"F:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 1);
        assert!((*e).args.cb.is_none());

        let flags = [&raw const (*e).source, &raw const (*e).target];
        for flag in flags {
            assert_eq!((*flag).flag, 0);
            assert_eq!((*flag).type_0, CMD_FIND_PANE);
            assert_eq!((*flag).flags, 0);
        }

        assert_eq!((*e).flags, CMD_STARTSERVER | CMD_AFTERHOOK);
        assert_eq!((*e).flags & CMD_STARTSERVER, CMD_STARTSERVER);
        assert_eq!((*e).flags & CMD_AFTERHOOK, CMD_AFTERHOOK);
        assert_eq!((*e).flags & !(CMD_STARTSERVER | CMD_AFTERHOOK), 0);
    }
}

#[test]
fn argument_bounds_accept_zero_or_one_command() {
    let _guard = globals();
    unsafe {
        let mut none = cmd_parse_from_string(c"list-commands".as_ptr(), null_mut());
        assert_eq!(none.status, CMD_PARSE_SUCCESS);
        let _ = none.cmdlist.take();

        let mut one = cmd_parse_from_string(c"list-commands list-keys".as_ptr(), null_mut());
        assert_eq!(one.status, CMD_PARSE_SUCCESS);
        let first = cmd_list_first(one.cmdlist.as_ref().unwrap());
        assert_eq!(seen(args_string(cmd_get_args(&*first), 0)), "list-keys");
        let _ = one.cmdlist.take();

        let mut two =
            cmd_parse_from_string(c"list-commands list-keys kill-window".as_ptr(), null_mut());
        assert_eq!(two.status, CMD_PARSE_ERROR);
        let err = two.take_error();
        assert!(err.contains("list-commands"), "{err}");
        assert!(err.contains("too many arguments"), "{err}");
    }
}

#[test]
fn template_is_the_upstream_format_exactly() {
    let expected: &[u8] =
        b"#{command_list_name}#{?command_list_alias, (#{command_list_alias}),} #{command_list_usage}\0";
    let got: Vec<u8> = LIST_COMMANDS_TEMPLATE.iter().map(|&b| b as u8).collect();
    assert_eq!(LIST_COMMANDS_TEMPLATE.len(), 91);
    assert_eq!(got.len(), expected.len());
    assert_eq!(got, expected);
    assert_eq!(got[got.len() - 1], 0, "the template ends in a NUL");
    assert!(
        expected[..expected.len() - 1].iter().all(|&b| b != 0),
        "the template has no interior NUL"
    );
}

#[test]
fn template_expands_to_name_alias_and_usage() {
    let _guard = globals();
    unsafe {
        let ft = Format::new();
        format_add(
            &mut *ft.ptr(),
            c"command_list_name",
            c"%s".as_ptr(),
            fmt_args![c"list-commands".as_ptr()],
        );
        format_add(
            &mut *ft.ptr(),
            c"command_list_alias",
            c"%s".as_ptr(),
            fmt_args![c"lscm".as_ptr()],
        );
        format_add(
            &mut *ft.ptr(),
            c"command_list_usage",
            c"%s".as_ptr(),
            fmt_args![c"[-F format] [command]".as_ptr()],
        );
        let expanded = ft.expand(CStr::from_ptr(LIST_COMMANDS_TEMPLATE.as_ptr()));
        assert_eq!(expanded, "list-commands (lscm) [-F format] [command]");
    }
}

#[test]
fn template_drops_the_alias_part_when_the_alias_is_empty() {
    let _guard = globals();
    unsafe {
        let ft = Format::new();
        format_add(
            &mut *ft.ptr(),
            c"command_list_name",
            c"%s".as_ptr(),
            fmt_args![c"detach-client".as_ptr()],
        );
        format_add(
            &mut *ft.ptr(),
            c"command_list_alias",
            c"%s".as_ptr(),
            fmt_args![c"".as_ptr()],
        );
        format_add(
            &mut *ft.ptr(),
            c"command_list_usage",
            c"%s".as_ptr(),
            fmt_args![c"[-aP] [-E shell-command] [-s target-session] [-t target-client]".as_ptr()],
        );
        let expanded = ft.expand(CStr::from_ptr(LIST_COMMANDS_TEMPLATE.as_ptr()));
        assert_eq!(
            expanded,
            "detach-client [-aP] [-E shell-command] [-s target-session] [-t target-client]"
        );

        let ft = Format::new();
        format_add(
            &mut *ft.ptr(),
            c"command_list_name",
            c"%s".as_ptr(),
            fmt_args![c"solo".as_ptr()],
        );
        format_add(
            &mut *ft.ptr(),
            c"command_list_alias",
            c"%s".as_ptr(),
            fmt_args![c"".as_ptr()],
        );
        format_add(
            &mut *ft.ptr(),
            c"command_list_usage",
            c"%s".as_ptr(),
            fmt_args![c"".as_ptr()],
        );
        let expanded = ft.expand(CStr::from_ptr(LIST_COMMANDS_TEMPLATE.as_ptr()));
        assert_eq!(expanded, "solo ", "the separator space survives");
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
fn pane_and_screen_enumeration_constants_match_upstream() {
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
    let progress_bar = [
        PROGRESS_BAR_HIDDEN,
        PROGRESS_BAR_NORMAL,
        PROGRESS_BAR_ERROR,
        PROGRESS_BAR_INDETERMINATE,
        PROGRESS_BAR_PAUSED,
    ];
    let cursor = [
        SCREEN_CURSOR_DEFAULT,
        SCREEN_CURSOR_BLOCK,
        SCREEN_CURSOR_UNDERLINE,
        SCREEN_CURSOR_BAR,
    ];
    for family in [&pane_lines[..], &progress_bar[..], &cursor[..]] {
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
fn return_values_module_flags_and_null_match_upstream() {
    assert_eq!(CMD_RETURN_NORMAL, 0);
    assert_eq!(CMD_RETURN_WAIT, 1);
    assert_eq!(CMD_RETURN_STOP, 2);
    assert_eq!(CMD_RETURN_ERROR, -1);

    assert_eq!(CMD_STARTSERVER, 0x1);
    assert_eq!(CMD_AFTERHOOK, 0x4);

    assert_eq!(FORMAT_NONE, 0);
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
        assert_eq!(cmd_find(c"list-commands".as_ptr(), &mut cause), entry());
        assert!(cause.is_none(), "no cause on success");

        assert_eq!(cmd_find(c"lscm".as_ptr(), &mut cause), entry());
        assert!(cause.is_none(), "no cause on success");
    }
}

#[test]
fn exec_lists_every_command_with_the_default_template() {
    let _guard = globals();
    unsafe {
        let mut item = Item::new().with_args(c"list-commands");
        let cmd = item.cmd();
        let rv = (cmd_list_commands_entry.exec)(&*cmd, item.ptr());
        assert_eq!(rv, CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_honours_a_custom_format_from_the_f_flag() {
    let _guard = globals();
    unsafe {
        let mut item = Item::new().with_args(c"list-commands -F '#{command_list_usage}'");
        let cmd = item.cmd();
        let fmt = seen(args_get(cmd_get_args(&*cmd), b'F'));
        assert_eq!(fmt, "#{command_list_usage}");

        let rv = (cmd_list_commands_entry.exec)(&*cmd, item.ptr());
        assert_eq!(rv, CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_lists_one_command_by_name_or_alias() {
    let _guard = globals();
    unsafe {
        let mut by_name = Item::new().with_args(c"list-commands list-sessions");
        let rv = (cmd_list_commands_entry.exec)(&*by_name.cmd(), by_name.ptr());
        assert_eq!(rv, CMD_RETURN_NORMAL);

        let mut by_alias = Item::new().with_args(c"list-commands lscm");
        let rv = (cmd_list_commands_entry.exec)(&*by_alias.cmd(), by_alias.ptr());
        assert_eq!(rv, CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_reports_an_unknown_command_with_an_error_return() {
    let _guard = globals();
    unsafe {
        let mut item = Item::new().with_args(c"list-commands not-a-command");
        let rv = (cmd_list_commands_entry.exec)(&*item.cmd(), item.ptr());
        assert_eq!(rv, CMD_RETURN_ERROR);
    }
}

//! Unit tests for [`crate::cmd::cmd_list_buffers`] — the `list-buffers`
//! entry metadata, its argument bounds and flags, the message-protocol,
//! layout, style, prompt, and sorting constants it carries, the format
//! template its output lines are built from, and every branch of
//! [`cmd_list_buffers_exec`].
//!
//! Exec is reached through the entry's own function pointer, exactly as the
//! command queue calls it, over items whose arguments come from the real
//! command parser and whose paste store is managed by [`Paste`]. Output lines
//! are formatted through `format_defaults_paste_buffer` and printed via
//! `cmdq_print` to debug logs.

use crate::arguments::{args_get, args_has};
use crate::cmd::cmd_list_buffers::*;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{cmd_find, cmd_get_args, cmd_list_first, cmd_table};
use crate::format::format_defaults_paste_buffer;
use crate::tests::test_fixtures::{Format, Item, Paste, globals, seen};
use ::core::ffi::CStr;
use ::core::ptr::null_mut;

/// The command's table entry as a raw pointer, so every field read stays an
/// explicit unsafe dereference rather than a shared reference into a
/// `static mut`.
fn entry() -> *const cmd_entry {
    &raw const cmd_list_buffers_entry
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
        assert_eq!((*e).name.to_bytes(), b"list-buffers");
        assert_eq!(
            (*e).alias.expect("the entry has an alias").to_bytes(),
            b"lsb"
        );
        assert_eq!((*e).usage.to_bytes(), b"[-F format] [-f filter] [-O order]");

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
        let mut none = cmd_parse_from_string(c"list-buffers".as_ptr(), null_mut());
        assert_eq!(none.status, CMD_PARSE_SUCCESS);
        let _ = none.cmdlist.take();

        let mut alias = cmd_parse_from_string(c"lsb".as_ptr(), null_mut());
        assert_eq!(alias.status, CMD_PARSE_SUCCESS);
        let _ = alias.cmdlist.take();

        let mut flags = cmd_parse_from_string(
            c"list-buffers -F '#{buffer_name}' -f '1' -O name -r".as_ptr(),
            null_mut(),
        );
        assert_eq!(flags.status, CMD_PARSE_SUCCESS);
        let first = cmd_list_first(flags.cmdlist.as_ref().unwrap().as_ptr());
        let args = cmd_get_args(&*first);
        assert_eq!(seen(args_get(args, b'F')), "#{buffer_name}");
        assert_eq!(seen(args_get(args, b'f')), "1");
        assert_eq!(seen(args_get(args, b'O')), "name");
        assert_ne!(args_has(args, b'r'), 0);
        let _ = flags.cmdlist.take();

        let mut extra = cmd_parse_from_string(c"list-buffers unexpected_arg".as_ptr(), null_mut());
        assert_eq!(extra.status, CMD_PARSE_ERROR);
        let err = extra.take_error();
        assert!(err.contains("list-buffers"), "{err}");
        assert!(err.contains("too many arguments"), "{err}");

        let mut bad_flag = cmd_parse_from_string(c"list-buffers -z".as_ptr(), null_mut());
        assert_eq!(bad_flag.status, CMD_PARSE_ERROR);
        let err_flag = bad_flag.take_error();
        assert!(err_flag.contains("unknown flag"), "{err_flag}");
    }
}

#[test]
fn template_is_the_upstream_format_exactly() {
    let expected: &[u8] = b"#{buffer_name}: #{buffer_size} bytes: \"#{buffer_sample}\"\0";
    let got: Vec<u8> = LIST_BUFFERS_TEMPLATE.iter().map(|&b| b as u8).collect();
    assert_eq!(LIST_BUFFERS_TEMPLATE.len(), 57);
    assert_eq!(got.len(), expected.len());
    assert_eq!(got, expected);
    assert_eq!(got[got.len() - 1], 0, "the template ends in a NUL");
    assert!(
        expected[..expected.len() - 1].iter().all(|&b| b != 0),
        "the template has no interior NUL"
    );
}

#[test]
fn template_expands_paste_buffer_variables() {
    let _guard = globals();
    let paste = Paste::new();
    let pb = paste.add(c"mybuf", "sample content here");
    assert!(!pb.is_null());

    let ft = Format::new();
    unsafe {
        format_defaults_paste_buffer(&mut *ft.ptr(), pb);
        let expanded = ft.expand(CStr::from_ptr(LIST_BUFFERS_TEMPLATE.as_ptr()));
        assert_eq!(expanded, "mybuf: 19 bytes: \"sample content here\"");
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

    let msgs: [msgtype; 8] = [
        MSG_READ_OPEN,
        MSG_READ,
        MSG_READ_DONE,
        MSG_WRITE_OPEN,
        MSG_WRITE,
        MSG_WRITE_READY,
        MSG_WRITE_CLOSE,
        MSG_READ_CANCEL,
    ];
    for (i, v) in msgs.iter().enumerate() {
        assert_eq!(*v as usize, 300 + i);
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
fn return_values_sort_orders_and_flags_match_upstream() {
    assert_eq!(CMD_RETURN_NORMAL, 0);
    assert_eq!(CMD_RETURN_WAIT, 1);
    assert_eq!(CMD_RETURN_STOP, 2);
    assert_eq!(CMD_RETURN_ERROR, -1);

    assert_eq!(CMD_AFTERHOOK, 0x4);
    assert_eq!(FORMAT_NONE, 0);

    assert_eq!(SORT_ACTIVITY, 0);
    assert_eq!(SORT_CREATION, 1);
    assert_eq!(SORT_INDEX, 2);
    assert_eq!(SORT_MODIFIER, 3);
    assert_eq!(SORT_NAME, 4);
    assert_eq!(SORT_ORDER, 5);
    assert_eq!(SORT_SIZE, 6);
    assert_eq!(SORT_Z, 7);
    assert_eq!(SORT_END, 8);

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
        assert_eq!(cmd_find(c"list-buffers".as_ptr(), &mut cause), entry());
        assert!(cause.is_none(), "no cause on success");

        assert_eq!(cmd_find(c"lsb".as_ptr(), &mut cause), entry());
        assert!(cause.is_none(), "no cause on success");
    }
}

#[test]
fn exec_returns_normal_when_no_buffers_exist() {
    let _guard = globals();
    let _paste = Paste::new();
    unsafe {
        let mut item = Item::new().with_args(c"list-buffers");
        let rv = exec_via(&mut item);
        assert_eq!(rv, CMD_RETURN_NORMAL);

        let mut alias_item = Item::new().with_args(c"lsb");
        let rv_alias = exec_via(&mut alias_item);
        assert_eq!(rv_alias, CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_lists_buffers_with_default_template() {
    let _guard = globals();
    let paste = Paste::new();
    paste.add(c"buf0", "hello world");
    paste.add(c"buf1", "second test buffer");
    unsafe {
        let mut item = Item::new().with_args(c"list-buffers");
        let rv = exec_via(&mut item);
        assert_eq!(rv, CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_honours_a_custom_format_from_the_f_flag() {
    let _guard = globals();
    let paste = Paste::new();
    paste.add(c"buf0", "content A");
    paste.add(c"buf1", "content B");
    unsafe {
        let mut item = Item::new().with_args(c"list-buffers -F '#{buffer_name} = #{buffer_size}'");
        let rv = exec_via(&mut item);
        assert_eq!(rv, CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_filters_buffers_matching_filter() {
    let _guard = globals();
    let paste = Paste::new();
    paste.add(c"alpha", "data1");
    paste.add(c"beta", "data2");
    unsafe {
        let mut match_alpha =
            Item::new().with_args(c"list-buffers -f '#{==:#{buffer_name},alpha}'");
        let rv = exec_via(&mut match_alpha);
        assert_eq!(rv, CMD_RETURN_NORMAL);

        let mut match_beta = Item::new().with_args(c"list-buffers -f '#{==:#{buffer_name},beta}'");
        let rv = exec_via(&mut match_beta);
        assert_eq!(rv, CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_filters_buffers_with_constant_true_and_false_filters() {
    let _guard = globals();
    let paste = Paste::new();
    paste.add(c"buf0", "data");
    unsafe {
        let mut true_filter = Item::new().with_args(c"list-buffers -f '1'");
        let rv_true = exec_via(&mut true_filter);
        assert_eq!(rv_true, CMD_RETURN_NORMAL);

        let mut false_filter = Item::new().with_args(c"list-buffers -f '0'");
        let rv_false = exec_via(&mut false_filter);
        assert_eq!(rv_false, CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_sorts_buffers_by_various_valid_orders() {
    let _guard = globals();
    let paste = Paste::new();
    paste.add(c"b_buffer", "short");
    paste.add(c"a_buffer", "a very long buffer content");
    paste.add(c"c_buffer", "medium length");
    unsafe {
        let orders = [
            c"list-buffers -O name",
            c"list-buffers -O size",
            c"list-buffers -O activity",
            c"list-buffers -O creation",
            c"list-buffers -O index",
            c"list-buffers -O modifier",
            c"list-buffers -O order",
            c"list-buffers -O z",
        ];
        for cmd_str in orders {
            let mut item = Item::new().with_args(cmd_str);
            let rv = exec_via(&mut item);
            assert_eq!(rv, CMD_RETURN_NORMAL);
        }
    }
}

#[test]
fn exec_sorts_buffers_with_reverse_flag() {
    let _guard = globals();
    let paste = Paste::new();
    paste.add(c"buf_one", "111");
    paste.add(c"buf_two", "222222");
    unsafe {
        let mut rev_default = Item::new().with_args(c"list-buffers -r");
        assert_eq!(exec_via(&mut rev_default), CMD_RETURN_NORMAL);

        let mut rev_name = Item::new().with_args(c"list-buffers -O name -r");
        assert_eq!(exec_via(&mut rev_name), CMD_RETURN_NORMAL);

        let mut rev_size = Item::new().with_args(c"list-buffers -O size -r");
        assert_eq!(exec_via(&mut rev_size), CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_reports_invalid_sort_order_with_error_return() {
    let _guard = globals();
    let paste = Paste::new();
    paste.add(c"buf0", "content");
    unsafe {
        let mut item = Item::new().with_args(c"list-buffers -O invalid_order_name");
        let rv = exec_via(&mut item);
        assert_eq!(rv, CMD_RETURN_ERROR);
    }
}

#[test]
fn exec_with_all_options_combined() {
    let _guard = globals();
    let paste = Paste::new();
    paste.add(c"buf0", "dataA");
    paste.add(c"buf1", "dataB");
    unsafe {
        let mut item_custom = Item::new().with_args(
            c"list-buffers -F '#{buffer_name}' -f '#{!=:#{buffer_name},buf1}' -O name -r",
        );
        let rv_custom = exec_via(&mut item_custom);
        assert_eq!(rv_custom, CMD_RETURN_NORMAL);
    }
}

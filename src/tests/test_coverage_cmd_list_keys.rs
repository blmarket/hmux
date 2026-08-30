//! Unit tests for [`crate::cmd::cmd_list_keys`] — the `list-keys` entry's
//! metadata, its argument bounds and flags, the message-protocol, enumeration,
//! style, prompt and sorting constants it carries, the mouse-key encodings it
//! redeclares, the format template its output lines are built from, and every
//! deterministic branch of [`cmd_list_keys_exec`].
//!
//! Exec is reached through the entry's own function pointer, exactly as the
//! command queue calls it, over items whose arguments come from the real
//! command parser. That one path is what the private helpers behind it
//! (`cmd_list_keys_get_prefix`, the width helpers, the root-and-prefix merge,
//! the key-list filter and the per-binding format filler) all serve, so no
//! production visibility is widened to reach them by name.
//!
//! Bindings land in the process-wide `key_tables` tree, so every exec test
//! holds [`globals`] and brings a [`Tables`] guard that takes its tables back
//! down even through a failed assertion; the `-N` path reads the *default*
//! `prefix` and `root` tables, so those tests clear and rebuild them first and
//! remove them again afterwards. With no client behind the item, a line for a
//! single binding reaches `status_message_set`, which files it into the
//! server's message log — where the tests read it back — while lines for
//! several bindings only reach `cmdq_print` and then `log_debug`, so those
//! tests pin return values instead.

use crate::arguments::{args_get, args_has, args_string};
use crate::cmd::cmd_list_keys::*;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{cmd_find, cmd_get_args, cmd_list_first, cmd_table};
use crate::fmt_args;
use crate::key_bindings::{key_bindings_add, key_bindings_remove_table};
use crate::server::message_log;
use crate::tests::test_fixtures::{Item, globals, seen};
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::{null, null_mut};
use ::std::ffi::CString;

/// Where the tests' items claim to come from, which is what `cmdq_error`
/// reports them under.
const FILE: &CStr = c"test-coverage-cmd-list-keys.conf";

/// The command's table entry as a raw pointer, so every field read stays an
/// explicit unsafe dereference rather than a shared reference into a
/// `static mut`.
fn entry() -> *const cmd_entry {
    &raw const cmd_list_keys_entry
}

/// Runs the parsed command an item carries through `entry`'s exec hook, the
/// way the command queue calls it.
unsafe fn exec_via(item: &mut Item) -> cmd_retval {
    unsafe {
        let exec = (*entry()).exec;
        exec(&*item.cmd(), item.ptr())
    }
}

/// An item carrying a parsed `list-keys` command line, sourced from [`FILE`].
fn list_item(line: &'static CStr) -> Item {
    Item::new().from_file(FILE, 1).with_args(line)
}

/// The tables a test touched, taken back down again when the guard goes away —
/// even through a failed assertion — so the global tree is left as found.
struct Tables(Vec<&'static CStr>);

impl Tables {
    fn new() -> Tables {
        Tables(Vec::new())
    }

    /// Remembers `name` for cleanup without creating anything.
    fn track(&mut self, name: &'static CStr) {
        self.0.push(name);
    }

    /// Makes sure `name` does not exist now, remembering it so that it stays
    /// gone whatever the rest of the test does.
    fn clear(&mut self, name: &'static CStr) {
        self.track(name);
        unsafe { key_bindings_remove_table(name.as_ptr()) };
    }
}

impl Drop for Tables {
    fn drop(&mut self) {
        for name in &self.0 {
            unsafe { key_bindings_remove_table(name.as_ptr()) };
        }
    }
}

/// Parses one command line and hands back the list it built, reference
/// included, ready to be handed over to a binding that will free it.
unsafe fn parsed_list(s: &CStr) -> CmdListRef {
    unsafe {
        let mut pr = cmd_parse_from_string(s.as_ptr(), null_mut::<cmd_parse_input>());
        assert_eq!(pr.status, CMD_PARSE_SUCCESS, "{s:?} did not parse");
        pr.cmdlist.take().unwrap()
    }
}

/// Binds `key` in the table named `table` to `display-panes`, with an optional
/// note and repeat flag, through the real `key_bindings_add`.
unsafe fn bind(table: &CStr, key: key_code, note: Option<&CStr>, repeat: c_int) {
    unsafe {
        key_bindings_add(
            table.as_ptr(),
            key,
            note.map_or(null::<c_char>(), |n| n.as_ptr()),
            repeat,
            Some(parsed_list(c"display-panes")),
        );
    }
}

/// A watch on the server's message log across one exec, held by the number of
/// the newest line at the time it started: `server_add_message` numbers every
/// line it records and never reuses a number.
struct StatusMessages {
    before: Option<u_int>,
}

impl StatusMessages {
    fn watch() -> StatusMessages {
        StatusMessages {
            before: message_log.queue().back().map(|m| m.msg_num),
        }
    }

    /// The line logged since the watch started, if there is one. A client-less
    /// status message lands here as `message: <line>` via
    /// `server_add_message`.
    unsafe fn since(&self) -> Option<String> {
        unsafe {
            let tail = message_log.queue().back()?;
            if Some(tail.msg_num) == self.before {
                None
            } else {
                Some(seen(tail.msg.as_ptr()))
            }
        }
    }
}

#[test]
fn entry_metadata_matches_upstream() {
    unsafe {
        let e = entry();
        assert_eq!((*e).name.to_bytes(), b"list-keys");
        assert_eq!(
            (*e).alias.expect("the entry has an alias").to_bytes(),
            b"lsk"
        );
        assert_eq!(
            (*e).usage.to_bytes(),
            b"[-1aNr] [-F format] [-O order] [-P prefix-string][-T key-table] [key]"
        );

        assert_eq!((*e).args.template.to_bytes(), b"1aF:NO:P:rT:");
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
fn argument_bounds_accept_one_positional_and_the_documented_flags() {
    let _guard = globals();
    unsafe {
        let mut none = cmd_parse_from_string(c"list-keys".as_ptr(), null_mut());
        assert_eq!(none.status, CMD_PARSE_SUCCESS);
        let _ = none.cmdlist.take();

        let mut alias = cmd_parse_from_string(c"lsk".as_ptr(), null_mut());
        assert_eq!(alias.status, CMD_PARSE_SUCCESS);
        let _ = alias.cmdlist.take();

        let mut flags = cmd_parse_from_string(
            c"list-keys -1aNr -F fmt -O name -P pfx -T tbl C-b".as_ptr(),
            null_mut(),
        );
        assert_eq!(flags.status, CMD_PARSE_SUCCESS);
        let list = flags.cmdlist.as_ref().unwrap();
        let first = cmd_list_first(list.as_ptr());
        let args = cmd_get_args(&*first);
        assert_eq!(seen(args_get(args, b'F')), "fmt");
        assert_eq!(seen(args_get(args, b'O')), "name");
        assert_eq!(seen(args_get(args, b'P')), "pfx");
        assert_eq!(seen(args_get(args, b'T')), "tbl");
        assert_ne!(args_has(args, b'1'), 0);
        assert_ne!(args_has(args, b'a'), 0);
        assert_ne!(args_has(args, b'N'), 0);
        assert_ne!(args_has(args, b'r'), 0);
        assert_eq!(seen(args_string(args, 0)), "C-b");
        let _ = flags.cmdlist.take();

        let mut extra = cmd_parse_from_string(c"list-keys C-b C-c".as_ptr(), null_mut());
        assert_eq!(extra.status, CMD_PARSE_ERROR);
        let err = extra.take_error();
        assert!(err.contains("list-keys"), "{err}");
        assert!(err.contains("too many arguments"), "{err}");

        let mut bad_flag = cmd_parse_from_string(c"list-keys -z".as_ptr(), null_mut());
        assert_eq!(bad_flag.status, CMD_PARSE_ERROR);
        let err_flag = bad_flag.take_error();
        assert!(err_flag.contains("unknown flag"), "{err_flag}");
    }
}

/// The template as the pieces the upstream source writes it in, joined.
fn expected_template() -> Vec<u8> {
    let mut expected: Vec<u8> = Vec::new();
    expected.extend_from_slice(b"#{?notes_only,");
    expected.extend_from_slice(b"#{key_prefix}");
    expected.extend_from_slice(b" ");
    expected.extend_from_slice(b"#{p|#{key_string_width}:key_string}");
    expected.extend_from_slice(b" ");
    expected.extend_from_slice(b"#{?key_note,#{key_note},#{key_command}}");
    expected.extend_from_slice(b",bind-key ");
    expected.extend_from_slice(b"#{?key_has_repeat,#{?key_repeat,-r,  },}");
    expected.extend_from_slice(b" -T ");
    expected.extend_from_slice(b"#{p|#{key_table_width}:key_table}");
    expected.extend_from_slice(b" ");
    expected.extend_from_slice(b"#{p|#{key_string_width}:#{q|a:key_string}}");
    expected.extend_from_slice(b" ");
    expected.extend_from_slice(b"#{key_command}}");
    expected
}

#[test]
fn template_is_the_upstream_format_exactly() {
    let expected = expected_template();
    let got: Vec<u8> = LIST_KEYS_TEMPLATE.iter().map(|&b| b as u8).collect();
    assert_eq!(LIST_KEYS_TEMPLATE.len(), 250);
    assert!(expected.len() < got.len(), "the string fits the array");
    assert_eq!(&got[..expected.len()], &expected[..]);
    assert_eq!(got[expected.len()], 0, "the string ends in a NUL");
    assert!(
        got[expected.len() + 1..].iter().all(|&b| b == 0),
        "the rest of the array is padding"
    );
}

#[test]
fn template_expands_the_key_variables_it_names() {
    let _guard = globals();
    unsafe {
        let ft = crate::tests::test_fixtures::Format::new();
        let mut add_str = |name: &CStr, value: &CStr| {
            crate::format::format_add(
                &mut *ft.ptr(),
                name,
                c"%s".as_ptr(),
                fmt_args![value.as_ptr()],
            );
        };
        crate::format::format_add(
            &mut *ft.ptr(),
            c"notes_only",
            c"%d".as_ptr(),
            fmt_args![0 as c_int],
        );
        crate::format::format_add(
            &mut *ft.ptr(),
            c"key_has_repeat",
            c"%d".as_ptr(),
            fmt_args![1 as c_int],
        );
        crate::format::format_add(
            &mut *ft.ptr(),
            c"key_string_width",
            c"%u".as_ptr(),
            fmt_args![1 as u_int],
        );
        crate::format::format_add(
            &mut *ft.ptr(),
            c"key_table_width",
            c"%u".as_ptr(),
            fmt_args![11 as u_int],
        );
        add_str(c"key_prefix", c"C-b");
        add_str(c"key_repeat", c"-r");
        add_str(c"key_note", c"a note");
        add_str(c"key_command", c"display-panes");
        add_str(c"key_table", c"lk-template");
        add_str(c"key_string", c"x");
        let expanded = ft.expand(CStr::from_ptr(LIST_KEYS_TEMPLATE.as_ptr()));
        assert!(expanded.contains("bind-key"), "{expanded}");
        assert!(expanded.contains("-r"), "{expanded}");
        assert!(expanded.contains("-T"), "{expanded}");
        assert!(expanded.contains("lk-template"), "{expanded}");
        assert!(expanded.contains("display-panes"), "{expanded}");
        assert!(expanded.contains("x"), "{expanded}");
        assert!(
            !expanded.contains("C-b") && !expanded.contains("a note"),
            "the notes_only branch is skipped: {expanded}"
        );
    }
}

#[test]
fn template_shows_the_command_when_a_binding_has_no_note() {
    let _guard = globals();
    unsafe {
        let ft = crate::tests::test_fixtures::Format::new();
        let mut add_str = |name: &CStr, value: &CStr| {
            crate::format::format_add(
                &mut *ft.ptr(),
                name,
                c"%s".as_ptr(),
                fmt_args![value.as_ptr()],
            );
        };
        crate::format::format_add(
            &mut *ft.ptr(),
            c"notes_only",
            c"%d".as_ptr(),
            fmt_args![1 as c_int],
        );
        crate::format::format_add(
            &mut *ft.ptr(),
            c"key_has_repeat",
            c"%d".as_ptr(),
            fmt_args![0 as c_int],
        );
        crate::format::format_add(
            &mut *ft.ptr(),
            c"key_string_width",
            c"%u".as_ptr(),
            fmt_args![1 as u_int],
        );
        crate::format::format_add(
            &mut *ft.ptr(),
            c"key_table_width",
            c"%u".as_ptr(),
            fmt_args![11 as u_int],
        );
        add_str(c"key_prefix", c"C-b");
        add_str(c"key_repeat", c" ");
        add_str(c"key_command", c"display-panes");
        add_str(c"key_table", c"lk-template");
        add_str(c"key_string", c"x");
        let expanded = ft.expand(CStr::from_ptr(LIST_KEYS_TEMPLATE.as_ptr()));
        assert!(expanded.starts_with("C-b "), "{expanded}");
        assert!(expanded.contains("x"), "{expanded}");
        assert!(expanded.contains("display-panes"), "{expanded}");
        assert!(
            !expanded.contains("bind-key"),
            "the bind-key form is skipped when listing notes only: {expanded}"
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
fn return_values_sort_orders_and_key_flags_match_upstream() {
    assert_eq!(CMD_RETURN_NORMAL, 0);
    assert_eq!(CMD_RETURN_WAIT, 1);
    assert_eq!(CMD_RETURN_STOP, 2);
    assert_eq!(CMD_RETURN_ERROR, -1);

    assert_eq!(CMD_STARTSERVER, 0x1);
    assert_eq!(CMD_AFTERHOOK, 0x4);
    assert_eq!(FORMAT_NONE, 0);
    assert_eq!(KEY_BINDING_REPEAT, 0x1);
    assert_eq!(CMD_LIST_PRINT_ESCAPED, 0x1);
    assert_eq!(CMD_LIST_PRINT_NO_GROUPS, 0x2);

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
fn special_and_mouse_key_constants_match_upstream() {
    assert_eq!(KEYC_NONE, 0x2_0000_0000);
    assert_eq!(KEYC_UNKNOWN, 0x2_0000_0001);
    assert_eq!(KEYC_FOCUS_IN, 0x2_0000_0002);
    assert_eq!(KEYC_FOCUS_OUT, 0x2_0000_0003);
    assert_eq!(KEYC_ANY, 0x2_0000_0004);
    assert_eq!(KEYC_PASTE_START, 0x2_0000_0005);
    assert_eq!(KEYC_PASTE_END, 0x2_0000_0006);
    assert_eq!(KEYC_USER, 0x1_0000_0000);
    assert_eq!(KEYC_MASK_KEY, 0xff_ffff_ffff);
    assert_eq!(KEYC_MASK_MODIFIERS, 0xff00_0000_0000);

    let pane_base = [
        KEYC_MOUSEDRAG_PANE,
        KEYC_MOUSEDRAGEND_PANE,
        KEYC_SECONDCLICK_PANE,
        KEYC_DOUBLECLICK_PANE,
        KEYC_TRIPLECLICK_PANE,
    ];
    assert_eq!(pane_base[0], 25769803776);
    assert_eq!(pane_base[1], 30064771072);
    assert_eq!(pane_base[2], 42949672960);
    assert_eq!(pane_base[3], 47244640256);
    assert_eq!(pane_base[4], 51539607552);
    for (i, v) in pane_base.iter().enumerate() {
        for w in &pane_base[i + 1..] {
            assert_ne!(v, w, "mouse families stay distinct");
        }
    }

    assert_eq!(KEYC_TRIPLECLICK_STATUS, KEYC_TRIPLECLICK_PANE + 1);
    assert_eq!(KEYC_TRIPLECLICK_BORDER, KEYC_TRIPLECLICK_STATUS + 4);
    assert_eq!(
        KEYC_DOUBLECLICK_CONTROL0,
        KEYC_DOUBLECLICK_SCROLLBAR_DOWN + 1
    );
    assert_eq!(
        KEYC_TRIPLECLICK_PANE - KEYC_DOUBLECLICK_PANE,
        KEYC_USER,
        "each click count steps one user range"
    );
    assert_eq!(KEYC_MOUSEDRAGEND_PANE - KEYC_MOUSEDRAG_PANE, KEYC_USER);

    let triple = [
        KEYC_TRIPLECLICK_PANE,
        KEYC_TRIPLECLICK_STATUS,
        KEYC_TRIPLECLICK_BORDER,
        KEYC_TRIPLECLICK_SCROLLBAR_UP,
        KEYC_TRIPLECLICK_SCROLLBAR_DOWN,
        KEYC_TRIPLECLICK_SCROLLBAR_SLIDER,
        KEYC_TRIPLECLICK_CONTROL0,
    ];
    for (i, v) in triple.iter().enumerate() {
        for w in &triple[i + 1..] {
            assert_ne!(v, w, "triple-click targets stay distinct");
        }
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
        assert_eq!(cmd_find(c"list-keys".as_ptr(), &mut cause), entry());
        assert!(cause.is_none(), "no cause on success");

        assert_eq!(cmd_find(c"lsk".as_ptr(), &mut cause), entry());
        assert!(cause.is_none(), "no cause on success");
    }
}

#[test]
fn exec_returns_normal_over_whatever_tables_exist() {
    let _guard = globals();
    unsafe {
        let mut item = list_item(c"list-keys");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);

        let mut alias_item = list_item(c"lsk");
        assert_eq!(exec_via(&mut alias_item), CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_lists_a_scoped_table_with_the_default_template() {
    let _guard = globals();
    let mut ts = Tables::new();
    let table = c"lk-scope";
    ts.clear(table);
    unsafe {
        bind(table, b'a' as key_code, None, 0);
        bind(table, b'd' as key_code, Some(c"third"), 0);
        bind(table, b'b' as key_code, None, 1);

        let mut item = list_item(c"list-keys -T lk-scope");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_reports_an_invalid_key_with_an_error_return() {
    let _guard = globals();
    let messages = StatusMessages::watch();
    unsafe {
        let mut item = list_item(c"list-keys NoSuchKeyX");
        assert_eq!(exec_via(&mut item), CMD_RETURN_ERROR);
        assert_eq!(messages.since(), None, "nothing was listed");

        let mut hex_item = list_item(c"list-keys 0xzz");
        assert_eq!(exec_via(&mut hex_item), CMD_RETURN_ERROR);
    }
}

#[test]
fn exec_errors_when_the_named_table_does_not_exist() {
    let _guard = globals();
    let mut ts = Tables::new();
    let missing = c"lk-nowhere";
    ts.clear(missing);
    unsafe {
        let mut item = list_item(c"list-keys -T lk-nowhere");
        assert_eq!(exec_via(&mut item), CMD_RETURN_ERROR);
    }
}

#[test]
fn exec_rejects_an_invalid_sort_order_with_an_error_return() {
    let _guard = globals();
    unsafe {
        let mut item = list_item(c"list-keys -O not_an_order");
        assert_eq!(exec_via(&mut item), CMD_RETURN_ERROR);
    }
}

#[test]
fn exec_filters_to_one_key_and_prints_it_as_a_status_message() {
    let _guard = globals();
    let mut ts = Tables::new();
    let table = c"lk-filter";
    ts.clear(table);
    unsafe {
        bind(table, b'a' as key_code, None, 0);
        bind(table, b'b' as key_code, None, 0);

        let messages = StatusMessages::watch();
        let mut item = list_item(c"list-keys -T lk-filter -F '#{key_string}' b");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(
            messages.since(),
            Some("message: b".to_string()),
            "the one surviving line is reported as a status message"
        );
    }
}

#[test]
fn exec_formats_prefix_note_repeat_table_command_and_notes_only() {
    let _guard = globals();
    let mut ts = Tables::new();
    let table = c"lk-fields";
    ts.clear(table);
    unsafe {
        bind(table, b'x' as key_code, Some(c"the note"), 1);

        let messages = StatusMessages::watch();
        let mut item = list_item(
            c"list-keys -T lk-fields -F '#{key_repeat}|#{key_note}|#{key_prefix}|#{key_table}|#{key_command}|#{notes_only}'",
        );
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(
            messages.since(),
            Some("message: 1|the note|C-b|lk-fields|display-panes|0".to_string()),
        );
    }
}

#[test]
fn exec_takes_the_prefix_string_from_the_p_flag() {
    let _guard = globals();
    let mut ts = Tables::new();
    let table = c"lk-prefix-flag";
    ts.clear(table);
    unsafe {
        bind(table, b'y' as key_code, None, 0);

        let messages = StatusMessages::watch();
        let mut item = list_item(c"list-keys -T lk-prefix-flag -P C-x -F '#{key_prefix}'");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(messages.since(), Some("message: C-x".to_string()),);
    }
}

#[test]
fn exec_stops_after_the_first_line_with_the_1_flag() {
    let _guard = globals();
    let mut ts = Tables::new();
    let table = c"lk-single";
    ts.clear(table);
    unsafe {
        bind(table, b'c' as key_code, None, 0);
        bind(table, b'a' as key_code, None, 0);
        bind(table, b'd' as key_code, None, 0);

        let messages = StatusMessages::watch();
        let mut item = list_item(c"list-keys -T lk-single -1 -F '#{key_string}'");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(
            messages.since(),
            Some("message: a".to_string()),
            "exactly the first binding in key order is listed"
        );
    }
}

#[test]
fn exec_prints_nothing_for_a_line_that_expands_to_empty() {
    let _guard = globals();
    let mut ts = Tables::new();
    let table = c"lk-empty-line";
    ts.clear(table);
    unsafe {
        bind(table, b'a' as key_code, None, 0);
        bind(table, b'b' as key_code, None, 0);

        let messages = StatusMessages::watch();
        let mut item = list_item(c"list-keys -T lk-empty-line -F ''");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(messages.since(), None, "no empty line was printed");
    }
}

#[test]
fn exec_sorts_a_table_by_every_valid_order_forward_and_reversed() {
    let _guard = globals();
    let mut ts = Tables::new();
    let table = c"lk-sort";
    ts.clear(table);
    unsafe {
        bind(table, b'b' as key_code, None, 0);
        bind(table, b'a' as key_code, None, 0);

        for order in [
            "activity", "creation", "index", "modifier", "name", "order", "size", "z",
        ] {
            let plain = CString::new(format!("-O {order}")).expect("no NUL");
            let mut line: Vec<u8> = b"list-keys -T lk-sort ".to_vec();
            line.extend_from_slice(plain.as_bytes());
            line.push(0);
            let line = CStr::from_bytes_with_nul(&line).expect("no interior NUL");
            let mut item = Item::new().from_file(FILE, 1).with_args(line);
            assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL, "{order:?}");

            let reversed_line: Vec<u8> = {
                let mut v = b"list-keys -T lk-sort ".to_vec();
                v.extend_from_slice(plain.as_bytes());
                v.extend_from_slice(b" -r");
                v.push(0);
                v
            };
            let reversed_line = CStr::from_bytes_with_nul(&reversed_line).expect("no interior NUL");
            let mut rev_item = Item::new().from_file(FILE, 1).with_args(reversed_line);
            assert_eq!(exec_via(&mut rev_item), CMD_RETURN_NORMAL, "{order:?} -r");
        }
    }
}

#[test]
fn exec_lists_only_noted_bindings_from_root_and_prefix_without_a() {
    let _guard = globals();
    let mut ts = Tables::new();
    ts.clear(c"prefix");
    ts.clear(c"root");
    unsafe {
        bind(c"prefix", b'x' as key_code, Some(c"kept"), 0);
        bind(c"root", b'w' as key_code, None, 0);

        let messages = StatusMessages::watch();
        let mut item = list_item(c"list-keys -N -F '#{key_string}'");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(
            messages.since(),
            Some("message: x".to_string()),
            "only the noted prefix-table binding survives the notes filter"
        );

        let all = StatusMessages::watch();
        let mut unfiltered = list_item(c"list-keys -N -a");
        assert_eq!(exec_via(&mut unfiltered), CMD_RETURN_NORMAL);
        assert_eq!(
            all.since(),
            None,
            "two bindings are printed through cmdq_print, which logs only"
        );
    }
}

#[test]
fn exec_filters_root_and_prefix_by_key_when_one_is_named() {
    let _guard = globals();
    let mut ts = Tables::new();
    ts.clear(c"prefix");
    ts.clear(c"root");
    unsafe {
        bind(c"prefix", b'x' as key_code, None, 0);
        bind(c"root", b'w' as key_code, Some(c"noted"), 0);

        let messages = StatusMessages::watch();
        let mut item = list_item(c"list-keys -N -F '#{key_string}' w");
        assert_eq!(exec_via(&mut item), CMD_RETURN_NORMAL);
        assert_eq!(
            messages.since(),
            Some("message: w".to_string()),
            "the named key wins even though it comes from the root table"
        );
    }
}

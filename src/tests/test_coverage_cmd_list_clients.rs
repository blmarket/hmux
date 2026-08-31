//! Unit tests for [`crate::cmd::cmd_list_clients`] — the `list-clients`
//! entry metadata, its argument bounds and flags, the message-protocol and
//! enumeration constants it carries, the default format template, and every
//! execution path of [`cmd_list_clients_exec`] reachable through the entry's
//! exec hook.

use crate::arguments::{args_get, args_has};
use crate::cmd::cmd_list_clients::*;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{cmd_find, cmd_get_args, cmd_list_first, cmd_table};
use crate::ffi::getuid;
use crate::proc::PEER_BAD;
use crate::server::{CLIENT_ATTACHED, CLIENT_DEAD, CLIENT_EXIT, CLIENT_READONLY, CLIENT_SUSPENDED};
use crate::tests::test_fixtures::{Clients, Format, Item, Session, globals, seen, zeroed};
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;

/// A peer marked bad so no unexpected message transmission occurs, owned by
/// whichever client it is wired to.
fn bad_peer() -> Box<tmuxpeer> {
    let mut p = zeroed::<tmuxpeer>();
    p.flags |= PEER_BAD;
    p.uid = unsafe { getuid() };
    p
}

/// The command's table entry as a raw pointer.
fn entry() -> *const cmd_entry {
    &raw const cmd_list_clients_entry
}

/// Connects a client to a peer, a session, terminal name, and specific flags.
unsafe fn wire_client(
    c: *mut client,
    session: *mut session,
    term_name: &'static CStr,
    flags: uint64_t,
) {
    unsafe {
        (*c).peer = Some(bad_peer());
        (*c).session = session;
        (*c).flags = flags;
        (*c).term_name = Some(term_name.to_owned());
        (*c).tty.flags |= crate::tty::TTY_STARTED;
    }
}

#[test]
fn entry_metadata_matches_upstream() {
    unsafe {
        let e = entry();
        assert_eq!((*e).name.to_bytes(), b"list-clients");
        assert_eq!(
            (*e).alias.expect("the entry has an alias").to_bytes(),
            b"lsc"
        );
        assert_eq!(
            (*e).usage.to_bytes(),
            b"[-F format] [-f filter] [-O order][-t target-session]"
        );

        assert_eq!((*e).args.template.to_bytes(), b"F:f:O:rt:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 0);
        assert!((*e).args.cb.is_none());

        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);

        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_SESSION);
        assert_eq!((*e).target.flags, 0);

        assert_eq!((*e).flags, CMD_READONLY | CMD_AFTERHOOK);
        assert_eq!((*e).flags & CMD_READONLY, CMD_READONLY);
        assert_eq!((*e).flags & CMD_AFTERHOOK, CMD_AFTERHOOK);
        assert_eq!((*e).flags & !(CMD_READONLY | CMD_AFTERHOOK), 0);
    }
}

#[test]
fn argument_bounds_and_flags_parsing() {
    let _guard = globals();
    unsafe {
        let mut plain = cmd_parse_from_string(c"list-clients".as_ptr(), null_mut());
        assert_eq!(plain.status, CMD_PARSE_SUCCESS);
        let _ = plain.cmdlist.take();

        let mut with_format =
            cmd_parse_from_string(c"list-clients -F '#{client_name}'".as_ptr(), null_mut());
        assert_eq!(with_format.status, CMD_PARSE_SUCCESS);
        let first = cmd_list_first(with_format.cmdlist.as_ref().unwrap());
        assert_eq!(
            seen(args_get(cmd_get_args(&*first), b'F')),
            "#{client_name}"
        );
        let _ = with_format.cmdlist.take();

        let mut with_filter = cmd_parse_from_string(
            c"list-clients -f '#{!=:#{client_name},c1}'".as_ptr(),
            null_mut(),
        );
        assert_eq!(with_filter.status, CMD_PARSE_SUCCESS);
        let first = cmd_list_first(with_filter.cmdlist.as_ref().unwrap());
        assert_eq!(
            seen(args_get(cmd_get_args(&*first), b'f')),
            "#{!=:#{client_name},c1}"
        );
        let _ = with_filter.cmdlist.take();

        let mut with_order = cmd_parse_from_string(c"list-clients -O name".as_ptr(), null_mut());
        assert_eq!(with_order.status, CMD_PARSE_SUCCESS);
        let first = cmd_list_first(with_order.cmdlist.as_ref().unwrap());
        assert_eq!(seen(args_get(cmd_get_args(&*first), b'O')), "name");
        let _ = with_order.cmdlist.take();

        let mut with_reverse = cmd_parse_from_string(c"list-clients -r".as_ptr(), null_mut());
        assert_eq!(with_reverse.status, CMD_PARSE_SUCCESS);
        let first = cmd_list_first(with_reverse.cmdlist.as_ref().unwrap());
        assert_eq!(args_has(cmd_get_args(&*first), b'r'), 1);
        let _ = with_reverse.cmdlist.take();

        let mut with_target = cmd_parse_from_string(c"list-clients -t mysess".as_ptr(), null_mut());
        assert_eq!(with_target.status, CMD_PARSE_SUCCESS);
        let first = cmd_list_first(with_target.cmdlist.as_ref().unwrap());
        assert_eq!(seen(args_get(cmd_get_args(&*first), b't')), "mysess");
        let _ = with_target.cmdlist.take();

        let mut with_all = cmd_parse_from_string(
            c"list-clients -F '#{line}' -f '1' -O size -r -t mysess".as_ptr(),
            null_mut(),
        );
        assert_eq!(with_all.status, CMD_PARSE_SUCCESS);
        let _ = with_all.cmdlist.take();

        let mut extra_arg =
            cmd_parse_from_string(c"list-clients extra_argument".as_ptr(), null_mut());
        assert_eq!(extra_arg.status, CMD_PARSE_ERROR);
        let err = extra_arg.take_error();
        assert!(err.contains("list-clients"), "{err}");
        assert!(err.contains("too many arguments"), "{err}");
    }
}

#[test]
fn template_is_the_upstream_format_exactly() {
    let expected: &[u8] =
        b"#{client_name}: #{session_name} [#{client_width}x#{client_height} #{client_termname}] #{?#{!=:#{client_uid},#{uid}},[user #{?client_user,#{client_user},#{client_uid},}] ,}#{?client_flags,(,}#{client_flags}#{?client_flags,),}\0";
    let got: Vec<u8> = LIST_CLIENTS_TEMPLATE.iter().map(|&b| b as u8).collect();
    assert_eq!(LIST_CLIENTS_TEMPLATE.len(), 225);
    assert_eq!(got.len(), expected.len());
    assert_eq!(got, expected);
    assert_eq!(got[got.len() - 1], 0, "the template ends in a NUL");
    assert!(
        expected[..expected.len() - 1].iter().all(|&b| b != 0),
        "the template has no interior NUL"
    );
}

#[test]
fn template_expansion_with_format_engine() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut session = Session::new(1, "main");
    let c = clients.add("/dev/pts/1", 120, 40);
    unsafe {
        wire_client(
            c,
            session.ptr(),
            c"xterm-256color",
            CLIENT_ATTACHED as uint64_t,
        );
        let ft = Format::defaults(c, session.ptr(), null_mut(), null_mut());
        let expanded = ft.expand(CStr::from_ptr(LIST_CLIENTS_TEMPLATE.as_ptr()));
        assert_eq!(
            expanded,
            "/dev/pts/1: main [120x40 xterm-256color] (attached)"
        );

        (*c).peer = Some(bad_peer());
        (*(*c).peer_ptr()).uid = getuid().wrapping_add(1);
        (*c).user = Some(c"alice".to_owned());
        let ft2 = Format::defaults(c, session.ptr(), null_mut(), null_mut());
        let expanded2 = ft2.expand(CStr::from_ptr(LIST_CLIENTS_TEMPLATE.as_ptr()));
        assert_eq!(
            expanded2,
            "/dev/pts/1: main [120x40 xterm-256color] [user alice] (attached)"
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
    for (i, v) in pane_lines.iter().enumerate() {
        for w in &pane_lines[i + 1..] {
            assert_ne!(v, w);
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

    assert_eq!(CMD_READONLY, 0x2);
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
        assert_eq!(found, 1, "entry appears exactly once");

        let mut cause = None;
        assert_eq!(cmd_find(c"list-clients".as_ptr(), &mut cause), entry());
        assert!(cause.is_none());

        assert_eq!(cmd_find(c"lsc".as_ptr(), &mut cause), entry());
        assert!(cause.is_none());
    }
}

#[test]
fn exec_with_no_clients_returns_normal() {
    let _guard = globals();
    let _clients = Clients::new();
    unsafe {
        let mut item = Item::new().with_args(c"list-clients");
        let exec = (*entry()).exec;
        let rv = exec(&*item.cmd(), item.ptr());
        assert_eq!(rv, CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_lists_attached_clients_with_default_format() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut session = Session::new(1, "main-sess");

    let c1 = clients.add("/dev/pts/1", 80, 24);
    let c2 = clients.add("/dev/pts/2", 120, 40);
    unsafe {
        wire_client(
            c1,
            session.ptr(),
            c"xterm-256color",
            CLIENT_ATTACHED as uint64_t,
        );
        wire_client(c2, session.ptr(), c"screen", CLIENT_ATTACHED as uint64_t);

        let mut item = Item::new().with_args(c"list-clients");
        let exec = (*entry()).exec;
        let rv = exec(&*item.cmd(), item.ptr());
        assert_eq!(rv, CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_skips_unattached_and_dead_clients() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut session = Session::new(2, "active-sess");

    let attached = clients.add("attached_client", 80, 24);
    let unattached = clients.add("unattached_client", 80, 24);
    let dead = clients.add("dead_client", 80, 24);
    let suspended = clients.add("suspended_client", 80, 24);
    let exited = clients.add("exited_client", 80, 24);
    let readonly = clients.add("readonly_client", 80, 24);
    let null_sess = clients.add("null_sess_client", 80, 24);

    unsafe {
        wire_client(
            attached,
            session.ptr(),
            c"xterm",
            CLIENT_ATTACHED as uint64_t,
        );
        wire_client(unattached, null_mut(), c"xterm", 0);
        wire_client(
            dead,
            session.ptr(),
            c"xterm",
            (CLIENT_ATTACHED | CLIENT_DEAD) as uint64_t,
        );
        wire_client(
            suspended,
            session.ptr(),
            c"xterm",
            (CLIENT_ATTACHED | CLIENT_SUSPENDED) as uint64_t,
        );
        wire_client(
            exited,
            session.ptr(),
            c"xterm",
            (CLIENT_ATTACHED | CLIENT_EXIT) as uint64_t,
        );
        wire_client(
            readonly,
            session.ptr(),
            c"xterm",
            (CLIENT_ATTACHED | CLIENT_READONLY) as uint64_t,
        );
        wire_client(null_sess, null_mut(), c"xterm", CLIENT_ATTACHED as uint64_t);

        let mut item = Item::new().with_args(c"list-clients");
        let exec = (*entry()).exec;
        let rv = exec(&*item.cmd(), item.ptr());
        assert_eq!(rv, CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_with_target_session_filters_clients() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut s1 = Session::new(10, "sess-one");
    let mut s2 = Session::new(20, "sess-two");
    let mut s_empty = Session::new(30, "sess-empty");

    let c1 = clients.add("client_s1", 80, 24);
    let c2 = clients.add("client_s2", 80, 24);

    unsafe {
        wire_client(c1, s1.ptr(), c"xterm", CLIENT_ATTACHED as uint64_t);
        wire_client(c2, s2.ptr(), c"xterm", CLIENT_ATTACHED as uint64_t);

        let mut item = Item::new().with_args(c"list-clients -t sess-one");
        (*item.ptr()).target.set_session(s1.ptr());
        let exec = (*entry()).exec;
        let rv = exec(&*item.cmd(), item.ptr());
        assert_eq!(rv, CMD_RETURN_NORMAL);

        let mut item_empty = Item::new().with_args(c"list-clients -t sess-empty");
        (*item_empty.ptr()).target.set_session(s_empty.ptr());
        let rv_empty = exec(&*item_empty.cmd(), item_empty.ptr());
        assert_eq!(rv_empty, CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_with_custom_format_flag() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut session = Session::new(3, "custom-sess");

    let c1 = clients.add("client_custom", 90, 30);
    unsafe {
        wire_client(
            c1,
            session.ptr(),
            c"xterm-256color",
            CLIENT_ATTACHED as uint64_t,
        );

        let mut item = Item::new().with_args(
            c"list-clients -F '#{line}: #{client_name} -> #{session_name} (#{client_width}x#{client_height})'",
        );
        let exec = (*entry()).exec;
        let rv = exec(&*item.cmd(), item.ptr());
        assert_eq!(rv, CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_with_filter_flag() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut session = Session::new(4, "filter-sess");

    let c1 = clients.add("client_alpha", 80, 24);
    let c2 = clients.add("client_beta", 80, 24);
    unsafe {
        wire_client(c1, session.ptr(), c"xterm", CLIENT_ATTACHED as uint64_t);
        wire_client(c2, session.ptr(), c"xterm", CLIENT_ATTACHED as uint64_t);

        let mut item_matching =
            Item::new().with_args(c"list-clients -f '#{==:#{client_name},client_alpha}'");
        let exec = (*entry()).exec;
        let rv1 = exec(&*item_matching.cmd(), item_matching.ptr());
        assert_eq!(rv1, CMD_RETURN_NORMAL);

        let mut item_none = Item::new().with_args(c"list-clients -f '0'");
        let rv2 = exec(&*item_none.cmd(), item_none.ptr());
        assert_eq!(rv2, CMD_RETURN_NORMAL);

        let mut item_all = Item::new().with_args(c"list-clients -f '1'");
        let rv3 = exec(&*item_all.cmd(), item_all.ptr());
        assert_eq!(rv3, CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_with_valid_and_invalid_sort_orders() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut session = Session::new(5, "sort-sess");

    let c1 = clients.add("client_aaa", 80, 24);
    let c2 = clients.add("client_bbb", 120, 40);
    unsafe {
        wire_client(c1, session.ptr(), c"xterm", CLIENT_ATTACHED as uint64_t);
        wire_client(c2, session.ptr(), c"xterm", CLIENT_ATTACHED as uint64_t);

        let orders = [
            c"list-clients -O name",
            c"list-clients -O size",
            c"list-clients -O creation",
            c"list-clients -O activity",
            c"list-clients -O index",
            c"list-clients -O title",
            c"list-clients -O z",
        ];

        let exec = (*entry()).exec;
        for cmd in orders {
            let mut item = Item::new().with_args(cmd);
            let rv = exec(&*item.cmd(), item.ptr());
            assert_eq!(rv, CMD_RETURN_NORMAL);
        }

        let mut invalid_item = Item::new().with_args(c"list-clients -O nosuchorder");
        let rv_err = exec(&*invalid_item.cmd(), invalid_item.ptr());
        assert_eq!(rv_err, CMD_RETURN_ERROR);
    }
}

#[test]
fn exec_with_reversed_order() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut session = Session::new(6, "rev-sess");

    let c1 = clients.add("client_1", 80, 24);
    let c2 = clients.add("client_2", 100, 30);
    unsafe {
        wire_client(c1, session.ptr(), c"xterm", CLIENT_ATTACHED as uint64_t);
        wire_client(c2, session.ptr(), c"xterm", CLIENT_ATTACHED as uint64_t);

        let mut item = Item::new().with_args(c"list-clients -r");
        let exec = (*entry()).exec;
        let rv = exec(&*item.cmd(), item.ptr());
        assert_eq!(rv, CMD_RETURN_NORMAL);

        let mut item_rev_name = Item::new().with_args(c"list-clients -O name -r");
        let rv2 = exec(&*item_rev_name.cmd(), item_rev_name.ptr());
        assert_eq!(rv2, CMD_RETURN_NORMAL);
    }
}

#[test]
fn exec_with_all_flags_combined() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut s1 = Session::new(7, "combo-sess");

    let c1 = clients.add("client_c1", 80, 24);
    let c2 = clients.add("client_c2", 120, 40);
    unsafe {
        wire_client(c1, s1.ptr(), c"xterm", CLIENT_ATTACHED as uint64_t);
        wire_client(c2, s1.ptr(), c"xterm", CLIENT_ATTACHED as uint64_t);

        let mut item = Item::new().with_args(
            c"list-clients -F '#{line} #{client_name}' -f '#{!=:#{client_name},none}' -O name -r -t combo-sess",
        );
        (*item.ptr()).target.set_session(s1.ptr());
        let exec = (*entry()).exec;
        let rv = exec(&*item.cmd(), item.ptr());
        assert_eq!(rv, CMD_RETURN_NORMAL);
    }
}

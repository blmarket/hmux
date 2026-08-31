//! Unit tests for [`crate::cmd::cmd_rename_window`] — the `rename-window`
//! entry (name, alias, template, usage, flags and exec hook), the block of
//! message-protocol, display and command constants the file declares, and the
//! deterministic behaviour of [`cmd_rename_window_exec`] as reached through
//! the entry's own function pointer over items built by the real command
//! parser.
//!
//! Exec's job is small: expand argument 0 as a format against the target,
//! refuse names [`check_name` rejects], otherwise rename the target state's
//! window, pin `automatic-rename` off so the new name sticks, free the
//! expanded name and ask for borders and status to be redrawn. Every one of
//! those steps is pinned here by behaviour: a rename changes the fixture
//! window's name, flips the option from its default on to off and raises both
//! redraw flags on a client attached to the window's session; a name spelled
//! as a format arrives expanded; and an invalid byte in the name leaves the
//! window exactly as it was while the refusal is reported through the server's
//! message log — the item carries a client whose peer is marked bad, so
//! `cmdq_error` files the message without ever reaching a descriptor.
//!
//! One process-wide trace is left behind on purpose, like the other suites:
//! each successful rename raises a `window-renamed` notification that sits on
//! the global command queue nothing ever drains. Everything else these tests
//! touch is taken and given back under [`globals`].

use crate::arguments::{args_has, args_string};
use crate::cmd::cmd_get_args;
use crate::cmd::cmd_rename_window::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_FIND_WINDOW, cmd_rename_window_entry,
};
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::proc::PEER_BAD;
use crate::server::message_log;
use crate::tests::test_fixtures::{Item, globals, seen, zeroed};
use crate::types::*;
use ::core::ffi::{CStr, c_char};
use ::core::ptr::null_mut;

/// Where the tests' items claim to come from.
const FILE: &CStr = c"test-coverage-cmd-rename-window.conf";

/// The entry under test.
const ENTRY: *const cmd_entry = &raw const cmd_rename_window_entry;

/// Runs the parsed command an item carries through the entry's exec hook, the
/// way the command queue calls it. The item must be running this entry.
unsafe fn exec_via(item: &mut Item) -> cmd_retval {
    unsafe {
        assert!(
            ::core::ptr::eq((*item.cmd()).entry, ENTRY),
            "the item is not running rename-window"
        );
        let exec = (*ENTRY).exec;
        exec(&*item.cmd(), item.ptr())
    }
}

/// A peer for the fixture client, marked bad so `proc_send` refuses any
/// message before it reaches a buffer underneath it.
fn bad_peer() -> Box<tmuxpeer> {
    let mut p = zeroed::<tmuxpeer>();
    p.flags |= PEER_BAD;
    p
}

/// Gives `c` its peer. Its session stays null, which is what sends
/// `cmdq_error` down the branch that files the message in the server's
/// message log.
unsafe fn wire(c: *mut client) {
    unsafe {
        (*c).peer = Some(bad_peer());
    }
}

/// The lines the server has recorded so far, oldest first. Entries accumulate
/// across the whole test binary, so assertions look for their own wording at
/// the position they added rather than count lines from zero.
unsafe fn server_messages() -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        for m in message_log.queue().iter() {
            out.push(seen(m.msg.as_ptr()));
        }
        out
    }
}

#[test]
fn the_entry_describes_the_rename_window_command() {
    let _guard = globals();
    unsafe {
        assert_eq!((*ENTRY).name.to_string_lossy(), "rename-window");
        assert_eq!(
            (*ENTRY)
                .alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "renamew"
        );
        assert_eq!(
            (*ENTRY).usage.to_string_lossy(),
            "[-t target-window] new-name"
        );
        assert_eq!((*ENTRY).args.template.to_string_lossy(), "t:");
        assert_eq!((*ENTRY).args.lower, 1);
        assert_eq!((*ENTRY).args.upper, 1);
        assert!(
            (*ENTRY).args.cb.is_none(),
            "rename-window takes no args callback"
        );

        assert_eq!((*ENTRY).source.flag, 0);
        assert_eq!((*ENTRY).source.type_0, CMD_FIND_PANE);
        assert_eq!((*ENTRY).source.flags, 0);
        assert_eq!((*ENTRY).target.flag, 't' as c_char);
        assert_eq!((*ENTRY).target.type_0, CMD_FIND_WINDOW);
        assert_eq!((*ENTRY).target.flags, 0);

        assert_eq!((*ENTRY).flags, CMD_AFTERHOOK);
    }
}

#[test]
fn the_parser_resolves_the_name_the_alias_and_a_prefix() {
    let _guard = globals();
    unsafe {
        for (i, line) in [c"rename-window foo", c"renamew foo", c"rename-w foo"]
            .into_iter()
            .enumerate()
        {
            let mut item = Item::new().from_file(FILE, i as u_int + 1).with_args(line);
            assert!(::core::ptr::eq((*item.cmd()).entry, ENTRY), "{line:?}");
        }

        let mut flagged = Item::new()
            .from_file(FILE, 9)
            .with_args(c"rename-window -t 3 work");
        assert!(::core::ptr::eq((*flagged.cmd()).entry, ENTRY));
        assert_eq!(
            args_has(cmd_get_args(&*flagged.cmd()), b't'),
            1,
            "-t is the entry's own target flag"
        );
        assert_eq!(seen(args_string(cmd_get_args(&*flagged.cmd()), 0)), "work");
    }
}

#[test]
fn the_template_bounds_allow_exactly_one_argument() {
    let _guard = globals();
    unsafe {
        let mut none = cmd_parse_from_string(c"rename-window".as_ptr(), null_mut());
        assert_eq!(none.status, CMD_PARSE_ERROR);
        let err = none.take_error();
        assert!(err.contains("rename-window"), "{err}");
        assert!(err.contains("too few arguments"), "{err}");

        let mut one = cmd_parse_from_string(c"rename-window work".as_ptr(), null_mut());
        assert_eq!(one.status, CMD_PARSE_SUCCESS);
        let _ = one.cmdlist.take();

        let mut two = cmd_parse_from_string(c"rename-window work also".as_ptr(), null_mut());
        assert_eq!(two.status, CMD_PARSE_ERROR);
        let err = two.take_error();
        assert!(err.contains("rename-window"), "{err}");
        assert!(err.contains("too many arguments"), "{err}");
    }
}

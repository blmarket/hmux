//! Unit tests for [`crate::cmd::cmd_save_buffer`] — the `save-buffer` and
//! `show-buffer` entries, which share one exec function, the protocol,
//! enumeration and flag constants this generated module re-declares, the
//! argument bounds and flags the parser enforces for both names, registration
//! of both entries in the command table, and the deterministic exec branches
//! that need no descriptor: the two buffer-lookup refusals, which report
//! through `cmdq_error` onto an attached client.
//!
//! A note on how far these tests go. Everything past the buffer lookup is
//! deliberately left uncovered. The save tail — `format_single_from_target`
//! followed by `file_write` — ends in a real `fopen` or an asynchronous write,
//! and the completion callback `cmd_save_buffer_done` is only ever driven by
//! the file code. The show tail's print branch needs a client carrying a live
//! session or a control state, either of which drags the server's redraw or
//! control machinery in, so no fixture reaches it; the refusals ahead of both
//! tails are covered instead, with the shared exec reached through each
//! entry's own function pointer. Nothing here arms ensure_reactor, opens a file or
//! leaves state behind on the process.

use crate::arguments::{args_count, args_get, args_has, args_string};
use crate::cmd::cmd_attach_session::CLIENT_ATTACHED;
use crate::cmd::cmd_save_buffer::*;
use crate::cmd::{CMD_PARSE_ERROR, cmd_parse_from_string};
use crate::cmd::{cmd_find, cmd_table};
use crate::paste::paste_get_name;
use crate::server::message_log;
use crate::tests::test_fixtures::{Args, Clients, Item, Paste, globals, seen};
use ::core::ffi::CStr;
use ::core::ptr::null_mut;

/// Where the tests' items claim to come from, which is what `cmdq_error`
/// would report them under if any item here were client-less.
const FILE: &CStr = c"test-coverage-cmd-save-buffer.conf";

/// Runs one entry's exec function through its own function pointer, the way
/// the command queue calls it, and answers what it answers.
unsafe fn run(entry: *const cmd_entry, item: &mut Item) -> cmd_retval {
    unsafe {
        let run = (*entry).exec;
        run(&*item.cmd(), item.ptr())
    }
}

/// A fixture client marked attached, so `file_error` downstream of
/// `cmdq_error` declines to open anything while the message machinery itself
/// still runs.
fn attached_client(clients: &mut Clients, name: &str) -> *mut client {
    let c = clients.add(name, 80, 24);
    unsafe { (*c).flags |= CLIENT_ATTACHED as uint64_t };
    c
}

/// The lines the server has recorded so far, oldest first. Entries accumulate
/// across the whole test binary, so assertions look for their own wording
/// rather than count lines from zero.
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
fn the_entries_describe_two_commands_sharing_one_exec() {
    unsafe {
        let save_e = &raw const cmd_save_buffer_entry;
        let show_e = &raw const cmd_show_buffer_entry;
        assert_ne!(save_e, show_e);

        assert_eq!((*save_e).name.to_bytes(), b"save-buffer");
        assert_eq!(
            (*save_e).alias.expect("the entry has an alias").to_bytes(),
            b"saveb"
        );
        assert_eq!((*save_e).usage.to_bytes(), b"[-a] [-b buffer-name] path");
        assert_eq!((*save_e).args.template.to_bytes(), b"ab:");
        assert_eq!((*save_e).args.lower, 1);
        assert_eq!((*save_e).args.upper, 1);
        assert!((*save_e).args.cb.is_none());

        assert_eq!((*show_e).name.to_bytes(), b"show-buffer");
        assert_eq!(
            (*show_e).alias.expect("the entry has an alias").to_bytes(),
            b"showb"
        );
        assert_eq!((*show_e).usage, CMD_BUFFER_USAGE);
        assert_eq!((*show_e).usage.to_bytes(), b"[-b buffer-name]");
        assert_eq!((*show_e).args.template.to_bytes(), b"b:");
        assert_eq!((*show_e).args.lower, 0);
        assert_eq!((*show_e).args.upper, 0);
        assert!((*show_e).args.cb.is_none());

        for e in [save_e, show_e] {
            for flag in [&raw const (*e).source, &raw const (*e).target] {
                assert_eq!((*flag).flag, 0);
                assert_eq!((*flag).type_0, CMD_FIND_PANE);
                assert_eq!((*flag).flags, 0);
            }
            assert_eq!((*e).flags, CMD_AFTERHOOK);
            assert_eq!((*e).flags & CMD_AFTERHOOK, CMD_AFTERHOOK);
            assert_eq!((*e).flags & !CMD_AFTERHOOK, 0);
        }

        assert!(::core::ptr::fn_addr_eq((*save_e).exec, (*show_e).exec));
    }
}

#[test]
fn both_entries_are_registered_once_and_findable_by_name_alias_and_prefix() {
    let _guard = globals();
    unsafe {
        let save_e = &raw const cmd_save_buffer_entry;
        let show_e = &raw const cmd_show_buffer_entry;

        let count = |want| {
            cmd_table
                .iter()
                .filter(|slot| ::core::ptr::eq(**slot, want))
                .count()
        };
        let found_save = count(save_e);
        let found_show = count(show_e);
        assert_eq!(found_save, 1, "save-buffer appears exactly once");
        assert_eq!(found_show, 1, "show-buffer appears exactly once");

        let mut cause = None;
        for name in [
            c"save-buffer",
            c"saveb",
            c"save-b",
            c"show-buffer",
            c"showb",
            c"show-b",
        ] {
            let want = if name.to_bytes().starts_with(b"save") {
                save_e
            } else {
                show_e
            };
            assert_eq!(cmd_find(name.as_ptr(), &mut cause), want, "{name:?}");
            assert!(cause.is_none(), "no cause on success for {name:?}");
        }
    }
}

#[test]
fn parsing_resolves_both_names_and_carries_the_a_and_b_flags_and_the_path() {
    let _guard = globals();
    unsafe {
        let plain = Args::parse(c"save-buffer /tmp/paste");
        assert!(::core::ptr::eq(
            (*plain.cmd()).entry,
            &cmd_save_buffer_entry
        ));
        let args = plain.ptr();
        assert_eq!(args_has(&*args, b'a'), 0);
        assert_eq!(args_has(&*args, b'b'), 0);
        assert_eq!(args_count(&*args), 1);
        assert_eq!(seen(args_string(&*args, 0)), "/tmp/paste");

        let full = Args::parse(c"saveb -a -b buf /tmp/other");
        assert!(::core::ptr::eq((*full.cmd()).entry, &cmd_save_buffer_entry));
        let args = full.ptr();
        assert_eq!(args_has(&*args, b'a'), 1);
        assert_eq!(seen(args_get(&*args, b'b')), "buf");
        assert_eq!(args_count(&*args), 1);
        assert_eq!(seen(args_string(&*args, 0)), "/tmp/other");

        let bare = Args::parse(c"show-buffer");
        assert!(::core::ptr::eq((*bare.cmd()).entry, &cmd_show_buffer_entry));
        assert_eq!(args_count(&*bare.ptr()), 0);
        assert_eq!(args_has(&*bare.ptr(), b'b'), 0);

        let named = Args::parse(c"show-buffer -b buf");
        assert!(::core::ptr::eq(
            (*named.cmd()).entry,
            &cmd_show_buffer_entry
        ));
        assert_eq!(args_count(&*named.ptr()), 0);
        assert_eq!(args_has(&*named.ptr(), b'b'), 1);
        assert_eq!(seen(args_get(&*named.ptr(), b'b')), "buf");

        let alias = Args::parse(c"showb");
        assert!(::core::ptr::eq(
            (*alias.cmd()).entry,
            &cmd_show_buffer_entry
        ));
        assert_eq!(args_count(&*alias.ptr()), 0);
    }
}

#[test]
fn parsing_enforces_the_argument_bounds_and_rejects_unknown_flags() {
    let _guard = globals();
    unsafe {
        let mut none = cmd_parse_from_string(c"save-buffer".as_ptr(), null_mut());
        assert_eq!(none.status, CMD_PARSE_ERROR);
        let err = none.take_error();
        assert!(err.contains("too few arguments"), "{err}");
        assert!(err.contains("need at least 1"), "{err}");

        let mut alias_none = cmd_parse_from_string(c"saveb".as_ptr(), null_mut());
        assert_eq!(alias_none.status, CMD_PARSE_ERROR);
        let err = alias_none.take_error();
        assert!(err.contains("too few arguments"), "{err}");

        let mut extra =
            cmd_parse_from_string(c"save-buffer /tmp/one /tmp/two".as_ptr(), null_mut());
        assert_eq!(extra.status, CMD_PARSE_ERROR);
        let err = extra.take_error();
        assert!(err.contains("too many arguments"), "{err}");
        assert!(err.contains("need at most 1"), "{err}");

        let mut show_extra = cmd_parse_from_string(c"show-buffer one".as_ptr(), null_mut());
        assert_eq!(show_extra.status, CMD_PARSE_ERROR);
        let err = show_extra.take_error();
        assert!(err.contains("too many arguments"), "{err}");

        let mut bad_flag = cmd_parse_from_string(c"save-buffer -z /tmp/x".as_ptr(), null_mut());
        assert_eq!(bad_flag.status, CMD_PARSE_ERROR);
        let err = bad_flag.take_error();
        assert!(err.contains("unknown flag"), "{err}");

        let mut alias_bad_flag = cmd_parse_from_string(c"showb -z".as_ptr(), null_mut());
        assert_eq!(alias_bad_flag.status, CMD_PARSE_ERROR);
        let err = alias_bad_flag.take_error();
        assert!(err.contains("unknown flag"), "{err}");

        let ok = Args::parse(c"save-buffer /tmp/x");
        assert!(::core::ptr::eq((*ok.cmd()).entry, &cmd_save_buffer_entry));
        let ok_alias = Args::parse(c"showb");
        assert!(::core::ptr::eq(
            (*ok_alias.cmd()).entry,
            &cmd_show_buffer_entry
        ));
    }
}

#[test]
fn an_empty_store_refuses_the_unnamed_save_with_no_buffers() {
    let _guard = globals();
    let _store = Paste::new();
    let mut clients = Clients::new();
    unsafe {
        let c = attached_client(&mut clients, "sb-empty");
        let mut item = Item::new()
            .from_file(FILE, 1)
            .with_args(c"save-buffer /tmp/sb");
        item.set_client(c);

        assert_eq!(
            run(&raw const cmd_save_buffer_entry, &mut item),
            CMD_RETURN_ERROR
        );
        assert_eq!((*c).retval, 1, "cmdq_error marks the client's return value");
        let msgs = server_messages();
        assert!(
            msgs.iter()
                .any(|m| m.contains("sb-empty") && m.contains("no buffers")),
            "{msgs:?}"
        );
    }
}

#[test]
fn a_missing_named_buffer_is_refused_for_both_commands() {
    let _guard = globals();
    let store = Paste::new();
    let mut clients = Clients::new();
    unsafe {
        store.add(c"buf", "something");
        assert!(paste_get_name(c"nosuch".as_ptr()).is_null());

        let c = attached_client(&mut clients, "sb-missing");
        for (n, line) in [c"save-buffer -b nosuch /tmp/sb", c"show-buffer -b nosuch"]
            .into_iter()
            .enumerate()
        {
            let mut item = Item::new()
                .from_file(FILE, (n + 2) as u_int)
                .with_args(line);
            item.set_client(c);

            let e = if n == 0 {
                &raw const cmd_save_buffer_entry
            } else {
                &raw const cmd_show_buffer_entry
            };
            assert_eq!(run(e, &mut item), CMD_RETURN_ERROR, "{line:?}");
            assert_eq!((*c).retval, 1, "{line:?}");
            let msgs = server_messages();
            assert!(
                msgs.iter().any(|m| m.contains("no buffer nosuch")),
                "{line:?}: {msgs:?}"
            );
            (*c).retval = 0;
        }
    }
}

#[test]
fn show_buffer_with_session_client_succeeds() {
    let _guard = globals();
    let store = Paste::new();
    let mut clients = Clients::new();
    let mut target = crate::tests::test_fixtures::Target::new(80, 24);
    unsafe {
        store.add(c"mybuf", "hello world");
        let mut peer = crate::tests::test_fixtures::zeroed::<tmuxpeer>();
        peer.flags |= crate::proc::PEER_BAD;

        let c = clients.add("show-client", 80, 24);
        (*c).session = target.session();
        (*c).peer = Some(peer);

        let mut item = Item::new().with_args(c"show-buffer -b mybuf");
        item.set_client(c);

        assert_eq!(
            run(&raw const cmd_show_buffer_entry, &mut item),
            CMD_RETURN_NORMAL
        );
    }
}

/// The write is still outstanding when the test ends, and the done callback
/// the client's release fires reaches back into the item, so the item is
/// declared first and outlives the clients.
#[test]
fn save_buffer_initiates_file_write() {
    let _guard = globals();
    let store = Paste::new();
    let mut item = Item::new().with_args(c"save-buffer -a -b mybuf2 /tmp/out.txt");
    let mut clients = Clients::new();
    unsafe {
        store.add(c"mybuf2", "data to save");

        let mut peer = crate::tests::test_fixtures::zeroed::<tmuxpeer>();
        peer.flags |= crate::proc::PEER_BAD;

        let c = clients.add("save-client", 80, 24);
        (*c).peer = Some(peer);

        item.set_client(c);

        assert_eq!(
            run(&raw const cmd_save_buffer_entry, &mut item),
            CMD_RETURN_WAIT
        );
    }
}

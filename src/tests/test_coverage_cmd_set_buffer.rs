//! Unit tests for [`crate::cmd::cmd_set_buffer`] — the `set-buffer` and
//! `delete-buffer` entries, which share one exec routine, the protocol,
//! enumeration and flag constants the generated module re-declares, the
//! argument bounds the parser enforces for both names, registration of both
//! entries in the command table, and every deterministic exec branch that
//! needs neither a live server nor a terminal: buffer lookup by name and by
//! newest automatic, deletion, renaming, storing under a name and appending
//! to what is already there, the empty-data short cut, and the refusals,
//! which report through `cmdq_error` onto an attached client and answer
//! [`CMD_RETURN_ERROR`].
//!
//! A note on how far these tests go. The `-w` selection offer runs against a
//! fixture client whose terminal never started, so `tty_set_selection`
//! declines at its first check and nothing reaches a descriptor; ensure_reactor is
//! never armed, no file is opened and the paste store is emptied again after
//! each test, so nothing is left behind on the process.

use crate::arguments::{args_count, args_get, args_has, args_string};
use crate::cmd::cmd_attach_session::CLIENT_ATTACHED;
use crate::cmd::cmd_set_buffer::*;
use crate::cmd::cmdq_set_target_client;
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{cmd_find, cmd_table};
use crate::paste::{paste_buffer_data, paste_get_name, paste_get_top, paste_set};
use crate::server::message_log;
use crate::tests::test_fixtures::{Args, Clients, Item, Paste, globals, seen};
use ::core::ffi::CStr;
use ::core::ptr::{null, null_mut};

/// Where the tests' items claim to come from, which is what `cmdq_error`
/// would report them under if any item here were client-less.
const FILE: &CStr = c"test-coverage-cmd-set-buffer.conf";

/// A named buffer the tests put in the store.
const BUF: &CStr = c"buf";

/// Another named buffer, for appending where none was.
const FRESH: &CStr = c"fresh";

/// The `set-buffer` entry as a raw pointer.
fn set_entry() -> *const cmd_entry {
    &raw const cmd_set_buffer_entry
}

/// The `delete-buffer` entry as a raw pointer.
fn delete_entry() -> *const cmd_entry {
    &raw const cmd_delete_buffer_entry
}

/// Runs one entry's exec function through its own function pointer, the way
/// the command queue calls it, and answers what it answers.
unsafe fn run(entry: *const cmd_entry, item: &mut Item) -> cmd_retval {
    unsafe {
        let run = (*entry).exec;
        run(&*item.cmd(), item.ptr())
    }
}

/// An item carrying a parsed command line, sourced from [`FILE`], with no
/// client behind it — enough for every branch that never reports anything.
fn item(line: &'static CStr, number: u_int) -> Item {
    Item::new().from_file(FILE, number).with_args(line)
}

/// Runs `line` through `entry`'s exec with an attached fixture client behind
/// the item, both as its client and as its target client.
unsafe fn run_as(
    entry: *const cmd_entry,
    line: &'static CStr,
    number: u_int,
    c: *mut client,
) -> cmd_retval {
    unsafe {
        let mut it = item(line, number);
        it.set_client(c);
        cmdq_set_target_client(it.ptr(), c);
        run(entry, &mut it)
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

/// Adds an automatic buffer holding `data`, as an unnamed `set-buffer` would.
/// Automatic ones are exactly what an unnamed `delete-buffer`, an unnamed
/// rename and the append branch find.
unsafe fn add_automatic(data: &str) {
    unsafe {
        assert!(
            paste_set(data.as_bytes().to_vec(), null()).is_ok(),
            "{data:?} was not set"
        );
    }
}

/// The bytes of the newest automatic buffer, which `paste_get_top` answers.
unsafe fn top_bytes() -> Vec<u8> {
    unsafe {
        let pb = paste_get_top(None);
        assert!(!pb.is_null(), "no automatic buffer on top");
        paste_buffer_data(&*pb).to_vec()
    }
}

/// Whether a buffer called `name` exists in the store.
fn exists(name: &CStr) -> bool {
    !unsafe { paste_get_name(name.as_ptr()) }.is_null()
}

/// The bytes stored under `name`.
unsafe fn bytes_of(name: &CStr) -> Vec<u8> {
    unsafe {
        let pb = paste_get_name(name.as_ptr());
        assert!(!pb.is_null(), "no buffer {name:?}");
        paste_buffer_data(&*pb).to_vec()
    }
}

#[test]
fn the_entries_describe_two_commands_sharing_one_exec() {
    unsafe {
        let se = set_entry();
        let de = delete_entry();
        assert_ne!(se, de);

        assert_eq!((*se).name.to_bytes(), b"set-buffer");
        assert_eq!(
            (*se).alias.expect("the entry has an alias").to_bytes(),
            b"setb"
        );
        assert_eq!(
            (*se).usage.to_bytes(),
            b"[-aw] [-b buffer-name] [-n new-buffer-name] [-t target-client] [data]"
        );
        assert_eq!((*se).args.template.to_bytes(), b"ab:t:n:w");
        assert_eq!((*se).args.lower, 0);
        assert_eq!((*se).args.upper, 1);
        assert!((*se).args.cb.is_none());

        assert_eq!((*de).name.to_bytes(), b"delete-buffer");
        assert_eq!(
            (*de).alias.expect("the entry has an alias").to_bytes(),
            b"deleteb"
        );
        assert_eq!((*de).usage, CMD_BUFFER_USAGE);
        assert_eq!((*de).usage.to_bytes(), b"[-b buffer-name]");
        assert_eq!((*de).args.template.to_bytes(), b"b:");
        assert_eq!((*de).args.lower, 0);
        assert_eq!((*de).args.upper, 0);
        assert!((*de).args.cb.is_none());

        for e in [se, de] {
            for flag in [&raw const (*e).source, &raw const (*e).target] {
                assert_eq!((*flag).flag, 0);
                assert_eq!((*flag).type_0, CMD_FIND_PANE);
                assert_eq!((*flag).flags, 0);
            }
        }

        assert_eq!(
            (*se).flags,
            CMD_AFTERHOOK | CMD_CLIENT_TFLAG | CMD_CLIENT_CANFAIL
        );
        assert_eq!((*de).flags, CMD_AFTERHOOK);

        assert!(::core::ptr::fn_addr_eq((*se).exec, (*de).exec));
    }
}

#[test]
fn both_entries_are_registered_once_and_findable_by_name_alias_and_prefix() {
    let _guard = globals();
    unsafe {
        let count = |want| {
            cmd_table
                .iter()
                .filter(|slot| ::core::ptr::eq(**slot, want))
                .count()
        };
        let found_set = count(set_entry());
        let found_del = count(delete_entry());
        assert_eq!(found_set, 1, "set-buffer appears exactly once");
        assert_eq!(found_del, 1, "delete-buffer appears exactly once");

        let mut cause = None;
        for (name, want) in [
            (c"set-buffer", set_entry()),
            (c"setb", set_entry()),
            (c"set-b", set_entry()),
            (c"delete-buffer", delete_entry()),
            (c"deleteb", delete_entry()),
            (c"delete-b", delete_entry()),
        ] {
            assert_eq!(cmd_find(name.as_ptr(), &mut cause), want, "{name:?}");
            assert!(cause.is_none(), "no cause on success for {name:?}");
        }
    }
}

#[test]
fn parsing_resolves_both_names_and_carries_the_flags_and_the_data() {
    let _guard = globals();
    unsafe {
        let plain = Args::parse(c"set-buffer hello");
        assert!(::core::ptr::eq((*plain.cmd()).entry, set_entry()));
        let args = plain.ptr();
        assert_eq!(args_has(&*args, b'a'), 0);
        assert_eq!(args_has(&*args, b'n'), 0);
        assert_eq!(args_has(&*args, b'w'), 0);
        assert!(args_get(&*args, b'b').is_null());
        assert_eq!(args_count(&*args), 1);
        assert_eq!(seen(args_string(&*args, 0)), "hello");

        let full = Args::parse(c"setb -aw -b buf -n new more");
        assert!(::core::ptr::eq((*full.cmd()).entry, set_entry()));
        let args = full.ptr();
        assert_eq!(args_has(&*args, b'a'), 1);
        assert_eq!(args_has(&*args, b'w'), 1);
        assert_eq!(args_has(&*args, b'n'), 1);
        assert_eq!(seen(args_get(&*args, b'b')), "buf");
        assert_eq!(seen(args_get(&*args, b'n')), "new");
        assert_eq!(args_count(&*args), 1);
        assert_eq!(seen(args_string(&*args, 0)), "more");

        let bare = Args::parse(c"set-buffer");
        assert!(::core::ptr::eq((*bare.cmd()).entry, set_entry()));
        assert_eq!(args_count(&*bare.ptr()), 0);

        let del = Args::parse(c"delete-buffer -b doomed");
        assert!(::core::ptr::eq((*del.cmd()).entry, delete_entry()));
        assert_eq!(args_count(&*del.ptr()), 0);
        assert_eq!(seen(args_get(&*del.ptr(), b'b')), "doomed");

        let del_bare = Args::parse(c"deleteb");
        assert!(::core::ptr::eq((*del_bare.cmd()).entry, delete_entry()));
        assert_eq!(args_count(&*del_bare.ptr()), 0);
    }
}

#[test]
fn parsing_enforces_the_argument_bounds_and_rejects_unknown_flags() {
    let _guard = globals();
    unsafe {
        let mut extra = cmd_parse_from_string(c"set-buffer one two".as_ptr(), null_mut());
        assert_eq!(extra.status, CMD_PARSE_ERROR);
        let err = extra.take_error();
        assert!(err.contains("too many arguments"), "{err}");
        assert!(err.contains("at most 1"), "{err}");

        let mut bare_ok = cmd_parse_from_string(c"set-buffer".as_ptr(), null_mut());
        assert_eq!(bare_ok.status, CMD_PARSE_SUCCESS);
        let _ = bare_ok.cmdlist.take();

        let mut del_extra = cmd_parse_from_string(c"delete-buffer x".as_ptr(), null_mut());
        assert_eq!(del_extra.status, CMD_PARSE_ERROR);
        let err = del_extra.take_error();
        assert!(err.contains("too many arguments"), "{err}");

        let mut del_bare = cmd_parse_from_string(c"deleteb".as_ptr(), null_mut());
        assert_eq!(del_bare.status, CMD_PARSE_SUCCESS);
        let _ = del_bare.cmdlist.take();

        for line in [c"set-buffer -z x", c"deleteb -z"] {
            let mut bad = cmd_parse_from_string(line.as_ptr(), null_mut());
            assert_eq!(bad.status, CMD_PARSE_ERROR, "{line:?}");
            let err = bad.take_error();
            assert!(err.contains("unknown flag"), "{line:?}: {err}");
        }

        let good = Args::parse(c"setb -aw -b buf -n new more");
        assert!(::core::ptr::eq((*good.cmd()).entry, set_entry()));
    }
}

#[test]
fn storing_makes_an_automatic_buffer_or_nothing_at_all_for_empty_data() {
    let _guard = globals();
    let _store = Paste::new();
    unsafe {
        let mut unnamed = item(c"set-buffer hello", 6);
        assert_eq!(run(set_entry(), &mut unnamed), CMD_RETURN_NORMAL);
        assert_eq!(
            top_bytes(),
            b"hello",
            "unnamed data becomes an automatic buffer"
        );

        let mut empty = item(c"set-buffer ''", 7);
        assert_eq!(run(set_entry(), &mut empty), CMD_RETURN_NORMAL);
        assert_eq!(top_bytes(), b"hello", "empty data stores nothing");

        let mut named_empty = item(c"set-buffer -b buf ''", 8);
        assert_eq!(run(set_entry(), &mut named_empty), CMD_RETURN_NORMAL);
        assert!(
            !exists(BUF),
            "an empty store creates no named buffer either"
        );
    }
}

#[test]
fn storing_under_a_name_creates_it_again_replaces_and_a_appends() {
    let _guard = globals();
    let _store = Paste::new();
    unsafe {
        let mut first = item(c"set-buffer -b buf first", 9);
        assert_eq!(run(set_entry(), &mut first), CMD_RETURN_NORMAL);
        assert_eq!(bytes_of(BUF), b"first");

        let mut second = item(c"setb -b buf second", 10);
        assert_eq!(run(set_entry(), &mut second), CMD_RETURN_NORMAL);
        assert_eq!(bytes_of(BUF), b"second", "a same-named store replaces");

        let mut append = item(c"set-buffer -a -b buf def", 11);
        assert_eq!(run(set_entry(), &mut append), CMD_RETURN_NORMAL);
        assert_eq!(bytes_of(BUF), b"seconddef");

        let mut append_missing = item(c"set-buffer -a -b fresh xyz", 12);
        assert_eq!(run(set_entry(), &mut append_missing), CMD_RETURN_NORMAL);
        assert_eq!(
            bytes_of(FRESH),
            b"xyz",
            "-a without a buffer starts one anyway"
        );
    }
}

#[test]
fn deleting_takes_a_named_buffer_or_the_newest_automatic_one() {
    let _guard = globals();
    let store = Paste::new();
    unsafe {
        store.add(c"doomed", "xy");
        store.add(c"keeper", "z");

        let mut named = item(c"delete-buffer -b doomed", 14);
        assert_eq!(run(delete_entry(), &mut named), CMD_RETURN_NORMAL);
        assert!(!exists(c"doomed"), "the named buffer is gone");
        assert!(exists(c"keeper"), "and only that one");

        add_automatic("older");
        add_automatic("newer");
        assert_eq!(top_bytes(), b"newer");

        let mut unnamed = item(c"delete-buffer", 15);
        assert_eq!(run(delete_entry(), &mut unnamed), CMD_RETURN_NORMAL);
        assert_eq!(
            top_bytes(),
            b"older",
            "the newest automatic buffer is the one an unnamed delete takes"
        );

        let mut last = item(c"deleteb", 16);
        assert_eq!(run(delete_entry(), &mut last), CMD_RETURN_NORMAL);
        assert!(
            paste_get_top(None).is_null(),
            "no automatic buffers are left"
        );
    }
}

#[test]
fn renaming_moves_the_named_or_newest_automatic_buffer() {
    let _guard = globals();
    let store = Paste::new();
    unsafe {
        store.add(BUF, "abc");

        let mut named = item(c"set-buffer -b buf -n better", 17);
        assert_eq!(run(set_entry(), &mut named), CMD_RETURN_NORMAL);
        assert!(!exists(BUF), "the old name went with the rename");
        assert_eq!(bytes_of(c"better"), b"abc");

        add_automatic("top data");
        assert_eq!(top_bytes(), b"top data");

        let mut top = item(c"setb -n moved", 18);
        assert_eq!(run(set_entry(), &mut top), CMD_RETURN_NORMAL);
        assert_eq!(
            bytes_of(c"moved"),
            b"top data",
            "without -b the rename takes the newest automatic buffer"
        );
        assert!(
            paste_get_top(None).is_null(),
            "a renamed buffer stops being automatic"
        );
    }
}

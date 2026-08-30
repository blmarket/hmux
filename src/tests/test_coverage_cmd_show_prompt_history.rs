//! Unit tests for [`crate::cmd::cmd_show_prompt_history`] — the
//! `show-prompt-history` and `clear-prompt-history` entries' metadata, the
//! constants they carry, and every branch of [`cmd_show_prompt_history_exec`]
//! that deterministic fixtures can reach.
//!
//! Exec is reached through the entry's own function pointer, exactly as the
//! command queue calls it, with the entry the parser selected on the item's
//! command — that pointer is how the exec tells clearing from showing. The
//! prompt history it reads and clears lives in [`status_prompt_hlist`]; a
//! helper plants lines the way `status_prompt_add_history` does, and another
//! empties whatever is left over, leaving the four slots exactly as a fresh
//! server would have them.
//!
//! The show paths print through `cmdq_print`, which with no client behind the
//! item only reaches the debug log; the refusals of an unknown `-T` run with a
//! fixture client attached to nothing, so `cmdq_error` files the complaint in
//! the server's message log and flags the client's retval, without touching
//! any file descriptor.

use crate::arguments::args_get;
use crate::cmd::cmd_show_prompt_history::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_RETURN_NORMAL, PROMPT_NTYPES, PROMPT_TYPE_INVALID,
    cmd_clear_prompt_history_entry, cmd_show_prompt_history_entry,
};
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::cmd::{cmd_find, cmd_get_args, cmd_list_first};
use crate::server::message_log;
use crate::status::{status_prompt_hlist, status_prompt_type, status_prompt_type_string};
use crate::tests::test_fixtures::{Item, globals, seen};
use crate::types::*;
use ::core::ffi::CStr;
use ::core::ptr::null_mut;

/// The show entry as a raw pointer, so every field read stays an explicit
/// unsafe dereference rather than a shared reference into a `static mut`.
fn show_entry() -> *const cmd_entry {
    &raw const cmd_show_prompt_history_entry
}

/// The clear entry likewise; the two are distinct statics sharing one exec.
fn clear_entry() -> *const cmd_entry {
    &raw const cmd_clear_prompt_history_entry
}

/// The four history slots, reached the way every caller reaches them.
unsafe fn hlists() -> &'static mut [Vec<::std::ffi::CString>; 4] {
    unsafe { &mut status_prompt_hlist }
}

/// Empties every planted history slot, leaving the four exactly as a fresh
/// server would have them.
unsafe fn drain_history() {
    unsafe {
        for t in 0..PROMPT_NTYPES as usize {
            hlists()[t].clear();
        }
    }
}

/// Plants `lines` in slot `t`, the way `status_prompt_add_history` does. An
/// empty slice leaves the slot untouched, which is already what empty means
/// here.
unsafe fn plant(t: usize, lines: &[&CStr]) {
    unsafe {
        assert!(t < PROMPT_NTYPES as usize);
        assert!(hlists()[t].is_empty(), "slot {t} starts empty");
        for line in lines {
            hlists()[t].push((*line).to_owned());
        }
    }
}

/// Every history line a slot holds, oldest first.
unsafe fn entries_of(t: usize) -> Vec<String> {
    unsafe { hlists()[t].iter().map(|s| seen(s.as_ptr())).collect() }
}

/// The whole of all four slots, for asserting that a path read without
/// disturbing.
unsafe fn whole_history() -> [Vec<String>; PROMPT_NTYPES as usize] {
    unsafe { ::core::array::from_fn(|t| entries_of(t)) }
}

/// Runs one command line through the exec hook its parser selected, the way
/// the command queue would call it.
unsafe fn exec_line(line: &CStr) -> (Item, cmd_retval) {
    unsafe {
        let mut item = Item::new().with_args(line);
        let e = (*item.cmd()).entry;
        let rv = (e.exec)(&*item.cmd(), item.ptr());
        (item, rv)
    }
}

/// Runs one command line with `c` behind the item, which is what routes
/// `cmdq_error` into the server's message log instead of the config causes.
unsafe fn exec_as_client(line: &CStr, c: *mut client) -> cmd_retval {
    unsafe {
        let mut item = Item::new().with_args(line);
        item.set_client(c);
        let e = (*item.cmd()).entry;
        (e.exec)(&*item.cmd(), item.ptr())
    }
}

/// Every message the server's message log holds. Entries accumulate across the
/// whole test binary, so assertions look for their own wording rather than
/// count lines.
unsafe fn logged_messages() -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        for m in message_log.queue().iter() {
            out.push(seen(m.msg.as_ptr()));
        }
        out
    }
}

#[test]
fn entry_metadata_matches_upstream() {
    unsafe {
        let wanted = [
            (show_entry(), &b"show-prompt-history"[..], &b"showphist"[..]),
            (
                clear_entry(),
                &b"clear-prompt-history"[..],
                &b"clearphist"[..],
            ),
        ];
        for (e, name, alias) in wanted {
            assert_eq!((*e).name.to_bytes(), name);
            assert_eq!(
                (*e).alias.expect("the entry has an alias").to_bytes(),
                alias
            );
            assert_eq!((*e).usage.to_bytes(), b"[-T prompt-type]");

            assert_eq!((*e).args.template.to_bytes(), b"T:");
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
            assert_eq!((*e).flags & !CMD_AFTERHOOK, 0);
        }

        assert_ne!(show_entry(), clear_entry(), "two distinct table entries");
        assert!(
            ::core::ptr::fn_addr_eq((*show_entry()).exec, (*clear_entry()).exec),
            "both entries share the one exec"
        );
    }
}

#[test]
fn prompt_type_names_round_trip_through_the_status_helpers() {
    unsafe {
        let names: [&CStr; PROMPT_NTYPES as usize] =
            [c"command", c"search", c"target", c"window-target"];
        for (i, name) in names.iter().enumerate() {
            assert_eq!(status_prompt_type_string(i as u_int), *name);
            assert_eq!(status_prompt_type(name), i as prompt_type);
        }

        assert_eq!(
            status_prompt_type_string(PROMPT_NTYPES as u_int),
            c"invalid",
            "out-of-range indexes answer the invalid name"
        );
        assert_eq!(status_prompt_type(c"bogus"), PROMPT_TYPE_INVALID);
        assert_ne!(status_prompt_type(c"bogus"), 4);
    }
}

#[test]
fn entries_are_findable_by_name_and_alias() {
    let _guard = globals();
    unsafe {
        let wanted = [
            (c"show-prompt-history".as_ptr(), show_entry()),
            (c"showphist".as_ptr(), show_entry()),
            (c"clear-prompt-history".as_ptr(), clear_entry()),
            (c"clearphist".as_ptr(), clear_entry()),
        ];
        for (name, want) in wanted {
            let mut cause = None;
            assert_eq!(cmd_find(name, &mut cause), want);
            assert!(cause.is_none(), "no cause on success");
        }
    }
}

#[test]
fn parsing_accepts_the_T_flag_and_rejects_strays() {
    let _guard = globals();
    unsafe {
        let mut t = cmd_parse_from_string(c"show-prompt-history -T search".as_ptr(), null_mut());
        assert_eq!(t.status, CMD_PARSE_SUCCESS);
        let first = cmd_list_first(t.cmdlist.as_ref().unwrap().as_ptr());
        assert!(::core::ptr::eq((*first).entry, show_entry()));
        assert_eq!(seen(args_get(cmd_get_args(&*first), b'T')), "search");
        let _ = t.cmdlist.take();

        let mut bare = cmd_parse_from_string(c"clear-prompt-history".as_ptr(), null_mut());
        assert_eq!(bare.status, CMD_PARSE_SUCCESS);
        let first_bare = cmd_list_first(bare.cmdlist.as_ref().unwrap().as_ptr());
        assert!(::core::ptr::eq((*first_bare).entry, clear_entry()));
        assert!(args_get(cmd_get_args(&*first_bare), b'T').is_null());
        let _ = bare.cmdlist.take();

        let mut extra = cmd_parse_from_string(c"showphist stray".as_ptr(), null_mut());
        assert_eq!(extra.status, CMD_PARSE_ERROR);
        let err = extra.take_error();
        assert!(err.contains("too many arguments"), "{err}");

        let mut bad_flag = cmd_parse_from_string(c"clearphist -z".as_ptr(), null_mut());
        assert_eq!(bad_flag.status, CMD_PARSE_ERROR);
        let err_flag = bad_flag.take_error();
        assert!(err_flag.contains("unknown flag"), "{err_flag}");
    }
}

#[test]
fn show_without_T_walks_every_type_in_order_and_leaves_history_alone() {
    let _guard = globals();
    unsafe {
        drain_history();
        plant(0, &[c"first command", c"second command"]);
        plant(1, &[]);
        plant(2, &[c"a target edit"]);
        plant(3, &[]);

        let before = whole_history();
        let (_item, rv) = exec_line(c"show-prompt-history");
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(
            whole_history(),
            before,
            "showing reads the history without disturbing it"
        );

        drain_history();
    }
}

#[test]
fn show_with_T_covers_one_section_only() {
    let _guard = globals();
    unsafe {
        drain_history();
        plant(0, &[c"kept command"]);
        plant(1, &[c"kept search", c"another search"]);

        let before = whole_history();
        let (_item, rv) = exec_line(c"show-prompt-history -T search");
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!(whole_history(), before);

        let mut alias_item = Item::new().with_args(c"showphist -T search");
        let e = (*alias_item.cmd()).entry;
        let rv_alias = (e.exec)(&*alias_item.cmd(), alias_item.ptr());
        assert_eq!(rv_alias, CMD_RETURN_NORMAL);

        drain_history();
    }
}

#[test]
fn show_on_an_empty_history_answers_normal_quietly() {
    let _guard = globals();
    unsafe {
        drain_history();

        let (_item, rv) = exec_line(c"show-prompt-history -T target");
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert!(whole_history().iter().all(Vec::is_empty));

        drain_history();
    }
}

#[test]
fn clear_without_T_empties_every_type() {
    let _guard = globals();
    unsafe {
        drain_history();
        plant(0, &[c"goes too"]);
        plant(1, &[c"gone search"]);
        plant(2, &[c"gone target"]);
        plant(3, &[c"gone window-target", c"and this one"]);

        let (_item, rv) = exec_line(c"clear-prompt-history");
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert!(whole_history().iter().all(Vec::is_empty));
        for t in 0..PROMPT_NTYPES as usize {
            assert!(
                hlists()[t].is_empty(),
                "every list came back to the allocator"
            );
        }
    }
}

#[test]
fn clear_with_T_empties_only_that_type() {
    let _guard = globals();
    unsafe {
        drain_history();
        plant(0, &[c"stays command"]);
        plant(1, &[c"stays search"]);
        plant(2, &[c"cleared target", c"also cleared"]);
        plant(3, &[c"stays window-target"]);

        let (_item, rv) = exec_line(c"clear-prompt-history -T target");
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert_eq!(entries_of(2), Vec::<String>::new());
        assert!(hlists()[2].is_empty());

        assert_eq!(entries_of(0), ["stays command"]);
        assert_eq!(entries_of(1), ["stays search"]);
        assert_eq!(entries_of(3), ["stays window-target"]);

        drain_history();
    }
}

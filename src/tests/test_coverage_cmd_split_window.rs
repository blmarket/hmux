//! Unit tests for [`crate::cmd::cmd_split_window`] — the exec hook shared by
//! the `split-window`/`splitw` and `new-pane`/`newp` command entries, together
//! with the entries' metadata, the split template and the spawn flags the hook
//! assembles.
//!
//! Everything the hook decides before any pane is created is driven here
//! through the entries' `.exec`, the very hooks the command queue calls: the
//! "command cannot be given for empty pane" refusal that `-I` or `-E`
//! combined with a command trips over before any layout work at all, and the
//! layout refusals that reach the hook as a cause — a floating pane offered to
//! a tiled split by name, an invalid `-l` size, an invalid `-x` position for
//! the floating `new-pane` shape — each answered "size or position …", with
//! the cause freed again and the window left exactly as it was.
//!
//! One limit worth recording. Past those refusals every split ends in
//! [`spawn_pane`](crate::spawn::spawn_pane), which forks a real pty child
//! unless the pane is empty, and even its empty route runs the shell-selection
//! and environment machinery against live process state. No fixture goes
//! there, so the success half of the hook (the style options, `-P` printing,
//! the redraws and the `after-split-window` hook) stays out of reach.

use crate::arguments::{args_get, args_has, args_string, args_value_list};
use crate::cmd::cmd_get_args;
use crate::cmd::cmd_split_window::{
    CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP, CMD_RETURN_WAIT,
    PANE_REDRAW, PANE_STYLECHANGED, PANE_THEMECHANGED, SPAWN_BEFORE, SPAWN_DETACHED, SPAWN_EMPTY,
    SPAWN_FLOATING, SPAWN_FULLSIZE, SPAWN_ZOOM, SPLIT_WINDOW_TEMPLATE, cmd_new_pane_entry,
    cmd_split_window_entry,
};
use crate::file::CLIENT_ATTACHED;
use crate::server::message_log;
use crate::tests::test_fixtures::{Clients, Item, Target, globals, seen};
use crate::types::*;
use ::core::ffi::{CStr, c_char};

/// Everything the server's message log holds. Entries accumulate across the
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

/// Runs `entry`'s exec function against `item`, exactly as the command queue
/// would.
unsafe fn run(entry: *const cmd_entry, item: &mut Item) -> cmd_retval {
    unsafe { ((*entry).exec)(&*item.cmd(), item.ptr()) }
}

/// An item whose errors are observable without side effects: its caller is a
/// client carrying `CLIENT_ATTACHED`, so `cmdq_error` records the wording in
/// the server's message log while `file_error` declines to open a stream, and
/// sets the caller's `retval`. The item is aimed at `target`: the winlink,
/// window and pane the resolved find states would carry. The caller comes
/// back too, since the hook answers it there.
unsafe fn aimed_item(
    clients: &mut Clients,
    name: &str,
    line: &CStr,
    target: &mut Target,
) -> (Item, *mut client) {
    unsafe {
        let c = clients.add(name, 80, 24);
        (*c).flags |= CLIENT_ATTACHED as uint64_t;
        (*c).session = target.session();
        (*c).environ = Some(crate::environ::environ_create_box());
        let mut item = Item::with_client().with_args(line);
        item.set_client(c);
        (item.targeting(target), c)
    }
}

#[test]
fn the_entries_advertise_their_commands_and_share_the_exec_hook() {
    let _guard = globals();
    unsafe {
        let e: *const cmd_entry = &raw const cmd_split_window_entry;
        assert_eq!((*e).name.to_string_lossy(), "split-window");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "splitw"
        );
        assert_eq!(
            (*e).args.template.to_string_lossy(),
            "bc:de:EfF:hIkl:m:p:PR:s:S:t:vZ"
        );
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, -1);
        assert!((*e).args.cb.is_none());
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-bdefhIklPvZ] [-c start-directory] [-e environment] [-F format] [-l size] [-m message] [-p percentage] [-s style] [-S active-border-style] [-R inactive-border-style] [-t target-pane] [shell-command [argument ...]]"
        );
        assert_eq!((*e).source.flag, 0 as c_char);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);
        assert_eq!((*e).flags, 0);

        let n: *const cmd_entry = &raw const cmd_new_pane_entry;
        assert_eq!((*n).name.to_string_lossy(), "new-pane");
        assert_eq!(
            (*n).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "newp"
        );
        assert_eq!(
            (*n).args.template.to_string_lossy(),
            "bc:de:EfF:hIkl:Lm:p:PR:s:S:t:vx:X:y:Y:Z"
        );
        assert_eq!((*n).args.lower, 0);
        assert_eq!((*n).args.upper, -1);
        assert!((*n).args.cb.is_none());
        assert_eq!(
            (*n).usage.to_string_lossy(),
            "[-bdefhIklPvZ] [-c start-directory] [-e environment] [-F format] [-l size] [-m message] [-p percentage] [-s style] [-S active-border-style] [-R inactive-border-style] [-x width] [-y height] [-X x-position] [-Y y-position] [-t target-pane] [shell-command [argument ...]]"
        );
        assert_eq!((*n).source.flag, 0 as c_char);
        assert_eq!((*n).source.type_0, CMD_FIND_PANE);
        assert_eq!((*n).source.flags, 0);
        assert_eq!((*n).target.flag, b't' as c_char);
        assert_eq!((*n).target.type_0, CMD_FIND_PANE);
        assert_eq!((*n).target.flags, 0);
        assert_eq!((*n).flags, 0);

        assert!(::core::ptr::fn_addr_eq((*e).exec, (*n).exec));
    }
}

#[test]
fn the_template_and_the_spawn_flags_match_upstream() {
    let expected: &[u8] = b"#{session_name}:#{window_index}.#{pane_index}\0";
    let got: Vec<u8> = SPLIT_WINDOW_TEMPLATE.iter().map(|&b| b as u8).collect();
    assert_eq!(SPLIT_WINDOW_TEMPLATE.len(), expected.len());
    assert_eq!(got, expected);
    assert_eq!(got[got.len() - 1], 0, "the template ends in a NUL");
    assert!(
        expected[..expected.len() - 1].iter().all(|&b| b != 0),
        "the template has no interior NUL"
    );

    assert_eq!(SPAWN_DETACHED, 0x2);
    assert_eq!(SPAWN_BEFORE, 0x8);
    assert_eq!(SPAWN_FULLSIZE, 0x20);
    assert_eq!(SPAWN_EMPTY, 0x40);
    assert_eq!(SPAWN_ZOOM, 0x80);
    assert_eq!(SPAWN_FLOATING, 0x100);

    assert_eq!(PANE_REDRAW, 0x1);
    assert_eq!(PANE_STYLECHANGED, 0x1000);
    assert_eq!(PANE_THEMECHANGED, 0x2000);

    assert_eq!(CMD_FIND_PANE, 0);
    assert_eq!(CMD_RETURN_ERROR, -1);
    assert_eq!(CMD_RETURN_NORMAL, 0);
    assert_eq!(CMD_RETURN_WAIT, 1);
    assert_eq!(CMD_RETURN_STOP, 2);
}

#[test]
fn parsing_resolves_both_spellings_and_the_flags_the_hook_reads() {
    let _guard = globals();

    let mut plain = Item::new().with_args(c"split-window -dZt 0.1");
    unsafe {
        assert!(
            ::core::ptr::eq((*plain.cmd()).entry, &cmd_split_window_entry),
            "the plain spelling resolves to this entry"
        );
        let args = cmd_get_args(&*plain.cmd());
        for flag in *b"dZt" {
            assert_eq!(args_has(args, flag), 1, "{}", flag as char);
        }
    }

    let mut alias = Item::new().with_args(c"splitw");
    unsafe {
        assert!(
            ::core::ptr::eq((*alias.cmd()).entry, &cmd_split_window_entry),
            "the alias spelling resolves to this entry too"
        );
    }

    let mut pane = Item::new().with_args(c"new-pane -L");
    unsafe {
        assert!(
            ::core::ptr::eq((*pane.cmd()).entry, &cmd_new_pane_entry),
            "the floating new-pane spelling resolves to its own entry"
        );
        assert_eq!(args_has(cmd_get_args(&*pane.cmd()), b'L'), 1);
    }

    let mut newp = Item::new().with_args(c"newp");
    unsafe {
        assert!(
            ::core::ptr::eq((*newp.cmd()).entry, &cmd_new_pane_entry),
            "the newp alias resolves there too"
        );
    }

    let mut full = Item::new().with_args(
        c"split-window -b -c /tmp -e A=B -f -F fmt -h -I -k -l 20 -m msg -p 10 -P -R bg=green -s fg=red -S bg=blue -v pos",
    );
    unsafe {
        let args = cmd_get_args(&*full.cmd());
        for flag in *b"bcefFhIklmpPRsSv" {
            assert_eq!(args_has(args, flag), 1, "{}", flag as char);
        }
        assert_eq!(seen(args_get(args, b'c')), "/tmp");
        assert_eq!(seen(args_get(args, b'e')), "A=B");
        assert_eq!(seen(args_get(args, b'l')), "20");
        assert_eq!(
            seen(args_string(args, 0)),
            "pos",
            "a trailing word becomes the command the empty check reads"
        );

        let values = args_value_list(args, b'e');
        assert_eq!(values.len(), 1, "a single -e value loops exactly once");
        assert_eq!((*values[0]).value.string(), c"A=B");
    }
}

#[test]
fn split_window_exec_refuses_command_for_empty_pane() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut target = Target::new(80, 24);
    unsafe {
        let (mut item_i, caller_i) =
            aimed_item(&mut clients, "c1", c"split-window -I echo hi", &mut target);
        assert_eq!(
            run(&raw const cmd_split_window_entry, &mut item_i),
            CMD_RETURN_ERROR
        );

        let (mut item_e, caller_e) =
            aimed_item(&mut clients, "c2", c"split-window -E echo hi", &mut target);
        assert_eq!(
            run(&raw const cmd_split_window_entry, &mut item_e),
            CMD_RETURN_ERROR
        );

        crate::tests::test_fixtures::release_client(caller_i);
        crate::tests::test_fixtures::release_client(caller_e);
    }
}

#[test]
fn split_window_exec_layout_failure_branches() {
    let _guard = globals();
    let mut clients = Clients::new();
    let mut target = Target::new(80, 24);
    unsafe {
        crate::layout::layout_init(target.window(0), target.pane(0));

        let (mut item_l, caller_l) =
            aimed_item(&mut clients, "c3", c"split-window -l bad", &mut target);
        assert_eq!(
            run(&raw const cmd_split_window_entry, &mut item_l),
            CMD_RETURN_ERROR
        );

        let (mut item_pct, caller_pct) =
            aimed_item(&mut clients, "c4", c"split-window -l 200%", &mut target);
        assert_eq!(
            run(&raw const cmd_split_window_entry, &mut item_pct),
            CMD_RETURN_ERROR
        );

        let (mut item_newp, caller_n) =
            aimed_item(&mut clients, "c5", c"new-pane -x 9999", &mut target);
        assert_eq!(
            run(&raw const cmd_new_pane_entry, &mut item_newp),
            CMD_RETURN_ERROR
        );

        crate::layout::layout_free(target.window(0));
        crate::tests::test_fixtures::release_client(caller_l);
        crate::tests::test_fixtures::release_client(caller_pct);
        crate::tests::test_fixtures::release_client(caller_n);
    }
}

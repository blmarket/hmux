//! Unit tests for [`crate::cmd::cmd_pipe_pane`], the exec hook behind the
//! `pipe-pane` command.
//!
//! What is covered here is everything that answers without creating anything:
//! the entry's metadata and the constants the module re-declares for its own
//! compilation, the parser's resolution of both spellings `pipe-pane` and
//! `pipep` together with its enforcement of the `IOot:` template, and the
//! deterministic heads of [`cmd_pipe_pane_exec`] — the refusal of a pane that
//! has already exited, the tearing-down of a pipe that is already running when
//! the command carries no new one or carries `-o`, and the empty-command
//! short circuit. Each of those ends in a plain `CMD_RETURN_NORMAL` or an
//! error filed through the item's client, and none of them reaches the
//! `socketpair`/`fork`/`execl` tail of exec, where a child shell would be
//! spawned and a ensure_reactor buffer event armed; coverage deliberately stops at
//! that boundary rather than forcing it. The read, write and error callbacks
//! are driven by the event loop and are likewise left alone.
//!
//! One mechanical note. Exec frees `wp->pipe_event` whenever it tears an old
//! pipe down, so the rig hands the pane a real, callback-free buffer event
//! built the way the harness's own [`StreamBuffer`] fixture builds them —
//! constructed once, never enabled, never polled — over one end of a private
//! socket pair. The pane's `fd` is the other end's twin so that
//! `window_pane_exited` sees a live pane and the `ioctl(FIONREAD)` probe in
//! `window_pane_destroy_ready` sees an empty queue; every descriptor the rig
//! opens is closed again by the time its test ends.

use crate::arguments::{args_count, args_has, args_string};
use crate::cmd::cmd_get_args;
use crate::cmd::cmd_pipe_pane::{
    CMD_AFTERHOOK, CMD_FIND_PANE, CMD_RETURN_NORMAL, cmd_pipe_pane_entry,
};
use crate::cmd::{CMD_PARSE_ERROR, CMD_PARSE_SUCCESS, cmd_parse_from_string};
use crate::proc::PEER_BAD;
use crate::server::message_log;
use crate::tests::test_fixtures::{Args, Item, Target, ensure_reactor, globals, seen, zeroed};
use crate::types::*;
use ::core::ffi::{c_char, c_int};
use ::core::ptr::null_mut;

/// A peer for the fixture client, marked bad so `proc_send` refuses any
/// message before it reaches a buffer underneath it.
fn bad_peer() -> Box<tmuxpeer> {
    let mut p = zeroed::<tmuxpeer>();
    p.flags |= PEER_BAD;
    p
}

/// Gives `c` its peer. Its session stays null and its flags stay clear, which
/// sends `cmdq_error` down the branch that files the message in the server's
/// message log before opening the client's error stream.
unsafe fn wire(c: *mut client) {
    unsafe {
        (*c).peer = Some(bad_peer());
    }
}

/// Runs the item's parsed command through the entry's exec hook, the way the
/// command queue would.
unsafe fn run(item: &mut Item) -> cmd_retval {
    unsafe {
        let e = &raw const cmd_pipe_pane_entry;
        ((*e).exec)(&*item.cmd(), item.ptr())
    }
}

/// The lines the server has recorded so far, oldest first.
unsafe fn server_messages() -> Vec<String> {
    unsafe {
        let mut out = Vec::new();
        for m in message_log.queue().iter() {
            out.push(seen(m.msg.as_ptr()));
        }
        out
    }
}

/// A private socket pair whose still-open ends are closed when it goes away.
/// Tests hand the ends out with [`Pair::take`], which stops the drop from
/// closing them again once somebody else — usually exec itself — has.
struct Pair([c_int; 2]);

impl Pair {
    fn new() -> Pair {
        let mut fds = [-1 as c_int; 2];
        assert_eq!(
            unsafe {
                ::libc::socketpair(::libc::AF_UNIX, ::libc::SOCK_STREAM, 0, fds.as_mut_ptr())
            },
            0,
            "no socket pair"
        );
        assert!(fds[0] >= 0 && fds[1] >= 0);
        Pair(fds)
    }

    /// Takes end `i` out, answering its descriptor and leaving its closure to
    /// whoever took it.
    fn take(&mut self, i: usize) -> c_int {
        let fd = self.0[i];
        self.0[i] = -1;
        fd
    }
}

impl Drop for Pair {
    fn drop(&mut self) {
        for &fd in &self.0 {
            if fd >= 0 {
                unsafe {
                    ::libc::close(fd);
                }
            }
        }
    }
}

/// Makes the target's active pane look alive: its own descriptor is one end of
/// `pair`, its running pipe the other, watched by a real but inert buffer
/// event with no callbacks, which exec may free. Answers the buffer event.
unsafe fn arm(wp: *mut window_pane, pair: &mut Pair) -> Stream {
    ensure_reactor();
    unsafe {
        let bev = Stream::new(pair.0[1], None, None, None);
        assert!(!bev.is_none(), "no buffer event");
        (*wp).fd = pair.take(0);
        (*wp).pipe_fd = pair.take(1);
        (*wp).pipe_event = bev;
        bev
    }
}

#[test]
fn the_entry_advertises_pipe_pane() {
    let _guard = globals();
    unsafe {
        let e = &raw const cmd_pipe_pane_entry;
        assert_eq!((*e).name.to_string_lossy(), "pipe-pane");
        assert_eq!(
            (*e).alias
                .expect("the entry has an alias")
                .to_string_lossy(),
            "pipep"
        );
        assert_eq!((*e).args.template.to_string_lossy(), "IOot:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 1);
        assert!((*e).args.cb.is_none());
        assert_eq!(
            (*e).usage.to_string_lossy(),
            "[-IOo] [-t target-pane] [shell-command]"
        );
        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);
        assert_eq!((*e).target.flag, b't' as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, 0);
        assert_eq!((*e).flags, CMD_AFTERHOOK);
        assert_eq!(CMD_AFTERHOOK, 0x4);
        let exec = (*e).exec as usize;
        assert_ne!(exec, 0);
    }
}

#[test]
fn parsing_resolves_both_spellings_of_the_command_to_this_entry() {
    let _guard = globals();
    unsafe {
        for line in [c"pipe-pane", c"pipep"] {
            let parsed = Args::parse(line);
            assert!(
                ::core::ptr::eq((*parsed.cmd()).entry, &cmd_pipe_pane_entry),
                "{line:?} did not resolve"
            );
            let args = parsed.ptr();
            assert_eq!(args_count(&*args), 0);
            assert_eq!(args_has(&*args, b'I'), 0);
            assert_eq!(args_has(&*args, b'O'), 0);
            assert_eq!(args_has(&*args, b'o'), 0);
            assert_eq!(args_has(&*args, b't'), 0);
        }

        let out = Args::parse(c"pipe-pane -O cat");
        assert!(::core::ptr::eq((*out.cmd()).entry, &cmd_pipe_pane_entry));
        let args = out.ptr();
        assert_eq!(args_has(&*args, b'O'), 1);
        assert_eq!(args_has(&*args, b'I'), 0);
        assert_eq!(args_count(&*args), 1);
        assert_eq!(seen(args_string(&*args, 0)), "cat");

        let everything = Args::parse(c"pipep -I -O -o -t %1 cat");
        assert!(::core::ptr::eq(
            (*everything.cmd()).entry,
            &cmd_pipe_pane_entry
        ));
        let args = everything.ptr();
        assert_eq!(args_has(&*args, b'I'), 1);
        assert_eq!(args_has(&*args, b'O'), 1);
        assert_eq!(args_has(&*args, b'o'), 1);
        assert_eq!(args_has(&*args, b't'), 1);
        assert_eq!(args_count(&*args), 1);
        assert_eq!(seen(args_string(&*args, 0)), "cat");
    }
}

#[test]
fn parsing_enforces_at_most_one_shell_command_and_known_flags() {
    let _guard = globals();
    unsafe {
        let mut extra = cmd_parse_from_string(c"pipe-pane cat dog".as_ptr(), null_mut());
        assert_eq!(extra.status, CMD_PARSE_ERROR);
        let err = extra.take_error();
        assert!(err.contains("too many arguments"), "{err}");

        let mut bad_flag = cmd_parse_from_string(c"pipe-pane -z cat".as_ptr(), null_mut());
        assert_eq!(bad_flag.status, CMD_PARSE_ERROR);
        let err_flag = bad_flag.take_error();
        assert!(err_flag.contains("unknown flag"), "{err_flag}");

        let mut good = cmd_parse_from_string(c"pipe-pane -o true".as_ptr(), null_mut());
        assert_eq!(good.status, CMD_PARSE_SUCCESS);
        let _ = good.cmdlist.take();
    }
}

#[test]
fn stopping_an_active_pipe_tears_it_down_and_answers_normal() {
    let _guard = globals();
    let mut pair = Pair::new();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let bev = arm(wp, &mut pair);
        let live_fd = (*wp).fd;

        let mut item = Item::new().with_args(c"pipe-pane").targeting(&mut t);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);

        assert_eq!((*wp).pipe_fd, -1, "the running pipe was closed");
        assert_eq!((*wp).pipe_event, bev, "exec leaves the stale pointer alone");
        assert_eq!((*wp).fd, live_fd, "the pane's own descriptor survives");
    }
}

#[test]
fn an_empty_shell_command_stops_the_pipe_without_spawning_anything() {
    let _guard = globals();
    let mut pair = Pair::new();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        arm(wp, &mut pair);
        let live_fd = (*wp).fd;

        let mut item = Item::new().with_args(c"pipe-pane \"\"").targeting(&mut t);
        let args = cmd_get_args(&*item.cmd());
        assert_eq!(args_count(args), 1);
        assert_eq!(*args_string(args, 0), 0, "the argument is one NUL byte");

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*wp).pipe_fd, -1);
        assert_eq!((*wp).fd, live_fd);
    }
}

#[test]
fn the_once_flag_over_a_live_pipe_answers_normal_without_rebuilding() {
    let _guard = globals();
    let mut pair = Pair::new();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        let bev = arm(wp, &mut pair);
        let live_fd = (*wp).fd;

        let mut item = Item::new().with_args(c"pipe-pane -o cat").targeting(&mut t);
        assert_eq!(args_has(cmd_get_args(&*item.cmd()), b'o'), 1);
        assert_eq!(seen(args_string(cmd_get_args(&*item.cmd()), 0)), "cat");

        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_eq!((*wp).pipe_fd, -1, "the old pipe was still torn down");
        assert_eq!((*wp).pipe_event, bev);
        assert_eq!((*wp).fd, live_fd);
    }
}

#[test]
fn spawning_pipe_with_command_and_flags() {
    let _guard = globals();
    ensure_reactor();
    let mut proc = Box::new(tmuxproc::default());
    let prev_proc = unsafe { crate::server::server_proc };
    unsafe { crate::server::server_proc = &raw mut *proc };
    let mut pair = Pair::new();
    let mut t = Target::new(80, 24);
    unsafe {
        let wp = t.pane(0);
        (*wp).fd = pair.take(0);

        let mut item = Item::new()
            .with_args(c"pipe-pane -I -O true")
            .targeting(&mut t);
        assert_eq!(run(&mut item), CMD_RETURN_NORMAL);
        assert_ne!((*wp).pipe_fd, -1);
        assert!(!(*wp).pipe_event.is_none());

        let mut item_stop = Item::new().with_args(c"pipe-pane").targeting(&mut t);
        assert_eq!(run(&mut item_stop), CMD_RETURN_NORMAL);
        assert_eq!((*wp).pipe_fd, -1);
        crate::server::server_proc = prev_proc;
    }
}

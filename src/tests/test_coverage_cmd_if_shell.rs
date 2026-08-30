//! Unit tests for [`crate::cmd::cmd_if_shell`] — the `if-shell` entry
//! metadata, the constants the file re-declares, its argument-classification
//! callback and every deterministic branch of [`exec`] and of the job
//! completion callback the fixtures can reach without spawning a process.
//!
//! Exec is reached through the entry's own function pointer, exactly as the
//! command queue calls it, over items whose arguments come from the real
//! command parser and whose targets are resolved against a registered
//! [`Target`]. Only the `-F` side of exec is driven: without `-F` exec hands
//! the condition to `job_run`, which forks a real `/bin/sh` — the same line
//! the other suites draw. The `-F` branches are fully deterministic, though:
//! the formatted condition decides between the then-commands, the
//! else-commands and doing nothing, and every outcome is answered straight
//! away with commands spliced after the running item.
//!
//! The completion callback and the free function have no public caller short
//! of a finished job — they only ever run from inside the job machinery — so
//! both carry a test-only `pub` on their definitions and are driven here the
//! way `job_complete`/`job_free` would: a fixture [`job`] whose data is the
//! private [`cmd_if_shell_data`] built exactly as exec builds it, with command
//! states prepared through the real `args_make_commands_prepare`. Exit status
//! bit patterns choose the if- or else-state, queue effects are read back off
//! a fixture queue, and cleanup goes through the free function itself.
//!
//! Queues under items are wired and taken apart the way the confirm-before
//! suite does it; refusals report through `cmdq_error` onto clients whose
//! `CLIENT_ATTACHED` flag keeps `file_error` out of the peer's way.

use crate::arguments::args_create;
use crate::arguments::args_make_commands_prepare;
use crate::cmd::cmd_attach_session::CLIENT_ATTACHED;
use crate::cmd::cmd_if_shell::{
    ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID, ARGS_PARSE_STRING,
    CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CMD_FIND_CANFAIL, CMD_FIND_PANE,
    CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP,
    CMD_RETURN_WAIT, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MSG_COMMAND,
    MSG_DETACH, MSG_DETACHKILL, MSG_EXEC, MSG_EXIT, MSG_EXITED, MSG_EXITING, MSG_FLAGS,
    MSG_IDENTIFY_CLIENTPID, MSG_IDENTIFY_CWD, MSG_IDENTIFY_DONE, MSG_IDENTIFY_ENVIRON,
    MSG_IDENTIFY_FEATURES, MSG_IDENTIFY_FLAGS, MSG_IDENTIFY_LONGFLAGS, MSG_IDENTIFY_OLDCWD,
    MSG_IDENTIFY_STDIN, MSG_IDENTIFY_STDOUT, MSG_IDENTIFY_TERM, MSG_IDENTIFY_TERMINFO,
    MSG_IDENTIFY_TTYNAME, MSG_LOCK, MSG_OLDSTDERR, MSG_OLDSTDIN, MSG_OLDSTDOUT, MSG_READ,
    MSG_READ_CANCEL, MSG_READ_DONE, MSG_READ_OPEN, MSG_READY, MSG_RESIZE, MSG_SHELL, MSG_SHUTDOWN,
    MSG_SUSPEND, MSG_UNLOCK, MSG_VERSION, MSG_WAKEUP, MSG_WRITE, MSG_WRITE_CLOSE, MSG_WRITE_OPEN,
    MSG_WRITE_READY, PANE_LINES_DOUBLE, PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE,
    PANE_LINES_SINGLE, PANE_LINES_SPACES, PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN,
    PROGRESS_BAR_INDETERMINATE, PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED, PROMPT_COMMAND,
    PROMPT_ENTRY, PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET,
    PROMPT_TYPE_WINDOW_TARGET, SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT,
    SCREEN_CURSOR_UNDERLINE, STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT,
    STYLE_ALIGN_LEFT, STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, cmd_if_shell_callback, cmd_if_shell_data,
    cmd_if_shell_entry, cmd_if_shell_free,
};
use crate::cmd::{CMDQ_WAITING, CmdqType};
use crate::cmd::{CmdqItemWeak, cmdq_item_weak_from_ptr};
use crate::job::job;
use crate::server::message_log;
use crate::server::{TTY_FREEZE, TTY_NOCURSOR, client_ref_from_ptr};
use crate::status::{status_init, status_message_clear};
use crate::tests::test_fixtures::{Clients, Item, Target, ensure_reactor, globals, seen, zeroed};
use crate::types::*;
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::{null, null_mut};
use std::sync::MutexGuard;

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

/// A command queue a test owns until a client takes it over, wired onto a
/// fixture item or client whose queue field would otherwise still be zeroed.
struct Queue {
    q: *mut cmdq_list,
    owned: Option<Box<cmdq_list>>,
    current: ::core::cell::Cell<*mut cmdq_item>,
}

impl Queue {
    fn new() -> Queue {
        let mut owned = crate::cmd::cmdq_new();
        Queue {
            q: &raw mut *owned,
            owned: Some(owned),
            current: ::core::cell::Cell::new(null_mut()),
        }
    }

    /// Hands the queue to `c`, which owns it from then on: the fixture keeps
    /// reading it, and the client frees it when it goes.
    fn attach(&mut self, c: *mut client) {
        unsafe { (*c).queue = self.owned.take() };
    }

    /// Registers `current` as the item the queue is in the middle of running,
    /// which is what a real queue holding a waiting command looks like. Only
    /// then does something inserted *after* it land on this queue at all:
    /// `cmdq_insert_after` splices behind the running item and never touches
    /// the head pointer itself. The running item's shared state gets an extra
    /// reference for the fixture holding it, so that taking the queued items
    /// off again can never release — let alone free — what the fixture owns.
    fn wire(&mut self, current: &mut Item) {
        unsafe {
            (*self.q).list.clear();
            self.current.set(current.queue_onto(&mut *self.q));
        }
    }

    /// The items queued behind the wired one, in order.
    fn behind(&self) -> Vec<*mut cmdq_item> {
        unsafe {
            let skip = usize::from(!self.current.get().is_null());
            (*self.q)
                .list
                .iter()
                .skip(skip)
                .map(|item| item.as_ptr())
                .collect()
        }
    }

    /// The first item queued behind the wired one — what the tests assert on.
    fn start(&self) -> *mut cmdq_item {
        self.behind()
            .first()
            .copied()
            .unwrap_or(null_mut::<cmdq_item>())
    }

    /// The items queued behind the wired one, by name, in order.
    fn names(&self) -> Vec<String> {
        unsafe {
            self.behind()
                .into_iter()
                .map(|it| seen(cstr_ptr(&(*it).name)))
                .collect()
        }
    }

    /// Takes every item queued behind the wired one off again the way
    /// `cmdq_remove` would, which unrefs its client and drops its
    /// command-list reference and shared state with it. The wired item itself
    /// stays put and keeps owning the list.
    fn discard(&mut self) -> usize {
        unsafe {
            let skip = usize::from(!self.current.get().is_null());
            let n = (*self.q).list.len().saturating_sub(skip);
            (*self.q).list.truncate(skip);
            n
        }
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        self.discard();
    }
}

/// What [`ran`] hands back, kept together so it comes apart in a safe order:
/// queued commands are taken apart while the client fixtures are still alive,
/// the item goes next, then the registered target, then the clients, and the
/// globals turn is given up last.
struct Ran {
    queue: Queue,
    item: Item,
    target: Target,
    clients: Clients,
    c: *mut client,
    _guard: MutexGuard<'static, ()>,
}

/// Runs an `if-shell -F …` line through the entry's own exec hook, the way
/// the command queue would call it. The item carries a parsed target and runs
/// wired into a fixture queue, since exec splices its answer in right behind
/// the running item. A fixture client rides along as the item's (target)
/// client, marked attached so error reports stay clear of the peer.
fn ran(line: &'static CStr) -> (Ran, cmd_retval) {
    let _guard = globals();
    ensure_reactor();
    let mut target = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("if-shell", 80, 24);
    unsafe { (*c).flags |= CLIENT_ATTACHED as u64 };
    let mut item = Item::new().with_args(line);
    item.set_client(c);
    let mut item = item.targeting(&mut target);
    let mut queue = Queue::new();
    unsafe {
        queue.wire(&mut item);
        let rv = (cmd_if_shell_entry.exec)(&*item.cmd(), item.ptr());
        (
            Ran {
                queue,
                item,
                target,
                clients,
                c,
                _guard,
            },
            rv,
        )
    }
}

/// What a fabricated job carries into the completion callback: the private
/// data built exactly as exec builds it — command states prepared through the
/// real prepare hook over the parsed arguments — parked behind a zeroed job
/// whose status field tells the callback how the shell fared.
struct Harnessed {
    queue: Queue,
    job: Box<job>,
    cdata: *mut cmd_if_shell_data,
    item: Item,
    target: Target,
    clients: Clients,
    c: *mut client,
    freed: bool,
    _guard: MutexGuard<'static, ()>,
}

impl Drop for Harnessed {
    fn drop(&mut self) {
        if !self.freed {
            unsafe {
                let data = ::core::mem::take(&mut self.job.data);
                cmd_if_shell_free(data);
            }
        }
    }
}

impl Harnessed {
    /// Fires the completion callback the way `job_complete` would once the
    /// child has been reaped.
    unsafe fn fire(&mut self) {
        unsafe { cmd_if_shell_callback(&raw mut *self.job) }
    }
}

/// Builds the callback harness. `line` supplies the arguments the command
/// states hang off, `status` is the wait status the fake job reports,
/// `with_else` prepares a second state for argument two, and `keep_item`
/// leaves the waiting item reachable from the data — otherwise the data is
/// headless, which is what `-b` jobs look like to the callback once their
/// item has gone, and the client's own queue receives whatever runs.
unsafe fn harnessed(
    line: &'static CStr,
    status: c_int,
    with_else: bool,
    keep_item: bool,
) -> Harnessed {
    let _guard = globals();
    ensure_reactor();
    let mut target = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("cb", 80, 24);
    unsafe { (*c).flags |= CLIENT_ATTACHED as u64 };
    let mut item = Item::new().with_args(line);
    item.set_client(c);
    let mut item = item.targeting(&mut target);
    let mut queue = Queue::new();
    unsafe {
        let iptr = item.ptr();
        (*iptr).queue = queue.q;
        (*iptr).client = client_ref_from_ptr(c);
        if keep_item {
            (*iptr).flags |= CMDQ_WAITING;
            queue.wire(&mut item);
        } else {
            queue.attach(c);
        }
        let cmd = item.cmd();
        let cmd_if = Some(args_make_commands_prepare(&*cmd, iptr, 1, null(), 0, 0));
        let cmd_else = match with_else {
            true => Some(args_make_commands_prepare(&*cmd, iptr, 2, null(), 0, 0)),
            false => None,
        };
        let cdata = Box::new(cmd_if_shell_data {
            cmd_if,
            cmd_else,
            client_ref: client_ref_from_ptr(c),
            item: match keep_item {
                true => cmdq_item_weak_from_ptr(iptr),
                false => None,
            },
        });
        let cdata_ptr = cdata.as_ref() as *const cmd_if_shell_data as *mut cmd_if_shell_data;
        let mut jb = zeroed::<job>();
        jb.data = JobData::IfShell(cdata);
        jb.status = status;
        Harnessed {
            queue,
            job: jb,
            cdata: cdata_ptr,
            item,
            target,
            clients,
            c,
            freed: false,
            _guard,
        }
    }
}

#[test]
fn entry_metadata_matches_upstream() {
    unsafe {
        let e: *const cmd_entry = &raw const cmd_if_shell_entry;
        assert_eq!((*e).name.to_bytes(), b"if-shell");
        assert_eq!(
            (*e).alias.expect("the entry has an alias").to_bytes(),
            b"if"
        );
        assert_eq!(
            (*e).usage.to_bytes(),
            b"[-bF] [-t target-pane] shell-command command [command]"
        );

        assert_eq!((*e).args.template.to_bytes(), b"bFt:");
        assert_eq!((*e).args.lower, 2);
        assert_eq!((*e).args.upper, 3);
        assert!((*e).args.cb.is_some());

        assert_eq!((*e).source.flag, 0);
        assert_eq!((*e).source.type_0, CMD_FIND_PANE);
        assert_eq!((*e).source.flags, 0);

        assert_eq!((*e).target.flag, 't' as i32 as c_char);
        assert_eq!((*e).target.type_0, CMD_FIND_PANE);
        assert_eq!((*e).target.flags, CMD_FIND_CANFAIL);

        assert_eq!((*e).flags, 0);
    }
}

/// The entry's argument callback classifies by position alone: the condition
/// and the two command slots accept strings and `{ }` blocks alike, anything
/// else is a plain string.
#[test]
fn the_arguments_callback_classifies_by_position() {
    unsafe {
        let cb = cmd_if_shell_entry.args.cb.unwrap();
        let mut cause = None;
        assert_eq!(cb(&args_create(), 0, &mut cause), ARGS_PARSE_STRING);
        assert_eq!(
            cb(&args_create(), 1, &mut cause),
            ARGS_PARSE_COMMANDS_OR_STRING
        );
        assert_eq!(
            cb(&args_create(), 2, &mut cause),
            ARGS_PARSE_COMMANDS_OR_STRING
        );
        assert_eq!(cb(&args_create(), 3, &mut cause), ARGS_PARSE_STRING);
        assert_eq!(cb(&args_create(), 9, &mut cause), ARGS_PARSE_STRING);
    }
}

#[test]
fn return_and_find_constants_match_upstream() {
    assert_eq!(CMD_RETURN_NORMAL, 0);
    assert_eq!(CMD_RETURN_WAIT, 1);
    assert_eq!(CMD_RETURN_STOP, 2);
    assert_eq!(CMD_RETURN_ERROR, -1);

    assert_eq!(ARGS_PARSE_INVALID, 0);
    assert_eq!(ARGS_PARSE_STRING, 1);
    assert_eq!(ARGS_PARSE_COMMANDS_OR_STRING, 2);
    assert_eq!(ARGS_PARSE_COMMANDS, 3);

    assert_eq!(CMD_FIND_PANE, 0);
    assert_eq!(CMD_FIND_WINDOW, 1);
    assert_eq!(CMD_FIND_SESSION, 2);
    assert_eq!(CMD_FIND_CANFAIL, 0x40);
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
}

#[test]
fn style_layout_and_screen_constants_match_upstream() {
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

    assert_eq!(STYLE_DEFAULT_BASE, 0);
    assert_eq!(STYLE_DEFAULT_PUSH, 1);
    assert_eq!(STYLE_DEFAULT_POP, 2);
    assert_eq!(STYLE_DEFAULT_SET, 3);

    assert_eq!(STYLE_RANGE_NONE, 0);
    assert_eq!(STYLE_RANGE_LEFT, 1);
    assert_eq!(STYLE_RANGE_RIGHT, 2);
    assert_eq!(STYLE_RANGE_PANE, 3);
    assert_eq!(STYLE_RANGE_WINDOW, 4);
    assert_eq!(STYLE_RANGE_SESSION, 5);
    assert_eq!(STYLE_RANGE_USER, 6);
    assert_eq!(STYLE_RANGE_CONTROL, 7);

    assert_eq!(STYLE_LIST_OFF, 0);
    assert_eq!(STYLE_LIST_ON, 1);
    assert_eq!(STYLE_LIST_FOCUS, 2);
    assert_eq!(STYLE_LIST_LEFT_MARKER, 3);
    assert_eq!(STYLE_LIST_RIGHT_MARKER, 4);

    assert_eq!(STYLE_ALIGN_DEFAULT, 0);
    assert_eq!(STYLE_ALIGN_LEFT, 1);
    assert_eq!(STYLE_ALIGN_CENTRE, 2);
    assert_eq!(STYLE_ALIGN_RIGHT, 3);
    assert_eq!(STYLE_ALIGN_ABSOLUTE_CENTRE, 4);

    assert_eq!(THEME_UNKNOWN, 0);
    assert_eq!(THEME_LIGHT, 1);
    assert_eq!(THEME_DARK, 2);

    assert_eq!(LAYOUT_LEFTRIGHT, 0);
    assert_eq!(LAYOUT_TOPBOTTOM, 1);
    assert_eq!(LAYOUT_WINDOWPANE, 2);
}

#[test]
fn prompt_and_exit_constants_match_upstream() {
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
}

/// A true condition queues the then-commands directly behind the still
/// running item — sharing its state and client — and answers normal at once.
#[test]
fn exec_with_F_runs_the_then_commands_when_true() {
    let (mut r, rv) = ran(c"if-shell -F '1' display-panes");
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!((*r.item.ptr()).flags & CMDQ_WAITING, 0);

        let queued = r.queue.names();
        assert_eq!(queued.len(), 1, "{queued:?}");
        let qi = r.queue.start();
        assert!(
            seen(cstr_ptr(&(*qi).name)).starts_with("[display-panes/"),
            "{queued:?}"
        );
        assert!(matches!((*qi).type_0, CmdqType::Command { .. }));
        assert_eq!((*qi).state(), (*r.item.ptr()).state());
        assert_eq!(crate::cmd::cmdq_get_client(&*qi), r.c);
        assert_eq!(r.queue.discard(), 1);
    }
}

/// An untrue condition queues the else-commands instead.
#[test]
fn exec_with_F_runs_the_else_commands_when_false() {
    let (mut r, rv) = ran(c"if-shell -F '0' display-panes run-shell");
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);
        let queued = r.queue.names();
        assert_eq!(queued.len(), 1, "{queued:?}");
        assert!(
            seen(cstr_ptr(&(*(r.queue.start())).name)).starts_with("[run-shell/"),
            "{queued:?}"
        );
        assert_eq!(r.queue.discard(), 1);
    }
}

/// Only the first character of the condition matters: anything beginning
/// with `0` counts as false, however long it goes on.
#[test]
fn exec_with_F_treats_a_leading_zero_as_false() {
    let (mut r, rv) = ran(c"if-shell -F '01' display-panes run-shell");
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);
        let queued = r.queue.names();
        assert_eq!(queued.len(), 1, "{queued:?}");
        assert!(
            seen(cstr_ptr(&(*(r.queue.start())).name)).starts_with("[run-shell/"),
            "{queued:?}"
        );
        assert_eq!(r.queue.discard(), 1);
    }
}

/// An empty condition is false too, and with nothing to run instead the
/// command simply answers normal having queued nothing.
#[test]
fn exec_with_F_treats_an_empty_condition_as_false_without_else() {
    let (mut r, rv) = ran(c"if-shell -F '' display-panes");
    assert_eq!(rv, CMD_RETURN_NORMAL);
    assert!(r.queue.names().is_empty());
    assert_eq!(r.queue.discard(), 0);
}

/// The condition is a format string evaluated against the target before the
/// truth test: this session's `$0` id expands to something true.
#[test]
fn exec_with_F_expands_a_true_condition_from_the_target() {
    let (mut r, rv) = ran(c"if-shell -F '#{session_id}' display-panes run-shell");
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);
        let queued = r.queue.names();
        assert_eq!(queued.len(), 1, "{queued:?}");
        assert!(
            seen(cstr_ptr(&(*(r.queue.start())).name)).starts_with("[display-panes/"),
            "{queued:?}"
        );
        assert_eq!(r.queue.discard(), 1);
    }
}

/// The same expansion against a value that comes out `0` — here the session
/// literally named "0" — takes the else branch.
#[test]
fn exec_with_F_expands_a_false_condition_from_the_target() {
    let (mut r, rv) = ran(c"if-shell -F '#{session_name}' display-panes run-shell");
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);
        let queued = r.queue.names();
        assert_eq!(queued.len(), 1, "{queued:?}");
        assert!(
            seen(cstr_ptr(&(*(r.queue.start())).name)).starts_with("[run-shell/"),
            "{queued:?}"
        );
        assert_eq!(r.queue.discard(), 1);
    }
}

/// A `{ }` block in the command slot reaches exec as ready-made commands and
/// is queued the same way a plain string would be.
#[test]
fn exec_with_F_queues_commands_given_in_braces() {
    let (mut r, rv) = ran(c"if-shell -F '1' { display-panes }");
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);
        let queued = r.queue.names();
        assert_eq!(queued.len(), 1, "{queued:?}");
        assert!(
            seen(cstr_ptr(&(*(r.queue.start())).name)).starts_with("[display-panes/"),
            "{queued:?}"
        );
        assert_eq!(r.queue.discard(), 1);
    }
}

/// A chosen branch that cannot itself be parsed refuses the whole command:
/// error, a complaint in the server's message log and the client's retval
/// set to say so.
#[test]
fn exec_with_F_reports_a_then_command_that_does_not_parse() {
    let (mut r, rv) = ran(c"if-shell -F '1' definitely-not-a-command");
    unsafe {
        assert_eq!(rv, CMD_RETURN_ERROR);
        assert_eq!((*r.c).retval, 1);
        assert!(r.queue.names().is_empty());
        let msgs = logged_messages();
        assert!(
            msgs.iter().any(|m| m.contains("definitely-not-a-command")),
            "{msgs:?}"
        );
    }
}

/// The else slot gets the same treatment when it is the one chosen and bad.
#[test]
fn exec_with_F_reports_an_else_command_that_does_not_parse() {
    let (mut r, rv) = ran(c"if-shell -F '0' display-panes definitely-not-a-command");
    unsafe {
        assert_eq!(rv, CMD_RETURN_ERROR);
        assert_eq!((*r.c).retval, 1);
        assert!(r.queue.names().is_empty());
        let msgs = logged_messages();
        assert!(
            msgs.iter().any(|m| m.contains("definitely-not-a-command")),
            "{msgs:?}"
        );
    }
}

/// A clean exit picks the if-commands even though an else was prepared, and
/// the waiting item is released to let the queue move on.
#[test]
fn the_callback_runs_the_if_commands_on_a_clean_exit() {
    let mut h = unsafe { harnessed(c"if-shell 'x' display-panes run-shell", 0, true, true) };
    unsafe {
        h.fire();

        assert_eq!((*h.item.ptr()).flags & CMDQ_WAITING, 0);
        let queued = h.queue.names();
        assert_eq!(queued.len(), 1, "{queued:?}");
        let qi = h.queue.start();
        assert!(
            seen(cstr_ptr(&(*qi).name)).starts_with("[display-panes/"),
            "{queued:?}"
        );
        assert!(matches!((*qi).type_0, CmdqType::Command { .. }));
        assert_eq!((*qi).state(), (*h.item.ptr()).state());
        assert_eq!(crate::cmd::cmdq_get_client(&*qi), h.c);
        assert_eq!(
            (*h.cdata)
                .item
                .as_ref()
                .and_then(CmdqItemWeak::upgrade)
                .map(|held| held.as_ptr()),
            Some(h.item.ptr())
        );

        assert_eq!(h.queue.discard(), 1);
        let data = ::core::mem::take(&mut h.job.data);
        cmd_if_shell_free(data);
        h.freed = true;
    }
}

/// A non-zero exit code picks the else-commands.
#[test]
fn the_callback_runs_the_else_commands_after_a_nonzero_exit() {
    let mut h = unsafe { harnessed(c"if-shell 'x' display-panes run-shell", 0x100, true, true) };
    unsafe {
        h.fire();

        assert_eq!((*h.item.ptr()).flags & CMDQ_WAITING, 0);
        let queued = h.queue.names();
        assert_eq!(queued.len(), 1, "{queued:?}");
        assert!(
            seen(cstr_ptr(&(*(h.queue.start())).name)).starts_with("[run-shell/"),
            "{queued:?}"
        );

        assert_eq!(h.queue.discard(), 1);
    }
}

/// Death by signal picks the else-commands as well: any set low byte wins
/// over an empty exit-code byte.
#[test]
fn the_callback_runs_the_else_commands_after_a_signal() {
    let mut h = unsafe { harnessed(c"if-shell 'x' display-panes run-shell", 15, true, true) };
    unsafe {
        h.fire();

        let queued = h.queue.names();
        assert_eq!(queued.len(), 1, "{queued:?}");
        assert!(
            seen(cstr_ptr(&(*(h.queue.start())).name)).starts_with("[run-shell/"),
            "{queued:?}"
        );

        assert_eq!(h.queue.discard(), 1);
    }
}

/// Without an else prepared, a failed shell has nothing to run: the item is
/// still continued and the queue stays empty.
#[test]
fn the_callback_without_an_else_leaves_nothing_queued() {
    let mut h = unsafe { harnessed(c"if-shell 'x' display-panes", 0x7f, false, true) };
    unsafe {
        h.fire();

        assert_eq!((*h.item.ptr()).flags & CMDQ_WAITING, 0);
        assert!(h.queue.names().is_empty());
        assert_eq!(h.queue.discard(), 0);
    }
}

/// With no item left to continue — the `-b` case after its caller moved on —
/// the chosen commands land on the client's own queue instead.
#[test]
fn the_callback_appends_to_the_clients_queue_when_headless() {
    let mut h = unsafe { harnessed(c"if-shell 'x' display-panes run-shell", 0, true, false) };
    unsafe {
        h.fire();

        let queued = h.queue.names();
        assert_eq!(queued.len(), 1, "{queued:?}");
        let qi = h.queue.start();
        assert!(
            seen(cstr_ptr(&(*qi).name)).starts_with("[display-panes/"),
            "{queued:?}"
        );
        assert!(matches!((*qi).type_0, CmdqType::Command { .. }));
        assert_eq!(crate::cmd::cmdq_get_client(&*qi), h.c);
        assert_eq!(h.queue.discard(), 1);

        let data = ::core::mem::take(&mut h.job.data);
        cmd_if_shell_free(data);
        h.freed = true;
    }
}

/// Commands that cannot be parsed when there is an item to tell report
/// through `cmdq_error`: the complaint lands in the server's message log and
/// the client's retval says something went wrong.
#[test]
fn the_callback_reports_an_unparseable_command_through_the_item() {
    let mut h = unsafe { harnessed(c"if-shell 'x' definitely-not-a-command", 0, false, true) };
    unsafe {
        h.fire();

        assert_eq!((*h.c).retval, 1);
        assert_eq!((*h.item.ptr()).flags & CMDQ_WAITING, 0);
        assert!(h.queue.names().is_empty());
        let msgs = logged_messages();
        assert!(
            msgs.iter().any(|m| m.contains("definitely-not-a-command")),
            "{msgs:?}"
        );
    }
}

/// The same failure with nobody left to tell becomes a server message, its
/// first letter capitalised for display.
#[test]
fn the_callback_announces_an_unparseable_command_when_nothing_remains() {
    let mut h = unsafe { harnessed(c"if-shell 'x' definitely-not-a-command", 0, false, false) };
    unsafe {
        let _ = (*h.cdata).client_ref.take();

        h.fire();

        let msgs = logged_messages();
        assert!(
            msgs.iter()
                .any(|m| m.starts_with("message: Unknown command: definitely-not-a-command")),
            "{msgs:?}"
        );
        assert!(h.queue.names().is_empty());
    }
}

/// When the client still has a session, the report goes to its message line
/// instead: uppercased, frozen onto the terminal, and leaving the retval
/// alone because the queue will collect it another way.
#[test]
fn the_callback_shows_the_error_on_the_message_line_of_a_session_client() {
    let mut h = unsafe { harnessed(c"if-shell 'x' definitely-not-a-command", 0, false, true) };
    unsafe {
        (*h.c).session = h.target.session();
        (*(*h.c).session).statuslines = 1;
        status_init(h.c);

        h.fire();

        assert!((*h.c).message_string.is_some());
        let shown = seen(cstr_ptr(&(*h.c).message_string));
        assert_eq!(
            shown, "Unknown command: definitely-not-a-command",
            "{shown}"
        );
        assert_eq!(
            (*h.c).tty.flags & (TTY_FREEZE | TTY_NOCURSOR),
            TTY_FREEZE | TTY_NOCURSOR
        );
        assert_eq!((*h.c).retval, 0);
        assert!(h.queue.names().is_empty());

        status_message_clear(h.c);
        assert!((*h.c).message_string.is_none());
    }
}

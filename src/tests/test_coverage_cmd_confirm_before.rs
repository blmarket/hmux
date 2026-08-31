//! Unit tests for [`crate::cmd::cmd_confirm_before`] — the `confirm-before`
//! entry metadata, its constants, and every branch of [`exec`], the answer
//! callback and the free function the fixtures can reach without a terminal.
//!
//! The exec path ends in `status_prompt_set`, which parks the command's
//! private [`cmd_confirm_before_data`] on the client together with two
//! callbacks: the answer handler and the free function the client's
//! [`Prompt`] names. The tests drive both exactly the way the status line
//! would — reading them back off the client and calling them directly — so no
//! visibility has to be widened. A confirmation hands the parsed command list
//! to the command queue: appended onto the target client's own queue with
//! `-b`, or inserted after the still-waiting item without it. Fixtures own
//! those queues here; [`Queue::discard`] takes their items apart again the
//! way `cmdq_remove` would, giving each state a format tree first because
//! freshly created states carry none and `cmdq_free_state` frees one
//! unconditionally.
//!
//! Two pieces of dressing give the bare fixture client what a real one gets
//! at creation: `status_init`, without which the prompt's screen push/pop
//! would free nothing and then free it, and `CLIENT_ATTACHED`, which keeps
//! error reports in the server's message log instead of letting `file_error`
//! try to open a stream to the peer the fixture does not have.

use crate::arguments::args_create;
use crate::cmd::CmdqItemWeak;
use crate::cmd::cmd_confirm_before::{
    ARGS_PARSE_COMMANDS_OR_STRING, CLIENT_DEAD, CMD_CLIENT_TFLAG, CMD_FIND_PANE, CMD_RETURN_ERROR,
    CMD_RETURN_NORMAL, CMD_RETURN_WAIT, PROMPT_ENTRY, PROMPT_SINGLE, PROMPT_TYPE_COMMAND,
    cmd_confirm_before_data, cmd_confirm_before_entry,
};
use crate::cmd::{CMDQ_WAITING, CmdqType, cmdq_new};
use crate::server::message_log;
use crate::server::{CLIENT_REDRAWSTATUS, TTY_FREEZE};
use crate::status::status_prompt_clear;
use crate::tests::test_fixtures::{Clients, Item, Target, ensure_reactor, globals, seen};
use crate::types::*;
use ::core::ffi::{CStr, c_int};
use ::core::ptr::null_mut;
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
        let mut owned = cmdq_new();
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
                .map(|it| seen((*it).name_ptr()))
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

/// The private data exec parks on the client.
unsafe fn data_of(c: *mut client) -> *mut cmd_confirm_before_data {
    unsafe {
        let PromptData::ConfirmBefore(data) = &mut (*c).prompt_data else {
            panic!("the confirm-before data is missing");
        };
        &raw mut **data
    }
}

/// The item the prompt is waiting on, or null when it waits on none.
unsafe fn item_of(d: *mut cmd_confirm_before_data) -> *mut cmdq_item {
    unsafe {
        (*d).item
            .as_ref()
            .and_then(CmdqItemWeak::upgrade)
            .map_or(null_mut(), |item| item.as_ptr())
    }
}

/// Answers the prompt the way the status line would, handing the string
/// straight to the input callback the command installed.
unsafe fn answer(c: *mut client, s: &CStr) -> c_int {
    unsafe { (*c).prompt.input(c, &mut (*c).prompt_data, Some(s), 0) }
}

/// The standard setup: one registered target, one client named "confirm", an
/// item carrying `line`'s parsed arguments aimed at both, and a fixture queue
/// wired underneath the item. Answers with the command's own `.exec`, which
/// installs the prompt on the client.
fn prompted(line: &'static CStr) -> (Prompted, cmd_retval) {
    let _guard = globals();
    ensure_reactor();
    let mut target = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("confirm", 80, 24);
    unsafe {
        crate::status::status_init(c);
        (*c).flags |= crate::cmd::cmd_attach_session::CLIENT_ATTACHED as u64;
    }
    let mut item = Item::new().with_args(line);
    item.set_client(c);
    let mut item = item.targeting(&mut target);
    let mut queue = Queue::new();
    unsafe {
        (*item.ptr()).queue = queue.q;
        let rv = (cmd_confirm_before_entry.exec)(&*item.cmd(), item.ptr());
        (
            Prompted {
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

/// What [`prompted`] hands back, kept together so it comes apart in a safe
/// order: queued commands are taken apart while the client fixtures are still
/// alive, the item goes next, then the registered target, then the clients,
/// and the globals turn is given up last.
struct Prompted {
    queue: Queue,
    item: Item,
    target: Target,
    clients: Clients,
    c: *mut client,
    _guard: MutexGuard<'static, ()>,
}

#[test]
fn entry_metadata_matches_upstream() {
    unsafe {
        let e: *const cmd_entry = &raw const cmd_confirm_before_entry;
        assert_eq!((*e).name.to_bytes(), b"confirm-before");
        assert_eq!(
            (*e).alias.expect("the entry has an alias").to_bytes(),
            b"confirm"
        );
        assert_eq!(
            (*e).usage.to_bytes(),
            b"[-by] [-c confirm-key] [-p prompt] [-t target-client] command"
        );

        assert_eq!((*e).args.template.to_bytes(), b"bc:p:t:y");
        assert_eq!((*e).args.lower, 1);
        assert_eq!((*e).args.upper, 1);

        for flag in [&raw const (*e).source, &raw const (*e).target] {
            assert_eq!((*flag).flag, 0);
            assert_eq!((*flag).type_0, CMD_FIND_PANE);
            assert_eq!((*flag).flags, 0);
        }

        assert_eq!((*e).flags, CMD_CLIENT_TFLAG);
    }
}

/// The entry's argument callback answers for every argument the same way,
/// whatever it is handed: strings and `{ }` blocks alike are accepted, which
/// is what lets `confirm-before` take both.
#[test]
fn the_arguments_callback_accepts_commands_or_strings() {
    unsafe {
        let mut cause = None;
        assert_eq!(
            cmd_confirm_before_entry.args.cb.unwrap()(&args_create(), 0, &mut cause),
            ARGS_PARSE_COMMANDS_OR_STRING
        );
    }
}

#[test]
fn constants_used_by_the_confirm_paths_match_upstream() {
    assert_eq!(ARGS_PARSE_COMMANDS_OR_STRING, 2);
    assert_eq!(CMD_FIND_PANE, 0);

    assert_eq!(CMD_RETURN_NORMAL, 0);
    assert_eq!(CMD_RETURN_WAIT, 1);
    assert_eq!(CMD_RETURN_ERROR, -1);

    assert_eq!(PROMPT_SINGLE, 0x1);
    assert_eq!(CMD_CLIENT_TFLAG, 0x10);
    assert_eq!(CLIENT_DEAD, 0x200);

    assert_eq!(PROMPT_TYPE_COMMAND, 0);
    assert_eq!(PROMPT_ENTRY, 0);
}

/// The plain path: the prompt is installed and the command answers wait. A
/// wrong key declines it — leaving the waiting client's retval at 1 — and so
/// does Enter, because the prompt was not made default-yes; the right key
/// queues the confirmed command right after the waiting item and answers 0.
#[test]
fn exec_prompts_and_waits_for_a_confirmation() {
    let (mut p, rv) = prompted(c"confirm-before -p \"really?\" kill-session");
    unsafe {
        assert_eq!(rv, CMD_RETURN_WAIT);

        assert_eq!(seen((*p.c).prompt_string_ptr()), "really? ");
        assert_eq!((*p.c).prompt_flags & PROMPT_SINGLE, PROMPT_SINGLE);
        assert_eq!((*p.c).prompt_type, PROMPT_TYPE_COMMAND);
        assert_eq!((*p.c).prompt_mode, PROMPT_ENTRY);
        assert_eq!((*p.c).prompt, Prompt::ConfirmBefore);

        let d = data_of(p.c);
        assert_eq!(item_of(d), p.item.ptr());
        assert_eq!((*d).default_yes, 0);
        assert_eq!((*d).confirm_key, b'y');
        assert!((*d).cmdlist.is_some());

        assert_eq!((*p.c).tty.flags & TTY_FREEZE, TTY_FREEZE);
        assert_eq!(
            (*p.c).flags & CLIENT_REDRAWSTATUS as u64,
            CLIENT_REDRAWSTATUS as u64
        );
        p.queue.wire(&mut p.item);

        assert_eq!(answer(p.c, c"n"), 0);
        assert_eq!((*p.c).retval, 1);
        assert!(p.queue.names().is_empty());

        assert_eq!(answer(p.c, c"\r"), 0);
        assert_eq!((*p.c).retval, 1);
        assert!(p.queue.names().is_empty());

        assert_eq!(answer(p.c, c"y"), 0);
        assert_eq!((*p.c).retval, 0);
        let queued = p.queue.names();
        assert_eq!(queued.len(), 1, "{queued:?}");
        let qi = p.queue.start();
        let CmdqType::Command { cmdlist, .. } = &(*qi).type_0 else {
            panic!("the command was not queued");
        };
        assert_eq!(*cmdlist, (*d).cmdlist);
        assert_eq!((*qi).state(), (*p.item.ptr()).state());
        assert_eq!(crate::cmd::cmdq_get_client(&*qi), p.c);
        assert_eq!((*p.item.ptr()).flags & CMDQ_WAITING, 0);
        assert_eq!(p.queue.discard(), 1);

        status_prompt_clear(p.c);
        assert!((*p.c).prompt_string.is_none());
        assert_eq!((*p.c).tty.flags & TTY_FREEZE, 0);
    }
}

/// Without `-p` the prompt names the first command of the confirmed list and
/// the key it waits for.
#[test]
fn exec_builds_the_prompt_from_the_confirmed_command() {
    let (mut p, rv) = prompted(c"confirm-before kill-session");
    unsafe {
        assert_eq!(rv, CMD_RETURN_WAIT);
        assert_eq!(
            seen((*p.c).prompt_string_ptr()),
            "Confirm 'kill-session'? (y/n) "
        );
        status_prompt_clear(p.c);
    }
}

/// With `-b` nothing waits: the command answers normal at once, the data
/// carries no item to continue, and a later confirmation appends the command
/// straight onto the client's own queue. Declining appends nothing and leaves
/// the client alone.
#[test]
fn exec_with_b_answers_normal_and_queues_on_confirmation() {
    let (mut p, rv) = prompted(c"confirm-before -b kill-session");
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert!(!data_of(p.c).is_null());
        assert_eq!(item_of(data_of(p.c)), null_mut());

        p.queue.attach(p.c);

        assert_eq!(answer(p.c, c"n"), 0);
        assert!(p.queue.names().is_empty());
        assert_eq!((*p.c).retval, 0);

        assert_eq!(answer(p.c, c"y"), 0);
        assert_eq!(item_of(data_of(p.c)), null_mut());
        let queued = p.queue.names();
        assert_eq!(queued.len(), 1, "{queued:?}");
        let qi = p.queue.first();
        assert_eq!(crate::cmd::cmdq_get_client(&*qi), p.c);
        let CmdqType::Command { cmdlist, .. } = &(*qi).type_0 else {
            panic!("the command was not queued");
        };
        assert_eq!(*cmdlist, (*data_of(p.c)).cmdlist);
        assert_eq!(p.queue.discard(), 1);

        status_prompt_clear(p.c);
    }
}

/// A custom key replaces the default `y` everywhere: it shows in the prompt,
/// it confirms, and the old key stops working.
#[test]
fn exec_honours_a_custom_confirm_key() {
    let (mut p, rv) = prompted(c"confirm-before -b -c z kill-session");
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!((*data_of(p.c)).confirm_key, b'z');
        assert_eq!(
            seen((*p.c).prompt_string_ptr()),
            "Confirm 'kill-session'? (z/n) "
        );

        p.queue.attach(p.c);

        assert_eq!(answer(p.c, c"y"), 0);
        assert!(p.queue.names().is_empty());

        assert_eq!(answer(p.c, c"z"), 0);
        assert_eq!(p.queue.names().len(), 1);

        assert_eq!(p.queue.discard(), 1);
        status_prompt_clear(p.c);
    }
}

/// `-y` makes Enter mean yes.
#[test]
fn enter_accepts_when_default_yes() {
    let (mut p, rv) = prompted(c"confirm-before -b -y kill-session");
    unsafe {
        assert_eq!(rv, CMD_RETURN_NORMAL);
        assert_eq!((*data_of(p.c)).default_yes, 1);

        p.queue.attach(p.c);

        assert_eq!(answer(p.c, c"\r"), 0);
        assert_eq!(p.queue.names().len(), 1);

        assert_eq!(p.queue.discard(), 1);
        status_prompt_clear(p.c);
    }
}

/// A confirm key that is not one printable character is refused outright:
/// error, a complaint in the server's message log, the client marked with
/// retval 1, and no prompt installed at all.
#[test]
fn exec_rejects_an_unusable_confirm_key() {
    let (mut p, rv) = prompted(c"confirm-before -c ab kill-session");
    unsafe {
        assert_eq!(rv, CMD_RETURN_ERROR);
        assert_eq!((*p.c).retval, 1);
        assert!((*p.c).prompt_string.is_none());
        assert!((*p.c).prompt.is_none());
        let msgs = logged_messages();
        assert!(
            msgs.iter()
                .any(|m| m.contains("invalid confirm key") && m.contains("confirm")),
            "{msgs:?}"
        );
    }
}

/// A confirmed command that cannot itself be parsed is refused the same way,
/// with the parser's complaint passed on through `cmdq_error`.
#[test]
fn exec_reports_a_command_that_does_not_parse() {
    let (mut p, rv) = prompted(c"confirm-before definitely-not-a-command");
    unsafe {
        assert_eq!(rv, CMD_RETURN_ERROR);
        assert_eq!((*p.c).retval, 1);
        assert!((*p.c).prompt_string.is_none());
        let msgs = logged_messages();
        assert!(
            msgs.iter().any(|m| m.contains("definitely-not-a-command")),
            "{msgs:?}"
        );
    }
}

/// A dead client is not asked anything: even the right key leaves the command
/// unqueued, though the waiting item behind the prompt is still released.
#[test]
fn a_dead_client_is_not_asked() {
    let (mut p, rv) = prompted(c"confirm-before kill-session");
    unsafe {
        assert_eq!(rv, CMD_RETURN_WAIT);
        (*p.c).flags |= CLIENT_DEAD as u64;

        assert_eq!(answer(p.c, c"y"), 0);
        assert!(p.queue.names().is_empty());
        assert_eq!((*p.c).retval, 1);
        assert_eq!((*p.item.ptr()).flags & CMDQ_WAITING, 0);

        status_prompt_clear(p.c);
    }
}

/// An absent answer — what a cancelled prompt reports — declines quietly.
#[test]
fn an_absent_answer_declines() {
    let (mut p, rv) = prompted(c"confirm-before kill-session");
    unsafe {
        assert_eq!(rv, CMD_RETURN_WAIT);
        (*p.c).prompt.input(p.c, &mut (*p.c).prompt_data, None, 0);
        assert!(p.queue.names().is_empty());
        assert_eq!((*p.c).retval, 1);
        status_prompt_clear(p.c);
    }
}

/// A client still attached to a session gets its retval from somewhere else,
/// so the answer handler leaves it alone even while continuing the item.
#[test]
fn a_clients_retval_is_left_alone_when_it_has_a_session() {
    let (mut p, rv) = prompted(c"confirm-before kill-session");
    unsafe {
        assert_eq!(rv, CMD_RETURN_WAIT);
        (*p.c).session = p.target.session();
        (*p.c).retval = 5;

        assert_eq!(answer(p.c, c"n"), 0);
        assert_eq!((*p.c).retval, 5);
        assert_eq!((*p.item.ptr()).flags & CMDQ_WAITING, 0);

        status_prompt_clear(p.c);
    }
}

/// The first item queued behind the wired one, if any.
impl Queue {
    fn first(&self) -> *mut cmdq_item {
        self.start()
    }
}

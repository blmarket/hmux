//! Unit tests for [`crate::cmd::cmd_command_prompt`] — the `command-prompt`
//! command's entry metadata, its constants, and the paths of
//! [`cmd_command_prompt_exec`] and [`cmd_command_prompt_callback`] that the
//! fixtures can reach without a terminal.
//!
//! The command's own functions are private, so every exec goes through
//! [`cmd_command_prompt_entry`]'s `.exec` slot — the very pointer the command
//! queue would call — and the callback through the input callback that
//! `status_prompt_set` installs on the client. Both need a client, because
//! the exec dereferences the target client before anything else. The prompt
//! state it leaves behind is taken down again with [`status_prompt_clear`],
//! which hands the command's data to its own free callback; what that frees
//! can be inspected first by casting the client's prompt data back to
//! [`cmd_command_prompt_cdata`], which is public.
//!
//! The callback builds commands and hands them to the queue: with an item it
//! inserts after that item, without one (`-b`, `-i`) it appends to the
//! client's queue. A fixture client or item carries no queue pointer, and both
//! `cmdq_append` and `cmdq_insert_after` would follow it, so these tests hand
//! out a private [`Queue`]. When it goes away each item still on it is freed
//! exactly the way `cmdq_remove` would: its client reference, command-list
//! reference and state reference are given back before the name and body.
//! Callback data is dropped too, including an error item's owned message.
//! Because a queued item holds a reference on the fixture client,
//! the client's reference count is only asserted once every queue has been
//! dropped. A fixture item whose state gets linked into a queued command has
//! that state's reference count raised to one first, so that releasing the
//! item's reference stops short of freeing memory the test's `Item` owns.
//!
//! The callback's early exit when something else has taken the client's input
//! callback over is reached by putting a stand-in in its place and calling the
//! saved callback by hand, which is the same state a prompt opened by the
//! command being built would have left. What stays uncovered here is the
//! redraw a live server would do behind the client flags these paths set.

use crate::arguments::args_create;
use crate::cmd::CmdqItemWeak;
use crate::cmd::cmd_attach_session::CLIENT_ATTACHED;
use crate::cmd::cmd_command_prompt::{
    ARGS_PARSE_COMMANDS, ARGS_PARSE_COMMANDS_OR_STRING, ARGS_PARSE_INVALID, ARGS_PARSE_STRING,
    CLIENT_EXIT_DETACH, CLIENT_EXIT_RETURN, CLIENT_EXIT_SHUTDOWN, CMD_CLIENT_TFLAG, CMD_FIND_PANE,
    CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_STOP,
    CMD_RETURN_WAIT, LAYOUT_LEFTRIGHT, LAYOUT_TOPBOTTOM, LAYOUT_WINDOWPANE, MSG_COMMAND,
    MSG_IDENTIFY_FLAGS, MSG_READ, MSG_READ_CANCEL, MSG_READY, MSG_VERSION, MSG_WRITE_CLOSE,
    PANE_LINES_DOUBLE, PANE_LINES_HEAVY, PANE_LINES_NUMBER, PANE_LINES_SIMPLE, PANE_LINES_SINGLE,
    PANE_LINES_SPACES, PROGRESS_BAR_ERROR, PROGRESS_BAR_HIDDEN, PROGRESS_BAR_INDETERMINATE,
    PROGRESS_BAR_NORMAL, PROGRESS_BAR_PAUSED, PROMPT_BSPACE_EXIT, PROMPT_COMMAND, PROMPT_ENTRY,
    PROMPT_INCREMENTAL, PROMPT_KEY, PROMPT_NOFREEZE, PROMPT_NUMERIC, PROMPT_SINGLE,
    PROMPT_TYPE_COMMAND, PROMPT_TYPE_INVALID, PROMPT_TYPE_SEARCH, PROMPT_TYPE_TARGET,
    PROMPT_TYPE_WINDOW_TARGET, SCREEN_CURSOR_BAR, SCREEN_CURSOR_BLOCK, SCREEN_CURSOR_DEFAULT,
    SCREEN_CURSOR_UNDERLINE, STYLE_ALIGN_ABSOLUTE_CENTRE, STYLE_ALIGN_CENTRE, STYLE_ALIGN_DEFAULT,
    STYLE_ALIGN_LEFT, STYLE_ALIGN_RIGHT, STYLE_DEFAULT_BASE, STYLE_DEFAULT_POP, STYLE_DEFAULT_PUSH,
    STYLE_DEFAULT_SET, STYLE_LIST_FOCUS, STYLE_LIST_LEFT_MARKER, STYLE_LIST_OFF, STYLE_LIST_ON,
    STYLE_LIST_RIGHT_MARKER, STYLE_RANGE_CONTROL, STYLE_RANGE_LEFT, STYLE_RANGE_NONE,
    STYLE_RANGE_PANE, STYLE_RANGE_RIGHT, STYLE_RANGE_SESSION, STYLE_RANGE_USER, STYLE_RANGE_WINDOW,
    THEME_DARK, THEME_LIGHT, THEME_UNKNOWN, cmd_command_prompt_cdata, cmd_command_prompt_entry,
};
use crate::cmd::{CMDQ_WAITING, CmdqType, cmdq_free, cmdq_new};
use crate::server::message_log;
use crate::status::{status_prompt_clear, status_prompt_type};
use crate::tests::test_fixtures::{Clients, Item, Target, ensure_reactor, globals, seen};
use crate::text::utf8_vec_strlen;
use crate::tty::{CLIENT_REDRAWSTATUS, TTY_FREEZE};
use crate::types::*;
use ::core::ffi::CStr;
use ::std::ffi::CString;

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

/// A command queue of the test's own, handed to a fixture client or item so
/// that the callback has somewhere to put the commands it builds. Nothing runs
/// the queue, so when it goes away each item left on it is freed here the way
/// `cmdq_remove` does: its client reference, command-list reference and state
/// reference are given back before the name and body, then callback data is
/// dropped with the rest of the item.
///
/// An item [`Queue::place`]d on the queue is different: a fixture item the
/// command inserts its work after has to be *on* the queue for
/// `cmdq_insert_after` to link against, but the test's own [`Item`] keeps
/// owning it, so at drop time it is unlinked and never freed.
struct Queue {
    q: *mut cmdq_list,
    owned: Option<Box<cmdq_list>>,
}

impl Queue {
    unsafe fn new() -> Queue {
        let mut owned = cmdq_new();
        let q = &raw mut *owned;
        Queue {
            q,
            owned: Some(owned),
        }
    }

    /// The same queue, but the client's own, freed when the client goes.
    unsafe fn for_client(c: *mut client) -> Queue {
        let q = unsafe { &raw mut **(*c).queue.insert(cmdq_new()) };
        Queue { q, owned: None }
    }

    fn ptr(&self) -> *mut cmdq_list {
        self.q
    }

    /// How many items are waiting on it.
    unsafe fn len(&self) -> usize {
        unsafe { (*self.q).list.len() }
    }

    /// Puts `item` at the tail of the queue the way `cmdq_append` would, but
    /// without taking a client reference, so the counts stay as the command
    /// under test leaves them.
    unsafe fn place(&mut self, item: &mut Item) {
        unsafe { item.queue_onto(&mut *self.q) };
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        unsafe {
            (*self.q).list.clear();
            if let Some(owned) = self.owned.take() {
                cmdq_free(owned);
            }
        }
    }
}

/// The command's per-prompt state behind a live prompt. It belongs to the
/// free callback, but stays readable until the prompt is cleared.
unsafe fn cdata_of<'a>(c: *mut client) -> &'a cmd_command_prompt_cdata {
    unsafe {
        let PromptData::CommandPrompt(data) = &(*c).prompt_data else {
            panic!("the client is not prompting");
        };
        data
    }
}

/// Runs the entry's exec against `args` with `c` as the item's client and, by
/// way of the target state, its target client. The item comes back because
/// the command keeps hold of it for as long as it waits.
///
/// A fresh fixture client has a zeroed status line, where upstream points the
/// active screen at the client's own embedded screen from `status_init`
/// onwards — the invariant [`status_prompt_set`] pushes and pops against — so
/// it is restored here before the command runs. The embedded screen itself is
/// never freed by these paths, so it can stay as it is.
unsafe fn run_exec(t: &mut Target, c: *mut client, args: &CStr) -> (Item, cmd_retval) {
    unsafe {
        ensure_reactor();
        (*c).status.active = crate::types::StatusActive::Own;
        let mut item = Item::new().with_args(args);
        item.set_client(c);
        let mut item = item.targeting(t);
        let rv = (cmd_command_prompt_entry.exec)(&*item.cmd(), item.ptr());
        (item, rv)
    }
}

#[test]
fn entry_metadata_matches_upstream() {
    unsafe {
        let e: *const cmd_entry = &raw const cmd_command_prompt_entry;
        assert_eq!((*e).name.to_bytes(), b"command-prompt");
        assert!((*e).alias.is_none());
        assert_eq!(
            (*e).usage.to_bytes(),
            b"[-1CbeFiklN] [-I inputs] [-p prompts] [-t target-client] [-T prompt-type] [template]"
        );

        assert_eq!((*e).args.template.to_bytes(), b"1CbeFiklI:Np:t:T:");
        assert_eq!((*e).args.lower, 0);
        assert_eq!((*e).args.upper, 1);
        assert!((*e).args.cb.is_some());

        for flag in [&raw const (*e).source, &raw const (*e).target] {
            assert_eq!((*flag).flag, 0);
            assert_eq!((*flag).type_0, CMD_FIND_PANE);
            assert_eq!((*flag).flags, 0);
        }

        assert_eq!((*e).flags, CMD_CLIENT_TFLAG);
    }
}

#[test]
fn constants_declared_by_the_subject_match_upstream() {
    assert_eq!(MSG_VERSION, 12);
    assert_eq!(MSG_IDENTIFY_FLAGS, 100);
    assert_eq!(MSG_COMMAND, 200);
    assert_eq!(MSG_READY, 207);
    assert_eq!(MSG_READ, 301);
    assert_eq!(MSG_WRITE_CLOSE, 306);
    assert_eq!(MSG_READ_CANCEL, 307);

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

    assert_eq!(THEME_UNKNOWN, 0);
    assert_eq!(THEME_LIGHT, 1);
    assert_eq!(THEME_DARK, 2);

    assert_eq!(LAYOUT_LEFTRIGHT, 0);
    assert_eq!(LAYOUT_TOPBOTTOM, 1);
    assert_eq!(LAYOUT_WINDOWPANE, 2);

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

    assert_eq!(CMD_RETURN_ERROR, -1);
    assert_eq!(CMD_RETURN_NORMAL, 0);
    assert_eq!(CMD_RETURN_WAIT, 1);
    assert_eq!(CMD_RETURN_STOP, 2);

    assert_eq!(PROMPT_SINGLE, 0x1);
    assert_eq!(PROMPT_NUMERIC, 0x2);
    assert_eq!(PROMPT_INCREMENTAL, 0x4);
    assert_eq!(PROMPT_KEY, 0x10);
    assert_eq!(PROMPT_BSPACE_EXIT, 0x80);
    assert_eq!(PROMPT_NOFREEZE, 0x100);
    assert_eq!(CMD_CLIENT_TFLAG, 0x10);
}

/// The entry's args-parse hook ignores everything it is handed and always asks
/// for the last argument to be taken as commands when they parse, falling back
/// to a plain string — so null placeholders stand in for real ones here.
#[test]
fn args_parse_callback_always_answers_commands_or_string() {
    unsafe {
        let e: *const cmd_entry = &raw const cmd_command_prompt_entry;
        let mut cause = None;
        let rv = (*e).args.cb.unwrap()(&args_create(), 0, &mut cause);
        assert_eq!(rv, ARGS_PARSE_COMMANDS_OR_STRING);
    }
}

/// A client already answering a prompt is left alone: the command answers
/// normal at once, before any of its own state exists.
#[test]
fn exec_when_the_client_is_already_prompting_answers_normal_at_once() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("busy", 80, 24);
    unsafe {
        (*c).prompt_string = Some(c"existing".to_owned());
        let (_item, rv) = run_exec(&mut t, c, c"command-prompt");
        assert_eq!(rv, CMD_RETURN_NORMAL);

        assert_eq!(seen((*c).prompt_string_ptr()), "existing");
        assert!((*c).prompt.is_none());
        assert_eq!((*c).prompt_data, PromptData::None);

        (*c).prompt_string = None;
    }
}

/// A bare `command-prompt` puts a single `:` prompt holding the empty input up
/// on the client, marks it frozen, and holds the calling item while it waits.
#[test]
fn exec_without_arguments_prompts_for_a_command_and_waits() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("asker", 80, 24);
    unsafe {
        let (mut item, rv) = run_exec(&mut t, c, c"command-prompt");
        assert_eq!(rv, CMD_RETURN_WAIT);

        assert_eq!(seen((*c).prompt_string_ptr()), ":");
        assert_eq!((*c).prompt_index, 0);
        assert_eq!(utf8_vec_strlen(&(*c).prompt_buffer), 0);
        assert_eq!((*c).prompt, Prompt::CommandPrompt);
        let PromptData::CommandPrompt(_) = &(*c).prompt_data else {
            panic!("the command prompt data is missing");
        };
        assert_eq!((*c).prompt_flags, 0);
        assert_eq!((*c).prompt_type, PROMPT_TYPE_COMMAND);
        assert_eq!((*c).prompt_mode, PROMPT_ENTRY);
        assert_ne!((*c).tty.flags & TTY_FREEZE, 0);
        assert_ne!((*c).flags & CLIENT_REDRAWSTATUS as u64, 0);

        let cd = cdata_of(c);
        assert_eq!(
            cd.item
                .as_ref()
                .and_then(CmdqItemWeak::upgrade)
                .map(|held| held.as_ptr()),
            Some(item.ptr())
        );
        assert_eq!(cd.prompts.len(), 1);
        assert_eq!(cd.current, 0);
        assert_eq!(cd.argv.len(), 0);
        assert!(cd.argv.is_empty());
        assert_eq!(cd.flags, 0);
        assert_eq!(cd.prompt_type, PROMPT_TYPE_COMMAND);
        assert_eq!(seen(cstr_ptr(&cd.prompts[0].prompt)), ":");
        assert_eq!(seen(cstr_ptr(&cd.prompts[0].input)), "");

        status_prompt_clear(c);
        assert!((*c).prompt_string.is_none());
    }
}

/// Dismissing the prompt with no answer releases the held item without
/// building anything: no answer is recorded and the wait flag goes away.
#[test]
fn the_callback_dismissed_with_a_null_answer_continues_the_item() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("quitter", 80, 24);
    unsafe {
        let (mut item, rv) = run_exec(&mut t, c, c"command-prompt");
        assert_eq!(rv, CMD_RETURN_WAIT);
        item.set_flags(CMDQ_WAITING);

        let cd = cdata_of(c);
        let prompt = (*c).prompt;
        let rc = prompt.input(c, &mut (*c).prompt_data, None, 1);
        assert_eq!(rc, 0);

        assert_eq!(cd.argv.len(), 0);
        assert_eq!(cd.current, 0);
        assert_eq!((*item.ptr()).flags & CMDQ_WAITING, 0);

        status_prompt_clear(c);
    }
}

/// Answering the prompt runs the template against the answer: the built
/// command is inserted after the held item on a private queue, the answer is
/// kept in the command's own argv for any later prompts, and the item is let
/// go.
#[test]
fn the_callback_builds_the_template_command_from_the_answer() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("answerer", 80, 24);
    unsafe {
        let (mut item, rv) = run_exec(&mut t, c, c"command-prompt");
        assert_eq!(rv, CMD_RETURN_WAIT);
        item.set_flags(CMDQ_WAITING);

        let mut queue = Queue::new();
        queue.place(&mut item);

        let cd = cdata_of(c);
        let prompt = (*c).prompt;
        let rc = prompt.input(c, &mut (*c).prompt_data, Some(c"display-panes"), 1);
        assert_eq!(rc, 0);

        assert_eq!(cd.argv.len(), 1);
        assert_eq!(seen(cd.argv[0].as_ptr()), "display-panes");
        assert_eq!(cd.current, 1);
        assert_eq!((*item.ptr()).flags & CMDQ_WAITING, 0);

        assert_eq!(queue.len(), 2);
        assert_eq!((*queue.ptr()).list[0].as_ptr(), item.ptr());
        assert!(matches!(
            (*queue.ptr()).list[1].item().type_0,
            CmdqType::Command {
                cmdlist: Some(_),
                ..
            }
        ));

        status_prompt_clear(c);
        drop(queue);
    }
}

/// With a template argument and no `-p`, the prompt shows the template in
/// brackets so the user knows what will be run.
#[test]
fn an_argument_becomes_the_prompt_hint() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("hinter", 80, 24);
    unsafe {
        let (_item, rv) = run_exec(&mut t, c, c"command-prompt refresh-client");
        assert_eq!(rv, CMD_RETURN_WAIT);

        assert_eq!(seen((*c).prompt_string_ptr()), "(refresh-client) ");
        let cd = cdata_of(c);
        assert_eq!(cd.prompts.len(), 1);
        assert_eq!(seen(cstr_ptr(&cd.prompts[0].prompt)), "(refresh-client) ");

        status_prompt_clear(c);
    }
}

/// `-p` splits its value into one prompt per comma, answered in order: each
/// finished answer is appended to the command's argv, the next prompt replaces
/// the client's prompt string while the held item keeps waiting, and only the
/// last answer lets the item go.
#[test]
fn prompts_are_taken_one_at_a_time_until_all_are_answered() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("stepper", 80, 24);
    unsafe {
        let (mut item, rv) = run_exec(
            &mut t,
            c,
            c"command-prompt -p 'first,second' display-message",
        );
        assert_eq!(rv, CMD_RETURN_WAIT);
        item.set_flags(CMDQ_WAITING);

        let mut queue = Queue::new();
        queue.place(&mut item);

        let cd = cdata_of(c);
        assert_eq!(cd.prompts.len(), 2);
        assert_eq!(seen(cstr_ptr(&cd.prompts[0].prompt)), "first ");
        assert_eq!(seen(cstr_ptr(&cd.prompts[1].prompt)), "second ");
        assert_eq!(seen((*c).prompt_string_ptr()), "first ");

        let prompt = (*c).prompt;
        let rc = prompt.input(c, &mut (*c).prompt_data, Some(c"one"), 1);
        assert_eq!(rc, 1, "there is a second prompt to go");

        assert_eq!(cd.current, 1);
        assert_eq!(cd.argv.len(), 1);
        assert_eq!(seen(cd.argv[0].as_ptr()), "one");
        assert_eq!(seen((*c).prompt_string_ptr()), "second ");
        assert_eq!((*item.ptr()).flags & CMDQ_WAITING, CMDQ_WAITING);

        let rc = prompt.input(c, &mut (*c).prompt_data, Some(c"two"), 1);
        assert_eq!(rc, 0);

        assert_eq!(cd.current, 2);
        assert_eq!(cd.argv.len(), 2);
        assert_eq!(seen(cd.argv[0].as_ptr()), "one");
        assert_eq!(seen(cd.argv[1].as_ptr()), "two");
        assert_eq!((*item.ptr()).flags & CMDQ_WAITING, 0);

        assert_eq!(queue.len(), 2);
        let queued = (*queue.ptr()).list[1].as_ptr();
        assert!(matches!((*queued).type_0, CmdqType::Command { .. }));

        status_prompt_clear(c);
        drop(queue);
    }
}

/// `-l` takes `-p` and `-I` whole instead of splitting them: a single prompt
/// carrying the text verbatim, with the whole input pre-filled.
#[test]
fn l_keeps_whole_prompts_and_inputs() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("literaller", 80, 24);
    unsafe {
        let (_item, rv) = run_exec(&mut t, c, c"command-prompt -l -p 'name: ' -I prefix");
        assert_eq!(rv, CMD_RETURN_WAIT);

        assert_eq!(seen((*c).prompt_string_ptr()), "name: ");
        let cd = cdata_of(c);
        assert_eq!(cd.prompts.len(), 1);
        assert_eq!(seen(cstr_ptr(&cd.prompts[0].prompt)), "name: ");
        assert_eq!(seen(cstr_ptr(&cd.prompts[0].input)), "prefix");

        status_prompt_clear(c);
    }
}

/// `-T` names the kind of prompt to draw; a known one is carried into both the
/// client and the command's own state.
#[test]
fn T_selects_the_prompt_type() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("typer", 80, 24);
    unsafe {
        let (_item, rv) = run_exec(&mut t, c, c"command-prompt -T search -p x");
        assert_eq!(rv, CMD_RETURN_WAIT);

        assert_eq!((*c).prompt_type, PROMPT_TYPE_SEARCH);
        let cd = cdata_of(c);
        assert_eq!(cd.prompt_type, PROMPT_TYPE_SEARCH);
        assert_eq!(status_prompt_type(c"search"), PROMPT_TYPE_SEARCH);

        status_prompt_clear(c);
    }
}

/// An unknown `-T` value refuses the command before any prompt is drawn,
/// reports the offending word to the server's message log, flags the failure
/// on the client, and frees its own half-built state.
#[test]
fn exec_reports_an_unknown_prompt_type_and_frees_its_state() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("mistyper", 80, 24);
    unsafe {
        (*c).flags |= CLIENT_ATTACHED as u64;
        let (_item, rv) = run_exec(&mut t, c, c"command-prompt -T bogus -p x");
        assert_eq!(rv, CMD_RETURN_ERROR);

        assert_eq!((*c).retval, 1);
        assert!((*c).prompt_string.is_none());
        assert!((*c).prompt.is_none());
        assert_eq!((*c).prompt_data, PromptData::None);

        let msgs = logged_messages();
        assert!(
            msgs.iter()
                .any(|m| m.contains("mistyper message: unknown type: bogus")),
            "{msgs:?}"
        );
    }
}

/// Every other mode flag reaches both the client and the command's own state
/// unchanged, and none of them stops the command from waiting.
#[test]
fn mode_flags_reach_the_client_unchanged() {
    for (flag_args, bit) in [
        ("-1", PROMPT_SINGLE),
        ("-N", PROMPT_NUMERIC),
        ("-k", PROMPT_KEY),
        ("-e", PROMPT_BSPACE_EXIT),
        ("-C", PROMPT_NOFREEZE),
    ] {
        let _guard = globals();
        let mut t = Target::new(80, 24);
        let mut clients = Clients::new();
        let c = clients.add("modefan", 80, 24);
        unsafe {
            let args = CString::new(format!("command-prompt {flag_args} -p x")).expect("no NUL");
            let (_item, rv) = run_exec(&mut t, c, args.as_c_str());
            assert_eq!(rv, CMD_RETURN_WAIT, "{flag_args}");

            let cd = cdata_of(c);
            assert_ne!(cd.flags & bit, 0, "{flag_args}");
            assert_ne!((*c).prompt_flags & bit, 0, "{flag_args}");

            status_prompt_clear(c);
        }
    }
}

/// A stand-in for the client's prompt: whatever some other prompt, opened
/// while this one was still up, would have left in its place.
const ANOTHER_PROMPT: Prompt = Prompt::ConfirmBefore;

/// `-I` is split on commas the same way `-p` is, one input per prompt in
/// order; a prompt with no input left over gets the empty string, and so does
/// every prompt after it.
#[test]
fn I_hands_each_prompt_its_own_input_and_runs_out_quietly() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("filler", 80, 24);
    unsafe {
        let (_item, rv) = run_exec(
            &mut t,
            c,
            c"command-prompt -p 'a,b,c' -I 'x,y' display-message",
        );
        assert_eq!(rv, CMD_RETURN_WAIT);

        let cd = cdata_of(c);
        assert_eq!(cd.prompts.len(), 3);
        assert_eq!(seen(cstr_ptr(&cd.prompts[0].prompt)), "a ");
        assert_eq!(seen(cstr_ptr(&cd.prompts[0].input)), "x");
        assert_eq!(seen(cstr_ptr(&cd.prompts[1].prompt)), "b ");
        assert_eq!(seen(cstr_ptr(&cd.prompts[1].input)), "y");
        assert_eq!(seen(cstr_ptr(&cd.prompts[2].prompt)), "c ");
        assert_eq!(
            seen(cstr_ptr(&cd.prompts[2].input)),
            "",
            "the third prompt had no input of its own"
        );
        assert_eq!(seen((*c).prompt_string_ptr()), "a ");

        status_prompt_clear(c);
    }
}

/// An incremental prompt that is finished rather than typed into builds
/// nothing at all: the answer is dropped, no command is queued, and the
/// callback reports that it is done.
#[test]
fn an_incremental_prompt_being_finished_builds_nothing() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("finisher", 80, 24);
    unsafe {
        let mut queue = Queue::for_client(c);

        let (_item, rv) = run_exec(&mut t, c, c"command-prompt -i");
        assert_eq!(rv, CMD_RETURN_NORMAL);
        let queued_while_typing = queue.len();

        let cd = cdata_of(c);
        let prompt = (*c).prompt;
        let rc = prompt.input(c, &mut (*c).prompt_data, Some(c"display-panes"), 1);
        assert_eq!(rc, 0);

        assert_eq!(cd.argv.len(), 0, "the finishing answer was not recorded");
        assert!(cd.argv.is_empty());
        assert_eq!(cd.current, 0);
        assert_eq!(
            queue.len(),
            queued_while_typing,
            "finishing an incremental prompt queued something"
        );

        status_prompt_clear(c);
        drop(queue);
    }
}

/// `-b` does not hold the calling item, so the command the answer builds has
/// no item to be inserted after: it is appended to the client's own queue with
/// a state of its own.
#[test]
fn b_appends_the_built_command_to_the_clients_queue() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("backgrounder", 80, 24);
    unsafe {
        let mut queue = Queue::for_client(c);

        let (_item, rv) = run_exec(&mut t, c, c"command-prompt -b");
        assert_eq!(rv, CMD_RETURN_NORMAL, "-b never waits");

        let cd = cdata_of(c);
        assert!(cd.item.is_none(), "-b never holds the item");

        let prompt = (*c).prompt;
        let rc = prompt.input(c, &mut (*c).prompt_data, Some(c"display-panes"), 1);
        assert_eq!(rc, 0);

        assert_eq!(cd.argv.len(), 1);
        assert_eq!(seen(cd.argv[0].as_ptr()), "display-panes");
        assert_eq!(queue.len(), 1);
        let queued = (*queue.ptr()).list[0].as_ptr();
        assert!(matches!(
            (*queued).type_0,
            CmdqType::Command {
                cmdlist: Some(_),
                ..
            }
        ));

        status_prompt_clear(c);
        drop(queue);
    }
}

/// If something else took the client's prompt over while this prompt's answer
/// was being turned into a command — which is what happens when that command
/// opens a prompt of its own — the callback stops as soon as the command is
/// queued: the held item is left waiting for whoever took over.
#[test]
fn a_prompt_taken_over_meanwhile_leaves_the_held_item_waiting() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("handover", 80, 24);
    unsafe {
        let (mut item, rv) = run_exec(&mut t, c, c"command-prompt");
        assert_eq!(rv, CMD_RETURN_WAIT);
        item.set_flags(CMDQ_WAITING);

        let mut queue = Queue::new();
        queue.place(&mut item);

        let prompt = (*c).prompt;
        (*c).prompt = ANOTHER_PROMPT;
        let rc = prompt.input(c, &mut (*c).prompt_data, Some(c"display-panes"), 1);
        (*c).prompt = prompt;
        assert_eq!(rc, 1, "the callback did not stop for the new prompt");

        assert_eq!(queue.len(), 2, "the command was still built and queued");
        let queued = (*queue.ptr()).list[1].as_ptr();
        assert!(matches!((*queued).type_0, CmdqType::Command { .. }));
        assert_eq!(
            (*item.ptr()).flags & CMDQ_WAITING,
            CMDQ_WAITING,
            "the held item was let go anyway"
        );

        status_prompt_clear(c);
        drop(queue);
    }
}

/// `-i` answers at once instead of waiting: the command returns normal
/// immediately and never holds the item. Setting an incremental prompt feeds
/// it a first answer there and then, whose template fill does not parse — the
/// resulting complaint lands as an error item on the client's own queue.
#[test]
fn i_answers_at_once_without_waiting() {
    let _guard = globals();
    let mut t = Target::new(80, 24);
    let mut clients = Clients::new();
    let c = clients.add("incrementer", 80, 24);
    unsafe {
        let mut queue = Queue::for_client(c);

        let (_item, rv) = run_exec(&mut t, c, c"command-prompt -i");
        assert_eq!(rv, CMD_RETURN_NORMAL);

        let cd = cdata_of(c);
        assert_ne!(cd.flags & PROMPT_INCREMENTAL, 0);
        assert_ne!((*c).prompt_flags & PROMPT_INCREMENTAL, 0);
        assert!(cd.item.is_none(), "-i never holds the item");
        assert!((*c).prompt_last.is_some());
        assert_eq!((*c).prompt, Prompt::CommandPrompt);

        assert_eq!(queue.len(), 1);
        let queued = (*queue.ptr()).list[0].as_ptr();
        assert!(matches!((*queued).type_0, CmdqType::Callback { .. }));

        status_prompt_clear(c);
        drop(queue);
    }
}

//! `confirm-before`: puts a yes/no question on a client's status line and runs
//! the command it was given only if the answer is yes.
//!
//! Exec parses the command list straight away, parks it with the rest of the
//! command's private state in a boxed [`cmd_confirm_before_data`] owned by the
//! status line, which hands it to the answer handler. Without
//! `-b` the item that asked stays on the queue, so the confirmed command is
//! spliced in behind it and the answer decides the asking client's exit code;
//! with `-b` there is no item to wait on and the command goes onto the
//! answering client's own queue instead.
//!
//! Quirks kept:
//!
//! * The confirm key is checked only after the command list has been parsed,
//!   so `confirm-before -c ab not-a-command` reports the command that does not
//!   parse and never mentions the key.
//! * `-b` gives up more than the wait. With no item behind the prompt the
//!   answer handler sets no exit code at all, so a declined `-b` looks exactly
//!   like a confirmed one from the outside, and the confirmed command is
//!   appended to whichever client answered rather than inserted after the
//!   command that asked.
//! * The answer is only ever the first byte of what was typed, so a longer
//!   reply is confirmed whenever it starts with the key — `yes` confirms, and
//!   so does `y!`.
//! * An empty answer declines rather than taking the default: `-y` makes the
//!   carriage return mean yes, and an empty string is not one.
//! * `-p` has a space appended to whatever it says, the same space the
//!   built-in question ends with, so a prompt written with its own trailing
//!   space gets two.
//!
//! One upstream read is narrowed rather than reproduced. The C tests
//! `confirm_key[1]` before it tests `confirm_key[0]`, so `-c ""` reads the
//! byte past the terminator of a one-byte string; whatever is there the answer
//! is the same refusal, since `confirm_key[0]` is then the terminator itself
//! and not printable. The rewrite reads the length off the string instead,
//! which gives that same refusal without the overread.

use crate::arguments::{args_get_str, args_has, args_make_commands_now};
use crate::cmd::queue::CmdqItemWeak;
use crate::cmd::queue::{cmdq_append, cmdq_item_weak_of};
use crate::cmd::{cmd_get_args, cmd_get_entry};
use crate::fmt_args;
use crate::prompt_history::PromptHistoryType;
use crate::status::status_prompt_for_client;
pub use crate::types::*;
use ::core::ffi::{CStr, c_int};
use ::std::ffi::CString;

pub use crate::consts::{
    ARGS_PARSE_COMMANDS_OR_STRING, CLIENT_DEAD, CMD_CLIENT_TFLAG, CMD_FIND_PANE, CMD_RETURN_ERROR,
    CMD_RETURN_NORMAL, CMD_RETURN_WAIT, PROMPT_SINGLE,
};

/// What the command leaves on the client while the question is up: the item
/// waiting on the answer, if any, the command list to run once it is yes, the
/// byte that means yes, and whether the carriage return means it too.
#[derive(Default)]
#[repr(C)]
pub struct cmd_confirm_before_data {
    pub(crate) item: Option<CmdqItemWeak>,
    pub(crate) cmdlist: Option<CmdListRef>,
    pub confirm_key: u_char,
    pub default_yes: c_int,
}

pub(crate) static cmd_confirm_before_entry: RustCommandEntry = RustCommandEntry {
    name: c"confirm-before",
    alias: Some(c"confirm"),
    args: args_parse_t {
        template: c"bc:p:t:y",
        lower: 1,
        upper: 1,
        cb: Some(cmd_confirm_before_args_parse),
    },
    usage: c"[-by] [-c confirm-key] [-p prompt] [-t target-client] command",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: CMD_CLIENT_TFLAG,
    exec: cmd_confirm_before_exec,
};

/// How the parser is told to read the command the question is about: as a
/// command list if it parses as one, and as a plain string otherwise.
fn cmd_confirm_before_args_parse(
    _args: &args,
    _idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    ARGS_PARSE_COMMANDS_OR_STRING
}

/// The byte that means yes, or nothing when `-c` does not name exactly one
/// printable character. A command with no `-c` at all confirms on `y`.
///
/// Printable here is 32 through 126: the C reads the byte as a signed `char`,
/// so anything from 128 up is negative and fails the lower bound, which is the
/// same set an unsigned byte fails the upper bound on.
fn cmd_confirm_before_key(args: &args) -> Option<u_char> {
    let Some(confirm_key) = args_get_str(args, b'c') else {
        return Some(b'y');
    };
    match confirm_key.to_bytes() {
        &[key] if key > 31 && key < 127 => Some(key),
        _ => None,
    }
}

/// The question to put on the status line, null-terminated: what `-p` says, or
/// the name of the first command in the list with the key that confirms it.
/// Both end in the same space, which is where the two `xasprintf` formats
/// agree.
fn cmd_confirm_before_prompt(args: &args, cdata: &cmd_confirm_before_data) -> Vec<u8> {
    let mut prompt = Vec::new();
    if let Some(given) = args_get_str(args, b'p') {
        prompt.extend_from_slice(given.to_bytes());
    } else {
        let command = cdata
            .cmdlist
            .as_ref()
            .unwrap()
            .command(0)
            .expect("the confirmed command");
        let name = crate::CommandEntry::name(cmd_get_entry(&command));
        prompt.extend_from_slice(b"Confirm '");
        prompt.extend_from_slice(name.to_bytes());
        prompt.extend_from_slice(b"'? (");
        prompt.push(cdata.confirm_key);
        prompt.extend_from_slice(b"/n)");
    }
    prompt.extend_from_slice(b" \0");
    prompt
}

unsafe fn cmd_confirm_before_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let wait = args_has(args, b'b') == 0;

    let mut cdata = Box::<cmd_confirm_before_data>::default();
    unsafe { cdata.cmdlist = args_make_commands_now(self_0, item, 0 as u_int, 1) };
    if cdata.cmdlist.is_none() {
        return CMD_RETURN_ERROR;
    }
    if wait {
        cdata.item = cmdq_item_weak_of(item);
    }
    cdata.default_yes = args_has(args, b'y');
    match cmd_confirm_before_key(args) {
        Some(key) => cdata.confirm_key = key,
        None => {
            unsafe { item.error(c"invalid confirm key", fmt_args![]) };
            let _ = cdata.cmdlist.take();
            return CMD_RETURN_ERROR;
        }
    }

    let prompt = cmd_confirm_before_prompt(args, &cdata);
    let target_client = item.target_client();
    unsafe {
        status_prompt_for_client(
            &mut target_client.expect("confirm-before target client"),
            Some(&item.target),
            CStr::from_bytes_with_nul(&prompt).expect("a prompt ends with a NUL"),
            None,
            Prompt::ConfirmBefore,
            PromptData::ConfirmBefore(cdata),
            PROMPT_SINGLE,
            PromptHistoryType::Command,
        )
    };

    match wait {
        true => CMD_RETURN_WAIT,
        false => CMD_RETURN_NORMAL,
    }
}

/// Whether `s` is a yes. A client that has died in the meantime and a prompt
/// that was cancelled — which reports a null string — are both a no, and so is
/// anything whose first byte is neither the confirm key nor, with `-y`, the
/// carriage return. The comparison is made as `int` the way C's is, so a byte
/// above 127 can never match a key.
unsafe fn cmd_confirm_before_confirmed(
    c: &ClientRef,
    cdata: &cmd_confirm_before_data,
    s: Option<&CStr>,
) -> bool {
    let Some(s) = s else {
        return false;
    };
    if unsafe { c.flags() } & CLIENT_DEAD as uint64_t != 0 {
        return false;
    }
    let first = s.to_bytes_with_nul()[0] as core::ffi::c_char as c_int;
    first == cdata.confirm_key as c_int || (first == '\r' as c_int && cdata.default_yes != 0)
}

/// The status line's answer handler. A yes queues the command list — after the
/// waiting item when there is one, on the answering client otherwise — and a
/// no leaves it alone; either way a waiting item is released, and the client
/// behind it is told the outcome as an exit code unless it has a session,
/// which means its exit code comes from somewhere else.
pub(crate) unsafe fn cmd_confirm_before_callback(
    c: &mut ClientRef,
    data: &mut cmd_confirm_before_data,
    s: Option<&CStr>,
    _done: c_int,
) -> c_int {
    unsafe {
        let item = data.item.as_ref().and_then(CmdqItemWeak::upgrade);
        let retcode = match cmd_confirm_before_confirmed(c, data, s) {
            false => 1,
            true => {
                match &item {
                    None => {
                        cmdq_append(Some(c), (data.cmdlist.as_ref().unwrap()).queue_items(None))
                    }
                    Some(item) => {
                        let state = item.state_ref();
                        Some(item.insert_after(
                            (data.cmdlist.as_ref().unwrap()).queue_items(Some(&state)),
                        ))
                    }
                };
                0
            }
        };
        if let Some(item) = &item {
            if let Some(asked) = item.client()
                && asked.attached_session().is_none()
            {
                asked.set_return_code(retcode);
            }
            item.resume();
        }
        0
    }
}

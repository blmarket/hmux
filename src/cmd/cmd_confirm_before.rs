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
//!
//! Coverage exemptions: none.

use crate::arguments::{args_get, args_has, args_make_commands_now};
use crate::cmd::queue::CmdqItemWeak;
use crate::cmd::queue::{
    cmdq_append, cmdq_continue, cmdq_error, cmdq_get_client, cmdq_get_command, cmdq_get_state_ref,
    cmdq_get_target, cmdq_get_target_client, cmdq_insert_after, cmdq_item_weak_from_ptr,
};
use crate::cmd::{cmd_get_args, cmd_get_entry, cmd_list_first};
use crate::fmt_args;
use crate::status::status_prompt_set;
pub use crate::types::*;
use ::core::ffi::{CStr, c_int};
use ::std::ffi::CString;

pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
pub const PROMPT_ENTRY: client_prompt_mode = 0;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const CMD_CLIENT_TFLAG: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CLIENT_DEAD: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const PROMPT_SINGLE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;

/// What the command leaves on the client while the question is up: the item
/// waiting on the answer, if any, the command list to run once it is yes, the
/// byte that means yes, and whether the carriage return means it too.
#[derive(Default)]
#[repr(C)]
pub struct cmd_confirm_before_data {
    pub(crate) item: Option<CmdqItemWeak>,
    pub(crate) cmdlist: Option<CmdListRef>,
    pub confirm_key: u_char,
    pub default_yes: ::core::ffi::c_int,
}

pub(crate) static cmd_confirm_before_entry: cmd_entry = cmd_entry {
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
unsafe fn cmd_confirm_before_key(args: &args) -> Option<u_char> {
    unsafe {
        let confirm_key = args_get(args, b'c');
        if confirm_key.is_null() {
            return Some(b'y');
        }
        match CStr::from_ptr(confirm_key).to_bytes() {
            &[key] if key > 31 && key < 127 => Some(key),
            _ => None,
        }
    }
}

/// The question to put on the status line, null-terminated: what `-p` says, or
/// the name of the first command in the list with the key that confirms it.
/// Both end in the same space, which is where the two `xasprintf` formats
/// agree.
unsafe fn cmd_confirm_before_prompt(args: &args, cdata: *mut cmd_confirm_before_data) -> Vec<u8> {
    unsafe {
        let mut prompt = Vec::new();
        let given = args_get(args, b'p');
        if given.is_null() {
            let name = cmd_get_entry(&*cmd_list_first(
                (*cdata).cmdlist.as_ref().unwrap().as_ptr(),
            ))
            .name;
            prompt.extend_from_slice(b"Confirm '");
            prompt.extend_from_slice(name.to_bytes());
            prompt.extend_from_slice(b"'? (");
            prompt.push((*cdata).confirm_key);
            prompt.extend_from_slice(b"/n)");
        } else {
            prompt.extend_from_slice(CStr::from_ptr(given).to_bytes());
        }
        prompt.extend_from_slice(b" \0");
        prompt
    }
}

unsafe fn cmd_confirm_before_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let wait = args_has(args, b'b') == 0;

        let mut cdata = Box::<cmd_confirm_before_data>::default();
        let cdata_ptr = &raw mut *cdata;
        cdata.cmdlist = args_make_commands_now(self_0, item, 0 as u_int, 1);
        if cdata.cmdlist.is_none() {
            return CMD_RETURN_ERROR;
        }
        if wait {
            cdata.item = cmdq_item_weak_from_ptr(item);
        }
        cdata.default_yes = args_has(args, b'y');
        match cmd_confirm_before_key(args) {
            Some(key) => cdata.confirm_key = key,
            None => {
                cmdq_error(item, c"invalid confirm key".as_ptr(), fmt_args![]);
                let _ = cdata.cmdlist.take();
                return CMD_RETURN_ERROR;
            }
        }

        let prompt = cmd_confirm_before_prompt(args, cdata_ptr);
        status_prompt_set(
            cmdq_get_target_client(&*item),
            cmdq_get_target(item),
            CStr::from_bytes_with_nul(&prompt).expect("a prompt ends with a NUL"),
            None,
            Prompt::ConfirmBefore,
            PromptData::ConfirmBefore(cdata),
            PROMPT_SINGLE,
            PROMPT_TYPE_COMMAND,
        );

        match wait {
            true => CMD_RETURN_WAIT,
            false => CMD_RETURN_NORMAL,
        }
    }
}

/// Whether `s` is a yes. A client that has died in the meantime and a prompt
/// that was cancelled — which reports a null string — are both a no, and so is
/// anything whose first byte is neither the confirm key nor, with `-y`, the
/// carriage return. The comparison is made as `int` the way C's is, so a byte
/// above 127 can never match a key.
unsafe fn cmd_confirm_before_confirmed(
    c: *mut client,
    cdata: *mut cmd_confirm_before_data,
    s: Option<&CStr>,
) -> bool {
    unsafe {
        let Some(s) = s else {
            return false;
        };
        if (*c).flags & CLIENT_DEAD as uint64_t != 0 {
            return false;
        }
        let first = *s.as_ptr() as c_int;
        first == (*cdata).confirm_key as c_int
            || (first == '\r' as c_int && (*cdata).default_yes != 0)
    }
}

/// The status line's answer handler. A yes queues the command list — after the
/// waiting item when there is one, on the answering client otherwise — and a
/// no leaves it alone; either way a waiting item is released, and the client
/// behind it is told the outcome as an exit code unless it has a session,
/// which means its exit code comes from somewhere else.
pub(crate) unsafe fn cmd_confirm_before_callback(
    c: *mut client,
    data: *mut cmd_confirm_before_data,
    s: Option<&CStr>,
    _done: c_int,
) -> c_int {
    unsafe {
        let cdata = data;
        let item = (*cdata).item.as_ref().and_then(CmdqItemWeak::upgrade);
        let retcode = match cmd_confirm_before_confirmed(c, cdata, s) {
            false => 1,
            true => {
                match &item {
                    None => cmdq_append(
                        c,
                        cmdq_get_command((*cdata).cmdlist.as_ref().unwrap(), None),
                    ),
                    Some(item) => cmdq_insert_after(
                        item.as_ptr(),
                        cmdq_get_command(
                            (*cdata).cmdlist.as_ref().unwrap(),
                            Some(cmdq_get_state_ref(item.as_ptr())),
                        ),
                    ),
                };
                0
            }
        };
        if let Some(item) = &item {
            let asked = cmdq_get_client(&*item.as_ptr());
            if !asked.is_null() && (*asked).session.is_null() {
                (*asked).retval = retcode;
            }
            cmdq_continue(item.as_ptr());
        }
        0
    }
}

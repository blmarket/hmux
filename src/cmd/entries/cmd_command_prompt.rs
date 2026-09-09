//! `command-prompt`: puts a prompt up on a client, and runs a command built
//! from what is typed into it.
//!
//! The command line's template — `%1`, `%2`, … standing for the answers — is
//! prepared once by the argument layer and kept in the command's own state
//! until every prompt has been answered. `-p` names the prompts, one per
//! comma, `-I` the text each starts out holding, again one per comma, and `-l`
//! takes both whole instead of splitting them. Without `-p` the prompt is the
//! template in brackets, or a bare `:` when there is no template at all.
//!
//! Answers arrive through [`cmd_command_prompt_callback`], which the status
//! line calls once per answer: each finished answer is appended to the
//! command's argv and the next prompt put up, and the last one builds the
//! command list. Where that list goes depends on whether the command is
//! waiting — `command-prompt` normally holds the item that ran it and inserts
//! its work after it, while `-b` and `-i` do not wait and append to the
//! client's queue instead.
//!
//! The prompts live in one heap array of [`cmd_command_prompt_prompt`] with a
//! count beside it, which [`cmd_command_prompt_free`] gives back one string at
//! a time; the array's element type is crate-canonical, so it stays a C array
//! rather than becoming a `Vec`.
//!
//! Quirks kept: the target client is read before anything checks there is one;
//! a prompt built from the template is split on commas like any other, so a
//! template carrying a comma becomes two prompts; `-l` ignores the space that
//! is otherwise appended to every prompt, and with no `-I` it hands the status
//! line a null input; `-i` overrides `-b`'s waiting choice by clearing the
//! same flag; and an unknown `-T` is only refused after every prompt has
//! already been built, which is why the refusal frees the command's state by
//! hand.

use crate::args::RustArguments;
use crate::args::{
    args_count, args_get_str, args_has, args_make_commands, args_make_commands_get_command,
    args_make_commands_prepare,
};
use crate::cmd::CmdqItemRef;
use crate::cmd::cmd_get_args;
use crate::cmd::CmdqItemWeak;
use crate::cmd::{cmdq_append, cmdq_item_weak_of};
use crate::consts::{
    ARGS_PARSE_COMMANDS_OR_STRING, CMD_CLIENT_TFLAG, CMD_FIND_PANE, CMD_RETURN_ERROR,
    CMD_RETURN_NORMAL, CMD_RETURN_WAIT, PROMPT_BSPACE_EXIT, PROMPT_INCREMENTAL, PROMPT_KEY,
    PROMPT_NOFREEZE, PROMPT_NUMERIC, PROMPT_SINGLE,
};
use crate::fmt_args;
use crate::prompt_history::PromptHistoryType;
use crate::status::{status_prompt_for_client, status_prompt_update_for_client};
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::cmd::cmdq_item;
use crate::types::{
    ClientRef, Prompt, PromptData, args, args_command_state, args_parse_t, args_parse_type,
    cmd_command_prompt_prompt, u_int,
};
use ::core::ffi::{CStr, c_int};
use ::std::ffi::CString;

#[derive(Clone)]
#[repr(C)]
pub struct cmd_command_prompt_cdata {
    pub(crate) item: Option<CmdqItemWeak>,
    pub state: Option<Box<args_command_state>>,
    pub flags: c_int,
    pub prompt_type: PromptHistoryType,
    pub prompts: Vec<cmd_command_prompt_prompt>,
    pub current: u_int,
    pub argv: Vec<CString>,
}
pub(crate) static cmd_command_prompt_entry: RustCommandEntry = RustCommandEntry {
    name: c"command-prompt",
    alias: None,
    args: args_parse_t {
        template: c"1CbeFiklI:Np:t:T:",
        lower: 0,
        upper: 1,
        cb: Some(cmd_command_prompt_args_parse),
    },
    usage: c"[-1CbeFiklN] [-I inputs] [-p prompts] [-t target-client] [-T prompt-type] [template]",
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
    exec: cmd_command_prompt_exec,
};

fn cmd_command_prompt_args_parse(
    _args: &args,
    _idx: u_int,
    _cause: &mut Option<CString>,
) -> args_parse_type {
    ARGS_PARSE_COMMANDS_OR_STRING
}

fn push_prompt(
    cdata: &mut cmd_command_prompt_cdata,
    prompt: Option<CString>,
    input: Option<CString>,
) {
    cdata
        .prompts
        .push(cmd_command_prompt_prompt { prompt, input });
}

fn split_prompts(
    cdata: &mut cmd_command_prompt_cdata,
    prompts: Option<&CStr>,
    inputs: Option<&CStr>,
    space: bool,
) {
    let Some(prompts) = prompts else { return };
    let mut next_input = inputs.map(|c| c.to_bytes());
    for field in prompts.to_bytes().split(|b| *b == b',') {
        let prompt = if space {
            let mut spaced = field.to_vec();
            spaced.push(b' ');
            CString::new(spaced).ok()
        } else {
            CString::new(field).ok()
        };
        let input = match next_input {
            Some(rest) => match rest.iter().position(|b| *b == b',') {
                Some(at) => {
                    next_input = Some(&rest[at + 1..]);
                    CString::new(&rest[..at]).ok()
                }
                None => {
                    next_input = None;
                    CString::new(rest).ok()
                }
            },
            None => Some(CString::default()),
        };
        push_prompt(cdata, prompt, input);
    }
}

/// The mode bit the flags ask for, at most one of them: the C tests these five
/// in order and stops at the first that is there.
fn mode_flag(args: &RustArguments) -> c_int {
    for (flag, bit) in [
        (b'1', PROMPT_SINGLE),
        (b'N', PROMPT_NUMERIC),
        (b'i', PROMPT_INCREMENTAL),
        (b'k', PROMPT_KEY),
        (b'e', PROMPT_BSPACE_EXIT),
    ] {
        if args_has(args, flag) != 0 {
            return bit;
        }
    }
    0
}

unsafe fn cmd_command_prompt_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let mut tc = item
        .target_client()
        .expect("the command has a target client");

    if tc.has_prompt_text() {
        return CMD_RETURN_NORMAL;
    }
    let wait = (args_has(args, b'b') == 0 && args_has(args, b'i') == 0) as c_int;

    let mut cdata = unsafe {
        Box::new(cmd_command_prompt_cdata {
            item: match wait != 0 {
                true => cmdq_item_weak_of(item),
                false => None,
            },
            state: Some(args_make_commands_prepare(
                self_0,
                item,
                0,
                Some(c"%1"),
                wait,
                |cmd| {
                    if args_has(args, b'F') != 0 {
                        crate::format::format_single_from_target(item, cmd)
                    } else {
                        cmd.to_owned()
                    }
                },
            )),
            flags: 0,
            prompt_type: PromptHistoryType::Command,
            prompts: Vec::new(),
            current: 0,
            argv: Vec::new(),
        })
    };

    let mut space = true;
    let prompts = if let Some(s) = args_get_str(args, b'p') {
        Some(s.to_owned())
    } else if args_count(args) != 0 {
        let tmp = unsafe { args_make_commands_get_command(cdata.state.as_deref().unwrap()) };
        let mut spelled = b"(".to_vec();
        spelled.extend_from_slice(tmp.as_bytes());
        spelled.push(b')');
        CString::new(spelled).ok()
    } else {
        space = false;
        Some(CString::new(":").unwrap())
    };
    let inputs = args_get_str(args, b'I').map(|s| s.to_owned());

    if args_has(args, b'l') != 0 {
        cdata.prompts.push(cmd_command_prompt_prompt {
            prompt: prompts,
            input: inputs,
        });
    } else {
        split_prompts(&mut cdata, prompts.as_deref(), inputs.as_deref(), space);
    }

    if let Some(type_0) = args_get_str(args, b'T') {
        let Some(kind) = PromptHistoryType::parse(type_0) else {
            unsafe { item.error(c"unknown type: %s", fmt_args![type_0]) };
            cmd_command_prompt_free(cdata);
            return CMD_RETURN_ERROR;
        };
        cdata.prompt_type = kind;
    } else {
        cdata.prompt_type = PromptHistoryType::Command;
    }

    cdata.flags |= mode_flag(args);
    if args_has(args, b'C') != 0 {
        cdata.flags |= PROMPT_NOFREEZE;
    }
    let first = &(&cdata.prompts)[0];
    let flags = cdata.flags;
    let prompt_type = cdata.prompt_type;
    let first_prompt = first.prompt.clone().unwrap_or_else(|| c"".to_owned());
    let first_input = first.input.clone();
    unsafe {
        status_prompt_for_client(
            &mut tc,
            Some(&item.target),
            &first_prompt,
            first_input.as_deref(),
            Prompt::CommandPrompt,
            PromptData::CommandPrompt(cdata),
            flags,
            prompt_type,
        )
    };

    if wait == 0 {
        return CMD_RETURN_NORMAL;
    }
    CMD_RETURN_WAIT
}

pub(crate) unsafe fn cmd_command_prompt_callback(
    c: &mut ClientRef,
    cdata: &mut cmd_command_prompt_cdata,
    s: Option<&CStr>,
    done: c_int,
) -> c_int {
    unsafe {
        let item = cdata.item.as_ref().and_then(CmdqItemWeak::upgrade);

        'out: {
            let Some(s) = s else {
                break 'out;
            };
            if done != 0 {
                if cdata.flags & PROMPT_INCREMENTAL != 0 {
                    break 'out;
                }
                cdata.argv.push(s.to_owned());
                cdata.current = cdata.current.wrapping_add(1);
                if (cdata.current as usize) < cdata.prompts.len() {
                    let prompt = &cdata.prompts[cdata.current as usize];
                    status_prompt_update_for_client(
                        c,
                        prompt.prompt.as_deref().unwrap_or(c""),
                        prompt.input.as_deref(),
                    );
                    return 1;
                }
            }

            let mut argv = cdata.argv.clone();
            if done == 0 {
                argv.push(s.to_owned());
            }

            let mut error = None;
            let cmdlist =
                args_make_commands(cdata.state.as_deref_mut().unwrap(), &argv, &mut error);
            if let Some(error) = error.as_ref() {
                cmdq_append(Some(c), CmdqItemRef::error_items(error));
            } else if let Some(item) = &item {
                let state = item.state_ref();
                item.insert_after((cmdlist.as_ref().unwrap()).queue_items(Some(&state)));
            } else {
                cmdq_append(Some(c), (cmdlist.as_ref().unwrap()).queue_items(None));
            }
            if c.prompt() != Prompt::CommandPrompt {
                return 1;
            }
        }

        if let Some(item) = &item {
            item.resume();
        }
        0
    }
}

#[allow(clippy::boxed_local)]
pub(crate) fn cmd_command_prompt_free(_data: Box<cmd_command_prompt_cdata>) {}

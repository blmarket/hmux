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
//!
//! Coverage exemptions: none.

use crate::arguments::{
    args_count, args_get, args_has, args_make_commands, args_make_commands_get_command,
    args_make_commands_prepare,
};
use crate::cmd::cmd_get_args;
use crate::cmd::queue::CmdqItemWeak;
use crate::cmd::queue::{
    cmdq_append, cmdq_continue, cmdq_error, cmdq_get_command, cmdq_get_error, cmdq_get_state_ref,
    cmdq_get_target, cmdq_get_target_client, cmdq_insert_after, cmdq_item_weak_from_ptr,
};
use crate::fmt_args;
use crate::status::{status_prompt_set, status_prompt_type, status_prompt_update};
pub use crate::types::*;
use ::core::ffi::{CStr, c_int};
use ::std::ffi::CString;
pub const MSG_READ_CANCEL: msgtype = 307;
pub const MSG_WRITE_CLOSE: msgtype = 306;
pub const MSG_READ: msgtype = 301;
pub const MSG_READY: msgtype = 207;
pub const MSG_COMMAND: msgtype = 200;
pub const MSG_IDENTIFY_FLAGS: msgtype = 100;
pub const MSG_VERSION: msgtype = 12;
pub const PANE_LINES_SPACES: pane_lines = 5;
pub const PANE_LINES_NUMBER: pane_lines = 4;
pub const PANE_LINES_SIMPLE: pane_lines = 3;
pub const PANE_LINES_HEAVY: pane_lines = 2;
pub const PANE_LINES_DOUBLE: pane_lines = 1;
pub const PANE_LINES_SINGLE: pane_lines = 0;
pub const PROGRESS_BAR_PAUSED: progress_bar_state = 4;
pub const PROGRESS_BAR_INDETERMINATE: progress_bar_state = 3;
pub const PROGRESS_BAR_ERROR: progress_bar_state = 2;
pub const PROGRESS_BAR_NORMAL: progress_bar_state = 1;
pub const PROGRESS_BAR_HIDDEN: progress_bar_state = 0;
pub const SCREEN_CURSOR_BAR: screen_cursor_style = 3;
pub const SCREEN_CURSOR_UNDERLINE: screen_cursor_style = 2;
pub const SCREEN_CURSOR_BLOCK: screen_cursor_style = 1;
pub const SCREEN_CURSOR_DEFAULT: screen_cursor_style = 0;
pub const STYLE_DEFAULT_SET: style_default_type = 3;
pub const STYLE_DEFAULT_POP: style_default_type = 2;
pub const STYLE_DEFAULT_PUSH: style_default_type = 1;
pub const STYLE_DEFAULT_BASE: style_default_type = 0;
pub const STYLE_RANGE_CONTROL: style_range_type = 7;
pub const STYLE_RANGE_USER: style_range_type = 6;
pub const STYLE_RANGE_SESSION: style_range_type = 5;
pub const STYLE_RANGE_WINDOW: style_range_type = 4;
pub const STYLE_RANGE_PANE: style_range_type = 3;
pub const STYLE_RANGE_RIGHT: style_range_type = 2;
pub const STYLE_RANGE_LEFT: style_range_type = 1;
pub const STYLE_RANGE_NONE: style_range_type = 0;
pub const STYLE_LIST_RIGHT_MARKER: style_list = 4;
pub const STYLE_LIST_LEFT_MARKER: style_list = 3;
pub const STYLE_LIST_FOCUS: style_list = 2;
pub const STYLE_LIST_ON: style_list = 1;
pub const STYLE_LIST_OFF: style_list = 0;
pub const STYLE_ALIGN_ABSOLUTE_CENTRE: style_align = 4;
pub const STYLE_ALIGN_RIGHT: style_align = 3;
pub const STYLE_ALIGN_CENTRE: style_align = 2;
pub const STYLE_ALIGN_LEFT: style_align = 1;
pub const STYLE_ALIGN_DEFAULT: style_align = 0;
pub const THEME_DARK: client_theme = 2;
pub const THEME_LIGHT: client_theme = 1;
pub const THEME_UNKNOWN: client_theme = 0;
pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
pub const PROMPT_TYPE_INVALID: prompt_type = 255;
pub const PROMPT_TYPE_WINDOW_TARGET: prompt_type = 3;
pub const PROMPT_TYPE_TARGET: prompt_type = 2;
pub const PROMPT_TYPE_SEARCH: prompt_type = 1;
pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
pub const PROMPT_COMMAND: client_prompt_mode = 1;
pub const PROMPT_ENTRY: client_prompt_mode = 0;
pub const CLIENT_EXIT_DETACH: client_exit_type = 2;
pub const CLIENT_EXIT_SHUTDOWN: client_exit_type = 1;
pub const CLIENT_EXIT_RETURN: client_exit_type = 0;
pub const ARGS_PARSE_COMMANDS: args_parse_type = 3;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const ARGS_PARSE_STRING: args_parse_type = 1;
pub const ARGS_PARSE_INVALID: args_parse_type = 0;
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const CMD_CLIENT_TFLAG: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const PROMPT_SINGLE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PROMPT_NUMERIC: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const PROMPT_INCREMENTAL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const PROMPT_KEY: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const PROMPT_BSPACE_EXIT: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const PROMPT_NOFREEZE: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
#[derive(Clone)]
#[repr(C)]
pub struct cmd_command_prompt_cdata {
    pub(crate) item: Option<CmdqItemWeak>,
    pub state: Option<Box<args_command_state>>,
    pub flags: ::core::ffi::c_int,
    pub prompt_type: prompt_type,
    pub prompts: Vec<cmd_command_prompt_prompt>,
    pub current: u_int,
    pub argv: Vec<CString>,
}
pub(crate) static cmd_command_prompt_entry: cmd_entry = cmd_entry {
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

unsafe fn push_prompt(
    cdata: *mut cmd_command_prompt_cdata,
    prompt: Option<CString>,
    input: Option<CString>,
) {
    unsafe {
        (*cdata)
            .prompts
            .push(cmd_command_prompt_prompt { prompt, input });
    }
}

unsafe fn split_prompts(
    cdata: *mut cmd_command_prompt_cdata,
    prompts: Option<&CStr>,
    inputs: Option<&CStr>,
    space: bool,
) {
    unsafe {
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
}

/// The mode bit the flags ask for, at most one of them: the C tests these five
/// in order and stops at the first that is there.
fn mode_flag(args: &args) -> c_int {
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

unsafe fn cmd_command_prompt_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let tc = cmdq_get_target_client(&*item);
        if (*tc).prompt_string.is_some() {
            return CMD_RETURN_NORMAL;
        }
        let wait = (args_has(args, b'b') == 0 && args_has(args, b'i') == 0) as c_int;

        let mut cdata = Box::new(cmd_command_prompt_cdata {
            item: match wait != 0 {
                true => cmdq_item_weak_from_ptr(item),
                false => None,
            },
            state: Some(args_make_commands_prepare(
                self_0,
                item,
                0,
                c"%1".as_ptr(),
                wait,
                args_has(args, b'F'),
            )),
            flags: 0,
            prompt_type: PROMPT_TYPE_COMMAND,
            prompts: Vec::new(),
            current: 0,
            argv: Vec::new(),
        });
        let cdata_ptr = &raw mut *cdata;

        let mut space = true;
        let s = args_get(args, b'p');
        let prompts = if !s.is_null() {
            Some(CStr::from_ptr(s).to_owned())
        } else if args_count(args) != 0 {
            let tmp = args_make_commands_get_command(cdata.state.as_deref().unwrap());
            let mut spelled = b"(".to_vec();
            spelled.extend_from_slice(tmp.as_bytes());
            spelled.push(b')');
            CString::new(spelled).ok()
        } else {
            space = false;
            Some(CString::new(":").unwrap())
        };
        let s = args_get(args, b'I');
        let inputs = if s.is_null() {
            None
        } else {
            Some(CStr::from_ptr(s).to_owned())
        };

        if args_has(args, b'l') != 0 {
            cdata.prompts.push(cmd_command_prompt_prompt {
                prompt: prompts,
                input: inputs,
            });
        } else {
            split_prompts(cdata_ptr, prompts.as_deref(), inputs.as_deref(), space);
        }

        let type_0 = args_get(args, b'T');
        if !type_0.is_null() {
            cdata.prompt_type = status_prompt_type(CStr::from_ptr(type_0));
            if cdata.prompt_type == PROMPT_TYPE_INVALID {
                cmdq_error(item, c"unknown type: %s".as_ptr(), fmt_args![type_0]);
                cmd_command_prompt_free(cdata);
                return CMD_RETURN_ERROR;
            }
        } else {
            cdata.prompt_type = PROMPT_TYPE_COMMAND;
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
        status_prompt_set(
            tc,
            cmdq_get_target(item),
            &first_prompt,
            first_input.as_deref(),
            Prompt::CommandPrompt,
            PromptData::CommandPrompt(cdata),
            flags,
            prompt_type,
        );

        if wait == 0 {
            return CMD_RETURN_NORMAL;
        }
        CMD_RETURN_WAIT
    }
}

pub(crate) unsafe fn cmd_command_prompt_callback(
    c: *mut client,
    data: *mut cmd_command_prompt_cdata,
    s: Option<&CStr>,
    done: c_int,
) -> c_int {
    unsafe {
        let cdata = data;
        let item = (*cdata).item.as_ref().and_then(CmdqItemWeak::upgrade);

        'out: {
            let Some(s) = s else {
                break 'out;
            };
            if done != 0 {
                if (*cdata).flags & PROMPT_INCREMENTAL != 0 {
                    break 'out;
                }
                (*cdata).argv.push(s.to_owned());
                (*cdata).current = (*cdata).current.wrapping_add(1);
                if ((*cdata).current as usize) < (*cdata).prompts.len() {
                    let prompt = &(&(*cdata).prompts)[(*cdata).current as usize];
                    status_prompt_update(
                        c,
                        prompt.prompt.as_deref().unwrap_or(c""),
                        prompt.input.as_deref(),
                    );
                    return 1;
                }
            }

            let mut argv = (*cdata).argv.clone();
            if done == 0 {
                argv.push(s.to_owned());
            }

            let mut error = None;
            let cmdlist =
                args_make_commands((*cdata).state.as_deref_mut().unwrap(), &argv, &mut error);
            if let Some(error) = error.as_ref() {
                cmdq_append(c, cmdq_get_error(error.as_ptr()));
            } else if let Some(item) = &item {
                cmdq_insert_after(
                    item.as_ptr(),
                    cmdq_get_command(
                        cmdlist.as_ref().unwrap(),
                        Some(cmdq_get_state_ref(item.as_ptr())),
                    ),
                );
            } else {
                cmdq_append(c, cmdq_get_command(cmdlist.as_ref().unwrap(), None));
            }
            if (*c).prompt != Prompt::CommandPrompt {
                return 1;
            }
        }

        if let Some(item) = &item {
            cmdq_continue(item.as_ptr());
        }
        0
    }
}

pub(crate) unsafe fn cmd_command_prompt_free(mut data: Box<cmd_command_prompt_cdata>) {
    drop(data.state.take());
}

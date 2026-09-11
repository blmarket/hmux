//! `paste-buffer`: sends a paste buffer's bytes to a pane, a line at a time.
//!
//! The buffer is the one `-b` names or, without it, the top of the store; a
//! `-b` naming nothing is the command's one error besides a pane that has
//! already exited. The bytes are cut at every newline and each cut line is
//! followed by a separator — `-s`'s text if it was given, a newline under
//! `-r`, and a carriage return otherwise, which is what makes a pasted
//! multi-line buffer arrive at a shell as if it had been typed. What follows
//! the last newline is sent without a separator after it, and a buffer ending
//! in a newline therefore sends nothing more. Each line goes through
//! `utf8_stravisx`, which replaces the bytes a terminal would act on with
//! their visible form, unless `-S` asks for them raw. With `-p` and a pane
//! whose screen has bracketed paste on, the whole stream is wrapped in the
//! `ESC [ 200 ~` and `ESC [ 201 ~` markers. `-d` frees the buffer afterwards,
//! whether or not anything was sent.
//!
//! Everything reaches the pane through `Stream::write`, which only fills the
//! pane's output buffer; the event loop is what later writes it to the pane's
//! descriptor.
//!
//! Quirks kept: a pane with `PANE_INPUTOFF` is sent nothing at all, and the
//! bracketed-paste markers are not sent either, but `-d` still frees the
//! buffer; and `-p` is read before the exited-pane refusal, which no
//! observable behaviour depends on.

use crate::args::arguments_trait::Arguments as _;
use crate::args::RustArguments;

use crate::args::args_parse_t;
use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{CMD_AFTERHOOK, CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL};
use crate::fmt_args;
use crate::paste::{PasteBufferStore, with_paste_buffers, with_paste_buffers_mut};
use ::core::ffi::{CStr, c_char};
use std::ffi::CString;

pub(crate) static cmd_paste_buffer_entry: RustCommandEntry = RustCommandEntry {
    name: c"paste-buffer",
    alias: Some(c"pasteb"),
    args: args_parse_t {
        template: c"db:prSs:t:",
        lower: 0,
        upper: 0,
        cb: None,
    },
    usage: c"[-dprS] [-s separator] [-b buffer-name] [-t target-pane]",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    flags: CMD_AFTERHOOK,
    exec: cmd_paste_buffer_exec,
};

/// The separator that follows every newline-closed line: `-s`'s text when it
/// was given, a newline under `-r`, and a carriage return otherwise.
fn separator(args: &RustArguments) -> &CStr {
    if let Some(sepstr) = args.argument_flag_string(b's') {
        sepstr
    } else if args.argument_flag_count(b'r') != 0 {
        c"\n"
    } else {
        c"\r"
    }
}

/// The buffer the command is to send: the one `-b` names, or the top of the
/// store when it was not given, which is nothing at all when the store is
/// empty. A `-b` that names no buffer is the error.
unsafe fn wanted_buffer(
    args: &RustArguments,
    item: &cmdq_item,
) -> Result<Option<(CString, Vec<u8>)>, ()> {
    let bufname = if args.argument_flag_count(b'b') != 0 {
        args.argument_flag_string(b'b')
    } else {
        None
    };
    let selected = with_paste_buffers(|buffers| {
        let buffer = match bufname {
            Some(name) => buffers.get(name),
            None => buffers.top(),
        }?;
        Some((buffer.name.to_owned(), buffer.data.to_vec()))
    });
    if selected.is_none()
        && let Some(bufname) = bufname
    {
        unsafe { item.error(c"no buffer %s", fmt_args![bufname]) };
        return Err(());
    }
    Ok(selected)
}

unsafe fn cmd_paste_buffer_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = crate::Command::command_arguments(self_0).expect("the command carries arguments");
    let pane = item
        .target
        .pane_ref()
        .expect("the command target has a pane");
    let bracket = args.argument_flag_count(b'p') != 0;

    if unsafe { pane.has_exited() != Some(false) } {
        unsafe { item.error(c"target pane has exited", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }

    let Ok(buffer) = (unsafe { wanted_buffer(args, item) }) else {
        return CMD_RETURN_ERROR;
    };

    if let Some((_, bytes)) = buffer.as_ref() {
        unsafe {
            pane.paste_buffer(
                bytes,
                separator(args).to_bytes(),
                args.argument_flag_count(b'S') != 0,
                bracket,
            )
        };
    }

    if let Some((name, _)) = buffer
        && args.argument_flag_count(b'd') != 0
    {
        with_paste_buffers_mut(|buffers| buffers.remove(name.as_c_str()));
    }
    CMD_RETURN_NORMAL
}

#[cfg(test)]
#[path = "../../tests/test_cmd_paste_buffer.rs"]
mod tests;

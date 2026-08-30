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
//!
//! Coverage exemptions: none. The message-protocol constants below are not
//! this module's own, but `test_coverage_cmd_paste_buffer` reads and pins
//! them through it, so they stay where the transpiler put them.
use crate::arguments::{args_get, args_has};
use crate::cmd::cmd_get_args;
use crate::cmd::queue::{cmdq_error, cmdq_get_target};
use crate::fmt_args;
use crate::paste::{paste_buffer_data, paste_free, paste_get_name, paste_get_top};
use crate::text::utf8_stravisx;
pub use crate::types::*;
use crate::window::window_pane_exited;
use ::core::ffi::{CStr, c_char, c_int};
use ::core::ptr::{null, null_mut};
pub const MSG_READ_CANCEL: msgtype = 307;
pub const MSG_READ_DONE: msgtype = 302;
pub const MSG_READ: msgtype = 301;
pub const MSG_READ_OPEN: msgtype = 300;
pub const MSG_FLAGS: msgtype = 218;
pub const MSG_EXEC: msgtype = 217;
pub const MSG_WAKEUP: msgtype = 216;
pub const MSG_SUSPEND: msgtype = 214;
pub const MSG_SHUTDOWN: msgtype = 210;
pub const MSG_SHELL: msgtype = 209;
pub const MSG_RESIZE: msgtype = 208;
pub const MSG_READY: msgtype = 207;
pub const MSG_LOCK: msgtype = 206;
pub const MSG_EXITED: msgtype = 204;
pub const MSG_DETACHKILL: msgtype = 202;
pub const MSG_COMMAND: msgtype = 200;
pub const MSG_IDENTIFY_TERMINFO: msgtype = 112;
pub const MSG_IDENTIFY_LONGFLAGS: msgtype = 111;
pub const MSG_IDENTIFY_CWD: msgtype = 108;
pub const MSG_IDENTIFY_FLAGS: msgtype = 100;
pub const MSG_VERSION: msgtype = 12;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const VIS_SAFE: c_int = 0x20;
pub const VIS_NOSLASH: c_int = 0x40;
pub const MODE_BRACKETPASTE: c_int = 0x400;
pub const PANE_INPUTOFF: c_int = 0x40;
pub const CMD_AFTERHOOK: c_int = 0x4;
pub(crate) static cmd_paste_buffer_entry: cmd_entry = cmd_entry {
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

/// Hands `bytes` to the pane's output buffer as they are.
unsafe fn write_bytes(wp: *mut window_pane, bytes: &[u8]) {
    unsafe {
        (*wp).event.write(bytes.as_ptr(), bytes.len());
    }
}

/// Hands `buf` to the pane in the visible form `utf8_stravisx` builds, which
/// is what keeps a pasted control byte from being acted on by the program in
/// the pane.
unsafe fn cmd_paste_buffer_paste(wp: *mut window_pane, buf: &[u8]) {
    unsafe {
        let visible = utf8_stravisx(
            buf.as_ptr() as *const c_char,
            buf.len(),
            VIS_SAFE | VIS_NOSLASH,
        );
        let bytes = visible.as_bytes();
        (*wp).event.write(bytes.as_ptr(), bytes.len());
    }
}

/// Sends one line to the pane: `-S` hands the bytes over as they are, and
/// without it they go in their visible form first.
unsafe fn send_line(wp: *mut window_pane, line: &[u8], raw: bool) {
    unsafe {
        if raw {
            write_bytes(wp, line);
        } else {
            cmd_paste_buffer_paste(wp, line);
        }
    }
}

/// The separator that follows every newline-closed line: `-s`'s text when it
/// was given, a newline under `-r`, and a carriage return otherwise.
unsafe fn separator(args: &args) -> &'static CStr {
    unsafe {
        let sepstr = args_get(args, b's');
        if !sepstr.is_null() {
            CStr::from_ptr(sepstr)
        } else if args_has(args, b'r') != 0 {
            c"\n"
        } else {
            c"\r"
        }
    }
}

/// The bytes `pb` holds. A buffer in the store always holds at least one
/// byte, since `paste_set` frees an empty one instead of storing it.
unsafe fn buffer_bytes(pb: *mut paste_buffer) -> &'static [u8] {
    unsafe { paste_buffer_data(&*pb) }
}

/// The buffer the command is to send: the one `-b` names, or the top of the
/// store when it was not given, which is nothing at all when the store is
/// empty. A `-b` that names no buffer is the error.
unsafe fn wanted_buffer(args: &args, item: *mut cmdq_item) -> Result<*mut paste_buffer, ()> {
    unsafe {
        let bufname = if args_has(args, b'b') != 0 {
            args_get(args, b'b')
        } else {
            null()
        };
        if bufname.is_null() {
            return Ok(paste_get_top(null_mut()));
        }
        let pb = paste_get_name(bufname);
        if pb.is_null() {
            cmdq_error(item, c"no buffer %s".as_ptr(), fmt_args![bufname]);
            return Err(());
        }
        Ok(pb)
    }
}

unsafe fn cmd_paste_buffer_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let wp = (*cmdq_get_target(item)).pane();
        let bracket = args_has(args, b'p') != 0;

        if window_pane_exited(wp) != 0 {
            cmdq_error(item, c"target pane has exited".as_ptr(), fmt_args![]);
            return CMD_RETURN_ERROR;
        }

        let Ok(pb) = wanted_buffer(args, item) else {
            return CMD_RETURN_ERROR;
        };

        if !pb.is_null() && (*wp).flags & PANE_INPUTOFF == 0 {
            let sep = separator(args).to_bytes();
            let raw = args_has(args, b'S') != 0;
            let wrap = bracket && (*(*wp).screen()).mode & MODE_BRACKETPASTE != 0;

            if wrap {
                write_bytes(wp, b"\x1b[200~");
            }
            for line in buffer_bytes(pb).split_inclusive(|&b| b == b'\n') {
                match line.strip_suffix(b"\n") {
                    Some(head) => {
                        send_line(wp, head, raw);
                        write_bytes(wp, sep);
                    }
                    None => send_line(wp, line, raw),
                }
            }
            if wrap {
                write_bytes(wp, b"\x1b[201~");
            }
        }

        if !pb.is_null() && args_has(args, b'd') != 0 {
            paste_free(pb);
        }
        CMD_RETURN_NORMAL
    }
}

#[cfg(test)]
#[path = "../tests/test_cmd_paste_buffer.rs"]
mod tests;

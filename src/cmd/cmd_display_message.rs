//! `display-message`: shows a message, expanded from a format, on a client.
//!
//! The message is the command's own argument, or the template `-F` gives, or —
//! with neither — [`DISPLAY_MESSAGE_TEMPLATE`], the session, window and pane
//! line with a clock on the end. It is expanded against a format tree filled
//! from the target's session, winlink and pane and from the best client of
//! that session, unless `-l` takes the template literally instead.
//!
//! Where the expansion goes depends on who is asking. An item with no client
//! behind it at all — the config loader — files it as a cause; `-p` prints it
//! back through the command queue; a control client is sent a `%message` line;
//! and anyone else gets it on the status line, where `-d` says for how long,
//! `-N` marks it as one keys may not dismiss and `-C` leaves the terminal
//! unfrozen. `-a` short-circuits all of that and prints every format entry
//! instead, and `-I` is not about messages at all: it hands the pane over to
//! the client's own standard input.
//!
//! Quirks kept:
//!
//! * A message with nowhere to go is silently thrown away. The last arm of the
//!   delivery chain has no else, so an item that has a client but no target
//!   client — a `-c` the finder gave up on, which the entry's
//!   `CMD_CLIENT_CANFAIL` allows — expands the message and frees it again.
//! * `-d` is read as a number up to `UINT_MAX` and then kept in an `int`, so a
//!   delay above `INT_MAX` comes out negative: the message stays up for good,
//!   the way `-d 0` asks for, and the one delay that wraps to exactly -1,
//!   `-d 4294967295`, means "no `-d` at all" and falls back to the
//!   `display-time` option.
//! * The `-F`-and-an-argument refusal is checked before `-d` is read, so a
//!   command carrying both reports the clash and never mentions a bad delay.
//! * `-a` is decided after the format tree has been filled but before the
//!   message is built, so `-a` with `-F`, `-l` or `-d` ignores them without
//!   complaint — the `-F`-and-an-argument clash above being the exception.
//! * The target client's flags are read to pick the control-channel arm
//!   without checking that a client asked for anything, and `-p` reads the
//!   item's own client rather than the target one.
//!
//! Coverage exemptions: none.

use crate::arguments::{args_count, args_get, args_has, args_string, args_strtonum};
use crate::cmd::cmd_get_args;
use crate::cmd::find::cmd_find_best_client;
use crate::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_target, cmdq_get_target_client, cmdq_print,
};
use crate::fmt_args;
use crate::fmt_engine::format_buf;
use crate::format::{format_create, format_defaults, format_each, format_expand_time};
use crate::server::server_client_print;
use crate::status::status_message_set;
pub use crate::types::*;
use crate::window::window_pane_start_input;
use ::core::ffi::{CStr, c_char, c_int, c_longlong};
use ::core::ptr::null_mut;

pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const CMD_FIND_CANFAIL: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_CLIENT_CFLAG: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CMD_CLIENT_CANFAIL: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const FORMAT_VERBOSE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const FORMAT_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

/// What a `display-message` with neither an argument nor `-F` shows.
pub const DISPLAY_MESSAGE_TEMPLATE: [::core::ffi::c_char; 96] = unsafe {
    ::core::mem::transmute::<
        [u8; 96],
        [::core::ffi::c_char; 96],
    >(
        *b"[#{session_name}] #{window_index}:#{window_name}, current pane #{pane_index} - (%H:%M %d-%b-%y)\0",
    )
};

pub(crate) static cmd_display_message_entry: cmd_entry = cmd_entry {
    name: c"display-message",
    alias: Some(c"display"),
    args: args_parse_t {
        template: c"aCc:d:lINpt:F:v",
        lower: 0,
        upper: 1,
        cb: None,
    },
    usage: c"[-aCIlNpv] [-c target-client] [-d delay] [-F format] [-t target-pane] [message]",
    source: cmd_entry_flag {
        flag: 0,
        type_0: CMD_FIND_PANE,
        flags: 0,
    },
    target: cmd_entry_flag {
        flag: b't' as c_char,
        type_0: CMD_FIND_PANE,
        flags: CMD_FIND_CANFAIL,
    },
    flags: CMD_AFTERHOOK | CMD_CLIENT_CFLAG | CMD_CLIENT_CANFAIL,
    exec: cmd_display_message_exec,
};

/// One entry of the format tree `-a` walks, printed back through the command
/// queue. `arg` is the item, which is what the walk was started with.
unsafe fn cmd_display_message_each(
    key: &::core::ffi::CStr,
    value: &::core::ffi::CStr,
    arg: *mut cmdq_item,
) {
    unsafe {
        cmdq_print(
            arg,
            c"%s=%s".as_ptr(),
            fmt_args![key.as_ptr(), value.as_ptr()],
        );
    }
}

/// The `-I` path: hands `wp` over to the client behind `item` as its standard
/// input, and answers what the command should.
///
/// `window_pane_start_input` gives back exactly three things — -1 with a cause
/// it allocated for a pane that already holds content, 1 for a client that
/// cannot take input at all (dead, exiting, or still holding a session), and 0
/// once the read has been handed to the client's peer — so each of them ends
/// the command here. The transpiled fourth arm, which fell through to the rest
/// of exec, could not be reached and is gone with the rewrite.
unsafe fn cmd_display_message_input(wp: *mut window_pane, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        if wp.is_null() {
            return CMD_RETURN_NORMAL;
        }
        match window_pane_start_input(wp, item) {
            Err(cause) => {
                cmdq_error(item, c"%s".as_ptr(), fmt_args![cause.as_ptr()]);
                CMD_RETURN_ERROR
            }
            Ok(1) => CMD_RETURN_NORMAL,
            Ok(_) => CMD_RETURN_WAIT,
        }
    }
}

/// How long `-d` asks the message to stay up, in milliseconds, or the cause
/// the number parser gave for turning it down. -1, which is what a command
/// with no `-d` answers, means the `display-time` option decides.
///
/// The accepted range is a `long long` up to `UINT_MAX`, and the answer is an
/// `int`, so the top half of that range wraps negative exactly as upstream's
/// does.
fn cmd_display_message_delay(args: &args) -> Result<c_int, ::std::ffi::CString> {
    if args_has(args, b'd') == 0 {
        return Ok(-1);
    }
    let mut cause = None;
    let delay = args_strtonum(args, b'd', 0, UINT_MAX as c_longlong, &mut cause);
    match cause {
        None => Ok(delay as c_int),
        Some(cause) => Err(cause),
    }
}

/// Hands the expanded `msg` to whoever is meant to see it.
///
/// The four arms are tried in order: an item with no client behind it files
/// the text as a config cause, `-p` prints it back through the queue, a
/// control client is sent a `%message` line, and anything else is a status
/// line. There is no fifth arm, so a message with no target client and no `-p`
/// goes nowhere.
unsafe fn cmd_display_message_show(
    item: *mut cmdq_item,
    tc: *mut client,
    args: &args,
    msg: *const c_char,
    delay: c_int,
) {
    unsafe {
        if cmdq_get_client(&*item).is_null() {
            cmdq_error(item, c"%s".as_ptr(), fmt_args![msg]);
        } else if args_has(args, b'p') != 0 {
            cmdq_print(item, c"%s".as_ptr(), fmt_args![msg]);
        } else if !tc.is_null() && (*tc).flags & CLIENT_CONTROL as uint64_t != 0 {
            let mut evb = Buf::new();
            format_buf(&mut evb, c"%%message %s".as_ptr(), fmt_args![msg]);
            server_client_print(tc, 0, &mut evb);
        } else if !tc.is_null() {
            status_message_set(
                tc,
                delay,
                0,
                args_has(args, b'N'),
                args_has(args, b'C'),
                c"%s".as_ptr(),
                fmt_args![msg],
            );
        }
    }
}

unsafe fn cmd_display_message_exec(self_0: &cmd, item: *mut cmdq_item) -> cmd_retval {
    unsafe {
        let args = cmd_get_args(self_0);
        let target = cmdq_get_target(item);
        let tc = cmdq_get_target_client(&*item);
        let s = (*target).session();
        let wl = (*target).winlink();
        let wp = (*target).pane();
        let count = args_count(args);

        if args_has(args, b'I') != 0 {
            return cmd_display_message_input(wp, item);
        }
        if args_has(args, b'F') != 0 && count != 0 {
            cmdq_error(
                item,
                c"only one of -F or argument must be given".as_ptr(),
                fmt_args![],
            );
            return CMD_RETURN_ERROR;
        }
        let delay = match cmd_display_message_delay(args) {
            Ok(delay) => delay,
            Err(cause) => {
                cmdq_error(item, c"delay %s".as_ptr(), fmt_args![cause.as_ptr()]);
                return CMD_RETURN_ERROR;
            }
        };

        let mut template = match count {
            0 => args_get(args, b'F'),
            _ => args_string(args, 0),
        };
        if template.is_null() {
            template = DISPLAY_MESSAGE_TEMPLATE.as_ptr();
        }

        let c = if !tc.is_null() && (*tc).session == s {
            tc
        } else if !s.is_null() {
            cmd_find_best_client(s)
        } else {
            null_mut::<client>()
        };

        let flags = match args_has(args, b'v') {
            0 => 0,
            _ => FORMAT_VERBOSE,
        };
        let mut ft = format_create(cmdq_get_client(&*item), item, FORMAT_NONE, flags);
        format_defaults(&mut ft, c, s, wl, wp);

        if args_has(args, b'a') != 0 {
            format_each(&mut ft, Some(cmd_display_message_each), item);
            return CMD_RETURN_NORMAL;
        }

        let msg = match args_has(args, b'l') {
            0 => format_expand_time(&mut ft, CStr::from_ptr(template)),
            _ => CStr::from_ptr(template).to_owned(),
        };
        cmd_display_message_show(item, tc, args, msg.as_ptr(), delay);
        CMD_RETURN_NORMAL
    }
}

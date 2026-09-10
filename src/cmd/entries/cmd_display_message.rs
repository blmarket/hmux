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

use crate::args::RustArguments;
use crate::args::{args_strtonum};
use crate::cmd::cmd_find_best_client_for_session;
use crate::cmd::cmd_get_args;

use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::fmt_args;
use crate::fmt_engine::format_buf;
use crate::format::{
    format_create_for_client, format_defaults_for_handles, format_each, format_expand_time,
};
use crate::server::client_print_buffer;
use crate::status::status_message_for_client;
use crate::types::{ByteBuffer, ClientRef, RustWindowPaneWeak, args_parse_t, uint64_t};
use ::core::ffi::{CStr, c_char, c_int, c_longlong};

use crate::consts::{
    CLIENT_CONTROL, CMD_AFTERHOOK, CMD_CLIENT_CANFAIL, CMD_CLIENT_CFLAG, CMD_FIND_CANFAIL,
    CMD_FIND_PANE, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMD_RETURN_WAIT, FORMAT_NONE,
    FORMAT_VERBOSE, UINT_MAX,
};

/// What a `display-message` with neither an argument nor `-F` shows.
pub const DISPLAY_MESSAGE_TEMPLATE: &CStr = c"[#{session_name}] #{window_index}:#{window_name}, current pane #{pane_index} - (%H:%M %d-%b-%y)";

pub(crate) static cmd_display_message_entry: RustCommandEntry = RustCommandEntry {
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

/// The `-I` path: hands `wp` over to the client behind `item` as its standard
/// input, and answers what the command should.
///
/// `window_pane_start_input` gives back exactly three things — -1 with a cause
/// it allocated for a pane that already holds content, 1 for a client that
/// cannot take input at all (dead, exiting, or still holding a session), and 0
/// once the read has been handed to the client's peer — so each of them ends
/// the command here. The transpiled fourth arm, which fell through to the rest
/// of exec, could not be reached and is gone with the rewrite.
unsafe fn cmd_display_message_input(
    wp: Option<&RustWindowPaneWeak>,
    item: &cmdq_item,
) -> cmd_retval {
    unsafe {
        let Some(wp) = wp else {
            return CMD_RETURN_NORMAL;
        };
        match wp.start_input(&crate::cmd::cmdq_item_ref_of(item).expect("the command has an owner"))
        {
            Err(cause) => {
                item.error(c"%s", fmt_args![cause.as_c_str()]);
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
fn cmd_display_message_delay(args: &RustArguments) -> Result<c_int, std::ffi::CString> {
    if args.argument_flag_count(b'd') == 0 {
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
    item: &cmdq_item,
    tc: Option<&mut ClientRef>,
    args: &RustArguments,
    msg: &CStr,
    delay: c_int,
) {
    unsafe {
        if item.client().is_none() {
            item.error(c"%s", fmt_args![msg]);
        } else if args.argument_flag_count(b'p') != 0 {
            item.print(c"%s", fmt_args![msg]);
        } else if tc
            .as_deref()
            .is_some_and(|tc| tc.flags() & CLIENT_CONTROL as uint64_t != 0)
        {
            let mut evb = ByteBuffer::new();
            format_buf(&mut evb, c"%%message %s", fmt_args![msg]);
            client_print_buffer(tc, 0, &mut evb);
        } else if let Some(tc) = tc {
            status_message_for_client(
                Some(tc),
                delay,
                0,
                args.argument_flag_count(b'N'),
                args.argument_flag_count(b'C'),
                c"%s",
                fmt_args![msg],
            );
        }
    }
}

unsafe fn cmd_display_message_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args = cmd_get_args(self_0);
    let target_client = item.target_client();
    let mut tc = target_client.clone();
    let session = item.target.session();
    let link = item.target.winlink_ref();
    let pane = item.target.pane_ref();
    let count = args.argument_count();

    if args.argument_flag_count(b'I') != 0 {
        return unsafe { cmd_display_message_input(pane.as_ref(), item) };
    }
    if args.argument_flag_count(b'F') != 0 && count != 0 {
        unsafe { item.error(c"only one of -F or argument must be given", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }
    let delay = match cmd_display_message_delay(args) {
        Ok(delay) => delay,
        Err(cause) => {
            unsafe { item.error(c"delay %s", fmt_args![cause.as_c_str()]) };
            return CMD_RETURN_ERROR;
        }
    };

    let given = match count {
        0 => args.argument_flag_string(b'F'),
        _ => args.argument_string(0),
    };
    let template = given.unwrap_or(DISPLAY_MESSAGE_TEMPLATE);

    let c = if target_client
        .as_ref()
        .is_some_and(|tc| match session.as_ref() {
            Some(session) => tc
                .attached_session()
                .is_some_and(|current| session.ptr_eq(&current)),
            None => tc.attached_session().is_none(),
        }) {
        target_client.clone()
    } else {
        session
            .as_ref()
            .and_then(|session| unsafe { cmd_find_best_client_for_session(session) })
    };

    let flags = match args.argument_flag_count(b'v') {
        0 => 0,
        _ => FORMAT_VERBOSE,
    };
    let mut ft = format_create_for_client(item.client().as_ref(), Some(item), FORMAT_NONE, flags);
    unsafe {
        format_defaults_for_handles(
            &mut ft,
            c.as_ref(),
            session.as_ref(),
            link.as_ref(),
            pane.as_ref(),
        )
    };

    if args.argument_flag_count(b'a') != 0 {
        unsafe {
            format_each(&mut ft, |key, value| {
                (*item).print(c"%s=%s", fmt_args![key, value]);
            });
        }
        return CMD_RETURN_NORMAL;
    }

    let msg = match args.argument_flag_count(b'l') {
        0 => unsafe { format_expand_time(&mut ft, template) },
        _ => template.to_owned(),
    };
    unsafe { cmd_display_message_show(item, tc.as_mut(), args, &msg, delay) };
    CMD_RETURN_NORMAL
}

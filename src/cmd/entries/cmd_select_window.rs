use crate::args::arguments_trait::Arguments as _;
use crate::args::RustArguments;
use crate::args::args_parse_t;
use crate::cmd::cmd_find_from_session_ref;

use crate::fmt_args;
use crate::resize::recalculate_sizes;

use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{
    CMD_FIND_PANE, CMD_FIND_SESSION, CMD_FIND_WINDOW, CMD_RETURN_ERROR, CMD_RETURN_NORMAL,
    CMD_TARGET_SESSION_USAGE,
};
use crate::types::u_char;

pub(crate) static cmd_select_window_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"select-window",
        alias: Some(c"selectw"),
        args: args_parse_t {
            template: c"lnpTt:",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-lnpT] [-t target-window]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as core::ffi::c_int,
        },
        flags: 0 as core::ffi::c_int,
        exec: cmd_select_window_exec,
    }
};
pub(crate) static cmd_next_window_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"next-window",
        alias: Some(c"next"),
        args: args_parse_t {
            template: c"at:",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-a] [-t target-session]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as core::ffi::c_int,
        },
        flags: 0 as core::ffi::c_int,
        exec: cmd_select_window_exec,
    }
};
pub(crate) static cmd_previous_window_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"previous-window",
        alias: Some(c"prev"),
        args: args_parse_t {
            template: c"at:",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-a] [-t target-session]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as core::ffi::c_int,
        },
        flags: 0 as core::ffi::c_int,
        exec: cmd_select_window_exec,
    }
};
pub(crate) static cmd_last_window_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"last-window",
        alias: Some(c"last"),
        args: args_parse_t {
            template: c"t:",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_SESSION_USAGE,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as core::ffi::c_int,
        },
        flags: 0 as core::ffi::c_int,
        exec: cmd_select_window_exec,
    }
};
unsafe fn cmd_select_window_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &RustArguments = crate::Command::command_arguments(self_0).expect("the command carries arguments");
    let c = item.client();
    let current_state_ref = item.state_ref();
    let mut current = current_state_ref.current_snapshot();
    let target_index = item.target.winlink_ref().map(|link| link.index());
    let session = item
        .target
        .session()
        .expect("the command target has a session");
    let mut next: core::ffi::c_int;
    let mut previous: core::ffi::c_int;
    let mut last: core::ffi::c_int;
    let activity: core::ffi::c_int;
    next = core::ptr::eq(crate::Command::command_entry(self_0), &cmd_next_window_entry) as core::ffi::c_int;
    if ({
        let flag = 'n' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        next = 1 as core::ffi::c_int;
    }
    previous = core::ptr::eq(crate::Command::command_entry(self_0), &cmd_previous_window_entry) as core::ffi::c_int;
    if ({
        let flag = 'p' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        previous = 1 as core::ffi::c_int;
    }
    last = core::ptr::eq(crate::Command::command_entry(self_0), &cmd_last_window_entry) as core::ffi::c_int;
    if ({
        let flag = 'l' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        last = 1 as core::ffi::c_int;
    }
    if next != 0 || previous != 0 || last != 0 {
        activity = {
            let flag = 'a' as i32 as u_char;
            args.argument_flag_count(flag)
        };
        if next != 0 {
            if unsafe { session.next(activity) != 0 as core::ffi::c_int } {
                unsafe { item.error(c"no next window", fmt_args![]) };
                return CMD_RETURN_ERROR;
            }
        } else if previous != 0 {
            if unsafe { session.previous(activity) != 0 as core::ffi::c_int } {
                unsafe { item.error(c"no previous window", fmt_args![]) };
                return CMD_RETURN_ERROR;
            }
        } else if unsafe { session.last() != 0 as core::ffi::c_int } {
            unsafe { item.error(c"no last window", fmt_args![]) };
            return CMD_RETURN_ERROR;
        }
        unsafe { cmd_find_from_session_ref(&mut current, &session, 0 as core::ffi::c_int) };
        unsafe { session.request_redraw() };
    } else {
        if ({
            let flag = 'T' as i32 as u_char;
            args.argument_flag_count(flag)
        }) != 0
            && target_index == session.curw().map(|link| link.index())
        {
            if unsafe { session.last() != 0 as core::ffi::c_int } {
                unsafe { item.error(c"no last window", fmt_args![]) };
                return CMD_RETURN_ERROR;
            }
            if current
                .session()
                .is_some_and(|current| current.ptr_eq(&session))
            {
                unsafe { cmd_find_from_session_ref(&mut current, &session, 0 as core::ffi::c_int) };
            }
            unsafe { session.request_redraw() };
        } else if unsafe {
            session.select(target_index.expect("the command target has a window link"))
                == 0 as core::ffi::c_int
        } {
            unsafe { cmd_find_from_session_ref(&mut current, &session, 0 as core::ffi::c_int) };
            unsafe { session.request_redraw() };
        }
    }
    current_state_ref.replace_current(current.clone());
    unsafe {
        (crate::cmd::cmdq_item_ref_of(item).expect("the command has an owner")).insert_session_hook(
            Some(&session),
            Some(&current),
            c"after-select-window",
            fmt_args![],
        )
    };
    if let Some(client) = c.as_ref()
        && { !client.attached_session().is_none() }
        && let Some(window) = { session.current_window() }
    {
        unsafe { window.set_latest_client(Some(client)) };
    }
    recalculate_sizes();
    CMD_RETURN_NORMAL
}

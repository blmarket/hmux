use crate::src::arguments::args_has;
use crate::src::cmd::find::cmd_find_from_session;
use crate::src::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_current, cmdq_get_target, cmdq_insert_hook,
};
use crate::src::cmd::{cmd_get_args_mut, cmd_get_entry};
use crate::src::resize::recalculate_sizes;
use crate::src::server_fn::server_redraw_session;
use crate::src::session::{session_last, session_next, session_previous, session_select};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::client::client;
use crate::src::shared::command::CMD_TARGET_SESSION_USAGE;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::session::session;
use crate::src::shared::window::winlink;
pub static cmd_select_window_entry: cmd_entry = {
    cmd_entry {
        name: c"select-window",
        alias: Some(c"selectw"),
        args: args_parse {
            template: c"lnpTt:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-lnpT] [-t target-window]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(cmd_select_window_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static cmd_next_window_entry: cmd_entry = {
    cmd_entry {
        name: c"next-window",
        alias: Some(c"next"),
        args: args_parse {
            template: c"at:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-a] [-t target-session]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(cmd_select_window_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static cmd_previous_window_entry: cmd_entry = {
    cmd_entry {
        name: c"previous-window",
        alias: Some(c"prev"),
        args: args_parse {
            template: c"at:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-a] [-t target-session]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(cmd_select_window_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static cmd_last_window_entry: cmd_entry = {
    cmd_entry {
        name: c"last-window",
        alias: Some(c"last"),
        args: args_parse {
            template: c"t:",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_SESSION_USAGE,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(cmd_select_window_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
unsafe fn cmd_select_window_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args_mut(&mut *self_0).map_or(std::ptr::null_mut(), |args| args);
    let mut c: *mut client = cmdq_get_client(item);
    let mut current: *mut cmd_find_state = cmdq_get_current(item);
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut wl: *mut winlink = (*target).wl_ptr();
    let mut s: *mut session = (*target).s_ptr();
    let mut next: ::core::ffi::c_int = 0;
    let mut previous: ::core::ffi::c_int = 0;
    let mut last: ::core::ffi::c_int = 0;
    let mut activity: ::core::ffi::c_int = 0;
    next = (std::ptr::eq(cmd_get_entry(&*self_0), &cmd_next_window_entry)) as ::core::ffi::c_int;
    if args_has(args, 'n' as i32 as u_char) != 0 {
        next = 1 as ::core::ffi::c_int;
    }
    previous =
        (std::ptr::eq(cmd_get_entry(&*self_0), &cmd_previous_window_entry)) as ::core::ffi::c_int;
    if args_has(args, 'p' as i32 as u_char) != 0 {
        previous = 1 as ::core::ffi::c_int;
    }
    last = (std::ptr::eq(cmd_get_entry(&*self_0), &cmd_last_window_entry)) as ::core::ffi::c_int;
    if args_has(args, 'l' as i32 as u_char) != 0 {
        last = 1 as ::core::ffi::c_int;
    }
    if next != 0 || previous != 0 || last != 0 {
        activity = args_has(args, 'a' as i32 as u_char);
        if next != 0 {
            if session_next(s, activity) != 0 as ::core::ffi::c_int {
                cmdq_error(item, |out| out.write_all(b"no next window"));
                return CMD_RETURN_ERROR;
            }
        } else if previous != 0 {
            if session_previous(s, activity) != 0 as ::core::ffi::c_int {
                cmdq_error(item, |out| out.write_all(b"no previous window"));
                return CMD_RETURN_ERROR;
            }
        } else if session_last(s) != 0 as ::core::ffi::c_int {
            cmdq_error(item, |out| out.write_all(b"no last window"));
            return CMD_RETURN_ERROR;
        }
        cmd_find_from_session(current, s, 0 as ::core::ffi::c_int);
        server_redraw_session(s);
        cmdq_insert_hook(s, item, current, |out| {
            out.write_all(b"after-select-window")
        });
    } else {
        if args_has(args, 'T' as i32 as u_char) != 0 && wl == (*s).curw {
            if session_last(s) != 0 as ::core::ffi::c_int {
                cmdq_error(item, |out| out.write_all(b"no last window"));
                return CMD_RETURN_ERROR;
            }
            if (*current).s_ptr() == s {
                cmd_find_from_session(current, s, 0 as ::core::ffi::c_int);
            }
            server_redraw_session(s);
        } else if session_select(s, (*wl).idx) == 0 as ::core::ffi::c_int {
            cmd_find_from_session(current, s, 0 as ::core::ffi::c_int);
            server_redraw_session(s);
        }
        cmdq_insert_hook(s, item, current, |out| {
            out.write_all(b"after-select-window")
        });
    }
    if !c.is_null() && !(*c).session.is_null() {
        (*(*(*s).curw).window_ptr()).latest = c as *mut ::core::ffi::c_void;
    }
    recalculate_sizes();
    return CMD_RETURN_NORMAL;
}

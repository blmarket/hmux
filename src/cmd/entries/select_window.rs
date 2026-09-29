use crate::src::arguments::args_has;
use crate::src::cmd::find::cmd_find_from_session;
use crate::src::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_state_owned, cmdq_get_target, cmdq_insert_hook,
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
        exec: Some(cmd_select_window_exec),
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
        exec: Some(cmd_select_window_exec),
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
        exec: Some(cmd_select_window_exec),
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
        exec: Some(cmd_select_window_exec),
    }
};
unsafe fn cmd_select_window_exec(mut self_0: refbox::Weak<cmd>, item_handle: &std::rc::Rc<std::cell::UnsafeCell<cmdq_item>>) -> cmd_retval {
    let item = item_handle.get();
    let mut args: *mut args = cmd_get_args_mut(self_0.get_mut_unchecked()).map_or(std::ptr::null_mut(), |args| args);
    let c_owner = cmdq_get_client((item).as_ref());
    let mut c: *mut client = c_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    let current = cmdq_get_state_owned(&*(item));
    let mut target: *mut cmd_find_state = crate::src::cmd::queue::cmdq_get_target_mut(&mut *item);
    let mut wl: refbox::Weak<winlink> = (*target).winlink_handle();
    let mut s: *mut session = (*target).session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get());
    let mut next: ::core::ffi::c_int = 0;
    let mut previous: ::core::ffi::c_int = 0;
    let mut last: ::core::ffi::c_int = 0;
    let mut activity: ::core::ffi::c_int = 0;
    next = (std::ptr::eq(cmd_get_entry(self_0.get_unchecked()), &cmd_next_window_entry)) as ::core::ffi::c_int;
    if args_has(args, 'n' as i32 as u_char) != 0 {
        next = 1 as ::core::ffi::c_int;
    }
    previous =
        (std::ptr::eq(cmd_get_entry(self_0.get_unchecked()), &cmd_previous_window_entry)) as ::core::ffi::c_int;
    if args_has(args, 'p' as i32 as u_char) != 0 {
        previous = 1 as ::core::ffi::c_int;
    }
    last = (std::ptr::eq(cmd_get_entry(self_0.get_unchecked()), &cmd_last_window_entry)) as ::core::ffi::c_int;
    if args_has(args, 'l' as i32 as u_char) != 0 {
        last = 1 as ::core::ffi::c_int;
    }
    if next != 0 || previous != 0 || last != 0 {
        activity = args_has(args, 'a' as i32 as u_char);
        if next != 0 {
            if session_next(&(*s).observer.upgrade().expect("live session"), activity) != 0 as ::core::ffi::c_int {
                cmdq_error(item_handle, |out| out.write_all(b"no next window"));
                return CMD_RETURN_ERROR;
            }
        } else if previous != 0 {
            if session_previous(&(*s).observer.upgrade().expect("live session"), activity) != 0 as ::core::ffi::c_int {
                cmdq_error(item_handle, |out| out.write_all(b"no previous window"));
                return CMD_RETURN_ERROR;
            }
        } else if session_last(&(*s).observer.upgrade().expect("live session")) != 0 as ::core::ffi::c_int {
            cmdq_error(item_handle, |out| out.write_all(b"no last window"));
            return CMD_RETURN_ERROR;
        }
        cmd_find_from_session(&mut *current.current.borrow_mut(), &(*(s)).observer.upgrade().expect("live session"), 0 as ::core::ffi::c_int);
        server_redraw_session(&*(s));
        cmdq_insert_hook((s).as_ref().and_then(|model| model.observer.upgrade()).as_ref(), item_handle, &mut current.current_snapshot(), |out| {
            out.write_all(b"after-select-window")
        });
    } else {
        if args_has(args, 'T' as i32 as u_char) != 0 && wl == (*s).current_winlink() {
            if session_last(&(*s).observer.upgrade().expect("live session")) != 0 as ::core::ffi::c_int {
                cmdq_error(item_handle, |out| out.write_all(b"no last window"));
                return CMD_RETURN_ERROR;
            }
            if current.current.borrow().session_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get()) == s {
                cmd_find_from_session(&mut *current.current.borrow_mut(), &(*(s)).observer.upgrade().expect("live session"), 0 as ::core::ffi::c_int);
            }
            server_redraw_session(&*(s));
        } else if session_select(&(*s).observer.upgrade().expect("live session"), wl.get_unchecked().idx) == 0 as ::core::ffi::c_int {
            cmd_find_from_session(&mut *current.current.borrow_mut(), &(*(s)).observer.upgrade().expect("live session"), 0 as ::core::ffi::c_int);
            server_redraw_session(&*(s));
        }
        cmdq_insert_hook((s).as_ref().and_then(|model| model.observer.upgrade()).as_ref(), item_handle, &mut current.current_snapshot(), |out| {
            out.write_all(b"after-select-window")
        });
    }
    if !c.is_null() && !(*c).session_handle().is_none() {
        (*((*s).current_winlink()).get_unchecked().window_handle().as_ref().map_or(std::ptr::null_mut(), |owner| owner.get())).latest = c.as_ref().map_or_else(std::rc::Weak::new, |client| client.observer.clone());
    }
    recalculate_sizes();
    return CMD_RETURN_NORMAL;
}

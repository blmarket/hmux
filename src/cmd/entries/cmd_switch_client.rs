use crate::args::arguments_trait::Arguments as _;
use crate::args::RustArguments;
use crate::args::args_parse_t;
use crate::cmd::cmd_find_target;
use crate::cmd::cmd_get_args;

use crate::ffi::getuid;
use crate::fmt_args;
use crate::key_bindings::key_bindings_get_table;

use crate::cmd::cmd_find_type;
use crate::cmd::cmdq_item;
use crate::cmd::{RustCommandEntry, cmd, cmd_entry_flag, cmd_retval};
use crate::consts::{
    CLIENT_READONLY, CMD_CLIENT_CFLAG, CMD_FIND_PANE, CMD_FIND_PREFER_UNATTACHED, CMD_FIND_SESSION,
    CMD_READONLY, CMD_RETURN_ERROR, CMD_RETURN_NORMAL, CMDQ_STATE_REPEAT, SORT_END,
};
use crate::sort::{RustSortCriteria, SortCriteria};
use crate::types::{cmd_find_state, sort_criteria_t, u_char, uid_t, uint64_t};

pub(crate) static cmd_switch_client_entry: RustCommandEntry = {
    RustCommandEntry {
        name: c"switch-client",
        alias: Some(c"switchc"),
        args: args_parse_t {
            template: c"c:EFlnO:pt:rT:Z",
            lower: 0 as core::ffi::c_int,
            upper: 0 as core::ffi::c_int,
            cb: None,
        },
        usage: c"[-ElnprZ] [-c target-client] [-t target-session] [-T key-table] [-O order]",
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
        flags: CMD_READONLY | CMD_CLIENT_CFLAG,
        exec: cmd_switch_client_exec,
    }
};
unsafe fn cmd_switch_client_exec(self_0: &cmd, item: &cmdq_item) -> cmd_retval {
    let args: &RustArguments = cmd_get_args(self_0);
    let current_state_ref = item.state_ref();
    let mut target = cmd_find_state::default();
    let tflag = {
        let flag = 't' as i32 as u_char;
        args.argument_flag_string(flag)
    };
    let type_0: cmd_find_type;
    let flags: core::ffi::c_int;
    let c = item.client();
    let mut tc = item.target_client();
    let mut sort_crit = sort_criteria_t::default();
    let uid: uid_t;
    if tflag.is_some_and(|tflag| {
        tflag
            .to_bytes()
            .iter()
            .any(|b| matches!(b, b':' | b'.' | b'%'))
            || tflag == c"="
    }) {
        type_0 = CMD_FIND_PANE;
        flags = 0 as core::ffi::c_int;
    } else {
        type_0 = CMD_FIND_SESSION;
        flags = CMD_FIND_PREFER_UNATTACHED;
    }
    if unsafe { cmd_find_target(&mut target, item, tflag, type_0, flags) != 0 as core::ffi::c_int }
    {
        return CMD_RETURN_ERROR;
    }
    let mut selected_session = target
        .session()
        .expect("a resolved switch target has a session");
    let link = target.winlink_ref();
    let pane = target.pane_list_ref();
    if ({
        let flag = 'r' as i32 as u_char;
        args.argument_flag_count(flag)
    }) != 0
    {
        if unsafe {
            tc.as_ref().expect("the command has a client").flags() & CLIENT_READONLY as uint64_t
                != 0
        } {
            uid = (c.as_ref().expect("the command has a client").peer_handle()).uid();
            if unsafe { uid != getuid() } {
                unsafe { item.error(c"client is read-only", fmt_args![]) };
                return CMD_RETURN_ERROR;
            }
        }
        unsafe {
            tc.as_mut()
                .expect("the command has a client")
                .toggle_read_only()
        };
    }
    let tablename = {
        let flag = 'T' as i32 as u_char;
        args.argument_flag_string(flag)
    };
    if let Some(tablename) = tablename {
        let table = key_bindings_get_table(tablename, 0 as core::ffi::c_int);
        if table.is_none() {
            unsafe { item.error(c"table %s doesn't exist", fmt_args![tablename]) };
            return CMD_RETURN_ERROR;
        }
        unsafe {
            tc.as_mut()
                .expect("the command has a client")
                .set_key_table(Some(tablename))
        };
        return CMD_RETURN_NORMAL;
    }
    sort_crit.set_order(RustSortCriteria::parse_order({
        let flag = 'O' as i32 as u_char;
        args.argument_flag_string(flag)
    }));
    if sort_crit.order() as core::ffi::c_uint == SORT_END as core::ffi::c_int as core::ffi::c_uint
        && ({
            let flag = 'O' as i32 as u_char;
            args.argument_flag_count(flag)
        }) != 0
    {
        unsafe { item.error(c"invalid sort order", fmt_args![]) };
        return CMD_RETURN_ERROR;
    }
    sort_crit.set_reversed(
        ({
            let flag = 'r' as i32 as u_char;
            args.argument_flag_count(flag)
        }) != 0,
    );
    if args.argument_flag_count(b'n') != 0 {
        let current = {
            tc.as_ref()
                .expect("the command has a client")
                .attached_session()
        };
        let Some(found) = current
            .as_ref()
            .and_then(|session| session.next_session(&sort_crit))
        else {
            unsafe { item.error(c"can't find next session", fmt_args![]) };
            return CMD_RETURN_ERROR;
        };
        selected_session = found;
    } else if args.argument_flag_count(b'p') != 0 {
        let current = {
            tc.as_ref()
                .expect("the command has a client")
                .attached_session()
        };
        let Some(found) = current
            .as_ref()
            .and_then(|session| session.previous_session(&sort_crit))
        else {
            unsafe { item.error(c"can't find previous session", fmt_args![]) };
            return CMD_RETURN_ERROR;
        };
        selected_session = found;
    } else if args.argument_flag_count(b'l') != 0 {
        let Some(last) = (unsafe {
            tc.as_ref()
                .expect("the command has a client")
                .last_session()
                .filter(|session| session.is_registered())
        }) else {
            unsafe { item.error(c"can't find last session", fmt_args![]) };
            return CMD_RETURN_ERROR;
        };
        selected_session = last;
    } else {
        if item.client().is_none() {
            return CMD_RETURN_NORMAL;
        }
        if let (Some(link), Some(pane)) = (link.as_ref(), pane.as_ref())
            && let Some(window) = link.window()
            && {
                window
                    .active_pane()
                    .is_none_or(|active| !active.ptr_eq(pane))
            }
        {
            if unsafe { window.push_zoom(0, args.argument_flag_count(b'Z')) != 0 } {
                unsafe { window.redraw() };
            }
            unsafe {
                window.redraw_active_switch(
                    &crate::window::window_pane_find_by_id(pane.id())
                        .expect("the selected pane exists"),
                )
            };
            unsafe {
                window.set_active_pane(
                    &crate::window::window_pane_find_by_id(pane.id())
                        .expect("the selected pane exists"),
                    1,
                )
            };
            if unsafe { window.pop_zoom() != 0 } {
                unsafe { window.redraw() };
            }
        }
        if let Some(link) = link.as_ref() {
            unsafe { selected_session.set_current(Some(link.index())) };
            unsafe { current_state_ref.update_current_session(&selected_session, 0) };
        }
    }
    if ({
        let flag = 'E' as i32 as u_char;
        args.argument_flag_count(flag)
    }) == 0
    {
        unsafe {
            selected_session.update_environment_from(tc.as_ref().expect("the command has a client"))
        };
    }
    unsafe {
        tc.as_mut()
            .expect("the command has a client")
            .set_session(Some(&selected_session))
    };
    if !item.flags() & CMDQ_STATE_REPEAT != 0 {
        unsafe {
            tc.as_mut()
                .expect("the command has a client")
                .set_key_table(None)
        };
    }
    CMD_RETURN_NORMAL
}

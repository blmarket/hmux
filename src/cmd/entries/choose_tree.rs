use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target};
use crate::src::cmd::{cmd_get_args, cmd_get_entry};
use crate::src::paste::paste_is_empty;
use crate::src::server_client::server_client_how_many;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
use crate::src::shared::arguments::{args, args_parse};
use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
use crate::src::shared::command::{cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmdq_item};
use crate::src::shared::pane::window_pane;
use crate::src::shared::sort::*;
use crate::src::shared::window::window_mode;
use crate::src::sort::sort_order_from_string;
use crate::src::window::window_pane_set_mode;
use crate::src::window_buffer::window_buffer_mode;
use crate::src::window_client::window_client_mode;
use crate::src::window_customize::window_customize_mode;
use crate::src::window_panes::window_panes_mode;
use crate::src::window_switch::window_switch_mode;
use crate::src::window_tree::window_tree_mode;
pub static cmd_choose_tree_entry: cmd_entry = {
    cmd_entry {
        name: c"choose-tree",
        alias: None,
        args: args_parse {
            template: c"F:f:GhK:kNO:rst:wyZ",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: Some(
                cmd_choose_tree_args_parse
            ),
        },
        usage: c"[-GhkNrswZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_choose_tree_exec
                as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
pub static cmd_choose_client_entry: cmd_entry = {
    cmd_entry {
        name: c"choose-client",
        alias: None,
        args: args_parse {
            template: c"F:f:hiK:kNO:rt:yZ",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: Some(
                cmd_choose_tree_args_parse
            ),
        },
        usage: c"[-hikNrZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_choose_tree_exec
                as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
pub static cmd_choose_buffer_entry: cmd_entry = {
    cmd_entry {
        name: c"choose-buffer",
        alias: None,
        args: args_parse {
            template: c"F:f:K:kNO:rt:yZ",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: Some(
                cmd_choose_tree_args_parse
            ),
        },
        usage: c"[-kNrZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_choose_tree_exec
                as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
pub static cmd_customize_mode_entry: cmd_entry = {
    cmd_entry {
        name: c"customize-mode",
        alias: None,
        args: args_parse {
            template: c"F:f:kNt:yZ",
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: c"[-kNZ] [-F format] [-f filter] [-t target-pane]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(cmd_choose_tree_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static cmd_switch_mode_entry: cmd_entry = {
    cmd_entry {
        name: c"switch-mode",
        alias: None,
        args: args_parse {
            template: c"F:kst:wZ",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: Some(cmd_choose_tree_args_parse),
        },
        usage: c"[-kswZ] [-F format] [-t target-pane] [command]",
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(cmd_choose_tree_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
pub static cmd_display_panes_entry: cmd_entry = {
    cmd_entry {
        name: c"display-panes",
        alias: Some(c"displayp"),
        args: args_parse {
            template: c"d:kNs:t:Z",
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: Some(cmd_choose_tree_args_parse),
        },
        usage: c"[-kNZ] [-d duration] [-s source-window] [-t target-pane] [template]",
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(cmd_choose_tree_exec as unsafe fn(*mut cmd, *mut cmdq_item) -> cmd_retval),
    }
};
fn cmd_choose_tree_args_parse(
    _args: &mut args,
    _idx: u_int,
) -> Result<args_parse_type, ArgsParseError> {
    Ok(ARGS_PARSE_COMMANDS_OR_STRING)
}
unsafe fn cmd_choose_tree_exec(mut self_0: *mut cmd, mut item: *mut cmdq_item) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut wp: *mut window_pane = (*target).wp;
    let mode: &'static window_mode;
    let mut order: sort_order = SORT_ACTIVITY;
    order = sort_order_from_string(args_get(args, 'O' as i32 as u_char));
    if order as ::core::ffi::c_uint == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
        && args_has(args, 'O' as i32 as u_char) != 0
    {
        cmdq_error(item, |out| out.write_all(b"invalid sort order"));
        return CMD_RETURN_ERROR;
    }
    if std::ptr::eq(cmd_get_entry(&*self_0), &cmd_choose_buffer_entry) {
        if paste_is_empty() != 0 {
            return CMD_RETURN_NORMAL;
        }
        mode = &window_buffer_mode;
    } else if std::ptr::eq(cmd_get_entry(&*self_0), &cmd_choose_client_entry) {
        if server_client_how_many() == 0 as u_int {
            return CMD_RETURN_NORMAL;
        }
        mode = &window_client_mode;
    } else if std::ptr::eq(cmd_get_entry(&*self_0), &cmd_customize_mode_entry) {
        mode = &window_customize_mode;
    } else if std::ptr::eq(cmd_get_entry(&*self_0), &cmd_switch_mode_entry) {
        mode = &window_switch_mode;
    } else if std::ptr::eq(cmd_get_entry(&*self_0), &cmd_display_panes_entry) {
        mode = &window_panes_mode;
    } else {
        mode = &window_tree_mode;
    }
    window_pane_set_mode(
        wp,
        ::core::ptr::null_mut::<window_pane>(),
        mode,
        item,
        target,
        args,
    );
    return CMD_RETURN_NORMAL;
}

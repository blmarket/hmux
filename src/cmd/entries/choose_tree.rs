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

#[no_mangle]
pub static mut cmd_choose_tree_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"choose-tree\0" as *const u8 as *const ::core::ffi::c_char,
        alias: ::core::ptr::null::<::core::ffi::c_char>(),
        args: args_parse {
            template: b"F:f:GhK:kNO:rst:wyZ\0" as *const u8
                as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: Some(
                cmd_choose_tree_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-GhkNrswZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]\0"
            as *const u8 as *const ::core::ffi::c_char,
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
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_choose_client_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"choose-client\0" as *const u8 as *const ::core::ffi::c_char,
        alias: ::core::ptr::null::<::core::ffi::c_char>(),
        args: args_parse {
            template: b"F:f:hiK:kNO:rt:yZ\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: Some(
                cmd_choose_tree_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-hikNrZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]\0"
            as *const u8 as *const ::core::ffi::c_char,
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
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_choose_buffer_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"choose-buffer\0" as *const u8 as *const ::core::ffi::c_char,
        alias: ::core::ptr::null::<::core::ffi::c_char>(),
        args: args_parse {
            template: b"F:f:K:kNO:rt:yZ\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: Some(
                cmd_choose_tree_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-kNrZ] [-F format] [-f filter] [-K key-format] [-O sort-order] [-t target-pane] [template]\0"
            as *const u8 as *const ::core::ffi::c_char,
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
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_customize_mode_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"customize-mode\0" as *const u8 as *const ::core::ffi::c_char,
        alias: ::core::ptr::null::<::core::ffi::c_char>(),
        args: args_parse {
            template: b"F:f:kNt:yZ\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-kNZ] [-F format] [-f filter] [-t target-pane]\0" as *const u8
            as *const ::core::ffi::c_char,
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
            cmd_choose_tree_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_switch_mode_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"switch-mode\0" as *const u8 as *const ::core::ffi::c_char,
        alias: ::core::ptr::null::<::core::ffi::c_char>(),
        args: args_parse {
            template: b"F:kst:wZ\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: Some(
                cmd_choose_tree_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-kswZ] [-F format] [-t target-pane] [command]\0" as *const u8
            as *const ::core::ffi::c_char,
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
            cmd_choose_tree_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_display_panes_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"display-panes\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"displayp\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"d:kNs:t:Z\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: Some(
                cmd_choose_tree_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-kNZ] [-d duration] [-s source-window] [-t target-pane] [template]\0" as *const u8
            as *const ::core::ffi::c_char,
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
        exec: Some(
            cmd_choose_tree_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_choose_tree_args_parse(
    _args: *mut args,
    _idx: u_int,
    _cause: *mut *mut ::core::ffi::c_char,
) -> args_parse_type {
    return ARGS_PARSE_COMMANDS_OR_STRING;
}
unsafe extern "C" fn cmd_choose_tree_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut wp: *mut window_pane = (*target).wp;
    let mut mode: *const window_mode = ::core::ptr::null::<window_mode>();
    let mut order: sort_order = SORT_ACTIVITY;
    order = sort_order_from_string(args_get(args, 'O' as i32 as u_char));
    if order as ::core::ffi::c_uint == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
        && args_has(args, 'O' as i32 as u_char) != 0
    {
        cmdq_error(
            item,
            b"invalid sort order\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if cmd_get_entry(self_0) == &raw const cmd_choose_buffer_entry {
        if paste_is_empty() != 0 {
            return CMD_RETURN_NORMAL;
        }
        mode = &raw const window_buffer_mode;
    } else if cmd_get_entry(self_0) == &raw const cmd_choose_client_entry {
        if server_client_how_many() == 0 as u_int {
            return CMD_RETURN_NORMAL;
        }
        mode = &raw const window_client_mode;
    } else if cmd_get_entry(self_0) == &raw const cmd_customize_mode_entry {
        mode = &raw const window_customize_mode;
    } else if cmd_get_entry(self_0) == &raw const cmd_switch_mode_entry {
        mode = &raw const window_switch_mode;
    } else if cmd_get_entry(self_0) == &raw const cmd_display_panes_entry {
        mode = &raw const window_panes_mode;
    } else {
        mode = &raw const window_tree_mode;
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

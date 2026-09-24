use crate::src::arguments::{args_count, args_get, args_has, args_string};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_target};
use crate::src::environ::{environ_clear, environ_set, environ_unset};
use crate::src::ffi::libc::strchr;
use crate::src::format::format_single_from_target_cstring;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
use crate::src::shared::client::*;
use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_FIND_CANFAIL};
use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
use crate::src::shared::display::{visible_range, visible_ranges};
use crate::src::shared::environment::environ;
use crate::src::shared::environment::ENVIRON_HIDDEN;
use crate::src::shared::event::*;
use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
use crate::src::shared::hyperlinks::hyperlinks;
use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::options;
use crate::src::shared::pane::{
    window_pane, window_pane_modes, window_pane_prompt, window_pane_tree_entry, window_panes,
};
use crate::src::shared::pane::{window_pane_offset, window_pane_resize, window_pane_resizes};
use crate::src::shared::process::tmuxpeer;
use crate::src::shared::prompt::prompt;
use crate::src::shared::redraw::redraw_scene;
use crate::src::shared::screen::{screen, screen_sel, screen_titles};
use crate::src::shared::screen_write::screen_write_cline;
use crate::src::shared::session::{session, session_entry};
use crate::src::shared::spawn::spawn_editor_state;
use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
use crate::src::shared::window::{
    window, window_entry, window_mode, window_mode_entry, window_winlinks, winlink, winlink_entry,
    winlink_stack, winlinks,
};
use crate::src::tmux::global_environ;

use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[no_mangle]
pub static mut cmd_set_environment_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"set-environment\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"setenv\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"Fhgrt:u\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: 2 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-Fhgru] [-t target-session] variable [value]\0" as *const u8
            as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: CMD_FIND_CANFAIL,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_set_environment_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_set_environment_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut name: *const ::core::ffi::c_char = args_string(args, 0 as u_int);
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tflag: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut expanded: Option<std::ffi::CString> = None;
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    if *name as ::core::ffi::c_int == '\0' as i32 {
        cmdq_error(
            item,
            b"empty variable name\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if !strchr(name, '=' as i32).is_null() {
        cmdq_error(
            item,
            b"variable name contains =\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if args_count(args) < 2 as u_int {
        value = ::core::ptr::null::<::core::ffi::c_char>();
    } else {
        value = args_string(args, 1 as u_int);
    }
    if !value.is_null() && args_has(args, 'F' as i32 as u_char) != 0 {
        expanded = Some(format_single_from_target_cstring(item, value));
        value = expanded.as_ref().expect("expanded value was set").as_ptr();
    }
    if args_has(args, 'g' as i32 as u_char) != 0 {
        env = global_environ;
        current_block = 224731115979188411;
    } else if (*target).s.is_null() {
        tflag = args_get(args, 't' as i32 as u_char);
        if !tflag.is_null() {
            cmdq_error(
                item,
                b"no such session: %s\0" as *const u8 as *const ::core::ffi::c_char,
                tflag,
            );
        } else {
            cmdq_error(
                item,
                b"no current session\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        retval = CMD_RETURN_ERROR;
        current_block = 2189724439242469808;
    } else {
        env = (*(*target).s).environ;
        current_block = 224731115979188411;
    }
    match current_block {
        224731115979188411 => {
            if args_has(args, 'u' as i32 as u_char) != 0 {
                if !value.is_null() {
                    cmdq_error(
                        item,
                        b"can't specify a value with -u\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                    retval = CMD_RETURN_ERROR;
                } else {
                    environ_unset(env, name);
                }
            } else if args_has(args, 'r' as i32 as u_char) != 0 {
                if !value.is_null() {
                    cmdq_error(
                        item,
                        b"can't specify a value with -r\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                    retval = CMD_RETURN_ERROR;
                } else {
                    environ_clear(env, name);
                }
            } else if value.is_null() {
                cmdq_error(
                    item,
                    b"no value specified\0" as *const u8 as *const ::core::ffi::c_char,
                );
                retval = CMD_RETURN_ERROR;
            } else if args_has(args, 'h' as i32 as u_char) != 0 {
                environ_set(
                    env,
                    name,
                    ENVIRON_HIDDEN,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    value,
                );
            } else {
                environ_set(
                    env,
                    name,
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    value,
                );
            }
        }
        _ => {}
    }
    return retval;
}

use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::queue::{
    cmdq_error, cmdq_get_client, cmdq_get_source, cmdq_get_target_client,
};
use crate::src::cmd::{cmd_get_args, cmd_get_entry};
use crate::src::server::clients;
use crate::src::server_client::{server_client_detach, server_client_exec, server_client_suspend};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
use crate::src::shared::client::CLIENT_READONLY;
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
use crate::src::shared::command::{
    CMD_CLIENT_TFLAG, CMD_FIND_CANFAIL, CMD_READONLY, CMD_TARGET_CLIENT_USAGE,
};
use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
use crate::src::shared::display::{visible_range, visible_ranges};
use crate::src::shared::environment::environ;
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

use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[no_mangle]
pub static mut cmd_detach_client_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"detach-client\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"detach\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"aE:s:t:P\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-aP] [-E shell-command] [-s target-session] [-t target-client]\0" as *const u8
            as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: CMD_FIND_CANFAIL,
        },
        target: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        flags: CMD_READONLY | CMD_CLIENT_TFLAG,
        exec: Some(
            cmd_detach_client_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_suspend_client_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"suspend-client\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"suspendc\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_CLIENT_USAGE.as_ptr(),
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
        flags: CMD_CLIENT_TFLAG,
        exec: Some(
            cmd_detach_client_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_detach_client_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut source: *mut cmd_find_state = cmdq_get_source(item);
    let mut c: *mut client = cmdq_get_client(item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut msgtype: msgtype = 0 as msgtype;
    let mut cmd: *const ::core::ffi::c_char = args_get(args, 'E' as i32 as u_char);
    if cmd_get_entry(self_0) == &raw const cmd_suspend_client_entry {
        server_client_suspend(tc);
        return CMD_RETURN_NORMAL;
    }
    if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
        if args_has(args, 's' as i32 as u_char) != 0
            || args_has(args, 'a' as i32 as u_char) != 0
            || c != tc
        {
            cmdq_error(
                item,
                b"client is read-only\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
    }
    if args_has(args, 'P' as i32 as u_char) != 0 {
        msgtype = MSG_DETACHKILL;
    } else {
        msgtype = MSG_DETACH;
    }
    if args_has(args, 's' as i32 as u_char) != 0 {
        s = (*source).s;
        if s.is_null() {
            return CMD_RETURN_NORMAL;
        }
        loop_0 = clients.first();
        while !loop_0.is_null() {
            if (*loop_0).session == s {
                if !cmd.is_null() {
                    server_client_exec(loop_0, cmd);
                } else {
                    server_client_detach(loop_0, msgtype);
                }
            }
            loop_0 = clients.next(loop_0);
        }
        return CMD_RETURN_STOP;
    }
    if args_has(args, 'a' as i32 as u_char) != 0 {
        loop_0 = clients.first();
        while !loop_0.is_null() {
            if !(*loop_0).session.is_null() && loop_0 != tc {
                if !cmd.is_null() {
                    server_client_exec(loop_0, cmd);
                } else {
                    server_client_detach(loop_0, msgtype);
                }
            }
            loop_0 = clients.next(loop_0);
        }
        return CMD_RETURN_NORMAL;
    }
    if !cmd.is_null() {
        server_client_exec(tc, cmd);
    } else {
        server_client_detach(tc, msgtype);
    }
    return CMD_RETURN_STOP;
}

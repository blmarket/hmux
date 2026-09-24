use crate::src::arguments::{args_get, args_has, args_make_commands_now};
use crate::src::cmd::{cmd_get_args, cmd_get_entry, cmd_list_first, cmd_list_free};
use crate::src::cmd::queue::{
    cmdq_append, cmdq_continue, cmdq_error, cmdq_get_client, cmdq_get_command, cmdq_get_state,
    cmdq_get_target, cmdq_get_target_client, cmdq_insert_after,
};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
use crate::src::shared::client::CLIENT_DEAD;
use crate::src::shared::client::*;
use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::CMD_CLIENT_TFLAG;
use crate::src::shared::command::*;
use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmdq_state,
    cmds,
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
use crate::src::shared::prompt::*;
use crate::src::shared::prompt::{
    prompt_free_cb, prompt_result, PROMPT_CLOSE, PROMPT_CONTINUE, PROMPT_SINGLE,
};
use crate::src::shared::redraw::redraw_scene;
use crate::src::shared::screen::{screen, screen_sel, screen_titles};
use crate::src::shared::screen_write::screen_write_cline;
use crate::src::shared::session::{session, session_entry};
use crate::src::shared::spawn::spawn_editor_state;
use crate::src::shared::status::{status_line, status_prompt_input_cb};
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
use crate::src::shared::window::{
    window, window_entry, window_mode, window_mode_entry, window_winlinks, winlink, winlink_entry,
    winlink_stack, winlinks,
};
use crate::src::status::status_prompt_set;
use std::ffi::{CStr, CString};

use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_confirm_before_data {
    pub item: *mut cmdq_item,
    pub cmdlist: *mut cmd_list,
    pub confirm_key: u_char,
    pub default_yes: ::core::ffi::c_int,
}
#[no_mangle]
pub static mut cmd_confirm_before_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"confirm-before\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"confirm\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"bc:p:t:y\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: Some(
                cmd_confirm_before_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-by] [-c confirm-key] [-p prompt] [-t target-client] command\0" as *const u8
            as *const ::core::ffi::c_char,
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
            cmd_confirm_before_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_confirm_before_args_parse(
    mut args: *mut args,
    mut idx: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> args_parse_type {
    return ARGS_PARSE_COMMANDS_OR_STRING;
}
unsafe extern "C" fn cmd_confirm_before_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut confirm_key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut prompt: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut wait: ::core::ffi::c_int =
        (args_has(args, 'b' as i32 as u_char) == 0) as ::core::ffi::c_int;
    let mut cdata = Box::new(cmd_confirm_before_data {
        item: ::core::ptr::null_mut(),
        cmdlist: ::core::ptr::null_mut(),
        confirm_key: 0,
        default_yes: 0,
    });
    cdata.cmdlist = args_make_commands_now(self_0, item, 0 as u_int, 1 as ::core::ffi::c_int);
    if cdata.cmdlist.is_null() {
        return CMD_RETURN_ERROR;
    }
    if wait != 0 {
        cdata.item = item;
    }
    cdata.default_yes = args_has(args, 'y' as i32 as u_char);
    confirm_key = args_get(args, 'c' as i32 as u_char);
    if !confirm_key.is_null() {
        if *confirm_key.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\0' as i32
            && *confirm_key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 31 as ::core::ffi::c_int
            && (*confirm_key.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                < 127 as ::core::ffi::c_int
        {
            cdata.confirm_key = *confirm_key.offset(0 as ::core::ffi::c_int as isize) as u_char;
        } else {
            cmdq_error(
                item,
                b"invalid confirm key\0" as *const u8 as *const ::core::ffi::c_char,
            );
            cmd_list_free(cdata.cmdlist);
            return CMD_RETURN_ERROR;
        }
    } else {
        cdata.confirm_key = 'y' as i32 as u_char;
    }
    prompt = args_get(args, 'p' as i32 as u_char);
    let new_prompt = if !prompt.is_null() {
        let mut bytes = CStr::from_ptr(prompt).to_bytes().to_vec();
        bytes.push(b' ');
        CString::new(bytes).expect("C string prompt contains no interior NUL")
    } else {
        cmd = (*cmd_get_entry(cmd_list_first(cdata.cmdlist))).name;
        let mut bytes = b"Confirm '".to_vec();
        bytes.extend_from_slice(CStr::from_ptr(cmd).to_bytes());
        bytes.extend_from_slice(b"'? (");
        bytes.push(cdata.confirm_key);
        bytes.extend_from_slice(b"/n) ");
        CString::new(bytes).expect("C string command and validated key contain no interior NUL")
    };
    status_prompt_set(
        tc,
        target,
        new_prompt.as_ptr(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        Some(
            cmd_confirm_before_callback
                as unsafe extern "C" fn(
                    *mut client,
                    *mut ::core::ffi::c_void,
                    *const ::core::ffi::c_char,
                    prompt_key_result,
                ) -> prompt_result,
        ),
        Some(cmd_confirm_before_free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        Box::into_raw(cdata).cast(),
        PROMPT_SINGLE,
        PROMPT_TYPE_COMMAND,
    );
    drop(new_prompt);
    if wait == 0 {
        return CMD_RETURN_NORMAL;
    }
    return CMD_RETURN_WAIT;
}
unsafe extern "C" fn cmd_confirm_before_callback(
    mut c: *mut client,
    mut data: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut cdata: *mut cmd_confirm_before_data = data as *mut cmd_confirm_before_data;
    let mut item: *mut cmdq_item = (*cdata).item;
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut retcode: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if !((*c).flags & CLIENT_DEAD as uint64_t != 0) {
        if !s.is_null() {
            if !(*s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != (*cdata).confirm_key as ::core::ffi::c_int
                && (*s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '\r' as i32
                    || (*cdata).default_yes == 0))
            {
                retcode = 0 as ::core::ffi::c_int;
                if item.is_null() {
                    new_item =
                        cmdq_get_command((*cdata).cmdlist, ::core::ptr::null_mut::<cmdq_state>());
                    cmdq_append(c, new_item);
                } else {
                    new_item = cmdq_get_command((*cdata).cmdlist, cmdq_get_state(item));
                    cmdq_insert_after(item, new_item);
                }
            }
        }
    }
    if !item.is_null() {
        if !cmdq_get_client(item).is_null() && (*cmdq_get_client(item)).session.is_null() {
            (*cmdq_get_client(item)).retval = retcode;
        }
        cmdq_continue(item);
    }
    return PROMPT_CLOSE;
}
unsafe extern "C" fn cmd_confirm_before_free(mut data: *mut ::core::ffi::c_void) {
    let cdata = Box::from_raw(data as *mut cmd_confirm_before_data);
    cmd_list_free(cdata.cmdlist);
}

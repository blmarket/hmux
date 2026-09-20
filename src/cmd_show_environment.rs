use crate::src::arguments::{args_get, args_has, args_string};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd_queue::{cmdq_error, cmdq_get_target, cmdq_print};
use crate::src::environ::{environ_find, environ_first, environ_next};
use crate::src::ffi::libc::{free, strlen};
use crate::src::tmux::global_environ;
use crate::src::xmalloc::xmalloc;
pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_data};
pub use crate::src::shared::options::{options};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::process::{tmuxpeer};
pub use crate::src::shared::prompt::{prompt};
pub use crate::src::shared::redraw::{redraw_scene};
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::{spawn_editor_state};
pub use crate::src::shared::status::{status_line};
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::environment::{environ, environ_entry, environ_entry_entry};
pub use crate::src::shared::environment::{ENVIRON_HIDDEN};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{CMD_AFTERHOOK, CMD_FIND_CANFAIL};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::arguments::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

#[no_mangle]
pub static mut cmd_show_environment_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"show-environment\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"showenv\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"hgst:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-hgs] [-t target-session] [variable]\0" as *const u8
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
            cmd_show_environment_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_show_environment_escape(
    mut envent: *mut environ_entry,
) -> *mut ::core::ffi::c_char {
    let mut value: *const ::core::ffi::c_char = (*envent).value;
    let mut c: ::core::ffi::c_char = 0;
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ret: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ret = xmalloc(
        strlen(value)
            .wrapping_mul(2 as size_t)
            .wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char;
    out = ret;
    loop {
        let fresh0 = value;
        value = value.offset(1);
        c = *fresh0;
        if !(c as ::core::ffi::c_int != '\0' as i32) {
            break;
        }
        if c as ::core::ffi::c_int == '$' as i32
            || c as ::core::ffi::c_int == '`' as i32
            || c as ::core::ffi::c_int == '"' as i32
            || c as ::core::ffi::c_int == '\\' as i32
        {
            let fresh1 = out;
            out = out.offset(1);
            *fresh1 = '\\' as i32 as ::core::ffi::c_char;
        }
        let fresh2 = out;
        out = out.offset(1);
        *fresh2 = c;
    }
    *out = '\0' as i32 as ::core::ffi::c_char;
    return ret;
}
unsafe extern "C" fn cmd_show_environment_print(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
    mut envent: *mut environ_entry,
) {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut escaped: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if args_has(args, 'h' as i32 as u_char) == 0 && (*envent).flags & ENVIRON_HIDDEN != 0 {
        return;
    }
    if args_has(args, 'h' as i32 as u_char) != 0 && !(*envent).flags & ENVIRON_HIDDEN != 0 {
        return;
    }
    if args_has(args, 's' as i32 as u_char) == 0 {
        if !(*envent).value.is_null() {
            cmdq_print(
                item,
                b"%s=%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*envent).name,
                (*envent).value,
            );
        } else {
            cmdq_print(
                item,
                b"-%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*envent).name,
            );
        }
        return;
    }
    if !(*envent).value.is_null() {
        escaped = cmd_show_environment_escape(envent);
        cmdq_print(
            item,
            b"%s=\"%s\"; export %s;\0" as *const u8 as *const ::core::ffi::c_char,
            (*envent).name,
            escaped,
            (*envent).name,
        );
        free(escaped as *mut ::core::ffi::c_void);
    } else {
        cmdq_print(
            item,
            b"unset %s;\0" as *const u8 as *const ::core::ffi::c_char,
            (*envent).name,
        );
    };
}
unsafe extern "C" fn cmd_show_environment_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut tflag: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = args_string(args, 0 as u_int);
    tflag = args_get(args, 't' as i32 as u_char);
    if !tflag.is_null() {
        if (*target).s.is_null() {
            cmdq_error(
                item,
                b"no such session: %s\0" as *const u8 as *const ::core::ffi::c_char,
                tflag,
            );
            return CMD_RETURN_ERROR;
        }
    }
    if args_has(args, 'g' as i32 as u_char) != 0 {
        env = global_environ;
    } else {
        if (*target).s.is_null() {
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
            return CMD_RETURN_ERROR;
        }
        env = (*(*target).s).environ;
    }
    if !name.is_null() {
        envent = environ_find(env, name);
        if envent.is_null() {
            cmdq_error(
                item,
                b"unknown variable: %s\0" as *const u8 as *const ::core::ffi::c_char,
                name,
            );
            return CMD_RETURN_ERROR;
        }
        cmd_show_environment_print(self_0, item, envent);
        return CMD_RETURN_NORMAL;
    }
    envent = environ_first(env);
    while !envent.is_null() {
        cmd_show_environment_print(self_0, item, envent);
        envent = environ_next(envent);
    }
    return CMD_RETURN_NORMAL;
}

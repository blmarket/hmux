pub use crate::src::shared::arguments::{args_command_state};
pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmdq_state,
    cmds,
};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::job::{job, job_complete_cb, job_free_cb, job_update_cb};
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
pub use crate::src::shared::environment::{environ};
pub use crate::src::shared::abi::{__int32_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{CMD_FIND_CANFAIL};
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
extern "C" {

    fn __ctype_toupper_loc() -> *mut *const __int32_t;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn format_single_from_target(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn job_run(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
        _: *mut environ,
        _: *mut session,
        _: *const ::core::ffi::c_char,
        _: job_update_cb,
        _: job_complete_cb,
        _: job_free_cb,
        _: *mut ::core::ffi::c_void,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut job;
    fn job_get_status(_: *mut job) -> ::core::ffi::c_int;
    fn job_get_data(_: *mut job) -> *mut ::core::ffi::c_void;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn args_make_commands_now(
        _: *mut cmd,
        _: *mut cmdq_item,
        _: u_int,
        _: ::core::ffi::c_int,
    ) -> *mut cmd_list;
    fn args_make_commands_prepare(
        _: *mut cmd,
        _: *mut cmdq_item,
        _: u_int,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut args_command_state;
    fn args_make_commands(
        _: *mut args_command_state,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut cmd_list;
    fn args_make_commands_free(_: *mut args_command_state);
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmd_list_free(_: *mut cmd_list);
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_state(_: *mut cmdq_item) -> *mut cmdq_state;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_command(_: *mut cmd_list, _: *mut cmdq_state) -> *mut cmdq_item;
    fn cmdq_insert_after(_: *mut cmdq_item, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_append(_: *mut client, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_continue(_: *mut cmdq_item);
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn server_client_unref(_: *mut client);
    fn server_client_get_cwd(_: *mut client, _: *mut session) -> *const ::core::ffi::c_char;
    fn status_message_set(
        _: *mut client,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    );
}

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_if_shell_data {
    pub cmd_if: *mut args_command_state,
    pub cmd_else: *mut args_command_state,
    pub client: *mut client,
    pub item: *mut cmdq_item,
}
#[inline]
unsafe extern "C" fn toupper(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_toupper_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}
#[no_mangle]
pub static mut cmd_if_shell_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"if-shell\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"if\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"bFt:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 2 as ::core::ffi::c_int,
            upper: 3 as ::core::ffi::c_int,
            cb: Some(
                cmd_if_shell_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-bF] [-t target-pane] shell-command command [command]\0" as *const u8
            as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_CANFAIL,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_if_shell_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_if_shell_args_parse(
    mut args: *mut args,
    mut idx: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> args_parse_type {
    if idx == 1 as u_int || idx == 2 as u_int {
        return ARGS_PARSE_COMMANDS_OR_STRING;
    }
    return ARGS_PARSE_STRING;
}
unsafe extern "C" fn cmd_if_shell_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut cdata: *mut cmd_if_shell_data = ::core::ptr::null_mut::<cmd_if_shell_data>();
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut shellcmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut s: *mut session = (*target).s;
    let mut cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    let mut count: u_int = args_count(args);
    let mut wait: ::core::ffi::c_int =
        (args_has(args, 'b' as i32 as u_char) == 0) as ::core::ffi::c_int;
    shellcmd = format_single_from_target(item, args_string(args, 0 as u_int));
    if args_has(args, 'F' as i32 as u_char) != 0 {
        if *shellcmd as ::core::ffi::c_int != '0' as i32
            && *shellcmd as ::core::ffi::c_int != '\0' as i32
        {
            cmdlist = args_make_commands_now(self_0, item, 1 as u_int, 0 as ::core::ffi::c_int);
        } else if count == 3 as u_int {
            cmdlist = args_make_commands_now(self_0, item, 2 as u_int, 0 as ::core::ffi::c_int);
        } else {
            free(shellcmd as *mut ::core::ffi::c_void);
            return CMD_RETURN_NORMAL;
        }
        free(shellcmd as *mut ::core::ffi::c_void);
        if cmdlist.is_null() {
            return CMD_RETURN_ERROR;
        }
        new_item = cmdq_get_command(cmdlist, cmdq_get_state(item));
        cmdq_insert_after(item, new_item);
        cmd_list_free(cmdlist);
        return CMD_RETURN_NORMAL;
    }
    cdata = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<cmd_if_shell_data>() as size_t,
    ) as *mut cmd_if_shell_data;
    (*cdata).cmd_if = args_make_commands_prepare(
        self_0,
        item,
        1 as u_int,
        ::core::ptr::null::<::core::ffi::c_char>(),
        wait,
        0 as ::core::ffi::c_int,
    );
    if count == 3 as u_int {
        (*cdata).cmd_else = args_make_commands_prepare(
            self_0,
            item,
            2 as u_int,
            ::core::ptr::null::<::core::ffi::c_char>(),
            wait,
            0 as ::core::ffi::c_int,
        );
    }
    if wait != 0 {
        (*cdata).client = cmdq_get_client(item);
        (*cdata).item = item;
    } else {
        (*cdata).client = tc;
    }
    if !(*cdata).client.is_null() {
        (*(*cdata).client).references += 1;
    }
    if job_run(
        shellcmd,
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        ::core::ptr::null_mut::<environ>(),
        s,
        server_client_get_cwd(cmdq_get_client(item), s),
        None,
        Some(cmd_if_shell_callback as unsafe extern "C" fn(*mut job) -> ()),
        Some(cmd_if_shell_free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        cdata as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    )
    .is_null()
    {
        cmdq_error(
            item,
            b"failed to run command: %s\0" as *const u8 as *const ::core::ffi::c_char,
            shellcmd,
        );
        free(shellcmd as *mut ::core::ffi::c_void);
        cmd_if_shell_free(cdata as *mut ::core::ffi::c_void);
        return CMD_RETURN_ERROR;
    }
    free(shellcmd as *mut ::core::ffi::c_void);
    if wait == 0 {
        return CMD_RETURN_NORMAL;
    }
    return CMD_RETURN_WAIT;
}
unsafe extern "C" fn cmd_if_shell_callback(mut job: *mut job) {
    let mut cdata: *mut cmd_if_shell_data = job_get_data(job) as *mut cmd_if_shell_data;
    let mut c: *mut client = (*cdata).client;
    let mut item: *mut cmdq_item = (*cdata).item;
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut state: *mut args_command_state = ::core::ptr::null_mut::<args_command_state>();
    let mut cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut status: ::core::ffi::c_int = 0;
    status = job_get_status(job);
    if !(status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int)
        || (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
    {
        state = (*cdata).cmd_else;
    } else {
        state = (*cdata).cmd_if;
    }
    if !state.is_null() {
        cmdlist = args_make_commands(
            state,
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            &raw mut error,
        );
        if cmdlist.is_null() {
            if (*cdata).item.is_null() {
                *error = ({
                    let mut __res: ::core::ffi::c_int = 0;
                    if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                        if 0 != 0 {
                            let mut __c: ::core::ffi::c_int =
                                *error as u_char as ::core::ffi::c_int;
                            __res = (if __c < -(128 as ::core::ffi::c_int)
                                || __c > 255 as ::core::ffi::c_int
                            {
                                __c as __int32_t
                            } else {
                                *(*__ctype_toupper_loc()).offset(__c as isize)
                            }) as ::core::ffi::c_int;
                        } else {
                            __res = toupper(*error as u_char as ::core::ffi::c_int);
                        }
                    } else {
                        __res = *(*__ctype_toupper_loc())
                            .offset(*error as u_char as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int;
                    }
                    __res
                }) as ::core::ffi::c_char;
                status_message_set(
                    c,
                    -(1 as ::core::ffi::c_int),
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    error,
                );
            } else {
                cmdq_error(
                    (*cdata).item,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    error,
                );
            }
            free(error as *mut ::core::ffi::c_void);
        } else if item.is_null() {
            new_item = cmdq_get_command(cmdlist, ::core::ptr::null_mut::<cmdq_state>());
            cmdq_append(c, new_item);
            cmd_list_free(cmdlist);
        } else {
            new_item = cmdq_get_command(cmdlist, cmdq_get_state(item));
            cmdq_insert_after(item, new_item);
            cmd_list_free(cmdlist);
        }
    }
    if !(*cdata).item.is_null() {
        cmdq_continue((*cdata).item);
    }
}
unsafe extern "C" fn cmd_if_shell_free(mut data: *mut ::core::ffi::c_void) {
    let mut cdata: *mut cmd_if_shell_data = data as *mut cmd_if_shell_data;
    if !(*cdata).client.is_null() {
        server_client_unref((*cdata).client);
    }
    if !(*cdata).cmd_else.is_null() {
        args_make_commands_free((*cdata).cmd_else);
    }
    args_make_commands_free((*cdata).cmd_if);
    free(cdata as *mut ::core::ffi::c_void);
}

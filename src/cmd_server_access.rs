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
pub use crate::src::shared::environment::{environ};
pub use crate::src::shared::server_acl::{SERVER_ACL_IS_GROUP};
pub use crate::src::shared::account::{group, passwd};
pub use crate::src::shared::abi::{__gid_t, __id_t, __uid_t, id_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{CMD_CLIENT_CANFAIL};
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

    fn getgrnam(__name: *const ::core::ffi::c_char) -> *mut group;
    fn getpwnam(__name: *const ::core::ffi::c_char) -> *mut passwd;
    fn getuid() -> __uid_t;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn format_single(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut ::core::ffi::c_char;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn server_acl_find(_: id_t, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn server_acl_display(_: *mut cmdq_item);
    fn server_acl_allow(_: id_t, _: ::core::ffi::c_int);
    fn server_acl_deny(_: id_t, _: ::core::ffi::c_int);
    fn server_acl_allow_write(_: id_t, _: ::core::ffi::c_int);
    fn server_acl_deny_write(_: id_t, _: ::core::ffi::c_int);
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_12 {
    pub offset: u_int,
    pub data: C2RustUnnamed_13,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_13 {
    pub attr: u_char,
    pub fg: u_char,
    pub bg: u_char,
    pub data: u_char,
}

#[no_mangle]
pub static mut cmd_server_access_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"server-access\0" as *const u8 as *const ::core::ffi::c_char,
        alias: ::core::ptr::null::<::core::ffi::c_char>(),
        args: args_parse {
            template: b"adglrw\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-adglrw] [-t target-pane] [user|group]\0" as *const u8
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
        flags: CMD_CLIENT_CANFAIL,
        exec: Some(
            cmd_server_access_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_server_access_deny(
    mut item: *mut cmdq_item,
    mut id: id_t,
    mut flags: ::core::ffi::c_int,
    mut type_0: *const ::core::ffi::c_char,
    mut name: *const ::core::ffi::c_char,
) -> cmd_retval {
    if server_acl_find(id, flags) == 0 {
        cmdq_error(
            item,
            b"%s %s not found\0" as *const u8 as *const ::core::ffi::c_char,
            type_0,
            name,
        );
        return CMD_RETURN_ERROR;
    }
    server_acl_deny(id, flags);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_server_access_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut c: *mut client = cmdq_get_target_client(item);
    let mut arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut type_0: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    let mut gr: *mut group = ::core::ptr::null_mut::<group>();
    let mut id: id_t = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if args_has(args, 'l' as i32 as u_char) != 0 {
        server_acl_display(item);
        return CMD_RETURN_NORMAL;
    }
    if args_count(args) == 0 as u_int {
        cmdq_error(
            item,
            b"missing user or group argument\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    arg = format_single(
        item,
        args_string(args, 0 as u_int),
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    if args_has(args, 'g' as i32 as u_char) != 0 {
        type_0 = b"group\0" as *const u8 as *const ::core::ffi::c_char;
        gr = getgrnam(arg);
        if !gr.is_null() {
            id = (*gr).gr_gid as id_t;
            name = (*gr).gr_name;
            flags |= SERVER_ACL_IS_GROUP;
        }
    } else {
        type_0 = b"user\0" as *const u8 as *const ::core::ffi::c_char;
        pw = getpwnam(arg);
        if !pw.is_null() {
            id = (*pw).pw_uid as id_t;
            name = (*pw).pw_name;
        }
    }
    if name.is_null() {
        cmdq_error(
            item,
            b"unknown %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            type_0,
            arg,
        );
        free(arg as *mut ::core::ffi::c_void);
        return CMD_RETURN_ERROR;
    }
    free(arg as *mut ::core::ffi::c_void);
    if !flags & SERVER_ACL_IS_GROUP != 0 && (id == 0 as id_t || id == getuid()) {
        cmdq_error(
            item,
            b"%s owns the server, can't change access\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'a' as i32 as u_char) != 0 && args_has(args, 'd' as i32 as u_char) != 0 {
        cmdq_error(
            item,
            b"-a and -d cannot be used together\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'w' as i32 as u_char) != 0 && args_has(args, 'r' as i32 as u_char) != 0 {
        cmdq_error(
            item,
            b"-r and -w cannot be used together\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'd' as i32 as u_char) != 0 {
        return cmd_server_access_deny(item, id, flags, type_0, name);
    }
    if args_has(args, 'a' as i32 as u_char) != 0 {
        if server_acl_find(id, flags) != 0 {
            cmdq_error(
                item,
                b"%s %s is already added\0" as *const u8 as *const ::core::ffi::c_char,
                type_0,
                name,
            );
            return CMD_RETURN_ERROR;
        }
        server_acl_allow(id, flags);
    } else if args_has(args, 'r' as i32 as u_char) != 0 || args_has(args, 'w' as i32 as u_char) != 0
    {
        if server_acl_find(id, flags) == 0 {
            server_acl_allow(id, flags);
        }
    }
    if args_has(args, 'w' as i32 as u_char) != 0 {
        if server_acl_find(id, flags) == 0 {
            cmdq_error(
                item,
                b"%s %s not found\0" as *const u8 as *const ::core::ffi::c_char,
                type_0,
                name,
            );
            return CMD_RETURN_ERROR;
        }
        server_acl_allow_write(id, flags);
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'r' as i32 as u_char) != 0 {
        if server_acl_find(id, flags) == 0 {
            cmdq_error(
                item,
                b"%s %s not found\0" as *const u8 as *const ::core::ffi::c_char,
                type_0,
                name,
            );
            return CMD_RETURN_ERROR;
        }
        server_acl_deny_write(id, flags);
        return CMD_RETURN_NORMAL;
    }
    return CMD_RETURN_NORMAL;
}

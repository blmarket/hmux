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
pub use crate::src::shared::paste::{paste_buffer, paste_buffer_name_entry, paste_buffer_time_entry};
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
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{
    CMD_AFTERHOOK, CMD_BUFFER_USAGE, CMD_CLIENT_CANFAIL, CMD_CLIENT_TFLAG,
};
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

    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(_: *mut ::core::ffi::c_void, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn paste_buffer_data(_: *mut paste_buffer, _: *mut size_t) -> *const ::core::ffi::c_char;
    fn paste_get_top(_: *mut *mut ::core::ffi::c_char) -> *mut paste_buffer;
    fn paste_get_name(_: *const ::core::ffi::c_char) -> *mut paste_buffer;
    fn paste_free(_: *mut paste_buffer);
    fn paste_rename(
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn paste_set(
        _: *mut ::core::ffi::c_char,
        _: size_t,
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn tty_set_selection(
        _: *mut tty,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: size_t,
    );
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn cmd_get_entry(_: *mut cmd) -> *const cmd_entry;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
}

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

#[no_mangle]
pub static mut cmd_set_buffer_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"set-buffer\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"setb\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"ab:t:n:w\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-aw] [-b buffer-name] [-n new-buffer-name] [-t target-client] [data]\0"
            as *const u8 as *const ::core::ffi::c_char,
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
        flags: CMD_AFTERHOOK | CMD_CLIENT_TFLAG | CMD_CLIENT_CANFAIL,
        exec: Some(
            cmd_set_buffer_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_delete_buffer_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"delete-buffer\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"deleteb\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"b:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_BUFFER_USAGE.as_ptr(),
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
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_set_buffer_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_set_buffer_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut bufname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut bufdata: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut olddata: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut bufsize: size_t = 0 as size_t;
    let mut newsize: size_t = 0;
    if !args_get(args, 'b' as i32 as u_char).is_null() {
        bufname = xstrdup(args_get(args, 'b' as i32 as u_char));
        pb = paste_get_name(bufname);
    }
    if cmd_get_entry(self_0) == &raw const cmd_delete_buffer_entry {
        if pb.is_null() {
            if !bufname.is_null() {
                cmdq_error(
                    item,
                    b"unknown buffer: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    bufname,
                );
                current_block = 17843714670734105592;
            } else {
                pb = paste_get_top(&raw mut bufname);
                current_block = 3640593987805443782;
            }
        } else {
            current_block = 3640593987805443782;
        }
        match current_block {
            17843714670734105592 => {}
            _ => {
                if pb.is_null() {
                    cmdq_error(
                        item,
                        b"no buffer\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                } else {
                    paste_free(pb);
                    free(bufname as *mut ::core::ffi::c_void);
                    return CMD_RETURN_NORMAL;
                }
            }
        }
    } else if args_has(args, 'n' as i32 as u_char) != 0 {
        if pb.is_null() {
            if !bufname.is_null() {
                cmdq_error(
                    item,
                    b"unknown buffer: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    bufname,
                );
                current_block = 17843714670734105592;
            } else {
                pb = paste_get_top(&raw mut bufname);
                current_block = 15904375183555213903;
            }
        } else {
            current_block = 15904375183555213903;
        }
        match current_block {
            17843714670734105592 => {}
            _ => {
                if pb.is_null() {
                    cmdq_error(
                        item,
                        b"no buffer\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                } else if paste_rename(
                    bufname,
                    args_get(args, 'n' as i32 as u_char),
                    &raw mut cause,
                ) != 0 as ::core::ffi::c_int
                {
                    cmdq_error(
                        item,
                        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                        cause,
                    );
                } else {
                    free(bufname as *mut ::core::ffi::c_void);
                    return CMD_RETURN_NORMAL;
                }
            }
        }
    } else if args_count(args) != 1 as u_int {
        cmdq_error(
            item,
            b"no data specified\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        newsize = strlen(args_string(args, 0 as u_int));
        if newsize == 0 as size_t {
            free(bufname as *mut ::core::ffi::c_void);
            return CMD_RETURN_NORMAL;
        }
        if args_has(args, 'a' as i32 as u_char) != 0 && !pb.is_null() {
            olddata = paste_buffer_data(pb, &raw mut bufsize);
            bufdata = xmalloc(bufsize) as *mut ::core::ffi::c_char;
            memcpy(
                bufdata as *mut ::core::ffi::c_void,
                olddata as *const ::core::ffi::c_void,
                bufsize,
            );
        }
        bufdata = xrealloc(
            bufdata as *mut ::core::ffi::c_void,
            bufsize.wrapping_add(newsize),
        ) as *mut ::core::ffi::c_char;
        memcpy(
            bufdata.offset(bufsize as isize) as *mut ::core::ffi::c_void,
            args_string(args, 0 as u_int) as *const ::core::ffi::c_void,
            newsize,
        );
        bufsize = bufsize.wrapping_add(newsize);
        if paste_set(bufdata, bufsize, bufname, &raw mut cause) != 0 as ::core::ffi::c_int {
            cmdq_error(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
        } else {
            if args_has(args, 'w' as i32 as u_char) != 0 && !tc.is_null() {
                tty_set_selection(
                    &raw mut (*tc).tty,
                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                    bufdata,
                    bufsize,
                );
            }
            free(bufname as *mut ::core::ffi::c_void);
            return CMD_RETURN_NORMAL;
        }
    }
    free(bufdata as *mut ::core::ffi::c_void);
    free(bufname as *mut ::core::ffi::c_void);
    free(cause as *mut ::core::ffi::c_void);
    return CMD_RETURN_ERROR;
}

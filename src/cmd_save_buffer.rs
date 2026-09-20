use crate::src::arguments::{args_get, args_has, args_string};
use crate::src::cmd::{cmd_get_args, cmd_get_entry};
use crate::src::cmd_queue::{cmdq_continue, cmdq_error, cmdq_get_client, cmdq_print_data};
use crate::src::ffi::libc::{free, strerror};
use crate::src::ffi::libevent::{evbuffer_add, evbuffer_free, evbuffer_new};
use crate::src::file::file_write;
use crate::src::format::format_single_from_target;
use crate::src::log::fatalx;
use crate::src::paste::{paste_buffer_data, paste_get_name, paste_get_top};
use crate::src::xmalloc::xstrdup;
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
pub use crate::src::shared::posix_io::{O_APPEND, O_TRUNC};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{CMD_AFTERHOOK, CMD_BUFFER_USAGE};
pub use crate::src::shared::client::{CLIENT_CONTROL};
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
pub static mut cmd_save_buffer_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"save-buffer\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"saveb\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"ab:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-a] [-b buffer-name] path\0" as *const u8 as *const ::core::ffi::c_char,
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
            cmd_save_buffer_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_show_buffer_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"show-buffer\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"showb\0" as *const u8 as *const ::core::ffi::c_char,
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
            cmd_save_buffer_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_save_buffer_done(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut error: ::core::ffi::c_int,
    mut closed: ::core::ffi::c_int,
    mut buffer: *mut evbuffer,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut item: *mut cmdq_item = data as *mut cmdq_item;
    if closed == 0 {
        return;
    }
    if error != 0 as ::core::ffi::c_int {
        cmdq_error(
            item,
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            strerror(error),
            path,
        );
    }
    cmdq_continue(item);
}
unsafe extern "C" fn cmd_save_buffer_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut c: *mut client = cmdq_get_client(item);
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut flags: ::core::ffi::c_int = 0;
    let mut bufname: *const ::core::ffi::c_char = args_get(args, 'b' as i32 as u_char);
    let mut bufdata: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut bufsize: size_t = 0;
    let mut path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut evb: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    if bufname.is_null() {
        pb = paste_get_top(::core::ptr::null_mut::<*mut ::core::ffi::c_char>());
        if pb.is_null() {
            cmdq_error(
                item,
                b"no buffers\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
    } else {
        pb = paste_get_name(bufname);
        if pb.is_null() {
            cmdq_error(
                item,
                b"no buffer %s\0" as *const u8 as *const ::core::ffi::c_char,
                bufname,
            );
            return CMD_RETURN_ERROR;
        }
    }
    bufdata = paste_buffer_data(pb, &raw mut bufsize);
    if cmd_get_entry(self_0) == &raw const cmd_show_buffer_entry {
        if !(*c).session.is_null() || (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            evb = evbuffer_new();
            if evb.is_null() {
                fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
            }
            evbuffer_add(evb, bufdata as *const ::core::ffi::c_void, bufsize);
            cmdq_print_data(item, evb);
            evbuffer_free(evb);
            return CMD_RETURN_NORMAL;
        }
        path = xstrdup(b"-\0" as *const u8 as *const ::core::ffi::c_char);
    } else {
        path = format_single_from_target(item, args_string(args, 0 as u_int));
    }
    if args_has(args, 'a' as i32 as u_char) != 0 {
        flags = O_APPEND;
    } else {
        flags = O_TRUNC;
    }
    file_write(
        cmdq_get_client(item),
        path,
        flags,
        bufdata as *const ::core::ffi::c_void,
        bufsize,
        Some(
            cmd_save_buffer_done
                as unsafe extern "C" fn(
                    *mut client,
                    *const ::core::ffi::c_char,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                    *mut evbuffer,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        item as *mut ::core::ffi::c_void,
    );
    free(path as *mut ::core::ffi::c_void);
    return CMD_RETURN_WAIT;
}

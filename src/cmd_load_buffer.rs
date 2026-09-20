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
pub use crate::src::shared::abi::{ssize_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{CMD_AFTERHOOK, CMD_CLIENT_CANFAIL, CMD_CLIENT_TFLAG};
pub use crate::src::shared::client::{CLIENT_DEAD};
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
    fn strerror(__errnum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn evbuffer_get_length(buf: *const evbuffer) -> size_t;
    fn evbuffer_pullup(buf: *mut evbuffer, size: ssize_t) -> *mut ::core::ffi::c_uchar;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn paste_set(
        _: *mut ::core::ffi::c_char,
        _: size_t,
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn format_single_from_target(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn tty_set_selection(
        _: *mut tty,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: size_t,
    );
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_continue(_: *mut cmdq_item);
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn file_read(
        _: *mut client,
        _: *const ::core::ffi::c_char,
        _: client_file_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut client_file;
    fn server_client_unref(_: *mut client);
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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_load_buffer_data {
    pub client: *mut client,
    pub item: *mut cmdq_item,
    pub name: *mut ::core::ffi::c_char,
}
#[no_mangle]
pub static mut cmd_load_buffer_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"load-buffer\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"loadb\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"b:t:w\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-b buffer-name] [-t target-client] path\0" as *const u8
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
        flags: CMD_AFTERHOOK | CMD_CLIENT_TFLAG | CMD_CLIENT_CANFAIL,
        exec: Some(
            cmd_load_buffer_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_load_buffer_done(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut error: ::core::ffi::c_int,
    mut closed: ::core::ffi::c_int,
    mut buffer: *mut evbuffer,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut cdata: *mut cmd_load_buffer_data = data as *mut cmd_load_buffer_data;
    let mut tc: *mut client = (*cdata).client;
    let mut item: *mut cmdq_item = (*cdata).item;
    let mut bdata: *mut ::core::ffi::c_void =
        evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t) as *mut ::core::ffi::c_void;
    let mut bsize: size_t = evbuffer_get_length(buffer);
    let mut copy: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
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
    } else if bsize != 0 as size_t {
        copy = xmalloc(bsize);
        memcpy(copy, bdata, bsize);
        if paste_set(
            copy as *mut ::core::ffi::c_char,
            bsize,
            (*cdata).name,
            &raw mut cause,
        ) != 0 as ::core::ffi::c_int
        {
            cmdq_error(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
            free(copy);
        } else if !tc.is_null()
            && !(*tc).session.is_null()
            && !(*tc).flags & CLIENT_DEAD as uint64_t != 0
        {
            tty_set_selection(
                &raw mut (*tc).tty,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
                copy as *const ::core::ffi::c_char,
                bsize,
            );
        }
        if !tc.is_null() {
            server_client_unref(tc);
        }
    }
    cmdq_continue(item);
    free((*cdata).name as *mut ::core::ffi::c_void);
    free(cdata as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn cmd_load_buffer_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut cdata: *mut cmd_load_buffer_data = ::core::ptr::null_mut::<cmd_load_buffer_data>();
    let mut bufname: *const ::core::ffi::c_char = args_get(args, 'b' as i32 as u_char);
    let mut path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    cdata = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<cmd_load_buffer_data>() as size_t,
    ) as *mut cmd_load_buffer_data;
    (*cdata).item = item;
    if !bufname.is_null() {
        (*cdata).name = xstrdup(bufname);
    }
    if args_has(args, 'w' as i32 as u_char) != 0 && !tc.is_null() {
        (*cdata).client = tc;
        (*(*cdata).client).references += 1;
    }
    path = format_single_from_target(item, args_string(args, 0 as u_int));
    file_read(
        cmdq_get_client(item),
        path,
        Some(
            cmd_load_buffer_done
                as unsafe extern "C" fn(
                    *mut client,
                    *const ::core::ffi::c_char,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                    *mut evbuffer,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        cdata as *mut ::core::ffi::c_void,
    );
    free(path as *mut ::core::ffi::c_void);
    return CMD_RETURN_WAIT;
}

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
pub use crate::src::shared::vis::{VIS_NOSLASH, VIS_SAFE};
pub use crate::src::shared::pane::{
    PANE_INPUTOFF, window_pane_offset, window_pane_resize, window_pane_resize_entry,
    window_pane_resizes,
};
pub use crate::src::shared::screen::{MODE_BRACKETPASTE, screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::command::{CMD_AFTERHOOK};
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

    fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn bufferevent_write(
        bufev: *mut bufferevent,
        data: *const ::core::ffi::c_void,
        size: size_t,
    ) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn paste_buffer_data(_: *mut paste_buffer, _: *mut size_t) -> *const ::core::ffi::c_char;
    fn paste_get_top(_: *mut *mut ::core::ffi::c_char) -> *mut paste_buffer;
    fn paste_get_name(_: *const ::core::ffi::c_char) -> *mut paste_buffer;
    fn paste_free(_: *mut paste_buffer);
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn window_pane_exited(_: *mut window_pane) -> ::core::ffi::c_int;
    fn utf8_stravisx(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: size_t,
        _: ::core::ffi::c_int,
    ) -> size_t;
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
pub static mut cmd_paste_buffer_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"paste-buffer\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"pasteb\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"db:prSs:t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-dprS] [-s separator] [-b buffer-name] [-t target-pane]\0" as *const u8
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
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_paste_buffer_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_paste_buffer_paste(
    mut wp: *mut window_pane,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
) {
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut n: size_t = 0;
    n = utf8_stravisx(&raw mut cp, buf, len, VIS_SAFE | VIS_NOSLASH);
    bufferevent_write((*wp).event, cp as *const ::core::ffi::c_void, n);
    free(cp as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn cmd_paste_buffer_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut wp: *mut window_pane = (*target).wp;
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut sepstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut bufname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut bufdata: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut bufend: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut seplen: size_t = 0;
    let mut bufsize: size_t = 0;
    let mut len: size_t = 0;
    let mut bracket: ::core::ffi::c_int = args_has(args, 'p' as i32 as u_char);
    if window_pane_exited(wp) != 0 {
        cmdq_error(
            item,
            b"target pane has exited\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    bufname = ::core::ptr::null::<::core::ffi::c_char>();
    if args_has(args, 'b' as i32 as u_char) != 0 {
        bufname = args_get(args, 'b' as i32 as u_char);
    }
    if bufname.is_null() {
        pb = paste_get_top(::core::ptr::null_mut::<*mut ::core::ffi::c_char>());
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
    if !pb.is_null() && !(*wp).flags & PANE_INPUTOFF != 0 {
        sepstr = args_get(args, 's' as i32 as u_char);
        if sepstr.is_null() {
            if args_has(args, 'r' as i32 as u_char) != 0 {
                sepstr = b"\n\0" as *const u8 as *const ::core::ffi::c_char;
            } else {
                sepstr = b"\r\0" as *const u8 as *const ::core::ffi::c_char;
            }
        }
        seplen = strlen(sepstr);
        if bracket != 0 && (*(*wp).screen).mode & MODE_BRACKETPASTE != 0 {
            bufferevent_write(
                (*wp).event,
                b"\x1B[200~\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                6 as size_t,
            );
        }
        bufdata = paste_buffer_data(pb, &raw mut bufsize);
        bufend = bufdata.offset(bufsize as isize);
        loop {
            line = memchr(
                bufdata as *const ::core::ffi::c_void,
                '\n' as i32,
                bufend.offset_from(bufdata) as ::core::ffi::c_long as size_t,
            ) as *const ::core::ffi::c_char;
            if line.is_null() {
                break;
            }
            len = line.offset_from(bufdata) as ::core::ffi::c_long as size_t;
            if args_has(args, 'S' as i32 as u_char) != 0 {
                bufferevent_write((*wp).event, bufdata as *const ::core::ffi::c_void, len);
            } else {
                cmd_paste_buffer_paste(wp, bufdata, len);
            }
            bufferevent_write((*wp).event, sepstr as *const ::core::ffi::c_void, seplen);
            bufdata = line.offset(1 as ::core::ffi::c_int as isize);
        }
        if bufdata != bufend {
            len = bufend.offset_from(bufdata) as ::core::ffi::c_long as size_t;
            if args_has(args, 'S' as i32 as u_char) != 0 {
                bufferevent_write((*wp).event, bufdata as *const ::core::ffi::c_void, len);
            } else {
                cmd_paste_buffer_paste(wp, bufdata, len);
            }
        }
        if bracket != 0 && (*(*wp).screen).mode & MODE_BRACKETPASTE != 0 {
            bufferevent_write(
                (*wp).event,
                b"\x1B[201~\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                6 as size_t,
            );
        }
    }
    if !pb.is_null() && args_has(args, 'd' as i32 as u_char) != 0 {
        paste_free(pb);
    }
    return CMD_RETURN_NORMAL;
}

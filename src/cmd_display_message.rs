pub use crate::src::shared::json::{json_node};
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
pub use crate::src::shared::format::{FORMAT_NONE, FORMAT_VERBOSE};
pub use crate::src::shared::limits::{__INT_MAX__, UINT_MAX};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{
    CMD_AFTERHOOK, CMD_CLIENT_CANFAIL, CMD_CLIENT_CFLAG, CMD_FIND_CANFAIL,
};
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
extern "C" {

    fn evbuffer_new() -> *mut evbuffer;
    fn evbuffer_free(buf: *mut evbuffer);
    fn evbuffer_add_printf(
        buf: *mut evbuffer,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn format_create(
        _: *mut client,
        _: *mut cmdq_item,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut format_tree;
    fn format_free(_: *mut format_tree);
    fn format_each(
        _: *mut format_tree,
        _: Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_char,
                *const ::core::ffi::c_char,
                *mut ::core::ffi::c_void,
            ) -> (),
        >,
        _: *mut ::core::ffi::c_void,
    );
    fn format_expand_time(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn format_defaults(
        _: *mut format_tree,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    );
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn args_strtonum(
        _: *mut args,
        _: u_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn cmd_find_best_client(_: *mut session) -> *mut client;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_print(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn server_client_print(_: *mut client, _: ::core::ffi::c_int, _: *mut evbuffer);
    fn status_message_set(
        _: *mut client,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn window_pane_start_input(
        _: *mut window_pane,
        _: *mut cmdq_item,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
    fn json_parse(
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut json_node;
    fn json_destroy_node(_: *mut json_node);
    fn json_to_string(_: *mut json_node) -> *mut ::core::ffi::c_char;
}

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

pub const DISPLAY_MESSAGE_TEMPLATE: [::core::ffi::c_char; 96] = unsafe {
    ::core::mem::transmute::<
        [u8; 96],
        [::core::ffi::c_char; 96],
    >(
        *b"[#{session_name}] #{window_index}:#{window_name}, current pane #{pane_index} - (%H:%M %d-%b-%y)\0",
    )
};
#[no_mangle]
pub static mut cmd_display_message_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"display-message\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"display\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"aCc:d:jlINpt:F:v\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-aCIjlNpv] [-c target-client] [-d delay] [-F format] [-t target-pane] [message]\0"
            as *const u8 as *const ::core::ffi::c_char,
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
        flags: CMD_AFTERHOOK | CMD_CLIENT_CFLAG | CMD_CLIENT_CANFAIL,
        exec: Some(
            cmd_display_message_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_display_message_each(
    mut key: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut item: *mut cmdq_item = arg as *mut cmdq_item;
    cmdq_print(
        item,
        b"%s=%s\0" as *const u8 as *const ::core::ffi::c_char,
        key,
        value,
    );
}
unsafe extern "C" fn cmd_display_message_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = (*target).s;
    let mut wl: *mut winlink = (*target).wl;
    let mut wp: *mut window_pane = (*target).wp;
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut msg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut delay: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut flags: ::core::ffi::c_int = 0;
    let mut Nflag: ::core::ffi::c_int = args_has(args, 'N' as i32 as u_char);
    let mut Cflag: ::core::ffi::c_int = args_has(args, 'C' as i32 as u_char);
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut count: u_int = args_count(args);
    let mut evb: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut jn: *mut json_node = ::core::ptr::null_mut::<json_node>();
    if args_has(args, 'I' as i32 as u_char) != 0 && args_has(args, 'j' as i32 as u_char) == 0 {
        if wp.is_null() {
            return CMD_RETURN_NORMAL;
        }
        match window_pane_start_input(wp, item, &raw mut cause) {
            -1 => {
                cmdq_error(
                    item,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    cause,
                );
                free(cause as *mut ::core::ffi::c_void);
                return CMD_RETURN_ERROR;
            }
            1 => return CMD_RETURN_NORMAL,
            0 => return CMD_RETURN_WAIT,
            _ => {}
        }
    }
    if args_has(args, 'F' as i32 as u_char) != 0 && count != 0 as u_int {
        cmdq_error(
            item,
            b"only one of -F or argument must be given\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'd' as i32 as u_char) != 0 {
        delay = args_strtonum(
            args,
            'd' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            UINT_MAX as ::core::ffi::c_longlong,
            &raw mut cause,
        ) as ::core::ffi::c_int;
        if !cause.is_null() {
            cmdq_error(
                item,
                b"delay %s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
    }
    if count != 0 as u_int {
        template = args_string(args, 0 as u_int);
    } else {
        template = args_get(args, 'F' as i32 as u_char);
    }
    if args_has(args, 'j' as i32 as u_char) != 0 && template.is_null() {
        template = b"\0" as *const u8 as *const ::core::ffi::c_char;
    } else if template.is_null() {
        template = DISPLAY_MESSAGE_TEMPLATE.as_ptr();
    }
    if !tc.is_null() && (*tc).session == s {
        c = tc;
    } else if !s.is_null() {
        c = cmd_find_best_client(s);
    } else {
        c = ::core::ptr::null_mut::<client>();
    }
    if args_has(args, 'v' as i32 as u_char) != 0 {
        flags = FORMAT_VERBOSE;
    } else {
        flags = 0 as ::core::ffi::c_int;
    }
    ft = format_create(cmdq_get_client(item), item, FORMAT_NONE, flags);
    format_defaults(ft, c, s, wl, wp);
    if args_has(args, 'a' as i32 as u_char) != 0 && args_has(args, 'j' as i32 as u_char) == 0 {
        format_each(
            ft,
            Some(
                cmd_display_message_each
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_char,
                        *const ::core::ffi::c_char,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            item as *mut ::core::ffi::c_void,
        );
        format_free(ft);
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'l' as i32 as u_char) != 0 {
        msg = xstrdup(template);
    } else {
        msg = format_expand_time(ft, template);
    }
    if args_has(args, 'j' as i32 as u_char) != 0 {
        jn = json_parse(msg, &raw mut cause);
        if jn.is_null() {
            cmdq_error(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
            free(msg as *mut ::core::ffi::c_void);
            format_free(ft);
            return CMD_RETURN_ERROR;
        }
        free(msg as *mut ::core::ffi::c_void);
        msg = json_to_string(jn);
        json_destroy_node(jn);
    }
    if cmdq_get_client(item).is_null() {
        cmdq_error(
            item,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            msg,
        );
    } else if args_has(args, 'p' as i32 as u_char) != 0 {
        cmdq_print(
            item,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            msg,
        );
    } else if !tc.is_null() && (*tc).flags & CLIENT_CONTROL as uint64_t != 0 {
        evb = evbuffer_new();
        if evb.is_null() {
            fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
        }
        evbuffer_add_printf(
            evb,
            b"%%message %s\0" as *const u8 as *const ::core::ffi::c_char,
            msg,
        );
        server_client_print(tc, 0 as ::core::ffi::c_int, evb);
        evbuffer_free(evb);
    } else if !tc.is_null() {
        status_message_set(
            tc,
            delay,
            0 as ::core::ffi::c_int,
            Nflag,
            Cflag,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            msg,
        );
    }
    free(msg as *mut ::core::ffi::c_void);
    format_free(ft);
    return CMD_RETURN_NORMAL;
}

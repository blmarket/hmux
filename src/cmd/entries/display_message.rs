use crate::src::arguments::{args_count, args_get, args_has, args_string, args_strtonum_result};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd_find::cmd_find_best_client;
use crate::src::cmd_queue::{
    cmdq_error, cmdq_get_client, cmdq_get_target, cmdq_get_target_client, cmdq_print,
};
use crate::src::ffi::libc::free;
use crate::src::format::{
    format_create, format_defaults, format_each, format_expand_time, format_free,
};
use crate::src::json::{json_destroy_node, json_parse, json_to_string};
use crate::src::log::fatalx;
use crate::src::reactor::{evbuffer_add_printf, evbuffer_free, evbuffer_new};
use crate::src::server_client::server_client_print;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
pub use crate::src::shared::client::CLIENT_CONTROL;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::command::{
    CMD_AFTERHOOK, CMD_CLIENT_CANFAIL, CMD_CLIENT_CFLAG, CMD_FIND_CANFAIL,
};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::format::{FORMAT_NONE, FORMAT_VERBOSE};
use crate::src::shared::grid::*;
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::json::json_node;
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::limits::{__INT_MAX__, UINT_MAX};
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::status::status_message_set;
use crate::src::window::window_pane_start_input;
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

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

// format_expand_time and json_to_string return libc-owned C strings. Copy
// their visible C-string bytes before releasing the foreign allocation.
unsafe fn cmd_display_message_take_string(raw: *mut ::core::ffi::c_char) -> CString {
    let owned = CStr::from_ptr(raw).to_owned();
    free(raw as *mut ::core::ffi::c_void);
    owned
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
    let mut cause: Option<CString> = None;
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
        match window_pane_start_input(wp, item) {
            Err(error) => {
                cmdq_error(
                    item,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    error.as_ptr(),
                );
                return CMD_RETURN_ERROR;
            }
            Ok(1) => return CMD_RETURN_NORMAL,
            Ok(0) => return CMD_RETURN_WAIT,
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
        delay = match args_strtonum_result(
            args,
            'd' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            UINT_MAX as ::core::ffi::c_longlong,
        ) {
            Ok(value) => value as ::core::ffi::c_int,
            Err(error) => {
                cmdq_error(
                    item,
                    b"delay %s\0" as *const u8 as *const ::core::ffi::c_char,
                    error.message().as_ptr(),
                );
                return CMD_RETURN_ERROR;
            }
        };
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
    let mut msg = if args_has(args, 'l' as i32 as u_char) != 0 {
        CStr::from_ptr(template).to_owned()
    } else {
        cmd_display_message_take_string(format_expand_time(ft, template))
    };
    if args_has(args, 'j' as i32 as u_char) != 0 {
        jn = json_parse(msg.as_ptr(), &raw mut cause);
        if jn.is_null() {
            cmdq_error(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause.as_ref().map_or(::core::ptr::null(), |message| message.as_ptr()),
            );
            drop(msg);
            format_free(ft);
            return CMD_RETURN_ERROR;
        }
        drop(msg);
        msg = cmd_display_message_take_string(json_to_string(jn));
        json_destroy_node(jn);
    }
    if cmdq_get_client(item).is_null() {
        cmdq_error(
            item,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            msg.as_ptr(),
        );
    } else if args_has(args, 'p' as i32 as u_char) != 0 {
        cmdq_print(
            item,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            msg.as_ptr(),
        );
    } else if !tc.is_null() && (*tc).flags & CLIENT_CONTROL as uint64_t != 0 {
        evb = evbuffer_new();
        if evb.is_null() {
            fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
        }
        evbuffer_add_printf(
            evb,
            b"%%message %s\0" as *const u8 as *const ::core::ffi::c_char,
            msg.as_ptr(),
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
            msg.as_ptr(),
        );
    }
    drop(msg);
    format_free(ft);
    return CMD_RETURN_NORMAL;
}

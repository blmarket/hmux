pub use crate::src::shared::command::{cmd_parse_input, cmd_parse_result};
pub use crate::src::shared::arguments::{
    args, args_parse, args_parse_cb, args_value, args_value_c2rust_unnamed, args_value_entry,
};
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
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{CMD_AFTERHOOK};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::arguments::*;
use crate::src::shared::command::*;
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

    fn free(__ptr: *mut ::core::ffi::c_void);
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_count(_: *mut args) -> u_int;
    fn args_values(_: *mut args) -> *mut args_value;
    fn args_value(_: *mut args, _: u_int) -> *mut args_value;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmd_parse_from_string(
        _: *const ::core::ffi::c_char,
        _: *mut cmd_parse_input,
    ) -> *mut cmd_parse_result;
    fn cmd_parse_from_arguments(
        _: *mut args_value,
        _: u_int,
        _: *mut cmd_parse_input,
    ) -> *mut cmd_parse_result;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn key_bindings_add(
        _: *const ::core::ffi::c_char,
        _: key_code,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut cmd_list,
    );
    fn key_string_lookup_string(_: *const ::core::ffi::c_char) -> key_code;
}

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

pub type C2RustUnnamed_35 = ::core::ffi::c_ulong;

#[no_mangle]
pub static mut cmd_bind_key_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"bind-key\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"bind\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"nrN:T:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: Some(
                cmd_bind_key_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-nr] [-T key-table] [-N note] key [command [argument ...]]\0" as *const u8
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
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_bind_key_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_bind_key_args_parse(
    mut args: *mut args,
    mut idx: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> args_parse_type {
    return ARGS_PARSE_COMMANDS_OR_STRING;
}
unsafe extern "C" fn cmd_bind_key_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut key: key_code = 0;
    let mut tablename: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut note: *const ::core::ffi::c_char = args_get(args, 'N' as i32 as u_char);
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut repeat: ::core::ffi::c_int = 0;
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut count: u_int = args_count(args);
    key = key_string_lookup_string(args_string(args, 0 as u_int));
    if key == KEYC_NONE as ::core::ffi::c_ulong as key_code
        || key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code
    {
        cmdq_error(
            item,
            b"unknown key: %s\0" as *const u8 as *const ::core::ffi::c_char,
            args_string(args, 0 as u_int),
        );
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'T' as i32 as u_char) != 0 {
        tablename = args_get(args, 'T' as i32 as u_char);
    } else if args_has(args, 'n' as i32 as u_char) != 0 {
        tablename = b"root\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        tablename = b"prefix\0" as *const u8 as *const ::core::ffi::c_char;
    }
    repeat = args_has(args, 'r' as i32 as u_char);
    if count == 1 as u_int {
        key_bindings_add(
            tablename,
            key,
            note,
            repeat,
            ::core::ptr::null_mut::<cmd_list>(),
        );
        return CMD_RETURN_NORMAL;
    }
    value = args_value(args, 1 as u_int);
    if count == 2 as u_int
        && (*value).type_0 as ::core::ffi::c_uint
            == ARGS_COMMANDS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        key_bindings_add(
            tablename,
            key,
            note,
            repeat,
            (*value).c2rust_unnamed.cmdlist as *mut cmd_list,
        );
        (*(*value).c2rust_unnamed.cmdlist).references += 1;
        return CMD_RETURN_NORMAL;
    }
    if count == 2 as u_int {
        pr = cmd_parse_from_string(
            args_string(args, 1 as u_int),
            ::core::ptr::null_mut::<cmd_parse_input>(),
        );
    } else {
        pr = cmd_parse_from_arguments(
            args_values(args).offset(1 as ::core::ffi::c_int as isize),
            count.wrapping_sub(1 as u_int),
            ::core::ptr::null_mut::<cmd_parse_input>(),
        );
    }
    match (*pr).status as ::core::ffi::c_uint {
        0 => {
            cmdq_error(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*pr).error,
            );
            free((*pr).error as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
        1 | _ => {}
    }
    key_bindings_add(tablename, key, note, repeat, (*pr).cmdlist);
    return CMD_RETURN_NORMAL;
}

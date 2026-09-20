pub use crate::src::shared::events::{event_payload};
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
pub use crate::src::shared::options::{
    options, options_array, options_array_item, options_entry, options_table_entry,
    options_value,
};
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
pub use crate::src::shared::monitor::{
    MONITOR_ALL_PANES, MONITOR_ALL_WINDOWS, MONITOR_NOTIFY_TRUE, MONITOR_PANE, MONITOR_SESSION,
    MONITOR_WINDOW, monitor_type,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::options::{OPTIONS_TABLE_NONE, OPTIONS_TABLE_WINDOW};
pub use crate::src::shared::command::{CMD_AFTERHOOK, CMD_FIND_CANFAIL};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::options::*;
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

    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    static mut global_options: *mut options;
    static mut global_s_options: *mut options;
    static mut global_w_options: *mut options;
    fn format_single_from_target(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn event_payload_create() -> *mut event_payload;
    fn event_payload_set_target(_: *mut event_payload, _: *mut cmd_find_state);
    fn event_payload_set_int(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    );
    fn event_payload_set_client(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut client,
    );
    fn event_payload_set_session(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut session,
    );
    fn event_payload_set_window(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut window,
    );
    fn event_payload_set_pane(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut window_pane,
    );
    fn events_fire(_: *const ::core::ffi::c_char, _: *mut event_payload);
    fn hooks_add_event(_: *const ::core::ffi::c_char);
    fn hooks_run(_: *mut cmdq_item, _: *const ::core::ffi::c_char);
    fn hooks_monitor_add(
        _: *mut cmdq_item,
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: monitor_type,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut cmd_find_state,
        _: *mut session,
    );
    fn hooks_monitor_remove(_: *mut options, _: *const ::core::ffi::c_char);
    fn options_empty(_: *mut options, _: *const options_table_entry) -> *mut options_entry;
    fn options_table_entry(_: *mut options_entry) -> *const options_table_entry;
    fn options_get_only(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_get(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_array_clear(_: *mut options_entry);
    fn options_array_get(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
    ) -> *mut options_value;
    fn options_array_set(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn options_array_assign(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn options_is_array(_: *mut options_entry) -> ::core::ffi::c_int;
    fn options_match(
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
        _: *mut ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn options_set_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    ) -> *mut options_entry;
    fn options_scope_from_name(
        _: *mut args,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        _: *mut cmd_find_state,
        _: *mut *mut options,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn options_from_string(
        _: *mut options,
        _: *const options_table_entry,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn options_push_changes(_: *const ::core::ffi::c_char);
    fn options_remove_or_default(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn cmd_find_copy_state(_: *mut cmd_find_state, _: *mut cmd_find_state);
    fn cmd_get_entry(_: *mut cmd) -> *const cmd_entry;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn monitor_parse(
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
        _: *mut monitor_type,
        _: *mut ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
}

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

#[no_mangle]
pub static mut cmd_set_option_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"set-option\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"set\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"aFgopqst:uUw\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: 2 as ::core::ffi::c_int,
            cb: Some(
                cmd_set_option_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-aFgopqsuUw] [-t target-pane] option [value]\0" as *const u8
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
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_set_option_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_set_window_option_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"set-window-option\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"setw\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"aFgoqt:u\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: 2 as ::core::ffi::c_int,
            cb: Some(
                cmd_set_option_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-aFgoqu] [-t target-window] option [value]\0" as *const u8
            as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: CMD_FIND_CANFAIL,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_set_option_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_set_hook_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"set-hook\0" as *const u8 as *const ::core::ffi::c_char,
        alias: ::core::ptr::null::<::core::ffi::c_char>(),
        args: args_parse {
            template: b"agpERTt:uB:w\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 2 as ::core::ffi::c_int,
            cb: Some(
                cmd_set_option_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-agpERTuw] [-B name:what:format] [-t target-pane] [hook] [command]\0" as *const u8
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
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_set_option_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_set_option_args_parse(
    mut args: *mut args,
    mut idx: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> args_parse_type {
    if args_has(args, 'B' as i32 as u_char) != 0 {
        return ARGS_PARSE_COMMANDS_OR_STRING;
    }
    if idx == 1 as u_int {
        return ARGS_PARSE_COMMANDS_OR_STRING;
    }
    return ARGS_PARSE_STRING;
}
unsafe extern "C" fn cmd_set_hook_event_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut argument: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if args_count(args) == 0 as u_int {
        cmdq_error(
            item,
            b"missing argument\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if args_count(args) != 1 as u_int {
        cmdq_error(
            item,
            b"too many arguments\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    argument = format_single_from_target(item, args_string(args, 0 as u_int));
    if *argument as ::core::ffi::c_int != '@' as i32 {
        cmdq_error(
            item,
            b"event name must start with @\0" as *const u8 as *const ::core::ffi::c_char,
        );
        free(argument as *mut ::core::ffi::c_void);
        return CMD_RETURN_ERROR;
    }
    ep = event_payload_create();
    event_payload_set_target(ep, target);
    c = cmdq_get_client(item);
    if !c.is_null() {
        event_payload_set_client(
            ep,
            b"client\0" as *const u8 as *const ::core::ffi::c_char,
            c,
        );
    }
    if !(*target).s.is_null() {
        event_payload_set_session(
            ep,
            b"session\0" as *const u8 as *const ::core::ffi::c_char,
            (*target).s,
        );
    }
    if !(*target).w.is_null() {
        event_payload_set_window(
            ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            (*target).w,
        );
    }
    if !(*target).wl.is_null() {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*target).wl).idx,
        );
    } else if (*target).idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*target).idx,
        );
    }
    if !(*target).wp.is_null() {
        event_payload_set_pane(
            ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            (*target).wp,
        );
    }
    events_fire(argument, ep);
    free(argument as *mut ::core::ffi::c_void);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_set_hook_monitor_exec(
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut window: ::core::ffi::c_int,
) -> cmd_retval {
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut format: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut newvalue: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut old: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut type_0: monitor_type = MONITOR_SESSION;
    let mut id: ::core::ffi::c_int = 0;
    let mut scope: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if args_count(args) > 1 as u_int {
        cmdq_error(
            item,
            b"too many arguments\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    value = args_get(args, 'B' as i32 as u_char);
    if args_has(args, 'u' as i32 as u_char) != 0 {
        if monitor_parse(
            value,
            &raw mut name,
            &raw mut type_0,
            &raw mut id,
            &raw mut format,
        ) != 0 as ::core::ffi::c_int
        {
            name = xstrdup(value);
        }
        free(format as *mut ::core::ffi::c_void);
        format = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else if monitor_parse(
        value,
        &raw mut name,
        &raw mut type_0,
        &raw mut id,
        &raw mut format,
    ) != 0 as ::core::ffi::c_int
    {
        cmdq_error(
            item,
            b"invalid subscription: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return CMD_RETURN_ERROR;
    }
    if *name as ::core::ffi::c_int != '@' as i32 {
        cmdq_error(
            item,
            b"monitor hook name must start with @\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        scope = options_scope_from_name(args, window, name, target, &raw mut oo, &raw mut cause);
        if scope == OPTIONS_TABLE_NONE {
            cmdq_error(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
        } else {
            cmd_find_copy_state(&raw mut fs, target);
            if args_has(args, 'u' as i32 as u_char) != 0 {
                hooks_monitor_remove(oo, name);
            } else {
                if args_count(args) != 0 as u_int {
                    value = args_string(args, 0 as u_int);
                    if args_has(args, 'F' as i32 as u_char) != 0 {
                        expanded = format_single_from_target(item, value);
                        value = expanded;
                    }
                    o = options_get_only(oo, name);
                    if args_has(args, 'o' as i32 as u_char) == 0 || o.is_null() {
                        if args_has(args, 'a' as i32 as u_char) != 0 && !o.is_null() {
                            old = options_get_string(oo, name);
                            xasprintf(
                                &raw mut newvalue,
                                b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                                old,
                                value,
                            );
                            value = newvalue;
                        }
                        options_set_string(
                            oo,
                            name,
                            0 as ::core::ffi::c_int,
                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                            value,
                        );
                        options_push_changes(name);
                    }
                }
                if oo != global_options && oo != global_s_options && oo != global_w_options {
                    s = (*target).s;
                }
                if args_has(args, 'T' as i32 as u_char) != 0 {
                    flags |= MONITOR_NOTIFY_TRUE;
                }
                hooks_monitor_add(item, oo, name, type_0, id, format, flags, &raw mut fs, s);
            }
            free(newvalue as *mut ::core::ffi::c_void);
            free(expanded as *mut ::core::ffi::c_void);
            free(name as *mut ::core::ffi::c_void);
            free(format as *mut ::core::ffi::c_void);
            return CMD_RETURN_NORMAL;
        }
    }
    free(newvalue as *mut ::core::ffi::c_void);
    free(expanded as *mut ::core::ffi::c_void);
    free(name as *mut ::core::ffi::c_void);
    free(format as *mut ::core::ffi::c_void);
    return CMD_RETURN_ERROR;
}
unsafe extern "C" fn cmd_set_option_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut append: ::core::ffi::c_int = args_has(args, 'a' as i32 as u_char);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut loop_0: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut parent: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut po: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut argument: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut array_key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut window: ::core::ffi::c_int = 0;
    let mut already: ::core::ffi::c_int = 0;
    let mut error: ::core::ffi::c_int = 0;
    let mut ambiguous: ::core::ffi::c_int = 0;
    let mut scope: ::core::ffi::c_int = 0;
    window =
        (cmd_get_entry(self_0) == &raw const cmd_set_window_option_entry) as ::core::ffi::c_int;
    if cmd_get_entry(self_0) == &raw const cmd_set_hook_entry
        && args_has(args, 'E' as i32 as u_char) != 0
    {
        return cmd_set_hook_event_exec(self_0, item);
    }
    if cmd_get_entry(self_0) == &raw const cmd_set_hook_entry
        && args_has(args, 'B' as i32 as u_char) != 0
    {
        return cmd_set_hook_monitor_exec(item, args, window);
    }
    if args_count(args) == 0 as u_int {
        cmdq_error(
            item,
            b"missing argument\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    argument = format_single_from_target(item, args_string(args, 0 as u_int));
    if cmd_get_entry(self_0) == &raw const cmd_set_hook_entry
        && args_has(args, 'R' as i32 as u_char) != 0
    {
        hooks_run(item, argument);
        free(argument as *mut ::core::ffi::c_void);
        return CMD_RETURN_NORMAL;
    }
    name = options_match(argument, &raw mut array_key, &raw mut ambiguous);
    if name.is_null() {
        if args_has(args, 'q' as i32 as u_char) != 0 {
            current_block = 710513931074292511;
        } else {
            if ambiguous != 0 {
                cmdq_error(
                    item,
                    b"ambiguous option: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    argument,
                );
            } else {
                cmdq_error(
                    item,
                    b"invalid option: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    argument,
                );
            }
            current_block = 8517774764635037400;
        }
    } else {
        if args_count(args) < 2 as u_int {
            value = ::core::ptr::null::<::core::ffi::c_char>();
        } else {
            value = args_string(args, 1 as u_int);
        }
        if !value.is_null() && args_has(args, 'F' as i32 as u_char) != 0 {
            expanded = format_single_from_target(item, value);
            value = expanded;
        }
        scope = options_scope_from_name(args, window, name, target, &raw mut oo, &raw mut cause);
        if scope == OPTIONS_TABLE_NONE {
            if args_has(args, 'q' as i32 as u_char) != 0 {
                current_block = 710513931074292511;
            } else {
                cmdq_error(
                    item,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    cause,
                );
                free(cause as *mut ::core::ffi::c_void);
                current_block = 8517774764635037400;
            }
        } else {
            o = options_get_only(oo, name);
            parent = options_get(oo, name);
            if !array_key.is_null()
                && (*name as ::core::ffi::c_int == '@' as i32 || options_is_array(parent) == 0)
            {
                cmdq_error(
                    item,
                    b"not an array: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    argument,
                );
                current_block = 8517774764635037400;
            } else {
                if args_has(args, 'u' as i32 as u_char) == 0
                    && args_has(args, 'o' as i32 as u_char) != 0
                {
                    if array_key.is_null() {
                        already = (o != NULL as *mut options_entry) as ::core::ffi::c_int;
                    } else if o.is_null() {
                        already = 0 as ::core::ffi::c_int;
                    } else if !options_array_get(o, array_key).is_null() {
                        already = 1 as ::core::ffi::c_int;
                    } else {
                        already = 0 as ::core::ffi::c_int;
                    }
                    if already != 0 {
                        if args_has(args, 'q' as i32 as u_char) != 0 {
                            current_block = 710513931074292511;
                        } else {
                            cmdq_error(
                                item,
                                b"already set: %s\0" as *const u8 as *const ::core::ffi::c_char,
                                argument,
                            );
                            current_block = 8517774764635037400;
                        }
                    } else {
                        current_block = 12997042908615822766;
                    }
                } else {
                    current_block = 12997042908615822766;
                }
                match current_block {
                    710513931074292511 => {}
                    8517774764635037400 => {}
                    _ => {
                        if args_has(args, 'U' as i32 as u_char) != 0
                            && scope == OPTIONS_TABLE_WINDOW
                        {
                            loop_0 = (*(*target).w).panes.tqh_first;
                            loop {
                                if loop_0.is_null() {
                                    current_block = 10095721787123848864;
                                    break;
                                }
                                po = options_get_only((*loop_0).options, name);
                                if !po.is_null() {
                                    if options_remove_or_default(po, array_key, &raw mut cause)
                                        != 0 as ::core::ffi::c_int
                                    {
                                        cmdq_error(
                                            item,
                                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            cause,
                                        );
                                        free(cause as *mut ::core::ffi::c_void);
                                        current_block = 8517774764635037400;
                                        break;
                                    }
                                }
                                loop_0 = (*loop_0).entry.tqe_next;
                            }
                        } else {
                            current_block = 10095721787123848864;
                        }
                        match current_block {
                            8517774764635037400 => {}
                            _ => {
                                if args_has(args, 'u' as i32 as u_char) != 0
                                    || args_has(args, 'U' as i32 as u_char) != 0
                                {
                                    if o.is_null() {
                                        current_block = 710513931074292511;
                                    } else if options_remove_or_default(
                                        o,
                                        array_key,
                                        &raw mut cause,
                                    ) != 0 as ::core::ffi::c_int
                                    {
                                        cmdq_error(
                                            item,
                                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            cause,
                                        );
                                        free(cause as *mut ::core::ffi::c_void);
                                        current_block = 8517774764635037400;
                                    } else {
                                        current_block = 16231175055492490595;
                                    }
                                } else if *name as ::core::ffi::c_int == '@' as i32 {
                                    if value.is_null() {
                                        cmdq_error(
                                            item,
                                            b"empty value\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                        );
                                        current_block = 8517774764635037400;
                                    } else {
                                        options_set_string(
                                            oo,
                                            name,
                                            append,
                                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            value,
                                        );
                                        if cmd_get_entry(self_0) == &raw const cmd_set_hook_entry {
                                            hooks_add_event(name);
                                        }
                                        current_block = 16231175055492490595;
                                    }
                                } else if array_key.is_null() && options_is_array(parent) == 0 {
                                    error = options_from_string(
                                        oo,
                                        options_table_entry(parent),
                                        (*options_table_entry(parent)).name,
                                        value,
                                        args_has(args, 'a' as i32 as u_char),
                                        &raw mut cause,
                                    );
                                    if error != 0 as ::core::ffi::c_int {
                                        cmdq_error(
                                            item,
                                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            cause,
                                        );
                                        free(cause as *mut ::core::ffi::c_void);
                                        current_block = 8517774764635037400;
                                    } else {
                                        current_block = 16231175055492490595;
                                    }
                                } else if value.is_null() {
                                    cmdq_error(
                                        item,
                                        b"empty value\0" as *const u8 as *const ::core::ffi::c_char,
                                    );
                                    current_block = 8517774764635037400;
                                } else {
                                    if o.is_null() {
                                        o = options_empty(oo, options_table_entry(parent));
                                    }
                                    if array_key.is_null() {
                                        if append == 0 {
                                            options_array_clear(o);
                                        }
                                        if options_array_assign(o, value, &raw mut cause)
                                            != 0 as ::core::ffi::c_int
                                        {
                                            cmdq_error(
                                                item,
                                                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                                cause,
                                            );
                                            free(cause as *mut ::core::ffi::c_void);
                                            current_block = 8517774764635037400;
                                        } else {
                                            current_block = 16231175055492490595;
                                        }
                                    } else if options_array_set(
                                        o,
                                        array_key,
                                        value,
                                        append,
                                        &raw mut cause,
                                    ) != 0 as ::core::ffi::c_int
                                    {
                                        cmdq_error(
                                            item,
                                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            cause,
                                        );
                                        free(cause as *mut ::core::ffi::c_void);
                                        current_block = 8517774764635037400;
                                    } else {
                                        current_block = 16231175055492490595;
                                    }
                                }
                                match current_block {
                                    8517774764635037400 => {}
                                    710513931074292511 => {}
                                    _ => {
                                        options_push_changes(name);
                                        current_block = 710513931074292511;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    match current_block {
        8517774764635037400 => {
            free(argument as *mut ::core::ffi::c_void);
            free(expanded as *mut ::core::ffi::c_void);
            free(name as *mut ::core::ffi::c_void);
            free(array_key as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
        _ => {
            free(argument as *mut ::core::ffi::c_void);
            free(expanded as *mut ::core::ffi::c_void);
            free(name as *mut ::core::ffi::c_void);
            free(array_key as *mut ::core::ffi::c_void);
            return CMD_RETURN_NORMAL;
        }
    };
}

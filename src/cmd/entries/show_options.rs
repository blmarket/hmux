use crate::src::arguments::{args_count, args_get, args_has, args_string};
use crate::src::cmd::{cmd_get_args, cmd_get_entry};
use crate::src::cmd_queue::{cmdq_error, cmdq_get_target, cmdq_print};
use crate::src::ffi::libc::free;
use crate::src::format::{
    format_add, format_add_tv, format_create_from_target, format_expand, format_free,
    format_single_from_target,
};
use crate::src::hooks::{
    hooks_is_event, hooks_monitor_get, hooks_monitor_get_fire_count, hooks_monitor_get_fire_time,
    hooks_monitor_to_cstring,
};
pub use crate::src::options::options_table_entry;
use crate::src::options::{
    options_array_first, options_array_item_key, options_array_next, options_first, options_get,
    options_get_fire_count, options_get_fire_time, options_get_monitor_data, options_get_only,
    options_is_array, options_is_string, options_match_owned, options_name, options_next,
    options_scope_from_flags, options_scope_from_name, options_to_cstring, options_to_string,
    OptionMatchFailure,
};
use crate::src::options_table::options_table;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::command::{CMD_AFTERHOOK, CMD_FIND_CANFAIL};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::monitor::{
    monitor_type, MONITOR_ALL_PANES, MONITOR_ALL_WINDOWS, MONITOR_PANE, MONITOR_SESSION,
    MONITOR_WINDOW,
};
pub use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::*;
pub use crate::src::shared::options::{options, options_array_item, options_entry};
pub use crate::src::shared::options::{OPTIONS_TABLE_IS_HOOK, OPTIONS_TABLE_NONE};
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
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub const SHOW_OPTIONS_TEMPLATE: [::core::ffi::c_char; 202] = unsafe {
    ::core::mem::transmute::<
        [u8; 202],
        [::core::ffi::c_char; 202],
    >(
        *b"#{?option_value_only,#{option_value},#{option_name}#{?option_has_array_key,[#{option_array_key}],}#{?option_is_parent,*,}#{?option_has_value, #{?option_is_string,#{q/a:option_value},#{option_value}},}}\0",
    )
};
pub const SHOW_HOOKS_MONITOR_TEMPLATE: [::core::ffi::c_char; 61] = unsafe {
    ::core::mem::transmute::<[u8; 61], [::core::ffi::c_char; 61]>(
        *b"#{option_name}:#{hook_monitor_target}:#{hook_monitor_format}\0",
    )
};
#[no_mangle]
pub static mut cmd_show_options_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"show-options\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"show\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"AgF:Hpqst:vw\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-AgHpqsvw] [-F format] [-t target-pane] [option]\0" as *const u8
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
            cmd_show_options_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_show_window_options_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"show-window-options\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"showw\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"F:gvt:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-gv] [-F format] [-t target-window] [option]\0" as *const u8
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
            cmd_show_options_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_show_hooks_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"show-hooks\0" as *const u8 as *const ::core::ffi::c_char,
        alias: ::core::ptr::null::<::core::ffi::c_char>(),
        args: args_parse {
            template: b"BF:gpt:w\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-Bgpw] [-F format] [-t target-pane] [hook]\0" as *const u8
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
            cmd_show_options_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_show_options_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut argument: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut array_key: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut window: ::core::ffi::c_int = 0;
    let mut ambiguous: ::core::ffi::c_int = 0;
    let mut parent: ::core::ffi::c_int = 0;
    let mut print_parent: ::core::ffi::c_int = 0;
    let mut scope: ::core::ffi::c_int = 0;
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    window =
        (cmd_get_entry(self_0) == &raw const cmd_show_window_options_entry) as ::core::ffi::c_int;
    if args_count(args) == 0 as u_int {
        scope = options_scope_from_flags(args, window, target, &raw mut oo, &raw mut cause);
        if scope == OPTIONS_TABLE_NONE {
            if args_has(args, 'q' as i32 as u_char) != 0 {
                return CMD_RETURN_NORMAL;
            }
            cmdq_error(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
        if cmd_get_entry(self_0) == &raw const cmd_show_hooks_entry
            && args_has(args, 'B' as i32 as u_char) != 0
        {
            o = options_first(oo);
            while !o.is_null() {
                cmd_show_hooks_print_monitor(self_0, item, o);
                o = options_next(o);
            }
            return CMD_RETURN_NORMAL;
        }
        return cmd_show_options_all(self_0, item, scope, oo);
    }
    argument = format_single_from_target(item, args_string(args, 0 as u_int));
    let matched = options_match_owned(CStr::from_ptr(argument));
    if let Ok(parsed) = &matched {
        name = parsed.name.as_ptr();
        array_key = parsed
            .array_key
            .as_ref()
            .map_or(::core::ptr::null(), |key| key.as_ptr());
    }
    ambiguous = matches!(matched, Err(OptionMatchFailure::Ambiguous)) as ::core::ffi::c_int;
    if name.is_null() {
        if args_has(args, 'q' as i32 as u_char) != 0 {
            current_block = 9776955515550960483;
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
            current_block = 18040240512796061664;
        }
    } else {
        scope = options_scope_from_name(args, window, name, target, &raw mut oo, &raw mut cause);
        if scope == OPTIONS_TABLE_NONE {
            if args_has(args, 'q' as i32 as u_char) != 0 {
                current_block = 9776955515550960483;
            } else {
                cmdq_error(
                    item,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    cause,
                );
                free(cause as *mut ::core::ffi::c_void);
                current_block = 18040240512796061664;
            }
        } else {
            o = options_get_only(oo, name);
            if args_has(args, 'A' as i32 as u_char) != 0 && o.is_null() {
                o = options_get(oo, name);
                parent = 1 as ::core::ffi::c_int;
            } else {
                parent = 0 as ::core::ffi::c_int;
            }
            if !o.is_null() {
                if cmd_get_entry(self_0) == &raw const cmd_show_hooks_entry
                    && args_has(args, 'B' as i32 as u_char) != 0
                {
                    cmd_show_hooks_print_monitor(self_0, item, o);
                } else {
                    print_parent = parent;
                    if array_key.is_null()
                        && options_is_array(o) != 0
                        && options_array_first(o).is_null()
                    {
                        print_parent = 0 as ::core::ffi::c_int;
                    }
                    cmd_show_options_print(self_0, item, o, array_key, print_parent);
                }
                current_block = 9776955515550960483;
            } else if *name as ::core::ffi::c_int == '@' as i32 {
                if args_has(args, 'q' as i32 as u_char) != 0 {
                    current_block = 9776955515550960483;
                } else {
                    cmdq_error(
                        item,
                        b"invalid option: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        argument,
                    );
                    current_block = 18040240512796061664;
                }
            } else {
                current_block = 9776955515550960483;
            }
        }
    }
    match current_block {
        18040240512796061664 => {
            free(argument as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
        _ => {
            free(argument as *mut ::core::ffi::c_void);
            return CMD_RETURN_NORMAL;
        }
    };
}
unsafe fn cmd_show_options_value(
    o: *mut options_entry,
    array_key: *const ::core::ffi::c_char,
) -> CString {
    options_to_cstring(o, array_key, 0)
}

unsafe extern "C" fn cmd_show_options_print(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
    mut o: *mut options_entry,
    mut array_key: *const ::core::ffi::c_char,
    mut parent: ::core::ffi::c_int,
) {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut name: *const ::core::ffi::c_char = options_name(o);
    let mut template: *const ::core::ffi::c_char = args_get(args, 'F' as i32 as u_char);
    let value: CString;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tv: timeval = timeval {
        tv_sec: 0 as __time_t,
        tv_usec: 0,
    };
    let mut fire_count: u_int = 0;
    let mut fire_time: time_t = 0;
    let mut is_hook: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut is_user: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut has_value: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut oe: *const options_table_entry = options_table_entry(o);
    if !array_key.is_null() {
        value = cmd_show_options_value(o, array_key);
    } else if options_is_array(o) != 0 {
        a = options_array_first(o);
        if !a.is_null() {
            while !a.is_null() {
                array_key = options_array_item_key(a);
                cmd_show_options_print(self_0, item, o, array_key, parent);
                a = options_array_next(a);
            }
            return;
        }
        if template.is_null() && args_has(args, 'v' as i32 as u_char) != 0 {
            return;
        }
        value = CString::default();
        has_value = 0 as ::core::ffi::c_int;
    } else {
        value = cmd_show_options_value(o, ::core::ptr::null());
    }
    if template.is_null() {
        template = SHOW_OPTIONS_TEMPLATE.as_ptr();
    }
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0 {
        is_hook = 1 as ::core::ffi::c_int;
    } else if oe.is_null() {
        is_user = 1 as ::core::ffi::c_int;
    }
    ft = format_create_from_target(item);
    format_add(
        ft,
        b"option_name\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    format_add(
        ft,
        b"option_value\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        value.as_ptr(),
    );
    format_add(
        ft,
        b"option_value_only\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        args_has(args, 'v' as i32 as u_char),
    );
    format_add(
        ft,
        b"option_is_parent\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        parent,
    );
    format_add(
        ft,
        b"option_is_array\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        options_is_array(o),
    );
    format_add(
        ft,
        b"option_is_string\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        options_is_string(o),
    );
    format_add(
        ft,
        b"option_is_hook\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        is_hook,
    );
    format_add(
        ft,
        b"option_is_user\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        is_user,
    );
    format_add(
        ft,
        b"option_has_value\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        has_value,
    );
    if cmd_get_entry(self_0) == &raw const cmd_show_hooks_entry {
        fire_count = options_get_fire_count(o);
        format_add(
            ft,
            b"hook_fire_count\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            fire_count,
        );
        fire_time = options_get_fire_time(o);
        if fire_time != 0 as time_t {
            tv.tv_sec = fire_time as __time_t;
            format_add_tv(
                ft,
                b"hook_fire_time\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut tv,
            );
        }
    }
    if !array_key.is_null() {
        format_add(
            ft,
            b"option_array_key\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            array_key,
        );
        format_add(
            ft,
            b"option_has_array_key\0" as *const u8 as *const ::core::ffi::c_char,
            b"1\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        format_add(
            ft,
            b"option_array_key\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
        );
        format_add(
            ft,
            b"option_has_array_key\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    line = format_expand(ft, template);
    format_free(ft);
    cmdq_print(
        item,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        line,
    );
    free(line as *mut ::core::ffi::c_void);
    drop(value);
}
unsafe extern "C" fn cmd_show_hooks_print_monitor(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
    mut o: *mut options_entry,
) {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut type_0: monitor_type = MONITOR_SESSION;
    let mut template: *const ::core::ffi::c_char = args_get(args, 'F' as i32 as u_char);
    let mut format: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tv: timeval = timeval {
        tv_sec: 0 as __time_t,
        tv_usec: 0,
    };
    let mut fire_count: u_int = 0;
    let mut fire_time: time_t = 0;
    let mut id: ::core::ffi::c_int = 0;
    let Some(value) = hooks_monitor_to_cstring(o) else {
        return;
    };
    if hooks_monitor_get(o, &raw mut type_0, &raw mut id, &raw mut format) == 0 {
        return;
    }
    if template.is_null() {
        template = SHOW_HOOKS_MONITOR_TEMPLATE.as_ptr();
    }
    let target = match type_0 {
        MONITOR_SESSION => Some(CString::new("").unwrap()),
        MONITOR_PANE => Some(CString::new(format!("%{id}")).unwrap()),
        MONITOR_ALL_PANES => Some(CString::new("%*").unwrap()),
        MONITOR_WINDOW => Some(CString::new(format!("@{id}")).unwrap()),
        MONITOR_ALL_WINDOWS => Some(CString::new("@*").unwrap()),
        _ => None,
    };
    ft = format_create_from_target(item);
    format_add(
        ft,
        b"option_name\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        options_name(o),
    );
    format_add(
        ft,
        b"option_value\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        value.as_ptr(),
    );
    format_add(
        ft,
        b"option_value_only\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
    );
    format_add(
        ft,
        b"option_is_parent\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
    );
    format_add(
        ft,
        b"option_is_array\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
    );
    format_add(
        ft,
        b"option_is_string\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    format_add(
        ft,
        b"option_is_hook\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    format_add(
        ft,
        b"option_is_user\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    format_add(
        ft,
        b"option_has_value\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    format_add(
        ft,
        b"option_array_key\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"\0" as *const u8 as *const ::core::ffi::c_char,
    );
    format_add(
        ft,
        b"option_has_array_key\0" as *const u8 as *const ::core::ffi::c_char,
        b"0\0" as *const u8 as *const ::core::ffi::c_char,
    );
    format_add(
        ft,
        b"hook_monitor_target\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        target.as_ref().map_or(::core::ptr::null(), |s| s.as_ptr()),
    );
    format_add(
        ft,
        b"hook_monitor_format\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        format,
    );
    fire_count = hooks_monitor_get_fire_count(o);
    format_add(
        ft,
        b"hook_fire_count\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        fire_count,
    );
    fire_time = hooks_monitor_get_fire_time(o);
    if fire_time != 0 as time_t {
        tv.tv_sec = fire_time as __time_t;
        format_add_tv(
            ft,
            b"hook_fire_time\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut tv,
        );
    }
    line = format_expand(ft, template);
    format_free(ft);
    cmdq_print(
        item,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        line,
    );
    free(line as *mut ::core::ffi::c_void);
    drop(target);
}
unsafe extern "C" fn cmd_show_options_all(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
    mut scope: ::core::ffi::c_int,
    mut oo: *mut options,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut parent: ::core::ffi::c_int = 0;
    let mut is_user_hook: ::core::ffi::c_int = 0;
    o = options_first(oo);
    while !o.is_null() {
        if options_table_entry(o).is_null() {
            name = options_name(o);
            is_user_hook = 0 as ::core::ffi::c_int;
            if *name as ::core::ffi::c_int == '@' as i32 {
                if hooks_is_event(name) != 0 || !options_get_monitor_data(o).is_null() {
                    is_user_hook = 1 as ::core::ffi::c_int;
                }
            }
            if cmd_get_entry(self_0) != &raw const cmd_show_hooks_entry {
                if is_user_hook == 0 || args_has(args, 'H' as i32 as u_char) != 0 {
                    cmd_show_options_print(
                        self_0,
                        item,
                        o,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        0 as ::core::ffi::c_int,
                    );
                }
            } else if is_user_hook != 0 {
                cmd_show_options_print(
                    self_0,
                    item,
                    o,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    0 as ::core::ffi::c_int,
                );
            }
        }
        o = options_next(o);
    }
    let mut current_block_25: u64;
    oe = &raw const options_table as *const options_table_entry;
    while !(*oe).name.is_null() {
        if !(!(*oe).scope & scope != 0) {
            if !(cmd_get_entry(self_0) != &raw const cmd_show_hooks_entry
                && args_has(args, 'H' as i32 as u_char) == 0
                && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0
                || cmd_get_entry(self_0) == &raw const cmd_show_hooks_entry
                    && !(*oe).flags & OPTIONS_TABLE_IS_HOOK != 0)
            {
                o = options_get_only(oo, (*oe).name);
                if o.is_null() {
                    if args_has(args, 'A' as i32 as u_char) == 0 {
                        current_block_25 = 2370887241019905314;
                    } else {
                        o = options_get(oo, (*oe).name);
                        if o.is_null() {
                            current_block_25 = 2370887241019905314;
                        } else {
                            parent = 1 as ::core::ffi::c_int;
                            current_block_25 = 15345278821338558188;
                        }
                    }
                } else {
                    parent = 0 as ::core::ffi::c_int;
                    current_block_25 = 15345278821338558188;
                }
                match current_block_25 {
                    2370887241019905314 => {}
                    _ => {
                        cmd_show_options_print(
                            self_0,
                            item,
                            o,
                            ::core::ptr::null::<::core::ffi::c_char>(),
                            parent,
                        );
                    }
                }
            }
        }
        oe = oe.offset(1);
    }
    return CMD_RETURN_NORMAL;
}

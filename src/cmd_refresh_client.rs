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
pub use crate::src::shared::limits::{__INT_MAX__, INT_MAX};
pub use crate::src::shared::monitor::{
    MONITOR_ALL_PANES, MONITOR_ALL_WINDOWS, MONITOR_PANE, MONITOR_SESSION, MONITOR_WINDOW,
    monitor_type,
};
pub use crate::src::shared::window::{WINDOW_MAXIMUM, WINDOW_MINIMUM};
pub use crate::src::shared::pane::{
    PANE_MINIMUM, PANE_THEMECHANGED, window_pane_offset, window_pane_resize,
    window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::command::{CMD_AFTERHOOK, CMD_CLIENT_TFLAG};
pub use crate::src::shared::client::{
    CLIENT_CONTROL, CLIENT_SIZECHANGED, CLIENT_STATUSFORCE, CLIENT_WINDOWSIZECHANGED,
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

    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn sscanf(
        __s: *const ::core::ffi::c_char,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn strtonum(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn tty_update_client_offset(_: *mut client);
    fn tty_clipboard_query(_: *mut tty);
    fn tty_set_size(_: *mut tty, _: u_int, _: u_int, _: u_int, _: u_int);
    fn tty_keys_colours(
        _: *mut tty,
        _: *const ::core::ffi::c_char,
        _: size_t,
        _: *mut size_t,
        _: *mut ::core::ffi::c_int,
        _: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn args_first_value(_: *mut args, _: u_char) -> *mut args_value;
    fn args_next_value(_: *mut args_value) -> *mut args_value;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn server_client_set_flags(_: *mut client, _: *const ::core::ffi::c_char);
    fn server_redraw_client(_: *mut client);
    fn server_status_client(_: *mut client);
    fn recalculate_sizes_now(_: ::core::ffi::c_int);
    fn window_pane_find_by_id(_: u_int) -> *mut window_pane;
    fn monitor_parse(
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
        _: *mut monitor_type,
        _: *mut ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn control_set_pane_on(_: *mut client, _: *mut window_pane);
    fn control_set_pane_off(_: *mut client, _: *mut window_pane);
    fn control_continue_pane(_: *mut client, _: *mut window_pane);
    fn control_pause_pane(_: *mut client, _: *mut window_pane);
    fn control_set_window_size(_: *mut client, _: u_int, _: u_int, _: u_int);
    fn control_clear_window_size(_: *mut client, _: u_int);
    fn control_add_sub(
        _: *mut client,
        _: *const ::core::ffi::c_char,
        _: monitor_type,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
    );
    fn control_remove_sub(_: *mut client, _: *const ::core::ffi::c_char);
    fn log_debug(_: *const ::core::ffi::c_char, ...);
}

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

#[no_mangle]
pub static mut cmd_refresh_client_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"refresh-client\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"refresh\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"A:B:cC:Df:r:F:lLRSt:U\0" as *const u8
                as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-cDlLRSU] [-A pane:state] [-B name:what:format] [-C XxY] [-f flags] [-r pane:report] [-t target-client] [adjustment]\0"
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
        flags: CMD_AFTERHOOK | CMD_CLIENT_TFLAG,
        exec: Some(
            cmd_refresh_client_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_refresh_client_update_subscription(
    mut tc: *mut client,
    mut value: *const ::core::ffi::c_char,
) {
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut format: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut type_0: monitor_type = MONITOR_SESSION;
    let mut id: ::core::ffi::c_int = 0;
    if monitor_parse(
        value,
        &raw mut name,
        &raw mut type_0,
        &raw mut id,
        &raw mut format,
    ) != 0 as ::core::ffi::c_int
    {
        control_remove_sub(tc, value);
        return;
    }
    control_add_sub(tc, name, type_0, id, format);
    free(name as *mut ::core::ffi::c_void);
    free(format as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn cmd_refresh_client_control_client_size(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut size: *const ::core::ffi::c_char = args_get(args, 'C' as i32 as u_char);
    let mut w: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if sscanf(
        size,
        b"@%u:%ux%u\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut w,
        &raw mut x,
        &raw mut y,
    ) == 3 as ::core::ffi::c_int
    {
        if x < WINDOW_MINIMUM as u_int
            || x > WINDOW_MAXIMUM as u_int
            || y < WINDOW_MINIMUM as u_int
            || y > WINDOW_MAXIMUM as u_int
        {
            cmdq_error(
                item,
                b"size too small or too big\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
        log_debug(
            b"%s: client %s window @%u: size %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_refresh_client_control_client_size\0" as *const u8 as *const ::core::ffi::c_char,
            (*tc).name,
            w,
            x,
            y,
        );
        control_set_window_size(tc, w, x, y);
        (*tc).flags =
            ((*tc).flags as ::core::ffi::c_ulonglong | CLIENT_WINDOWSIZECHANGED) as uint64_t;
        recalculate_sizes_now(1 as ::core::ffi::c_int);
        return CMD_RETURN_NORMAL;
    }
    if sscanf(
        size,
        b"@%u:\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut w,
    ) == 1 as ::core::ffi::c_int
    {
        log_debug(
            b"%s: client %s window @%u: no size\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_refresh_client_control_client_size\0" as *const u8 as *const ::core::ffi::c_char,
            (*tc).name,
            w,
        );
        control_clear_window_size(tc, w);
        recalculate_sizes_now(1 as ::core::ffi::c_int);
        return CMD_RETURN_NORMAL;
    }
    if sscanf(
        size,
        b"%u,%u\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut x,
        &raw mut y,
    ) != 2 as ::core::ffi::c_int
        && sscanf(
            size,
            b"%ux%u\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut x,
            &raw mut y,
        ) != 2 as ::core::ffi::c_int
    {
        cmdq_error(
            item,
            b"bad size argument\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if x < WINDOW_MINIMUM as u_int
        || x > WINDOW_MAXIMUM as u_int
        || y < WINDOW_MINIMUM as u_int
        || y > WINDOW_MAXIMUM as u_int
    {
        cmdq_error(
            item,
            b"size too small or too big\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    tty_set_size(&raw mut (*tc).tty, x, y, 0 as u_int, 0 as u_int);
    (*tc).flags |= CLIENT_SIZECHANGED as uint64_t;
    recalculate_sizes_now(1 as ::core::ffi::c_int);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_refresh_client_update_offset(
    mut tc: *mut client,
    mut value: *const ::core::ffi::c_char,
) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut split: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pane: u_int = 0;
    if *value as ::core::ffi::c_int != '%' as i32 {
        return;
    }
    copy = xstrdup(value);
    split = strchr(copy, ':' as i32);
    if !split.is_null() {
        let fresh0 = split;
        split = split.offset(1);
        *fresh0 = '\0' as i32 as ::core::ffi::c_char;
        if !(sscanf(
            copy,
            b"%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut pane,
        ) != 1 as ::core::ffi::c_int)
        {
            wp = window_pane_find_by_id(pane);
            if !wp.is_null() {
                if strcmp(split, b"on\0" as *const u8 as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    control_set_pane_on(tc, wp);
                } else if strcmp(split, b"off\0" as *const u8 as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    control_set_pane_off(tc, wp);
                } else if strcmp(
                    split,
                    b"continue\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    control_continue_pane(tc, wp);
                } else if strcmp(split, b"pause\0" as *const u8 as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    control_pause_pane(tc, wp);
                }
            }
        }
    }
    free(copy as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn cmd_refresh_report(mut tty: *mut tty, mut value: *const ::core::ffi::c_char) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut pane: u_int = 0;
    let mut fg: ::core::ffi::c_int = 0;
    let mut bg: ::core::ffi::c_int = 0;
    let mut size: size_t = 0 as size_t;
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut split: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *value as ::core::ffi::c_int != '%' as i32 {
        return;
    }
    copy = xstrdup(value);
    split = strchr(copy, ':' as i32);
    if !split.is_null() {
        let fresh1 = split;
        split = split.offset(1);
        *fresh1 = '\0' as i32 as ::core::ffi::c_char;
        if !(sscanf(
            copy,
            b"%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut pane,
        ) != 1 as ::core::ffi::c_int)
        {
            wp = window_pane_find_by_id(pane);
            if !wp.is_null() {
                fg = (*wp).control_fg;
                bg = (*wp).control_bg;
                if tty_keys_colours(
                    tty,
                    split,
                    strlen(split),
                    &raw mut size,
                    &raw mut fg,
                    &raw mut bg,
                ) == 0 as ::core::ffi::c_int
                {
                    if bg != (*wp).control_bg {
                        (*wp).flags |= PANE_THEMECHANGED;
                    }
                    (*wp).control_fg = fg;
                    (*wp).control_bg = bg;
                }
            }
        }
    }
    free(copy as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn cmd_refresh_client_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut tty: *mut tty = &raw mut (*tc).tty;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut adjust: u_int = 0;
    let mut av: *mut args_value = ::core::ptr::null_mut::<args_value>();
    if args_has(args, 'c' as i32 as u_char) != 0
        || args_has(args, 'L' as i32 as u_char) != 0
        || args_has(args, 'R' as i32 as u_char) != 0
        || args_has(args, 'U' as i32 as u_char) != 0
        || args_has(args, 'D' as i32 as u_char) != 0
    {
        if args_count(args) == 0 as u_int {
            adjust = 1 as u_int;
        } else {
            adjust = strtonum(
                args_string(args, 0 as u_int),
                1 as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as u_int;
            if !errstr.is_null() {
                cmdq_error(
                    item,
                    b"adjustment %s\0" as *const u8 as *const ::core::ffi::c_char,
                    errstr,
                );
                return CMD_RETURN_ERROR;
            }
        }
        if args_has(args, 'c' as i32 as u_char) != 0 {
            (*tc).pan_window = NULL;
        } else {
            w = (*(*(*tc).session).curw).window;
            if (*tc).pan_window != w as *mut ::core::ffi::c_void {
                (*tc).pan_window = w as *mut ::core::ffi::c_void;
                (*tc).pan_ox = (*tty).oox;
                (*tc).pan_oy = (*tty).ooy;
            }
            if args_has(args, 'L' as i32 as u_char) != 0 {
                if (*tc).pan_ox > adjust {
                    (*tc).pan_ox = (*tc).pan_ox.wrapping_sub(adjust);
                } else {
                    (*tc).pan_ox = 0 as u_int;
                }
            } else if args_has(args, 'R' as i32 as u_char) != 0 {
                (*tc).pan_ox = (*tc).pan_ox.wrapping_add(adjust);
                if (*tc).pan_ox > (*w).sx.wrapping_sub((*tty).osx) {
                    (*tc).pan_ox = (*w).sx.wrapping_sub((*tty).osx);
                }
            } else if args_has(args, 'U' as i32 as u_char) != 0 {
                if (*tc).pan_oy > adjust {
                    (*tc).pan_oy = (*tc).pan_oy.wrapping_sub(adjust);
                } else {
                    (*tc).pan_oy = 0 as u_int;
                }
            } else if args_has(args, 'D' as i32 as u_char) != 0 {
                (*tc).pan_oy = (*tc).pan_oy.wrapping_add(adjust);
                if (*tc).pan_oy > (*w).sy.wrapping_sub((*tty).osy) {
                    (*tc).pan_oy = (*w).sy.wrapping_sub((*tty).osy);
                }
            }
        }
        tty_update_client_offset(tc);
        server_redraw_client(tc);
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'l' as i32 as u_char) != 0 {
        tty_clipboard_query(&raw mut (*tc).tty);
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'F' as i32 as u_char) != 0 {
        server_client_set_flags(tc, args_get(args, 'F' as i32 as u_char));
    }
    if args_has(args, 'f' as i32 as u_char) != 0 {
        server_client_set_flags(tc, args_get(args, 'f' as i32 as u_char));
    }
    if args_has(args, 'r' as i32 as u_char) != 0 {
        cmd_refresh_report(tty, args_get(args, 'r' as i32 as u_char));
    }
    if args_has(args, 'A' as i32 as u_char) != 0 {
        if !(!(*tc).flags & CLIENT_CONTROL as uint64_t != 0) {
            av = args_first_value(args, 'A' as i32 as u_char);
            while !av.is_null() {
                cmd_refresh_client_update_offset(tc, (*av).c2rust_unnamed.string);
                av = args_next_value(av);
            }
            return CMD_RETURN_NORMAL;
        }
    } else if args_has(args, 'B' as i32 as u_char) != 0 {
        if !(!(*tc).flags & CLIENT_CONTROL as uint64_t != 0) {
            av = args_first_value(args, 'B' as i32 as u_char);
            while !av.is_null() {
                cmd_refresh_client_update_subscription(tc, (*av).c2rust_unnamed.string);
                av = args_next_value(av);
            }
            return CMD_RETURN_NORMAL;
        }
    } else if args_has(args, 'C' as i32 as u_char) != 0 {
        if !(!(*tc).flags & CLIENT_CONTROL as uint64_t != 0) {
            return cmd_refresh_client_control_client_size(self_0, item);
        }
    } else {
        if args_has(args, 'S' as i32 as u_char) != 0 {
            (*tc).flags |= CLIENT_STATUSFORCE as uint64_t;
            server_status_client(tc);
        } else {
            (*tc).flags |= CLIENT_STATUSFORCE as uint64_t;
            server_redraw_client(tc);
        }
        return CMD_RETURN_NORMAL;
    }
    cmdq_error(
        item,
        b"not a control client\0" as *const u8 as *const ::core::ffi::c_char,
    );
    return CMD_RETURN_ERROR;
}

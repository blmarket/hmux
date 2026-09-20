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
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{CMD_AFTERHOOK, CMD_TARGET_WINDOW_USAGE};
pub use crate::src::shared::client::{CLIENT_CONTROL, CLIENT_CONTROL_NEWLAYOUTS};
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

    fn free(__ptr: *mut ::core::ffi::c_void);
    fn events_fire_window(_: *const ::core::ffi::c_char, _: *mut window);
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn cmd_get_entry(_: *mut cmd) -> *const cmd_entry;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn server_redraw_window(_: *mut window);
    fn server_unzoom_window(_: *mut window);
    fn recalculate_sizes();
    fn layout_spread_out(_: *mut window_pane);
    fn layout_dump(
        _: *mut window,
        _: *mut layout_cell,
        _: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn layout_parse(
        _: *mut window,
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn layout_set_lookup(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn layout_set_select(_: *mut window, _: u_int) -> u_int;
    fn layout_set_next(_: *mut window) -> u_int;
    fn layout_set_previous(_: *mut window) -> u_int;
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
pub static mut cmd_select_layout_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"select-layout\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"selectl\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"Enopt:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-Enop] [-t target-pane] [layout-name]\0" as *const u8
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
            cmd_select_layout_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_next_layout_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"next-layout\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"nextl\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_WINDOW_USAGE.as_ptr(),
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_select_layout_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_previous_layout_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"previous-layout\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"prevl\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_WINDOW_USAGE.as_ptr(),
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_select_layout_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_select_layout_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut c: *mut client = cmdq_get_target_client(item);
    let mut wl: *mut winlink = (*target).wl;
    let mut w: *mut window = (*wl).window;
    let mut wp: *mut window_pane = (*target).wp;
    let mut layoutname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut oldlayout: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next: ::core::ffi::c_int = 0;
    let mut previous: ::core::ffi::c_int = 0;
    let mut layout: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    server_unzoom_window(w);
    next = (cmd_get_entry(self_0) == &raw const cmd_next_layout_entry) as ::core::ffi::c_int;
    if args_has(args, 'n' as i32 as u_char) != 0 {
        next = 1 as ::core::ffi::c_int;
    }
    previous =
        (cmd_get_entry(self_0) == &raw const cmd_previous_layout_entry) as ::core::ffi::c_int;
    if args_has(args, 'p' as i32 as u_char) != 0 {
        previous = 1 as ::core::ffi::c_int;
    }
    if !c.is_null()
        && (*c).flags & CLIENT_CONTROL as uint64_t != 0
        && !(*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_NEWLAYOUTS != 0
    {
        flags |= LAYOUT_CUSTOM_OLD_FORMAT;
    }
    oldlayout = (*w).old_layout;
    (*w).old_layout = layout_dump(w, (*w).layout_root, flags);
    if next != 0 || previous != 0 {
        if next != 0 {
            layout_set_next(w);
        } else {
            layout_set_previous(w);
        }
    } else if args_has(args, 'E' as i32 as u_char) != 0 {
        layout_spread_out(wp);
    } else {
        if args_count(args) != 0 as u_int {
            layoutname = args_string(args, 0 as u_int);
        } else if args_has(args, 'o' as i32 as u_char) != 0 {
            layoutname = oldlayout;
        } else {
            layoutname = ::core::ptr::null::<::core::ffi::c_char>();
        }
        if args_has(args, 'o' as i32 as u_char) == 0 {
            if layoutname.is_null() {
                layout = (*w).lastlayout;
            } else {
                layout = layout_set_lookup(layoutname);
            }
            if layout != -(1 as ::core::ffi::c_int) {
                layout_set_select(w, layout as u_int);
                current_block = 16863505586472967431;
            } else {
                current_block = 15125582407903384992;
            }
        } else {
            current_block = 15125582407903384992;
        }
        match current_block {
            16863505586472967431 => {}
            _ => {
                if !layoutname.is_null() {
                    if layout_parse(w, layoutname, &raw mut cause) == -(1 as ::core::ffi::c_int) {
                        cmdq_error(
                            item,
                            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                            cause,
                            layoutname,
                        );
                        free(cause as *mut ::core::ffi::c_void);
                        free((*w).old_layout as *mut ::core::ffi::c_void);
                        (*w).old_layout = oldlayout;
                        return CMD_RETURN_ERROR;
                    }
                } else {
                    free(oldlayout as *mut ::core::ffi::c_void);
                    return CMD_RETURN_NORMAL;
                }
            }
        }
    }
    free(oldlayout as *mut ::core::ffi::c_void);
    recalculate_sizes();
    server_redraw_window(w);
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    return CMD_RETURN_NORMAL;
}

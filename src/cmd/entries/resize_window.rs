use crate::src::arguments::{args_count, args_has, args_string, args_strtonum_result};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd_queue::{cmdq_error, cmdq_get_target};
use crate::src::compat::strtonum::strtonum;
use crate::src::options::options_set_number;
use crate::src::resize::{default_window_size, recalculate_size};
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
pub use crate::src::shared::options::{options, options_entry};
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
pub use crate::src::shared::window::{
    WINDOW_MAXIMUM, WINDOW_MINIMUM, WINDOW_SIZE_LARGEST, WINDOW_SIZE_MANUAL,
};
pub use crate::src::shared::pane::{
    PANE_MINIMUM, window_pane_offset, window_pane_resize, window_pane_resize_entry,
    window_pane_resizes,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
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

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

pub const WINDOW_SIZE_SMALLEST: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
#[no_mangle]
pub static mut cmd_resize_window_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"resize-window\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"resizew\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"aADLRt:Ux:y:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-aADLRU] [-x width] [-y height] [-t target-window] [adjustment]\0" as *const u8
            as *const ::core::ffi::c_char,
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
            cmd_resize_window_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_resize_window_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut wl: *mut winlink = (*target).wl;
    let mut w: *mut window = (*wl).window;
    let mut s: *mut session = (*target).s;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut adjust: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xpixel: u_int = 0 as u_int;
    let mut ypixel: u_int = 0 as u_int;
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
    sx = (*w).sx;
    sy = (*w).sy;
    if args_has(args, 'x' as i32 as u_char) != 0 {
        sx = match args_strtonum_result(
            args,
            'x' as i32 as u_char,
            WINDOW_MINIMUM as ::core::ffi::c_longlong,
            WINDOW_MAXIMUM as ::core::ffi::c_longlong,
        ) {
            Ok(value) => value as u_int,
            Err(error) => {
                cmdq_error(
                    item,
                    b"width %s\0" as *const u8 as *const ::core::ffi::c_char,
                    error.message().as_ptr(),
                );
                return CMD_RETURN_ERROR;
            }
        };
    }
    if args_has(args, 'y' as i32 as u_char) != 0 {
        sy = match args_strtonum_result(
            args,
            'y' as i32 as u_char,
            WINDOW_MINIMUM as ::core::ffi::c_longlong,
            WINDOW_MAXIMUM as ::core::ffi::c_longlong,
        ) {
            Ok(value) => value as u_int,
            Err(error) => {
                cmdq_error(
                    item,
                    b"height %s\0" as *const u8 as *const ::core::ffi::c_char,
                    error.message().as_ptr(),
                );
                return CMD_RETURN_ERROR;
            }
        };
    }
    if args_has(args, 'L' as i32 as u_char) != 0 {
        if sx >= adjust {
            sx = sx.wrapping_sub(adjust);
        }
    } else if args_has(args, 'R' as i32 as u_char) != 0 {
        sx = sx.wrapping_add(adjust);
    } else if args_has(args, 'U' as i32 as u_char) != 0 {
        if sy >= adjust {
            sy = sy.wrapping_sub(adjust);
        }
    } else if args_has(args, 'D' as i32 as u_char) != 0 {
        sy = sy.wrapping_add(adjust);
    }
    if args_has(args, 'A' as i32 as u_char) != 0 {
        default_window_size(
            ::core::ptr::null_mut::<client>(),
            s,
            w,
            &raw mut sx,
            &raw mut sy,
            &raw mut xpixel,
            &raw mut ypixel,
            WINDOW_SIZE_LARGEST,
        );
    } else if args_has(args, 'a' as i32 as u_char) != 0 {
        default_window_size(
            ::core::ptr::null_mut::<client>(),
            s,
            w,
            &raw mut sx,
            &raw mut sy,
            &raw mut xpixel,
            &raw mut ypixel,
            WINDOW_SIZE_SMALLEST,
        );
    }
    options_set_number(
        (*w).options,
        b"window-size\0" as *const u8 as *const ::core::ffi::c_char,
        WINDOW_SIZE_MANUAL as ::core::ffi::c_longlong,
    );
    (*w).manual_sx = sx;
    (*w).manual_sy = sy;
    recalculate_size(w, 1 as ::core::ffi::c_int);
    return CMD_RETURN_NORMAL;
}

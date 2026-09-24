use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::{cmd_get_args, cmd_get_entry};
use crate::src::cmd::find::cmd_find_target;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_source};
use crate::src::ffi::libc::free;
use crate::src::options::options_get_number;
use crate::src::resize::recalculate_sizes;
use crate::src::server_fn::{server_link_window, server_status_session, server_unlink_window};
use crate::src::session::session_renumber_windows;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
use crate::src::shared::client::*;
use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
use crate::src::shared::command::{CMD_FIND_QUIET, CMD_FIND_WINDOW_INDEX};
use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
use crate::src::shared::display::{visible_range, visible_ranges};
use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
use crate::src::shared::hyperlinks::hyperlinks;
use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::options;
use crate::src::shared::pane::{
    window_pane, window_pane_modes, window_pane_prompt, window_pane_tree_entry, window_panes,
};
use crate::src::shared::pane::{window_pane_offset, window_pane_resize, window_pane_resizes};
use crate::src::shared::process::tmuxpeer;
use crate::src::shared::prompt::prompt;
use crate::src::shared::redraw::redraw_scene;
use crate::src::shared::screen::{screen, screen_sel, screen_titles};
use crate::src::shared::screen_write::screen_write_cline;
use crate::src::shared::session::{session, session_entry};
use crate::src::shared::spawn::spawn_editor_state;
use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
use crate::src::shared::window::{
    window, window_entry, window_mode, window_mode_entry, window_winlinks, winlink, winlink_entry,
    winlink_stack, winlinks,
};
use crate::src::window::winlink_shuffle_up;

use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[no_mangle]
pub static mut cmd_move_window_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"move-window\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"movew\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"abdkrs:t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-abdkr] [-s src-window] [-t dst-window]\0" as *const u8
            as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        target: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_move_window_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_link_window_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"link-window\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"linkw\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"abdks:t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-abdk] [-s src-window] [-t dst-window]\0" as *const u8
            as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        target: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_move_window_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_move_window_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut source: *mut cmd_find_state = cmdq_get_source(item);
    let mut target: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut tflag: *const ::core::ffi::c_char = args_get(args, 't' as i32 as u_char);
    let mut src: *mut session = (*source).s;
    let mut dst: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = (*source).wl;
    let mut idx: ::core::ffi::c_int = 0;
    let mut kflag: ::core::ffi::c_int = 0;
    let mut dflag: ::core::ffi::c_int = 0;
    let mut sflag: ::core::ffi::c_int = 0;
    let mut before: ::core::ffi::c_int = 0;
    if args_has(args, 'r' as i32 as u_char) != 0 {
        if cmd_find_target(
            &raw mut target,
            item,
            tflag,
            CMD_FIND_SESSION,
            CMD_FIND_QUIET,
        ) != 0 as ::core::ffi::c_int
        {
            return CMD_RETURN_ERROR;
        }
        session_renumber_windows(target.s);
        recalculate_sizes();
        server_status_session(target.s);
        return CMD_RETURN_NORMAL;
    }
    if cmd_find_target(
        &raw mut target,
        item,
        tflag,
        CMD_FIND_WINDOW,
        CMD_FIND_WINDOW_INDEX,
    ) != 0 as ::core::ffi::c_int
    {
        return CMD_RETURN_ERROR;
    }
    dst = target.s;
    idx = target.idx;
    kflag = args_has(args, 'k' as i32 as u_char);
    dflag = args_has(args, 'd' as i32 as u_char);
    sflag = args_has(args, 's' as i32 as u_char);
    before = args_has(args, 'b' as i32 as u_char);
    if args_has(args, 'a' as i32 as u_char) != 0 || before != 0 {
        if !target.wl.is_null() {
            idx = winlink_shuffle_up(dst, target.wl, before);
        } else {
            idx = winlink_shuffle_up(dst, (*dst).curw, before);
        }
        if idx == -(1 as ::core::ffi::c_int) {
            return CMD_RETURN_ERROR;
        }
    }
    if let Err(cause) = server_link_window(
        src,
        wl,
        dst,
        idx,
        kflag,
        (dflag == 0) as ::core::ffi::c_int,
    )
    {
        cmdq_error(
            item,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            cause.as_ptr(),
        );
        return CMD_RETURN_ERROR;
    }
    if cmd_get_entry(self_0) == &raw const cmd_move_window_entry {
        server_unlink_window(src, wl);
    }
    if sflag == 0
        && options_get_number(
            (*src).options,
            b"renumber-windows\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
    {
        session_renumber_windows(src);
    }
    recalculate_sizes();
    return CMD_RETURN_NORMAL;
}

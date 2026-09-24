use crate::src::arguments::args_has;
use crate::src::cmd::cmd_get_args;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_source, cmdq_get_target};
use crate::src::resize::recalculate_sizes;
use crate::src::server::marked_pane;
use crate::src::server_fn::server_redraw_session_group;
use crate::src::session::{session_group_contains, session_group_synchronize_from, session_select};
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
use crate::src::shared::command::{CMD_AFTERHOOK, CMD_FIND_DEFAULT_MARKED};
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
use crate::src::shared::session::{session_group, session_group_entry};
use crate::src::shared::spawn::spawn_editor_state;
use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
use crate::src::shared::window::{
    window, window_entry, window_mode, window_mode_entry, window_winlinks, winlink, winlink_entry,
    winlink_stack, winlinks,
};
use crate::src::window::{window_winlinks_append, window_winlinks_remove};

use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[no_mangle]
pub static mut cmd_swap_window_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"swap-window\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"swapw\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"ds:t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-d] [-s src-window] [-t dst-window]\0" as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: CMD_FIND_DEFAULT_MARKED,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_swap_window_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_swap_window_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut source: *mut cmd_find_state = cmdq_get_source(item);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut src: *mut session = (*source).s;
    let mut dst: *mut session = (*target).s;
    let mut sg_src: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut sg_dst: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut wl_src: *mut winlink = (*source).wl;
    let mut wl_dst: *mut winlink = (*target).wl;
    let mut w_src: *mut window = ::core::ptr::null_mut::<window>();
    let mut w_dst: *mut window = ::core::ptr::null_mut::<window>();
    sg_src = session_group_contains(src);
    sg_dst = session_group_contains(dst);
    if src != dst && !sg_src.is_null() && !sg_dst.is_null() && sg_src == sg_dst {
        cmdq_error(
            item,
            b"can't move window, sessions are grouped\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if (*wl_dst).window == (*wl_src).window {
        return CMD_RETURN_NORMAL;
    }
    w_dst = (*wl_dst).window;
    w_src = (*wl_src).window;
    window_winlinks_remove(w_dst, wl_dst);
    window_winlinks_remove(w_src, wl_src);
    (*wl_dst).window = w_src;
    window_winlinks_append(w_src, wl_dst);
    (*wl_src).window = w_dst;
    window_winlinks_append(w_dst, wl_src);
    if marked_pane.wl == wl_src {
        marked_pane.wl = wl_dst;
    }
    if args_has(args, 'd' as i32 as u_char) != 0 {
        session_select(dst, (*wl_dst).idx);
        if src != dst {
            session_select(src, (*wl_src).idx);
        }
    }
    session_group_synchronize_from(src);
    server_redraw_session_group(src);
    if src != dst {
        session_group_synchronize_from(dst);
        server_redraw_session_group(dst);
    }
    recalculate_sizes();
    return CMD_RETURN_NORMAL;
}

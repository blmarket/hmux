use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::{cmd_get_args, cmd_get_entry};
use crate::src::cmd_queue::{cmdq_error, cmdq_get_client, cmdq_get_target};
use crate::src::ffi::libc::free;
use crate::src::format::{format_create, format_defaults, format_expand, format_free, format_true};
use crate::src::resize::recalculate_sizes;
use crate::src::server_fn::{server_kill_window, server_renumber_all, server_unlink_window};
use crate::src::session::session_is_linked;
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
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::format::FORMAT_NONE;
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
pub use crate::src::shared::tree::RB_NEGINF;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::window::{winlinks_minmax, winlinks_next, winlinks_prev};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[no_mangle]
pub static mut cmd_kill_window_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"kill-window\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"killw\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"af:t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-a] [-f filter] [-t target-window]\0" as *const u8 as *const ::core::ffi::c_char,
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
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_kill_window_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_unlink_window_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"unlink-window\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"unlinkw\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"kt:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-k] [-t target-window]\0" as *const u8 as *const ::core::ffi::c_char,
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
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_kill_window_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_kill_window_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut wl: *mut winlink = (*target).wl;
    let mut w: *mut window = (*wl).window;
    let mut s: *mut session = (*target).s;
    let mut filter: *const ::core::ffi::c_char = args_get(args, 'f' as i32 as u_char);
    if !filter.is_null() && args_has(args, 'a' as i32 as u_char) == 0 {
        cmdq_error(
            item,
            b"-f only valid with -a\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if cmd_get_entry(self_0) == &raw const cmd_unlink_window_entry {
        if args_has(args, 'k' as i32 as u_char) == 0 && session_is_linked(s, w) == 0 {
            cmdq_error(
                item,
                b"window only linked to one session\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
        server_unlink_window(s, wl);
        recalculate_sizes();
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'a' as i32 as u_char) != 0 {
        return cmd_kill_window_all(item, filter);
    }
    server_kill_window((*wl).window, 1 as ::core::ffi::c_int);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_kill_window_all(
    mut item: *mut cmdq_item,
    mut filter: *const ::core::ffi::c_char,
) -> cmd_retval {
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut s: *mut session = (*target).s;
    let mut wl: *mut winlink = (*target).wl;
    let mut loop_0: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut found: u_int = 0;
    let mut kill_current: u_int = 0;
    if winlinks_prev(wl).is_null() && winlinks_next(wl).is_null() {
        return CMD_RETURN_NORMAL;
    }
    loop {
        found = 0 as u_int;
        loop_0 = winlinks_minmax(&raw mut (*s).windows, RB_NEGINF);
        while !loop_0.is_null() {
            if (*loop_0).window != (*wl).window
                && cmd_kill_window_filter(item, s, loop_0, filter) != 0
            {
                server_kill_window((*loop_0).window, 0 as ::core::ffi::c_int);
                found = found.wrapping_add(1);
                break;
            } else {
                loop_0 = winlinks_next(loop_0);
            }
        }
        if !(found != 0 as u_int) {
            break;
        }
    }
    kill_current = 0 as u_int;
    found = kill_current;
    loop_0 = winlinks_minmax(&raw mut (*s).windows, RB_NEGINF);
    while !loop_0.is_null() {
        if (*loop_0).window == (*wl).window {
            found = found.wrapping_add(1);
            if cmd_kill_window_filter(item, s, loop_0, filter) != 0 {
                kill_current = 1 as u_int;
            }
        }
        loop_0 = winlinks_next(loop_0);
    }
    if kill_current != 0 && found > 1 as u_int {
        server_kill_window((*wl).window, 0 as ::core::ffi::c_int);
    }
    server_renumber_all();
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_kill_window_filter(
    mut item: *mut cmdq_item,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut filter: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut flag: ::core::ffi::c_int = 0;
    if filter.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    ft = format_create(
        cmdq_get_client(item),
        item,
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(
        ft,
        ::core::ptr::null_mut::<client>(),
        s,
        wl,
        ::core::ptr::null_mut::<window_pane>(),
    );
    expanded = format_expand(ft, filter);
    flag = format_true(expanded);
    free(expanded as *mut ::core::ffi::c_void);
    format_free(ft);
    return flag;
}

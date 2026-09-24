use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_target, cmdq_print};
use crate::src::format::{
    format_add, format_create, format_defaults, format_expand_cstring, format_free, format_true,
};
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
use crate::src::shared::client::*;
use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files, overlay_check_cb,
    overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb, overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::CMD_AFTERHOOK;
use crate::src::shared::command::*;
use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
use crate::src::shared::display::{visible_range, visible_ranges};
use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
use crate::src::shared::format::FORMAT_NONE;
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
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
use crate::src::shared::spawn::spawn_editor_state;
use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
use crate::src::shared::window::{
    window, window_entry, window_mode, window_mode_entry, window_winlinks, winlink, winlink_entry,
    winlink_stack, winlinks,
};
use crate::src::sort::{sort_get_winlinks, sort_get_winlinks_session, sort_order_from_string};

use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub const LIST_WINDOWS_WITH_SESSION_TEMPLATE: [::core::ffi::c_char; 127] = unsafe {
    ::core::mem::transmute::<
        [u8; 127],
        [::core::ffi::c_char; 127],
    >(
        *b"#{session_name}:#{window_index}: #{window_name}#{window_raw_flags} (#{window_panes} panes) [#{window_width}x#{window_height}] \0",
    )
};
#[no_mangle]
pub static mut cmd_list_windows_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"list-windows\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"lsw\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"aF:f:O:rt:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-ar] [-F format] [-f filter] [-O order][-t target-session]\0" as *const u8
            as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_list_windows_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_list_windows_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut c: *mut client = cmdq_get_client(item);
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut i: u_int = 0;
    let mut n: u_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut filter: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut flag: ::core::ffi::c_int = 0;
    let mut sort_crit: sort_criteria = sort_criteria {
        order: SORT_ACTIVITY,
        reversed: 0,
        order_seq: ::core::ptr::null_mut::<sort_order>(),
    };
    template = args_get(args, 'F' as i32 as u_char);
    filter = args_get(args, 'f' as i32 as u_char);
    sort_crit.order = sort_order_from_string(args_get(args, 'O' as i32 as u_char));
    if sort_crit.order as ::core::ffi::c_uint
        == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
        && args_has(args, 'O' as i32 as u_char) != 0
    {
        cmdq_error(
            item,
            b"invalid sort order\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    sort_crit.reversed = args_has(args, 'r' as i32 as u_char);
    let winlinks = if args_has(args, 'a' as i32 as u_char) != 0 {
        let links = sort_get_winlinks(&raw mut sort_crit);
        if template.is_null() {
            template = LIST_WINDOWS_WITH_SESSION_TEMPLATE.as_ptr();
        }
        links
    } else {
        let links = sort_get_winlinks_session((*target).s, &raw mut sort_crit);
        if template.is_null() {
            template = b"#{window_index}: #{window_name}#{window_raw_flags} (#{window_panes} panes) [#{window_width}x#{window_height}] [layout #{window_layout}] #{window_id}#{?window_active, (active),}\0"
                as *const u8 as *const ::core::ffi::c_char;
        }
        links
    };
    n = u_int::try_from(winlinks.len()).expect("too many winlinks to list");
    i = 0 as u_int;
    while i < n {
        wl = winlinks[i as usize];
        s = (*wl).session;
        ft = format_create(
            cmdq_get_client(item),
            item,
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        );
        format_add(
            ft,
            b"line\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
        format_defaults(ft, c, s, wl, ::core::ptr::null_mut::<window_pane>());
        if !filter.is_null() {
            let expanded = format_expand_cstring(ft, filter);
            flag = format_true(expanded.as_ptr());
        } else {
            flag = 1 as ::core::ffi::c_int;
        }
        if flag != 0 {
            let line = format_expand_cstring(ft, template);
            cmdq_print(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                line.as_ptr(),
            );
        }
        format_free(ft);
        i = i.wrapping_add(1);
    }
    return CMD_RETURN_NORMAL;
}

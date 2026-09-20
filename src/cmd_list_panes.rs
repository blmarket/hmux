pub use crate::src::shared::session::{sessions};
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
pub use crate::src::shared::format::{FORMAT_NONE};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::tree::{RB_NEGINF};
pub use crate::src::shared::command::{CMD_AFTERHOOK};
pub use crate::src::shared::sort::{sort_criteria};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::arguments::*;
use crate::src::shared::sort::*;
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
    fn sort_order_from_string(_: *const ::core::ffi::c_char) -> sort_order;
    fn sort_get_panes_window(
        _: *mut window,
        _: *mut u_int,
        _: *mut sort_criteria,
    ) -> *mut *mut window_pane;
    fn format_true(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn format_create(
        _: *mut client,
        _: *mut cmdq_item,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut format_tree;
    fn format_free(_: *mut format_tree);
    fn format_add(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn format_expand(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn format_defaults(
        _: *mut format_tree,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    );
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_print(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn winlinks_RB_NEXT(_: *mut winlink) -> *mut winlink;
    fn winlinks_RB_MINMAX(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    static mut sessions: sessions;
    fn sessions_RB_NEXT(_: *mut session) -> *mut session;
    fn sessions_RB_MINMAX(_: *mut sessions, _: ::core::ffi::c_int) -> *mut session;
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
pub static mut cmd_list_panes_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"list-panes\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"lsp\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"aF:f:O:rst:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-asr] [-F format] [-f filter] [-O order][-t target-window]\0" as *const u8
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
            cmd_list_panes_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_list_panes_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut s: *mut session = (*target).s;
    let mut wl: *mut winlink = (*target).wl;
    let mut order: sort_order = SORT_ACTIVITY;
    order = sort_order_from_string(args_get(args, 'O' as i32 as u_char));
    if order as ::core::ffi::c_uint == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
        && args_has(args, 'O' as i32 as u_char) != 0
    {
        cmdq_error(
            item,
            b"invalid sort order\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'a' as i32 as u_char) != 0 {
        cmd_list_panes_server(self_0, item);
    } else if args_has(args, 's' as i32 as u_char) != 0 {
        cmd_list_panes_session(self_0, s, item, 1 as ::core::ffi::c_int);
    } else {
        cmd_list_panes_window(self_0, s, wl, item, 0 as ::core::ffi::c_int);
    }
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_list_panes_server(mut self_0: *mut cmd, mut item: *mut cmdq_item) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        cmd_list_panes_session(self_0, s, item, 2 as ::core::ffi::c_int);
        s = sessions_RB_NEXT(s);
    }
}
unsafe extern "C" fn cmd_list_panes_session(
    mut self_0: *mut cmd,
    mut s: *mut session,
    mut item: *mut cmdq_item,
    mut type_0: ::core::ffi::c_int,
) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        cmd_list_panes_window(self_0, s, wl, item, type_0);
        wl = winlinks_RB_NEXT(wl);
    }
}
unsafe extern "C" fn cmd_list_panes_window(
    mut self_0: *mut cmd,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut item: *mut cmdq_item,
    mut type_0: ::core::ffi::c_int,
) {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut c: *mut client = cmdq_get_client(item);
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut l: *mut *mut window_pane = ::core::ptr::null_mut::<*mut window_pane>();
    let mut i: u_int = 0;
    let mut n: u_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut filter: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut flag: ::core::ffi::c_int = 0;
    let mut sort_crit: sort_criteria = sort_criteria {
        order: SORT_ACTIVITY,
        reversed: 0,
        order_seq: ::core::ptr::null_mut::<sort_order>(),
    };
    template = args_get(args, 'F' as i32 as u_char);
    if template.is_null() {
        match type_0 {
            0 => {
                template = b"#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}\0"
                    as *const u8 as *const ::core::ffi::c_char;
            }
            1 => {
                template = b"#{window_index}.#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}\0"
                    as *const u8 as *const ::core::ffi::c_char;
            }
            2 => {
                template = b"#{session_name}:#{window_index}.#{pane_index}: [#{pane_width}x#{pane_height}#{?pane_floating_flag, #{pane_x}#,#{pane_y}#,#{pane_z}}] [history #{history_size}/#{history_limit}, #{history_bytes} bytes] #{pane_id}#{?pane_active, (active),}#{?pane_dead, (dead),}\0"
                    as *const u8 as *const ::core::ffi::c_char;
            }
            _ => {}
        }
    }
    filter = args_get(args, 'f' as i32 as u_char);
    sort_crit.order = sort_order_from_string(args_get(args, 'O' as i32 as u_char));
    sort_crit.reversed = args_has(args, 'r' as i32 as u_char);
    l = sort_get_panes_window((*wl).window, &raw mut n, &raw mut sort_crit);
    i = 0 as u_int;
    while i < n {
        wp = *l.offset(i as isize);
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
        format_defaults(ft, c, s, wl, wp);
        if !filter.is_null() {
            expanded = format_expand(ft, filter);
            flag = format_true(expanded);
            free(expanded as *mut ::core::ffi::c_void);
        } else {
            flag = 1 as ::core::ffi::c_int;
        }
        if flag != 0 {
            line = format_expand(ft, template);
            cmdq_print(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                line,
            );
            free(line as *mut ::core::ffi::c_void);
        }
        format_free(ft);
        i = i.wrapping_add(1);
    }
}

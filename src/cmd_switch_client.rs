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
pub use crate::src::shared::abi::{__uid_t, uid_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{
    CMDQ_STATE_REPEAT, CMD_CLIENT_CFLAG, CMD_FIND_PREFER_UNATTACHED, CMD_READONLY,
};
pub use crate::src::shared::client::{CLIENT_IGNORESIZE, CLIENT_READONLY};
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

    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strcspn(
        __s: *const ::core::ffi::c_char,
        __reject: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_ulong;
    fn getuid() -> __uid_t;
    fn proc_get_peer_uid(_: *mut tmuxpeer) -> uid_t;
    fn sort_order_from_string(_: *const ::core::ffi::c_char) -> sort_order;
    fn environ_update(_: *mut options, _: *mut environ, _: *mut environ);
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn cmd_find_target(
        _: *mut cmd_find_state,
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
        _: cmd_find_type,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_find_from_session(_: *mut cmd_find_state, _: *mut session, _: ::core::ffi::c_int);
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_current(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_flags(_: *mut cmdq_item) -> ::core::ffi::c_int;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn key_bindings_get_table(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut key_table;
    fn key_bindings_unref_table(_: *mut key_table);
    fn server_client_set_key_table(_: *mut client, _: *const ::core::ffi::c_char);
    fn server_client_set_session(_: *mut client, _: *mut session);
    fn server_redraw_window(_: *mut window);
    fn window_set_active_pane(
        _: *mut window,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn window_redraw_active_switch(_: *mut window, _: *mut window_pane);
    fn window_push_zoom(
        _: *mut window,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn window_pop_zoom(_: *mut window) -> ::core::ffi::c_int;
    fn window_pane_is_visible(_: *mut window_pane) -> ::core::ffi::c_int;
    fn session_alive(_: *mut session) -> ::core::ffi::c_int;
    fn session_next_session(_: *mut session, _: *mut sort_criteria) -> *mut session;
    fn session_previous_session(_: *mut session, _: *mut sort_criteria) -> *mut session;
    fn session_set_current(_: *mut session, _: *mut winlink) -> ::core::ffi::c_int;
}

pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;
pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;

#[no_mangle]
pub static mut cmd_switch_client_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"switch-client\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"switchc\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"c:EFlnO:pt:rT:Z\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-ElnprZ] [-c target-client] [-t target-session] [-T key-table] [-O order]\0"
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
        flags: CMD_READONLY | CMD_CLIENT_CFLAG,
        exec: Some(
            cmd_switch_client_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_switch_client_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut current: *mut cmd_find_state = cmdq_get_current(item);
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
    let mut type_0: cmd_find_type = CMD_FIND_PANE;
    let mut flags: ::core::ffi::c_int = 0;
    let mut visible: ::core::ffi::c_int = 0;
    let mut Zflag: ::core::ffi::c_int = args_has(args, 'Z' as i32 as u_char);
    let mut c: *mut client = cmdq_get_client(item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut tablename: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut sort_crit: sort_criteria = sort_criteria {
        order: SORT_ACTIVITY,
        reversed: 0,
        order_seq: ::core::ptr::null_mut::<sort_order>(),
    };
    let mut uid: uid_t = 0;
    if !tflag.is_null()
        && (*tflag
            .offset(strcspn(tflag, b":.%\0" as *const u8 as *const ::core::ffi::c_char) as isize)
            as ::core::ffi::c_int
            != '\0' as i32
            || strcmp(tflag, b"=\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int)
    {
        type_0 = CMD_FIND_PANE;
        flags = 0 as ::core::ffi::c_int;
    } else {
        type_0 = CMD_FIND_SESSION;
        flags = CMD_FIND_PREFER_UNATTACHED;
    }
    if cmd_find_target(&raw mut target, item, tflag, type_0, flags) != 0 as ::core::ffi::c_int {
        return CMD_RETURN_ERROR;
    }
    s = target.s;
    wl = target.wl;
    wp = target.wp;
    if args_has(args, 'r' as i32 as u_char) != 0 {
        if (*tc).flags & CLIENT_READONLY as uint64_t != 0 {
            uid = proc_get_peer_uid((*c).peer);
            if uid != getuid() {
                cmdq_error(
                    item,
                    b"client is read-only\0" as *const u8 as *const ::core::ffi::c_char,
                );
                return CMD_RETURN_ERROR;
            }
        }
        if (*tc).flags & CLIENT_READONLY as uint64_t != 0 {
            (*tc).flags &= !(CLIENT_READONLY | CLIENT_IGNORESIZE) as uint64_t;
        } else {
            (*tc).flags |= (CLIENT_READONLY | CLIENT_IGNORESIZE) as uint64_t;
        }
    }
    tablename = args_get(args, 'T' as i32 as u_char);
    if !tablename.is_null() {
        table = key_bindings_get_table(tablename, 0 as ::core::ffi::c_int);
        if table.is_null() {
            cmdq_error(
                item,
                b"table %s doesn't exist\0" as *const u8 as *const ::core::ffi::c_char,
                tablename,
            );
            return CMD_RETURN_ERROR;
        }
        (*table).references = (*table).references.wrapping_add(1);
        key_bindings_unref_table((*tc).keytable as *mut key_table);
        (*tc).keytable = table as *mut key_table;
        return CMD_RETURN_NORMAL;
    }
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
    if args_has(args, 'n' as i32 as u_char) != 0 {
        s = session_next_session((*tc).session, &raw mut sort_crit);
        if s.is_null() {
            cmdq_error(
                item,
                b"can't find next session\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
    } else if args_has(args, 'p' as i32 as u_char) != 0 {
        s = session_previous_session((*tc).session, &raw mut sort_crit);
        if s.is_null() {
            cmdq_error(
                item,
                b"can't find previous session\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
    } else if args_has(args, 'l' as i32 as u_char) != 0 {
        if !(*tc).last_session.is_null() && session_alive((*tc).last_session) != 0 {
            s = (*tc).last_session;
        } else {
            s = ::core::ptr::null_mut::<session>();
        }
        if s.is_null() {
            cmdq_error(
                item,
                b"can't find last session\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
    } else {
        if cmdq_get_client(item).is_null() {
            return CMD_RETURN_NORMAL;
        }
        if !wl.is_null() && !wp.is_null() && wp != (*(*wl).window).active {
            w = (*wl).window;
            if !(*w).modal.is_null() && wp != (*w).modal {
                visible = 1 as ::core::ffi::c_int;
            } else {
                visible = window_pane_is_visible(wp);
            }
            if visible == 0 && window_push_zoom(w, 0 as ::core::ffi::c_int, Zflag) != 0 {
                server_redraw_window(w);
            }
            window_redraw_active_switch(w, wp);
            window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
            if visible == 0 && window_pop_zoom(w) != 0 {
                server_redraw_window(w);
            }
        }
        if !wl.is_null() {
            session_set_current(s, wl);
            cmd_find_from_session(current, s, 0 as ::core::ffi::c_int);
        }
    }
    if args_has(args, 'E' as i32 as u_char) == 0 {
        environ_update((*s).options, (*tc).environ, (*s).environ);
    }
    server_client_set_session(tc, s);
    if !cmdq_get_flags(item) & CMDQ_STATE_REPEAT != 0 {
        server_client_set_key_table(tc, ::core::ptr::null::<::core::ffi::c_char>());
    }
    return CMD_RETURN_NORMAL;
}

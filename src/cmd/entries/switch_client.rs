use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd_find::{cmd_find_from_session, cmd_find_target};
use crate::src::cmd_queue::{
    cmdq_error, cmdq_get_client, cmdq_get_current, cmdq_get_flags, cmdq_get_target_client,
};
use crate::src::environ::environ_update;
use crate::src::ffi::libc::{getuid, strcmp, strcspn};
use crate::src::key_bindings::{key_bindings_get_table, key_bindings_unref_table};
use crate::src::proc::proc_get_peer_uid;
use crate::src::server_client::{server_client_set_key_table, server_client_set_session};
use crate::src::server_fn::server_redraw_window;
use crate::src::session::{
    session_alive, session_next_session, session_previous_session, session_set_current,
};
use crate::src::shared::abi::*;
pub use crate::src::shared::abi::{__uid_t, uid_t};
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{args, args_parse, args_parse_cb};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::client::{CLIENT_IGNORESIZE, CLIENT_READONLY};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::command::{
    CMDQ_STATE_REPEAT, CMD_CLIENT_CFLAG, CMD_FIND_PREFER_UNATTACHED, CMD_READONLY,
};
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
pub use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
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
use crate::src::sort::sort_order_from_string;
use crate::src::window::{
    window_pane_is_visible, window_pop_zoom, window_push_zoom, window_redraw_active_switch,
    window_set_active_pane,
};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

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

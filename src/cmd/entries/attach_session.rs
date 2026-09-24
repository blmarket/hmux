use crate::src::arguments::{args_get, args_has};
use crate::src::cfg::{cfg_finished, cfg_show_causes};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd_find::{cmd_find_from_winlink, cmd_find_from_winlink_pane, cmd_find_target};
use crate::src::cmd_queue::{cmdq_error, cmdq_get_client, cmdq_get_current, cmdq_get_flags};
use crate::src::environ::environ_update;
use crate::src::events::events_fire_client;
use crate::src::ffi::libc::{getuid, strcspn};
use crate::src::format::format_single_cstring;
use crate::src::proc::{proc_get_peer_uid, proc_send};
pub use crate::src::server::clients;
use crate::src::server_client::{
    server_client_check_nested, server_client_detach, server_client_open, server_client_set_flags,
    server_client_set_key_table, server_client_set_session,
};
pub use crate::src::session::sessions;
use crate::src::session::{session_set_current, session_set_cwd};
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
pub use crate::src::shared::client::{
    CLIENT_ATTACHED, CLIENT_CONTROL, CLIENT_IGNORESIZE, CLIENT_READONLY,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds,
};
pub use crate::src::shared::command::{
    CMDQ_STATE_REPEAT, CMD_FIND_PREFER_UNATTACHED, CMD_READONLY, CMD_STARTSERVER,
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
    window_pane, window_pane_modes, window_pane_prompt, window_pane_tree_entry, window_panes,
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
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::window::window_set_active_pane;

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[no_mangle]
pub static mut cmd_attach_session_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"attach-session\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"attach\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"c:dEf:rt:x\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-dErx] [-c working-directory] [-f flags] [-t target-session]\0" as *const u8
            as *const ::core::ffi::c_char,
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
        flags: CMD_STARTSERVER | CMD_READONLY,
        exec: Some(
            cmd_attach_session_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub unsafe extern "C" fn cmd_attach_session(
    mut item: *mut cmdq_item,
    mut tflag: *const ::core::ffi::c_char,
    mut dflag: ::core::ffi::c_int,
    mut xflag: ::core::ffi::c_int,
    mut rflag: ::core::ffi::c_int,
    mut cflag: *const ::core::ffi::c_char,
    mut Eflag: ::core::ffi::c_int,
    mut fflag: *const ::core::ffi::c_char,
) -> cmd_retval {
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
    let mut type_0: cmd_find_type = CMD_FIND_PANE;
    let mut flags: ::core::ffi::c_int = 0;
    let mut c: *mut client = cmdq_get_client(item);
    let mut c_loop: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut msgtype: msgtype = 0 as msgtype;
    let mut uid: uid_t = 0;
    if sessions.storage.is_none() {
        cmdq_error(
            item,
            b"no sessions\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if c.is_null() {
        return CMD_RETURN_NORMAL;
    }
    if server_client_check_nested(c) != 0 {
        cmdq_error(
            item,
            b"sessions should be nested with care, unset $TMUX to force\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if !tflag.is_null()
        && *tflag
            .offset(strcspn(tflag, b":.\0" as *const u8 as *const ::core::ffi::c_char) as isize)
            as ::core::ffi::c_int
            != '\0' as i32
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
    if !wl.is_null() {
        if !wp.is_null() {
            window_set_active_pane((*wp).window as *mut window, wp, 1 as ::core::ffi::c_int);
        }
        session_set_current(s, wl);
        if !wp.is_null() {
            cmd_find_from_winlink_pane(current, wl, wp, 0 as ::core::ffi::c_int);
        } else {
            cmd_find_from_winlink(current, wl, 0 as ::core::ffi::c_int);
        }
    }
    if !cflag.is_null() {
        session_set_cwd(s, Some(format_single_cstring(item, cflag, c, s, wl, wp)));
    }
    if !fflag.is_null() {
        server_client_set_flags(c, fflag);
    }
    if rflag != 0 {
        if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
            uid = proc_get_peer_uid((*c).peer);
            if uid != getuid() {
                cmdq_error(
                    item,
                    b"client is read-only\0" as *const u8 as *const ::core::ffi::c_char,
                );
                return CMD_RETURN_ERROR;
            }
        }
        (*c).flags |= (CLIENT_READONLY | CLIENT_IGNORESIZE) as uint64_t;
    }
    (*c).last_session = (*c).session;
    if !(*c).session.is_null() {
        if dflag != 0 || xflag != 0 {
            if xflag != 0 {
                msgtype = MSG_DETACHKILL;
            } else {
                msgtype = MSG_DETACH;
            }
            c_loop = clients.first();
            while !c_loop.is_null() {
                if !((*c_loop).session != s || c == c_loop) {
                    server_client_detach(c_loop, msgtype);
                }
                c_loop = clients.next(c_loop);
            }
        }
        if Eflag == 0 {
            environ_update((*s).options, (*c).environ, (*s).environ);
        }
        server_client_set_session(c, s);
        if !cmdq_get_flags(item) & CMDQ_STATE_REPEAT != 0 {
            server_client_set_key_table(c, ::core::ptr::null::<::core::ffi::c_char>());
        }
    } else {
        if let Err(cause) = server_client_open(c) {
            cmdq_error(
                item,
                b"open terminal failed: %s\0" as *const u8 as *const ::core::ffi::c_char,
                cause.as_ptr(),
            );
            return CMD_RETURN_ERROR;
        }
        if dflag != 0 || xflag != 0 {
            if xflag != 0 {
                msgtype = MSG_DETACHKILL;
            } else {
                msgtype = MSG_DETACH;
            }
            c_loop = clients.first();
            while !c_loop.is_null() {
                if !((*c_loop).session != s || c == c_loop) {
                    server_client_detach(c_loop, msgtype);
                }
                c_loop = clients.next(c_loop);
            }
        }
        if Eflag == 0 {
            environ_update((*s).options, (*c).environ, (*s).environ);
        }
        server_client_set_session(c, s);
        server_client_set_key_table(c, ::core::ptr::null::<::core::ffi::c_char>());
        if !(*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            proc_send(
                (*c).peer,
                MSG_READY,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
        }
        events_fire_client(
            b"client-attached\0" as *const u8 as *const ::core::ffi::c_char,
            c,
        );
        (*c).flags |= CLIENT_ATTACHED as uint64_t;
    }
    if cfg_finished != 0 {
        cfg_show_causes(s);
    }
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_attach_session_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    return cmd_attach_session(
        item,
        args_get(args, 't' as i32 as u_char),
        args_has(args, 'd' as i32 as u_char),
        args_has(args, 'x' as i32 as u_char),
        args_has(args, 'r' as i32 as u_char),
        args_get(args, 'c' as i32 as u_char),
        args_has(args, 'E' as i32 as u_char),
        args_get(args, 'f' as i32 as u_char),
    );
}

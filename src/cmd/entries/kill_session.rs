use crate::src::arguments::{args_get, args_has};
use crate::src::cmd::cmd_get_args;
use crate::src::cmd_queue::{cmdq_error, cmdq_get_client, cmdq_get_target};
use crate::src::ffi::libc::free;
use crate::src::format::{format_create, format_defaults, format_expand, format_free, format_true};
use crate::src::server_fn::{server_destroy_session, server_redraw_session};
pub use crate::src::session::sessions;
use crate::src::session::{
    session_destroy, session_group_contains, sessions_RB_MINMAX, sessions_RB_NEXT,
};
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
pub use crate::src::shared::session::{session_group, session_group_entry, session_group_sessions};
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
pub use crate::src::shared::window::{
    WINDOW_ACTIVITY, WINDOW_ALERTFLAGS, WINDOW_BELL, WINDOW_SILENCE, WINLINK_ACTIVITY,
    WINLINK_ALERTFLAGS, WINLINK_BELL, WINLINK_SILENCE,
};
use crate::src::window::{winlinks_RB_MINMAX, winlinks_RB_NEXT};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

#[no_mangle]
pub static mut cmd_kill_session_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"kill-session\0" as *const u8 as *const ::core::ffi::c_char,
        alias: ::core::ptr::null::<::core::ffi::c_char>(),
        args: args_parse {
            template: b"aCgf:t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-aCg] [-f filter] [-t target-session]\0" as *const u8
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
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_kill_session_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_kill_session_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut s: *mut session = (*target).s;
    let mut sloop: *mut session = ::core::ptr::null_mut::<session>();
    let mut stmp: *mut session = ::core::ptr::null_mut::<session>();
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut filter: *const ::core::ffi::c_char = args_get(args, 'f' as i32 as u_char);
    if !filter.is_null()
        && (args_has(args, 'a' as i32 as u_char) == 0 || args_has(args, 'C' as i32 as u_char) != 0)
    {
        cmdq_error(
            item,
            b"-f only valid with -a\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'C' as i32 as u_char) != 0 {
        wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
        while !wl.is_null() {
            (*(*wl).window).flags &= !WINDOW_ALERTFLAGS;
            (*wl).flags &= !WINLINK_ALERTFLAGS;
            wl = winlinks_RB_NEXT(wl);
        }
        server_redraw_session(s);
    } else if args_has(args, 'a' as i32 as u_char) != 0 {
        return cmd_kill_session_all(item, filter);
    } else if args_has(args, 'g' as i32 as u_char) != 0 && {
        sg = session_group_contains(s);
        !sg.is_null()
    } {
        sloop = (*sg).sessions.tqh_first;
        while !sloop.is_null() && {
            stmp = (*sloop).gentry.tqe_next;
            1 as ::core::ffi::c_int != 0
        } {
            server_destroy_session(sloop);
            session_destroy(
                sloop,
                1 as ::core::ffi::c_int,
                b"cmd_kill_session_exec\0" as *const u8 as *const ::core::ffi::c_char,
            );
            sloop = stmp;
        }
    } else {
        server_destroy_session(s);
        session_destroy(
            s,
            1 as ::core::ffi::c_int,
            b"cmd_kill_session_exec\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_kill_session_all(
    mut item: *mut cmdq_item,
    mut filter: *const ::core::ffi::c_char,
) -> cmd_retval {
    let mut s: *mut session = (*cmdq_get_target(item)).s;
    let mut sloop: *mut session = ::core::ptr::null_mut::<session>();
    let mut stmp: *mut session = ::core::ptr::null_mut::<session>();
    sloop = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !sloop.is_null() && {
        stmp = sessions_RB_NEXT(sloop);
        1 as ::core::ffi::c_int != 0
    } {
        if !(sloop == s) {
            if !(cmd_kill_session_filter(item, sloop, filter) == 0) {
                server_destroy_session(sloop);
                session_destroy(
                    sloop,
                    1 as ::core::ffi::c_int,
                    b"cmd_kill_session_all\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        sloop = stmp;
    }
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_kill_session_filter(
    mut item: *mut cmdq_item,
    mut s: *mut session,
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
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    expanded = format_expand(ft, filter);
    flag = format_true(expanded);
    free(expanded as *mut ::core::ffi::c_void);
    format_free(ft);
    return flag;
}

pub use crate::src::shared::session::{
    session_group, session_group_entry, session_group_sessions, sessions,
};
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
pub use crate::src::shared::window::{
    WINDOW_ACTIVITY, WINDOW_ALERTFLAGS, WINDOW_BELL, WINDOW_SILENCE, WINLINK_ACTIVITY,
    WINLINK_ALERTFLAGS, WINLINK_BELL, WINLINK_SILENCE,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::tree::{RB_NEGINF};
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
extern "C" {

    fn free(__ptr: *mut ::core::ffi::c_void);
    fn format_true(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn format_create(
        _: *mut client,
        _: *mut cmdq_item,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut format_tree;
    fn format_free(_: *mut format_tree);
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
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn server_redraw_session(_: *mut session);
    fn server_destroy_session(_: *mut session);
    fn winlinks_RB_MINMAX(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn winlinks_RB_NEXT(_: *mut winlink) -> *mut winlink;
    static mut sessions: sessions;
    fn sessions_RB_MINMAX(_: *mut sessions, _: ::core::ffi::c_int) -> *mut session;
    fn sessions_RB_NEXT(_: *mut session) -> *mut session;
    fn session_destroy(_: *mut session, _: ::core::ffi::c_int, _: *const ::core::ffi::c_char);
    fn session_group_contains(_: *mut session) -> *mut session_group;
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

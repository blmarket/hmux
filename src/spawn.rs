use crate::src::cmd::{cmd_copy_argv, cmd_free_argv, cmd_log_argv, cmd_stringify_argv};
use crate::src::cmd_find::cmd_find_from_winlink_pane;
use crate::src::cmd_queue::{cmdq_get_client, cmdq_get_target};
use crate::src::compat::fdforkpty::fdforkpty;
use crate::src::compat::systemd::systemd_move_to_new_cgroup;
use crate::src::control::control_reset_pane;
use crate::src::environ::{
    environ_copy, environ_find, environ_for_session, environ_log, environ_push, environ_set,
    EnvironOwner,
};
use crate::src::events::{events_fire, events_fire_window, events_fire_winlink};
use crate::src::events_payload::{
    event_payload_create, event_payload_set_int, event_payload_set_pane, event_payload_set_session,
    event_payload_set_string, event_payload_set_target, event_payload_set_window,
};
use crate::src::ffi::libc::{
    __errno_location, _exit, chdir, close, closefrom, execl, execvp, fclose, fdopen, fopen, fread,
    free, fseeko, ftello, fwrite, getcwd, getpid, kill, malloc, memcpy, memset, mkstemp,
    sigfillset, sigprocmask, strerror, strrchr, tcgetattr, tcsetattr, unlink,
};
use crate::src::ffi::utempter::utempter_add_record;
use crate::src::format::format_single;
use crate::src::input::input_free;
use crate::src::layout::{
    layout_assign_pane, layout_close_pane, layout_floating_pane, layout_free, layout_init,
};
use crate::src::log::{log_close, log_debug};
use crate::src::names::default_window_name;
use crate::src::options::{options_get_number, options_get_string, options_set_number};
use crate::src::proc::proc_clear_signals;
use crate::src::reactor::bufferevent_free;
use crate::src::resize::default_window_size;
use crate::src::screen::screen_reinit;
pub use crate::src::server::clients;
use crate::src::server::server_proc;
use crate::src::server_client::{server_client_get_cwd, server_client_remove_pane};
use crate::src::session::{session_group_synchronize_from, session_select};
pub use crate::src::shared::events::event_payload;
pub use crate::src::shared::spawn::spawn_context;
use crate::src::tmux::{checkshell, find_home, global_options, ptm_fd};
pub use crate::src::window::window_pane_resize;
use crate::src::window::{
    window_add_pane, window_create, window_destroy_panes, window_pane_index,
    window_pane_reset_mode_all, window_pane_set_event, window_pop_zoom, window_push_zoom,
    window_redraw_active_switch, window_remove_pane, window_set_active_pane, winlink_add,
    winlink_find_by_index, winlink_remove, winlink_set_window, winlink_stack_remove,
};
use crate::src::window_border::window_set_fill_cells;
use crate::src::xmalloc::{xasprintf, xcalloc, xsnprintf, xstrdup};

use crate::src::shared::abi::*;
pub use crate::src::shared::abi::{__off64_t, __off_t};
pub use crate::src::shared::arguments::args;
pub use crate::src::shared::client::CLIENT_CONTROL;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::{environ, environ_entry};
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
pub use crate::src::shared::limits::SIZE_MAX;
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::{options, options_entry};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize_entry, window_pane_resizes, PANE_EMPTY, PANE_EXITED,
    PANE_FLOATOVERZOOM, PANE_STATUSDRAWN, PANE_STATUSREADY,
};
pub use crate::src::shared::posix_io::{_PATH_BSHELL, STDERR_FILENO, STDIN_FILENO};
pub use crate::src::shared::posix_terminal::{winsize, TCSANOW, VERASE};
pub use crate::src::shared::process::{tmuxpeer, tmuxproc};
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles, MODE_CRLF, MODE_CURSOR};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::signal::{
    __sigset_t, sigset_t, SIGCHLD, SIGHUP, SIG_BLOCK, SIG_SETMASK,
};
pub use crate::src::shared::spawn::{spawn_editor_state, spawn_finish_edit_cb};
pub use crate::src::shared::spawn::{
    SPAWN_DETACHED, SPAWN_EMPTY, SPAWN_FLOATING, SPAWN_FLOATOVERZOOM, SPAWN_KILL, SPAWN_MODAL,
    SPAWN_NONOTIFY, SPAWN_RESPAWN, SPAWN_ZOOM,
};
pub use crate::src::shared::status::status_line;
pub use crate::src::shared::stdio::{
    _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, _IO_FILE, FILE,
};
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::window::{
    WINDOW_ZOOMED, WINLINK_ACTIVITY, WINLINK_ALERTFLAGS, WINLINK_BELL, WINLINK_SILENCE,
};

pub type off_t = __off_t;

pub type uintmax_t = ::libc::uintmax_t;

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;

pub const IUTF8: ::core::ffi::c_int = 0o40000 as ::core::ffi::c_int;

pub const _PATH_DEFPATH: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"/usr/bin:/bin\0") };

pub const _PATH_TMP: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"/tmp/\0") };

unsafe extern "C" fn spawn_log(mut from: *const ::core::ffi::c_char, mut sc: *mut spawn_context) {
    let mut s: *mut session = (*sc).s;
    let mut wl: *mut winlink = (*sc).wl;
    let mut wp0: *mut window_pane = (*sc).wp0;
    let mut name: *const ::core::ffi::c_char = if (*sc).name.is_null() {
        b"none\0" as *const u8 as *const ::core::ffi::c_char
    } else {
        (*sc).name
    };
    let mut tmp: [::core::ffi::c_char; 128] = [0; 128];
    log_debug(
        b"%s: name=%s, flags=%#x\0" as *const u8 as *const ::core::ffi::c_char,
        from,
        name,
        (*sc).flags,
    );
    if !wl.is_null() && !wp0.is_null() {
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
            b"wl=%d wp0=%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wl).idx,
            (*wp0).id,
        );
    } else if !wl.is_null() {
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
            b"wl=%d wp0=none\0" as *const u8 as *const ::core::ffi::c_char,
            (*wl).idx,
        );
    } else if !wp0.is_null() {
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
            b"wl=none wp0=%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp0).id,
        );
    } else {
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
            b"wl=none wp0=none\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    log_debug(
        b"%s: s=$%u %s idx=%d\0" as *const u8 as *const ::core::ffi::c_char,
        from,
        (*s).id,
        &raw mut tmp as *mut ::core::ffi::c_char,
        (*sc).idx,
    );
}
unsafe extern "C" fn spawn_fire_pane_created(mut sc: *mut spawn_context, mut wp: *mut window_pane) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cwd: *const ::core::ffi::c_char = (*wp).cwd;
    ep = event_payload_create();
    cmd_find_from_winlink_pane(&raw mut fs, (*sc).wl, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_session(
        ep,
        b"session\0" as *const u8 as *const ::core::ffi::c_char,
        (*sc).s,
    );
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    event_payload_set_int(
        ep,
        b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
        (*(*sc).wl).idx,
    );
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    if (*wp).argc != 0 as ::core::ffi::c_int {
        cmd = cmd_stringify_argv((*wp).argc, (*wp).argv);
    }
    if !cmd.is_null() && *cmd as ::core::ffi::c_int != '\0' as i32 {
        event_payload_set_string(
            ep,
            b"pane_command\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            cmd,
        );
    } else if !(*wp).shell.is_null() {
        event_payload_set_string(
            ep,
            b"pane_command\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).shell,
        );
    }
    free(cmd as *mut ::core::ffi::c_void);
    if !cwd.is_null() {
        event_payload_set_string(
            ep,
            b"pane_current_path\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            cwd,
        );
    }
    if (*sc).flags & SPAWN_EMPTY != 0 {
        event_payload_set_int(
            ep,
            b"created_empty\0" as *const u8 as *const ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
    } else {
        event_payload_set_int(
            ep,
            b"created_empty\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    }
    if (*sc).flags & SPAWN_RESPAWN != 0 {
        event_payload_set_int(
            ep,
            b"created_respawn\0" as *const u8 as *const ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
    } else {
        event_payload_set_int(
            ep,
            b"created_respawn\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    }
    events_fire(
        b"pane-created\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
#[no_mangle]
pub unsafe extern "C" fn spawn_window(
    mut sc: *mut spawn_context,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut winlink {
    let mut s: *mut session = (*sc).s;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut idx: ::core::ffi::c_int = (*sc).idx;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xpixel: u_int = 0;
    let mut ypixel: u_int = 0;
    spawn_log(
        b"spawn_window\0" as *const u8 as *const ::core::ffi::c_char,
        sc,
    );
    if (*sc).flags & SPAWN_RESPAWN != 0 {
        w = (*(*sc).wl).window;
        if !(*sc).flags & SPAWN_KILL != 0 {
            wp = (*w).panes.tqh_first;
            while !wp.is_null() {
                if (*wp).fd != -(1 as ::core::ffi::c_int) {
                    break;
                }
                wp = (*wp).entry.tqe_next;
            }
            if !wp.is_null() {
                xasprintf(
                    cause,
                    b"window %s:%d still active\0" as *const u8 as *const ::core::ffi::c_char,
                    (*s).name,
                    (*(*sc).wl).idx,
                );
                return ::core::ptr::null_mut::<winlink>();
            }
        }
        (*sc).wp0 = (*w).panes.tqh_first;
        if !(*(*sc).wp0).entry.tqe_next.is_null() {
            (*(*(*sc).wp0).entry.tqe_next).entry.tqe_prev = (*(*sc).wp0).entry.tqe_prev;
        } else {
            (*w).panes.tqh_last = (*(*sc).wp0).entry.tqe_prev;
        }
        *(*(*sc).wp0).entry.tqe_prev = (*(*sc).wp0).entry.tqe_next;
        layout_free(w, 0 as ::core::ffi::c_int);
        window_destroy_panes(w);
        (*(*sc).wp0).entry.tqe_next = (*w).panes.tqh_first;
        if !(*(*sc).wp0).entry.tqe_next.is_null() {
            (*(*w).panes.tqh_first).entry.tqe_prev = &raw mut (*(*sc).wp0).entry.tqe_next;
        } else {
            (*w).panes.tqh_last = &raw mut (*(*sc).wp0).entry.tqe_next;
        }
        (*w).panes.tqh_first = (*sc).wp0;
        (*(*sc).wp0).entry.tqe_prev = &raw mut (*w).panes.tqh_first;
        window_pane_resize((*sc).wp0, (*w).sx, (*w).sy);
        layout_init(w, (*sc).wp0);
        (*w).active = ::core::ptr::null_mut::<window_pane>();
        window_set_active_pane(w, (*sc).wp0, 0 as ::core::ffi::c_int);
    }
    if !(*sc).flags & SPAWN_RESPAWN != 0 && idx != -(1 as ::core::ffi::c_int) {
        wl = winlink_find_by_index(&raw mut (*s).windows, idx);
        if !wl.is_null() && !(*sc).flags & SPAWN_KILL != 0 {
            xasprintf(
                cause,
                b"index %d in use\0" as *const u8 as *const ::core::ffi::c_char,
                idx,
            );
            return ::core::ptr::null_mut::<winlink>();
        }
        if !wl.is_null() {
            (*wl).flags &= !WINLINK_ALERTFLAGS;
            events_fire_winlink(
                b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
                wl,
            );
            winlink_stack_remove(&raw mut (*s).lastw, wl);
            winlink_remove(&raw mut (*s).windows, wl);
            if (*s).curw == wl {
                (*s).curw = ::core::ptr::null_mut::<winlink>();
                (*sc).flags &= !SPAWN_DETACHED;
            }
        }
    }
    if !(*sc).flags & SPAWN_RESPAWN != 0 {
        if idx == -(1 as ::core::ffi::c_int) {
            idx = (-(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong
                - options_get_number(
                    (*s).options,
                    b"base-index\0" as *const u8 as *const ::core::ffi::c_char,
                )) as ::core::ffi::c_int;
        }
        (*sc).wl = winlink_add(&raw mut (*s).windows, idx);
        if (*sc).wl.is_null() {
            xasprintf(
                cause,
                b"couldn't add window %d\0" as *const u8 as *const ::core::ffi::c_char,
                idx,
            );
            return ::core::ptr::null_mut::<winlink>();
        }
        default_window_size(
            (*sc).tc,
            s,
            ::core::ptr::null_mut::<window>(),
            &raw mut sx,
            &raw mut sy,
            &raw mut xpixel,
            &raw mut ypixel,
            -(1 as ::core::ffi::c_int),
        );
        w = window_create(sx, sy, xpixel, ypixel);
        if w.is_null() {
            winlink_remove(&raw mut (*s).windows, (*sc).wl);
            xasprintf(
                cause,
                b"couldn't create window %d\0" as *const u8 as *const ::core::ffi::c_char,
                idx,
            );
            return ::core::ptr::null_mut::<winlink>();
        }
        if (*s).curw.is_null() {
            (*s).curw = (*sc).wl;
        }
        (*(*sc).wl).session = s;
        (*w).latest = (*sc).tc as *mut ::core::ffi::c_void;
        winlink_set_window((*sc).wl, w);
    } else {
        w = ::core::ptr::null_mut::<window>();
    }
    (*sc).flags |= SPAWN_NONOTIFY;
    wp = spawn_pane(sc, cause);
    if wp.is_null() {
        if !(*sc).flags & SPAWN_RESPAWN != 0 {
            winlink_remove(&raw mut (*s).windows, (*sc).wl);
        }
        return ::core::ptr::null_mut::<winlink>();
    }
    if !(*sc).flags & SPAWN_RESPAWN != 0 {
        free((*w).name as *mut ::core::ffi::c_void);
        if (*sc).name.is_null() {
            (*w).name = default_window_name(w);
        } else {
            (*w).name = xstrdup((*sc).name);
            options_set_number(
                (*w).options,
                b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_longlong,
            );
        }
        window_set_fill_cells(w);
    }
    if !(*sc).flags & SPAWN_DETACHED != 0 {
        session_select(s, (*(*sc).wl).idx);
    }
    if !(*sc).flags & SPAWN_RESPAWN != 0 {
        events_fire_window(
            b"window-created\0" as *const u8 as *const ::core::ffi::c_char,
            w,
        );
        events_fire_winlink(
            b"window-linked\0" as *const u8 as *const ::core::ffi::c_char,
            (*sc).wl,
        );
    }
    session_group_synchronize_from(s);
    return (*sc).wl;
}
#[no_mangle]
pub unsafe extern "C" fn spawn_pane(
    mut sc: *mut spawn_context,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut window_pane {
    let mut item: *mut cmdq_item = (*sc).item;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = (*sc).s;
    let mut ts: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window = (*(*sc).wl).window;
    let mut new_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut child: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut ee: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut argv: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut argvp: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut argv0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cwd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut new_cwd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut path: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tmp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut home: *const ::core::ffi::c_char = find_home();
    let mut actual_cwd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut argc: ::core::ffi::c_int = 0;
    let mut idx: u_int = 0;
    let mut now: termios = termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_line: 0,
        c_cc: [0; 32],
        c2rust_unnamed: termios_input_speed { __ispeed: 0 },
        c2rust_unnamed_0: termios_output_speed { __ospeed: 0 },
    };
    let mut hlimit: u_int = 0;
    let mut ws: winsize = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let mut set: sigset_t = __sigset_t { __val: [0; 16] };
    let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
    let mut key: key_code = 0;
    if !item.is_null() {
        ts = (*cmdq_get_target(item)).s;
        c = cmdq_get_client(item);
    } else {
        ts = s;
        c = (*sc).tc;
    }
    spawn_log(
        b"spawn_pane\0" as *const u8 as *const ::core::ffi::c_char,
        sc,
    );
    if (*sc).flags & SPAWN_MODAL != 0 {
        if !(*sc).flags & SPAWN_FLOATING != 0 {
            xasprintf(
                cause,
                b"modal pane must be floating\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return ::core::ptr::null_mut::<window_pane>();
        }
        if !(*w).modal.is_null() {
            xasprintf(
                cause,
                b"window already has a modal pane\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return ::core::ptr::null_mut::<window_pane>();
        }
    }
    if !(*sc).cwd.is_null() {
        if !item.is_null() {
            cwd = format_single(
                item,
                (*sc).cwd,
                c,
                ts,
                ::core::ptr::null_mut::<winlink>(),
                ::core::ptr::null_mut::<window_pane>(),
            );
        } else {
            cwd = xstrdup((*sc).cwd);
        }
        if *cwd as ::core::ffi::c_int != '/' as i32 {
            xasprintf(
                &raw mut new_cwd,
                b"%s%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                server_client_get_cwd(c, ts),
                if *cwd as ::core::ffi::c_int != '\0' as i32 {
                    b"/\0" as *const u8 as *const ::core::ffi::c_char
                } else {
                    b"\0" as *const u8 as *const ::core::ffi::c_char
                },
                cwd,
            );
            free(cwd as *mut ::core::ffi::c_void);
            cwd = new_cwd;
        }
    } else if !(*sc).flags & SPAWN_RESPAWN != 0 {
        cwd = xstrdup(server_client_get_cwd(c, ts));
    } else {
        cwd = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    hlimit = options_get_number(
        (*s).options,
        b"history-limit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    if (*sc).flags & SPAWN_RESPAWN != 0 {
        if (*(*sc).wp0).fd != -(1 as ::core::ffi::c_int) && !(*sc).flags & SPAWN_KILL != 0 {
            window_pane_index((*sc).wp0, &raw mut idx);
            xasprintf(
                cause,
                b"pane %s:%d.%u still active\0" as *const u8 as *const ::core::ffi::c_char,
                (*s).name,
                (*(*sc).wl).idx,
                idx,
            );
            free(cwd as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<window_pane>();
        }
        if !(*(*sc).wp0).event.is_null() {
            bufferevent_free((*(*sc).wp0).event);
            (*(*sc).wp0).event = ::core::ptr::null_mut::<bufferevent>();
        }
        if (*(*sc).wp0).fd != -(1 as ::core::ffi::c_int) {
            close((*(*sc).wp0).fd);
            (*(*sc).wp0).fd = -(1 as ::core::ffi::c_int);
        }
        window_pane_reset_mode_all((*sc).wp0);
        screen_reinit(&raw mut (*(*sc).wp0).base, 0 as ::core::ffi::c_int);
        if !(*(*sc).wp0).ictx.is_null() {
            input_free((*(*sc).wp0).ictx);
            (*(*sc).wp0).ictx = ::core::ptr::null_mut::<input_ctx>();
        }
        (*(*sc).wp0).offset.used = 0 as size_t;
        (*(*sc).wp0).base_offset = 0 as size_t;
        (*(*sc).wp0).pipe_offset.used = 0 as size_t;
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            if (*loop_0).flags & CLIENT_CONTROL as uint64_t != 0 {
                control_reset_pane(loop_0, (*sc).wp0);
            }
            loop_0 = (*loop_0).entry.tqe_next;
        }
        new_wp = (*sc).wp0;
        (*new_wp).flags &= !(PANE_STATUSREADY | PANE_STATUSDRAWN);
    } else {
        if (*sc).lc.is_null() {
            new_wp = window_add_pane(
                w,
                ::core::ptr::null_mut::<window_pane>(),
                hlimit,
                (*sc).flags,
            );
            layout_init(w, new_wp);
        } else {
            new_wp = window_add_pane(w, (*sc).wp0, hlimit, (*sc).flags);
            if (*sc).flags & SPAWN_ZOOM != 0 {
                layout_assign_pane((*sc).lc, new_wp, 1 as ::core::ffi::c_int);
            } else {
                layout_assign_pane((*sc).lc, new_wp, 0 as ::core::ffi::c_int);
            }
        }
        if (*sc).flags & SPAWN_FLOATING != 0 {
            (*(*new_wp).layout_cell).flags |= LAYOUT_CELL_FLOATING;
        }
        if (*sc).flags & SPAWN_FLOATOVERZOOM != 0 {
            (*new_wp).flags |= PANE_FLOATOVERZOOM;
        }
        if (*w).flags & WINDOW_ZOOMED != 0 {
            (*new_wp).saved_layout_cell = (*new_wp).layout_cell as *mut layout_cell;
        }
    }
    if (*sc).argc == 0 as ::core::ffi::c_int && !(*sc).flags & SPAWN_RESPAWN != 0 {
        cmd = options_get_string(
            (*s).options,
            b"default-command\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if !cmd.is_null() && *cmd as ::core::ffi::c_int != '\0' as i32 {
            argc = 1 as ::core::ffi::c_int;
            argv = &raw mut cmd as *mut *mut ::core::ffi::c_char;
        } else {
            argc = 0 as ::core::ffi::c_int;
            argv = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        }
    } else {
        argc = (*sc).argc;
        argv = (*sc).argv;
    }
    if !cwd.is_null() {
        free((*new_wp).cwd as *mut ::core::ffi::c_void);
        (*new_wp).cwd = cwd;
    }
    if argc > 0 as ::core::ffi::c_int {
        cmd_free_argv((*new_wp).argc, (*new_wp).argv);
        (*new_wp).argc = argc;
        (*new_wp).argv = cmd_copy_argv(argc, argv);
    }
    // `child_owner` owns the C tree for the whole synchronous spawn
    // operation. Its raw pointer is borrowed by the translated C calls below;
    // it is not stored in `spawn_context` or any other calloc-managed record.
    let mut child_owner = Some(EnvironOwner::from_raw(environ_for_session(
        s,
        0 as ::core::ffi::c_int,
    )));
    child = child_owner
        .as_ref()
        .expect("spawn environment owner must exist")
        .as_ptr();
    if !(*sc).environ.is_null() {
        environ_copy((*sc).environ, child);
    }
    environ_set(
        child,
        b"TMUX_PANE\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
        b"%%%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*new_wp).id,
    );
    if !c.is_null() && (*c).session.is_null() {
        ee = environ_find(
            (*c).environ,
            b"PATH\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if !ee.is_null() {
            environ_set(
                child,
                b"PATH\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*ee).value,
            );
        }
    }
    if child_owner
        .as_ref()
        .expect("spawn environment owner must exist")
        .find(c"PATH")
        .is_none()
    {
        environ_set(
            child,
            b"PATH\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            _PATH_DEFPATH.as_ptr(),
        );
    }
    if !(*sc).flags & SPAWN_RESPAWN != 0 {
        tmp = options_get_string(
            (*s).options,
            b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if checkshell(tmp) == 0 {
            tmp = _PATH_BSHELL.as_ptr();
        }
        free((*new_wp).shell as *mut ::core::ffi::c_void);
        (*new_wp).shell = xstrdup(tmp);
    }
    environ_set(
        child,
        b"SHELL\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        (*new_wp).shell,
    );
    log_debug(
        b"%s: shell=%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"spawn_pane\0" as *const u8 as *const ::core::ffi::c_char,
        (*new_wp).shell,
    );
    if (*new_wp).argc != 0 as ::core::ffi::c_int {
        cp = cmd_stringify_argv((*new_wp).argc, (*new_wp).argv);
        log_debug(
            b"%s: cmd=%s\0" as *const u8 as *const ::core::ffi::c_char,
            b"spawn_pane\0" as *const u8 as *const ::core::ffi::c_char,
            cp,
        );
        free(cp as *mut ::core::ffi::c_void);
    }
    log_debug(
        b"%s: cwd=%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"spawn_pane\0" as *const u8 as *const ::core::ffi::c_char,
        (*new_wp).cwd,
    );
    cmd_log_argv(
        (*new_wp).argc,
        (*new_wp).argv,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"spawn_pane\0" as *const u8 as *const ::core::ffi::c_char,
    );
    environ_log(
        child,
        b"%s: environment \0" as *const u8 as *const ::core::ffi::c_char,
        b"spawn_pane\0" as *const u8 as *const ::core::ffi::c_char,
    );
    memset(
        &raw mut ws as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<winsize>() as size_t,
    );
    ws.ws_col = (*(*new_wp).base.grid).sx as ::core::ffi::c_ushort;
    ws.ws_row = (*(*new_wp).base.grid).sy as ::core::ffi::c_ushort;
    ws.ws_xpixel = (*w).xpixel.wrapping_mul(ws.ws_col as u_int) as ::core::ffi::c_ushort;
    ws.ws_ypixel = (*w).ypixel.wrapping_mul(ws.ws_row as u_int) as ::core::ffi::c_ushort;
    sigfillset(&raw mut set);
    sigprocmask(SIG_BLOCK, &raw mut set, &raw mut oldset);
    if (*sc).flags & SPAWN_EMPTY != 0 {
        (*new_wp).flags |= PANE_EMPTY;
        (*new_wp).base.mode &= !MODE_CURSOR;
        (*new_wp).base.mode |= MODE_CRLF;
    } else {
        (*new_wp).flags &= !PANE_EMPTY;
        if !getcwd(
            &raw mut path as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>() as size_t,
        )
        .is_null()
        {
            if chdir((*new_wp).cwd) == 0 as ::core::ffi::c_int {
                actual_cwd = (*new_wp).cwd;
            } else if !home.is_null() && chdir(home) == 0 as ::core::ffi::c_int {
                actual_cwd = home;
            } else if chdir(b"/\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                actual_cwd = b"/\0" as *const u8 as *const ::core::ffi::c_char;
            }
        }
        (*new_wp).pid = fdforkpty(
            ptm_fd,
            &raw mut (*new_wp).fd,
            &raw mut (*new_wp).tty as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<termios>(),
            &raw mut ws,
        );
        if (*new_wp).pid == -(1 as ::core::ffi::c_int) {
            xasprintf(
                cause,
                b"fork failed: %s\0" as *const u8 as *const ::core::ffi::c_char,
                strerror(*__errno_location()),
            );
            (*new_wp).fd = -(1 as ::core::ffi::c_int);
            if !(*sc).flags & SPAWN_RESPAWN != 0 {
                server_client_remove_pane(new_wp);
                layout_close_pane(new_wp);
                window_remove_pane(w, new_wp);
            }
            sigprocmask(
                SIG_SETMASK,
                &raw mut oldset,
                ::core::ptr::null_mut::<sigset_t>(),
            );
            return ::core::ptr::null_mut::<window_pane>();
        }
        if (*new_wp).pid != 0 as ::core::ffi::c_int {
            if !actual_cwd.is_null()
                && chdir(&raw mut path as *mut ::core::ffi::c_char) != 0 as ::core::ffi::c_int
                && (home.is_null() || chdir(home) != 0 as ::core::ffi::c_int)
            {
                chdir(b"/\0" as *const u8 as *const ::core::ffi::c_char);
            }
        } else {
            if systemd_move_to_new_cgroup(cause) < 0 as ::core::ffi::c_int {
                log_debug(
                    b"%s: moving pane to new cgroup failed: %s\0" as *const u8
                        as *const ::core::ffi::c_char,
                    b"spawn_pane\0" as *const u8 as *const ::core::ffi::c_char,
                    *cause,
                );
                free(*cause as *mut ::core::ffi::c_void);
            }
            if !actual_cwd.is_null() {
                environ_set(
                    child,
                    b"PWD\0" as *const u8 as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    actual_cwd,
                );
            }
            if tcgetattr(STDIN_FILENO, &raw mut now) != 0 as ::core::ffi::c_int {
                _exit(1 as ::core::ffi::c_int);
            }
            if !(*s).tio.is_null() {
                memcpy(
                    &raw mut now.c_cc as *mut cc_t as *mut ::core::ffi::c_void,
                    &raw mut (*(*s).tio).c_cc as *mut cc_t as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<[cc_t; 32]>() as size_t,
                );
            }
            key = options_get_number(
                global_options,
                b"backspace\0" as *const u8 as *const ::core::ffi::c_char,
            ) as key_code;
            if key >= 0x7f as key_code {
                now.c_cc[VERASE as usize] = '\u{7f}' as i32 as cc_t;
            } else {
                now.c_cc[VERASE as usize] = key as cc_t;
            }
            now.c_iflag |= IUTF8 as tcflag_t;
            if tcsetattr(STDIN_FILENO, TCSANOW, &raw mut now) != 0 as ::core::ffi::c_int {
                _exit(1 as ::core::ffi::c_int);
            }
            proc_clear_signals(server_proc, 1 as ::core::ffi::c_int);
            closefrom(STDERR_FILENO + 1 as ::core::ffi::c_int);
            sigprocmask(
                SIG_SETMASK,
                &raw mut oldset,
                ::core::ptr::null_mut::<sigset_t>(),
            );
            log_close();
            environ_push(child);
            // After fork this is the child's private copy. Release it after
            // publishing the process environment; the parent retains its own
            // owner and drops it when this function returns.
            drop(child_owner.take());
            if (*new_wp).argc != 0 as ::core::ffi::c_int
                && (*new_wp).argc != 1 as ::core::ffi::c_int
            {
                argvp = cmd_copy_argv((*new_wp).argc, (*new_wp).argv);
                execvp(
                    *argvp.offset(0 as ::core::ffi::c_int as isize),
                    argvp as *const *mut ::core::ffi::c_char,
                );
                _exit(1 as ::core::ffi::c_int);
            }
            cp = strrchr((*new_wp).shell, '/' as i32);
            if (*new_wp).argc == 1 as ::core::ffi::c_int {
                tmp = *(*new_wp).argv.offset(0 as ::core::ffi::c_int as isize);
                if !cp.is_null()
                    && *cp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != '\0' as i32
                {
                    xasprintf(
                        &raw mut argv0,
                        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                        cp.offset(1 as ::core::ffi::c_int as isize),
                    );
                } else {
                    xasprintf(
                        &raw mut argv0,
                        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                        (*new_wp).shell,
                    );
                }
                execl(
                    (*new_wp).shell,
                    argv0,
                    b"-c\0" as *const u8 as *const ::core::ffi::c_char,
                    tmp,
                    NULL as *mut ::core::ffi::c_char,
                );
                _exit(1 as ::core::ffi::c_int);
            }
            if !cp.is_null()
                && *cp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
            {
                xasprintf(
                    &raw mut argv0,
                    b"-%s\0" as *const u8 as *const ::core::ffi::c_char,
                    cp.offset(1 as ::core::ffi::c_int as isize),
                );
            } else {
                xasprintf(
                    &raw mut argv0,
                    b"-%s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*new_wp).shell,
                );
            }
            execl((*new_wp).shell, argv0, NULL as *mut ::core::ffi::c_char);
            _exit(1 as ::core::ffi::c_int);
        }
    }
    if !(*new_wp).flags & PANE_EMPTY != 0 {
        xasprintf(
            &raw mut cp,
            b"tmux(%lu).%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            getpid() as ::core::ffi::c_long,
            (*new_wp).id,
        );
        utempter_add_record((*new_wp).fd, cp);
        kill(getpid(), SIGCHLD);
        free(cp as *mut ::core::ffi::c_void);
    }
    (*new_wp).flags &= !PANE_EXITED;
    sigprocmask(
        SIG_SETMASK,
        &raw mut oldset,
        ::core::ptr::null_mut::<sigset_t>(),
    );
    window_pane_set_event(new_wp);
    drop(child_owner.take());
    spawn_fire_pane_created(sc, new_wp);
    if (*sc).flags & SPAWN_RESPAWN != 0 {
        return new_wp;
    }
    if (*sc).flags & SPAWN_MODAL != 0 {
        (*w).modal_last = (*w).active;
        (*w).modal = new_wp;
        window_redraw_active_switch(w, new_wp);
        if (*sc).flags & SPAWN_NONOTIFY != 0 {
            window_set_active_pane(w, new_wp, 0 as ::core::ffi::c_int);
        } else {
            window_set_active_pane(w, new_wp, 1 as ::core::ffi::c_int);
        }
    } else if (!(*sc).flags & SPAWN_DETACHED != 0 || (*w).active.is_null()) && (*w).modal.is_null()
    {
        if (*sc).flags & SPAWN_NONOTIFY != 0 {
            window_set_active_pane(w, new_wp, 0 as ::core::ffi::c_int);
        } else {
            window_set_active_pane(w, new_wp, 1 as ::core::ffi::c_int);
        }
    }
    if !(*sc).flags & SPAWN_NONOTIFY != 0 {
        events_fire_window(
            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
            w,
        );
    }
    return new_wp;
}
unsafe extern "C" fn spawn_editor_free(mut es: *mut spawn_editor_state) {
    unlink((*es).path);
    free((*es).path as *mut ::core::ffi::c_void);
    free(es as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn spawn_cancel_editor(mut es: *mut spawn_editor_state) {
    if es.is_null() {
        return;
    }
    (*es).cb = None;
    (*es).arg = NULL;
}
#[no_mangle]
pub unsafe extern "C" fn spawn_get_editor_pid(mut es: *mut spawn_editor_state) -> pid_t {
    if es.is_null() {
        return -(1 as pid_t);
    }
    return (*es).pid;
}
#[no_mangle]
pub unsafe extern "C" fn spawn_editor_finish(mut wp: *mut window_pane) {
    let mut es: *mut spawn_editor_state = (*wp).editor as *mut spawn_editor_state;
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: off_t = 0 as off_t;
    let mut status: ::core::ffi::c_int = 128 as ::core::ffi::c_int + SIGHUP;
    if es.is_null() {
        return;
    }
    (*wp).editor = ::core::ptr::null_mut::<spawn_editor_state>();
    if (*wp).flags & PANE_STATUSREADY != 0 {
        if (*wp).status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            status = ((*wp).status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
        } else if (((*wp).status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
            as ::core::ffi::c_schar as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int
            > 0 as ::core::ffi::c_int
        {
            status = ((*wp).status & 0x7f as ::core::ffi::c_int) + 128 as ::core::ffi::c_int;
        }
    }
    if (*es).cb.is_none() {
        spawn_editor_free(es);
        return;
    }
    if status != 0 as ::core::ffi::c_int {
        (*es).cb.expect("non-null function pointer")(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as size_t,
            (*es).arg,
        );
        spawn_editor_free(es);
        return;
    }
    f = fopen(
        (*es).path,
        b"r\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut FILE;
    if !f.is_null() {
        if fseeko(f, 0 as __off_t, SEEK_END) == 0 as ::core::ffi::c_int {
            len = ftello(f) as off_t;
            if len > 0 as off_t && len as uintmax_t <= SIZE_MAX as uintmax_t {
                if fseeko(f, 0 as __off_t, SEEK_SET) == 0 as ::core::ffi::c_int {
                    buf = malloc(len as size_t) as *mut ::core::ffi::c_char;
                    if !buf.is_null()
                        && fread(
                            buf as *mut ::core::ffi::c_void,
                            len as size_t,
                            1 as size_t,
                            f,
                        ) != 1 as ::core::ffi::c_ulong
                    {
                        free(buf as *mut ::core::ffi::c_void);
                        buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        len = 0 as off_t;
                    }
                }
            } else {
                len = 0 as off_t;
            }
        }
        fclose(f);
    }
    (*es).cb.expect("non-null function pointer")(buf, len as size_t, (*es).arg);
    spawn_editor_free(es);
}
#[no_mangle]
pub unsafe extern "C" fn spawn_editor(
    mut c: *mut client,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut cb: spawn_finish_edit_cb,
    mut arg: *mut ::core::ffi::c_void,
) -> *mut spawn_editor_state {
    let mut es: *mut spawn_editor_state = ::core::ptr::null_mut::<spawn_editor_state>();
    let mut sc: spawn_context = spawn_context {
        item: ::core::ptr::null_mut::<cmdq_item>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        tc: ::core::ptr::null_mut::<client>(),
        wp0: ::core::ptr::null_mut::<window_pane>(),
        lc: ::core::ptr::null_mut::<layout_cell>(),
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        argv: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        argc: 0,
        environ: ::core::ptr::null_mut::<environ>(),
        idx: 0,
        cwd: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: 0,
    };
    let mut s: *mut session = (*c).session;
    let mut wl: *mut winlink = (*s).curw;
    let mut w: *mut window = (*wl).window;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lg: layout_geometry = layout_geometry {
        sx: 0,
        sy: 0,
        xoff: 0,
        yoff: 0,
    };
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut path: [::core::ffi::c_char; 19] =
        ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"/tmp/tmux.XXXXXXXX\0");
    let mut editor: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut fd: ::core::ffi::c_int = 0;
    if !(*w).modal.is_null() {
        return ::core::ptr::null_mut::<spawn_editor_state>();
    }
    editor = options_get_string(
        global_options,
        b"editor\0" as *const u8 as *const ::core::ffi::c_char,
    );
    fd = mkstemp(&raw mut path as *mut ::core::ffi::c_char);
    if fd == -(1 as ::core::ffi::c_int) {
        return ::core::ptr::null_mut::<spawn_editor_state>();
    }
    f = fdopen(fd, b"w\0" as *const u8 as *const ::core::ffi::c_char);
    if f.is_null() {
        close(fd);
        unlink(&raw mut path as *mut ::core::ffi::c_char);
        return ::core::ptr::null_mut::<spawn_editor_state>();
    }
    if fwrite(buf as *const ::core::ffi::c_void, len, 1 as size_t, f) != 1 as ::core::ffi::c_ulong {
        fclose(f);
        unlink(&raw mut path as *mut ::core::ffi::c_char);
        return ::core::ptr::null_mut::<spawn_editor_state>();
    }
    fclose(f);
    es = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<spawn_editor_state>() as size_t,
    ) as *mut spawn_editor_state;
    (*es).path = xstrdup(&raw mut path as *mut ::core::ffi::c_char);
    (*es).cb = cb;
    (*es).arg = arg;
    lg.sx = (*w).sx.wrapping_mul(9 as u_int).wrapping_div(10 as u_int);
    lg.sy = (*w).sy.wrapping_mul(9 as u_int).wrapping_div(10 as u_int);
    lg.xoff = (*w)
        .sx
        .wrapping_div(2 as u_int)
        .wrapping_sub(lg.sx.wrapping_div(2 as u_int)) as ::core::ffi::c_int;
    lg.yoff = (*w)
        .sy
        .wrapping_div(2 as u_int)
        .wrapping_sub(lg.sy.wrapping_div(2 as u_int)) as ::core::ffi::c_int;
    window_push_zoom(w, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    lc = layout_floating_pane(w, ::core::ptr::null_mut::<window_pane>(), &raw mut lg);
    if lc.is_null() {
        window_pop_zoom(w);
        spawn_editor_free(es);
        return ::core::ptr::null_mut::<spawn_editor_state>();
    }
    xasprintf(
        &raw mut cmd,
        b"%s %s\0" as *const u8 as *const ::core::ffi::c_char,
        editor,
        &raw mut path as *mut ::core::ffi::c_char,
    );
    // `env_owner` remains outside the C `spawn_context` and releases this
    // temporary tree on every return path after the synchronous spawn call.
    let env_owner = EnvironOwner::new();
    env = env_owner.as_ptr();
    sc.s = s;
    sc.wl = wl;
    sc.tc = c;
    sc.wp0 = (*w).active;
    sc.lc = lc;
    sc.argc = 1 as ::core::ffi::c_int;
    sc.argv = &raw mut cmd;
    sc.environ = env;
    sc.idx = -(1 as ::core::ffi::c_int);
    sc.cwd = _PATH_TMP.as_ptr();
    sc.flags = SPAWN_FLOATING | SPAWN_MODAL | SPAWN_FLOATOVERZOOM;
    wp = spawn_pane(&raw mut sc, &raw mut cause);
    free(cmd as *mut ::core::ffi::c_void);
    if wp.is_null() {
        free(cause as *mut ::core::ffi::c_void);
        window_pop_zoom(w);
        spawn_editor_free(es);
        return ::core::ptr::null_mut::<spawn_editor_state>();
    }
    window_pop_zoom(w);
    options_set_number(
        (*wp).options,
        b"remain-on-exit\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_longlong,
    );
    (*es).pid = (*wp).pid;
    (*wp).editor = es as *mut spawn_editor_state;
    return es;
}

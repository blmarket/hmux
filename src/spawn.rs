use crate::cmd::{cmd_log_argv, cmd_stringify_argv};
use crate::cmd::{cmdq_get_client, cmdq_get_name, cmdq_get_target};
use crate::compat::fdforkpty;
use crate::compat::systemd_move_to_new_cgroup;
use crate::environ::{
    environ_copy, environ_entry_value, environ_find, environ_for_session, environ_log, environ_ptr,
    environ_push, environ_set,
};
use crate::ffi::{
    __errno_location, _exit, chdir, close, closefrom, execl, execvp, getcwd, getpid, kill,
    sigfillset, sigprocmask, strerror, strrchr, tcgetattr, tcsetattr, utempter_add_record,
};
use crate::fmt_args;
use crate::format::format_single;
use crate::input::input_free_box;
use crate::layout::{layout_assign_pane, layout_close_pane, layout_free, layout_init};
use crate::log::{log_close, log_debug};
use crate::names::default_window_name;
use crate::notify::{notify_session_window, notify_window};
use crate::options::{options_get_number, options_get_string, options_set_number};
use crate::proc::proc_clear_signals;
use crate::resize::default_window_size;
use crate::screen::{screen_grid_ptr, screen_reinit};
use crate::server::server_proc;
use crate::server::{server_client_get_cwd, server_client_remove_pane};
use crate::session::{
    session_get_curw, session_id, session_name, session_options, session_set_curw, session_tio,
};
use crate::session::{session_group_synchronize_from, session_select};
use crate::tmux::{checkshell, find_home};
use crate::tmux::{global_options, ptm_fd};
pub use crate::types::*;
use crate::window::window_set_latest;
use crate::window::{
    window_add_pane, window_create, window_destroy_panes, window_pane_index,
    window_pane_reset_mode_all, window_pane_resize, window_pane_set_event, window_panes_first,
    window_panes_insert_head, window_panes_next, window_panes_take, window_remove_pane,
    window_set_active_pane, winlink_add, winlink_find_by_index, winlink_remove,
    winlink_set_window_ref, winlink_stack_remove,
};
use crate::window::{window_get_active, window_set_active};
use crate::xmalloc::xasprintf;
use ::core::ffi::CStr;
use ::std::ffi::CString;
pub const MSG_READ_CANCEL: msgtype = 307;
pub const MSG_WRITE_CLOSE: msgtype = 306;
pub const MSG_WRITE_READY: msgtype = 305;
pub const MSG_WRITE: msgtype = 304;
pub const MSG_WRITE_OPEN: msgtype = 303;
pub const MSG_READ_DONE: msgtype = 302;
pub const MSG_READ: msgtype = 301;
pub const MSG_READ_OPEN: msgtype = 300;
pub const MSG_FLAGS: msgtype = 218;
pub const MSG_EXEC: msgtype = 217;
pub const MSG_WAKEUP: msgtype = 216;
pub const MSG_UNLOCK: msgtype = 215;
pub const MSG_SUSPEND: msgtype = 214;
pub const MSG_OLDSTDOUT: msgtype = 213;
pub const MSG_OLDSTDIN: msgtype = 212;
pub const MSG_OLDSTDERR: msgtype = 211;
pub const MSG_SHUTDOWN: msgtype = 210;
pub const MSG_SHELL: msgtype = 209;
pub const MSG_RESIZE: msgtype = 208;
pub const MSG_READY: msgtype = 207;
pub const MSG_LOCK: msgtype = 206;
pub const MSG_EXITING: msgtype = 205;
pub const MSG_EXITED: msgtype = 204;
pub const MSG_EXIT: msgtype = 203;
pub const MSG_DETACHKILL: msgtype = 202;
pub const MSG_DETACH: msgtype = 201;
pub const MSG_COMMAND: msgtype = 200;
pub const MSG_IDENTIFY_TERMINFO: msgtype = 112;
pub const MSG_IDENTIFY_LONGFLAGS: msgtype = 111;
pub const MSG_IDENTIFY_STDOUT: msgtype = 110;
pub const MSG_IDENTIFY_FEATURES: msgtype = 109;
pub const MSG_IDENTIFY_CWD: msgtype = 108;
pub const MSG_IDENTIFY_CLIENTPID: msgtype = 107;
pub const MSG_IDENTIFY_DONE: msgtype = 106;
pub const MSG_IDENTIFY_ENVIRON: msgtype = 105;
pub const MSG_IDENTIFY_STDIN: msgtype = 104;
pub const MSG_IDENTIFY_OLDCWD: msgtype = 103;
pub const MSG_IDENTIFY_TTYNAME: msgtype = 102;
pub const MSG_IDENTIFY_TERM: msgtype = 101;
pub const MSG_IDENTIFY_FLAGS: msgtype = 100;
pub const MSG_VERSION: msgtype = 12;
pub const PANE_LINES_SPACES: pane_lines = 5;
pub const PANE_LINES_NUMBER: pane_lines = 4;
pub const PANE_LINES_SIMPLE: pane_lines = 3;
pub const PANE_LINES_HEAVY: pane_lines = 2;
pub const PANE_LINES_DOUBLE: pane_lines = 1;
pub const PANE_LINES_SINGLE: pane_lines = 0;
pub const PROGRESS_BAR_PAUSED: progress_bar_state = 4;
pub const PROGRESS_BAR_INDETERMINATE: progress_bar_state = 3;
pub const PROGRESS_BAR_ERROR: progress_bar_state = 2;
pub const PROGRESS_BAR_NORMAL: progress_bar_state = 1;
pub const PROGRESS_BAR_HIDDEN: progress_bar_state = 0;
pub const SCREEN_CURSOR_BAR: screen_cursor_style = 3;
pub const SCREEN_CURSOR_UNDERLINE: screen_cursor_style = 2;
pub const SCREEN_CURSOR_BLOCK: screen_cursor_style = 1;
pub const SCREEN_CURSOR_DEFAULT: screen_cursor_style = 0;
pub const STYLE_DEFAULT_SET: style_default_type = 3;
pub const STYLE_DEFAULT_POP: style_default_type = 2;
pub const STYLE_DEFAULT_PUSH: style_default_type = 1;
pub const STYLE_DEFAULT_BASE: style_default_type = 0;
pub const STYLE_RANGE_CONTROL: style_range_type = 7;
pub const STYLE_RANGE_USER: style_range_type = 6;
pub const STYLE_RANGE_SESSION: style_range_type = 5;
pub const STYLE_RANGE_WINDOW: style_range_type = 4;
pub const STYLE_RANGE_PANE: style_range_type = 3;
pub const STYLE_RANGE_RIGHT: style_range_type = 2;
pub const STYLE_RANGE_LEFT: style_range_type = 1;
pub const STYLE_RANGE_NONE: style_range_type = 0;
pub const STYLE_LIST_RIGHT_MARKER: style_list = 4;
pub const STYLE_LIST_LEFT_MARKER: style_list = 3;
pub const STYLE_LIST_FOCUS: style_list = 2;
pub const STYLE_LIST_ON: style_list = 1;
pub const STYLE_LIST_OFF: style_list = 0;
pub const STYLE_ALIGN_ABSOLUTE_CENTRE: style_align = 4;
pub const STYLE_ALIGN_RIGHT: style_align = 3;
pub const STYLE_ALIGN_CENTRE: style_align = 2;
pub const STYLE_ALIGN_LEFT: style_align = 1;
pub const STYLE_ALIGN_DEFAULT: style_align = 0;
pub const THEME_DARK: client_theme = 2;
pub const THEME_LIGHT: client_theme = 1;
pub const THEME_UNKNOWN: client_theme = 0;
pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
pub const PROMPT_TYPE_INVALID: prompt_type = 255;
pub const PROMPT_TYPE_WINDOW_TARGET: prompt_type = 3;
pub const PROMPT_TYPE_TARGET: prompt_type = 2;
pub const PROMPT_TYPE_SEARCH: prompt_type = 1;
pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
pub const PROMPT_COMMAND: client_prompt_mode = 1;
pub const PROMPT_ENTRY: client_prompt_mode = 0;
pub const CLIENT_EXIT_DETACH: client_exit_type = 2;
pub const CLIENT_EXIT_SHUTDOWN: client_exit_type = 1;
pub const CLIENT_EXIT_RETURN: client_exit_type = 0;
pub const SIGCHLD: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const SIG_BLOCK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SIG_SETMASK: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const VERASE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const IUTF8: ::core::ffi::c_int = 0o40000 as ::core::ffi::c_int;
pub const TCSANOW: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const _PATH_DEFPATH: &CStr = c"/usr/bin:/bin";
pub const _PATH_BSHELL: &CStr = c"/bin/sh";
pub const MODE_CURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MODE_CRLF: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const PANE_EXITED: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const PANE_STATUSREADY: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const PANE_STATUSDRAWN: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const PANE_EMPTY: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const WINDOW_ZOOMED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const WINLINK_BELL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINLINK_ACTIVITY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const WINLINK_SILENCE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const WINLINK_ALERTFLAGS: ::core::ffi::c_int =
    WINLINK_BELL | WINLINK_ACTIVITY | WINLINK_SILENCE;
pub const LAYOUT_CELL_FLOATING: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const SPAWN_KILL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const SPAWN_DETACHED: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const SPAWN_RESPAWN: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const SPAWN_NONOTIFY: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const SPAWN_EMPTY: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const SPAWN_ZOOM: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const SPAWN_FLOATING: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
unsafe fn spawn_log(mut from: *const ::core::ffi::c_char, sc: &mut spawn_context) {
    unsafe {
        let mut s: *mut session = sc.s;
        let mut wl: *mut winlink = sc.wl;
        let mut wp0: *mut window_pane = sc.wp0;
        let item = spawn_item(sc);
        log_debug(
            c"%s: %s, flags=%#x".as_ptr(),
            fmt_args![from, cmdq_get_name(&*item), sc.flags],
        );
        let tmp = if !wl.is_null() && !wp0.is_null() {
            xasprintf(c"wl=%d wp0=%%%u".as_ptr(), fmt_args![(*wl).idx, (*wp0).id])
        } else if !wl.is_null() {
            xasprintf(c"wl=%d wp0=none".as_ptr(), fmt_args![(*wl).idx])
        } else if !wp0.is_null() {
            xasprintf(c"wl=none wp0=%%%u".as_ptr(), fmt_args![(*wp0).id])
        } else {
            xasprintf(c"wl=none wp0=none".as_ptr(), fmt_args![])
        };
        log_debug(
            c"%s: s=$%u %s idx=%d".as_ptr(),
            fmt_args![from, session_id(s), tmp.as_ptr(), sc.idx],
        );
        log_debug(
            c"%s: name=%s".as_ptr(),
            fmt_args![from, sc.name.map_or(c"none".as_ptr(), CStr::as_ptr)],
        );
    }
}
/// The queue item the spawn was asked from, which is waiting on it.
fn spawn_item(sc: &spawn_context) -> *mut cmdq_item {
    sc.item
        .as_ref()
        .and_then(|item| item.upgrade())
        .map_or(::core::ptr::null_mut::<cmdq_item>(), |item| item.as_ptr())
}

/// The client the spawn was asked for, or none.
fn spawn_client(sc: &spawn_context) -> *mut client {
    sc.tc
        .as_ref()
        .and_then(ClientWeak::upgrade)
        .map_or(::core::ptr::null_mut::<client>(), |c| c.as_ptr())
}

pub unsafe fn spawn_window(sc: &mut spawn_context, cause: &mut Option<CString>) -> *mut winlink {
    unsafe {
        let mut s: *mut session = sc.s;
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut w_ref: Option<WindowRef> = None;
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        let mut idx: ::core::ffi::c_int = sc.idx;
        let mut sx: u_int = 0;
        let mut sy: u_int = 0;
        let mut xpixel: u_int = 0;
        let mut ypixel: u_int = 0;
        spawn_log(c"spawn_window".as_ptr(), &mut *sc);
        if sc.flags & SPAWN_RESPAWN != 0 {
            w = (*sc.wl).window();
            w_ref = (*sc.wl).window_ref.clone();
            if !sc.flags & SPAWN_KILL != 0 {
                wp = window_panes_first(w);
                while !wp.is_null() {
                    if (*wp).fd != -(1 as ::core::ffi::c_int) {
                        break;
                    }
                    wp = window_panes_next(w, wp);
                }
                if !wp.is_null() {
                    *cause = Some(xasprintf(
                        c"window %s:%d still active".as_ptr(),
                        fmt_args![session_name(s), (*sc.wl).idx],
                    ));
                    return ::core::ptr::null_mut::<winlink>();
                }
            }
            sc.wp0 = window_panes_first(w);
            let kept = window_panes_take(w, sc.wp0).expect("the window holds its first pane");
            layout_free(w);
            window_destroy_panes(w);
            sc.wp0 = window_panes_insert_head(w, kept);
            window_pane_resize(sc.wp0, (*w).sx, (*w).sy);
            layout_init(w, sc.wp0);
            window_set_active(w, ::core::ptr::null_mut::<window_pane>());
            window_set_active_pane(w, sc.wp0, 0 as ::core::ffi::c_int);
        }
        if !sc.flags & SPAWN_RESPAWN != 0 && idx != -(1 as ::core::ffi::c_int) {
            wl = winlink_find_by_index(&raw mut (*s).windows, idx);
            if !wl.is_null() && !sc.flags & SPAWN_KILL != 0 {
                *cause = Some(xasprintf(c"index %d in use".as_ptr(), fmt_args![idx]));
                return ::core::ptr::null_mut::<winlink>();
            }
            if !wl.is_null() {
                (*wl).flags &= !WINLINK_ALERTFLAGS;
                notify_session_window(c"window-unlinked".as_ptr(), s, (*wl).window());
                winlink_stack_remove(&raw mut (*s).lastw, wl);
                winlink_remove(&raw mut (*s).windows, wl);
                if session_get_curw(s) == wl {
                    session_set_curw(s, ::core::ptr::null_mut::<winlink>());
                    sc.flags &= !SPAWN_DETACHED;
                }
            }
        }
        if !sc.flags & SPAWN_RESPAWN != 0 {
            if idx == -(1 as ::core::ffi::c_int) {
                idx = (-(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong
                    - options_get_number(session_options(s), c"base-index".as_ptr()))
                    as ::core::ffi::c_int;
            }
            sc.wl = winlink_add(&raw mut (*s).windows, idx);
            if sc.wl.is_null() {
                *cause = Some(xasprintf(
                    c"couldn't add window %d".as_ptr(),
                    fmt_args![idx],
                ));
                return ::core::ptr::null_mut::<winlink>();
            }
            (sx, sy, xpixel, ypixel) = default_window_size(
                spawn_client(sc),
                s,
                ::core::ptr::null_mut::<window>(),
                -(1 as ::core::ffi::c_int),
            );
            w_ref = Some(window_create(sx, sy, xpixel, ypixel));
            w = w_ref.as_ref().unwrap().as_ptr();
            if session_get_curw(s).is_null() {
                session_set_curw(s, sc.wl);
            }
            (*sc.wl).set_session(s);
            window_set_latest(w, spawn_client(sc));
            winlink_set_window_ref(sc.wl, w_ref.as_ref().unwrap().clone());
        } else {
            w = ::core::ptr::null_mut::<window>();
        }
        sc.flags |= SPAWN_NONOTIFY;
        wp = spawn_pane(&mut *sc, cause);
        if wp.is_null() {
            if !sc.flags & SPAWN_RESPAWN != 0 {
                if session_get_curw(s) == sc.wl {
                    session_set_curw(s, ::core::ptr::null_mut::<winlink>());
                }
                winlink_remove(&raw mut (*s).windows, sc.wl);
            }
            return ::core::ptr::null_mut::<winlink>();
        }
        if !sc.flags & SPAWN_RESPAWN != 0 {
            if sc.name.is_none_or(|name| name.is_empty()) {
                (*w).name = Some(default_window_name(w));
            } else {
                (*w).name = Some(sc.name.expect("the name just looked at").to_owned());
                options_set_number(
                    (*w).options_ptr(),
                    c"automatic-rename".as_ptr(),
                    0 as ::core::ffi::c_longlong,
                );
            }
        }
        if !sc.flags & SPAWN_DETACHED != 0 {
            session_select(s, (*sc.wl).idx);
        }
        if !sc.flags & SPAWN_RESPAWN != 0 {
            notify_session_window(c"window-linked".as_ptr(), s, w);
        }
        session_group_synchronize_from(s);
        sc.wl
    }
}
pub unsafe fn spawn_pane(sc: &mut spawn_context, cause: &mut Option<CString>) -> *mut window_pane {
    unsafe {
        let mut item: *mut cmdq_item = spawn_item(sc);
        let mut target: *mut cmd_find_state = cmdq_get_target(item);
        let mut c: *mut client = cmdq_get_client(&*item);
        let mut s: *mut session = sc.s;
        let mut w: *mut window = (*sc.wl).window();
        let mut new_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut cwd: Option<CString> = None;
        let mut path: [::core::ffi::c_char; 4096] = [0; 4096];
        let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut tmp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut home: *const ::core::ffi::c_char =
            find_home().map_or(::core::ptr::null(), CStr::as_ptr);
        let mut actual_cwd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut idx: u_int = 0;
        let mut now: termios = ::core::mem::zeroed();
        let mut hlimit: u_int = 0;
        let mut ws = winsize::default();
        let mut set: sigset_t = __sigset_t { __val: [0; 16] };
        let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
        let mut key: key_code = 0;
        spawn_log(c"spawn_pane".as_ptr(), &mut *sc);
        if let Some(sc_cwd) = sc.cwd {
            let expanded = format_single(
                item,
                sc_cwd,
                c,
                (*target).session(),
                ::core::ptr::null_mut::<winlink>(),
                ::core::ptr::null_mut::<window_pane>(),
            );
            let relative = expanded.as_ptr();
            if *relative as ::core::ffi::c_int != '/' as i32 {
                cwd = Some(xasprintf(
                    c"%s%s%s".as_ptr(),
                    fmt_args![
                        server_client_get_cwd(c, (*target).session()),
                        if *relative as ::core::ffi::c_int != '\0' as i32 {
                            c"/".as_ptr()
                        } else {
                            c"".as_ptr()
                        },
                        relative
                    ],
                ));
            } else {
                cwd = Some(expanded);
            }
        } else if !sc.flags & SPAWN_RESPAWN != 0 {
            cwd = Some(CStr::from_ptr(server_client_get_cwd(c, (*target).session())).to_owned());
        } else {
            cwd = None;
        }
        hlimit = options_get_number(session_options(s), c"history-limit".as_ptr()) as u_int;
        if sc.flags & SPAWN_RESPAWN != 0 {
            if (*sc.wp0).fd != -(1 as ::core::ffi::c_int) && !sc.flags & SPAWN_KILL != 0 {
                (_, idx) = window_pane_index(sc.wp0);
                *cause = Some(xasprintf(
                    c"pane %s:%d.%u still active".as_ptr(),
                    fmt_args![session_name(s), (*sc.wl).idx, idx],
                ));
                return ::core::ptr::null_mut::<window_pane>();
            }
            if (*sc.wp0).fd != -(1 as ::core::ffi::c_int) {
                (*sc.wp0).event.free();
                close((*sc.wp0).fd);
            }
            window_pane_reset_mode_all(sc.wp0);
            screen_reinit(&raw mut (*sc.wp0).base);
            if let Some(ictx) = (*sc.wp0).ictx.take() {
                input_free_box(ictx);
            }
            new_wp = sc.wp0;
            (*new_wp).flags &= !(PANE_STATUSREADY | PANE_STATUSDRAWN);
        } else {
            if sc.lc.is_null() {
                new_wp =
                    window_add_pane(w, ::core::ptr::null_mut::<window_pane>(), hlimit, sc.flags);
                layout_init(w, new_wp);
            } else {
                new_wp = window_add_pane(w, sc.wp0, hlimit, sc.flags);
                if sc.flags & SPAWN_ZOOM != 0 {
                    layout_assign_pane(sc.lc, new_wp, 1 as ::core::ffi::c_int);
                } else {
                    layout_assign_pane(sc.lc, new_wp, 0 as ::core::ffi::c_int);
                }
            }
            if sc.flags & SPAWN_FLOATING != 0 {
                (*(*new_wp).layout_cell).flags |= LAYOUT_CELL_FLOATING;
            }
            if (*w).flags & WINDOW_ZOOMED != 0 {
                (*new_wp).saved_layout_cell = (*new_wp).layout_cell;
            }
        }
        let argv = if sc.argv.is_empty() && sc.flags & SPAWN_RESPAWN == 0 {
            cmd = options_get_string(session_options(s), c"default-command".as_ptr());
            if !cmd.is_null() && *cmd as ::core::ffi::c_int != '\0' as i32 {
                vec![CStr::from_ptr(cmd).to_owned()]
            } else {
                Vec::new()
            }
        } else {
            ::core::mem::take(&mut sc.argv)
        };
        if cwd.is_some() {
            (*new_wp).cwd = cwd.take();
        }
        if !argv.is_empty() {
            (*new_wp).argv = argv;
        }
        let mut child = environ_for_session(s, 0 as ::core::ffi::c_int);
        if sc.environ.is_some() {
            environ_copy(environ_ptr(&sc.environ), &mut *child);
        }
        environ_set(
            &mut *child,
            c"TMUX_PANE".as_ptr(),
            0 as ::core::ffi::c_int,
            c"%%%u".as_ptr(),
            fmt_args![(*new_wp).id],
        );
        if !c.is_null() && (*c).session.is_null() {
            let path = environ_find(&*environ_ptr(&(*c).environ), c"PATH".as_ptr())
                .and_then(environ_entry_value);
            if let Some(path) = path {
                environ_set(
                    &mut *child,
                    c"PATH".as_ptr(),
                    0 as ::core::ffi::c_int,
                    c"%s".as_ptr(),
                    fmt_args![path],
                );
            }
        }
        if environ_find(&child, c"PATH".as_ptr()).is_none() {
            environ_set(
                &mut *child,
                c"PATH".as_ptr(),
                0 as ::core::ffi::c_int,
                c"%s".as_ptr(),
                fmt_args![_PATH_DEFPATH.as_ptr()],
            );
        }
        if !sc.flags & SPAWN_RESPAWN != 0 {
            tmp = options_get_string(session_options(s), c"default-shell".as_ptr());
            if checkshell(tmp) == 0 {
                tmp = _PATH_BSHELL.as_ptr();
            }
            (*new_wp).shell = Some(CStr::from_ptr(tmp).to_owned());
        }
        environ_set(
            &mut *child,
            c"SHELL".as_ptr(),
            0 as ::core::ffi::c_int,
            c"%s".as_ptr(),
            fmt_args![(*new_wp).shell.as_deref()],
        );
        log_debug(
            c"%s: shell=%s".as_ptr(),
            fmt_args![c"spawn_pane".as_ptr(), (*new_wp).shell.as_deref()],
        );
        if !(*new_wp).argv.is_empty() {
            let command = cmd_stringify_argv(&(*new_wp).argv);
            log_debug(
                c"%s: cmd=%s".as_ptr(),
                fmt_args![c"spawn_pane".as_ptr(), command.as_ptr()],
            );
        }
        log_debug(
            c"%s: cwd=%s".as_ptr(),
            fmt_args![c"spawn_pane".as_ptr(), (*new_wp).cwd.as_deref()],
        );
        cmd_log_argv(
            &(*new_wp).argv,
            c"%s".as_ptr(),
            fmt_args![c"spawn_pane".as_ptr()],
        );
        environ_log(
            &child,
            c"%s: environment ".as_ptr(),
            fmt_args![c"spawn_pane".as_ptr()],
        );
        ws = ::core::mem::zeroed();
        ws.ws_col = (*screen_grid_ptr(&raw mut (*new_wp).base)).sx as ::core::ffi::c_ushort;
        ws.ws_row = (*screen_grid_ptr(&raw mut (*new_wp).base)).sy as ::core::ffi::c_ushort;
        ws.ws_xpixel = (*w).xpixel.wrapping_mul(ws.ws_col as u_int) as ::core::ffi::c_ushort;
        ws.ws_ypixel = (*w).ypixel.wrapping_mul(ws.ws_row as u_int) as ::core::ffi::c_ushort;
        sigfillset(&raw mut set);
        sigprocmask(SIG_BLOCK, &raw mut set, &raw mut oldset);
        if sc.flags & SPAWN_EMPTY != 0 {
            (*new_wp).flags |= PANE_EMPTY;
            (*new_wp).base.mode &= !MODE_CURSOR;
            (*new_wp).base.mode |= MODE_CRLF;
        } else {
            if !getcwd(
                &raw mut path as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4096]>() as size_t,
            )
            .is_null()
            {
                if chdir(cstr_ptr(&(*new_wp).cwd)) == 0 as ::core::ffi::c_int {
                    actual_cwd = cstr_ptr(&(*new_wp).cwd);
                } else if !home.is_null() && chdir(home) == 0 as ::core::ffi::c_int {
                    actual_cwd = home;
                } else if chdir(c"/".as_ptr()) == 0 as ::core::ffi::c_int {
                    actual_cwd = c"/".as_ptr();
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
                *cause = Some(xasprintf(
                    c"fork failed: %s".as_ptr(),
                    fmt_args![strerror(*__errno_location())],
                ));
                (*new_wp).fd = -(1 as ::core::ffi::c_int);
                if !sc.flags & SPAWN_RESPAWN != 0 {
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
                    chdir(c"/".as_ptr());
                }
            } else {
                let mut cgroup_cause = None;
                if systemd_move_to_new_cgroup(&mut cgroup_cause) < 0 as ::core::ffi::c_int
                    && let Some(cgroup_cause) = cgroup_cause
                {
                    log_debug(
                        c"%s: moving pane to new cgroup failed: %s".as_ptr(),
                        fmt_args![c"spawn_pane".as_ptr(), cgroup_cause.as_ptr()],
                    );
                }
                if !actual_cwd.is_null() {
                    environ_set(
                        &mut *child,
                        c"PWD".as_ptr(),
                        0 as ::core::ffi::c_int,
                        c"%s".as_ptr(),
                        fmt_args![actual_cwd],
                    );
                }
                if tcgetattr(STDIN_FILENO, &raw mut now) != 0 as ::core::ffi::c_int {
                    _exit(1 as ::core::ffi::c_int);
                }
                if let Some(tio) = &session_tio(s) {
                    now.c_cc = tio.c_cc;
                }
                key = options_get_number(global_options, c"backspace".as_ptr()) as key_code;
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
                environ_push(&child);
                if (*new_wp).argv.len() > 1 {
                    let argvp: Vec<*mut ::core::ffi::c_char> = (*new_wp)
                        .argv
                        .iter()
                        .map(|arg| arg.as_ptr() as *mut ::core::ffi::c_char)
                        .chain(::core::iter::once(::core::ptr::null_mut()))
                        .collect();
                    execvp(argvp[0], argvp.as_ptr());
                    _exit(1 as ::core::ffi::c_int);
                }
                cp = strrchr(cstr_ptr(&(*new_wp).shell), '/' as i32);
                if (*new_wp).argv.len() == 1 {
                    tmp = (&(*new_wp).argv)[0].as_ptr();
                    let argv0 = if !cp.is_null()
                        && *cp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != '\0' as i32
                    {
                        xasprintf(
                            c"%s".as_ptr(),
                            fmt_args![cp.offset(1 as ::core::ffi::c_int as isize)],
                        )
                    } else {
                        xasprintf(c"%s".as_ptr(), fmt_args![(*new_wp).shell.as_deref()])
                    };
                    execl(
                        cstr_ptr(&(*new_wp).shell),
                        argv0.as_ptr() as *mut ::core::ffi::c_char,
                        c"-c".as_ptr(),
                        tmp,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    );
                    _exit(1 as ::core::ffi::c_int);
                }
                let argv0 = if !cp.is_null()
                    && *cp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != '\0' as i32
                {
                    xasprintf(
                        c"-%s".as_ptr(),
                        fmt_args![cp.offset(1 as ::core::ffi::c_int as isize)],
                    )
                } else {
                    xasprintf(c"-%s".as_ptr(), fmt_args![(*new_wp).shell.as_deref()])
                };
                execl(
                    cstr_ptr(&(*new_wp).shell),
                    argv0.as_ptr() as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                );
                _exit(1 as ::core::ffi::c_int);
            }
        }
        if !(*new_wp).flags & PANE_EMPTY != 0 {
            let cp = xasprintf(
                c"tmux(%lu).%%%u".as_ptr(),
                fmt_args![getpid() as ::core::ffi::c_long, (*new_wp).id],
            );
            utempter_add_record((*new_wp).fd, cp.as_ptr() as *mut ::core::ffi::c_char);
            kill(getpid(), SIGCHLD);
        }
        (*new_wp).flags &= !PANE_EXITED;
        sigprocmask(
            SIG_SETMASK,
            &raw mut oldset,
            ::core::ptr::null_mut::<sigset_t>(),
        );
        window_pane_set_event(new_wp);
        if sc.flags & SPAWN_RESPAWN != 0 {
            return new_wp;
        }
        if !sc.flags & SPAWN_DETACHED != 0 || window_get_active(w).is_null() {
            if sc.flags & SPAWN_NONOTIFY != 0 {
                window_set_active_pane(w, new_wp, 0 as ::core::ffi::c_int);
            } else {
                window_set_active_pane(w, new_wp, 1 as ::core::ffi::c_int);
            }
        }
        if !sc.flags & SPAWN_NONOTIFY != 0 {
            notify_window(c"window-layout-changed".as_ptr(), w);
        }
        new_wp
    }
}

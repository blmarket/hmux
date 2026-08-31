use crate::cmd::queue::{cmdq_error, cmdq_get_client, cmdq_get_current, cmdq_get_event};
use crate::cmd::{cmd_mouse_pane, cmd_mouse_window};
use crate::compat::strtonum;
use crate::environ::{environ_entry_value, environ_find, environ_ptr};
use crate::ffi::{fnmatch, strchr, strcmp, strlen, strncmp};
use crate::fmt_args;
use crate::log::{fatalx, log_debug};
use crate::server::client_walk;
use crate::server::marked_pane;
use crate::server::server_check_marked;
use crate::server::server_client_get_pane;
use crate::session::winlink_of;
use crate::session::{
    session_activity_time, session_attached, session_get_curw, session_id, session_name,
};
use crate::session::{
    session_alive, session_find, session_find_by_id_str, session_has, session_owners,
};
pub use crate::types::*;
use crate::window::PaneStack;
use crate::window::pane_walk;
use crate::window::window_get_active;
use crate::window::{
    window_find_by_id_str, window_find_string, window_has_pane, window_pane_at_index,
    window_pane_find_by_id_str, window_pane_find_down, window_pane_find_left,
    window_pane_find_right, window_pane_find_up, window_pane_next_by_number,
    window_pane_previous_by_number, window_pane_stack_first, winlink_find_by_index,
    winlink_next_by_number, winlink_previous_by_number, winlinks_after, winlinks_first,
    winlinks_last,
};
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
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const _PATH_DEV: &CStr = c"/dev/";
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const RB_INF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CMD_FIND_PREFER_UNATTACHED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_FIND_QUIET: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMD_FIND_WINDOW_INDEX: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_FIND_DEFAULT_MARKED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CMD_FIND_EXACT_SESSION: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CMD_FIND_EXACT_WINDOW: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CMD_FIND_CANFAIL: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
static cmd_find_session_table: ReadOnly<[[*const ::core::ffi::c_char; 2]; 1]> = ReadOnly::new([[
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
]]);
static cmd_find_window_table: ReadOnly<[[*const ::core::ffi::c_char; 2]; 6]> = ReadOnly::new([
    [c"{start}".as_ptr(), c"^".as_ptr()],
    [c"{last}".as_ptr(), c"!".as_ptr()],
    [c"{end}".as_ptr(), c"$".as_ptr()],
    [c"{next}".as_ptr(), c"+".as_ptr()],
    [c"{previous}".as_ptr(), c"-".as_ptr()],
    [
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
]);
static cmd_find_pane_table: ReadOnly<[[*const ::core::ffi::c_char; 2]; 16]> = ReadOnly::new([
    [c"{last}".as_ptr(), c"!".as_ptr()],
    [c"{next}".as_ptr(), c"+".as_ptr()],
    [c"{previous}".as_ptr(), c"-".as_ptr()],
    [c"{top}".as_ptr(), c"top".as_ptr()],
    [c"{bottom}".as_ptr(), c"bottom".as_ptr()],
    [c"{left}".as_ptr(), c"left".as_ptr()],
    [c"{right}".as_ptr(), c"right".as_ptr()],
    [c"{top-left}".as_ptr(), c"top-left".as_ptr()],
    [c"{top-right}".as_ptr(), c"top-right".as_ptr()],
    [c"{bottom-left}".as_ptr(), c"bottom-left".as_ptr()],
    [c"{bottom-right}".as_ptr(), c"bottom-right".as_ptr()],
    [c"{up-of}".as_ptr(), c"{up-of}".as_ptr()],
    [c"{down-of}".as_ptr(), c"{down-of}".as_ptr()],
    [c"{left-of}".as_ptr(), c"{left-of}".as_ptr()],
    [c"{right-of}".as_ptr(), c"{right-of}".as_ptr()],
    [
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
]);
unsafe fn cmd_find_inside_pane(mut c: *mut client) -> *mut window_pane {
    unsafe {
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        if c.is_null() {
            return ::core::ptr::null_mut::<window_pane>();
        }
        wp = pane_walk()
            .find(|&wp| {
                (*wp).fd != -(1 as ::core::ffi::c_int)
                    && strcmp(
                        &raw mut (*wp).tty as *mut ::core::ffi::c_char,
                        cstr_ptr(&(*c).ttyname),
                    ) == 0 as ::core::ffi::c_int
            })
            .unwrap_or(::core::ptr::null_mut::<window_pane>());
        if wp.is_null()
            && let Some(value) = environ_find(&*environ_ptr(&(*c).environ), c"TMUX_PANE".as_ptr())
                .and_then(environ_entry_value)
        {
            wp = window_pane_find_by_id_str(value.as_ptr());
        }
        if !wp.is_null() {
            log_debug(
                c"%s: got pane %%%u (%s)".as_ptr(),
                fmt_args![
                    c"cmd_find_inside_pane".as_ptr(),
                    (*wp).id,
                    &raw mut (*wp).tty as *mut ::core::ffi::c_char
                ],
            );
        }
        wp
    }
}
unsafe fn cmd_find_client_better(mut c: *mut client, mut than: *mut client) -> ::core::ffi::c_int {
    unsafe {
        if than.is_null() {
            return 1 as ::core::ffi::c_int;
        }
        if (*c).activity_time.tv_sec == (*than).activity_time.tv_sec {
            ((*c).activity_time.tv_usec > (*than).activity_time.tv_usec) as ::core::ffi::c_int
        } else {
            ((*c).activity_time.tv_sec > (*than).activity_time.tv_sec) as ::core::ffi::c_int
        }
    }
}
pub unsafe fn cmd_find_best_client(mut s: *mut session) -> *mut client {
    unsafe {
        let mut c: *mut client = ::core::ptr::null_mut::<client>();
        if session_attached(s) == 0 as u_int {
            s = ::core::ptr::null_mut::<session>();
        }
        c = ::core::ptr::null_mut::<client>();
        for c_loop in client_walk() {
            if !(*c_loop).session.is_null()
                && !(!s.is_null() && (*c_loop).session != s)
                && cmd_find_client_better(c_loop, c) != 0
            {
                c = c_loop;
            }
        }
        c
    }
}
unsafe fn cmd_find_session_better(
    mut s: *mut session,
    mut than: *mut session,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut attached: ::core::ffi::c_int = 0;
        if than.is_null() {
            return 1 as ::core::ffi::c_int;
        }
        if flags & CMD_FIND_PREFER_UNATTACHED != 0 {
            attached = (session_attached(than) != 0 as u_int) as ::core::ffi::c_int;
            if attached != 0 && session_attached(s) == 0 as u_int {
                return 1 as ::core::ffi::c_int;
            } else if attached == 0 && session_attached(s) != 0 as u_int {
                return 0 as ::core::ffi::c_int;
            }
        }
        if session_activity_time(s).tv_sec == session_activity_time(than).tv_sec {
            (session_activity_time(s).tv_usec > session_activity_time(than).tv_usec)
                as ::core::ffi::c_int
        } else {
            (session_activity_time(s).tv_sec > session_activity_time(than).tv_sec)
                as ::core::ffi::c_int
        }
    }
}
unsafe fn cmd_find_best_session(
    slist: &[*mut session],
    mut flags: ::core::ffi::c_int,
) -> *mut session {
    unsafe {
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        let mut i: u_int = 0;
        log_debug(
            c"%s: %u sessions to try".as_ptr(),
            fmt_args![c"cmd_find_best_session".as_ptr(), slist.len() as u_int],
        );
        s = ::core::ptr::null_mut::<session>();
        if !slist.is_empty() {
            i = 0 as u_int;
            while i < slist.len() as u_int {
                if cmd_find_session_better(slist[i as usize], s, flags) != 0 {
                    s = slist[i as usize];
                }
                i = i.wrapping_add(1);
            }
        } else {
            for s_loop in session_owners() {
                if cmd_find_session_better(s_loop.as_ptr(), s, flags) != 0 {
                    s = s_loop.as_ptr();
                }
            }
        }
        s
    }
}
unsafe fn cmd_find_best_session_with_window(fs: &mut cmd_find_state) -> ::core::ffi::c_int {
    unsafe {
        let mut slist: Vec<*mut session> = Vec::new();
        let w: *mut window = (*fs).window();
        if w.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        log_debug(
            c"%s: window is @%u".as_ptr(),
            fmt_args![c"cmd_find_best_session_with_window".as_ptr(), (*w).id],
        );
        for s_loop in session_owners() {
            if !(session_has(s_loop.as_ptr(), w) == 0) {
                slist.push(s_loop.as_ptr());
            }
        }
        if !slist.is_empty() {
            (*fs).set_session(cmd_find_best_session(&slist, fs.flags));
            if !(*fs).session().is_null() {
                return cmd_find_best_winlink_with_window(fs);
            }
        }
        -(1 as ::core::ffi::c_int)
    }
}
unsafe fn cmd_find_best_winlink_with_window(fs: &mut cmd_find_state) -> ::core::ffi::c_int {
    unsafe {
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        let mut wl_loop: *mut winlink = ::core::ptr::null_mut::<winlink>();
        if (*fs).window().is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        log_debug(
            c"%s: window is @%u".as_ptr(),
            fmt_args![
                c"cmd_find_best_winlink_with_window".as_ptr(),
                (*(*fs).window()).id
            ],
        );
        wl = ::core::ptr::null_mut::<winlink>();
        if !session_get_curw((*fs).session()).is_null()
            && (*session_get_curw((*fs).session())).window() == (*fs).window()
        {
            wl = session_get_curw((*fs).session());
        } else {
            wl_loop = winlinks_first(&raw mut (*(*fs).session()).windows);
            while !wl_loop.is_null() {
                if (*wl_loop).window() == (*fs).window() {
                    wl = wl_loop;
                    break;
                } else {
                    wl_loop = winlinks_after(wl_loop);
                }
            }
        }
        if wl.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        (*fs).set_winlink(wl);
        fs.idx = (*(*fs).winlink()).idx;
        0 as ::core::ffi::c_int
    }
}
unsafe fn cmd_find_map_table(
    mut table: *mut [*const ::core::ffi::c_char; 2],
    mut s: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut i: u_int = 0;
        i = 0 as u_int;
        while !(*table.offset(i as isize))[0 as ::core::ffi::c_int as usize].is_null() {
            if strcmp(
                s,
                (*table.offset(i as isize))[0 as ::core::ffi::c_int as usize],
            ) == 0 as ::core::ffi::c_int
            {
                return (*table.offset(i as isize))[1 as ::core::ffi::c_int as usize];
            }
            i = i.wrapping_add(1);
        }
        s
    }
}
unsafe fn cmd_find_get_session(
    fs: &mut cmd_find_state,
    mut session: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        let mut c: *mut client = ::core::ptr::null_mut::<client>();
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"cmd_find_get_session".as_ptr(), session],
        );
        if *session as ::core::ffi::c_int == '$' as i32 {
            (*fs).set_session(session_find_by_id_str(session));
            if (*fs).session().is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            return 0 as ::core::ffi::c_int;
        }
        (*fs).set_session(session_find(session));
        if !(*fs).session().is_null() {
            return 0 as ::core::ffi::c_int;
        }
        c = cmd_find_client(
            ::core::ptr::null_mut::<cmdq_item>(),
            session,
            1 as ::core::ffi::c_int,
        );
        if !c.is_null() && !(*c).session.is_null() {
            (*fs).set_session((*c).session);
            return 0 as ::core::ffi::c_int;
        }
        if fs.flags & CMD_FIND_EXACT_SESSION != 0 {
            return -(1 as ::core::ffi::c_int);
        }
        s = ::core::ptr::null_mut::<session>();
        for s_loop in session_owners() {
            if strncmp(session, session_name(s_loop.as_ptr()), strlen(session))
                == 0 as ::core::ffi::c_int
            {
                if !s.is_null() {
                    return -(1 as ::core::ffi::c_int);
                }
                s = s_loop.as_ptr();
            }
        }
        if !s.is_null() {
            (*fs).set_session(s);
            return 0 as ::core::ffi::c_int;
        }
        s = ::core::ptr::null_mut::<session>();
        for s_loop in session_owners() {
            if fnmatch(
                session,
                session_name(s_loop.as_ptr()),
                0 as ::core::ffi::c_int,
            ) == 0 as ::core::ffi::c_int
            {
                if !s.is_null() {
                    return -(1 as ::core::ffi::c_int);
                }
                s = s_loop.as_ptr();
            }
        }
        if !s.is_null() {
            (*fs).set_session(s);
            return 0 as ::core::ffi::c_int;
        }
        -(1 as ::core::ffi::c_int)
    }
}
unsafe fn cmd_find_get_window(
    current: &cmd_find_state,
    fs: &mut cmd_find_state,
    mut window: *const ::core::ffi::c_char,
    mut only: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"cmd_find_get_window".as_ptr(), window],
        );
        if *window as ::core::ffi::c_int == '@' as i32 {
            (*fs).set_window(window_find_by_id_str(window));
            if (*fs).window().is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            return cmd_find_best_session_with_window(fs);
        }
        (*fs).set_session((*current).session());
        if cmd_find_get_window_with_session(fs, window) == 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if only == 0 && cmd_find_get_session(fs, window) == 0 as ::core::ffi::c_int {
            (*fs).set_winlink(session_get_curw((*fs).session()));
            (*fs).set_window((*(*fs).winlink()).window());
            if !fs.flags & CMD_FIND_WINDOW_INDEX != 0 {
                fs.idx = (*(*fs).winlink()).idx;
            }
            return 0 as ::core::ffi::c_int;
        }
        -(1 as ::core::ffi::c_int)
    }
}
unsafe fn cmd_find_get_window_with_session(
    fs: &mut cmd_find_state,
    mut window: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        let mut idx: ::core::ffi::c_int = 0;
        let mut n: ::core::ffi::c_int = 0;
        let mut exact: ::core::ffi::c_int = 0;
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"cmd_find_get_window_with_session".as_ptr(), window],
        );
        exact = fs.flags & CMD_FIND_EXACT_WINDOW;
        (*fs).set_winlink(session_get_curw((*fs).session()));
        (*fs).set_window((*(*fs).winlink()).window());
        if *window as ::core::ffi::c_int == '@' as i32 {
            (*fs).set_window(window_find_by_id_str(window));
            if (*fs).window().is_null() || session_has((*fs).session(), (*fs).window()) == 0 {
                return -(1 as ::core::ffi::c_int);
            }
            return cmd_find_best_winlink_with_window(fs);
        }
        if exact == 0
            && (*window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '+' as i32
                || *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '-' as i32)
        {
            if *window.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
            {
                n = strtonum(
                    window.offset(1 as ::core::ffi::c_int as isize),
                    1 as ::core::ffi::c_longlong,
                    INT_MAX as ::core::ffi::c_longlong,
                )
                .unwrap_or(0) as ::core::ffi::c_int;
            } else {
                n = 1 as ::core::ffi::c_int;
            }
            s = (*fs).session();
            if fs.flags & CMD_FIND_WINDOW_INDEX != 0 {
                if *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '+' as i32
                {
                    if INT_MAX - (*session_get_curw(s)).idx < n {
                        return -(1 as ::core::ffi::c_int);
                    }
                    fs.idx = (*session_get_curw(s)).idx + n;
                } else {
                    if n > (*session_get_curw(s)).idx {
                        return -(1 as ::core::ffi::c_int);
                    }
                    fs.idx = (*session_get_curw(s)).idx - n;
                }
                return 0 as ::core::ffi::c_int;
            }
            if *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '+' as i32
            {
                (*fs).set_winlink(winlink_next_by_number(session_get_curw(s), s, n));
            } else {
                (*fs).set_winlink(winlink_previous_by_number(session_get_curw(s), s, n));
            }
            if !(*fs).winlink().is_null() {
                fs.idx = (*(*fs).winlink()).idx;
                (*fs).set_window((*(*fs).winlink()).window());
                return 0 as ::core::ffi::c_int;
            }
        }
        if exact == 0 {
            if strcmp(window, c"!".as_ptr()) == 0 as ::core::ffi::c_int {
                (*fs).set_winlink(winlink_of(
                    (*fs).session(),
                    (*(*fs).session()).lastw.first().copied(),
                ));
                if (*fs).winlink().is_null() {
                    return -(1 as ::core::ffi::c_int);
                }
                fs.idx = (*(*fs).winlink()).idx;
                (*fs).set_window((*(*fs).winlink()).window());
                return 0 as ::core::ffi::c_int;
            } else if strcmp(window, c"^".as_ptr()) == 0 as ::core::ffi::c_int {
                (*fs).set_winlink(winlinks_first(&raw mut (*(*fs).session()).windows));
                if (*fs).winlink().is_null() {
                    return -(1 as ::core::ffi::c_int);
                }
                fs.idx = (*(*fs).winlink()).idx;
                (*fs).set_window((*(*fs).winlink()).window());
                return 0 as ::core::ffi::c_int;
            } else if strcmp(window, c"$".as_ptr()) == 0 as ::core::ffi::c_int {
                (*fs).set_winlink(winlinks_last(&raw mut (*(*fs).session()).windows));
                if (*fs).winlink().is_null() {
                    return -(1 as ::core::ffi::c_int);
                }
                fs.idx = (*(*fs).winlink()).idx;
                (*fs).set_window((*(*fs).winlink()).window());
                return 0 as ::core::ffi::c_int;
            }
        }
        if *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '+' as i32
            && *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '-' as i32
        {
            let parsed = strtonum(
                window,
                0 as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
            );
            idx = parsed.unwrap_or(0) as ::core::ffi::c_int;
            if parsed.is_ok() {
                (*fs).set_winlink(winlink_find_by_index(
                    &raw mut (*(*fs).session()).windows,
                    idx,
                ));
                if !(*fs).winlink().is_null() {
                    fs.idx = (*(*fs).winlink()).idx;
                    (*fs).set_window((*(*fs).winlink()).window());
                    return 0 as ::core::ffi::c_int;
                }
                if fs.flags & CMD_FIND_WINDOW_INDEX != 0 {
                    fs.idx = idx;
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
        (*fs).set_winlink(::core::ptr::null_mut::<winlink>());
        wl = winlinks_first(&raw mut (*(*fs).session()).windows);
        while !wl.is_null() {
            if strcmp(window, cstr_ptr(&(*(*wl).window()).name)) == 0 as ::core::ffi::c_int {
                if !(*fs).winlink().is_null() {
                    return -(1 as ::core::ffi::c_int);
                }
                (*fs).set_winlink(wl);
            }
            wl = winlinks_after(wl);
        }
        if !(*fs).winlink().is_null() {
            fs.idx = (*(*fs).winlink()).idx;
            (*fs).set_window((*(*fs).winlink()).window());
            return 0 as ::core::ffi::c_int;
        }
        if exact != 0 {
            return -(1 as ::core::ffi::c_int);
        }
        (*fs).set_winlink(::core::ptr::null_mut::<winlink>());
        wl = winlinks_first(&raw mut (*(*fs).session()).windows);
        while !wl.is_null() {
            if strncmp(window, cstr_ptr(&(*(*wl).window()).name), strlen(window))
                == 0 as ::core::ffi::c_int
            {
                if !(*fs).winlink().is_null() {
                    return -(1 as ::core::ffi::c_int);
                }
                (*fs).set_winlink(wl);
            }
            wl = winlinks_after(wl);
        }
        if !(*fs).winlink().is_null() {
            fs.idx = (*(*fs).winlink()).idx;
            (*fs).set_window((*(*fs).winlink()).window());
            return 0 as ::core::ffi::c_int;
        }
        (*fs).set_winlink(::core::ptr::null_mut::<winlink>());
        wl = winlinks_first(&raw mut (*(*fs).session()).windows);
        while !wl.is_null() {
            if fnmatch(
                window,
                cstr_ptr(&(*(*wl).window()).name),
                0 as ::core::ffi::c_int,
            ) == 0 as ::core::ffi::c_int
            {
                if !(*fs).winlink().is_null() {
                    return -(1 as ::core::ffi::c_int);
                }
                (*fs).set_winlink(wl);
            }
            wl = winlinks_after(wl);
        }
        if !(*fs).winlink().is_null() {
            fs.idx = (*(*fs).winlink()).idx;
            (*fs).set_window((*(*fs).winlink()).window());
            return 0 as ::core::ffi::c_int;
        }
        -(1 as ::core::ffi::c_int)
    }
}
unsafe fn cmd_find_get_pane(
    current: &cmd_find_state,
    fs: &mut cmd_find_state,
    mut pane: *const ::core::ffi::c_char,
    mut only: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"cmd_find_get_pane".as_ptr(), pane],
        );
        if *pane as ::core::ffi::c_int == '%' as i32 {
            (*fs).set_pane(window_pane_find_by_id_str(pane));
            if (*fs).pane().is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).set_window((*(*fs).pane()).window);
            return cmd_find_best_session_with_window(fs);
        }
        (*fs).set_session((*current).session());
        (*fs).set_winlink((*current).winlink());
        fs.idx = current.idx;
        (*fs).set_window((*current).window());
        if cmd_find_get_pane_with_window(fs, pane) == 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if only == 0
            && cmd_find_get_window(current, fs, pane, 0 as ::core::ffi::c_int)
                == 0 as ::core::ffi::c_int
        {
            (*fs).set_pane(window_get_active((*fs).window()));
            return 0 as ::core::ffi::c_int;
        }
        -(1 as ::core::ffi::c_int)
    }
}
unsafe fn cmd_find_get_pane_with_session(
    fs: &mut cmd_find_state,
    mut pane: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"cmd_find_get_pane_with_session".as_ptr(), pane],
        );
        if *pane as ::core::ffi::c_int == '%' as i32 {
            (*fs).set_pane(window_pane_find_by_id_str(pane));
            if (*fs).pane().is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).set_window((*(*fs).pane()).window);
            return cmd_find_best_winlink_with_window(fs);
        }
        (*fs).set_winlink(session_get_curw((*fs).session()));
        fs.idx = (*(*fs).winlink()).idx;
        (*fs).set_window((*(*fs).winlink()).window());
        cmd_find_get_pane_with_window(fs, pane)
    }
}
unsafe fn cmd_find_get_pane_with_window(
    fs: &mut cmd_find_state,
    mut pane: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut idx: ::core::ffi::c_int = 0;
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut n: u_int = 0;
        log_debug(
            c"%s: %s".as_ptr(),
            fmt_args![c"cmd_find_get_pane_with_window".as_ptr(), pane],
        );
        if *pane as ::core::ffi::c_int == '%' as i32 {
            (*fs).set_pane(window_pane_find_by_id_str(pane));
            if (*fs).pane().is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            if (*(*fs).pane()).window != (*fs).window() {
                return -(1 as ::core::ffi::c_int);
            }
            return 0 as ::core::ffi::c_int;
        }
        if strcmp(pane, c"!".as_ptr()) == 0 as ::core::ffi::c_int {
            (*fs).set_pane(window_pane_stack_first((*fs).window(), PaneStack::LastUsed));
            if (*fs).pane().is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            return 0 as ::core::ffi::c_int;
        } else if strcmp(pane, c"{up-of}".as_ptr()) == 0 as ::core::ffi::c_int {
            (*fs).set_pane(window_pane_find_up(window_get_active((*fs).window())));
            if (*fs).pane().is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            return 0 as ::core::ffi::c_int;
        } else if strcmp(pane, c"{down-of}".as_ptr()) == 0 as ::core::ffi::c_int {
            (*fs).set_pane(window_pane_find_down(window_get_active((*fs).window())));
            if (*fs).pane().is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            return 0 as ::core::ffi::c_int;
        } else if strcmp(pane, c"{left-of}".as_ptr()) == 0 as ::core::ffi::c_int {
            (*fs).set_pane(window_pane_find_left(window_get_active((*fs).window())));
            if (*fs).pane().is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            return 0 as ::core::ffi::c_int;
        } else if strcmp(pane, c"{right-of}".as_ptr()) == 0 as ::core::ffi::c_int {
            (*fs).set_pane(window_pane_find_right(window_get_active((*fs).window())));
            if (*fs).pane().is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            return 0 as ::core::ffi::c_int;
        }
        if *pane.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '+' as i32
            || *pane.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '-' as i32
        {
            if *pane.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32 {
                n = strtonum(
                    pane.offset(1 as ::core::ffi::c_int as isize),
                    1 as ::core::ffi::c_longlong,
                    INT_MAX as ::core::ffi::c_longlong,
                )
                .unwrap_or(0) as u_int;
            } else {
                n = 1 as u_int;
            }
            wp = window_get_active((*fs).window());
            if *pane.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '+' as i32 {
                (*fs).set_pane(window_pane_next_by_number((*fs).window(), wp, n));
            } else {
                (*fs).set_pane(window_pane_previous_by_number((*fs).window(), wp, n));
            }
            if !(*fs).pane().is_null() {
                return 0 as ::core::ffi::c_int;
            }
        }
        let parsed = strtonum(
            pane,
            0 as ::core::ffi::c_longlong,
            INT_MAX as ::core::ffi::c_longlong,
        );
        idx = parsed.unwrap_or(0) as ::core::ffi::c_int;
        if parsed.is_ok() {
            (*fs).set_pane(window_pane_at_index((*fs).window(), idx as u_int));
            if !(*fs).pane().is_null() {
                return 0 as ::core::ffi::c_int;
            }
        }
        (*fs).set_pane(window_find_string((*fs).window(), pane));
        if !(*fs).pane().is_null() {
            return 0 as ::core::ffi::c_int;
        }
        -(1 as ::core::ffi::c_int)
    }
}
pub fn cmd_find_clear_state(fs: &mut cmd_find_state, mut flags: ::core::ffi::c_int) {
    *fs = cmd_find_state::default();
    fs.flags = flags;
    fs.idx = -(1 as ::core::ffi::c_int);
}
pub fn cmd_find_empty_state(fs: &cmd_find_state) -> ::core::ffi::c_int {
    if fs.session().is_null()
        && fs.winlink().is_null()
        && fs.window().is_null()
        && fs.pane().is_null()
    {
        return 1 as ::core::ffi::c_int;
    }
    0 as ::core::ffi::c_int
}
pub unsafe fn cmd_find_valid_state(fs: &cmd_find_state) -> ::core::ffi::c_int {
    unsafe {
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        if (*fs).session().is_null()
            || (*fs).winlink().is_null()
            || (*fs).window().is_null()
            || (*fs).pane().is_null()
        {
            return 0 as ::core::ffi::c_int;
        }
        if session_alive((*fs).session()) == 0 {
            return 0 as ::core::ffi::c_int;
        }
        wl = winlinks_first(&raw mut (*(*fs).session()).windows);
        while !wl.is_null() {
            if (*wl).window() == (*fs).window() && wl == (*fs).winlink() {
                break;
            }
            wl = winlinks_after(wl);
        }
        if wl.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        if (*fs).window() != (*(*fs).winlink()).window() {
            return 0 as ::core::ffi::c_int;
        }
        window_has_pane((*fs).window(), (*fs).pane())
    }
}
pub unsafe fn cmd_find_copy_state(dst: &mut cmd_find_state, src: &cmd_find_state) {
    unsafe {
        (*dst).set_session((*src).session());
        (*dst).set_winlink((*src).winlink());
        dst.idx = src.idx;
        (*dst).set_window((*src).window());
        (*dst).set_pane((*src).pane());
    }
}
unsafe fn cmd_find_log_state(mut prefix: *const ::core::ffi::c_char, fs: &mut cmd_find_state) {
    unsafe {
        if !(*fs).session().is_null() {
            log_debug(
                c"%s: s=$%u %s".as_ptr(),
                fmt_args![
                    prefix,
                    session_id((*fs).session()),
                    session_name((*fs).session())
                ],
            );
        } else {
            log_debug(c"%s: s=none".as_ptr(), fmt_args![prefix]);
        }
        if !(*fs).winlink().is_null() {
            log_debug(
                c"%s: wl=%u %d w=@%u %s".as_ptr(),
                fmt_args![
                    prefix,
                    (*(*fs).winlink()).idx,
                    ((*(*fs).winlink()).window() == (*fs).window()) as ::core::ffi::c_int,
                    (*(*fs).window()).id,
                    (*(*fs).window()).name.as_deref()
                ],
            );
        } else {
            log_debug(c"%s: wl=none".as_ptr(), fmt_args![prefix]);
        }
        if !(*fs).pane().is_null() {
            log_debug(
                c"%s: wp=%%%u".as_ptr(),
                fmt_args![prefix, (*(*fs).pane()).id],
            );
        } else {
            log_debug(c"%s: wp=none".as_ptr(), fmt_args![prefix]);
        }
        if fs.idx != -(1 as ::core::ffi::c_int) {
            log_debug(c"%s: idx=%d".as_ptr(), fmt_args![prefix, fs.idx]);
        } else {
            log_debug(c"%s: idx=none".as_ptr(), fmt_args![prefix]);
        };
    }
}
pub unsafe fn cmd_find_from_session(
    fs: &mut cmd_find_state,
    mut s: *mut session,
    mut flags: ::core::ffi::c_int,
) {
    unsafe {
        cmd_find_clear_state(fs, flags);
        (*fs).set_session(s);
        (*fs).set_winlink(session_get_curw((*fs).session()));
        (*fs).set_window((*(*fs).winlink()).window());
        (*fs).set_pane(window_get_active((*fs).window()));
        cmd_find_log_state(c"cmd_find_from_session".as_ptr(), fs);
    }
}
pub unsafe fn cmd_find_from_winlink(
    fs: &mut cmd_find_state,
    mut wl: *mut winlink,
    mut flags: ::core::ffi::c_int,
) {
    unsafe {
        cmd_find_clear_state(fs, flags);
        (*fs).set_session((*wl).session());
        (*fs).set_winlink(wl);
        (*fs).set_window((*wl).window());
        (*fs).set_pane(window_get_active((*wl).window()));
        cmd_find_log_state(c"cmd_find_from_winlink".as_ptr(), fs);
    }
}
pub unsafe fn cmd_find_from_session_window(
    fs: &mut cmd_find_state,
    mut s: *mut session,
    mut w: *mut window,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        cmd_find_clear_state(fs, flags);
        (*fs).set_session(s);
        (*fs).set_window(w);
        if cmd_find_best_winlink_with_window(fs) != 0 as ::core::ffi::c_int {
            cmd_find_clear_state(fs, flags);
            return -(1 as ::core::ffi::c_int);
        }
        (*fs).set_pane(window_get_active((*fs).window()));
        cmd_find_log_state(c"cmd_find_from_session_window".as_ptr(), fs);
        0 as ::core::ffi::c_int
    }
}
pub unsafe fn cmd_find_from_window(
    fs: &mut cmd_find_state,
    mut w: *mut window,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        cmd_find_clear_state(fs, flags);
        (*fs).set_window(w);
        if cmd_find_best_session_with_window(fs) != 0 as ::core::ffi::c_int {
            cmd_find_clear_state(fs, flags);
            return -(1 as ::core::ffi::c_int);
        }
        if cmd_find_best_winlink_with_window(fs) != 0 as ::core::ffi::c_int {
            cmd_find_clear_state(fs, flags);
            return -(1 as ::core::ffi::c_int);
        }
        (*fs).set_pane(window_get_active((*fs).window()));
        cmd_find_log_state(c"cmd_find_from_window".as_ptr(), fs);
        0 as ::core::ffi::c_int
    }
}
pub unsafe fn cmd_find_from_winlink_pane(
    fs: &mut cmd_find_state,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
) {
    unsafe {
        cmd_find_clear_state(fs, flags);
        (*fs).set_session((*wl).session());
        (*fs).set_winlink(wl);
        fs.idx = (*(*fs).winlink()).idx;
        (*fs).set_window((*(*fs).winlink()).window());
        (*fs).set_pane(wp);
        cmd_find_log_state(c"cmd_find_from_winlink_pane".as_ptr(), fs);
    }
}
pub unsafe fn cmd_find_from_pane(
    fs: &mut cmd_find_state,
    mut wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if cmd_find_from_window(fs, (*wp).window, flags) != 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
        (*fs).set_pane(wp);
        cmd_find_log_state(c"cmd_find_from_pane".as_ptr(), fs);
        0 as ::core::ffi::c_int
    }
}
pub unsafe fn cmd_find_from_nothing(
    fs: &mut cmd_find_state,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        cmd_find_clear_state(fs, flags);
        (*fs).set_session(cmd_find_best_session(&[], flags));
        if (*fs).session().is_null() {
            cmd_find_clear_state(fs, flags);
            return -(1 as ::core::ffi::c_int);
        }
        (*fs).set_winlink(session_get_curw((*fs).session()));
        fs.idx = (*(*fs).winlink()).idx;
        (*fs).set_window((*(*fs).winlink()).window());
        (*fs).set_pane(window_get_active((*fs).window()));
        cmd_find_log_state(c"cmd_find_from_nothing".as_ptr(), fs);
        0 as ::core::ffi::c_int
    }
}
pub unsafe fn cmd_find_from_mouse(
    fs: &mut cmd_find_state,
    mut m: *mut mouse_event,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        cmd_find_clear_state(fs, flags);
        if (*m).valid == 0 {
            return -(1 as ::core::ffi::c_int);
        }
        if let Some((found, link, pane)) = cmd_mouse_pane(m) {
            (*fs).set_session(found);
            (*fs).set_winlink(link);
            (*fs).set_pane(pane);
        } else {
            (*fs).set_pane(::core::ptr::null_mut::<window_pane>());
        }
        if (*fs).pane().is_null() {
            cmd_find_clear_state(fs, flags);
            return -(1 as ::core::ffi::c_int);
        }
        (*fs).set_window((*(*fs).winlink()).window());
        cmd_find_log_state(c"cmd_find_from_mouse".as_ptr(), fs);
        0 as ::core::ffi::c_int
    }
}
pub unsafe fn cmd_find_from_client(
    fs: &mut cmd_find_state,
    mut c: *mut client,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        if c.is_null() {
            return cmd_find_from_nothing(fs, flags);
        }
        if !(*c).session.is_null() {
            cmd_find_clear_state(fs, flags);
            (*fs).set_pane(server_client_get_pane(c));
            if (*fs).pane().is_null() {
                cmd_find_from_session(fs, (*c).session, flags);
                return 0 as ::core::ffi::c_int;
            }
            (*fs).set_session((*c).session);
            (*fs).set_winlink(session_get_curw((*fs).session()));
            (*fs).set_window((*(*fs).winlink()).window());
            cmd_find_log_state(c"cmd_find_from_client".as_ptr(), fs);
            return 0 as ::core::ffi::c_int;
        }
        cmd_find_clear_state(fs, flags);
        wp = cmd_find_inside_pane(c);
        if !wp.is_null() {
            (*fs).set_window((*wp).window);
            if !(cmd_find_best_session_with_window(fs) != 0 as ::core::ffi::c_int) {
                (*fs).set_winlink(session_get_curw((*fs).session()));
                (*fs).set_window((*(*fs).winlink()).window());
                (*fs).set_pane(window_get_active((*fs).window()));
                cmd_find_log_state(c"cmd_find_from_client".as_ptr(), fs);
                return 0 as ::core::ffi::c_int;
            }
        }
        cmd_find_from_nothing(fs, flags)
    }
}
pub unsafe fn cmd_find_target(
    fs: &mut cmd_find_state,
    mut item: *mut cmdq_item,
    mut target: *const ::core::ffi::c_char,
    mut type_0: cmd_find_type,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut current_block: u64;
        let mut m: *mut mouse_event = ::core::ptr::null_mut::<mouse_event>();
        let mut c: *mut client = ::core::ptr::null_mut::<client>();
        let mut current = cmd_find_state::default();
        let mut colon: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut period: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut session: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut window: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut pane: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut window_only: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut pane_only: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if flags & CMD_FIND_CANFAIL != 0 {
            flags |= CMD_FIND_QUIET;
        }
        if type_0 as ::core::ffi::c_uint
            == CMD_FIND_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            s = c"pane".as_ptr();
        } else if type_0 as ::core::ffi::c_uint
            == CMD_FIND_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            s = c"window".as_ptr();
        } else if type_0 as ::core::ffi::c_uint
            == CMD_FIND_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            s = c"session".as_ptr();
        } else {
            s = c"unknown".as_ptr();
        }
        let mut named: Vec<&::core::ffi::CStr> = Vec::new();
        for (bit, name) in [
            (CMD_FIND_PREFER_UNATTACHED, c"PREFER_UNATTACHED"),
            (CMD_FIND_QUIET, c"QUIET"),
            (CMD_FIND_WINDOW_INDEX, c"WINDOW_INDEX"),
            (CMD_FIND_DEFAULT_MARKED, c"DEFAULT_MARKED"),
            (CMD_FIND_EXACT_SESSION, c"EXACT_SESSION"),
            (CMD_FIND_EXACT_WINDOW, c"EXACT_WINDOW"),
            (CMD_FIND_CANFAIL, c"CANFAIL"),
        ] {
            if flags & bit != 0 {
                named.push(name);
            }
        }
        let mut tmp: Vec<u8> = Vec::new();
        if named.is_empty() {
            tmp.extend_from_slice(b"NONE");
        } else {
            for (at, name) in named.iter().enumerate() {
                if at != 0 {
                    tmp.push(b',');
                }
                tmp.extend_from_slice(name.to_bytes());
            }
        }
        let tmp = CString::new(tmp).expect("flag names have no NUL");
        log_debug(
            c"%s: target %s, type %s, item %p, flags %s".as_ptr(),
            fmt_args![
                c"cmd_find_target".as_ptr(),
                if target.is_null() {
                    c"none".as_ptr()
                } else {
                    target
                },
                s,
                item,
                tmp.as_ptr()
            ],
        );
        cmd_find_clear_state(fs, flags);
        let mut current_state: *const cmd_find_state = ::core::ptr::null();
        if server_check_marked() != 0 && flags & CMD_FIND_DEFAULT_MARKED != 0 {
            current_state = &raw const marked_pane;
            log_debug(
                c"%s: current is marked pane".as_ptr(),
                fmt_args![c"cmd_find_target".as_ptr()],
            );
            current_block = 1836292691772056875;
        } else if cmd_find_valid_state(&*cmdq_get_current(item)) != 0 {
            current_state = cmdq_get_current(item);
            log_debug(
                c"%s: current is from queue".as_ptr(),
                fmt_args![c"cmd_find_target".as_ptr()],
            );
            current_block = 1836292691772056875;
        } else if cmd_find_from_client(&mut current, cmdq_get_client(&*item), flags)
            == 0 as ::core::ffi::c_int
        {
            current_state = &raw const current;
            log_debug(
                c"%s: current is from client".as_ptr(),
                fmt_args![c"cmd_find_target".as_ptr()],
            );
            current_block = 1836292691772056875;
        } else {
            if !flags & CMD_FIND_QUIET != 0 {
                cmdq_error(item, c"no current target".as_ptr(), fmt_args![]);
            }
            current_block = 5874756215481722497;
        }
        if current_block == 1836292691772056875 {
            if cmd_find_valid_state(&*current_state) == 0 {
                fatalx(c"invalid current find state".as_ptr(), fmt_args![]);
            }
            if target.is_null() || *target as ::core::ffi::c_int == '\0' as i32 {
                current_block = 1976276580247140074;
            } else if strcmp(target, c"@".as_ptr()) == 0 as ::core::ffi::c_int
                || strcmp(target, c"{active}".as_ptr()) == 0 as ::core::ffi::c_int
                || strcmp(target, c"{current}".as_ptr()) == 0 as ::core::ffi::c_int
            {
                c = cmdq_get_client(&*item);
                if c.is_null() {
                    cmdq_error(item, c"no current client".as_ptr(), fmt_args![]);
                    current_block = 5874756215481722497;
                } else {
                    (*fs).set_winlink(session_get_curw((*c).session));
                    (*fs).set_pane(window_get_active(
                        (*session_get_curw((*c).session)).window(),
                    ));
                    (*fs).set_window((*session_get_curw((*c).session)).window());
                    current_block = 8711307700714518445;
                }
            } else if strcmp(target, c"=".as_ptr()) == 0 as ::core::ffi::c_int
                || strcmp(target, c"{mouse}".as_ptr()) == 0 as ::core::ffi::c_int
            {
                m = &raw mut (*(cmdq_get_event)(item)).m;
                let mut current_block_56: u64;
                match type_0 {
                    CMD_FIND_PANE => {
                        if let Some((found, link, pane)) = cmd_mouse_pane(m) {
                            (*fs).set_session(found);
                            (*fs).set_winlink(link);
                            (*fs).set_pane(pane);
                        } else {
                            (*fs).set_pane(::core::ptr::null_mut::<window_pane>());
                        }
                        if !(*fs).pane().is_null() {
                            (*fs).set_window((*(*fs).winlink()).window());
                            current_block_56 = 7343950298149844727;
                        } else {
                            current_block_56 = 1142519184231123645;
                        }
                    }
                    CMD_FIND_WINDOW | CMD_FIND_SESSION => {
                        current_block_56 = 1142519184231123645;
                    }
                    _ => {
                        current_block_56 = 7343950298149844727;
                    }
                }
                if current_block_56 == 1142519184231123645 {
                    if let Some((found, link)) = cmd_mouse_window(m) {
                        (*fs).set_session(found);
                        (*fs).set_winlink(link);
                    } else {
                        (*fs).set_winlink(::core::ptr::null_mut::<winlink>());
                    }
                    if (*fs).winlink().is_null() && !(*fs).session().is_null() {
                        (*fs).set_winlink(session_get_curw((*fs).session()));
                    }
                    if !(*fs).winlink().is_null() {
                        (*fs).set_window((*(*fs).winlink()).window());
                        (*fs).set_pane(window_get_active((*fs).window()));
                    }
                }
                if (*fs).pane().is_null() {
                    if !flags & CMD_FIND_QUIET != 0 {
                        cmdq_error(item, c"no mouse target".as_ptr(), fmt_args![]);
                    }
                    current_block = 5874756215481722497;
                } else {
                    current_block = 8711307700714518445;
                }
            } else if strcmp(target, c"~".as_ptr()) == 0 as ::core::ffi::c_int
                || strcmp(target, c"{marked}".as_ptr()) == 0 as ::core::ffi::c_int
            {
                if server_check_marked() == 0 {
                    if !flags & CMD_FIND_QUIET != 0 {
                        cmdq_error(item, c"no marked target".as_ptr(), fmt_args![]);
                    }
                    current_block = 5874756215481722497;
                } else {
                    cmd_find_copy_state(fs, &marked_pane);
                    current_block = 8711307700714518445;
                }
            } else {
                let copy = ::std::ffi::CString::new(::core::ffi::CStr::from_ptr(target).to_bytes())
                    .unwrap();
                let copy_ptr = copy.as_ptr() as *mut ::core::ffi::c_char;
                colon = strchr(copy_ptr, ':' as i32);
                if !colon.is_null() {
                    let fresh0 = colon;
                    colon = colon.offset(1);
                    *fresh0 = '\0' as i32 as ::core::ffi::c_char;
                }
                if colon.is_null() {
                    period = strchr(copy_ptr, '.' as i32);
                } else {
                    period = strchr(colon, '.' as i32);
                }
                if !period.is_null() {
                    let fresh1 = period;
                    period = period.offset(1);
                    *fresh1 = '\0' as i32 as ::core::ffi::c_char;
                }
                pane = ::core::ptr::null::<::core::ffi::c_char>();
                window = pane;
                session = window;
                if !colon.is_null() && !period.is_null() {
                    session = copy_ptr;
                    window = colon;
                    window_only = 1 as ::core::ffi::c_int;
                    pane = period;
                    pane_only = 1 as ::core::ffi::c_int;
                } else if !colon.is_null() && period.is_null() {
                    session = copy_ptr;
                    window = colon;
                    window_only = 1 as ::core::ffi::c_int;
                } else if colon.is_null() && !period.is_null() {
                    window = copy_ptr;
                    pane = period;
                    pane_only = 1 as ::core::ffi::c_int;
                } else if *copy_ptr as ::core::ffi::c_int == '$' as i32 {
                    session = copy_ptr;
                } else if *copy_ptr as ::core::ffi::c_int == '@' as i32 {
                    window = copy_ptr;
                } else if *copy_ptr as ::core::ffi::c_int == '%' as i32 {
                    pane = copy_ptr;
                } else {
                    match type_0 {
                        CMD_FIND_SESSION => {
                            session = copy_ptr;
                        }
                        CMD_FIND_WINDOW => {
                            window = copy_ptr;
                        }
                        CMD_FIND_PANE => {
                            pane = copy_ptr;
                        }
                        _ => {}
                    }
                }
                if !session.is_null() && *session as ::core::ffi::c_int == '=' as i32 {
                    session = session.offset(1);
                    fs.flags |= CMD_FIND_EXACT_SESSION;
                }
                if !window.is_null() && *window as ::core::ffi::c_int == '=' as i32 {
                    window = window.offset(1);
                    fs.flags |= CMD_FIND_EXACT_WINDOW;
                }
                if !session.is_null() && *session as ::core::ffi::c_int == '\0' as i32 {
                    session = ::core::ptr::null::<::core::ffi::c_char>();
                }
                if !window.is_null() && *window as ::core::ffi::c_int == '\0' as i32 {
                    window = ::core::ptr::null::<::core::ffi::c_char>();
                }
                if !pane.is_null() && *pane as ::core::ffi::c_int == '\0' as i32 {
                    pane = ::core::ptr::null::<::core::ffi::c_char>();
                }
                if !session.is_null() {
                    session = cmd_find_map_table(cmd_find_session_table.as_ptr(), session);
                }
                if !window.is_null() {
                    window = cmd_find_map_table(cmd_find_window_table.as_ptr(), window);
                }
                if !pane.is_null() {
                    pane = cmd_find_map_table(cmd_find_pane_table.as_ptr(), pane);
                }
                if !session.is_null() || !window.is_null() || !pane.is_null() {
                    log_debug(
                        c"%s: target %s is %s%s%s%s%s%s".as_ptr(),
                        fmt_args![
                            c"cmd_find_target".as_ptr(),
                            target,
                            if session.is_null() {
                                c"".as_ptr()
                            } else {
                                c"session ".as_ptr()
                            },
                            if session.is_null() {
                                c"".as_ptr()
                            } else {
                                session
                            },
                            if window.is_null() {
                                c"".as_ptr()
                            } else {
                                c"window ".as_ptr()
                            },
                            if window.is_null() {
                                c"".as_ptr()
                            } else {
                                window
                            },
                            if pane.is_null() {
                                c"".as_ptr()
                            } else {
                                c"pane ".as_ptr()
                            },
                            if pane.is_null() { c"".as_ptr() } else { pane }
                        ],
                    );
                }
                if !pane.is_null() && flags & CMD_FIND_WINDOW_INDEX != 0 {
                    if !flags & CMD_FIND_QUIET != 0 {
                        cmdq_error(item, c"can't specify pane here".as_ptr(), fmt_args![]);
                    }
                    current_block = 5874756215481722497;
                } else {
                    if !session.is_null() {
                        if cmd_find_get_session(fs, session) != 0 as ::core::ffi::c_int {
                            if !flags & CMD_FIND_QUIET != 0 {
                                cmdq_error(
                                    item,
                                    c"can't find session: %s".as_ptr(),
                                    fmt_args![session],
                                );
                            }
                            current_block = 5874756215481722497;
                        } else if window.is_null() && pane.is_null() {
                            (*fs).set_winlink(session_get_curw((*fs).session()));
                            fs.idx = -(1 as ::core::ffi::c_int);
                            (*fs).set_window((*(*fs).winlink()).window());
                            (*fs).set_pane(window_get_active((*fs).window()));
                            current_block = 8711307700714518445;
                        } else if !window.is_null() && pane.is_null() {
                            if cmd_find_get_window_with_session(fs, window)
                                != 0 as ::core::ffi::c_int
                            {
                                current_block = 8113074638138919487;
                            } else {
                                if !(*fs).winlink().is_null() {
                                    (*fs).set_pane(window_get_active((*(*fs).winlink()).window()));
                                }
                                current_block = 8711307700714518445;
                            }
                        } else if window.is_null() && !pane.is_null() {
                            if cmd_find_get_pane_with_session(fs, pane) != 0 as ::core::ffi::c_int {
                                current_block = 251766546907006956;
                            } else {
                                current_block = 8711307700714518445;
                            }
                        } else if cmd_find_get_window_with_session(fs, window)
                            != 0 as ::core::ffi::c_int
                        {
                            current_block = 8113074638138919487;
                        } else if cmd_find_get_pane_with_window(fs, pane) != 0 as ::core::ffi::c_int
                        {
                            current_block = 251766546907006956;
                        } else {
                            current_block = 8711307700714518445;
                        }
                    } else if !window.is_null() && !pane.is_null() {
                        if cmd_find_get_window(&*current_state, fs, window, window_only)
                            != 0 as ::core::ffi::c_int
                        {
                            current_block = 8113074638138919487;
                        } else if cmd_find_get_pane_with_window(fs, pane) != 0 as ::core::ffi::c_int
                        {
                            current_block = 251766546907006956;
                        } else {
                            current_block = 8711307700714518445;
                        }
                    } else if !window.is_null() && pane.is_null() {
                        if cmd_find_get_window(&*current_state, fs, window, window_only)
                            != 0 as ::core::ffi::c_int
                        {
                            current_block = 8113074638138919487;
                        } else {
                            if !(*fs).winlink().is_null() {
                                (*fs).set_pane(window_get_active((*(*fs).winlink()).window()));
                            }
                            current_block = 8711307700714518445;
                        }
                    } else if window.is_null() && !pane.is_null() {
                        if cmd_find_get_pane(&*current_state, fs, pane, pane_only)
                            != 0 as ::core::ffi::c_int
                        {
                            current_block = 251766546907006956;
                        } else {
                            current_block = 8711307700714518445;
                        }
                    } else {
                        current_block = 1976276580247140074;
                    }
                    match current_block {
                        5874756215481722497 => {}
                        8711307700714518445 => {}
                        1976276580247140074 => {}
                        _ => {
                            match current_block {
                                8113074638138919487 => {
                                    if !flags & CMD_FIND_QUIET != 0 {
                                        cmdq_error(
                                            item,
                                            c"can't find window: %s".as_ptr(),
                                            fmt_args![window],
                                        );
                                    }
                                }
                                _ => {
                                    if !flags & CMD_FIND_QUIET != 0 {
                                        cmdq_error(
                                            item,
                                            c"can't find pane: %s".as_ptr(),
                                            fmt_args![pane],
                                        );
                                    }
                                }
                            }
                            current_block = 5874756215481722497;
                        }
                    }
                }
            }
            match current_block {
                5874756215481722497 => {}
                _ => {
                    if current_block == 1976276580247140074 {
                        cmd_find_copy_state(fs, &*current_state);
                        if flags & CMD_FIND_WINDOW_INDEX != 0 {
                            fs.idx = -(1 as ::core::ffi::c_int);
                        }
                    }
                    cmd_find_log_state(c"cmd_find_target".as_ptr(), fs);
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
        log_debug(
            c"%s: error".as_ptr(),
            fmt_args![c"cmd_find_target".as_ptr()],
        );
        if flags & CMD_FIND_CANFAIL != 0 {
            return 0 as ::core::ffi::c_int;
        }
        -(1 as ::core::ffi::c_int)
    }
}
unsafe fn cmd_find_current_client(
    mut item: *mut cmdq_item,
    mut quiet: ::core::ffi::c_int,
) -> *mut client {
    unsafe {
        let mut c: *mut client = ::core::ptr::null_mut::<client>();
        let mut found: *mut client = ::core::ptr::null_mut::<client>();
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut fs = cmd_find_state::default();
        if !item.is_null() {
            c = cmdq_get_client(&*item);
        }
        if !c.is_null() && !(*c).session.is_null() {
            return c;
        }
        found = ::core::ptr::null_mut::<client>();
        if !c.is_null() && {
            wp = cmd_find_inside_pane(c);
            !wp.is_null()
        } {
            cmd_find_clear_state(&mut fs, CMD_FIND_QUIET);
            fs.set_window((*wp).window);
            if cmd_find_best_session_with_window(&mut fs) == 0 as ::core::ffi::c_int {
                found = cmd_find_best_client(fs.session());
            }
        } else {
            s = cmd_find_best_session(&[], CMD_FIND_QUIET);
            if !s.is_null() {
                found = cmd_find_best_client(s);
            }
        }
        if found.is_null() && !item.is_null() && quiet == 0 {
            cmdq_error(item, c"no current client".as_ptr(), fmt_args![]);
        }
        log_debug(
            c"%s: no target, return %p".as_ptr(),
            fmt_args![c"cmd_find_current_client".as_ptr(), found],
        );
        found
    }
}
pub unsafe fn cmd_find_client(
    mut item: *mut cmdq_item,
    mut target: *const ::core::ffi::c_char,
    mut quiet: ::core::ffi::c_int,
) -> *mut client {
    unsafe {
        let mut c: *mut client = ::core::ptr::null_mut::<client>();
        if target.is_null() {
            return cmd_find_current_client(item, quiet);
        }
        let target_bytes = CStr::from_ptr(target).to_bytes();
        let target_bytes = target_bytes.strip_suffix(b":").unwrap_or(target_bytes);
        let copy = CString::new(target_bytes).expect("a C string has no interior NUL");
        for candidate in client_walk() {
            if (*candidate).session.is_null() {
                continue;
            }
            if strcmp(copy.as_ptr(), cstr_ptr(&(*candidate).name)) == 0 as ::core::ffi::c_int {
                c = candidate;
                break;
            }
            if (*candidate).ttyname.as_ref().unwrap().as_bytes().is_empty() {
                continue;
            }
            if strcmp(copy.as_ptr(), cstr_ptr(&(*candidate).ttyname)) == 0 as ::core::ffi::c_int {
                c = candidate;
                break;
            }
            if !(strncmp(
                cstr_ptr(&(*candidate).ttyname),
                _PATH_DEV.as_ptr(),
                (::core::mem::size_of::<[::core::ffi::c_char; 6]>() as size_t)
                    .wrapping_sub(1 as size_t),
            ) != 0 as ::core::ffi::c_int)
                && strcmp(
                    copy.as_ptr(),
                    cstr_ptr(&(*candidate).ttyname)
                        .add(::core::mem::size_of::<[::core::ffi::c_char; 6]>() as usize)
                        .offset(-(1 as ::core::ffi::c_int as isize)),
                ) == 0 as ::core::ffi::c_int
            {
                c = candidate;
                break;
            }
        }
        if c.is_null() && quiet == 0 {
            cmdq_error(
                item,
                c"can't find client: %s".as_ptr(),
                fmt_args![copy.as_ptr()],
            );
        }
        log_debug(
            c"%s: target %s, return %p".as_ptr(),
            fmt_args![c"cmd_find_client".as_ptr(), target, c],
        );
        c
    }
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

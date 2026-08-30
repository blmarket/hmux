use super::state::control_write;
use crate::fmt_args;
use crate::format::format_single;
use crate::layout::layout_root_ptr;
use crate::server::client_walk;
use crate::session::{session_get_curw, session_id, session_name};
pub use crate::types::*;
use crate::window::window_get_active;
use crate::window::winlink_find_by_window_id;
use crate::window::winlinks_into;
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
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub fn control_notify_pane_mode_changed(mut pane: ::core::ffi::c_int) {
    unsafe {
        for c in client_walk() {
            if !c.is_null()
                && (*c).flags & CLIENT_CONTROL as uint64_t != 0
                && (*c).control_state.is_some()
            {
                control_write(c, c"%%pane-mode-changed %%%u".as_ptr(), fmt_args![pane]);
            }
        }
    }
}
pub unsafe fn control_notify_window_layout_changed(mut w: *mut window) {
    unsafe {
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        template = c"%layout-change #{window_id} #{window_layout} #{window_visible_layout} #{window_raw_flags}".as_ptr();
        wl = winlinks_into(w)
            .next()
            .unwrap_or(::core::ptr::null_mut::<winlink>());
        if wl.is_null() || layout_root_ptr(&(*w).layout_root).is_null() {
            return;
        }
        let cp = format_single(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ffi::CStr::from_ptr(template),
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<session>(),
            wl,
            ::core::ptr::null_mut::<window_pane>(),
        );
        for c in client_walk() {
            if !(!(!c.is_null()
                && (*c).flags & CLIENT_CONTROL as uint64_t != 0
                && (*c).control_state.is_some())
                || (*c).session.is_null())
            {
                s = (*c).session;
                if !winlink_find_by_window_id(&raw mut (*s).windows, (*w).id).is_null() {
                    control_write(c, c"%s".as_ptr(), fmt_args![cp.as_ptr()]);
                }
            }
        }
    }
}
pub unsafe fn control_notify_window_pane_changed(mut w: *mut window) {
    unsafe {
        if window_get_active(w).is_null() {
            return;
        }
        for c in client_walk() {
            if !c.is_null()
                && (*c).flags & CLIENT_CONTROL as uint64_t != 0
                && (*c).control_state.is_some()
            {
                control_write(
                    c,
                    c"%%window-pane-changed @%u %%%u".as_ptr(),
                    fmt_args![(*w).id, (*window_get_active(w)).id],
                );
            }
        }
    }
}
pub unsafe fn control_notify_window_unlinked(_s: *mut session, mut w: *mut window) {
    unsafe {
        let mut cs: *mut session = ::core::ptr::null_mut::<session>();
        for c in client_walk() {
            if !(!(!c.is_null()
                && (*c).flags & CLIENT_CONTROL as uint64_t != 0
                && (*c).control_state.is_some())
                || (*c).session.is_null())
            {
                cs = (*c).session;
                if !winlink_find_by_window_id(&raw mut (*cs).windows, (*w).id).is_null() {
                    control_write(c, c"%%window-close @%u".as_ptr(), fmt_args![(*w).id]);
                } else {
                    control_write(
                        c,
                        c"%%unlinked-window-close @%u".as_ptr(),
                        fmt_args![(*w).id],
                    );
                }
            }
        }
    }
}
pub unsafe fn control_notify_window_linked(_s: *mut session, mut w: *mut window) {
    unsafe {
        let mut cs: *mut session = ::core::ptr::null_mut::<session>();
        for c in client_walk() {
            if !(!(!c.is_null()
                && (*c).flags & CLIENT_CONTROL as uint64_t != 0
                && (*c).control_state.is_some())
                || (*c).session.is_null())
            {
                cs = (*c).session;
                if !winlink_find_by_window_id(&raw mut (*cs).windows, (*w).id).is_null() {
                    control_write(c, c"%%window-add @%u".as_ptr(), fmt_args![(*w).id]);
                } else {
                    control_write(c, c"%%unlinked-window-add @%u".as_ptr(), fmt_args![(*w).id]);
                }
            }
        }
    }
}
pub unsafe fn control_notify_window_renamed(mut w: *mut window) {
    unsafe {
        let mut cs: *mut session = ::core::ptr::null_mut::<session>();
        for c in client_walk() {
            if !(!(!c.is_null()
                && (*c).flags & CLIENT_CONTROL as uint64_t != 0
                && (*c).control_state.is_some())
                || (*c).session.is_null())
            {
                cs = (*c).session;
                if !winlink_find_by_window_id(&raw mut (*cs).windows, (*w).id).is_null() {
                    control_write(
                        c,
                        c"%%window-renamed @%u %s".as_ptr(),
                        fmt_args![(*w).id, cstr_ptr(&(*w).name)],
                    );
                } else {
                    control_write(
                        c,
                        c"%%unlinked-window-renamed @%u %s".as_ptr(),
                        fmt_args![(*w).id, cstr_ptr(&(*w).name)],
                    );
                }
            }
        }
    }
}
pub unsafe fn control_notify_client_session_changed(mut cc: *mut client) {
    unsafe {
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        if (*cc).session.is_null() {
            return;
        }
        s = (*cc).session;
        for c in client_walk() {
            if !(!(!c.is_null()
                && (*c).flags & CLIENT_CONTROL as uint64_t != 0
                && (*c).control_state.is_some())
                || (*c).session.is_null())
            {
                if cc == c {
                    control_write(
                        c,
                        c"%%session-changed $%u %s".as_ptr(),
                        fmt_args![session_id(s), session_name(s)],
                    );
                } else {
                    control_write(
                        c,
                        c"%%client-session-changed %s $%u %s".as_ptr(),
                        fmt_args![cstr_ptr(&(*cc).name), session_id(s), session_name(s)],
                    );
                }
            }
        }
    }
}
pub unsafe fn control_notify_client_detached(mut cc: *mut client) {
    unsafe {
        for c in client_walk() {
            if !c.is_null()
                && (*c).flags & CLIENT_CONTROL as uint64_t != 0
                && (*c).control_state.is_some()
            {
                control_write(
                    c,
                    c"%%client-detached %s".as_ptr(),
                    fmt_args![cstr_ptr(&(*cc).name)],
                );
            }
        }
    }
}
pub unsafe fn control_notify_session_renamed(mut s: *mut session) {
    unsafe {
        for c in client_walk() {
            if !c.is_null()
                && (*c).flags & CLIENT_CONTROL as uint64_t != 0
                && (*c).control_state.is_some()
            {
                control_write(
                    c,
                    c"%%session-renamed $%u %s".as_ptr(),
                    fmt_args![session_id(s), session_name(s)],
                );
            }
        }
    }
}
pub unsafe fn control_notify_session_created(_s: *mut session) {
    unsafe {
        for c in client_walk() {
            if !c.is_null()
                && (*c).flags & CLIENT_CONTROL as uint64_t != 0
                && (*c).control_state.is_some()
            {
                control_write(c, c"%%sessions-changed".as_ptr(), fmt_args![]);
            }
        }
    }
}
pub unsafe fn control_notify_session_closed(_s: *mut session) {
    unsafe {
        for c in client_walk() {
            if !c.is_null()
                && (*c).flags & CLIENT_CONTROL as uint64_t != 0
                && (*c).control_state.is_some()
            {
                control_write(c, c"%%sessions-changed".as_ptr(), fmt_args![]);
            }
        }
    }
}
pub unsafe fn control_notify_session_window_changed(mut s: *mut session) {
    unsafe {
        for c in client_walk() {
            if !c.is_null()
                && (*c).flags & CLIENT_CONTROL as uint64_t != 0
                && (*c).control_state.is_some()
            {
                control_write(
                    c,
                    c"%%session-window-changed $%u @%u".as_ptr(),
                    fmt_args![session_id(s), (*(*session_get_curw(s)).window()).id],
                );
            }
        }
    }
}
pub unsafe fn control_notify_paste_buffer_changed(mut name: *const ::core::ffi::c_char) {
    unsafe {
        for c in client_walk() {
            if !c.is_null()
                && (*c).flags & CLIENT_CONTROL as uint64_t != 0
                && (*c).control_state.is_some()
            {
                control_write(c, c"%%paste-buffer-changed %s".as_ptr(), fmt_args![name]);
            }
        }
    }
}
pub unsafe fn control_notify_paste_buffer_deleted(mut name: *const ::core::ffi::c_char) {
    unsafe {
        for c in client_walk() {
            if !c.is_null()
                && (*c).flags & CLIENT_CONTROL as uint64_t != 0
                && (*c).control_state.is_some()
            {
                control_write(c, c"%%paste-buffer-deleted %s".as_ptr(), fmt_args![name]);
            }
        }
    }
}

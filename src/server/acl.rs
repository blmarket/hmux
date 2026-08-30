use super::run::client_walk;
use crate::cmd::cmdq_print;
use crate::ffi::{getpwuid, getuid};
use crate::fmt_args;
use crate::proc::{peer_ptr, proc_get_peer_uid};
use crate::tree::GlobalTree;
pub use crate::types::*;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct server_acl_user {
    pub uid: uid_t,
    pub flags: ::core::ffi::c_int,
}
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const CLIENT_READONLY: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const SERVER_ACL_READONLY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub static server_acl_entries: GlobalTree<uid_t, Box<server_acl_user>> = GlobalTree::new();
pub fn server_acl_init() {
    unsafe {
        server_acl_entries.map().clear();
        if getuid() != 0 as __uid_t {
            server_acl_user_allow(0 as uid_t);
        }
        server_acl_user_allow(getuid() as uid_t);
    }
}
pub fn server_acl_user_find(uid: uid_t) -> *mut server_acl_user {
    match server_acl_entries.map().get_mut(&uid) {
        Some(user) => &raw mut **user,
        None => ::core::ptr::null_mut::<server_acl_user>(),
    }
}
pub unsafe fn server_acl_display(mut item: *mut cmdq_item) {
    unsafe {
        let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
        let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        for loop_0 in server_acl_entries.map().values() {
            if !(loop_0.uid == 0 as uid_t) {
                pw = getpwuid(loop_0.uid as __uid_t);
                if !pw.is_null() {
                    name = (*pw).pw_name;
                } else {
                    name = c"unknown".as_ptr();
                }
                if loop_0.flags == SERVER_ACL_READONLY {
                    cmdq_print(item, c"%s (R)".as_ptr(), fmt_args![name]);
                } else {
                    cmdq_print(item, c"%s (W)".as_ptr(), fmt_args![name]);
                }
            }
        }
    }
}
pub fn server_acl_user_allow(uid: uid_t) {
    server_acl_entries
        .map()
        .entry(uid)
        .or_insert_with(|| Box::new(server_acl_user { uid, flags: 0 }));
}
pub fn server_acl_user_deny(uid: uid_t) {
    server_acl_entries.map().remove(&uid);
}
pub fn server_acl_user_allow_write(mut uid: uid_t) {
    unsafe {
        let mut user: *mut server_acl_user = ::core::ptr::null_mut::<server_acl_user>();
        user = server_acl_user_find(uid);
        if user.is_null() {
            return;
        }
        (*user).flags &= !SERVER_ACL_READONLY;
        for c in client_walk() {
            uid = proc_get_peer_uid(peer_ptr(&(*c).peer));
            if uid != -(1 as ::core::ffi::c_int) as uid_t && uid == (*user).uid {
                (*c).flags &= !CLIENT_READONLY as uint64_t;
            }
        }
    }
}
pub fn server_acl_user_deny_write(mut uid: uid_t) {
    unsafe {
        let mut user: *mut server_acl_user = ::core::ptr::null_mut::<server_acl_user>();
        user = server_acl_user_find(uid);
        if user.is_null() {
            return;
        }
        (*user).flags |= SERVER_ACL_READONLY;
        for c in client_walk() {
            uid = proc_get_peer_uid(peer_ptr(&(*c).peer));
            if uid != -(1 as ::core::ffi::c_int) as uid_t && uid == (*user).uid {
                (*c).flags |= CLIENT_READONLY as uint64_t;
            }
        }
    }
}
pub unsafe fn server_acl_join(mut c: *mut client) -> ::core::ffi::c_int {
    unsafe {
        let mut user: *mut server_acl_user = ::core::ptr::null_mut::<server_acl_user>();
        let mut uid: uid_t = 0;
        uid = proc_get_peer_uid(peer_ptr(&(*c).peer));
        if uid == -(1 as ::core::ffi::c_int) as uid_t {
            return 0 as ::core::ffi::c_int;
        }
        user = server_acl_user_find(uid);
        if user.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        if (*user).flags & SERVER_ACL_READONLY != 0 {
            (*c).flags |= CLIENT_READONLY as uint64_t;
        }
        1 as ::core::ffi::c_int
    }
}
pub unsafe fn server_acl_get_uid(mut user: *mut server_acl_user) -> uid_t {
    unsafe { (*user).uid }
}

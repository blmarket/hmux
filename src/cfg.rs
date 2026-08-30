use crate::cmd::{
    CmdqItemWeak, cmdq_add_format, cmdq_append, cmdq_continue, cmdq_copy_state, cmdq_get_callback1,
    cmdq_get_client, cmdq_get_command, cmdq_get_state_ref, cmdq_insert_after,
    cmdq_item_weak_from_ptr, cmdq_new_state, cmdq_print,
};
use crate::cmd::{cmd_parse_from_buffer, cmd_parse_from_file};
use crate::control::control_write;
use crate::ffi::strerror;
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc};
use crate::log::log_debug;
use crate::modes::window_copy_add;
use crate::server::client_ref_from_ptr;
use crate::server::first_client;
use crate::session::sessions_first;
use crate::session::{session_attached, session_get_curw};
use crate::status::status_prompt_load_history;
pub use crate::types::*;
use crate::window::window_get_active;
use crate::window::window_pane_current_mode;
use crate::window::window_pane_set_mode;
use ::std::ffi::{CStr, CString, OsStr};
use ::std::fs::File;
use ::std::io::Read;
use ::std::os::unix::ffi::OsStrExt;
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
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const CMD_PARSE_SUCCESS: cmd_parse_status = 1;
pub const CMD_PARSE_ERROR: cmd_parse_status = 0;
pub const ENOENT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const CMD_PARSE_QUIET: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_PARSE_PARSEONLY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
/// The client the config load is running for, observed rather than held, so
/// that a client which goes away mid-load leaves nothing behind.
static mut CFG_CLIENT: Option<ClientWeak> = None;

/// The client the config load is running for, or null once the load has no
/// client or that client has gone.
pub fn cfg_client() -> *mut client {
    unsafe {
        CFG_CLIENT
            .as_ref()
            .and_then(ClientWeak::upgrade)
            .map_or(::core::ptr::null_mut(), |c| c.as_ptr())
    }
}
pub static mut cfg_finished: ::core::ffi::c_int = 0;
static mut cfg_causes: Vec<CString> = Vec::new();
/// The item the config load has left waiting on the first client, while it
/// waits.
static mut cfg_item: Option<CmdqItemWeak> = None;
pub static mut cfg_quiet: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub static mut cfg_files: Vec<CString> = Vec::new();
unsafe fn cfg_client_done(_item: *mut cmdq_item, _data: CmdqCallbackData) -> cmd_retval {
    unsafe {
        if cfg_finished == 0 {
            return CMD_RETURN_WAIT;
        }
        CMD_RETURN_NORMAL
    }
}
unsafe fn cfg_done(_item: *mut cmdq_item, _data: CmdqCallbackData) -> cmd_retval {
    unsafe {
        if cfg_finished != 0 {
            return CMD_RETURN_NORMAL;
        }
        cfg_finished = 1 as ::core::ffi::c_int;
        cfg_show_causes(::core::ptr::null_mut::<session>());
        if let Some(item) = cfg_item.as_ref().and_then(CmdqItemWeak::upgrade) {
            cmdq_continue(item.as_ptr());
        }
        status_prompt_load_history();
        CMD_RETURN_NORMAL
    }
}
pub fn start_cfg() {
    unsafe {
        let mut c: *mut client = ::core::ptr::null_mut::<client>();
        let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        c = first_client();
        CFG_CLIENT = client_ref_from_ptr(c).map(|c| c.downgrade());
        if !c.is_null() {
            cfg_item = cmdq_item_weak_from_ptr(cmdq_append(
                c,
                cmdq_get_callback1(
                    c"cfg_client_done".as_ptr(),
                    Some(cfg_client_done),
                    CmdqCallbackData::None,
                ),
            ));
        }
        if cfg_quiet != 0 {
            flags = CMD_PARSE_QUIET;
        }
        for file in &cfg_files {
            load_cfg(
                file.as_ptr(),
                c,
                ::core::ptr::null_mut::<cmdq_item>(),
                ::core::ptr::null_mut::<cmd_find_state>(),
                flags,
                None,
            );
        }
        cmdq_append(
            ::core::ptr::null_mut::<client>(),
            cmdq_get_callback1(c"cfg_done".as_ptr(), Some(cfg_done), CmdqCallbackData::None),
        );
    }
}
pub unsafe fn load_cfg(
    mut path: *const ::core::ffi::c_char,
    mut c: *mut client,
    mut item: *mut cmdq_item,
    mut current: *mut cmd_find_state,
    mut flags: ::core::ffi::c_int,
    mut new_item: Option<&mut *mut cmdq_item>,
) -> ::core::ffi::c_int {
    unsafe {
        let mut pi = cmd_parse_input::default();
        if let Some(new_item) = new_item.as_deref_mut() {
            *new_item = ::core::ptr::null_mut::<cmdq_item>();
        }
        log_debug(c"loading %s".as_ptr(), fmt_args![path]);
        let mut file = match File::open(OsStr::from_bytes(CStr::from_ptr(path).to_bytes())) {
            Ok(file) => file,
            Err(err) => {
                let errno = err.raw_os_error().unwrap_or(0);
                if errno == ENOENT && flags & CMD_PARSE_QUIET != 0 {
                    return 0 as ::core::ffi::c_int;
                }
                cfg_add_cause(c"%s: %s".as_ptr(), fmt_args![path, strerror(errno)]);
                return -(1 as ::core::ffi::c_int);
            }
        };
        let mut contents: Vec<u8> = Vec::new();
        let _ = file.read_to_end(&mut contents);
        pi.flags = flags;
        pi.file = Some(CStr::from_ptr(path).to_owned());
        pi.line = 1 as u_int;
        pi.item = item;
        pi.c = client_ref_from_ptr(c).map(|c| c.downgrade());
        let mut pr = cmd_parse_from_file(contents, &raw mut pi);
        if pr.status as ::core::ffi::c_uint
            == CMD_PARSE_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let error = pr.error.take().unwrap();
            cfg_add_cause(c"%s".as_ptr(), fmt_args![error.as_ptr()]);
            return -(1 as ::core::ffi::c_int);
        }
        if flags & CMD_PARSE_PARSEONLY != 0 {
            let _ = pr.cmdlist.take();
            return 0 as ::core::ffi::c_int;
        }
        let state = if !item.is_null() {
            cmdq_copy_state(cmdq_get_state_ref(item), current)
        } else {
            cmdq_new_state(
                ::core::ptr::null_mut::<cmd_find_state>(),
                ::core::ptr::null_mut::<key_event>(),
                0 as ::core::ffi::c_int,
            )
        };
        cmdq_add_format(
            state.as_ptr(),
            c"current_file".as_ptr(),
            c"%s".as_ptr(),
            fmt_args![pi.file()],
        );
        let cmdlist = pr.cmdlist.take().unwrap();
        let queued = cmdq_get_command(&cmdlist, Some(&state));
        let last = if !item.is_null() {
            cmdq_insert_after(item, queued)
        } else {
            cmdq_append(::core::ptr::null_mut::<client>(), queued)
        };
        if let Some(new_item) = new_item {
            *new_item = last;
        }
        0 as ::core::ffi::c_int
    }
}
pub unsafe fn load_cfg_from_buffer(
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut path: *const ::core::ffi::c_char,
    mut c: *mut client,
    mut item: *mut cmdq_item,
    mut current: *mut cmd_find_state,
    mut flags: ::core::ffi::c_int,
    mut new_item: Option<&mut *mut cmdq_item>,
) -> ::core::ffi::c_int {
    unsafe {
        let mut pi = cmd_parse_input::default();
        if let Some(new_item) = new_item.as_deref_mut() {
            *new_item = ::core::ptr::null_mut::<cmdq_item>();
        }
        log_debug(c"loading %s".as_ptr(), fmt_args![path]);
        pi.flags = flags;
        pi.file = Some(CStr::from_ptr(path).to_owned());
        pi.line = 1 as u_int;
        pi.item = item;
        pi.c = client_ref_from_ptr(c).map(|c| c.downgrade());
        let mut pr = cmd_parse_from_buffer(buf, len, &raw mut pi);
        if pr.status as ::core::ffi::c_uint
            == CMD_PARSE_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let error = pr.error.take().unwrap();
            cfg_add_cause(c"%s".as_ptr(), fmt_args![error.as_ptr()]);
            return -(1 as ::core::ffi::c_int);
        }
        if flags & CMD_PARSE_PARSEONLY != 0 {
            let _ = pr.cmdlist.take();
            return 0 as ::core::ffi::c_int;
        }
        let state = if !item.is_null() {
            cmdq_copy_state(cmdq_get_state_ref(item), current)
        } else {
            cmdq_new_state(
                ::core::ptr::null_mut::<cmd_find_state>(),
                ::core::ptr::null_mut::<key_event>(),
                0 as ::core::ffi::c_int,
            )
        };
        cmdq_add_format(
            state.as_ptr(),
            c"current_file".as_ptr(),
            c"%s".as_ptr(),
            fmt_args![pi.file()],
        );
        let cmdlist = pr.cmdlist.take().unwrap();
        let queued = cmdq_get_command(&cmdlist, Some(&state));
        let last = if !item.is_null() {
            cmdq_insert_after(item, queued)
        } else {
            cmdq_append(::core::ptr::null_mut::<client>(), queued)
        };
        if let Some(new_item) = new_item {
            *new_item = last;
        }
        0 as ::core::ffi::c_int
    }
}
pub unsafe fn cfg_add_cause(mut fmt: *const ::core::ffi::c_char, args: &[FmtArg]) {
    unsafe {
        let msg = format_alloc(fmt, args);
        cfg_causes.push(msg);
    }
}
pub unsafe fn cfg_print_causes(mut item: *mut cmdq_item) {
    unsafe {
        let mut c: *mut client = cmdq_get_client(&*item);
        for msg in ::core::mem::take(&mut cfg_causes) {
            if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                control_write(c, c"%%config-error %s".as_ptr(), fmt_args![msg.as_ptr()]);
            } else {
                cmdq_print(item, c"%s".as_ptr(), fmt_args![msg.as_ptr()]);
            }
        }
    }
}
pub unsafe fn cfg_show_causes(mut s: *mut session) {
    unsafe {
        let c: *mut client = first_client();
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
        if cfg_causes.is_empty() {
            return;
        }
        if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            for msg in ::core::mem::take(&mut cfg_causes) {
                control_write(c, c"%%config-error %s".as_ptr(), fmt_args![msg.as_ptr()]);
            }
        } else {
            if s.is_null() {
                if !c.is_null() && !(*c).session.is_null() {
                    s = (*c).session;
                } else {
                    s = sessions_first();
                }
            }
            if s.is_null() || session_attached(s) == 0 as u_int {
                return;
            }
            wp = window_get_active((*session_get_curw(s)).window());
            wme = window_pane_current_mode(wp);
            if wme.is_null() || (*wme).mode() != WindowMode::View {
                window_pane_set_mode(
                    wp,
                    ::core::ptr::null_mut::<window_pane>(),
                    WindowMode::View,
                    ::core::ptr::null_mut::<cmd_find_state>(),
                    ::core::ptr::null_mut::<args>(),
                );
            }
            for msg in ::core::mem::take(&mut cfg_causes) {
                window_copy_add(
                    wp,
                    0 as ::core::ffi::c_int,
                    c"%s".as_ptr(),
                    fmt_args![msg.as_ptr()],
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/test_cfg.rs"]
mod tests;

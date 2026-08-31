use crate::cmd::cmd_parse_and_append;
use crate::cmd::{cmdq_append, cmdq_get_callback1, cmdq_get_client, cmdq_guard, cmdq_new_state};
use crate::ffi::{close, strlen};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc, format_buf};
use crate::format::{format_create_defaults, format_expand};
use crate::list::foreach_safe;
use crate::log::{fatalx, log_debug};
use crate::reactor::{Interest, Timer};
use crate::server::client_ref_from_ptr;
use crate::session::session_id;
use crate::tmux::{get_timer, setblocking};
pub use crate::types::*;
use crate::window::winlinks_into;
use crate::window::{
    window_find_by_id, window_pane_find_by_id, window_pane_get_new_data,
    window_pane_update_used_data, window_panes_first, window_panes_next, winlink_find_by_window,
    winlinks_after, winlinks_first,
};
use ::core::ffi::CStr;
use ::std::ffi::CString;
pub const BUFFER_EOL_NUL: ::core::ffi::c_uint = 4;
pub const BUFFER_EOL_LF: ::core::ffi::c_uint = 3;
pub const BUFFER_EOL_CRLF_STRICT: ::core::ffi::c_uint = 2;
pub const BUFFER_EOL_CRLF: ::core::ffi::c_uint = 1;
pub const BUFFER_EOL_ANY: ::core::ffi::c_uint = 0;
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
#[derive(Default)]
#[repr(C)]
pub struct control_state {
    pub panes: control_panes,
    /// The panes with output waiting, in the order it arrived. The panes
    /// themselves belong to `panes`.
    pub pending_list: control_pending_list,
    pub pending_count: u_int,
    /// Every block this client has, whether a line or pane output, oldest
    /// first. A block is raw: `control_free_block` is what gives one up.
    pub all_blocks: control_blocks,
    pub read_event: Stream,
    pub write_event: Stream,
    pub subs: control_subs,
    pub subs_timer: TimerHandle,
}
/// The subscriptions of one control client, by name. The order decides the
/// order `%subscription-changed` lines come out in within a tick.
pub type control_subs = ::std::collections::BTreeMap<CString, Box<control_sub>>;

#[repr(C)]
pub struct control_sub {
    pub name: CString,
    pub format: CString,
    pub type_0: control_sub_type,
    pub id: u_int,
    pub last: Option<CString>,
    pub panes: control_sub_panes,
    pub windows: control_sub_windows,
}

/// The last value a subscription sent for a pane at a window index, which is
/// all that is kept about it.
pub type control_sub_panes = ::std::collections::BTreeMap<(u_int, u_int), CString>;

/// The last value a subscription sent for a window at an index.
pub type control_sub_windows = ::std::collections::BTreeMap<(u_int, u_int), CString>;
pub const CONTROL_SUB_ALL_WINDOWS: control_sub_type = 4;
pub const CONTROL_SUB_WINDOW: control_sub_type = 3;
pub const CONTROL_SUB_ALL_PANES: control_sub_type = 2;
pub const CONTROL_SUB_PANE: control_sub_type = 1;
pub const CONTROL_SUB_SESSION: control_sub_type = 0;

#[repr(C)]
pub struct control_block {
    pub size: size_t,
    pub line: Option<::std::ffi::CString>,
    pub t: uint64_t,
}

/// Blocks in the order they were made, which is the order they go out in.
/// This list owns them; a pane's list only points at them.
pub type control_blocks = ::std::vec::Vec<Box<control_block>>;

/// One pane's share of the blocks its client owns in [`control_blocks`].
pub type control_pane_blocks = ::std::vec::Vec<*mut control_block>;

/// Panes with output waiting, in the order it arrived.
pub type control_pending_list = ::std::vec::Vec<*mut control_pane>;

#[repr(C)]
pub struct control_pane {
    pub pane: u_int,
    pub offset: window_pane_offset,
    pub queued: window_pane_offset,
    pub flags: ::core::ffi::c_int,
    pub pending_flag: ::core::ffi::c_int,
    /// This pane's own output blocks, which the client's `all_blocks` owns.
    pub blocks: control_pane_blocks,
}

/// The panes a control client is watching, by pane id.
pub type control_panes = ::std::collections::BTreeMap<u_int, Box<control_pane>>;
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub const CMD_PARSE_SUCCESS: cmd_parse_status = 1;
pub const CMD_PARSE_ERROR: cmd_parse_status = 0;
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const EV_TIMEOUT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const EV_READ: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const EV_WRITE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMDQ_STATE_CONTROL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CLIENT_EXIT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CLIENT_SUSPENDED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLIENT_DEAD: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const CLIENT_CONTROLCONTROL: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const CLIENT_CONTROL_NOOUTPUT: ::core::ffi::c_int = 0x4000000 as ::core::ffi::c_int;
pub const CLIENT_CONTROL_PAUSEAFTER: ::core::ffi::c_ulonglong =
    0x100000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_UNATTACHEDFLAGS: ::core::ffi::c_int = CLIENT_DEAD | CLIENT_SUSPENDED | CLIENT_EXIT;
pub const CONTROL_PANE_OFF: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CONTROL_PANE_PAUSED: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CONTROL_BUFFER_LOW: ::core::ffi::c_int = 512 as ::core::ffi::c_int;
pub const CONTROL_BUFFER_HIGH: ::core::ffi::c_int = 8192 as ::core::ffi::c_int;
pub const CONTROL_WRITE_MINIMUM: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const CONTROL_MAXIMUM_AGE: ::core::ffi::c_int = 300000 as ::core::ffi::c_int;
pub const CONTROL_IGNORE_FLAGS: ::core::ffi::c_int =
    CLIENT_CONTROL_NOOUTPUT | CLIENT_UNATTACHEDFLAGS;
/// Takes `cb` off `blocks`, the pane list that only points at it.
unsafe fn control_unlink_block(blocks: *mut control_pane_blocks, cb: *mut control_block) {
    unsafe {
        if let Some(at) = (*blocks).iter().position(|&block| block == cb) {
            (*blocks).remove(at);
        }
    }
}

unsafe fn control_free_block(cs: *mut control_state, cb: *mut control_block) {
    unsafe {
        if let Some(at) = (*cs)
            .all_blocks
            .iter()
            .position(|block| ::core::ptr::eq(&raw const **block, cb))
        {
            (*cs).all_blocks.remove(at);
        }
    }
}
unsafe fn control_get_pane(c: *mut client, wp: *mut window_pane) -> *mut control_pane {
    unsafe {
        let cs: *mut control_state = control_state_of(c);
        (*cs)
            .panes
            .get(&(*wp).id)
            .map(|cp| cp.as_ref() as *const control_pane as *mut control_pane)
            .unwrap_or(::core::ptr::null_mut::<control_pane>())
    }
}
unsafe fn control_add_pane(mut c: *mut client, mut wp: *mut window_pane) -> *mut control_pane {
    unsafe {
        let mut cs: *mut control_state = control_state_of(c);
        let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
        cp = control_get_pane(c, wp);
        if !cp.is_null() {
            return cp;
        }
        let mut cp_box = Box::new(control_pane {
            pane: (*wp).id,
            offset: (*wp).offset,
            queued: (*wp).offset,
            flags: 0,
            pending_flag: 0,
            blocks: control_pane_blocks::new(),
        });
        cp = &raw mut *cp_box;
        (*cs).panes.insert((*cp).pane, cp_box);
        cp
    }
}
unsafe fn control_discard_pane(mut c: *mut client, mut cp: *mut control_pane) {
    unsafe {
        let cs: *mut control_state = control_state_of(c);
        for cb in ::core::mem::take(&mut (*cp).blocks) {
            control_free_block(cs, cb);
        }
    }
}
unsafe fn control_window_pane(mut c: *mut client, mut pane: u_int) -> *mut window_pane {
    unsafe {
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        if (*c).session.is_null() {
            return ::core::ptr::null_mut::<window_pane>();
        }
        wp = window_pane_find_by_id(pane);
        if wp.is_null() {
            return ::core::ptr::null_mut::<window_pane>();
        }
        if winlink_find_by_window(&raw mut (*(*c).session).windows, (*wp).window).is_null() {
            return ::core::ptr::null_mut::<window_pane>();
        }
        wp
    }
}
pub unsafe fn control_reset_offsets(mut c: *mut client) {
    unsafe {
        let mut cs: *mut control_state = control_state_of(c);
        (*cs).pending_list.clear();
        (*cs).pending_count = 0 as u_int;
        for cp in ::core::mem::take(&mut (*cs).panes).into_values() {
            let cp = cp.as_ref() as *const control_pane as *mut control_pane;
            control_discard_pane(c, cp);
        }
    }
}
/// Where a control client's reader stands in a pane's output, and whether
/// what it has already queued is enough to stop reading more.
pub unsafe fn control_pane_offset(
    mut c: *mut client,
    mut wp: *mut window_pane,
) -> (*mut window_pane_offset, ::core::ffi::c_int) {
    unsafe {
        let mut cs: *mut control_state = control_state_of(c);
        let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
        if (*c).flags & CLIENT_CONTROL_NOOUTPUT as uint64_t != 0 {
            return (::core::ptr::null_mut::<window_pane_offset>(), 0);
        }
        cp = control_get_pane(c, wp);
        if cp.is_null() || (*cp).flags & CONTROL_PANE_PAUSED != 0 {
            return (::core::ptr::null_mut::<window_pane_offset>(), 0);
        }
        if (*cp).flags & CONTROL_PANE_OFF != 0 {
            return (::core::ptr::null_mut::<window_pane_offset>(), 1);
        }
        let off =
            ((*cs).write_event.output_len() >= CONTROL_BUFFER_LOW as size_t) as ::core::ffi::c_int;
        (&raw mut (*cp).offset, off)
    }
}
pub unsafe fn control_set_pane_on(mut c: *mut client, mut wp: *mut window_pane) {
    unsafe {
        let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
        cp = control_get_pane(c, wp);
        if !cp.is_null() && (*cp).flags & CONTROL_PANE_OFF != 0 {
            (*cp).flags &= !CONTROL_PANE_OFF;
            (*cp).offset = (*wp).offset;
            (*cp).queued = (*wp).offset;
        }
    }
}
pub unsafe fn control_set_pane_off(mut c: *mut client, mut wp: *mut window_pane) {
    unsafe {
        let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
        cp = control_add_pane(c, wp);
        control_discard_pane(c, cp);
        (*cp).offset = (*wp).offset;
        (*cp).queued = (*wp).offset;
        (*cp).flags |= CONTROL_PANE_OFF;
    }
}
pub unsafe fn control_continue_pane(mut c: *mut client, mut wp: *mut window_pane) {
    unsafe {
        let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
        cp = control_get_pane(c, wp);
        if !cp.is_null() && (*cp).flags & CONTROL_PANE_PAUSED != 0 {
            (*cp).flags &= !CONTROL_PANE_PAUSED;
            (*cp).offset = (*wp).offset;
            (*cp).queued = (*wp).offset;
            control_write(c, c"%%continue %%%u".as_ptr(), fmt_args![(*wp).id]);
        }
    }
}
pub unsafe fn control_pause_pane(mut c: *mut client, mut wp: *mut window_pane) {
    unsafe {
        let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
        cp = control_add_pane(c, wp);
        if !(*cp).flags & CONTROL_PANE_PAUSED != 0 {
            (*cp).flags |= CONTROL_PANE_PAUSED;
            control_discard_pane(c, cp);
            control_write(c, c"%%pause %%%u".as_ptr(), fmt_args![(*wp).id]);
        }
    }
}
unsafe fn control_vwrite(mut c: *mut client, mut fmt: *const ::core::ffi::c_char, args: &[FmtArg]) {
    unsafe {
        let mut cs: *mut control_state = control_state_of(c);
        let s = format_alloc(fmt, args);
        log_debug(
            c"%s: %s: writing line: %s".as_ptr(),
            fmt_args![c"control_vwrite".as_ptr(), (*c).name.as_deref(), s.as_ptr()],
        );
        (*cs)
            .write_event
            .write(s.as_ptr() as *const u8, s.as_bytes().len());
        (*cs).write_event.write(b"\n\0".as_ptr(), 1 as size_t);
        (*cs).write_event.enable(Interest::Write);
    }
}
pub unsafe fn control_write(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    args: &[FmtArg],
) {
    unsafe {
        let mut cs: *mut control_state = control_state_of(c);
        if (*cs).all_blocks.is_empty() {
            control_vwrite(c, fmt, args);
            return;
        }
        let cb = Box::new(control_block {
            size: 0,
            line: Some(format_alloc(fmt, args)),
            t: get_timer(),
        });
        log_debug(
            c"%s: %s: storing line: %s".as_ptr(),
            fmt_args![
                c"control_write".as_ptr(),
                (*c).name.as_deref(),
                cb.line.as_deref()
            ],
        );
        (*cs).all_blocks.push(cb);
        (*cs).write_event.enable(Interest::Write);
    }
}
unsafe fn control_check_age(
    mut c: *mut client,
    mut wp: *mut window_pane,
    mut cp: *mut control_pane,
) -> ::core::ffi::c_int {
    unsafe {
        let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
        let mut t: uint64_t = 0;
        let mut age: uint64_t = 0;
        cb = (*cp)
            .blocks
            .first()
            .copied()
            .unwrap_or(::core::ptr::null_mut::<control_block>());
        if cb.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        t = get_timer();
        if (*cb).t >= t {
            return 0 as ::core::ffi::c_int;
        }
        age = t.wrapping_sub((*cb).t);
        log_debug(
            c"%s: %s: %%%u is %llu behind".as_ptr(),
            fmt_args![
                c"control_check_age".as_ptr(),
                (*c).name.as_deref(),
                (*wp).id,
                age as ::core::ffi::c_ulonglong
            ],
        );
        if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_PAUSEAFTER != 0 {
            if age < (*c).pause_age as uint64_t {
                return 0 as ::core::ffi::c_int;
            }
            (*cp).flags |= CONTROL_PANE_PAUSED;
            control_discard_pane(c, cp);
            control_write(c, c"%%pause %%%u".as_ptr(), fmt_args![(*wp).id]);
        } else {
            if age < CONTROL_MAXIMUM_AGE as uint64_t {
                return 0 as ::core::ffi::c_int;
            }
            (*c).exit_message = Some(c"too far behind".to_owned());
            (*c).flags |= CLIENT_EXIT as uint64_t;
            control_discard(c);
        }
        1 as ::core::ffi::c_int
    }
}
pub unsafe fn control_write_output(mut c: *mut client, mut wp: *mut window_pane) {
    unsafe {
        let mut cs: *mut control_state = control_state_of(c);
        let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
        let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
        let mut new_size: size_t = 0;
        if winlink_find_by_window(&raw mut (*(*c).session).windows, (*wp).window).is_null() {
            return;
        }
        if (*c).flags & CONTROL_IGNORE_FLAGS as uint64_t != 0 {
            cp = control_get_pane(c, wp);
            if cp.is_null() {
                return;
            }
        } else {
            cp = control_add_pane(c, wp);
            if !((*cp).flags & (CONTROL_PANE_OFF | CONTROL_PANE_PAUSED) != 0) {
                if control_check_age(c, wp, cp) != 0 {
                    return;
                }
                new_size = window_pane_get_new_data(wp, &raw mut (*cp).queued).1;
                if new_size == 0 as size_t {
                    return;
                }
                window_pane_update_used_data(wp, &raw mut (*cp).queued, new_size);
                let mut cb_owned = Box::new(control_block {
                    size: new_size,
                    line: None,
                    t: get_timer(),
                });
                cb = &raw mut *cb_owned;
                (*cs).all_blocks.push(cb_owned);
                (*cp).blocks.push(cb);
                log_debug(
                    c"%s: %s: new output block of %zu for %%%u".as_ptr(),
                    fmt_args![
                        c"control_write_output".as_ptr(),
                        (*c).name.as_deref(),
                        (*cb).size,
                        (*wp).id
                    ],
                );
                if (*cp).pending_flag == 0 {
                    log_debug(
                        c"%s: %s: %%%u now pending".as_ptr(),
                        fmt_args![
                            c"control_write_output".as_ptr(),
                            (*c).name.as_deref(),
                            (*wp).id
                        ],
                    );
                    (*cs).pending_list.push(cp);
                    (*cp).pending_flag = 1 as ::core::ffi::c_int;
                    (*cs).pending_count = (*cs).pending_count.wrapping_add(1);
                }
                (*cs).write_event.enable(Interest::Write);
                return;
            }
        }
        log_debug(
            c"%s: %s: ignoring pane %%%u".as_ptr(),
            fmt_args![
                c"control_write_output".as_ptr(),
                (*c).name.as_deref(),
                (*wp).id
            ],
        );
        window_pane_update_used_data(wp, &raw mut (*cp).offset, SIZE_MAX as size_t);
        window_pane_update_used_data(wp, &raw mut (*cp).queued, SIZE_MAX as size_t);
    }
}
unsafe fn control_error(mut item: *mut cmdq_item, data: CmdqCallbackData) -> cmd_retval {
    unsafe {
        let mut c: *mut client = cmdq_get_client(&*item);
        let CmdqCallbackData::String(error) = data else {
            return CMD_RETURN_ERROR;
        };
        cmdq_guard(item, c"begin".as_ptr(), 1 as ::core::ffi::c_int);
        control_write(c, c"parse error: %s".as_ptr(), fmt_args![error.as_ptr()]);
        cmdq_guard(item, c"error".as_ptr(), 1 as ::core::ffi::c_int);
        CMD_RETURN_NORMAL
    }
}
/// A stream callback that runs `body` on the client it was made for, and
/// does nothing at all once that client has gone.
fn on_client(
    watching: &Option<ClientWeak>,
    body: unsafe fn(*mut client),
) -> ::std::rc::Rc<dyn Fn(Stream)> {
    let watching = watching.clone();
    ::std::rc::Rc::new(move |_stream| {
        if let Some(c) = watching.as_ref().and_then(ClientWeak::upgrade) {
            unsafe { body(c.as_ptr()) };
        }
    })
}

/// The same, for the callback a failed stream makes.
fn on_client_error(
    watching: &Option<ClientWeak>,
    body: unsafe fn(*mut client),
) -> ::std::rc::Rc<dyn Fn(Stream, ::core::ffi::c_short)> {
    let watching = watching.clone();
    ::std::rc::Rc::new(move |_stream, _what| {
        if let Some(c) = watching.as_ref().and_then(ClientWeak::upgrade) {
            unsafe { body(c.as_ptr()) };
        }
    })
}

unsafe fn control_error_callback(mut c: *mut client) {
    unsafe {
        (*c).flags |= CLIENT_EXIT as uint64_t;
    }
}
unsafe fn control_read_callback(mut c: *mut client) {
    unsafe {
        let mut cs: *mut control_state = control_state_of(c);
        let mut error = None;
        loop {
            let Some(line) = (*cs)
                .read_event
                .with_input(|buffer| buffer.read_line())
                .flatten()
            else {
                break;
            };
            let mut line_data = line.to_vec();
            line_data.push(0);
            let line = line_data.as_mut_ptr() as *mut ::core::ffi::c_char;
            log_debug(
                c"%s: %s: %s".as_ptr(),
                fmt_args![
                    c"control_read_callback".as_ptr(),
                    (*c).name.as_deref(),
                    line
                ],
            );
            if *line as ::core::ffi::c_int == '\0' as i32 {
                (*c).flags |= CLIENT_EXIT as uint64_t;
                break;
            } else {
                let state = cmdq_new_state(
                    ::core::ptr::null_mut::<cmd_find_state>(),
                    ::core::ptr::null_mut::<key_event>(),
                    CMDQ_STATE_CONTROL,
                );
                cmd_parse_and_append(
                    line,
                    ::core::ptr::null_mut::<cmd_parse_input>(),
                    c,
                    &state,
                    &mut error,
                );
                if let Some(error) = error.take() {
                    cmdq_append(
                        c,
                        cmdq_get_callback1(
                            c"control_error".as_ptr(),
                            Some(control_error),
                            CmdqCallbackData::String(error),
                        ),
                    );
                }
            }
        }
    }
}
pub unsafe fn control_all_done(mut c: *mut client) -> ::core::ffi::c_int {
    unsafe {
        let mut cs: *mut control_state = control_state_of(c);
        if !(*cs).all_blocks.is_empty() {
            return 0 as ::core::ffi::c_int;
        }
        ((*cs).write_event.output_len() == 0 as size_t) as ::core::ffi::c_int
    }
}
unsafe fn control_flush_all_blocks(mut c: *mut client) {
    unsafe {
        let cs: *mut control_state = control_state_of(c);
        while let Some(cb) = (*cs).all_blocks.first() {
            if cb.size != 0 as size_t {
                break;
            }
            let line = cstr_ptr(&cb.line);
            log_debug(
                c"%s: %s: flushing line: %s".as_ptr(),
                fmt_args![
                    c"control_flush_all_blocks".as_ptr(),
                    (*c).name.as_deref(),
                    line
                ],
            );
            (*cs).write_event.write(line as *const u8, strlen(line));
            (*cs).write_event.write(b"\n\0".as_ptr(), 1 as size_t);
            (*cs).all_blocks.remove(0);
        }
    }
}
unsafe fn control_append_data(
    mut c: *mut client,
    mut cp: *mut control_pane,
    mut age: uint64_t,
    message: Option<Box<Buf>>,
    mut wp: *mut window_pane,
    mut size: size_t,
) -> Option<Box<Buf>> {
    unsafe {
        let mut new_data: *mut u_char = ::core::ptr::null_mut::<u_char>();
        let mut new_size: size_t = 0;
        let mut start: size_t = 0;
        let mut i: u_int = 0;
        let mut message = match message {
            Some(message) => message,
            None => {
                let mut message = Box::new(Buf::new());
                if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_PAUSEAFTER != 0 {
                    format_buf(
                        &mut message,
                        c"%%extended-output %%%u %llu : ".as_ptr(),
                        fmt_args![(*wp).id, age as ::core::ffi::c_ulonglong],
                    );
                } else {
                    format_buf(
                        &mut message,
                        c"%%output %%%u ".as_ptr(),
                        fmt_args![(*wp).id],
                    );
                }
                message
            }
        };
        let taken = window_pane_get_new_data(wp, &raw mut (*cp).offset);
        new_data = taken.0 as *mut u_char;
        new_size = taken.1;
        if new_size < size {
            fatalx(
                c"not enough data: %zu < %zu".as_ptr(),
                fmt_args![new_size, size],
            );
        }
        i = 0 as u_int;
        while (i as size_t) < size {
            if (*new_data.offset(i as isize) as ::core::ffi::c_int) < ' ' as i32
                || *new_data.offset(i as isize) as ::core::ffi::c_int == '\\' as i32
            {
                format_buf(
                    &mut message,
                    c"\\%03o".as_ptr(),
                    fmt_args![*new_data.offset(i as isize) as ::core::ffi::c_int],
                );
            } else {
                start = i as size_t;
                while (i.wrapping_add(1 as u_int) as size_t) < size
                    && *new_data.offset(i.wrapping_add(1 as u_int) as isize) as ::core::ffi::c_int
                        >= ' ' as i32
                    && *new_data.offset(i.wrapping_add(1 as u_int) as isize) as ::core::ffi::c_int
                        != '\\' as i32
                {
                    i = i.wrapping_add(1);
                }
                message.append(::core::slice::from_raw_parts(
                    new_data.add(start),
                    (i as size_t).wrapping_sub(start).wrapping_add(1 as size_t),
                ));
            }
            i = i.wrapping_add(1);
        }
        window_pane_update_used_data(wp, &raw mut (*cp).offset, size);
        Some(message)
    }
}
unsafe fn control_write_data(mut c: *mut client, mut message: Box<Buf>) {
    unsafe {
        let mut cs: *mut control_state = control_state_of(c);
        let data = message.as_slice();
        log_debug(
            c"%s: %s: %.*s".as_ptr(),
            fmt_args![
                c"control_write_data".as_ptr(),
                (*c).name.as_deref(),
                data.len() as ::core::ffi::c_int,
                data.as_ptr()
            ],
        );
        message.append(b"\n");
        (*cs).write_event.write_buffer(&mut message);
        drop(message);
    }
}
unsafe fn control_write_pending(
    mut c: *mut client,
    mut cp: *mut control_pane,
    mut limit: size_t,
) -> ::core::ffi::c_int {
    unsafe {
        let mut cs: *mut control_state = control_state_of(c);
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut message: Option<Box<Buf>> = None;
        let mut used: size_t = 0 as size_t;
        let mut size: size_t = 0;
        let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
        let mut age: uint64_t = 0;
        let mut t: uint64_t = get_timer();
        wp = control_window_pane(c, (*cp).pane);
        if wp.is_null() || (*wp).fd == -(1 as ::core::ffi::c_int) {
            for cb in ::core::mem::take(&mut (*cp).blocks) {
                control_free_block(cs, cb);
            }
            control_flush_all_blocks(c);
            return 0 as ::core::ffi::c_int;
        }
        while used != limit && !(*cp).blocks.is_empty() {
            if control_check_age(c, wp, cp) != 0 {
                message = None;
                break;
            } else {
                cb = (*cp).blocks[0];
                if (*cb).t < t {
                    age = t.wrapping_sub((*cb).t);
                } else {
                    age = 0 as uint64_t;
                }
                log_debug(
                    c"%s: %s: output block %zu (age %llu) for %%%u (used %zu/%zu)".as_ptr(),
                    fmt_args![
                        c"control_write_pending".as_ptr(),
                        (*c).name.as_deref(),
                        (*cb).size,
                        age as ::core::ffi::c_ulonglong,
                        (*cp).pane,
                        used,
                        limit
                    ],
                );
                size = (*cb).size;
                if size > limit.wrapping_sub(used) {
                    size = limit.wrapping_sub(used);
                }
                used = used.wrapping_add(size);
                message = control_append_data(c, cp, age, message, wp, size);
                (*cb).size = (*cb).size.wrapping_sub(size);
                if (*cb).size == 0 as size_t {
                    control_unlink_block(&raw mut (*cp).blocks, cb);
                    control_free_block(cs, cb);
                    if (*cs)
                        .all_blocks
                        .first()
                        .is_some_and(|block| block.size == 0 as size_t)
                    {
                        if !wp.is_null()
                            && let Some(message) = message.take()
                        {
                            control_write_data(c, message);
                        }
                        control_flush_all_blocks(c);
                    }
                }
            }
        }
        if let Some(message) = message {
            control_write_data(c, message);
        }
        !(*cp).blocks.is_empty() as ::core::ffi::c_int
    }
}
unsafe fn control_write_callback(mut c: *mut client) {
    unsafe {
        let mut cs: *mut control_state = control_state_of(c);
        let mut space: size_t = 0;
        let mut limit: size_t = 0;
        control_flush_all_blocks(c);
        while (*cs).write_event.output_len() < CONTROL_BUFFER_HIGH as size_t {
            if (*cs).pending_count == 0 as u_int {
                break;
            }
            space = (CONTROL_BUFFER_HIGH as size_t).wrapping_sub((*cs).write_event.output_len());
            log_debug(
                c"%s: %s: %zu bytes available, %u panes".as_ptr(),
                fmt_args![
                    c"control_write_callback".as_ptr(),
                    (*c).name.as_deref(),
                    space,
                    (*cs).pending_count
                ],
            );
            limit = space
                .wrapping_div((*cs).pending_count as size_t)
                .wrapping_div(3 as size_t);
            if limit < CONTROL_WRITE_MINIMUM as size_t {
                limit = CONTROL_WRITE_MINIMUM as size_t;
            }
            for cp in foreach_safe(&raw mut (*cs).pending_list) {
                if (*cs).write_event.output_len() >= CONTROL_BUFFER_HIGH as size_t {
                    break;
                }
                if !(control_write_pending(c, cp, limit) != 0) {
                    if let Some(at) = (*cs).pending_list.iter().position(|&waiting| waiting == cp) {
                        (*cs).pending_list.remove(at);
                    }
                    (*cp).pending_flag = 0 as ::core::ffi::c_int;
                    (*cs).pending_count = (*cs).pending_count.wrapping_sub(1);
                }
            }
        }
        if (*cs).write_event.output_len() == 0 as size_t {
            (*cs).write_event.disable(Interest::Write);
        }
    }
}
/// The control state a client carries, or null if it has none.
unsafe fn control_state_of(c: *mut client) -> *mut control_state {
    unsafe {
        (*c).control_state
            .as_deref_mut()
            .map_or(::core::ptr::null_mut::<control_state>(), |cs| &raw mut *cs)
    }
}
pub unsafe fn control_start(mut c: *mut client) {
    unsafe {
        let mut cs: *mut control_state = ::core::ptr::null_mut::<control_state>();
        if (*c).flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
            close((*c).out_fd);
            (*c).out_fd = -(1 as ::core::ffi::c_int);
        } else {
            setblocking((*c).out_fd, 0 as ::core::ffi::c_int);
        }
        setblocking((*c).fd, 0 as ::core::ffi::c_int);
        cs = &raw mut **(*c)
            .control_state
            .insert(Box::new(control_state::default()));
        let watching = client_ref_from_ptr(c).map(|held| held.downgrade());
        (*cs).read_event = Stream::new(
            (*c).fd,
            Some(on_client(&watching, |c| control_read_callback(c))),
            Some(on_client(&watching, |c| control_write_callback(c))),
            Some(on_client_error(&watching, |c| control_error_callback(c))),
        );
        if (*cs).read_event.is_none() {
            fatalx(c"out of memory".as_ptr(), fmt_args![]);
        }
        if (*c).flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
            (*cs).write_event = (*cs).read_event;
        } else {
            (*cs).write_event = Stream::new(
                (*c).out_fd,
                None,
                Some(on_client(&watching, |c| control_write_callback(c))),
                Some(on_client_error(&watching, |c| control_error_callback(c))),
            );
            if (*cs).write_event.is_none() {
                fatalx(c"out of memory".as_ptr(), fmt_args![]);
            }
        }
        (*cs)
            .write_event
            .set_write_watermark(CONTROL_BUFFER_LOW as size_t, 0 as size_t);
        if (*c).flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
            (*cs)
                .write_event
                .write(b"\x1BP1000p\0".as_ptr(), 7 as size_t);
            (*cs).write_event.enable(Interest::Write);
        }
    }
}
pub unsafe fn control_ready(mut c: *mut client) {
    unsafe {
        (*control_state_of(c)).read_event.enable(Interest::Read);
    }
}
pub unsafe fn control_discard(mut c: *mut client) {
    unsafe {
        let mut cs: *mut control_state = control_state_of(c);
        let panes: Vec<*mut control_pane> = (*cs)
            .panes
            .values()
            .map(|cp| cp.as_ref() as *const control_pane as *mut control_pane)
            .collect();
        for cp in panes {
            control_discard_pane(c, cp);
        }
        (*cs).read_event.disable(Interest::Read);
    }
}
pub unsafe fn control_stop(mut c: *mut client) {
    unsafe {
        let mut cs: *mut control_state = control_state_of(c);
        if cs.is_null() {
            return;
        }
        if !(*c).flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
            (*cs).write_event.free();
        }
        (*cs).read_event.free();
        (*cs).subs.clear();
        (*cs).subs_timer.disarm();
        control_reset_offsets(c);
        (*c).control_state = None;
    }
}
unsafe fn control_check_subs_session(
    mut c: *mut client,
    mut csub: *mut control_sub,
    ft: &mut format_tree,
) {
    unsafe {
        let mut s: *mut session = (*c).session;
        let value = format_expand(&mut *ft, CStr::from_ptr((*csub).format.as_ptr()));
        if (*csub).last.as_deref() == Some(value.as_c_str()) {
            return;
        }
        control_write(
            c,
            c"%%subscription-changed %s $%u - - - : %s".as_ptr(),
            fmt_args![(*csub).name.as_ptr(), session_id(s), value.as_ptr()],
        );
        (*csub).last = Some(value);
    }
}
unsafe fn control_check_subs_pane(mut c: *mut client, mut csub: *mut control_sub) {
    unsafe {
        let mut s: *mut session = (*c).session;
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        wp = window_pane_find_by_id((*csub).id);
        if wp.is_null() || (*wp).fd == -(1 as ::core::ffi::c_int) {
            return;
        }
        w = (*wp).window;
        for wl in winlinks_into(w) {
            if !((*wl).session() != s) {
                let mut ft =
                    format_create_defaults(::core::ptr::null_mut::<cmdq_item>(), c, s, wl, wp);
                let value = format_expand(&mut ft, CStr::from_ptr((*csub).format.as_ptr()));
                let key = ((*wp).id, (*wl).idx as u_int);
                let last = (*csub).panes.get(&key);
                if last.map(CString::as_c_str) != Some(value.as_c_str()) {
                    control_write(
                        c,
                        c"%%subscription-changed %s $%u @%u %u %%%u : %s".as_ptr(),
                        fmt_args![
                            (*csub).name.as_ptr(),
                            session_id(s),
                            (*w).id,
                            (*wl).idx,
                            (*wp).id,
                            value.as_ptr()
                        ],
                    );
                    (*csub).panes.insert(key, value);
                }
            }
        }
    }
}
unsafe fn control_check_subs_all_panes_one(
    mut c: *mut client,
    mut csub: *mut control_sub,
    ft: &mut format_tree,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) {
    unsafe {
        let mut s: *mut session = (*c).session;
        let mut w: *mut window = (*wl).window();
        let value = format_expand(&mut *ft, CStr::from_ptr((*csub).format.as_ptr()));
        let key = ((*wp).id, (*wl).idx as u_int);
        let last = (*csub).panes.get(&key);
        if last.map(CString::as_c_str) == Some(value.as_c_str()) {
            return;
        }
        control_write(
            c,
            c"%%subscription-changed %s $%u @%u %u %%%u : %s".as_ptr(),
            fmt_args![
                (*csub).name.as_ptr(),
                session_id(s),
                (*w).id,
                (*wl).idx,
                (*wp).id,
                value.as_ptr()
            ],
        );
        (*csub).panes.insert(key, value);
    }
}
unsafe fn control_check_subs_window(mut c: *mut client, mut csub: *mut control_sub) {
    unsafe {
        let mut s: *mut session = (*c).session;
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        w = window_find_by_id((*csub).id);
        if w.is_null() {
            return;
        }
        for wl in winlinks_into(w) {
            if !((*wl).session() != s) {
                let mut ft = format_create_defaults(
                    ::core::ptr::null_mut::<cmdq_item>(),
                    c,
                    s,
                    wl,
                    ::core::ptr::null_mut::<window_pane>(),
                );
                let value = format_expand(&mut ft, CStr::from_ptr((*csub).format.as_ptr()));
                let key = ((*w).id, (*wl).idx as u_int);
                let last = (*csub).windows.get(&key);
                if last.map(CString::as_c_str) != Some(value.as_c_str()) {
                    control_write(
                        c,
                        c"%%subscription-changed %s $%u @%u %u - : %s".as_ptr(),
                        fmt_args![
                            (*csub).name.as_ptr(),
                            session_id(s),
                            (*w).id,
                            (*wl).idx,
                            value.as_ptr()
                        ],
                    );
                    (*csub).windows.insert(key, value);
                }
            }
        }
    }
}
unsafe fn control_check_subs_all_windows_one(
    mut c: *mut client,
    mut csub: *mut control_sub,
    ft: &mut format_tree,
    mut wl: *mut winlink,
) {
    unsafe {
        let mut s: *mut session = (*c).session;
        let mut w: *mut window = (*wl).window();
        let value = format_expand(&mut *ft, CStr::from_ptr((*csub).format.as_ptr()));
        let key = ((*w).id, (*wl).idx as u_int);
        let last = (*csub).windows.get(&key);
        if last.map(CString::as_c_str) == Some(value.as_c_str()) {
            return;
        }
        control_write(
            c,
            c"%%subscription-changed %s $%u @%u %u - : %s".as_ptr(),
            fmt_args![
                (*csub).name.as_ptr(),
                session_id(s),
                (*w).id,
                (*wl).idx,
                value.as_ptr()
            ],
        );
        (*csub).windows.insert(key, value);
    }
}
unsafe fn control_check_subs_timer(c: *mut client) {
    unsafe {
        let mut cs: *mut control_state = control_state_of(c);
        let s: *mut session = (*c).session;
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut tv = timeval::from_secs(1 as __time_t);
        let mut have_session: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut have_all_panes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut have_all_windows: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        log_debug(
            c"%s: timer fired".as_ptr(),
            fmt_args![c"control_check_subs_timer".as_ptr()],
        );
        (*cs).subs_timer.arm(tv);
        if s.is_null() {
            return;
        }
        for csub in (*cs).subs.values_mut().map(|csub| &raw mut **csub) {
            match (*csub).type_0 {
                CONTROL_SUB_SESSION => {
                    have_session = 1 as ::core::ffi::c_int;
                }
                CONTROL_SUB_ALL_PANES => {
                    have_all_panes = 1 as ::core::ffi::c_int;
                }
                CONTROL_SUB_ALL_WINDOWS => {
                    have_all_windows = 1 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        if have_session != 0 {
            let mut ft = format_create_defaults(
                ::core::ptr::null_mut::<cmdq_item>(),
                c,
                s,
                ::core::ptr::null_mut::<winlink>(),
                ::core::ptr::null_mut::<window_pane>(),
            );
            for csub in (*cs)
                .subs
                .values_mut()
                .map(|csub| &raw mut **csub)
                .collect::<Vec<_>>()
            {
                if (*csub).type_0 as ::core::ffi::c_uint
                    == CONTROL_SUB_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    control_check_subs_session(c, csub, &mut ft);
                }
            }
        }
        for csub in (*cs)
            .subs
            .values_mut()
            .map(|csub| &raw mut **csub)
            .collect::<Vec<_>>()
        {
            match (*csub).type_0 {
                CONTROL_SUB_PANE => {
                    control_check_subs_pane(c, csub);
                }
                CONTROL_SUB_WINDOW => {
                    control_check_subs_window(c, csub);
                }
                _ => {}
            }
        }
        if have_all_panes != 0 {
            wl = winlinks_first(&raw mut (*s).windows);
            while !wl.is_null() {
                let cw: *mut window = (*wl).window();
                wp = window_panes_first(cw);
                while !wp.is_null() {
                    let mut ft =
                        format_create_defaults(::core::ptr::null_mut::<cmdq_item>(), c, s, wl, wp);
                    for csub in (*cs)
                        .subs
                        .values_mut()
                        .map(|csub| &raw mut **csub)
                        .collect::<Vec<_>>()
                    {
                        if !((*csub).type_0 as ::core::ffi::c_uint
                            != CONTROL_SUB_ALL_PANES as ::core::ffi::c_int as ::core::ffi::c_uint)
                        {
                            control_check_subs_all_panes_one(c, csub, &mut ft, wl, wp);
                        }
                    }
                    wp = window_panes_next(cw, wp);
                }
                wl = winlinks_after(wl);
            }
        }
        if have_all_windows != 0 {
            wl = winlinks_first(&raw mut (*s).windows);
            while !wl.is_null() {
                let mut ft = format_create_defaults(
                    ::core::ptr::null_mut::<cmdq_item>(),
                    c,
                    s,
                    wl,
                    ::core::ptr::null_mut::<window_pane>(),
                );
                for csub in (*cs)
                    .subs
                    .values_mut()
                    .map(|csub| &raw mut **csub)
                    .collect::<Vec<_>>()
                {
                    if !((*csub).type_0 as ::core::ffi::c_uint
                        != CONTROL_SUB_ALL_WINDOWS as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        control_check_subs_all_windows_one(c, csub, &mut ft, wl);
                    }
                }
                wl = winlinks_after(wl);
            }
        }
    }
}
pub unsafe fn control_add_sub(
    mut c: *mut client,
    mut name: *const ::core::ffi::c_char,
    mut type_0: control_sub_type,
    mut id: ::core::ffi::c_int,
    mut format: *const ::core::ffi::c_char,
) {
    unsafe {
        let mut cs: *mut control_state = control_state_of(c);
        let mut tv = timeval::from_secs(1 as __time_t);
        (*cs).subs.remove(CStr::from_ptr(name));
        let csub = Box::new(control_sub {
            name: CStr::from_ptr(name).to_owned(),
            format: CStr::from_ptr(format).to_owned(),
            type_0,
            id: id as u_int,
            last: None,
            panes: control_sub_panes::new(),
            windows: control_sub_windows::new(),
        });
        (*cs).subs.insert(csub.name.clone(), csub);
        if !(*cs).subs_timer.is_set() {
            (*cs)
                .subs_timer
                .set_callback(move || control_check_subs_timer(c));
        }
        if !(*cs).subs_timer.is_armed() {
            (*cs).subs_timer.arm(tv);
        }
    }
}
pub unsafe fn control_remove_sub(mut c: *mut client, mut name: *const ::core::ffi::c_char) {
    unsafe {
        let mut cs: *mut control_state = control_state_of(c);
        (*cs).subs.remove(CStr::from_ptr(name));
        if (*cs).subs.is_empty() {
            (*cs).subs_timer.disarm();
        }
    }
}

use crate::src::arguments::args_escape_cstring;
use crate::src::cfg::cfg_files;
use crate::src::cmd::queue::{
    cmdq_get_client, cmdq_get_event, cmdq_get_target, cmdq_get_target_client, cmdq_merge_formats,
    cmdq_print,
};
use crate::src::cmd::{cmd_mouse_at, cmd_mouse_pane, cmd_stringify_argv_cstring};
use crate::src::compat::strtonum::strtonum;
use crate::src::environ::{environ_find, environ_iter};
use crate::src::ffi::libc::{
    __ctype_b_loc, __xpg_basename, ctime_r, dirname, fnmatch, free, gethostname, getpid, getpwuid,
    getuid, localtime_r, memcmp, memcpy, memset, strcasecmp, strchr, strcmp, strcspn, strftime,
    strlcat, strlen, strstr, strtod, time,
};
use crate::src::ffi::libm::{fabs, fmod};
use crate::src::ffi::regex::RegexStorage;
use crate::src::format::bytes::format_message_with;
use crate::src::format::bytes::{write_cstr, write_cstr_n};
use crate::src::format_draw::{format_trim_left_bytes, format_trim_right_bytes, format_width};
use crate::src::fuzzy::fuzzy_match_owned;
use crate::src::grid::view::grid_view_get_cell;
use crate::src::grid::{grid_get_cell, grid_get_line, grid_line_length, grid_peek_line};
use crate::src::hyperlinks::hyperlinks_get;
use crate::src::job::{job_free, job_get_event, job_run};
use crate::src::layout::custom::layout_dump_owned;
use crate::src::layout::layout_add_horizontal_border;
use crate::src::log::{fatalx, log_cstr, log_debug, log_get_level};
use crate::src::names::parse_window_name_cstring;
use crate::src::options::options_table_entry;
use crate::src::options::{
    options_array_item_key, options_get,
    options_get_number, options_get_string, options_is_array, options_name, options_parse_owned, options_to_cstring,
};
use crate::src::osdep_linux::{osdep_get_cwd, osdep_get_name_cstring};
use crate::src::paste::{
    paste_buffer_created, paste_buffer_data, paste_buffer_name, paste_get_top,
    paste_make_sample_cstring,
};
use crate::src::proc::proc_get_peer_uid;
use crate::src::reactor::{
    evbuffer_add, evbuffer_get_length, evbuffer_new, evbuffer_pullup,
    evbuffer_readline, event_add, event_initialized, event_pending, event_set,
};
use crate::src::regsub::regsub_cstring;
use crate::src::server::clients;
use crate::src::server::{marked_pane, server_check_marked};
use crate::src::server_client::{
    server_client_get_cwd, server_client_get_flags, server_client_get_key_table,
};
use crate::src::server_fn::server_status_client;
use crate::src::session::{
    next_session_id, session_alive, session_group_attached_count, session_group_contains,
    session_group_count, session_groups_minmax, session_groups_next, sessions_minmax,
    sessions_next,
};
use crate::src::session::{session_groups, sessions};
use crate::src::shared::session::session_group;
use crate::src::sort::{
    sort_get_clients, sort_get_panes_window, sort_get_sessions, sort_get_winlinks_session,
};
use crate::src::status::status_get_range;
use crate::src::style::colour::{
    colour_force_rgb, colour_format, colour_format_escape_for_client, colour_parse_cstr,
};
use crate::src::text::utf8::{utf8_cstrhas, utf8_pad_cstring, utf8_set, utf8_tocstr_cstring};
use crate::src::tmux::{
    get_timer, getversion, global_environ, global_options, global_s_options, global_w_options,
    sig2name, socket_path, start_time,
};
use crate::src::tty::{tty_default_colours, tty_window_offset};
use crate::src::tty_features::{tty_feature_present, tty_get_features};
use crate::src::tty_term::{tty_term_has_name, tty_term_number};
use crate::src::window::{
    window_count_panes, window_get_pane_status, window_pane_get_pane_status, window_pane_index,
    window_pane_is_floating, window_pane_mode, window_pane_printable_flags,
    window_pane_scrollbar_reserve, window_pane_search, window_pane_zindex, window_printable_flags,
    winlink_count, winlink_find_by_window, winlinks_minmax, winlinks_next,
};
use crate::src::window_buffer::window_buffer_mode;
use crate::src::window_client::window_client_mode;
use crate::src::window_copy::{
    window_copy_get_hyperlink_cstring, window_copy_get_line_cstring, window_copy_get_word_cstring,
};
use crate::src::window_tree::window_tree_mode;
use std::ffi::{CStr, CString};

use crate::src::shared::abi::NULL_0;
use crate::src::shared::abi::*;
use crate::src::shared::abi::{__uid_t, ssize_t, uid_t};
use crate::src::shared::account::passwd;
use crate::src::shared::client::client;
use crate::src::shared::client::{
    CLIENT_CONTROL, CLIENT_CONTROL_NEWLAYOUTS, CLIENT_READONLY, CLIENT_REDRAWSTATUS,
    CLIENT_UNATTACHEDFLAGS, CLIENT_UTF8,
};
use crate::src::shared::command::{cmd_find_state, cmdq_item};
use crate::src::shared::ctype::_ISpunct;
use crate::src::shared::environment::ENVIRON_HIDDEN;
use crate::src::shared::environment::{environ, environ_entry};
use crate::src::shared::event::EV_TIMEOUT;
use crate::src::shared::event::*;
pub use crate::src::shared::format::{
    format_entry, format_entry_tree, format_job, format_job_tree, format_tree, format_type,
};
use crate::src::shared::format::{
    FORMAT_BASENAME, FORMAT_CHARACTER, FORMAT_CLIENTS, FORMAT_CLIENT_ENVIRON,
    FORMAT_CLIENT_TERMCAP, FORMAT_CLIENT_TERMFEAT, FORMAT_COLOUR, FORMAT_COLOUR_ESC_BG,
    FORMAT_COLOUR_ESC_FG, FORMAT_CYCLE, FORMAT_CYCLE_PERIOD, FORMAT_DIFFERENCE, FORMAT_DIRNAME,
    FORMAT_ENVIRON, FORMAT_EXPAND, FORMAT_EXPANDTIME, FORMAT_EXPAND_NOCYCLE, FORMAT_EXPAND_NOJOBS,
    FORMAT_EXPAND_TIME, FORMAT_FORCE, FORMAT_LENGTH, FORMAT_LITERAL, FORMAT_LOOP_LIMIT,
    FORMAT_MAX_PRECISION, FORMAT_MAX_REPEAT, FORMAT_MAX_WIDTH, FORMAT_NOJOBS, FORMAT_NONE,
    FORMAT_NOT, FORMAT_NOT_NOT, FORMAT_OPTIONS, FORMAT_PANE, FORMAT_PANES, FORMAT_PRETTY,
    FORMAT_QUOTE_ARGUMENTS, FORMAT_QUOTE_SHELL, FORMAT_QUOTE_SHELL_SQ, FORMAT_QUOTE_STYLE,
    FORMAT_RELATIVE, FORMAT_REPEAT, FORMAT_SESSIONS, FORMAT_SESSION_NAME, FORMAT_STATUS,
    FORMAT_TIMESTRING, FORMAT_TIME_LIMIT, FORMAT_TIME_LOOP_CHECK, FORMAT_VERBOSE, FORMAT_WIDTH,
    FORMAT_WINDOW, FORMAT_WINDOWS, FORMAT_WINDOW_NAME,
};
use crate::src::shared::grid::*;
use crate::src::shared::job::JOB_NOWAIT;
use crate::src::shared::job::{job, job_update_callback, JobCompletion};
use crate::src::shared::key::key_event;
use crate::src::shared::layout::layout_cell;
use crate::src::shared::layout::*;
use crate::src::shared::mouse::mouse_event;
use crate::src::shared::options::OPTIONS_TABLE_IS_HOOK;
use crate::src::shared::options::*;
use crate::src::shared::options::{options, options_array_item, options_entry};
use crate::src::shared::pane::window_pane;
use crate::src::shared::pane::{
    PANE_CMDRUNNING, PANE_INPUTOFF, PANE_MINIMUM, PANE_SCROLLBARS_ALWAYS, PANE_STATUSDRAWN,
    PANE_STATUSREADY, PANE_STATUS_BOTTOM, PANE_STATUS_TOP, PANE_UNSEENCHANGES, PANE_ZOOMED,
};
use crate::src::shared::paste::PasteBufferRef;
use crate::src::shared::posix_io::FNM_CASEFOLD;
use libc::{REG_EXTENDED, REG_ICASE};
use crate::src::shared::screen::{
    screen, ALL_MOUSE_MODES, EXTENDED_KEY_MODES, MODE_BRACKETPASTE, MODE_CURSOR,
    MODE_CURSOR_BLINKING, MODE_CURSOR_BLINKING_SET, MODE_CURSOR_VERY_VISIBLE, MODE_FOCUSON,
    MODE_INSERT, MODE_KCURSOR, MODE_KEYS_EXTENDED, MODE_KEYS_EXTENDED_2, MODE_KKEYPAD,
    MODE_MOUSE_ALL, MODE_MOUSE_BUTTON, MODE_MOUSE_SGR, MODE_MOUSE_STANDARD, MODE_MOUSE_UTF8,
    MODE_ORIGIN, MODE_SYNC, MODE_THEME_UPDATES, MODE_WRAP,
};
use crate::src::shared::session::session;
use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
use crate::src::shared::style::*;
use crate::src::shared::time::tm;
use crate::src::shared::tree::{RB_INF, RB_NEGINF};
use crate::src::shared::tty::tty_term;
use crate::src::shared::tty::*;
use crate::src::shared::tty::{TERM_256COLOURS, TERM_RGBCOLOURS, TTY_STARTED};
use crate::src::shared::window::{window, window_mode_entry, winlink};
use crate::src::shared::window::{
    WINDOW_PANE_NO_MODE, WINDOW_SIZE_MANUAL, WINDOW_ZOOMED, WINLINK_ACTIVITY, WINLINK_ALERTFLAGS,
    WINLINK_BELL, WINLINK_SILENCE,
};

/*
 * This module is the stable facade for format handling.  The private
 * implementation modules below keep the translated C ABI and the existing
 * `crate::src::format::*` paths in one place while separating responsibilities:
 * tree stores the red-black trees and format-tree CRUD, jobs owns the shared
 * format-job cache and its callbacks, callbacks owns the default callback
 * table, and expression owns parsing and evaluation.  The facade remains the
 * dependency boundary for generated shared types, FFI declarations, and the
 * small logging/state helpers used by more than one group.
 */
pub mod bytes;
mod tree;
pub use tree::format_add_owned_cb;
use tree::*;
pub use tree::{
    format_add, format_add_cstr, format_add_tv, format_create, format_create_with_client, format_create_owned, format_owner_ptr, format_each, format_free,
    format_get_pane, format_log_debug, format_merge,
};
mod jobs;
use jobs::*;
pub use jobs::{format_lost_client, format_tidy_jobs};
mod callbacks;
use callbacks::*;
mod expression;
pub use expression::format_expand_cstring;
pub(crate) use expression::format_pretty_time_cstring;
use expression::*;
pub(crate) use expression::{
    format_expand_time_cstring, format_single_cstring, format_single_from_state_cstring,
    format_single_from_target_cstring,
};
pub use expression::{format_skip, format_true};

pub struct format_modifier {
    pub modifier: [::core::ffi::c_char; 3],
    pub size: u_int,
    pub argv: Vec<std::ffi::CString>,
}

impl format_modifier {
    fn argc(&self) -> ::core::ffi::c_int {
        self.argv
            .len()
            .try_into()
            .expect("modifier argument count fits c_int")
    }

    fn arg(&self, index: usize) -> *const ::core::ffi::c_char {
        self.argv[index].as_ptr()
    }
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct format_expand_state {
    pub ft: *mut format_tree,
    pub loop_0: u_int,
    pub start_time: uint64_t,
    pub flags: ::core::ffi::c_int,
    pub time: time_t,
    pub tm: tm,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_43 {
    pub mode: ::core::ffi::c_int,
    pub number: ::core::ffi::c_int,
}
pub const LESS_THAN_EQUAL: C2RustUnnamed_44 = 10;
pub const LESS_THAN: C2RustUnnamed_44 = 9;
pub const GREATER_THAN_EQUAL: C2RustUnnamed_44 = 8;
pub const GREATER_THAN: C2RustUnnamed_44 = 7;
pub const NOT_EQUAL: C2RustUnnamed_44 = 6;
pub const EQUAL: C2RustUnnamed_44 = 5;
pub const MODULUS: C2RustUnnamed_44 = 4;
pub const DIVIDE: C2RustUnnamed_44 = 3;
pub const MULTIPLY: C2RustUnnamed_44 = 2;
pub const SUBTRACT: C2RustUnnamed_44 = 1;
pub const ADD: C2RustUnnamed_44 = 0;
pub type C2RustUnnamed_44 = ::core::ffi::c_uint;

pub const REG_NOSUB: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;

static mut sort_crit: sort_criteria = sort_criteria {
    order: SORT_ACTIVITY,
    reversed: 0,
    order_seq: &[],
};
static mut format_upper: [*const ::core::ffi::c_char; 26] = [
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"pane_id\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"window_flags\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"host\0" as *const u8 as *const ::core::ffi::c_char,
    b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"pane_index\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"session_name\0" as *const u8 as *const ::core::ffi::c_char,
    b"pane_title\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"window_name\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut format_lower: [*const ::core::ffi::c_char; 26] = [
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"host_short\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
];
#[inline]
unsafe fn format_logging(mut ft: *mut format_tree) -> ::core::ffi::c_int {
    return (log_get_level() != 0 as ::core::ffi::c_int || (*ft).flags & FORMAT_VERBOSE != 0)
        as ::core::ffi::c_int;
}
unsafe fn format_log1(
    mut es: *mut format_expand_state,
    mut from: *const ::core::ffi::c_char,
    write: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) {
    let mut ft: *mut format_tree = (*es).ft;
    if format_logging(ft) == 0 {
        return;
    }
    let s = format_message_with(write);
    log_debug(format_args!(
        "{}: {}",
        log_cstr((from) as *const _),
        log_cstr((s.as_ptr()) as *const _)
    ));
    if let Some(item) = (*ft).item.upgrade().filter(|_| (*ft).flags & FORMAT_VERBOSE != 0) {
        cmdq_print(item.get(), |out| {
            out.write_all(b"#")?;
            write_cstr_n(out, c"          ".as_ptr(), ((*es).loop_0) as i32)?;
            write_cstr(out, s.as_ptr())
        });
    }
}
unsafe fn format_copy_state(
    mut to: *mut format_expand_state,
    mut from: *mut format_expand_state,
    mut flags: ::core::ffi::c_int,
) {
    (*to).ft = (*from).ft;
    (*to).loop_0 = (*from).loop_0;
    (*to).time = (*from).time;
    memcpy(
        &raw mut (*to).tm as *mut ::core::ffi::c_void,
        &raw mut (*from).tm as *const ::core::ffi::c_void,
        ::core::mem::size_of::<tm>() as size_t,
    );
    (*to).flags = (*from).flags | flags;
    (*to).start_time = (*from).start_time;
}
pub unsafe fn format_create_defaults(
    mut item: *mut cmdq_item,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) -> *mut format_tree {
    let queue_client = cmdq_get_client(item);
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    if !item.is_null() {
        ft = format_create_with_client(
            queue_client.as_ref(),
            item,
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        );
    } else {
        ft = format_create(
            ::core::ptr::null_mut::<client>(),
            item,
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        );
    }
    format_defaults(ft, c, s, wl, wp);
    return ft;
}
pub unsafe fn format_create_from_state(
    mut item: *mut cmdq_item,
    mut c: *mut client,
    fs: &cmd_find_state,
) -> *mut format_tree {
    return format_create_defaults(item, c, fs.s_ptr(), fs.wl_ptr(), fs.wp_ptr());
}
pub unsafe fn format_create_from_target(mut item: *mut cmdq_item) -> *mut format_tree {
    let tc_owner = cmdq_get_target_client(item);
    let mut tc: *mut client = tc_owner.as_ref().map_or(std::ptr::null_mut(), crate::src::shared::rc::as_ptr);
    return format_create_from_state(item, tc, &*crate::src::cmd::queue::cmdq_get_target_mut(&mut *item));
}
pub unsafe fn format_defaults(
    mut ft: *mut format_tree,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) {
    if !c.is_null() && !(*c).name.is_none() {
        log_debug(format_args!(
            "{}: c={}",
            "format_defaults",
            log_cstr(
                (((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()))
                    as *const _
            )
        ));
    } else {
        log_debug(format_args!("{}: c=none", "format_defaults"));
    }
    if !s.is_null() {
        log_debug(format_args!(
            "{}: s=${}",
            "format_defaults",
            ((*s).id) as u32
        ));
    } else {
        log_debug(format_args!("{}: s=none", "format_defaults"));
    }
    if !wl.is_null() {
        log_debug(format_args!(
            "{}: wl={}",
            "format_defaults",
            ((*wl).idx) as u32
        ));
    } else {
        log_debug(format_args!("{}: wl=none", "format_defaults"));
    }
    if !wp.is_null() {
        log_debug(format_args!(
            "{}: wp=%{}",
            "format_defaults",
            ((*wp).id) as u32
        ));
    } else {
        log_debug(format_args!("{}: wp=none", "format_defaults"));
    }
    if !c.is_null() && !s.is_null() && (*c).session_ptr() != s {
        log_debug(format_args!(
            "{}: session does not match",
            "format_defaults"
        ));
    }
    if !wp.is_null() {
        (*ft).type_0 = FORMAT_TYPE_PANE;
    } else if !wl.is_null() {
        (*ft).type_0 = FORMAT_TYPE_WINDOW;
    } else if !s.is_null() {
        (*ft).type_0 = FORMAT_TYPE_SESSION;
    } else {
        (*ft).type_0 = FORMAT_TYPE_UNKNOWN;
    }
    if s.is_null() && !c.is_null() {
        s = (*c).session_ptr();
    }
    if wl.is_null() && !s.is_null() {
        wl = (*s).curw_ptr();
    }
    if wp.is_null() && !wl.is_null() {
        wp = (*(*wl).window_ptr()).active_ptr();
    }
    if !c.is_null() {
        format_defaults_client(ft, c);
    }
    if !s.is_null() {
        format_defaults_session(ft, s);
    }
    if !wl.is_null() {
        format_defaults_winlink(ft, wl);
    }
    if !wp.is_null() {
        format_defaults_pane(ft, wp);
    }
    if let Some(pb) = paste_get_top(None) {
        format_defaults_paste_buffer(&mut *ft, &pb);
    }
}
unsafe fn format_defaults_session(mut ft: *mut format_tree, mut s: *mut session) {
    (*ft).s = (*s).observer.clone();
}
unsafe fn format_defaults_client(mut ft: *mut format_tree, mut c: *mut client) {
    if (*ft).s.upgrade().is_none() {
    (*ft).s = (*c).session.clone();
    }
    (*ft).c = (*c).observer.clone();
}
pub unsafe fn format_defaults_window(mut ft: *mut format_tree, mut w: *mut window) {
    (*ft).w = w.as_ref().map_or_else(std::rc::Weak::new, |window| window.observer.clone());
}
unsafe fn format_defaults_winlink(mut ft: *mut format_tree, mut wl: *mut winlink) {
    if (*ft).w.upgrade().is_none() {
        format_defaults_window(ft, (*wl).window_ptr());
    }
    (*ft).wl = (*wl).observer.clone();
}
pub unsafe fn format_defaults_pane(mut ft: *mut format_tree, mut wp: *mut window_pane) {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    if (*ft).w.upgrade().is_none() {
        format_defaults_window(ft, (*wp).window_ptr());
    }
    (*ft).wp = (*wp).observer.clone();
    wme = (*wp).modes.active_ptr();
    if !wme.is_null() && (*(*wme).mode).formats.is_some() {
        (*(*wme).mode).formats.expect("non-null function pointer")(wme, ft);
    }
}
pub fn format_defaults_paste_buffer(ft: &mut format_tree, pb: &PasteBufferRef) {
    ft.pb = Some(pb.clone());
}
fn format_is_word_separator(ws: &CStr, gc: &grid_cell) -> bool {
    if utf8_cstrhas(ws, &gc.data) {
        return true;
    }
    if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
        return true;
    }
    gc.data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int && gc.data.data[0] == b' '
}
pub unsafe fn format_grid_word(gd: &grid, x: u_int, y: u_int) -> Option<CString> {
    format_grid_word_cstring(gd, x, y)
}
pub(crate) unsafe fn format_grid_word_cstring(
    gd: &grid,
    mut x: u_int,
    mut y: u_int,
) -> Option<CString> {
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let ws: &CStr;
    let mut ud: Vec<utf8_data> = Vec::new();
    let mut end: u_int = 0;
    let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut s = None;
    ws = CStr::from_ptr(options_get_string(
        global_s_options,
        b"word-separators\0" as *const u8 as *const ::core::ffi::c_char,
    ));
    loop {
        grid_get_cell(gd, x, y, &mut gc);
        if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0
            && format_is_word_separator(ws, &gc)
        {
            found = 1 as ::core::ffi::c_int;
            break;
        } else {
            if x == 0 as u_int {
                if y == 0 as u_int {
                    break;
                }
                let gl = grid_peek_line(gd, y.wrapping_sub(1 as u_int))?;
                if !(gl.flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED != 0 {
                    break;
                }
                y = y.wrapping_sub(1);
                x = grid_line_length(gd, y);
                if x == 0 as u_int {
                    break;
                }
            }
            x = x.wrapping_sub(1);
        }
    }
    loop {
        if found != 0 {
            end = grid_line_length(gd, y);
            if end == 0 as u_int || x == end.wrapping_sub(1 as u_int) {
                if y == gd.hsize.wrapping_add(gd.sy).wrapping_sub(1 as u_int) {
                    break;
                }
                let gl = grid_peek_line(gd, y)?;
                if !(gl.flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED != 0 {
                    break;
                }
                y = y.wrapping_add(1);
                x = 0 as u_int;
            } else {
                x = x.wrapping_add(1);
            }
        }
        found = 1 as ::core::ffi::c_int;
        grid_get_cell(gd, x, y, &mut gc);
        if gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
            continue;
        }
        if format_is_word_separator(ws, &gc) {
            break;
        }
        ud.push(gc.data);
    }
    if !ud.is_empty() {
        ud.push(utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        });
        s = Some(utf8_tocstr_cstring(&ud));
    }
    return s;
}
pub unsafe fn format_grid_line(gd: &grid, y: u_int) -> Option<CString> {
    format_grid_line_cstring(gd, y)
}
pub(crate) unsafe fn format_grid_line_cstring(gd: &grid, mut y: u_int) -> Option<CString> {
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut ud: Vec<utf8_data> = Vec::new();
    let mut x: u_int = 0;
    let mut s = None;
    x = 0 as u_int;
    while x < grid_line_length(gd, y) {
        grid_get_cell(gd, x, y, &mut gc);
        if !(gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0) {
            if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                let mut tab = gc.data;
                utf8_set(&mut tab, '\t' as i32 as u_char);
                ud.push(tab);
            } else {
                ud.push(gc.data);
            }
        }
        x = x.wrapping_add(1);
    }
    if !ud.is_empty() {
        ud.push(utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        });
        s = Some(utf8_tocstr_cstring(&ud));
        drop(ud);
    }
    return s;
}
pub(crate) unsafe fn format_grid_hyperlink_cstring(
    gd: &grid,
    mut x: u_int,
    mut y: u_int,
    s: &screen,
) -> Option<CString> {
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    loop {
        grid_get_cell(gd, x, y, &mut gc);
        if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
            break;
        }
        if x == 0 as u_int {
            return None;
        }
        x = x.wrapping_sub(1);
    }
    if s.hyperlinks.is_none() || gc.link == 0 as u_int {
        return None;
    }
    hyperlinks_get(s.hyperlinks.as_ref()?, gc.link).map(|link| link.uri.clone())
}

pub const FORMAT_TYPE_PANE: format_type = 3;

pub const FORMAT_TYPE_WINDOW: format_type = 2;

pub const FORMAT_TYPE_SESSION: format_type = 1;

pub const FORMAT_TYPE_UNKNOWN: format_type = 0;

use super::draw::{format_trim_left, format_trim_right, format_width};
use crate::arguments::args_escape;
use crate::cfg::cfg_files;
use crate::cmd::CmdqItemWeak;
use crate::cmd::cmd_stringify_argv;
use crate::cmd::{cmd_mouse_at, cmd_mouse_pane};
use crate::cmd::{
    cmdq_get_client, cmdq_get_event, cmdq_get_target, cmdq_get_target_client, cmdq_merge_formats,
    cmdq_print,
};
use crate::compat::strtonum;
use crate::environ::{environ_entry_value, environ_find, environ_t};
use crate::ffi::{
    __ctype_b_loc, __xpg_basename, ctime_r, dirname, fabs, fmod, fnmatch, gethostname, getpid,
    getpwuid, getuid, localtime_r, regcomp, regexec, regfree, strchr, strcmp, strcspn, strftime,
    strlen, strstr, strtod, time,
};
use crate::fmt_args;
use crate::fmt_engine::{FmtArg, format_alloc, format_buf};
use crate::grid::grid_view_get_cell;
use crate::grid::hyperlinks_get;
use crate::grid::{
    grid_default_cell, grid_get_cell, grid_get_line, grid_line_length, grid_peek_line,
};
use crate::job::{job_find_by_id, job_free, job_get_data, job_get_event, job_id, job_run};
use crate::key_bindings::key_table_name;
use crate::layout::layout_dump;
use crate::log::{log_debug, log_get_level};
use crate::modes::{window_copy_get_hyperlink, window_copy_get_line, window_copy_get_word};
use crate::names::parse_window_name;
use crate::options::{
    options_first, options_get_number, options_get_string, options_name, options_next,
    options_parse_get, options_to_string,
};
use crate::osdep_linux::{osdep_get_cwd, osdep_get_name};
use crate::paste::{
    paste_buffer_created, paste_buffer_data, paste_buffer_name, paste_get_top, paste_make_sample,
};
use crate::proc::proc_get_peer_uid;
use crate::regsub::regsub;
use crate::screen::screen_grid;
use crate::screen::screen_grid_ptr;
use crate::server::client_get_last_session;
use crate::server::client_walk;
use crate::server::marked_pane;
use crate::server::server_check_marked;
use crate::server::server_status_client;
use crate::server::{
    client_ref_from_ptr, server_client_get_cwd, server_client_get_flags,
    server_client_get_key_table,
};
use crate::session::session_get_curw;
use crate::session::{
    group_walk, session_activity_time, session_attached, session_cwd, session_environ, session_id,
    session_name, session_options,
};
use crate::session::{
    next_session_id, session_alive, session_group_attached_count, session_group_contains,
    session_group_count, session_group_name, session_groups_after, session_groups_first,
    session_owners,
};
use crate::sort::{
    sort_get_clients, sort_get_panes_window, sort_get_sessions, sort_get_winlinks_session,
};
use crate::status::status_get_range;
use crate::style::{colour_force_rgb, colour_fromstring, colour_tostring};
use crate::terminfo::tty_get_features;
use crate::text::{utf8_cstrhas, utf8_padcstr, utf8_rpadcstr, utf8_set, utf8_vec_tocstr};
use crate::tmux::{get_timer, getversion, sig2name};
use crate::tmux::{
    global_environ, global_options, global_s_options, global_w_options, socket_path, start_time,
};
use crate::tree::GlobalTree;
use crate::tty::{tty_default_colours, tty_window_offset};
pub use crate::types::*;
use crate::window::PaneStack;
use crate::window::window_get_active;
use crate::window::window_pane_current_mode;
use crate::window::winlinks_into;
use crate::window::{
    window_count_panes, window_pane_index, window_pane_is_floating, window_pane_mode,
    window_pane_printable_flags, window_pane_search, window_pane_stack_first, window_pane_zindex,
    window_printable_flags, winlink_count, winlink_find_by_window, winlinks_after, winlinks_first,
    winlinks_last,
};
use crate::xmalloc::xasprintf;
use ::core::ffi::CStr;
use ::std::ffi::CString;
use ::std::sync::OnceLock;
pub type ctype_mask = ::core::ffi::c_uint;
pub const _ISalnum: ctype_mask = 8;
pub const _ISpunct: ctype_mask = 4;
pub const _IScntrl: ctype_mask = 2;
pub const _ISblank: ctype_mask = 1;
pub const _ISgraph: ctype_mask = 32768;
pub const _ISprint: ctype_mask = 16384;
pub const _ISspace: ctype_mask = 8192;
pub const _ISxdigit: ctype_mask = 4096;
pub const _ISdigit: ctype_mask = 2048;
pub const _ISalpha: ctype_mask = 1024;
pub const _ISlower: ctype_mask = 512;
pub const _ISupper: ctype_mask = 256;
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
#[derive(Default)]
#[repr(C)]
pub struct format_tree {
    pub type_0: format_type,
    /// The client the tree draws its client formats from, observed rather
    /// than held. This is not the client the tree was created for — that one
    /// is `client_ref` below — but the one `format_defaults` picked out.
    pub(crate) c_ref: Option<ClientWeak>,
    /// The session the tree draws on, observed rather than held.
    pub(crate) s_ref: Option<SessionWeak>,
    /// The link the tree draws on, named by the session that holds it and
    /// the index it holds it at, or nothing when it draws on none.
    pub(crate) wl_ref: Option<(SessionWeak, ::core::ffi::c_int)>,
    /// The window the tree draws on, observed the same way.
    pub(crate) w_ref: Option<WindowWeak>,
    /// The id of the pane the tree draws on, or nothing when it draws on
    /// none. A pane is named by its id and nothing else.
    pub wp_id: Option<u_int>,
    /// The name of the buffer the tree draws on, or nothing when it draws on
    /// none. A buffer is named by its name and nothing else.
    pub pb_name: Option<::std::ffi::CString>,
    /// The queue item the tree was made for, observed rather than held.
    pub(crate) item_ref: Option<CmdqItemWeak>,
    pub(crate) client_ref: Option<ClientRef>,
    pub flags: ::core::ffi::c_int,
    pub tag: u_int,
    pub m: mouse_event,
    pub tree: format_entry_tree,
}

impl format_tree {
    /// The session the tree draws on, or null when it draws on none or the
    /// server has since given it up.
    pub(crate) fn session(&self) -> *mut session {
        self.s_ref
            .as_ref()
            .and_then(SessionWeak::upgrade)
            .map_or(::core::ptr::null_mut(), |s| s.as_ptr())
    }

    /// Records `s` as the session the tree draws on.
    pub(crate) fn set_session(&mut self, s: *mut session) {
        self.s_ref = crate::session::session_ref_from_ptr(s).map(|s| s.downgrade());
    }

    /// The link the tree draws on, or null when it draws on none or the
    /// session has since given it up.
    pub(crate) fn winlink(&self) -> *mut winlink {
        let Some((held, idx)) = self.wl_ref.as_ref() else {
            return ::core::ptr::null_mut();
        };
        let Some(held) = held.upgrade() else {
            return ::core::ptr::null_mut();
        };
        crate::session::winlink_of(held.as_ptr(), Some(*idx))
    }

    /// Records `wl` as the link the tree draws on.
    pub(crate) fn set_winlink(&mut self, wl: *mut winlink) {
        self.wl_ref = unsafe {
            wl.as_ref().and_then(|wl| {
                crate::session::session_ref_from_ptr(wl.session()).map(|s| (s.downgrade(), wl.idx))
            })
        };
    }

    /// The window the tree draws on, or null the same way.
    pub(crate) fn window(&self) -> *mut window {
        self.w_ref
            .as_ref()
            .and_then(WindowWeak::upgrade)
            .map_or(::core::ptr::null_mut(), |w| w.as_ptr())
    }

    /// Records `w` as the window the tree draws on.
    pub(crate) fn set_window(&mut self, w: *mut window) {
        self.w_ref = crate::window::window_ref_from_ptr(w).map(|w| w.downgrade());
    }

    /// The pane the tree draws on, or null when it draws on none or the
    /// server has since given that pane up.
    pub(crate) fn pane(&self) -> *mut window_pane {
        let Some(id) = self.wp_id else {
            return ::core::ptr::null_mut();
        };
        let w = self.window();
        match w.is_null() {
            true => crate::window::window_pane_find_by_id(id),
            false => crate::window::window_pane_of_id(w, id),
        }
    }

    /// Records `wp` as the pane the tree draws on.
    pub(crate) fn set_pane(&mut self, wp: *mut window_pane) {
        self.wp_id = unsafe { wp.as_ref().map(|wp| wp.id) };
    }

    /// The client the tree draws its client formats from, or null when it
    /// draws on none or the server has since given it up.
    pub(crate) fn drawn_client(&self) -> *mut client {
        self.c_ref
            .as_ref()
            .and_then(ClientWeak::upgrade)
            .map_or(::core::ptr::null_mut(), |c| c.as_ptr())
    }

    /// Records `c` as the client the tree draws its client formats from.
    pub(crate) fn set_drawn_client(&mut self, c: *mut client) {
        self.c_ref = client_ref_from_ptr(c).map(|c| c.downgrade());
    }

    /// The queue item the tree was made for, or null when it was made for
    /// none or the queue has since given it up.
    pub(crate) fn item(&self) -> *mut cmdq_item {
        self.item_ref
            .as_ref()
            .and_then(CmdqItemWeak::upgrade)
            .map_or(::core::ptr::null_mut(), |item| item.as_ptr())
    }

    /// Records `item` as the item the tree was made for.
    pub(crate) fn set_item(&mut self, item: *mut cmdq_item) {
        self.item_ref = crate::cmd::cmdq_item_weak_from_ptr(item);
    }

    /// The buffer the tree draws on, or null when it draws on none or the
    /// store has since given it up.
    pub(crate) fn buffer(&self) -> *mut paste_buffer {
        match self.pb_name.as_ref() {
            Some(name) => unsafe { crate::paste::paste_get_name(name.as_ptr()) },
            None => ::core::ptr::null_mut(),
        }
    }

    /// Records `pb` as the buffer the tree draws on.
    pub(crate) fn set_buffer(&mut self, pb: *mut paste_buffer) {
        self.pb_name = unsafe {
            pb.as_ref()
                .map(|pb| crate::paste::paste_buffer_name(pb).to_owned())
        };
    }

    /// The client whose jobs and working directory this tree draws on, or
    /// null when it was created without one.
    pub(crate) fn client(&self) -> *mut client {
        self.client_ref
            .as_ref()
            .map_or(::core::ptr::null_mut(), ClientRef::as_ptr)
    }
}
/// The entries of a format tree, by key. An entry lives in the map, so a
/// pointer to one lasts only until the same tree is added to again.
pub type format_entry_tree = ::std::collections::BTreeMap<CString, format_entry>;
#[repr(C)]
pub struct format_entry {
    pub value: Option<CString>,
    pub time: time_t,
    pub cb: format_entry_cb,
}
pub type format_type = ::core::ffi::c_uint;
pub const FORMAT_TYPE_PANE: format_type = 3;
pub const FORMAT_TYPE_WINDOW: format_type = 2;
pub const FORMAT_TYPE_SESSION: format_type = 1;
pub const FORMAT_TYPE_UNKNOWN: format_type = 0;
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
/// The key a job hangs under: the tree's tag, then the command.
pub type format_job_key = (u_int, CString);

/// The running jobs of one client, or of the server. A job is boxed because
/// the job it runs carries its address as the callback data.
pub type format_job_tree = ::std::collections::BTreeMap<format_job_key, Box<format_job>>;
#[repr(C)]
pub struct format_job {
    pub(crate) client: Option<ClientWeak>,
    pub tag: u_int,
    pub cmd: CString,
    pub expanded: Option<CString>,
    pub last: time_t,
    pub out: Option<CString>,
    pub updated: ::core::ffi::c_int,
    /// The id of the job the entry is running, or nothing while it runs
    /// none. A job is named by its id and nothing else, so the entry never
    /// names one that has finished.
    pub job_id: Option<u_int>,
    pub status: ::core::ffi::c_int,
}

impl format_job {
    /// The job the entry is running, or null while it runs none.
    fn job(&self) -> *mut job {
        self.job_id
            .map_or(::core::ptr::null_mut::<job>(), job_find_by_id)
    }
}
pub const SORT_END: sort_order = 8;
pub const SORT_Z: sort_order = 7;
pub const SORT_SIZE: sort_order = 6;
pub const SORT_ORDER: sort_order = 5;
pub const SORT_NAME: sort_order = 4;
pub const SORT_MODIFIER: sort_order = 3;
pub const SORT_INDEX: sort_order = 2;
pub const SORT_CREATION: sort_order = 1;
pub const SORT_ACTIVITY: sort_order = 0;
/// The state of one format expansion. The default is a top-level expansion:
/// no tree, no loop depth and no flags.
///
/// The tree stays a raw view rather than a borrow: expanding fills entries in
/// and starts jobs, so a shared reference would be wrong, and a `&mut` cannot
/// live in a state that a nested loop copies while the outer one still holds
/// its own.
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
pub enum FormatTableCallback {
    String(unsafe fn(&format_tree) -> Option<CString>),
    Time(unsafe fn(&format_tree) -> Option<timeval>),
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct format_table_entry {
    pub key: &'static CStr,
    pub cb: FormatTableCallback,
}

pub const LESS_THAN_EQUAL: format_operator = 10;
pub const LESS_THAN: format_operator = 9;
pub const GREATER_THAN_EQUAL: format_operator = 8;
pub const GREATER_THAN: format_operator = 7;
pub const NOT_EQUAL: format_operator = 6;
pub const EQUAL: format_operator = 5;
pub const MODULUS: format_operator = 4;
pub const DIVIDE: format_operator = 3;
pub const MULTIPLY: format_operator = 2;
pub const SUBTRACT: format_operator = 1;
pub const ADD: format_operator = 0;
pub type format_operator = ::core::ffi::c_uint;
pub const FNM_CASEFOLD: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << 4 as ::core::ffi::c_int;
pub const REG_EXTENDED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const REG_ICASE: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
pub const REG_NOSUB: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const MODE_CURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MODE_INSERT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MODE_KCURSOR: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MODE_KKEYPAD: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const MODE_WRAP: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const MODE_MOUSE_STANDARD: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const MODE_MOUSE_BUTTON: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const MODE_CURSOR_BLINKING: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const MODE_MOUSE_UTF8: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const MODE_MOUSE_SGR: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const MODE_BRACKETPASTE: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const MODE_MOUSE_ALL: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const MODE_ORIGIN: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const MODE_KEYS_EXTENDED: ::core::ffi::c_int = 32768;
pub const MODE_CURSOR_VERY_VISIBLE: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const MODE_KEYS_EXTENDED_2: ::core::ffi::c_int = 262144;
pub const MODE_SYNC: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;
pub const ALL_MOUSE_MODES: ::core::ffi::c_int =
    MODE_MOUSE_STANDARD | MODE_MOUSE_BUTTON | MODE_MOUSE_ALL;
pub const EXTENDED_KEY_MODES: ::core::ffi::c_int = MODE_KEYS_EXTENDED | MODE_KEYS_EXTENDED_2;
pub const GRID_FLAG_PADDING: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRID_FLAG_TAB: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const GRID_LINE_WRAPPED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINDOW_PANE_NO_MODE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_ZOOMED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const PANE_INPUTOFF: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const PANE_STATUSREADY: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const PANE_STATUSDRAWN: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const PANE_UNSEENCHANGES: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const WINDOW_ZOOMED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const WINLINK_BELL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINLINK_ACTIVITY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const WINLINK_SILENCE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const WINLINK_ALERTFLAGS: ::core::ffi::c_int =
    WINLINK_BELL | WINLINK_ACTIVITY | WINLINK_SILENCE;
pub const PANE_STATUS_TOP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_STATUS_BOTTOM: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const TTY_STARTED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CLIENT_READONLY: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CLIENT_UTF8: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const FORMAT_STATUS: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const FORMAT_FORCE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const FORMAT_NOJOBS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const FORMAT_VERBOSE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const FORMAT_LAST: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const FORMAT_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FORMAT_PANE: ::core::ffi::c_uint = 0x80000000 as ::core::ffi::c_uint;
pub const FORMAT_WINDOW: ::core::ffi::c_uint = 0x40000000 as ::core::ffi::c_uint;
pub const JOB_NOWAIT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
static format_jobs: GlobalTree<format_job_key, Box<format_job>> = GlobalTree::new();
pub const FORMAT_MAX_WIDTH: ::core::ffi::c_int = 10000 as ::core::ffi::c_int;
pub const FORMAT_MAX_REPEAT: ::core::ffi::c_int = 10000 as ::core::ffi::c_int;
pub const FORMAT_MAX_PRECISION: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
pub const FORMAT_TIMESTRING: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const FORMAT_BASENAME: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const FORMAT_DIRNAME: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const FORMAT_QUOTE_SHELL: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const FORMAT_LITERAL: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const FORMAT_EXPAND: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const FORMAT_EXPANDTIME: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const FORMAT_SESSIONS: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const FORMAT_WINDOWS: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const FORMAT_PANES: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const FORMAT_PRETTY: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const FORMAT_LENGTH: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const FORMAT_WIDTH: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const FORMAT_QUOTE_STYLE: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const FORMAT_WINDOW_NAME: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const FORMAT_SESSION_NAME: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
pub const FORMAT_CHARACTER: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const FORMAT_COLOUR: ::core::ffi::c_int = 0x20000 as ::core::ffi::c_int;
pub const FORMAT_CLIENTS: ::core::ffi::c_int = 0x40000 as ::core::ffi::c_int;
pub const FORMAT_NOT: ::core::ffi::c_int = 0x80000 as ::core::ffi::c_int;
pub const FORMAT_NOT_NOT: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;
pub const FORMAT_REPEAT: ::core::ffi::c_int = 0x200000 as ::core::ffi::c_int;
pub const FORMAT_QUOTE_ARGUMENTS: ::core::ffi::c_int = 0x400000 as ::core::ffi::c_int;
pub const FORMAT_LOOP_LIMIT: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
pub const FORMAT_TIME_LIMIT: ::core::ffi::c_int = 100 as ::core::ffi::c_int;
pub const FORMAT_EXPAND_TIME: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const FORMAT_EXPAND_NOJOBS: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
static format_upper: [Option<&::core::ffi::CStr>; 26] = [
    None,
    None,
    None,
    Some(c"pane_id"),
    None,
    Some(c"window_flags"),
    None,
    Some(c"host"),
    Some(c"window_index"),
    None,
    None,
    None,
    None,
    None,
    None,
    Some(c"pane_index"),
    None,
    None,
    Some(c"session_name"),
    Some(c"pane_title"),
    None,
    None,
    Some(c"window_name"),
    None,
    None,
    None,
];
static format_lower: [Option<&::core::ffi::CStr>; 26] = [
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(c"host_short"),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];
#[inline]
unsafe fn format_logging(ft: &mut format_tree) -> ::core::ffi::c_int {
    (log_get_level() != 0 as ::core::ffi::c_int || ft.flags & FORMAT_VERBOSE != 0)
        as ::core::ffi::c_int
}
unsafe fn format_log1(
    es: &mut format_expand_state,
    mut from: *const ::core::ffi::c_char,
    mut fmt: *const ::core::ffi::c_char,
    args: &[FmtArg],
) {
    unsafe {
        let ft: *mut format_tree = es.ft;
        static spaces: [::core::ffi::c_char; 11] = unsafe {
            ::core::mem::transmute::<[u8; 11], [::core::ffi::c_char; 11]>(*b"          \0")
        };
        if format_logging(&mut *ft) == 0 {
            return;
        }
        let s = format_alloc(fmt, args);
        log_debug(c"%s: %s".as_ptr(), fmt_args![from, s.as_ptr()]);
        if !(*ft).item().is_null() && (*ft).flags & FORMAT_VERBOSE != 0 {
            cmdq_print(
                (*ft).item(),
                c"#%.*s%s".as_ptr(),
                fmt_args![
                    es.loop_0,
                    &raw const spaces as *const ::core::ffi::c_char,
                    s.as_ptr()
                ],
            );
        }
    }
}
unsafe fn format_job_update(mut job: *mut job) {
    unsafe {
        let mut fj: *mut format_job = match job_get_data(job) {
            JobData::Format(data) => *data,
            _ => panic!("format job data is not format data"),
        };
        let event = job_get_event(job);
        let mut line = None;
        let mut t: time_t = 0;
        loop {
            let Some(next) = event.with_input(|buffer| buffer.read_line()).flatten() else {
                break;
            };
            line = Some(next);
        }
        let Some(line) = line else {
            return;
        };
        (*fj).updated = 1 as ::core::ffi::c_int;
        (*fj).out = Some(CString::from_vec_unchecked(line.to_vec()));
        log_debug(
            c"%s: %p %s: %s".as_ptr(),
            fmt_args![
                c"format_job_update".as_ptr(),
                fj,
                (*fj).cmd.as_ptr(),
                (*fj).out.as_deref()
            ],
        );
        t = time(::core::ptr::null_mut::<time_t>());
        if (*fj).status != 0 && (*fj).last != t {
            if let Some(c) = (*fj).client.as_ref().and_then(ClientWeak::upgrade) {
                server_status_client(c.as_ptr());
            }
            (*fj).last = t;
        }
    }
}
unsafe fn format_job_complete(mut job: *mut job) {
    unsafe {
        let mut fj: *mut format_job = match job_get_data(job) {
            JobData::Format(data) => *data,
            _ => panic!("format job data is not format data"),
        };
        let event = job_get_event(job);
        (*fj).job_id = None;
        let bytes = event
            .with_input(|buffer| {
                buffer
                    .read_line()
                    .map_or_else(|| buffer.as_slice().to_vec(), |line| line.to_vec())
            })
            .unwrap_or_default();
        let mut buf = bytes.clone();
        buf.push(0);
        log_debug(
            c"%s: %p %s: %s".as_ptr(),
            fmt_args![
                c"format_job_complete".as_ptr(),
                fj,
                (*fj).cmd.as_ptr(),
                buf.as_ptr() as *const ::core::ffi::c_char
            ],
        );
        if !bytes.is_empty() || (*fj).updated == 0 {
            (*fj).out = Some(CString::from_vec_unchecked(bytes));
        }
        if (*fj).status != 0 {
            if let Some(c) = (*fj).client.as_ref().and_then(ClientWeak::upgrade) {
                server_status_client(c.as_ptr());
            }
            (*fj).status = 0 as ::core::ffi::c_int;
        }
    }
}
unsafe fn format_list_value(buffer: &mut Buf) -> Option<CString> {
    unsafe {
        if buffer.is_empty() {
            return None;
        }
        let data = buffer.as_slice();
        let value = xasprintf(
            c"%.*s".as_ptr(),
            fmt_args![
                data.len() as ::core::ffi::c_int,
                data.as_ptr() as *const ::core::ffi::c_char
            ],
        );
        Some(value)
    }
}
unsafe fn format_job_get(es: &mut format_expand_state, cmd: &CStr) -> CString {
    unsafe {
        let cmd = cmd.as_ptr();
        let ft: *mut format_tree = es.ft;
        let mut jobs: *mut format_job_tree = ::core::ptr::null_mut::<format_job_tree>();
        let mut fj: *mut format_job = ::core::ptr::null_mut::<format_job>();
        let mut t: time_t = 0;
        let mut force: ::core::ffi::c_int = 0;
        let mut next = format_expand_state::default();
        if (*ft).client().is_null() {
            jobs = format_jobs.map() as *mut format_job_tree;
        } else {
            let client = &mut *(*ft).client();
            jobs = &raw mut **client
                .jobs
                .get_or_insert_with(|| Box::new(format_job_tree::new()));
        }
        let key: format_job_key = ((*ft).tag, CStr::from_ptr(cmd).to_owned());
        fj = &raw mut **(*jobs).entry(key).or_insert_with(|| {
            Box::new(format_job {
                client: client_ref_from_ptr((*ft).client()).map(|c| c.downgrade()),
                tag: (*ft).tag,
                cmd: CStr::from_ptr(cmd).to_owned(),
                expanded: None,
                last: 0 as time_t,
                out: None,
                updated: 0 as ::core::ffi::c_int,
                job_id: None,
                status: 0 as ::core::ffi::c_int,
            })
        });
        next = *es;
        next.flags |= FORMAT_EXPAND_NOJOBS;
        next.flags &= !FORMAT_EXPAND_TIME;
        let expanded = format_expand1(&mut next, CStr::from_ptr(cmd));
        if (*fj).expanded.as_deref() != Some(expanded.as_c_str()) {
            (*fj).expanded = Some(expanded.clone());
            force = 1 as ::core::ffi::c_int;
        } else {
            force = (*ft).flags & FORMAT_FORCE;
        }
        t = time(::core::ptr::null_mut::<time_t>());
        if force != 0 && !(*fj).job().is_null() {
            job_free((*fj).job());
            (*fj).job_id = None;
        }
        if force != 0 || (*fj).job_id.is_none() && (*fj).last != t {
            let job = job_run(
                expanded.as_ptr(),
                &[],
                ::core::ptr::null_mut::<environ_t>(),
                ::core::ptr::null_mut::<session>(),
                server_client_get_cwd((*ft).client(), ::core::ptr::null_mut::<session>()),
                Some(format_job_update),
                Some(format_job_complete),
                None,
                JobData::Format(fj),
                JOB_NOWAIT,
                -(1 as ::core::ffi::c_int),
                -(1 as ::core::ffi::c_int),
            );
            (*fj).job_id = job_id(job.as_ref());
            if (*fj).job_id.is_none() {
                (*fj).out = Some(format_alloc(
                    c"<'%s' didn't start>".as_ptr(),
                    fmt_args![(*fj).cmd.as_ptr()],
                ));
            }
            (*fj).last = t;
            (*fj).updated = 0 as ::core::ffi::c_int;
        } else if (*fj).job_id.is_some() && t - (*fj).last > 1 as time_t && (*fj).out.is_none() {
            (*fj).out = Some(format_alloc(
                c"<'%s' not ready>".as_ptr(),
                fmt_args![(*fj).cmd.as_ptr()],
            ));
        }
        if (*ft).flags & FORMAT_STATUS != 0 {
            (*fj).status = 1 as ::core::ffi::c_int;
        }
        if (*fj).out.is_none() {
            return CString::default();
        }
        format_expand1(&mut next, (*fj).out.as_deref().unwrap_or(c""))
    }
}
unsafe fn format_job_tidy(jobs: &mut format_job_tree, mut force: ::core::ffi::c_int) {
    unsafe {
        let now: time_t = time(::core::ptr::null_mut::<time_t>());
        let all: Vec<(format_job_key, *mut format_job)> = jobs
            .iter_mut()
            .map(|(key, fj)| (key.clone(), &raw mut **fj))
            .collect();
        for (key, fj) in all {
            if !(force == 0 && ((*fj).last > now || now - (*fj).last < 3600 as time_t)) {
                log_debug(
                    c"%s: %s".as_ptr(),
                    fmt_args![c"format_job_tidy".as_ptr(), (*fj).cmd.as_ptr()],
                );
                if !(*fj).job().is_null() {
                    job_free((*fj).job());
                }
                jobs.remove(&key);
            }
        }
    }
}
/// `fmt` through `strftime` for `tm`, or nothing when what it spells out
/// would not fit in `max` bytes counting the terminator, which is the case
/// strftime reports by answering zero.
unsafe fn format_strftime(
    max: size_t,
    fmt: *const ::core::ffi::c_char,
    tm: *const tm,
) -> Option<CString> {
    unsafe {
        let mut buf = ::std::vec::from_elem(0u8, max);
        let used = strftime(buf.as_mut_ptr() as *mut ::core::ffi::c_char, max, fmt, tm);
        if used == 0 as size_t {
            return None;
        }
        buf.truncate(used as usize);
        CString::new(buf).ok()
    }
}
pub fn format_tidy_jobs() {
    unsafe {
        format_job_tidy(format_jobs.map(), 0 as ::core::ffi::c_int);
        for c in client_walk() {
            if let Some(jobs) = (*c).jobs.as_deref_mut() {
                format_job_tidy(jobs, 0 as ::core::ffi::c_int);
            }
        }
    }
}
pub unsafe fn format_lost_client(mut c: *mut client) {
    unsafe {
        if let Some(mut jobs) = (*c).jobs.take() {
            format_job_tidy(&mut jobs, 1 as ::core::ffi::c_int);
        }
    }
}
unsafe fn format_printf(mut fmt: *const ::core::ffi::c_char, args: &[FmtArg]) -> CString {
    unsafe { format_alloc(fmt, args) }
}
fn format_callback_copy(value: &CStr) -> CString {
    value.to_owned()
}
unsafe fn format_cb_host(_ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut host: [::core::ffi::c_char; 65] = [0; 65];
        if gethostname(
            &raw mut host as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 65]>() as size_t,
        ) != 0 as ::core::ffi::c_int
        {
            return Some(format_callback_copy(c""));
        }
        Some(format_callback_copy(CStr::from_ptr(
            &raw mut host as *mut ::core::ffi::c_char,
        )))
    }
}
unsafe fn format_cb_host_short(_ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut host: [::core::ffi::c_char; 65] = [0; 65];
        let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if gethostname(
            &raw mut host as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 65]>() as size_t,
        ) != 0 as ::core::ffi::c_int
        {
            return Some(format_callback_copy(c""));
        }
        cp = strchr(&raw mut host as *mut ::core::ffi::c_char, '.' as i32);
        if !cp.is_null() {
            *cp = '\0' as i32 as ::core::ffi::c_char;
        }
        Some(format_callback_copy(CStr::from_ptr(
            &raw mut host as *mut ::core::ffi::c_char,
        )))
    }
}
unsafe fn format_cb_pid(_ft: &format_tree) -> Option<CString> {
    unsafe {
        let value = xasprintf(c"%ld".as_ptr(), fmt_args![getpid() as ::core::ffi::c_long]);
        Some(value)
    }
}
unsafe fn format_cb_session_attached_list(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut s: *mut session = (*ft).session();
        let mut buffer = Buf::new();
        if s.is_null() {
            return None;
        }
        for loop_0 in client_walk() {
            if (*loop_0).session == s {
                if !buffer.is_empty() {
                    buffer.append(b",");
                }
                format_buf(
                    &mut buffer,
                    c"%s".as_ptr(),
                    fmt_args![(*loop_0).name.as_deref()],
                );
            }
        }
        format_list_value(&mut buffer)
    }
}
unsafe fn format_cb_session_alert(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut s: *mut session = (*ft).session();
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        let mut alerts: Vec<u8> = Vec::new();
        let mut alerted: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if s.is_null() {
            return None;
        }
        wl = winlinks_first(&mut (*s).windows);
        while !wl.is_null() {
            if !((*wl).flags & WINLINK_ALERTFLAGS == 0 as ::core::ffi::c_int) {
                if !alerted & (*wl).flags & WINLINK_ACTIVITY != 0 {
                    alerts.push(b'#');
                    alerted |= WINLINK_ACTIVITY;
                }
                if !alerted & (*wl).flags & WINLINK_BELL != 0 {
                    alerts.push(b'!');
                    alerted |= WINLINK_BELL;
                }
                if !alerted & (*wl).flags & WINLINK_SILENCE != 0 {
                    alerts.push(b'~');
                    alerted |= WINLINK_SILENCE;
                }
            }
            wl = winlinks_after(wl);
        }
        Some(CString::new(alerts).expect("alert marks have no NUL"))
    }
}
unsafe fn format_cb_session_alerts(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut s: *mut session = (*ft).session();
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        let mut alerts: Vec<u8> = Vec::new();
        if s.is_null() {
            return None;
        }
        wl = winlinks_first(&mut (*s).windows);
        while !wl.is_null() {
            if !((*wl).flags & WINLINK_ALERTFLAGS == 0 as ::core::ffi::c_int) {
                if !alerts.is_empty() {
                    alerts.push(b',');
                }
                alerts.extend_from_slice(::std::format!("{}", (*wl).idx).as_bytes());
                if (*wl).flags & WINLINK_ACTIVITY != 0 {
                    alerts.push(b'#');
                }
                if (*wl).flags & WINLINK_BELL != 0 {
                    alerts.push(b'!');
                }
                if (*wl).flags & WINLINK_SILENCE != 0 {
                    alerts.push(b'~');
                }
            }
            wl = winlinks_after(wl);
        }
        alerts.truncate(1023);
        Some(CString::new(alerts).expect("window numbers and marks have no NUL"))
    }
}
unsafe fn format_cb_session_stack(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut s: *mut session = (*ft).session();
        if s.is_null() {
            return None;
        }
        let mut result: Vec<u8> = ::std::format!("{}", (*session_get_curw(s)).idx).into_bytes();
        for idx in (*s).lastw.clone() {
            if !result.is_empty() {
                result.push(b',');
            }
            result.extend_from_slice(::std::format!("{}", idx).as_bytes());
        }
        result.truncate(1023);
        Some(CString::new(result).expect("window numbers have no NUL"))
    }
}
unsafe fn format_cb_window_stack_index(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        let mut idx: u_int = 0;
        if (*ft).winlink().is_null() {
            return None;
        }
        s = (*(*ft).winlink()).session();
        idx = 0 as u_int;
        wl = ::core::ptr::null_mut::<winlink>();
        for stacked in (*s).lastw.clone() {
            idx = idx.wrapping_add(1);
            if stacked == (*(*ft).winlink()).idx {
                wl = (*ft).winlink();
                break;
            }
        }
        if wl.is_null() {
            return Some(format_callback_copy(c"0"));
        }
        let value = xasprintf(c"%u".as_ptr(), fmt_args![idx]);
        Some(value)
    }
}
unsafe fn format_cb_window_linked_sessions_list(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut buffer = Buf::new();
        if (*ft).winlink().is_null() {
            return None;
        }
        w = (*(*ft).winlink()).window();
        for wl in winlinks_into(w) {
            if !buffer.is_empty() {
                buffer.append(b",");
            }
            format_buf(
                &mut buffer,
                c"%s".as_ptr(),
                fmt_args![session_name((*wl).session())],
            );
        }
        format_list_value(&mut buffer)
    }
}
unsafe fn format_cb_window_active_sessions(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut n: u_int = 0 as u_int;
        if (*ft).winlink().is_null() {
            return None;
        }
        w = (*(*ft).winlink()).window();
        for wl in winlinks_into(w) {
            if session_get_curw((*wl).session()) == wl {
                n = n.wrapping_add(1);
            }
        }
        let value = xasprintf(c"%u".as_ptr(), fmt_args![n]);
        Some(value)
    }
}
unsafe fn format_cb_window_active_sessions_list(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut buffer = Buf::new();
        if (*ft).winlink().is_null() {
            return None;
        }
        w = (*(*ft).winlink()).window();
        for wl in winlinks_into(w) {
            if session_get_curw((*wl).session()) == wl {
                if !buffer.is_empty() {
                    buffer.append(b",");
                }
                format_buf(
                    &mut buffer,
                    c"%s".as_ptr(),
                    fmt_args![session_name((*wl).session())],
                );
            }
        }
        format_list_value(&mut buffer)
    }
}
unsafe fn format_cb_window_active_clients(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut client_session: *mut session = ::core::ptr::null_mut::<session>();
        let mut n: u_int = 0 as u_int;
        if (*ft).winlink().is_null() {
            return None;
        }
        w = (*(*ft).winlink()).window();
        for loop_0 in client_walk() {
            client_session = (*loop_0).session;
            if !client_session.is_null() && w == (*session_get_curw(client_session)).window() {
                n = n.wrapping_add(1);
            }
        }
        let value = xasprintf(c"%u".as_ptr(), fmt_args![n]);
        Some(value)
    }
}
unsafe fn format_cb_window_active_clients_list(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut client_session: *mut session = ::core::ptr::null_mut::<session>();
        let mut buffer = Buf::new();
        if (*ft).winlink().is_null() {
            return None;
        }
        w = (*(*ft).winlink()).window();
        for loop_0 in client_walk() {
            client_session = (*loop_0).session;
            if !client_session.is_null() && w == (*session_get_curw(client_session)).window() {
                if !buffer.is_empty() {
                    buffer.append(b",");
                }
                format_buf(
                    &mut buffer,
                    c"%s".as_ptr(),
                    fmt_args![(*loop_0).name.as_deref()],
                );
            }
        }
        format_list_value(&mut buffer)
    }
}
unsafe fn format_cb_window_layout(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut w: *mut window = (*ft).window();
        if w.is_null() {
            return None;
        }
        if !(*w).saved_layout_root_ptr().is_null() {
            return layout_dump(w, (*w).saved_layout_root_ptr());
        }
        layout_dump(w, (*w).layout_root_ptr())
    }
}
unsafe fn format_cb_window_visible_layout(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut w: *mut window = (*ft).window();
        if w.is_null() {
            return None;
        }
        layout_dump(w, (*w).layout_root_ptr())
    }
}
unsafe fn format_cb_start_command(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        if wp.is_null() {
            return None;
        }
        let command = cmd_stringify_argv(&(*wp).argv);
        Some(format_callback_copy(&command))
    }
}
unsafe fn format_cb_start_path(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        if wp.is_null() {
            return None;
        }
        if (*wp).cwd.is_none() {
            return Some(format_callback_copy(c""));
        }
        Some(format_callback_copy((*wp).cwd.as_deref().unwrap_or(c"")))
    }
}
unsafe fn format_cb_current_command(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        if wp.is_null() || (*wp).shell.is_none() {
            return None;
        }
        let cmd = osdep_get_name((*wp).fd).filter(|cmd| !cmd.as_bytes().is_empty());
        let Some(cmd) = cmd else {
            let command = cmd_stringify_argv(&(*wp).argv);
            if command.as_bytes().is_empty() {
                let value = parse_window_name((*wp).shell.as_deref().unwrap_or(c""));
                return Some(format_callback_copy(&value));
            }
            let value = parse_window_name(&command);
            return Some(format_callback_copy(&value));
        };
        let value = parse_window_name(&cmd);
        Some(format_callback_copy(&value))
    }
}
unsafe fn format_cb_current_path(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        if wp.is_null() {
            return None;
        }
        let Some(cwd) = osdep_get_cwd((*wp).fd) else {
            return None;
        };
        Some(format_callback_copy(&cwd))
    }
}
unsafe fn format_cb_history_bytes(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
        let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
        let mut size: size_t = 0 as size_t;
        let mut i: u_int = 0;
        if wp.is_null() {
            return None;
        }
        gd = screen_grid_ptr(&mut (*wp).base);
        i = 0 as u_int;
        while i < (*gd).hsize.wrapping_add((*gd).sy) {
            gl = grid_get_line(&mut *gd, i);
            size = (size as ::core::ffi::c_ulong).wrapping_add(
                ((*gl).cellsize() as usize).wrapping_mul(::core::mem::size_of::<grid_cell_entry>())
                    as ::core::ffi::c_ulong,
            ) as size_t as size_t;
            size = (size as ::core::ffi::c_ulong).wrapping_add(
                ((*gl).extdsize() as usize).wrapping_mul(::core::mem::size_of::<grid_extd_entry>())
                    as ::core::ffi::c_ulong,
            ) as size_t as size_t;
            i = i.wrapping_add(1);
        }
        size = (size as ::core::ffi::c_ulong).wrapping_add(
            ((*gd).hsize.wrapping_add((*gd).sy) as usize)
                .wrapping_mul(::core::mem::size_of::<grid_line>())
                as ::core::ffi::c_ulong,
        ) as size_t as size_t;
        let value = xasprintf(c"%zu".as_ptr(), fmt_args![size]);
        Some(value)
    }
}
unsafe fn format_cb_history_all_bytes(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
        let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
        let mut i: u_int = 0;
        let mut lines: u_int = 0;
        let mut cells: u_int = 0 as u_int;
        let mut extended_cells: u_int = 0 as u_int;
        if wp.is_null() {
            return None;
        }
        gd = screen_grid_ptr(&mut (*wp).base);
        lines = (*gd).hsize.wrapping_add((*gd).sy);
        i = 0 as u_int;
        while i < lines {
            gl = grid_get_line(&mut *gd, i);
            cells = cells.wrapping_add((*gl).cellsize());
            extended_cells = extended_cells.wrapping_add((*gl).extdsize());
            i = i.wrapping_add(1);
        }
        let value = xasprintf(
            c"%u,%zu,%u,%zu,%u,%zu".as_ptr(),
            fmt_args![
                lines,
                (lines as usize).wrapping_mul(::core::mem::size_of::<grid_line>()),
                cells,
                (cells as usize).wrapping_mul(::core::mem::size_of::<grid_cell_entry>()),
                extended_cells,
                (extended_cells as usize).wrapping_mul(::core::mem::size_of::<grid_extd_entry>())
            ],
        );
        Some(value)
    }
}
unsafe fn format_cb_pane_tabs(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        let mut buffer = Buf::new();
        let mut i: u_int = 0;
        if wp.is_null() {
            return None;
        }
        i = 0 as u_int;
        let tabs = &(*wp).base.tabs;
        while i < (*screen_grid_ptr(&mut (*wp).base)).sx {
            if !(tabs[(i >> 3 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << (i & 0x7 as u_int)
                == 0)
            {
                if !buffer.is_empty() {
                    buffer.append(b",");
                }
                format_buf(&mut buffer, c"%u".as_ptr(), fmt_args![i]);
            }
            i = i.wrapping_add(1);
        }
        format_list_value(&mut buffer)
    }
}
unsafe fn format_cb_pane_fg(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        let mut gc = grid_default_cell;
        if wp.is_null() {
            return None;
        }
        tty_default_colours(&mut gc, wp);
        Some(colour_tostring(gc.fg))
    }
}
unsafe fn format_cb_pane_flags(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(window_pane_printable_flags((*ft).pane()));
        }
        None
    }
}
unsafe fn format_cb_pane_floating_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        if !wp.is_null() {
            if window_pane_is_floating(wp) != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_pane_bg(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        let mut gc = grid_default_cell;
        if wp.is_null() {
            return None;
        }
        tty_default_colours(&mut gc, wp);
        Some(colour_tostring(gc.bg))
    }
}
unsafe fn format_cb_session_group_list(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut s: *mut session = (*ft).session();
        let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
        let mut buffer = Buf::new();
        if s.is_null() {
            return None;
        }
        sg = session_group_contains(s);
        if sg.is_null() {
            return None;
        }
        for loop_0 in group_walk(sg) {
            if !buffer.is_empty() {
                buffer.append(b",");
            }
            format_buf(&mut buffer, c"%s".as_ptr(), fmt_args![session_name(loop_0)]);
        }
        format_list_value(&mut buffer)
    }
}
unsafe fn format_cb_session_group_attached_list(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut s: *mut session = (*ft).session();
        let mut client_session: *mut session = ::core::ptr::null_mut::<session>();
        let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
        let mut buffer = Buf::new();
        if s.is_null() {
            return None;
        }
        sg = session_group_contains(s);
        if sg.is_null() {
            return None;
        }
        for loop_0 in client_walk() {
            client_session = (*loop_0).session;
            if !client_session.is_null() {
                for session_loop in group_walk(sg) {
                    if session_loop == client_session {
                        if !buffer.is_empty() {
                            buffer.append(b",");
                        }
                        format_buf(
                            &mut buffer,
                            c"%s".as_ptr(),
                            fmt_args![(*loop_0).name.as_deref()],
                        );
                    }
                }
            }
        }
        format_list_value(&mut buffer)
    }
}
unsafe fn format_cb_pane_in_mode(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        if wp.is_null() {
            return None;
        }
        let n: u_int = (*wp).modes.len() as u_int;
        let value = xasprintf(c"%u".as_ptr(), fmt_args![n]);
        Some(value)
    }
}
unsafe fn format_cb_pane_at_top(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut status: ::core::ffi::c_int = 0;
        let mut flag: ::core::ffi::c_int = 0;
        if wp.is_null() {
            return None;
        }
        w = (*wp).window;
        status = options_get_number((*w).options_ptr(), c"pane-border-status".as_ptr())
            as ::core::ffi::c_int;
        if status == PANE_STATUS_TOP {
            flag = ((*wp).yoff == 1 as ::core::ffi::c_int) as ::core::ffi::c_int;
        } else {
            flag = ((*wp).yoff == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        }
        let value = xasprintf(c"%d".as_ptr(), fmt_args![flag]);
        Some(value)
    }
}
unsafe fn format_cb_pane_at_bottom(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut status: ::core::ffi::c_int = 0;
        let mut flag: ::core::ffi::c_int = 0;
        if wp.is_null() {
            return None;
        }
        w = (*wp).window;
        status = options_get_number((*w).options_ptr(), c"pane-border-status".as_ptr())
            as ::core::ffi::c_int;
        if status == PANE_STATUS_BOTTOM {
            flag = ((*wp).yoff + (*wp).sy as ::core::ffi::c_int
                == (*w).sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                as ::core::ffi::c_int;
        } else {
            flag = ((*wp).yoff + (*wp).sy as ::core::ffi::c_int == (*w).sy as ::core::ffi::c_int)
                as ::core::ffi::c_int;
        }
        let value = xasprintf(c"%d".as_ptr(), fmt_args![flag]);
        Some(value)
    }
}
unsafe fn format_cb_cursor_character(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        let mut gc = grid_default_cell;
        let mut value: Option<CString> = None;
        if wp.is_null() {
            return None;
        }
        gc = grid_view_get_cell(screen_grid(&(*wp).base), (*wp).base.cx, (*wp).base.cy);
        if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
            value = Some(xasprintf(
                c"%.*s".as_ptr(),
                fmt_args![
                    gc.data.size as ::core::ffi::c_int,
                    &raw mut gc.data.data as *mut u_char
                ],
            ));
        }
        value
    }
}
unsafe fn format_cb_cursor_colour(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        if wp.is_null() || (*wp).screen().is_null() {
            return None;
        }
        if (*(*wp).screen()).ccolour != -(1 as ::core::ffi::c_int) {
            return Some(colour_tostring((*(*wp).screen()).ccolour));
        }
        Some(colour_tostring((*(*wp).screen()).default_ccolour))
    }
}
unsafe fn format_cb_mouse_word(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
        let mut x: u_int = 0;
        let mut y: u_int = 0;
        if ft.m.valid == 0 {
            return None;
        }
        let Some((_, _, wp)) = cmd_mouse_pane(&ft.m) else {
            return None;
        };
        if match cmd_mouse_at(wp, &ft.m, 0 as ::core::ffi::c_int) {
            Some((at_x, at_y)) => {
                (x, y) = (at_x, at_y);
                false
            }
            None => true,
        } {
            return None;
        }
        if !(*wp).modes.is_empty() {
            if window_pane_mode(wp) != WINDOW_PANE_NO_MODE {
                return window_copy_get_word(wp, x, y);
            }
            return None;
        }
        gd = screen_grid_ptr(&mut (*wp).base);
        format_grid_word(gd, x, (*gd).hsize.wrapping_add(y))
    }
}
unsafe fn format_cb_mouse_hyperlink(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
        let mut x: u_int = 0;
        let mut y: u_int = 0;
        if ft.m.valid == 0 {
            return None;
        }
        let Some((_, _, wp)) = cmd_mouse_pane(&ft.m) else {
            return None;
        };
        if match cmd_mouse_at(wp, &ft.m, 0 as ::core::ffi::c_int) {
            Some((at_x, at_y)) => {
                (x, y) = (at_x, at_y);
                false
            }
            None => true,
        } {
            return None;
        }
        if !(*wp).modes.is_empty() {
            if window_pane_mode(wp) != WINDOW_PANE_NO_MODE {
                return window_copy_get_hyperlink(wp, x, y);
            }
            return None;
        }
        gd = screen_grid_ptr(&mut (*wp).base);
        format_grid_hyperlink(gd, x, (*gd).hsize.wrapping_add(y), (*wp).screen())
    }
}
unsafe fn format_cb_mouse_line(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
        let mut y: u_int = 0;
        if ft.m.valid == 0 {
            return None;
        }
        let Some((_, _, wp)) = cmd_mouse_pane(&ft.m) else {
            return None;
        };
        if match cmd_mouse_at(wp, &ft.m, 0 as ::core::ffi::c_int) {
            Some((at_x, at_y)) => {
                (_, y) = (at_x, at_y);
                false
            }
            None => true,
        } {
            return None;
        }
        if !(*wp).modes.is_empty() {
            if window_pane_mode(wp) != WINDOW_PANE_NO_MODE {
                return Some(window_copy_get_line(wp, y));
            }
            return None;
        }
        gd = screen_grid_ptr(&mut (*wp).base);
        Some(format_grid_line(gd, (*gd).hsize.wrapping_add(y)))
    }
}
unsafe fn format_cb_mouse_status_line(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut y: u_int = 0;
        if ft.m.valid == 0 {
            return None;
        }
        if (*ft).drawn_client().is_null() || !(*(*ft).drawn_client()).tty.flags & TTY_STARTED != 0 {
            return None;
        }
        if ft.m.statusat == 0 as ::core::ffi::c_int && ft.m.y < ft.m.statuslines {
            y = ft.m.y;
        } else if ft.m.statusat > 0 as ::core::ffi::c_int && ft.m.y >= ft.m.statusat as u_int {
            y = ft.m.y.wrapping_sub(ft.m.statusat as u_int);
        } else {
            return None;
        }
        let value = xasprintf(c"%u".as_ptr(), fmt_args![y]);
        Some(value)
    }
}
unsafe fn format_cb_mouse_status_range(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
        let mut x: u_int = 0;
        let mut y: u_int = 0;
        if ft.m.valid == 0 {
            return None;
        }
        if (*ft).drawn_client().is_null() || !(*(*ft).drawn_client()).tty.flags & TTY_STARTED != 0 {
            return None;
        }
        if ft.m.statusat == 0 as ::core::ffi::c_int && ft.m.y < ft.m.statuslines {
            x = ft.m.x;
            y = ft.m.y;
        } else if ft.m.statusat > 0 as ::core::ffi::c_int && ft.m.y >= ft.m.statusat as u_int {
            x = ft.m.x;
            y = ft.m.y.wrapping_sub(ft.m.statusat as u_int);
        } else {
            return None;
        }
        sr = status_get_range((*ft).drawn_client(), x, y);
        if sr.is_null() {
            return None;
        }
        match (*sr).type_0 {
            STYLE_RANGE_NONE => return None,
            STYLE_RANGE_LEFT => {
                return Some(format_callback_copy(c"left"));
            }
            STYLE_RANGE_RIGHT => {
                return Some(format_callback_copy(c"right"));
            }
            STYLE_RANGE_PANE => {
                return Some(format_callback_copy(c"pane"));
            }
            STYLE_RANGE_WINDOW => {
                return Some(format_callback_copy(c"window"));
            }
            STYLE_RANGE_SESSION => {
                return Some(format_callback_copy(c"session"));
            }
            STYLE_RANGE_USER => {
                return Some(format_callback_copy(CStr::from_ptr(
                    &raw mut (*sr).string as *mut ::core::ffi::c_char,
                )));
            }
            STYLE_RANGE_CONTROL => {
                return Some(format_callback_copy(c"control"));
            }
            _ => {}
        }
        None
    }
}
unsafe fn format_cb_alternate_on(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).base.saved_grid.is_some() {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_alternate_saved_x(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*(*ft).pane()).base.saved_cx],
            ));
        }
        None
    }
}
unsafe fn format_cb_alternate_saved_y(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*(*ft).pane()).base.saved_cy],
            ));
        }
        None
    }
}
unsafe fn format_cb_bracket_paste_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() && !(*(*ft).pane()).screen().is_null() {
            if (*(*(*ft).pane()).screen()).mode & MODE_BRACKETPASTE != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_buffer_name(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).buffer().is_null() {
            return Some(format_callback_copy(paste_buffer_name(&*(*ft).buffer())));
        }
        None
    }
}
unsafe fn format_cb_buffer_sample(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).buffer().is_null() {
            return Some(paste_make_sample(&*(*ft).buffer()));
        }
        None
    }
}
unsafe fn format_cb_buffer_full(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).buffer().is_null() {
            let bytes = paste_buffer_data(&*(*ft).buffer());
            let end = bytes
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(bytes.len());
            return Some(CString::new(&bytes[..end]).expect("paste buffer data has no NUL"));
        }
        None
    }
}
unsafe fn format_cb_buffer_size(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).buffer().is_null() {
            let size = paste_buffer_data(&*(*ft).buffer()).len() as size_t;
            return Some(format_printf(c"%zu".as_ptr(), fmt_args![size]));
        }
        None
    }
}
unsafe fn format_cb_client_cell_height(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() && (*(*ft).drawn_client()).tty.flags & TTY_STARTED != 0 {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*(*ft).drawn_client()).tty.ypixel],
            ));
        }
        None
    }
}
unsafe fn format_cb_client_cell_width(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() && (*(*ft).drawn_client()).tty.flags & TTY_STARTED != 0 {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*(*ft).drawn_client()).tty.xpixel],
            ));
        }
        None
    }
}
unsafe fn format_cb_client_control_mode(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            if (*(*ft).drawn_client()).flags & CLIENT_CONTROL as uint64_t != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_client_discarded(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            return Some(format_printf(
                c"%zu".as_ptr(),
                fmt_args![(*(*ft).drawn_client()).discarded],
            ));
        }
        None
    }
}
unsafe fn format_cb_client_flags(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            return Some(server_client_get_flags((*ft).drawn_client()));
        }
        None
    }
}
unsafe fn format_cb_client_height(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() && (*(*ft).drawn_client()).tty.flags & TTY_STARTED != 0 {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*(*ft).drawn_client()).tty.sy],
            ));
        }
        None
    }
}
unsafe fn format_cb_client_key_table(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            return Some(format_callback_copy(key_table_name(
                (*(*ft).drawn_client()).keytable(),
            )));
        }
        None
    }
}
unsafe fn format_cb_client_last_session(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            let last = client_get_last_session((*ft).drawn_client());
            if !last.is_null() && session_alive(last) != 0 {
                return Some(format_callback_copy(CStr::from_ptr(session_name(last))));
            }
        }
        None
    }
}
unsafe fn format_cb_client_name(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            return Some(format_callback_copy(
                (*(*ft).drawn_client()).name.as_deref().unwrap_or(c""),
            ));
        }
        None
    }
}
unsafe fn format_cb_client_pid(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            return Some(format_printf(
                c"%ld".as_ptr(),
                fmt_args![(*(*ft).drawn_client()).pid as ::core::ffi::c_long],
            ));
        }
        None
    }
}
unsafe fn format_cb_client_prefix(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if !(*ft).drawn_client().is_null() {
            name = server_client_get_key_table((*ft).drawn_client());
            if key_table_name((*(*ft).drawn_client()).keytable()) == CStr::from_ptr(name) {
                return Some(format_callback_copy(c"0"));
            }
            return Some(format_callback_copy(c"1"));
        }
        None
    }
}
unsafe fn format_cb_client_readonly(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            if (*(*ft).drawn_client()).flags & CLIENT_READONLY as uint64_t != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_client_session(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() && !(*(*ft).drawn_client()).session.is_null() {
            return Some(format_callback_copy(CStr::from_ptr(session_name(
                (*(*ft).drawn_client()).session,
            ))));
        }
        None
    }
}
unsafe fn format_cb_client_termfeatures(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            return Some(tty_get_features((*(*ft).drawn_client()).term_features));
        }
        None
    }
}
unsafe fn format_cb_client_termname(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            return Some(format_callback_copy(
                (*(*ft).drawn_client()).term_name.as_deref().unwrap_or(c""),
            ));
        }
        None
    }
}
unsafe fn format_cb_client_termtype(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            if (*(*ft).drawn_client()).term_type.is_none() {
                return Some(format_callback_copy(c""));
            }
            return Some(format_callback_copy(
                (*(*ft).drawn_client()).term_type.as_deref().unwrap_or(c""),
            ));
        }
        None
    }
}
unsafe fn format_cb_client_tty(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            return Some(format_callback_copy(
                (*(*ft).drawn_client()).ttyname.as_deref().unwrap_or(c""),
            ));
        }
        None
    }
}
unsafe fn format_cb_client_uid(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut uid: uid_t = 0;
        if !(*ft).drawn_client().is_null() {
            uid = proc_get_peer_uid((*(*ft).drawn_client()).peer_ptr());
            if uid != -(1 as ::core::ffi::c_int) as uid_t {
                return Some(format_printf(
                    c"%ld".as_ptr(),
                    fmt_args![uid as ::core::ffi::c_long],
                ));
            }
        }
        None
    }
}
unsafe fn format_cb_client_user(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut uid: uid_t = 0;
        let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
        if !(*ft).drawn_client().is_null() {
            if (*(*ft).drawn_client()).user.is_some() {
                return Some(format_callback_copy(
                    (*(*ft).drawn_client()).user.as_deref().unwrap_or(c""),
                ));
            }
            uid = proc_get_peer_uid((*(*ft).drawn_client()).peer_ptr());
            if uid != -(1 as ::core::ffi::c_int) as uid_t && {
                pw = getpwuid(uid as __uid_t);
                !pw.is_null()
            } {
                (*(*ft).drawn_client()).user = Some(CStr::from_ptr((*pw).pw_name).to_owned());
                return Some(format_callback_copy(
                    (*(*ft).drawn_client()).user.as_deref().unwrap_or(c""),
                ));
            }
        }
        None
    }
}
unsafe fn format_cb_client_utf8(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            if (*(*ft).drawn_client()).flags & CLIENT_UTF8 as uint64_t != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_client_width(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*(*ft).drawn_client()).tty.sx],
            ));
        }
        None
    }
}
unsafe fn format_cb_client_written(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            return Some(format_printf(
                c"%zu".as_ptr(),
                fmt_args![(*(*ft).drawn_client()).written],
            ));
        }
        None
    }
}
unsafe fn format_cb_client_theme(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            match (*(*ft).drawn_client()).theme {
                THEME_DARK => {
                    return Some(format_callback_copy(c"dark"));
                }
                THEME_LIGHT => {
                    return Some(format_callback_copy(c"light"));
                }
                THEME_UNKNOWN => return None,
                _ => {}
            }
        }
        None
    }
}
unsafe fn format_cb_config_files(_ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut bytes: Vec<u8> = Vec::new();
        for file in &cfg_files {
            bytes.extend_from_slice(file.as_bytes());
            bytes.push(b',');
        }
        bytes.pop();
        Some(CString::new(bytes).expect("a config file path holds no nul"))
    }
}
unsafe fn format_cb_cursor_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).base.mode & MODE_CURSOR != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_cursor_shape(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() && !(*(*ft).pane()).screen().is_null() {
            match (*(*(*ft).pane()).screen()).cstyle {
                SCREEN_CURSOR_BLOCK => {
                    return Some(format_callback_copy(c"block"));
                }
                SCREEN_CURSOR_UNDERLINE => {
                    return Some(format_callback_copy(c"underline"));
                }
                SCREEN_CURSOR_BAR => {
                    return Some(format_callback_copy(c"bar"));
                }
                _ => {
                    return Some(format_callback_copy(c"default"));
                }
            }
        }
        None
    }
}
unsafe fn format_cb_cursor_very_visible(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() && !(*(*ft).pane()).screen().is_null() {
            if (*(*(*ft).pane()).screen()).mode & MODE_CURSOR_VERY_VISIBLE != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_cursor_x(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*(*ft).pane()).base.cx],
            ));
        }
        None
    }
}
unsafe fn format_cb_cursor_y(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*(*ft).pane()).base.cy],
            ));
        }
        None
    }
}
unsafe fn format_cb_cursor_blinking(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() && !(*(*ft).pane()).screen().is_null() {
            if (*(*(*ft).pane()).screen()).mode & MODE_CURSOR_BLINKING != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_history_limit(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*screen_grid_ptr(&mut (*(*ft).pane()).base)).hlimit],
            ));
        }
        None
    }
}
unsafe fn format_cb_history_size(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*screen_grid_ptr(&mut (*(*ft).pane()).base)).hsize],
            ));
        }
        None
    }
}
unsafe fn format_cb_insert_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).base.mode & MODE_INSERT != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_keypad_cursor_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).base.mode & MODE_KCURSOR != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_keypad_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).base.mode & MODE_KKEYPAD != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_loop_last_flag(ft: &format_tree) -> Option<CString> {
    if ft.flags & FORMAT_LAST != 0 {
        return Some(format_callback_copy(c"1"));
    }
    Some(format_callback_copy(c"0"))
}
unsafe fn format_cb_mouse_all_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).base.mode & MODE_MOUSE_ALL != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_mouse_any_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).base.mode & ALL_MOUSE_MODES != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_mouse_button_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).base.mode & MODE_MOUSE_BUTTON != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_mouse_pane(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        if ft.m.valid != 0 {
            wp = cmd_mouse_pane(&ft.m)
                .map_or(::core::ptr::null_mut::<window_pane>(), |(_, _, wp)| wp);
            if !wp.is_null() {
                return Some(format_printf(c"%%%u".as_ptr(), fmt_args![(*wp).id]));
            }
            return None;
        }
        None
    }
}
unsafe fn format_cb_mouse_sgr_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).base.mode & MODE_MOUSE_SGR != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_mouse_standard_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).base.mode & MODE_MOUSE_STANDARD != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_mouse_utf8_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).base.mode & MODE_MOUSE_UTF8 != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_mouse_x(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut x: u_int = 0;
        if ft.m.valid == 0 {
            return None;
        }
        wp = cmd_mouse_pane(&ft.m).map_or(::core::ptr::null_mut::<window_pane>(), |(_, _, wp)| wp);
        if !wp.is_null()
            && match cmd_mouse_at(wp, &ft.m, 0 as ::core::ffi::c_int) {
                Some((at_x, at_y)) => {
                    (x, _) = (at_x, at_y);
                    true
                }
                None => false,
            }
        {
            return Some(format_printf(c"%u".as_ptr(), fmt_args![x]));
        }
        if !(*ft).drawn_client().is_null() && (*(*ft).drawn_client()).tty.flags & TTY_STARTED != 0 {
            if ft.m.statusat == 0 as ::core::ffi::c_int && ft.m.y < ft.m.statuslines {
                return Some(format_printf(c"%u".as_ptr(), fmt_args![ft.m.x]));
            }
            if ft.m.statusat > 0 as ::core::ffi::c_int && ft.m.y >= ft.m.statusat as u_int {
                return Some(format_printf(c"%u".as_ptr(), fmt_args![ft.m.x]));
            }
        }
        None
    }
}
unsafe fn format_cb_mouse_y(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        let mut y: u_int = 0;
        if ft.m.valid == 0 {
            return None;
        }
        wp = cmd_mouse_pane(&ft.m).map_or(::core::ptr::null_mut::<window_pane>(), |(_, _, wp)| wp);
        if !wp.is_null()
            && match cmd_mouse_at(wp, &ft.m, 0 as ::core::ffi::c_int) {
                Some((at_x, at_y)) => {
                    (_, y) = (at_x, at_y);
                    true
                }
                None => false,
            }
        {
            return Some(format_printf(c"%u".as_ptr(), fmt_args![y]));
        }
        if !(*ft).drawn_client().is_null() && (*(*ft).drawn_client()).tty.flags & TTY_STARTED != 0 {
            if ft.m.statusat == 0 as ::core::ffi::c_int && ft.m.y < ft.m.statuslines {
                return Some(format_printf(c"%u".as_ptr(), fmt_args![ft.m.y]));
            }
            if ft.m.statusat > 0 as ::core::ffi::c_int && ft.m.y >= ft.m.statusat as u_int {
                return Some(format_printf(
                    c"%u".as_ptr(),
                    fmt_args![ft.m.y.wrapping_sub(ft.m.statusat as u_int)],
                ));
            }
        }
        None
    }
}
unsafe fn format_cb_next_session_id(_ft: &format_tree) -> Option<CString> {
    unsafe { Some(format_printf(c"$%u".as_ptr(), fmt_args![next_session_id])) }
}
unsafe fn format_cb_origin_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).base.mode & MODE_ORIGIN != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_synchronized_output_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).base.mode & MODE_SYNC != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_pane_active(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*ft).pane() == window_get_active((*(*ft).pane()).window) {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_pane_at_left(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).xoff == 0 as ::core::ffi::c_int {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_pane_at_right(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).xoff + (*(*ft).pane()).sx as ::core::ffi::c_int
                == (*(*(*ft).pane()).window).sx as ::core::ffi::c_int
            {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_pane_bottom(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        if !wp.is_null() {
            return Some(format_printf(
                c"%d".as_ptr(),
                fmt_args![(*wp).yoff + (*wp).sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int],
            ));
        }
        None
    }
}
unsafe fn format_cb_pane_dead(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        if !wp.is_null() {
            if (*wp).fd == -(1 as ::core::ffi::c_int) && (*wp).flags & PANE_STATUSREADY != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_pane_dead_signal(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        if !wp.is_null() {
            if (*wp).flags & PANE_STATUSREADY != 0
                && (((*wp).status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_schar as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int
                    > 0 as ::core::ffi::c_int
            {
                let name = sig2name((*wp).status & 0x7f as ::core::ffi::c_int);
                return Some(format_printf(c"%s".as_ptr(), fmt_args![name.as_ptr()]));
            }
            return None;
        }
        None
    }
}
unsafe fn format_cb_pane_dead_status(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        if !wp.is_null() {
            if (*wp).flags & PANE_STATUSREADY != 0
                && (*wp).status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            {
                return Some(format_printf(
                    c"%d".as_ptr(),
                    fmt_args![
                        ((*wp).status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                    ],
                ));
            }
            return None;
        }
        None
    }
}
unsafe fn format_cb_pane_dead_time(ft: &format_tree) -> Option<timeval> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        if !wp.is_null() && (*wp).flags & PANE_STATUSDRAWN != 0 {
            return Some((*wp).dead_time);
        }
        None
    }
}
unsafe fn format_cb_pane_format(ft: &format_tree) -> Option<CString> {
    if ft.type_0 as ::core::ffi::c_uint
        == FORMAT_TYPE_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return Some(format_callback_copy(c"1"));
    }
    Some(format_callback_copy(c"0"))
}
unsafe fn format_cb_pane_height(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_printf(c"%u".as_ptr(), fmt_args![(*(*ft).pane()).sy]));
        }
        None
    }
}
unsafe fn format_cb_pane_id(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_printf(
                c"%%%u".as_ptr(),
                fmt_args![(*(*ft).pane()).id],
            ));
        }
        None
    }
}
unsafe fn format_cb_pane_index(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null()
            && let (0, idx) = window_pane_index((*ft).pane())
        {
            return Some(format_printf(c"%u".as_ptr(), fmt_args![idx]));
        }
        None
    }
}
unsafe fn format_cb_pane_input_off(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).flags & PANE_INPUTOFF != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_pane_unseen_changes(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).flags & PANE_UNSEENCHANGES != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_pane_key_mode(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() && !(*(*ft).pane()).screen().is_null() {
            match (*(*(*ft).pane()).screen()).mode & EXTENDED_KEY_MODES {
                MODE_KEYS_EXTENDED => {
                    return Some(format_callback_copy(c"Ext 1"));
                }
                MODE_KEYS_EXTENDED_2 => {
                    return Some(format_callback_copy(c"Ext 2"));
                }
                _ => {
                    return Some(format_callback_copy(c"VT10x"));
                }
            }
        }
        None
    }
}
unsafe fn format_cb_pane_last(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*ft).pane() == window_pane_stack_first((*(*ft).pane()).window, PaneStack::LastUsed)
            {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_pane_left(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_printf(
                c"%d".as_ptr(),
                fmt_args![(*(*ft).pane()).xoff],
            ));
        }
        None
    }
}
unsafe fn format_cb_pane_marked(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if server_check_marked() != 0 && marked_pane.pane() == (*ft).pane() {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_pane_marked_set(ft: &format_tree) -> Option<CString> {
    if !(*ft).pane().is_null() {
        if server_check_marked() != 0 {
            return Some(format_callback_copy(c"1"));
        }
        return Some(format_callback_copy(c"0"));
    }
    None
}
unsafe fn format_cb_pane_mode(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
        if !(*ft).pane().is_null() {
            wme = window_pane_current_mode((*ft).pane());
            if !wme.is_null() {
                return Some(format_callback_copy(CStr::from_ptr(
                    (*wme).mode().name().as_ptr(),
                )));
            }
            return None;
        }
        None
    }
}
unsafe fn format_cb_pane_path(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).base.path.is_none() {
                return Some(format_callback_copy(c""));
            }
            return Some(format_callback_copy(
                (*(*ft).pane()).base.path.as_deref().unwrap_or(c""),
            ));
        }
        None
    }
}
unsafe fn format_cb_pane_pid(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_printf(
                c"%ld".as_ptr(),
                fmt_args![(*(*ft).pane()).pid as ::core::ffi::c_long],
            ));
        }
        None
    }
}
unsafe fn format_cb_pane_pipe(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).pipe_fd != -(1 as ::core::ffi::c_int) {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_pane_pipe_pid(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() && (*(*ft).pane()).pipe_fd != -(1 as ::core::ffi::c_int) {
            return Some(xasprintf(
                c"%ld".as_ptr(),
                fmt_args![(*(*ft).pane()).pipe_pid as ::core::ffi::c_long],
            ));
        }
        None
    }
}
unsafe fn format_cb_pane_pb_progress(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(xasprintf(
                c"%d".as_ptr(),
                fmt_args![(*(*ft).pane()).base.progress_bar.progress],
            ));
        }
        None
    }
}
unsafe fn format_cb_pane_pb_state(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            match (*(*ft).pane()).base.progress_bar.state {
                PROGRESS_BAR_HIDDEN => {
                    return Some(format_callback_copy(c"hidden"));
                }
                PROGRESS_BAR_NORMAL => {
                    return Some(format_callback_copy(c"normal"));
                }
                PROGRESS_BAR_ERROR => {
                    return Some(format_callback_copy(c"error"));
                }
                PROGRESS_BAR_INDETERMINATE => {
                    return Some(format_callback_copy(c"indeterminate"));
                }
                PROGRESS_BAR_PAUSED => {
                    return Some(format_callback_copy(c"paused"));
                }
                _ => {}
            }
        }
        None
    }
}
unsafe fn format_cb_pane_right(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        if !wp.is_null() {
            return Some(format_printf(
                c"%d".as_ptr(),
                fmt_args![(*wp).xoff + (*wp).sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int],
            ));
        }
        None
    }
}
unsafe fn format_cb_pane_search_string(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).searchstr.is_none() {
                return Some(format_callback_copy(c""));
            }
            return Some(format_callback_copy(
                (*(*ft).pane()).searchstr.as_deref().unwrap_or(c""),
            ));
        }
        None
    }
}
unsafe fn format_cb_pane_synchronized(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if options_get_number((*(*ft).pane()).options_ptr(), c"synchronize-panes".as_ptr()) != 0
            {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_pane_title(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_callback_copy(
                (*(*ft).pane()).base.title.as_deref().unwrap_or(c""),
            ));
        }
        None
    }
}
unsafe fn format_cb_pane_top(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_printf(
                c"%d".as_ptr(),
                fmt_args![(*(*ft).pane()).yoff],
            ));
        }
        None
    }
}
unsafe fn format_cb_pane_tty(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_callback_copy(CStr::from_ptr(
                &raw mut (*(*ft).pane()).tty as *mut ::core::ffi::c_char,
            )));
        }
        None
    }
}
unsafe fn format_cb_pane_width(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_printf(c"%u".as_ptr(), fmt_args![(*(*ft).pane()).sx]));
        }
        None
    }
}
unsafe fn format_cb_pane_x(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_printf(
                c"%d".as_ptr(),
                fmt_args![(*(*ft).pane()).xoff],
            ));
        }
        None
    }
}
unsafe fn format_cb_pane_y(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_printf(
                c"%d".as_ptr(),
                fmt_args![(*(*ft).pane()).yoff],
            ));
        }
        None
    }
}
unsafe fn format_cb_pane_z(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null()
            && let (0, idx) = window_pane_zindex((*ft).pane())
        {
            return Some(format_printf(c"%u".as_ptr(), fmt_args![idx]));
        }
        None
    }
}
unsafe fn format_cb_pane_zoomed_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wp: *mut window_pane = (*ft).pane();
        if !wp.is_null() {
            if (*wp).flags & PANE_ZOOMED != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_scroll_region_lower(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*(*ft).pane()).base.rlower],
            ));
        }
        None
    }
}
unsafe fn format_cb_scroll_region_upper(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*(*ft).pane()).base.rupper],
            ));
        }
        None
    }
}
unsafe fn format_cb_server_sessions(_ft: &format_tree) -> Option<CString> {
    unsafe {
        let n = session_owners().len() as u_int;
        Some(format_printf(c"%u".as_ptr(), fmt_args![n]))
    }
}
unsafe fn format_cb_session_active(ft: &format_tree) -> Option<CString> {
    unsafe {
        if (*ft).session().is_null() || (*ft).drawn_client().is_null() {
            return None;
        }
        if (*(*ft).drawn_client()).session == (*ft).session() {
            return Some(format_callback_copy(c"1"));
        }
        Some(format_callback_copy(c"0"))
    }
}
unsafe fn format_cb_session_activity_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        if !(*ft).session().is_null() {
            wl = winlinks_first(&mut (*(*ft).session()).windows);
            if !wl.is_null() {
                if (*(*ft).winlink()).flags & WINLINK_ACTIVITY != 0 {
                    return Some(format_callback_copy(c"1"));
                }
                return Some(format_callback_copy(c"0"));
            }
        }
        None
    }
}
unsafe fn format_cb_session_bell_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        if !(*ft).session().is_null() {
            wl = winlinks_first(&mut (*(*ft).session()).windows);
            if !wl.is_null() {
                if (*wl).flags & WINLINK_BELL != 0 {
                    return Some(format_callback_copy(c"1"));
                }
                return Some(format_callback_copy(c"0"));
            }
        }
        None
    }
}
unsafe fn format_cb_session_silence_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        if !(*ft).session().is_null() {
            wl = winlinks_first(&mut (*(*ft).session()).windows);
            if !wl.is_null() {
                if (*(*ft).winlink()).flags & WINLINK_SILENCE != 0 {
                    return Some(format_callback_copy(c"1"));
                }
                return Some(format_callback_copy(c"0"));
            }
        }
        None
    }
}
unsafe fn format_cb_session_attached(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).session().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![session_attached((*ft).session())],
            ));
        }
        None
    }
}
unsafe fn format_cb_session_format(ft: &format_tree) -> Option<CString> {
    if ft.type_0 as ::core::ffi::c_uint
        == FORMAT_TYPE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return Some(format_callback_copy(c"1"));
    }
    Some(format_callback_copy(c"0"))
}
unsafe fn format_cb_session_group(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
        if !(*ft).session().is_null() && {
            sg = session_group_contains((*ft).session());
            !sg.is_null()
        } {
            return Some(format_callback_copy(session_group_name(sg)));
        }
        None
    }
}
unsafe fn format_cb_session_group_attached(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
        if !(*ft).session().is_null() && {
            sg = session_group_contains((*ft).session());
            !sg.is_null()
        } {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![session_group_attached_count(sg)],
            ));
        }
        None
    }
}
unsafe fn format_cb_session_group_many_attached(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
        if !(*ft).session().is_null() && {
            sg = session_group_contains((*ft).session());
            !sg.is_null()
        } {
            if session_group_attached_count(sg) > 1 as u_int {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_session_group_size(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
        if !(*ft).session().is_null() && {
            sg = session_group_contains((*ft).session());
            !sg.is_null()
        } {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![session_group_count(sg)],
            ));
        }
        None
    }
}
unsafe fn format_cb_session_grouped(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).session().is_null() {
            if !session_group_contains((*ft).session()).is_null() {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_session_id(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).session().is_null() {
            return Some(format_printf(
                c"$%u".as_ptr(),
                fmt_args![session_id((*ft).session())],
            ));
        }
        None
    }
}
unsafe fn format_cb_session_many_attached(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).session().is_null() {
            if session_attached((*ft).session()) > 1 as u_int {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_session_marked(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).session().is_null() {
            if server_check_marked() != 0 && marked_pane.session() == (*ft).session() {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_session_name(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).session().is_null() {
            return Some(format_callback_copy(CStr::from_ptr(session_name(
                (*ft).session(),
            ))));
        }
        None
    }
}
unsafe fn format_cb_session_path(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).session().is_null() {
            return session_cwd((*ft).session()).map(format_callback_copy);
        }
        None
    }
}
unsafe fn format_cb_session_windows(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).session().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![winlink_count(&(*(*ft).session()).windows)],
            ));
        }
        None
    }
}
unsafe fn format_cb_socket_path(_ft: &format_tree) -> Option<CString> {
    unsafe { socket_path.as_deref().map(format_callback_copy) }
}
fn format_cb_version(_ft: &format_tree) -> Option<CString> {
    Some(format_callback_copy(getversion()))
}
fn format_cb_sixel_support(_ft: &format_tree) -> Option<CString> {
    Some(format_callback_copy(c"0"))
}
unsafe fn format_cb_active_window_index(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).session().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*session_get_curw((*ft).session())).idx],
            ));
        }
        None
    }
}
unsafe fn format_cb_last_window_index(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        if !(*ft).session().is_null() {
            wl = winlinks_last(&mut (*(*ft).session()).windows);
            return Some(format_printf(c"%u".as_ptr(), fmt_args![(*wl).idx]));
        }
        None
    }
}
unsafe fn format_cb_window_active(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).winlink().is_null() {
            if (*ft).winlink() == session_get_curw((*(*ft).winlink()).session()) {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_window_activity_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).winlink().is_null() {
            if (*(*ft).winlink()).flags & WINLINK_ACTIVITY != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_window_bell_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).winlink().is_null() {
            if (*(*ft).winlink()).flags & WINLINK_BELL != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_window_bigger(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            let (window_bigger, ..) = tty_window_offset(&(*(*ft).drawn_client()).tty);
            if window_bigger != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_window_cell_height(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).window().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*(*ft).window()).ypixel],
            ));
        }
        None
    }
}
unsafe fn format_cb_window_cell_width(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).window().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*(*ft).window()).xpixel],
            ));
        }
        None
    }
}
unsafe fn format_cb_window_end_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).winlink().is_null() {
            if (*ft).winlink() == winlinks_last(&mut (*(*(*ft).winlink()).session()).windows) {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_window_flags(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).winlink().is_null() {
            return Some(window_printable_flags(
                (*ft).winlink(),
                1 as ::core::ffi::c_int,
            ));
        }
        None
    }
}
unsafe fn format_cb_window_format(ft: &format_tree) -> Option<CString> {
    if ft.type_0 as ::core::ffi::c_uint
        == FORMAT_TYPE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return Some(format_callback_copy(c"1"));
    }
    Some(format_callback_copy(c"0"))
}
unsafe fn format_cb_window_height(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).window().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*(*ft).window()).sy],
            ));
        }
        None
    }
}
unsafe fn format_cb_window_id(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).window().is_null() {
            return Some(format_printf(
                c"@%u".as_ptr(),
                fmt_args![(*(*ft).window()).id],
            ));
        }
        None
    }
}
unsafe fn format_cb_window_index(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).winlink().is_null() {
            return Some(format_printf(
                c"%d".as_ptr(),
                fmt_args![(*(*ft).winlink()).idx],
            ));
        }
        None
    }
}
unsafe fn format_cb_window_last_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).winlink().is_null() {
            if (*(*(*ft).winlink()).session()).lastw.first() == Some(&(*(*ft).winlink()).idx) {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_window_linked(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if !(*ft).winlink().is_null() {
            for s_ref in session_owners() {
                let s = s_ref.as_ptr();
                wl = winlinks_first(&mut (*s).windows);
                while !wl.is_null() {
                    if (*wl).window() == (*(*ft).winlink()).window() {
                        if found != 0 {
                            return Some(format_callback_copy(c"1"));
                        }
                        found = 1 as ::core::ffi::c_int;
                    }
                    wl = winlinks_after(wl);
                }
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_window_linked_sessions(ft: &format_tree) -> Option<CString> {
    unsafe {
        let mut w: *mut window = ::core::ptr::null_mut::<window>();
        let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
        let mut s: *mut session = ::core::ptr::null_mut::<session>();
        let mut n: u_int = 0 as u_int;
        if (*ft).winlink().is_null() {
            return None;
        }
        w = (*(*ft).winlink()).window();
        sg = session_groups_first();
        while !sg.is_null() {
            s = group_walk(sg)
                .next()
                .unwrap_or(::core::ptr::null_mut::<session>());
            if !s.is_null() && !winlink_find_by_window(&mut (*s).windows, w).is_null() {
                n = n.wrapping_add(1);
            }
            sg = session_groups_after(sg);
        }
        for s_ref in session_owners() {
            let s = s_ref.as_ptr();
            if session_group_contains(s).is_null()
                && !winlink_find_by_window(&mut (*s).windows, w).is_null()
            {
                n = n.wrapping_add(1);
            }
        }
        Some(format_printf(c"%u".as_ptr(), fmt_args![n]))
    }
}
unsafe fn format_cb_window_marked_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).winlink().is_null() {
            if server_check_marked() != 0 && marked_pane.winlink() == (*ft).winlink() {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_window_name(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).window().is_null() {
            return Some(format_printf(
                c"%s".as_ptr(),
                fmt_args![(*(*ft).window()).name.as_deref()],
            ));
        }
        None
    }
}
unsafe fn format_cb_window_offset_x(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            let (window_bigger, ox, ..) = tty_window_offset(&(*(*ft).drawn_client()).tty);
            if window_bigger != 0 {
                return Some(format_printf(c"%u".as_ptr(), fmt_args![ox]));
            }
            return None;
        }
        None
    }
}
unsafe fn format_cb_window_offset_y(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            let (window_bigger, _ox, oy, ..) = tty_window_offset(&(*(*ft).drawn_client()).tty);
            if window_bigger != 0 {
                return Some(format_printf(c"%u".as_ptr(), fmt_args![oy]));
            }
            return None;
        }
        None
    }
}
unsafe fn format_cb_window_panes(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).window().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![window_count_panes((*ft).window(), 1 as ::core::ffi::c_int)],
            ));
        }
        None
    }
}
unsafe fn format_cb_window_raw_flags(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).winlink().is_null() {
            return Some(window_printable_flags(
                (*ft).winlink(),
                0 as ::core::ffi::c_int,
            ));
        }
        None
    }
}
unsafe fn format_cb_window_silence_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).winlink().is_null() {
            if (*(*ft).winlink()).flags & WINLINK_SILENCE != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_window_start_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).winlink().is_null() {
            if (*ft).winlink() == winlinks_first(&mut (*(*(*ft).winlink()).session()).windows) {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_window_width(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).window().is_null() {
            return Some(format_printf(
                c"%u".as_ptr(),
                fmt_args![(*(*ft).window()).sx],
            ));
        }
        None
    }
}
unsafe fn format_cb_window_zoomed_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).window().is_null() {
            if (*(*ft).window()).flags & WINDOW_ZOOMED != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_wrap_flag(ft: &format_tree) -> Option<CString> {
    unsafe {
        if !(*ft).pane().is_null() {
            if (*(*ft).pane()).base.mode & MODE_WRAP != 0 {
                return Some(format_callback_copy(c"1"));
            }
            return Some(format_callback_copy(c"0"));
        }
        None
    }
}
unsafe fn format_cb_buffer_created(ft: &format_tree) -> Option<timeval> {
    unsafe {
        if !(*ft).buffer().is_null() {
            return Some(timeval {
                tv_sec: paste_buffer_created(&*(*ft).buffer()) as __time_t,
                tv_usec: 0 as __suseconds_t,
            });
        }
        None
    }
}
unsafe fn format_cb_client_activity(ft: &format_tree) -> Option<timeval> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            return Some((*(*ft).drawn_client()).activity_time);
        }
        None
    }
}
unsafe fn format_cb_client_created(ft: &format_tree) -> Option<timeval> {
    unsafe {
        if !(*ft).drawn_client().is_null() {
            return Some((*(*ft).drawn_client()).creation_time);
        }
        None
    }
}
unsafe fn format_cb_session_activity(ft: &format_tree) -> Option<timeval> {
    unsafe {
        if !(*ft).session().is_null() {
            return Some(session_activity_time((*ft).session()));
        }
        None
    }
}
unsafe fn format_cb_session_created(ft: &format_tree) -> Option<timeval> {
    unsafe {
        if !(*ft).session().is_null() {
            return Some((*(*ft).session()).creation_time);
        }
        None
    }
}
unsafe fn format_cb_session_last_attached(ft: &format_tree) -> Option<timeval> {
    unsafe {
        if !(*ft).session().is_null() {
            return Some((*(*ft).session()).last_attached_time);
        }
        None
    }
}
unsafe fn format_cb_start_time(_ft: &format_tree) -> Option<timeval> {
    unsafe { Some(start_time) }
}
unsafe fn format_cb_window_activity(ft: &format_tree) -> Option<timeval> {
    unsafe {
        if !(*ft).window().is_null() {
            return Some((*(*ft).window()).activity_time);
        }
        None
    }
}
unsafe fn format_cb_buffer_mode_format(_ft: &format_tree) -> Option<CString> {
    WindowMode::Buffer
        .default_format()
        .map(format_callback_copy)
}
unsafe fn format_cb_client_mode_format(_ft: &format_tree) -> Option<CString> {
    WindowMode::Client
        .default_format()
        .map(format_callback_copy)
}
unsafe fn format_cb_tree_mode_format(_ft: &format_tree) -> Option<CString> {
    WindowMode::Tree.default_format().map(format_callback_copy)
}
unsafe fn format_cb_uid(_ft: &format_tree) -> Option<CString> {
    unsafe {
        Some(format_printf(
            c"%ld".as_ptr(),
            fmt_args![getuid() as ::core::ffi::c_long],
        ))
    }
}
unsafe fn format_cb_user(_ft: &format_tree) -> Option<CString> {
    unsafe {
        static CACHED_USER: OnceLock<CString> = OnceLock::new();
        if let Some(value) = CACHED_USER.get() {
            return Some(value.clone());
        }
        let pw: *mut passwd = getpwuid(getuid());
        if pw.is_null() {
            return None;
        }
        let name = CStr::from_ptr((*pw).pw_name).to_owned();
        Some(CACHED_USER.get_or_init(|| name).clone())
    }
}
static format_table: [format_table_entry; 195] = {
    [
        format_table_entry {
            key: c"active_window_index",
            cb: FormatTableCallback::String(format_cb_active_window_index),
        },
        format_table_entry {
            key: c"alternate_on",
            cb: FormatTableCallback::String(format_cb_alternate_on),
        },
        format_table_entry {
            key: c"alternate_saved_x",
            cb: FormatTableCallback::String(format_cb_alternate_saved_x),
        },
        format_table_entry {
            key: c"alternate_saved_y",
            cb: FormatTableCallback::String(format_cb_alternate_saved_y),
        },
        format_table_entry {
            key: c"bracket_paste_flag",
            cb: FormatTableCallback::String(format_cb_bracket_paste_flag),
        },
        format_table_entry {
            key: c"buffer_created",
            cb: FormatTableCallback::Time(format_cb_buffer_created),
        },
        format_table_entry {
            key: c"buffer_full",
            cb: FormatTableCallback::String(format_cb_buffer_full),
        },
        format_table_entry {
            key: c"buffer_mode_format",
            cb: FormatTableCallback::String(format_cb_buffer_mode_format),
        },
        format_table_entry {
            key: c"buffer_name",
            cb: FormatTableCallback::String(format_cb_buffer_name),
        },
        format_table_entry {
            key: c"buffer_sample",
            cb: FormatTableCallback::String(format_cb_buffer_sample),
        },
        format_table_entry {
            key: c"buffer_size",
            cb: FormatTableCallback::String(format_cb_buffer_size),
        },
        format_table_entry {
            key: c"client_activity",
            cb: FormatTableCallback::Time(format_cb_client_activity),
        },
        format_table_entry {
            key: c"client_cell_height",
            cb: FormatTableCallback::String(format_cb_client_cell_height),
        },
        format_table_entry {
            key: c"client_cell_width",
            cb: FormatTableCallback::String(format_cb_client_cell_width),
        },
        format_table_entry {
            key: c"client_control_mode",
            cb: FormatTableCallback::String(format_cb_client_control_mode),
        },
        format_table_entry {
            key: c"client_created",
            cb: FormatTableCallback::Time(format_cb_client_created),
        },
        format_table_entry {
            key: c"client_discarded",
            cb: FormatTableCallback::String(format_cb_client_discarded),
        },
        format_table_entry {
            key: c"client_flags",
            cb: FormatTableCallback::String(format_cb_client_flags),
        },
        format_table_entry {
            key: c"client_height",
            cb: FormatTableCallback::String(format_cb_client_height),
        },
        format_table_entry {
            key: c"client_key_table",
            cb: FormatTableCallback::String(format_cb_client_key_table),
        },
        format_table_entry {
            key: c"client_last_session",
            cb: FormatTableCallback::String(format_cb_client_last_session),
        },
        format_table_entry {
            key: c"client_mode_format",
            cb: FormatTableCallback::String(format_cb_client_mode_format),
        },
        format_table_entry {
            key: c"client_name",
            cb: FormatTableCallback::String(format_cb_client_name),
        },
        format_table_entry {
            key: c"client_pid",
            cb: FormatTableCallback::String(format_cb_client_pid),
        },
        format_table_entry {
            key: c"client_prefix",
            cb: FormatTableCallback::String(format_cb_client_prefix),
        },
        format_table_entry {
            key: c"client_readonly",
            cb: FormatTableCallback::String(format_cb_client_readonly),
        },
        format_table_entry {
            key: c"client_session",
            cb: FormatTableCallback::String(format_cb_client_session),
        },
        format_table_entry {
            key: c"client_termfeatures",
            cb: FormatTableCallback::String(format_cb_client_termfeatures),
        },
        format_table_entry {
            key: c"client_termname",
            cb: FormatTableCallback::String(format_cb_client_termname),
        },
        format_table_entry {
            key: c"client_termtype",
            cb: FormatTableCallback::String(format_cb_client_termtype),
        },
        format_table_entry {
            key: c"client_theme",
            cb: FormatTableCallback::String(format_cb_client_theme),
        },
        format_table_entry {
            key: c"client_tty",
            cb: FormatTableCallback::String(format_cb_client_tty),
        },
        format_table_entry {
            key: c"client_uid",
            cb: FormatTableCallback::String(format_cb_client_uid),
        },
        format_table_entry {
            key: c"client_user",
            cb: FormatTableCallback::String(format_cb_client_user),
        },
        format_table_entry {
            key: c"client_utf8",
            cb: FormatTableCallback::String(format_cb_client_utf8),
        },
        format_table_entry {
            key: c"client_width",
            cb: FormatTableCallback::String(format_cb_client_width),
        },
        format_table_entry {
            key: c"client_written",
            cb: FormatTableCallback::String(format_cb_client_written),
        },
        format_table_entry {
            key: c"config_files",
            cb: FormatTableCallback::String(format_cb_config_files),
        },
        format_table_entry {
            key: c"cursor_blinking",
            cb: FormatTableCallback::String(format_cb_cursor_blinking),
        },
        format_table_entry {
            key: c"cursor_character",
            cb: FormatTableCallback::String(format_cb_cursor_character),
        },
        format_table_entry {
            key: c"cursor_colour",
            cb: FormatTableCallback::String(format_cb_cursor_colour),
        },
        format_table_entry {
            key: c"cursor_flag",
            cb: FormatTableCallback::String(format_cb_cursor_flag),
        },
        format_table_entry {
            key: c"cursor_shape",
            cb: FormatTableCallback::String(format_cb_cursor_shape),
        },
        format_table_entry {
            key: c"cursor_very_visible",
            cb: FormatTableCallback::String(format_cb_cursor_very_visible),
        },
        format_table_entry {
            key: c"cursor_x",
            cb: FormatTableCallback::String(format_cb_cursor_x),
        },
        format_table_entry {
            key: c"cursor_y",
            cb: FormatTableCallback::String(format_cb_cursor_y),
        },
        format_table_entry {
            key: c"history_all_bytes",
            cb: FormatTableCallback::String(format_cb_history_all_bytes),
        },
        format_table_entry {
            key: c"history_bytes",
            cb: FormatTableCallback::String(format_cb_history_bytes),
        },
        format_table_entry {
            key: c"history_limit",
            cb: FormatTableCallback::String(format_cb_history_limit),
        },
        format_table_entry {
            key: c"history_size",
            cb: FormatTableCallback::String(format_cb_history_size),
        },
        format_table_entry {
            key: c"host",
            cb: FormatTableCallback::String(format_cb_host),
        },
        format_table_entry {
            key: c"host_short",
            cb: FormatTableCallback::String(format_cb_host_short),
        },
        format_table_entry {
            key: c"insert_flag",
            cb: FormatTableCallback::String(format_cb_insert_flag),
        },
        format_table_entry {
            key: c"keypad_cursor_flag",
            cb: FormatTableCallback::String(format_cb_keypad_cursor_flag),
        },
        format_table_entry {
            key: c"keypad_flag",
            cb: FormatTableCallback::String(format_cb_keypad_flag),
        },
        format_table_entry {
            key: c"last_window_index",
            cb: FormatTableCallback::String(format_cb_last_window_index),
        },
        format_table_entry {
            key: c"loop_last_flag",
            cb: FormatTableCallback::String(format_cb_loop_last_flag),
        },
        format_table_entry {
            key: c"mouse_all_flag",
            cb: FormatTableCallback::String(format_cb_mouse_all_flag),
        },
        format_table_entry {
            key: c"mouse_any_flag",
            cb: FormatTableCallback::String(format_cb_mouse_any_flag),
        },
        format_table_entry {
            key: c"mouse_button_flag",
            cb: FormatTableCallback::String(format_cb_mouse_button_flag),
        },
        format_table_entry {
            key: c"mouse_hyperlink",
            cb: FormatTableCallback::String(format_cb_mouse_hyperlink),
        },
        format_table_entry {
            key: c"mouse_line",
            cb: FormatTableCallback::String(format_cb_mouse_line),
        },
        format_table_entry {
            key: c"mouse_pane",
            cb: FormatTableCallback::String(format_cb_mouse_pane),
        },
        format_table_entry {
            key: c"mouse_sgr_flag",
            cb: FormatTableCallback::String(format_cb_mouse_sgr_flag),
        },
        format_table_entry {
            key: c"mouse_standard_flag",
            cb: FormatTableCallback::String(format_cb_mouse_standard_flag),
        },
        format_table_entry {
            key: c"mouse_status_line",
            cb: FormatTableCallback::String(format_cb_mouse_status_line),
        },
        format_table_entry {
            key: c"mouse_status_range",
            cb: FormatTableCallback::String(format_cb_mouse_status_range),
        },
        format_table_entry {
            key: c"mouse_utf8_flag",
            cb: FormatTableCallback::String(format_cb_mouse_utf8_flag),
        },
        format_table_entry {
            key: c"mouse_word",
            cb: FormatTableCallback::String(format_cb_mouse_word),
        },
        format_table_entry {
            key: c"mouse_x",
            cb: FormatTableCallback::String(format_cb_mouse_x),
        },
        format_table_entry {
            key: c"mouse_y",
            cb: FormatTableCallback::String(format_cb_mouse_y),
        },
        format_table_entry {
            key: c"next_session_id",
            cb: FormatTableCallback::String(format_cb_next_session_id),
        },
        format_table_entry {
            key: c"origin_flag",
            cb: FormatTableCallback::String(format_cb_origin_flag),
        },
        format_table_entry {
            key: c"pane_active",
            cb: FormatTableCallback::String(format_cb_pane_active),
        },
        format_table_entry {
            key: c"pane_at_bottom",
            cb: FormatTableCallback::String(format_cb_pane_at_bottom),
        },
        format_table_entry {
            key: c"pane_at_left",
            cb: FormatTableCallback::String(format_cb_pane_at_left),
        },
        format_table_entry {
            key: c"pane_at_right",
            cb: FormatTableCallback::String(format_cb_pane_at_right),
        },
        format_table_entry {
            key: c"pane_at_top",
            cb: FormatTableCallback::String(format_cb_pane_at_top),
        },
        format_table_entry {
            key: c"pane_bg",
            cb: FormatTableCallback::String(format_cb_pane_bg),
        },
        format_table_entry {
            key: c"pane_bottom",
            cb: FormatTableCallback::String(format_cb_pane_bottom),
        },
        format_table_entry {
            key: c"pane_current_command",
            cb: FormatTableCallback::String(format_cb_current_command),
        },
        format_table_entry {
            key: c"pane_current_path",
            cb: FormatTableCallback::String(format_cb_current_path),
        },
        format_table_entry {
            key: c"pane_dead",
            cb: FormatTableCallback::String(format_cb_pane_dead),
        },
        format_table_entry {
            key: c"pane_dead_signal",
            cb: FormatTableCallback::String(format_cb_pane_dead_signal),
        },
        format_table_entry {
            key: c"pane_dead_status",
            cb: FormatTableCallback::String(format_cb_pane_dead_status),
        },
        format_table_entry {
            key: c"pane_dead_time",
            cb: FormatTableCallback::Time(format_cb_pane_dead_time),
        },
        format_table_entry {
            key: c"pane_fg",
            cb: FormatTableCallback::String(format_cb_pane_fg),
        },
        format_table_entry {
            key: c"pane_flags",
            cb: FormatTableCallback::String(format_cb_pane_flags),
        },
        format_table_entry {
            key: c"pane_floating_flag",
            cb: FormatTableCallback::String(format_cb_pane_floating_flag),
        },
        format_table_entry {
            key: c"pane_format",
            cb: FormatTableCallback::String(format_cb_pane_format),
        },
        format_table_entry {
            key: c"pane_height",
            cb: FormatTableCallback::String(format_cb_pane_height),
        },
        format_table_entry {
            key: c"pane_id",
            cb: FormatTableCallback::String(format_cb_pane_id),
        },
        format_table_entry {
            key: c"pane_in_mode",
            cb: FormatTableCallback::String(format_cb_pane_in_mode),
        },
        format_table_entry {
            key: c"pane_index",
            cb: FormatTableCallback::String(format_cb_pane_index),
        },
        format_table_entry {
            key: c"pane_input_off",
            cb: FormatTableCallback::String(format_cb_pane_input_off),
        },
        format_table_entry {
            key: c"pane_key_mode",
            cb: FormatTableCallback::String(format_cb_pane_key_mode),
        },
        format_table_entry {
            key: c"pane_last",
            cb: FormatTableCallback::String(format_cb_pane_last),
        },
        format_table_entry {
            key: c"pane_left",
            cb: FormatTableCallback::String(format_cb_pane_left),
        },
        format_table_entry {
            key: c"pane_marked",
            cb: FormatTableCallback::String(format_cb_pane_marked),
        },
        format_table_entry {
            key: c"pane_marked_set",
            cb: FormatTableCallback::String(format_cb_pane_marked_set),
        },
        format_table_entry {
            key: c"pane_mode",
            cb: FormatTableCallback::String(format_cb_pane_mode),
        },
        format_table_entry {
            key: c"pane_path",
            cb: FormatTableCallback::String(format_cb_pane_path),
        },
        format_table_entry {
            key: c"pane_pb_progress",
            cb: FormatTableCallback::String(format_cb_pane_pb_progress),
        },
        format_table_entry {
            key: c"pane_pb_state",
            cb: FormatTableCallback::String(format_cb_pane_pb_state),
        },
        format_table_entry {
            key: c"pane_pid",
            cb: FormatTableCallback::String(format_cb_pane_pid),
        },
        format_table_entry {
            key: c"pane_pipe",
            cb: FormatTableCallback::String(format_cb_pane_pipe),
        },
        format_table_entry {
            key: c"pane_pipe_pid",
            cb: FormatTableCallback::String(format_cb_pane_pipe_pid),
        },
        format_table_entry {
            key: c"pane_right",
            cb: FormatTableCallback::String(format_cb_pane_right),
        },
        format_table_entry {
            key: c"pane_search_string",
            cb: FormatTableCallback::String(format_cb_pane_search_string),
        },
        format_table_entry {
            key: c"pane_start_command",
            cb: FormatTableCallback::String(format_cb_start_command),
        },
        format_table_entry {
            key: c"pane_start_path",
            cb: FormatTableCallback::String(format_cb_start_path),
        },
        format_table_entry {
            key: c"pane_synchronized",
            cb: FormatTableCallback::String(format_cb_pane_synchronized),
        },
        format_table_entry {
            key: c"pane_tabs",
            cb: FormatTableCallback::String(format_cb_pane_tabs),
        },
        format_table_entry {
            key: c"pane_title",
            cb: FormatTableCallback::String(format_cb_pane_title),
        },
        format_table_entry {
            key: c"pane_top",
            cb: FormatTableCallback::String(format_cb_pane_top),
        },
        format_table_entry {
            key: c"pane_tty",
            cb: FormatTableCallback::String(format_cb_pane_tty),
        },
        format_table_entry {
            key: c"pane_unseen_changes",
            cb: FormatTableCallback::String(format_cb_pane_unseen_changes),
        },
        format_table_entry {
            key: c"pane_width",
            cb: FormatTableCallback::String(format_cb_pane_width),
        },
        format_table_entry {
            key: c"pane_x",
            cb: FormatTableCallback::String(format_cb_pane_x),
        },
        format_table_entry {
            key: c"pane_y",
            cb: FormatTableCallback::String(format_cb_pane_y),
        },
        format_table_entry {
            key: c"pane_z",
            cb: FormatTableCallback::String(format_cb_pane_z),
        },
        format_table_entry {
            key: c"pane_zoomed_flag",
            cb: FormatTableCallback::String(format_cb_pane_zoomed_flag),
        },
        format_table_entry {
            key: c"pid",
            cb: FormatTableCallback::String(format_cb_pid),
        },
        format_table_entry {
            key: c"scroll_region_lower",
            cb: FormatTableCallback::String(format_cb_scroll_region_lower),
        },
        format_table_entry {
            key: c"scroll_region_upper",
            cb: FormatTableCallback::String(format_cb_scroll_region_upper),
        },
        format_table_entry {
            key: c"server_sessions",
            cb: FormatTableCallback::String(format_cb_server_sessions),
        },
        format_table_entry {
            key: c"session_active",
            cb: FormatTableCallback::String(format_cb_session_active),
        },
        format_table_entry {
            key: c"session_activity",
            cb: FormatTableCallback::Time(format_cb_session_activity),
        },
        format_table_entry {
            key: c"session_activity_flag",
            cb: FormatTableCallback::String(format_cb_session_activity_flag),
        },
        format_table_entry {
            key: c"session_alert",
            cb: FormatTableCallback::String(format_cb_session_alert),
        },
        format_table_entry {
            key: c"session_alerts",
            cb: FormatTableCallback::String(format_cb_session_alerts),
        },
        format_table_entry {
            key: c"session_attached",
            cb: FormatTableCallback::String(format_cb_session_attached),
        },
        format_table_entry {
            key: c"session_attached_list",
            cb: FormatTableCallback::String(format_cb_session_attached_list),
        },
        format_table_entry {
            key: c"session_bell_flag",
            cb: FormatTableCallback::String(format_cb_session_bell_flag),
        },
        format_table_entry {
            key: c"session_created",
            cb: FormatTableCallback::Time(format_cb_session_created),
        },
        format_table_entry {
            key: c"session_format",
            cb: FormatTableCallback::String(format_cb_session_format),
        },
        format_table_entry {
            key: c"session_group",
            cb: FormatTableCallback::String(format_cb_session_group),
        },
        format_table_entry {
            key: c"session_group_attached",
            cb: FormatTableCallback::String(format_cb_session_group_attached),
        },
        format_table_entry {
            key: c"session_group_attached_list",
            cb: FormatTableCallback::String(format_cb_session_group_attached_list),
        },
        format_table_entry {
            key: c"session_group_list",
            cb: FormatTableCallback::String(format_cb_session_group_list),
        },
        format_table_entry {
            key: c"session_group_many_attached",
            cb: FormatTableCallback::String(format_cb_session_group_many_attached),
        },
        format_table_entry {
            key: c"session_group_size",
            cb: FormatTableCallback::String(format_cb_session_group_size),
        },
        format_table_entry {
            key: c"session_grouped",
            cb: FormatTableCallback::String(format_cb_session_grouped),
        },
        format_table_entry {
            key: c"session_id",
            cb: FormatTableCallback::String(format_cb_session_id),
        },
        format_table_entry {
            key: c"session_last_attached",
            cb: FormatTableCallback::Time(format_cb_session_last_attached),
        },
        format_table_entry {
            key: c"session_many_attached",
            cb: FormatTableCallback::String(format_cb_session_many_attached),
        },
        format_table_entry {
            key: c"session_marked",
            cb: FormatTableCallback::String(format_cb_session_marked),
        },
        format_table_entry {
            key: c"session_name",
            cb: FormatTableCallback::String(format_cb_session_name),
        },
        format_table_entry {
            key: c"session_path",
            cb: FormatTableCallback::String(format_cb_session_path),
        },
        format_table_entry {
            key: c"session_silence_flag",
            cb: FormatTableCallback::String(format_cb_session_silence_flag),
        },
        format_table_entry {
            key: c"session_stack",
            cb: FormatTableCallback::String(format_cb_session_stack),
        },
        format_table_entry {
            key: c"session_windows",
            cb: FormatTableCallback::String(format_cb_session_windows),
        },
        format_table_entry {
            key: c"sixel_support",
            cb: FormatTableCallback::String(format_cb_sixel_support),
        },
        format_table_entry {
            key: c"socket_path",
            cb: FormatTableCallback::String(format_cb_socket_path),
        },
        format_table_entry {
            key: c"start_time",
            cb: FormatTableCallback::Time(format_cb_start_time),
        },
        format_table_entry {
            key: c"synchronized_output_flag",
            cb: FormatTableCallback::String(format_cb_synchronized_output_flag),
        },
        format_table_entry {
            key: c"tree_mode_format",
            cb: FormatTableCallback::String(format_cb_tree_mode_format),
        },
        format_table_entry {
            key: c"uid",
            cb: FormatTableCallback::String(format_cb_uid),
        },
        format_table_entry {
            key: c"user",
            cb: FormatTableCallback::String(format_cb_user),
        },
        format_table_entry {
            key: c"version",
            cb: FormatTableCallback::String(format_cb_version),
        },
        format_table_entry {
            key: c"window_active",
            cb: FormatTableCallback::String(format_cb_window_active),
        },
        format_table_entry {
            key: c"window_active_clients",
            cb: FormatTableCallback::String(format_cb_window_active_clients),
        },
        format_table_entry {
            key: c"window_active_clients_list",
            cb: FormatTableCallback::String(format_cb_window_active_clients_list),
        },
        format_table_entry {
            key: c"window_active_sessions",
            cb: FormatTableCallback::String(format_cb_window_active_sessions),
        },
        format_table_entry {
            key: c"window_active_sessions_list",
            cb: FormatTableCallback::String(format_cb_window_active_sessions_list),
        },
        format_table_entry {
            key: c"window_activity",
            cb: FormatTableCallback::Time(format_cb_window_activity),
        },
        format_table_entry {
            key: c"window_activity_flag",
            cb: FormatTableCallback::String(format_cb_window_activity_flag),
        },
        format_table_entry {
            key: c"window_bell_flag",
            cb: FormatTableCallback::String(format_cb_window_bell_flag),
        },
        format_table_entry {
            key: c"window_bigger",
            cb: FormatTableCallback::String(format_cb_window_bigger),
        },
        format_table_entry {
            key: c"window_cell_height",
            cb: FormatTableCallback::String(format_cb_window_cell_height),
        },
        format_table_entry {
            key: c"window_cell_width",
            cb: FormatTableCallback::String(format_cb_window_cell_width),
        },
        format_table_entry {
            key: c"window_end_flag",
            cb: FormatTableCallback::String(format_cb_window_end_flag),
        },
        format_table_entry {
            key: c"window_flags",
            cb: FormatTableCallback::String(format_cb_window_flags),
        },
        format_table_entry {
            key: c"window_format",
            cb: FormatTableCallback::String(format_cb_window_format),
        },
        format_table_entry {
            key: c"window_height",
            cb: FormatTableCallback::String(format_cb_window_height),
        },
        format_table_entry {
            key: c"window_id",
            cb: FormatTableCallback::String(format_cb_window_id),
        },
        format_table_entry {
            key: c"window_index",
            cb: FormatTableCallback::String(format_cb_window_index),
        },
        format_table_entry {
            key: c"window_last_flag",
            cb: FormatTableCallback::String(format_cb_window_last_flag),
        },
        format_table_entry {
            key: c"window_layout",
            cb: FormatTableCallback::String(format_cb_window_layout),
        },
        format_table_entry {
            key: c"window_linked",
            cb: FormatTableCallback::String(format_cb_window_linked),
        },
        format_table_entry {
            key: c"window_linked_sessions",
            cb: FormatTableCallback::String(format_cb_window_linked_sessions),
        },
        format_table_entry {
            key: c"window_linked_sessions_list",
            cb: FormatTableCallback::String(format_cb_window_linked_sessions_list),
        },
        format_table_entry {
            key: c"window_marked_flag",
            cb: FormatTableCallback::String(format_cb_window_marked_flag),
        },
        format_table_entry {
            key: c"window_name",
            cb: FormatTableCallback::String(format_cb_window_name),
        },
        format_table_entry {
            key: c"window_offset_x",
            cb: FormatTableCallback::String(format_cb_window_offset_x),
        },
        format_table_entry {
            key: c"window_offset_y",
            cb: FormatTableCallback::String(format_cb_window_offset_y),
        },
        format_table_entry {
            key: c"window_panes",
            cb: FormatTableCallback::String(format_cb_window_panes),
        },
        format_table_entry {
            key: c"window_raw_flags",
            cb: FormatTableCallback::String(format_cb_window_raw_flags),
        },
        format_table_entry {
            key: c"window_silence_flag",
            cb: FormatTableCallback::String(format_cb_window_silence_flag),
        },
        format_table_entry {
            key: c"window_stack_index",
            cb: FormatTableCallback::String(format_cb_window_stack_index),
        },
        format_table_entry {
            key: c"window_start_flag",
            cb: FormatTableCallback::String(format_cb_window_start_flag),
        },
        format_table_entry {
            key: c"window_visible_layout",
            cb: FormatTableCallback::String(format_cb_window_visible_layout),
        },
        format_table_entry {
            key: c"window_width",
            cb: FormatTableCallback::String(format_cb_window_width),
        },
        format_table_entry {
            key: c"window_zoomed_flag",
            cb: FormatTableCallback::String(format_cb_window_zoomed_flag),
        },
        format_table_entry {
            key: c"wrap_flag",
            cb: FormatTableCallback::String(format_cb_wrap_flag),
        },
    ]
};
fn format_table_get(key: &CStr) -> Option<&'static format_table_entry> {
    {
        match format_table.binary_search_by(|entry| entry.key.cmp(key)) {
            Ok(index) => Some(&format_table[index]),
            Err(_) => None,
        }
    }
}
pub unsafe fn format_merge(ft: &mut format_tree, from: &format_tree) {
    unsafe {
        let entries: Vec<(CString, CString)> = from
            .tree
            .iter()
            .filter_map(|(key, fe)| fe.value.clone().map(|value| (key.clone(), value)))
            .collect();
        for (key, value) in entries {
            format_add(ft, &key, c"%s".as_ptr(), fmt_args![value.as_ptr()]);
        }
    }
}
pub unsafe fn format_get_pane(ft: &format_tree) -> *mut window_pane {
    (*ft).pane()
}
unsafe fn format_create_add_item(ft: &mut format_tree, mut item: *mut cmdq_item) {
    unsafe {
        let mut event: *mut key_event = cmdq_get_event(item);
        cmdq_merge_formats(item, ft);
        ft.m = (*event).m;
    }
}
pub unsafe fn format_create(
    mut c: *mut client,
    mut item: *mut cmdq_item,
    mut tag: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> Box<format_tree> {
    unsafe {
        let mut ft = Box::new(format_tree {
            client_ref: client_ref_from_ptr(c),
            flags,
            tag: tag as u_int,
            ..Default::default()
        });
        ft.set_item(item);
        if !item.is_null() {
            format_create_add_item(&mut ft, item);
        }
        ft
    }
}
unsafe fn format_log_debug_cb(key: &CStr, value: &CStr, arg: &CStr) {
    unsafe {
        log_debug(
            c"%s: %s=%s".as_ptr(),
            fmt_args![arg.as_ptr(), key.as_ptr(), value.as_ptr()],
        );
    }
}
pub unsafe fn format_log_debug(ft: &mut format_tree, prefix: &CStr) {
    unsafe {
        format_each(ft, Some(format_log_debug_cb), prefix);
    }
}
pub unsafe fn format_each<T: Copy>(
    ft: &mut format_tree,
    mut cb: Option<unsafe fn(&CStr, &CStr, T) -> ()>,
    mut arg: T,
) {
    unsafe {
        for fte in &format_table {
            match fte.cb {
                FormatTableCallback::Time(table_cb) => {
                    if let Some(tv) = table_cb(ft) {
                        let s = xasprintf(
                            c"%lld".as_ptr(),
                            fmt_args![tv.tv_sec as ::core::ffi::c_longlong],
                        );
                        cb.expect("non-null function pointer")(fte.key, &s, arg);
                    }
                }
                FormatTableCallback::String(table_cb) => {
                    if let Some(value) = table_cb(ft) {
                        cb.expect("non-null function pointer")(fte.key, &value, arg);
                    }
                }
            }
        }
        for (key, value) in crate::plugin::each(ft.wp_id) {
            cb.expect("non-null function pointer")(&key, &value, arg);
        }
        let keys: Vec<CString> = ft.tree.keys().cloned().collect();
        for key in keys {
            let time = ft.tree[&key].time;
            if time != 0 as time_t {
                let s = xasprintf(c"%lld".as_ptr(), fmt_args![time as ::core::ffi::c_longlong]);
                cb.expect("non-null function pointer")(&key, &s, arg);
            } else {
                format_entry_fill(ft, &key);
                let value = ft.tree[&key].value.clone().unwrap_or_default();
                cb.expect("non-null function pointer")(&key, &value, arg);
            }
        }
    }
}
/// Fills in the entry `key` names so that it holds the string the rest of the
/// walk reads. An entry that already holds one, or has no callback, is left
/// as it is. The callback is asked outside the map, since it reads the tree
/// the entry sits in.
unsafe fn format_entry_fill(ft: &mut format_tree, key: &CStr) {
    unsafe {
        let Some(fe) = ft.tree.get(key) else {
            return;
        };
        if fe.value.is_some() {
            return;
        }
        let Some(cb) = fe.cb else {
            return;
        };
        let value = cb(ft).unwrap_or_default();
        if let Some(fe) = ft.tree.get_mut(key) {
            fe.value = Some(value);
        }
    }
}

/// The entry `key` names in `ft`, made and put into the tree if it is not
/// there yet. Whatever value it held is given up, since every caller sets one
/// of its own.
fn format_entry_for(ft: &mut format_tree, key: &CStr) -> *mut format_entry {
    {
        let fe = ft.tree.entry(key.to_owned()).or_insert(format_entry {
            value: None,
            time: 0 as time_t,
            cb: None,
        });
        fe.value = None;
        &raw mut *fe
    }
}

pub unsafe fn format_add(
    ft: &mut format_tree,
    key: &CStr,
    mut fmt: *const ::core::ffi::c_char,
    args: &[FmtArg],
) {
    unsafe {
        let fe = format_entry_for(ft, key);
        (*fe).cb = None;
        (*fe).time = 0 as time_t;
        (*fe).value = Some(format_alloc(fmt, args));
    }
}
pub unsafe fn format_add_tv(ft: &mut format_tree, key: &CStr, mut tv: *mut timeval) {
    unsafe {
        let fe = format_entry_for(ft, key);
        (*fe).cb = None;
        (*fe).time = (*tv).tv_sec as time_t;
        (*fe).value = None;
    }
}
pub unsafe fn format_add_cb(ft: &mut format_tree, key: &CStr, mut cb: format_entry_cb) {
    unsafe {
        let fe = format_entry_for(ft, key);
        (*fe).cb = cb;
        (*fe).time = 0 as time_t;
        (*fe).value = None;
    }
}
fn format_quote_shell(s: &CStr) -> CString {
    {
        let mut out = Vec::with_capacity(s.to_bytes().len().saturating_mul(2));
        for byte in s.to_bytes() {
            if b"|&;<>()$`\\\"'*?[# =%".contains(byte) {
                out.push(b'\\');
            }
            out.push(*byte);
        }
        CString::new(out).expect("format shell quote output has no NUL")
    }
}
fn format_quote_style(s: &CStr) -> CString {
    {
        let mut out = Vec::with_capacity(s.to_bytes().len().saturating_mul(2));
        for byte in s.to_bytes() {
            if *byte == b'#' {
                out.push(b'#');
            }
            out.push(*byte);
        }
        CString::new(out).expect("format style quote output has no NUL")
    }
}
pub fn format_pretty_time(mut t: time_t, mut seconds: ::core::ffi::c_int) -> CString {
    unsafe {
        let mut now_tm = tm::default();
        let mut tm = tm::default();
        let mut now: time_t = 0;
        let mut age: time_t = 0;
        let mut s: [::core::ffi::c_char; 9] = [0; 9];
        time(&raw mut now);
        if now < t {
            now = t;
        }
        age = now - t;
        localtime_r(&raw mut now, &raw mut now_tm);
        localtime_r(&raw mut t, &raw mut tm);
        if age < (24 as ::core::ffi::c_int * 3600 as ::core::ffi::c_int) as time_t {
            if seconds != 0 {
                strftime(
                    &raw mut s as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t,
                    c"%H:%M:%S".as_ptr(),
                    &raw mut tm,
                );
            } else {
                strftime(
                    &raw mut s as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t,
                    c"%H:%M".as_ptr(),
                    &raw mut tm,
                );
            }
            return CStr::from_ptr(&raw mut s as *mut ::core::ffi::c_char).to_owned();
        }
        if tm.tm_year == now_tm.tm_year && tm.tm_mon == now_tm.tm_mon
            || age
                < (28 as ::core::ffi::c_int * 24 as ::core::ffi::c_int * 3600 as ::core::ffi::c_int)
                    as time_t
        {
            strftime(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t,
                c"%a%d".as_ptr(),
                &raw mut tm,
            );
            return CStr::from_ptr(&raw mut s as *mut ::core::ffi::c_char).to_owned();
        }
        if tm.tm_year == now_tm.tm_year && tm.tm_mon < now_tm.tm_mon
            || tm.tm_year == now_tm.tm_year - 1 as ::core::ffi::c_int && tm.tm_mon > now_tm.tm_mon
        {
            strftime(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t,
                c"%d%b".as_ptr(),
                &raw mut tm,
            );
            return CStr::from_ptr(&raw mut s as *mut ::core::ffi::c_char).to_owned();
        }
        strftime(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t,
            c"%h%y".as_ptr(),
            &raw mut tm,
        );
        CStr::from_ptr(&raw mut s as *mut ::core::ffi::c_char).to_owned()
    }
}
unsafe fn format_find(
    ft: &mut format_tree,
    key: &CStr,
    mut modifiers: ::core::ffi::c_int,
    time_format: Option<&CStr>,
) -> Option<CString> {
    unsafe {
        let mut current_block: u64;
        let mut envent: Option<&environ_entry> = None;
        let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
        let mut idx: ::core::ffi::c_int = 0;
        let mut found: Option<CString> = None;
        let mut s: [::core::ffi::c_char; 512] = [0; 512];
        let mut t: time_t = 0 as time_t;
        let mut tm = tm::default();
        o = options_parse_get(global_options, key, &mut idx, 0 as ::core::ffi::c_int);
        if o.is_null() && !(*ft).pane().is_null() {
            o = options_parse_get(
                (*(*ft).pane()).options_ptr(),
                key,
                &mut idx,
                0 as ::core::ffi::c_int,
            );
        }
        if o.is_null() && !(*ft).window().is_null() {
            o = options_parse_get(
                (*(*ft).window()).options_ptr(),
                key,
                &mut idx,
                0 as ::core::ffi::c_int,
            );
        }
        if o.is_null() {
            o = options_parse_get(global_w_options, key, &mut idx, 0 as ::core::ffi::c_int);
        }
        if o.is_null() && !(*ft).session().is_null() {
            o = options_parse_get(
                session_options((*ft).session()),
                key,
                &mut idx,
                0 as ::core::ffi::c_int,
            );
        }
        if o.is_null() {
            o = options_parse_get(global_s_options, key, &mut idx, 0 as ::core::ffi::c_int);
        }
        if !o.is_null() {
            let option = options_to_string(o, idx, 1 as ::core::ffi::c_int);
            found = Some(option);
        } else {
            if let Some(fte) = format_table_get(key) {
                match fte.cb {
                    FormatTableCallback::Time(table_cb) => {
                        if let Some(value) = table_cb(ft) {
                            t = value.tv_sec as time_t;
                        }
                    }
                    FormatTableCallback::String(table_cb) => {
                        if let Some(value) = table_cb(ft) {
                            found = Some(value);
                        }
                    }
                }
            } else if let Some(value) = crate::plugin::find(ft.wp_id, key) {
                found = Some(value);
            } else {
                let wanted = key;
                if let Some(time) = ft.tree.get(wanted).map(|fe| fe.time) {
                    if time != 0 as time_t {
                        t = time;
                    } else {
                        format_entry_fill(ft, wanted);
                        found = Some(CStr::from_ptr(cstr_ptr(&ft.tree[wanted].value)).to_owned());
                    }
                } else {
                    if !modifiers & FORMAT_TIMESTRING != 0 {
                        envent = None;
                        if !(*ft).session().is_null() {
                            envent = environ_find(&*session_environ((*ft).session()), key.as_ptr());
                        }
                        if envent.is_none() {
                            envent = environ_find(&*global_environ, key.as_ptr());
                        }
                        if let Some(value) = envent.and_then(environ_entry_value) {
                            found = Some(value.to_owned());
                            current_block = 9836515120145841630;
                        } else {
                            current_block = 17184638872671510253;
                        }
                    } else {
                        current_block = 17184638872671510253;
                    }
                    match current_block {
                        9836515120145841630 => {}
                        _ => return None,
                    }
                }
            }
        }
        if modifiers & FORMAT_TIMESTRING != 0 {
            if t == 0 as time_t {
                let Some(found_value) = found.as_ref() else {
                    return None;
                };
                t = strtonum(
                    found_value.as_ptr(),
                    0 as ::core::ffi::c_longlong,
                    INT64_MAX as ::core::ffi::c_longlong,
                )
                .map_or(0 as time_t, |value| value as time_t);
                found = None;
            }
            if t == 0 as time_t {
                return None;
            }
            if modifiers & FORMAT_PRETTY != 0 {
                found = Some(format_pretty_time(t, 0 as ::core::ffi::c_int));
            } else {
                if let Some(time_format) = time_format {
                    localtime_r(&raw mut t, &raw mut tm);
                    found = Some(
                        format_strftime(512 as size_t, time_format.as_ptr(), &raw mut tm)
                            .unwrap_or_default(),
                    );
                } else {
                    ctime_r(&raw mut t, &raw mut s as *mut ::core::ffi::c_char);
                    s[strcspn(&raw mut s as *mut ::core::ffi::c_char, c"\n".as_ptr()) as usize] =
                        '\0' as i32 as ::core::ffi::c_char;
                    found = Some(CStr::from_ptr(&raw mut s as *mut ::core::ffi::c_char).to_owned());
                }
            }
            return found;
        }
        if t != 0 as time_t {
            found = Some(xasprintf(
                c"%lld".as_ptr(),
                fmt_args![t as ::core::ffi::c_longlong],
            ));
        } else if found.is_none() {
            return None;
        }
        if modifiers & FORMAT_BASENAME != 0 {
            let mut path = found.take().unwrap().into_bytes_with_nul();
            let basename = __xpg_basename(path.as_mut_ptr() as *mut ::core::ffi::c_char);
            found = Some(CStr::from_ptr(basename).to_owned());
        }
        if modifiers & FORMAT_DIRNAME != 0 {
            let mut path = found.take().unwrap().into_bytes_with_nul();
            let dirname = dirname(path.as_mut_ptr() as *mut ::core::ffi::c_char);
            found = Some(CStr::from_ptr(dirname).to_owned());
        }
        if modifiers & FORMAT_QUOTE_SHELL != 0 {
            let value = found.take().unwrap();
            found = Some(format_quote_shell(&value));
        }
        if modifiers & FORMAT_QUOTE_STYLE != 0 {
            let value = found.take().unwrap();
            found = Some(format_quote_style(&value));
        }
        if modifiers & FORMAT_QUOTE_ARGUMENTS != 0 {
            let value = found.take().unwrap();
            found = Some(args_escape(value.as_ptr()));
        }
        found
    }
}
unsafe fn format_check_time(es: &mut format_expand_state) -> ::core::ffi::c_int {
    unsafe {
        let mut t: uint64_t = get_timer();
        if t.wrapping_sub(es.start_time) < FORMAT_TIME_LIMIT as uint64_t {
            return 1 as ::core::ffi::c_int;
        }
        t = t.wrapping_sub(es.start_time);
        format_log1(
            es,
            c"format_check_time".as_ptr(),
            c"reached time limit (%llu)".as_ptr(),
            fmt_args![t as ::core::ffi::c_ulonglong],
        );
        0 as ::core::ffi::c_int
    }
}
unsafe fn format_unescape(es: &mut format_expand_state, s: &CStr) -> CString {
    unsafe {
        let mut brackets: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let bytes = s.to_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if format_check_time(es) == 0 {
                return CString::default();
            }
            let next = bytes.get(i + 1).copied().unwrap_or(0);
            if bytes[i] == b'#' && bytes.get(i + 1) == Some(&b'{') {
                brackets += 1;
            }
            if brackets == 0 && bytes[i] == b'#' && b",#{}:\0".contains(&next) {
                if next != 0 {
                    out.push(next);
                    i += 2;
                } else {
                    i += 1;
                }
            } else {
                if bytes[i] == b'}' {
                    brackets -= 1;
                }
                out.push(bytes[i]);
                i += 1;
            }
        }
        CString::new(out).expect("format unescape output has no NUL")
    }
}
unsafe fn format_strip(es: &mut format_expand_state, s: &CStr) -> CString {
    unsafe {
        let mut brackets: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let bytes = s.to_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if format_check_time(es) == 0 {
                return CString::default();
            }
            let next = bytes.get(i + 1).copied().unwrap_or(0);
            if bytes[i] == b'#' && bytes.get(i + 1) == Some(&b'{') {
                brackets += 1;
            }
            if bytes[i] == b'#' && b",#{}:\0".contains(&next) {
                if brackets != 0 {
                    out.push(bytes[i]);
                }
            } else {
                if bytes[i] == b'}' {
                    brackets -= 1;
                }
                out.push(bytes[i]);
            }
            i += 1;
        }
        CString::new(out).expect("format strip output has no NUL")
    }
}
unsafe fn format_skip1(
    mut es: Option<&mut format_expand_state>,
    mut s: *const ::core::ffi::c_char,
    end: &CStr,
) -> *const ::core::ffi::c_char {
    unsafe {
        let end = end.as_ptr();
        let mut brackets: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while *s as ::core::ffi::c_int != '\0' as i32 {
            if es
                .as_deref_mut()
                .is_some_and(|es| format_check_time(es) == 0)
            {
                return ::core::ptr::null::<::core::ffi::c_char>();
            }
            if *s as ::core::ffi::c_int == '#' as i32
                && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '{' as i32
            {
                brackets += 1;
            }
            if *s as ::core::ffi::c_int == '#' as i32
                && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
                && !strchr(
                    c",#{}:".as_ptr(),
                    *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                )
                .is_null()
            {
                s = s.offset(1);
            } else {
                if *s as ::core::ffi::c_int == '}' as i32 {
                    brackets -= 1;
                }
                if !strchr(end, *s as ::core::ffi::c_int).is_null()
                    && brackets == 0 as ::core::ffi::c_int
                {
                    break;
                }
            }
            s = s.offset(1);
        }
        if *s as ::core::ffi::c_int == '\0' as i32 {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        s
    }
}
pub unsafe fn format_skip(
    mut s: *const ::core::ffi::c_char,
    end: &CStr,
) -> *const ::core::ffi::c_char {
    unsafe { format_skip1(None, s, end) }
}
unsafe fn format_choose(
    es: &mut format_expand_state,
    mut s: *const ::core::ffi::c_char,
    mut expand: ::core::ffi::c_int,
) -> Option<(CString, CString)> {
    unsafe {
        let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        cp = format_skip1(Some(es), s, c",");
        if cp.is_null() {
            return None;
        }
        if expand != 0 {
            let left0 = CString::new(::core::slice::from_raw_parts(
                s as *const u8,
                cp.offset_from(s) as usize,
            ))
            .expect("format left operand has no NUL");
            let right0 = CStr::from_ptr(cp.offset(1)).to_owned();
            let left = format_expand1(es, &left0);
            let right = format_expand1(es, &right0);
            Some((left, right))
        } else {
            let left = CString::new(::core::slice::from_raw_parts(
                s as *const u8,
                cp.offset_from(s) as usize,
            ))
            .expect("format left operand has no NUL");
            let right = CStr::from_ptr(cp.offset(1)).to_owned();
            Some((left, right))
        }
    }
}
pub unsafe fn format_true(s: Option<&CStr>) -> ::core::ffi::c_int {
    unsafe {
        let s = s.map_or(::core::ptr::null(), CStr::as_ptr);
        if !s.is_null()
            && *s as ::core::ffi::c_int != '\0' as i32
            && (*s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '0' as i32
                || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32)
        {
            return 1 as ::core::ffi::c_int;
        }
        0 as ::core::ffi::c_int
    }
}
fn format_is_end(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    (c as ::core::ffi::c_int == ';' as i32 || c as ::core::ffi::c_int == ':' as i32)
        as ::core::ffi::c_int
}
unsafe fn format_add_modifier(
    list: &mut Vec<format_modifier>,
    c: *const ::core::ffi::c_char,
    n: usize,
    argv: Vec<CString>,
) {
    unsafe {
        let mut modifier = [0 as ::core::ffi::c_char; 3];
        ::core::ptr::copy_nonoverlapping(c as *const u8, modifier.as_mut_ptr() as *mut u8, n);
        modifier[n] = '\0' as i32 as ::core::ffi::c_char;
        list.push(format_modifier {
            modifier,
            size: n as u_int,
            argv,
        });
    }
}
unsafe fn format_build_modifiers(
    es: &mut format_expand_state,
    s: &mut *const ::core::ffi::c_char,
) -> Option<Vec<format_modifier>> {
    unsafe {
        let mut cp: *const ::core::ffi::c_char = *s;
        let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut list: Vec<format_modifier> = Vec::new();
        let mut c: ::core::ffi::c_char = 0;
        let mut last: [::core::ffi::c_char; 4] =
            ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"X;:\0");
        let mut argv: Vec<CString> = Vec::new();
        while *cp as ::core::ffi::c_int != '\0' as i32 && *cp as ::core::ffi::c_int != ':' as i32 {
            if *cp as ::core::ffi::c_int == ';' as i32 {
                cp = cp.offset(1);
            }
            if *cp as ::core::ffi::c_int == '\0' as i32 {
                break;
            }
            if !strchr(
                c"labcdnwETSWPL!<>".as_ptr(),
                *cp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            )
            .is_null()
                && format_is_end(*cp.offset(1 as ::core::ffi::c_int as isize)) != 0
            {
                format_add_modifier(&mut list, cp, 1 as size_t, Vec::new());
                cp = cp.offset(1);
            } else if (::core::slice::from_raw_parts(cp as *const u8, 2) == b"||"
                || ::core::slice::from_raw_parts(cp as *const u8, 2) == b"&&"
                || ::core::slice::from_raw_parts(cp as *const u8, 2) == b"!!"
                || ::core::slice::from_raw_parts(cp as *const u8, 2) == b"!="
                || ::core::slice::from_raw_parts(cp as *const u8, 2) == b"=="
                || ::core::slice::from_raw_parts(cp as *const u8, 2) == b"<="
                || ::core::slice::from_raw_parts(cp as *const u8, 2) == b">=")
                && format_is_end(*cp.offset(2 as ::core::ffi::c_int as isize)) != 0
            {
                format_add_modifier(&mut list, cp, 2 as size_t, Vec::new());
                cp = cp.offset(2 as ::core::ffi::c_int as isize);
            } else {
                if strchr(
                    c"mCLNPSst=pReqW".as_ptr(),
                    *cp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
                )
                .is_null()
                {
                    break;
                }
                c = *cp.offset(0 as ::core::ffi::c_int as isize);
                if format_is_end(*cp.offset(1 as ::core::ffi::c_int as isize)) != 0 {
                    format_add_modifier(&mut list, cp, 1 as size_t, Vec::new());
                    cp = cp.offset(1);
                } else {
                    argv = Vec::new();
                    if *(*__ctype_b_loc())
                        .offset(*cp.offset(1 as ::core::ffi::c_int as isize) as u_char
                            as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        & _ISpunct as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        == 0
                        || *cp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '-' as i32
                    {
                        end = format_skip1(
                            Some(es),
                            cp.offset(1 as ::core::ffi::c_int as isize),
                            c":;",
                        );
                        if end.is_null() {
                            break;
                        }
                        let value = CString::new(::core::slice::from_raw_parts(
                            cp.offset(1 as ::core::ffi::c_int as isize) as *const u8,
                            end.offset_from(cp.offset(1 as ::core::ffi::c_int as isize)) as usize,
                        ))
                        .expect("format modifier argument has no NUL");
                        let expanded = format_expand1(es, &value);
                        argv.push(expanded);
                        format_add_modifier(&mut list, &raw mut c, 1 as size_t, argv);
                        cp = end;
                    } else {
                        last[0 as ::core::ffi::c_int as usize] =
                            *cp.offset(1 as ::core::ffi::c_int as isize);
                        cp = cp.offset(1);
                        loop {
                            if *cp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == last[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                                && format_is_end(*cp.offset(1 as ::core::ffi::c_int as isize)) != 0
                            {
                                cp = cp.offset(1);
                                break;
                            } else {
                                end = format_skip1(
                                    Some(es),
                                    cp.offset(1 as ::core::ffi::c_int as isize),
                                    CStr::from_ptr(&raw const last as *const ::core::ffi::c_char),
                                );
                                if end.is_null() {
                                    break;
                                }
                                cp = cp.offset(1);
                                let value = CString::new(::core::slice::from_raw_parts(
                                    cp as *const u8,
                                    end.offset_from(cp) as usize,
                                ))
                                .expect("format modifier argument has no NUL");
                                let expanded = format_expand1(es, &value);
                                argv.push(expanded);
                                cp = end;
                                if !(format_is_end(*cp.offset(0 as ::core::ffi::c_int as isize))
                                    == 0)
                                {
                                    break;
                                }
                            }
                        }
                        format_add_modifier(&mut list, &raw mut c, 1 as size_t, argv);
                    }
                }
            }
        }
        if *cp as ::core::ffi::c_int != ':' as i32 {
            return None;
        }
        *s = cp.offset(1 as ::core::ffi::c_int as isize);
        Some(list)
    }
}
unsafe fn format_match(fm: &format_modifier, pattern: &CStr, text: &CStr) -> CString {
    unsafe {
        let pattern = pattern.as_ptr();
        let text = text.as_ptr();
        let mut s: *const ::core::ffi::c_char = c"".as_ptr();
        let mut r: regex_t = regex_t::default();
        let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (fm.argv.len() as ::core::ffi::c_int) >= 1 as ::core::ffi::c_int {
            s = fm.argv[0].as_ptr();
        }
        if strchr(s, 'r' as i32).is_null() {
            if !strchr(s, 'i' as i32).is_null() {
                flags |= FNM_CASEFOLD;
            }
            if fnmatch(pattern, text, flags) != 0 as ::core::ffi::c_int {
                return c"0".to_owned();
            }
        } else {
            flags = REG_EXTENDED | REG_NOSUB;
            if !strchr(s, 'i' as i32).is_null() {
                flags |= REG_ICASE;
            }
            if regcomp(&raw mut r, pattern, flags) != 0 as ::core::ffi::c_int {
                return c"0".to_owned();
            }
            if regexec(
                &raw mut r,
                text,
                0 as size_t,
                ::core::ptr::null_mut::<regmatch_t>(),
                0 as ::core::ffi::c_int,
            ) != 0 as ::core::ffi::c_int
            {
                regfree(&raw mut r);
                return c"0".to_owned();
            }
            regfree(&raw mut r);
        }
        c"1".to_owned()
    }
}
unsafe fn format_sub(fm: &format_modifier, text: &CStr, pattern: &CStr, with: &CStr) -> CString {
    let mut flags: ::core::ffi::c_int = REG_EXTENDED;
    if (fm.argv.len() as ::core::ffi::c_int) >= 3 as ::core::ffi::c_int
        && !unsafe { strchr(fm.argv[2].as_ptr(), 'i' as i32) }.is_null()
    {
        flags |= REG_ICASE;
    }
    regsub(pattern, with, text, flags).unwrap_or_else(|| text.to_owned())
}
unsafe fn format_search(fm: &format_modifier, mut wp: *mut window_pane, s: &CStr) -> CString {
    unsafe {
        let s = s.as_ptr();
        let mut ignore: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut regex: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (fm.argv.len() as ::core::ffi::c_int) >= 1 as ::core::ffi::c_int {
            if !strchr(fm.argv[0].as_ptr(), 'i' as i32).is_null() {
                ignore = 1 as ::core::ffi::c_int;
            }
            if !strchr(fm.argv[0].as_ptr(), 'r' as i32).is_null() {
                regex = 1 as ::core::ffi::c_int;
            }
        }
        xasprintf(
            c"%u".as_ptr(),
            fmt_args![window_pane_search(wp, s, regex, ignore)],
        )
    }
}
unsafe fn format_bool_op_1(
    es: &mut format_expand_state,
    fmt: &CStr,
    mut not: ::core::ffi::c_int,
) -> CString {
    unsafe {
        let mut result: ::core::ffi::c_int = 0;
        let expanded = format_expand1(es, fmt);
        result = format_true(Some(&expanded));
        if not != 0 {
            result = (result == 0) as ::core::ffi::c_int;
        }
        if result != 0 {
            c"1".to_owned()
        } else {
            c"0".to_owned()
        }
    }
}
unsafe fn format_bool_op_n(
    es: &mut format_expand_state,
    fmt: &CStr,
    mut and: ::core::ffi::c_int,
) -> CString {
    unsafe {
        let mut result: ::core::ffi::c_int = 0;
        let mut cp1: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut cp2: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        result = if and != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        cp1 = fmt.as_ptr();
        while if and != 0 {
            result
        } else {
            (result == 0) as ::core::ffi::c_int
        } != 0
        {
            cp2 = format_skip1(Some(es), cp1, c",");
            let raw = if cp2.is_null() {
                CStr::from_ptr(cp1).to_owned()
            } else {
                let len = cp2.offset_from(cp1) as usize;
                CString::new(::core::slice::from_raw_parts(cp1 as *const u8, len))
                    .expect("format operand has no NUL")
            };
            let expanded = format_expand1(es, &raw);
            format_log1(
                es,
                c"format_bool_op_n".as_ptr(),
                c"operator %s has operand: %s".as_ptr(),
                fmt_args![
                    if and != 0 {
                        c"&&".as_ptr()
                    } else {
                        c"||".as_ptr()
                    },
                    expanded.as_ptr()
                ],
            );
            if and != 0 {
                result = (result != 0 && format_true(Some(&expanded)) != 0) as ::core::ffi::c_int;
            } else {
                result = (result != 0 || format_true(Some(&expanded)) != 0) as ::core::ffi::c_int;
            }
            if cp2.is_null() {
                break;
            }
            cp1 = cp2.offset(1 as ::core::ffi::c_int as isize);
        }
        if result != 0 {
            c"1".to_owned()
        } else {
            c"0".to_owned()
        }
    }
}
unsafe fn format_session_name(es: &mut format_expand_state, fmt: &CStr) -> Option<CString> {
    unsafe {
        let name = format_expand1(es, fmt);
        for s in session_owners() {
            if strcmp(session_name(s.as_ptr()), name.as_ptr()) == 0 as ::core::ffi::c_int {
                return Some(c"1".to_owned());
            }
        }
        Some(c"0".to_owned())
    }
}
unsafe fn format_loop_sessions(
    es: &mut format_expand_state,
    fmt: &CStr,
    sc: &sort_criteria_t,
) -> Option<CString> {
    unsafe {
        let ft: *mut format_tree = es.ft;
        let mut c: *mut client = (*ft).client();
        let mut item: *mut cmdq_item = (*ft).item();
        let mut next = format_expand_state::default();
        let (all, active) = format_choose(es, fmt.as_ptr(), 0 as ::core::ffi::c_int)
            .map(|(all, active)| (all, Some(active)))
            .unwrap_or_else(|| (fmt.to_owned(), None));
        let mut value = Vec::new();
        let l = sort_get_sessions(sc);
        let n = l.len();
        for (i, &s) in l.iter().enumerate() {
            format_log1(
                es,
                c"format_loop_sessions".as_ptr(),
                c"session loop: $%u".as_ptr(),
                fmt_args![session_id(s)],
            );
            let use_0 = if !(*ft).drawn_client().is_null()
                && active.is_some()
                && session_id(s) == session_id((*(*ft).drawn_client()).session)
            {
                active.as_ref().unwrap()
            } else {
                &all
            };
            let mut last = 0 as ::core::ffi::c_int;
            if i == n - 1 {
                last = FORMAT_LAST;
            }
            let mut nft = format_create(c, item, FORMAT_NONE, (*ft).flags | last);
            let nft_ptr = &raw mut *nft;
            format_defaults(
                &mut *nft_ptr,
                (*ft).drawn_client(),
                s,
                ::core::ptr::null_mut::<winlink>(),
                ::core::ptr::null_mut::<window_pane>(),
            );
            next = *es;
            next.flags |= 0 as ::core::ffi::c_int;
            next.ft = nft_ptr;
            let expanded = format_expand1(&mut next, use_0);
            value.extend_from_slice(expanded.as_bytes());
        }
        Some(CString::new(value).expect("format session loop output has no NUL"))
    }
}
unsafe fn format_window_name(es: &mut format_expand_state, fmt: &CStr) -> Option<CString> {
    unsafe {
        let ft: *mut format_tree = es.ft;
        let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
        if (*ft).session().is_null() {
            format_log1(
                es,
                c"format_window_name".as_ptr(),
                c"window name but no session".as_ptr(),
                fmt_args![],
            );
            return None;
        }
        let name = format_expand1(es, fmt);
        wl = winlinks_first(&mut (*(*ft).session()).windows);
        while !wl.is_null() {
            if (*(*wl).window()).name.as_deref() == Some(name.as_c_str()) {
                return Some(c"1".to_owned());
            }
            wl = winlinks_after(wl);
        }
        Some(c"0".to_owned())
    }
}
unsafe fn format_add_window_neighbor(
    mut nft: &mut format_tree,
    mut wl: *mut winlink,
    mut s: *mut session,
    prefix: &CStr,
) {
    unsafe {
        let prefix = prefix.as_ptr();
        let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
        let mut oname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let key = xasprintf(c"%s_window_index".as_ptr(), fmt_args![prefix]);
        format_add(nft, &key, c"%u".as_ptr(), fmt_args![(*wl).idx]);
        let key = xasprintf(c"%s_window_active".as_ptr(), fmt_args![prefix]);
        format_add(
            nft,
            &key,
            c"%d".as_ptr(),
            fmt_args![(wl == session_get_curw(s)) as ::core::ffi::c_int],
        );
        o = options_first((*(*wl).window()).options_ptr());
        while !o.is_null() {
            oname = options_name(o).as_ptr();
            if *oname as ::core::ffi::c_int == '@' as i32 {
                let prefixed = xasprintf(c"%s_%s".as_ptr(), fmt_args![prefix, oname]);
                let oval =
                    options_to_string(o, -(1 as ::core::ffi::c_int), 1 as ::core::ffi::c_int);
                format_add(nft, &prefixed, c"%s".as_ptr(), fmt_args![oval.as_ptr()]);
            }
            o = options_next(o);
        }
    }
}
unsafe fn format_loop_windows(
    es: &mut format_expand_state,
    fmt: &CStr,
    sc: &sort_criteria_t,
) -> Option<CString> {
    unsafe {
        let ft: *mut format_tree = es.ft;
        let mut c: *mut client = (*ft).client();
        let mut item: *mut cmdq_item = (*ft).item();
        let mut next = format_expand_state::default();
        if (*ft).session().is_null() {
            format_log1(
                es,
                c"format_loop_windows".as_ptr(),
                c"window loop but no session".as_ptr(),
                fmt_args![],
            );
            return None;
        }
        let (all, active) = format_choose(es, fmt.as_ptr(), 0 as ::core::ffi::c_int)
            .map(|(all, active)| (all, Some(active)))
            .unwrap_or_else(|| (fmt.to_owned(), None));
        let mut value = Vec::new();
        let l = sort_get_winlinks_session((*ft).session(), sc);
        let n = l.len();
        for (i, &wl) in l.iter().enumerate() {
            let w = (*wl).window();
            format_log1(
                es,
                c"format_loop_windows".as_ptr(),
                c"window loop: %u @%u".as_ptr(),
                fmt_args![(*wl).idx, (*w).id],
            );
            let use_0 = if active.is_some() && wl == session_get_curw((*ft).session()) {
                active.as_ref().unwrap()
            } else {
                &all
            };
            let mut last = 0 as ::core::ffi::c_int;
            if i == n - 1 {
                last = FORMAT_LAST;
            }
            let mut nft = format_create(
                c,
                item,
                (FORMAT_WINDOW | (*w).id) as ::core::ffi::c_int,
                (*ft).flags | last,
            );
            let nft_ptr = &raw mut *nft;
            format_defaults(
                &mut *nft_ptr,
                (*ft).drawn_client(),
                (*ft).session(),
                wl,
                ::core::ptr::null_mut::<window_pane>(),
            );
            format_add(
                &mut *nft_ptr,
                c"window_after_active",
                c"%d".as_ptr(),
                fmt_args![
                    (i > 0 && l[i - 1] == session_get_curw((*ft).session())) as ::core::ffi::c_int
                ],
            );
            format_add(
                &mut *nft_ptr,
                c"window_before_active",
                c"%d".as_ptr(),
                fmt_args![
                    (i + 1 < n && l[i + 1] == session_get_curw((*ft).session()))
                        as ::core::ffi::c_int
                ],
            );
            if i + 1 < n {
                format_add_window_neighbor(&mut *nft_ptr, l[i + 1], (*ft).session(), c"next");
            }
            if i > 0 {
                format_add_window_neighbor(&mut *nft_ptr, l[i - 1], (*ft).session(), c"prev");
            }
            next = *es;
            next.flags |= 0 as ::core::ffi::c_int;
            next.ft = nft_ptr;
            let expanded = format_expand1(&mut next, use_0);
            value.extend_from_slice(expanded.as_bytes());
        }
        Some(CString::new(value).expect("format window loop output has no NUL"))
    }
}
unsafe fn format_loop_panes(
    es: &mut format_expand_state,
    fmt: &CStr,
    sc: &sort_criteria_t,
) -> Option<CString> {
    unsafe {
        let ft: *mut format_tree = es.ft;
        let mut c: *mut client = (*ft).client();
        let mut item: *mut cmdq_item = (*ft).item();
        let mut next = format_expand_state::default();
        if (*ft).window().is_null() {
            format_log1(
                es,
                c"format_loop_panes".as_ptr(),
                c"pane loop but no window".as_ptr(),
                fmt_args![],
            );
            return None;
        }
        let (all, active) = format_choose(es, fmt.as_ptr(), 0 as ::core::ffi::c_int)
            .map(|(all, active)| (all, Some(active)))
            .unwrap_or_else(|| (fmt.to_owned(), None));
        let mut value = Vec::new();
        let l = sort_get_panes_window((*ft).window(), sc);
        let n = l.len();
        for (i, &wp) in l.iter().enumerate() {
            format_log1(
                es,
                c"format_loop_panes".as_ptr(),
                c"pane loop: %%%u".as_ptr(),
                fmt_args![(*wp).id],
            );
            let use_0 = if active.is_some() && wp == window_get_active((*ft).window()) {
                active.as_ref().unwrap()
            } else {
                &all
            };
            let mut last = 0 as ::core::ffi::c_int;
            if i == n - 1 {
                last = FORMAT_LAST;
            }
            let mut nft = format_create(
                c,
                item,
                (FORMAT_PANE | (*wp).id) as ::core::ffi::c_int,
                (*ft).flags | last,
            );
            let nft_ptr = &raw mut *nft;
            format_defaults(
                &mut *nft_ptr,
                (*ft).drawn_client(),
                (*ft).session(),
                (*ft).winlink(),
                wp,
            );
            next = *es;
            next.flags |= 0 as ::core::ffi::c_int;
            next.ft = nft_ptr;
            let expanded = format_expand1(&mut next, use_0);
            value.extend_from_slice(expanded.as_bytes());
        }
        Some(CString::new(value).expect("format pane loop output has no NUL"))
    }
}
unsafe fn format_loop_clients(
    es: &mut format_expand_state,
    fmt: &CStr,
    sc: &sort_criteria_t,
) -> Option<CString> {
    unsafe {
        let ft: *mut format_tree = es.ft;
        let mut item: *mut cmdq_item = (*ft).item();
        let mut next = format_expand_state::default();
        let mut value = Vec::new();
        let l = sort_get_clients(sc);
        let n = l.len();
        for (i, &c) in l.iter().enumerate() {
            format_log1(
                es,
                c"format_loop_clients".as_ptr(),
                c"client loop: %s".as_ptr(),
                fmt_args![(*c).name.as_deref()],
            );
            let mut last = 0 as ::core::ffi::c_int;
            if i == n - 1 {
                last = FORMAT_LAST;
            }
            let mut nft = format_create(c, item, 0 as ::core::ffi::c_int, (*ft).flags | last);
            let nft_ptr = &raw mut *nft;
            format_defaults(
                &mut *nft_ptr,
                c,
                (*ft).session(),
                (*ft).winlink(),
                (*ft).pane(),
            );
            next = *es;
            next.flags |= 0 as ::core::ffi::c_int;
            next.ft = nft_ptr;
            let expanded = format_expand1(&mut next, fmt);
            value.extend_from_slice(expanded.as_bytes());
        }
        Some(CString::new(value).expect("format client loop output has no NUL"))
    }
}
unsafe fn format_replace_expression(
    mexp: &format_modifier,
    es: &mut format_expand_state,
    copy: &CStr,
) -> Option<CString> {
    unsafe {
        let copy = copy.as_ptr();
        let mut current_block: u64;
        let mut argc: ::core::ffi::c_int = mexp.argv.len() as ::core::ffi::c_int;
        let mut endch: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut value: Option<CString> = None;
        let mut left: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut right: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut use_fp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut prec: u_int = 0 as u_int;
        let mut mleft: ::core::ffi::c_double = 0.;
        let mut mright: ::core::ffi::c_double = 0.;
        let mut result: ::core::ffi::c_double = 0.;
        let mut operator: format_operator = ADD;
        if strcmp(mexp.argv[0].as_ptr(), c"+".as_ptr()) == 0 as ::core::ffi::c_int {
            operator = ADD;
            current_block = 4495394744059808450;
        } else if strcmp(mexp.argv[0].as_ptr(), c"-".as_ptr()) == 0 as ::core::ffi::c_int {
            operator = SUBTRACT;
            current_block = 4495394744059808450;
        } else if strcmp(mexp.argv[0].as_ptr(), c"*".as_ptr()) == 0 as ::core::ffi::c_int {
            operator = MULTIPLY;
            current_block = 4495394744059808450;
        } else if strcmp(mexp.argv[0].as_ptr(), c"/".as_ptr()) == 0 as ::core::ffi::c_int {
            operator = DIVIDE;
            current_block = 4495394744059808450;
        } else if strcmp(mexp.argv[0].as_ptr(), c"%".as_ptr()) == 0 as ::core::ffi::c_int
            || strcmp(mexp.argv[0].as_ptr(), c"m".as_ptr()) == 0 as ::core::ffi::c_int
        {
            operator = MODULUS;
            current_block = 4495394744059808450;
        } else if strcmp(mexp.argv[0].as_ptr(), c"==".as_ptr()) == 0 as ::core::ffi::c_int {
            operator = EQUAL;
            current_block = 4495394744059808450;
        } else if strcmp(mexp.argv[0].as_ptr(), c"!=".as_ptr()) == 0 as ::core::ffi::c_int {
            operator = NOT_EQUAL;
            current_block = 4495394744059808450;
        } else if strcmp(mexp.argv[0].as_ptr(), c">".as_ptr()) == 0 as ::core::ffi::c_int {
            operator = GREATER_THAN;
            current_block = 4495394744059808450;
        } else if strcmp(mexp.argv[0].as_ptr(), c"<".as_ptr()) == 0 as ::core::ffi::c_int {
            operator = LESS_THAN;
            current_block = 4495394744059808450;
        } else if strcmp(mexp.argv[0].as_ptr(), c">=".as_ptr()) == 0 as ::core::ffi::c_int {
            operator = GREATER_THAN_EQUAL;
            current_block = 4495394744059808450;
        } else if strcmp(mexp.argv[0].as_ptr(), c"<=".as_ptr()) == 0 as ::core::ffi::c_int {
            operator = LESS_THAN_EQUAL;
            current_block = 4495394744059808450;
        } else {
            format_log1(
                es,
                c"format_replace_expression".as_ptr(),
                c"expression has no valid operator: '%s'".as_ptr(),
                fmt_args![mexp.argv[0].as_ptr()],
            );
            current_block = 17888409041102335484;
        }
        if current_block == 4495394744059808450 {
            if argc >= 2 as ::core::ffi::c_int
                && !strchr(mexp.argv[1].as_ptr(), 'f' as i32).is_null()
            {
                use_fp = 1 as ::core::ffi::c_int;
                prec = 2 as u_int;
            }
            if argc >= 3 as ::core::ffi::c_int {
                match strtonum(
                    mexp.argv[2].as_ptr(),
                    -FORMAT_MAX_PRECISION as ::core::ffi::c_longlong,
                    FORMAT_MAX_PRECISION as ::core::ffi::c_longlong,
                ) {
                    Ok(value) => {
                        prec = value as u_int;
                        current_block = 3437258052017859086;
                    }
                    Err(errstr) => {
                        format_log1(
                            es,
                            c"format_replace_expression".as_ptr(),
                            c"expression precision %s: %s".as_ptr(),
                            fmt_args![errstr.as_ptr(), mexp.argv[2].as_ptr()],
                        );
                        current_block = 17888409041102335484;
                    }
                }
            } else {
                current_block = 3437258052017859086;
            }
            match current_block {
                17888409041102335484 => {}
                _ => {
                    let operands = format_choose(es, copy, 1 as ::core::ffi::c_int);
                    if operands.is_none() {
                        format_log1(
                            es,
                            c"format_replace_expression".as_ptr(),
                            c"expression syntax error".as_ptr(),
                            fmt_args![],
                        );
                    } else {
                        let (left_value, right_value) = operands.unwrap();
                        left = left_value.as_ptr() as *mut ::core::ffi::c_char;
                        right = right_value.as_ptr() as *mut ::core::ffi::c_char;
                        mleft = strtod(left, &raw mut endch);
                        if *endch as ::core::ffi::c_int != '\0' as i32 {
                            format_log1(
                                es,
                                c"format_replace_expression".as_ptr(),
                                c"expression left side is invalid: %s".as_ptr(),
                                fmt_args![left],
                            );
                        } else {
                            mright = strtod(right, &raw mut endch);
                            if *endch as ::core::ffi::c_int != '\0' as i32 {
                                format_log1(
                                    es,
                                    c"format_replace_expression".as_ptr(),
                                    c"expression right side is invalid: %s".as_ptr(),
                                    fmt_args![right],
                                );
                            } else {
                                if use_fp == 0 {
                                    mleft =
                                        mleft as ::core::ffi::c_longlong as ::core::ffi::c_double;
                                    mright =
                                        mright as ::core::ffi::c_longlong as ::core::ffi::c_double;
                                }
                                format_log1(
                                    es,
                                    c"format_replace_expression".as_ptr(),
                                    c"expression left side is: %.*f".as_ptr(),
                                    fmt_args![prec, mleft],
                                );
                                format_log1(
                                    es,
                                    c"format_replace_expression".as_ptr(),
                                    c"expression right side is: %.*f".as_ptr(),
                                    fmt_args![prec, mright],
                                );
                                match operator {
                                    ADD => {
                                        result = mleft + mright;
                                    }
                                    SUBTRACT => {
                                        result = mleft - mright;
                                    }
                                    MULTIPLY => {
                                        result = mleft * mright;
                                    }
                                    DIVIDE => {
                                        result = mleft / mright;
                                    }
                                    MODULUS => {
                                        result = fmod(mleft, mright);
                                    }
                                    EQUAL => {
                                        result = (fabs(mleft - mright) < 1e-9f64)
                                            as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    NOT_EQUAL => {
                                        result = (fabs(mleft - mright) > 1e-9f64)
                                            as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    GREATER_THAN => {
                                        result = (mleft > mright) as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    GREATER_THAN_EQUAL => {
                                        result = (mleft >= mright) as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    LESS_THAN => {
                                        result = (mleft < mright) as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    LESS_THAN_EQUAL => {
                                        result = (mleft <= mright) as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    _ => {}
                                }
                                if use_fp != 0 {
                                    value =
                                        Some(xasprintf(c"%.*f".as_ptr(), fmt_args![prec, result]));
                                } else {
                                    value = Some(xasprintf(
                                        c"%.*f".as_ptr(),
                                        fmt_args![
                                            prec,
                                            result as ::core::ffi::c_longlong
                                                as ::core::ffi::c_double
                                        ],
                                    ));
                                }
                                format_log1(
                                    es,
                                    c"format_replace_expression".as_ptr(),
                                    c"expression result is %s".as_ptr(),
                                    fmt_args![value.as_ref().unwrap().as_ptr()],
                                );
                                return value;
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
unsafe fn format_replace(
    es: &mut format_expand_state,
    mut key: *const ::core::ffi::c_char,
    mut keylen: size_t,
    out: &mut Vec<u8>,
) -> ::core::ffi::c_int {
    unsafe {
        let mut current_block: u64;
        let mut sort_crit = sort_criteria_t::default();
        let ft: *mut format_tree = es.ft;
        let mut wp: *mut window_pane = (*ft).pane();
        let mut copy: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut cp2: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut marker: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut time_format: Option<CString> = None;
        let mut value: Option<CString> = None;
        let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut limit: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut j: ::core::ffi::c_int = 0;
        let mut c: ::core::ffi::c_int = 0;
        let mut cmp: Option<usize> = None;
        let mut search: Option<usize> = None;
        let mut sub: Vec<usize> = Vec::new();
        let mut mexp: Option<usize> = None;
        let mut bool_op_n: Option<usize> = None;
        let mut i: u_int = 0;
        let mut count: u_int = 0;
        let mut nsub: u_int = 0 as u_int;
        let mut nrep: u_int = 0;
        let mut next = format_expand_state::default();
        sort_crit.order = SORT_ORDER;
        sort_crit.reversed = 0 as ::core::ffi::c_int;
        let copy0 = CString::new(::core::slice::from_raw_parts(key as *const u8, keylen))
            .expect("format replacement key has no NUL");
        copy = copy0.as_ptr();
        let list = format_build_modifiers(es, &mut copy).unwrap_or_default();
        count = list.len() as u_int;
        i = 0 as u_int;
        while i < count {
            let fm = &list[i as usize];
            if format_logging(&mut *ft) != 0 {
                format_log1(
                    es,
                    c"format_replace".as_ptr(),
                    c"modifier %u is %s".as_ptr(),
                    fmt_args![i, fm.modifier.as_ptr()],
                );
                j = 0 as ::core::ffi::c_int;
                while j < (fm.argv.len() as ::core::ffi::c_int) {
                    format_log1(
                        es,
                        c"format_replace".as_ptr(),
                        c"modifier %u argument %d: %s".as_ptr(),
                        fmt_args![i, j, fm.argv[j as usize].as_ptr()],
                    );
                    j += 1;
                }
            }
            if fm.size == 1 as u_int {
                match fm.modifier[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int {
                    109 | 60 | 62 => {
                        cmp = Some(i as usize);
                    }
                    33 => {
                        modifiers |= FORMAT_NOT;
                    }
                    67 => {
                        search = Some(i as usize);
                    }
                    115 => {
                        if !((fm.argv.len() as ::core::ffi::c_int) < 2 as ::core::ffi::c_int) {
                            sub.push(i as usize);
                            nsub = nsub.wrapping_add(1);
                        }
                    }
                    61 => {
                        if !((fm.argv.len() as ::core::ffi::c_int) < 1 as ::core::ffi::c_int) {
                            limit = strtonum(
                                fm.argv[0].as_ptr(),
                                -FORMAT_MAX_WIDTH as ::core::ffi::c_longlong,
                                FORMAT_MAX_WIDTH as ::core::ffi::c_longlong,
                            )
                            .map_or(0, |value| value as ::core::ffi::c_int);
                            if (fm.argv.len() as ::core::ffi::c_int) >= 2 as ::core::ffi::c_int {
                                marker = fm.argv[1].as_ptr();
                            }
                        }
                    }
                    112 => {
                        if !((fm.argv.len() as ::core::ffi::c_int) < 1 as ::core::ffi::c_int) {
                            width = strtonum(
                                fm.argv[0].as_ptr(),
                                -FORMAT_MAX_WIDTH as ::core::ffi::c_longlong,
                                FORMAT_MAX_WIDTH as ::core::ffi::c_longlong,
                            )
                            .map_or(0, |value| value as ::core::ffi::c_int);
                        }
                    }
                    119 => {
                        modifiers |= FORMAT_WIDTH;
                    }
                    101 => {
                        if !((fm.argv.len() as ::core::ffi::c_int) < 1 as ::core::ffi::c_int
                            || (fm.argv.len() as ::core::ffi::c_int) > 3 as ::core::ffi::c_int)
                        {
                            mexp = Some(i as usize);
                        }
                    }
                    108 => {
                        modifiers |= FORMAT_LITERAL;
                    }
                    97 => {
                        modifiers |= FORMAT_CHARACTER;
                    }
                    98 => {
                        modifiers |= FORMAT_BASENAME;
                    }
                    99 => {
                        modifiers |= FORMAT_COLOUR;
                    }
                    100 => {
                        modifiers |= FORMAT_DIRNAME;
                    }
                    110 => {
                        modifiers |= FORMAT_LENGTH;
                    }
                    116 => {
                        modifiers |= FORMAT_TIMESTRING;
                        if !((fm.argv.len() as ::core::ffi::c_int) < 1 as ::core::ffi::c_int) {
                            if !strchr(fm.argv[0].as_ptr(), 'p' as i32).is_null() {
                                modifiers |= FORMAT_PRETTY;
                            } else if (fm.argv.len() as ::core::ffi::c_int)
                                >= 2 as ::core::ffi::c_int
                                && !strchr(fm.argv[0].as_ptr(), 'f' as i32).is_null()
                            {
                                time_format =
                                    Some(format_strip(es, CStr::from_ptr(fm.argv[1].as_ptr())));
                            }
                        }
                    }
                    113 => {
                        if (fm.argv.len() as ::core::ffi::c_int) < 1 as ::core::ffi::c_int {
                            modifiers |= FORMAT_QUOTE_SHELL;
                        } else if !strchr(fm.argv[0].as_ptr(), 'e' as i32).is_null()
                            || !strchr(fm.argv[0].as_ptr(), 'h' as i32).is_null()
                        {
                            modifiers |= FORMAT_QUOTE_STYLE;
                        } else if !strchr(fm.argv[0].as_ptr(), 'a' as i32).is_null() {
                            modifiers |= FORMAT_QUOTE_ARGUMENTS;
                        }
                    }
                    69 => {
                        modifiers |= FORMAT_EXPAND;
                    }
                    84 => {
                        modifiers |= FORMAT_EXPANDTIME;
                    }
                    78 => {
                        if (fm.argv.len() as ::core::ffi::c_int) < 1 as ::core::ffi::c_int
                            || !strchr(fm.argv[0].as_ptr(), 'w' as i32).is_null()
                        {
                            modifiers |= FORMAT_WINDOW_NAME;
                        } else if !strchr(fm.argv[0].as_ptr(), 's' as i32).is_null() {
                            modifiers |= FORMAT_SESSION_NAME;
                        }
                    }
                    83 => {
                        modifiers |= FORMAT_SESSIONS;
                        if (fm.argv.len() as ::core::ffi::c_int) < 1 as ::core::ffi::c_int {
                            sort_crit.order = SORT_INDEX;
                            sort_crit.reversed = 0 as ::core::ffi::c_int;
                        } else {
                            if !strchr(fm.argv[0].as_ptr(), 'i' as i32).is_null() {
                                sort_crit.order = SORT_INDEX;
                            } else if !strchr(fm.argv[0].as_ptr(), 'n' as i32).is_null() {
                                sort_crit.order = SORT_NAME;
                            } else if !strchr(fm.argv[0].as_ptr(), 't' as i32).is_null() {
                                sort_crit.order = SORT_ACTIVITY;
                            } else {
                                sort_crit.order = SORT_INDEX;
                            }
                            if !strchr(fm.argv[0].as_ptr(), 'r' as i32).is_null() {
                                sort_crit.reversed = 1 as ::core::ffi::c_int;
                            } else {
                                sort_crit.reversed = 0 as ::core::ffi::c_int;
                            }
                        }
                    }
                    87 => {
                        modifiers |= FORMAT_WINDOWS;
                        if (fm.argv.len() as ::core::ffi::c_int) < 1 as ::core::ffi::c_int {
                            sort_crit.order = SORT_ORDER;
                            sort_crit.reversed = 0 as ::core::ffi::c_int;
                        } else {
                            if !strchr(fm.argv[0].as_ptr(), 'i' as i32).is_null() {
                                sort_crit.order = SORT_ORDER;
                            } else if !strchr(fm.argv[0].as_ptr(), 'n' as i32).is_null() {
                                sort_crit.order = SORT_NAME;
                            } else if !strchr(fm.argv[0].as_ptr(), 't' as i32).is_null() {
                                sort_crit.order = SORT_ACTIVITY;
                            } else {
                                sort_crit.order = SORT_ORDER;
                            }
                            if !strchr(fm.argv[0].as_ptr(), 'r' as i32).is_null() {
                                sort_crit.reversed = 1 as ::core::ffi::c_int;
                            } else {
                                sort_crit.reversed = 0 as ::core::ffi::c_int;
                            }
                        }
                    }
                    80 => {
                        modifiers |= FORMAT_PANES;
                        sort_crit.order = SORT_CREATION;
                        if (fm.argv.len() as ::core::ffi::c_int) < 1 as ::core::ffi::c_int {
                            sort_crit.reversed = 0 as ::core::ffi::c_int;
                        } else if !strchr(fm.argv[0].as_ptr(), 'r' as i32).is_null() {
                            sort_crit.reversed = 1 as ::core::ffi::c_int;
                        } else {
                            sort_crit.reversed = 0 as ::core::ffi::c_int;
                        }
                    }
                    76 => {
                        modifiers |= FORMAT_CLIENTS;
                        if (fm.argv.len() as ::core::ffi::c_int) < 1 as ::core::ffi::c_int {
                            sort_crit.order = SORT_ORDER;
                            sort_crit.reversed = 0 as ::core::ffi::c_int;
                        } else {
                            if !strchr(fm.argv[0].as_ptr(), 'i' as i32).is_null() {
                                sort_crit.order = SORT_ORDER;
                            } else if !strchr(fm.argv[0].as_ptr(), 'n' as i32).is_null() {
                                sort_crit.order = SORT_NAME;
                            } else if !strchr(fm.argv[0].as_ptr(), 't' as i32).is_null() {
                                sort_crit.order = SORT_ACTIVITY;
                            } else {
                                sort_crit.order = SORT_ORDER;
                            }
                            if !strchr(fm.argv[0].as_ptr(), 'r' as i32).is_null() {
                                sort_crit.reversed = 1 as ::core::ffi::c_int;
                            } else {
                                sort_crit.reversed = 0 as ::core::ffi::c_int;
                            }
                        }
                    }
                    82 => {
                        modifiers |= FORMAT_REPEAT;
                    }
                    _ => {}
                }
            } else if fm.size == 2 as u_int {
                if strcmp(fm.modifier.as_ptr(), c"||".as_ptr()) == 0 as ::core::ffi::c_int
                    || strcmp(fm.modifier.as_ptr(), c"&&".as_ptr()) == 0 as ::core::ffi::c_int
                {
                    bool_op_n = Some(i as usize);
                } else if strcmp(fm.modifier.as_ptr(), c"!!".as_ptr()) == 0 as ::core::ffi::c_int {
                    modifiers |= FORMAT_NOT_NOT;
                } else if strcmp(fm.modifier.as_ptr(), c"==".as_ptr()) == 0 as ::core::ffi::c_int
                    || strcmp(fm.modifier.as_ptr(), c"!=".as_ptr()) == 0 as ::core::ffi::c_int
                    || strcmp(fm.modifier.as_ptr(), c">=".as_ptr()) == 0 as ::core::ffi::c_int
                    || strcmp(fm.modifier.as_ptr(), c"<=".as_ptr()) == 0 as ::core::ffi::c_int
                {
                    cmp = Some(i as usize);
                }
            }
            i = i.wrapping_add(1);
        }
        let cmp = cmp.map(|i| &list[i]);
        let search = search.map(|i| &list[i]);
        let mexp = mexp.map(|i| &list[i]);
        let bool_op_n = bool_op_n.map(|i| &list[i]);
        let sub: Vec<&format_modifier> = sub.into_iter().map(|i| &list[i]).collect();
        if modifiers & FORMAT_LITERAL != 0 {
            format_log1(
                es,
                c"format_replace".as_ptr(),
                c"literal string is '%s'".as_ptr(),
                fmt_args![copy],
            );
            value = Some(format_unescape(es, CStr::from_ptr(copy)));
        } else if modifiers & FORMAT_CHARACTER != 0 {
            let new = format_expand1(es, CStr::from_ptr(copy));
            value = match strtonum(
                new.as_ptr(),
                32 as ::core::ffi::c_longlong,
                126 as ::core::ffi::c_longlong,
            ) {
                Ok(value) => {
                    c = value as ::core::ffi::c_int;
                    Some(xasprintf(c"%c".as_ptr(), fmt_args![c]))
                }
                Err(_) => Some(CString::default()),
            };
        } else if modifiers & FORMAT_COLOUR != 0 {
            let new = format_expand1(es, CStr::from_ptr(copy));
            c = colour_fromstring(new.as_ptr());
            if c == -(1 as ::core::ffi::c_int) || {
                c = colour_force_rgb(c);
                c == -(1 as ::core::ffi::c_int)
            } {
                value = Some(CString::default());
            } else {
                value = Some(xasprintf(
                    c"%06x".as_ptr(),
                    fmt_args![c & 0xffffff as ::core::ffi::c_int],
                ));
            }
        } else {
            if modifiers & FORMAT_SESSIONS != 0 {
                value = format_loop_sessions(es, CStr::from_ptr(copy), &sort_crit);
                current_block = if value.is_some() {
                    4781510679662115254
                } else {
                    75153483021275631
                };
            } else if modifiers & FORMAT_WINDOWS != 0 {
                value = format_loop_windows(es, CStr::from_ptr(copy), &sort_crit);
                current_block = if value.is_some() {
                    4781510679662115254
                } else {
                    75153483021275631
                };
            } else if modifiers & FORMAT_PANES != 0 {
                value = format_loop_panes(es, CStr::from_ptr(copy), &sort_crit);
                current_block = if value.is_some() {
                    4781510679662115254
                } else {
                    75153483021275631
                };
            } else if modifiers & FORMAT_CLIENTS != 0 {
                value = format_loop_clients(es, CStr::from_ptr(copy), &sort_crit);
                current_block = if value.is_some() {
                    4781510679662115254
                } else {
                    75153483021275631
                };
            } else if modifiers & FORMAT_WINDOW_NAME != 0 {
                value = format_window_name(es, CStr::from_ptr(copy));
                current_block = if value.is_some() {
                    4781510679662115254
                } else {
                    75153483021275631
                };
            } else if modifiers & FORMAT_SESSION_NAME != 0 {
                value = format_session_name(es, CStr::from_ptr(copy));
                current_block = if value.is_some() {
                    4781510679662115254
                } else {
                    75153483021275631
                };
            } else if let Some(search) = search {
                let new = format_expand1(es, CStr::from_ptr(copy));
                if wp.is_null() {
                    format_log1(
                        es,
                        c"format_replace".as_ptr(),
                        c"search '%s' but no pane".as_ptr(),
                        fmt_args![new.as_ptr()],
                    );
                    value = Some(c"0".to_owned());
                } else {
                    format_log1(
                        es,
                        c"format_replace".as_ptr(),
                        c"search '%s' pane %%%u".as_ptr(),
                        fmt_args![new.as_ptr(), (*wp).id],
                    );
                    value = Some(format_search(search, wp, &new));
                }
                current_block = 4781510679662115254;
            } else if modifiers & FORMAT_REPEAT != 0 {
                if let Some((left, right)) = format_choose(es, copy, 1 as ::core::ffi::c_int) {
                    let parsed = strtonum(
                        right.as_ptr(),
                        1 as ::core::ffi::c_longlong,
                        FORMAT_MAX_REPEAT as ::core::ffi::c_longlong,
                    );
                    if let Ok(parsed) = parsed {
                        nrep = parsed as u_int;
                    }
                    if parsed.is_err() {
                        value = Some(CString::default());
                        current_block = 4781510679662115254;
                    } else {
                        let mut repeated = Vec::new();
                        let mut failed = false;
                        i = 0 as u_int;
                        while i < nrep {
                            if format_check_time(es) == 0 {
                                failed = true;
                                break;
                            }
                            repeated.extend_from_slice(left.as_bytes());
                            i = i.wrapping_add(1);
                        }
                        if failed {
                            current_block = 75153483021275631;
                        } else {
                            value = Some(
                                CString::new(repeated).expect("format repeat output has no NUL"),
                            );
                            current_block = 4781510679662115254;
                        }
                    }
                } else {
                    format_log1(
                        es,
                        c"format_replace".as_ptr(),
                        c"repeat syntax error: %s".as_ptr(),
                        fmt_args![copy],
                    );
                    current_block = 75153483021275631;
                }
            } else if modifiers & FORMAT_NOT != 0 {
                value = Some(format_bool_op_1(
                    es,
                    CStr::from_ptr(copy),
                    1 as ::core::ffi::c_int,
                ));
                current_block = 4781510679662115254;
            } else if modifiers & FORMAT_NOT_NOT != 0 {
                value = Some(format_bool_op_1(
                    es,
                    CStr::from_ptr(copy),
                    0 as ::core::ffi::c_int,
                ));
                current_block = 4781510679662115254;
            } else if let Some(bool_op_n) = bool_op_n {
                if strcmp(bool_op_n.modifier.as_ptr(), c"||".as_ptr()) == 0 as ::core::ffi::c_int {
                    value = Some(format_bool_op_n(
                        es,
                        CStr::from_ptr(copy),
                        0 as ::core::ffi::c_int,
                    ));
                } else if strcmp(bool_op_n.modifier.as_ptr(), c"&&".as_ptr())
                    == 0 as ::core::ffi::c_int
                {
                    value = Some(format_bool_op_n(
                        es,
                        CStr::from_ptr(copy),
                        1 as ::core::ffi::c_int,
                    ));
                }
                current_block = 4781510679662115254;
            } else if let Some(cmp) = cmp {
                if let Some((left, right)) = format_choose(es, copy, 1 as ::core::ffi::c_int) {
                    format_log1(
                        es,
                        c"format_replace".as_ptr(),
                        c"compare %s left is: %s".as_ptr(),
                        fmt_args![cmp.modifier.as_ptr(), left.as_ptr()],
                    );
                    format_log1(
                        es,
                        c"format_replace".as_ptr(),
                        c"compare %s right is: %s".as_ptr(),
                        fmt_args![cmp.modifier.as_ptr(), right.as_ptr()],
                    );
                    if strcmp(cmp.modifier.as_ptr(), c"m".as_ptr()) == 0 as ::core::ffi::c_int {
                        value = Some(format_match(cmp, &left, &right));
                    } else {
                        let comparison = if strcmp(cmp.modifier.as_ptr(), c"==".as_ptr())
                            == 0 as ::core::ffi::c_int
                        {
                            strcmp(left.as_ptr(), right.as_ptr()) == 0
                        } else if strcmp(cmp.modifier.as_ptr(), c"!=".as_ptr())
                            == 0 as ::core::ffi::c_int
                        {
                            strcmp(left.as_ptr(), right.as_ptr()) != 0
                        } else if strcmp(cmp.modifier.as_ptr(), c"<".as_ptr())
                            == 0 as ::core::ffi::c_int
                        {
                            strcmp(left.as_ptr(), right.as_ptr()) < 0
                        } else if strcmp(cmp.modifier.as_ptr(), c">".as_ptr())
                            == 0 as ::core::ffi::c_int
                        {
                            strcmp(left.as_ptr(), right.as_ptr()) > 0
                        } else if strcmp(cmp.modifier.as_ptr(), c"<=".as_ptr())
                            == 0 as ::core::ffi::c_int
                        {
                            strcmp(left.as_ptr(), right.as_ptr()) <= 0
                        } else if strcmp(cmp.modifier.as_ptr(), c">=".as_ptr())
                            == 0 as ::core::ffi::c_int
                        {
                            strcmp(left.as_ptr(), right.as_ptr()) >= 0
                        } else {
                            false
                        };
                        value = Some(if comparison {
                            c"1".to_owned()
                        } else {
                            c"0".to_owned()
                        });
                    }
                    current_block = 4781510679662115254;
                } else {
                    format_log1(
                        es,
                        c"format_replace".as_ptr(),
                        c"compare %s syntax error: %s".as_ptr(),
                        fmt_args![cmp.modifier.as_ptr(), copy],
                    );
                    current_block = 75153483021275631;
                }
            } else {
                if *copy as ::core::ffi::c_int == '?' as i32 {
                    cp = copy.offset(1 as ::core::ffi::c_int as isize);
                    loop {
                        cp2 = format_skip1(Some(es), cp, c",");
                        if cp2.is_null() {
                            format_log1(
                                es,
                                c"format_replace".as_ptr(),
                                c"no condition matched in '%s'; using last arg".as_ptr(),
                                fmt_args![copy.offset(1 as ::core::ffi::c_int as isize)],
                            );
                            value = Some(format_expand1(es, CStr::from_ptr(cp)));
                            break;
                        } else {
                            let condition = CString::new(::core::slice::from_raw_parts(
                                cp as *const u8,
                                cp2.offset_from(cp) as usize,
                            ))
                            .expect("format condition has no NUL");
                            format_log1(
                                es,
                                c"format_replace".as_ptr(),
                                c"condition is: %s".as_ptr(),
                                fmt_args![condition.as_ptr()],
                            );
                            let time_format_ptr = time_format.as_deref();
                            let mut found =
                                format_find(&mut *ft, &condition, modifiers, time_format_ptr);
                            if found.is_none() {
                                let expanded = format_expand1(es, &condition);
                                if strcmp(expanded.as_ptr(), condition.as_ptr()) == 0 {
                                    found = Some(CString::default());
                                    format_log1(
                                        es,
                                        c"format_replace".as_ptr(),
                                        c"condition '%s' not found; assuming false".as_ptr(),
                                        fmt_args![condition.as_ptr()],
                                    );
                                } else {
                                    found = Some(expanded);
                                }
                            } else {
                                format_log1(
                                    es,
                                    c"format_replace".as_ptr(),
                                    c"condition '%s' found: %s".as_ptr(),
                                    fmt_args![condition.as_ptr(), found.as_ref().unwrap().as_ptr()],
                                );
                            }
                            cp = cp2.offset(1 as ::core::ffi::c_int as isize);
                            cp2 = format_skip1(Some(es), cp, c",");
                            if format_true(found.as_deref()) != 0 {
                                format_log1(
                                    es,
                                    c"format_replace".as_ptr(),
                                    c"condition '%s' is true".as_ptr(),
                                    fmt_args![condition.as_ptr()],
                                );
                                if cp2.is_null() {
                                    value = Some(format_expand1(es, CStr::from_ptr(cp)));
                                } else {
                                    let right = CString::new(::core::slice::from_raw_parts(
                                        cp as *const u8,
                                        cp2.offset_from(cp) as usize,
                                    ))
                                    .expect("format conditional result has no NUL");
                                    value = Some(format_expand1(es, &right));
                                }
                                break;
                            } else {
                                format_log1(
                                    es,
                                    c"format_replace".as_ptr(),
                                    c"condition '%s' is false".as_ptr(),
                                    fmt_args![condition.as_ptr()],
                                );
                                if cp2.is_null() {
                                    format_log1(
                                        es,
                                        c"format_replace".as_ptr(),
                                        c"no condition matched in '%s'; using empty string"
                                            .as_ptr(),
                                        fmt_args![copy.offset(1 as ::core::ffi::c_int as isize)],
                                    );
                                    value = Some(CString::default());
                                    break;
                                } else {
                                    cp = cp2.offset(1 as ::core::ffi::c_int as isize);
                                }
                            }
                        }
                    }
                } else if let Some(mexp) = mexp {
                    value = format_replace_expression(mexp, es, CStr::from_ptr(copy));
                    if value.is_none() {
                        value = Some(CString::default());
                    }
                } else if !strstr(copy, c"#{".as_ptr()).is_null() {
                    format_log1(
                        es,
                        c"format_replace".as_ptr(),
                        c"expanding inner format '%s'".as_ptr(),
                        fmt_args![copy],
                    );
                    value = Some(format_expand1(es, CStr::from_ptr(copy)));
                } else {
                    let time_format_ptr = time_format.as_deref();
                    if let Some(result) =
                        format_find(&mut *ft, CStr::from_ptr(copy), modifiers, time_format_ptr)
                    {
                        format_log1(
                            es,
                            c"format_replace".as_ptr(),
                            c"format '%s' found: %s".as_ptr(),
                            fmt_args![copy, result.as_ptr()],
                        );
                        value = Some(result);
                    } else {
                        format_log1(
                            es,
                            c"format_replace".as_ptr(),
                            c"format '%s' not found".as_ptr(),
                            fmt_args![copy],
                        );
                        value = Some(CString::default());
                    }
                }
                current_block = 4781510679662115254;
            }
            match current_block {
                4781510679662115254 => {}
                _ => {
                    format_log1(
                        es,
                        c"format_replace".as_ptr(),
                        c"failed %s".as_ptr(),
                        fmt_args![copy0.as_ptr()],
                    );
                    return -(1 as ::core::ffi::c_int);
                }
            }
        }
        let mut value = value.expect("format replacement has no result");
        if modifiers & FORMAT_EXPAND != 0 {
            value = format_expand1(es, &value);
        } else if modifiers & FORMAT_EXPANDTIME != 0 {
            next = *es;
            next.flags |= FORMAT_EXPAND_TIME;
            value = format_expand1(&mut next, &value);
        }
        i = 0 as u_int;
        while i < nsub {
            let left = format_expand1(es, CStr::from_ptr(sub[i as usize].argv[0].as_ptr()));
            let right = format_expand1(es, CStr::from_ptr(sub[i as usize].argv[1].as_ptr()));
            let result = format_sub(sub[i as usize], &value, &left, &right);
            format_log1(
                es,
                c"format_replace".as_ptr(),
                c"substitute '%s' to '%s': %s".as_ptr(),
                fmt_args![left.as_ptr(), right.as_ptr(), result.as_ptr()],
            );
            value = result;
            i = i.wrapping_add(1);
        }
        if limit > 0 as ::core::ffi::c_int {
            let trimmed = format_trim_left(value.as_bytes(), limit as u_int);
            if !marker.is_null() && trimmed.as_bytes() != value.as_bytes() {
                value = xasprintf(c"%s%s".as_ptr(), fmt_args![trimmed.as_ptr(), marker]);
            } else {
                value = trimmed;
            }
            format_log1(
                es,
                c"format_replace".as_ptr(),
                c"applied length limit %d: %s".as_ptr(),
                fmt_args![limit, value.as_ptr()],
            );
        } else if limit < 0 as ::core::ffi::c_int {
            let trimmed = format_trim_right(value.as_bytes(), -limit as u_int);
            if !marker.is_null() && trimmed.as_bytes() != value.as_bytes() {
                value = xasprintf(c"%s%s".as_ptr(), fmt_args![marker, trimmed.as_ptr()]);
            } else {
                value = trimmed;
            }
            format_log1(
                es,
                c"format_replace".as_ptr(),
                c"applied length limit %d: %s".as_ptr(),
                fmt_args![limit, value.as_ptr()],
            );
        }
        if width > 0 as ::core::ffi::c_int {
            value = utf8_padcstr(&value, width as u_int);
            format_log1(
                es,
                c"format_replace".as_ptr(),
                c"applied padding width %d: %s".as_ptr(),
                fmt_args![width, value.as_ptr()],
            );
        } else if width < 0 as ::core::ffi::c_int {
            value = utf8_rpadcstr(&value, -width as u_int);
            format_log1(
                es,
                c"format_replace".as_ptr(),
                c"applied padding width %d: %s".as_ptr(),
                fmt_args![width, value.as_ptr()],
            );
        }
        if modifiers & FORMAT_LENGTH != 0 {
            value = xasprintf(c"%zu".as_ptr(), fmt_args![value.as_bytes().len()]);
            format_log1(
                es,
                c"format_replace".as_ptr(),
                c"replacing with length: %s".as_ptr(),
                fmt_args![value.as_ptr()],
            );
        }
        if modifiers & FORMAT_WIDTH != 0 {
            value = xasprintf(c"%u".as_ptr(), fmt_args![format_width(value.as_bytes())]);
            format_log1(
                es,
                c"format_replace".as_ptr(),
                c"replacing with width: %s".as_ptr(),
                fmt_args![value.as_ptr()],
            );
        }
        out.extend_from_slice(value.as_bytes());
        format_log1(
            es,
            c"format_replace".as_ptr(),
            c"replaced '%s' with '%s'".as_ptr(),
            fmt_args![copy0.as_ptr(), value.as_ptr()],
        );
        0 as ::core::ffi::c_int
    }
}
unsafe fn format_expand1(es: &mut format_expand_state, fmt: &CStr) -> CString {
    unsafe {
        let mut fmt = fmt.as_ptr();
        let ft: *mut format_tree = es.ft;
        let mut buf: Vec<u8> = Vec::with_capacity(64);
        let mut ptr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut style_end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut n: size_t = 0;
        let mut ch: ::core::ffi::c_int = 0;
        let mut brackets: ::core::ffi::c_int = 0;
        let mut expanded: Option<CString> = None;
        if fmt.is_null() || *fmt as ::core::ffi::c_int == '\0' as i32 || format_check_time(es) == 0
        {
            return CString::default();
        }
        if es.loop_0 == FORMAT_LOOP_LIMIT as u_int {
            format_log1(
                es,
                c"format_expand1".as_ptr(),
                c"reached loop limit (%u)".as_ptr(),
                fmt_args![FORMAT_LOOP_LIMIT],
            );
            return CString::default();
        }
        es.loop_0 = es.loop_0.wrapping_add(1);
        format_log1(
            es,
            c"format_expand1".as_ptr(),
            c"expanding format: %s".as_ptr(),
            fmt_args![fmt],
        );
        if es.flags & FORMAT_EXPAND_TIME != 0 && !strchr(fmt, '%' as i32).is_null() {
            if es.time == 0 as time_t {
                es.time = time(::core::ptr::null_mut::<time_t>());
                localtime_r(&raw mut es.time, &raw mut es.tm);
            }
            let Some(text) = format_strftime(8192 as size_t, fmt, &raw mut es.tm) else {
                format_log1(
                    es,
                    c"format_expand1".as_ptr(),
                    c"format is too long".as_ptr(),
                    fmt_args![],
                );
                return CString::default();
            };
            if format_logging(&mut *ft) != 0 && text.as_c_str() != CStr::from_ptr(fmt) {
                format_log1(
                    es,
                    c"format_expand1".as_ptr(),
                    c"after time expanded: %s".as_ptr(),
                    fmt_args![text.as_ptr()],
                );
            }
            fmt = expanded.insert(text).as_ptr();
        }
        while *fmt as ::core::ffi::c_int != '\0' as i32 {
            if *fmt as ::core::ffi::c_int != '#' as i32 {
                buf.push(*fmt as u8);
                fmt = fmt.offset(1);
            } else {
                fmt = fmt.offset(1);
                if *fmt as ::core::ffi::c_int == '\0' as i32 {
                    break;
                }
                let fresh5 = fmt;
                fmt = fmt.offset(1);
                ch = *fresh5 as u_char as ::core::ffi::c_int;
                match ch {
                    40 => {
                        brackets = 1 as ::core::ffi::c_int;
                        ptr = fmt;
                        while *ptr as ::core::ffi::c_int != '\0' as i32 {
                            if *ptr as ::core::ffi::c_int == '(' as i32 {
                                brackets += 1;
                            }
                            if *ptr as ::core::ffi::c_int == ')' as i32 && {
                                brackets -= 1;
                                brackets == 0 as ::core::ffi::c_int
                            } {
                                break;
                            }
                            ptr = ptr.offset(1);
                        }
                        if *ptr as ::core::ffi::c_int != ')' as i32
                            || brackets != 0 as ::core::ffi::c_int
                        {
                            break;
                        }
                        n = ptr.offset_from(fmt) as ::core::ffi::c_long as size_t;
                        let name = CString::new(::core::slice::from_raw_parts(
                            fmt as *const u8,
                            n as usize,
                        ))
                        .expect("format job name has no NUL");
                        format_log1(
                            es,
                            c"format_expand1".as_ptr(),
                            c"found #(): %s".as_ptr(),
                            fmt_args![name.as_ptr()],
                        );
                        let out = if (*ft).flags & FORMAT_NOJOBS != 0
                            || es.flags & FORMAT_EXPAND_NOJOBS != 0
                        {
                            format_log1(
                                es,
                                c"format_expand1".as_ptr(),
                                c"#() is disabled".as_ptr(),
                                fmt_args![],
                            );
                            CString::default()
                        } else {
                            let out = format_job_get(es, &name);
                            format_log1(
                                es,
                                c"format_expand1".as_ptr(),
                                c"#() result: %s".as_ptr(),
                                fmt_args![out.as_ptr()],
                            );
                            out
                        };
                        buf.extend_from_slice(out.as_bytes());
                        fmt = fmt.add(n.wrapping_add(1 as size_t));
                        continue;
                    }
                    123 => {
                        ptr = format_skip1(
                            Some(es),
                            (fmt as *mut ::core::ffi::c_char)
                                .offset(-(2 as ::core::ffi::c_int as isize)),
                            c"}",
                        );
                        if ptr.is_null() {
                            break;
                        }
                        n = ptr.offset_from(fmt) as ::core::ffi::c_long as size_t;
                        format_log1(
                            es,
                            c"format_expand1".as_ptr(),
                            c"found #{}: %.*s".as_ptr(),
                            fmt_args![n as ::core::ffi::c_int, fmt],
                        );
                        if format_replace(es, fmt, n, &mut buf) != 0 as ::core::ffi::c_int {
                            break;
                        }
                        fmt = fmt.add(n.wrapping_add(1 as size_t));
                        continue;
                    }
                    91 | 35 => {
                        ptr = fmt.offset(-((ch == '[' as i32) as ::core::ffi::c_int as isize));
                        n = (2 as ::core::ffi::c_int - (ch == '[' as i32) as ::core::ffi::c_int)
                            as size_t;
                        while *ptr as ::core::ffi::c_int == '#' as i32 {
                            ptr = ptr.offset(1);
                            n = n.wrapping_add(1);
                        }
                        if *ptr as ::core::ffi::c_int == '[' as i32 {
                            style_end = format_skip1(
                                Some(es),
                                fmt.offset(-(2 as ::core::ffi::c_int as isize)),
                                c"]",
                            );
                            format_log1(
                                es,
                                c"format_expand1".as_ptr(),
                                c"found #*%zu[".as_ptr(),
                                fmt_args![n],
                            );
                            buf.extend_from_slice(::core::slice::from_raw_parts(
                                fmt.offset(-(2 as ::core::ffi::c_int as isize)) as *const u8,
                                n.wrapping_add(1 as size_t),
                            ));
                            fmt = ptr.offset(1 as ::core::ffi::c_int as isize);
                            continue;
                        }
                    }
                    125 | 44 => {}
                    _ => {
                        let mut named: Option<&::core::ffi::CStr> = None;
                        if fmt > style_end {
                            if ch >= 'A' as i32 && ch <= 'Z' as i32 {
                                named = format_upper[(ch - 'A' as i32) as usize];
                            } else if ch >= 'a' as i32 && ch <= 'z' as i32 {
                                named = format_lower[(ch - 'a' as i32) as usize];
                            }
                        }
                        s = named.map_or(::core::ptr::null(), |named| named.as_ptr());
                        if s.is_null() {
                            buf.push(b'#');
                            buf.push(ch as u8);
                            continue;
                        } else {
                            n = strlen(s);
                            format_log1(
                                es,
                                c"format_expand1".as_ptr(),
                                c"found #%c: %s".as_ptr(),
                                fmt_args![ch, s],
                            );
                            if format_replace(es, s, n, &mut buf) != 0 as ::core::ffi::c_int {
                                break;
                            } else {
                                continue;
                            }
                        }
                    }
                }
                format_log1(
                    es,
                    c"format_expand1".as_ptr(),
                    c"found #%c".as_ptr(),
                    fmt_args![ch],
                );
                buf.push(ch as u8);
            }
        }
        let result = CString::new(buf).expect("expanded format has no NUL");
        format_log1(
            es,
            c"format_expand1".as_ptr(),
            c"result is: %s".as_ptr(),
            fmt_args![result.as_ptr()],
        );
        es.loop_0 = es.loop_0.wrapping_sub(1);
        result
    }
}
pub unsafe fn format_expand_time(ft: &mut format_tree, fmt: &CStr) -> CString {
    unsafe {
        let mut es = format_expand_state {
            ft,
            flags: FORMAT_EXPAND_TIME,
            start_time: get_timer(),
            ..Default::default()
        };
        format_expand1(&mut es, fmt)
    }
}
/// Expands `fmt` against `ft`, giving back the answer the caller owns.
pub unsafe fn format_expand(ft: &mut format_tree, fmt: &CStr) -> CString {
    unsafe {
        let mut es = format_expand_state {
            ft,
            start_time: get_timer(),
            ..Default::default()
        };
        format_expand1(&mut es, fmt)
    }
}
pub unsafe fn format_single(
    mut item: *mut cmdq_item,
    fmt: &CStr,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) -> CString {
    unsafe {
        let mut ft = format_create_defaults(item, c, s, wl, wp);
        format_expand(&mut ft, fmt)
    }
}
/// Expands `fmt` against the state `fs` resolved to, giving back the answer
/// the caller owns.
pub unsafe fn format_single_from_state(
    mut item: *mut cmdq_item,
    fmt: &CStr,
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
) -> CString {
    unsafe { format_single(item, fmt, c, (*fs).session(), (*fs).winlink(), (*fs).pane()) }
}
pub unsafe fn format_single_from_target(mut item: *mut cmdq_item, fmt: &CStr) -> CString {
    unsafe {
        let mut tc: *mut client = cmdq_get_target_client(&*item);
        format_single_from_state(item, fmt, tc, cmdq_get_target(item))
    }
}
pub unsafe fn format_create_defaults(
    mut item: *mut cmdq_item,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) -> Box<format_tree> {
    unsafe {
        let mut ft = if !item.is_null() {
            format_create(
                cmdq_get_client(&*item),
                item,
                FORMAT_NONE,
                0 as ::core::ffi::c_int,
            )
        } else {
            format_create(
                ::core::ptr::null_mut::<client>(),
                item,
                FORMAT_NONE,
                0 as ::core::ffi::c_int,
            )
        };
        format_defaults(&mut ft, c, s, wl, wp);
        ft
    }
}
pub unsafe fn format_create_from_state(
    mut item: *mut cmdq_item,
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
) -> Box<format_tree> {
    unsafe { format_create_defaults(item, c, (*fs).session(), (*fs).winlink(), (*fs).pane()) }
}
pub unsafe fn format_create_from_target(mut item: *mut cmdq_item) -> Box<format_tree> {
    unsafe {
        let mut tc: *mut client = cmdq_get_target_client(&*item);
        format_create_from_state(item, tc, cmdq_get_target(item))
    }
}
pub unsafe fn format_defaults(
    ft: &mut format_tree,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) {
    unsafe {
        let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
        if !c.is_null() && (*c).name.is_some() {
            log_debug(
                c"%s: c=%s".as_ptr(),
                fmt_args![c"format_defaults".as_ptr(), (*c).name.as_deref()],
            );
        } else {
            log_debug(
                c"%s: c=none".as_ptr(),
                fmt_args![c"format_defaults".as_ptr()],
            );
        }
        if !s.is_null() {
            log_debug(
                c"%s: s=$%u".as_ptr(),
                fmt_args![c"format_defaults".as_ptr(), session_id(s)],
            );
        } else {
            log_debug(
                c"%s: s=none".as_ptr(),
                fmt_args![c"format_defaults".as_ptr()],
            );
        }
        if !wl.is_null() {
            log_debug(
                c"%s: wl=%u".as_ptr(),
                fmt_args![c"format_defaults".as_ptr(), (*wl).idx],
            );
        } else {
            log_debug(
                c"%s: wl=none".as_ptr(),
                fmt_args![c"format_defaults".as_ptr()],
            );
        }
        if !wp.is_null() {
            log_debug(
                c"%s: wp=%%%u".as_ptr(),
                fmt_args![c"format_defaults".as_ptr(), (*wp).id],
            );
        } else {
            log_debug(
                c"%s: wp=none".as_ptr(),
                fmt_args![c"format_defaults".as_ptr()],
            );
        }
        if !c.is_null() && !s.is_null() && (*c).session != s {
            log_debug(
                c"%s: session does not match".as_ptr(),
                fmt_args![c"format_defaults".as_ptr()],
            );
        }
        if !wp.is_null() {
            ft.type_0 = FORMAT_TYPE_PANE;
        } else if !wl.is_null() {
            ft.type_0 = FORMAT_TYPE_WINDOW;
        } else if !s.is_null() {
            ft.type_0 = FORMAT_TYPE_SESSION;
        } else {
            ft.type_0 = FORMAT_TYPE_UNKNOWN;
        }
        if s.is_null() && !c.is_null() {
            s = (*c).session;
        }
        if wl.is_null() && !s.is_null() {
            wl = session_get_curw(s);
        }
        if wp.is_null() && !wl.is_null() {
            wp = window_get_active((*wl).window());
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
        pb = paste_get_top(None);
        if !pb.is_null() {
            format_defaults_paste_buffer(ft, pb);
        }
    }
}
unsafe fn format_defaults_session(ft: &mut format_tree, mut s: *mut session) {
    (*ft).set_session(s);
}
unsafe fn format_defaults_client(ft: &mut format_tree, mut c: *mut client) {
    unsafe {
        if (*ft).session().is_null() {
            (*ft).set_session((*c).session);
        }
        (*ft).set_drawn_client(c);
    }
}
pub unsafe fn format_defaults_window(ft: &mut format_tree, mut w: *mut window) {
    (*ft).set_window(w);
}
unsafe fn format_defaults_winlink(ft: &mut format_tree, mut wl: *mut winlink) {
    unsafe {
        if (*ft).window().is_null() {
            format_defaults_window(ft, (*wl).window());
        }
        (*ft).set_winlink(wl);
    }
}
pub unsafe fn format_defaults_pane(ft: &mut format_tree, mut wp: *mut window_pane) {
    unsafe {
        let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
        if (*ft).window().is_null() {
            format_defaults_window(ft, (*wp).window);
        }
        (*ft).set_pane(wp);
        wme = window_pane_current_mode(wp);
        if !wme.is_null() {
            (*wme).mode().formats(wme, ft);
        }
    }
}
pub unsafe fn format_defaults_paste_buffer(ft: &mut format_tree, mut pb: *mut paste_buffer) {
    (*ft).set_buffer(pb);
}
unsafe fn format_is_word_separator(ws: &CStr, gc: &grid_cell) -> ::core::ffi::c_int {
    unsafe {
        if utf8_cstrhas(ws.as_ptr(), &(*gc).data) != 0 {
            return 1 as ::core::ffi::c_int;
        }
        if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
            return 1 as ::core::ffi::c_int;
        }
        ((*gc).data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && *(&raw const (*gc).data.data as *const u_char) as ::core::ffi::c_int == ' ' as i32)
            as ::core::ffi::c_int
    }
}
pub unsafe fn format_grid_word(mut gd: *mut grid, mut x: u_int, mut y: u_int) -> Option<CString> {
    unsafe {
        let mut gl: Option<&grid_line>;
        let mut gc = grid_default_cell;
        let mut ws: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut ud: Vec<utf8_data> = Vec::new();
        let mut end: u_int = 0;
        let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut s: Option<CString> = None;
        ws = options_get_string(global_s_options, c"word-separators".as_ptr());
        loop {
            gc = grid_get_cell(&*gd, x, y);
            if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0
                && format_is_word_separator(CStr::from_ptr(ws), &mut gc) != 0
            {
                found = 1 as ::core::ffi::c_int;
                break;
            } else {
                if x == 0 as u_int {
                    if y == 0 as u_int {
                        break;
                    }
                    gl = grid_peek_line(&*gd, y.wrapping_sub(1 as u_int));
                    if !gl.is_some_and(|gl| gl.flags & GRID_LINE_WRAPPED != 0) {
                        break;
                    }
                    y = y.wrapping_sub(1);
                    x = grid_line_length(&*gd, y);
                    if x == 0 as u_int {
                        break;
                    }
                }
                x = x.wrapping_sub(1);
            }
        }
        loop {
            if found != 0 {
                end = grid_line_length(&*gd, y);
                if end == 0 as u_int || x == end.wrapping_sub(1 as u_int) {
                    if y == (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int) {
                        break;
                    }
                    gl = grid_peek_line(&*gd, y);
                    if !gl.is_some_and(|gl| gl.flags & GRID_LINE_WRAPPED != 0) {
                        break;
                    }
                    y = y.wrapping_add(1);
                    x = 0 as u_int;
                } else {
                    x = x.wrapping_add(1);
                }
            }
            found = 1 as ::core::ffi::c_int;
            gc = grid_get_cell(&*gd, x, y);
            if gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
                continue;
            }
            if format_is_word_separator(CStr::from_ptr(ws), &mut gc) != 0 {
                break;
            }
            ud.push(gc.data);
        }
        if !ud.is_empty() {
            s = Some(utf8_vec_tocstr(&ud));
        }
        s
    }
}
pub unsafe fn format_grid_line(mut gd: *mut grid, mut y: u_int) -> CString {
    unsafe {
        let mut gc = grid_default_cell;
        let mut ud: Vec<utf8_data> = Vec::new();
        let mut x: u_int = 0;
        while x < grid_line_length(&*gd, y) {
            gc = grid_get_cell(&*gd, x, y);
            if !(gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0) {
                ud.push(gc.data);
                if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                    utf8_set(&mut *ud.last_mut().unwrap(), '\t' as i32 as u_char);
                }
            }
            x = x.wrapping_add(1);
        }
        utf8_vec_tocstr(&ud)
    }
}
pub unsafe fn format_grid_hyperlink(
    mut gd: *mut grid,
    mut x: u_int,
    mut y: u_int,
    mut s: *mut screen,
) -> Option<CString> {
    unsafe {
        let mut gc = grid_default_cell;
        loop {
            gc = grid_get_cell(&*gd, x, y);
            if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
                break;
            }
            if x == 0 as u_int {
                return None;
            }
            x = x.wrapping_sub(1);
        }
        let hyperlinks = (*s).hyperlinks_ptr();
        if hyperlinks.is_null() || gc.link == 0 as u_int {
            return None;
        }
        let (uri, _, _) = hyperlinks_get(&*hyperlinks, gc.link)?;
        Some(uri.to_owned())
    }
}
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const RB_INF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;

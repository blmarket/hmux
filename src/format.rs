pub use crate::src::shared::session::{
    session_group, session_group_entry, session_group_sessions, session_groups, sessions,
};
pub use crate::src::shared::client::{clients};
pub use crate::src::shared::arguments::{args};
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::{control_state};
pub use crate::src::shared::format::{
    format_cb, format_entry, format_entry_entry, format_entry_tree, format_job,
    format_job_entry, format_job_tree, format_tree, format_type,
};
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::job::{job, job_complete_cb, job_free_cb, job_update_cb};
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::{menu_data};
pub use crate::src::shared::options::{
    options, options_array_item, options_entry, options_table_entry,
};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::paste::{paste_buffer, paste_buffer_name_entry, paste_buffer_time_entry};
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
pub use crate::src::shared::environment::{environ, environ_entry, environ_entry_entry};
pub use crate::src::shared::ctype::{
    _ISalnum, _ISalpha, _ISblank, _IScntrl, _ISdigit, _ISgraph, _ISlower, _ISprint, _ISpunct,
    _ISspace, _ISupper, _ISxdigit, ctype_code,
};
pub use crate::src::shared::abi::{NULL_0};
pub use crate::src::shared::environment::{ENVIRON_HIDDEN};
pub use crate::src::shared::posix_io::{FNM_CASEFOLD};
pub use crate::src::shared::job::{JOB_NOWAIT};
pub use crate::src::shared::format::{
    FORMAT_BASENAME, FORMAT_CHARACTER, FORMAT_CLIENTS, FORMAT_CLIENT_ENVIRON,
    FORMAT_CLIENT_TERMCAP, FORMAT_CLIENT_TERMFEAT, FORMAT_COLOUR, FORMAT_COLOUR_ESC_BG,
    FORMAT_COLOUR_ESC_FG, FORMAT_CYCLE, FORMAT_CYCLE_PERIOD, FORMAT_DIFFERENCE, FORMAT_DIRNAME,
    FORMAT_ENVIRON, FORMAT_EXPAND, FORMAT_EXPANDTIME, FORMAT_EXPAND_NOCYCLE,
    FORMAT_EXPAND_NOJOBS, FORMAT_EXPAND_TIME, FORMAT_FORCE, FORMAT_LENGTH, FORMAT_LITERAL,
    FORMAT_LOOP_LIMIT, FORMAT_MAX_PRECISION, FORMAT_MAX_REPEAT, FORMAT_MAX_WIDTH, FORMAT_NOJOBS,
    FORMAT_NONE, FORMAT_NOT, FORMAT_NOT_NOT, FORMAT_OPTIONS, FORMAT_PANE, FORMAT_PANES,
    FORMAT_PRETTY, FORMAT_QUOTE_ARGUMENTS, FORMAT_QUOTE_SHELL, FORMAT_QUOTE_SHELL_SQ,
    FORMAT_QUOTE_STYLE, FORMAT_RELATIVE, FORMAT_REPEAT, FORMAT_SESSIONS, FORMAT_SESSION_NAME,
    FORMAT_STATUS, FORMAT_TIMESTRING, FORMAT_TIME_LIMIT, FORMAT_TIME_LOOP_CHECK, FORMAT_VERBOSE,
    FORMAT_WIDTH, FORMAT_WINDOW, FORMAT_WINDOWS, FORMAT_WINDOW_NAME,
};
pub use crate::src::shared::account::passwd;
pub use crate::src::shared::regex::{
    __re_long_size_t, re_dfa_t, re_pattern_buffer, reg_syntax_t, regex_t, regmatch_t, regoff_t,
    REG_EXTENDED, REG_ICASE,
};
pub use crate::src::shared::time::tm;
pub use crate::src::shared::window::{
    WINDOW_PANE_NO_MODE, WINDOW_SIZE_MANUAL, WINDOW_ZOOMED, WINLINK_ACTIVITY,
    WINLINK_ALERTFLAGS, WINLINK_BELL, WINLINK_SILENCE,
};
pub use crate::src::shared::pane::{
    PANE_CMDRUNNING, PANE_INPUTOFF, PANE_MINIMUM, PANE_SCROLLBARS_ALWAYS, PANE_STATUSDRAWN,
    PANE_STATUSREADY, PANE_STATUS_BOTTOM, PANE_STATUS_TOP, PANE_UNSEENCHANGES, PANE_ZOOMED,
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::screen::{
    ALL_MOUSE_MODES, EXTENDED_KEY_MODES, MODE_BRACKETPASTE, MODE_CURSOR, MODE_CURSOR_BLINKING,
    MODE_CURSOR_BLINKING_SET, MODE_CURSOR_VERY_VISIBLE, MODE_FOCUSON, MODE_INSERT, MODE_KCURSOR,
    MODE_KEYS_EXTENDED, MODE_KEYS_EXTENDED_2, MODE_KKEYPAD, MODE_MOUSE_ALL, MODE_MOUSE_BUTTON,
    MODE_MOUSE_SGR, MODE_MOUSE_STANDARD, MODE_MOUSE_UTF8, MODE_ORIGIN, MODE_SYNC,
    MODE_THEME_UPDATES, MODE_WRAP, screen, screen_sel, screen_titles,
};
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::abi::{__compar_fn_t, __gid_t, __uid_t, ssize_t, uid_t};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::tty::{TERM_256COLOURS, TERM_RGBCOLOURS, TTY_STARTED};
pub use crate::src::shared::tree::{RB_BLACK, RB_INF, RB_NEGINF, RB_RED};
pub use crate::src::shared::event::{EV_TIMEOUT};
pub use crate::src::shared::options::{OPTIONS_TABLE_IS_HOOK};
pub use crate::src::shared::client::{
    CLIENT_CONTROL, CLIENT_CONTROL_NEWLAYOUTS, CLIENT_DEAD, CLIENT_EXIT, CLIENT_READONLY,
    CLIENT_REDRAWSTATUS, CLIENT_SUSPENDED, CLIENT_UNATTACHEDFLAGS, CLIENT_UTF8,
};
pub use crate::src::shared::sort::{sort_criteria};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::tty::*;
use crate::src::shared::options::*;
use crate::src::shared::sort::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;
extern "C" {

    fn getpid() -> __pid_t;
    fn getuid() -> __uid_t;
    fn gethostname(__name: *mut ::core::ffi::c_char, __len: size_t) -> ::core::ffi::c_int;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn fnmatch(
        __pattern: *const ::core::ffi::c_char,
        __name: *const ::core::ffi::c_char,
        __flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn dirname(__path: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn __xpg_basename(__path: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn fabs(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    fn fmod(__x: ::core::ffi::c_double, __y: ::core::ffi::c_double) -> ::core::ffi::c_double;
    fn getpwuid(__uid: __uid_t) -> *mut passwd;
    fn regcomp(
        __preg: *mut regex_t,
        __pattern: *const ::core::ffi::c_char,
        __cflags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn regexec(
        __preg: *const regex_t,
        __String: *const ::core::ffi::c_char,
        __nmatch: size_t,
        __pmatch: *mut regmatch_t,
        __eflags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn regfree(__preg: *mut regex_t);
    fn strtod(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_double;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strcspn(
        __s: *const ::core::ffi::c_char,
        __reject: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_ulong;
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strlcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn time(__timer: *mut time_t) -> time_t;
    fn strftime(
        __s: *mut ::core::ffi::c_char,
        __maxsize: size_t,
        __format: *const ::core::ffi::c_char,
        __tp: *const tm,
    ) -> size_t;
    fn localtime_r(__timer: *const time_t, __tp: *mut tm) -> *mut tm;
    fn ctime_r(__timer: *const time_t, __buf: *mut ::core::ffi::c_char)
        -> *mut ::core::ffi::c_char;
    fn event_add(ev: *mut event, timeout: *const timeval) -> ::core::ffi::c_int;
    fn event_pending(
        ev: *const event,
        events: ::core::ffi::c_short,
        tv: *mut timeval,
    ) -> ::core::ffi::c_int;
    fn event_initialized(ev: *const event) -> ::core::ffi::c_int;
    fn event_set(
        _: *mut event,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_short,
        _: Option<
            unsafe extern "C" fn(
                ::core::ffi::c_int,
                ::core::ffi::c_short,
                *mut ::core::ffi::c_void,
            ) -> (),
        >,
        _: *mut ::core::ffi::c_void,
    );
    fn evbuffer_new() -> *mut evbuffer;
    fn evbuffer_free(buf: *mut evbuffer);
    fn evbuffer_get_length(buf: *const evbuffer) -> size_t;
    fn evbuffer_add(
        buf: *mut evbuffer,
        data: *const ::core::ffi::c_void,
        datlen: size_t,
    ) -> ::core::ffi::c_int;
    fn evbuffer_add_printf(
        buf: *mut evbuffer,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn evbuffer_pullup(buf: *mut evbuffer, size: ssize_t) -> *mut ::core::ffi::c_uchar;
    fn evbuffer_readline(buffer: *mut evbuffer) -> *mut ::core::ffi::c_char;
    fn strtonum(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    static mut global_options: *mut options;
    static mut global_s_options: *mut options;
    static mut global_w_options: *mut options;
    static mut global_environ: *mut environ;
    static mut start_time: timeval;
    static mut socket_path: *const ::core::ffi::c_char;
    fn get_timer() -> uint64_t;
    fn sig2name(_: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn getversion() -> *const ::core::ffi::c_char;
    fn proc_get_peer_uid(_: *mut tmuxpeer) -> uid_t;
    static mut cfg_files: *mut *mut ::core::ffi::c_char;
    static mut cfg_nfiles: u_int;
    fn paste_buffer_name(_: *mut paste_buffer) -> *const ::core::ffi::c_char;
    fn paste_buffer_created(_: *mut paste_buffer) -> time_t;
    fn paste_buffer_data(_: *mut paste_buffer, _: *mut size_t) -> *const ::core::ffi::c_char;
    fn paste_get_top(_: *mut *mut ::core::ffi::c_char) -> *mut paste_buffer;
    fn paste_make_sample(_: *mut paste_buffer) -> *mut ::core::ffi::c_char;
    fn sort_get_clients(_: *mut u_int, _: *mut sort_criteria) -> *mut *mut client;
    fn sort_get_sessions(_: *mut u_int, _: *mut sort_criteria) -> *mut *mut session;
    fn sort_get_panes_window(
        _: *mut window,
        _: *mut u_int,
        _: *mut sort_criteria,
    ) -> *mut *mut window_pane;
    fn sort_get_winlinks_session(
        _: *mut session,
        _: *mut u_int,
        _: *mut sort_criteria,
    ) -> *mut *mut winlink;
    fn format_width(_: *const ::core::ffi::c_char) -> u_int;
    fn format_trim_left(_: *const ::core::ffi::c_char, _: u_int) -> *mut ::core::ffi::c_char;
    fn format_trim_right(_: *const ::core::ffi::c_char, _: u_int) -> *mut ::core::ffi::c_char;
    fn options_first(_: *mut options) -> *mut options_entry;
    fn options_next(_: *mut options_entry) -> *mut options_entry;
    fn options_name(_: *mut options_entry) -> *const ::core::ffi::c_char;
    fn options_table_entry(_: *mut options_entry) -> *const options_table_entry;
    fn options_array_first(_: *mut options_entry) -> *mut options_array_item;
    fn options_array_next(_: *mut options_array_item) -> *mut options_array_item;
    fn options_array_item_key(_: *mut options_array_item) -> *const ::core::ffi::c_char;
    fn options_is_array(_: *mut options_entry) -> ::core::ffi::c_int;
    fn options_to_string(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn options_parse_get(
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut options_entry;
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn job_run(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
        _: *mut environ,
        _: *mut session,
        _: *const ::core::ffi::c_char,
        _: job_update_cb,
        _: job_complete_cb,
        _: job_free_cb,
        _: *mut ::core::ffi::c_void,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut job;
    fn job_free(_: *mut job);
    fn job_get_data(_: *mut job) -> *mut ::core::ffi::c_void;
    fn job_get_event(_: *mut job) -> *mut bufferevent;
    fn environ_first(_: *mut environ) -> *mut environ_entry;
    fn environ_next(_: *mut environ_entry) -> *mut environ_entry;
    fn environ_find(_: *mut environ, _: *const ::core::ffi::c_char) -> *mut environ_entry;
    fn tty_window_offset(
        _: *mut tty,
        _: *mut u_int,
        _: *mut u_int,
        _: *mut u_int,
        _: *mut u_int,
    ) -> ::core::ffi::c_int;
    fn tty_default_colours(_: *mut grid_cell, _: *mut window_pane, _: *mut u_int);
    fn tty_term_has_name(_: *mut tty_term, _: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn tty_term_number(_: *mut tty_term, _: tty_code_code) -> ::core::ffi::c_int;
    fn tty_get_features(_: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn tty_feature_present(_: *mut tty_term, _: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn args_escape(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn cmd_free_argv(_: ::core::ffi::c_int, _: *mut *mut ::core::ffi::c_char);
    fn cmd_stringify_argv(
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn cmd_mouse_at(
        _: *mut window_pane,
        _: *mut mouse_event,
        _: *mut u_int,
        _: *mut u_int,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_mouse_pane(
        _: *mut mouse_event,
        _: *mut *mut session,
        _: *mut *mut winlink,
    ) -> *mut window_pane;
    fn cmdq_merge_formats(_: *mut cmdq_item, _: *mut format_tree);
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_event(_: *mut cmdq_item) -> *mut key_event;
    fn cmdq_print(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    static mut clients: clients;
    static mut marked_pane: cmd_find_state;
    fn server_check_marked() -> ::core::ffi::c_int;
    fn server_client_get_key_table(_: *mut client) -> *const ::core::ffi::c_char;
    fn server_client_unref(_: *mut client);
    fn server_client_get_cwd(_: *mut client, _: *mut session) -> *const ::core::ffi::c_char;
    fn server_client_get_flags(_: *mut client) -> *const ::core::ffi::c_char;
    fn server_status_client(_: *mut client);
    fn status_get_range(_: *mut client, _: u_int, _: u_int) -> *mut style_range;
    fn colour_force_rgb(_: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn colour_tostring(_: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn colour_toescape(
        _: *mut client,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *const ::core::ffi::c_char;
    fn colour_fromstring(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn fuzzy_match(
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: u_int,
        _: *mut u_int,
    ) -> *mut bitstr_t;
    fn grid_peek_line(_: *mut grid, _: u_int) -> *const grid_line;
    fn grid_get_cell(_: *mut grid, _: u_int, _: u_int, _: *mut grid_cell);
    fn grid_get_line(_: *mut grid, _: u_int) -> *mut grid_line;
    fn grid_line_length(_: *mut grid, _: u_int) -> u_int;
    fn grid_view_get_cell(_: *mut grid, _: u_int, _: u_int, _: *mut grid_cell);
    fn winlinks_RB_NEXT(_: *mut winlink) -> *mut winlink;
    fn winlinks_RB_MINMAX(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn winlink_find_by_window(_: *mut winlinks, _: *mut window) -> *mut winlink;
    fn winlink_count(_: *mut winlinks) -> u_int;
    fn window_pane_index(_: *mut window_pane, _: *mut u_int) -> ::core::ffi::c_int;
    fn window_pane_zindex(_: *mut window_pane, _: *mut u_int) -> ::core::ffi::c_int;
    fn window_count_panes(_: *mut window, _: ::core::ffi::c_int) -> u_int;
    fn window_pane_search(
        _: *mut window_pane,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> u_int;
    fn window_printable_flags(_: *mut winlink, _: ::core::ffi::c_int)
        -> *const ::core::ffi::c_char;
    fn window_pane_printable_flags(_: *mut window_pane) -> *const ::core::ffi::c_char;
    fn window_pane_mode(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_scrollbar_reserve(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_get_pane_status(_: *mut window) -> ::core::ffi::c_int;
    fn window_pane_get_pane_status(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_is_floating(_: *mut window_pane) -> ::core::ffi::c_int;
    fn layout_add_horizontal_border(
        _: *mut layout_cell,
        _: *mut layout_cell,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn layout_dump(
        _: *mut window,
        _: *mut layout_cell,
        _: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    static window_buffer_mode: window_mode;
    static window_tree_mode: window_mode;
    static window_client_mode: window_mode;
    fn window_copy_get_word(_: *mut window_pane, _: u_int, _: u_int) -> *mut ::core::ffi::c_char;
    fn window_copy_get_line(_: *mut window_pane, _: u_int) -> *mut ::core::ffi::c_char;
    fn window_copy_get_hyperlink(
        _: *mut window_pane,
        _: u_int,
        _: u_int,
    ) -> *mut ::core::ffi::c_char;
    fn parse_window_name(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut sessions: sessions;
    static mut session_groups: session_groups;
    static mut next_session_id: u_int;
    fn sessions_RB_MINMAX(_: *mut sessions, _: ::core::ffi::c_int) -> *mut session;
    fn sessions_RB_NEXT(_: *mut session) -> *mut session;
    fn session_groups_RB_NEXT(_: *mut session_group) -> *mut session_group;
    fn session_groups_RB_MINMAX(
        _: *mut session_groups,
        _: ::core::ffi::c_int,
    ) -> *mut session_group;
    fn session_alive(_: *mut session) -> ::core::ffi::c_int;
    fn session_group_contains(_: *mut session) -> *mut session_group;
    fn session_group_count(_: *mut session_group) -> u_int;
    fn session_group_attached_count(_: *mut session_group) -> u_int;
    fn utf8_set(_: *mut utf8_data, _: u_char);
    fn utf8_tocstr(_: *mut utf8_data) -> *mut ::core::ffi::c_char;
    fn utf8_padcstr(_: *const ::core::ffi::c_char, _: u_int) -> *mut ::core::ffi::c_char;
    fn utf8_rpadcstr(_: *const ::core::ffi::c_char, _: u_int) -> *mut ::core::ffi::c_char;
    fn utf8_cstrhas(_: *const ::core::ffi::c_char, _: *const utf8_data) -> ::core::ffi::c_int;
    fn osdep_get_name(
        _: ::core::ffi::c_int,
        _: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn osdep_get_cwd(_: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn log_get_level() -> ::core::ffi::c_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
    fn regsub(
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn hyperlinks_get(
        _: *mut hyperlinks,
        _: u_int,
        _: *mut *const ::core::ffi::c_char,
        _: *mut *const ::core::ffi::c_char,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(_: *mut ::core::ffi::c_void, _: size_t) -> *mut ::core::ffi::c_void;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xstrndup(_: *const ::core::ffi::c_char, _: size_t) -> *mut ::core::ffi::c_char;
    fn xmemdup(_: *const ::core::ffi::c_void, _: size_t) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn xvasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    fn xsnprintf(
        _: *mut ::core::ffi::c_char,
        _: size_t,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_13 {
    pub offset: u_int,
    pub data: C2RustUnnamed_14,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_14 {
    pub attr: u_char,
    pub fg: u_char,
    pub bg: u_char,
    pub data: u_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct format_modifier {
    pub modifier: [::core::ffi::c_char; 3],
    pub size: u_int,
    pub argv: *mut *mut ::core::ffi::c_char,
    pub argc: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
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
pub struct format_table_entry {
    pub key: *const ::core::ffi::c_char,
    pub type_0: format_table_type,
    pub cb: format_cb,
}
pub type format_table_type = ::core::ffi::c_uint;

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
#[inline]
unsafe extern "C" fn bsearch(
    mut __key: *const ::core::ffi::c_void,
    mut __base: *const ::core::ffi::c_void,
    mut __nmemb: size_t,
    mut __size: size_t,
    mut __compar: __compar_fn_t,
) -> *mut ::core::ffi::c_void {
    let mut __p: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
    let mut __comparison: ::core::ffi::c_int = 0;
    while __nmemb != 0 {
        __p = (__base as *const ::core::ffi::c_char)
            .offset((__nmemb >> 1 as ::core::ffi::c_int).wrapping_mul(__size) as isize)
            as *const ::core::ffi::c_void;
        __comparison = Some(__compar.expect("non-null function pointer"))
            .expect("non-null function pointer")(__key, __p);
        if __comparison == 0 as ::core::ffi::c_int {
            return __p as *mut ::core::ffi::c_void;
        }
        if __comparison > 0 as ::core::ffi::c_int {
            __base = (__p as *const ::core::ffi::c_char).offset(__size as isize)
                as *const ::core::ffi::c_void;
            __nmemb = __nmemb.wrapping_sub(1);
        }
        __nmemb >>= 1 as ::core::ffi::c_int;
    }
    return NULL;
}
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;

static mut format_jobs: format_job_tree = format_job_tree {
    rbh_root: ::core::ptr::null::<format_job>() as *mut format_job,
};
unsafe extern "C" fn format_job_tree_RB_NEXT(mut elm: *mut format_job) -> *mut format_job {
    if !(*elm).entry.rbe_right.is_null() {
        elm = (*elm).entry.rbe_right;
        while !(*elm).entry.rbe_left.is_null() {
            elm = (*elm).entry.rbe_left;
        }
    } else if !(*elm).entry.rbe_parent.is_null() && elm == (*(*elm).entry.rbe_parent).entry.rbe_left
    {
        elm = (*elm).entry.rbe_parent;
    } else {
        while !(*elm).entry.rbe_parent.is_null()
            && elm == (*(*elm).entry.rbe_parent).entry.rbe_right
        {
            elm = (*elm).entry.rbe_parent;
        }
        elm = (*elm).entry.rbe_parent;
    }
    return elm;
}
unsafe extern "C" fn format_job_tree_RB_REMOVE_COLOR(
    mut head: *mut format_job_tree,
    mut parent: *mut format_job,
    mut elm: *mut format_job,
) {
    let mut tmp: *mut format_job = ::core::ptr::null_mut::<format_job>();
    while (elm.is_null() || (*elm).entry.rbe_color == RB_BLACK) && elm != (*head).rbh_root {
        if (*parent).entry.rbe_left == elm {
            tmp = (*parent).entry.rbe_right;
            if (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_RED;
                tmp = (*parent).entry.rbe_right;
                (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*parent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                tmp = (*parent).entry.rbe_right;
            }
            if ((*tmp).entry.rbe_left.is_null()
                || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK)
                && ((*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK)
            {
                (*tmp).entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).entry.rbe_parent;
            } else {
                if (*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK
                {
                    let mut oleft: *mut format_job = ::core::ptr::null_mut::<format_job>();
                    oleft = (*tmp).entry.rbe_left;
                    if !oleft.is_null() {
                        (*oleft).entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).entry.rbe_color = RB_RED;
                    oleft = (*tmp).entry.rbe_left;
                    (*tmp).entry.rbe_left = (*oleft).entry.rbe_right;
                    if !(*tmp).entry.rbe_left.is_null() {
                        (*(*oleft).entry.rbe_right).entry.rbe_parent = tmp;
                    }
                    (*oleft).entry.rbe_parent = (*tmp).entry.rbe_parent;
                    if !(*oleft).entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).entry.rbe_parent).entry.rbe_left {
                            (*(*tmp).entry.rbe_parent).entry.rbe_left = oleft;
                        } else {
                            (*(*tmp).entry.rbe_parent).entry.rbe_right = oleft;
                        }
                    } else {
                        (*head).rbh_root = oleft;
                    }
                    (*oleft).entry.rbe_right = tmp;
                    (*tmp).entry.rbe_parent = oleft;
                    !(*oleft).entry.rbe_parent.is_null();
                    tmp = (*parent).entry.rbe_right;
                }
                (*tmp).entry.rbe_color = (*parent).entry.rbe_color;
                (*parent).entry.rbe_color = RB_BLACK;
                if !(*tmp).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).entry.rbe_right;
                (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*parent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        } else {
            tmp = (*parent).entry.rbe_left;
            if (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_RED;
                tmp = (*parent).entry.rbe_left;
                (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*parent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                tmp = (*parent).entry.rbe_left;
            }
            if ((*tmp).entry.rbe_left.is_null()
                || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK)
                && ((*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK)
            {
                (*tmp).entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).entry.rbe_parent;
            } else {
                if (*tmp).entry.rbe_left.is_null()
                    || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK
                {
                    let mut oright: *mut format_job = ::core::ptr::null_mut::<format_job>();
                    oright = (*tmp).entry.rbe_right;
                    if !oright.is_null() {
                        (*oright).entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).entry.rbe_color = RB_RED;
                    oright = (*tmp).entry.rbe_right;
                    (*tmp).entry.rbe_right = (*oright).entry.rbe_left;
                    if !(*tmp).entry.rbe_right.is_null() {
                        (*(*oright).entry.rbe_left).entry.rbe_parent = tmp;
                    }
                    (*oright).entry.rbe_parent = (*tmp).entry.rbe_parent;
                    if !(*oright).entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).entry.rbe_parent).entry.rbe_left {
                            (*(*tmp).entry.rbe_parent).entry.rbe_left = oright;
                        } else {
                            (*(*tmp).entry.rbe_parent).entry.rbe_right = oright;
                        }
                    } else {
                        (*head).rbh_root = oright;
                    }
                    (*oright).entry.rbe_left = tmp;
                    (*tmp).entry.rbe_parent = oright;
                    !(*oright).entry.rbe_parent.is_null();
                    tmp = (*parent).entry.rbe_left;
                }
                (*tmp).entry.rbe_color = (*parent).entry.rbe_color;
                (*parent).entry.rbe_color = RB_BLACK;
                if !(*tmp).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).entry.rbe_left;
                (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*parent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        }
    }
    if !elm.is_null() {
        (*elm).entry.rbe_color = RB_BLACK;
    }
}
unsafe extern "C" fn format_job_tree_RB_INSERT(
    mut head: *mut format_job_tree,
    mut elm: *mut format_job,
) -> *mut format_job {
    let mut tmp: *mut format_job = ::core::ptr::null_mut::<format_job>();
    let mut parent: *mut format_job = ::core::ptr::null_mut::<format_job>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = format_job_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<format_job>();
    (*elm).entry.rbe_left = (*elm).entry.rbe_right;
    (*elm).entry.rbe_color = RB_RED;
    if !parent.is_null() {
        if comp < 0 as ::core::ffi::c_int {
            (*parent).entry.rbe_left = elm;
        } else {
            (*parent).entry.rbe_right = elm;
        }
    } else {
        (*head).rbh_root = elm;
    }
    format_job_tree_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<format_job>();
}
unsafe extern "C" fn format_job_tree_RB_REMOVE(
    mut head: *mut format_job_tree,
    mut elm: *mut format_job,
) -> *mut format_job {
    let mut current_block: u64;
    let mut child: *mut format_job = ::core::ptr::null_mut::<format_job>();
    let mut parent: *mut format_job = ::core::ptr::null_mut::<format_job>();
    let mut old: *mut format_job = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut format_job = ::core::ptr::null_mut::<format_job>();
        elm = (*elm).entry.rbe_right;
        loop {
            left = (*elm).entry.rbe_left;
            if left.is_null() {
                break;
            }
            elm = left;
        }
        child = (*elm).entry.rbe_right;
        parent = (*elm).entry.rbe_parent;
        color = (*elm).entry.rbe_color;
        if !child.is_null() {
            (*child).entry.rbe_parent = parent;
        }
        if !parent.is_null() {
            if (*parent).entry.rbe_left == elm {
                (*parent).entry.rbe_left = child;
            } else {
                (*parent).entry.rbe_right = child;
            }
        } else {
            (*head).rbh_root = child;
        }
        if (*elm).entry.rbe_parent == old {
            parent = elm;
        }
        (*elm).entry = (*old).entry;
        if !(*old).entry.rbe_parent.is_null() {
            if (*(*old).entry.rbe_parent).entry.rbe_left == old {
                (*(*old).entry.rbe_parent).entry.rbe_left = elm;
            } else {
                (*(*old).entry.rbe_parent).entry.rbe_right = elm;
            }
        } else {
            (*head).rbh_root = elm;
        }
        (*(*old).entry.rbe_left).entry.rbe_parent = elm;
        if !(*old).entry.rbe_right.is_null() {
            (*(*old).entry.rbe_right).entry.rbe_parent = elm;
        }
        if !parent.is_null() {
            left = parent;
            loop {
                left = (*left).entry.rbe_parent;
                if left.is_null() {
                    break;
                }
            }
        }
        current_block = 4175337610307587336;
    }
    match current_block {
        7245201122033322888 => {
            parent = (*elm).entry.rbe_parent;
            color = (*elm).entry.rbe_color;
            if !child.is_null() {
                (*child).entry.rbe_parent = parent;
            }
            if !parent.is_null() {
                if (*parent).entry.rbe_left == elm {
                    (*parent).entry.rbe_left = child;
                } else {
                    (*parent).entry.rbe_right = child;
                }
            } else {
                (*head).rbh_root = child;
            }
        }
        _ => {}
    }
    if color == RB_BLACK {
        format_job_tree_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn format_job_tree_RB_INSERT_COLOR(
    mut head: *mut format_job_tree,
    mut elm: *mut format_job,
) {
    let mut parent: *mut format_job = ::core::ptr::null_mut::<format_job>();
    let mut gparent: *mut format_job = ::core::ptr::null_mut::<format_job>();
    let mut tmp: *mut format_job = ::core::ptr::null_mut::<format_job>();
    loop {
        parent = (*elm).entry.rbe_parent;
        if !(!parent.is_null() && (*parent).entry.rbe_color == RB_RED) {
            break;
        }
        gparent = (*parent).entry.rbe_parent;
        if parent == (*gparent).entry.rbe_left {
            tmp = (*gparent).entry.rbe_right;
            if !tmp.is_null() && (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).entry.rbe_right == elm {
                    tmp = (*parent).entry.rbe_right;
                    (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                    if !(*parent).entry.rbe_right.is_null() {
                        (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                    }
                    (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                    if !(*tmp).entry.rbe_parent.is_null() {
                        if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                            (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                        } else {
                            (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).entry.rbe_left = parent;
                    (*parent).entry.rbe_parent = tmp;
                    !(*tmp).entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                tmp = (*gparent).entry.rbe_left;
                (*gparent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*gparent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = gparent;
                }
                (*tmp).entry.rbe_parent = (*gparent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).entry.rbe_parent).entry.rbe_left {
                        (*(*gparent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = gparent;
                (*gparent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
            }
        } else {
            tmp = (*gparent).entry.rbe_left;
            if !tmp.is_null() && (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).entry.rbe_left == elm {
                    tmp = (*parent).entry.rbe_left;
                    (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                    if !(*parent).entry.rbe_left.is_null() {
                        (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                    }
                    (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                    if !(*tmp).entry.rbe_parent.is_null() {
                        if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                            (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                        } else {
                            (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).entry.rbe_right = parent;
                    (*parent).entry.rbe_parent = tmp;
                    !(*tmp).entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                tmp = (*gparent).entry.rbe_right;
                (*gparent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*gparent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = gparent;
                }
                (*tmp).entry.rbe_parent = (*gparent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).entry.rbe_parent).entry.rbe_left {
                        (*(*gparent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = gparent;
                (*gparent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
            }
        }
    }
    (*(*head).rbh_root).entry.rbe_color = RB_BLACK;
}
unsafe extern "C" fn format_job_tree_RB_MINMAX(
    mut head: *mut format_job_tree,
    mut val: ::core::ffi::c_int,
) -> *mut format_job {
    let mut tmp: *mut format_job = (*head).rbh_root;
    let mut parent: *mut format_job = ::core::ptr::null_mut::<format_job>();
    while !tmp.is_null() {
        parent = tmp;
        if val < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else {
            tmp = (*tmp).entry.rbe_right;
        }
    }
    return parent;
}
unsafe extern "C" fn format_job_tree_RB_FIND(
    mut head: *mut format_job_tree,
    mut elm: *mut format_job,
) -> *mut format_job {
    let mut tmp: *mut format_job = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = format_job_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<format_job>();
}
unsafe extern "C" fn format_job_cmp(
    mut fj1: *mut format_job,
    mut fj2: *mut format_job,
) -> ::core::ffi::c_int {
    if (*fj1).tag < (*fj2).tag {
        return -(1 as ::core::ffi::c_int);
    }
    if (*fj1).tag > (*fj2).tag {
        return 1 as ::core::ffi::c_int;
    }
    return strcmp((*fj1).cmd, (*fj2).cmd);
}

static mut sort_crit: sort_criteria = sort_criteria {
    order: SORT_ACTIVITY,
    reversed: 0,
    order_seq: ::core::ptr::null::<sort_order>() as *mut sort_order,
};
unsafe extern "C" fn format_entry_tree_RB_REMOVE(
    mut head: *mut format_entry_tree,
    mut elm: *mut format_entry,
) -> *mut format_entry {
    let mut current_block: u64;
    let mut child: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut parent: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut old: *mut format_entry = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
        elm = (*elm).entry.rbe_right;
        loop {
            left = (*elm).entry.rbe_left;
            if left.is_null() {
                break;
            }
            elm = left;
        }
        child = (*elm).entry.rbe_right;
        parent = (*elm).entry.rbe_parent;
        color = (*elm).entry.rbe_color;
        if !child.is_null() {
            (*child).entry.rbe_parent = parent;
        }
        if !parent.is_null() {
            if (*parent).entry.rbe_left == elm {
                (*parent).entry.rbe_left = child;
            } else {
                (*parent).entry.rbe_right = child;
            }
        } else {
            (*head).rbh_root = child;
        }
        if (*elm).entry.rbe_parent == old {
            parent = elm;
        }
        (*elm).entry = (*old).entry;
        if !(*old).entry.rbe_parent.is_null() {
            if (*(*old).entry.rbe_parent).entry.rbe_left == old {
                (*(*old).entry.rbe_parent).entry.rbe_left = elm;
            } else {
                (*(*old).entry.rbe_parent).entry.rbe_right = elm;
            }
        } else {
            (*head).rbh_root = elm;
        }
        (*(*old).entry.rbe_left).entry.rbe_parent = elm;
        if !(*old).entry.rbe_right.is_null() {
            (*(*old).entry.rbe_right).entry.rbe_parent = elm;
        }
        if !parent.is_null() {
            left = parent;
            loop {
                left = (*left).entry.rbe_parent;
                if left.is_null() {
                    break;
                }
            }
        }
        current_block = 9333106395061089618;
    }
    match current_block {
        7245201122033322888 => {
            parent = (*elm).entry.rbe_parent;
            color = (*elm).entry.rbe_color;
            if !child.is_null() {
                (*child).entry.rbe_parent = parent;
            }
            if !parent.is_null() {
                if (*parent).entry.rbe_left == elm {
                    (*parent).entry.rbe_left = child;
                } else {
                    (*parent).entry.rbe_right = child;
                }
            } else {
                (*head).rbh_root = child;
            }
        }
        _ => {}
    }
    if color == RB_BLACK {
        format_entry_tree_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn format_entry_tree_RB_FIND(
    mut head: *mut format_entry_tree,
    mut elm: *mut format_entry,
) -> *mut format_entry {
    let mut tmp: *mut format_entry = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = format_entry_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<format_entry>();
}
unsafe extern "C" fn format_entry_tree_RB_INSERT_COLOR(
    mut head: *mut format_entry_tree,
    mut elm: *mut format_entry,
) {
    let mut parent: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut gparent: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut tmp: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    loop {
        parent = (*elm).entry.rbe_parent;
        if !(!parent.is_null() && (*parent).entry.rbe_color == RB_RED) {
            break;
        }
        gparent = (*parent).entry.rbe_parent;
        if parent == (*gparent).entry.rbe_left {
            tmp = (*gparent).entry.rbe_right;
            if !tmp.is_null() && (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).entry.rbe_right == elm {
                    tmp = (*parent).entry.rbe_right;
                    (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                    if !(*parent).entry.rbe_right.is_null() {
                        (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                    }
                    (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                    if !(*tmp).entry.rbe_parent.is_null() {
                        if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                            (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                        } else {
                            (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).entry.rbe_left = parent;
                    (*parent).entry.rbe_parent = tmp;
                    !(*tmp).entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                tmp = (*gparent).entry.rbe_left;
                (*gparent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*gparent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = gparent;
                }
                (*tmp).entry.rbe_parent = (*gparent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).entry.rbe_parent).entry.rbe_left {
                        (*(*gparent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = gparent;
                (*gparent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
            }
        } else {
            tmp = (*gparent).entry.rbe_left;
            if !tmp.is_null() && (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).entry.rbe_left == elm {
                    tmp = (*parent).entry.rbe_left;
                    (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                    if !(*parent).entry.rbe_left.is_null() {
                        (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                    }
                    (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                    if !(*tmp).entry.rbe_parent.is_null() {
                        if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                            (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                        } else {
                            (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).entry.rbe_right = parent;
                    (*parent).entry.rbe_parent = tmp;
                    !(*tmp).entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                tmp = (*gparent).entry.rbe_right;
                (*gparent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*gparent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = gparent;
                }
                (*tmp).entry.rbe_parent = (*gparent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).entry.rbe_parent).entry.rbe_left {
                        (*(*gparent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = gparent;
                (*gparent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
            }
        }
    }
    (*(*head).rbh_root).entry.rbe_color = RB_BLACK;
}
unsafe extern "C" fn format_entry_tree_RB_MINMAX(
    mut head: *mut format_entry_tree,
    mut val: ::core::ffi::c_int,
) -> *mut format_entry {
    let mut tmp: *mut format_entry = (*head).rbh_root;
    let mut parent: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    while !tmp.is_null() {
        parent = tmp;
        if val < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else {
            tmp = (*tmp).entry.rbe_right;
        }
    }
    return parent;
}
unsafe extern "C" fn format_entry_tree_RB_REMOVE_COLOR(
    mut head: *mut format_entry_tree,
    mut parent: *mut format_entry,
    mut elm: *mut format_entry,
) {
    let mut tmp: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    while (elm.is_null() || (*elm).entry.rbe_color == RB_BLACK) && elm != (*head).rbh_root {
        if (*parent).entry.rbe_left == elm {
            tmp = (*parent).entry.rbe_right;
            if (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_RED;
                tmp = (*parent).entry.rbe_right;
                (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*parent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                tmp = (*parent).entry.rbe_right;
            }
            if ((*tmp).entry.rbe_left.is_null()
                || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK)
                && ((*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK)
            {
                (*tmp).entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).entry.rbe_parent;
            } else {
                if (*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK
                {
                    let mut oleft: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
                    oleft = (*tmp).entry.rbe_left;
                    if !oleft.is_null() {
                        (*oleft).entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).entry.rbe_color = RB_RED;
                    oleft = (*tmp).entry.rbe_left;
                    (*tmp).entry.rbe_left = (*oleft).entry.rbe_right;
                    if !(*tmp).entry.rbe_left.is_null() {
                        (*(*oleft).entry.rbe_right).entry.rbe_parent = tmp;
                    }
                    (*oleft).entry.rbe_parent = (*tmp).entry.rbe_parent;
                    if !(*oleft).entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).entry.rbe_parent).entry.rbe_left {
                            (*(*tmp).entry.rbe_parent).entry.rbe_left = oleft;
                        } else {
                            (*(*tmp).entry.rbe_parent).entry.rbe_right = oleft;
                        }
                    } else {
                        (*head).rbh_root = oleft;
                    }
                    (*oleft).entry.rbe_right = tmp;
                    (*tmp).entry.rbe_parent = oleft;
                    !(*oleft).entry.rbe_parent.is_null();
                    tmp = (*parent).entry.rbe_right;
                }
                (*tmp).entry.rbe_color = (*parent).entry.rbe_color;
                (*parent).entry.rbe_color = RB_BLACK;
                if !(*tmp).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).entry.rbe_right;
                (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*parent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        } else {
            tmp = (*parent).entry.rbe_left;
            if (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_RED;
                tmp = (*parent).entry.rbe_left;
                (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*parent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                tmp = (*parent).entry.rbe_left;
            }
            if ((*tmp).entry.rbe_left.is_null()
                || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK)
                && ((*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK)
            {
                (*tmp).entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).entry.rbe_parent;
            } else {
                if (*tmp).entry.rbe_left.is_null()
                    || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK
                {
                    let mut oright: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
                    oright = (*tmp).entry.rbe_right;
                    if !oright.is_null() {
                        (*oright).entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).entry.rbe_color = RB_RED;
                    oright = (*tmp).entry.rbe_right;
                    (*tmp).entry.rbe_right = (*oright).entry.rbe_left;
                    if !(*tmp).entry.rbe_right.is_null() {
                        (*(*oright).entry.rbe_left).entry.rbe_parent = tmp;
                    }
                    (*oright).entry.rbe_parent = (*tmp).entry.rbe_parent;
                    if !(*oright).entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).entry.rbe_parent).entry.rbe_left {
                            (*(*tmp).entry.rbe_parent).entry.rbe_left = oright;
                        } else {
                            (*(*tmp).entry.rbe_parent).entry.rbe_right = oright;
                        }
                    } else {
                        (*head).rbh_root = oright;
                    }
                    (*oright).entry.rbe_left = tmp;
                    (*tmp).entry.rbe_parent = oright;
                    !(*oright).entry.rbe_parent.is_null();
                    tmp = (*parent).entry.rbe_left;
                }
                (*tmp).entry.rbe_color = (*parent).entry.rbe_color;
                (*parent).entry.rbe_color = RB_BLACK;
                if !(*tmp).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).entry.rbe_left;
                (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*parent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        }
    }
    if !elm.is_null() {
        (*elm).entry.rbe_color = RB_BLACK;
    }
}
unsafe extern "C" fn format_entry_tree_RB_NEXT(mut elm: *mut format_entry) -> *mut format_entry {
    if !(*elm).entry.rbe_right.is_null() {
        elm = (*elm).entry.rbe_right;
        while !(*elm).entry.rbe_left.is_null() {
            elm = (*elm).entry.rbe_left;
        }
    } else if !(*elm).entry.rbe_parent.is_null() && elm == (*(*elm).entry.rbe_parent).entry.rbe_left
    {
        elm = (*elm).entry.rbe_parent;
    } else {
        while !(*elm).entry.rbe_parent.is_null()
            && elm == (*(*elm).entry.rbe_parent).entry.rbe_right
        {
            elm = (*elm).entry.rbe_parent;
        }
        elm = (*elm).entry.rbe_parent;
    }
    return elm;
}
unsafe extern "C" fn format_entry_tree_RB_INSERT(
    mut head: *mut format_entry_tree,
    mut elm: *mut format_entry,
) -> *mut format_entry {
    let mut tmp: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut parent: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = format_entry_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<format_entry>();
    (*elm).entry.rbe_left = (*elm).entry.rbe_right;
    (*elm).entry.rbe_color = RB_RED;
    if !parent.is_null() {
        if comp < 0 as ::core::ffi::c_int {
            (*parent).entry.rbe_left = elm;
        } else {
            (*parent).entry.rbe_right = elm;
        }
    } else {
        (*head).rbh_root = elm;
    }
    format_entry_tree_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<format_entry>();
}
unsafe extern "C" fn format_entry_cmp(
    mut fe1: *mut format_entry,
    mut fe2: *mut format_entry,
) -> ::core::ffi::c_int {
    return strcmp((*fe1).key, (*fe2).key);
}
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
unsafe extern "C" fn format_logging(mut ft: *mut format_tree) -> ::core::ffi::c_int {
    return (log_get_level() != 0 as ::core::ffi::c_int || (*ft).flags & FORMAT_VERBOSE != 0)
        as ::core::ffi::c_int;
}
unsafe extern "C" fn format_log1(
    mut es: *mut format_expand_state,
    mut from: *const ::core::ffi::c_char,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ft: *mut format_tree = (*es).ft;
    let mut ap: ::core::ffi::VaList;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static mut spaces: [::core::ffi::c_char; 11] =
        unsafe { ::core::mem::transmute::<[u8; 11], [::core::ffi::c_char; 11]>(*b"          \0") };
    if format_logging(ft) == 0 {
        return;
    }
    ap = args.clone();
    xvasprintf(&raw mut s, fmt, ap);
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        from,
        s,
    );
    if !(*ft).item.is_null() && (*ft).flags & FORMAT_VERBOSE != 0 {
        cmdq_print(
            (*ft).item,
            b"#%.*s%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*es).loop_0,
            &raw const spaces as *const ::core::ffi::c_char,
            s,
        );
    }
    free(s as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn format_copy_state(
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
unsafe extern "C" fn format_job_update(mut job: *mut job) {
    let mut fj: *mut format_job = job_get_data(job) as *mut format_job;
    let mut evb: *mut evbuffer = (*job_get_event(job)).input;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut t: time_t = 0;
    loop {
        next = evbuffer_readline(evb);
        if next.is_null() {
            break;
        }
        free(line as *mut ::core::ffi::c_void);
        line = next;
    }
    if line.is_null() {
        return;
    }
    (*fj).updated = 1 as ::core::ffi::c_int;
    free((*fj).out as *mut ::core::ffi::c_void);
    (*fj).out = line;
    log_debug(
        b"%s: %p %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"format_job_update\0" as *const u8 as *const ::core::ffi::c_char,
        fj,
        (*fj).cmd,
        (*fj).out,
    );
    t = time(::core::ptr::null_mut::<time_t>());
    if (*fj).status != 0 && (*fj).last != t {
        if !(*fj).client.is_null() {
            server_status_client((*fj).client);
        }
        (*fj).last = t;
    }
}
unsafe extern "C" fn format_job_complete(mut job: *mut job) {
    let mut fj: *mut format_job = job_get_data(job) as *mut format_job;
    let mut evb: *mut evbuffer = (*job_get_event(job)).input;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    (*fj).job = ::core::ptr::null_mut::<job>();
    buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
    line = evbuffer_readline(evb);
    if line.is_null() {
        len = evbuffer_get_length(evb);
        buf = xmalloc(len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        if len != 0 as size_t {
            memcpy(
                buf as *mut ::core::ffi::c_void,
                evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t)
                    as *const ::core::ffi::c_void,
                len,
            );
        }
        *buf.offset(len as isize) = '\0' as i32 as ::core::ffi::c_char;
    } else {
        buf = line;
    }
    log_debug(
        b"%s: %p %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"format_job_complete\0" as *const u8 as *const ::core::ffi::c_char,
        fj,
        (*fj).cmd,
        buf,
    );
    if *buf as ::core::ffi::c_int != '\0' as i32 || (*fj).updated == 0 {
        free((*fj).out as *mut ::core::ffi::c_void);
        (*fj).out = buf;
    } else {
        free(buf as *mut ::core::ffi::c_void);
    }
    if (*fj).status != 0 {
        if !(*fj).client.is_null() {
            server_status_client((*fj).client);
        }
        (*fj).status = 0 as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn format_job_get(
    mut es: *mut format_expand_state,
    mut cmd: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut ft: *mut format_tree = (*es).ft;
    let mut jobs: *mut format_job_tree = ::core::ptr::null_mut::<format_job_tree>();
    let mut fj0: format_job = format_job {
        client: ::core::ptr::null_mut::<client>(),
        tag: 0,
        cmd: ::core::ptr::null::<::core::ffi::c_char>(),
        expanded: ::core::ptr::null::<::core::ffi::c_char>(),
        last: 0,
        out: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        updated: 0,
        job: ::core::ptr::null_mut::<job>(),
        status: 0,
        entry: format_job_entry {
            rbe_left: ::core::ptr::null_mut::<format_job>(),
            rbe_right: ::core::ptr::null_mut::<format_job>(),
            rbe_parent: ::core::ptr::null_mut::<format_job>(),
            rbe_color: 0,
        },
    };
    let mut fj: *mut format_job = ::core::ptr::null_mut::<format_job>();
    let mut t: time_t = 0;
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut force: ::core::ffi::c_int = 0;
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    if (*ft).client.is_null() {
        jobs = &raw mut format_jobs as *mut format_job_tree;
    } else if !(*(*ft).client).jobs.is_null() {
        jobs = (*(*ft).client).jobs;
    } else {
        (*(*ft).client).jobs =
            xmalloc(::core::mem::size_of::<format_job_tree>() as size_t) as *mut format_job_tree;
        jobs = (*(*ft).client).jobs;
        (*jobs).rbh_root = ::core::ptr::null_mut::<format_job>();
    }
    fj0.tag = (*ft).tag;
    fj0.cmd = cmd;
    fj = format_job_tree_RB_FIND(jobs, &raw mut fj0);
    if fj.is_null() {
        fj =
            xcalloc(1 as size_t, ::core::mem::size_of::<format_job>() as size_t) as *mut format_job;
        (*fj).client = (*ft).client;
        (*fj).tag = (*ft).tag;
        (*fj).cmd = xstrdup(cmd);
        format_job_tree_RB_INSERT(jobs, fj);
    }
    format_copy_state(
        &raw mut next,
        es,
        FORMAT_EXPAND_NOJOBS | FORMAT_EXPAND_NOCYCLE,
    );
    next.flags &= !FORMAT_EXPAND_TIME;
    expanded = format_expand1(&raw mut next, cmd);
    if (*fj).expanded.is_null() || strcmp(expanded, (*fj).expanded) != 0 as ::core::ffi::c_int {
        free((*fj).expanded as *mut ::core::ffi::c_void);
        (*fj).expanded = xstrdup(expanded);
        force = 1 as ::core::ffi::c_int;
    } else {
        force = (*ft).flags & FORMAT_FORCE;
    }
    t = time(::core::ptr::null_mut::<time_t>());
    if force != 0 && !(*fj).job.is_null() {
        job_free((*fj).job);
    }
    if force != 0 || (*fj).job.is_null() && (*fj).last != t {
        (*fj).job = job_run(
            expanded,
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<environ>(),
            ::core::ptr::null_mut::<session>(),
            server_client_get_cwd((*ft).client, ::core::ptr::null_mut::<session>()),
            Some(format_job_update as unsafe extern "C" fn(*mut job) -> ()),
            Some(format_job_complete as unsafe extern "C" fn(*mut job) -> ()),
            None,
            fj as *mut ::core::ffi::c_void,
            JOB_NOWAIT,
            -(1 as ::core::ffi::c_int),
            -(1 as ::core::ffi::c_int),
        );
        if (*fj).job.is_null() {
            free((*fj).out as *mut ::core::ffi::c_void);
            xasprintf(
                &raw mut (*fj).out,
                b"<'%s' didn't start>\0" as *const u8 as *const ::core::ffi::c_char,
                (*fj).cmd,
            );
        }
        (*fj).last = t;
        (*fj).updated = 0 as ::core::ffi::c_int;
    } else if !(*fj).job.is_null() && t - (*fj).last > 1 as time_t && (*fj).out.is_null() {
        xasprintf(
            &raw mut (*fj).out,
            b"<'%s' not ready>\0" as *const u8 as *const ::core::ffi::c_char,
            (*fj).cmd,
        );
    }
    free(expanded as *mut ::core::ffi::c_void);
    if (*ft).flags & FORMAT_STATUS != 0 {
        (*fj).status = 1 as ::core::ffi::c_int;
    }
    if (*fj).out.is_null() {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    return format_expand1(&raw mut next, (*fj).out);
}
unsafe extern "C" fn format_job_tidy(
    mut jobs: *mut format_job_tree,
    mut force: ::core::ffi::c_int,
) {
    let mut fj: *mut format_job = ::core::ptr::null_mut::<format_job>();
    let mut fj1: *mut format_job = ::core::ptr::null_mut::<format_job>();
    let mut now: time_t = 0;
    now = time(::core::ptr::null_mut::<time_t>());
    fj = format_job_tree_RB_MINMAX(jobs, RB_NEGINF);
    while !fj.is_null() && {
        fj1 = format_job_tree_RB_NEXT(fj);
        1 as ::core::ffi::c_int != 0
    } {
        if !(force == 0 && ((*fj).last > now || now - (*fj).last < 3600 as time_t)) {
            format_job_tree_RB_REMOVE(jobs, fj);
            log_debug(
                b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                b"format_job_tidy\0" as *const u8 as *const ::core::ffi::c_char,
                (*fj).cmd,
            );
            if !(*fj).job.is_null() {
                job_free((*fj).job);
            }
            free((*fj).expanded as *mut ::core::ffi::c_void);
            free((*fj).cmd as *mut ::core::ffi::c_void);
            free((*fj).out as *mut ::core::ffi::c_void);
            free(fj as *mut ::core::ffi::c_void);
        }
        fj = fj1;
    }
}
unsafe extern "C" fn format_strftime(
    mut s: *mut ::core::ffi::c_char,
    mut max: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut tm: *const tm,
) -> size_t {
    return strftime(s, max, fmt, tm);
}
#[no_mangle]
pub unsafe extern "C" fn format_tidy_jobs() {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    format_job_tidy(&raw mut format_jobs, 0 as ::core::ffi::c_int);
    c = clients.tqh_first;
    while !c.is_null() {
        if !(*c).jobs.is_null() {
            format_job_tidy((*c).jobs, 0 as ::core::ffi::c_int);
        }
        c = (*c).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn format_lost_client(mut c: *mut client) {
    if !(*c).jobs.is_null() {
        format_job_tidy((*c).jobs, 1 as ::core::ffi::c_int);
    }
    free((*c).jobs as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn format_printf(
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> *mut ::core::ffi::c_char {
    let mut ap: ::core::ffi::VaList;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ap = args.clone();
    xvasprintf(&raw mut s, fmt, ap);
    return s;
}
unsafe extern "C" fn format_cb_host(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut host: [::core::ffi::c_char; 65] = [0; 65];
    if gethostname(
        &raw mut host as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 65]>() as size_t,
    ) != 0 as ::core::ffi::c_int
    {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return xstrdup(&raw mut host as *mut ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_host_short(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut host: [::core::ffi::c_char; 65] = [0; 65];
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if gethostname(
        &raw mut host as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 65]>() as size_t,
    ) != 0 as ::core::ffi::c_int
    {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    cp = strchr(&raw mut host as *mut ::core::ffi::c_char, '.' as i32);
    if !cp.is_null() {
        *cp = '\0' as i32 as ::core::ffi::c_char;
    }
    return xstrdup(&raw mut host as *mut ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pid(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    xasprintf(
        &raw mut value,
        b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
        getpid() as ::core::ffi::c_long,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_attached_list(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut s: *mut session = (*ft).s;
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if s.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        if (*loop_0).session == s {
            if evbuffer_get_length(buffer) > 0 as size_t {
                evbuffer_add(
                    buffer,
                    b",\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                    1 as size_t,
                );
            }
            evbuffer_add_printf(
                buffer,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*loop_0).name,
            );
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    size = evbuffer_get_length(buffer) as ::core::ffi::c_int;
    if size != 0 as ::core::ffi::c_int {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size as size_t,
        );
    }
    evbuffer_free(buffer);
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_alert(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut s: *mut session = (*ft).s;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut alerts: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut alerted: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if s.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    *(&raw mut alerts as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        if !((*wl).flags & WINLINK_ALERTFLAGS == 0 as ::core::ffi::c_int) {
            if !alerted & (*wl).flags & WINLINK_ACTIVITY != 0 {
                strlcat(
                    &raw mut alerts as *mut ::core::ffi::c_char,
                    b"#\0" as *const u8 as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
                );
                alerted |= WINLINK_ACTIVITY;
            }
            if !alerted & (*wl).flags & WINLINK_BELL != 0 {
                strlcat(
                    &raw mut alerts as *mut ::core::ffi::c_char,
                    b"!\0" as *const u8 as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
                );
                alerted |= WINLINK_BELL;
            }
            if !alerted & (*wl).flags & WINLINK_SILENCE != 0 {
                strlcat(
                    &raw mut alerts as *mut ::core::ffi::c_char,
                    b"~\0" as *const u8 as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
                );
                alerted |= WINLINK_SILENCE;
            }
        }
        wl = winlinks_RB_NEXT(wl);
    }
    return xstrdup(&raw mut alerts as *mut ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_alerts(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut s: *mut session = (*ft).s;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut alerts: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut tmp: [::core::ffi::c_char; 16] = [0; 16];
    if s.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    *(&raw mut alerts as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        if !((*wl).flags & WINLINK_ALERTFLAGS == 0 as ::core::ffi::c_int) {
            xsnprintf(
                &raw mut tmp as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*wl).idx,
            );
            if *(&raw mut alerts as *mut ::core::ffi::c_char) as ::core::ffi::c_int != '\0' as i32 {
                strlcat(
                    &raw mut alerts as *mut ::core::ffi::c_char,
                    b",\0" as *const u8 as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
                );
            }
            strlcat(
                &raw mut alerts as *mut ::core::ffi::c_char,
                &raw mut tmp as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
            );
            if (*wl).flags & WINLINK_ACTIVITY != 0 {
                strlcat(
                    &raw mut alerts as *mut ::core::ffi::c_char,
                    b"#\0" as *const u8 as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
                );
            }
            if (*wl).flags & WINLINK_BELL != 0 {
                strlcat(
                    &raw mut alerts as *mut ::core::ffi::c_char,
                    b"!\0" as *const u8 as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
                );
            }
            if (*wl).flags & WINLINK_SILENCE != 0 {
                strlcat(
                    &raw mut alerts as *mut ::core::ffi::c_char,
                    b"~\0" as *const u8 as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
                );
            }
        }
        wl = winlinks_RB_NEXT(wl);
    }
    return xstrdup(&raw mut alerts as *mut ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_stack(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut s: *mut session = (*ft).s;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut result: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut tmp: [::core::ffi::c_char; 16] = [0; 16];
    if s.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    xsnprintf(
        &raw mut result as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*(*s).curw).idx,
    );
    wl = (*s).lastw.tqh_first;
    while !wl.is_null() {
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wl).idx,
        );
        if *(&raw mut result as *mut ::core::ffi::c_char) as ::core::ffi::c_int != '\0' as i32 {
            strlcat(
                &raw mut result as *mut ::core::ffi::c_char,
                b",\0" as *const u8 as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
            );
        }
        strlcat(
            &raw mut result as *mut ::core::ffi::c_char,
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
        wl = (*wl).sentry.tqe_next;
    }
    return xstrdup(&raw mut result as *mut ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_stack_index(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut idx: u_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*ft).wl.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    s = (*(*ft).wl).session;
    idx = 0 as u_int;
    wl = (*s).lastw.tqh_first;
    while !wl.is_null() {
        idx = idx.wrapping_add(1);
        if wl == (*ft).wl {
            break;
        }
        wl = (*wl).sentry.tqe_next;
    }
    if wl.is_null() {
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    xasprintf(
        &raw mut value,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        idx,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_linked_sessions_list(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*ft).wl.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    w = (*(*ft).wl).window;
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if evbuffer_get_length(buffer) > 0 as size_t {
            evbuffer_add(
                buffer,
                b",\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
        }
        evbuffer_add_printf(
            buffer,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*wl).session).name,
        );
        wl = (*wl).wentry.tqe_next;
    }
    size = evbuffer_get_length(buffer) as ::core::ffi::c_int;
    if size != 0 as ::core::ffi::c_int {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size as size_t,
        );
    }
    evbuffer_free(buffer);
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_active_sessions(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut n: u_int = 0 as u_int;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*ft).wl.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    w = (*(*ft).wl).window;
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if (*(*wl).session).curw == wl {
            n = n.wrapping_add(1);
        }
        wl = (*wl).wentry.tqe_next;
    }
    xasprintf(
        &raw mut value,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        n,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_active_sessions_list(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*ft).wl.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    w = (*(*ft).wl).window;
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if (*(*wl).session).curw == wl {
            if evbuffer_get_length(buffer) > 0 as size_t {
                evbuffer_add(
                    buffer,
                    b",\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                    1 as size_t,
                );
            }
            evbuffer_add_printf(
                buffer,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*(*wl).session).name,
            );
        }
        wl = (*wl).wentry.tqe_next;
    }
    size = evbuffer_get_length(buffer) as ::core::ffi::c_int;
    if size != 0 as ::core::ffi::c_int {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size as size_t,
        );
    }
    evbuffer_free(buffer);
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_active_clients(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut client_session: *mut session = ::core::ptr::null_mut::<session>();
    let mut n: u_int = 0 as u_int;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*ft).wl.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    w = (*(*ft).wl).window;
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        client_session = (*loop_0).session;
        if !client_session.is_null() {
            if w == (*(*client_session).curw).window {
                n = n.wrapping_add(1);
            }
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    xasprintf(
        &raw mut value,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        n,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_active_clients_list(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut client_session: *mut session = ::core::ptr::null_mut::<session>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*ft).wl.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    w = (*(*ft).wl).window;
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        client_session = (*loop_0).session;
        if !client_session.is_null() {
            if w == (*(*client_session).curw).window {
                if evbuffer_get_length(buffer) > 0 as size_t {
                    evbuffer_add(
                        buffer,
                        b",\0" as *const u8 as *const ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        1 as size_t,
                    );
                }
                evbuffer_add_printf(
                    buffer,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*loop_0).name,
                );
            }
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    size = evbuffer_get_length(buffer) as ::core::ffi::c_int;
    if size != 0 as ::core::ffi::c_int {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size as size_t,
        );
    }
    evbuffer_free(buffer);
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_layout(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut c: *mut client = (*ft).client;
    let mut w: *mut window = (*ft).w;
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if w.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if !(*w).saved_layout_root.is_null() {
        lcroot = (*w).saved_layout_root;
    } else {
        lcroot = (*w).layout_root;
    }
    if !c.is_null()
        && (*c).flags & CLIENT_CONTROL as uint64_t != 0
        && !(*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_NEWLAYOUTS != 0
    {
        flags |= LAYOUT_CUSTOM_OLD_FORMAT;
    }
    return layout_dump(w, lcroot, flags) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_visible_layout(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut c: *mut client = (*ft).client;
    let mut w: *mut window = (*ft).w;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if w.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if !c.is_null()
        && (*c).flags & CLIENT_CONTROL as uint64_t != 0
        && !(*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_NEWLAYOUTS != 0
    {
        flags |= LAYOUT_CUSTOM_OLD_FORMAT;
    }
    return layout_dump(w, (*w).layout_root, flags) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_start_command(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return cmd_stringify_argv((*wp).argc, (*wp).argv) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_start_command_list(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0 as size_t;
    let mut i: ::core::ffi::c_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*wp).argc == 0 as ::core::ffi::c_int {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    i = 0 as ::core::ffi::c_int;
    while i < (*wp).argc {
        s = format_quote_shell_single(*(*wp).argv.offset(i as isize));
        len = len.wrapping_add(strlen(s).wrapping_add(1 as size_t));
        buf = xrealloc(buf as *mut ::core::ffi::c_void, len) as *mut ::core::ffi::c_char;
        if i == 0 as ::core::ffi::c_int {
            *buf = '\0' as i32 as ::core::ffi::c_char;
        } else {
            strlcat(buf, b" \0" as *const u8 as *const ::core::ffi::c_char, len);
        }
        strlcat(buf, s, len);
        free(s as *mut ::core::ffi::c_void);
        i += 1;
    }
    return buf as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_start_path(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*wp).cwd.is_null() {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return xstrdup((*wp).cwd) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_current_command(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() || (*wp).shell.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    cmd = osdep_get_name((*wp).fd, &raw mut (*wp).tty as *mut ::core::ffi::c_char);
    if cmd.is_null() || *cmd as ::core::ffi::c_int == '\0' as i32 {
        free(cmd as *mut ::core::ffi::c_void);
        cmd = cmd_stringify_argv((*wp).argc, (*wp).argv);
        if cmd.is_null() || *cmd as ::core::ffi::c_int == '\0' as i32 {
            free(cmd as *mut ::core::ffi::c_void);
            cmd = xstrdup((*wp).shell);
        }
    }
    value = parse_window_name(cmd);
    free(cmd as *mut ::core::ffi::c_void);
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_current_path(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut cwd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    cwd = osdep_get_cwd((*wp).fd);
    if cwd.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return xstrdup(cwd) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_history_bytes(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut size: size_t = 0 as size_t;
    let mut i: u_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    gd = (*wp).base.grid;
    i = 0 as u_int;
    while i < (*gd).hsize.wrapping_add((*gd).sy) {
        gl = grid_get_line(gd, i);
        size = (size as ::core::ffi::c_ulong).wrapping_add(
            ((*gl).cellsize as usize)
                .wrapping_mul(::core::mem::size_of::<grid_cell_entry>() as usize)
                as ::core::ffi::c_ulong,
        ) as size_t as size_t;
        size = (size as ::core::ffi::c_ulong).wrapping_add(
            ((*gl).extdsize as usize)
                .wrapping_mul(::core::mem::size_of::<grid_extd_entry>() as usize)
                as ::core::ffi::c_ulong,
        ) as size_t as size_t;
        i = i.wrapping_add(1);
    }
    size = (size as ::core::ffi::c_ulong).wrapping_add(
        ((*gd).hsize.wrapping_add((*gd).sy) as usize)
            .wrapping_mul(::core::mem::size_of::<grid_line>() as usize)
            as ::core::ffi::c_ulong,
    ) as size_t as size_t;
    xasprintf(
        &raw mut value,
        b"%zu\0" as *const u8 as *const ::core::ffi::c_char,
        size,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_history_all_bytes(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut i: u_int = 0;
    let mut lines: u_int = 0;
    let mut cells: u_int = 0 as u_int;
    let mut extended_cells: u_int = 0 as u_int;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    gd = (*wp).base.grid;
    lines = (*gd).hsize.wrapping_add((*gd).sy);
    i = 0 as u_int;
    while i < lines {
        gl = grid_get_line(gd, i);
        cells = cells.wrapping_add((*gl).cellsize as u_int);
        extended_cells = extended_cells.wrapping_add((*gl).extdsize);
        i = i.wrapping_add(1);
    }
    xasprintf(
        &raw mut value,
        b"%u,%zu,%u,%zu,%u,%zu\0" as *const u8 as *const ::core::ffi::c_char,
        lines,
        (lines as usize).wrapping_mul(::core::mem::size_of::<grid_line>() as usize),
        cells,
        (cells as usize).wrapping_mul(::core::mem::size_of::<grid_cell_entry>() as usize),
        extended_cells,
        (extended_cells as usize).wrapping_mul(::core::mem::size_of::<grid_extd_entry>() as usize),
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_tabs(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut i: u_int = 0;
    let mut size: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    i = 0 as u_int;
    while i < (*(*wp).base.grid).sx {
        if !(*(*wp)
            .base
            .tabs
            .offset((i >> 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << (i & 0x7 as u_int)
            == 0)
        {
            if evbuffer_get_length(buffer) > 0 as size_t {
                evbuffer_add(
                    buffer,
                    b",\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                    1 as size_t,
                );
            }
            evbuffer_add_printf(
                buffer,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                i,
            );
        }
        i = i.wrapping_add(1);
    }
    size = evbuffer_get_length(buffer) as ::core::ffi::c_int;
    if size != 0 as ::core::ffi::c_int {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size as size_t,
        );
    }
    evbuffer_free(buffer);
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_fg(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
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
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    tty_default_colours(&raw mut gc, wp, ::core::ptr::null_mut::<u_int>());
    return xstrdup(colour_tostring(gc.fg)) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_flags(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return xstrdup(window_pane_printable_flags((*ft).wp)) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_floating_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if window_pane_is_floating(wp) != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_modal_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if wp == (*(*wp).window).modal {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_bg(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
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
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    tty_default_colours(&raw mut gc, wp, ::core::ptr::null_mut::<u_int>());
    return xstrdup(colour_tostring(gc.bg)) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_group_list(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut s: *mut session = (*ft).s;
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut loop_0: *mut session = ::core::ptr::null_mut::<session>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if s.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    sg = session_group_contains(s);
    if sg.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    loop_0 = (*sg).sessions.tqh_first;
    while !loop_0.is_null() {
        if evbuffer_get_length(buffer) > 0 as size_t {
            evbuffer_add(
                buffer,
                b",\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
        }
        evbuffer_add_printf(
            buffer,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*loop_0).name,
        );
        loop_0 = (*loop_0).gentry.tqe_next;
    }
    size = evbuffer_get_length(buffer) as ::core::ffi::c_int;
    if size != 0 as ::core::ffi::c_int {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size as size_t,
        );
    }
    evbuffer_free(buffer);
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_group_attached_list(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut s: *mut session = (*ft).s;
    let mut client_session: *mut session = ::core::ptr::null_mut::<session>();
    let mut session_loop: *mut session = ::core::ptr::null_mut::<session>();
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if s.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    sg = session_group_contains(s);
    if sg.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        client_session = (*loop_0).session;
        if !client_session.is_null() {
            session_loop = (*sg).sessions.tqh_first;
            while !session_loop.is_null() {
                if session_loop == client_session {
                    if evbuffer_get_length(buffer) > 0 as size_t {
                        evbuffer_add(
                            buffer,
                            b",\0" as *const u8 as *const ::core::ffi::c_char
                                as *const ::core::ffi::c_void,
                            1 as size_t,
                        );
                    }
                    evbuffer_add_printf(
                        buffer,
                        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                        (*loop_0).name,
                    );
                }
                session_loop = (*session_loop).gentry.tqe_next;
            }
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    size = evbuffer_get_length(buffer) as ::core::ffi::c_int;
    if size != 0 as ::core::ffi::c_int {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size as size_t,
        );
    }
    evbuffer_free(buffer);
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_in_mode(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut n: u_int = 0 as u_int;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    wme = (*wp).modes.tqh_first;
    while !wme.is_null() {
        n = n.wrapping_add(1);
        wme = (*wme).entry.tqe_next;
    }
    xasprintf(
        &raw mut value,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        n,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_at_top(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut status: ::core::ffi::c_int = 0;
    let mut flag: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    status = window_pane_get_pane_status(wp);
    if status == PANE_STATUS_TOP {
        flag = ((*wp).yoff == 1 as ::core::ffi::c_int) as ::core::ffi::c_int;
    } else {
        flag = ((*wp).yoff == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
    xasprintf(
        &raw mut value,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        flag,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_at_bottom(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut status: ::core::ffi::c_int = 0;
    let mut flag: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    w = (*wp).window as *mut window;
    status = window_pane_get_pane_status(wp);
    if status == PANE_STATUS_BOTTOM {
        flag = ((*wp).yoff + (*wp).sy as ::core::ffi::c_int
            == (*w).sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
            as ::core::ffi::c_int;
    } else {
        flag = ((*wp).yoff + (*wp).sy as ::core::ffi::c_int == (*w).sy as ::core::ffi::c_int)
            as ::core::ffi::c_int;
    }
    xasprintf(
        &raw mut value,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        flag,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_cursor_character(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
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
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    grid_view_get_cell((*wp).base.grid, (*wp).base.cx, (*wp).base.cy, &raw mut gc);
    if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
        xasprintf(
            &raw mut value,
            b"%.*s\0" as *const u8 as *const ::core::ffi::c_char,
            gc.data.size as ::core::ffi::c_int,
            &raw mut gc.data.data as *mut u_char,
        );
    }
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_cursor_colour(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if wp.is_null() || (*wp).screen.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*(*wp).screen).ccolour != -(1 as ::core::ffi::c_int) {
        return xstrdup(colour_tostring((*(*wp).screen).ccolour)) as *mut ::core::ffi::c_void;
    }
    return xstrdup(colour_tostring((*(*wp).screen).default_ccolour)) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_mouse_word(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    wp = cmd_mouse_pane(
        &raw mut (*ft).m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if cmd_mouse_at(
        wp,
        &raw mut (*ft).m,
        &raw mut x,
        &raw mut y,
        0 as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if !(*wp).modes.tqh_first.is_null() {
        if window_pane_mode(wp) != WINDOW_PANE_NO_MODE {
            return window_copy_get_word(wp, x, y) as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    gd = (*wp).base.grid;
    return format_grid_word(gd, x, (*gd).hsize.wrapping_add(y)) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_mouse_hyperlink(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    wp = cmd_mouse_pane(
        &raw mut (*ft).m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if cmd_mouse_at(
        wp,
        &raw mut (*ft).m,
        &raw mut x,
        &raw mut y,
        0 as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if !(*wp).modes.tqh_first.is_null() {
        if window_pane_mode(wp) != WINDOW_PANE_NO_MODE {
            return window_copy_get_hyperlink(wp, x, y) as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    gd = (*wp).base.grid;
    return format_grid_hyperlink(gd, x, (*gd).hsize.wrapping_add(y), (*wp).screen)
        as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_mouse_line(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    wp = cmd_mouse_pane(
        &raw mut (*ft).m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if cmd_mouse_at(
        wp,
        &raw mut (*ft).m,
        &raw mut x,
        &raw mut y,
        0 as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if !(*wp).modes.tqh_first.is_null() {
        if window_pane_mode(wp) != WINDOW_PANE_NO_MODE {
            return window_copy_get_line(wp, y) as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    gd = (*wp).base.grid;
    return format_grid_line(gd, (*gd).hsize.wrapping_add(y)) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_mouse_status_line(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*ft).c.is_null() || !(*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*ft).m.statusat == 0 as ::core::ffi::c_int && (*ft).m.y < (*ft).m.statuslines {
        y = (*ft).m.y;
    } else if (*ft).m.statusat > 0 as ::core::ffi::c_int && (*ft).m.y >= (*ft).m.statusat as u_int {
        y = (*ft).m.y.wrapping_sub((*ft).m.statusat as u_int);
    } else {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    xasprintf(
        &raw mut value,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        y,
    );
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_mouse_status_range(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*ft).c.is_null() || !(*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*ft).m.statusat == 0 as ::core::ffi::c_int && (*ft).m.y < (*ft).m.statuslines {
        x = (*ft).m.x;
        y = (*ft).m.y;
    } else if (*ft).m.statusat > 0 as ::core::ffi::c_int && (*ft).m.y >= (*ft).m.statusat as u_int {
        x = (*ft).m.x;
        y = (*ft).m.y.wrapping_sub((*ft).m.statusat as u_int);
    } else {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    sr = status_get_range((*ft).c, x, y);
    if sr.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    match (*sr).type_0 as ::core::ffi::c_uint {
        0 => return ::core::ptr::null_mut::<::core::ffi::c_void>(),
        1 => {
            return xstrdup(b"left\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        2 => {
            return xstrdup(b"right\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        3 => {
            return xstrdup(b"pane\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        4 => {
            return xstrdup(b"window\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        5 => {
            return xstrdup(b"session\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        6 => {
            return xstrdup(&raw mut (*sr).string as *mut ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        7 => {
            return xstrdup(b"control\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        _ => {}
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_alternate_on(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if !(*(*ft).wp).base.saved_grid.is_null() {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_alternate_saved_x(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).base.saved_cx,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_alternate_saved_y(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).base.saved_cy,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_bracket_paste_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() && !(*(*ft).wp).screen.is_null() {
        if (*(*(*ft).wp).screen).mode & MODE_BRACKETPASTE != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_buffer_name(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).pb.is_null() {
        return xstrdup(paste_buffer_name((*ft).pb)) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_buffer_sample(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).pb.is_null() {
        return paste_make_sample((*ft).pb) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_buffer_full(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut size: size_t = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !(*ft).pb.is_null() {
        s = paste_buffer_data((*ft).pb, &raw mut size);
        if !s.is_null() {
            return xstrndup(s, size) as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_buffer_size(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut size: size_t = 0;
    if !(*ft).pb.is_null() {
        paste_buffer_data((*ft).pb, &raw mut size);
        return format_printf(b"%zu\0" as *const u8 as *const ::core::ffi::c_char, size)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_cell_height(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() && (*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).c).tty.ypixel,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_cell_width(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() && (*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).c).tty.xpixel,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_colours(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut term: *mut tty_term = ::core::ptr::null_mut::<tty_term>();
    let mut colours: u_int = 0;
    if (*ft).c.is_null() || !(*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    term = (*(*ft).c).tty.term;
    if (*term).flags & TERM_RGBCOLOURS != 0 {
        colours = 16777216 as ::core::ffi::c_int as u_int;
    } else if (*term).flags & TERM_256COLOURS != 0 {
        colours = 256 as u_int;
    } else {
        colours = tty_term_number(term, TTYC_COLORS) as u_int;
        if colours < 8 as u_int {
            colours = 2 as u_int;
        } else if colours < 16 as u_int {
            colours = 8 as u_int;
        } else {
            colours = 16 as u_int;
        }
    }
    return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, colours)
        as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_client_control_mode(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        if (*(*ft).c).flags & CLIENT_CONTROL as uint64_t != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_discarded(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return format_printf(
            b"%zu\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).c).discarded,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_flags(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return xstrdup(server_client_get_flags((*ft).c)) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_height(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() && (*(*ft).c).tty.flags & TTY_STARTED != 0 {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).c).tty.sy,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_key_table(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return xstrdup((*(*(*ft).c).keytable).name) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_last_session(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null()
        && !(*(*ft).c).last_session.is_null()
        && session_alive((*(*ft).c).last_session) != 0
    {
        return xstrdup((*(*(*ft).c).last_session).name) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_name(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return xstrdup((*(*ft).c).name) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_pid(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return format_printf(
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).c).pid as ::core::ffi::c_long,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_prefix(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !(*ft).c.is_null() {
        name = server_client_get_key_table((*ft).c);
        if strcmp((*(*(*ft).c).keytable).name, name) == 0 as ::core::ffi::c_int {
            return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_readonly(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        if (*(*ft).c).flags & CLIENT_READONLY as uint64_t != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_session(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() && !(*(*ft).c).session.is_null() {
        return xstrdup((*(*(*ft).c).session).name) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_termfeatures(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return xstrdup(tty_get_features((*(*ft).c).term_features)) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_termname(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return xstrdup((*(*ft).c).term_name) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_termtype(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        if (*(*ft).c).term_type.is_null() {
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup((*(*ft).c).term_type) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_tty(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return xstrdup((*(*ft).c).ttyname) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_uid(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut uid: uid_t = 0;
    if !(*ft).c.is_null() {
        uid = proc_get_peer_uid((*(*ft).c).peer);
        if uid != -(1 as ::core::ffi::c_int) as uid_t {
            return format_printf(
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                uid as ::core::ffi::c_long,
            ) as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_user(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut uid: uid_t = 0;
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    if !(*ft).c.is_null() {
        if !(*(*ft).c).user.is_null() {
            return xstrdup((*(*ft).c).user) as *mut ::core::ffi::c_void;
        }
        uid = proc_get_peer_uid((*(*ft).c).peer);
        if uid != -(1 as ::core::ffi::c_int) as uid_t && {
            pw = getpwuid(uid as __uid_t);
            !pw.is_null()
        } {
            (*(*ft).c).user = xstrdup((*pw).pw_name);
            return xstrdup((*(*ft).c).user) as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_utf8(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        if (*(*ft).c).flags & CLIENT_UTF8 as uint64_t != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_width(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).c).tty.sx,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_written(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return format_printf(
            b"%zu\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).c).written,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_theme(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        match (*(*ft).c).theme as ::core::ffi::c_uint {
            2 => {
                return xstrdup(b"dark\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            1 => {
                return xstrdup(b"light\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            0 => return ::core::ptr::null_mut::<::core::ffi::c_void>(),
            _ => {}
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_config_files(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut slen: size_t = 0 as size_t;
    let mut i: u_int = 0;
    let mut n: size_t = 0;
    i = 0 as u_int;
    while i < cfg_nfiles {
        n = strlen(*cfg_files.offset(i as isize)).wrapping_add(1 as size_t);
        s = xrealloc(
            s as *mut ::core::ffi::c_void,
            slen.wrapping_add(n).wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char;
        slen = slen.wrapping_add(xsnprintf(
            s.offset(slen as isize),
            n.wrapping_add(1 as size_t),
            b"%s,\0" as *const u8 as *const ::core::ffi::c_char,
            *cfg_files.offset(i as isize),
        ) as size_t);
        i = i.wrapping_add(1);
    }
    if s.is_null() {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    *s.offset(slen.wrapping_sub(1 as size_t) as isize) = '\0' as i32 as ::core::ffi::c_char;
    return s as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_cursor_flag(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_CURSOR != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_cursor_shape(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() && !(*(*ft).wp).screen.is_null() {
        match (*(*(*ft).wp).screen).cstyle as ::core::ffi::c_uint {
            1 => {
                return xstrdup(b"block\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            2 => {
                return xstrdup(b"underline\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            3 => {
                return xstrdup(b"bar\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            _ => {
                return xstrdup(b"default\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_cursor_very_visible(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() && !(*(*ft).wp).screen.is_null() {
        if (*(*(*ft).wp).screen).mode & MODE_CURSOR_VERY_VISIBLE != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_cursor_x(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).base.cx,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_cursor_y(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).base.cy,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_cursor_blinking(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() && !(*(*ft).wp).screen.is_null() {
        if (*(*(*ft).wp).screen).mode & MODE_CURSOR_BLINKING != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_history_added(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*(*ft).wp).base.grid).scroll_added,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_history_collected(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*wp).base.grid).scroll_collected,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_history_generation(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*wp).base.grid).scroll_generation,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_history_limit(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*(*ft).wp).base.grid).hlimit,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_history_size(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*(*ft).wp).base.grid).hsize,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_insert_flag(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_INSERT != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_keypad_cursor_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_KCURSOR != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_keypad_flag(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_KKEYPAD != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_all_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_MOUSE_ALL != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_any_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & ALL_MOUSE_MODES != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_button_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_MOUSE_BUTTON != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_pane(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if (*ft).m.valid != 0 {
        wp = cmd_mouse_pane(
            &raw mut (*ft).m,
            ::core::ptr::null_mut::<*mut session>(),
            ::core::ptr::null_mut::<*mut winlink>(),
        );
        if !wp.is_null() {
            return format_printf(
                b"%%%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            ) as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_sgr_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_MOUSE_SGR != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_standard_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_MOUSE_STANDARD != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_utf8_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_MOUSE_UTF8 != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_x(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    wp = cmd_mouse_pane(
        &raw mut (*ft).m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if !wp.is_null()
        && cmd_mouse_at(
            wp,
            &raw mut (*ft).m,
            &raw mut x,
            &raw mut y,
            0 as ::core::ffi::c_int,
        ) == 0 as ::core::ffi::c_int
    {
        return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, x)
            as *mut ::core::ffi::c_void;
    }
    if !(*ft).c.is_null() && (*(*ft).c).tty.flags & TTY_STARTED != 0 {
        if (*ft).m.statusat == 0 as ::core::ffi::c_int && (*ft).m.y < (*ft).m.statuslines {
            return format_printf(
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*ft).m.x,
            ) as *mut ::core::ffi::c_void;
        }
        if (*ft).m.statusat > 0 as ::core::ffi::c_int && (*ft).m.y >= (*ft).m.statusat as u_int {
            return format_printf(
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*ft).m.x,
            ) as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_mouse_y(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*ft).m.valid == 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    wp = cmd_mouse_pane(
        &raw mut (*ft).m,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
    );
    if !wp.is_null()
        && cmd_mouse_at(
            wp,
            &raw mut (*ft).m,
            &raw mut x,
            &raw mut y,
            0 as ::core::ffi::c_int,
        ) == 0 as ::core::ffi::c_int
    {
        return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, y)
            as *mut ::core::ffi::c_void;
    }
    if !(*ft).c.is_null() && (*(*ft).c).tty.flags & TTY_STARTED != 0 {
        if (*ft).m.statusat == 0 as ::core::ffi::c_int && (*ft).m.y < (*ft).m.statuslines {
            return format_printf(
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*ft).m.y,
            ) as *mut ::core::ffi::c_void;
        }
        if (*ft).m.statusat > 0 as ::core::ffi::c_int && (*ft).m.y >= (*ft).m.statusat as u_int {
            return format_printf(
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*ft).m.y.wrapping_sub((*ft).m.statusat as u_int),
            ) as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_next_session_id(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    return format_printf(
        b"$%u\0" as *const u8 as *const ::core::ffi::c_char,
        next_session_id,
    ) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_origin_flag(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_ORIGIN != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_synchronized_output_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_SYNC != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_private_modes(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    static mut table: [C2RustUnnamed_43; 14] = [
        C2RustUnnamed_43 {
            mode: MODE_KCURSOR,
            number: 1 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_ORIGIN,
            number: 6 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_WRAP,
            number: 7 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_CURSOR_BLINKING,
            number: 12 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_CURSOR,
            number: 25 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_MOUSE_STANDARD,
            number: 1000 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_MOUSE_BUTTON,
            number: 1002 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_MOUSE_ALL,
            number: 1003 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_FOCUSON,
            number: 1004 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_MOUSE_UTF8,
            number: 1005 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_MOUSE_SGR,
            number: 1006 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_BRACKETPASTE,
            number: 2004 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_SYNC,
            number: 2026 as ::core::ffi::c_int,
        },
        C2RustUnnamed_43 {
            mode: MODE_THEME_UPDATES,
            number: 2031 as ::core::ffi::c_int,
        },
    ];
    let mut mode: ::core::ffi::c_int = 0;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    if (*ft).wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    mode = (*(*ft).wp).base.mode;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_43; 14]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_43>() as usize)
    {
        if !(!mode & table[i as usize].mode != 0) {
            if !(table[i as usize].mode == MODE_CURSOR_BLINKING
                && !mode & MODE_CURSOR_BLINKING_SET != 0)
            {
                if value.is_null() {
                    xasprintf(
                        &raw mut value,
                        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                        table[i as usize].number,
                    );
                } else {
                    xasprintf(
                        &raw mut tmp,
                        b"%s,%d\0" as *const u8 as *const ::core::ffi::c_char,
                        value,
                        table[i as usize].number,
                    );
                    free(value as *mut ::core::ffi::c_void);
                    value = tmp;
                }
            }
        }
        i = i.wrapping_add(1);
    }
    if value.is_null() {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_active(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*ft).wp == (*(*(*ft).wp).window).active {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_at_left(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).xoff == 0 as ::core::ffi::c_int {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_at_right(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).xoff + (*(*ft).wp).sx as ::core::ffi::c_int
            == (*(*(*ft).wp).window).sx as ::core::ffi::c_int
        {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_bottom(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).yoff + (*wp).sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_dead(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if (*wp).fd == -(1 as ::core::ffi::c_int) && (*wp).flags & PANE_STATUSREADY != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_dead_signal(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !wp.is_null() {
        if (*wp).flags & PANE_STATUSREADY != 0
            && (((*wp).status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                as ::core::ffi::c_schar as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int
                > 0 as ::core::ffi::c_int
        {
            name = sig2name((*wp).status & 0x7f as ::core::ffi::c_int);
            return format_printf(b"%s\0" as *const u8 as *const ::core::ffi::c_char, name)
                as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_dead_status(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if (*wp).flags & PANE_STATUSREADY != 0
            && (*wp).status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        {
            return format_printf(
                b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                ((*wp).status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int,
            ) as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_dead_time(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if (*wp).flags & PANE_STATUSDRAWN != 0 {
            return &raw mut (*wp).dead_time as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_last_output_time(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    static mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if !wp.is_null() && (*wp).last_output_time != 0 as time_t {
        tv.tv_sec = (*wp).last_output_time as __time_t;
        tv.tv_usec = 0 as __suseconds_t;
        return &raw mut tv as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_output_generation(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut value: ::core::ffi::c_ulonglong = 0;
    if !(*ft).wp.is_null() {
        value = (*(*ft).wp).output_generation as ::core::ffi::c_ulonglong;
        return format_printf(b"%llu\0" as *const u8 as *const ::core::ffi::c_char, value)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_last_prompt_time(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    static mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if !wp.is_null() && (*wp).last_prompt_time != 0 as time_t {
        tv.tv_sec = (*wp).last_prompt_time as __time_t;
        tv.tv_usec = 0 as __suseconds_t;
        return &raw mut tv as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_command_start_time(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    static mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if !wp.is_null() && (*wp).cmd_start_time != 0 as time_t {
        tv.tv_sec = (*wp).cmd_start_time as __time_t;
        tv.tv_usec = 0 as __suseconds_t;
        return &raw mut tv as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_command_end_time(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    static mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if !wp.is_null() && (*wp).cmd_end_time != 0 as time_t {
        tv.tv_sec = (*wp).cmd_end_time as __time_t;
        tv.tv_usec = 0 as __suseconds_t;
        return &raw mut tv as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_command_running(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            ((*wp).flags & PANE_CMDRUNNING != 0) as ::core::ffi::c_int,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_command_duration(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut end: time_t = 0;
    if wp.is_null() || (*wp).cmd_start_time == 0 as time_t {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*wp).flags & PANE_CMDRUNNING != 0 {
        end = time(::core::ptr::null_mut::<time_t>());
    } else {
        end = (*wp).cmd_end_time;
    }
    if end < (*wp).cmd_start_time {
        end = (*wp).cmd_start_time;
    }
    return format_printf(
        b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
        (end - (*wp).cmd_start_time) as ::core::ffi::c_longlong,
    ) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_command_status(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() && (*wp).cmd_status != -(1 as ::core::ffi::c_int) {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).cmd_status,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_format(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if (*ft).type_0 as ::core::ffi::c_uint
        == FORMAT_TYPE_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_height(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).sy,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_id(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).id,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_index(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut idx: u_int = 0;
    if !(*ft).wp.is_null() && window_pane_index((*ft).wp, &raw mut idx) == 0 as ::core::ffi::c_int {
        return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, idx)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_input_off(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).flags & PANE_INPUTOFF != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_unseen_changes(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).flags & PANE_UNSEENCHANGES != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_key_mode(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() && !(*(*ft).wp).screen.is_null() {
        match (*(*(*ft).wp).screen).mode & EXTENDED_KEY_MODES {
            MODE_KEYS_EXTENDED => {
                return xstrdup(b"Ext 1\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            MODE_KEYS_EXTENDED_2 => {
                return xstrdup(b"Ext 2\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            _ => {
                return xstrdup(b"VT10x\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_last(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*ft).wp == (*(*(*ft).wp).window).last_panes.tqh_first {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_left(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).xoff,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_marked(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if server_check_marked() != 0 && marked_pane.wp == (*ft).wp {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_marked_set(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if server_check_marked() != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_mode(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    if !(*ft).wp.is_null() {
        wme = (*(*ft).wp).modes.tqh_first;
        if !wme.is_null() {
            return xstrdup((*(*wme).mode).name) as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_path(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.path.is_null() {
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup((*(*ft).wp).base.path) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_pid(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() && (*(*ft).wp).fd != -(1 as ::core::ffi::c_int) {
        return format_printf(
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).pid as ::core::ffi::c_long,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_pipe(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).pipe_fd != -(1 as ::core::ffi::c_int) {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_pipe_pid(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(*ft).wp.is_null() && (*(*ft).wp).pipe_fd != -(1 as ::core::ffi::c_int) {
        xasprintf(
            &raw mut value,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).pipe_pid as ::core::ffi::c_long,
        );
    }
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_pb_progress(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(*ft).wp.is_null() {
        xasprintf(
            &raw mut value,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).base.progress_bar.progress,
        );
    }
    return value as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_pb_state(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        match (*(*ft).wp).base.progress_bar.state as ::core::ffi::c_uint {
            0 => {
                return xstrdup(b"hidden\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            1 => {
                return xstrdup(b"normal\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            2 => {
                return xstrdup(b"error\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            3 => {
                return xstrdup(b"indeterminate\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            4 => {
                return xstrdup(b"paused\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            _ => {}
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_right(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).xoff + (*wp).sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_search_string(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).searchstr.is_null() {
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup((*(*ft).wp).searchstr) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_synchronized(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if options_get_number(
            (*(*ft).wp).options,
            b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
        {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_title(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return xstrdup((*(*ft).wp).base.title) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_top(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).yoff,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_tty(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return xstrdup(&raw mut (*(*ft).wp).tty as *mut ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_unzoomed_height(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut root: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut status: ::core::ffi::c_int = 0;
    let mut floating: ::core::ffi::c_int = 0;
    let mut sy: u_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    w = (*wp).window as *mut window;
    lc = (*wp).saved_layout_cell;
    if lc.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
    }
    if lc.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    sy = (*lc).g.sy;
    floating = (*lc).flags & LAYOUT_CELL_FLOATING;
    root = (*w).saved_layout_root;
    if root.is_null() {
        root = (*w).layout_root;
    }
    if lc == (*wp).saved_layout_cell && floating == 0 {
        status = window_get_pane_status(w);
    } else {
        status = window_pane_get_pane_status(wp);
    }
    if floating == 0
        && !root.is_null()
        && layout_add_horizontal_border(root, lc, status) != 0
        && sy > 1 as u_int
    {
        sy = sy.wrapping_sub(1);
    }
    return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, sy)
        as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_unzoomed_width(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut saved: ::core::ffi::c_int = 0;
    let mut sb_w: ::core::ffi::c_int = 0;
    let mut sb_pad: ::core::ffi::c_int = 0;
    let mut sx: u_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    lc = (*wp).saved_layout_cell;
    saved = (lc != NULL_0 as *mut layout_cell) as ::core::ffi::c_int;
    if lc.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
    }
    if lc.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    sx = (*lc).g.sx;
    if saved != 0 && (*wp).base.saved_grid.is_null() && (*(*wp).window).sb == PANE_SCROLLBARS_ALWAYS
        || saved == 0 && window_pane_scrollbar_reserve(wp) != 0
    {
        sb_w = (*wp).scrollbar_style.width;
        sb_pad = (*wp).scrollbar_style.pad;
        if sb_w < 1 as ::core::ffi::c_int {
            sb_w = 1 as ::core::ffi::c_int;
        }
        if sb_pad < 0 as ::core::ffi::c_int {
            sb_pad = 0 as ::core::ffi::c_int;
        }
        if sx as ::core::ffi::c_int - sb_w - sb_pad < PANE_MINIMUM {
            sx = PANE_MINIMUM as u_int;
        } else {
            sx = sx.wrapping_sub((sb_w + sb_pad) as u_int);
        }
    }
    return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, sx)
        as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_pane_width(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).sx,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_x(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).xoff,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_y(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).yoff,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_z(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut idx: u_int = 0;
    if !(*ft).wp.is_null() && window_pane_zindex((*ft).wp, &raw mut idx) == 0 as ::core::ffi::c_int
    {
        return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, idx)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_pane_zoomed_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wp: *mut window_pane = (*ft).wp;
    if !wp.is_null() {
        if (*wp).flags & PANE_ZOOMED != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_scroll_region_lower(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).base.rlower,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_scroll_region_upper(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wp).base.rupper,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_server_sessions(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut n: u_int = 0 as u_int;
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        n = n.wrapping_add(1);
        s = sessions_RB_NEXT(s);
    }
    return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, n)
        as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_active(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if (*ft).s.is_null() || (*ft).c.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if (*(*ft).c).session == (*ft).s {
        return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_activity_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !(*ft).s.is_null() {
        wl = winlinks_RB_MINMAX(&raw mut (*(*ft).s).windows, RB_NEGINF);
        if !wl.is_null() {
            if (*(*ft).wl).flags & WINLINK_ACTIVITY != 0 {
                return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_bell_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !(*ft).s.is_null() {
        wl = winlinks_RB_MINMAX(&raw mut (*(*ft).s).windows, RB_NEGINF);
        if !wl.is_null() {
            if (*wl).flags & WINLINK_BELL != 0 {
                return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_silence_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !(*ft).s.is_null() {
        wl = winlinks_RB_MINMAX(&raw mut (*(*ft).s).windows, RB_NEGINF);
        if !wl.is_null() {
            if (*(*ft).wl).flags & WINLINK_SILENCE != 0 {
                return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                    as *mut ::core::ffi::c_void;
            }
            return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_attached(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).s).attached,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_format(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if (*ft).type_0 as ::core::ffi::c_uint
        == FORMAT_TYPE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_session_group(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    if !(*ft).s.is_null() && {
        sg = session_group_contains((*ft).s);
        !sg.is_null()
    } {
        return xstrdup((*sg).name) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_group_attached(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    if !(*ft).s.is_null() && {
        sg = session_group_contains((*ft).s);
        !sg.is_null()
    } {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            session_group_attached_count(sg),
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_group_many_attached(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    if !(*ft).s.is_null() && {
        sg = session_group_contains((*ft).s);
        !sg.is_null()
    } {
        if session_group_attached_count(sg) > 1 as u_int {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_group_size(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    if !(*ft).s.is_null() && {
        sg = session_group_contains((*ft).s);
        !sg.is_null()
    } {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            session_group_count(sg),
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_grouped(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        if !session_group_contains((*ft).s).is_null() {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_id(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return format_printf(
            b"$%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).s).id,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_many_attached(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        if (*(*ft).s).attached > 1 as u_int {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_marked(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        if server_check_marked() != 0 && marked_pane.s == (*ft).s {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_name(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return xstrdup((*(*ft).s).name) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_path(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return xstrdup((*(*ft).s).cwd) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_windows(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            winlink_count(&raw mut (*(*ft).s).windows),
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_socket_path(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    return xstrdup(socket_path) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_version(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    return xstrdup(getversion()) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_sixel_support(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_active_window_index(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*(*ft).s).curw).idx,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_last_window_index(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !(*ft).s.is_null() {
        wl = winlinks_RB_MINMAX(&raw mut (*(*ft).s).windows, RB_INF);
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wl).idx,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_active(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        if (*ft).wl == (*(*(*ft).wl).session).curw {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_activity_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        if (*(*ft).wl).flags & WINLINK_ACTIVITY != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_bell_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        if (*(*ft).wl).flags & WINLINK_BELL != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_bigger(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if !(*ft).c.is_null() {
        if tty_window_offset(
            &raw mut (*(*ft).c).tty,
            &raw mut ox,
            &raw mut oy,
            &raw mut sx,
            &raw mut sy,
        ) != 0
        {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_cell_height(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).w).ypixel,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_cell_width(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).w).xpixel,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_end_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        if (*ft).wl == winlinks_RB_MINMAX(&raw mut (*(*(*ft).wl).session).windows, RB_INF) {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_flags(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        return xstrdup(window_printable_flags((*ft).wl, 1 as ::core::ffi::c_int))
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_format(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if (*ft).type_0 as ::core::ffi::c_uint
        == FORMAT_TYPE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_height(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).w).sy,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_manual_height(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut w: *mut window = (*ft).w;
    if w.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if options_get_number(
        (*w).options,
        b"window-size\0" as *const u8 as *const ::core::ffi::c_char,
    ) != WINDOW_SIZE_MANUAL as ::core::ffi::c_longlong
    {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return format_printf(
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).manual_sy,
    ) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_id(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        return format_printf(
            b"@%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).w).id,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_index(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        return format_printf(
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).wl).idx,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_last_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        if (*ft).wl == (*(*(*ft).wl).session).lastw.tqh_first {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_linked(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !(*ft).wl.is_null() {
        s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
        while !s.is_null() {
            wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
            while !wl.is_null() {
                if (*wl).window == (*(*ft).wl).window {
                    if found != 0 {
                        return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                            as *mut ::core::ffi::c_void;
                    }
                    found = 1 as ::core::ffi::c_int;
                }
                wl = winlinks_RB_NEXT(wl);
            }
            s = sessions_RB_NEXT(s);
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_linked_sessions(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut n: u_int = 0 as u_int;
    if (*ft).wl.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    w = (*(*ft).wl).window;
    sg = session_groups_RB_MINMAX(&raw mut session_groups, RB_NEGINF);
    while !sg.is_null() {
        s = (*sg).sessions.tqh_first;
        if !winlink_find_by_window(&raw mut (*s).windows, w).is_null() {
            n = n.wrapping_add(1);
        }
        sg = session_groups_RB_NEXT(sg);
    }
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        if session_group_contains(s).is_null() {
            if !winlink_find_by_window(&raw mut (*s).windows, w).is_null() {
                n = n.wrapping_add(1);
            }
        }
        s = sessions_RB_NEXT(s);
    }
    return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, n)
        as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_marked_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        if server_check_marked() != 0 && marked_pane.wl == (*ft).wl {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_modal_pane(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() && !(*(*ft).w).modal.is_null() {
        return format_printf(
            b"%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*(*ft).w).modal).id,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_name(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        return format_printf(
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).w).name,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_offset_x(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if !(*ft).c.is_null() {
        if tty_window_offset(
            &raw mut (*(*ft).c).tty,
            &raw mut ox,
            &raw mut oy,
            &raw mut sx,
            &raw mut sy,
        ) != 0
        {
            return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, ox)
                as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_offset_y(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if !(*ft).c.is_null() {
        if tty_window_offset(
            &raw mut (*(*ft).c).tty,
            &raw mut ox,
            &raw mut oy,
            &raw mut sx,
            &raw mut sy,
        ) != 0
        {
            return format_printf(b"%u\0" as *const u8 as *const ::core::ffi::c_char, oy)
                as *mut ::core::ffi::c_void;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_panes(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            window_count_panes((*ft).w, 1 as ::core::ffi::c_int),
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_raw_flags(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        return xstrdup(window_printable_flags((*ft).wl, 0 as ::core::ffi::c_int))
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_silence_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        if (*(*ft).wl).flags & WINLINK_SILENCE != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_start_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).wl.is_null() {
        if (*ft).wl == winlinks_RB_MINMAX(&raw mut (*(*(*ft).wl).session).windows, RB_NEGINF) {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_width(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        return format_printf(
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*ft).w).sx,
        ) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_window_manual_width(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    let mut w: *mut window = (*ft).w;
    if w.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    if options_get_number(
        (*w).options,
        b"window-size\0" as *const u8 as *const ::core::ffi::c_char,
    ) != WINDOW_SIZE_MANUAL as ::core::ffi::c_longlong
    {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return format_printf(
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).manual_sx,
    ) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_zoomed_flag(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        if (*(*ft).w).flags & WINDOW_ZOOMED != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_wrap_flag(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    if !(*ft).wp.is_null() {
        if (*(*ft).wp).base.mode & MODE_WRAP != 0 {
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char)
                as *mut ::core::ffi::c_void;
        }
        return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char)
            as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_buffer_created(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    static mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if !(*ft).pb.is_null() {
        tv.tv_usec = 0 as __suseconds_t;
        tv.tv_sec = tv.tv_usec as __time_t;
        tv.tv_sec = paste_buffer_created((*ft).pb) as __time_t;
        return &raw mut tv as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_activity(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return &raw mut (*(*ft).c).activity_time as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_client_created(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).c.is_null() {
        return &raw mut (*(*ft).c).creation_time as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_activity(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return &raw mut (*(*ft).s).activity_time as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_created(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return &raw mut (*(*ft).s).creation_time as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_session_last_attached(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).s.is_null() {
        return &raw mut (*(*ft).s).last_attached_time as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_start_time(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    return &raw mut start_time as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_window_activity(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    if !(*ft).w.is_null() {
        return &raw mut (*(*ft).w).activity_time as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
unsafe extern "C" fn format_cb_buffer_mode_format(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    return xstrdup(window_buffer_mode.default_format) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_client_mode_format(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    return xstrdup(window_client_mode.default_format) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_tree_mode_format(
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_void {
    return xstrdup(window_tree_mode.default_format) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_uid(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    return format_printf(
        b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
        getuid() as ::core::ffi::c_long,
    ) as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn format_cb_user(mut ft: *mut format_tree) -> *mut ::core::ffi::c_void {
    static mut cached: *mut ::core::ffi::c_char =
        ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char;
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    if cached.is_null() && {
        pw = getpwuid(getuid());
        !pw.is_null()
    } {
        cached = xstrdup((*pw).pw_name);
    }
    if !cached.is_null() {
        return xstrdup(cached) as *mut ::core::ffi::c_void;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_void>();
}
static mut format_table: [format_table_entry; 214] = unsafe {
    [
        format_table_entry {
            key: b"active_window_index\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_active_window_index
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"alternate_on\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_alternate_on
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"alternate_saved_x\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_alternate_saved_x
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"alternate_saved_y\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_alternate_saved_y
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"bracket_paste_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_bracket_paste_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"buffer_created\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_buffer_created
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"buffer_full\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_buffer_full
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"buffer_mode_format\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_buffer_mode_format
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"buffer_name\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_buffer_name
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"buffer_sample\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_buffer_sample
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"buffer_size\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_buffer_size
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_activity\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_client_activity
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_cell_height\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_cell_height
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_cell_width\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_cell_width
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_colours\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_colours
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_control_mode\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_control_mode
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_created\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_client_created
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_discarded\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_discarded
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_flags\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_flags
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_height\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_height
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_key_table\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_key_table
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_last_session\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_last_session
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_mode_format\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_mode_format
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_name\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_name
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_pid\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_pid
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_prefix\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_prefix
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_readonly\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_readonly
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_session\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_session
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_termfeatures\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_termfeatures
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_termname\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_termname
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_termtype\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_termtype
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_theme\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_theme
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_tty\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_tty
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_uid\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_uid
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_user\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_user
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_utf8\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_utf8
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_width\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_width
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"client_written\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_client_written
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"config_files\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_config_files
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"cursor_blinking\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_cursor_blinking
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"cursor_character\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_cursor_character
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"cursor_colour\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_cursor_colour
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"cursor_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_cursor_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"cursor_shape\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_cursor_shape
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"cursor_very_visible\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_cursor_very_visible
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"cursor_x\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_cursor_x
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"cursor_y\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_cursor_y
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"history_added\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_history_added
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"history_all_bytes\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_history_all_bytes
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"history_bytes\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_history_bytes
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"history_collected\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_history_collected
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"history_generation\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_history_generation
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"history_limit\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_history_limit
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"history_size\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_history_size
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"host\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_host
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"host_short\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_host_short
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"insert_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_insert_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"keypad_cursor_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_keypad_cursor_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"keypad_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_keypad_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"last_window_index\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_last_window_index
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_all_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_all_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_any_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_any_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_button_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_button_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_hyperlink\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_hyperlink
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_line\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_line
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_pane\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_pane
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_sgr_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_sgr_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_standard_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_standard_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_status_line\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_status_line
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_status_range\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_status_range
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_utf8_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_utf8_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_word\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_word
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_x\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_x
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"mouse_y\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_mouse_y
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"next_session_id\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_next_session_id
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"origin_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_origin_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_active\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_active
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_at_bottom\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_at_bottom
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_at_left\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_at_left
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_at_right\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_at_right
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_at_top\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_at_top
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_bg\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_bg
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_bottom\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_bottom
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_command_duration\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_command_duration
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_command_end_time\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_pane_command_end_time
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_command_running\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_command_running
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_command_start_time\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_pane_command_start_time
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_command_status\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_command_status
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_current_command\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_current_command
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_current_path\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_current_path
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_dead\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_dead
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_dead_signal\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_dead_signal
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_dead_status\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_dead_status
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_dead_time\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_pane_dead_time
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_fg\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_fg
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_flags\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_flags
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_floating_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_floating_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_format\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_format
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_height\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_height
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_id\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_id
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_in_mode\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_in_mode
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_index\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_index
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_input_off\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_input_off
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_key_mode\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_key_mode
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_last\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_last
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_last_output_time\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_pane_last_output_time
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_last_prompt_time\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_pane_last_prompt_time
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_left\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_left
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_marked\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_marked
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_marked_set\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_marked_set
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_modal_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_modal_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_mode\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_mode
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_output_generation\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_output_generation
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_path\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_path
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_pb_progress\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_pb_progress
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_pb_state\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_pb_state
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_pid\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_pid
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_pipe\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_pipe
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_pipe_pid\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_pipe_pid
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_private_modes\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_private_modes
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_right\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_right
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_search_string\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_search_string
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_start_command\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_start_command
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_start_command_list\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_start_command_list
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_start_path\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_start_path
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_synchronized\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_synchronized
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_tabs\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_tabs
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_title\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_title
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_top\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_top
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_tty\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_tty
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_unseen_changes\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_unseen_changes
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_unzoomed_height\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_unzoomed_height
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_unzoomed_width\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_unzoomed_width
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_width\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_width
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_x\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_x
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_y\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_y
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_z\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_z
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pane_zoomed_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pane_zoomed_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"pid\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_pid as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"scroll_region_lower\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_scroll_region_lower
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"scroll_region_upper\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_scroll_region_upper
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"server_sessions\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_server_sessions
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_active\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_active
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_activity\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_session_activity
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_activity_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_activity_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_alert\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_alert
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_alerts\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_alerts
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_attached\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_attached
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_attached_list\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_attached_list
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_bell_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_bell_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_created\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_session_created
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_format\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_format
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_group\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_group
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_group_attached\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_group_attached
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_group_attached_list\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_group_attached_list
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_group_list\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_group_list
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_group_many_attached\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_group_many_attached
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_group_size\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_group_size
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_grouped\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_grouped
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_id\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_id
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_last_attached\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_session_last_attached
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_many_attached\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_many_attached
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_marked\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_marked
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_name\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_name
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_path\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_path
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_silence_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_silence_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_stack\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_stack
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"session_windows\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_session_windows
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"sixel_support\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_sixel_support
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"socket_path\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_socket_path
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"start_time\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_start_time
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"synchronized_output_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_synchronized_output_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"tree_mode_format\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_tree_mode_format
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"uid\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_uid as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"user\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_user
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"version\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_version
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_active\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_active
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_active_clients\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_active_clients
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_active_clients_list\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_active_clients_list
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_active_sessions\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_active_sessions
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_active_sessions_list\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_active_sessions_list
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_activity\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_TIME,
            cb: Some(
                format_cb_window_activity
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_activity_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_activity_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_bell_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_bell_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_bigger\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_bigger
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_cell_height\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_cell_height
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_cell_width\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_cell_width
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_end_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_end_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_flags\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_flags
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_format\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_format
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_height\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_height
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_id\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_id
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_index
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_last_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_layout\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_layout
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_linked\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_linked
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_linked_sessions\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_linked_sessions
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_linked_sessions_list\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_linked_sessions_list
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_manual_height\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_manual_height
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_manual_width\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_manual_width
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_marked_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_marked_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_modal_pane\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_modal_pane
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_name\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_name
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_offset_x\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_offset_x
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_offset_y\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_offset_y
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_panes\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_panes
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_raw_flags\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_raw_flags
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_silence_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_silence_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_stack_index\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_stack_index
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_start_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_start_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_visible_layout\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_visible_layout
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_width\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_width
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"window_zoomed_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_window_zoomed_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
        format_table_entry {
            key: b"wrap_flag\0" as *const u8 as *const ::core::ffi::c_char,
            type_0: FORMAT_TABLE_STRING,
            cb: Some(
                format_cb_wrap_flag
                    as unsafe extern "C" fn(*mut format_tree) -> *mut ::core::ffi::c_void,
            ),
        },
    ]
};
unsafe extern "C" fn format_table_compare(
    mut key0: *const ::core::ffi::c_void,
    mut entry0: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut key: *const ::core::ffi::c_char = key0 as *const ::core::ffi::c_char;
    let mut entry: *const format_table_entry = entry0 as *const format_table_entry;
    return strcmp(key, (*entry).key);
}
unsafe extern "C" fn format_table_get(
    mut key: *const ::core::ffi::c_char,
) -> *const format_table_entry {
    return bsearch(
        key as *const ::core::ffi::c_void,
        &raw const format_table as *const format_table_entry as *const ::core::ffi::c_void,
        (::core::mem::size_of::<[format_table_entry; 214]>() as size_t)
            .wrapping_div(::core::mem::size_of::<format_table_entry>() as size_t),
        ::core::mem::size_of::<format_table_entry>() as size_t,
        Some(
            format_table_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    ) as *const format_table_entry;
}
#[no_mangle]
pub unsafe extern "C" fn format_merge(mut ft: *mut format_tree, mut from: *mut format_tree) {
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    fe = format_entry_tree_RB_MINMAX(&raw mut (*from).tree, RB_NEGINF);
    while !fe.is_null() {
        if !(*fe).value.is_null() {
            format_add(
                ft,
                (*fe).key,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*fe).value,
            );
        }
        fe = format_entry_tree_RB_NEXT(fe);
    }
}
#[no_mangle]
pub unsafe extern "C" fn format_get_pane(mut ft: *mut format_tree) -> *mut window_pane {
    return (*ft).wp;
}
unsafe extern "C" fn format_create_add_item(mut ft: *mut format_tree, mut item: *mut cmdq_item) {
    let mut event: *mut key_event = cmdq_get_event(item);
    let mut m: *mut mouse_event = &raw mut (*event).m;
    cmdq_merge_formats(item, ft);
    memcpy(
        &raw mut (*ft).m as *mut ::core::ffi::c_void,
        m as *const ::core::ffi::c_void,
        ::core::mem::size_of::<mouse_event>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn format_create(
    mut c: *mut client,
    mut item: *mut cmdq_item,
    mut tag: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> *mut format_tree {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    ft = xcalloc(1 as size_t, ::core::mem::size_of::<format_tree>() as size_t) as *mut format_tree;
    (*ft).tree.rbh_root = ::core::ptr::null_mut::<format_entry>();
    if !c.is_null() {
        (*ft).client = c;
        (*(*ft).client).references += 1;
    }
    (*ft).item = item;
    (*ft).tag = tag as u_int;
    (*ft).flags = flags;
    if !item.is_null() {
        format_create_add_item(ft, item);
    }
    return ft;
}
#[no_mangle]
pub unsafe extern "C" fn format_free(mut ft: *mut format_tree) {
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut fe1: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    fe = format_entry_tree_RB_MINMAX(&raw mut (*ft).tree, RB_NEGINF);
    while !fe.is_null() && {
        fe1 = format_entry_tree_RB_NEXT(fe);
        1 as ::core::ffi::c_int != 0
    } {
        format_entry_tree_RB_REMOVE(&raw mut (*ft).tree, fe);
        free((*fe).value as *mut ::core::ffi::c_void);
        free((*fe).key as *mut ::core::ffi::c_void);
        free(fe as *mut ::core::ffi::c_void);
        fe = fe1;
    }
    if !(*ft).client.is_null() {
        server_client_unref((*ft).client);
    }
    free(ft as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn format_log_debug_cb(
    mut key: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut prefix: *const ::core::ffi::c_char = arg as *const ::core::ffi::c_char;
    log_debug(
        b"%s: %s=%s\0" as *const u8 as *const ::core::ffi::c_char,
        prefix,
        key,
        value,
    );
}
#[no_mangle]
pub unsafe extern "C" fn format_log_debug(
    mut ft: *mut format_tree,
    mut prefix: *const ::core::ffi::c_char,
) {
    if log_get_level() == 0 as ::core::ffi::c_int {
        return;
    }
    format_each(
        ft,
        Some(
            format_log_debug_cb
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_char,
                    *const ::core::ffi::c_char,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        prefix as *mut ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn format_each(
    mut ft: *mut format_tree,
    mut cb: Option<
        unsafe extern "C" fn(
            *const ::core::ffi::c_char,
            *const ::core::ffi::c_char,
            *mut ::core::ffi::c_void,
        ) -> (),
    >,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut fte: *const format_table_entry = ::core::ptr::null::<format_table_entry>();
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut i: u_int = 0;
    let mut s: [::core::ffi::c_char; 64] = [0; 64];
    let mut value: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut tv: *mut timeval = ::core::ptr::null_mut::<timeval>();
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[format_table_entry; 214]>() as usize)
            .wrapping_div(::core::mem::size_of::<format_table_entry>() as usize)
    {
        fte = (&raw const format_table as *const format_table_entry).offset(i as isize)
            as *const format_table_entry;
        value = (*fte).cb.expect("non-null function pointer")(ft);
        if !value.is_null() {
            if (*fte).type_0 as ::core::ffi::c_uint
                == FORMAT_TABLE_TIME as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                tv = value as *mut timeval;
                xsnprintf(
                    &raw mut s as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                    b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                    (*tv).tv_sec as ::core::ffi::c_longlong,
                );
                cb.expect("non-null function pointer")(
                    (*fte).key,
                    &raw mut s as *mut ::core::ffi::c_char,
                    arg,
                );
            } else {
                cb.expect("non-null function pointer")(
                    (*fte).key,
                    value as *const ::core::ffi::c_char,
                    arg,
                );
                free(value);
            }
        }
        i = i.wrapping_add(1);
    }
    fe = format_entry_tree_RB_MINMAX(&raw mut (*ft).tree, RB_NEGINF);
    while !fe.is_null() {
        if (*fe).time != 0 as time_t {
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                (*fe).time as ::core::ffi::c_longlong,
            );
            cb.expect("non-null function pointer")(
                (*fe).key,
                &raw mut s as *mut ::core::ffi::c_char,
                arg,
            );
        } else {
            if (*fe).value.is_null() && (*fe).cb.is_some() {
                (*fe).value =
                    (*fe).cb.expect("non-null function pointer")(ft) as *mut ::core::ffi::c_char;
                if (*fe).value.is_null() {
                    (*fe).value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                }
            }
            cb.expect("non-null function pointer")((*fe).key, (*fe).value, arg);
        }
        fe = format_entry_tree_RB_NEXT(fe);
    }
}
#[no_mangle]
pub unsafe extern "C" fn format_add(
    mut ft: *mut format_tree,
    mut key: *const ::core::ffi::c_char,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut fe_now: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut ap: ::core::ffi::VaList;
    fe = xmalloc(::core::mem::size_of::<format_entry>() as size_t) as *mut format_entry;
    (*fe).key = xstrdup(key);
    fe_now = format_entry_tree_RB_INSERT(&raw mut (*ft).tree, fe);
    if !fe_now.is_null() {
        free((*fe).key as *mut ::core::ffi::c_void);
        free(fe as *mut ::core::ffi::c_void);
        free((*fe_now).value as *mut ::core::ffi::c_void);
        fe = fe_now;
    }
    (*fe).cb = None;
    (*fe).time = 0 as time_t;
    ap = args.clone();
    xvasprintf(&raw mut (*fe).value, fmt, ap);
}
#[no_mangle]
pub unsafe extern "C" fn format_add_tv(
    mut ft: *mut format_tree,
    mut key: *const ::core::ffi::c_char,
    mut tv: *mut timeval,
) {
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut fe_now: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    fe = xmalloc(::core::mem::size_of::<format_entry>() as size_t) as *mut format_entry;
    (*fe).key = xstrdup(key);
    fe_now = format_entry_tree_RB_INSERT(&raw mut (*ft).tree, fe);
    if !fe_now.is_null() {
        free((*fe).key as *mut ::core::ffi::c_void);
        free(fe as *mut ::core::ffi::c_void);
        free((*fe_now).value as *mut ::core::ffi::c_void);
        fe = fe_now;
    }
    (*fe).cb = None;
    (*fe).time = (*tv).tv_sec as time_t;
    (*fe).value = ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn format_add_cb(
    mut ft: *mut format_tree,
    mut key: *const ::core::ffi::c_char,
    mut cb: format_cb,
) {
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut fe_now: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    fe = xmalloc(::core::mem::size_of::<format_entry>() as size_t) as *mut format_entry;
    (*fe).key = xstrdup(key);
    fe_now = format_entry_tree_RB_INSERT(&raw mut (*ft).tree, fe);
    if !fe_now.is_null() {
        free((*fe).key as *mut ::core::ffi::c_void);
        free(fe as *mut ::core::ffi::c_void);
        free((*fe_now).value as *mut ::core::ffi::c_void);
        fe = fe_now;
    }
    (*fe).cb = cb;
    (*fe).time = 0 as time_t;
    (*fe).value = ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn format_quote_shell(
    mut s: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut at: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    out = xmalloc(
        strlen(s)
            .wrapping_mul(2 as size_t)
            .wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char;
    at = out;
    cp = s;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if !strchr(
            b"|&;<>(){}$`\\\"'*?[# =%\n\t\0" as *const u8 as *const ::core::ffi::c_char,
            *cp as ::core::ffi::c_int,
        )
        .is_null()
        {
            let fresh20 = at;
            at = at.offset(1);
            *fresh20 = '\\' as i32 as ::core::ffi::c_char;
        }
        let fresh21 = at;
        at = at.offset(1);
        *fresh21 = *cp;
        cp = cp.offset(1);
    }
    *at = '\0' as i32 as ::core::ffi::c_char;
    return out;
}
unsafe extern "C" fn format_quote_shell_single(
    mut s: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut at: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    out = xmalloc(
        strlen(s)
            .wrapping_mul(4 as size_t)
            .wrapping_add(3 as size_t),
    ) as *mut ::core::ffi::c_char;
    at = out;
    let fresh0 = at;
    at = at.offset(1);
    *fresh0 = '\'' as i32 as ::core::ffi::c_char;
    cp = s;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if *cp as ::core::ffi::c_int == '\'' as i32 {
            let fresh1 = at;
            at = at.offset(1);
            *fresh1 = '\'' as i32 as ::core::ffi::c_char;
            let fresh2 = at;
            at = at.offset(1);
            *fresh2 = '\\' as i32 as ::core::ffi::c_char;
            let fresh3 = at;
            at = at.offset(1);
            *fresh3 = '\'' as i32 as ::core::ffi::c_char;
            let fresh4 = at;
            at = at.offset(1);
            *fresh4 = '\'' as i32 as ::core::ffi::c_char;
        } else {
            let fresh5 = at;
            at = at.offset(1);
            *fresh5 = *cp;
        }
        cp = cp.offset(1);
    }
    let fresh6 = at;
    at = at.offset(1);
    *fresh6 = '\'' as i32 as ::core::ffi::c_char;
    *at = '\0' as i32 as ::core::ffi::c_char;
    return out;
}
unsafe extern "C" fn format_quote_style(
    mut s: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut at: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    out = xmalloc(
        strlen(s)
            .wrapping_mul(2 as size_t)
            .wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char;
    at = out;
    cp = s;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if *cp as ::core::ffi::c_int == '#' as i32 {
            let fresh18 = at;
            at = at.offset(1);
            *fresh18 = '#' as i32 as ::core::ffi::c_char;
        }
        let fresh19 = at;
        at = at.offset(1);
        *fresh19 = *cp;
        cp = cp.offset(1);
    }
    *at = '\0' as i32 as ::core::ffi::c_char;
    return out;
}
#[no_mangle]
pub unsafe extern "C" fn format_pretty_time(
    mut t: time_t,
    mut seconds: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut now_tm: tm = tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut tm: tm = tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
    };
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
                b"%H:%M:%S\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut tm,
            );
        } else {
            strftime(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t,
                b"%H:%M\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut tm,
            );
        }
        return xstrdup(&raw mut s as *mut ::core::ffi::c_char);
    }
    if tm.tm_year == now_tm.tm_year && tm.tm_mon == now_tm.tm_mon
        || age
            < (28 as ::core::ffi::c_int * 24 as ::core::ffi::c_int * 3600 as ::core::ffi::c_int)
                as time_t
    {
        strftime(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t,
            b"%a%d\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut tm,
        );
        return xstrdup(&raw mut s as *mut ::core::ffi::c_char);
    }
    if tm.tm_year == now_tm.tm_year && tm.tm_mon < now_tm.tm_mon
        || tm.tm_year == now_tm.tm_year - 1 as ::core::ffi::c_int && tm.tm_mon > now_tm.tm_mon
    {
        strftime(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t,
            b"%d%b\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut tm,
        );
        return xstrdup(&raw mut s as *mut ::core::ffi::c_char);
    }
    strftime(
        &raw mut s as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t,
        b"%h%y\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut tm,
    );
    return xstrdup(&raw mut s as *mut ::core::ffi::c_char);
}
unsafe extern "C" fn format_relative_time(mut t: time_t) -> *mut ::core::ffi::c_char {
    let mut now: time_t = 0;
    let mut age: time_t = 0;
    let mut d: u_int = 0;
    let mut h: u_int = 0;
    let mut m: u_int = 0;
    let mut s: u_int = 0;
    let mut out: [::core::ffi::c_char; 32] = [0; 32];
    time(&raw mut now);
    if t > now {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if t == now {
        return xstrdup(b"0s\0" as *const u8 as *const ::core::ffi::c_char);
    }
    age = now - t;
    d = (age / 86400 as time_t) as u_int;
    h = (age % 86400 as time_t / 3600 as time_t) as u_int;
    m = (age % 3600 as time_t / 60 as time_t) as u_int;
    s = (age % 60 as time_t) as u_int;
    if d != 0 as u_int {
        if h != 0 as u_int {
            xsnprintf(
                &raw mut out as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%ud%uh\0" as *const u8 as *const ::core::ffi::c_char,
                d,
                h,
            );
        } else {
            xsnprintf(
                &raw mut out as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%ud\0" as *const u8 as *const ::core::ffi::c_char,
                d,
            );
        }
    } else if h != 0 as u_int {
        if m != 0 as u_int {
            xsnprintf(
                &raw mut out as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%uh%um\0" as *const u8 as *const ::core::ffi::c_char,
                h,
                m,
            );
        } else {
            xsnprintf(
                &raw mut out as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%uh\0" as *const u8 as *const ::core::ffi::c_char,
                h,
            );
        }
    } else if m != 0 as u_int {
        if s != 0 as u_int {
            xsnprintf(
                &raw mut out as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%um%us\0" as *const u8 as *const ::core::ffi::c_char,
                m,
                s,
            );
        } else {
            xsnprintf(
                &raw mut out as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%um\0" as *const u8 as *const ::core::ffi::c_char,
                m,
            );
        }
    } else {
        xsnprintf(
            &raw mut out as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"%us\0" as *const u8 as *const ::core::ffi::c_char,
            s,
        );
    }
    return xstrdup(&raw mut out as *mut ::core::ffi::c_char);
}
unsafe extern "C" fn format_time_difference(mut t: time_t) -> *mut ::core::ffi::c_char {
    let mut now: time_t = time(::core::ptr::null_mut::<time_t>());
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    xasprintf(
        &raw mut out,
        b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
        now - t,
    );
    return out;
}
unsafe extern "C" fn format_find(
    mut ft: *mut format_tree,
    mut key: *const ::core::ffi::c_char,
    mut modifiers: uint64_t,
    mut time_format: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut current_block: u64;
    let mut fte: *const format_table_entry = ::core::ptr::null::<format_table_entry>();
    let mut value: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut fe: *mut format_entry = ::core::ptr::null_mut::<format_entry>();
    let mut fe_find: format_entry = format_entry {
        key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        value: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        time: 0,
        cb: None,
        entry: format_entry_entry {
            rbe_left: ::core::ptr::null_mut::<format_entry>(),
            rbe_right: ::core::ptr::null_mut::<format_entry>(),
            rbe_parent: ::core::ptr::null_mut::<format_entry>(),
            rbe_color: 0,
        },
    };
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut found: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut saved: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: [::core::ffi::c_char; 512] = [0; 512];
    let mut array_key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut t: time_t = 0 as time_t;
    let mut tm: tm = tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    o = options_parse_get(
        global_options,
        key,
        &raw mut array_key,
        0 as ::core::ffi::c_int,
    );
    if o.is_null() && !(*ft).wp.is_null() {
        o = options_parse_get(
            (*(*ft).wp).options,
            key,
            &raw mut array_key,
            0 as ::core::ffi::c_int,
        );
    }
    if o.is_null() && !(*ft).w.is_null() {
        o = options_parse_get(
            (*(*ft).w).options,
            key,
            &raw mut array_key,
            0 as ::core::ffi::c_int,
        );
    }
    if o.is_null() {
        o = options_parse_get(
            global_w_options,
            key,
            &raw mut array_key,
            0 as ::core::ffi::c_int,
        );
    }
    if o.is_null() && !(*ft).s.is_null() {
        o = options_parse_get(
            (*(*ft).s).options,
            key,
            &raw mut array_key,
            0 as ::core::ffi::c_int,
        );
    }
    if o.is_null() {
        o = options_parse_get(
            global_s_options,
            key,
            &raw mut array_key,
            0 as ::core::ffi::c_int,
        );
    }
    if !o.is_null() {
        found = options_to_string(o, array_key, 1 as ::core::ffi::c_int);
        free(array_key as *mut ::core::ffi::c_void);
    } else {
        fte = format_table_get(key);
        if !fte.is_null() {
            value = (*fte).cb.expect("non-null function pointer")(ft);
            if (*fte).type_0 as ::core::ffi::c_uint
                == FORMAT_TABLE_TIME as ::core::ffi::c_int as ::core::ffi::c_uint
                && !value.is_null()
            {
                t = (*(value as *mut timeval)).tv_sec as time_t;
            } else {
                found = value as *mut ::core::ffi::c_char;
            }
        } else {
            fe_find.key = key as *mut ::core::ffi::c_char;
            fe = format_entry_tree_RB_FIND(&raw mut (*ft).tree, &raw mut fe_find);
            if !fe.is_null() {
                if (*fe).time != 0 as time_t {
                    t = (*fe).time;
                } else {
                    if (*fe).value.is_null() && (*fe).cb.is_some() {
                        (*fe).value = (*fe).cb.expect("non-null function pointer")(ft)
                            as *mut ::core::ffi::c_char;
                        if (*fe).value.is_null() {
                            (*fe).value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                        }
                    }
                    found = xstrdup((*fe).value);
                }
            } else {
                if !modifiers & FORMAT_TIMESTRING as uint64_t != 0 {
                    envent = ::core::ptr::null_mut::<environ_entry>();
                    if !(*ft).s.is_null() {
                        envent = environ_find((*(*ft).s).environ, key);
                    }
                    if envent.is_null() {
                        envent = environ_find(global_environ, key);
                    }
                    if !envent.is_null() && !(*envent).value.is_null() {
                        found = xstrdup((*envent).value);
                        current_block = 11739001764845178280;
                    } else {
                        current_block = 1836292691772056875;
                    }
                } else {
                    current_block = 1836292691772056875;
                }
                match current_block {
                    11739001764845178280 => {}
                    _ => return ::core::ptr::null_mut::<::core::ffi::c_char>(),
                }
            }
        }
    }
    if modifiers & FORMAT_TIMESTRING as uint64_t != 0 {
        if t == 0 as time_t && !found.is_null() {
            t = strtonum(
                found,
                0 as ::core::ffi::c_longlong,
                INT64_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as time_t;
            if !errstr.is_null() {
                t = 0 as time_t;
            }
            free(found as *mut ::core::ffi::c_void);
        }
        if t == 0 as time_t {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if modifiers & FORMAT_RELATIVE as uint64_t != 0 {
            found = format_relative_time(t);
        } else if modifiers as ::core::ffi::c_ulonglong & FORMAT_DIFFERENCE != 0 {
            found = format_time_difference(t);
        } else if modifiers & FORMAT_PRETTY as uint64_t != 0 {
            found = format_pretty_time(t, 0 as ::core::ffi::c_int);
        } else {
            if !time_format.is_null() {
                localtime_r(&raw mut t, &raw mut tm);
                format_strftime(
                    &raw mut s as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 512]>() as size_t,
                    time_format,
                    &raw mut tm,
                );
            } else {
                ctime_r(&raw mut t, &raw mut s as *mut ::core::ffi::c_char);
                s[strcspn(
                    &raw mut s as *mut ::core::ffi::c_char,
                    b"\n\0" as *const u8 as *const ::core::ffi::c_char,
                ) as usize] = '\0' as i32 as ::core::ffi::c_char;
            }
            found = xstrdup(&raw mut s as *mut ::core::ffi::c_char);
        }
        return found;
    }
    if t != 0 as time_t {
        xasprintf(
            &raw mut found,
            b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
            t as ::core::ffi::c_longlong,
        );
    } else if found.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if modifiers & FORMAT_BASENAME as uint64_t != 0 {
        saved = found;
        found = xstrdup(__xpg_basename(saved));
        free(saved as *mut ::core::ffi::c_void);
    }
    if modifiers & FORMAT_DIRNAME as uint64_t != 0 {
        saved = found;
        found = xstrdup(dirname(saved));
        free(saved as *mut ::core::ffi::c_void);
    }
    if modifiers & FORMAT_QUOTE_SHELL as uint64_t != 0 {
        saved = found;
        found = format_quote_shell(saved);
        free(saved as *mut ::core::ffi::c_void);
    }
    if modifiers & FORMAT_QUOTE_SHELL_SQ as uint64_t != 0 {
        saved = found;
        found = format_quote_shell_single(saved);
        free(saved as *mut ::core::ffi::c_void);
    }
    if modifiers & FORMAT_QUOTE_STYLE as uint64_t != 0 {
        saved = found;
        found = format_quote_style(saved);
        free(saved as *mut ::core::ffi::c_void);
    }
    if modifiers & FORMAT_QUOTE_ARGUMENTS as uint64_t != 0 {
        saved = found;
        found = args_escape(saved);
        free(saved as *mut ::core::ffi::c_void);
    }
    return found;
}
unsafe extern "C" fn format_check_time(
    mut es: *mut format_expand_state,
    mut check: *mut u_int,
) -> ::core::ffi::c_int {
    let mut t: uint64_t = 0;
    if !check.is_null() && {
        *check = (*check).wrapping_add(1);
        (*check).wrapping_rem(FORMAT_TIME_LOOP_CHECK as u_int) != 0 as u_int
    } {
        return 1 as ::core::ffi::c_int;
    }
    t = get_timer();
    if t.wrapping_sub((*es).start_time) < FORMAT_TIME_LIMIT as uint64_t {
        return 1 as ::core::ffi::c_int;
    }
    t = t.wrapping_sub((*es).start_time);
    format_log1(
        es,
        b"format_check_time\0" as *const u8 as *const ::core::ffi::c_char,
        b"reached time limit (%llu)\0" as *const u8 as *const ::core::ffi::c_char,
        t as ::core::ffi::c_ulonglong,
    );
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn format_unescape(
    mut es: *mut format_expand_state,
    mut s: *const ::core::ffi::c_char,
    mut n: size_t,
) -> *mut ::core::ffi::c_char {
    let mut end: *const ::core::ffi::c_char = s.offset(n as isize);
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut brackets: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut check: u_int = 0 as u_int;
    out = xmalloc(n.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    cp = out;
    while s != end {
        if format_check_time(es, &raw mut check) == 0 {
            free(out as *mut ::core::ffi::c_void);
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        }
        if *s as ::core::ffi::c_int == '#' as i32
            && s.offset(1 as ::core::ffi::c_int as isize) != end
            && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '{' as i32
        {
            brackets += 1;
        }
        if brackets == 0 as ::core::ffi::c_int
            && *s as ::core::ffi::c_int == '#' as i32
            && s.offset(1 as ::core::ffi::c_int as isize) != end
            && !strchr(
                b",#{}:\0" as *const u8 as *const ::core::ffi::c_char,
                *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            )
            .is_null()
        {
            s = s.offset(1);
            let fresh22 = cp;
            cp = cp.offset(1);
            *fresh22 = *s;
        } else {
            if *s as ::core::ffi::c_int == '}' as i32 {
                brackets -= 1;
            }
            let fresh23 = cp;
            cp = cp.offset(1);
            *fresh23 = *s;
        }
        s = s.offset(1);
    }
    *cp = '\0' as i32 as ::core::ffi::c_char;
    return out;
}
unsafe extern "C" fn format_strip(
    mut es: *mut format_expand_state,
    mut s: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut brackets: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut check: u_int = 0 as u_int;
    out = xmalloc(strlen(s).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    cp = out;
    while *s as ::core::ffi::c_int != '\0' as i32 {
        if format_check_time(es, &raw mut check) == 0 {
            free(out as *mut ::core::ffi::c_void);
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        }
        if *s as ::core::ffi::c_int == '#' as i32
            && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '{' as i32
        {
            brackets += 1;
        }
        if *s as ::core::ffi::c_int == '#' as i32
            && !strchr(
                b",#{}:\0" as *const u8 as *const ::core::ffi::c_char,
                *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            )
            .is_null()
        {
            if brackets != 0 as ::core::ffi::c_int {
                let fresh24 = cp;
                cp = cp.offset(1);
                *fresh24 = *s;
            }
        } else {
            if *s as ::core::ffi::c_int == '}' as i32 {
                brackets -= 1;
            }
            let fresh25 = cp;
            cp = cp.offset(1);
            *fresh25 = *s;
        }
        s = s.offset(1);
    }
    *cp = '\0' as i32 as ::core::ffi::c_char;
    return out;
}
unsafe extern "C" fn format_skip1(
    mut es: *mut format_expand_state,
    mut s: *const ::core::ffi::c_char,
    mut end: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut brackets: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut check: u_int = 0 as u_int;
    while *s as ::core::ffi::c_int != '\0' as i32 {
        if !es.is_null() && format_check_time(es, &raw mut check) == 0 {
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
                b",#{}:\0" as *const u8 as *const ::core::ffi::c_char,
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
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn format_skip(
    mut s: *const ::core::ffi::c_char,
    mut end: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    return format_skip1(::core::ptr::null_mut::<format_expand_state>(), s, end);
}
unsafe extern "C" fn format_choose(
    mut es: *mut format_expand_state,
    mut s: *const ::core::ffi::c_char,
    mut left: *mut *mut ::core::ffi::c_char,
    mut right: *mut *mut ::core::ffi::c_char,
    mut expand: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut left0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut right0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    cp = format_skip1(es, s, b",\0" as *const u8 as *const ::core::ffi::c_char);
    if cp.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    left0 = xstrndup(s, cp.offset_from(s) as ::core::ffi::c_long as size_t);
    right0 = xstrdup(cp.offset(1 as ::core::ffi::c_int as isize));
    if expand != 0 {
        *left = format_expand1(es, left0);
        free(left0 as *mut ::core::ffi::c_void);
        *right = format_expand1(es, right0);
        free(right0 as *mut ::core::ffi::c_void);
    } else {
        *left = left0;
        *right = right0;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn format_true(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if !s.is_null()
        && *s as ::core::ffi::c_int != '\0' as i32
        && (*s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '0' as i32
            || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32)
    {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn format_is_end(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (c as ::core::ffi::c_int == ';' as i32 || c as ::core::ffi::c_int == ':' as i32)
        as ::core::ffi::c_int;
}
unsafe extern "C" fn format_add_modifier(
    mut list: *mut *mut format_modifier,
    mut count: *mut u_int,
    mut c: *const ::core::ffi::c_char,
    mut n: size_t,
    mut argv: *mut *mut ::core::ffi::c_char,
    mut argc: ::core::ffi::c_int,
) {
    let mut fm: *mut format_modifier = ::core::ptr::null_mut::<format_modifier>();
    *list = xreallocarray(
        *list as *mut ::core::ffi::c_void,
        (*count).wrapping_add(1 as u_int) as size_t,
        ::core::mem::size_of::<format_modifier>() as size_t,
    ) as *mut format_modifier;
    let fresh29 = *count;
    *count = (*count).wrapping_add(1);
    fm = (*list).offset(fresh29 as isize) as *mut format_modifier;
    memcpy(
        &raw mut (*fm).modifier as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        c as *const ::core::ffi::c_void,
        n,
    );
    (*fm).modifier[n as usize] = '\0' as i32 as ::core::ffi::c_char;
    (*fm).size = n as u_int;
    (*fm).argv = argv;
    (*fm).argc = argc;
}
unsafe extern "C" fn format_free_modifiers(mut list: *mut format_modifier, mut count: u_int) {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < count {
        cmd_free_argv(
            (*list.offset(i as isize)).argc,
            (*list.offset(i as isize)).argv,
        );
        i = i.wrapping_add(1);
    }
    free(list as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn format_build_modifiers(
    mut es: *mut format_expand_state,
    mut s: *mut *const ::core::ffi::c_char,
    mut count: *mut u_int,
) -> *mut format_modifier {
    let mut cp: *const ::core::ffi::c_char = *s;
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut list: *mut format_modifier = ::core::ptr::null_mut::<format_modifier>();
    let mut c: ::core::ffi::c_char = 0;
    let mut last: [::core::ffi::c_char; 4] =
        ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"X;:\0");
    let mut argv: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut argc: ::core::ffi::c_int = 0;
    *count = 0 as u_int;
    while *cp as ::core::ffi::c_int != '\0' as i32 && *cp as ::core::ffi::c_int != ':' as i32 {
        if *cp as ::core::ffi::c_int == ';' as i32 {
            cp = cp.offset(1);
        }
        if *cp as ::core::ffi::c_int == '\0' as i32 {
            break;
        }
        if !strchr(
            b"labdnwETSWPOVL!<>A\0" as *const u8 as *const ::core::ffi::c_char,
            *cp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
        )
        .is_null()
            && format_is_end(*cp.offset(1 as ::core::ffi::c_int as isize)) != 0
        {
            format_add_modifier(
                &raw mut list,
                count,
                cp,
                1 as size_t,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
            );
            cp = cp.offset(1);
        } else if (memcmp(
            b"||\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            cp as *const ::core::ffi::c_void,
            2 as size_t,
        ) == 0 as ::core::ffi::c_int
            || memcmp(
                b"&&\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                cp as *const ::core::ffi::c_void,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
            || memcmp(
                b"!!\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                cp as *const ::core::ffi::c_void,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
            || memcmp(
                b"!=\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                cp as *const ::core::ffi::c_void,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
            || memcmp(
                b"==\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                cp as *const ::core::ffi::c_void,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
            || memcmp(
                b"<=\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                cp as *const ::core::ffi::c_void,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
            || memcmp(
                b">=\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                cp as *const ::core::ffi::c_void,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int)
            && format_is_end(*cp.offset(2 as ::core::ffi::c_int as isize)) != 0
        {
            format_add_modifier(
                &raw mut list,
                count,
                cp,
                2 as size_t,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
            );
            cp = cp.offset(2 as ::core::ffi::c_int as isize);
        } else {
            if strchr(
                b"ImCLNPSOVst=pReqWcA\0" as *const u8 as *const ::core::ffi::c_char,
                *cp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            )
            .is_null()
            {
                break;
            }
            c = *cp.offset(0 as ::core::ffi::c_int as isize);
            if format_is_end(*cp.offset(1 as ::core::ffi::c_int as isize)) != 0 {
                format_add_modifier(
                    &raw mut list,
                    count,
                    cp,
                    1 as size_t,
                    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                    0 as ::core::ffi::c_int,
                );
                cp = cp.offset(1);
            } else {
                argv = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
                argc = 0 as ::core::ffi::c_int;
                if *(*__ctype_b_loc())
                    .offset(*cp.offset(1 as ::core::ffi::c_int as isize) as u_char
                        as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    & _ISpunct as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0
                    || *cp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '-' as i32
                {
                    end = format_skip1(
                        es,
                        cp.offset(1 as ::core::ffi::c_int as isize),
                        b":;\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                    if end.is_null() {
                        break;
                    }
                    argv = xcalloc(
                        1 as size_t,
                        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
                    ) as *mut *mut ::core::ffi::c_char;
                    value = format_unescape(
                        es,
                        cp.offset(1 as ::core::ffi::c_int as isize),
                        end.offset_from(cp.offset(1 as ::core::ffi::c_int as isize))
                            as ::core::ffi::c_long as size_t,
                    );
                    let ref mut fresh26 = *argv.offset(0 as ::core::ffi::c_int as isize);
                    *fresh26 = format_expand1(es, value);
                    free(value as *mut ::core::ffi::c_void);
                    argc = 1 as ::core::ffi::c_int;
                    format_add_modifier(&raw mut list, count, &raw mut c, 1 as size_t, argv, argc);
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
                                es,
                                cp.offset(1 as ::core::ffi::c_int as isize),
                                &raw mut last as *mut ::core::ffi::c_char,
                            );
                            if end.is_null() {
                                break;
                            }
                            cp = cp.offset(1);
                            argv = xreallocarray(
                                argv as *mut ::core::ffi::c_void,
                                (argc + 1 as ::core::ffi::c_int) as size_t,
                                ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
                            ) as *mut *mut ::core::ffi::c_char;
                            value = format_unescape(
                                es,
                                cp,
                                end.offset_from(cp) as ::core::ffi::c_long as size_t,
                            );
                            let fresh27 = argc;
                            argc = argc + 1;
                            let ref mut fresh28 = *argv.offset(fresh27 as isize);
                            *fresh28 = format_expand1(es, value);
                            free(value as *mut ::core::ffi::c_void);
                            cp = end;
                            if !(format_is_end(*cp.offset(0 as ::core::ffi::c_int as isize)) == 0) {
                                break;
                            }
                        }
                    }
                    format_add_modifier(&raw mut list, count, &raw mut c, 1 as size_t, argv, argc);
                }
            }
        }
    }
    if *cp as ::core::ffi::c_int != ':' as i32 {
        format_free_modifiers(list, *count);
        *count = 0 as u_int;
        return ::core::ptr::null_mut::<format_modifier>();
    }
    *s = cp.offset(1 as ::core::ffi::c_int as isize);
    return list;
}
unsafe extern "C" fn format_match_fuzzy(
    mut pattern: *const ::core::ffi::c_char,
    mut text: *const ::core::ffi::c_char,
    mut positions: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut bs: *mut bitstr_t = ::core::ptr::null_mut::<bitstr_t>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut size: size_t = 0;
    let mut i: u_int = 0;
    let mut width: u_int = 0;
    width = format_width(text);
    if width == 0 as u_int {
        width = 1 as u_int;
    }
    bs = fuzzy_match(pattern, text, width, ::core::ptr::null_mut::<u_int>());
    if bs.is_null() {
        return xstrdup(if positions != 0 {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"0\0" as *const u8 as *const ::core::ffi::c_char
        });
    }
    if positions == 0 {
        free(bs as *mut ::core::ffi::c_void);
        return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    i = 0 as u_int;
    while i < width {
        if !(*bs.offset((i >> 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << (i & 0x7 as u_int)
            == 0)
        {
            if evbuffer_get_length(buffer) != 0 as size_t {
                evbuffer_add(
                    buffer,
                    b",\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                    1 as size_t,
                );
            }
            evbuffer_add_printf(
                buffer,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                i,
            );
        }
        i = i.wrapping_add(1);
    }
    size = evbuffer_get_length(buffer);
    if size != 0 as size_t {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
    } else {
        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    evbuffer_free(buffer);
    free(bs as *mut ::core::ffi::c_void);
    return value;
}
unsafe extern "C" fn format_match(
    mut fm: *mut format_modifier,
    mut pattern: *const ::core::ffi::c_char,
    mut text: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut s: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut r: regex_t = re_pattern_buffer {
        buffer: ::core::ptr::null_mut::<re_dfa_t>(),
        allocated: 0,
        used: 0,
        syntax: 0,
        fastmap: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        translate: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
        re_nsub: 0,
        can_be_null_regs_allocated_fastmap_accurate_no_sub_not_bol_not_eol_newline_anchor: [0; 1],
        c2rust_padding: [0; 7],
    };
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*fm).argc >= 1 as ::core::ffi::c_int {
        s = *(*fm).argv.offset(0 as ::core::ffi::c_int as isize);
    }
    if !strchr(s, 'p' as i32).is_null() {
        return format_match_fuzzy(pattern, text, 1 as ::core::ffi::c_int);
    }
    if !strchr(s, 'z' as i32).is_null() {
        return format_match_fuzzy(pattern, text, 0 as ::core::ffi::c_int);
    }
    if strchr(s, 'r' as i32).is_null() {
        if !strchr(s, 'i' as i32).is_null() {
            flags |= FNM_CASEFOLD;
        }
        if fnmatch(pattern, text, flags) != 0 as ::core::ffi::c_int {
            return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
        }
    } else {
        flags = REG_EXTENDED | REG_NOSUB;
        if !strchr(s, 'i' as i32).is_null() {
            flags |= REG_ICASE;
        }
        if regcomp(&raw mut r, pattern, flags) != 0 as ::core::ffi::c_int {
            return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
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
            return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
        }
        regfree(&raw mut r);
    }
    return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
}
unsafe extern "C" fn format_sub(
    mut fm: *mut format_modifier,
    mut text: *const ::core::ffi::c_char,
    mut pattern: *const ::core::ffi::c_char,
    mut with: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut flags: ::core::ffi::c_int = REG_EXTENDED;
    if (*fm).argc >= 3 as ::core::ffi::c_int
        && !strchr(
            *(*fm).argv.offset(2 as ::core::ffi::c_int as isize),
            'i' as i32,
        )
        .is_null()
    {
        flags |= REG_ICASE;
    }
    value = regsub(pattern, with, text, flags);
    if value.is_null() {
        return xstrdup(text);
    }
    return value;
}
unsafe extern "C" fn format_search(
    mut fm: *mut format_modifier,
    mut wp: *mut window_pane,
    mut s: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut ignore: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut regex: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*fm).argc >= 1 as ::core::ffi::c_int {
        if !strchr(
            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
            'i' as i32,
        )
        .is_null()
        {
            ignore = 1 as ::core::ffi::c_int;
        }
        if !strchr(
            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
            'r' as i32,
        )
        .is_null()
        {
            regex = 1 as ::core::ffi::c_int;
        }
    }
    xasprintf(
        &raw mut value,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        window_pane_search(wp, s, regex, ignore),
    );
    return value;
}
unsafe extern "C" fn format_bool_op_1(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    mut not: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut result: ::core::ffi::c_int = 0;
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    expanded = format_expand1(es, fmt);
    result = format_true(expanded);
    if not != 0 {
        result = (result == 0) as ::core::ffi::c_int;
    }
    free(expanded as *mut ::core::ffi::c_void);
    return xstrdup(if result != 0 {
        b"1\0" as *const u8 as *const ::core::ffi::c_char
    } else {
        b"0\0" as *const u8 as *const ::core::ffi::c_char
    });
}
unsafe extern "C" fn format_bool_op_n(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    mut and: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut result: ::core::ffi::c_int = 0;
    let mut cp1: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cp2: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut raw: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    result = if and != 0 {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    cp1 = fmt;
    while if and != 0 {
        result
    } else {
        (result == 0) as ::core::ffi::c_int
    } != 0
    {
        cp2 = format_skip1(es, cp1, b",\0" as *const u8 as *const ::core::ffi::c_char);
        if cp2.is_null() {
            raw = xstrdup(cp1);
        } else {
            raw = xstrndup(cp1, cp2.offset_from(cp1) as ::core::ffi::c_long as size_t);
        }
        expanded = format_expand1(es, raw);
        free(raw as *mut ::core::ffi::c_void);
        format_log1(
            es,
            b"format_bool_op_n\0" as *const u8 as *const ::core::ffi::c_char,
            b"operator %s has operand: %s\0" as *const u8 as *const ::core::ffi::c_char,
            if and != 0 {
                b"&&\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"||\0" as *const u8 as *const ::core::ffi::c_char
            },
            expanded,
        );
        if and != 0 {
            result = (result != 0 && format_true(expanded) != 0) as ::core::ffi::c_int;
        } else {
            result = (result != 0 || format_true(expanded) != 0) as ::core::ffi::c_int;
        }
        free(expanded as *mut ::core::ffi::c_void);
        if cp2.is_null() {
            break;
        }
        cp1 = cp2.offset(1 as ::core::ffi::c_int as isize);
    }
    return xstrdup(if result != 0 {
        b"1\0" as *const u8 as *const ::core::ffi::c_char
    } else {
        b"0\0" as *const u8 as *const ::core::ffi::c_char
    });
}
unsafe extern "C" fn format_session_name(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    name = format_expand1(es, fmt);
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        if strcmp((*s).name, name) == 0 as ::core::ffi::c_int {
            free(name as *mut ::core::ffi::c_void);
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
        }
        s = sessions_RB_NEXT(s);
    }
    free(name as *mut ::core::ffi::c_void);
    return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
}
unsafe extern "C" fn format_loop_sessions(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut sc: *mut sort_criteria = &raw mut sort_crit;
    let mut ft: *mut format_tree = (*es).ft;
    let mut c: *mut client = (*ft).client;
    let mut item: *mut cmdq_item = (*ft).item;
    let mut nft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    let mut all: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut active: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut use_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: size_t = 0;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut l: *mut *mut session = ::core::ptr::null_mut::<*mut session>();
    let mut i: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    if format_choose(
        es,
        fmt,
        &raw mut all,
        &raw mut active,
        0 as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        all = xstrdup(fmt);
        active = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    l = sort_get_sessions(&raw mut n as *mut u_int, sc);
    i = 0 as ::core::ffi::c_int;
    while i < n {
        s = *l.offset(i as isize);
        format_log1(
            es,
            b"format_loop_sessions\0" as *const u8 as *const ::core::ffi::c_char,
            b"session loop: $%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*s).id,
        );
        if !active.is_null()
            && !(*ft).c.is_null()
            && !(*(*ft).c).session.is_null()
            && (*s).id == (*(*(*ft).c).session).id
        {
            use_0 = active;
        } else {
            use_0 = all;
        }
        nft = format_create(c, item, FORMAT_NONE, (*ft).flags);
        format_add(
            nft,
            b"loop_index\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        format_add(
            nft,
            b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (i == n - 1 as ::core::ffi::c_int) as ::core::ffi::c_int,
        );
        format_defaults(
            nft,
            (*ft).c,
            s,
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
        format_copy_state(&raw mut next, es, 0 as ::core::ffi::c_int);
        next.ft = nft;
        expanded = format_expand1(&raw mut next, use_0);
        format_free(next.ft);
        evbuffer_add(
            buffer,
            expanded as *const ::core::ffi::c_void,
            strlen(expanded),
        );
        free(expanded as *mut ::core::ffi::c_void);
        i += 1;
    }
    free(active as *mut ::core::ffi::c_void);
    free(all as *mut ::core::ffi::c_void);
    size = evbuffer_get_length(buffer);
    if size != 0 as size_t {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
    } else {
        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    evbuffer_free(buffer);
    return value;
}
unsafe extern "C" fn format_window_name(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut ft: *mut format_tree = (*es).ft;
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if (*ft).s.is_null() {
        format_log1(
            es,
            b"format_window_name\0" as *const u8 as *const ::core::ffi::c_char,
            b"window name but no session\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    name = format_expand1(es, fmt);
    wl = winlinks_RB_MINMAX(&raw mut (*(*ft).s).windows, RB_NEGINF);
    while !wl.is_null() {
        if strcmp((*(*wl).window).name, name) == 0 as ::core::ffi::c_int {
            free(name as *mut ::core::ffi::c_void);
            return xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
        }
        wl = winlinks_RB_NEXT(wl);
    }
    free(name as *mut ::core::ffi::c_void);
    return xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
}
unsafe extern "C" fn format_add_window_neighbour(
    mut nft: *mut format_tree,
    mut wl: *mut winlink,
    mut s: *mut session,
    mut prefix: *const ::core::ffi::c_char,
) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut prefixed: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut oval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    xasprintf(
        &raw mut key,
        b"%s_window_index\0" as *const u8 as *const ::core::ffi::c_char,
        prefix,
    );
    format_add(
        nft,
        key,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).idx,
    );
    free(key as *mut ::core::ffi::c_void);
    xasprintf(
        &raw mut key,
        b"%s_window_active\0" as *const u8 as *const ::core::ffi::c_char,
        prefix,
    );
    format_add(
        nft,
        key,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        (wl == (*s).curw) as ::core::ffi::c_int,
    );
    free(key as *mut ::core::ffi::c_void);
    o = options_first((*(*wl).window).options);
    while !o.is_null() {
        oname = options_name(o);
        if *oname as ::core::ffi::c_int == '@' as i32 {
            xasprintf(
                &raw mut prefixed,
                b"%s_%s\0" as *const u8 as *const ::core::ffi::c_char,
                prefix,
                oname,
            );
            oval = options_to_string(
                o,
                ::core::ptr::null::<::core::ffi::c_char>(),
                1 as ::core::ffi::c_int,
            );
            format_add(
                nft,
                prefixed,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                oval,
            );
            free(oval as *mut ::core::ffi::c_void);
            free(prefixed as *mut ::core::ffi::c_void);
        }
        o = options_next(o);
    }
}
unsafe extern "C" fn format_loop_windows(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut sc: *mut sort_criteria = &raw mut sort_crit;
    let mut ft: *mut format_tree = (*es).ft;
    let mut c: *mut client = (*ft).client;
    let mut s: *mut session = (*ft).s;
    let mut item: *mut cmdq_item = (*ft).item;
    let mut nft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    let mut all: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut active: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut use_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: size_t = 0;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut l: *mut *mut winlink = ::core::ptr::null_mut::<*mut winlink>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut i: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    if s.is_null() {
        format_log1(
            es,
            b"format_loop_windows\0" as *const u8 as *const ::core::ffi::c_char,
            b"window loop but no session\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if format_choose(
        es,
        fmt,
        &raw mut all,
        &raw mut active,
        0 as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        all = xstrdup(fmt);
        active = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    l = sort_get_winlinks_session(s, &raw mut n as *mut u_int, sc);
    i = 0 as ::core::ffi::c_int;
    while i < n {
        wl = *l.offset(i as isize);
        w = (*wl).window;
        format_log1(
            es,
            b"format_loop_windows\0" as *const u8 as *const ::core::ffi::c_char,
            b"window loop: %u @%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wl).idx,
            (*w).id,
        );
        if !active.is_null() && wl == (*s).curw {
            use_0 = active;
        } else {
            use_0 = all;
        }
        nft = format_create(
            c,
            item,
            (FORMAT_WINDOW | (*w).id) as ::core::ffi::c_int,
            (*ft).flags,
        );
        format_add(
            nft,
            b"loop_index\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        format_add(
            nft,
            b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (i == n - 1 as ::core::ffi::c_int) as ::core::ffi::c_int,
        );
        if i > 0 as ::core::ffi::c_int
            && *l.offset((i - 1 as ::core::ffi::c_int) as isize) == (*s).curw
        {
            format_add(
                nft,
                b"window_after_active\0" as *const u8 as *const ::core::ffi::c_char,
                b"1\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else {
            format_add(
                nft,
                b"window_after_active\0" as *const u8 as *const ::core::ffi::c_char,
                b"0\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if (i + 1 as ::core::ffi::c_int) < n
            && *l.offset((i + 1 as ::core::ffi::c_int) as isize) == (*s).curw
        {
            format_add(
                nft,
                b"window_before_active\0" as *const u8 as *const ::core::ffi::c_char,
                b"1\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else {
            format_add(
                nft,
                b"window_before_active\0" as *const u8 as *const ::core::ffi::c_char,
                b"0\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if (i + 1 as ::core::ffi::c_int) < n {
            format_add_window_neighbour(
                nft,
                *l.offset((i + 1 as ::core::ffi::c_int) as isize),
                s,
                b"next\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if i > 0 as ::core::ffi::c_int {
            format_add_window_neighbour(
                nft,
                *l.offset((i - 1 as ::core::ffi::c_int) as isize),
                s,
                b"prev\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        format_defaults(nft, (*ft).c, s, wl, ::core::ptr::null_mut::<window_pane>());
        format_copy_state(&raw mut next, es, 0 as ::core::ffi::c_int);
        next.ft = nft;
        expanded = format_expand1(&raw mut next, use_0);
        format_free(nft);
        evbuffer_add(
            buffer,
            expanded as *const ::core::ffi::c_void,
            strlen(expanded),
        );
        free(expanded as *mut ::core::ffi::c_void);
        i += 1;
    }
    free(active as *mut ::core::ffi::c_void);
    free(all as *mut ::core::ffi::c_void);
    size = evbuffer_get_length(buffer);
    if size != 0 as size_t {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
    } else {
        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    evbuffer_free(buffer);
    return value;
}
unsafe extern "C" fn format_loop_panes(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut sc: *mut sort_criteria = &raw mut sort_crit;
    let mut ft: *mut format_tree = (*es).ft;
    let mut c: *mut client = (*ft).client;
    let mut item: *mut cmdq_item = (*ft).item;
    let mut nft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    let mut all: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut active: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut use_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: size_t = 0;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut l: *mut *mut window_pane = ::core::ptr::null_mut::<*mut window_pane>();
    let mut i: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    if (*ft).w.is_null() {
        format_log1(
            es,
            b"format_loop_panes\0" as *const u8 as *const ::core::ffi::c_char,
            b"pane loop but no window\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if format_choose(
        es,
        fmt,
        &raw mut all,
        &raw mut active,
        0 as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        all = xstrdup(fmt);
        active = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    l = sort_get_panes_window((*ft).w, &raw mut n as *mut u_int, sc);
    i = 0 as ::core::ffi::c_int;
    while i < n {
        wp = *l.offset(i as isize);
        format_log1(
            es,
            b"format_loop_panes\0" as *const u8 as *const ::core::ffi::c_char,
            b"pane loop: %%%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
        );
        if !active.is_null() && wp == (*(*ft).w).active {
            use_0 = active;
        } else {
            use_0 = all;
        }
        nft = format_create(
            c,
            item,
            (FORMAT_PANE | (*wp).id) as ::core::ffi::c_int,
            (*ft).flags,
        );
        format_add(
            nft,
            b"loop_index\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        format_add(
            nft,
            b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (i == n - 1 as ::core::ffi::c_int) as ::core::ffi::c_int,
        );
        format_defaults(nft, (*ft).c, (*ft).s, (*ft).wl, wp);
        format_copy_state(&raw mut next, es, 0 as ::core::ffi::c_int);
        next.ft = nft;
        expanded = format_expand1(&raw mut next, use_0);
        format_free(nft);
        evbuffer_add(
            buffer,
            expanded as *const ::core::ffi::c_void,
            strlen(expanded),
        );
        free(expanded as *mut ::core::ffi::c_void);
        i += 1;
    }
    free(active as *mut ::core::ffi::c_void);
    free(all as *mut ::core::ffi::c_void);
    size = evbuffer_get_length(buffer);
    if size != 0 as size_t {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
    } else {
        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    evbuffer_free(buffer);
    return value;
}
unsafe extern "C" fn format_loop_add_option(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    mut buffer: *mut evbuffer,
    mut o: *mut options_entry,
    mut n: u_int,
    mut i: u_int,
) {
    let mut ft: *mut format_tree = (*es).ft;
    let mut nft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    let mut oe: *const options_table_entry = options_table_entry(o);
    let mut name: *const ::core::ffi::c_char = options_name(o);
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut is_array: ::core::ffi::c_int = options_is_array(o);
    format_log1(
        es,
        b"format_loop_add_option\0" as *const u8 as *const ::core::ffi::c_char,
        b"option loop: %s\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    nft = format_create((*ft).client, (*ft).item, FORMAT_NONE, (*ft).flags);
    format_add(
        nft,
        b"option_name\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    s = options_to_string(
        o,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
    );
    format_add(
        nft,
        b"option_value\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    free(s as *mut ::core::ffi::c_void);
    format_add(
        nft,
        b"option_is_array\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        is_array,
    );
    format_add(
        nft,
        b"option_array_key\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"\0" as *const u8 as *const ::core::ffi::c_char,
    );
    format_add(
        nft,
        b"option_array_index\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"\0" as *const u8 as *const ::core::ffi::c_char,
    );
    format_add(
        nft,
        b"option_array_first\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        is_array,
    );
    format_add(
        nft,
        b"option_array_last\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        is_array,
    );
    format_add(
        nft,
        b"option_array_count\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        n,
    );
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0 {
        format_add(
            nft,
            b"option_is_hook\0" as *const u8 as *const ::core::ffi::c_char,
            b"1\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        format_add(
            nft,
            b"option_is_hook\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    format_add(
        nft,
        b"option_is_user\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        (oe == NULL_0 as *const options_table_entry) as ::core::ffi::c_int,
    );
    if options_next(o).is_null() {
        format_add(
            nft,
            b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            b"1\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        format_add(
            nft,
            b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    format_add(
        nft,
        b"loop_index\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        i,
    );
    format_defaults(nft, (*ft).c, (*ft).s, (*ft).wl, (*ft).wp);
    format_copy_state(&raw mut next, es, 0 as ::core::ffi::c_int);
    next.ft = nft;
    expanded = format_expand1(&raw mut next, fmt);
    format_free(nft);
    evbuffer_add(
        buffer,
        expanded as *const ::core::ffi::c_void,
        strlen(expanded),
    );
    free(expanded as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn format_loop_add_array_item(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    mut buffer: *mut evbuffer,
    mut o: *mut options_entry,
    mut a: *mut options_array_item,
    mut n: ::core::ffi::c_int,
    mut i: u_int,
) {
    let mut ft: *mut format_tree = (*es).ft;
    let mut nft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    let mut oe: *const options_table_entry = options_table_entry(o);
    let mut name: *const ::core::ffi::c_char = options_name(o);
    let mut array_key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    array_key = options_array_item_key(a);
    format_log1(
        es,
        b"format_loop_add_array_item\0" as *const u8 as *const ::core::ffi::c_char,
        b"option loop: %s[%s]\0" as *const u8 as *const ::core::ffi::c_char,
        name,
        array_key,
    );
    nft = format_create((*ft).client, (*ft).item, FORMAT_NONE, (*ft).flags);
    format_add(
        nft,
        b"option_name\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    s = options_to_string(o, array_key, 0 as ::core::ffi::c_int);
    format_add(
        nft,
        b"option_value\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    free(s as *mut ::core::ffi::c_void);
    format_add(
        nft,
        b"option_is_array\0" as *const u8 as *const ::core::ffi::c_char,
        b"1\0" as *const u8 as *const ::core::ffi::c_char,
    );
    format_add(
        nft,
        b"option_array_key\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        array_key,
    );
    format_add(
        nft,
        b"option_array_index\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        array_key,
    );
    if a == options_array_first(o) {
        format_add(
            nft,
            b"option_array_first\0" as *const u8 as *const ::core::ffi::c_char,
            b"1\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        format_add(
            nft,
            b"option_array_first\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if options_array_next(a).is_null() {
        format_add(
            nft,
            b"option_array_last\0" as *const u8 as *const ::core::ffi::c_char,
            b"1\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        format_add(
            nft,
            b"option_array_last\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    format_add(
        nft,
        b"option_array_count\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        n,
    );
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0 {
        format_add(
            nft,
            b"option_is_hook\0" as *const u8 as *const ::core::ffi::c_char,
            b"1\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        format_add(
            nft,
            b"option_is_hook\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    format_add(
        nft,
        b"option_is_user\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        (oe == NULL_0 as *const options_table_entry) as ::core::ffi::c_int,
    );
    if options_array_next(a).is_null() && options_next(o).is_null() {
        format_add(
            nft,
            b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            b"1\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        format_add(
            nft,
            b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    format_add(
        nft,
        b"loop_index\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        i,
    );
    format_defaults(nft, (*ft).c, (*ft).s, (*ft).wl, (*ft).wp);
    format_copy_state(&raw mut next, es, 0 as ::core::ffi::c_int);
    next.ft = nft;
    expanded = format_expand1(&raw mut next, fmt);
    format_free(nft);
    evbuffer_add(
        buffer,
        expanded as *const ::core::ffi::c_void,
        strlen(expanded),
    );
    free(expanded as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn format_loop_options(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    mut flags: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut ft: *mut format_tree = (*es).ft;
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: size_t = 0;
    let mut i: u_int = 0 as u_int;
    let mut n: u_int = 0;
    let mut global: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if flags.is_null() || *flags as ::core::ffi::c_int == '\0' as i32 {
        flags = b"s\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if !strchr(flags, 'v' as i32).is_null() {
        oo = global_options;
    } else {
        if !strchr(flags, 'g' as i32).is_null() {
            global = 1 as ::core::ffi::c_int;
        }
        if !strchr(flags, 'w' as i32).is_null() {
            if global != 0 {
                oo = global_w_options;
            } else if !(*ft).w.is_null() {
                oo = (*(*ft).w).options;
            }
        } else if !strchr(flags, 's' as i32).is_null() {
            if global != 0 {
                oo = global_s_options;
            } else if !(*ft).s.is_null() {
                oo = (*(*ft).s).options;
            }
        } else if !strchr(flags, 'p' as i32).is_null() {
            if !(global != 0) {
                if !(*ft).wp.is_null() {
                    oo = (*(*ft).wp).options;
                }
            }
        } else if global != 0 {
            oo = global_s_options;
        }
    }
    if oo.is_null() {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    o = options_first(oo);
    while !o.is_null() {
        n = 0 as u_int;
        if options_is_array(o) != 0 {
            a = options_array_first(o);
            while !a.is_null() {
                n = n.wrapping_add(1);
                a = options_array_next(a);
            }
        }
        if options_is_array(o) == 0 || n == 0 as u_int {
            format_loop_add_option(es, fmt, buffer, o, n, i);
            i = i.wrapping_add(1);
            o = options_next(o);
        } else {
            a = options_array_first(o);
            while !a.is_null() {
                format_loop_add_array_item(es, fmt, buffer, o, a, n as ::core::ffi::c_int, i);
                i = i.wrapping_add(1);
                a = options_array_next(a);
            }
            o = options_next(o);
        }
    }
    size = evbuffer_get_length(buffer);
    if size != 0 as size_t {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
    } else {
        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    evbuffer_free(buffer);
    return value;
}
unsafe extern "C" fn format_loop_environ(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
    mut flags: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut ft: *mut format_tree = (*es).ft;
    let mut nft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut c: *mut client = (*ft).client;
    let mut item: *mut cmdq_item = (*ft).item;
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: size_t = 0;
    let mut i: u_int = 0 as u_int;
    if flags.is_null()
        || *flags as ::core::ffi::c_int == '\0' as i32
        || strcmp(flags, b"s\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        if !(*ft).s.is_null() {
            env = (*(*ft).s).environ;
        }
    } else if strcmp(flags, b"g\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        env = global_environ;
    } else if strcmp(flags, b"c\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        if !(*ft).client.is_null() {
            env = (*(*ft).client).environ;
        }
    }
    if env.is_null() {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    envent = environ_first(env);
    while !envent.is_null() {
        format_log1(
            es,
            b"format_loop_environ\0" as *const u8 as *const ::core::ffi::c_char,
            b"environment loop: %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*envent).name,
        );
        nft = format_create(c, item, FORMAT_NONE, (*ft).flags);
        format_add(
            nft,
            b"environ_name\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*envent).name,
        );
        if (*envent).value.is_null() {
            format_add(
                nft,
                b"environ_value\0" as *const u8 as *const ::core::ffi::c_char,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else {
            format_add(
                nft,
                b"environ_value\0" as *const u8 as *const ::core::ffi::c_char,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*envent).value,
            );
        }
        if (*envent).flags & ENVIRON_HIDDEN != 0 {
            format_add(
                nft,
                b"environ_hidden\0" as *const u8 as *const ::core::ffi::c_char,
                b"1\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else {
            format_add(
                nft,
                b"environ_hidden\0" as *const u8 as *const ::core::ffi::c_char,
                b"0\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        format_add(
            nft,
            b"environ_removed\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            ((*envent).value == NULL_0 as *mut ::core::ffi::c_char) as ::core::ffi::c_int,
        );
        if environ_next(envent).is_null() {
            format_add(
                nft,
                b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
                b"1\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else {
            format_add(
                nft,
                b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
                b"0\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        format_add(
            nft,
            b"loop_index\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        format_defaults(nft, (*ft).c, (*ft).s, (*ft).wl, (*ft).wp);
        format_copy_state(&raw mut next, es, 0 as ::core::ffi::c_int);
        next.ft = nft;
        expanded = format_expand1(&raw mut next, fmt);
        format_free(nft);
        evbuffer_add(
            buffer,
            expanded as *const ::core::ffi::c_void,
            strlen(expanded),
        );
        free(expanded as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
        envent = environ_next(envent);
    }
    size = evbuffer_get_length(buffer);
    if size != 0 as size_t {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
    } else {
        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    evbuffer_free(buffer);
    return value;
}
unsafe extern "C" fn format_loop_clients(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut sc: *mut sort_criteria = &raw mut sort_crit;
    let mut ft: *mut format_tree = (*es).ft;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut l: *mut *mut client = ::core::ptr::null_mut::<*mut client>();
    let mut item: *mut cmdq_item = (*ft).item;
    let mut nft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buffer: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut size: size_t = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    buffer = evbuffer_new();
    if buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    l = sort_get_clients(&raw mut n as *mut u_int, sc);
    i = 0 as ::core::ffi::c_int;
    while i < n {
        c = *l.offset(i as isize);
        format_log1(
            es,
            b"format_loop_clients\0" as *const u8 as *const ::core::ffi::c_char,
            b"client loop: %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
        nft = format_create(c, item, 0 as ::core::ffi::c_int, (*ft).flags);
        format_add(
            nft,
            b"loop_index\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        format_add(
            nft,
            b"loop_last_flag\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            (i == n - 1 as ::core::ffi::c_int) as ::core::ffi::c_int,
        );
        format_defaults(nft, c, (*ft).s, (*ft).wl, (*ft).wp);
        format_copy_state(&raw mut next, es, 0 as ::core::ffi::c_int);
        next.ft = nft;
        expanded = format_expand1(&raw mut next, fmt);
        format_free(nft);
        evbuffer_add(
            buffer,
            expanded as *const ::core::ffi::c_void,
            strlen(expanded),
        );
        free(expanded as *mut ::core::ffi::c_void);
        i += 1;
    }
    size = evbuffer_get_length(buffer);
    if size != 0 as size_t {
        value = xmemdup(
            evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
    } else {
        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    evbuffer_free(buffer);
    return value;
}
unsafe extern "C" fn format_replace_expression(
    mut mexp: *mut format_modifier,
    mut es: *mut format_expand_state,
    mut copy: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut current_block: u64;
    let mut argc: ::core::ffi::c_int = (*mexp).argc;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut endch: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut left: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut right: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut use_fp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut prec: u_int = 0 as u_int;
    let mut mleft: ::core::ffi::c_double = 0.;
    let mut mright: ::core::ffi::c_double = 0.;
    let mut result: ::core::ffi::c_double = 0.;
    let mut operator: C2RustUnnamed_44 = ADD;
    if strcmp(
        *(*mexp).argv.offset(0 as ::core::ffi::c_int as isize),
        b"+\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = ADD;
        current_block = 4495394744059808450;
    } else if strcmp(
        *(*mexp).argv.offset(0 as ::core::ffi::c_int as isize),
        b"-\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = SUBTRACT;
        current_block = 4495394744059808450;
    } else if strcmp(
        *(*mexp).argv.offset(0 as ::core::ffi::c_int as isize),
        b"*\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = MULTIPLY;
        current_block = 4495394744059808450;
    } else if strcmp(
        *(*mexp).argv.offset(0 as ::core::ffi::c_int as isize),
        b"/\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = DIVIDE;
        current_block = 4495394744059808450;
    } else if strcmp(
        *(*mexp).argv.offset(0 as ::core::ffi::c_int as isize),
        b"%\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            *(*mexp).argv.offset(0 as ::core::ffi::c_int as isize),
            b"m\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        operator = MODULUS;
        current_block = 4495394744059808450;
    } else if strcmp(
        *(*mexp).argv.offset(0 as ::core::ffi::c_int as isize),
        b"==\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = EQUAL;
        current_block = 4495394744059808450;
    } else if strcmp(
        *(*mexp).argv.offset(0 as ::core::ffi::c_int as isize),
        b"!=\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = NOT_EQUAL;
        current_block = 4495394744059808450;
    } else if strcmp(
        *(*mexp).argv.offset(0 as ::core::ffi::c_int as isize),
        b">\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = GREATER_THAN;
        current_block = 4495394744059808450;
    } else if strcmp(
        *(*mexp).argv.offset(0 as ::core::ffi::c_int as isize),
        b"<\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = LESS_THAN;
        current_block = 4495394744059808450;
    } else if strcmp(
        *(*mexp).argv.offset(0 as ::core::ffi::c_int as isize),
        b">=\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = GREATER_THAN_EQUAL;
        current_block = 4495394744059808450;
    } else if strcmp(
        *(*mexp).argv.offset(0 as ::core::ffi::c_int as isize),
        b"<=\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        operator = LESS_THAN_EQUAL;
        current_block = 4495394744059808450;
    } else {
        format_log1(
            es,
            b"format_replace_expression\0" as *const u8 as *const ::core::ffi::c_char,
            b"expression has no valid operator: '%s'\0" as *const u8 as *const ::core::ffi::c_char,
            *(*mexp).argv.offset(0 as ::core::ffi::c_int as isize),
        );
        current_block = 7376217411786091060;
    }
    match current_block {
        4495394744059808450 => {
            if argc >= 2 as ::core::ffi::c_int
                && !strchr(
                    *(*mexp).argv.offset(1 as ::core::ffi::c_int as isize),
                    'f' as i32,
                )
                .is_null()
            {
                use_fp = 1 as ::core::ffi::c_int;
                prec = 2 as u_int;
            }
            if argc >= 3 as ::core::ffi::c_int {
                prec = strtonum(
                    *(*mexp).argv.offset(2 as ::core::ffi::c_int as isize),
                    -FORMAT_MAX_PRECISION as ::core::ffi::c_longlong,
                    FORMAT_MAX_PRECISION as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    format_log1(
                        es,
                        b"format_replace_expression\0" as *const u8 as *const ::core::ffi::c_char,
                        b"expression precision %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        errstr,
                        *(*mexp).argv.offset(2 as ::core::ffi::c_int as isize),
                    );
                    current_block = 7376217411786091060;
                } else {
                    current_block = 3437258052017859086;
                }
            } else {
                current_block = 3437258052017859086;
            }
            match current_block {
                7376217411786091060 => {}
                _ => {
                    if format_choose(
                        es,
                        copy,
                        &raw mut left,
                        &raw mut right,
                        1 as ::core::ffi::c_int,
                    ) != 0 as ::core::ffi::c_int
                    {
                        format_log1(
                            es,
                            b"format_replace_expression\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"expression syntax error\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    } else {
                        mleft = strtod(left, &raw mut endch);
                        if *endch as ::core::ffi::c_int != '\0' as i32 {
                            format_log1(
                                es,
                                b"format_replace_expression\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                b"expression left side is invalid: %s\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                left,
                            );
                        } else {
                            mright = strtod(right, &raw mut endch);
                            if *endch as ::core::ffi::c_int != '\0' as i32 {
                                format_log1(
                                    es,
                                    b"format_replace_expression\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"expression right side is invalid: %s\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    right,
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
                                    b"format_replace_expression\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"expression left side is: %.*f\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    prec,
                                    mleft,
                                );
                                format_log1(
                                    es,
                                    b"format_replace_expression\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"expression right side is: %.*f\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    prec,
                                    mright,
                                );
                                match operator as ::core::ffi::c_uint {
                                    0 => {
                                        result = mleft + mright;
                                    }
                                    1 => {
                                        result = mleft - mright;
                                    }
                                    2 => {
                                        result = mleft * mright;
                                    }
                                    3 => {
                                        result = mleft / mright;
                                    }
                                    4 => {
                                        result = fmod(mleft, mright);
                                    }
                                    5 => {
                                        result = (fabs(mleft - mright) < 1e-9f64)
                                            as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    6 => {
                                        result = (fabs(mleft - mright) > 1e-9f64)
                                            as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    7 => {
                                        result = (mleft > mright) as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    8 => {
                                        result = (mleft >= mright) as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    9 => {
                                        result = (mleft < mright) as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    10 => {
                                        result = (mleft <= mright) as ::core::ffi::c_int
                                            as ::core::ffi::c_double;
                                    }
                                    _ => {}
                                }
                                if use_fp != 0 {
                                    xasprintf(
                                        &raw mut value,
                                        b"%.*f\0" as *const u8 as *const ::core::ffi::c_char,
                                        prec,
                                        result,
                                    );
                                } else {
                                    xasprintf(
                                        &raw mut value,
                                        b"%.*f\0" as *const u8 as *const ::core::ffi::c_char,
                                        prec,
                                        result as ::core::ffi::c_longlong as ::core::ffi::c_double,
                                    );
                                }
                                format_log1(
                                    es,
                                    b"format_replace_expression\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    b"expression result is %s\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    value,
                                );
                                free(right as *mut ::core::ffi::c_void);
                                free(left as *mut ::core::ffi::c_void);
                                return value;
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    free(right as *mut ::core::ffi::c_void);
    free(left as *mut ::core::ffi::c_void);
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn format_cycle_callback(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = arg as *mut client;
    if (*c).message_string.is_null() && (*c).prompt.is_null() {
        (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
    }
}
unsafe extern "C" fn format_cycle_start_timer(mut c: *mut client) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    tv.tv_sec = (FORMAT_CYCLE_PERIOD / 1000 as ::core::ffi::c_int) as __time_t;
    tv.tv_usec = ((FORMAT_CYCLE_PERIOD % 1000 as ::core::ffi::c_int) as ::core::ffi::c_long
        * 1000 as ::core::ffi::c_long) as __suseconds_t;
    if event_initialized(&raw mut (*c).cycle_timer) == 0 {
        event_set(
            &raw mut (*c).cycle_timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            Some(
                format_cycle_callback
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            c as *mut ::core::ffi::c_void,
        );
    }
    if event_pending(
        &raw mut (*c).cycle_timer,
        EV_TIMEOUT as ::core::ffi::c_short,
        ::core::ptr::null_mut::<timeval>(),
    ) == 0
    {
        event_add(&raw mut (*c).cycle_timer, &raw mut tv);
    }
}
unsafe extern "C" fn format_cycle(
    mut es: *mut format_expand_state,
    mut frames: *const ::core::ffi::c_char,
    mut count: u_int,
) -> *mut ::core::ffi::c_char {
    let mut ft: *mut format_tree = (*es).ft;
    let mut start: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: u_int = 0;
    let mut index: u_int = 0;
    let mut i: u_int = 0;
    if (*ft).flags & FORMAT_STATUS == 0 || (*es).flags & FORMAT_EXPAND_NOCYCLE != 0 {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if *frames as ::core::ffi::c_int == '\0' as i32 {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    n = 1 as u_int;
    cp = frames;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if *cp as ::core::ffi::c_int == ',' as i32 {
            n = n.wrapping_add(1);
        }
        cp = cp.offset(1);
    }
    index = (*es)
        .start_time
        .wrapping_div(count.wrapping_mul(FORMAT_CYCLE_PERIOD as u_int) as uint64_t)
        .wrapping_rem(n as uint64_t) as u_int;
    if n > 1 as u_int && !(*ft).client.is_null() {
        format_cycle_start_timer((*ft).client);
    }
    start = frames;
    i = 0 as u_int;
    while i < index {
        start = strchr(start, ',' as i32).offset(1 as ::core::ffi::c_int as isize);
        i = i.wrapping_add(1);
    }
    end = strchr(start, ',' as i32);
    if end.is_null() {
        end = start.offset(strlen(start) as isize);
    }
    return xstrndup(
        start,
        end.offset_from(start) as ::core::ffi::c_long as size_t,
    );
}
unsafe extern "C" fn format_replace(
    mut es: *mut format_expand_state,
    mut key: *const ::core::ffi::c_char,
    mut keylen: size_t,
    mut buf: *mut *mut ::core::ffi::c_char,
    mut len: *mut size_t,
    mut off: *mut size_t,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut sc: *mut sort_criteria = &raw mut sort_crit;
    let mut ft: *mut format_tree = (*es).ft;
    let mut wp: *mut window_pane = (*ft).wp;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut copy: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cp2: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut marker: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut time_format: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut copy0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut condition: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut found: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut new: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut left: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut right: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut valuelen: size_t = 0;
    let mut modifiers: uint64_t = 0 as uint64_t;
    let mut limit: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut j: ::core::ffi::c_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    let mut list: *mut format_modifier = ::core::ptr::null_mut::<format_modifier>();
    let mut cmp: *mut format_modifier = ::core::ptr::null_mut::<format_modifier>();
    let mut search: *mut format_modifier = ::core::ptr::null_mut::<format_modifier>();
    let mut sub: *mut *mut format_modifier = ::core::ptr::null_mut::<*mut format_modifier>();
    let mut mexp: *mut format_modifier = ::core::ptr::null_mut::<format_modifier>();
    let mut fm: *mut format_modifier = ::core::ptr::null_mut::<format_modifier>();
    let mut bool_op_n: *mut format_modifier = ::core::ptr::null_mut::<format_modifier>();
    let mut cycle_count: u_int = 1 as u_int;
    let mut i: u_int = 0;
    let mut count: u_int = 0;
    let mut nsub: u_int = 0 as u_int;
    let mut nrep: u_int = 0;
    let mut check: u_int = 0 as u_int;
    let mut loop_flags: *const ::core::ffi::c_char =
        b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut next: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    (*sc).order = SORT_ORDER;
    (*sc).reversed = 0 as ::core::ffi::c_int;
    copy0 = xstrndup(key, keylen);
    copy = copy0;
    list = format_build_modifiers(es, &raw mut copy, &raw mut count);
    i = 0 as u_int;
    while i < count {
        fm = list.offset(i as isize) as *mut format_modifier;
        if format_logging(ft) != 0 {
            format_log1(
                es,
                b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                b"modifier %u is %s\0" as *const u8 as *const ::core::ffi::c_char,
                i,
                &raw mut (*fm).modifier as *mut ::core::ffi::c_char,
            );
            j = 0 as ::core::ffi::c_int;
            while j < (*fm).argc {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"modifier %u argument %d: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    i,
                    j,
                    *(*fm).argv.offset(j as isize),
                );
                j += 1;
            }
        }
        if (*fm).size == 1 as u_int {
            match (*fm).modifier[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int {
                109 | 60 | 62 => {
                    cmp = fm;
                }
                33 => {
                    modifiers |= FORMAT_NOT as uint64_t;
                }
                67 => {
                    search = fm;
                }
                115 => {
                    if !((*fm).argc < 2 as ::core::ffi::c_int) {
                        sub = xreallocarray(
                            sub as *mut ::core::ffi::c_void,
                            nsub.wrapping_add(1 as u_int) as size_t,
                            ::core::mem::size_of::<*mut format_modifier>() as size_t,
                        ) as *mut *mut format_modifier;
                        let fresh16 = nsub;
                        nsub = nsub.wrapping_add(1);
                        let ref mut fresh17 = *sub.offset(fresh16 as isize);
                        *fresh17 = fm;
                    }
                }
                61 => {
                    if !((*fm).argc < 1 as ::core::ffi::c_int) {
                        limit = strtonum(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            -FORMAT_MAX_WIDTH as ::core::ffi::c_longlong,
                            FORMAT_MAX_WIDTH as ::core::ffi::c_longlong,
                            &raw mut errstr,
                        ) as ::core::ffi::c_int;
                        if !errstr.is_null() {
                            limit = 0 as ::core::ffi::c_int;
                        }
                        if (*fm).argc >= 2 as ::core::ffi::c_int
                            && !(*(*fm).argv.offset(1 as ::core::ffi::c_int as isize)).is_null()
                        {
                            marker = *(*fm).argv.offset(1 as ::core::ffi::c_int as isize);
                        }
                    }
                }
                112 => {
                    if !((*fm).argc < 1 as ::core::ffi::c_int) {
                        width = strtonum(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            -FORMAT_MAX_WIDTH as ::core::ffi::c_longlong,
                            FORMAT_MAX_WIDTH as ::core::ffi::c_longlong,
                            &raw mut errstr,
                        ) as ::core::ffi::c_int;
                        if !errstr.is_null() {
                            width = 0 as ::core::ffi::c_int;
                        }
                    }
                }
                65 => {
                    modifiers = (modifiers as ::core::ffi::c_ulonglong | FORMAT_CYCLE) as uint64_t;
                    if !((*fm).argc < 1 as ::core::ffi::c_int) {
                        cycle_count = strtonum(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            1 as ::core::ffi::c_longlong,
                            100 as ::core::ffi::c_longlong,
                            &raw mut errstr,
                        ) as u_int;
                        if !errstr.is_null() {
                            cycle_count = 1 as u_int;
                        }
                    }
                }
                119 => {
                    modifiers |= FORMAT_WIDTH as uint64_t;
                }
                101 => {
                    if !((*fm).argc < 1 as ::core::ffi::c_int
                        || (*fm).argc > 3 as ::core::ffi::c_int)
                    {
                        mexp = fm;
                    }
                }
                108 => {
                    modifiers |= FORMAT_LITERAL as uint64_t;
                }
                97 => {
                    modifiers |= FORMAT_CHARACTER as uint64_t;
                }
                98 => {
                    modifiers |= FORMAT_BASENAME as uint64_t;
                }
                99 => {
                    modifiers |= FORMAT_COLOUR as uint64_t;
                    if !((*fm).argc < 1 as ::core::ffi::c_int) {
                        if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'f' as i32,
                        )
                        .is_null()
                        {
                            modifiers |= FORMAT_COLOUR_ESC_FG as uint64_t;
                        }
                        if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'b' as i32,
                        )
                        .is_null()
                        {
                            modifiers |= FORMAT_COLOUR_ESC_BG as uint64_t;
                        }
                    }
                }
                100 => {
                    modifiers |= FORMAT_DIRNAME as uint64_t;
                }
                110 => {
                    modifiers |= FORMAT_LENGTH as uint64_t;
                }
                73 => {
                    if !((*fm).argc < 1 as ::core::ffi::c_int) {
                        if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'f' as i32,
                        )
                        .is_null()
                        {
                            modifiers |= FORMAT_CLIENT_TERMFEAT as uint64_t;
                        }
                        if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'c' as i32,
                        )
                        .is_null()
                        {
                            modifiers |= FORMAT_CLIENT_TERMCAP as uint64_t;
                        }
                        if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'e' as i32,
                        )
                        .is_null()
                        {
                            modifiers |= FORMAT_CLIENT_ENVIRON as uint64_t;
                        }
                    }
                }
                116 => {
                    modifiers |= FORMAT_TIMESTRING as uint64_t;
                    if !((*fm).argc < 1 as ::core::ffi::c_int) {
                        if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'p' as i32,
                        )
                        .is_null()
                        {
                            modifiers |= FORMAT_PRETTY as uint64_t;
                        } else if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'r' as i32,
                        )
                        .is_null()
                        {
                            modifiers |= FORMAT_RELATIVE as uint64_t;
                        } else if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'd' as i32,
                        )
                        .is_null()
                        {
                            modifiers = (modifiers as ::core::ffi::c_ulonglong | FORMAT_DIFFERENCE)
                                as uint64_t;
                        } else if (*fm).argc >= 2 as ::core::ffi::c_int
                            && !strchr(
                                *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                                'f' as i32,
                            )
                            .is_null()
                        {
                            free(time_format as *mut ::core::ffi::c_void);
                            time_format = format_strip(
                                es,
                                *(*fm).argv.offset(1 as ::core::ffi::c_int as isize),
                            );
                        }
                    }
                }
                113 => {
                    if (*fm).argc < 1 as ::core::ffi::c_int {
                        modifiers |= FORMAT_QUOTE_SHELL as uint64_t;
                    } else if !strchr(
                        *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                        's' as i32,
                    )
                    .is_null()
                    {
                        modifiers |= FORMAT_QUOTE_SHELL_SQ as uint64_t;
                    } else if !strchr(
                        *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                        'e' as i32,
                    )
                    .is_null()
                        || !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'h' as i32,
                        )
                        .is_null()
                    {
                        modifiers |= FORMAT_QUOTE_STYLE as uint64_t;
                    } else if !strchr(
                        *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                        'a' as i32,
                    )
                    .is_null()
                    {
                        modifiers |= FORMAT_QUOTE_ARGUMENTS as uint64_t;
                    }
                }
                69 => {
                    modifiers |= FORMAT_EXPAND as uint64_t;
                }
                84 => {
                    modifiers |= FORMAT_EXPANDTIME as uint64_t;
                }
                78 => {
                    if (*fm).argc < 1 as ::core::ffi::c_int
                        || !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'w' as i32,
                        )
                        .is_null()
                    {
                        modifiers |= FORMAT_WINDOW_NAME as uint64_t;
                    } else if !strchr(
                        *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                        's' as i32,
                    )
                    .is_null()
                    {
                        modifiers |= FORMAT_SESSION_NAME as uint64_t;
                    }
                }
                83 => {
                    modifiers |= FORMAT_SESSIONS as uint64_t;
                    if (*fm).argc < 1 as ::core::ffi::c_int {
                        (*sc).order = SORT_INDEX;
                        (*sc).reversed = 0 as ::core::ffi::c_int;
                    } else {
                        if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'i' as i32,
                        )
                        .is_null()
                        {
                            (*sc).order = SORT_INDEX;
                        } else if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'n' as i32,
                        )
                        .is_null()
                        {
                            (*sc).order = SORT_NAME;
                        } else if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            't' as i32,
                        )
                        .is_null()
                        {
                            (*sc).order = SORT_ACTIVITY;
                        } else {
                            (*sc).order = SORT_INDEX;
                        }
                        if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'r' as i32,
                        )
                        .is_null()
                        {
                            (*sc).reversed = 1 as ::core::ffi::c_int;
                        } else {
                            (*sc).reversed = 0 as ::core::ffi::c_int;
                        }
                    }
                }
                87 => {
                    modifiers |= FORMAT_WINDOWS as uint64_t;
                    if (*fm).argc < 1 as ::core::ffi::c_int {
                        (*sc).order = SORT_ORDER;
                        (*sc).reversed = 0 as ::core::ffi::c_int;
                    } else {
                        if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'i' as i32,
                        )
                        .is_null()
                        {
                            (*sc).order = SORT_ORDER;
                        } else if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'n' as i32,
                        )
                        .is_null()
                        {
                            (*sc).order = SORT_NAME;
                        } else if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            't' as i32,
                        )
                        .is_null()
                        {
                            (*sc).order = SORT_ACTIVITY;
                        } else {
                            (*sc).order = SORT_ORDER;
                        }
                        if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'r' as i32,
                        )
                        .is_null()
                        {
                            (*sc).reversed = 1 as ::core::ffi::c_int;
                        } else {
                            (*sc).reversed = 0 as ::core::ffi::c_int;
                        }
                    }
                }
                80 => {
                    modifiers |= FORMAT_PANES as uint64_t;
                    (*sc).order = SORT_CREATION;
                    if (*fm).argc < 1 as ::core::ffi::c_int {
                        (*sc).reversed = 0 as ::core::ffi::c_int;
                    } else {
                        if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'i' as i32,
                        )
                        .is_null()
                        {
                            (*sc).order = SORT_INDEX;
                        } else if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'z' as i32,
                        )
                        .is_null()
                        {
                            (*sc).order = SORT_Z;
                        } else {
                            (*sc).order = SORT_CREATION;
                        }
                        if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'r' as i32,
                        )
                        .is_null()
                        {
                            (*sc).reversed = 1 as ::core::ffi::c_int;
                        } else {
                            (*sc).reversed = 0 as ::core::ffi::c_int;
                        }
                    }
                }
                79 => {
                    modifiers |= FORMAT_OPTIONS as uint64_t;
                    if (*fm).argc == 1 as ::core::ffi::c_int {
                        loop_flags = *(*fm).argv.offset(0 as ::core::ffi::c_int as isize);
                    }
                }
                86 => {
                    modifiers =
                        (modifiers as ::core::ffi::c_ulonglong | FORMAT_ENVIRON) as uint64_t;
                    if (*fm).argc == 1 as ::core::ffi::c_int {
                        loop_flags = *(*fm).argv.offset(0 as ::core::ffi::c_int as isize);
                    }
                }
                76 => {
                    modifiers |= FORMAT_CLIENTS as uint64_t;
                    if (*fm).argc < 1 as ::core::ffi::c_int {
                        (*sc).order = SORT_ORDER;
                        (*sc).reversed = 0 as ::core::ffi::c_int;
                    } else {
                        if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'i' as i32,
                        )
                        .is_null()
                        {
                            (*sc).order = SORT_ORDER;
                        } else if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'n' as i32,
                        )
                        .is_null()
                        {
                            (*sc).order = SORT_NAME;
                        } else if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            't' as i32,
                        )
                        .is_null()
                        {
                            (*sc).order = SORT_ACTIVITY;
                        } else {
                            (*sc).order = SORT_ORDER;
                        }
                        if !strchr(
                            *(*fm).argv.offset(0 as ::core::ffi::c_int as isize),
                            'r' as i32,
                        )
                        .is_null()
                        {
                            (*sc).reversed = 1 as ::core::ffi::c_int;
                        } else {
                            (*sc).reversed = 0 as ::core::ffi::c_int;
                        }
                    }
                }
                82 => {
                    modifiers |= FORMAT_REPEAT as uint64_t;
                }
                _ => {}
            }
        } else if (*fm).size == 2 as u_int {
            if strcmp(
                &raw mut (*fm).modifier as *mut ::core::ffi::c_char,
                b"||\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
                || strcmp(
                    &raw mut (*fm).modifier as *mut ::core::ffi::c_char,
                    b"&&\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                bool_op_n = fm;
            } else if strcmp(
                &raw mut (*fm).modifier as *mut ::core::ffi::c_char,
                b"!!\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                modifiers |= FORMAT_NOT_NOT as uint64_t;
            } else if strcmp(
                &raw mut (*fm).modifier as *mut ::core::ffi::c_char,
                b"==\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
                || strcmp(
                    &raw mut (*fm).modifier as *mut ::core::ffi::c_char,
                    b"!=\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                || strcmp(
                    &raw mut (*fm).modifier as *mut ::core::ffi::c_char,
                    b">=\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                || strcmp(
                    &raw mut (*fm).modifier as *mut ::core::ffi::c_char,
                    b"<=\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                cmp = fm;
            }
        }
        i = i.wrapping_add(1);
    }
    if modifiers & FORMAT_CLIENT_TERMCAP as uint64_t != 0
        || modifiers & FORMAT_CLIENT_TERMFEAT as uint64_t != 0
        || modifiers & FORMAT_CLIENT_ENVIRON as uint64_t != 0
    {
        if (*ft).c.is_null()
            || (*(*ft).c).tty.term.is_null()
            || (*(*ft).c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0
        {
            value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        } else {
            if modifiers & FORMAT_CLIENT_TERMCAP as uint64_t != 0 {
                if tty_term_has_name((*(*ft).c).tty.term, copy) != 0 {
                    value = xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
                } else {
                    value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
                }
            }
            if modifiers & FORMAT_CLIENT_TERMFEAT as uint64_t != 0 {
                if tty_feature_present((*(*ft).c).tty.term, copy) != 0 {
                    value = xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
                } else {
                    value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
                }
            }
            if modifiers & FORMAT_CLIENT_ENVIRON as uint64_t != 0 {
                envent = environ_find((*(*ft).c).environ, copy);
                if !envent.is_null() && !(*envent).value.is_null() {
                    value = xstrdup((*envent).value);
                } else {
                    value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                }
            }
        }
    } else if modifiers as ::core::ffi::c_ulonglong & FORMAT_CYCLE != 0 {
        value = format_cycle(es, copy, cycle_count);
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"cycle '%s' is: %s\0" as *const u8 as *const ::core::ffi::c_char,
            copy,
            value,
        );
    } else if modifiers & FORMAT_LITERAL as uint64_t != 0 {
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"literal string is '%s'\0" as *const u8 as *const ::core::ffi::c_char,
            copy,
        );
        value = format_unescape(es, copy, strlen(copy));
    } else if modifiers & FORMAT_CHARACTER as uint64_t != 0 {
        new = format_expand1(es, copy);
        c = strtonum(
            new,
            32 as ::core::ffi::c_longlong,
            126 as ::core::ffi::c_longlong,
            &raw mut errstr,
        ) as ::core::ffi::c_int;
        if !errstr.is_null() {
            value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        } else {
            xasprintf(
                &raw mut value,
                b"%c\0" as *const u8 as *const ::core::ffi::c_char,
                c,
            );
        }
        free(new as *mut ::core::ffi::c_void);
    } else if modifiers & FORMAT_COLOUR as uint64_t != 0 {
        new = format_expand1(es, copy);
        if modifiers & (FORMAT_COLOUR_ESC_FG | FORMAT_COLOUR_ESC_BG) as uint64_t != 0 {
            if strcasecmp(new, b"none\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                value = xstrdup(b"\x1B[0m\0" as *const u8 as *const ::core::ffi::c_char);
            } else {
                c = colour_fromstring(new);
                if c == -(1 as ::core::ffi::c_int) {
                    value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                } else {
                    if modifiers & FORMAT_COLOUR_ESC_BG as uint64_t != 0 {
                        cp = colour_toescape((*ft).c, c, 1 as ::core::ffi::c_int);
                    } else {
                        cp = colour_toescape((*ft).c, c, 0 as ::core::ffi::c_int);
                    }
                    if cp.is_null() {
                        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                    } else {
                        value = xstrdup(cp);
                    }
                }
            }
        } else {
            c = colour_fromstring(new);
            if c == -(1 as ::core::ffi::c_int) || {
                c = colour_force_rgb(c);
                c == -(1 as ::core::ffi::c_int)
            } {
                value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
            } else {
                xasprintf(
                    &raw mut value,
                    b"%06x\0" as *const u8 as *const ::core::ffi::c_char,
                    c & 0xffffff as ::core::ffi::c_int,
                );
            }
        }
        free(new as *mut ::core::ffi::c_void);
    } else {
        if modifiers & FORMAT_SESSIONS as uint64_t != 0 {
            value = format_loop_sessions(es, copy);
            if value.is_null() {
                current_block = 6506207624831006569;
            } else {
                current_block = 1803726662341650892;
            }
        } else if modifiers & FORMAT_WINDOWS as uint64_t != 0 {
            value = format_loop_windows(es, copy);
            if value.is_null() {
                current_block = 6506207624831006569;
            } else {
                current_block = 1803726662341650892;
            }
        } else if modifiers & FORMAT_PANES as uint64_t != 0 {
            value = format_loop_panes(es, copy);
            if value.is_null() {
                current_block = 6506207624831006569;
            } else {
                current_block = 1803726662341650892;
            }
        } else if modifiers & FORMAT_CLIENTS as uint64_t != 0 {
            value = format_loop_clients(es, copy);
            if value.is_null() {
                current_block = 6506207624831006569;
            } else {
                current_block = 1803726662341650892;
            }
        } else if modifiers & FORMAT_OPTIONS as uint64_t != 0 {
            value = format_loop_options(es, copy, loop_flags);
            if value.is_null() {
                current_block = 6506207624831006569;
            } else {
                current_block = 1803726662341650892;
            }
        } else if modifiers as ::core::ffi::c_ulonglong & FORMAT_ENVIRON != 0 {
            value = format_loop_environ(es, copy, loop_flags);
            if value.is_null() {
                current_block = 6506207624831006569;
            } else {
                current_block = 1803726662341650892;
            }
        } else if modifiers & FORMAT_WINDOW_NAME as uint64_t != 0 {
            value = format_window_name(es, copy);
            if value.is_null() {
                current_block = 6506207624831006569;
            } else {
                current_block = 1803726662341650892;
            }
        } else if modifiers & FORMAT_SESSION_NAME as uint64_t != 0 {
            value = format_session_name(es, copy);
            if value.is_null() {
                current_block = 6506207624831006569;
            } else {
                current_block = 1803726662341650892;
            }
        } else if !search.is_null() {
            new = format_expand1(es, copy);
            if wp.is_null() {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"search '%s' but no pane\0" as *const u8 as *const ::core::ffi::c_char,
                    new,
                );
                value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
            } else {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"search '%s' pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
                    new,
                    (*wp).id,
                );
                value = format_search(search, wp, new);
            }
            free(new as *mut ::core::ffi::c_void);
            current_block = 1803726662341650892;
        } else if modifiers & FORMAT_REPEAT as uint64_t != 0 {
            if format_choose(
                es,
                copy,
                &raw mut left,
                &raw mut right,
                1 as ::core::ffi::c_int,
            ) != 0 as ::core::ffi::c_int
            {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"repeat syntax error: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    copy,
                );
                current_block = 6506207624831006569;
            } else {
                nrep = strtonum(
                    right,
                    1 as ::core::ffi::c_longlong,
                    FORMAT_MAX_REPEAT as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                    current_block = 6055351187523413397;
                } else {
                    value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                    i = 0 as u_int;
                    loop {
                        if !(i < nrep) {
                            current_block = 6055351187523413397;
                            break;
                        }
                        if format_check_time(es, &raw mut check) == 0 {
                            free(right as *mut ::core::ffi::c_void);
                            free(left as *mut ::core::ffi::c_void);
                            free(value as *mut ::core::ffi::c_void);
                            current_block = 6506207624831006569;
                            break;
                        } else {
                            xasprintf(
                                &raw mut new,
                                b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                                value,
                                left,
                            );
                            free(value as *mut ::core::ffi::c_void);
                            value = new;
                            i = i.wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    6506207624831006569 => {}
                    _ => {
                        free(right as *mut ::core::ffi::c_void);
                        free(left as *mut ::core::ffi::c_void);
                        current_block = 1803726662341650892;
                    }
                }
            }
        } else if modifiers & FORMAT_NOT as uint64_t != 0 {
            value = format_bool_op_1(es, copy, 1 as ::core::ffi::c_int);
            current_block = 1803726662341650892;
        } else if modifiers & FORMAT_NOT_NOT as uint64_t != 0 {
            value = format_bool_op_1(es, copy, 0 as ::core::ffi::c_int);
            current_block = 1803726662341650892;
        } else if !bool_op_n.is_null() {
            if strcmp(
                &raw mut (*bool_op_n).modifier as *mut ::core::ffi::c_char,
                b"||\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                value = format_bool_op_n(es, copy, 0 as ::core::ffi::c_int);
            } else if strcmp(
                &raw mut (*bool_op_n).modifier as *mut ::core::ffi::c_char,
                b"&&\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                value = format_bool_op_n(es, copy, 1 as ::core::ffi::c_int);
            }
            current_block = 1803726662341650892;
        } else if !cmp.is_null() {
            if format_choose(
                es,
                copy,
                &raw mut left,
                &raw mut right,
                1 as ::core::ffi::c_int,
            ) != 0 as ::core::ffi::c_int
            {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"compare %s syntax error: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    copy,
                );
                current_block = 6506207624831006569;
            } else {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"compare %s left is: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    left,
                );
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"compare %s right is: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    right,
                );
                if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b"==\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left, right) == 0 as ::core::ffi::c_int {
                        value = xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
                    } else {
                        value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b"!=\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left, right) != 0 as ::core::ffi::c_int {
                        value = xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
                    } else {
                        value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b"<\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left, right) < 0 as ::core::ffi::c_int {
                        value = xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
                    } else {
                        value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b">\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left, right) > 0 as ::core::ffi::c_int {
                        value = xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
                    } else {
                        value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b"<=\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left, right) <= 0 as ::core::ffi::c_int {
                        value = xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
                    } else {
                        value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b">=\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    if strcmp(left, right) >= 0 as ::core::ffi::c_int {
                        value = xstrdup(b"1\0" as *const u8 as *const ::core::ffi::c_char);
                    } else {
                        value = xstrdup(b"0\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                } else if strcmp(
                    &raw mut (*cmp).modifier as *mut ::core::ffi::c_char,
                    b"m\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    value = format_match(cmp, left, right);
                }
                free(right as *mut ::core::ffi::c_void);
                free(left as *mut ::core::ffi::c_void);
                current_block = 1803726662341650892;
            }
        } else {
            if *copy as ::core::ffi::c_int == '?' as i32 {
                cp = copy.offset(1 as ::core::ffi::c_int as isize);
                loop {
                    cp2 = format_skip1(es, cp, b",\0" as *const u8 as *const ::core::ffi::c_char);
                    if cp2.is_null() {
                        format_log1(
                            es,
                            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                            b"no condition matched in '%s'; using last arg\0" as *const u8
                                as *const ::core::ffi::c_char,
                            copy.offset(1 as ::core::ffi::c_int as isize),
                        );
                        value = format_expand1(es, cp);
                        break;
                    } else {
                        condition =
                            xstrndup(cp, cp2.offset_from(cp) as ::core::ffi::c_long as size_t);
                        format_log1(
                            es,
                            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                            b"condition is: %s\0" as *const u8 as *const ::core::ffi::c_char,
                            condition,
                        );
                        found = format_find(ft, condition, modifiers, time_format);
                        if found.is_null() {
                            found = format_expand1(es, condition);
                            if strcmp(found, condition) == 0 as ::core::ffi::c_int {
                                free(found as *mut ::core::ffi::c_void);
                                found = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                                format_log1(
                                    es,
                                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                                    b"condition '%s' not found; assuming false\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    condition,
                                );
                            }
                        } else {
                            format_log1(
                                es,
                                b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                                b"condition '%s' found: %s\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                condition,
                                found,
                            );
                        }
                        cp = cp2.offset(1 as ::core::ffi::c_int as isize);
                        cp2 =
                            format_skip1(es, cp, b",\0" as *const u8 as *const ::core::ffi::c_char);
                        if format_true(found) != 0 {
                            format_log1(
                                es,
                                b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                                b"condition '%s' is true\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                condition,
                            );
                            if cp2.is_null() {
                                value = format_expand1(es, cp);
                            } else {
                                right = xstrndup(
                                    cp,
                                    cp2.offset_from(cp) as ::core::ffi::c_long as size_t,
                                );
                                value = format_expand1(es, right);
                                free(right as *mut ::core::ffi::c_void);
                            }
                            free(condition as *mut ::core::ffi::c_void);
                            free(found as *mut ::core::ffi::c_void);
                            break;
                        } else {
                            format_log1(
                                es,
                                b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                                b"condition '%s' is false\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                condition,
                            );
                            free(condition as *mut ::core::ffi::c_void);
                            free(found as *mut ::core::ffi::c_void);
                            if cp2.is_null() {
                                format_log1(
                                    es,
                                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                                    b"no condition matched in '%s'; using empty string\0"
                                        as *const u8
                                        as *const ::core::ffi::c_char,
                                    copy.offset(1 as ::core::ffi::c_int as isize),
                                );
                                value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                                break;
                            } else {
                                cp = cp2.offset(1 as ::core::ffi::c_int as isize);
                            }
                        }
                    }
                }
            } else if !mexp.is_null() {
                value = format_replace_expression(mexp, es, copy);
                if value.is_null() {
                    value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                }
            } else if !strstr(copy, b"#{\0" as *const u8 as *const ::core::ffi::c_char).is_null() {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"expanding inner format '%s'\0" as *const u8 as *const ::core::ffi::c_char,
                    copy,
                );
                value = format_expand1(es, copy);
            } else {
                value = format_find(ft, copy, modifiers, time_format);
                if value.is_null() {
                    format_log1(
                        es,
                        b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                        b"format '%s' not found\0" as *const u8 as *const ::core::ffi::c_char,
                        copy,
                    );
                    value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                } else {
                    format_log1(
                        es,
                        b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                        b"format '%s' found: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        copy,
                        value,
                    );
                }
            }
            current_block = 1803726662341650892;
        }
        match current_block {
            1803726662341650892 => {}
            _ => {
                format_log1(
                    es,
                    b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
                    b"failed %s\0" as *const u8 as *const ::core::ffi::c_char,
                    copy0,
                );
                free(sub as *mut ::core::ffi::c_void);
                format_free_modifiers(list, count);
                free(copy0 as *mut ::core::ffi::c_void);
                free(time_format as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
        }
    }
    if modifiers & FORMAT_EXPAND as uint64_t != 0 {
        new = format_expand1(es, value);
        free(value as *mut ::core::ffi::c_void);
        value = new;
    } else if modifiers & FORMAT_EXPANDTIME as uint64_t != 0 {
        format_copy_state(&raw mut next, es, FORMAT_EXPAND_TIME);
        new = format_expand1(&raw mut next, value);
        free(value as *mut ::core::ffi::c_void);
        value = new;
    }
    i = 0 as u_int;
    while i < nsub {
        left = format_expand1(
            es,
            *(**sub.offset(i as isize))
                .argv
                .offset(0 as ::core::ffi::c_int as isize),
        );
        right = format_expand1(
            es,
            *(**sub.offset(i as isize))
                .argv
                .offset(1 as ::core::ffi::c_int as isize),
        );
        new = format_sub(*sub.offset(i as isize), value, left, right);
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"substitute '%s' to '%s': %s\0" as *const u8 as *const ::core::ffi::c_char,
            left,
            right,
            new,
        );
        free(value as *mut ::core::ffi::c_void);
        value = new;
        free(right as *mut ::core::ffi::c_void);
        free(left as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    if limit > 0 as ::core::ffi::c_int {
        new = format_trim_left(value, limit as u_int);
        if !marker.is_null() && strcmp(new, value) != 0 as ::core::ffi::c_int {
            free(value as *mut ::core::ffi::c_void);
            xasprintf(
                &raw mut value,
                b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                new,
                marker,
            );
            free(new as *mut ::core::ffi::c_void);
        } else {
            free(value as *mut ::core::ffi::c_void);
            value = new;
        }
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"applied length limit %d: %s\0" as *const u8 as *const ::core::ffi::c_char,
            limit,
            value,
        );
    } else if limit < 0 as ::core::ffi::c_int {
        new = format_trim_right(value, -limit as u_int);
        if !marker.is_null() && strcmp(new, value) != 0 as ::core::ffi::c_int {
            free(value as *mut ::core::ffi::c_void);
            xasprintf(
                &raw mut value,
                b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                marker,
                new,
            );
            free(new as *mut ::core::ffi::c_void);
        } else {
            free(value as *mut ::core::ffi::c_void);
            value = new;
        }
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"applied length limit %d: %s\0" as *const u8 as *const ::core::ffi::c_char,
            limit,
            value,
        );
    }
    if width > 0 as ::core::ffi::c_int {
        new = utf8_padcstr(value, width as u_int);
        free(value as *mut ::core::ffi::c_void);
        value = new;
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"applied padding width %d: %s\0" as *const u8 as *const ::core::ffi::c_char,
            width,
            value,
        );
    } else if width < 0 as ::core::ffi::c_int {
        new = utf8_rpadcstr(value, -width as u_int);
        free(value as *mut ::core::ffi::c_void);
        value = new;
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"applied padding width %d: %s\0" as *const u8 as *const ::core::ffi::c_char,
            width,
            value,
        );
    }
    if modifiers & FORMAT_LENGTH as uint64_t != 0 {
        xasprintf(
            &raw mut new,
            b"%zu\0" as *const u8 as *const ::core::ffi::c_char,
            strlen(value),
        );
        free(value as *mut ::core::ffi::c_void);
        value = new;
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"replacing with length: %s\0" as *const u8 as *const ::core::ffi::c_char,
            new,
        );
    }
    if modifiers & FORMAT_WIDTH as uint64_t != 0 {
        xasprintf(
            &raw mut new,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            format_width(value),
        );
        free(value as *mut ::core::ffi::c_void);
        value = new;
        format_log1(
            es,
            b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
            b"replacing with width: %s\0" as *const u8 as *const ::core::ffi::c_char,
            new,
        );
    }
    valuelen = strlen(value);
    while (*len).wrapping_sub(*off) < valuelen.wrapping_add(1 as size_t) {
        *buf = xreallocarray(*buf as *mut ::core::ffi::c_void, 2 as size_t, *len)
            as *mut ::core::ffi::c_char;
        *len = (*len).wrapping_mul(2 as size_t);
    }
    memcpy(
        (*buf).offset(*off as isize) as *mut ::core::ffi::c_void,
        value as *const ::core::ffi::c_void,
        valuelen,
    );
    *off = (*off).wrapping_add(valuelen);
    format_log1(
        es,
        b"format_replace\0" as *const u8 as *const ::core::ffi::c_char,
        b"replaced '%s' with '%s'\0" as *const u8 as *const ::core::ffi::c_char,
        copy0,
        value,
    );
    free(value as *mut ::core::ffi::c_void);
    free(sub as *mut ::core::ffi::c_void);
    format_free_modifiers(list, count);
    free(copy0 as *mut ::core::ffi::c_void);
    free(time_format as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn format_expand1(
    mut es: *mut format_expand_state,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut ft: *mut format_tree = (*es).ft;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ptr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut style_end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut off: size_t = 0;
    let mut len: size_t = 0;
    let mut n: size_t = 0;
    let mut outlen: size_t = 0;
    let mut ch: ::core::ffi::c_int = 0;
    let mut brackets: ::core::ffi::c_int = 0;
    let mut expanded: [::core::ffi::c_char; 8192] = [0; 8192];
    if fmt.is_null()
        || *fmt as ::core::ffi::c_int == '\0' as i32
        || format_check_time(es, ::core::ptr::null_mut::<u_int>()) == 0
    {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if (*es).loop_0 == FORMAT_LOOP_LIMIT as u_int {
        format_log1(
            es,
            b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
            b"reached loop limit (%u)\0" as *const u8 as *const ::core::ffi::c_char,
            FORMAT_LOOP_LIMIT,
        );
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    (*es).loop_0 = (*es).loop_0.wrapping_add(1);
    format_log1(
        es,
        b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
        b"expanding format: %s\0" as *const u8 as *const ::core::ffi::c_char,
        fmt,
    );
    if (*es).flags & FORMAT_EXPAND_TIME != 0 && !strchr(fmt, '%' as i32).is_null() {
        if (*es).time == 0 as time_t {
            (*es).time = time(::core::ptr::null_mut::<time_t>());
            localtime_r(&raw mut (*es).time, &raw mut (*es).tm);
        }
        if format_strftime(
            &raw mut expanded as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as size_t,
            fmt,
            &raw mut (*es).tm,
        ) == 0 as size_t
        {
            format_log1(
                es,
                b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                b"format is too long\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        }
        if format_logging(ft) != 0
            && strcmp(&raw mut expanded as *mut ::core::ffi::c_char, fmt) != 0 as ::core::ffi::c_int
        {
            format_log1(
                es,
                b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                b"after time expanded: %s\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut expanded as *mut ::core::ffi::c_char,
            );
        }
        fmt = &raw mut expanded as *mut ::core::ffi::c_char;
    }
    len = 64 as size_t;
    buf = xmalloc(len) as *mut ::core::ffi::c_char;
    off = 0 as size_t;
    while *fmt as ::core::ffi::c_int != '\0' as i32 {
        if *fmt as ::core::ffi::c_int != '#' as i32 {
            while len.wrapping_sub(off) < 2 as size_t {
                buf = xreallocarray(buf as *mut ::core::ffi::c_void, 2 as size_t, len)
                    as *mut ::core::ffi::c_char;
                len = len.wrapping_mul(2 as size_t);
            }
            let fresh10 = fmt;
            fmt = fmt.offset(1);
            let fresh11 = off;
            off = off.wrapping_add(1);
            *buf.offset(fresh11 as isize) = *fresh10;
        } else {
            fmt = fmt.offset(1);
            if *fmt as ::core::ffi::c_int == '\0' as i32 {
                break;
            }
            let fresh12 = fmt;
            fmt = fmt.offset(1);
            ch = *fresh12 as u_char as ::core::ffi::c_int;
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
                    name = xstrndup(fmt, n);
                    format_log1(
                        es,
                        b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                        b"found #(): %s\0" as *const u8 as *const ::core::ffi::c_char,
                        name,
                    );
                    if (*ft).flags & FORMAT_NOJOBS != 0 || (*es).flags & FORMAT_EXPAND_NOJOBS != 0 {
                        out = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                        format_log1(
                            es,
                            b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                            b"#() is disabled\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    } else {
                        out = format_job_get(es, name);
                        format_log1(
                            es,
                            b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                            b"#() result: %s\0" as *const u8 as *const ::core::ffi::c_char,
                            out,
                        );
                    }
                    free(name as *mut ::core::ffi::c_void);
                    outlen = strlen(out);
                    while len.wrapping_sub(off) < outlen.wrapping_add(1 as size_t) {
                        buf = xreallocarray(buf as *mut ::core::ffi::c_void, 2 as size_t, len)
                            as *mut ::core::ffi::c_char;
                        len = len.wrapping_mul(2 as size_t);
                    }
                    memcpy(
                        buf.offset(off as isize) as *mut ::core::ffi::c_void,
                        out as *const ::core::ffi::c_void,
                        outlen,
                    );
                    off = off.wrapping_add(outlen);
                    free(out as *mut ::core::ffi::c_void);
                    fmt = fmt.offset(n.wrapping_add(1 as size_t) as isize);
                    continue;
                }
                123 => {
                    ptr = format_skip1(
                        es,
                        (fmt as *mut ::core::ffi::c_char)
                            .offset(-(2 as ::core::ffi::c_int as isize)),
                        b"}\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                    if ptr.is_null() {
                        break;
                    }
                    n = ptr.offset_from(fmt) as ::core::ffi::c_long as size_t;
                    format_log1(
                        es,
                        b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                        b"found #{}: %.*s\0" as *const u8 as *const ::core::ffi::c_char,
                        n as ::core::ffi::c_int,
                        fmt,
                    );
                    if format_replace(es, fmt, n, &raw mut buf, &raw mut len, &raw mut off)
                        != 0 as ::core::ffi::c_int
                    {
                        break;
                    }
                    fmt = fmt.offset(n.wrapping_add(1 as size_t) as isize);
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
                            es,
                            fmt.offset(-(2 as ::core::ffi::c_int as isize)),
                            b"]\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                        format_log1(
                            es,
                            b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                            b"found #*%zu[\0" as *const u8 as *const ::core::ffi::c_char,
                            n,
                        );
                        while len.wrapping_sub(off) < n.wrapping_add(2 as size_t) {
                            buf = xreallocarray(buf as *mut ::core::ffi::c_void, 2 as size_t, len)
                                as *mut ::core::ffi::c_char;
                            len = len.wrapping_mul(2 as size_t);
                        }
                        memcpy(
                            buf.offset(off as isize) as *mut ::core::ffi::c_void,
                            fmt.offset(-(2 as ::core::ffi::c_int as isize))
                                as *const ::core::ffi::c_void,
                            n.wrapping_add(1 as size_t),
                        );
                        off = off.wrapping_add(n.wrapping_add(1 as size_t));
                        fmt = ptr.offset(1 as ::core::ffi::c_int as isize);
                        continue;
                    }
                }
                125 | 44 => {}
                _ => {
                    s = ::core::ptr::null::<::core::ffi::c_char>();
                    if fmt > style_end {
                        if ch >= 'A' as i32 && ch <= 'Z' as i32 {
                            s = format_upper[(ch - 'A' as i32) as usize];
                        } else if ch >= 'a' as i32 && ch <= 'z' as i32 {
                            s = format_lower[(ch - 'a' as i32) as usize];
                        }
                    }
                    if s.is_null() {
                        while len.wrapping_sub(off) < 3 as size_t {
                            buf = xreallocarray(buf as *mut ::core::ffi::c_void, 2 as size_t, len)
                                as *mut ::core::ffi::c_char;
                            len = len.wrapping_mul(2 as size_t);
                        }
                        let fresh14 = off;
                        off = off.wrapping_add(1);
                        *buf.offset(fresh14 as isize) = '#' as i32 as ::core::ffi::c_char;
                        let fresh15 = off;
                        off = off.wrapping_add(1);
                        *buf.offset(fresh15 as isize) = ch as ::core::ffi::c_char;
                        continue;
                    } else {
                        n = strlen(s);
                        format_log1(
                            es,
                            b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                            b"found #%c: %s\0" as *const u8 as *const ::core::ffi::c_char,
                            ch,
                            s,
                        );
                        if format_replace(es, s, n, &raw mut buf, &raw mut len, &raw mut off)
                            != 0 as ::core::ffi::c_int
                        {
                            break;
                        } else {
                            continue;
                        }
                    }
                }
            }
            format_log1(
                es,
                b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
                b"found #%c\0" as *const u8 as *const ::core::ffi::c_char,
                ch,
            );
            while len.wrapping_sub(off) < 2 as size_t {
                buf = xreallocarray(buf as *mut ::core::ffi::c_void, 2 as size_t, len)
                    as *mut ::core::ffi::c_char;
                len = len.wrapping_mul(2 as size_t);
            }
            let fresh13 = off;
            off = off.wrapping_add(1);
            *buf.offset(fresh13 as isize) = ch as ::core::ffi::c_char;
        }
    }
    *buf.offset(off as isize) = '\0' as i32 as ::core::ffi::c_char;
    format_log1(
        es,
        b"format_expand1\0" as *const u8 as *const ::core::ffi::c_char,
        b"result is: %s\0" as *const u8 as *const ::core::ffi::c_char,
        buf,
    );
    (*es).loop_0 = (*es).loop_0.wrapping_sub(1);
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn format_expand_time(
    mut ft: *mut format_tree,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut es: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    memset(
        &raw mut es as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<format_expand_state>() as size_t,
    );
    es.ft = ft;
    es.flags = FORMAT_EXPAND_TIME;
    es.start_time = get_timer();
    return format_expand1(&raw mut es, fmt);
}
#[no_mangle]
pub unsafe extern "C" fn format_expand(
    mut ft: *mut format_tree,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut es: format_expand_state = format_expand_state {
        ft: ::core::ptr::null_mut::<format_tree>(),
        loop_0: 0,
        start_time: 0,
        flags: 0,
        time: 0,
        tm: tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: 0,
            tm_mon: 0,
            tm_year: 0,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_gmtoff: 0,
            tm_zone: ::core::ptr::null::<::core::ffi::c_char>(),
        },
    };
    memset(
        &raw mut es as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<format_expand_state>() as size_t,
    );
    es.ft = ft;
    es.flags = 0 as ::core::ffi::c_int;
    es.start_time = get_timer();
    return format_expand1(&raw mut es, fmt);
}
#[no_mangle]
pub unsafe extern "C" fn format_single(
    mut item: *mut cmdq_item,
    mut fmt: *const ::core::ffi::c_char,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) -> *mut ::core::ffi::c_char {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ft = format_create_defaults(item, c, s, wl, wp);
    expanded = format_expand(ft, fmt);
    format_free(ft);
    return expanded;
}
#[no_mangle]
pub unsafe extern "C" fn format_single_from_state(
    mut item: *mut cmdq_item,
    mut fmt: *const ::core::ffi::c_char,
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
) -> *mut ::core::ffi::c_char {
    return format_single(item, fmt, c, (*fs).s, (*fs).wl, (*fs).wp);
}
#[no_mangle]
pub unsafe extern "C" fn format_single_from_target(
    mut item: *mut cmdq_item,
    mut fmt: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut tc: *mut client = cmdq_get_target_client(item);
    return format_single_from_state(item, fmt, tc, cmdq_get_target(item));
}
#[no_mangle]
pub unsafe extern "C" fn format_create_defaults(
    mut item: *mut cmdq_item,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) -> *mut format_tree {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    if !item.is_null() {
        ft = format_create(
            cmdq_get_client(item),
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
#[no_mangle]
pub unsafe extern "C" fn format_create_from_state(
    mut item: *mut cmdq_item,
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
) -> *mut format_tree {
    return format_create_defaults(item, c, (*fs).s, (*fs).wl, (*fs).wp);
}
#[no_mangle]
pub unsafe extern "C" fn format_create_from_target(mut item: *mut cmdq_item) -> *mut format_tree {
    let mut tc: *mut client = cmdq_get_target_client(item);
    return format_create_from_state(item, tc, cmdq_get_target(item));
}
#[no_mangle]
pub unsafe extern "C" fn format_defaults(
    mut ft: *mut format_tree,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    if !c.is_null() && !(*c).name.is_null() {
        log_debug(
            b"%s: c=%s\0" as *const u8 as *const ::core::ffi::c_char,
            b"format_defaults\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
    } else {
        log_debug(
            b"%s: c=none\0" as *const u8 as *const ::core::ffi::c_char,
            b"format_defaults\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !s.is_null() {
        log_debug(
            b"%s: s=$%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"format_defaults\0" as *const u8 as *const ::core::ffi::c_char,
            (*s).id,
        );
    } else {
        log_debug(
            b"%s: s=none\0" as *const u8 as *const ::core::ffi::c_char,
            b"format_defaults\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !wl.is_null() {
        log_debug(
            b"%s: wl=%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"format_defaults\0" as *const u8 as *const ::core::ffi::c_char,
            (*wl).idx,
        );
    } else {
        log_debug(
            b"%s: wl=none\0" as *const u8 as *const ::core::ffi::c_char,
            b"format_defaults\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !wp.is_null() {
        log_debug(
            b"%s: wp=%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"format_defaults\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
        );
    } else {
        log_debug(
            b"%s: wp=none\0" as *const u8 as *const ::core::ffi::c_char,
            b"format_defaults\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !c.is_null() && !s.is_null() && (*c).session != s {
        log_debug(
            b"%s: session does not match\0" as *const u8 as *const ::core::ffi::c_char,
            b"format_defaults\0" as *const u8 as *const ::core::ffi::c_char,
        );
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
        s = (*c).session;
    }
    if wl.is_null() && !s.is_null() {
        wl = (*s).curw;
    }
    if wp.is_null() && !wl.is_null() {
        wp = (*(*wl).window).active;
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
    pb = paste_get_top(::core::ptr::null_mut::<*mut ::core::ffi::c_char>());
    if !pb.is_null() {
        format_defaults_paste_buffer(ft, pb);
    }
}
unsafe extern "C" fn format_defaults_session(mut ft: *mut format_tree, mut s: *mut session) {
    (*ft).s = s;
}
unsafe extern "C" fn format_defaults_client(mut ft: *mut format_tree, mut c: *mut client) {
    if (*ft).s.is_null() {
        (*ft).s = (*c).session;
    }
    (*ft).c = c;
}
#[no_mangle]
pub unsafe extern "C" fn format_defaults_window(mut ft: *mut format_tree, mut w: *mut window) {
    (*ft).w = w;
}
unsafe extern "C" fn format_defaults_winlink(mut ft: *mut format_tree, mut wl: *mut winlink) {
    if (*ft).w.is_null() {
        format_defaults_window(ft, (*wl).window);
    }
    (*ft).wl = wl;
}
#[no_mangle]
pub unsafe extern "C" fn format_defaults_pane(mut ft: *mut format_tree, mut wp: *mut window_pane) {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    if (*ft).w.is_null() {
        format_defaults_window(ft, (*wp).window as *mut window);
    }
    (*ft).wp = wp;
    wme = (*wp).modes.tqh_first;
    if !wme.is_null() && (*(*wme).mode).formats.is_some() {
        (*(*wme).mode).formats.expect("non-null function pointer")(wme, ft);
    }
}
#[no_mangle]
pub unsafe extern "C" fn format_defaults_paste_buffer(
    mut ft: *mut format_tree,
    mut pb: *mut paste_buffer,
) {
    (*ft).pb = pb;
}
unsafe extern "C" fn format_is_word_separator(
    mut ws: *const ::core::ffi::c_char,
    mut gc: *const grid_cell,
) -> ::core::ffi::c_int {
    if utf8_cstrhas(ws, &raw const (*gc).data) != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
        return 1 as ::core::ffi::c_int;
    }
    return ((*gc).data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && *(&raw const (*gc).data.data as *const u_char) as ::core::ffi::c_int == ' ' as i32)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn format_grid_word(
    mut gd: *mut grid,
    mut x: u_int,
    mut y: u_int,
) -> *mut ::core::ffi::c_char {
    let mut gl: *const grid_line = ::core::ptr::null::<grid_line>();
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
    let mut ws: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ud: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut end: u_int = 0;
    let mut size: size_t = 0 as size_t;
    let mut found: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ws = options_get_string(
        global_s_options,
        b"word-separators\0" as *const u8 as *const ::core::ffi::c_char,
    );
    loop {
        grid_get_cell(gd, x, y, &raw mut gc);
        if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0
            && format_is_word_separator(ws, &raw mut gc) != 0
        {
            found = 1 as ::core::ffi::c_int;
            break;
        } else {
            if x == 0 as u_int {
                if y == 0 as u_int {
                    break;
                }
                gl = grid_peek_line(gd, y.wrapping_sub(1 as u_int));
                if !((*gl).flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED != 0 {
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
                if y == (*gd).hsize.wrapping_add((*gd).sy).wrapping_sub(1 as u_int) {
                    break;
                }
                gl = grid_peek_line(gd, y);
                if !((*gl).flags as ::core::ffi::c_int) & GRID_LINE_WRAPPED != 0 {
                    break;
                }
                y = y.wrapping_add(1);
                x = 0 as u_int;
            } else {
                x = x.wrapping_add(1);
            }
        }
        found = 1 as ::core::ffi::c_int;
        grid_get_cell(gd, x, y, &raw mut gc);
        if gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
            continue;
        }
        if format_is_word_separator(ws, &raw mut gc) != 0 {
            break;
        }
        ud = xreallocarray(
            ud as *mut ::core::ffi::c_void,
            size.wrapping_add(2 as size_t),
            ::core::mem::size_of::<utf8_data>() as size_t,
        ) as *mut utf8_data;
        let fresh7 = size;
        size = size.wrapping_add(1);
        memcpy(
            ud.offset(fresh7 as isize) as *mut utf8_data as *mut ::core::ffi::c_void,
            &raw mut gc.data as *const ::core::ffi::c_void,
            ::core::mem::size_of::<utf8_data>() as size_t,
        );
    }
    if size != 0 as size_t {
        (*ud.offset(size as isize)).size = 0 as u_char;
        s = utf8_tocstr(ud);
        free(ud as *mut ::core::ffi::c_void);
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn format_grid_line(
    mut gd: *mut grid,
    mut y: u_int,
) -> *mut ::core::ffi::c_char {
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
    let mut ud: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut x: u_int = 0;
    let mut size: size_t = 0 as size_t;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    x = 0 as u_int;
    while x < grid_line_length(gd, y) {
        grid_get_cell(gd, x, y, &raw mut gc);
        if !(gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0) {
            ud = xreallocarray(
                ud as *mut ::core::ffi::c_void,
                size.wrapping_add(2 as size_t),
                ::core::mem::size_of::<utf8_data>() as size_t,
            ) as *mut utf8_data;
            if gc.flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                let fresh8 = size;
                size = size.wrapping_add(1);
                utf8_set(
                    ud.offset(fresh8 as isize) as *mut utf8_data,
                    '\t' as i32 as u_char,
                );
            } else {
                let fresh9 = size;
                size = size.wrapping_add(1);
                memcpy(
                    ud.offset(fresh9 as isize) as *mut utf8_data as *mut ::core::ffi::c_void,
                    &raw mut gc.data as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<utf8_data>() as size_t,
                );
            }
        }
        x = x.wrapping_add(1);
    }
    if size != 0 as size_t {
        (*ud.offset(size as isize)).size = 0 as u_char;
        s = utf8_tocstr(ud);
        free(ud as *mut ::core::ffi::c_void);
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn format_grid_hyperlink(
    mut gd: *mut grid,
    mut x: u_int,
    mut y: u_int,
    mut s: *mut screen,
) -> *mut ::core::ffi::c_char {
    let mut uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
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
        grid_get_cell(gd, x, y, &raw mut gc);
        if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
            break;
        }
        if x == 0 as u_int {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        x = x.wrapping_sub(1);
    }
    if (*s).hyperlinks.is_null() || gc.link == 0 as u_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if hyperlinks_get(
        (*s).hyperlinks,
        gc.link,
        &raw mut uri,
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
    ) == 0
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return xstrdup(uri);
}

pub const FORMAT_TYPE_PANE: format_type = 3;

pub const FORMAT_TYPE_WINDOW: format_type = 2;

pub const FORMAT_TYPE_SESSION: format_type = 1;

pub const FORMAT_TYPE_UNKNOWN: format_type = 0;

pub const FORMAT_TABLE_TIME: format_table_type = 1;

pub const FORMAT_TABLE_STRING: format_table_type = 0;

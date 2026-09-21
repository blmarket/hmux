use crate::src::arguments::args_escape;
use crate::src::cfg::{cfg_files, cfg_nfiles};
use crate::src::cmd::{cmd_free_argv, cmd_mouse_at, cmd_mouse_pane, cmd_stringify_argv};
use crate::src::cmd_queue::{
    cmdq_get_client, cmdq_get_event, cmdq_get_target, cmdq_get_target_client, cmdq_merge_formats,
    cmdq_print,
};
use crate::src::colour::{
    colour_force_rgb, colour_format, colour_format_escape_for_client, colour_parse_cstr,
};
use crate::src::compat::strtonum::strtonum;
use crate::src::environ::{environ_find, environ_first, environ_next};
use crate::src::ffi::libc::{
    __ctype_b_loc, __xpg_basename, ctime_r, dirname, fnmatch, free, gethostname, getpid, getpwuid,
    getuid, localtime_r, memcmp, memcpy, memset, regcomp, regexec, regfree, strcasecmp, strchr,
    strcmp, strcspn, strftime, strlcat, strlen, strstr, strtod, time,
};
use crate::src::ffi::libm::{fabs, fmod};
use crate::src::format_draw::{format_trim_left, format_trim_right, format_width};
use crate::src::fuzzy::fuzzy_match;
use crate::src::grid::{grid_get_cell, grid_get_line, grid_line_length, grid_peek_line};
use crate::src::grid_view::grid_view_get_cell;
use crate::src::hyperlinks::hyperlinks_get;
use crate::src::job::{job_free, job_get_data, job_get_event, job_run};
use crate::src::layout::layout_add_horizontal_border;
use crate::src::layout_custom::layout_dump;
use crate::src::log::{fatalx, log_debug, log_get_level};
use crate::src::names::parse_window_name;
pub use crate::src::options::options_table_entry;
use crate::src::options::{
    options_array_first, options_array_item_key, options_array_next, options_first,
    options_get_number, options_get_string, options_is_array, options_name, options_next,
    options_parse_get, options_to_string,
};
use crate::src::osdep_linux::{osdep_get_cwd, osdep_get_name};
use crate::src::paste::{
    paste_buffer_created, paste_buffer_data, paste_buffer_name, paste_get_top, paste_make_sample,
};
use crate::src::proc::proc_get_peer_uid;
use crate::src::reactor::{
    evbuffer_add, evbuffer_add_printf, evbuffer_free, evbuffer_get_length, evbuffer_new,
    evbuffer_pullup, evbuffer_readline, event_add, event_initialized, event_pending, event_set,
};
use crate::src::regsub::regsub;
pub use crate::src::server::clients;
use crate::src::server::{marked_pane, server_check_marked};
use crate::src::server_client::{
    server_client_get_cwd, server_client_get_flags, server_client_get_key_table,
    server_client_unref,
};
use crate::src::server_fn::server_status_client;
use crate::src::session::{
    next_session_id, session_alive, session_group_attached_count, session_group_contains,
    session_group_count, session_groups_RB_MINMAX, session_groups_RB_NEXT, sessions_RB_MINMAX,
    sessions_RB_NEXT,
};
pub use crate::src::session::{session_groups, sessions};
pub use crate::src::shared::session::{session_group, session_group_entry, session_group_sessions};
use crate::src::sort::{
    sort_get_clients, sort_get_panes_window, sort_get_sessions, sort_get_winlinks_session,
};
use crate::src::status::status_get_range;
use crate::src::tmux::{
    get_timer, getversion, global_environ, global_options, global_s_options, global_w_options,
    sig2name, socket_path, start_time,
};
use crate::src::tty::{tty_default_colours, tty_window_offset};
use crate::src::tty_features::{tty_feature_present, tty_get_features};
use crate::src::tty_term::{tty_term_has_name, tty_term_number};
use crate::src::utf8::{utf8_cstrhas, utf8_padcstr, utf8_rpadcstr, utf8_set, utf8_tocstr};
use crate::src::window::{
    window_count_panes, window_get_pane_status, window_pane_get_pane_status, window_pane_index,
    window_pane_is_floating, window_pane_mode, window_pane_printable_flags,
    window_pane_scrollbar_reserve, window_pane_search, window_pane_zindex, window_printable_flags,
    winlink_count, winlink_find_by_window, winlinks_RB_MINMAX, winlinks_RB_NEXT,
};
use crate::src::window_buffer::window_buffer_mode;
use crate::src::window_client::window_client_mode;
use crate::src::window_copy::{
    window_copy_get_hyperlink, window_copy_get_line, window_copy_get_word,
};
use crate::src::window_tree::window_tree_mode;
use crate::src::xmalloc::{
    xasprintf, xcalloc, xmalloc, xmemdup, xrealloc, xreallocarray, xsnprintf, xstrdup, xstrndup,
    xvasprintf,
};

pub use crate::src::shared::abi::NULL_0;
use crate::src::shared::abi::*;
pub use crate::src::shared::abi::{__compar_fn_t, __gid_t, __uid_t, ssize_t, uid_t};
pub use crate::src::shared::account::passwd;
pub use crate::src::shared::arguments::args;
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::client::{
    CLIENT_CONTROL, CLIENT_CONTROL_NEWLAYOUTS, CLIENT_DEAD, CLIENT_EXIT, CLIENT_READONLY,
    CLIENT_REDRAWSTATUS, CLIENT_SUSPENDED, CLIENT_UNATTACHEDFLAGS, CLIENT_UTF8,
};
use crate::src::shared::colour::*;
pub use crate::src::shared::command::{cmd_find_state, cmd_list, cmdq_item, cmdq_list, cmds};
pub use crate::src::shared::control::control_state;
pub use crate::src::shared::ctype::{
    _ISalnum, _ISalpha, _ISblank, _IScntrl, _ISdigit, _ISgraph, _ISlower, _ISprint, _ISpunct,
    _ISspace, _ISupper, _ISxdigit, ctype_code,
};
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::ENVIRON_HIDDEN;
pub use crate::src::shared::environment::{environ, environ_entry, environ_entry_entry};
pub use crate::src::shared::event::EV_TIMEOUT;
use crate::src::shared::event::*;
pub use crate::src::shared::format::{
    format_cb, format_entry, format_entry_entry, format_entry_tree, format_job, format_job_tree,
    format_tree, format_type,
};
pub use crate::src::shared::format::{
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
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
pub use crate::src::shared::job::JOB_NOWAIT;
pub use crate::src::shared::job::{job, job_complete_cb, job_free_cb, job_update_cb};
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::OPTIONS_TABLE_IS_HOOK;
use crate::src::shared::options::*;
pub use crate::src::shared::options::{options, options_array_item, options_entry};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
    PANE_CMDRUNNING, PANE_INPUTOFF, PANE_MINIMUM, PANE_SCROLLBARS_ALWAYS, PANE_STATUSDRAWN,
    PANE_STATUSREADY, PANE_STATUS_BOTTOM, PANE_STATUS_TOP, PANE_UNSEENCHANGES, PANE_ZOOMED,
};
pub use crate::src::shared::paste::{
    paste_buffer, paste_buffer_name_entry, paste_buffer_time_entry,
};
pub use crate::src::shared::posix_io::FNM_CASEFOLD;
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::regex::{
    __re_long_size_t, re_dfa_t, re_pattern_buffer, reg_syntax_t, regex_t, regmatch_t, regoff_t,
    REG_EXTENDED, REG_ICASE,
};
pub use crate::src::shared::screen::{
    screen, screen_sel, screen_titles, ALL_MOUSE_MODES, EXTENDED_KEY_MODES, MODE_BRACKETPASTE,
    MODE_CURSOR, MODE_CURSOR_BLINKING, MODE_CURSOR_BLINKING_SET, MODE_CURSOR_VERY_VISIBLE,
    MODE_FOCUSON, MODE_INSERT, MODE_KCURSOR, MODE_KEYS_EXTENDED, MODE_KEYS_EXTENDED_2,
    MODE_KKEYPAD, MODE_MOUSE_ALL, MODE_MOUSE_BUTTON, MODE_MOUSE_SGR, MODE_MOUSE_STANDARD,
    MODE_MOUSE_UTF8, MODE_ORIGIN, MODE_SYNC, MODE_THEME_UPDATES, MODE_WRAP,
};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::sort::sort_criteria;
use crate::src::shared::sort::*;
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::time::tm;
pub use crate::src::shared::tree::{RB_BLACK, RB_INF, RB_NEGINF, RB_RED};
use crate::src::shared::tty::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::tty::{TERM_256COLOURS, TERM_RGBCOLOURS, TTY_STARTED};
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
pub use crate::src::shared::window::{
    WINDOW_PANE_NO_MODE, WINDOW_SIZE_MANUAL, WINDOW_ZOOMED, WINLINK_ACTIVITY, WINLINK_ALERTFLAGS,
    WINLINK_BELL, WINLINK_SILENCE,
};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_14;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_13;

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
mod tree;
use tree::*;
pub use tree::{
    format_add, format_add_cb, format_add_tv, format_create, format_each, format_free,
    format_get_pane, format_log_debug, format_merge,
};
mod jobs;
use jobs::*;
pub use jobs::{format_lost_client, format_tidy_jobs};
mod callbacks;
use callbacks::*;
mod expression;
use expression::*;
pub use expression::{
    format_expand, format_expand_time, format_pretty_time, format_single, format_single_from_state,
    format_single_from_target, format_skip, format_true,
};

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

static mut sort_crit: sort_criteria = sort_criteria {
    order: SORT_ACTIVITY,
    reversed: 0,
    order_seq: ::core::ptr::null::<sort_order>() as *mut sort_order,
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

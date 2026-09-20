use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;
extern "C" {
    pub type event_base;
    pub type evbuffer;
    pub type bufferevent_ops;
    pub type args;
    pub type tmuxpeer;
    pub type hyperlinks;
    pub type screen_write_cline;
    pub type screen_sel;
    pub type screen_titles;
    pub type environ;
    pub type options;
    pub type menu_data;
    pub type window_pane_prompt;
    pub type prompt;
    pub type format_tree;
    pub type cmdq_item;
    pub type input_ctx;
    pub type spawn_editor_state;
    pub type cmds;
    pub type input_request;
    pub type redraw_scene;
    pub type tty_key;
    pub type tty_code;
    pub type format_job_tree;
    pub type control_state;
    pub type cmdq_list;
    fn fnmatch(
        __pattern: *const ::core::ffi::c_char,
        __name: *const ::core::ffi::c_char,
        __flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strlcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn strtonum(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn environ_find(_: *mut environ, _: *const ::core::ffi::c_char) -> *mut environ_entry;
    fn cmd_mouse_window(_: *mut mouse_event, _: *mut *mut session) -> *mut winlink;
    fn cmd_mouse_pane(
        _: *mut mouse_event,
        _: *mut *mut session,
        _: *mut *mut winlink,
    ) -> *mut window_pane;
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_event(_: *mut cmdq_item) -> *mut key_event;
    fn cmdq_get_current(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    static mut clients: clients;
    static mut marked_pane: cmd_find_state;
    fn server_check_marked() -> ::core::ffi::c_int;
    static mut all_window_panes: window_pane_tree;
    fn winlinks_RB_NEXT(_: *mut winlink) -> *mut winlink;
    fn winlinks_RB_MINMAX(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn window_pane_tree_RB_MINMAX(
        _: *mut window_pane_tree,
        _: ::core::ffi::c_int,
    ) -> *mut window_pane;
    fn window_pane_tree_RB_NEXT(_: *mut window_pane) -> *mut window_pane;
    fn winlink_find_by_index(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn winlink_next_by_number(
        _: *mut winlink,
        _: *mut session,
        _: ::core::ffi::c_int,
    ) -> *mut winlink;
    fn winlink_previous_by_number(
        _: *mut winlink,
        _: *mut session,
        _: ::core::ffi::c_int,
    ) -> *mut winlink;
    fn window_find_by_id_str(_: *const ::core::ffi::c_char) -> *mut window;
    fn window_find_string(_: *mut window, _: *const ::core::ffi::c_char) -> *mut window_pane;
    fn window_has_pane(_: *mut window, _: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_at_index(_: *mut window, _: u_int) -> *mut window_pane;
    fn window_pane_next_by_number(
        _: *mut window,
        _: *mut window_pane,
        _: u_int,
    ) -> *mut window_pane;
    fn window_pane_previous_by_number(
        _: *mut window,
        _: *mut window_pane,
        _: u_int,
    ) -> *mut window_pane;
    fn window_pane_find_by_id_str(_: *const ::core::ffi::c_char) -> *mut window_pane;
    fn window_pane_find_up(_: *mut window_pane) -> *mut window_pane;
    fn window_pane_find_down(_: *mut window_pane) -> *mut window_pane;
    fn window_pane_find_left(_: *mut window_pane) -> *mut window_pane;
    fn window_pane_find_right(_: *mut window_pane) -> *mut window_pane;
    static mut sessions: sessions;
    fn sessions_RB_NEXT(_: *mut session) -> *mut session;
    fn sessions_RB_MINMAX(_: *mut sessions, _: ::core::ffi::c_int) -> *mut session;
    fn session_alive(_: *mut session) -> ::core::ffi::c_int;
    fn session_find(_: *const ::core::ffi::c_char) -> *mut session;
    fn session_find_by_id_str(_: *const ::core::ffi::c_char) -> *mut session;
    fn session_has(_: *mut session, _: *mut window) -> ::core::ffi::c_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct termios {
    pub c_iflag: tcflag_t,
    pub c_oflag: tcflag_t,
    pub c_cflag: tcflag_t,
    pub c_lflag: tcflag_t,
    pub c_line: cc_t,
    pub c_cc: [cc_t; 32],
    pub c2rust_unnamed: C2RustUnnamed_0,
    pub c2rust_unnamed_0: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub __ospeed: speed_t,
    pub c_ospeed: speed_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_0 {
    pub __ispeed: speed_t,
    pub c_ispeed: speed_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct event {
    pub ev_evcallback: event_callback,
    pub ev_timeout_pos: C2RustUnnamed_6,
    pub ev_fd: ::core::ffi::c_int,
    pub ev_base: *mut event_base,
    pub ev_: C2RustUnnamed_1,
    pub ev_events: ::core::ffi::c_short,
    pub ev_res: ::core::ffi::c_short,
    pub ev_timeout: timeval,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_1 {
    pub ev_io: C2RustUnnamed_4,
    pub ev_signal: C2RustUnnamed_2,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_2 {
    pub ev_signal_next: C2RustUnnamed_3,
    pub ev_ncalls: ::core::ffi::c_short,
    pub ev_pncalls: *mut ::core::ffi::c_short,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_3 {
    pub le_next: *mut event,
    pub le_prev: *mut *mut event,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_4 {
    pub ev_io_next: C2RustUnnamed_5,
    pub ev_timeout: timeval,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_5 {
    pub le_next: *mut event,
    pub le_prev: *mut *mut event,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_6 {
    pub ev_next_with_common_timeout: C2RustUnnamed_7,
    pub min_heap_idx: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_7 {
    pub tqe_next: *mut event,
    pub tqe_prev: *mut *mut event,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_callback {
    pub evcb_active_next: C2RustUnnamed_9,
    pub evcb_flags: ::core::ffi::c_short,
    pub evcb_pri: uint8_t,
    pub evcb_closure: uint8_t,
    pub evcb_cb_union: C2RustUnnamed_8,
    pub evcb_arg: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_8 {
    pub evcb_callback: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            ::core::ffi::c_short,
            *mut ::core::ffi::c_void,
        ) -> (),
    >,
    pub evcb_selfcb:
        Option<unsafe extern "C" fn(*mut event_callback, *mut ::core::ffi::c_void) -> ()>,
    pub evcb_evfinalize: Option<unsafe extern "C" fn(*mut event, *mut ::core::ffi::c_void) -> ()>,
    pub evcb_cbfinalize:
        Option<unsafe extern "C" fn(*mut event_callback, *mut ::core::ffi::c_void) -> ()>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_9 {
    pub tqe_next: *mut event_callback,
    pub tqe_prev: *mut *mut event_callback,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bufferevent {
    pub ev_base: *mut event_base,
    pub be_ops: *const bufferevent_ops,
    pub ev_read: event,
    pub ev_write: event,
    pub input: *mut evbuffer,
    pub output: *mut evbuffer,
    pub wm_read: event_watermark,
    pub wm_write: event_watermark,
    pub readcb: bufferevent_data_cb,
    pub writecb: bufferevent_data_cb,
    pub errorcb: bufferevent_event_cb,
    pub cbarg: *mut ::core::ffi::c_void,
    pub timeout_read: timeval,
    pub timeout_write: timeval,
    pub enabled: ::core::ffi::c_short,
}
pub type bufferevent_event_cb = Option<
    unsafe extern "C" fn(*mut bufferevent, ::core::ffi::c_short, *mut ::core::ffi::c_void) -> (),
>;
pub type bufferevent_data_cb =
    Option<unsafe extern "C" fn(*mut bufferevent, *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_watermark {
    pub low: size_t,
    pub high: size_t,
}
pub type msgtype = ::core::ffi::c_uint;
pub const MSG_WRITE_DONE: msgtype = 308;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct client {
    pub name: *const ::core::ffi::c_char,
    pub peer: *mut tmuxpeer,
    pub user: *const ::core::ffi::c_char,
    pub queue: *mut cmdq_list,
    pub control_state: *mut control_state,
    pub pause_age: u_int,
    pub pid: pid_t,
    pub fd: ::core::ffi::c_int,
    pub out_fd: ::core::ffi::c_int,
    pub event: event,
    pub retval: ::core::ffi::c_int,
    pub creation_time: timeval,
    pub activity_time: timeval,
    pub last_activity_time: timeval,
    pub environ: *mut environ,
    pub jobs: *mut format_job_tree,
    pub title: *mut ::core::ffi::c_char,
    pub path: *mut ::core::ffi::c_char,
    pub cwd: *const ::core::ffi::c_char,
    pub progress_bar: progress_bar,
    pub term_name: *mut ::core::ffi::c_char,
    pub term_features: ::core::ffi::c_int,
    pub term_nofeatures: ::core::ffi::c_int,
    pub term_type: *mut ::core::ffi::c_char,
    pub term_caps: *mut *mut ::core::ffi::c_char,
    pub term_ncaps: u_int,
    pub ttyname: *mut ::core::ffi::c_char,
    pub tty: tty,
    pub written: size_t,
    pub discarded: size_t,
    pub redraw: size_t,
    pub redraw_scene: *mut redraw_scene,
    pub repeat_timer: event,
    pub click_timer: event,
    pub click_loc: ::core::ffi::c_int,
    pub click_wp: ::core::ffi::c_int,
    pub exit_timer: event,
    pub click_button: u_int,
    pub click_event: mouse_event,
    pub status: status_line,
    pub cycle_timer: event,
    pub theme: client_theme,
    pub input_requests: input_requests,
    pub flags: uint64_t,
    pub exit_type: C2RustUnnamed_33,
    pub exit_msgtype: msgtype,
    pub exit_session: *mut ::core::ffi::c_char,
    pub exit_message: *mut ::core::ffi::c_char,
    pub keytable: *mut key_table,
    pub last_key: key_code,
    pub paste_time: time_t,
    pub message_ignore_keys: ::core::ffi::c_int,
    pub message_ignore_styles: ::core::ffi::c_int,
    pub message_string: *mut ::core::ffi::c_char,
    pub message_timer: event,
    pub prompt: *mut prompt,
    pub session: *mut session,
    pub last_session: *mut session,
    pub references: ::core::ffi::c_int,
    pub theme_colours: [::core::ffi::c_int; 10],
    pub pan_window: *mut ::core::ffi::c_void,
    pub pan_ox: u_int,
    pub pan_oy: u_int,
    pub overlay_check: overlay_check_cb,
    pub overlay_mode: overlay_mode_cb,
    pub overlay_draw: overlay_draw_cb,
    pub overlay_key: overlay_key_cb,
    pub overlay_free: overlay_free_cb,
    pub overlay_resize: overlay_resize_cb,
    pub overlay_data: *mut ::core::ffi::c_void,
    pub overlay_timer: event,
    pub files: client_files,
    pub source_file_depth: u_int,
    pub clipboard_panes: *mut u_int,
    pub clipboard_npanes: u_int,
    pub entry: C2RustUnnamed_10,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_10 {
    pub tqe_next: *mut client,
    pub tqe_prev: *mut *mut client,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct client_files {
    pub rbh_root: *mut client_file,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct client_file {
    pub c: *mut client,
    pub peer: *mut tmuxpeer,
    pub tree: *mut client_files,
    pub references: ::core::ffi::c_int,
    pub stream: ::core::ffi::c_int,
    pub path: *mut ::core::ffi::c_char,
    pub buffer: *mut evbuffer,
    pub event: *mut bufferevent,
    pub fd: ::core::ffi::c_int,
    pub error: ::core::ffi::c_int,
    pub closed: ::core::ffi::c_int,
    pub cb: client_file_cb,
    pub data: *mut ::core::ffi::c_void,
    pub entry: C2RustUnnamed_11,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_11 {
    pub rbe_left: *mut client_file,
    pub rbe_right: *mut client_file,
    pub rbe_parent: *mut client_file,
    pub rbe_color: ::core::ffi::c_int,
}
pub type client_file_cb = Option<
    unsafe extern "C" fn(
        *mut client,
        *const ::core::ffi::c_char,
        ::core::ffi::c_int,
        ::core::ffi::c_int,
        *mut evbuffer,
        *mut ::core::ffi::c_void,
    ) -> (),
>;
pub type overlay_resize_cb =
    Option<unsafe extern "C" fn(*mut client, *mut ::core::ffi::c_void) -> ()>;
pub type overlay_free_cb =
    Option<unsafe extern "C" fn(*mut client, *mut ::core::ffi::c_void) -> ()>;
pub type overlay_key_cb = Option<
    unsafe extern "C" fn(
        *mut client,
        *mut ::core::ffi::c_void,
        *mut key_event,
    ) -> ::core::ffi::c_int,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_event {
    pub client: *mut client,
    pub key: key_code,
    pub m: mouse_event,
    pub buf: *mut ::core::ffi::c_char,
    pub len: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mouse_event {
    pub valid: ::core::ffi::c_int,
    pub ignore: ::core::ffi::c_int,
    pub key: key_code,
    pub statusat: ::core::ffi::c_int,
    pub statuslines: u_int,
    pub x: u_int,
    pub y: u_int,
    pub b: u_int,
    pub lx: u_int,
    pub ly: u_int,
    pub lb: u_int,
    pub ox: u_int,
    pub oy: u_int,
    pub s: ::core::ffi::c_int,
    pub w: ::core::ffi::c_int,
    pub wp: ::core::ffi::c_int,
    pub sgr_type: u_int,
    pub sgr_b: u_int,
}
pub type overlay_draw_cb =
    Option<unsafe extern "C" fn(*mut client, *mut ::core::ffi::c_void) -> ()>;
pub type overlay_mode_cb = Option<
    unsafe extern "C" fn(
        *mut client,
        *mut ::core::ffi::c_void,
        *mut u_int,
        *mut u_int,
    ) -> *mut screen,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_12 {
    pub offset: u_int,
    pub data: C2RustUnnamed_13,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_13 {
    pub attr: u_char,
    pub fg: u_char,
    pub bg: u_char,
    pub data: u_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct screen {
    pub title: *mut ::core::ffi::c_char,
    pub path: *mut ::core::ffi::c_char,
    pub titles: *mut screen_titles,
    pub ntitles: u_int,
    pub grid: *mut grid,
    pub cx: u_int,
    pub cy: u_int,
    pub cstyle: screen_cursor_style,
    pub default_cstyle: screen_cursor_style,
    pub ccolour: ::core::ffi::c_int,
    pub default_ccolour: ::core::ffi::c_int,
    pub rupper: u_int,
    pub rlower: u_int,
    pub mode: ::core::ffi::c_int,
    pub default_mode: ::core::ffi::c_int,
    pub saved_cx: u_int,
    pub saved_cy: u_int,
    pub saved_grid: *mut grid,
    pub saved_cell: grid_cell,
    pub saved_flags: ::core::ffi::c_int,
    pub tabs: *mut bitstr_t,
    pub sel: *mut screen_sel,
    pub write_list: *mut screen_write_cline,
    pub hyperlinks: *mut hyperlinks,
    pub progress_bar: progress_bar,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct progress_bar {
    pub state: progress_bar_state,
    pub progress: ::core::ffi::c_int,
}
pub type progress_bar_state = ::core::ffi::c_uint;
pub const PROGRESS_BAR_PAUSED: progress_bar_state = 4;
pub const PROGRESS_BAR_INDETERMINATE: progress_bar_state = 3;
pub const PROGRESS_BAR_ERROR: progress_bar_state = 2;
pub const PROGRESS_BAR_NORMAL: progress_bar_state = 1;
pub const PROGRESS_BAR_HIDDEN: progress_bar_state = 0;
pub type screen_cursor_style = ::core::ffi::c_uint;
pub const SCREEN_CURSOR_BAR: screen_cursor_style = 3;
pub const SCREEN_CURSOR_UNDERLINE: screen_cursor_style = 2;
pub const SCREEN_CURSOR_BLOCK: screen_cursor_style = 1;
pub const SCREEN_CURSOR_DEFAULT: screen_cursor_style = 0;
pub type overlay_check_cb = Option<
    unsafe extern "C" fn(
        *mut client,
        *mut ::core::ffi::c_void,
        u_int,
        u_int,
        u_int,
    ) -> *mut visible_ranges,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct visible_ranges {
    pub ranges: *mut visible_range,
    pub used: u_int,
    pub size: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct visible_range {
    pub px: u_int,
    pub nx: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct session {
    pub id: u_int,
    pub name: *mut ::core::ffi::c_char,
    pub cwd: *const ::core::ffi::c_char,
    pub creation_time: timeval,
    pub last_attached_time: timeval,
    pub activity_time: timeval,
    pub last_activity_time: timeval,
    pub lock_timer: event,
    pub curw: *mut winlink,
    pub lastw: winlink_stack,
    pub windows: winlinks,
    pub statusat: ::core::ffi::c_int,
    pub statuslines: u_int,
    pub options: *mut options,
    pub flags: ::core::ffi::c_int,
    pub attached: u_int,
    pub tio: *mut termios,
    pub environ: *mut environ,
    pub references: ::core::ffi::c_int,
    pub gentry: C2RustUnnamed_15,
    pub entry: C2RustUnnamed_14,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_14 {
    pub rbe_left: *mut session,
    pub rbe_right: *mut session,
    pub rbe_parent: *mut session,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_15 {
    pub tqe_next: *mut session,
    pub tqe_prev: *mut *mut session,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct winlinks {
    pub rbh_root: *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct winlink {
    pub idx: ::core::ffi::c_int,
    pub session: *mut session,
    pub window: *mut window,
    pub flags: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_18,
    pub wentry: C2RustUnnamed_17,
    pub sentry: C2RustUnnamed_16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_16 {
    pub tqe_next: *mut winlink,
    pub tqe_prev: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_17 {
    pub tqe_next: *mut winlink,
    pub tqe_prev: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_18 {
    pub rbe_left: *mut winlink,
    pub rbe_right: *mut winlink,
    pub rbe_parent: *mut winlink,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window {
    pub id: u_int,
    pub latest: *mut ::core::ffi::c_void,
    pub name: *mut ::core::ffi::c_char,
    pub name_event: event,
    pub name_time: timeval,
    pub alerts_timer: event,
    pub offset_timer: event,
    pub activity_time: timeval,
    pub creation_time: timeval,
    pub active: *mut window_pane,
    pub modal: *mut window_pane,
    pub modal_last: *mut window_pane,
    pub was_zoomed: *mut window_pane,
    pub last_panes: window_panes,
    pub z_index: window_panes,
    pub panes: window_panes,
    pub lastlayout: ::core::ffi::c_int,
    pub layout_root: *mut layout_cell,
    pub saved_layout_root: *mut layout_cell,
    pub old_layout: *mut ::core::ffi::c_char,
    pub sx: u_int,
    pub sy: u_int,
    pub manual_sx: u_int,
    pub manual_sy: u_int,
    pub xpixel: u_int,
    pub ypixel: u_int,
    pub new_sx: u_int,
    pub new_sy: u_int,
    pub new_xpixel: u_int,
    pub new_ypixel: u_int,
    pub redraw_scene_generation: uint64_t,
    pub menu: *mut menu_data,
    pub menu_last_px: u_int,
    pub menu_last_py: u_int,
    pub last_new_pane_x: u_int,
    pub last_new_pane_y: u_int,
    pub sb: ::core::ffi::c_int,
    pub sb_pos: ::core::ffi::c_int,
    pub inside_cell: grid_cell,
    pub outside_cell: grid_cell,
    pub flags: ::core::ffi::c_int,
    pub alerts_queued: ::core::ffi::c_int,
    pub alerts_entry: C2RustUnnamed_21,
    pub options: *mut options,
    pub references: u_int,
    pub winlinks: C2RustUnnamed_20,
    pub entry: C2RustUnnamed_19,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_19 {
    pub rbe_left: *mut window,
    pub rbe_right: *mut window,
    pub rbe_parent: *mut window,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_20 {
    pub tqh_first: *mut winlink,
    pub tqh_last: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_21 {
    pub tqe_next: *mut window,
    pub tqe_prev: *mut *mut window,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct layout_cell {
    pub type_0: layout_type,
    pub flags: ::core::ffi::c_int,
    pub parent: *mut layout_cell,
    pub g: layout_geometry,
    pub fg: layout_geometry,
    pub wp: *mut window_pane,
    pub cells: layout_cells,
    pub entry: C2RustUnnamed_22,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_22 {
    pub tqe_next: *mut layout_cell,
    pub tqe_prev: *mut *mut layout_cell,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct layout_cells {
    pub tqh_first: *mut layout_cell,
    pub tqh_last: *mut *mut layout_cell,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_pane {
    pub id: u_int,
    pub references: ::core::ffi::c_int,
    pub active_point: u_int,
    pub window: *mut window,
    pub options: *mut options,
    pub layout_cell: *mut layout_cell,
    pub saved_layout_cell: *mut layout_cell,
    pub sx: u_int,
    pub sy: u_int,
    pub xoff: ::core::ffi::c_int,
    pub yoff: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
    pub sync_dirty: *mut bitstr_t,
    pub sync_dirty_size: u_int,
    pub sb_slider_y: u_int,
    pub sb_slider_h: u_int,
    pub sb_auto_visible: ::core::ffi::c_int,
    pub sb_auto_hover: ::core::ffi::c_int,
    pub sb_auto_timer: event,
    pub argc: ::core::ffi::c_int,
    pub argv: *mut *mut ::core::ffi::c_char,
    pub shell: *mut ::core::ffi::c_char,
    pub cwd: *mut ::core::ffi::c_char,
    pub pid: pid_t,
    pub tty: [::core::ffi::c_char; 32],
    pub status: ::core::ffi::c_int,
    pub dead_time: timeval,
    pub wait_item: *mut cmdq_item,
    pub editor: *mut spawn_editor_state,
    pub output_generation: uint64_t,
    pub last_output_time: time_t,
    pub last_prompt_time: time_t,
    pub cmd_start_time: time_t,
    pub cmd_end_time: time_t,
    pub cmd_status: ::core::ffi::c_int,
    pub fd: ::core::ffi::c_int,
    pub event: *mut bufferevent,
    pub offset: window_pane_offset,
    pub base_offset: size_t,
    pub resize_queue: window_pane_resizes,
    pub resize_timer: event,
    pub sync_timer: event,
    pub ictx: *mut input_ctx,
    pub cached_gc: grid_cell,
    pub cached_active_gc: grid_cell,
    pub cached_dim: u_int,
    pub cached_active_dim: u_int,
    pub palette: colour_palette,
    pub last_theme: client_theme,
    pub border_status_line: style_line_entry,
    pub pipe_fd: ::core::ffi::c_int,
    pub pipe_pid: pid_t,
    pub pipe_event: *mut bufferevent,
    pub pipe_offset: window_pane_offset,
    pub screen: *mut screen,
    pub base: screen,
    pub status_screen: screen,
    pub modes: C2RustUnnamed_27,
    pub searchstr: *mut ::core::ffi::c_char,
    pub searchregex: ::core::ffi::c_int,
    pub prompt: *mut prompt,
    pub prompt_data: *mut window_pane_prompt,
    pub prompt_cx: u_int,
    pub border_gc_set: ::core::ffi::c_int,
    pub border_gc: grid_cell,
    pub active_border_gc_set: ::core::ffi::c_int,
    pub active_border_gc: grid_cell,
    pub control_bg: ::core::ffi::c_int,
    pub control_fg: ::core::ffi::c_int,
    pub scrollbar_style: style,
    pub r: visible_ranges,
    pub entry: C2RustUnnamed_26,
    pub sentry: C2RustUnnamed_25,
    pub zentry: C2RustUnnamed_24,
    pub tree_entry: C2RustUnnamed_23,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_23 {
    pub rbe_left: *mut window_pane,
    pub rbe_right: *mut window_pane,
    pub rbe_parent: *mut window_pane,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_24 {
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_25 {
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_26 {
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_27 {
    pub tqh_first: *mut window_mode_entry,
    pub tqh_last: *mut *mut window_mode_entry,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_mode_entry {
    pub wp: *mut window_pane,
    pub swp: *mut window_pane,
    pub mode: *const window_mode,
    pub data: *mut ::core::ffi::c_void,
    pub screen: *mut screen,
    pub prefix: u_int,
    pub kill: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_28,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_28 {
    pub tqe_next: *mut window_mode_entry,
    pub tqe_prev: *mut *mut window_mode_entry,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_mode {
    pub name: *const ::core::ffi::c_char,
    pub default_format: *const ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
    pub init: Option<
        unsafe extern "C" fn(
            *mut window_mode_entry,
            *mut cmdq_item,
            *mut cmd_find_state,
            *mut args,
        ) -> *mut screen,
    >,
    pub free: Option<unsafe extern "C" fn(*mut window_mode_entry) -> ()>,
    pub resize: Option<unsafe extern "C" fn(*mut window_mode_entry, u_int, u_int) -> ()>,
    pub update: Option<unsafe extern "C" fn(*mut window_mode_entry) -> ()>,
    pub style_changed: Option<unsafe extern "C" fn(*mut window_mode_entry) -> ()>,
    pub key: Option<
        unsafe extern "C" fn(
            *mut window_mode_entry,
            *mut client,
            *mut session,
            *mut winlink,
            key_code,
            *mut mouse_event,
        ) -> (),
    >,
    pub key_table:
        Option<unsafe extern "C" fn(*mut window_mode_entry) -> *const ::core::ffi::c_char>,
    pub command: Option<
        unsafe extern "C" fn(
            *mut window_mode_entry,
            *mut client,
            *mut session,
            *mut winlink,
            *mut args,
            *mut mouse_event,
        ) -> (),
    >,
    pub formats: Option<unsafe extern "C" fn(*mut window_mode_entry, *mut format_tree) -> ()>,
    pub get_screen: Option<unsafe extern "C" fn(*mut window_mode_entry) -> *mut screen>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_find_state {
    pub flags: ::core::ffi::c_int,
    pub current: *mut cmd_find_state,
    pub s: *mut session,
    pub wl: *mut winlink,
    pub w: *mut window,
    pub wp: *mut window_pane,
    pub idx: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_pane_offset {
    pub used: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_pane_resizes {
    pub tqh_first: *mut window_pane_resize,
    pub tqh_last: *mut *mut window_pane_resize,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_pane_resize {
    pub sx: u_int,
    pub sy: u_int,
    pub osx: u_int,
    pub osy: u_int,
    pub entry: C2RustUnnamed_30,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_30 {
    pub tqe_next: *mut window_pane_resize,
    pub tqe_prev: *mut *mut window_pane_resize,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct layout_geometry {
    pub sx: u_int,
    pub sy: u_int,
    pub xoff: ::core::ffi::c_int,
    pub yoff: ::core::ffi::c_int,
}
pub type layout_type = ::core::ffi::c_uint;
pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_panes {
    pub tqh_first: *mut window_pane,
    pub tqh_last: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct winlink_stack {
    pub tqh_first: *mut winlink,
    pub tqh_last: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_table {
    pub name: *const ::core::ffi::c_char,
    pub activity_time: timeval,
    pub key_bindings: key_bindings,
    pub default_key_bindings: key_bindings,
    pub references: u_int,
    pub entry: C2RustUnnamed_31,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_31 {
    pub rbe_left: *mut key_table,
    pub rbe_right: *mut key_table,
    pub rbe_parent: *mut key_table,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_bindings {
    pub rbh_root: *mut key_binding,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_binding {
    pub key: key_code,
    pub cmdlist: *mut cmd_list,
    pub note: *const ::core::ffi::c_char,
    pub tablename: *const ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_32 {
    pub rbe_left: *mut key_binding,
    pub rbe_right: *mut key_binding,
    pub rbe_parent: *mut key_binding,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_list {
    pub references: ::core::ffi::c_int,
    pub group: u_int,
    pub list: *mut cmds,
}
pub type C2RustUnnamed_33 = ::core::ffi::c_uint;
pub const CLIENT_EXIT_DETACH: C2RustUnnamed_33 = 2;
pub const CLIENT_EXIT_SHUTDOWN: C2RustUnnamed_33 = 1;
pub const CLIENT_EXIT_RETURN: C2RustUnnamed_33 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_requests {
    pub tqh_first: *mut input_request,
    pub tqh_last: *mut *mut input_request,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct status_line {
    pub timer: event,
    pub screen: screen,
    pub active: *mut screen,
    pub references: ::core::ffi::c_int,
    pub prompt_cx: u_int,
    pub style: grid_cell,
    pub entries: [style_line_entry; 5],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty {
    pub client: *mut client,
    pub start_timer: event,
    pub clipboard_timer: event,
    pub last_requests: time_t,
    pub sx: u_int,
    pub sy: u_int,
    pub xpixel: u_int,
    pub ypixel: u_int,
    pub cx: u_int,
    pub cy: u_int,
    pub cstyle: screen_cursor_style,
    pub ccolour: ::core::ffi::c_int,
    pub oflag: ::core::ffi::c_int,
    pub oox: u_int,
    pub ooy: u_int,
    pub osx: u_int,
    pub osy: u_int,
    pub mode: ::core::ffi::c_int,
    pub fg: ::core::ffi::c_int,
    pub bg: ::core::ffi::c_int,
    pub rlower: u_int,
    pub rupper: u_int,
    pub rleft: u_int,
    pub rright: u_int,
    pub event_in: event,
    pub in_0: *mut evbuffer,
    pub event_out: event,
    pub out: *mut evbuffer,
    pub timer: event,
    pub discarded: size_t,
    pub tio: termios,
    pub r: visible_ranges,
    pub cell: grid_cell,
    pub last_cell: grid_cell,
    pub flags: ::core::ffi::c_int,
    pub term: *mut tty_term,
    pub mouse_last_x: u_int,
    pub mouse_last_y: u_int,
    pub mouse_last_b: u_int,
    pub mouse_drag_flag: ::core::ffi::c_int,
    pub mouse_drag_x: u_int,
    pub mouse_drag_y: u_int,
    pub mouse_scrolling_flag: ::core::ffi::c_int,
    pub mouse_slider_mpos: ::core::ffi::c_int,
    pub mouse_last_pane: ::core::ffi::c_int,
    pub mouse_drag_update: Option<unsafe extern "C" fn(*mut client, *mut mouse_event) -> ()>,
    pub mouse_drag_release: Option<unsafe extern "C" fn(*mut client, *mut mouse_event) -> ()>,
    pub key_timer: event,
    pub key_tree: *mut tty_key,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_term {
    pub name: *mut ::core::ffi::c_char,
    pub tty: *mut tty,
    pub applied_features: ::core::ffi::c_int,
    pub acs: [[::core::ffi::c_char; 2]; 256],
    pub codes: *mut tty_code,
    pub flags: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_34,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_34 {
    pub le_next: *mut tty_term,
    pub le_prev: *mut *mut tty_term,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_pane_tree {
    pub rbh_root: *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct environ_entry {
    pub name: *mut ::core::ffi::c_char,
    pub value: *mut ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub rbe_left: *mut environ_entry,
    pub rbe_right: *mut environ_entry,
    pub rbe_parent: *mut environ_entry,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sessions {
    pub rbh_root: *mut session,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct clients {
    pub tqh_first: *mut client,
    pub tqh_last: *mut *mut client,
}
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const _PATH_DEV: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"/dev/\0") };
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const RB_INF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CMD_FIND_PREFER_UNATTACHED: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_FIND_QUIET: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CMD_FIND_WINDOW_INDEX: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_FIND_DEFAULT_MARKED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CMD_FIND_EXACT_SESSION: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CMD_FIND_EXACT_WINDOW: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CMD_FIND_CANFAIL: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
static mut cmd_find_session_table: [[*const ::core::ffi::c_char; 2]; 1] = [[
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
]];
static mut cmd_find_window_table: [[*const ::core::ffi::c_char; 2]; 6] = [
    [
        b"{start}\0" as *const u8 as *const ::core::ffi::c_char,
        b"^\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{last}\0" as *const u8 as *const ::core::ffi::c_char,
        b"!\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{end}\0" as *const u8 as *const ::core::ffi::c_char,
        b"$\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{next}\0" as *const u8 as *const ::core::ffi::c_char,
        b"+\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{previous}\0" as *const u8 as *const ::core::ffi::c_char,
        b"-\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
];
static mut cmd_find_pane_table: [[*const ::core::ffi::c_char; 2]; 16] = [
    [
        b"{last}\0" as *const u8 as *const ::core::ffi::c_char,
        b"!\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{next}\0" as *const u8 as *const ::core::ffi::c_char,
        b"+\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{previous}\0" as *const u8 as *const ::core::ffi::c_char,
        b"-\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{top}\0" as *const u8 as *const ::core::ffi::c_char,
        b"top\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{bottom}\0" as *const u8 as *const ::core::ffi::c_char,
        b"bottom\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{left}\0" as *const u8 as *const ::core::ffi::c_char,
        b"left\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{right}\0" as *const u8 as *const ::core::ffi::c_char,
        b"right\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{top-left}\0" as *const u8 as *const ::core::ffi::c_char,
        b"top-left\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{top-right}\0" as *const u8 as *const ::core::ffi::c_char,
        b"top-right\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{bottom-left}\0" as *const u8 as *const ::core::ffi::c_char,
        b"bottom-left\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{bottom-right}\0" as *const u8 as *const ::core::ffi::c_char,
        b"bottom-right\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{up-of}\0" as *const u8 as *const ::core::ffi::c_char,
        b"{up-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{down-of}\0" as *const u8 as *const ::core::ffi::c_char,
        b"{down-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{left-of}\0" as *const u8 as *const ::core::ffi::c_char,
        b"{left-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        b"{right-of}\0" as *const u8 as *const ::core::ffi::c_char,
        b"{right-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ],
    [
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
];
unsafe extern "C" fn cmd_find_inside_pane(mut c: *mut client) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    if c.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
    while !wp.is_null() {
        if (*wp).fd != -(1 as ::core::ffi::c_int)
            && strcmp(&raw mut (*wp).tty as *mut ::core::ffi::c_char, (*c).ttyname)
                == 0 as ::core::ffi::c_int
        {
            break;
        }
        wp = window_pane_tree_RB_NEXT(wp);
    }
    if wp.is_null() {
        envent = environ_find(
            (*c).environ,
            b"TMUX_PANE\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if !envent.is_null() {
            wp = window_pane_find_by_id_str((*envent).value);
        }
    }
    if !wp.is_null() {
        log_debug(
            b"%s: got pane %%%u (%s)\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_find_inside_pane\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
            &raw mut (*wp).tty as *mut ::core::ffi::c_char,
        );
    }
    return wp;
}
unsafe extern "C" fn cmd_find_client_better(
    mut c: *mut client,
    mut than: *mut client,
) -> ::core::ffi::c_int {
    if than.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    return if (*c).activity_time.tv_sec == (*than).activity_time.tv_sec {
        ((*c).activity_time.tv_usec > (*than).activity_time.tv_usec) as ::core::ffi::c_int
    } else {
        ((*c).activity_time.tv_sec > (*than).activity_time.tv_sec) as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_best_client(mut s: *mut session) -> *mut client {
    let mut c_loop: *mut client = ::core::ptr::null_mut::<client>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if (*s).attached == 0 as u_int {
        s = ::core::ptr::null_mut::<session>();
    }
    c = ::core::ptr::null_mut::<client>();
    c_loop = clients.tqh_first;
    while !c_loop.is_null() {
        if !(*c_loop).session.is_null() {
            if !(!s.is_null() && (*c_loop).session != s) {
                if cmd_find_client_better(c_loop, c) != 0 {
                    c = c_loop;
                }
            }
        }
        c_loop = (*c_loop).entry.tqe_next;
    }
    return c;
}
unsafe extern "C" fn cmd_find_session_better(
    mut s: *mut session,
    mut than: *mut session,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut attached: ::core::ffi::c_int = 0;
    if than.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    if flags & CMD_FIND_PREFER_UNATTACHED != 0 {
        attached = ((*than).attached != 0 as u_int) as ::core::ffi::c_int;
        if attached != 0 && (*s).attached == 0 as u_int {
            return 1 as ::core::ffi::c_int;
        } else if attached == 0 && (*s).attached != 0 as u_int {
            return 0 as ::core::ffi::c_int;
        }
    }
    return if (*s).activity_time.tv_sec == (*than).activity_time.tv_sec {
        ((*s).activity_time.tv_usec > (*than).activity_time.tv_usec) as ::core::ffi::c_int
    } else {
        ((*s).activity_time.tv_sec > (*than).activity_time.tv_sec) as ::core::ffi::c_int
    };
}
unsafe extern "C" fn cmd_find_session_valid(mut s: *mut session) -> ::core::ffi::c_int {
    if session_alive(s) == 0
        || (*s).curw.is_null()
        || (*(*s).curw).window.is_null()
        || (*(*(*s).curw).window).active.is_null()
    {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn cmd_find_best_session(
    mut slist: *mut *mut session,
    mut ssize: u_int,
    mut flags: ::core::ffi::c_int,
) -> *mut session {
    let mut s_loop: *mut session = ::core::ptr::null_mut::<session>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut i: u_int = 0;
    log_debug(
        b"%s: %u sessions to try\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_best_session\0" as *const u8 as *const ::core::ffi::c_char,
        ssize,
    );
    s = ::core::ptr::null_mut::<session>();
    if !slist.is_null() {
        i = 0 as u_int;
        while i < ssize {
            if !(cmd_find_session_valid(*slist.offset(i as isize)) == 0) {
                if cmd_find_session_better(*slist.offset(i as isize), s, flags) != 0 {
                    s = *slist.offset(i as isize);
                }
            }
            i = i.wrapping_add(1);
        }
    } else {
        s_loop = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
        while !s_loop.is_null() {
            if !(cmd_find_session_valid(s_loop) == 0) {
                if cmd_find_session_better(s_loop, s, flags) != 0 {
                    s = s_loop;
                }
            }
            s_loop = sessions_RB_NEXT(s_loop);
        }
    }
    return s;
}
unsafe extern "C" fn cmd_find_best_session_with_window(
    mut fs: *mut cmd_find_state,
) -> ::core::ffi::c_int {
    let mut slist: *mut *mut session = ::core::ptr::null_mut::<*mut session>();
    let mut ssize: u_int = 0;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    log_debug(
        b"%s: window is @%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_best_session_with_window\0" as *const u8 as *const ::core::ffi::c_char,
        (*(*fs).w).id,
    );
    ssize = 0 as u_int;
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        if !(session_has(s, (*fs).w) == 0) {
            slist = xreallocarray(
                slist as *mut ::core::ffi::c_void,
                ssize.wrapping_add(1 as u_int) as size_t,
                ::core::mem::size_of::<*mut session>() as size_t,
            ) as *mut *mut session;
            let fresh2 = ssize;
            ssize = ssize.wrapping_add(1);
            let ref mut fresh3 = *slist.offset(fresh2 as isize);
            *fresh3 = s;
        }
        s = sessions_RB_NEXT(s);
    }
    if !(ssize == 0 as u_int) {
        (*fs).s = cmd_find_best_session(slist, ssize, (*fs).flags);
        if !(*fs).s.is_null() {
            free(slist as *mut ::core::ffi::c_void);
            return cmd_find_best_winlink_with_window(fs);
        }
    }
    free(slist as *mut ::core::ffi::c_void);
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn cmd_find_best_winlink_with_window(
    mut fs: *mut cmd_find_state,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wl_loop: *mut winlink = ::core::ptr::null_mut::<winlink>();
    log_debug(
        b"%s: window is @%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_best_winlink_with_window\0" as *const u8 as *const ::core::ffi::c_char,
        (*(*fs).w).id,
    );
    wl = ::core::ptr::null_mut::<winlink>();
    if !(*(*fs).s).curw.is_null() && (*(*(*fs).s).curw).window == (*fs).w {
        wl = (*(*fs).s).curw;
    } else {
        wl_loop = winlinks_RB_MINMAX(&raw mut (*(*fs).s).windows, RB_NEGINF);
        while !wl_loop.is_null() {
            if (*wl_loop).window == (*fs).w {
                wl = wl_loop;
                break;
            } else {
                wl_loop = winlinks_RB_NEXT(wl_loop);
            }
        }
    }
    if wl.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).wl = wl;
    (*fs).idx = (*(*fs).wl).idx;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn cmd_find_map_table(
    mut table: *mut [*const ::core::ffi::c_char; 2],
    mut s: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
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
    return s;
}
unsafe extern "C" fn cmd_find_get_session(
    mut fs: *mut cmd_find_state,
    mut session: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut s_loop: *mut session = ::core::ptr::null_mut::<session>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_get_session\0" as *const u8 as *const ::core::ffi::c_char,
        session,
    );
    if *session as ::core::ffi::c_int == '$' as i32 {
        (*fs).s = session_find_by_id_str(session);
        if (*fs).s.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    }
    (*fs).s = session_find(session);
    if !(*fs).s.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    c = cmd_find_client(
        ::core::ptr::null_mut::<cmdq_item>(),
        session,
        1 as ::core::ffi::c_int,
    );
    if !c.is_null() && !(*c).session.is_null() {
        (*fs).s = (*c).session;
        return 0 as ::core::ffi::c_int;
    }
    if (*fs).flags & CMD_FIND_EXACT_SESSION != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    s = ::core::ptr::null_mut::<session>();
    s_loop = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s_loop.is_null() {
        if strncmp(session, (*s_loop).name, strlen(session)) == 0 as ::core::ffi::c_int {
            if !s.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            s = s_loop;
        }
        s_loop = sessions_RB_NEXT(s_loop);
    }
    if !s.is_null() {
        (*fs).s = s;
        return 0 as ::core::ffi::c_int;
    }
    s = ::core::ptr::null_mut::<session>();
    s_loop = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s_loop.is_null() {
        if fnmatch(session, (*s_loop).name, 0 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int {
            if !s.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            s = s_loop;
        }
        s_loop = sessions_RB_NEXT(s_loop);
    }
    if !s.is_null() {
        (*fs).s = s;
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn cmd_find_get_window(
    mut fs: *mut cmd_find_state,
    mut window: *const ::core::ffi::c_char,
    mut only: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_get_window\0" as *const u8 as *const ::core::ffi::c_char,
        window,
    );
    if *window as ::core::ffi::c_int == '@' as i32 {
        (*fs).w = window_find_by_id_str(window);
        if (*fs).w.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return cmd_find_best_session_with_window(fs);
    }
    (*fs).s = (*(*fs).current).s;
    if cmd_find_get_window_with_session(fs, window) == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if only == 0 && cmd_find_get_session(fs, window) == 0 as ::core::ffi::c_int {
        (*fs).wl = (*(*fs).s).curw;
        (*fs).w = (*(*fs).wl).window;
        if !(*fs).flags & CMD_FIND_WINDOW_INDEX != 0 {
            (*fs).idx = (*(*fs).wl).idx;
        }
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn cmd_find_get_window_with_session(
    mut fs: *mut cmd_find_state,
    mut window: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut exact: ::core::ffi::c_int = 0;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_get_window_with_session\0" as *const u8 as *const ::core::ffi::c_char,
        window,
    );
    exact = (*fs).flags & CMD_FIND_EXACT_WINDOW;
    (*fs).wl = (*(*fs).s).curw;
    (*fs).w = (*(*fs).wl).window;
    if *window as ::core::ffi::c_int == '@' as i32 {
        (*fs).w = window_find_by_id_str(window);
        if (*fs).w.is_null() || session_has((*fs).s, (*fs).w) == 0 {
            return -(1 as ::core::ffi::c_int);
        }
        return cmd_find_best_winlink_with_window(fs);
    }
    if exact == 0
        && (*window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '+' as i32
            || *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '-' as i32)
    {
        if *window.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32 {
            n = strtonum(
                window.offset(1 as ::core::ffi::c_int as isize),
                1 as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as ::core::ffi::c_int;
            if !errstr.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
        } else {
            n = 1 as ::core::ffi::c_int;
        }
        s = (*fs).s;
        if (*fs).flags & CMD_FIND_WINDOW_INDEX != 0 {
            if *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '+' as i32
            {
                if INT_MAX - (*(*s).curw).idx < n {
                    return -(1 as ::core::ffi::c_int);
                }
                (*fs).idx = (*(*s).curw).idx + n;
            } else {
                if n > (*(*s).curw).idx {
                    return -(1 as ::core::ffi::c_int);
                }
                (*fs).idx = (*(*s).curw).idx - n;
            }
            return 0 as ::core::ffi::c_int;
        }
        if *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '+' as i32 {
            (*fs).wl = winlink_next_by_number((*s).curw, s, n);
        } else {
            (*fs).wl = winlink_previous_by_number((*s).curw, s, n);
        }
        if !(*fs).wl.is_null() {
            (*fs).idx = (*(*fs).wl).idx;
            (*fs).w = (*(*fs).wl).window;
            return 0 as ::core::ffi::c_int;
        }
    }
    if exact == 0 {
        if strcmp(window, b"!\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            (*fs).wl = (*(*fs).s).lastw.tqh_first;
            if (*fs).wl.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).idx = (*(*fs).wl).idx;
            (*fs).w = (*(*fs).wl).window;
            return 0 as ::core::ffi::c_int;
        } else if strcmp(window, b"^\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            (*fs).wl = winlinks_RB_MINMAX(&raw mut (*(*fs).s).windows, RB_NEGINF);
            if (*fs).wl.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).idx = (*(*fs).wl).idx;
            (*fs).w = (*(*fs).wl).window;
            return 0 as ::core::ffi::c_int;
        } else if strcmp(window, b"$\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            (*fs).wl = winlinks_RB_MINMAX(&raw mut (*(*fs).s).windows, RB_INF);
            if (*fs).wl.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).idx = (*(*fs).wl).idx;
            (*fs).w = (*(*fs).wl).window;
            return 0 as ::core::ffi::c_int;
        }
    }
    if *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '+' as i32
        && *window.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '-' as i32
    {
        idx = strtonum(
            window,
            0 as ::core::ffi::c_longlong,
            INT_MAX as ::core::ffi::c_longlong,
            &raw mut errstr,
        ) as ::core::ffi::c_int;
        if errstr.is_null() {
            (*fs).wl = winlink_find_by_index(&raw mut (*(*fs).s).windows, idx);
            if !(*fs).wl.is_null() {
                (*fs).idx = (*(*fs).wl).idx;
                (*fs).w = (*(*fs).wl).window;
                return 0 as ::core::ffi::c_int;
            }
            if (*fs).flags & CMD_FIND_WINDOW_INDEX != 0 {
                (*fs).idx = idx;
                return 0 as ::core::ffi::c_int;
            }
        }
    }
    (*fs).wl = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_RB_MINMAX(&raw mut (*(*fs).s).windows, RB_NEGINF);
    while !wl.is_null() {
        if strcmp(window, (*(*wl).window).name) == 0 as ::core::ffi::c_int {
            if !(*fs).wl.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).wl = wl;
        }
        wl = winlinks_RB_NEXT(wl);
    }
    if !(*fs).wl.is_null() {
        (*fs).idx = (*(*fs).wl).idx;
        (*fs).w = (*(*fs).wl).window;
        return 0 as ::core::ffi::c_int;
    }
    if exact != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).wl = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_RB_MINMAX(&raw mut (*(*fs).s).windows, RB_NEGINF);
    while !wl.is_null() {
        if strncmp(window, (*(*wl).window).name, strlen(window)) == 0 as ::core::ffi::c_int {
            if !(*fs).wl.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).wl = wl;
        }
        wl = winlinks_RB_NEXT(wl);
    }
    if !(*fs).wl.is_null() {
        (*fs).idx = (*(*fs).wl).idx;
        (*fs).w = (*(*fs).wl).window;
        return 0 as ::core::ffi::c_int;
    }
    (*fs).wl = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_RB_MINMAX(&raw mut (*(*fs).s).windows, RB_NEGINF);
    while !wl.is_null() {
        if fnmatch(window, (*(*wl).window).name, 0 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int
        {
            if !(*fs).wl.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
            (*fs).wl = wl;
        }
        wl = winlinks_RB_NEXT(wl);
    }
    if !(*fs).wl.is_null() {
        (*fs).idx = (*(*fs).wl).idx;
        (*fs).w = (*(*fs).wl).window;
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn cmd_find_get_pane(
    mut fs: *mut cmd_find_state,
    mut pane: *const ::core::ffi::c_char,
    mut only: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_get_pane\0" as *const u8 as *const ::core::ffi::c_char,
        pane,
    );
    if *pane as ::core::ffi::c_int == '%' as i32 {
        (*fs).wp = window_pane_find_by_id_str(pane);
        if (*fs).wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        (*fs).w = (*(*fs).wp).window as *mut window;
        return cmd_find_best_session_with_window(fs);
    }
    (*fs).s = (*(*fs).current).s;
    (*fs).wl = (*(*fs).current).wl;
    (*fs).idx = (*(*fs).current).idx;
    (*fs).w = (*(*fs).current).w;
    if cmd_find_get_pane_with_window(fs, pane) == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if only == 0
        && cmd_find_get_window(fs, pane, 0 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int
    {
        (*fs).wp = (*(*fs).w).active;
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn cmd_find_get_pane_with_session(
    mut fs: *mut cmd_find_state,
    mut pane: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_get_pane_with_session\0" as *const u8 as *const ::core::ffi::c_char,
        pane,
    );
    if *pane as ::core::ffi::c_int == '%' as i32 {
        (*fs).wp = window_pane_find_by_id_str(pane);
        if (*fs).wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        (*fs).w = (*(*fs).wp).window as *mut window;
        return cmd_find_best_winlink_with_window(fs);
    }
    (*fs).wl = (*(*fs).s).curw;
    (*fs).idx = (*(*fs).wl).idx;
    (*fs).w = (*(*fs).wl).window;
    return cmd_find_get_pane_with_window(fs, pane);
}
unsafe extern "C" fn cmd_find_get_pane_with_window(
    mut fs: *mut cmd_find_state,
    mut pane: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = 0;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut n: u_int = 0;
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_get_pane_with_window\0" as *const u8 as *const ::core::ffi::c_char,
        pane,
    );
    if *pane as ::core::ffi::c_int == '%' as i32 {
        (*fs).wp = window_pane_find_by_id_str(pane);
        if (*fs).wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        if (*(*fs).wp).window != (*fs).w {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    }
    if strcmp(pane, b"!\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        (*fs).wp = (*(*fs).w).last_panes.tqh_first;
        if (*fs).wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    } else if strcmp(
        pane,
        b"{up-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        (*fs).wp = window_pane_find_up((*(*fs).w).active);
        if (*fs).wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    } else if strcmp(
        pane,
        b"{down-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        (*fs).wp = window_pane_find_down((*(*fs).w).active);
        if (*fs).wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    } else if strcmp(
        pane,
        b"{left-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        (*fs).wp = window_pane_find_left((*(*fs).w).active);
        if (*fs).wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        return 0 as ::core::ffi::c_int;
    } else if strcmp(
        pane,
        b"{right-of}\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        (*fs).wp = window_pane_find_right((*(*fs).w).active);
        if (*fs).wp.is_null() {
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
                &raw mut errstr,
            ) as u_int;
            if !errstr.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
        } else {
            n = 1 as u_int;
        }
        wp = (*(*fs).w).active;
        if *pane.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '+' as i32 {
            (*fs).wp = window_pane_next_by_number((*fs).w, wp, n);
        } else {
            (*fs).wp = window_pane_previous_by_number((*fs).w, wp, n);
        }
        if !(*fs).wp.is_null() {
            return 0 as ::core::ffi::c_int;
        }
    }
    idx = strtonum(
        pane,
        0 as ::core::ffi::c_longlong,
        INT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as ::core::ffi::c_int;
    if errstr.is_null() {
        (*fs).wp = window_pane_at_index((*fs).w, idx as u_int);
        if !(*fs).wp.is_null() {
            return 0 as ::core::ffi::c_int;
        }
    }
    (*fs).wp = window_find_string((*fs).w, pane);
    if !(*fs).wp.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_clear_state(
    mut fs: *mut cmd_find_state,
    mut flags: ::core::ffi::c_int,
) {
    memset(
        fs as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_find_state>() as size_t,
    );
    (*fs).flags = flags;
    (*fs).idx = -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_empty_state(mut fs: *mut cmd_find_state) -> ::core::ffi::c_int {
    if (*fs).s.is_null() && (*fs).wl.is_null() && (*fs).w.is_null() && (*fs).wp.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_valid_state(mut fs: *mut cmd_find_state) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if (*fs).s.is_null() || (*fs).wl.is_null() || (*fs).w.is_null() || (*fs).wp.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if session_alive((*fs).s) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    wl = winlinks_RB_MINMAX(&raw mut (*(*fs).s).windows, RB_NEGINF);
    while !wl.is_null() {
        if (*wl).window == (*fs).w && wl == (*fs).wl {
            break;
        }
        wl = winlinks_RB_NEXT(wl);
    }
    if wl.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*fs).w != (*(*fs).wl).window {
        return 0 as ::core::ffi::c_int;
    }
    return window_has_pane((*fs).w, (*fs).wp);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_copy_state(
    mut dst: *mut cmd_find_state,
    mut src: *mut cmd_find_state,
) {
    (*dst).s = (*src).s;
    (*dst).wl = (*src).wl;
    (*dst).idx = (*src).idx;
    (*dst).w = (*src).w;
    (*dst).wp = (*src).wp;
}
unsafe extern "C" fn cmd_find_log_state(
    mut prefix: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
) {
    if !(*fs).s.is_null() {
        log_debug(
            b"%s: s=$%u %s\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
            (*(*fs).s).id,
            (*(*fs).s).name,
        );
    } else {
        log_debug(
            b"%s: s=none\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
        );
    }
    if !(*fs).wl.is_null() {
        log_debug(
            b"%s: wl=%u %d w=@%u %s\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
            (*(*fs).wl).idx,
            ((*(*fs).wl).window == (*fs).w) as ::core::ffi::c_int,
            (*(*fs).w).id,
            (*(*fs).w).name,
        );
    } else {
        log_debug(
            b"%s: wl=none\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
        );
    }
    if !(*fs).wp.is_null() {
        log_debug(
            b"%s: wp=%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
            (*(*fs).wp).id,
        );
    } else {
        log_debug(
            b"%s: wp=none\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
        );
    }
    if (*fs).idx != -(1 as ::core::ffi::c_int) {
        log_debug(
            b"%s: idx=%d\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
            (*fs).idx,
        );
    } else {
        log_debug(
            b"%s: idx=none\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_session(
    mut fs: *mut cmd_find_state,
    mut s: *mut session,
    mut flags: ::core::ffi::c_int,
) {
    cmd_find_clear_state(fs, flags);
    (*fs).s = s;
    (*fs).wl = (*(*fs).s).curw;
    (*fs).w = (*(*fs).wl).window;
    (*fs).wp = (*(*fs).w).active;
    cmd_find_log_state(
        b"cmd_find_from_session\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_winlink(
    mut fs: *mut cmd_find_state,
    mut wl: *mut winlink,
    mut flags: ::core::ffi::c_int,
) {
    cmd_find_clear_state(fs, flags);
    (*fs).s = (*wl).session;
    (*fs).wl = wl;
    (*fs).w = (*wl).window;
    (*fs).wp = (*(*wl).window).active;
    cmd_find_log_state(
        b"cmd_find_from_winlink\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_session_window(
    mut fs: *mut cmd_find_state,
    mut s: *mut session,
    mut w: *mut window,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    cmd_find_clear_state(fs, flags);
    (*fs).s = s;
    (*fs).w = w;
    if cmd_find_best_winlink_with_window(fs) != 0 as ::core::ffi::c_int {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).wp = (*(*fs).w).active;
    cmd_find_log_state(
        b"cmd_find_from_session_window\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_window(
    mut fs: *mut cmd_find_state,
    mut w: *mut window,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    cmd_find_clear_state(fs, flags);
    (*fs).w = w;
    if cmd_find_best_session_with_window(fs) != 0 as ::core::ffi::c_int {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    if cmd_find_best_winlink_with_window(fs) != 0 as ::core::ffi::c_int {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).wp = (*(*fs).w).active;
    cmd_find_log_state(
        b"cmd_find_from_window\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_winlink_pane(
    mut fs: *mut cmd_find_state,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
) {
    cmd_find_clear_state(fs, flags);
    (*fs).s = (*wl).session;
    (*fs).wl = wl;
    (*fs).idx = (*(*fs).wl).idx;
    (*fs).w = (*(*fs).wl).window;
    (*fs).wp = wp;
    cmd_find_log_state(
        b"cmd_find_from_winlink_pane\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_pane(
    mut fs: *mut cmd_find_state,
    mut wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if cmd_find_from_window(fs, (*wp).window as *mut window, flags) != 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).wp = wp;
    cmd_find_log_state(
        b"cmd_find_from_pane\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_nothing(
    mut fs: *mut cmd_find_state,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    cmd_find_clear_state(fs, flags);
    (*fs).s = cmd_find_best_session(::core::ptr::null_mut::<*mut session>(), 0 as u_int, flags);
    if (*fs).s.is_null() {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).wl = (*(*fs).s).curw;
    (*fs).idx = (*(*fs).wl).idx;
    (*fs).w = (*(*fs).wl).window;
    (*fs).wp = (*(*fs).w).active;
    cmd_find_log_state(
        b"cmd_find_from_nothing\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_mouse(
    mut fs: *mut cmd_find_state,
    mut m: *mut mouse_event,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    cmd_find_clear_state(fs, flags);
    if (*m).valid == 0 {
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).wp = cmd_mouse_pane(m, &raw mut (*fs).s, &raw mut (*fs).wl);
    if (*fs).wp.is_null() {
        cmd_find_clear_state(fs, flags);
        return -(1 as ::core::ffi::c_int);
    }
    (*fs).w = (*(*fs).wl).window;
    cmd_find_log_state(
        b"cmd_find_from_mouse\0" as *const u8 as *const ::core::ffi::c_char,
        fs,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_from_client(
    mut fs: *mut cmd_find_state,
    mut c: *mut client,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if c.is_null() {
        return cmd_find_from_nothing(fs, flags);
    }
    if !(*c).session.is_null() {
        cmd_find_clear_state(fs, flags);
        (*fs).wp = (*(*(*(*c).session).curw).window).active;
        if (*fs).wp.is_null() {
            cmd_find_from_session(fs, (*c).session, flags);
            return 0 as ::core::ffi::c_int;
        }
        (*fs).s = (*c).session;
        (*fs).wl = (*(*fs).s).curw;
        (*fs).w = (*(*fs).wl).window;
        cmd_find_log_state(
            b"cmd_find_from_client\0" as *const u8 as *const ::core::ffi::c_char,
            fs,
        );
        return 0 as ::core::ffi::c_int;
    }
    cmd_find_clear_state(fs, flags);
    wp = cmd_find_inside_pane(c);
    if !wp.is_null() {
        (*fs).w = (*wp).window as *mut window;
        if !(cmd_find_best_session_with_window(fs) != 0 as ::core::ffi::c_int) {
            (*fs).wl = (*(*fs).s).curw;
            (*fs).w = (*(*fs).wl).window;
            (*fs).wp = (*(*fs).w).active;
            cmd_find_log_state(
                b"cmd_find_from_client\0" as *const u8 as *const ::core::ffi::c_char,
                fs,
            );
            return 0 as ::core::ffi::c_int;
        }
    }
    return cmd_find_from_nothing(fs, flags);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_target(
    mut fs: *mut cmd_find_state,
    mut item: *mut cmdq_item,
    mut target: *const ::core::ffi::c_char,
    mut type_0: cmd_find_type,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut m: *mut mouse_event = ::core::ptr::null_mut::<mouse_event>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut current: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut colon: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut period: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tmp: [::core::ffi::c_char; 256] = [0; 256];
    let mut session: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut window: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut pane: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut window_only: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut pane_only: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if flags & CMD_FIND_CANFAIL != 0 {
        flags |= CMD_FIND_QUIET;
    }
    if type_0 as ::core::ffi::c_uint == CMD_FIND_PANE as ::core::ffi::c_int as ::core::ffi::c_uint {
        s = b"pane\0" as *const u8 as *const ::core::ffi::c_char;
    } else if type_0 as ::core::ffi::c_uint
        == CMD_FIND_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        s = b"window\0" as *const u8 as *const ::core::ffi::c_char;
    } else if type_0 as ::core::ffi::c_uint
        == CMD_FIND_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        s = b"session\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        s = b"unknown\0" as *const u8 as *const ::core::ffi::c_char;
    }
    *(&raw mut tmp as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if flags & CMD_FIND_PREFER_UNATTACHED != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"PREFER_UNATTACHED,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & CMD_FIND_QUIET != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"QUIET,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & CMD_FIND_WINDOW_INDEX != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"WINDOW_INDEX,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & CMD_FIND_DEFAULT_MARKED != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"DEFAULT_MARKED,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & CMD_FIND_EXACT_SESSION != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"EXACT_SESSION,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & CMD_FIND_EXACT_WINDOW != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"EXACT_WINDOW,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & CMD_FIND_CANFAIL != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"CANFAIL,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if *(&raw mut tmp as *mut ::core::ffi::c_char) as ::core::ffi::c_int != '\0' as i32 {
        tmp[strlen(&raw mut tmp as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
            '\0' as i32 as ::core::ffi::c_char;
    } else {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"NONE\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    log_debug(
        b"%s: target %s, type %s, item %p, flags %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_target\0" as *const u8 as *const ::core::ffi::c_char,
        if target.is_null() {
            b"none\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            target
        },
        s,
        item,
        &raw mut tmp as *mut ::core::ffi::c_char,
    );
    cmd_find_clear_state(fs, flags);
    if server_check_marked() != 0 && flags & CMD_FIND_DEFAULT_MARKED != 0 {
        (*fs).current = &raw mut marked_pane;
        log_debug(
            b"%s: current is marked pane\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_find_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        current_block = 1836292691772056875;
    } else if cmd_find_valid_state(cmdq_get_current(item)) != 0 {
        (*fs).current = cmdq_get_current(item);
        log_debug(
            b"%s: current is from queue\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_find_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        current_block = 1836292691772056875;
    } else if cmd_find_from_client(&raw mut current, cmdq_get_client(item), flags)
        == 0 as ::core::ffi::c_int
    {
        (*fs).current = &raw mut current;
        log_debug(
            b"%s: current is from client\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_find_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        current_block = 1836292691772056875;
    } else {
        if !flags & CMD_FIND_QUIET != 0 {
            cmdq_error(
                item,
                b"no current target\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        current_block = 5193823237153215208;
    }
    match current_block {
        1836292691772056875 => {
            if cmd_find_valid_state((*fs).current) == 0 {
                fatalx(b"invalid current find state\0" as *const u8 as *const ::core::ffi::c_char);
            }
            if target.is_null() || *target as ::core::ffi::c_int == '\0' as i32 {
                current_block = 6284300254771030961;
            } else if strcmp(target, b"@\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
                || strcmp(
                    target,
                    b"{active}\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                || strcmp(
                    target,
                    b"{current}\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                c = cmdq_get_client(item);
                if c.is_null() || (*c).session.is_null() {
                    cmdq_error(
                        item,
                        b"no current client\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                    current_block = 5193823237153215208;
                } else {
                    (*fs).wl = (*(*c).session).curw;
                    (*fs).wp = (*(*(*(*c).session).curw).window).active;
                    (*fs).w = (*(*(*c).session).curw).window;
                    current_block = 15319680530019787978;
                }
            } else if strcmp(target, b"=\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
                || strcmp(
                    target,
                    b"{mouse}\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                m = &raw mut (*(cmdq_get_event
                    as unsafe extern "C" fn(*mut cmdq_item) -> *mut key_event)(
                    item
                ))
                .m;
                let mut current_block_56: u64;
                match type_0 as ::core::ffi::c_uint {
                    0 => {
                        (*fs).wp = cmd_mouse_pane(m, &raw mut (*fs).s, &raw mut (*fs).wl);
                        if !(*fs).wp.is_null() {
                            (*fs).w = (*(*fs).wl).window;
                            current_block_56 = 7343950298149844727;
                        } else {
                            current_block_56 = 2308649987175926278;
                        }
                    }
                    1 | 2 => {
                        current_block_56 = 2308649987175926278;
                    }
                    _ => {
                        current_block_56 = 7343950298149844727;
                    }
                }
                match current_block_56 {
                    2308649987175926278 => {
                        (*fs).wl = cmd_mouse_window(m, &raw mut (*fs).s);
                        if (*fs).wl.is_null() && !(*fs).s.is_null() {
                            (*fs).wl = (*(*fs).s).curw;
                        }
                        if !(*fs).wl.is_null() {
                            (*fs).w = (*(*fs).wl).window;
                            (*fs).wp = (*(*fs).w).active;
                        }
                    }
                    _ => {}
                }
                if (*fs).wp.is_null() {
                    if !flags & CMD_FIND_QUIET != 0 {
                        cmdq_error(
                            item,
                            b"no mouse target\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    }
                    current_block = 5193823237153215208;
                } else {
                    current_block = 15319680530019787978;
                }
            } else if strcmp(target, b"~\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
                || strcmp(
                    target,
                    b"{marked}\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                if server_check_marked() == 0 {
                    if !flags & CMD_FIND_QUIET != 0 {
                        cmdq_error(
                            item,
                            b"no marked target\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    }
                    current_block = 5193823237153215208;
                } else {
                    cmd_find_copy_state(fs, &raw mut marked_pane);
                    current_block = 15319680530019787978;
                }
            } else {
                copy = xstrdup(target);
                colon = strchr(copy, ':' as i32);
                if !colon.is_null() {
                    let fresh0 = colon;
                    colon = colon.offset(1);
                    *fresh0 = '\0' as i32 as ::core::ffi::c_char;
                }
                if colon.is_null() {
                    period = strchr(copy, '.' as i32);
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
                    session = copy;
                    window = colon;
                    window_only = 1 as ::core::ffi::c_int;
                    pane = period;
                    pane_only = 1 as ::core::ffi::c_int;
                } else if !colon.is_null() && period.is_null() {
                    session = copy;
                    window = colon;
                    window_only = 1 as ::core::ffi::c_int;
                } else if colon.is_null() && !period.is_null() {
                    window = copy;
                    pane = period;
                    pane_only = 1 as ::core::ffi::c_int;
                } else if *copy as ::core::ffi::c_int == '$' as i32 {
                    session = copy;
                } else if *copy as ::core::ffi::c_int == '@' as i32 {
                    window = copy;
                } else if *copy as ::core::ffi::c_int == '%' as i32 {
                    pane = copy;
                } else {
                    match type_0 as ::core::ffi::c_uint {
                        2 => {
                            session = copy;
                        }
                        1 => {
                            window = copy;
                        }
                        0 => {
                            pane = copy;
                        }
                        _ => {}
                    }
                }
                if !session.is_null() && *session as ::core::ffi::c_int == '=' as i32 {
                    session = session.offset(1);
                    (*fs).flags |= CMD_FIND_EXACT_SESSION;
                }
                if !window.is_null() && *window as ::core::ffi::c_int == '=' as i32 {
                    window = window.offset(1);
                    (*fs).flags |= CMD_FIND_EXACT_WINDOW;
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
                    session = cmd_find_map_table(
                        &raw mut cmd_find_session_table as *mut [*const ::core::ffi::c_char; 2],
                        session,
                    );
                }
                if !window.is_null() {
                    window = cmd_find_map_table(
                        &raw mut cmd_find_window_table as *mut [*const ::core::ffi::c_char; 2],
                        window,
                    );
                }
                if !pane.is_null() {
                    pane = cmd_find_map_table(
                        &raw mut cmd_find_pane_table as *mut [*const ::core::ffi::c_char; 2],
                        pane,
                    );
                }
                if !session.is_null() || !window.is_null() || !pane.is_null() {
                    log_debug(
                        b"%s: target %s is %s%s%s%s%s%s\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"cmd_find_target\0" as *const u8 as *const ::core::ffi::c_char,
                        target,
                        if session.is_null() {
                            b"\0" as *const u8 as *const ::core::ffi::c_char
                        } else {
                            b"session \0" as *const u8 as *const ::core::ffi::c_char
                        },
                        if session.is_null() {
                            b"\0" as *const u8 as *const ::core::ffi::c_char
                        } else {
                            session
                        },
                        if window.is_null() {
                            b"\0" as *const u8 as *const ::core::ffi::c_char
                        } else {
                            b"window \0" as *const u8 as *const ::core::ffi::c_char
                        },
                        if window.is_null() {
                            b"\0" as *const u8 as *const ::core::ffi::c_char
                        } else {
                            window
                        },
                        if pane.is_null() {
                            b"\0" as *const u8 as *const ::core::ffi::c_char
                        } else {
                            b"pane \0" as *const u8 as *const ::core::ffi::c_char
                        },
                        if pane.is_null() {
                            b"\0" as *const u8 as *const ::core::ffi::c_char
                        } else {
                            pane
                        },
                    );
                }
                if !pane.is_null() && flags & CMD_FIND_WINDOW_INDEX != 0 {
                    if !flags & CMD_FIND_QUIET != 0 {
                        cmdq_error(
                            item,
                            b"can't specify pane here\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    }
                    current_block = 5193823237153215208;
                } else {
                    if !session.is_null() {
                        if cmd_find_get_session(fs, session) != 0 as ::core::ffi::c_int {
                            if !flags & CMD_FIND_QUIET != 0 {
                                cmdq_error(
                                    item,
                                    b"can't find session: %s\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    session,
                                );
                            }
                            current_block = 5193823237153215208;
                        } else if window.is_null() && pane.is_null() {
                            (*fs).wl = (*(*fs).s).curw;
                            (*fs).idx = -(1 as ::core::ffi::c_int);
                            (*fs).w = (*(*fs).wl).window;
                            (*fs).wp = (*(*fs).w).active;
                            current_block = 15319680530019787978;
                        } else if !window.is_null() && pane.is_null() {
                            if cmd_find_get_window_with_session(fs, window)
                                != 0 as ::core::ffi::c_int
                            {
                                current_block = 2743676411188200708;
                            } else {
                                if !(*fs).wl.is_null() {
                                    (*fs).wp = (*(*(*fs).wl).window).active;
                                }
                                current_block = 15319680530019787978;
                            }
                        } else if window.is_null() && !pane.is_null() {
                            if cmd_find_get_pane_with_session(fs, pane) != 0 as ::core::ffi::c_int {
                                current_block = 14917847580669770662;
                            } else {
                                current_block = 15319680530019787978;
                            }
                        } else if cmd_find_get_window_with_session(fs, window)
                            != 0 as ::core::ffi::c_int
                        {
                            current_block = 2743676411188200708;
                        } else if cmd_find_get_pane_with_window(fs, pane) != 0 as ::core::ffi::c_int
                        {
                            current_block = 14917847580669770662;
                        } else {
                            current_block = 15319680530019787978;
                        }
                    } else if !window.is_null() && !pane.is_null() {
                        if cmd_find_get_window(fs, window, window_only) != 0 as ::core::ffi::c_int {
                            current_block = 2743676411188200708;
                        } else if cmd_find_get_pane_with_window(fs, pane) != 0 as ::core::ffi::c_int
                        {
                            current_block = 14917847580669770662;
                        } else {
                            current_block = 15319680530019787978;
                        }
                    } else if !window.is_null() && pane.is_null() {
                        if cmd_find_get_window(fs, window, window_only) != 0 as ::core::ffi::c_int {
                            current_block = 2743676411188200708;
                        } else {
                            if !(*fs).wl.is_null() {
                                (*fs).wp = (*(*(*fs).wl).window).active;
                            }
                            current_block = 15319680530019787978;
                        }
                    } else if window.is_null() && !pane.is_null() {
                        if cmd_find_get_pane(fs, pane, pane_only) != 0 as ::core::ffi::c_int {
                            current_block = 14917847580669770662;
                        } else {
                            current_block = 15319680530019787978;
                        }
                    } else {
                        current_block = 6284300254771030961;
                    }
                    match current_block {
                        5193823237153215208 => {}
                        15319680530019787978 => {}
                        6284300254771030961 => {}
                        _ => {
                            match current_block {
                                2743676411188200708 => {
                                    if !flags & CMD_FIND_QUIET != 0 {
                                        cmdq_error(
                                            item,
                                            b"can't find window: %s\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            window,
                                        );
                                    }
                                }
                                _ => {
                                    if !flags & CMD_FIND_QUIET != 0 {
                                        cmdq_error(
                                            item,
                                            b"can't find pane: %s\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            pane,
                                        );
                                    }
                                }
                            }
                            current_block = 5193823237153215208;
                        }
                    }
                }
            }
            match current_block {
                5193823237153215208 => {}
                _ => {
                    match current_block {
                        6284300254771030961 => {
                            cmd_find_copy_state(fs, (*fs).current);
                            if flags & CMD_FIND_WINDOW_INDEX != 0 {
                                (*fs).idx = -(1 as ::core::ffi::c_int);
                            }
                        }
                        _ => {}
                    }
                    (*fs).current = ::core::ptr::null_mut::<cmd_find_state>();
                    cmd_find_log_state(
                        b"cmd_find_target\0" as *const u8 as *const ::core::ffi::c_char,
                        fs,
                    );
                    free(copy as *mut ::core::ffi::c_void);
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
        _ => {}
    }
    (*fs).current = ::core::ptr::null_mut::<cmd_find_state>();
    log_debug(
        b"%s: error\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_target\0" as *const u8 as *const ::core::ffi::c_char,
    );
    free(copy as *mut ::core::ffi::c_void);
    if flags & CMD_FIND_CANFAIL != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn cmd_find_current_client(
    mut item: *mut cmdq_item,
    mut quiet: ::core::ffi::c_int,
) -> *mut client {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut found: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    if !item.is_null() {
        c = cmdq_get_client(item);
    }
    if !c.is_null() && !(*c).session.is_null() {
        return c;
    }
    found = ::core::ptr::null_mut::<client>();
    if !c.is_null() && {
        wp = cmd_find_inside_pane(c);
        !wp.is_null()
    } {
        cmd_find_clear_state(&raw mut fs, CMD_FIND_QUIET);
        fs.w = (*wp).window as *mut window;
        if cmd_find_best_session_with_window(&raw mut fs) == 0 as ::core::ffi::c_int {
            found = cmd_find_best_client(fs.s);
        }
    } else {
        s = cmd_find_best_session(
            ::core::ptr::null_mut::<*mut session>(),
            0 as u_int,
            CMD_FIND_QUIET,
        );
        if !s.is_null() {
            found = cmd_find_best_client(s);
        }
    }
    if found.is_null() && !item.is_null() && quiet == 0 {
        cmdq_error(
            item,
            b"no current client\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    log_debug(
        b"%s: no target, return %p\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_current_client\0" as *const u8 as *const ::core::ffi::c_char,
        found,
    );
    return found;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find_client(
    mut item: *mut cmdq_item,
    mut target: *const ::core::ffi::c_char,
    mut quiet: ::core::ffi::c_int,
) -> *mut client {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut size: size_t = 0;
    if target.is_null() {
        return cmd_find_current_client(item, quiet);
    }
    copy = xstrdup(target);
    size = strlen(copy);
    if size != 0 as size_t
        && *copy.offset(size.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int == ':' as i32
    {
        *copy.offset(size.wrapping_sub(1 as size_t) as isize) = '\0' as i32 as ::core::ffi::c_char;
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !(*c).session.is_null() {
            if strcmp(copy, (*c).name) == 0 as ::core::ffi::c_int {
                break;
            }
            if !(*(*c).ttyname as ::core::ffi::c_int == '\0' as i32) {
                if strcmp(copy, (*c).ttyname) == 0 as ::core::ffi::c_int {
                    break;
                }
                if !(strncmp(
                    (*c).ttyname,
                    _PATH_DEV.as_ptr(),
                    (::core::mem::size_of::<[::core::ffi::c_char; 6]>() as size_t)
                        .wrapping_sub(1 as size_t),
                ) != 0 as ::core::ffi::c_int)
                {
                    if strcmp(
                        copy,
                        (*c).ttyname
                            .offset(::core::mem::size_of::<[::core::ffi::c_char; 6]>() as usize
                                as isize)
                            .offset(-(1 as ::core::ffi::c_int as isize)),
                    ) == 0 as ::core::ffi::c_int
                    {
                        break;
                    }
                }
            }
        }
        c = (*c).entry.tqe_next;
    }
    if c.is_null() && quiet == 0 {
        cmdq_error(
            item,
            b"can't find client: %s\0" as *const u8 as *const ::core::ffi::c_char,
            copy,
        );
    }
    free(copy as *mut ::core::ffi::c_void);
    log_debug(
        b"%s: target %s, return %p\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_find_client\0" as *const u8 as *const ::core::ffi::c_char,
        target,
        c,
    );
    return c;
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

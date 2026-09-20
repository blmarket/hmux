use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
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
    pub type options_entry;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
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
    fn strspn(
        __s: *const ::core::ffi::c_char,
        __accept: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_ulong;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn strtonum(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn xsnprintf(
        _: *mut ::core::ffi::c_char,
        _: size_t,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn format_create(
        _: *mut client,
        _: *mut cmdq_item,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut format_tree;
    fn format_free(_: *mut format_tree);
    fn format_single(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut ::core::ffi::c_char;
    fn options_table_entry(_: *mut options_entry) -> *const options_table_entry;
    fn options_get(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn options_string_to_style(
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: *mut format_tree,
    ) -> *mut style;
    fn colour_tostring(_: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn colour_fromstring(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn attributes_tostring(_: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn attributes_fromstring(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    static grid_default_cell: grid_cell;
    fn utf8_set(_: *mut utf8_data, _: u_char);
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
    fn hyperlinks_put(
        _: *mut hyperlinks,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
    ) -> u_int;
    fn hyperlinks_get(
        _: *mut hyperlinks,
        _: u_int,
        _: *mut *const ::core::ffi::c_char,
        _: *mut *const ::core::ffi::c_char,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn hyperlinks_init() -> *mut hyperlinks;
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
pub type options_table_type = ::core::ffi::c_uint;
pub const OPTIONS_TABLE_COMMAND: options_table_type = 6;
pub const OPTIONS_TABLE_CHOICE: options_table_type = 5;
pub const OPTIONS_TABLE_FLAG: options_table_type = 4;
pub const OPTIONS_TABLE_COLOUR: options_table_type = 3;
pub const OPTIONS_TABLE_KEY: options_table_type = 2;
pub const OPTIONS_TABLE_NUMBER: options_table_type = 1;
pub const OPTIONS_TABLE_STRING: options_table_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_table_entry {
    pub name: *const ::core::ffi::c_char,
    pub alternative_name: *const ::core::ffi::c_char,
    pub type_0: options_table_type,
    pub scope: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
    pub minimum: u_int,
    pub maximum: u_int,
    pub choices: *mut *const ::core::ffi::c_char,
    pub default_str: *const ::core::ffi::c_char,
    pub default_num: ::core::ffi::c_longlong,
    pub default_arr: *mut *const ::core::ffi::c_char,
    pub separator: *const ::core::ffi::c_char,
    pub pattern: *const ::core::ffi::c_char,
    pub text: *const ::core::ffi::c_char,
    pub unit: *const ::core::ffi::c_char,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const STYLE_WIDTH_DEFAULT: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const STYLE_PAD_DEFAULT: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const PANE_SCROLLBARS_DEFAULT_PADDING: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_DEFAULT_WIDTH: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_CHARACTER: ::core::ffi::c_int = ' ' as i32;
pub const FORMAT_NOJOBS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
static mut style_default: style = unsafe {
    style {
        gc: grid_cell {
            data: utf8_data {
                data: [
                    ' ' as i32 as u_char,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                have: 0 as u_char,
                size: 1 as u_char,
                width: 1 as u_char,
            },
            attr: 0 as u_short,
            flags: 0 as u_char,
            fg: 8 as ::core::ffi::c_int,
            bg: 8 as ::core::ffi::c_int,
            us: 0 as ::core::ffi::c_int,
            link: 0 as u_int,
        },
        ignore: 0 as ::core::ffi::c_int,
        dim: 0 as ::core::ffi::c_int,
        fill: 8 as ::core::ffi::c_int,
        align: STYLE_ALIGN_DEFAULT,
        list: STYLE_LIST_OFF,
        range_type: STYLE_RANGE_NONE,
        range_argument: 0 as u_int,
        range_string: ::core::mem::transmute::<[u8; 16], [::core::ffi::c_char; 16]>(
            *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
        width: STYLE_WIDTH_DEFAULT,
        width_percentage: 0 as ::core::ffi::c_int,
        pad: STYLE_PAD_DEFAULT,
        default_type: STYLE_DEFAULT_BASE,
        link: 0 as u_int,
    }
};
static mut style_hyperlinks: *mut hyperlinks = ::core::ptr::null::<hyperlinks>() as *mut hyperlinks;
unsafe extern "C" fn style_set_range_string(mut sy: *mut style, mut s: *const ::core::ffi::c_char) {
    strlcpy(
        &raw mut (*sy).range_string as *mut ::core::ffi::c_char,
        s,
        ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn style_parse(
    mut sy: *mut style,
    mut base: *const grid_cell,
    mut in_0: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut saved: style = style {
        gc: grid_cell {
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
        },
        ignore: 0,
        dim: 0,
        fill: 0,
        align: STYLE_ALIGN_DEFAULT,
        list: STYLE_LIST_OFF,
        range_type: STYLE_RANGE_NONE,
        range_argument: 0,
        range_string: [0; 16],
        width: 0,
        width_percentage: 0,
        pad: 0,
        default_type: STYLE_DEFAULT_BASE,
        link: 0,
    };
    let delimiters: [::core::ffi::c_char; 4] =
        ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b" ,\n\0");
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tmp: [::core::ffi::c_char; 256] = [0; 256];
    let mut found: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: ::core::ffi::c_int = 0;
    let mut end: size_t = 0;
    let mut n: u_int = 0;
    if *in_0 as ::core::ffi::c_int == '\0' as i32 {
        return 0 as ::core::ffi::c_int;
    }
    style_copy(&raw mut saved, sy);
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"style_parse\0" as *const u8 as *const ::core::ffi::c_char,
        in_0,
    );
    loop {
        while *in_0 as ::core::ffi::c_int != '\0' as i32
            && !strchr(
                &raw const delimiters as *const ::core::ffi::c_char,
                *in_0 as ::core::ffi::c_int,
            )
            .is_null()
        {
            in_0 = in_0.offset(1);
        }
        if *in_0 as ::core::ffi::c_int == '\0' as i32 {
            current_block = 5832582820025303349;
            break;
        }
        end = strcspn(in_0, &raw const delimiters as *const ::core::ffi::c_char) as size_t;
        if end
            > (::core::mem::size_of::<[::core::ffi::c_char; 256]>() as usize)
                .wrapping_sub(1 as usize)
        {
            current_block = 6605876559004397942;
            break;
        }
        memcpy(
            &raw mut tmp as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            in_0 as *const ::core::ffi::c_void,
            end,
        );
        tmp[end as usize] = '\0' as i32 as ::core::ffi::c_char;
        log_debug(
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"style_parse\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut tmp as *mut ::core::ffi::c_char,
        );
        if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"default\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).gc.fg = (*base).fg;
            (*sy).gc.bg = (*base).bg;
            (*sy).gc.us = (*base).us;
            (*sy).gc.attr = (*base).attr;
            (*sy).gc.flags = (*base).flags;
            (*sy).link = 0 as u_int;
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"ignore\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).ignore = 1 as ::core::ffi::c_int;
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"noignore\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).ignore = 0 as ::core::ffi::c_int;
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"push-default\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).default_type = STYLE_DEFAULT_PUSH;
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"pop-default\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).default_type = STYLE_DEFAULT_POP;
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"set-default\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).default_type = STYLE_DEFAULT_SET;
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"nolist\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).list = STYLE_LIST_OFF;
        } else if strncasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"list=\0" as *const u8 as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(5 as ::core::ffi::c_int as isize),
                b"on\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*sy).list = STYLE_LIST_ON;
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(5 as ::core::ffi::c_int as isize),
                b"focus\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*sy).list = STYLE_LIST_FOCUS;
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(5 as ::core::ffi::c_int as isize),
                b"left-marker\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*sy).list = STYLE_LIST_LEFT_MARKER;
            } else {
                if !(strcasecmp(
                    (&raw mut tmp as *mut ::core::ffi::c_char)
                        .offset(5 as ::core::ffi::c_int as isize),
                    b"right-marker\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int)
                {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).list = STYLE_LIST_RIGHT_MARKER;
            }
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"norange\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).range_type = style_default.range_type;
            (*sy).range_argument = style_default.range_type as u_int;
            strlcpy(
                &raw mut (*sy).range_string as *mut ::core::ffi::c_char,
                &raw mut style_default.range_string as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
            );
        } else if end > 6 as size_t
            && strncasecmp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"range=\0" as *const u8 as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            found = strchr(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                '|' as i32,
            );
            if !found.is_null() {
                let fresh0 = found;
                found = found.offset(1);
                *fresh0 = '\0' as i32 as ::core::ffi::c_char;
                if *found as ::core::ffi::c_int == '\0' as i32 {
                    current_block = 6605876559004397942;
                    break;
                }
            }
            if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"left\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                if !found.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).range_type = STYLE_RANGE_LEFT;
                (*sy).range_argument = 0 as u_int;
                style_set_range_string(sy, b"\0" as *const u8 as *const ::core::ffi::c_char);
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"right\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                if !found.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).range_type = STYLE_RANGE_RIGHT;
                (*sy).range_argument = 0 as u_int;
                style_set_range_string(sy, b"\0" as *const u8 as *const ::core::ffi::c_char);
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"control\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                if found.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                n = strtonum(
                    found,
                    0 as ::core::ffi::c_longlong,
                    9 as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).range_type = STYLE_RANGE_CONTROL;
                (*sy).range_argument = n;
                style_set_range_string(sy, b"\0" as *const u8 as *const ::core::ffi::c_char);
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                if found.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                if *found as ::core::ffi::c_int != '%' as i32
                    || *found.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\0' as i32
                {
                    current_block = 6605876559004397942;
                    break;
                }
                n = strtonum(
                    found.offset(1 as ::core::ffi::c_int as isize),
                    0 as ::core::ffi::c_longlong,
                    UINT_MAX as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).range_type = STYLE_RANGE_PANE;
                (*sy).range_argument = n;
                style_set_range_string(sy, b"\0" as *const u8 as *const ::core::ffi::c_char);
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"window\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                if found.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                n = strtonum(
                    found,
                    0 as ::core::ffi::c_longlong,
                    UINT_MAX as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).range_type = STYLE_RANGE_WINDOW;
                (*sy).range_argument = n;
                style_set_range_string(sy, b"\0" as *const u8 as *const ::core::ffi::c_char);
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"session\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                if found.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                if *found as ::core::ffi::c_int != '$' as i32
                    || *found.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\0' as i32
                {
                    current_block = 6605876559004397942;
                    break;
                }
                n = strtonum(
                    found.offset(1 as ::core::ffi::c_int as isize),
                    0 as ::core::ffi::c_longlong,
                    UINT_MAX as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).range_type = STYLE_RANGE_SESSION;
                (*sy).range_argument = n;
                style_set_range_string(sy, b"\0" as *const u8 as *const ::core::ffi::c_char);
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"user\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                if found.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).range_type = STYLE_RANGE_USER;
                (*sy).range_argument = 0 as u_int;
                style_set_range_string(sy, found);
            }
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"noalign\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).align = style_default.align;
        } else if end > 6 as size_t
            && strncasecmp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"align=\0" as *const u8 as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"left\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*sy).align = STYLE_ALIGN_LEFT;
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"centre\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*sy).align = STYLE_ALIGN_CENTRE;
            } else if strcasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(6 as ::core::ffi::c_int as isize),
                b"right\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*sy).align = STYLE_ALIGN_RIGHT;
            } else {
                if !(strcasecmp(
                    (&raw mut tmp as *mut ::core::ffi::c_char)
                        .offset(6 as ::core::ffi::c_int as isize),
                    b"absolute-centre\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int)
                {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).align = STYLE_ALIGN_ABSOLUTE_CENTRE;
            }
        } else if end > 5 as size_t
            && strncasecmp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"fill=\0" as *const u8 as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            value = colour_fromstring(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(5 as ::core::ffi::c_int as isize),
            );
            if value == -(1 as ::core::ffi::c_int) {
                current_block = 6605876559004397942;
                break;
            }
            (*sy).fill = value;
        } else if end > 4 as size_t
            && strncasecmp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"dim=\0" as *const u8 as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            if tmp[end.wrapping_sub(1 as size_t) as usize] as ::core::ffi::c_int == '%' as i32 {
                tmp[end.wrapping_sub(1 as size_t) as usize] = '\0' as i32 as ::core::ffi::c_char;
            }
            n = strtonum(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(4 as ::core::ffi::c_int as isize),
                0 as ::core::ffi::c_longlong,
                100 as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as u_int;
            if !errstr.is_null() {
                current_block = 6605876559004397942;
                break;
            }
            (*sy).dim = n as ::core::ffi::c_int;
        } else if end > 3 as size_t
            && strncasecmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize),
                b"g=\0" as *const u8 as *const ::core::ffi::c_char,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            value = colour_fromstring(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(3 as ::core::ffi::c_int as isize),
            );
            if value == -(1 as ::core::ffi::c_int) {
                current_block = 6605876559004397942;
                break;
            }
            if *in_0 as ::core::ffi::c_int == 'f' as i32
                || *in_0 as ::core::ffi::c_int == 'F' as i32
            {
                if value != 8 as ::core::ffi::c_int {
                    (*sy).gc.fg = value;
                } else {
                    (*sy).gc.fg = (*base).fg;
                }
            } else {
                if !(*in_0 as ::core::ffi::c_int == 'b' as i32
                    || *in_0 as ::core::ffi::c_int == 'B' as i32)
                {
                    current_block = 6605876559004397942;
                    break;
                }
                if value != 8 as ::core::ffi::c_int {
                    (*sy).gc.bg = value;
                } else {
                    (*sy).gc.bg = (*base).bg;
                }
            }
        } else if end > 3 as size_t
            && strncasecmp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"us=\0" as *const u8 as *const ::core::ffi::c_char,
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            value = colour_fromstring(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(3 as ::core::ffi::c_int as isize),
            );
            if value == -(1 as ::core::ffi::c_int) {
                current_block = 6605876559004397942;
                break;
            }
            if value != 8 as ::core::ffi::c_int {
                (*sy).gc.us = value;
            } else {
                (*sy).gc.us = (*base).us;
            }
        } else if strcasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"none\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            (*sy).gc.attr = 0 as u_short;
        } else if end > 2 as size_t
            && strncasecmp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"no\0" as *const u8 as *const ::core::ffi::c_char,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            if strcmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(2 as ::core::ffi::c_int as isize),
                b"link\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*sy).link = 0 as u_int;
            } else if strcmp(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(2 as ::core::ffi::c_int as isize),
                b"attr\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
            {
                (*sy).gc.attr = ((*sy).gc.attr as ::core::ffi::c_int | GRID_ATTR_NOATTR) as u_short;
            } else {
                value = attributes_fromstring(
                    (&raw mut tmp as *mut ::core::ffi::c_char)
                        .offset(2 as ::core::ffi::c_int as isize),
                );
                if value == -(1 as ::core::ffi::c_int) {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).gc.attr = ((*sy).gc.attr as ::core::ffi::c_int & !value) as u_short;
            }
        } else if end > 6 as size_t
            && strncasecmp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"width=\0" as *const u8 as *const ::core::ffi::c_char,
                6 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            if end > 7 as size_t
                && tmp[end.wrapping_sub(1 as size_t) as usize] as ::core::ffi::c_int == '%' as i32
            {
                tmp[end.wrapping_sub(1 as size_t) as usize] = '\0' as i32 as ::core::ffi::c_char;
                n = strtonum(
                    (&raw mut tmp as *mut ::core::ffi::c_char)
                        .offset(6 as ::core::ffi::c_int as isize),
                    0 as ::core::ffi::c_longlong,
                    100 as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).width = n as ::core::ffi::c_int;
                (*sy).width_percentage = 1 as ::core::ffi::c_int;
            } else {
                n = strtonum(
                    (&raw mut tmp as *mut ::core::ffi::c_char)
                        .offset(6 as ::core::ffi::c_int as isize),
                    0 as ::core::ffi::c_longlong,
                    UINT_MAX as ::core::ffi::c_longlong,
                    &raw mut errstr,
                ) as u_int;
                if !errstr.is_null() {
                    current_block = 6605876559004397942;
                    break;
                }
                (*sy).width = n as ::core::ffi::c_int;
                (*sy).width_percentage = 0 as ::core::ffi::c_int;
            }
        } else if end > 4 as size_t
            && strncasecmp(
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"pad=\0" as *const u8 as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            n = strtonum(
                (&raw mut tmp as *mut ::core::ffi::c_char).offset(4 as ::core::ffi::c_int as isize),
                0 as ::core::ffi::c_longlong,
                UINT_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as u_int;
            if !errstr.is_null() {
                current_block = 6605876559004397942;
                break;
            }
            (*sy).pad = n as ::core::ffi::c_int;
        } else if strncasecmp(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"link=\0" as *const u8 as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            if tmp[5 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == '\0' as i32 {
                (*sy).link = 0 as u_int;
            } else {
                if style_hyperlinks.is_null() {
                    style_hyperlinks = hyperlinks_init();
                }
                (*sy).link = hyperlinks_put(
                    style_hyperlinks,
                    (&raw mut tmp as *mut ::core::ffi::c_char)
                        .offset(5 as ::core::ffi::c_int as isize),
                    (&raw mut tmp as *mut ::core::ffi::c_char)
                        .offset(5 as ::core::ffi::c_int as isize),
                );
            }
        } else {
            value = attributes_fromstring(&raw mut tmp as *mut ::core::ffi::c_char);
            if value == -(1 as ::core::ffi::c_int) {
                current_block = 6605876559004397942;
                break;
            }
            (*sy).gc.attr = ((*sy).gc.attr as ::core::ffi::c_int | value) as u_short;
        }
        in_0 = in_0.offset(end.wrapping_add(strspn(
            in_0.offset(end as isize),
            &raw const delimiters as *const ::core::ffi::c_char,
        ) as size_t) as isize);
        if !(*in_0 as ::core::ffi::c_int != '\0' as i32) {
            current_block = 5832582820025303349;
            break;
        }
    }
    match current_block {
        5832582820025303349 => return 0 as ::core::ffi::c_int,
        _ => {
            style_copy(sy, &raw mut saved);
            return -(1 as ::core::ffi::c_int);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn style_tostring(mut sy: *mut style) -> *const ::core::ffi::c_char {
    let mut gc: *mut grid_cell = &raw mut (*sy).gc;
    let mut off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut comma: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut tmp: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    static mut s: [::core::ffi::c_char; 2048] = [0; 2048];
    let mut b: [::core::ffi::c_char; 21] = [0; 21];
    *(&raw mut s as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if (*sy).list as ::core::ffi::c_uint
        != STYLE_LIST_OFF as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*sy).list as ::core::ffi::c_uint
            == STYLE_LIST_ON as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"on\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).list as ::core::ffi::c_uint
            == STYLE_LIST_FOCUS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"focus\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).list as ::core::ffi::c_uint
            == STYLE_LIST_LEFT_MARKER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"left-marker\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).list as ::core::ffi::c_uint
            == STYLE_LIST_RIGHT_MARKER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"right-marker\0" as *const u8 as *const ::core::ffi::c_char;
        }
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%slist=%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            tmp,
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*sy).range_type as ::core::ffi::c_uint
        != STYLE_RANGE_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*sy).range_type as ::core::ffi::c_uint
            == STYLE_RANGE_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"left\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).range_type as ::core::ffi::c_uint
            == STYLE_RANGE_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"right\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).range_type as ::core::ffi::c_uint
            == STYLE_RANGE_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            snprintf(
                &raw mut b as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 21]>() as size_t,
                b"pane|%%%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*sy).range_argument,
            );
            tmp = &raw mut b as *mut ::core::ffi::c_char;
        } else if (*sy).range_type as ::core::ffi::c_uint
            == STYLE_RANGE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            snprintf(
                &raw mut b as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 21]>() as size_t,
                b"window|%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*sy).range_argument,
            );
            tmp = &raw mut b as *mut ::core::ffi::c_char;
        } else if (*sy).range_type as ::core::ffi::c_uint
            == STYLE_RANGE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            snprintf(
                &raw mut b as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 21]>() as size_t,
                b"session|$%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*sy).range_argument,
            );
            tmp = &raw mut b as *mut ::core::ffi::c_char;
        } else if (*sy).range_type as ::core::ffi::c_uint
            == STYLE_RANGE_USER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            snprintf(
                &raw mut b as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 21]>() as size_t,
                b"user|%s\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut (*sy).range_string as *mut ::core::ffi::c_char,
            );
            tmp = &raw mut b as *mut ::core::ffi::c_char;
        }
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%srange=%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            tmp,
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*sy).align as ::core::ffi::c_uint
        != STYLE_ALIGN_DEFAULT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*sy).align as ::core::ffi::c_uint
            == STYLE_ALIGN_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"left\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).align as ::core::ffi::c_uint
            == STYLE_ALIGN_CENTRE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"centre\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).align as ::core::ffi::c_uint
            == STYLE_ALIGN_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"right\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).align as ::core::ffi::c_uint
            == STYLE_ALIGN_ABSOLUTE_CENTRE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"absolute-centre\0" as *const u8 as *const ::core::ffi::c_char;
        }
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%salign=%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            tmp,
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*sy).default_type as ::core::ffi::c_uint
        != STYLE_DEFAULT_BASE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*sy).default_type as ::core::ffi::c_uint
            == STYLE_DEFAULT_PUSH as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"push-default\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).default_type as ::core::ffi::c_uint
            == STYLE_DEFAULT_POP as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"pop-default\0" as *const u8 as *const ::core::ffi::c_char;
        } else if (*sy).default_type as ::core::ffi::c_uint
            == STYLE_DEFAULT_SET as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tmp = b"set-default\0" as *const u8 as *const ::core::ffi::c_char;
        }
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            tmp,
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*sy).fill != 8 as ::core::ffi::c_int {
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%sfill=%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            colour_tostring((*sy).fill),
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*sy).dim != 0 as ::core::ffi::c_int {
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%sdim=%d%%\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            (*sy).dim,
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*gc).fg != 8 as ::core::ffi::c_int {
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%sfg=%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            colour_tostring((*gc).fg),
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*gc).bg != 8 as ::core::ffi::c_int {
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%sbg=%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            colour_tostring((*gc).bg),
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*gc).us != 8 as ::core::ffi::c_int {
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%sus=%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            colour_tostring((*gc).us),
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*gc).attr as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            attributes_tostring((*gc).attr as ::core::ffi::c_int),
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*sy).width >= 0 as ::core::ffi::c_int {
        if (*sy).width_percentage != 0 {
            off += xsnprintf(
                (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
                (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                    .wrapping_sub(off as size_t),
                b"%swidth=%u%%\0" as *const u8 as *const ::core::ffi::c_char,
                comma,
                (*sy).width,
            );
        } else {
            off += xsnprintf(
                (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
                (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                    .wrapping_sub(off as size_t),
                b"%swidth=%u\0" as *const u8 as *const ::core::ffi::c_char,
                comma,
                (*sy).width,
            );
        }
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*sy).pad >= 0 as ::core::ffi::c_int {
        off += xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%spad=%u\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            (*sy).pad,
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    uri = style_link(sy);
    if !uri.is_null() {
        xsnprintf(
            (&raw mut s as *mut ::core::ffi::c_char).offset(off as isize),
            (::core::mem::size_of::<[::core::ffi::c_char; 2048]>() as size_t)
                .wrapping_sub(off as size_t),
            b"%slink=%s\0" as *const u8 as *const ::core::ffi::c_char,
            comma,
            uri,
        );
        comma = b",\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if *(&raw mut s as *mut ::core::ffi::c_char) as ::core::ffi::c_int == '\0' as i32 {
        return b"default\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return &raw mut s as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn style_link(mut sy: *mut style) -> *const ::core::ffi::c_char {
    let mut uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*sy).link == 0 as u_int || style_hyperlinks.is_null() {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if hyperlinks_get(
        style_hyperlinks,
        (*sy).link,
        &raw mut uri,
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
    ) == 0
    {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return uri;
}
#[no_mangle]
pub unsafe extern "C" fn style_add(
    mut gc: *mut grid_cell,
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut ft: *mut format_tree,
) -> *mut style {
    let mut sy: *mut style = ::core::ptr::null_mut::<style>();
    let mut ft0: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    if ft.is_null() {
        ft0 = format_create(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<cmdq_item>(),
            0 as ::core::ffi::c_int,
            FORMAT_NOJOBS,
        );
        ft = ft0;
    }
    sy = options_string_to_style(oo, name, ft);
    if sy.is_null() {
        sy = &raw mut style_default;
    }
    if (*sy).gc.fg != 8 as ::core::ffi::c_int {
        (*gc).fg = (*sy).gc.fg;
    }
    if (*sy).gc.bg != 8 as ::core::ffi::c_int {
        (*gc).bg = (*sy).gc.bg;
    }
    if (*sy).gc.us != 8 as ::core::ffi::c_int {
        (*gc).us = (*sy).gc.us;
    }
    (*gc).attr =
        ((*gc).attr as ::core::ffi::c_int | (*sy).gc.attr as ::core::ffi::c_int) as u_short;
    if !ft0.is_null() {
        format_free(ft0);
    }
    return sy;
}
#[no_mangle]
pub unsafe extern "C" fn style_apply(
    mut gc: *mut grid_cell,
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut ft: *mut format_tree,
) {
    memcpy(
        gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_add(gc, oo, name, ft);
}
#[no_mangle]
pub unsafe extern "C" fn style_parse_colour(
    mut sy: *mut style,
    mut base: *const grid_cell,
    mut s: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    style_set(sy, base);
    if *s as ::core::ffi::c_int == '\0' as i32 {
        (*sy).gc.fg = -(1 as ::core::ffi::c_int);
        return 0 as ::core::ffi::c_int;
    }
    c = colour_fromstring(s);
    if c == -(1 as ::core::ffi::c_int) {
        return -(1 as ::core::ffi::c_int);
    }
    if c == 8 as ::core::ffi::c_int {
        (*sy).gc.fg = (*base).fg;
    } else {
        (*sy).gc.fg = c;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn style_set(mut sy: *mut style, mut gc: *const grid_cell) {
    memcpy(
        sy as *mut ::core::ffi::c_void,
        &raw mut style_default as *const ::core::ffi::c_void,
        ::core::mem::size_of::<style>() as size_t,
    );
    memcpy(
        &raw mut (*sy).gc as *mut ::core::ffi::c_void,
        gc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn style_copy(mut dst: *mut style, mut src: *mut style) {
    memcpy(
        dst as *mut ::core::ffi::c_void,
        src as *const ::core::ffi::c_void,
        ::core::mem::size_of::<style>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn style_set_scrollbar_style_from_option(
    mut sb_style: *mut style,
    mut oo: *mut options,
) {
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut style: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    style_set(sb_style, &raw const grid_default_cell);
    o = options_get(
        oo,
        b"pane-scrollbars-style\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if o.is_null() {
        fatalx(b"missing pane-scrollbars-style\0" as *const u8 as *const ::core::ffi::c_char);
    }
    oe = options_table_entry(o);
    style = format_single(
        ::core::ptr::null_mut::<cmdq_item>(),
        (*oe).default_str,
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    if style_parse(sb_style, &raw const grid_default_cell, style) != 0 as ::core::ffi::c_int {
        fatalx(b"bad pane-scrollbars-style default\0" as *const u8 as *const ::core::ffi::c_char);
    }
    s = options_get_string(
        oo,
        b"pane-scrollbars-style\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if !s.is_null() {
        expanded = format_single(
            ::core::ptr::null_mut::<cmdq_item>(),
            s,
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
        if style_parse(sb_style, &raw const grid_default_cell, expanded) != 0 as ::core::ffi::c_int
        {
            style_parse(sb_style, &raw const grid_default_cell, style);
        }
        free(expanded as *mut ::core::ffi::c_void);
    }
    free(style as *mut ::core::ffi::c_void);
    if (*sb_style).width < 1 as ::core::ffi::c_int {
        (*sb_style).width = PANE_SCROLLBARS_DEFAULT_WIDTH;
    }
    if (*sb_style).pad < 0 as ::core::ffi::c_int {
        (*sb_style).pad = PANE_SCROLLBARS_DEFAULT_PADDING;
    }
    utf8_set(
        &raw mut (*sb_style).gc.data,
        PANE_SCROLLBARS_CHARACTER as u_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn style_ranges_init(mut srs: *mut style_ranges) {
    (*srs).tqh_first = ::core::ptr::null_mut::<style_range>();
    (*srs).tqh_last = &raw mut (*srs).tqh_first;
}
#[no_mangle]
pub unsafe extern "C" fn style_ranges_free(mut srs: *mut style_ranges) {
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    let mut sr1: *mut style_range = ::core::ptr::null_mut::<style_range>();
    sr = (*srs).tqh_first;
    while !sr.is_null() && {
        sr1 = (*sr).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(*sr).entry.tqe_next.is_null() {
            (*(*sr).entry.tqe_next).entry.tqe_prev = (*sr).entry.tqe_prev;
        } else {
            (*srs).tqh_last = (*sr).entry.tqe_prev;
        }
        *(*sr).entry.tqe_prev = (*sr).entry.tqe_next;
        free(sr as *mut ::core::ffi::c_void);
        sr = sr1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn style_ranges_get_range(
    mut srs: *mut style_ranges,
    mut x: u_int,
) -> *mut style_range {
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    if srs.is_null() {
        return ::core::ptr::null_mut::<style_range>();
    }
    sr = (*srs).tqh_first;
    while !sr.is_null() {
        if x >= (*sr).start && x < (*sr).end {
            return sr;
        }
        sr = (*sr).entry.tqe_next;
    }
    return ::core::ptr::null_mut::<style_range>();
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

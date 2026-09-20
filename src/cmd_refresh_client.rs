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
    pub type cmd;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn sscanf(
        __s: *const ::core::ffi::c_char,
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
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn tty_update_client_offset(_: *mut client);
    fn tty_clipboard_query(_: *mut tty);
    fn tty_set_size(_: *mut tty, _: u_int, _: u_int, _: u_int, _: u_int);
    fn tty_keys_colours(
        _: *mut tty,
        _: *const ::core::ffi::c_char,
        _: size_t,
        _: *mut size_t,
        _: *mut ::core::ffi::c_int,
        _: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn args_first_value(_: *mut args, _: u_char) -> *mut args_value;
    fn args_next_value(_: *mut args_value) -> *mut args_value;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn server_client_set_flags(_: *mut client, _: *const ::core::ffi::c_char);
    fn server_redraw_client(_: *mut client);
    fn server_status_client(_: *mut client);
    fn recalculate_sizes_now(_: ::core::ffi::c_int);
    fn window_pane_find_by_id(_: u_int) -> *mut window_pane;
    fn monitor_parse(
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
        _: *mut monitor_type,
        _: *mut ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn control_set_pane_on(_: *mut client, _: *mut window_pane);
    fn control_set_pane_off(_: *mut client, _: *mut window_pane);
    fn control_continue_pane(_: *mut client, _: *mut window_pane);
    fn control_pause_pane(_: *mut client, _: *mut window_pane);
    fn control_set_window_size(_: *mut client, _: u_int, _: u_int, _: u_int);
    fn control_clear_window_size(_: *mut client, _: u_int);
    fn control_add_sub(
        _: *mut client,
        _: *const ::core::ffi::c_char,
        _: monitor_type,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
    );
    fn control_remove_sub(_: *mut client, _: *const ::core::ffi::c_char);
    fn log_debug(_: *const ::core::ffi::c_char, ...);
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
pub type args_type = ::core::ffi::c_uint;
pub const ARGS_COMMANDS: args_type = 2;
pub const ARGS_STRING: args_type = 1;
pub const ARGS_NONE: args_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_value {
    pub type_0: args_type,
    pub c2rust_unnamed: C2RustUnnamed_36,
    pub cached: *mut ::core::ffi::c_char,
    pub entry: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub tqe_next: *mut args_value,
    pub tqe_prev: *mut *mut args_value,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_36 {
    pub string: *mut ::core::ffi::c_char,
    pub cmdlist: *mut cmd_list,
}
pub type args_parse_type = ::core::ffi::c_uint;
pub const ARGS_PARSE_COMMANDS: args_parse_type = 3;
pub const ARGS_PARSE_COMMANDS_OR_STRING: args_parse_type = 2;
pub const ARGS_PARSE_STRING: args_parse_type = 1;
pub const ARGS_PARSE_INVALID: args_parse_type = 0;
pub type args_parse_cb = Option<
    unsafe extern "C" fn(*mut args, u_int, *mut *mut ::core::ffi::c_char) -> args_parse_type,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_parse {
    pub template: *const ::core::ffi::c_char,
    pub lower: ::core::ffi::c_int,
    pub upper: ::core::ffi::c_int,
    pub cb: args_parse_cb,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_entry_flag {
    pub flag: ::core::ffi::c_char,
    pub type_0: cmd_find_type,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_entry {
    pub name: *const ::core::ffi::c_char,
    pub alias: *const ::core::ffi::c_char,
    pub args: args_parse,
    pub usage: *const ::core::ffi::c_char,
    pub source: cmd_entry_flag,
    pub target: cmd_entry_flag,
    pub flags: ::core::ffi::c_int,
    pub exec: Option<unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval>,
}
pub type monitor_type = ::core::ffi::c_uint;
pub const MONITOR_ALL_WINDOWS: monitor_type = 4;
pub const MONITOR_WINDOW: monitor_type = 3;
pub const MONITOR_ALL_PANES: monitor_type = 2;
pub const MONITOR_PANE: monitor_type = 1;
pub const MONITOR_SESSION: monitor_type = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const PANE_MINIMUM: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const WINDOW_MINIMUM: ::core::ffi::c_int = PANE_MINIMUM;
pub const WINDOW_MAXIMUM: ::core::ffi::c_int = 10000 as ::core::ffi::c_int;
pub const PANE_THEMECHANGED: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_CLIENT_TFLAG: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CLIENT_STATUSFORCE: ::core::ffi::c_int = 0x80000 as ::core::ffi::c_int;
pub const CLIENT_SIZECHANGED: ::core::ffi::c_int = 0x400000 as ::core::ffi::c_int;
pub const CLIENT_WINDOWSIZECHANGED: ::core::ffi::c_ulonglong =
    0x400000000 as ::core::ffi::c_ulonglong;
#[no_mangle]
pub static mut cmd_refresh_client_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"refresh-client\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"refresh\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"A:B:cC:Df:r:F:lLRSt:U\0" as *const u8
                as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 1 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-cDlLRSU] [-A pane:state] [-B name:what:format] [-C XxY] [-f flags] [-r pane:report] [-t target-client] [adjustment]\0"
            as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        flags: CMD_AFTERHOOK | CMD_CLIENT_TFLAG,
        exec: Some(
            cmd_refresh_client_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_refresh_client_update_subscription(
    mut tc: *mut client,
    mut value: *const ::core::ffi::c_char,
) {
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut format: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut type_0: monitor_type = MONITOR_SESSION;
    let mut id: ::core::ffi::c_int = 0;
    if monitor_parse(
        value,
        &raw mut name,
        &raw mut type_0,
        &raw mut id,
        &raw mut format,
    ) != 0 as ::core::ffi::c_int
    {
        control_remove_sub(tc, value);
        return;
    }
    control_add_sub(tc, name, type_0, id, format);
    free(name as *mut ::core::ffi::c_void);
    free(format as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn cmd_refresh_client_control_client_size(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut size: *const ::core::ffi::c_char = args_get(args, 'C' as i32 as u_char);
    let mut w: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if sscanf(
        size,
        b"@%u:%ux%u\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut w,
        &raw mut x,
        &raw mut y,
    ) == 3 as ::core::ffi::c_int
    {
        if x < WINDOW_MINIMUM as u_int
            || x > WINDOW_MAXIMUM as u_int
            || y < WINDOW_MINIMUM as u_int
            || y > WINDOW_MAXIMUM as u_int
        {
            cmdq_error(
                item,
                b"size too small or too big\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
        log_debug(
            b"%s: client %s window @%u: size %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_refresh_client_control_client_size\0" as *const u8 as *const ::core::ffi::c_char,
            (*tc).name,
            w,
            x,
            y,
        );
        control_set_window_size(tc, w, x, y);
        (*tc).flags =
            ((*tc).flags as ::core::ffi::c_ulonglong | CLIENT_WINDOWSIZECHANGED) as uint64_t;
        recalculate_sizes_now(1 as ::core::ffi::c_int);
        return CMD_RETURN_NORMAL;
    }
    if sscanf(
        size,
        b"@%u:\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut w,
    ) == 1 as ::core::ffi::c_int
    {
        log_debug(
            b"%s: client %s window @%u: no size\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_refresh_client_control_client_size\0" as *const u8 as *const ::core::ffi::c_char,
            (*tc).name,
            w,
        );
        control_clear_window_size(tc, w);
        recalculate_sizes_now(1 as ::core::ffi::c_int);
        return CMD_RETURN_NORMAL;
    }
    if sscanf(
        size,
        b"%u,%u\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut x,
        &raw mut y,
    ) != 2 as ::core::ffi::c_int
        && sscanf(
            size,
            b"%ux%u\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut x,
            &raw mut y,
        ) != 2 as ::core::ffi::c_int
    {
        cmdq_error(
            item,
            b"bad size argument\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if x < WINDOW_MINIMUM as u_int
        || x > WINDOW_MAXIMUM as u_int
        || y < WINDOW_MINIMUM as u_int
        || y > WINDOW_MAXIMUM as u_int
    {
        cmdq_error(
            item,
            b"size too small or too big\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    tty_set_size(&raw mut (*tc).tty, x, y, 0 as u_int, 0 as u_int);
    (*tc).flags |= CLIENT_SIZECHANGED as uint64_t;
    recalculate_sizes_now(1 as ::core::ffi::c_int);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_refresh_client_update_offset(
    mut tc: *mut client,
    mut value: *const ::core::ffi::c_char,
) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut split: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pane: u_int = 0;
    if *value as ::core::ffi::c_int != '%' as i32 {
        return;
    }
    copy = xstrdup(value);
    split = strchr(copy, ':' as i32);
    if !split.is_null() {
        let fresh0 = split;
        split = split.offset(1);
        *fresh0 = '\0' as i32 as ::core::ffi::c_char;
        if !(sscanf(
            copy,
            b"%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut pane,
        ) != 1 as ::core::ffi::c_int)
        {
            wp = window_pane_find_by_id(pane);
            if !wp.is_null() {
                if strcmp(split, b"on\0" as *const u8 as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    control_set_pane_on(tc, wp);
                } else if strcmp(split, b"off\0" as *const u8 as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    control_set_pane_off(tc, wp);
                } else if strcmp(
                    split,
                    b"continue\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    control_continue_pane(tc, wp);
                } else if strcmp(split, b"pause\0" as *const u8 as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    control_pause_pane(tc, wp);
                }
            }
        }
    }
    free(copy as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn cmd_refresh_report(mut tty: *mut tty, mut value: *const ::core::ffi::c_char) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut pane: u_int = 0;
    let mut fg: ::core::ffi::c_int = 0;
    let mut bg: ::core::ffi::c_int = 0;
    let mut size: size_t = 0 as size_t;
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut split: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *value as ::core::ffi::c_int != '%' as i32 {
        return;
    }
    copy = xstrdup(value);
    split = strchr(copy, ':' as i32);
    if !split.is_null() {
        let fresh1 = split;
        split = split.offset(1);
        *fresh1 = '\0' as i32 as ::core::ffi::c_char;
        if !(sscanf(
            copy,
            b"%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut pane,
        ) != 1 as ::core::ffi::c_int)
        {
            wp = window_pane_find_by_id(pane);
            if !wp.is_null() {
                fg = (*wp).control_fg;
                bg = (*wp).control_bg;
                if tty_keys_colours(
                    tty,
                    split,
                    strlen(split),
                    &raw mut size,
                    &raw mut fg,
                    &raw mut bg,
                ) == 0 as ::core::ffi::c_int
                {
                    if bg != (*wp).control_bg {
                        (*wp).flags |= PANE_THEMECHANGED;
                    }
                    (*wp).control_fg = fg;
                    (*wp).control_bg = bg;
                }
            }
        }
    }
    free(copy as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn cmd_refresh_client_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut tty: *mut tty = &raw mut (*tc).tty;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut adjust: u_int = 0;
    let mut av: *mut args_value = ::core::ptr::null_mut::<args_value>();
    if args_has(args, 'c' as i32 as u_char) != 0
        || args_has(args, 'L' as i32 as u_char) != 0
        || args_has(args, 'R' as i32 as u_char) != 0
        || args_has(args, 'U' as i32 as u_char) != 0
        || args_has(args, 'D' as i32 as u_char) != 0
    {
        if args_count(args) == 0 as u_int {
            adjust = 1 as u_int;
        } else {
            adjust = strtonum(
                args_string(args, 0 as u_int),
                1 as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as u_int;
            if !errstr.is_null() {
                cmdq_error(
                    item,
                    b"adjustment %s\0" as *const u8 as *const ::core::ffi::c_char,
                    errstr,
                );
                return CMD_RETURN_ERROR;
            }
        }
        if args_has(args, 'c' as i32 as u_char) != 0 {
            (*tc).pan_window = NULL;
        } else {
            w = (*(*(*tc).session).curw).window;
            if (*tc).pan_window != w as *mut ::core::ffi::c_void {
                (*tc).pan_window = w as *mut ::core::ffi::c_void;
                (*tc).pan_ox = (*tty).oox;
                (*tc).pan_oy = (*tty).ooy;
            }
            if args_has(args, 'L' as i32 as u_char) != 0 {
                if (*tc).pan_ox > adjust {
                    (*tc).pan_ox = (*tc).pan_ox.wrapping_sub(adjust);
                } else {
                    (*tc).pan_ox = 0 as u_int;
                }
            } else if args_has(args, 'R' as i32 as u_char) != 0 {
                (*tc).pan_ox = (*tc).pan_ox.wrapping_add(adjust);
                if (*tc).pan_ox > (*w).sx.wrapping_sub((*tty).osx) {
                    (*tc).pan_ox = (*w).sx.wrapping_sub((*tty).osx);
                }
            } else if args_has(args, 'U' as i32 as u_char) != 0 {
                if (*tc).pan_oy > adjust {
                    (*tc).pan_oy = (*tc).pan_oy.wrapping_sub(adjust);
                } else {
                    (*tc).pan_oy = 0 as u_int;
                }
            } else if args_has(args, 'D' as i32 as u_char) != 0 {
                (*tc).pan_oy = (*tc).pan_oy.wrapping_add(adjust);
                if (*tc).pan_oy > (*w).sy.wrapping_sub((*tty).osy) {
                    (*tc).pan_oy = (*w).sy.wrapping_sub((*tty).osy);
                }
            }
        }
        tty_update_client_offset(tc);
        server_redraw_client(tc);
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'l' as i32 as u_char) != 0 {
        tty_clipboard_query(&raw mut (*tc).tty);
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 'F' as i32 as u_char) != 0 {
        server_client_set_flags(tc, args_get(args, 'F' as i32 as u_char));
    }
    if args_has(args, 'f' as i32 as u_char) != 0 {
        server_client_set_flags(tc, args_get(args, 'f' as i32 as u_char));
    }
    if args_has(args, 'r' as i32 as u_char) != 0 {
        cmd_refresh_report(tty, args_get(args, 'r' as i32 as u_char));
    }
    if args_has(args, 'A' as i32 as u_char) != 0 {
        if !(!(*tc).flags & CLIENT_CONTROL as uint64_t != 0) {
            av = args_first_value(args, 'A' as i32 as u_char);
            while !av.is_null() {
                cmd_refresh_client_update_offset(tc, (*av).c2rust_unnamed.string);
                av = args_next_value(av);
            }
            return CMD_RETURN_NORMAL;
        }
    } else if args_has(args, 'B' as i32 as u_char) != 0 {
        if !(!(*tc).flags & CLIENT_CONTROL as uint64_t != 0) {
            av = args_first_value(args, 'B' as i32 as u_char);
            while !av.is_null() {
                cmd_refresh_client_update_subscription(tc, (*av).c2rust_unnamed.string);
                av = args_next_value(av);
            }
            return CMD_RETURN_NORMAL;
        }
    } else if args_has(args, 'C' as i32 as u_char) != 0 {
        if !(!(*tc).flags & CLIENT_CONTROL as uint64_t != 0) {
            return cmd_refresh_client_control_client_size(self_0, item);
        }
    } else {
        if args_has(args, 'S' as i32 as u_char) != 0 {
            (*tc).flags |= CLIENT_STATUSFORCE as uint64_t;
            server_status_client(tc);
        } else {
            (*tc).flags |= CLIENT_STATUSFORCE as uint64_t;
            server_redraw_client(tc);
        }
        return CMD_RETURN_NORMAL;
    }
    cmdq_error(
        item,
        b"not a control client\0" as *const u8 as *const ::core::ffi::c_char,
    );
    return CMD_RETURN_ERROR;
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

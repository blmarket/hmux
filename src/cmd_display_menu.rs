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
    pub type options_entry;
    fn strtol(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn checkshell(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn format_free(_: *mut format_tree);
    fn format_add(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn format_expand(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn format_single_from_target(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn format_create_from_target(_: *mut cmdq_item) -> *mut format_tree;
    fn options_table_entry(_: *mut options_entry) -> *const options_table_entry;
    fn options_get(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn options_find_choice(
        _: *const options_table_entry,
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn environ_create() -> *mut environ;
    fn environ_free(_: *mut environ);
    fn environ_put(_: *mut environ, _: *const ::core::ffi::c_char, _: ::core::ffi::c_int);
    fn tty_window_offset(
        _: *mut tty,
        _: *mut u_int,
        _: *mut u_int,
        _: *mut u_int,
        _: *mut u_int,
    ) -> ::core::ffi::c_int;
    fn args_to_vector(
        _: *mut args,
        _: *mut ::core::ffi::c_int,
        _: *mut *mut *mut ::core::ffi::c_char,
    );
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn args_first_value(_: *mut args, _: u_char) -> *mut args_value;
    fn args_next_value(_: *mut args_value) -> *mut args_value;
    fn args_strtonum(
        _: *mut args,
        _: u_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn args_percentage(
        _: *mut args,
        _: u_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn cmd_append_argv(
        _: *mut ::core::ffi::c_int,
        _: *mut *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
    );
    fn cmd_free_argv(_: ::core::ffi::c_int, _: *mut *mut ::core::ffi::c_char);
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_event(_: *mut cmdq_item) -> *mut key_event;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn key_string_lookup_string(_: *const ::core::ffi::c_char) -> key_code;
    fn server_client_clear_overlay(_: *mut client);
    fn server_client_get_cwd(_: *mut client, _: *mut session) -> *const ::core::ffi::c_char;
    fn status_at_line(_: *mut client) -> ::core::ffi::c_int;
    fn status_line_size(_: *mut client) -> u_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn menu_create(_: *const ::core::ffi::c_char) -> *mut menu;
    fn menu_add_item(
        _: *mut menu,
        _: *const menu_item,
        _: *mut cmdq_item,
        _: *mut client,
        _: *mut cmd_find_state,
    );
    fn menu_free(_: *mut menu);
    fn menu_display(
        _: *mut menu,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: *mut cmdq_item,
        _: u_int,
        _: u_int,
        _: *mut client,
        _: box_lines,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: *mut cmd_find_state,
        _: menu_choice_cb,
        _: *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int;
    fn popup_display(
        _: ::core::ffi::c_int,
        _: box_lines,
        _: *mut cmdq_item,
        _: u_int,
        _: u_int,
        _: u_int,
        _: u_int,
        _: *mut environ,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: *mut client,
        _: *mut session,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: popup_close_cb,
        _: *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int;
    fn popup_present(_: *mut client) -> ::core::ffi::c_int;
    fn popup_modify(
        _: *mut client,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: box_lines,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
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
pub type box_lines = ::core::ffi::c_int;
pub const BOX_LINES_NONE: box_lines = 6;
pub const BOX_LINES_PADDED: box_lines = 5;
pub const BOX_LINES_ROUNDED: box_lines = 4;
pub const BOX_LINES_SIMPLE: box_lines = 3;
pub const BOX_LINES_HEAVY: box_lines = 2;
pub const BOX_LINES_DOUBLE: box_lines = 1;
pub const BOX_LINES_SINGLE: box_lines = 0;
pub const BOX_LINES_DEFAULT: box_lines = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct menu_item {
    pub name: *const ::core::ffi::c_char,
    pub key: key_code,
    pub command: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct menu {
    pub title: *const ::core::ffi::c_char,
    pub items: *mut menu_item,
    pub count: u_int,
    pub width: u_int,
}
pub type menu_choice_cb =
    Option<unsafe extern "C" fn(*mut menu, u_int, key_code, *mut ::core::ffi::c_void) -> ()>;
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
pub type popup_close_cb =
    Option<unsafe extern "C" fn(::core::ffi::c_int, *mut ::core::ffi::c_void) -> ()>;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const _PATH_BSHELL: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"/bin/sh\0") };
pub const CMD_AFTERHOOK: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CMD_CLIENT_CFLAG: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const MENU_NOMOUSE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MENU_STAYOPEN: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const POPUP_CLOSEEXIT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const POPUP_CLOSEEXITZERO: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const POPUP_CLOSEANYKEY: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
#[no_mangle]
pub static mut cmd_display_menu_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"display-menu\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"menu\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"b:c:C:H:s:S:MOt:T:x:y:\0" as *const u8
                as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: Some(
                cmd_display_menu_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-MO] [-b border-lines] [-c target-client] [-C starting-choice] [-H selected-style] [-s style] [-S border-style] [-t target-pane] [-T title] [-x position] [-y position] name [key] [command] ...\0"
            as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK | CMD_CLIENT_CFLAG,
        exec: Some(
            cmd_display_menu_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_display_popup_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"display-popup\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"popup\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"Bb:Cc:d:e:Eh:kNs:S:t:T:w:x:y:\0" as *const u8
                as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: b"[-BCEkN] [-b border-lines] [-c target-client] [-d start-directory] [-e environment] [-h height] [-s style] [-S border-style] [-t target-pane] [-T title] [-w width] [-x position] [-y position] [shell-command [argument ...]]\0"
            as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: CMD_AFTERHOOK | CMD_CLIENT_CFLAG,
        exec: Some(
            cmd_display_popup_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_display_menu_args_parse(
    mut args: *mut args,
    mut idx: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> args_parse_type {
    let mut i: u_int = 0 as u_int;
    let mut type_0: args_parse_type = ARGS_PARSE_STRING;
    loop {
        type_0 = ARGS_PARSE_STRING;
        if i == idx {
            break;
        }
        let fresh0 = i;
        i = i.wrapping_add(1);
        if *args_string(args, fresh0) as ::core::ffi::c_int == '\0' as i32 {
            continue;
        }
        type_0 = ARGS_PARSE_STRING;
        let fresh1 = i;
        i = i.wrapping_add(1);
        if fresh1 == idx {
            break;
        }
        type_0 = ARGS_PARSE_COMMANDS_OR_STRING;
        let fresh2 = i;
        i = i.wrapping_add(1);
        if fresh2 == idx {
            break;
        }
    }
    return type_0;
}
unsafe extern "C" fn cmd_display_menu_get_popup_pos(
    mut tc: *mut client,
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut px: *mut u_int,
    mut py: *mut u_int,
    mut w: u_int,
    mut h: u_int,
) -> ::core::ffi::c_int {
    let mut tty: *mut tty = &raw mut (*tc).tty;
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut event: *mut key_event = cmdq_get_event(item);
    let mut s: *mut session = (*tc).session;
    let mut wl: *mut winlink = (*target).wl;
    let mut wp: *mut window_pane = (*target).wp;
    let mut ranges: *mut style_ranges = ::core::ptr::null_mut::<style_ranges>();
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    let mut xp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut yp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut top: ::core::ffi::c_int = 0;
    let mut line: u_int = 0;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut lines: u_int = 0;
    let mut position: u_int = 0;
    let mut n: ::core::ffi::c_long = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    if w > (*tty).sx || h > (*tty).sy {
        return 0 as ::core::ffi::c_int;
    }
    ft = format_create_from_target(item);
    if (*event).m.valid != 0 {
        format_add(
            ft,
            b"popup_mouse_x\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*event).m.x,
        );
        format_add(
            ft,
            b"popup_mouse_y\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*event).m.y,
        );
    }
    format_add(
        ft,
        b"popup_last_x\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*(*target).w).menu_last_px,
    );
    format_add(
        ft,
        b"popup_last_y\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*(*target).w).menu_last_py.wrapping_add(h),
    );
    top = status_at_line(tc);
    if top != -(1 as ::core::ffi::c_int) {
        lines = status_line_size(tc);
        if top == 0 as ::core::ffi::c_int {
            top = lines as ::core::ffi::c_int;
        } else {
            top = 0 as ::core::ffi::c_int;
        }
        position = options_get_number(
            (*s).options,
            b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
        ) as u_int;
        line = 0 as u_int;
        while line < lines {
            ranges = &raw mut (*(&raw mut (*tc).status.entries as *mut style_line_entry)
                .offset(line as isize))
            .ranges;
            sr = (*ranges).tqh_first;
            while !sr.is_null() {
                if !((*sr).type_0 as ::core::ffi::c_uint
                    != STYLE_RANGE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    if (*sr).argument == (*wl).idx as u_int {
                        break;
                    }
                }
                sr = (*sr).entry.tqe_next;
            }
            if !sr.is_null() {
                break;
            }
            line = line.wrapping_add(1);
        }
        if !sr.is_null() {
            format_add(
                ft,
                b"popup_window_status_line_x\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*sr).start,
            );
            if position == 0 as u_int {
                format_add(
                    ft,
                    b"popup_window_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                    b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                    line.wrapping_add(1 as u_int).wrapping_add(h),
                );
            } else {
                format_add(
                    ft,
                    b"popup_window_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                    b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                    (*tty).sy.wrapping_sub(lines).wrapping_add(line),
                );
            }
        }
        if position == 0 as u_int {
            format_add(
                ft,
                b"popup_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                lines.wrapping_add(h),
            );
        } else {
            format_add(
                ft,
                b"popup_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*tty).sy.wrapping_sub(lines),
            );
        }
    } else {
        top = 0 as ::core::ffi::c_int;
    }
    format_add(
        ft,
        b"popup_width\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    format_add(
        ft,
        b"popup_height\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        h,
    );
    n = (*tty).sx.wrapping_sub(1 as u_int) as ::core::ffi::c_long / 2 as ::core::ffi::c_long
        - w.wrapping_div(2 as u_int) as ::core::ffi::c_long;
    if n < 0 as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    } else {
        format_add(
            ft,
            b"popup_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
    }
    n = (*tty)
        .sy
        .wrapping_sub(1 as u_int)
        .wrapping_div(2 as u_int)
        .wrapping_add(h.wrapping_div(2 as u_int)) as ::core::ffi::c_long;
    if n >= (*tty).sy as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*tty).sy.wrapping_sub(h),
        );
    } else {
        format_add(
            ft,
            b"popup_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
    }
    if (*event).m.valid != 0 {
        n = (*event).m.x as ::core::ffi::c_long - w.wrapping_div(2 as u_int) as ::core::ffi::c_long;
        if n < 0 as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                n,
            );
        }
        n = (*event).m.y.wrapping_sub(h.wrapping_div(2 as u_int)) as ::core::ffi::c_long;
        if n + h as ::core::ffi::c_long >= (*tty).sy as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*tty).sy.wrapping_sub(h),
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                n,
            );
        }
        n = (*event).m.y as ::core::ffi::c_long + h as ::core::ffi::c_long;
        if n >= (*tty).sy as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_top\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*tty).sy.wrapping_sub(1 as u_int),
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_top\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                n,
            );
        }
        n = (*event).m.y.wrapping_sub(h) as ::core::ffi::c_long;
        if n < 0 as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_bottom\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_bottom\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                n,
            );
        }
    }
    tty_window_offset(
        &raw mut (*tc).tty,
        &raw mut ox,
        &raw mut oy,
        &raw mut sx,
        &raw mut sy,
    );
    n = ((top + (*wp).yoff) as u_int)
        .wrapping_sub(oy)
        .wrapping_add(h) as ::core::ffi::c_long;
    if n >= (*tty).sy as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_pane_top\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*tty).sy.wrapping_sub(h),
        );
    } else {
        format_add(
            ft,
            b"popup_pane_top\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
    }
    format_add(
        ft,
        b"popup_pane_bottom\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        ((top + (*wp).yoff) as u_int)
            .wrapping_add((*wp).sy)
            .wrapping_sub(oy),
    );
    format_add(
        ft,
        b"popup_pane_left\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        ((*wp).xoff as u_int).wrapping_sub(ox),
    );
    n = (*wp).xoff as ::core::ffi::c_long + (*wp).sx as ::core::ffi::c_long
        - ox as ::core::ffi::c_long
        - w as ::core::ffi::c_long;
    if n < 0 as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_pane_right\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    } else {
        format_add(
            ft,
            b"popup_pane_right\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
    }
    xp = args_get(args, 'x' as i32 as u_char);
    if xp.is_null()
        || strcmp(xp, b"C\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_centre_x}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"R\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_pane_right}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"P\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_pane_left}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"M\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_mouse_centre_x}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"L\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_last_x}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"W\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_window_status_line_x}\0" as *const u8 as *const ::core::ffi::c_char;
    }
    p = format_expand(ft, xp);
    n = strtol(
        p,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        10 as ::core::ffi::c_int,
    );
    if n + w as ::core::ffi::c_long >= (*tty).sx as ::core::ffi::c_long {
        n = (*tty).sx.wrapping_sub(w) as ::core::ffi::c_long;
    } else if n < 0 as ::core::ffi::c_long {
        n = 0 as ::core::ffi::c_long;
    }
    *px = n as u_int;
    log_debug(
        b"%s: -x: %s = %s = %u (-w %u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_display_menu_get_popup_pos\0" as *const u8 as *const ::core::ffi::c_char,
        xp,
        p,
        *px,
        w,
    );
    free(p as *mut ::core::ffi::c_void);
    yp = args_get(args, 'y' as i32 as u_char);
    if yp.is_null()
        || strcmp(yp, b"C\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_centre_y}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"P\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_pane_bottom}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"M\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_mouse_top}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"L\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_last_y}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"S\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_status_line_y}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"W\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_window_status_line_y}\0" as *const u8 as *const ::core::ffi::c_char;
    }
    p = format_expand(ft, yp);
    n = strtol(
        p,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        10 as ::core::ffi::c_int,
    );
    if n < h as ::core::ffi::c_long {
        n = 0 as ::core::ffi::c_long;
    } else {
        n -= h as ::core::ffi::c_long;
    }
    if n + h as ::core::ffi::c_long >= (*tty).sy as ::core::ffi::c_long {
        n = (*tty).sy.wrapping_sub(h) as ::core::ffi::c_long;
    } else if n < 0 as ::core::ffi::c_long {
        n = 0 as ::core::ffi::c_long;
    }
    *py = n as u_int;
    log_debug(
        b"%s: -y: %s = %s = %u (-h %u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_display_menu_get_popup_pos\0" as *const u8 as *const ::core::ffi::c_char,
        yp,
        p,
        *py,
        h,
    );
    free(p as *mut ::core::ffi::c_void);
    format_free(ft);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn cmd_display_menu_get_menu_pos(
    mut tc: *mut client,
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut px: *mut u_int,
    mut py: *mut u_int,
    mut w: u_int,
    mut h: u_int,
) -> ::core::ffi::c_int {
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut event: *mut key_event = cmdq_get_event(item);
    let mut s: *mut session = (*tc).session;
    let mut wl: *mut winlink = (*target).wl;
    let mut window: *mut window = (*target).w;
    let mut wp: *mut window_pane = (*target).wp;
    let mut ranges: *mut style_ranges = ::core::ptr::null_mut::<style_ranges>();
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    let mut xp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut yp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line: u_int = 0;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut lines: u_int = 0;
    let mut position: u_int = 0;
    let mut n: ::core::ffi::c_long = 0;
    let mut max_x: ::core::ffi::c_long = 0;
    let mut max_y: ::core::ffi::c_long = 0;
    let mut mouse_x: ::core::ffi::c_long = 0 as ::core::ffi::c_long;
    let mut mouse_y: ::core::ffi::c_long = 0 as ::core::ffi::c_long;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    max_x = (if (*window).sx > w {
        (*window).sx.wrapping_sub(w)
    } else {
        0 as u_int
    }) as ::core::ffi::c_long;
    max_y = (if (*window).sy > h {
        (*window).sy.wrapping_sub(h)
    } else {
        0 as u_int
    }) as ::core::ffi::c_long;
    tty_window_offset(
        &raw mut (*tc).tty,
        &raw mut ox,
        &raw mut oy,
        &raw mut sx,
        &raw mut sy,
    );
    ft = format_create_from_target(item);
    if (*event).m.valid != 0 {
        mouse_x = (*event).m.x.wrapping_add(ox) as ::core::ffi::c_long;
        if (*event).m.statusat == 0 as ::core::ffi::c_int {
            if (*event).m.y >= (*event).m.statuslines {
                mouse_y = (*event)
                    .m
                    .y
                    .wrapping_sub((*event).m.statuslines)
                    .wrapping_add(oy) as ::core::ffi::c_long;
            } else {
                mouse_y = oy as ::core::ffi::c_long;
            }
        } else if (*event).m.statusat > 0 as ::core::ffi::c_int
            && (*event).m.y >= (*event).m.statusat as u_int
        {
            mouse_y = oy.wrapping_add(sy).wrapping_sub(1 as u_int) as ::core::ffi::c_long;
        } else {
            mouse_y = (*event).m.y.wrapping_add(oy) as ::core::ffi::c_long;
        }
        format_add(
            ft,
            b"popup_mouse_x\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            mouse_x,
        );
        format_add(
            ft,
            b"popup_mouse_y\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            mouse_y,
        );
    }
    format_add(
        ft,
        b"popup_last_x\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*window).menu_last_px,
    );
    format_add(
        ft,
        b"popup_last_y\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*window).menu_last_py.wrapping_add(h),
    );
    lines = status_line_size(tc);
    position = options_get_number(
        (*s).options,
        b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    if status_at_line(tc) != -(1 as ::core::ffi::c_int) && lines != 0 as u_int {
        line = 0 as u_int;
        while line < lines {
            ranges = &raw mut (*(&raw mut (*tc).status.entries as *mut style_line_entry)
                .offset(line as isize))
            .ranges;
            sr = (*ranges).tqh_first;
            while !sr.is_null() {
                if !((*sr).type_0 as ::core::ffi::c_uint
                    != STYLE_RANGE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    if (*sr).argument == (*wl).idx as u_int {
                        break;
                    }
                }
                sr = (*sr).entry.tqe_next;
            }
            if !sr.is_null() {
                break;
            }
            line = line.wrapping_add(1);
        }
        if !sr.is_null() {
            format_add(
                ft,
                b"popup_window_status_line_x\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*sr).start.wrapping_add(ox),
            );
            if position == 0 as u_int {
                format_add(
                    ft,
                    b"popup_window_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                    b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                    h,
                );
            } else {
                format_add(
                    ft,
                    b"popup_window_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                    b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                    (*window).sy,
                );
            }
        }
        if position == 0 as u_int {
            format_add(
                ft,
                b"popup_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                h,
            );
        } else {
            format_add(
                ft,
                b"popup_status_line_y\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*window).sy,
            );
        }
    }
    format_add(
        ft,
        b"popup_width\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    format_add(
        ft,
        b"popup_height\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        h,
    );
    n = ((*window).sx as ::core::ffi::c_long - 1 as ::core::ffi::c_long) / 2 as ::core::ffi::c_long
        - w.wrapping_div(2 as u_int) as ::core::ffi::c_long;
    if n < 0 as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    } else {
        format_add(
            ft,
            b"popup_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
    }
    n = ((*window).sy as ::core::ffi::c_long - 1 as ::core::ffi::c_long) / 2 as ::core::ffi::c_long
        + h.wrapping_div(2 as u_int) as ::core::ffi::c_long;
    if n >= (*window).sy as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            max_y,
        );
    } else {
        format_add(
            ft,
            b"popup_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
    }
    if (*event).m.valid != 0 {
        n = mouse_x - w.wrapping_div(2 as u_int) as ::core::ffi::c_long;
        if n < 0 as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_centre_x\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                n,
            );
        }
        n = mouse_y - h.wrapping_div(2 as u_int) as ::core::ffi::c_long;
        if n + h as ::core::ffi::c_long >= (*window).sy as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                max_y,
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_centre_y\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                n,
            );
        }
        n = mouse_y + h as ::core::ffi::c_long;
        if n >= (*window).sy as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_top\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*window).sy.wrapping_sub(1 as u_int),
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_top\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                n,
            );
        }
        n = mouse_y - h as ::core::ffi::c_long;
        if n < 0 as ::core::ffi::c_long {
            format_add(
                ft,
                b"popup_mouse_bottom\0" as *const u8 as *const ::core::ffi::c_char,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        } else {
            format_add(
                ft,
                b"popup_mouse_bottom\0" as *const u8 as *const ::core::ffi::c_char,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                n,
            );
        }
    }
    n = ((*wp).yoff as u_int).wrapping_add(h) as ::core::ffi::c_long;
    if n >= (*window).sy as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_pane_top\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            max_y,
        );
    } else {
        format_add(
            ft,
            b"popup_pane_top\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
    }
    format_add(
        ft,
        b"popup_pane_bottom\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        ((*wp).yoff as u_int).wrapping_add((*wp).sy),
    );
    format_add(
        ft,
        b"popup_pane_left\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).xoff,
    );
    n = (*wp).xoff as ::core::ffi::c_long + (*wp).sx as ::core::ffi::c_long
        - w as ::core::ffi::c_long;
    if n < 0 as ::core::ffi::c_long {
        format_add(
            ft,
            b"popup_pane_right\0" as *const u8 as *const ::core::ffi::c_char,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    } else {
        format_add(
            ft,
            b"popup_pane_right\0" as *const u8 as *const ::core::ffi::c_char,
            b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
            n,
        );
    }
    xp = args_get(args, 'x' as i32 as u_char);
    if xp.is_null()
        || strcmp(xp, b"C\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_centre_x}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"R\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_pane_right}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"P\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_pane_left}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"M\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_mouse_centre_x}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"L\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_last_x}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(xp, b"W\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xp = b"#{popup_window_status_line_x}\0" as *const u8 as *const ::core::ffi::c_char;
    }
    p = format_expand(ft, xp);
    n = strtol(
        p,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        10 as ::core::ffi::c_int,
    );
    if n < 0 as ::core::ffi::c_long {
        n = 0 as ::core::ffi::c_long;
    }
    if n > max_x {
        n = max_x;
    }
    *px = n as u_int;
    log_debug(
        b"%s: -x: %s = %s = %u (-w %u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_display_menu_get_menu_pos\0" as *const u8 as *const ::core::ffi::c_char,
        xp,
        p,
        *px,
        w,
    );
    free(p as *mut ::core::ffi::c_void);
    yp = args_get(args, 'y' as i32 as u_char);
    if yp.is_null()
        || strcmp(yp, b"C\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_centre_y}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"P\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_pane_bottom}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"M\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_mouse_top}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"L\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_last_y}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"S\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_status_line_y}\0" as *const u8 as *const ::core::ffi::c_char;
    } else if strcmp(yp, b"W\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        yp = b"#{popup_window_status_line_y}\0" as *const u8 as *const ::core::ffi::c_char;
    }
    p = format_expand(ft, yp);
    n = strtol(
        p,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        10 as ::core::ffi::c_int,
    );
    if n < h as ::core::ffi::c_long {
        n = 0 as ::core::ffi::c_long;
    } else {
        n -= h as ::core::ffi::c_long;
    }
    if n < 0 as ::core::ffi::c_long {
        n = 0 as ::core::ffi::c_long;
    }
    if n > max_y {
        n = max_y;
    }
    *py = n as u_int;
    log_debug(
        b"%s: -y: %s = %s = %u (-h %u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_display_menu_get_menu_pos\0" as *const u8 as *const ::core::ffi::c_char,
        yp,
        p,
        *py,
        h,
    );
    free(p as *mut ::core::ffi::c_void);
    format_free(ft);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn cmd_display_menu_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut event: *mut key_event = cmdq_get_event(item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut menu: *mut menu = ::core::ptr::null_mut::<menu>();
    let mut menu_item: menu_item = menu_item {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        key: 0,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut style: *const ::core::ffi::c_char = args_get(args, 's' as i32 as u_char);
    let mut border_style: *const ::core::ffi::c_char = args_get(args, 'S' as i32 as u_char);
    let mut selected_style: *const ::core::ffi::c_char = args_get(args, 'H' as i32 as u_char);
    let mut lines: box_lines = BOX_LINES_DEFAULT;
    let mut title: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut starting_choice: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut i: u_int = 0;
    let mut count: u_int = args_count(args);
    let mut o: *mut options = (*(*(*(*target).s).curw).window).options;
    let mut oe: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if args_has(args, 'C' as i32 as u_char) != 0 {
        if strcmp(
            args_get(args, 'C' as i32 as u_char),
            b"-\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            starting_choice = -(1 as ::core::ffi::c_int);
            current_block = 1841672684692190573;
        } else {
            starting_choice = args_strtonum(
                args,
                'C' as i32 as u_char,
                0 as ::core::ffi::c_longlong,
                UINT_MAX as ::core::ffi::c_longlong,
                &raw mut cause,
            ) as ::core::ffi::c_int;
            if !cause.is_null() {
                cmdq_error(
                    item,
                    b"starting choice %s\0" as *const u8 as *const ::core::ffi::c_char,
                    cause,
                );
                current_block = 17658167438033882251;
            } else {
                current_block = 1841672684692190573;
            }
        }
    } else {
        current_block = 1841672684692190573;
    }
    match current_block {
        1841672684692190573 => {
            if args_has(args, 'T' as i32 as u_char) != 0 {
                title = format_single_from_target(item, args_get(args, 'T' as i32 as u_char));
            } else {
                title = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
            }
            menu = menu_create(title);
            free(title as *mut ::core::ffi::c_void);
            i = 0 as u_int;
            loop {
                if !(i != count) {
                    current_block = 2232869372362427478;
                    break;
                }
                let fresh3 = i;
                i = i.wrapping_add(1);
                name = args_string(args, fresh3);
                if *name as ::core::ffi::c_int == '\0' as i32 {
                    menu_add_item(menu, ::core::ptr::null::<menu_item>(), item, tc, target);
                } else if count.wrapping_sub(i) < 2 as u_int {
                    cmdq_error(
                        item,
                        b"not enough arguments\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                    current_block = 17658167438033882251;
                    break;
                } else {
                    let fresh4 = i;
                    i = i.wrapping_add(1);
                    key = args_string(args, fresh4);
                    menu_item.name = name;
                    menu_item.key = key_string_lookup_string(key);
                    let fresh5 = i;
                    i = i.wrapping_add(1);
                    menu_item.command = args_string(args, fresh5);
                    menu_add_item(menu, &raw mut menu_item, item, tc, target);
                }
            }
            match current_block {
                17658167438033882251 => {}
                _ => {
                    if menu.is_null() {
                        cmdq_error(
                            item,
                            b"invalid menu arguments\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    } else {
                        if (*menu).count == 0 as u_int {
                            current_block = 14896172439631163786;
                        } else if cmd_display_menu_get_menu_pos(
                            tc,
                            item,
                            args,
                            &raw mut px,
                            &raw mut py,
                            (*menu).width.wrapping_add(4 as u_int),
                            (*menu).count.wrapping_add(2 as u_int),
                        ) == 0
                        {
                            current_block = 14896172439631163786;
                        } else {
                            value = args_get(args, 'b' as i32 as u_char);
                            if !value.is_null() {
                                oe = options_get(
                                    o,
                                    b"menu-border-lines\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                lines = options_find_choice(
                                    options_table_entry(oe),
                                    value,
                                    &raw mut cause,
                                ) as box_lines;
                                if lines as ::core::ffi::c_int == -(1 as ::core::ffi::c_int) {
                                    cmdq_error(
                                        item,
                                        b"menu-border-lines %s\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        cause,
                                    );
                                    current_block = 17658167438033882251;
                                } else {
                                    current_block = 7245201122033322888;
                                }
                            } else {
                                current_block = 7245201122033322888;
                            }
                            match current_block {
                                17658167438033882251 => {}
                                _ => {
                                    if args_has(args, 'O' as i32 as u_char) != 0 {
                                        flags |= MENU_STAYOPEN;
                                    }
                                    if (*event).m.valid == 0
                                        && args_has(args, 'M' as i32 as u_char) == 0
                                    {
                                        flags |= MENU_NOMOUSE;
                                    }
                                    if menu_display(
                                        menu,
                                        flags,
                                        starting_choice,
                                        item,
                                        px,
                                        py,
                                        tc,
                                        lines,
                                        style,
                                        selected_style,
                                        border_style,
                                        target,
                                        None,
                                        NULL,
                                    ) != 0 as ::core::ffi::c_int
                                    {
                                        current_block = 14896172439631163786;
                                    } else {
                                        return CMD_RETURN_NORMAL;
                                    }
                                }
                            }
                        }
                        match current_block {
                            17658167438033882251 => {}
                            _ => {
                                menu_free(menu);
                                return CMD_RETURN_NORMAL;
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    free(cause as *mut ::core::ffi::c_void);
    menu_free(menu);
    return CMD_RETURN_ERROR;
}
unsafe extern "C" fn cmd_display_popup_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut s: *mut session = (*target).s;
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut tty: *mut tty = &raw mut (*tc).tty;
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut shell: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut shellcmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut style: *const ::core::ffi::c_char = args_get(args, 's' as i32 as u_char);
    let mut border_style: *const ::core::ffi::c_char = args_get(args, 'S' as i32 as u_char);
    let mut cwd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut argv: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut title: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut modify: ::core::ffi::c_int = popup_present(tc);
    let mut flags: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut argc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut lines: box_lines = BOX_LINES_DEFAULT;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut w: u_int = 0;
    let mut h: u_int = 0;
    let mut count: u_int = args_count(args);
    let mut av: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut o: *mut options = (*(*(*s).curw).window).options;
    let mut oe: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if args_has(args, 'C' as i32 as u_char) != 0 {
        server_client_clear_overlay(tc);
        return CMD_RETURN_NORMAL;
    }
    if (*tc).flags & CLIENT_CONTROL as uint64_t != 0 {
        return CMD_RETURN_NORMAL;
    }
    if modify == 0 && (*tc).overlay_draw.is_some() {
        return CMD_RETURN_NORMAL;
    }
    if modify == 0 {
        h = (*tty).sy.wrapping_div(2 as u_int);
        if args_has(args, 'h' as i32 as u_char) != 0 {
            h = args_percentage(
                args,
                'h' as i32 as u_char,
                1 as ::core::ffi::c_longlong,
                (*tty).sy as ::core::ffi::c_longlong,
                (*tty).sy as ::core::ffi::c_longlong,
                &raw mut cause,
            ) as u_int;
            if !cause.is_null() {
                cmdq_error(
                    item,
                    b"height %s\0" as *const u8 as *const ::core::ffi::c_char,
                    cause,
                );
                current_block = 1988999557336856620;
            } else {
                current_block = 17833034027772472439;
            }
        } else {
            current_block = 17833034027772472439;
        }
        match current_block {
            1988999557336856620 => {}
            _ => {
                w = (*tty).sx.wrapping_div(2 as u_int);
                if args_has(args, 'w' as i32 as u_char) != 0 {
                    w = args_percentage(
                        args,
                        'w' as i32 as u_char,
                        1 as ::core::ffi::c_longlong,
                        (*tty).sx as ::core::ffi::c_longlong,
                        (*tty).sx as ::core::ffi::c_longlong,
                        &raw mut cause,
                    ) as u_int;
                    if !cause.is_null() {
                        cmdq_error(
                            item,
                            b"width %s\0" as *const u8 as *const ::core::ffi::c_char,
                            cause,
                        );
                        current_block = 1988999557336856620;
                    } else {
                        current_block = 11042950489265723346;
                    }
                } else {
                    current_block = 11042950489265723346;
                }
                match current_block {
                    1988999557336856620 => {}
                    _ => {
                        if w > (*tty).sx {
                            w = (*tty).sx;
                        }
                        if h > (*tty).sy {
                            h = (*tty).sy;
                        }
                        if cmd_display_menu_get_popup_pos(
                            tc,
                            item,
                            args,
                            &raw mut px,
                            &raw mut py,
                            w,
                            h,
                        ) == 0
                        {
                            current_block = 6589043366517631393;
                        } else {
                            value = args_get(args, 'd' as i32 as u_char);
                            if !value.is_null() {
                                cwd = format_single_from_target(item, value);
                            } else {
                                cwd = xstrdup(server_client_get_cwd(tc, s));
                            }
                            if count == 0 as u_int {
                                shellcmd = options_get_string(
                                    (*s).options,
                                    b"default-command\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                            } else if count == 1 as u_int {
                                shellcmd = args_string(args, 0 as u_int);
                            }
                            if count <= 1 as u_int
                                && (shellcmd.is_null()
                                    || *shellcmd as ::core::ffi::c_int == '\0' as i32)
                            {
                                shellcmd = ::core::ptr::null::<::core::ffi::c_char>();
                                shell = options_get_string(
                                    (*s).options,
                                    b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                if checkshell(shell) == 0 {
                                    shell = _PATH_BSHELL.as_ptr();
                                }
                                cmd_append_argv(&raw mut argc, &raw mut argv, shell);
                            } else {
                                args_to_vector(args, &raw mut argc, &raw mut argv);
                            }
                            if args_has(args, 'e' as i32 as u_char) >= 1 as ::core::ffi::c_int {
                                env = environ_create();
                                av = args_first_value(args, 'e' as i32 as u_char);
                                while !av.is_null() {
                                    environ_put(
                                        env,
                                        (*av).c2rust_unnamed.string,
                                        0 as ::core::ffi::c_int,
                                    );
                                    av = args_next_value(av);
                                }
                            }
                            current_block = 1345366029464561491;
                        }
                    }
                }
            }
        }
    } else {
        current_block = 1345366029464561491;
    }
    match current_block {
        1345366029464561491 => {
            value = args_get(args, 'b' as i32 as u_char);
            if args_has(args, 'B' as i32 as u_char) != 0 {
                lines = BOX_LINES_NONE;
                current_block = 8151474771948790331;
            } else if !value.is_null() {
                oe = options_get(
                    o,
                    b"popup-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
                );
                lines = options_find_choice(options_table_entry(oe), value, &raw mut cause)
                    as box_lines;
                if !cause.is_null() {
                    cmdq_error(
                        item,
                        b"popup-border-lines %s\0" as *const u8 as *const ::core::ffi::c_char,
                        cause,
                    );
                    current_block = 1988999557336856620;
                } else {
                    current_block = 8151474771948790331;
                }
            } else {
                current_block = 8151474771948790331;
            }
            match current_block {
                1988999557336856620 => {}
                _ => {
                    if args_has(args, 'T' as i32 as u_char) != 0 {
                        title =
                            format_single_from_target(item, args_get(args, 'T' as i32 as u_char));
                    } else {
                        title = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                    if args_has(args, 'N' as i32 as u_char) != 0 || modify == 0 {
                        flags = 0 as ::core::ffi::c_int;
                    }
                    if args_has(args, 'E' as i32 as u_char) > 1 as ::core::ffi::c_int {
                        if flags == -(1 as ::core::ffi::c_int) {
                            flags = 0 as ::core::ffi::c_int;
                        }
                        flags |= POPUP_CLOSEEXITZERO;
                    } else if args_has(args, 'E' as i32 as u_char) != 0 {
                        if flags == -(1 as ::core::ffi::c_int) {
                            flags = 0 as ::core::ffi::c_int;
                        }
                        flags |= POPUP_CLOSEEXIT;
                    }
                    if args_has(args, 'k' as i32 as u_char) != 0 {
                        if flags == -(1 as ::core::ffi::c_int) {
                            flags = 0 as ::core::ffi::c_int;
                        }
                        flags |= POPUP_CLOSEANYKEY;
                    }
                    if modify != 0 {
                        popup_modify(tc, title, style, border_style, lines, flags);
                    } else if !(popup_display(
                        flags,
                        lines,
                        item,
                        px,
                        py,
                        w,
                        h,
                        env,
                        shellcmd,
                        argc,
                        argv,
                        cwd,
                        title,
                        tc,
                        s,
                        style,
                        border_style,
                        None,
                        NULL,
                    ) != 0 as ::core::ffi::c_int)
                    {
                        environ_free(env);
                        free(cwd as *mut ::core::ffi::c_void);
                        free(title as *mut ::core::ffi::c_void);
                        cmd_free_argv(argc, argv);
                        return CMD_RETURN_WAIT;
                    }
                    current_block = 6589043366517631393;
                }
            }
        }
        _ => {}
    }
    match current_block {
        1988999557336856620 => {
            free(cause as *mut ::core::ffi::c_void);
            cmd_free_argv(argc, argv);
            environ_free(env);
            free(cwd as *mut ::core::ffi::c_void);
            free(title as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
        _ => {
            cmd_free_argv(argc, argv);
            environ_free(env);
            free(cwd as *mut ::core::ffi::c_void);
            free(title as *mut ::core::ffi::c_void);
            return CMD_RETURN_NORMAL;
        }
    };
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

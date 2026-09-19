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
    pub type cmdq_state;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn cmd_list_free(_: *mut cmd_list);
    fn cmd_list_print(_: *const cmd_list, _: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn cmd_list_all_have(_: *mut cmd_list, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn cmd_parse_from_string(
        _: *const ::core::ffi::c_char,
        _: *mut cmd_parse_input,
    ) -> *mut cmd_parse_result;
    fn cmdq_new_state(
        _: *mut cmd_find_state,
        _: *mut key_event,
        _: ::core::ffi::c_int,
    ) -> *mut cmdq_state;
    fn cmdq_free_state(_: *mut cmdq_state);
    fn cmdq_get_command(_: *mut cmd_list, _: *mut cmdq_state) -> *mut cmdq_item;
    fn cmdq_get_callback1(
        _: *const ::core::ffi::c_char,
        _: cmdq_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut cmdq_item;
    fn cmdq_insert_after(_: *mut cmdq_item, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_append(_: *mut client, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn key_string_lookup_key(_: key_code, _: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    static mut clients: clients;
    fn server_client_set_key_table(_: *mut client, _: *const ::core::ffi::c_char);
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
}
pub type __u_char = ::core::ffi::c_uchar;
pub type __u_short = ::core::ffi::c_ushort;
pub type __u_int = ::core::ffi::c_uint;
pub type __uint8_t = u8;
pub type __uint64_t = u64;
pub type __pid_t = ::core::ffi::c_int;
pub type __time_t = ::core::ffi::c_long;
pub type __suseconds_t = ::core::ffi::c_long;
pub type u_char = __u_char;
pub type u_short = __u_short;
pub type u_int = __u_int;
pub type pid_t = __pid_t;
pub type time_t = __time_t;
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct timeval {
    pub tv_sec: __time_t,
    pub tv_usec: __suseconds_t,
}
pub type cc_t = ::core::ffi::c_uchar;
pub type speed_t = ::core::ffi::c_uint;
pub type tcflag_t = ::core::ffi::c_uint;
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
pub type uint8_t = __uint8_t;
pub type uint64_t = __uint64_t;
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
pub type bitstr_t = ::core::ffi::c_uchar;
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
pub type key_code = ::core::ffi::c_ulonglong;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct grid_cell {
    pub data: utf8_data,
    pub attr: u_short,
    pub flags: u_char,
    pub fg: ::core::ffi::c_int,
    pub bg: ::core::ffi::c_int,
    pub us: ::core::ffi::c_int,
    pub link: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct utf8_data {
    pub data: [u_char; 32],
    pub have: u_char,
    pub size: u_char,
    pub width: u_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct grid {
    pub flags: ::core::ffi::c_int,
    pub sx: u_int,
    pub sy: u_int,
    pub hscrolled: u_int,
    pub hsize: u_int,
    pub hlimit: u_int,
    pub scroll_added: u_int,
    pub scroll_collected: u_int,
    pub scroll_generation: u_int,
    pub linedata: *mut grid_line,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct grid_line {
    pub celldata: *mut grid_cell_entry,
    pub extddata: *mut grid_extd_entry,
    pub cellused: u_short,
    pub cellsize: u_short,
    pub extdsize: u_int,
    pub time: u_int,
    pub osc133_data: osc133_data,
    pub flags: u_short,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct osc133_data {
    pub prompt_col: u_short,
    pub cmd_col: u_short,
    pub out_start_col: u_short,
    pub out_end_col: u_short,
    pub exit_status: u_char,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct grid_extd_entry {
    pub data: utf8_char,
    pub attr: u_short,
    pub flags: u_char,
    pub fg: ::core::ffi::c_int,
    pub bg: ::core::ffi::c_int,
    pub us: ::core::ffi::c_int,
    pub link: u_int,
}
pub type utf8_char = u_int;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct grid_cell_entry {
    pub c2rust_unnamed: C2RustUnnamed_12,
    pub flags: u_char,
}
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
pub struct style {
    pub gc: grid_cell,
    pub ignore: ::core::ffi::c_int,
    pub dim: ::core::ffi::c_int,
    pub fill: ::core::ffi::c_int,
    pub align: style_align,
    pub list: style_list,
    pub range_type: style_range_type,
    pub range_argument: u_int,
    pub range_string: [::core::ffi::c_char; 16],
    pub width: ::core::ffi::c_int,
    pub width_percentage: ::core::ffi::c_int,
    pub pad: ::core::ffi::c_int,
    pub default_type: style_default_type,
    pub link: u_int,
}
pub type style_default_type = ::core::ffi::c_uint;
pub const STYLE_DEFAULT_SET: style_default_type = 3;
pub const STYLE_DEFAULT_POP: style_default_type = 2;
pub const STYLE_DEFAULT_PUSH: style_default_type = 1;
pub const STYLE_DEFAULT_BASE: style_default_type = 0;
pub type style_range_type = ::core::ffi::c_uint;
pub const STYLE_RANGE_CONTROL: style_range_type = 7;
pub const STYLE_RANGE_USER: style_range_type = 6;
pub const STYLE_RANGE_SESSION: style_range_type = 5;
pub const STYLE_RANGE_WINDOW: style_range_type = 4;
pub const STYLE_RANGE_PANE: style_range_type = 3;
pub const STYLE_RANGE_RIGHT: style_range_type = 2;
pub const STYLE_RANGE_LEFT: style_range_type = 1;
pub const STYLE_RANGE_NONE: style_range_type = 0;
pub type style_list = ::core::ffi::c_uint;
pub const STYLE_LIST_RIGHT_MARKER: style_list = 4;
pub const STYLE_LIST_LEFT_MARKER: style_list = 3;
pub const STYLE_LIST_FOCUS: style_list = 2;
pub const STYLE_LIST_ON: style_list = 1;
pub const STYLE_LIST_OFF: style_list = 0;
pub type style_align = ::core::ffi::c_uint;
pub const STYLE_ALIGN_ABSOLUTE_CENTRE: style_align = 4;
pub const STYLE_ALIGN_RIGHT: style_align = 3;
pub const STYLE_ALIGN_CENTRE: style_align = 2;
pub const STYLE_ALIGN_LEFT: style_align = 1;
pub const STYLE_ALIGN_DEFAULT: style_align = 0;
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
pub struct style_line_entry {
    pub expanded: *mut ::core::ffi::c_char,
    pub ranges: style_ranges,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct style_ranges {
    pub tqh_first: *mut style_range,
    pub tqh_last: *mut *mut style_range,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct style_range {
    pub type_0: style_range_type,
    pub argument: u_int,
    pub string: [::core::ffi::c_char; 16],
    pub start: u_int,
    pub end: u_int,
    pub entry: C2RustUnnamed_29,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_29 {
    pub tqe_next: *mut style_range,
    pub tqe_prev: *mut *mut style_range,
}
pub type client_theme = ::core::ffi::c_uint;
pub const THEME_DARK: client_theme = 2;
pub const THEME_LIGHT: client_theme = 1;
pub const THEME_UNKNOWN: client_theme = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct colour_palette {
    pub fg: ::core::ffi::c_int,
    pub bg: ::core::ffi::c_int,
    pub palette: *mut ::core::ffi::c_int,
    pub default_palette: *mut ::core::ffi::c_int,
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
pub type cmd_retval = ::core::ffi::c_int;
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub type cmd_parse_status = ::core::ffi::c_uint;
pub const CMD_PARSE_SUCCESS: cmd_parse_status = 1;
pub const CMD_PARSE_ERROR: cmd_parse_status = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_parse_result {
    pub status: cmd_parse_status,
    pub cmdlist: *mut cmd_list,
    pub error: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_parse_input {
    pub flags: ::core::ffi::c_int,
    pub file: *const ::core::ffi::c_char,
    pub line: u_int,
    pub item: *mut cmdq_item,
    pub c: *mut client,
    pub fs: cmd_find_state,
}
pub type cmdq_cb =
    Option<unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct clients {
    pub tqh_first: *mut client,
    pub tqh_last: *mut *mut client,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_tables {
    pub rbh_root: *mut key_table,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const KEYC_MASK_FLAGS: ::core::ffi::c_ulonglong = 0xff000000000000 as ::core::ffi::c_ulonglong;
pub const CMDQ_STATE_REPEAT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_READONLY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CLIENT_READONLY: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const KEY_BINDING_REPEAT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
unsafe extern "C" fn key_bindings_RB_NEXT(mut elm: *mut key_binding) -> *mut key_binding {
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
unsafe extern "C" fn key_bindings_RB_FIND(
    mut head: *mut key_bindings,
    mut elm: *mut key_binding,
) -> *mut key_binding {
    let mut tmp: *mut key_binding = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = key_bindings_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<key_binding>();
}
unsafe extern "C" fn key_bindings_RB_INSERT(
    mut head: *mut key_bindings,
    mut elm: *mut key_binding,
) -> *mut key_binding {
    let mut tmp: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut parent: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = key_bindings_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<key_binding>();
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
    key_bindings_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<key_binding>();
}
unsafe extern "C" fn key_bindings_RB_MINMAX(
    mut head: *mut key_bindings,
    mut val: ::core::ffi::c_int,
) -> *mut key_binding {
    let mut tmp: *mut key_binding = (*head).rbh_root;
    let mut parent: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
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
unsafe extern "C" fn key_bindings_RB_INSERT_COLOR(
    mut head: *mut key_bindings,
    mut elm: *mut key_binding,
) {
    let mut parent: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut gparent: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut tmp: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
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
unsafe extern "C" fn key_bindings_RB_REMOVE_COLOR(
    mut head: *mut key_bindings,
    mut parent: *mut key_binding,
    mut elm: *mut key_binding,
) {
    let mut tmp: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
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
                    let mut oleft: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
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
                    let mut oright: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
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
unsafe extern "C" fn key_bindings_RB_REMOVE(
    mut head: *mut key_bindings,
    mut elm: *mut key_binding,
) -> *mut key_binding {
    let mut current_block: u64;
    let mut child: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut parent: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut old: *mut key_binding = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
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
        current_block = 13343809724831086129;
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
        key_bindings_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn key_tables_RB_INSERT(
    mut head: *mut key_tables,
    mut elm: *mut key_table,
) -> *mut key_table {
    let mut tmp: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut parent: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = key_table_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<key_table>();
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
    key_tables_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<key_table>();
}
unsafe extern "C" fn key_tables_RB_INSERT_COLOR(
    mut head: *mut key_tables,
    mut elm: *mut key_table,
) {
    let mut parent: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut gparent: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut tmp: *mut key_table = ::core::ptr::null_mut::<key_table>();
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
unsafe extern "C" fn key_tables_RB_REMOVE_COLOR(
    mut head: *mut key_tables,
    mut parent: *mut key_table,
    mut elm: *mut key_table,
) {
    let mut tmp: *mut key_table = ::core::ptr::null_mut::<key_table>();
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
                    let mut oleft: *mut key_table = ::core::ptr::null_mut::<key_table>();
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
                    let mut oright: *mut key_table = ::core::ptr::null_mut::<key_table>();
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
unsafe extern "C" fn key_tables_RB_FIND(
    mut head: *mut key_tables,
    mut elm: *mut key_table,
) -> *mut key_table {
    let mut tmp: *mut key_table = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = key_table_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<key_table>();
}
unsafe extern "C" fn key_tables_RB_MINMAX(
    mut head: *mut key_tables,
    mut val: ::core::ffi::c_int,
) -> *mut key_table {
    let mut tmp: *mut key_table = (*head).rbh_root;
    let mut parent: *mut key_table = ::core::ptr::null_mut::<key_table>();
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
unsafe extern "C" fn key_tables_RB_REMOVE(
    mut head: *mut key_tables,
    mut elm: *mut key_table,
) -> *mut key_table {
    let mut current_block: u64;
    let mut child: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut parent: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut old: *mut key_table = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut key_table = ::core::ptr::null_mut::<key_table>();
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
        current_block = 1672104530829368166;
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
        key_tables_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn key_tables_RB_NEXT(mut elm: *mut key_table) -> *mut key_table {
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
static mut key_tables: key_tables = key_tables {
    rbh_root: ::core::ptr::null::<key_table>() as *mut key_table,
};
unsafe extern "C" fn key_table_cmp(
    mut table1: *mut key_table,
    mut table2: *mut key_table,
) -> ::core::ffi::c_int {
    return strcmp((*table1).name, (*table2).name);
}
unsafe extern "C" fn key_bindings_cmp(
    mut bd1: *mut key_binding,
    mut bd2: *mut key_binding,
) -> ::core::ffi::c_int {
    if (*bd1).key < (*bd2).key {
        return -(1 as ::core::ffi::c_int);
    }
    if (*bd1).key > (*bd2).key {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn key_bindings_free(mut bd: *mut key_binding) {
    cmd_list_free((*bd).cmdlist);
    free((*bd).note as *mut ::core::ffi::c_void);
    free(bd as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_get_table(
    mut name: *const ::core::ffi::c_char,
    mut create: ::core::ffi::c_int,
) -> *mut key_table {
    let mut table_find: key_table = key_table {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        activity_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        key_bindings: key_bindings {
            rbh_root: ::core::ptr::null_mut::<key_binding>(),
        },
        default_key_bindings: key_bindings {
            rbh_root: ::core::ptr::null_mut::<key_binding>(),
        },
        references: 0,
        entry: C2RustUnnamed_31 {
            rbe_left: ::core::ptr::null_mut::<key_table>(),
            rbe_right: ::core::ptr::null_mut::<key_table>(),
            rbe_parent: ::core::ptr::null_mut::<key_table>(),
            rbe_color: 0,
        },
    };
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    table_find.name = name;
    table = key_tables_RB_FIND(&raw mut key_tables, &raw mut table_find);
    if !table.is_null() || create == 0 {
        return table;
    }
    table = xmalloc(::core::mem::size_of::<key_table>() as size_t) as *mut key_table;
    (*table).name = xstrdup(name);
    (*table).key_bindings.rbh_root = ::core::ptr::null_mut::<key_binding>();
    (*table).default_key_bindings.rbh_root = ::core::ptr::null_mut::<key_binding>();
    (*table).references = 1 as u_int;
    key_tables_RB_INSERT(&raw mut key_tables, table);
    return table;
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_first_table() -> *mut key_table {
    return key_tables_RB_MINMAX(&raw mut key_tables, RB_NEGINF);
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_next_table(mut table: *mut key_table) -> *mut key_table {
    return key_tables_RB_NEXT(table);
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_unref_table(mut table: *mut key_table) {
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut bd1: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    (*table).references = (*table).references.wrapping_sub(1);
    if (*table).references != 0 as u_int {
        return;
    }
    bd = key_bindings_RB_MINMAX(&raw mut (*table).key_bindings, RB_NEGINF);
    while !bd.is_null() && {
        bd1 = key_bindings_RB_NEXT(bd);
        1 as ::core::ffi::c_int != 0
    } {
        key_bindings_RB_REMOVE(&raw mut (*table).key_bindings, bd);
        key_bindings_free(bd);
        bd = bd1;
    }
    bd = key_bindings_RB_MINMAX(&raw mut (*table).default_key_bindings, RB_NEGINF);
    while !bd.is_null() && {
        bd1 = key_bindings_RB_NEXT(bd);
        1 as ::core::ffi::c_int != 0
    } {
        key_bindings_RB_REMOVE(&raw mut (*table).default_key_bindings, bd);
        key_bindings_free(bd);
        bd = bd1;
    }
    free((*table).name as *mut ::core::ffi::c_void);
    free(table as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_get(
    mut table: *mut key_table,
    mut key: key_code,
) -> *mut key_binding {
    let mut bd: key_binding = key_binding {
        key: 0,
        cmdlist: ::core::ptr::null_mut::<cmd_list>(),
        note: ::core::ptr::null::<::core::ffi::c_char>(),
        tablename: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: 0,
        entry: C2RustUnnamed_32 {
            rbe_left: ::core::ptr::null_mut::<key_binding>(),
            rbe_right: ::core::ptr::null_mut::<key_binding>(),
            rbe_parent: ::core::ptr::null_mut::<key_binding>(),
            rbe_color: 0,
        },
    };
    bd.key = key;
    return key_bindings_RB_FIND(&raw mut (*table).key_bindings, &raw mut bd);
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_get_default(
    mut table: *mut key_table,
    mut key: key_code,
) -> *mut key_binding {
    let mut bd: key_binding = key_binding {
        key: 0,
        cmdlist: ::core::ptr::null_mut::<cmd_list>(),
        note: ::core::ptr::null::<::core::ffi::c_char>(),
        tablename: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: 0,
        entry: C2RustUnnamed_32 {
            rbe_left: ::core::ptr::null_mut::<key_binding>(),
            rbe_right: ::core::ptr::null_mut::<key_binding>(),
            rbe_parent: ::core::ptr::null_mut::<key_binding>(),
            rbe_color: 0,
        },
    };
    bd.key = key;
    return key_bindings_RB_FIND(&raw mut (*table).default_key_bindings, &raw mut bd);
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_first(mut table: *mut key_table) -> *mut key_binding {
    return key_bindings_RB_MINMAX(&raw mut (*table).key_bindings, RB_NEGINF);
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_next(
    mut table: *mut key_table,
    mut bd: *mut key_binding,
) -> *mut key_binding {
    return key_bindings_RB_NEXT(bd);
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_add(
    mut name: *const ::core::ffi::c_char,
    mut key: key_code,
    mut note: *const ::core::ffi::c_char,
    mut repeat: ::core::ffi::c_int,
    mut cmdlist: *mut cmd_list,
) {
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    table = key_bindings_get_table(name, 1 as ::core::ffi::c_int);
    bd = key_bindings_get(table, key & !KEYC_MASK_FLAGS);
    if cmdlist.is_null() {
        if !bd.is_null() {
            if !note.is_null() {
                free((*bd).note as *mut ::core::ffi::c_void);
                (*bd).note = xstrdup(note);
            }
            if repeat != 0 {
                (*bd).flags |= KEY_BINDING_REPEAT;
            }
        }
        return;
    }
    if !bd.is_null() {
        key_bindings_RB_REMOVE(&raw mut (*table).key_bindings, bd);
        key_bindings_free(bd);
    }
    bd = xcalloc(1 as size_t, ::core::mem::size_of::<key_binding>() as size_t) as *mut key_binding;
    (*bd).key = (key as ::core::ffi::c_ulonglong & !KEYC_MASK_FLAGS) as key_code;
    (*bd).tablename = (*table).name;
    if !note.is_null() {
        (*bd).note = xstrdup(note);
    }
    key_bindings_RB_INSERT(&raw mut (*table).key_bindings, bd);
    if repeat != 0 {
        (*bd).flags |= KEY_BINDING_REPEAT;
    }
    (*bd).cmdlist = cmdlist;
    s = cmd_list_print((*bd).cmdlist, 0 as ::core::ffi::c_int);
    log_debug(
        b"%s: %#llx %s = %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"key_bindings_add\0" as *const u8 as *const ::core::ffi::c_char,
        (*bd).key,
        key_string_lookup_key((*bd).key, 1 as ::core::ffi::c_int),
        s,
    );
    free(s as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_remove(
    mut name: *const ::core::ffi::c_char,
    mut key: key_code,
) {
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    table = key_bindings_get_table(name, 0 as ::core::ffi::c_int);
    if table.is_null() {
        return;
    }
    bd = key_bindings_get(table, key & !KEYC_MASK_FLAGS);
    if bd.is_null() {
        return;
    }
    log_debug(
        b"%s: %#llx %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"key_bindings_remove\0" as *const u8 as *const ::core::ffi::c_char,
        (*bd).key,
        key_string_lookup_key((*bd).key, 1 as ::core::ffi::c_int),
    );
    key_bindings_RB_REMOVE(&raw mut (*table).key_bindings, bd);
    key_bindings_free(bd);
    if (*table).key_bindings.rbh_root.is_null() && (*table).default_key_bindings.rbh_root.is_null()
    {
        key_tables_RB_REMOVE(&raw mut key_tables, table);
        key_bindings_unref_table(table);
    }
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_reset(
    mut name: *const ::core::ffi::c_char,
    mut key: key_code,
) {
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut dd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    table = key_bindings_get_table(name, 0 as ::core::ffi::c_int);
    if table.is_null() {
        return;
    }
    bd = key_bindings_get(table, key & !KEYC_MASK_FLAGS);
    if bd.is_null() {
        return;
    }
    dd = key_bindings_get_default(table, (*bd).key);
    if dd.is_null() {
        key_bindings_remove(name, (*bd).key);
        return;
    }
    cmd_list_free((*bd).cmdlist);
    (*bd).cmdlist = (*dd).cmdlist;
    (*(*bd).cmdlist).references += 1;
    free((*bd).note as *mut ::core::ffi::c_void);
    if !(*dd).note.is_null() {
        (*bd).note = xstrdup((*dd).note);
    } else {
        (*bd).note = ::core::ptr::null::<::core::ffi::c_char>();
    }
    (*bd).flags = (*dd).flags;
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_remove_table(mut name: *const ::core::ffi::c_char) {
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    table = key_bindings_get_table(name, 0 as ::core::ffi::c_int);
    if !table.is_null() {
        key_tables_RB_REMOVE(&raw mut key_tables, table);
        c = clients.tqh_first;
        while !c.is_null() {
            if (*c).keytable == table {
                server_client_set_key_table(c, ::core::ptr::null::<::core::ffi::c_char>());
            }
            c = (*c).entry.tqe_next;
        }
        key_bindings_unref_table(table);
    }
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_reset_table(mut name: *const ::core::ffi::c_char) {
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut bd1: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    table = key_bindings_get_table(name, 0 as ::core::ffi::c_int);
    if table.is_null() {
        return;
    }
    if (*table).default_key_bindings.rbh_root.is_null() {
        key_bindings_remove_table(name);
        return;
    }
    bd = key_bindings_RB_MINMAX(&raw mut (*table).key_bindings, RB_NEGINF);
    while !bd.is_null() && {
        bd1 = key_bindings_RB_NEXT(bd);
        1 as ::core::ffi::c_int != 0
    } {
        key_bindings_reset(name, (*bd).key);
        bd = bd1;
    }
}
unsafe extern "C" fn key_bindings_init_done(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut new_bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    table = key_tables_RB_MINMAX(&raw mut key_tables, RB_NEGINF);
    while !table.is_null() {
        bd = key_bindings_RB_MINMAX(&raw mut (*table).key_bindings, RB_NEGINF);
        while !bd.is_null() {
            new_bd = xcalloc(1 as size_t, ::core::mem::size_of::<key_binding>() as size_t)
                as *mut key_binding;
            (*new_bd).key = (*bd).key;
            if !(*bd).note.is_null() {
                (*new_bd).note = xstrdup((*bd).note);
            }
            (*new_bd).flags = (*bd).flags;
            (*new_bd).cmdlist = (*bd).cmdlist;
            (*(*new_bd).cmdlist).references += 1;
            key_bindings_RB_INSERT(&raw mut (*table).default_key_bindings, new_bd);
            bd = key_bindings_RB_NEXT(bd);
        }
        table = key_tables_RB_NEXT(table);
    }
    return CMD_RETURN_NORMAL;
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_init() {
    static mut defaults: [*const ::core::ffi::c_char; 308] = [
        b"bind -N 'Send the prefix key' C-b { send-prefix }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Rotate through the panes' C-o { rotate-window }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Suspend the current client' C-z { suspend-client }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select next layout' Space { next-layout }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Break pane to a new window' ! { break-pane }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Split window vertically' '\"' { split-window }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'List all paste buffers' '#' { list-buffers }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Rename current session' '$' { command-prompt -I'#S' { rename-session -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Split window horizontally' % { split-window -h }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Kill current window' & { confirm-before -p\"kill-window #W? (y/n)\" kill-window }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Prompt for window index to select' \"'\" { command-prompt -pindex { select-window -t ':%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'New floating pane' * { new-pane }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Toggle pane between floating and tiled' @ { if -F '#{pane_floating_flag}' { join-pane } { break-pane -W } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Switch to previous client' ( { switch-client -p }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Switch to next client' ) { switch-client -n }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Rename current window' , { command-prompt -I'#W' { rename-window -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Delete the most recent paste buffer' - { delete-buffer }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Move the current window' . { command-prompt { move-window -t '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Describe key binding' '/' { command-prompt -kpkey  { list-keys -1N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Select window 0' 0 { select-window -t:=0 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 1' 1 { select-window -t:=1 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 2' 2 { select-window -t:=2 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 3' 3 { select-window -t:=3 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 4' 4 { select-window -t:=4 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 5' 5 { select-window -t:=5 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 6' 6 { select-window -t:=6 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 7' 7 { select-window -t:=7 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 8' 8 { select-window -t:=8 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select window 9' 9 { select-window -t:=9 }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Prompt for a command' : { command-prompt }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Move to the previously active pane' \\; { last-pane }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Choose a paste buffer from a list' = { choose-buffer -Z }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'List key bindings' ? { list-keys -N }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Choose and detach a client from a list' D { choose-client -Z }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Spread panes out evenly' E { select-layout -E }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Switch to the last client' L { switch-client -l }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Clear the marked pane' M { select-pane -M }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Change the pane title' T { command-prompt -I'#T' { select-pane -T '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Enter copy mode' [ { copy-mode }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Paste the most recent paste buffer' ] { paste-buffer -p }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Create a new window' c { new-window }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Detach the current client' d { detach-client }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Search for a pane' f { command-prompt { find-window -Z -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Display window information' i { display-message }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select the previously current window' l { last-window }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Toggle the marked pane' m { select-pane -m }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select the next window' n { next-window }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select the next pane' o { select-pane -t:.+ }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Customize options' C { customize-mode -Z }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select the previous window' p { previous-window }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Display pane numbers' q { display-panes }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Redraw the current client' r { refresh-client }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Choose a session from a list' s { choose-tree -Zs }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Show a clock' t { clock-mode }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Switch to a window' Tab { new-pane -E -x75% -y30% -X0 -Y0; move-pane -P bottom-centre; switch-mode -wk }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Switch to a session' BTab { new-pane -E -x75% -y30% -X0 -Y0; move-pane -P bottom-centre; switch-mode -sk }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Choose a window from a list' w { choose-tree -Zw }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Kill the active pane' x { confirm-before -p\"kill-pane #P? (y/n)\" kill-pane }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Zoom the active pane' z { resize-pane -Z }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Swap the active pane with the pane above' '{' { swap-pane -U }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Swap the active pane with the pane below' '}' { swap-pane -D }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Show messages' '~' { show-messages }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Enter copy mode and scroll up' PPage { copy-mode -u }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Select the pane above the active pane' -r Up { select-pane -U }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Select the pane below the active pane' -r Down { select-pane -D }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Select the pane to the left of the active pane' -r Left { select-pane -L }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Select the pane to the right of the active pane' -r Right { select-pane -R }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Set the even-horizontal layout' M-1 { select-layout even-horizontal }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Set the even-vertical layout' M-2 { select-layout even-vertical }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Set the main-horizontal layout' M-3 { select-layout main-horizontal }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Set the main-vertical layout' M-4 { select-layout main-vertical }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Select the tiled layout' M-5 { select-layout tiled }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Set the main-horizontal-mirrored layout' M-6 { select-layout main-horizontal-mirrored }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Set the main-vertical-mirrored layout' M-7 { select-layout main-vertical-mirrored }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Select the next window with an alert' M-n { next-window -a }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Rotate through the panes in reverse' M-o { rotate-window -D }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Select the previous window with an alert' M-p { previous-window -a }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Move the visible part of the window up' -r S-Up { refresh-client -U 10 }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Move the visible part of the window down' -r S-Down { refresh-client -D 10 }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Move the visible part of the window left' -r S-Left { refresh-client -L 10 }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Move the visible part of the window right' -r S-Right { refresh-client -R 10 }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Reset so the visible part of the window follows the cursor' -r DC { refresh-client -c }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Resize the pane up by 5' -r M-Up if -F '#{?floating_pane_flag}' { resizep -D-5 } { resize-pane -U 5 }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Resize the pane down by 5' -r M-Down { resize-pane -D 5 }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Resize the pane left by 5' -r M-Left if -F '#{?floating_pane_flag}' { resizep -R-5 } { resize-pane -L 5 }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Resize the pane right by 5' -r M-Right resize-pane -R 5\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Resize the pane up' -r C-Up if -F '#{?floating_pane_flag}' { resizep -D-1 } { resize-pane -U }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Resize the pane down' -r C-Down { resize-pane -D }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Resize the pane left' -r C-Left if -F '#{?floating_pane_flag}' { resizep -R-1 } { resize-pane -L }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Resize the pane right' -r C-Right { resize-pane -R }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -N 'Move a floating pane' g { switch-client -Tmove }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to top-left corner' 1 { move-pane -P top-left }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to top-right corner' 2 { move-pane -P top-right }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to bottom-left corner' 3 { move-pane -P bottom-left }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to bottom-right corner' 4 { move-pane -P bottom-right }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to top-left corner and resize' M-1 { resize-pane -x50% -y50%; move-pane -P top-left }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to top-right corner and resize' M-2 { resize-pane -x50% -y50%; move-pane -P top-right }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to bottom-left corner and resize' M-3 { resize-pane -x50% -y50%; move-pane -P bottom-left }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to bottom-right corner and resize' M-4 { resize-pane -x50% -y50%; move-pane -P bottom-right }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to top' 'Up' { move-pane -P top-centre }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to bottom' 'Down' { move-pane -P bottom-centre  }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to left' 'Left' { move-pane -P centre-left }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to right' 'Right' { move-pane -P centre-right }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to top and resize' 'M-Up' { resizep -x100% -y50%; move-pane -P top-centre }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to bottom and resize' 'M-Down' { resizep -x100% -y50%; move-pane -P bottom-centre  }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to left and resize' 'M-Left' { resizep -x50% -y100%; move-pane -P centre-left }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to right and resize' 'M-Right' { resizep -x50% -y100%; move-pane -P centre-right }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Move pane to fill the window' 0 { resize-pane -x100% -y100%; move-pane -P top-left }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Display move menu' , { if -F '#{pane_floating_flag}' { display-menu -xP -yP -T '#[align=centre]Move'  'Centre' 'c' {move-pane -P centre} '' 'Top Left' '1' {move-pane -P top-left} 'Top Right' '2' {move-pane -P top-right} 'Bottom Left' '3' {move-pane -P bottom-left} 'Bottom Right' '4' {move-pane -P bottom-right} '' 'Top' 't' {move-pane -P top-centre} 'Bottom' 'b' {move-pane -P bottom-centre} 'Left' 'l' {move-pane -P centre-left} 'Right' 'r' {move-pane -P centre-right} } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tmove -N 'Display move and resize menu' . { if -F '#{pane_floating_flag}' { display-menu -xP -yP -T '#[align=centre]Move & Resize'  'Fill' '0' {resize-pane -x100% -y100%; move-pane -P top-left} '' 'Top Left' '1' {resize-pane -x50% -y50%; move-pane -P top-left} 'Top Right' '2' {resize-pane -x50% -y50%; move-pane -P top-right} 'Bottom Left' '3' {resize-pane -x50% -y50%; move-pane -P bottom-left} 'Bottom Right' '4' {resize-pane -x50% -y50%; move-pane -P bottom-right} '' 'Top' 't' {resize-pane -x100% -y50%; move-pane -P top-centre} 'Bottom' 'b' {resize-pane -x100% -y50%; move-pane -P bottom-centre} 'Left' 'l' {resize-pane -x50% -y100%; move-pane -P centre-left} 'Right' 'r' {resize-pane -x50% -y100%; move-pane -P centre-right} } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Display window menu' < { display-menu -xW -yW -T '#[align=centre]#{window_index}:#{window_name}'  '#{?#{>:#{session_windows},1},,-}Swap Left' 'l' {swap-window -t:-1} '#{?#{>:#{session_windows},1},,-}Swap Right' 'r' {swap-window -t:+1} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-window} '' 'Kill' 'X' {kill-window} 'Respawn' 'R' {respawn-window -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} 'Rename' 'n' {command-prompt -FI \"#W\" {rename-window -t '#{window_id}' -- '%%'}} '' 'New After' 'w' {new-window -a} 'New At End' 'W' {new-window} }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -N 'Display pane menu' > { display-menu -xP -yP -T '#[align=centre]#{pane_index} (#{pane_id})'  '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Top,}' '<' {send -X history-top} '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Bottom,}' '>' {send -X history-bottom} '' '#{?#{==:#{pane_mode},copy-mode},#{?copy_line_numbers,Hide Line Numbers,Show Line Numbers},}' 'L' {send -X line-numbers-toggle} '#{?#{==:#{pane_mode},copy-mode},#{?refresh_active,Refresh Off,Refresh On},}' 'r' {send -X refresh-toggle} '' '#{?#{&&:#{buffer_size},#{!:#{pane_in_mode}}},Paste #[underscore]#{=/9/...:buffer_sample},}' 'p' {paste-buffer} '' '#{?mouse_word,Search For #[underscore]#{=/9/...:mouse_word},}' 'C-r' {if -F '#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}' 'copy-mode -t='; send -Xt= search-backward -- \"#{q:mouse_word}\"} '#{?mouse_word,Type #[underscore]#{=/9/...:mouse_word},}' 'C-y' {copy-mode -q; send-keys -l -- \"#{q:mouse_word}\"} '#{?mouse_word,Copy #[underscore]#{=/9/...:mouse_word},}' 'c' {copy-mode -q; set-buffer -- \"#{q:mouse_word}\"} '#{?mouse_line,Copy Line,}' 'l' {copy-mode -q; set-buffer -- \"#{q:mouse_line}\"} '' '#{?mouse_hyperlink,Type #[underscore]#{=/9/...:mouse_hyperlink},}' 'C-h' {copy-mode -q; send-keys -l -- \"#{q:mouse_hyperlink}\"} '#{?mouse_hyperlink,Copy #[underscore]#{=/9/...:mouse_hyperlink},}' 'h' {copy-mode -q; set-buffer -- \"#{q:mouse_hyperlink}\"} '' '#{?#{#{pane_floating_flag}},Move,}' '' {display-menu -xL -yL -T '#[align=centre]Move'  'Centre' 'c' {move-pane -P centre} '' 'Top Left' '1' {move-pane -P top-left} 'Top Right' '2' {move-pane -P top-right} 'Bottom Left' '3' {move-pane -P bottom-left} 'Bottom Right' '4' {move-pane -P bottom-right} '' 'Top' 't' {move-pane -P top-centre} 'Bottom' 'b' {move-pane -P bottom-centre} 'Left' 'l' {move-pane -P centre-left} 'Right' 'r' {move-pane -P centre-right} } '#{?#{#{pane_floating_flag}},Move & Resize,}' '' {display-menu -xL -yL -T '#[align=centre]Move & Resize'  'Fill' '0' {resize-pane -x100% -y100%; move-pane -P top-left} '' 'Top Left' '1' {resize-pane -x50% -y50%; move-pane -P top-left} 'Top Right' '2' {resize-pane -x50% -y50%; move-pane -P top-right} 'Bottom Left' '3' {resize-pane -x50% -y50%; move-pane -P bottom-left} 'Bottom Right' '4' {resize-pane -x50% -y50%; move-pane -P bottom-right} '' 'Top' 't' {resize-pane -x100% -y50%; move-pane -P top-centre} 'Bottom' 'b' {resize-pane -x100% -y50%; move-pane -P bottom-centre} 'Left' 'l' {resize-pane -x50% -y100%; move-pane -P centre-left} 'Right' 'r' {resize-pane -x50% -y100%; move-pane -P centre-right} } '#{?#{#{pane_floating_flag}},Tile,}' 't' { join-pane } '#{?#{!:#{pane_floating_flag}},Float,}' 'f' { break-pane -W } '#{?#{!:#{pane_floating_flag}},Horizontal Split,}' 'h' {split-window -h} '#{?#{!:#{pane_floating_flag}},Vertical Split,}' 'v' {split-window -v} '' '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Up,}' 'u' {swap-pane -U} '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Down,}' 'd' {swap-pane -D} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-pane} '' 'Kill' 'X' {kill-pane} 'Respawn' 'R' {respawn-pane -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} '#{?#{>:#{window_panes},1},,-}#{?window_zoomed_flag,Unzoom,Zoom}' 'z' {resize-pane -Z} }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown1Pane { select-pane -t=; send -M }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n MouseDrag1Pane { if -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' { send -M } { copy-mode -M } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n C-MouseDrag1Pane { new-pane -M }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n C-MouseDrag1Empty { new-pane -M }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n M-MouseDrag1Pane { move-pane -M }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n WheelUpPane { if -F '#{||:#{alternate_on},#{pane_in_mode},#{mouse_any_flag}}' { send -M } { copy-mode -e } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown2Pane { select-pane -t=; if -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' { send -M } { paste -p } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n DoubleClick1Pane { select-pane -t=; if -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' { send -M } { copy-mode -H; send -X select-word; run -d0.3; send -X copy-pipe-and-cancel } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n TripleClick1Pane { select-pane -t=; if -F '#{||:#{pane_in_mode},#{mouse_any_flag}}' { send -M } { copy-mode -H; send -X select-line; run -d0.3; send -X copy-pipe-and-cancel } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown1Border { select-pane -M }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n MouseDrag1Border { resize-pane -M }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n M-MouseDrag1Border { move-pane -M }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n MouseDown1Status { switch-client -t= }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n C-MouseDown1Status { swap-window -t@ }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n MouseDown1Control9 { display-menu -t= -xM -yM -O -T 'Kill pane #{pane_index}?' 'Yes' 'y' { kill-pane -t= } 'No' 'n' {}}\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown1Control8 { resize-pane -Z }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n MouseDown1Control7 { if -Ft= '#{pane_floating_flag}' { join-pane } { break-pane -W } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n WheelDownStatus { next-window }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n WheelUpStatus { previous-window }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -n MouseDown3StatusLeft { run -C \"display-menu -t= -xM -yW -T '#[align=centre]#{session_name}'  #{S/t:#{?#{&&:#{<:#{loop_index},6},#{!:#{session_active}}},'Switch To #[underscore]#{session_name}' '' {switch-client -t=#{session_id}#} ,}} '' 'Renumber' 'N' {move-window -r} 'Rename' 'r' {command-prompt -I '#S' {rename-session -- '%%'}} 'Detach' 'd' {detach-client} '' 'New Session' 's' {new-session} 'New Window' 'w' {new-window}\" }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n M-MouseDown3StatusLeft { run -C \"display-menu -t= -xM -yW -T '#[align=centre]#{session_name}'  #{S/t:#{?#{&&:#{<:#{loop_index},6},#{!:#{session_active}}},'Switch To #[underscore]#{session_name}' '' {switch-client -t=#{session_id}#} ,}} '' 'Renumber' 'N' {move-window -r} 'Rename' 'r' {command-prompt -I '#S' {rename-session -- '%%'}} 'Detach' 'd' {detach-client} '' 'New Session' 's' {new-session} 'New Window' 'w' {new-window}\" }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown3Status { display-menu -t= -xW -yW -T '#[align=centre]#{window_index}:#{window_name}'  '#{?#{>:#{session_windows},1},,-}Swap Left' 'l' {swap-window -t:-1} '#{?#{>:#{session_windows},1},,-}Swap Right' 'r' {swap-window -t:+1} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-window} '' 'Kill' 'X' {kill-window} 'Respawn' 'R' {respawn-window -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} 'Rename' 'n' {command-prompt -FI \"#W\" {rename-window -t '#{window_id}' -- '%%'}} '' 'New After' 'w' {new-window -a} 'New At End' 'W' {new-window}}\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n M-MouseDown3Status { display-menu -t= -xW -yW -T '#[align=centre]#{window_index}:#{window_name}'  '#{?#{>:#{session_windows},1},,-}Swap Left' 'l' {swap-window -t:-1} '#{?#{>:#{session_windows},1},,-}Swap Right' 'r' {swap-window -t:+1} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-window} '' 'Kill' 'X' {kill-window} 'Respawn' 'R' {respawn-window -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} 'Rename' 'n' {command-prompt -FI \"#W\" {rename-window -t '#{window_id}' -- '%%'}} '' 'New After' 'w' {new-window -a} 'New At End' 'W' {new-window}}\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown3Pane { if -Ft= '#{||:#{mouse_any_flag},#{&&:#{pane_in_mode},#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}}}' { select-pane -t=; send -M } { display-menu -t= -xM -yM -T '#[align=centre]#{pane_index} (#{pane_id})'  '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Top,}' '<' {send -X history-top} '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Bottom,}' '>' {send -X history-bottom} '' '#{?#{==:#{pane_mode},copy-mode},#{?copy_line_numbers,Hide Line Numbers,Show Line Numbers},}' 'L' {send -X line-numbers-toggle} '#{?#{==:#{pane_mode},copy-mode},#{?refresh_active,Refresh Off,Refresh On},}' 'r' {send -X refresh-toggle} '' '#{?#{&&:#{buffer_size},#{!:#{pane_in_mode}}},Paste #[underscore]#{=/9/...:buffer_sample},}' 'p' {paste-buffer} '' '#{?mouse_word,Search For #[underscore]#{=/9/...:mouse_word},}' 'C-r' {if -F '#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}' 'copy-mode -t='; send -Xt= search-backward -- \"#{q:mouse_word}\"} '#{?mouse_word,Type #[underscore]#{=/9/...:mouse_word},}' 'C-y' {copy-mode -q; send-keys -l -- \"#{q:mouse_word}\"} '#{?mouse_word,Copy #[underscore]#{=/9/...:mouse_word},}' 'c' {copy-mode -q; set-buffer -- \"#{q:mouse_word}\"} '#{?mouse_line,Copy Line,}' 'l' {copy-mode -q; set-buffer -- \"#{q:mouse_line}\"} '' '#{?mouse_hyperlink,Type #[underscore]#{=/9/...:mouse_hyperlink},}' 'C-h' {copy-mode -q; send-keys -l -- \"#{q:mouse_hyperlink}\"} '#{?mouse_hyperlink,Copy #[underscore]#{=/9/...:mouse_hyperlink},}' 'h' {copy-mode -q; set-buffer -- \"#{q:mouse_hyperlink}\"} '' '#{?#{#{pane_floating_flag}},Move,}' '' {display-menu -xL -yL -T '#[align=centre]Move'  'Centre' 'c' {move-pane -P centre} '' 'Top Left' '1' {move-pane -P top-left} 'Top Right' '2' {move-pane -P top-right} 'Bottom Left' '3' {move-pane -P bottom-left} 'Bottom Right' '4' {move-pane -P bottom-right} '' 'Top' 't' {move-pane -P top-centre} 'Bottom' 'b' {move-pane -P bottom-centre} 'Left' 'l' {move-pane -P centre-left} 'Right' 'r' {move-pane -P centre-right} } '#{?#{#{pane_floating_flag}},Move & Resize,}' '' {display-menu -xL -yL -T '#[align=centre]Move & Resize'  'Fill' '0' {resize-pane -x100% -y100%; move-pane -P top-left} '' 'Top Left' '1' {resize-pane -x50% -y50%; move-pane -P top-left} 'Top Right' '2' {resize-pane -x50% -y50%; move-pane -P top-right} 'Bottom Left' '3' {resize-pane -x50% -y50%; move-pane -P bottom-left} 'Bottom Right' '4' {resize-pane -x50% -y50%; move-pane -P bottom-right} '' 'Top' 't' {resize-pane -x100% -y50%; move-pane -P top-centre} 'Bottom' 'b' {resize-pane -x100% -y50%; move-pane -P bottom-centre} 'Left' 'l' {resize-pane -x50% -y100%; move-pane -P centre-left} 'Right' 'r' {resize-pane -x50% -y100%; move-pane -P centre-right} } '#{?#{#{pane_floating_flag}},Tile,}' 't' { join-pane } '#{?#{!:#{pane_floating_flag}},Float,}' 'f' { break-pane -W } '#{?#{!:#{pane_floating_flag}},Horizontal Split,}' 'h' {split-window -h} '#{?#{!:#{pane_floating_flag}},Vertical Split,}' 'v' {split-window -v} '' '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Up,}' 'u' {swap-pane -U} '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Down,}' 'd' {swap-pane -D} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-pane} '' 'Kill' 'X' {kill-pane} 'Respawn' 'R' {respawn-pane -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} '#{?#{>:#{window_panes},1},,-}#{?window_zoomed_flag,Unzoom,Zoom}' 'z' {resize-pane -Z} } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n M-MouseDown3Pane { display-menu -t= -xM -yM -T '#[align=centre]#{pane_index} (#{pane_id})'  '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Top,}' '<' {send -X history-top} '#{?#{m/r:(copy|view)-mode,#{pane_mode}},Go To Bottom,}' '>' {send -X history-bottom} '' '#{?#{==:#{pane_mode},copy-mode},#{?copy_line_numbers,Hide Line Numbers,Show Line Numbers},}' 'L' {send -X line-numbers-toggle} '#{?#{==:#{pane_mode},copy-mode},#{?refresh_active,Refresh Off,Refresh On},}' 'r' {send -X refresh-toggle} '' '#{?#{&&:#{buffer_size},#{!:#{pane_in_mode}}},Paste #[underscore]#{=/9/...:buffer_sample},}' 'p' {paste-buffer} '' '#{?mouse_word,Search For #[underscore]#{=/9/...:mouse_word},}' 'C-r' {if -F '#{?#{m/r:(copy|view)-mode,#{pane_mode}},0,1}' 'copy-mode -t='; send -Xt= search-backward -- \"#{q:mouse_word}\"} '#{?mouse_word,Type #[underscore]#{=/9/...:mouse_word},}' 'C-y' {copy-mode -q; send-keys -l -- \"#{q:mouse_word}\"} '#{?mouse_word,Copy #[underscore]#{=/9/...:mouse_word},}' 'c' {copy-mode -q; set-buffer -- \"#{q:mouse_word}\"} '#{?mouse_line,Copy Line,}' 'l' {copy-mode -q; set-buffer -- \"#{q:mouse_line}\"} '' '#{?mouse_hyperlink,Type #[underscore]#{=/9/...:mouse_hyperlink},}' 'C-h' {copy-mode -q; send-keys -l -- \"#{q:mouse_hyperlink}\"} '#{?mouse_hyperlink,Copy #[underscore]#{=/9/...:mouse_hyperlink},}' 'h' {copy-mode -q; set-buffer -- \"#{q:mouse_hyperlink}\"} '' '#{?#{#{pane_floating_flag}},Move,}' '' {display-menu -xL -yL -T '#[align=centre]Move'  'Centre' 'c' {move-pane -P centre} '' 'Top Left' '1' {move-pane -P top-left} 'Top Right' '2' {move-pane -P top-right} 'Bottom Left' '3' {move-pane -P bottom-left} 'Bottom Right' '4' {move-pane -P bottom-right} '' 'Top' 't' {move-pane -P top-centre} 'Bottom' 'b' {move-pane -P bottom-centre} 'Left' 'l' {move-pane -P centre-left} 'Right' 'r' {move-pane -P centre-right} } '#{?#{#{pane_floating_flag}},Move & Resize,}' '' {display-menu -xL -yL -T '#[align=centre]Move & Resize'  'Fill' '0' {resize-pane -x100% -y100%; move-pane -P top-left} '' 'Top Left' '1' {resize-pane -x50% -y50%; move-pane -P top-left} 'Top Right' '2' {resize-pane -x50% -y50%; move-pane -P top-right} 'Bottom Left' '3' {resize-pane -x50% -y50%; move-pane -P bottom-left} 'Bottom Right' '4' {resize-pane -x50% -y50%; move-pane -P bottom-right} '' 'Top' 't' {resize-pane -x100% -y50%; move-pane -P top-centre} 'Bottom' 'b' {resize-pane -x100% -y50%; move-pane -P bottom-centre} 'Left' 'l' {resize-pane -x50% -y100%; move-pane -P centre-left} 'Right' 'r' {resize-pane -x50% -y100%; move-pane -P centre-right} } '#{?#{#{pane_floating_flag}},Tile,}' 't' { join-pane } '#{?#{!:#{pane_floating_flag}},Float,}' 'f' { break-pane -W } '#{?#{!:#{pane_floating_flag}},Horizontal Split,}' 'h' {split-window -h} '#{?#{!:#{pane_floating_flag}},Vertical Split,}' 'v' {split-window -v} '' '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Up,}' 'u' {swap-pane -U} '#{?#{&&:#{!:#{pane_floating_flag}},#{>:#{window_panes},1}},Swap Down,}' 'd' {swap-pane -D} '#{?pane_marked_set,,-}Swap Marked' 's' {swap-pane} '' 'Kill' 'X' {kill-pane} 'Respawn' 'R' {respawn-pane -k} '#{?pane_marked,Unmark,Mark}' 'm' {select-pane -m} '#{?#{>:#{window_panes},1},,-}#{?window_zoomed_flag,Unzoom,Zoom}' 'z' {resize-pane -Z} }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown3Empty { display-menu -t= -xM -yM -T '#[align=centre]#{window_index}:#{window_name}'  'New Pane' 'p' {new-pane; join-pane} 'New Window' 'w' {new-window} }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n M-MouseDown3Empty { display-menu -t= -xM -yM -T '#[align=centre]#{window_index}:#{window_name}'  'New Pane' 'p' {new-pane; join-pane} 'New Window' 'w' {new-window} }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown1ScrollbarUp { if -Ft= '#{pane_in_mode}' { send -X page-up } {copy-mode -u } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDown1ScrollbarDown { if -Ft= '#{pane_in_mode}' { send -X page-down } {copy-mode -d } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -n MouseDrag1ScrollbarSlider { if -Ft= '#{pane_in_mode}' { send -X scroll-to-mouse } { copy-mode -S } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-Space { send -X begin-selection }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-a { send -X start-of-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-c { send -X cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-e { send -X end-of-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-f { send -X cursor-right }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-b { send -X cursor-left }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-g { send -X clear-selection }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-k { send -X copy-pipe-end-of-line-and-cancel }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-l { send -X recentre-top-bottom }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-l { send -X cursor-centre-horizontal }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-n { send -X cursor-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-p { send -X cursor-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-r { command-prompt -P -T search -ip'(search up)' -I'#{pane_search_string}' { send -X search-backward-incremental -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-s { command-prompt -P -T search -ip'(search down)' -I'#{pane_search_string}' { send -X search-forward-incremental -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-v { send -X page-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-w { send -X copy-pipe-and-cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode Escape { send -X cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-[ { send -X cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode Space { send -X page-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode , { send -X jump-reverse }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode \\; { send -X jump-again }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode F { command-prompt -P -1p'(jump backward)' { send -X jump-backward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode L { send -X line-numbers-toggle }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode N { send -X search-reverse }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode P { send -X toggle-position }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode R { send -X rectangle-toggle }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode T { command-prompt -P -1p'(jump to backward)' { send -X jump-to-backward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode X { send -X set-mark }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode f { command-prompt -P -1p'(jump forward)' { send -X jump-forward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode g { command-prompt -P -p'(goto line)' { send -X goto-line -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode n { send -X search-again }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode q { send -X cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode r { send -X refresh-now }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode t { command-prompt -P -1p'(jump to forward)' { send -X jump-to-forward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode Home { send -X start-of-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode End { send -X end-of-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode MouseDown1Pane select-pane\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode MouseDrag1Pane { select-pane; send -X begin-selection }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode MouseDragEnd1Pane { send -X copy-pipe-and-cancel }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode WheelUpPane { select-pane; send -N5 -X scroll-up }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode WheelDownPane { select-pane; send -N5 -X scroll-down }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode DoubleClick1Pane { select-pane; send -X select-word; run -d0.3; send -X copy-pipe-and-cancel }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode TripleClick1Pane { select-pane; send -X select-line; run -d0.3; send -X copy-pipe-and-cancel }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode NPage { send -X page-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode PPage { send -X page-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode Up { send -X cursor-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode Down { send -X cursor-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode Left { send -X cursor-left }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode Right { send -X cursor-right }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-1 { command-prompt -P -Np'(repeat)' -I1 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-2 { command-prompt -P -Np'(repeat)' -I2 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-3 { command-prompt -P -Np'(repeat)' -I3 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-4 { command-prompt -P -Np'(repeat)' -I4 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-5 { command-prompt -P -Np'(repeat)' -I5 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-6 { command-prompt -P -Np'(repeat)' -I6 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-7 { command-prompt -P -Np'(repeat)' -I7 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-8 { command-prompt -P -Np'(repeat)' -I8 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-9 { command-prompt -P -Np'(repeat)' -I9 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-< { send -X history-top }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-> { send -X history-bottom }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-R { send -X top-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-b { send -X previous-word }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-M-b { send -X previous-matching-bracket }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-f { send -X next-word-end }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-M-f { send -X next-matching-bracket }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-m { send -X back-to-indentation }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-r { send -X middle-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-v { send -X page-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-w { send -X copy-pipe-and-cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-x { send -X jump-to-mark }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode 'M-{' { send -X previous-paragraph }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode 'M-}' { send -X next-paragraph }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-Up { send -X halfpage-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-Down { send -X halfpage-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-Up { send -X scroll-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode C-Down { send -X scroll-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-C-Up { send -X previous-prompt }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode M-C-Down { send -X next-prompt }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi '#' { send -FX search-backward -- '#{copy_cursor_word}' }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi * { send -FX search-forward -- '#{copy_cursor_word}' }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-c { send -X cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-d { send -X halfpage-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-e { send -X scroll-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-b { send -X page-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-f { send -X page-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-h { send -X cursor-left }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-j { send -X copy-pipe-and-cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi Enter { send -X copy-pipe-and-cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-u { send -X halfpage-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-v { send -X rectangle-toggle }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-y { send -X scroll-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi Escape { send -X clear-selection }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-[ { send -X clear-selection }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi Space { send -X begin-selection }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi '$' { send -X end-of-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi , { send -X jump-reverse }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi / { command-prompt -P -T search -p'(search down)' { send -X search-forward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 0 { send -X start-of-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 1 { command-prompt -P -Np'(repeat)' -I1 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 2 { command-prompt -P -Np'(repeat)' -I2 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 3 { command-prompt -P -Np'(repeat)' -I3 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 4 { command-prompt -P -Np'(repeat)' -I4 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 5 { command-prompt -P -Np'(repeat)' -I5 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 6 { command-prompt -P -Np'(repeat)' -I6 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 7 { command-prompt -P -Np'(repeat)' -I7 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 8 { command-prompt -P -Np'(repeat)' -I8 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi 9 { command-prompt -P -Np'(repeat)' -I9 { send -N '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi : { command-prompt -P -p'(goto line)' { send -X goto-line -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi \\; { send -X jump-again }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi ? { command-prompt -P -T search -p'(search up)' { send -X search-backward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi A { send -X append-selection-and-cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi B { send -X previous-space }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi D { send -X copy-pipe-end-of-line-and-cancel }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi E { send -X next-space-end }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi F { command-prompt -P -1p'(jump backward)' { send -X jump-backward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi G { send -X history-bottom }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi H { send -X top-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi J { send -X scroll-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi K { send -X scroll-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi L { send -X bottom-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi M { send -X middle-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi N { send -X search-reverse }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi P { send -X toggle-position }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi T { command-prompt -P -1p'(jump to backward)' { send -X jump-to-backward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi V { send -X select-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi W { send -X next-space }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi X { send -X set-mark }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi ^ { send -X back-to-indentation }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi b { send -X previous-word }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi e { send -X next-word-end }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi f { command-prompt -P -1p'(jump forward)' { send -X jump-forward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi g { send -X history-top }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi h { send -X cursor-left }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi j { send -X cursor-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi k { send -X cursor-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi z { send -X scroll-middle }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi l { send -X cursor-right }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi n { send -X search-again }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi o { send -X other-end }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi q { send -X cancel }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi r { send -X refresh-now }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi t { command-prompt -P -1p'(jump to forward)' { send -X jump-to-forward -- '%%' } }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi v { send -X rectangle-toggle }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi w { send -X next-word }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi '{' { send -X previous-paragraph }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi '}' { send -X next-paragraph }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi % { send -X next-matching-bracket }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi Home { send -X start-of-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi End { send -X end-of-line }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi MouseDown1Pane { select-pane }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi MouseDrag1Pane { select-pane; send -X begin-selection }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi MouseDragEnd1Pane { send -X copy-pipe-and-cancel }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi WheelUpPane { select-pane; send -N5 -X scroll-up }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi WheelDownPane { select-pane; send -N5 -X scroll-down }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi DoubleClick1Pane { select-pane; send -X select-word; run -d0.3; send -X copy-pipe-and-cancel }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi TripleClick1Pane { select-pane; send -X select-line; run -d0.3; send -X copy-pipe-and-cancel }\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi BSpace { send -X cursor-left }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi NPage { send -X page-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi PPage { send -X page-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi Up { send -X cursor-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi Down { send -X cursor-down }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi Left { send -X cursor-left }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi Right { send -X cursor-right }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi M-x { send -X jump-to-mark }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-Up { send -X scroll-up }\0" as *const u8
            as *const ::core::ffi::c_char,
        b"bind -Tcopy-mode-vi C-Down { send -X scroll-down }\0" as *const u8
            as *const ::core::ffi::c_char,
    ];
    let mut i: u_int = 0;
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[*const ::core::ffi::c_char; 308]>() as usize)
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>() as usize)
    {
        pr = cmd_parse_from_string(
            defaults[i as usize],
            ::core::ptr::null_mut::<cmd_parse_input>(),
        );
        if (*pr).status as ::core::ffi::c_uint
            != CMD_PARSE_SUCCESS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            log_debug(
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*pr).error,
            );
            fatalx(
                b"bad default key: %s\0" as *const u8 as *const ::core::ffi::c_char,
                defaults[i as usize],
            );
        }
        cmdq_append(
            ::core::ptr::null_mut::<client>(),
            cmdq_get_command((*pr).cmdlist, ::core::ptr::null_mut::<cmdq_state>()),
        );
        cmd_list_free((*pr).cmdlist);
        i = i.wrapping_add(1);
    }
    cmdq_append(
        ::core::ptr::null_mut::<client>(),
        cmdq_get_callback1(
            b"key_bindings_init_done\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                key_bindings_init_done
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ),
    );
}
unsafe extern "C" fn key_bindings_read_only(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    cmdq_error(
        item,
        b"client is read-only\0" as *const u8 as *const ::core::ffi::c_char,
    );
    return CMD_RETURN_ERROR;
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_dispatch(
    mut bd: *mut key_binding,
    mut item: *mut cmdq_item,
    mut c: *mut client,
    mut event: *mut key_event,
    mut fs: *mut cmd_find_state,
) -> *mut cmdq_item {
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut new_state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    let mut readonly: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if c.is_null() || !(*c).flags & CLIENT_READONLY as uint64_t != 0 {
        readonly = 1 as ::core::ffi::c_int;
    } else {
        readonly = cmd_list_all_have((*bd).cmdlist, CMD_READONLY);
    }
    if readonly == 0 {
        new_item = cmdq_get_callback1(
            b"key_bindings_read_only\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                key_bindings_read_only
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        );
    } else {
        if (*bd).flags & KEY_BINDING_REPEAT != 0 {
            flags |= CMDQ_STATE_REPEAT;
        }
        new_state = cmdq_new_state(fs, event, flags);
        new_item = cmdq_get_command((*bd).cmdlist, new_state);
        cmdq_free_state(new_state);
    }
    if !item.is_null() {
        new_item = cmdq_insert_after(item, new_item);
    } else {
        new_item = cmdq_append(c, new_item);
    }
    return new_item;
}
#[no_mangle]
pub unsafe extern "C" fn key_bindings_has_repeat(
    mut l: *mut *mut key_binding,
    mut n: u_int,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < n {
        if (**l.offset(i as isize)).flags & KEY_BINDING_REPEAT != 0 {
            return 1 as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}

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
    pub type screen_write_citem;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strlcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    fn time(__timer: *mut time_t) -> time_t;
    fn strftime(
        __s: *mut ::core::ffi::c_char,
        __maxsize: size_t,
        __format: *const ::core::ffi::c_char,
        __tp: *const tm,
    ) -> size_t;
    fn localtime(__timer: *const time_t) -> *mut tm;
    fn gmtime_r(__timer: *const time_t, __tp: *mut tm) -> *mut tm;
    fn clock_gettime(__clock_id: clockid_t, __tp: *mut timespec) -> ::core::ffi::c_int;
    fn event_add(ev: *mut event, timeout: *const timeval) -> ::core::ffi::c_int;
    fn event_del(_: *mut event) -> ::core::ffi::c_int;
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
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn format_free(_: *mut format_tree);
    fn format_create_defaults(
        _: *mut cmdq_item,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut format_tree;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    static grid_default_cell: grid_cell;
    fn screen_write_start(_: *mut screen_write_ctx, _: *mut screen);
    fn screen_write_stop(_: *mut screen_write_ctx);
    fn screen_write_puts(
        _: *mut screen_write_ctx,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn screen_write_putc(_: *mut screen_write_ctx, _: *const grid_cell, _: u_char);
    fn screen_write_cursormove(
        _: *mut screen_write_ctx,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn screen_write_clearscreen(_: *mut screen_write_ctx, _: u_int);
    fn screen_init(_: *mut screen, _: u_int, _: u_int, _: u_int);
    fn screen_free(_: *mut screen);
    fn screen_resize(_: *mut screen, _: u_int, _: u_int, _: ::core::ffi::c_int);
    fn window_pane_reset_mode(_: *mut window_pane);
    fn style_apply(
        _: *mut grid_cell,
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: *mut format_tree,
    );
}
pub type __u_char = ::core::ffi::c_uchar;
pub type __u_short = ::core::ffi::c_ushort;
pub type __u_int = ::core::ffi::c_uint;
pub type __uint8_t = u8;
pub type __uint64_t = u64;
pub type __pid_t = ::core::ffi::c_int;
pub type __time_t = ::core::ffi::c_long;
pub type __suseconds_t = ::core::ffi::c_long;
pub type __clockid_t = ::core::ffi::c_int;
pub type __syscall_slong_t = ::core::ffi::c_long;
pub type u_char = __u_char;
pub type u_short = __u_short;
pub type u_int = __u_int;
pub type pid_t = __pid_t;
pub type clockid_t = __clockid_t;
pub type time_t = __time_t;
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct timeval {
    pub tv_sec: __time_t,
    pub tv_usec: __suseconds_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct timespec {
    pub tv_sec: __time_t,
    pub tv_nsec: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tm {
    pub tm_sec: ::core::ffi::c_int,
    pub tm_min: ::core::ffi::c_int,
    pub tm_hour: ::core::ffi::c_int,
    pub tm_mday: ::core::ffi::c_int,
    pub tm_mon: ::core::ffi::c_int,
    pub tm_year: ::core::ffi::c_int,
    pub tm_wday: ::core::ffi::c_int,
    pub tm_yday: ::core::ffi::c_int,
    pub tm_isdst: ::core::ffi::c_int,
    pub tm_gmtoff: ::core::ffi::c_long,
    pub tm_zone: *const ::core::ffi::c_char,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct screen_write_ctx {
    pub wp: *mut window_pane,
    pub s: *mut screen,
    pub flags: ::core::ffi::c_int,
    pub init_ctx_cb: screen_write_init_ctx_cb,
    pub arg: *mut ::core::ffi::c_void,
    pub item: *mut screen_write_citem,
    pub scrolled: u_int,
    pub bg: u_int,
}
pub type screen_write_init_ctx_cb =
    Option<unsafe extern "C" fn(*mut screen_write_ctx, *mut tty_ctx) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_ctx {
    pub s: *mut screen,
    pub redraw_cb: tty_ctx_redraw_cb,
    pub set_client_cb: tty_ctx_set_client_cb,
    pub arg: *mut ::core::ffi::c_void,
    pub cell: *const grid_cell,
    pub flags: ::core::ffi::c_int,
    pub c2rust_unnamed: C2RustUnnamed_35,
    pub ocx: u_int,
    pub ocy: u_int,
    pub orupper: u_int,
    pub orlower: u_int,
    pub xoff: ::core::ffi::c_int,
    pub yoff: ::core::ffi::c_int,
    pub rxoff: ::core::ffi::c_int,
    pub ryoff: ::core::ffi::c_int,
    pub sx: u_int,
    pub sy: u_int,
    pub bg: u_int,
    pub defaults: grid_cell,
    pub style_ctx: tty_style_ctx,
    pub wox: u_int,
    pub woy: u_int,
    pub wsx: u_int,
    pub wsy: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_style_ctx {
    pub defaults: *const grid_cell,
    pub palette: *mut colour_palette,
    pub dim: u_int,
    pub hyperlinks: *mut hyperlinks,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_35 {
    pub n: u_int,
    pub data: C2RustUnnamed_37,
    pub sel: C2RustUnnamed_36,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_36 {
    pub clip: *const ::core::ffi::c_char,
    pub data: *const ::core::ffi::c_char,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
    pub data: *const ::core::ffi::c_char,
    pub size: size_t,
}
pub type tty_ctx_set_client_cb =
    Option<unsafe extern "C" fn(*mut tty_ctx, *mut client) -> ::core::ffi::c_int>;
pub type tty_ctx_redraw_cb = Option<unsafe extern "C" fn(*const tty_ctx) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_clock_mode_data {
    pub screen: screen,
    pub tim: time_t,
    pub timer: event,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const CLOCK_REALTIME: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MODE_CURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const GRID_FLAG_NOPALETTE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const PANE_REDRAW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
#[no_mangle]
pub static mut window_clock_mode: window_mode = unsafe {
    window_mode {
        name: b"clock-mode\0" as *const u8 as *const ::core::ffi::c_char,
        default_format: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: 0,
        init: Some(
            window_clock_init
                as unsafe extern "C" fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_clock_free as unsafe extern "C" fn(*mut window_mode_entry) -> ()),
        resize: Some(
            window_clock_resize as unsafe extern "C" fn(*mut window_mode_entry, u_int, u_int) -> (),
        ),
        update: None,
        style_changed: None,
        key: Some(
            window_clock_key
                as unsafe extern "C" fn(
                    *mut window_mode_entry,
                    *mut client,
                    *mut session,
                    *mut winlink,
                    key_code,
                    *mut mouse_event,
                ) -> (),
        ),
        key_table: None,
        command: None,
        formats: None,
        get_screen: None,
    }
};
#[no_mangle]
pub static mut window_clock_table: [[[::core::ffi::c_char; 5]; 5]; 14] = [
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
    [
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
        [
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            0 as ::core::ffi::c_int as ::core::ffi::c_char,
            1 as ::core::ffi::c_int as ::core::ffi::c_char,
        ],
    ],
];
unsafe extern "C" fn window_clock_start_timer(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_clock_mode_data = (*wme).data as *mut window_clock_mode_data;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut ts: timespec = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let mut delay: ::core::ffi::c_long = 0;
    clock_gettime(CLOCK_REALTIME, &raw mut ts);
    delay = (1000000 as __syscall_slong_t - ts.tv_nsec / 1000 as __syscall_slong_t)
        as ::core::ffi::c_long;
    tv.tv_sec = (delay / 1000000 as ::core::ffi::c_long) as __time_t;
    tv.tv_usec = (delay % 1000000 as ::core::ffi::c_long) as __suseconds_t;
    if tv.tv_sec < 0 as __time_t || tv.tv_sec == 0 as __time_t && tv.tv_usec <= 0 as __suseconds_t {
        tv.tv_sec = 1 as __time_t;
        tv.tv_usec = 0 as __suseconds_t;
    }
    event_add(&raw mut (*data).timer, &raw mut tv);
}
unsafe extern "C" fn window_clock_timer_callback(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut wme: *mut window_mode_entry = arg as *mut window_mode_entry;
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_clock_mode_data = (*wme).data as *mut window_clock_mode_data;
    let mut now: tm = tm {
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
    let mut then: tm = tm {
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
    let mut t: time_t = 0;
    event_del(&raw mut (*data).timer);
    t = time(::core::ptr::null_mut::<time_t>());
    gmtime_r(&raw mut t, &raw mut now);
    gmtime_r(&raw mut (*data).tim, &raw mut then);
    if now.tm_sec != then.tm_sec {
        (*data).tim = t;
        window_clock_draw_screen(wme);
        (*wp).flags |= PANE_REDRAW;
    }
    window_clock_start_timer(wme);
}
unsafe extern "C" fn window_clock_init(
    mut wme: *mut window_mode_entry,
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_clock_mode_data = ::core::ptr::null_mut::<window_clock_mode_data>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    data = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_clock_mode_data>() as size_t,
    ) as *mut window_clock_mode_data;
    (*wme).data = data as *mut ::core::ffi::c_void;
    (*data).tim = time(::core::ptr::null_mut::<time_t>());
    event_set(
        &raw mut (*data).timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            window_clock_timer_callback
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        wme as *mut ::core::ffi::c_void,
    );
    window_clock_start_timer(wme);
    s = &raw mut (*data).screen;
    screen_init(s, (*(*wp).base.grid).sx, (*(*wp).base.grid).sy, 0 as u_int);
    (*s).mode &= !MODE_CURSOR;
    window_clock_draw_screen(wme);
    return s;
}
unsafe extern "C" fn window_clock_free(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_clock_mode_data = (*wme).data as *mut window_clock_mode_data;
    event_del(&raw mut (*data).timer);
    screen_free(&raw mut (*data).screen);
    free(data as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_clock_resize(
    mut wme: *mut window_mode_entry,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_clock_mode_data = (*wme).data as *mut window_clock_mode_data;
    let mut s: *mut screen = &raw mut (*data).screen;
    screen_resize(s, sx, sy, 0 as ::core::ffi::c_int);
    window_clock_draw_screen(wme);
}
unsafe extern "C" fn window_clock_key(
    mut wme: *mut window_mode_entry,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    window_pane_reset_mode((*wme).wp);
}
unsafe extern "C" fn window_clock_draw_screen(mut wme: *mut window_mode_entry) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut w: *mut window = (*wp).window as *mut window;
    let mut data: *mut window_clock_mode_data = (*wme).data as *mut window_clock_mode_data;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: ::core::ptr::null_mut::<screen_write_citem>(),
        scrolled: 0,
        bg: 0,
    };
    let mut colour: ::core::ffi::c_int = 0;
    let mut style: ::core::ffi::c_int = 0;
    let mut s: *mut screen = &raw mut (*data).screen;
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
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut tim: [::core::ffi::c_char; 64] = [0; 64];
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut t: time_t = 0;
    let mut tm: *mut tm = ::core::ptr::null_mut::<tm>();
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut idx: u_int = 0;
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        wp,
    );
    style_apply(
        &raw mut gc,
        (*w).options,
        b"clock-mode-colour\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    format_free(ft);
    colour = gc.fg;
    style = options_get_number(
        (*w).options,
        b"clock-mode-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    screen_write_start(&raw mut ctx, s);
    t = time(::core::ptr::null_mut::<time_t>());
    tm = localtime(&raw mut t);
    if style == 0 as ::core::ffi::c_int || style == 2 as ::core::ffi::c_int {
        if style == 2 as ::core::ffi::c_int {
            strftime(
                &raw mut tim as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                b"%l:%M:%S \0" as *const u8 as *const ::core::ffi::c_char,
                localtime(&raw mut t),
            );
        } else {
            strftime(
                &raw mut tim as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                b"%l:%M \0" as *const u8 as *const ::core::ffi::c_char,
                localtime(&raw mut t),
            );
        }
        if (*tm).tm_hour >= 12 as ::core::ffi::c_int {
            strlcat(
                &raw mut tim as *mut ::core::ffi::c_char,
                b"PM\0" as *const u8 as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
            );
        } else {
            strlcat(
                &raw mut tim as *mut ::core::ffi::c_char,
                b"AM\0" as *const u8 as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
            );
        }
    } else if style == 3 as ::core::ffi::c_int {
        strftime(
            &raw mut tim as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
            b"%H:%M:%S\0" as *const u8 as *const ::core::ffi::c_char,
            tm,
        );
    } else {
        strftime(
            &raw mut tim as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
            b"%H:%M\0" as *const u8 as *const ::core::ffi::c_char,
            tm,
        );
    }
    screen_write_clearscreen(&raw mut ctx, 8 as u_int);
    if ((*(*s).grid).sx as size_t)
        < (6 as size_t).wrapping_mul(strlen(&raw mut tim as *mut ::core::ffi::c_char))
        || (*(*s).grid).sy < 6 as u_int
    {
        if (*(*s).grid).sx as size_t >= strlen(&raw mut tim as *mut ::core::ffi::c_char)
            && (*(*s).grid).sy != 0 as u_int
        {
            x = ((*(*s).grid).sx.wrapping_div(2 as u_int) as size_t).wrapping_sub(
                strlen(&raw mut tim as *mut ::core::ffi::c_char).wrapping_div(2 as size_t),
            ) as u_int;
            y = (*(*s).grid).sy.wrapping_div(2 as u_int);
            screen_write_cursormove(
                &raw mut ctx,
                x as ::core::ffi::c_int,
                y as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            memcpy(
                &raw mut gc as *mut ::core::ffi::c_void,
                &raw const grid_default_cell as *const ::core::ffi::c_void,
                ::core::mem::size_of::<grid_cell>() as size_t,
            );
            gc.flags = (gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
            gc.fg = colour;
            screen_write_puts(
                &raw mut ctx,
                &raw mut gc,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut tim as *mut ::core::ffi::c_char,
            );
        }
        screen_write_stop(&raw mut ctx);
        return;
    }
    x = ((*(*s).grid).sx.wrapping_div(2 as u_int) as size_t)
        .wrapping_sub((3 as size_t).wrapping_mul(strlen(&raw mut tim as *mut ::core::ffi::c_char)))
        as u_int;
    y = (*(*s).grid)
        .sy
        .wrapping_div(2 as u_int)
        .wrapping_sub(3 as u_int);
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    gc.flags = (gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
    gc.bg = colour;
    gc.fg = colour;
    let mut current_block_56: u64;
    ptr = &raw mut tim as *mut ::core::ffi::c_char;
    while *ptr as ::core::ffi::c_int != '\0' as i32 {
        if *ptr as ::core::ffi::c_int >= '0' as i32 && *ptr as ::core::ffi::c_int <= '9' as i32 {
            idx = (*ptr as ::core::ffi::c_int - '0' as i32) as u_int;
            current_block_56 = 2543120759711851213;
        } else if *ptr as ::core::ffi::c_int == ':' as i32 {
            idx = 10 as u_int;
            current_block_56 = 2543120759711851213;
        } else if *ptr as ::core::ffi::c_int == 'A' as i32 {
            idx = 11 as u_int;
            current_block_56 = 2543120759711851213;
        } else if *ptr as ::core::ffi::c_int == 'P' as i32 {
            idx = 12 as u_int;
            current_block_56 = 2543120759711851213;
        } else if *ptr as ::core::ffi::c_int == 'M' as i32 {
            idx = 13 as u_int;
            current_block_56 = 2543120759711851213;
        } else {
            x = x.wrapping_add(6 as u_int);
            current_block_56 = 11913429853522160501;
        }
        match current_block_56 {
            2543120759711851213 => {
                j = 0 as u_int;
                while j < 5 as u_int {
                    i = 0 as u_int;
                    while i < 5 as u_int {
                        screen_write_cursormove(
                            &raw mut ctx,
                            x.wrapping_add(i) as ::core::ffi::c_int,
                            y.wrapping_add(j) as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        );
                        if window_clock_table[idx as usize][j as usize][i as usize] != 0 {
                            screen_write_putc(&raw mut ctx, &raw mut gc, '#' as i32 as u_char);
                        }
                        i = i.wrapping_add(1);
                    }
                    j = j.wrapping_add(1);
                }
                x = x.wrapping_add(6 as u_int);
            }
            _ => {}
        }
        ptr = ptr.offset(1);
    }
    screen_write_stop(&raw mut ctx);
}

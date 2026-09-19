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
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn strtonum(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn events_fire_window(_: *const ::core::ffi::c_char, _: *mut window);
    fn options_set_parent(_: *mut options, _: *mut options);
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_percentage_and_expand(
        _: *mut args,
        _: u_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut cmdq_item,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn cmd_find_from_session(_: *mut cmd_find_state, _: *mut session, _: ::core::ffi::c_int);
    fn cmd_get_entry(_: *mut cmd) -> *const cmd_entry;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmd_mouse_pane(
        _: *mut mouse_event,
        _: *mut *mut session,
        _: *mut *mut winlink,
    ) -> *mut window_pane;
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_source(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_event(_: *mut cmdq_item) -> *mut key_event;
    fn cmdq_get_current(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn server_client_remove_pane(_: *mut window_pane);
    fn server_redraw_session(_: *mut session);
    fn server_status_session(_: *mut session);
    fn server_redraw_window(_: *mut window);
    fn server_redraw_window_borders(_: *mut window);
    fn server_kill_window(_: *mut window, _: ::core::ffi::c_int);
    fn server_unzoom_window(_: *mut window);
    fn recalculate_sizes();
    fn colour_palette_from_option(_: *mut colour_palette, _: *mut options);
    fn redraw_invalidate_scene(_: *mut window);
    fn window_set_active_pane(
        _: *mut window,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn window_fire_pane_moved(
        _: *mut window_pane,
        _: *mut window,
        _: ::core::ffi::c_int,
        _: *mut window,
        _: ::core::ffi::c_int,
    );
    fn window_redraw_active_switch(_: *mut window, _: *mut window_pane);
    fn window_lost_pane(_: *mut window, _: *mut window_pane);
    fn window_count_panes(_: *mut window, _: ::core::ffi::c_int) -> u_int;
    fn window_pane_get_pane_lines(_: *mut window_pane) -> pane_lines;
    fn window_pane_is_floating(_: *mut window_pane) -> ::core::ffi::c_int;
    fn layout_fix_offsets(_: *mut window);
    fn layout_fix_panes(_: *mut window, _: *mut window_pane);
    fn layout_assign_pane(_: *mut layout_cell, _: *mut window_pane, _: ::core::ffi::c_int);
    fn layout_close_pane(_: *mut window_pane);
    fn layout_get_tiled_cell(
        _: *mut cmdq_item,
        _: *mut args,
        _: *mut window,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut layout_cell;
    fn layout_insert_tile(_: *mut window, _: *mut layout_cell) -> ::core::ffi::c_int;
    fn session_select(_: *mut session, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
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
pub type pane_lines = ::core::ffi::c_uint;
pub const PANE_LINES_ROUNDED: pane_lines = 7;
pub const PANE_LINES_NONE: pane_lines = 6;
pub const PANE_LINES_SPACES: pane_lines = 5;
pub const PANE_LINES_NUMBER: pane_lines = 4;
pub const PANE_LINES_SIMPLE: pane_lines = 3;
pub const PANE_LINES_HEAVY: pane_lines = 2;
pub const PANE_LINES_DOUBLE: pane_lines = 1;
pub const PANE_LINES_SINGLE: pane_lines = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_panes_zindex {
    pub tqh_first: *mut window_pane,
    pub tqh_last: *mut *mut window_pane,
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
pub type cmd_find_type = ::core::ffi::c_uint;
pub const CMD_FIND_SESSION: cmd_find_type = 2;
pub const CMD_FIND_WINDOW: cmd_find_type = 1;
pub const CMD_FIND_PANE: cmd_find_type = 0;
pub type cmd_retval = ::core::ffi::c_int;
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
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
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const INT_MIN: ::core::ffi::c_int = -__INT_MAX__ - 1 as ::core::ffi::c_int;
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const PANE_STYLECHANGED: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const PANE_THEMECHANGED: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const WINDOW_ZOOMED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const LAYOUT_CELL_FLOATING: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CMD_FIND_DEFAULT_MARKED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const SPAWN_BEFORE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const SPAWN_FULLSIZE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const SPAWN_HORIZONTAL: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
#[no_mangle]
pub static mut cmd_join_pane_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"join-pane\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"joinp\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"bdfhvp:l:s:t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-bdfhv] [-l size] [-s src-pane] [-t dst-pane]\0" as *const u8
            as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_DEFAULT_MARKED,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_join_pane_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_move_pane_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"move-pane\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"movep\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"bdD::fhMvl:L::P:R::s:t:U::X:Y:z:\0" as *const u8
                as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: b"[-bdfhMv] [-D lines] [-l size] [-L columns] [-P position] [-R columns] [-s src-pane] [-t dst-pane] [-U lines] [-X x-position] [-Y y-position] [-z z-index]\0"
            as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 's' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_DEFAULT_MARKED,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_join_pane_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_join_pane_place(
    mut item: *mut cmdq_item,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut position: *const ::core::ffi::c_char,
) -> cmd_retval {
    let mut w: *mut window = (*wl).window;
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    let mut owp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wx: ::core::ffi::c_int = (*w).sx as ::core::ffi::c_int;
    let mut wy: ::core::ffi::c_int = (*w).sy as ::core::ffi::c_int;
    let mut px: ::core::ffi::c_int = (*lc).g.sx as ::core::ffi::c_int;
    let mut py: ::core::ffi::c_int = (*lc).g.sy as ::core::ffi::c_int;
    let mut xoff: ::core::ffi::c_int = (*lc).g.xoff;
    let mut yoff: ::core::ffi::c_int = (*lc).g.yoff;
    let mut border: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if window_pane_get_pane_lines(wp) as ::core::ffi::c_uint
        == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        border = 0 as ::core::ffi::c_int;
    }
    if strcmp(
        position,
        b"top-left\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        xoff = border;
        yoff = border;
    } else if strcmp(
        position,
        b"top-centre\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"top-center\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff = (wx - px) / 2 as ::core::ffi::c_int;
        yoff = border;
    } else if strcmp(
        position,
        b"top-right\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        xoff = wx - px - border;
        yoff = border;
    } else if strcmp(
        position,
        b"centre-left\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"center-left\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff = border;
        yoff = (wy - py) / 2 as ::core::ffi::c_int;
    } else if strcmp(
        position,
        b"centre\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"center\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff = (wx - px) / 2 as ::core::ffi::c_int;
        yoff = (wy - py) / 2 as ::core::ffi::c_int;
    } else if strcmp(
        position,
        b"centre-right\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"center-right\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff = wx - px - border;
        yoff = (wy - py) / 2 as ::core::ffi::c_int;
    } else if strcmp(
        position,
        b"bottom-left\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        xoff = border;
        yoff = wy - py - border;
    } else if strcmp(
        position,
        b"bottom-centre\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"bottom-center\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff = (wx - px) / 2 as ::core::ffi::c_int;
        yoff = wy - py - border;
    } else if strcmp(
        position,
        b"bottom-right\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        xoff = wx - px - border;
        yoff = wy - py - border;
    } else if strcmp(
        position,
        b"top-left-centre\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"top-left-center\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff = wx / 4 as ::core::ffi::c_int - px / 2 as ::core::ffi::c_int;
        yoff = wy / 4 as ::core::ffi::c_int - py / 2 as ::core::ffi::c_int;
    } else if strcmp(
        position,
        b"top-right-centre\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"top-right-center\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff =
            3 as ::core::ffi::c_int * wx / 4 as ::core::ffi::c_int - px / 2 as ::core::ffi::c_int;
        yoff = wy / 4 as ::core::ffi::c_int - py / 2 as ::core::ffi::c_int;
    } else if strcmp(
        position,
        b"bottom-left-centre\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"bottom-left-center\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff = wx / 4 as ::core::ffi::c_int - px / 2 as ::core::ffi::c_int;
        yoff =
            3 as ::core::ffi::c_int * wy / 4 as ::core::ffi::c_int - py / 2 as ::core::ffi::c_int;
    } else if strcmp(
        position,
        b"bottom-right-centre\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            position,
            b"bottom-right-center\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        xoff =
            3 as ::core::ffi::c_int * wx / 4 as ::core::ffi::c_int - px / 2 as ::core::ffi::c_int;
        yoff =
            3 as ::core::ffi::c_int * wy / 4 as ::core::ffi::c_int - py / 2 as ::core::ffi::c_int;
    } else if strcmp(
        position,
        b"front\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        if !(*wp).zentry.tqe_next.is_null() {
            (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
        } else {
            (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
        }
        *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
        (*wp).zentry.tqe_next = (*w).z_index.tqh_first;
        if !(*wp).zentry.tqe_next.is_null() {
            (*(*w).z_index.tqh_first).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
        } else {
            (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
        }
        (*w).z_index.tqh_first = wp;
        (*wp).zentry.tqe_prev = &raw mut (*w).z_index.tqh_first;
    } else if strcmp(
        position,
        b"back\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        if !(*wp).zentry.tqe_next.is_null() {
            (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
        } else {
            (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
        }
        *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
        owp = (*w).z_index.tqh_first;
        while !owp.is_null() {
            if window_pane_is_floating(owp) == 0 {
                break;
            }
            owp = (*owp).zentry.tqe_next;
        }
        if !owp.is_null() {
            (*wp).zentry.tqe_prev = (*owp).zentry.tqe_prev;
            (*wp).zentry.tqe_next = owp;
            *(*owp).zentry.tqe_prev = wp;
            (*owp).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
        } else {
            (*wp).zentry.tqe_next = ::core::ptr::null_mut::<window_pane>();
            (*wp).zentry.tqe_prev = (*w).z_index.tqh_last;
            *(*w).z_index.tqh_last = wp;
            (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
        }
    } else if strcmp(
        position,
        b"forward\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        owp = *(*((*wp).zentry.tqe_prev as *mut window_panes_zindex)).tqh_last;
        if !owp.is_null() {
            if !(*wp).zentry.tqe_next.is_null() {
                (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
            } else {
                (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
            }
            *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
            (*wp).zentry.tqe_prev = (*owp).zentry.tqe_prev;
            (*wp).zentry.tqe_next = owp;
            *(*owp).zentry.tqe_prev = wp;
            (*owp).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
        }
    } else if strcmp(
        position,
        b"backward\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        owp = (*wp).zentry.tqe_next;
        if !owp.is_null() && window_pane_is_floating(owp) != 0 {
            if !(*wp).zentry.tqe_next.is_null() {
                (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
            } else {
                (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
            }
            *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
            (*wp).zentry.tqe_next = (*owp).zentry.tqe_next;
            if !(*wp).zentry.tqe_next.is_null() {
                (*(*wp).zentry.tqe_next).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
            } else {
                (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
            }
            (*owp).zentry.tqe_next = wp;
            (*wp).zentry.tqe_prev = &raw mut (*owp).zentry.tqe_next;
        }
    } else if strcmp(
        position,
        b"forward-loop\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        owp = *(*((*wp).zentry.tqe_prev as *mut window_panes_zindex)).tqh_last;
        if !(*wp).zentry.tqe_next.is_null() {
            (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
        } else {
            (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
        }
        *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
        if !owp.is_null() {
            (*wp).zentry.tqe_prev = (*owp).zentry.tqe_prev;
            (*wp).zentry.tqe_next = owp;
            *(*owp).zentry.tqe_prev = wp;
            (*owp).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
        } else {
            owp = (*w).z_index.tqh_first;
            while !owp.is_null() {
                if window_pane_is_floating(owp) == 0 {
                    break;
                }
                owp = (*owp).zentry.tqe_next;
            }
            if !owp.is_null() {
                (*wp).zentry.tqe_prev = (*owp).zentry.tqe_prev;
                (*wp).zentry.tqe_next = owp;
                *(*owp).zentry.tqe_prev = wp;
                (*owp).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
            } else {
                (*wp).zentry.tqe_next = ::core::ptr::null_mut::<window_pane>();
                (*wp).zentry.tqe_prev = (*w).z_index.tqh_last;
                *(*w).z_index.tqh_last = wp;
                (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
            }
        }
    } else if strcmp(
        position,
        b"backward-loop\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        owp = (*wp).zentry.tqe_next;
        if !owp.is_null() && window_pane_is_floating(owp) != 0 {
            if !(*wp).zentry.tqe_next.is_null() {
                (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
            } else {
                (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
            }
            *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
            (*wp).zentry.tqe_next = (*owp).zentry.tqe_next;
            if !(*wp).zentry.tqe_next.is_null() {
                (*(*wp).zentry.tqe_next).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
            } else {
                (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
            }
            (*owp).zentry.tqe_next = wp;
            (*wp).zentry.tqe_prev = &raw mut (*owp).zentry.tqe_next;
        } else {
            if !(*wp).zentry.tqe_next.is_null() {
                (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
            } else {
                (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
            }
            *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
            (*wp).zentry.tqe_next = (*w).z_index.tqh_first;
            if !(*wp).zentry.tqe_next.is_null() {
                (*(*w).z_index.tqh_first).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
            } else {
                (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
            }
            (*w).z_index.tqh_first = wp;
            (*wp).zentry.tqe_prev = &raw mut (*w).z_index.tqh_first;
        }
    } else {
        cmdq_error(
            item,
            b"unknown position: %s\0" as *const u8 as *const ::core::ffi::c_char,
            position,
        );
        return CMD_RETURN_ERROR;
    }
    if xoff != (*lc).g.xoff || yoff != (*lc).g.yoff {
        (*lc).g.xoff = xoff;
        (*lc).g.yoff = yoff;
        layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    }
    redraw_invalidate_scene(w);
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_join_pane_move(
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) -> cmd_retval {
    let mut w: *mut window = (*wl).window;
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut argval: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let flags: [::core::ffi::c_char; 4] = [
        'U' as i32 as ::core::ffi::c_char,
        'D' as i32 as ::core::ffi::c_char,
        'L' as i32 as ::core::ffi::c_char,
        'R' as i32 as ::core::ffi::c_char,
    ];
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut flag: ::core::ffi::c_char = 0;
    let mut xoff: ::core::ffi::c_int = (*lc).g.xoff;
    let mut yoff: ::core::ffi::c_int = (*lc).g.yoff;
    let mut adjust: ::core::ffi::c_int = 0;
    let mut i: u_int = 0;
    let mut lines: pane_lines = window_pane_get_pane_lines(wp);
    if args_has(args, 'X' as i32 as u_char) != 0 {
        xoff = args_percentage_and_expand(
            args,
            'X' as i32 as u_char,
            -((*w).sx as ::core::ffi::c_int) as ::core::ffi::c_longlong,
            (*w).sx as ::core::ffi::c_longlong,
            (*w).sx as ::core::ffi::c_longlong,
            item,
            &raw mut cause,
        ) as ::core::ffi::c_int;
        if !cause.is_null() {
            cmdq_error(
                item,
                b"position %s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
        if lines as ::core::ffi::c_uint
            != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            xoff += 1 as ::core::ffi::c_int;
        }
    }
    if args_has(args, 'Y' as i32 as u_char) != 0 {
        yoff = args_percentage_and_expand(
            args,
            'Y' as i32 as u_char,
            -((*w).sy as ::core::ffi::c_int) as ::core::ffi::c_longlong,
            (*w).sy as ::core::ffi::c_longlong,
            (*w).sy as ::core::ffi::c_longlong,
            item,
            &raw mut cause,
        ) as ::core::ffi::c_int;
        if !cause.is_null() {
            cmdq_error(
                item,
                b"position %s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
        if lines as ::core::ffi::c_uint
            != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            yoff += 1 as ::core::ffi::c_int;
        }
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>() as usize)
    {
        flag = flags[i as usize];
        if !(args_has(args, flag as u_char) == 0) {
            argval = args_get(args, flag as u_char);
            if argval.is_null() {
                argval = b"1\0" as *const u8 as *const ::core::ffi::c_char;
            }
            adjust = strtonum(
                argval,
                INT_MIN as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as ::core::ffi::c_int;
            if !errstr.is_null() {
                cmdq_error(
                    item,
                    b"offset %s\0" as *const u8 as *const ::core::ffi::c_char,
                    errstr,
                );
                return CMD_RETURN_ERROR;
            }
            if flag as ::core::ffi::c_int == 'U' as i32 {
                yoff -= adjust;
            } else if flag as ::core::ffi::c_int == 'D' as i32 {
                yoff += adjust;
            } else if flag as ::core::ffi::c_int == 'L' as i32 {
                xoff -= adjust;
            } else {
                xoff += adjust;
            }
        }
        i = i.wrapping_add(1);
    }
    if xoff != (*lc).g.xoff || yoff != (*lc).g.yoff {
        (*lc).g.xoff = xoff;
        (*lc).g.yoff = yoff;
        layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
        events_fire_window(
            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
            w,
        );
        server_redraw_window(w);
    }
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_join_pane_mouse_update(mut item: *mut cmdq_item) -> cmd_retval {
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut event: *mut key_event = cmdq_get_event(item);
    let mut c: *mut client = cmdq_get_client(item);
    let mut s: *mut session = (*target).s;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if (*event).m.valid == 0 {
        return CMD_RETURN_NORMAL;
    }
    wp = cmd_mouse_pane(&raw mut (*event).m, &raw mut s, &raw mut wl);
    if wp.is_null() || c.is_null() || (*c).session != s {
        return CMD_RETURN_NORMAL;
    }
    if window_pane_is_floating(wp) == 0 {
        return CMD_RETURN_NORMAL;
    }
    w = (*wl).window;
    window_redraw_active_switch(w, wp);
    window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
    (*c).tty.mouse_drag_update =
        Some(cmd_join_pane_mouse_move as unsafe extern "C" fn(*mut client, *mut mouse_event) -> ())
            as Option<unsafe extern "C" fn(*mut client, *mut mouse_event) -> ()>;
    cmd_join_pane_mouse_move(c, &raw mut (*event).m);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_join_pane_mouse_move(mut c: *mut client, mut m: *mut mouse_event) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut y: ::core::ffi::c_int = 0;
    let mut ly: ::core::ffi::c_int = 0;
    let mut x: ::core::ffi::c_int = 0;
    let mut lx: ::core::ffi::c_int = 0;
    wp = cmd_mouse_pane(m, ::core::ptr::null_mut::<*mut session>(), &raw mut wl);
    if wp.is_null() {
        (*c).tty.mouse_drag_update = None;
        return;
    }
    w = (*wl).window;
    lc = (*wp).layout_cell as *mut layout_cell;
    y = (*m).y.wrapping_add((*m).oy) as ::core::ffi::c_int;
    x = (*m).x.wrapping_add((*m).ox) as ::core::ffi::c_int;
    if (*m).statusat == 0 as ::core::ffi::c_int && y >= (*m).statuslines as ::core::ffi::c_int {
        y = (y as u_int).wrapping_sub((*m).statuslines) as ::core::ffi::c_int as ::core::ffi::c_int;
    } else if (*m).statusat > 0 as ::core::ffi::c_int && y >= (*m).statusat {
        y = (*m).statusat - 1 as ::core::ffi::c_int;
    }
    ly = (*m).ly.wrapping_add((*m).oy) as ::core::ffi::c_int;
    lx = (*m).lx.wrapping_add((*m).ox) as ::core::ffi::c_int;
    if (*m).statusat == 0 as ::core::ffi::c_int && ly >= (*m).statuslines as ::core::ffi::c_int {
        ly = (ly as u_int).wrapping_sub((*m).statuslines) as ::core::ffi::c_int
            as ::core::ffi::c_int;
    } else if (*m).statusat > 0 as ::core::ffi::c_int && ly >= (*m).statusat {
        ly = (*m).statusat - 1 as ::core::ffi::c_int;
    }
    if x != lx || y != ly {
        (*lc).g.xoff += x - lx;
        (*lc).g.yoff += y - ly;
        layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
        server_redraw_window(w);
        server_redraw_window_borders(w);
    }
}
unsafe extern "C" fn cmd_join_pane_zindex(
    mut item: *mut cmdq_item,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut s: *const ::core::ffi::c_char,
) -> cmd_retval {
    let mut w: *mut window = (*wl).window;
    let mut owp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: u_int = 0;
    let mut z: u_int = 0;
    z = strtonum(
        s,
        0 as ::core::ffi::c_longlong,
        UINT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as u_int;
    if !errstr.is_null() {
        cmdq_error(
            item,
            b"z-index %s\0" as *const u8 as *const ::core::ffi::c_char,
            errstr,
        );
        return CMD_RETURN_ERROR;
    }
    if !(*wp).zentry.tqe_next.is_null() {
        (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
    } else {
        (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
    }
    *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
    n = 0 as u_int;
    owp = (*w).z_index.tqh_first;
    while !owp.is_null() {
        if window_pane_is_floating(owp) == 0 {
            break;
        }
        if n >= z {
            break;
        }
        n = n.wrapping_add(1);
        owp = (*owp).zentry.tqe_next;
    }
    if !owp.is_null() {
        (*wp).zentry.tqe_prev = (*owp).zentry.tqe_prev;
        (*wp).zentry.tqe_next = owp;
        *(*owp).zentry.tqe_prev = wp;
        (*owp).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
    } else {
        (*wp).zentry.tqe_next = ::core::ptr::null_mut::<window_pane>();
        (*wp).zentry.tqe_prev = (*w).z_index.tqh_last;
        *(*w).z_index.tqh_last = wp;
        (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
    }
    redraw_invalidate_scene(w);
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_join_pane_tile(
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut w: *mut window,
    mut wp: *mut window_pane,
) -> cmd_retval {
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    if window_pane_is_floating(wp) == 0 {
        cmdq_error(
            item,
            b"pane is not floating\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if (*w).flags & WINDOW_ZOOMED != 0 {
        cmdq_error(
            item,
            b"can't tile a pane while window is zoomed\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    (*lc).fg.sx = (*lc).g.sx;
    (*lc).fg.sy = (*lc).g.sy;
    (*lc).fg.xoff = (*lc).g.xoff;
    (*lc).fg.yoff = (*lc).g.yoff;
    if layout_insert_tile(w, lc) != 0 as ::core::ffi::c_int {
        cmdq_error(
            item,
            b"no space for a new pane\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    (*lc).flags &= !LAYOUT_CELL_FLOATING;
    if !(*wp).zentry.tqe_next.is_null() {
        (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
    } else {
        (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
    }
    *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
    (*wp).zentry.tqe_next = ::core::ptr::null_mut::<window_pane>();
    (*wp).zentry.tqe_prev = (*w).z_index.tqh_last;
    *(*w).z_index.tqh_last = wp;
    (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
    if args_has(args, 'd' as i32 as u_char) == 0 {
        window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
    }
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    redraw_invalidate_scene(w);
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_join_pane_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut current: *mut cmd_find_state = cmdq_get_current(item);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut source: *mut cmd_find_state = cmdq_get_source(item);
    let mut dst_s: *mut session = ::core::ptr::null_mut::<session>();
    let mut src_wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut dst_wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut src_w: *mut window = ::core::ptr::null_mut::<window>();
    let mut dst_w: *mut window = ::core::ptr::null_mut::<window>();
    let mut src_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut dst_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut dst_idx: ::core::ffi::c_int = 0;
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    dst_s = (*target).s;
    dst_wl = (*target).wl;
    dst_wp = (*target).wp;
    dst_w = (*dst_wl).window;
    dst_idx = (*dst_wl).idx;
    if cmd_get_entry(self_0) == &raw const cmd_move_pane_entry {
        if args_has(args, 'M' as i32 as u_char) != 0 {
            return cmd_join_pane_mouse_update(item);
        }
        if args_has(args, 'P' as i32 as u_char) != 0
            || args_has(args, 'z' as i32 as u_char) != 0
            || args_has(args, 'X' as i32 as u_char) != 0
            || args_has(args, 'Y' as i32 as u_char) != 0
            || args_has(args, 'U' as i32 as u_char) != 0
            || args_has(args, 'D' as i32 as u_char) != 0
            || args_has(args, 'L' as i32 as u_char) != 0
            || args_has(args, 'R' as i32 as u_char) != 0
        {
            if window_pane_is_floating(dst_wp) == 0 {
                cmdq_error(
                    item,
                    b"pane is not floating\0" as *const u8 as *const ::core::ffi::c_char,
                );
                return CMD_RETURN_ERROR;
            }
            server_unzoom_window(dst_w);
            s = args_get(args, 'P' as i32 as u_char);
            if !s.is_null() {
                return cmd_join_pane_place(item, dst_wl, dst_wp, s);
            }
            s = args_get(args, 'z' as i32 as u_char);
            if !s.is_null() {
                return cmd_join_pane_zindex(item, dst_wl, dst_wp, s);
            }
            return cmd_join_pane_move(item, args, dst_wl, dst_wp);
        }
    }
    src_wl = (*source).wl;
    src_wp = (*source).wp;
    src_w = (*src_wl).window;
    if src_wp == (*src_w).modal || dst_wp == (*dst_w).modal {
        cmdq_error(
            item,
            b"pane is modal\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    server_unzoom_window(dst_w);
    server_unzoom_window(src_w);
    if src_wp == dst_wp {
        if window_pane_is_floating(src_wp) != 0 {
            return cmd_join_pane_tile(item, args, src_w, src_wp);
        }
        cmdq_error(
            item,
            b"source and target panes must be different\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if args_has(args, 'h' as i32 as u_char) != 0 {
        flags |= SPAWN_HORIZONTAL;
    }
    if args_has(args, 'b' as i32 as u_char) != 0 {
        flags |= SPAWN_BEFORE;
    }
    if args_has(args, 'f' as i32 as u_char) != 0 {
        flags |= SPAWN_FULLSIZE;
    }
    lc = layout_get_tiled_cell(item, args, dst_w, dst_wp, flags, &raw mut cause);
    if !cause.is_null() {
        cmdq_error(
            item,
            b"size or position %s\0" as *const u8 as *const ::core::ffi::c_char,
            cause,
        );
        free(cause as *mut ::core::ffi::c_void);
        return CMD_RETURN_ERROR;
    }
    layout_close_pane(src_wp);
    server_client_remove_pane(src_wp);
    window_lost_pane(src_w, src_wp);
    if !(*src_wp).entry.tqe_next.is_null() {
        (*(*src_wp).entry.tqe_next).entry.tqe_prev = (*src_wp).entry.tqe_prev;
    } else {
        (*src_w).panes.tqh_last = (*src_wp).entry.tqe_prev;
    }
    *(*src_wp).entry.tqe_prev = (*src_wp).entry.tqe_next;
    if !(*src_wp).zentry.tqe_next.is_null() {
        (*(*src_wp).zentry.tqe_next).zentry.tqe_prev = (*src_wp).zentry.tqe_prev;
    } else {
        (*src_w).z_index.tqh_last = (*src_wp).zentry.tqe_prev;
    }
    *(*src_wp).zentry.tqe_prev = (*src_wp).zentry.tqe_next;
    (*src_wp).window = dst_w as *mut window;
    options_set_parent((*src_wp).options, (*dst_w).options);
    (*src_wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
    if flags & SPAWN_BEFORE != 0 {
        (*src_wp).entry.tqe_prev = (*dst_wp).entry.tqe_prev;
        (*src_wp).entry.tqe_next = dst_wp;
        *(*dst_wp).entry.tqe_prev = src_wp;
        (*dst_wp).entry.tqe_prev = &raw mut (*src_wp).entry.tqe_next;
        (*src_wp).zentry.tqe_prev = (*dst_wp).zentry.tqe_prev;
        (*src_wp).zentry.tqe_next = dst_wp;
        *(*dst_wp).zentry.tqe_prev = src_wp;
        (*dst_wp).zentry.tqe_prev = &raw mut (*src_wp).zentry.tqe_next;
    } else {
        (*src_wp).entry.tqe_next = (*dst_wp).entry.tqe_next;
        if !(*src_wp).entry.tqe_next.is_null() {
            (*(*src_wp).entry.tqe_next).entry.tqe_prev = &raw mut (*src_wp).entry.tqe_next;
        } else {
            (*dst_w).panes.tqh_last = &raw mut (*src_wp).entry.tqe_next;
        }
        (*dst_wp).entry.tqe_next = src_wp;
        (*src_wp).entry.tqe_prev = &raw mut (*dst_wp).entry.tqe_next;
        (*src_wp).zentry.tqe_next = (*dst_wp).zentry.tqe_next;
        if !(*src_wp).zentry.tqe_next.is_null() {
            (*(*src_wp).zentry.tqe_next).zentry.tqe_prev = &raw mut (*src_wp).zentry.tqe_next;
        } else {
            (*dst_w).z_index.tqh_last = &raw mut (*src_wp).zentry.tqe_next;
        }
        (*dst_wp).zentry.tqe_next = src_wp;
        (*src_wp).zentry.tqe_prev = &raw mut (*dst_wp).zentry.tqe_next;
    }
    layout_assign_pane(lc, src_wp, 0 as ::core::ffi::c_int);
    colour_palette_from_option(&raw mut (*src_wp).palette, (*src_wp).options);
    recalculate_sizes();
    server_redraw_window(src_w);
    server_redraw_window(dst_w);
    if args_has(args, 'd' as i32 as u_char) == 0 {
        window_set_active_pane(dst_w, src_wp, 1 as ::core::ffi::c_int);
        session_select(dst_s, dst_idx);
        cmd_find_from_session(current, dst_s, 0 as ::core::ffi::c_int);
        server_redraw_session(dst_s);
    } else {
        server_status_session(dst_s);
    }
    window_fire_pane_moved(src_wp, src_w, (*src_wl).idx, dst_w, dst_idx);
    if window_count_panes(src_w, 1 as ::core::ffi::c_int) == 0 as u_int {
        server_kill_window(src_w, 1 as ::core::ffi::c_int);
    } else {
        events_fire_window(
            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
            src_w,
        );
    }
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        dst_w,
    );
    return CMD_RETURN_NORMAL;
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

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
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strlcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
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
    fn tty_draw_line(
        _: *mut tty,
        _: *mut screen,
        _: u_int,
        _: u_int,
        _: u_int,
        _: u_int,
        _: u_int,
        _: *const tty_style_ctx,
    );
    fn tty_window_offset(
        _: *mut tty,
        _: *mut u_int,
        _: *mut u_int,
        _: *mut u_int,
        _: *mut u_int,
    ) -> ::core::ffi::c_int;
    fn tty_reset(_: *mut tty);
    fn tty_cursor(_: *mut tty, _: u_int, _: u_int);
    fn tty_puts(_: *mut tty, _: *const ::core::ffi::c_char);
    fn tty_cell(_: *mut tty, _: *const grid_cell, _: *const tty_style_ctx);
    fn tty_update_mode(_: *mut tty, _: ::core::ffi::c_int, _: *mut screen);
    fn tty_check_overlay_range(_: *mut tty, _: u_int, _: u_int, _: u_int) -> *mut visible_ranges;
    fn tty_sync_start(_: *mut tty);
    fn tty_default_colours(_: *mut grid_cell, _: *mut window_pane, _: *mut u_int);
    fn tty_term_has(_: *mut tty_term, _: tty_code_code) -> ::core::ffi::c_int;
    static mut marked_pane: cmd_find_state;
    fn server_is_marked(
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> ::core::ffi::c_int;
    fn status_line_size(_: *mut client) -> u_int;
    fn status_redraw(_: *mut client) -> ::core::ffi::c_int;
    fn status_message_redraw(_: *mut client) -> ::core::ffi::c_int;
    fn status_prompt_redraw(_: *mut client) -> ::core::ffi::c_int;
    fn prompt_draw(_: *mut prompt, _: *mut prompt_draw_data);
    static grid_default_cell: grid_cell;
    fn screen_write_start(_: *mut screen_write_ctx, _: *mut screen);
    fn screen_write_stop(_: *mut screen_write_ctx);
    fn screen_write_stop_sync(_: *mut window_pane);
    fn screen_write_clear_dirty(_: *mut window_pane);
    fn screen_init(_: *mut screen, _: u_int, _: u_int, _: u_int);
    fn screen_free(_: *mut screen);
    static mut windows: windows;
    fn windows_RB_NEXT(_: *mut window) -> *mut window;
    fn windows_RB_MINMAX(_: *mut windows, _: ::core::ffi::c_int) -> *mut window;
    fn window_pane_is_visible(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_mode(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_scrollbar_visible(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_scrollbar_overlay(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_get_pane_lines(_: *mut window_pane) -> pane_lines;
    fn window_pane_get_pane_status(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_is_floating(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_get_border_cell(
        _: *mut window_pane,
        _: pane_lines,
        _: ::core::ffi::c_int,
        _: *mut grid_cell,
    );
    fn window_get_fill_cell(_: *mut window, _: ::core::ffi::c_int, _: *mut grid_cell);
    fn window_pane_get_border_cell(_: *mut window_pane, _: ::core::ffi::c_int, _: *mut grid_cell);
    fn window_pane_get_border_style(_: *mut window_pane, _: *mut client, _: *mut grid_cell);
    fn window_make_pane_status(
        _: *mut window_pane,
        _: *mut client,
        _: u_int,
        _: *mut redraw_span,
    ) -> ::core::ffi::c_int;
    fn window_copy_get_current_offset(
        _: *mut window_pane,
        _: *mut u_int,
        _: *mut u_int,
    ) -> ::core::ffi::c_int;
    fn utf8_set(_: *mut utf8_data, _: u_char);
    fn log_get_level() -> ::core::ffi::c_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
    fn menu_update(_: *mut menu_data);
    fn menu_screen(_: *mut menu_data) -> *mut screen;
    fn menu_width(_: *mut menu_data) -> u_int;
    fn menu_height(_: *mut menu_data) -> u_int;
    fn menu_x(_: *mut menu_data) -> u_int;
    fn menu_y(_: *mut menu_data) -> u_int;
    fn style_add(
        _: *mut grid_cell,
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: *mut format_tree,
    ) -> *mut style;
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
pub type uint8_t = __uint8_t;
pub type uint64_t = __uint64_t;
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
pub struct redraw_scene {
    pub c: *mut client,
    pub w: *mut window,
    pub lines: *mut redraw_line,
    pub generation: uint64_t,
    pub sx: u_int,
    pub sy: u_int,
    pub ox: u_int,
    pub oy: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_line {
    pub spans: [redraw_spans; 7],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_spans {
    pub tqh_first: *mut redraw_span,
    pub tqh_last: *mut *mut redraw_span,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_span {
    pub x: u_int,
    pub width: u_int,
    pub data: redraw_span_data,
    pub entry: C2RustUnnamed_34,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_34 {
    pub tqe_next: *mut redraw_span,
    pub tqe_prev: *mut *mut redraw_span,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_span_data {
    pub type_0: redraw_span_type,
    pub c2rust_unnamed: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_35 {
    pub p: C2RustUnnamed_40,
    pub b: C2RustUnnamed_39,
    pub st: C2RustUnnamed_38,
    pub sb: C2RustUnnamed_37,
    pub m: C2RustUnnamed_36,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_36 {
    pub md: *mut menu_data,
    pub px: u_int,
    pub py: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
    pub wp: *mut window_pane,
    pub y: u_int,
    pub height: u_int,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_38 {
    pub wp: *mut window_pane,
    pub offset: u_int,
    pub cell_type: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_39 {
    pub top_wp: *mut window_pane,
    pub bottom_wp: *mut window_pane,
    pub left_wp: *mut window_pane,
    pub right_wp: *mut window_pane,
    pub style_wp: *mut window_pane,
    pub cell_type: ::core::ffi::c_int,
    pub cell_mask: ::core::ffi::c_int,
    pub top_lines: pane_lines,
    pub bottom_lines: pane_lines,
    pub left_lines: pane_lines,
    pub right_lines: pane_lines,
    pub flags: ::core::ffi::c_int,
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
pub struct C2RustUnnamed_40 {
    pub wp: *mut window_pane,
    pub px: u_int,
    pub py: u_int,
}
pub type redraw_span_type = ::core::ffi::c_uint;
pub const REDRAW_SPAN_MENU: redraw_span_type = 6;
pub const REDRAW_SPAN_SCROLLBAR: redraw_span_type = 5;
pub const REDRAW_SPAN_BORDER: redraw_span_type = 4;
pub const REDRAW_SPAN_STATUS: redraw_span_type = 3;
pub const REDRAW_SPAN_EMPTY: redraw_span_type = 2;
pub const REDRAW_SPAN_OUTSIDE: redraw_span_type = 1;
pub const REDRAW_SPAN_PANE: redraw_span_type = 0;
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
    pub entry: C2RustUnnamed_41,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_41 {
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
    pub c2rust_unnamed: C2RustUnnamed_42,
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
pub union C2RustUnnamed_42 {
    pub n: u_int,
    pub data: C2RustUnnamed_44,
    pub sel: C2RustUnnamed_43,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_43 {
    pub clip: *const ::core::ffi::c_char,
    pub data: *const ::core::ffi::c_char,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_44 {
    pub data: *const ::core::ffi::c_char,
    pub size: size_t,
}
pub type tty_ctx_set_client_cb =
    Option<unsafe extern "C" fn(*mut tty_ctx, *mut client) -> ::core::ffi::c_int>;
pub type tty_ctx_redraw_cb = Option<unsafe extern "C" fn(*const tty_ctx) -> ()>;
pub type tty_code_code = ::core::ffi::c_uint;
pub const TTYC_XT: tty_code_code = 233;
pub const TTYC_VPA: tty_code_code = 232;
pub const TTYC_U8: tty_code_code = 231;
pub const TTYC_TSL: tty_code_code = 230;
pub const TTYC_TC: tty_code_code = 229;
pub const TTYC_SYNC: tty_code_code = 228;
pub const TTYC_SWD: tty_code_code = 227;
pub const TTYC_SS: tty_code_code = 226;
pub const TTYC_SXL: tty_code_code = 225;
pub const TTYC_SPB: tty_code_code = 224;
pub const TTYC_SMXX: tty_code_code = 223;
pub const TTYC_SMULX: tty_code_code = 222;
pub const TTYC_SMUL: tty_code_code = 221;
pub const TTYC_SMSO: tty_code_code = 220;
pub const TTYC_SMOL: tty_code_code = 219;
pub const TTYC_SMKX: tty_code_code = 218;
pub const TTYC_SMCUP: tty_code_code = 217;
pub const TTYC_SMACS: tty_code_code = 216;
pub const TTYC_SITM: tty_code_code = 215;
pub const TTYC_SGR0: tty_code_code = 214;
pub const TTYC_SETULC1: tty_code_code = 213;
pub const TTYC_SETULC: tty_code_code = 212;
pub const TTYC_SETRGBF: tty_code_code = 211;
pub const TTYC_SETRGBB: tty_code_code = 210;
pub const TTYC_SETAL: tty_code_code = 209;
pub const TTYC_SETAF: tty_code_code = 208;
pub const TTYC_SETAB: tty_code_code = 207;
pub const TTYC_SE: tty_code_code = 206;
pub const TTYC_RMKX: tty_code_code = 205;
pub const TTYC_RMCUP: tty_code_code = 204;
pub const TTYC_RMACS: tty_code_code = 203;
pub const TTYC_RIN: tty_code_code = 202;
pub const TTYC_RI: tty_code_code = 201;
pub const TTYC_RGB: tty_code_code = 200;
pub const TTYC_REV: tty_code_code = 199;
pub const TTYC_RECT: tty_code_code = 198;
pub const TTYC_OP: tty_code_code = 197;
pub const TTYC_OL: tty_code_code = 196;
pub const TTYC_NOBR: tty_code_code = 195;
pub const TTYC_MS: tty_code_code = 194;
pub const TTYC_KUP7: tty_code_code = 193;
pub const TTYC_KUP6: tty_code_code = 192;
pub const TTYC_KUP5: tty_code_code = 191;
pub const TTYC_KUP4: tty_code_code = 190;
pub const TTYC_KUP3: tty_code_code = 189;
pub const TTYC_KUP2: tty_code_code = 188;
pub const TTYC_KRIT7: tty_code_code = 187;
pub const TTYC_KRIT6: tty_code_code = 186;
pub const TTYC_KRIT5: tty_code_code = 185;
pub const TTYC_KRIT4: tty_code_code = 184;
pub const TTYC_KRIT3: tty_code_code = 183;
pub const TTYC_KRIT2: tty_code_code = 182;
pub const TTYC_KRI: tty_code_code = 181;
pub const TTYC_KPRV7: tty_code_code = 180;
pub const TTYC_KPRV6: tty_code_code = 179;
pub const TTYC_KPRV5: tty_code_code = 178;
pub const TTYC_KPRV4: tty_code_code = 177;
pub const TTYC_KPRV3: tty_code_code = 176;
pub const TTYC_KPRV2: tty_code_code = 175;
pub const TTYC_KPP: tty_code_code = 174;
pub const TTYC_KNXT7: tty_code_code = 173;
pub const TTYC_KNXT6: tty_code_code = 172;
pub const TTYC_KNXT5: tty_code_code = 171;
pub const TTYC_KNXT4: tty_code_code = 170;
pub const TTYC_KNXT3: tty_code_code = 169;
pub const TTYC_KNXT2: tty_code_code = 168;
pub const TTYC_KNP: tty_code_code = 167;
pub const TTYC_KMOUS: tty_code_code = 166;
pub const TTYC_KLFT7: tty_code_code = 165;
pub const TTYC_KLFT6: tty_code_code = 164;
pub const TTYC_KLFT5: tty_code_code = 163;
pub const TTYC_KLFT4: tty_code_code = 162;
pub const TTYC_KLFT3: tty_code_code = 161;
pub const TTYC_KLFT2: tty_code_code = 160;
pub const TTYC_KIND: tty_code_code = 159;
pub const TTYC_KICH1: tty_code_code = 158;
pub const TTYC_KIC7: tty_code_code = 157;
pub const TTYC_KIC6: tty_code_code = 156;
pub const TTYC_KIC5: tty_code_code = 155;
pub const TTYC_KIC4: tty_code_code = 154;
pub const TTYC_KIC3: tty_code_code = 153;
pub const TTYC_KIC2: tty_code_code = 152;
pub const TTYC_KHOME: tty_code_code = 151;
pub const TTYC_KHOM7: tty_code_code = 150;
pub const TTYC_KHOM6: tty_code_code = 149;
pub const TTYC_KHOM5: tty_code_code = 148;
pub const TTYC_KHOM4: tty_code_code = 147;
pub const TTYC_KHOM3: tty_code_code = 146;
pub const TTYC_KHOM2: tty_code_code = 145;
pub const TTYC_KF9: tty_code_code = 144;
pub const TTYC_KF8: tty_code_code = 143;
pub const TTYC_KF7: tty_code_code = 142;
pub const TTYC_KF63: tty_code_code = 141;
pub const TTYC_KF62: tty_code_code = 140;
pub const TTYC_KF61: tty_code_code = 139;
pub const TTYC_KF60: tty_code_code = 138;
pub const TTYC_KF6: tty_code_code = 137;
pub const TTYC_KF59: tty_code_code = 136;
pub const TTYC_KF58: tty_code_code = 135;
pub const TTYC_KF57: tty_code_code = 134;
pub const TTYC_KF56: tty_code_code = 133;
pub const TTYC_KF55: tty_code_code = 132;
pub const TTYC_KF54: tty_code_code = 131;
pub const TTYC_KF53: tty_code_code = 130;
pub const TTYC_KF52: tty_code_code = 129;
pub const TTYC_KF51: tty_code_code = 128;
pub const TTYC_KF50: tty_code_code = 127;
pub const TTYC_KF5: tty_code_code = 126;
pub const TTYC_KF49: tty_code_code = 125;
pub const TTYC_KF48: tty_code_code = 124;
pub const TTYC_KF47: tty_code_code = 123;
pub const TTYC_KF46: tty_code_code = 122;
pub const TTYC_KF45: tty_code_code = 121;
pub const TTYC_KF44: tty_code_code = 120;
pub const TTYC_KF43: tty_code_code = 119;
pub const TTYC_KF42: tty_code_code = 118;
pub const TTYC_KF41: tty_code_code = 117;
pub const TTYC_KF40: tty_code_code = 116;
pub const TTYC_KF4: tty_code_code = 115;
pub const TTYC_KF39: tty_code_code = 114;
pub const TTYC_KF38: tty_code_code = 113;
pub const TTYC_KF37: tty_code_code = 112;
pub const TTYC_KF36: tty_code_code = 111;
pub const TTYC_KF35: tty_code_code = 110;
pub const TTYC_KF34: tty_code_code = 109;
pub const TTYC_KF33: tty_code_code = 108;
pub const TTYC_KF32: tty_code_code = 107;
pub const TTYC_KF31: tty_code_code = 106;
pub const TTYC_KF30: tty_code_code = 105;
pub const TTYC_KF3: tty_code_code = 104;
pub const TTYC_KF29: tty_code_code = 103;
pub const TTYC_KF28: tty_code_code = 102;
pub const TTYC_KF27: tty_code_code = 101;
pub const TTYC_KF26: tty_code_code = 100;
pub const TTYC_KF25: tty_code_code = 99;
pub const TTYC_KF24: tty_code_code = 98;
pub const TTYC_KF23: tty_code_code = 97;
pub const TTYC_KF22: tty_code_code = 96;
pub const TTYC_KF21: tty_code_code = 95;
pub const TTYC_KF20: tty_code_code = 94;
pub const TTYC_KF2: tty_code_code = 93;
pub const TTYC_KF19: tty_code_code = 92;
pub const TTYC_KF18: tty_code_code = 91;
pub const TTYC_KF17: tty_code_code = 90;
pub const TTYC_KF16: tty_code_code = 89;
pub const TTYC_KF15: tty_code_code = 88;
pub const TTYC_KF14: tty_code_code = 87;
pub const TTYC_KF13: tty_code_code = 86;
pub const TTYC_KF12: tty_code_code = 85;
pub const TTYC_KF11: tty_code_code = 84;
pub const TTYC_KF10: tty_code_code = 83;
pub const TTYC_KF1: tty_code_code = 82;
pub const TTYC_KEND7: tty_code_code = 81;
pub const TTYC_KEND6: tty_code_code = 80;
pub const TTYC_KEND5: tty_code_code = 79;
pub const TTYC_KEND4: tty_code_code = 78;
pub const TTYC_KEND3: tty_code_code = 77;
pub const TTYC_KEND2: tty_code_code = 76;
pub const TTYC_KEND: tty_code_code = 75;
pub const TTYC_KDN7: tty_code_code = 74;
pub const TTYC_KDN6: tty_code_code = 73;
pub const TTYC_KDN5: tty_code_code = 72;
pub const TTYC_KDN4: tty_code_code = 71;
pub const TTYC_KDN3: tty_code_code = 70;
pub const TTYC_KDN2: tty_code_code = 69;
pub const TTYC_KDCH1: tty_code_code = 68;
pub const TTYC_KDC7: tty_code_code = 67;
pub const TTYC_KDC6: tty_code_code = 66;
pub const TTYC_KDC5: tty_code_code = 65;
pub const TTYC_KDC4: tty_code_code = 64;
pub const TTYC_KDC3: tty_code_code = 63;
pub const TTYC_KDC2: tty_code_code = 62;
pub const TTYC_KCUU1: tty_code_code = 61;
pub const TTYC_KCUF1: tty_code_code = 60;
pub const TTYC_KCUD1: tty_code_code = 59;
pub const TTYC_KCUB1: tty_code_code = 58;
pub const TTYC_KCBT: tty_code_code = 57;
pub const TTYC_INVIS: tty_code_code = 56;
pub const TTYC_INDN: tty_code_code = 55;
pub const TTYC_IND: tty_code_code = 54;
pub const TTYC_IL1: tty_code_code = 53;
pub const TTYC_IL: tty_code_code = 52;
pub const TTYC_ICH1: tty_code_code = 51;
pub const TTYC_ICH: tty_code_code = 50;
pub const TTYC_HPA: tty_code_code = 49;
pub const TTYC_HOME: tty_code_code = 48;
pub const TTYC_HLS: tty_code_code = 47;
pub const TTYC_FSL: tty_code_code = 46;
pub const TTYC_ENMG: tty_code_code = 45;
pub const TTYC_ENFCS: tty_code_code = 44;
pub const TTYC_ENEKS: tty_code_code = 43;
pub const TTYC_ENBP: tty_code_code = 42;
pub const TTYC_ENACS: tty_code_code = 41;
pub const TTYC_EL1: tty_code_code = 40;
pub const TTYC_EL: tty_code_code = 39;
pub const TTYC_ED: tty_code_code = 38;
pub const TTYC_ECH: tty_code_code = 37;
pub const TTYC_E3: tty_code_code = 36;
pub const TTYC_DSMG: tty_code_code = 35;
pub const TTYC_DSFCS: tty_code_code = 34;
pub const TTYC_DSEKS: tty_code_code = 33;
pub const TTYC_DSBP: tty_code_code = 32;
pub const TTYC_DL1: tty_code_code = 31;
pub const TTYC_DL: tty_code_code = 30;
pub const TTYC_DIM: tty_code_code = 29;
pub const TTYC_DCH1: tty_code_code = 28;
pub const TTYC_DCH: tty_code_code = 27;
pub const TTYC_CVVIS: tty_code_code = 26;
pub const TTYC_CUU1: tty_code_code = 25;
pub const TTYC_CUU: tty_code_code = 24;
pub const TTYC_CUP: tty_code_code = 23;
pub const TTYC_CUF1: tty_code_code = 22;
pub const TTYC_CUF: tty_code_code = 21;
pub const TTYC_CUD1: tty_code_code = 20;
pub const TTYC_CUD: tty_code_code = 19;
pub const TTYC_CUB1: tty_code_code = 18;
pub const TTYC_CUB: tty_code_code = 17;
pub const TTYC_CSR: tty_code_code = 16;
pub const TTYC_CS: tty_code_code = 15;
pub const TTYC_CR: tty_code_code = 14;
pub const TTYC_COLORS: tty_code_code = 13;
pub const TTYC_CNORM: tty_code_code = 12;
pub const TTYC_CMG: tty_code_code = 11;
pub const TTYC_CLMG: tty_code_code = 10;
pub const TTYC_CLEAR: tty_code_code = 9;
pub const TTYC_CIVIS: tty_code_code = 8;
pub const TTYC_BOLD: tty_code_code = 7;
pub const TTYC_BLINK: tty_code_code = 6;
pub const TTYC_BIDI: tty_code_code = 5;
pub const TTYC_BEL: tty_code_code = 4;
pub const TTYC_BCE: tty_code_code = 3;
pub const TTYC_AX: tty_code_code = 2;
pub const TTYC_AM: tty_code_code = 1;
pub const TTYC_ACSC: tty_code_code = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_panes_zindex {
    pub tqh_first: *mut window_pane,
    pub tqh_last: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct windows {
    pub rbh_root: *mut window,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct prompt_draw_data {
    pub ctx: *mut screen_write_ctx,
    pub cursor_x: *mut u_int,
    pub area_x: u_int,
    pub area_width: u_int,
    pub prompt_line: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_draw_ctx {
    pub scene: *mut redraw_scene,
    pub active: *mut window_pane,
    pub marked: *mut window_pane,
    pub status_lines: u_int,
    pub pane_lines: pane_lines,
    pub default_gc: grid_cell,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_build_cell {
    pub data: redraw_span_data,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct redraw_build_ctx {
    pub c: *mut client,
    pub w: *mut window,
    pub ox: u_int,
    pub oy: u_int,
    pub sx: u_int,
    pub sy: u_int,
    pub ind: ::core::ffi::c_int,
    pub cells: *mut redraw_build_cell,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const MODE_CURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MODE_CURSOR_BLINKING: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const MODE_CURSOR_VERY_VISIBLE: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const MODE_SYNC: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;
pub const CURSOR_MODES: ::core::ffi::c_int =
    MODE_CURSOR | MODE_CURSOR_BLINKING | MODE_CURSOR_VERY_VISIBLE;
pub const GRID_ATTR_REVERSE: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const GRID_ATTR_CHARSET: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const CELL_UD: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CELL_LR: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CELL_RD: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const CELL_LD: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const CELL_RU: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const CELL_LU: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const CELL_LRD: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const CELL_LRU: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const CELL_URD: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const CELL_ULD: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const CELL_LRUD: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const CELL_NONE: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const PANE_BORDER_COLOUR: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_BORDER_ARROWS: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PANE_BORDER_BOTH: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const WINDOW_PANE_NO_MODE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_NEWSTATUS: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const PANE_STATUS_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_STATUS_TOP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_STATUS_BOTTOM: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_LEFT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CLIENT_REDRAWWINDOW: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSTATUS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CLIENT_SUSPENDED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLIENT_REDRAWBORDERS: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const CLIENT_UTF8: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWOVERLAY: ::core::ffi::c_int = 0x2000000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWMENU: ::core::ffi::c_int = 0x20000000 as ::core::ffi::c_int;
pub const REDRAW_SPAN_TYPES: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const REDRAW_BORDER_L: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const REDRAW_BORDER_R: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const REDRAW_BORDER_U: ::core::ffi::c_int = 4;
pub const REDRAW_BORDER_D: ::core::ffi::c_int = 8;
pub const REDRAW_BORDER_IS_ARROW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const REDRAW_SCROLLBAR_LEFT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const REDRAW_SCROLLBAR_RIGHT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const REDRAW_SCROLLBAR_OVERLAY: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const REDRAW_PANE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const REDRAW_OUTSIDE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const REDRAW_EMPTY: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const REDRAW_PANE_BORDER: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const REDRAW_PANE_STATUS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const REDRAW_PANE_SCROLLBAR: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const REDRAW_STATUS: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const REDRAW_MENU: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const REDRAW_OVERLAY: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const REDRAW_ALL: ::core::ffi::c_int = 0x7fffffff as ::core::ffi::c_int;
pub const REDRAW_START_ISOLATE: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"\xE2\x81\xA6\0") };
pub const REDRAW_END_ISOLATE: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"\xE2\x81\xA9\0") };
static mut redraw_cells: *mut redraw_build_cell =
    ::core::ptr::null::<redraw_build_cell>() as *mut redraw_build_cell;
static mut redraw_ncells: size_t = 0;
pub const REDRAW_ISOLATES: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const REDRAW_DEFAULT_SET: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const REDRAW_STATUS_TOP: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
unsafe extern "C" fn redraw_flags_to_string(
    mut flags: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 128] = [0; 128];
    *(&raw mut s as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if flags & REDRAW_STATUS != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"status \0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & REDRAW_PANE != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"pane \0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & REDRAW_PANE_BORDER != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"border \0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & REDRAW_PANE_STATUS != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"pane-status \0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & REDRAW_PANE_SCROLLBAR != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"scrollbar \0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & REDRAW_MENU != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"menu \0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags & REDRAW_OVERLAY != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"overlay \0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if flags == REDRAW_ALL {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"all \0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
        );
    }
    if *(&raw mut s as *mut ::core::ffi::c_char) as ::core::ffi::c_int != '\0' as i32 {
        s[strlen(&raw mut s as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
            '\0' as i32 as ::core::ffi::c_char;
    }
    return &raw mut s as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn redraw_get_window_offset(
    mut c: *mut client,
    mut ox: *mut u_int,
    mut oy: *mut u_int,
    mut sx: *mut u_int,
    mut sy: *mut u_int,
) {
    let mut tty_sx: u_int = 0;
    let mut tty_sy: u_int = 0;
    tty_window_offset(&raw mut (*c).tty, ox, oy, sx, sy);
    tty_sx = (*c).tty.sx;
    tty_sy = (*c).tty.sy.wrapping_sub(status_line_size(c));
    if *sx < tty_sx {
        *sx = tty_sx;
    }
    if *sy < tty_sy {
        *sy = tty_sy;
    }
}
unsafe extern "C" fn redraw_set_context(mut c: *mut client, mut bctx: *mut redraw_build_ctx) {
    let mut s: *mut session = (*c).session;
    let mut w: *mut window = (*(*s).curw).window;
    memset(
        bctx as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<redraw_build_ctx>() as size_t,
    );
    (*bctx).c = c;
    (*bctx).w = w;
    redraw_get_window_offset(
        c,
        &raw mut (*bctx).ox,
        &raw mut (*bctx).oy,
        &raw mut (*bctx).sx,
        &raw mut (*bctx).sy,
    );
    (*bctx).ind = options_get_number(
        (*w).options,
        b"pane-border-indicators\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
}
unsafe extern "C" fn redraw_get_build_cell(
    mut bctx: *mut redraw_build_ctx,
    mut x: u_int,
    mut y: u_int,
) -> *mut redraw_build_cell {
    return (*bctx)
        .cells
        .offset(y.wrapping_mul((*bctx).sx).wrapping_add(x) as isize)
        as *mut redraw_build_cell;
}
unsafe extern "C" fn redraw_reset_cell(
    mut bctx: *mut redraw_build_ctx,
    mut x: u_int,
    mut y: u_int,
) {
    let mut bc: *mut redraw_build_cell = redraw_get_build_cell(bctx, x, y);
    let mut w: *mut window = (*bctx).w;
    memset(
        bc as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<redraw_build_cell>() as size_t,
    );
    if (*bctx).ox.wrapping_add(x) < (*w).sx && (*bctx).oy.wrapping_add(y) < (*w).sy {
        (*bc).data.type_0 = REDRAW_SPAN_EMPTY;
    } else {
        (*bc).data.type_0 = REDRAW_SPAN_OUTSIDE;
    };
}
unsafe extern "C" fn redraw_window_to_scene(
    mut bctx: *mut redraw_build_ctx,
    mut wx: ::core::ffi::c_int,
    mut wy: ::core::ffi::c_int,
    mut x: *mut u_int,
    mut y: *mut u_int,
) -> ::core::ffi::c_int {
    let mut sx: ::core::ffi::c_int = 0;
    let mut sy: ::core::ffi::c_int = 0;
    if wx < 0 as ::core::ffi::c_int || wy < 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if wx as u_int > (*(*bctx).w).sx || wy as u_int > (*(*bctx).w).sy {
        return 0 as ::core::ffi::c_int;
    }
    if wx < (*bctx).ox as ::core::ffi::c_int || wy < (*bctx).oy as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    sx = wx - (*bctx).ox as ::core::ffi::c_int;
    sy = wy - (*bctx).oy as ::core::ffi::c_int;
    if sx as u_int >= (*bctx).sx || sy as u_int >= (*bctx).sy {
        return 0 as ::core::ffi::c_int;
    }
    *x = sx as u_int;
    *y = sy as u_int;
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn redraw_pane_to_scene(
    mut bctx: *mut redraw_build_ctx,
    mut wp: *mut window_pane,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
    mut x: *mut u_int,
    mut y: *mut u_int,
) -> ::core::ffi::c_int {
    let mut wx: ::core::ffi::c_int = (*wp).xoff + px;
    let mut wy: ::core::ffi::c_int = (*wp).yoff + py;
    let mut left: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut top: ::core::ffi::c_int = 0;
    let mut bottom: ::core::ffi::c_int = 0;
    if window_pane_is_floating(wp) != 0 {
        left = (*wp).xoff - 1 as ::core::ffi::c_int;
        right = ((*wp).xoff as u_int).wrapping_add((*wp).sx) as ::core::ffi::c_int;
        top = (*wp).yoff - 1 as ::core::ffi::c_int;
        bottom = ((*wp).yoff as u_int).wrapping_add((*wp).sy) as ::core::ffi::c_int;
        if left < 0 as ::core::ffi::c_int && wx < 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if right > (*(*bctx).w).sx as ::core::ffi::c_int
            && wx >= (*(*bctx).w).sx as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
        if top < 0 as ::core::ffi::c_int && wy < 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        if bottom > (*(*bctx).w).sy as ::core::ffi::c_int
            && wy >= (*(*bctx).w).sy as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
    }
    return redraw_window_to_scene(bctx, wx, wy, x, y);
}
unsafe extern "C" fn redraw_get_cell_type(mut mask: ::core::ffi::c_int) -> ::core::ffi::c_int {
    match mask {
        15 => return 11 as ::core::ffi::c_int,
        7 => return 8 as ::core::ffi::c_int,
        11 => return 7 as ::core::ffi::c_int,
        3 | REDRAW_BORDER_L | REDRAW_BORDER_R => return 2 as ::core::ffi::c_int,
        13 => return 10 as ::core::ffi::c_int,
        5 => return 6 as ::core::ffi::c_int,
        9 => return 4 as ::core::ffi::c_int,
        14 => return 9 as ::core::ffi::c_int,
        6 => return 5 as ::core::ffi::c_int,
        10 => return 3 as ::core::ffi::c_int,
        12 | REDRAW_BORDER_U | REDRAW_BORDER_D => return 1 as ::core::ffi::c_int,
        _ => {}
    }
    return 12 as ::core::ffi::c_int;
}
unsafe extern "C" fn redraw_check_two_pane_colours(
    mut w: *mut window,
    mut type_0: *mut layout_type,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut count: u_int = 0 as u_int;
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if !(window_pane_is_floating(wp) != 0 || (*wp).layout_cell.is_null()) {
            count = count.wrapping_add(1);
            if count > 2 as u_int || (*(*wp).layout_cell).parent.is_null() {
                return 0 as ::core::ffi::c_int;
            }
            *type_0 = (*(*(*wp).layout_cell).parent).type_0;
        }
        wp = (*wp).entry.tqe_next;
    }
    return (count == 2 as u_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn redraw_mark_pane_inside(
    mut bctx: *mut redraw_build_ctx,
    mut wp: *mut window_pane,
) {
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    py = 0 as u_int;
    while py < (*wp).sy {
        px = 0 as u_int;
        while px < (*wp).sx {
            if !(redraw_pane_to_scene(
                bctx,
                wp,
                px as ::core::ffi::c_int,
                py as ::core::ffi::c_int,
                &raw mut x,
                &raw mut y,
            ) == 0)
            {
                bc = redraw_get_build_cell(bctx, x, y);
                memset(
                    bc as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    ::core::mem::size_of::<redraw_build_cell>() as size_t,
                );
                (*bc).data.type_0 = REDRAW_SPAN_PANE;
                (*bc).data.c2rust_unnamed.p.wp = wp;
                (*bc).data.c2rust_unnamed.p.px = px;
                (*bc).data.c2rust_unnamed.p.py = py;
            }
            px = px.wrapping_add(1);
        }
        py = py.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_mark_pane_scrollbar(
    mut bctx: *mut redraw_build_ctx,
    mut wp: *mut window_pane,
    mut sb_w: ::core::ffi::c_int,
    mut sb_left: ::core::ffi::c_int,
    mut overlay: ::core::ffi::c_int,
) {
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut wx: ::core::ffi::c_int = 0;
    let mut wy: ::core::ffi::c_int = 0;
    let mut sx: ::core::ffi::c_int = 0;
    let mut ex: ::core::ffi::c_int = 0;
    let mut sy: u_int = 0;
    if sb_w == 0 as ::core::ffi::c_int {
        return;
    }
    if overlay != 0 && sb_left != 0 {
        sx = (*wp).xoff;
        ex = sx + sb_w - 1 as ::core::ffi::c_int;
    } else if overlay != 0 {
        ex = (*wp).xoff + (*wp).sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        sx = ex - sb_w + 1 as ::core::ffi::c_int;
    } else if sb_left != 0 {
        sx = (*wp).xoff - sb_w;
        ex = (*wp).xoff - 1 as ::core::ffi::c_int;
    } else {
        sx = (*wp).xoff + (*wp).sx as ::core::ffi::c_int;
        ex = sx + sb_w - 1 as ::core::ffi::c_int;
    }
    sy = 0 as u_int;
    while sy < (*wp).sy {
        wy = (*wp).yoff + sy as ::core::ffi::c_int;
        wx = sx;
        while wx <= ex {
            if !(redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) == 0) {
                bc = redraw_get_build_cell(bctx, x, y);
                memset(
                    bc as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    ::core::mem::size_of::<redraw_build_cell>() as size_t,
                );
                (*bc).data.type_0 = REDRAW_SPAN_SCROLLBAR;
                (*bc).data.c2rust_unnamed.sb.wp = wp;
                (*bc).data.c2rust_unnamed.sb.y = sy;
                (*bc).data.c2rust_unnamed.sb.height = (*wp).sy;
                if sb_left != 0 {
                    (*bc).data.c2rust_unnamed.sb.flags |= REDRAW_SCROLLBAR_LEFT;
                } else {
                    (*bc).data.c2rust_unnamed.sb.flags |= REDRAW_SCROLLBAR_RIGHT;
                }
                if overlay != 0 {
                    (*bc).data.c2rust_unnamed.sb.flags |= REDRAW_SCROLLBAR_OVERLAY;
                }
            }
            wx += 1;
        }
        sy = sy.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_data_has_pane(
    mut data: *mut redraw_span_data,
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    if (*data).c2rust_unnamed.b.top_wp == wp {
        return 1 as ::core::ffi::c_int;
    }
    if (*data).c2rust_unnamed.b.bottom_wp == wp {
        return 1 as ::core::ffi::c_int;
    }
    if (*data).c2rust_unnamed.b.left_wp == wp {
        return 1 as ::core::ffi::c_int;
    }
    if (*data).c2rust_unnamed.b.right_wp == wp {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn redraw_mark_border_cell(
    mut bctx: *mut redraw_build_ctx,
    mut wx: ::core::ffi::c_int,
    mut wy: ::core::ffi::c_int,
    mut wp: *mut window_pane,
    mut top_owner: ::core::ffi::c_int,
    mut bottom_owner: ::core::ffi::c_int,
    mut mask: ::core::ffi::c_int,
    mut pane_lines: pane_lines,
    mut floating: ::core::ffi::c_int,
) {
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut reset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) == 0 {
        return;
    }
    bc = redraw_get_build_cell(bctx, x, y);
    if floating == 0 {
        if (*bc).data.type_0 as ::core::ffi::c_uint
            == REDRAW_SPAN_EMPTY as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*bc).data.type_0 as ::core::ffi::c_uint
                == REDRAW_SPAN_OUTSIDE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            reset = 1 as ::core::ffi::c_int;
        } else if (*bc).data.type_0 as ::core::ffi::c_uint
            != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return;
        }
    } else if (*bc).data.type_0 as ::core::ffi::c_uint
        != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
        || redraw_data_has_pane(&raw mut (*bc).data, wp) == 0
    {
        reset = 1 as ::core::ffi::c_int;
    }
    if reset != 0 {
        memset(
            bc as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<redraw_build_cell>() as size_t,
        );
        (*bc).data.type_0 = REDRAW_SPAN_BORDER;
    }
    if top_owner != 0 {
        (*bc).data.c2rust_unnamed.b.top_wp = wp;
        (*bc).data.c2rust_unnamed.b.top_lines = pane_lines;
    }
    if bottom_owner != 0 {
        (*bc).data.c2rust_unnamed.b.bottom_wp = wp;
        (*bc).data.c2rust_unnamed.b.bottom_lines = pane_lines;
    }
    if mask & (REDRAW_BORDER_U | REDRAW_BORDER_D) != 0 {
        if wx < (*wp).xoff {
            (*bc).data.c2rust_unnamed.b.right_wp = wp;
            (*bc).data.c2rust_unnamed.b.right_lines = pane_lines;
        } else if wx >= (*wp).xoff + (*wp).sx as ::core::ffi::c_int {
            (*bc).data.c2rust_unnamed.b.left_wp = wp;
            (*bc).data.c2rust_unnamed.b.left_lines = pane_lines;
        }
    }
    mask |= (*bc).data.c2rust_unnamed.b.cell_mask;
    (*bc).data.c2rust_unnamed.b.cell_mask = mask;
    (*bc).data.c2rust_unnamed.b.cell_type = redraw_get_cell_type(mask);
}
unsafe extern "C" fn redraw_mark_border_status(
    mut bctx: *mut redraw_build_ctx,
    mut wp: *mut window_pane,
    mut left: ::core::ffi::c_int,
    mut right: ::core::ffi::c_int,
    mut top: ::core::ffi::c_int,
    mut bottom: ::core::ffi::c_int,
) {
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut off: u_int = 0 as u_int;
    let mut pane_status: ::core::ffi::c_int = 0;
    let mut wy: ::core::ffi::c_int = 0;
    let mut sx: ::core::ffi::c_int = 0;
    let mut ex: ::core::ffi::c_int = 0;
    let mut wx: ::core::ffi::c_int = 0;
    let mut cell_type: ::core::ffi::c_int = 0;
    pane_status = window_pane_get_pane_status(wp);
    if pane_status == PANE_STATUS_OFF {
        return;
    }
    if pane_status == PANE_STATUS_TOP {
        wy = top;
    } else {
        wy = bottom;
    }
    sx = (*wp).xoff + 2 as ::core::ffi::c_int;
    ex = right - 1 as ::core::ffi::c_int;
    if sx > ex {
        return;
    }
    wx = sx;
    while wx <= ex {
        if !(redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) == 0) {
            bc = redraw_get_build_cell(bctx, x, y);
            if !((*bc).data.type_0 as ::core::ffi::c_uint
                != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                cell_type = (*bc).data.c2rust_unnamed.b.cell_type;
                (*bc).data.type_0 = REDRAW_SPAN_STATUS;
                (*bc).data.c2rust_unnamed.st.wp = wp;
                (*bc).data.c2rust_unnamed.st.offset = off;
                (*bc).data.c2rust_unnamed.st.cell_type = cell_type;
            }
        }
        wx += 1;
        off = off.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_mark_border_arrows(
    mut bctx: *mut redraw_build_ctx,
    mut wp: *mut window_pane,
    mut left: ::core::ffi::c_int,
    mut right: ::core::ffi::c_int,
    mut top: ::core::ffi::c_int,
    mut bottom: ::core::ffi::c_int,
) {
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut wx: ::core::ffi::c_int = 0;
    let mut wy: ::core::ffi::c_int = 0;
    if (*bctx).ind != PANE_BORDER_ARROWS && (*bctx).ind != PANE_BORDER_BOTH {
        return;
    }
    wx = (*wp).xoff + 1 as ::core::ffi::c_int;
    if wx >= left && wx <= right {
        wy = top;
        if redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) != 0 {
            bc = redraw_get_build_cell(bctx, x, y);
            if (*bc).data.type_0 as ::core::ffi::c_uint
                == REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*bc).data.c2rust_unnamed.b.flags |= REDRAW_BORDER_IS_ARROW;
            }
        }
        wy = bottom;
        if redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) != 0 {
            bc = redraw_get_build_cell(bctx, x, y);
            if (*bc).data.type_0 as ::core::ffi::c_uint
                == REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*bc).data.c2rust_unnamed.b.flags |= REDRAW_BORDER_IS_ARROW;
            }
        }
    }
    wy = (*wp).yoff + 1 as ::core::ffi::c_int;
    if wy >= top && wy <= bottom {
        wx = left;
        if redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) != 0 {
            bc = redraw_get_build_cell(bctx, x, y);
            if (*bc).data.type_0 as ::core::ffi::c_uint
                == REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*bc).data.c2rust_unnamed.b.flags |= REDRAW_BORDER_IS_ARROW;
            }
        }
        wx = right;
        if redraw_window_to_scene(bctx, wx, wy, &raw mut x, &raw mut y) != 0 {
            bc = redraw_get_build_cell(bctx, x, y);
            if (*bc).data.type_0 as ::core::ffi::c_uint
                == REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*bc).data.c2rust_unnamed.b.flags |= REDRAW_BORDER_IS_ARROW;
            }
        }
    }
}
unsafe extern "C" fn redraw_mark_pane_borders(
    mut bctx: *mut redraw_build_ctx,
    mut wp: *mut window_pane,
    mut sb_w: ::core::ffi::c_int,
    mut sb_left: ::core::ffi::c_int,
) {
    let mut pane_lines: pane_lines = window_pane_get_pane_lines(wp);
    let mut pane_status: ::core::ffi::c_int = 0;
    let mut left: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut top: ::core::ffi::c_int = 0;
    let mut bottom: ::core::ffi::c_int = 0;
    let mut wx: ::core::ffi::c_int = 0;
    let mut wy: ::core::ffi::c_int = 0;
    let mut mark_top: ::core::ffi::c_int = 0;
    let mut mark_bottom: ::core::ffi::c_int = 0;
    let mut mark_left: ::core::ffi::c_int = 0;
    let mut mark_right: ::core::ffi::c_int = 0;
    let mut mask: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut floating: ::core::ffi::c_int = window_pane_is_floating(wp);
    if floating != 0
        && pane_lines as ::core::ffi::c_uint
            == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    pane_status = window_pane_get_pane_status(wp);
    left = (*wp).xoff - 1 as ::core::ffi::c_int;
    right = ((*wp).xoff as u_int).wrapping_add((*wp).sx) as ::core::ffi::c_int;
    if sb_w != 0 as ::core::ffi::c_int {
        if sb_left != 0 {
            left -= sb_w;
        } else {
            right += sb_w;
        }
    }
    top = (*wp).yoff - 1 as ::core::ffi::c_int;
    bottom = ((*wp).yoff as u_int).wrapping_add((*wp).sy) as ::core::ffi::c_int;
    mark_left = (left >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    mark_top = (top >= 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    if floating != 0 {
        mark_right = (right < (*(*bctx).w).sx as ::core::ffi::c_int) as ::core::ffi::c_int;
        mark_bottom = (bottom < (*(*bctx).w).sy as ::core::ffi::c_int) as ::core::ffi::c_int;
        if left < 0 as ::core::ffi::c_int {
            left = 0 as ::core::ffi::c_int;
        }
        if right >= (*(*bctx).w).sx as ::core::ffi::c_int {
            right = (*(*bctx).w).sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        }
        if top < 0 as ::core::ffi::c_int {
            top = 0 as ::core::ffi::c_int;
        }
        if bottom >= (*(*bctx).w).sy as ::core::ffi::c_int {
            bottom = (*(*bctx).w).sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        }
    } else {
        mark_right = (right <= (*(*bctx).w).sx as ::core::ffi::c_int) as ::core::ffi::c_int;
        mark_bottom = (bottom <= (*(*bctx).w).sy as ::core::ffi::c_int) as ::core::ffi::c_int;
        if pane_status == PANE_STATUS_TOP && bottom < (*(*bctx).w).sy as ::core::ffi::c_int {
            mark_bottom = 0 as ::core::ffi::c_int;
        } else if pane_status == PANE_STATUS_BOTTOM {
            mark_top = 0 as ::core::ffi::c_int;
        }
    }
    if mark_top != 0 {
        wx = left;
        while wx <= right {
            mask = 0 as ::core::ffi::c_int;
            if wx > left {
                mask |= REDRAW_BORDER_L;
            }
            if wx < right {
                mask |= REDRAW_BORDER_R;
            }
            redraw_mark_border_cell(
                bctx,
                wx,
                top,
                wp,
                0 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
                mask,
                pane_lines,
                floating,
            );
            wx += 1;
        }
    }
    if mark_bottom != 0 {
        wx = left;
        while wx <= right {
            mask = 0 as ::core::ffi::c_int;
            if wx > left {
                mask |= REDRAW_BORDER_L;
            }
            if wx < right {
                mask |= REDRAW_BORDER_R;
            }
            redraw_mark_border_cell(
                bctx,
                wx,
                bottom,
                wp,
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                mask,
                pane_lines,
                floating,
            );
            wx += 1;
        }
    }
    if mark_left != 0 {
        wy = top;
        while wy <= bottom {
            mask = 0 as ::core::ffi::c_int;
            if wy > top {
                mask |= REDRAW_BORDER_U;
            }
            if wy < bottom {
                mask |= REDRAW_BORDER_D;
            }
            redraw_mark_border_cell(
                bctx,
                left,
                wy,
                wp,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                mask,
                pane_lines,
                floating,
            );
            wy += 1;
        }
    }
    if mark_right != 0 {
        wy = top;
        while wy <= bottom {
            mask = 0 as ::core::ffi::c_int;
            if wy > top {
                mask |= REDRAW_BORDER_U;
            }
            if wy < bottom {
                mask |= REDRAW_BORDER_D;
            }
            redraw_mark_border_cell(
                bctx,
                right,
                wy,
                wp,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                mask,
                pane_lines,
                floating,
            );
            wy += 1;
        }
    }
    redraw_mark_border_status(bctx, wp, left, right, top, bottom);
    redraw_mark_border_arrows(bctx, wp, left, right, top, bottom);
}
unsafe extern "C" fn redraw_mark_pane(mut bctx: *mut redraw_build_ctx, mut wp: *mut window_pane) {
    let mut sb_w: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sb_left: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut overlay: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if window_pane_is_visible(wp) == 0 {
        return;
    }
    if window_pane_scrollbar_visible(wp) != 0 {
        overlay = window_pane_scrollbar_overlay(wp);
        if overlay != 0 {
            sb_w = (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad;
            if sb_w > (*wp).sx as ::core::ffi::c_int {
                sb_w = (*wp).scrollbar_style.width;
                if sb_w > (*wp).sx as ::core::ffi::c_int {
                    sb_w = (*wp).sx as ::core::ffi::c_int;
                }
            }
        } else {
            sb_w = (*wp).scrollbar_style.width + (*wp).scrollbar_style.pad;
        }
    }
    if sb_w != 0 as ::core::ffi::c_int && (*(*bctx).w).sb_pos == PANE_SCROLLBARS_LEFT {
        sb_left = 1 as ::core::ffi::c_int;
    }
    redraw_mark_pane_inside(bctx, wp);
    redraw_mark_pane_borders(
        bctx,
        wp,
        if overlay != 0 {
            0 as ::core::ffi::c_int
        } else {
            sb_w
        },
        sb_left,
    );
    redraw_mark_pane_scrollbar(bctx, wp, sb_w, sb_left, overlay);
}
unsafe extern "C" fn redraw_mark_two_pane_colours(mut bctx: *mut redraw_build_ctx) {
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut sd: *mut redraw_span_data = ::core::ptr::null_mut::<redraw_span_data>();
    let mut type_0: layout_type = LAYOUT_LEFTRIGHT;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut wx: u_int = 0;
    let mut wy: u_int = 0;
    if (*bctx).ind != PANE_BORDER_COLOUR && (*bctx).ind != PANE_BORDER_BOTH {
        return;
    }
    if redraw_check_two_pane_colours((*bctx).w, &raw mut type_0) == 0 {
        return;
    }
    y = 0 as u_int;
    while y < (*bctx).sy {
        x = 0 as u_int;
        while x < (*bctx).sx {
            bc = redraw_get_build_cell(bctx, x, y);
            if !((*bc).data.type_0 as ::core::ffi::c_uint
                != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                sd = &raw mut (*bc).data;
                wx = (*bctx).ox.wrapping_add(x);
                wy = (*bctx).oy.wrapping_add(y);
                if type_0 as ::core::ffi::c_uint
                    == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !(*sd).c2rust_unnamed.b.left_wp.is_null()
                    && !(*sd).c2rust_unnamed.b.right_wp.is_null()
                {
                    if wy <= (*(*bctx).w).sy.wrapping_div(2 as u_int) {
                        (*sd).c2rust_unnamed.b.style_wp = (*sd).c2rust_unnamed.b.left_wp;
                    } else {
                        (*sd).c2rust_unnamed.b.style_wp = (*sd).c2rust_unnamed.b.right_wp;
                    }
                } else if type_0 as ::core::ffi::c_uint
                    == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !(*sd).c2rust_unnamed.b.top_wp.is_null()
                    && !(*sd).c2rust_unnamed.b.bottom_wp.is_null()
                {
                    if wx <= (*(*bctx).w).sx.wrapping_div(2 as u_int) {
                        (*sd).c2rust_unnamed.b.style_wp = (*sd).c2rust_unnamed.b.top_wp;
                    } else {
                        (*sd).c2rust_unnamed.b.style_wp = (*sd).c2rust_unnamed.b.bottom_wp;
                    }
                }
            }
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_mark_menu(mut bctx: *mut redraw_build_ctx) {
    let mut md: *mut menu_data = (*(*bctx).w).menu;
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if md.is_null() {
        return;
    }
    sx = menu_width(md);
    sy = menu_height(md);
    py = 0 as u_int;
    while py < sy {
        px = 0 as u_int;
        while px < sx {
            if !(redraw_window_to_scene(
                bctx,
                menu_x(md).wrapping_add(px) as ::core::ffi::c_int,
                menu_y(md).wrapping_add(py) as ::core::ffi::c_int,
                &raw mut x,
                &raw mut y,
            ) == 0)
            {
                bc = redraw_get_build_cell(bctx, x, y);
                memset(
                    bc as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    ::core::mem::size_of::<redraw_build_cell>() as size_t,
                );
                (*bc).data.type_0 = REDRAW_SPAN_MENU;
                (*bc).data.c2rust_unnamed.m.md = md;
                (*bc).data.c2rust_unnamed.m.px = px;
                (*bc).data.c2rust_unnamed.m.py = py;
            }
            px = px.wrapping_add(1);
        }
        py = py.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_compare_data(
    mut a: *mut redraw_build_cell,
    mut b: *mut redraw_build_cell,
) -> ::core::ffi::c_int {
    let mut ad: *mut redraw_span_data = &raw mut (*a).data;
    let mut bd: *mut redraw_span_data = &raw mut (*b).data;
    if (*ad).type_0 as ::core::ffi::c_uint != (*bd).type_0 as ::core::ffi::c_uint {
        return 0 as ::core::ffi::c_int;
    }
    match (*ad).type_0 as ::core::ffi::c_uint {
        0 => {
            if (*ad).c2rust_unnamed.p.wp != (*bd).c2rust_unnamed.p.wp
                || (*ad).c2rust_unnamed.p.py != (*bd).c2rust_unnamed.p.py
                || (*ad).c2rust_unnamed.p.px.wrapping_add(1 as u_int) != (*bd).c2rust_unnamed.p.px
            {
                return 0 as ::core::ffi::c_int;
            }
            return 1 as ::core::ffi::c_int;
        }
        4 => {
            if (*ad).c2rust_unnamed.b.top_wp != (*bd).c2rust_unnamed.b.top_wp
                || (*ad).c2rust_unnamed.b.bottom_wp != (*bd).c2rust_unnamed.b.bottom_wp
                || (*ad).c2rust_unnamed.b.left_wp != (*bd).c2rust_unnamed.b.left_wp
                || (*ad).c2rust_unnamed.b.right_wp != (*bd).c2rust_unnamed.b.right_wp
                || (*ad).c2rust_unnamed.b.style_wp != (*bd).c2rust_unnamed.b.style_wp
                || (*ad).c2rust_unnamed.b.top_lines as ::core::ffi::c_uint
                    != (*bd).c2rust_unnamed.b.top_lines as ::core::ffi::c_uint
                || (*ad).c2rust_unnamed.b.bottom_lines as ::core::ffi::c_uint
                    != (*bd).c2rust_unnamed.b.bottom_lines as ::core::ffi::c_uint
                || (*ad).c2rust_unnamed.b.left_lines as ::core::ffi::c_uint
                    != (*bd).c2rust_unnamed.b.left_lines as ::core::ffi::c_uint
                || (*ad).c2rust_unnamed.b.right_lines as ::core::ffi::c_uint
                    != (*bd).c2rust_unnamed.b.right_lines as ::core::ffi::c_uint
                || (*ad).c2rust_unnamed.b.cell_type != (*bd).c2rust_unnamed.b.cell_type
                || (*ad).c2rust_unnamed.b.cell_mask != (*bd).c2rust_unnamed.b.cell_mask
                || (*ad).c2rust_unnamed.b.flags != (*bd).c2rust_unnamed.b.flags
            {
                return 0 as ::core::ffi::c_int;
            }
            if (*ad).c2rust_unnamed.b.flags & REDRAW_BORDER_IS_ARROW != 0 {
                return 0 as ::core::ffi::c_int;
            }
            return 1 as ::core::ffi::c_int;
        }
        3 => {
            if (*ad).c2rust_unnamed.st.wp != (*bd).c2rust_unnamed.st.wp
                || (*ad).c2rust_unnamed.st.offset.wrapping_add(1 as u_int)
                    != (*bd).c2rust_unnamed.st.offset
                || (*ad).c2rust_unnamed.st.cell_type != (*bd).c2rust_unnamed.st.cell_type
            {
                return 0 as ::core::ffi::c_int;
            }
            return 1 as ::core::ffi::c_int;
        }
        5 => {
            if (*ad).c2rust_unnamed.sb.wp != (*bd).c2rust_unnamed.sb.wp
                || (*ad).c2rust_unnamed.sb.y != (*bd).c2rust_unnamed.sb.y
                || (*ad).c2rust_unnamed.sb.height != (*bd).c2rust_unnamed.sb.height
                || (*ad).c2rust_unnamed.sb.flags != (*bd).c2rust_unnamed.sb.flags
            {
                return 0 as ::core::ffi::c_int;
            }
            return 1 as ::core::ffi::c_int;
        }
        6 => {
            if (*ad).c2rust_unnamed.m.md != (*bd).c2rust_unnamed.m.md
                || (*ad).c2rust_unnamed.m.py != (*bd).c2rust_unnamed.m.py
                || (*ad).c2rust_unnamed.m.px.wrapping_add(1 as u_int) != (*bd).c2rust_unnamed.m.px
            {
                return 0 as ::core::ffi::c_int;
            }
            return 1 as ::core::ffi::c_int;
        }
        1 | 2 => return 1 as ::core::ffi::c_int,
        _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn redraw_build_cells(mut bctx: *mut redraw_build_ctx) {
    let mut w: *mut window = (*bctx).w;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut ncells: size_t = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*bctx).sx != 0 as u_int
        && (*bctx).sy as ::core::ffi::c_ulong
            > SIZE_MAX.wrapping_div((*bctx).sx as ::core::ffi::c_ulong)
    {
        fatalx(
            b"%s: too many cells\0" as *const u8 as *const ::core::ffi::c_char,
            b"redraw_build_cells\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    ncells = ((*bctx).sx as size_t).wrapping_mul((*bctx).sy as size_t);
    if ncells > redraw_ncells {
        redraw_cells = xreallocarray(
            redraw_cells as *mut ::core::ffi::c_void,
            ncells,
            ::core::mem::size_of::<redraw_build_cell>() as size_t,
        ) as *mut redraw_build_cell;
        redraw_ncells = ncells;
    }
    (*bctx).cells = redraw_cells;
    y = 0 as u_int;
    while y < (*bctx).sy {
        x = 0 as u_int;
        while x < (*bctx).sx {
            redraw_reset_cell(bctx, x, y);
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    wp = *(*((*w).z_index.tqh_last as *mut window_panes_zindex)).tqh_last;
    while !wp.is_null() {
        redraw_mark_pane(bctx, wp);
        wp = *(*((*wp).zentry.tqe_prev as *mut window_panes_zindex)).tqh_last;
    }
    redraw_mark_two_pane_colours(bctx);
    redraw_mark_menu(bctx);
}
unsafe extern "C" fn redraw_make_scene(mut c: *mut client) -> *mut redraw_scene {
    let mut s: *mut session = (*c).session;
    let mut w: *mut window = (*(*s).curw).window;
    let mut bctx: redraw_build_ctx = redraw_build_ctx {
        c: ::core::ptr::null_mut::<client>(),
        w: ::core::ptr::null_mut::<window>(),
        ox: 0,
        oy: 0,
        sx: 0,
        sy: 0,
        ind: 0,
        cells: ::core::ptr::null_mut::<redraw_build_cell>(),
    };
    let mut scene: *mut redraw_scene = ::core::ptr::null_mut::<redraw_scene>();
    let mut bc: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut last: *mut redraw_build_cell = ::core::ptr::null_mut::<redraw_build_cell>();
    let mut line: *mut redraw_line = ::core::ptr::null_mut::<redraw_line>();
    let mut span: *mut redraw_span = ::core::ptr::null_mut::<redraw_span>();
    let mut type_0: redraw_span_type = REDRAW_SPAN_PANE;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut x0: u_int = 0;
    if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
        return ::core::ptr::null_mut::<redraw_scene>();
    }
    redraw_set_context(c, &raw mut bctx);
    log_debug(
        b"%s: building @%u scene (%ux%u %u,%u; generation %llu)\0" as *const u8
            as *const ::core::ffi::c_char,
        (*c).name,
        (*w).id,
        bctx.sx,
        bctx.sy,
        bctx.ox,
        bctx.oy,
        (*w).redraw_scene_generation as ::core::ffi::c_ulonglong,
    );
    redraw_build_cells(&raw mut bctx);
    scene = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<redraw_scene>() as size_t,
    ) as *mut redraw_scene;
    (*scene).c = c;
    (*scene).w = w;
    (*scene).lines = xcalloc(
        bctx.sy as size_t,
        ::core::mem::size_of::<redraw_line>() as size_t,
    ) as *mut redraw_line;
    (*scene).generation = (*w).redraw_scene_generation;
    (*scene).sx = bctx.sx;
    (*scene).sy = bctx.sy;
    (*scene).ox = bctx.ox;
    (*scene).oy = bctx.oy;
    y = 0 as u_int;
    while y < bctx.sy {
        line = (*scene).lines.offset(y as isize) as *mut redraw_line;
        type_0 = REDRAW_SPAN_PANE;
        while (type_0 as ::core::ffi::c_uint) < REDRAW_SPAN_TYPES as ::core::ffi::c_uint {
            (*line).spans[type_0 as usize].tqh_first = ::core::ptr::null_mut::<redraw_span>();
            (*line).spans[type_0 as usize].tqh_last =
                &raw mut (*(&raw mut (*line).spans as *mut redraw_spans).offset(type_0 as isize))
                    .tqh_first;
            type_0 += 1;
        }
        x = 0 as u_int;
        while x < bctx.sx {
            x0 = x;
            last = redraw_get_build_cell(&raw mut bctx, x, y);
            x = x.wrapping_add(1);
            while x < bctx.sx {
                bc = redraw_get_build_cell(&raw mut bctx, x, y);
                if redraw_compare_data(last, bc) == 0 {
                    break;
                }
                last = bc;
                x = x.wrapping_add(1);
            }
            bc = redraw_get_build_cell(&raw mut bctx, x0, y);
            type_0 = (*bc).data.type_0;
            span = xcalloc(1 as size_t, ::core::mem::size_of::<redraw_span>() as size_t)
                as *mut redraw_span;
            (*span).x = x0;
            (*span).width = x.wrapping_sub(x0);
            (*span).data = (*bc).data;
            (*span).entry.tqe_next = ::core::ptr::null_mut::<redraw_span>();
            (*span).entry.tqe_prev = (*line).spans[type_0 as usize].tqh_last;
            *(*line).spans[type_0 as usize].tqh_last = span;
            (*line).spans[type_0 as usize].tqh_last = &raw mut (*span).entry.tqe_next;
        }
        y = y.wrapping_add(1);
    }
    log_debug(
        b"%s: finished building @%u scene\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        (*w).id,
    );
    return scene;
}
#[no_mangle]
pub unsafe extern "C" fn redraw_free_scene(mut scene: *mut redraw_scene) {
    let mut spans: *mut redraw_spans = ::core::ptr::null_mut::<redraw_spans>();
    let mut span: *mut redraw_span = ::core::ptr::null_mut::<redraw_span>();
    let mut span1: *mut redraw_span = ::core::ptr::null_mut::<redraw_span>();
    let mut y: u_int = 0;
    let mut type_0: u_int = 0;
    if scene.is_null() {
        return;
    }
    y = 0 as u_int;
    while y < (*scene).sy {
        type_0 = 0 as u_int;
        while type_0 < REDRAW_SPAN_TYPES as u_int {
            spans = (&raw mut (*(*scene).lines.offset(y as isize)).spans as *mut redraw_spans)
                .offset(type_0 as isize) as *mut redraw_spans;
            span = (*spans).tqh_first;
            while !span.is_null() && {
                span1 = (*span).entry.tqe_next;
                1 as ::core::ffi::c_int != 0
            } {
                if !(*span).entry.tqe_next.is_null() {
                    (*(*span).entry.tqe_next).entry.tqe_prev = (*span).entry.tqe_prev;
                } else {
                    (*spans).tqh_last = (*span).entry.tqe_prev;
                }
                *(*span).entry.tqe_prev = (*span).entry.tqe_next;
                free(span as *mut ::core::ffi::c_void);
                span = span1;
            }
            type_0 = type_0.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    free((*scene).lines as *mut ::core::ffi::c_void);
    free(scene as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn redraw_invalidate_scene(mut w: *mut window) {
    (*w).redraw_scene_generation = (*w).redraw_scene_generation.wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn redraw_invalidate_all_scenes() {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
    while !w.is_null() {
        redraw_invalidate_scene(w);
        w = windows_RB_NEXT(w);
    }
}
unsafe extern "C" fn redraw_get_scene(mut c: *mut client) -> *mut redraw_scene {
    let mut scene: *mut redraw_scene = (*c).redraw_scene;
    let mut w: *mut window = (*(*(*c).session).curw).window;
    let mut reason: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    redraw_get_window_offset(c, &raw mut ox, &raw mut oy, &raw mut sx, &raw mut sy);
    if scene.is_null() {
        reason = b"missing\0" as *const u8 as *const ::core::ffi::c_char;
    } else if (*scene).w != w {
        reason = b"window changed\0" as *const u8 as *const ::core::ffi::c_char;
    } else if (*scene).generation != (*w).redraw_scene_generation {
        reason = b"generation changed\0" as *const u8 as *const ::core::ffi::c_char;
    } else if (*scene).ox != ox || (*scene).oy != oy {
        reason = b"offset changed\0" as *const u8 as *const ::core::ffi::c_char;
    } else if (*scene).sx != sx || (*scene).sy != sy {
        reason = b"size changed\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if !reason.is_null() {
        log_debug(
            b"%s: @%u scene invalid: %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            (*w).id,
            reason,
        );
        redraw_free_scene(scene);
        scene = redraw_make_scene(c);
        (*c).redraw_scene = scene;
    }
    return scene;
}
unsafe extern "C" fn redraw_draw_pane_span(
    mut dctx: *mut redraw_draw_ctx,
    mut span: *mut redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut c: *mut client = (*scene).c;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut wp: *mut window_pane = (*span).data.c2rust_unnamed.p.wp;
    let mut s: *mut screen = (*wp).screen;
    let mut defaults: grid_cell = grid_cell {
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
    let mut style_ctx: tty_style_ctx = tty_style_ctx {
        defaults: ::core::ptr::null::<grid_cell>(),
        palette: ::core::ptr::null_mut::<colour_palette>(),
        dim: 0,
        hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
    };
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    tty_default_colours(&raw mut defaults, wp, &raw mut style_ctx.dim);
    style_ctx.defaults = &raw mut defaults;
    style_ctx.palette = &raw mut (*wp).palette;
    style_ctx.hyperlinks = (*s).hyperlinks;
    px = (*span)
        .data
        .c2rust_unnamed
        .p
        .px
        .wrapping_add(x.wrapping_sub((*span).x));
    py = (*span).data.c2rust_unnamed.p.py;
    tty_draw_line(tty, s, px, py, n, x, y, &raw mut style_ctx);
}
unsafe extern "C" fn redraw_get_default_border_style(
    mut dctx: *mut redraw_draw_ctx,
    mut gc: *mut grid_cell,
    mut pane_lines: *mut pane_lines,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut c: *mut client = (*scene).c;
    let mut s: *mut session = (*c).session;
    let mut oo: *mut options = (*(*scene).w).options;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut dgc: *mut grid_cell = &raw mut (*dctx).default_gc;
    if !(*dctx).flags & REDRAW_DEFAULT_SET != 0 {
        ft = format_create_defaults(
            ::core::ptr::null_mut::<cmdq_item>(),
            c,
            s,
            (*s).curw,
            ::core::ptr::null_mut::<window_pane>(),
        );
        memcpy(
            dgc as *mut ::core::ffi::c_void,
            &raw const grid_default_cell as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
        style_add(
            dgc,
            oo,
            b"pane-border-style\0" as *const u8 as *const ::core::ffi::c_char,
            ft,
        );
        format_free(ft);
        (*dctx).pane_lines = options_get_number(
            oo,
            b"pane-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
        ) as pane_lines;
        (*dctx).flags |= REDRAW_DEFAULT_SET;
    }
    memcpy(
        gc as *mut ::core::ffi::c_void,
        dgc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    *pane_lines = (*dctx).pane_lines;
}
unsafe extern "C" fn redraw_get_pane_for_border_style(
    mut dctx: *mut redraw_draw_ctx,
    mut span: *mut redraw_span,
) -> *mut window_pane {
    let mut active: *mut window_pane = (*dctx).active;
    if (*span).data.type_0 as ::core::ffi::c_uint
        != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<window_pane>();
    }
    if !(*span).data.c2rust_unnamed.b.style_wp.is_null() {
        return (*span).data.c2rust_unnamed.b.style_wp;
    }
    if !active.is_null() && redraw_data_has_pane(&raw mut (*span).data, active) != 0 {
        return active;
    }
    if !(*span).data.c2rust_unnamed.b.top_wp.is_null() {
        return (*span).data.c2rust_unnamed.b.top_wp;
    }
    if !(*span).data.c2rust_unnamed.b.bottom_wp.is_null() {
        return (*span).data.c2rust_unnamed.b.bottom_wp;
    }
    if !(*span).data.c2rust_unnamed.b.left_wp.is_null() {
        return (*span).data.c2rust_unnamed.b.left_wp;
    }
    if !(*span).data.c2rust_unnamed.b.right_wp.is_null() {
        return (*span).data.c2rust_unnamed.b.right_wp;
    }
    return ::core::ptr::null_mut::<window_pane>();
}
unsafe extern "C" fn redraw_draw_border_arrow(
    mut dctx: *mut redraw_draw_ctx,
    mut span: *mut redraw_span,
    mut gc: *mut grid_cell,
) {
    let mut active: *mut window_pane = (*dctx).active;
    let mut ch: ::core::ffi::c_char = 0;
    if (*span).data.type_0 as ::core::ffi::c_uint
        != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
        || active.is_null()
    {
        return;
    }
    if !(*span).data.c2rust_unnamed.b.flags & REDRAW_BORDER_IS_ARROW != 0 {
        return;
    }
    if (*span).data.c2rust_unnamed.b.left_wp == active {
        ch = ',' as i32 as ::core::ffi::c_char;
    } else if (*span).data.c2rust_unnamed.b.right_wp == active {
        ch = '+' as i32 as ::core::ffi::c_char;
    } else if (*span).data.c2rust_unnamed.b.top_wp == active {
        ch = '-' as i32 as ::core::ffi::c_char;
    } else if (*span).data.c2rust_unnamed.b.bottom_wp == active {
        ch = '.' as i32 as ::core::ffi::c_char;
    } else {
        return;
    }
    utf8_set(&raw mut (*gc).data, ch as u_char);
    (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
}
unsafe extern "C" fn redraw_draw_border_span(
    mut dctx: *mut redraw_draw_ctx,
    mut span: *mut redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut c: *mut client = (*scene).c;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut w: *mut window = (*scene).w;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
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
    let mut pane_lines: pane_lines = PANE_LINES_SINGLE;
    let mut i: u_int = 0;
    let mut cell_type: u_int = 0;
    let mut isolates: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*span).data.type_0 as ::core::ffi::c_uint
        != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        cell_type = CELL_NONE as u_int;
    } else {
        wp = redraw_get_pane_for_border_style(dctx, span);
        cell_type = (*span).data.c2rust_unnamed.b.cell_type as u_int;
    }
    if wp.is_null() {
        redraw_get_default_border_style(dctx, &raw mut gc, &raw mut pane_lines);
        if (*span).data.type_0 as ::core::ffi::c_uint
            == REDRAW_SPAN_OUTSIDE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_get_fill_cell(w, 0 as ::core::ffi::c_int, &raw mut gc);
        } else if (*span).data.type_0 as ::core::ffi::c_uint
            == REDRAW_SPAN_EMPTY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            window_get_fill_cell(w, 1 as ::core::ffi::c_int, &raw mut gc);
        } else {
            if (*span).data.type_0 as ::core::ffi::c_uint
                != REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                pane_lines = PANE_LINES_SINGLE;
            }
            window_get_border_cell(
                ::core::ptr::null_mut::<window_pane>(),
                pane_lines,
                cell_type as ::core::ffi::c_int,
                &raw mut gc,
            );
        }
    } else {
        window_pane_get_border_style(wp, c, &raw mut gc);
        window_pane_get_border_cell(wp, cell_type as ::core::ffi::c_int, &raw mut gc);
    }
    if (*span).data.type_0 as ::core::ffi::c_uint
        == REDRAW_SPAN_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*dctx).marked.is_null()
        && redraw_data_has_pane(&raw mut (*span).data, (*dctx).marked) != 0
    {
        gc.attr = (gc.attr as ::core::ffi::c_int ^ GRID_ATTR_REVERSE) as u_short;
    }
    redraw_draw_border_arrow(dctx, span, &raw mut gc);
    if cell_type == CELL_UD as u_int && (*dctx).flags & REDRAW_ISOLATES != 0 {
        isolates = 1 as ::core::ffi::c_int;
    }
    tty_cursor(tty, x, y);
    if isolates != 0 {
        tty_puts(tty, REDRAW_END_ISOLATE.as_ptr());
    }
    i = 0 as u_int;
    while i < n {
        tty_cell(tty, &raw mut gc, ::core::ptr::null::<tty_style_ctx>());
        i = i.wrapping_add(1);
    }
    if isolates != 0 {
        tty_puts(tty, REDRAW_START_ISOLATE.as_ptr());
    }
}
unsafe extern "C" fn redraw_draw_status_span(
    mut dctx: *mut redraw_draw_ctx,
    mut span: *mut redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut c: *mut client = (*scene).c;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut wp: *mut window_pane = (*span).data.c2rust_unnamed.st.wp;
    let mut s: *mut screen = &raw mut (*wp).status_screen;
    let mut px: u_int = 0;
    let mut sx: u_int = (*(*s).grid).sx;
    px = (*span)
        .data
        .c2rust_unnamed
        .st
        .offset
        .wrapping_add(x.wrapping_sub((*span).x));
    if px < sx {
        if n > sx.wrapping_sub(px) {
            n = sx.wrapping_sub(px);
        }
        tty_draw_line(
            tty,
            s,
            px,
            0 as u_int,
            n,
            x,
            y,
            ::core::ptr::null::<tty_style_ctx>(),
        );
    }
}
unsafe extern "C" fn redraw_draw_scrollbar_span(
    mut dctx: *mut redraw_draw_ctx,
    mut span: *mut redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut wp: *mut window_pane = (*span).data.c2rust_unnamed.sb.wp;
    let mut s: *mut screen = (*wp).screen;
    let mut tty: *mut tty = &raw mut (*(*scene).c).tty;
    let mut sb_style: *mut style = &raw mut (*wp).scrollbar_style;
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
    let mut slgc: grid_cell = grid_cell {
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
    let mut pad_gc: grid_cell = grid_cell {
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
    let mut gcp: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
    let mut pct_view: ::core::ffi::c_double = 0.;
    let mut total_height: u_int = 0;
    let mut slider_h: u_int = 0;
    let mut slider_y: u_int = 0;
    let mut sb_h: u_int = (*span).data.c2rust_unnamed.sb.height;
    let mut sb_y: u_int = (*span).data.c2rust_unnamed.sb.y;
    let mut i: u_int = 0;
    let mut off: u_int = 0;
    let mut sb_w: u_int = 0;
    let mut sb_pad: u_int = 0;
    let mut cm_y: ::core::ffi::c_int = 0;
    let mut cm_size: ::core::ffi::c_int = 0;
    if window_pane_mode(wp) == WINDOW_PANE_NO_MODE {
        total_height = (*(*s).grid).sy.wrapping_add((*(*s).grid).hsize);
        if total_height == 0 as u_int {
            return;
        }
        pct_view = sb_h as ::core::ffi::c_double / total_height as ::core::ffi::c_double;
        slider_h = (sb_h as ::core::ffi::c_double * pct_view) as u_int;
        slider_y = sb_h.wrapping_sub(slider_h);
    } else {
        if (*wp).modes.tqh_first.is_null() {
            return;
        }
        if window_copy_get_current_offset(
            wp,
            &raw mut cm_y as *mut u_int,
            &raw mut cm_size as *mut u_int,
        ) == 0 as ::core::ffi::c_int
        {
            return;
        }
        total_height = (cm_size as u_int).wrapping_add(sb_h);
        if total_height == 0 as u_int {
            return;
        }
        pct_view = sb_h as ::core::ffi::c_double / total_height as ::core::ffi::c_double;
        slider_h = (sb_h as ::core::ffi::c_double * pct_view) as u_int;
        slider_y = (sb_h.wrapping_add(1 as u_int) as ::core::ffi::c_double
            * (cm_y as ::core::ffi::c_double / total_height as ::core::ffi::c_double))
            as u_int;
    }
    if slider_h < 1 as u_int {
        slider_h = 1 as u_int;
    }
    if slider_y >= sb_h {
        slider_y = sb_h.wrapping_sub(1 as u_int);
    }
    (*wp).sb_slider_y = slider_y;
    (*wp).sb_slider_h = slider_h;
    gc = (*sb_style).gc;
    memcpy(
        &raw mut slgc as *mut ::core::ffi::c_void,
        &raw mut gc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    slgc.fg = gc.bg;
    slgc.bg = gc.fg;
    tty_default_colours(&raw mut pad_gc, wp, ::core::ptr::null_mut::<u_int>());
    sb_w = (*sb_style).width as u_int;
    sb_pad = (*sb_style).pad as u_int;
    off = x.wrapping_sub((*span).x);
    tty_cursor(tty, x, y);
    let mut current_block_40: u64;
    i = 0 as u_int;
    while i < n {
        if (*span).data.c2rust_unnamed.sb.flags & REDRAW_SCROLLBAR_LEFT != 0 {
            if off.wrapping_add(i) >= sb_w && off.wrapping_add(i) < sb_w.wrapping_add(sb_pad) {
                tty_cell(tty, &raw mut pad_gc, ::core::ptr::null::<tty_style_ctx>());
                current_block_40 = 3437258052017859086;
            } else {
                current_block_40 = 7828949454673616476;
            }
        } else if off.wrapping_add(i) < sb_pad {
            tty_cell(tty, &raw mut pad_gc, ::core::ptr::null::<tty_style_ctx>());
            current_block_40 = 3437258052017859086;
        } else {
            current_block_40 = 7828949454673616476;
        }
        match current_block_40 {
            7828949454673616476 => {
                if sb_y >= slider_y && sb_y < slider_y.wrapping_add(slider_h) {
                    gcp = &raw mut slgc;
                } else {
                    gcp = &raw mut gc;
                }
                tty_cell(tty, gcp, ::core::ptr::null::<tty_style_ctx>());
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_draw_menu_span(
    mut dctx: *mut redraw_draw_ctx,
    mut span: *mut redraw_span,
    mut x: u_int,
    mut y: u_int,
    mut n: u_int,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut tty: *mut tty = &raw mut (*(*scene).c).tty;
    let mut s: *mut screen = menu_screen((*span).data.c2rust_unnamed.m.md);
    let mut px: u_int = 0;
    px = (*span)
        .data
        .c2rust_unnamed
        .m
        .px
        .wrapping_add(x.wrapping_sub((*span).x));
    tty_draw_line(
        tty,
        s,
        px,
        (*span).data.c2rust_unnamed.m.py,
        n,
        x,
        y,
        ::core::ptr::null::<tty_style_ctx>(),
    );
}
unsafe extern "C" fn redraw_draw_span(
    mut dctx: *mut redraw_draw_ctx,
    mut span: *mut redraw_span,
    mut y: u_int,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut data: *mut redraw_span_data = &raw mut (*span).data;
    let mut type_0: redraw_span_type = (*data).type_0;
    let mut c: *mut client = (*scene).c;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut rr: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    let mut i: u_int = 0;
    let mut x: u_int = 0;
    let mut n: u_int = 0;
    if type_0 as ::core::ffi::c_uint
        == REDRAW_SPAN_STATUS as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*(*data).c2rust_unnamed.st.wp).flags & PANE_NEWSTATUS != 0
    {
        return;
    }
    r = tty_check_overlay_range(tty, (*span).x, y, (*span).width);
    i = 0 as u_int;
    while i < (*r).used {
        rr = (*r).ranges.offset(i as isize) as *mut visible_range;
        if !((*rr).nx == 0 as u_int) {
            x = (*rr).px;
            n = (*rr).nx;
            match (*span).data.type_0 as ::core::ffi::c_uint {
                0 => {
                    redraw_draw_pane_span(dctx, span, x, y, n);
                }
                4 | 2 | 1 => {
                    redraw_draw_border_span(dctx, span, x, y, n);
                }
                3 => {
                    redraw_draw_status_span(dctx, span, x, y, n);
                }
                5 => {
                    redraw_draw_scrollbar_span(dctx, span, x, y, n);
                }
                6 => {
                    redraw_draw_menu_span(dctx, span, x, y, n);
                }
                _ => {}
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_draw_pane_lines(
    mut dctx: *mut redraw_draw_ctx,
    mut wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut line: *mut redraw_line = ::core::ptr::null_mut::<redraw_line>();
    let mut spans: *mut redraw_spans = ::core::ptr::null_mut::<redraw_spans>();
    let mut span: *mut redraw_span = ::core::ptr::null_mut::<redraw_span>();
    let mut cy: u_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut top: ::core::ffi::c_int = 0;
    let mut bottom: ::core::ffi::c_int = 0;
    top = (*wp).yoff - (*scene).oy as ::core::ffi::c_int;
    if top < 0 as ::core::ffi::c_int {
        top = 0 as ::core::ffi::c_int;
    }
    bottom = (*wp).yoff + (*wp).sy as ::core::ffi::c_int - (*scene).oy as ::core::ffi::c_int;
    if bottom < 0 as ::core::ffi::c_int {
        bottom = 0 as ::core::ffi::c_int;
    }
    if bottom > (*scene).sy as ::core::ffi::c_int {
        bottom = (*scene).sy as ::core::ffi::c_int;
    }
    y = top;
    while y < bottom {
        line = (*scene).lines.offset(y as isize) as *mut redraw_line;
        if (*dctx).flags & REDRAW_STATUS_TOP != 0 {
            cy = (*dctx).status_lines.wrapping_add(y as u_int);
        } else {
            cy = y as u_int;
        }
        if flags & REDRAW_PANE != 0 {
            spans = (&raw mut (*line).spans as *mut redraw_spans)
                .offset(REDRAW_SPAN_PANE as ::core::ffi::c_int as isize)
                as *mut redraw_spans;
            span = (*spans).tqh_first;
            while !span.is_null() {
                if (*span).data.c2rust_unnamed.p.wp == wp {
                    redraw_draw_span(dctx, span, cy);
                }
                span = (*span).entry.tqe_next;
            }
        }
        if flags & REDRAW_PANE_SCROLLBAR != 0 {
            spans = (&raw mut (*line).spans as *mut redraw_spans)
                .offset(REDRAW_SPAN_SCROLLBAR as ::core::ffi::c_int as isize)
                as *mut redraw_spans;
            span = (*spans).tqh_first;
            while !span.is_null() {
                if (*span).data.c2rust_unnamed.sb.wp == wp {
                    redraw_draw_span(dctx, span, cy);
                }
                span = (*span).entry.tqe_next;
            }
        }
        y += 1;
    }
}
unsafe extern "C" fn redraw_draw_lines(
    mut dctx: *mut redraw_draw_ctx,
    mut flags: ::core::ffi::c_int,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut line: *mut redraw_line = ::core::ptr::null_mut::<redraw_line>();
    let mut spans: *mut redraw_spans = ::core::ptr::null_mut::<redraw_spans>();
    let mut span: *mut redraw_span = ::core::ptr::null_mut::<redraw_span>();
    let mut y: u_int = 0;
    let mut cy: u_int = 0;
    let mut type_0: u_int = 0;
    y = 0 as u_int;
    while y < (*scene).sy {
        line = (*scene).lines.offset(y as isize) as *mut redraw_line;
        if (*dctx).flags & REDRAW_STATUS_TOP != 0 {
            cy = (*dctx).status_lines.wrapping_add(y);
        } else {
            cy = y;
        }
        let mut current_block_9: u64;
        type_0 = 0 as u_int;
        while type_0 < REDRAW_SPAN_TYPES as u_int {
            if !(flags == REDRAW_ALL) {
                match type_0 {
                    0 => {
                        current_block_9 = 12159231939852168738;
                        match current_block_9 {
                            10170200351409165677 => {
                                if !flags & REDRAW_MENU != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            16319640041109108851 => {
                                if !flags & REDRAW_OUTSIDE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            11740996042087507061 => {
                                if !flags & REDRAW_EMPTY != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            10899678633117585587 => {
                                if !flags & REDRAW_PANE_BORDER != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            7695618138115701323 => {
                                if !flags & REDRAW_PANE_STATUS != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            6251484932569512093 => {
                                if !flags & REDRAW_PANE_SCROLLBAR != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            _ => {
                                if !flags & REDRAW_PANE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                        }
                    }
                    1 => {
                        current_block_9 = 16319640041109108851;
                        match current_block_9 {
                            10170200351409165677 => {
                                if !flags & REDRAW_MENU != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            16319640041109108851 => {
                                if !flags & REDRAW_OUTSIDE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            11740996042087507061 => {
                                if !flags & REDRAW_EMPTY != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            10899678633117585587 => {
                                if !flags & REDRAW_PANE_BORDER != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            7695618138115701323 => {
                                if !flags & REDRAW_PANE_STATUS != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            6251484932569512093 => {
                                if !flags & REDRAW_PANE_SCROLLBAR != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            _ => {
                                if !flags & REDRAW_PANE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                        }
                    }
                    2 => {
                        current_block_9 = 11740996042087507061;
                        match current_block_9 {
                            10170200351409165677 => {
                                if !flags & REDRAW_MENU != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            16319640041109108851 => {
                                if !flags & REDRAW_OUTSIDE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            11740996042087507061 => {
                                if !flags & REDRAW_EMPTY != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            10899678633117585587 => {
                                if !flags & REDRAW_PANE_BORDER != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            7695618138115701323 => {
                                if !flags & REDRAW_PANE_STATUS != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            6251484932569512093 => {
                                if !flags & REDRAW_PANE_SCROLLBAR != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            _ => {
                                if !flags & REDRAW_PANE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                        }
                    }
                    4 => {
                        current_block_9 = 10899678633117585587;
                        match current_block_9 {
                            10170200351409165677 => {
                                if !flags & REDRAW_MENU != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            16319640041109108851 => {
                                if !flags & REDRAW_OUTSIDE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            11740996042087507061 => {
                                if !flags & REDRAW_EMPTY != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            10899678633117585587 => {
                                if !flags & REDRAW_PANE_BORDER != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            7695618138115701323 => {
                                if !flags & REDRAW_PANE_STATUS != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            6251484932569512093 => {
                                if !flags & REDRAW_PANE_SCROLLBAR != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            _ => {
                                if !flags & REDRAW_PANE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                        }
                    }
                    3 => {
                        current_block_9 = 7695618138115701323;
                        match current_block_9 {
                            10170200351409165677 => {
                                if !flags & REDRAW_MENU != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            16319640041109108851 => {
                                if !flags & REDRAW_OUTSIDE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            11740996042087507061 => {
                                if !flags & REDRAW_EMPTY != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            10899678633117585587 => {
                                if !flags & REDRAW_PANE_BORDER != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            7695618138115701323 => {
                                if !flags & REDRAW_PANE_STATUS != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            6251484932569512093 => {
                                if !flags & REDRAW_PANE_SCROLLBAR != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            _ => {
                                if !flags & REDRAW_PANE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                        }
                    }
                    5 => {
                        current_block_9 = 6251484932569512093;
                        match current_block_9 {
                            10170200351409165677 => {
                                if !flags & REDRAW_MENU != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            16319640041109108851 => {
                                if !flags & REDRAW_OUTSIDE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            11740996042087507061 => {
                                if !flags & REDRAW_EMPTY != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            10899678633117585587 => {
                                if !flags & REDRAW_PANE_BORDER != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            7695618138115701323 => {
                                if !flags & REDRAW_PANE_STATUS != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            6251484932569512093 => {
                                if !flags & REDRAW_PANE_SCROLLBAR != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            _ => {
                                if !flags & REDRAW_PANE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                        }
                    }
                    6 => {
                        current_block_9 = 10170200351409165677;
                        match current_block_9 {
                            10170200351409165677 => {
                                if !flags & REDRAW_MENU != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            16319640041109108851 => {
                                if !flags & REDRAW_OUTSIDE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            11740996042087507061 => {
                                if !flags & REDRAW_EMPTY != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            10899678633117585587 => {
                                if !flags & REDRAW_PANE_BORDER != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            7695618138115701323 => {
                                if !flags & REDRAW_PANE_STATUS != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            6251484932569512093 => {
                                if !flags & REDRAW_PANE_SCROLLBAR != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                            _ => {
                                if !flags & REDRAW_PANE != 0 {
                                    current_block_9 = 7351195479953500246;
                                } else {
                                    current_block_9 = 11194104282611034094;
                                }
                            }
                        }
                    }
                    _ => {
                        current_block_9 = 7351195479953500246;
                    }
                }
            } else {
                current_block_9 = 11194104282611034094;
            }
            match current_block_9 {
                11194104282611034094 => {
                    spans = (&raw mut (*line).spans as *mut redraw_spans).offset(type_0 as isize)
                        as *mut redraw_spans;
                    span = (*spans).tqh_first;
                    while !span.is_null() {
                        redraw_draw_span(dctx, span, cy);
                        span = (*span).entry.tqe_next;
                    }
                }
                _ => {}
            }
            type_0 = type_0.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_draw_menu_lines(mut dctx: *mut redraw_draw_ctx) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut line: *mut redraw_line = ::core::ptr::null_mut::<redraw_line>();
    let mut span: *mut redraw_span = ::core::ptr::null_mut::<redraw_span>();
    let mut y: u_int = 0;
    let mut cy: u_int = 0;
    y = 0 as u_int;
    while y < (*scene).sy {
        line = (*scene).lines.offset(y as isize) as *mut redraw_line;
        if (*dctx).flags & REDRAW_STATUS_TOP != 0 {
            cy = (*dctx).status_lines.wrapping_add(y);
        } else {
            cy = y;
        }
        span = (*line).spans[REDRAW_SPAN_MENU as ::core::ffi::c_int as usize].tqh_first;
        while !span.is_null() {
            redraw_draw_span(dctx, span, cy);
            span = (*span).entry.tqe_next;
        }
        y = y.wrapping_add(1);
    }
}
unsafe extern "C" fn redraw_pane_status_line(
    mut dctx: *mut redraw_draw_ctx,
    mut wp: *mut window_pane,
    mut line: *mut u_int,
) -> ::core::ffi::c_int {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut pane_status: ::core::ffi::c_int = 0;
    let mut wy: ::core::ffi::c_int = 0;
    pane_status = window_pane_get_pane_status(wp);
    if pane_status == PANE_STATUS_OFF {
        return 0 as ::core::ffi::c_int;
    }
    if pane_status == PANE_STATUS_TOP {
        wy = (*wp).yoff - 1 as ::core::ffi::c_int;
    } else {
        wy = ((*wp).yoff as u_int).wrapping_add((*wp).sy) as ::core::ffi::c_int;
    }
    if wy < 0 as ::core::ffi::c_int || wy < (*scene).oy as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if wy as u_int >= (*scene).oy.wrapping_add((*scene).sy) {
        return 0 as ::core::ffi::c_int;
    }
    *line = (wy as u_int).wrapping_sub((*scene).oy);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn redraw_pane_status_width(
    mut dctx: *mut redraw_draw_ctx,
    mut wp: *mut window_pane,
    mut first: *mut *mut redraw_span,
) -> u_int {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut span: *mut redraw_span = ::core::ptr::null_mut::<redraw_span>();
    let mut y: u_int = 0;
    let mut width: u_int = 0 as u_int;
    let mut end: u_int = 0;
    if redraw_pane_status_line(dctx, wp, &raw mut y) == 0 {
        return 0 as u_int;
    }
    *first = ::core::ptr::null_mut::<redraw_span>();
    span = (*(*scene).lines.offset(y as isize)).spans
        [REDRAW_SPAN_STATUS as ::core::ffi::c_int as usize]
        .tqh_first;
    while !span.is_null() {
        if (*span).data.c2rust_unnamed.st.wp == wp {
            if (*first).is_null() {
                *first = span;
            }
            end = (*span)
                .data
                .c2rust_unnamed
                .st
                .offset
                .wrapping_add((*span).width);
            if end > width {
                width = end;
            }
        }
        span = (*span).entry.tqe_next;
    }
    return width;
}
unsafe extern "C" fn redraw_set_draw_context(
    mut dctx: *mut redraw_draw_ctx,
    mut scene: *mut redraw_scene,
) {
    let mut c: *mut client = (*scene).c;
    let mut s: *mut session = (*c).session;
    let mut oo: *mut options = (*s).options;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut lines: u_int = 0;
    memset(
        dctx as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<redraw_draw_ctx>() as size_t,
    );
    (*dctx).scene = scene;
    if server_is_marked(s, (*s).curw, marked_pane.wp) != 0 {
        (*dctx).marked = marked_pane.wp;
    }
    (*dctx).active = (*(*(*s).curw).window).active;
    lines = status_line_size(c);
    if options_get_number(
        oo,
        b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_longlong
    {
        (*dctx).flags |= REDRAW_STATUS_TOP;
    }
    (*dctx).status_lines = lines;
    if (*c).flags & CLIENT_UTF8 as uint64_t != 0 && tty_term_has((*tty).term, TTYC_BIDI) != 0 {
        (*dctx).flags |= REDRAW_ISOLATES;
    }
}
unsafe extern "C" fn redraw_draw_pane_prompt(
    mut dctx: *mut redraw_draw_ctx,
    mut wp: *mut window_pane,
) {
    let mut scene: *mut redraw_scene = (*dctx).scene;
    let mut c: *mut client = (*scene).c;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut screen: screen = screen {
        title: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        titles: ::core::ptr::null_mut::<screen_titles>(),
        ntitles: 0,
        grid: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
        cstyle: SCREEN_CURSOR_DEFAULT,
        default_cstyle: SCREEN_CURSOR_DEFAULT,
        ccolour: 0,
        default_ccolour: 0,
        rupper: 0,
        rlower: 0,
        mode: 0,
        default_mode: 0,
        saved_cx: 0,
        saved_cy: 0,
        saved_grid: ::core::ptr::null_mut::<grid>(),
        saved_cell: grid_cell {
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
        saved_flags: 0,
        tabs: ::core::ptr::null_mut::<bitstr_t>(),
        sel: ::core::ptr::null_mut::<screen_sel>(),
        write_list: ::core::ptr::null_mut::<screen_write_cline>(),
        hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        progress_bar: progress_bar {
            state: PROGRESS_BAR_HIDDEN,
            progress: 0,
        },
    };
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
    let mut pdd: prompt_draw_data = prompt_draw_data {
        ctx: ::core::ptr::null_mut::<screen_write_ctx>(),
        cursor_x: ::core::ptr::null_mut::<u_int>(),
        area_x: 0,
        area_width: 0,
        prompt_line: 0,
    };
    let mut ox: ::core::ffi::c_int = (*scene).ox as ::core::ffi::c_int;
    let mut oy: ::core::ffi::c_int = (*scene).oy as ::core::ffi::c_int;
    let mut sx: ::core::ffi::c_int = (*scene).sx as ::core::ffi::c_int;
    let mut sy: ::core::ffi::c_int = (*scene).sy as ::core::ffi::c_int;
    let mut line: ::core::ffi::c_int = 0;
    let mut cy: ::core::ffi::c_int = 0;
    let mut px: ::core::ffi::c_int = 0;
    let mut offset: ::core::ffi::c_int = 0;
    let mut width: ::core::ffi::c_int = 0;
    let mut wy: ::core::ffi::c_int = 0;
    if (*wp).prompt.is_null() || (*wp).sx == 0 as u_int || (*wp).sy == 0 as u_int {
        return;
    }
    if !(*dctx).flags & REDRAW_STATUS_TOP != 0 {
        wy = (*wp).yoff + (*wp).sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    } else {
        wy = (*wp).yoff;
    }
    if wy < oy || wy >= oy + sy {
        return;
    }
    line = wy - oy;
    if (*dctx).flags & REDRAW_STATUS_TOP != 0 {
        cy = (*dctx).status_lines.wrapping_add(line as u_int) as ::core::ffi::c_int;
    } else {
        cy = line;
    }
    if (*wp).xoff + (*wp).sx as ::core::ffi::c_int <= ox || (*wp).xoff >= ox + sx {
        return;
    }
    if (*wp).xoff < ox {
        offset = ox - (*wp).xoff;
        px = 0 as ::core::ffi::c_int;
    } else {
        offset = 0 as ::core::ffi::c_int;
        px = (*wp).xoff - ox;
    }
    width = (*wp).sx.wrapping_sub(offset as u_int) as ::core::ffi::c_int;
    if px + width > sx {
        width = sx - px;
    }
    screen_init(&raw mut screen, (*wp).sx, 1 as u_int, 0 as u_int);
    screen_write_start(&raw mut ctx, &raw mut screen);
    pdd.ctx = &raw mut ctx;
    pdd.cursor_x = &raw mut (*wp).prompt_cx;
    pdd.area_x = 0 as u_int;
    pdd.area_width = (*wp).sx;
    pdd.prompt_line = 0 as u_int;
    prompt_draw((*wp).prompt, &raw mut pdd);
    screen_write_stop(&raw mut ctx);
    tty_draw_line(
        tty,
        &raw mut screen,
        0 as u_int,
        offset as u_int,
        width as u_int,
        px as u_int,
        cy as u_int,
        ::core::ptr::null::<tty_style_ctx>(),
    );
    screen_free(&raw mut screen);
}
unsafe extern "C" fn redraw_draw(
    mut c: *mut client,
    mut wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
) {
    let mut dctx: redraw_draw_ctx = redraw_draw_ctx {
        scene: ::core::ptr::null_mut::<redraw_scene>(),
        active: ::core::ptr::null_mut::<window_pane>(),
        marked: ::core::ptr::null_mut::<window_pane>(),
        status_lines: 0,
        pane_lines: PANE_LINES_SINGLE,
        default_gc: grid_cell {
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
        flags: 0,
    };
    let mut s: *mut session = (*c).session;
    let mut w: *mut window = (*(*s).curw).window;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut sl: *mut screen = ::core::ptr::null_mut::<screen>();
    let mut scene: *mut redraw_scene = ::core::ptr::null_mut::<redraw_scene>();
    let mut loop_0: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut width: u_int = 0;
    let mut i: u_int = 0;
    let mut y: u_int = 0;
    let mut lines: u_int = 0;
    let mut j: u_int = 0;
    let mut first: *mut redraw_span = ::core::ptr::null_mut::<redraw_span>();
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut rr: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    let mut redraw: ::core::ffi::c_int = 0;
    if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
        return;
    }
    if flags & REDRAW_STATUS != 0 {
        if !(*c).message_string.is_null() {
            redraw = status_message_redraw(c);
        } else if !(*c).prompt.is_null() {
            redraw = status_prompt_redraw(c);
        } else {
            redraw = status_redraw(c);
        }
        if redraw == 0 && !(flags == REDRAW_ALL) {
            flags &= !REDRAW_STATUS;
            if flags == 0 as ::core::ffi::c_int {
                return;
            }
        }
    }
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(
            b"%s: starting @%u redraw (%s)\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            (*w).id,
            redraw_flags_to_string(flags),
        );
    }
    scene = redraw_get_scene(c);
    if scene.is_null() {
        return;
    }
    redraw_set_draw_context(&raw mut dctx, scene);
    if !(*w).menu.is_null() {
        menu_update((*w).menu);
    }
    if flags & (REDRAW_PANE_BORDER | REDRAW_PANE_STATUS) != 0 {
        loop_0 = (*(*scene).w).panes.tqh_first;
        while !loop_0.is_null() {
            (*loop_0).border_gc_set = 0 as ::core::ffi::c_int;
            (*loop_0).active_border_gc_set = 0 as ::core::ffi::c_int;
            loop_0 = (*loop_0).entry.tqe_next;
        }
    }
    if flags & REDRAW_PANE_STATUS != 0 {
        redraw = 0 as ::core::ffi::c_int;
        loop_0 = (*(*scene).w).panes.tqh_first;
        while !loop_0.is_null() {
            if flags == REDRAW_ALL {
                (*loop_0).flags |= PANE_NEWSTATUS;
            } else {
                (*loop_0).flags &= !PANE_NEWSTATUS;
            }
            width = redraw_pane_status_width(&raw mut dctx, loop_0, &raw mut first);
            if !(width == 0 as u_int) {
                if window_make_pane_status(loop_0, c, width, first) != 0 {
                    (*loop_0).flags |= PANE_NEWSTATUS;
                    redraw = 1 as ::core::ffi::c_int;
                }
            }
            loop_0 = (*loop_0).entry.tqe_next;
        }
        if redraw == 0 && !(flags == REDRAW_ALL) {
            flags &= !REDRAW_PANE_STATUS;
            if flags == 0 as ::core::ffi::c_int {
                return;
            }
        }
    }
    if flags & REDRAW_PANE != 0 {
        if !wp.is_null() {
            if (*wp).base.mode & MODE_SYNC != 0 {
                screen_write_stop_sync(wp);
            }
            screen_write_clear_dirty(wp);
        } else {
            loop_0 = (*(*scene).w).panes.tqh_first;
            while !loop_0.is_null() {
                if !(window_pane_is_visible(loop_0) == 0) {
                    if (*loop_0).base.mode & MODE_SYNC != 0 {
                        screen_write_stop_sync(loop_0);
                    }
                    screen_write_clear_dirty(loop_0);
                }
                loop_0 = (*loop_0).entry.tqe_next;
            }
        }
    }
    tty_sync_start(tty);
    tty_update_mode(
        tty,
        (*tty).mode & !CURSOR_MODES,
        ::core::ptr::null_mut::<screen>(),
    );
    if !wp.is_null() {
        redraw_draw_pane_lines(&raw mut dctx, wp, flags);
    } else {
        redraw_draw_lines(&raw mut dctx, flags);
    }
    if flags & REDRAW_PANE != 0 {
        if !wp.is_null() {
            redraw_draw_pane_prompt(&raw mut dctx, wp);
        } else {
            loop_0 = (*(*scene).w).panes.tqh_first;
            while !loop_0.is_null() {
                if window_pane_is_visible(loop_0) != 0 {
                    redraw_draw_pane_prompt(&raw mut dctx, loop_0);
                }
                loop_0 = (*loop_0).entry.tqe_next;
            }
        }
    }
    if !(*w).menu.is_null() && flags & REDRAW_MENU != 0 {
        redraw_draw_menu_lines(&raw mut dctx);
    }
    if flags & REDRAW_STATUS != 0 {
        lines = dctx.status_lines;
        if !(*c).message_string.is_null() || !(*c).prompt.is_null() {
            lines = if lines == 0 as u_int {
                1 as u_int
            } else {
                lines
            };
        }
        if dctx.flags & REDRAW_STATUS_TOP != 0 {
            y = 0 as u_int;
        } else {
            y = (*c).tty.sy.wrapping_sub(lines);
        }
        sl = (*c).status.active;
        i = 0 as u_int;
        while i < lines {
            r = tty_check_overlay_range(tty, 0 as u_int, y.wrapping_add(i), (*tty).sx);
            j = 0 as u_int;
            while j < (*r).used {
                rr = (*r).ranges.offset(j as isize) as *mut visible_range;
                if !((*rr).nx == 0 as u_int) {
                    tty_draw_line(
                        tty,
                        sl,
                        (*rr).px,
                        i,
                        (*rr).nx,
                        (*rr).px,
                        y.wrapping_add(i),
                        ::core::ptr::null::<tty_style_ctx>(),
                    );
                }
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
    }
    if (*c).overlay_draw.is_some() && flags & REDRAW_OVERLAY != 0 {
        (*c).overlay_draw.expect("non-null function pointer")(c, (*c).overlay_data);
    }
    tty_reset(tty);
    log_debug(
        b"%s: finished @%u redraw\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        (*(*scene).w).id,
    );
}
#[no_mangle]
pub unsafe extern "C" fn redraw_get_status_border_cell_type(
    mut spanp: *mut *mut redraw_span,
    mut x: u_int,
) -> ::core::ffi::c_int {
    let mut span: *mut redraw_span = *spanp;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    if span.is_null()
        || (*span).data.type_0 as ::core::ffi::c_uint
            != REDRAW_SPAN_STATUS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 2 as ::core::ffi::c_int;
    }
    wp = (*span).data.c2rust_unnamed.st.wp;
    while !span.is_null() {
        if !((*span).data.type_0 as ::core::ffi::c_uint
            != REDRAW_SPAN_STATUS as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            if !((*span).data.c2rust_unnamed.st.wp != wp) {
                start = (*span).data.c2rust_unnamed.st.offset;
                end = start.wrapping_add((*span).width);
                if x >= start && x < end {
                    *spanp = span;
                    return (*span).data.c2rust_unnamed.st.cell_type;
                }
                if start > x {
                    *spanp = span;
                    break;
                }
            }
        }
        span = (*span).entry.tqe_next;
    }
    if span.is_null() {
        *spanp = ::core::ptr::null_mut::<redraw_span>();
    }
    return 2 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn redraw_screen(mut c: *mut client) {
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
        if (*c).flags & CLIENT_REDRAWOVERLAY as uint64_t != 0 {
            redraw_draw(c, ::core::ptr::null_mut::<window_pane>(), REDRAW_ALL);
        } else {
            redraw_draw(
                c,
                ::core::ptr::null_mut::<window_pane>(),
                REDRAW_ALL & !REDRAW_OVERLAY,
            );
        }
    } else {
        if (*c).flags & CLIENT_REDRAWBORDERS as uint64_t != 0 {
            flags |= REDRAW_PANE_BORDER | REDRAW_PANE_STATUS;
        }
        if (*c).flags & CLIENT_REDRAWSTATUS as uint64_t != 0 {
            flags |= REDRAW_STATUS | REDRAW_PANE_STATUS;
        }
        if (*c).flags & CLIENT_REDRAWOVERLAY as uint64_t != 0 {
            flags |= REDRAW_OVERLAY;
        }
        if (*c).flags & CLIENT_REDRAWMENU as uint64_t != 0 {
            flags |= REDRAW_MENU;
        }
        if !(*(*(*(*c).session).curw).window).menu.is_null() {
            flags |= REDRAW_MENU;
        }
        if flags != 0 as ::core::ffi::c_int {
            redraw_draw(c, ::core::ptr::null_mut::<window_pane>(), flags);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn redraw_pane(mut c: *mut client, mut wp: *mut window_pane) {
    redraw_draw(c, wp, REDRAW_PANE | REDRAW_PANE_SCROLLBAR);
    if !(*(*(*(*c).session).curw).window).menu.is_null() {
        redraw_draw(c, ::core::ptr::null_mut::<window_pane>(), REDRAW_MENU);
    }
}
#[no_mangle]
pub unsafe extern "C" fn redraw_pane_scrollbar(mut c: *mut client, mut wp: *mut window_pane) {
    redraw_draw(c, wp, REDRAW_PANE_SCROLLBAR);
}

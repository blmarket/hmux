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
    pub type cmds;
    pub type menu_data;
    pub type window_pane_prompt;
    pub type prompt;
    pub type format_tree;
    pub type cmdq_item;
    pub type input_ctx;
    pub type spawn_editor_state;
    pub type input_request;
    pub type redraw_scene;
    pub type tty_key;
    pub type tty_code;
    pub type format_job_tree;
    pub type control_state;
    pub type cmdq_list;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn fnmatch(
        __pattern: *const ::core::ffi::c_char,
        __name: *const ::core::ffi::c_char,
        __flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
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
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strsep(
        __stringp: *mut *mut ::core::ffi::c_char,
        __delim: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn strtonum(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xstrndup(_: *const ::core::ffi::c_char, _: size_t) -> *mut ::core::ffi::c_char;
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
    static mut global_options: *mut options;
    static mut global_s_options: *mut options;
    static mut global_w_options: *mut options;
    fn checkshell(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn format_expand(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn hooks_monitor_free(_: *mut ::core::ffi::c_void);
    static options_table: [options_table_entry; 0];
    static options_other_names: [options_name_map; 0];
    fn tty_invalidate(_: *mut tty);
    fn tty_keys_build(_: *mut tty);
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn cmd_list_free(_: *mut cmd_list);
    fn cmd_list_print(_: *const cmd_list, _: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn cmd_parse_from_string(
        _: *const ::core::ffi::c_char,
        _: *mut cmd_parse_input,
    ) -> *mut cmd_parse_result;
    fn key_string_lookup_string(_: *const ::core::ffi::c_char) -> key_code;
    fn key_string_lookup_key(_: key_code, _: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn alerts_reset_all();
    static mut clients: clients;
    static mut current_time: time_t;
    fn server_client_set_key_table(_: *mut client, _: *const ::core::ffi::c_char);
    fn server_redraw_client(_: *mut client);
    fn server_client_update_theme_colours(_: *mut client);
    fn status_timer_start_all();
    fn status_update_cache(_: *mut session);
    fn recalculate_sizes();
    fn input_set_buffer_size(_: size_t);
    fn colour_tostring(_: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn colour_fromstring(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn colour_palette_from_option(_: *mut colour_palette, _: *mut options);
    static grid_default_cell: grid_cell;
    fn redraw_invalidate_all_scenes();
    static mut windows: windows;
    static mut all_window_panes: window_pane_tree;
    fn windows_RB_NEXT(_: *mut window) -> *mut window;
    fn windows_RB_MINMAX(_: *mut windows, _: ::core::ffi::c_int) -> *mut window;
    fn window_pane_tree_RB_MINMAX(
        _: *mut window_pane_tree,
        _: ::core::ffi::c_int,
    ) -> *mut window_pane;
    fn window_pane_tree_RB_NEXT(_: *mut window_pane) -> *mut window_pane;
    fn window_pane_default_cursor(_: *mut window_pane);
    fn window_pane_scrollbar_hide(_: *mut window_pane);
    fn window_set_fill_cells(_: *mut window);
    fn layout_fix_panes(_: *mut window, _: *mut window_pane);
    static mut sessions: sessions;
    fn sessions_RB_NEXT(_: *mut session) -> *mut session;
    fn sessions_RB_MINMAX(_: *mut sessions, _: ::core::ffi::c_int) -> *mut session;
    fn session_update_history(_: *mut session);
    fn utf8_update_width_cache();
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
    fn style_parse(
        _: *mut style,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn style_parse_colour(
        _: *mut style,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn style_set(_: *mut style, _: *const grid_cell);
    fn style_set_scrollbar_style_from_option(_: *mut style, _: *mut options);
}
pub type __builtin_va_list = [__va_list_tag; 1];
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __va_list_tag {
    pub gp_offset: ::core::ffi::c_uint,
    pub fp_offset: ::core::ffi::c_uint,
    pub overflow_arg_area: *mut ::core::ffi::c_void,
    pub reg_save_area: *mut ::core::ffi::c_void,
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
pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2RustUnnamed = 8;
pub const _ISpunct: C2RustUnnamed = 4;
pub const _IScntrl: C2RustUnnamed = 2;
pub const _ISblank: C2RustUnnamed = 1;
pub const _ISgraph: C2RustUnnamed = 32768;
pub const _ISprint: C2RustUnnamed = 16384;
pub const _ISspace: C2RustUnnamed = 8192;
pub const _ISxdigit: C2RustUnnamed = 4096;
pub const _ISdigit: C2RustUnnamed = 2048;
pub const _ISalpha: C2RustUnnamed = 1024;
pub const _ISlower: C2RustUnnamed = 512;
pub const _ISupper: C2RustUnnamed = 256;
pub type va_list = __builtin_va_list;
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
    pub c2rust_unnamed: C2RustUnnamed_1,
    pub c2rust_unnamed_0: C2RustUnnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_0 {
    pub __ospeed: speed_t,
    pub c_ospeed: speed_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_1 {
    pub __ispeed: speed_t,
    pub c_ispeed: speed_t,
}
pub type uint8_t = __uint8_t;
pub type uint64_t = __uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct event {
    pub ev_evcallback: event_callback,
    pub ev_timeout_pos: C2RustUnnamed_7,
    pub ev_fd: ::core::ffi::c_int,
    pub ev_base: *mut event_base,
    pub ev_: C2RustUnnamed_2,
    pub ev_events: ::core::ffi::c_short,
    pub ev_res: ::core::ffi::c_short,
    pub ev_timeout: timeval,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_2 {
    pub ev_io: C2RustUnnamed_5,
    pub ev_signal: C2RustUnnamed_3,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_3 {
    pub ev_signal_next: C2RustUnnamed_4,
    pub ev_ncalls: ::core::ffi::c_short,
    pub ev_pncalls: *mut ::core::ffi::c_short,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_4 {
    pub le_next: *mut event,
    pub le_prev: *mut *mut event,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_5 {
    pub ev_io_next: C2RustUnnamed_6,
    pub ev_timeout: timeval,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_6 {
    pub le_next: *mut event,
    pub le_prev: *mut *mut event,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_7 {
    pub ev_next_with_common_timeout: C2RustUnnamed_8,
    pub min_heap_idx: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_8 {
    pub tqe_next: *mut event,
    pub tqe_prev: *mut *mut event,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_callback {
    pub evcb_active_next: C2RustUnnamed_10,
    pub evcb_flags: ::core::ffi::c_short,
    pub evcb_pri: uint8_t,
    pub evcb_closure: uint8_t,
    pub evcb_cb_union: C2RustUnnamed_9,
    pub evcb_arg: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_9 {
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
pub struct C2RustUnnamed_10 {
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
    pub exit_type: C2RustUnnamed_36,
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
    pub entry: C2RustUnnamed_11,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_11 {
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
    pub entry: C2RustUnnamed_12,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_12 {
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
    pub c2rust_unnamed: C2RustUnnamed_13,
    pub flags: u_char,
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
    pub gentry: C2RustUnnamed_16,
    pub entry: C2RustUnnamed_15,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_15 {
    pub rbe_left: *mut session,
    pub rbe_right: *mut session,
    pub rbe_parent: *mut session,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_16 {
    pub tqe_next: *mut session,
    pub tqe_prev: *mut *mut session,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options {
    pub tree: options_tree,
    pub parent: *mut options,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_tree {
    pub rbh_root: *mut options_entry,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_entry {
    pub owner: *mut options,
    pub name: *const ::core::ffi::c_char,
    pub tableentry: *const options_table_entry,
    pub value: options_value,
    pub cached: ::core::ffi::c_int,
    pub style: style,
    pub monitor_data: *mut ::core::ffi::c_void,
    pub fire_count: u_int,
    pub fire_time: time_t,
    pub entry: C2RustUnnamed_17,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_17 {
    pub rbe_left: *mut options_entry,
    pub rbe_right: *mut options_entry,
    pub rbe_parent: *mut options_entry,
    pub rbe_color: ::core::ffi::c_int,
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
pub union options_value {
    pub string: *mut ::core::ffi::c_char,
    pub number: ::core::ffi::c_longlong,
    pub style: style,
    pub array: options_array,
    pub cmdlist: *mut cmd_list,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_list {
    pub references: ::core::ffi::c_int,
    pub group: u_int,
    pub list: *mut cmds,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_array {
    pub rbh_root: *mut options_array_item,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_array_item {
    pub key: *mut ::core::ffi::c_char,
    pub value: options_value,
    pub entry: C2RustUnnamed_18,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_18 {
    pub rbe_left: *mut options_array_item,
    pub rbe_right: *mut options_array_item,
    pub rbe_parent: *mut options_array_item,
    pub rbe_color: ::core::ffi::c_int,
}
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
    pub entry: C2RustUnnamed_21,
    pub wentry: C2RustUnnamed_20,
    pub sentry: C2RustUnnamed_19,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_19 {
    pub tqe_next: *mut winlink,
    pub tqe_prev: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_20 {
    pub tqe_next: *mut winlink,
    pub tqe_prev: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_21 {
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
    pub alerts_entry: C2RustUnnamed_24,
    pub options: *mut options,
    pub references: u_int,
    pub winlinks: C2RustUnnamed_23,
    pub entry: C2RustUnnamed_22,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_22 {
    pub rbe_left: *mut window,
    pub rbe_right: *mut window,
    pub rbe_parent: *mut window,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_23 {
    pub tqh_first: *mut winlink,
    pub tqh_last: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_24 {
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
    pub entry: C2RustUnnamed_25,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_25 {
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
    pub modes: C2RustUnnamed_30,
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
    pub entry: C2RustUnnamed_29,
    pub sentry: C2RustUnnamed_28,
    pub zentry: C2RustUnnamed_27,
    pub tree_entry: C2RustUnnamed_26,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_26 {
    pub rbe_left: *mut window_pane,
    pub rbe_right: *mut window_pane,
    pub rbe_parent: *mut window_pane,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_27 {
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_28 {
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_29 {
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_30 {
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
    pub entry: C2RustUnnamed_31,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_31 {
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
    pub entry: C2RustUnnamed_32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_32 {
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
    pub entry: C2RustUnnamed_33,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_33 {
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
    pub entry: C2RustUnnamed_34,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_34 {
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
    pub entry: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub rbe_left: *mut key_binding,
    pub rbe_right: *mut key_binding,
    pub rbe_parent: *mut key_binding,
    pub rbe_color: ::core::ffi::c_int,
}
pub type C2RustUnnamed_36 = ::core::ffi::c_uint;
pub const CLIENT_EXIT_DETACH: C2RustUnnamed_36 = 2;
pub const CLIENT_EXIT_SHUTDOWN: C2RustUnnamed_36 = 1;
pub const CLIENT_EXIT_RETURN: C2RustUnnamed_36 = 0;
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
    pub entry: C2RustUnnamed_37,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
    pub le_next: *mut tty_term,
    pub le_prev: *mut *mut tty_term,
}
pub type C2RustUnnamed_38 = ::core::ffi::c_ulong;
pub const KEYC_TRIPLECLICK11_CONTROL9: C2RustUnnamed_38 = 51539610387;
pub const KEYC_TRIPLECLICK10_CONTROL9: C2RustUnnamed_38 = 51539610131;
pub const KEYC_TRIPLECLICK9_CONTROL9: C2RustUnnamed_38 = 51539609875;
pub const KEYC_TRIPLECLICK8_CONTROL9: C2RustUnnamed_38 = 51539609619;
pub const KEYC_TRIPLECLICK7_CONTROL9: C2RustUnnamed_38 = 51539609363;
pub const KEYC_TRIPLECLICK6_CONTROL9: C2RustUnnamed_38 = 51539609107;
pub const KEYC_TRIPLECLICK3_CONTROL9: C2RustUnnamed_38 = 51539608339;
pub const KEYC_TRIPLECLICK2_CONTROL9: C2RustUnnamed_38 = 51539608083;
pub const KEYC_TRIPLECLICK1_CONTROL9: C2RustUnnamed_38 = 51539607827;
pub const KEYC_TRIPLECLICK_CONTROL9: C2RustUnnamed_38 = 51539607571;
pub const KEYC_TRIPLECLICK11_CONTROL8: C2RustUnnamed_38 = 51539610386;
pub const KEYC_TRIPLECLICK10_CONTROL8: C2RustUnnamed_38 = 51539610130;
pub const KEYC_TRIPLECLICK9_CONTROL8: C2RustUnnamed_38 = 51539609874;
pub const KEYC_TRIPLECLICK8_CONTROL8: C2RustUnnamed_38 = 51539609618;
pub const KEYC_TRIPLECLICK7_CONTROL8: C2RustUnnamed_38 = 51539609362;
pub const KEYC_TRIPLECLICK6_CONTROL8: C2RustUnnamed_38 = 51539609106;
pub const KEYC_TRIPLECLICK3_CONTROL8: C2RustUnnamed_38 = 51539608338;
pub const KEYC_TRIPLECLICK2_CONTROL8: C2RustUnnamed_38 = 51539608082;
pub const KEYC_TRIPLECLICK1_CONTROL8: C2RustUnnamed_38 = 51539607826;
pub const KEYC_TRIPLECLICK_CONTROL8: C2RustUnnamed_38 = 51539607570;
pub const KEYC_TRIPLECLICK11_CONTROL7: C2RustUnnamed_38 = 51539610385;
pub const KEYC_TRIPLECLICK10_CONTROL7: C2RustUnnamed_38 = 51539610129;
pub const KEYC_TRIPLECLICK9_CONTROL7: C2RustUnnamed_38 = 51539609873;
pub const KEYC_TRIPLECLICK8_CONTROL7: C2RustUnnamed_38 = 51539609617;
pub const KEYC_TRIPLECLICK7_CONTROL7: C2RustUnnamed_38 = 51539609361;
pub const KEYC_TRIPLECLICK6_CONTROL7: C2RustUnnamed_38 = 51539609105;
pub const KEYC_TRIPLECLICK3_CONTROL7: C2RustUnnamed_38 = 51539608337;
pub const KEYC_TRIPLECLICK2_CONTROL7: C2RustUnnamed_38 = 51539608081;
pub const KEYC_TRIPLECLICK1_CONTROL7: C2RustUnnamed_38 = 51539607825;
pub const KEYC_TRIPLECLICK_CONTROL7: C2RustUnnamed_38 = 51539607569;
pub const KEYC_TRIPLECLICK11_CONTROL6: C2RustUnnamed_38 = 51539610384;
pub const KEYC_TRIPLECLICK10_CONTROL6: C2RustUnnamed_38 = 51539610128;
pub const KEYC_TRIPLECLICK9_CONTROL6: C2RustUnnamed_38 = 51539609872;
pub const KEYC_TRIPLECLICK8_CONTROL6: C2RustUnnamed_38 = 51539609616;
pub const KEYC_TRIPLECLICK7_CONTROL6: C2RustUnnamed_38 = 51539609360;
pub const KEYC_TRIPLECLICK6_CONTROL6: C2RustUnnamed_38 = 51539609104;
pub const KEYC_TRIPLECLICK3_CONTROL6: C2RustUnnamed_38 = 51539608336;
pub const KEYC_TRIPLECLICK2_CONTROL6: C2RustUnnamed_38 = 51539608080;
pub const KEYC_TRIPLECLICK1_CONTROL6: C2RustUnnamed_38 = 51539607824;
pub const KEYC_TRIPLECLICK_CONTROL6: C2RustUnnamed_38 = 51539607568;
pub const KEYC_TRIPLECLICK11_CONTROL5: C2RustUnnamed_38 = 51539610383;
pub const KEYC_TRIPLECLICK10_CONTROL5: C2RustUnnamed_38 = 51539610127;
pub const KEYC_TRIPLECLICK9_CONTROL5: C2RustUnnamed_38 = 51539609871;
pub const KEYC_TRIPLECLICK8_CONTROL5: C2RustUnnamed_38 = 51539609615;
pub const KEYC_TRIPLECLICK7_CONTROL5: C2RustUnnamed_38 = 51539609359;
pub const KEYC_TRIPLECLICK6_CONTROL5: C2RustUnnamed_38 = 51539609103;
pub const KEYC_TRIPLECLICK3_CONTROL5: C2RustUnnamed_38 = 51539608335;
pub const KEYC_TRIPLECLICK2_CONTROL5: C2RustUnnamed_38 = 51539608079;
pub const KEYC_TRIPLECLICK1_CONTROL5: C2RustUnnamed_38 = 51539607823;
pub const KEYC_TRIPLECLICK_CONTROL5: C2RustUnnamed_38 = 51539607567;
pub const KEYC_TRIPLECLICK11_CONTROL4: C2RustUnnamed_38 = 51539610382;
pub const KEYC_TRIPLECLICK10_CONTROL4: C2RustUnnamed_38 = 51539610126;
pub const KEYC_TRIPLECLICK9_CONTROL4: C2RustUnnamed_38 = 51539609870;
pub const KEYC_TRIPLECLICK8_CONTROL4: C2RustUnnamed_38 = 51539609614;
pub const KEYC_TRIPLECLICK7_CONTROL4: C2RustUnnamed_38 = 51539609358;
pub const KEYC_TRIPLECLICK6_CONTROL4: C2RustUnnamed_38 = 51539609102;
pub const KEYC_TRIPLECLICK3_CONTROL4: C2RustUnnamed_38 = 51539608334;
pub const KEYC_TRIPLECLICK2_CONTROL4: C2RustUnnamed_38 = 51539608078;
pub const KEYC_TRIPLECLICK1_CONTROL4: C2RustUnnamed_38 = 51539607822;
pub const KEYC_TRIPLECLICK_CONTROL4: C2RustUnnamed_38 = 51539607566;
pub const KEYC_TRIPLECLICK11_CONTROL3: C2RustUnnamed_38 = 51539610381;
pub const KEYC_TRIPLECLICK10_CONTROL3: C2RustUnnamed_38 = 51539610125;
pub const KEYC_TRIPLECLICK9_CONTROL3: C2RustUnnamed_38 = 51539609869;
pub const KEYC_TRIPLECLICK8_CONTROL3: C2RustUnnamed_38 = 51539609613;
pub const KEYC_TRIPLECLICK7_CONTROL3: C2RustUnnamed_38 = 51539609357;
pub const KEYC_TRIPLECLICK6_CONTROL3: C2RustUnnamed_38 = 51539609101;
pub const KEYC_TRIPLECLICK3_CONTROL3: C2RustUnnamed_38 = 51539608333;
pub const KEYC_TRIPLECLICK2_CONTROL3: C2RustUnnamed_38 = 51539608077;
pub const KEYC_TRIPLECLICK1_CONTROL3: C2RustUnnamed_38 = 51539607821;
pub const KEYC_TRIPLECLICK_CONTROL3: C2RustUnnamed_38 = 51539607565;
pub const KEYC_TRIPLECLICK11_CONTROL2: C2RustUnnamed_38 = 51539610380;
pub const KEYC_TRIPLECLICK10_CONTROL2: C2RustUnnamed_38 = 51539610124;
pub const KEYC_TRIPLECLICK9_CONTROL2: C2RustUnnamed_38 = 51539609868;
pub const KEYC_TRIPLECLICK8_CONTROL2: C2RustUnnamed_38 = 51539609612;
pub const KEYC_TRIPLECLICK7_CONTROL2: C2RustUnnamed_38 = 51539609356;
pub const KEYC_TRIPLECLICK6_CONTROL2: C2RustUnnamed_38 = 51539609100;
pub const KEYC_TRIPLECLICK3_CONTROL2: C2RustUnnamed_38 = 51539608332;
pub const KEYC_TRIPLECLICK2_CONTROL2: C2RustUnnamed_38 = 51539608076;
pub const KEYC_TRIPLECLICK1_CONTROL2: C2RustUnnamed_38 = 51539607820;
pub const KEYC_TRIPLECLICK_CONTROL2: C2RustUnnamed_38 = 51539607564;
pub const KEYC_TRIPLECLICK11_CONTROL1: C2RustUnnamed_38 = 51539610379;
pub const KEYC_TRIPLECLICK10_CONTROL1: C2RustUnnamed_38 = 51539610123;
pub const KEYC_TRIPLECLICK9_CONTROL1: C2RustUnnamed_38 = 51539609867;
pub const KEYC_TRIPLECLICK8_CONTROL1: C2RustUnnamed_38 = 51539609611;
pub const KEYC_TRIPLECLICK7_CONTROL1: C2RustUnnamed_38 = 51539609355;
pub const KEYC_TRIPLECLICK6_CONTROL1: C2RustUnnamed_38 = 51539609099;
pub const KEYC_TRIPLECLICK3_CONTROL1: C2RustUnnamed_38 = 51539608331;
pub const KEYC_TRIPLECLICK2_CONTROL1: C2RustUnnamed_38 = 51539608075;
pub const KEYC_TRIPLECLICK1_CONTROL1: C2RustUnnamed_38 = 51539607819;
pub const KEYC_TRIPLECLICK_CONTROL1: C2RustUnnamed_38 = 51539607563;
pub const KEYC_TRIPLECLICK11_CONTROL0: C2RustUnnamed_38 = 51539610378;
pub const KEYC_TRIPLECLICK10_CONTROL0: C2RustUnnamed_38 = 51539610122;
pub const KEYC_TRIPLECLICK9_CONTROL0: C2RustUnnamed_38 = 51539609866;
pub const KEYC_TRIPLECLICK8_CONTROL0: C2RustUnnamed_38 = 51539609610;
pub const KEYC_TRIPLECLICK7_CONTROL0: C2RustUnnamed_38 = 51539609354;
pub const KEYC_TRIPLECLICK6_CONTROL0: C2RustUnnamed_38 = 51539609098;
pub const KEYC_TRIPLECLICK3_CONTROL0: C2RustUnnamed_38 = 51539608330;
pub const KEYC_TRIPLECLICK2_CONTROL0: C2RustUnnamed_38 = 51539608074;
pub const KEYC_TRIPLECLICK1_CONTROL0: C2RustUnnamed_38 = 51539607818;
pub const KEYC_TRIPLECLICK_CONTROL0: C2RustUnnamed_38 = 51539607562;
pub const KEYC_TRIPLECLICK11_EMPTY: C2RustUnnamed_38 = 51539610377;
pub const KEYC_TRIPLECLICK10_EMPTY: C2RustUnnamed_38 = 51539610121;
pub const KEYC_TRIPLECLICK9_EMPTY: C2RustUnnamed_38 = 51539609865;
pub const KEYC_TRIPLECLICK8_EMPTY: C2RustUnnamed_38 = 51539609609;
pub const KEYC_TRIPLECLICK7_EMPTY: C2RustUnnamed_38 = 51539609353;
pub const KEYC_TRIPLECLICK6_EMPTY: C2RustUnnamed_38 = 51539609097;
pub const KEYC_TRIPLECLICK3_EMPTY: C2RustUnnamed_38 = 51539608329;
pub const KEYC_TRIPLECLICK2_EMPTY: C2RustUnnamed_38 = 51539608073;
pub const KEYC_TRIPLECLICK1_EMPTY: C2RustUnnamed_38 = 51539607817;
pub const KEYC_TRIPLECLICK_EMPTY: C2RustUnnamed_38 = 51539607561;
pub const KEYC_TRIPLECLICK11_SCROLLBAR_DOWN: C2RustUnnamed_38 = 51539610376;
pub const KEYC_TRIPLECLICK10_SCROLLBAR_DOWN: C2RustUnnamed_38 = 51539610120;
pub const KEYC_TRIPLECLICK9_SCROLLBAR_DOWN: C2RustUnnamed_38 = 51539609864;
pub const KEYC_TRIPLECLICK8_SCROLLBAR_DOWN: C2RustUnnamed_38 = 51539609608;
pub const KEYC_TRIPLECLICK7_SCROLLBAR_DOWN: C2RustUnnamed_38 = 51539609352;
pub const KEYC_TRIPLECLICK6_SCROLLBAR_DOWN: C2RustUnnamed_38 = 51539609096;
pub const KEYC_TRIPLECLICK3_SCROLLBAR_DOWN: C2RustUnnamed_38 = 51539608328;
pub const KEYC_TRIPLECLICK2_SCROLLBAR_DOWN: C2RustUnnamed_38 = 51539608072;
pub const KEYC_TRIPLECLICK1_SCROLLBAR_DOWN: C2RustUnnamed_38 = 51539607816;
pub const KEYC_TRIPLECLICK_SCROLLBAR_DOWN: C2RustUnnamed_38 = 51539607560;
pub const KEYC_TRIPLECLICK11_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 51539610375;
pub const KEYC_TRIPLECLICK10_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 51539610119;
pub const KEYC_TRIPLECLICK9_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 51539609863;
pub const KEYC_TRIPLECLICK8_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 51539609607;
pub const KEYC_TRIPLECLICK7_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 51539609351;
pub const KEYC_TRIPLECLICK6_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 51539609095;
pub const KEYC_TRIPLECLICK3_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 51539608327;
pub const KEYC_TRIPLECLICK2_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 51539608071;
pub const KEYC_TRIPLECLICK1_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 51539607815;
pub const KEYC_TRIPLECLICK_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 51539607559;
pub const KEYC_TRIPLECLICK11_SCROLLBAR_UP: C2RustUnnamed_38 = 51539610374;
pub const KEYC_TRIPLECLICK10_SCROLLBAR_UP: C2RustUnnamed_38 = 51539610118;
pub const KEYC_TRIPLECLICK9_SCROLLBAR_UP: C2RustUnnamed_38 = 51539609862;
pub const KEYC_TRIPLECLICK8_SCROLLBAR_UP: C2RustUnnamed_38 = 51539609606;
pub const KEYC_TRIPLECLICK7_SCROLLBAR_UP: C2RustUnnamed_38 = 51539609350;
pub const KEYC_TRIPLECLICK6_SCROLLBAR_UP: C2RustUnnamed_38 = 51539609094;
pub const KEYC_TRIPLECLICK3_SCROLLBAR_UP: C2RustUnnamed_38 = 51539608326;
pub const KEYC_TRIPLECLICK2_SCROLLBAR_UP: C2RustUnnamed_38 = 51539608070;
pub const KEYC_TRIPLECLICK1_SCROLLBAR_UP: C2RustUnnamed_38 = 51539607814;
pub const KEYC_TRIPLECLICK_SCROLLBAR_UP: C2RustUnnamed_38 = 51539607558;
pub const KEYC_TRIPLECLICK11_BORDER: C2RustUnnamed_38 = 51539610373;
pub const KEYC_TRIPLECLICK10_BORDER: C2RustUnnamed_38 = 51539610117;
pub const KEYC_TRIPLECLICK9_BORDER: C2RustUnnamed_38 = 51539609861;
pub const KEYC_TRIPLECLICK8_BORDER: C2RustUnnamed_38 = 51539609605;
pub const KEYC_TRIPLECLICK7_BORDER: C2RustUnnamed_38 = 51539609349;
pub const KEYC_TRIPLECLICK6_BORDER: C2RustUnnamed_38 = 51539609093;
pub const KEYC_TRIPLECLICK3_BORDER: C2RustUnnamed_38 = 51539608325;
pub const KEYC_TRIPLECLICK2_BORDER: C2RustUnnamed_38 = 51539608069;
pub const KEYC_TRIPLECLICK1_BORDER: C2RustUnnamed_38 = 51539607813;
pub const KEYC_TRIPLECLICK_BORDER: C2RustUnnamed_38 = 51539607557;
pub const KEYC_TRIPLECLICK11_STATUS_DEFAULT: C2RustUnnamed_38 = 51539610372;
pub const KEYC_TRIPLECLICK10_STATUS_DEFAULT: C2RustUnnamed_38 = 51539610116;
pub const KEYC_TRIPLECLICK9_STATUS_DEFAULT: C2RustUnnamed_38 = 51539609860;
pub const KEYC_TRIPLECLICK8_STATUS_DEFAULT: C2RustUnnamed_38 = 51539609604;
pub const KEYC_TRIPLECLICK7_STATUS_DEFAULT: C2RustUnnamed_38 = 51539609348;
pub const KEYC_TRIPLECLICK6_STATUS_DEFAULT: C2RustUnnamed_38 = 51539609092;
pub const KEYC_TRIPLECLICK3_STATUS_DEFAULT: C2RustUnnamed_38 = 51539608324;
pub const KEYC_TRIPLECLICK2_STATUS_DEFAULT: C2RustUnnamed_38 = 51539608068;
pub const KEYC_TRIPLECLICK1_STATUS_DEFAULT: C2RustUnnamed_38 = 51539607812;
pub const KEYC_TRIPLECLICK_STATUS_DEFAULT: C2RustUnnamed_38 = 51539607556;
pub const KEYC_TRIPLECLICK11_STATUS_RIGHT: C2RustUnnamed_38 = 51539610371;
pub const KEYC_TRIPLECLICK10_STATUS_RIGHT: C2RustUnnamed_38 = 51539610115;
pub const KEYC_TRIPLECLICK9_STATUS_RIGHT: C2RustUnnamed_38 = 51539609859;
pub const KEYC_TRIPLECLICK8_STATUS_RIGHT: C2RustUnnamed_38 = 51539609603;
pub const KEYC_TRIPLECLICK7_STATUS_RIGHT: C2RustUnnamed_38 = 51539609347;
pub const KEYC_TRIPLECLICK6_STATUS_RIGHT: C2RustUnnamed_38 = 51539609091;
pub const KEYC_TRIPLECLICK3_STATUS_RIGHT: C2RustUnnamed_38 = 51539608323;
pub const KEYC_TRIPLECLICK2_STATUS_RIGHT: C2RustUnnamed_38 = 51539608067;
pub const KEYC_TRIPLECLICK1_STATUS_RIGHT: C2RustUnnamed_38 = 51539607811;
pub const KEYC_TRIPLECLICK_STATUS_RIGHT: C2RustUnnamed_38 = 51539607555;
pub const KEYC_TRIPLECLICK11_STATUS_LEFT: C2RustUnnamed_38 = 51539610370;
pub const KEYC_TRIPLECLICK10_STATUS_LEFT: C2RustUnnamed_38 = 51539610114;
pub const KEYC_TRIPLECLICK9_STATUS_LEFT: C2RustUnnamed_38 = 51539609858;
pub const KEYC_TRIPLECLICK8_STATUS_LEFT: C2RustUnnamed_38 = 51539609602;
pub const KEYC_TRIPLECLICK7_STATUS_LEFT: C2RustUnnamed_38 = 51539609346;
pub const KEYC_TRIPLECLICK6_STATUS_LEFT: C2RustUnnamed_38 = 51539609090;
pub const KEYC_TRIPLECLICK3_STATUS_LEFT: C2RustUnnamed_38 = 51539608322;
pub const KEYC_TRIPLECLICK2_STATUS_LEFT: C2RustUnnamed_38 = 51539608066;
pub const KEYC_TRIPLECLICK1_STATUS_LEFT: C2RustUnnamed_38 = 51539607810;
pub const KEYC_TRIPLECLICK_STATUS_LEFT: C2RustUnnamed_38 = 51539607554;
pub const KEYC_TRIPLECLICK11_STATUS: C2RustUnnamed_38 = 51539610369;
pub const KEYC_TRIPLECLICK10_STATUS: C2RustUnnamed_38 = 51539610113;
pub const KEYC_TRIPLECLICK9_STATUS: C2RustUnnamed_38 = 51539609857;
pub const KEYC_TRIPLECLICK8_STATUS: C2RustUnnamed_38 = 51539609601;
pub const KEYC_TRIPLECLICK7_STATUS: C2RustUnnamed_38 = 51539609345;
pub const KEYC_TRIPLECLICK6_STATUS: C2RustUnnamed_38 = 51539609089;
pub const KEYC_TRIPLECLICK3_STATUS: C2RustUnnamed_38 = 51539608321;
pub const KEYC_TRIPLECLICK2_STATUS: C2RustUnnamed_38 = 51539608065;
pub const KEYC_TRIPLECLICK1_STATUS: C2RustUnnamed_38 = 51539607809;
pub const KEYC_TRIPLECLICK_STATUS: C2RustUnnamed_38 = 51539607553;
pub const KEYC_TRIPLECLICK11_PANE: C2RustUnnamed_38 = 51539610368;
pub const KEYC_TRIPLECLICK10_PANE: C2RustUnnamed_38 = 51539610112;
pub const KEYC_TRIPLECLICK9_PANE: C2RustUnnamed_38 = 51539609856;
pub const KEYC_TRIPLECLICK8_PANE: C2RustUnnamed_38 = 51539609600;
pub const KEYC_TRIPLECLICK7_PANE: C2RustUnnamed_38 = 51539609344;
pub const KEYC_TRIPLECLICK6_PANE: C2RustUnnamed_38 = 51539609088;
pub const KEYC_TRIPLECLICK3_PANE: C2RustUnnamed_38 = 51539608320;
pub const KEYC_TRIPLECLICK2_PANE: C2RustUnnamed_38 = 51539608064;
pub const KEYC_TRIPLECLICK1_PANE: C2RustUnnamed_38 = 51539607808;
pub const KEYC_TRIPLECLICK_PANE: C2RustUnnamed_38 = 51539607552;
pub const KEYC_DOUBLECLICK11_CONTROL9: C2RustUnnamed_38 = 47244643091;
pub const KEYC_DOUBLECLICK10_CONTROL9: C2RustUnnamed_38 = 47244642835;
pub const KEYC_DOUBLECLICK9_CONTROL9: C2RustUnnamed_38 = 47244642579;
pub const KEYC_DOUBLECLICK8_CONTROL9: C2RustUnnamed_38 = 47244642323;
pub const KEYC_DOUBLECLICK7_CONTROL9: C2RustUnnamed_38 = 47244642067;
pub const KEYC_DOUBLECLICK6_CONTROL9: C2RustUnnamed_38 = 47244641811;
pub const KEYC_DOUBLECLICK3_CONTROL9: C2RustUnnamed_38 = 47244641043;
pub const KEYC_DOUBLECLICK2_CONTROL9: C2RustUnnamed_38 = 47244640787;
pub const KEYC_DOUBLECLICK1_CONTROL9: C2RustUnnamed_38 = 47244640531;
pub const KEYC_DOUBLECLICK_CONTROL9: C2RustUnnamed_38 = 47244640275;
pub const KEYC_DOUBLECLICK11_CONTROL8: C2RustUnnamed_38 = 47244643090;
pub const KEYC_DOUBLECLICK10_CONTROL8: C2RustUnnamed_38 = 47244642834;
pub const KEYC_DOUBLECLICK9_CONTROL8: C2RustUnnamed_38 = 47244642578;
pub const KEYC_DOUBLECLICK8_CONTROL8: C2RustUnnamed_38 = 47244642322;
pub const KEYC_DOUBLECLICK7_CONTROL8: C2RustUnnamed_38 = 47244642066;
pub const KEYC_DOUBLECLICK6_CONTROL8: C2RustUnnamed_38 = 47244641810;
pub const KEYC_DOUBLECLICK3_CONTROL8: C2RustUnnamed_38 = 47244641042;
pub const KEYC_DOUBLECLICK2_CONTROL8: C2RustUnnamed_38 = 47244640786;
pub const KEYC_DOUBLECLICK1_CONTROL8: C2RustUnnamed_38 = 47244640530;
pub const KEYC_DOUBLECLICK_CONTROL8: C2RustUnnamed_38 = 47244640274;
pub const KEYC_DOUBLECLICK11_CONTROL7: C2RustUnnamed_38 = 47244643089;
pub const KEYC_DOUBLECLICK10_CONTROL7: C2RustUnnamed_38 = 47244642833;
pub const KEYC_DOUBLECLICK9_CONTROL7: C2RustUnnamed_38 = 47244642577;
pub const KEYC_DOUBLECLICK8_CONTROL7: C2RustUnnamed_38 = 47244642321;
pub const KEYC_DOUBLECLICK7_CONTROL7: C2RustUnnamed_38 = 47244642065;
pub const KEYC_DOUBLECLICK6_CONTROL7: C2RustUnnamed_38 = 47244641809;
pub const KEYC_DOUBLECLICK3_CONTROL7: C2RustUnnamed_38 = 47244641041;
pub const KEYC_DOUBLECLICK2_CONTROL7: C2RustUnnamed_38 = 47244640785;
pub const KEYC_DOUBLECLICK1_CONTROL7: C2RustUnnamed_38 = 47244640529;
pub const KEYC_DOUBLECLICK_CONTROL7: C2RustUnnamed_38 = 47244640273;
pub const KEYC_DOUBLECLICK11_CONTROL6: C2RustUnnamed_38 = 47244643088;
pub const KEYC_DOUBLECLICK10_CONTROL6: C2RustUnnamed_38 = 47244642832;
pub const KEYC_DOUBLECLICK9_CONTROL6: C2RustUnnamed_38 = 47244642576;
pub const KEYC_DOUBLECLICK8_CONTROL6: C2RustUnnamed_38 = 47244642320;
pub const KEYC_DOUBLECLICK7_CONTROL6: C2RustUnnamed_38 = 47244642064;
pub const KEYC_DOUBLECLICK6_CONTROL6: C2RustUnnamed_38 = 47244641808;
pub const KEYC_DOUBLECLICK3_CONTROL6: C2RustUnnamed_38 = 47244641040;
pub const KEYC_DOUBLECLICK2_CONTROL6: C2RustUnnamed_38 = 47244640784;
pub const KEYC_DOUBLECLICK1_CONTROL6: C2RustUnnamed_38 = 47244640528;
pub const KEYC_DOUBLECLICK_CONTROL6: C2RustUnnamed_38 = 47244640272;
pub const KEYC_DOUBLECLICK11_CONTROL5: C2RustUnnamed_38 = 47244643087;
pub const KEYC_DOUBLECLICK10_CONTROL5: C2RustUnnamed_38 = 47244642831;
pub const KEYC_DOUBLECLICK9_CONTROL5: C2RustUnnamed_38 = 47244642575;
pub const KEYC_DOUBLECLICK8_CONTROL5: C2RustUnnamed_38 = 47244642319;
pub const KEYC_DOUBLECLICK7_CONTROL5: C2RustUnnamed_38 = 47244642063;
pub const KEYC_DOUBLECLICK6_CONTROL5: C2RustUnnamed_38 = 47244641807;
pub const KEYC_DOUBLECLICK3_CONTROL5: C2RustUnnamed_38 = 47244641039;
pub const KEYC_DOUBLECLICK2_CONTROL5: C2RustUnnamed_38 = 47244640783;
pub const KEYC_DOUBLECLICK1_CONTROL5: C2RustUnnamed_38 = 47244640527;
pub const KEYC_DOUBLECLICK_CONTROL5: C2RustUnnamed_38 = 47244640271;
pub const KEYC_DOUBLECLICK11_CONTROL4: C2RustUnnamed_38 = 47244643086;
pub const KEYC_DOUBLECLICK10_CONTROL4: C2RustUnnamed_38 = 47244642830;
pub const KEYC_DOUBLECLICK9_CONTROL4: C2RustUnnamed_38 = 47244642574;
pub const KEYC_DOUBLECLICK8_CONTROL4: C2RustUnnamed_38 = 47244642318;
pub const KEYC_DOUBLECLICK7_CONTROL4: C2RustUnnamed_38 = 47244642062;
pub const KEYC_DOUBLECLICK6_CONTROL4: C2RustUnnamed_38 = 47244641806;
pub const KEYC_DOUBLECLICK3_CONTROL4: C2RustUnnamed_38 = 47244641038;
pub const KEYC_DOUBLECLICK2_CONTROL4: C2RustUnnamed_38 = 47244640782;
pub const KEYC_DOUBLECLICK1_CONTROL4: C2RustUnnamed_38 = 47244640526;
pub const KEYC_DOUBLECLICK_CONTROL4: C2RustUnnamed_38 = 47244640270;
pub const KEYC_DOUBLECLICK11_CONTROL3: C2RustUnnamed_38 = 47244643085;
pub const KEYC_DOUBLECLICK10_CONTROL3: C2RustUnnamed_38 = 47244642829;
pub const KEYC_DOUBLECLICK9_CONTROL3: C2RustUnnamed_38 = 47244642573;
pub const KEYC_DOUBLECLICK8_CONTROL3: C2RustUnnamed_38 = 47244642317;
pub const KEYC_DOUBLECLICK7_CONTROL3: C2RustUnnamed_38 = 47244642061;
pub const KEYC_DOUBLECLICK6_CONTROL3: C2RustUnnamed_38 = 47244641805;
pub const KEYC_DOUBLECLICK3_CONTROL3: C2RustUnnamed_38 = 47244641037;
pub const KEYC_DOUBLECLICK2_CONTROL3: C2RustUnnamed_38 = 47244640781;
pub const KEYC_DOUBLECLICK1_CONTROL3: C2RustUnnamed_38 = 47244640525;
pub const KEYC_DOUBLECLICK_CONTROL3: C2RustUnnamed_38 = 47244640269;
pub const KEYC_DOUBLECLICK11_CONTROL2: C2RustUnnamed_38 = 47244643084;
pub const KEYC_DOUBLECLICK10_CONTROL2: C2RustUnnamed_38 = 47244642828;
pub const KEYC_DOUBLECLICK9_CONTROL2: C2RustUnnamed_38 = 47244642572;
pub const KEYC_DOUBLECLICK8_CONTROL2: C2RustUnnamed_38 = 47244642316;
pub const KEYC_DOUBLECLICK7_CONTROL2: C2RustUnnamed_38 = 47244642060;
pub const KEYC_DOUBLECLICK6_CONTROL2: C2RustUnnamed_38 = 47244641804;
pub const KEYC_DOUBLECLICK3_CONTROL2: C2RustUnnamed_38 = 47244641036;
pub const KEYC_DOUBLECLICK2_CONTROL2: C2RustUnnamed_38 = 47244640780;
pub const KEYC_DOUBLECLICK1_CONTROL2: C2RustUnnamed_38 = 47244640524;
pub const KEYC_DOUBLECLICK_CONTROL2: C2RustUnnamed_38 = 47244640268;
pub const KEYC_DOUBLECLICK11_CONTROL1: C2RustUnnamed_38 = 47244643083;
pub const KEYC_DOUBLECLICK10_CONTROL1: C2RustUnnamed_38 = 47244642827;
pub const KEYC_DOUBLECLICK9_CONTROL1: C2RustUnnamed_38 = 47244642571;
pub const KEYC_DOUBLECLICK8_CONTROL1: C2RustUnnamed_38 = 47244642315;
pub const KEYC_DOUBLECLICK7_CONTROL1: C2RustUnnamed_38 = 47244642059;
pub const KEYC_DOUBLECLICK6_CONTROL1: C2RustUnnamed_38 = 47244641803;
pub const KEYC_DOUBLECLICK3_CONTROL1: C2RustUnnamed_38 = 47244641035;
pub const KEYC_DOUBLECLICK2_CONTROL1: C2RustUnnamed_38 = 47244640779;
pub const KEYC_DOUBLECLICK1_CONTROL1: C2RustUnnamed_38 = 47244640523;
pub const KEYC_DOUBLECLICK_CONTROL1: C2RustUnnamed_38 = 47244640267;
pub const KEYC_DOUBLECLICK11_CONTROL0: C2RustUnnamed_38 = 47244643082;
pub const KEYC_DOUBLECLICK10_CONTROL0: C2RustUnnamed_38 = 47244642826;
pub const KEYC_DOUBLECLICK9_CONTROL0: C2RustUnnamed_38 = 47244642570;
pub const KEYC_DOUBLECLICK8_CONTROL0: C2RustUnnamed_38 = 47244642314;
pub const KEYC_DOUBLECLICK7_CONTROL0: C2RustUnnamed_38 = 47244642058;
pub const KEYC_DOUBLECLICK6_CONTROL0: C2RustUnnamed_38 = 47244641802;
pub const KEYC_DOUBLECLICK3_CONTROL0: C2RustUnnamed_38 = 47244641034;
pub const KEYC_DOUBLECLICK2_CONTROL0: C2RustUnnamed_38 = 47244640778;
pub const KEYC_DOUBLECLICK1_CONTROL0: C2RustUnnamed_38 = 47244640522;
pub const KEYC_DOUBLECLICK_CONTROL0: C2RustUnnamed_38 = 47244640266;
pub const KEYC_DOUBLECLICK11_EMPTY: C2RustUnnamed_38 = 47244643081;
pub const KEYC_DOUBLECLICK10_EMPTY: C2RustUnnamed_38 = 47244642825;
pub const KEYC_DOUBLECLICK9_EMPTY: C2RustUnnamed_38 = 47244642569;
pub const KEYC_DOUBLECLICK8_EMPTY: C2RustUnnamed_38 = 47244642313;
pub const KEYC_DOUBLECLICK7_EMPTY: C2RustUnnamed_38 = 47244642057;
pub const KEYC_DOUBLECLICK6_EMPTY: C2RustUnnamed_38 = 47244641801;
pub const KEYC_DOUBLECLICK3_EMPTY: C2RustUnnamed_38 = 47244641033;
pub const KEYC_DOUBLECLICK2_EMPTY: C2RustUnnamed_38 = 47244640777;
pub const KEYC_DOUBLECLICK1_EMPTY: C2RustUnnamed_38 = 47244640521;
pub const KEYC_DOUBLECLICK_EMPTY: C2RustUnnamed_38 = 47244640265;
pub const KEYC_DOUBLECLICK11_SCROLLBAR_DOWN: C2RustUnnamed_38 = 47244643080;
pub const KEYC_DOUBLECLICK10_SCROLLBAR_DOWN: C2RustUnnamed_38 = 47244642824;
pub const KEYC_DOUBLECLICK9_SCROLLBAR_DOWN: C2RustUnnamed_38 = 47244642568;
pub const KEYC_DOUBLECLICK8_SCROLLBAR_DOWN: C2RustUnnamed_38 = 47244642312;
pub const KEYC_DOUBLECLICK7_SCROLLBAR_DOWN: C2RustUnnamed_38 = 47244642056;
pub const KEYC_DOUBLECLICK6_SCROLLBAR_DOWN: C2RustUnnamed_38 = 47244641800;
pub const KEYC_DOUBLECLICK3_SCROLLBAR_DOWN: C2RustUnnamed_38 = 47244641032;
pub const KEYC_DOUBLECLICK2_SCROLLBAR_DOWN: C2RustUnnamed_38 = 47244640776;
pub const KEYC_DOUBLECLICK1_SCROLLBAR_DOWN: C2RustUnnamed_38 = 47244640520;
pub const KEYC_DOUBLECLICK_SCROLLBAR_DOWN: C2RustUnnamed_38 = 47244640264;
pub const KEYC_DOUBLECLICK11_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 47244643079;
pub const KEYC_DOUBLECLICK10_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 47244642823;
pub const KEYC_DOUBLECLICK9_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 47244642567;
pub const KEYC_DOUBLECLICK8_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 47244642311;
pub const KEYC_DOUBLECLICK7_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 47244642055;
pub const KEYC_DOUBLECLICK6_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 47244641799;
pub const KEYC_DOUBLECLICK3_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 47244641031;
pub const KEYC_DOUBLECLICK2_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 47244640775;
pub const KEYC_DOUBLECLICK1_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 47244640519;
pub const KEYC_DOUBLECLICK_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 47244640263;
pub const KEYC_DOUBLECLICK11_SCROLLBAR_UP: C2RustUnnamed_38 = 47244643078;
pub const KEYC_DOUBLECLICK10_SCROLLBAR_UP: C2RustUnnamed_38 = 47244642822;
pub const KEYC_DOUBLECLICK9_SCROLLBAR_UP: C2RustUnnamed_38 = 47244642566;
pub const KEYC_DOUBLECLICK8_SCROLLBAR_UP: C2RustUnnamed_38 = 47244642310;
pub const KEYC_DOUBLECLICK7_SCROLLBAR_UP: C2RustUnnamed_38 = 47244642054;
pub const KEYC_DOUBLECLICK6_SCROLLBAR_UP: C2RustUnnamed_38 = 47244641798;
pub const KEYC_DOUBLECLICK3_SCROLLBAR_UP: C2RustUnnamed_38 = 47244641030;
pub const KEYC_DOUBLECLICK2_SCROLLBAR_UP: C2RustUnnamed_38 = 47244640774;
pub const KEYC_DOUBLECLICK1_SCROLLBAR_UP: C2RustUnnamed_38 = 47244640518;
pub const KEYC_DOUBLECLICK_SCROLLBAR_UP: C2RustUnnamed_38 = 47244640262;
pub const KEYC_DOUBLECLICK11_BORDER: C2RustUnnamed_38 = 47244643077;
pub const KEYC_DOUBLECLICK10_BORDER: C2RustUnnamed_38 = 47244642821;
pub const KEYC_DOUBLECLICK9_BORDER: C2RustUnnamed_38 = 47244642565;
pub const KEYC_DOUBLECLICK8_BORDER: C2RustUnnamed_38 = 47244642309;
pub const KEYC_DOUBLECLICK7_BORDER: C2RustUnnamed_38 = 47244642053;
pub const KEYC_DOUBLECLICK6_BORDER: C2RustUnnamed_38 = 47244641797;
pub const KEYC_DOUBLECLICK3_BORDER: C2RustUnnamed_38 = 47244641029;
pub const KEYC_DOUBLECLICK2_BORDER: C2RustUnnamed_38 = 47244640773;
pub const KEYC_DOUBLECLICK1_BORDER: C2RustUnnamed_38 = 47244640517;
pub const KEYC_DOUBLECLICK_BORDER: C2RustUnnamed_38 = 47244640261;
pub const KEYC_DOUBLECLICK11_STATUS_DEFAULT: C2RustUnnamed_38 = 47244643076;
pub const KEYC_DOUBLECLICK10_STATUS_DEFAULT: C2RustUnnamed_38 = 47244642820;
pub const KEYC_DOUBLECLICK9_STATUS_DEFAULT: C2RustUnnamed_38 = 47244642564;
pub const KEYC_DOUBLECLICK8_STATUS_DEFAULT: C2RustUnnamed_38 = 47244642308;
pub const KEYC_DOUBLECLICK7_STATUS_DEFAULT: C2RustUnnamed_38 = 47244642052;
pub const KEYC_DOUBLECLICK6_STATUS_DEFAULT: C2RustUnnamed_38 = 47244641796;
pub const KEYC_DOUBLECLICK3_STATUS_DEFAULT: C2RustUnnamed_38 = 47244641028;
pub const KEYC_DOUBLECLICK2_STATUS_DEFAULT: C2RustUnnamed_38 = 47244640772;
pub const KEYC_DOUBLECLICK1_STATUS_DEFAULT: C2RustUnnamed_38 = 47244640516;
pub const KEYC_DOUBLECLICK_STATUS_DEFAULT: C2RustUnnamed_38 = 47244640260;
pub const KEYC_DOUBLECLICK11_STATUS_RIGHT: C2RustUnnamed_38 = 47244643075;
pub const KEYC_DOUBLECLICK10_STATUS_RIGHT: C2RustUnnamed_38 = 47244642819;
pub const KEYC_DOUBLECLICK9_STATUS_RIGHT: C2RustUnnamed_38 = 47244642563;
pub const KEYC_DOUBLECLICK8_STATUS_RIGHT: C2RustUnnamed_38 = 47244642307;
pub const KEYC_DOUBLECLICK7_STATUS_RIGHT: C2RustUnnamed_38 = 47244642051;
pub const KEYC_DOUBLECLICK6_STATUS_RIGHT: C2RustUnnamed_38 = 47244641795;
pub const KEYC_DOUBLECLICK3_STATUS_RIGHT: C2RustUnnamed_38 = 47244641027;
pub const KEYC_DOUBLECLICK2_STATUS_RIGHT: C2RustUnnamed_38 = 47244640771;
pub const KEYC_DOUBLECLICK1_STATUS_RIGHT: C2RustUnnamed_38 = 47244640515;
pub const KEYC_DOUBLECLICK_STATUS_RIGHT: C2RustUnnamed_38 = 47244640259;
pub const KEYC_DOUBLECLICK11_STATUS_LEFT: C2RustUnnamed_38 = 47244643074;
pub const KEYC_DOUBLECLICK10_STATUS_LEFT: C2RustUnnamed_38 = 47244642818;
pub const KEYC_DOUBLECLICK9_STATUS_LEFT: C2RustUnnamed_38 = 47244642562;
pub const KEYC_DOUBLECLICK8_STATUS_LEFT: C2RustUnnamed_38 = 47244642306;
pub const KEYC_DOUBLECLICK7_STATUS_LEFT: C2RustUnnamed_38 = 47244642050;
pub const KEYC_DOUBLECLICK6_STATUS_LEFT: C2RustUnnamed_38 = 47244641794;
pub const KEYC_DOUBLECLICK3_STATUS_LEFT: C2RustUnnamed_38 = 47244641026;
pub const KEYC_DOUBLECLICK2_STATUS_LEFT: C2RustUnnamed_38 = 47244640770;
pub const KEYC_DOUBLECLICK1_STATUS_LEFT: C2RustUnnamed_38 = 47244640514;
pub const KEYC_DOUBLECLICK_STATUS_LEFT: C2RustUnnamed_38 = 47244640258;
pub const KEYC_DOUBLECLICK11_STATUS: C2RustUnnamed_38 = 47244643073;
pub const KEYC_DOUBLECLICK10_STATUS: C2RustUnnamed_38 = 47244642817;
pub const KEYC_DOUBLECLICK9_STATUS: C2RustUnnamed_38 = 47244642561;
pub const KEYC_DOUBLECLICK8_STATUS: C2RustUnnamed_38 = 47244642305;
pub const KEYC_DOUBLECLICK7_STATUS: C2RustUnnamed_38 = 47244642049;
pub const KEYC_DOUBLECLICK6_STATUS: C2RustUnnamed_38 = 47244641793;
pub const KEYC_DOUBLECLICK3_STATUS: C2RustUnnamed_38 = 47244641025;
pub const KEYC_DOUBLECLICK2_STATUS: C2RustUnnamed_38 = 47244640769;
pub const KEYC_DOUBLECLICK1_STATUS: C2RustUnnamed_38 = 47244640513;
pub const KEYC_DOUBLECLICK_STATUS: C2RustUnnamed_38 = 47244640257;
pub const KEYC_DOUBLECLICK11_PANE: C2RustUnnamed_38 = 47244643072;
pub const KEYC_DOUBLECLICK10_PANE: C2RustUnnamed_38 = 47244642816;
pub const KEYC_DOUBLECLICK9_PANE: C2RustUnnamed_38 = 47244642560;
pub const KEYC_DOUBLECLICK8_PANE: C2RustUnnamed_38 = 47244642304;
pub const KEYC_DOUBLECLICK7_PANE: C2RustUnnamed_38 = 47244642048;
pub const KEYC_DOUBLECLICK6_PANE: C2RustUnnamed_38 = 47244641792;
pub const KEYC_DOUBLECLICK3_PANE: C2RustUnnamed_38 = 47244641024;
pub const KEYC_DOUBLECLICK2_PANE: C2RustUnnamed_38 = 47244640768;
pub const KEYC_DOUBLECLICK1_PANE: C2RustUnnamed_38 = 47244640512;
pub const KEYC_DOUBLECLICK_PANE: C2RustUnnamed_38 = 47244640256;
pub const KEYC_SECONDCLICK11_CONTROL9: C2RustUnnamed_38 = 42949675795;
pub const KEYC_SECONDCLICK10_CONTROL9: C2RustUnnamed_38 = 42949675539;
pub const KEYC_SECONDCLICK9_CONTROL9: C2RustUnnamed_38 = 42949675283;
pub const KEYC_SECONDCLICK8_CONTROL9: C2RustUnnamed_38 = 42949675027;
pub const KEYC_SECONDCLICK7_CONTROL9: C2RustUnnamed_38 = 42949674771;
pub const KEYC_SECONDCLICK6_CONTROL9: C2RustUnnamed_38 = 42949674515;
pub const KEYC_SECONDCLICK3_CONTROL9: C2RustUnnamed_38 = 42949673747;
pub const KEYC_SECONDCLICK2_CONTROL9: C2RustUnnamed_38 = 42949673491;
pub const KEYC_SECONDCLICK1_CONTROL9: C2RustUnnamed_38 = 42949673235;
pub const KEYC_SECONDCLICK_CONTROL9: C2RustUnnamed_38 = 42949672979;
pub const KEYC_SECONDCLICK11_CONTROL8: C2RustUnnamed_38 = 42949675794;
pub const KEYC_SECONDCLICK10_CONTROL8: C2RustUnnamed_38 = 42949675538;
pub const KEYC_SECONDCLICK9_CONTROL8: C2RustUnnamed_38 = 42949675282;
pub const KEYC_SECONDCLICK8_CONTROL8: C2RustUnnamed_38 = 42949675026;
pub const KEYC_SECONDCLICK7_CONTROL8: C2RustUnnamed_38 = 42949674770;
pub const KEYC_SECONDCLICK6_CONTROL8: C2RustUnnamed_38 = 42949674514;
pub const KEYC_SECONDCLICK3_CONTROL8: C2RustUnnamed_38 = 42949673746;
pub const KEYC_SECONDCLICK2_CONTROL8: C2RustUnnamed_38 = 42949673490;
pub const KEYC_SECONDCLICK1_CONTROL8: C2RustUnnamed_38 = 42949673234;
pub const KEYC_SECONDCLICK_CONTROL8: C2RustUnnamed_38 = 42949672978;
pub const KEYC_SECONDCLICK11_CONTROL7: C2RustUnnamed_38 = 42949675793;
pub const KEYC_SECONDCLICK10_CONTROL7: C2RustUnnamed_38 = 42949675537;
pub const KEYC_SECONDCLICK9_CONTROL7: C2RustUnnamed_38 = 42949675281;
pub const KEYC_SECONDCLICK8_CONTROL7: C2RustUnnamed_38 = 42949675025;
pub const KEYC_SECONDCLICK7_CONTROL7: C2RustUnnamed_38 = 42949674769;
pub const KEYC_SECONDCLICK6_CONTROL7: C2RustUnnamed_38 = 42949674513;
pub const KEYC_SECONDCLICK3_CONTROL7: C2RustUnnamed_38 = 42949673745;
pub const KEYC_SECONDCLICK2_CONTROL7: C2RustUnnamed_38 = 42949673489;
pub const KEYC_SECONDCLICK1_CONTROL7: C2RustUnnamed_38 = 42949673233;
pub const KEYC_SECONDCLICK_CONTROL7: C2RustUnnamed_38 = 42949672977;
pub const KEYC_SECONDCLICK11_CONTROL6: C2RustUnnamed_38 = 42949675792;
pub const KEYC_SECONDCLICK10_CONTROL6: C2RustUnnamed_38 = 42949675536;
pub const KEYC_SECONDCLICK9_CONTROL6: C2RustUnnamed_38 = 42949675280;
pub const KEYC_SECONDCLICK8_CONTROL6: C2RustUnnamed_38 = 42949675024;
pub const KEYC_SECONDCLICK7_CONTROL6: C2RustUnnamed_38 = 42949674768;
pub const KEYC_SECONDCLICK6_CONTROL6: C2RustUnnamed_38 = 42949674512;
pub const KEYC_SECONDCLICK3_CONTROL6: C2RustUnnamed_38 = 42949673744;
pub const KEYC_SECONDCLICK2_CONTROL6: C2RustUnnamed_38 = 42949673488;
pub const KEYC_SECONDCLICK1_CONTROL6: C2RustUnnamed_38 = 42949673232;
pub const KEYC_SECONDCLICK_CONTROL6: C2RustUnnamed_38 = 42949672976;
pub const KEYC_SECONDCLICK11_CONTROL5: C2RustUnnamed_38 = 42949675791;
pub const KEYC_SECONDCLICK10_CONTROL5: C2RustUnnamed_38 = 42949675535;
pub const KEYC_SECONDCLICK9_CONTROL5: C2RustUnnamed_38 = 42949675279;
pub const KEYC_SECONDCLICK8_CONTROL5: C2RustUnnamed_38 = 42949675023;
pub const KEYC_SECONDCLICK7_CONTROL5: C2RustUnnamed_38 = 42949674767;
pub const KEYC_SECONDCLICK6_CONTROL5: C2RustUnnamed_38 = 42949674511;
pub const KEYC_SECONDCLICK3_CONTROL5: C2RustUnnamed_38 = 42949673743;
pub const KEYC_SECONDCLICK2_CONTROL5: C2RustUnnamed_38 = 42949673487;
pub const KEYC_SECONDCLICK1_CONTROL5: C2RustUnnamed_38 = 42949673231;
pub const KEYC_SECONDCLICK_CONTROL5: C2RustUnnamed_38 = 42949672975;
pub const KEYC_SECONDCLICK11_CONTROL4: C2RustUnnamed_38 = 42949675790;
pub const KEYC_SECONDCLICK10_CONTROL4: C2RustUnnamed_38 = 42949675534;
pub const KEYC_SECONDCLICK9_CONTROL4: C2RustUnnamed_38 = 42949675278;
pub const KEYC_SECONDCLICK8_CONTROL4: C2RustUnnamed_38 = 42949675022;
pub const KEYC_SECONDCLICK7_CONTROL4: C2RustUnnamed_38 = 42949674766;
pub const KEYC_SECONDCLICK6_CONTROL4: C2RustUnnamed_38 = 42949674510;
pub const KEYC_SECONDCLICK3_CONTROL4: C2RustUnnamed_38 = 42949673742;
pub const KEYC_SECONDCLICK2_CONTROL4: C2RustUnnamed_38 = 42949673486;
pub const KEYC_SECONDCLICK1_CONTROL4: C2RustUnnamed_38 = 42949673230;
pub const KEYC_SECONDCLICK_CONTROL4: C2RustUnnamed_38 = 42949672974;
pub const KEYC_SECONDCLICK11_CONTROL3: C2RustUnnamed_38 = 42949675789;
pub const KEYC_SECONDCLICK10_CONTROL3: C2RustUnnamed_38 = 42949675533;
pub const KEYC_SECONDCLICK9_CONTROL3: C2RustUnnamed_38 = 42949675277;
pub const KEYC_SECONDCLICK8_CONTROL3: C2RustUnnamed_38 = 42949675021;
pub const KEYC_SECONDCLICK7_CONTROL3: C2RustUnnamed_38 = 42949674765;
pub const KEYC_SECONDCLICK6_CONTROL3: C2RustUnnamed_38 = 42949674509;
pub const KEYC_SECONDCLICK3_CONTROL3: C2RustUnnamed_38 = 42949673741;
pub const KEYC_SECONDCLICK2_CONTROL3: C2RustUnnamed_38 = 42949673485;
pub const KEYC_SECONDCLICK1_CONTROL3: C2RustUnnamed_38 = 42949673229;
pub const KEYC_SECONDCLICK_CONTROL3: C2RustUnnamed_38 = 42949672973;
pub const KEYC_SECONDCLICK11_CONTROL2: C2RustUnnamed_38 = 42949675788;
pub const KEYC_SECONDCLICK10_CONTROL2: C2RustUnnamed_38 = 42949675532;
pub const KEYC_SECONDCLICK9_CONTROL2: C2RustUnnamed_38 = 42949675276;
pub const KEYC_SECONDCLICK8_CONTROL2: C2RustUnnamed_38 = 42949675020;
pub const KEYC_SECONDCLICK7_CONTROL2: C2RustUnnamed_38 = 42949674764;
pub const KEYC_SECONDCLICK6_CONTROL2: C2RustUnnamed_38 = 42949674508;
pub const KEYC_SECONDCLICK3_CONTROL2: C2RustUnnamed_38 = 42949673740;
pub const KEYC_SECONDCLICK2_CONTROL2: C2RustUnnamed_38 = 42949673484;
pub const KEYC_SECONDCLICK1_CONTROL2: C2RustUnnamed_38 = 42949673228;
pub const KEYC_SECONDCLICK_CONTROL2: C2RustUnnamed_38 = 42949672972;
pub const KEYC_SECONDCLICK11_CONTROL1: C2RustUnnamed_38 = 42949675787;
pub const KEYC_SECONDCLICK10_CONTROL1: C2RustUnnamed_38 = 42949675531;
pub const KEYC_SECONDCLICK9_CONTROL1: C2RustUnnamed_38 = 42949675275;
pub const KEYC_SECONDCLICK8_CONTROL1: C2RustUnnamed_38 = 42949675019;
pub const KEYC_SECONDCLICK7_CONTROL1: C2RustUnnamed_38 = 42949674763;
pub const KEYC_SECONDCLICK6_CONTROL1: C2RustUnnamed_38 = 42949674507;
pub const KEYC_SECONDCLICK3_CONTROL1: C2RustUnnamed_38 = 42949673739;
pub const KEYC_SECONDCLICK2_CONTROL1: C2RustUnnamed_38 = 42949673483;
pub const KEYC_SECONDCLICK1_CONTROL1: C2RustUnnamed_38 = 42949673227;
pub const KEYC_SECONDCLICK_CONTROL1: C2RustUnnamed_38 = 42949672971;
pub const KEYC_SECONDCLICK11_CONTROL0: C2RustUnnamed_38 = 42949675786;
pub const KEYC_SECONDCLICK10_CONTROL0: C2RustUnnamed_38 = 42949675530;
pub const KEYC_SECONDCLICK9_CONTROL0: C2RustUnnamed_38 = 42949675274;
pub const KEYC_SECONDCLICK8_CONTROL0: C2RustUnnamed_38 = 42949675018;
pub const KEYC_SECONDCLICK7_CONTROL0: C2RustUnnamed_38 = 42949674762;
pub const KEYC_SECONDCLICK6_CONTROL0: C2RustUnnamed_38 = 42949674506;
pub const KEYC_SECONDCLICK3_CONTROL0: C2RustUnnamed_38 = 42949673738;
pub const KEYC_SECONDCLICK2_CONTROL0: C2RustUnnamed_38 = 42949673482;
pub const KEYC_SECONDCLICK1_CONTROL0: C2RustUnnamed_38 = 42949673226;
pub const KEYC_SECONDCLICK_CONTROL0: C2RustUnnamed_38 = 42949672970;
pub const KEYC_SECONDCLICK11_EMPTY: C2RustUnnamed_38 = 42949675785;
pub const KEYC_SECONDCLICK10_EMPTY: C2RustUnnamed_38 = 42949675529;
pub const KEYC_SECONDCLICK9_EMPTY: C2RustUnnamed_38 = 42949675273;
pub const KEYC_SECONDCLICK8_EMPTY: C2RustUnnamed_38 = 42949675017;
pub const KEYC_SECONDCLICK7_EMPTY: C2RustUnnamed_38 = 42949674761;
pub const KEYC_SECONDCLICK6_EMPTY: C2RustUnnamed_38 = 42949674505;
pub const KEYC_SECONDCLICK3_EMPTY: C2RustUnnamed_38 = 42949673737;
pub const KEYC_SECONDCLICK2_EMPTY: C2RustUnnamed_38 = 42949673481;
pub const KEYC_SECONDCLICK1_EMPTY: C2RustUnnamed_38 = 42949673225;
pub const KEYC_SECONDCLICK_EMPTY: C2RustUnnamed_38 = 42949672969;
pub const KEYC_SECONDCLICK11_SCROLLBAR_DOWN: C2RustUnnamed_38 = 42949675784;
pub const KEYC_SECONDCLICK10_SCROLLBAR_DOWN: C2RustUnnamed_38 = 42949675528;
pub const KEYC_SECONDCLICK9_SCROLLBAR_DOWN: C2RustUnnamed_38 = 42949675272;
pub const KEYC_SECONDCLICK8_SCROLLBAR_DOWN: C2RustUnnamed_38 = 42949675016;
pub const KEYC_SECONDCLICK7_SCROLLBAR_DOWN: C2RustUnnamed_38 = 42949674760;
pub const KEYC_SECONDCLICK6_SCROLLBAR_DOWN: C2RustUnnamed_38 = 42949674504;
pub const KEYC_SECONDCLICK3_SCROLLBAR_DOWN: C2RustUnnamed_38 = 42949673736;
pub const KEYC_SECONDCLICK2_SCROLLBAR_DOWN: C2RustUnnamed_38 = 42949673480;
pub const KEYC_SECONDCLICK1_SCROLLBAR_DOWN: C2RustUnnamed_38 = 42949673224;
pub const KEYC_SECONDCLICK_SCROLLBAR_DOWN: C2RustUnnamed_38 = 42949672968;
pub const KEYC_SECONDCLICK11_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 42949675783;
pub const KEYC_SECONDCLICK10_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 42949675527;
pub const KEYC_SECONDCLICK9_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 42949675271;
pub const KEYC_SECONDCLICK8_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 42949675015;
pub const KEYC_SECONDCLICK7_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 42949674759;
pub const KEYC_SECONDCLICK6_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 42949674503;
pub const KEYC_SECONDCLICK3_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 42949673735;
pub const KEYC_SECONDCLICK2_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 42949673479;
pub const KEYC_SECONDCLICK1_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 42949673223;
pub const KEYC_SECONDCLICK_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 42949672967;
pub const KEYC_SECONDCLICK11_SCROLLBAR_UP: C2RustUnnamed_38 = 42949675782;
pub const KEYC_SECONDCLICK10_SCROLLBAR_UP: C2RustUnnamed_38 = 42949675526;
pub const KEYC_SECONDCLICK9_SCROLLBAR_UP: C2RustUnnamed_38 = 42949675270;
pub const KEYC_SECONDCLICK8_SCROLLBAR_UP: C2RustUnnamed_38 = 42949675014;
pub const KEYC_SECONDCLICK7_SCROLLBAR_UP: C2RustUnnamed_38 = 42949674758;
pub const KEYC_SECONDCLICK6_SCROLLBAR_UP: C2RustUnnamed_38 = 42949674502;
pub const KEYC_SECONDCLICK3_SCROLLBAR_UP: C2RustUnnamed_38 = 42949673734;
pub const KEYC_SECONDCLICK2_SCROLLBAR_UP: C2RustUnnamed_38 = 42949673478;
pub const KEYC_SECONDCLICK1_SCROLLBAR_UP: C2RustUnnamed_38 = 42949673222;
pub const KEYC_SECONDCLICK_SCROLLBAR_UP: C2RustUnnamed_38 = 42949672966;
pub const KEYC_SECONDCLICK11_BORDER: C2RustUnnamed_38 = 42949675781;
pub const KEYC_SECONDCLICK10_BORDER: C2RustUnnamed_38 = 42949675525;
pub const KEYC_SECONDCLICK9_BORDER: C2RustUnnamed_38 = 42949675269;
pub const KEYC_SECONDCLICK8_BORDER: C2RustUnnamed_38 = 42949675013;
pub const KEYC_SECONDCLICK7_BORDER: C2RustUnnamed_38 = 42949674757;
pub const KEYC_SECONDCLICK6_BORDER: C2RustUnnamed_38 = 42949674501;
pub const KEYC_SECONDCLICK3_BORDER: C2RustUnnamed_38 = 42949673733;
pub const KEYC_SECONDCLICK2_BORDER: C2RustUnnamed_38 = 42949673477;
pub const KEYC_SECONDCLICK1_BORDER: C2RustUnnamed_38 = 42949673221;
pub const KEYC_SECONDCLICK_BORDER: C2RustUnnamed_38 = 42949672965;
pub const KEYC_SECONDCLICK11_STATUS_DEFAULT: C2RustUnnamed_38 = 42949675780;
pub const KEYC_SECONDCLICK10_STATUS_DEFAULT: C2RustUnnamed_38 = 42949675524;
pub const KEYC_SECONDCLICK9_STATUS_DEFAULT: C2RustUnnamed_38 = 42949675268;
pub const KEYC_SECONDCLICK8_STATUS_DEFAULT: C2RustUnnamed_38 = 42949675012;
pub const KEYC_SECONDCLICK7_STATUS_DEFAULT: C2RustUnnamed_38 = 42949674756;
pub const KEYC_SECONDCLICK6_STATUS_DEFAULT: C2RustUnnamed_38 = 42949674500;
pub const KEYC_SECONDCLICK3_STATUS_DEFAULT: C2RustUnnamed_38 = 42949673732;
pub const KEYC_SECONDCLICK2_STATUS_DEFAULT: C2RustUnnamed_38 = 42949673476;
pub const KEYC_SECONDCLICK1_STATUS_DEFAULT: C2RustUnnamed_38 = 42949673220;
pub const KEYC_SECONDCLICK_STATUS_DEFAULT: C2RustUnnamed_38 = 42949672964;
pub const KEYC_SECONDCLICK11_STATUS_RIGHT: C2RustUnnamed_38 = 42949675779;
pub const KEYC_SECONDCLICK10_STATUS_RIGHT: C2RustUnnamed_38 = 42949675523;
pub const KEYC_SECONDCLICK9_STATUS_RIGHT: C2RustUnnamed_38 = 42949675267;
pub const KEYC_SECONDCLICK8_STATUS_RIGHT: C2RustUnnamed_38 = 42949675011;
pub const KEYC_SECONDCLICK7_STATUS_RIGHT: C2RustUnnamed_38 = 42949674755;
pub const KEYC_SECONDCLICK6_STATUS_RIGHT: C2RustUnnamed_38 = 42949674499;
pub const KEYC_SECONDCLICK3_STATUS_RIGHT: C2RustUnnamed_38 = 42949673731;
pub const KEYC_SECONDCLICK2_STATUS_RIGHT: C2RustUnnamed_38 = 42949673475;
pub const KEYC_SECONDCLICK1_STATUS_RIGHT: C2RustUnnamed_38 = 42949673219;
pub const KEYC_SECONDCLICK_STATUS_RIGHT: C2RustUnnamed_38 = 42949672963;
pub const KEYC_SECONDCLICK11_STATUS_LEFT: C2RustUnnamed_38 = 42949675778;
pub const KEYC_SECONDCLICK10_STATUS_LEFT: C2RustUnnamed_38 = 42949675522;
pub const KEYC_SECONDCLICK9_STATUS_LEFT: C2RustUnnamed_38 = 42949675266;
pub const KEYC_SECONDCLICK8_STATUS_LEFT: C2RustUnnamed_38 = 42949675010;
pub const KEYC_SECONDCLICK7_STATUS_LEFT: C2RustUnnamed_38 = 42949674754;
pub const KEYC_SECONDCLICK6_STATUS_LEFT: C2RustUnnamed_38 = 42949674498;
pub const KEYC_SECONDCLICK3_STATUS_LEFT: C2RustUnnamed_38 = 42949673730;
pub const KEYC_SECONDCLICK2_STATUS_LEFT: C2RustUnnamed_38 = 42949673474;
pub const KEYC_SECONDCLICK1_STATUS_LEFT: C2RustUnnamed_38 = 42949673218;
pub const KEYC_SECONDCLICK_STATUS_LEFT: C2RustUnnamed_38 = 42949672962;
pub const KEYC_SECONDCLICK11_STATUS: C2RustUnnamed_38 = 42949675777;
pub const KEYC_SECONDCLICK10_STATUS: C2RustUnnamed_38 = 42949675521;
pub const KEYC_SECONDCLICK9_STATUS: C2RustUnnamed_38 = 42949675265;
pub const KEYC_SECONDCLICK8_STATUS: C2RustUnnamed_38 = 42949675009;
pub const KEYC_SECONDCLICK7_STATUS: C2RustUnnamed_38 = 42949674753;
pub const KEYC_SECONDCLICK6_STATUS: C2RustUnnamed_38 = 42949674497;
pub const KEYC_SECONDCLICK3_STATUS: C2RustUnnamed_38 = 42949673729;
pub const KEYC_SECONDCLICK2_STATUS: C2RustUnnamed_38 = 42949673473;
pub const KEYC_SECONDCLICK1_STATUS: C2RustUnnamed_38 = 42949673217;
pub const KEYC_SECONDCLICK_STATUS: C2RustUnnamed_38 = 42949672961;
pub const KEYC_SECONDCLICK11_PANE: C2RustUnnamed_38 = 42949675776;
pub const KEYC_SECONDCLICK10_PANE: C2RustUnnamed_38 = 42949675520;
pub const KEYC_SECONDCLICK9_PANE: C2RustUnnamed_38 = 42949675264;
pub const KEYC_SECONDCLICK8_PANE: C2RustUnnamed_38 = 42949675008;
pub const KEYC_SECONDCLICK7_PANE: C2RustUnnamed_38 = 42949674752;
pub const KEYC_SECONDCLICK6_PANE: C2RustUnnamed_38 = 42949674496;
pub const KEYC_SECONDCLICK3_PANE: C2RustUnnamed_38 = 42949673728;
pub const KEYC_SECONDCLICK2_PANE: C2RustUnnamed_38 = 42949673472;
pub const KEYC_SECONDCLICK1_PANE: C2RustUnnamed_38 = 42949673216;
pub const KEYC_SECONDCLICK_PANE: C2RustUnnamed_38 = 42949672960;
pub const KEYC_MOUSEDRAGEND11_CONTROL9: C2RustUnnamed_38 = 30064773907;
pub const KEYC_MOUSEDRAGEND10_CONTROL9: C2RustUnnamed_38 = 30064773651;
pub const KEYC_MOUSEDRAGEND9_CONTROL9: C2RustUnnamed_38 = 30064773395;
pub const KEYC_MOUSEDRAGEND8_CONTROL9: C2RustUnnamed_38 = 30064773139;
pub const KEYC_MOUSEDRAGEND7_CONTROL9: C2RustUnnamed_38 = 30064772883;
pub const KEYC_MOUSEDRAGEND6_CONTROL9: C2RustUnnamed_38 = 30064772627;
pub const KEYC_MOUSEDRAGEND3_CONTROL9: C2RustUnnamed_38 = 30064771859;
pub const KEYC_MOUSEDRAGEND2_CONTROL9: C2RustUnnamed_38 = 30064771603;
pub const KEYC_MOUSEDRAGEND1_CONTROL9: C2RustUnnamed_38 = 30064771347;
pub const KEYC_MOUSEDRAGEND_CONTROL9: C2RustUnnamed_38 = 30064771091;
pub const KEYC_MOUSEDRAGEND11_CONTROL8: C2RustUnnamed_38 = 30064773906;
pub const KEYC_MOUSEDRAGEND10_CONTROL8: C2RustUnnamed_38 = 30064773650;
pub const KEYC_MOUSEDRAGEND9_CONTROL8: C2RustUnnamed_38 = 30064773394;
pub const KEYC_MOUSEDRAGEND8_CONTROL8: C2RustUnnamed_38 = 30064773138;
pub const KEYC_MOUSEDRAGEND7_CONTROL8: C2RustUnnamed_38 = 30064772882;
pub const KEYC_MOUSEDRAGEND6_CONTROL8: C2RustUnnamed_38 = 30064772626;
pub const KEYC_MOUSEDRAGEND3_CONTROL8: C2RustUnnamed_38 = 30064771858;
pub const KEYC_MOUSEDRAGEND2_CONTROL8: C2RustUnnamed_38 = 30064771602;
pub const KEYC_MOUSEDRAGEND1_CONTROL8: C2RustUnnamed_38 = 30064771346;
pub const KEYC_MOUSEDRAGEND_CONTROL8: C2RustUnnamed_38 = 30064771090;
pub const KEYC_MOUSEDRAGEND11_CONTROL7: C2RustUnnamed_38 = 30064773905;
pub const KEYC_MOUSEDRAGEND10_CONTROL7: C2RustUnnamed_38 = 30064773649;
pub const KEYC_MOUSEDRAGEND9_CONTROL7: C2RustUnnamed_38 = 30064773393;
pub const KEYC_MOUSEDRAGEND8_CONTROL7: C2RustUnnamed_38 = 30064773137;
pub const KEYC_MOUSEDRAGEND7_CONTROL7: C2RustUnnamed_38 = 30064772881;
pub const KEYC_MOUSEDRAGEND6_CONTROL7: C2RustUnnamed_38 = 30064772625;
pub const KEYC_MOUSEDRAGEND3_CONTROL7: C2RustUnnamed_38 = 30064771857;
pub const KEYC_MOUSEDRAGEND2_CONTROL7: C2RustUnnamed_38 = 30064771601;
pub const KEYC_MOUSEDRAGEND1_CONTROL7: C2RustUnnamed_38 = 30064771345;
pub const KEYC_MOUSEDRAGEND_CONTROL7: C2RustUnnamed_38 = 30064771089;
pub const KEYC_MOUSEDRAGEND11_CONTROL6: C2RustUnnamed_38 = 30064773904;
pub const KEYC_MOUSEDRAGEND10_CONTROL6: C2RustUnnamed_38 = 30064773648;
pub const KEYC_MOUSEDRAGEND9_CONTROL6: C2RustUnnamed_38 = 30064773392;
pub const KEYC_MOUSEDRAGEND8_CONTROL6: C2RustUnnamed_38 = 30064773136;
pub const KEYC_MOUSEDRAGEND7_CONTROL6: C2RustUnnamed_38 = 30064772880;
pub const KEYC_MOUSEDRAGEND6_CONTROL6: C2RustUnnamed_38 = 30064772624;
pub const KEYC_MOUSEDRAGEND3_CONTROL6: C2RustUnnamed_38 = 30064771856;
pub const KEYC_MOUSEDRAGEND2_CONTROL6: C2RustUnnamed_38 = 30064771600;
pub const KEYC_MOUSEDRAGEND1_CONTROL6: C2RustUnnamed_38 = 30064771344;
pub const KEYC_MOUSEDRAGEND_CONTROL6: C2RustUnnamed_38 = 30064771088;
pub const KEYC_MOUSEDRAGEND11_CONTROL5: C2RustUnnamed_38 = 30064773903;
pub const KEYC_MOUSEDRAGEND10_CONTROL5: C2RustUnnamed_38 = 30064773647;
pub const KEYC_MOUSEDRAGEND9_CONTROL5: C2RustUnnamed_38 = 30064773391;
pub const KEYC_MOUSEDRAGEND8_CONTROL5: C2RustUnnamed_38 = 30064773135;
pub const KEYC_MOUSEDRAGEND7_CONTROL5: C2RustUnnamed_38 = 30064772879;
pub const KEYC_MOUSEDRAGEND6_CONTROL5: C2RustUnnamed_38 = 30064772623;
pub const KEYC_MOUSEDRAGEND3_CONTROL5: C2RustUnnamed_38 = 30064771855;
pub const KEYC_MOUSEDRAGEND2_CONTROL5: C2RustUnnamed_38 = 30064771599;
pub const KEYC_MOUSEDRAGEND1_CONTROL5: C2RustUnnamed_38 = 30064771343;
pub const KEYC_MOUSEDRAGEND_CONTROL5: C2RustUnnamed_38 = 30064771087;
pub const KEYC_MOUSEDRAGEND11_CONTROL4: C2RustUnnamed_38 = 30064773902;
pub const KEYC_MOUSEDRAGEND10_CONTROL4: C2RustUnnamed_38 = 30064773646;
pub const KEYC_MOUSEDRAGEND9_CONTROL4: C2RustUnnamed_38 = 30064773390;
pub const KEYC_MOUSEDRAGEND8_CONTROL4: C2RustUnnamed_38 = 30064773134;
pub const KEYC_MOUSEDRAGEND7_CONTROL4: C2RustUnnamed_38 = 30064772878;
pub const KEYC_MOUSEDRAGEND6_CONTROL4: C2RustUnnamed_38 = 30064772622;
pub const KEYC_MOUSEDRAGEND3_CONTROL4: C2RustUnnamed_38 = 30064771854;
pub const KEYC_MOUSEDRAGEND2_CONTROL4: C2RustUnnamed_38 = 30064771598;
pub const KEYC_MOUSEDRAGEND1_CONTROL4: C2RustUnnamed_38 = 30064771342;
pub const KEYC_MOUSEDRAGEND_CONTROL4: C2RustUnnamed_38 = 30064771086;
pub const KEYC_MOUSEDRAGEND11_CONTROL3: C2RustUnnamed_38 = 30064773901;
pub const KEYC_MOUSEDRAGEND10_CONTROL3: C2RustUnnamed_38 = 30064773645;
pub const KEYC_MOUSEDRAGEND9_CONTROL3: C2RustUnnamed_38 = 30064773389;
pub const KEYC_MOUSEDRAGEND8_CONTROL3: C2RustUnnamed_38 = 30064773133;
pub const KEYC_MOUSEDRAGEND7_CONTROL3: C2RustUnnamed_38 = 30064772877;
pub const KEYC_MOUSEDRAGEND6_CONTROL3: C2RustUnnamed_38 = 30064772621;
pub const KEYC_MOUSEDRAGEND3_CONTROL3: C2RustUnnamed_38 = 30064771853;
pub const KEYC_MOUSEDRAGEND2_CONTROL3: C2RustUnnamed_38 = 30064771597;
pub const KEYC_MOUSEDRAGEND1_CONTROL3: C2RustUnnamed_38 = 30064771341;
pub const KEYC_MOUSEDRAGEND_CONTROL3: C2RustUnnamed_38 = 30064771085;
pub const KEYC_MOUSEDRAGEND11_CONTROL2: C2RustUnnamed_38 = 30064773900;
pub const KEYC_MOUSEDRAGEND10_CONTROL2: C2RustUnnamed_38 = 30064773644;
pub const KEYC_MOUSEDRAGEND9_CONTROL2: C2RustUnnamed_38 = 30064773388;
pub const KEYC_MOUSEDRAGEND8_CONTROL2: C2RustUnnamed_38 = 30064773132;
pub const KEYC_MOUSEDRAGEND7_CONTROL2: C2RustUnnamed_38 = 30064772876;
pub const KEYC_MOUSEDRAGEND6_CONTROL2: C2RustUnnamed_38 = 30064772620;
pub const KEYC_MOUSEDRAGEND3_CONTROL2: C2RustUnnamed_38 = 30064771852;
pub const KEYC_MOUSEDRAGEND2_CONTROL2: C2RustUnnamed_38 = 30064771596;
pub const KEYC_MOUSEDRAGEND1_CONTROL2: C2RustUnnamed_38 = 30064771340;
pub const KEYC_MOUSEDRAGEND_CONTROL2: C2RustUnnamed_38 = 30064771084;
pub const KEYC_MOUSEDRAGEND11_CONTROL1: C2RustUnnamed_38 = 30064773899;
pub const KEYC_MOUSEDRAGEND10_CONTROL1: C2RustUnnamed_38 = 30064773643;
pub const KEYC_MOUSEDRAGEND9_CONTROL1: C2RustUnnamed_38 = 30064773387;
pub const KEYC_MOUSEDRAGEND8_CONTROL1: C2RustUnnamed_38 = 30064773131;
pub const KEYC_MOUSEDRAGEND7_CONTROL1: C2RustUnnamed_38 = 30064772875;
pub const KEYC_MOUSEDRAGEND6_CONTROL1: C2RustUnnamed_38 = 30064772619;
pub const KEYC_MOUSEDRAGEND3_CONTROL1: C2RustUnnamed_38 = 30064771851;
pub const KEYC_MOUSEDRAGEND2_CONTROL1: C2RustUnnamed_38 = 30064771595;
pub const KEYC_MOUSEDRAGEND1_CONTROL1: C2RustUnnamed_38 = 30064771339;
pub const KEYC_MOUSEDRAGEND_CONTROL1: C2RustUnnamed_38 = 30064771083;
pub const KEYC_MOUSEDRAGEND11_CONTROL0: C2RustUnnamed_38 = 30064773898;
pub const KEYC_MOUSEDRAGEND10_CONTROL0: C2RustUnnamed_38 = 30064773642;
pub const KEYC_MOUSEDRAGEND9_CONTROL0: C2RustUnnamed_38 = 30064773386;
pub const KEYC_MOUSEDRAGEND8_CONTROL0: C2RustUnnamed_38 = 30064773130;
pub const KEYC_MOUSEDRAGEND7_CONTROL0: C2RustUnnamed_38 = 30064772874;
pub const KEYC_MOUSEDRAGEND6_CONTROL0: C2RustUnnamed_38 = 30064772618;
pub const KEYC_MOUSEDRAGEND3_CONTROL0: C2RustUnnamed_38 = 30064771850;
pub const KEYC_MOUSEDRAGEND2_CONTROL0: C2RustUnnamed_38 = 30064771594;
pub const KEYC_MOUSEDRAGEND1_CONTROL0: C2RustUnnamed_38 = 30064771338;
pub const KEYC_MOUSEDRAGEND_CONTROL0: C2RustUnnamed_38 = 30064771082;
pub const KEYC_MOUSEDRAGEND11_EMPTY: C2RustUnnamed_38 = 30064773897;
pub const KEYC_MOUSEDRAGEND10_EMPTY: C2RustUnnamed_38 = 30064773641;
pub const KEYC_MOUSEDRAGEND9_EMPTY: C2RustUnnamed_38 = 30064773385;
pub const KEYC_MOUSEDRAGEND8_EMPTY: C2RustUnnamed_38 = 30064773129;
pub const KEYC_MOUSEDRAGEND7_EMPTY: C2RustUnnamed_38 = 30064772873;
pub const KEYC_MOUSEDRAGEND6_EMPTY: C2RustUnnamed_38 = 30064772617;
pub const KEYC_MOUSEDRAGEND3_EMPTY: C2RustUnnamed_38 = 30064771849;
pub const KEYC_MOUSEDRAGEND2_EMPTY: C2RustUnnamed_38 = 30064771593;
pub const KEYC_MOUSEDRAGEND1_EMPTY: C2RustUnnamed_38 = 30064771337;
pub const KEYC_MOUSEDRAGEND_EMPTY: C2RustUnnamed_38 = 30064771081;
pub const KEYC_MOUSEDRAGEND11_SCROLLBAR_DOWN: C2RustUnnamed_38 = 30064773896;
pub const KEYC_MOUSEDRAGEND10_SCROLLBAR_DOWN: C2RustUnnamed_38 = 30064773640;
pub const KEYC_MOUSEDRAGEND9_SCROLLBAR_DOWN: C2RustUnnamed_38 = 30064773384;
pub const KEYC_MOUSEDRAGEND8_SCROLLBAR_DOWN: C2RustUnnamed_38 = 30064773128;
pub const KEYC_MOUSEDRAGEND7_SCROLLBAR_DOWN: C2RustUnnamed_38 = 30064772872;
pub const KEYC_MOUSEDRAGEND6_SCROLLBAR_DOWN: C2RustUnnamed_38 = 30064772616;
pub const KEYC_MOUSEDRAGEND3_SCROLLBAR_DOWN: C2RustUnnamed_38 = 30064771848;
pub const KEYC_MOUSEDRAGEND2_SCROLLBAR_DOWN: C2RustUnnamed_38 = 30064771592;
pub const KEYC_MOUSEDRAGEND1_SCROLLBAR_DOWN: C2RustUnnamed_38 = 30064771336;
pub const KEYC_MOUSEDRAGEND_SCROLLBAR_DOWN: C2RustUnnamed_38 = 30064771080;
pub const KEYC_MOUSEDRAGEND11_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 30064773895;
pub const KEYC_MOUSEDRAGEND10_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 30064773639;
pub const KEYC_MOUSEDRAGEND9_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 30064773383;
pub const KEYC_MOUSEDRAGEND8_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 30064773127;
pub const KEYC_MOUSEDRAGEND7_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 30064772871;
pub const KEYC_MOUSEDRAGEND6_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 30064772615;
pub const KEYC_MOUSEDRAGEND3_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 30064771847;
pub const KEYC_MOUSEDRAGEND2_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 30064771591;
pub const KEYC_MOUSEDRAGEND1_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 30064771335;
pub const KEYC_MOUSEDRAGEND_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 30064771079;
pub const KEYC_MOUSEDRAGEND11_SCROLLBAR_UP: C2RustUnnamed_38 = 30064773894;
pub const KEYC_MOUSEDRAGEND10_SCROLLBAR_UP: C2RustUnnamed_38 = 30064773638;
pub const KEYC_MOUSEDRAGEND9_SCROLLBAR_UP: C2RustUnnamed_38 = 30064773382;
pub const KEYC_MOUSEDRAGEND8_SCROLLBAR_UP: C2RustUnnamed_38 = 30064773126;
pub const KEYC_MOUSEDRAGEND7_SCROLLBAR_UP: C2RustUnnamed_38 = 30064772870;
pub const KEYC_MOUSEDRAGEND6_SCROLLBAR_UP: C2RustUnnamed_38 = 30064772614;
pub const KEYC_MOUSEDRAGEND3_SCROLLBAR_UP: C2RustUnnamed_38 = 30064771846;
pub const KEYC_MOUSEDRAGEND2_SCROLLBAR_UP: C2RustUnnamed_38 = 30064771590;
pub const KEYC_MOUSEDRAGEND1_SCROLLBAR_UP: C2RustUnnamed_38 = 30064771334;
pub const KEYC_MOUSEDRAGEND_SCROLLBAR_UP: C2RustUnnamed_38 = 30064771078;
pub const KEYC_MOUSEDRAGEND11_BORDER: C2RustUnnamed_38 = 30064773893;
pub const KEYC_MOUSEDRAGEND10_BORDER: C2RustUnnamed_38 = 30064773637;
pub const KEYC_MOUSEDRAGEND9_BORDER: C2RustUnnamed_38 = 30064773381;
pub const KEYC_MOUSEDRAGEND8_BORDER: C2RustUnnamed_38 = 30064773125;
pub const KEYC_MOUSEDRAGEND7_BORDER: C2RustUnnamed_38 = 30064772869;
pub const KEYC_MOUSEDRAGEND6_BORDER: C2RustUnnamed_38 = 30064772613;
pub const KEYC_MOUSEDRAGEND3_BORDER: C2RustUnnamed_38 = 30064771845;
pub const KEYC_MOUSEDRAGEND2_BORDER: C2RustUnnamed_38 = 30064771589;
pub const KEYC_MOUSEDRAGEND1_BORDER: C2RustUnnamed_38 = 30064771333;
pub const KEYC_MOUSEDRAGEND_BORDER: C2RustUnnamed_38 = 30064771077;
pub const KEYC_MOUSEDRAGEND11_STATUS_DEFAULT: C2RustUnnamed_38 = 30064773892;
pub const KEYC_MOUSEDRAGEND10_STATUS_DEFAULT: C2RustUnnamed_38 = 30064773636;
pub const KEYC_MOUSEDRAGEND9_STATUS_DEFAULT: C2RustUnnamed_38 = 30064773380;
pub const KEYC_MOUSEDRAGEND8_STATUS_DEFAULT: C2RustUnnamed_38 = 30064773124;
pub const KEYC_MOUSEDRAGEND7_STATUS_DEFAULT: C2RustUnnamed_38 = 30064772868;
pub const KEYC_MOUSEDRAGEND6_STATUS_DEFAULT: C2RustUnnamed_38 = 30064772612;
pub const KEYC_MOUSEDRAGEND3_STATUS_DEFAULT: C2RustUnnamed_38 = 30064771844;
pub const KEYC_MOUSEDRAGEND2_STATUS_DEFAULT: C2RustUnnamed_38 = 30064771588;
pub const KEYC_MOUSEDRAGEND1_STATUS_DEFAULT: C2RustUnnamed_38 = 30064771332;
pub const KEYC_MOUSEDRAGEND_STATUS_DEFAULT: C2RustUnnamed_38 = 30064771076;
pub const KEYC_MOUSEDRAGEND11_STATUS_RIGHT: C2RustUnnamed_38 = 30064773891;
pub const KEYC_MOUSEDRAGEND10_STATUS_RIGHT: C2RustUnnamed_38 = 30064773635;
pub const KEYC_MOUSEDRAGEND9_STATUS_RIGHT: C2RustUnnamed_38 = 30064773379;
pub const KEYC_MOUSEDRAGEND8_STATUS_RIGHT: C2RustUnnamed_38 = 30064773123;
pub const KEYC_MOUSEDRAGEND7_STATUS_RIGHT: C2RustUnnamed_38 = 30064772867;
pub const KEYC_MOUSEDRAGEND6_STATUS_RIGHT: C2RustUnnamed_38 = 30064772611;
pub const KEYC_MOUSEDRAGEND3_STATUS_RIGHT: C2RustUnnamed_38 = 30064771843;
pub const KEYC_MOUSEDRAGEND2_STATUS_RIGHT: C2RustUnnamed_38 = 30064771587;
pub const KEYC_MOUSEDRAGEND1_STATUS_RIGHT: C2RustUnnamed_38 = 30064771331;
pub const KEYC_MOUSEDRAGEND_STATUS_RIGHT: C2RustUnnamed_38 = 30064771075;
pub const KEYC_MOUSEDRAGEND11_STATUS_LEFT: C2RustUnnamed_38 = 30064773890;
pub const KEYC_MOUSEDRAGEND10_STATUS_LEFT: C2RustUnnamed_38 = 30064773634;
pub const KEYC_MOUSEDRAGEND9_STATUS_LEFT: C2RustUnnamed_38 = 30064773378;
pub const KEYC_MOUSEDRAGEND8_STATUS_LEFT: C2RustUnnamed_38 = 30064773122;
pub const KEYC_MOUSEDRAGEND7_STATUS_LEFT: C2RustUnnamed_38 = 30064772866;
pub const KEYC_MOUSEDRAGEND6_STATUS_LEFT: C2RustUnnamed_38 = 30064772610;
pub const KEYC_MOUSEDRAGEND3_STATUS_LEFT: C2RustUnnamed_38 = 30064771842;
pub const KEYC_MOUSEDRAGEND2_STATUS_LEFT: C2RustUnnamed_38 = 30064771586;
pub const KEYC_MOUSEDRAGEND1_STATUS_LEFT: C2RustUnnamed_38 = 30064771330;
pub const KEYC_MOUSEDRAGEND_STATUS_LEFT: C2RustUnnamed_38 = 30064771074;
pub const KEYC_MOUSEDRAGEND11_STATUS: C2RustUnnamed_38 = 30064773889;
pub const KEYC_MOUSEDRAGEND10_STATUS: C2RustUnnamed_38 = 30064773633;
pub const KEYC_MOUSEDRAGEND9_STATUS: C2RustUnnamed_38 = 30064773377;
pub const KEYC_MOUSEDRAGEND8_STATUS: C2RustUnnamed_38 = 30064773121;
pub const KEYC_MOUSEDRAGEND7_STATUS: C2RustUnnamed_38 = 30064772865;
pub const KEYC_MOUSEDRAGEND6_STATUS: C2RustUnnamed_38 = 30064772609;
pub const KEYC_MOUSEDRAGEND3_STATUS: C2RustUnnamed_38 = 30064771841;
pub const KEYC_MOUSEDRAGEND2_STATUS: C2RustUnnamed_38 = 30064771585;
pub const KEYC_MOUSEDRAGEND1_STATUS: C2RustUnnamed_38 = 30064771329;
pub const KEYC_MOUSEDRAGEND_STATUS: C2RustUnnamed_38 = 30064771073;
pub const KEYC_MOUSEDRAGEND11_PANE: C2RustUnnamed_38 = 30064773888;
pub const KEYC_MOUSEDRAGEND10_PANE: C2RustUnnamed_38 = 30064773632;
pub const KEYC_MOUSEDRAGEND9_PANE: C2RustUnnamed_38 = 30064773376;
pub const KEYC_MOUSEDRAGEND8_PANE: C2RustUnnamed_38 = 30064773120;
pub const KEYC_MOUSEDRAGEND7_PANE: C2RustUnnamed_38 = 30064772864;
pub const KEYC_MOUSEDRAGEND6_PANE: C2RustUnnamed_38 = 30064772608;
pub const KEYC_MOUSEDRAGEND3_PANE: C2RustUnnamed_38 = 30064771840;
pub const KEYC_MOUSEDRAGEND2_PANE: C2RustUnnamed_38 = 30064771584;
pub const KEYC_MOUSEDRAGEND1_PANE: C2RustUnnamed_38 = 30064771328;
pub const KEYC_MOUSEDRAGEND_PANE: C2RustUnnamed_38 = 30064771072;
pub const KEYC_MOUSEDRAG11_CONTROL9: C2RustUnnamed_38 = 25769806611;
pub const KEYC_MOUSEDRAG10_CONTROL9: C2RustUnnamed_38 = 25769806355;
pub const KEYC_MOUSEDRAG9_CONTROL9: C2RustUnnamed_38 = 25769806099;
pub const KEYC_MOUSEDRAG8_CONTROL9: C2RustUnnamed_38 = 25769805843;
pub const KEYC_MOUSEDRAG7_CONTROL9: C2RustUnnamed_38 = 25769805587;
pub const KEYC_MOUSEDRAG6_CONTROL9: C2RustUnnamed_38 = 25769805331;
pub const KEYC_MOUSEDRAG3_CONTROL9: C2RustUnnamed_38 = 25769804563;
pub const KEYC_MOUSEDRAG2_CONTROL9: C2RustUnnamed_38 = 25769804307;
pub const KEYC_MOUSEDRAG1_CONTROL9: C2RustUnnamed_38 = 25769804051;
pub const KEYC_MOUSEDRAG_CONTROL9: C2RustUnnamed_38 = 25769803795;
pub const KEYC_MOUSEDRAG11_CONTROL8: C2RustUnnamed_38 = 25769806610;
pub const KEYC_MOUSEDRAG10_CONTROL8: C2RustUnnamed_38 = 25769806354;
pub const KEYC_MOUSEDRAG9_CONTROL8: C2RustUnnamed_38 = 25769806098;
pub const KEYC_MOUSEDRAG8_CONTROL8: C2RustUnnamed_38 = 25769805842;
pub const KEYC_MOUSEDRAG7_CONTROL8: C2RustUnnamed_38 = 25769805586;
pub const KEYC_MOUSEDRAG6_CONTROL8: C2RustUnnamed_38 = 25769805330;
pub const KEYC_MOUSEDRAG3_CONTROL8: C2RustUnnamed_38 = 25769804562;
pub const KEYC_MOUSEDRAG2_CONTROL8: C2RustUnnamed_38 = 25769804306;
pub const KEYC_MOUSEDRAG1_CONTROL8: C2RustUnnamed_38 = 25769804050;
pub const KEYC_MOUSEDRAG_CONTROL8: C2RustUnnamed_38 = 25769803794;
pub const KEYC_MOUSEDRAG11_CONTROL7: C2RustUnnamed_38 = 25769806609;
pub const KEYC_MOUSEDRAG10_CONTROL7: C2RustUnnamed_38 = 25769806353;
pub const KEYC_MOUSEDRAG9_CONTROL7: C2RustUnnamed_38 = 25769806097;
pub const KEYC_MOUSEDRAG8_CONTROL7: C2RustUnnamed_38 = 25769805841;
pub const KEYC_MOUSEDRAG7_CONTROL7: C2RustUnnamed_38 = 25769805585;
pub const KEYC_MOUSEDRAG6_CONTROL7: C2RustUnnamed_38 = 25769805329;
pub const KEYC_MOUSEDRAG3_CONTROL7: C2RustUnnamed_38 = 25769804561;
pub const KEYC_MOUSEDRAG2_CONTROL7: C2RustUnnamed_38 = 25769804305;
pub const KEYC_MOUSEDRAG1_CONTROL7: C2RustUnnamed_38 = 25769804049;
pub const KEYC_MOUSEDRAG_CONTROL7: C2RustUnnamed_38 = 25769803793;
pub const KEYC_MOUSEDRAG11_CONTROL6: C2RustUnnamed_38 = 25769806608;
pub const KEYC_MOUSEDRAG10_CONTROL6: C2RustUnnamed_38 = 25769806352;
pub const KEYC_MOUSEDRAG9_CONTROL6: C2RustUnnamed_38 = 25769806096;
pub const KEYC_MOUSEDRAG8_CONTROL6: C2RustUnnamed_38 = 25769805840;
pub const KEYC_MOUSEDRAG7_CONTROL6: C2RustUnnamed_38 = 25769805584;
pub const KEYC_MOUSEDRAG6_CONTROL6: C2RustUnnamed_38 = 25769805328;
pub const KEYC_MOUSEDRAG3_CONTROL6: C2RustUnnamed_38 = 25769804560;
pub const KEYC_MOUSEDRAG2_CONTROL6: C2RustUnnamed_38 = 25769804304;
pub const KEYC_MOUSEDRAG1_CONTROL6: C2RustUnnamed_38 = 25769804048;
pub const KEYC_MOUSEDRAG_CONTROL6: C2RustUnnamed_38 = 25769803792;
pub const KEYC_MOUSEDRAG11_CONTROL5: C2RustUnnamed_38 = 25769806607;
pub const KEYC_MOUSEDRAG10_CONTROL5: C2RustUnnamed_38 = 25769806351;
pub const KEYC_MOUSEDRAG9_CONTROL5: C2RustUnnamed_38 = 25769806095;
pub const KEYC_MOUSEDRAG8_CONTROL5: C2RustUnnamed_38 = 25769805839;
pub const KEYC_MOUSEDRAG7_CONTROL5: C2RustUnnamed_38 = 25769805583;
pub const KEYC_MOUSEDRAG6_CONTROL5: C2RustUnnamed_38 = 25769805327;
pub const KEYC_MOUSEDRAG3_CONTROL5: C2RustUnnamed_38 = 25769804559;
pub const KEYC_MOUSEDRAG2_CONTROL5: C2RustUnnamed_38 = 25769804303;
pub const KEYC_MOUSEDRAG1_CONTROL5: C2RustUnnamed_38 = 25769804047;
pub const KEYC_MOUSEDRAG_CONTROL5: C2RustUnnamed_38 = 25769803791;
pub const KEYC_MOUSEDRAG11_CONTROL4: C2RustUnnamed_38 = 25769806606;
pub const KEYC_MOUSEDRAG10_CONTROL4: C2RustUnnamed_38 = 25769806350;
pub const KEYC_MOUSEDRAG9_CONTROL4: C2RustUnnamed_38 = 25769806094;
pub const KEYC_MOUSEDRAG8_CONTROL4: C2RustUnnamed_38 = 25769805838;
pub const KEYC_MOUSEDRAG7_CONTROL4: C2RustUnnamed_38 = 25769805582;
pub const KEYC_MOUSEDRAG6_CONTROL4: C2RustUnnamed_38 = 25769805326;
pub const KEYC_MOUSEDRAG3_CONTROL4: C2RustUnnamed_38 = 25769804558;
pub const KEYC_MOUSEDRAG2_CONTROL4: C2RustUnnamed_38 = 25769804302;
pub const KEYC_MOUSEDRAG1_CONTROL4: C2RustUnnamed_38 = 25769804046;
pub const KEYC_MOUSEDRAG_CONTROL4: C2RustUnnamed_38 = 25769803790;
pub const KEYC_MOUSEDRAG11_CONTROL3: C2RustUnnamed_38 = 25769806605;
pub const KEYC_MOUSEDRAG10_CONTROL3: C2RustUnnamed_38 = 25769806349;
pub const KEYC_MOUSEDRAG9_CONTROL3: C2RustUnnamed_38 = 25769806093;
pub const KEYC_MOUSEDRAG8_CONTROL3: C2RustUnnamed_38 = 25769805837;
pub const KEYC_MOUSEDRAG7_CONTROL3: C2RustUnnamed_38 = 25769805581;
pub const KEYC_MOUSEDRAG6_CONTROL3: C2RustUnnamed_38 = 25769805325;
pub const KEYC_MOUSEDRAG3_CONTROL3: C2RustUnnamed_38 = 25769804557;
pub const KEYC_MOUSEDRAG2_CONTROL3: C2RustUnnamed_38 = 25769804301;
pub const KEYC_MOUSEDRAG1_CONTROL3: C2RustUnnamed_38 = 25769804045;
pub const KEYC_MOUSEDRAG_CONTROL3: C2RustUnnamed_38 = 25769803789;
pub const KEYC_MOUSEDRAG11_CONTROL2: C2RustUnnamed_38 = 25769806604;
pub const KEYC_MOUSEDRAG10_CONTROL2: C2RustUnnamed_38 = 25769806348;
pub const KEYC_MOUSEDRAG9_CONTROL2: C2RustUnnamed_38 = 25769806092;
pub const KEYC_MOUSEDRAG8_CONTROL2: C2RustUnnamed_38 = 25769805836;
pub const KEYC_MOUSEDRAG7_CONTROL2: C2RustUnnamed_38 = 25769805580;
pub const KEYC_MOUSEDRAG6_CONTROL2: C2RustUnnamed_38 = 25769805324;
pub const KEYC_MOUSEDRAG3_CONTROL2: C2RustUnnamed_38 = 25769804556;
pub const KEYC_MOUSEDRAG2_CONTROL2: C2RustUnnamed_38 = 25769804300;
pub const KEYC_MOUSEDRAG1_CONTROL2: C2RustUnnamed_38 = 25769804044;
pub const KEYC_MOUSEDRAG_CONTROL2: C2RustUnnamed_38 = 25769803788;
pub const KEYC_MOUSEDRAG11_CONTROL1: C2RustUnnamed_38 = 25769806603;
pub const KEYC_MOUSEDRAG10_CONTROL1: C2RustUnnamed_38 = 25769806347;
pub const KEYC_MOUSEDRAG9_CONTROL1: C2RustUnnamed_38 = 25769806091;
pub const KEYC_MOUSEDRAG8_CONTROL1: C2RustUnnamed_38 = 25769805835;
pub const KEYC_MOUSEDRAG7_CONTROL1: C2RustUnnamed_38 = 25769805579;
pub const KEYC_MOUSEDRAG6_CONTROL1: C2RustUnnamed_38 = 25769805323;
pub const KEYC_MOUSEDRAG3_CONTROL1: C2RustUnnamed_38 = 25769804555;
pub const KEYC_MOUSEDRAG2_CONTROL1: C2RustUnnamed_38 = 25769804299;
pub const KEYC_MOUSEDRAG1_CONTROL1: C2RustUnnamed_38 = 25769804043;
pub const KEYC_MOUSEDRAG_CONTROL1: C2RustUnnamed_38 = 25769803787;
pub const KEYC_MOUSEDRAG11_CONTROL0: C2RustUnnamed_38 = 25769806602;
pub const KEYC_MOUSEDRAG10_CONTROL0: C2RustUnnamed_38 = 25769806346;
pub const KEYC_MOUSEDRAG9_CONTROL0: C2RustUnnamed_38 = 25769806090;
pub const KEYC_MOUSEDRAG8_CONTROL0: C2RustUnnamed_38 = 25769805834;
pub const KEYC_MOUSEDRAG7_CONTROL0: C2RustUnnamed_38 = 25769805578;
pub const KEYC_MOUSEDRAG6_CONTROL0: C2RustUnnamed_38 = 25769805322;
pub const KEYC_MOUSEDRAG3_CONTROL0: C2RustUnnamed_38 = 25769804554;
pub const KEYC_MOUSEDRAG2_CONTROL0: C2RustUnnamed_38 = 25769804298;
pub const KEYC_MOUSEDRAG1_CONTROL0: C2RustUnnamed_38 = 25769804042;
pub const KEYC_MOUSEDRAG_CONTROL0: C2RustUnnamed_38 = 25769803786;
pub const KEYC_MOUSEDRAG11_EMPTY: C2RustUnnamed_38 = 25769806601;
pub const KEYC_MOUSEDRAG10_EMPTY: C2RustUnnamed_38 = 25769806345;
pub const KEYC_MOUSEDRAG9_EMPTY: C2RustUnnamed_38 = 25769806089;
pub const KEYC_MOUSEDRAG8_EMPTY: C2RustUnnamed_38 = 25769805833;
pub const KEYC_MOUSEDRAG7_EMPTY: C2RustUnnamed_38 = 25769805577;
pub const KEYC_MOUSEDRAG6_EMPTY: C2RustUnnamed_38 = 25769805321;
pub const KEYC_MOUSEDRAG3_EMPTY: C2RustUnnamed_38 = 25769804553;
pub const KEYC_MOUSEDRAG2_EMPTY: C2RustUnnamed_38 = 25769804297;
pub const KEYC_MOUSEDRAG1_EMPTY: C2RustUnnamed_38 = 25769804041;
pub const KEYC_MOUSEDRAG_EMPTY: C2RustUnnamed_38 = 25769803785;
pub const KEYC_MOUSEDRAG11_SCROLLBAR_DOWN: C2RustUnnamed_38 = 25769806600;
pub const KEYC_MOUSEDRAG10_SCROLLBAR_DOWN: C2RustUnnamed_38 = 25769806344;
pub const KEYC_MOUSEDRAG9_SCROLLBAR_DOWN: C2RustUnnamed_38 = 25769806088;
pub const KEYC_MOUSEDRAG8_SCROLLBAR_DOWN: C2RustUnnamed_38 = 25769805832;
pub const KEYC_MOUSEDRAG7_SCROLLBAR_DOWN: C2RustUnnamed_38 = 25769805576;
pub const KEYC_MOUSEDRAG6_SCROLLBAR_DOWN: C2RustUnnamed_38 = 25769805320;
pub const KEYC_MOUSEDRAG3_SCROLLBAR_DOWN: C2RustUnnamed_38 = 25769804552;
pub const KEYC_MOUSEDRAG2_SCROLLBAR_DOWN: C2RustUnnamed_38 = 25769804296;
pub const KEYC_MOUSEDRAG1_SCROLLBAR_DOWN: C2RustUnnamed_38 = 25769804040;
pub const KEYC_MOUSEDRAG_SCROLLBAR_DOWN: C2RustUnnamed_38 = 25769803784;
pub const KEYC_MOUSEDRAG11_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 25769806599;
pub const KEYC_MOUSEDRAG10_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 25769806343;
pub const KEYC_MOUSEDRAG9_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 25769806087;
pub const KEYC_MOUSEDRAG8_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 25769805831;
pub const KEYC_MOUSEDRAG7_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 25769805575;
pub const KEYC_MOUSEDRAG6_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 25769805319;
pub const KEYC_MOUSEDRAG3_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 25769804551;
pub const KEYC_MOUSEDRAG2_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 25769804295;
pub const KEYC_MOUSEDRAG1_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 25769804039;
pub const KEYC_MOUSEDRAG_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 25769803783;
pub const KEYC_MOUSEDRAG11_SCROLLBAR_UP: C2RustUnnamed_38 = 25769806598;
pub const KEYC_MOUSEDRAG10_SCROLLBAR_UP: C2RustUnnamed_38 = 25769806342;
pub const KEYC_MOUSEDRAG9_SCROLLBAR_UP: C2RustUnnamed_38 = 25769806086;
pub const KEYC_MOUSEDRAG8_SCROLLBAR_UP: C2RustUnnamed_38 = 25769805830;
pub const KEYC_MOUSEDRAG7_SCROLLBAR_UP: C2RustUnnamed_38 = 25769805574;
pub const KEYC_MOUSEDRAG6_SCROLLBAR_UP: C2RustUnnamed_38 = 25769805318;
pub const KEYC_MOUSEDRAG3_SCROLLBAR_UP: C2RustUnnamed_38 = 25769804550;
pub const KEYC_MOUSEDRAG2_SCROLLBAR_UP: C2RustUnnamed_38 = 25769804294;
pub const KEYC_MOUSEDRAG1_SCROLLBAR_UP: C2RustUnnamed_38 = 25769804038;
pub const KEYC_MOUSEDRAG_SCROLLBAR_UP: C2RustUnnamed_38 = 25769803782;
pub const KEYC_MOUSEDRAG11_BORDER: C2RustUnnamed_38 = 25769806597;
pub const KEYC_MOUSEDRAG10_BORDER: C2RustUnnamed_38 = 25769806341;
pub const KEYC_MOUSEDRAG9_BORDER: C2RustUnnamed_38 = 25769806085;
pub const KEYC_MOUSEDRAG8_BORDER: C2RustUnnamed_38 = 25769805829;
pub const KEYC_MOUSEDRAG7_BORDER: C2RustUnnamed_38 = 25769805573;
pub const KEYC_MOUSEDRAG6_BORDER: C2RustUnnamed_38 = 25769805317;
pub const KEYC_MOUSEDRAG3_BORDER: C2RustUnnamed_38 = 25769804549;
pub const KEYC_MOUSEDRAG2_BORDER: C2RustUnnamed_38 = 25769804293;
pub const KEYC_MOUSEDRAG1_BORDER: C2RustUnnamed_38 = 25769804037;
pub const KEYC_MOUSEDRAG_BORDER: C2RustUnnamed_38 = 25769803781;
pub const KEYC_MOUSEDRAG11_STATUS_DEFAULT: C2RustUnnamed_38 = 25769806596;
pub const KEYC_MOUSEDRAG10_STATUS_DEFAULT: C2RustUnnamed_38 = 25769806340;
pub const KEYC_MOUSEDRAG9_STATUS_DEFAULT: C2RustUnnamed_38 = 25769806084;
pub const KEYC_MOUSEDRAG8_STATUS_DEFAULT: C2RustUnnamed_38 = 25769805828;
pub const KEYC_MOUSEDRAG7_STATUS_DEFAULT: C2RustUnnamed_38 = 25769805572;
pub const KEYC_MOUSEDRAG6_STATUS_DEFAULT: C2RustUnnamed_38 = 25769805316;
pub const KEYC_MOUSEDRAG3_STATUS_DEFAULT: C2RustUnnamed_38 = 25769804548;
pub const KEYC_MOUSEDRAG2_STATUS_DEFAULT: C2RustUnnamed_38 = 25769804292;
pub const KEYC_MOUSEDRAG1_STATUS_DEFAULT: C2RustUnnamed_38 = 25769804036;
pub const KEYC_MOUSEDRAG_STATUS_DEFAULT: C2RustUnnamed_38 = 25769803780;
pub const KEYC_MOUSEDRAG11_STATUS_RIGHT: C2RustUnnamed_38 = 25769806595;
pub const KEYC_MOUSEDRAG10_STATUS_RIGHT: C2RustUnnamed_38 = 25769806339;
pub const KEYC_MOUSEDRAG9_STATUS_RIGHT: C2RustUnnamed_38 = 25769806083;
pub const KEYC_MOUSEDRAG8_STATUS_RIGHT: C2RustUnnamed_38 = 25769805827;
pub const KEYC_MOUSEDRAG7_STATUS_RIGHT: C2RustUnnamed_38 = 25769805571;
pub const KEYC_MOUSEDRAG6_STATUS_RIGHT: C2RustUnnamed_38 = 25769805315;
pub const KEYC_MOUSEDRAG3_STATUS_RIGHT: C2RustUnnamed_38 = 25769804547;
pub const KEYC_MOUSEDRAG2_STATUS_RIGHT: C2RustUnnamed_38 = 25769804291;
pub const KEYC_MOUSEDRAG1_STATUS_RIGHT: C2RustUnnamed_38 = 25769804035;
pub const KEYC_MOUSEDRAG_STATUS_RIGHT: C2RustUnnamed_38 = 25769803779;
pub const KEYC_MOUSEDRAG11_STATUS_LEFT: C2RustUnnamed_38 = 25769806594;
pub const KEYC_MOUSEDRAG10_STATUS_LEFT: C2RustUnnamed_38 = 25769806338;
pub const KEYC_MOUSEDRAG9_STATUS_LEFT: C2RustUnnamed_38 = 25769806082;
pub const KEYC_MOUSEDRAG8_STATUS_LEFT: C2RustUnnamed_38 = 25769805826;
pub const KEYC_MOUSEDRAG7_STATUS_LEFT: C2RustUnnamed_38 = 25769805570;
pub const KEYC_MOUSEDRAG6_STATUS_LEFT: C2RustUnnamed_38 = 25769805314;
pub const KEYC_MOUSEDRAG3_STATUS_LEFT: C2RustUnnamed_38 = 25769804546;
pub const KEYC_MOUSEDRAG2_STATUS_LEFT: C2RustUnnamed_38 = 25769804290;
pub const KEYC_MOUSEDRAG1_STATUS_LEFT: C2RustUnnamed_38 = 25769804034;
pub const KEYC_MOUSEDRAG_STATUS_LEFT: C2RustUnnamed_38 = 25769803778;
pub const KEYC_MOUSEDRAG11_STATUS: C2RustUnnamed_38 = 25769806593;
pub const KEYC_MOUSEDRAG10_STATUS: C2RustUnnamed_38 = 25769806337;
pub const KEYC_MOUSEDRAG9_STATUS: C2RustUnnamed_38 = 25769806081;
pub const KEYC_MOUSEDRAG8_STATUS: C2RustUnnamed_38 = 25769805825;
pub const KEYC_MOUSEDRAG7_STATUS: C2RustUnnamed_38 = 25769805569;
pub const KEYC_MOUSEDRAG6_STATUS: C2RustUnnamed_38 = 25769805313;
pub const KEYC_MOUSEDRAG3_STATUS: C2RustUnnamed_38 = 25769804545;
pub const KEYC_MOUSEDRAG2_STATUS: C2RustUnnamed_38 = 25769804289;
pub const KEYC_MOUSEDRAG1_STATUS: C2RustUnnamed_38 = 25769804033;
pub const KEYC_MOUSEDRAG_STATUS: C2RustUnnamed_38 = 25769803777;
pub const KEYC_MOUSEDRAG11_PANE: C2RustUnnamed_38 = 25769806592;
pub const KEYC_MOUSEDRAG10_PANE: C2RustUnnamed_38 = 25769806336;
pub const KEYC_MOUSEDRAG9_PANE: C2RustUnnamed_38 = 25769806080;
pub const KEYC_MOUSEDRAG8_PANE: C2RustUnnamed_38 = 25769805824;
pub const KEYC_MOUSEDRAG7_PANE: C2RustUnnamed_38 = 25769805568;
pub const KEYC_MOUSEDRAG6_PANE: C2RustUnnamed_38 = 25769805312;
pub const KEYC_MOUSEDRAG3_PANE: C2RustUnnamed_38 = 25769804544;
pub const KEYC_MOUSEDRAG2_PANE: C2RustUnnamed_38 = 25769804288;
pub const KEYC_MOUSEDRAG1_PANE: C2RustUnnamed_38 = 25769804032;
pub const KEYC_MOUSEDRAG_PANE: C2RustUnnamed_38 = 25769803776;
pub const KEYC_MOUSEUP11_CONTROL9: C2RustUnnamed_38 = 21474839315;
pub const KEYC_MOUSEUP10_CONTROL9: C2RustUnnamed_38 = 21474839059;
pub const KEYC_MOUSEUP9_CONTROL9: C2RustUnnamed_38 = 21474838803;
pub const KEYC_MOUSEUP8_CONTROL9: C2RustUnnamed_38 = 21474838547;
pub const KEYC_MOUSEUP7_CONTROL9: C2RustUnnamed_38 = 21474838291;
pub const KEYC_MOUSEUP6_CONTROL9: C2RustUnnamed_38 = 21474838035;
pub const KEYC_MOUSEUP3_CONTROL9: C2RustUnnamed_38 = 21474837267;
pub const KEYC_MOUSEUP2_CONTROL9: C2RustUnnamed_38 = 21474837011;
pub const KEYC_MOUSEUP1_CONTROL9: C2RustUnnamed_38 = 21474836755;
pub const KEYC_MOUSEUP_CONTROL9: C2RustUnnamed_38 = 21474836499;
pub const KEYC_MOUSEUP11_CONTROL8: C2RustUnnamed_38 = 21474839314;
pub const KEYC_MOUSEUP10_CONTROL8: C2RustUnnamed_38 = 21474839058;
pub const KEYC_MOUSEUP9_CONTROL8: C2RustUnnamed_38 = 21474838802;
pub const KEYC_MOUSEUP8_CONTROL8: C2RustUnnamed_38 = 21474838546;
pub const KEYC_MOUSEUP7_CONTROL8: C2RustUnnamed_38 = 21474838290;
pub const KEYC_MOUSEUP6_CONTROL8: C2RustUnnamed_38 = 21474838034;
pub const KEYC_MOUSEUP3_CONTROL8: C2RustUnnamed_38 = 21474837266;
pub const KEYC_MOUSEUP2_CONTROL8: C2RustUnnamed_38 = 21474837010;
pub const KEYC_MOUSEUP1_CONTROL8: C2RustUnnamed_38 = 21474836754;
pub const KEYC_MOUSEUP_CONTROL8: C2RustUnnamed_38 = 21474836498;
pub const KEYC_MOUSEUP11_CONTROL7: C2RustUnnamed_38 = 21474839313;
pub const KEYC_MOUSEUP10_CONTROL7: C2RustUnnamed_38 = 21474839057;
pub const KEYC_MOUSEUP9_CONTROL7: C2RustUnnamed_38 = 21474838801;
pub const KEYC_MOUSEUP8_CONTROL7: C2RustUnnamed_38 = 21474838545;
pub const KEYC_MOUSEUP7_CONTROL7: C2RustUnnamed_38 = 21474838289;
pub const KEYC_MOUSEUP6_CONTROL7: C2RustUnnamed_38 = 21474838033;
pub const KEYC_MOUSEUP3_CONTROL7: C2RustUnnamed_38 = 21474837265;
pub const KEYC_MOUSEUP2_CONTROL7: C2RustUnnamed_38 = 21474837009;
pub const KEYC_MOUSEUP1_CONTROL7: C2RustUnnamed_38 = 21474836753;
pub const KEYC_MOUSEUP_CONTROL7: C2RustUnnamed_38 = 21474836497;
pub const KEYC_MOUSEUP11_CONTROL6: C2RustUnnamed_38 = 21474839312;
pub const KEYC_MOUSEUP10_CONTROL6: C2RustUnnamed_38 = 21474839056;
pub const KEYC_MOUSEUP9_CONTROL6: C2RustUnnamed_38 = 21474838800;
pub const KEYC_MOUSEUP8_CONTROL6: C2RustUnnamed_38 = 21474838544;
pub const KEYC_MOUSEUP7_CONTROL6: C2RustUnnamed_38 = 21474838288;
pub const KEYC_MOUSEUP6_CONTROL6: C2RustUnnamed_38 = 21474838032;
pub const KEYC_MOUSEUP3_CONTROL6: C2RustUnnamed_38 = 21474837264;
pub const KEYC_MOUSEUP2_CONTROL6: C2RustUnnamed_38 = 21474837008;
pub const KEYC_MOUSEUP1_CONTROL6: C2RustUnnamed_38 = 21474836752;
pub const KEYC_MOUSEUP_CONTROL6: C2RustUnnamed_38 = 21474836496;
pub const KEYC_MOUSEUP11_CONTROL5: C2RustUnnamed_38 = 21474839311;
pub const KEYC_MOUSEUP10_CONTROL5: C2RustUnnamed_38 = 21474839055;
pub const KEYC_MOUSEUP9_CONTROL5: C2RustUnnamed_38 = 21474838799;
pub const KEYC_MOUSEUP8_CONTROL5: C2RustUnnamed_38 = 21474838543;
pub const KEYC_MOUSEUP7_CONTROL5: C2RustUnnamed_38 = 21474838287;
pub const KEYC_MOUSEUP6_CONTROL5: C2RustUnnamed_38 = 21474838031;
pub const KEYC_MOUSEUP3_CONTROL5: C2RustUnnamed_38 = 21474837263;
pub const KEYC_MOUSEUP2_CONTROL5: C2RustUnnamed_38 = 21474837007;
pub const KEYC_MOUSEUP1_CONTROL5: C2RustUnnamed_38 = 21474836751;
pub const KEYC_MOUSEUP_CONTROL5: C2RustUnnamed_38 = 21474836495;
pub const KEYC_MOUSEUP11_CONTROL4: C2RustUnnamed_38 = 21474839310;
pub const KEYC_MOUSEUP10_CONTROL4: C2RustUnnamed_38 = 21474839054;
pub const KEYC_MOUSEUP9_CONTROL4: C2RustUnnamed_38 = 21474838798;
pub const KEYC_MOUSEUP8_CONTROL4: C2RustUnnamed_38 = 21474838542;
pub const KEYC_MOUSEUP7_CONTROL4: C2RustUnnamed_38 = 21474838286;
pub const KEYC_MOUSEUP6_CONTROL4: C2RustUnnamed_38 = 21474838030;
pub const KEYC_MOUSEUP3_CONTROL4: C2RustUnnamed_38 = 21474837262;
pub const KEYC_MOUSEUP2_CONTROL4: C2RustUnnamed_38 = 21474837006;
pub const KEYC_MOUSEUP1_CONTROL4: C2RustUnnamed_38 = 21474836750;
pub const KEYC_MOUSEUP_CONTROL4: C2RustUnnamed_38 = 21474836494;
pub const KEYC_MOUSEUP11_CONTROL3: C2RustUnnamed_38 = 21474839309;
pub const KEYC_MOUSEUP10_CONTROL3: C2RustUnnamed_38 = 21474839053;
pub const KEYC_MOUSEUP9_CONTROL3: C2RustUnnamed_38 = 21474838797;
pub const KEYC_MOUSEUP8_CONTROL3: C2RustUnnamed_38 = 21474838541;
pub const KEYC_MOUSEUP7_CONTROL3: C2RustUnnamed_38 = 21474838285;
pub const KEYC_MOUSEUP6_CONTROL3: C2RustUnnamed_38 = 21474838029;
pub const KEYC_MOUSEUP3_CONTROL3: C2RustUnnamed_38 = 21474837261;
pub const KEYC_MOUSEUP2_CONTROL3: C2RustUnnamed_38 = 21474837005;
pub const KEYC_MOUSEUP1_CONTROL3: C2RustUnnamed_38 = 21474836749;
pub const KEYC_MOUSEUP_CONTROL3: C2RustUnnamed_38 = 21474836493;
pub const KEYC_MOUSEUP11_CONTROL2: C2RustUnnamed_38 = 21474839308;
pub const KEYC_MOUSEUP10_CONTROL2: C2RustUnnamed_38 = 21474839052;
pub const KEYC_MOUSEUP9_CONTROL2: C2RustUnnamed_38 = 21474838796;
pub const KEYC_MOUSEUP8_CONTROL2: C2RustUnnamed_38 = 21474838540;
pub const KEYC_MOUSEUP7_CONTROL2: C2RustUnnamed_38 = 21474838284;
pub const KEYC_MOUSEUP6_CONTROL2: C2RustUnnamed_38 = 21474838028;
pub const KEYC_MOUSEUP3_CONTROL2: C2RustUnnamed_38 = 21474837260;
pub const KEYC_MOUSEUP2_CONTROL2: C2RustUnnamed_38 = 21474837004;
pub const KEYC_MOUSEUP1_CONTROL2: C2RustUnnamed_38 = 21474836748;
pub const KEYC_MOUSEUP_CONTROL2: C2RustUnnamed_38 = 21474836492;
pub const KEYC_MOUSEUP11_CONTROL1: C2RustUnnamed_38 = 21474839307;
pub const KEYC_MOUSEUP10_CONTROL1: C2RustUnnamed_38 = 21474839051;
pub const KEYC_MOUSEUP9_CONTROL1: C2RustUnnamed_38 = 21474838795;
pub const KEYC_MOUSEUP8_CONTROL1: C2RustUnnamed_38 = 21474838539;
pub const KEYC_MOUSEUP7_CONTROL1: C2RustUnnamed_38 = 21474838283;
pub const KEYC_MOUSEUP6_CONTROL1: C2RustUnnamed_38 = 21474838027;
pub const KEYC_MOUSEUP3_CONTROL1: C2RustUnnamed_38 = 21474837259;
pub const KEYC_MOUSEUP2_CONTROL1: C2RustUnnamed_38 = 21474837003;
pub const KEYC_MOUSEUP1_CONTROL1: C2RustUnnamed_38 = 21474836747;
pub const KEYC_MOUSEUP_CONTROL1: C2RustUnnamed_38 = 21474836491;
pub const KEYC_MOUSEUP11_CONTROL0: C2RustUnnamed_38 = 21474839306;
pub const KEYC_MOUSEUP10_CONTROL0: C2RustUnnamed_38 = 21474839050;
pub const KEYC_MOUSEUP9_CONTROL0: C2RustUnnamed_38 = 21474838794;
pub const KEYC_MOUSEUP8_CONTROL0: C2RustUnnamed_38 = 21474838538;
pub const KEYC_MOUSEUP7_CONTROL0: C2RustUnnamed_38 = 21474838282;
pub const KEYC_MOUSEUP6_CONTROL0: C2RustUnnamed_38 = 21474838026;
pub const KEYC_MOUSEUP3_CONTROL0: C2RustUnnamed_38 = 21474837258;
pub const KEYC_MOUSEUP2_CONTROL0: C2RustUnnamed_38 = 21474837002;
pub const KEYC_MOUSEUP1_CONTROL0: C2RustUnnamed_38 = 21474836746;
pub const KEYC_MOUSEUP_CONTROL0: C2RustUnnamed_38 = 21474836490;
pub const KEYC_MOUSEUP11_EMPTY: C2RustUnnamed_38 = 21474839305;
pub const KEYC_MOUSEUP10_EMPTY: C2RustUnnamed_38 = 21474839049;
pub const KEYC_MOUSEUP9_EMPTY: C2RustUnnamed_38 = 21474838793;
pub const KEYC_MOUSEUP8_EMPTY: C2RustUnnamed_38 = 21474838537;
pub const KEYC_MOUSEUP7_EMPTY: C2RustUnnamed_38 = 21474838281;
pub const KEYC_MOUSEUP6_EMPTY: C2RustUnnamed_38 = 21474838025;
pub const KEYC_MOUSEUP3_EMPTY: C2RustUnnamed_38 = 21474837257;
pub const KEYC_MOUSEUP2_EMPTY: C2RustUnnamed_38 = 21474837001;
pub const KEYC_MOUSEUP1_EMPTY: C2RustUnnamed_38 = 21474836745;
pub const KEYC_MOUSEUP_EMPTY: C2RustUnnamed_38 = 21474836489;
pub const KEYC_MOUSEUP11_SCROLLBAR_DOWN: C2RustUnnamed_38 = 21474839304;
pub const KEYC_MOUSEUP10_SCROLLBAR_DOWN: C2RustUnnamed_38 = 21474839048;
pub const KEYC_MOUSEUP9_SCROLLBAR_DOWN: C2RustUnnamed_38 = 21474838792;
pub const KEYC_MOUSEUP8_SCROLLBAR_DOWN: C2RustUnnamed_38 = 21474838536;
pub const KEYC_MOUSEUP7_SCROLLBAR_DOWN: C2RustUnnamed_38 = 21474838280;
pub const KEYC_MOUSEUP6_SCROLLBAR_DOWN: C2RustUnnamed_38 = 21474838024;
pub const KEYC_MOUSEUP3_SCROLLBAR_DOWN: C2RustUnnamed_38 = 21474837256;
pub const KEYC_MOUSEUP2_SCROLLBAR_DOWN: C2RustUnnamed_38 = 21474837000;
pub const KEYC_MOUSEUP1_SCROLLBAR_DOWN: C2RustUnnamed_38 = 21474836744;
pub const KEYC_MOUSEUP_SCROLLBAR_DOWN: C2RustUnnamed_38 = 21474836488;
pub const KEYC_MOUSEUP11_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 21474839303;
pub const KEYC_MOUSEUP10_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 21474839047;
pub const KEYC_MOUSEUP9_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 21474838791;
pub const KEYC_MOUSEUP8_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 21474838535;
pub const KEYC_MOUSEUP7_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 21474838279;
pub const KEYC_MOUSEUP6_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 21474838023;
pub const KEYC_MOUSEUP3_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 21474837255;
pub const KEYC_MOUSEUP2_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 21474836999;
pub const KEYC_MOUSEUP1_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 21474836743;
pub const KEYC_MOUSEUP_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 21474836487;
pub const KEYC_MOUSEUP11_SCROLLBAR_UP: C2RustUnnamed_38 = 21474839302;
pub const KEYC_MOUSEUP10_SCROLLBAR_UP: C2RustUnnamed_38 = 21474839046;
pub const KEYC_MOUSEUP9_SCROLLBAR_UP: C2RustUnnamed_38 = 21474838790;
pub const KEYC_MOUSEUP8_SCROLLBAR_UP: C2RustUnnamed_38 = 21474838534;
pub const KEYC_MOUSEUP7_SCROLLBAR_UP: C2RustUnnamed_38 = 21474838278;
pub const KEYC_MOUSEUP6_SCROLLBAR_UP: C2RustUnnamed_38 = 21474838022;
pub const KEYC_MOUSEUP3_SCROLLBAR_UP: C2RustUnnamed_38 = 21474837254;
pub const KEYC_MOUSEUP2_SCROLLBAR_UP: C2RustUnnamed_38 = 21474836998;
pub const KEYC_MOUSEUP1_SCROLLBAR_UP: C2RustUnnamed_38 = 21474836742;
pub const KEYC_MOUSEUP_SCROLLBAR_UP: C2RustUnnamed_38 = 21474836486;
pub const KEYC_MOUSEUP11_BORDER: C2RustUnnamed_38 = 21474839301;
pub const KEYC_MOUSEUP10_BORDER: C2RustUnnamed_38 = 21474839045;
pub const KEYC_MOUSEUP9_BORDER: C2RustUnnamed_38 = 21474838789;
pub const KEYC_MOUSEUP8_BORDER: C2RustUnnamed_38 = 21474838533;
pub const KEYC_MOUSEUP7_BORDER: C2RustUnnamed_38 = 21474838277;
pub const KEYC_MOUSEUP6_BORDER: C2RustUnnamed_38 = 21474838021;
pub const KEYC_MOUSEUP3_BORDER: C2RustUnnamed_38 = 21474837253;
pub const KEYC_MOUSEUP2_BORDER: C2RustUnnamed_38 = 21474836997;
pub const KEYC_MOUSEUP1_BORDER: C2RustUnnamed_38 = 21474836741;
pub const KEYC_MOUSEUP_BORDER: C2RustUnnamed_38 = 21474836485;
pub const KEYC_MOUSEUP11_STATUS_DEFAULT: C2RustUnnamed_38 = 21474839300;
pub const KEYC_MOUSEUP10_STATUS_DEFAULT: C2RustUnnamed_38 = 21474839044;
pub const KEYC_MOUSEUP9_STATUS_DEFAULT: C2RustUnnamed_38 = 21474838788;
pub const KEYC_MOUSEUP8_STATUS_DEFAULT: C2RustUnnamed_38 = 21474838532;
pub const KEYC_MOUSEUP7_STATUS_DEFAULT: C2RustUnnamed_38 = 21474838276;
pub const KEYC_MOUSEUP6_STATUS_DEFAULT: C2RustUnnamed_38 = 21474838020;
pub const KEYC_MOUSEUP3_STATUS_DEFAULT: C2RustUnnamed_38 = 21474837252;
pub const KEYC_MOUSEUP2_STATUS_DEFAULT: C2RustUnnamed_38 = 21474836996;
pub const KEYC_MOUSEUP1_STATUS_DEFAULT: C2RustUnnamed_38 = 21474836740;
pub const KEYC_MOUSEUP_STATUS_DEFAULT: C2RustUnnamed_38 = 21474836484;
pub const KEYC_MOUSEUP11_STATUS_RIGHT: C2RustUnnamed_38 = 21474839299;
pub const KEYC_MOUSEUP10_STATUS_RIGHT: C2RustUnnamed_38 = 21474839043;
pub const KEYC_MOUSEUP9_STATUS_RIGHT: C2RustUnnamed_38 = 21474838787;
pub const KEYC_MOUSEUP8_STATUS_RIGHT: C2RustUnnamed_38 = 21474838531;
pub const KEYC_MOUSEUP7_STATUS_RIGHT: C2RustUnnamed_38 = 21474838275;
pub const KEYC_MOUSEUP6_STATUS_RIGHT: C2RustUnnamed_38 = 21474838019;
pub const KEYC_MOUSEUP3_STATUS_RIGHT: C2RustUnnamed_38 = 21474837251;
pub const KEYC_MOUSEUP2_STATUS_RIGHT: C2RustUnnamed_38 = 21474836995;
pub const KEYC_MOUSEUP1_STATUS_RIGHT: C2RustUnnamed_38 = 21474836739;
pub const KEYC_MOUSEUP_STATUS_RIGHT: C2RustUnnamed_38 = 21474836483;
pub const KEYC_MOUSEUP11_STATUS_LEFT: C2RustUnnamed_38 = 21474839298;
pub const KEYC_MOUSEUP10_STATUS_LEFT: C2RustUnnamed_38 = 21474839042;
pub const KEYC_MOUSEUP9_STATUS_LEFT: C2RustUnnamed_38 = 21474838786;
pub const KEYC_MOUSEUP8_STATUS_LEFT: C2RustUnnamed_38 = 21474838530;
pub const KEYC_MOUSEUP7_STATUS_LEFT: C2RustUnnamed_38 = 21474838274;
pub const KEYC_MOUSEUP6_STATUS_LEFT: C2RustUnnamed_38 = 21474838018;
pub const KEYC_MOUSEUP3_STATUS_LEFT: C2RustUnnamed_38 = 21474837250;
pub const KEYC_MOUSEUP2_STATUS_LEFT: C2RustUnnamed_38 = 21474836994;
pub const KEYC_MOUSEUP1_STATUS_LEFT: C2RustUnnamed_38 = 21474836738;
pub const KEYC_MOUSEUP_STATUS_LEFT: C2RustUnnamed_38 = 21474836482;
pub const KEYC_MOUSEUP11_STATUS: C2RustUnnamed_38 = 21474839297;
pub const KEYC_MOUSEUP10_STATUS: C2RustUnnamed_38 = 21474839041;
pub const KEYC_MOUSEUP9_STATUS: C2RustUnnamed_38 = 21474838785;
pub const KEYC_MOUSEUP8_STATUS: C2RustUnnamed_38 = 21474838529;
pub const KEYC_MOUSEUP7_STATUS: C2RustUnnamed_38 = 21474838273;
pub const KEYC_MOUSEUP6_STATUS: C2RustUnnamed_38 = 21474838017;
pub const KEYC_MOUSEUP3_STATUS: C2RustUnnamed_38 = 21474837249;
pub const KEYC_MOUSEUP2_STATUS: C2RustUnnamed_38 = 21474836993;
pub const KEYC_MOUSEUP1_STATUS: C2RustUnnamed_38 = 21474836737;
pub const KEYC_MOUSEUP_STATUS: C2RustUnnamed_38 = 21474836481;
pub const KEYC_MOUSEUP11_PANE: C2RustUnnamed_38 = 21474839296;
pub const KEYC_MOUSEUP10_PANE: C2RustUnnamed_38 = 21474839040;
pub const KEYC_MOUSEUP9_PANE: C2RustUnnamed_38 = 21474838784;
pub const KEYC_MOUSEUP8_PANE: C2RustUnnamed_38 = 21474838528;
pub const KEYC_MOUSEUP7_PANE: C2RustUnnamed_38 = 21474838272;
pub const KEYC_MOUSEUP6_PANE: C2RustUnnamed_38 = 21474838016;
pub const KEYC_MOUSEUP3_PANE: C2RustUnnamed_38 = 21474837248;
pub const KEYC_MOUSEUP2_PANE: C2RustUnnamed_38 = 21474836992;
pub const KEYC_MOUSEUP1_PANE: C2RustUnnamed_38 = 21474836736;
pub const KEYC_MOUSEUP_PANE: C2RustUnnamed_38 = 21474836480;
pub const KEYC_MOUSEDOWN11_CONTROL9: C2RustUnnamed_38 = 17179872019;
pub const KEYC_MOUSEDOWN10_CONTROL9: C2RustUnnamed_38 = 17179871763;
pub const KEYC_MOUSEDOWN9_CONTROL9: C2RustUnnamed_38 = 17179871507;
pub const KEYC_MOUSEDOWN8_CONTROL9: C2RustUnnamed_38 = 17179871251;
pub const KEYC_MOUSEDOWN7_CONTROL9: C2RustUnnamed_38 = 17179870995;
pub const KEYC_MOUSEDOWN6_CONTROL9: C2RustUnnamed_38 = 17179870739;
pub const KEYC_MOUSEDOWN3_CONTROL9: C2RustUnnamed_38 = 17179869971;
pub const KEYC_MOUSEDOWN2_CONTROL9: C2RustUnnamed_38 = 17179869715;
pub const KEYC_MOUSEDOWN1_CONTROL9: C2RustUnnamed_38 = 17179869459;
pub const KEYC_MOUSEDOWN_CONTROL9: C2RustUnnamed_38 = 17179869203;
pub const KEYC_MOUSEDOWN11_CONTROL8: C2RustUnnamed_38 = 17179872018;
pub const KEYC_MOUSEDOWN10_CONTROL8: C2RustUnnamed_38 = 17179871762;
pub const KEYC_MOUSEDOWN9_CONTROL8: C2RustUnnamed_38 = 17179871506;
pub const KEYC_MOUSEDOWN8_CONTROL8: C2RustUnnamed_38 = 17179871250;
pub const KEYC_MOUSEDOWN7_CONTROL8: C2RustUnnamed_38 = 17179870994;
pub const KEYC_MOUSEDOWN6_CONTROL8: C2RustUnnamed_38 = 17179870738;
pub const KEYC_MOUSEDOWN3_CONTROL8: C2RustUnnamed_38 = 17179869970;
pub const KEYC_MOUSEDOWN2_CONTROL8: C2RustUnnamed_38 = 17179869714;
pub const KEYC_MOUSEDOWN1_CONTROL8: C2RustUnnamed_38 = 17179869458;
pub const KEYC_MOUSEDOWN_CONTROL8: C2RustUnnamed_38 = 17179869202;
pub const KEYC_MOUSEDOWN11_CONTROL7: C2RustUnnamed_38 = 17179872017;
pub const KEYC_MOUSEDOWN10_CONTROL7: C2RustUnnamed_38 = 17179871761;
pub const KEYC_MOUSEDOWN9_CONTROL7: C2RustUnnamed_38 = 17179871505;
pub const KEYC_MOUSEDOWN8_CONTROL7: C2RustUnnamed_38 = 17179871249;
pub const KEYC_MOUSEDOWN7_CONTROL7: C2RustUnnamed_38 = 17179870993;
pub const KEYC_MOUSEDOWN6_CONTROL7: C2RustUnnamed_38 = 17179870737;
pub const KEYC_MOUSEDOWN3_CONTROL7: C2RustUnnamed_38 = 17179869969;
pub const KEYC_MOUSEDOWN2_CONTROL7: C2RustUnnamed_38 = 17179869713;
pub const KEYC_MOUSEDOWN1_CONTROL7: C2RustUnnamed_38 = 17179869457;
pub const KEYC_MOUSEDOWN_CONTROL7: C2RustUnnamed_38 = 17179869201;
pub const KEYC_MOUSEDOWN11_CONTROL6: C2RustUnnamed_38 = 17179872016;
pub const KEYC_MOUSEDOWN10_CONTROL6: C2RustUnnamed_38 = 17179871760;
pub const KEYC_MOUSEDOWN9_CONTROL6: C2RustUnnamed_38 = 17179871504;
pub const KEYC_MOUSEDOWN8_CONTROL6: C2RustUnnamed_38 = 17179871248;
pub const KEYC_MOUSEDOWN7_CONTROL6: C2RustUnnamed_38 = 17179870992;
pub const KEYC_MOUSEDOWN6_CONTROL6: C2RustUnnamed_38 = 17179870736;
pub const KEYC_MOUSEDOWN3_CONTROL6: C2RustUnnamed_38 = 17179869968;
pub const KEYC_MOUSEDOWN2_CONTROL6: C2RustUnnamed_38 = 17179869712;
pub const KEYC_MOUSEDOWN1_CONTROL6: C2RustUnnamed_38 = 17179869456;
pub const KEYC_MOUSEDOWN_CONTROL6: C2RustUnnamed_38 = 17179869200;
pub const KEYC_MOUSEDOWN11_CONTROL5: C2RustUnnamed_38 = 17179872015;
pub const KEYC_MOUSEDOWN10_CONTROL5: C2RustUnnamed_38 = 17179871759;
pub const KEYC_MOUSEDOWN9_CONTROL5: C2RustUnnamed_38 = 17179871503;
pub const KEYC_MOUSEDOWN8_CONTROL5: C2RustUnnamed_38 = 17179871247;
pub const KEYC_MOUSEDOWN7_CONTROL5: C2RustUnnamed_38 = 17179870991;
pub const KEYC_MOUSEDOWN6_CONTROL5: C2RustUnnamed_38 = 17179870735;
pub const KEYC_MOUSEDOWN3_CONTROL5: C2RustUnnamed_38 = 17179869967;
pub const KEYC_MOUSEDOWN2_CONTROL5: C2RustUnnamed_38 = 17179869711;
pub const KEYC_MOUSEDOWN1_CONTROL5: C2RustUnnamed_38 = 17179869455;
pub const KEYC_MOUSEDOWN_CONTROL5: C2RustUnnamed_38 = 17179869199;
pub const KEYC_MOUSEDOWN11_CONTROL4: C2RustUnnamed_38 = 17179872014;
pub const KEYC_MOUSEDOWN10_CONTROL4: C2RustUnnamed_38 = 17179871758;
pub const KEYC_MOUSEDOWN9_CONTROL4: C2RustUnnamed_38 = 17179871502;
pub const KEYC_MOUSEDOWN8_CONTROL4: C2RustUnnamed_38 = 17179871246;
pub const KEYC_MOUSEDOWN7_CONTROL4: C2RustUnnamed_38 = 17179870990;
pub const KEYC_MOUSEDOWN6_CONTROL4: C2RustUnnamed_38 = 17179870734;
pub const KEYC_MOUSEDOWN3_CONTROL4: C2RustUnnamed_38 = 17179869966;
pub const KEYC_MOUSEDOWN2_CONTROL4: C2RustUnnamed_38 = 17179869710;
pub const KEYC_MOUSEDOWN1_CONTROL4: C2RustUnnamed_38 = 17179869454;
pub const KEYC_MOUSEDOWN_CONTROL4: C2RustUnnamed_38 = 17179869198;
pub const KEYC_MOUSEDOWN11_CONTROL3: C2RustUnnamed_38 = 17179872013;
pub const KEYC_MOUSEDOWN10_CONTROL3: C2RustUnnamed_38 = 17179871757;
pub const KEYC_MOUSEDOWN9_CONTROL3: C2RustUnnamed_38 = 17179871501;
pub const KEYC_MOUSEDOWN8_CONTROL3: C2RustUnnamed_38 = 17179871245;
pub const KEYC_MOUSEDOWN7_CONTROL3: C2RustUnnamed_38 = 17179870989;
pub const KEYC_MOUSEDOWN6_CONTROL3: C2RustUnnamed_38 = 17179870733;
pub const KEYC_MOUSEDOWN3_CONTROL3: C2RustUnnamed_38 = 17179869965;
pub const KEYC_MOUSEDOWN2_CONTROL3: C2RustUnnamed_38 = 17179869709;
pub const KEYC_MOUSEDOWN1_CONTROL3: C2RustUnnamed_38 = 17179869453;
pub const KEYC_MOUSEDOWN_CONTROL3: C2RustUnnamed_38 = 17179869197;
pub const KEYC_MOUSEDOWN11_CONTROL2: C2RustUnnamed_38 = 17179872012;
pub const KEYC_MOUSEDOWN10_CONTROL2: C2RustUnnamed_38 = 17179871756;
pub const KEYC_MOUSEDOWN9_CONTROL2: C2RustUnnamed_38 = 17179871500;
pub const KEYC_MOUSEDOWN8_CONTROL2: C2RustUnnamed_38 = 17179871244;
pub const KEYC_MOUSEDOWN7_CONTROL2: C2RustUnnamed_38 = 17179870988;
pub const KEYC_MOUSEDOWN6_CONTROL2: C2RustUnnamed_38 = 17179870732;
pub const KEYC_MOUSEDOWN3_CONTROL2: C2RustUnnamed_38 = 17179869964;
pub const KEYC_MOUSEDOWN2_CONTROL2: C2RustUnnamed_38 = 17179869708;
pub const KEYC_MOUSEDOWN1_CONTROL2: C2RustUnnamed_38 = 17179869452;
pub const KEYC_MOUSEDOWN_CONTROL2: C2RustUnnamed_38 = 17179869196;
pub const KEYC_MOUSEDOWN11_CONTROL1: C2RustUnnamed_38 = 17179872011;
pub const KEYC_MOUSEDOWN10_CONTROL1: C2RustUnnamed_38 = 17179871755;
pub const KEYC_MOUSEDOWN9_CONTROL1: C2RustUnnamed_38 = 17179871499;
pub const KEYC_MOUSEDOWN8_CONTROL1: C2RustUnnamed_38 = 17179871243;
pub const KEYC_MOUSEDOWN7_CONTROL1: C2RustUnnamed_38 = 17179870987;
pub const KEYC_MOUSEDOWN6_CONTROL1: C2RustUnnamed_38 = 17179870731;
pub const KEYC_MOUSEDOWN3_CONTROL1: C2RustUnnamed_38 = 17179869963;
pub const KEYC_MOUSEDOWN2_CONTROL1: C2RustUnnamed_38 = 17179869707;
pub const KEYC_MOUSEDOWN1_CONTROL1: C2RustUnnamed_38 = 17179869451;
pub const KEYC_MOUSEDOWN_CONTROL1: C2RustUnnamed_38 = 17179869195;
pub const KEYC_MOUSEDOWN11_CONTROL0: C2RustUnnamed_38 = 17179872010;
pub const KEYC_MOUSEDOWN10_CONTROL0: C2RustUnnamed_38 = 17179871754;
pub const KEYC_MOUSEDOWN9_CONTROL0: C2RustUnnamed_38 = 17179871498;
pub const KEYC_MOUSEDOWN8_CONTROL0: C2RustUnnamed_38 = 17179871242;
pub const KEYC_MOUSEDOWN7_CONTROL0: C2RustUnnamed_38 = 17179870986;
pub const KEYC_MOUSEDOWN6_CONTROL0: C2RustUnnamed_38 = 17179870730;
pub const KEYC_MOUSEDOWN3_CONTROL0: C2RustUnnamed_38 = 17179869962;
pub const KEYC_MOUSEDOWN2_CONTROL0: C2RustUnnamed_38 = 17179869706;
pub const KEYC_MOUSEDOWN1_CONTROL0: C2RustUnnamed_38 = 17179869450;
pub const KEYC_MOUSEDOWN_CONTROL0: C2RustUnnamed_38 = 17179869194;
pub const KEYC_MOUSEDOWN11_EMPTY: C2RustUnnamed_38 = 17179872009;
pub const KEYC_MOUSEDOWN10_EMPTY: C2RustUnnamed_38 = 17179871753;
pub const KEYC_MOUSEDOWN9_EMPTY: C2RustUnnamed_38 = 17179871497;
pub const KEYC_MOUSEDOWN8_EMPTY: C2RustUnnamed_38 = 17179871241;
pub const KEYC_MOUSEDOWN7_EMPTY: C2RustUnnamed_38 = 17179870985;
pub const KEYC_MOUSEDOWN6_EMPTY: C2RustUnnamed_38 = 17179870729;
pub const KEYC_MOUSEDOWN3_EMPTY: C2RustUnnamed_38 = 17179869961;
pub const KEYC_MOUSEDOWN2_EMPTY: C2RustUnnamed_38 = 17179869705;
pub const KEYC_MOUSEDOWN1_EMPTY: C2RustUnnamed_38 = 17179869449;
pub const KEYC_MOUSEDOWN_EMPTY: C2RustUnnamed_38 = 17179869193;
pub const KEYC_MOUSEDOWN11_SCROLLBAR_DOWN: C2RustUnnamed_38 = 17179872008;
pub const KEYC_MOUSEDOWN10_SCROLLBAR_DOWN: C2RustUnnamed_38 = 17179871752;
pub const KEYC_MOUSEDOWN9_SCROLLBAR_DOWN: C2RustUnnamed_38 = 17179871496;
pub const KEYC_MOUSEDOWN8_SCROLLBAR_DOWN: C2RustUnnamed_38 = 17179871240;
pub const KEYC_MOUSEDOWN7_SCROLLBAR_DOWN: C2RustUnnamed_38 = 17179870984;
pub const KEYC_MOUSEDOWN6_SCROLLBAR_DOWN: C2RustUnnamed_38 = 17179870728;
pub const KEYC_MOUSEDOWN3_SCROLLBAR_DOWN: C2RustUnnamed_38 = 17179869960;
pub const KEYC_MOUSEDOWN2_SCROLLBAR_DOWN: C2RustUnnamed_38 = 17179869704;
pub const KEYC_MOUSEDOWN1_SCROLLBAR_DOWN: C2RustUnnamed_38 = 17179869448;
pub const KEYC_MOUSEDOWN_SCROLLBAR_DOWN: C2RustUnnamed_38 = 17179869192;
pub const KEYC_MOUSEDOWN11_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 17179872007;
pub const KEYC_MOUSEDOWN10_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 17179871751;
pub const KEYC_MOUSEDOWN9_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 17179871495;
pub const KEYC_MOUSEDOWN8_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 17179871239;
pub const KEYC_MOUSEDOWN7_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 17179870983;
pub const KEYC_MOUSEDOWN6_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 17179870727;
pub const KEYC_MOUSEDOWN3_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 17179869959;
pub const KEYC_MOUSEDOWN2_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 17179869703;
pub const KEYC_MOUSEDOWN1_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 17179869447;
pub const KEYC_MOUSEDOWN_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 17179869191;
pub const KEYC_MOUSEDOWN11_SCROLLBAR_UP: C2RustUnnamed_38 = 17179872006;
pub const KEYC_MOUSEDOWN10_SCROLLBAR_UP: C2RustUnnamed_38 = 17179871750;
pub const KEYC_MOUSEDOWN9_SCROLLBAR_UP: C2RustUnnamed_38 = 17179871494;
pub const KEYC_MOUSEDOWN8_SCROLLBAR_UP: C2RustUnnamed_38 = 17179871238;
pub const KEYC_MOUSEDOWN7_SCROLLBAR_UP: C2RustUnnamed_38 = 17179870982;
pub const KEYC_MOUSEDOWN6_SCROLLBAR_UP: C2RustUnnamed_38 = 17179870726;
pub const KEYC_MOUSEDOWN3_SCROLLBAR_UP: C2RustUnnamed_38 = 17179869958;
pub const KEYC_MOUSEDOWN2_SCROLLBAR_UP: C2RustUnnamed_38 = 17179869702;
pub const KEYC_MOUSEDOWN1_SCROLLBAR_UP: C2RustUnnamed_38 = 17179869446;
pub const KEYC_MOUSEDOWN_SCROLLBAR_UP: C2RustUnnamed_38 = 17179869190;
pub const KEYC_MOUSEDOWN11_BORDER: C2RustUnnamed_38 = 17179872005;
pub const KEYC_MOUSEDOWN10_BORDER: C2RustUnnamed_38 = 17179871749;
pub const KEYC_MOUSEDOWN9_BORDER: C2RustUnnamed_38 = 17179871493;
pub const KEYC_MOUSEDOWN8_BORDER: C2RustUnnamed_38 = 17179871237;
pub const KEYC_MOUSEDOWN7_BORDER: C2RustUnnamed_38 = 17179870981;
pub const KEYC_MOUSEDOWN6_BORDER: C2RustUnnamed_38 = 17179870725;
pub const KEYC_MOUSEDOWN3_BORDER: C2RustUnnamed_38 = 17179869957;
pub const KEYC_MOUSEDOWN2_BORDER: C2RustUnnamed_38 = 17179869701;
pub const KEYC_MOUSEDOWN1_BORDER: C2RustUnnamed_38 = 17179869445;
pub const KEYC_MOUSEDOWN_BORDER: C2RustUnnamed_38 = 17179869189;
pub const KEYC_MOUSEDOWN11_STATUS_DEFAULT: C2RustUnnamed_38 = 17179872004;
pub const KEYC_MOUSEDOWN10_STATUS_DEFAULT: C2RustUnnamed_38 = 17179871748;
pub const KEYC_MOUSEDOWN9_STATUS_DEFAULT: C2RustUnnamed_38 = 17179871492;
pub const KEYC_MOUSEDOWN8_STATUS_DEFAULT: C2RustUnnamed_38 = 17179871236;
pub const KEYC_MOUSEDOWN7_STATUS_DEFAULT: C2RustUnnamed_38 = 17179870980;
pub const KEYC_MOUSEDOWN6_STATUS_DEFAULT: C2RustUnnamed_38 = 17179870724;
pub const KEYC_MOUSEDOWN3_STATUS_DEFAULT: C2RustUnnamed_38 = 17179869956;
pub const KEYC_MOUSEDOWN2_STATUS_DEFAULT: C2RustUnnamed_38 = 17179869700;
pub const KEYC_MOUSEDOWN1_STATUS_DEFAULT: C2RustUnnamed_38 = 17179869444;
pub const KEYC_MOUSEDOWN_STATUS_DEFAULT: C2RustUnnamed_38 = 17179869188;
pub const KEYC_MOUSEDOWN11_STATUS_RIGHT: C2RustUnnamed_38 = 17179872003;
pub const KEYC_MOUSEDOWN10_STATUS_RIGHT: C2RustUnnamed_38 = 17179871747;
pub const KEYC_MOUSEDOWN9_STATUS_RIGHT: C2RustUnnamed_38 = 17179871491;
pub const KEYC_MOUSEDOWN8_STATUS_RIGHT: C2RustUnnamed_38 = 17179871235;
pub const KEYC_MOUSEDOWN7_STATUS_RIGHT: C2RustUnnamed_38 = 17179870979;
pub const KEYC_MOUSEDOWN6_STATUS_RIGHT: C2RustUnnamed_38 = 17179870723;
pub const KEYC_MOUSEDOWN3_STATUS_RIGHT: C2RustUnnamed_38 = 17179869955;
pub const KEYC_MOUSEDOWN2_STATUS_RIGHT: C2RustUnnamed_38 = 17179869699;
pub const KEYC_MOUSEDOWN1_STATUS_RIGHT: C2RustUnnamed_38 = 17179869443;
pub const KEYC_MOUSEDOWN_STATUS_RIGHT: C2RustUnnamed_38 = 17179869187;
pub const KEYC_MOUSEDOWN11_STATUS_LEFT: C2RustUnnamed_38 = 17179872002;
pub const KEYC_MOUSEDOWN10_STATUS_LEFT: C2RustUnnamed_38 = 17179871746;
pub const KEYC_MOUSEDOWN9_STATUS_LEFT: C2RustUnnamed_38 = 17179871490;
pub const KEYC_MOUSEDOWN8_STATUS_LEFT: C2RustUnnamed_38 = 17179871234;
pub const KEYC_MOUSEDOWN7_STATUS_LEFT: C2RustUnnamed_38 = 17179870978;
pub const KEYC_MOUSEDOWN6_STATUS_LEFT: C2RustUnnamed_38 = 17179870722;
pub const KEYC_MOUSEDOWN3_STATUS_LEFT: C2RustUnnamed_38 = 17179869954;
pub const KEYC_MOUSEDOWN2_STATUS_LEFT: C2RustUnnamed_38 = 17179869698;
pub const KEYC_MOUSEDOWN1_STATUS_LEFT: C2RustUnnamed_38 = 17179869442;
pub const KEYC_MOUSEDOWN_STATUS_LEFT: C2RustUnnamed_38 = 17179869186;
pub const KEYC_MOUSEDOWN11_STATUS: C2RustUnnamed_38 = 17179872001;
pub const KEYC_MOUSEDOWN10_STATUS: C2RustUnnamed_38 = 17179871745;
pub const KEYC_MOUSEDOWN9_STATUS: C2RustUnnamed_38 = 17179871489;
pub const KEYC_MOUSEDOWN8_STATUS: C2RustUnnamed_38 = 17179871233;
pub const KEYC_MOUSEDOWN7_STATUS: C2RustUnnamed_38 = 17179870977;
pub const KEYC_MOUSEDOWN6_STATUS: C2RustUnnamed_38 = 17179870721;
pub const KEYC_MOUSEDOWN3_STATUS: C2RustUnnamed_38 = 17179869953;
pub const KEYC_MOUSEDOWN2_STATUS: C2RustUnnamed_38 = 17179869697;
pub const KEYC_MOUSEDOWN1_STATUS: C2RustUnnamed_38 = 17179869441;
pub const KEYC_MOUSEDOWN_STATUS: C2RustUnnamed_38 = 17179869185;
pub const KEYC_MOUSEDOWN11_PANE: C2RustUnnamed_38 = 17179872000;
pub const KEYC_MOUSEDOWN10_PANE: C2RustUnnamed_38 = 17179871744;
pub const KEYC_MOUSEDOWN9_PANE: C2RustUnnamed_38 = 17179871488;
pub const KEYC_MOUSEDOWN8_PANE: C2RustUnnamed_38 = 17179871232;
pub const KEYC_MOUSEDOWN7_PANE: C2RustUnnamed_38 = 17179870976;
pub const KEYC_MOUSEDOWN6_PANE: C2RustUnnamed_38 = 17179870720;
pub const KEYC_MOUSEDOWN3_PANE: C2RustUnnamed_38 = 17179869952;
pub const KEYC_MOUSEDOWN2_PANE: C2RustUnnamed_38 = 17179869696;
pub const KEYC_MOUSEDOWN1_PANE: C2RustUnnamed_38 = 17179869440;
pub const KEYC_MOUSEDOWN_PANE: C2RustUnnamed_38 = 17179869184;
pub const KEYC_WHEELUP11_CONTROL9: C2RustUnnamed_38 = 38654708499;
pub const KEYC_WHEELUP10_CONTROL9: C2RustUnnamed_38 = 38654708243;
pub const KEYC_WHEELUP9_CONTROL9: C2RustUnnamed_38 = 38654707987;
pub const KEYC_WHEELUP8_CONTROL9: C2RustUnnamed_38 = 38654707731;
pub const KEYC_WHEELUP7_CONTROL9: C2RustUnnamed_38 = 38654707475;
pub const KEYC_WHEELUP6_CONTROL9: C2RustUnnamed_38 = 38654707219;
pub const KEYC_WHEELUP3_CONTROL9: C2RustUnnamed_38 = 38654706451;
pub const KEYC_WHEELUP2_CONTROL9: C2RustUnnamed_38 = 38654706195;
pub const KEYC_WHEELUP1_CONTROL9: C2RustUnnamed_38 = 38654705939;
pub const KEYC_WHEELUP_CONTROL9: C2RustUnnamed_38 = 38654705683;
pub const KEYC_WHEELUP11_CONTROL8: C2RustUnnamed_38 = 38654708498;
pub const KEYC_WHEELUP10_CONTROL8: C2RustUnnamed_38 = 38654708242;
pub const KEYC_WHEELUP9_CONTROL8: C2RustUnnamed_38 = 38654707986;
pub const KEYC_WHEELUP8_CONTROL8: C2RustUnnamed_38 = 38654707730;
pub const KEYC_WHEELUP7_CONTROL8: C2RustUnnamed_38 = 38654707474;
pub const KEYC_WHEELUP6_CONTROL8: C2RustUnnamed_38 = 38654707218;
pub const KEYC_WHEELUP3_CONTROL8: C2RustUnnamed_38 = 38654706450;
pub const KEYC_WHEELUP2_CONTROL8: C2RustUnnamed_38 = 38654706194;
pub const KEYC_WHEELUP1_CONTROL8: C2RustUnnamed_38 = 38654705938;
pub const KEYC_WHEELUP_CONTROL8: C2RustUnnamed_38 = 38654705682;
pub const KEYC_WHEELUP11_CONTROL7: C2RustUnnamed_38 = 38654708497;
pub const KEYC_WHEELUP10_CONTROL7: C2RustUnnamed_38 = 38654708241;
pub const KEYC_WHEELUP9_CONTROL7: C2RustUnnamed_38 = 38654707985;
pub const KEYC_WHEELUP8_CONTROL7: C2RustUnnamed_38 = 38654707729;
pub const KEYC_WHEELUP7_CONTROL7: C2RustUnnamed_38 = 38654707473;
pub const KEYC_WHEELUP6_CONTROL7: C2RustUnnamed_38 = 38654707217;
pub const KEYC_WHEELUP3_CONTROL7: C2RustUnnamed_38 = 38654706449;
pub const KEYC_WHEELUP2_CONTROL7: C2RustUnnamed_38 = 38654706193;
pub const KEYC_WHEELUP1_CONTROL7: C2RustUnnamed_38 = 38654705937;
pub const KEYC_WHEELUP_CONTROL7: C2RustUnnamed_38 = 38654705681;
pub const KEYC_WHEELUP11_CONTROL6: C2RustUnnamed_38 = 38654708496;
pub const KEYC_WHEELUP10_CONTROL6: C2RustUnnamed_38 = 38654708240;
pub const KEYC_WHEELUP9_CONTROL6: C2RustUnnamed_38 = 38654707984;
pub const KEYC_WHEELUP8_CONTROL6: C2RustUnnamed_38 = 38654707728;
pub const KEYC_WHEELUP7_CONTROL6: C2RustUnnamed_38 = 38654707472;
pub const KEYC_WHEELUP6_CONTROL6: C2RustUnnamed_38 = 38654707216;
pub const KEYC_WHEELUP3_CONTROL6: C2RustUnnamed_38 = 38654706448;
pub const KEYC_WHEELUP2_CONTROL6: C2RustUnnamed_38 = 38654706192;
pub const KEYC_WHEELUP1_CONTROL6: C2RustUnnamed_38 = 38654705936;
pub const KEYC_WHEELUP_CONTROL6: C2RustUnnamed_38 = 38654705680;
pub const KEYC_WHEELUP11_CONTROL5: C2RustUnnamed_38 = 38654708495;
pub const KEYC_WHEELUP10_CONTROL5: C2RustUnnamed_38 = 38654708239;
pub const KEYC_WHEELUP9_CONTROL5: C2RustUnnamed_38 = 38654707983;
pub const KEYC_WHEELUP8_CONTROL5: C2RustUnnamed_38 = 38654707727;
pub const KEYC_WHEELUP7_CONTROL5: C2RustUnnamed_38 = 38654707471;
pub const KEYC_WHEELUP6_CONTROL5: C2RustUnnamed_38 = 38654707215;
pub const KEYC_WHEELUP3_CONTROL5: C2RustUnnamed_38 = 38654706447;
pub const KEYC_WHEELUP2_CONTROL5: C2RustUnnamed_38 = 38654706191;
pub const KEYC_WHEELUP1_CONTROL5: C2RustUnnamed_38 = 38654705935;
pub const KEYC_WHEELUP_CONTROL5: C2RustUnnamed_38 = 38654705679;
pub const KEYC_WHEELUP11_CONTROL4: C2RustUnnamed_38 = 38654708494;
pub const KEYC_WHEELUP10_CONTROL4: C2RustUnnamed_38 = 38654708238;
pub const KEYC_WHEELUP9_CONTROL4: C2RustUnnamed_38 = 38654707982;
pub const KEYC_WHEELUP8_CONTROL4: C2RustUnnamed_38 = 38654707726;
pub const KEYC_WHEELUP7_CONTROL4: C2RustUnnamed_38 = 38654707470;
pub const KEYC_WHEELUP6_CONTROL4: C2RustUnnamed_38 = 38654707214;
pub const KEYC_WHEELUP3_CONTROL4: C2RustUnnamed_38 = 38654706446;
pub const KEYC_WHEELUP2_CONTROL4: C2RustUnnamed_38 = 38654706190;
pub const KEYC_WHEELUP1_CONTROL4: C2RustUnnamed_38 = 38654705934;
pub const KEYC_WHEELUP_CONTROL4: C2RustUnnamed_38 = 38654705678;
pub const KEYC_WHEELUP11_CONTROL3: C2RustUnnamed_38 = 38654708493;
pub const KEYC_WHEELUP10_CONTROL3: C2RustUnnamed_38 = 38654708237;
pub const KEYC_WHEELUP9_CONTROL3: C2RustUnnamed_38 = 38654707981;
pub const KEYC_WHEELUP8_CONTROL3: C2RustUnnamed_38 = 38654707725;
pub const KEYC_WHEELUP7_CONTROL3: C2RustUnnamed_38 = 38654707469;
pub const KEYC_WHEELUP6_CONTROL3: C2RustUnnamed_38 = 38654707213;
pub const KEYC_WHEELUP3_CONTROL3: C2RustUnnamed_38 = 38654706445;
pub const KEYC_WHEELUP2_CONTROL3: C2RustUnnamed_38 = 38654706189;
pub const KEYC_WHEELUP1_CONTROL3: C2RustUnnamed_38 = 38654705933;
pub const KEYC_WHEELUP_CONTROL3: C2RustUnnamed_38 = 38654705677;
pub const KEYC_WHEELUP11_CONTROL2: C2RustUnnamed_38 = 38654708492;
pub const KEYC_WHEELUP10_CONTROL2: C2RustUnnamed_38 = 38654708236;
pub const KEYC_WHEELUP9_CONTROL2: C2RustUnnamed_38 = 38654707980;
pub const KEYC_WHEELUP8_CONTROL2: C2RustUnnamed_38 = 38654707724;
pub const KEYC_WHEELUP7_CONTROL2: C2RustUnnamed_38 = 38654707468;
pub const KEYC_WHEELUP6_CONTROL2: C2RustUnnamed_38 = 38654707212;
pub const KEYC_WHEELUP3_CONTROL2: C2RustUnnamed_38 = 38654706444;
pub const KEYC_WHEELUP2_CONTROL2: C2RustUnnamed_38 = 38654706188;
pub const KEYC_WHEELUP1_CONTROL2: C2RustUnnamed_38 = 38654705932;
pub const KEYC_WHEELUP_CONTROL2: C2RustUnnamed_38 = 38654705676;
pub const KEYC_WHEELUP11_CONTROL1: C2RustUnnamed_38 = 38654708491;
pub const KEYC_WHEELUP10_CONTROL1: C2RustUnnamed_38 = 38654708235;
pub const KEYC_WHEELUP9_CONTROL1: C2RustUnnamed_38 = 38654707979;
pub const KEYC_WHEELUP8_CONTROL1: C2RustUnnamed_38 = 38654707723;
pub const KEYC_WHEELUP7_CONTROL1: C2RustUnnamed_38 = 38654707467;
pub const KEYC_WHEELUP6_CONTROL1: C2RustUnnamed_38 = 38654707211;
pub const KEYC_WHEELUP3_CONTROL1: C2RustUnnamed_38 = 38654706443;
pub const KEYC_WHEELUP2_CONTROL1: C2RustUnnamed_38 = 38654706187;
pub const KEYC_WHEELUP1_CONTROL1: C2RustUnnamed_38 = 38654705931;
pub const KEYC_WHEELUP_CONTROL1: C2RustUnnamed_38 = 38654705675;
pub const KEYC_WHEELUP11_CONTROL0: C2RustUnnamed_38 = 38654708490;
pub const KEYC_WHEELUP10_CONTROL0: C2RustUnnamed_38 = 38654708234;
pub const KEYC_WHEELUP9_CONTROL0: C2RustUnnamed_38 = 38654707978;
pub const KEYC_WHEELUP8_CONTROL0: C2RustUnnamed_38 = 38654707722;
pub const KEYC_WHEELUP7_CONTROL0: C2RustUnnamed_38 = 38654707466;
pub const KEYC_WHEELUP6_CONTROL0: C2RustUnnamed_38 = 38654707210;
pub const KEYC_WHEELUP3_CONTROL0: C2RustUnnamed_38 = 38654706442;
pub const KEYC_WHEELUP2_CONTROL0: C2RustUnnamed_38 = 38654706186;
pub const KEYC_WHEELUP1_CONTROL0: C2RustUnnamed_38 = 38654705930;
pub const KEYC_WHEELUP_CONTROL0: C2RustUnnamed_38 = 38654705674;
pub const KEYC_WHEELUP11_EMPTY: C2RustUnnamed_38 = 38654708489;
pub const KEYC_WHEELUP10_EMPTY: C2RustUnnamed_38 = 38654708233;
pub const KEYC_WHEELUP9_EMPTY: C2RustUnnamed_38 = 38654707977;
pub const KEYC_WHEELUP8_EMPTY: C2RustUnnamed_38 = 38654707721;
pub const KEYC_WHEELUP7_EMPTY: C2RustUnnamed_38 = 38654707465;
pub const KEYC_WHEELUP6_EMPTY: C2RustUnnamed_38 = 38654707209;
pub const KEYC_WHEELUP3_EMPTY: C2RustUnnamed_38 = 38654706441;
pub const KEYC_WHEELUP2_EMPTY: C2RustUnnamed_38 = 38654706185;
pub const KEYC_WHEELUP1_EMPTY: C2RustUnnamed_38 = 38654705929;
pub const KEYC_WHEELUP_EMPTY: C2RustUnnamed_38 = 38654705673;
pub const KEYC_WHEELUP11_SCROLLBAR_DOWN: C2RustUnnamed_38 = 38654708488;
pub const KEYC_WHEELUP10_SCROLLBAR_DOWN: C2RustUnnamed_38 = 38654708232;
pub const KEYC_WHEELUP9_SCROLLBAR_DOWN: C2RustUnnamed_38 = 38654707976;
pub const KEYC_WHEELUP8_SCROLLBAR_DOWN: C2RustUnnamed_38 = 38654707720;
pub const KEYC_WHEELUP7_SCROLLBAR_DOWN: C2RustUnnamed_38 = 38654707464;
pub const KEYC_WHEELUP6_SCROLLBAR_DOWN: C2RustUnnamed_38 = 38654707208;
pub const KEYC_WHEELUP3_SCROLLBAR_DOWN: C2RustUnnamed_38 = 38654706440;
pub const KEYC_WHEELUP2_SCROLLBAR_DOWN: C2RustUnnamed_38 = 38654706184;
pub const KEYC_WHEELUP1_SCROLLBAR_DOWN: C2RustUnnamed_38 = 38654705928;
pub const KEYC_WHEELUP_SCROLLBAR_DOWN: C2RustUnnamed_38 = 38654705672;
pub const KEYC_WHEELUP11_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 38654708487;
pub const KEYC_WHEELUP10_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 38654708231;
pub const KEYC_WHEELUP9_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 38654707975;
pub const KEYC_WHEELUP8_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 38654707719;
pub const KEYC_WHEELUP7_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 38654707463;
pub const KEYC_WHEELUP6_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 38654707207;
pub const KEYC_WHEELUP3_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 38654706439;
pub const KEYC_WHEELUP2_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 38654706183;
pub const KEYC_WHEELUP1_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 38654705927;
pub const KEYC_WHEELUP_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 38654705671;
pub const KEYC_WHEELUP11_SCROLLBAR_UP: C2RustUnnamed_38 = 38654708486;
pub const KEYC_WHEELUP10_SCROLLBAR_UP: C2RustUnnamed_38 = 38654708230;
pub const KEYC_WHEELUP9_SCROLLBAR_UP: C2RustUnnamed_38 = 38654707974;
pub const KEYC_WHEELUP8_SCROLLBAR_UP: C2RustUnnamed_38 = 38654707718;
pub const KEYC_WHEELUP7_SCROLLBAR_UP: C2RustUnnamed_38 = 38654707462;
pub const KEYC_WHEELUP6_SCROLLBAR_UP: C2RustUnnamed_38 = 38654707206;
pub const KEYC_WHEELUP3_SCROLLBAR_UP: C2RustUnnamed_38 = 38654706438;
pub const KEYC_WHEELUP2_SCROLLBAR_UP: C2RustUnnamed_38 = 38654706182;
pub const KEYC_WHEELUP1_SCROLLBAR_UP: C2RustUnnamed_38 = 38654705926;
pub const KEYC_WHEELUP_SCROLLBAR_UP: C2RustUnnamed_38 = 38654705670;
pub const KEYC_WHEELUP11_BORDER: C2RustUnnamed_38 = 38654708485;
pub const KEYC_WHEELUP10_BORDER: C2RustUnnamed_38 = 38654708229;
pub const KEYC_WHEELUP9_BORDER: C2RustUnnamed_38 = 38654707973;
pub const KEYC_WHEELUP8_BORDER: C2RustUnnamed_38 = 38654707717;
pub const KEYC_WHEELUP7_BORDER: C2RustUnnamed_38 = 38654707461;
pub const KEYC_WHEELUP6_BORDER: C2RustUnnamed_38 = 38654707205;
pub const KEYC_WHEELUP3_BORDER: C2RustUnnamed_38 = 38654706437;
pub const KEYC_WHEELUP2_BORDER: C2RustUnnamed_38 = 38654706181;
pub const KEYC_WHEELUP1_BORDER: C2RustUnnamed_38 = 38654705925;
pub const KEYC_WHEELUP_BORDER: C2RustUnnamed_38 = 38654705669;
pub const KEYC_WHEELUP11_STATUS_DEFAULT: C2RustUnnamed_38 = 38654708484;
pub const KEYC_WHEELUP10_STATUS_DEFAULT: C2RustUnnamed_38 = 38654708228;
pub const KEYC_WHEELUP9_STATUS_DEFAULT: C2RustUnnamed_38 = 38654707972;
pub const KEYC_WHEELUP8_STATUS_DEFAULT: C2RustUnnamed_38 = 38654707716;
pub const KEYC_WHEELUP7_STATUS_DEFAULT: C2RustUnnamed_38 = 38654707460;
pub const KEYC_WHEELUP6_STATUS_DEFAULT: C2RustUnnamed_38 = 38654707204;
pub const KEYC_WHEELUP3_STATUS_DEFAULT: C2RustUnnamed_38 = 38654706436;
pub const KEYC_WHEELUP2_STATUS_DEFAULT: C2RustUnnamed_38 = 38654706180;
pub const KEYC_WHEELUP1_STATUS_DEFAULT: C2RustUnnamed_38 = 38654705924;
pub const KEYC_WHEELUP_STATUS_DEFAULT: C2RustUnnamed_38 = 38654705668;
pub const KEYC_WHEELUP11_STATUS_RIGHT: C2RustUnnamed_38 = 38654708483;
pub const KEYC_WHEELUP10_STATUS_RIGHT: C2RustUnnamed_38 = 38654708227;
pub const KEYC_WHEELUP9_STATUS_RIGHT: C2RustUnnamed_38 = 38654707971;
pub const KEYC_WHEELUP8_STATUS_RIGHT: C2RustUnnamed_38 = 38654707715;
pub const KEYC_WHEELUP7_STATUS_RIGHT: C2RustUnnamed_38 = 38654707459;
pub const KEYC_WHEELUP6_STATUS_RIGHT: C2RustUnnamed_38 = 38654707203;
pub const KEYC_WHEELUP3_STATUS_RIGHT: C2RustUnnamed_38 = 38654706435;
pub const KEYC_WHEELUP2_STATUS_RIGHT: C2RustUnnamed_38 = 38654706179;
pub const KEYC_WHEELUP1_STATUS_RIGHT: C2RustUnnamed_38 = 38654705923;
pub const KEYC_WHEELUP_STATUS_RIGHT: C2RustUnnamed_38 = 38654705667;
pub const KEYC_WHEELUP11_STATUS_LEFT: C2RustUnnamed_38 = 38654708482;
pub const KEYC_WHEELUP10_STATUS_LEFT: C2RustUnnamed_38 = 38654708226;
pub const KEYC_WHEELUP9_STATUS_LEFT: C2RustUnnamed_38 = 38654707970;
pub const KEYC_WHEELUP8_STATUS_LEFT: C2RustUnnamed_38 = 38654707714;
pub const KEYC_WHEELUP7_STATUS_LEFT: C2RustUnnamed_38 = 38654707458;
pub const KEYC_WHEELUP6_STATUS_LEFT: C2RustUnnamed_38 = 38654707202;
pub const KEYC_WHEELUP3_STATUS_LEFT: C2RustUnnamed_38 = 38654706434;
pub const KEYC_WHEELUP2_STATUS_LEFT: C2RustUnnamed_38 = 38654706178;
pub const KEYC_WHEELUP1_STATUS_LEFT: C2RustUnnamed_38 = 38654705922;
pub const KEYC_WHEELUP_STATUS_LEFT: C2RustUnnamed_38 = 38654705666;
pub const KEYC_WHEELUP11_STATUS: C2RustUnnamed_38 = 38654708481;
pub const KEYC_WHEELUP10_STATUS: C2RustUnnamed_38 = 38654708225;
pub const KEYC_WHEELUP9_STATUS: C2RustUnnamed_38 = 38654707969;
pub const KEYC_WHEELUP8_STATUS: C2RustUnnamed_38 = 38654707713;
pub const KEYC_WHEELUP7_STATUS: C2RustUnnamed_38 = 38654707457;
pub const KEYC_WHEELUP6_STATUS: C2RustUnnamed_38 = 38654707201;
pub const KEYC_WHEELUP3_STATUS: C2RustUnnamed_38 = 38654706433;
pub const KEYC_WHEELUP2_STATUS: C2RustUnnamed_38 = 38654706177;
pub const KEYC_WHEELUP1_STATUS: C2RustUnnamed_38 = 38654705921;
pub const KEYC_WHEELUP_STATUS: C2RustUnnamed_38 = 38654705665;
pub const KEYC_WHEELUP11_PANE: C2RustUnnamed_38 = 38654708480;
pub const KEYC_WHEELUP10_PANE: C2RustUnnamed_38 = 38654708224;
pub const KEYC_WHEELUP9_PANE: C2RustUnnamed_38 = 38654707968;
pub const KEYC_WHEELUP8_PANE: C2RustUnnamed_38 = 38654707712;
pub const KEYC_WHEELUP7_PANE: C2RustUnnamed_38 = 38654707456;
pub const KEYC_WHEELUP6_PANE: C2RustUnnamed_38 = 38654707200;
pub const KEYC_WHEELUP3_PANE: C2RustUnnamed_38 = 38654706432;
pub const KEYC_WHEELUP2_PANE: C2RustUnnamed_38 = 38654706176;
pub const KEYC_WHEELUP1_PANE: C2RustUnnamed_38 = 38654705920;
pub const KEYC_WHEELUP_PANE: C2RustUnnamed_38 = 38654705664;
pub const KEYC_WHEELDOWN11_CONTROL9: C2RustUnnamed_38 = 34359741203;
pub const KEYC_WHEELDOWN10_CONTROL9: C2RustUnnamed_38 = 34359740947;
pub const KEYC_WHEELDOWN9_CONTROL9: C2RustUnnamed_38 = 34359740691;
pub const KEYC_WHEELDOWN8_CONTROL9: C2RustUnnamed_38 = 34359740435;
pub const KEYC_WHEELDOWN7_CONTROL9: C2RustUnnamed_38 = 34359740179;
pub const KEYC_WHEELDOWN6_CONTROL9: C2RustUnnamed_38 = 34359739923;
pub const KEYC_WHEELDOWN3_CONTROL9: C2RustUnnamed_38 = 34359739155;
pub const KEYC_WHEELDOWN2_CONTROL9: C2RustUnnamed_38 = 34359738899;
pub const KEYC_WHEELDOWN1_CONTROL9: C2RustUnnamed_38 = 34359738643;
pub const KEYC_WHEELDOWN_CONTROL9: C2RustUnnamed_38 = 34359738387;
pub const KEYC_WHEELDOWN11_CONTROL8: C2RustUnnamed_38 = 34359741202;
pub const KEYC_WHEELDOWN10_CONTROL8: C2RustUnnamed_38 = 34359740946;
pub const KEYC_WHEELDOWN9_CONTROL8: C2RustUnnamed_38 = 34359740690;
pub const KEYC_WHEELDOWN8_CONTROL8: C2RustUnnamed_38 = 34359740434;
pub const KEYC_WHEELDOWN7_CONTROL8: C2RustUnnamed_38 = 34359740178;
pub const KEYC_WHEELDOWN6_CONTROL8: C2RustUnnamed_38 = 34359739922;
pub const KEYC_WHEELDOWN3_CONTROL8: C2RustUnnamed_38 = 34359739154;
pub const KEYC_WHEELDOWN2_CONTROL8: C2RustUnnamed_38 = 34359738898;
pub const KEYC_WHEELDOWN1_CONTROL8: C2RustUnnamed_38 = 34359738642;
pub const KEYC_WHEELDOWN_CONTROL8: C2RustUnnamed_38 = 34359738386;
pub const KEYC_WHEELDOWN11_CONTROL7: C2RustUnnamed_38 = 34359741201;
pub const KEYC_WHEELDOWN10_CONTROL7: C2RustUnnamed_38 = 34359740945;
pub const KEYC_WHEELDOWN9_CONTROL7: C2RustUnnamed_38 = 34359740689;
pub const KEYC_WHEELDOWN8_CONTROL7: C2RustUnnamed_38 = 34359740433;
pub const KEYC_WHEELDOWN7_CONTROL7: C2RustUnnamed_38 = 34359740177;
pub const KEYC_WHEELDOWN6_CONTROL7: C2RustUnnamed_38 = 34359739921;
pub const KEYC_WHEELDOWN3_CONTROL7: C2RustUnnamed_38 = 34359739153;
pub const KEYC_WHEELDOWN2_CONTROL7: C2RustUnnamed_38 = 34359738897;
pub const KEYC_WHEELDOWN1_CONTROL7: C2RustUnnamed_38 = 34359738641;
pub const KEYC_WHEELDOWN_CONTROL7: C2RustUnnamed_38 = 34359738385;
pub const KEYC_WHEELDOWN11_CONTROL6: C2RustUnnamed_38 = 34359741200;
pub const KEYC_WHEELDOWN10_CONTROL6: C2RustUnnamed_38 = 34359740944;
pub const KEYC_WHEELDOWN9_CONTROL6: C2RustUnnamed_38 = 34359740688;
pub const KEYC_WHEELDOWN8_CONTROL6: C2RustUnnamed_38 = 34359740432;
pub const KEYC_WHEELDOWN7_CONTROL6: C2RustUnnamed_38 = 34359740176;
pub const KEYC_WHEELDOWN6_CONTROL6: C2RustUnnamed_38 = 34359739920;
pub const KEYC_WHEELDOWN3_CONTROL6: C2RustUnnamed_38 = 34359739152;
pub const KEYC_WHEELDOWN2_CONTROL6: C2RustUnnamed_38 = 34359738896;
pub const KEYC_WHEELDOWN1_CONTROL6: C2RustUnnamed_38 = 34359738640;
pub const KEYC_WHEELDOWN_CONTROL6: C2RustUnnamed_38 = 34359738384;
pub const KEYC_WHEELDOWN11_CONTROL5: C2RustUnnamed_38 = 34359741199;
pub const KEYC_WHEELDOWN10_CONTROL5: C2RustUnnamed_38 = 34359740943;
pub const KEYC_WHEELDOWN9_CONTROL5: C2RustUnnamed_38 = 34359740687;
pub const KEYC_WHEELDOWN8_CONTROL5: C2RustUnnamed_38 = 34359740431;
pub const KEYC_WHEELDOWN7_CONTROL5: C2RustUnnamed_38 = 34359740175;
pub const KEYC_WHEELDOWN6_CONTROL5: C2RustUnnamed_38 = 34359739919;
pub const KEYC_WHEELDOWN3_CONTROL5: C2RustUnnamed_38 = 34359739151;
pub const KEYC_WHEELDOWN2_CONTROL5: C2RustUnnamed_38 = 34359738895;
pub const KEYC_WHEELDOWN1_CONTROL5: C2RustUnnamed_38 = 34359738639;
pub const KEYC_WHEELDOWN_CONTROL5: C2RustUnnamed_38 = 34359738383;
pub const KEYC_WHEELDOWN11_CONTROL4: C2RustUnnamed_38 = 34359741198;
pub const KEYC_WHEELDOWN10_CONTROL4: C2RustUnnamed_38 = 34359740942;
pub const KEYC_WHEELDOWN9_CONTROL4: C2RustUnnamed_38 = 34359740686;
pub const KEYC_WHEELDOWN8_CONTROL4: C2RustUnnamed_38 = 34359740430;
pub const KEYC_WHEELDOWN7_CONTROL4: C2RustUnnamed_38 = 34359740174;
pub const KEYC_WHEELDOWN6_CONTROL4: C2RustUnnamed_38 = 34359739918;
pub const KEYC_WHEELDOWN3_CONTROL4: C2RustUnnamed_38 = 34359739150;
pub const KEYC_WHEELDOWN2_CONTROL4: C2RustUnnamed_38 = 34359738894;
pub const KEYC_WHEELDOWN1_CONTROL4: C2RustUnnamed_38 = 34359738638;
pub const KEYC_WHEELDOWN_CONTROL4: C2RustUnnamed_38 = 34359738382;
pub const KEYC_WHEELDOWN11_CONTROL3: C2RustUnnamed_38 = 34359741197;
pub const KEYC_WHEELDOWN10_CONTROL3: C2RustUnnamed_38 = 34359740941;
pub const KEYC_WHEELDOWN9_CONTROL3: C2RustUnnamed_38 = 34359740685;
pub const KEYC_WHEELDOWN8_CONTROL3: C2RustUnnamed_38 = 34359740429;
pub const KEYC_WHEELDOWN7_CONTROL3: C2RustUnnamed_38 = 34359740173;
pub const KEYC_WHEELDOWN6_CONTROL3: C2RustUnnamed_38 = 34359739917;
pub const KEYC_WHEELDOWN3_CONTROL3: C2RustUnnamed_38 = 34359739149;
pub const KEYC_WHEELDOWN2_CONTROL3: C2RustUnnamed_38 = 34359738893;
pub const KEYC_WHEELDOWN1_CONTROL3: C2RustUnnamed_38 = 34359738637;
pub const KEYC_WHEELDOWN_CONTROL3: C2RustUnnamed_38 = 34359738381;
pub const KEYC_WHEELDOWN11_CONTROL2: C2RustUnnamed_38 = 34359741196;
pub const KEYC_WHEELDOWN10_CONTROL2: C2RustUnnamed_38 = 34359740940;
pub const KEYC_WHEELDOWN9_CONTROL2: C2RustUnnamed_38 = 34359740684;
pub const KEYC_WHEELDOWN8_CONTROL2: C2RustUnnamed_38 = 34359740428;
pub const KEYC_WHEELDOWN7_CONTROL2: C2RustUnnamed_38 = 34359740172;
pub const KEYC_WHEELDOWN6_CONTROL2: C2RustUnnamed_38 = 34359739916;
pub const KEYC_WHEELDOWN3_CONTROL2: C2RustUnnamed_38 = 34359739148;
pub const KEYC_WHEELDOWN2_CONTROL2: C2RustUnnamed_38 = 34359738892;
pub const KEYC_WHEELDOWN1_CONTROL2: C2RustUnnamed_38 = 34359738636;
pub const KEYC_WHEELDOWN_CONTROL2: C2RustUnnamed_38 = 34359738380;
pub const KEYC_WHEELDOWN11_CONTROL1: C2RustUnnamed_38 = 34359741195;
pub const KEYC_WHEELDOWN10_CONTROL1: C2RustUnnamed_38 = 34359740939;
pub const KEYC_WHEELDOWN9_CONTROL1: C2RustUnnamed_38 = 34359740683;
pub const KEYC_WHEELDOWN8_CONTROL1: C2RustUnnamed_38 = 34359740427;
pub const KEYC_WHEELDOWN7_CONTROL1: C2RustUnnamed_38 = 34359740171;
pub const KEYC_WHEELDOWN6_CONTROL1: C2RustUnnamed_38 = 34359739915;
pub const KEYC_WHEELDOWN3_CONTROL1: C2RustUnnamed_38 = 34359739147;
pub const KEYC_WHEELDOWN2_CONTROL1: C2RustUnnamed_38 = 34359738891;
pub const KEYC_WHEELDOWN1_CONTROL1: C2RustUnnamed_38 = 34359738635;
pub const KEYC_WHEELDOWN_CONTROL1: C2RustUnnamed_38 = 34359738379;
pub const KEYC_WHEELDOWN11_CONTROL0: C2RustUnnamed_38 = 34359741194;
pub const KEYC_WHEELDOWN10_CONTROL0: C2RustUnnamed_38 = 34359740938;
pub const KEYC_WHEELDOWN9_CONTROL0: C2RustUnnamed_38 = 34359740682;
pub const KEYC_WHEELDOWN8_CONTROL0: C2RustUnnamed_38 = 34359740426;
pub const KEYC_WHEELDOWN7_CONTROL0: C2RustUnnamed_38 = 34359740170;
pub const KEYC_WHEELDOWN6_CONTROL0: C2RustUnnamed_38 = 34359739914;
pub const KEYC_WHEELDOWN3_CONTROL0: C2RustUnnamed_38 = 34359739146;
pub const KEYC_WHEELDOWN2_CONTROL0: C2RustUnnamed_38 = 34359738890;
pub const KEYC_WHEELDOWN1_CONTROL0: C2RustUnnamed_38 = 34359738634;
pub const KEYC_WHEELDOWN_CONTROL0: C2RustUnnamed_38 = 34359738378;
pub const KEYC_WHEELDOWN11_EMPTY: C2RustUnnamed_38 = 34359741193;
pub const KEYC_WHEELDOWN10_EMPTY: C2RustUnnamed_38 = 34359740937;
pub const KEYC_WHEELDOWN9_EMPTY: C2RustUnnamed_38 = 34359740681;
pub const KEYC_WHEELDOWN8_EMPTY: C2RustUnnamed_38 = 34359740425;
pub const KEYC_WHEELDOWN7_EMPTY: C2RustUnnamed_38 = 34359740169;
pub const KEYC_WHEELDOWN6_EMPTY: C2RustUnnamed_38 = 34359739913;
pub const KEYC_WHEELDOWN3_EMPTY: C2RustUnnamed_38 = 34359739145;
pub const KEYC_WHEELDOWN2_EMPTY: C2RustUnnamed_38 = 34359738889;
pub const KEYC_WHEELDOWN1_EMPTY: C2RustUnnamed_38 = 34359738633;
pub const KEYC_WHEELDOWN_EMPTY: C2RustUnnamed_38 = 34359738377;
pub const KEYC_WHEELDOWN11_SCROLLBAR_DOWN: C2RustUnnamed_38 = 34359741192;
pub const KEYC_WHEELDOWN10_SCROLLBAR_DOWN: C2RustUnnamed_38 = 34359740936;
pub const KEYC_WHEELDOWN9_SCROLLBAR_DOWN: C2RustUnnamed_38 = 34359740680;
pub const KEYC_WHEELDOWN8_SCROLLBAR_DOWN: C2RustUnnamed_38 = 34359740424;
pub const KEYC_WHEELDOWN7_SCROLLBAR_DOWN: C2RustUnnamed_38 = 34359740168;
pub const KEYC_WHEELDOWN6_SCROLLBAR_DOWN: C2RustUnnamed_38 = 34359739912;
pub const KEYC_WHEELDOWN3_SCROLLBAR_DOWN: C2RustUnnamed_38 = 34359739144;
pub const KEYC_WHEELDOWN2_SCROLLBAR_DOWN: C2RustUnnamed_38 = 34359738888;
pub const KEYC_WHEELDOWN1_SCROLLBAR_DOWN: C2RustUnnamed_38 = 34359738632;
pub const KEYC_WHEELDOWN_SCROLLBAR_DOWN: C2RustUnnamed_38 = 34359738376;
pub const KEYC_WHEELDOWN11_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 34359741191;
pub const KEYC_WHEELDOWN10_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 34359740935;
pub const KEYC_WHEELDOWN9_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 34359740679;
pub const KEYC_WHEELDOWN8_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 34359740423;
pub const KEYC_WHEELDOWN7_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 34359740167;
pub const KEYC_WHEELDOWN6_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 34359739911;
pub const KEYC_WHEELDOWN3_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 34359739143;
pub const KEYC_WHEELDOWN2_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 34359738887;
pub const KEYC_WHEELDOWN1_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 34359738631;
pub const KEYC_WHEELDOWN_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 34359738375;
pub const KEYC_WHEELDOWN11_SCROLLBAR_UP: C2RustUnnamed_38 = 34359741190;
pub const KEYC_WHEELDOWN10_SCROLLBAR_UP: C2RustUnnamed_38 = 34359740934;
pub const KEYC_WHEELDOWN9_SCROLLBAR_UP: C2RustUnnamed_38 = 34359740678;
pub const KEYC_WHEELDOWN8_SCROLLBAR_UP: C2RustUnnamed_38 = 34359740422;
pub const KEYC_WHEELDOWN7_SCROLLBAR_UP: C2RustUnnamed_38 = 34359740166;
pub const KEYC_WHEELDOWN6_SCROLLBAR_UP: C2RustUnnamed_38 = 34359739910;
pub const KEYC_WHEELDOWN3_SCROLLBAR_UP: C2RustUnnamed_38 = 34359739142;
pub const KEYC_WHEELDOWN2_SCROLLBAR_UP: C2RustUnnamed_38 = 34359738886;
pub const KEYC_WHEELDOWN1_SCROLLBAR_UP: C2RustUnnamed_38 = 34359738630;
pub const KEYC_WHEELDOWN_SCROLLBAR_UP: C2RustUnnamed_38 = 34359738374;
pub const KEYC_WHEELDOWN11_BORDER: C2RustUnnamed_38 = 34359741189;
pub const KEYC_WHEELDOWN10_BORDER: C2RustUnnamed_38 = 34359740933;
pub const KEYC_WHEELDOWN9_BORDER: C2RustUnnamed_38 = 34359740677;
pub const KEYC_WHEELDOWN8_BORDER: C2RustUnnamed_38 = 34359740421;
pub const KEYC_WHEELDOWN7_BORDER: C2RustUnnamed_38 = 34359740165;
pub const KEYC_WHEELDOWN6_BORDER: C2RustUnnamed_38 = 34359739909;
pub const KEYC_WHEELDOWN3_BORDER: C2RustUnnamed_38 = 34359739141;
pub const KEYC_WHEELDOWN2_BORDER: C2RustUnnamed_38 = 34359738885;
pub const KEYC_WHEELDOWN1_BORDER: C2RustUnnamed_38 = 34359738629;
pub const KEYC_WHEELDOWN_BORDER: C2RustUnnamed_38 = 34359738373;
pub const KEYC_WHEELDOWN11_STATUS_DEFAULT: C2RustUnnamed_38 = 34359741188;
pub const KEYC_WHEELDOWN10_STATUS_DEFAULT: C2RustUnnamed_38 = 34359740932;
pub const KEYC_WHEELDOWN9_STATUS_DEFAULT: C2RustUnnamed_38 = 34359740676;
pub const KEYC_WHEELDOWN8_STATUS_DEFAULT: C2RustUnnamed_38 = 34359740420;
pub const KEYC_WHEELDOWN7_STATUS_DEFAULT: C2RustUnnamed_38 = 34359740164;
pub const KEYC_WHEELDOWN6_STATUS_DEFAULT: C2RustUnnamed_38 = 34359739908;
pub const KEYC_WHEELDOWN3_STATUS_DEFAULT: C2RustUnnamed_38 = 34359739140;
pub const KEYC_WHEELDOWN2_STATUS_DEFAULT: C2RustUnnamed_38 = 34359738884;
pub const KEYC_WHEELDOWN1_STATUS_DEFAULT: C2RustUnnamed_38 = 34359738628;
pub const KEYC_WHEELDOWN_STATUS_DEFAULT: C2RustUnnamed_38 = 34359738372;
pub const KEYC_WHEELDOWN11_STATUS_RIGHT: C2RustUnnamed_38 = 34359741187;
pub const KEYC_WHEELDOWN10_STATUS_RIGHT: C2RustUnnamed_38 = 34359740931;
pub const KEYC_WHEELDOWN9_STATUS_RIGHT: C2RustUnnamed_38 = 34359740675;
pub const KEYC_WHEELDOWN8_STATUS_RIGHT: C2RustUnnamed_38 = 34359740419;
pub const KEYC_WHEELDOWN7_STATUS_RIGHT: C2RustUnnamed_38 = 34359740163;
pub const KEYC_WHEELDOWN6_STATUS_RIGHT: C2RustUnnamed_38 = 34359739907;
pub const KEYC_WHEELDOWN3_STATUS_RIGHT: C2RustUnnamed_38 = 34359739139;
pub const KEYC_WHEELDOWN2_STATUS_RIGHT: C2RustUnnamed_38 = 34359738883;
pub const KEYC_WHEELDOWN1_STATUS_RIGHT: C2RustUnnamed_38 = 34359738627;
pub const KEYC_WHEELDOWN_STATUS_RIGHT: C2RustUnnamed_38 = 34359738371;
pub const KEYC_WHEELDOWN11_STATUS_LEFT: C2RustUnnamed_38 = 34359741186;
pub const KEYC_WHEELDOWN10_STATUS_LEFT: C2RustUnnamed_38 = 34359740930;
pub const KEYC_WHEELDOWN9_STATUS_LEFT: C2RustUnnamed_38 = 34359740674;
pub const KEYC_WHEELDOWN8_STATUS_LEFT: C2RustUnnamed_38 = 34359740418;
pub const KEYC_WHEELDOWN7_STATUS_LEFT: C2RustUnnamed_38 = 34359740162;
pub const KEYC_WHEELDOWN6_STATUS_LEFT: C2RustUnnamed_38 = 34359739906;
pub const KEYC_WHEELDOWN3_STATUS_LEFT: C2RustUnnamed_38 = 34359739138;
pub const KEYC_WHEELDOWN2_STATUS_LEFT: C2RustUnnamed_38 = 34359738882;
pub const KEYC_WHEELDOWN1_STATUS_LEFT: C2RustUnnamed_38 = 34359738626;
pub const KEYC_WHEELDOWN_STATUS_LEFT: C2RustUnnamed_38 = 34359738370;
pub const KEYC_WHEELDOWN11_STATUS: C2RustUnnamed_38 = 34359741185;
pub const KEYC_WHEELDOWN10_STATUS: C2RustUnnamed_38 = 34359740929;
pub const KEYC_WHEELDOWN9_STATUS: C2RustUnnamed_38 = 34359740673;
pub const KEYC_WHEELDOWN8_STATUS: C2RustUnnamed_38 = 34359740417;
pub const KEYC_WHEELDOWN7_STATUS: C2RustUnnamed_38 = 34359740161;
pub const KEYC_WHEELDOWN6_STATUS: C2RustUnnamed_38 = 34359739905;
pub const KEYC_WHEELDOWN3_STATUS: C2RustUnnamed_38 = 34359739137;
pub const KEYC_WHEELDOWN2_STATUS: C2RustUnnamed_38 = 34359738881;
pub const KEYC_WHEELDOWN1_STATUS: C2RustUnnamed_38 = 34359738625;
pub const KEYC_WHEELDOWN_STATUS: C2RustUnnamed_38 = 34359738369;
pub const KEYC_WHEELDOWN11_PANE: C2RustUnnamed_38 = 34359741184;
pub const KEYC_WHEELDOWN10_PANE: C2RustUnnamed_38 = 34359740928;
pub const KEYC_WHEELDOWN9_PANE: C2RustUnnamed_38 = 34359740672;
pub const KEYC_WHEELDOWN8_PANE: C2RustUnnamed_38 = 34359740416;
pub const KEYC_WHEELDOWN7_PANE: C2RustUnnamed_38 = 34359740160;
pub const KEYC_WHEELDOWN6_PANE: C2RustUnnamed_38 = 34359739904;
pub const KEYC_WHEELDOWN3_PANE: C2RustUnnamed_38 = 34359739136;
pub const KEYC_WHEELDOWN2_PANE: C2RustUnnamed_38 = 34359738880;
pub const KEYC_WHEELDOWN1_PANE: C2RustUnnamed_38 = 34359738624;
pub const KEYC_WHEELDOWN_PANE: C2RustUnnamed_38 = 34359738368;
pub const KEYC_MOUSEMOVE11_CONTROL9: C2RustUnnamed_38 = 12884904723;
pub const KEYC_MOUSEMOVE10_CONTROL9: C2RustUnnamed_38 = 12884904467;
pub const KEYC_MOUSEMOVE9_CONTROL9: C2RustUnnamed_38 = 12884904211;
pub const KEYC_MOUSEMOVE8_CONTROL9: C2RustUnnamed_38 = 12884903955;
pub const KEYC_MOUSEMOVE7_CONTROL9: C2RustUnnamed_38 = 12884903699;
pub const KEYC_MOUSEMOVE6_CONTROL9: C2RustUnnamed_38 = 12884903443;
pub const KEYC_MOUSEMOVE3_CONTROL9: C2RustUnnamed_38 = 12884902675;
pub const KEYC_MOUSEMOVE2_CONTROL9: C2RustUnnamed_38 = 12884902419;
pub const KEYC_MOUSEMOVE1_CONTROL9: C2RustUnnamed_38 = 12884902163;
pub const KEYC_MOUSEMOVE_CONTROL9: C2RustUnnamed_38 = 12884901907;
pub const KEYC_MOUSEMOVE11_CONTROL8: C2RustUnnamed_38 = 12884904722;
pub const KEYC_MOUSEMOVE10_CONTROL8: C2RustUnnamed_38 = 12884904466;
pub const KEYC_MOUSEMOVE9_CONTROL8: C2RustUnnamed_38 = 12884904210;
pub const KEYC_MOUSEMOVE8_CONTROL8: C2RustUnnamed_38 = 12884903954;
pub const KEYC_MOUSEMOVE7_CONTROL8: C2RustUnnamed_38 = 12884903698;
pub const KEYC_MOUSEMOVE6_CONTROL8: C2RustUnnamed_38 = 12884903442;
pub const KEYC_MOUSEMOVE3_CONTROL8: C2RustUnnamed_38 = 12884902674;
pub const KEYC_MOUSEMOVE2_CONTROL8: C2RustUnnamed_38 = 12884902418;
pub const KEYC_MOUSEMOVE1_CONTROL8: C2RustUnnamed_38 = 12884902162;
pub const KEYC_MOUSEMOVE_CONTROL8: C2RustUnnamed_38 = 12884901906;
pub const KEYC_MOUSEMOVE11_CONTROL7: C2RustUnnamed_38 = 12884904721;
pub const KEYC_MOUSEMOVE10_CONTROL7: C2RustUnnamed_38 = 12884904465;
pub const KEYC_MOUSEMOVE9_CONTROL7: C2RustUnnamed_38 = 12884904209;
pub const KEYC_MOUSEMOVE8_CONTROL7: C2RustUnnamed_38 = 12884903953;
pub const KEYC_MOUSEMOVE7_CONTROL7: C2RustUnnamed_38 = 12884903697;
pub const KEYC_MOUSEMOVE6_CONTROL7: C2RustUnnamed_38 = 12884903441;
pub const KEYC_MOUSEMOVE3_CONTROL7: C2RustUnnamed_38 = 12884902673;
pub const KEYC_MOUSEMOVE2_CONTROL7: C2RustUnnamed_38 = 12884902417;
pub const KEYC_MOUSEMOVE1_CONTROL7: C2RustUnnamed_38 = 12884902161;
pub const KEYC_MOUSEMOVE_CONTROL7: C2RustUnnamed_38 = 12884901905;
pub const KEYC_MOUSEMOVE11_CONTROL6: C2RustUnnamed_38 = 12884904720;
pub const KEYC_MOUSEMOVE10_CONTROL6: C2RustUnnamed_38 = 12884904464;
pub const KEYC_MOUSEMOVE9_CONTROL6: C2RustUnnamed_38 = 12884904208;
pub const KEYC_MOUSEMOVE8_CONTROL6: C2RustUnnamed_38 = 12884903952;
pub const KEYC_MOUSEMOVE7_CONTROL6: C2RustUnnamed_38 = 12884903696;
pub const KEYC_MOUSEMOVE6_CONTROL6: C2RustUnnamed_38 = 12884903440;
pub const KEYC_MOUSEMOVE3_CONTROL6: C2RustUnnamed_38 = 12884902672;
pub const KEYC_MOUSEMOVE2_CONTROL6: C2RustUnnamed_38 = 12884902416;
pub const KEYC_MOUSEMOVE1_CONTROL6: C2RustUnnamed_38 = 12884902160;
pub const KEYC_MOUSEMOVE_CONTROL6: C2RustUnnamed_38 = 12884901904;
pub const KEYC_MOUSEMOVE11_CONTROL5: C2RustUnnamed_38 = 12884904719;
pub const KEYC_MOUSEMOVE10_CONTROL5: C2RustUnnamed_38 = 12884904463;
pub const KEYC_MOUSEMOVE9_CONTROL5: C2RustUnnamed_38 = 12884904207;
pub const KEYC_MOUSEMOVE8_CONTROL5: C2RustUnnamed_38 = 12884903951;
pub const KEYC_MOUSEMOVE7_CONTROL5: C2RustUnnamed_38 = 12884903695;
pub const KEYC_MOUSEMOVE6_CONTROL5: C2RustUnnamed_38 = 12884903439;
pub const KEYC_MOUSEMOVE3_CONTROL5: C2RustUnnamed_38 = 12884902671;
pub const KEYC_MOUSEMOVE2_CONTROL5: C2RustUnnamed_38 = 12884902415;
pub const KEYC_MOUSEMOVE1_CONTROL5: C2RustUnnamed_38 = 12884902159;
pub const KEYC_MOUSEMOVE_CONTROL5: C2RustUnnamed_38 = 12884901903;
pub const KEYC_MOUSEMOVE11_CONTROL4: C2RustUnnamed_38 = 12884904718;
pub const KEYC_MOUSEMOVE10_CONTROL4: C2RustUnnamed_38 = 12884904462;
pub const KEYC_MOUSEMOVE9_CONTROL4: C2RustUnnamed_38 = 12884904206;
pub const KEYC_MOUSEMOVE8_CONTROL4: C2RustUnnamed_38 = 12884903950;
pub const KEYC_MOUSEMOVE7_CONTROL4: C2RustUnnamed_38 = 12884903694;
pub const KEYC_MOUSEMOVE6_CONTROL4: C2RustUnnamed_38 = 12884903438;
pub const KEYC_MOUSEMOVE3_CONTROL4: C2RustUnnamed_38 = 12884902670;
pub const KEYC_MOUSEMOVE2_CONTROL4: C2RustUnnamed_38 = 12884902414;
pub const KEYC_MOUSEMOVE1_CONTROL4: C2RustUnnamed_38 = 12884902158;
pub const KEYC_MOUSEMOVE_CONTROL4: C2RustUnnamed_38 = 12884901902;
pub const KEYC_MOUSEMOVE11_CONTROL3: C2RustUnnamed_38 = 12884904717;
pub const KEYC_MOUSEMOVE10_CONTROL3: C2RustUnnamed_38 = 12884904461;
pub const KEYC_MOUSEMOVE9_CONTROL3: C2RustUnnamed_38 = 12884904205;
pub const KEYC_MOUSEMOVE8_CONTROL3: C2RustUnnamed_38 = 12884903949;
pub const KEYC_MOUSEMOVE7_CONTROL3: C2RustUnnamed_38 = 12884903693;
pub const KEYC_MOUSEMOVE6_CONTROL3: C2RustUnnamed_38 = 12884903437;
pub const KEYC_MOUSEMOVE3_CONTROL3: C2RustUnnamed_38 = 12884902669;
pub const KEYC_MOUSEMOVE2_CONTROL3: C2RustUnnamed_38 = 12884902413;
pub const KEYC_MOUSEMOVE1_CONTROL3: C2RustUnnamed_38 = 12884902157;
pub const KEYC_MOUSEMOVE_CONTROL3: C2RustUnnamed_38 = 12884901901;
pub const KEYC_MOUSEMOVE11_CONTROL2: C2RustUnnamed_38 = 12884904716;
pub const KEYC_MOUSEMOVE10_CONTROL2: C2RustUnnamed_38 = 12884904460;
pub const KEYC_MOUSEMOVE9_CONTROL2: C2RustUnnamed_38 = 12884904204;
pub const KEYC_MOUSEMOVE8_CONTROL2: C2RustUnnamed_38 = 12884903948;
pub const KEYC_MOUSEMOVE7_CONTROL2: C2RustUnnamed_38 = 12884903692;
pub const KEYC_MOUSEMOVE6_CONTROL2: C2RustUnnamed_38 = 12884903436;
pub const KEYC_MOUSEMOVE3_CONTROL2: C2RustUnnamed_38 = 12884902668;
pub const KEYC_MOUSEMOVE2_CONTROL2: C2RustUnnamed_38 = 12884902412;
pub const KEYC_MOUSEMOVE1_CONTROL2: C2RustUnnamed_38 = 12884902156;
pub const KEYC_MOUSEMOVE_CONTROL2: C2RustUnnamed_38 = 12884901900;
pub const KEYC_MOUSEMOVE11_CONTROL1: C2RustUnnamed_38 = 12884904715;
pub const KEYC_MOUSEMOVE10_CONTROL1: C2RustUnnamed_38 = 12884904459;
pub const KEYC_MOUSEMOVE9_CONTROL1: C2RustUnnamed_38 = 12884904203;
pub const KEYC_MOUSEMOVE8_CONTROL1: C2RustUnnamed_38 = 12884903947;
pub const KEYC_MOUSEMOVE7_CONTROL1: C2RustUnnamed_38 = 12884903691;
pub const KEYC_MOUSEMOVE6_CONTROL1: C2RustUnnamed_38 = 12884903435;
pub const KEYC_MOUSEMOVE3_CONTROL1: C2RustUnnamed_38 = 12884902667;
pub const KEYC_MOUSEMOVE2_CONTROL1: C2RustUnnamed_38 = 12884902411;
pub const KEYC_MOUSEMOVE1_CONTROL1: C2RustUnnamed_38 = 12884902155;
pub const KEYC_MOUSEMOVE_CONTROL1: C2RustUnnamed_38 = 12884901899;
pub const KEYC_MOUSEMOVE11_CONTROL0: C2RustUnnamed_38 = 12884904714;
pub const KEYC_MOUSEMOVE10_CONTROL0: C2RustUnnamed_38 = 12884904458;
pub const KEYC_MOUSEMOVE9_CONTROL0: C2RustUnnamed_38 = 12884904202;
pub const KEYC_MOUSEMOVE8_CONTROL0: C2RustUnnamed_38 = 12884903946;
pub const KEYC_MOUSEMOVE7_CONTROL0: C2RustUnnamed_38 = 12884903690;
pub const KEYC_MOUSEMOVE6_CONTROL0: C2RustUnnamed_38 = 12884903434;
pub const KEYC_MOUSEMOVE3_CONTROL0: C2RustUnnamed_38 = 12884902666;
pub const KEYC_MOUSEMOVE2_CONTROL0: C2RustUnnamed_38 = 12884902410;
pub const KEYC_MOUSEMOVE1_CONTROL0: C2RustUnnamed_38 = 12884902154;
pub const KEYC_MOUSEMOVE_CONTROL0: C2RustUnnamed_38 = 12884901898;
pub const KEYC_MOUSEMOVE11_EMPTY: C2RustUnnamed_38 = 12884904713;
pub const KEYC_MOUSEMOVE10_EMPTY: C2RustUnnamed_38 = 12884904457;
pub const KEYC_MOUSEMOVE9_EMPTY: C2RustUnnamed_38 = 12884904201;
pub const KEYC_MOUSEMOVE8_EMPTY: C2RustUnnamed_38 = 12884903945;
pub const KEYC_MOUSEMOVE7_EMPTY: C2RustUnnamed_38 = 12884903689;
pub const KEYC_MOUSEMOVE6_EMPTY: C2RustUnnamed_38 = 12884903433;
pub const KEYC_MOUSEMOVE3_EMPTY: C2RustUnnamed_38 = 12884902665;
pub const KEYC_MOUSEMOVE2_EMPTY: C2RustUnnamed_38 = 12884902409;
pub const KEYC_MOUSEMOVE1_EMPTY: C2RustUnnamed_38 = 12884902153;
pub const KEYC_MOUSEMOVE_EMPTY: C2RustUnnamed_38 = 12884901897;
pub const KEYC_MOUSEMOVE11_SCROLLBAR_DOWN: C2RustUnnamed_38 = 12884904712;
pub const KEYC_MOUSEMOVE10_SCROLLBAR_DOWN: C2RustUnnamed_38 = 12884904456;
pub const KEYC_MOUSEMOVE9_SCROLLBAR_DOWN: C2RustUnnamed_38 = 12884904200;
pub const KEYC_MOUSEMOVE8_SCROLLBAR_DOWN: C2RustUnnamed_38 = 12884903944;
pub const KEYC_MOUSEMOVE7_SCROLLBAR_DOWN: C2RustUnnamed_38 = 12884903688;
pub const KEYC_MOUSEMOVE6_SCROLLBAR_DOWN: C2RustUnnamed_38 = 12884903432;
pub const KEYC_MOUSEMOVE3_SCROLLBAR_DOWN: C2RustUnnamed_38 = 12884902664;
pub const KEYC_MOUSEMOVE2_SCROLLBAR_DOWN: C2RustUnnamed_38 = 12884902408;
pub const KEYC_MOUSEMOVE1_SCROLLBAR_DOWN: C2RustUnnamed_38 = 12884902152;
pub const KEYC_MOUSEMOVE_SCROLLBAR_DOWN: C2RustUnnamed_38 = 12884901896;
pub const KEYC_MOUSEMOVE11_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 12884904711;
pub const KEYC_MOUSEMOVE10_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 12884904455;
pub const KEYC_MOUSEMOVE9_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 12884904199;
pub const KEYC_MOUSEMOVE8_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 12884903943;
pub const KEYC_MOUSEMOVE7_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 12884903687;
pub const KEYC_MOUSEMOVE6_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 12884903431;
pub const KEYC_MOUSEMOVE3_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 12884902663;
pub const KEYC_MOUSEMOVE2_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 12884902407;
pub const KEYC_MOUSEMOVE1_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 12884902151;
pub const KEYC_MOUSEMOVE_SCROLLBAR_SLIDER: C2RustUnnamed_38 = 12884901895;
pub const KEYC_MOUSEMOVE11_SCROLLBAR_UP: C2RustUnnamed_38 = 12884904710;
pub const KEYC_MOUSEMOVE10_SCROLLBAR_UP: C2RustUnnamed_38 = 12884904454;
pub const KEYC_MOUSEMOVE9_SCROLLBAR_UP: C2RustUnnamed_38 = 12884904198;
pub const KEYC_MOUSEMOVE8_SCROLLBAR_UP: C2RustUnnamed_38 = 12884903942;
pub const KEYC_MOUSEMOVE7_SCROLLBAR_UP: C2RustUnnamed_38 = 12884903686;
pub const KEYC_MOUSEMOVE6_SCROLLBAR_UP: C2RustUnnamed_38 = 12884903430;
pub const KEYC_MOUSEMOVE3_SCROLLBAR_UP: C2RustUnnamed_38 = 12884902662;
pub const KEYC_MOUSEMOVE2_SCROLLBAR_UP: C2RustUnnamed_38 = 12884902406;
pub const KEYC_MOUSEMOVE1_SCROLLBAR_UP: C2RustUnnamed_38 = 12884902150;
pub const KEYC_MOUSEMOVE_SCROLLBAR_UP: C2RustUnnamed_38 = 12884901894;
pub const KEYC_MOUSEMOVE11_BORDER: C2RustUnnamed_38 = 12884904709;
pub const KEYC_MOUSEMOVE10_BORDER: C2RustUnnamed_38 = 12884904453;
pub const KEYC_MOUSEMOVE9_BORDER: C2RustUnnamed_38 = 12884904197;
pub const KEYC_MOUSEMOVE8_BORDER: C2RustUnnamed_38 = 12884903941;
pub const KEYC_MOUSEMOVE7_BORDER: C2RustUnnamed_38 = 12884903685;
pub const KEYC_MOUSEMOVE6_BORDER: C2RustUnnamed_38 = 12884903429;
pub const KEYC_MOUSEMOVE3_BORDER: C2RustUnnamed_38 = 12884902661;
pub const KEYC_MOUSEMOVE2_BORDER: C2RustUnnamed_38 = 12884902405;
pub const KEYC_MOUSEMOVE1_BORDER: C2RustUnnamed_38 = 12884902149;
pub const KEYC_MOUSEMOVE_BORDER: C2RustUnnamed_38 = 12884901893;
pub const KEYC_MOUSEMOVE11_STATUS_DEFAULT: C2RustUnnamed_38 = 12884904708;
pub const KEYC_MOUSEMOVE10_STATUS_DEFAULT: C2RustUnnamed_38 = 12884904452;
pub const KEYC_MOUSEMOVE9_STATUS_DEFAULT: C2RustUnnamed_38 = 12884904196;
pub const KEYC_MOUSEMOVE8_STATUS_DEFAULT: C2RustUnnamed_38 = 12884903940;
pub const KEYC_MOUSEMOVE7_STATUS_DEFAULT: C2RustUnnamed_38 = 12884903684;
pub const KEYC_MOUSEMOVE6_STATUS_DEFAULT: C2RustUnnamed_38 = 12884903428;
pub const KEYC_MOUSEMOVE3_STATUS_DEFAULT: C2RustUnnamed_38 = 12884902660;
pub const KEYC_MOUSEMOVE2_STATUS_DEFAULT: C2RustUnnamed_38 = 12884902404;
pub const KEYC_MOUSEMOVE1_STATUS_DEFAULT: C2RustUnnamed_38 = 12884902148;
pub const KEYC_MOUSEMOVE_STATUS_DEFAULT: C2RustUnnamed_38 = 12884901892;
pub const KEYC_MOUSEMOVE11_STATUS_RIGHT: C2RustUnnamed_38 = 12884904707;
pub const KEYC_MOUSEMOVE10_STATUS_RIGHT: C2RustUnnamed_38 = 12884904451;
pub const KEYC_MOUSEMOVE9_STATUS_RIGHT: C2RustUnnamed_38 = 12884904195;
pub const KEYC_MOUSEMOVE8_STATUS_RIGHT: C2RustUnnamed_38 = 12884903939;
pub const KEYC_MOUSEMOVE7_STATUS_RIGHT: C2RustUnnamed_38 = 12884903683;
pub const KEYC_MOUSEMOVE6_STATUS_RIGHT: C2RustUnnamed_38 = 12884903427;
pub const KEYC_MOUSEMOVE3_STATUS_RIGHT: C2RustUnnamed_38 = 12884902659;
pub const KEYC_MOUSEMOVE2_STATUS_RIGHT: C2RustUnnamed_38 = 12884902403;
pub const KEYC_MOUSEMOVE1_STATUS_RIGHT: C2RustUnnamed_38 = 12884902147;
pub const KEYC_MOUSEMOVE_STATUS_RIGHT: C2RustUnnamed_38 = 12884901891;
pub const KEYC_MOUSEMOVE11_STATUS_LEFT: C2RustUnnamed_38 = 12884904706;
pub const KEYC_MOUSEMOVE10_STATUS_LEFT: C2RustUnnamed_38 = 12884904450;
pub const KEYC_MOUSEMOVE9_STATUS_LEFT: C2RustUnnamed_38 = 12884904194;
pub const KEYC_MOUSEMOVE8_STATUS_LEFT: C2RustUnnamed_38 = 12884903938;
pub const KEYC_MOUSEMOVE7_STATUS_LEFT: C2RustUnnamed_38 = 12884903682;
pub const KEYC_MOUSEMOVE6_STATUS_LEFT: C2RustUnnamed_38 = 12884903426;
pub const KEYC_MOUSEMOVE3_STATUS_LEFT: C2RustUnnamed_38 = 12884902658;
pub const KEYC_MOUSEMOVE2_STATUS_LEFT: C2RustUnnamed_38 = 12884902402;
pub const KEYC_MOUSEMOVE1_STATUS_LEFT: C2RustUnnamed_38 = 12884902146;
pub const KEYC_MOUSEMOVE_STATUS_LEFT: C2RustUnnamed_38 = 12884901890;
pub const KEYC_MOUSEMOVE11_STATUS: C2RustUnnamed_38 = 12884904705;
pub const KEYC_MOUSEMOVE10_STATUS: C2RustUnnamed_38 = 12884904449;
pub const KEYC_MOUSEMOVE9_STATUS: C2RustUnnamed_38 = 12884904193;
pub const KEYC_MOUSEMOVE8_STATUS: C2RustUnnamed_38 = 12884903937;
pub const KEYC_MOUSEMOVE7_STATUS: C2RustUnnamed_38 = 12884903681;
pub const KEYC_MOUSEMOVE6_STATUS: C2RustUnnamed_38 = 12884903425;
pub const KEYC_MOUSEMOVE3_STATUS: C2RustUnnamed_38 = 12884902657;
pub const KEYC_MOUSEMOVE2_STATUS: C2RustUnnamed_38 = 12884902401;
pub const KEYC_MOUSEMOVE1_STATUS: C2RustUnnamed_38 = 12884902145;
pub const KEYC_MOUSEMOVE_STATUS: C2RustUnnamed_38 = 12884901889;
pub const KEYC_MOUSEMOVE11_PANE: C2RustUnnamed_38 = 12884904704;
pub const KEYC_MOUSEMOVE10_PANE: C2RustUnnamed_38 = 12884904448;
pub const KEYC_MOUSEMOVE9_PANE: C2RustUnnamed_38 = 12884904192;
pub const KEYC_MOUSEMOVE8_PANE: C2RustUnnamed_38 = 12884903936;
pub const KEYC_MOUSEMOVE7_PANE: C2RustUnnamed_38 = 12884903680;
pub const KEYC_MOUSEMOVE6_PANE: C2RustUnnamed_38 = 12884903424;
pub const KEYC_MOUSEMOVE3_PANE: C2RustUnnamed_38 = 12884902656;
pub const KEYC_MOUSEMOVE2_PANE: C2RustUnnamed_38 = 12884902400;
pub const KEYC_MOUSEMOVE1_PANE: C2RustUnnamed_38 = 12884902144;
pub const KEYC_MOUSEMOVE_PANE: C2RustUnnamed_38 = 12884901888;
pub const KEYC_DOUBLECLICK: C2RustUnnamed_38 = 8589934643;
pub const KEYC_DRAGGING: C2RustUnnamed_38 = 8589934642;
pub const KEYC_MOUSE: C2RustUnnamed_38 = 8589934641;
pub const KEYC_REPORT_LIGHT_THEME: C2RustUnnamed_38 = 8589934640;
pub const KEYC_REPORT_DARK_THEME: C2RustUnnamed_38 = 8589934639;
pub const KEYC_KP_PERIOD: C2RustUnnamed_38 = 8589934638;
pub const KEYC_KP_ZERO: C2RustUnnamed_38 = 8589934637;
pub const KEYC_KP_ENTER: C2RustUnnamed_38 = 8589934636;
pub const KEYC_KP_THREE: C2RustUnnamed_38 = 8589934635;
pub const KEYC_KP_TWO: C2RustUnnamed_38 = 8589934634;
pub const KEYC_KP_ONE: C2RustUnnamed_38 = 8589934633;
pub const KEYC_KP_SIX: C2RustUnnamed_38 = 8589934632;
pub const KEYC_KP_FIVE: C2RustUnnamed_38 = 8589934631;
pub const KEYC_KP_FOUR: C2RustUnnamed_38 = 8589934630;
pub const KEYC_KP_PLUS: C2RustUnnamed_38 = 8589934629;
pub const KEYC_KP_NINE: C2RustUnnamed_38 = 8589934628;
pub const KEYC_KP_EIGHT: C2RustUnnamed_38 = 8589934627;
pub const KEYC_KP_SEVEN: C2RustUnnamed_38 = 8589934626;
pub const KEYC_KP_MINUS: C2RustUnnamed_38 = 8589934625;
pub const KEYC_KP_STAR: C2RustUnnamed_38 = 8589934624;
pub const KEYC_KP_SLASH: C2RustUnnamed_38 = 8589934623;
pub const KEYC_RIGHT: C2RustUnnamed_38 = 8589934622;
pub const KEYC_LEFT: C2RustUnnamed_38 = 8589934621;
pub const KEYC_DOWN: C2RustUnnamed_38 = 8589934620;
pub const KEYC_UP: C2RustUnnamed_38 = 8589934619;
pub const KEYC_BTAB: C2RustUnnamed_38 = 8589934618;
pub const KEYC_PPAGE: C2RustUnnamed_38 = 8589934617;
pub const KEYC_NPAGE: C2RustUnnamed_38 = 8589934616;
pub const KEYC_END: C2RustUnnamed_38 = 8589934615;
pub const KEYC_HOME: C2RustUnnamed_38 = 8589934614;
pub const KEYC_DC: C2RustUnnamed_38 = 8589934613;
pub const KEYC_IC: C2RustUnnamed_38 = 8589934612;
pub const KEYC_F12: C2RustUnnamed_38 = 8589934611;
pub const KEYC_F11: C2RustUnnamed_38 = 8589934610;
pub const KEYC_F10: C2RustUnnamed_38 = 8589934609;
pub const KEYC_F9: C2RustUnnamed_38 = 8589934608;
pub const KEYC_F8: C2RustUnnamed_38 = 8589934607;
pub const KEYC_F7: C2RustUnnamed_38 = 8589934606;
pub const KEYC_F6: C2RustUnnamed_38 = 8589934605;
pub const KEYC_F5: C2RustUnnamed_38 = 8589934604;
pub const KEYC_F4: C2RustUnnamed_38 = 8589934603;
pub const KEYC_F3: C2RustUnnamed_38 = 8589934602;
pub const KEYC_F2: C2RustUnnamed_38 = 8589934601;
pub const KEYC_F1: C2RustUnnamed_38 = 8589934600;
pub const KEYC_BSPACE: C2RustUnnamed_38 = 8589934599;
pub const KEYC_PASTE_END: C2RustUnnamed_38 = 8589934598;
pub const KEYC_PASTE_START: C2RustUnnamed_38 = 8589934597;
pub const KEYC_ANY: C2RustUnnamed_38 = 8589934596;
pub const KEYC_FOCUS_OUT: C2RustUnnamed_38 = 8589934595;
pub const KEYC_FOCUS_IN: C2RustUnnamed_38 = 8589934594;
pub const KEYC_UNKNOWN: C2RustUnnamed_38 = 8589934593;
pub const KEYC_NONE: C2RustUnnamed_38 = 8589934592;
pub const KEYC_USER: C2RustUnnamed_38 = 4294967296;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_pane_tree {
    pub rbh_root: *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct windows {
    pub rbh_root: *mut window,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sessions {
    pub rbh_root: *mut session,
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct clients {
    pub tqh_first: *mut client,
    pub tqh_last: *mut *mut client,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_name_map {
    pub from: *const ::core::ffi::c_char,
    pub to: *const ::core::ffi::c_char,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const PANE_CHANGED: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const PANE_STYLECHANGED: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const PANE_THEMECHANGED: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const TTY_OPENED: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_SERVER: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_SESSION: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_WINDOW: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_PANE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_IS_ARRAY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_IS_STYLE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_IS_COLOUR: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
unsafe extern "C" fn options_array_key_to_number(
    mut key: *const ::core::ffi::c_char,
    mut idx: *mut u_int,
) -> ::core::ffi::c_int {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: ::core::ffi::c_longlong = 0;
    if *key as ::core::ffi::c_int == '\0' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    let mut cp: *const ::core::ffi::c_char = key;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if *(*__ctype_b_loc()).offset(*cp as u_char as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            == 0
        {
            return 0 as ::core::ffi::c_int;
        }
        cp = cp.offset(1);
    }
    n = strtonum(
        key,
        0 as ::core::ffi::c_longlong,
        UINT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    );
    if !errstr.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if !idx.is_null() {
        *idx = n as u_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn options_array_correct_key(
    mut key: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut idx: u_int = 0;
    let mut numeric: ::core::ffi::c_int = 0;
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    numeric = options_array_key_to_number(key, &raw mut idx);
    if numeric == -(1 as ::core::ffi::c_int) {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if numeric == 1 as ::core::ffi::c_int {
        xasprintf(
            &raw mut out,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            idx,
        );
        return out;
    }
    return xstrdup(key);
}
unsafe extern "C" fn options_array_cmp(
    mut a1: *mut options_array_item,
    mut a2: *mut options_array_item,
) -> ::core::ffi::c_int {
    let mut i1: u_int = 0;
    let mut i2: u_int = 0;
    let mut n1: ::core::ffi::c_int = 0;
    let mut n2: ::core::ffi::c_int = 0;
    n1 = options_array_key_to_number((*a1).key, &raw mut i1);
    n2 = options_array_key_to_number((*a2).key, &raw mut i2);
    if n1 != 0 && n2 != 0 {
        if i1 < i2 {
            return -(1 as ::core::ffi::c_int);
        }
        if i1 > i2 {
            return 1 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
    if n1 != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if n2 != 0 {
        return 1 as ::core::ffi::c_int;
    }
    return strcmp((*a1).key, (*a2).key);
}
unsafe extern "C" fn options_array_RB_REMOVE(
    mut head: *mut options_array,
    mut elm: *mut options_array_item,
) -> *mut options_array_item {
    let mut current_block: u64;
    let mut child: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut parent: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut old: *mut options_array_item = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
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
        current_block = 5235237659983604069;
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
        options_array_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn options_array_RB_MINMAX(
    mut head: *mut options_array,
    mut val: ::core::ffi::c_int,
) -> *mut options_array_item {
    let mut tmp: *mut options_array_item = (*head).rbh_root;
    let mut parent: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
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
unsafe extern "C" fn options_array_RB_REMOVE_COLOR(
    mut head: *mut options_array,
    mut parent: *mut options_array_item,
    mut elm: *mut options_array_item,
) {
    let mut tmp: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
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
                    let mut oleft: *mut options_array_item =
                        ::core::ptr::null_mut::<options_array_item>();
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
                    let mut oright: *mut options_array_item =
                        ::core::ptr::null_mut::<options_array_item>();
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
unsafe extern "C" fn options_array_RB_FIND(
    mut head: *mut options_array,
    mut elm: *mut options_array_item,
) -> *mut options_array_item {
    let mut tmp: *mut options_array_item = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = options_array_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<options_array_item>();
}
unsafe extern "C" fn options_array_RB_INSERT(
    mut head: *mut options_array,
    mut elm: *mut options_array_item,
) -> *mut options_array_item {
    let mut tmp: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut parent: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = options_array_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<options_array_item>();
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
    options_array_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<options_array_item>();
}
unsafe extern "C" fn options_array_RB_INSERT_COLOR(
    mut head: *mut options_array,
    mut elm: *mut options_array_item,
) {
    let mut parent: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut gparent: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut tmp: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
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
unsafe extern "C" fn options_array_RB_NEXT(
    mut elm: *mut options_array_item,
) -> *mut options_array_item {
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
unsafe extern "C" fn options_tree_RB_NEXT(mut elm: *mut options_entry) -> *mut options_entry {
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
unsafe extern "C" fn options_tree_RB_MINMAX(
    mut head: *mut options_tree,
    mut val: ::core::ffi::c_int,
) -> *mut options_entry {
    let mut tmp: *mut options_entry = (*head).rbh_root;
    let mut parent: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
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
unsafe extern "C" fn options_tree_RB_INSERT(
    mut head: *mut options_tree,
    mut elm: *mut options_entry,
) -> *mut options_entry {
    let mut tmp: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut parent: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = options_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<options_entry>();
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
    options_tree_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<options_entry>();
}
unsafe extern "C" fn options_tree_RB_REMOVE_COLOR(
    mut head: *mut options_tree,
    mut parent: *mut options_entry,
    mut elm: *mut options_entry,
) {
    let mut tmp: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
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
                    let mut oleft: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
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
                    let mut oright: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
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
unsafe extern "C" fn options_tree_RB_INSERT_COLOR(
    mut head: *mut options_tree,
    mut elm: *mut options_entry,
) {
    let mut parent: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut gparent: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut tmp: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
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
unsafe extern "C" fn options_tree_RB_REMOVE(
    mut head: *mut options_tree,
    mut elm: *mut options_entry,
) -> *mut options_entry {
    let mut current_block: u64;
    let mut child: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut parent: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut old: *mut options_entry = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
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
        current_block = 3778725392729552373;
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
        options_tree_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn options_tree_RB_FIND(
    mut head: *mut options_tree,
    mut elm: *mut options_entry,
) -> *mut options_entry {
    let mut tmp: *mut options_entry = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = options_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<options_entry>();
}
unsafe extern "C" fn options_cmp(
    mut lhs: *mut options_entry,
    mut rhs: *mut options_entry,
) -> ::core::ffi::c_int {
    return strcmp((*lhs).name, (*rhs).name);
}
unsafe extern "C" fn options_map_name(
    mut name: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut map: *const options_name_map = ::core::ptr::null::<options_name_map>();
    map = &raw const options_other_names as *const options_name_map;
    while !(*map).from.is_null() {
        if strcmp((*map).from, name) == 0 as ::core::ffi::c_int {
            return (*map).to;
        }
        map = map.offset(1);
    }
    return name;
}
unsafe extern "C" fn options_parent_table_entry(
    mut oo: *mut options,
    mut s: *const ::core::ffi::c_char,
) -> *const options_table_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if (*oo).parent.is_null() {
        fatalx(
            b"no parent options for %s\0" as *const u8 as *const ::core::ffi::c_char,
            s,
        );
    }
    o = options_get((*oo).parent, s);
    if o.is_null() {
        fatalx(
            b"%s not in parent options\0" as *const u8 as *const ::core::ffi::c_char,
            s,
        );
    }
    return (*o).tableentry;
}
unsafe extern "C" fn options_value_free(mut o: *mut options_entry, mut ov: *mut options_value) {
    if (*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        free((*ov).string as *mut ::core::ffi::c_void);
    }
    if !(*o).tableentry.is_null()
        && (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*ov).cmdlist.is_null()
    {
        cmd_list_free((*ov).cmdlist);
    }
}
unsafe extern "C" fn options_value_to_string(
    mut o: *mut options_entry,
    mut ov: *mut options_value,
    mut numeric: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(*o).tableentry.is_null()
        && (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return cmd_list_print((*ov).cmdlist, 0 as ::core::ffi::c_int);
    }
    if !(*o).tableentry.is_null()
        && ((*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_COLOUR as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        match (*(*o).tableentry).type_0 as ::core::ffi::c_uint {
            1 => {
                xasprintf(
                    &raw mut s,
                    b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                    (*ov).number,
                );
            }
            2 => {
                s = xstrdup(key_string_lookup_key(
                    (*ov).number as key_code,
                    0 as ::core::ffi::c_int,
                ));
            }
            3 => {
                s = xstrdup(colour_tostring((*ov).number as ::core::ffi::c_int));
            }
            4 => {
                if numeric != 0 {
                    xasprintf(
                        &raw mut s,
                        b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                        (*ov).number,
                    );
                } else {
                    s = xstrdup(if (*ov).number != 0 {
                        b"on\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        b"off\0" as *const u8 as *const ::core::ffi::c_char
                    });
                }
            }
            5 => {
                s = xstrdup(*(*(*o).tableentry).choices.offset((*ov).number as isize));
            }
            _ => {
                fatalx(b"not a number option type\0" as *const u8 as *const ::core::ffi::c_char);
            }
        }
        return s;
    }
    if (*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return xstrdup((*ov).string);
    }
    return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn options_create(mut parent: *mut options) -> *mut options {
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    oo = xcalloc(1 as size_t, ::core::mem::size_of::<options>() as size_t) as *mut options;
    (*oo).tree.rbh_root = ::core::ptr::null_mut::<options_entry>();
    (*oo).parent = parent;
    return oo;
}
#[no_mangle]
pub unsafe extern "C" fn options_free(mut oo: *mut options) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut tmp: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_tree_RB_MINMAX(&raw mut (*oo).tree, RB_NEGINF);
    while !o.is_null() && {
        tmp = options_tree_RB_NEXT(o);
        1 as ::core::ffi::c_int != 0
    } {
        options_remove(o);
        o = tmp;
    }
    free(oo as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn options_get_parent(mut oo: *mut options) -> *mut options {
    return (*oo).parent;
}
#[no_mangle]
pub unsafe extern "C" fn options_set_parent(mut oo: *mut options, mut parent: *mut options) {
    (*oo).parent = parent;
}
#[no_mangle]
pub unsafe extern "C" fn options_first(mut oo: *mut options) -> *mut options_entry {
    return options_tree_RB_MINMAX(&raw mut (*oo).tree, RB_NEGINF);
}
#[no_mangle]
pub unsafe extern "C" fn options_next(mut o: *mut options_entry) -> *mut options_entry {
    return options_tree_RB_NEXT(o);
}
#[no_mangle]
pub unsafe extern "C" fn options_get_only(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *mut options_entry {
    let mut o: options_entry = options_entry {
        owner: ::core::ptr::null_mut::<options>(),
        name: name,
        tableentry: ::core::ptr::null::<options_table_entry>(),
        value: options_value {
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        cached: 0,
        style: style {
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
        },
        monitor_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        fire_count: 0,
        fire_time: 0,
        entry: C2RustUnnamed_17 {
            rbe_left: ::core::ptr::null_mut::<options_entry>(),
            rbe_right: ::core::ptr::null_mut::<options_entry>(),
            rbe_parent: ::core::ptr::null_mut::<options_entry>(),
            rbe_color: 0,
        },
    };
    let mut found: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    found = options_tree_RB_FIND(&raw mut (*oo).tree, &raw mut o);
    if found.is_null() {
        o.name = options_map_name(name);
        return options_tree_RB_FIND(&raw mut (*oo).tree, &raw mut o);
    }
    return found;
}
#[no_mangle]
pub unsafe extern "C" fn options_get(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_get_only(oo, name);
    while o.is_null() {
        oo = (*oo).parent;
        if oo.is_null() {
            break;
        }
        o = options_get_only(oo, name);
    }
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_empty(
    mut oo: *mut options,
    mut oe: *const options_table_entry,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_add(oo, (*oe).name);
    (*o).tableentry = oe;
    if (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        (*o).value.array.rbh_root = ::core::ptr::null_mut::<options_array_item>();
    }
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_default(
    mut oo: *mut options,
    mut oe: *const options_table_entry,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut key: [::core::ffi::c_char; 32] = [0; 32];
    let mut i: u_int = 0;
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    o = options_empty(oo, oe);
    ov = &raw mut (*o).value;
    if (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        if (*oe).default_arr.is_null() {
            options_array_assign(
                o,
                (*oe).default_str,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
            return o;
        }
        i = 0 as u_int;
        while !(*(*oe).default_arr.offset(i as isize)).is_null() {
            xsnprintf(
                &raw mut key as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                i,
            );
            options_array_set(
                o,
                &raw mut key as *mut ::core::ffi::c_char,
                *(*oe).default_arr.offset(i as isize),
                0 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
            i = i.wrapping_add(1);
        }
        return o;
    }
    match (*oe).type_0 as ::core::ffi::c_uint {
        0 => {
            (*ov).string = xstrdup((*oe).default_str);
        }
        6 => {
            pr = cmd_parse_from_string(
                (*oe).default_str,
                ::core::ptr::null_mut::<cmd_parse_input>(),
            );
            match (*pr).status as ::core::ffi::c_uint {
                0 => {
                    free((*pr).error as *mut ::core::ffi::c_void);
                }
                1 => {
                    (*ov).cmdlist = (*pr).cmdlist;
                }
                _ => {}
            }
        }
        _ => {
            (*ov).number = (*oe).default_num;
        }
    }
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_default_to_string(
    mut oe: *const options_table_entry,
) -> *mut ::core::ffi::c_char {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    match (*oe).type_0 as ::core::ffi::c_uint {
        0 | 6 => {
            s = xstrdup((*oe).default_str);
        }
        1 => {
            xasprintf(
                &raw mut s,
                b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                (*oe).default_num,
            );
        }
        2 => {
            s = xstrdup(key_string_lookup_key(
                (*oe).default_num as key_code,
                0 as ::core::ffi::c_int,
            ));
        }
        3 => {
            s = xstrdup(colour_tostring((*oe).default_num as ::core::ffi::c_int));
        }
        4 => {
            s = xstrdup(if (*oe).default_num != 0 {
                b"on\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"off\0" as *const u8 as *const ::core::ffi::c_char
            });
        }
        5 => {
            s = xstrdup(*(*oe).choices.offset((*oe).default_num as isize));
        }
        _ => {
            fatalx(b"unknown option type\0" as *const u8 as *const ::core::ffi::c_char);
        }
    }
    return s;
}
unsafe extern "C" fn options_add(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_get_only(oo, name);
    if !o.is_null() {
        options_remove(o);
    }
    o = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<options_entry>() as size_t,
    ) as *mut options_entry;
    (*o).owner = oo;
    (*o).name = xstrdup(name);
    options_tree_RB_INSERT(&raw mut (*oo).tree, o);
    return o;
}
unsafe extern "C" fn options_remove(mut o: *mut options_entry) {
    let mut oo: *mut options = (*o).owner;
    if !(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        options_array_clear(o);
    } else {
        options_value_free(o, &raw mut (*o).value);
    }
    if !(*o).monitor_data.is_null() {
        hooks_monitor_free((*o).monitor_data);
    }
    options_tree_RB_REMOVE(&raw mut (*oo).tree, o);
    free((*o).name as *mut ::core::ffi::c_void);
    free(o as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn options_name(mut o: *mut options_entry) -> *const ::core::ffi::c_char {
    return (*o).name;
}
#[no_mangle]
pub unsafe extern "C" fn options_owner(mut o: *mut options_entry) -> *mut options {
    return (*o).owner;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_monitor_data(
    mut o: *mut options_entry,
) -> *mut ::core::ffi::c_void {
    return (*o).monitor_data;
}
#[no_mangle]
pub unsafe extern "C" fn options_set_monitor_data(
    mut o: *mut options_entry,
    mut data: *mut ::core::ffi::c_void,
) {
    (*o).monitor_data = data;
}
#[no_mangle]
pub unsafe extern "C" fn options_hook_fired(mut o: *mut options_entry) {
    (*o).fire_count = (*o).fire_count.wrapping_add(1);
    (*o).fire_time = current_time;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_fire_count(mut o: *mut options_entry) -> u_int {
    return (*o).fire_count;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_fire_time(mut o: *mut options_entry) -> time_t {
    return (*o).fire_time;
}
#[no_mangle]
pub unsafe extern "C" fn options_table_entry(
    mut o: *mut options_entry,
) -> *const options_table_entry {
    return (*o).tableentry;
}
unsafe extern "C" fn options_array_item(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
) -> *mut options_array_item {
    let mut a: options_array_item = options_array_item {
        key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        value: options_value {
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        entry: C2RustUnnamed_18 {
            rbe_left: ::core::ptr::null_mut::<options_array_item>(),
            rbe_right: ::core::ptr::null_mut::<options_array_item>(),
            rbe_parent: ::core::ptr::null_mut::<options_array_item>(),
            rbe_color: 0,
        },
    };
    a.key = key as *mut ::core::ffi::c_char;
    return options_array_RB_FIND(&raw mut (*o).value.array, &raw mut a);
}
unsafe extern "C" fn options_array_new(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
) -> *mut options_array_item {
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    a = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<options_array_item>() as size_t,
    ) as *mut options_array_item;
    (*a).key = xstrdup(key);
    options_array_RB_INSERT(&raw mut (*o).value.array, a);
    return a;
}
unsafe extern "C" fn options_array_free(mut o: *mut options_entry, mut a: *mut options_array_item) {
    options_value_free(o, &raw mut (*a).value);
    options_array_RB_REMOVE(&raw mut (*o).value.array, a);
    free((*a).key as *mut ::core::ffi::c_void);
    free(a as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn options_array_clear(mut o: *mut options_entry) {
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut a1: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    if !(!(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0) {
        return;
    }
    a = options_array_RB_MINMAX(&raw mut (*o).value.array, RB_NEGINF);
    while !a.is_null() && {
        a1 = options_array_RB_NEXT(a);
        1 as ::core::ffi::c_int != 0
    } {
        options_array_free(o, a);
        a = a1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn options_array_get(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
) -> *mut options_value {
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut new_key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(!(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0) {
        return ::core::ptr::null_mut::<options_value>();
    }
    new_key = options_array_correct_key(key);
    if new_key.is_null() {
        return ::core::ptr::null_mut::<options_value>();
    }
    a = options_array_item(o, new_key);
    free(new_key as *mut ::core::ffi::c_void);
    if a.is_null() {
        return ::core::ptr::null_mut::<options_value>();
    }
    return &raw mut (*a).value;
}
#[no_mangle]
pub unsafe extern "C" fn options_array_getv(
    mut o: *mut options_entry,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> *mut options_value {
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut ap: ::core::ffi::VaList;
    let mut key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ap = args.clone();
    xvasprintf(&raw mut key, fmt, ap);
    ov = options_array_get(o, key);
    free(key as *mut ::core::ffi::c_void);
    return ov;
}
#[no_mangle]
pub unsafe extern "C" fn options_array_set(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut append: ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut new: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut new_key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut number: ::core::ffi::c_longlong = 0;
    if !(!(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0) {
        if !cause.is_null() {
            *cause = xstrdup(b"not an array\0" as *const u8 as *const ::core::ffi::c_char);
        }
        return -(1 as ::core::ffi::c_int);
    }
    new_key = options_array_correct_key(key);
    if new_key.is_null() {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"bad array key: %s\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    if value.is_null() {
        a = options_array_item(o, new_key);
        if !a.is_null() {
            options_array_free(o, a);
        }
        free(new_key as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if !(*o).tableentry.is_null()
        && (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        pr = cmd_parse_from_string(value, ::core::ptr::null_mut::<cmd_parse_input>());
        match (*pr).status as ::core::ffi::c_uint {
            0 => {
                if !cause.is_null() {
                    *cause = (*pr).error;
                } else {
                    free((*pr).error as *mut ::core::ffi::c_void);
                }
                free(new_key as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            1 | _ => {}
        }
        a = options_array_item(o, new_key);
        if a.is_null() {
            a = options_array_new(o, new_key);
        } else {
            options_value_free(o, &raw mut (*a).value);
        }
        (*a).value.cmdlist = (*pr).cmdlist;
        free(new_key as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if (*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        a = options_array_item(o, new_key);
        if !a.is_null() && append != 0 {
            xasprintf(
                &raw mut new,
                b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*a).value.string,
                value,
            );
        } else {
            new = xstrdup(value);
        }
        if a.is_null() {
            a = options_array_new(o, new_key);
        } else {
            options_value_free(o, &raw mut (*a).value);
        }
        (*a).value.string = new;
        free(new_key as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if (*(*o).tableentry).type_0 as ::core::ffi::c_uint
        == OPTIONS_TABLE_COLOUR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        number = colour_fromstring(value) as ::core::ffi::c_longlong;
        if number == -(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong {
            xasprintf(
                cause,
                b"bad colour: %s\0" as *const u8 as *const ::core::ffi::c_char,
                value,
            );
            free(new_key as *mut ::core::ffi::c_void);
            return -(1 as ::core::ffi::c_int);
        }
        a = options_array_item(o, new_key);
        if a.is_null() {
            a = options_array_new(o, new_key);
        } else {
            options_value_free(o, &raw mut (*a).value);
        }
        (*a).value.number = number;
        free(new_key as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if !cause.is_null() {
        *cause = xstrdup(b"wrong array type\0" as *const u8 as *const ::core::ffi::c_char);
    }
    free(new_key as *mut ::core::ffi::c_void);
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn options_array_assign(
    mut o: *mut options_entry,
    mut s: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut separator: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut string: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: [::core::ffi::c_char; 32] = [0; 32];
    let mut i: u_int = 0;
    separator = (*(*o).tableentry).separator;
    if separator.is_null() {
        separator = b" ,\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if *separator as ::core::ffi::c_int == '\0' as i32 {
        if *s as ::core::ffi::c_int == '\0' as i32 {
            return 0 as ::core::ffi::c_int;
        }
        i = 0 as u_int;
        while i < UINT_MAX {
            if options_array_getv(o, b"%u\0" as *const u8 as *const ::core::ffi::c_char, i)
                .is_null()
            {
                break;
            }
            i = i.wrapping_add(1);
        }
        xsnprintf(
            &raw mut key as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        return options_array_set(
            o,
            &raw mut key as *mut ::core::ffi::c_char,
            s,
            0 as ::core::ffi::c_int,
            cause,
        );
    }
    if *s as ::core::ffi::c_int == '\0' as i32 {
        return 0 as ::core::ffi::c_int;
    }
    string = xstrdup(s);
    copy = string;
    loop {
        next = strsep(&raw mut string, separator);
        if next.is_null() {
            break;
        }
        if *next as ::core::ffi::c_int == '\0' as i32 {
            continue;
        }
        i = 0 as u_int;
        while i < UINT_MAX {
            if options_array_getv(o, b"%u\0" as *const u8 as *const ::core::ffi::c_char, i)
                .is_null()
            {
                break;
            }
            i = i.wrapping_add(1);
        }
        if i == UINT_MAX {
            break;
        }
        xsnprintf(
            &raw mut key as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        if options_array_set(
            o,
            &raw mut key as *mut ::core::ffi::c_char,
            next,
            0 as ::core::ffi::c_int,
            cause,
        ) != 0 as ::core::ffi::c_int
        {
            free(copy as *mut ::core::ffi::c_void);
            return -(1 as ::core::ffi::c_int);
        }
    }
    free(copy as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn options_array_first(mut o: *mut options_entry) -> *mut options_array_item {
    if !(!(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0) {
        return ::core::ptr::null_mut::<options_array_item>();
    }
    return options_array_RB_MINMAX(&raw mut (*o).value.array, RB_NEGINF);
}
#[no_mangle]
pub unsafe extern "C" fn options_array_next(
    mut a: *mut options_array_item,
) -> *mut options_array_item {
    return options_array_RB_NEXT(a);
}
#[no_mangle]
pub unsafe extern "C" fn options_array_item_key(
    mut a: *mut options_array_item,
) -> *const ::core::ffi::c_char {
    return (*a).key;
}
#[no_mangle]
pub unsafe extern "C" fn options_array_item_value(
    mut a: *mut options_array_item,
) -> *mut options_value {
    return &raw mut (*a).value;
}
#[no_mangle]
pub unsafe extern "C" fn options_is_array(mut o: *mut options_entry) -> ::core::ffi::c_int {
    return (!(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn options_is_string(mut o: *mut options_entry) -> ::core::ffi::c_int {
    return ((*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn options_to_string(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
    mut numeric: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut result: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut last: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut new_key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        if key.is_null() {
            a = options_array_RB_MINMAX(&raw mut (*o).value.array, RB_NEGINF);
            while !a.is_null() {
                next = options_value_to_string(o, &raw mut (*a).value, numeric);
                if last.is_null() {
                    result = next;
                } else {
                    xasprintf(
                        &raw mut result,
                        b"%s %s\0" as *const u8 as *const ::core::ffi::c_char,
                        last,
                        next,
                    );
                    free(last as *mut ::core::ffi::c_void);
                    free(next as *mut ::core::ffi::c_void);
                }
                last = result;
                a = options_array_RB_NEXT(a);
            }
            if result.is_null() {
                return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
            }
            return result;
        }
        new_key = options_array_correct_key(key);
        if new_key.is_null() {
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        }
        a = options_array_item(o, new_key);
        free(new_key as *mut ::core::ffi::c_void);
        if a.is_null() {
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        }
        return options_value_to_string(o, &raw mut (*a).value, numeric);
    }
    return options_value_to_string(o, &raw mut (*o).value, numeric);
}
#[no_mangle]
pub unsafe extern "C" fn options_parse(
    mut name: *const ::core::ffi::c_char,
    mut key: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut raw: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut new_key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *name as ::core::ffi::c_int == '\0' as i32 {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    *key = ::core::ptr::null_mut::<::core::ffi::c_char>();
    copy = xstrdup(name);
    cp = strchr(copy, '[' as i32);
    if cp.is_null() {
        return copy;
    }
    end = strchr(cp.offset(1 as ::core::ffi::c_int as isize), ']' as i32);
    if end.is_null()
        || *end.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
        || end == cp.offset(1 as ::core::ffi::c_int as isize)
    {
        free(copy as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    raw = xstrndup(
        cp.offset(1 as ::core::ffi::c_int as isize),
        end.offset_from(cp.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_long
            as size_t,
    );
    new_key = options_array_correct_key(raw);
    free(raw as *mut ::core::ffi::c_void);
    if new_key.is_null() {
        free(copy as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    *key = new_key;
    *cp = '\0' as i32 as ::core::ffi::c_char;
    return copy;
}
#[no_mangle]
pub unsafe extern "C" fn options_parse_get(
    mut oo: *mut options,
    mut s: *const ::core::ffi::c_char,
    mut key: *mut *mut ::core::ffi::c_char,
    mut only: ::core::ffi::c_int,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    name = options_parse(s, key);
    if name.is_null() {
        return ::core::ptr::null_mut::<options_entry>();
    }
    if only != 0 {
        o = options_get_only(oo, name);
    } else {
        o = options_get(oo, name);
    }
    free(name as *mut ::core::ffi::c_void);
    if o.is_null() {
        free(*key as *mut ::core::ffi::c_void);
        *key = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_search(
    mut name: *const ::core::ffi::c_char,
) -> *const options_table_entry {
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    oe = &raw const options_table as *const options_table_entry;
    while !(*oe).name.is_null() {
        if strcmp((*oe).name, name) == 0 as ::core::ffi::c_int {
            return oe;
        }
        oe = oe.offset(1);
    }
    return ::core::ptr::null::<options_table_entry>();
}
#[no_mangle]
pub unsafe extern "C" fn options_match(
    mut s: *const ::core::ffi::c_char,
    mut key: *mut *mut ::core::ffi::c_char,
    mut ambiguous: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut found: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut parsed: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut namelen: size_t = 0;
    parsed = options_parse(s, key);
    if parsed.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if *parsed as ::core::ffi::c_int == '@' as i32 {
        *ambiguous = 0 as ::core::ffi::c_int;
        return parsed;
    }
    name = options_map_name(parsed);
    namelen = strlen(name);
    found = ::core::ptr::null::<options_table_entry>();
    oe = &raw const options_table as *const options_table_entry;
    while !(*oe).name.is_null() {
        if strcmp((*oe).name, name) == 0 as ::core::ffi::c_int {
            found = oe;
            break;
        } else {
            if strncmp((*oe).name, name, namelen) == 0 as ::core::ffi::c_int {
                if !found.is_null() {
                    *ambiguous = 1 as ::core::ffi::c_int;
                    free(parsed as *mut ::core::ffi::c_void);
                    free(*key as *mut ::core::ffi::c_void);
                    *key = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                found = oe;
            }
            oe = oe.offset(1);
        }
    }
    free(parsed as *mut ::core::ffi::c_void);
    if found.is_null() {
        *ambiguous = 0 as ::core::ffi::c_int;
        free(*key as *mut ::core::ffi::c_void);
        *key = ::core::ptr::null_mut::<::core::ffi::c_char>();
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return xstrdup((*found).name);
}
#[no_mangle]
pub unsafe extern "C" fn options_match_get(
    mut oo: *mut options,
    mut s: *const ::core::ffi::c_char,
    mut key: *mut *mut ::core::ffi::c_char,
    mut only: ::core::ffi::c_int,
    mut ambiguous: *mut ::core::ffi::c_int,
) -> *mut options_entry {
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    name = options_match(s, key, ambiguous);
    if name.is_null() {
        return ::core::ptr::null_mut::<options_entry>();
    }
    *ambiguous = 0 as ::core::ffi::c_int;
    if only != 0 {
        o = options_get_only(oo, name);
    } else {
        o = options_get(oo, name);
    }
    free(name as *mut ::core::ffi::c_void);
    if o.is_null() {
        free(*key as *mut ::core::ffi::c_void);
        *key = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_string(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_get(oo, name);
    if o.is_null() {
        fatalx(
            b"missing option %s\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    if !((*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        fatalx(
            b"option %s is not a string\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    return (*o).value.string;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_number(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_get(oo, name);
    if o.is_null() {
        fatalx(
            b"missing option %s\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    if !(!(*o).tableentry.is_null()
        && ((*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_COLOUR as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint))
    {
        fatalx(
            b"option %s is not a number\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    return (*o).value.number;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_command(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *mut cmd_list {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_get(oo, name);
    if o.is_null() {
        fatalx(
            b"missing option %s\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    if !(!(*o).tableentry.is_null()
        && (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        fatalx(
            b"option %s is not a command\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    return (*o).value.cmdlist;
}
#[no_mangle]
pub unsafe extern "C" fn options_set_string(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut append: ::core::ffi::c_int,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut ap: ::core::ffi::VaList;
    let mut separator: *const ::core::ffi::c_char =
        b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ap = args.clone();
    xvasprintf(&raw mut s, fmt, ap);
    o = options_get_only(oo, name);
    if !o.is_null()
        && append != 0
        && ((*o).tableentry.is_null()
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        if *name as ::core::ffi::c_int != '@' as i32 {
            separator = (*(*o).tableentry).separator;
            if separator.is_null() {
                separator = b"\0" as *const u8 as *const ::core::ffi::c_char;
            }
        }
        xasprintf(
            &raw mut value,
            b"%s%s%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*o).value.string,
            separator,
            s,
        );
        free(s as *mut ::core::ffi::c_void);
    } else {
        value = s;
    }
    if o.is_null() && *name as ::core::ffi::c_int == '@' as i32 {
        o = options_add(oo, name);
    } else if o.is_null() {
        o = options_default(oo, options_parent_table_entry(oo, name));
        if o.is_null() {
            return ::core::ptr::null_mut::<options_entry>();
        }
    }
    if !((*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        fatalx(
            b"option %s is not a string\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    free((*o).value.string as *mut ::core::ffi::c_void);
    (*o).value.string = value;
    (*o).cached = 0 as ::core::ffi::c_int;
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_set_number(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut value: ::core::ffi::c_longlong,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if *name as ::core::ffi::c_int == '@' as i32 {
        fatalx(
            b"user option %s must be a string\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    o = options_get_only(oo, name);
    if o.is_null() {
        o = options_default(oo, options_parent_table_entry(oo, name));
        if o.is_null() {
            return ::core::ptr::null_mut::<options_entry>();
        }
    }
    if !(!(*o).tableentry.is_null()
        && ((*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_COLOUR as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint))
    {
        fatalx(
            b"option %s is not a number\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    (*o).value.number = value;
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_set_command(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut value: *mut cmd_list,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if *name as ::core::ffi::c_int == '@' as i32 {
        fatalx(
            b"user option %s must be a string\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    o = options_get_only(oo, name);
    if o.is_null() {
        o = options_default(oo, options_parent_table_entry(oo, name));
        if o.is_null() {
            return ::core::ptr::null_mut::<options_entry>();
        }
    }
    if !(!(*o).tableentry.is_null()
        && (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        fatalx(
            b"option %s is not a command\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    if !(*o).value.cmdlist.is_null() {
        cmd_list_free((*o).value.cmdlist);
    }
    (*o).value.cmdlist = value;
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_scope_from_name(
    mut args: *mut args,
    mut window: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
    mut oo: *mut *mut options,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut s: *mut session = (*fs).s;
    let mut wl: *mut winlink = (*fs).wl;
    let mut wp: *mut window_pane = (*fs).wp;
    let mut target: *const ::core::ffi::c_char = args_get(args, 't' as i32 as u_char);
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut scope: ::core::ffi::c_int = OPTIONS_TABLE_NONE;
    if *name as ::core::ffi::c_int == '@' as i32 {
        return options_scope_from_flags(args, window, fs, oo, cause);
    }
    oe = &raw const options_table as *const options_table_entry;
    while !(*oe).name.is_null() {
        if strcmp((*oe).name, name) == 0 as ::core::ffi::c_int {
            break;
        }
        oe = oe.offset(1);
    }
    if (*oe).name.is_null() {
        xasprintf(
            cause,
            b"unknown option: %s\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
        return 0 as ::core::ffi::c_int;
    }
    let mut current_block_38: u64;
    match (*oe).scope {
        OPTIONS_TABLE_SERVER => {
            *oo = global_options;
            scope = OPTIONS_TABLE_SERVER;
            current_block_38 = 980989089337379490;
        }
        OPTIONS_TABLE_SESSION => {
            if args_has(args, 'g' as i32 as u_char) != 0 {
                *oo = global_s_options;
                scope = OPTIONS_TABLE_SESSION;
            } else if s.is_null() && !target.is_null() {
                xasprintf(
                    cause,
                    b"no such session: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else if s.is_null() {
                xasprintf(
                    cause,
                    b"no current session\0" as *const u8 as *const ::core::ffi::c_char,
                );
            } else {
                *oo = (*s).options;
                scope = OPTIONS_TABLE_SESSION;
            }
            current_block_38 = 980989089337379490;
        }
        12 => {
            if args_has(args, 'p' as i32 as u_char) != 0 {
                if wp.is_null() && !target.is_null() {
                    xasprintf(
                        cause,
                        b"no such pane: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        target,
                    );
                } else if wp.is_null() {
                    xasprintf(
                        cause,
                        b"no current pane\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                } else {
                    *oo = (*wp).options;
                    scope = OPTIONS_TABLE_PANE;
                }
                current_block_38 = 980989089337379490;
            } else {
                current_block_38 = 7230205663690532434;
            }
        }
        OPTIONS_TABLE_WINDOW => {
            current_block_38 = 7230205663690532434;
        }
        _ => {
            current_block_38 = 980989089337379490;
        }
    }
    match current_block_38 {
        7230205663690532434 => {
            if args_has(args, 'g' as i32 as u_char) != 0 {
                *oo = global_w_options;
                scope = OPTIONS_TABLE_WINDOW;
            } else if wl.is_null() && !target.is_null() {
                xasprintf(
                    cause,
                    b"no such window: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else if wl.is_null() {
                xasprintf(
                    cause,
                    b"no current window\0" as *const u8 as *const ::core::ffi::c_char,
                );
            } else {
                *oo = (*(*wl).window).options;
                scope = OPTIONS_TABLE_WINDOW;
            }
        }
        _ => {}
    }
    return scope;
}
#[no_mangle]
pub unsafe extern "C" fn options_scope_from_flags(
    mut args: *mut args,
    mut window: ::core::ffi::c_int,
    mut fs: *mut cmd_find_state,
    mut oo: *mut *mut options,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut s: *mut session = (*fs).s;
    let mut wl: *mut winlink = (*fs).wl;
    let mut wp: *mut window_pane = (*fs).wp;
    let mut target: *const ::core::ffi::c_char = args_get(args, 't' as i32 as u_char);
    if args_has(args, 's' as i32 as u_char) != 0 {
        *oo = global_options;
        return 0x1 as ::core::ffi::c_int;
    }
    if args_has(args, 'p' as i32 as u_char) != 0 {
        if wp.is_null() {
            if !target.is_null() {
                xasprintf(
                    cause,
                    b"no such pane: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else {
                xasprintf(
                    cause,
                    b"no current pane\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return 0 as ::core::ffi::c_int;
        }
        *oo = (*wp).options;
        return 0x8 as ::core::ffi::c_int;
    } else if window != 0 || args_has(args, 'w' as i32 as u_char) != 0 {
        if args_has(args, 'g' as i32 as u_char) != 0 {
            *oo = global_w_options;
            return 0x4 as ::core::ffi::c_int;
        }
        if wl.is_null() {
            if !target.is_null() {
                xasprintf(
                    cause,
                    b"no such window: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else {
                xasprintf(
                    cause,
                    b"no current window\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return 0 as ::core::ffi::c_int;
        }
        *oo = (*(*wl).window).options;
        return 0x4 as ::core::ffi::c_int;
    } else {
        if args_has(args, 'g' as i32 as u_char) != 0 {
            *oo = global_s_options;
            return 0x2 as ::core::ffi::c_int;
        }
        if s.is_null() {
            if !target.is_null() {
                xasprintf(
                    cause,
                    b"no such session: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else {
                xasprintf(
                    cause,
                    b"no current session\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return 0 as ::core::ffi::c_int;
        }
        *oo = (*s).options;
        return 0x2 as ::core::ffi::c_int;
    };
}
#[no_mangle]
pub unsafe extern "C" fn options_string_to_style(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut ft: *mut format_tree,
) -> *mut style {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut dgc: *const grid_cell = &raw const grid_default_cell;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut failed: ::core::ffi::c_int = 0;
    o = options_get(oo, name);
    if o.is_null()
        || !((*o).tableentry.is_null()
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        return ::core::ptr::null_mut::<style>();
    }
    if (*o).cached != 0 {
        return &raw mut (*o).style;
    }
    s = (*o).value.string;
    oe = (*o).tableentry;
    log_debug(
        b"%s: %s is '%s'\0" as *const u8 as *const ::core::ffi::c_char,
        b"options_string_to_style\0" as *const u8 as *const ::core::ffi::c_char,
        name,
        s,
    );
    style_set(&raw mut (*o).style, dgc);
    (*o).cached = (strstr(s, b"#{\0" as *const u8 as *const ::core::ffi::c_char)
        == NULL as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
    if !ft.is_null() && (*o).cached == 0 {
        expanded = format_expand(ft, s);
        if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_COLOUR != 0 {
            failed = style_parse_colour(&raw mut (*o).style, dgc, expanded);
        } else {
            failed = style_parse(&raw mut (*o).style, dgc, expanded);
        }
        free(expanded as *mut ::core::ffi::c_void);
        if failed != 0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<style>();
        }
    } else {
        if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_COLOUR != 0 {
            failed = style_parse_colour(&raw mut (*o).style, dgc, s);
        } else {
            failed = style_parse(&raw mut (*o).style, dgc, s);
        }
        if failed != 0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<style>();
        }
    }
    return &raw mut (*o).style;
}
unsafe extern "C" fn options_from_string_check(
    mut oe: *const options_table_entry,
    mut value: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut sy: style = style {
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
    if oe.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if strcmp(
        (*oe).name,
        b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        && checkshell(value) == 0
    {
        xasprintf(
            cause,
            b"not a suitable shell: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    if !(*oe).pattern.is_null()
        && fnmatch((*oe).pattern, value, 0 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int
    {
        xasprintf(
            cause,
            b"value is invalid: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    if (*oe).flags & OPTIONS_TABLE_IS_STYLE != 0
        && strstr(value, b"#{\0" as *const u8 as *const ::core::ffi::c_char).is_null()
        && style_parse(&raw mut sy, &raw const grid_default_cell, value) != 0 as ::core::ffi::c_int
    {
        xasprintf(
            cause,
            b"invalid style: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    if (*oe).flags & OPTIONS_TABLE_IS_COLOUR != 0
        && strstr(value, b"#{\0" as *const u8 as *const ::core::ffi::c_char).is_null()
        && style_parse_colour(&raw mut sy, &raw const grid_default_cell, value)
            != 0 as ::core::ffi::c_int
    {
        xasprintf(
            cause,
            b"invalid colour: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn options_from_string_flag(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut flag: ::core::ffi::c_int = 0;
    if value.is_null() || *value as ::core::ffi::c_int == '\0' as i32 {
        flag = (options_get_number(oo, name) == 0) as ::core::ffi::c_int;
    } else if strcmp(value, b"1\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcasecmp(value, b"on\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        || strcasecmp(value, b"yes\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        flag = 1 as ::core::ffi::c_int;
    } else if strcmp(value, b"0\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcasecmp(value, b"off\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        || strcasecmp(value, b"no\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        flag = 0 as ::core::ffi::c_int;
    } else {
        xasprintf(
            cause,
            b"bad value: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    options_set_number(oo, name, flag as ::core::ffi::c_longlong);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn options_find_choice(
    mut oe: *const options_table_entry,
    mut value: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut cp: *mut *const ::core::ffi::c_char =
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut choice: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    cp = (*oe).choices;
    while !(*cp).is_null() {
        if strcmp(*cp, value) == 0 as ::core::ffi::c_int {
            choice = n;
        }
        n += 1;
        cp = cp.offset(1);
    }
    if choice == -(1 as ::core::ffi::c_int) {
        xasprintf(
            cause,
            b"unknown value: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    return choice;
}
unsafe extern "C" fn options_from_string_choice(
    mut oe: *const options_table_entry,
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut choice: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    if value.is_null() {
        choice = options_get_number(oo, name) as ::core::ffi::c_int;
        if choice < 2 as ::core::ffi::c_int {
            choice = (choice == 0) as ::core::ffi::c_int;
        }
    } else {
        choice = options_find_choice(oe, value, cause);
        if choice < 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
    }
    options_set_number(oo, name, choice as ::core::ffi::c_longlong);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn options_from_string(
    mut oo: *mut options,
    mut oe: *const options_table_entry,
    mut name: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut append: ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut type_0: options_table_type = OPTIONS_TABLE_STRING;
    let mut number: ::core::ffi::c_longlong = 0;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut new: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut old: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: key_code = 0;
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    if !oe.is_null() {
        if value.is_null()
            && (*oe).type_0 as ::core::ffi::c_uint
                != OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*oe).type_0 as ::core::ffi::c_uint
                != OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            xasprintf(
                cause,
                b"empty value\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        type_0 = (*oe).type_0;
    } else {
        if *name as ::core::ffi::c_int != '@' as i32 {
            xasprintf(
                cause,
                b"bad option name\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        type_0 = OPTIONS_TABLE_STRING;
    }
    match type_0 as ::core::ffi::c_uint {
        0 => {
            old = xstrdup(options_get_string(oo, name));
            options_set_string(
                oo,
                name,
                append,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                value,
            );
            new = options_get_string(oo, name);
            if options_from_string_check(oe, new, cause) != 0 as ::core::ffi::c_int {
                options_set_string(
                    oo,
                    name,
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    old,
                );
                free(old as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            free(old as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        }
        1 => {
            number = strtonum(
                value,
                (*oe).minimum as ::core::ffi::c_longlong,
                (*oe).maximum as ::core::ffi::c_longlong,
                &raw mut errstr,
            );
            if !errstr.is_null() {
                xasprintf(
                    cause,
                    b"value is %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    errstr,
                    value,
                );
                return -(1 as ::core::ffi::c_int);
            }
            options_set_number(oo, name, number);
            return 0 as ::core::ffi::c_int;
        }
        2 => {
            key = key_string_lookup_string(value);
            if key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
                xasprintf(
                    cause,
                    b"bad key: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    value,
                );
                return -(1 as ::core::ffi::c_int);
            }
            options_set_number(oo, name, key as ::core::ffi::c_longlong);
            return 0 as ::core::ffi::c_int;
        }
        3 => {
            number = colour_fromstring(value) as ::core::ffi::c_longlong;
            if number == -(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong {
                xasprintf(
                    cause,
                    b"bad colour: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    value,
                );
                return -(1 as ::core::ffi::c_int);
            }
            options_set_number(oo, name, number);
            return 0 as ::core::ffi::c_int;
        }
        4 => return options_from_string_flag(oo, name, value, cause),
        5 => return options_from_string_choice(oe, oo, name, value, cause),
        6 => {
            pr = cmd_parse_from_string(value, ::core::ptr::null_mut::<cmd_parse_input>());
            match (*pr).status as ::core::ffi::c_uint {
                0 => {
                    *cause = (*pr).error;
                    return -(1 as ::core::ffi::c_int);
                }
                1 => {
                    options_set_command(oo, name, (*pr).cmdlist);
                    return 0 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        _ => {}
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn options_push_changes(mut name: *const ::core::ffi::c_char) {
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"options_push_changes\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    if strcmp(name, b"theme\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strncmp(
            name,
            b"dark-theme-\0" as *const u8 as *const ::core::ffi::c_char,
            11 as size_t,
        ) == 0 as ::core::ffi::c_int
        || strncmp(
            name,
            b"light-theme-\0" as *const u8 as *const ::core::ffi::c_char,
            12 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            server_client_update_theme_colours(loop_0);
            if (*loop_0).tty.flags & TTY_OPENED != 0 {
                tty_invalidate(&raw mut (*loop_0).tty);
            }
            server_redraw_client(loop_0);
            loop_0 = (*loop_0).entry.tqe_next;
        }
    }
    if strcmp(
        name,
        b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
        while !w.is_null() {
            if !(*w).active.is_null() {
                if options_get_number((*w).options, name) != 0 {
                    (*(*w).active).flags |= PANE_CHANGED;
                }
            }
            w = windows_RB_NEXT(w);
        }
    }
    if strcmp(
        name,
        b"cursor-colour\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            window_pane_default_cursor(wp);
            wp = window_pane_tree_RB_NEXT(wp);
        }
    }
    if strcmp(
        name,
        b"cursor-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            window_pane_default_cursor(wp);
            wp = window_pane_tree_RB_NEXT(wp);
        }
    }
    if strcmp(
        name,
        b"fill-character\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
        while !w.is_null() {
            window_set_fill_cells(w);
            w = windows_RB_NEXT(w);
        }
    }
    if strcmp(
        name,
        b"key-table\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            server_client_set_key_table(loop_0, ::core::ptr::null::<::core::ffi::c_char>());
            loop_0 = (*loop_0).entry.tqe_next;
        }
    }
    if strcmp(
        name,
        b"user-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            if (*loop_0).tty.flags & TTY_OPENED != 0 {
                tty_keys_build(&raw mut (*loop_0).tty);
            }
            loop_0 = (*loop_0).entry.tqe_next;
        }
    }
    if strcmp(name, b"status\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"status-interval\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        status_timer_start_all();
    }
    if strcmp(name, b"status\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-border-indicators\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-border-status\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars-timeout\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars-position\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars-style\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        redraw_invalidate_all_scenes();
    }
    if strcmp(
        name,
        b"monitor-silence\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        alerts_reset_all();
    }
    if strcmp(
        name,
        b"window-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"window-active-style\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            (*wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
            wp = window_pane_tree_RB_NEXT(wp);
        }
    }
    if *name as ::core::ffi::c_int == '@' as i32 {
        wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            (*wp).flags |= PANE_STYLECHANGED;
            wp = window_pane_tree_RB_NEXT(wp);
        }
    }
    if strcmp(
        name,
        b"pane-colours\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            colour_palette_from_option(&raw mut (*wp).palette, (*wp).options);
            wp = window_pane_tree_RB_NEXT(wp);
        }
    }
    if strcmp(
        name,
        b"pane-border-status\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars-position\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
        while !w.is_null() {
            (*w).sb = options_get_number(
                (*w).options,
                b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
            (*w).sb_pos = options_get_number(
                (*w).options,
                b"pane-scrollbars-position\0" as *const u8 as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
            layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
            w = windows_RB_NEXT(w);
        }
    }
    if strcmp(
        name,
        b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            window_pane_scrollbar_hide(wp);
            wp = window_pane_tree_RB_NEXT(wp);
        }
    }
    if strcmp(
        name,
        b"pane-scrollbars-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            style_set_scrollbar_style_from_option(&raw mut (*wp).scrollbar_style, (*wp).options);
            wp = window_pane_tree_RB_NEXT(wp);
        }
        w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
        while !w.is_null() {
            layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
            w = windows_RB_NEXT(w);
        }
    }
    if strcmp(
        name,
        b"codepoint-widths\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        utf8_update_width_cache();
    }
    if strcmp(
        name,
        b"input-buffer-size\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        input_set_buffer_size(options_get_number(global_options, name) as size_t);
    }
    if strcmp(
        name,
        b"history-limit\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
        while !s.is_null() {
            session_update_history(s);
            s = sessions_RB_NEXT(s);
        }
    }
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        status_update_cache(s);
        s = sessions_RB_NEXT(s);
    }
    recalculate_sizes();
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        if !(*loop_0).session.is_null() {
            server_redraw_client(loop_0);
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn options_remove_or_default(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut oo: *mut options = (*o).owner;
    if key.is_null() {
        if !(*o).tableentry.is_null()
            && (oo == global_options || oo == global_s_options || oo == global_w_options)
        {
            options_default(oo, (*o).tableentry);
        } else {
            options_remove(o);
        }
    } else if options_array_set(
        o,
        key,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
        cause,
    ) != 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    return 0 as ::core::ffi::c_int;
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

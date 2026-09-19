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
    pub type screen_write_citem;
    fn __ctype_toupper_loc() -> *mut *const __int32_t;
    fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn sort_get_sessions(_: *mut u_int, _: *mut sort_criteria) -> *mut *mut session;
    fn sort_get_winlinks(_: *mut u_int, _: *mut sort_criteria) -> *mut *mut winlink;
    fn format_create(
        _: *mut client,
        _: *mut cmdq_item,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut format_tree;
    fn format_free(_: *mut format_tree);
    fn format_expand(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn format_defaults(
        _: *mut format_tree,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    );
    fn format_draw(
        _: *mut screen_write_ctx,
        _: *const grid_cell,
        _: u_int,
        _: *const ::core::ffi::c_char,
        _: *mut style_ranges,
        _: ::core::ffi::c_int,
    );
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn cmd_find_clear_state(_: *mut cmd_find_state, _: ::core::ffi::c_int);
    fn cmd_find_from_session(_: *mut cmd_find_state, _: *mut session, _: ::core::ffi::c_int);
    fn cmd_find_from_winlink(_: *mut cmd_find_state, _: *mut winlink, _: ::core::ffi::c_int);
    fn cmd_mouse_at(
        _: *mut window_pane,
        _: *mut mouse_event,
        _: *mut u_int,
        _: *mut u_int,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_template_replace(
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn cmd_parse_and_append(
        _: *const ::core::ffi::c_char,
        _: *mut cmd_parse_input,
        _: *mut client,
        _: *mut cmdq_state,
        _: *mut *mut ::core::ffi::c_char,
    ) -> cmd_parse_status;
    fn cmdq_new_state(
        _: *mut cmd_find_state,
        _: *mut key_event,
        _: ::core::ffi::c_int,
    ) -> *mut cmdq_state;
    fn cmdq_free_state(_: *mut cmdq_state);
    fn server_redraw_window(_: *mut window);
    fn server_unzoom_window(_: *mut window);
    fn status_message_set(
        _: *mut client,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn prompt_set_options(_: *mut prompt_create_data, _: *mut session);
    fn prompt_create(_: *const prompt_create_data) -> *mut prompt;
    fn prompt_free(_: *mut prompt);
    fn prompt_incremental_start(_: *mut prompt);
    fn prompt_draw(_: *mut prompt, _: *mut prompt_draw_data);
    fn prompt_key(_: *mut prompt, _: key_code, _: *mut ::core::ffi::c_int) -> prompt_key_result;
    fn prompt_mouse(
        _: *mut prompt,
        _: u_int,
        _: u_int,
        _: u_int,
        _: *mut ::core::ffi::c_int,
    ) -> prompt_key_result;
    fn prompt_update(_: *mut prompt, _: *const ::core::ffi::c_char, _: *const ::core::ffi::c_char);
    fn fuzzy_match(
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: u_int,
        _: *mut u_int,
    ) -> *mut bitstr_t;
    static grid_default_cell: grid_cell;
    fn grid_get_cell(_: *mut grid, _: u_int, _: u_int, _: *mut grid_cell);
    fn screen_write_start(_: *mut screen_write_ctx, _: *mut screen);
    fn screen_write_stop(_: *mut screen_write_ctx);
    fn screen_write_clearendofline(_: *mut screen_write_ctx, _: u_int);
    fn screen_write_cursormove(
        _: *mut screen_write_ctx,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn screen_write_clearscreen(_: *mut screen_write_ctx, _: u_int);
    fn screen_write_cell(_: *mut screen_write_ctx, _: *const grid_cell);
    fn screen_init(_: *mut screen, _: u_int, _: u_int, _: u_int);
    fn screen_free(_: *mut screen);
    fn screen_resize(_: *mut screen, _: u_int, _: u_int, _: ::core::ffi::c_int);
    fn winlink_find_by_index(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn window_zoom(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_reset_mode(_: *mut window_pane);
    fn session_find_by_id(_: u_int) -> *mut session;
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
pub type __int32_t = i32;
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
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
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
pub type key_code_type = ::core::ffi::c_uint;
pub const KEYC_TYPE_NOTYPE: key_code_type = 13;
pub const KEYC_TYPE_TRIPLECLICK: key_code_type = 12;
pub const KEYC_TYPE_DOUBLECLICK: key_code_type = 11;
pub const KEYC_TYPE_SECONDCLICK: key_code_type = 10;
pub const KEYC_TYPE_WHEELUP: key_code_type = 9;
pub const KEYC_TYPE_WHEELDOWN: key_code_type = 8;
pub const KEYC_TYPE_MOUSEDRAGEND: key_code_type = 7;
pub const KEYC_TYPE_MOUSEDRAG: key_code_type = 6;
pub const KEYC_TYPE_MOUSEUP: key_code_type = 5;
pub const KEYC_TYPE_MOUSEDOWN: key_code_type = 4;
pub const KEYC_TYPE_MOUSEMOVE: key_code_type = 3;
pub const KEYC_TYPE_FUNCTION: key_code_type = 2;
pub const KEYC_TYPE_USER: key_code_type = 1;
pub const KEYC_TYPE_UNICODE: key_code_type = 0;
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
pub type cmd_parse_status = ::core::ffi::c_uint;
pub const CMD_PARSE_SUCCESS: cmd_parse_status = 1;
pub const CMD_PARSE_ERROR: cmd_parse_status = 0;
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
pub type prompt_type = ::core::ffi::c_uint;
pub const PROMPT_TYPE_INVALID: prompt_type = 255;
pub const PROMPT_TYPE_SEARCH: prompt_type = 1;
pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
pub type prompt_result = ::core::ffi::c_uint;
pub const PROMPT_CLOSE: prompt_result = 1;
pub const PROMPT_CONTINUE: prompt_result = 0;
pub type prompt_key_result = ::core::ffi::c_uint;
pub const PROMPT_KEY_MOVE: prompt_key_result = 3;
pub const PROMPT_KEY_CLOSE: prompt_key_result = 2;
pub const PROMPT_KEY_HANDLED: prompt_key_result = 1;
pub const PROMPT_KEY_NOT_HANDLED: prompt_key_result = 0;
pub type prompt_input_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *const ::core::ffi::c_char,
        prompt_key_result,
    ) -> prompt_result,
>;
pub type prompt_free_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct prompt_create_data {
    pub fs: *mut cmd_find_state,
    pub prompt: *const ::core::ffi::c_char,
    pub input: *const ::core::ffi::c_char,
    pub type_0: prompt_type,
    pub flags: ::core::ffi::c_int,
    pub style: grid_cell,
    pub command_style: grid_cell,
    pub style_str: *const ::core::ffi::c_char,
    pub command_style_str: *const ::core::ffi::c_char,
    pub cstyle: screen_cursor_style,
    pub command_cstyle: screen_cursor_style,
    pub ccolour: ::core::ffi::c_int,
    pub command_ccolour: ::core::ffi::c_int,
    pub cmode: ::core::ffi::c_int,
    pub command_cmode: ::core::ffi::c_int,
    pub message_format: *const ::core::ffi::c_char,
    pub keys: ::core::ffi::c_int,
    pub word_separators: *const ::core::ffi::c_char,
    pub inputcb: prompt_input_cb,
    pub freecb: prompt_free_cb,
    pub data: *mut ::core::ffi::c_void,
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
pub type sort_order = ::core::ffi::c_uint;
pub const SORT_END: sort_order = 8;
pub const SORT_Z: sort_order = 7;
pub const SORT_SIZE: sort_order = 6;
pub const SORT_ORDER: sort_order = 5;
pub const SORT_NAME: sort_order = 4;
pub const SORT_MODIFIER: sort_order = 3;
pub const SORT_INDEX: sort_order = 2;
pub const SORT_CREATION: sort_order = 1;
pub const SORT_ACTIVITY: sort_order = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sort_criteria {
    pub order: sort_order,
    pub reversed: ::core::ffi::c_int,
    pub order_seq: *mut sort_order,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_switch_modedata {
    pub wp: *mut window_pane,
    pub screen: screen,
    pub zoomed: ::core::ffi::c_int,
    pub format: *mut ::core::ffi::c_char,
    pub command: *mut ::core::ffi::c_char,
    pub type_0: window_switch_type,
    pub filter: *mut ::core::ffi::c_char,
    pub prompt: *mut prompt,
    pub prompt_cx: u_int,
    pub item_list: *mut *mut window_switch_itemdata,
    pub item_size: u_int,
    pub matches: *mut *mut window_switch_itemdata,
    pub matches_size: u_int,
    pub current: u_int,
    pub offset: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_switch_itemdata {
    pub type_0: window_switch_type,
    pub session: ::core::ffi::c_int,
    pub winlink: ::core::ffi::c_int,
    pub tag: uint64_t,
    pub text: *mut ::core::ffi::c_char,
    pub match_0: *mut bitstr_t,
    pub score: u_int,
    pub order: u_int,
}
pub type window_switch_type = ::core::ffi::c_uint;
pub const WINDOW_SWITCH_TYPE_WINDOW: window_switch_type = 1;
pub const WINDOW_SWITCH_TYPE_SESSION: window_switch_type = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[inline]
unsafe extern "C" fn toupper(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_toupper_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}
pub const KEYC_CTRL: ::core::ffi::c_ulonglong = 0x200000000000 as ::core::ffi::c_ulonglong;
pub const KEYC_MASK_TYPE: ::core::ffi::c_ulonglong = 0xff00000000 as ::core::ffi::c_ulonglong;
pub const KEYC_MASK_KEY: ::core::ffi::c_ulonglong = 0xffffffffff as ::core::ffi::c_ulonglong;
pub const MODE_CURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_REDRAW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINDOW_ZOOMED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const MOUSE_MASK_BUTTONS: ::core::ffi::c_int = 195 as ::core::ffi::c_int;
pub const MOUSE_MASK_DRAG: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PROMPT_INCREMENTAL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const PROMPT_NOFORMAT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const PROMPT_ISMODE: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const PROMPT_EDITARROWS: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const FORMAT_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const WINDOW_SWITCH_DEFAULT_COMMAND: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"switch-client -Zt '%%'\0")
};
pub const WINDOW_SWITCH_DEFAULT_FORMAT: [::core::ffi::c_char; 350] = unsafe {
    ::core::mem::transmute::<
        [u8; 350],
        [::core::ffi::c_char; 350],
    >(
        *b"#{?window_format,#{window_name} #[dim]#{session_name}:#{window_index}#{window_flags}#[default] #[dim]#{pane_current_command}#[default] #[dim]#{?#{!=:#{pane_title},#{host_short}},#{pane_title},}#[default],#{session_name} #[dim]#{session_windows} windows#[default] #{?session_attached,attached,#[dim]detached#[default]} #[dim]#{window_name}#[default]}\0",
    )
};
#[no_mangle]
pub static mut window_switch_mode: window_mode = unsafe {
    window_mode {
        name: b"switch-mode\0" as *const u8 as *const ::core::ffi::c_char,
        default_format: WINDOW_SWITCH_DEFAULT_FORMAT.as_ptr(),
        flags: 0,
        init: Some(
            window_switch_init
                as unsafe extern "C" fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_switch_free as unsafe extern "C" fn(*mut window_mode_entry) -> ()),
        resize: Some(
            window_switch_resize
                as unsafe extern "C" fn(*mut window_mode_entry, u_int, u_int) -> (),
        ),
        update: None,
        style_changed: None,
        key: Some(
            window_switch_key
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
unsafe extern "C" fn window_switch_free_item(mut item: *mut window_switch_itemdata) {
    free((*item).match_0 as *mut ::core::ffi::c_void);
    free((*item).text as *mut ::core::ffi::c_void);
    free(item as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_switch_add_item(
    mut data: *mut window_switch_modedata,
) -> *mut window_switch_itemdata {
    let mut item: *mut window_switch_itemdata = ::core::ptr::null_mut::<window_switch_itemdata>();
    (*data).item_list = xreallocarray(
        (*data).item_list as *mut ::core::ffi::c_void,
        (*data).item_size.wrapping_add(1 as u_int) as size_t,
        ::core::mem::size_of::<*mut window_switch_itemdata>() as size_t,
    ) as *mut *mut window_switch_itemdata;
    let fresh5 = (*data).item_size;
    (*data).item_size = (*data).item_size.wrapping_add(1);
    let ref mut fresh6 = *(*data).item_list.offset(fresh5 as isize);
    *fresh6 = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_switch_itemdata>() as size_t,
    ) as *mut window_switch_itemdata;
    item = *fresh6;
    return item;
}
unsafe extern "C" fn window_switch_add_session(
    mut data: *mut window_switch_modedata,
    mut s: *mut session,
    mut order: *mut u_int,
) {
    let mut item: *mut window_switch_itemdata = ::core::ptr::null_mut::<window_switch_itemdata>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(
        ft,
        ::core::ptr::null_mut::<client>(),
        s,
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    item = window_switch_add_item(data);
    (*item).type_0 = WINDOW_SWITCH_TYPE_SESSION;
    (*item).session = (*s).id as ::core::ffi::c_int;
    (*item).winlink = -(1 as ::core::ffi::c_int);
    (*item).tag = s as uint64_t;
    let fresh7 = *order;
    *order = (*order).wrapping_add(1);
    (*item).order = fresh7;
    (*item).text = format_expand(ft, (*data).format);
    format_free(ft);
}
unsafe extern "C" fn window_switch_add_window(
    mut data: *mut window_switch_modedata,
    mut wl: *mut winlink,
    mut order: *mut u_int,
) {
    let mut item: *mut window_switch_itemdata = ::core::ptr::null_mut::<window_switch_itemdata>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(
        ft,
        ::core::ptr::null_mut::<client>(),
        (*wl).session,
        wl,
        ::core::ptr::null_mut::<window_pane>(),
    );
    item = window_switch_add_item(data);
    (*item).type_0 = WINDOW_SWITCH_TYPE_WINDOW;
    (*item).session = (*(*wl).session).id as ::core::ffi::c_int;
    (*item).winlink = (*wl).idx;
    (*item).tag = wl as uint64_t;
    let fresh4 = *order;
    *order = (*order).wrapping_add(1);
    (*item).order = fresh4;
    (*item).text = format_expand(ft, (*data).format);
    format_free(ft);
}
unsafe extern "C" fn window_switch_compare(
    mut a0: *const ::core::ffi::c_void,
    mut b0: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut a: *const *mut window_switch_itemdata = a0 as *const *mut window_switch_itemdata;
    let mut b: *const *mut window_switch_itemdata = b0 as *const *mut window_switch_itemdata;
    if (**a).score > (**b).score {
        return -(1 as ::core::ffi::c_int);
    }
    if (**a).score < (**b).score {
        return 1 as ::core::ffi::c_int;
    }
    if (**a).order < (**b).order {
        return -(1 as ::core::ffi::c_int);
    }
    if (**a).order > (**b).order {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_switch_build(mut data: *mut window_switch_modedata) {
    let mut item: *mut window_switch_itemdata = ::core::ptr::null_mut::<window_switch_itemdata>();
    let mut m: *mut *mut window_switch_itemdata =
        ::core::ptr::null_mut::<*mut window_switch_itemdata>();
    let mut f: *const ::core::ffi::c_char = (*data).filter;
    let mut ns: u_int = 0;
    let mut nw: u_int = 0;
    let mut i: u_int = 0;
    let mut n: u_int = 0 as u_int;
    let mut order: u_int = 0 as u_int;
    let mut sx: u_int = (*(*data).screen.grid).sx;
    let mut sl: *mut *mut session = ::core::ptr::null_mut::<*mut session>();
    let mut wl: *mut *mut winlink = ::core::ptr::null_mut::<*mut winlink>();
    let mut sort_crit: sort_criteria = sort_criteria {
        order: SORT_ACTIVITY,
        reversed: 0,
        order_seq: ::core::ptr::null_mut::<sort_order>(),
    };
    sort_crit.order = SORT_NAME;
    sort_crit.reversed = 0 as ::core::ffi::c_int;
    i = 0 as u_int;
    while i < (*data).item_size {
        window_switch_free_item(*(*data).item_list.offset(i as isize));
        i = i.wrapping_add(1);
    }
    free((*data).item_list as *mut ::core::ffi::c_void);
    (*data).item_list = ::core::ptr::null_mut::<*mut window_switch_itemdata>();
    (*data).item_size = 0 as u_int;
    match (*data).type_0 as ::core::ffi::c_uint {
        0 => {
            sl = sort_get_sessions(&raw mut ns, &raw mut sort_crit);
            i = 0 as u_int;
            while i < ns {
                window_switch_add_session(data, *sl.offset(i as isize), &raw mut order);
                i = i.wrapping_add(1);
            }
        }
        1 => {
            wl = sort_get_winlinks(&raw mut nw, &raw mut sort_crit);
            i = 0 as u_int;
            while i < nw {
                window_switch_add_window(data, *wl.offset(i as isize), &raw mut order);
                i = i.wrapping_add(1);
            }
        }
        _ => {}
    }
    i = 0 as u_int;
    while i < (*data).item_size {
        item = *(*data).item_list.offset(i as isize);
        if *f as ::core::ffi::c_int == '\0' as i32 {
            m = xreallocarray(
                m as *mut ::core::ffi::c_void,
                n.wrapping_add(1 as u_int) as size_t,
                ::core::mem::size_of::<*mut window_switch_itemdata>() as size_t,
            ) as *mut *mut window_switch_itemdata;
            let fresh0 = n;
            n = n.wrapping_add(1);
            let ref mut fresh1 = *m.offset(fresh0 as isize);
            *fresh1 = item;
        } else {
            (*item).match_0 = fuzzy_match(f, (*item).text, sx, &raw mut (*item).score);
            if !(*item).match_0.is_null() {
                m = xreallocarray(
                    m as *mut ::core::ffi::c_void,
                    n.wrapping_add(1 as u_int) as size_t,
                    ::core::mem::size_of::<*mut window_switch_itemdata>() as size_t,
                ) as *mut *mut window_switch_itemdata;
                let fresh2 = n;
                n = n.wrapping_add(1);
                let ref mut fresh3 = *m.offset(fresh2 as isize);
                *fresh3 = item;
            }
        }
        i = i.wrapping_add(1);
    }
    if n > 1 as u_int {
        qsort(
            m as *mut ::core::ffi::c_void,
            n as size_t,
            ::core::mem::size_of::<*mut window_switch_itemdata>() as size_t,
            Some(
                window_switch_compare
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
    }
    free((*data).matches as *mut ::core::ffi::c_void);
    (*data).matches = m;
    (*data).matches_size = n;
}
unsafe extern "C" fn window_switch_visible(mut data: *mut window_switch_modedata) -> u_int {
    let mut sy: u_int = (*(*data).screen.grid).sy;
    if sy <= 1 as u_int {
        return 0 as u_int;
    }
    return sy.wrapping_sub(1 as u_int);
}
unsafe extern "C" fn window_switch_set_current(
    mut data: *mut window_switch_modedata,
    mut current: u_int,
) {
    let mut visible: u_int = window_switch_visible(data);
    if (*data).matches_size == 0 as u_int {
        (*data).current = 0 as u_int;
        (*data).offset = 0 as u_int;
        return;
    }
    if current > (*data).matches_size.wrapping_sub(1 as u_int) {
        current = (*data).matches_size.wrapping_sub(1 as u_int);
    }
    (*data).current = current;
    if (*data).current < (*data).offset {
        (*data).offset = (*data).current;
    } else if visible != 0 as u_int && (*data).current >= (*data).offset.wrapping_add(visible) {
        (*data).offset = (*data)
            .current
            .wrapping_sub(visible)
            .wrapping_add(1 as u_int);
    }
}
unsafe extern "C" fn window_switch_draw_screen(mut wme: *mut window_mode_entry) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_switch_modedata = (*wme).data as *mut window_switch_modedata;
    let mut oo: *mut options = (*wp).options;
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
    let mut s: *mut screen = &raw mut (*data).screen;
    let mut sx: u_int = (*(*s).grid).sx;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut sy: u_int = (*(*s).grid).sy;
    let mut visible: u_int = 0;
    let mut idx: u_int = 0;
    let mut item: *mut window_switch_itemdata = ::core::ptr::null_mut::<window_switch_itemdata>();
    let mut mgc: grid_cell = grid_cell {
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
    let mut sgc: grid_cell = grid_cell {
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
    let mut dgc: *const grid_cell = &raw const grid_default_cell;
    let mut pdd: prompt_draw_data = prompt_draw_data {
        ctx: ::core::ptr::null_mut::<screen_write_ctx>(),
        cursor_x: ::core::ptr::null_mut::<u_int>(),
        area_x: 0,
        area_width: 0,
        prompt_line: 0,
    };
    screen_write_start(&raw mut ctx, s);
    screen_write_clearscreen(&raw mut ctx, 8 as u_int);
    if sy <= 1 as u_int {
        screen_write_stop(&raw mut ctx);
        return;
    }
    style_apply(
        &raw mut mgc,
        oo,
        b"switch-mode-match-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    style_apply(
        &raw mut sgc,
        oo,
        b"mode-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    visible = window_switch_visible(data);
    i = 0 as u_int;
    while i < visible {
        idx = (*data).offset.wrapping_add(i);
        if idx >= (*data).matches_size {
            break;
        }
        item = *(*data).matches.offset(idx as isize);
        screen_write_cursormove(
            &raw mut ctx,
            0 as ::core::ffi::c_int,
            i as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        if idx != (*data).current {
            format_draw(
                &raw mut ctx,
                dgc,
                sx,
                (*item).text,
                ::core::ptr::null_mut::<style_ranges>(),
                0 as ::core::ffi::c_int,
            );
        } else {
            screen_write_clearendofline(&raw mut ctx, sgc.bg as u_int);
            format_draw(
                &raw mut ctx,
                &raw mut sgc,
                sx,
                (*item).text,
                ::core::ptr::null_mut::<style_ranges>(),
                0 as ::core::ffi::c_int,
            );
        }
        if !(*item).match_0.is_null() {
            j = 0 as u_int;
            while j < sx {
                if !(*(*item)
                    .match_0
                    .offset((j >> 3 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << (j & 0x7 as u_int)
                    == 0)
                {
                    grid_get_cell((*s).grid, j, i, &raw mut gc);
                    gc.attr = mgc.attr;
                    gc.fg = mgc.fg;
                    gc.bg = mgc.bg;
                    screen_write_cursormove(
                        &raw mut ctx,
                        j as ::core::ffi::c_int,
                        i as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    screen_write_cell(&raw mut ctx, &raw mut gc);
                }
                j = j.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
    if !(*data).prompt.is_null() {
        pdd.ctx = &raw mut ctx;
        pdd.cursor_x = &raw mut (*data).prompt_cx;
        pdd.area_x = 0 as u_int;
        pdd.area_width = sx;
        pdd.prompt_line = sy.wrapping_sub(1 as u_int);
        (*s).mode |= MODE_CURSOR;
        prompt_draw((*data).prompt, &raw mut pdd);
        screen_write_cursormove(
            &raw mut ctx,
            (*data).prompt_cx as ::core::ffi::c_int,
            sy.wrapping_sub(1 as u_int) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    screen_write_stop(&raw mut ctx);
}
unsafe extern "C" fn window_switch_init(
    mut wme: *mut window_mode_entry,
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_switch_modedata = ::core::ptr::null_mut::<window_switch_modedata>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    let mut pd: prompt_create_data = prompt_create_data {
        fs: ::core::ptr::null_mut::<cmd_find_state>(),
        prompt: ::core::ptr::null::<::core::ffi::c_char>(),
        input: ::core::ptr::null::<::core::ffi::c_char>(),
        type_0: PROMPT_TYPE_COMMAND,
        flags: 0,
        style: grid_cell {
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
        command_style: grid_cell {
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
        style_str: ::core::ptr::null::<::core::ffi::c_char>(),
        command_style_str: ::core::ptr::null::<::core::ffi::c_char>(),
        cstyle: SCREEN_CURSOR_DEFAULT,
        command_cstyle: SCREEN_CURSOR_DEFAULT,
        ccolour: 0,
        command_ccolour: 0,
        cmode: 0,
        command_cmode: 0,
        message_format: ::core::ptr::null::<::core::ffi::c_char>(),
        keys: 0,
        word_separators: ::core::ptr::null::<::core::ffi::c_char>(),
        inputcb: None,
        freecb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    data = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_switch_modedata>() as size_t,
    ) as *mut window_switch_modedata;
    (*wme).data = data as *mut ::core::ffi::c_void;
    (*data).wp = wp;
    if args_has(args, 'w' as i32 as u_char) != 0 {
        (*data).type_0 = WINDOW_SWITCH_TYPE_WINDOW;
    } else {
        (*data).type_0 = WINDOW_SWITCH_TYPE_SESSION;
    }
    (*data).filter = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    if args.is_null() || args_has(args, 'F' as i32 as u_char) == 0 {
        (*data).format = xstrdup(WINDOW_SWITCH_DEFAULT_FORMAT.as_ptr());
    } else {
        (*data).format = xstrdup(args_get(args, 'F' as i32 as u_char));
    }
    if args.is_null() || args_count(args) == 0 as u_int {
        (*data).command = xstrdup(WINDOW_SWITCH_DEFAULT_COMMAND.as_ptr());
    } else {
        (*data).command = xstrdup(args_string(args, 0 as u_int));
    }
    memset(
        &raw mut pd as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<prompt_create_data>() as size_t,
    );
    prompt_set_options(&raw mut pd, (*fs).s);
    pd.fs = fs;
    pd.prompt = b"(search) \0" as *const u8 as *const ::core::ffi::c_char;
    pd.input = b"\0" as *const u8 as *const ::core::ffi::c_char;
    pd.type_0 = PROMPT_TYPE_SEARCH;
    pd.flags = PROMPT_INCREMENTAL | PROMPT_NOFORMAT | PROMPT_ISMODE | PROMPT_EDITARROWS;
    pd.inputcb = Some(
        window_switch_prompt_callback
            as unsafe extern "C" fn(
                *mut ::core::ffi::c_void,
                *const ::core::ffi::c_char,
                prompt_key_result,
            ) -> prompt_result,
    ) as prompt_input_cb;
    pd.data = data as *mut ::core::ffi::c_void;
    (*data).prompt = prompt_create(&raw mut pd);
    prompt_update(
        (*data).prompt,
        b"(search) \0" as *const u8 as *const ::core::ffi::c_char,
        (*data).filter,
    );
    s = &raw mut (*data).screen;
    screen_init(s, (*(*wp).base.grid).sx, (*(*wp).base.grid).sy, 0 as u_int);
    if args_has(args, 'Z' as i32 as u_char) == 0 {
        (*data).zoomed = -(1 as ::core::ffi::c_int);
    } else {
        (*data).zoomed = (*(*wp).window).flags & WINDOW_ZOOMED;
        if (*data).zoomed == 0 && window_zoom(wp) == 0 as ::core::ffi::c_int {
            server_redraw_window((*wp).window as *mut window);
        }
    }
    window_switch_build(data);
    prompt_incremental_start((*data).prompt);
    window_switch_draw_screen(wme);
    return s;
}
unsafe extern "C" fn window_switch_free(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_switch_modedata = (*wme).data as *mut window_switch_modedata;
    let mut i: u_int = 0;
    if (*data).zoomed == 0 as ::core::ffi::c_int {
        server_unzoom_window((*(*wme).wp).window as *mut window);
    }
    i = 0 as u_int;
    while i < (*data).item_size {
        window_switch_free_item(*(*data).item_list.offset(i as isize));
        i = i.wrapping_add(1);
    }
    free((*data).item_list as *mut ::core::ffi::c_void);
    free((*data).matches as *mut ::core::ffi::c_void);
    free((*data).filter as *mut ::core::ffi::c_void);
    prompt_free((*data).prompt);
    free((*data).format as *mut ::core::ffi::c_void);
    free((*data).command as *mut ::core::ffi::c_void);
    screen_free(&raw mut (*data).screen);
    free(data as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_switch_resize(
    mut wme: *mut window_mode_entry,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_switch_modedata = (*wme).data as *mut window_switch_modedata;
    let mut s: *mut screen = &raw mut (*data).screen;
    screen_resize(s, sx, sy, 0 as ::core::ffi::c_int);
    window_switch_build(data);
    window_switch_set_current(data, (*data).current);
    window_switch_draw_screen(wme);
}
unsafe extern "C" fn window_switch_run_command(
    mut data: *mut window_switch_modedata,
    mut c: *mut client,
) -> ::core::ffi::c_int {
    let mut item: *mut window_switch_itemdata = ::core::ptr::null_mut::<window_switch_itemdata>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut target: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    let mut command: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut status: cmd_parse_status = CMD_PARSE_ERROR;
    if (*data).matches_size == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    item = *(*data).matches.offset((*data).current as isize);
    cmd_find_clear_state(&raw mut fs, 0 as ::core::ffi::c_int);
    match (*item).type_0 as ::core::ffi::c_uint {
        0 => {
            s = session_find_by_id((*item).session as u_int);
            if !s.is_null() {
                xasprintf(
                    &raw mut target,
                    b"=%s:\0" as *const u8 as *const ::core::ffi::c_char,
                    (*s).name,
                );
                cmd_find_from_session(&raw mut fs, s, 0 as ::core::ffi::c_int);
            }
        }
        1 => {
            s = session_find_by_id((*item).session as u_int);
            if !s.is_null() {
                wl = winlink_find_by_index(&raw mut (*s).windows, (*item).winlink);
                if !s.is_null() && !wl.is_null() {
                    xasprintf(
                        &raw mut target,
                        b"=%s:%u.\0" as *const u8 as *const ::core::ffi::c_char,
                        (*s).name,
                        (*wl).idx,
                    );
                    cmd_find_from_winlink(&raw mut fs, wl, 0 as ::core::ffi::c_int);
                }
            }
        }
        _ => {}
    }
    if target.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    command = cmd_template_replace((*data).command, target, 1 as ::core::ffi::c_int);
    if !command.is_null() && *command as ::core::ffi::c_int != '\0' as i32 {
        state = cmdq_new_state(
            &raw mut fs,
            ::core::ptr::null_mut::<key_event>(),
            0 as ::core::ffi::c_int,
        );
        status = cmd_parse_and_append(
            command,
            ::core::ptr::null_mut::<cmd_parse_input>(),
            c,
            state,
            &raw mut error,
        );
        if status as ::core::ffi::c_uint
            == CMD_PARSE_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if !c.is_null() {
                *error = ({
                    let mut __res: ::core::ffi::c_int = 0;
                    if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                        if 0 != 0 {
                            let mut __c: ::core::ffi::c_int =
                                *error as u_char as ::core::ffi::c_int;
                            __res = (if __c < -(128 as ::core::ffi::c_int)
                                || __c > 255 as ::core::ffi::c_int
                            {
                                __c as __int32_t
                            } else {
                                *(*__ctype_toupper_loc()).offset(__c as isize)
                            }) as ::core::ffi::c_int;
                        } else {
                            __res = toupper(*error as u_char as ::core::ffi::c_int);
                        }
                    } else {
                        __res = *(*__ctype_toupper_loc())
                            .offset(*error as u_char as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int;
                    }
                    __res
                }) as ::core::ffi::c_char;
                status_message_set(
                    c,
                    -(1 as ::core::ffi::c_int),
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    error,
                );
            }
            free(error as *mut ::core::ffi::c_void);
        }
        cmdq_free_state(state);
    }
    free(command as *mut ::core::ffi::c_void);
    free(target as *mut ::core::ffi::c_void);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_switch_prompt_callback(
    mut arg: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut data: *mut window_switch_modedata = arg as *mut window_switch_modedata;
    if key as ::core::ffi::c_uint != PROMPT_KEY_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return PROMPT_CONTINUE;
    }
    if s.is_null() {
        s = b"\0" as *const u8 as *const ::core::ffi::c_char;
    } else if *s as ::core::ffi::c_int != '\0' as i32 {
        s = s.offset(1);
    }
    free((*data).filter as *mut ::core::ffi::c_void);
    (*data).filter = xstrdup(s);
    window_switch_build(data);
    (*data).current = 0 as u_int;
    (*data).offset = 0 as u_int;
    return PROMPT_CONTINUE;
}
unsafe extern "C" fn window_switch_key(
    mut wme: *mut window_mode_entry,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    let mut current_block: u64;
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_switch_modedata = (*wme).data as *mut window_switch_modedata;
    let mut visible: u_int = 0;
    let mut current: u_int = (*data).current;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut size: u_int = (*data).matches_size;
    let mut result: prompt_key_result = PROMPT_KEY_NOT_HANDLED;
    let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        if m.is_null()
            || cmd_mouse_at(wp, m, &raw mut x, &raw mut y, 0 as ::core::ffi::c_int)
                != 0 as ::core::ffi::c_int
        {
            return;
        }
        if !(*data).prompt.is_null()
            && (*(*data).screen.grid).sy != 0 as u_int
            && y == (*(*data).screen.grid).sy.wrapping_sub(1 as u_int)
            && (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_1 as u_int
            && (*m).b & MOUSE_MASK_DRAG as u_int == 0
            && !((*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int)
        {
            result = prompt_mouse(
                (*data).prompt,
                x,
                0 as u_int,
                (*(*data).screen.grid).sx,
                &raw mut redraw,
            );
            if redraw != 0
                || result as ::core::ffi::c_uint
                    == PROMPT_KEY_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_switch_draw_screen(wme);
                (*wp).flags |= PANE_REDRAW;
            }
            return;
        }
        match key {
            38654705664 => {
                if size != 0 as u_int && current != 0 as u_int {
                    window_switch_set_current(data, current.wrapping_sub(1 as u_int));
                }
            }
            34359738368 => {
                if size != 0 as u_int && current != size.wrapping_sub(1 as u_int) {
                    window_switch_set_current(data, current.wrapping_add(1 as u_int));
                }
            }
            17179869440 | 47244640512 => {
                if y >= window_switch_visible(data) || (*data).offset.wrapping_add(y) >= size {
                    return;
                }
                window_switch_set_current(data, (*data).offset.wrapping_add(y));
                if key == KEYC_DOUBLECLICK1_PANE as ::core::ffi::c_ulong as key_code {
                    if window_switch_run_command(data, c) != 0 {
                        window_pane_reset_mode(wp);
                    }
                    return;
                }
            }
            _ => return,
        }
    } else {
        match key {
            35184372088944 | 35184372088939 => {
                key = KEYC_UP as ::core::ffi::c_ulong as key_code;
            }
            35184372088942 | 35184372088938 => {
                key = KEYC_DOWN as ::core::ffi::c_ulong as key_code;
            }
            _ => {}
        }
        match key {
            13 => {
                if window_switch_run_command(data, c) != 0 {
                    window_pane_reset_mode(wp);
                }
                return;
            }
            27 | 35184372088923 | 35184372088931 | 35184372088935 => {
                window_pane_reset_mode(wp);
                return;
            }
            _ => {}
        }
        if !(*data).prompt.is_null() {
            result = prompt_key((*data).prompt, key, &raw mut redraw);
            if redraw != 0 {
                window_switch_draw_screen(wme);
                (*wp).flags |= PANE_REDRAW;
            }
            if result as ::core::ffi::c_uint
                == PROMPT_KEY_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
                || result as ::core::ffi::c_uint
                    == PROMPT_KEY_NOT_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return;
            }
            current = (*data).current;
            size = (*data).matches_size;
        }
        match key {
            8589934619 => {
                current_block = 2855313901578758422;
                match current_block {
                    10411635476396469942 => {
                        if size > 0 as u_int {
                            window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                        }
                    }
                    6377616723418564240 => {
                        if !(size == 0 as u_int) {
                            if current == size.wrapping_sub(1 as u_int) {
                                window_switch_set_current(data, 0 as u_int);
                            } else {
                                window_switch_set_current(data, current.wrapping_add(1 as u_int));
                            }
                        }
                    }
                    10953541916736708013 => {
                        visible = window_switch_visible(data);
                        if current >= visible {
                            window_switch_set_current(data, current.wrapping_sub(visible));
                        } else {
                            window_switch_set_current(data, 0 as u_int);
                        }
                    }
                    14247072322348479886 => {
                        visible = window_switch_visible(data);
                        window_switch_set_current(data, current.wrapping_add(visible));
                    }
                    12328944762703010638 => {
                        window_switch_set_current(data, 0 as u_int);
                    }
                    _ => {
                        if !(size == 0 as u_int) {
                            if current == 0 as u_int {
                                window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                            } else {
                                window_switch_set_current(data, current.wrapping_sub(1 as u_int));
                            }
                        }
                    }
                }
            }
            8589934620 => {
                current_block = 6377616723418564240;
                match current_block {
                    10411635476396469942 => {
                        if size > 0 as u_int {
                            window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                        }
                    }
                    6377616723418564240 => {
                        if !(size == 0 as u_int) {
                            if current == size.wrapping_sub(1 as u_int) {
                                window_switch_set_current(data, 0 as u_int);
                            } else {
                                window_switch_set_current(data, current.wrapping_add(1 as u_int));
                            }
                        }
                    }
                    10953541916736708013 => {
                        visible = window_switch_visible(data);
                        if current >= visible {
                            window_switch_set_current(data, current.wrapping_sub(visible));
                        } else {
                            window_switch_set_current(data, 0 as u_int);
                        }
                    }
                    14247072322348479886 => {
                        visible = window_switch_visible(data);
                        window_switch_set_current(data, current.wrapping_add(visible));
                    }
                    12328944762703010638 => {
                        window_switch_set_current(data, 0 as u_int);
                    }
                    _ => {
                        if !(size == 0 as u_int) {
                            if current == 0 as u_int {
                                window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                            } else {
                                window_switch_set_current(data, current.wrapping_sub(1 as u_int));
                            }
                        }
                    }
                }
            }
            8589934617 => {
                current_block = 10953541916736708013;
                match current_block {
                    10411635476396469942 => {
                        if size > 0 as u_int {
                            window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                        }
                    }
                    6377616723418564240 => {
                        if !(size == 0 as u_int) {
                            if current == size.wrapping_sub(1 as u_int) {
                                window_switch_set_current(data, 0 as u_int);
                            } else {
                                window_switch_set_current(data, current.wrapping_add(1 as u_int));
                            }
                        }
                    }
                    10953541916736708013 => {
                        visible = window_switch_visible(data);
                        if current >= visible {
                            window_switch_set_current(data, current.wrapping_sub(visible));
                        } else {
                            window_switch_set_current(data, 0 as u_int);
                        }
                    }
                    14247072322348479886 => {
                        visible = window_switch_visible(data);
                        window_switch_set_current(data, current.wrapping_add(visible));
                    }
                    12328944762703010638 => {
                        window_switch_set_current(data, 0 as u_int);
                    }
                    _ => {
                        if !(size == 0 as u_int) {
                            if current == 0 as u_int {
                                window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                            } else {
                                window_switch_set_current(data, current.wrapping_sub(1 as u_int));
                            }
                        }
                    }
                }
            }
            8589934616 => {
                current_block = 14247072322348479886;
                match current_block {
                    10411635476396469942 => {
                        if size > 0 as u_int {
                            window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                        }
                    }
                    6377616723418564240 => {
                        if !(size == 0 as u_int) {
                            if current == size.wrapping_sub(1 as u_int) {
                                window_switch_set_current(data, 0 as u_int);
                            } else {
                                window_switch_set_current(data, current.wrapping_add(1 as u_int));
                            }
                        }
                    }
                    10953541916736708013 => {
                        visible = window_switch_visible(data);
                        if current >= visible {
                            window_switch_set_current(data, current.wrapping_sub(visible));
                        } else {
                            window_switch_set_current(data, 0 as u_int);
                        }
                    }
                    14247072322348479886 => {
                        visible = window_switch_visible(data);
                        window_switch_set_current(data, current.wrapping_add(visible));
                    }
                    12328944762703010638 => {
                        window_switch_set_current(data, 0 as u_int);
                    }
                    _ => {
                        if !(size == 0 as u_int) {
                            if current == 0 as u_int {
                                window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                            } else {
                                window_switch_set_current(data, current.wrapping_sub(1 as u_int));
                            }
                        }
                    }
                }
            }
            8589934614 => {
                current_block = 12328944762703010638;
                match current_block {
                    10411635476396469942 => {
                        if size > 0 as u_int {
                            window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                        }
                    }
                    6377616723418564240 => {
                        if !(size == 0 as u_int) {
                            if current == size.wrapping_sub(1 as u_int) {
                                window_switch_set_current(data, 0 as u_int);
                            } else {
                                window_switch_set_current(data, current.wrapping_add(1 as u_int));
                            }
                        }
                    }
                    10953541916736708013 => {
                        visible = window_switch_visible(data);
                        if current >= visible {
                            window_switch_set_current(data, current.wrapping_sub(visible));
                        } else {
                            window_switch_set_current(data, 0 as u_int);
                        }
                    }
                    14247072322348479886 => {
                        visible = window_switch_visible(data);
                        window_switch_set_current(data, current.wrapping_add(visible));
                    }
                    12328944762703010638 => {
                        window_switch_set_current(data, 0 as u_int);
                    }
                    _ => {
                        if !(size == 0 as u_int) {
                            if current == 0 as u_int {
                                window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                            } else {
                                window_switch_set_current(data, current.wrapping_sub(1 as u_int));
                            }
                        }
                    }
                }
            }
            8589934615 => {
                current_block = 10411635476396469942;
                match current_block {
                    10411635476396469942 => {
                        if size > 0 as u_int {
                            window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                        }
                    }
                    6377616723418564240 => {
                        if !(size == 0 as u_int) {
                            if current == size.wrapping_sub(1 as u_int) {
                                window_switch_set_current(data, 0 as u_int);
                            } else {
                                window_switch_set_current(data, current.wrapping_add(1 as u_int));
                            }
                        }
                    }
                    10953541916736708013 => {
                        visible = window_switch_visible(data);
                        if current >= visible {
                            window_switch_set_current(data, current.wrapping_sub(visible));
                        } else {
                            window_switch_set_current(data, 0 as u_int);
                        }
                    }
                    14247072322348479886 => {
                        visible = window_switch_visible(data);
                        window_switch_set_current(data, current.wrapping_add(visible));
                    }
                    12328944762703010638 => {
                        window_switch_set_current(data, 0 as u_int);
                    }
                    _ => {
                        if !(size == 0 as u_int) {
                            if current == 0 as u_int {
                                window_switch_set_current(data, size.wrapping_sub(1 as u_int));
                            } else {
                                window_switch_set_current(data, current.wrapping_sub(1 as u_int));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    window_switch_draw_screen(wme);
    (*wp).flags |= PANE_REDRAW;
}

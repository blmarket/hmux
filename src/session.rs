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
    pub type event_payload;
    fn gettimeofday(__tv: *mut timeval, __tz: *mut ::core::ffi::c_void) -> ::core::ffi::c_int;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn event_add(ev: *mut event, timeout: *const timeval) -> ::core::ffi::c_int;
    fn event_del(_: *mut event) -> ::core::ffi::c_int;
    fn event_initialized(ev: *const event) -> ::core::ffi::c_int;
    fn event_once(
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
        _: *const timeval,
    ) -> ::core::ffi::c_int;
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
    fn strtonum(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    static mut global_options: *mut options;
    fn sort_get_sessions(_: *mut u_int, _: *mut sort_criteria) -> *mut *mut session;
    fn event_payload_create() -> *mut event_payload;
    fn event_payload_set_target(_: *mut event_payload, _: *mut cmd_find_state);
    fn event_payload_set_string(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn event_payload_set_int(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    );
    fn event_payload_set_uint(_: *mut event_payload, _: *const ::core::ffi::c_char, _: u_int);
    fn event_payload_set_session(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut session,
    );
    fn event_payload_set_window(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut window,
    );
    fn events_fire(_: *const ::core::ffi::c_char, _: *mut event_payload);
    fn events_fire_session(_: *const ::core::ffi::c_char, _: *mut session);
    fn events_fire_winlink(_: *const ::core::ffi::c_char, _: *mut winlink);
    fn options_free(_: *mut options);
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn environ_free(_: *mut environ);
    fn tty_update_window_offset(_: *mut window);
    fn cmd_find_from_session(_: *mut cmd_find_state, _: *mut session, _: ::core::ffi::c_int);
    fn cmd_find_from_winlink(_: *mut cmd_find_state, _: *mut winlink, _: ::core::ffi::c_int);
    static mut marked_pane: cmd_find_state;
    fn server_clear_marked();
    fn server_lock_session(_: *mut session);
    fn status_update_cache(_: *mut session);
    fn recalculate_sizes();
    fn grid_collect_history(_: *mut grid, _: ::core::ffi::c_int);
    fn winlinks_RB_MINMAX(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn winlinks_RB_NEXT(_: *mut winlink) -> *mut winlink;
    fn winlink_find_by_index(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn winlink_find_by_window(_: *mut winlinks, _: *mut window) -> *mut winlink;
    fn winlink_find_by_window_id(_: *mut winlinks, _: u_int) -> *mut winlink;
    fn winlink_add(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn winlink_set_window(_: *mut winlink, _: *mut window);
    fn winlink_remove(_: *mut winlinks, _: *mut winlink);
    fn winlink_next(_: *mut winlink) -> *mut winlink;
    fn winlink_previous(_: *mut winlink) -> *mut winlink;
    fn winlink_stack_push(_: *mut winlink_stack, _: *mut winlink);
    fn winlink_stack_remove(_: *mut winlink_stack, _: *mut winlink);
    fn window_update_activity(_: *mut window);
    fn window_update_focus(_: *mut window);
    fn winlink_clear_flags(_: *mut winlink);
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatal(_: *const ::core::ffi::c_char, ...) -> !;
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
pub struct session_group {
    pub name: *const ::core::ffi::c_char,
    pub sessions: C2RustUnnamed_36,
    pub entry: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub rbe_left: *mut session_group,
    pub rbe_right: *mut session_group,
    pub rbe_parent: *mut session_group,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_36 {
    pub tqh_first: *mut session,
    pub tqh_last: *mut *mut session,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct session_groups {
    pub rbh_root: *mut session_group,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sessions {
    pub rbh_root: *mut session,
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
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const RB_INF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const EV_TIMEOUT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_THEMECHANGED: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const WINLINK_BELL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINLINK_ACTIVITY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const WINLINK_SILENCE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const WINLINK_ALERTFLAGS: ::core::ffi::c_int =
    WINLINK_BELL | WINLINK_ACTIVITY | WINLINK_SILENCE;
pub const WINLINK_VISITED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
#[no_mangle]
pub static mut sessions: sessions = sessions {
    rbh_root: ::core::ptr::null::<session>() as *mut session,
};
#[no_mangle]
pub static mut next_session_id: u_int = 0;
#[no_mangle]
pub static mut session_groups: session_groups = session_groups {
    rbh_root: ::core::ptr::null::<session_group>() as *mut session_group,
};
#[no_mangle]
pub unsafe extern "C" fn session_cmp(
    mut s1: *mut session,
    mut s2: *mut session,
) -> ::core::ffi::c_int {
    return strcmp((*s1).name, (*s2).name);
}
#[no_mangle]
pub unsafe extern "C" fn sessions_RB_FIND(
    mut head: *mut sessions,
    mut elm: *mut session,
) -> *mut session {
    let mut tmp: *mut session = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = session_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<session>();
}
#[no_mangle]
pub unsafe extern "C" fn sessions_RB_REMOVE_COLOR(
    mut head: *mut sessions,
    mut parent: *mut session,
    mut elm: *mut session,
) {
    let mut tmp: *mut session = ::core::ptr::null_mut::<session>();
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
                    let mut oleft: *mut session = ::core::ptr::null_mut::<session>();
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
                    let mut oright: *mut session = ::core::ptr::null_mut::<session>();
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
#[no_mangle]
pub unsafe extern "C" fn sessions_RB_INSERT_COLOR(mut head: *mut sessions, mut elm: *mut session) {
    let mut parent: *mut session = ::core::ptr::null_mut::<session>();
    let mut gparent: *mut session = ::core::ptr::null_mut::<session>();
    let mut tmp: *mut session = ::core::ptr::null_mut::<session>();
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
#[no_mangle]
pub unsafe extern "C" fn sessions_RB_INSERT(
    mut head: *mut sessions,
    mut elm: *mut session,
) -> *mut session {
    let mut tmp: *mut session = ::core::ptr::null_mut::<session>();
    let mut parent: *mut session = ::core::ptr::null_mut::<session>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = session_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<session>();
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
    sessions_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<session>();
}
#[no_mangle]
pub unsafe extern "C" fn sessions_RB_NFIND(
    mut head: *mut sessions,
    mut elm: *mut session,
) -> *mut session {
    let mut tmp: *mut session = (*head).rbh_root;
    let mut res: *mut session = ::core::ptr::null_mut::<session>();
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = session_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            res = tmp;
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn sessions_RB_REMOVE(
    mut head: *mut sessions,
    mut elm: *mut session,
) -> *mut session {
    let mut current_block: u64;
    let mut child: *mut session = ::core::ptr::null_mut::<session>();
    let mut parent: *mut session = ::core::ptr::null_mut::<session>();
    let mut old: *mut session = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut session = ::core::ptr::null_mut::<session>();
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
        current_block = 9084725308750582075;
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
        sessions_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
#[no_mangle]
pub unsafe extern "C" fn sessions_RB_NEXT(mut elm: *mut session) -> *mut session {
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
#[no_mangle]
pub unsafe extern "C" fn sessions_RB_PREV(mut elm: *mut session) -> *mut session {
    if !(*elm).entry.rbe_left.is_null() {
        elm = (*elm).entry.rbe_left;
        while !(*elm).entry.rbe_right.is_null() {
            elm = (*elm).entry.rbe_right;
        }
    } else if !(*elm).entry.rbe_parent.is_null()
        && elm == (*(*elm).entry.rbe_parent).entry.rbe_right
    {
        elm = (*elm).entry.rbe_parent;
    } else {
        while !(*elm).entry.rbe_parent.is_null() && elm == (*(*elm).entry.rbe_parent).entry.rbe_left
        {
            elm = (*elm).entry.rbe_parent;
        }
        elm = (*elm).entry.rbe_parent;
    }
    return elm;
}
#[no_mangle]
pub unsafe extern "C" fn sessions_RB_MINMAX(
    mut head: *mut sessions,
    mut val: ::core::ffi::c_int,
) -> *mut session {
    let mut tmp: *mut session = (*head).rbh_root;
    let mut parent: *mut session = ::core::ptr::null_mut::<session>();
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
#[no_mangle]
pub unsafe extern "C" fn session_group_cmp(
    mut s1: *mut session_group,
    mut s2: *mut session_group,
) -> ::core::ffi::c_int {
    return strcmp((*s1).name, (*s2).name);
}
#[no_mangle]
pub unsafe extern "C" fn session_groups_RB_INSERT_COLOR(
    mut head: *mut session_groups,
    mut elm: *mut session_group,
) {
    let mut parent: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut gparent: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut tmp: *mut session_group = ::core::ptr::null_mut::<session_group>();
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
#[no_mangle]
pub unsafe extern "C" fn session_groups_RB_REMOVE_COLOR(
    mut head: *mut session_groups,
    mut parent: *mut session_group,
    mut elm: *mut session_group,
) {
    let mut tmp: *mut session_group = ::core::ptr::null_mut::<session_group>();
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
                    let mut oleft: *mut session_group = ::core::ptr::null_mut::<session_group>();
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
                    let mut oright: *mut session_group = ::core::ptr::null_mut::<session_group>();
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
#[no_mangle]
pub unsafe extern "C" fn session_groups_RB_PREV(mut elm: *mut session_group) -> *mut session_group {
    if !(*elm).entry.rbe_left.is_null() {
        elm = (*elm).entry.rbe_left;
        while !(*elm).entry.rbe_right.is_null() {
            elm = (*elm).entry.rbe_right;
        }
    } else if !(*elm).entry.rbe_parent.is_null()
        && elm == (*(*elm).entry.rbe_parent).entry.rbe_right
    {
        elm = (*elm).entry.rbe_parent;
    } else {
        while !(*elm).entry.rbe_parent.is_null() && elm == (*(*elm).entry.rbe_parent).entry.rbe_left
        {
            elm = (*elm).entry.rbe_parent;
        }
        elm = (*elm).entry.rbe_parent;
    }
    return elm;
}
#[no_mangle]
pub unsafe extern "C" fn session_groups_RB_REMOVE(
    mut head: *mut session_groups,
    mut elm: *mut session_group,
) -> *mut session_group {
    let mut current_block: u64;
    let mut child: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut parent: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut old: *mut session_group = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut session_group = ::core::ptr::null_mut::<session_group>();
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
        current_block = 17309243258499378356;
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
        session_groups_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
#[no_mangle]
pub unsafe extern "C" fn session_groups_RB_INSERT(
    mut head: *mut session_groups,
    mut elm: *mut session_group,
) -> *mut session_group {
    let mut tmp: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut parent: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = session_group_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<session_group>();
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
    session_groups_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<session_group>();
}
#[no_mangle]
pub unsafe extern "C" fn session_groups_RB_FIND(
    mut head: *mut session_groups,
    mut elm: *mut session_group,
) -> *mut session_group {
    let mut tmp: *mut session_group = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = session_group_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<session_group>();
}
#[no_mangle]
pub unsafe extern "C" fn session_groups_RB_NFIND(
    mut head: *mut session_groups,
    mut elm: *mut session_group,
) -> *mut session_group {
    let mut tmp: *mut session_group = (*head).rbh_root;
    let mut res: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = session_group_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            res = tmp;
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn session_groups_RB_NEXT(mut elm: *mut session_group) -> *mut session_group {
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
#[no_mangle]
pub unsafe extern "C" fn session_groups_RB_MINMAX(
    mut head: *mut session_groups,
    mut val: ::core::ffi::c_int,
) -> *mut session_group {
    let mut tmp: *mut session_group = (*head).rbh_root;
    let mut parent: *mut session_group = ::core::ptr::null_mut::<session_group>();
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
#[no_mangle]
pub unsafe extern "C" fn session_alive(mut s: *mut session) -> ::core::ffi::c_int {
    let mut s_loop: *mut session = ::core::ptr::null_mut::<session>();
    s_loop = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s_loop.is_null() {
        if s_loop == s {
            return 1 as ::core::ffi::c_int;
        }
        s_loop = sessions_RB_NEXT(s_loop);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn session_find(mut name: *const ::core::ffi::c_char) -> *mut session {
    let mut s: session = session {
        id: 0,
        name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cwd: ::core::ptr::null::<::core::ffi::c_char>(),
        creation_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        last_attached_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        activity_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        last_activity_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        lock_timer: event {
            ev_evcallback: event_callback {
                evcb_active_next: C2RustUnnamed_9 {
                    tqe_next: ::core::ptr::null_mut::<event_callback>(),
                    tqe_prev: ::core::ptr::null_mut::<*mut event_callback>(),
                },
                evcb_flags: 0,
                evcb_pri: 0,
                evcb_closure: 0,
                evcb_cb_union: C2RustUnnamed_8 {
                    evcb_callback: None,
                },
                evcb_arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
            ev_timeout_pos: C2RustUnnamed_6 {
                ev_next_with_common_timeout: C2RustUnnamed_7 {
                    tqe_next: ::core::ptr::null_mut::<event>(),
                    tqe_prev: ::core::ptr::null_mut::<*mut event>(),
                },
            },
            ev_fd: 0,
            ev_base: ::core::ptr::null_mut::<event_base>(),
            ev_: C2RustUnnamed_1 {
                ev_io: C2RustUnnamed_4 {
                    ev_io_next: C2RustUnnamed_5 {
                        le_next: ::core::ptr::null_mut::<event>(),
                        le_prev: ::core::ptr::null_mut::<*mut event>(),
                    },
                    ev_timeout: timeval {
                        tv_sec: 0,
                        tv_usec: 0,
                    },
                },
            },
            ev_events: 0,
            ev_res: 0,
            ev_timeout: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
        },
        curw: ::core::ptr::null_mut::<winlink>(),
        lastw: winlink_stack {
            tqh_first: ::core::ptr::null_mut::<winlink>(),
            tqh_last: ::core::ptr::null_mut::<*mut winlink>(),
        },
        windows: winlinks {
            rbh_root: ::core::ptr::null_mut::<winlink>(),
        },
        statusat: 0,
        statuslines: 0,
        options: ::core::ptr::null_mut::<options>(),
        flags: 0,
        attached: 0,
        tio: ::core::ptr::null_mut::<termios>(),
        environ: ::core::ptr::null_mut::<environ>(),
        references: 0,
        gentry: C2RustUnnamed_15 {
            tqe_next: ::core::ptr::null_mut::<session>(),
            tqe_prev: ::core::ptr::null_mut::<*mut session>(),
        },
        entry: C2RustUnnamed_14 {
            rbe_left: ::core::ptr::null_mut::<session>(),
            rbe_right: ::core::ptr::null_mut::<session>(),
            rbe_parent: ::core::ptr::null_mut::<session>(),
            rbe_color: 0,
        },
    };
    s.name = name as *mut ::core::ffi::c_char;
    return sessions_RB_FIND(&raw mut sessions, &raw mut s);
}
#[no_mangle]
pub unsafe extern "C" fn session_find_by_id_str(mut s: *const ::core::ffi::c_char) -> *mut session {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: u_int = 0;
    if *s as ::core::ffi::c_int != '$' as i32 {
        return ::core::ptr::null_mut::<session>();
    }
    id = strtonum(
        s.offset(1 as ::core::ffi::c_int as isize),
        0 as ::core::ffi::c_longlong,
        UINT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as u_int;
    if !errstr.is_null() {
        return ::core::ptr::null_mut::<session>();
    }
    return session_find_by_id(id);
}
#[no_mangle]
pub unsafe extern "C" fn session_find_by_id(mut id: u_int) -> *mut session {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        if (*s).id == id {
            return s;
        }
        s = sessions_RB_NEXT(s);
    }
    return ::core::ptr::null_mut::<session>();
}
#[no_mangle]
pub unsafe extern "C" fn session_create(
    mut prefix: *const ::core::ffi::c_char,
    mut name: *const ::core::ffi::c_char,
    mut cwd: *const ::core::ffi::c_char,
    mut env: *mut environ,
    mut oo: *mut options,
    mut tio: *mut termios,
) -> *mut session {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    s = xcalloc(1 as size_t, ::core::mem::size_of::<session>() as size_t) as *mut session;
    (*s).references = 1 as ::core::ffi::c_int;
    (*s).flags = 0 as ::core::ffi::c_int;
    (*s).cwd = xstrdup(cwd);
    (*s).lastw.tqh_first = ::core::ptr::null_mut::<winlink>();
    (*s).lastw.tqh_last = &raw mut (*s).lastw.tqh_first;
    (*s).windows.rbh_root = ::core::ptr::null_mut::<winlink>();
    (*s).environ = env;
    (*s).options = oo;
    status_update_cache(s);
    (*s).tio = ::core::ptr::null_mut::<termios>();
    if !tio.is_null() {
        (*s).tio = xmalloc(::core::mem::size_of::<termios>() as size_t) as *mut termios;
        memcpy(
            (*s).tio as *mut ::core::ffi::c_void,
            tio as *const ::core::ffi::c_void,
            ::core::mem::size_of::<termios>() as size_t,
        );
    }
    if !name.is_null() {
        (*s).name = xstrdup(name);
        let fresh0 = next_session_id;
        next_session_id = next_session_id.wrapping_add(1);
        (*s).id = fresh0;
    } else {
        loop {
            let fresh1 = next_session_id;
            next_session_id = next_session_id.wrapping_add(1);
            (*s).id = fresh1;
            free((*s).name as *mut ::core::ffi::c_void);
            if !prefix.is_null() {
                xasprintf(
                    &raw mut (*s).name,
                    b"%s-%u\0" as *const u8 as *const ::core::ffi::c_char,
                    prefix,
                    (*s).id,
                );
            } else {
                xasprintf(
                    &raw mut (*s).name,
                    b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                    (*s).id,
                );
            }
            if sessions_RB_FIND(&raw mut sessions, s).is_null() {
                break;
            }
        }
    }
    sessions_RB_INSERT(&raw mut sessions, s);
    log_debug(
        b"new session %s $%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).name,
        (*s).id,
    );
    if gettimeofday(&raw mut (*s).creation_time, NULL) != 0 as ::core::ffi::c_int {
        fatal(b"gettimeofday failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    session_update_activity(s, &raw mut (*s).creation_time);
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn session_add_ref(
    mut s: *mut session,
    mut from: *const ::core::ffi::c_char,
) {
    (*s).references += 1;
    log_debug(
        b"%s: %s %s, now %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"session_add_ref\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).name,
        from,
        (*s).references,
    );
}
#[no_mangle]
pub unsafe extern "C" fn session_remove_ref(
    mut s: *mut session,
    mut from: *const ::core::ffi::c_char,
) {
    (*s).references -= 1;
    log_debug(
        b"%s: %s %s, now %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"session_remove_ref\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).name,
        from,
        (*s).references,
    );
    if (*s).references == 0 as ::core::ffi::c_int {
        event_once(
            -(1 as ::core::ffi::c_int),
            EV_TIMEOUT as ::core::ffi::c_short,
            Some(
                session_free
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            s as *mut ::core::ffi::c_void,
            ::core::ptr::null::<timeval>(),
        );
    }
}
unsafe extern "C" fn session_free(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut s: *mut session = arg as *mut session;
    log_debug(
        b"session %s freed (%d references)\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).name,
        (*s).references,
    );
    if (*s).references == 0 as ::core::ffi::c_int {
        environ_free((*s).environ);
        options_free((*s).options);
        free((*s).name as *mut ::core::ffi::c_void);
        free(s as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_destroy(
    mut s: *mut session,
    mut notify: ::core::ffi::c_int,
    mut from: *const ::core::ffi::c_char,
) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    log_debug(
        b"session %s destroyed (%s)\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).name,
        from,
    );
    if (*s).curw.is_null() {
        return;
    }
    (*s).curw = ::core::ptr::null_mut::<winlink>();
    sessions_RB_REMOVE(&raw mut sessions, s);
    if notify != 0 {
        events_fire_session(
            b"session-closed\0" as *const u8 as *const ::core::ffi::c_char,
            s,
        );
    }
    free((*s).tio as *mut ::core::ffi::c_void);
    if event_initialized(&raw mut (*s).lock_timer) != 0 {
        event_del(&raw mut (*s).lock_timer);
    }
    session_group_remove(s);
    while !(*s).lastw.tqh_first.is_null() {
        winlink_stack_remove(&raw mut (*s).lastw, (*s).lastw.tqh_first);
    }
    while !(*s).windows.rbh_root.is_null() {
        wl = (*s).windows.rbh_root;
        events_fire_winlink(
            b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
            wl,
        );
        winlink_remove(&raw mut (*s).windows, wl);
    }
    free((*s).cwd as *mut ::core::ffi::c_void);
    session_remove_ref(
        s,
        b"session_destroy\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
unsafe extern "C" fn session_lock_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut s: *mut session = arg as *mut session;
    if (*s).attached == 0 as u_int {
        return;
    }
    log_debug(
        b"session %s locked, activity time %lld\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).name,
        (*s).activity_time.tv_sec as ::core::ffi::c_longlong,
    );
    server_lock_session(s);
    recalculate_sizes();
}
#[no_mangle]
pub unsafe extern "C" fn session_update_activity(mut s: *mut session, mut from: *mut timeval) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if from.is_null() {
        gettimeofday(&raw mut (*s).activity_time, NULL);
    } else {
        memcpy(
            &raw mut (*s).activity_time as *mut ::core::ffi::c_void,
            from as *const ::core::ffi::c_void,
            ::core::mem::size_of::<timeval>() as size_t,
        );
    }
    log_debug(
        b"session $%u %s activity %lld.%06d\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).id,
        (*s).name,
        (*s).activity_time.tv_sec as ::core::ffi::c_longlong,
        (*s).activity_time.tv_usec as ::core::ffi::c_int,
    );
    if event_initialized(&raw mut (*s).lock_timer) != 0 {
        event_del(&raw mut (*s).lock_timer);
    } else {
        event_set(
            &raw mut (*s).lock_timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            Some(
                session_lock_timer
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            s as *mut ::core::ffi::c_void,
        );
    }
    if (*s).attached != 0 as u_int {
        tv.tv_usec = 0 as __suseconds_t;
        tv.tv_sec = tv.tv_usec as __time_t;
        tv.tv_sec = options_get_number(
            (*s).options,
            b"lock-after-time\0" as *const u8 as *const ::core::ffi::c_char,
        ) as __time_t;
        if tv.tv_sec != 0 as __time_t {
            event_add(&raw mut (*s).lock_timer, &raw mut tv);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_next_session(
    mut s: *mut session,
    mut sort_crit: *mut sort_criteria,
) -> *mut session {
    let mut l: *mut *mut session = ::core::ptr::null_mut::<*mut session>();
    let mut n: u_int = 0;
    let mut i: u_int = 0;
    if sessions.rbh_root.is_null() || session_alive(s) == 0 {
        return ::core::ptr::null_mut::<session>();
    }
    l = sort_get_sessions(&raw mut n, sort_crit);
    i = 0 as u_int;
    while i < n {
        if *l.offset(i as isize) == s {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i == n {
        fatalx(
            b"session %s not found in sorted list\0" as *const u8 as *const ::core::ffi::c_char,
            (*s).name,
        );
    }
    i = i.wrapping_add(1);
    if i == n {
        i = 0 as u_int;
    }
    return *l.offset(i as isize);
}
#[no_mangle]
pub unsafe extern "C" fn session_previous_session(
    mut s: *mut session,
    mut sort_crit: *mut sort_criteria,
) -> *mut session {
    let mut l: *mut *mut session = ::core::ptr::null_mut::<*mut session>();
    let mut n: u_int = 0;
    let mut i: u_int = 0;
    if sessions.rbh_root.is_null() || session_alive(s) == 0 {
        return ::core::ptr::null_mut::<session>();
    }
    l = sort_get_sessions(&raw mut n, sort_crit);
    i = 0 as u_int;
    while i < n {
        if *l.offset(i as isize) == s {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i == n {
        fatalx(
            b"session %s not found in sorted list\0" as *const u8 as *const ::core::ffi::c_char,
            (*s).name,
        );
    }
    if i == 0 as u_int {
        i = n;
    }
    i = i.wrapping_sub(1);
    return *l.offset(i as isize);
}
#[no_mangle]
pub unsafe extern "C" fn session_attach(
    mut s: *mut session,
    mut w: *mut window,
    mut idx: ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut winlink {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = winlink_add(&raw mut (*s).windows, idx);
    if wl.is_null() {
        xasprintf(
            cause,
            b"index in use: %d\0" as *const u8 as *const ::core::ffi::c_char,
            idx,
        );
        return ::core::ptr::null_mut::<winlink>();
    }
    (*wl).session = s;
    winlink_set_window(wl, w);
    events_fire_winlink(
        b"window-linked\0" as *const u8 as *const ::core::ffi::c_char,
        wl,
    );
    session_group_synchronize_from(s);
    return wl;
}
#[no_mangle]
pub unsafe extern "C" fn session_detach(
    mut s: *mut session,
    mut wl: *mut winlink,
) -> ::core::ffi::c_int {
    if (*s).curw == wl
        && session_last(s) != 0 as ::core::ffi::c_int
        && session_previous(s, 0 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int
    {
        session_next(s, 0 as ::core::ffi::c_int);
    }
    (*wl).flags &= !WINLINK_ALERTFLAGS;
    events_fire_winlink(
        b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
        wl,
    );
    winlink_stack_remove(&raw mut (*s).lastw, wl);
    winlink_remove(&raw mut (*s).windows, wl);
    session_group_synchronize_from(s);
    if (*s).windows.rbh_root.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn session_has(
    mut s: *mut session,
    mut w: *mut window,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if (*wl).session == s {
            return 1 as ::core::ffi::c_int;
        }
        wl = (*wl).wentry.tqe_next;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn session_is_linked(
    mut s: *mut session,
    mut w: *mut window,
) -> ::core::ffi::c_int {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains(s);
    if !sg.is_null() {
        return ((*w).references != session_group_count(sg)) as ::core::ffi::c_int;
    }
    return ((*w).references != 1 as u_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn session_next_alert(mut wl: *mut winlink) -> *mut winlink {
    while !wl.is_null() {
        if (*wl).flags & WINLINK_ALERTFLAGS != 0 {
            break;
        }
        wl = winlink_next(wl);
    }
    return wl;
}
#[no_mangle]
pub unsafe extern "C" fn session_next(
    mut s: *mut session,
    mut alert: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if (*s).curw.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    wl = winlink_next((*s).curw);
    if alert != 0 {
        wl = session_next_alert(wl);
    }
    if wl.is_null() {
        wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
        if alert != 0 && {
            wl = session_next_alert(wl);
            wl.is_null()
        } {
            return -(1 as ::core::ffi::c_int);
        }
    }
    return session_set_current(s, wl);
}
unsafe extern "C" fn session_previous_alert(mut wl: *mut winlink) -> *mut winlink {
    while !wl.is_null() {
        if (*wl).flags & WINLINK_ALERTFLAGS != 0 {
            break;
        }
        wl = winlink_previous(wl);
    }
    return wl;
}
#[no_mangle]
pub unsafe extern "C" fn session_previous(
    mut s: *mut session,
    mut alert: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if (*s).curw.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    wl = winlink_previous((*s).curw);
    if alert != 0 {
        wl = session_previous_alert(wl);
    }
    if wl.is_null() {
        wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_INF);
        if alert != 0 && {
            wl = session_previous_alert(wl);
            wl.is_null()
        } {
            return -(1 as ::core::ffi::c_int);
        }
    }
    return session_set_current(s, wl);
}
#[no_mangle]
pub unsafe extern "C" fn session_select(
    mut s: *mut session,
    mut idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = winlink_find_by_index(&raw mut (*s).windows, idx);
    return session_set_current(s, wl);
}
#[no_mangle]
pub unsafe extern "C" fn session_last(mut s: *mut session) -> ::core::ffi::c_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = (*s).lastw.tqh_first;
    if wl.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if wl == (*s).curw {
        return 1 as ::core::ffi::c_int;
    }
    return session_set_current(s, wl);
}
unsafe extern "C" fn session_fire_window_changed(
    mut s: *mut session,
    mut wl: *mut winlink,
    mut old: *mut winlink,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_winlink(&raw mut fs, wl, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_session(
        ep,
        b"session\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).window,
    );
    event_payload_set_window(
        ep,
        b"new_window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).window,
    );
    event_payload_set_int(
        ep,
        b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).idx,
    );
    event_payload_set_int(
        ep,
        b"new_window_index\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).idx,
    );
    if !old.is_null() {
        event_payload_set_window(
            ep,
            b"old_window\0" as *const u8 as *const ::core::ffi::c_char,
            (*old).window,
        );
        event_payload_set_int(
            ep,
            b"old_window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*old).idx,
        );
    }
    events_fire(
        b"session-window-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
#[no_mangle]
pub unsafe extern "C" fn session_set_current(
    mut s: *mut session,
    mut wl: *mut winlink,
) -> ::core::ffi::c_int {
    let mut old: *mut winlink = (*s).curw;
    if wl.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if wl == (*s).curw {
        return 1 as ::core::ffi::c_int;
    }
    winlink_stack_remove(&raw mut (*s).lastw, wl);
    winlink_stack_push(&raw mut (*s).lastw, (*s).curw);
    (*s).curw = wl;
    if options_get_number(
        global_options,
        b"focus-events\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        if !old.is_null() {
            window_update_focus((*old).window);
        }
        window_update_focus((*wl).window);
    }
    winlink_clear_flags(wl);
    window_update_activity((*wl).window);
    tty_update_window_offset((*wl).window);
    session_fire_window_changed(s, wl, old);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn session_group_contains(mut target: *mut session) -> *mut session_group {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    sg = session_groups_RB_MINMAX(&raw mut session_groups, RB_NEGINF);
    while !sg.is_null() {
        s = (*sg).sessions.tqh_first;
        while !s.is_null() {
            if s == target {
                return sg;
            }
            s = (*s).gentry.tqe_next;
        }
        sg = session_groups_RB_NEXT(sg);
    }
    return ::core::ptr::null_mut::<session_group>();
}
#[no_mangle]
pub unsafe extern "C" fn session_group_find(
    mut name: *const ::core::ffi::c_char,
) -> *mut session_group {
    let mut sg: session_group = session_group {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        sessions: C2RustUnnamed_36 {
            tqh_first: ::core::ptr::null_mut::<session>(),
            tqh_last: ::core::ptr::null_mut::<*mut session>(),
        },
        entry: C2RustUnnamed_35 {
            rbe_left: ::core::ptr::null_mut::<session_group>(),
            rbe_right: ::core::ptr::null_mut::<session_group>(),
            rbe_parent: ::core::ptr::null_mut::<session_group>(),
            rbe_color: 0,
        },
    };
    sg.name = name;
    return session_groups_RB_FIND(&raw mut session_groups, &raw mut sg);
}
#[no_mangle]
pub unsafe extern "C" fn session_group_new(
    mut name: *const ::core::ffi::c_char,
) -> *mut session_group {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_find(name);
    if !sg.is_null() {
        return sg;
    }
    sg = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<session_group>() as size_t,
    ) as *mut session_group;
    (*sg).name = xstrdup(name);
    (*sg).sessions.tqh_first = ::core::ptr::null_mut::<session>();
    (*sg).sessions.tqh_last = &raw mut (*sg).sessions.tqh_first;
    session_groups_RB_INSERT(&raw mut session_groups, sg);
    return sg;
}
unsafe extern "C" fn session_group_fire(
    mut name: *const ::core::ffi::c_char,
    mut sg: *mut session_group,
    mut s: *mut session,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    ep = event_payload_create();
    if session_alive(s) != 0 {
        cmd_find_from_session(&raw mut fs, s, 0 as ::core::ffi::c_int);
        event_payload_set_target(ep, &raw mut fs);
    }
    event_payload_set_session(
        ep,
        b"session\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    event_payload_set_string(
        ep,
        b"group\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        (*sg).name,
    );
    event_payload_set_uint(
        ep,
        b"group_size\0" as *const u8 as *const ::core::ffi::c_char,
        session_group_count(sg),
    );
    events_fire(name, ep);
}
#[no_mangle]
pub unsafe extern "C" fn session_group_add(mut sg: *mut session_group, mut s: *mut session) {
    if session_group_contains(s).is_null() {
        (*s).gentry.tqe_next = ::core::ptr::null_mut::<session>();
        (*s).gentry.tqe_prev = (*sg).sessions.tqh_last;
        *(*sg).sessions.tqh_last = s;
        (*sg).sessions.tqh_last = &raw mut (*s).gentry.tqe_next;
        session_group_fire(
            b"session-added-to-group\0" as *const u8 as *const ::core::ffi::c_char,
            sg,
            s,
        );
    }
}
unsafe extern "C" fn session_group_remove(mut s: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains(s);
    if sg.is_null() {
        return;
    }
    session_group_fire(
        b"session-removed-from-group\0" as *const u8 as *const ::core::ffi::c_char,
        sg,
        s,
    );
    if !(*s).gentry.tqe_next.is_null() {
        (*(*s).gentry.tqe_next).gentry.tqe_prev = (*s).gentry.tqe_prev;
    } else {
        (*sg).sessions.tqh_last = (*s).gentry.tqe_prev;
    }
    *(*s).gentry.tqe_prev = (*s).gentry.tqe_next;
    if (*sg).sessions.tqh_first.is_null() {
        session_groups_RB_REMOVE(&raw mut session_groups, sg);
        free((*sg).name as *mut ::core::ffi::c_void);
        free(sg as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_group_count(mut sg: *mut session_group) -> u_int {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut n: u_int = 0;
    n = 0 as u_int;
    s = (*sg).sessions.tqh_first;
    while !s.is_null() {
        n = n.wrapping_add(1);
        s = (*s).gentry.tqe_next;
    }
    return n;
}
#[no_mangle]
pub unsafe extern "C" fn session_group_attached_count(mut sg: *mut session_group) -> u_int {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut n: u_int = 0;
    n = 0 as u_int;
    s = (*sg).sessions.tqh_first;
    while !s.is_null() {
        n = n.wrapping_add((*s).attached);
        s = (*s).gentry.tqe_next;
    }
    return n;
}
#[no_mangle]
pub unsafe extern "C" fn session_group_synchronize_to(mut s: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut target: *mut session = ::core::ptr::null_mut::<session>();
    sg = session_group_contains(s);
    if sg.is_null() {
        return;
    }
    target = ::core::ptr::null_mut::<session>();
    target = (*sg).sessions.tqh_first;
    while !target.is_null() {
        if target != s {
            break;
        }
        target = (*target).gentry.tqe_next;
    }
    if !target.is_null() {
        session_group_synchronize1(target, s);
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_group_synchronize_from(mut target: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    sg = session_group_contains(target);
    if sg.is_null() {
        return;
    }
    s = (*sg).sessions.tqh_first;
    while !s.is_null() {
        if s != target {
            session_group_synchronize1(target, s);
        }
        s = (*s).gentry.tqe_next;
    }
}
unsafe extern "C" fn session_group_synchronize1(mut target: *mut session, mut s: *mut session) {
    let mut old_windows: winlinks = winlinks {
        rbh_root: ::core::ptr::null_mut::<winlink>(),
    };
    let mut ww: *mut winlinks = ::core::ptr::null_mut::<winlinks>();
    let mut old_lastw: winlink_stack = winlink_stack {
        tqh_first: ::core::ptr::null_mut::<winlink>(),
        tqh_last: ::core::ptr::null_mut::<*mut winlink>(),
    };
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wl2: *mut winlink = ::core::ptr::null_mut::<winlink>();
    ww = &raw mut (*target).windows;
    if (*ww).rbh_root.is_null() {
        return;
    }
    if !(*s).curw.is_null()
        && winlink_find_by_index(ww, (*(*s).curw).idx).is_null()
        && session_last(s) != 0 as ::core::ffi::c_int
        && session_previous(s, 0 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int
    {
        session_next(s, 0 as ::core::ffi::c_int);
    }
    memcpy(
        &raw mut old_windows as *mut ::core::ffi::c_void,
        &raw mut (*s).windows as *const ::core::ffi::c_void,
        ::core::mem::size_of::<winlinks>() as size_t,
    );
    (*s).windows.rbh_root = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_RB_MINMAX(ww, RB_NEGINF);
    while !wl.is_null() {
        wl2 = winlink_add(&raw mut (*s).windows, (*wl).idx);
        (*wl2).session = s;
        winlink_set_window(wl2, (*wl).window);
        events_fire_winlink(
            b"window-linked\0" as *const u8 as *const ::core::ffi::c_char,
            wl2,
        );
        (*wl2).flags |= (*wl).flags & WINLINK_ALERTFLAGS;
        wl = winlinks_RB_NEXT(wl);
    }
    if !(*s).curw.is_null() {
        (*s).curw = winlink_find_by_index(&raw mut (*s).windows, (*(*s).curw).idx);
    } else if !(*target).curw.is_null() {
        (*s).curw = winlink_find_by_index(&raw mut (*s).windows, (*(*target).curw).idx);
    }
    if (*s).curw.is_null() {
        (*s).curw = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
    }
    memcpy(
        &raw mut old_lastw as *mut ::core::ffi::c_void,
        &raw mut (*s).lastw as *const ::core::ffi::c_void,
        ::core::mem::size_of::<winlink_stack>() as size_t,
    );
    (*s).lastw.tqh_first = ::core::ptr::null_mut::<winlink>();
    (*s).lastw.tqh_last = &raw mut (*s).lastw.tqh_first;
    wl = old_lastw.tqh_first;
    while !wl.is_null() {
        wl2 = winlink_find_by_index(&raw mut (*s).windows, (*wl).idx);
        if !wl2.is_null() {
            (*wl2).sentry.tqe_next = ::core::ptr::null_mut::<winlink>();
            (*wl2).sentry.tqe_prev = (*s).lastw.tqh_last;
            *(*s).lastw.tqh_last = wl2;
            (*s).lastw.tqh_last = &raw mut (*wl2).sentry.tqe_next;
            (*wl2).flags |= WINLINK_VISITED;
        }
        wl = (*wl).sentry.tqe_next;
    }
    while !old_windows.rbh_root.is_null() {
        wl = old_windows.rbh_root;
        wl2 = winlink_find_by_window_id(&raw mut (*s).windows, (*(*wl).window).id);
        if wl2.is_null() {
            events_fire_winlink(
                b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
                wl,
            );
        }
        winlink_remove(&raw mut old_windows, wl);
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_renumber_windows(mut s: *mut session) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wl1: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wl_new: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut old_wins: winlinks = winlinks {
        rbh_root: ::core::ptr::null_mut::<winlink>(),
    };
    let mut old_lastw: winlink_stack = winlink_stack {
        tqh_first: ::core::ptr::null_mut::<winlink>(),
        tqh_last: ::core::ptr::null_mut::<*mut winlink>(),
    };
    let mut new_idx: ::core::ffi::c_int = 0;
    let mut new_curw_idx: ::core::ffi::c_int = 0;
    let mut marked_idx: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    memcpy(
        &raw mut old_wins as *mut ::core::ffi::c_void,
        &raw mut (*s).windows as *const ::core::ffi::c_void,
        ::core::mem::size_of::<winlinks>() as size_t,
    );
    (*s).windows.rbh_root = ::core::ptr::null_mut::<winlink>();
    new_idx = options_get_number(
        (*s).options,
        b"base-index\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    new_curw_idx = 0 as ::core::ffi::c_int;
    wl = winlinks_RB_MINMAX(&raw mut old_wins, RB_NEGINF);
    while !wl.is_null() {
        wl_new = winlink_add(&raw mut (*s).windows, new_idx);
        (*wl_new).session = s;
        winlink_set_window(wl_new, (*wl).window);
        (*wl_new).flags |= (*wl).flags & WINLINK_ALERTFLAGS;
        if wl == marked_pane.wl {
            marked_idx = (*wl_new).idx;
        }
        if wl == (*s).curw {
            new_curw_idx = (*wl_new).idx;
        }
        new_idx += 1;
        wl = winlinks_RB_NEXT(wl);
    }
    memcpy(
        &raw mut old_lastw as *mut ::core::ffi::c_void,
        &raw mut (*s).lastw as *const ::core::ffi::c_void,
        ::core::mem::size_of::<winlink_stack>() as size_t,
    );
    (*s).lastw.tqh_first = ::core::ptr::null_mut::<winlink>();
    (*s).lastw.tqh_last = &raw mut (*s).lastw.tqh_first;
    wl = old_lastw.tqh_first;
    while !wl.is_null() {
        (*wl).flags &= !WINLINK_VISITED;
        wl_new = winlink_find_by_window(&raw mut (*s).windows, (*wl).window);
        if !wl_new.is_null() {
            (*wl_new).sentry.tqe_next = ::core::ptr::null_mut::<winlink>();
            (*wl_new).sentry.tqe_prev = (*s).lastw.tqh_last;
            *(*s).lastw.tqh_last = wl_new;
            (*s).lastw.tqh_last = &raw mut (*wl_new).sentry.tqe_next;
            (*wl_new).flags |= WINLINK_VISITED;
        }
        wl = (*wl).sentry.tqe_next;
    }
    if marked_idx != -(1 as ::core::ffi::c_int) {
        marked_pane.wl = winlink_find_by_index(&raw mut (*s).windows, marked_idx);
        if marked_pane.wl.is_null() {
            server_clear_marked();
        }
    }
    (*s).curw = winlink_find_by_index(&raw mut (*s).windows, new_curw_idx);
    wl = winlinks_RB_MINMAX(&raw mut old_wins, RB_NEGINF);
    while !wl.is_null() && {
        wl1 = winlinks_RB_NEXT(wl);
        1 as ::core::ffi::c_int != 0
    } {
        winlink_remove(&raw mut old_wins, wl);
        wl = wl1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_theme_changed(mut s: *mut session) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !s.is_null() {
        wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
        while !wl.is_null() {
            wp = (*(*wl).window).panes.tqh_first;
            while !wp.is_null() {
                (*wp).flags |= PANE_THEMECHANGED;
                wp = (*wp).entry.tqe_next;
            }
            wl = winlinks_RB_NEXT(wl);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn session_update_history(mut s: *mut session) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gd: *mut grid = ::core::ptr::null_mut::<grid>();
    let mut limit: u_int = 0;
    let mut osize: u_int = 0;
    limit = options_get_number(
        (*s).options,
        b"history-limit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        wp = (*(*wl).window).panes.tqh_first;
        while !wp.is_null() {
            gd = (*wp).base.grid;
            osize = (*gd).hsize;
            (*gd).hlimit = limit;
            grid_collect_history(gd, 1 as ::core::ffi::c_int);
            if (*gd).hsize != osize {
                log_debug(
                    b"%s: %%%u %u -> %u\0" as *const u8 as *const ::core::ffi::c_char,
                    b"session_update_history\0" as *const u8 as *const ::core::ffi::c_char,
                    (*wp).id,
                    osize,
                    (*gd).hsize,
                );
            }
            wp = (*wp).entry.tqe_next;
        }
        wl = winlinks_RB_NEXT(wl);
    }
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

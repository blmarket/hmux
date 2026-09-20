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
    pub type screen_write_citem;
    fn kill(__pid: __pid_t, __sig: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn getpid() -> __pid_t;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn gettimeofday(__tv: *mut timeval, __tz: *mut ::core::ffi::c_void) -> ::core::ffi::c_int;
    fn utempter_remove_record(master_fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn bufferevent_free(bufev: *mut bufferevent);
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn sig2name(_: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn proc_send(
        _: *mut tmuxpeer,
        _: msgtype,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_void,
        _: size_t,
    ) -> ::core::ffi::c_int;
    fn format_single(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut ::core::ffi::c_char;
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
    fn event_payload_set_window(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut window,
    );
    fn event_payload_set_pane(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut window_pane,
    );
    fn events_fire(_: *const ::core::ffi::c_char, _: *mut event_payload);
    fn events_fire_winlink(_: *const ::core::ffi::c_char, _: *mut winlink);
    fn format_draw(
        _: *mut screen_write_ctx,
        _: *const grid_cell,
        _: u_int,
        _: *const ::core::ffi::c_char,
        _: *mut style_ranges,
        _: ::core::ffi::c_int,
    );
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn tty_raw(_: *mut tty, _: *const ::core::ffi::c_char);
    fn tty_stop_tty(_: *mut tty);
    fn tty_term_string(_: *mut tty_term, _: tty_code_code) -> *const ::core::ffi::c_char;
    fn cmd_find_from_pane(
        _: *mut cmd_find_state,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    static mut clients: clients;
    static mut marked_pane: cmd_find_state;
    fn server_client_set_session(_: *mut client, _: *mut session);
    fn server_client_remove_pane(_: *mut window_pane);
    fn recalculate_sizes();
    static grid_default_cell: grid_cell;
    fn screen_write_start_pane(_: *mut screen_write_ctx, _: *mut window_pane, _: *mut screen);
    fn screen_write_stop(_: *mut screen_write_ctx);
    fn screen_write_cursormove(
        _: *mut screen_write_ctx,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn screen_write_scrollregion(_: *mut screen_write_ctx, _: u_int, _: u_int);
    fn screen_write_linefeed(_: *mut screen_write_ctx, _: ::core::ffi::c_int, _: u_int);
    fn winlink_find_by_index(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn winlink_find_by_window(_: *mut winlinks, _: *mut window) -> *mut winlink;
    fn winlink_remove(_: *mut winlinks, _: *mut winlink);
    fn winlink_stack_remove(_: *mut winlink_stack, _: *mut winlink);
    fn window_unzoom(_: *mut window, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn window_push_zoom(
        _: *mut window,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn window_pop_zoom(_: *mut window) -> ::core::ffi::c_int;
    fn window_remove_pane(_: *mut window, _: *mut window_pane);
    fn window_count_panes(_: *mut window, _: ::core::ffi::c_int) -> u_int;
    fn window_add_ref(_: *mut window, _: *const ::core::ffi::c_char);
    fn window_remove_ref(_: *mut window, _: *const ::core::ffi::c_char);
    fn layout_close_pane(_: *mut window_pane);
    static mut sessions: sessions;
    fn sessions_RB_NEXT(_: *mut session) -> *mut session;
    fn sessions_RB_MINMAX(_: *mut sessions, _: ::core::ffi::c_int) -> *mut session;
    fn session_destroy(_: *mut session, _: ::core::ffi::c_int, _: *const ::core::ffi::c_char);
    fn session_next_session(_: *mut session, _: *mut sort_criteria) -> *mut session;
    fn session_previous_session(_: *mut session, _: *mut sort_criteria) -> *mut session;
    fn session_attach(
        _: *mut session,
        _: *mut window,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut winlink;
    fn session_detach(_: *mut session, _: *mut winlink) -> ::core::ffi::c_int;
    fn session_has(_: *mut session, _: *mut window) -> ::core::ffi::c_int;
    fn session_select(_: *mut session, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn session_group_contains(_: *mut session) -> *mut session_group;
    fn session_group_count(_: *mut session_group) -> u_int;
    fn session_renumber_windows(_: *mut session);
}
pub type __uint32_t = u32;
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
pub type uint32_t = __uint32_t;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct imsg_hdr {
    pub type_0: uint32_t,
    pub len: uint32_t,
    pub peerid: uint32_t,
    pub pid: uint32_t,
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
pub struct session_group {
    pub name: *const ::core::ffi::c_char,
    pub sessions: C2RustUnnamed_39,
    pub entry: C2RustUnnamed_38,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_38 {
    pub rbe_left: *mut session_group,
    pub rbe_right: *mut session_group,
    pub rbe_parent: *mut session_group,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_39 {
    pub tqh_first: *mut session,
    pub tqh_last: *mut *mut session,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sessions {
    pub rbh_root: *mut session,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct clients {
    pub tqh_first: *mut client,
    pub tqh_last: *mut *mut client,
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
pub const SIGCHLD: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const IMSG_HEADER_SIZE: usize = ::core::mem::size_of::<imsg_hdr>();
pub const MAX_IMSGSIZE: ::core::ffi::c_int = 16384 as ::core::ffi::c_int;
pub const MODE_CURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_REDRAW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_STATUSREADY: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const PANE_STATUSDRAWN: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const PANE_FLOATOVERZOOM: ::core::ffi::c_int = 0x200000 as ::core::ffi::c_int;
pub const WINLINK_BELL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINLINK_ACTIVITY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const WINLINK_SILENCE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const WINLINK_ALERTFLAGS: ::core::ffi::c_int =
    WINLINK_BELL | WINLINK_ACTIVITY | WINLINK_SILENCE;
pub const CLIENT_EXIT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CLIENT_REDRAWWINDOW: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSTATUS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CLIENT_SUSPENDED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLIENT_REDRAWBORDERS: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSTATUSALWAYS: ::core::ffi::c_int = 0x1000000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWOVERLAY: ::core::ffi::c_int = 0x2000000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWMENU: ::core::ffi::c_int = 0x20000000 as ::core::ffi::c_int;
pub const CLIENT_NO_DETACH_ON_DESTROY: ::core::ffi::c_ulonglong =
    0x8000000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_ALLREDRAWFLAGS: ::core::ffi::c_int = CLIENT_REDRAWWINDOW
    | CLIENT_REDRAWSTATUS
    | CLIENT_REDRAWSTATUSALWAYS
    | CLIENT_REDRAWBORDERS
    | CLIENT_REDRAWOVERLAY
    | CLIENT_REDRAWMENU;
unsafe extern "C" fn server_fire_pane_exit(
    mut name: *const ::core::ffi::c_char,
    mut wp: *mut window_pane,
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
    let mut status: ::core::ffi::c_int = (*wp).status;
    let mut signame: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if ((status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int) as ::core::ffi::c_schar
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int
        > 0 as ::core::ffi::c_int
    {
        signame = sig2name(status & 0x7f as ::core::ffi::c_int);
    }
    ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    if status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        event_payload_set_int(
            ep,
            b"exit_status\0" as *const u8 as *const ::core::ffi::c_char,
            (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int,
        );
    } else if ((status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
        as ::core::ffi::c_schar as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int
        > 0 as ::core::ffi::c_int
    {
        event_payload_set_string(
            ep,
            b"exit_signal\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            signame,
        );
    }
    event_payload_set_int(
        ep,
        b"exit_success\0" as *const u8 as *const ::core::ffi::c_char,
        (status == 0 as ::core::ffi::c_int) as ::core::ffi::c_int,
    );
    events_fire(name, ep);
}
#[no_mangle]
pub unsafe extern "C" fn server_redraw_client(mut c: *mut client) {
    (*c).flags |= CLIENT_ALLREDRAWFLAGS as uint64_t;
}
#[no_mangle]
pub unsafe extern "C" fn server_status_client(mut c: *mut client) {
    (*c).flags |= CLIENT_REDRAWSTATUS as uint64_t;
}
#[no_mangle]
pub unsafe extern "C" fn server_redraw_session(mut s: *mut session) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.tqh_first;
    while !c.is_null() {
        if (*c).session == s {
            server_redraw_client(c);
        }
        c = (*c).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_redraw_session_group(mut s: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains(s);
    if sg.is_null() {
        server_redraw_session(s);
    } else {
        s = (*sg).sessions.tqh_first;
        while !s.is_null() {
            server_redraw_session(s);
            s = (*s).gentry.tqe_next;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn server_status_session(mut s: *mut session) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.tqh_first;
    while !c.is_null() {
        if (*c).session == s {
            server_status_client(c);
        }
        c = (*c).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_status_session_group(mut s: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    sg = session_group_contains(s);
    if sg.is_null() {
        server_status_session(s);
    } else {
        s = (*sg).sessions.tqh_first;
        while !s.is_null() {
            server_status_session(s);
            s = (*s).gentry.tqe_next;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn server_redraw_window(mut w: *mut window) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.tqh_first;
    while !c.is_null() {
        if !(*c).session.is_null()
            && !(*(*c).session).curw.is_null()
            && (*(*(*c).session).curw).window == w
        {
            server_redraw_client(c);
        }
        c = (*c).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_redraw_window_menu(mut w: *mut window) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.tqh_first;
    while !c.is_null() {
        if !(*c).session.is_null()
            && !(*(*c).session).curw.is_null()
            && (*(*(*c).session).curw).window == w
        {
            (*c).flags |= CLIENT_REDRAWMENU as uint64_t;
        }
        c = (*c).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_redraw_window_borders(mut w: *mut window) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.tqh_first;
    while !c.is_null() {
        if !(*c).session.is_null()
            && !(*(*c).session).curw.is_null()
            && (*(*(*c).session).curw).window == w
        {
            (*c).flags |= CLIENT_REDRAWBORDERS as uint64_t;
        }
        c = (*c).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_status_window(mut w: *mut window) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        if session_has(s, w) != 0 {
            server_status_session(s);
        }
        s = sessions_RB_NEXT(s);
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_lock() {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.tqh_first;
    while !c.is_null() {
        if !(*c).session.is_null() {
            server_lock_client(c);
        }
        c = (*c).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_lock_session(mut s: *mut session) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.tqh_first;
    while !c.is_null() {
        if (*c).session == s {
            server_lock_client(c);
        }
        c = (*c).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_lock_client(mut c: *mut client) {
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        return;
    }
    if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
        return;
    }
    cmd = options_get_string(
        (*(*c).session).options,
        b"lock-command\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if *cmd as ::core::ffi::c_int == '\0' as i32
        || strlen(cmd).wrapping_add(1 as size_t)
            > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE)
    {
        return;
    }
    tty_stop_tty(&raw mut (*c).tty);
    tty_raw(
        &raw mut (*c).tty,
        tty_term_string((*c).tty.term, TTYC_SMCUP),
    );
    tty_raw(
        &raw mut (*c).tty,
        tty_term_string((*c).tty.term, TTYC_CLEAR),
    );
    tty_raw(&raw mut (*c).tty, tty_term_string((*c).tty.term, TTYC_E3));
    (*c).flags |= CLIENT_SUSPENDED as uint64_t;
    proc_send(
        (*c).peer,
        MSG_LOCK,
        -(1 as ::core::ffi::c_int),
        cmd as *const ::core::ffi::c_void,
        strlen(cmd).wrapping_add(1 as size_t),
    );
}
#[no_mangle]
pub unsafe extern "C" fn server_kill_pane(mut wp: *mut window_pane) {
    let mut w: *mut window = (*wp).window as *mut window;
    if window_count_panes(w, 1 as ::core::ffi::c_int) == 1 as u_int {
        server_kill_window(w, 1 as ::core::ffi::c_int);
        recalculate_sizes();
    } else {
        window_push_zoom(w, 0 as ::core::ffi::c_int, (*wp).flags & PANE_FLOATOVERZOOM);
        server_client_remove_pane(wp);
        layout_close_pane(wp);
        window_remove_pane(w, wp);
        window_pop_zoom(w);
        server_redraw_window(w);
    };
}
#[no_mangle]
pub unsafe extern "C" fn server_kill_window(mut w: *mut window, mut renumber: ::core::ffi::c_int) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut s1: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    window_add_ref(
        w,
        b"server_kill_window\0" as *const u8 as *const ::core::ffi::c_char,
    );
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() && {
        s1 = sessions_RB_NEXT(s);
        1 as ::core::ffi::c_int != 0
    } {
        if !(session_has(s, w) == 0) {
            server_unzoom_window(w);
            loop {
                wl = winlink_find_by_window(&raw mut (*s).windows, w);
                if wl.is_null() {
                    break;
                }
                if session_detach(s, wl) != 0 {
                    server_destroy_session_group(s);
                    break;
                } else {
                    server_redraw_session_group(s);
                }
            }
            if renumber != 0 {
                server_renumber_session(s);
            }
        }
        s = s1;
    }
    recalculate_sizes();
    window_remove_ref(
        w,
        b"server_kill_window\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn server_renumber_session(mut s: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    if options_get_number(
        (*s).options,
        b"renumber-windows\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        sg = session_group_contains(s);
        if !sg.is_null() {
            s = (*sg).sessions.tqh_first;
            while !s.is_null() {
                session_renumber_windows(s);
                s = (*s).gentry.tqe_next;
            }
        } else {
            session_renumber_windows(s);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_renumber_all() {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        server_renumber_session(s);
        s = sessions_RB_NEXT(s);
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_link_window(
    mut src: *mut session,
    mut srcwl: *mut winlink,
    mut dst: *mut session,
    mut dstidx: ::core::ffi::c_int,
    mut killflag: ::core::ffi::c_int,
    mut selectflag: ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut dstwl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut srcsg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut dstsg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    srcsg = session_group_contains(src);
    dstsg = session_group_contains(dst);
    if src != dst && !srcsg.is_null() && !dstsg.is_null() && srcsg == dstsg {
        xasprintf(
            cause,
            b"sessions are grouped\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return -(1 as ::core::ffi::c_int);
    }
    dstwl = ::core::ptr::null_mut::<winlink>();
    if dstidx != -(1 as ::core::ffi::c_int) {
        dstwl = winlink_find_by_index(&raw mut (*dst).windows, dstidx);
    }
    if !dstwl.is_null() {
        if (*dstwl).window == (*srcwl).window {
            xasprintf(
                cause,
                b"same index: %d\0" as *const u8 as *const ::core::ffi::c_char,
                dstidx,
            );
            return -(1 as ::core::ffi::c_int);
        }
        if killflag != 0 {
            events_fire_winlink(
                b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
                dstwl,
            );
            (*dstwl).flags &= !WINLINK_ALERTFLAGS;
            winlink_stack_remove(&raw mut (*dst).lastw, dstwl);
            winlink_remove(&raw mut (*dst).windows, dstwl);
            if dstwl == (*dst).curw {
                selectflag = 1 as ::core::ffi::c_int;
                (*dst).curw = ::core::ptr::null_mut::<winlink>();
            }
        }
    }
    if dstidx == -(1 as ::core::ffi::c_int) {
        dstidx = (-(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong
            - options_get_number(
                (*dst).options,
                b"base-index\0" as *const u8 as *const ::core::ffi::c_char,
            )) as ::core::ffi::c_int;
    }
    dstwl = session_attach(dst, (*srcwl).window, dstidx, cause);
    if dstwl.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if marked_pane.wl == srcwl {
        marked_pane.wl = dstwl;
    }
    if selectflag != 0 {
        session_select(dst, (*dstwl).idx);
    }
    server_redraw_session_group(dst);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn server_unlink_window(mut s: *mut session, mut wl: *mut winlink) {
    if session_detach(s, wl) != 0 {
        server_destroy_session_group(s);
    } else {
        server_redraw_session_group(s);
    };
}
#[no_mangle]
pub unsafe extern "C" fn server_destroy_pane(
    mut wp: *mut window_pane,
    mut notify: ::core::ffi::c_int,
) {
    let mut w: *mut window = (*wp).window as *mut window;
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
    let mut remain_on_exit: ::core::ffi::c_int = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut sx: u_int = (*(*wp).base.grid).sx;
    let mut sy: u_int = (*(*wp).base.grid).sy;
    if (*wp).fd != -(1 as ::core::ffi::c_int) {
        utempter_remove_record((*wp).fd);
        kill(getpid(), SIGCHLD);
        bufferevent_free((*wp).event);
        (*wp).event = ::core::ptr::null_mut::<bufferevent>();
        close((*wp).fd);
        (*wp).fd = -(1 as ::core::ffi::c_int);
    }
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
        bufferevent_free((*wp).pipe_event);
        (*wp).pipe_event = ::core::ptr::null_mut::<bufferevent>();
        close((*wp).pipe_fd);
        (*wp).pipe_fd = -(1 as ::core::ffi::c_int);
    }
    if !(*wp).flags & PANE_STATUSREADY != 0 {
        return;
    }
    remain_on_exit = options_get_number(
        (*wp).options,
        b"remain-on-exit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    let mut current_block_37: u64;
    match remain_on_exit {
        2 | 4 => {
            if (*wp).status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                && ((*wp).status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                current_block_37 = 3275366147856559585;
            } else {
                current_block_37 = 2300157484894416861;
            }
        }
        1 | 3 => {
            current_block_37 = 2300157484894416861;
        }
        0 | _ => {
            current_block_37 = 3275366147856559585;
        }
    }
    match current_block_37 {
        3275366147856559585 => {}
        _ => {
            if (*wp).flags & PANE_STATUSDRAWN != 0 {
                return;
            }
            (*wp).flags |= PANE_STATUSDRAWN;
            gettimeofday(&raw mut (*wp).dead_time, NULL);
            if notify != 0 {
                server_fire_pane_exit(
                    b"pane-died\0" as *const u8 as *const ::core::ffi::c_char,
                    wp,
                );
            }
            s = options_get_string(
                (*wp).options,
                b"remain-on-exit-format\0" as *const u8 as *const ::core::ffi::c_char,
            );
            if *s as ::core::ffi::c_int != '\0' as i32 {
                screen_write_start_pane(&raw mut ctx, wp, &raw mut (*wp).base);
                screen_write_scrollregion(&raw mut ctx, 0 as u_int, sy.wrapping_sub(1 as u_int));
                screen_write_cursormove(
                    &raw mut ctx,
                    0 as ::core::ffi::c_int,
                    sy.wrapping_sub(1 as u_int) as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                screen_write_linefeed(&raw mut ctx, 1 as ::core::ffi::c_int, 8 as u_int);
                memcpy(
                    &raw mut gc as *mut ::core::ffi::c_void,
                    &raw const grid_default_cell as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<grid_cell>() as size_t,
                );
                expanded = format_single(
                    ::core::ptr::null_mut::<cmdq_item>(),
                    s,
                    ::core::ptr::null_mut::<client>(),
                    ::core::ptr::null_mut::<session>(),
                    ::core::ptr::null_mut::<winlink>(),
                    wp,
                );
                format_draw(
                    &raw mut ctx,
                    &raw mut gc,
                    sx,
                    expanded,
                    ::core::ptr::null_mut::<style_ranges>(),
                    0 as ::core::ffi::c_int,
                );
                free(expanded as *mut ::core::ffi::c_void);
                screen_write_stop(&raw mut ctx);
            }
            (*wp).base.mode &= !MODE_CURSOR;
            (*wp).flags |= PANE_REDRAW;
            return;
        }
    }
    if notify != 0 {
        server_fire_pane_exit(
            b"pane-exited\0" as *const u8 as *const ::core::ffi::c_char,
            wp,
        );
    }
    window_push_zoom(w, 0 as ::core::ffi::c_int, (*wp).flags & PANE_FLOATOVERZOOM);
    server_client_remove_pane(wp);
    layout_close_pane(wp);
    window_remove_pane(w, wp);
    if (*w).panes.tqh_first.is_null() {
        server_kill_window(w, 1 as ::core::ffi::c_int);
    } else {
        window_pop_zoom(w);
        server_redraw_window(w);
    };
}
unsafe extern "C" fn server_destroy_session_group(mut s: *mut session) {
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut s1: *mut session = ::core::ptr::null_mut::<session>();
    sg = session_group_contains(s);
    if sg.is_null() {
        server_destroy_session(s);
        session_destroy(
            s,
            1 as ::core::ffi::c_int,
            b"server_destroy_session_group\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        s = (*sg).sessions.tqh_first;
        while !s.is_null() && {
            s1 = (*s).gentry.tqe_next;
            1 as ::core::ffi::c_int != 0
        } {
            server_destroy_session(s);
            session_destroy(
                s,
                1 as ::core::ffi::c_int,
                b"server_destroy_session_group\0" as *const u8 as *const ::core::ffi::c_char,
            );
            s = s1;
        }
    };
}
unsafe extern "C" fn server_find_session(
    mut s: *mut session,
    mut f: Option<unsafe extern "C" fn(*mut session, *mut session) -> ::core::ffi::c_int>,
) -> *mut session {
    let mut s_loop: *mut session = ::core::ptr::null_mut::<session>();
    let mut s_out: *mut session = ::core::ptr::null_mut::<session>();
    s_loop = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s_loop.is_null() {
        if s_loop != s && f.expect("non-null function pointer")(s_loop, s_out) != 0 {
            s_out = s_loop;
        }
        s_loop = sessions_RB_NEXT(s_loop);
    }
    return s_out;
}
unsafe extern "C" fn server_newer_session(
    mut s_loop: *mut session,
    mut s_out: *mut session,
) -> ::core::ffi::c_int {
    if s_out.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    return if (*s_loop).activity_time.tv_sec == (*s_out).activity_time.tv_sec {
        ((*s_loop).activity_time.tv_usec > (*s_out).activity_time.tv_usec) as ::core::ffi::c_int
    } else {
        ((*s_loop).activity_time.tv_sec > (*s_out).activity_time.tv_sec) as ::core::ffi::c_int
    };
}
unsafe extern "C" fn server_newer_detached_session(
    mut s_loop: *mut session,
    mut s_out: *mut session,
) -> ::core::ffi::c_int {
    if (*s_loop).attached != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return server_newer_session(s_loop, s_out);
}
#[no_mangle]
pub unsafe extern "C" fn server_destroy_session(mut s: *mut session) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut s_new: *mut session = ::core::ptr::null_mut::<session>();
    let mut cs_new: *mut session = ::core::ptr::null_mut::<session>();
    let mut use_s: *mut session = ::core::ptr::null_mut::<session>();
    let mut sort_crit: sort_criteria = sort_criteria {
        order: SORT_NAME,
        reversed: 0,
        order_seq: ::core::ptr::null_mut::<sort_order>(),
    };
    let mut detach_on_destroy: ::core::ffi::c_int = 0;
    detach_on_destroy = options_get_number(
        (*s).options,
        b"detach-on-destroy\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if detach_on_destroy == 0 as ::core::ffi::c_int {
        s_new = server_find_session(
            s,
            Some(
                server_newer_session
                    as unsafe extern "C" fn(*mut session, *mut session) -> ::core::ffi::c_int,
            ),
        );
    } else if detach_on_destroy == 2 as ::core::ffi::c_int {
        s_new = server_find_session(
            s,
            Some(
                server_newer_detached_session
                    as unsafe extern "C" fn(*mut session, *mut session) -> ::core::ffi::c_int,
            ),
        );
    } else if detach_on_destroy == 3 as ::core::ffi::c_int {
        s_new = session_previous_session(s, &raw mut sort_crit);
    } else if detach_on_destroy == 4 as ::core::ffi::c_int {
        s_new = session_next_session(s, &raw mut sort_crit);
    }
    if s_new == s {
        s_new = ::core::ptr::null_mut::<session>();
    }
    if s_new.is_null()
        && (detach_on_destroy == 1 as ::core::ffi::c_int
            || detach_on_destroy == 2 as ::core::ffi::c_int)
    {
        cs_new = server_find_session(
            s,
            Some(
                server_newer_session
                    as unsafe extern "C" fn(*mut session, *mut session) -> ::core::ffi::c_int,
            ),
        );
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !((*c).session != s) {
            use_s = s_new;
            if use_s.is_null()
                && (*c).flags as ::core::ffi::c_ulonglong & CLIENT_NO_DETACH_ON_DESTROY != 0
            {
                use_s = cs_new;
            }
            (*c).session = ::core::ptr::null_mut::<session>();
            (*c).last_session = ::core::ptr::null_mut::<session>();
            server_client_set_session(c, use_s);
            if use_s.is_null() {
                (*c).flags |= CLIENT_EXIT as uint64_t;
            }
        }
        c = (*c).entry.tqe_next;
    }
    recalculate_sizes();
}
#[no_mangle]
pub unsafe extern "C" fn server_check_unattached() {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut current_block_4: u64;
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        if !((*s).attached != 0 as u_int) {
            match options_get_number(
                (*s).options,
                b"destroy-unattached\0" as *const u8 as *const ::core::ffi::c_char,
            ) {
                0 => {}
                2 => {
                    current_block_4 = 6116987625208566775;
                    match current_block_4 {
                        11000743977270914936 => {
                            sg = session_group_contains(s);
                            if !sg.is_null() && session_group_count(sg) == 1 as u_int {
                                current_block_4 = 16668937799742929182;
                            } else {
                                current_block_4 = 13109137661213826276;
                            }
                        }
                        6116987625208566775 => {
                            sg = session_group_contains(s);
                            if sg.is_null() || session_group_count(sg) <= 1 as u_int {
                                current_block_4 = 16668937799742929182;
                            } else {
                                current_block_4 = 13109137661213826276;
                            }
                        }
                        _ => {}
                    }
                    match current_block_4 {
                        16668937799742929182 => {}
                        _ => {
                            server_destroy_session(s);
                            session_destroy(
                                s,
                                1 as ::core::ffi::c_int,
                                b"server_check_unattached\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    }
                }
                3 => {
                    current_block_4 = 11000743977270914936;
                    match current_block_4 {
                        11000743977270914936 => {
                            sg = session_group_contains(s);
                            if !sg.is_null() && session_group_count(sg) == 1 as u_int {
                                current_block_4 = 16668937799742929182;
                            } else {
                                current_block_4 = 13109137661213826276;
                            }
                        }
                        6116987625208566775 => {
                            sg = session_group_contains(s);
                            if sg.is_null() || session_group_count(sg) <= 1 as u_int {
                                current_block_4 = 16668937799742929182;
                            } else {
                                current_block_4 = 13109137661213826276;
                            }
                        }
                        _ => {}
                    }
                    match current_block_4 {
                        16668937799742929182 => {}
                        _ => {
                            server_destroy_session(s);
                            session_destroy(
                                s,
                                1 as ::core::ffi::c_int,
                                b"server_check_unattached\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    }
                }
                1 | _ => {
                    current_block_4 = 13109137661213826276;
                    match current_block_4 {
                        11000743977270914936 => {
                            sg = session_group_contains(s);
                            if !sg.is_null() && session_group_count(sg) == 1 as u_int {
                                current_block_4 = 16668937799742929182;
                            } else {
                                current_block_4 = 13109137661213826276;
                            }
                        }
                        6116987625208566775 => {
                            sg = session_group_contains(s);
                            if sg.is_null() || session_group_count(sg) <= 1 as u_int {
                                current_block_4 = 16668937799742929182;
                            } else {
                                current_block_4 = 13109137661213826276;
                            }
                        }
                        _ => {}
                    }
                    match current_block_4 {
                        16668937799742929182 => {}
                        _ => {
                            server_destroy_session(s);
                            session_destroy(
                                s,
                                1 as ::core::ffi::c_int,
                                b"server_check_unattached\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    }
                }
            }
        }
        s = sessions_RB_NEXT(s);
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_unzoom_window(mut w: *mut window) {
    if window_unzoom(w, 1 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int {
        server_redraw_window(w);
    }
}

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
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn tty_term_has(_: *mut tty_term, _: tty_code_code) -> ::core::ffi::c_int;
    fn tty_term_number(_: *mut tty_term, _: tty_code_code) -> ::core::ffi::c_int;
}
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
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
pub struct tty_acs_entry {
    pub key: u_char,
    pub string: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_acs_reverse_entry {
    pub string: *const ::core::ffi::c_char,
    pub key: u_char,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[inline]
unsafe extern "C" fn bsearch(
    mut __key: *const ::core::ffi::c_void,
    mut __base: *const ::core::ffi::c_void,
    mut __nmemb: size_t,
    mut __size: size_t,
    mut __compar: __compar_fn_t,
) -> *mut ::core::ffi::c_void {
    let mut __p: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
    let mut __comparison: ::core::ffi::c_int = 0;
    while __nmemb != 0 {
        __p = (__base as *const ::core::ffi::c_char)
            .offset((__nmemb >> 1 as ::core::ffi::c_int).wrapping_mul(__size) as isize)
            as *const ::core::ffi::c_void;
        __comparison = Some(__compar.expect("non-null function pointer"))
            .expect("non-null function pointer")(__key, __p);
        if __comparison == 0 as ::core::ffi::c_int {
            return __p as *mut ::core::ffi::c_void;
        }
        if __comparison > 0 as ::core::ffi::c_int {
            __base = (__p as *const ::core::ffi::c_char).offset(__size as isize)
                as *const ::core::ffi::c_void;
            __nmemb = __nmemb.wrapping_sub(1);
        }
        __nmemb >>= 1 as ::core::ffi::c_int;
    }
    return NULL;
}
pub const CLIENT_UTF8: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
static mut tty_acs_table: [tty_acs_entry; 36] = [
    tty_acs_entry {
        key: '+' as i32 as u_char,
        string: b"\xE2\x86\x92\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: ',' as i32 as u_char,
        string: b"\xE2\x86\x90\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: '-' as i32 as u_char,
        string: b"\xE2\x86\x91\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: '.' as i32 as u_char,
        string: b"\xE2\x86\x93\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: '0' as i32 as u_char,
        string: b"\xE2\x96\xAE\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: '`' as i32 as u_char,
        string: b"\xE2\x97\x86\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'a' as i32 as u_char,
        string: b"\xE2\x96\x92\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'b' as i32 as u_char,
        string: b"\xE2\x90\x89\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'c' as i32 as u_char,
        string: b"\xE2\x90\x8C\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'd' as i32 as u_char,
        string: b"\xE2\x90\x8D\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'e' as i32 as u_char,
        string: b"\xE2\x90\x8A\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'f' as i32 as u_char,
        string: b"\xC2\xB0\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'g' as i32 as u_char,
        string: b"\xC2\xB1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'h' as i32 as u_char,
        string: b"\xE2\x90\xA4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'i' as i32 as u_char,
        string: b"\xE2\x90\x8B\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'j' as i32 as u_char,
        string: b"\xE2\x94\x98\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'k' as i32 as u_char,
        string: b"\xE2\x94\x90\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'l' as i32 as u_char,
        string: b"\xE2\x94\x8C\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'm' as i32 as u_char,
        string: b"\xE2\x94\x94\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'n' as i32 as u_char,
        string: b"\xE2\x94\xBC\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'o' as i32 as u_char,
        string: b"\xE2\x8E\xBA\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'p' as i32 as u_char,
        string: b"\xE2\x8E\xBB\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'q' as i32 as u_char,
        string: b"\xE2\x94\x80\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'r' as i32 as u_char,
        string: b"\xE2\x8E\xBC\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 's' as i32 as u_char,
        string: b"\xE2\x8E\xBD\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 't' as i32 as u_char,
        string: b"\xE2\x94\x9C\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'u' as i32 as u_char,
        string: b"\xE2\x94\xA4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'v' as i32 as u_char,
        string: b"\xE2\x94\xB4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'w' as i32 as u_char,
        string: b"\xE2\x94\xAC\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'x' as i32 as u_char,
        string: b"\xE2\x94\x82\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'y' as i32 as u_char,
        string: b"\xE2\x89\xA4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: 'z' as i32 as u_char,
        string: b"\xE2\x89\xA5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: '{' as i32 as u_char,
        string: b"\xCF\x80\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: '|' as i32 as u_char,
        string: b"\xE2\x89\xA0\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: '}' as i32 as u_char,
        string: b"\xC2\xA3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_acs_entry {
        key: '~' as i32 as u_char,
        string: b"\xC2\xB7\0" as *const u8 as *const ::core::ffi::c_char,
    },
];
static mut tty_acs_reverse2: [tty_acs_reverse_entry; 1] = [tty_acs_reverse_entry {
    string: b"\xC2\xB7\0" as *const u8 as *const ::core::ffi::c_char,
    key: '~' as i32 as u_char,
}];
static mut tty_acs_reverse3: [tty_acs_reverse_entry; 32] = [
    tty_acs_reverse_entry {
        string: b"\xE2\x94\x80\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'q' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\x81\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'q' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\x82\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'x' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\x83\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'x' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\x8C\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'l' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\x8F\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'k' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\x90\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'k' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\x93\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'l' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\x94\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'm' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\x97\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'm' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\x98\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'j' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\x9B\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'j' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\x9C\0" as *const u8 as *const ::core::ffi::c_char,
        key: 't' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\xA3\0" as *const u8 as *const ::core::ffi::c_char,
        key: 't' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\xA4\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'u' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\xAB\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'u' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\xB3\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'w' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\xB4\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'v' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\xBB\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'v' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x94\xBC\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'n' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x95\x8B\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'n' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x95\x90\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'q' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x95\x91\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'x' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x95\x94\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'l' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x95\x97\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'k' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x95\x9A\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'm' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x95\x9D\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'j' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x95\xA0\0" as *const u8 as *const ::core::ffi::c_char,
        key: 't' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x95\xA3\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'u' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x95\xA6\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'w' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x95\xA9\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'v' as i32 as u_char,
    },
    tty_acs_reverse_entry {
        string: b"\xE2\x95\xAC\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'n' as i32 as u_char,
    },
];
static mut tty_acs_double_borders_list: [utf8_data; 13] = unsafe {
    [
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 0 as u_char,
            width: 0 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\x91\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\x90\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\x94\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\x97\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\x9A\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\x9D\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xA6\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xA9\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xA3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xAC\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xC2\xB7\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 2 as u_char,
            width: 1 as u_char,
        },
    ]
};
static mut tty_acs_heavy_borders_list: [utf8_data; 13] = unsafe {
    [
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 0 as u_char,
            width: 0 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x83\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x81\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x8F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x93\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x97\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x9B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\xB3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\xBB\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\xA3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\xAB\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\x8B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xC2\xB7\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 2 as u_char,
            width: 1 as u_char,
        },
    ]
};
static mut tty_acs_rounded_borders_list: [utf8_data; 13] = unsafe {
    [
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 0 as u_char,
            width: 0 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x82\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x80\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xAD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xAE\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xB0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\xAF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\xB3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\xBB\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\x9C\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x94\xA4\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xE2\x95\x8B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        },
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xC2\xB7\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 0 as u_char,
            size: 2 as u_char,
            width: 1 as u_char,
        },
    ]
};
#[no_mangle]
pub unsafe extern "C" fn tty_acs_double_borders(
    mut cell_type: ::core::ffi::c_int,
) -> *const utf8_data {
    return (&raw const tty_acs_double_borders_list as *const utf8_data).offset(cell_type as isize)
        as *const utf8_data;
}
#[no_mangle]
pub unsafe extern "C" fn tty_acs_heavy_borders(
    mut cell_type: ::core::ffi::c_int,
) -> *const utf8_data {
    return (&raw const tty_acs_heavy_borders_list as *const utf8_data).offset(cell_type as isize)
        as *const utf8_data;
}
#[no_mangle]
pub unsafe extern "C" fn tty_acs_rounded_borders(
    mut cell_type: ::core::ffi::c_int,
) -> *const utf8_data {
    return (&raw const tty_acs_rounded_borders_list as *const utf8_data).offset(cell_type as isize)
        as *const utf8_data;
}
unsafe extern "C" fn tty_acs_cmp(
    mut key: *const ::core::ffi::c_void,
    mut value: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut entry: *const tty_acs_entry = value as *const tty_acs_entry;
    let mut test: ::core::ffi::c_int = *(key as *mut u_char) as ::core::ffi::c_int;
    return test - (*entry).key as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_acs_reverse_cmp(
    mut key: *const ::core::ffi::c_void,
    mut value: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut entry: *const tty_acs_reverse_entry = value as *const tty_acs_reverse_entry;
    let mut test: *const ::core::ffi::c_char = key as *const ::core::ffi::c_char;
    return strcmp(test, (*entry).string);
}
#[no_mangle]
pub unsafe extern "C" fn tty_acs_needed(mut tty: *mut tty) -> ::core::ffi::c_int {
    if tty.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if tty_term_has((*tty).term, TTYC_U8) != 0
        && tty_term_number((*tty).term, TTYC_U8) == 0 as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    if (*(*tty).client).flags & CLIENT_UTF8 as uint64_t != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tty_acs_get(
    mut tty: *mut tty,
    mut ch: u_char,
) -> *const ::core::ffi::c_char {
    let mut entry: *const tty_acs_entry = ::core::ptr::null::<tty_acs_entry>();
    if tty_acs_needed(tty) != 0 {
        if (*(*tty).term).acs[ch as usize][0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            == '\0' as i32
        {
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        return (&raw mut *(&raw mut (*(*tty).term).acs as *mut [::core::ffi::c_char; 2])
            .offset(ch as isize) as *mut ::core::ffi::c_char)
            .offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
    }
    entry = bsearch(
        &raw mut ch as *const ::core::ffi::c_void,
        &raw const tty_acs_table as *const tty_acs_entry as *const ::core::ffi::c_void,
        (::core::mem::size_of::<[tty_acs_entry; 36]>() as size_t)
            .wrapping_div(::core::mem::size_of::<tty_acs_entry>() as size_t),
        ::core::mem::size_of::<tty_acs_entry>() as size_t,
        Some(
            tty_acs_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    ) as *const tty_acs_entry;
    if entry.is_null() {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return (*entry).string;
}
#[no_mangle]
pub unsafe extern "C" fn tty_acs_reverse_get(
    mut tty: *mut tty,
    mut s: *const ::core::ffi::c_char,
    mut slen: size_t,
) -> ::core::ffi::c_int {
    let mut table: *const tty_acs_reverse_entry = ::core::ptr::null::<tty_acs_reverse_entry>();
    let mut entry: *const tty_acs_reverse_entry = ::core::ptr::null::<tty_acs_reverse_entry>();
    let mut items: u_int = 0;
    if slen == 2 as size_t {
        table = &raw const tty_acs_reverse2 as *const tty_acs_reverse_entry;
        items = (::core::mem::size_of::<[tty_acs_reverse_entry; 1]>() as usize)
            .wrapping_div(::core::mem::size_of::<tty_acs_reverse_entry>() as usize)
            as u_int;
    } else if slen == 3 as size_t {
        table = &raw const tty_acs_reverse3 as *const tty_acs_reverse_entry;
        items = (::core::mem::size_of::<[tty_acs_reverse_entry; 32]>() as usize)
            .wrapping_div(::core::mem::size_of::<tty_acs_reverse_entry>() as usize)
            as u_int;
    } else {
        return -(1 as ::core::ffi::c_int);
    }
    entry = bsearch(
        s as *const ::core::ffi::c_void,
        table as *const ::core::ffi::c_void,
        items as size_t,
        ::core::mem::size_of::<tty_acs_reverse_entry>() as size_t,
        Some(
            tty_acs_reverse_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    ) as *const tty_acs_reverse_entry;
    if entry.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    return (*entry).key as ::core::ffi::c_int;
}

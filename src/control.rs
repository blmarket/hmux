pub use crate::src::shared::errno::{EAGAIN, EINTR};
pub use crate::src::shared::limits::SIZE_MAX;
pub use crate::src::shared::event::{
    EVBUFFER_EOL_ANY, EVBUFFER_EOL_CRLF, EVBUFFER_EOL_CRLF_STRICT, EVBUFFER_EOL_LF,
    EVBUFFER_EOL_NUL, EV_READ, EV_WRITE, evbuffer_eol_style,
};
pub use crate::src::shared::monitor::{
    MONITOR_ALL_PANES, MONITOR_ALL_WINDOWS, MONITOR_NOTIFY_INITIAL, MONITOR_PANE,
    MONITOR_SESSION, MONITOR_WINDOW, monitor_type,
};
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::abi::{ssize_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
pub use crate::src::shared::command::{CMDQ_STATE_CONTROL};
pub use crate::src::shared::client::{
    CLIENT_CONTROLCONTROL, CLIENT_CONTROL_DISCARD, CLIENT_CONTROL_NOOUTPUT,
    CLIENT_CONTROL_PAUSEAFTER, CLIENT_DEAD, CLIENT_EXIT, CLIENT_SUSPENDED,
    CLIENT_UNATTACHEDFLAGS,
};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::command::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;
extern "C" {
    pub type args;
    pub type tmuxpeer;
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
    pub type monitor_set;
    pub type cmdq_list;
    pub type cmdq_state;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn evbuffer_new() -> *mut evbuffer;
    fn evbuffer_free(buf: *mut evbuffer);
    fn evbuffer_get_length(buf: *const evbuffer) -> size_t;
    fn evbuffer_add(
        buf: *mut evbuffer,
        data: *const ::core::ffi::c_void,
        datlen: size_t,
    ) -> ::core::ffi::c_int;
    fn evbuffer_readln(
        buffer: *mut evbuffer,
        n_read_out: *mut size_t,
        eol_style: evbuffer_eol_style,
    ) -> *mut ::core::ffi::c_char;
    fn evbuffer_add_printf(
        buf: *mut evbuffer,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn evbuffer_read(
        buffer: *mut evbuffer,
        fd: ::core::ffi::c_int,
        howmuch: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn evbuffer_pullup(buf: *mut evbuffer, size: ssize_t) -> *mut ::core::ffi::c_uchar;
    fn bufferevent_free(bufev: *mut bufferevent);
    fn bufferevent_write(
        bufev: *mut bufferevent,
        data: *const ::core::ffi::c_void,
        size: size_t,
    ) -> ::core::ffi::c_int;
    fn bufferevent_write_buffer(bufev: *mut bufferevent, buf: *mut evbuffer) -> ::core::ffi::c_int;
    fn bufferevent_enable(
        bufev: *mut bufferevent,
        event: ::core::ffi::c_short,
    ) -> ::core::ffi::c_int;
    fn bufferevent_disable(
        bufev: *mut bufferevent,
        event: ::core::ffi::c_short,
    ) -> ::core::ffi::c_int;
    fn bufferevent_setwatermark(
        bufev: *mut bufferevent,
        events: ::core::ffi::c_short,
        lowmark: size_t,
        highmark: size_t,
    );
    fn bufferevent_new(
        fd: ::core::ffi::c_int,
        readcb: bufferevent_data_cb,
        writecb: bufferevent_data_cb,
        errorcb: bufferevent_event_cb,
        cbarg: *mut ::core::ffi::c_void,
    ) -> *mut bufferevent;
    fn poll(
        __fds: *mut pollfd,
        __nfds: nfds_t,
        __timeout: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
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
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
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
    fn setblocking(_: ::core::ffi::c_int, _: ::core::ffi::c_int);
    fn get_timer() -> uint64_t;
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
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_callback1(
        _: *const ::core::ffi::c_char,
        _: cmdq_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut cmdq_item;
    fn cmdq_append(_: *mut client, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_guard(_: *mut cmdq_item, _: *const ::core::ffi::c_char, _: ::core::ffi::c_int);
    fn winlink_find_by_window(_: *mut winlinks, _: *mut window) -> *mut winlink;
    fn window_pane_find_by_id(_: u_int) -> *mut window_pane;
    fn window_pane_get_new_data(
        _: *mut window_pane,
        _: *mut window_pane_offset,
        _: *mut size_t,
    ) -> *mut ::core::ffi::c_void;
    fn window_pane_update_used_data(_: *mut window_pane, _: *mut window_pane_offset, _: size_t);
    fn monitor_create_client(
        _: *mut client,
        _: monitor_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut monitor_set;
    fn monitor_destroy(_: *mut monitor_set);
    fn monitor_add(
        _: *mut monitor_set,
        _: *const ::core::ffi::c_char,
        _: monitor_type,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    );
    fn monitor_remove(_: *mut monitor_set, _: *const ::core::ffi::c_char);
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
}
pub type nfds_t = ::core::ffi::c_ulong;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct pollfd {
    pub fd: ::core::ffi::c_int,
    pub events: ::core::ffi::c_short,
    pub revents: ::core::ffi::c_short,
}
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
    pub exit_type: client_exit_type,
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
pub struct control_state {
    pub panes: control_panes,
    pub windows: control_windows,
    pub pending_list: C2RustUnnamed_40,
    pub pending_count: u_int,
    pub all_blocks: C2RustUnnamed_37,
    pub queued_reply_bytes: size_t,
    pub read_event: *mut bufferevent,
    pub write_event: *mut bufferevent,
    pub subs: *mut monitor_set,
    pub guard_depth: ::core::ffi::c_int,
    pub deferred: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub tqh_first: *mut control_line,
    pub tqh_last: *mut *mut control_line,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_line {
    pub line: *mut ::core::ffi::c_char,
    pub entry: C2RustUnnamed_36,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_36 {
    pub tqe_next: *mut control_line,
    pub tqe_prev: *mut *mut control_line,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
    pub tqh_first: *mut control_block,
    pub tqh_last: *mut *mut control_block,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_block {
    pub size: size_t,
    pub line: *mut ::core::ffi::c_char,
    pub t: uint64_t,
    pub entry: C2RustUnnamed_39,
    pub all_entry: C2RustUnnamed_38,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_38 {
    pub tqe_next: *mut control_block,
    pub tqe_prev: *mut *mut control_block,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_39 {
    pub tqe_next: *mut control_block,
    pub tqe_prev: *mut *mut control_block,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_40 {
    pub tqh_first: *mut control_pane,
    pub tqh_last: *mut *mut control_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_pane {
    pub pane: u_int,
    pub offset: window_pane_offset,
    pub queued: window_pane_offset,
    pub flags: ::core::ffi::c_int,
    pub pending_flag: ::core::ffi::c_int,
    pub pending_entry: C2RustUnnamed_43,
    pub blocks: C2RustUnnamed_42,
    pub entry: C2RustUnnamed_41,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_41 {
    pub rbe_left: *mut control_pane,
    pub rbe_right: *mut control_pane,
    pub rbe_parent: *mut control_pane,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_42 {
    pub tqh_first: *mut control_block,
    pub tqh_last: *mut *mut control_block,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_43 {
    pub tqe_next: *mut control_pane,
    pub tqe_prev: *mut *mut control_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_windows {
    pub rbh_root: *mut control_window,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_window {
    pub window: u_int,
    pub sx: u_int,
    pub sy: u_int,
    pub entry: C2RustUnnamed_44,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_44 {
    pub rbe_left: *mut control_window,
    pub rbe_right: *mut control_window,
    pub rbe_parent: *mut control_window,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct control_panes {
    pub rbh_root: *mut control_pane,
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
pub struct monitor_change {
    pub name: *const ::core::ffi::c_char,
    pub value: *const ::core::ffi::c_char,
    pub last: *const ::core::ffi::c_char,
    pub c: *mut client,
    pub s: *mut session,
    pub wl: *mut winlink,
    pub wp: *mut window_pane,
}
pub type monitor_cb =
    Option<unsafe extern "C" fn(*mut monitor_change, *mut ::core::ffi::c_void) -> ()>;

pub const POLLIN: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const INFTIM: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const CONTROL_PANE_OFF: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CONTROL_PANE_PAUSED: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CONTROL_BUFFER_LOW: ::core::ffi::c_int = 512 as ::core::ffi::c_int;
pub const CONTROL_BUFFER_HIGH: ::core::ffi::c_int = 8192 as ::core::ffi::c_int;
pub const CONTROL_WRITE_MINIMUM: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const CONTROL_MAXIMUM_AGE: ::core::ffi::c_int = 300000 as ::core::ffi::c_int;
pub const CONTROL_MAXIMUM_REPLY_BUFFER: ::core::ffi::c_int =
    64 as ::core::ffi::c_int * 1024 as ::core::ffi::c_int * 1024 as ::core::ffi::c_int;
pub const CONTROL_IGNORE_FLAGS: ::core::ffi::c_int =
    CLIENT_CONTROL_NOOUTPUT | CLIENT_UNATTACHEDFLAGS;
unsafe extern "C" fn control_pane_cmp(
    mut cp1: *mut control_pane,
    mut cp2: *mut control_pane,
) -> ::core::ffi::c_int {
    if (*cp1).pane < (*cp2).pane {
        return -(1 as ::core::ffi::c_int);
    }
    if (*cp1).pane > (*cp2).pane {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn control_panes_RB_REMOVE_COLOR(
    mut head: *mut control_panes,
    mut parent: *mut control_pane,
    mut elm: *mut control_pane,
) {
    let mut tmp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
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
                    let mut oleft: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
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
                    let mut oright: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
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
unsafe extern "C" fn control_panes_RB_FIND(
    mut head: *mut control_panes,
    mut elm: *mut control_pane,
) -> *mut control_pane {
    let mut tmp: *mut control_pane = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = control_pane_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<control_pane>();
}
unsafe extern "C" fn control_panes_RB_NEXT(mut elm: *mut control_pane) -> *mut control_pane {
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
unsafe extern "C" fn control_panes_RB_MINMAX(
    mut head: *mut control_panes,
    mut val: ::core::ffi::c_int,
) -> *mut control_pane {
    let mut tmp: *mut control_pane = (*head).rbh_root;
    let mut parent: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
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
unsafe extern "C" fn control_panes_RB_REMOVE(
    mut head: *mut control_panes,
    mut elm: *mut control_pane,
) -> *mut control_pane {
    let mut current_block: u64;
    let mut child: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    let mut parent: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    let mut old: *mut control_pane = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
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
        current_block = 7630474801164167706;
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
        control_panes_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn control_panes_RB_INSERT(
    mut head: *mut control_panes,
    mut elm: *mut control_pane,
) -> *mut control_pane {
    let mut tmp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    let mut parent: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = control_pane_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<control_pane>();
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
    control_panes_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<control_pane>();
}
unsafe extern "C" fn control_panes_RB_INSERT_COLOR(
    mut head: *mut control_panes,
    mut elm: *mut control_pane,
) {
    let mut parent: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    let mut gparent: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    let mut tmp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
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
unsafe extern "C" fn control_window_cmp(
    mut cw1: *mut control_window,
    mut cw2: *mut control_window,
) -> ::core::ffi::c_int {
    if (*cw1).window < (*cw2).window {
        return -(1 as ::core::ffi::c_int);
    }
    if (*cw1).window > (*cw2).window {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn control_windows_RB_NEXT(mut elm: *mut control_window) -> *mut control_window {
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
unsafe extern "C" fn control_windows_RB_REMOVE_COLOR(
    mut head: *mut control_windows,
    mut parent: *mut control_window,
    mut elm: *mut control_window,
) {
    let mut tmp: *mut control_window = ::core::ptr::null_mut::<control_window>();
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
                    let mut oleft: *mut control_window = ::core::ptr::null_mut::<control_window>();
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
                    let mut oright: *mut control_window = ::core::ptr::null_mut::<control_window>();
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
unsafe extern "C" fn control_windows_RB_MINMAX(
    mut head: *mut control_windows,
    mut val: ::core::ffi::c_int,
) -> *mut control_window {
    let mut tmp: *mut control_window = (*head).rbh_root;
    let mut parent: *mut control_window = ::core::ptr::null_mut::<control_window>();
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
unsafe extern "C" fn control_windows_RB_REMOVE(
    mut head: *mut control_windows,
    mut elm: *mut control_window,
) -> *mut control_window {
    let mut current_block: u64;
    let mut child: *mut control_window = ::core::ptr::null_mut::<control_window>();
    let mut parent: *mut control_window = ::core::ptr::null_mut::<control_window>();
    let mut old: *mut control_window = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut control_window = ::core::ptr::null_mut::<control_window>();
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
        current_block = 16105667263266440559;
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
        control_windows_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn control_windows_RB_INSERT(
    mut head: *mut control_windows,
    mut elm: *mut control_window,
) -> *mut control_window {
    let mut tmp: *mut control_window = ::core::ptr::null_mut::<control_window>();
    let mut parent: *mut control_window = ::core::ptr::null_mut::<control_window>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = control_window_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<control_window>();
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
    control_windows_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<control_window>();
}
unsafe extern "C" fn control_windows_RB_INSERT_COLOR(
    mut head: *mut control_windows,
    mut elm: *mut control_window,
) {
    let mut parent: *mut control_window = ::core::ptr::null_mut::<control_window>();
    let mut gparent: *mut control_window = ::core::ptr::null_mut::<control_window>();
    let mut tmp: *mut control_window = ::core::ptr::null_mut::<control_window>();
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
unsafe extern "C" fn control_windows_RB_FIND(
    mut head: *mut control_windows,
    mut elm: *mut control_window,
) -> *mut control_window {
    let mut tmp: *mut control_window = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = control_window_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<control_window>();
}
unsafe extern "C" fn control_free_block(mut cs: *mut control_state, mut cb: *mut control_block) {
    let mut size: size_t = 0;
    if (*cb).size == 0 as size_t && !(*cb).line.is_null() {
        size = strlen((*cb).line).wrapping_add(1 as size_t);
        if (*cs).queued_reply_bytes > size {
            (*cs).queued_reply_bytes = (*cs).queued_reply_bytes.wrapping_sub(size);
        } else {
            (*cs).queued_reply_bytes = 0 as size_t;
        }
    }
    free((*cb).line as *mut ::core::ffi::c_void);
    if !(*cb).all_entry.tqe_next.is_null() {
        (*(*cb).all_entry.tqe_next).all_entry.tqe_prev = (*cb).all_entry.tqe_prev;
    } else {
        (*cs).all_blocks.tqh_last = (*cb).all_entry.tqe_prev;
    }
    *(*cb).all_entry.tqe_prev = (*cb).all_entry.tqe_next;
    free(cb as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn control_get_pane(
    mut c: *mut client,
    mut wp: *mut window_pane,
) -> *mut control_pane {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cp: control_pane = control_pane {
        pane: (*wp).id,
        offset: window_pane_offset { used: 0 },
        queued: window_pane_offset { used: 0 },
        flags: 0,
        pending_flag: 0,
        pending_entry: C2RustUnnamed_43 {
            tqe_next: ::core::ptr::null_mut::<control_pane>(),
            tqe_prev: ::core::ptr::null_mut::<*mut control_pane>(),
        },
        blocks: C2RustUnnamed_42 {
            tqh_first: ::core::ptr::null_mut::<control_block>(),
            tqh_last: ::core::ptr::null_mut::<*mut control_block>(),
        },
        entry: C2RustUnnamed_41 {
            rbe_left: ::core::ptr::null_mut::<control_pane>(),
            rbe_right: ::core::ptr::null_mut::<control_pane>(),
            rbe_parent: ::core::ptr::null_mut::<control_pane>(),
            rbe_color: 0,
        },
    };
    return control_panes_RB_FIND(&raw mut (*cs).panes, &raw mut cp);
}
unsafe extern "C" fn control_add_pane(
    mut c: *mut client,
    mut wp: *mut window_pane,
) -> *mut control_pane {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    cp = control_get_pane(c, wp);
    if !cp.is_null() {
        return cp;
    }
    cp = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<control_pane>() as size_t,
    ) as *mut control_pane;
    (*cp).pane = (*wp).id;
    control_panes_RB_INSERT(&raw mut (*cs).panes, cp);
    memcpy(
        &raw mut (*cp).offset as *mut ::core::ffi::c_void,
        &raw mut (*wp).offset as *const ::core::ffi::c_void,
        ::core::mem::size_of::<window_pane_offset>() as size_t,
    );
    memcpy(
        &raw mut (*cp).queued as *mut ::core::ffi::c_void,
        &raw mut (*wp).offset as *const ::core::ffi::c_void,
        ::core::mem::size_of::<window_pane_offset>() as size_t,
    );
    (*cp).blocks.tqh_first = ::core::ptr::null_mut::<control_block>();
    (*cp).blocks.tqh_last = &raw mut (*cp).blocks.tqh_first;
    return cp;
}
unsafe extern "C" fn control_get_window(
    mut c: *mut client,
    mut window: u_int,
) -> *mut control_window {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cw: control_window = control_window {
        window: window,
        sx: 0,
        sy: 0,
        entry: C2RustUnnamed_44 {
            rbe_left: ::core::ptr::null_mut::<control_window>(),
            rbe_right: ::core::ptr::null_mut::<control_window>(),
            rbe_parent: ::core::ptr::null_mut::<control_window>(),
            rbe_color: 0,
        },
    };
    if cs.is_null() {
        return ::core::ptr::null_mut::<control_window>();
    }
    return control_windows_RB_FIND(&raw mut (*cs).windows, &raw mut cw);
}
#[no_mangle]
pub unsafe extern "C" fn control_set_window_size(
    mut c: *mut client,
    mut window: u_int,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cw: *mut control_window = ::core::ptr::null_mut::<control_window>();
    if cs.is_null() {
        return;
    }
    cw = control_get_window(c, window);
    if cw.is_null() {
        cw = xcalloc(
            1 as size_t,
            ::core::mem::size_of::<control_window>() as size_t,
        ) as *mut control_window;
        (*cw).window = window;
        control_windows_RB_INSERT(&raw mut (*cs).windows, cw);
    }
    (*cw).sx = sx;
    (*cw).sy = sy;
}
#[no_mangle]
pub unsafe extern "C" fn control_get_window_size(
    mut c: *mut client,
    mut window: u_int,
    mut sx: *mut u_int,
    mut sy: *mut u_int,
) -> ::core::ffi::c_int {
    let mut cw: *mut control_window = ::core::ptr::null_mut::<control_window>();
    cw = control_get_window(c, window);
    if cw.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    *sx = (*cw).sx;
    *sy = (*cw).sy;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn control_clear_window_size(mut c: *mut client, mut window: u_int) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cw: *mut control_window = ::core::ptr::null_mut::<control_window>();
    if cs.is_null() {
        return;
    }
    cw = control_get_window(c, window);
    if !cw.is_null() {
        control_windows_RB_REMOVE(&raw mut (*cs).windows, cw);
        free(cw as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn control_discard_pane(mut c: *mut client, mut cp: *mut control_pane) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut cb1: *mut control_block = ::core::ptr::null_mut::<control_block>();
    cb = (*cp).blocks.tqh_first;
    while !cb.is_null() && {
        cb1 = (*cb).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(*cb).entry.tqe_next.is_null() {
            (*(*cb).entry.tqe_next).entry.tqe_prev = (*cb).entry.tqe_prev;
        } else {
            (*cp).blocks.tqh_last = (*cb).entry.tqe_prev;
        }
        *(*cb).entry.tqe_prev = (*cb).entry.tqe_next;
        control_free_block(cs, cb);
        cb = cb1;
    }
}
unsafe extern "C" fn control_window_pane(mut c: *mut client, mut pane: u_int) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if (*c).session.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    wp = window_pane_find_by_id(pane);
    if wp.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    if winlink_find_by_window(
        &raw mut (*(*c).session).windows,
        (*wp).window as *mut window,
    )
    .is_null()
    {
        return ::core::ptr::null_mut::<window_pane>();
    }
    return wp;
}
#[no_mangle]
pub unsafe extern "C" fn control_reset_offsets(mut c: *mut client) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    let mut cp1: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    cp = control_panes_RB_MINMAX(&raw mut (*cs).panes, RB_NEGINF);
    while !cp.is_null() && {
        cp1 = control_panes_RB_NEXT(cp);
        1 as ::core::ffi::c_int != 0
    } {
        control_discard_pane(c, cp);
        control_panes_RB_REMOVE(&raw mut (*cs).panes, cp);
        free(cp as *mut ::core::ffi::c_void);
        cp = cp1;
    }
    (*cs).pending_list.tqh_first = ::core::ptr::null_mut::<control_pane>();
    (*cs).pending_list.tqh_last = &raw mut (*cs).pending_list.tqh_first;
    (*cs).pending_count = 0 as u_int;
}
#[no_mangle]
pub unsafe extern "C" fn control_pane_offset(
    mut c: *mut client,
    mut wp: *mut window_pane,
    mut off: *mut ::core::ffi::c_int,
) -> *mut window_pane_offset {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    if (*c).flags & CLIENT_CONTROL_NOOUTPUT as uint64_t != 0 {
        *off = 0 as ::core::ffi::c_int;
        return ::core::ptr::null_mut::<window_pane_offset>();
    }
    cp = control_get_pane(c, wp);
    if cp.is_null() || (*cp).flags & CONTROL_PANE_PAUSED != 0 {
        *off = 0 as ::core::ffi::c_int;
        return ::core::ptr::null_mut::<window_pane_offset>();
    }
    if (*cp).flags & CONTROL_PANE_OFF != 0 {
        *off = 1 as ::core::ffi::c_int;
        return ::core::ptr::null_mut::<window_pane_offset>();
    }
    *off = (evbuffer_get_length((*(*cs).write_event).output) >= CONTROL_BUFFER_LOW as size_t)
        as ::core::ffi::c_int;
    return &raw mut (*cp).offset;
}
#[no_mangle]
pub unsafe extern "C" fn control_set_pane_on(mut c: *mut client, mut wp: *mut window_pane) {
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    cp = control_get_pane(c, wp);
    if !cp.is_null() && (*cp).flags & CONTROL_PANE_OFF != 0 {
        (*cp).flags &= !CONTROL_PANE_OFF;
        memcpy(
            &raw mut (*cp).offset as *mut ::core::ffi::c_void,
            &raw mut (*wp).offset as *const ::core::ffi::c_void,
            ::core::mem::size_of::<window_pane_offset>() as size_t,
        );
        memcpy(
            &raw mut (*cp).queued as *mut ::core::ffi::c_void,
            &raw mut (*wp).offset as *const ::core::ffi::c_void,
            ::core::mem::size_of::<window_pane_offset>() as size_t,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn control_set_pane_off(mut c: *mut client, mut wp: *mut window_pane) {
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    cp = control_add_pane(c, wp);
    control_discard_pane(c, cp);
    memcpy(
        &raw mut (*cp).offset as *mut ::core::ffi::c_void,
        &raw mut (*wp).offset as *const ::core::ffi::c_void,
        ::core::mem::size_of::<window_pane_offset>() as size_t,
    );
    memcpy(
        &raw mut (*cp).queued as *mut ::core::ffi::c_void,
        &raw mut (*wp).offset as *const ::core::ffi::c_void,
        ::core::mem::size_of::<window_pane_offset>() as size_t,
    );
    (*cp).flags |= CONTROL_PANE_OFF;
}
#[no_mangle]
pub unsafe extern "C" fn control_continue_pane(mut c: *mut client, mut wp: *mut window_pane) {
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    cp = control_get_pane(c, wp);
    if !cp.is_null() && (*cp).flags & CONTROL_PANE_PAUSED != 0 {
        (*cp).flags &= !CONTROL_PANE_PAUSED;
        memcpy(
            &raw mut (*cp).offset as *mut ::core::ffi::c_void,
            &raw mut (*wp).offset as *const ::core::ffi::c_void,
            ::core::mem::size_of::<window_pane_offset>() as size_t,
        );
        memcpy(
            &raw mut (*cp).queued as *mut ::core::ffi::c_void,
            &raw mut (*wp).offset as *const ::core::ffi::c_void,
            ::core::mem::size_of::<window_pane_offset>() as size_t,
        );
        control_notify_write(
            c,
            b"%%continue %%%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn control_pause_pane(mut c: *mut client, mut wp: *mut window_pane) {
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    cp = control_add_pane(c, wp);
    if !(*cp).flags & CONTROL_PANE_PAUSED != 0 {
        (*cp).flags |= CONTROL_PANE_PAUSED;
        control_discard_pane(c, cp);
        control_notify_write(
            c,
            b"%%pause %%%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn control_reset_pane(mut c: *mut client, mut wp: *mut window_pane) {
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    if (*c).control_state.is_null() {
        return;
    }
    cp = control_get_pane(c, wp);
    if cp.is_null() {
        return;
    }
    control_discard_pane(c, cp);
    memcpy(
        &raw mut (*cp).offset as *mut ::core::ffi::c_void,
        &raw mut (*wp).offset as *const ::core::ffi::c_void,
        ::core::mem::size_of::<window_pane_offset>() as size_t,
    );
    memcpy(
        &raw mut (*cp).queued as *mut ::core::ffi::c_void,
        &raw mut (*wp).offset as *const ::core::ffi::c_void,
        ::core::mem::size_of::<window_pane_offset>() as size_t,
    );
}
unsafe extern "C" fn control_check_reply_buffer(
    mut c: *mut client,
    mut added: size_t,
) -> ::core::ffi::c_int {
    let mut cs: *mut control_state = (*c).control_state;
    let mut size: size_t = 0;
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_DISCARD != 0 {
        return 1 as ::core::ffi::c_int;
    }
    size = evbuffer_get_length((*(*cs).write_event).output);
    size = size.wrapping_add((*cs).queued_reply_bytes);
    size = size.wrapping_add(added);
    if size < CONTROL_MAXIMUM_REPLY_BUFFER as size_t {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: %s: %zu bytes of replies buffered\0" as *const u8 as *const ::core::ffi::c_char,
        b"control_check_reply_buffer\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        size,
    );
    if !(*c).flags & CLIENT_EXIT as uint64_t != 0 {
        (*c).exit_message = xstrdup(b"too far behind\0" as *const u8 as *const ::core::ffi::c_char);
        (*c).flags |= CLIENT_EXIT as uint64_t;
        control_discard(c);
    }
    (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong | CLIENT_CONTROL_DISCARD) as uint64_t;
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn control_write_line(mut c: *mut client, mut line: *mut ::core::ffi::c_char) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut size: size_t = strlen(line).wrapping_add(1 as size_t);
    if control_check_reply_buffer(c, size) != 0 {
        free(line as *mut ::core::ffi::c_void);
        return;
    }
    if (*cs).all_blocks.tqh_first.is_null() {
        log_debug(
            b"%s: %s: writing line: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"control_write_line\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            line,
        );
        bufferevent_write(
            (*cs).write_event,
            line as *const ::core::ffi::c_void,
            size.wrapping_sub(1 as size_t),
        );
        bufferevent_write(
            (*cs).write_event,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            1 as size_t,
        );
        bufferevent_enable((*cs).write_event, EV_WRITE as ::core::ffi::c_short);
        free(line as *mut ::core::ffi::c_void);
        return;
    }
    cb = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<control_block>() as size_t,
    ) as *mut control_block;
    (*cb).line = line;
    (*cb).all_entry.tqe_next = ::core::ptr::null_mut::<control_block>();
    (*cb).all_entry.tqe_prev = (*cs).all_blocks.tqh_last;
    *(*cs).all_blocks.tqh_last = cb;
    (*cs).all_blocks.tqh_last = &raw mut (*cb).all_entry.tqe_next;
    (*cs).queued_reply_bytes = (*cs).queued_reply_bytes.wrapping_add(size);
    (*cb).t = get_timer();
    log_debug(
        b"%s: %s: storing line: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"control_write_line\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        (*cb).line,
    );
    bufferevent_enable((*cs).write_event, EV_WRITE as ::core::ffi::c_short);
}
unsafe extern "C" fn control_flush_deferred(mut c: *mut client) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cl: *mut control_line = ::core::ptr::null_mut::<control_line>();
    let mut cl1: *mut control_line = ::core::ptr::null_mut::<control_line>();
    cl = (*cs).deferred.tqh_first;
    while !cl.is_null() && {
        cl1 = (*cl).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(*cl).entry.tqe_next.is_null() {
            (*(*cl).entry.tqe_next).entry.tqe_prev = (*cl).entry.tqe_prev;
        } else {
            (*cs).deferred.tqh_last = (*cl).entry.tqe_prev;
        }
        *(*cl).entry.tqe_prev = (*cl).entry.tqe_next;
        control_write_line(c, (*cl).line);
        free(cl as *mut ::core::ffi::c_void);
        cl = cl1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn control_write(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ap: ::core::ffi::VaList;
    if cs.is_null() {
        return;
    }
    ap = args.clone();
    xvasprintf(&raw mut line, fmt, ap);
    control_write_line(c, line);
}
#[no_mangle]
pub unsafe extern "C" fn control_write_guard(
    mut c: *mut client,
    mut guard: *const ::core::ffi::c_char,
    mut t: ::core::ffi::c_long,
    mut number: u_int,
    mut flags: ::core::ffi::c_int,
) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if cs.is_null() {
        return;
    }
    if strcmp(guard, b"begin\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        (*cs).guard_depth += 1;
    }
    xasprintf(
        &raw mut line,
        b"%%%s %ld %u %d\0" as *const u8 as *const ::core::ffi::c_char,
        guard,
        t,
        number,
        flags,
    );
    control_write_line(c, line);
    if strcmp(guard, b"begin\0" as *const u8 as *const ::core::ffi::c_char)
        != 0 as ::core::ffi::c_int
        && (*cs).guard_depth > 0 as ::core::ffi::c_int
        && {
            (*cs).guard_depth -= 1;
            (*cs).guard_depth == 0 as ::core::ffi::c_int
        }
    {
        control_flush_deferred(c);
    }
}
#[no_mangle]
pub unsafe extern "C" fn control_notify_write(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cl: *mut control_line = ::core::ptr::null_mut::<control_line>();
    let mut ap: ::core::ffi::VaList;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if cs.is_null() {
        return;
    }
    ap = args.clone();
    xvasprintf(&raw mut line, fmt, ap);
    if (*cs).guard_depth == 0 as ::core::ffi::c_int {
        control_write_line(c, line);
        return;
    }
    log_debug(
        b"%s: %s: deferring notification: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"control_notify_write\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        line,
    );
    cl = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<control_line>() as size_t,
    ) as *mut control_line;
    (*cl).line = line;
    (*cl).entry.tqe_next = ::core::ptr::null_mut::<control_line>();
    (*cl).entry.tqe_prev = (*cs).deferred.tqh_last;
    *(*cs).deferred.tqh_last = cl;
    (*cs).deferred.tqh_last = &raw mut (*cl).entry.tqe_next;
}
unsafe extern "C" fn control_check_age(
    mut c: *mut client,
    mut wp: *mut window_pane,
    mut cp: *mut control_pane,
) -> ::core::ffi::c_int {
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut t: uint64_t = 0;
    let mut age: uint64_t = 0;
    cb = (*cp).blocks.tqh_first;
    if cb.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    t = get_timer();
    if (*cb).t >= t {
        return 0 as ::core::ffi::c_int;
    }
    age = t.wrapping_sub((*cb).t);
    log_debug(
        b"%s: %s: %%%u is %llu behind\0" as *const u8 as *const ::core::ffi::c_char,
        b"control_check_age\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        (*wp).id,
        age as ::core::ffi::c_ulonglong,
    );
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_PAUSEAFTER != 0 {
        if age < (*c).pause_age as uint64_t {
            return 0 as ::core::ffi::c_int;
        }
        (*cp).flags |= CONTROL_PANE_PAUSED;
        control_discard_pane(c, cp);
        control_notify_write(
            c,
            b"%%pause %%%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
        );
    } else {
        if age < CONTROL_MAXIMUM_AGE as uint64_t {
            return 0 as ::core::ffi::c_int;
        }
        (*c).exit_message = xstrdup(b"too far behind\0" as *const u8 as *const ::core::ffi::c_char);
        (*c).flags |= CLIENT_EXIT as uint64_t;
        control_discard(c);
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn control_write_output(mut c: *mut client, mut wp: *mut window_pane) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut new_size: size_t = 0;
    if winlink_find_by_window(
        &raw mut (*(*c).session).windows,
        (*wp).window as *mut window,
    )
    .is_null()
    {
        return;
    }
    if (*c).flags & (CONTROL_IGNORE_FLAGS | CLIENT_EXIT) as uint64_t != 0 {
        cp = control_get_pane(c, wp);
        if cp.is_null() {
            return;
        }
    } else {
        cp = control_add_pane(c, wp);
        if !((*cp).flags & (CONTROL_PANE_OFF | CONTROL_PANE_PAUSED) != 0) {
            if control_check_age(c, wp, cp) != 0 {
                return;
            }
            window_pane_get_new_data(wp, &raw mut (*cp).queued, &raw mut new_size);
            if new_size == 0 as size_t {
                return;
            }
            window_pane_update_used_data(wp, &raw mut (*cp).queued, new_size);
            cb = xcalloc(
                1 as size_t,
                ::core::mem::size_of::<control_block>() as size_t,
            ) as *mut control_block;
            (*cb).size = new_size;
            (*cb).all_entry.tqe_next = ::core::ptr::null_mut::<control_block>();
            (*cb).all_entry.tqe_prev = (*cs).all_blocks.tqh_last;
            *(*cs).all_blocks.tqh_last = cb;
            (*cs).all_blocks.tqh_last = &raw mut (*cb).all_entry.tqe_next;
            (*cb).t = get_timer();
            (*cb).entry.tqe_next = ::core::ptr::null_mut::<control_block>();
            (*cb).entry.tqe_prev = (*cp).blocks.tqh_last;
            *(*cp).blocks.tqh_last = cb;
            (*cp).blocks.tqh_last = &raw mut (*cb).entry.tqe_next;
            log_debug(
                b"%s: %s: new output block of %zu for %%%u\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"control_write_output\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
                (*cb).size,
                (*wp).id,
            );
            if (*cp).pending_flag == 0 {
                log_debug(
                    b"%s: %s: %%%u now pending\0" as *const u8 as *const ::core::ffi::c_char,
                    b"control_write_output\0" as *const u8 as *const ::core::ffi::c_char,
                    (*c).name,
                    (*wp).id,
                );
                (*cp).pending_entry.tqe_next = ::core::ptr::null_mut::<control_pane>();
                (*cp).pending_entry.tqe_prev = (*cs).pending_list.tqh_last;
                *(*cs).pending_list.tqh_last = cp;
                (*cs).pending_list.tqh_last = &raw mut (*cp).pending_entry.tqe_next;
                (*cp).pending_flag = 1 as ::core::ffi::c_int;
                (*cs).pending_count = (*cs).pending_count.wrapping_add(1);
            }
            bufferevent_enable((*cs).write_event, EV_WRITE as ::core::ffi::c_short);
            return;
        }
    }
    log_debug(
        b"%s: %s: ignoring pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"control_write_output\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        (*wp).id,
    );
    window_pane_update_used_data(wp, &raw mut (*cp).offset, SIZE_MAX as size_t);
    window_pane_update_used_data(wp, &raw mut (*cp).queued, SIZE_MAX as size_t);
}
unsafe extern "C" fn control_error(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut c: *mut client = cmdq_get_client(item);
    let mut error: *mut ::core::ffi::c_char = data as *mut ::core::ffi::c_char;
    cmdq_guard(
        item,
        b"begin\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    control_write(
        c,
        b"parse error: %s\0" as *const u8 as *const ::core::ffi::c_char,
        error,
    );
    cmdq_guard(
        item,
        b"error\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    );
    free(error as *mut ::core::ffi::c_void);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn control_error_callback(
    mut bufev: *mut bufferevent,
    mut what: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = data as *mut client;
    (*c).flags |= CLIENT_EXIT as uint64_t;
}
unsafe extern "C" fn control_read_callback(
    mut bufev: *mut bufferevent,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = data as *mut client;
    let mut cs: *mut control_state = (*c).control_state;
    let mut buffer: *mut evbuffer = (*(*cs).read_event).input;
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    let mut status: cmd_parse_status = CMD_PARSE_ERROR;
    loop {
        line = evbuffer_readln(buffer, ::core::ptr::null_mut::<size_t>(), EVBUFFER_EOL_LF);
        if line.is_null() {
            break;
        }
        log_debug(
            b"%s: %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"control_read_callback\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            line,
        );
        if *line as ::core::ffi::c_int == '\0' as i32 {
            free(line as *mut ::core::ffi::c_void);
            (*c).flags |= CLIENT_EXIT as uint64_t;
            break;
        } else {
            state = cmdq_new_state(
                ::core::ptr::null_mut::<cmd_find_state>(),
                ::core::ptr::null_mut::<key_event>(),
                CMDQ_STATE_CONTROL,
            );
            status = cmd_parse_and_append(
                line,
                ::core::ptr::null_mut::<cmd_parse_input>(),
                c,
                state,
                &raw mut error,
            );
            if status as ::core::ffi::c_uint
                == CMD_PARSE_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                cmdq_append(
                    c,
                    cmdq_get_callback1(
                        b"control_error\0" as *const u8 as *const ::core::ffi::c_char,
                        Some(
                            control_error
                                as unsafe extern "C" fn(
                                    *mut cmdq_item,
                                    *mut ::core::ffi::c_void,
                                )
                                    -> cmd_retval,
                        ),
                        error as *mut ::core::ffi::c_void,
                    ),
                );
            }
            cmdq_free_state(state);
            free(line as *mut ::core::ffi::c_void);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn control_all_done(mut c: *mut client) -> ::core::ffi::c_int {
    let mut cs: *mut control_state = (*c).control_state;
    if !(*cs).all_blocks.tqh_first.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (evbuffer_get_length((*(*cs).write_event).output) == 0 as size_t) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn control_wait_exit(mut fd: ::core::ffi::c_int) {
    let mut pfd: pollfd = pollfd {
        fd: 0,
        events: 0,
        revents: 0,
    };
    let mut evb: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut n: ::core::ffi::c_int = 0;
    evb = evbuffer_new();
    if evb.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    loop {
        line = evbuffer_readln(evb, ::core::ptr::null_mut::<size_t>(), EVBUFFER_EOL_LF);
        if !line.is_null() {
            if *line as ::core::ffi::c_int == '\0' as i32 {
                free(line as *mut ::core::ffi::c_void);
                break;
            } else {
                free(line as *mut ::core::ffi::c_void);
            }
        } else {
            memset(
                &raw mut pfd as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<pollfd>() as size_t,
            );
            pfd.fd = fd;
            pfd.events = POLLIN as ::core::ffi::c_short;
            if poll(&raw mut pfd, 1 as nfds_t, INFTIM) == -(1 as ::core::ffi::c_int) {
                if !(*__errno_location() == EINTR) {
                    break;
                }
            } else {
                n = evbuffer_read(evb, fd, -(1 as ::core::ffi::c_int));
                if n == 0 as ::core::ffi::c_int {
                    break;
                }
                if n == -(1 as ::core::ffi::c_int)
                    && *__errno_location() != EAGAIN
                    && *__errno_location() != EINTR
                {
                    break;
                }
            }
        }
    }
    evbuffer_free(evb);
}
unsafe extern "C" fn control_flush_all_blocks(mut c: *mut client) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut cb1: *mut control_block = ::core::ptr::null_mut::<control_block>();
    cb = (*cs).all_blocks.tqh_first;
    while !cb.is_null() && {
        cb1 = (*cb).all_entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if (*cb).size != 0 as size_t {
            break;
        }
        log_debug(
            b"%s: %s: flushing line: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"control_flush_all_blocks\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            (*cb).line,
        );
        bufferevent_write(
            (*cs).write_event,
            (*cb).line as *const ::core::ffi::c_void,
            strlen((*cb).line),
        );
        bufferevent_write(
            (*cs).write_event,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            1 as size_t,
        );
        control_free_block(cs, cb);
        cb = cb1;
    }
}
unsafe extern "C" fn control_append_data(
    mut c: *mut client,
    mut cp: *mut control_pane,
    mut age: uint64_t,
    mut message: *mut evbuffer,
    mut wp: *mut window_pane,
    mut size: size_t,
) -> *mut evbuffer {
    let mut new_data: *mut u_char = ::core::ptr::null_mut::<u_char>();
    let mut new_size: size_t = 0;
    let mut start: size_t = 0;
    let mut i: u_int = 0;
    if message.is_null() {
        message = evbuffer_new();
        if message.is_null() {
            fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
        }
        if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_PAUSEAFTER != 0 {
            evbuffer_add_printf(
                message,
                b"%%extended-output %%%u %llu : \0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
                age as ::core::ffi::c_ulonglong,
            );
        } else {
            evbuffer_add_printf(
                message,
                b"%%output %%%u \0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            );
        }
    }
    new_data =
        window_pane_get_new_data(wp, &raw mut (*cp).offset, &raw mut new_size) as *mut u_char;
    if new_size < size {
        fatalx(
            b"not enough data: %zu < %zu\0" as *const u8 as *const ::core::ffi::c_char,
            new_size,
            size,
        );
    }
    i = 0 as u_int;
    while (i as size_t) < size {
        if (*new_data.offset(i as isize) as ::core::ffi::c_int) < ' ' as i32
            || *new_data.offset(i as isize) as ::core::ffi::c_int == '\\' as i32
        {
            evbuffer_add_printf(
                message,
                b"\\%03o\0" as *const u8 as *const ::core::ffi::c_char,
                *new_data.offset(i as isize) as ::core::ffi::c_int,
            );
        } else {
            start = i as size_t;
            while (i.wrapping_add(1 as u_int) as size_t) < size
                && *new_data.offset(i.wrapping_add(1 as u_int) as isize) as ::core::ffi::c_int
                    >= ' ' as i32
                && *new_data.offset(i.wrapping_add(1 as u_int) as isize) as ::core::ffi::c_int
                    != '\\' as i32
            {
                i = i.wrapping_add(1);
            }
            evbuffer_add(
                message,
                new_data.offset(start as isize) as *const ::core::ffi::c_void,
                (i as size_t).wrapping_sub(start).wrapping_add(1 as size_t),
            );
        }
        i = i.wrapping_add(1);
    }
    window_pane_update_used_data(wp, &raw mut (*cp).offset, size);
    return message;
}
unsafe extern "C" fn control_write_data(mut c: *mut client, mut message: *mut evbuffer) {
    let mut cs: *mut control_state = (*c).control_state;
    log_debug(
        b"%s: %s: %.*s\0" as *const u8 as *const ::core::ffi::c_char,
        b"control_write_data\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        evbuffer_get_length(message) as ::core::ffi::c_int,
        evbuffer_pullup(message, -(1 as ::core::ffi::c_int) as ssize_t),
    );
    evbuffer_add(
        message,
        b"\n\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        1 as size_t,
    );
    bufferevent_write_buffer((*cs).write_event, message);
    evbuffer_free(message);
}
unsafe extern "C" fn control_write_pending(
    mut c: *mut client,
    mut cp: *mut control_pane,
    mut limit: size_t,
) -> ::core::ffi::c_int {
    let mut cs: *mut control_state = (*c).control_state;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut message: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut used: size_t = 0 as size_t;
    let mut size: size_t = 0;
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut cb1: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut age: uint64_t = 0;
    let mut t: uint64_t = get_timer();
    wp = control_window_pane(c, (*cp).pane);
    if wp.is_null() || (*wp).fd == -(1 as ::core::ffi::c_int) {
        cb = (*cp).blocks.tqh_first;
        while !cb.is_null() && {
            cb1 = (*cb).entry.tqe_next;
            1 as ::core::ffi::c_int != 0
        } {
            if !(*cb).entry.tqe_next.is_null() {
                (*(*cb).entry.tqe_next).entry.tqe_prev = (*cb).entry.tqe_prev;
            } else {
                (*cp).blocks.tqh_last = (*cb).entry.tqe_prev;
            }
            *(*cb).entry.tqe_prev = (*cb).entry.tqe_next;
            control_free_block(cs, cb);
            cb = cb1;
        }
        control_flush_all_blocks(c);
        return 0 as ::core::ffi::c_int;
    }
    while used != limit && !(*cp).blocks.tqh_first.is_null() {
        if control_check_age(c, wp, cp) != 0 {
            if !message.is_null() {
                evbuffer_free(message);
            }
            message = ::core::ptr::null_mut::<evbuffer>();
            break;
        } else {
            cb = (*cp).blocks.tqh_first;
            if (*cb).t < t {
                age = t.wrapping_sub((*cb).t);
            } else {
                age = 0 as uint64_t;
            }
            log_debug(
                b"%s: %s: output block %zu (age %llu) for %%%u (used %zu/%zu)\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"control_write_pending\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
                (*cb).size,
                age as ::core::ffi::c_ulonglong,
                (*cp).pane,
                used,
                limit,
            );
            size = (*cb).size;
            if size > limit.wrapping_sub(used) {
                size = limit.wrapping_sub(used);
            }
            used = used.wrapping_add(size);
            message = control_append_data(c, cp, age, message, wp, size);
            (*cb).size = (*cb).size.wrapping_sub(size);
            if (*cb).size == 0 as size_t {
                if !(*cb).entry.tqe_next.is_null() {
                    (*(*cb).entry.tqe_next).entry.tqe_prev = (*cb).entry.tqe_prev;
                } else {
                    (*cp).blocks.tqh_last = (*cb).entry.tqe_prev;
                }
                *(*cb).entry.tqe_prev = (*cb).entry.tqe_next;
                control_free_block(cs, cb);
                cb = (*cs).all_blocks.tqh_first;
                if !cb.is_null() && (*cb).size == 0 as size_t {
                    if !wp.is_null() && !message.is_null() {
                        control_write_data(c, message);
                        message = ::core::ptr::null_mut::<evbuffer>();
                    }
                    control_flush_all_blocks(c);
                }
            }
        }
    }
    if !message.is_null() {
        control_write_data(c, message);
    }
    return !(*cp).blocks.tqh_first.is_null() as ::core::ffi::c_int;
}
unsafe extern "C" fn control_write_callback(
    mut bufev: *mut bufferevent,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = data as *mut client;
    let mut cs: *mut control_state = (*c).control_state;
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    let mut cp1: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    let mut evb: *mut evbuffer = (*(*cs).write_event).output;
    let mut space: size_t = 0;
    let mut limit: size_t = 0;
    control_flush_all_blocks(c);
    while evbuffer_get_length(evb) < CONTROL_BUFFER_HIGH as size_t {
        if (*cs).pending_count == 0 as u_int {
            break;
        }
        space = (CONTROL_BUFFER_HIGH as size_t).wrapping_sub(evbuffer_get_length(evb));
        log_debug(
            b"%s: %s: %zu bytes available, %u panes\0" as *const u8 as *const ::core::ffi::c_char,
            b"control_write_callback\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            space,
            (*cs).pending_count,
        );
        limit = space
            .wrapping_div((*cs).pending_count as size_t)
            .wrapping_div(3 as size_t);
        if limit < CONTROL_WRITE_MINIMUM as size_t {
            limit = CONTROL_WRITE_MINIMUM as size_t;
        }
        cp = (*cs).pending_list.tqh_first;
        while !cp.is_null() && {
            cp1 = (*cp).pending_entry.tqe_next;
            1 as ::core::ffi::c_int != 0
        } {
            if evbuffer_get_length(evb) >= CONTROL_BUFFER_HIGH as size_t {
                break;
            }
            if !(control_write_pending(c, cp, limit) != 0) {
                if !(*cp).pending_entry.tqe_next.is_null() {
                    (*(*cp).pending_entry.tqe_next).pending_entry.tqe_prev =
                        (*cp).pending_entry.tqe_prev;
                } else {
                    (*cs).pending_list.tqh_last = (*cp).pending_entry.tqe_prev;
                }
                *(*cp).pending_entry.tqe_prev = (*cp).pending_entry.tqe_next;
                (*cp).pending_flag = 0 as ::core::ffi::c_int;
                (*cs).pending_count = (*cs).pending_count.wrapping_sub(1);
            }
            cp = cp1;
        }
    }
    if evbuffer_get_length(evb) == 0 as size_t {
        bufferevent_disable((*cs).write_event, EV_WRITE as ::core::ffi::c_short);
    }
}
unsafe extern "C" fn control_sub_change(
    mut change: *mut monitor_change,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = (*change).c;
    let mut s: *mut session = (*change).s;
    let mut wl: *mut winlink = (*change).wl;
    let mut wp: *mut window_pane = (*change).wp;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    if !wp.is_null() {
        w = (*wp).window as *mut window;
        control_notify_write(
            c,
            b"%%subscription-changed %s $%u @%u %u %%%u : %s\0" as *const u8
                as *const ::core::ffi::c_char,
            (*change).name,
            (*s).id,
            (*w).id,
            (*wl).idx,
            (*wp).id,
            (*change).value,
        );
    } else if !wl.is_null() {
        w = (*wl).window;
        control_notify_write(
            c,
            b"%%subscription-changed %s $%u @%u %u - : %s\0" as *const u8
                as *const ::core::ffi::c_char,
            (*change).name,
            (*s).id,
            (*w).id,
            (*wl).idx,
            (*change).value,
        );
    } else {
        control_notify_write(
            c,
            b"%%subscription-changed %s $%u - - - : %s\0" as *const u8
                as *const ::core::ffi::c_char,
            (*change).name,
            (*s).id,
            (*change).value,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn control_start(mut c: *mut client) {
    let mut cs: *mut control_state = ::core::ptr::null_mut::<control_state>();
    if (*c).flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
        close((*c).out_fd);
        (*c).out_fd = -(1 as ::core::ffi::c_int);
    } else {
        setblocking((*c).out_fd, 0 as ::core::ffi::c_int);
    }
    setblocking((*c).fd, 0 as ::core::ffi::c_int);
    (*c).control_state = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<control_state>() as size_t,
    ) as *mut control_state;
    cs = (*c).control_state;
    (*cs).panes.rbh_root = ::core::ptr::null_mut::<control_pane>();
    (*cs).windows.rbh_root = ::core::ptr::null_mut::<control_window>();
    (*cs).pending_list.tqh_first = ::core::ptr::null_mut::<control_pane>();
    (*cs).pending_list.tqh_last = &raw mut (*cs).pending_list.tqh_first;
    (*cs).all_blocks.tqh_first = ::core::ptr::null_mut::<control_block>();
    (*cs).all_blocks.tqh_last = &raw mut (*cs).all_blocks.tqh_first;
    (*cs).deferred.tqh_first = ::core::ptr::null_mut::<control_line>();
    (*cs).deferred.tqh_last = &raw mut (*cs).deferred.tqh_first;
    (*cs).subs = monitor_create_client(
        c,
        Some(
            control_sub_change
                as unsafe extern "C" fn(*mut monitor_change, *mut ::core::ffi::c_void) -> (),
        ),
        NULL,
    ) as *mut monitor_set;
    (*cs).read_event = bufferevent_new(
        (*c).fd,
        Some(
            control_read_callback
                as unsafe extern "C" fn(*mut bufferevent, *mut ::core::ffi::c_void) -> (),
        ),
        Some(
            control_write_callback
                as unsafe extern "C" fn(*mut bufferevent, *mut ::core::ffi::c_void) -> (),
        ),
        Some(
            control_error_callback
                as unsafe extern "C" fn(
                    *mut bufferevent,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        c as *mut ::core::ffi::c_void,
    );
    if (*cs).read_event.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if (*c).flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
        (*cs).write_event = (*cs).read_event;
    } else {
        (*cs).write_event = bufferevent_new(
            (*c).out_fd,
            None,
            Some(
                control_write_callback
                    as unsafe extern "C" fn(*mut bufferevent, *mut ::core::ffi::c_void) -> (),
            ),
            Some(
                control_error_callback
                    as unsafe extern "C" fn(
                        *mut bufferevent,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            c as *mut ::core::ffi::c_void,
        );
        if (*cs).write_event.is_null() {
            fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
        }
    }
    bufferevent_setwatermark(
        (*cs).write_event,
        EV_WRITE as ::core::ffi::c_short,
        CONTROL_BUFFER_LOW as size_t,
        0 as size_t,
    );
    if (*c).flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
        bufferevent_write(
            (*cs).write_event,
            b"\x1BP1000p\0" as *const u8 as *const ::core::ffi::c_char
                as *const ::core::ffi::c_void,
            7 as size_t,
        );
        bufferevent_enable((*cs).write_event, EV_WRITE as ::core::ffi::c_short);
    }
}
#[no_mangle]
pub unsafe extern "C" fn control_ready(mut c: *mut client) {
    bufferevent_enable(
        (*(*c).control_state).read_event,
        EV_READ as ::core::ffi::c_short,
    );
}
#[no_mangle]
pub unsafe extern "C" fn control_discard(mut c: *mut client) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cp: *mut control_pane = ::core::ptr::null_mut::<control_pane>();
    cp = control_panes_RB_MINMAX(&raw mut (*cs).panes, RB_NEGINF);
    while !cp.is_null() {
        control_discard_pane(c, cp);
        cp = control_panes_RB_NEXT(cp);
    }
    bufferevent_disable((*cs).read_event, EV_READ as ::core::ffi::c_short);
}
#[no_mangle]
pub unsafe extern "C" fn control_discard_all(mut c: *mut client) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut cb1: *mut control_block = ::core::ptr::null_mut::<control_block>();
    control_discard(c);
    cb = (*cs).all_blocks.tqh_first;
    while !cb.is_null() && {
        cb1 = (*cb).all_entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        control_free_block(cs, cb);
        cb = cb1;
    }
    (*cs).queued_reply_bytes = 0 as size_t;
    bufferevent_disable((*cs).write_event, EV_WRITE as ::core::ffi::c_short);
}
#[no_mangle]
pub unsafe extern "C" fn control_stop(mut c: *mut client) {
    let mut cs: *mut control_state = (*c).control_state;
    let mut cb: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut cb1: *mut control_block = ::core::ptr::null_mut::<control_block>();
    let mut cw: *mut control_window = ::core::ptr::null_mut::<control_window>();
    let mut cw1: *mut control_window = ::core::ptr::null_mut::<control_window>();
    let mut cl: *mut control_line = ::core::ptr::null_mut::<control_line>();
    let mut cl1: *mut control_line = ::core::ptr::null_mut::<control_line>();
    if cs.is_null() {
        return;
    }
    monitor_destroy((*cs).subs);
    cl = (*cs).deferred.tqh_first;
    while !cl.is_null() && {
        cl1 = (*cl).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(*cl).entry.tqe_next.is_null() {
            (*(*cl).entry.tqe_next).entry.tqe_prev = (*cl).entry.tqe_prev;
        } else {
            (*cs).deferred.tqh_last = (*cl).entry.tqe_prev;
        }
        *(*cl).entry.tqe_prev = (*cl).entry.tqe_next;
        free((*cl).line as *mut ::core::ffi::c_void);
        free(cl as *mut ::core::ffi::c_void);
        cl = cl1;
    }
    if !(*c).flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
        bufferevent_free((*cs).write_event);
    }
    bufferevent_free((*cs).read_event);
    control_reset_offsets(c);
    cw = control_windows_RB_MINMAX(&raw mut (*cs).windows, RB_NEGINF);
    while !cw.is_null() && {
        cw1 = control_windows_RB_NEXT(cw);
        1 as ::core::ffi::c_int != 0
    } {
        control_windows_RB_REMOVE(&raw mut (*cs).windows, cw);
        free(cw as *mut ::core::ffi::c_void);
        cw = cw1;
    }
    cb = (*cs).all_blocks.tqh_first;
    while !cb.is_null() && {
        cb1 = (*cb).all_entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        control_free_block(cs, cb);
        cb = cb1;
    }
    (*c).control_state = ::core::ptr::null_mut::<control_state>();
    free(cs as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn control_add_sub(
    mut c: *mut client,
    mut name: *const ::core::ffi::c_char,
    mut type_0: monitor_type,
    mut id: ::core::ffi::c_int,
    mut format: *const ::core::ffi::c_char,
) {
    let mut cs: *mut control_state = (*c).control_state;
    monitor_add((*cs).subs, name, type_0, id, format, MONITOR_NOTIFY_INITIAL);
}
#[no_mangle]
pub unsafe extern "C" fn control_remove_sub(
    mut c: *mut client,
    mut name: *const ::core::ffi::c_char,
) {
    let mut cs: *mut control_state = (*c).control_state;
    monitor_remove((*cs).subs, name);
}

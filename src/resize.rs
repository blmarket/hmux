pub use crate::src::shared::limits::{__INT_MAX__, UINT_MAX};
pub use crate::src::shared::window::{
    WINDOW_MAXIMUM, WINDOW_MINIMUM, WINDOW_RESIZE, WINDOW_SIZE_LARGEST, WINDOW_SIZE_LATEST,
    WINDOW_SIZE_MANUAL,
};
pub use crate::src::shared::pane::{
    PANE_MINIMUM, window_pane_offset, window_pane_resize, window_pane_resize_entry,
    window_pane_resizes,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::tree::{RB_NEGINF};
pub use crate::src::shared::client::{
    CLIENT_CONTROL, CLIENT_DEAD, CLIENT_EXIT, CLIENT_IGNORESIZE, CLIENT_NOSIZEFLAGS,
    CLIENT_SIZECHANGED, CLIENT_STATUSOFF, CLIENT_SUSPENDED, CLIENT_UNATTACHEDFLAGS,
    CLIENT_WINDOWSIZECHANGED,
};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::terminal::*;
use crate::src::shared::event::*;
use crate::src::shared::display::*;
use crate::src::shared::layout::*;
use crate::src::shared::message::*;
use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
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
    pub type control_state;
    pub type cmdq_list;
    pub type event_payload;
    fn sscanf(
        __s: *const ::core::ffi::c_char,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    static mut global_w_options: *mut options;
    fn event_payload_create() -> *mut event_payload;
    fn event_payload_set_target(_: *mut event_payload, _: *mut cmd_find_state);
    fn event_payload_set_uint(_: *mut event_payload, _: *const ::core::ffi::c_char, _: u_int);
    fn event_payload_set_window(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut window,
    );
    fn events_fire(_: *const ::core::ffi::c_char, _: *mut event_payload);
    fn events_fire_window(_: *const ::core::ffi::c_char, _: *mut window);
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn tty_update_window_offset(_: *mut window);
    fn cmd_find_from_window(
        _: *mut cmd_find_state,
        _: *mut window,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    static mut clients: clients;
    fn server_redraw_window(_: *mut window);
    fn status_update_cache(_: *mut session);
    fn status_line_size(_: *mut client) -> u_int;
    static mut windows: windows;
    fn windows_RB_MINMAX(_: *mut windows, _: ::core::ffi::c_int) -> *mut window;
    fn windows_RB_NEXT(_: *mut window) -> *mut window;
    fn window_has_pane(_: *mut window, _: *mut window_pane) -> ::core::ffi::c_int;
    fn window_resize(
        _: *mut window,
        _: u_int,
        _: u_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn window_zoom(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_unzoom(_: *mut window, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn window_zoomed_pane(_: *mut window) -> *mut window_pane;
    fn layout_resize(_: *mut window, _: u_int, _: u_int);
    fn control_get_window_size(
        _: *mut client,
        _: u_int,
        _: *mut u_int,
        _: *mut u_int,
    ) -> ::core::ffi::c_int;
    static mut sessions: sessions;
    fn sessions_RB_NEXT(_: *mut session) -> *mut session;
    fn sessions_RB_MINMAX(_: *mut sessions, _: ::core::ffi::c_int) -> *mut session;
    fn session_has(_: *mut session, _: *mut window) -> ::core::ffi::c_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
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
pub struct windows {
    pub rbh_root: *mut window,
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

unsafe extern "C" fn resize_fire_window_resized(
    mut w: *mut window,
    mut old_sx: u_int,
    mut old_sy: u_int,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_window(&raw mut fs, w, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    event_payload_set_uint(
        ep,
        b"width\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).sx,
    );
    event_payload_set_uint(
        ep,
        b"height\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).sy,
    );
    event_payload_set_uint(
        ep,
        b"old_width\0" as *const u8 as *const ::core::ffi::c_char,
        old_sx,
    );
    event_payload_set_uint(
        ep,
        b"old_height\0" as *const u8 as *const ::core::ffi::c_char,
        old_sy,
    );
    events_fire(
        b"window-resized\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
#[no_mangle]
pub unsafe extern "C" fn resize_window(
    mut w: *mut window,
    mut sx: u_int,
    mut sy: u_int,
    mut xpixel: ::core::ffi::c_int,
    mut ypixel: ::core::ffi::c_int,
) {
    let mut zwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut old_sx: u_int = (*w).sx;
    let mut old_sy: u_int = (*w).sy;
    if sx < WINDOW_MINIMUM as u_int {
        sx = WINDOW_MINIMUM as u_int;
    }
    if sx > WINDOW_MAXIMUM as u_int {
        sx = WINDOW_MAXIMUM as u_int;
    }
    if sy < WINDOW_MINIMUM as u_int {
        sy = WINDOW_MINIMUM as u_int;
    }
    if sy > WINDOW_MAXIMUM as u_int {
        sy = WINDOW_MAXIMUM as u_int;
    }
    zwp = window_zoomed_pane(w);
    if !zwp.is_null() {
        window_unzoom(w, 1 as ::core::ffi::c_int);
    }
    layout_resize(w, sx, sy);
    if sx < (*(*w).layout_root).g.sx {
        sx = (*(*w).layout_root).g.sx;
    }
    if sy < (*(*w).layout_root).g.sy {
        sy = (*(*w).layout_root).g.sy;
    }
    window_resize(w, sx, sy, xpixel, ypixel);
    log_debug(
        b"%s: @%u resized to %ux%u; layout %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"resize_window\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        sx,
        sy,
        (*(*w).layout_root).g.sx,
        (*(*w).layout_root).g.sy,
    );
    if !zwp.is_null() && window_has_pane(w, zwp) != 0 {
        window_zoom(zwp);
    }
    tty_update_window_offset(w);
    server_redraw_window(w);
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    resize_fire_window_resized(w, old_sx, old_sy);
    (*w).flags &= !WINDOW_RESIZE;
}
unsafe extern "C" fn ignore_client_size(mut c: *mut client) -> ::core::ffi::c_int {
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    if (*c).session.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    if (*c).flags & CLIENT_NOSIZEFLAGS as uint64_t != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if (*c).flags & CLIENT_IGNORESIZE as uint64_t != 0 {
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            if !(*loop_0).session.is_null() {
                if !((*loop_0).flags & CLIENT_NOSIZEFLAGS as uint64_t != 0) {
                    if !(*loop_0).flags & CLIENT_IGNORESIZE as uint64_t != 0 {
                        return 1 as ::core::ffi::c_int;
                    }
                }
            }
            loop_0 = (*loop_0).entry.tqe_next;
        }
    }
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0
        && !(*c).flags & CLIENT_SIZECHANGED as uint64_t != 0
        && !(*c).flags as ::core::ffi::c_ulonglong & CLIENT_WINDOWSIZECHANGED != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn clients_with_window(mut w: *mut window) -> u_int {
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut n: u_int = 0 as u_int;
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        if !(ignore_client_size(loop_0) != 0 || session_has((*loop_0).session, w) == 0) {
            n = n.wrapping_add(1);
            if n > 1 as u_int {
                break;
            }
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    return n;
}
unsafe extern "C" fn clients_calculate_size(
    mut type_0: ::core::ffi::c_int,
    mut current: ::core::ffi::c_int,
    mut c: *mut client,
    mut s: *mut session,
    mut w: *mut window,
    mut skip_client: Option<
        unsafe extern "C" fn(
            *mut client,
            ::core::ffi::c_int,
            ::core::ffi::c_int,
            *mut session,
            *mut window,
        ) -> ::core::ffi::c_int,
    >,
    mut sx: *mut u_int,
    mut sy: *mut u_int,
    mut xpixel: *mut u_int,
    mut ypixel: *mut u_int,
) -> ::core::ffi::c_int {
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut n: u_int = 0 as u_int;
    if type_0 == WINDOW_SIZE_LARGEST {
        *sx = 0 as u_int;
        *sy = 0 as u_int;
    } else if !w.is_null() && type_0 == WINDOW_SIZE_MANUAL {
        *sx = (*w).manual_sx;
        *sy = (*w).manual_sy;
        log_debug(
            b"%s: manual size %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
            *sx,
            *sy,
        );
    } else {
        *sx = UINT_MAX as u_int;
        *sy = UINT_MAX as u_int;
    }
    *ypixel = 0 as u_int;
    *xpixel = *ypixel;
    if type_0 == WINDOW_SIZE_LATEST && !w.is_null() {
        n = clients_with_window(w);
    }
    if !(type_0 == WINDOW_SIZE_MANUAL) {
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            if loop_0 != c && ignore_client_size(loop_0) != 0 {
                log_debug(
                    b"%s: ignoring %s (1)\0" as *const u8 as *const ::core::ffi::c_char,
                    b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
                    (*loop_0).name,
                );
            } else if loop_0 != c
                && skip_client.expect("non-null function pointer")(loop_0, type_0, current, s, w)
                    != 0
            {
                log_debug(
                    b"%s: skipping %s (1)\0" as *const u8 as *const ::core::ffi::c_char,
                    b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
                    (*loop_0).name,
                );
            } else if type_0 == WINDOW_SIZE_LATEST
                && n > 1 as u_int
                && loop_0 != (*w).latest as *mut client
            {
                log_debug(
                    b"%s: %s is not latest\0" as *const u8 as *const ::core::ffi::c_char,
                    b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
                    (*loop_0).name,
                );
            } else {
                if w.is_null()
                    || control_get_window_size(loop_0, (*w).id, &raw mut cx, &raw mut cy) == 0
                    || cx == 0 as u_int
                    || cy == 0 as u_int
                {
                    cx = (*loop_0).tty.sx;
                    cy = (*loop_0).tty.sy.wrapping_sub(status_line_size(loop_0));
                }
                if type_0 == WINDOW_SIZE_LARGEST {
                    if cx > *sx {
                        *sx = cx;
                    }
                    if cy > *sy {
                        *sy = cy;
                    }
                } else {
                    if cx < *sx {
                        *sx = cx;
                    }
                    if cy < *sy {
                        *sy = cy;
                    }
                }
                if (*loop_0).tty.xpixel > *xpixel && (*loop_0).tty.ypixel > *ypixel {
                    *xpixel = (*loop_0).tty.xpixel;
                    *ypixel = (*loop_0).tty.ypixel;
                }
                log_debug(
                    b"%s: after %s (%ux%u), size is %ux%u\0" as *const u8
                        as *const ::core::ffi::c_char,
                    b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
                    (*loop_0).name,
                    cx,
                    cy,
                    *sx,
                    *sy,
                );
            }
            loop_0 = (*loop_0).entry.tqe_next;
        }
        if *sx != UINT_MAX && *sy != UINT_MAX {
            log_debug(
                b"%s: calculated size %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
                b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
                *sx,
                *sy,
            );
        } else {
            log_debug(
                b"%s: no calculated size\0" as *const u8 as *const ::core::ffi::c_char,
                b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    }
    if !w.is_null() {
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            if !(loop_0 != c && ignore_client_size(loop_0) != 0) {
                if !(loop_0 != c
                    && skip_client.expect("non-null function pointer")(
                        loop_0, type_0, current, s, w,
                    ) != 0)
                {
                    if !(!(*loop_0).flags as ::core::ffi::c_ulonglong & CLIENT_WINDOWSIZECHANGED
                        != 0)
                    {
                        if !(control_get_window_size(loop_0, (*w).id, &raw mut cx, &raw mut cy)
                            == 0)
                        {
                            log_debug(
                                b"%s: %s size for @%u is %ux%u\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                b"clients_calculate_size\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                (*loop_0).name,
                                (*w).id,
                                cx,
                                cy,
                            );
                            if cx != 0 as u_int && *sx > cx {
                                *sx = cx;
                            }
                            if cy != 0 as u_int && *sy > cy {
                                *sy = cy;
                            }
                        }
                    }
                }
            }
            loop_0 = (*loop_0).entry.tqe_next;
        }
    }
    if *sx != UINT_MAX && *sy != UINT_MAX {
        log_debug(
            b"%s: calculated size %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
            *sx,
            *sy,
        );
    } else {
        log_debug(
            b"%s: no calculated size\0" as *const u8 as *const ::core::ffi::c_char,
            b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if type_0 == WINDOW_SIZE_MANUAL {
        log_debug(
            b"%s: type is manual\0" as *const u8 as *const ::core::ffi::c_char,
            b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return (w != NULL as *mut window) as ::core::ffi::c_int;
    }
    if type_0 == WINDOW_SIZE_LARGEST {
        log_debug(
            b"%s: type is largest\0" as *const u8 as *const ::core::ffi::c_char,
            b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return (*sx != 0 as u_int && *sy != 0 as u_int) as ::core::ffi::c_int;
    }
    if type_0 == WINDOW_SIZE_LATEST {
        log_debug(
            b"%s: type is latest\0" as *const u8 as *const ::core::ffi::c_char,
            b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        log_debug(
            b"%s: type is smallest\0" as *const u8 as *const ::core::ffi::c_char,
            b"clients_calculate_size\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    return (*sx != UINT_MAX && *sy != UINT_MAX) as ::core::ffi::c_int;
}
unsafe extern "C" fn default_window_size_skip_client(
    mut loop_0: *mut client,
    mut type_0: ::core::ffi::c_int,
    mut current: ::core::ffi::c_int,
    mut s: *mut session,
    mut w: *mut window,
) -> ::core::ffi::c_int {
    if !w.is_null() && session_has((*loop_0).session, w) == 0 {
        return 1 as ::core::ffi::c_int;
    }
    if w.is_null() && (*loop_0).session != s {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn default_window_size(
    mut c: *mut client,
    mut s: *mut session,
    mut w: *mut window,
    mut sx: *mut u_int,
    mut sy: *mut u_int,
    mut xpixel: *mut u_int,
    mut ypixel: *mut u_int,
    mut type_0: ::core::ffi::c_int,
) {
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if type_0 == -(1 as ::core::ffi::c_int) {
        type_0 = options_get_number(
            global_w_options,
            b"window-size\0" as *const u8 as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int;
    }
    if type_0 == WINDOW_SIZE_LATEST && !c.is_null() && ignore_client_size(c) == 0 {
        *sx = (*c).tty.sx;
        *sy = (*c).tty.sy.wrapping_sub(status_line_size(c));
        *xpixel = (*c).tty.xpixel;
        *ypixel = (*c).tty.ypixel;
        log_debug(
            b"%s: using %ux%u from %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"default_window_size\0" as *const u8 as *const ::core::ffi::c_char,
            *sx,
            *sy,
            (*c).name,
        );
    } else {
        if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            c = ::core::ptr::null_mut::<client>();
        }
        if clients_calculate_size(
            type_0,
            0 as ::core::ffi::c_int,
            c,
            s,
            w,
            Some(
                default_window_size_skip_client
                    as unsafe extern "C" fn(
                        *mut client,
                        ::core::ffi::c_int,
                        ::core::ffi::c_int,
                        *mut session,
                        *mut window,
                    ) -> ::core::ffi::c_int,
            ),
            sx,
            sy,
            xpixel,
            ypixel,
        ) == 0
        {
            value = options_get_string(
                (*s).options,
                b"default-size\0" as *const u8 as *const ::core::ffi::c_char,
            );
            if sscanf(
                value,
                b"%ux%u\0" as *const u8 as *const ::core::ffi::c_char,
                sx,
                sy,
            ) != 2 as ::core::ffi::c_int
            {
                *sx = 80 as u_int;
                *sy = 24 as u_int;
            }
            log_debug(
                b"%s: using %ux%u from default-size\0" as *const u8 as *const ::core::ffi::c_char,
                b"default_window_size\0" as *const u8 as *const ::core::ffi::c_char,
                *sx,
                *sy,
            );
        }
    }
    if *sx < WINDOW_MINIMUM as u_int {
        *sx = WINDOW_MINIMUM as u_int;
    }
    if *sx > WINDOW_MAXIMUM as u_int {
        *sx = WINDOW_MAXIMUM as u_int;
    }
    if *sy < WINDOW_MINIMUM as u_int {
        *sy = WINDOW_MINIMUM as u_int;
    }
    if *sy > WINDOW_MAXIMUM as u_int {
        *sy = WINDOW_MAXIMUM as u_int;
    }
    log_debug(
        b"%s: resulting size is %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"default_window_size\0" as *const u8 as *const ::core::ffi::c_char,
        *sx,
        *sy,
    );
}
unsafe extern "C" fn recalculate_size_skip_client(
    mut loop_0: *mut client,
    mut type_0: ::core::ffi::c_int,
    mut current: ::core::ffi::c_int,
    mut s: *mut session,
    mut w: *mut window,
) -> ::core::ffi::c_int {
    if (*(*loop_0).session).curw.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    if current != 0 {
        return ((*(*(*loop_0).session).curw).window != w) as ::core::ffi::c_int;
    }
    return (session_has((*loop_0).session, w) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn recalculate_size(mut w: *mut window, mut now: ::core::ffi::c_int) {
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xpixel: u_int = 0 as u_int;
    let mut ypixel: u_int = 0 as u_int;
    let mut type_0: ::core::ffi::c_int = 0;
    let mut current: ::core::ffi::c_int = 0;
    let mut changed: ::core::ffi::c_int = 0;
    if (*w).active.is_null() {
        return;
    }
    log_debug(
        b"%s: @%u is %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"recalculate_size\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        (*w).sx,
        (*w).sy,
    );
    type_0 = options_get_number(
        (*w).options,
        b"window-size\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    current = options_get_number(
        (*w).options,
        b"aggressive-resize\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    changed = clients_calculate_size(
        type_0,
        current,
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        w,
        Some(
            recalculate_size_skip_client
                as unsafe extern "C" fn(
                    *mut client,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                    *mut session,
                    *mut window,
                ) -> ::core::ffi::c_int,
        ),
        &raw mut sx,
        &raw mut sy,
        &raw mut xpixel,
        &raw mut ypixel,
    );
    if (*w).flags & WINDOW_RESIZE != 0 {
        if now == 0 && changed != 0 && (*w).new_sx == sx && (*w).new_sy == sy {
            changed = 0 as ::core::ffi::c_int;
        }
    } else if now == 0 && changed != 0 && (*w).sx == sx && (*w).sy == sy {
        changed = 0 as ::core::ffi::c_int;
    }
    if changed == 0 {
        log_debug(
            b"%s: @%u no size change\0" as *const u8 as *const ::core::ffi::c_char,
            b"recalculate_size\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
        );
        tty_update_window_offset(w);
        return;
    }
    log_debug(
        b"%s: @%u new size %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"recalculate_size\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        sx,
        sy,
    );
    if now != 0 || type_0 == WINDOW_SIZE_MANUAL {
        resize_window(
            w,
            sx,
            sy,
            xpixel as ::core::ffi::c_int,
            ypixel as ::core::ffi::c_int,
        );
    } else {
        (*w).new_sx = sx;
        (*w).new_sy = sy;
        (*w).new_xpixel = xpixel;
        (*w).new_ypixel = ypixel;
        (*w).flags |= WINDOW_RESIZE;
        tty_update_window_offset(w);
    };
}
#[no_mangle]
pub unsafe extern "C" fn recalculate_sizes() {
    recalculate_sizes_now(0 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn recalculate_sizes_now(mut now: ::core::ffi::c_int) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        (*s).attached = 0 as u_int;
        status_update_cache(s);
        s = sessions_RB_NEXT(s);
    }
    c = clients.tqh_first;
    while !c.is_null() {
        s = (*c).session;
        if !s.is_null() && (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t == 0 {
            (*s).attached = (*s).attached.wrapping_add(1);
        }
        if !(ignore_client_size(c) != 0) {
            if (*c).tty.sy <= (*s).statuslines || (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                (*c).flags |= CLIENT_STATUSOFF as uint64_t;
            } else {
                (*c).flags &= !CLIENT_STATUSOFF as uint64_t;
            }
        }
        c = (*c).entry.tqe_next;
    }
    w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
    while !w.is_null() {
        recalculate_size(w, now);
        w = windows_RB_NEXT(w);
    }
}

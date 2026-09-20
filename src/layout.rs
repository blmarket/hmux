pub use crate::src::shared::spawn::{
    SPAWN_BEFORE, SPAWN_FLOATOVERZOOM, SPAWN_FULLSIZE, SPAWN_HORIZONTAL, SPAWN_SPLIT,
    SPAWN_ZOOM,
};
pub use crate::src::shared::pane::{
    PANE_MAXIMUM, PANE_MINIMUM, PANE_REDRAWSCROLLBAR, PANE_SCROLLBARS_ALWAYS,
    PANE_SCROLLBARS_LEFT, PANE_STATUS_BOTTOM, PANE_STATUS_TOP, window_pane_offset,
    window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::layout::*;
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
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn events_fire_window(_: *const ::core::ffi::c_char, _: *mut window);
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_strtonum_and_expand(
        _: *mut args,
        _: u_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut cmdq_item,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn args_percentage_and_expand(
        _: *mut args,
        _: u_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut cmdq_item,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn redraw_invalidate_scene(_: *mut window);
    fn window_active_pane_is_over_zoom(_: *mut window) -> ::core::ffi::c_int;
    fn window_push_zoom(
        _: *mut window,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn window_pane_resize(_: *mut window_pane, _: u_int, _: u_int);
    fn window_pane_scrollbar_reserve(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_get_pane_lines(_: *mut window_pane) -> pane_lines;
    fn window_get_pane_status(_: *mut window) -> ::core::ffi::c_int;
    fn window_pane_get_pane_status(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_is_floating(_: *mut window_pane) -> ::core::ffi::c_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
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
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
unsafe extern "C" fn layout_geometry_init(mut lg: *mut layout_geometry) {
    (*lg).sx = UINT_MAX as u_int;
    (*lg).sy = UINT_MAX as u_int;
    (*lg).xoff = INT_MAX;
    (*lg).yoff = INT_MAX;
}
#[no_mangle]
pub unsafe extern "C" fn layout_create_cell(mut lcparent: *mut layout_cell) -> *mut layout_cell {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    lc = xcalloc(1 as size_t, ::core::mem::size_of::<layout_cell>() as size_t) as *mut layout_cell;
    (*lc).type_0 = LAYOUT_WINDOWPANE;
    (*lc).parent = lcparent;
    (*lc).cells.tqh_first = ::core::ptr::null_mut::<layout_cell>();
    (*lc).cells.tqh_last = &raw mut (*lc).cells.tqh_first;
    layout_geometry_init(&raw mut (*lc).g);
    layout_geometry_init(&raw mut (*lc).fg);
    return lc;
}
#[no_mangle]
pub unsafe extern "C" fn layout_free_cell(
    mut lc: *mut layout_cell,
    mut only_nodes: ::core::ffi::c_int,
) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcnext: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if lc.is_null()
        || only_nodes != 0
            && (*lc).type_0 as ::core::ffi::c_uint
                == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    match (*lc).type_0 as ::core::ffi::c_uint {
        0 | 1 => {
            lcchild = (*lc).cells.tqh_first;
            while !lcchild.is_null() {
                lcnext = (*lcchild).entry.tqe_next;
                if only_nodes == 0
                    || (*lcchild).type_0 as ::core::ffi::c_uint
                        != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if !(*lcchild).entry.tqe_next.is_null() {
                        (*(*lcchild).entry.tqe_next).entry.tqe_prev = (*lcchild).entry.tqe_prev;
                    } else {
                        (*lc).cells.tqh_last = (*lcchild).entry.tqe_prev;
                    }
                    *(*lcchild).entry.tqe_prev = (*lcchild).entry.tqe_next;
                    layout_free_cell(lcchild, only_nodes);
                }
                lcchild = lcnext;
            }
        }
        2 => {
            if !(*lc).wp.is_null() && !(*(*lc).wp).layout_cell.is_null() {
                (*(*(*lc).wp).layout_cell).parent = ::core::ptr::null_mut::<layout_cell>();
                (*(*lc).wp).layout_cell = ::core::ptr::null_mut::<layout_cell>();
            }
        }
        _ => {}
    }
    free(lc as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn layout_print_cell(
    mut lc: *mut layout_cell,
    mut hdr: *const ::core::ffi::c_char,
    mut n: u_int,
) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut type_0: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if lc.is_null() {
        return;
    }
    match (*lc).type_0 as ::core::ffi::c_uint {
        0 => {
            type_0 = b"LEFTRIGHT\0" as *const u8 as *const ::core::ffi::c_char;
        }
        1 => {
            type_0 = b"TOPBOTTOM\0" as *const u8 as *const ::core::ffi::c_char;
        }
        2 => {
            type_0 = b"WINDOWPANE\0" as *const u8 as *const ::core::ffi::c_char;
        }
        _ => {
            type_0 = b"UNKNOWN\0" as *const u8 as *const ::core::ffi::c_char;
        }
    }
    log_debug(
        b"%s:%*s%p type %s [parent %p] wp=%p [%d,%d %ux%u]\0" as *const u8
            as *const ::core::ffi::c_char,
        hdr,
        n,
        b" \0" as *const u8 as *const ::core::ffi::c_char,
        lc,
        type_0,
        (*lc).parent,
        (*lc).wp,
        (*lc).g.xoff,
        (*lc).g.yoff,
        (*lc).g.sx,
        (*lc).g.sy,
    );
    match (*lc).type_0 as ::core::ffi::c_uint {
        0 | 1 => {
            lcchild = (*lc).cells.tqh_first;
            while !lcchild.is_null() {
                layout_print_cell(lcchild, hdr, n.wrapping_add(1 as u_int));
                lcchild = (*lcchild).entry.tqe_next;
            }
        }
        2 | _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn layout_search_by_border(
    mut lc: *mut layout_cell,
    mut x: u_int,
    mut y: u_int,
) -> *mut layout_cell {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut last: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    lcchild = (*lc).cells.tqh_first;
    while !lcchild.is_null() {
        if x as ::core::ffi::c_int >= (*lcchild).g.xoff
            && (x as ::core::ffi::c_int) < (*lcchild).g.xoff + (*lcchild).g.sx as ::core::ffi::c_int
            && y as ::core::ffi::c_int >= (*lcchild).g.yoff
            && (y as ::core::ffi::c_int) < (*lcchild).g.yoff + (*lcchild).g.sy as ::core::ffi::c_int
        {
            return layout_search_by_border(lcchild, x, y);
        }
        if last.is_null() {
            last = lcchild;
        } else {
            match (*lc).type_0 as ::core::ffi::c_uint {
                0 => {
                    if (x as ::core::ffi::c_int) < (*lcchild).g.xoff
                        && x as ::core::ffi::c_int
                            >= (*last).g.xoff + (*last).g.sx as ::core::ffi::c_int
                    {
                        return last;
                    }
                }
                1 => {
                    if (y as ::core::ffi::c_int) < (*lcchild).g.yoff
                        && y as ::core::ffi::c_int
                            >= (*last).g.yoff + (*last).g.sy as ::core::ffi::c_int
                    {
                        return last;
                    }
                }
                2 | _ => {}
            }
            last = lcchild;
        }
        lcchild = (*lcchild).entry.tqe_next;
    }
    return ::core::ptr::null_mut::<layout_cell>();
}
#[no_mangle]
pub unsafe extern "C" fn layout_set_size(
    mut lc: *mut layout_cell,
    mut sx: u_int,
    mut sy: u_int,
    mut xoff: ::core::ffi::c_int,
    mut yoff: ::core::ffi::c_int,
) {
    (*lc).g.sx = sx;
    (*lc).g.sy = sy;
    (*lc).g.xoff = xoff;
    (*lc).g.yoff = yoff;
}
#[no_mangle]
pub unsafe extern "C" fn layout_make_leaf(mut lc: *mut layout_cell, mut wp: *mut window_pane) {
    (*lc).type_0 = LAYOUT_WINDOWPANE;
    (*lc).cells.tqh_first = ::core::ptr::null_mut::<layout_cell>();
    (*lc).cells.tqh_last = &raw mut (*lc).cells.tqh_first;
    (*wp).layout_cell = lc as *mut layout_cell;
    (*lc).wp = wp;
}
#[no_mangle]
pub unsafe extern "C" fn layout_make_node(mut lc: *mut layout_cell, mut type_0: layout_type) {
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fatalx(b"bad layout type\0" as *const u8 as *const ::core::ffi::c_char);
    }
    (*lc).type_0 = type_0;
    (*lc).cells.tqh_first = ::core::ptr::null_mut::<layout_cell>();
    (*lc).cells.tqh_last = &raw mut (*lc).cells.tqh_first;
    if !(*lc).wp.is_null() {
        (*(*lc).wp).layout_cell = ::core::ptr::null_mut::<layout_cell>();
    }
    (*lc).wp = ::core::ptr::null_mut::<window_pane>();
}
#[no_mangle]
pub unsafe extern "C" fn layout_cell_is_tiled(mut lc: *mut layout_cell) -> ::core::ffi::c_int {
    let mut is_leaf: ::core::ffi::c_int = ((*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
    let mut is_floating: ::core::ffi::c_int = (*lc).flags & LAYOUT_CELL_FLOATING;
    return (is_leaf != 0 && is_floating == 0) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn layout_cell_has_tiled_child(
    mut lc: *mut layout_cell,
) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    lcchild = (*lc).cells.tqh_first;
    while !lcchild.is_null() {
        if layout_cell_is_tiled(lcchild) != 0 || layout_cell_has_tiled_child(lcchild) != 0 {
            return 1 as ::core::ffi::c_int;
        }
        lcchild = (*lcchild).entry.tqe_next;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn layout_cell_is_first_tiled(mut lc: *mut layout_cell) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = (*lc).parent;
    if lcparent.is_null() {
        return layout_cell_is_tiled(lc);
    }
    lcchild = (*lcparent).cells.tqh_first;
    while !lcchild.is_null() {
        if layout_cell_is_tiled(lcchild) != 0 || layout_cell_has_tiled_child(lcchild) != 0 {
            break;
        }
        lcchild = (*lcchild).entry.tqe_next;
    }
    return (lcchild == lc) as ::core::ffi::c_int;
}
unsafe extern "C" fn layout_cell_get_first_tiled(mut lc: *mut layout_cell) -> *mut layout_cell {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild2: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if layout_cell_is_tiled(lc) != 0 {
        return lc;
    }
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    lcchild = (*lc).cells.tqh_first;
    while !lcchild.is_null() {
        if layout_cell_is_tiled(lcchild) != 0 {
            return lcchild;
        }
        if (*lcchild).type_0 as ::core::ffi::c_uint
            != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            lcchild2 = layout_cell_get_first_tiled(lcchild);
            if !lcchild2.is_null() {
                return lcchild2;
            }
        }
        lcchild = (*lcchild).entry.tqe_next;
    }
    return ::core::ptr::null_mut::<layout_cell>();
}
unsafe extern "C" fn layout_fix_offsets1(mut lc: *mut layout_cell) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        xoff = (*lc).g.xoff;
        lcchild = (*lc).cells.tqh_first;
        while !lcchild.is_null() {
            if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
                (*lcchild).g.xoff = xoff;
                (*lcchild).g.yoff = (*lc).g.yoff;
                if (*lcchild).type_0 as ::core::ffi::c_uint
                    != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    layout_fix_offsets1(lcchild);
                }
                xoff = (xoff as u_int).wrapping_add((*lcchild).g.sx.wrapping_add(1 as u_int))
                    as ::core::ffi::c_int as ::core::ffi::c_int;
            }
            lcchild = (*lcchild).entry.tqe_next;
        }
    } else {
        yoff = (*lc).g.yoff;
        lcchild = (*lc).cells.tqh_first;
        while !lcchild.is_null() {
            if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
                (*lcchild).g.xoff = (*lc).g.xoff;
                (*lcchild).g.yoff = yoff;
                if (*lcchild).type_0 as ::core::ffi::c_uint
                    != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    layout_fix_offsets1(lcchild);
                }
                yoff = (yoff as u_int).wrapping_add((*lcchild).g.sy.wrapping_add(1 as u_int))
                    as ::core::ffi::c_int as ::core::ffi::c_int;
            }
            lcchild = (*lcchild).entry.tqe_next;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn layout_fix_offsets(mut w: *mut window) {
    let mut lc: *mut layout_cell = (*w).layout_root;
    if (*lc).flags & LAYOUT_CELL_FLOATING != 0 {
        return;
    }
    (*lc).g.xoff = 0 as ::core::ffi::c_int;
    (*lc).g.yoff = 0 as ::core::ffi::c_int;
    layout_fix_offsets1(lc);
}
unsafe extern "C" fn layout_cell_is_last_tiled(mut lc: *mut layout_cell) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = (*lc).parent;
    if lcparent.is_null() {
        return layout_cell_is_tiled(lc);
    }
    lcchild = *(*((*lcparent).cells.tqh_last as *mut layout_cells)).tqh_last;
    while !lcchild.is_null() {
        if layout_cell_is_tiled(lcchild) != 0 || layout_cell_has_tiled_child(lcchild) != 0 {
            break;
        }
        lcchild = *(*((*lcchild).entry.tqe_prev as *mut layout_cells)).tqh_last;
    }
    return (lcchild == lc) as ::core::ffi::c_int;
}
unsafe extern "C" fn layout_cell_is_top(
    mut root: *mut layout_cell,
    mut lc: *mut layout_cell,
) -> ::core::ffi::c_int {
    let mut next: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    while lc != root {
        next = (*lc).parent;
        if next.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        if (*next).type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            && layout_cell_is_first_tiled(lc) == 0
        {
            return 0 as ::core::ffi::c_int;
        }
        lc = next;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn layout_cell_is_bottom(
    mut root: *mut layout_cell,
    mut lc: *mut layout_cell,
) -> ::core::ffi::c_int {
    let mut next: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    while lc != root {
        next = (*lc).parent;
        if next.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        if (*next).type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            && layout_cell_is_last_tiled(lc) == 0
        {
            return 0 as ::core::ffi::c_int;
        }
        lc = next;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn layout_add_horizontal_border(
    mut root: *mut layout_cell,
    mut lc: *mut layout_cell,
    mut status: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if status == PANE_STATUS_TOP {
        return layout_cell_is_top(root, lc);
    }
    if status == PANE_STATUS_BOTTOM {
        return layout_cell_is_bottom(root, lc);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn layout_fix_panes(mut w: *mut window, mut skip: *mut window_pane) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut root: *mut layout_cell = (*w).layout_root;
    let mut status: ::core::ffi::c_int = 0;
    let mut sb_w: ::core::ffi::c_int = 0;
    let mut sb_pad: ::core::ffi::c_int = 0;
    let mut old_xoff: ::core::ffi::c_int = 0;
    let mut old_yoff: ::core::ffi::c_int = 0;
    let mut changed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut old_sx: u_int = 0;
    let mut old_sy: u_int = 0;
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
        if !(lc.is_null() || wp == skip) {
            old_xoff = (*wp).xoff;
            old_yoff = (*wp).yoff;
            old_sx = (*wp).sx;
            old_sy = (*wp).sy;
            (*wp).xoff = (*lc).g.xoff;
            (*wp).yoff = (*lc).g.yoff;
            sx = (*lc).g.sx;
            sy = (*lc).g.sy;
            status = window_pane_get_pane_status(wp);
            if window_pane_is_floating(wp) == 0
                && layout_add_horizontal_border(root, lc, status) != 0
            {
                if status == PANE_STATUS_TOP {
                    (*wp).yoff += 1;
                }
                if sy > 1 as u_int {
                    sy = sy.wrapping_sub(1);
                }
            }
            if window_pane_scrollbar_reserve(wp) != 0 {
                sb_w = (*wp).scrollbar_style.width;
                sb_pad = (*wp).scrollbar_style.pad;
                if sb_w < 1 as ::core::ffi::c_int {
                    sb_w = 1 as ::core::ffi::c_int;
                }
                if sb_pad < 0 as ::core::ffi::c_int {
                    sb_pad = 0 as ::core::ffi::c_int;
                }
                if (*w).sb_pos == PANE_SCROLLBARS_LEFT {
                    if sx as ::core::ffi::c_int - sb_w - sb_pad < PANE_MINIMUM {
                        (*wp).xoff = (*wp).xoff + sx as ::core::ffi::c_int - PANE_MINIMUM;
                        sx = PANE_MINIMUM as u_int;
                    } else {
                        sx = sx.wrapping_sub(sb_w as u_int).wrapping_sub(sb_pad as u_int);
                        (*wp).xoff = (*wp).xoff + sb_w + sb_pad;
                    }
                } else if sx as ::core::ffi::c_int - sb_w - sb_pad < PANE_MINIMUM {
                    sx = PANE_MINIMUM as u_int;
                } else {
                    sx = sx.wrapping_sub(sb_w as u_int).wrapping_sub(sb_pad as u_int);
                }
                (*wp).flags |= PANE_REDRAWSCROLLBAR;
            }
            window_pane_resize(wp, sx, sy);
            if (*wp).xoff != old_xoff
                || (*wp).yoff != old_yoff
                || (*wp).sx != old_sx
                || (*wp).sy != old_sy
            {
                changed = 1 as ::core::ffi::c_int;
            }
        }
        wp = (*wp).entry.tqe_next;
    }
    if changed != 0 {
        redraw_invalidate_scene(w);
    }
}
#[no_mangle]
pub unsafe extern "C" fn layout_count_cells(
    mut lc: *mut layout_cell,
    mut with_floating: ::core::ffi::c_int,
) -> u_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut count: u_int = 0 as u_int;
    match (*lc).type_0 as ::core::ffi::c_uint {
        2 => {
            if (*lc).flags & LAYOUT_CELL_FLOATING != 0 && with_floating == 0 {
                return 0 as u_int;
            }
            return 1 as u_int;
        }
        0 | 1 => {
            lcchild = (*lc).cells.tqh_first;
            while !lcchild.is_null() {
                count = count.wrapping_add(layout_count_cells(lcchild, with_floating));
                lcchild = (*lcchild).entry.tqe_next;
            }
            return count;
        }
        _ => {
            fatalx(b"bad layout type\0" as *const u8 as *const ::core::ffi::c_char);
        }
    };
}
unsafe extern "C" fn layout_resize_check(
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
) -> u_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut root: *mut layout_cell = (*w).layout_root;
    let mut sb_style: *mut style = &raw mut (*(*w).active).scrollbar_style;
    let mut available: u_int = 0;
    let mut minimum: u_int = 0;
    let mut status: ::core::ffi::c_int = 0;
    status = window_get_pane_status(w);
    if layout_cell_is_tiled(lc) == 0 && layout_cell_has_tiled_child(lc) == 0 {
        return 0 as u_int;
    }
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            available = (*lc).g.sx;
            if (*w).sb == PANE_SCROLLBARS_ALWAYS {
                minimum = (PANE_MINIMUM + (*sb_style).width + (*sb_style).pad) as u_int;
            } else {
                minimum = PANE_MINIMUM as u_int;
            }
        } else {
            available = (*lc).g.sy;
            if layout_add_horizontal_border(root, lc, status) != 0 {
                minimum = (PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int;
            } else {
                minimum = PANE_MINIMUM as u_int;
            }
        }
        if available > minimum {
            available = available.wrapping_sub(minimum);
        } else {
            available = 0 as u_int;
        }
    } else if (*lc).type_0 as ::core::ffi::c_uint == type_0 as ::core::ffi::c_uint {
        available = 0 as u_int;
        lcchild = (*lc).cells.tqh_first;
        while !lcchild.is_null() {
            available = available.wrapping_add(layout_resize_check(w, lcchild, type_0));
            lcchild = (*lcchild).entry.tqe_next;
        }
    } else {
        minimum = UINT_MAX as u_int;
        lcchild = (*lc).cells.tqh_first;
        while !lcchild.is_null() {
            if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
                available = layout_resize_check(w, lcchild, type_0);
                if available < minimum {
                    minimum = available;
                }
            }
            lcchild = (*lcchild).entry.tqe_next;
        }
        available = minimum;
    }
    return available;
}
#[no_mangle]
pub unsafe extern "C" fn layout_resize_adjust(
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut change: ::core::ffi::c_int,
) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut changed: ::core::ffi::c_int = 0;
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*lc).g.sx = (*lc).g.sx.wrapping_add(change as u_int);
    } else {
        (*lc).g.sy = (*lc).g.sy.wrapping_add(change as u_int);
    }
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    if (*lc).type_0 as ::core::ffi::c_uint != type_0 as ::core::ffi::c_uint {
        lcchild = (*lc).cells.tqh_first;
        while !lcchild.is_null() {
            if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
                layout_resize_adjust(w, lcchild, type_0, change);
            }
            lcchild = (*lcchild).entry.tqe_next;
        }
        return;
    }
    if layout_cell_has_tiled_child(lc) == 0 {
        return;
    }
    while change != 0 as ::core::ffi::c_int {
        changed = 0 as ::core::ffi::c_int;
        lcchild = (*lc).cells.tqh_first;
        while !lcchild.is_null() {
            if change == 0 as ::core::ffi::c_int {
                break;
            }
            if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
                if change > 0 as ::core::ffi::c_int {
                    layout_resize_adjust(w, lcchild, type_0, 1 as ::core::ffi::c_int);
                    change -= 1;
                    changed = 1 as ::core::ffi::c_int;
                } else if layout_resize_check(w, lcchild, type_0) > 0 as u_int {
                    layout_resize_adjust(w, lcchild, type_0, -(1 as ::core::ffi::c_int));
                    change += 1;
                    changed = 1 as ::core::ffi::c_int;
                }
            }
            lcchild = (*lcchild).entry.tqe_next;
        }
        if changed == 0 {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn layout_resize_set_size(
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut size: u_int,
) {
    let mut change: ::core::ffi::c_int = 0;
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        change = size.wrapping_sub((*lc).g.sx) as ::core::ffi::c_int;
    } else {
        change = size.wrapping_sub((*lc).g.sy) as ::core::ffi::c_int;
    }
    layout_resize_adjust(w, lc, type_0, change);
}
unsafe extern "C" fn layout_cell_get_neighbour_dir(
    mut lc: *mut layout_cell,
    mut direction: ::core::ffi::c_int,
) -> *mut layout_cell {
    let mut lcn: *mut layout_cell = lc;
    loop {
        if direction != 0 {
            lcn = (*lcn).entry.tqe_next;
        } else {
            lcn = *(*((*lcn).entry.tqe_prev as *mut layout_cells)).tqh_last;
        }
        if lcn.is_null() || layout_cell_is_tiled(lcn) != 0 || layout_cell_has_tiled_child(lcn) != 0
        {
            return lcn;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn layout_cell_get_neighbour(mut lc: *mut layout_cell) -> *mut layout_cell {
    let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = (*lc).parent;
    let mut direction: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if lcparent.is_null() {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    if lc == *(*((*lcparent).cells.tqh_last as *mut layout_cells)).tqh_last {
        direction = (direction == 0) as ::core::ffi::c_int;
    }
    lcother = layout_cell_get_neighbour_dir(lc, direction);
    if lcother.is_null() {
        lcother = layout_cell_get_neighbour_dir(lc, (direction == 0) as ::core::ffi::c_int);
    }
    return lcother;
}
#[no_mangle]
pub unsafe extern "C" fn layout_destroy_cell(
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut lcroot: *mut *mut layout_cell,
) {
    let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut change: ::core::ffi::c_int = 0;
    lcparent = (*lc).parent;
    if lcparent.is_null() {
        if *lcroot == lc {
            *lcroot = ::core::ptr::null_mut::<layout_cell>();
        }
        layout_free_cell(lc, 0 as ::core::ffi::c_int);
        return;
    }
    if layout_cell_is_tiled(lc) == 0 {
        if !(*lc).entry.tqe_next.is_null() {
            (*(*lc).entry.tqe_next).entry.tqe_prev = (*lc).entry.tqe_prev;
        } else {
            (*lcparent).cells.tqh_last = (*lc).entry.tqe_prev;
        }
        *(*lc).entry.tqe_prev = (*lc).entry.tqe_next;
        layout_free_cell(lc, 0 as ::core::ffi::c_int);
    } else {
        lcother = layout_cell_get_neighbour(lc);
        if !lcother.is_null() {
            if (*lcparent).type_0 as ::core::ffi::c_uint
                == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                change = (*lc).g.sx.wrapping_add(1 as u_int) as ::core::ffi::c_int;
            } else {
                change = (*lc).g.sy.wrapping_add(1 as u_int) as ::core::ffi::c_int;
            }
            layout_resize_adjust(w, lcother, (*lcparent).type_0, change);
        } else {
            layout_remove_tile(w, lcparent);
        }
        if !(*lc).entry.tqe_next.is_null() {
            (*(*lc).entry.tqe_next).entry.tqe_prev = (*lc).entry.tqe_prev;
        } else {
            (*lcparent).cells.tqh_last = (*lc).entry.tqe_prev;
        }
        *(*lc).entry.tqe_prev = (*lc).entry.tqe_next;
        layout_free_cell(lc, 0 as ::core::ffi::c_int);
    }
    lc = (*lcparent).cells.tqh_first;
    if !lc.is_null() && (*lc).entry.tqe_next.is_null() {
        if !(*lc).entry.tqe_next.is_null() {
            (*(*lc).entry.tqe_next).entry.tqe_prev = (*lc).entry.tqe_prev;
        } else {
            (*lcparent).cells.tqh_last = (*lc).entry.tqe_prev;
        }
        *(*lc).entry.tqe_prev = (*lc).entry.tqe_next;
        (*lc).parent = (*lcparent).parent;
        if (*lc).parent.is_null() {
            if layout_cell_is_tiled(lc) != 0 {
                (*lc).g.xoff = 0 as ::core::ffi::c_int;
                (*lc).g.yoff = 0 as ::core::ffi::c_int;
            }
            *lcroot = lc;
        } else {
            (*lc).entry.tqe_next = (*lcparent).entry.tqe_next;
            if !(*lc).entry.tqe_next.is_null() {
                (*(*lc).entry.tqe_next).entry.tqe_prev = &raw mut (*lc).entry.tqe_next;
            } else {
                (*(*lc).parent).cells.tqh_last = &raw mut (*lc).entry.tqe_next;
            }
            (*lc).entry.tqe_prev = (*lcparent).entry.tqe_prev;
            *(*lc).entry.tqe_prev = lc;
        }
        layout_free_cell(lcparent, 0 as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn layout_init(mut w: *mut window, mut wp: *mut window_pane) {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    (*w).layout_root = layout_create_cell(::core::ptr::null_mut::<layout_cell>());
    lc = (*w).layout_root;
    layout_set_size(
        lc,
        (*w).sx,
        (*w).sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_leaf(lc, wp);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
}
#[no_mangle]
pub unsafe extern "C" fn layout_free(mut w: *mut window, mut only_nodes: ::core::ffi::c_int) {
    layout_free_cell((*w).layout_root, only_nodes);
}
unsafe extern "C" fn layout_clamp_floating_panes(mut w: *mut window, mut sx: u_int, mut sy: u_int) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut pad: u_int = 0;
    let mut avail: u_int = 0;
    let mut csx: u_int = 0;
    let mut csy: u_int = 0;
    wp = (*w).z_index.tqh_first;
    while !wp.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
        if !(lc.is_null() || !(*lc).flags & LAYOUT_CELL_FLOATING != 0) {
            if window_pane_get_pane_lines(wp) as ::core::ffi::c_uint
                == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                pad = 0 as u_int;
            } else {
                pad = 1 as u_int;
            }
            csx = (*lc).g.sx;
            avail = if sx > (2 as u_int).wrapping_mul(pad) {
                sx.wrapping_sub((2 as u_int).wrapping_mul(pad))
            } else {
                0 as u_int
            };
            if csx > avail {
                csx = if avail > PANE_MINIMUM as u_int {
                    avail
                } else {
                    PANE_MINIMUM as u_int
                };
            }
            csy = (*lc).g.sy;
            avail = if sy > (2 as u_int).wrapping_mul(pad) {
                sy.wrapping_sub((2 as u_int).wrapping_mul(pad))
            } else {
                0 as u_int
            };
            if csy > avail {
                csy = if avail > PANE_MINIMUM as u_int {
                    avail
                } else {
                    PANE_MINIMUM as u_int
                };
            }
            if csx != (*lc).g.sx || csy != (*lc).g.sy {
                layout_set_size(lc, csx, csy, (*lc).g.xoff, (*lc).g.yoff);
            }
            if ((*lc).g.xoff as u_int)
                .wrapping_add((*lc).g.sx)
                .wrapping_add(pad)
                > sx
            {
                if (*lc).g.sx.wrapping_add((2 as u_int).wrapping_mul(pad)) >= sx {
                    (*lc).g.xoff = pad as ::core::ffi::c_int;
                } else {
                    (*lc).g.xoff =
                        sx.wrapping_sub((*lc).g.sx).wrapping_sub(pad) as ::core::ffi::c_int;
                }
            }
            if ((*lc).g.yoff as u_int)
                .wrapping_add((*lc).g.sy)
                .wrapping_add(pad)
                > sy
            {
                if (*lc).g.sy.wrapping_add((2 as u_int).wrapping_mul(pad)) >= sy {
                    (*lc).g.yoff = pad as ::core::ffi::c_int;
                } else {
                    (*lc).g.yoff =
                        sy.wrapping_sub((*lc).g.sy).wrapping_sub(pad) as ::core::ffi::c_int;
                }
            }
        }
        wp = (*wp).zentry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn layout_resize(mut w: *mut window, mut sx: u_int, mut sy: u_int) {
    let mut lc: *mut layout_cell = (*w).layout_root;
    let mut xlimit: ::core::ffi::c_int = 0;
    let mut ylimit: ::core::ffi::c_int = 0;
    let mut xchange: ::core::ffi::c_int = 0;
    let mut ychange: ::core::ffi::c_int = 0;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*lc).flags & LAYOUT_CELL_FLOATING != 0
    {
        layout_clamp_floating_panes(w, sx, sy);
        layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
        return;
    }
    xchange = sx.wrapping_sub((*lc).g.sx) as ::core::ffi::c_int;
    xlimit = layout_resize_check(w, lc, LAYOUT_LEFTRIGHT) as ::core::ffi::c_int;
    if xchange < 0 as ::core::ffi::c_int && xchange < -xlimit {
        xchange = -xlimit;
    }
    if xlimit == 0 as ::core::ffi::c_int {
        if sx <= (*lc).g.sx {
            xchange = 0 as ::core::ffi::c_int;
        } else {
            xchange = sx.wrapping_sub((*lc).g.sx) as ::core::ffi::c_int;
        }
    }
    if xchange != 0 as ::core::ffi::c_int {
        layout_resize_adjust(w, lc, LAYOUT_LEFTRIGHT, xchange);
    }
    ychange = sy.wrapping_sub((*lc).g.sy) as ::core::ffi::c_int;
    ylimit = layout_resize_check(w, lc, LAYOUT_TOPBOTTOM) as ::core::ffi::c_int;
    if ychange < 0 as ::core::ffi::c_int && ychange < -ylimit {
        ychange = -ylimit;
    }
    if ylimit == 0 as ::core::ffi::c_int {
        if sy <= (*lc).g.sy {
            ychange = 0 as ::core::ffi::c_int;
        } else {
            ychange = sy.wrapping_sub((*lc).g.sy) as ::core::ffi::c_int;
        }
    }
    if ychange != 0 as ::core::ffi::c_int {
        layout_resize_adjust(w, lc, LAYOUT_TOPBOTTOM, ychange);
    }
    layout_fix_offsets(w);
    layout_clamp_floating_panes(w, sx, sy);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
}
#[no_mangle]
pub unsafe extern "C" fn layout_resize_pane_to(
    mut wp: *mut window_pane,
    mut type_0: layout_type,
    mut new_size: u_int,
) {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut change: ::core::ffi::c_int = 0;
    let mut size: ::core::ffi::c_int = 0;
    lc = (*wp).layout_cell as *mut layout_cell;
    lcparent = (*lc).parent;
    while !lcparent.is_null()
        && (*lcparent).type_0 as ::core::ffi::c_uint != type_0 as ::core::ffi::c_uint
    {
        lc = lcparent;
        lcparent = (*lc).parent;
    }
    if lcparent.is_null() {
        return;
    }
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        size = (*lc).g.sx as ::core::ffi::c_int;
    } else {
        size = (*lc).g.sy as ::core::ffi::c_int;
    }
    if layout_cell_is_last_tiled(lc) != 0 {
        change = (size as u_int).wrapping_sub(new_size) as ::core::ffi::c_int;
    } else {
        change = new_size.wrapping_sub(size as u_int) as ::core::ffi::c_int;
    }
    layout_resize_pane(wp, type_0, change, 1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn layout_resize_floating_pane_to(
    mut wp: *mut window_pane,
    mut type_0: layout_type,
    mut size: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    if !(*lc).flags & LAYOUT_CELL_FLOATING != 0 {
        *cause = xstrdup(b"pane is not floating\0" as *const u8 as *const ::core::ffi::c_char);
        return -(1 as ::core::ffi::c_int);
    }
    if window_pane_get_pane_lines(wp) as ::core::ffi::c_uint
        != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        && size >= (PANE_MINIMUM + 2 as ::core::ffi::c_int) as u_int
    {
        size = size.wrapping_sub(2 as u_int);
    }
    if size < PANE_MINIMUM as u_int || size > PANE_MAXIMUM as u_int {
        *cause =
            xstrdup(b"size is too big or too small\0" as *const u8 as *const ::core::ffi::c_char);
        return -(1 as ::core::ffi::c_int);
    }
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*lc).g.sy == size {
            return 0 as ::core::ffi::c_int;
        }
        (*lc).g.sy = size;
    } else {
        if (*lc).g.sx == size {
            return 0 as ::core::ffi::c_int;
        }
        (*lc).g.sx = size;
    }
    redraw_invalidate_scene((*wp).window as *mut window);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn layout_resize_floating_pane(
    mut wp: *mut window_pane,
    mut type_0: layout_type,
    mut change: ::core::ffi::c_int,
    mut opposite: ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    let mut size: u_int = 0;
    if !(*lc).flags & LAYOUT_CELL_FLOATING != 0 {
        *cause = xstrdup(b"pane is not floating\0" as *const u8 as *const ::core::ffi::c_char);
        return -(1 as ::core::ffi::c_int);
    }
    if change == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        size = (*lc).g.sy.wrapping_add(change as u_int);
        if size < PANE_MINIMUM as u_int || size > PANE_MAXIMUM as u_int {
            *cause = xstrdup(
                b"change is too big or too small\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        (*lc).g.sy = size;
        if opposite != 0 {
            (*lc).g.yoff -= change;
        }
    } else {
        size = (*lc).g.sx.wrapping_add(change as u_int);
        if size < PANE_MINIMUM as u_int || size > PANE_MAXIMUM as u_int {
            *cause = xstrdup(
                b"change is too big or too small\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        (*lc).g.sx = size;
        if opposite != 0 {
            (*lc).g.xoff -= change;
        }
    }
    redraw_invalidate_scene((*wp).window as *mut window);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn layout_resize_layout(
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut change: ::core::ffi::c_int,
    mut opposite: ::core::ffi::c_int,
) {
    let mut needed: ::core::ffi::c_int = 0;
    let mut size: ::core::ffi::c_int = 0;
    needed = change;
    while needed != 0 as ::core::ffi::c_int {
        if change > 0 as ::core::ffi::c_int {
            size = layout_resize_pane_grow(w, lc, type_0, needed, opposite);
            needed -= size;
        } else {
            size = layout_resize_pane_shrink(w, lc, type_0, needed);
            needed += size;
        }
        if size == 0 as ::core::ffi::c_int {
            break;
        }
    }
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
}
#[no_mangle]
pub unsafe extern "C" fn layout_resize_pane(
    mut wp: *mut window_pane,
    mut type_0: layout_type,
    mut change: ::core::ffi::c_int,
    mut opposite: ::core::ffi::c_int,
) {
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    lcparent = (*lc).parent;
    while !lcparent.is_null()
        && (*lcparent).type_0 as ::core::ffi::c_uint != type_0 as ::core::ffi::c_uint
    {
        lc = lcparent;
        lcparent = (*lc).parent;
    }
    if lcparent.is_null() {
        return;
    }
    if layout_cell_is_last_tiled(lc) != 0 {
        lc = layout_cell_get_neighbour_dir(lc, 0 as ::core::ffi::c_int);
        if lc.is_null() {
            return;
        }
    }
    layout_resize_layout((*wp).window as *mut window, lc, type_0, change, opposite);
}
unsafe extern "C" fn layout_resize_pane_grow(
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut needed: ::core::ffi::c_int,
    mut opposite: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lcadd: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcremove: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut size: u_int = 0 as u_int;
    lcadd = lc;
    lcremove = layout_cell_get_neighbour_dir(lc, 1 as ::core::ffi::c_int);
    while !lcremove.is_null() {
        size = layout_resize_check(w, lcremove, type_0);
        if size > 0 as u_int {
            break;
        }
        lcremove = layout_cell_get_neighbour_dir(lcremove, 1 as ::core::ffi::c_int);
    }
    if opposite != 0 && lcremove.is_null() {
        lcremove = layout_cell_get_neighbour_dir(lc, 0 as ::core::ffi::c_int);
        while !lcremove.is_null() {
            size = layout_resize_check(w, lcremove, type_0);
            if size > 0 as u_int {
                break;
            }
            lcremove = layout_cell_get_neighbour_dir(lcremove, 0 as ::core::ffi::c_int);
        }
    }
    if lcremove.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if size > needed as u_int {
        size = needed as u_int;
    }
    layout_resize_adjust(w, lcadd, type_0, size as ::core::ffi::c_int);
    layout_resize_adjust(
        w,
        lcremove,
        type_0,
        size.wrapping_neg() as ::core::ffi::c_int,
    );
    return size as ::core::ffi::c_int;
}
unsafe extern "C" fn layout_resize_pane_shrink(
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut needed: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lcadd: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcremove: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut size: u_int = 0;
    lcremove = lc;
    loop {
        size = layout_resize_check(w, lcremove, type_0);
        if size != 0 as u_int {
            break;
        }
        lcremove = layout_cell_get_neighbour_dir(lcremove, 0 as ::core::ffi::c_int);
        if lcremove.is_null() {
            break;
        }
    }
    if lcremove.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    lcadd = layout_cell_get_neighbour_dir(lc, 1 as ::core::ffi::c_int);
    if lcadd.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if size > -needed as u_int {
        size = -needed as u_int;
    }
    layout_resize_adjust(w, lcadd, type_0, size as ::core::ffi::c_int);
    layout_resize_adjust(
        w,
        lcremove,
        type_0,
        size.wrapping_neg() as ::core::ffi::c_int,
    );
    return size as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn layout_assign_pane(
    mut lc: *mut layout_cell,
    mut wp: *mut window_pane,
    mut do_not_resize: ::core::ffi::c_int,
) {
    layout_make_leaf(lc, wp);
    if do_not_resize != 0 {
        layout_fix_panes((*wp).window as *mut window, wp);
    } else {
        layout_fix_panes(
            (*wp).window as *mut window,
            ::core::ptr::null_mut::<window_pane>(),
        );
    };
}
unsafe extern "C" fn layout_new_pane_size(
    mut w: *mut window,
    mut previous: u_int,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut size: u_int,
    mut count_left: u_int,
    mut size_left: u_int,
) -> u_int {
    let mut new_size: u_int = 0;
    let mut min: u_int = 0;
    let mut max: u_int = 0;
    let mut available: u_int = 0;
    if count_left == 1 as u_int {
        return size_left;
    }
    available = layout_resize_check(w, lc, type_0);
    min = ((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
        .wrapping_mul(count_left.wrapping_sub(1 as u_int));
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*lc).g.sx.wrapping_sub(available) > min {
            min = (*lc).g.sx.wrapping_sub(available);
        }
        new_size = (*lc).g.sx.wrapping_mul(size).wrapping_div(previous);
    } else {
        if (*lc).g.sy.wrapping_sub(available) > min {
            min = (*lc).g.sy.wrapping_sub(available);
        }
        new_size = (*lc).g.sy.wrapping_mul(size).wrapping_div(previous);
    }
    max = size_left.wrapping_sub(min);
    if new_size > max {
        new_size = max;
    }
    if new_size < PANE_MINIMUM as u_int {
        new_size = PANE_MINIMUM as u_int;
    }
    return new_size;
}
unsafe extern "C" fn layout_set_size_check(
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
    mut size: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut new_size: u_int = 0;
    let mut available: u_int = 0;
    let mut previous: u_int = 0;
    let mut count: u_int = 0;
    let mut idx: u_int = 0;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return (size >= PANE_MINIMUM) as ::core::ffi::c_int;
    }
    available = size as u_int;
    count = 0 as u_int;
    lcchild = (*lc).cells.tqh_first;
    while !lcchild.is_null() {
        count = count.wrapping_add(1);
        lcchild = (*lcchild).entry.tqe_next;
    }
    if (*lc).type_0 as ::core::ffi::c_uint == type_0 as ::core::ffi::c_uint {
        if available < count.wrapping_mul(2 as u_int).wrapping_sub(1 as u_int) {
            return 0 as ::core::ffi::c_int;
        }
        if type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            previous = (*lc).g.sx;
        } else {
            previous = (*lc).g.sy;
        }
        idx = 0 as u_int;
        lcchild = (*lc).cells.tqh_first;
        while !lcchild.is_null() {
            new_size = layout_new_pane_size(
                w,
                previous,
                lcchild,
                type_0,
                size as u_int,
                count.wrapping_sub(idx),
                available,
            );
            if idx == count.wrapping_sub(1 as u_int) {
                if new_size > available {
                    return 0 as ::core::ffi::c_int;
                }
                available = available.wrapping_sub(new_size);
            } else {
                if new_size.wrapping_add(1 as u_int) > available {
                    return 0 as ::core::ffi::c_int;
                }
                available = available.wrapping_sub(new_size.wrapping_add(1 as u_int));
            }
            if layout_set_size_check(w, lcchild, type_0, new_size as ::core::ffi::c_int) == 0 {
                return 0 as ::core::ffi::c_int;
            }
            idx = idx.wrapping_add(1);
            lcchild = (*lcchild).entry.tqe_next;
        }
    } else {
        lcchild = (*lc).cells.tqh_first;
        while !lcchild.is_null() {
            if !((*lcchild).type_0 as ::core::ffi::c_uint
                == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                if layout_set_size_check(w, lcchild, type_0, size) == 0 {
                    return 0 as ::core::ffi::c_int;
                }
            }
            lcchild = (*lcchild).entry.tqe_next;
        }
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn layout_resize_child_cells(mut w: *mut window, mut lc: *mut layout_cell) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut prev: u_int = 0;
    let mut available: u_int = 0;
    let mut count: u_int = 0;
    let mut idx: u_int = 0;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    count = 0 as u_int;
    prev = 0 as u_int;
    lcchild = (*lc).cells.tqh_first;
    while !lcchild.is_null() {
        if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
            count = count.wrapping_add(1);
            if (*lc).type_0 as ::core::ffi::c_uint
                == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                prev = prev.wrapping_add((*lcchild).g.sx);
            } else if (*lc).type_0 as ::core::ffi::c_uint
                == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                prev = prev.wrapping_add((*lcchild).g.sy);
            }
        }
        lcchild = (*lcchild).entry.tqe_next;
    }
    prev = prev.wrapping_add(count.wrapping_sub(1 as u_int));
    available = 0 as u_int;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        available = (*lc).g.sx;
    } else if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        available = (*lc).g.sy;
    }
    idx = 0 as u_int;
    lcchild = (*lc).cells.tqh_first;
    while !lcchild.is_null() {
        if !(layout_cell_is_tiled(lcchild) == 0 && layout_cell_has_tiled_child(lcchild) == 0) {
            if (*lc).type_0 as ::core::ffi::c_uint
                == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*lcchild).g.sx = (*lc).g.sx;
                (*lcchild).g.xoff = (*lc).g.xoff;
            } else {
                (*lcchild).g.sx = layout_new_pane_size(
                    w,
                    prev,
                    lcchild,
                    (*lc).type_0,
                    (*lc).g.sx,
                    count.wrapping_sub(idx),
                    available,
                );
                available = available.wrapping_sub((*lcchild).g.sx.wrapping_add(1 as u_int));
            }
            if (*lc).type_0 as ::core::ffi::c_uint
                == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*lcchild).g.sy = (*lc).g.sy;
                (*lcchild).g.yoff = (*lc).g.yoff;
            } else {
                (*lcchild).g.sy = layout_new_pane_size(
                    w,
                    prev,
                    lcchild,
                    (*lc).type_0,
                    (*lc).g.sy,
                    count.wrapping_sub(idx),
                    available,
                );
                available = available.wrapping_sub((*lcchild).g.sy.wrapping_add(1 as u_int));
            }
            layout_resize_child_cells(w, lcchild);
            idx = idx.wrapping_add(1);
        }
        lcchild = (*lcchild).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn layout_replace_with_node(
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
) -> *mut layout_cell {
    let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    lcparent = layout_create_cell((*lc).parent);
    layout_make_node(lcparent, type_0);
    layout_set_size(lcparent, (*lc).g.sx, (*lc).g.sy, (*lc).g.xoff, (*lc).g.yoff);
    if (*lc).parent.is_null() {
        (*w).layout_root = lcparent;
    } else {
        (*lcparent).entry.tqe_next = (*lc).entry.tqe_next;
        if !(*lcparent).entry.tqe_next.is_null() {
            (*(*lcparent).entry.tqe_next).entry.tqe_prev = &raw mut (*lcparent).entry.tqe_next;
        } else {
            (*(*lc).parent).cells.tqh_last = &raw mut (*lcparent).entry.tqe_next;
        }
        (*lcparent).entry.tqe_prev = (*lc).entry.tqe_prev;
        *(*lcparent).entry.tqe_prev = lcparent;
    }
    (*lc).parent = lcparent;
    (*lc).entry.tqe_next = (*lcparent).cells.tqh_first;
    if !(*lc).entry.tqe_next.is_null() {
        (*(*lcparent).cells.tqh_first).entry.tqe_prev = &raw mut (*lc).entry.tqe_next;
    } else {
        (*lcparent).cells.tqh_last = &raw mut (*lc).entry.tqe_next;
    }
    (*lcparent).cells.tqh_first = lc;
    (*lc).entry.tqe_prev = &raw mut (*lcparent).cells.tqh_first;
    return lcparent;
}
#[no_mangle]
pub unsafe extern "C" fn layout_split_check_space(
    mut wp: *mut window_pane,
    mut lc: *mut layout_cell,
    mut type_0: layout_type,
) -> ::core::ffi::c_int {
    let mut root: *mut layout_cell = (*(*wp).window).layout_root;
    let mut sb_style: *mut style = &raw mut (*wp).scrollbar_style;
    let mut minimum: u_int = 0;
    let mut sx: u_int = (*lc).g.sx;
    let mut sy: u_int = (*lc).g.sy;
    let mut status: ::core::ffi::c_int = 0;
    if (*lc).flags & LAYOUT_CELL_FLOATING != 0 {
        fatalx(b"floating cells cannot be split\0" as *const u8 as *const ::core::ffi::c_char);
    }
    status = window_get_pane_status((*wp).window as *mut window);
    match type_0 as ::core::ffi::c_uint {
        0 => {
            if (*(*wp).window).sb == PANE_SCROLLBARS_ALWAYS {
                minimum = (PANE_MINIMUM * 2 as ::core::ffi::c_int
                    + (*sb_style).width
                    + (*sb_style).pad) as u_int;
            } else {
                minimum =
                    (PANE_MINIMUM * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as u_int;
            }
            if sx < minimum {
                return 0 as ::core::ffi::c_int;
            }
        }
        1 => {
            if layout_add_horizontal_border(root, lc, status) != 0 {
                minimum =
                    (PANE_MINIMUM * 2 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as u_int;
            } else {
                minimum =
                    (PANE_MINIMUM * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as u_int;
            }
            if sy < minimum {
                return 0 as ::core::ffi::c_int;
            }
        }
        _ => {
            fatalx(b"bad layout type\0" as *const u8 as *const ::core::ffi::c_char);
        }
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn layout_split_sizes(
    mut lc: *mut layout_cell,
    mut size: ::core::ffi::c_int,
    mut before: ::core::ffi::c_int,
    mut type_0: layout_type,
    mut size1: *mut u_int,
    mut size2: *mut u_int,
    mut saved_size: *mut u_int,
) {
    let mut s1: u_int = 0;
    let mut s2: u_int = 0;
    let mut ss: u_int = 0;
    let mut sx: u_int = (*lc).g.sx;
    let mut sy: u_int = (*lc).g.sy;
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        ss = sx;
    } else {
        ss = sy;
    }
    if size < 0 as ::core::ffi::c_int {
        s2 = ss
            .wrapping_add(1 as u_int)
            .wrapping_div(2 as u_int)
            .wrapping_sub(1 as u_int);
    } else if before != 0 {
        s2 = ss.wrapping_sub(size as u_int).wrapping_sub(1 as u_int);
    } else {
        s2 = size as u_int;
    }
    if s2 < PANE_MINIMUM as u_int {
        s2 = PANE_MINIMUM as u_int;
    } else if s2 > ss.wrapping_sub(2 as u_int) {
        s2 = ss.wrapping_sub(2 as u_int);
    }
    s1 = ss.wrapping_sub(1 as u_int).wrapping_sub(s2);
    *size1 = s1;
    *size2 = s2;
    *saved_size = ss;
}
#[no_mangle]
pub unsafe extern "C" fn layout_split_pane(
    mut wp: *mut window_pane,
    mut type_0: layout_type,
    mut size: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> *mut layout_cell {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcnew: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lc1: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lc2: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xoff: u_int = 0;
    let mut yoff: u_int = 0;
    let mut size1: u_int = 0;
    let mut size2: u_int = 0;
    let mut new_size: u_int = 0;
    let mut saved_size: u_int = 0;
    let mut resize_first: u_int = 0 as u_int;
    let mut full_size: ::core::ffi::c_int = flags & SPAWN_FULLSIZE;
    let mut before: ::core::ffi::c_int = flags & SPAWN_BEFORE;
    if full_size != 0 {
        lc = (*(*wp).window).layout_root;
    } else {
        lc = (*wp).layout_cell as *mut layout_cell;
    }
    sx = (*lc).g.sx;
    sy = (*lc).g.sy;
    xoff = (*lc).g.xoff as u_int;
    yoff = (*lc).g.yoff as u_int;
    if layout_split_check_space(wp, lc, type_0) == 0 {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    layout_split_sizes(
        lc,
        size,
        before,
        type_0,
        &raw mut size1,
        &raw mut size2,
        &raw mut saved_size,
    );
    if flags & SPAWN_BEFORE != 0 {
        new_size = size2;
    } else {
        new_size = size1;
    }
    if full_size != 0
        && layout_set_size_check(
            (*wp).window as *mut window,
            lc,
            type_0,
            new_size as ::core::ffi::c_int,
        ) == 0
    {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    if !(*lc).parent.is_null()
        && (*(*lc).parent).type_0 as ::core::ffi::c_uint == type_0 as ::core::ffi::c_uint
    {
        lcparent = (*lc).parent;
        lcnew = layout_create_cell(lcparent);
        if flags & SPAWN_BEFORE != 0 {
            (*lcnew).entry.tqe_prev = (*lc).entry.tqe_prev;
            (*lcnew).entry.tqe_next = lc;
            *(*lc).entry.tqe_prev = lcnew;
            (*lc).entry.tqe_prev = &raw mut (*lcnew).entry.tqe_next;
        } else {
            (*lcnew).entry.tqe_next = (*lc).entry.tqe_next;
            if !(*lcnew).entry.tqe_next.is_null() {
                (*(*lcnew).entry.tqe_next).entry.tqe_prev = &raw mut (*lcnew).entry.tqe_next;
            } else {
                (*lcparent).cells.tqh_last = &raw mut (*lcnew).entry.tqe_next;
            }
            (*lc).entry.tqe_next = lcnew;
            (*lcnew).entry.tqe_prev = &raw mut (*lc).entry.tqe_next;
        }
    } else if full_size != 0
        && (*lc).parent.is_null()
        && (*lc).type_0 as ::core::ffi::c_uint == type_0 as ::core::ffi::c_uint
    {
        if (*lc).type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*lc).g.sx = new_size;
            layout_resize_child_cells((*wp).window as *mut window, lc);
            (*lc).g.sx = saved_size;
        } else if (*lc).type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*lc).g.sy = new_size;
            layout_resize_child_cells((*wp).window as *mut window, lc);
            (*lc).g.sy = saved_size;
        }
        resize_first = 1 as u_int;
        lcnew = layout_create_cell(lc);
        size = saved_size.wrapping_sub(1 as u_int).wrapping_sub(new_size) as ::core::ffi::c_int;
        if (*lc).type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            layout_set_size(
                lcnew,
                size as u_int,
                sy,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        } else if (*lc).type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            layout_set_size(
                lcnew,
                sx,
                size as u_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
        }
        if flags & SPAWN_BEFORE != 0 {
            (*lcnew).entry.tqe_next = (*lc).cells.tqh_first;
            if !(*lcnew).entry.tqe_next.is_null() {
                (*(*lc).cells.tqh_first).entry.tqe_prev = &raw mut (*lcnew).entry.tqe_next;
            } else {
                (*lc).cells.tqh_last = &raw mut (*lcnew).entry.tqe_next;
            }
            (*lc).cells.tqh_first = lcnew;
            (*lcnew).entry.tqe_prev = &raw mut (*lc).cells.tqh_first;
        } else {
            (*lcnew).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
            (*lcnew).entry.tqe_prev = (*lc).cells.tqh_last;
            *(*lc).cells.tqh_last = lcnew;
            (*lc).cells.tqh_last = &raw mut (*lcnew).entry.tqe_next;
        }
    } else {
        lcparent = layout_replace_with_node((*wp).window as *mut window, lc, type_0);
        lcnew = layout_create_cell(lcparent);
        if flags & SPAWN_BEFORE != 0 {
            (*lcnew).entry.tqe_next = (*lcparent).cells.tqh_first;
            if !(*lcnew).entry.tqe_next.is_null() {
                (*(*lcparent).cells.tqh_first).entry.tqe_prev = &raw mut (*lcnew).entry.tqe_next;
            } else {
                (*lcparent).cells.tqh_last = &raw mut (*lcnew).entry.tqe_next;
            }
            (*lcparent).cells.tqh_first = lcnew;
            (*lcnew).entry.tqe_prev = &raw mut (*lcparent).cells.tqh_first;
        } else {
            (*lcnew).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
            (*lcnew).entry.tqe_prev = (*lcparent).cells.tqh_last;
            *(*lcparent).cells.tqh_last = lcnew;
            (*lcparent).cells.tqh_last = &raw mut (*lcnew).entry.tqe_next;
        }
    }
    if flags & SPAWN_BEFORE != 0 {
        lc1 = lcnew;
        lc2 = lc;
    } else {
        lc1 = lc;
        lc2 = lcnew;
    }
    if resize_first == 0
        && type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        layout_set_size(
            lc1,
            size1,
            sy,
            xoff as ::core::ffi::c_int,
            yoff as ::core::ffi::c_int,
        );
        layout_set_size(
            lc2,
            size2,
            sy,
            xoff.wrapping_add((*lc1).g.sx).wrapping_add(1 as u_int) as ::core::ffi::c_int,
            yoff as ::core::ffi::c_int,
        );
    } else if resize_first == 0
        && type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        layout_set_size(
            lc1,
            sx,
            size1,
            xoff as ::core::ffi::c_int,
            yoff as ::core::ffi::c_int,
        );
        layout_set_size(
            lc2,
            sx,
            size2,
            xoff as ::core::ffi::c_int,
            yoff.wrapping_add((*lc1).g.sy).wrapping_add(1 as u_int) as ::core::ffi::c_int,
        );
    }
    if full_size != 0 {
        if resize_first == 0 {
            layout_resize_child_cells((*wp).window as *mut window, lc);
        }
        layout_fix_offsets((*wp).window as *mut window);
    } else {
        layout_make_leaf(lc, wp);
    }
    return lcnew;
}
#[no_mangle]
pub unsafe extern "C" fn layout_floating_pane(
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut lg: *mut layout_geometry,
) -> *mut layout_cell {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcnew: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if wp.is_null() {
        lc = (*w).layout_root;
    } else {
        lc = (*wp).layout_cell as *mut layout_cell;
    }
    lcparent = (*lc).parent;
    if lcparent.is_null() {
        lcparent = layout_replace_with_node(w, lc, LAYOUT_TOPBOTTOM);
    }
    lcnew = layout_create_cell(lcparent);
    (*lcnew).entry.tqe_next = (*lc).entry.tqe_next;
    if !(*lcnew).entry.tqe_next.is_null() {
        (*(*lcnew).entry.tqe_next).entry.tqe_prev = &raw mut (*lcnew).entry.tqe_next;
    } else {
        (*lcparent).cells.tqh_last = &raw mut (*lcnew).entry.tqe_next;
    }
    (*lc).entry.tqe_next = lcnew;
    (*lcnew).entry.tqe_prev = &raw mut (*lc).entry.tqe_next;
    (*lcnew).flags |= LAYOUT_CELL_FLOATING;
    layout_set_size(lcnew, (*lg).sx, (*lg).sy, (*lg).xoff, (*lg).yoff);
    return lcnew;
}
#[no_mangle]
pub unsafe extern "C" fn layout_close_pane(mut wp: *mut window_pane) {
    let mut w: *mut window = (*wp).window as *mut window;
    if (*wp).layout_cell.is_null() {
        return;
    }
    layout_destroy_cell(
        w,
        (*wp).layout_cell as *mut layout_cell,
        &raw mut (*w).layout_root,
    );
    (*wp).layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if !(*w).layout_root.is_null() {
        layout_fix_offsets(w);
        layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    }
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
}
#[no_mangle]
pub unsafe extern "C" fn layout_spread_cell(
    mut w: *mut window,
    mut parent: *mut layout_cell,
) -> ::core::ffi::c_int {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut root: *mut layout_cell = (*w).layout_root;
    let mut number: u_int = 0;
    let mut each: u_int = 0;
    let mut size: u_int = 0;
    let mut this: u_int = 0;
    let mut remainder: u_int = 0;
    let mut change: ::core::ffi::c_int = 0;
    let mut changed: ::core::ffi::c_int = 0;
    let mut status: ::core::ffi::c_int = 0;
    number = 0 as u_int;
    lc = (*parent).cells.tqh_first;
    while !lc.is_null() {
        if layout_cell_is_tiled(lc) != 0 {
            number = number.wrapping_add(1);
        }
        lc = (*lc).entry.tqe_next;
    }
    if number <= 1 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    status = window_get_pane_status(w);
    if (*parent).type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        size = (*parent).g.sx;
    } else if (*parent).type_0 as ::core::ffi::c_uint
        == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if layout_add_horizontal_border(root, parent, status) != 0 {
            size = (*parent).g.sy.wrapping_sub(1 as u_int);
        } else {
            size = (*parent).g.sy;
        }
    } else {
        return 0 as ::core::ffi::c_int;
    }
    if size < number.wrapping_sub(1 as u_int) {
        return 0 as ::core::ffi::c_int;
    }
    each = size
        .wrapping_sub(number.wrapping_sub(1 as u_int))
        .wrapping_div(number);
    if each == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    remainder = size
        .wrapping_sub(number.wrapping_mul(each.wrapping_add(1 as u_int)))
        .wrapping_add(1 as u_int);
    changed = 0 as ::core::ffi::c_int;
    lc = (*parent).cells.tqh_first;
    while !lc.is_null() {
        if !(layout_cell_is_tiled(lc) == 0) {
            change = 0 as ::core::ffi::c_int;
            if (*parent).type_0 as ::core::ffi::c_uint
                == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                change = each.wrapping_sub((*lc).g.sx as ::core::ffi::c_int as u_int)
                    as ::core::ffi::c_int;
                if remainder > 0 as u_int {
                    change += 1;
                    remainder = remainder.wrapping_sub(1);
                }
                layout_resize_adjust(w, lc, LAYOUT_LEFTRIGHT, change);
            } else if (*parent).type_0 as ::core::ffi::c_uint
                == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if layout_add_horizontal_border(root, lc, status) != 0 {
                    this = each.wrapping_add(1 as u_int);
                } else {
                    this = each;
                }
                if remainder > 0 as u_int {
                    this = this.wrapping_add(1);
                    remainder = remainder.wrapping_sub(1);
                }
                change = this.wrapping_sub((*lc).g.sy as ::core::ffi::c_int as u_int)
                    as ::core::ffi::c_int;
                layout_resize_adjust(w, lc, LAYOUT_TOPBOTTOM, change);
            }
            if change != 0 as ::core::ffi::c_int {
                changed = 1 as ::core::ffi::c_int;
            }
        }
        lc = (*lc).entry.tqe_next;
    }
    return changed;
}
#[no_mangle]
pub unsafe extern "C" fn layout_spread_out(mut wp: *mut window_pane) {
    let mut parent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut w: *mut window = (*wp).window as *mut window;
    parent = (*(*wp).layout_cell).parent;
    if parent.is_null() {
        return;
    }
    loop {
        if layout_spread_cell(w, parent) != 0 {
            layout_fix_offsets(w);
            layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
            break;
        } else {
            parent = (*parent).parent;
            if parent.is_null() {
                break;
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn layout_get_tiled_cell(
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut layout_cell {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut type_0: layout_type = LAYOUT_TOPBOTTOM;
    let mut curval: u_int = 0;
    let mut size: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if window_pane_is_floating(wp) != 0 {
        *cause =
            xstrdup(b"can't split a floating pane\0" as *const u8 as *const ::core::ffi::c_char);
        return ::core::ptr::null_mut::<layout_cell>();
    }
    if flags & SPAWN_HORIZONTAL != 0 {
        type_0 = LAYOUT_LEFTRIGHT;
    }
    if args_has(args, 'l' as i32 as u_char) != 0 || args_has(args, 'p' as i32 as u_char) != 0 {
        if flags & SPAWN_FULLSIZE != 0 {
            if type_0 as ::core::ffi::c_uint
                == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                curval = (*w).sy;
            } else {
                curval = (*w).sx;
            }
        } else if type_0 as ::core::ffi::c_uint
            == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            curval = (*wp).sy;
        } else {
            curval = (*wp).sx;
        }
    }
    if args_has(args, 'l' as i32 as u_char) != 0 {
        size = args_percentage_and_expand(
            args,
            'l' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            INT_MAX as ::core::ffi::c_longlong,
            curval as ::core::ffi::c_longlong,
            item,
            &raw mut error,
        ) as ::core::ffi::c_int;
    } else if args_has(args, 'p' as i32 as u_char) != 0 {
        size = args_strtonum_and_expand(
            args,
            'p' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            100 as ::core::ffi::c_longlong,
            item,
            &raw mut error,
        ) as ::core::ffi::c_int;
        if error.is_null() {
            size = curval
                .wrapping_mul(size as u_int)
                .wrapping_div(100 as u_int) as ::core::ffi::c_int;
        }
    }
    if !error.is_null() {
        xasprintf(
            cause,
            b"invalid tiled geometry %s\0" as *const u8 as *const ::core::ffi::c_char,
            error,
        );
        free(error as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<layout_cell>();
    }
    if window_active_pane_is_over_zoom(w) != 0 {
        window_push_zoom(w, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    } else {
        window_push_zoom(w, 1 as ::core::ffi::c_int, flags & SPAWN_ZOOM);
    }
    lc = layout_split_pane(wp, type_0, size, flags);
    if lc.is_null() {
        *cause = xstrdup(b"no space for a new pane\0" as *const u8 as *const ::core::ffi::c_char);
    }
    return lc;
}
#[no_mangle]
pub unsafe extern "C" fn layout_get_floating_cell(
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut lines: pane_lines,
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut flags: ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut layout_cell {
    let mut lcnew: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    let mut fg: layout_geometry = layout_geometry {
        sx: 0,
        sy: 0,
        xoff: 0,
        yoff: 0,
    };
    layout_geometry_init(&raw mut fg);
    if flags & SPAWN_SPLIT != 0 {
        if layout_split_floating_cell(lc, w, &raw mut fg, lines, flags, cause)
            != 0 as ::core::ffi::c_int
        {
            return ::core::ptr::null_mut::<layout_cell>();
        }
    } else if layout_floating_args_parse(item, args, lines, w, &raw mut fg, cause)
        != 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    if flags & SPAWN_FLOATOVERZOOM != 0 {
        window_push_zoom(
            (*wp).window as *mut window,
            0 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
    } else if window_active_pane_is_over_zoom(w) != 0 {
        window_push_zoom(
            (*wp).window as *mut window,
            0 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
    } else {
        window_push_zoom(
            (*wp).window as *mut window,
            1 as ::core::ffi::c_int,
            flags & SPAWN_ZOOM,
        );
    }
    lcnew = layout_floating_pane(w, wp, &raw mut fg);
    return lcnew;
}
#[no_mangle]
pub unsafe extern "C" fn layout_floating_args_parse(
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut lines: pane_lines,
    mut w: *mut window,
    mut lg: *mut layout_geometry,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut sx: ::core::ffi::c_int = 0;
    let mut sy: ::core::ffi::c_int = 0;
    let mut ox: ::core::ffi::c_int = 0;
    let mut oy: ::core::ffi::c_int = 0;
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    sx = (if (*lg).sx == UINT_MAX {
        (*w).sx.wrapping_div(2 as u_int)
    } else {
        (*lg).sx
    }) as ::core::ffi::c_int;
    sy = (if (*lg).sy == UINT_MAX {
        (*w).sy.wrapping_div(4 as u_int)
    } else {
        (*lg).sy
    }) as ::core::ffi::c_int;
    ox = (*lg).xoff;
    oy = (*lg).yoff;
    if args_has(args, 'x' as i32 as u_char) != 0 {
        sx = args_percentage_and_expand(
            args,
            'x' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            PANE_MAXIMUM as ::core::ffi::c_longlong,
            (*w).sx as ::core::ffi::c_longlong,
            item,
            &raw mut error,
        ) as ::core::ffi::c_int;
        if !error.is_null() {
            xasprintf(
                cause,
                b"position %s\0" as *const u8 as *const ::core::ffi::c_char,
                error,
            );
            free(error as *mut ::core::ffi::c_void);
            return -(1 as ::core::ffi::c_int);
        }
        if lines as ::core::ffi::c_uint
            != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            sx -= 2 as ::core::ffi::c_int;
        }
    }
    if args_has(args, 'y' as i32 as u_char) != 0 {
        sy = args_percentage_and_expand(
            args,
            'y' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            PANE_MAXIMUM as ::core::ffi::c_longlong,
            (*w).sy as ::core::ffi::c_longlong,
            item,
            &raw mut error,
        ) as ::core::ffi::c_int;
        if !error.is_null() {
            xasprintf(
                cause,
                b"position %s\0" as *const u8 as *const ::core::ffi::c_char,
                error,
            );
            free(error as *mut ::core::ffi::c_void);
            return -(1 as ::core::ffi::c_int);
        }
        if lines as ::core::ffi::c_uint
            != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            sy -= 2 as ::core::ffi::c_int;
        }
    }
    if args_has(args, 'X' as i32 as u_char) != 0 {
        ox = args_percentage_and_expand(
            args,
            'X' as i32 as u_char,
            -sx as ::core::ffi::c_longlong,
            (*w).sx as ::core::ffi::c_longlong,
            (*w).sx as ::core::ffi::c_longlong,
            item,
            &raw mut error,
        ) as ::core::ffi::c_int;
        if !error.is_null() {
            xasprintf(
                cause,
                b"position %s\0" as *const u8 as *const ::core::ffi::c_char,
                error,
            );
            free(error as *mut ::core::ffi::c_void);
            return -(1 as ::core::ffi::c_int);
        }
    }
    if args_has(args, 'Y' as i32 as u_char) != 0 {
        oy = args_percentage_and_expand(
            args,
            'Y' as i32 as u_char,
            -sy as ::core::ffi::c_longlong,
            (*w).sy as ::core::ffi::c_longlong,
            (*w).sy as ::core::ffi::c_longlong,
            item,
            &raw mut error,
        ) as ::core::ffi::c_int;
        if !error.is_null() {
            xasprintf(
                cause,
                b"position %s\0" as *const u8 as *const ::core::ffi::c_char,
                error,
            );
            free(error as *mut ::core::ffi::c_void);
            return -(1 as ::core::ffi::c_int);
        }
    }
    if ox == INT_MAX {
        if (*w).last_new_pane_x == 0 as u_int {
            ox = 4 as ::core::ffi::c_int;
        } else {
            ox = (*w).last_new_pane_x.wrapping_add(4 as u_int) as ::core::ffi::c_int;
            if (*w).last_new_pane_x > (*w).sx {
                ox = 4 as ::core::ffi::c_int;
            }
        }
        (*w).last_new_pane_x = ox as u_int;
    } else if args_has(args, 'X' as i32 as u_char) != 0 {
        if lines as ::core::ffi::c_uint
            != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ox += 1 as ::core::ffi::c_int;
        }
    }
    if oy == INT_MAX {
        if (*w).last_new_pane_y == 0 as u_int {
            oy = 2 as ::core::ffi::c_int;
        } else {
            oy = (*w).last_new_pane_y.wrapping_add(2 as u_int) as ::core::ffi::c_int;
            if (*w).last_new_pane_y > (*w).sy {
                oy = 2 as ::core::ffi::c_int;
            }
        }
        (*w).last_new_pane_y = oy as u_int;
    } else if args_has(args, 'Y' as i32 as u_char) != 0 {
        if lines as ::core::ffi::c_uint
            != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            oy += 1 as ::core::ffi::c_int;
        }
    }
    if sx < PANE_MINIMUM || sx > PANE_MAXIMUM {
        *cause = xstrdup(b"invalid width\0" as *const u8 as *const ::core::ffi::c_char);
        return -(1 as ::core::ffi::c_int);
    }
    if sy < PANE_MINIMUM || sy > PANE_MAXIMUM {
        *cause = xstrdup(b"invalid height\0" as *const u8 as *const ::core::ffi::c_char);
        return -(1 as ::core::ffi::c_int);
    }
    (*lg).sx = sx as u_int;
    (*lg).sy = sy as u_int;
    (*lg).xoff = ox;
    (*lg).yoff = oy;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn layout_split_floating_cell(
    mut lc: *mut layout_cell,
    mut w: *mut window,
    mut out: *mut layout_geometry,
    mut lines: pane_lines,
    mut flags: ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut old: layout_geometry = layout_geometry {
        sx: 0,
        sy: 0,
        xoff: 0,
        yoff: 0,
    };
    let mut new: layout_geometry = layout_geometry {
        sx: 0,
        sy: 0,
        xoff: 0,
        yoff: 0,
    };
    let mut tborder: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut bborder: ::core::ffi::c_int = (*w).sy.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    let mut lborder: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
    let mut rborder: ::core::ffi::c_int = (*w).sx.wrapping_sub(3 as u_int) as ::core::ffi::c_int;
    let mut border: ::core::ffi::c_int = if lines as ::core::ffi::c_uint
        != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
    let mut size: ::core::ffi::c_int = 0;
    let mut space: ::core::ffi::c_int = 0;
    memcpy(
        &raw mut old as *mut ::core::ffi::c_void,
        &raw mut (*lc).g as *const ::core::ffi::c_void,
        ::core::mem::size_of::<layout_geometry>() as size_t,
    );
    if lborder > old.xoff - border {
        old.xoff = lborder + border;
    }
    if rborder < old.xoff + old.sx as ::core::ffi::c_int + border {
        old.xoff = rborder - old.sx as ::core::ffi::c_int - border;
    }
    if tborder > old.yoff - border {
        old.yoff = tborder + border;
    }
    if bborder < old.yoff + old.sy as ::core::ffi::c_int + border {
        old.yoff = bborder - old.sy as ::core::ffi::c_int - border;
    }
    memcpy(
        &raw mut new as *mut ::core::ffi::c_void,
        &raw mut old as *const ::core::ffi::c_void,
        ::core::mem::size_of::<layout_geometry>() as size_t,
    );
    if flags & SPAWN_HORIZONTAL != 0 {
        if flags & SPAWN_BEFORE != 0 {
            new.xoff = (new.xoff as u_int).wrapping_sub(
                old.sx
                    .wrapping_add((2 as ::core::ffi::c_int * border) as u_int),
            ) as ::core::ffi::c_int as ::core::ffi::c_int;
        } else {
            new.xoff = (new.xoff as u_int).wrapping_add(
                old.sx
                    .wrapping_add((2 as ::core::ffi::c_int * border) as u_int),
            ) as ::core::ffi::c_int as ::core::ffi::c_int;
        }
    } else if flags & SPAWN_BEFORE != 0 {
        new.yoff = (new.yoff as u_int).wrapping_sub(
            old.sy
                .wrapping_add((2 as ::core::ffi::c_int * border) as u_int),
        ) as ::core::ffi::c_int as ::core::ffi::c_int;
    } else {
        new.yoff = (new.yoff as u_int).wrapping_add(
            old.sy
                .wrapping_add((2 as ::core::ffi::c_int * border) as u_int),
        ) as ::core::ffi::c_int as ::core::ffi::c_int;
    }
    if lborder > new.xoff - border {
        space = (old.xoff as u_int)
            .wrapping_add(old.sx)
            .wrapping_sub(lborder as u_int)
            .wrapping_sub((3 as ::core::ffi::c_int * border) as u_int)
            .wrapping_add(1 as u_int) as ::core::ffi::c_int;
        size = space / 2 as ::core::ffi::c_int;
        new.sx = size as u_int;
        old.sx = size as u_int;
        new.xoff = lborder + border;
        old.xoff = (new.xoff as u_int)
            .wrapping_add(new.sx)
            .wrapping_add((2 as ::core::ffi::c_int * border) as u_int)
            as ::core::ffi::c_int;
        if space % 2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            old.sx = old.sx.wrapping_sub(1 as u_int);
        }
    } else if rborder < new.xoff + new.sx as ::core::ffi::c_int + border {
        space = rborder - old.xoff - 3 as ::core::ffi::c_int * border + 1 as ::core::ffi::c_int;
        size = space / 2 as ::core::ffi::c_int;
        new.sx = size as u_int;
        old.sx = size as u_int;
        new.xoff = (old.xoff as u_int)
            .wrapping_add(old.sx)
            .wrapping_add((2 as ::core::ffi::c_int * border) as u_int)
            as ::core::ffi::c_int;
        if space % 2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            new.sx = new.sx.wrapping_sub(1 as u_int);
        }
    } else if tborder > new.yoff - border {
        space = old
            .sy
            .wrapping_add(old.yoff as u_int)
            .wrapping_sub(tborder as u_int)
            .wrapping_sub((3 as ::core::ffi::c_int * border) as u_int)
            .wrapping_add(1 as u_int) as ::core::ffi::c_int;
        size = space / 2 as ::core::ffi::c_int;
        new.sy = size as u_int;
        old.sy = size as u_int;
        new.yoff = tborder + border;
        old.yoff = (new.yoff as u_int)
            .wrapping_add(new.sy)
            .wrapping_add((2 as ::core::ffi::c_int * border) as u_int)
            as ::core::ffi::c_int;
        if space % 2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            old.sy = old.sy.wrapping_sub(1 as u_int);
        }
    } else if bborder < new.yoff + new.sy as ::core::ffi::c_int + border {
        space = bborder - old.yoff - 3 as ::core::ffi::c_int * border + 1 as ::core::ffi::c_int;
        size = space / 2 as ::core::ffi::c_int;
        new.sy = size as u_int;
        old.sy = size as u_int;
        new.yoff = (old.yoff as u_int)
            .wrapping_add(old.sy)
            .wrapping_add((2 as ::core::ffi::c_int * border) as u_int)
            as ::core::ffi::c_int;
        if space % 2 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            new.sy = new.sy.wrapping_sub(1 as u_int);
        }
    }
    if flags & SPAWN_FULLSIZE != 0 {
        if flags & SPAWN_HORIZONTAL != 0 {
            new.yoff = tborder + border;
            new.sy = (bborder - tborder - 2 as ::core::ffi::c_int * border) as u_int;
            if flags & SPAWN_BEFORE != 0 {
                new.xoff = lborder + border;
                new.sx = (old.xoff - new.xoff - 2 as ::core::ffi::c_int * border) as u_int;
            } else {
                new.sx = (rborder - new.xoff - border) as u_int;
            }
        } else {
            new.xoff = lborder + border;
            new.sx = (rborder - lborder - 2 as ::core::ffi::c_int * border) as u_int;
            if flags & SPAWN_BEFORE != 0 {
                new.yoff = tborder + border;
                new.sy = (old.yoff - new.yoff - 2 as ::core::ffi::c_int * border) as u_int;
            } else {
                new.sy = (bborder - new.yoff - border) as u_int;
            }
        }
    }
    if new.sx < PANE_MINIMUM as u_int
        || new.sy < PANE_MINIMUM as u_int
        || old.sx < PANE_MINIMUM as u_int
        || old.sy < PANE_MINIMUM as u_int
    {
        *cause = xstrdup(b"no space for a new pane\0" as *const u8 as *const ::core::ffi::c_char);
        return -(1 as ::core::ffi::c_int);
    }
    layout_set_size(lc, old.sx, old.sy, old.xoff, old.yoff);
    memcpy(
        out as *mut ::core::ffi::c_void,
        &raw mut new as *const ::core::ffi::c_void,
        ::core::mem::size_of::<layout_geometry>() as size_t,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn layout_remove_tile(
    mut w: *mut window,
    mut lc: *mut layout_cell,
) -> ::core::ffi::c_int {
    let mut lcneighbour: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut type_0: layout_type = LAYOUT_LEFTRIGHT;
    let mut change: ::core::ffi::c_int = 0;
    if (*lc).flags & LAYOUT_CELL_FLOATING != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    lcneighbour = layout_cell_get_neighbour(lc);
    if lcneighbour.is_null() {
        if !(*lc).parent.is_null() {
            layout_remove_tile(w, (*lc).parent);
        }
    } else {
        lcparent = (*lcneighbour).parent;
        if !lcparent.is_null() {
            type_0 = (*lcparent).type_0;
            if type_0 as ::core::ffi::c_uint
                == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                change = (*lc).g.sy.wrapping_add(1 as u_int) as ::core::ffi::c_int;
            } else {
                change = (*lc).g.sx.wrapping_add(1 as u_int) as ::core::ffi::c_int;
            }
            layout_resize_adjust(w, lcneighbour, type_0, change);
        }
    }
    if !(*lc).parent.is_null() {
        layout_set_size(
            lc,
            0 as u_int,
            0 as u_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn layout_insert_tile(
    mut w: *mut window,
    mut lc: *mut layout_cell,
) -> ::core::ffi::c_int {
    let mut lcneighbour: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lctiled: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcparent: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut type_0: layout_type = LAYOUT_LEFTRIGHT;
    let mut size1: u_int = 0;
    let mut size2: u_int = 0;
    let mut saved_size: u_int = 0;
    if lc.is_null() {
        fatalx(
            b"layout cell cannot be null when tiling\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if layout_cell_is_tiled(lc) != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    lcparent = (*lc).parent;
    if lcparent.is_null() {
        layout_set_size(
            lc,
            (*w).sx,
            (*w).sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        return 0 as ::core::ffi::c_int;
    }
    type_0 = (*lcparent).type_0;
    lcneighbour = layout_cell_get_neighbour(lc);
    if lcneighbour.is_null() {
        layout_insert_tile(w, lcparent);
        if type_0 as ::core::ffi::c_uint
            == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            size1 = (*lcparent).g.sx;
        } else {
            size1 = (*lcparent).g.sy;
        }
        layout_resize_set_size(w, lc, type_0, size1);
    } else {
        lctiled = layout_cell_get_first_tiled(lcneighbour);
        if layout_split_check_space((*lctiled).wp, lcneighbour, type_0) == 0 {
            return -(1 as ::core::ffi::c_int);
        }
        layout_split_sizes(
            lcneighbour,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
            type_0,
            &raw mut size1,
            &raw mut size2,
            &raw mut saved_size,
        );
        layout_resize_set_size(w, lc, type_0, size1);
        layout_resize_set_size(w, lcneighbour, type_0, size2);
    }
    if (*lcparent).type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        size1 = (*lcparent).g.sy;
        type_0 = LAYOUT_TOPBOTTOM;
    } else {
        size1 = (*lcparent).g.sx;
        type_0 = LAYOUT_LEFTRIGHT;
    }
    layout_resize_set_size(w, lc, type_0, size1);
    return 0 as ::core::ffi::c_int;
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

pub use crate::src::shared::variadic::{__builtin_va_list, __gnuc_va_list, __va_list_tag, va_list};
pub use crate::src::shared::stdio::{
    FILE, _IO_FILE, _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data,
};
pub use crate::src::shared::abi::{__off64_t, __off_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::tree::{RB_NEGINF};
pub use crate::src::shared::command::{CMD_PARSE_PARSEONLY, CMD_PARSE_QUIET};
pub use crate::src::shared::client::{CLIENT_CONTROL};
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
    pub type control_state;
    pub type cmdq_list;
    pub type cmdq_state;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strerror(__errnum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xvasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    fn cmd_list_free(_: *mut cmd_list);
    fn cmd_parse_from_file(_: *mut FILE, _: *mut cmd_parse_input) -> *mut cmd_parse_result;
    fn cmd_parse_from_buffer(
        _: *const ::core::ffi::c_void,
        _: size_t,
        _: *mut cmd_parse_input,
    ) -> *mut cmd_parse_result;
    fn cmdq_new_state(
        _: *mut cmd_find_state,
        _: *mut key_event,
        _: ::core::ffi::c_int,
    ) -> *mut cmdq_state;
    fn cmdq_copy_state(_: *mut cmdq_state, _: *mut cmd_find_state) -> *mut cmdq_state;
    fn cmdq_free_state(_: *mut cmdq_state);
    fn cmdq_add_format(
        _: *mut cmdq_state,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_state(_: *mut cmdq_item) -> *mut cmdq_state;
    fn cmdq_get_command(_: *mut cmd_list, _: *mut cmdq_state) -> *mut cmdq_item;
    fn cmdq_get_callback1(
        _: *const ::core::ffi::c_char,
        _: cmdq_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut cmdq_item;
    fn cmdq_insert_after(_: *mut cmdq_item, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_append(_: *mut client, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_continue(_: *mut cmdq_item);
    fn cmdq_print(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    static mut clients: clients;
    fn prompt_load_history();
    fn window_pane_set_mode(
        _: *mut window_pane,
        _: *mut window_pane,
        _: *const window_mode,
        _: *mut cmdq_item,
        _: *mut cmd_find_state,
        _: *mut args,
    ) -> ::core::ffi::c_int;
    static window_view_mode: window_mode;
    fn window_copy_add(
        _: *mut window_pane,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn control_notify_write(_: *mut client, _: *const ::core::ffi::c_char, ...);
    static mut sessions: sessions;
    fn sessions_RB_MINMAX(_: *mut sessions, _: ::core::ffi::c_int) -> *mut session;
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
pub struct sessions {
    pub rbh_root: *mut session,
}
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
pub type cmdq_cb =
    Option<unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct clients {
    pub tqh_first: *mut client,
    pub tqh_last: *mut *mut client,
}
pub const ENOENT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[no_mangle]
pub static mut cfg_client: *mut client = ::core::ptr::null::<client>() as *mut client;
#[no_mangle]
pub static mut cfg_finished: ::core::ffi::c_int = 0;
static mut cfg_causes: *mut *mut ::core::ffi::c_char =
    ::core::ptr::null::<*mut ::core::ffi::c_char>() as *mut *mut ::core::ffi::c_char;
static mut cfg_ncauses: u_int = 0;
static mut cfg_item: *mut cmdq_item = ::core::ptr::null::<cmdq_item>() as *mut cmdq_item;
#[no_mangle]
pub static mut cfg_quiet: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
#[no_mangle]
pub static mut cfg_files: *mut *mut ::core::ffi::c_char =
    ::core::ptr::null::<*mut ::core::ffi::c_char>() as *mut *mut ::core::ffi::c_char;
#[no_mangle]
pub static mut cfg_nfiles: u_int = 0;
unsafe extern "C" fn cfg_client_done(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    if cfg_finished == 0 {
        return CMD_RETURN_WAIT;
    }
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cfg_done(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    if cfg_finished != 0 {
        return CMD_RETURN_NORMAL;
    }
    cfg_finished = 1 as ::core::ffi::c_int;
    cfg_show_causes(::core::ptr::null_mut::<session>());
    if !cfg_item.is_null() {
        cmdq_continue(cfg_item);
    }
    prompt_load_history();
    return CMD_RETURN_NORMAL;
}
#[no_mangle]
pub unsafe extern "C" fn start_cfg() {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut i: u_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    c = clients.tqh_first;
    cfg_client = c;
    if !c.is_null() {
        cfg_item = cmdq_get_callback1(
            b"cfg_client_done\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                cfg_client_done
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        );
        cmdq_append(c, cfg_item);
    }
    if cfg_quiet != 0 {
        flags = CMD_PARSE_QUIET;
    }
    i = 0 as u_int;
    while i < cfg_nfiles {
        load_cfg(
            *cfg_files.offset(i as isize),
            c,
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<cmd_find_state>(),
            flags,
            ::core::ptr::null_mut::<*mut cmdq_item>(),
        );
        i = i.wrapping_add(1);
    }
    cmdq_append(
        ::core::ptr::null_mut::<client>(),
        cmdq_get_callback1(
            b"cfg_done\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                cfg_done
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ),
    );
}
#[no_mangle]
pub unsafe extern "C" fn load_cfg(
    mut path: *const ::core::ffi::c_char,
    mut c: *mut client,
    mut item: *mut cmdq_item,
    mut current: *mut cmd_find_state,
    mut flags: ::core::ffi::c_int,
    mut new_item: *mut *mut cmdq_item,
) -> ::core::ffi::c_int {
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut pi: cmd_parse_input = cmd_parse_input {
        flags: 0,
        file: ::core::ptr::null::<::core::ffi::c_char>(),
        line: 0,
        item: ::core::ptr::null_mut::<cmdq_item>(),
        c: ::core::ptr::null_mut::<client>(),
        fs: cmd_find_state {
            flags: 0,
            current: ::core::ptr::null_mut::<cmd_find_state>(),
            s: ::core::ptr::null_mut::<session>(),
            wl: ::core::ptr::null_mut::<winlink>(),
            w: ::core::ptr::null_mut::<window>(),
            wp: ::core::ptr::null_mut::<window_pane>(),
            idx: 0,
        },
    };
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut new_item0: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    if !new_item.is_null() {
        *new_item = ::core::ptr::null_mut::<cmdq_item>();
    }
    log_debug(
        b"loading %s\0" as *const u8 as *const ::core::ffi::c_char,
        path,
    );
    f = fopen(path, b"rb\0" as *const u8 as *const ::core::ffi::c_char) as *mut FILE;
    if f.is_null() {
        if *__errno_location() == ENOENT && flags & CMD_PARSE_QUIET != 0 {
            return 0 as ::core::ffi::c_int;
        }
        cfg_add_cause(
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            path,
            strerror(*__errno_location()),
        );
        return -(1 as ::core::ffi::c_int);
    }
    memset(
        &raw mut pi as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_input>() as size_t,
    );
    pi.flags = flags;
    pi.file = path;
    pi.line = 1 as u_int;
    pi.item = item;
    pi.c = c;
    pr = cmd_parse_from_file(f, &raw mut pi);
    fclose(f);
    if (*pr).status as ::core::ffi::c_uint
        == CMD_PARSE_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        cfg_add_cause(
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*pr).error,
        );
        free((*pr).error as *mut ::core::ffi::c_void);
        return -(1 as ::core::ffi::c_int);
    }
    if flags & CMD_PARSE_PARSEONLY != 0 {
        cmd_list_free((*pr).cmdlist);
        return 0 as ::core::ffi::c_int;
    }
    if !item.is_null() {
        state = cmdq_copy_state(cmdq_get_state(item), current);
    } else {
        state = cmdq_new_state(
            ::core::ptr::null_mut::<cmd_find_state>(),
            ::core::ptr::null_mut::<key_event>(),
            0 as ::core::ffi::c_int,
        );
    }
    cmdq_add_format(
        state,
        b"current_file\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        pi.file,
    );
    new_item0 = cmdq_get_command((*pr).cmdlist, state);
    if !item.is_null() {
        new_item0 = cmdq_insert_after(item, new_item0);
    } else {
        new_item0 = cmdq_append(::core::ptr::null_mut::<client>(), new_item0);
    }
    cmd_list_free((*pr).cmdlist);
    cmdq_free_state(state);
    if !new_item.is_null() {
        *new_item = new_item0;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn load_cfg_from_buffer(
    mut buf: *const ::core::ffi::c_void,
    mut len: size_t,
    mut path: *const ::core::ffi::c_char,
    mut c: *mut client,
    mut item: *mut cmdq_item,
    mut current: *mut cmd_find_state,
    mut flags: ::core::ffi::c_int,
    mut new_item: *mut *mut cmdq_item,
) -> ::core::ffi::c_int {
    let mut pi: cmd_parse_input = cmd_parse_input {
        flags: 0,
        file: ::core::ptr::null::<::core::ffi::c_char>(),
        line: 0,
        item: ::core::ptr::null_mut::<cmdq_item>(),
        c: ::core::ptr::null_mut::<client>(),
        fs: cmd_find_state {
            flags: 0,
            current: ::core::ptr::null_mut::<cmd_find_state>(),
            s: ::core::ptr::null_mut::<session>(),
            wl: ::core::ptr::null_mut::<winlink>(),
            w: ::core::ptr::null_mut::<window>(),
            wp: ::core::ptr::null_mut::<window_pane>(),
            idx: 0,
        },
    };
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut new_item0: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    if !new_item.is_null() {
        *new_item = ::core::ptr::null_mut::<cmdq_item>();
    }
    log_debug(
        b"loading %s\0" as *const u8 as *const ::core::ffi::c_char,
        path,
    );
    memset(
        &raw mut pi as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_input>() as size_t,
    );
    pi.flags = flags;
    pi.file = path;
    pi.line = 1 as u_int;
    pi.item = item;
    pi.c = c;
    pr = cmd_parse_from_buffer(buf, len, &raw mut pi);
    if (*pr).status as ::core::ffi::c_uint
        == CMD_PARSE_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        cfg_add_cause(
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*pr).error,
        );
        free((*pr).error as *mut ::core::ffi::c_void);
        return -(1 as ::core::ffi::c_int);
    }
    if flags & CMD_PARSE_PARSEONLY != 0 {
        cmd_list_free((*pr).cmdlist);
        return 0 as ::core::ffi::c_int;
    }
    if !item.is_null() {
        state = cmdq_copy_state(cmdq_get_state(item), current);
    } else {
        state = cmdq_new_state(
            ::core::ptr::null_mut::<cmd_find_state>(),
            ::core::ptr::null_mut::<key_event>(),
            0 as ::core::ffi::c_int,
        );
    }
    cmdq_add_format(
        state,
        b"current_file\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        pi.file,
    );
    new_item0 = cmdq_get_command((*pr).cmdlist, state);
    if !item.is_null() {
        new_item0 = cmdq_insert_after(item, new_item0);
    } else {
        new_item0 = cmdq_append(::core::ptr::null_mut::<client>(), new_item0);
    }
    cmd_list_free((*pr).cmdlist);
    cmdq_free_state(state);
    if !new_item.is_null() {
        *new_item = new_item0;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cfg_add_cause(mut fmt: *const ::core::ffi::c_char, mut args: ...) {
    let mut ap: ::core::ffi::VaList;
    let mut msg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ap = args.clone();
    xvasprintf(&raw mut msg, fmt, ap);
    cfg_ncauses = cfg_ncauses.wrapping_add(1);
    cfg_causes = xreallocarray(
        cfg_causes as *mut ::core::ffi::c_void,
        cfg_ncauses as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
    ) as *mut *mut ::core::ffi::c_char;
    let ref mut fresh0 = *cfg_causes.offset(cfg_ncauses.wrapping_sub(1 as u_int) as isize);
    *fresh0 = msg;
}
#[no_mangle]
pub unsafe extern "C" fn cfg_print_causes(mut item: *mut cmdq_item) {
    let mut c: *mut client = cmdq_get_client(item);
    let mut i: u_int = 0;
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    i = 0 as u_int;
    while i < cfg_ncauses {
        cause = *cfg_causes.offset(i as isize);
        if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_notify_write(
                c,
                b"%%config-error %s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
        } else {
            cmdq_print(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
        }
        free(cause as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    free(cfg_causes as *mut ::core::ffi::c_void);
    cfg_causes = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    cfg_ncauses = 0 as u_int;
}
#[no_mangle]
pub unsafe extern "C" fn cfg_show_causes(mut s: *mut session) {
    let mut c: *mut client = clients.tqh_first;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut i: u_int = 0;
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if cfg_ncauses == 0 as u_int {
        return;
    }
    if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        i = 0 as u_int;
        while i < cfg_ncauses {
            cause = *cfg_causes.offset(i as isize);
            control_notify_write(
                c,
                b"%%config-error %s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
            i = i.wrapping_add(1);
        }
    } else {
        if s.is_null() {
            if !c.is_null() && !(*c).session.is_null() {
                s = (*c).session;
            } else {
                s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
            }
        }
        if s.is_null() || (*s).attached == 0 as u_int {
            return;
        }
        wp = (*(*(*s).curw).window).active;
        wme = (*wp).modes.tqh_first;
        if wme.is_null() || (*wme).mode != &raw const window_view_mode {
            window_pane_set_mode(
                wp,
                ::core::ptr::null_mut::<window_pane>(),
                &raw const window_view_mode,
                ::core::ptr::null_mut::<cmdq_item>(),
                ::core::ptr::null_mut::<cmd_find_state>(),
                ::core::ptr::null_mut::<args>(),
            );
        }
        i = 0 as u_int;
        while i < cfg_ncauses {
            window_copy_add(
                wp,
                0 as ::core::ffi::c_int,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                *cfg_causes.offset(i as isize),
            );
            free(*cfg_causes.offset(i as isize) as *mut ::core::ffi::c_void);
            i = i.wrapping_add(1);
        }
    }
    free(cfg_causes as *mut ::core::ffi::c_void);
    cfg_causes = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    cfg_ncauses = 0 as u_int;
}

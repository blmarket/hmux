pub use crate::src::shared::account::passwd;
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::abi::{__gid_t, __int32_t, __uid_t, uid_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{
    CMDQ_FIRED, CMDQ_STATE_CONTROL, CMDQ_STATE_NOHOOKS, CMDQ_WAITING, CMD_AFTERHOOK,
    CMD_CLIENT_CANFAIL, CMD_CLIENT_CFLAG, CMD_CLIENT_TFLAG,
};
pub use crate::src::shared::client::{CLIENT_CONTROL, CLIENT_UTF8};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::arguments::*;
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
    pub type cmd;
    pub type cmds;
    pub type input_ctx;
    pub type spawn_editor_state;
    pub type input_request;
    pub type redraw_scene;
    pub type tty_key;
    pub type tty_code;
    pub type format_job_tree;
    pub type control_state;
    pub type event_payload;
    pub type args_entry;
    fn __ctype_toupper_loc() -> *mut *const __int32_t;
    fn getpwuid(__uid: __uid_t) -> *mut passwd;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn time(__timer: *mut time_t) -> time_t;
    fn getuid() -> __uid_t;
    fn evbuffer_new() -> *mut evbuffer;
    fn evbuffer_free(buf: *mut evbuffer);
    fn evbuffer_add_vprintf(
        buf: *mut evbuffer,
        fmt: *const ::core::ffi::c_char,
        ap: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
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
    fn xsnprintf(
        _: *mut ::core::ffi::c_char,
        _: size_t,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn proc_get_peer_uid(_: *mut tmuxpeer) -> uid_t;
    static mut cfg_finished: ::core::ffi::c_int;
    fn cfg_add_cause(_: *const ::core::ffi::c_char, ...);
    fn format_create(
        _: *mut client,
        _: *mut cmdq_item,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut format_tree;
    fn format_free(_: *mut format_tree);
    fn format_merge(_: *mut format_tree, _: *mut format_tree);
    fn format_add(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn event_payload_create() -> *mut event_payload;
    fn event_payload_set_target(_: *mut event_payload, _: *mut cmd_find_state);
    fn event_payload_set_string(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn event_payload_set_pointer(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut ::core::ffi::c_void,
        _: event_payload_free_cb,
        _: event_payload_print_cb,
    );
    fn events_fire(_: *const ::core::ffi::c_char, _: *mut event_payload);
    fn args_print(_: *mut args) -> *mut ::core::ffi::c_char;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_first(_: *mut args, _: *mut *mut args_entry) -> u_char;
    fn args_next(_: *mut *mut args_entry) -> u_char;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn args_first_value(_: *mut args, _: u_char) -> *mut args_value;
    fn args_next_value(_: *mut args_value) -> *mut args_value;
    fn cmd_find_target(
        _: *mut cmd_find_state,
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
        _: cmd_find_type,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_find_client(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut client;
    fn cmd_find_clear_state(_: *mut cmd_find_state, _: ::core::ffi::c_int);
    fn cmd_find_valid_state(_: *mut cmd_find_state) -> ::core::ffi::c_int;
    fn cmd_find_copy_state(_: *mut cmd_find_state, _: *mut cmd_find_state);
    fn cmd_find_from_client(
        _: *mut cmd_find_state,
        _: *mut client,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_get_entry(_: *mut cmd) -> *const cmd_entry;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmd_get_group(_: *mut cmd) -> u_int;
    fn cmd_get_source(_: *mut cmd, _: *mut *const ::core::ffi::c_char, _: *mut u_int);
    fn cmd_print(_: *mut cmd) -> *mut ::core::ffi::c_char;
    fn cmd_list_free(_: *mut cmd_list);
    fn cmd_list_first(_: *mut cmd_list) -> *mut cmd;
    fn cmd_list_next(_: *mut cmd) -> *mut cmd;
    fn key_string_lookup_key(_: key_code, _: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn file_error(_: *mut client, _: *const ::core::ffi::c_char, ...);
    fn server_add_message(_: *const ::core::ffi::c_char, ...);
    fn server_client_unref(_: *mut client);
    fn server_client_print(_: *mut client, _: ::core::ffi::c_int, _: *mut evbuffer);
    fn status_message_set(
        _: *mut client,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn control_write(_: *mut client, _: *const ::core::ffi::c_char, ...);
    fn control_write_guard(
        _: *mut client,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_long,
        _: u_int,
        _: ::core::ffi::c_int,
    );
    fn utf8_sanitize(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn log_get_level() -> ::core::ffi::c_int;
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
pub struct cmdq_item {
    pub name: *mut ::core::ffi::c_char,
    pub queue: *mut cmdq_list,
    pub next: *mut cmdq_item,
    pub client: *mut client,
    pub target_client: *mut client,
    pub type_0: cmdq_type,
    pub group: u_int,
    pub number: u_int,
    pub time: time_t,
    pub flags: ::core::ffi::c_int,
    pub state: *mut cmdq_state,
    pub source: cmd_find_state,
    pub target: cmd_find_state,
    pub cmdlist: *mut cmd_list,
    pub cmd: *mut cmd,
    pub cb: cmdq_cb,
    pub data: *mut ::core::ffi::c_void,
    pub entry: C2RustUnnamed_29,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_29 {
    pub tqe_next: *mut cmdq_item,
    pub tqe_prev: *mut *mut cmdq_item,
}
pub type cmdq_cb =
    Option<unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_list {
    pub references: ::core::ffi::c_int,
    pub group: u_int,
    pub list: *mut cmds,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdq_state {
    pub references: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
    pub formats: *mut format_tree,
    pub event: key_event,
    pub current: cmd_find_state,
}
pub type cmdq_type = ::core::ffi::c_uint;
pub const CMDQ_CALLBACK: cmdq_type = 1;
pub const CMDQ_COMMAND: cmdq_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdq_list {
    pub item: *mut cmdq_item,
    pub list: cmdq_item_list,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmdq_item_list {
    pub tqh_first: *mut cmdq_item,
    pub tqh_last: *mut *mut cmdq_item,
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
    pub entry: C2RustUnnamed_32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_32 {
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
    pub entry: C2RustUnnamed_33,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_33 {
    pub rbe_left: *mut key_binding,
    pub rbe_right: *mut key_binding,
    pub rbe_parent: *mut key_binding,
    pub rbe_color: ::core::ffi::c_int,
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
    pub entry: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub le_next: *mut tty_term,
    pub le_prev: *mut *mut tty_term,
}
pub type C2RustUnnamed_36 = ::core::ffi::c_ulong;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_value {
    pub type_0: args_type,
    pub c2rust_unnamed: C2RustUnnamed_38,
    pub cached: *mut ::core::ffi::c_char,
    pub entry: C2RustUnnamed_37,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
    pub tqe_next: *mut args_value,
    pub tqe_prev: *mut *mut args_value,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_38 {
    pub string: *mut ::core::ffi::c_char,
    pub cmdlist: *mut cmd_list,
}
pub type args_parse_cb = Option<
    unsafe extern "C" fn(*mut args, u_int, *mut *mut ::core::ffi::c_char) -> args_parse_type,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_parse {
    pub template: *const ::core::ffi::c_char,
    pub lower: ::core::ffi::c_int,
    pub upper: ::core::ffi::c_int,
    pub cb: args_parse_cb,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_entry_flag {
    pub flag: ::core::ffi::c_char,
    pub type_0: cmd_find_type,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_entry {
    pub name: *const ::core::ffi::c_char,
    pub alias: *const ::core::ffi::c_char,
    pub args: args_parse,
    pub usage: *const ::core::ffi::c_char,
    pub source: cmd_entry_flag,
    pub target: cmd_entry_flag,
    pub flags: ::core::ffi::c_int,
    pub exec: Option<unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval>,
}
pub type event_payload_free_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type event_payload_print_cb =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut evbuffer) -> ()>;
#[inline]
unsafe extern "C" fn toupper(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_toupper_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}
pub const FORMAT_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe extern "C" fn cmdq_name(mut c: *mut client) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 256] = [0; 256];
    if c.is_null() {
        return b"<global>\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if !(*c).name.is_null() {
        xsnprintf(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
            b"<%s>\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
    } else {
        xsnprintf(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
            b"<%p>\0" as *const u8 as *const ::core::ffi::c_char,
            c,
        );
    }
    return &raw mut s as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn cmdq_get(mut c: *mut client) -> *mut cmdq_list {
    static mut global_queue: *mut cmdq_list = ::core::ptr::null::<cmdq_list>() as *mut cmdq_list;
    if c.is_null() {
        if global_queue.is_null() {
            global_queue = cmdq_new();
        }
        return global_queue;
    }
    return (*c).queue;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_new() -> *mut cmdq_list {
    let mut queue: *mut cmdq_list = ::core::ptr::null_mut::<cmdq_list>();
    queue = xcalloc(1 as size_t, ::core::mem::size_of::<cmdq_list>() as size_t) as *mut cmdq_list;
    (*queue).list.tqh_first = ::core::ptr::null_mut::<cmdq_item>();
    (*queue).list.tqh_last = &raw mut (*queue).list.tqh_first;
    return queue;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_free(mut queue: *mut cmdq_list) {
    if !(*queue).list.tqh_first.is_null() {
        fatalx(b"queue not empty\0" as *const u8 as *const ::core::ffi::c_char);
    }
    free(queue as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_name(mut item: *mut cmdq_item) -> *const ::core::ffi::c_char {
    return (*item).name;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_cmd(mut item: *mut cmdq_item) -> *mut cmd {
    return (*item).cmd;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_client(mut item: *mut cmdq_item) -> *mut client {
    return (*item).client;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_target_client(mut item: *mut cmdq_item) -> *mut client {
    return (*item).target_client;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_state(mut item: *mut cmdq_item) -> *mut cmdq_state {
    return (*item).state;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_target(mut item: *mut cmdq_item) -> *mut cmd_find_state {
    return &raw mut (*item).target;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_source(mut item: *mut cmdq_item) -> *mut cmd_find_state {
    return &raw mut (*item).source;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_event(mut item: *mut cmdq_item) -> *mut key_event {
    return &raw mut (*(*item).state).event;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_current(mut item: *mut cmdq_item) -> *mut cmd_find_state {
    return &raw mut (*(*item).state).current;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_flags(mut item: *mut cmdq_item) -> ::core::ffi::c_int {
    return (*(*item).state).flags;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_new_state(
    mut current: *mut cmd_find_state,
    mut event: *mut key_event,
    mut flags: ::core::ffi::c_int,
) -> *mut cmdq_state {
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    state = xcalloc(1 as size_t, ::core::mem::size_of::<cmdq_state>() as size_t) as *mut cmdq_state;
    (*state).references = 1 as ::core::ffi::c_int;
    (*state).flags = flags;
    if !event.is_null() {
        memcpy(
            &raw mut (*state).event as *mut ::core::ffi::c_void,
            event as *const ::core::ffi::c_void,
            ::core::mem::size_of::<key_event>() as size_t,
        );
    } else {
        (*state).event.key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
    }
    if !current.is_null() && cmd_find_valid_state(current) != 0 {
        cmd_find_copy_state(&raw mut (*state).current, current);
    } else {
        cmd_find_clear_state(&raw mut (*state).current, 0 as ::core::ffi::c_int);
    }
    return state;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_link_state(mut state: *mut cmdq_state) -> *mut cmdq_state {
    (*state).references += 1;
    return state;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_copy_state(
    mut state: *mut cmdq_state,
    mut current: *mut cmd_find_state,
) -> *mut cmdq_state {
    if !current.is_null() {
        return cmdq_new_state(current, &raw mut (*state).event, (*state).flags);
    }
    return cmdq_new_state(
        &raw mut (*state).current,
        &raw mut (*state).event,
        (*state).flags,
    );
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_free_state(mut state: *mut cmdq_state) {
    (*state).references -= 1;
    if (*state).references != 0 as ::core::ffi::c_int {
        return;
    }
    if !(*state).formats.is_null() {
        format_free((*state).formats);
    }
    free(state as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_add_format(
    mut state: *mut cmdq_state,
    mut key: *const ::core::ffi::c_char,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ap = args.clone();
    xvasprintf(&raw mut value, fmt, ap);
    if (*state).formats.is_null() {
        (*state).formats = format_create(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<cmdq_item>(),
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        );
    }
    format_add(
        (*state).formats,
        key,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        value,
    );
    free(value as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_add_formats(mut state: *mut cmdq_state, mut ft: *mut format_tree) {
    if (*state).formats.is_null() {
        (*state).formats = format_create(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<cmdq_item>(),
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        );
    }
    format_merge((*state).formats, ft);
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_merge_formats(mut item: *mut cmdq_item, mut ft: *mut format_tree) {
    let mut entry: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
    if !(*item).cmd.is_null() {
        entry = cmd_get_entry((*item).cmd);
        format_add(
            ft,
            b"command\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*entry).name,
        );
    }
    if !(*(*item).state).formats.is_null() {
        format_merge(ft, (*(*item).state).formats);
    }
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_append(
    mut c: *mut client,
    mut item: *mut cmdq_item,
) -> *mut cmdq_item {
    let mut queue: *mut cmdq_list = cmdq_get(c);
    let mut next: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    loop {
        next = (*item).next;
        (*item).next = ::core::ptr::null_mut::<cmdq_item>();
        if !c.is_null() {
            (*c).references += 1;
        }
        (*item).client = c;
        (*item).queue = queue;
        (*item).entry.tqe_next = ::core::ptr::null_mut::<cmdq_item>();
        (*item).entry.tqe_prev = (*queue).list.tqh_last;
        *(*queue).list.tqh_last = item;
        (*queue).list.tqh_last = &raw mut (*item).entry.tqe_next;
        log_debug(
            b"%s %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_append\0" as *const u8 as *const ::core::ffi::c_char,
            cmdq_name(c),
            (*item).name,
        );
        item = next;
        if item.is_null() {
            break;
        }
    }
    return *(*((*queue).list.tqh_last as *mut cmdq_item_list)).tqh_last;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_insert_after(
    mut after: *mut cmdq_item,
    mut item: *mut cmdq_item,
) -> *mut cmdq_item {
    let mut c: *mut client = (*after).client;
    let mut queue: *mut cmdq_list = (*after).queue;
    let mut next: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    loop {
        next = (*item).next;
        (*item).next = (*after).next;
        (*after).next = item;
        if !c.is_null() {
            (*c).references += 1;
        }
        (*item).client = c;
        (*item).queue = queue;
        (*item).entry.tqe_next = (*after).entry.tqe_next;
        if !(*item).entry.tqe_next.is_null() {
            (*(*item).entry.tqe_next).entry.tqe_prev = &raw mut (*item).entry.tqe_next;
        } else {
            (*queue).list.tqh_last = &raw mut (*item).entry.tqe_next;
        }
        (*after).entry.tqe_next = item;
        (*item).entry.tqe_prev = &raw mut (*after).entry.tqe_next;
        log_debug(
            b"%s %s: %s after %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_insert_after\0" as *const u8 as *const ::core::ffi::c_char,
            cmdq_name(c),
            (*item).name,
            (*after).name,
        );
        after = item;
        item = next;
        if item.is_null() {
            break;
        }
    }
    return after;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_insert_hook(
    mut s: *mut session,
    mut item: *mut cmdq_item,
    mut current: *mut cmd_find_state,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut cmd: *mut cmd = (*item).cmd;
    let mut args_0: *mut args = cmd_get_args(cmd);
    let mut ae: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut av: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut ap: ::core::ffi::VaList;
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tmp: [::core::ffi::c_char; 32] = [0; 32];
    let mut flag: ::core::ffi::c_char = 0;
    let mut arguments: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*(*item).state).flags & CMDQ_STATE_NOHOOKS != 0 {
        return;
    }
    ap = args.clone();
    xvasprintf(&raw mut name, fmt, ap);
    ep = event_payload_create();
    if !current.is_null() {
        event_payload_set_target(ep, current);
    }
    event_payload_set_pointer(
        ep,
        b"_cmdq_item\0" as *const u8 as *const ::core::ffi::c_char,
        item as *mut ::core::ffi::c_void,
        None,
        None,
    );
    arguments = args_print(args_0);
    event_payload_set_string(
        ep,
        b"arguments\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        arguments,
    );
    free(arguments as *mut ::core::ffi::c_void);
    i = 0 as u_int;
    while i < args_count(args_0) {
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"argument_%u\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        event_payload_set_string(
            ep,
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            args_string(args_0, i),
        );
        i = i.wrapping_add(1);
    }
    flag = args_first(args_0, &raw mut ae) as ::core::ffi::c_char;
    while flag as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        value = args_get(args_0, flag as u_char);
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"flag_%c\0" as *const u8 as *const ::core::ffi::c_char,
            flag as ::core::ffi::c_int,
        );
        if value.is_null() {
            event_payload_set_string(
                ep,
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"1\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else {
            event_payload_set_string(
                ep,
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                value,
            );
        }
        i = 0 as u_int;
        av = args_first_value(args_0, flag as u_char);
        while !av.is_null() {
            xsnprintf(
                &raw mut tmp as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"flag_%c_%u\0" as *const u8 as *const ::core::ffi::c_char,
                flag as ::core::ffi::c_int,
                i,
            );
            event_payload_set_string(
                ep,
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*av).c2rust_unnamed.string,
            );
            i = i.wrapping_add(1);
            av = args_next_value(av);
        }
        flag = args_next(&raw mut ae) as ::core::ffi::c_char;
    }
    events_fire(name, ep);
    free(name as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_continue(mut item: *mut cmdq_item) {
    (*item).flags &= !CMDQ_WAITING;
}
unsafe extern "C" fn cmdq_remove(mut item: *mut cmdq_item) {
    if !(*item).client.is_null() {
        server_client_unref((*item).client);
    }
    if !(*item).cmdlist.is_null() {
        cmd_list_free((*item).cmdlist);
    }
    cmdq_free_state((*item).state);
    if !(*item).entry.tqe_next.is_null() {
        (*(*item).entry.tqe_next).entry.tqe_prev = (*item).entry.tqe_prev;
    } else {
        (*(*item).queue).list.tqh_last = (*item).entry.tqe_prev;
    }
    *(*item).entry.tqe_prev = (*item).entry.tqe_next;
    free((*item).name as *mut ::core::ffi::c_void);
    free(item as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn cmdq_remove_group(mut item: *mut cmdq_item) {
    let mut this: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut next: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    if (*item).group == 0 as u_int {
        return;
    }
    this = (*item).entry.tqe_next;
    while !this.is_null() {
        next = (*this).entry.tqe_next;
        if (*this).group == (*item).group {
            cmdq_remove(this);
        }
        this = next;
    }
}
unsafe extern "C" fn cmdq_empty_command(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    return CMD_RETURN_NORMAL;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_command(
    mut cmdlist: *mut cmd_list,
    mut state: *mut cmdq_state,
) -> *mut cmdq_item {
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut first: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut last: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut entry: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
    let mut created: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    cmd = cmd_list_first(cmdlist);
    if cmd.is_null() {
        return cmdq_get_callback1(
            b"cmdq_empty_command\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                cmdq_empty_command
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        );
    }
    if state.is_null() {
        state = cmdq_new_state(
            ::core::ptr::null_mut::<cmd_find_state>(),
            ::core::ptr::null_mut::<key_event>(),
            0 as ::core::ffi::c_int,
        );
        created = 1 as ::core::ffi::c_int;
    }
    while !cmd.is_null() {
        entry = cmd_get_entry(cmd);
        item =
            xcalloc(1 as size_t, ::core::mem::size_of::<cmdq_item>() as size_t) as *mut cmdq_item;
        xasprintf(
            &raw mut (*item).name,
            b"[%s/%p]\0" as *const u8 as *const ::core::ffi::c_char,
            (*entry).name,
            item,
        );
        (*item).type_0 = CMDQ_COMMAND;
        (*item).group = cmd_get_group(cmd);
        (*item).state = cmdq_link_state(state);
        (*item).cmdlist = cmdlist;
        (*item).cmd = cmd;
        (*cmdlist).references += 1;
        log_debug(
            b"%s: %s group %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_get_command\0" as *const u8 as *const ::core::ffi::c_char,
            (*item).name,
            (*item).group,
        );
        if first.is_null() {
            first = item;
        }
        if !last.is_null() {
            (*last).next = item;
        }
        last = item;
        cmd = cmd_list_next(cmd);
    }
    if created != 0 {
        cmdq_free_state(state);
    }
    return first;
}
unsafe extern "C" fn cmdq_find_flag(
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut flag: *const cmd_entry_flag,
) -> cmd_retval {
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*flag).flag as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        cmd_find_from_client(fs, (*item).target_client, 0 as ::core::ffi::c_int);
        return CMD_RETURN_NORMAL;
    }
    value = args_get(cmd_get_args((*item).cmd), (*flag).flag as u_char);
    if cmd_find_target(fs, item, value, (*flag).type_0, (*flag).flags) != 0 as ::core::ffi::c_int {
        cmd_find_clear_state(fs, 0 as ::core::ffi::c_int);
        return CMD_RETURN_ERROR;
    }
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmdq_add_message(mut item: *mut cmdq_item) {
    let mut c: *mut client = (*item).client;
    let mut state: *mut cmdq_state = (*item).state;
    let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut uid: uid_t = 0;
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    let mut user: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    tmp = cmd_print((*item).cmd);
    if !c.is_null() {
        uid = proc_get_peer_uid((*c).peer);
        if uid != -(1 as ::core::ffi::c_int) as uid_t && uid != getuid() {
            pw = getpwuid(uid as __uid_t);
            if !pw.is_null() {
                xasprintf(
                    &raw mut user,
                    b"[%s]\0" as *const u8 as *const ::core::ffi::c_char,
                    (*pw).pw_name,
                );
            } else {
                user = xstrdup(b"[unknown]\0" as *const u8 as *const ::core::ffi::c_char);
            }
        } else {
            user = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        }
        if !(*c).session.is_null()
            && (*state).event.key != KEYC_NONE as ::core::ffi::c_ulong as key_code
        {
            key = key_string_lookup_key((*state).event.key, 0 as ::core::ffi::c_int);
            server_add_message(
                b"%s%s key %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
                user,
                key,
                tmp,
            );
        } else {
            server_add_message(
                b"%s%s command: %s\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
                user,
                tmp,
            );
        }
        free(user as *mut ::core::ffi::c_void);
    } else {
        server_add_message(
            b"command: %s\0" as *const u8 as *const ::core::ffi::c_char,
            tmp,
        );
    }
    free(tmp as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn cmdq_fire_command(mut item: *mut cmdq_item) -> cmd_retval {
    let mut current_block: u64;
    let mut name: *const ::core::ffi::c_char = cmdq_name((*item).client);
    let mut state: *mut cmdq_state = (*item).state;
    let mut cmd: *mut cmd = (*item).cmd;
    let mut args: *mut args = cmd_get_args(cmd);
    let mut entry: *const cmd_entry = cmd_get_entry(cmd);
    let mut tc: *mut client = ::core::ptr::null_mut::<client>();
    let mut saved: *mut client = (*item).client;
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    let mut fsp: *mut cmd_find_state = ::core::ptr::null_mut::<cmd_find_state>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut flags: ::core::ffi::c_int = 0;
    let mut quiet: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if cfg_finished != 0 {
        cmdq_add_message(item);
    }
    if log_get_level() > 1 as ::core::ffi::c_int {
        tmp = cmd_print(cmd);
        log_debug(
            b"%s %s: (%u) %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_fire_command\0" as *const u8 as *const ::core::ffi::c_char,
            name,
            (*item).group,
            tmp,
        );
        free(tmp as *mut ::core::ffi::c_void);
    }
    flags = ((*state).flags & CMDQ_STATE_CONTROL != 0) as ::core::ffi::c_int;
    cmdq_guard(
        item,
        b"begin\0" as *const u8 as *const ::core::ffi::c_char,
        flags,
    );
    if (*item).client.is_null() {
        (*item).client = cmd_find_client(
            item,
            ::core::ptr::null::<::core::ffi::c_char>(),
            1 as ::core::ffi::c_int,
        );
    }
    if (*entry).flags & CMD_CLIENT_CANFAIL != 0 {
        quiet = 1 as ::core::ffi::c_int;
    }
    if (*entry).flags & CMD_CLIENT_CFLAG != 0 {
        tc = cmd_find_client(item, args_get(args, 'c' as i32 as u_char), quiet);
        if tc.is_null() && quiet == 0 {
            retval = CMD_RETURN_ERROR;
            current_block = 7379054416801212160;
        } else {
            current_block = 18317007320854588510;
        }
    } else if (*entry).flags & CMD_CLIENT_TFLAG != 0 {
        tc = cmd_find_client(item, args_get(args, 't' as i32 as u_char), quiet);
        if tc.is_null() && quiet == 0 {
            retval = CMD_RETURN_ERROR;
            current_block = 7379054416801212160;
        } else {
            current_block = 18317007320854588510;
        }
    } else {
        tc = cmd_find_client(
            item,
            ::core::ptr::null::<::core::ffi::c_char>(),
            1 as ::core::ffi::c_int,
        );
        current_block = 18317007320854588510;
    }
    match current_block {
        18317007320854588510 => {
            (*item).target_client = tc;
            retval = cmdq_find_flag(item, &raw mut (*item).source, &raw const (*entry).source);
            if !(retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int) {
                retval = cmdq_find_flag(item, &raw mut (*item).target, &raw const (*entry).target);
                if !(retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int) {
                    retval = (*entry).exec.expect("non-null function pointer")(cmd, item);
                    if !(retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int) {
                        if (*entry).flags & CMD_AFTERHOOK != 0 {
                            if cmd_find_valid_state(&raw mut (*item).target) != 0 {
                                fsp = &raw mut (*item).target;
                                current_block = 8704759739624374314;
                            } else if cmd_find_valid_state(&raw mut (*(*item).state).current) != 0 {
                                fsp = &raw mut (*(*item).state).current;
                                current_block = 8704759739624374314;
                            } else if cmd_find_from_client(
                                &raw mut fs,
                                (*item).client,
                                0 as ::core::ffi::c_int,
                            ) == 0 as ::core::ffi::c_int
                            {
                                fsp = &raw mut fs;
                                current_block = 8704759739624374314;
                            } else {
                                current_block = 7379054416801212160;
                            }
                            match current_block {
                                7379054416801212160 => {}
                                _ => {
                                    cmdq_insert_hook(
                                        (*fsp).s,
                                        item,
                                        fsp,
                                        b"after-%s\0" as *const u8 as *const ::core::ffi::c_char,
                                        (*entry).name,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    (*item).client = saved;
    if retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int {
        fsp = ::core::ptr::null_mut::<cmd_find_state>();
        if cmd_find_valid_state(&raw mut (*item).target) != 0 {
            fsp = &raw mut (*item).target;
        } else if cmd_find_valid_state(&raw mut (*(*item).state).current) != 0 {
            fsp = &raw mut (*(*item).state).current;
        } else if cmd_find_from_client(&raw mut fs, (*item).client, 0 as ::core::ffi::c_int)
            == 0 as ::core::ffi::c_int
        {
            fsp = &raw mut fs;
        }
        cmdq_insert_hook(
            if !fsp.is_null() {
                (*fsp).s
            } else {
                ::core::ptr::null_mut::<session>()
            },
            item,
            fsp,
            b"command-error\0" as *const u8 as *const ::core::ffi::c_char,
        );
        cmdq_guard(
            item,
            b"error\0" as *const u8 as *const ::core::ffi::c_char,
            flags,
        );
    } else {
        cmdq_guard(
            item,
            b"end\0" as *const u8 as *const ::core::ffi::c_char,
            flags,
        );
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_callback1(
    mut name: *const ::core::ffi::c_char,
    mut cb: cmdq_cb,
    mut data: *mut ::core::ffi::c_void,
) -> *mut cmdq_item {
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    item = xcalloc(1 as size_t, ::core::mem::size_of::<cmdq_item>() as size_t) as *mut cmdq_item;
    xasprintf(
        &raw mut (*item).name,
        b"[%s/%p]\0" as *const u8 as *const ::core::ffi::c_char,
        name,
        item,
    );
    (*item).type_0 = CMDQ_CALLBACK;
    (*item).group = 0 as u_int;
    (*item).state = cmdq_new_state(
        ::core::ptr::null_mut::<cmd_find_state>(),
        ::core::ptr::null_mut::<key_event>(),
        0 as ::core::ffi::c_int,
    );
    (*item).cb = cb;
    (*item).data = data;
    return item;
}
unsafe extern "C" fn cmdq_error_callback(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut error: *mut ::core::ffi::c_char = data as *mut ::core::ffi::c_char;
    cmdq_error(
        item,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        error,
    );
    free(error as *mut ::core::ffi::c_void);
    return CMD_RETURN_NORMAL;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_error(mut error: *const ::core::ffi::c_char) -> *mut cmdq_item {
    return cmdq_get_callback1(
        b"cmdq_error_callback\0" as *const u8 as *const ::core::ffi::c_char,
        Some(
            cmdq_error_callback
                as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
        ),
        xstrdup(error) as *mut ::core::ffi::c_void,
    );
}
unsafe extern "C" fn cmdq_fire_callback(mut item: *mut cmdq_item) -> cmd_retval {
    return (*item).cb.expect("non-null function pointer")(item, (*item).data);
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_next(mut c: *mut client) -> u_int {
    let mut current_block: u64;
    let mut queue: *mut cmdq_list = cmdq_get(c);
    let mut name: *const ::core::ffi::c_char = cmdq_name(c);
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    let mut items: u_int = 0 as u_int;
    static mut number: u_int = 0;
    if (*queue).list.tqh_first.is_null() {
        log_debug(
            b"%s %s: empty\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_next\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
        return 0 as u_int;
    }
    if (*(*queue).list.tqh_first).flags & CMDQ_WAITING != 0 {
        log_debug(
            b"%s %s: waiting\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_next\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
        return 0 as u_int;
    }
    log_debug(
        b"%s %s: enter\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmdq_next\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    loop {
        (*queue).item = (*queue).list.tqh_first;
        item = (*queue).item;
        if item.is_null() {
            current_block = 7056779235015430508;
            break;
        }
        log_debug(
            b"%s %s: %s (%d), flags %x\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_next\0" as *const u8 as *const ::core::ffi::c_char,
            name,
            (*item).name,
            (*item).type_0 as ::core::ffi::c_uint,
            (*item).flags,
        );
        if (*item).flags & CMDQ_WAITING != 0 {
            current_block = 8107417685847451382;
            break;
        }
        if !(*item).flags & CMDQ_FIRED != 0 {
            (*item).time = time(::core::ptr::null_mut::<time_t>());
            number = number.wrapping_add(1);
            (*item).number = number;
            match (*item).type_0 as ::core::ffi::c_uint {
                0 => {
                    retval = cmdq_fire_command(item);
                    if retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int {
                        cmdq_remove_group(item);
                    }
                }
                1 => {
                    retval = cmdq_fire_callback(item);
                }
                _ => {
                    retval = CMD_RETURN_ERROR;
                }
            }
            (*item).flags |= CMDQ_FIRED;
            if retval as ::core::ffi::c_int == CMD_RETURN_WAIT as ::core::ffi::c_int {
                (*item).flags |= CMDQ_WAITING;
                current_block = 8107417685847451382;
                break;
            } else {
                items = items.wrapping_add(1);
            }
        }
        cmdq_remove(item);
    }
    match current_block {
        8107417685847451382 => {
            log_debug(
                b"%s %s: exit (wait)\0" as *const u8 as *const ::core::ffi::c_char,
                b"cmdq_next\0" as *const u8 as *const ::core::ffi::c_char,
                name,
            );
            return items;
        }
        _ => {
            (*queue).item = ::core::ptr::null_mut::<cmdq_item>();
            log_debug(
                b"%s %s: exit (empty)\0" as *const u8 as *const ::core::ffi::c_char,
                b"cmdq_next\0" as *const u8 as *const ::core::ffi::c_char,
                name,
            );
            return items;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_running(mut c: *mut client) -> *mut cmdq_item {
    let mut queue: *mut cmdq_list = cmdq_get(c);
    if (*queue).item.is_null() {
        return ::core::ptr::null_mut::<cmdq_item>();
    }
    if (*(*queue).item).flags & CMDQ_WAITING != 0 {
        return ::core::ptr::null_mut::<cmdq_item>();
    }
    return (*queue).item;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_guard(
    mut item: *mut cmdq_item,
    mut guard: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) {
    let mut c: *mut client = (*item).client;
    let mut t: ::core::ffi::c_long = (*item).time as ::core::ffi::c_long;
    let mut number: u_int = (*item).number;
    if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        control_write_guard(c, guard, t, number, flags);
    }
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_print_data(mut item: *mut cmdq_item, mut evb: *mut evbuffer) {
    server_client_print((*item).client, 1 as ::core::ffi::c_int, evb);
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_print(
    mut item: *mut cmdq_item,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    let mut evb: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    evb = evbuffer_new();
    if evb.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    ap = args.clone();
    evbuffer_add_vprintf(evb, fmt, ap);
    cmdq_print_data(item, evb);
    evbuffer_free(evb);
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_error(
    mut item: *mut cmdq_item,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut c: *mut client = (*item).client;
    let mut cmd: *mut cmd = (*item).cmd;
    let mut ap: ::core::ffi::VaList;
    let mut msg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut file: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut line: u_int = 0;
    ap = args.clone();
    xvasprintf(&raw mut msg, fmt, ap);
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmdq_error\0" as *const u8 as *const ::core::ffi::c_char,
        msg,
    );
    if c.is_null() {
        cmd_get_source(cmd, &raw mut file, &raw mut line);
        if cfg_finished == 0 {
            if !file.is_null() {
                cfg_add_cause(
                    b"%s:%u: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    file,
                    line,
                    msg,
                );
            } else {
                cfg_add_cause(b"%s\0" as *const u8 as *const ::core::ffi::c_char, msg);
            }
        } else if !file.is_null() {
            server_add_message(
                b"message: %s:%u: %s\0" as *const u8 as *const ::core::ffi::c_char,
                file,
                line,
                msg,
            );
        } else {
            server_add_message(
                b"message: %s\0" as *const u8 as *const ::core::ffi::c_char,
                msg,
            );
        }
    } else if (*c).session.is_null() || (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        server_add_message(
            b"%s message: %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            msg,
        );
        if !(*c).flags & CLIENT_UTF8 as uint64_t != 0 {
            tmp = msg;
            msg = utf8_sanitize(tmp);
            free(tmp as *mut ::core::ffi::c_void);
        }
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_write(c, b"%s\0" as *const u8 as *const ::core::ffi::c_char, msg);
        } else {
            file_error(c, b"%s\n\0" as *const u8 as *const ::core::ffi::c_char, msg);
        }
        (*c).retval = 1 as ::core::ffi::c_int;
    } else {
        *msg = ({
            let mut __res: ::core::ffi::c_int = 0;
            if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                if 0 != 0 {
                    let mut __c: ::core::ffi::c_int = *msg as u_char as ::core::ffi::c_int;
                    __res =
                        (if __c < -(128 as ::core::ffi::c_int) || __c > 255 as ::core::ffi::c_int {
                            __c as __int32_t
                        } else {
                            *(*__ctype_toupper_loc()).offset(__c as isize)
                        }) as ::core::ffi::c_int;
                } else {
                    __res = toupper(*msg as u_char as ::core::ffi::c_int);
                }
            } else {
                __res = *(*__ctype_toupper_loc())
                    .offset(*msg as u_char as ::core::ffi::c_int as isize)
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
            msg,
        );
    }
    free(msg as *mut ::core::ffi::c_void);
}

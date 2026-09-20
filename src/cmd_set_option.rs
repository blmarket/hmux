pub use crate::src::shared::monitor::{
    MONITOR_ALL_PANES, MONITOR_ALL_WINDOWS, MONITOR_NOTIFY_TRUE, MONITOR_PANE, MONITOR_SESSION,
    MONITOR_WINDOW, monitor_type,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::options::{OPTIONS_TABLE_NONE, OPTIONS_TABLE_WINDOW};
pub use crate::src::shared::command::{CMD_AFTERHOOK, CMD_FIND_CANFAIL};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::options::*;
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
    pub type cmd;
    pub type event_payload;
    pub type options_array_item;
    pub type options_entry;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    static mut global_options: *mut options;
    static mut global_s_options: *mut options;
    static mut global_w_options: *mut options;
    fn format_single_from_target(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn event_payload_create() -> *mut event_payload;
    fn event_payload_set_target(_: *mut event_payload, _: *mut cmd_find_state);
    fn event_payload_set_int(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    );
    fn event_payload_set_client(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut client,
    );
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
    fn event_payload_set_pane(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut window_pane,
    );
    fn events_fire(_: *const ::core::ffi::c_char, _: *mut event_payload);
    fn hooks_add_event(_: *const ::core::ffi::c_char);
    fn hooks_run(_: *mut cmdq_item, _: *const ::core::ffi::c_char);
    fn hooks_monitor_add(
        _: *mut cmdq_item,
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: monitor_type,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut cmd_find_state,
        _: *mut session,
    );
    fn hooks_monitor_remove(_: *mut options, _: *const ::core::ffi::c_char);
    fn options_empty(_: *mut options, _: *const options_table_entry) -> *mut options_entry;
    fn options_table_entry(_: *mut options_entry) -> *const options_table_entry;
    fn options_get_only(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_get(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_array_clear(_: *mut options_entry);
    fn options_array_get(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
    ) -> *mut options_value;
    fn options_array_set(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn options_array_assign(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn options_is_array(_: *mut options_entry) -> ::core::ffi::c_int;
    fn options_match(
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
        _: *mut ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn options_set_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    ) -> *mut options_entry;
    fn options_scope_from_name(
        _: *mut args,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        _: *mut cmd_find_state,
        _: *mut *mut options,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn options_from_string(
        _: *mut options,
        _: *const options_table_entry,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn options_push_changes(_: *const ::core::ffi::c_char);
    fn options_remove_or_default(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn cmd_find_copy_state(_: *mut cmd_find_state, _: *mut cmd_find_state);
    fn cmd_get_entry(_: *mut cmd) -> *const cmd_entry;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn monitor_parse(
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
        _: *mut monitor_type,
        _: *mut ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_array {
    pub rbh_root: *mut options_array_item,
}
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
#[no_mangle]
pub static mut cmd_set_option_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"set-option\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"set\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"aFgopqst:uUw\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: 2 as ::core::ffi::c_int,
            cb: Some(
                cmd_set_option_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-aFgopqsuUw] [-t target-pane] option [value]\0" as *const u8
            as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_CANFAIL,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_set_option_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_set_window_option_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"set-window-option\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"setw\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"aFgoqt:u\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: 2 as ::core::ffi::c_int,
            cb: Some(
                cmd_set_option_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-aFgoqu] [-t target-window] option [value]\0" as *const u8
            as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_WINDOW,
            flags: CMD_FIND_CANFAIL,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_set_option_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_set_hook_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"set-hook\0" as *const u8 as *const ::core::ffi::c_char,
        alias: ::core::ptr::null::<::core::ffi::c_char>(),
        args: args_parse {
            template: b"agpERTt:uB:w\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 2 as ::core::ffi::c_int,
            cb: Some(
                cmd_set_option_args_parse
                    as unsafe extern "C" fn(
                        *mut args,
                        u_int,
                        *mut *mut ::core::ffi::c_char,
                    ) -> args_parse_type,
            ),
        },
        usage: b"[-agpERTuw] [-B name:what:format] [-t target-pane] [hook] [command]\0" as *const u8
            as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: CMD_FIND_CANFAIL,
        },
        flags: CMD_AFTERHOOK,
        exec: Some(
            cmd_set_option_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_set_option_args_parse(
    mut args: *mut args,
    mut idx: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> args_parse_type {
    if args_has(args, 'B' as i32 as u_char) != 0 {
        return ARGS_PARSE_COMMANDS_OR_STRING;
    }
    if idx == 1 as u_int {
        return ARGS_PARSE_COMMANDS_OR_STRING;
    }
    return ARGS_PARSE_STRING;
}
unsafe extern "C" fn cmd_set_hook_event_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut argument: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if args_count(args) == 0 as u_int {
        cmdq_error(
            item,
            b"missing argument\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if args_count(args) != 1 as u_int {
        cmdq_error(
            item,
            b"too many arguments\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    argument = format_single_from_target(item, args_string(args, 0 as u_int));
    if *argument as ::core::ffi::c_int != '@' as i32 {
        cmdq_error(
            item,
            b"event name must start with @\0" as *const u8 as *const ::core::ffi::c_char,
        );
        free(argument as *mut ::core::ffi::c_void);
        return CMD_RETURN_ERROR;
    }
    ep = event_payload_create();
    event_payload_set_target(ep, target);
    c = cmdq_get_client(item);
    if !c.is_null() {
        event_payload_set_client(
            ep,
            b"client\0" as *const u8 as *const ::core::ffi::c_char,
            c,
        );
    }
    if !(*target).s.is_null() {
        event_payload_set_session(
            ep,
            b"session\0" as *const u8 as *const ::core::ffi::c_char,
            (*target).s,
        );
    }
    if !(*target).w.is_null() {
        event_payload_set_window(
            ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            (*target).w,
        );
    }
    if !(*target).wl.is_null() {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*target).wl).idx,
        );
    } else if (*target).idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*target).idx,
        );
    }
    if !(*target).wp.is_null() {
        event_payload_set_pane(
            ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            (*target).wp,
        );
    }
    events_fire(argument, ep);
    free(argument as *mut ::core::ffi::c_void);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_set_hook_monitor_exec(
    mut item: *mut cmdq_item,
    mut args: *mut args,
    mut window: ::core::ffi::c_int,
) -> cmd_retval {
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut format: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut newvalue: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut old: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut type_0: monitor_type = MONITOR_SESSION;
    let mut id: ::core::ffi::c_int = 0;
    let mut scope: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if args_count(args) > 1 as u_int {
        cmdq_error(
            item,
            b"too many arguments\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    value = args_get(args, 'B' as i32 as u_char);
    if args_has(args, 'u' as i32 as u_char) != 0 {
        if monitor_parse(
            value,
            &raw mut name,
            &raw mut type_0,
            &raw mut id,
            &raw mut format,
        ) != 0 as ::core::ffi::c_int
        {
            name = xstrdup(value);
        }
        free(format as *mut ::core::ffi::c_void);
        format = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else if monitor_parse(
        value,
        &raw mut name,
        &raw mut type_0,
        &raw mut id,
        &raw mut format,
    ) != 0 as ::core::ffi::c_int
    {
        cmdq_error(
            item,
            b"invalid subscription: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return CMD_RETURN_ERROR;
    }
    if *name as ::core::ffi::c_int != '@' as i32 {
        cmdq_error(
            item,
            b"monitor hook name must start with @\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        scope = options_scope_from_name(args, window, name, target, &raw mut oo, &raw mut cause);
        if scope == OPTIONS_TABLE_NONE {
            cmdq_error(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
        } else {
            cmd_find_copy_state(&raw mut fs, target);
            if args_has(args, 'u' as i32 as u_char) != 0 {
                hooks_monitor_remove(oo, name);
            } else {
                if args_count(args) != 0 as u_int {
                    value = args_string(args, 0 as u_int);
                    if args_has(args, 'F' as i32 as u_char) != 0 {
                        expanded = format_single_from_target(item, value);
                        value = expanded;
                    }
                    o = options_get_only(oo, name);
                    if args_has(args, 'o' as i32 as u_char) == 0 || o.is_null() {
                        if args_has(args, 'a' as i32 as u_char) != 0 && !o.is_null() {
                            old = options_get_string(oo, name);
                            xasprintf(
                                &raw mut newvalue,
                                b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                                old,
                                value,
                            );
                            value = newvalue;
                        }
                        options_set_string(
                            oo,
                            name,
                            0 as ::core::ffi::c_int,
                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                            value,
                        );
                        options_push_changes(name);
                    }
                }
                if oo != global_options && oo != global_s_options && oo != global_w_options {
                    s = (*target).s;
                }
                if args_has(args, 'T' as i32 as u_char) != 0 {
                    flags |= MONITOR_NOTIFY_TRUE;
                }
                hooks_monitor_add(item, oo, name, type_0, id, format, flags, &raw mut fs, s);
            }
            free(newvalue as *mut ::core::ffi::c_void);
            free(expanded as *mut ::core::ffi::c_void);
            free(name as *mut ::core::ffi::c_void);
            free(format as *mut ::core::ffi::c_void);
            return CMD_RETURN_NORMAL;
        }
    }
    free(newvalue as *mut ::core::ffi::c_void);
    free(expanded as *mut ::core::ffi::c_void);
    free(name as *mut ::core::ffi::c_void);
    free(format as *mut ::core::ffi::c_void);
    return CMD_RETURN_ERROR;
}
unsafe extern "C" fn cmd_set_option_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut append: ::core::ffi::c_int = args_has(args, 'a' as i32 as u_char);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut loop_0: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut parent: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut po: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut argument: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut array_key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut window: ::core::ffi::c_int = 0;
    let mut already: ::core::ffi::c_int = 0;
    let mut error: ::core::ffi::c_int = 0;
    let mut ambiguous: ::core::ffi::c_int = 0;
    let mut scope: ::core::ffi::c_int = 0;
    window =
        (cmd_get_entry(self_0) == &raw const cmd_set_window_option_entry) as ::core::ffi::c_int;
    if cmd_get_entry(self_0) == &raw const cmd_set_hook_entry
        && args_has(args, 'E' as i32 as u_char) != 0
    {
        return cmd_set_hook_event_exec(self_0, item);
    }
    if cmd_get_entry(self_0) == &raw const cmd_set_hook_entry
        && args_has(args, 'B' as i32 as u_char) != 0
    {
        return cmd_set_hook_monitor_exec(item, args, window);
    }
    if args_count(args) == 0 as u_int {
        cmdq_error(
            item,
            b"missing argument\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    argument = format_single_from_target(item, args_string(args, 0 as u_int));
    if cmd_get_entry(self_0) == &raw const cmd_set_hook_entry
        && args_has(args, 'R' as i32 as u_char) != 0
    {
        hooks_run(item, argument);
        free(argument as *mut ::core::ffi::c_void);
        return CMD_RETURN_NORMAL;
    }
    name = options_match(argument, &raw mut array_key, &raw mut ambiguous);
    if name.is_null() {
        if args_has(args, 'q' as i32 as u_char) != 0 {
            current_block = 710513931074292511;
        } else {
            if ambiguous != 0 {
                cmdq_error(
                    item,
                    b"ambiguous option: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    argument,
                );
            } else {
                cmdq_error(
                    item,
                    b"invalid option: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    argument,
                );
            }
            current_block = 8517774764635037400;
        }
    } else {
        if args_count(args) < 2 as u_int {
            value = ::core::ptr::null::<::core::ffi::c_char>();
        } else {
            value = args_string(args, 1 as u_int);
        }
        if !value.is_null() && args_has(args, 'F' as i32 as u_char) != 0 {
            expanded = format_single_from_target(item, value);
            value = expanded;
        }
        scope = options_scope_from_name(args, window, name, target, &raw mut oo, &raw mut cause);
        if scope == OPTIONS_TABLE_NONE {
            if args_has(args, 'q' as i32 as u_char) != 0 {
                current_block = 710513931074292511;
            } else {
                cmdq_error(
                    item,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    cause,
                );
                free(cause as *mut ::core::ffi::c_void);
                current_block = 8517774764635037400;
            }
        } else {
            o = options_get_only(oo, name);
            parent = options_get(oo, name);
            if !array_key.is_null()
                && (*name as ::core::ffi::c_int == '@' as i32 || options_is_array(parent) == 0)
            {
                cmdq_error(
                    item,
                    b"not an array: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    argument,
                );
                current_block = 8517774764635037400;
            } else {
                if args_has(args, 'u' as i32 as u_char) == 0
                    && args_has(args, 'o' as i32 as u_char) != 0
                {
                    if array_key.is_null() {
                        already = (o != NULL as *mut options_entry) as ::core::ffi::c_int;
                    } else if o.is_null() {
                        already = 0 as ::core::ffi::c_int;
                    } else if !options_array_get(o, array_key).is_null() {
                        already = 1 as ::core::ffi::c_int;
                    } else {
                        already = 0 as ::core::ffi::c_int;
                    }
                    if already != 0 {
                        if args_has(args, 'q' as i32 as u_char) != 0 {
                            current_block = 710513931074292511;
                        } else {
                            cmdq_error(
                                item,
                                b"already set: %s\0" as *const u8 as *const ::core::ffi::c_char,
                                argument,
                            );
                            current_block = 8517774764635037400;
                        }
                    } else {
                        current_block = 12997042908615822766;
                    }
                } else {
                    current_block = 12997042908615822766;
                }
                match current_block {
                    710513931074292511 => {}
                    8517774764635037400 => {}
                    _ => {
                        if args_has(args, 'U' as i32 as u_char) != 0
                            && scope == OPTIONS_TABLE_WINDOW
                        {
                            loop_0 = (*(*target).w).panes.tqh_first;
                            loop {
                                if loop_0.is_null() {
                                    current_block = 10095721787123848864;
                                    break;
                                }
                                po = options_get_only((*loop_0).options, name);
                                if !po.is_null() {
                                    if options_remove_or_default(po, array_key, &raw mut cause)
                                        != 0 as ::core::ffi::c_int
                                    {
                                        cmdq_error(
                                            item,
                                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            cause,
                                        );
                                        free(cause as *mut ::core::ffi::c_void);
                                        current_block = 8517774764635037400;
                                        break;
                                    }
                                }
                                loop_0 = (*loop_0).entry.tqe_next;
                            }
                        } else {
                            current_block = 10095721787123848864;
                        }
                        match current_block {
                            8517774764635037400 => {}
                            _ => {
                                if args_has(args, 'u' as i32 as u_char) != 0
                                    || args_has(args, 'U' as i32 as u_char) != 0
                                {
                                    if o.is_null() {
                                        current_block = 710513931074292511;
                                    } else if options_remove_or_default(
                                        o,
                                        array_key,
                                        &raw mut cause,
                                    ) != 0 as ::core::ffi::c_int
                                    {
                                        cmdq_error(
                                            item,
                                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            cause,
                                        );
                                        free(cause as *mut ::core::ffi::c_void);
                                        current_block = 8517774764635037400;
                                    } else {
                                        current_block = 16231175055492490595;
                                    }
                                } else if *name as ::core::ffi::c_int == '@' as i32 {
                                    if value.is_null() {
                                        cmdq_error(
                                            item,
                                            b"empty value\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                        );
                                        current_block = 8517774764635037400;
                                    } else {
                                        options_set_string(
                                            oo,
                                            name,
                                            append,
                                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            value,
                                        );
                                        if cmd_get_entry(self_0) == &raw const cmd_set_hook_entry {
                                            hooks_add_event(name);
                                        }
                                        current_block = 16231175055492490595;
                                    }
                                } else if array_key.is_null() && options_is_array(parent) == 0 {
                                    error = options_from_string(
                                        oo,
                                        options_table_entry(parent),
                                        (*options_table_entry(parent)).name,
                                        value,
                                        args_has(args, 'a' as i32 as u_char),
                                        &raw mut cause,
                                    );
                                    if error != 0 as ::core::ffi::c_int {
                                        cmdq_error(
                                            item,
                                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            cause,
                                        );
                                        free(cause as *mut ::core::ffi::c_void);
                                        current_block = 8517774764635037400;
                                    } else {
                                        current_block = 16231175055492490595;
                                    }
                                } else if value.is_null() {
                                    cmdq_error(
                                        item,
                                        b"empty value\0" as *const u8 as *const ::core::ffi::c_char,
                                    );
                                    current_block = 8517774764635037400;
                                } else {
                                    if o.is_null() {
                                        o = options_empty(oo, options_table_entry(parent));
                                    }
                                    if array_key.is_null() {
                                        if append == 0 {
                                            options_array_clear(o);
                                        }
                                        if options_array_assign(o, value, &raw mut cause)
                                            != 0 as ::core::ffi::c_int
                                        {
                                            cmdq_error(
                                                item,
                                                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                                cause,
                                            );
                                            free(cause as *mut ::core::ffi::c_void);
                                            current_block = 8517774764635037400;
                                        } else {
                                            current_block = 16231175055492490595;
                                        }
                                    } else if options_array_set(
                                        o,
                                        array_key,
                                        value,
                                        append,
                                        &raw mut cause,
                                    ) != 0 as ::core::ffi::c_int
                                    {
                                        cmdq_error(
                                            item,
                                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            cause,
                                        );
                                        free(cause as *mut ::core::ffi::c_void);
                                        current_block = 8517774764635037400;
                                    } else {
                                        current_block = 16231175055492490595;
                                    }
                                }
                                match current_block {
                                    8517774764635037400 => {}
                                    710513931074292511 => {}
                                    _ => {
                                        options_push_changes(name);
                                        current_block = 710513931074292511;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    match current_block {
        8517774764635037400 => {
            free(argument as *mut ::core::ffi::c_void);
            free(expanded as *mut ::core::ffi::c_void);
            free(name as *mut ::core::ffi::c_void);
            free(array_key as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
        _ => {
            free(argument as *mut ::core::ffi::c_void);
            free(expanded as *mut ::core::ffi::c_void);
            free(name as *mut ::core::ffi::c_void);
            free(array_key as *mut ::core::ffi::c_void);
            return CMD_RETURN_NORMAL;
        }
    };
}

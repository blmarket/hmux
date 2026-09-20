pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::tree::{RB_NEGINF};
pub use crate::src::shared::command::{
    CMDQ_STATE_REPEAT, CMD_FIND_CANFAIL, CMD_STARTSERVER, CMD_TARGET_SESSION_USAGE,
};
pub use crate::src::shared::client::{CLIENT_ATTACHED, CLIENT_CONTROL};
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
    pub type options_entry;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn tcgetattr(__fd: ::core::ffi::c_int, __termios_p: *mut termios) -> ::core::ffi::c_int;
    fn sscanf(
        __s: *const ::core::ffi::c_char,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn strtonum(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut global_s_options: *mut options;
    fn clean_name(_: *const ::core::ffi::c_char, _: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn check_name(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn proc_send(
        _: *mut tmuxpeer,
        _: msgtype,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_void,
        _: size_t,
    ) -> ::core::ffi::c_int;
    static mut cfg_finished: ::core::ffi::c_int;
    fn cfg_show_causes(_: *mut session);
    fn format_single(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut ::core::ffi::c_char;
    fn events_fire_session(_: *const ::core::ffi::c_char, _: *mut session);
    fn options_create(_: *mut options) -> *mut options;
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn options_set_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    ) -> *mut options_entry;
    fn environ_create() -> *mut environ;
    fn environ_put(_: *mut environ, _: *const ::core::ffi::c_char, _: ::core::ffi::c_int);
    fn environ_update(_: *mut options, _: *mut environ, _: *mut environ);
    fn args_to_vector(
        _: *mut args,
        _: *mut ::core::ffi::c_int,
        _: *mut *mut *mut ::core::ffi::c_char,
    );
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_count(_: *mut args) -> u_int;
    fn args_first_value(_: *mut args, _: u_char) -> *mut args_value;
    fn args_next_value(_: *mut args_value) -> *mut args_value;
    fn cmd_find_from_session(_: *mut cmd_find_state, _: *mut session, _: ::core::ffi::c_int);
    fn cmd_free_argv(_: ::core::ffi::c_int, _: *mut *mut ::core::ffi::c_char);
    fn cmd_get_entry(_: *mut cmd) -> *const cmd_entry;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmd_attach_session(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
    ) -> cmd_retval;
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_current(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_flags(_: *mut cmdq_item) -> ::core::ffi::c_int;
    fn cmdq_insert_hook(
        _: *mut session,
        _: *mut cmdq_item,
        _: *mut cmd_find_state,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn cmdq_print(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn server_client_set_key_table(_: *mut client, _: *const ::core::ffi::c_char);
    fn server_client_check_nested(_: *mut client) -> ::core::ffi::c_int;
    fn server_client_open(_: *mut client, _: *mut *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn server_client_set_session(_: *mut client, _: *mut session);
    fn server_client_get_cwd(_: *mut client, _: *mut session) -> *const ::core::ffi::c_char;
    fn server_client_set_flags(_: *mut client, _: *const ::core::ffi::c_char);
    fn winlinks_RB_MINMAX(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn session_find(_: *const ::core::ffi::c_char) -> *mut session;
    fn session_create(
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: *mut environ,
        _: *mut options,
        _: *mut termios,
    ) -> *mut session;
    fn session_destroy(_: *mut session, _: ::core::ffi::c_int, _: *const ::core::ffi::c_char);
    fn session_select(_: *mut session, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn session_group_contains(_: *mut session) -> *mut session_group;
    fn session_group_find(_: *const ::core::ffi::c_char) -> *mut session_group;
    fn session_group_new(_: *const ::core::ffi::c_char) -> *mut session_group;
    fn session_group_add(_: *mut session_group, _: *mut session);
    fn session_group_synchronize_to(_: *mut session);
    fn fatal(_: *const ::core::ffi::c_char, ...) -> !;
    fn spawn_window(_: *mut spawn_context, _: *mut *mut ::core::ffi::c_char) -> *mut winlink;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct spawn_context {
    pub item: *mut cmdq_item,
    pub s: *mut session,
    pub wl: *mut winlink,
    pub tc: *mut client,
    pub wp0: *mut window_pane,
    pub lc: *mut layout_cell,
    pub name: *const ::core::ffi::c_char,
    pub argv: *mut *mut ::core::ffi::c_char,
    pub argc: ::core::ffi::c_int,
    pub environ: *mut environ,
    pub idx: ::core::ffi::c_int,
    pub cwd: *const ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
}
pub const USHRT_MAX: ::core::ffi::c_int =
    __SHRT_MAX__ * 2 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const NEW_SESSION_TEMPLATE: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"#{session_name}:\0")
};
#[no_mangle]
pub static mut cmd_new_session_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"new-session\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"new\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"Ac:dDe:EF:f:n:Ps:t:x:Xy:\0" as *const u8
                as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: b"[-AdDEPX] [-c start-directory] [-e environment] [-F format] [-f flags] [-n window-name] [-s session-name] [-t target-session] [-x width] [-y height] [shell-command [argument ...]]\0"
            as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: CMD_FIND_CANFAIL,
        },
        flags: CMD_STARTSERVER,
        exec: Some(
            cmd_new_session_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_has_session_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"has-session\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"has\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"t:\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: 0 as ::core::ffi::c_int,
            cb: None,
        },
        usage: CMD_TARGET_SESSION_USAGE.as_ptr(),
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_SESSION,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_new_session_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_new_session_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut current: *mut cmd_find_state = cmdq_get_current(item);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut c: *mut client = cmdq_get_client(item);
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut as_0: *mut session = ::core::ptr::null_mut::<session>();
    let mut groupwith: *mut session = ::core::ptr::null_mut::<session>();
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut tio: termios = termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_line: 0,
        c_cc: [0; 32],
        c2rust_unnamed: termios_input_speed { __ispeed: 0 },
        c2rust_unnamed_0: termios_output_speed { __ospeed: 0 },
    };
    let mut tiop: *mut termios = ::core::ptr::null_mut::<termios>();
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut group: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tmp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cwd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ename: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut wname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut sname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut prefix: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut detached: ::core::ffi::c_int = 0;
    let mut already_attached: ::core::ffi::c_int = 0;
    let mut is_control: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut dsx: u_int = 0;
    let mut dsy: u_int = 0;
    let mut count: u_int = args_count(args);
    let mut sc: spawn_context = spawn_context {
        item: ::core::ptr::null_mut::<cmdq_item>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        tc: ::core::ptr::null_mut::<client>(),
        wp0: ::core::ptr::null_mut::<window_pane>(),
        lc: ::core::ptr::null_mut::<layout_cell>(),
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        argv: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        argc: 0,
        environ: ::core::ptr::null_mut::<environ>(),
        idx: 0,
        cwd: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: 0,
    };
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut av: *mut args_value = ::core::ptr::null_mut::<args_value>();
    if cmd_get_entry(self_0) == &raw const cmd_has_session_entry {
        return CMD_RETURN_NORMAL;
    }
    if args_has(args, 't' as i32 as u_char) != 0
        && (count != 0 as u_int || args_has(args, 'n' as i32 as u_char) != 0)
    {
        cmdq_error(
            item,
            b"command or window name given with target\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    tmp = args_get(args, 'n' as i32 as u_char);
    if !tmp.is_null() {
        ename = format_single(
            item,
            tmp,
            c,
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
        if check_name(ename) == 0 {
            cmdq_error(
                item,
                b"invalid window name: %s\0" as *const u8 as *const ::core::ffi::c_char,
                ename,
            );
            free(ename as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
        wname = clean_name(ename, 0 as ::core::ffi::c_int);
        free(ename as *mut ::core::ffi::c_void);
    }
    tmp = args_get(args, 's' as i32 as u_char);
    if !tmp.is_null() {
        ename = format_single(
            item,
            tmp,
            c,
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
        if check_name(ename) == 0 {
            cmdq_error(
                item,
                b"invalid session name: %s\0" as *const u8 as *const ::core::ffi::c_char,
                ename,
            );
            free(ename as *mut ::core::ffi::c_void);
            current_block = 5193972633326621385;
        } else {
            sname = clean_name(ename, 0 as ::core::ffi::c_int);
            free(ename as *mut ::core::ffi::c_void);
            current_block = 10043043949733653460;
        }
    } else {
        current_block = 10043043949733653460;
    }
    match current_block {
        10043043949733653460 => {
            if args_has(args, 'A' as i32 as u_char) != 0 {
                if !sname.is_null() {
                    as_0 = session_find(sname);
                } else {
                    as_0 = (*target).s;
                }
                if !as_0.is_null() {
                    retval = cmd_attach_session(
                        item,
                        (*as_0).name,
                        args_has(args, 'D' as i32 as u_char),
                        args_has(args, 'X' as i32 as u_char),
                        0 as ::core::ffi::c_int,
                        args_get(args, 'c' as i32 as u_char),
                        args_has(args, 'E' as i32 as u_char),
                        args_get(args, 'f' as i32 as u_char),
                    );
                    free(wname as *mut ::core::ffi::c_void);
                    free(sname as *mut ::core::ffi::c_void);
                    return retval;
                }
            }
            if !sname.is_null() && !session_find(sname).is_null() {
                cmdq_error(
                    item,
                    b"duplicate session: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    sname,
                );
            } else {
                group = args_get(args, 't' as i32 as u_char);
                if !group.is_null() {
                    groupwith = (*target).s;
                    if groupwith.is_null() {
                        sg = session_group_find(group);
                    } else {
                        sg = session_group_contains(groupwith);
                    }
                    if !sg.is_null() {
                        prefix = xstrdup((*sg).name);
                        current_block = 6717214610478484138;
                    } else if !groupwith.is_null() {
                        prefix = xstrdup((*groupwith).name);
                        current_block = 6717214610478484138;
                    } else if check_name(group) == 0 {
                        cmdq_error(
                            item,
                            b"invalid session group name: %s\0" as *const u8
                                as *const ::core::ffi::c_char,
                            group,
                        );
                        current_block = 5193972633326621385;
                    } else {
                        prefix = clean_name(group, 0 as ::core::ffi::c_int);
                        current_block = 6717214610478484138;
                    }
                } else {
                    current_block = 6717214610478484138;
                }
                match current_block {
                    5193972633326621385 => {}
                    _ => {
                        detached = args_has(args, 'd' as i32 as u_char);
                        if c.is_null() {
                            detached = 1 as ::core::ffi::c_int;
                        } else if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                            is_control = 1 as ::core::ffi::c_int;
                        }
                        already_attached = 0 as ::core::ffi::c_int;
                        if !c.is_null() && !(*c).session.is_null() {
                            already_attached = 1 as ::core::ffi::c_int;
                        }
                        tmp = args_get(args, 'c' as i32 as u_char);
                        if !tmp.is_null() {
                            cwd = format_single(
                                item,
                                tmp,
                                c,
                                ::core::ptr::null_mut::<session>(),
                                ::core::ptr::null_mut::<winlink>(),
                                ::core::ptr::null_mut::<window_pane>(),
                            );
                        } else {
                            cwd = xstrdup(server_client_get_cwd(
                                c,
                                ::core::ptr::null_mut::<session>(),
                            ));
                        }
                        if detached == 0
                            && already_attached == 0
                            && (*c).fd != -(1 as ::core::ffi::c_int)
                            && !(*c).flags & CLIENT_CONTROL as uint64_t != 0
                        {
                            if server_client_check_nested(cmdq_get_client(item)) != 0 {
                                cmdq_error(
                                    item,
                                    b"sessions should be nested with care, unset $TMUX to force\0"
                                        as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                current_block = 5193972633326621385;
                            } else {
                                if tcgetattr((*c).fd, &raw mut tio) != 0 as ::core::ffi::c_int {
                                    fatal(
                                        b"tcgetattr failed\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                    );
                                }
                                tiop = &raw mut tio;
                                current_block = 6545907279487748450;
                            }
                        } else {
                            tiop = ::core::ptr::null_mut::<termios>();
                            current_block = 6545907279487748450;
                        }
                        match current_block {
                            5193972633326621385 => {}
                            _ => {
                                if detached == 0 && already_attached == 0 {
                                    if server_client_open(c, &raw mut cause)
                                        != 0 as ::core::ffi::c_int
                                    {
                                        cmdq_error(
                                            item,
                                            b"open terminal failed: %s\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            cause,
                                        );
                                        free(cause as *mut ::core::ffi::c_void);
                                        current_block = 5193972633326621385;
                                    } else {
                                        current_block = 5181772461570869434;
                                    }
                                } else {
                                    current_block = 5181772461570869434;
                                }
                                match current_block {
                                    5193972633326621385 => {}
                                    _ => {
                                        if args_has(args, 'x' as i32 as u_char) != 0 {
                                            tmp = args_get(args, 'x' as i32 as u_char);
                                            if strcmp(
                                                tmp,
                                                b"-\0" as *const u8 as *const ::core::ffi::c_char,
                                            ) == 0 as ::core::ffi::c_int
                                            {
                                                if !c.is_null() {
                                                    dsx = (*c).tty.sx;
                                                } else {
                                                    dsx = 80 as u_int;
                                                }
                                                current_block = 5873035170358615968;
                                            } else {
                                                dsx = strtonum(
                                                    tmp,
                                                    1 as ::core::ffi::c_longlong,
                                                    USHRT_MAX as ::core::ffi::c_longlong,
                                                    &raw mut errstr,
                                                )
                                                    as u_int;
                                                if !errstr.is_null() {
                                                    cmdq_error(
                                                        item,
                                                        b"width %s\0" as *const u8
                                                            as *const ::core::ffi::c_char,
                                                        errstr,
                                                    );
                                                    current_block = 5193972633326621385;
                                                } else {
                                                    current_block = 5873035170358615968;
                                                }
                                            }
                                        } else {
                                            dsx = 80 as u_int;
                                            current_block = 5873035170358615968;
                                        }
                                        match current_block {
                                            5193972633326621385 => {}
                                            _ => {
                                                if args_has(args, 'y' as i32 as u_char) != 0 {
                                                    tmp = args_get(args, 'y' as i32 as u_char);
                                                    if strcmp(
                                                        tmp,
                                                        b"-\0" as *const u8
                                                            as *const ::core::ffi::c_char,
                                                    ) == 0 as ::core::ffi::c_int
                                                    {
                                                        if !c.is_null() {
                                                            dsy = (*c).tty.sy;
                                                        } else {
                                                            dsy = 24 as u_int;
                                                        }
                                                        current_block = 15855550149339537395;
                                                    } else {
                                                        dsy = strtonum(
                                                            tmp,
                                                            1 as ::core::ffi::c_longlong,
                                                            USHRT_MAX as ::core::ffi::c_longlong,
                                                            &raw mut errstr,
                                                        )
                                                            as u_int;
                                                        if !errstr.is_null() {
                                                            cmdq_error(
                                                                item,
                                                                b"height %s\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                errstr,
                                                            );
                                                            current_block = 5193972633326621385;
                                                        } else {
                                                            current_block = 15855550149339537395;
                                                        }
                                                    }
                                                } else {
                                                    dsy = 24 as u_int;
                                                    current_block = 15855550149339537395;
                                                }
                                                match current_block {
                                                    5193972633326621385 => {}
                                                    _ => {
                                                        if detached == 0 && is_control == 0 {
                                                            sx = (*c).tty.sx;
                                                            sy = (*c).tty.sy;
                                                            if sy > 0 as u_int
                                                                && options_get_number(
                                                                    global_s_options,
                                                                    b"status\0" as *const u8 as *const ::core::ffi::c_char,
                                                                ) != 0
                                                            {
                                                                sy = sy.wrapping_sub(1);
                                                            }
                                                        } else {
                                                            tmp = options_get_string(
                                                                global_s_options,
                                                                b"default-size\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                            );
                                                            if sscanf(
                                                                tmp,
                                                                b"%ux%u\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                &raw mut sx,
                                                                &raw mut sy,
                                                            ) != 2 as ::core::ffi::c_int
                                                            {
                                                                sx = dsx;
                                                                sy = dsy;
                                                            } else {
                                                                if args_has(
                                                                    args,
                                                                    'x' as i32 as u_char,
                                                                ) != 0
                                                                {
                                                                    sx = dsx;
                                                                }
                                                                if args_has(
                                                                    args,
                                                                    'y' as i32 as u_char,
                                                                ) != 0
                                                                {
                                                                    sy = dsy;
                                                                }
                                                            }
                                                        }
                                                        if sx == 0 as u_int {
                                                            sx = 1 as u_int;
                                                        }
                                                        if sy == 0 as u_int {
                                                            sy = 1 as u_int;
                                                        }
                                                        oo = options_create(global_s_options);
                                                        if args_has(args, 'x' as i32 as u_char) != 0
                                                            || args_has(args, 'y' as i32 as u_char)
                                                                != 0
                                                        {
                                                            if args_has(args, 'x' as i32 as u_char)
                                                                == 0
                                                            {
                                                                dsx = sx;
                                                            }
                                                            if args_has(args, 'y' as i32 as u_char)
                                                                == 0
                                                            {
                                                                dsy = sy;
                                                            }
                                                            options_set_string(
                                                                oo,
                                                                b"default-size\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                0 as ::core::ffi::c_int,
                                                                b"%ux%u\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                dsx,
                                                                dsy,
                                                            );
                                                        }
                                                        env = environ_create();
                                                        if !c.is_null()
                                                            && args_has(args, 'E' as i32 as u_char)
                                                                == 0
                                                        {
                                                            environ_update(
                                                                global_s_options,
                                                                (*c).environ,
                                                                env,
                                                            );
                                                        }
                                                        av = args_first_value(
                                                            args,
                                                            'e' as i32 as u_char,
                                                        );
                                                        while !av.is_null() {
                                                            environ_put(
                                                                env,
                                                                (*av).c2rust_unnamed.string,
                                                                0 as ::core::ffi::c_int,
                                                            );
                                                            av = args_next_value(av);
                                                        }
                                                        s = session_create(
                                                            prefix, sname, cwd, env, oo, tiop,
                                                        );
                                                        sc.item = item;
                                                        sc.s = s;
                                                        if detached == 0 {
                                                            sc.tc = c;
                                                        }
                                                        sc.name = wname;
                                                        args_to_vector(
                                                            args,
                                                            &raw mut sc.argc,
                                                            &raw mut sc.argv,
                                                        );
                                                        sc.idx = -(1 as ::core::ffi::c_int);
                                                        sc.cwd =
                                                            args_get(args, 'c' as i32 as u_char);
                                                        sc.flags = 0 as ::core::ffi::c_int;
                                                        if spawn_window(&raw mut sc, &raw mut cause)
                                                            .is_null()
                                                        {
                                                            session_destroy(
                                                                s,
                                                                0 as ::core::ffi::c_int,
                                                                b"cmd_new_session_exec\0"
                                                                    as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                            );
                                                            cmdq_error(
                                                                item,
                                                                b"create window failed: %s\0"
                                                                    as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                cause,
                                                            );
                                                            free(cause as *mut ::core::ffi::c_void);
                                                        } else {
                                                            if !group.is_null() {
                                                                if sg.is_null() {
                                                                    if !groupwith.is_null() {
                                                                        sg = session_group_new(
                                                                            (*groupwith).name,
                                                                        );
                                                                        session_group_add(
                                                                            sg, groupwith,
                                                                        );
                                                                    } else {
                                                                        sg = session_group_new(
                                                                            group,
                                                                        );
                                                                    }
                                                                }
                                                                session_group_add(sg, s);
                                                                session_group_synchronize_to(s);
                                                                session_select(
                                                                    s,
                                                                    (*winlinks_RB_MINMAX(
                                                                        &raw mut (*s).windows,
                                                                        RB_NEGINF,
                                                                    ))
                                                                    .idx,
                                                                );
                                                            }
                                                            events_fire_session(
                                                                b"session-created\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                s,
                                                            );
                                                            if detached == 0 {
                                                                if args_has(
                                                                    args,
                                                                    'f' as i32 as u_char,
                                                                ) != 0
                                                                {
                                                                    server_client_set_flags(
                                                                        c,
                                                                        args_get(
                                                                            args,
                                                                            'f' as i32 as u_char,
                                                                        ),
                                                                    );
                                                                }
                                                                if already_attached == 0 {
                                                                    if !(*c).flags
                                                                        & CLIENT_CONTROL as uint64_t
                                                                        != 0
                                                                    {
                                                                        proc_send(
                                                                            (*c).peer,
                                                                            MSG_READY,
                                                                            -(1 as ::core::ffi::c_int),
                                                                            ::core::ptr::null::<::core::ffi::c_void>(),
                                                                            0 as size_t,
                                                                        );
                                                                    }
                                                                } else if !(*c).session.is_null() {
                                                                    (*c).last_session =
                                                                        (*c).session;
                                                                }
                                                                server_client_set_session(c, s);
                                                                if !cmdq_get_flags(item)
                                                                    & CMDQ_STATE_REPEAT
                                                                    != 0
                                                                {
                                                                    server_client_set_key_table(
                                                                        c,
                                                                        ::core::ptr::null::<
                                                                            ::core::ffi::c_char,
                                                                        >(
                                                                        ),
                                                                    );
                                                                }
                                                            }
                                                            if args_has(args, 'P' as i32 as u_char)
                                                                != 0
                                                            {
                                                                template = args_get(
                                                                    args,
                                                                    'F' as i32 as u_char,
                                                                );
                                                                if template.is_null() {
                                                                    template = NEW_SESSION_TEMPLATE
                                                                        .as_ptr();
                                                                }
                                                                cp = format_single(
                                                                    item,
                                                                    template,
                                                                    c,
                                                                    s,
                                                                    (*s).curw,
                                                                    ::core::ptr::null_mut::<
                                                                        window_pane,
                                                                    >(
                                                                    ),
                                                                );
                                                                cmdq_print(
                                                                    item,
                                                                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                                                    cp,
                                                                );
                                                                free(
                                                                    cp as *mut ::core::ffi::c_void,
                                                                );
                                                            }
                                                            if detached == 0 {
                                                                (*c).flags |=
                                                                    CLIENT_ATTACHED as uint64_t;
                                                            }
                                                            if args_has(args, 'd' as i32 as u_char)
                                                                == 0
                                                            {
                                                                cmd_find_from_session(
                                                                    current,
                                                                    s,
                                                                    0 as ::core::ffi::c_int,
                                                                );
                                                            }
                                                            cmd_find_from_session(
                                                                &raw mut fs,
                                                                s,
                                                                0 as ::core::ffi::c_int,
                                                            );
                                                            cmdq_insert_hook(
                                                                s,
                                                                item,
                                                                &raw mut fs,
                                                                b"after-new-session\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                            );
                                                            if cfg_finished != 0 {
                                                                cfg_show_causes(s);
                                                            }
                                                            if !sc.argv.is_null() {
                                                                cmd_free_argv(sc.argc, sc.argv);
                                                            }
                                                            free(cwd as *mut ::core::ffi::c_void);
                                                            free(wname as *mut ::core::ffi::c_void);
                                                            free(sname as *mut ::core::ffi::c_void);
                                                            free(
                                                                prefix as *mut ::core::ffi::c_void,
                                                            );
                                                            return CMD_RETURN_NORMAL;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    if !sc.argv.is_null() {
        cmd_free_argv(sc.argc, sc.argv);
    }
    free(cwd as *mut ::core::ffi::c_void);
    free(wname as *mut ::core::ffi::c_void);
    free(sname as *mut ::core::ffi::c_void);
    free(prefix as *mut ::core::ffi::c_void);
    return CMD_RETURN_ERROR;
}
pub const __SHRT_MAX__: ::core::ffi::c_int = 32767 as ::core::ffi::c_int;

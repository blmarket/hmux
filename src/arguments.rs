pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::arguments::*;
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
    pub type cmds;
    pub type window_pane_prompt;
    pub type prompt;
    pub type format_tree;
    pub type options;
    pub type menu_data;
    pub type environ;
    pub type tmuxpeer;
    pub type input_request;
    pub type redraw_scene;
    pub type tty_key;
    pub type tty_code;
    pub type format_job_tree;
    pub type control_state;
    pub type cmdq_list;
    pub type cmdq_item;
    pub type input_ctx;
    pub type spawn_editor_state;
    pub type cmd;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strcspn(
        __s: *const ::core::ffi::c_char,
        __reject: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_ulong;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strlcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn strtonum(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(_: *mut ::core::ffi::c_void, _: size_t) -> *mut ::core::ffi::c_void;
    fn xrecallocarray(
        _: *mut ::core::ffi::c_void,
        _: size_t,
        _: size_t,
        _: size_t,
    ) -> *mut ::core::ffi::c_void;
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
    fn format_single_from_target(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn cmd_find_copy_state(_: *mut cmd_find_state, _: *mut cmd_find_state);
    fn cmd_log_argv(
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn cmd_append_argv(
        _: *mut ::core::ffi::c_int,
        _: *mut *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
    );
    fn cmd_get_entry(_: *mut cmd) -> *const cmd_entry;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmd_get_source(_: *mut cmd, _: *mut *const ::core::ffi::c_char, _: *mut u_int);
    fn cmd_list_copy(
        _: *const cmd_list,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut cmd_list;
    fn cmd_list_free(_: *mut cmd_list);
    fn cmd_list_print(_: *const cmd_list, _: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn cmd_list_first(_: *mut cmd_list) -> *mut cmd;
    fn cmd_template_replace(
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn cmd_parse_from_string(
        _: *const ::core::ffi::c_char,
        _: *mut cmd_parse_input,
    ) -> *mut cmd_parse_result;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn server_client_unref(_: *mut client);
    fn utf8_stravis(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> size_t;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
}
pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2RustUnnamed = 8;
pub const _ISpunct: C2RustUnnamed = 4;
pub const _IScntrl: C2RustUnnamed = 2;
pub const _ISblank: C2RustUnnamed = 1;
pub const _ISgraph: C2RustUnnamed = 32768;
pub const _ISprint: C2RustUnnamed = 16384;
pub const _ISspace: C2RustUnnamed = 8192;
pub const _ISxdigit: C2RustUnnamed = 4096;
pub const _ISdigit: C2RustUnnamed = 2048;
pub const _ISalpha: C2RustUnnamed = 1024;
pub const _ISlower: C2RustUnnamed = 512;
pub const _ISupper: C2RustUnnamed = 256;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args {
    pub tree: args_tree,
    pub count: u_int,
    pub values: *mut args_value,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_value {
    pub type_0: args_type,
    pub c2rust_unnamed: C2RustUnnamed_12,
    pub cached: *mut ::core::ffi::c_char,
    pub entry: C2RustUnnamed_11,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_11 {
    pub tqe_next: *mut args_value,
    pub tqe_prev: *mut *mut args_value,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_12 {
    pub string: *mut ::core::ffi::c_char,
    pub cmdlist: *mut cmd_list,
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
pub struct args_tree {
    pub rbh_root: *mut args_entry,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_entry {
    pub flag: u_char,
    pub values: args_values,
    pub count: u_int,
    pub flags: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_13,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_13 {
    pub rbe_left: *mut args_entry,
    pub rbe_right: *mut args_entry,
    pub rbe_parent: *mut args_entry,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_values {
    pub tqh_first: *mut args_value,
    pub tqh_last: *mut *mut args_value,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_command_state {
    pub cmdlist: *mut cmd_list,
    pub cmd: *mut ::core::ffi::c_char,
    pub pi: cmd_parse_input,
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
    pub modes: C2RustUnnamed_18,
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
    pub entry: C2RustUnnamed_17,
    pub sentry: C2RustUnnamed_16,
    pub zentry: C2RustUnnamed_15,
    pub tree_entry: C2RustUnnamed_14,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_14 {
    pub rbe_left: *mut window_pane,
    pub rbe_right: *mut window_pane,
    pub rbe_parent: *mut window_pane,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_15 {
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_16 {
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_17 {
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_18 {
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
    pub entry: C2RustUnnamed_19,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_19 {
    pub tqe_next: *mut window_mode_entry,
    pub tqe_prev: *mut *mut window_mode_entry,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_20 {
    pub offset: u_int,
    pub data: C2RustUnnamed_21,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_21 {
    pub attr: u_char,
    pub fg: u_char,
    pub bg: u_char,
    pub data: u_char,
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
pub struct winlink {
    pub idx: ::core::ffi::c_int,
    pub session: *mut session,
    pub window: *mut window,
    pub flags: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_24,
    pub wentry: C2RustUnnamed_23,
    pub sentry: C2RustUnnamed_22,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_22 {
    pub tqe_next: *mut winlink,
    pub tqe_prev: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_23 {
    pub tqe_next: *mut winlink,
    pub tqe_prev: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_24 {
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
    pub alerts_entry: C2RustUnnamed_27,
    pub options: *mut options,
    pub references: u_int,
    pub winlinks: C2RustUnnamed_26,
    pub entry: C2RustUnnamed_25,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_25 {
    pub rbe_left: *mut window,
    pub rbe_right: *mut window,
    pub rbe_parent: *mut window,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_26 {
    pub tqh_first: *mut winlink,
    pub tqh_last: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_27 {
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
    pub entry: C2RustUnnamed_28,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_28 {
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
pub struct window_panes {
    pub tqh_first: *mut window_pane,
    pub tqh_last: *mut *mut window_pane,
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
    pub gentry: C2RustUnnamed_30,
    pub entry: C2RustUnnamed_29,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_29 {
    pub rbe_left: *mut session,
    pub rbe_right: *mut session,
    pub rbe_parent: *mut session,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_30 {
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
pub struct winlink_stack {
    pub tqh_first: *mut winlink,
    pub tqh_last: *mut *mut winlink,
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
    pub entry: C2RustUnnamed_31,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_31 {
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
    pub entry: C2RustUnnamed_32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_32 {
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
pub struct key_table {
    pub name: *const ::core::ffi::c_char,
    pub activity_time: timeval,
    pub key_bindings: key_bindings,
    pub default_key_bindings: key_bindings,
    pub references: u_int,
    pub entry: C2RustUnnamed_33,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_33 {
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
    pub entry: C2RustUnnamed_34,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_34 {
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
    pub entry: C2RustUnnamed_37,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
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
pub struct cmd_parse_result {
    pub status: cmd_parse_status,
    pub cmdlist: *mut cmd_list,
    pub error: *mut ::core::ffi::c_char,
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
pub const VIS_OCTAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VIS_CSTYLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VIS_TAB: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const VIS_NL: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const VIS_DQ: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const ARGS_ENTRY_OPTIONAL_VALUE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
unsafe extern "C" fn args_tree_RB_INSERT(
    mut head: *mut args_tree,
    mut elm: *mut args_entry,
) -> *mut args_entry {
    let mut tmp: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut parent: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = args_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<args_entry>();
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
    args_tree_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<args_entry>();
}
unsafe extern "C" fn args_tree_RB_FIND(
    mut head: *mut args_tree,
    mut elm: *mut args_entry,
) -> *mut args_entry {
    let mut tmp: *mut args_entry = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = args_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<args_entry>();
}
unsafe extern "C" fn args_tree_RB_INSERT_COLOR(mut head: *mut args_tree, mut elm: *mut args_entry) {
    let mut parent: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut gparent: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut tmp: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
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
unsafe extern "C" fn args_tree_RB_NEXT(mut elm: *mut args_entry) -> *mut args_entry {
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
unsafe extern "C" fn args_tree_RB_REMOVE(
    mut head: *mut args_tree,
    mut elm: *mut args_entry,
) -> *mut args_entry {
    let mut current_block: u64;
    let mut child: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut parent: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut old: *mut args_entry = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
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
        current_block = 10138408787860760292;
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
        args_tree_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn args_tree_RB_REMOVE_COLOR(
    mut head: *mut args_tree,
    mut parent: *mut args_entry,
    mut elm: *mut args_entry,
) {
    let mut tmp: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
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
                    let mut oleft: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
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
                    let mut oright: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
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
unsafe extern "C" fn args_tree_RB_MINMAX(
    mut head: *mut args_tree,
    mut val: ::core::ffi::c_int,
) -> *mut args_entry {
    let mut tmp: *mut args_entry = (*head).rbh_root;
    let mut parent: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
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
unsafe extern "C" fn args_cmp(
    mut a1: *mut args_entry,
    mut a2: *mut args_entry,
) -> ::core::ffi::c_int {
    return (*a1).flag as ::core::ffi::c_int - (*a2).flag as ::core::ffi::c_int;
}
unsafe extern "C" fn args_find(mut args: *mut args, mut flag: u_char) -> *mut args_entry {
    let mut entry: args_entry = args_entry {
        flag: 0,
        values: args_values {
            tqh_first: ::core::ptr::null_mut::<args_value>(),
            tqh_last: ::core::ptr::null_mut::<*mut args_value>(),
        },
        count: 0,
        flags: 0,
        entry: C2RustUnnamed_13 {
            rbe_left: ::core::ptr::null_mut::<args_entry>(),
            rbe_right: ::core::ptr::null_mut::<args_entry>(),
            rbe_parent: ::core::ptr::null_mut::<args_entry>(),
            rbe_color: 0,
        },
    };
    entry.flag = flag;
    return args_tree_RB_FIND(&raw mut (*args).tree, &raw mut entry);
}
unsafe extern "C" fn args_copy_value(mut to: *mut args_value, mut from: *mut args_value) {
    (*to).type_0 = (*from).type_0;
    match (*from).type_0 as ::core::ffi::c_uint {
        2 => {
            (*to).c2rust_unnamed.cmdlist = (*from).c2rust_unnamed.cmdlist;
            (*(*to).c2rust_unnamed.cmdlist).references += 1;
        }
        1 => {
            (*to).c2rust_unnamed.string = xstrdup((*from).c2rust_unnamed.string);
        }
        0 | _ => {}
    };
}
unsafe extern "C" fn args_type_to_string(mut type_0: args_type) -> *const ::core::ffi::c_char {
    match type_0 as ::core::ffi::c_uint {
        0 => return b"NONE\0" as *const u8 as *const ::core::ffi::c_char,
        1 => return b"STRING\0" as *const u8 as *const ::core::ffi::c_char,
        2 => return b"COMMANDS\0" as *const u8 as *const ::core::ffi::c_char,
        _ => {}
    }
    return b"INVALID\0" as *const u8 as *const ::core::ffi::c_char;
}
unsafe extern "C" fn args_value_as_string(
    mut value: *mut args_value,
) -> *const ::core::ffi::c_char {
    match (*value).type_0 as ::core::ffi::c_uint {
        0 => return b"\0" as *const u8 as *const ::core::ffi::c_char,
        2 => {
            if (*value).cached.is_null() {
                (*value).cached =
                    cmd_list_print((*value).c2rust_unnamed.cmdlist, 0 as ::core::ffi::c_int);
            }
            return (*value).cached;
        }
        1 => return (*value).c2rust_unnamed.string,
        _ => {}
    }
    fatalx(b"unexpected argument type\0" as *const u8 as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn args_create() -> *mut args {
    let mut args: *mut args = ::core::ptr::null_mut::<args>();
    args = xcalloc(1 as size_t, ::core::mem::size_of::<args>() as size_t) as *mut args;
    (*args).tree.rbh_root = ::core::ptr::null_mut::<args_entry>();
    return args;
}
unsafe extern "C" fn args_parse_flag_argument(
    mut values: *mut args_value,
    mut count: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
    mut args: *mut args,
    mut i: *mut u_int,
    mut string: *const ::core::ffi::c_char,
    mut flag: ::core::ffi::c_int,
    mut optional_argument: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut argument: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut new: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut as_0: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    new = xcalloc(1 as size_t, ::core::mem::size_of::<args_value>() as size_t) as *mut args_value;
    if *string as ::core::ffi::c_int != '\0' as i32 {
        (*new).type_0 = ARGS_STRING;
        (*new).c2rust_unnamed.string = xstrdup(string);
    } else {
        if *i == count {
            argument = ::core::ptr::null_mut::<args_value>();
        } else {
            argument = values.offset(*i as isize) as *mut args_value;
            if (*argument).type_0 as ::core::ffi::c_uint
                != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                xasprintf(
                    cause,
                    b"-%c argument must be a string\0" as *const u8 as *const ::core::ffi::c_char,
                    flag,
                );
                args_free_value(new);
                free(new as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
        }
        if argument.is_null() {
            args_free_value(new);
            free(new as *mut ::core::ffi::c_void);
            if optional_argument != 0 {
                log_debug(
                    b"%s: -%c (optional)\0" as *const u8 as *const ::core::ffi::c_char,
                    b"args_parse_flag_argument\0" as *const u8 as *const ::core::ffi::c_char,
                    flag,
                );
                args_set(
                    args,
                    flag as u_char,
                    ::core::ptr::null_mut::<args_value>(),
                    ARGS_ENTRY_OPTIONAL_VALUE,
                );
                return 0 as ::core::ffi::c_int;
            }
            xasprintf(
                cause,
                b"-%c expects an argument\0" as *const u8 as *const ::core::ffi::c_char,
                flag,
            );
            return -(1 as ::core::ffi::c_int);
        }
        if optional_argument != 0
            && (*argument).type_0 as ::core::ffi::c_uint
                == ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            as_0 = (*argument).c2rust_unnamed.string;
            if *as_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '-' as i32
                && (*as_0.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '-' as i32
                    || *(*__ctype_b_loc())
                        .offset(*as_0.offset(1 as ::core::ffi::c_int as isize) as u_char
                            as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        & _ISalpha as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        != 0)
            {
                args_free_value(new);
                free(new as *mut ::core::ffi::c_void);
                log_debug(
                    b"%s: -%c (optional)\0" as *const u8 as *const ::core::ffi::c_char,
                    b"args_parse_flag_argument\0" as *const u8 as *const ::core::ffi::c_char,
                    flag,
                );
                args_set(
                    args,
                    flag as u_char,
                    ::core::ptr::null_mut::<args_value>(),
                    ARGS_ENTRY_OPTIONAL_VALUE,
                );
                return 0 as ::core::ffi::c_int;
            }
        }
        args_copy_value(new, argument);
        *i = (*i).wrapping_add(1);
    }
    s = args_value_as_string(new);
    log_debug(
        b"%s: -%c = %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_parse_flag_argument\0" as *const u8 as *const ::core::ffi::c_char,
        flag,
        s,
    );
    args_set(args, flag as u_char, new, 0 as ::core::ffi::c_int);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn args_parse_flags(
    mut parse: *const args_parse,
    mut values: *mut args_value,
    mut count: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
    mut args: *mut args,
    mut i: *mut u_int,
) -> ::core::ffi::c_int {
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut flag: u_char = 0;
    let mut found: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut string: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut optional_argument: ::core::ffi::c_int = 0;
    value = values.offset(*i as isize) as *mut args_value;
    if (*value).type_0 as ::core::ffi::c_uint
        != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 1 as ::core::ffi::c_int;
    }
    string = (*value).c2rust_unnamed.string;
    log_debug(
        b"%s: next %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_parse_flags\0" as *const u8 as *const ::core::ffi::c_char,
        string,
    );
    let fresh1 = string;
    string = string.offset(1);
    if *fresh1 as ::core::ffi::c_int != '-' as i32 || *string as ::core::ffi::c_int == '\0' as i32 {
        return 1 as ::core::ffi::c_int;
    }
    *i = (*i).wrapping_add(1);
    if *string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '-' as i32
        && *string.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
    {
        return 1 as ::core::ffi::c_int;
    }
    loop {
        let fresh2 = string;
        string = string.offset(1);
        flag = *fresh2 as u_char;
        if flag as ::core::ffi::c_int == '\0' as i32 {
            return 0 as ::core::ffi::c_int;
        }
        if flag as ::core::ffi::c_int == '?' as i32 {
            return -(1 as ::core::ffi::c_int);
        }
        if *(*__ctype_b_loc()).offset(flag as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            == 0
        {
            xasprintf(
                cause,
                b"invalid flag -%c\0" as *const u8 as *const ::core::ffi::c_char,
                flag as ::core::ffi::c_int,
            );
            return -(1 as ::core::ffi::c_int);
        }
        found = strchr((*parse).template, flag as ::core::ffi::c_int);
        if found.is_null() {
            xasprintf(
                cause,
                b"unknown flag -%c\0" as *const u8 as *const ::core::ffi::c_char,
                flag as ::core::ffi::c_int,
            );
            return -(1 as ::core::ffi::c_int);
        }
        if *found.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ':' as i32 {
            log_debug(
                b"%s: -%c\0" as *const u8 as *const ::core::ffi::c_char,
                b"args_parse_flags\0" as *const u8 as *const ::core::ffi::c_char,
                flag as ::core::ffi::c_int,
            );
            args_set(
                args,
                flag,
                ::core::ptr::null_mut::<args_value>(),
                0 as ::core::ffi::c_int,
            );
        } else {
            optional_argument = (*found.offset(2 as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                == ':' as i32) as ::core::ffi::c_int;
            return args_parse_flag_argument(
                values,
                count,
                cause,
                args,
                i,
                string,
                flag as ::core::ffi::c_int,
                optional_argument,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn args_parse(
    mut parse: *const args_parse,
    mut values: *mut args_value,
    mut count: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut args {
    let mut args: *mut args = ::core::ptr::null_mut::<args>();
    let mut i: u_int = 0;
    let mut type_0: args_parse_type = ARGS_PARSE_INVALID;
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut new: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut stop: ::core::ffi::c_int = 0;
    if count == 0 as u_int {
        return args_create();
    }
    args = args_create();
    i = 1 as u_int;
    while i < count {
        stop = args_parse_flags(parse, values, count, cause, args, &raw mut i);
        if stop == -(1 as ::core::ffi::c_int) {
            args_free(args);
            return ::core::ptr::null_mut::<args>();
        }
        if stop == 1 as ::core::ffi::c_int {
            break;
        }
    }
    log_debug(
        b"%s: flags end at %u of %u\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_parse\0" as *const u8 as *const ::core::ffi::c_char,
        i,
        count,
    );
    if i != count {
        while i < count {
            value = values.offset(i as isize) as *mut args_value;
            s = args_value_as_string(value);
            log_debug(
                b"%s: %u = %s (type %s)\0" as *const u8 as *const ::core::ffi::c_char,
                b"args_parse\0" as *const u8 as *const ::core::ffi::c_char,
                i,
                s,
                args_type_to_string((*value).type_0),
            );
            if (*parse).cb.is_some() {
                type_0 =
                    (*parse).cb.expect("non-null function pointer")(args, (*args).count, cause);
                if type_0 as ::core::ffi::c_uint
                    == ARGS_PARSE_INVALID as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    args_free(args);
                    return ::core::ptr::null_mut::<args>();
                }
            } else {
                type_0 = ARGS_PARSE_STRING;
            }
            (*args).values = xrecallocarray(
                (*args).values as *mut ::core::ffi::c_void,
                (*args).count as size_t,
                (*args).count.wrapping_add(1 as u_int) as size_t,
                ::core::mem::size_of::<args_value>() as size_t,
            ) as *mut args_value;
            let fresh0 = (*args).count;
            (*args).count = (*args).count.wrapping_add(1);
            new = (*args).values.offset(fresh0 as isize) as *mut args_value;
            match type_0 as ::core::ffi::c_uint {
                0 => {
                    fatalx(
                        b"unexpected argument type\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                1 => {
                    if (*value).type_0 as ::core::ffi::c_uint
                        != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        xasprintf(
                            cause,
                            b"argument %u must be \"string\"\0" as *const u8
                                as *const ::core::ffi::c_char,
                            (*args).count,
                        );
                        args_free(args);
                        return ::core::ptr::null_mut::<args>();
                    }
                    args_copy_value(new, value);
                }
                2 => {
                    args_copy_value(new, value);
                }
                3 => {
                    if (*value).type_0 as ::core::ffi::c_uint
                        != ARGS_COMMANDS as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        xasprintf(
                            cause,
                            b"argument %u must be { commands }\0" as *const u8
                                as *const ::core::ffi::c_char,
                            (*args).count,
                        );
                        args_free(args);
                        return ::core::ptr::null_mut::<args>();
                    }
                    args_copy_value(new, value);
                }
                _ => {}
            }
            i = i.wrapping_add(1);
        }
    }
    if (*parse).lower != -(1 as ::core::ffi::c_int) && (*args).count < (*parse).lower as u_int {
        xasprintf(
            cause,
            b"too few arguments (need at least %u)\0" as *const u8 as *const ::core::ffi::c_char,
            (*parse).lower,
        );
        args_free(args);
        return ::core::ptr::null_mut::<args>();
    }
    if (*parse).upper != -(1 as ::core::ffi::c_int) && (*args).count > (*parse).upper as u_int {
        xasprintf(
            cause,
            b"too many arguments (need at most %u)\0" as *const u8 as *const ::core::ffi::c_char,
            (*parse).upper,
        );
        args_free(args);
        return ::core::ptr::null_mut::<args>();
    }
    return args;
}
unsafe extern "C" fn args_copy_copy_value(
    mut to: *mut args_value,
    mut from: *mut args_value,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0;
    (*to).type_0 = (*from).type_0;
    match (*from).type_0 as ::core::ffi::c_uint {
        1 => {
            expanded = xstrdup((*from).c2rust_unnamed.string);
            i = 0 as ::core::ffi::c_int;
            while i < argc {
                s = cmd_template_replace(
                    expanded,
                    *argv.offset(i as isize),
                    i + 1 as ::core::ffi::c_int,
                );
                free(expanded as *mut ::core::ffi::c_void);
                expanded = s;
                i += 1;
            }
            (*to).c2rust_unnamed.string = expanded;
        }
        2 => {
            (*to).c2rust_unnamed.cmdlist =
                cmd_list_copy((*from).c2rust_unnamed.cmdlist, argc, argv) as *mut cmd_list;
        }
        0 | _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn args_copy(
    mut args: *mut args,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) -> *mut args {
    let mut new_args: *mut args = ::core::ptr::null_mut::<args>();
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut new_value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut i: u_int = 0;
    cmd_log_argv(
        argc,
        argv,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_copy\0" as *const u8 as *const ::core::ffi::c_char,
    );
    new_args = args_create();
    entry = args_tree_RB_MINMAX(&raw mut (*args).tree, RB_NEGINF);
    while !entry.is_null() {
        if (*entry).values.tqh_first.is_null() {
            i = 0 as u_int;
            while i < (*entry).count {
                args_set(
                    new_args,
                    (*entry).flag,
                    ::core::ptr::null_mut::<args_value>(),
                    0 as ::core::ffi::c_int,
                );
                i = i.wrapping_add(1);
            }
        } else {
            value = (*entry).values.tqh_first;
            while !value.is_null() {
                new_value = xcalloc(1 as size_t, ::core::mem::size_of::<args_value>() as size_t)
                    as *mut args_value;
                args_copy_copy_value(new_value, value, argc, argv);
                args_set(new_args, (*entry).flag, new_value, 0 as ::core::ffi::c_int);
                value = (*value).entry.tqe_next;
            }
        }
        entry = args_tree_RB_NEXT(entry);
    }
    if (*args).count == 0 as u_int {
        return new_args;
    }
    (*new_args).count = (*args).count;
    (*new_args).values = xcalloc(
        (*args).count as size_t,
        ::core::mem::size_of::<args_value>() as size_t,
    ) as *mut args_value;
    i = 0 as u_int;
    while i < (*args).count {
        new_value = (*new_args).values.offset(i as isize) as *mut args_value;
        args_copy_copy_value(
            new_value,
            (*args).values.offset(i as isize) as *mut args_value,
            argc,
            argv,
        );
        i = i.wrapping_add(1);
    }
    return new_args;
}
#[no_mangle]
pub unsafe extern "C" fn args_free_value(mut value: *mut args_value) {
    match (*value).type_0 as ::core::ffi::c_uint {
        1 => {
            free((*value).c2rust_unnamed.string as *mut ::core::ffi::c_void);
        }
        2 => {
            cmd_list_free((*value).c2rust_unnamed.cmdlist as *mut cmd_list);
        }
        0 | _ => {}
    }
    free((*value).cached as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn args_free_values(mut values: *mut args_value, mut count: u_int) {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < count {
        args_free_value(values.offset(i as isize) as *mut args_value);
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn args_free(mut args: *mut args) {
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut entry1: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut value1: *mut args_value = ::core::ptr::null_mut::<args_value>();
    args_free_values((*args).values, (*args).count);
    free((*args).values as *mut ::core::ffi::c_void);
    entry = args_tree_RB_MINMAX(&raw mut (*args).tree, RB_NEGINF);
    while !entry.is_null() && {
        entry1 = args_tree_RB_NEXT(entry);
        1 as ::core::ffi::c_int != 0
    } {
        args_tree_RB_REMOVE(&raw mut (*args).tree, entry);
        value = (*entry).values.tqh_first;
        while !value.is_null() && {
            value1 = (*value).entry.tqe_next;
            1 as ::core::ffi::c_int != 0
        } {
            if !(*value).entry.tqe_next.is_null() {
                (*(*value).entry.tqe_next).entry.tqe_prev = (*value).entry.tqe_prev;
            } else {
                (*entry).values.tqh_last = (*value).entry.tqe_prev;
            }
            *(*value).entry.tqe_prev = (*value).entry.tqe_next;
            args_free_value(value);
            free(value as *mut ::core::ffi::c_void);
            value = value1;
        }
        free(entry as *mut ::core::ffi::c_void);
        entry = entry1;
    }
    free(args as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn args_to_vector(
    mut args: *mut args,
    mut argc: *mut ::core::ffi::c_int,
    mut argv: *mut *mut *mut ::core::ffi::c_char,
) {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    *argc = 0 as ::core::ffi::c_int;
    *argv = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    i = 0 as u_int;
    while i < (*args).count {
        match (*(*args).values.offset(i as isize)).type_0 as ::core::ffi::c_uint {
            1 => {
                cmd_append_argv(
                    argc,
                    argv,
                    (*(*args).values.offset(i as isize)).c2rust_unnamed.string,
                );
            }
            2 => {
                s = cmd_list_print(
                    (*(*args).values.offset(i as isize)).c2rust_unnamed.cmdlist,
                    0 as ::core::ffi::c_int,
                );
                cmd_append_argv(argc, argv, s);
                free(s as *mut ::core::ffi::c_void);
            }
            0 | _ => {}
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn args_from_vector(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) -> *mut args_value {
    let mut values: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut i: ::core::ffi::c_int = 0;
    values = xcalloc(
        argc as size_t,
        ::core::mem::size_of::<args_value>() as size_t,
    ) as *mut args_value;
    i = 0 as ::core::ffi::c_int;
    while i < argc {
        (*values.offset(i as isize)).type_0 = ARGS_STRING;
        let ref mut fresh3 = (*values.offset(i as isize)).c2rust_unnamed.string;
        *fresh3 = xstrdup(*argv.offset(i as isize));
        i += 1;
    }
    return values;
}
unsafe extern "C" fn args_print_add(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut len: *mut size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut slen: size_t = 0;
    ap = args.clone();
    slen = xvasprintf(&raw mut s, fmt, ap) as size_t;
    *len = (*len).wrapping_add(slen);
    *buf = xrealloc(*buf as *mut ::core::ffi::c_void, *len) as *mut ::core::ffi::c_char;
    strlcat(*buf, s, *len);
    free(s as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn args_print_add_value(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut len: *mut size_t,
    mut value: *mut args_value,
) {
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if **buf as ::core::ffi::c_int != '\0' as i32 {
        args_print_add(buf, len, b" \0" as *const u8 as *const ::core::ffi::c_char);
    }
    match (*value).type_0 as ::core::ffi::c_uint {
        2 => {
            expanded = cmd_list_print((*value).c2rust_unnamed.cmdlist, 0 as ::core::ffi::c_int);
            args_print_add(
                buf,
                len,
                b"{ %s }\0" as *const u8 as *const ::core::ffi::c_char,
                expanded,
            );
        }
        1 => {
            expanded = args_escape((*value).c2rust_unnamed.string);
            args_print_add(
                buf,
                len,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                expanded,
            );
        }
        0 | _ => {}
    }
    free(expanded as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn args_print(mut args: *mut args) -> *mut ::core::ffi::c_char {
    let mut len: size_t = 0;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut last: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    len = 1 as size_t;
    buf = xcalloc(1 as size_t, len) as *mut ::core::ffi::c_char;
    entry = args_tree_RB_MINMAX(&raw mut (*args).tree, RB_NEGINF);
    while !entry.is_null() {
        if !((*entry).flags & ARGS_ENTRY_OPTIONAL_VALUE != 0) {
            if (*entry).values.tqh_first.is_null() {
                if *buf as ::core::ffi::c_int == '\0' as i32 {
                    args_print_add(
                        &raw mut buf,
                        &raw mut len,
                        b"-\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                j = 0 as u_int;
                while j < (*entry).count {
                    args_print_add(
                        &raw mut buf,
                        &raw mut len,
                        b"%c\0" as *const u8 as *const ::core::ffi::c_char,
                        (*entry).flag as ::core::ffi::c_int,
                    );
                    j = j.wrapping_add(1);
                }
            }
        }
        entry = args_tree_RB_NEXT(entry);
    }
    entry = args_tree_RB_MINMAX(&raw mut (*args).tree, RB_NEGINF);
    while !entry.is_null() {
        if (*entry).flags & ARGS_ENTRY_OPTIONAL_VALUE != 0 {
            if *buf as ::core::ffi::c_int != '\0' as i32 {
                args_print_add(
                    &raw mut buf,
                    &raw mut len,
                    b" -%c\0" as *const u8 as *const ::core::ffi::c_char,
                    (*entry).flag as ::core::ffi::c_int,
                );
            } else {
                args_print_add(
                    &raw mut buf,
                    &raw mut len,
                    b"-%c\0" as *const u8 as *const ::core::ffi::c_char,
                    (*entry).flag as ::core::ffi::c_int,
                );
            }
            last = entry;
        } else if !(*entry).values.tqh_first.is_null() {
            value = (*entry).values.tqh_first;
            while !value.is_null() {
                if *buf as ::core::ffi::c_int != '\0' as i32 {
                    args_print_add(
                        &raw mut buf,
                        &raw mut len,
                        b" -%c\0" as *const u8 as *const ::core::ffi::c_char,
                        (*entry).flag as ::core::ffi::c_int,
                    );
                } else {
                    args_print_add(
                        &raw mut buf,
                        &raw mut len,
                        b"-%c\0" as *const u8 as *const ::core::ffi::c_char,
                        (*entry).flag as ::core::ffi::c_int,
                    );
                }
                args_print_add_value(&raw mut buf, &raw mut len, value);
                value = (*value).entry.tqe_next;
            }
            last = entry;
        }
        entry = args_tree_RB_NEXT(entry);
    }
    if !last.is_null() && (*last).flags & ARGS_ENTRY_OPTIONAL_VALUE != 0 {
        args_print_add(
            &raw mut buf,
            &raw mut len,
            b" --\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    i = 0 as u_int;
    while i < (*args).count {
        args_print_add_value(
            &raw mut buf,
            &raw mut len,
            (*args).values.offset(i as isize) as *mut args_value,
        );
        i = i.wrapping_add(1);
    }
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn args_escape(
    mut s: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    static mut dquoted: [::core::ffi::c_char; 9] =
        unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b" #';${}%\0") };
    static mut squoted: [::core::ffi::c_char; 3] =
        unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b" \"\0") };
    let mut escaped: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut result: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut flags: ::core::ffi::c_int = 0;
    let mut quotes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *s as ::core::ffi::c_int == '\0' as i32 {
        xasprintf(
            &raw mut result,
            b"''\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return result;
    }
    if *s.offset(strcspn(s, &raw const dquoted as *const ::core::ffi::c_char) as isize)
        as ::core::ffi::c_int
        != '\0' as i32
    {
        quotes = '"' as i32;
    } else if *s.offset(strcspn(s, &raw const squoted as *const ::core::ffi::c_char) as isize)
        as ::core::ffi::c_int
        != '\0' as i32
    {
        quotes = '\'' as i32;
    }
    if *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ' ' as i32
        && *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
        && (quotes != 0 as ::core::ffi::c_int
            || *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '~' as i32)
    {
        xasprintf(
            &raw mut escaped,
            b"\\%c\0" as *const u8 as *const ::core::ffi::c_char,
            *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
        );
        return escaped;
    }
    flags = VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL;
    if quotes == '"' as i32 {
        flags |= VIS_DQ;
    }
    utf8_stravis(&raw mut escaped, s, flags);
    if quotes == '\'' as i32 {
        xasprintf(
            &raw mut result,
            b"'%s'\0" as *const u8 as *const ::core::ffi::c_char,
            escaped,
        );
    } else if quotes == '"' as i32 {
        if *escaped as ::core::ffi::c_int == '~' as i32 {
            xasprintf(
                &raw mut result,
                b"\"\\%s\"\0" as *const u8 as *const ::core::ffi::c_char,
                escaped,
            );
        } else {
            xasprintf(
                &raw mut result,
                b"\"%s\"\0" as *const u8 as *const ::core::ffi::c_char,
                escaped,
            );
        }
    } else if *escaped as ::core::ffi::c_int == '~' as i32 {
        xasprintf(
            &raw mut result,
            b"\\%s\0" as *const u8 as *const ::core::ffi::c_char,
            escaped,
        );
    } else {
        result = xstrdup(escaped);
    }
    free(escaped as *mut ::core::ffi::c_void);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn args_has(mut args: *mut args, mut flag: u_char) -> ::core::ffi::c_int {
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    entry = args_find(args, flag);
    if entry.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*entry).count as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn args_set(
    mut args: *mut args,
    mut flag: u_char,
    mut value: *mut args_value,
    mut flags: ::core::ffi::c_int,
) {
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    entry = args_find(args, flag);
    if entry.is_null() {
        entry =
            xcalloc(1 as size_t, ::core::mem::size_of::<args_entry>() as size_t) as *mut args_entry;
        (*entry).flag = flag;
        (*entry).count = 1 as u_int;
        (*entry).flags = flags;
        (*entry).values.tqh_first = ::core::ptr::null_mut::<args_value>();
        (*entry).values.tqh_last = &raw mut (*entry).values.tqh_first;
        args_tree_RB_INSERT(&raw mut (*args).tree, entry);
    } else {
        (*entry).count = (*entry).count.wrapping_add(1);
    }
    if !value.is_null()
        && (*value).type_0 as ::core::ffi::c_uint
            != ARGS_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*value).entry.tqe_next = ::core::ptr::null_mut::<args_value>();
        (*value).entry.tqe_prev = (*entry).values.tqh_last;
        *(*entry).values.tqh_last = value;
        (*entry).values.tqh_last = &raw mut (*value).entry.tqe_next;
    } else {
        free(value as *mut ::core::ffi::c_void);
    };
}
#[no_mangle]
pub unsafe extern "C" fn args_get(
    mut args: *mut args,
    mut flag: u_char,
) -> *const ::core::ffi::c_char {
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    entry = args_find(args, flag);
    if entry.is_null() {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if (*entry).values.tqh_first.is_null() {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return (**(*((*entry).values.tqh_last as *mut args_values)).tqh_last)
        .c2rust_unnamed
        .string;
}
#[no_mangle]
pub unsafe extern "C" fn args_first(
    mut args: *mut args,
    mut entry: *mut *mut args_entry,
) -> u_char {
    *entry = args_tree_RB_MINMAX(&raw mut (*args).tree, RB_NEGINF);
    if (*entry).is_null() {
        return 0 as u_char;
    }
    return (**entry).flag;
}
#[no_mangle]
pub unsafe extern "C" fn args_next(mut entry: *mut *mut args_entry) -> u_char {
    *entry = args_tree_RB_NEXT(*entry);
    if (*entry).is_null() {
        return 0 as u_char;
    }
    return (**entry).flag;
}
#[no_mangle]
pub unsafe extern "C" fn args_count(mut args: *mut args) -> u_int {
    return (*args).count;
}
#[no_mangle]
pub unsafe extern "C" fn args_values(mut args: *mut args) -> *mut args_value {
    return (*args).values;
}
#[no_mangle]
pub unsafe extern "C" fn args_value(mut args: *mut args, mut idx: u_int) -> *mut args_value {
    if idx >= (*args).count {
        return ::core::ptr::null_mut::<args_value>();
    }
    return (*args).values.offset(idx as isize) as *mut args_value;
}
#[no_mangle]
pub unsafe extern "C" fn args_string(
    mut args: *mut args,
    mut idx: u_int,
) -> *const ::core::ffi::c_char {
    if idx >= (*args).count {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return args_value_as_string((*args).values.offset(idx as isize) as *mut args_value);
}
#[no_mangle]
pub unsafe extern "C" fn args_make_commands_now(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
    mut idx: u_int,
    mut expand: ::core::ffi::c_int,
) -> *mut cmd_list {
    let mut state: *mut args_command_state = ::core::ptr::null_mut::<args_command_state>();
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    state = args_make_commands_prepare(
        self_0,
        item,
        idx,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
        expand,
    );
    cmdlist = args_make_commands(
        state,
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        &raw mut error,
    );
    if cmdlist.is_null() {
        cmdq_error(
            item,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            error,
        );
        free(error as *mut ::core::ffi::c_void);
    }
    args_make_commands_free(state);
    return cmdlist;
}
#[no_mangle]
pub unsafe extern "C" fn args_make_commands_prepare(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
    mut idx: u_int,
    mut default_command: *const ::core::ffi::c_char,
    mut wait: ::core::ffi::c_int,
    mut expand: ::core::ffi::c_int,
) -> *mut args_command_state {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut state: *mut args_command_state = ::core::ptr::null_mut::<args_command_state>();
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut file: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    state = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<args_command_state>() as size_t,
    ) as *mut args_command_state;
    if idx < (*args).count {
        value = (*args).values.offset(idx as isize) as *mut args_value;
        if (*value).type_0 as ::core::ffi::c_uint
            == ARGS_COMMANDS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*state).cmdlist = (*value).c2rust_unnamed.cmdlist as *mut cmd_list;
            (*(*state).cmdlist).references += 1;
            return state;
        }
        cmd = (*value).c2rust_unnamed.string;
    } else {
        if default_command.is_null() {
            fatalx(b"argument out of range\0" as *const u8 as *const ::core::ffi::c_char);
        }
        cmd = default_command;
    }
    if expand != 0 {
        (*state).cmd = format_single_from_target(item, cmd);
    } else {
        (*state).cmd = xstrdup(cmd);
    }
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_make_commands_prepare\0" as *const u8 as *const ::core::ffi::c_char,
        (*state).cmd,
    );
    if wait != 0 {
        (*state).pi.item = item;
    }
    cmd_get_source(self_0, &raw mut file, &raw mut (*state).pi.line);
    if !file.is_null() {
        (*state).pi.file = xstrdup(file);
    }
    (*state).pi.c = tc;
    if !(*state).pi.c.is_null() {
        (*(*state).pi.c).references += 1;
    }
    cmd_find_copy_state(&raw mut (*state).pi.fs, target);
    return state;
}
#[no_mangle]
pub unsafe extern "C" fn args_make_commands(
    mut state: *mut args_command_state,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
    mut error: *mut *mut ::core::ffi::c_char,
) -> *mut cmd_list {
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut new_cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0;
    if !(*state).cmdlist.is_null() {
        if argc == 0 as ::core::ffi::c_int {
            (*(*state).cmdlist).references += 1;
            return (*state).cmdlist;
        }
        return cmd_list_copy((*state).cmdlist, argc, argv);
    }
    cmd = xstrdup((*state).cmd);
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_make_commands\0" as *const u8 as *const ::core::ffi::c_char,
        cmd,
    );
    cmd_log_argv(
        argc,
        argv,
        b"args_make_commands\0" as *const u8 as *const ::core::ffi::c_char,
    );
    i = 0 as ::core::ffi::c_int;
    while i < argc {
        new_cmd = cmd_template_replace(cmd, *argv.offset(i as isize), i + 1 as ::core::ffi::c_int);
        log_debug(
            b"%s: %%%u %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"args_make_commands\0" as *const u8 as *const ::core::ffi::c_char,
            i + 1 as ::core::ffi::c_int,
            *argv.offset(i as isize),
            new_cmd,
        );
        free(cmd as *mut ::core::ffi::c_void);
        cmd = new_cmd;
        i += 1;
    }
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"args_make_commands\0" as *const u8 as *const ::core::ffi::c_char,
        cmd,
    );
    pr = cmd_parse_from_string(cmd, &raw mut (*state).pi);
    free(cmd as *mut ::core::ffi::c_void);
    match (*pr).status as ::core::ffi::c_uint {
        0 => {
            *error = (*pr).error;
            return ::core::ptr::null_mut::<cmd_list>();
        }
        1 => return (*pr).cmdlist,
        _ => {}
    }
    fatalx(b"invalid parse return state\0" as *const u8 as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn args_make_commands_free(mut state: *mut args_command_state) {
    if !(*state).cmdlist.is_null() {
        cmd_list_free((*state).cmdlist);
    }
    if !(*state).pi.c.is_null() {
        server_client_unref((*state).pi.c);
    }
    free((*state).pi.file as *mut ::core::ffi::c_void);
    free((*state).cmd as *mut ::core::ffi::c_void);
    free(state as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn args_make_commands_get_command(
    mut state: *mut args_command_state,
) -> *mut ::core::ffi::c_char {
    let mut first: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut n: ::core::ffi::c_int = 0;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(*state).cmdlist.is_null() {
        first = cmd_list_first((*state).cmdlist);
        if first.is_null() {
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        }
        return xstrdup((*cmd_get_entry(first)).name);
    }
    n = strcspn(
        (*state).cmd,
        b" ,\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    xasprintf(
        &raw mut s,
        b"%.*s\0" as *const u8 as *const ::core::ffi::c_char,
        n,
        (*state).cmd,
    );
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn args_first_value(
    mut args: *mut args,
    mut flag: u_char,
) -> *mut args_value {
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    entry = args_find(args, flag);
    if entry.is_null() {
        return ::core::ptr::null_mut::<args_value>();
    }
    return (*entry).values.tqh_first;
}
#[no_mangle]
pub unsafe extern "C" fn args_next_value(mut value: *mut args_value) -> *mut args_value {
    return (*value).entry.tqe_next;
}
#[no_mangle]
pub unsafe extern "C" fn args_strtonum(
    mut args: *mut args,
    mut flag: u_char,
    mut minval: ::core::ffi::c_longlong,
    mut maxval: ::core::ffi::c_longlong,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ll: ::core::ffi::c_longlong = 0;
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    entry = args_find(args, flag);
    if entry.is_null() {
        *cause = xstrdup(b"missing\0" as *const u8 as *const ::core::ffi::c_char);
        return 0 as ::core::ffi::c_longlong;
    }
    value = *(*((*entry).values.tqh_last as *mut args_values)).tqh_last;
    if value.is_null()
        || (*value).type_0 as ::core::ffi::c_uint
            != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*value).c2rust_unnamed.string.is_null()
    {
        *cause = xstrdup(b"missing\0" as *const u8 as *const ::core::ffi::c_char);
        return 0 as ::core::ffi::c_longlong;
    }
    ll = strtonum(
        (*value).c2rust_unnamed.string,
        minval,
        maxval,
        &raw mut errstr,
    );
    if !errstr.is_null() {
        *cause = xstrdup(errstr);
        return 0 as ::core::ffi::c_longlong;
    }
    *cause = ::core::ptr::null_mut::<::core::ffi::c_char>();
    return ll;
}
#[no_mangle]
pub unsafe extern "C" fn args_strtonum_and_expand(
    mut args: *mut args,
    mut flag: u_char,
    mut minval: ::core::ffi::c_longlong,
    mut maxval: ::core::ffi::c_longlong,
    mut item: *mut cmdq_item,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut formatted: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ll: ::core::ffi::c_longlong = 0;
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut value: *mut args_value = ::core::ptr::null_mut::<args_value>();
    entry = args_find(args, flag);
    if entry.is_null() {
        *cause = xstrdup(b"missing\0" as *const u8 as *const ::core::ffi::c_char);
        return 0 as ::core::ffi::c_longlong;
    }
    value = *(*((*entry).values.tqh_last as *mut args_values)).tqh_last;
    if value.is_null()
        || (*value).type_0 as ::core::ffi::c_uint
            != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*value).c2rust_unnamed.string.is_null()
    {
        *cause = xstrdup(b"missing\0" as *const u8 as *const ::core::ffi::c_char);
        return 0 as ::core::ffi::c_longlong;
    }
    formatted = format_single_from_target(item, (*value).c2rust_unnamed.string);
    ll = strtonum(formatted, minval, maxval, &raw mut errstr);
    free(formatted as *mut ::core::ffi::c_void);
    if !errstr.is_null() {
        *cause = xstrdup(errstr);
        return 0 as ::core::ffi::c_longlong;
    }
    *cause = ::core::ptr::null_mut::<::core::ffi::c_char>();
    return ll;
}
#[no_mangle]
pub unsafe extern "C" fn args_percentage(
    mut args: *mut args,
    mut flag: u_char,
    mut minval: ::core::ffi::c_longlong,
    mut maxval: ::core::ffi::c_longlong,
    mut curval: ::core::ffi::c_longlong,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    entry = args_find(args, flag);
    if entry.is_null() {
        *cause = xstrdup(b"missing\0" as *const u8 as *const ::core::ffi::c_char);
        return 0 as ::core::ffi::c_longlong;
    }
    if (*entry).values.tqh_first.is_null() {
        *cause = xstrdup(b"empty\0" as *const u8 as *const ::core::ffi::c_char);
        return 0 as ::core::ffi::c_longlong;
    }
    value = (**(*((*entry).values.tqh_last as *mut args_values)).tqh_last)
        .c2rust_unnamed
        .string;
    return args_string_percentage(value, minval, maxval, curval, cause);
}
#[no_mangle]
pub unsafe extern "C" fn args_string_percentage(
    mut value: *const ::core::ffi::c_char,
    mut minval: ::core::ffi::c_longlong,
    mut maxval: ::core::ffi::c_longlong,
    mut curval: ::core::ffi::c_longlong,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ll: ::core::ffi::c_longlong = 0;
    let mut valuelen: size_t = strlen(value);
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if valuelen == 0 as size_t {
        *cause = xstrdup(b"empty\0" as *const u8 as *const ::core::ffi::c_char);
        return 0 as ::core::ffi::c_longlong;
    }
    if *value.offset(valuelen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
        == '%' as i32
    {
        copy = xstrdup(value);
        *copy.offset(valuelen.wrapping_sub(1 as size_t) as isize) =
            '\0' as i32 as ::core::ffi::c_char;
        ll = strtonum(
            copy,
            0 as ::core::ffi::c_longlong,
            1000 as ::core::ffi::c_longlong,
            &raw mut errstr,
        );
        free(copy as *mut ::core::ffi::c_void);
        if !errstr.is_null() {
            *cause = xstrdup(errstr);
            return 0 as ::core::ffi::c_longlong;
        }
        ll = curval * ll / 100 as ::core::ffi::c_longlong;
        if ll < minval {
            *cause = xstrdup(b"too small\0" as *const u8 as *const ::core::ffi::c_char);
            return 0 as ::core::ffi::c_longlong;
        }
        if ll > maxval {
            *cause = xstrdup(b"too large\0" as *const u8 as *const ::core::ffi::c_char);
            return 0 as ::core::ffi::c_longlong;
        }
    } else {
        ll = strtonum(value, minval, maxval, &raw mut errstr);
        if !errstr.is_null() {
            *cause = xstrdup(errstr);
            return 0 as ::core::ffi::c_longlong;
        }
    }
    *cause = ::core::ptr::null_mut::<::core::ffi::c_char>();
    return ll;
}
#[no_mangle]
pub unsafe extern "C" fn args_percentage_and_expand(
    mut args: *mut args,
    mut flag: u_char,
    mut minval: ::core::ffi::c_longlong,
    mut maxval: ::core::ffi::c_longlong,
    mut curval: ::core::ffi::c_longlong,
    mut item: *mut cmdq_item,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut entry: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    entry = args_find(args, flag);
    if entry.is_null() {
        *cause = xstrdup(b"missing\0" as *const u8 as *const ::core::ffi::c_char);
        return 0 as ::core::ffi::c_longlong;
    }
    if (*entry).values.tqh_first.is_null() {
        *cause = xstrdup(b"empty\0" as *const u8 as *const ::core::ffi::c_char);
        return 0 as ::core::ffi::c_longlong;
    }
    value = (**(*((*entry).values.tqh_last as *mut args_values)).tqh_last)
        .c2rust_unnamed
        .string;
    return args_string_percentage_and_expand(value, minval, maxval, curval, item, cause);
}
#[no_mangle]
pub unsafe extern "C" fn args_string_percentage_and_expand(
    mut value: *const ::core::ffi::c_char,
    mut minval: ::core::ffi::c_longlong,
    mut maxval: ::core::ffi::c_longlong,
    mut curval: ::core::ffi::c_longlong,
    mut item: *mut cmdq_item,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ll: ::core::ffi::c_longlong = 0;
    let mut valuelen: size_t = strlen(value);
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut f: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *value.offset(valuelen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
        == '%' as i32
    {
        copy = xstrdup(value);
        *copy.offset(valuelen.wrapping_sub(1 as size_t) as isize) =
            '\0' as i32 as ::core::ffi::c_char;
        f = format_single_from_target(item, copy);
        ll = strtonum(
            f,
            0 as ::core::ffi::c_longlong,
            1000 as ::core::ffi::c_longlong,
            &raw mut errstr,
        );
        free(f as *mut ::core::ffi::c_void);
        free(copy as *mut ::core::ffi::c_void);
        if !errstr.is_null() {
            *cause = xstrdup(errstr);
            return 0 as ::core::ffi::c_longlong;
        }
        ll = curval * ll / 100 as ::core::ffi::c_longlong;
        if ll < minval {
            *cause = xstrdup(b"too small\0" as *const u8 as *const ::core::ffi::c_char);
            return 0 as ::core::ffi::c_longlong;
        }
        if ll > maxval {
            *cause = xstrdup(b"too large\0" as *const u8 as *const ::core::ffi::c_char);
            return 0 as ::core::ffi::c_longlong;
        }
    } else {
        f = format_single_from_target(item, value);
        ll = strtonum(f, minval, maxval, &raw mut errstr);
        free(f as *mut ::core::ffi::c_void);
        if !errstr.is_null() {
            *cause = xstrdup(errstr);
            return 0 as ::core::ffi::c_longlong;
        }
    }
    *cause = ::core::ptr::null_mut::<::core::ffi::c_char>();
    return ll;
}

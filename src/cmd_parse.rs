pub use crate::src::shared::variadic::{__builtin_va_list, __gnuc_va_list, __va_list_tag, va_list};
pub use crate::src::shared::stdio::{
    FILE, _IO_FILE, _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data,
};
pub use crate::src::shared::abi::{__gid_t, __off64_t, __off_t, __uid_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{
    CMD_PARSE_MAX_ENVIRON_LEN, CMD_PARSE_NOALIAS, CMD_PARSE_ONEGROUP, CMD_PARSE_PARSEONLY,
    CMD_PARSE_VERBOSE,
};
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
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;
use ::libc;
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
    pub type cmdq_state;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn getpwuid(__uid: __uid_t) -> *mut passwd;
    fn getpwnam(__name: *const ::core::ffi::c_char) -> *mut passwd;
    fn wctomb(__s: *mut ::core::ffi::c_char, __wchar: wchar_t) -> ::core::ffi::c_int;
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
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn getuid() -> __uid_t;
    fn sscanf(
        __s: *const ::core::ffi::c_char,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn getc(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn ungetc(__c: ::core::ffi::c_int, __stream: *mut FILE) -> ::core::ffi::c_int;
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
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
    static mut global_environ: *mut environ;
    fn format_true(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn format_create(
        _: *mut client,
        _: *mut cmdq_item,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut format_tree;
    fn format_free(_: *mut format_tree);
    fn format_expand(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn format_defaults(
        _: *mut format_tree,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    );
    fn environ_find(_: *mut environ, _: *const ::core::ffi::c_char) -> *mut environ_entry;
    fn environ_put(_: *mut environ, _: *const ::core::ffi::c_char, _: ::core::ffi::c_int);
    fn args_free_value(_: *mut args_value);
    fn cmd_find_valid_state(_: *mut cmd_find_state) -> ::core::ffi::c_int;
    fn cmd_find_from_client(
        _: *mut cmd_find_state,
        _: *mut client,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_get_alias(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn cmd_parse(
        _: *mut args_value,
        _: u_int,
        _: *const ::core::ffi::c_char,
        _: u_int,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut cmd;
    fn cmd_list_new() -> *mut cmd_list;
    fn cmd_list_append(_: *mut cmd_list, _: *mut cmd);
    fn cmd_list_append_all(_: *mut cmd_list, _: *mut cmd_list);
    fn cmd_list_move(_: *mut cmd_list, _: *mut cmd_list);
    fn cmd_list_free(_: *mut cmd_list);
    fn cmd_list_print(_: *const cmd_list, _: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn cmdq_get_command(_: *mut cmd_list, _: *mut cmdq_state) -> *mut cmdq_item;
    fn cmdq_insert_after(_: *mut cmdq_item, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_append(_: *mut client, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_print(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
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
pub struct passwd {
    pub pw_name: *mut ::core::ffi::c_char,
    pub pw_passwd: *mut ::core::ffi::c_char,
    pub pw_uid: __uid_t,
    pub pw_gid: __gid_t,
    pub pw_gecos: *mut ::core::ffi::c_char,
    pub pw_dir: *mut ::core::ffi::c_char,
    pub pw_shell: *mut ::core::ffi::c_char,
}
pub type wchar_t = ::libc::wchar_t;
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
    pub entry: C2RustUnnamed_11,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_11 {
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
    pub entry: C2RustUnnamed_12,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_12 {
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
pub union C2RustUnnamed_13 {
    pub offset: u_int,
    pub data: C2RustUnnamed_14,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_14 {
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
    pub gentry: C2RustUnnamed_16,
    pub entry: C2RustUnnamed_15,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_15 {
    pub rbe_left: *mut session,
    pub rbe_right: *mut session,
    pub rbe_parent: *mut session,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_16 {
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
    pub entry: C2RustUnnamed_19,
    pub wentry: C2RustUnnamed_18,
    pub sentry: C2RustUnnamed_17,
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
    pub tqe_next: *mut winlink,
    pub tqe_prev: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_19 {
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
    pub alerts_entry: C2RustUnnamed_22,
    pub options: *mut options,
    pub references: u_int,
    pub winlinks: C2RustUnnamed_21,
    pub entry: C2RustUnnamed_20,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_20 {
    pub rbe_left: *mut window,
    pub rbe_right: *mut window,
    pub rbe_parent: *mut window,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_21 {
    pub tqh_first: *mut winlink,
    pub tqh_last: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_22 {
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
    pub entry: C2RustUnnamed_23,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_23 {
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
    pub modes: C2RustUnnamed_28,
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
    pub entry: C2RustUnnamed_27,
    pub sentry: C2RustUnnamed_26,
    pub zentry: C2RustUnnamed_25,
    pub tree_entry: C2RustUnnamed_24,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_24 {
    pub rbe_left: *mut window_pane,
    pub rbe_right: *mut window_pane,
    pub rbe_parent: *mut window_pane,
    pub rbe_color: ::core::ffi::c_int,
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
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_28 {
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
    pub entry: C2RustUnnamed_29,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_29 {
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
    pub entry: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub le_next: *mut tty_term,
    pub le_prev: *mut *mut tty_term,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct environ_entry {
    pub name: *mut ::core::ffi::c_char,
    pub value: *mut ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_36,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_36 {
    pub rbe_left: *mut environ_entry,
    pub rbe_right: *mut environ_entry,
    pub rbe_parent: *mut environ_entry,
    pub rbe_color: ::core::ffi::c_int,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_parse_commands {
    pub tqh_first: *mut cmd_parse_command,
    pub tqh_last: *mut *mut cmd_parse_command,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_parse_command {
    pub line: u_int,
    pub arguments: cmd_parse_arguments,
    pub entry: C2RustUnnamed_39,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_39 {
    pub tqe_next: *mut cmd_parse_command,
    pub tqe_prev: *mut *mut cmd_parse_command,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_parse_arguments {
    pub tqh_first: *mut cmd_parse_argument,
    pub tqh_last: *mut *mut cmd_parse_argument,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_parse_argument {
    pub type_0: cmd_parse_argument_type,
    pub string: *mut ::core::ffi::c_char,
    pub commands: *mut cmd_parse_commands,
    pub cmdlist: *mut cmd_list,
    pub entry: C2RustUnnamed_40,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_40 {
    pub tqe_next: *mut cmd_parse_argument,
    pub tqe_prev: *mut *mut cmd_parse_argument,
}
pub type cmd_parse_argument_type = ::core::ffi::c_uint;
pub const CMD_PARSE_PARSED_COMMANDS: cmd_parse_argument_type = 2;
pub const CMD_PARSE_COMMANDS: cmd_parse_argument_type = 1;
pub const CMD_PARSE_STRING: cmd_parse_argument_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_parse_state {
    pub f: *mut FILE,
    pub buf: *const ::core::ffi::c_char,
    pub len: size_t,
    pub off: size_t,
    pub condition: ::core::ffi::c_int,
    pub eol: ::core::ffi::c_int,
    pub eof: ::core::ffi::c_int,
    pub input: *mut cmd_parse_input,
    pub escapes: u_int,
    pub error: *mut ::core::ffi::c_char,
    pub commands: *mut cmd_parse_commands,
    pub scope: *mut cmd_parse_scope,
    pub stack: C2RustUnnamed_41,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_41 {
    pub tqh_first: *mut cmd_parse_scope,
    pub tqh_last: *mut *mut cmd_parse_scope,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_parse_scope {
    pub flag: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_42,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_42 {
    pub tqe_next: *mut cmd_parse_scope,
    pub tqe_prev: *mut *mut cmd_parse_scope,
}
pub type yy_state_t = yytype_int8;
pub type yytype_int8 = ::core::ffi::c_schar;
#[derive(Copy, Clone)]
#[repr(C)]
pub union YYSTYPE {
    pub token: *mut ::core::ffi::c_char,
    pub arguments: *mut cmd_parse_arguments,
    pub argument: *mut cmd_parse_argument,
    pub flag: ::core::ffi::c_int,
    pub elif: C2RustUnnamed_43,
    pub commands: *mut cmd_parse_commands,
    pub command: *mut cmd_parse_command,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_43 {
    pub flag: ::core::ffi::c_int,
    pub commands: *mut cmd_parse_commands,
}
pub type yysymbol_kind_t = ::core::ffi::c_int;
pub const YYSYMBOL_argument_statements: yysymbol_kind_t = 37;
pub const YYSYMBOL_argument: yysymbol_kind_t = 36;
pub const YYSYMBOL_arguments: yysymbol_kind_t = 35;
pub const YYSYMBOL_elif1: yysymbol_kind_t = 34;
pub const YYSYMBOL_condition1: yysymbol_kind_t = 33;
pub const YYSYMBOL_command: yysymbol_kind_t = 32;
pub const YYSYMBOL_commands: yysymbol_kind_t = 31;
pub const YYSYMBOL_elif: yysymbol_kind_t = 30;
pub const YYSYMBOL_condition: yysymbol_kind_t = 29;
pub const YYSYMBOL_if_close: yysymbol_kind_t = 28;
pub const YYSYMBOL_if_elif: yysymbol_kind_t = 27;
pub const YYSYMBOL_if_else: yysymbol_kind_t = 26;
pub const YYSYMBOL_if_open: yysymbol_kind_t = 25;
pub const YYSYMBOL_hidden_assignment: yysymbol_kind_t = 24;
pub const YYSYMBOL_assignment: yysymbol_kind_t = 23;
pub const YYSYMBOL_optional_assignment: yysymbol_kind_t = 22;
pub const YYSYMBOL_expanded: yysymbol_kind_t = 21;
pub const YYSYMBOL_format: yysymbol_kind_t = 20;
pub const YYSYMBOL_statement: yysymbol_kind_t = 19;
pub const YYSYMBOL_statements: yysymbol_kind_t = 18;
pub const YYSYMBOL_lines: yysymbol_kind_t = 17;
pub const YYSYMBOL_YYACCEPT: yysymbol_kind_t = 16;
pub const YYSYMBOL_15_: yysymbol_kind_t = 15;
pub const YYSYMBOL_14_: yysymbol_kind_t = 14;
pub const YYSYMBOL_13_: yysymbol_kind_t = 13;
pub const YYSYMBOL_12_n_: yysymbol_kind_t = 12;
pub const YYSYMBOL_EQUALS: yysymbol_kind_t = 11;
pub const YYSYMBOL_TOKEN: yysymbol_kind_t = 10;
pub const YYSYMBOL_FORMAT: yysymbol_kind_t = 9;
pub const YYSYMBOL_ENDIF: yysymbol_kind_t = 8;
pub const YYSYMBOL_ELIF: yysymbol_kind_t = 7;
pub const YYSYMBOL_ELSE: yysymbol_kind_t = 6;
pub const YYSYMBOL_IF: yysymbol_kind_t = 5;
pub const YYSYMBOL_HIDDEN: yysymbol_kind_t = 4;
pub const YYSYMBOL_ERROR: yysymbol_kind_t = 3;
pub const YYSYMBOL_YYUNDEF: yysymbol_kind_t = 2;
pub const YYSYMBOL_YYerror: yysymbol_kind_t = 1;
pub const YYSYMBOL_YYEOF: yysymbol_kind_t = 0;
pub const YYSYMBOL_YYEMPTY: yysymbol_kind_t = -2;
pub type yy_state_fast_t = ::core::ffi::c_int;
pub type C2RustUnnamed_44 = ::core::ffi::c_uint;
pub const SINGLE_QUOTES: C2RustUnnamed_44 = 3;
pub const DOUBLE_QUOTES: C2RustUnnamed_44 = 2;
pub const NONE: C2RustUnnamed_44 = 1;
pub const START: C2RustUnnamed_44 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union yyalloc {
    pub yyss_alloc: yy_state_t,
    pub yyvs_alloc: YYSTYPE,
}
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const EOF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const ENVIRON_HIDDEN: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const FORMAT_NOJOBS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const FORMAT_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut parse_state: cmd_parse_state = cmd_parse_state {
    f: ::core::ptr::null::<FILE>() as *mut FILE,
    buf: ::core::ptr::null::<::core::ffi::c_char>(),
    len: 0,
    off: 0,
    condition: 0,
    eol: 0,
    eof: 0,
    input: ::core::ptr::null::<cmd_parse_input>() as *mut cmd_parse_input,
    escapes: 0,
    error: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
    commands: ::core::ptr::null::<cmd_parse_commands>() as *mut cmd_parse_commands,
    scope: ::core::ptr::null::<cmd_parse_scope>() as *mut cmd_parse_scope,
    stack: C2RustUnnamed_41 {
        tqh_first: ::core::ptr::null::<cmd_parse_scope>() as *mut cmd_parse_scope,
        tqh_last: ::core::ptr::null::<*mut cmd_parse_scope>() as *mut *mut cmd_parse_scope,
    },
};
pub const YYEMPTY: ::core::ffi::c_int = -(2 as ::core::ffi::c_int);
pub const YYEOF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const YYerror: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
pub const YYUNDEF: ::core::ffi::c_int = 257 as ::core::ffi::c_int;
pub const ERROR: ::core::ffi::c_int = 258 as ::core::ffi::c_int;
pub const HIDDEN: ::core::ffi::c_int = 259 as ::core::ffi::c_int;
pub const IF: ::core::ffi::c_int = 260 as ::core::ffi::c_int;
pub const ELSE: ::core::ffi::c_int = 261 as ::core::ffi::c_int;
pub const ELIF: ::core::ffi::c_int = 262 as ::core::ffi::c_int;
pub const ENDIF: ::core::ffi::c_int = 263 as ::core::ffi::c_int;
pub const FORMAT: ::core::ffi::c_int = 264 as ::core::ffi::c_int;
pub const TOKEN: ::core::ffi::c_int = 265 as ::core::ffi::c_int;
pub const EQUALS: ::core::ffi::c_int = 266 as ::core::ffi::c_int;
pub const YYSTACK_GAP_MAXIMUM: ::core::ffi::c_long =
    ::core::mem::size_of::<yyalloc>() as ::core::ffi::c_long - 1 as ::core::ffi::c_long;
unsafe extern "C" fn cmd_parse_get_error(
    mut file: *const ::core::ffi::c_char,
    mut line: u_int,
    mut error: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if file.is_null() {
        s = xstrdup(error);
    } else {
        xasprintf(
            &raw mut s,
            b"%s:%u: %s\0" as *const u8 as *const ::core::ffi::c_char,
            file,
            line,
            error,
        );
    }
    return s;
}
unsafe extern "C" fn cmd_parse_print_commands(
    mut pi: *mut cmd_parse_input,
    mut cmdlist: *mut cmd_list,
) {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*pi).item.is_null() || !(*pi).flags & CMD_PARSE_VERBOSE != 0 {
        return;
    }
    s = cmd_list_print(cmdlist, 0 as ::core::ffi::c_int);
    if !(*pi).file.is_null() {
        cmdq_print(
            (*pi).item,
            b"%s:%u: %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*pi).file,
            (*pi).line,
            s,
        );
    } else {
        cmdq_print(
            (*pi).item,
            b"%u: %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*pi).line,
            s,
        );
    }
    free(s as *mut ::core::ffi::c_void);
}
pub const YYFINAL: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const YYLAST: ::core::ffi::c_int = 104 as ::core::ffi::c_int;
unsafe extern "C" fn cmd_parse_free_argument(mut arg: *mut cmd_parse_argument) {
    match (*arg).type_0 as ::core::ffi::c_uint {
        0 => {
            free((*arg).string as *mut ::core::ffi::c_void);
        }
        1 => {
            cmd_parse_free_commands((*arg).commands as *mut cmd_parse_commands);
        }
        2 => {
            cmd_list_free((*arg).cmdlist);
        }
        _ => {}
    }
    free(arg as *mut ::core::ffi::c_void);
}
pub const YYNTOKENS: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const YYMAXUTOK: ::core::ffi::c_int = 266 as ::core::ffi::c_int;
unsafe extern "C" fn cmd_parse_free_arguments(mut args: *mut cmd_parse_arguments) {
    let mut arg: *mut cmd_parse_argument = ::core::ptr::null_mut::<cmd_parse_argument>();
    let mut arg1: *mut cmd_parse_argument = ::core::ptr::null_mut::<cmd_parse_argument>();
    arg = (*args).tqh_first;
    while !arg.is_null() && {
        arg1 = (*arg).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(*arg).entry.tqe_next.is_null() {
            (*(*arg).entry.tqe_next).entry.tqe_prev = (*arg).entry.tqe_prev;
        } else {
            (*args).tqh_last = (*arg).entry.tqe_prev;
        }
        *(*arg).entry.tqe_prev = (*arg).entry.tqe_next;
        cmd_parse_free_argument(arg);
        arg = arg1;
    }
}
static mut yytranslate: [yytype_int8; 267] = [
    0 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    13 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    14 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    15 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    4 as ::core::ffi::c_int as yytype_int8,
    5 as ::core::ffi::c_int as yytype_int8,
    6 as ::core::ffi::c_int as yytype_int8,
    7 as ::core::ffi::c_int as yytype_int8,
    8 as ::core::ffi::c_int as yytype_int8,
    9 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    11 as ::core::ffi::c_int as yytype_int8,
];
unsafe extern "C" fn cmd_parse_free_command(mut cmd: *mut cmd_parse_command) {
    cmd_parse_free_arguments(&raw mut (*cmd).arguments);
    free(cmd as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn cmd_parse_new_commands() -> *mut cmd_parse_commands {
    let mut cmds: *mut cmd_parse_commands = ::core::ptr::null_mut::<cmd_parse_commands>();
    cmds =
        xmalloc(::core::mem::size_of::<cmd_parse_commands>() as size_t) as *mut cmd_parse_commands;
    (*cmds).tqh_first = ::core::ptr::null_mut::<cmd_parse_command>();
    (*cmds).tqh_last = &raw mut (*cmds).tqh_first;
    return cmds;
}
unsafe extern "C" fn cmd_parse_free_commands(mut cmds: *mut cmd_parse_commands) {
    let mut cmd: *mut cmd_parse_command = ::core::ptr::null_mut::<cmd_parse_command>();
    let mut cmd1: *mut cmd_parse_command = ::core::ptr::null_mut::<cmd_parse_command>();
    cmd = (*cmds).tqh_first;
    while !cmd.is_null() && {
        cmd1 = (*cmd).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(*cmd).entry.tqe_next.is_null() {
            (*(*cmd).entry.tqe_next).entry.tqe_prev = (*cmd).entry.tqe_prev;
        } else {
            (*cmds).tqh_last = (*cmd).entry.tqe_prev;
        }
        *(*cmd).entry.tqe_prev = (*cmd).entry.tqe_next;
        cmd_parse_free_command(cmd);
        cmd = cmd1;
    }
    free(cmds as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn cmd_parse_run_parser(
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut cmd_parse_commands {
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    let mut scope: *mut cmd_parse_scope = ::core::ptr::null_mut::<cmd_parse_scope>();
    let mut scope1: *mut cmd_parse_scope = ::core::ptr::null_mut::<cmd_parse_scope>();
    let mut retval: ::core::ffi::c_int = 0;
    (*ps).commands = ::core::ptr::null_mut::<cmd_parse_commands>();
    (*ps).stack.tqh_first = ::core::ptr::null_mut::<cmd_parse_scope>();
    (*ps).stack.tqh_last = &raw mut (*ps).stack.tqh_first;
    retval = yyparse();
    scope = (*ps).stack.tqh_first;
    while !scope.is_null() && {
        scope1 = (*scope).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(*scope).entry.tqe_next.is_null() {
            (*(*scope).entry.tqe_next).entry.tqe_prev = (*scope).entry.tqe_prev;
        } else {
            (*ps).stack.tqh_last = (*scope).entry.tqe_prev;
        }
        *(*scope).entry.tqe_prev = (*scope).entry.tqe_next;
        free(scope as *mut ::core::ffi::c_void);
        scope = scope1;
    }
    if retval != 0 as ::core::ffi::c_int {
        *cause = (*ps).error;
        return ::core::ptr::null_mut::<cmd_parse_commands>();
    }
    if (*ps).commands.is_null() {
        return cmd_parse_new_commands();
    }
    return (*ps).commands;
}
unsafe extern "C" fn cmd_parse_do_file(
    mut f: *mut FILE,
    mut pi: *mut cmd_parse_input,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut cmd_parse_commands {
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    memset(
        ps as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_state>() as size_t,
    );
    (*ps).input = pi;
    (*ps).f = f;
    return cmd_parse_run_parser(cause);
}
pub const YYPACT_NINF: ::core::ffi::c_int = -(32 as ::core::ffi::c_int);
unsafe extern "C" fn cmd_parse_do_buffer(
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut pi: *mut cmd_parse_input,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut cmd_parse_commands {
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    memset(
        ps as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_state>() as size_t,
    );
    (*ps).input = pi;
    (*ps).buf = buf;
    (*ps).len = len;
    return cmd_parse_run_parser(cause);
}
static mut yypact: [yytype_int8; 75] = [
    49 as ::core::ffi::c_int as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    14 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    33 as ::core::ffi::c_int as yytype_int8,
    49 as ::core::ffi::c_int as yytype_int8,
    29 as ::core::ffi::c_int as yytype_int8,
    47 as ::core::ffi::c_int as yytype_int8,
    60 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    4 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    35 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    68 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    83 as ::core::ffi::c_int as yytype_int8,
    81 as ::core::ffi::c_int as yytype_int8,
    38 as ::core::ffi::c_int as yytype_int8,
    5 as ::core::ffi::c_int as yytype_int8,
    17 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    81 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    83 as ::core::ffi::c_int as yytype_int8,
    71 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    14 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    38 as ::core::ffi::c_int as yytype_int8,
    38 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(1 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    81 as ::core::ffi::c_int as yytype_int8,
    40 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    76 as ::core::ffi::c_int as yytype_int8,
    86 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(1 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    32 as ::core::ffi::c_int as yytype_int8,
    58 as ::core::ffi::c_int as yytype_int8,
    38 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    84 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    81 as ::core::ffi::c_int as yytype_int8,
    81 as ::core::ffi::c_int as yytype_int8,
    88 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    32 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    79 as ::core::ffi::c_int as yytype_int8,
    62 as ::core::ffi::c_int as yytype_int8,
    81 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    79 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
];
unsafe extern "C" fn cmd_parse_log_commands(
    mut cmds: *mut cmd_parse_commands,
    mut prefix: *const ::core::ffi::c_char,
) {
    let mut cmd: *mut cmd_parse_command = ::core::ptr::null_mut::<cmd_parse_command>();
    let mut arg: *mut cmd_parse_argument = ::core::ptr::null_mut::<cmd_parse_argument>();
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    i = 0 as u_int;
    cmd = (*cmds).tqh_first;
    while !cmd.is_null() {
        j = 0 as u_int;
        arg = (*cmd).arguments.tqh_first;
        while !arg.is_null() {
            match (*arg).type_0 as ::core::ffi::c_uint {
                0 => {
                    log_debug(
                        b"%s %u:%u: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        prefix,
                        i,
                        j,
                        (*arg).string,
                    );
                }
                1 => {
                    xasprintf(
                        &raw mut s,
                        b"%s %u:%u\0" as *const u8 as *const ::core::ffi::c_char,
                        prefix,
                        i,
                        j,
                    );
                    cmd_parse_log_commands((*arg).commands as *mut cmd_parse_commands, s);
                    free(s as *mut ::core::ffi::c_void);
                }
                2 => {
                    s = cmd_list_print((*arg).cmdlist, 0 as ::core::ffi::c_int);
                    log_debug(
                        b"%s %u:%u: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        prefix,
                        i,
                        j,
                        s,
                    );
                    free(s as *mut ::core::ffi::c_void);
                }
                _ => {}
            }
            j = j.wrapping_add(1);
            arg = (*arg).entry.tqe_next;
        }
        i = i.wrapping_add(1);
        cmd = (*cmd).entry.tqe_next;
    }
}
static mut yydefact: [yytype_int8; 75] = [
    2 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    15 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    32 as ::core::ffi::c_int as yytype_int8,
    7 as ::core::ffi::c_int as yytype_int8,
    13 as ::core::ffi::c_int as yytype_int8,
    8 as ::core::ffi::c_int as yytype_int8,
    9 as ::core::ffi::c_int as yytype_int8,
    27 as ::core::ffi::c_int as yytype_int8,
    31 as ::core::ffi::c_int as yytype_int8,
    16 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    11 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    17 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    4 as ::core::ffi::c_int as yytype_int8,
    33 as ::core::ffi::c_int as yytype_int8,
    6 as ::core::ffi::c_int as yytype_int8,
    13 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    28 as ::core::ffi::c_int as yytype_int8,
    5 as ::core::ffi::c_int as yytype_int8,
    43 as ::core::ffi::c_int as yytype_int8,
    44 as ::core::ffi::c_int as yytype_int8,
    6 as ::core::ffi::c_int as yytype_int8,
    34 as ::core::ffi::c_int as yytype_int8,
    41 as ::core::ffi::c_int as yytype_int8,
    6 as ::core::ffi::c_int as yytype_int8,
    18 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    20 as ::core::ffi::c_int as yytype_int8,
    13 as ::core::ffi::c_int as yytype_int8,
    13 as ::core::ffi::c_int as yytype_int8,
    35 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    30 as ::core::ffi::c_int as yytype_int8,
    29 as ::core::ffi::c_int as yytype_int8,
    6 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    45 as ::core::ffi::c_int as yytype_int8,
    42 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    21 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    19 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    39 as ::core::ffi::c_int as yytype_int8,
    13 as ::core::ffi::c_int as yytype_int8,
    37 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    46 as ::core::ffi::c_int as yytype_int8,
    6 as ::core::ffi::c_int as yytype_int8,
    6 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    23 as ::core::ffi::c_int as yytype_int8,
    36 as ::core::ffi::c_int as yytype_int8,
    40 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    47 as ::core::ffi::c_int as yytype_int8,
    6 as ::core::ffi::c_int as yytype_int8,
    25 as ::core::ffi::c_int as yytype_int8,
    6 as ::core::ffi::c_int as yytype_int8,
    38 as ::core::ffi::c_int as yytype_int8,
    22 as ::core::ffi::c_int as yytype_int8,
    26 as ::core::ffi::c_int as yytype_int8,
    6 as ::core::ffi::c_int as yytype_int8,
    24 as ::core::ffi::c_int as yytype_int8,
];
static mut yypgoto: [yytype_int8; 22] = [
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(23 as ::core::ffi::c_int) as yytype_int8,
    -(5 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    59 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(8 as ::core::ffi::c_int) as yytype_int8,
    -(31 as ::core::ffi::c_int) as yytype_int8,
    -(30 as ::core::ffi::c_int) as yytype_int8,
    -(9 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(18 as ::core::ffi::c_int) as yytype_int8,
    -(4 as ::core::ffi::c_int) as yytype_int8,
    74 as ::core::ffi::c_int as yytype_int8,
    75 as ::core::ffi::c_int as yytype_int8,
    50 as ::core::ffi::c_int as yytype_int8,
    70 as ::core::ffi::c_int as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
    -(32 as ::core::ffi::c_int) as yytype_int8,
];
static mut yydefgoto: [yytype_int8; 22] = [
    0 as ::core::ffi::c_int as yytype_int8,
    4 as ::core::ffi::c_int as yytype_int8,
    5 as ::core::ffi::c_int as yytype_int8,
    6 as ::core::ffi::c_int as yytype_int8,
    18 as ::core::ffi::c_int as yytype_int8,
    19 as ::core::ffi::c_int as yytype_int8,
    7 as ::core::ffi::c_int as yytype_int8,
    8 as ::core::ffi::c_int as yytype_int8,
    9 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    38 as ::core::ffi::c_int as yytype_int8,
    39 as ::core::ffi::c_int as yytype_int8,
    40 as ::core::ffi::c_int as yytype_int8,
    11 as ::core::ffi::c_int as yytype_int8,
    51 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    13 as ::core::ffi::c_int as yytype_int8,
    14 as ::core::ffi::c_int as yytype_int8,
    41 as ::core::ffi::c_int as yytype_int8,
    32 as ::core::ffi::c_int as yytype_int8,
    33 as ::core::ffi::c_int as yytype_int8,
    46 as ::core::ffi::c_int as yytype_int8,
];
unsafe extern "C" fn cmd_parse_expand_alias(
    mut cmd: *mut cmd_parse_command,
    mut pi: *mut cmd_parse_input,
    mut pr: *mut cmd_parse_result,
) -> ::core::ffi::c_int {
    let mut first: *mut cmd_parse_argument = ::core::ptr::null_mut::<cmd_parse_argument>();
    let mut cmds: *mut cmd_parse_commands = ::core::ptr::null_mut::<cmd_parse_commands>();
    let mut last: *mut cmd_parse_command = ::core::ptr::null_mut::<cmd_parse_command>();
    let mut alias: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*pi).flags & CMD_PARSE_NOALIAS != 0 {
        return 0 as ::core::ffi::c_int;
    }
    memset(
        pr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_result>() as size_t,
    );
    first = (*cmd).arguments.tqh_first;
    if first.is_null()
        || (*first).type_0 as ::core::ffi::c_uint
            != CMD_PARSE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*pr).status = CMD_PARSE_SUCCESS;
        (*pr).cmdlist = cmd_list_new();
        return 1 as ::core::ffi::c_int;
    }
    name = (*first).string;
    alias = cmd_get_alias(name);
    if alias.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: %u alias %s = %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_parse_expand_alias\0" as *const u8 as *const ::core::ffi::c_char,
        (*pi).line,
        name,
        alias,
    );
    cmds = cmd_parse_do_buffer(alias, strlen(alias), pi, &raw mut cause);
    free(alias as *mut ::core::ffi::c_void);
    if cmds.is_null() {
        (*pr).status = CMD_PARSE_ERROR;
        (*pr).error = cause;
        return 1 as ::core::ffi::c_int;
    }
    last = *(*((*cmds).tqh_last as *mut cmd_parse_commands)).tqh_last;
    if last.is_null() {
        (*pr).status = CMD_PARSE_SUCCESS;
        (*pr).cmdlist = cmd_list_new();
        cmd_parse_free_commands(cmds);
        return 1 as ::core::ffi::c_int;
    }
    if !(*first).entry.tqe_next.is_null() {
        (*(*first).entry.tqe_next).entry.tqe_prev = (*first).entry.tqe_prev;
    } else {
        (*cmd).arguments.tqh_last = (*first).entry.tqe_prev;
    }
    *(*first).entry.tqe_prev = (*first).entry.tqe_next;
    cmd_parse_free_argument(first);
    if !(*cmd).arguments.tqh_first.is_null() {
        *(*last).arguments.tqh_last = (*cmd).arguments.tqh_first;
        (*(*cmd).arguments.tqh_first).entry.tqe_prev = (*last).arguments.tqh_last;
        (*last).arguments.tqh_last = (*cmd).arguments.tqh_last;
        (*cmd).arguments.tqh_first = ::core::ptr::null_mut::<cmd_parse_argument>();
        (*cmd).arguments.tqh_last = &raw mut (*cmd).arguments.tqh_first;
    }
    cmd_parse_log_commands(
        cmds,
        b"cmd_parse_expand_alias\0" as *const u8 as *const ::core::ffi::c_char,
    );
    (*pi).flags |= CMD_PARSE_NOALIAS;
    cmd_parse_build_commands(cmds, pi, pr);
    (*pi).flags &= !CMD_PARSE_NOALIAS;
    cmd_parse_free_commands(cmds);
    return 1 as ::core::ffi::c_int;
}
static mut yytable: [yytype_int8; 105] = [
    21 as ::core::ffi::c_int as yytype_int8,
    34 as ::core::ffi::c_int as yytype_int8,
    25 as ::core::ffi::c_int as yytype_int8,
    48 as ::core::ffi::c_int as yytype_int8,
    49 as ::core::ffi::c_int as yytype_int8,
    35 as ::core::ffi::c_int as yytype_int8,
    26 as ::core::ffi::c_int as yytype_int8,
    37 as ::core::ffi::c_int as yytype_int8,
    44 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    55 as ::core::ffi::c_int as yytype_int8,
    35 as ::core::ffi::c_int as yytype_int8,
    36 as ::core::ffi::c_int as yytype_int8,
    37 as ::core::ffi::c_int as yytype_int8,
    15 as ::core::ffi::c_int as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    24 as ::core::ffi::c_int as yytype_int8,
    25 as ::core::ffi::c_int as yytype_int8,
    27 as ::core::ffi::c_int as yytype_int8,
    25 as ::core::ffi::c_int as yytype_int8,
    61 as ::core::ffi::c_int as yytype_int8,
    26 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    16 as ::core::ffi::c_int as yytype_int8,
    17 as ::core::ffi::c_int as yytype_int8,
    50 as ::core::ffi::c_int as yytype_int8,
    45 as ::core::ffi::c_int as yytype_int8,
    -(13 as ::core::ffi::c_int) as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    21 as ::core::ffi::c_int as yytype_int8,
    25 as ::core::ffi::c_int as yytype_int8,
    25 as ::core::ffi::c_int as yytype_int8,
    56 as ::core::ffi::c_int as yytype_int8,
    20 as ::core::ffi::c_int as yytype_int8,
    53 as ::core::ffi::c_int as yytype_int8,
    54 as ::core::ffi::c_int as yytype_int8,
    67 as ::core::ffi::c_int as yytype_int8,
    68 as ::core::ffi::c_int as yytype_int8,
    49 as ::core::ffi::c_int as yytype_int8,
    57 as ::core::ffi::c_int as yytype_int8,
    37 as ::core::ffi::c_int as yytype_int8,
    22 as ::core::ffi::c_int as yytype_int8,
    62 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    63 as ::core::ffi::c_int as yytype_int8,
    27 as ::core::ffi::c_int as yytype_int8,
    73 as ::core::ffi::c_int as yytype_int8,
    25 as ::core::ffi::c_int as yytype_int8,
    27 as ::core::ffi::c_int as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    72 as ::core::ffi::c_int as yytype_int8,
    65 as ::core::ffi::c_int as yytype_int8,
    22 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    58 as ::core::ffi::c_int as yytype_int8,
    70 as ::core::ffi::c_int as yytype_int8,
    23 as ::core::ffi::c_int as yytype_int8,
    71 as ::core::ffi::c_int as yytype_int8,
    -(13 as ::core::ffi::c_int) as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    -(6 as ::core::ffi::c_int) as yytype_int8,
    21 as ::core::ffi::c_int as yytype_int8,
    21 as ::core::ffi::c_int as yytype_int8,
    74 as ::core::ffi::c_int as yytype_int8,
    36 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    21 as ::core::ffi::c_int as yytype_int8,
    36 as ::core::ffi::c_int as yytype_int8,
    -(14 as ::core::ffi::c_int) as yytype_int8,
    27 as ::core::ffi::c_int as yytype_int8,
    -(13 as ::core::ffi::c_int) as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    -(6 as ::core::ffi::c_int) as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    35 as ::core::ffi::c_int as yytype_int8,
    36 as ::core::ffi::c_int as yytype_int8,
    37 as ::core::ffi::c_int as yytype_int8,
    28 as ::core::ffi::c_int as yytype_int8,
    -(13 as ::core::ffi::c_int) as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    37 as ::core::ffi::c_int as yytype_int8,
    59 as ::core::ffi::c_int as yytype_int8,
    -(13 as ::core::ffi::c_int) as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    -(13 as ::core::ffi::c_int) as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    29 as ::core::ffi::c_int as yytype_int8,
    30 as ::core::ffi::c_int as yytype_int8,
    52 as ::core::ffi::c_int as yytype_int8,
    28 as ::core::ffi::c_int as yytype_int8,
    31 as ::core::ffi::c_int as yytype_int8,
    60 as ::core::ffi::c_int as yytype_int8,
    66 as ::core::ffi::c_int as yytype_int8,
    69 as ::core::ffi::c_int as yytype_int8,
    42 as ::core::ffi::c_int as yytype_int8,
    43 as ::core::ffi::c_int as yytype_int8,
    47 as ::core::ffi::c_int as yytype_int8,
    64 as ::core::ffi::c_int as yytype_int8,
];
static mut yycheck: [yytype_int8; 105] = [
    5 as ::core::ffi::c_int as yytype_int8,
    24 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    34 as ::core::ffi::c_int as yytype_int8,
    34 as ::core::ffi::c_int as yytype_int8,
    6 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    8 as ::core::ffi::c_int as yytype_int8,
    31 as ::core::ffi::c_int as yytype_int8,
    5 as ::core::ffi::c_int as yytype_int8,
    41 as ::core::ffi::c_int as yytype_int8,
    6 as ::core::ffi::c_int as yytype_int8,
    7 as ::core::ffi::c_int as yytype_int8,
    8 as ::core::ffi::c_int as yytype_int8,
    11 as ::core::ffi::c_int as yytype_int8,
    11 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    25 as ::core::ffi::c_int as yytype_int8,
    13 as ::core::ffi::c_int as yytype_int8,
    27 as ::core::ffi::c_int as yytype_int8,
    51 as ::core::ffi::c_int as yytype_int8,
    25 as ::core::ffi::c_int as yytype_int8,
    5 as ::core::ffi::c_int as yytype_int8,
    9 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    34 as ::core::ffi::c_int as yytype_int8,
    31 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    11 as ::core::ffi::c_int as yytype_int8,
    34 as ::core::ffi::c_int as yytype_int8,
    38 as ::core::ffi::c_int as yytype_int8,
    39 as ::core::ffi::c_int as yytype_int8,
    41 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    38 as ::core::ffi::c_int as yytype_int8,
    39 as ::core::ffi::c_int as yytype_int8,
    59 as ::core::ffi::c_int as yytype_int8,
    60 as ::core::ffi::c_int as yytype_int8,
    68 as ::core::ffi::c_int as yytype_int8,
    44 as ::core::ffi::c_int as yytype_int8,
    8 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    51 as ::core::ffi::c_int as yytype_int8,
    5 as ::core::ffi::c_int as yytype_int8,
    53 as ::core::ffi::c_int as yytype_int8,
    13 as ::core::ffi::c_int as yytype_int8,
    69 as ::core::ffi::c_int as yytype_int8,
    55 as ::core::ffi::c_int as yytype_int8,
    13 as ::core::ffi::c_int as yytype_int8,
    11 as ::core::ffi::c_int as yytype_int8,
    68 as ::core::ffi::c_int as yytype_int8,
    55 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    4 as ::core::ffi::c_int as yytype_int8,
    5 as ::core::ffi::c_int as yytype_int8,
    15 as ::core::ffi::c_int as yytype_int8,
    65 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    67 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    11 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    67 as ::core::ffi::c_int as yytype_int8,
    68 as ::core::ffi::c_int as yytype_int8,
    73 as ::core::ffi::c_int as yytype_int8,
    7 as ::core::ffi::c_int as yytype_int8,
    4 as ::core::ffi::c_int as yytype_int8,
    5 as ::core::ffi::c_int as yytype_int8,
    73 as ::core::ffi::c_int as yytype_int8,
    7 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    13 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    11 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    4 as ::core::ffi::c_int as yytype_int8,
    5 as ::core::ffi::c_int as yytype_int8,
    6 as ::core::ffi::c_int as yytype_int8,
    7 as ::core::ffi::c_int as yytype_int8,
    8 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    11 as ::core::ffi::c_int as yytype_int8,
    4 as ::core::ffi::c_int as yytype_int8,
    5 as ::core::ffi::c_int as yytype_int8,
    4 as ::core::ffi::c_int as yytype_int8,
    5 as ::core::ffi::c_int as yytype_int8,
    8 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    11 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    11 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    11 as ::core::ffi::c_int as yytype_int8,
    36 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    14 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    15 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    27 as ::core::ffi::c_int as yytype_int8,
    27 as ::core::ffi::c_int as yytype_int8,
    33 as ::core::ffi::c_int as yytype_int8,
    54 as ::core::ffi::c_int as yytype_int8,
];
static mut yystos: [yytype_int8; 75] = [
    0 as ::core::ffi::c_int as yytype_int8,
    4 as ::core::ffi::c_int as yytype_int8,
    5 as ::core::ffi::c_int as yytype_int8,
    11 as ::core::ffi::c_int as yytype_int8,
    17 as ::core::ffi::c_int as yytype_int8,
    18 as ::core::ffi::c_int as yytype_int8,
    19 as ::core::ffi::c_int as yytype_int8,
    22 as ::core::ffi::c_int as yytype_int8,
    23 as ::core::ffi::c_int as yytype_int8,
    24 as ::core::ffi::c_int as yytype_int8,
    25 as ::core::ffi::c_int as yytype_int8,
    29 as ::core::ffi::c_int as yytype_int8,
    31 as ::core::ffi::c_int as yytype_int8,
    32 as ::core::ffi::c_int as yytype_int8,
    33 as ::core::ffi::c_int as yytype_int8,
    11 as ::core::ffi::c_int as yytype_int8,
    9 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    20 as ::core::ffi::c_int as yytype_int8,
    21 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    19 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    25 as ::core::ffi::c_int as yytype_int8,
    31 as ::core::ffi::c_int as yytype_int8,
    13 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    10 as ::core::ffi::c_int as yytype_int8,
    11 as ::core::ffi::c_int as yytype_int8,
    14 as ::core::ffi::c_int as yytype_int8,
    35 as ::core::ffi::c_int as yytype_int8,
    36 as ::core::ffi::c_int as yytype_int8,
    18 as ::core::ffi::c_int as yytype_int8,
    6 as ::core::ffi::c_int as yytype_int8,
    7 as ::core::ffi::c_int as yytype_int8,
    8 as ::core::ffi::c_int as yytype_int8,
    26 as ::core::ffi::c_int as yytype_int8,
    27 as ::core::ffi::c_int as yytype_int8,
    28 as ::core::ffi::c_int as yytype_int8,
    34 as ::core::ffi::c_int as yytype_int8,
    32 as ::core::ffi::c_int as yytype_int8,
    33 as ::core::ffi::c_int as yytype_int8,
    18 as ::core::ffi::c_int as yytype_int8,
    19 as ::core::ffi::c_int as yytype_int8,
    37 as ::core::ffi::c_int as yytype_int8,
    35 as ::core::ffi::c_int as yytype_int8,
    26 as ::core::ffi::c_int as yytype_int8,
    27 as ::core::ffi::c_int as yytype_int8,
    28 as ::core::ffi::c_int as yytype_int8,
    30 as ::core::ffi::c_int as yytype_int8,
    21 as ::core::ffi::c_int as yytype_int8,
    31 as ::core::ffi::c_int as yytype_int8,
    31 as ::core::ffi::c_int as yytype_int8,
    26 as ::core::ffi::c_int as yytype_int8,
    28 as ::core::ffi::c_int as yytype_int8,
    19 as ::core::ffi::c_int as yytype_int8,
    15 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    26 as ::core::ffi::c_int as yytype_int8,
    28 as ::core::ffi::c_int as yytype_int8,
    28 as ::core::ffi::c_int as yytype_int8,
    34 as ::core::ffi::c_int as yytype_int8,
    31 as ::core::ffi::c_int as yytype_int8,
    15 as ::core::ffi::c_int as yytype_int8,
    18 as ::core::ffi::c_int as yytype_int8,
    18 as ::core::ffi::c_int as yytype_int8,
    12 as ::core::ffi::c_int as yytype_int8,
    28 as ::core::ffi::c_int as yytype_int8,
    28 as ::core::ffi::c_int as yytype_int8,
    30 as ::core::ffi::c_int as yytype_int8,
    18 as ::core::ffi::c_int as yytype_int8,
    28 as ::core::ffi::c_int as yytype_int8,
];
static mut yyr1: [yytype_int8; 48] = [
    0 as ::core::ffi::c_int as yytype_int8,
    16 as ::core::ffi::c_int as yytype_int8,
    17 as ::core::ffi::c_int as yytype_int8,
    17 as ::core::ffi::c_int as yytype_int8,
    18 as ::core::ffi::c_int as yytype_int8,
    18 as ::core::ffi::c_int as yytype_int8,
    19 as ::core::ffi::c_int as yytype_int8,
    19 as ::core::ffi::c_int as yytype_int8,
    19 as ::core::ffi::c_int as yytype_int8,
    19 as ::core::ffi::c_int as yytype_int8,
    20 as ::core::ffi::c_int as yytype_int8,
    20 as ::core::ffi::c_int as yytype_int8,
    21 as ::core::ffi::c_int as yytype_int8,
    22 as ::core::ffi::c_int as yytype_int8,
    22 as ::core::ffi::c_int as yytype_int8,
    23 as ::core::ffi::c_int as yytype_int8,
    24 as ::core::ffi::c_int as yytype_int8,
    25 as ::core::ffi::c_int as yytype_int8,
    26 as ::core::ffi::c_int as yytype_int8,
    27 as ::core::ffi::c_int as yytype_int8,
    28 as ::core::ffi::c_int as yytype_int8,
    29 as ::core::ffi::c_int as yytype_int8,
    29 as ::core::ffi::c_int as yytype_int8,
    29 as ::core::ffi::c_int as yytype_int8,
    29 as ::core::ffi::c_int as yytype_int8,
    30 as ::core::ffi::c_int as yytype_int8,
    30 as ::core::ffi::c_int as yytype_int8,
    31 as ::core::ffi::c_int as yytype_int8,
    31 as ::core::ffi::c_int as yytype_int8,
    31 as ::core::ffi::c_int as yytype_int8,
    31 as ::core::ffi::c_int as yytype_int8,
    31 as ::core::ffi::c_int as yytype_int8,
    32 as ::core::ffi::c_int as yytype_int8,
    32 as ::core::ffi::c_int as yytype_int8,
    32 as ::core::ffi::c_int as yytype_int8,
    33 as ::core::ffi::c_int as yytype_int8,
    33 as ::core::ffi::c_int as yytype_int8,
    33 as ::core::ffi::c_int as yytype_int8,
    33 as ::core::ffi::c_int as yytype_int8,
    34 as ::core::ffi::c_int as yytype_int8,
    34 as ::core::ffi::c_int as yytype_int8,
    35 as ::core::ffi::c_int as yytype_int8,
    35 as ::core::ffi::c_int as yytype_int8,
    36 as ::core::ffi::c_int as yytype_int8,
    36 as ::core::ffi::c_int as yytype_int8,
    36 as ::core::ffi::c_int as yytype_int8,
    37 as ::core::ffi::c_int as yytype_int8,
    37 as ::core::ffi::c_int as yytype_int8,
];
unsafe extern "C" fn cmd_parse_build_command(
    mut cmd: *mut cmd_parse_command,
    mut pi: *mut cmd_parse_input,
    mut pr: *mut cmd_parse_result,
) {
    let mut current_block: u64;
    let mut arg: *mut cmd_parse_argument = ::core::ptr::null_mut::<cmd_parse_argument>();
    let mut add: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut values: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut count: u_int = 0 as u_int;
    let mut idx: u_int = 0;
    memset(
        pr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_result>() as size_t,
    );
    if cmd_parse_expand_alias(cmd, pi, pr) != 0 {
        return;
    }
    arg = (*cmd).arguments.tqh_first;
    loop {
        if arg.is_null() {
            current_block = 5143058163439228106;
            break;
        }
        values = xrecallocarray(
            values as *mut ::core::ffi::c_void,
            count as size_t,
            count.wrapping_add(1 as u_int) as size_t,
            ::core::mem::size_of::<args_value>() as size_t,
        ) as *mut args_value;
        match (*arg).type_0 as ::core::ffi::c_uint {
            0 => {
                (*values.offset(count as isize)).type_0 = ARGS_STRING;
                let ref mut fresh0 = (*values.offset(count as isize)).c2rust_unnamed.string;
                *fresh0 = xstrdup((*arg).string);
            }
            1 => {
                cmd_parse_build_commands((*arg).commands as *mut cmd_parse_commands, pi, pr);
                if (*pr).status as ::core::ffi::c_uint
                    != CMD_PARSE_SUCCESS as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    current_block = 16207960823932980356;
                    break;
                }
                (*values.offset(count as isize)).type_0 = ARGS_COMMANDS;
                let ref mut fresh1 = (*values.offset(count as isize)).c2rust_unnamed.cmdlist;
                *fresh1 = (*pr).cmdlist as *mut cmd_list;
            }
            2 => {
                (*values.offset(count as isize)).type_0 = ARGS_COMMANDS;
                let ref mut fresh2 = (*values.offset(count as isize)).c2rust_unnamed.cmdlist;
                *fresh2 = (*arg).cmdlist as *mut cmd_list;
                let ref mut fresh3 =
                    (*(*values.offset(count as isize)).c2rust_unnamed.cmdlist).references;
                *fresh3 += 1;
            }
            _ => {}
        }
        count = count.wrapping_add(1);
        arg = (*arg).entry.tqe_next;
    }
    match current_block {
        5143058163439228106 => {
            add = cmd_parse(
                values,
                count,
                (*pi).file,
                (*pi).line,
                (*pi).flags,
                &raw mut cause,
            );
            if add.is_null() {
                (*pr).status = CMD_PARSE_ERROR;
                (*pr).error = cmd_parse_get_error((*pi).file, (*pi).line, cause);
                free(cause as *mut ::core::ffi::c_void);
            } else {
                (*pr).status = CMD_PARSE_SUCCESS;
                (*pr).cmdlist = cmd_list_new();
                cmd_list_append((*pr).cmdlist, add);
            }
        }
        _ => {}
    }
    idx = 0 as u_int;
    while idx < count {
        args_free_value(values.offset(idx as isize) as *mut args_value);
        idx = idx.wrapping_add(1);
    }
    free(values as *mut ::core::ffi::c_void);
}
static mut yyr2: [yytype_int8; 48] = [
    0 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    0 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    4 as ::core::ffi::c_int as yytype_int8,
    7 as ::core::ffi::c_int as yytype_int8,
    5 as ::core::ffi::c_int as yytype_int8,
    8 as ::core::ffi::c_int as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    4 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    5 as ::core::ffi::c_int as yytype_int8,
    4 as ::core::ffi::c_int as yytype_int8,
    6 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    1 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    2 as ::core::ffi::c_int as yytype_int8,
    3 as ::core::ffi::c_int as yytype_int8,
];
unsafe extern "C" fn cmd_parse_build_commands(
    mut cmds: *mut cmd_parse_commands,
    mut pi: *mut cmd_parse_input,
    mut pr: *mut cmd_parse_result,
) {
    let mut cmd: *mut cmd_parse_command = ::core::ptr::null_mut::<cmd_parse_command>();
    let mut line: u_int = UINT_MAX;
    let mut current: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    let mut result: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    memset(
        pr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_result>() as size_t,
    );
    if (*cmds).tqh_first.is_null() {
        (*pr).status = CMD_PARSE_SUCCESS;
        (*pr).cmdlist = cmd_list_new();
        return;
    }
    cmd_parse_log_commands(
        cmds,
        b"cmd_parse_build_commands\0" as *const u8 as *const ::core::ffi::c_char,
    );
    result = cmd_list_new();
    cmd = (*cmds).tqh_first;
    while !cmd.is_null() {
        if !(*pi).flags & CMD_PARSE_ONEGROUP != 0 && (*cmd).line != line {
            if !current.is_null() {
                cmd_parse_print_commands(pi, current);
                cmd_list_move(result, current);
                cmd_list_free(current);
            }
            current = cmd_list_new();
        }
        if current.is_null() {
            current = cmd_list_new();
        }
        (*pi).line = (*cmd).line;
        line = (*pi).line;
        cmd_parse_build_command(cmd, pi, pr);
        if (*pr).status as ::core::ffi::c_uint
            != CMD_PARSE_SUCCESS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            cmd_list_free(result);
            cmd_list_free(current);
            return;
        }
        cmd_list_append_all(current, (*pr).cmdlist);
        cmd_list_free((*pr).cmdlist);
        cmd = (*cmd).entry.tqe_next;
    }
    if !current.is_null() {
        cmd_parse_print_commands(pi, current);
        cmd_list_move(result, current);
        cmd_list_free(current);
    }
    s = cmd_list_print(result, 0 as ::core::ffi::c_int);
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_parse_build_commands\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    free(s as *mut ::core::ffi::c_void);
    (*pr).status = CMD_PARSE_SUCCESS;
    (*pr).cmdlist = result;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_parse_from_file(
    mut f: *mut FILE,
    mut pi: *mut cmd_parse_input,
) -> *mut cmd_parse_result {
    static mut pr: cmd_parse_result = cmd_parse_result {
        status: CMD_PARSE_ERROR,
        cmdlist: ::core::ptr::null::<cmd_list>() as *mut cmd_list,
        error: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
    };
    let mut input: cmd_parse_input = cmd_parse_input {
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
    let mut cmds: *mut cmd_parse_commands = ::core::ptr::null_mut::<cmd_parse_commands>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if pi.is_null() {
        memset(
            &raw mut input as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<cmd_parse_input>() as size_t,
        );
        pi = &raw mut input;
    }
    memset(
        &raw mut pr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_result>() as size_t,
    );
    cmds = cmd_parse_do_file(f, pi, &raw mut cause);
    if cmds.is_null() {
        pr.status = CMD_PARSE_ERROR;
        pr.error = cause;
        return &raw mut pr;
    }
    cmd_parse_build_commands(cmds, pi, &raw mut pr);
    cmd_parse_free_commands(cmds);
    return &raw mut pr;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_parse_from_string(
    mut s: *const ::core::ffi::c_char,
    mut pi: *mut cmd_parse_input,
) -> *mut cmd_parse_result {
    let mut input: cmd_parse_input = cmd_parse_input {
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
    if pi.is_null() {
        memset(
            &raw mut input as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<cmd_parse_input>() as size_t,
        );
        pi = &raw mut input;
    }
    (*pi).flags |= CMD_PARSE_ONEGROUP;
    return cmd_parse_from_buffer(s as *const ::core::ffi::c_void, strlen(s), pi);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_parse_and_insert(
    mut s: *const ::core::ffi::c_char,
    mut pi: *mut cmd_parse_input,
    mut after: *mut cmdq_item,
    mut state: *mut cmdq_state,
    mut error: *mut *mut ::core::ffi::c_char,
) -> cmd_parse_status {
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    pr = cmd_parse_from_string(s, pi);
    match (*pr).status as ::core::ffi::c_uint {
        0 => {
            if !error.is_null() {
                *error = (*pr).error;
            } else {
                free((*pr).error as *mut ::core::ffi::c_void);
            }
        }
        1 => {
            item = cmdq_get_command((*pr).cmdlist, state);
            cmdq_insert_after(after, item);
            cmd_list_free((*pr).cmdlist);
        }
        _ => {}
    }
    return (*pr).status;
}
pub const YYINITDEPTH: ::core::ffi::c_int = 200 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn cmd_parse_and_append(
    mut s: *const ::core::ffi::c_char,
    mut pi: *mut cmd_parse_input,
    mut c: *mut client,
    mut state: *mut cmdq_state,
    mut error: *mut *mut ::core::ffi::c_char,
) -> cmd_parse_status {
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    pr = cmd_parse_from_string(s, pi);
    match (*pr).status as ::core::ffi::c_uint {
        0 => {
            if !error.is_null() {
                *error = (*pr).error;
            } else {
                free((*pr).error as *mut ::core::ffi::c_void);
            }
        }
        1 => {
            item = cmdq_get_command((*pr).cmdlist, state);
            cmdq_append(c, item);
            cmd_list_free((*pr).cmdlist);
        }
        _ => {}
    }
    return (*pr).status;
}
pub const YYMAXDEPTH: ::core::ffi::c_int = 10000 as ::core::ffi::c_int;
unsafe extern "C" fn yydestruct(
    mut yymsg: *const ::core::ffi::c_char,
    mut yykind: yysymbol_kind_t,
    mut yyvaluep: *mut YYSTYPE,
) {
    if yymsg.is_null() {
        yymsg = b"Deleting\0" as *const u8 as *const ::core::ffi::c_char;
    }
}
#[no_mangle]
pub unsafe extern "C" fn cmd_parse_from_buffer(
    mut buf: *const ::core::ffi::c_void,
    mut len: size_t,
    mut pi: *mut cmd_parse_input,
) -> *mut cmd_parse_result {
    static mut pr: cmd_parse_result = cmd_parse_result {
        status: CMD_PARSE_ERROR,
        cmdlist: ::core::ptr::null::<cmd_list>() as *mut cmd_list,
        error: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
    };
    let mut input: cmd_parse_input = cmd_parse_input {
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
    let mut cmds: *mut cmd_parse_commands = ::core::ptr::null_mut::<cmd_parse_commands>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if pi.is_null() {
        memset(
            &raw mut input as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<cmd_parse_input>() as size_t,
        );
        pi = &raw mut input;
    }
    memset(
        &raw mut pr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_result>() as size_t,
    );
    if len == 0 as size_t {
        pr.status = CMD_PARSE_SUCCESS;
        pr.cmdlist = cmd_list_new();
        return &raw mut pr;
    }
    cmds = cmd_parse_do_buffer(buf as *const ::core::ffi::c_char, len, pi, &raw mut cause);
    if cmds.is_null() {
        pr.status = CMD_PARSE_ERROR;
        pr.error = cause;
        return &raw mut pr;
    }
    cmd_parse_build_commands(cmds, pi, &raw mut pr);
    cmd_parse_free_commands(cmds);
    return &raw mut pr;
}
#[no_mangle]
pub static mut yychar: ::core::ffi::c_int = 0;
#[no_mangle]
pub static mut yylval: YYSTYPE = YYSTYPE {
    token: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
#[no_mangle]
pub static mut yynerrs: ::core::ffi::c_int = 0;
unsafe extern "C" fn yyparse() -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut yystate: yy_state_fast_t = 0 as yy_state_fast_t;
    let mut yyerrstatus: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut yystacksize: ::core::ffi::c_long = YYINITDEPTH as ::core::ffi::c_long;
    let mut yyssa: [yy_state_t; 200] = [0; 200];
    let mut yyss: *mut yy_state_t = &raw mut yyssa as *mut yy_state_t;
    let mut yyssp: *mut yy_state_t = yyss;
    let mut yyvsa: [YYSTYPE; 200] = [YYSTYPE {
        token: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    }; 200];
    let mut yyvs: *mut YYSTYPE = &raw mut yyvsa as *mut YYSTYPE;
    let mut yyvsp: *mut YYSTYPE = yyvs;
    let mut yyn: ::core::ffi::c_int = 0;
    let mut yyresult: ::core::ffi::c_int = 0;
    let mut yytoken: yysymbol_kind_t = YYSYMBOL_YYEMPTY;
    let mut yyval: YYSTYPE = YYSTYPE {
        token: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut yylen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    yychar = YYEMPTY;
    's_46: loop {
        (0 as ::core::ffi::c_int != 0
            && (0 as ::core::ffi::c_int <= yystate && yystate < 75 as ::core::ffi::c_int))
            as ::core::ffi::c_int;
        *yyssp = yystate as yy_state_t;
        if yyss
            .offset(yystacksize as isize)
            .offset(-(1 as ::core::ffi::c_int as isize))
            <= yyssp
        {
            let mut yysize: ::core::ffi::c_long =
                yyssp.offset_from(yyss) as ::core::ffi::c_long + 1 as ::core::ffi::c_long;
            if YYMAXDEPTH as ::core::ffi::c_long <= yystacksize {
                current_block = 18325888159776088768;
                break;
            }
            yystacksize *= 2 as ::core::ffi::c_long;
            if (YYMAXDEPTH as ::core::ffi::c_long) < yystacksize {
                yystacksize = YYMAXDEPTH as ::core::ffi::c_long;
            }
            let mut yyss1: *mut yy_state_t = yyss;
            let mut yyptr: *mut yyalloc = malloc(
                (yystacksize
                    * (::core::mem::size_of::<yy_state_t>() as ::core::ffi::c_long
                        + ::core::mem::size_of::<YYSTYPE>() as ::core::ffi::c_long)
                    + (::core::mem::size_of::<yyalloc>() as ::core::ffi::c_long
                        - 1 as ::core::ffi::c_long)) as size_t,
            ) as *mut yyalloc;
            if yyptr.is_null() {
                current_block = 18325888159776088768;
                break;
            }
            let mut yynewbytes: ::core::ffi::c_long = 0;
            ::libc::memcpy(
                &raw mut (*yyptr).yyss_alloc as *mut ::core::ffi::c_void,
                yyss as *const ::core::ffi::c_void,
                (yysize as usize).wrapping_mul(::core::mem::size_of::<yy_state_t>() as usize)
                    as ::libc::size_t,
            );
            yyss = &raw mut (*yyptr).yyss_alloc;
            yynewbytes = yystacksize * ::core::mem::size_of::<yy_state_t>() as ::core::ffi::c_long
                + YYSTACK_GAP_MAXIMUM;
            yyptr = yyptr.offset(
                (yynewbytes / ::core::mem::size_of::<yyalloc>() as ::core::ffi::c_long) as isize,
            );
            let mut yynewbytes_0: ::core::ffi::c_long = 0;
            ::libc::memcpy(
                &raw mut (*yyptr).yyvs_alloc as *mut ::core::ffi::c_void,
                yyvs as *const ::core::ffi::c_void,
                (yysize as usize).wrapping_mul(::core::mem::size_of::<YYSTYPE>() as usize)
                    as ::libc::size_t,
            );
            yyvs = &raw mut (*yyptr).yyvs_alloc;
            yynewbytes_0 = yystacksize * ::core::mem::size_of::<YYSTYPE>() as ::core::ffi::c_long
                + YYSTACK_GAP_MAXIMUM;
            yyptr = yyptr.offset(
                (yynewbytes_0 / ::core::mem::size_of::<yyalloc>() as ::core::ffi::c_long) as isize,
            );
            if yyss1 != &raw mut yyssa as *mut yy_state_t {
                free(yyss1 as *mut ::core::ffi::c_void);
            }
            yyssp = yyss
                .offset(yysize as isize)
                .offset(-(1 as ::core::ffi::c_int as isize));
            yyvsp = yyvs
                .offset(yysize as isize)
                .offset(-(1 as ::core::ffi::c_int as isize));
            if yyss
                .offset(yystacksize as isize)
                .offset(-(1 as ::core::ffi::c_int as isize))
                <= yyssp
            {
                current_block = 7016822936316948504;
                break;
            }
        }
        if yystate == YYFINAL {
            yyresult = 0 as ::core::ffi::c_int;
            current_block = 12167128357836294174;
            break;
        } else {
            yyn = yypact[yystate as usize] as ::core::ffi::c_int;
            if yyn == YYPACT_NINF {
                current_block = 9471547057618582048;
            } else {
                if yychar == YYEMPTY {
                    yychar = yylex();
                }
                if yychar <= YYEOF {
                    yychar = YYEOF;
                    yytoken = YYSYMBOL_YYEOF;
                    current_block = 6174974146017752131;
                } else if yychar == YYerror {
                    yychar = YYUNDEF;
                    yytoken = YYSYMBOL_YYerror;
                    current_block = 3337289579594398066;
                } else {
                    yytoken = (if 0 as ::core::ffi::c_int <= yychar && yychar <= YYMAXUTOK {
                        yytranslate[yychar as usize] as yysymbol_kind_t as ::core::ffi::c_int
                    } else {
                        YYSYMBOL_YYUNDEF as ::core::ffi::c_int
                    }) as yysymbol_kind_t;
                    current_block = 6174974146017752131;
                }
                match current_block {
                    3337289579594398066 => {}
                    _ => {
                        yyn += yytoken as ::core::ffi::c_int;
                        if yyn < 0 as ::core::ffi::c_int
                            || YYLAST < yyn
                            || yycheck[yyn as usize] as ::core::ffi::c_int
                                != yytoken as ::core::ffi::c_int
                        {
                            current_block = 9471547057618582048;
                        } else {
                            yyn = yytable[yyn as usize] as ::core::ffi::c_int;
                            if yyn <= 0 as ::core::ffi::c_int {
                                yyn = -yyn;
                                current_block = 9044536117179848658;
                            } else {
                                if yyerrstatus != 0 {
                                    yyerrstatus -= 1;
                                }
                                yystate = yyn as yy_state_fast_t;
                                yyvsp = yyvsp.offset(1);
                                *yyvsp = yylval;
                                yychar = YYEMPTY;
                                current_block = 12804197543227953123;
                            }
                        }
                    }
                }
            }
            match current_block {
                9471547057618582048 => {
                    yyn = yydefact[yystate as usize] as ::core::ffi::c_int;
                    if yyn == 0 as ::core::ffi::c_int {
                        yytoken = (if yychar == YYEMPTY {
                            YYSYMBOL_YYEMPTY as ::core::ffi::c_int
                        } else if 0 as ::core::ffi::c_int <= yychar && yychar <= YYMAXUTOK {
                            yytranslate[yychar as usize] as yysymbol_kind_t as ::core::ffi::c_int
                        } else {
                            YYSYMBOL_YYUNDEF as ::core::ffi::c_int
                        }) as yysymbol_kind_t;
                        if yyerrstatus == 0 {
                            yynerrs += 1;
                            yyerror(b"syntax error\0" as *const u8 as *const ::core::ffi::c_char);
                        }
                        if yyerrstatus == 3 as ::core::ffi::c_int {
                            if yychar <= YYEOF {
                                if yychar == YYEOF {
                                    current_block = 7016822936316948504;
                                    break;
                                }
                            } else {
                                yydestruct(
                                    b"Error: discarding\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    yytoken,
                                    &raw mut yylval,
                                );
                                yychar = YYEMPTY;
                            }
                        }
                        current_block = 3337289579594398066;
                    } else {
                        current_block = 9044536117179848658;
                    }
                }
                _ => {}
            }
            match current_block {
                3337289579594398066 => {
                    yyerrstatus = 3 as ::core::ffi::c_int;
                    loop {
                        yyn = yypact[yystate as usize] as ::core::ffi::c_int;
                        if !(yyn == YYPACT_NINF) {
                            yyn += YYSYMBOL_YYerror as ::core::ffi::c_int;
                            if 0 as ::core::ffi::c_int <= yyn
                                && yyn <= YYLAST
                                && yycheck[yyn as usize] as ::core::ffi::c_int
                                    == YYSYMBOL_YYerror as ::core::ffi::c_int
                            {
                                yyn = yytable[yyn as usize] as ::core::ffi::c_int;
                                if (0 as ::core::ffi::c_int) < yyn {
                                    break;
                                }
                            }
                        }
                        if yyssp == yyss {
                            current_block = 7016822936316948504;
                            break 's_46;
                        }
                        yydestruct(
                            b"Error: popping\0" as *const u8 as *const ::core::ffi::c_char,
                            yystos[yystate as usize] as yysymbol_kind_t,
                            yyvsp,
                        );
                        yyvsp = yyvsp.offset(-(1 as ::core::ffi::c_int as isize));
                        yyssp = yyssp.offset(-(1 as ::core::ffi::c_int as isize));
                        yystate = *yyssp as yy_state_fast_t;
                    }
                    yyvsp = yyvsp.offset(1);
                    *yyvsp = yylval;
                    yystate = yyn as yy_state_fast_t;
                }
                9044536117179848658 => {
                    yylen = yyr2[yyn as usize] as ::core::ffi::c_int;
                    yyval = *yyvsp.offset((1 as ::core::ffi::c_int - yylen) as isize);
                    match yyn {
                        3 => {
                            let mut ps: *mut cmd_parse_state = &raw mut parse_state;
                            (*ps).commands =
                                (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands;
                        }
                        4 => {
                            yyval.commands =
                                (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands;
                        }
                        5 => {
                            yyval.commands =
                                (*yyvsp.offset(-(2 as ::core::ffi::c_int) as isize)).commands;
                            if !(*(*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands)
                                .tqh_first
                                .is_null()
                            {
                                *(*yyval.commands).tqh_last = (*(*yyvsp
                                    .offset(-(1 as ::core::ffi::c_int) as isize))
                                .commands)
                                    .tqh_first;
                                let ref mut fresh4 = (*(*(*yyvsp
                                    .offset(-(1 as ::core::ffi::c_int) as isize))
                                .commands)
                                    .tqh_first)
                                    .entry
                                    .tqe_prev;
                                *fresh4 = (*yyval.commands).tqh_last;
                                (*yyval.commands).tqh_last = (*(*yyvsp
                                    .offset(-(1 as ::core::ffi::c_int) as isize))
                                .commands)
                                    .tqh_last;
                                let ref mut fresh5 = (*(*yyvsp
                                    .offset(-(1 as ::core::ffi::c_int) as isize))
                                .commands)
                                    .tqh_first;
                                *fresh5 = ::core::ptr::null_mut::<cmd_parse_command>();
                                let ref mut fresh6 = (*(*yyvsp
                                    .offset(-(1 as ::core::ffi::c_int) as isize))
                                .commands)
                                    .tqh_last;
                                *fresh6 = &raw mut (*(*yyvsp
                                    .offset(-(1 as ::core::ffi::c_int) as isize))
                                .commands)
                                    .tqh_first;
                            }
                            free(
                                (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands
                                    as *mut ::core::ffi::c_void,
                            );
                        }
                        6 => {
                            yyval.commands =
                                xmalloc(::core::mem::size_of::<cmd_parse_commands>() as size_t)
                                    as *mut cmd_parse_commands;
                            (*yyval.commands).tqh_first =
                                ::core::ptr::null_mut::<cmd_parse_command>();
                            (*yyval.commands).tqh_last = &raw mut (*yyval.commands).tqh_first;
                        }
                        7 => {
                            yyval.commands =
                                xmalloc(::core::mem::size_of::<cmd_parse_commands>() as size_t)
                                    as *mut cmd_parse_commands;
                            (*yyval.commands).tqh_first =
                                ::core::ptr::null_mut::<cmd_parse_command>();
                            (*yyval.commands).tqh_last = &raw mut (*yyval.commands).tqh_first;
                        }
                        8 => {
                            let mut ps_0: *mut cmd_parse_state = &raw mut parse_state;
                            if (*ps_0).scope.is_null() || (*(*ps_0).scope).flag != 0 {
                                yyval.commands =
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands;
                            } else {
                                yyval.commands = cmd_parse_new_commands();
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands,
                                );
                            }
                        }
                        9 => {
                            let mut ps_1: *mut cmd_parse_state = &raw mut parse_state;
                            if (*ps_1).scope.is_null() || (*(*ps_1).scope).flag != 0 {
                                yyval.commands =
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands;
                            } else {
                                yyval.commands = cmd_parse_new_commands();
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands,
                                );
                            }
                        }
                        10 => {
                            yyval.token = (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token;
                        }
                        11 => {
                            yyval.token = (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token;
                        }
                        12 => {
                            let mut ps_2: *mut cmd_parse_state = &raw mut parse_state;
                            let mut pi: *mut cmd_parse_input = (*ps_2).input;
                            let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
                            let mut c: *mut client = (*pi).c;
                            let mut fsp: *mut cmd_find_state =
                                ::core::ptr::null_mut::<cmd_find_state>();
                            let mut fs: cmd_find_state = cmd_find_state {
                                flags: 0,
                                current: ::core::ptr::null_mut::<cmd_find_state>(),
                                s: ::core::ptr::null_mut::<session>(),
                                wl: ::core::ptr::null_mut::<winlink>(),
                                w: ::core::ptr::null_mut::<window>(),
                                wp: ::core::ptr::null_mut::<window_pane>(),
                                idx: 0,
                            };
                            let mut flags: ::core::ffi::c_int = FORMAT_NOJOBS;
                            if cmd_find_valid_state(&raw mut (*pi).fs) != 0 {
                                fsp = &raw mut (*pi).fs;
                            } else {
                                cmd_find_from_client(&raw mut fs, c, 0 as ::core::ffi::c_int);
                                fsp = &raw mut fs;
                            }
                            ft = format_create(c, (*pi).item, FORMAT_NONE, flags);
                            format_defaults(ft, c, (*fsp).s, (*fsp).wl, (*fsp).wp);
                            yyval.token = format_expand(
                                ft,
                                (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token,
                            );
                            format_free(ft);
                            free(
                                (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token
                                    as *mut ::core::ffi::c_void,
                            );
                        }
                        15 => {
                            let mut ps_3: *mut cmd_parse_state = &raw mut parse_state;
                            let mut flags_0: ::core::ffi::c_int = (*(*ps_3).input).flags;
                            let mut flag: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                            let mut scope: *mut cmd_parse_scope =
                                ::core::ptr::null_mut::<cmd_parse_scope>();
                            if !(*ps_3).scope.is_null() {
                                flag = (*(*ps_3).scope).flag;
                                scope = (*ps_3).stack.tqh_first;
                                while !scope.is_null() {
                                    flag = (flag != 0 && (*scope).flag != 0) as ::core::ffi::c_int;
                                    scope = (*scope).entry.tqe_next;
                                }
                            }
                            if strlen((*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token)
                                > CMD_PARSE_MAX_ENVIRON_LEN as size_t
                            {
                                yyerror(
                                    b"environment variable is too long\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                current_block = 7016822936316948504;
                                break;
                            } else {
                                if !flags_0 & CMD_PARSE_PARSEONLY != 0 && flag != 0 {
                                    environ_put(
                                        global_environ,
                                        (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token,
                                        0 as ::core::ffi::c_int,
                                    );
                                }
                                free(
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token
                                        as *mut ::core::ffi::c_void,
                                );
                            }
                        }
                        16 => {
                            let mut ps_4: *mut cmd_parse_state = &raw mut parse_state;
                            let mut flags_1: ::core::ffi::c_int = (*(*ps_4).input).flags;
                            let mut flag_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                            let mut scope_0: *mut cmd_parse_scope =
                                ::core::ptr::null_mut::<cmd_parse_scope>();
                            if !(*ps_4).scope.is_null() {
                                flag_0 = (*(*ps_4).scope).flag;
                                scope_0 = (*ps_4).stack.tqh_first;
                                while !scope_0.is_null() {
                                    flag_0 =
                                        (flag_0 != 0 && (*scope_0).flag != 0) as ::core::ffi::c_int;
                                    scope_0 = (*scope_0).entry.tqe_next;
                                }
                            }
                            if strlen((*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token)
                                > CMD_PARSE_MAX_ENVIRON_LEN as size_t
                            {
                                yyerror(
                                    b"environment variable is too long\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                                current_block = 7016822936316948504;
                                break;
                            } else {
                                if !flags_1 & CMD_PARSE_PARSEONLY != 0 && flag_0 != 0 {
                                    environ_put(
                                        global_environ,
                                        (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token,
                                        ENVIRON_HIDDEN,
                                    );
                                }
                                free(
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token
                                        as *mut ::core::ffi::c_void,
                                );
                            }
                        }
                        17 => {
                            let mut ps_5: *mut cmd_parse_state = &raw mut parse_state;
                            let mut scope_1: *mut cmd_parse_scope =
                                ::core::ptr::null_mut::<cmd_parse_scope>();
                            scope_1 = xmalloc(::core::mem::size_of::<cmd_parse_scope>() as size_t)
                                as *mut cmd_parse_scope;
                            (*scope_1).flag = format_true(
                                (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token,
                            );
                            yyval.flag = (*scope_1).flag;
                            free(
                                (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token
                                    as *mut ::core::ffi::c_void,
                            );
                            if !(*ps_5).scope.is_null() {
                                (*(*ps_5).scope).entry.tqe_next = (*ps_5).stack.tqh_first;
                                if !(*(*ps_5).scope).entry.tqe_next.is_null() {
                                    (*(*ps_5).stack.tqh_first).entry.tqe_prev =
                                        &raw mut (*(*ps_5).scope).entry.tqe_next;
                                } else {
                                    (*ps_5).stack.tqh_last =
                                        &raw mut (*(*ps_5).scope).entry.tqe_next;
                                }
                                (*ps_5).stack.tqh_first = (*ps_5).scope;
                                (*(*ps_5).scope).entry.tqe_prev = &raw mut (*ps_5).stack.tqh_first;
                            }
                            (*ps_5).scope = scope_1;
                        }
                        18 => {
                            let mut ps_6: *mut cmd_parse_state = &raw mut parse_state;
                            let mut scope_2: *mut cmd_parse_scope =
                                ::core::ptr::null_mut::<cmd_parse_scope>();
                            scope_2 = xmalloc(::core::mem::size_of::<cmd_parse_scope>() as size_t)
                                as *mut cmd_parse_scope;
                            (*scope_2).flag = ((*(*ps_6).scope).flag == 0) as ::core::ffi::c_int;
                            free((*ps_6).scope as *mut ::core::ffi::c_void);
                            (*ps_6).scope = scope_2;
                        }
                        19 => {
                            let mut ps_7: *mut cmd_parse_state = &raw mut parse_state;
                            let mut scope_3: *mut cmd_parse_scope =
                                ::core::ptr::null_mut::<cmd_parse_scope>();
                            scope_3 = xmalloc(::core::mem::size_of::<cmd_parse_scope>() as size_t)
                                as *mut cmd_parse_scope;
                            (*scope_3).flag = format_true(
                                (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token,
                            );
                            yyval.flag = (*scope_3).flag;
                            free(
                                (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token
                                    as *mut ::core::ffi::c_void,
                            );
                            free((*ps_7).scope as *mut ::core::ffi::c_void);
                            (*ps_7).scope = scope_3;
                        }
                        20 => {
                            let mut ps_8: *mut cmd_parse_state = &raw mut parse_state;
                            free((*ps_8).scope as *mut ::core::ffi::c_void);
                            (*ps_8).scope = (*ps_8).stack.tqh_first;
                            if !(*ps_8).scope.is_null() {
                                if !(*(*ps_8).scope).entry.tqe_next.is_null() {
                                    (*(*(*ps_8).scope).entry.tqe_next).entry.tqe_prev =
                                        (*(*ps_8).scope).entry.tqe_prev;
                                } else {
                                    (*ps_8).stack.tqh_last = (*(*ps_8).scope).entry.tqe_prev;
                                }
                                *(*(*ps_8).scope).entry.tqe_prev = (*(*ps_8).scope).entry.tqe_next;
                            }
                        }
                        21 => {
                            if (*yyvsp.offset(-(3 as ::core::ffi::c_int) as isize)).flag != 0 {
                                yyval.commands =
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands;
                            } else {
                                yyval.commands = cmd_parse_new_commands();
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands,
                                );
                            }
                        }
                        22 => {
                            if (*yyvsp.offset(-(6 as ::core::ffi::c_int) as isize)).flag != 0 {
                                yyval.commands =
                                    (*yyvsp.offset(-(4 as ::core::ffi::c_int) as isize)).commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands,
                                );
                            } else {
                                yyval.commands =
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(4 as ::core::ffi::c_int) as isize)).commands,
                                );
                            }
                        }
                        23 => {
                            if (*yyvsp.offset(-(4 as ::core::ffi::c_int) as isize)).flag != 0 {
                                yyval.commands =
                                    (*yyvsp.offset(-(2 as ::core::ffi::c_int) as isize)).commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize))
                                        .elif
                                        .commands,
                                );
                            } else if (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize))
                                .elif
                                .flag
                                != 0
                            {
                                yyval.commands = (*yyvsp
                                    .offset(-(1 as ::core::ffi::c_int) as isize))
                                .elif
                                .commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(2 as ::core::ffi::c_int) as isize)).commands,
                                );
                            } else {
                                yyval.commands = cmd_parse_new_commands();
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(2 as ::core::ffi::c_int) as isize)).commands,
                                );
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize))
                                        .elif
                                        .commands,
                                );
                            }
                        }
                        24 => {
                            if (*yyvsp.offset(-(7 as ::core::ffi::c_int) as isize)).flag != 0 {
                                yyval.commands =
                                    (*yyvsp.offset(-(5 as ::core::ffi::c_int) as isize)).commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(4 as ::core::ffi::c_int) as isize))
                                        .elif
                                        .commands,
                                );
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands,
                                );
                            } else if (*yyvsp.offset(-(4 as ::core::ffi::c_int) as isize))
                                .elif
                                .flag
                                != 0
                            {
                                yyval.commands = (*yyvsp
                                    .offset(-(4 as ::core::ffi::c_int) as isize))
                                .elif
                                .commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(5 as ::core::ffi::c_int) as isize)).commands,
                                );
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands,
                                );
                            } else {
                                yyval.commands =
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(5 as ::core::ffi::c_int) as isize)).commands,
                                );
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(4 as ::core::ffi::c_int) as isize))
                                        .elif
                                        .commands,
                                );
                            }
                        }
                        25 => {
                            if (*yyvsp.offset(-(2 as ::core::ffi::c_int) as isize)).flag != 0 {
                                yyval.elif.flag = 1 as ::core::ffi::c_int;
                                yyval.elif.commands =
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands;
                            } else {
                                yyval.elif.flag = 0 as ::core::ffi::c_int;
                                yyval.elif.commands = cmd_parse_new_commands();
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands,
                                );
                            }
                        }
                        26 => {
                            if (*yyvsp.offset(-(3 as ::core::ffi::c_int) as isize)).flag != 0 {
                                yyval.elif.flag = 1 as ::core::ffi::c_int;
                                yyval.elif.commands =
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize))
                                        .elif
                                        .commands,
                                );
                            } else if (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).elif.flag
                                != 0
                            {
                                yyval.elif.flag = 1 as ::core::ffi::c_int;
                                yyval.elif.commands = (*yyvsp
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .elif
                                .commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands,
                                );
                            } else {
                                yyval.elif.flag = 0 as ::core::ffi::c_int;
                                yyval.elif.commands = cmd_parse_new_commands();
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands,
                                );
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize))
                                        .elif
                                        .commands,
                                );
                            }
                        }
                        27 => {
                            let mut ps_9: *mut cmd_parse_state = &raw mut parse_state;
                            yyval.commands = cmd_parse_new_commands();
                            if !(*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).command)
                                .arguments
                                .tqh_first
                                .is_null()
                                && ((*ps_9).scope.is_null() || (*(*ps_9).scope).flag != 0)
                            {
                                let ref mut fresh7 =
                                    (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).command)
                                        .entry
                                        .tqe_next;
                                *fresh7 = ::core::ptr::null_mut::<cmd_parse_command>();
                                let ref mut fresh8 =
                                    (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).command)
                                        .entry
                                        .tqe_prev;
                                *fresh8 = (*yyval.commands).tqh_last;
                                *(*yyval.commands).tqh_last =
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).command;
                                (*yyval.commands).tqh_last = &raw mut (*(*yyvsp
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .command)
                                    .entry
                                    .tqe_next;
                            } else {
                                cmd_parse_free_command(
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).command,
                                );
                            }
                        }
                        28 => {
                            yyval.commands =
                                (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands;
                        }
                        29 => {
                            yyval.commands =
                                (*yyvsp.offset(-(2 as ::core::ffi::c_int) as isize)).commands;
                            if !(*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands)
                                .tqh_first
                                .is_null()
                            {
                                *(*yyval.commands).tqh_last =
                                    (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands)
                                        .tqh_first;
                                let ref mut fresh9 = (*(*(*yyvsp
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .commands)
                                    .tqh_first)
                                    .entry
                                    .tqe_prev;
                                *fresh9 = (*yyval.commands).tqh_last;
                                (*yyval.commands).tqh_last =
                                    (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands)
                                        .tqh_last;
                                let ref mut fresh10 =
                                    (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands)
                                        .tqh_first;
                                *fresh10 = ::core::ptr::null_mut::<cmd_parse_command>();
                                let ref mut fresh11 =
                                    (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands)
                                        .tqh_last;
                                *fresh11 = &raw mut (*(*yyvsp
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .commands)
                                    .tqh_first;
                            }
                            free(
                                (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands
                                    as *mut ::core::ffi::c_void,
                            );
                        }
                        30 => {
                            let mut ps_10: *mut cmd_parse_state = &raw mut parse_state;
                            if !(*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).command)
                                .arguments
                                .tqh_first
                                .is_null()
                                && ((*ps_10).scope.is_null() || (*(*ps_10).scope).flag != 0)
                            {
                                yyval.commands =
                                    (*yyvsp.offset(-(2 as ::core::ffi::c_int) as isize)).commands;
                                let ref mut fresh12 =
                                    (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).command)
                                        .entry
                                        .tqe_next;
                                *fresh12 = ::core::ptr::null_mut::<cmd_parse_command>();
                                let ref mut fresh13 =
                                    (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).command)
                                        .entry
                                        .tqe_prev;
                                *fresh13 = (*yyval.commands).tqh_last;
                                *(*yyval.commands).tqh_last =
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).command;
                                (*yyval.commands).tqh_last = &raw mut (*(*yyvsp
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .command)
                                    .entry
                                    .tqe_next;
                            } else {
                                yyval.commands = cmd_parse_new_commands();
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(2 as ::core::ffi::c_int) as isize)).commands,
                                );
                                cmd_parse_free_command(
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).command,
                                );
                            }
                        }
                        31 => {
                            yyval.commands =
                                (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands;
                        }
                        32 => {
                            let mut ps_11: *mut cmd_parse_state = &raw mut parse_state;
                            yyval.command = xcalloc(
                                1 as size_t,
                                ::core::mem::size_of::<cmd_parse_command>() as size_t,
                            ) as *mut cmd_parse_command;
                            (*yyval.command).line = (*(*ps_11).input).line;
                            (*yyval.command).arguments.tqh_first =
                                ::core::ptr::null_mut::<cmd_parse_argument>();
                            (*yyval.command).arguments.tqh_last =
                                &raw mut (*yyval.command).arguments.tqh_first;
                        }
                        33 => {
                            let mut ps_12: *mut cmd_parse_state = &raw mut parse_state;
                            let mut arg: *mut cmd_parse_argument =
                                ::core::ptr::null_mut::<cmd_parse_argument>();
                            yyval.command = xcalloc(
                                1 as size_t,
                                ::core::mem::size_of::<cmd_parse_command>() as size_t,
                            ) as *mut cmd_parse_command;
                            (*yyval.command).line = (*(*ps_12).input).line;
                            (*yyval.command).arguments.tqh_first =
                                ::core::ptr::null_mut::<cmd_parse_argument>();
                            (*yyval.command).arguments.tqh_last =
                                &raw mut (*yyval.command).arguments.tqh_first;
                            arg = xcalloc(
                                1 as size_t,
                                ::core::mem::size_of::<cmd_parse_argument>() as size_t,
                            ) as *mut cmd_parse_argument;
                            (*arg).type_0 = CMD_PARSE_STRING;
                            (*arg).string = (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token;
                            (*arg).entry.tqe_next = (*yyval.command).arguments.tqh_first;
                            if !(*arg).entry.tqe_next.is_null() {
                                (*(*yyval.command).arguments.tqh_first).entry.tqe_prev =
                                    &raw mut (*arg).entry.tqe_next;
                            } else {
                                (*yyval.command).arguments.tqh_last =
                                    &raw mut (*arg).entry.tqe_next;
                            }
                            (*yyval.command).arguments.tqh_first = arg;
                            (*arg).entry.tqe_prev = &raw mut (*yyval.command).arguments.tqh_first;
                        }
                        34 => {
                            let mut ps_13: *mut cmd_parse_state = &raw mut parse_state;
                            let mut arg_0: *mut cmd_parse_argument =
                                ::core::ptr::null_mut::<cmd_parse_argument>();
                            yyval.command = xcalloc(
                                1 as size_t,
                                ::core::mem::size_of::<cmd_parse_command>() as size_t,
                            ) as *mut cmd_parse_command;
                            (*yyval.command).line = (*(*ps_13).input).line;
                            (*yyval.command).arguments.tqh_first =
                                ::core::ptr::null_mut::<cmd_parse_argument>();
                            (*yyval.command).arguments.tqh_last =
                                &raw mut (*yyval.command).arguments.tqh_first;
                            if !(*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).arguments)
                                .tqh_first
                                .is_null()
                            {
                                *(*yyval.command).arguments.tqh_last =
                                    (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).arguments)
                                        .tqh_first;
                                let ref mut fresh14 = (*(*(*yyvsp
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .arguments)
                                    .tqh_first)
                                    .entry
                                    .tqe_prev;
                                *fresh14 = (*yyval.command).arguments.tqh_last;
                                (*yyval.command).arguments.tqh_last =
                                    (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).arguments)
                                        .tqh_last;
                                let ref mut fresh15 =
                                    (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).arguments)
                                        .tqh_first;
                                *fresh15 = ::core::ptr::null_mut::<cmd_parse_argument>();
                                let ref mut fresh16 =
                                    (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).arguments)
                                        .tqh_last;
                                *fresh16 = &raw mut (*(*yyvsp
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .arguments)
                                    .tqh_first;
                            }
                            free(
                                (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).arguments
                                    as *mut ::core::ffi::c_void,
                            );
                            arg_0 = xcalloc(
                                1 as size_t,
                                ::core::mem::size_of::<cmd_parse_argument>() as size_t,
                            ) as *mut cmd_parse_argument;
                            (*arg_0).type_0 = CMD_PARSE_STRING;
                            (*arg_0).string =
                                (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).token;
                            (*arg_0).entry.tqe_next = (*yyval.command).arguments.tqh_first;
                            if !(*arg_0).entry.tqe_next.is_null() {
                                (*(*yyval.command).arguments.tqh_first).entry.tqe_prev =
                                    &raw mut (*arg_0).entry.tqe_next;
                            } else {
                                (*yyval.command).arguments.tqh_last =
                                    &raw mut (*arg_0).entry.tqe_next;
                            }
                            (*yyval.command).arguments.tqh_first = arg_0;
                            (*arg_0).entry.tqe_prev = &raw mut (*yyval.command).arguments.tqh_first;
                        }
                        35 => {
                            if (*yyvsp.offset(-(2 as ::core::ffi::c_int) as isize)).flag != 0 {
                                yyval.commands =
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands;
                            } else {
                                yyval.commands = cmd_parse_new_commands();
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands,
                                );
                            }
                        }
                        36 => {
                            if (*yyvsp.offset(-(4 as ::core::ffi::c_int) as isize)).flag != 0 {
                                yyval.commands =
                                    (*yyvsp.offset(-(3 as ::core::ffi::c_int) as isize)).commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands,
                                );
                            } else {
                                yyval.commands =
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(3 as ::core::ffi::c_int) as isize)).commands,
                                );
                            }
                        }
                        37 => {
                            if (*yyvsp.offset(-(3 as ::core::ffi::c_int) as isize)).flag != 0 {
                                yyval.commands =
                                    (*yyvsp.offset(-(2 as ::core::ffi::c_int) as isize)).commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize))
                                        .elif
                                        .commands,
                                );
                            } else if (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize))
                                .elif
                                .flag
                                != 0
                            {
                                yyval.commands = (*yyvsp
                                    .offset(-(1 as ::core::ffi::c_int) as isize))
                                .elif
                                .commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(2 as ::core::ffi::c_int) as isize)).commands,
                                );
                            } else {
                                yyval.commands = cmd_parse_new_commands();
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(2 as ::core::ffi::c_int) as isize)).commands,
                                );
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize))
                                        .elif
                                        .commands,
                                );
                            }
                        }
                        38 => {
                            if (*yyvsp.offset(-(5 as ::core::ffi::c_int) as isize)).flag != 0 {
                                yyval.commands =
                                    (*yyvsp.offset(-(4 as ::core::ffi::c_int) as isize)).commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(3 as ::core::ffi::c_int) as isize))
                                        .elif
                                        .commands,
                                );
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands,
                                );
                            } else if (*yyvsp.offset(-(3 as ::core::ffi::c_int) as isize))
                                .elif
                                .flag
                                != 0
                            {
                                yyval.commands = (*yyvsp
                                    .offset(-(3 as ::core::ffi::c_int) as isize))
                                .elif
                                .commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(4 as ::core::ffi::c_int) as isize)).commands,
                                );
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands,
                                );
                            } else {
                                yyval.commands =
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(4 as ::core::ffi::c_int) as isize)).commands,
                                );
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(3 as ::core::ffi::c_int) as isize))
                                        .elif
                                        .commands,
                                );
                            }
                        }
                        39 => {
                            if (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).flag != 0 {
                                yyval.elif.flag = 1 as ::core::ffi::c_int;
                                yyval.elif.commands =
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands;
                            } else {
                                yyval.elif.flag = 0 as ::core::ffi::c_int;
                                yyval.elif.commands = cmd_parse_new_commands();
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands,
                                );
                            }
                        }
                        40 => {
                            if (*yyvsp.offset(-(2 as ::core::ffi::c_int) as isize)).flag != 0 {
                                yyval.elif.flag = 1 as ::core::ffi::c_int;
                                yyval.elif.commands =
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize))
                                        .elif
                                        .commands,
                                );
                            } else if (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).elif.flag
                                != 0
                            {
                                yyval.elif.flag = 1 as ::core::ffi::c_int;
                                yyval.elif.commands = (*yyvsp
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .elif
                                .commands;
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands,
                                );
                            } else {
                                yyval.elif.flag = 0 as ::core::ffi::c_int;
                                yyval.elif.commands = cmd_parse_new_commands();
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands,
                                );
                                cmd_parse_free_commands(
                                    (*yyvsp.offset(0 as ::core::ffi::c_int as isize))
                                        .elif
                                        .commands,
                                );
                            }
                        }
                        41 => {
                            yyval.arguments = xcalloc(
                                1 as size_t,
                                ::core::mem::size_of::<cmd_parse_arguments>() as size_t,
                            )
                                as *mut cmd_parse_arguments;
                            (*yyval.arguments).tqh_first =
                                ::core::ptr::null_mut::<cmd_parse_argument>();
                            (*yyval.arguments).tqh_last = &raw mut (*yyval.arguments).tqh_first;
                            let ref mut fresh17 =
                                (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).argument)
                                    .entry
                                    .tqe_next;
                            *fresh17 = (*yyval.arguments).tqh_first;
                            if !(*fresh17).is_null() {
                                (*(*yyval.arguments).tqh_first).entry.tqe_prev =
                                    &raw mut (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize))
                                        .argument)
                                        .entry
                                        .tqe_next;
                            } else {
                                (*yyval.arguments).tqh_last = &raw mut (*(*yyvsp
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .argument)
                                    .entry
                                    .tqe_next;
                            }
                            (*yyval.arguments).tqh_first =
                                (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).argument;
                            let ref mut fresh18 =
                                (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).argument)
                                    .entry
                                    .tqe_prev;
                            *fresh18 = &raw mut (*yyval.arguments).tqh_first;
                        }
                        42 => {
                            let ref mut fresh19 =
                                (*(*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).argument)
                                    .entry
                                    .tqe_next;
                            *fresh19 = (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize))
                                .arguments)
                                .tqh_first;
                            if !(*fresh19).is_null() {
                                let ref mut fresh20 = (*(*(*yyvsp
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .arguments)
                                    .tqh_first)
                                    .entry
                                    .tqe_prev;
                                *fresh20 = &raw mut (*(*yyvsp
                                    .offset(-(1 as ::core::ffi::c_int) as isize))
                                .argument)
                                    .entry
                                    .tqe_next;
                            } else {
                                let ref mut fresh21 =
                                    (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).arguments)
                                        .tqh_last;
                                *fresh21 = &raw mut (*(*yyvsp
                                    .offset(-(1 as ::core::ffi::c_int) as isize))
                                .argument)
                                    .entry
                                    .tqe_next;
                            }
                            let ref mut fresh22 =
                                (*(*yyvsp.offset(0 as ::core::ffi::c_int as isize)).arguments)
                                    .tqh_first;
                            *fresh22 =
                                (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).argument;
                            let ref mut fresh23 =
                                (*(*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).argument)
                                    .entry
                                    .tqe_prev;
                            *fresh23 = &raw mut (*(*yyvsp
                                .offset(0 as ::core::ffi::c_int as isize))
                            .arguments)
                                .tqh_first;
                            yyval.arguments =
                                (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).arguments;
                        }
                        43 => {
                            yyval.argument = xcalloc(
                                1 as size_t,
                                ::core::mem::size_of::<cmd_parse_argument>() as size_t,
                            )
                                as *mut cmd_parse_argument;
                            (*yyval.argument).type_0 = CMD_PARSE_STRING;
                            (*yyval.argument).string =
                                (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token;
                        }
                        44 => {
                            yyval.argument = xcalloc(
                                1 as size_t,
                                ::core::mem::size_of::<cmd_parse_argument>() as size_t,
                            )
                                as *mut cmd_parse_argument;
                            (*yyval.argument).type_0 = CMD_PARSE_STRING;
                            (*yyval.argument).string =
                                (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).token;
                        }
                        45 => {
                            yyval.argument = xcalloc(
                                1 as size_t,
                                ::core::mem::size_of::<cmd_parse_argument>() as size_t,
                            )
                                as *mut cmd_parse_argument;
                            (*yyval.argument).type_0 = CMD_PARSE_COMMANDS;
                            (*yyval.argument).commands =
                                (*yyvsp.offset(0 as ::core::ffi::c_int as isize)).commands
                                    as *mut cmd_parse_commands;
                        }
                        46 => {
                            yyval.commands =
                                (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands;
                        }
                        47 => {
                            yyval.commands =
                                (*yyvsp.offset(-(2 as ::core::ffi::c_int) as isize)).commands;
                            if !(*(*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands)
                                .tqh_first
                                .is_null()
                            {
                                *(*yyval.commands).tqh_last = (*(*yyvsp
                                    .offset(-(1 as ::core::ffi::c_int) as isize))
                                .commands)
                                    .tqh_first;
                                let ref mut fresh24 = (*(*(*yyvsp
                                    .offset(-(1 as ::core::ffi::c_int) as isize))
                                .commands)
                                    .tqh_first)
                                    .entry
                                    .tqe_prev;
                                *fresh24 = (*yyval.commands).tqh_last;
                                (*yyval.commands).tqh_last = (*(*yyvsp
                                    .offset(-(1 as ::core::ffi::c_int) as isize))
                                .commands)
                                    .tqh_last;
                                let ref mut fresh25 = (*(*yyvsp
                                    .offset(-(1 as ::core::ffi::c_int) as isize))
                                .commands)
                                    .tqh_first;
                                *fresh25 = ::core::ptr::null_mut::<cmd_parse_command>();
                                let ref mut fresh26 = (*(*yyvsp
                                    .offset(-(1 as ::core::ffi::c_int) as isize))
                                .commands)
                                    .tqh_last;
                                *fresh26 = &raw mut (*(*yyvsp
                                    .offset(-(1 as ::core::ffi::c_int) as isize))
                                .commands)
                                    .tqh_first;
                            }
                            free(
                                (*yyvsp.offset(-(1 as ::core::ffi::c_int) as isize)).commands
                                    as *mut ::core::ffi::c_void,
                            );
                        }
                        _ => {}
                    }
                    yyvsp = yyvsp.offset(-(yylen as isize));
                    yyssp = yyssp.offset(-(yylen as isize));
                    yylen = 0 as ::core::ffi::c_int;
                    yyvsp = yyvsp.offset(1);
                    *yyvsp = yyval;
                    let yylhs: ::core::ffi::c_int =
                        yyr1[yyn as usize] as ::core::ffi::c_int - YYNTOKENS;
                    let yyi: ::core::ffi::c_int = yypgoto[yylhs as usize] as ::core::ffi::c_int
                        + *yyssp as ::core::ffi::c_int;
                    yystate = (if 0 as ::core::ffi::c_int <= yyi
                        && yyi <= YYLAST
                        && yycheck[yyi as usize] as ::core::ffi::c_int
                            == *yyssp as ::core::ffi::c_int
                    {
                        yytable[yyi as usize] as ::core::ffi::c_int
                    } else {
                        yydefgoto[yylhs as usize] as ::core::ffi::c_int
                    }) as yy_state_fast_t;
                }
                _ => {}
            }
            yyssp = yyssp.offset(1);
        }
    }
    match current_block {
        18325888159776088768 => {
            yyerror(b"memory exhausted\0" as *const u8 as *const ::core::ffi::c_char);
            yyresult = 2 as ::core::ffi::c_int;
        }
        7016822936316948504 => {
            yyresult = 1 as ::core::ffi::c_int;
        }
        _ => {}
    }
    if yychar != YYEMPTY {
        yytoken = (if 0 as ::core::ffi::c_int <= yychar && yychar <= YYMAXUTOK {
            yytranslate[yychar as usize] as yysymbol_kind_t as ::core::ffi::c_int
        } else {
            YYSYMBOL_YYUNDEF as ::core::ffi::c_int
        }) as yysymbol_kind_t;
        yydestruct(
            b"Cleanup: discarding lookahead\0" as *const u8 as *const ::core::ffi::c_char,
            yytoken,
            &raw mut yylval,
        );
    }
    yyvsp = yyvsp.offset(-(yylen as isize));
    yyssp = yyssp.offset(-(yylen as isize));
    while yyssp != yyss {
        yydestruct(
            b"Cleanup: popping\0" as *const u8 as *const ::core::ffi::c_char,
            yystos[*yyssp as ::core::ffi::c_int as usize] as yysymbol_kind_t,
            yyvsp,
        );
        yyvsp = yyvsp.offset(-(1 as ::core::ffi::c_int as isize));
        yyssp = yyssp.offset(-(1 as ::core::ffi::c_int as isize));
    }
    if yyss != &raw mut yyssa as *mut yy_state_t {
        free(yyss as *mut ::core::ffi::c_void);
    }
    return yyresult;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_parse_from_arguments(
    mut values: *mut args_value,
    mut count: u_int,
    mut pi: *mut cmd_parse_input,
) -> *mut cmd_parse_result {
    static mut pr: cmd_parse_result = cmd_parse_result {
        status: CMD_PARSE_ERROR,
        cmdlist: ::core::ptr::null::<cmd_list>() as *mut cmd_list,
        error: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
    };
    let mut input: cmd_parse_input = cmd_parse_input {
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
    let mut cmds: *mut cmd_parse_commands = ::core::ptr::null_mut::<cmd_parse_commands>();
    let mut cmd: *mut cmd_parse_command = ::core::ptr::null_mut::<cmd_parse_command>();
    let mut arg: *mut cmd_parse_argument = ::core::ptr::null_mut::<cmd_parse_argument>();
    let mut i: u_int = 0;
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut size: size_t = 0;
    let mut end: ::core::ffi::c_int = 0;
    if pi.is_null() {
        memset(
            &raw mut input as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<cmd_parse_input>() as size_t,
        );
        pi = &raw mut input;
    }
    memset(
        &raw mut pr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<cmd_parse_result>() as size_t,
    );
    cmds = cmd_parse_new_commands();
    cmd = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<cmd_parse_command>() as size_t,
    ) as *mut cmd_parse_command;
    (*cmd).line = (*pi).line;
    (*cmd).arguments.tqh_first = ::core::ptr::null_mut::<cmd_parse_argument>();
    (*cmd).arguments.tqh_last = &raw mut (*cmd).arguments.tqh_first;
    i = 0 as u_int;
    while i < count {
        end = 0 as ::core::ffi::c_int;
        if (*values.offset(i as isize)).type_0 as ::core::ffi::c_uint
            == ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            copy = xstrdup((*values.offset(i as isize)).c2rust_unnamed.string);
            size = strlen(copy);
            if size != 0 as size_t
                && *copy.offset(size.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    == ';' as i32
            {
                size = size.wrapping_sub(1);
                *copy.offset(size as isize) = '\0' as i32 as ::core::ffi::c_char;
                if size > 0 as size_t
                    && *copy.offset(size.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                        == '\\' as i32
                {
                    *copy.offset(size.wrapping_sub(1 as size_t) as isize) =
                        ';' as i32 as ::core::ffi::c_char;
                } else {
                    end = 1 as ::core::ffi::c_int;
                }
            }
            if end == 0 || size != 0 as size_t {
                arg = xcalloc(
                    1 as size_t,
                    ::core::mem::size_of::<cmd_parse_argument>() as size_t,
                ) as *mut cmd_parse_argument;
                (*arg).type_0 = CMD_PARSE_STRING;
                (*arg).string = copy;
                (*arg).entry.tqe_next = ::core::ptr::null_mut::<cmd_parse_argument>();
                (*arg).entry.tqe_prev = (*cmd).arguments.tqh_last;
                *(*cmd).arguments.tqh_last = arg;
                (*cmd).arguments.tqh_last = &raw mut (*arg).entry.tqe_next;
            } else {
                free(copy as *mut ::core::ffi::c_void);
            }
        } else if (*values.offset(i as isize)).type_0 as ::core::ffi::c_uint
            == ARGS_COMMANDS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            arg = xcalloc(
                1 as size_t,
                ::core::mem::size_of::<cmd_parse_argument>() as size_t,
            ) as *mut cmd_parse_argument;
            (*arg).type_0 = CMD_PARSE_PARSED_COMMANDS;
            (*arg).cmdlist = (*values.offset(i as isize)).c2rust_unnamed.cmdlist as *mut cmd_list;
            (*(*arg).cmdlist).references += 1;
            (*arg).entry.tqe_next = ::core::ptr::null_mut::<cmd_parse_argument>();
            (*arg).entry.tqe_prev = (*cmd).arguments.tqh_last;
            *(*cmd).arguments.tqh_last = arg;
            (*cmd).arguments.tqh_last = &raw mut (*arg).entry.tqe_next;
        } else {
            fatalx(b"unknown argument type\0" as *const u8 as *const ::core::ffi::c_char);
        }
        if end != 0 {
            (*cmd).entry.tqe_next = ::core::ptr::null_mut::<cmd_parse_command>();
            (*cmd).entry.tqe_prev = (*cmds).tqh_last;
            *(*cmds).tqh_last = cmd;
            (*cmds).tqh_last = &raw mut (*cmd).entry.tqe_next;
            cmd = xcalloc(
                1 as size_t,
                ::core::mem::size_of::<cmd_parse_command>() as size_t,
            ) as *mut cmd_parse_command;
            (*cmd).line = (*pi).line;
            (*cmd).arguments.tqh_first = ::core::ptr::null_mut::<cmd_parse_argument>();
            (*cmd).arguments.tqh_last = &raw mut (*cmd).arguments.tqh_first;
        }
        i = i.wrapping_add(1);
    }
    if !(*cmd).arguments.tqh_first.is_null() {
        (*cmd).entry.tqe_next = ::core::ptr::null_mut::<cmd_parse_command>();
        (*cmd).entry.tqe_prev = (*cmds).tqh_last;
        *(*cmds).tqh_last = cmd;
        (*cmds).tqh_last = &raw mut (*cmd).entry.tqe_next;
    } else {
        free(cmd as *mut ::core::ffi::c_void);
    }
    cmd_parse_build_commands(cmds, pi, &raw mut pr);
    cmd_parse_free_commands(cmds);
    return &raw mut pr;
}
unsafe extern "C" fn yyerror(mut fmt: *const ::core::ffi::c_char, mut args: ...) {
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    let mut pi: *mut cmd_parse_input = (*ps).input;
    let mut ap: ::core::ffi::VaList;
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(*ps).error.is_null() {
        return;
    }
    ap = args.clone();
    xvasprintf(&raw mut error, fmt, ap);
    (*ps).error = cmd_parse_get_error((*pi).file, (*pi).line, error);
    free(error as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn yylex_is_var(
    mut ch: ::core::ffi::c_char,
    mut first: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if ch as ::core::ffi::c_int == '=' as i32 {
        return 0 as ::core::ffi::c_int;
    }
    if first != 0
        && *(*__ctype_b_loc()).offset(ch as u_char as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    return (*(*__ctype_b_loc()).offset(ch as u_char as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
        || ch as ::core::ffi::c_int == '_' as i32) as ::core::ffi::c_int;
}
unsafe extern "C" fn yylex_append(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut len: *mut size_t,
    mut add: *const ::core::ffi::c_char,
    mut addlen: size_t,
) {
    if addlen > (SIZE_MAX as size_t).wrapping_sub(1 as size_t)
        || *len
            > (SIZE_MAX as size_t)
                .wrapping_sub(1 as size_t)
                .wrapping_sub(addlen)
    {
        fatalx(b"buffer is too big\0" as *const u8 as *const ::core::ffi::c_char);
    }
    *buf = xrealloc(
        *buf as *mut ::core::ffi::c_void,
        (*len).wrapping_add(1 as size_t).wrapping_add(addlen),
    ) as *mut ::core::ffi::c_char;
    memcpy(
        (*buf).offset(*len as isize) as *mut ::core::ffi::c_void,
        add as *const ::core::ffi::c_void,
        addlen,
    );
    *len = (*len).wrapping_add(addlen);
}
unsafe extern "C" fn yylex_append1(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut len: *mut size_t,
    mut add: ::core::ffi::c_char,
) {
    yylex_append(buf, len, &raw mut add, 1 as size_t);
}
unsafe extern "C" fn yylex_getc1() -> ::core::ffi::c_int {
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    let mut ch: ::core::ffi::c_int = 0;
    if !(*ps).f.is_null() {
        ch = getc((*ps).f);
    } else if (*ps).off == (*ps).len {
        ch = EOF;
    } else {
        let fresh27 = (*ps).off;
        (*ps).off = (*ps).off.wrapping_add(1);
        ch = *(*ps).buf.offset(fresh27 as isize) as ::core::ffi::c_int;
    }
    return ch;
}
unsafe extern "C" fn yylex_ungetc(mut ch: ::core::ffi::c_int) {
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    if !(*ps).f.is_null() {
        ungetc(ch, (*ps).f);
    } else if (*ps).off > 0 as size_t && ch != EOF {
        (*ps).off = (*ps).off.wrapping_sub(1);
    }
}
unsafe extern "C" fn yylex_getc() -> ::core::ffi::c_int {
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    let mut ch: ::core::ffi::c_int = 0;
    if (*ps).escapes != 0 as u_int {
        (*ps).escapes = (*ps).escapes.wrapping_sub(1);
        return '\\' as i32;
    }
    loop {
        ch = yylex_getc1();
        if ch == '\\' as i32 {
            (*ps).escapes = (*ps).escapes.wrapping_add(1);
        } else if ch == '\n' as i32 && (*ps).escapes.wrapping_rem(2 as u_int) == 1 as u_int {
            (*(*ps).input).line = (*(*ps).input).line.wrapping_add(1);
            (*ps).escapes = (*ps).escapes.wrapping_sub(1);
        } else {
            if (*ps).escapes != 0 as u_int {
                yylex_ungetc(ch);
                (*ps).escapes = (*ps).escapes.wrapping_sub(1);
                return '\\' as i32;
            }
            return ch;
        }
    }
}
unsafe extern "C" fn yylex_get_word(mut ch: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    len = 0 as size_t;
    buf = xmalloc(1 as size_t) as *mut ::core::ffi::c_char;
    loop {
        yylex_append1(&raw mut buf, &raw mut len, ch as ::core::ffi::c_char);
        ch = yylex_getc();
        if !(ch != EOF
            && strchr(b" \t\n\0" as *const u8 as *const ::core::ffi::c_char, ch).is_null())
        {
            break;
        }
    }
    yylex_ungetc(ch);
    *buf.offset(len as isize) = '\0' as i32 as ::core::ffi::c_char;
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"yylex_get_word\0" as *const u8 as *const ::core::ffi::c_char,
        buf,
    );
    return buf;
}
unsafe extern "C" fn yylex() -> ::core::ffi::c_int {
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    let mut token: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ch: ::core::ffi::c_int = 0;
    let mut next: ::core::ffi::c_int = 0;
    let mut condition: ::core::ffi::c_int = 0;
    if (*ps).eol != 0 {
        (*(*ps).input).line = (*(*ps).input).line.wrapping_add(1);
    }
    (*ps).eol = 0 as ::core::ffi::c_int;
    condition = (*ps).condition;
    (*ps).condition = 0 as ::core::ffi::c_int;
    loop {
        ch = yylex_getc();
        if ch == EOF {
            if (*ps).eof != 0 {
                break;
            }
            (*ps).eof = 1 as ::core::ffi::c_int;
            return '\n' as i32;
        } else {
            if ch == ' ' as i32 || ch == '\t' as i32 {
                continue;
            }
            if ch == '\r' as i32 {
                ch = yylex_getc();
                if ch != '\n' as i32 {
                    yylex_ungetc(ch);
                    ch = '\r' as i32;
                }
            }
            if ch == '\n' as i32 {
                (*ps).eol = 1 as ::core::ffi::c_int;
                return '\n' as i32;
            }
            if ch == ';' as i32 || ch == '{' as i32 || ch == '}' as i32 {
                return ch;
            }
            if ch == '#' as i32 {
                next = yylex_getc();
                if condition != 0 && next == '{' as i32 {
                    yylval.token = yylex_format();
                    if yylval.token.is_null() {
                        return 258 as ::core::ffi::c_int;
                    }
                    return 264 as ::core::ffi::c_int;
                }
                while next != '\n' as i32 && next != EOF {
                    next = yylex_getc();
                }
                if next == '\n' as i32 {
                    (*(*ps).input).line = (*(*ps).input).line.wrapping_add(1);
                    return '\n' as i32;
                }
            } else {
                if ch == '%' as i32 {
                    yylval.token = yylex_get_word('%' as i32);
                    cp = yylval.token;
                    while *cp as ::core::ffi::c_int != '\0' as i32 {
                        if *cp as ::core::ffi::c_int != '%' as i32
                            && *(*__ctype_b_loc())
                                .offset(*cp as u_char as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                                    as ::core::ffi::c_int
                                == 0
                        {
                            break;
                        }
                        cp = cp.offset(1);
                    }
                    if *cp as ::core::ffi::c_int == '\0' as i32 {
                        return 265 as ::core::ffi::c_int;
                    }
                    (*ps).condition = 1 as ::core::ffi::c_int;
                    if strcmp(
                        yylval.token,
                        b"%hidden\0" as *const u8 as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        free(yylval.token as *mut ::core::ffi::c_void);
                        return 259 as ::core::ffi::c_int;
                    }
                    if strcmp(
                        yylval.token,
                        b"%if\0" as *const u8 as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        free(yylval.token as *mut ::core::ffi::c_void);
                        return 260 as ::core::ffi::c_int;
                    }
                    if strcmp(
                        yylval.token,
                        b"%else\0" as *const u8 as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        free(yylval.token as *mut ::core::ffi::c_void);
                        return 261 as ::core::ffi::c_int;
                    }
                    if strcmp(
                        yylval.token,
                        b"%elif\0" as *const u8 as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        free(yylval.token as *mut ::core::ffi::c_void);
                        return 262 as ::core::ffi::c_int;
                    }
                    if strcmp(
                        yylval.token,
                        b"%endif\0" as *const u8 as *const ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        free(yylval.token as *mut ::core::ffi::c_void);
                        return 263 as ::core::ffi::c_int;
                    }
                    free(yylval.token as *mut ::core::ffi::c_void);
                    return 258 as ::core::ffi::c_int;
                }
                token = yylex_token(ch);
                if token.is_null() {
                    return 258 as ::core::ffi::c_int;
                }
                yylval.token = token;
                if !strchr(token, '=' as i32).is_null()
                    && yylex_is_var(*token, 1 as ::core::ffi::c_int) != 0
                {
                    cp = token.offset(1 as ::core::ffi::c_int as isize);
                    while *cp as ::core::ffi::c_int != '=' as i32 {
                        if yylex_is_var(*cp, 0 as ::core::ffi::c_int) == 0 {
                            break;
                        }
                        cp = cp.offset(1);
                    }
                    if *cp as ::core::ffi::c_int == '=' as i32 {
                        return 266 as ::core::ffi::c_int;
                    }
                }
                return 265 as ::core::ffi::c_int;
            }
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn yylex_format() -> *mut ::core::ffi::c_char {
    let mut current_block: u64;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut ch: ::core::ffi::c_int = 0;
    let mut brackets: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    len = 0 as size_t;
    buf = xmalloc(1 as size_t) as *mut ::core::ffi::c_char;
    yylex_append(
        &raw mut buf,
        &raw mut len,
        b"#{\0" as *const u8 as *const ::core::ffi::c_char,
        2 as size_t,
    );
    loop {
        ch = yylex_getc();
        if ch == EOF || ch == '\n' as i32 {
            current_block = 14771530845615209615;
            break;
        }
        if ch == '#' as i32 {
            ch = yylex_getc();
            if ch == EOF || ch == '\n' as i32 {
                current_block = 14771530845615209615;
                break;
            }
            if ch == '{' as i32 {
                brackets += 1;
            }
            yylex_append1(
                &raw mut buf,
                &raw mut len,
                '#' as i32 as ::core::ffi::c_char,
            );
        } else if ch == '}' as i32 {
            if brackets != 0 as ::core::ffi::c_int && {
                brackets -= 1;
                brackets == 0 as ::core::ffi::c_int
            } {
                yylex_append1(&raw mut buf, &raw mut len, ch as ::core::ffi::c_char);
                current_block = 10048703153582371463;
                break;
            }
        }
        yylex_append1(&raw mut buf, &raw mut len, ch as ::core::ffi::c_char);
    }
    match current_block {
        10048703153582371463 => {
            if !(brackets != 0 as ::core::ffi::c_int) {
                *buf.offset(len as isize) = '\0' as i32 as ::core::ffi::c_char;
                log_debug(
                    b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    b"yylex_format\0" as *const u8 as *const ::core::ffi::c_char,
                    buf,
                );
                return buf;
            }
        }
        _ => {}
    }
    free(buf as *mut ::core::ffi::c_void);
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn yylex_token_escape(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut len: *mut size_t,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut ch: ::core::ffi::c_int = 0;
    let mut type_0: ::core::ffi::c_int = 0;
    let mut o2: ::core::ffi::c_int = 0;
    let mut o3: ::core::ffi::c_int = 0;
    let mut mlen: ::core::ffi::c_int = 0;
    let mut size: u_int = 0;
    let mut i: u_int = 0;
    let mut tmp: u_int = 0;
    let mut s: [::core::ffi::c_char; 9] = [0; 9];
    let mut m: [::core::ffi::c_char; 16] = [0; 16];
    ch = yylex_getc();
    if ch >= '4' as i32 && ch <= '7' as i32 {
        yyerror(b"invalid octal escape\0" as *const u8 as *const ::core::ffi::c_char);
        return 0 as ::core::ffi::c_int;
    }
    if ch >= '0' as i32 && ch <= '3' as i32 {
        o2 = yylex_getc();
        if o2 >= '0' as i32 && o2 <= '7' as i32 {
            o3 = yylex_getc();
            if o3 >= '0' as i32 && o3 <= '7' as i32 {
                ch = 64 as ::core::ffi::c_int * (ch - '0' as i32)
                    + 8 as ::core::ffi::c_int * (o2 - '0' as i32)
                    + (o3 - '0' as i32);
                yylex_append1(buf, len, ch as ::core::ffi::c_char);
                return 1 as ::core::ffi::c_int;
            }
        }
        yyerror(b"invalid octal escape\0" as *const u8 as *const ::core::ffi::c_char);
        return 0 as ::core::ffi::c_int;
    }
    match ch {
        EOF => return 0 as ::core::ffi::c_int,
        97 => {
            ch = '\u{7}' as i32;
            current_block = 17281240262373992796;
        }
        98 => {
            ch = '\u{8}' as i32;
            current_block = 17281240262373992796;
        }
        101 => {
            ch = '\u{1b}' as i32;
            current_block = 17281240262373992796;
        }
        102 => {
            ch = '\u{c}' as i32;
            current_block = 17281240262373992796;
        }
        115 => {
            ch = ' ' as i32;
            current_block = 17281240262373992796;
        }
        118 => {
            ch = '\u{b}' as i32;
            current_block = 17281240262373992796;
        }
        114 => {
            ch = '\r' as i32;
            current_block = 17281240262373992796;
        }
        110 => {
            ch = '\n' as i32;
            current_block = 17281240262373992796;
        }
        116 => {
            ch = '\t' as i32;
            current_block = 17281240262373992796;
        }
        117 => {
            type_0 = 'u' as i32;
            size = 4 as u_int;
            current_block = 16115080005730334406;
        }
        85 => {
            type_0 = 'U' as i32;
            size = 8 as u_int;
            current_block = 16115080005730334406;
        }
        _ => {
            current_block = 17281240262373992796;
        }
    }
    match current_block {
        16115080005730334406 => {
            i = 0 as u_int;
            while i < size {
                ch = yylex_getc();
                if ch == EOF || ch == '\n' as i32 {
                    return 0 as ::core::ffi::c_int;
                }
                if *(*__ctype_b_loc()).offset(ch as u_char as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    & _ISxdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0
                {
                    yyerror(
                        b"invalid \\%c argument\0" as *const u8 as *const ::core::ffi::c_char,
                        type_0,
                    );
                    return 0 as ::core::ffi::c_int;
                }
                s[i as usize] = ch as ::core::ffi::c_char;
                i = i.wrapping_add(1);
            }
            s[i as usize] = '\0' as i32 as ::core::ffi::c_char;
            if size == 4 as u_int
                && sscanf(
                    &raw mut s as *mut ::core::ffi::c_char,
                    b"%4x\0" as *const u8 as *const ::core::ffi::c_char,
                    &raw mut tmp,
                ) != 1 as ::core::ffi::c_int
                || size == 8 as u_int
                    && sscanf(
                        &raw mut s as *mut ::core::ffi::c_char,
                        b"%8x\0" as *const u8 as *const ::core::ffi::c_char,
                        &raw mut tmp,
                    ) != 1 as ::core::ffi::c_int
            {
                yyerror(
                    b"invalid \\%c argument\0" as *const u8 as *const ::core::ffi::c_char,
                    type_0,
                );
                return 0 as ::core::ffi::c_int;
            }
            mlen = wctomb(&raw mut m as *mut ::core::ffi::c_char, tmp as wchar_t);
            if mlen <= 0 as ::core::ffi::c_int
                || mlen > ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as ::core::ffi::c_int
            {
                yyerror(
                    b"invalid \\%c argument\0" as *const u8 as *const ::core::ffi::c_char,
                    type_0,
                );
                return 0 as ::core::ffi::c_int;
            }
            yylex_append(
                buf,
                len,
                &raw mut m as *mut ::core::ffi::c_char,
                mlen as size_t,
            );
            return 1 as ::core::ffi::c_int;
        }
        _ => {
            yylex_append1(buf, len, ch as ::core::ffi::c_char);
            return 1 as ::core::ffi::c_int;
        }
    };
}
unsafe extern "C" fn yylex_token_variable(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut len: *mut size_t,
) -> ::core::ffi::c_int {
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut ch: ::core::ffi::c_int = 0;
    let mut brackets: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut name: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut namelen: size_t = 0 as size_t;
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    ch = yylex_getc();
    if ch == EOF {
        return 0 as ::core::ffi::c_int;
    }
    if ch == '{' as i32 {
        brackets = 1 as ::core::ffi::c_int;
    } else {
        if yylex_is_var(ch as ::core::ffi::c_char, 1 as ::core::ffi::c_int) == 0 {
            yylex_append1(buf, len, '$' as i32 as ::core::ffi::c_char);
            yylex_ungetc(ch);
            return 1 as ::core::ffi::c_int;
        }
        let fresh28 = namelen;
        namelen = namelen.wrapping_add(1);
        name[fresh28 as usize] = ch as ::core::ffi::c_char;
    }
    loop {
        ch = yylex_getc();
        if brackets != 0 && ch == '}' as i32 {
            break;
        }
        if ch == EOF || yylex_is_var(ch as ::core::ffi::c_char, 0 as ::core::ffi::c_int) == 0 {
            if brackets == 0 {
                yylex_ungetc(ch);
                break;
            } else {
                yyerror(
                    b"invalid environment variable\0" as *const u8 as *const ::core::ffi::c_char,
                );
                return 0 as ::core::ffi::c_int;
            }
        } else {
            if namelen
                == (::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as usize)
                    .wrapping_sub(2 as usize)
            {
                yyerror(
                    b"environment variable is too long\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
                return 0 as ::core::ffi::c_int;
            }
            let fresh29 = namelen;
            namelen = namelen.wrapping_add(1);
            name[fresh29 as usize] = ch as ::core::ffi::c_char;
        }
    }
    name[namelen as usize] = '\0' as i32 as ::core::ffi::c_char;
    envent = environ_find(global_environ, &raw mut name as *mut ::core::ffi::c_char);
    if !envent.is_null() && !(*envent).value.is_null() {
        value = (*envent).value;
        log_debug(
            b"%s: %s -> %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"yylex_token_variable\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut name as *mut ::core::ffi::c_char,
            value,
        );
        yylex_append(buf, len, value, strlen(value));
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn yylex_token_tilde(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut len: *mut size_t,
) -> ::core::ffi::c_int {
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut ch: ::core::ffi::c_int = 0;
    let mut name: [::core::ffi::c_char; 1024] = [0; 1024];
    let mut namelen: size_t = 0 as size_t;
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    let mut home: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    loop {
        ch = yylex_getc();
        if ch == EOF
            || !strchr(
                b"/ \t\n\"'\0" as *const u8 as *const ::core::ffi::c_char,
                ch,
            )
            .is_null()
        {
            yylex_ungetc(ch);
            break;
        } else {
            if namelen
                == (::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as usize)
                    .wrapping_sub(2 as usize)
            {
                yyerror(b"user name is too long\0" as *const u8 as *const ::core::ffi::c_char);
                return 0 as ::core::ffi::c_int;
            }
            let fresh30 = namelen;
            namelen = namelen.wrapping_add(1);
            name[fresh30 as usize] = ch as ::core::ffi::c_char;
        }
    }
    name[namelen as usize] = '\0' as i32 as ::core::ffi::c_char;
    if *(&raw mut name as *mut ::core::ffi::c_char) as ::core::ffi::c_int == '\0' as i32 {
        envent = environ_find(
            global_environ,
            b"HOME\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if !envent.is_null()
            && !(*envent).value.is_null()
            && *(*envent).value as ::core::ffi::c_int != '\0' as i32
        {
            home = (*envent).value;
        } else {
            pw = getpwuid(getuid());
            if !pw.is_null() {
                home = (*pw).pw_dir;
            }
        }
    } else {
        pw = getpwnam(&raw mut name as *mut ::core::ffi::c_char);
        if !pw.is_null() {
            home = (*pw).pw_dir;
        }
    }
    if home.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: ~%s -> %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"yylex_token_tilde\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut name as *mut ::core::ffi::c_char,
        home,
    );
    yylex_append(buf, len, home, strlen(home));
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn yylex_token(mut ch: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    let mut current_block: u64;
    let mut ps: *mut cmd_parse_state = &raw mut parse_state;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut state: C2RustUnnamed_44 = NONE;
    let mut last: C2RustUnnamed_44 = START;
    len = 0 as size_t;
    buf = xmalloc(1 as size_t) as *mut ::core::ffi::c_char;
    loop {
        if ch == EOF {
            log_debug(
                b"%s: end at EOF\0" as *const u8 as *const ::core::ffi::c_char,
                b"yylex_token\0" as *const u8 as *const ::core::ffi::c_char,
            );
            current_block = 13321564401369230990;
            break;
        } else {
            if state as ::core::ffi::c_uint == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                && ch == '\r' as i32
            {
                ch = yylex_getc();
                if ch != '\n' as i32 {
                    yylex_ungetc(ch);
                    ch = '\r' as i32;
                }
            }
            if ch == '\n' as i32 {
                if state as ::core::ffi::c_uint == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    log_debug(
                        b"%s: end at EOL\0" as *const u8 as *const ::core::ffi::c_char,
                        b"yylex_token\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                    current_block = 13321564401369230990;
                    break;
                } else {
                    (*(*ps).input).line = (*(*ps).input).line.wrapping_add(1);
                }
            }
            if state as ::core::ffi::c_uint == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                && (ch == ' ' as i32 || ch == '\t' as i32)
            {
                log_debug(
                    b"%s: end at WS\0" as *const u8 as *const ::core::ffi::c_char,
                    b"yylex_token\0" as *const u8 as *const ::core::ffi::c_char,
                );
                current_block = 13321564401369230990;
                break;
            } else if state as ::core::ffi::c_uint
                == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                && (ch == ';' as i32 || ch == '}' as i32)
            {
                log_debug(
                    b"%s: end at %c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"yylex_token\0" as *const u8 as *const ::core::ffi::c_char,
                    ch,
                );
                current_block = 13321564401369230990;
                break;
            } else if ch == '\n' as i32
                && state as ::core::ffi::c_uint != NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                yylex_append1(
                    &raw mut buf,
                    &raw mut len,
                    '\n' as i32 as ::core::ffi::c_char,
                );
                loop {
                    ch = yylex_getc();
                    if !(ch == ' ' as i32 || ch == '\t' as i32) {
                        break;
                    }
                }
                if ch != '#' as i32 {
                    continue;
                }
                ch = yylex_getc();
                if !strchr(b",#{}:\0" as *const u8 as *const ::core::ffi::c_char, ch).is_null() {
                    yylex_ungetc(ch);
                    ch = '#' as i32;
                } else {
                    loop {
                        ch = yylex_getc();
                        if !(ch != '\n' as i32 && ch != EOF) {
                            break;
                        }
                    }
                }
            } else {
                if ch == '\\' as i32
                    && state as ::core::ffi::c_uint
                        != SINGLE_QUOTES as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if yylex_token_escape(&raw mut buf, &raw mut len) == 0 {
                        current_block = 9856007333916341158;
                        break;
                    }
                    current_block = 2515528795579319319;
                } else if ch == '~' as i32
                    && last as ::core::ffi::c_uint != state as ::core::ffi::c_uint
                    && state as ::core::ffi::c_uint
                        != SINGLE_QUOTES as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if yylex_token_tilde(&raw mut buf, &raw mut len) == 0 {
                        current_block = 9856007333916341158;
                        break;
                    }
                    current_block = 2515528795579319319;
                } else if ch == '$' as i32
                    && state as ::core::ffi::c_uint
                        != SINGLE_QUOTES as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if yylex_token_variable(&raw mut buf, &raw mut len) == 0 {
                        current_block = 9856007333916341158;
                        break;
                    }
                    current_block = 2515528795579319319;
                } else {
                    if ch == '}' as i32
                        && state as ::core::ffi::c_uint
                            == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        current_block = 9856007333916341158;
                        break;
                    }
                    if ch == '\'' as i32 {
                        if state as ::core::ffi::c_uint
                            == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            state = SINGLE_QUOTES;
                            current_block = 13302697912176427116;
                        } else if state as ::core::ffi::c_uint
                            == SINGLE_QUOTES as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            state = NONE;
                            current_block = 13302697912176427116;
                        } else {
                            current_block = 1847472278776910194;
                        }
                    } else {
                        current_block = 1847472278776910194;
                    }
                    match current_block {
                        13302697912176427116 => {}
                        _ => {
                            if ch == '"' as i32 {
                                if state as ::core::ffi::c_uint
                                    == NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    state = DOUBLE_QUOTES;
                                    current_block = 13302697912176427116;
                                } else if state as ::core::ffi::c_uint
                                    == DOUBLE_QUOTES as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    state = NONE;
                                    current_block = 13302697912176427116;
                                } else {
                                    current_block = 14220266465818359136;
                                }
                            } else {
                                current_block = 14220266465818359136;
                            }
                            match current_block {
                                13302697912176427116 => {}
                                _ => {
                                    yylex_append1(
                                        &raw mut buf,
                                        &raw mut len,
                                        ch as ::core::ffi::c_char,
                                    );
                                    current_block = 2515528795579319319;
                                }
                            }
                        }
                    }
                }
                match current_block {
                    2515528795579319319 => {
                        last = state;
                    }
                    _ => {}
                }
                ch = yylex_getc();
            }
        }
    }
    match current_block {
        9856007333916341158 => {
            free(buf as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        _ => {
            yylex_ungetc(ch);
            *buf.offset(len as isize) = '\0' as i32 as ::core::ffi::c_char;
            log_debug(
                b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                b"yylex_token\0" as *const u8 as *const ::core::ffi::c_char,
                buf,
            );
            return buf;
        }
    };
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

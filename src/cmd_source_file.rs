pub use crate::src::shared::abi::{__size_t, ssize_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{
    CMD_FIND_CANFAIL, CMD_PARSE_PARSEONLY, CMD_PARSE_QUIET, CMD_PARSE_VERBOSE,
    CMD_SOURCE_FILE_DEPTH_LIMIT,
};
pub use crate::src::shared::client::{CLIENT_CONTROL};
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
    pub type stat;
    pub type dirent;
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
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn glob(
        __pattern: *const ::core::ffi::c_char,
        __flags: ::core::ffi::c_int,
        __errfunc: Option<
            unsafe extern "C" fn(
                *const ::core::ffi::c_char,
                ::core::ffi::c_int,
            ) -> ::core::ffi::c_int,
        >,
        __pglob: *mut glob_t,
    ) -> ::core::ffi::c_int;
    fn globfree(__pglob: *mut glob_t);
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strerror(__errnum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn evbuffer_get_length(buf: *const evbuffer) -> size_t;
    fn evbuffer_pullup(buf: *mut evbuffer, size: ssize_t) -> *mut ::core::ffi::c_uchar;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    static mut cfg_finished: ::core::ffi::c_int;
    fn load_cfg_from_buffer(
        _: *const ::core::ffi::c_void,
        _: size_t,
        _: *const ::core::ffi::c_char,
        _: *mut client,
        _: *mut cmdq_item,
        _: *mut cmd_find_state,
        _: ::core::ffi::c_int,
        _: *mut *mut cmdq_item,
    ) -> ::core::ffi::c_int;
    fn cfg_print_causes(_: *mut cmdq_item);
    fn format_single_from_target(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmd_get_parse_flags(_: *mut cmd) -> ::core::ffi::c_int;
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_callback1(
        _: *const ::core::ffi::c_char,
        _: cmdq_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut cmdq_item;
    fn cmdq_insert_after(_: *mut cmdq_item, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_continue(_: *mut cmdq_item);
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn file_read(
        _: *mut client,
        _: *const ::core::ffi::c_char,
        _: client_file_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut client_file;
    fn server_client_unref(_: *mut client);
    fn server_client_get_cwd(_: *mut client, _: *mut session) -> *const ::core::ffi::c_char;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
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
pub struct glob_t {
    pub gl_pathc: __size_t,
    pub gl_pathv: *mut *mut ::core::ffi::c_char,
    pub gl_offs: __size_t,
    pub gl_flags: ::core::ffi::c_int,
    pub gl_closedir: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    pub gl_readdir: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> *mut dirent>,
    pub gl_opendir:
        Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> *mut ::core::ffi::c_void>,
    pub gl_lstat:
        Option<unsafe extern "C" fn(*const ::core::ffi::c_char, *mut stat) -> ::core::ffi::c_int>,
    pub gl_stat:
        Option<unsafe extern "C" fn(*const ::core::ffi::c_char, *mut stat) -> ::core::ffi::c_int>,
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
pub type cmdq_cb =
    Option<unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval>;
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
pub struct cmd_source_file_data {
    pub item: *mut cmdq_item,
    pub client: *mut client,
    pub flags: ::core::ffi::c_int,
    pub after: *mut cmdq_item,
    pub retval: cmd_retval,
    pub current: u_int,
    pub files: *mut *mut ::core::ffi::c_char,
    pub nfiles: u_int,
}
pub const ENOENT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ENOMEM: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const GLOB_NOSPACE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const GLOB_NOMATCH: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
static mut cmd_source_file_depth: u_int = 0;
#[no_mangle]
pub static mut cmd_source_file_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"source-file\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"source\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"t:Fnqv\0" as *const u8 as *const ::core::ffi::c_char,
            lower: 1 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: b"[-Fnqv] [-t target-pane] path ...\0" as *const u8 as *const ::core::ffi::c_char,
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
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_source_file_exec as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_source_file_free_data(mut cdata: *mut cmd_source_file_data) {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < (*cdata).nfiles {
        free(*(*cdata).files.offset(i as isize) as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    free((*cdata).files as *mut ::core::ffi::c_void);
    if !(*cdata).client.is_null() {
        server_client_unref((*cdata).client);
    }
    free(cdata as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn cmd_source_file_complete_cb(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut cdata: *mut cmd_source_file_data = data as *mut cmd_source_file_data;
    let mut c: *mut client = (*cdata).client;
    if c.is_null() {
        cmd_source_file_depth = cmd_source_file_depth.wrapping_sub(1);
        log_debug(
            b"%s: depth now %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_source_file_complete_cb\0" as *const u8 as *const ::core::ffi::c_char,
            cmd_source_file_depth,
        );
    } else {
        (*c).source_file_depth = (*c).source_file_depth.wrapping_sub(1);
        log_debug(
            b"%s: depth now %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_source_file_complete_cb\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).source_file_depth,
        );
    }
    cfg_print_causes(item);
    cmd_source_file_free_data(cdata);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmd_source_file_complete(mut cdata: *mut cmd_source_file_data) {
    let mut c: *mut client = (*cdata).client;
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    if cfg_finished == 0 {
        cmd_source_file_free_data(cdata);
        return;
    }
    if (*cdata).retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int
        && !c.is_null()
        && (*c).session.is_null()
    {
        (*c).retval = 1 as ::core::ffi::c_int;
    }
    new_item = cmdq_get_callback1(
        b"cmd_source_file_complete_cb\0" as *const u8 as *const ::core::ffi::c_char,
        Some(
            cmd_source_file_complete_cb
                as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
        ),
        cdata as *mut ::core::ffi::c_void,
    );
    cmdq_insert_after((*cdata).after, new_item);
}
unsafe extern "C" fn cmd_source_file_done(
    mut oc: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut error: ::core::ffi::c_int,
    mut closed: ::core::ffi::c_int,
    mut buffer: *mut evbuffer,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut cdata: *mut cmd_source_file_data = data as *mut cmd_source_file_data;
    let mut item: *mut cmdq_item = (*cdata).item;
    let mut c: *mut client = (*cdata).client;
    let mut bdata: *mut ::core::ffi::c_void =
        evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t) as *mut ::core::ffi::c_void;
    let mut bsize: size_t = evbuffer_get_length(buffer);
    let mut n: u_int = 0;
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    if closed == 0 {
        return;
    }
    if error != 0 as ::core::ffi::c_int {
        cmdq_error(
            item,
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            strerror(error),
            path,
        );
    } else if bsize != 0 as size_t {
        if load_cfg_from_buffer(
            bdata,
            bsize,
            path,
            c,
            (*cdata).after,
            target,
            (*cdata).flags,
            &raw mut new_item,
        ) < 0 as ::core::ffi::c_int
        {
            (*cdata).retval = CMD_RETURN_ERROR;
        } else if !new_item.is_null() {
            (*cdata).after = new_item;
        }
    }
    (*cdata).current = (*cdata).current.wrapping_add(1);
    n = (*cdata).current;
    if n < (*cdata).nfiles {
        file_read(
            c,
            *(*cdata).files.offset(n as isize),
            Some(
                cmd_source_file_done
                    as unsafe extern "C" fn(
                        *mut client,
                        *const ::core::ffi::c_char,
                        ::core::ffi::c_int,
                        ::core::ffi::c_int,
                        *mut evbuffer,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            cdata as *mut ::core::ffi::c_void,
        );
    } else {
        cmd_source_file_complete(cdata);
        cmdq_continue(item);
    };
}
unsafe extern "C" fn cmd_source_file_add(
    mut cdata: *mut cmd_source_file_data,
    mut path: *const ::core::ffi::c_char,
) {
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_source_file_add\0" as *const u8 as *const ::core::ffi::c_char,
        path,
    );
    (*cdata).files = xreallocarray(
        (*cdata).files as *mut ::core::ffi::c_void,
        (*cdata).nfiles.wrapping_add(1 as u_int) as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
    ) as *mut *mut ::core::ffi::c_char;
    let fresh0 = (*cdata).nfiles;
    (*cdata).nfiles = (*cdata).nfiles.wrapping_add(1);
    let ref mut fresh1 = *(*cdata).files.offset(fresh0 as isize);
    *fresh1 = xstrdup(path);
}
unsafe extern "C" fn cmd_source_file_quote_for_glob(
    mut path: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut quoted: *mut ::core::ffi::c_char = xmalloc(
        (2 as size_t)
            .wrapping_mul(strlen(path))
            .wrapping_add(1 as size_t),
    ) as *mut ::core::ffi::c_char;
    let mut q: *mut ::core::ffi::c_char = quoted;
    let mut p: *const ::core::ffi::c_char = path;
    while *p as ::core::ffi::c_int != '\0' as i32 {
        if (*p as u_char as ::core::ffi::c_int) < 128 as ::core::ffi::c_int
            && *(*__ctype_b_loc()).offset(*p as u_char as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & _ISalnum as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                == 0
            && *p as ::core::ffi::c_int != '/' as i32
        {
            let fresh2 = q;
            q = q.offset(1);
            *fresh2 = '\\' as i32 as ::core::ffi::c_char;
        }
        let fresh3 = p;
        p = p.offset(1);
        let fresh4 = q;
        q = q.offset(1);
        *fresh4 = *fresh3;
    }
    *q = '\0' as i32 as ::core::ffi::c_char;
    return quoted;
}
unsafe extern "C" fn cmd_source_file_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut args: *mut args = cmd_get_args(self_0);
    let mut cdata: *mut cmd_source_file_data = ::core::ptr::null_mut::<cmd_source_file_data>();
    let mut c: *mut client = cmdq_get_client(item);
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    let mut pattern: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cwd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut error: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut g: glob_t = glob_t {
        gl_pathc: 0,
        gl_pathv: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        gl_offs: 0,
        gl_flags: 0,
        gl_closedir: None,
        gl_readdir: None,
        gl_opendir: None,
        gl_lstat: None,
        gl_stat: None,
    };
    let mut result: ::core::ffi::c_int = 0;
    let mut parse_flags: ::core::ffi::c_int = 0;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    if c.is_null() {
        if cmd_source_file_depth >= CMD_SOURCE_FILE_DEPTH_LIMIT as u_int {
            cmdq_error(
                item,
                b"too many nested files\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
        cmd_source_file_depth = cmd_source_file_depth.wrapping_add(1);
        log_debug(
            b"%s: depth now %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_source_file_exec\0" as *const u8 as *const ::core::ffi::c_char,
            cmd_source_file_depth,
        );
    } else {
        if (*c).source_file_depth >= CMD_SOURCE_FILE_DEPTH_LIMIT as u_int {
            cmdq_error(
                item,
                b"too many nested files\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
        (*c).source_file_depth = (*c).source_file_depth.wrapping_add(1);
        log_debug(
            b"%s: depth now %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_source_file_exec\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).source_file_depth,
        );
    }
    cdata = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<cmd_source_file_data>() as size_t,
    ) as *mut cmd_source_file_data;
    (*cdata).item = item;
    (*cdata).client = c;
    if !c.is_null() {
        (*c).references += 1;
    }
    if args_has(args, 'q' as i32 as u_char) != 0 {
        (*cdata).flags |= CMD_PARSE_QUIET;
    }
    if args_has(args, 'n' as i32 as u_char) != 0 {
        (*cdata).flags |= CMD_PARSE_PARSEONLY;
    }
    if c.is_null() || !(*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        parse_flags = cmd_get_parse_flags(self_0);
        if args_has(args, 'v' as i32 as u_char) != 0 || parse_flags & CMD_PARSE_VERBOSE != 0 {
            (*cdata).flags |= CMD_PARSE_VERBOSE;
        }
    }
    cwd = cmd_source_file_quote_for_glob(server_client_get_cwd(
        c,
        ::core::ptr::null_mut::<session>(),
    ));
    i = 0 as u_int;
    while i < args_count(args) {
        path = args_string(args, i);
        if args_has(args, 'F' as i32 as u_char) != 0 {
            free(expanded as *mut ::core::ffi::c_void);
            expanded = format_single_from_target(item, path);
            path = expanded;
        }
        if strcmp(path, b"-\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            cmd_source_file_add(cdata, b"-\0" as *const u8 as *const ::core::ffi::c_char);
        } else {
            if *path as ::core::ffi::c_int == '/' as i32 {
                pattern = xstrdup(path);
            } else {
                xasprintf(
                    &raw mut pattern,
                    b"%s/%s\0" as *const u8 as *const ::core::ffi::c_char,
                    cwd,
                    path,
                );
            }
            log_debug(
                b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                b"cmd_source_file_exec\0" as *const u8 as *const ::core::ffi::c_char,
                pattern,
            );
            result = glob(pattern, 0 as ::core::ffi::c_int, None, &raw mut g);
            if result != 0 as ::core::ffi::c_int {
                if result != GLOB_NOMATCH || !(*cdata).flags & CMD_PARSE_QUIET != 0 {
                    if result == GLOB_NOMATCH {
                        error = strerror(ENOENT);
                    } else if result == GLOB_NOSPACE {
                        error = strerror(ENOMEM);
                    } else {
                        error = strerror(EINVAL);
                    }
                    cmdq_error(
                        item,
                        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        error,
                        path,
                    );
                    retval = CMD_RETURN_ERROR;
                }
                globfree(&raw mut g);
                free(pattern as *mut ::core::ffi::c_void);
            } else {
                free(pattern as *mut ::core::ffi::c_void);
                j = 0 as u_int;
                while (j as __size_t) < g.gl_pathc {
                    cmd_source_file_add(cdata, *g.gl_pathv.offset(j as isize));
                    j = j.wrapping_add(1);
                }
                globfree(&raw mut g);
            }
        }
        i = i.wrapping_add(1);
    }
    free(expanded as *mut ::core::ffi::c_void);
    (*cdata).after = item;
    (*cdata).retval = retval;
    if (*cdata).nfiles != 0 as u_int {
        file_read(
            c,
            *(*cdata).files.offset(0 as ::core::ffi::c_int as isize),
            Some(
                cmd_source_file_done
                    as unsafe extern "C" fn(
                        *mut client,
                        *const ::core::ffi::c_char,
                        ::core::ffi::c_int,
                        ::core::ffi::c_int,
                        *mut evbuffer,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            cdata as *mut ::core::ffi::c_void,
        );
        retval = CMD_RETURN_WAIT;
    } else {
        cmd_source_file_complete(cdata);
    }
    free(cwd as *mut ::core::ffi::c_void);
    return retval;
}

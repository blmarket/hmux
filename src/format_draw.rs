pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_citem, screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
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
use crate::src::shared::utf8::*;
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
    fn strlcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xstrndup(_: *const ::core::ffi::c_char, _: size_t) -> *mut ::core::ffi::c_char;
    fn format_skip(
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    static grid_default_cell: grid_cell;
    fn screen_write_start(_: *mut screen_write_ctx, _: *mut screen);
    fn screen_write_stop(_: *mut screen_write_ctx);
    fn screen_write_putc(_: *mut screen_write_ctx, _: *const grid_cell, _: u_char);
    fn screen_write_fast_copy(
        _: *mut screen_write_ctx,
        _: *mut screen,
        _: u_int,
        _: u_int,
        _: u_int,
        _: u_int,
    );
    fn screen_write_clearendofline(_: *mut screen_write_ctx, _: u_int);
    fn screen_write_cursormove(
        _: *mut screen_write_ctx,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn screen_write_cell(_: *mut screen_write_ctx, _: *const grid_cell);
    fn screen_init(_: *mut screen, _: u_int, _: u_int, _: u_int);
    fn screen_free(_: *mut screen);
    fn utf8_set(_: *mut utf8_data, _: u_char);
    fn utf8_open(_: *mut utf8_data, _: u_char) -> utf8_state;
    fn utf8_append(_: *mut utf8_data, _: u_char) -> utf8_state;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn style_parse(
        _: *mut style,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn style_tostring(_: *mut style) -> *const ::core::ffi::c_char;
    fn style_link(_: *mut style) -> *const ::core::ffi::c_char;
    fn style_set(_: *mut style, _: *const grid_cell);
    fn style_copy(_: *mut style, _: *mut style);
    fn hyperlinks_put(
        _: *mut hyperlinks,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
    ) -> u_int;
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
pub struct screen_write_ctx {
    pub wp: *mut window_pane,
    pub s: *mut screen,
    pub flags: ::core::ffi::c_int,
    pub init_ctx_cb: screen_write_init_ctx_cb,
    pub arg: *mut ::core::ffi::c_void,
    pub item: *mut screen_write_citem,
    pub scrolled: u_int,
    pub bg: u_int,
}
pub type screen_write_init_ctx_cb =
    Option<unsafe extern "C" fn(*mut screen_write_ctx, *mut tty_ctx) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_ctx {
    pub s: *mut screen,
    pub redraw_cb: tty_ctx_redraw_cb,
    pub set_client_cb: tty_ctx_set_client_cb,
    pub arg: *mut ::core::ffi::c_void,
    pub cell: *const grid_cell,
    pub flags: ::core::ffi::c_int,
    pub c2rust_unnamed: C2RustUnnamed_35,
    pub ocx: u_int,
    pub ocy: u_int,
    pub orupper: u_int,
    pub orlower: u_int,
    pub xoff: ::core::ffi::c_int,
    pub yoff: ::core::ffi::c_int,
    pub rxoff: ::core::ffi::c_int,
    pub ryoff: ::core::ffi::c_int,
    pub sx: u_int,
    pub sy: u_int,
    pub bg: u_int,
    pub defaults: grid_cell,
    pub style_ctx: tty_style_ctx,
    pub wox: u_int,
    pub woy: u_int,
    pub wsx: u_int,
    pub wsy: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_style_ctx {
    pub defaults: *const grid_cell,
    pub palette: *mut colour_palette,
    pub dim: u_int,
    pub hyperlinks: *mut hyperlinks,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_35 {
    pub n: u_int,
    pub data: C2RustUnnamed_37,
    pub sel: C2RustUnnamed_36,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_36 {
    pub clip: *const ::core::ffi::c_char,
    pub data: *const ::core::ffi::c_char,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
    pub data: *const ::core::ffi::c_char,
    pub size: size_t,
}
pub type tty_ctx_set_client_cb =
    Option<unsafe extern "C" fn(*mut tty_ctx, *mut client) -> ::core::ffi::c_int>;
pub type tty_ctx_redraw_cb = Option<unsafe extern "C" fn(*const tty_ctx) -> ()>;
pub const TOTAL: C2RustUnnamed_39 = 8;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct format_range {
    pub index: u_int,
    pub s: *mut screen,
    pub start: u_int,
    pub end: u_int,
    pub type_0: style_range_type,
    pub argument: u_int,
    pub string: [::core::ffi::c_char; 16],
    pub entry: C2RustUnnamed_38,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_38 {
    pub tqe_next: *mut format_range,
    pub tqe_prev: *mut *mut format_range,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct format_ranges {
    pub tqh_first: *mut format_range,
    pub tqh_last: *mut *mut format_range,
}
pub const AFTER: C2RustUnnamed_39 = 7;
pub const LIST_RIGHT: C2RustUnnamed_39 = 6;
pub const LIST_LEFT: C2RustUnnamed_39 = 5;
pub const LIST: C2RustUnnamed_39 = 4;
pub const ABSOLUTE_CENTRE: C2RustUnnamed_39 = 3;
pub const RIGHT: C2RustUnnamed_39 = 2;
pub const CENTRE: C2RustUnnamed_39 = 1;
pub const LEFT: C2RustUnnamed_39 = 0;
pub type C2RustUnnamed_39 = ::core::ffi::c_uint;
unsafe extern "C" fn format_is_type(
    mut fr: *mut format_range,
    mut sy: *mut style,
) -> ::core::ffi::c_int {
    if (*fr).type_0 as ::core::ffi::c_uint != (*sy).range_type as ::core::ffi::c_uint {
        return 0 as ::core::ffi::c_int;
    }
    match (*fr).type_0 as ::core::ffi::c_uint {
        0 | 1 | 2 | 7 => return 1 as ::core::ffi::c_int,
        3 | 4 | 5 => {
            return ((*fr).argument == (*sy).range_argument) as ::core::ffi::c_int;
        }
        6 => {
            return (strcmp(
                &raw mut (*fr).string as *mut ::core::ffi::c_char,
                &raw mut (*sy).range_string as *mut ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        }
        _ => {}
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn format_free_range(mut frs: *mut format_ranges, mut fr: *mut format_range) {
    if !(*fr).entry.tqe_next.is_null() {
        (*(*fr).entry.tqe_next).entry.tqe_prev = (*fr).entry.tqe_prev;
    } else {
        (*frs).tqh_last = (*fr).entry.tqe_prev;
    }
    *(*fr).entry.tqe_prev = (*fr).entry.tqe_next;
    free(fr as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn format_update_ranges(
    mut frs: *mut format_ranges,
    mut s: *mut screen,
    mut offset: u_int,
    mut start: u_int,
    mut width: u_int,
) {
    let mut fr: *mut format_range = ::core::ptr::null_mut::<format_range>();
    let mut fr1: *mut format_range = ::core::ptr::null_mut::<format_range>();
    if frs.is_null() {
        return;
    }
    fr = (*frs).tqh_first;
    while !fr.is_null() && {
        fr1 = (*fr).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !((*fr).s != s) {
            if (*fr).end <= start || (*fr).start >= start.wrapping_add(width) {
                format_free_range(frs, fr);
            } else {
                if (*fr).start < start {
                    (*fr).start = start;
                }
                if (*fr).end > start.wrapping_add(width) {
                    (*fr).end = start.wrapping_add(width);
                }
                if (*fr).start == (*fr).end {
                    format_free_range(frs, fr);
                } else {
                    (*fr).start = (*fr).start.wrapping_sub(start);
                    (*fr).end = (*fr).end.wrapping_sub(start);
                    (*fr).start = (*fr).start.wrapping_add(offset);
                    (*fr).end = (*fr).end.wrapping_add(offset);
                }
            }
        }
        fr = fr1;
    }
}
unsafe extern "C" fn format_draw_put(
    mut octx: *mut screen_write_ctx,
    mut ocx: u_int,
    mut ocy: u_int,
    mut s: *mut screen,
    mut frs: *mut format_ranges,
    mut offset: u_int,
    mut start: u_int,
    mut width: u_int,
) {
    screen_write_cursormove(
        octx,
        ocx.wrapping_add(offset) as ::core::ffi::c_int,
        ocy as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_fast_copy(octx, s, start, 0 as u_int, width, 1 as u_int);
    format_update_ranges(frs, s, offset, start, width);
}
unsafe extern "C" fn format_draw_put_list(
    mut octx: *mut screen_write_ctx,
    mut ocx: u_int,
    mut ocy: u_int,
    mut offset: u_int,
    mut width: u_int,
    mut list: *mut screen,
    mut list_left: *mut screen,
    mut list_right: *mut screen,
    mut focus_start: ::core::ffi::c_int,
    mut focus_end: ::core::ffi::c_int,
    mut frs: *mut format_ranges,
) {
    let mut start: u_int = 0;
    let mut focus_centre: u_int = 0;
    if width >= (*list).cx {
        format_draw_put(octx, ocx, ocy, list, frs, offset, 0 as u_int, width);
        return;
    }
    focus_centre = (focus_start + (focus_end - focus_start) / 2 as ::core::ffi::c_int) as u_int;
    if focus_centre < width.wrapping_div(2 as u_int) {
        start = 0 as u_int;
    } else {
        start = focus_centre.wrapping_sub(width.wrapping_div(2 as u_int));
    }
    if start.wrapping_add(width) > (*list).cx {
        start = (*list).cx.wrapping_sub(width);
    }
    if start != 0 as u_int && width > (*list_left).cx {
        screen_write_cursormove(
            octx,
            ocx.wrapping_add(offset) as ::core::ffi::c_int,
            ocy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_fast_copy(
            octx,
            list_left,
            0 as u_int,
            0 as u_int,
            (*list_left).cx,
            1 as u_int,
        );
        offset = offset.wrapping_add((*list_left).cx);
        start = start.wrapping_add((*list_left).cx);
        width = width.wrapping_sub((*list_left).cx);
    }
    if start.wrapping_add(width) < (*list).cx && width > (*list_right).cx {
        screen_write_cursormove(
            octx,
            ocx.wrapping_add(offset)
                .wrapping_add(width)
                .wrapping_sub((*list_right).cx) as ::core::ffi::c_int,
            ocy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_fast_copy(
            octx,
            list_right,
            0 as u_int,
            0 as u_int,
            (*list_right).cx,
            1 as u_int,
        );
        width = width.wrapping_sub((*list_right).cx);
    }
    format_draw_put(octx, ocx, ocy, list, frs, offset, start, width);
}
unsafe extern "C" fn format_draw_none(
    mut octx: *mut screen_write_ctx,
    mut available: u_int,
    mut ocx: u_int,
    mut ocy: u_int,
    mut left: *mut screen,
    mut centre: *mut screen,
    mut right: *mut screen,
    mut abs_centre: *mut screen,
    mut frs: *mut format_ranges,
) {
    let mut width_left: u_int = 0;
    let mut width_centre: u_int = 0;
    let mut width_right: u_int = 0;
    let mut width_abs_centre: u_int = 0;
    width_left = (*left).cx;
    width_centre = (*centre).cx;
    width_right = (*right).cx;
    width_abs_centre = (*abs_centre).cx;
    while width_left
        .wrapping_add(width_centre)
        .wrapping_add(width_right)
        > available
    {
        if width_centre > 0 as u_int {
            width_centre = width_centre.wrapping_sub(1);
        } else if width_right > 0 as u_int {
            width_right = width_right.wrapping_sub(1);
        } else {
            width_left = width_left.wrapping_sub(1);
        }
    }
    format_draw_put(
        octx, ocx, ocy, left, frs, 0 as u_int, 0 as u_int, width_left,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        right,
        frs,
        available.wrapping_sub(width_right),
        (*right).cx.wrapping_sub(width_right),
        width_right,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        centre,
        frs,
        width_left
            .wrapping_add(
                available
                    .wrapping_sub(width_right)
                    .wrapping_sub(width_left)
                    .wrapping_div(2 as u_int),
            )
            .wrapping_sub(width_centre.wrapping_div(2 as u_int)),
        (*centre)
            .cx
            .wrapping_div(2 as u_int)
            .wrapping_sub(width_centre.wrapping_div(2 as u_int)),
        width_centre,
    );
    if width_abs_centre > available {
        width_abs_centre = available;
    }
    format_draw_put(
        octx,
        ocx,
        ocy,
        abs_centre,
        frs,
        available
            .wrapping_sub(width_abs_centre)
            .wrapping_div(2 as u_int),
        0 as u_int,
        width_abs_centre,
    );
}
unsafe extern "C" fn format_draw_left(
    mut octx: *mut screen_write_ctx,
    mut available: u_int,
    mut ocx: u_int,
    mut ocy: u_int,
    mut left: *mut screen,
    mut centre: *mut screen,
    mut right: *mut screen,
    mut abs_centre: *mut screen,
    mut list: *mut screen,
    mut list_left: *mut screen,
    mut list_right: *mut screen,
    mut after: *mut screen,
    mut focus_start: ::core::ffi::c_int,
    mut focus_end: ::core::ffi::c_int,
    mut frs: *mut format_ranges,
) {
    let mut width_left: u_int = 0;
    let mut width_centre: u_int = 0;
    let mut width_right: u_int = 0;
    let mut width_list: u_int = 0;
    let mut width_after: u_int = 0;
    let mut width_abs_centre: u_int = 0;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: ::core::ptr::null_mut::<screen_write_citem>(),
        scrolled: 0,
        bg: 0,
    };
    width_left = (*left).cx;
    width_centre = (*centre).cx;
    width_right = (*right).cx;
    width_abs_centre = (*abs_centre).cx;
    width_list = (*list).cx;
    width_after = (*after).cx;
    while width_left
        .wrapping_add(width_centre)
        .wrapping_add(width_right)
        .wrapping_add(width_list)
        .wrapping_add(width_after)
        > available
    {
        if width_centre > 0 as u_int {
            width_centre = width_centre.wrapping_sub(1);
        } else if width_list > 0 as u_int {
            width_list = width_list.wrapping_sub(1);
        } else if width_right > 0 as u_int {
            width_right = width_right.wrapping_sub(1);
        } else if width_after > 0 as u_int {
            width_after = width_after.wrapping_sub(1);
        } else {
            width_left = width_left.wrapping_sub(1);
        }
    }
    if width_list == 0 as u_int {
        screen_write_start(&raw mut ctx, left);
        screen_write_fast_copy(
            &raw mut ctx,
            after,
            0 as u_int,
            0 as u_int,
            width_after,
            1 as u_int,
        );
        screen_write_stop(&raw mut ctx);
        format_draw_none(
            octx, available, ocx, ocy, left, centre, right, abs_centre, frs,
        );
        return;
    }
    format_draw_put(
        octx, ocx, ocy, left, frs, 0 as u_int, 0 as u_int, width_left,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        right,
        frs,
        available.wrapping_sub(width_right),
        (*right).cx.wrapping_sub(width_right),
        width_right,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        after,
        frs,
        width_left.wrapping_add(width_list),
        0 as u_int,
        width_after,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        centre,
        frs,
        width_left
            .wrapping_add(width_list)
            .wrapping_add(width_after)
            .wrapping_add(
                available
                    .wrapping_sub(width_right)
                    .wrapping_sub(
                        width_left
                            .wrapping_add(width_list)
                            .wrapping_add(width_after),
                    )
                    .wrapping_div(2 as u_int),
            )
            .wrapping_sub(width_centre.wrapping_div(2 as u_int)),
        (*centre)
            .cx
            .wrapping_div(2 as u_int)
            .wrapping_sub(width_centre.wrapping_div(2 as u_int)),
        width_centre,
    );
    if focus_start == -(1 as ::core::ffi::c_int) || focus_end == -(1 as ::core::ffi::c_int) {
        focus_end = 0 as ::core::ffi::c_int;
        focus_start = focus_end;
    }
    format_draw_put_list(
        octx,
        ocx,
        ocy,
        width_left,
        width_list,
        list,
        list_left,
        list_right,
        focus_start,
        focus_end,
        frs,
    );
    if width_abs_centre > available {
        width_abs_centre = available;
    }
    format_draw_put(
        octx,
        ocx,
        ocy,
        abs_centre,
        frs,
        available
            .wrapping_sub(width_abs_centre)
            .wrapping_div(2 as u_int),
        0 as u_int,
        width_abs_centre,
    );
}
unsafe extern "C" fn format_draw_centre(
    mut octx: *mut screen_write_ctx,
    mut available: u_int,
    mut ocx: u_int,
    mut ocy: u_int,
    mut left: *mut screen,
    mut centre: *mut screen,
    mut right: *mut screen,
    mut abs_centre: *mut screen,
    mut list: *mut screen,
    mut list_left: *mut screen,
    mut list_right: *mut screen,
    mut after: *mut screen,
    mut focus_start: ::core::ffi::c_int,
    mut focus_end: ::core::ffi::c_int,
    mut frs: *mut format_ranges,
) {
    let mut width_left: u_int = 0;
    let mut width_centre: u_int = 0;
    let mut width_right: u_int = 0;
    let mut middle: u_int = 0;
    let mut width_list: u_int = 0;
    let mut width_after: u_int = 0;
    let mut width_abs_centre: u_int = 0;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: ::core::ptr::null_mut::<screen_write_citem>(),
        scrolled: 0,
        bg: 0,
    };
    width_left = (*left).cx;
    width_centre = (*centre).cx;
    width_right = (*right).cx;
    width_abs_centre = (*abs_centre).cx;
    width_list = (*list).cx;
    width_after = (*after).cx;
    while width_left
        .wrapping_add(width_centre)
        .wrapping_add(width_right)
        .wrapping_add(width_list)
        .wrapping_add(width_after)
        > available
    {
        if width_list > 0 as u_int {
            width_list = width_list.wrapping_sub(1);
        } else if width_after > 0 as u_int {
            width_after = width_after.wrapping_sub(1);
        } else if width_centre > 0 as u_int {
            width_centre = width_centre.wrapping_sub(1);
        } else if width_right > 0 as u_int {
            width_right = width_right.wrapping_sub(1);
        } else {
            width_left = width_left.wrapping_sub(1);
        }
    }
    if width_list == 0 as u_int {
        screen_write_start(&raw mut ctx, centre);
        screen_write_fast_copy(
            &raw mut ctx,
            after,
            0 as u_int,
            0 as u_int,
            width_after,
            1 as u_int,
        );
        screen_write_stop(&raw mut ctx);
        format_draw_none(
            octx, available, ocx, ocy, left, centre, right, abs_centre, frs,
        );
        return;
    }
    format_draw_put(
        octx, ocx, ocy, left, frs, 0 as u_int, 0 as u_int, width_left,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        right,
        frs,
        available.wrapping_sub(width_right),
        (*right).cx.wrapping_sub(width_right),
        width_right,
    );
    middle = width_left.wrapping_add(
        available
            .wrapping_sub(width_right)
            .wrapping_sub(width_left)
            .wrapping_div(2 as u_int),
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        centre,
        frs,
        middle
            .wrapping_sub(width_list.wrapping_div(2 as u_int))
            .wrapping_sub(width_centre),
        0 as u_int,
        width_centre,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        after,
        frs,
        middle
            .wrapping_sub(width_list.wrapping_div(2 as u_int))
            .wrapping_add(width_list),
        0 as u_int,
        width_after,
    );
    if focus_start == -(1 as ::core::ffi::c_int) || focus_end == -(1 as ::core::ffi::c_int) {
        focus_end = (*list).cx.wrapping_div(2 as u_int) as ::core::ffi::c_int;
        focus_start = focus_end;
    }
    format_draw_put_list(
        octx,
        ocx,
        ocy,
        middle.wrapping_sub(width_list.wrapping_div(2 as u_int)),
        width_list,
        list,
        list_left,
        list_right,
        focus_start,
        focus_end,
        frs,
    );
    if width_abs_centre > available {
        width_abs_centre = available;
    }
    format_draw_put(
        octx,
        ocx,
        ocy,
        abs_centre,
        frs,
        available
            .wrapping_sub(width_abs_centre)
            .wrapping_div(2 as u_int),
        0 as u_int,
        width_abs_centre,
    );
}
unsafe extern "C" fn format_draw_right(
    mut octx: *mut screen_write_ctx,
    mut available: u_int,
    mut ocx: u_int,
    mut ocy: u_int,
    mut left: *mut screen,
    mut centre: *mut screen,
    mut right: *mut screen,
    mut abs_centre: *mut screen,
    mut list: *mut screen,
    mut list_left: *mut screen,
    mut list_right: *mut screen,
    mut after: *mut screen,
    mut focus_start: ::core::ffi::c_int,
    mut focus_end: ::core::ffi::c_int,
    mut frs: *mut format_ranges,
) {
    let mut width_left: u_int = 0;
    let mut width_centre: u_int = 0;
    let mut width_right: u_int = 0;
    let mut width_list: u_int = 0;
    let mut width_after: u_int = 0;
    let mut width_abs_centre: u_int = 0;
    let mut ctx: screen_write_ctx = screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: ::core::ptr::null_mut::<screen_write_citem>(),
        scrolled: 0,
        bg: 0,
    };
    width_left = (*left).cx;
    width_centre = (*centre).cx;
    width_right = (*right).cx;
    width_abs_centre = (*abs_centre).cx;
    width_list = (*list).cx;
    width_after = (*after).cx;
    while width_left
        .wrapping_add(width_centre)
        .wrapping_add(width_right)
        .wrapping_add(width_list)
        .wrapping_add(width_after)
        > available
    {
        if width_centre > 0 as u_int {
            width_centre = width_centre.wrapping_sub(1);
        } else if width_list > 0 as u_int {
            width_list = width_list.wrapping_sub(1);
        } else if width_right > 0 as u_int {
            width_right = width_right.wrapping_sub(1);
        } else if width_after > 0 as u_int {
            width_after = width_after.wrapping_sub(1);
        } else {
            width_left = width_left.wrapping_sub(1);
        }
    }
    if width_list == 0 as u_int {
        screen_write_start(&raw mut ctx, right);
        screen_write_fast_copy(
            &raw mut ctx,
            after,
            0 as u_int,
            0 as u_int,
            width_after,
            1 as u_int,
        );
        screen_write_stop(&raw mut ctx);
        format_draw_none(
            octx, available, ocx, ocy, left, centre, right, abs_centre, frs,
        );
        return;
    }
    format_draw_put(
        octx, ocx, ocy, left, frs, 0 as u_int, 0 as u_int, width_left,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        after,
        frs,
        available.wrapping_sub(width_after),
        (*after).cx.wrapping_sub(width_after),
        width_after,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        right,
        frs,
        available
            .wrapping_sub(width_right)
            .wrapping_sub(width_list)
            .wrapping_sub(width_after),
        0 as u_int,
        width_right,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        centre,
        frs,
        width_left
            .wrapping_add(
                available
                    .wrapping_sub(width_right)
                    .wrapping_sub(width_list)
                    .wrapping_sub(width_after)
                    .wrapping_sub(width_left)
                    .wrapping_div(2 as u_int),
            )
            .wrapping_sub(width_centre.wrapping_div(2 as u_int)),
        (*centre)
            .cx
            .wrapping_div(2 as u_int)
            .wrapping_sub(width_centre.wrapping_div(2 as u_int)),
        width_centre,
    );
    if focus_start == -(1 as ::core::ffi::c_int) || focus_end == -(1 as ::core::ffi::c_int) {
        focus_end = 0 as ::core::ffi::c_int;
        focus_start = focus_end;
    }
    format_draw_put_list(
        octx,
        ocx,
        ocy,
        available.wrapping_sub(width_list).wrapping_sub(width_after),
        width_list,
        list,
        list_left,
        list_right,
        focus_start,
        focus_end,
        frs,
    );
    if width_abs_centre > available {
        width_abs_centre = available;
    }
    format_draw_put(
        octx,
        ocx,
        ocy,
        abs_centre,
        frs,
        available
            .wrapping_sub(width_abs_centre)
            .wrapping_div(2 as u_int),
        0 as u_int,
        width_abs_centre,
    );
}
unsafe extern "C" fn format_draw_absolute_centre(
    mut octx: *mut screen_write_ctx,
    mut available: u_int,
    mut ocx: u_int,
    mut ocy: u_int,
    mut left: *mut screen,
    mut centre: *mut screen,
    mut right: *mut screen,
    mut abs_centre: *mut screen,
    mut list: *mut screen,
    mut list_left: *mut screen,
    mut list_right: *mut screen,
    mut after: *mut screen,
    mut focus_start: ::core::ffi::c_int,
    mut focus_end: ::core::ffi::c_int,
    mut frs: *mut format_ranges,
) {
    let mut width_left: u_int = 0;
    let mut width_centre: u_int = 0;
    let mut width_right: u_int = 0;
    let mut width_abs_centre: u_int = 0;
    let mut width_list: u_int = 0;
    let mut width_after: u_int = 0;
    let mut middle: u_int = 0;
    let mut abs_centre_offset: u_int = 0;
    width_left = (*left).cx;
    width_centre = (*centre).cx;
    width_right = (*right).cx;
    width_abs_centre = (*abs_centre).cx;
    width_list = (*list).cx;
    width_after = (*after).cx;
    while width_left
        .wrapping_add(width_centre)
        .wrapping_add(width_right)
        > available
    {
        if width_centre > 0 as u_int {
            width_centre = width_centre.wrapping_sub(1);
        } else if width_right > 0 as u_int {
            width_right = width_right.wrapping_sub(1);
        } else {
            width_left = width_left.wrapping_sub(1);
        }
    }
    while width_list
        .wrapping_add(width_after)
        .wrapping_add(width_abs_centre)
        > available
    {
        if width_list > 0 as u_int {
            width_list = width_list.wrapping_sub(1);
        } else if width_after > 0 as u_int {
            width_after = width_after.wrapping_sub(1);
        } else {
            width_abs_centre = width_abs_centre.wrapping_sub(1);
        }
    }
    format_draw_put(
        octx, ocx, ocy, left, frs, 0 as u_int, 0 as u_int, width_left,
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        right,
        frs,
        available.wrapping_sub(width_right),
        (*right).cx.wrapping_sub(width_right),
        width_right,
    );
    middle = width_left.wrapping_add(
        available
            .wrapping_sub(width_right)
            .wrapping_sub(width_left)
            .wrapping_div(2 as u_int),
    );
    format_draw_put(
        octx,
        ocx,
        ocy,
        centre,
        frs,
        middle.wrapping_sub(width_centre),
        0 as u_int,
        width_centre,
    );
    if focus_start == -(1 as ::core::ffi::c_int) || focus_end == -(1 as ::core::ffi::c_int) {
        focus_end = (*list).cx.wrapping_div(2 as u_int) as ::core::ffi::c_int;
        focus_start = focus_end;
    }
    abs_centre_offset = available
        .wrapping_sub(width_list)
        .wrapping_sub(width_abs_centre)
        .wrapping_div(2 as u_int);
    format_draw_put(
        octx,
        ocx,
        ocy,
        abs_centre,
        frs,
        abs_centre_offset,
        0 as u_int,
        width_abs_centre,
    );
    abs_centre_offset = abs_centre_offset.wrapping_add(width_abs_centre);
    format_draw_put_list(
        octx,
        ocx,
        ocy,
        abs_centre_offset,
        width_list,
        list,
        list_left,
        list_right,
        focus_start,
        focus_end,
        frs,
    );
    abs_centre_offset = abs_centre_offset.wrapping_add(width_list);
    format_draw_put(
        octx,
        ocx,
        ocy,
        after,
        frs,
        abs_centre_offset,
        0 as u_int,
        width_after,
    );
}
unsafe extern "C" fn format_leading_hashes(
    mut cp: *const ::core::ffi::c_char,
    mut n: *mut u_int,
    mut width: *mut u_int,
) -> *const ::core::ffi::c_char {
    *n = 0 as u_int;
    while *cp.offset(*n as isize) as ::core::ffi::c_int == '#' as i32 {
        *n = (*n).wrapping_add(1);
    }
    if *n == 0 as u_int {
        *width = 0 as u_int;
        return cp;
    }
    if *cp.offset(*n as isize) as ::core::ffi::c_int != '[' as i32 {
        if (*n).wrapping_rem(2 as u_int) == 0 as u_int {
            *width = (*n).wrapping_div(2 as u_int);
        } else {
            *width = (*n).wrapping_div(2 as u_int).wrapping_add(1 as u_int);
        }
        return cp.offset(*n as isize);
    }
    *width = (*n).wrapping_div(2 as u_int);
    if (*n).wrapping_rem(2 as u_int) == 0 as u_int {
        return cp.offset(*n as isize);
    }
    return cp
        .offset(*n as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
}
unsafe extern "C" fn format_draw_many(
    mut ctx: *mut screen_write_ctx,
    mut sy: *mut style,
    mut ch: ::core::ffi::c_char,
    mut n: u_int,
) {
    let mut i: u_int = 0;
    utf8_set(&raw mut (*sy).gc.data, ch as u_char);
    i = 0 as u_int;
    while i < n {
        screen_write_cell(ctx, &raw mut (*sy).gc);
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn format_draw(
    mut octx: *mut screen_write_ctx,
    mut base: *const grid_cell,
    mut available: u_int,
    mut expanded: *const ::core::ffi::c_char,
    mut srs: *mut style_ranges,
    mut default_colours: ::core::ffi::c_int,
) {
    let mut current_block: u64;
    let mut current: C2RustUnnamed_39 = LEFT;
    let mut last: C2RustUnnamed_39 = LEFT;
    let mut names: [*const ::core::ffi::c_char; 8] = [
        b"LEFT\0" as *const u8 as *const ::core::ffi::c_char,
        b"CENTRE\0" as *const u8 as *const ::core::ffi::c_char,
        b"RIGHT\0" as *const u8 as *const ::core::ffi::c_char,
        b"ABSOLUTE_CENTRE\0" as *const u8 as *const ::core::ffi::c_char,
        b"LIST\0" as *const u8 as *const ::core::ffi::c_char,
        b"LIST_LEFT\0" as *const u8 as *const ::core::ffi::c_char,
        b"LIST_RIGHT\0" as *const u8 as *const ::core::ffi::c_char,
        b"AFTER\0" as *const u8 as *const ::core::ffi::c_char,
    ];
    let mut size: size_t = strlen(expanded);
    let mut os: *mut screen = (*octx).s;
    let mut s: [screen; 8] = [screen {
        title: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        titles: ::core::ptr::null_mut::<screen_titles>(),
        ntitles: 0,
        grid: ::core::ptr::null_mut::<grid>(),
        cx: 0,
        cy: 0,
        cstyle: SCREEN_CURSOR_DEFAULT,
        default_cstyle: SCREEN_CURSOR_DEFAULT,
        ccolour: 0,
        default_ccolour: 0,
        rupper: 0,
        rlower: 0,
        mode: 0,
        default_mode: 0,
        saved_cx: 0,
        saved_cy: 0,
        saved_grid: ::core::ptr::null_mut::<grid>(),
        saved_cell: grid_cell {
            data: utf8_data {
                data: [0; 32],
                have: 0,
                size: 0,
                width: 0,
            },
            attr: 0,
            flags: 0,
            fg: 0,
            bg: 0,
            us: 0,
            link: 0,
        },
        saved_flags: 0,
        tabs: ::core::ptr::null_mut::<bitstr_t>(),
        sel: ::core::ptr::null_mut::<screen_sel>(),
        write_list: ::core::ptr::null_mut::<screen_write_cline>(),
        hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        progress_bar: progress_bar {
            state: PROGRESS_BAR_HIDDEN,
            progress: 0,
        },
    }; 8];
    let mut hl: *mut hyperlinks = (*os).hyperlinks;
    let mut ctx: [screen_write_ctx; 8] = [screen_write_ctx {
        wp: ::core::ptr::null_mut::<window_pane>(),
        s: ::core::ptr::null_mut::<screen>(),
        flags: 0,
        init_ctx_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        item: ::core::ptr::null_mut::<screen_write_citem>(),
        scrolled: 0,
        bg: 0,
    }; 8];
    let mut ocx: u_int = (*os).cx;
    let mut ocy: u_int = (*os).cy;
    let mut n: u_int = 0;
    let mut i: u_int = 0;
    let mut width: [u_int; 8] = [0; 8];
    let mut map: [u_int; 5] = [
        LEFT as ::core::ffi::c_int as u_int,
        LEFT as ::core::ffi::c_int as u_int,
        CENTRE as ::core::ffi::c_int as u_int,
        RIGHT as ::core::ffi::c_int as u_int,
        ABSOLUTE_CENTRE as ::core::ffi::c_int as u_int,
    ];
    let mut focus_start: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut focus_end: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut list_state: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut fill: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut even: ::core::ffi::c_int = 0;
    let mut list_align: style_align = STYLE_ALIGN_DEFAULT;
    let mut gc: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut current_default: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut base_default: grid_cell = grid_cell {
        data: utf8_data {
            data: [0; 32],
            have: 0,
            size: 0,
            width: 0,
        },
        attr: 0,
        flags: 0,
        fg: 0,
        bg: 0,
        us: 0,
        link: 0,
    };
    let mut sy: style = style {
        gc: grid_cell {
            data: utf8_data {
                data: [0; 32],
                have: 0,
                size: 0,
                width: 0,
            },
            attr: 0,
            flags: 0,
            fg: 0,
            bg: 0,
            us: 0,
            link: 0,
        },
        ignore: 0,
        dim: 0,
        fill: 0,
        align: STYLE_ALIGN_DEFAULT,
        list: STYLE_LIST_OFF,
        range_type: STYLE_RANGE_NONE,
        range_argument: 0,
        range_string: [0; 16],
        width: 0,
        width_percentage: 0,
        pad: 0,
        default_type: STYLE_DEFAULT_BASE,
        link: 0,
    };
    let mut saved_sy: style = style {
        gc: grid_cell {
            data: utf8_data {
                data: [0; 32],
                have: 0,
                size: 0,
                width: 0,
            },
            attr: 0,
            flags: 0,
            fg: 0,
            bg: 0,
            us: 0,
            link: 0,
        },
        ignore: 0,
        dim: 0,
        fill: 0,
        align: STYLE_ALIGN_DEFAULT,
        list: STYLE_LIST_OFF,
        range_type: STYLE_RANGE_NONE,
        range_argument: 0,
        range_string: [0; 16],
        width: 0,
        width_percentage: 0,
        pad: 0,
        default_type: STYLE_DEFAULT_BASE,
        link: 0,
    };
    let mut ud: *mut utf8_data = &raw mut sy.gc.data;
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut link_uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut more: utf8_state = UTF8_MORE;
    let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fr: *mut format_range = ::core::ptr::null_mut::<format_range>();
    let mut fr1: *mut format_range = ::core::ptr::null_mut::<format_range>();
    let mut frs: format_ranges = format_ranges {
        tqh_first: ::core::ptr::null_mut::<format_range>(),
        tqh_last: ::core::ptr::null_mut::<*mut format_range>(),
    };
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    memcpy(
        &raw mut base_default as *mut ::core::ffi::c_void,
        base as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    memcpy(
        &raw mut current_default as *mut ::core::ffi::c_void,
        base as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    base = &raw mut base_default;
    style_set(&raw mut sy, &raw mut current_default);
    frs.tqh_first = ::core::ptr::null_mut::<format_range>();
    frs.tqh_last = &raw mut frs.tqh_first;
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
        expanded,
    );
    i = 0 as u_int;
    while i < TOTAL as ::core::ffi::c_int as u_int {
        screen_init(
            (&raw mut s as *mut screen).offset(i as isize) as *mut screen,
            size as u_int,
            1 as u_int,
            0 as u_int,
        );
        screen_write_start(
            (&raw mut ctx as *mut screen_write_ctx).offset(i as isize) as *mut screen_write_ctx,
            (&raw mut s as *mut screen).offset(i as isize) as *mut screen,
        );
        screen_write_clearendofline(
            (&raw mut ctx as *mut screen_write_ctx).offset(i as isize) as *mut screen_write_ctx,
            current_default.bg as u_int,
        );
        width[i as usize] = 0 as u_int;
        i = i.wrapping_add(1);
    }
    cp = expanded;
    loop {
        if !(*cp as ::core::ffi::c_int != '\0' as i32) {
            current_block = 1830138855519935310;
            break;
        }
        if *cp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '#' as i32
            && *cp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '[' as i32
            && *cp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
        {
            n = 1 as u_int;
            while *cp.offset(n as isize) as ::core::ffi::c_int == '#' as i32 {
                n = n.wrapping_add(1);
            }
            even = (n.wrapping_rem(2 as u_int) == 0 as u_int) as ::core::ffi::c_int;
            if *cp.offset(n as isize) as ::core::ffi::c_int != '[' as i32 {
                cp = cp.offset(n as isize);
                if even != 0 {
                    n = n.wrapping_div(2 as u_int);
                } else {
                    n = n.wrapping_div(2 as u_int).wrapping_add(1 as u_int);
                }
                width[current as usize] = width[current as usize].wrapping_add(n);
                format_draw_many(
                    (&raw mut ctx as *mut screen_write_ctx).offset(current as isize)
                        as *mut screen_write_ctx,
                    &raw mut sy,
                    '#' as i32 as ::core::ffi::c_char,
                    n,
                );
            } else {
                if even != 0 {
                    cp = cp.offset(n.wrapping_add(1 as u_int) as isize);
                } else {
                    cp = cp.offset(n.wrapping_sub(1 as u_int) as isize);
                }
                if sy.ignore != 0 {
                    continue;
                }
                format_draw_many(
                    (&raw mut ctx as *mut screen_write_ctx).offset(current as isize)
                        as *mut screen_write_ctx,
                    &raw mut sy,
                    '#' as i32 as ::core::ffi::c_char,
                    n.wrapping_div(2 as u_int),
                );
                width[current as usize] =
                    width[current as usize].wrapping_add(n.wrapping_div(2 as u_int));
                if even != 0 {
                    utf8_set(ud, '[' as i32 as u_char);
                    screen_write_cell(
                        (&raw mut ctx as *mut screen_write_ctx).offset(current as isize)
                            as *mut screen_write_ctx,
                        &raw mut sy.gc,
                    );
                    width[current as usize] = width[current as usize].wrapping_add(1);
                }
            }
        } else if *cp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '#' as i32
            || *cp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '[' as i32
            || sy.ignore != 0
        {
            more = utf8_open(ud, *cp as u_char);
            if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                loop {
                    cp = cp.offset(1);
                    if !(*cp as ::core::ffi::c_int != '\0' as i32
                        && more as ::core::ffi::c_uint
                            == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        break;
                    }
                    more = utf8_append(ud, *cp as u_char);
                }
                if more as ::core::ffi::c_uint
                    != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    cp = cp.offset(-((*ud).have as ::core::ffi::c_int as isize));
                }
            }
            if more as ::core::ffi::c_uint != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (*cp as ::core::ffi::c_int) < 0x20 as ::core::ffi::c_int
                    || *cp as ::core::ffi::c_int > 0x7e as ::core::ffi::c_int
                {
                    cp = cp.offset(1);
                    continue;
                } else {
                    utf8_set(ud, *cp as u_char);
                    cp = cp.offset(1);
                }
            }
            screen_write_cell(
                (&raw mut ctx as *mut screen_write_ctx).offset(current as isize)
                    as *mut screen_write_ctx,
                &raw mut sy.gc,
            );
            width[current as usize] = width[current as usize].wrapping_add((*ud).width as u_int);
        } else {
            end = format_skip(
                cp.offset(2 as ::core::ffi::c_int as isize),
                b"]\0" as *const u8 as *const ::core::ffi::c_char,
            );
            if end.is_null() {
                log_debug(
                    b"%s: no terminating ] at '%s'\0" as *const u8 as *const ::core::ffi::c_char,
                    b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                    cp.offset(2 as ::core::ffi::c_int as isize),
                );
                fr = frs.tqh_first;
                while !fr.is_null() && {
                    fr1 = (*fr).entry.tqe_next;
                    1 as ::core::ffi::c_int != 0
                } {
                    format_free_range(&raw mut frs, fr);
                    fr = fr1;
                }
                i = 0 as u_int;
                while i < TOTAL as ::core::ffi::c_int as u_int {
                    screen_write_stop((&raw mut ctx as *mut screen_write_ctx).offset(i as isize)
                        as *mut screen_write_ctx);
                    i = i.wrapping_add(1);
                }
                current_block = 4329038292887906754;
                break;
            } else {
                tmp = xstrndup(
                    cp.offset(2 as ::core::ffi::c_int as isize),
                    end.offset_from(cp.offset(2 as ::core::ffi::c_int as isize))
                        as ::core::ffi::c_long as size_t,
                );
                style_copy(&raw mut saved_sy, &raw mut sy);
                if style_parse(&raw mut sy, &raw mut current_default, tmp)
                    != 0 as ::core::ffi::c_int
                {
                    log_debug(
                        b"%s: invalid style '%s'\0" as *const u8 as *const ::core::ffi::c_char,
                        b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                        tmp,
                    );
                    free(tmp as *mut ::core::ffi::c_void);
                    cp = end.offset(1 as ::core::ffi::c_int as isize);
                } else {
                    log_debug(
                        b"%s: style '%s' -> '%s'\0" as *const u8 as *const ::core::ffi::c_char,
                        b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                        tmp,
                        style_tostring(&raw mut sy),
                    );
                    free(tmp as *mut ::core::ffi::c_void);
                    if default_colours != 0 {
                        sy.gc.bg = (*base).bg;
                        sy.gc.fg = (*base).fg;
                    }
                    link_uri = style_link(&raw mut sy);
                    if !link_uri.is_null() && !hl.is_null() {
                        sy.gc.link = hyperlinks_put(hl, link_uri, link_uri);
                    } else {
                        sy.gc.link = 0 as u_int;
                    }
                    if sy.fill != 8 as ::core::ffi::c_int {
                        fill = sy.fill;
                    }
                    if sy.default_type as ::core::ffi::c_uint
                        == STYLE_DEFAULT_PUSH as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        memcpy(
                            &raw mut current_default as *mut ::core::ffi::c_void,
                            &raw mut saved_sy.gc as *const ::core::ffi::c_void,
                            ::core::mem::size_of::<grid_cell>() as size_t,
                        );
                        sy.default_type = STYLE_DEFAULT_BASE;
                    } else if sy.default_type as ::core::ffi::c_uint
                        == STYLE_DEFAULT_POP as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        memcpy(
                            &raw mut current_default as *mut ::core::ffi::c_void,
                            base as *const ::core::ffi::c_void,
                            ::core::mem::size_of::<grid_cell>() as size_t,
                        );
                        sy.default_type = STYLE_DEFAULT_BASE;
                    } else if sy.default_type as ::core::ffi::c_uint
                        == STYLE_DEFAULT_SET as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        memcpy(
                            &raw mut base_default as *mut ::core::ffi::c_void,
                            &raw mut saved_sy.gc as *const ::core::ffi::c_void,
                            ::core::mem::size_of::<grid_cell>() as size_t,
                        );
                        memcpy(
                            &raw mut current_default as *mut ::core::ffi::c_void,
                            &raw mut saved_sy.gc as *const ::core::ffi::c_void,
                            ::core::mem::size_of::<grid_cell>() as size_t,
                        );
                        sy.default_type = STYLE_DEFAULT_BASE;
                    }
                    match sy.list as ::core::ffi::c_uint {
                        1 => {
                            if list_state != 0 as ::core::ffi::c_int {
                                if !fr.is_null() {
                                    free(fr as *mut ::core::ffi::c_void);
                                    fr = ::core::ptr::null_mut::<format_range>();
                                }
                                list_state = 0 as ::core::ffi::c_int;
                                list_align = sy.align;
                            }
                            if focus_start != -(1 as ::core::ffi::c_int)
                                && focus_end == -(1 as ::core::ffi::c_int)
                            {
                                focus_end =
                                    s[LIST as ::core::ffi::c_int as usize].cx as ::core::ffi::c_int;
                            }
                            current = LIST;
                        }
                        2 => {
                            if !(list_state != 0 as ::core::ffi::c_int) {
                                if focus_start == -(1 as ::core::ffi::c_int) {
                                    focus_start = s[LIST as ::core::ffi::c_int as usize].cx
                                        as ::core::ffi::c_int;
                                }
                            }
                        }
                        0 => {
                            if list_state == 0 as ::core::ffi::c_int {
                                if !fr.is_null() {
                                    free(fr as *mut ::core::ffi::c_void);
                                    fr = ::core::ptr::null_mut::<format_range>();
                                }
                                if focus_start != -(1 as ::core::ffi::c_int)
                                    && focus_end == -(1 as ::core::ffi::c_int)
                                {
                                    focus_end = s[LIST as ::core::ffi::c_int as usize].cx
                                        as ::core::ffi::c_int;
                                }
                                map[list_align as usize] = AFTER as ::core::ffi::c_int as u_int;
                                if list_align as ::core::ffi::c_uint
                                    == STYLE_ALIGN_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    map[STYLE_ALIGN_DEFAULT as ::core::ffi::c_int as usize] =
                                        AFTER as ::core::ffi::c_int as u_int;
                                }
                                list_state = 1 as ::core::ffi::c_int;
                            }
                            current = map[sy.align as usize] as C2RustUnnamed_39;
                        }
                        3 => {
                            if !(list_state != 0 as ::core::ffi::c_int) {
                                if !(s[LIST_LEFT as ::core::ffi::c_int as usize].cx != 0 as u_int) {
                                    if !fr.is_null() {
                                        free(fr as *mut ::core::ffi::c_void);
                                        fr = ::core::ptr::null_mut::<format_range>();
                                    }
                                    if focus_start != -(1 as ::core::ffi::c_int)
                                        && focus_end == -(1 as ::core::ffi::c_int)
                                    {
                                        focus_end = -(1 as ::core::ffi::c_int);
                                        focus_start = focus_end;
                                    }
                                    current = LIST_LEFT;
                                }
                            }
                        }
                        4 => {
                            if !(list_state != 0 as ::core::ffi::c_int) {
                                if !(s[LIST_RIGHT as ::core::ffi::c_int as usize].cx != 0 as u_int)
                                {
                                    if !fr.is_null() {
                                        free(fr as *mut ::core::ffi::c_void);
                                        fr = ::core::ptr::null_mut::<format_range>();
                                    }
                                    if focus_start != -(1 as ::core::ffi::c_int)
                                        && focus_end == -(1 as ::core::ffi::c_int)
                                    {
                                        focus_end = -(1 as ::core::ffi::c_int);
                                        focus_start = focus_end;
                                    }
                                    current = LIST_RIGHT;
                                }
                            }
                        }
                        _ => {}
                    }
                    if current as ::core::ffi::c_uint != last as ::core::ffi::c_uint {
                        log_debug(
                            b"%s: change %s -> %s\0" as *const u8 as *const ::core::ffi::c_char,
                            b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                            names[last as usize],
                            names[current as usize],
                        );
                        last = current;
                    }
                    if !srs.is_null() {
                        if !fr.is_null() && format_is_type(fr, &raw mut sy) == 0 {
                            if s[current as usize].cx != (*fr).start {
                                (*fr).end = s[current as usize].cx;
                                (*fr).entry.tqe_next = ::core::ptr::null_mut::<format_range>();
                                (*fr).entry.tqe_prev = frs.tqh_last;
                                *frs.tqh_last = fr;
                                frs.tqh_last = &raw mut (*fr).entry.tqe_next;
                            } else {
                                free(fr as *mut ::core::ffi::c_void);
                            }
                            fr = ::core::ptr::null_mut::<format_range>();
                        }
                        if fr.is_null()
                            && sy.range_type as ::core::ffi::c_uint
                                != STYLE_RANGE_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            fr = xcalloc(
                                1 as size_t,
                                ::core::mem::size_of::<format_range>() as size_t,
                            ) as *mut format_range;
                            (*fr).index = current as u_int;
                            (*fr).s =
                                (&raw mut s as *mut screen).offset(current as isize) as *mut screen;
                            (*fr).start = s[current as usize].cx;
                            (*fr).type_0 = sy.range_type;
                            (*fr).argument = sy.range_argument;
                            strlcpy(
                                &raw mut (*fr).string as *mut ::core::ffi::c_char,
                                &raw mut sy.range_string as *mut ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
                            );
                        }
                    }
                    cp = end.offset(1 as ::core::ffi::c_int as isize);
                }
            }
        }
    }
    match current_block {
        1830138855519935310 => {
            free(fr as *mut ::core::ffi::c_void);
            i = 0 as u_int;
            while i < TOTAL as ::core::ffi::c_int as u_int {
                screen_write_stop((&raw mut ctx as *mut screen_write_ctx).offset(i as isize)
                    as *mut screen_write_ctx);
                log_debug(
                    b"%s: width %s is %u\0" as *const u8 as *const ::core::ffi::c_char,
                    b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                    names[i as usize],
                    width[i as usize],
                );
                i = i.wrapping_add(1);
            }
            if focus_start != -(1 as ::core::ffi::c_int) && focus_end != -(1 as ::core::ffi::c_int)
            {
                log_debug(
                    b"%s: focus %d-%d\0" as *const u8 as *const ::core::ffi::c_char,
                    b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                    focus_start,
                    focus_end,
                );
            }
            fr = frs.tqh_first;
            while !fr.is_null() {
                log_debug(
                    b"%s: range %d|%u is %s %u-%u\0" as *const u8 as *const ::core::ffi::c_char,
                    b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                    (*fr).type_0 as ::core::ffi::c_uint,
                    (*fr).argument,
                    names[(*fr).index as usize],
                    (*fr).start,
                    (*fr).end,
                );
                fr = (*fr).entry.tqe_next;
            }
            if fill != -(1 as ::core::ffi::c_int) {
                memcpy(
                    &raw mut gc as *mut ::core::ffi::c_void,
                    &raw const grid_default_cell as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<grid_cell>() as size_t,
                );
                gc.bg = fill;
                i = 0 as u_int;
                while i < available {
                    screen_write_putc(octx, &raw mut gc, ' ' as i32 as u_char);
                    i = i.wrapping_add(1);
                }
            }
            match list_align as ::core::ffi::c_uint {
                0 => {
                    format_draw_none(
                        octx,
                        available,
                        ocx,
                        ocy,
                        (&raw mut s as *mut screen).offset(LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(ABSOLUTE_CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        &raw mut frs,
                    );
                }
                1 => {
                    format_draw_left(
                        octx,
                        available,
                        ocx,
                        ocy,
                        (&raw mut s as *mut screen).offset(LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(ABSOLUTE_CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(LIST as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(LIST_LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(LIST_RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(AFTER as ::core::ffi::c_int as isize)
                            as *mut screen,
                        focus_start,
                        focus_end,
                        &raw mut frs,
                    );
                }
                2 => {
                    format_draw_centre(
                        octx,
                        available,
                        ocx,
                        ocy,
                        (&raw mut s as *mut screen).offset(LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(ABSOLUTE_CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(LIST as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(LIST_LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(LIST_RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(AFTER as ::core::ffi::c_int as isize)
                            as *mut screen,
                        focus_start,
                        focus_end,
                        &raw mut frs,
                    );
                }
                3 => {
                    format_draw_right(
                        octx,
                        available,
                        ocx,
                        ocy,
                        (&raw mut s as *mut screen).offset(LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(ABSOLUTE_CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(LIST as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(LIST_LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(LIST_RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(AFTER as ::core::ffi::c_int as isize)
                            as *mut screen,
                        focus_start,
                        focus_end,
                        &raw mut frs,
                    );
                }
                4 => {
                    format_draw_absolute_centre(
                        octx,
                        available,
                        ocx,
                        ocy,
                        (&raw mut s as *mut screen).offset(LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(ABSOLUTE_CENTRE as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(LIST as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(LIST_LEFT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen)
                            .offset(LIST_RIGHT as ::core::ffi::c_int as isize)
                            as *mut screen,
                        (&raw mut s as *mut screen).offset(AFTER as ::core::ffi::c_int as isize)
                            as *mut screen,
                        focus_start,
                        focus_end,
                        &raw mut frs,
                    );
                }
                _ => {}
            }
            fr = frs.tqh_first;
            while !fr.is_null() && {
                fr1 = (*fr).entry.tqe_next;
                1 as ::core::ffi::c_int != 0
            } {
                sr = xcalloc(1 as size_t, ::core::mem::size_of::<style_range>() as size_t)
                    as *mut style_range;
                (*sr).type_0 = (*fr).type_0;
                (*sr).argument = (*fr).argument;
                strlcpy(
                    &raw mut (*sr).string as *mut ::core::ffi::c_char,
                    &raw mut (*fr).string as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
                );
                (*sr).start = (*fr).start;
                (*sr).end = (*fr).end;
                (*sr).entry.tqe_next = ::core::ptr::null_mut::<style_range>();
                (*sr).entry.tqe_prev = (*srs).tqh_last;
                *(*srs).tqh_last = sr;
                (*srs).tqh_last = &raw mut (*sr).entry.tqe_next;
                match (*sr).type_0 as ::core::ffi::c_uint {
                    1 => {
                        log_debug(
                            b"%s: range left at %u-%u\0" as *const u8 as *const ::core::ffi::c_char,
                            b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                            (*sr).start,
                            (*sr).end,
                        );
                    }
                    2 => {
                        log_debug(
                            b"%s: range right at %u-%u\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                            (*sr).start,
                            (*sr).end,
                        );
                    }
                    3 => {
                        log_debug(
                            b"%s: range pane|%%%u at %u-%u\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                            (*sr).argument,
                            (*sr).start,
                            (*sr).end,
                        );
                    }
                    4 => {
                        log_debug(
                            b"%s: range window|%u at %u-%u\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                            (*sr).argument,
                            (*sr).start,
                            (*sr).end,
                        );
                    }
                    5 => {
                        log_debug(
                            b"%s: range session|$%u at %u-%u\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                            (*sr).argument,
                            (*sr).start,
                            (*sr).end,
                        );
                    }
                    6 => {
                        log_debug(
                            b"%s: range user|%u at %u-%u\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                            (*sr).argument,
                            (*sr).start,
                            (*sr).end,
                        );
                    }
                    7 => {
                        log_debug(
                            b"%s: range control|%u at %u-%u\0" as *const u8
                                as *const ::core::ffi::c_char,
                            b"format_draw\0" as *const u8 as *const ::core::ffi::c_char,
                            (*sr).argument,
                            (*sr).start,
                            (*sr).end,
                        );
                    }
                    0 | _ => {}
                }
                format_free_range(&raw mut frs, fr);
                fr = fr1;
            }
        }
        _ => {}
    }
    i = 0 as u_int;
    while i < TOTAL as ::core::ffi::c_int as u_int {
        screen_free((&raw mut s as *mut screen).offset(i as isize) as *mut screen);
        i = i.wrapping_add(1);
    }
    screen_write_cursormove(
        octx,
        ocx as ::core::ffi::c_int,
        ocy as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn format_width(mut expanded: *const ::core::ffi::c_char) -> u_int {
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: u_int = 0;
    let mut leading_width: u_int = 0;
    let mut width: u_int = 0 as u_int;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut more: utf8_state = UTF8_MORE;
    cp = expanded;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if *cp as ::core::ffi::c_int == '#' as i32 {
            end = format_leading_hashes(cp, &raw mut n, &raw mut leading_width);
            width = width.wrapping_add(leading_width);
            cp = end;
            if *cp as ::core::ffi::c_int == '#' as i32 {
                end = format_skip(
                    cp.offset(2 as ::core::ffi::c_int as isize),
                    b"]\0" as *const u8 as *const ::core::ffi::c_char,
                );
                if end.is_null() {
                    return 0 as u_int;
                }
                cp = end.offset(1 as ::core::ffi::c_int as isize);
            }
        } else {
            more = utf8_open(&raw mut ud, *cp as u_char);
            if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                loop {
                    cp = cp.offset(1);
                    if !(*cp as ::core::ffi::c_int != '\0' as i32
                        && more as ::core::ffi::c_uint
                            == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        break;
                    }
                    more = utf8_append(&raw mut ud, *cp as u_char);
                }
                if more as ::core::ffi::c_uint
                    == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    width = width.wrapping_add(ud.width as u_int);
                }
            } else if *cp as ::core::ffi::c_int > 0x1f as ::core::ffi::c_int
                && (*cp as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
            {
                width = width.wrapping_add(1);
                cp = cp.offset(1);
            } else {
                cp = cp.offset(1);
            }
        }
    }
    return width;
}
#[no_mangle]
pub unsafe extern "C" fn format_trim_left(
    mut expanded: *const ::core::ffi::c_char,
    mut limit: u_int,
) -> *mut ::core::ffi::c_char {
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *const ::core::ffi::c_char = expanded;
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: u_int = 0;
    let mut width: u_int = 0 as u_int;
    let mut leading_width: u_int = 0;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut more: utf8_state = UTF8_MORE;
    copy = xcalloc(2 as size_t, strlen(expanded).wrapping_add(1 as size_t))
        as *mut ::core::ffi::c_char;
    out = copy;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if width >= limit {
            break;
        }
        if *cp as ::core::ffi::c_int == '#' as i32 {
            end = format_leading_hashes(cp, &raw mut n, &raw mut leading_width);
            if leading_width > limit.wrapping_sub(width) {
                leading_width = limit.wrapping_sub(width);
            }
            if leading_width != 0 as u_int {
                if n == 1 as u_int {
                    let fresh0 = out;
                    out = out.offset(1);
                    *fresh0 = '#' as i32 as ::core::ffi::c_char;
                } else {
                    memset(
                        out as *mut ::core::ffi::c_void,
                        '#' as i32,
                        (2 as u_int).wrapping_mul(leading_width) as size_t,
                    );
                    out = out.offset((2 as u_int).wrapping_mul(leading_width) as isize);
                }
                width = width.wrapping_add(leading_width);
            }
            cp = end;
            if !(*cp as ::core::ffi::c_int == '#' as i32) {
                continue;
            }
            end = format_skip(
                cp.offset(2 as ::core::ffi::c_int as isize),
                b"]\0" as *const u8 as *const ::core::ffi::c_char,
            );
            if end.is_null() {
                break;
            }
            memcpy(
                out as *mut ::core::ffi::c_void,
                cp as *const ::core::ffi::c_void,
                end.offset(1 as ::core::ffi::c_int as isize).offset_from(cp) as ::core::ffi::c_long
                    as size_t,
            );
            out = out.offset(end.offset(1 as ::core::ffi::c_int as isize).offset_from(cp)
                as ::core::ffi::c_long as isize);
            cp = end.offset(1 as ::core::ffi::c_int as isize);
        } else {
            more = utf8_open(&raw mut ud, *cp as u_char);
            if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                loop {
                    cp = cp.offset(1);
                    if !(*cp as ::core::ffi::c_int != '\0' as i32
                        && more as ::core::ffi::c_uint
                            == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        break;
                    }
                    more = utf8_append(&raw mut ud, *cp as u_char);
                }
                if more as ::core::ffi::c_uint
                    == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if width.wrapping_add(ud.width as u_int) <= limit {
                        memcpy(
                            out as *mut ::core::ffi::c_void,
                            &raw mut ud.data as *mut u_char as *const ::core::ffi::c_void,
                            ud.size as size_t,
                        );
                        out = out.offset(ud.size as ::core::ffi::c_int as isize);
                    }
                    width = width.wrapping_add(ud.width as u_int);
                } else {
                    cp = cp.offset(-(ud.have as ::core::ffi::c_int as isize));
                    cp = cp.offset(1);
                }
            } else if *cp as ::core::ffi::c_int > 0x1f as ::core::ffi::c_int
                && (*cp as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
            {
                if width.wrapping_add(1 as u_int) <= limit {
                    let fresh1 = out;
                    out = out.offset(1);
                    *fresh1 = *cp;
                }
                width = width.wrapping_add(1);
                cp = cp.offset(1);
            } else {
                cp = cp.offset(1);
            }
        }
    }
    *out = '\0' as i32 as ::core::ffi::c_char;
    return copy;
}
#[no_mangle]
pub unsafe extern "C" fn format_trim_right(
    mut expanded: *const ::core::ffi::c_char,
    mut limit: u_int,
) -> *mut ::core::ffi::c_char {
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *const ::core::ffi::c_char = expanded;
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut width: u_int = 0 as u_int;
    let mut total_width: u_int = 0;
    let mut skip: u_int = 0;
    let mut n: u_int = 0;
    let mut leading_width: u_int = 0;
    let mut copy_width: u_int = 0;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut more: utf8_state = UTF8_MORE;
    total_width = format_width(expanded);
    if total_width <= limit {
        return xstrdup(expanded);
    }
    skip = total_width.wrapping_sub(limit);
    copy = xcalloc(2 as size_t, strlen(expanded).wrapping_add(1 as size_t))
        as *mut ::core::ffi::c_char;
    out = copy;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if *cp as ::core::ffi::c_int == '#' as i32 {
            end = format_leading_hashes(cp, &raw mut n, &raw mut leading_width);
            copy_width = leading_width;
            if width <= skip {
                if skip.wrapping_sub(width) >= copy_width {
                    copy_width = 0 as u_int;
                } else {
                    copy_width = copy_width.wrapping_sub(skip.wrapping_sub(width));
                }
            }
            if copy_width != 0 as u_int {
                if n == 1 as u_int {
                    let fresh2 = out;
                    out = out.offset(1);
                    *fresh2 = '#' as i32 as ::core::ffi::c_char;
                } else {
                    memset(
                        out as *mut ::core::ffi::c_void,
                        '#' as i32,
                        (2 as u_int).wrapping_mul(copy_width) as size_t,
                    );
                    out = out.offset((2 as u_int).wrapping_mul(copy_width) as isize);
                }
            }
            width = width.wrapping_add(leading_width);
            cp = end;
            if !(*cp as ::core::ffi::c_int == '#' as i32) {
                continue;
            }
            end = format_skip(
                cp.offset(2 as ::core::ffi::c_int as isize),
                b"]\0" as *const u8 as *const ::core::ffi::c_char,
            );
            if end.is_null() {
                break;
            }
            memcpy(
                out as *mut ::core::ffi::c_void,
                cp as *const ::core::ffi::c_void,
                end.offset(1 as ::core::ffi::c_int as isize).offset_from(cp) as ::core::ffi::c_long
                    as size_t,
            );
            out = out.offset(end.offset(1 as ::core::ffi::c_int as isize).offset_from(cp)
                as ::core::ffi::c_long as isize);
            cp = end.offset(1 as ::core::ffi::c_int as isize);
        } else {
            more = utf8_open(&raw mut ud, *cp as u_char);
            if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                loop {
                    cp = cp.offset(1);
                    if !(*cp as ::core::ffi::c_int != '\0' as i32
                        && more as ::core::ffi::c_uint
                            == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        break;
                    }
                    more = utf8_append(&raw mut ud, *cp as u_char);
                }
                if more as ::core::ffi::c_uint
                    == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if width >= skip {
                        memcpy(
                            out as *mut ::core::ffi::c_void,
                            &raw mut ud.data as *mut u_char as *const ::core::ffi::c_void,
                            ud.size as size_t,
                        );
                        out = out.offset(ud.size as ::core::ffi::c_int as isize);
                    }
                    width = width.wrapping_add(ud.width as u_int);
                } else {
                    cp = cp.offset(-(ud.have as ::core::ffi::c_int as isize));
                    cp = cp.offset(1);
                }
            } else if *cp as ::core::ffi::c_int > 0x1f as ::core::ffi::c_int
                && (*cp as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
            {
                if width >= skip {
                    let fresh3 = out;
                    out = out.offset(1);
                    *fresh3 = *cp;
                }
                width = width.wrapping_add(1);
                cp = cp.offset(1);
            } else {
                cp = cp.offset(1);
            }
        }
    }
    *out = '\0' as i32 as ::core::ffi::c_char;
    return copy;
}

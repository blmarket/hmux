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
    pub type hyperlinks;
    pub type screen_write_cline;
    pub type screen_sel;
    pub type screen_titles;
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
    fn evbuffer_new() -> *mut evbuffer;
    fn evbuffer_free(buf: *mut evbuffer);
    fn evbuffer_get_length(buf: *const evbuffer) -> size_t;
    fn evbuffer_add_printf(
        buf: *mut evbuffer,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn evbuffer_pullup(buf: *mut evbuffer, size: ssize_t) -> *mut ::core::ffi::c_uchar;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xmemdup(_: *const ::core::ffi::c_void, _: size_t) -> *mut ::core::ffi::c_char;
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
    fn format_add(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn cmd_find_clear_state(_: *mut cmd_find_state, _: ::core::ffi::c_int);
    fn cmd_find_valid_state(_: *mut cmd_find_state) -> ::core::ffi::c_int;
    fn cmd_find_from_session(_: *mut cmd_find_state, _: *mut session, _: ::core::ffi::c_int);
    fn cmd_find_from_winlink(_: *mut cmd_find_state, _: *mut winlink, _: ::core::ffi::c_int);
    fn cmd_find_from_session_window(
        _: *mut cmd_find_state,
        _: *mut session,
        _: *mut window,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_find_from_winlink_pane(
        _: *mut cmd_find_state,
        _: *mut winlink,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    );
    fn cmd_find_from_pane(
        _: *mut cmd_find_state,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_find_from_nothing(_: *mut cmd_find_state, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn server_client_unref(_: *mut client);
    fn winlink_find_by_index(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn window_has_pane(_: *mut window, _: *mut window_pane) -> ::core::ffi::c_int;
    fn window_add_ref(_: *mut window, _: *const ::core::ffi::c_char);
    fn window_remove_ref(_: *mut window, _: *const ::core::ffi::c_char);
    fn window_pane_add_ref(_: *mut window_pane, _: *const ::core::ffi::c_char);
    fn window_pane_remove_ref(_: *mut window_pane, _: *const ::core::ffi::c_char);
    fn session_alive(_: *mut session) -> ::core::ffi::c_int;
    fn session_add_ref(_: *mut session, _: *const ::core::ffi::c_char);
    fn session_remove_ref(_: *mut session, _: *const ::core::ffi::c_char);
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
}
pub type __builtin_va_list = [__va_list_tag; 1];
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __va_list_tag {
    pub gp_offset: ::core::ffi::c_uint,
    pub fp_offset: ::core::ffi::c_uint,
    pub overflow_arg_area: *mut ::core::ffi::c_void,
    pub reg_save_area: *mut ::core::ffi::c_void,
}
pub type ssize_t = isize;
pub type va_list = __builtin_va_list;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mouse_event {
    pub valid: ::core::ffi::c_int,
    pub ignore: ::core::ffi::c_int,
    pub key: key_code,
    pub statusat: ::core::ffi::c_int,
    pub statuslines: u_int,
    pub x: u_int,
    pub y: u_int,
    pub b: u_int,
    pub lx: u_int,
    pub ly: u_int,
    pub lb: u_int,
    pub ox: u_int,
    pub oy: u_int,
    pub s: ::core::ffi::c_int,
    pub w: ::core::ffi::c_int,
    pub wp: ::core::ffi::c_int,
    pub sgr_type: u_int,
    pub sgr_b: u_int,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct screen {
    pub title: *mut ::core::ffi::c_char,
    pub path: *mut ::core::ffi::c_char,
    pub titles: *mut screen_titles,
    pub ntitles: u_int,
    pub grid: *mut grid,
    pub cx: u_int,
    pub cy: u_int,
    pub cstyle: screen_cursor_style,
    pub default_cstyle: screen_cursor_style,
    pub ccolour: ::core::ffi::c_int,
    pub default_ccolour: ::core::ffi::c_int,
    pub rupper: u_int,
    pub rlower: u_int,
    pub mode: ::core::ffi::c_int,
    pub default_mode: ::core::ffi::c_int,
    pub saved_cx: u_int,
    pub saved_cy: u_int,
    pub saved_grid: *mut grid,
    pub saved_cell: grid_cell,
    pub saved_flags: ::core::ffi::c_int,
    pub tabs: *mut bitstr_t,
    pub sel: *mut screen_sel,
    pub write_list: *mut screen_write_cline,
    pub hyperlinks: *mut hyperlinks,
    pub progress_bar: progress_bar,
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
pub struct visible_ranges {
    pub ranges: *mut visible_range,
    pub used: u_int,
    pub size: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct visible_range {
    pub px: u_int,
    pub nx: u_int,
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
pub struct window_pane_offset {
    pub used: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_pane_resizes {
    pub tqh_first: *mut window_pane_resize,
    pub tqh_last: *mut *mut window_pane_resize,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_pane_resize {
    pub sx: u_int,
    pub sy: u_int,
    pub osx: u_int,
    pub osy: u_int,
    pub entry: C2RustUnnamed_30,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_30 {
    pub tqe_next: *mut window_pane_resize,
    pub tqe_prev: *mut *mut window_pane_resize,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct layout_geometry {
    pub sx: u_int,
    pub sy: u_int,
    pub xoff: ::core::ffi::c_int,
    pub yoff: ::core::ffi::c_int,
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
pub struct event_payload {
    pub items: event_payload_tree,
    pub target: cmd_find_state,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_payload_tree {
    pub rbh_root: *mut event_payload_item,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_payload_item {
    pub name: *mut ::core::ffi::c_char,
    pub type_0: event_payload_type,
    pub c2rust_unnamed: C2RustUnnamed_36,
    pub entry: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub rbe_left: *mut event_payload_item,
    pub rbe_right: *mut event_payload_item,
    pub rbe_parent: *mut event_payload_item,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_36 {
    pub string: *mut ::core::ffi::c_char,
    pub time: time_t,
    pub number: ::core::ffi::c_int,
    pub unsigned_number: u_int,
    pub client: *mut client,
    pub session: *mut session,
    pub window: *mut window,
    pub pane: *mut window_pane,
    pub pointer: C2RustUnnamed_37,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
    pub ptr: *mut ::core::ffi::c_void,
    pub free_cb: event_payload_free_cb,
    pub print_cb: event_payload_print_cb,
}
pub type event_payload_print_cb =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut evbuffer) -> ()>;
pub type event_payload_free_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type event_payload_type = ::core::ffi::c_uint;
pub const EVENT_PAYLOAD_POINTER: event_payload_type = 8;
pub const EVENT_PAYLOAD_PANE: event_payload_type = 7;
pub const EVENT_PAYLOAD_WINDOW: event_payload_type = 6;
pub const EVENT_PAYLOAD_SESSION: event_payload_type = 5;
pub const EVENT_PAYLOAD_CLIENT: event_payload_type = 4;
pub const EVENT_PAYLOAD_UINT: event_payload_type = 3;
pub const EVENT_PAYLOAD_INT: event_payload_type = 2;
pub const EVENT_PAYLOAD_TIME: event_payload_type = 1;
pub const EVENT_PAYLOAD_STRING: event_payload_type = 0;
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
unsafe extern "C" fn event_payload_cmp(
    mut epi1: *mut event_payload_item,
    mut epi2: *mut event_payload_item,
) -> ::core::ffi::c_int {
    return strcmp((*epi1).name, (*epi2).name);
}
unsafe extern "C" fn event_payload_tree_RB_NEXT(
    mut elm: *mut event_payload_item,
) -> *mut event_payload_item {
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
unsafe extern "C" fn event_payload_tree_RB_REMOVE(
    mut head: *mut event_payload_tree,
    mut elm: *mut event_payload_item,
) -> *mut event_payload_item {
    let mut current_block: u64;
    let mut child: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut parent: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut old: *mut event_payload_item = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
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
        current_block = 4623090504028811011;
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
        event_payload_tree_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn event_payload_tree_RB_REMOVE_COLOR(
    mut head: *mut event_payload_tree,
    mut parent: *mut event_payload_item,
    mut elm: *mut event_payload_item,
) {
    let mut tmp: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
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
                    let mut oleft: *mut event_payload_item =
                        ::core::ptr::null_mut::<event_payload_item>();
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
                    let mut oright: *mut event_payload_item =
                        ::core::ptr::null_mut::<event_payload_item>();
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
unsafe extern "C" fn event_payload_tree_RB_MINMAX(
    mut head: *mut event_payload_tree,
    mut val: ::core::ffi::c_int,
) -> *mut event_payload_item {
    let mut tmp: *mut event_payload_item = (*head).rbh_root;
    let mut parent: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
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
unsafe extern "C" fn event_payload_tree_RB_INSERT_COLOR(
    mut head: *mut event_payload_tree,
    mut elm: *mut event_payload_item,
) {
    let mut parent: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut gparent: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut tmp: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
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
unsafe extern "C" fn event_payload_tree_RB_INSERT(
    mut head: *mut event_payload_tree,
    mut elm: *mut event_payload_item,
) -> *mut event_payload_item {
    let mut tmp: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut parent: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = event_payload_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<event_payload_item>();
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
    event_payload_tree_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<event_payload_item>();
}
unsafe extern "C" fn event_payload_tree_RB_FIND(
    mut head: *mut event_payload_tree,
    mut elm: *mut event_payload_item,
) -> *mut event_payload_item {
    let mut tmp: *mut event_payload_item = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = event_payload_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<event_payload_item>();
}
unsafe extern "C" fn event_payload_find(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut event_payload_item {
    let mut find: event_payload_item = event_payload_item {
        name: name as *mut ::core::ffi::c_char,
        type_0: EVENT_PAYLOAD_STRING,
        c2rust_unnamed: C2RustUnnamed_36 {
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        entry: C2RustUnnamed_35 {
            rbe_left: ::core::ptr::null_mut::<event_payload_item>(),
            rbe_right: ::core::ptr::null_mut::<event_payload_item>(),
            rbe_parent: ::core::ptr::null_mut::<event_payload_item>(),
            rbe_color: 0,
        },
    };
    return event_payload_tree_RB_FIND(&raw mut (*ep).items, &raw mut find);
}
unsafe extern "C" fn event_payload_free_target(mut ep: *mut event_payload) {
    let mut target: *mut cmd_find_state = &raw mut (*ep).target;
    if !(*target).s.is_null() {
        session_remove_ref(
            (*target).s,
            b"event_payload_free_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !(*target).w.is_null() {
        window_remove_ref(
            (*target).w,
            b"event_payload_free_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !(*target).wp.is_null() {
        window_pane_remove_ref(
            (*target).wp,
            b"event_payload_free_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    cmd_find_clear_state(target, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn event_payload_free_value(mut epi: *mut event_payload_item) {
    match (*epi).type_0 as ::core::ffi::c_uint {
        0 => {
            free((*epi).c2rust_unnamed.string as *mut ::core::ffi::c_void);
        }
        4 => {
            server_client_unref((*epi).c2rust_unnamed.client);
        }
        5 => {
            session_remove_ref(
                (*epi).c2rust_unnamed.session,
                b"event_payload_free_value\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        6 => {
            window_remove_ref(
                (*epi).c2rust_unnamed.window,
                b"event_payload_free_value\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        7 => {
            window_pane_remove_ref(
                (*epi).c2rust_unnamed.pane,
                b"event_payload_free_value\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        8 => {
            if (*epi).c2rust_unnamed.pointer.free_cb.is_some() {
                (*epi)
                    .c2rust_unnamed
                    .pointer
                    .free_cb
                    .expect("non-null function pointer")(
                    (*epi).c2rust_unnamed.pointer.ptr
                );
            }
        }
        2 | 3 | 1 | _ => {}
    };
}
unsafe extern "C" fn event_payload_set_item(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut new: *mut event_payload_item,
) {
    let mut old: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    (*new).name = xstrdup(name);
    old = event_payload_tree_RB_INSERT(&raw mut (*ep).items, new);
    if !old.is_null() {
        event_payload_tree_RB_REMOVE(&raw mut (*ep).items, old);
        event_payload_free_value(old);
        free((*old).name as *mut ::core::ffi::c_void);
        free(old as *mut ::core::ffi::c_void);
        event_payload_tree_RB_INSERT(&raw mut (*ep).items, new);
    }
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_create() -> *mut event_payload {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    ep = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload>() as size_t,
    ) as *mut event_payload;
    (*ep).items.rbh_root = ::core::ptr::null_mut::<event_payload_item>();
    cmd_find_clear_state(&raw mut (*ep).target, 0 as ::core::ffi::c_int);
    return ep;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_free(mut ep: *mut event_payload) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut epi1: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    if !ep.is_null() {
        epi = event_payload_tree_RB_MINMAX(&raw mut (*ep).items, RB_NEGINF);
        while !epi.is_null() && {
            epi1 = event_payload_tree_RB_NEXT(epi);
            1 as ::core::ffi::c_int != 0
        } {
            event_payload_tree_RB_REMOVE(&raw mut (*ep).items, epi);
            event_payload_free_value(epi);
            free((*epi).name as *mut ::core::ffi::c_void);
            free(epi as *mut ::core::ffi::c_void);
            epi = epi1;
        }
        event_payload_free_target(ep);
        free(ep as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_target(
    mut ep: *mut event_payload,
    mut fs: *mut cmd_find_state,
) {
    let mut target: *mut cmd_find_state = &raw mut (*ep).target;
    event_payload_free_target(ep);
    if !(*fs).s.is_null() {
        session_add_ref(
            (*fs).s,
            b"event_payload_set_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*target).s = (*fs).s;
    }
    if !(*fs).wl.is_null() {
        (*target).idx = (*(*fs).wl).idx;
        if (*target).s.is_null() {
            session_add_ref(
                (*(*fs).wl).session,
                b"event_payload_set_target\0" as *const u8 as *const ::core::ffi::c_char,
            );
            (*target).s = (*(*fs).wl).session;
        }
    } else {
        (*target).idx = -(1 as ::core::ffi::c_int);
    }
    if !(*fs).w.is_null() {
        window_add_ref(
            (*fs).w,
            b"event_payload_set_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*target).w = (*fs).w;
    } else if !(*fs).wl.is_null() {
        window_add_ref(
            (*(*fs).wl).window,
            b"event_payload_set_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*target).w = (*(*fs).wl).window;
    }
    if !(*fs).wp.is_null() {
        window_pane_add_ref(
            (*fs).wp,
            b"event_payload_set_target\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*target).wp = (*fs).wp;
    }
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_target(
    mut ep: *mut event_payload,
    mut fs: *mut cmd_find_state,
) -> ::core::ffi::c_int {
    let mut t: *mut cmd_find_state = &raw mut (*ep).target;
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut flags: ::core::ffi::c_int = (*fs).flags;
    if (*t).idx != -(1 as ::core::ffi::c_int)
        && !(*t).s.is_null()
        && !(*t).w.is_null()
        && session_alive((*t).s) != 0
    {
        wl = winlink_find_by_index(&raw mut (*(*t).s).windows, (*t).idx);
        if !wl.is_null() && (*wl).window != (*t).w {
            wl = ::core::ptr::null_mut::<winlink>();
        }
    }
    cmd_find_clear_state(fs, flags);
    (*fs).s = (*t).s;
    (*fs).w = (*t).w;
    (*fs).wp = (*t).wp;
    (*fs).wl = wl;
    (*fs).idx = if !wl.is_null() {
        (*wl).idx
    } else {
        -(1 as ::core::ffi::c_int)
    };
    if cmd_find_valid_state(fs) != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if !wl.is_null() && !(*t).wp.is_null() && window_has_pane((*wl).window, (*t).wp) != 0 {
        cmd_find_from_winlink_pane(fs, wl, (*t).wp, flags);
        if cmd_find_valid_state(fs) != 0 {
            return 1 as ::core::ffi::c_int;
        }
    }
    if !(*t).wp.is_null()
        && cmd_find_from_pane(fs, (*t).wp, flags) == 0 as ::core::ffi::c_int
        && cmd_find_valid_state(fs) != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if !wl.is_null() {
        cmd_find_from_winlink(fs, wl, flags);
        if cmd_find_valid_state(fs) != 0 {
            return 1 as ::core::ffi::c_int;
        }
    }
    if !(*t).s.is_null()
        && !(*t).w.is_null()
        && session_alive((*t).s) != 0
        && cmd_find_from_session_window(fs, (*t).s, (*t).w, flags) == 0 as ::core::ffi::c_int
        && cmd_find_valid_state(fs) != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if !(*t).s.is_null() && session_alive((*t).s) != 0 {
        cmd_find_from_session(fs, (*t).s, flags);
        if cmd_find_valid_state(fs) != 0 {
            return 1 as ::core::ffi::c_int;
        }
    }
    if cmd_find_from_nothing(fs, flags) == 0 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    cmd_find_clear_state(fs, flags);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_string(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_STRING;
    xvasprintf(&raw mut (*epi).c2rust_unnamed.string, fmt, ap);
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_time(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: time_t,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_TIME;
    (*epi).c2rust_unnamed.time = value;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_int(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: ::core::ffi::c_int,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_INT;
    (*epi).c2rust_unnamed.number = value;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_uint(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: u_int,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_UINT;
    (*epi).c2rust_unnamed.unsigned_number = value;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_client(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut c: *mut client,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    (*c).references += 1;
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_CLIENT;
    (*epi).c2rust_unnamed.client = c;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_session(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut s: *mut session,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    session_add_ref(
        s,
        b"event_payload_set_session\0" as *const u8 as *const ::core::ffi::c_char,
    );
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_SESSION;
    (*epi).c2rust_unnamed.session = s;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_window(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut w: *mut window,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    window_add_ref(
        w,
        b"event_payload_set_window\0" as *const u8 as *const ::core::ffi::c_char,
    );
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_WINDOW;
    (*epi).c2rust_unnamed.window = w;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_pane(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut wp: *mut window_pane,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    window_pane_add_ref(
        wp,
        b"event_payload_set_pane\0" as *const u8 as *const ::core::ffi::c_char,
    );
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_PANE;
    (*epi).c2rust_unnamed.pane = wp;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_set_pointer(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut ptr: *mut ::core::ffi::c_void,
    mut free_cb: event_payload_free_cb,
    mut print_cb: event_payload_print_cb,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<event_payload_item>() as size_t,
    ) as *mut event_payload_item;
    (*epi).type_0 = EVENT_PAYLOAD_POINTER;
    (*epi).c2rust_unnamed.pointer.ptr = ptr;
    (*epi).c2rust_unnamed.pointer.free_cb = free_cb;
    (*epi).c2rust_unnamed.pointer.print_cb = print_cb;
    event_payload_set_item(ep, name, epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_string(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return (*epi).c2rust_unnamed.string;
}
unsafe extern "C" fn event_payload_add_item(
    mut epi: *mut event_payload_item,
    mut evb: *mut evbuffer,
) {
    match (*epi).type_0 as ::core::ffi::c_uint {
        0 => {
            evbuffer_add_printf(
                evb,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*epi).c2rust_unnamed.string,
            );
        }
        1 => {
            evbuffer_add_printf(
                evb,
                b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                (*epi).c2rust_unnamed.time as ::core::ffi::c_longlong,
            );
        }
        2 => {
            evbuffer_add_printf(
                evb,
                b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                (*epi).c2rust_unnamed.number,
            );
        }
        3 => {
            evbuffer_add_printf(
                evb,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*epi).c2rust_unnamed.unsigned_number,
            );
        }
        4 => {
            evbuffer_add_printf(
                evb,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*(*epi).c2rust_unnamed.client).name,
            );
        }
        5 => {
            evbuffer_add_printf(
                evb,
                b"$%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*(*epi).c2rust_unnamed.session).id,
            );
        }
        6 => {
            evbuffer_add_printf(
                evb,
                b"@%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*(*epi).c2rust_unnamed.window).id,
            );
        }
        7 => {
            evbuffer_add_printf(
                evb,
                b"%%%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*(*epi).c2rust_unnamed.pane).id,
            );
        }
        8 => {
            if (*epi).c2rust_unnamed.pointer.print_cb.is_some() {
                (*epi)
                    .c2rust_unnamed
                    .pointer
                    .print_cb
                    .expect("non-null function pointer")(
                    (*epi).c2rust_unnamed.pointer.ptr, evb
                );
            } else {
                evbuffer_add_printf(
                    evb,
                    b"%p\0" as *const u8 as *const ::core::ffi::c_char,
                    (*epi).c2rust_unnamed.pointer.ptr,
                );
            }
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_item_print(
    mut epi: *mut event_payload_item,
) -> *mut ::core::ffi::c_char {
    let mut evb: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut size: size_t = 0;
    evb = evbuffer_new();
    if evb.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    event_payload_add_item(epi, evb);
    size = evbuffer_get_length(evb);
    if size != 0 as size_t {
        value = xmemdup(
            evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            size,
        );
    } else {
        value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    evbuffer_free(evb);
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_print(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return event_payload_item_print(epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_add_formats(
    mut ep: *mut event_payload,
    mut ft: *mut format_tree,
    mut prefix: *const ::core::ffi::c_char,
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if prefix.is_null() {
        prefix = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    epi = event_payload_tree_RB_MINMAX(&raw mut (*ep).items, RB_NEGINF);
    while !epi.is_null() {
        key = (*epi).name;
        if !(*key as ::core::ffi::c_int == '_' as i32) {
            value = event_payload_item_print(epi);
            xasprintf(
                &raw mut name,
                b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                prefix,
                key,
            );
            format_add(
                ft,
                name,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                value,
            );
            free(name as *mut ::core::ffi::c_void);
            free(value as *mut ::core::ffi::c_void);
            if (*epi).type_0 as ::core::ffi::c_uint
                == EVENT_PAYLOAD_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                xasprintf(
                    &raw mut name,
                    b"%s%s_name\0" as *const u8 as *const ::core::ffi::c_char,
                    prefix,
                    key,
                );
                format_add(
                    ft,
                    name,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*(*epi).c2rust_unnamed.session).name,
                );
                free(name as *mut ::core::ffi::c_void);
            } else if (*epi).type_0 as ::core::ffi::c_uint
                == EVENT_PAYLOAD_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                xasprintf(
                    &raw mut name,
                    b"%s%s_name\0" as *const u8 as *const ::core::ffi::c_char,
                    prefix,
                    key,
                );
                format_add(
                    ft,
                    name,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*(*epi).c2rust_unnamed.window).name,
                );
                free(name as *mut ::core::ffi::c_void);
            }
        }
        epi = event_payload_tree_RB_NEXT(epi);
    }
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_first(
    mut ep: *mut event_payload,
) -> *mut event_payload_item {
    return event_payload_tree_RB_MINMAX(&raw mut (*ep).items, RB_NEGINF);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_next(
    mut epi: *mut event_payload_item,
) -> *mut event_payload_item {
    return event_payload_tree_RB_NEXT(epi);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_item_name(
    mut epi: *mut event_payload_item,
) -> *const ::core::ffi::c_char {
    return (*epi).name;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_item_type(
    mut epi: *mut event_payload_item,
) -> event_payload_type {
    return (*epi).type_0;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_log(
    mut ep: *mut event_payload,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    let mut evb: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    let mut ap: ::core::ffi::VaList;
    let mut prefix: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ap = args.clone();
    xvasprintf(&raw mut prefix, fmt, ap);
    evb = evbuffer_new();
    if evb.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if !ep.is_null() {
        epi = event_payload_tree_RB_MINMAX(&raw mut (*ep).items, RB_NEGINF);
        while !epi.is_null() {
            if evbuffer_get_length(evb) != 0 as size_t {
                evbuffer_add_printf(evb, b", \0" as *const u8 as *const ::core::ffi::c_char);
            }
            evbuffer_add_printf(
                evb,
                b"%s=\0" as *const u8 as *const ::core::ffi::c_char,
                (*epi).name,
            );
            event_payload_add_item(epi, evb);
            epi = event_payload_tree_RB_NEXT(epi);
        }
    }
    log_debug(
        b"%s%.*s\0" as *const u8 as *const ::core::ffi::c_char,
        prefix,
        evbuffer_get_length(evb) as ::core::ffi::c_int,
        evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t) as *mut ::core::ffi::c_char,
    );
    evbuffer_free(evb);
    free(prefix as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_time(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> time_t {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_TIME as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as time_t;
    }
    return (*epi).c2rust_unnamed.time;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_int(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_INT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    *value = (*epi).c2rust_unnamed.number;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_uint(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
    mut value: *mut u_int,
) -> ::core::ffi::c_int {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_UINT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    *value = (*epi).c2rust_unnamed.unsigned_number;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_client(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut client {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_CLIENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<client>();
    }
    return (*epi).c2rust_unnamed.client;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_session(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut session {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<session>();
    }
    return (*epi).c2rust_unnamed.session;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_window(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut window {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<window>();
    }
    return (*epi).c2rust_unnamed.window;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_pane(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut window_pane {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<window_pane>();
    }
    return (*epi).c2rust_unnamed.pane;
}
#[no_mangle]
pub unsafe extern "C" fn event_payload_get_pointer(
    mut ep: *mut event_payload,
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_void {
    let mut epi: *mut event_payload_item = ::core::ptr::null_mut::<event_payload_item>();
    epi = event_payload_find(ep, name);
    if epi.is_null()
        || (*epi).type_0 as ::core::ffi::c_uint
            != EVENT_PAYLOAD_POINTER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    return (*epi).c2rust_unnamed.pointer.ptr;
}

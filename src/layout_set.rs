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
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn events_fire_window(_: *const ::core::ffi::c_char, _: *mut window);
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn args_string_percentage(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn server_redraw_window(_: *mut window);
    fn window_resize(
        _: *mut window,
        _: u_int,
        _: u_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn window_count_panes(_: *mut window, _: ::core::ffi::c_int) -> u_int;
    fn layout_create_cell(_: *mut layout_cell) -> *mut layout_cell;
    fn layout_print_cell(_: *mut layout_cell, _: *const ::core::ffi::c_char, _: u_int);
    fn layout_set_size(
        _: *mut layout_cell,
        _: u_int,
        _: u_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn layout_make_node(_: *mut layout_cell, _: layout_type);
    fn layout_cell_is_tiled(_: *mut layout_cell) -> ::core::ffi::c_int;
    fn layout_fix_offsets(_: *mut window);
    fn layout_fix_panes(_: *mut window, _: *mut window_pane);
    fn layout_resize_adjust(
        _: *mut window,
        _: *mut layout_cell,
        _: layout_type,
        _: ::core::ffi::c_int,
    );
    fn layout_free(_: *mut window, _: ::core::ffi::c_int);
    fn layout_spread_cell(_: *mut window, _: *mut layout_cell) -> ::core::ffi::c_int;
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
pub struct C2RustUnnamed_35 {
    pub name: *const ::core::ffi::c_char,
    pub arrange: Option<unsafe extern "C" fn(*mut window) -> ()>,
}
pub const PANE_MINIMUM: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
static mut layout_sets: [C2RustUnnamed_35; 7] = unsafe {
    [
        C2RustUnnamed_35 {
            name: b"even-horizontal\0" as *const u8 as *const ::core::ffi::c_char,
            arrange: Some(layout_set_even_h as unsafe extern "C" fn(*mut window) -> ()),
        },
        C2RustUnnamed_35 {
            name: b"even-vertical\0" as *const u8 as *const ::core::ffi::c_char,
            arrange: Some(layout_set_even_v as unsafe extern "C" fn(*mut window) -> ()),
        },
        C2RustUnnamed_35 {
            name: b"main-horizontal\0" as *const u8 as *const ::core::ffi::c_char,
            arrange: Some(layout_set_main_h as unsafe extern "C" fn(*mut window) -> ()),
        },
        C2RustUnnamed_35 {
            name: b"main-horizontal-mirrored\0" as *const u8 as *const ::core::ffi::c_char,
            arrange: Some(layout_set_main_h_mirrored as unsafe extern "C" fn(*mut window) -> ()),
        },
        C2RustUnnamed_35 {
            name: b"main-vertical\0" as *const u8 as *const ::core::ffi::c_char,
            arrange: Some(layout_set_main_v as unsafe extern "C" fn(*mut window) -> ()),
        },
        C2RustUnnamed_35 {
            name: b"main-vertical-mirrored\0" as *const u8 as *const ::core::ffi::c_char,
            arrange: Some(layout_set_main_v_mirrored as unsafe extern "C" fn(*mut window) -> ()),
        },
        C2RustUnnamed_35 {
            name: b"tiled\0" as *const u8 as *const ::core::ffi::c_char,
            arrange: Some(layout_set_tiled as unsafe extern "C" fn(*mut window) -> ()),
        },
    ]
};
#[no_mangle]
pub unsafe extern "C" fn layout_set_lookup(
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    let mut matched: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
    {
        if strcmp(layout_sets[i as usize].name, name) == 0 as ::core::ffi::c_int {
            return i as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
    {
        if strncmp(layout_sets[i as usize].name, name, strlen(name)) == 0 as ::core::ffi::c_int {
            if matched != -(1 as ::core::ffi::c_int) {
                return -(1 as ::core::ffi::c_int);
            }
            matched = i as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    return matched;
}
#[no_mangle]
pub unsafe extern "C" fn layout_set_select(mut w: *mut window, mut layout: u_int) -> u_int {
    if layout as usize
        > (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
            .wrapping_sub(1 as usize)
    {
        layout = (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
            .wrapping_sub(1 as usize) as u_int;
    }
    if layout_sets[layout as usize].arrange.is_some() {
        layout_sets[layout as usize]
            .arrange
            .expect("non-null function pointer")(w);
    }
    (*w).lastlayout = layout as ::core::ffi::c_int;
    return layout;
}
#[no_mangle]
pub unsafe extern "C" fn layout_set_next(mut w: *mut window) -> u_int {
    let mut layout: u_int = 0;
    if (*w).lastlayout == -(1 as ::core::ffi::c_int) {
        layout = 0 as u_int;
    } else {
        layout = ((*w).lastlayout + 1 as ::core::ffi::c_int) as u_int;
        if layout as usize
            > (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
                .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
                .wrapping_sub(1 as usize)
        {
            layout = 0 as u_int;
        }
    }
    if layout_sets[layout as usize].arrange.is_some() {
        layout_sets[layout as usize]
            .arrange
            .expect("non-null function pointer")(w);
    }
    (*w).lastlayout = layout as ::core::ffi::c_int;
    return layout;
}
#[no_mangle]
pub unsafe extern "C" fn layout_set_previous(mut w: *mut window) -> u_int {
    let mut layout: u_int = 0;
    if (*w).lastlayout == -(1 as ::core::ffi::c_int) {
        layout = (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
            .wrapping_sub(1 as usize) as u_int;
    } else {
        layout = (*w).lastlayout as u_int;
        if layout == 0 as u_int {
            layout = (::core::mem::size_of::<[C2RustUnnamed_35; 7]>() as usize)
                .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
                .wrapping_sub(1 as usize) as u_int;
        } else {
            layout = layout.wrapping_sub(1);
        }
    }
    if layout_sets[layout as usize].arrange.is_some() {
        layout_sets[layout as usize]
            .arrange
            .expect("non-null function pointer")(w);
    }
    (*w).lastlayout = layout as ::core::ffi::c_int;
    return layout;
}
unsafe extern "C" fn layout_set_first_tiled(mut w: *mut window) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if !(*wp).layout_cell.is_null()
            && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) != 0
        {
            return wp;
        }
        wp = (*wp).entry.tqe_next;
    }
    return ::core::ptr::null_mut::<window_pane>();
}
unsafe extern "C" fn layout_set_link_floating(mut w: *mut window, mut lcroot: *mut layout_cell) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
        if layout_cell_is_tiled(lc) == 0 {
            (*lc).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
            (*lc).entry.tqe_prev = (*lcroot).cells.tqh_last;
            *(*lcroot).cells.tqh_last = lc;
            (*lcroot).cells.tqh_last = &raw mut (*lc).entry.tqe_next;
            (*lc).parent = lcroot;
        }
        wp = (*wp).entry.tqe_next;
    }
}
unsafe extern "C" fn layout_set_even(mut w: *mut window, mut type_0: layout_type) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_even\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        sx = n
            .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
            .wrapping_sub(1 as u_int);
        if sx < (*w).sx {
            sx = (*w).sx;
        }
        sy = (*w).sy;
    } else {
        sy = n
            .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
            .wrapping_sub(1 as u_int);
        if sy < (*w).sy {
            sy = (*w).sy;
        }
        sx = (*w).sx;
    }
    layout_free(w, 1 as ::core::ffi::c_int);
    (*w).layout_root = layout_create_cell(::core::ptr::null_mut::<layout_cell>());
    lcroot = (*w).layout_root;
    layout_set_size(
        lcroot,
        sx,
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, type_0);
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        lcchild = (*wp).layout_cell as *mut layout_cell;
        (*lcchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
        (*lcchild).entry.tqe_prev = (*lcroot).cells.tqh_last;
        *(*lcroot).cells.tqh_last = lcchild;
        (*lcroot).cells.tqh_last = &raw mut (*lcchild).entry.tqe_next;
        (*lcchild).parent = lcroot;
        if layout_cell_is_tiled(lcchild) != 0 {
            (*lcchild).g.sx = (*w).sx;
            (*lcchild).g.sy = (*w).sy;
        }
        wp = (*wp).entry.tqe_next;
    }
    layout_spread_cell(w, lcroot);
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_even\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        w,
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
}
unsafe extern "C" fn layout_set_even_h(mut w: *mut window) {
    layout_set_even(w, LAYOUT_LEFTRIGHT);
}
unsafe extern "C" fn layout_set_even_v(mut w: *mut window) {
    layout_set_even(w, LAYOUT_TOPBOTTOM);
}
unsafe extern "C" fn layout_set_main_h(mut w: *mut window) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wpmain: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcmain: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut mainh: u_int = 0;
    let mut otherh: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_main_h\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    n = n.wrapping_sub(1);
    sy = (*w).sy.wrapping_sub(1 as u_int);
    s = options_get_string(
        (*w).options,
        b"main-pane-height\0" as *const u8 as *const ::core::ffi::c_char,
    );
    mainh = args_string_percentage(
        s,
        0 as ::core::ffi::c_longlong,
        sy as ::core::ffi::c_longlong,
        sy as ::core::ffi::c_longlong,
        &raw mut cause,
    ) as u_int;
    if !cause.is_null() {
        mainh = 24 as u_int;
        free(cause as *mut ::core::ffi::c_void);
    }
    if mainh.wrapping_add(PANE_MINIMUM as u_int) >= sy {
        if sy <= (PANE_MINIMUM + PANE_MINIMUM) as u_int {
            mainh = PANE_MINIMUM as u_int;
        } else {
            mainh = sy.wrapping_sub(PANE_MINIMUM as u_int);
        }
        otherh = PANE_MINIMUM as u_int;
    } else {
        s = options_get_string(
            (*w).options,
            b"other-pane-height\0" as *const u8 as *const ::core::ffi::c_char,
        );
        otherh = args_string_percentage(
            s,
            0 as ::core::ffi::c_longlong,
            sy as ::core::ffi::c_longlong,
            sy as ::core::ffi::c_longlong,
            &raw mut cause,
        ) as u_int;
        if !cause.is_null() || otherh == 0 as u_int {
            otherh = sy.wrapping_sub(mainh);
            free(cause as *mut ::core::ffi::c_void);
        } else if otherh > sy || sy.wrapping_sub(otherh) < mainh {
            otherh = sy.wrapping_sub(mainh);
        } else {
            mainh = sy.wrapping_sub(otherh);
        }
    }
    sx = n
        .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
        .wrapping_sub(1 as u_int);
    if sx < (*w).sx {
        sx = (*w).sx;
    }
    layout_free(w, 1 as ::core::ffi::c_int);
    (*w).layout_root = layout_create_cell(::core::ptr::null_mut::<layout_cell>());
    lcroot = (*w).layout_root;
    layout_set_size(
        lcroot,
        sx,
        mainh.wrapping_add(otherh).wrapping_add(1 as u_int),
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, LAYOUT_TOPBOTTOM);
    wpmain = layout_set_first_tiled(w);
    lcmain = (*wpmain).layout_cell as *mut layout_cell;
    (*lcmain).parent = lcroot;
    layout_set_size(
        lcmain,
        sx,
        mainh,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    (*lcmain).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
    (*lcmain).entry.tqe_prev = (*lcroot).cells.tqh_last;
    *(*lcroot).cells.tqh_last = lcmain;
    (*lcroot).cells.tqh_last = &raw mut (*lcmain).entry.tqe_next;
    if n == 1 as u_int {
        wp = (*wpmain).entry.tqe_next;
        while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
            wp = (*wp).entry.tqe_next;
        }
        (*(*wp).layout_cell).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
        (*(*wp).layout_cell).entry.tqe_prev = (*lcroot).cells.tqh_last;
        *(*lcroot).cells.tqh_last = (*wp).layout_cell as *mut layout_cell;
        (*lcroot).cells.tqh_last = &raw mut (*(*wp).layout_cell).entry.tqe_next;
        (*(*wp).layout_cell).parent = lcroot;
        layout_set_size(
            (*wp).layout_cell as *mut layout_cell,
            sx,
            otherh,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_set_link_floating(w, lcroot);
    } else {
        lcother = layout_create_cell(lcroot);
        layout_set_size(
            lcother,
            sx,
            otherh,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_make_node(lcother, LAYOUT_LEFTRIGHT);
        (*lcother).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
        (*lcother).entry.tqe_prev = (*lcroot).cells.tqh_last;
        *(*lcroot).cells.tqh_last = lcother;
        (*lcroot).cells.tqh_last = &raw mut (*lcother).entry.tqe_next;
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if !(wp == wpmain) {
                lcchild = (*wp).layout_cell as *mut layout_cell;
                (*lcchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
                (*lcchild).entry.tqe_prev = (*lcother).cells.tqh_last;
                *(*lcother).cells.tqh_last = lcchild;
                (*lcother).cells.tqh_last = &raw mut (*lcchild).entry.tqe_next;
                (*lcchild).parent = lcother;
                if layout_cell_is_tiled(lcchild) != 0 {
                    layout_set_size(
                        lcchild,
                        PANE_MINIMUM as u_int,
                        otherh,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
            }
            wp = (*wp).entry.tqe_next;
        }
        layout_spread_cell(w, lcother);
    }
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_main_h\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        w,
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
}
unsafe extern "C" fn layout_set_main_h_mirrored(mut w: *mut window) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wpmain: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcmain: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut mainh: u_int = 0;
    let mut otherh: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_main_h_mirrored\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    n = n.wrapping_sub(1);
    sy = (*w).sy.wrapping_sub(1 as u_int);
    s = options_get_string(
        (*w).options,
        b"main-pane-height\0" as *const u8 as *const ::core::ffi::c_char,
    );
    mainh = args_string_percentage(
        s,
        0 as ::core::ffi::c_longlong,
        sy as ::core::ffi::c_longlong,
        sy as ::core::ffi::c_longlong,
        &raw mut cause,
    ) as u_int;
    if !cause.is_null() {
        mainh = 24 as u_int;
        free(cause as *mut ::core::ffi::c_void);
    }
    if mainh.wrapping_add(PANE_MINIMUM as u_int) >= sy {
        if sy <= (PANE_MINIMUM + PANE_MINIMUM) as u_int {
            mainh = PANE_MINIMUM as u_int;
        } else {
            mainh = sy.wrapping_sub(PANE_MINIMUM as u_int);
        }
        otherh = PANE_MINIMUM as u_int;
    } else {
        s = options_get_string(
            (*w).options,
            b"other-pane-height\0" as *const u8 as *const ::core::ffi::c_char,
        );
        otherh = args_string_percentage(
            s,
            0 as ::core::ffi::c_longlong,
            sy as ::core::ffi::c_longlong,
            sy as ::core::ffi::c_longlong,
            &raw mut cause,
        ) as u_int;
        if !cause.is_null() || otherh == 0 as u_int {
            otherh = sy.wrapping_sub(mainh);
            free(cause as *mut ::core::ffi::c_void);
        } else if otherh > sy || sy.wrapping_sub(otherh) < mainh {
            otherh = sy.wrapping_sub(mainh);
        } else {
            mainh = sy.wrapping_sub(otherh);
        }
    }
    sx = n
        .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
        .wrapping_sub(1 as u_int);
    if sx < (*w).sx {
        sx = (*w).sx;
    }
    layout_free(w, 1 as ::core::ffi::c_int);
    (*w).layout_root = layout_create_cell(::core::ptr::null_mut::<layout_cell>());
    lcroot = (*w).layout_root;
    layout_set_size(
        lcroot,
        sx,
        mainh.wrapping_add(otherh).wrapping_add(1 as u_int),
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, LAYOUT_TOPBOTTOM);
    wpmain = layout_set_first_tiled(w);
    lcmain = (*wpmain).layout_cell as *mut layout_cell;
    (*lcmain).parent = lcroot;
    layout_set_size(
        lcmain,
        sx,
        mainh,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    (*lcmain).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
    (*lcmain).entry.tqe_prev = (*lcroot).cells.tqh_last;
    *(*lcroot).cells.tqh_last = lcmain;
    (*lcroot).cells.tqh_last = &raw mut (*lcmain).entry.tqe_next;
    if n == 1 as u_int {
        wp = (*wpmain).entry.tqe_next;
        while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
            wp = (*wp).entry.tqe_next;
        }
        (*(*wp).layout_cell).entry.tqe_next = (*lcroot).cells.tqh_first;
        if !(*(*wp).layout_cell).entry.tqe_next.is_null() {
            (*(*lcroot).cells.tqh_first).entry.tqe_prev =
                &raw mut (*(*wp).layout_cell).entry.tqe_next;
        } else {
            (*lcroot).cells.tqh_last = &raw mut (*(*wp).layout_cell).entry.tqe_next;
        }
        (*lcroot).cells.tqh_first = (*wp).layout_cell as *mut layout_cell;
        (*(*wp).layout_cell).entry.tqe_prev = &raw mut (*lcroot).cells.tqh_first;
        (*(*wp).layout_cell).parent = lcroot;
        layout_set_size(
            (*wp).layout_cell as *mut layout_cell,
            sx,
            otherh,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_set_link_floating(w, lcroot);
    } else {
        lcother = layout_create_cell(lcroot);
        layout_set_size(
            lcother,
            sx,
            otherh,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_make_node(lcother, LAYOUT_LEFTRIGHT);
        (*lcother).entry.tqe_next = (*lcroot).cells.tqh_first;
        if !(*lcother).entry.tqe_next.is_null() {
            (*(*lcroot).cells.tqh_first).entry.tqe_prev = &raw mut (*lcother).entry.tqe_next;
        } else {
            (*lcroot).cells.tqh_last = &raw mut (*lcother).entry.tqe_next;
        }
        (*lcroot).cells.tqh_first = lcother;
        (*lcother).entry.tqe_prev = &raw mut (*lcroot).cells.tqh_first;
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if !(wp == wpmain) {
                lcchild = (*wp).layout_cell as *mut layout_cell;
                (*lcchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
                (*lcchild).entry.tqe_prev = (*lcother).cells.tqh_last;
                *(*lcother).cells.tqh_last = lcchild;
                (*lcother).cells.tqh_last = &raw mut (*lcchild).entry.tqe_next;
                (*lcchild).parent = lcother;
                if layout_cell_is_tiled(lcchild) != 0 {
                    layout_set_size(
                        lcchild,
                        PANE_MINIMUM as u_int,
                        otherh,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
            }
            wp = (*wp).entry.tqe_next;
        }
        layout_spread_cell(w, lcother);
    }
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_main_h_mirrored\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        w,
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
}
unsafe extern "C" fn layout_set_main_v(mut w: *mut window) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wpmain: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcmain: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut mainw: u_int = 0;
    let mut otherw: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_main_v\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    n = n.wrapping_sub(1);
    sx = (*w).sx.wrapping_sub(1 as u_int);
    s = options_get_string(
        (*w).options,
        b"main-pane-width\0" as *const u8 as *const ::core::ffi::c_char,
    );
    mainw = args_string_percentage(
        s,
        0 as ::core::ffi::c_longlong,
        sx as ::core::ffi::c_longlong,
        sx as ::core::ffi::c_longlong,
        &raw mut cause,
    ) as u_int;
    if !cause.is_null() {
        mainw = 80 as u_int;
        free(cause as *mut ::core::ffi::c_void);
    }
    if mainw.wrapping_add(PANE_MINIMUM as u_int) >= sx {
        if sx <= (PANE_MINIMUM + PANE_MINIMUM) as u_int {
            mainw = PANE_MINIMUM as u_int;
        } else {
            mainw = sx.wrapping_sub(PANE_MINIMUM as u_int);
        }
        otherw = PANE_MINIMUM as u_int;
    } else {
        s = options_get_string(
            (*w).options,
            b"other-pane-width\0" as *const u8 as *const ::core::ffi::c_char,
        );
        otherw = args_string_percentage(
            s,
            0 as ::core::ffi::c_longlong,
            sx as ::core::ffi::c_longlong,
            sx as ::core::ffi::c_longlong,
            &raw mut cause,
        ) as u_int;
        if !cause.is_null() || otherw == 0 as u_int {
            otherw = sx.wrapping_sub(mainw);
            free(cause as *mut ::core::ffi::c_void);
        } else if otherw > sx || sx.wrapping_sub(otherw) < mainw {
            otherw = sx.wrapping_sub(mainw);
        } else {
            mainw = sx.wrapping_sub(otherw);
        }
    }
    sy = n
        .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
        .wrapping_sub(1 as u_int);
    if sy < (*w).sy {
        sy = (*w).sy;
    }
    layout_free(w, 1 as ::core::ffi::c_int);
    (*w).layout_root = layout_create_cell(::core::ptr::null_mut::<layout_cell>());
    lcroot = (*w).layout_root;
    layout_set_size(
        lcroot,
        mainw.wrapping_add(otherw).wrapping_add(1 as u_int),
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, LAYOUT_LEFTRIGHT);
    wpmain = layout_set_first_tiled(w);
    lcmain = (*wpmain).layout_cell as *mut layout_cell;
    (*lcmain).parent = lcroot;
    layout_set_size(
        lcmain,
        mainw,
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    (*lcmain).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
    (*lcmain).entry.tqe_prev = (*lcroot).cells.tqh_last;
    *(*lcroot).cells.tqh_last = lcmain;
    (*lcroot).cells.tqh_last = &raw mut (*lcmain).entry.tqe_next;
    if n == 1 as u_int {
        wp = (*wpmain).entry.tqe_next;
        while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
            wp = (*wp).entry.tqe_next;
        }
        (*(*wp).layout_cell).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
        (*(*wp).layout_cell).entry.tqe_prev = (*lcroot).cells.tqh_last;
        *(*lcroot).cells.tqh_last = (*wp).layout_cell as *mut layout_cell;
        (*lcroot).cells.tqh_last = &raw mut (*(*wp).layout_cell).entry.tqe_next;
        (*(*wp).layout_cell).parent = lcroot;
        layout_set_size(
            (*wp).layout_cell as *mut layout_cell,
            otherw,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_set_link_floating(w, lcroot);
    } else {
        lcother = layout_create_cell(lcroot);
        layout_make_node(lcother, LAYOUT_TOPBOTTOM);
        layout_set_size(
            lcother,
            otherw,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        (*lcother).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
        (*lcother).entry.tqe_prev = (*lcroot).cells.tqh_last;
        *(*lcroot).cells.tqh_last = lcother;
        (*lcroot).cells.tqh_last = &raw mut (*lcother).entry.tqe_next;
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if !(wp == wpmain) {
                lcchild = (*wp).layout_cell as *mut layout_cell;
                (*lcchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
                (*lcchild).entry.tqe_prev = (*lcother).cells.tqh_last;
                *(*lcother).cells.tqh_last = lcchild;
                (*lcother).cells.tqh_last = &raw mut (*lcchild).entry.tqe_next;
                (*lcchild).parent = lcother;
                if layout_cell_is_tiled(lcchild) != 0 {
                    layout_set_size(
                        lcchild,
                        otherw,
                        PANE_MINIMUM as u_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
            }
            wp = (*wp).entry.tqe_next;
        }
        layout_spread_cell(w, lcother);
    }
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_main_v\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        w,
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
}
unsafe extern "C" fn layout_set_main_v_mirrored(mut w: *mut window) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wpmain: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcmain: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcother: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut mainw: u_int = 0;
    let mut otherw: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_main_v_mirrored\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    n = n.wrapping_sub(1);
    sx = (*w).sx.wrapping_sub(1 as u_int);
    s = options_get_string(
        (*w).options,
        b"main-pane-width\0" as *const u8 as *const ::core::ffi::c_char,
    );
    mainw = args_string_percentage(
        s,
        0 as ::core::ffi::c_longlong,
        sx as ::core::ffi::c_longlong,
        sx as ::core::ffi::c_longlong,
        &raw mut cause,
    ) as u_int;
    if !cause.is_null() {
        mainw = 80 as u_int;
        free(cause as *mut ::core::ffi::c_void);
    }
    if mainw.wrapping_add(PANE_MINIMUM as u_int) >= sx {
        if sx <= (PANE_MINIMUM + PANE_MINIMUM) as u_int {
            mainw = PANE_MINIMUM as u_int;
        } else {
            mainw = sx.wrapping_sub(PANE_MINIMUM as u_int);
        }
        otherw = PANE_MINIMUM as u_int;
    } else {
        s = options_get_string(
            (*w).options,
            b"other-pane-width\0" as *const u8 as *const ::core::ffi::c_char,
        );
        otherw = args_string_percentage(
            s,
            0 as ::core::ffi::c_longlong,
            sx as ::core::ffi::c_longlong,
            sx as ::core::ffi::c_longlong,
            &raw mut cause,
        ) as u_int;
        if !cause.is_null() || otherw == 0 as u_int {
            otherw = sx.wrapping_sub(mainw);
            free(cause as *mut ::core::ffi::c_void);
        } else if otherw > sx || sx.wrapping_sub(otherw) < mainw {
            otherw = sx.wrapping_sub(mainw);
        } else {
            mainw = sx.wrapping_sub(otherw);
        }
    }
    sy = n
        .wrapping_mul((PANE_MINIMUM + 1 as ::core::ffi::c_int) as u_int)
        .wrapping_sub(1 as u_int);
    if sy < (*w).sy {
        sy = (*w).sy;
    }
    layout_free(w, 1 as ::core::ffi::c_int);
    (*w).layout_root = layout_create_cell(::core::ptr::null_mut::<layout_cell>());
    lcroot = (*w).layout_root;
    layout_set_size(
        lcroot,
        mainw.wrapping_add(otherw).wrapping_add(1 as u_int),
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, LAYOUT_LEFTRIGHT);
    wpmain = layout_set_first_tiled(w);
    lcmain = (*wpmain).layout_cell as *mut layout_cell;
    (*lcmain).parent = lcroot;
    layout_set_size(
        lcmain,
        mainw,
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    (*lcmain).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
    (*lcmain).entry.tqe_prev = (*lcroot).cells.tqh_last;
    *(*lcroot).cells.tqh_last = lcmain;
    (*lcroot).cells.tqh_last = &raw mut (*lcmain).entry.tqe_next;
    if n == 1 as u_int {
        wp = (*wpmain).entry.tqe_next;
        while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
            wp = (*wp).entry.tqe_next;
        }
        (*(*wp).layout_cell).entry.tqe_next = (*lcroot).cells.tqh_first;
        if !(*(*wp).layout_cell).entry.tqe_next.is_null() {
            (*(*lcroot).cells.tqh_first).entry.tqe_prev =
                &raw mut (*(*wp).layout_cell).entry.tqe_next;
        } else {
            (*lcroot).cells.tqh_last = &raw mut (*(*wp).layout_cell).entry.tqe_next;
        }
        (*lcroot).cells.tqh_first = (*wp).layout_cell as *mut layout_cell;
        (*(*wp).layout_cell).entry.tqe_prev = &raw mut (*lcroot).cells.tqh_first;
        (*(*wp).layout_cell).parent = lcroot;
        layout_set_size(
            (*wp).layout_cell as *mut layout_cell,
            otherw,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        layout_set_link_floating(w, lcroot);
    } else {
        lcother = layout_create_cell(lcroot);
        layout_make_node(lcother, LAYOUT_TOPBOTTOM);
        layout_set_size(
            lcother,
            otherw,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        (*lcother).entry.tqe_next = (*lcroot).cells.tqh_first;
        if !(*lcother).entry.tqe_next.is_null() {
            (*(*lcroot).cells.tqh_first).entry.tqe_prev = &raw mut (*lcother).entry.tqe_next;
        } else {
            (*lcroot).cells.tqh_last = &raw mut (*lcother).entry.tqe_next;
        }
        (*lcroot).cells.tqh_first = lcother;
        (*lcother).entry.tqe_prev = &raw mut (*lcroot).cells.tqh_first;
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if !(wp == wpmain) {
                lcchild = (*wp).layout_cell as *mut layout_cell;
                (*lcchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
                (*lcchild).entry.tqe_prev = (*lcother).cells.tqh_last;
                *(*lcother).cells.tqh_last = lcchild;
                (*lcother).cells.tqh_last = &raw mut (*lcchild).entry.tqe_next;
                (*lcchild).parent = lcother;
                if layout_cell_is_tiled(lcchild) != 0 {
                    layout_set_size(
                        lcchild,
                        otherw,
                        PANE_MINIMUM as u_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                }
            }
            wp = (*wp).entry.tqe_next;
        }
        layout_spread_cell(w, lcother);
    }
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_main_v_mirrored\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        w,
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
}
unsafe extern "C" fn layout_set_tiled(mut w: *mut window) {
    let mut oo: *mut options = (*w).options;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcroot: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcrow: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0;
    let mut width: u_int = 0;
    let mut height: u_int = 0;
    let mut used: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut columns: u_int = 0;
    let mut rows: u_int = 0;
    let mut max_columns: u_int = 0;
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_tiled\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    n = window_count_panes(w, 0 as ::core::ffi::c_int);
    if n <= 1 as u_int {
        return;
    }
    max_columns = options_get_number(
        oo,
        b"tiled-layout-max-columns\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    columns = 1 as u_int;
    rows = columns;
    while rows.wrapping_mul(columns) < n {
        rows = rows.wrapping_add(1);
        if rows.wrapping_mul(columns) < n && (max_columns == 0 as u_int || columns < max_columns) {
            columns = columns.wrapping_add(1);
        }
    }
    width = (*w)
        .sx
        .wrapping_sub(columns.wrapping_sub(1 as u_int))
        .wrapping_div(columns);
    if width < PANE_MINIMUM as u_int {
        width = PANE_MINIMUM as u_int;
    }
    height = (*w)
        .sy
        .wrapping_sub(rows.wrapping_sub(1 as u_int))
        .wrapping_div(rows);
    if height < PANE_MINIMUM as u_int {
        height = PANE_MINIMUM as u_int;
    }
    sx = width
        .wrapping_add(1 as u_int)
        .wrapping_mul(columns)
        .wrapping_sub(1 as u_int);
    if sx < (*w).sx {
        sx = (*w).sx;
    }
    sy = height
        .wrapping_add(1 as u_int)
        .wrapping_mul(rows)
        .wrapping_sub(1 as u_int);
    if sy < (*w).sy {
        sy = (*w).sy;
    }
    layout_free(w, 1 as ::core::ffi::c_int);
    (*w).layout_root = layout_create_cell(::core::ptr::null_mut::<layout_cell>());
    lcroot = (*w).layout_root;
    layout_set_size(
        lcroot,
        sx,
        sy,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    layout_make_node(lcroot, LAYOUT_TOPBOTTOM);
    wp = (*w).panes.tqh_first;
    j = 0 as u_int;
    while j < rows {
        while !wp.is_null() && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0 {
            wp = (*wp).entry.tqe_next;
        }
        if wp.is_null() {
            break;
        }
        lcchild = (*wp).layout_cell as *mut layout_cell;
        if n.wrapping_sub(j.wrapping_mul(columns)) == 1 as u_int || columns == 1 as u_int {
            (*lcchild).parent = lcroot;
            (*lcchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
            (*lcchild).entry.tqe_prev = (*lcroot).cells.tqh_last;
            *(*lcroot).cells.tqh_last = lcchild;
            (*lcroot).cells.tqh_last = &raw mut (*lcchild).entry.tqe_next;
            layout_set_size(
                lcchild,
                (*w).sx,
                height,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            wp = (*wp).entry.tqe_next;
        } else {
            lcrow = layout_create_cell(lcroot);
            layout_make_node(lcrow, LAYOUT_LEFTRIGHT);
            layout_set_size(
                lcrow,
                (*w).sx,
                height,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            (*lcrow).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
            (*lcrow).entry.tqe_prev = (*lcroot).cells.tqh_last;
            *(*lcroot).cells.tqh_last = lcrow;
            (*lcroot).cells.tqh_last = &raw mut (*lcrow).entry.tqe_next;
            i = 0 as u_int;
            while i < columns {
                (*lcchild).parent = lcrow;
                (*lcchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
                (*lcchild).entry.tqe_prev = (*lcrow).cells.tqh_last;
                *(*lcrow).cells.tqh_last = lcchild;
                (*lcrow).cells.tqh_last = &raw mut (*lcchild).entry.tqe_next;
                layout_set_size(
                    lcchild,
                    width,
                    height,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                wp = (*wp).entry.tqe_next;
                while !wp.is_null()
                    && layout_cell_is_tiled((*wp).layout_cell as *mut layout_cell) == 0
                {
                    wp = (*wp).entry.tqe_next;
                }
                if wp.is_null() {
                    break;
                }
                lcchild = (*wp).layout_cell as *mut layout_cell;
                i = i.wrapping_add(1);
            }
            if i == columns {
                i = i.wrapping_sub(1);
            }
            used = i
                .wrapping_add(1 as u_int)
                .wrapping_mul(width.wrapping_add(1 as u_int))
                .wrapping_sub(1 as u_int);
            if !((*w).sx <= used) {
                lcchild = *(*((*lcrow).cells.tqh_last as *mut layout_cells)).tqh_last;
                layout_resize_adjust(
                    w,
                    lcchild,
                    LAYOUT_LEFTRIGHT,
                    (*w).sx.wrapping_sub(used) as ::core::ffi::c_int,
                );
            }
        }
        j = j.wrapping_add(1);
    }
    used = rows
        .wrapping_mul(height)
        .wrapping_add(rows)
        .wrapping_sub(1 as u_int);
    if (*w).sy > used {
        lcrow = *(*((*lcroot).cells.tqh_last as *mut layout_cells)).tqh_last;
        layout_resize_adjust(
            w,
            lcrow,
            LAYOUT_TOPBOTTOM,
            (*w).sy.wrapping_sub(used) as ::core::ffi::c_int,
        );
    }
    layout_set_link_floating(w, lcroot);
    layout_fix_offsets(w);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    layout_print_cell(
        (*w).layout_root,
        b"layout_set_tiled\0" as *const u8 as *const ::core::ffi::c_char,
        1 as u_int,
    );
    window_resize(
        w,
        (*lcroot).g.sx,
        (*lcroot).g.sy,
        -(1 as ::core::ffi::c_int),
        -(1 as ::core::ffi::c_int),
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    server_redraw_window(w);
}

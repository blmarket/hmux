use crate::src::shared::client::*;
use crate::src::shared::tty::*;
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
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn tty_attributes(_: *mut tty, _: *const grid_cell, _: *const tty_style_ctx);
    fn tty_region_off(_: *mut tty);
    fn tty_margin_off(_: *mut tty);
    fn tty_cursor(_: *mut tty, _: u_int, _: u_int);
    fn tty_fake_bce(_: *const tty, _: *const grid_cell, _: u_int) -> ::core::ffi::c_int;
    fn tty_repeat_space(_: *mut tty, _: u_int);
    fn tty_putcode(_: *mut tty, _: tty_code_code);
    fn tty_putcode_i(_: *mut tty, _: tty_code_code, _: ::core::ffi::c_int);
    fn tty_putc(_: *mut tty, _: u_char);
    fn tty_putn(_: *mut tty, _: *const ::core::ffi::c_void, _: size_t, _: u_int);
    fn tty_default_attributes(_: *mut tty, _: u_int, _: *const tty_style_ctx);
    fn tty_update_mode(_: *mut tty, _: ::core::ffi::c_int, _: *mut screen);
    fn tty_check_codeset(_: *mut tty, _: *const grid_cell) -> *const grid_cell;
    fn tty_term_has(_: *mut tty_term, _: tty_code_code) -> ::core::ffi::c_int;
    static grid_default_cell: grid_cell;
    fn grid_cells_look_equal(_: *const grid_cell, _: *const grid_cell) -> ::core::ffi::c_int;
    fn grid_get_line(_: *mut grid, _: u_int) -> *mut grid_line;
    fn grid_view_get_cell(_: *mut grid, _: u_int, _: u_int, _: *mut grid_cell);
    fn screen_select_cell(
        _: *mut screen,
        _: *mut grid_cell,
        _: *const grid_cell,
    ) -> ::core::ffi::c_int;
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
pub struct tty_style_ctx {
    pub defaults: *const grid_cell,
    pub palette: *mut colour_palette,
    pub dim: u_int,
    pub hyperlinks: *mut hyperlinks,
}
pub type tty_draw_line_state = ::core::ffi::c_uint;
pub const TTY_DRAW_LINE_DONE: tty_draw_line_state = 6;
pub const TTY_DRAW_LINE_SAME: tty_draw_line_state = 5;
pub const TTY_DRAW_LINE_EMPTY: tty_draw_line_state = 4;
pub const TTY_DRAW_LINE_NEW2: tty_draw_line_state = 3;
pub const TTY_DRAW_LINE_NEW1: tty_draw_line_state = 2;
pub const TTY_DRAW_LINE_FLUSH: tty_draw_line_state = 1;
pub const TTY_DRAW_LINE_FIRST: tty_draw_line_state = 0;
pub const TTY_NOCURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
static mut tty_draw_line_states: [*const ::core::ffi::c_char; 7] = [
    b"FIRST\0" as *const u8 as *const ::core::ffi::c_char,
    b"FLUSH\0" as *const u8 as *const ::core::ffi::c_char,
    b"NEW1\0" as *const u8 as *const ::core::ffi::c_char,
    b"NEW2\0" as *const u8 as *const ::core::ffi::c_char,
    b"EMPTY\0" as *const u8 as *const ::core::ffi::c_char,
    b"SAME\0" as *const u8 as *const ::core::ffi::c_char,
    b"DONE\0" as *const u8 as *const ::core::ffi::c_char,
];
unsafe extern "C" fn tty_draw_line_clear(
    mut tty: *mut tty,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut defaults: *const grid_cell,
    mut bg: u_int,
    mut wrapped: ::core::ffi::c_int,
) {
    if nx == 0 as u_int {
        return;
    }
    if (*(*tty).client).overlay_check.is_none()
        && wrapped == 0
        && nx >= 10 as u_int
        && tty_fake_bce(tty, defaults, bg) == 0
    {
        if px.wrapping_add(nx) >= (*tty).sx && tty_term_has((*tty).term, TTYC_EL) != 0 {
            tty_cursor(tty, px, py);
            tty_putcode(tty, TTYC_EL);
            return;
        }
        if px == 0 as u_int && tty_term_has((*tty).term, TTYC_EL1) != 0 {
            tty_cursor(tty, px.wrapping_add(nx).wrapping_sub(1 as u_int), py);
            tty_putcode(tty, TTYC_EL1);
            return;
        }
        if tty_term_has((*tty).term, TTYC_ECH) != 0 {
            tty_cursor(tty, px, py);
            tty_putcode_i(tty, TTYC_ECH, nx as ::core::ffi::c_int);
            return;
        }
    }
    if px != 0 as u_int || wrapped == 0 {
        tty_cursor(tty, px, py);
    }
    if nx == 1 as u_int {
        tty_putc(tty, ' ' as i32 as u_char);
    } else if nx == 2 as u_int {
        tty_putn(
            tty,
            b"  \0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
            2 as size_t,
            2 as u_int,
        );
    } else {
        tty_repeat_space(tty, nx);
    };
}
unsafe extern "C" fn tty_draw_line_get_empty(
    mut gc: *const grid_cell,
    mut last: *const grid_cell,
    mut nx: u_int,
) -> u_int {
    let mut empty: u_int = 0 as u_int;
    if (*gc).data.width as u_int > nx {
        empty = nx;
    } else if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        empty = 1 as u_int;
    } else if (*gc).data.width as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        empty = 1 as u_int;
    } else if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_SELECTED != 0 {
        empty = 0 as u_int;
    } else if (*gc).bg == (*last).bg
        && (*gc).attr as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        && (*gc).link == 0 as u_int
    {
        if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_CLEARED != 0 {
            empty = 1 as u_int;
        } else if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
            empty = (*gc).data.width as u_int;
        } else if (*gc).data.size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && *(&raw const (*gc).data.data as *const u_char) as ::core::ffi::c_int == ' ' as i32
        {
            empty = 1 as u_int;
        }
    }
    return empty;
}
#[no_mangle]
pub unsafe extern "C" fn tty_draw_line(
    mut tty: *mut tty,
    mut s: *mut screen,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut atx: u_int,
    mut aty: u_int,
    mut style_ctx: *const tty_style_ctx,
) {
    let mut current_block: u64;
    let mut gd: *mut grid = (*s).grid;
    let mut gcp: *const grid_cell = ::core::ptr::null::<grid_cell>();
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
    let mut ngc: grid_cell = grid_cell {
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
    let mut last: grid_cell = grid_cell {
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
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut last_i: u_int = 0;
    let mut cx: u_int = 0;
    let mut ex: u_int = 0;
    let mut width: u_int = 0;
    let mut cellsize: u_int = 0;
    let mut bg: u_int = 0;
    let mut flags: ::core::ffi::c_int = 0;
    let mut empty: ::core::ffi::c_int = 0;
    let mut wrapped: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut buf: [::core::ffi::c_char; 1000] = [0; 1000];
    let mut len: size_t = 0;
    let mut current_state: tty_draw_line_state = TTY_DRAW_LINE_FIRST;
    let mut next_state: tty_draw_line_state = TTY_DRAW_LINE_FIRST;
    let mut default_style_ctx: tty_style_ctx = tty_style_ctx {
        defaults: ::core::ptr::null::<grid_cell>(),
        palette: ::core::ptr::null_mut::<colour_palette>(),
        dim: 0,
        hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
    };
    let mut defaults: *const grid_cell = ::core::ptr::null::<grid_cell>();
    if style_ctx.is_null() {
        default_style_ctx.defaults = &raw const grid_default_cell;
        default_style_ctx.hyperlinks = (*s).hyperlinks;
        style_ctx = &raw mut default_style_ctx;
    }
    defaults = (*style_ctx).defaults;
    log_debug(
        b"%s: px=%u py=%u nx=%u atx=%u aty=%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"tty_draw_line\0" as *const u8 as *const ::core::ffi::c_char,
        px,
        py,
        nx,
        atx,
        aty,
    );
    if atx >= (*tty).sx {
        return;
    }
    if atx.wrapping_add(nx) >= (*tty).sx {
        nx = (*tty).sx.wrapping_sub(atx);
    }
    if nx == 0 as u_int {
        return;
    }
    cellsize = (*grid_get_line(gd, (*gd).hsize.wrapping_add(py))).cellsize as u_int;
    if (*(*s).grid).sx > cellsize {
        ex = cellsize;
    } else {
        ex = (*(*s).grid).sx;
    }
    log_debug(
        b"%s: drawing %u-%u,%u (end %u) at %u,%u; defaults: fg=%d, bg=%d\0" as *const u8
            as *const ::core::ffi::c_char,
        b"tty_draw_line\0" as *const u8 as *const ::core::ffi::c_char,
        px,
        px.wrapping_add(nx),
        py,
        ex,
        atx,
        aty,
        (*defaults).fg,
        (*defaults).bg,
    );
    flags = (*tty).flags & TTY_NOCURSOR;
    (*tty).flags |= TTY_NOCURSOR;
    tty_update_mode(tty, (*tty).mode, s);
    tty_region_off(tty);
    tty_margin_off(tty);
    memcpy(
        &raw mut last as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    last.bg = (*defaults).bg;
    tty_default_attributes(tty, 8 as u_int, style_ctx);
    cx = 0 as u_int;
    i = px;
    while i < px.wrapping_add(nx) {
        grid_view_get_cell(gd, i, py, &raw mut gc);
        if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
            break;
        }
        cx = cx.wrapping_add(1);
        i = i.wrapping_add(1);
    }
    if cx != 0 as u_int {
        i = px.wrapping_add(1 as u_int);
        while i > 0 as u_int {
            grid_view_get_cell(gd, i.wrapping_sub(1 as u_int), py, &raw mut gc);
            if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
                break;
            }
            i = i.wrapping_sub(1);
        }
        if i == 0 as u_int {
            bg = (*defaults).bg as u_int;
        } else {
            bg = gc.bg as u_int;
            if gc.flags as ::core::ffi::c_int & GRID_FLAG_SELECTED != 0 {
                memcpy(
                    &raw mut ngc as *mut ::core::ffi::c_void,
                    &raw mut gc as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<grid_cell>() as size_t,
                );
                if screen_select_cell(s, &raw mut ngc, &raw mut gc) != 0 {
                    bg = ngc.bg as u_int;
                }
            }
        }
        tty_attributes(tty, &raw mut last, style_ctx);
        log_debug(
            b"%s: clearing %u padding cells\0" as *const u8 as *const ::core::ffi::c_char,
            b"tty_draw_line\0" as *const u8 as *const ::core::ffi::c_char,
            cx,
        );
        tty_draw_line_clear(tty, atx, aty, cx, defaults, bg, 0 as ::core::ffi::c_int);
        if cx == ex {
            current_block = 15064833524635049977;
        } else {
            atx = atx.wrapping_add(cx);
            px = px.wrapping_add(cx);
            nx = nx.wrapping_sub(cx);
            current_block = 16799951812150840583;
        }
    } else {
        current_block = 16799951812150840583;
    }
    match current_block {
        16799951812150840583 => {
            if py != 0 as u_int && atx == 0 as u_int && (*tty).cx >= (*tty).sx && nx == (*tty).sx {
                gl = grid_get_line(gd, (*gd).hsize.wrapping_add(py).wrapping_sub(1 as u_int));
                if (*gl).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED != 0 {
                    wrapped = 1 as ::core::ffi::c_int;
                }
            }
            i = 0 as u_int;
            last_i = i;
            len = 0 as size_t;
            width = 0 as u_int;
            current_state = TTY_DRAW_LINE_FIRST;
            loop {
                if i == nx {
                    empty = 0 as ::core::ffi::c_int;
                    next_state = TTY_DRAW_LINE_DONE;
                    gcp = &raw const grid_default_cell;
                } else {
                    if i > nx {
                        fatalx(
                            b"position %u > width %u\0" as *const u8 as *const ::core::ffi::c_char,
                            i,
                            nx,
                        );
                    }
                    if px >= ex || i >= ex.wrapping_sub(px) {
                        empty = nx.wrapping_sub(i) as ::core::ffi::c_int;
                        gcp = &raw const grid_default_cell;
                    } else {
                        grid_view_get_cell(gd, px.wrapping_add(i), py, &raw mut gc);
                        empty =
                            tty_draw_line_get_empty(&raw mut gc, &raw mut last, nx.wrapping_sub(i))
                                as ::core::ffi::c_int;
                        if empty != 0 as ::core::ffi::c_int {
                            gcp = &raw mut gc;
                        } else {
                            gcp = tty_check_codeset(tty, &raw mut gc);
                            if (*gcp).flags as ::core::ffi::c_int & GRID_FLAG_SELECTED != 0 {
                                memcpy(
                                    &raw mut ngc as *mut ::core::ffi::c_void,
                                    gcp as *const ::core::ffi::c_void,
                                    ::core::mem::size_of::<grid_cell>() as size_t,
                                );
                                if screen_select_cell(s, &raw mut ngc, gcp) != 0 {
                                    gcp = &raw mut ngc;
                                }
                            }
                        }
                    }
                    if empty != 0 as ::core::ffi::c_int {
                        next_state = TTY_DRAW_LINE_EMPTY;
                    } else if current_state as ::core::ffi::c_uint
                        == TTY_DRAW_LINE_FIRST as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        next_state = TTY_DRAW_LINE_SAME;
                    } else if grid_cells_look_equal(gcp, &raw mut last) != 0 {
                        if (*gcp).data.size as usize
                            > (::core::mem::size_of::<[::core::ffi::c_char; 1000]>() as usize)
                                .wrapping_sub(len as usize)
                        {
                            next_state = TTY_DRAW_LINE_FLUSH;
                        } else {
                            next_state = TTY_DRAW_LINE_SAME;
                        }
                    } else if current_state as ::core::ffi::c_uint
                        == TTY_DRAW_LINE_NEW1 as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        next_state = TTY_DRAW_LINE_NEW2;
                    } else {
                        next_state = TTY_DRAW_LINE_NEW1;
                    }
                }
                if log_get_level() != 0 as ::core::ffi::c_int {
                    log_debug(
                        b"%s: cell %u empty %u, bg %u; state: current %s, next %s\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"tty_draw_line\0" as *const u8 as *const ::core::ffi::c_char,
                        px.wrapping_add(i),
                        empty,
                        (*gcp).bg,
                        tty_draw_line_states[current_state as usize],
                        tty_draw_line_states[next_state as usize],
                    );
                }
                if next_state as ::core::ffi::c_uint != current_state as ::core::ffi::c_uint {
                    if current_state as ::core::ffi::c_uint
                        == TTY_DRAW_LINE_EMPTY as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        tty_attributes(tty, &raw mut last, style_ctx);
                        tty_draw_line_clear(
                            tty,
                            atx.wrapping_add(last_i),
                            aty,
                            i.wrapping_sub(last_i),
                            defaults,
                            last.bg as u_int,
                            wrapped,
                        );
                        wrapped = 0 as ::core::ffi::c_int;
                    } else if next_state as ::core::ffi::c_uint
                        != TTY_DRAW_LINE_SAME as ::core::ffi::c_int as ::core::ffi::c_uint
                        && len != 0 as size_t
                    {
                        tty_attributes(tty, &raw mut last, style_ctx);
                        if atx.wrapping_add(i).wrapping_sub(width) != 0 as u_int || wrapped == 0 {
                            tty_cursor(tty, atx.wrapping_add(i).wrapping_sub(width), aty);
                        }
                        if !(last.attr as ::core::ffi::c_int) & GRID_ATTR_CHARSET != 0 {
                            tty_putn(
                                tty,
                                &raw mut buf as *mut ::core::ffi::c_char
                                    as *const ::core::ffi::c_void,
                                len,
                                width,
                            );
                        } else {
                            j = 0 as u_int;
                            while (j as size_t) < len {
                                tty_putc(tty, buf[j as usize] as u_char);
                                j = j.wrapping_add(1);
                            }
                        }
                        len = 0 as size_t;
                        width = 0 as u_int;
                        wrapped = 0 as ::core::ffi::c_int;
                    }
                    last_i = i;
                }
                if next_state as ::core::ffi::c_uint
                    != TTY_DRAW_LINE_EMPTY as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    memcpy(
                        (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize)
                            as *mut ::core::ffi::c_void,
                        &raw const (*gcp).data.data as *const u_char as *const ::core::ffi::c_void,
                        (*gcp).data.size as size_t,
                    );
                    len = len.wrapping_add((*gcp).data.size as size_t);
                    width = width.wrapping_add((*gcp).data.width as u_int);
                }
                if next_state as ::core::ffi::c_uint
                    == TTY_DRAW_LINE_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    break;
                }
                current_state = next_state;
                memcpy(
                    &raw mut last as *mut ::core::ffi::c_void,
                    gcp as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<grid_cell>() as size_t,
                );
                if empty != 0 as ::core::ffi::c_int {
                    i = i.wrapping_add(empty as u_int);
                } else {
                    i = i.wrapping_add((*gcp).data.width as u_int);
                }
            }
        }
        _ => {}
    }
    (*tty).flags = (*tty).flags & !TTY_NOCURSOR | flags;
    tty_update_mode(tty, (*tty).mode, s);
}

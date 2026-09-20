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
    pub type hyperlinks;
    pub type screen_write_cline;
    pub type screen_sel;
    pub type screen_titles;
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
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strcspn(
        __s: *const ::core::ffi::c_char,
        __reject: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_ulong;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strsep(
        __stringp: *mut *mut ::core::ffi::c_char,
        __delim: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn tty_term_apply(_: *mut tty_term, _: *const ::core::ffi::c_char, _: ::core::ffi::c_int);
    fn tty_term_has_name(_: *mut tty_term, _: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub offset: u_int,
    pub data: C2RustUnnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
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
    pub entry: C2RustUnnamed_12,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_12 {
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
    pub entry: C2RustUnnamed_13,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_13 {
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
pub struct tty_feature {
    pub name: *const ::core::ffi::c_char,
    pub capabilities: *const *const ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub name: *const ::core::ffi::c_char,
    pub version: u_int,
    pub features: *const ::core::ffi::c_char,
}
pub const TERM_256COLOURS: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const TERM_DECSLRM: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const TERM_DECFRA: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const TERM_RGBCOLOURS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const TERM_SIXEL: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLIENT_UTF8: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
static mut tty_feature_title_capabilities: [*const ::core::ffi::c_char; 3] = [
    b"tsl=\\E]0;\0" as *const u8 as *const ::core::ffi::c_char,
    b"fsl=\\a\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_title: tty_feature = unsafe {
    tty_feature {
        name: b"title\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_title_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_osc7_capabilities: [*const ::core::ffi::c_char; 3] = [
    b"Swd=\\E]7;\0" as *const u8 as *const ::core::ffi::c_char,
    b"fsl=\\a\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_osc7: tty_feature = unsafe {
    tty_feature {
        name: b"osc7\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_osc7_capabilities as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_mouse_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"kmous=\\E[M\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_mouse: tty_feature = unsafe {
    tty_feature {
        name: b"mouse\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_mouse_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_clipboard_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"Ms=\\E]52;%p1%s;%p2%s\\a\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_clipboard: tty_feature = unsafe {
    tty_feature {
        name: b"clipboard\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_clipboard_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_hyperlinks_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"Hls=\\E]8;%?%p1%l%tid=%p1%s%;;%p2%s\\E\\\\\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_hyperlinks: tty_feature = unsafe {
    tty_feature {
        name: b"hyperlinks\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_hyperlinks_capabilities
            as *mut *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_rgb_capabilities: [*const ::core::ffi::c_char; 6] = [
    b"AX\0" as *const u8 as *const ::core::ffi::c_char,
    b"setrgbf=\\E[38;2;%p1%d;%p2%d;%p3%dm\0" as *const u8 as *const ::core::ffi::c_char,
    b"setrgbb=\\E[48;2;%p1%d;%p2%d;%p3%dm\0" as *const u8 as *const ::core::ffi::c_char,
    b"setab=\\E[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0" as *const u8
        as *const ::core::ffi::c_char,
    b"setaf=\\E[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0" as *const u8
        as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_rgb: tty_feature = unsafe {
    tty_feature {
        name: b"RGB\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_rgb_capabilities as *const *const ::core::ffi::c_char,
        flags: TERM_256COLOURS | TERM_RGBCOLOURS,
    }
};
static mut tty_feature_256_capabilities: [*const ::core::ffi::c_char; 4] = [
    b"AX\0" as *const u8 as *const ::core::ffi::c_char,
    b"setab=\\E[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0" as *const u8
        as *const ::core::ffi::c_char,
    b"setaf=\\E[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0" as *const u8
        as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_256: tty_feature = unsafe {
    tty_feature {
        name: b"256\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_256_capabilities as *const *const ::core::ffi::c_char,
        flags: TERM_256COLOURS,
    }
};
static mut tty_feature_overline_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"Smol=\\E[53m\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_overline: tty_feature = unsafe {
    tty_feature {
        name: b"overline\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_overline_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_usstyle_capabilities: [*const ::core::ffi::c_char; 5] = [
    b"Smulx=\\E[4::%p1%dm\0" as *const u8 as *const ::core::ffi::c_char,
    b"Setulc=\\E[58::2::%p1%{65536}%/%d::%p1%{256}%/%{255}%&%d::%p1%{255}%&%d%;m\0" as *const u8
        as *const ::core::ffi::c_char,
    b"Setulc1=\\E[58::5::%p1%dm\0" as *const u8 as *const ::core::ffi::c_char,
    b"ol=\\E[59m\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_usstyle: tty_feature = unsafe {
    tty_feature {
        name: b"usstyle\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_usstyle_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_bpaste_capabilities: [*const ::core::ffi::c_char; 3] = [
    b"Enbp=\\E[?2004h\0" as *const u8 as *const ::core::ffi::c_char,
    b"Dsbp=\\E[?2004l\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_bpaste: tty_feature = unsafe {
    tty_feature {
        name: b"bpaste\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_bpaste_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_focus_capabilities: [*const ::core::ffi::c_char; 3] = [
    b"Enfcs=\\E[?1004h\0" as *const u8 as *const ::core::ffi::c_char,
    b"Dsfcs=\\E[?1004l\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_focus: tty_feature = unsafe {
    tty_feature {
        name: b"focus\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_focus_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_cstyle_capabilities: [*const ::core::ffi::c_char; 3] = [
    b"Ss=\\E[%p1%d q\0" as *const u8 as *const ::core::ffi::c_char,
    b"Se=\\E[2 q\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_cstyle: tty_feature = unsafe {
    tty_feature {
        name: b"cstyle\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_cstyle_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_ccolour_capabilities: [*const ::core::ffi::c_char; 3] = [
    b"Cs=\\E]12;%p1%s\\a\0" as *const u8 as *const ::core::ffi::c_char,
    b"Cr=\\E]112\\a\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_ccolour: tty_feature = unsafe {
    tty_feature {
        name: b"ccolour\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_ccolour_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_strikethrough_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"smxx=\\E[9m\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_strikethrough: tty_feature = unsafe {
    tty_feature {
        name: b"strikethrough\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_strikethrough_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_sync_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"Sync=\\E[?2026%?%p1%{1}%-%tl%eh%;\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_sync: tty_feature = unsafe {
    tty_feature {
        name: b"sync\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_sync_capabilities as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_extkeys_capabilities: [*const ::core::ffi::c_char; 3] = [
    b"Eneks=\\E[>4;2m\0" as *const u8 as *const ::core::ffi::c_char,
    b"Dseks=\\E[>4m\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_extkeys: tty_feature = unsafe {
    tty_feature {
        name: b"extkeys\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_extkeys_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_margins_capabilities: [*const ::core::ffi::c_char; 5] = [
    b"Enmg=\\E[?69h\0" as *const u8 as *const ::core::ffi::c_char,
    b"Dsmg=\\E[?69l\0" as *const u8 as *const ::core::ffi::c_char,
    b"Clmg=\\E[s\0" as *const u8 as *const ::core::ffi::c_char,
    b"Cmg=\\E[%i%p1%d;%p2%ds\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_margins: tty_feature = unsafe {
    tty_feature {
        name: b"margins\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_margins_capabilities
            as *const *const ::core::ffi::c_char,
        flags: TERM_DECSLRM,
    }
};
static mut tty_feature_rectfill_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"Rect\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_rectfill: tty_feature = unsafe {
    tty_feature {
        name: b"rectfill\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_rectfill_capabilities
            as *const *const ::core::ffi::c_char,
        flags: TERM_DECFRA,
    }
};
static mut tty_feature_ignorefkeys_capabilities: [*const ::core::ffi::c_char; 65] = [
    b"kf0@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf1@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf2@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf3@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf4@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf5@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf6@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf7@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf8@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf9@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf10@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf11@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf12@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf13@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf14@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf15@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf16@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf17@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf18@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf19@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf20@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf21@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf22@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf23@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf24@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf25@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf26@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf27@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf28@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf29@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf30@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf31@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf32@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf33@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf34@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf35@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf36@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf37@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf38@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf39@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf40@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf41@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf42@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf43@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf44@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf45@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf46@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf47@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf48@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf49@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf50@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf51@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf52@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf53@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf54@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf55@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf56@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf57@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf58@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf59@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf60@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf61@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf62@\0" as *const u8 as *const ::core::ffi::c_char,
    b"kf63@\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_ignorefkeys: tty_feature = unsafe {
    tty_feature {
        name: b"ignorefkeys\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_ignorefkeys_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_sixel_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"Sxl\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_sixel: tty_feature = unsafe {
    tty_feature {
        name: b"sixel\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_sixel_capabilities
            as *const *const ::core::ffi::c_char,
        flags: TERM_SIXEL,
    }
};
static mut tty_feature_progressbar_capabilities: [*const ::core::ffi::c_char; 2] = [
    b"Spb=\\E]9;4;%p1%d;%p2%d\\E\\\\\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
static mut tty_feature_progressbar: tty_feature = unsafe {
    tty_feature {
        name: b"progressbar\0" as *const u8 as *const ::core::ffi::c_char,
        capabilities: &raw const tty_feature_progressbar_capabilities
            as *const *const ::core::ffi::c_char,
        flags: 0 as ::core::ffi::c_int,
    }
};
static mut tty_feature_utf8: tty_feature = tty_feature {
    name: b"utf8\0" as *const u8 as *const ::core::ffi::c_char,
    capabilities: ::core::ptr::null::<*const ::core::ffi::c_char>(),
    flags: 0 as ::core::ffi::c_int,
};
static mut tty_features: [*const tty_feature; 22] = unsafe {
    [
        &raw const tty_feature_256,
        &raw const tty_feature_bpaste,
        &raw const tty_feature_ccolour,
        &raw const tty_feature_clipboard,
        &raw const tty_feature_hyperlinks,
        &raw const tty_feature_cstyle,
        &raw const tty_feature_extkeys,
        &raw const tty_feature_focus,
        &raw const tty_feature_ignorefkeys,
        &raw const tty_feature_margins,
        &raw const tty_feature_mouse,
        &raw const tty_feature_osc7,
        &raw const tty_feature_overline,
        &raw const tty_feature_progressbar,
        &raw const tty_feature_rectfill,
        &raw const tty_feature_rgb,
        &raw const tty_feature_sixel,
        &raw const tty_feature_strikethrough,
        &raw const tty_feature_sync,
        &raw const tty_feature_title,
        &raw const tty_feature_usstyle,
        &raw const tty_feature_utf8,
    ]
};
#[no_mangle]
pub unsafe extern "C" fn tty_parse_client_features(
    mut c: *mut client,
    mut s: *const ::core::ffi::c_char,
    mut sep: *const ::core::ffi::c_char,
) {
    tty_parse_features(
        s,
        sep,
        &raw mut (*c).term_features,
        &raw mut (*c).term_nofeatures,
    );
}
#[no_mangle]
pub unsafe extern "C" fn tty_parse_features(
    mut s: *const ::core::ffi::c_char,
    mut sep: *const ::core::ffi::c_char,
    mut enabled: *mut ::core::ffi::c_int,
    mut disabled: *mut ::core::ffi::c_int,
) {
    let mut tf: *const tty_feature = ::core::ptr::null::<tty_feature>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut loop_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut remove: ::core::ffi::c_int = 0;
    log_debug(
        b"adding terminal features %s\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    copy = xstrdup(s);
    loop_0 = copy;
    loop {
        next = strsep(&raw mut loop_0, sep);
        if next.is_null() {
            break;
        }
        remove = (*next as ::core::ffi::c_int != '\0' as i32
            && *next.offset(strlen(next).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                == '@' as i32) as ::core::ffi::c_int;
        if remove != 0 {
            *next.offset(strlen(next).wrapping_sub(1 as size_t) as isize) =
                '\0' as i32 as ::core::ffi::c_char;
        }
        i = 0 as u_int;
        while (i as usize)
            < (::core::mem::size_of::<[*const tty_feature; 22]>() as usize)
                .wrapping_div(::core::mem::size_of::<*const tty_feature>() as usize)
        {
            tf = tty_features[i as usize];
            if strcasecmp((*tf).name, next) == 0 as ::core::ffi::c_int {
                break;
            }
            i = i.wrapping_add(1);
        }
        if i as usize
            == (::core::mem::size_of::<[*const tty_feature; 22]>() as usize)
                .wrapping_div(::core::mem::size_of::<*const tty_feature>() as usize)
        {
            log_debug(
                b"unknown terminal feature: %s\0" as *const u8 as *const ::core::ffi::c_char,
                next,
            );
            break;
        } else if remove != 0 {
            log_debug(
                b"removing terminal feature: %s\0" as *const u8 as *const ::core::ffi::c_char,
                (*tf).name,
            );
            *enabled &= !((1 as ::core::ffi::c_int) << i);
            if !disabled.is_null() {
                *disabled |= (1 as ::core::ffi::c_int) << i;
            }
        } else {
            if !disabled.is_null() && *disabled & (1 as ::core::ffi::c_int) << i != 0 {
                continue;
            }
            if !*enabled & (1 as ::core::ffi::c_int) << i != 0 {
                log_debug(
                    b"adding terminal feature: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*tf).name,
                );
                *enabled |= (1 as ::core::ffi::c_int) << i;
            }
        }
    }
    free(copy as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn tty_get_features(
    mut feat: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut tf: *const tty_feature = ::core::ptr::null::<tty_feature>();
    static mut s: [::core::ffi::c_char; 512] = [0; 512];
    let mut i: u_int = 0;
    *(&raw mut s as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[*const tty_feature; 22]>() as usize)
            .wrapping_div(::core::mem::size_of::<*const tty_feature>() as usize)
    {
        if !(!feat & (1 as ::core::ffi::c_int) << i != 0) {
            tf = tty_features[i as usize];
            strlcat(
                &raw mut s as *mut ::core::ffi::c_char,
                (*tf).name,
                ::core::mem::size_of::<[::core::ffi::c_char; 512]>() as size_t,
            );
            strlcat(
                &raw mut s as *mut ::core::ffi::c_char,
                b",\0" as *const u8 as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 512]>() as size_t,
            );
        }
        i = i.wrapping_add(1);
    }
    if *(&raw mut s as *mut ::core::ffi::c_char) as ::core::ffi::c_int != '\0' as i32 {
        s[strlen(&raw mut s as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
            '\0' as i32 as ::core::ffi::c_char;
    }
    return &raw mut s as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn tty_feature_present(
    mut term: *mut tty_term,
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut tf: *const tty_feature = ::core::ptr::null::<tty_feature>();
    let mut capability: *const *const ::core::ffi::c_char =
        ::core::ptr::null::<*const ::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if strcmp(name, b"utf8\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        return ((*(*(*term).tty).client).flags & CLIENT_UTF8 as uint64_t != 0 as uint64_t)
            as ::core::ffi::c_int;
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[*const tty_feature; 22]>() as usize)
            .wrapping_div(::core::mem::size_of::<*const tty_feature>() as usize)
    {
        tf = tty_features[i as usize];
        if strcmp((*tf).name, name) == 0 as ::core::ffi::c_int {
            if (*term).applied_features & (1 as ::core::ffi::c_int) << i != 0 {
                return 1 as ::core::ffi::c_int;
            }
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if tf.is_null()
        || (*tf).capabilities.is_null()
        || strcmp(
            name,
            b"ignorefkeys\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if (*tf).flags != 0 as ::core::ffi::c_int && (*term).flags & (*tf).flags != (*tf).flags {
        return 0 as ::core::ffi::c_int;
    }
    capability = (*tf).capabilities;
    while !(*capability).is_null() {
        copy = xstrdup(*capability);
        *copy.offset(strcspn(copy, b"=\0" as *const u8 as *const ::core::ffi::c_char) as isize) =
            '\0' as i32 as ::core::ffi::c_char;
        if tty_term_has_name(term, copy) == 0 {
            free(copy as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        }
        free(copy as *mut ::core::ffi::c_void);
        capability = capability.offset(1);
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tty_apply_features(mut term: *mut tty_term) -> ::core::ffi::c_int {
    let mut c: *mut client = (*(*term).tty).client;
    let mut tf: *const tty_feature = ::core::ptr::null::<tty_feature>();
    let mut capability: *const *const ::core::ffi::c_char =
        ::core::ptr::null::<*const ::core::ffi::c_char>();
    let mut feat: ::core::ffi::c_int = 0;
    let mut i: u_int = 0;
    feat = (*c).term_features & !(*c).term_nofeatures;
    if feat == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"applying terminal features: %s\0" as *const u8 as *const ::core::ffi::c_char,
        tty_get_features(feat),
    );
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[*const tty_feature; 22]>() as usize)
            .wrapping_div(::core::mem::size_of::<*const tty_feature>() as usize)
    {
        if !((*term).applied_features & (1 as ::core::ffi::c_int) << i != 0
            || !feat & (1 as ::core::ffi::c_int) << i != 0)
        {
            tf = tty_features[i as usize];
            log_debug(
                b"applying terminal feature: %s\0" as *const u8 as *const ::core::ffi::c_char,
                (*tf).name,
            );
            if !(*tf).capabilities.is_null() {
                capability = (*tf).capabilities;
                while !(*capability).is_null() {
                    log_debug(
                        b"adding capability: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        *capability,
                    );
                    tty_term_apply(term, *capability, 1 as ::core::ffi::c_int);
                    capability = capability.offset(1);
                }
            }
            (*term).flags |= (*tf).flags;
            if tf == &raw const tty_feature_utf8 {
                (*c).flags |= CLIENT_UTF8 as uint64_t;
            }
        }
        i = i.wrapping_add(1);
    }
    if (*term).applied_features | feat == (*term).applied_features {
        return 0 as ::core::ffi::c_int;
    }
    (*term).applied_features |= feat;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tty_default_features(
    mut c: *mut client,
    mut name: *const ::core::ffi::c_char,
    mut version: u_int,
) {
    static mut table: [C2RustUnnamed_35; 9] = [
        C2RustUnnamed_35 {
            name: b"mintty\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,margins,overline,usstyle\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_35 {
            name: b"tmux\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,focus,overline,usstyle,hyperlinks,progressbar\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_35 {
            name: b"rxvt-unicode\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,bpaste,ccolour,cstyle,mouse,title,ignorefkeys\0" as *const u8
                as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_35 {
            name: b"iTerm2\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,RGB,bpaste,clipboard,mouse,strikethrough,title,cstyle,extkeys,margins,usstyle,sync,osc7,hyperlinks,progressbar\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_35 {
            name: b"foot\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,usstyle,sync,osc7,hyperlinks\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_35 {
            name: b"WezTerm\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,focus,hyperlinks,usstyle\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_35 {
            name: b"ghostty\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,focus,overline,hyperlinks,osc7,sync,usstyle,progressbar\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_35 {
            name: b"Rio\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,focus,overline,hyperlinks,osc7,sync,usstyle,progressbar\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
        C2RustUnnamed_35 {
            name: b"XTerm\0" as *const u8 as *const ::core::ffi::c_char,
            version: 0,
            features: b"256,RGB,bpaste,clipboard,mouse,strikethrough,title,ccolour,cstyle,extkeys,focus\0"
                as *const u8 as *const ::core::ffi::c_char,
        },
    ];
    let mut i: u_int = 0;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_35; 9]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_35>() as usize)
    {
        if !(strcmp(table[i as usize].name, name) != 0 as ::core::ffi::c_int) {
            if !(version != 0 as u_int && version < table[i as usize].version) {
                tty_parse_client_features(
                    c,
                    table[i as usize].features,
                    b",\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        i = i.wrapping_add(1);
    }
}

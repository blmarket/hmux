use crate::src::shared::client::*;
use crate::src::shared::sort::*;
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
    fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn paste_walk(_: *mut paste_buffer) -> *mut paste_buffer;
    fn key_bindings_first_table() -> *mut key_table;
    fn key_bindings_next_table(_: *mut key_table) -> *mut key_table;
    fn key_bindings_first(_: *mut key_table) -> *mut key_binding;
    fn key_bindings_next(_: *mut key_table, _: *mut key_binding) -> *mut key_binding;
    static mut clients: clients;
    fn winlinks_RB_NEXT(_: *mut winlink) -> *mut winlink;
    fn winlinks_RB_MINMAX(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn window_pane_index(_: *mut window_pane, _: *mut u_int) -> ::core::ffi::c_int;
    fn window_pane_zindex(_: *mut window_pane, _: *mut u_int) -> ::core::ffi::c_int;
    static mut sessions: sessions;
    fn sessions_RB_NEXT(_: *mut session) -> *mut session;
    fn sessions_RB_MINMAX(_: *mut sessions, _: ::core::ffi::c_int) -> *mut session;
}
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
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
pub struct sessions {
    pub rbh_root: *mut session,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct clients {
    pub tqh_first: *mut client,
    pub tqh_last: *mut *mut client,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct paste_buffer {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
    pub name: *mut ::core::ffi::c_char,
    pub created: time_t,
    pub automatic: ::core::ffi::c_int,
    pub order: u_int,
    pub name_entry: C2RustUnnamed_36,
    pub time_entry: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub rbe_left: *mut paste_buffer,
    pub rbe_right: *mut paste_buffer,
    pub rbe_parent: *mut paste_buffer,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_36 {
    pub rbe_left: *mut paste_buffer,
    pub rbe_right: *mut paste_buffer,
    pub rbe_parent: *mut paste_buffer,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sort_criteria {
    pub order: sort_order,
    pub reversed: ::core::ffi::c_int,
    pub order_seq: *mut sort_order,
}
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const CLIENT_EXIT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CLIENT_SUSPENDED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLIENT_ATTACHED: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const CLIENT_DEAD: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const CLIENT_UNATTACHEDFLAGS: ::core::ffi::c_int = CLIENT_DEAD | CLIENT_SUSPENDED | CLIENT_EXIT;
static mut sort_criteria: *mut sort_criteria =
    ::core::ptr::null::<sort_criteria>() as *mut sort_criteria;
unsafe extern "C" fn sort_qsort(
    mut l: *mut ::core::ffi::c_void,
    mut len: u_int,
    mut size: u_int,
    mut cmp: Option<
        unsafe extern "C" fn(
            *const ::core::ffi::c_void,
            *const ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    mut sort_crit: *mut sort_criteria,
) {
    let mut i: u_int = 0;
    let mut tmp: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut ll: *mut *mut ::core::ffi::c_void = ::core::ptr::null_mut::<*mut ::core::ffi::c_void>();
    if len < 2 as u_int
        || (*sort_crit).order as ::core::ffi::c_uint
            == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    if (*sort_crit).order as ::core::ffi::c_uint
        == SORT_ORDER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*sort_crit).reversed != 0 {
            ll = l as *mut *mut ::core::ffi::c_void;
            i = 0 as u_int;
            while i < len.wrapping_div(2 as u_int) {
                tmp = *ll.offset(i as isize);
                let ref mut fresh2 = *ll.offset(i as isize);
                *fresh2 = *ll.offset(len.wrapping_sub(1 as u_int).wrapping_sub(i) as isize);
                let ref mut fresh3 =
                    *ll.offset(len.wrapping_sub(1 as u_int).wrapping_sub(i) as isize);
                *fresh3 = tmp;
                i = i.wrapping_add(1);
            }
        }
    } else {
        sort_criteria = sort_crit;
        qsort(l, len as size_t, size as size_t, cmp as __compar_fn_t);
    };
}
unsafe extern "C" fn sort_buffer_cmp(
    mut a0: *const ::core::ffi::c_void,
    mut b0: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut sort_crit: *mut sort_criteria = sort_criteria;
    let mut a: *const *const paste_buffer = a0 as *const *const paste_buffer;
    let mut b: *const *const paste_buffer = b0 as *const *const paste_buffer;
    let mut pa: *const paste_buffer = *a;
    let mut pb: *const paste_buffer = *b;
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match (*sort_crit).order as ::core::ffi::c_uint {
        4 => {
            result = strcmp((*pa).name, (*pb).name);
        }
        1 => {
            if (*pa).order > (*pb).order {
                result = -(1 as ::core::ffi::c_int);
            } else if (*pa).order < (*pb).order {
                result = 1 as ::core::ffi::c_int;
            } else {
                result = 0 as ::core::ffi::c_int;
            }
        }
        6 => {
            result = (*pa).size.wrapping_sub((*pb).size) as ::core::ffi::c_int;
        }
        0 | 2 | 3 | 5 | 7 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = strcmp((*pa).name, (*pb).name);
    }
    if (*sort_crit).reversed != 0 {
        result = -result;
    }
    return result;
}
unsafe extern "C" fn sort_client_cmp(
    mut a0: *const ::core::ffi::c_void,
    mut b0: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut sort_crit: *mut sort_criteria = sort_criteria;
    let mut a: *const *const client = a0 as *const *const client;
    let mut b: *const *const client = b0 as *const *const client;
    let mut ca: *const client = *a;
    let mut cb: *const client = *b;
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match (*sort_crit).order as ::core::ffi::c_uint {
        4 => {
            result = strcmp((*ca).name, (*cb).name);
        }
        6 => {
            result = (*ca).tty.sx.wrapping_sub((*cb).tty.sx) as ::core::ffi::c_int;
            if result == 0 as ::core::ffi::c_int {
                result = (*ca).tty.sy.wrapping_sub((*cb).tty.sy) as ::core::ffi::c_int;
            }
        }
        1 => {
            if if (*ca).creation_time.tv_sec == (*cb).creation_time.tv_sec {
                ((*ca).creation_time.tv_usec > (*cb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*ca).creation_time.tv_sec > (*cb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            } else if if (*ca).creation_time.tv_sec == (*cb).creation_time.tv_sec {
                ((*ca).creation_time.tv_usec < (*cb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*ca).creation_time.tv_sec < (*cb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            }
        }
        0 => {
            if if (*ca).activity_time.tv_sec == (*cb).activity_time.tv_sec {
                ((*ca).activity_time.tv_usec > (*cb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*ca).activity_time.tv_sec > (*cb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            } else if if (*ca).activity_time.tv_sec == (*cb).activity_time.tv_sec {
                ((*ca).activity_time.tv_usec < (*cb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*ca).activity_time.tv_sec < (*cb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            }
        }
        2 | 3 | 5 | 7 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = strcmp((*ca).name, (*cb).name);
    }
    if (*sort_crit).reversed != 0 {
        result = -result;
    }
    return result;
}
unsafe extern "C" fn sort_session_cmp(
    mut a0: *const ::core::ffi::c_void,
    mut b0: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut sort_crit: *mut sort_criteria = sort_criteria;
    let mut a: *const *const session = a0 as *const *const session;
    let mut b: *const *const session = b0 as *const *const session;
    let mut sa: *const session = *a;
    let mut sb: *const session = *b;
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match (*sort_crit).order as ::core::ffi::c_uint {
        2 => {
            result = (*sa).id.wrapping_sub((*sb).id) as ::core::ffi::c_int;
        }
        1 => {
            if if (*sa).creation_time.tv_sec == (*sb).creation_time.tv_sec {
                ((*sa).creation_time.tv_usec > (*sb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*sa).creation_time.tv_sec > (*sb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            } else if if (*sa).creation_time.tv_sec == (*sb).creation_time.tv_sec {
                ((*sa).creation_time.tv_usec < (*sb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*sa).creation_time.tv_sec < (*sb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            }
        }
        0 => {
            if if (*sa).activity_time.tv_sec == (*sb).activity_time.tv_sec {
                ((*sa).activity_time.tv_usec > (*sb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*sa).activity_time.tv_sec > (*sb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            } else if if (*sa).activity_time.tv_sec == (*sb).activity_time.tv_sec {
                ((*sa).activity_time.tv_usec < (*sb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*sa).activity_time.tv_sec < (*sb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            }
        }
        4 => {
            result = strcmp((*sa).name, (*sb).name);
        }
        3 | 5 | 6 | 7 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = strcmp((*sa).name, (*sb).name);
    }
    if (*sort_crit).reversed != 0 {
        result = -result;
    }
    return result;
}
unsafe extern "C" fn sort_pane_cmp(
    mut a0: *const ::core::ffi::c_void,
    mut b0: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut sort_crit: *mut sort_criteria = sort_criteria;
    let mut a: *mut window_pane = *(a0 as *mut *mut window_pane);
    let mut b: *mut window_pane = *(b0 as *mut *mut window_pane);
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ai: u_int = 0;
    let mut bi: u_int = 0;
    match (*sort_crit).order as ::core::ffi::c_uint {
        0 => {
            result = (*a).active_point.wrapping_sub((*b).active_point) as ::core::ffi::c_int;
        }
        1 => {
            result = (*a).id.wrapping_sub((*b).id) as ::core::ffi::c_int;
        }
        6 => {
            result = (*a)
                .sx
                .wrapping_mul((*a).sy)
                .wrapping_sub((*b).sx.wrapping_mul((*b).sy))
                as ::core::ffi::c_int;
        }
        2 => {
            window_pane_index(a, &raw mut ai);
            window_pane_index(b, &raw mut bi);
            result = ai.wrapping_sub(bi) as ::core::ffi::c_int;
        }
        4 => {
            result = strcmp((*(*a).screen).title, (*(*b).screen).title);
        }
        7 => {
            window_pane_zindex(a, &raw mut ai);
            window_pane_zindex(b, &raw mut bi);
            result = ai.wrapping_sub(bi) as ::core::ffi::c_int;
        }
        3 | 5 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = strcmp((*(*a).screen).title, (*(*b).screen).title);
    }
    if (*sort_crit).reversed != 0 {
        result = -result;
    }
    return result;
}
unsafe extern "C" fn sort_winlink_cmp(
    mut a0: *const ::core::ffi::c_void,
    mut b0: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut sort_crit: *mut sort_criteria = sort_criteria;
    let mut a: *const *const winlink = a0 as *const *const winlink;
    let mut b: *const *const winlink = b0 as *const *const winlink;
    let mut wla: *const winlink = *a;
    let mut wlb: *const winlink = *b;
    let mut wa: *mut window = (*wla).window;
    let mut wb: *mut window = (*wlb).window;
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match (*sort_crit).order as ::core::ffi::c_uint {
        2 => {
            result = (*wla).idx - (*wlb).idx;
        }
        1 => {
            if if (*wa).creation_time.tv_sec == (*wb).creation_time.tv_sec {
                ((*wa).creation_time.tv_usec > (*wb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*wa).creation_time.tv_sec > (*wb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            } else if if (*wa).creation_time.tv_sec == (*wb).creation_time.tv_sec {
                ((*wa).creation_time.tv_usec < (*wb).creation_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*wa).creation_time.tv_sec < (*wb).creation_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            }
        }
        0 => {
            if if (*wa).activity_time.tv_sec == (*wb).activity_time.tv_sec {
                ((*wa).activity_time.tv_usec > (*wb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*wa).activity_time.tv_sec > (*wb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = -(1 as ::core::ffi::c_int);
            } else if if (*wa).activity_time.tv_sec == (*wb).activity_time.tv_sec {
                ((*wa).activity_time.tv_usec < (*wb).activity_time.tv_usec) as ::core::ffi::c_int
            } else {
                ((*wa).activity_time.tv_sec < (*wb).activity_time.tv_sec) as ::core::ffi::c_int
            } != 0
            {
                result = 1 as ::core::ffi::c_int;
            }
        }
        4 => {
            result = strcmp((*wa).name, (*wb).name);
        }
        6 => {
            result = (*wa)
                .sx
                .wrapping_mul((*wa).sy)
                .wrapping_sub((*wb).sx.wrapping_mul((*wb).sy))
                as ::core::ffi::c_int;
        }
        3 | 5 | 7 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = strcmp((*wa).name, (*wb).name);
    }
    if (*sort_crit).reversed != 0 {
        result = -result;
    }
    return result;
}
unsafe extern "C" fn sort_key_binding_cmp(
    mut a0: *const ::core::ffi::c_void,
    mut b0: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut sort_crit: *mut sort_criteria = sort_criteria;
    let mut a: *const key_binding = *(a0 as *mut *mut key_binding);
    let mut b: *const key_binding = *(b0 as *mut *mut key_binding);
    let mut result: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match (*sort_crit).order as ::core::ffi::c_uint {
        2 => {
            result = (*a).key.wrapping_sub((*b).key) as ::core::ffi::c_int;
        }
        3 => {
            result = ((*a).key as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS)
                .wrapping_sub((*b).key as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS)
                as ::core::ffi::c_int;
        }
        4 => {
            result = (strcasecmp((*a).tablename, (*b).tablename) == 0 as ::core::ffi::c_int)
                as ::core::ffi::c_int;
        }
        0 | 1 | 5 | 6 | 7 | 8 | _ => {}
    }
    if result == 0 as ::core::ffi::c_int {
        result = (strcasecmp((*a).tablename, (*b).tablename) == 0 as ::core::ffi::c_int)
            as ::core::ffi::c_int;
    }
    if (*sort_crit).reversed != 0 {
        result = -result;
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn sort_next_order(mut sort_crit: *mut sort_criteria) {
    let mut i: u_int = 0;
    if (*sort_crit).order_seq.is_null() {
        return;
    }
    i = 0 as u_int;
    while *(*sort_crit).order_seq.offset(i as isize) as ::core::ffi::c_uint
        != SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*sort_crit).order as ::core::ffi::c_uint
            == *(*sort_crit).order_seq.offset(i as isize) as ::core::ffi::c_uint
        {
            break;
        }
        i = i.wrapping_add(1);
    }
    if *(*sort_crit).order_seq.offset(i as isize) as ::core::ffi::c_uint
        == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        i = 0 as u_int;
    } else {
        i = i.wrapping_add(1);
        if *(*sort_crit).order_seq.offset(i as isize) as ::core::ffi::c_uint
            == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            i = 0 as u_int;
        }
    }
    (*sort_crit).order = *(*sort_crit).order_seq.offset(i as isize);
}
#[no_mangle]
pub unsafe extern "C" fn sort_order_from_string(
    mut order: *const ::core::ffi::c_char,
) -> sort_order {
    if !order.is_null() {
        if strcasecmp(
            order,
            b"activity\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            return SORT_ACTIVITY;
        }
        if strcasecmp(
            order,
            b"creation\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            return SORT_CREATION;
        }
        if strcasecmp(order, b"index\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
            || strcasecmp(order, b"key\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
        {
            return SORT_INDEX;
        }
        if strcasecmp(
            order,
            b"modifier\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            return SORT_MODIFIER;
        }
        if strcasecmp(order, b"name\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
            || strcasecmp(order, b"title\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
        {
            return SORT_NAME;
        }
        if strcasecmp(order, b"order\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            return SORT_ORDER;
        }
        if strcasecmp(order, b"size\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            return SORT_SIZE;
        }
        if strcasecmp(order, b"z\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            return SORT_Z;
        }
    }
    return SORT_END;
}
#[no_mangle]
pub unsafe extern "C" fn sort_order_to_string(mut order: sort_order) -> *const ::core::ffi::c_char {
    if order as ::core::ffi::c_uint == SORT_ACTIVITY as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"activity\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_CREATION as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"creation\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_INDEX as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"index\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_MODIFIER as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"modifier\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_NAME as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"name\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_ORDER as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"order\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_SIZE as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"size\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if order as ::core::ffi::c_uint == SORT_Z as ::core::ffi::c_int as ::core::ffi::c_uint {
        return b"z\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn sort_would_window_tree_swap(
    mut sort_crit: *mut sort_criteria,
    mut wla: *mut winlink,
    mut wlb: *mut winlink,
) -> ::core::ffi::c_int {
    if (*sort_crit).order as ::core::ffi::c_uint
        == SORT_INDEX as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    sort_criteria = sort_crit;
    return (sort_winlink_cmp(
        &raw mut wla as *const ::core::ffi::c_void,
        &raw mut wlb as *const ::core::ffi::c_void,
    ) != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn sort_get_buffers(
    mut n: *mut u_int,
    mut sort_crit: *mut sort_criteria,
) -> *mut *mut paste_buffer {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut i: u_int = 0;
    static mut l: *mut *mut paste_buffer =
        ::core::ptr::null::<*mut paste_buffer>() as *mut *mut paste_buffer;
    static mut lsz: u_int = 0 as u_int;
    i = 0 as u_int;
    loop {
        pb = paste_walk(pb);
        if pb.is_null() {
            break;
        }
        if lsz <= i {
            lsz = lsz.wrapping_add(100 as u_int);
            l = xreallocarray(
                l as *mut ::core::ffi::c_void,
                lsz as size_t,
                ::core::mem::size_of::<*mut paste_buffer>() as size_t,
            ) as *mut *mut paste_buffer;
        }
        let fresh0 = i;
        i = i.wrapping_add(1);
        let ref mut fresh1 = *l.offset(fresh0 as isize);
        *fresh1 = pb;
    }
    sort_qsort(
        l as *mut ::core::ffi::c_void,
        i,
        ::core::mem::size_of::<*mut paste_buffer>() as u_int,
        Some(
            sort_buffer_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    *n = i;
    return l;
}
#[no_mangle]
pub unsafe extern "C" fn sort_get_clients(
    mut n: *mut u_int,
    mut sort_crit: *mut sort_criteria,
) -> *mut *mut client {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut i: u_int = 0;
    static mut l: *mut *mut client = ::core::ptr::null::<*mut client>() as *mut *mut client;
    static mut lsz: u_int = 0 as u_int;
    i = 0 as u_int;
    c = clients.tqh_first;
    while !c.is_null() {
        if !((*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            if !(!(*c).flags & CLIENT_ATTACHED as uint64_t != 0) {
                if lsz <= i {
                    lsz = lsz.wrapping_add(100 as u_int);
                    l = xreallocarray(
                        l as *mut ::core::ffi::c_void,
                        lsz as size_t,
                        ::core::mem::size_of::<*mut client>() as size_t,
                    ) as *mut *mut client;
                }
                let fresh4 = i;
                i = i.wrapping_add(1);
                let ref mut fresh5 = *l.offset(fresh4 as isize);
                *fresh5 = c;
            }
        }
        c = (*c).entry.tqe_next;
    }
    sort_qsort(
        l as *mut ::core::ffi::c_void,
        i,
        ::core::mem::size_of::<*mut client>() as u_int,
        Some(
            sort_client_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    *n = i;
    return l;
}
#[no_mangle]
pub unsafe extern "C" fn sort_get_sessions(
    mut n: *mut u_int,
    mut sort_crit: *mut sort_criteria,
) -> *mut *mut session {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut i: u_int = 0;
    static mut l: *mut *mut session = ::core::ptr::null::<*mut session>() as *mut *mut session;
    static mut lsz: u_int = 0 as u_int;
    i = 0 as u_int;
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        if lsz <= i {
            lsz = lsz.wrapping_add(100 as u_int);
            l = xreallocarray(
                l as *mut ::core::ffi::c_void,
                lsz as size_t,
                ::core::mem::size_of::<*mut session>() as size_t,
            ) as *mut *mut session;
        }
        let fresh6 = i;
        i = i.wrapping_add(1);
        let ref mut fresh7 = *l.offset(fresh6 as isize);
        *fresh7 = s;
        s = sessions_RB_NEXT(s);
    }
    sort_qsort(
        l as *mut ::core::ffi::c_void,
        i,
        ::core::mem::size_of::<*mut session>() as u_int,
        Some(
            sort_session_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    *n = i;
    return l;
}
#[no_mangle]
pub unsafe extern "C" fn sort_get_panes(
    mut n: *mut u_int,
    mut sort_crit: *mut sort_criteria,
) -> *mut *mut window_pane {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut i: u_int = 0;
    static mut l: *mut *mut window_pane =
        ::core::ptr::null::<*mut window_pane>() as *mut *mut window_pane;
    static mut lsz: u_int = 0 as u_int;
    i = 0 as u_int;
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
        while !wl.is_null() {
            w = (*wl).window;
            wp = (*w).panes.tqh_first;
            while !wp.is_null() {
                if lsz <= i {
                    lsz = lsz.wrapping_add(100 as u_int);
                    l = xreallocarray(
                        l as *mut ::core::ffi::c_void,
                        lsz as size_t,
                        ::core::mem::size_of::<*mut window_pane>() as size_t,
                    ) as *mut *mut window_pane;
                }
                let fresh8 = i;
                i = i.wrapping_add(1);
                let ref mut fresh9 = *l.offset(fresh8 as isize);
                *fresh9 = wp;
                wp = (*wp).entry.tqe_next;
            }
            wl = winlinks_RB_NEXT(wl);
        }
        s = sessions_RB_NEXT(s);
    }
    sort_qsort(
        l as *mut ::core::ffi::c_void,
        i,
        ::core::mem::size_of::<*mut window_pane>() as u_int,
        Some(
            sort_pane_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    *n = i;
    return l;
}
#[no_mangle]
pub unsafe extern "C" fn sort_get_panes_session(
    mut s: *mut session,
    mut n: *mut u_int,
    mut sort_crit: *mut sort_criteria,
) -> *mut *mut window_pane {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut i: u_int = 0;
    static mut l: *mut *mut window_pane =
        ::core::ptr::null::<*mut window_pane>() as *mut *mut window_pane;
    static mut lsz: u_int = 0 as u_int;
    i = 0 as u_int;
    wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        w = (*wl).window;
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if lsz <= i {
                lsz = lsz.wrapping_add(100 as u_int);
                l = xreallocarray(
                    l as *mut ::core::ffi::c_void,
                    lsz as size_t,
                    ::core::mem::size_of::<*mut window_pane>() as size_t,
                ) as *mut *mut window_pane;
            }
            let fresh10 = i;
            i = i.wrapping_add(1);
            let ref mut fresh11 = *l.offset(fresh10 as isize);
            *fresh11 = wp;
            wp = (*wp).entry.tqe_next;
        }
        wl = winlinks_RB_NEXT(wl);
    }
    sort_qsort(
        l as *mut ::core::ffi::c_void,
        i,
        ::core::mem::size_of::<*mut window_pane>() as u_int,
        Some(
            sort_pane_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    *n = i;
    return l;
}
#[no_mangle]
pub unsafe extern "C" fn sort_get_panes_window(
    mut w: *mut window,
    mut n: *mut u_int,
    mut sort_crit: *mut sort_criteria,
) -> *mut *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut i: u_int = 0;
    static mut l: *mut *mut window_pane =
        ::core::ptr::null::<*mut window_pane>() as *mut *mut window_pane;
    static mut lsz: u_int = 0 as u_int;
    i = 0 as u_int;
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if lsz <= i {
            lsz = lsz.wrapping_add(100 as u_int);
            l = xreallocarray(
                l as *mut ::core::ffi::c_void,
                lsz as size_t,
                ::core::mem::size_of::<*mut window_pane>() as size_t,
            ) as *mut *mut window_pane;
        }
        let fresh12 = i;
        i = i.wrapping_add(1);
        let ref mut fresh13 = *l.offset(fresh12 as isize);
        *fresh13 = wp;
        wp = (*wp).entry.tqe_next;
    }
    sort_qsort(
        l as *mut ::core::ffi::c_void,
        i,
        ::core::mem::size_of::<*mut window_pane>() as u_int,
        Some(
            sort_pane_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    *n = i;
    return l;
}
#[no_mangle]
pub unsafe extern "C" fn sort_get_winlinks(
    mut n: *mut u_int,
    mut sort_crit: *mut sort_criteria,
) -> *mut *mut winlink {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut i: u_int = 0;
    static mut l: *mut *mut winlink = ::core::ptr::null::<*mut winlink>() as *mut *mut winlink;
    static mut lsz: u_int = 0 as u_int;
    i = 0 as u_int;
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
        while !wl.is_null() {
            if lsz <= i {
                lsz = lsz.wrapping_add(100 as u_int);
                l = xreallocarray(
                    l as *mut ::core::ffi::c_void,
                    lsz as size_t,
                    ::core::mem::size_of::<*mut winlink>() as size_t,
                ) as *mut *mut winlink;
            }
            let fresh14 = i;
            i = i.wrapping_add(1);
            let ref mut fresh15 = *l.offset(fresh14 as isize);
            *fresh15 = wl;
            wl = winlinks_RB_NEXT(wl);
        }
        s = sessions_RB_NEXT(s);
    }
    sort_qsort(
        l as *mut ::core::ffi::c_void,
        i,
        ::core::mem::size_of::<*mut winlink>() as u_int,
        Some(
            sort_winlink_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    *n = i;
    return l;
}
#[no_mangle]
pub unsafe extern "C" fn sort_get_winlinks_session(
    mut s: *mut session,
    mut n: *mut u_int,
    mut sort_crit: *mut sort_criteria,
) -> *mut *mut winlink {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut i: u_int = 0;
    static mut l: *mut *mut winlink = ::core::ptr::null::<*mut winlink>() as *mut *mut winlink;
    static mut lsz: u_int = 0 as u_int;
    i = 0 as u_int;
    wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        if lsz <= i {
            lsz = lsz.wrapping_add(100 as u_int);
            l = xreallocarray(
                l as *mut ::core::ffi::c_void,
                lsz as size_t,
                ::core::mem::size_of::<*mut winlink>() as size_t,
            ) as *mut *mut winlink;
        }
        let fresh16 = i;
        i = i.wrapping_add(1);
        let ref mut fresh17 = *l.offset(fresh16 as isize);
        *fresh17 = wl;
        wl = winlinks_RB_NEXT(wl);
    }
    sort_qsort(
        l as *mut ::core::ffi::c_void,
        i,
        ::core::mem::size_of::<*mut winlink>() as u_int,
        Some(
            sort_winlink_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    *n = i;
    return l;
}
#[no_mangle]
pub unsafe extern "C" fn sort_get_key_bindings(
    mut n: *mut u_int,
    mut sort_crit: *mut sort_criteria,
) -> *mut *mut key_binding {
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut i: u_int = 0 as u_int;
    static mut l: *mut *mut key_binding =
        ::core::ptr::null::<*mut key_binding>() as *mut *mut key_binding;
    static mut lsz: u_int = 0 as u_int;
    table = key_bindings_first_table();
    while !table.is_null() {
        bd = key_bindings_first(table);
        while !bd.is_null() {
            if lsz <= i {
                lsz = lsz.wrapping_add(100 as u_int);
                l = xreallocarray(
                    l as *mut ::core::ffi::c_void,
                    lsz as size_t,
                    ::core::mem::size_of::<*mut key_binding>() as size_t,
                ) as *mut *mut key_binding;
            }
            let fresh18 = i;
            i = i.wrapping_add(1);
            let ref mut fresh19 = *l.offset(fresh18 as isize);
            *fresh19 = bd;
            bd = key_bindings_next(table, bd);
        }
        table = key_bindings_next_table(table);
    }
    sort_qsort(
        l as *mut ::core::ffi::c_void,
        i,
        ::core::mem::size_of::<*mut key_binding>() as u_int,
        Some(
            sort_key_binding_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    *n = i;
    return l;
}
#[no_mangle]
pub unsafe extern "C" fn sort_get_key_bindings_table(
    mut table: *mut key_table,
    mut n: *mut u_int,
    mut sort_crit: *mut sort_criteria,
) -> *mut *mut key_binding {
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut i: u_int = 0 as u_int;
    static mut l: *mut *mut key_binding =
        ::core::ptr::null::<*mut key_binding>() as *mut *mut key_binding;
    static mut lsz: u_int = 0 as u_int;
    bd = key_bindings_first(table);
    while !bd.is_null() {
        if lsz <= i {
            lsz = lsz.wrapping_add(100 as u_int);
            l = xreallocarray(
                l as *mut ::core::ffi::c_void,
                lsz as size_t,
                ::core::mem::size_of::<*mut key_binding>() as size_t,
            ) as *mut *mut key_binding;
        }
        let fresh20 = i;
        i = i.wrapping_add(1);
        let ref mut fresh21 = *l.offset(fresh20 as isize);
        *fresh21 = bd;
        bd = key_bindings_next(table, bd);
    }
    sort_qsort(
        l as *mut ::core::ffi::c_void,
        i,
        ::core::mem::size_of::<*mut key_binding>() as u_int,
        Some(
            sort_key_binding_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
        sort_crit,
    );
    *n = i;
    return l;
}

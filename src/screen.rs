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
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strlcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn calloc(__nmemb: size_t, __size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut global_options: *mut options;
    fn clean_name(_: *const ::core::ffi::c_char, _: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn tty_acs_get(_: *mut tty, _: u_char) -> *const ::core::ffi::c_char;
    fn grid_check_is_clear(_: *mut grid);
    fn grid_empty_line(_: *mut grid, _: u_int, _: u_int);
    fn grid_create(_: u_int, _: u_int, _: u_int) -> *mut grid;
    fn grid_destroy(_: *mut grid);
    fn grid_adjust_lines(_: *mut grid, _: u_int);
    fn grid_clear_lines(_: *mut grid, _: u_int, _: u_int, _: u_int);
    fn grid_duplicate_lines(_: *mut grid, _: u_int, _: *mut grid, _: u_int, _: u_int);
    fn grid_reflow(_: *mut grid, _: u_int);
    fn grid_wrap_position(_: *mut grid, _: u_int, _: u_int, _: *mut u_int, _: *mut u_int);
    fn grid_unwrap_position(_: *mut grid, _: *mut u_int, _: *mut u_int, _: u_int, _: u_int);
    fn grid_view_clear(_: *mut grid, _: u_int, _: u_int, _: u_int, _: u_int, _: u_int);
    fn grid_view_delete_lines(_: *mut grid, _: u_int, _: u_int, _: u_int);
    fn screen_write_make_list(_: *mut screen);
    fn screen_write_free_list(_: *mut screen);
    fn utf8_to_data(_: utf8_char, _: *mut utf8_data);
    fn utf8_copy(_: *mut utf8_data, _: *const utf8_data);
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatal(_: *const ::core::ffi::c_char, ...) -> !;
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
    fn style_apply(
        _: *mut grid_cell,
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: *mut format_tree,
    );
    fn hyperlinks_init() -> *mut hyperlinks;
    fn hyperlinks_reset(_: *mut hyperlinks);
    fn hyperlinks_free(_: *mut hyperlinks);
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
pub struct screen_sel {
    pub hidden: ::core::ffi::c_int,
    pub rectangle: ::core::ffi::c_int,
    pub modekeys: ::core::ffi::c_int,
    pub sx: u_int,
    pub sy: u_int,
    pub ex: u_int,
    pub ey: u_int,
    pub clipx: u_int,
    pub cell: grid_cell,
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct screen_titles {
    pub tqh_first: *mut screen_title_entry,
    pub tqh_last: *mut *mut screen_title_entry,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct screen_title_entry {
    pub text: *mut ::core::ffi::c_char,
    pub entry: C2RustUnnamed_14,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_14 {
    pub tqe_next: *mut screen_title_entry,
    pub tqe_prev: *mut *mut screen_title_entry,
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
    pub entry: C2RustUnnamed_31,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_31 {
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
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const MODEKEY_EMACS: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MODE_CURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MODE_INSERT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MODE_KCURSOR: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MODE_KKEYPAD: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const MODE_WRAP: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const MODE_MOUSE_STANDARD: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const MODE_MOUSE_BUTTON: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const MODE_CURSOR_BLINKING: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const MODE_MOUSE_UTF8: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const MODE_MOUSE_SGR: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const MODE_BRACKETPASTE: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const MODE_FOCUSON: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const MODE_MOUSE_ALL: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const MODE_ORIGIN: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const MODE_CRLF: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const MODE_KEYS_EXTENDED: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
pub const MODE_CURSOR_VERY_VISIBLE: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const MODE_CURSOR_BLINKING_SET: ::core::ffi::c_int = 0x20000 as ::core::ffi::c_int;
pub const MODE_KEYS_EXTENDED_2: ::core::ffi::c_int = 0x40000 as ::core::ffi::c_int;
pub const MODE_THEME_UPDATES: ::core::ffi::c_int = 0x80000 as ::core::ffi::c_int;
pub const MODE_SYNC: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;
pub const ALL_MODES: ::core::ffi::c_int = 0xffffff as ::core::ffi::c_int;
pub const EXTENDED_KEY_MODES: ::core::ffi::c_int = MODE_KEYS_EXTENDED | MODE_KEYS_EXTENDED_2;
unsafe extern "C" fn screen_free_titles(mut s: *mut screen) {
    let mut title_entry: *mut screen_title_entry = ::core::ptr::null_mut::<screen_title_entry>();
    if (*s).titles.is_null() {
        return;
    }
    loop {
        title_entry = (*(*s).titles).tqh_first;
        if title_entry.is_null() {
            break;
        }
        if !(*title_entry).entry.tqe_next.is_null() {
            (*(*title_entry).entry.tqe_next).entry.tqe_prev = (*title_entry).entry.tqe_prev;
        } else {
            (*(*s).titles).tqh_last = (*title_entry).entry.tqe_prev;
        }
        *(*title_entry).entry.tqe_prev = (*title_entry).entry.tqe_next;
        free((*title_entry).text as *mut ::core::ffi::c_void);
        free(title_entry as *mut ::core::ffi::c_void);
    }
    free((*s).titles as *mut ::core::ffi::c_void);
    (*s).titles = ::core::ptr::null_mut::<screen_titles>();
    (*s).ntitles = 0 as u_int;
}
#[no_mangle]
pub unsafe extern "C" fn screen_init(
    mut s: *mut screen,
    mut sx: u_int,
    mut sy: u_int,
    mut hlimit: u_int,
) {
    (*s).grid = grid_create(sx, sy, hlimit);
    (*s).saved_grid = ::core::ptr::null_mut::<grid>();
    (*s).title = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    (*s).titles = ::core::ptr::null_mut::<screen_titles>();
    (*s).ntitles = 0 as u_int;
    (*s).path = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*s).cstyle = SCREEN_CURSOR_DEFAULT;
    (*s).default_cstyle = SCREEN_CURSOR_DEFAULT;
    (*s).mode = MODE_CURSOR;
    (*s).default_mode = 0 as ::core::ffi::c_int;
    (*s).ccolour = -(1 as ::core::ffi::c_int);
    (*s).default_ccolour = -(1 as ::core::ffi::c_int);
    (*s).tabs = ::core::ptr::null_mut::<bitstr_t>();
    (*s).sel = ::core::ptr::null_mut::<screen_sel>();
    (*s).write_list = ::core::ptr::null_mut::<screen_write_cline>();
    (*s).hyperlinks = ::core::ptr::null_mut::<hyperlinks>();
    screen_reinit(s, 1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_reinit(mut s: *mut screen, mut check: ::core::ffi::c_int) {
    (*s).cx = 0 as u_int;
    (*s).cy = 0 as u_int;
    (*s).rupper = 0 as u_int;
    (*s).rlower = (*(*s).grid).sy.wrapping_sub(1 as u_int);
    (*s).mode = MODE_CURSOR | MODE_WRAP | (*s).mode & MODE_CRLF;
    if options_get_number(
        global_options,
        b"extended-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 2 as ::core::ffi::c_longlong
    {
        (*s).mode = (*s).mode & !EXTENDED_KEY_MODES | MODE_KEYS_EXTENDED;
    }
    if !(*s).saved_grid.is_null() {
        screen_alternate_off(
            s,
            ::core::ptr::null_mut::<grid_cell>(),
            0 as ::core::ffi::c_int,
        );
    }
    (*s).saved_cx = UINT_MAX as u_int;
    (*s).saved_cy = UINT_MAX as u_int;
    screen_reset_tabs(s);
    if check != 0 {
        grid_check_is_clear((*s).grid);
    }
    grid_clear_lines((*s).grid, (*(*s).grid).hsize, (*(*s).grid).sy, 8 as u_int);
    screen_clear_selection(s);
    screen_free_titles(s);
    screen_set_progress_bar(s, PROGRESS_BAR_HIDDEN, 0 as ::core::ffi::c_int);
    screen_reset_hyperlinks(s);
}
#[no_mangle]
pub unsafe extern "C" fn screen_reset_hyperlinks(mut s: *mut screen) {
    if (*s).hyperlinks.is_null() {
        (*s).hyperlinks = hyperlinks_init();
    } else {
        hyperlinks_reset((*s).hyperlinks);
    };
}
#[no_mangle]
pub unsafe extern "C" fn screen_free(mut s: *mut screen) {
    free((*s).sel as *mut ::core::ffi::c_void);
    free((*s).tabs as *mut ::core::ffi::c_void);
    free((*s).path as *mut ::core::ffi::c_void);
    free((*s).title as *mut ::core::ffi::c_void);
    if !(*s).write_list.is_null() {
        screen_write_free_list(s);
    }
    if !(*s).saved_grid.is_null() {
        grid_destroy((*s).saved_grid);
    }
    grid_destroy((*s).grid);
    if !(*s).hyperlinks.is_null() {
        hyperlinks_free((*s).hyperlinks);
    }
    screen_free_titles(s);
}
#[no_mangle]
pub unsafe extern "C" fn screen_reset_tabs(mut s: *mut screen) {
    let mut i: u_int = 0;
    free((*s).tabs as *mut ::core::ffi::c_void);
    (*s).tabs = calloc(
        ((*(*s).grid).sx.wrapping_add(7 as u_int) >> 3 as ::core::ffi::c_int) as size_t,
        ::core::mem::size_of::<bitstr_t>() as size_t,
    ) as *mut bitstr_t;
    if (*s).tabs.is_null() {
        fatal(b"bit_alloc failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    i = 8 as u_int;
    while i < (*(*s).grid).sx {
        let ref mut fresh0 = *(*s).tabs.offset((i >> 3 as ::core::ffi::c_int) as isize);
        *fresh0 = (*fresh0 as ::core::ffi::c_int | (1 as ::core::ffi::c_int) << (i & 0x7 as u_int))
            as bitstr_t;
        i = i.wrapping_add(8 as u_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_set_default_cursor(mut s: *mut screen, mut oo: *mut options) {
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
    let mut c: ::core::ffi::c_int = 0;
    style_apply(
        &raw mut gc,
        oo,
        b"cursor-colour\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    (*s).default_ccolour = gc.fg;
    c = options_get_number(
        oo,
        b"cursor-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    (*s).default_mode = 0 as ::core::ffi::c_int;
    screen_set_cursor_style(
        c as u_int,
        &raw mut (*s).default_cstyle,
        &raw mut (*s).default_mode,
    );
}
#[no_mangle]
pub unsafe extern "C" fn screen_set_cursor_style(
    mut style: u_int,
    mut cstyle: *mut screen_cursor_style,
    mut mode: *mut ::core::ffi::c_int,
) {
    match style {
        0 => {
            *cstyle = SCREEN_CURSOR_DEFAULT;
        }
        1 => {
            *cstyle = SCREEN_CURSOR_BLOCK;
            *mode |= MODE_CURSOR_BLINKING;
        }
        2 => {
            *cstyle = SCREEN_CURSOR_BLOCK;
            *mode &= !MODE_CURSOR_BLINKING;
        }
        3 => {
            *cstyle = SCREEN_CURSOR_UNDERLINE;
            *mode |= MODE_CURSOR_BLINKING;
        }
        4 => {
            *cstyle = SCREEN_CURSOR_UNDERLINE;
            *mode &= !MODE_CURSOR_BLINKING;
        }
        5 => {
            *cstyle = SCREEN_CURSOR_BAR;
            *mode |= MODE_CURSOR_BLINKING;
        }
        6 => {
            *cstyle = SCREEN_CURSOR_BAR;
            *mode &= !MODE_CURSOR_BLINKING;
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn screen_set_cursor_colour(
    mut s: *mut screen,
    mut colour: ::core::ffi::c_int,
) {
    (*s).ccolour = colour;
}
#[no_mangle]
pub unsafe extern "C" fn screen_set_title(
    mut s: *mut screen,
    mut title: *const ::core::ffi::c_char,
    mut untrusted: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut new_title: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    new_title = clean_name(title, untrusted);
    if new_title.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    free((*s).title as *mut ::core::ffi::c_void);
    (*s).title = new_title;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn screen_set_path(
    mut s: *mut screen,
    mut path: *const ::core::ffi::c_char,
    mut untrusted: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut new_path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    new_path = clean_name(path, untrusted);
    if new_path.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    free((*s).path as *mut ::core::ffi::c_void);
    (*s).path = new_path;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn screen_push_title(mut s: *mut screen) {
    let mut title_entry: *mut screen_title_entry = ::core::ptr::null_mut::<screen_title_entry>();
    log_debug(
        b"%s: %u\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_push_title\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).ntitles,
    );
    while (*s).ntitles >= 10 as u_int {
        title_entry = *(*((*(*s).titles).tqh_last as *mut screen_titles)).tqh_last;
        free((*title_entry).text as *mut ::core::ffi::c_void);
        if !(*title_entry).entry.tqe_next.is_null() {
            (*(*title_entry).entry.tqe_next).entry.tqe_prev = (*title_entry).entry.tqe_prev;
        } else {
            (*(*s).titles).tqh_last = (*title_entry).entry.tqe_prev;
        }
        *(*title_entry).entry.tqe_prev = (*title_entry).entry.tqe_next;
        free(title_entry as *mut ::core::ffi::c_void);
        (*s).ntitles = (*s).ntitles.wrapping_sub(1);
    }
    if (*s).titles.is_null() {
        (*s).titles =
            xmalloc(::core::mem::size_of::<screen_titles>() as size_t) as *mut screen_titles;
        (*(*s).titles).tqh_first = ::core::ptr::null_mut::<screen_title_entry>();
        (*(*s).titles).tqh_last = &raw mut (*(*s).titles).tqh_first;
    }
    title_entry =
        xmalloc(::core::mem::size_of::<screen_title_entry>() as size_t) as *mut screen_title_entry;
    (*title_entry).text = xstrdup((*s).title);
    (*title_entry).entry.tqe_next = (*(*s).titles).tqh_first;
    if !(*title_entry).entry.tqe_next.is_null() {
        (*(*(*s).titles).tqh_first).entry.tqe_prev = &raw mut (*title_entry).entry.tqe_next;
    } else {
        (*(*s).titles).tqh_last = &raw mut (*title_entry).entry.tqe_next;
    }
    (*(*s).titles).tqh_first = title_entry;
    (*title_entry).entry.tqe_prev = &raw mut (*(*s).titles).tqh_first;
    (*s).ntitles = (*s).ntitles.wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn screen_pop_title(mut s: *mut screen) {
    let mut title_entry: *mut screen_title_entry = ::core::ptr::null_mut::<screen_title_entry>();
    if (*s).titles.is_null() {
        return;
    }
    log_debug(
        b"%s: %u\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_pop_title\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).ntitles,
    );
    title_entry = (*(*s).titles).tqh_first;
    if !title_entry.is_null() {
        free((*s).title as *mut ::core::ffi::c_void);
        (*s).title = (*title_entry).text;
        if !(*title_entry).entry.tqe_next.is_null() {
            (*(*title_entry).entry.tqe_next).entry.tqe_prev = (*title_entry).entry.tqe_prev;
        } else {
            (*(*s).titles).tqh_last = (*title_entry).entry.tqe_prev;
        }
        *(*title_entry).entry.tqe_prev = (*title_entry).entry.tqe_next;
        free(title_entry as *mut ::core::ffi::c_void);
        (*s).ntitles = (*s).ntitles.wrapping_sub(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_set_progress_bar(
    mut s: *mut screen,
    mut pbs: progress_bar_state,
    mut p: ::core::ffi::c_int,
) {
    (*s).progress_bar.state = pbs;
    if p >= 0 as ::core::ffi::c_int
        && pbs as ::core::ffi::c_uint
            != PROGRESS_BAR_INDETERMINATE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*s).progress_bar.progress = p;
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_resize_cursor(
    mut s: *mut screen,
    mut sx: u_int,
    mut sy: u_int,
    mut reflow: ::core::ffi::c_int,
    mut eat_empty: ::core::ffi::c_int,
    mut cursor: ::core::ffi::c_int,
) {
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*(*s).grid).hsize.wrapping_add((*s).cy);
    if !(*s).write_list.is_null() {
        screen_write_free_list(s);
    }
    log_debug(
        b"%s: new size %ux%u, now %ux%u (cursor %u,%u = %u,%u)\0" as *const u8
            as *const ::core::ffi::c_char,
        b"screen_resize_cursor\0" as *const u8 as *const ::core::ffi::c_char,
        sx,
        sy,
        (*(*s).grid).sx,
        (*(*s).grid).sy,
        (*s).cx,
        (*s).cy,
        cx,
        cy,
    );
    if sx < 1 as u_int {
        sx = 1 as u_int;
    }
    if sy < 1 as u_int {
        sy = 1 as u_int;
    }
    if sx != (*(*s).grid).sx {
        (*(*s).grid).sx = sx;
        screen_reset_tabs(s);
    } else {
        reflow = 0 as ::core::ffi::c_int;
    }
    if sy != (*(*s).grid).sy {
        screen_resize_y(s, sy, eat_empty, &raw mut cy);
    }
    if reflow != 0 {
        screen_reflow(s, sx, &raw mut cx, &raw mut cy, cursor);
    }
    if cy >= (*(*s).grid).hsize {
        (*s).cx = cx;
        (*s).cy = cy.wrapping_sub((*(*s).grid).hsize);
    } else {
        (*s).cx = 0 as u_int;
        (*s).cy = 0 as u_int;
    }
    log_debug(
        b"%s: cursor finished at %u,%u = %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_resize_cursor\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).cx,
        (*s).cy,
        cx,
        cy,
    );
    if !(*s).write_list.is_null() {
        screen_write_make_list(s);
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_resize(
    mut s: *mut screen,
    mut sx: u_int,
    mut sy: u_int,
    mut reflow: ::core::ffi::c_int,
) {
    screen_resize_cursor(
        s,
        sx,
        sy,
        reflow,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn screen_resize_y(
    mut s: *mut screen,
    mut sy: u_int,
    mut eat_empty: ::core::ffi::c_int,
    mut cy: *mut u_int,
) {
    let mut gd: *mut grid = (*s).grid;
    let mut needed: u_int = 0;
    let mut available: u_int = 0;
    let mut oldy: u_int = 0;
    let mut i: u_int = 0;
    if sy == 0 as u_int {
        fatalx(b"zero size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    oldy = (*(*s).grid).sy;
    if sy < oldy {
        needed = oldy.wrapping_sub(sy);
        if eat_empty != 0 {
            available = oldy.wrapping_sub(1 as u_int).wrapping_sub((*s).cy);
            if available > 0 as u_int {
                if available > needed {
                    available = needed;
                }
                grid_view_delete_lines(gd, oldy.wrapping_sub(available), available, 8 as u_int);
            }
            needed = needed.wrapping_sub(available);
        }
        available = (*s).cy;
        if (*gd).flags & GRID_HISTORY != 0 {
            (*gd).hscrolled = (*gd).hscrolled.wrapping_add(needed);
            (*gd).hsize = (*gd).hsize.wrapping_add(needed);
        } else if needed > 0 as u_int && available > 0 as u_int {
            if available > needed {
                available = needed;
            }
            grid_view_delete_lines(gd, 0 as u_int, available, 8 as u_int);
            *cy = (*cy).wrapping_sub(available);
        }
    }
    grid_adjust_lines(gd, (*gd).hsize.wrapping_add(sy));
    if sy > oldy {
        needed = sy.wrapping_sub(oldy);
        available = (*gd).hscrolled;
        if (*gd).flags & GRID_HISTORY != 0 && available > 0 as u_int {
            if available > needed {
                available = needed;
            }
            (*gd).hscrolled = (*gd).hscrolled.wrapping_sub(available);
            (*gd).hsize = (*gd).hsize.wrapping_sub(available);
        } else {
            available = 0 as u_int;
        }
        needed = needed.wrapping_sub(available);
        i = (*gd).hsize.wrapping_add(sy).wrapping_sub(needed);
        while i < (*gd).hsize.wrapping_add(sy) {
            grid_empty_line(gd, i, 8 as u_int);
            i = i.wrapping_add(1);
        }
    }
    (*gd).sy = sy;
    (*s).rupper = 0 as u_int;
    (*s).rlower = (*(*s).grid).sy.wrapping_sub(1 as u_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_set_selection(
    mut s: *mut screen,
    mut sx: u_int,
    mut sy: u_int,
    mut ex: u_int,
    mut ey: u_int,
    mut rectangle: u_int,
    mut clipx: u_int,
    mut modekeys: ::core::ffi::c_int,
    mut gc: *mut grid_cell,
) {
    if (*s).sel.is_null() {
        (*s).sel =
            xcalloc(1 as size_t, ::core::mem::size_of::<screen_sel>() as size_t) as *mut screen_sel;
    }
    memcpy(
        &raw mut (*(*s).sel).cell as *mut ::core::ffi::c_void,
        gc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    (*(*s).sel).hidden = 0 as ::core::ffi::c_int;
    (*(*s).sel).rectangle = rectangle as ::core::ffi::c_int;
    (*(*s).sel).modekeys = modekeys;
    (*(*s).sel).sx = sx;
    (*(*s).sel).sy = sy;
    (*(*s).sel).ex = ex;
    (*(*s).sel).ey = ey;
    (*(*s).sel).clipx = clipx;
}
#[no_mangle]
pub unsafe extern "C" fn screen_clear_selection(mut s: *mut screen) {
    free((*s).sel as *mut ::core::ffi::c_void);
    (*s).sel = ::core::ptr::null_mut::<screen_sel>();
}
#[no_mangle]
pub unsafe extern "C" fn screen_hide_selection(mut s: *mut screen) {
    if !(*s).sel.is_null() {
        (*(*s).sel).hidden = 1 as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_check_selection(
    mut s: *mut screen,
    mut px: u_int,
    mut py: u_int,
) -> ::core::ffi::c_int {
    let mut sel: *mut screen_sel = (*s).sel;
    let mut xx: u_int = 0;
    if sel.is_null() || (*sel).hidden != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if px < (*sel).clipx {
        return 0 as ::core::ffi::c_int;
    }
    if (*sel).rectangle != 0 {
        if (*sel).sy < (*sel).ey {
            if py < (*sel).sy || py > (*sel).ey {
                return 0 as ::core::ffi::c_int;
            }
        } else if (*sel).sy > (*sel).ey {
            if py > (*sel).sy || py < (*sel).ey {
                return 0 as ::core::ffi::c_int;
            }
        } else if py != (*sel).sy {
            return 0 as ::core::ffi::c_int;
        }
        if (*sel).ex < (*sel).sx {
            if px < (*sel).ex {
                return 0 as ::core::ffi::c_int;
            }
            if px > (*sel).sx {
                return 0 as ::core::ffi::c_int;
            }
        } else {
            if px < (*sel).sx {
                return 0 as ::core::ffi::c_int;
            }
            if px > (*sel).ex {
                return 0 as ::core::ffi::c_int;
            }
        }
    } else if (*sel).sy < (*sel).ey {
        if py < (*sel).sy || py > (*sel).ey {
            return 0 as ::core::ffi::c_int;
        }
        if py == (*sel).sy && px < (*sel).sx {
            return 0 as ::core::ffi::c_int;
        }
        if (*sel).modekeys == MODEKEY_EMACS {
            xx = if (*sel).ex == 0 as u_int {
                0 as u_int
            } else {
                (*sel).ex.wrapping_sub(1 as u_int)
            };
        } else {
            xx = (*sel).ex;
        }
        if py == (*sel).ey && px > xx {
            return 0 as ::core::ffi::c_int;
        }
    } else if (*sel).sy > (*sel).ey {
        if py > (*sel).sy || py < (*sel).ey {
            return 0 as ::core::ffi::c_int;
        }
        if py == (*sel).ey && px < (*sel).ex {
            return 0 as ::core::ffi::c_int;
        }
        if (*sel).modekeys == MODEKEY_EMACS {
            xx = (*sel).sx.wrapping_sub(1 as u_int);
        } else {
            xx = (*sel).sx;
        }
        if py == (*sel).sy && ((*sel).sx == 0 as u_int || px > xx) {
            return 0 as ::core::ffi::c_int;
        }
    } else {
        if py != (*sel).sy {
            return 0 as ::core::ffi::c_int;
        }
        if (*sel).ex < (*sel).sx {
            if (*sel).modekeys == MODEKEY_EMACS {
                xx = (*sel).sx.wrapping_sub(1 as u_int);
            } else {
                xx = (*sel).sx;
            }
            if px > xx || px < (*sel).ex {
                return 0 as ::core::ffi::c_int;
            }
        } else {
            if (*sel).modekeys == MODEKEY_EMACS {
                xx = if (*sel).ex == 0 as u_int {
                    0 as u_int
                } else {
                    (*sel).ex.wrapping_sub(1 as u_int)
                };
            } else {
                xx = (*sel).ex;
            }
            if px < (*sel).sx || px > xx {
                return 0 as ::core::ffi::c_int;
            }
        }
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn screen_select_cell(
    mut s: *mut screen,
    mut dst: *mut grid_cell,
    mut src: *const grid_cell,
) -> ::core::ffi::c_int {
    if (*s).sel.is_null() || (*(*s).sel).hidden != 0 {
        return 0 as ::core::ffi::c_int;
    }
    memcpy(
        dst as *mut ::core::ffi::c_void,
        &raw mut (*(*s).sel).cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    if (*dst).fg == 8 as ::core::ffi::c_int || (*dst).fg == 9 as ::core::ffi::c_int {
        (*dst).fg = (*src).fg;
    }
    if (*dst).bg == 8 as ::core::ffi::c_int || (*dst).bg == 9 as ::core::ffi::c_int {
        (*dst).bg = (*src).bg;
    }
    utf8_copy(&raw mut (*dst).data, &raw const (*src).data);
    (*dst).flags = (*src).flags;
    if (*dst).attr as ::core::ffi::c_int & GRID_ATTR_NOATTR != 0 {
        (*dst).attr = ((*dst).attr as ::core::ffi::c_int
            | (*src).attr as ::core::ffi::c_int & GRID_ATTR_CHARSET)
            as u_short;
    } else {
        (*dst).attr =
            ((*dst).attr as ::core::ffi::c_int | (*src).attr as ::core::ffi::c_int) as u_short;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn screen_reflow(
    mut s: *mut screen,
    mut new_x: u_int,
    mut cx: *mut u_int,
    mut cy: *mut u_int,
    mut cursor: ::core::ffi::c_int,
) {
    let mut wx: u_int = 0;
    let mut wy: u_int = 0;
    if cursor != 0 {
        grid_wrap_position((*s).grid, *cx, *cy, &raw mut wx, &raw mut wy);
        log_debug(
            b"%s: cursor %u,%u is %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_reflow\0" as *const u8 as *const ::core::ffi::c_char,
            *cx,
            *cy,
            wx,
            wy,
        );
    }
    grid_reflow((*s).grid, new_x);
    if cursor != 0 {
        grid_unwrap_position((*s).grid, cx, cy, wx, wy);
        log_debug(
            b"%s: new cursor is %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_reflow\0" as *const u8 as *const ::core::ffi::c_char,
            *cx,
            *cy,
        );
    } else {
        *cx = 0 as u_int;
        *cy = (*(*s).grid).hsize;
    };
}
#[no_mangle]
pub unsafe extern "C" fn screen_alternate_on(
    mut s: *mut screen,
    mut gc: *mut grid_cell,
    mut cursor: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if !(*s).saved_grid.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    sx = (*(*s).grid).sx;
    sy = (*(*s).grid).sy;
    (*s).saved_grid = grid_create(sx, sy, 0 as u_int);
    grid_duplicate_lines(
        (*s).saved_grid,
        0 as u_int,
        (*s).grid,
        (*(*s).grid).hsize,
        sy,
    );
    if cursor != 0 {
        (*s).saved_cx = (*s).cx;
        (*s).saved_cy = (*s).cy;
    }
    memcpy(
        &raw mut (*s).saved_cell as *mut ::core::ffi::c_void,
        gc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    grid_view_clear((*s).grid, 0 as u_int, 0 as u_int, sx, sy, 8 as u_int);
    (*s).saved_flags = (*(*s).grid).flags;
    (*(*s).grid).flags &= !GRID_HISTORY;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn screen_alternate_off(
    mut s: *mut screen,
    mut gc: *mut grid_cell,
    mut cursor: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut sx: u_int = (*(*s).grid).sx;
    let mut sy: u_int = (*(*s).grid).sy;
    if !(*s).saved_grid.is_null() {
        screen_resize(
            s,
            (*(*s).saved_grid).sx,
            (*(*s).saved_grid).sy,
            0 as ::core::ffi::c_int,
        );
    }
    if cursor != 0 && (*s).saved_cx != UINT_MAX && (*s).saved_cy != UINT_MAX {
        (*s).cx = (*s).saved_cx;
        (*s).cy = (*s).saved_cy;
        if !gc.is_null() {
            memcpy(
                gc as *mut ::core::ffi::c_void,
                &raw mut (*s).saved_cell as *const ::core::ffi::c_void,
                ::core::mem::size_of::<grid_cell>() as size_t,
            );
        }
    }
    if (*s).saved_grid.is_null() {
        if (*s).cx > (*(*s).grid).sx.wrapping_sub(1 as u_int) {
            (*s).cx = (*(*s).grid).sx.wrapping_sub(1 as u_int);
        }
        if (*s).cy > (*(*s).grid).sy.wrapping_sub(1 as u_int) {
            (*s).cy = (*(*s).grid).sy.wrapping_sub(1 as u_int);
        }
        return 0 as ::core::ffi::c_int;
    }
    grid_duplicate_lines(
        (*s).grid,
        (*(*s).grid).hsize,
        (*s).saved_grid,
        0 as u_int,
        (*(*s).saved_grid).sy,
    );
    if (*s).saved_flags & GRID_HISTORY != 0 {
        (*(*s).grid).flags |= GRID_HISTORY;
    }
    screen_resize(s, sx, sy, 1 as ::core::ffi::c_int);
    grid_destroy((*s).saved_grid);
    (*s).saved_grid = ::core::ptr::null_mut::<grid>();
    if (*s).cx > (*(*s).grid).sx.wrapping_sub(1 as u_int) {
        (*s).cx = (*(*s).grid).sx.wrapping_sub(1 as u_int);
    }
    if (*s).cy > (*(*s).grid).sy.wrapping_sub(1 as u_int) {
        (*s).cy = (*(*s).grid).sy.wrapping_sub(1 as u_int);
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn screen_mode_to_string(
    mut mode: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    static mut tmp: [::core::ffi::c_char; 1024] = [0; 1024];
    if mode == 0 as ::core::ffi::c_int {
        return b"NONE\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if mode == ALL_MODES {
        return b"ALL\0" as *const u8 as *const ::core::ffi::c_char;
    }
    *(&raw mut tmp as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if mode & MODE_CURSOR != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"CURSOR,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_INSERT != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"INSERT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_KCURSOR != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"KCURSOR,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_KKEYPAD != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"KKEYPAD,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_WRAP != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"WRAP,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_MOUSE_STANDARD != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"MOUSE_STANDARD,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_MOUSE_BUTTON != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"MOUSE_BUTTON,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_CURSOR_BLINKING != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"CURSOR_BLINKING,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_CURSOR_VERY_VISIBLE != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"CURSOR_VERY_VISIBLE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_CURSOR_BLINKING_SET != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"CURSOR_BLINKING_SET,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_MOUSE_UTF8 != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"MOUSE_UTF8,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_MOUSE_SGR != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"MOUSE_SGR,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_BRACKETPASTE != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"BRACKETPASTE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_FOCUSON != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"FOCUSON,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_MOUSE_ALL != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"MOUSE_ALL,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_ORIGIN != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"ORIGIN,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_CRLF != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"CRLF,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_KEYS_EXTENDED != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"KEYS_EXTENDED,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_KEYS_EXTENDED_2 != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"KEYS_EXTENDED_2,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_THEME_UPDATES != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"THEME_UPDATES,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if mode & MODE_SYNC != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"SYNC,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as size_t,
        );
    }
    if *(&raw mut tmp as *mut ::core::ffi::c_char) as ::core::ffi::c_int != '\0' as i32 {
        tmp[strlen(&raw mut tmp as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
            '\0' as i32 as ::core::ffi::c_char;
    }
    return &raw mut tmp as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn screen_print(
    mut s: *mut screen,
    mut line: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    static mut buf: *mut ::core::ffi::c_char =
        ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char;
    static mut len: size_t = 16384 as size_t;
    let mut acs: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut last: size_t = 0 as size_t;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut gce: *mut grid_cell_entry = ::core::ptr::null_mut::<grid_cell_entry>();
    if buf.is_null() {
        buf = xmalloc(len) as *mut ::core::ffi::c_char;
    }
    y = 0 as u_int;
    's_28: while y < (*(*s).grid).hsize.wrapping_add((*(*s).grid).sy) {
        if !(line >= 0 as ::core::ffi::c_int && y != line as u_int) {
            n = snprintf(
                buf.offset(last as isize),
                len.wrapping_sub(last),
                b"%.4d \"\0" as *const u8 as *const ::core::ffi::c_char,
                y,
            );
            if n <= 0 as ::core::ffi::c_int || n as u_int as size_t >= len.wrapping_sub(last) {
                break;
            }
            last = last.wrapping_add(n as size_t);
            gl = (*(*s).grid).linedata.offset(y as isize) as *mut grid_line;
            x = 0 as u_int;
            while x < (*gl).cellused as u_int {
                gce = (*gl).celldata.offset(x as isize) as *mut grid_cell_entry;
                if !((*gce).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0) {
                    if !((*gce).flags as ::core::ffi::c_int) & GRID_FLAG_EXTENDED != 0 {
                        if last.wrapping_add(2 as size_t) >= len {
                            break 's_28;
                        }
                        let fresh1 = last;
                        last = last.wrapping_add(1);
                        *buf.offset(fresh1 as isize) =
                            (*gce).c2rust_unnamed.data.data as ::core::ffi::c_char;
                    } else if (*gce).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
                        if last.wrapping_add(2 as size_t) >= len {
                            break 's_28;
                        }
                        let fresh2 = last;
                        last = last.wrapping_add(1);
                        *buf.offset(fresh2 as isize) = '\t' as i32 as ::core::ffi::c_char;
                    } else if (*gce).flags as ::core::ffi::c_int & GRID_ATTR_CHARSET != 0 {
                        acs = tty_acs_get(
                            ::core::ptr::null_mut::<tty>(),
                            (*gce).c2rust_unnamed.data.data,
                        );
                        if !acs.is_null() {
                            n = strlen(acs) as ::core::ffi::c_int;
                        } else {
                            acs = &raw mut (*gce).c2rust_unnamed.data.data
                                as *const ::core::ffi::c_char;
                            n = 1 as ::core::ffi::c_int;
                        }
                        if last.wrapping_add(n as size_t).wrapping_add(1 as size_t) >= len {
                            break 's_28;
                        }
                        memcpy(
                            buf.offset(last as isize) as *mut ::core::ffi::c_void,
                            acs as *const ::core::ffi::c_void,
                            n as size_t,
                        );
                        last = last.wrapping_add(n as size_t);
                    } else {
                        utf8_to_data(
                            (*(*gl).extddata.offset((*gce).c2rust_unnamed.offset as isize)).data,
                            &raw mut ud,
                        );
                        if ud.size as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
                            if last
                                .wrapping_add(ud.size as size_t)
                                .wrapping_add(1 as size_t)
                                >= len
                            {
                                break 's_28;
                            }
                            memcpy(
                                buf.offset(last as isize) as *mut ::core::ffi::c_void,
                                &raw mut ud.data as *mut u_char as *const ::core::ffi::c_void,
                                ud.size as size_t,
                            );
                            last = last.wrapping_add(ud.size as size_t);
                        }
                    }
                }
                x = x.wrapping_add(1);
            }
            if last.wrapping_add(3 as size_t) >= len {
                break;
            }
            let fresh3 = last;
            last = last.wrapping_add(1);
            *buf.offset(fresh3 as isize) = '"' as i32 as ::core::ffi::c_char;
            let fresh4 = last;
            last = last.wrapping_add(1);
            *buf.offset(fresh4 as isize) = '\n' as i32 as ::core::ffi::c_char;
        }
        y = y.wrapping_add(1);
    }
    *buf.offset(last as isize) = '\0' as i32 as ::core::ffi::c_char;
    return buf;
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

use crate::src::shared::client::*;
use crate::src::shared::layout::*;
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
    pub type job;
    pub type screen_write_citem;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn evbuffer_get_length(buf: *const evbuffer) -> size_t;
    fn evbuffer_drain(buf: *mut evbuffer, len: size_t) -> ::core::ffi::c_int;
    fn evbuffer_pullup(buf: *mut evbuffer, size: ssize_t) -> *mut ::core::ffi::c_uchar;
    fn bufferevent_write(
        bufev: *mut bufferevent,
        data: *const ::core::ffi::c_void,
        size: size_t,
    ) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut global_w_options: *mut options;
    fn format_free(_: *mut format_tree);
    fn format_create_defaults(
        _: *mut cmdq_item,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut format_tree;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn job_run(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
        _: *mut environ,
        _: *mut session,
        _: *const ::core::ffi::c_char,
        _: job_update_cb,
        _: job_complete_cb,
        _: job_free_cb,
        _: *mut ::core::ffi::c_void,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut job;
    fn job_free(_: *mut job);
    fn job_resize(_: *mut job, _: u_int, _: u_int);
    fn job_get_status(_: *mut job) -> ::core::ffi::c_int;
    fn job_get_data(_: *mut job) -> *mut ::core::ffi::c_void;
    fn job_get_event(_: *mut job) -> *mut bufferevent;
    fn tty_draw_line(
        _: *mut tty,
        _: *mut screen,
        _: u_int,
        _: u_int,
        _: u_int,
        _: u_int,
        _: u_int,
        _: *const tty_style_ctx,
    );
    fn tty_resize(_: *mut tty);
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_continue(_: *mut cmdq_item);
    fn server_client_set_overlay(
        _: *mut client,
        _: u_int,
        _: overlay_check_cb,
        _: overlay_mode_cb,
        _: overlay_draw_cb,
        _: overlay_key_cb,
        _: overlay_free_cb,
        _: overlay_resize_cb,
        _: *mut ::core::ffi::c_void,
    );
    fn server_client_clear_overlay(_: *mut client);
    fn server_client_overlay_range(
        _: u_int,
        _: u_int,
        _: u_int,
        _: u_int,
        _: u_int,
        _: u_int,
        _: u_int,
        _: *mut visible_ranges,
    );
    fn server_client_unref(_: *mut client);
    fn server_redraw_client(_: *mut client);
    fn input_init(
        _: *mut window_pane,
        _: *mut bufferevent,
        _: *mut colour_palette,
        _: *mut client,
    ) -> *mut input_ctx;
    fn input_free(_: *mut input_ctx);
    fn input_parse_screen(
        _: *mut input_ctx,
        _: *mut screen,
        _: screen_write_init_ctx_cb,
        _: *mut ::core::ffi::c_void,
        _: *const u_char,
        _: size_t,
    );
    fn input_key(_: *mut screen, _: *mut bufferevent, _: key_code) -> ::core::ffi::c_int;
    fn input_key_get_mouse(
        _: *mut screen,
        _: *mut mouse_event,
        _: u_int,
        _: u_int,
        _: *mut *const ::core::ffi::c_char,
        _: *mut size_t,
    ) -> ::core::ffi::c_int;
    fn colour_palette_init(_: *mut colour_palette);
    fn colour_palette_free(_: *mut colour_palette);
    fn colour_palette_from_option(_: *mut colour_palette, _: *mut options);
    static grid_default_cell: grid_cell;
    fn screen_write_start(_: *mut screen_write_ctx, _: *mut screen);
    fn screen_write_stop(_: *mut screen_write_ctx);
    fn screen_write_fast_copy(
        _: *mut screen_write_ctx,
        _: *mut screen,
        _: u_int,
        _: u_int,
        _: u_int,
        _: u_int,
    );
    fn screen_write_box(
        _: *mut screen_write_ctx,
        _: u_int,
        _: u_int,
        _: box_lines,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
    );
    fn screen_write_cursormove(
        _: *mut screen_write_ctx,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn screen_write_clearscreen(_: *mut screen_write_ctx, _: u_int);
    fn screen_init(_: *mut screen, _: u_int, _: u_int, _: u_int);
    fn screen_free(_: *mut screen);
    fn screen_set_default_cursor(_: *mut screen, _: *mut options);
    fn screen_resize(_: *mut screen, _: u_int, _: u_int, _: ::core::ffi::c_int);
    fn style_parse(
        _: *mut style,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn style_apply(
        _: *mut grid_cell,
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: *mut format_tree,
    );
    fn style_set(_: *mut style, _: *const grid_cell);
    fn hyperlinks_copy(_: *mut hyperlinks) -> *mut hyperlinks;
    fn hyperlinks_free(_: *mut hyperlinks);
}
pub type ssize_t = isize;
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
pub type C2RustUnnamed_38 = ::core::ffi::c_ulong;
pub type job_update_cb = Option<unsafe extern "C" fn(*mut job) -> ()>;
pub type job_complete_cb = Option<unsafe extern "C" fn(*mut job) -> ()>;
pub type job_free_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type popup_close_cb =
    Option<unsafe extern "C" fn(::core::ffi::c_int, *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct popup_data {
    pub c: *mut client,
    pub item: *mut cmdq_item,
    pub flags: ::core::ffi::c_int,
    pub title: *mut ::core::ffi::c_char,
    pub style: *mut ::core::ffi::c_char,
    pub border_style: *mut ::core::ffi::c_char,
    pub border_cell: grid_cell,
    pub border_lines: box_lines,
    pub s: screen,
    pub defaults: grid_cell,
    pub palette: colour_palette,
    pub r: visible_ranges,
    pub job: *mut job,
    pub ictx: *mut input_ctx,
    pub status: ::core::ffi::c_int,
    pub cb: popup_close_cb,
    pub arg: *mut ::core::ffi::c_void,
    pub close: ::core::ffi::c_int,
    pub px: u_int,
    pub py: u_int,
    pub sx: u_int,
    pub sy: u_int,
    pub ppx: u_int,
    pub ppy: u_int,
    pub psx: u_int,
    pub psy: u_int,
    pub dragging: C2RustUnnamed_39,
    pub dx: u_int,
    pub dy: u_int,
    pub lx: u_int,
    pub ly: u_int,
    pub lb: u_int,
}
pub type C2RustUnnamed_39 = ::core::ffi::c_uint;
pub const SIZE: C2RustUnnamed_39 = 2;
pub const MOVE: C2RustUnnamed_39 = 1;
pub const OFF: C2RustUnnamed_39 = 0;
pub const NONE: C2RustUnnamed_40 = 0;
pub type C2RustUnnamed_40 = ::core::ffi::c_uint;
pub const BOTTOM: C2RustUnnamed_40 = 4;
pub const TOP: C2RustUnnamed_40 = 3;
pub const RIGHT: C2RustUnnamed_40 = 2;
pub const LEFT: C2RustUnnamed_40 = 1;
pub const SIGHUP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const MOUSE_MASK_BUTTONS: ::core::ffi::c_int = 195 as ::core::ffi::c_int;
pub const MOUSE_MASK_SHIFT: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const MOUSE_MASK_META: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const MOUSE_MASK_CTRL: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const MOUSE_MASK_DRAG: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const MOUSE_MASK_MODIFIERS: ::core::ffi::c_int =
    MOUSE_MASK_SHIFT | MOUSE_MASK_META | MOUSE_MASK_CTRL;
pub const MOUSE_BUTTON_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_3: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const TTY_CTX_WINDOW_BIGGER: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CLIENT_REDRAWOVERLAY: ::core::ffi::c_int = 0x2000000 as ::core::ffi::c_int;
pub const JOB_NOWAIT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const JOB_KEEPWRITE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const JOB_PTY: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const JOB_DEFAULTSHELL: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const POPUP_CLOSEEXIT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const POPUP_CLOSEEXITZERO: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const POPUP_CLOSEANYKEY: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
unsafe extern "C" fn popup_free(mut pd: *mut popup_data) {
    server_client_unref((*pd).c);
    if !(*pd).job.is_null() {
        job_free((*pd).job);
    }
    if !(*pd).ictx.is_null() {
        input_free((*pd).ictx);
    }
    free((*pd).r.ranges as *mut ::core::ffi::c_void);
    screen_free(&raw mut (*pd).s);
    colour_palette_free(&raw mut (*pd).palette);
    free((*pd).title as *mut ::core::ffi::c_void);
    free((*pd).style as *mut ::core::ffi::c_void);
    free((*pd).border_style as *mut ::core::ffi::c_void);
    free(pd as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn popup_reapply_styles(mut pd: *mut popup_data) {
    let mut c: *mut client = (*pd).c;
    let mut s: *mut session = (*c).session;
    let mut o: *mut options = ::core::ptr::null_mut::<options>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut sytmp: style = style {
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
    if s.is_null() {
        return;
    }
    o = (*(*(*s).curw).window).options;
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        c,
        s,
        (*s).curw,
        ::core::ptr::null_mut::<window_pane>(),
    );
    memcpy(
        &raw mut (*pd).defaults as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut (*pd).defaults,
        o,
        b"popup-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    if !(*pd).style.is_null() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(&raw mut sytmp, &raw mut (*pd).defaults, (*pd).style)
            == 0 as ::core::ffi::c_int
        {
            (*pd).defaults.fg = sytmp.gc.fg;
            (*pd).defaults.bg = sytmp.gc.bg;
        }
    }
    (*pd).defaults.attr = 0 as u_short;
    memcpy(
        &raw mut (*pd).border_cell as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut (*pd).border_cell,
        o,
        b"popup-border-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    if !(*pd).border_style.is_null() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(
            &raw mut sytmp,
            &raw mut (*pd).border_cell,
            (*pd).border_style,
        ) == 0 as ::core::ffi::c_int
        {
            (*pd).border_cell.fg = sytmp.gc.fg;
            (*pd).border_cell.bg = sytmp.gc.bg;
        }
    }
    (*pd).border_cell.attr = 0 as u_short;
    format_free(ft);
}
unsafe extern "C" fn popup_redraw_cb(mut ttyctx: *const tty_ctx) {
    let mut pd: *mut popup_data = (*ttyctx).arg as *mut popup_data;
    (*(*pd).c).flags |= CLIENT_REDRAWOVERLAY as uint64_t;
}
unsafe extern "C" fn popup_set_client_cb(
    mut ttyctx: *mut tty_ctx,
    mut c: *mut client,
) -> ::core::ffi::c_int {
    let mut pd: *mut popup_data = (*ttyctx).arg as *mut popup_data;
    if c != (*pd).c {
        return 0 as ::core::ffi::c_int;
    }
    if (*(*pd).c).flags & CLIENT_REDRAWOVERLAY as uint64_t != 0 {
        return 0 as ::core::ffi::c_int;
    }
    (*ttyctx).wox = 0 as u_int;
    (*ttyctx).woy = 0 as u_int;
    (*ttyctx).wsx = (*c).tty.sx;
    (*ttyctx).wsy = (*c).tty.sy;
    if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
        (*ttyctx).rxoff = (*pd).px as ::core::ffi::c_int;
        (*ttyctx).xoff = (*ttyctx).rxoff;
        (*ttyctx).ryoff = (*pd).py as ::core::ffi::c_int;
        (*ttyctx).yoff = (*ttyctx).ryoff;
    } else {
        (*ttyctx).rxoff = (*pd).px.wrapping_add(1 as u_int) as ::core::ffi::c_int;
        (*ttyctx).xoff = (*ttyctx).rxoff;
        (*ttyctx).ryoff = (*pd).py.wrapping_add(1 as u_int) as ::core::ffi::c_int;
        (*ttyctx).yoff = (*ttyctx).ryoff;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn popup_init_ctx_cb(mut ctx: *mut screen_write_ctx, mut ttyctx: *mut tty_ctx) {
    let mut pd: *mut popup_data = (*ctx).arg as *mut popup_data;
    memcpy(
        &raw mut (*ttyctx).defaults as *mut ::core::ffi::c_void,
        &raw mut (*pd).defaults as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    (*ttyctx).flags &= !TTY_CTX_WINDOW_BIGGER;
    (*ttyctx).style_ctx.defaults = &raw mut (*ttyctx).defaults;
    (*ttyctx).style_ctx.palette = &raw mut (*pd).palette;
    (*ttyctx).redraw_cb =
        Some(popup_redraw_cb as unsafe extern "C" fn(*const tty_ctx) -> ()) as tty_ctx_redraw_cb;
    (*ttyctx).set_client_cb = Some(
        popup_set_client_cb
            as unsafe extern "C" fn(*mut tty_ctx, *mut client) -> ::core::ffi::c_int,
    ) as tty_ctx_set_client_cb;
    (*ttyctx).arg = pd as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn popup_mode_cb(
    mut c: *mut client,
    mut data: *mut ::core::ffi::c_void,
    mut cx: *mut u_int,
    mut cy: *mut u_int,
) -> *mut screen {
    let mut pd: *mut popup_data = data as *mut popup_data;
    if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
        *cx = (*pd).px.wrapping_add((*pd).s.cx);
        *cy = (*pd).py.wrapping_add((*pd).s.cy);
    } else {
        *cx = (*pd).px.wrapping_add(1 as u_int).wrapping_add((*pd).s.cx);
        *cy = (*pd).py.wrapping_add(1 as u_int).wrapping_add((*pd).s.cy);
    }
    return &raw mut (*pd).s;
}
unsafe extern "C" fn popup_check_cb(
    mut c: *mut client,
    mut data: *mut ::core::ffi::c_void,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
) -> *mut visible_ranges {
    let mut pd: *mut popup_data = data as *mut popup_data;
    let mut r: *mut visible_ranges = &raw mut (*pd).r;
    server_client_overlay_range((*pd).px, (*pd).py, (*pd).sx, (*pd).sy, px, py, nx, r);
    return r;
}
unsafe extern "C" fn popup_draw_cb(mut c: *mut client, mut data: *mut ::core::ffi::c_void) {
    let mut pd: *mut popup_data = data as *mut popup_data;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut s: screen = screen {
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
    };
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
    let mut i: u_int = 0;
    let mut px: u_int = (*pd).px;
    let mut py: u_int = (*pd).py;
    let mut defaults: grid_cell = grid_cell {
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
    let mut style_ctx: tty_style_ctx = tty_style_ctx {
        defaults: ::core::ptr::null::<grid_cell>(),
        palette: ::core::ptr::null_mut::<colour_palette>(),
        dim: 0,
        hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
    };
    popup_reapply_styles(pd);
    screen_init(&raw mut s, (*pd).sx, (*pd).sy, 0 as u_int);
    if !(*pd).s.hyperlinks.is_null() {
        hyperlinks_free(s.hyperlinks);
        s.hyperlinks = hyperlinks_copy((*pd).s.hyperlinks);
    }
    screen_write_start(&raw mut ctx, &raw mut s);
    screen_write_clearscreen(&raw mut ctx, 8 as u_int);
    if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
        screen_write_cursormove(
            &raw mut ctx,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_fast_copy(
            &raw mut ctx,
            &raw mut (*pd).s,
            0 as u_int,
            0 as u_int,
            (*pd).sx,
            (*pd).sy,
        );
    } else if (*pd).sx > 2 as u_int && (*pd).sy > 2 as u_int {
        screen_write_box(
            &raw mut ctx,
            (*pd).sx,
            (*pd).sy,
            (*pd).border_lines,
            &raw mut (*pd).border_cell,
            (*pd).title,
        );
        screen_write_cursormove(
            &raw mut ctx,
            1 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_fast_copy(
            &raw mut ctx,
            &raw mut (*pd).s,
            0 as u_int,
            0 as u_int,
            (*pd).sx.wrapping_sub(2 as u_int),
            (*pd).sy.wrapping_sub(2 as u_int),
        );
    }
    screen_write_stop(&raw mut ctx);
    memcpy(
        &raw mut defaults as *mut ::core::ffi::c_void,
        &raw mut (*pd).defaults as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    if defaults.fg == 8 as ::core::ffi::c_int {
        defaults.fg = (*pd).palette.fg;
    }
    if defaults.bg == 8 as ::core::ffi::c_int {
        defaults.bg = (*pd).palette.bg;
    }
    style_ctx.defaults = &raw mut defaults;
    style_ctx.palette = &raw mut (*pd).palette;
    style_ctx.dim = 0 as u_int;
    style_ctx.hyperlinks = s.hyperlinks;
    (*c).overlay_check = None;
    (*c).overlay_data = NULL;
    i = 0 as u_int;
    while i < (*pd).sy {
        tty_draw_line(
            tty,
            &raw mut s,
            0 as u_int,
            i,
            (*pd).sx,
            px,
            py.wrapping_add(i),
            &raw mut style_ctx,
        );
        i = i.wrapping_add(1);
    }
    screen_free(&raw mut s);
    (*c).overlay_check = Some(
        popup_check_cb
            as unsafe extern "C" fn(
                *mut client,
                *mut ::core::ffi::c_void,
                u_int,
                u_int,
                u_int,
            ) -> *mut visible_ranges,
    ) as overlay_check_cb;
    (*c).overlay_data = pd as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn popup_free_cb(mut c: *mut client, mut data: *mut ::core::ffi::c_void) {
    let mut pd: *mut popup_data = data as *mut popup_data;
    let mut item: *mut cmdq_item = (*pd).item;
    if (*pd).cb.is_some() {
        (*pd).cb.expect("non-null function pointer")((*pd).status, (*pd).arg);
    }
    if !item.is_null() {
        if !cmdq_get_client(item).is_null() && (*cmdq_get_client(item)).session.is_null() {
            (*cmdq_get_client(item)).retval = (*pd).status;
        }
        cmdq_continue(item);
    }
    popup_free(pd);
}
unsafe extern "C" fn popup_resize_cb(mut c: *mut client, mut data: *mut ::core::ffi::c_void) {
    let mut pd: *mut popup_data = data as *mut popup_data;
    let mut tty: *mut tty = &raw mut (*c).tty;
    if pd.is_null() {
        return;
    }
    if (*pd).psy > (*tty).sy {
        (*pd).sy = (*tty).sy;
    } else {
        (*pd).sy = (*pd).psy;
    }
    if (*pd).psx > (*tty).sx {
        (*pd).sx = (*tty).sx;
    } else {
        (*pd).sx = (*pd).psx;
    }
    if (*pd).ppy.wrapping_add((*pd).sy) > (*tty).sy {
        (*pd).py = (*tty).sy.wrapping_sub((*pd).sy);
    } else {
        (*pd).py = (*pd).ppy;
    }
    if (*pd).ppx.wrapping_add((*pd).sx) > (*tty).sx {
        (*pd).px = (*tty).sx.wrapping_sub((*pd).sx);
    } else {
        (*pd).px = (*pd).ppx;
    }
    if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
        screen_resize(
            &raw mut (*pd).s,
            (*pd).sx,
            (*pd).sy,
            0 as ::core::ffi::c_int,
        );
        if !(*pd).job.is_null() {
            job_resize((*pd).job, (*pd).sx, (*pd).sy);
        }
    } else if (*pd).sx > 2 as u_int && (*pd).sy > 2 as u_int {
        screen_resize(
            &raw mut (*pd).s,
            (*pd).sx.wrapping_sub(2 as u_int),
            (*pd).sy.wrapping_sub(2 as u_int),
            0 as ::core::ffi::c_int,
        );
        if !(*pd).job.is_null() {
            job_resize(
                (*pd).job,
                (*pd).sx.wrapping_sub(2 as u_int),
                (*pd).sy.wrapping_sub(2 as u_int),
            );
        }
    }
}
unsafe extern "C" fn popup_handle_drag(
    mut c: *mut client,
    mut pd: *mut popup_data,
    mut m: *mut mouse_event,
) {
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    if (*m).b & MOUSE_MASK_DRAG as u_int == 0 {
        (*pd).dragging = OFF;
    } else if (*pd).dragging as ::core::ffi::c_uint
        == MOVE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*m).x < (*pd).dx {
            px = 0 as u_int;
        } else if (*m).x.wrapping_sub((*pd).dx).wrapping_add((*pd).sx) > (*c).tty.sx {
            px = (*c).tty.sx.wrapping_sub((*pd).sx);
        } else {
            px = (*m).x.wrapping_sub((*pd).dx);
        }
        if (*m).y < (*pd).dy {
            py = 0 as u_int;
        } else if (*m).y.wrapping_sub((*pd).dy).wrapping_add((*pd).sy) > (*c).tty.sy {
            py = (*c).tty.sy.wrapping_sub((*pd).sy);
        } else {
            py = (*m).y.wrapping_sub((*pd).dy);
        }
        (*pd).px = px;
        (*pd).py = py;
        (*pd).dx = (*m).x.wrapping_sub((*pd).px);
        (*pd).dy = (*m).y.wrapping_sub((*pd).py);
        (*pd).ppx = px;
        (*pd).ppy = py;
        server_redraw_client(c);
    } else if (*pd).dragging as ::core::ffi::c_uint
        == SIZE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
            if (*m).x < (*pd).px.wrapping_add(1 as u_int) {
                return;
            }
            if (*m).y < (*pd).py.wrapping_add(1 as u_int) {
                return;
            }
        } else {
            if (*m).x < (*pd).px.wrapping_add(3 as u_int) {
                return;
            }
            if (*m).y < (*pd).py.wrapping_add(3 as u_int) {
                return;
            }
        }
        (*pd).sx = (*m).x.wrapping_sub((*pd).px);
        (*pd).sy = (*m).y.wrapping_sub((*pd).py);
        (*pd).psx = (*pd).sx;
        (*pd).psy = (*pd).sy;
        if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
            screen_resize(
                &raw mut (*pd).s,
                (*pd).sx,
                (*pd).sy,
                0 as ::core::ffi::c_int,
            );
            if !(*pd).job.is_null() {
                job_resize((*pd).job, (*pd).sx, (*pd).sy);
            }
        } else {
            screen_resize(
                &raw mut (*pd).s,
                (*pd).sx.wrapping_sub(2 as u_int),
                (*pd).sy.wrapping_sub(2 as u_int),
                0 as ::core::ffi::c_int,
            );
            if !(*pd).job.is_null() {
                job_resize(
                    (*pd).job,
                    (*pd).sx.wrapping_sub(2 as u_int),
                    (*pd).sy.wrapping_sub(2 as u_int),
                );
            }
        }
        server_redraw_client(c);
    }
}
unsafe extern "C" fn popup_key_cb(
    mut c: *mut client,
    mut data: *mut ::core::ffi::c_void,
    mut event: *mut key_event,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut pd: *mut popup_data = data as *mut popup_data;
    let mut m: *mut mouse_event = &raw mut (*event).m;
    let mut buf: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut border: C2RustUnnamed_40 = NONE;
    if (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        if (*pd).dragging as ::core::ffi::c_uint != OFF as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            popup_handle_drag(c, pd, m);
            current_block = 9022331712714349549;
        } else {
            if (*m).x < (*pd).px
                || (*m).x > (*pd).px.wrapping_add((*pd).sx).wrapping_sub(1 as u_int)
                || (*m).y < (*pd).py
                || (*m).y > (*pd).py.wrapping_add((*pd).sy).wrapping_sub(1 as u_int)
            {
                return 0 as ::core::ffi::c_int;
            }
            if (*pd).border_lines as ::core::ffi::c_int != BOX_LINES_NONE as ::core::ffi::c_int {
                if (*m).x == (*pd).px {
                    border = LEFT;
                } else if (*m).x == (*pd).px.wrapping_add((*pd).sx).wrapping_sub(1 as u_int) {
                    border = RIGHT;
                } else if (*m).y == (*pd).py {
                    border = TOP;
                } else if (*m).y == (*pd).py.wrapping_add((*pd).sy).wrapping_sub(1 as u_int) {
                    border = BOTTOM;
                }
            }
            if (*m).b & MOUSE_MASK_MODIFIERS as u_int == 0 as u_int
                && (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_3 as u_int
                && (border as ::core::ffi::c_uint
                    == LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
                    || border as ::core::ffi::c_uint
                        == TOP as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                current_block = 9022331712714349549;
            } else if (*m).b & MOUSE_MASK_MODIFIERS as u_int == MOUSE_MASK_META as u_int
                || border as ::core::ffi::c_uint
                    != NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                    && (*m).lb & MOUSE_MASK_DRAG as u_int == 0
            {
                if (*m).b & MOUSE_MASK_DRAG as u_int == 0 {
                    current_block = 9022331712714349549;
                } else {
                    if (*m).lb & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_1 as u_int {
                        (*pd).dragging = MOVE;
                    } else if (*m).lb & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_3 as u_int {
                        (*pd).dragging = SIZE;
                    }
                    (*pd).dx = (*m).lx.wrapping_sub((*pd).px);
                    (*pd).dy = (*m).ly.wrapping_sub((*pd).py);
                    current_block = 9022331712714349549;
                }
            } else {
                current_block = 13472856163611868459;
            }
        }
        match current_block {
            13472856163611868459 => {}
            _ => {
                (*pd).lx = (*m).x;
                (*pd).ly = (*m).y;
                (*pd).lb = (*m).b;
                return 0 as ::core::ffi::c_int;
            }
        }
    }
    if ((*pd).flags & (POPUP_CLOSEEXIT | POPUP_CLOSEEXITZERO) == 0 as ::core::ffi::c_int
        || (*pd).job.is_null())
        && ((*event).key == '\u{1b}' as i32 as key_code
            || (*event).key == 'c' as i32 as ::core::ffi::c_ulonglong | KEYC_CTRL)
    {
        return 1 as ::core::ffi::c_int;
    }
    if (*pd).job.is_null()
        && (*pd).flags & POPUP_CLOSEANYKEY != 0
        && !((*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int)
        && !((*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            == (KEYC_TYPE_FUNCTION as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && ((*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                == KEYC_PASTE_START as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == KEYC_PASTE_END as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong))
    {
        return 1 as ::core::ffi::c_int;
    }
    if !(*pd).job.is_null() {
        if (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int
        {
            if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
                px = (*m).x.wrapping_sub((*pd).px);
                py = (*m).y.wrapping_sub((*pd).py);
            } else {
                px = (*m).x.wrapping_sub((*pd).px).wrapping_sub(1 as u_int);
                py = (*m).y.wrapping_sub((*pd).py).wrapping_sub(1 as u_int);
            }
            if input_key_get_mouse(&raw mut (*pd).s, m, px, py, &raw mut buf, &raw mut len) == 0 {
                return 0 as ::core::ffi::c_int;
            }
            bufferevent_write(
                job_get_event((*pd).job),
                buf as *const ::core::ffi::c_void,
                len,
            );
            return 0 as ::core::ffi::c_int;
        }
        input_key(&raw mut (*pd).s, job_get_event((*pd).job), (*event).key);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn popup_job_update_cb(mut job: *mut job) {
    let mut pd: *mut popup_data = job_get_data(job) as *mut popup_data;
    let mut evb: *mut evbuffer = (*job_get_event(job)).input;
    let mut c: *mut client = (*pd).c;
    let mut s: *mut screen = &raw mut (*pd).s;
    let mut data: *mut ::core::ffi::c_void =
        evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t) as *mut ::core::ffi::c_void;
    let mut size: size_t = evbuffer_get_length(evb);
    if size == 0 as size_t {
        return;
    }
    (*c).overlay_check = None;
    (*c).overlay_data = NULL;
    input_parse_screen(
        (*pd).ictx,
        s,
        Some(popup_init_ctx_cb as unsafe extern "C" fn(*mut screen_write_ctx, *mut tty_ctx) -> ()),
        pd as *mut ::core::ffi::c_void,
        data as *const u_char,
        size,
    );
    (*c).overlay_check = Some(
        popup_check_cb
            as unsafe extern "C" fn(
                *mut client,
                *mut ::core::ffi::c_void,
                u_int,
                u_int,
                u_int,
            ) -> *mut visible_ranges,
    ) as overlay_check_cb;
    (*c).overlay_data = pd as *mut ::core::ffi::c_void;
    evbuffer_drain(evb, size);
}
unsafe extern "C" fn popup_job_complete_cb(mut job: *mut job) {
    let mut pd: *mut popup_data = job_get_data(job) as *mut popup_data;
    let mut status: ::core::ffi::c_int = 0;
    status = job_get_status((*pd).job);
    if status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        (*pd).status = (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
    } else if ((status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
        as ::core::ffi::c_schar as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int
        > 0 as ::core::ffi::c_int
    {
        (*pd).status = status & 0x7f as ::core::ffi::c_int;
    } else {
        (*pd).status = 0 as ::core::ffi::c_int;
    }
    (*pd).job = ::core::ptr::null_mut::<job>();
    if (*pd).flags & POPUP_CLOSEEXIT != 0
        || (*pd).flags & POPUP_CLOSEEXITZERO != 0 && (*pd).status == 0 as ::core::ffi::c_int
    {
        server_client_clear_overlay((*pd).c);
    }
}
#[no_mangle]
pub unsafe extern "C" fn popup_present(mut c: *mut client) -> ::core::ffi::c_int {
    return ((*c).overlay_draw
        == Some(popup_draw_cb as unsafe extern "C" fn(*mut client, *mut ::core::ffi::c_void) -> ()))
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn popup_modify(
    mut c: *mut client,
    mut title: *const ::core::ffi::c_char,
    mut style: *const ::core::ffi::c_char,
    mut border_style: *const ::core::ffi::c_char,
    mut lines: box_lines,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut pd: *mut popup_data = (*c).overlay_data as *mut popup_data;
    let mut sytmp: style = style {
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
    if !title.is_null() {
        if !(*pd).title.is_null() {
            free((*pd).title as *mut ::core::ffi::c_void);
        }
        (*pd).title = xstrdup(title);
    }
    if !border_style.is_null() {
        free((*pd).border_style as *mut ::core::ffi::c_void);
        (*pd).border_style = xstrdup(border_style);
        style_set(&raw mut sytmp, &raw mut (*pd).border_cell);
        if style_parse(&raw mut sytmp, &raw mut (*pd).border_cell, border_style)
            == 0 as ::core::ffi::c_int
        {
            (*pd).border_cell.fg = sytmp.gc.fg;
            (*pd).border_cell.bg = sytmp.gc.bg;
        }
    }
    if !style.is_null() {
        free((*pd).style as *mut ::core::ffi::c_void);
        (*pd).style = xstrdup(style);
        style_set(&raw mut sytmp, &raw mut (*pd).defaults);
        if style_parse(&raw mut sytmp, &raw mut (*pd).defaults, style) == 0 as ::core::ffi::c_int {
            (*pd).defaults.fg = sytmp.gc.fg;
            (*pd).defaults.bg = sytmp.gc.bg;
        }
    }
    if lines as ::core::ffi::c_int != BOX_LINES_DEFAULT as ::core::ffi::c_int {
        if lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int
            && (*pd).border_lines as ::core::ffi::c_int != lines as ::core::ffi::c_int
        {
            screen_resize(
                &raw mut (*pd).s,
                (*pd).sx,
                (*pd).sy,
                1 as ::core::ffi::c_int,
            );
            job_resize((*pd).job, (*pd).sx, (*pd).sy);
        } else if (*pd).border_lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int
            && (*pd).border_lines as ::core::ffi::c_int != lines as ::core::ffi::c_int
        {
            screen_resize(
                &raw mut (*pd).s,
                (*pd).sx.wrapping_sub(2 as u_int),
                (*pd).sy.wrapping_sub(2 as u_int),
                1 as ::core::ffi::c_int,
            );
            job_resize(
                (*pd).job,
                (*pd).sx.wrapping_sub(2 as u_int),
                (*pd).sy.wrapping_sub(2 as u_int),
            );
        }
        (*pd).border_lines = lines;
        tty_resize(&raw mut (*c).tty);
    }
    if flags != -(1 as ::core::ffi::c_int) {
        (*pd).flags = flags;
    }
    server_redraw_client(c);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn popup_display(
    mut flags: ::core::ffi::c_int,
    mut lines: box_lines,
    mut item: *mut cmdq_item,
    mut px: u_int,
    mut py: u_int,
    mut sx: u_int,
    mut sy: u_int,
    mut env: *mut environ,
    mut shellcmd: *const ::core::ffi::c_char,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
    mut cwd: *const ::core::ffi::c_char,
    mut title: *const ::core::ffi::c_char,
    mut c: *mut client,
    mut s: *mut session,
    mut style: *const ::core::ffi::c_char,
    mut border_style: *const ::core::ffi::c_char,
    mut cb: popup_close_cb,
    mut arg: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut pd: *mut popup_data = ::core::ptr::null_mut::<popup_data>();
    let mut jx: u_int = 0;
    let mut jy: u_int = 0;
    let mut o: *mut options = ::core::ptr::null_mut::<options>();
    let mut sytmp: style = style {
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
    if !s.is_null() {
        o = (*(*(*s).curw).window).options;
    } else {
        o = (*(*(*(*c).session).curw).window).options;
    }
    if lines as ::core::ffi::c_int == BOX_LINES_DEFAULT as ::core::ffi::c_int {
        lines = options_get_number(
            o,
            b"popup-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
        ) as box_lines;
    }
    if lines as ::core::ffi::c_int == BOX_LINES_NONE as ::core::ffi::c_int {
        if sx < 1 as u_int || sy < 1 as u_int {
            return -(1 as ::core::ffi::c_int);
        }
        jx = sx;
        jy = sy;
    } else {
        if sx < 3 as u_int || sy < 3 as u_int {
            return -(1 as ::core::ffi::c_int);
        }
        jx = sx.wrapping_sub(2 as u_int);
        jy = sy.wrapping_sub(2 as u_int);
    }
    if (*c).tty.sx < sx || (*c).tty.sy < sy {
        return -(1 as ::core::ffi::c_int);
    }
    pd = xcalloc(1 as size_t, ::core::mem::size_of::<popup_data>() as size_t) as *mut popup_data;
    (*pd).item = item;
    (*pd).flags = flags;
    if !title.is_null() {
        (*pd).title = xstrdup(title);
    }
    if !style.is_null() {
        (*pd).style = xstrdup(style);
    }
    if !border_style.is_null() {
        (*pd).border_style = xstrdup(border_style);
    }
    (*pd).c = c;
    (*(*pd).c).references += 1;
    (*pd).cb = cb;
    (*pd).arg = arg;
    (*pd).status = 128 as ::core::ffi::c_int + SIGHUP;
    (*pd).border_lines = lines;
    memcpy(
        &raw mut (*pd).border_cell as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut (*pd).border_cell,
        o,
        b"popup-border-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    if !border_style.is_null() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(&raw mut sytmp, &raw mut (*pd).border_cell, border_style)
            == 0 as ::core::ffi::c_int
        {
            (*pd).border_cell.fg = sytmp.gc.fg;
            (*pd).border_cell.bg = sytmp.gc.bg;
        }
    }
    (*pd).border_cell.attr = 0 as u_short;
    screen_init(&raw mut (*pd).s, jx, jy, 0 as u_int);
    screen_set_default_cursor(&raw mut (*pd).s, global_w_options);
    colour_palette_init(&raw mut (*pd).palette);
    colour_palette_from_option(&raw mut (*pd).palette, global_w_options);
    memcpy(
        &raw mut (*pd).defaults as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut (*pd).defaults,
        o,
        b"popup-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    if !style.is_null() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(&raw mut sytmp, &raw mut (*pd).defaults, style) == 0 as ::core::ffi::c_int {
            (*pd).defaults.fg = sytmp.gc.fg;
            (*pd).defaults.bg = sytmp.gc.bg;
        }
    }
    (*pd).defaults.attr = 0 as u_short;
    (*pd).px = px;
    (*pd).py = py;
    (*pd).sx = sx;
    (*pd).sy = sy;
    (*pd).ppx = px;
    (*pd).ppy = py;
    (*pd).psx = sx;
    (*pd).psy = sy;
    (*pd).job = job_run(
        shellcmd,
        argc,
        argv,
        env,
        s,
        cwd,
        Some(popup_job_update_cb as unsafe extern "C" fn(*mut job) -> ()),
        Some(popup_job_complete_cb as unsafe extern "C" fn(*mut job) -> ()),
        None,
        pd as *mut ::core::ffi::c_void,
        JOB_NOWAIT | JOB_PTY | JOB_KEEPWRITE | JOB_DEFAULTSHELL,
        jx as ::core::ffi::c_int,
        jy as ::core::ffi::c_int,
    );
    if (*pd).job.is_null() {
        popup_free(pd);
        return -(1 as ::core::ffi::c_int);
    }
    (*pd).ictx = input_init(
        ::core::ptr::null_mut::<window_pane>(),
        job_get_event((*pd).job),
        &raw mut (*pd).palette,
        c,
    );
    server_client_set_overlay(
        c,
        0 as u_int,
        Some(
            popup_check_cb
                as unsafe extern "C" fn(
                    *mut client,
                    *mut ::core::ffi::c_void,
                    u_int,
                    u_int,
                    u_int,
                ) -> *mut visible_ranges,
        ),
        Some(
            popup_mode_cb
                as unsafe extern "C" fn(
                    *mut client,
                    *mut ::core::ffi::c_void,
                    *mut u_int,
                    *mut u_int,
                ) -> *mut screen,
        ),
        Some(popup_draw_cb as unsafe extern "C" fn(*mut client, *mut ::core::ffi::c_void) -> ()),
        Some(
            popup_key_cb
                as unsafe extern "C" fn(
                    *mut client,
                    *mut ::core::ffi::c_void,
                    *mut key_event,
                ) -> ::core::ffi::c_int,
        ),
        Some(popup_free_cb as unsafe extern "C" fn(*mut client, *mut ::core::ffi::c_void) -> ()),
        Some(popup_resize_cb as unsafe extern "C" fn(*mut client, *mut ::core::ffi::c_void) -> ()),
        pd as *mut ::core::ffi::c_void,
    );
    return 0 as ::core::ffi::c_int;
}

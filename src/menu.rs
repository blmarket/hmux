use crate::src::shared::client::*;
use crate::src::shared::layout::*;
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
extern "C" {
    pub type args;
    pub type tmuxpeer;
    pub type hyperlinks;
    pub type screen_write_cline;
    pub type screen_sel;
    pub type screen_titles;
    pub type environ;
    pub type options;
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
    pub type cmdq_state;
    pub type screen_write_citem;
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
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn format_free(_: *mut format_tree);
    fn format_single(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut ::core::ffi::c_char;
    fn format_single_from_state(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
        _: *mut client,
        _: *mut cmd_find_state,
    ) -> *mut ::core::ffi::c_char;
    fn format_create_defaults(
        _: *mut cmdq_item,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut format_tree;
    fn format_width(_: *const ::core::ffi::c_char) -> u_int;
    fn format_trim_right(_: *const ::core::ffi::c_char, _: u_int) -> *mut ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn cmd_find_clear_state(_: *mut cmd_find_state, _: ::core::ffi::c_int);
    fn cmd_find_copy_state(_: *mut cmd_find_state, _: *mut cmd_find_state);
    fn cmd_find_from_window(
        _: *mut cmd_find_state,
        _: *mut window,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_parse_and_append(
        _: *const ::core::ffi::c_char,
        _: *mut cmd_parse_input,
        _: *mut client,
        _: *mut cmdq_state,
        _: *mut *mut ::core::ffi::c_char,
    ) -> cmd_parse_status;
    fn cmdq_new_state(
        _: *mut cmd_find_state,
        _: *mut key_event,
        _: ::core::ffi::c_int,
    ) -> *mut cmdq_state;
    fn cmdq_free_state(_: *mut cmdq_state);
    fn cmdq_get_event(_: *mut cmdq_item) -> *mut key_event;
    fn cmdq_get_error(_: *const ::core::ffi::c_char) -> *mut cmdq_item;
    fn cmdq_append(_: *mut client, _: *mut cmdq_item) -> *mut cmdq_item;
    fn key_string_lookup_key(_: key_code, _: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn server_redraw_window(_: *mut window);
    fn server_redraw_window_menu(_: *mut window);
    static grid_default_cell: grid_cell;
    fn screen_write_start(_: *mut screen_write_ctx, _: *mut screen);
    fn screen_write_stop(_: *mut screen_write_ctx);
    fn screen_write_menu(
        _: *mut screen_write_ctx,
        _: *mut menu,
        _: ::core::ffi::c_int,
        _: box_lines,
        _: *const grid_cell,
        _: *const grid_cell,
        _: *const grid_cell,
    );
    fn screen_write_box(
        _: *mut screen_write_ctx,
        _: u_int,
        _: u_int,
        _: box_lines,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
    );
    fn screen_write_clearscreen(_: *mut screen_write_ctx, _: u_int);
    fn redraw_invalidate_scene(_: *mut window);
    fn screen_init(_: *mut screen, _: u_int, _: u_int, _: u_int);
    fn screen_free(_: *mut screen);
    fn window_update_focus(_: *mut window);
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
pub struct menu_data {
    pub w: *mut window,
    pub flags: ::core::ffi::c_int,
    pub style: *mut ::core::ffi::c_char,
    pub border_style: *mut ::core::ffi::c_char,
    pub selected_style: *mut ::core::ffi::c_char,
    pub style_gc: grid_cell,
    pub border_style_gc: grid_cell,
    pub selected_style_gc: grid_cell,
    pub border_lines: box_lines,
    pub fs: cmd_find_state,
    pub key: key_code,
    pub m: mouse_event,
    pub s: screen,
    pub px: u_int,
    pub py: u_int,
    pub menu: *mut menu,
    pub choice: ::core::ffi::c_int,
    pub cb: menu_choice_cb,
    pub data: *mut ::core::ffi::c_void,
}
pub type menu_choice_cb =
    Option<unsafe extern "C" fn(*mut menu, u_int, key_code, *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct menu {
    pub title: *const ::core::ffi::c_char,
    pub items: *mut menu_item,
    pub count: u_int,
    pub width: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct menu_item {
    pub name: *const ::core::ffi::c_char,
    pub key: key_code,
    pub command: *const ::core::ffi::c_char,
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
    pub modes: C2RustUnnamed_26,
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
    pub entry: C2RustUnnamed_25,
    pub sentry: C2RustUnnamed_24,
    pub zentry: C2RustUnnamed_23,
    pub tree_entry: C2RustUnnamed_22,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_22 {
    pub rbe_left: *mut window_pane,
    pub rbe_right: *mut window_pane,
    pub rbe_parent: *mut window_pane,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_23 {
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
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
    pub entry: C2RustUnnamed_27,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_27 {
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
    pub entry: C2RustUnnamed_29,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_29 {
    pub tqe_next: *mut window_pane_resize,
    pub tqe_prev: *mut *mut window_pane_resize,
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
    pub entry: C2RustUnnamed_30,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_30 {
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
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const MODE_CURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MODE_MOUSE_BUTTON: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const MODE_MOUSE_ALL: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const MOUSE_MASK_BUTTONS: ::core::ffi::c_int = 195 as ::core::ffi::c_int;
pub const MOUSE_MASK_DRAG: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const MOUSE_WHEEL_UP: ::core::ffi::c_int = 64 as ::core::ffi::c_int;
pub const MOUSE_WHEEL_DOWN: ::core::ffi::c_int = 65 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MENU_NOMOUSE: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MENU_TAB: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MENU_STAYOPEN: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn menu_add_items(
    mut menu: *mut menu,
    mut items: *const menu_item,
    mut qitem: *mut cmdq_item,
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
) {
    let mut loop_0: *const menu_item = ::core::ptr::null::<menu_item>();
    loop_0 = items;
    while !(*loop_0).name.is_null() {
        menu_add_item(menu, loop_0, qitem, c, fs);
        loop_0 = loop_0.offset(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn menu_add_item(
    mut menu: *mut menu,
    mut item: *const menu_item,
    mut qitem: *mut cmdq_item,
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
) {
    let mut new_item: *mut menu_item = ::core::ptr::null_mut::<menu_item>();
    let mut key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut suffix: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut trimmed: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut width: u_int = 0;
    let mut max_width: u_int = 0;
    let mut line: ::core::ffi::c_int = 0;
    let mut keylen: size_t = 0;
    let mut slen: size_t = 0;
    line = (item.is_null()
        || (*item).name.is_null()
        || *(*item).name as ::core::ffi::c_int == '\0' as i32) as ::core::ffi::c_int;
    if line != 0 && (*menu).count == 0 as u_int {
        return;
    }
    if line != 0
        && (*(*menu)
            .items
            .offset((*menu).count.wrapping_sub(1 as u_int) as isize))
        .name
        .is_null()
    {
        return;
    }
    (*menu).items = xreallocarray(
        (*menu).items as *mut ::core::ffi::c_void,
        (*menu).count.wrapping_add(1 as u_int) as size_t,
        ::core::mem::size_of::<menu_item>() as size_t,
    ) as *mut menu_item;
    let fresh0 = (*menu).count;
    (*menu).count = (*menu).count.wrapping_add(1);
    new_item = (*menu).items.offset(fresh0 as isize) as *mut menu_item;
    memset(
        new_item as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<menu_item>() as size_t,
    );
    if line != 0 {
        return;
    }
    if !fs.is_null() {
        s = format_single_from_state(qitem, (*item).name, c, fs);
    } else {
        s = format_single(
            qitem,
            (*item).name,
            c,
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
    }
    if *s as ::core::ffi::c_int == '\0' as i32 {
        free(s as *mut ::core::ffi::c_void);
        (*menu).count = (*menu).count.wrapping_sub(1);
        return;
    }
    max_width = (*c).tty.sx.wrapping_sub(4 as u_int);
    slen = strlen(s);
    if *s as ::core::ffi::c_int != '-' as i32
        && (*item).key != KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code
        && (*item).key != KEYC_NONE as ::core::ffi::c_ulong as key_code
    {
        key = key_string_lookup_key((*item).key, 0 as ::core::ffi::c_int);
        keylen = strlen(key).wrapping_add(3 as size_t);
        if keylen <= max_width.wrapping_div(4 as u_int) as size_t {
            max_width = (max_width as size_t).wrapping_sub(keylen) as u_int as u_int;
        } else if keylen >= max_width as size_t
            || slen >= (max_width as size_t).wrapping_sub(keylen)
        {
            key = ::core::ptr::null::<::core::ffi::c_char>();
        }
    }
    if slen > max_width as size_t {
        max_width = max_width.wrapping_sub(1);
        suffix = b">\0" as *const u8 as *const ::core::ffi::c_char;
    }
    trimmed = format_trim_right(s, max_width);
    if !key.is_null() {
        xasprintf(
            &raw mut name,
            b"%s%s#[default] #[align=right](%s)\0" as *const u8 as *const ::core::ffi::c_char,
            trimmed,
            suffix,
            key,
        );
    } else {
        xasprintf(
            &raw mut name,
            b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
            trimmed,
            suffix,
        );
    }
    free(trimmed as *mut ::core::ffi::c_void);
    (*new_item).name = name;
    free(s as *mut ::core::ffi::c_void);
    cmd = (*item).command;
    if !cmd.is_null() {
        if !fs.is_null() {
            s = format_single_from_state(qitem, cmd, c, fs);
        } else {
            s = format_single(
                qitem,
                cmd,
                c,
                ::core::ptr::null_mut::<session>(),
                ::core::ptr::null_mut::<winlink>(),
                ::core::ptr::null_mut::<window_pane>(),
            );
        }
    } else {
        s = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    (*new_item).command = s;
    (*new_item).key = (*item).key;
    width = format_width((*new_item).name);
    if *(*new_item).name as ::core::ffi::c_int == '-' as i32 {
        width = width.wrapping_sub(1);
    }
    if width > (*menu).width {
        (*menu).width = width;
    }
}
#[no_mangle]
pub unsafe extern "C" fn menu_create(mut title: *const ::core::ffi::c_char) -> *mut menu {
    let mut menu: *mut menu = ::core::ptr::null_mut::<menu>();
    menu = xcalloc(1 as size_t, ::core::mem::size_of::<menu>() as size_t) as *mut menu;
    (*menu).title = xstrdup(title);
    (*menu).width = format_width(title);
    return menu;
}
#[no_mangle]
pub unsafe extern "C" fn menu_free(mut menu: *mut menu) {
    let mut i: u_int = 0;
    if menu.is_null() {
        return;
    }
    i = 0 as u_int;
    while i < (*menu).count {
        free((*(*menu).items.offset(i as isize)).name as *mut ::core::ffi::c_void);
        free((*(*menu).items.offset(i as isize)).command as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    free((*menu).items as *mut ::core::ffi::c_void);
    free((*menu).title as *mut ::core::ffi::c_void);
    free(menu as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn menu_reapply_styles(mut md: *mut menu_data) {
    let mut o: *mut options = (*(*md).w).options;
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
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        (*md).fs.s,
        (*md).fs.wl,
        (*md).fs.wp,
    );
    memcpy(
        &raw mut (*md).style_gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut (*md).style_gc,
        o,
        b"menu-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    if !(*md).style.is_null() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(&raw mut sytmp, &raw mut (*md).style_gc, (*md).style)
            == 0 as ::core::ffi::c_int
        {
            (*md).style_gc.fg = sytmp.gc.fg;
            (*md).style_gc.bg = sytmp.gc.bg;
        }
    }
    memcpy(
        &raw mut (*md).selected_style_gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut (*md).selected_style_gc,
        o,
        b"menu-selected-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    if !(*md).selected_style.is_null() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(
            &raw mut sytmp,
            &raw mut (*md).selected_style_gc,
            (*md).selected_style,
        ) == 0 as ::core::ffi::c_int
        {
            (*md).selected_style_gc.fg = sytmp.gc.fg;
            (*md).selected_style_gc.bg = sytmp.gc.bg;
        }
    }
    memcpy(
        &raw mut (*md).border_style_gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        &raw mut (*md).border_style_gc,
        o,
        b"menu-border-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    if !(*md).border_style.is_null() {
        style_set(&raw mut sytmp, &raw const grid_default_cell);
        if style_parse(
            &raw mut sytmp,
            &raw mut (*md).border_style_gc,
            (*md).border_style,
        ) == 0 as ::core::ffi::c_int
        {
            (*md).border_style_gc.fg = sytmp.gc.fg;
            (*md).border_style_gc.bg = sytmp.gc.bg;
        }
    }
    format_free(ft);
}
#[no_mangle]
pub unsafe extern "C" fn menu_update(mut md: *mut menu_data) {
    let mut s: *mut screen = &raw mut (*md).s;
    let mut menu: *mut menu = (*md).menu;
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
    menu_reapply_styles(md);
    screen_write_start(&raw mut ctx, s);
    screen_write_clearscreen(&raw mut ctx, 8 as u_int);
    if (*md).border_lines as ::core::ffi::c_int != BOX_LINES_NONE as ::core::ffi::c_int {
        screen_write_box(
            &raw mut ctx,
            (*menu).width.wrapping_add(4 as u_int),
            (*menu).count.wrapping_add(2 as u_int),
            (*md).border_lines,
            &raw mut (*md).border_style_gc,
            (*menu).title,
        );
    }
    screen_write_menu(
        &raw mut ctx,
        menu,
        (*md).choice,
        (*md).border_lines,
        &raw mut (*md).style_gc,
        &raw mut (*md).border_style_gc,
        &raw mut (*md).selected_style_gc,
    );
    screen_write_stop(&raw mut ctx);
}
unsafe extern "C" fn menu_free_data(mut md: *mut menu_data) {
    if !md.is_null() {
        if (*md).cb.is_some() {
            (*md).cb.expect("non-null function pointer")(
                (*md).menu,
                UINT_MAX,
                KEYC_NONE as ::core::ffi::c_ulong as key_code,
                (*md).data,
            );
        }
        screen_free(&raw mut (*md).s);
        menu_free((*md).menu);
        free((*md).style as *mut ::core::ffi::c_void);
        free((*md).selected_style as *mut ::core::ffi::c_void);
        free((*md).border_style as *mut ::core::ffi::c_void);
        free(md as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn menu_close(mut w: *mut window) {
    if !(*w).menu.is_null() {
        menu_free_data((*w).menu);
        (*w).menu = ::core::ptr::null_mut::<menu_data>();
        redraw_invalidate_scene(w);
        window_update_focus(w);
        server_redraw_window(w);
    }
}
#[no_mangle]
pub unsafe extern "C" fn menu_destroy(mut w: *mut window) {
    menu_free_data((*w).menu);
    (*w).menu = ::core::ptr::null_mut::<menu_data>();
}
#[no_mangle]
pub unsafe extern "C" fn menu_get_cursor(
    mut md: *mut menu_data,
    mut cx: *mut u_int,
    mut cy: *mut u_int,
) {
    *cx = (*md).px.wrapping_add(2 as u_int);
    if (*md).choice == -(1 as ::core::ffi::c_int) {
        *cy = (*md).py;
    } else {
        *cy = (*md)
            .py
            .wrapping_add(1 as u_int)
            .wrapping_add((*md).choice as u_int);
    };
}
#[no_mangle]
pub unsafe extern "C" fn menu_screen(mut md: *mut menu_data) -> *mut screen {
    return &raw mut (*md).s;
}
#[no_mangle]
pub unsafe extern "C" fn menu_width(mut md: *mut menu_data) -> u_int {
    return (*(*md).menu).width.wrapping_add(4 as u_int);
}
#[no_mangle]
pub unsafe extern "C" fn menu_height(mut md: *mut menu_data) -> u_int {
    return (*(*md).menu).count.wrapping_add(2 as u_int);
}
#[no_mangle]
pub unsafe extern "C" fn menu_x(mut md: *mut menu_data) -> u_int {
    return (*md).px;
}
#[no_mangle]
pub unsafe extern "C" fn menu_y(mut md: *mut menu_data) -> u_int {
    return (*md).py;
}
#[no_mangle]
pub unsafe extern "C" fn menu_key(
    mut c: *mut client,
    mut md: *mut menu_data,
    mut event: *mut key_event,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut menu: *mut menu = (*md).menu;
    let mut m: *mut mouse_event = &raw mut (*event).m;
    let mut i: u_int = 0;
    let mut n: ::core::ffi::c_int = (*menu).count as ::core::ffi::c_int;
    let mut old: ::core::ffi::c_int = (*md).choice;
    let mut move_0: ::core::ffi::c_int = 0;
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut item: *const menu_item = ::core::ptr::null::<menu_item>();
    let mut saved_event: key_event = key_event {
        client: ::core::ptr::null_mut::<client>(),
        key: 0,
        m: mouse_event {
            valid: 0,
            ignore: 0,
            key: 0,
            statusat: 0,
            statuslines: 0,
            x: 0,
            y: 0,
            b: 0,
            lx: 0,
            ly: 0,
            lb: 0,
            ox: 0,
            oy: 0,
            s: 0,
            w: 0,
            wp: 0,
            sgr_type: 0,
            sgr_b: 0,
        },
        buf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        len: 0,
    };
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    let mut status: cmd_parse_status = CMD_PARSE_ERROR;
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: key_code = 0;
    if (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        move_0 = ((*m).b & MOUSE_MASK_DRAG as u_int != 0
            && (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int)
            as ::core::ffi::c_int;
        if (*md).flags & MENU_NOMOUSE != 0 {
            if (*m).b & MOUSE_MASK_BUTTONS as u_int != MOUSE_BUTTON_1 as u_int {
                return 1 as ::core::ffi::c_int;
            }
            return 0 as ::core::ffi::c_int;
        }
        if (*m).x < (*md).px
            || (*m).x
                > (*md)
                    .px
                    .wrapping_add(4 as u_int)
                    .wrapping_add((*menu).width)
            || (*m).y < (*md).py.wrapping_add(1 as u_int)
            || (*m).y
                > (*md)
                    .py
                    .wrapping_add(1 as u_int)
                    .wrapping_add(n as u_int)
                    .wrapping_sub(1 as u_int)
        {
            if !(*md).flags & MENU_STAYOPEN != 0 {
                if move_0 == 0 && (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int {
                    return 1 as ::core::ffi::c_int;
                }
            } else if !((*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int)
                && !((*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
                    || (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int)
                && (*m).b & MOUSE_MASK_DRAG as u_int == 0
            {
                return 1 as ::core::ffi::c_int;
            }
            if (*md).choice != -(1 as ::core::ffi::c_int) {
                (*md).choice = -(1 as ::core::ffi::c_int);
                server_redraw_window_menu((*md).w);
            }
            return 0 as ::core::ffi::c_int;
        }
        if !(*md).flags & MENU_STAYOPEN != 0 {
            if move_0 == 0 && (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int {
                current_block = 4062906366992634423;
            } else {
                current_block = 11194104282611034094;
            }
        } else if !((*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
            || (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int)
            && (*m).b & MOUSE_MASK_DRAG as u_int == 0
        {
            current_block = 4062906366992634423;
        } else {
            current_block = 11194104282611034094;
        }
        match current_block {
            4062906366992634423 => {}
            _ => {
                (*md).choice =
                    (*m).y.wrapping_sub((*md).py.wrapping_add(1 as u_int)) as ::core::ffi::c_int;
                if (*md).choice != old {
                    server_redraw_window_menu((*md).w);
                }
                return 0 as ::core::ffi::c_int;
            }
        }
    } else {
        i = 0 as u_int;
        loop {
            if !(i < n as u_int) {
                current_block = 14434620278749266018;
                break;
            }
            name = (*(*menu).items.offset(i as isize)).name;
            if !(name.is_null() || *name as ::core::ffi::c_int == '-' as i32) {
                key = ((*event).key as ::core::ffi::c_ulonglong & !KEYC_MASK_FLAGS) as key_code;
                if key
                    == (*(*menu).items.offset(i as isize)).key as ::core::ffi::c_ulonglong
                        & !KEYC_MASK_FLAGS
                {
                    (*md).choice = i as ::core::ffi::c_int;
                    current_block = 4062906366992634423;
                    break;
                }
            }
            i = i.wrapping_add(1);
        }
        match current_block {
            4062906366992634423 => {}
            _ => match (*event).key as ::core::ffi::c_ulonglong & !KEYC_MASK_FLAGS {
                8589934618 | 8589934619 | 107 => {
                    current_block = 18228927260028731949;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                8589934599 => {
                    current_block = 5908772614365292188;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                9 => {
                    current_block = 6621853080098874574;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                8589934620 | 106 => {
                    current_block = 17659224811226724223;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                8589934617 | 35184372088930 => {
                    current_block = 11150558847591123549;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                8589934616 => {
                    current_block = 5459197107747055838;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                103 | 8589934614 => {
                    current_block = 10426959295196933295;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                71 | 8589934615 => {
                    current_block = 4678245943260944876;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                13 => {}
                27 | 35184372088923 | 35184372088931 | 35184372088935 | 113 => {
                    current_block = 16418760376262495662;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
                35184372088934 | _ => {
                    current_block = 12369290732426379360;
                    match current_block {
                        10426959295196933295 => {
                            (*md).choice = 0 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != n - 1 as ::core::ffi::c_int
                            {
                                (*md).choice += 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        5459197107747055838 => {
                            if (*md).choice > n - 6 as ::core::ffi::c_int {
                                (*md).choice = n - 1 as ::core::ffi::c_int;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice += 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != n - 1 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == n - 1 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        11150558847591123549 => {
                            if (*md).choice < 6 as ::core::ffi::c_int {
                                (*md).choice = 0 as ::core::ffi::c_int;
                            } else {
                                i = 5 as u_int;
                                while i > 0 as u_int {
                                    (*md).choice -= 1;
                                    name = (*(*menu).items.offset((*md).choice as isize)).name;
                                    if (*md).choice != 0 as ::core::ffi::c_int
                                        && (!name.is_null()
                                            && *name as ::core::ffi::c_int != '-' as i32)
                                    {
                                        i = i.wrapping_sub(1);
                                    } else if (*md).choice == 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        6621853080098874574 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                if (*md).choice == n - 1 as ::core::ffi::c_int {
                                    return 1 as ::core::ffi::c_int;
                                }
                                current_block = 17659224811226724223;
                            }
                        }
                        5908772614365292188 => {
                            if !(*md).flags & MENU_TAB != 0 {
                                current_block = 12369290732426379360;
                            } else {
                                return 1 as ::core::ffi::c_int;
                            }
                        }
                        18228927260028731949 => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == 0 as ::core::ffi::c_int
                                {
                                    (*md).choice = n - 1 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice -= 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                        4678245943260944876 => {
                            (*md).choice = n - 1 as ::core::ffi::c_int;
                            name = (*(*menu).items.offset((*md).choice as isize)).name;
                            while (name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                && (*md).choice != 0 as ::core::ffi::c_int
                            {
                                (*md).choice -= 1;
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                            }
                            server_redraw_window_menu((*md).w);
                            current_block = 12369290732426379360;
                        }
                        16418760376262495662 => return 1 as ::core::ffi::c_int,
                        _ => {}
                    }
                    match current_block {
                        12369290732426379360 => return 0 as ::core::ffi::c_int,
                        _ => {
                            if old == -(1 as ::core::ffi::c_int) {
                                old = 0 as ::core::ffi::c_int;
                            }
                            loop {
                                if (*md).choice == -(1 as ::core::ffi::c_int)
                                    || (*md).choice == n - 1 as ::core::ffi::c_int
                                {
                                    (*md).choice = 0 as ::core::ffi::c_int;
                                } else {
                                    (*md).choice += 1;
                                }
                                name = (*(*menu).items.offset((*md).choice as isize)).name;
                                if !((name.is_null() || *name as ::core::ffi::c_int == '-' as i32)
                                    && (*md).choice != old)
                                {
                                    break;
                                }
                            }
                            server_redraw_window_menu((*md).w);
                            return 0 as ::core::ffi::c_int;
                        }
                    }
                }
            },
        }
    }
    if (*md).choice == -(1 as ::core::ffi::c_int) {
        return 1 as ::core::ffi::c_int;
    }
    item = (*menu).items.offset((*md).choice as isize) as *mut menu_item;
    if (*item).name.is_null() || *(*item).name as ::core::ffi::c_int == '-' as i32 {
        if (*md).flags & MENU_STAYOPEN != 0 {
            return 0 as ::core::ffi::c_int;
        }
        return 1 as ::core::ffi::c_int;
    }
    if (*md).cb.is_some() {
        (*md).cb.expect("non-null function pointer")(
            (*md).menu,
            (*md).choice as u_int,
            (*item).key,
            (*md).data,
        );
        (*md).cb = None;
        return 1 as ::core::ffi::c_int;
    }
    if (*md).key != KEYC_NONE as ::core::ffi::c_ulong as key_code {
        memset(
            &raw mut saved_event as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<key_event>() as size_t,
        );
        saved_event.key = (*md).key;
        memcpy(
            &raw mut saved_event.m as *mut ::core::ffi::c_void,
            &raw mut (*md).m as *const ::core::ffi::c_void,
            ::core::mem::size_of::<mouse_event>() as size_t,
        );
        event = &raw mut saved_event;
    } else {
        event = ::core::ptr::null_mut::<key_event>();
    }
    state = cmdq_new_state(&raw mut (*md).fs, event, 0 as ::core::ffi::c_int);
    status = cmd_parse_and_append(
        (*item).command,
        ::core::ptr::null_mut::<cmd_parse_input>(),
        c,
        state,
        &raw mut error,
    );
    if status as ::core::ffi::c_uint == CMD_PARSE_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        cmdq_append(c, cmdq_get_error(error));
        free(error as *mut ::core::ffi::c_void);
    }
    cmdq_free_state(state);
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn menu_resize(mut md: *mut menu_data, mut w: *mut window) {
    let mut nx: u_int = 0;
    let mut ny: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if md.is_null() {
        return;
    }
    nx = (*md).px;
    ny = (*md).py;
    sx = (*(*md).menu).width.wrapping_add(4 as u_int);
    sy = (*(*md).menu).count.wrapping_add(2 as u_int);
    if nx.wrapping_add(sx) > (*w).sx {
        if (*w).sx <= sx {
            nx = 0 as u_int;
        } else {
            nx = (*w).sx.wrapping_sub(sx);
        }
    }
    if ny.wrapping_add(sy) > (*w).sy {
        if (*w).sy <= sy {
            ny = 0 as u_int;
        } else {
            ny = (*w).sy.wrapping_sub(sy);
        }
    }
    (*md).px = nx;
    (*md).py = ny;
}
#[no_mangle]
pub unsafe extern "C" fn menu_display(
    mut menu: *mut menu,
    mut flags: ::core::ffi::c_int,
    mut starting_choice: ::core::ffi::c_int,
    mut item: *mut cmdq_item,
    mut px: u_int,
    mut py: u_int,
    mut c: *mut client,
    mut lines: box_lines,
    mut style: *const ::core::ffi::c_char,
    mut selected_style: *const ::core::ffi::c_char,
    mut border_style: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
    mut cb: menu_choice_cb,
    mut data: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut md: *mut menu_data = ::core::ptr::null_mut::<menu_data>();
    let mut event: *mut key_event = ::core::ptr::null_mut::<key_event>();
    let mut choice: ::core::ffi::c_int = 0;
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut o: *mut options = ::core::ptr::null_mut::<options>();
    if fs.is_null() {
        w = (*(*(*c).session).curw).window;
    } else {
        w = (*fs).w;
    }
    o = (*w).options;
    sx = (*menu).width.wrapping_add(4 as u_int);
    sy = (*menu).count.wrapping_add(2 as u_int);
    if sx >= (*w).sx {
        px = 0 as u_int;
    } else if px.wrapping_add(sx) > (*w).sx {
        px = (*w).sx.wrapping_sub(sx);
    }
    if sy >= (*w).sy {
        py = 0 as u_int;
    } else if py.wrapping_add(sy) > (*w).sy {
        py = (*w).sy.wrapping_sub(sy);
    }
    (*w).menu_last_px = px;
    (*w).menu_last_py = py;
    if lines as ::core::ffi::c_int == BOX_LINES_DEFAULT as ::core::ffi::c_int {
        lines = options_get_number(
            o,
            b"menu-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
        ) as box_lines;
    }
    md = xcalloc(1 as size_t, ::core::mem::size_of::<menu_data>() as size_t) as *mut menu_data;
    (*md).w = w;
    (*md).flags = flags;
    (*md).border_lines = lines;
    (*md).key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
    if !item.is_null() {
        event = cmdq_get_event(item);
        (*md).key = (*event).key;
        memcpy(
            &raw mut (*md).m as *mut ::core::ffi::c_void,
            &raw mut (*event).m as *const ::core::ffi::c_void,
            ::core::mem::size_of::<mouse_event>() as size_t,
        );
    }
    if !style.is_null() {
        (*md).style = xstrdup(style);
    }
    if !selected_style.is_null() {
        (*md).selected_style = xstrdup(selected_style);
    }
    if !border_style.is_null() {
        (*md).border_style = xstrdup(border_style);
    }
    if !fs.is_null() {
        cmd_find_copy_state(&raw mut (*md).fs, fs);
    } else if cmd_find_from_window(&raw mut (*md).fs, w, 0 as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        cmd_find_clear_state(&raw mut (*md).fs, 0 as ::core::ffi::c_int);
    }
    screen_init(&raw mut (*md).s, sx, sy, 0 as u_int);
    if !(*md).flags & MENU_NOMOUSE != 0 {
        (*md).s.mode |= MODE_MOUSE_ALL | MODE_MOUSE_BUTTON;
    }
    (*md).s.mode &= !MODE_CURSOR;
    (*md).px = px;
    (*md).py = py;
    (*md).menu = menu;
    (*md).choice = -(1 as ::core::ffi::c_int);
    (*md).cb = cb;
    (*md).data = data;
    if (*md).flags & MENU_NOMOUSE != 0 {
        if starting_choice >= (*menu).count as ::core::ffi::c_int {
            starting_choice = (*menu).count.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
            choice = starting_choice + 1 as ::core::ffi::c_int;
            loop {
                name = (*(*menu)
                    .items
                    .offset((choice - 1 as ::core::ffi::c_int) as isize))
                .name;
                if !name.is_null() && *name as ::core::ffi::c_int != '-' as i32 {
                    (*md).choice = choice - 1 as ::core::ffi::c_int;
                    break;
                } else {
                    choice -= 1;
                    if choice == 0 as ::core::ffi::c_int {
                        choice = (*menu).count as ::core::ffi::c_int;
                    }
                    if choice == starting_choice + 1 as ::core::ffi::c_int {
                        break;
                    }
                }
            }
        } else if starting_choice >= 0 as ::core::ffi::c_int {
            choice = starting_choice;
            loop {
                name = (*(*menu).items.offset(choice as isize)).name;
                if !name.is_null() && *name as ::core::ffi::c_int != '-' as i32 {
                    (*md).choice = choice;
                    break;
                } else {
                    choice += 1;
                    if choice == (*menu).count as ::core::ffi::c_int {
                        choice = 0 as ::core::ffi::c_int;
                    }
                    if choice == starting_choice {
                        break;
                    }
                }
            }
        }
    }
    menu_close((*md).w);
    (*(*md).w).menu = md;
    redraw_invalidate_scene((*md).w);
    window_update_focus((*md).w);
    server_redraw_window((*md).w);
    return 0 as ::core::ffi::c_int;
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

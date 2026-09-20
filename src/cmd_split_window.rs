use crate::src::shared::client::*;
use crate::src::shared::layout::*;
use crate::src::shared::options::*;
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
    pub type cmd;
    pub type event_payload;
    pub type options_entry;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn format_single(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut ::core::ffi::c_char;
    fn format_single_from_target(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn event_payload_create() -> *mut event_payload;
    fn event_payload_set_target(_: *mut event_payload, _: *mut cmd_find_state);
    fn event_payload_set_string(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn event_payload_set_window(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut window,
    );
    fn event_payload_set_pane(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut window_pane,
    );
    fn events_fire(_: *const ::core::ffi::c_char, _: *mut event_payload);
    fn options_search(_: *const ::core::ffi::c_char) -> *const options_table_entry;
    fn options_set_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    ) -> *mut options_entry;
    fn options_set_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
    ) -> *mut options_entry;
    fn options_find_choice(
        _: *const options_table_entry,
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn environ_create() -> *mut environ;
    fn environ_free(_: *mut environ);
    fn environ_put(_: *mut environ, _: *const ::core::ffi::c_char, _: ::core::ffi::c_int);
    fn args_to_vector(
        _: *mut args,
        _: *mut ::core::ffi::c_int,
        _: *mut *mut *mut ::core::ffi::c_char,
    );
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn args_first_value(_: *mut args, _: u_char) -> *mut args_value;
    fn args_next_value(_: *mut args_value) -> *mut args_value;
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
    fn cmd_free_argv(_: ::core::ffi::c_int, _: *mut *mut ::core::ffi::c_char);
    fn cmd_get_entry(_: *mut cmd) -> *const cmd_entry;
    fn cmd_get_args(_: *mut cmd) -> *mut args;
    fn cmdq_get_target_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_event(_: *mut cmdq_item) -> *mut key_event;
    fn cmdq_get_current(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_insert_hook(
        _: *mut session,
        _: *mut cmdq_item,
        _: *mut cmd_find_state,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn cmdq_print(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn server_client_remove_pane(_: *mut window_pane);
    fn server_redraw_session(_: *mut session);
    fn server_redraw_window(_: *mut window);
    fn server_redraw_window_borders(_: *mut window);
    fn screen_set_title(
        _: *mut screen,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn window_unzoom(_: *mut window, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn window_active_pane_is_over_zoom(_: *mut window) -> ::core::ffi::c_int;
    fn window_pop_zoom(_: *mut window) -> ::core::ffi::c_int;
    fn window_remove_pane(_: *mut window, _: *mut window_pane);
    fn window_pane_find_by_id(_: u_int) -> *mut window_pane;
    fn window_pane_is_visible(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_start_input(
        _: *mut window_pane,
        _: *mut cmdq_item,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn window_pane_get_pane_lines(_: *mut window_pane) -> pane_lines;
    fn window_get_pane_lines(_: *mut window) -> pane_lines;
    fn window_pane_is_floating(_: *mut window_pane) -> ::core::ffi::c_int;
    fn layout_set_size(
        _: *mut layout_cell,
        _: u_int,
        _: u_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn layout_fix_panes(_: *mut window, _: *mut window_pane);
    fn layout_close_pane(_: *mut window_pane);
    fn layout_get_tiled_cell(
        _: *mut cmdq_item,
        _: *mut args,
        _: *mut window,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut layout_cell;
    fn layout_get_floating_cell(
        _: *mut cmdq_item,
        _: *mut args,
        _: pane_lines,
        _: *mut window,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut layout_cell;
    fn spawn_pane(_: *mut spawn_context, _: *mut *mut ::core::ffi::c_char) -> *mut window_pane;
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
pub struct args_value {
    pub type_0: args_type,
    pub c2rust_unnamed: C2RustUnnamed_36,
    pub cached: *mut ::core::ffi::c_char,
    pub entry: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub tqe_next: *mut args_value,
    pub tqe_prev: *mut *mut args_value,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_36 {
    pub string: *mut ::core::ffi::c_char,
    pub cmdlist: *mut cmd_list,
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
pub struct options_table_entry {
    pub name: *const ::core::ffi::c_char,
    pub alternative_name: *const ::core::ffi::c_char,
    pub type_0: options_table_type,
    pub scope: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
    pub minimum: u_int,
    pub maximum: u_int,
    pub choices: *mut *const ::core::ffi::c_char,
    pub default_str: *const ::core::ffi::c_char,
    pub default_num: ::core::ffi::c_longlong,
    pub default_arr: *mut *const ::core::ffi::c_char,
    pub separator: *const ::core::ffi::c_char,
    pub pattern: *const ::core::ffi::c_char,
    pub text: *const ::core::ffi::c_char,
    pub unit: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct spawn_context {
    pub item: *mut cmdq_item,
    pub s: *mut session,
    pub wl: *mut winlink,
    pub tc: *mut client,
    pub wp0: *mut window_pane,
    pub lc: *mut layout_cell,
    pub name: *const ::core::ffi::c_char,
    pub argv: *mut *mut ::core::ffi::c_char,
    pub argc: ::core::ffi::c_int,
    pub environ: *mut environ,
    pub idx: ::core::ffi::c_int,
    pub cwd: *const ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
}
pub const PANE_MINIMUM: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_REDRAW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_STYLECHANGED: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const PANE_THEMECHANGED: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const PANE_CLOSEONCLICK: ::core::ffi::c_int = 0x80000 as ::core::ffi::c_int;
pub const PANE_CAPTUREALLKEYS: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;
pub const PANE_CLOSEONCANCEL: ::core::ffi::c_int = 0x400000 as ::core::ffi::c_int;
pub const WINDOW_ZOOMED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const SPAWN_DETACHED: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const SPAWN_BEFORE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const SPAWN_FULLSIZE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const SPAWN_EMPTY: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const SPAWN_ZOOM: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const SPAWN_FLOATING: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const SPAWN_HORIZONTAL: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const SPAWN_SPLIT: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const SPAWN_MODAL: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const SPAWN_FLOATOVERZOOM: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const SPLIT_WINDOW_TEMPLATE: [::core::ffi::c_char; 46] = unsafe {
    ::core::mem::transmute::<[u8; 46], [::core::ffi::c_char; 46]>(
        *b"#{session_name}:#{window_index}.#{pane_index}\0",
    )
};
#[no_mangle]
pub static mut cmd_new_pane_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"new-pane\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"newp\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"AbB:Cc:Dde:EfF:hIkl:KLMm:Op:PR:s:S:t:T:vWx:X:y:Y:Z\0"
                as *const u8 as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: b"[-AbCDefhIkKLMOPvWZ] [-B border-lines] [-c start-directory] [-e environment] [-F format] [-l size] [-m message] [-p percentage] [-s style] [-S active-border-style] [-R inactive-border-style] [-T title] [-x width] [-y height] [-X x-position] [-Y y-position] [-t target-pane] [shell-command [argument ...]]\0"
            as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_split_window_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
#[no_mangle]
pub static mut cmd_split_window_entry: cmd_entry = unsafe {
    cmd_entry {
        name: b"split-window\0" as *const u8 as *const ::core::ffi::c_char,
        alias: b"splitw\0" as *const u8 as *const ::core::ffi::c_char,
        args: args_parse {
            template: b"bB:c:de:EfF:hIkl:m:p:PR:s:S:t:T:vWZ\0" as *const u8
                as *const ::core::ffi::c_char,
            lower: 0 as ::core::ffi::c_int,
            upper: -(1 as ::core::ffi::c_int),
            cb: None,
        },
        usage: b"[-bdefhIklPvWZ] [-B border-lines] [-c start-directory] [-e environment] [-F format] [-l size] [-m message] [-p percentage] [-s style] [-S active-border-style] [-R inactive-border-style] [-T title] [-t target-pane] [shell-command [argument ...]]\0"
            as *const u8 as *const ::core::ffi::c_char,
        source: cmd_entry_flag {
            flag: 0,
            type_0: CMD_FIND_PANE,
            flags: 0,
        },
        target: cmd_entry_flag {
            flag: 't' as i32 as ::core::ffi::c_char,
            type_0: CMD_FIND_PANE,
            flags: 0 as ::core::ffi::c_int,
        },
        flags: 0 as ::core::ffi::c_int,
        exec: Some(
            cmd_split_window_exec
                as unsafe extern "C" fn(*mut cmd, *mut cmdq_item) -> cmd_retval,
        ),
    }
};
unsafe extern "C" fn cmd_split_window_exec(
    mut self_0: *mut cmd,
    mut item: *mut cmdq_item,
) -> cmd_retval {
    let mut current_block: u64;
    let mut args: *mut args = cmd_get_args(self_0);
    let mut current: *mut cmd_find_state = cmdq_get_current(item);
    let mut target: *mut cmd_find_state = cmdq_get_target(item);
    let mut sc: spawn_context = spawn_context {
        item: ::core::ptr::null_mut::<cmdq_item>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        tc: ::core::ptr::null_mut::<client>(),
        wp0: ::core::ptr::null_mut::<window_pane>(),
        lc: ::core::ptr::null_mut::<layout_cell>(),
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        argv: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        argc: 0,
        environ: ::core::ptr::null_mut::<environ>(),
        idx: 0,
        cwd: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: 0,
    };
    let mut tc: *mut client = cmdq_get_target_client(item);
    let mut s: *mut session = (*target).s;
    let mut wl: *mut winlink = (*target).wl;
    let mut w: *mut window = (*wl).window;
    let mut wp: *mut window_pane = (*target).wp;
    let mut new_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut event: *mut key_event = cmdq_get_event(item);
    let mut input: ::core::ffi::c_int = 0;
    let mut empty: ::core::ffi::c_int = 0;
    let mut is_floating: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut restore_zoom: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut style: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut title: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut av: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut lines: pane_lines = PANE_LINES_SINGLE;
    let mut count: u_int = args_count(args);
    if window_active_pane_is_over_zoom(w) != 0 {
        restore_zoom = 1 as ::core::ffi::c_int;
    }
    if cmd_get_entry(self_0) == &raw const cmd_new_pane_entry {
        is_floating = (args_has(args, 'L' as i32 as u_char) == 0) as ::core::ffi::c_int;
    } else {
        if window_pane_is_visible(wp) == 0 {
            restore_zoom = 0 as ::core::ffi::c_int;
        }
        if restore_zoom == 0 {
            window_unzoom(w, 1 as ::core::ffi::c_int);
        }
        is_floating = window_pane_is_floating(wp);
        flags |= SPAWN_SPLIT;
    }
    if args_has(args, 'O' as i32 as u_char) != 0 {
        if is_floating == 0 {
            cmdq_error(
                item,
                b"modal pane must be floating\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
        if !(*w).modal.is_null() {
            cmdq_error(
                item,
                b"window already has a modal pane\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return CMD_RETURN_ERROR;
        }
    }
    if args_has(args, 'M' as i32 as u_char) != 0 && is_floating != 0 {
        if event.is_null() || (*event).m.valid == 0 || tc.is_null() {
            return CMD_RETURN_NORMAL;
        }
    }
    if is_floating != 0 {
        flags |= SPAWN_FLOATING;
    }
    if args_has(args, 'h' as i32 as u_char) != 0 {
        flags |= SPAWN_HORIZONTAL;
    }
    if args_has(args, 'b' as i32 as u_char) != 0 {
        flags |= SPAWN_BEFORE;
    }
    if args_has(args, 'f' as i32 as u_char) != 0 {
        flags |= SPAWN_FULLSIZE;
    }
    if args_has(args, 'd' as i32 as u_char) != 0 {
        flags |= SPAWN_DETACHED;
    }
    if args_has(args, 'Z' as i32 as u_char) != 0 {
        flags |= SPAWN_ZOOM;
    }
    if args_has(args, 'O' as i32 as u_char) != 0 {
        flags |= SPAWN_MODAL | SPAWN_FLOATOVERZOOM;
    }
    if is_floating != 0 && args_has(args, 'A' as i32 as u_char) != 0 {
        flags |= SPAWN_FLOATOVERZOOM;
    }
    if (*w).flags & WINDOW_ZOOMED != 0 && flags & SPAWN_FLOATOVERZOOM != 0 {
        restore_zoom = 1 as ::core::ffi::c_int;
    }
    input = args_has(args, 'I' as i32 as u_char);
    if input != 0
        || count == 1 as u_int
            && *args_string(args, 0 as u_int) as ::core::ffi::c_int == '\0' as i32
    {
        empty = 1 as ::core::ffi::c_int;
    } else {
        empty = args_has(args, 'E' as i32 as u_char);
    }
    if empty != 0
        && count != 0 as u_int
        && (count != 1 as u_int
            || *args_string(args, 0 as u_int) as ::core::ffi::c_int != '\0' as i32)
    {
        cmdq_error(
            item,
            b"command cannot be given for empty pane\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return CMD_RETURN_ERROR;
    }
    if empty != 0 {
        flags |= SPAWN_EMPTY;
    }
    value = args_get(args, 'B' as i32 as u_char);
    if value.is_null() {
        lines = window_get_pane_lines(w);
    } else {
        oe = options_search(b"pane-border-lines\0" as *const u8 as *const ::core::ffi::c_char);
        lines = options_find_choice(oe, value, &raw mut cause) as pane_lines;
        if !cause.is_null() {
            cmdq_error(
                item,
                b"pane-border-lines %s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
            return CMD_RETURN_ERROR;
        }
    }
    if flags & SPAWN_FLOATING != 0 {
        lc = layout_get_floating_cell(item, args, lines, w, wp, flags, &raw mut cause);
    } else {
        lc = layout_get_tiled_cell(item, args, w, wp, flags, &raw mut cause);
    }
    if !cause.is_null() {
        cmdq_error(
            item,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            cause,
        );
        free(cause as *mut ::core::ffi::c_void);
        if restore_zoom != 0 {
            window_pop_zoom(w);
        }
        return CMD_RETURN_ERROR;
    }
    sc.item = item;
    sc.s = s;
    sc.wl = wl;
    sc.wp0 = wp;
    sc.lc = lc;
    args_to_vector(args, &raw mut sc.argc, &raw mut sc.argv);
    sc.environ = environ_create();
    av = args_first_value(args, 'e' as i32 as u_char);
    while !av.is_null() {
        environ_put(
            sc.environ,
            (*av).c2rust_unnamed.string,
            0 as ::core::ffi::c_int,
        );
        av = args_next_value(av);
    }
    sc.idx = -(1 as ::core::ffi::c_int);
    sc.cwd = args_get(args, 'c' as i32 as u_char);
    sc.flags = flags;
    new_wp = spawn_pane(&raw mut sc, &raw mut cause);
    if new_wp.is_null() {
        cmdq_error(
            item,
            b"create pane failed: %s\0" as *const u8 as *const ::core::ffi::c_char,
            cause,
        );
        free(cause as *mut ::core::ffi::c_void);
    } else {
        if args_has(args, 'K' as i32 as u_char) != 0 && args_has(args, 'O' as i32 as u_char) != 0 {
            (*new_wp).flags |= PANE_CAPTUREALLKEYS;
        }
        if args_has(args, 'C' as i32 as u_char) != 0 && args_has(args, 'O' as i32 as u_char) != 0 {
            (*new_wp).flags |= PANE_CLOSEONCLICK;
        }
        if args_has(args, 'D' as i32 as u_char) != 0 && args_has(args, 'O' as i32 as u_char) != 0 {
            (*new_wp).flags |= PANE_CLOSEONCANCEL;
        }
        style = args_get(args, 's' as i32 as u_char);
        if !style.is_null() {
            if options_set_string(
                (*new_wp).options,
                b"window-style\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                style,
            )
            .is_null()
            {
                cmdq_error(
                    item,
                    b"bad style: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    style,
                );
                current_block = 9814746494299271243;
            } else {
                options_set_string(
                    (*new_wp).options,
                    b"window-active-style\0" as *const u8 as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    style,
                );
                (*new_wp).flags |= PANE_REDRAW | PANE_STYLECHANGED | PANE_THEMECHANGED;
                current_block = 14329534724295951598;
            }
        } else {
            current_block = 14329534724295951598;
        }
        match current_block {
            9814746494299271243 => {}
            _ => {
                style = args_get(args, 'S' as i32 as u_char);
                if !style.is_null() {
                    if options_set_string(
                        (*new_wp).options,
                        b"pane-active-border-style\0" as *const u8 as *const ::core::ffi::c_char,
                        0 as ::core::ffi::c_int,
                        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                        style,
                    )
                    .is_null()
                    {
                        cmdq_error(
                            item,
                            b"bad active border style: %s\0" as *const u8
                                as *const ::core::ffi::c_char,
                            style,
                        );
                        current_block = 9814746494299271243;
                    } else {
                        current_block = 12070711452894729854;
                    }
                } else {
                    current_block = 12070711452894729854;
                }
                match current_block {
                    9814746494299271243 => {}
                    _ => {
                        style = args_get(args, 'R' as i32 as u_char);
                        if !style.is_null() {
                            if options_set_string(
                                (*new_wp).options,
                                b"pane-border-style\0" as *const u8 as *const ::core::ffi::c_char,
                                0 as ::core::ffi::c_int,
                                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                style,
                            )
                            .is_null()
                            {
                                cmdq_error(
                                    item,
                                    b"bad inactive border style: %s\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    style,
                                );
                                current_block = 9814746494299271243;
                            } else {
                                current_block = 16313536926714486912;
                            }
                        } else {
                            current_block = 16313536926714486912;
                        }
                        match current_block {
                            9814746494299271243 => {}
                            _ => {
                                if args_has(args, 'B' as i32 as u_char) != 0 {
                                    options_set_number(
                                        (*new_wp).options,
                                        b"pane-border-lines\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        lines as ::core::ffi::c_longlong,
                                    );
                                }
                                if args_has(args, 'k' as i32 as u_char) != 0
                                    || args_has(args, 'm' as i32 as u_char) != 0
                                {
                                    options_set_number(
                                        (*new_wp).options,
                                        b"remain-on-exit\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        3 as ::core::ffi::c_longlong,
                                    );
                                    if args_has(args, 'm' as i32 as u_char) != 0 {
                                        options_set_string(
                                            (*new_wp).options,
                                            b"remain-on-exit-format\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            0 as ::core::ffi::c_int,
                                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            args_get(args, 'm' as i32 as u_char),
                                        );
                                    }
                                }
                                if args_has(args, 'T' as i32 as u_char) != 0 {
                                    title = format_single_from_target(
                                        item,
                                        args_get(args, 'T' as i32 as u_char),
                                    );
                                    screen_set_title(
                                        &raw mut (*new_wp).base,
                                        title,
                                        0 as ::core::ffi::c_int,
                                    );
                                    ep = event_payload_create();
                                    cmd_find_from_pane(
                                        &raw mut fs,
                                        new_wp,
                                        0 as ::core::ffi::c_int,
                                    );
                                    event_payload_set_target(ep, &raw mut fs);
                                    event_payload_set_pane(
                                        ep,
                                        b"pane\0" as *const u8 as *const ::core::ffi::c_char,
                                        new_wp,
                                    );
                                    event_payload_set_window(
                                        ep,
                                        b"window\0" as *const u8 as *const ::core::ffi::c_char,
                                        (*new_wp).window as *mut window,
                                    );
                                    event_payload_set_string(
                                        ep,
                                        b"new_title\0" as *const u8 as *const ::core::ffi::c_char,
                                        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                        title,
                                    );
                                    events_fire(
                                        b"pane-title-changed\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        ep,
                                    );
                                    free(title as *mut ::core::ffi::c_void);
                                }
                                if input != 0 {
                                    match window_pane_start_input(new_wp, item, &raw mut cause) {
                                        -1 => {
                                            current_block = 13617326970112485193;
                                            match current_block {
                                                14804284628290581338 => {
                                                    input = 0 as ::core::ffi::c_int;
                                                    current_block = 12543410360505780601;
                                                }
                                                _ => {
                                                    cmdq_error(
                                                        item,
                                                        b"%s\0" as *const u8
                                                            as *const ::core::ffi::c_char,
                                                        cause,
                                                    );
                                                    free(cause as *mut ::core::ffi::c_void);
                                                    current_block = 9814746494299271243;
                                                }
                                            }
                                        }
                                        1 => {
                                            current_block = 14804284628290581338;
                                            match current_block {
                                                14804284628290581338 => {
                                                    input = 0 as ::core::ffi::c_int;
                                                    current_block = 12543410360505780601;
                                                }
                                                _ => {
                                                    cmdq_error(
                                                        item,
                                                        b"%s\0" as *const u8
                                                            as *const ::core::ffi::c_char,
                                                        cause,
                                                    );
                                                    free(cause as *mut ::core::ffi::c_void);
                                                    current_block = 9814746494299271243;
                                                }
                                            }
                                        }
                                        _ => {
                                            current_block = 12543410360505780601;
                                        }
                                    }
                                } else {
                                    current_block = 12543410360505780601;
                                }
                                match current_block {
                                    9814746494299271243 => {}
                                    _ => {
                                        if !flags & SPAWN_DETACHED != 0 {
                                            cmd_find_from_winlink_pane(
                                                current,
                                                wl,
                                                new_wp,
                                                0 as ::core::ffi::c_int,
                                            );
                                        }
                                        if restore_zoom != 0 {
                                            window_pop_zoom((*wp).window as *mut window);
                                            server_redraw_window((*wp).window as *mut window);
                                        } else if !flags & SPAWN_FLOATING != 0
                                            && args_has(args, 'O' as i32 as u_char) == 0
                                        {
                                            window_pop_zoom((*wp).window as *mut window);
                                            server_redraw_window((*wp).window as *mut window);
                                        }
                                        server_redraw_session(s);
                                        if args_has(args, 'M' as i32 as u_char) != 0
                                            && is_floating != 0
                                        {
                                            (*tc).tty.mouse_last_pane =
                                                (*new_wp).id as ::core::ffi::c_int;
                                            (*tc).tty.mouse_drag_update = Some(
                                                cmd_split_window_mouse_resize
                                                    as unsafe extern "C" fn(
                                                        *mut client,
                                                        *mut mouse_event,
                                                    )
                                                        -> (),
                                            )
                                                as Option<
                                                    unsafe extern "C" fn(
                                                        *mut client,
                                                        *mut mouse_event,
                                                    )
                                                        -> (),
                                                >;
                                            cmd_split_window_mouse_resize(tc, &raw mut (*event).m);
                                        }
                                        if args_has(args, 'P' as i32 as u_char) != 0 {
                                            template = args_get(args, 'F' as i32 as u_char);
                                            if template.is_null() {
                                                template = SPLIT_WINDOW_TEMPLATE.as_ptr();
                                            }
                                            cp = format_single(item, template, tc, s, wl, new_wp);
                                            cmdq_print(
                                                item,
                                                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                                cp,
                                            );
                                            free(cp as *mut ::core::ffi::c_void);
                                        }
                                        cmd_find_from_winlink_pane(
                                            &raw mut fs,
                                            wl,
                                            new_wp,
                                            0 as ::core::ffi::c_int,
                                        );
                                        cmdq_insert_hook(
                                            s,
                                            item,
                                            &raw mut fs,
                                            b"after-split-window\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                        );
                                        if !sc.argv.is_null() {
                                            cmd_free_argv(sc.argc, sc.argv);
                                        }
                                        environ_free(sc.environ);
                                        if input != 0 {
                                            return CMD_RETURN_WAIT;
                                        }
                                        if args_has(args, 'W' as i32 as u_char) != 0 {
                                            (*new_wp).wait_item = item;
                                            return CMD_RETURN_WAIT;
                                        }
                                        return CMD_RETURN_NORMAL;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !new_wp.is_null() {
        server_client_remove_pane(new_wp);
        if is_floating == 0 {
            layout_close_pane(new_wp);
        }
        window_remove_pane((*wp).window as *mut window, new_wp);
    }
    if restore_zoom != 0 || !flags & SPAWN_FLOATING != 0 {
        window_pop_zoom((*wp).window as *mut window);
    }
    if !sc.argv.is_null() {
        cmd_free_argv(sc.argc, sc.argv);
    }
    environ_free(sc.environ);
    return CMD_RETURN_ERROR;
}
unsafe extern "C" fn cmd_split_window_mouse_resize(mut c: *mut client, mut m: *mut mouse_event) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lines: pane_lines = PANE_LINES_SINGLE;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut border: ::core::ffi::c_int = 0;
    if (*c).tty.mouse_last_pane == -(1 as ::core::ffi::c_int) {
        return;
    }
    wp = window_pane_find_by_id((*c).tty.mouse_last_pane as u_int);
    if wp.is_null() || window_pane_is_floating(wp) == 0 {
        (*c).tty.mouse_drag_update = None;
        return;
    }
    w = (*wp).window as *mut window;
    lc = (*wp).layout_cell as *mut layout_cell;
    x = (*m).x.wrapping_add((*m).ox) as ::core::ffi::c_int;
    y = (*m).y.wrapping_add((*m).oy) as ::core::ffi::c_int;
    if (*m).statusat == 0 as ::core::ffi::c_int && y >= (*m).statuslines as ::core::ffi::c_int {
        y = (y as u_int).wrapping_sub((*m).statuslines) as ::core::ffi::c_int as ::core::ffi::c_int;
    } else if (*m).statusat > 0 as ::core::ffi::c_int && y >= (*m).statusat {
        y = (*m).statusat - 1 as ::core::ffi::c_int;
    }
    lines = window_pane_get_pane_lines(wp);
    border = (lines as ::core::ffi::c_uint
        != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
    if x >= (*c).tty.mouse_drag_x as ::core::ffi::c_int {
        xoff = (*c).tty.mouse_drag_x.wrapping_add(border as u_int) as ::core::ffi::c_int;
        sx = (x as u_int)
            .wrapping_sub((*c).tty.mouse_drag_x)
            .wrapping_add(1 as u_int);
    } else {
        sx = (*c)
            .tty
            .mouse_drag_x
            .wrapping_sub(x as u_int)
            .wrapping_add(1 as u_int);
        xoff = (*c)
            .tty
            .mouse_drag_x
            .wrapping_sub(sx)
            .wrapping_add(1 as u_int) as ::core::ffi::c_int;
        if border != 0 {
            xoff += 1;
        }
    }
    if y >= (*c).tty.mouse_drag_y as ::core::ffi::c_int {
        yoff = (*c).tty.mouse_drag_y.wrapping_add(border as u_int) as ::core::ffi::c_int;
        sy = (y as u_int)
            .wrapping_sub((*c).tty.mouse_drag_y)
            .wrapping_add(1 as u_int);
    } else {
        sy = (*c)
            .tty
            .mouse_drag_y
            .wrapping_sub(y as u_int)
            .wrapping_add(1 as u_int);
        yoff = (*c)
            .tty
            .mouse_drag_y
            .wrapping_sub(sy)
            .wrapping_add(1 as u_int) as ::core::ffi::c_int;
        if border != 0 {
            yoff += 1;
        }
    }
    if border != 0 {
        if sx <= 2 as u_int {
            sx = PANE_MINIMUM as u_int;
        } else {
            sx = sx.wrapping_sub(2 as u_int);
        }
        if sy <= 2 as u_int {
            sy = PANE_MINIMUM as u_int;
        } else {
            sy = sy.wrapping_sub(2 as u_int);
        }
    }
    if sx < PANE_MINIMUM as u_int {
        sx = PANE_MINIMUM as u_int;
    }
    if sy < PANE_MINIMUM as u_int {
        sy = PANE_MINIMUM as u_int;
    }
    layout_set_size(lc, sx, sy, xoff, yoff);
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    server_redraw_window(w);
    server_redraw_window_borders(w);
}

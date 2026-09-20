pub use crate::src::shared::window::{
    WINDOW_MODE_HIDE_PANE_STATUS, WINDOW_MODE_HIDE_SCROLLBARS, WINDOW_MODE_NO_STACK,
    WINDOW_ZOOMED,
};
pub use crate::src::shared::pane::{
    PANE_REDRAW, PANE_STATUS_BOTTOM, PANE_STATUS_TOP, window_pane_offset, window_pane_resize,
    window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::screen::{MODE_CURSOR, screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_citem, screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
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
extern "C" {
    pub type args;
    pub type args_command_state;
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
    pub type cmd;
    pub type cmdq_state;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn event_add(ev: *mut event, timeout: *const timeval) -> ::core::ffi::c_int;
    fn event_del(_: *mut event) -> ::core::ffi::c_int;
    fn event_set(
        _: *mut event,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_short,
        _: Option<
            unsafe extern "C" fn(
                ::core::ffi::c_int,
                ::core::ffi::c_short,
                *mut ::core::ffi::c_void,
            ) -> (),
        >,
        _: *mut ::core::ffi::c_void,
    );
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn xsnprintf(
        _: *mut ::core::ffi::c_char,
        _: size_t,
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
    fn format_create_defaults(
        _: *mut cmdq_item,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut format_tree;
    fn format_draw(
        _: *mut screen_write_ctx,
        _: *const grid_cell,
        _: u_int,
        _: *const ::core::ffi::c_char,
        _: *mut style_ranges,
        _: ::core::ffi::c_int,
    );
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_make_commands_prepare(
        _: *mut cmd,
        _: *mut cmdq_item,
        _: u_int,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut args_command_state;
    fn args_make_commands(
        _: *mut args_command_state,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut cmd_list;
    fn args_make_commands_free(_: *mut args_command_state);
    fn args_strtonum(
        _: *mut args,
        _: u_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn cmd_list_free(_: *mut cmd_list);
    fn cmd_mouse_at(
        _: *mut window_pane,
        _: *mut mouse_event,
        _: *mut u_int,
        _: *mut u_int,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmdq_get_cmd(_: *mut cmdq_item) -> *mut cmd;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_source(_: *mut cmdq_item) -> *mut cmd_find_state;
    fn cmdq_get_command(_: *mut cmd_list, _: *mut cmdq_state) -> *mut cmdq_item;
    fn cmdq_get_error(_: *const ::core::ffi::c_char) -> *mut cmdq_item;
    fn cmdq_append(_: *mut client, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn server_redraw_window(_: *mut window);
    fn server_redraw_window_borders(_: *mut window);
    fn server_status_window(_: *mut window);
    fn server_unzoom_window(_: *mut window);
    static grid_default_cell: grid_cell;
    fn screen_write_start(_: *mut screen_write_ctx, _: *mut screen);
    fn screen_write_stop(_: *mut screen_write_ctx);
    fn screen_write_puts(
        _: *mut screen_write_ctx,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn screen_write_putc(_: *mut screen_write_ctx, _: *const grid_cell, _: u_char);
    fn screen_write_fast_copy(
        _: *mut screen_write_ctx,
        _: *mut screen,
        _: u_int,
        _: u_int,
        _: u_int,
        _: u_int,
    );
    fn screen_write_preview(_: *mut screen_write_ctx, _: *mut screen, _: u_int, _: u_int);
    fn screen_write_cursormove(
        _: *mut screen_write_ctx,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn screen_write_clearscreen(_: *mut screen_write_ctx, _: u_int);
    fn screen_write_cell(_: *mut screen_write_ctx, _: *const grid_cell);
    fn screen_init(_: *mut screen, _: u_int, _: u_int, _: u_int);
    fn screen_free(_: *mut screen);
    fn screen_resize(_: *mut screen, _: u_int, _: u_int, _: ::core::ffi::c_int);
    fn winlink_find_by_window(_: *mut winlinks, _: *mut window) -> *mut winlink;
    fn window_find_by_id(_: u_int) -> *mut window;
    fn window_zoom(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_unzoom(_: *mut window, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn window_pane_at_index(_: *mut window, _: u_int) -> *mut window_pane;
    fn window_pane_index(_: *mut window_pane, _: *mut u_int) -> ::core::ffi::c_int;
    fn window_pane_find_by_id(_: u_int) -> *mut window_pane;
    fn window_pane_reset_mode(_: *mut window_pane);
    fn window_pane_is_visible(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_get_pane_status(_: *mut window) -> ::core::ffi::c_int;
    fn layout_add_horizontal_border(
        _: *mut layout_cell,
        _: *mut layout_cell,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    static window_clock_table: [[[::core::ffi::c_char; 5]; 5]; 14];
    fn session_find_by_id(_: u_int) -> *mut session;
    fn utf8_set(_: *mut utf8_data, _: u_char);
    fn style_apply(
        _: *mut grid_cell,
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: *mut format_tree,
    );
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
pub type C2RustUnnamed_38 = ::core::ffi::c_ulong;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_panes_zindex {
    pub tqh_first: *mut window_pane,
    pub tqh_last: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_panes_modedata {
    pub wp: *mut window_pane,
    pub session: *mut session,
    pub source_session: u_int,
    pub source_window: u_int,
    pub screen: screen,
    pub preview: *mut screen,
    pub timer: event,
    pub state: *mut args_command_state,
    pub delay: u_int,
    pub ignore_keys: ::core::ffi::c_int,
    pub zoomed: ::core::ffi::c_int,
    pub areas: *mut window_panes_area,
    pub areas_size: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_panes_area {
    pub id: u_int,
    pub x: u_int,
    pub y: u_int,
    pub sx: u_int,
    pub sy: u_int,
}
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const CELL_UD: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CELL_LR: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CELL_RD: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const CELL_LD: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const CELL_RU: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const CELL_LU: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const CELL_LRD: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const CELL_LRU: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const CELL_URD: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const CELL_ULD: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const CELL_LRUD: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const CELL_NONE: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const CELL_BORDERS: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b" xqlkmjwvtun~\0") };
pub const WINDOW_MODE_FILL_WINDOW: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
#[no_mangle]
pub static mut window_panes_mode: window_mode = unsafe {
    window_mode {
        name: b"panes-mode\0" as *const u8 as *const ::core::ffi::c_char,
        default_format: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: WINDOW_MODE_HIDE_PANE_STATUS
            | WINDOW_MODE_NO_STACK
            | WINDOW_MODE_FILL_WINDOW
            | WINDOW_MODE_HIDE_SCROLLBARS,
        init: Some(
            window_panes_init
                as unsafe extern "C" fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_panes_free as unsafe extern "C" fn(*mut window_mode_entry) -> ()),
        resize: Some(
            window_panes_resize as unsafe extern "C" fn(*mut window_mode_entry, u_int, u_int) -> (),
        ),
        update: None,
        style_changed: None,
        key: Some(
            window_panes_key
                as unsafe extern "C" fn(
                    *mut window_mode_entry,
                    *mut client,
                    *mut session,
                    *mut winlink,
                    key_code,
                    *mut mouse_event,
                ) -> (),
        ),
        key_table: None,
        command: None,
        formats: None,
        get_screen: None,
    }
};
pub const WINDOW_PANES_BORDER_L: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINDOW_PANES_BORDER_R: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const WINDOW_PANES_BORDER_U: ::core::ffi::c_int = 4;
pub const WINDOW_PANES_BORDER_D: ::core::ffi::c_int = 8;
unsafe extern "C" fn window_panes_get_source(
    mut data: *mut window_panes_modedata,
    mut sp: *mut *mut session,
    mut wlp: *mut *mut winlink,
    mut wp: *mut *mut window,
) -> ::core::ffi::c_int {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    w = window_find_by_id((*data).source_window);
    if w.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    s = session_find_by_id((*data).source_session);
    if !s.is_null() {
        wl = winlink_find_by_window(&raw mut (*s).windows, w);
    }
    if wl.is_null() {
        s = (*data).session;
    }
    if !s.is_null() {
        wl = winlink_find_by_window(&raw mut (*s).windows, w);
    }
    if !sp.is_null() {
        *sp = s;
    }
    if !wlp.is_null() {
        *wlp = wl;
    }
    if !wp.is_null() {
        *wp = w;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_panes_set_preview(mut data: *mut window_panes_modedata) {
    let mut wp: *mut window_pane = (*data).wp;
    let mut src: *mut screen = &raw mut (*wp).base;
    let mut dst: *mut screen = ::core::ptr::null_mut::<screen>();
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
    let mut sx: u_int = (*(*src).grid).sx;
    let mut sy: u_int = (*(*src).grid).sy;
    dst = xmalloc(::core::mem::size_of::<screen>() as size_t) as *mut screen;
    (*data).preview = dst;
    screen_init(dst, sx, sy, 0 as u_int);
    screen_write_start(&raw mut ctx, dst);
    screen_write_fast_copy(&raw mut ctx, src, 0 as u_int, (*(*src).grid).hsize, sx, sy);
    screen_write_stop(&raw mut ctx);
    (*dst).mode = (*src).mode;
    (*dst).cx = (*src).cx;
    (*dst).cy = (*src).cy;
}
unsafe extern "C" fn window_panes_free_areas(mut data: *mut window_panes_modedata) {
    free((*data).areas as *mut ::core::ffi::c_void);
    (*data).areas = ::core::ptr::null_mut::<window_panes_area>();
    (*data).areas_size = 0 as u_int;
}
unsafe extern "C" fn window_panes_add_area(
    mut data: *mut window_panes_modedata,
    mut wp: *mut window_pane,
    mut x: u_int,
    mut y: u_int,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut area: *mut window_panes_area = ::core::ptr::null_mut::<window_panes_area>();
    (*data).areas = xreallocarray(
        (*data).areas as *mut ::core::ffi::c_void,
        (*data).areas_size.wrapping_add(1 as u_int) as size_t,
        ::core::mem::size_of::<window_panes_area>() as size_t,
    ) as *mut window_panes_area;
    let fresh1 = (*data).areas_size;
    (*data).areas_size = (*data).areas_size.wrapping_add(1);
    area = (*data).areas.offset(fresh1 as isize) as *mut window_panes_area;
    (*area).id = (*wp).id;
    (*area).x = x;
    (*area).y = y;
    (*area).sx = sx;
    (*area).sy = sy;
}
unsafe extern "C" fn window_panes_pane_floating(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut lc: *mut layout_cell = (*wp).saved_layout_cell;
    if lc.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
    }
    if lc.is_null() || !(*lc).flags & LAYOUT_CELL_FLOATING != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_panes_pane_visible(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    if !(*wp).saved_layout_cell.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    return window_pane_is_visible(wp);
}
unsafe extern "C" fn window_panes_get_geometry(
    mut wp: *mut window_pane,
    mut root: *mut layout_cell,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
    mut xp: *mut u_int,
    mut yp: *mut u_int,
    mut sxp: *mut u_int,
    mut syp: *mut u_int,
) -> ::core::ffi::c_int {
    let mut lc: *mut layout_cell = (*wp).saved_layout_cell;
    let mut status: ::core::ffi::c_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut x2: u_int = 0;
    let mut y2: u_int = 0;
    if lc.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
    }
    if lc.is_null()
        || osx == 0 as u_int
        || osy == 0 as u_int
        || dsx == 0 as u_int
        || dsy == 0 as u_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if osx <= dsx && osy <= dsy {
        x = (*lc).g.xoff as u_int;
        y = (*lc).g.yoff as u_int;
        x2 = x.wrapping_add((*lc).g.sx);
        y2 = y.wrapping_add((*lc).g.sy);
    } else {
        x = ((*lc).g.xoff as u_int).wrapping_mul(dsx).wrapping_div(osx);
        y = ((*lc).g.yoff as u_int).wrapping_mul(dsy).wrapping_div(osy);
        x2 = ((*lc).g.xoff as u_int)
            .wrapping_add((*lc).g.sx)
            .wrapping_mul(dsx)
            .wrapping_div(osx);
        y2 = ((*lc).g.yoff as u_int)
            .wrapping_add((*lc).g.sy)
            .wrapping_mul(dsy)
            .wrapping_div(osy);
    }
    if x >= dsx || y >= dsy {
        return 0 as ::core::ffi::c_int;
    }
    if x2 <= x {
        x2 = x.wrapping_add(1 as u_int);
    }
    if y2 <= y {
        y2 = y.wrapping_add(1 as u_int);
    }
    if x2 > dsx {
        x2 = dsx;
    }
    if y2 > dsy {
        y2 = dsy;
    }
    sx = x2.wrapping_sub(x);
    sy = y2.wrapping_sub(y);
    if sx == 0 as u_int || sy == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    status = window_get_pane_status((*wp).window as *mut window);
    if layout_add_horizontal_border(root, lc, status) != 0 && sy > 1 as u_int {
        if status == PANE_STATUS_TOP {
            y = y.wrapping_add(1);
        }
        sy = sy.wrapping_sub(1);
    }
    *xp = x;
    *yp = y;
    *sxp = sx;
    *syp = sy;
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_panes_get_border_cell(
    mut data: *mut window_panes_modedata,
    mut gc: *mut grid_cell,
) {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut wp: *mut window_pane = (*data).wp;
    let mut s: *mut session = (*data).session;
    memcpy(
        gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        s,
        (*s).curw,
        wp,
    );
    style_apply(
        gc,
        (*(*wp).window).options,
        b"display-panes-border-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
    format_free(ft);
}
unsafe extern "C" fn window_panes_map_x(
    mut x: u_int,
    mut osx: u_int,
    mut dsx: u_int,
) -> ::core::ffi::c_int {
    if osx <= dsx {
        return x as ::core::ffi::c_int;
    }
    return x.wrapping_mul(dsx).wrapping_div(osx) as ::core::ffi::c_int;
}
unsafe extern "C" fn window_panes_map_y(
    mut y: u_int,
    mut osy: u_int,
    mut dsy: u_int,
) -> ::core::ffi::c_int {
    if osy <= dsy {
        return y as ::core::ffi::c_int;
    }
    return y.wrapping_mul(dsy).wrapping_div(osy) as ::core::ffi::c_int;
}
unsafe extern "C" fn window_panes_next_tiled_cell(mut lc: *mut layout_cell) -> *mut layout_cell {
    let mut next: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    next = (*lc).entry.tqe_next;
    while !next.is_null() {
        if !(*next).flags & LAYOUT_CELL_FLOATING != 0 {
            return next;
        }
        next = (*next).entry.tqe_next;
    }
    return ::core::ptr::null_mut::<layout_cell>();
}
unsafe extern "C" fn window_panes_mark_border(
    mut map: *mut u_char,
    mut dsx: u_int,
    mut dsy: u_int,
    mut x: u_int,
    mut y: u_int,
    mut mask: u_char,
) {
    if x < dsx && y < dsy {
        let ref mut fresh0 = *map.offset(y.wrapping_mul(dsx).wrapping_add(x) as isize);
        *fresh0 = (*fresh0 as ::core::ffi::c_int | mask as ::core::ffi::c_int) as u_char;
    }
}
unsafe extern "C" fn window_panes_mark_vline(
    mut map: *mut u_char,
    mut dsx: u_int,
    mut dsy: u_int,
    mut x: ::core::ffi::c_int,
    mut y: ::core::ffi::c_int,
    mut y2: ::core::ffi::c_int,
) {
    let mut mask: u_char = 0;
    let mut yy: ::core::ffi::c_int = 0;
    if x < 0 as ::core::ffi::c_int || x as u_int >= dsx || y2 <= y {
        return;
    }
    if y < 0 as ::core::ffi::c_int {
        y = 0 as ::core::ffi::c_int;
    }
    if y2 as u_int > dsy {
        y2 = dsy as ::core::ffi::c_int;
    }
    yy = y;
    while yy < y2 {
        mask = 0 as u_char;
        if yy > y {
            mask = (mask as ::core::ffi::c_int | WINDOW_PANES_BORDER_U) as u_char;
        }
        if (yy + 1 as ::core::ffi::c_int) < y2 {
            mask = (mask as ::core::ffi::c_int | WINDOW_PANES_BORDER_D) as u_char;
        }
        if mask as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            mask = (WINDOW_PANES_BORDER_U | WINDOW_PANES_BORDER_D) as u_char;
        }
        window_panes_mark_border(map, dsx, dsy, x as u_int, yy as u_int, mask);
        yy += 1;
    }
}
unsafe extern "C" fn window_panes_mark_hline(
    mut map: *mut u_char,
    mut dsx: u_int,
    mut dsy: u_int,
    mut x: ::core::ffi::c_int,
    mut x2: ::core::ffi::c_int,
    mut y: ::core::ffi::c_int,
) {
    let mut mask: u_char = 0;
    let mut xx: ::core::ffi::c_int = 0;
    if y < 0 as ::core::ffi::c_int || y as u_int >= dsy || x2 <= x {
        return;
    }
    if x < 0 as ::core::ffi::c_int {
        x = 0 as ::core::ffi::c_int;
    }
    if x2 as u_int > dsx {
        x2 = dsx as ::core::ffi::c_int;
    }
    xx = x;
    while xx < x2 {
        mask = 0 as u_char;
        if xx > x {
            mask = (mask as ::core::ffi::c_int | WINDOW_PANES_BORDER_L) as u_char;
        }
        if (xx + 1 as ::core::ffi::c_int) < x2 {
            mask = (mask as ::core::ffi::c_int | WINDOW_PANES_BORDER_R) as u_char;
        }
        if mask as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            mask = (WINDOW_PANES_BORDER_L | WINDOW_PANES_BORDER_R) as u_char;
        }
        window_panes_mark_border(map, dsx, dsy, xx as u_int, y as u_int, mask);
        xx += 1;
    }
}
unsafe extern "C" fn window_panes_mark_borders_cell(
    mut map: *mut u_char,
    mut lc: *mut layout_cell,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcnext: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut x2: ::core::ffi::c_int = 0;
    let mut y2: ::core::ffi::c_int = 0;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    lcchild = (*lc).cells.tqh_first;
    while !lcchild.is_null() {
        window_panes_mark_borders_cell(map, lcchild, osx, osy, dsx, dsy);
        if !((*lcchild).flags & LAYOUT_CELL_FLOATING != 0) {
            lcnext = window_panes_next_tiled_cell(lcchild);
            if !lcnext.is_null() {
                if (*lc).type_0 as ::core::ffi::c_uint
                    == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    x = window_panes_map_x(
                        ((*lcchild).g.xoff as u_int).wrapping_add((*lcchild).g.sx),
                        osx,
                        dsx,
                    );
                    y = window_panes_map_y((*lc).g.yoff as u_int, osy, dsy);
                    y2 = window_panes_map_y(
                        ((*lc).g.yoff as u_int).wrapping_add((*lc).g.sy),
                        osy,
                        dsy,
                    );
                    window_panes_mark_vline(map, dsx, dsy, x, y, y2);
                } else {
                    x = window_panes_map_x((*lc).g.xoff as u_int, osx, dsx);
                    x2 = window_panes_map_x(
                        ((*lc).g.xoff as u_int).wrapping_add((*lc).g.sx),
                        osx,
                        dsx,
                    );
                    y = window_panes_map_y(
                        ((*lcchild).g.yoff as u_int).wrapping_add((*lcchild).g.sy),
                        osy,
                        dsy,
                    );
                    window_panes_mark_hline(map, dsx, dsy, x, x2, y);
                }
            }
        }
        lcchild = (*lcchild).entry.tqe_next;
    }
}
unsafe extern "C" fn window_panes_mark_pane_status_borders(
    mut map: *mut u_char,
    mut w: *mut window,
    mut root: *mut layout_cell,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut status: ::core::ffi::c_int = 0;
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut x2: ::core::ffi::c_int = 0;
    let mut y2: ::core::ffi::c_int = 0;
    status = window_get_pane_status(w);
    if status != PANE_STATUS_TOP && status != PANE_STATUS_BOTTOM {
        return;
    }
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if !(window_panes_pane_visible(wp) == 0) {
            lc = (*wp).saved_layout_cell;
            if lc.is_null() {
                lc = (*wp).layout_cell as *mut layout_cell;
            }
            if !(lc.is_null() || layout_add_horizontal_border(root, lc, status) == 0) {
                x = window_panes_map_x((*lc).g.xoff as u_int, osx, dsx);
                x2 = window_panes_map_x(((*lc).g.xoff as u_int).wrapping_add((*lc).g.sx), osx, dsx);
                if status == PANE_STATUS_TOP {
                    y = window_panes_map_y((*lc).g.yoff as u_int, osy, dsy);
                } else {
                    y2 = window_panes_map_y(
                        ((*lc).g.yoff as u_int).wrapping_add((*lc).g.sy),
                        osy,
                        dsy,
                    );
                    y = y2 - 1 as ::core::ffi::c_int;
                }
                window_panes_mark_hline(map, dsx, dsy, x, x2, y);
            }
        }
        wp = (*wp).entry.tqe_next;
    }
}
unsafe extern "C" fn window_panes_get_floating_borders(
    mut wp: *mut window_pane,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
    mut xp: *mut ::core::ffi::c_int,
    mut yp: *mut ::core::ffi::c_int,
    mut x2p: *mut ::core::ffi::c_int,
    mut y2p: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    lc = (*wp).saved_layout_cell;
    if lc.is_null() {
        lc = (*wp).layout_cell as *mut layout_cell;
    }
    if lc.is_null() || !(*lc).flags & LAYOUT_CELL_FLOATING != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*lc).g.xoff == 0 as ::core::ffi::c_int {
        *xp = -(1 as ::core::ffi::c_int);
    } else {
        *xp = window_panes_map_x(((*lc).g.xoff - 1 as ::core::ffi::c_int) as u_int, osx, dsx);
    }
    if (*lc).g.yoff == 0 as ::core::ffi::c_int {
        *yp = -(1 as ::core::ffi::c_int);
    } else {
        *yp = window_panes_map_y(((*lc).g.yoff - 1 as ::core::ffi::c_int) as u_int, osy, dsy);
    }
    *x2p = window_panes_map_x(((*lc).g.xoff as u_int).wrapping_add((*lc).g.sx), osx, dsx);
    *y2p = window_panes_map_y(((*lc).g.yoff as u_int).wrapping_add((*lc).g.sy), osy, dsy);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_panes_clip_floating_pane(
    mut wp: *mut window_pane,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
    mut xp: *mut u_int,
    mut yp: *mut u_int,
    mut sxp: *mut u_int,
    mut syp: *mut u_int,
) -> ::core::ffi::c_int {
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut x2: ::core::ffi::c_int = 0;
    let mut y2: ::core::ffi::c_int = 0;
    let mut bx: ::core::ffi::c_int = 0;
    let mut by: ::core::ffi::c_int = 0;
    let mut bx2: ::core::ffi::c_int = 0;
    let mut by2: ::core::ffi::c_int = 0;
    if window_panes_get_floating_borders(
        wp,
        osx,
        osy,
        dsx,
        dsy,
        &raw mut x,
        &raw mut y,
        &raw mut x2,
        &raw mut y2,
    ) == 0
    {
        return 1 as ::core::ffi::c_int;
    }
    bx = *xp as ::core::ffi::c_int;
    by = *yp as ::core::ffi::c_int;
    bx2 = (bx as u_int).wrapping_add(*sxp).wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    by2 = (by as u_int).wrapping_add(*syp).wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    if x >= 0 as ::core::ffi::c_int && bx <= x {
        bx = x + 1 as ::core::ffi::c_int;
    }
    if y >= 0 as ::core::ffi::c_int && by <= y {
        by = y + 1 as ::core::ffi::c_int;
    }
    if (x2 as u_int) < dsx && bx2 >= x2 {
        bx2 = x2 - 1 as ::core::ffi::c_int;
    }
    if (y2 as u_int) < dsy && by2 >= y2 {
        by2 = y2 - 1 as ::core::ffi::c_int;
    }
    if bx2 < bx || by2 < by {
        return 0 as ::core::ffi::c_int;
    }
    *xp = bx as u_int;
    *yp = by as u_int;
    *sxp = (bx2 - bx + 1 as ::core::ffi::c_int) as u_int;
    *syp = (by2 - by + 1 as ::core::ffi::c_int) as u_int;
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_panes_border_cell_type(mut mask: u_char) -> ::core::ffi::c_int {
    match mask as ::core::ffi::c_int {
        15 => return 11 as ::core::ffi::c_int,
        7 => return 8 as ::core::ffi::c_int,
        11 => return 7 as ::core::ffi::c_int,
        3 | WINDOW_PANES_BORDER_L | WINDOW_PANES_BORDER_R => {
            return 2 as ::core::ffi::c_int;
        }
        13 => return 10 as ::core::ffi::c_int,
        5 => return 6 as ::core::ffi::c_int,
        9 => return 4 as ::core::ffi::c_int,
        14 => return 9 as ::core::ffi::c_int,
        6 => return 5 as ::core::ffi::c_int,
        10 => return 3 as ::core::ffi::c_int,
        12 | WINDOW_PANES_BORDER_U | WINDOW_PANES_BORDER_D => {
            return 1 as ::core::ffi::c_int;
        }
        _ => {}
    }
    return 12 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_panes_border_has_horizontal(mut mask: u_char) -> ::core::ffi::c_int {
    return (mask as ::core::ffi::c_int & (WINDOW_PANES_BORDER_L | WINDOW_PANES_BORDER_R)
        != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn window_panes_border_has_vertical(mut mask: u_char) -> ::core::ffi::c_int {
    return (mask as ::core::ffi::c_int & (WINDOW_PANES_BORDER_U | WINDOW_PANES_BORDER_D)
        != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn window_panes_mark_border_joins_cell(
    mut map: *mut u_char,
    mut lc: *mut layout_cell,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcnext: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut x2: ::core::ffi::c_int = 0;
    let mut y2: ::core::ffi::c_int = 0;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    lcchild = (*lc).cells.tqh_first;
    while !lcchild.is_null() {
        window_panes_mark_border_joins_cell(map, lcchild, osx, osy, dsx, dsy);
        if !((*lcchild).flags & LAYOUT_CELL_FLOATING != 0) {
            lcnext = window_panes_next_tiled_cell(lcchild);
            if !lcnext.is_null() {
                if (*lc).type_0 as ::core::ffi::c_uint
                    == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    x = window_panes_map_x(
                        ((*lcchild).g.xoff as u_int).wrapping_add((*lcchild).g.sx),
                        osx,
                        dsx,
                    );
                    y = window_panes_map_y((*lc).g.yoff as u_int, osy, dsy);
                    y2 = window_panes_map_y(
                        ((*lc).g.yoff as u_int).wrapping_add((*lc).g.sy),
                        osy,
                        dsy,
                    );
                    if !(x < 0 as ::core::ffi::c_int || x as u_int >= dsx) {
                        if y > 0 as ::core::ffi::c_int
                            && window_panes_border_has_horizontal(
                                *map.offset(
                                    ((y - 1 as ::core::ffi::c_int) as u_int)
                                        .wrapping_mul(dsx)
                                        .wrapping_add(x as u_int)
                                        as isize,
                                ),
                            ) != 0
                        {
                            window_panes_mark_border(
                                map,
                                dsx,
                                dsy,
                                x as u_int,
                                (y - 1 as ::core::ffi::c_int) as u_int,
                                WINDOW_PANES_BORDER_D as u_char,
                            );
                            window_panes_mark_border(
                                map,
                                dsx,
                                dsy,
                                x as u_int,
                                y as u_int,
                                WINDOW_PANES_BORDER_U as u_char,
                            );
                        }
                        if (y2 as u_int) < dsy
                            && window_panes_border_has_horizontal(
                                *map.offset(
                                    (y2 as u_int).wrapping_mul(dsx).wrapping_add(x as u_int)
                                        as isize,
                                ),
                            ) != 0
                        {
                            window_panes_mark_border(
                                map,
                                dsx,
                                dsy,
                                x as u_int,
                                y2 as u_int,
                                WINDOW_PANES_BORDER_U as u_char,
                            );
                            window_panes_mark_border(
                                map,
                                dsx,
                                dsy,
                                x as u_int,
                                (y2 - 1 as ::core::ffi::c_int) as u_int,
                                WINDOW_PANES_BORDER_D as u_char,
                            );
                        }
                    }
                } else {
                    x = window_panes_map_x((*lc).g.xoff as u_int, osx, dsx);
                    x2 = window_panes_map_x(
                        ((*lc).g.xoff as u_int).wrapping_add((*lc).g.sx),
                        osx,
                        dsx,
                    );
                    y = window_panes_map_y(
                        ((*lcchild).g.yoff as u_int).wrapping_add((*lcchild).g.sy),
                        osy,
                        dsy,
                    );
                    if !(y < 0 as ::core::ffi::c_int || y as u_int >= dsy) {
                        if x > 0 as ::core::ffi::c_int
                            && window_panes_border_has_vertical(
                                *map.offset(
                                    (y as u_int)
                                        .wrapping_mul(dsx)
                                        .wrapping_add(x as u_int)
                                        .wrapping_sub(1 as u_int)
                                        as isize,
                                ),
                            ) != 0
                        {
                            window_panes_mark_border(
                                map,
                                dsx,
                                dsy,
                                (x - 1 as ::core::ffi::c_int) as u_int,
                                y as u_int,
                                WINDOW_PANES_BORDER_R as u_char,
                            );
                            window_panes_mark_border(
                                map,
                                dsx,
                                dsy,
                                x as u_int,
                                y as u_int,
                                WINDOW_PANES_BORDER_L as u_char,
                            );
                        }
                        if (x2 as u_int) < dsx
                            && window_panes_border_has_vertical(
                                *map.offset(
                                    (y as u_int).wrapping_mul(dsx).wrapping_add(x2 as u_int)
                                        as isize,
                                ),
                            ) != 0
                        {
                            window_panes_mark_border(
                                map,
                                dsx,
                                dsy,
                                x2 as u_int,
                                y as u_int,
                                WINDOW_PANES_BORDER_L as u_char,
                            );
                            window_panes_mark_border(
                                map,
                                dsx,
                                dsy,
                                (x2 - 1 as ::core::ffi::c_int) as u_int,
                                y as u_int,
                                WINDOW_PANES_BORDER_R as u_char,
                            );
                        }
                    }
                }
            }
        }
        lcchild = (*lcchild).entry.tqe_next;
    }
}
unsafe extern "C" fn window_panes_draw_borders(
    mut ctx: *mut screen_write_ctx,
    mut w: *mut window,
    mut lc: *mut layout_cell,
    mut gc: *const grid_cell,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
) {
    let mut border_gc: grid_cell = grid_cell {
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
    let mut map: *mut u_char = ::core::ptr::null_mut::<u_char>();
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    let mut cell_type: ::core::ffi::c_int = 0;
    if dsx == 0 as u_int || dsy == 0 as u_int {
        return;
    }
    map = xcalloc(dsx as size_t, dsy as size_t) as *mut u_char;
    window_panes_mark_borders_cell(map, lc, osx, osy, dsx, dsy);
    window_panes_mark_pane_status_borders(map, w, lc, osx, osy, dsx, dsy);
    window_panes_mark_border_joins_cell(map, lc, osx, osy, dsx, dsy);
    yy = 0 as u_int;
    while yy < dsy {
        xx = 0 as u_int;
        while xx < dsx {
            if !(*map.offset(yy.wrapping_mul(dsx).wrapping_add(xx) as isize) as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int)
            {
                cell_type = window_panes_border_cell_type(
                    *map.offset(yy.wrapping_mul(dsx).wrapping_add(xx) as isize),
                );
                memcpy(
                    &raw mut border_gc as *mut ::core::ffi::c_void,
                    gc as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<grid_cell>() as size_t,
                );
                border_gc.attr =
                    (border_gc.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
                utf8_set(
                    &raw mut border_gc.data,
                    CELL_BORDERS[cell_type as usize] as u_char,
                );
                screen_write_cursormove(
                    ctx,
                    xx as ::core::ffi::c_int,
                    yy as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                screen_write_cell(ctx, &raw mut border_gc);
            }
            xx = xx.wrapping_add(1);
        }
        yy = yy.wrapping_add(1);
    }
    free(map as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_panes_draw_floating_border(
    mut ctx: *mut screen_write_ctx,
    mut wp: *mut window_pane,
    mut gc: *const grid_cell,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
) {
    let mut border_gc: grid_cell = grid_cell {
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
    let mut map: *mut u_char = ::core::ptr::null_mut::<u_char>();
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut x2: ::core::ffi::c_int = 0;
    let mut y2: ::core::ffi::c_int = 0;
    let mut cell_type: ::core::ffi::c_int = 0;
    if dsx == 0 as u_int || dsy == 0 as u_int {
        return;
    }
    if window_panes_get_floating_borders(
        wp,
        osx,
        osy,
        dsx,
        dsy,
        &raw mut x,
        &raw mut y,
        &raw mut x2,
        &raw mut y2,
    ) == 0
    {
        return;
    }
    map = xcalloc(dsx as size_t, dsy as size_t) as *mut u_char;
    window_panes_mark_hline(map, dsx, dsy, x, x2 + 1 as ::core::ffi::c_int, y);
    window_panes_mark_hline(map, dsx, dsy, x, x2 + 1 as ::core::ffi::c_int, y2);
    window_panes_mark_vline(map, dsx, dsy, x, y, y2 + 1 as ::core::ffi::c_int);
    window_panes_mark_vline(map, dsx, dsy, x2, y, y2 + 1 as ::core::ffi::c_int);
    yy = 0 as u_int;
    while yy < dsy {
        xx = 0 as u_int;
        while xx < dsx {
            if !(*map.offset(yy.wrapping_mul(dsx).wrapping_add(xx) as isize) as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int)
            {
                cell_type = window_panes_border_cell_type(
                    *map.offset(yy.wrapping_mul(dsx).wrapping_add(xx) as isize),
                );
                memcpy(
                    &raw mut border_gc as *mut ::core::ffi::c_void,
                    gc as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<grid_cell>() as size_t,
                );
                border_gc.attr =
                    (border_gc.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
                utf8_set(
                    &raw mut border_gc.data,
                    CELL_BORDERS[cell_type as usize] as u_char,
                );
                screen_write_cursormove(
                    ctx,
                    xx as ::core::ffi::c_int,
                    yy as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                screen_write_cell(ctx, &raw mut border_gc);
            }
            xx = xx.wrapping_add(1);
        }
        yy = yy.wrapping_add(1);
    }
    free(map as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_panes_clear_floating_area(
    mut ctx: *mut screen_write_ctx,
    mut wp: *mut window_pane,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
) {
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
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut x2: ::core::ffi::c_int = 0;
    let mut y2: ::core::ffi::c_int = 0;
    let mut xx: ::core::ffi::c_int = 0;
    let mut yy: ::core::ffi::c_int = 0;
    if window_panes_get_floating_borders(
        wp,
        osx,
        osy,
        dsx,
        dsy,
        &raw mut x,
        &raw mut y,
        &raw mut x2,
        &raw mut y2,
    ) == 0
    {
        return;
    }
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    if x < 0 as ::core::ffi::c_int {
        x = 0 as ::core::ffi::c_int;
    }
    if y < 0 as ::core::ffi::c_int {
        y = 0 as ::core::ffi::c_int;
    }
    if x2 as u_int >= dsx {
        x2 = dsx.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    }
    if y2 as u_int >= dsy {
        y2 = dsy.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    }
    if x2 < x || y2 < y {
        return;
    }
    yy = y;
    while yy <= y2 {
        screen_write_cursormove(ctx, x, yy, 0 as ::core::ffi::c_int);
        xx = x;
        while xx <= x2 {
            screen_write_putc(ctx, &raw mut gc, ' ' as i32 as u_char);
            xx += 1;
        }
        yy += 1;
    }
}
unsafe extern "C" fn window_panes_draw_format(
    mut data: *mut window_panes_modedata,
    mut ctx: *mut screen_write_ctx,
    mut wp: *mut window_pane,
    mut x: u_int,
    mut y: u_int,
    mut sx: u_int,
    mut gc: *const grid_cell,
) {
    let mut s: *mut session = (*data).session;
    let mut wl: *mut winlink = (*s).curw;
    let mut format: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if sx == 0 as u_int {
        return;
    }
    format = options_get_string(
        (*(*(*data).wp).window).options,
        b"display-panes-format\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if *format as ::core::ffi::c_int == '\0' as i32 {
        return;
    }
    window_panes_get_source(
        data,
        &raw mut s,
        &raw mut wl,
        ::core::ptr::null_mut::<*mut window>(),
    );
    if s.is_null() {
        return;
    }
    expanded = format_single(
        ::core::ptr::null_mut::<cmdq_item>(),
        format,
        ::core::ptr::null_mut::<client>(),
        s,
        wl,
        wp,
    );
    if *expanded as ::core::ffi::c_int != '\0' as i32 {
        screen_write_cursormove(
            ctx,
            x as ::core::ffi::c_int,
            y as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        format_draw(
            ctx,
            gc,
            sx,
            expanded,
            ::core::ptr::null_mut::<style_ranges>(),
            0 as ::core::ffi::c_int,
        );
    }
    free(expanded as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_panes_draw_number(
    mut data: *mut window_panes_modedata,
    mut ctx: *mut screen_write_ctx,
    mut wp: *mut window_pane,
    mut pane: u_int,
    mut x: u_int,
    mut y: u_int,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut s: *mut session = (*data).session;
    let mut w: *mut window = (*wp).window as *mut window;
    let mut wl: *mut winlink = (*s).curw;
    let mut oo: *mut options = (*(*(*data).wp).window).options;
    let mut fgc: grid_cell = grid_cell {
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
    let mut bgc: grid_cell = grid_cell {
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
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut buf: [::core::ffi::c_char; 16] = [0; 16];
    let mut lbuf: [::core::ffi::c_char; 16] = [
        0 as ::core::ffi::c_int as ::core::ffi::c_char,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut llen: size_t = 0 as size_t;
    let mut width: size_t = 0;
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut idx: u_int = 0;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut format: u_int = 0;
    len = xsnprintf(
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        pane,
    ) as size_t;
    if pane > 9 as u_int && pane < 35 as u_int {
        llen = xsnprintf(
            &raw mut lbuf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
            b"%c\0" as *const u8 as *const ::core::ffi::c_char,
            ('a' as i32 as u_int).wrapping_add(pane.wrapping_sub(10 as u_int)),
        ) as size_t;
    }
    if (sx as size_t) < len {
        return;
    }
    window_panes_get_source(
        data,
        &raw mut s,
        &raw mut wl,
        ::core::ptr::null_mut::<*mut window>(),
    );
    if !s.is_null() {
        if wl.is_null() {
            wl = (*s).curw;
        }
    }
    if (*w).active == wp {
        name = b"display-panes-active-colour\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        name = b"display-panes-colour\0" as *const u8 as *const ::core::ffi::c_char;
    }
    ft = format_create_defaults(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        s,
        wl,
        wp,
    );
    style_apply(&raw mut fgc, oo, name, ft);
    format_free(ft);
    memcpy(
        &raw mut bgc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    bgc.bg = fgc.fg;
    format = 0 as u_int;
    if *options_get_string(
        oo,
        b"display-panes-format\0" as *const u8 as *const ::core::ffi::c_char,
    )
    .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        != '\0' as i32
    {
        format = 1 as u_int;
    }
    width = len.wrapping_mul(6 as size_t).wrapping_sub(1 as size_t);
    if (sx as size_t) < width
        || sy
            < (if format != 0 {
                7 as ::core::ffi::c_int
            } else {
                5 as ::core::ffi::c_int
            }) as u_int
    {
        width = len;
        if llen != 0 as size_t && sx as size_t >= len.wrapping_add(llen).wrapping_add(1 as size_t) {
            width = width.wrapping_add(llen.wrapping_add(1 as size_t));
        }
        cx = (x as size_t)
            .wrapping_add((sx as size_t).wrapping_sub(width).wrapping_div(2 as size_t))
            as u_int;
        cy = y.wrapping_add(sy.wrapping_div(2 as u_int));
        screen_write_cursormove(
            ctx,
            cx as ::core::ffi::c_int,
            cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_puts(
            ctx,
            &raw mut fgc,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if width > len {
            screen_write_puts(
                ctx,
                &raw mut fgc,
                b" %s\0" as *const u8 as *const ::core::ffi::c_char,
                &raw mut lbuf as *mut ::core::ffi::c_char,
            );
        }
        if format != 0 && sy > 1 as u_int {
            window_panes_draw_format(data, ctx, wp, x, y, sx, &raw mut fgc);
        }
        return;
    }
    px = (sx as size_t).wrapping_sub(width).wrapping_div(2 as size_t) as u_int;
    py = sy.wrapping_sub(5 as u_int).wrapping_div(2 as u_int);
    ptr = &raw mut buf as *mut ::core::ffi::c_char;
    while *ptr as ::core::ffi::c_int != '\0' as i32 {
        if !((*ptr as ::core::ffi::c_int) < '0' as i32 || *ptr as ::core::ffi::c_int > '9' as i32) {
            idx = (*ptr as ::core::ffi::c_int - '0' as i32) as u_int;
            j = 0 as u_int;
            while j < 5 as u_int {
                i = 0 as u_int;
                while i < 5 as u_int {
                    if !(window_clock_table[idx as usize][j as usize][i as usize] == 0) {
                        screen_write_cursormove(
                            ctx,
                            x.wrapping_add(px).wrapping_add(i) as ::core::ffi::c_int,
                            y.wrapping_add(py).wrapping_add(j) as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        );
                        screen_write_putc(ctx, &raw mut bgc, ' ' as i32 as u_char);
                    }
                    i = i.wrapping_add(1);
                }
                j = j.wrapping_add(1);
            }
            px = px.wrapping_add(6 as u_int);
        }
        ptr = ptr.offset(1);
    }
    if sy <= 6 as u_int {
        return;
    }
    window_panes_draw_format(data, ctx, wp, x, y, sx, &raw mut fgc);
    if llen != 0 as size_t {
        cx = (x.wrapping_add(px) as size_t)
            .wrapping_sub(llen)
            .wrapping_sub(1 as size_t) as u_int;
        cy = y.wrapping_add(py).wrapping_add(5 as u_int);
        screen_write_cursormove(
            ctx,
            cx as ::core::ffi::c_int,
            cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_puts(
            ctx,
            &raw mut fgc,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut lbuf as *mut ::core::ffi::c_char,
        );
    }
}
unsafe extern "C" fn window_panes_draw_pane(
    mut data: *mut window_panes_modedata,
    mut ctx: *mut screen_write_ctx,
    mut wp: *mut window_pane,
    mut root: *mut layout_cell,
    mut osx: u_int,
    mut osy: u_int,
    mut dsx: u_int,
    mut dsy: u_int,
) {
    let mut s: *mut screen = &raw mut (*wp).base;
    let mut pane: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if window_panes_pane_visible(wp) == 0 {
        return;
    }
    if window_panes_get_geometry(
        wp,
        root,
        osx,
        osy,
        dsx,
        dsy,
        &raw mut x,
        &raw mut y,
        &raw mut sx,
        &raw mut sy,
    ) == 0
    {
        return;
    }
    if window_panes_clip_floating_pane(
        wp,
        osx,
        osy,
        dsx,
        dsy,
        &raw mut x,
        &raw mut y,
        &raw mut sx,
        &raw mut sy,
    ) == 0
    {
        return;
    }
    if window_pane_index(wp, &raw mut pane) != 0 as ::core::ffi::c_int {
        return;
    }
    window_panes_add_area(data, wp, x, y, sx, sy);
    screen_write_cursormove(
        ctx,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if !(*data).preview.is_null()
        && wp == (*data).wp
        && sx <= (*(*(*data).preview).grid).sx
        && sy <= (*(*(*data).preview).grid).sy
    {
        s = (*data).preview;
    }
    if osx <= dsx && osy <= dsy {
        screen_write_fast_copy(ctx, s, 0 as u_int, (*(*s).grid).hsize, sx, sy);
    } else {
        screen_write_preview(ctx, s, sx, sy);
    }
    window_panes_draw_number(data, ctx, wp, pane, x, y, sx, sy);
}
unsafe extern "C" fn window_panes_draw_screen(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_panes_modedata = (*wme).data as *mut window_panes_modedata;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
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
    let mut root: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut border_gc: grid_cell = grid_cell {
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
    let mut osx: u_int = 0;
    let mut osy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if window_panes_get_source(
        data,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
        &raw mut w,
    ) == 0
    {
        return;
    }
    root = (*w).saved_layout_root;
    if root.is_null() {
        root = (*w).layout_root;
    }
    if root.is_null() {
        return;
    }
    osx = (*root).g.sx;
    osy = (*root).g.sy;
    sx = (*(*data).screen.grid).sx;
    sy = (*(*data).screen.grid).sy;
    window_panes_free_areas(data);
    screen_write_start(&raw mut ctx, &raw mut (*data).screen);
    screen_write_clearscreen(&raw mut ctx, 8 as u_int);
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if !(window_panes_pane_floating(wp) != 0) {
            window_panes_draw_pane(data, &raw mut ctx, wp, root, osx, osy, sx, sy);
        }
        wp = (*wp).entry.tqe_next;
    }
    window_panes_get_border_cell(data, &raw mut border_gc);
    window_panes_draw_borders(&raw mut ctx, w, root, &raw mut border_gc, osx, osy, sx, sy);
    wp = *(*((*w).z_index.tqh_last as *mut window_panes_zindex)).tqh_last;
    while !wp.is_null() {
        if !(window_panes_pane_floating(wp) == 0) {
            window_panes_clear_floating_area(&raw mut ctx, wp, osx, osy, sx, sy);
            window_panes_draw_pane(data, &raw mut ctx, wp, root, osx, osy, sx, sy);
            window_panes_draw_floating_border(
                &raw mut ctx,
                wp,
                &raw mut border_gc,
                osx,
                osy,
                sx,
                sy,
            );
        }
        wp = *(*((*wp).zentry.tqe_prev as *mut window_panes_zindex)).tqh_last;
    }
    screen_write_stop(&raw mut ctx);
    (*(*data).wp).flags |= PANE_REDRAW;
}
unsafe extern "C" fn window_panes_timer_callback(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut wme: *mut window_mode_entry = arg as *mut window_mode_entry;
    window_pane_reset_mode((*wme).wp);
}
unsafe extern "C" fn window_panes_init(
    mut wme: *mut window_mode_entry,
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut w: *mut window = (*wp).window as *mut window;
    let mut data: *mut window_panes_modedata = ::core::ptr::null_mut::<window_panes_modedata>();
    let mut self_0: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut source: *mut cmd_find_state = ::core::ptr::null_mut::<cmd_find_state>();
    let mut target: *mut cmd_find_state = ::core::ptr::null_mut::<cmd_find_state>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut sx: u_int = (*(*wp).base.grid).sx;
    let mut sy: u_int = (*(*wp).base.grid).sy;
    let mut delay: u_int = 0;
    if item.is_null() {
        return ::core::ptr::null_mut::<screen>();
    }
    self_0 = cmdq_get_cmd(item);
    if self_0.is_null() {
        return ::core::ptr::null_mut::<screen>();
    }
    source = cmdq_get_source(item);
    target = cmdq_get_target(item);
    s = (*target).s;
    if args_has(args, 'd' as i32 as u_char) == 0 {
        delay = options_get_number(
            (*w).options,
            b"display-panes-time\0" as *const u8 as *const ::core::ffi::c_char,
        ) as u_int;
    } else {
        delay = args_strtonum(
            args,
            'd' as i32 as u_char,
            0 as ::core::ffi::c_longlong,
            UINT_MAX as ::core::ffi::c_longlong,
            &raw mut cause,
        ) as u_int;
        if !cause.is_null() {
            cmdq_error(
                item,
                b"delay %s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<screen>();
        }
    }
    data = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_panes_modedata>() as size_t,
    ) as *mut window_panes_modedata;
    (*wme).data = data as *mut ::core::ffi::c_void;
    (*data).wp = wp;
    (*data).session = s;
    screen_init(&raw mut (*data).screen, sx, sy, 0 as u_int);
    (*data).screen.mode &= !MODE_CURSOR;
    (*data).state = args_make_commands_prepare(
        self_0,
        item,
        0 as u_int,
        b"select-pane -t \"%%%\"\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if args_has(args, 's' as i32 as u_char) != 0 {
        (*data).source_session = (*(*source).s).id;
        (*data).source_window = (*(*source).w).id;
    } else {
        (*data).source_session = (*(*target).s).id;
        (*data).source_window = (*(*target).w).id;
    }
    (*data).delay = delay;
    (*data).ignore_keys = args_has(args, 'N' as i32 as u_char);
    if args_has(args, 'Z' as i32 as u_char) != 0 {
        (*data).zoomed = -(1 as ::core::ffi::c_int);
    } else {
        (*data).zoomed = (*w).flags & WINDOW_ZOOMED;
        if (*data).zoomed == 0 {
            window_panes_set_preview(data);
        }
        if (*data).zoomed == 0 && window_zoom(wp) == 0 as ::core::ffi::c_int {
            server_redraw_window(w);
        }
    }
    event_set(
        &raw mut (*data).timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            window_panes_timer_callback
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        wme as *mut ::core::ffi::c_void,
    );
    if (*data).delay != 0 as u_int {
        tv.tv_sec = (*data).delay.wrapping_div(1000 as u_int) as __time_t;
        tv.tv_usec = (*data)
            .delay
            .wrapping_rem(1000 as u_int)
            .wrapping_mul(1000 as u_int) as __suseconds_t;
        event_add(&raw mut (*data).timer, &raw mut tv);
    }
    window_panes_draw_screen(wme);
    return &raw mut (*data).screen;
}
unsafe extern "C" fn window_panes_free(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_panes_modedata = (*wme).data as *mut window_panes_modedata;
    let mut w: *mut window = (*(*wme).wp).window as *mut window;
    event_del(&raw mut (*data).timer);
    if (*data).zoomed == 0 as ::core::ffi::c_int {
        server_unzoom_window(w);
    }
    server_redraw_window(w);
    server_redraw_window_borders(w);
    server_status_window(w);
    if !(*data).state.is_null() {
        args_make_commands_free((*data).state);
    }
    window_panes_free_areas(data);
    if !(*data).preview.is_null() {
        screen_free((*data).preview);
        free((*data).preview as *mut ::core::ffi::c_void);
    }
    screen_free(&raw mut (*data).screen);
    free(data as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_panes_resize(
    mut wme: *mut window_mode_entry,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_panes_modedata = (*wme).data as *mut window_panes_modedata;
    screen_resize(&raw mut (*data).screen, sx, sy, 0 as ::core::ffi::c_int);
    window_panes_draw_screen(wme);
}
unsafe extern "C" fn window_panes_run_command(
    mut data: *mut window_panes_modedata,
    mut c: *mut client,
    mut wp: *mut window_pane,
) {
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    xasprintf(
        &raw mut expanded,
        b"%%%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
    cmdlist = args_make_commands(
        (*data).state,
        1 as ::core::ffi::c_int,
        &raw mut expanded,
        &raw mut error,
    );
    if cmdlist.is_null() {
        cmdq_append(c, cmdq_get_error(error));
        free(error as *mut ::core::ffi::c_void);
    } else {
        new_item = cmdq_get_command(cmdlist, ::core::ptr::null_mut::<cmdq_state>());
        cmdq_append(c, new_item);
        cmd_list_free(cmdlist);
    }
    free(expanded as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_panes_find_pane(
    mut data: *mut window_panes_modedata,
    mut x: u_int,
    mut y: u_int,
) -> *mut window_pane {
    let mut area: *mut window_panes_area = ::core::ptr::null_mut::<window_panes_area>();
    let mut i: u_int = 0;
    i = (*data).areas_size;
    while i > 0 as u_int {
        area = (*data).areas.offset(i.wrapping_sub(1 as u_int) as isize) as *mut window_panes_area;
        if !(x < (*area).x || x >= (*area).x.wrapping_add((*area).sx)) {
            if !(y < (*area).y || y >= (*area).y.wrapping_add((*area).sy)) {
                return window_pane_find_by_id((*area).id);
            }
        }
        i = i.wrapping_sub(1);
    }
    return ::core::ptr::null_mut::<window_pane>();
}
unsafe extern "C" fn window_panes_key_pane(
    mut data: *mut window_panes_modedata,
    mut key: key_code,
) -> *mut window_pane {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut index: u_int = 0;
    if key >= '0' as i32 as key_code && key <= '9' as i32 as key_code {
        index = key.wrapping_sub('0' as i32 as key_code) as u_int;
    } else if key as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS == 0 as ::core::ffi::c_ulonglong
    {
        key &= KEYC_MASK_KEY;
        if key < 'a' as i32 as key_code || key > 'z' as i32 as key_code {
            return ::core::ptr::null_mut::<window_pane>();
        }
        index = (10 as key_code).wrapping_add(key.wrapping_sub('a' as i32 as key_code)) as u_int;
    } else {
        return ::core::ptr::null_mut::<window_pane>();
    }
    if window_panes_get_source(
        data,
        ::core::ptr::null_mut::<*mut session>(),
        ::core::ptr::null_mut::<*mut winlink>(),
        &raw mut w,
    ) == 0
    {
        return ::core::ptr::null_mut::<window_pane>();
    }
    return window_pane_at_index(w, index);
}
unsafe extern "C" fn window_panes_get_target(
    mut wme: *mut window_mode_entry,
    mut key: key_code,
    mut m: *mut mouse_event,
) -> *mut window_pane {
    let mut data: *mut window_panes_modedata = (*wme).data as *mut window_panes_modedata;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if (*data).ignore_keys != 0 {
        return ::core::ptr::null_mut::<window_pane>();
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        if key != KEYC_MOUSEDOWN1_PANE as ::core::ffi::c_ulong as key_code
            || m.is_null()
            || cmd_mouse_at(
                (*wme).wp,
                m,
                &raw mut x,
                &raw mut y,
                0 as ::core::ffi::c_int,
            ) != 0 as ::core::ffi::c_int
        {
            return ::core::ptr::null_mut::<window_pane>();
        }
        return window_panes_find_pane(data, x, y);
    }
    return window_panes_key_pane(data, key);
}
unsafe extern "C" fn window_panes_key(
    mut wme: *mut window_mode_entry,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut target: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut data: *mut window_panes_modedata = (*wme).data as *mut window_panes_modedata;
    if key == '\u{1b}' as i32 as key_code || key == 'q' as i32 as key_code {
        window_pane_reset_mode(wp);
        return;
    }
    target = window_panes_get_target(wme, key, m);
    if target.is_null() {
        if (*data).ignore_keys == 0
            && !(key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int
                    && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                        <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                            as ::core::ffi::c_ulonglong)
                            << 32 as ::core::ffi::c_int)
        {
            window_pane_reset_mode(wp);
        }
        return;
    }
    if (*(*wp).window).flags & WINDOW_ZOOMED != 0 {
        window_unzoom((*wp).window as *mut window, 1 as ::core::ffi::c_int);
    }
    window_panes_run_command(data, c, target);
    window_pane_reset_mode(wp);
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

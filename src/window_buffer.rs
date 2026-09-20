pub use crate::src::shared::pane::{
    PANE_REDRAW, window_pane_offset, window_pane_resize, window_pane_resize_entry,
    window_pane_resizes,
};
pub use crate::src::shared::menu::{menu_item};
pub use crate::src::shared::abi::{__int32_t, ssize_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_citem, screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::sort::{sort_criteria};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::layout::*;
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
    pub type mode_tree_data;
    pub type mode_tree_item;
    fn __ctype_tolower_loc() -> *mut *const __int32_t;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strcasestr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xsnprintf(
        _: *mut ::core::ffi::c_char,
        _: size_t,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn paste_buffer_name(_: *mut paste_buffer) -> *const ::core::ffi::c_char;
    fn paste_buffer_order(_: *mut paste_buffer) -> u_int;
    fn paste_buffer_data(_: *mut paste_buffer, _: *mut size_t) -> *const ::core::ffi::c_char;
    fn paste_is_empty() -> ::core::ffi::c_int;
    fn paste_get_name(_: *const ::core::ffi::c_char) -> *mut paste_buffer;
    fn paste_free(_: *mut paste_buffer);
    fn paste_replace(_: *mut paste_buffer, _: *mut ::core::ffi::c_char, _: size_t);
    fn sort_get_buffers(_: *mut u_int, _: *mut sort_criteria) -> *mut *mut paste_buffer;
    fn format_true(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn format_create(
        _: *mut client,
        _: *mut cmdq_item,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut format_tree;
    fn format_free(_: *mut format_tree);
    fn format_add(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn format_expand(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn format_defaults(
        _: *mut format_tree,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    );
    fn format_defaults_paste_buffer(_: *mut format_tree, _: *mut paste_buffer);
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn cmd_find_valid_state(_: *mut cmd_find_state) -> ::core::ffi::c_int;
    fn cmd_find_copy_state(_: *mut cmd_find_state, _: *mut cmd_find_state);
    fn key_string_lookup_string(_: *const ::core::ffi::c_char) -> key_code;
    static grid_default_cell: grid_cell;
    fn screen_write_start(_: *mut screen_write_ctx, _: *mut screen);
    fn screen_write_stop(_: *mut screen_write_ctx);
    fn screen_write_nputs(
        _: *mut screen_write_ctx,
        _: ssize_t,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn screen_write_box(
        _: *mut screen_write_ctx,
        _: u_int,
        _: u_int,
        _: box_lines,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
    );
    fn screen_write_clearcharacter(_: *mut screen_write_ctx, _: u_int, _: u_int);
    fn screen_write_cursormove(
        _: *mut screen_write_ctx,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn window_pane_find_by_id(_: u_int) -> *mut window_pane;
    fn window_pane_reset_mode(_: *mut window_pane);
    fn mode_tree_get_current(_: *mut mode_tree_data) -> *mut ::core::ffi::c_void;
    fn mode_tree_each_tagged(
        _: *mut mode_tree_data,
        _: mode_tree_each_cb,
        _: *mut client,
        _: key_code,
        _: ::core::ffi::c_int,
    );
    fn mode_tree_up(_: *mut mode_tree_data, _: ::core::ffi::c_int);
    fn mode_tree_down(_: *mut mode_tree_data, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn mode_tree_start(
        _: *mut window_pane,
        _: *mut args,
        _: mode_tree_build_cb,
        _: mode_tree_draw_cb,
        _: mode_tree_search_cb,
        _: mode_tree_menu_cb,
        _: mode_tree_height_cb,
        _: mode_tree_key_cb,
        _: mode_tree_swap_cb,
        _: mode_tree_sort_cb,
        _: mode_tree_help_cb,
        _: *mut ::core::ffi::c_void,
        _: *const menu_item,
        _: *mut *mut screen,
    ) -> *mut mode_tree_data;
    fn mode_tree_zoom(_: *mut mode_tree_data, _: *mut args);
    fn mode_tree_build(_: *mut mode_tree_data);
    fn mode_tree_free(_: *mut mode_tree_data);
    fn mode_tree_resize(_: *mut mode_tree_data, _: u_int, _: u_int);
    fn mode_tree_add(
        _: *mut mode_tree_data,
        _: *mut mode_tree_item,
        _: *mut ::core::ffi::c_void,
        _: uint64_t,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut mode_tree_item;
    fn mode_tree_draw(_: *mut mode_tree_data);
    fn mode_tree_key(
        _: *mut mode_tree_data,
        _: *mut client,
        _: *mut key_code,
        _: *mut mouse_event,
        _: *mut u_int,
        _: *mut u_int,
    ) -> ::core::ffi::c_int;
    fn mode_tree_run_command(
        _: *mut client,
        _: *mut cmd_find_state,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
    );
    fn utf8_strvis(
        _: *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: size_t,
        _: ::core::ffi::c_int,
    ) -> size_t;
    fn spawn_editor(
        _: *mut client,
        _: *const ::core::ffi::c_char,
        _: size_t,
        _: spawn_finish_edit_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut spawn_editor_state;
    fn spawn_cancel_editor(_: *mut spawn_editor_state);
    fn spawn_get_editor_pid(_: *mut spawn_editor_state) -> pid_t;
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
pub struct paste_buffer {
    pub data: *mut ::core::ffi::c_char,
    pub size: size_t,
    pub name: *mut ::core::ffi::c_char,
    pub created: time_t,
    pub automatic: ::core::ffi::c_int,
    pub order: u_int,
    pub name_entry: C2RustUnnamed_40,
    pub time_entry: C2RustUnnamed_39,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_39 {
    pub rbe_left: *mut paste_buffer,
    pub rbe_right: *mut paste_buffer,
    pub rbe_parent: *mut paste_buffer,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_40 {
    pub rbe_left: *mut paste_buffer,
    pub rbe_right: *mut paste_buffer,
    pub rbe_parent: *mut paste_buffer,
    pub rbe_color: ::core::ffi::c_int,
}
pub type mode_tree_build_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *mut sort_criteria,
        *mut uint64_t,
        *const ::core::ffi::c_char,
    ) -> (),
>;
pub type mode_tree_draw_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *mut ::core::ffi::c_void,
        *mut screen_write_ctx,
        u_int,
        u_int,
    ) -> (),
>;
pub type mode_tree_search_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *mut ::core::ffi::c_void,
        *const ::core::ffi::c_char,
        ::core::ffi::c_int,
    ) -> ::core::ffi::c_int,
>;
pub type mode_tree_menu_cb =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut client, key_code) -> ()>;
pub type mode_tree_height_cb =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, u_int) -> u_int>;
pub type mode_tree_key_cb = Option<
    unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut ::core::ffi::c_void, u_int) -> key_code,
>;
pub type mode_tree_swap_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *mut ::core::ffi::c_void,
        *mut sort_criteria,
    ) -> ::core::ffi::c_int,
>;
pub type mode_tree_sort_cb = Option<unsafe extern "C" fn(*mut sort_criteria) -> ()>;
pub type mode_tree_each_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *mut ::core::ffi::c_void,
        *mut client,
        key_code,
    ) -> (),
>;
pub type mode_tree_help_cb = Option<
    unsafe extern "C" fn(
        *mut u_int,
        *mut *const ::core::ffi::c_char,
    ) -> *mut *const ::core::ffi::c_char,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_buffer_modedata {
    pub wp: *mut window_pane,
    pub fs: cmd_find_state,
    pub data: *mut mode_tree_data,
    pub editor: *mut spawn_editor_state,
    pub edit: *mut window_buffer_editdata,
    pub command: *mut ::core::ffi::c_char,
    pub format: *mut ::core::ffi::c_char,
    pub key_format: *mut ::core::ffi::c_char,
    pub item_list: *mut *mut window_buffer_itemdata,
    pub item_size: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_buffer_itemdata {
    pub name: *const ::core::ffi::c_char,
    pub order: u_int,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_buffer_editdata {
    pub wp_id: u_int,
    pub name: *mut ::core::ffi::c_char,
    pub pb: *mut paste_buffer,
    pub editor: *mut spawn_editor_state,
}
pub type spawn_finish_edit_cb =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_char, size_t, *mut ::core::ffi::c_void) -> ()>;
#[inline]
unsafe extern "C" fn tolower(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_tolower_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}
pub const VIS_OCTAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VIS_CSTYLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VIS_TAB: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const FORMAT_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const WINDOW_BUFFER_DEFAULT_COMMAND: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(*b"paste-buffer -p -b '%%'\0")
};
pub const WINDOW_BUFFER_DEFAULT_FORMAT: [::core::ffi::c_char; 40] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_char; 40]>(
        *b"#{t/p:buffer_created}: #{buffer_sample}\0",
    )
};
pub const WINDOW_BUFFER_DEFAULT_KEY_FORMAT: [::core::ffi::c_char; 83] = unsafe {
    ::core::mem::transmute::<[u8; 83], [::core::ffi::c_char; 83]>(
        *b"#{?#{e|<:#{line},10},#{line},#{e|<:#{line},36},M-#{a:#{e|+:97,#{e|-:#{line},10}}}}\0",
    )
};
static mut window_buffer_menu_items: [menu_item; 12] = [
    menu_item {
        name: b"Paste\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'p' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Paste Tagged\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'P' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Tag\0" as *const u8 as *const ::core::ffi::c_char,
        key: 't' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Tag All\0" as *const u8 as *const ::core::ffi::c_char,
        key: '\u{14}' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Tag None\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'T' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Delete\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'd' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Delete Tagged\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'D' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Cancel\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'q' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];
#[no_mangle]
pub static mut window_buffer_mode: window_mode = unsafe {
    window_mode {
        name: b"buffer-mode\0" as *const u8 as *const ::core::ffi::c_char,
        default_format: WINDOW_BUFFER_DEFAULT_FORMAT.as_ptr(),
        flags: 0,
        init: Some(
            window_buffer_init
                as unsafe extern "C" fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_buffer_free as unsafe extern "C" fn(*mut window_mode_entry) -> ()),
        resize: Some(
            window_buffer_resize
                as unsafe extern "C" fn(*mut window_mode_entry, u_int, u_int) -> (),
        ),
        update: Some(window_buffer_update as unsafe extern "C" fn(*mut window_mode_entry) -> ()),
        style_changed: None,
        key: Some(
            window_buffer_key
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
static mut window_buffer_order_seq: [sort_order; 4] =
    [SORT_CREATION, SORT_NAME, SORT_SIZE, SORT_END];
unsafe extern "C" fn window_buffer_add_item(
    mut data: *mut window_buffer_modedata,
) -> *mut window_buffer_itemdata {
    let mut item: *mut window_buffer_itemdata = ::core::ptr::null_mut::<window_buffer_itemdata>();
    (*data).item_list = xreallocarray(
        (*data).item_list as *mut ::core::ffi::c_void,
        (*data).item_size.wrapping_add(1 as u_int) as size_t,
        ::core::mem::size_of::<*mut window_buffer_itemdata>() as size_t,
    ) as *mut *mut window_buffer_itemdata;
    let fresh0 = (*data).item_size;
    (*data).item_size = (*data).item_size.wrapping_add(1);
    let ref mut fresh1 = *(*data).item_list.offset(fresh0 as isize);
    *fresh1 = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_buffer_itemdata>() as size_t,
    ) as *mut window_buffer_itemdata;
    item = *fresh1;
    return item;
}
unsafe extern "C" fn window_buffer_free_item(mut item: *mut window_buffer_itemdata) {
    free((*item).name as *mut ::core::ffi::c_void);
    free(item as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_buffer_build(
    mut modedata: *mut ::core::ffi::c_void,
    mut sort_crit: *mut sort_criteria,
    mut tag: *mut uint64_t,
    mut filter: *const ::core::ffi::c_char,
) {
    let mut data: *mut window_buffer_modedata = modedata as *mut window_buffer_modedata;
    let mut item: *mut window_buffer_itemdata = ::core::ptr::null_mut::<window_buffer_itemdata>();
    let mut i: u_int = 0;
    let mut n: u_int = 0;
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut l: *mut *mut paste_buffer = ::core::ptr::null_mut::<*mut paste_buffer>();
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    i = 0 as u_int;
    while i < (*data).item_size {
        window_buffer_free_item(*(*data).item_list.offset(i as isize));
        i = i.wrapping_add(1);
    }
    free((*data).item_list as *mut ::core::ffi::c_void);
    (*data).item_list = ::core::ptr::null_mut::<*mut window_buffer_itemdata>();
    (*data).item_size = 0 as u_int;
    l = sort_get_buffers(&raw mut n, sort_crit);
    i = 0 as u_int;
    while i < n {
        item = window_buffer_add_item(data);
        (*item).name = xstrdup(paste_buffer_name(*l.offset(i as isize)));
        paste_buffer_data(*l.offset(i as isize), &raw mut (*item).size);
        (*item).order = paste_buffer_order(*l.offset(i as isize));
        i = i.wrapping_add(1);
    }
    if cmd_find_valid_state(&raw mut (*data).fs) != 0 {
        s = (*data).fs.s;
        wl = (*data).fs.wl;
        wp = (*data).fs.wp;
    }
    let mut current_block_32: u64;
    i = 0 as u_int;
    while i < (*data).item_size {
        item = *(*data).item_list.offset(i as isize);
        pb = paste_get_name((*item).name);
        if !pb.is_null() {
            ft = format_create(
                ::core::ptr::null_mut::<client>(),
                ::core::ptr::null_mut::<cmdq_item>(),
                FORMAT_NONE,
                0 as ::core::ffi::c_int,
            );
            format_defaults(ft, ::core::ptr::null_mut::<client>(), s, wl, wp);
            format_defaults_paste_buffer(ft, pb);
            if !filter.is_null() {
                cp = format_expand(ft, filter);
                if format_true(cp) == 0 {
                    free(cp as *mut ::core::ffi::c_void);
                    format_free(ft);
                    current_block_32 = 5948590327928692120;
                } else {
                    free(cp as *mut ::core::ffi::c_void);
                    current_block_32 = 1608152415753874203;
                }
            } else {
                current_block_32 = 1608152415753874203;
            }
            match current_block_32 {
                5948590327928692120 => {}
                _ => {
                    text = format_expand(ft, (*data).format);
                    mode_tree_add(
                        (*data).data,
                        ::core::ptr::null_mut::<mode_tree_item>(),
                        item as *mut ::core::ffi::c_void,
                        (*item).order as uint64_t,
                        (*item).name,
                        text,
                        -(1 as ::core::ffi::c_int),
                    );
                    free(text as *mut ::core::ffi::c_void);
                    format_free(ft);
                }
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn window_buffer_draw(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut item: *mut window_buffer_itemdata = itemdata as *mut window_buffer_itemdata;
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut pdata: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut start: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut psize: size_t = 0;
    let mut i: u_int = 0;
    let mut cx: u_int = (*(*ctx).s).cx;
    let mut cy: u_int = (*(*ctx).s).cy;
    pb = paste_get_name((*item).name);
    if pb.is_null() {
        return;
    }
    end = paste_buffer_data(pb, &raw mut psize);
    pdata = end;
    i = 0 as u_int;
    while i < sy {
        start = end;
        while end != pdata.offset(psize as isize) && *end as ::core::ffi::c_int != '\n' as i32 {
            end = end.offset(1);
        }
        buf = xreallocarray(
            buf as *mut ::core::ffi::c_void,
            4 as size_t,
            (end.offset_from(start) as ::core::ffi::c_long + 1 as ::core::ffi::c_long) as size_t,
        ) as *mut ::core::ffi::c_char;
        utf8_strvis(
            buf,
            start,
            end.offset_from(start) as ::core::ffi::c_long as size_t,
            VIS_OCTAL | VIS_CSTYLE | VIS_TAB,
        );
        if *buf as ::core::ffi::c_int != '\0' as i32 {
            screen_write_cursormove(
                ctx,
                cx as ::core::ffi::c_int,
                cy.wrapping_add(i) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_nputs(
                ctx,
                sx as ssize_t,
                &raw const grid_default_cell,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                buf,
            );
        }
        if end == pdata.offset(psize as isize) {
            break;
        }
        end = end.offset(1);
        i = i.wrapping_add(1);
    }
    free(buf as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_buffer_find(
    mut data: *const ::core::ffi::c_void,
    mut datalen: size_t,
    mut find: *const ::core::ffi::c_void,
    mut findlen: size_t,
    mut icase: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut udata: *const u_char = data as *const u_char;
    let mut ufind: *const u_char = find as *const u_char;
    let mut i: size_t = 0;
    let mut j: size_t = 0;
    if findlen == 0 as size_t || datalen < findlen {
        return 0 as ::core::ffi::c_int;
    }
    i = 0 as size_t;
    while i.wrapping_add(findlen) <= datalen {
        j = 0 as size_t;
        while j < findlen {
            if icase == 0
                && *udata.offset(i.wrapping_add(j) as isize) as ::core::ffi::c_int
                    != *ufind.offset(j as isize) as ::core::ffi::c_int
            {
                break;
            }
            if icase != 0
                && ({
                    let mut __res: ::core::ffi::c_int = 0;
                    if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                        if 0 != 0 {
                            let mut __c: ::core::ffi::c_int =
                                *udata.offset(i.wrapping_add(j) as isize) as ::core::ffi::c_int;
                            __res = (if __c < -(128 as ::core::ffi::c_int)
                                || __c > 255 as ::core::ffi::c_int
                            {
                                __c as __int32_t
                            } else {
                                *(*__ctype_tolower_loc()).offset(__c as isize)
                            }) as ::core::ffi::c_int;
                        } else {
                            __res = tolower(
                                *udata.offset(i.wrapping_add(j) as isize) as ::core::ffi::c_int
                            );
                        }
                    } else {
                        __res =
                            *(*__ctype_tolower_loc())
                                .offset(*udata.offset(i.wrapping_add(j) as isize)
                                    as ::core::ffi::c_int
                                    as isize) as ::core::ffi::c_int;
                    }
                    __res
                }) != ({
                    let mut __res: ::core::ffi::c_int = 0;
                    if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                        if 0 != 0 {
                            let mut __c: ::core::ffi::c_int =
                                *ufind.offset(j as isize) as ::core::ffi::c_int;
                            __res = (if __c < -(128 as ::core::ffi::c_int)
                                || __c > 255 as ::core::ffi::c_int
                            {
                                __c as __int32_t
                            } else {
                                *(*__ctype_tolower_loc()).offset(__c as isize)
                            }) as ::core::ffi::c_int;
                        } else {
                            __res = tolower(*ufind.offset(j as isize) as ::core::ffi::c_int);
                        }
                    } else {
                        __res = *(*__ctype_tolower_loc())
                            .offset(*ufind.offset(j as isize) as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int;
                    }
                    __res
                })
            {
                break;
            }
            j = j.wrapping_add(1);
        }
        if j == findlen {
            return 1 as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_buffer_search(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut ss: *const ::core::ffi::c_char,
    mut icase: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut item: *mut window_buffer_itemdata = itemdata as *mut window_buffer_itemdata;
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut bufdata: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut bufsize: size_t = 0;
    pb = paste_get_name((*item).name);
    if pb.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if icase != 0 {
        if !strcasestr((*item).name, ss).is_null() {
            return 1 as ::core::ffi::c_int;
        }
        bufdata = paste_buffer_data(pb, &raw mut bufsize);
        return window_buffer_find(
            bufdata as *const ::core::ffi::c_void,
            bufsize,
            ss as *const ::core::ffi::c_void,
            strlen(ss),
            icase,
        );
    } else {
        if !strstr((*item).name, ss).is_null() {
            return 1 as ::core::ffi::c_int;
        }
        bufdata = paste_buffer_data(pb, &raw mut bufsize);
        return window_buffer_find(
            bufdata as *const ::core::ffi::c_void,
            bufsize,
            ss as *const ::core::ffi::c_void,
            strlen(ss),
            icase,
        );
    };
}
unsafe extern "C" fn window_buffer_menu(
    mut modedata: *mut ::core::ffi::c_void,
    mut c: *mut client,
    mut key: key_code,
) {
    let mut data: *mut window_buffer_modedata = modedata as *mut window_buffer_modedata;
    let mut wp: *mut window_pane = (*data).wp;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    wme = (*wp).modes.tqh_first;
    if wme.is_null() || (*wme).data != modedata {
        return;
    }
    window_buffer_key(
        wme,
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        key,
        ::core::ptr::null_mut::<mouse_event>(),
    );
}
unsafe extern "C" fn window_buffer_get_key(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut line: u_int,
) -> key_code {
    let mut data: *mut window_buffer_modedata = modedata as *mut window_buffer_modedata;
    let mut item: *mut window_buffer_itemdata = itemdata as *mut window_buffer_itemdata;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: key_code = 0;
    if cmd_find_valid_state(&raw mut (*data).fs) != 0 {
        s = (*data).fs.s;
        wl = (*data).fs.wl;
        wp = (*data).fs.wp;
    }
    pb = paste_get_name((*item).name);
    if pb.is_null() {
        return KEYC_NONE as ::core::ffi::c_ulong as key_code;
    }
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(
        ft,
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    format_defaults(ft, ::core::ptr::null_mut::<client>(), s, wl, wp);
    format_defaults_paste_buffer(ft, pb);
    format_add(
        ft,
        b"line\0" as *const u8 as *const ::core::ffi::c_char,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        line,
    );
    expanded = format_expand(ft, (*data).key_format);
    key = key_string_lookup_string(expanded);
    free(expanded as *mut ::core::ffi::c_void);
    format_free(ft);
    return key;
}
unsafe extern "C" fn window_buffer_sort(mut sort_crit: *mut sort_criteria) {
    (*sort_crit).order_seq = &raw mut window_buffer_order_seq as *mut sort_order;
    if (*sort_crit).order as ::core::ffi::c_uint
        == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*sort_crit).order = *(*sort_crit)
            .order_seq
            .offset(0 as ::core::ffi::c_int as isize);
    }
}
static mut window_buffer_help_lines: [*const ::core::ffi::c_char; 8] = [
    b"#[fg=themelightgrey]      Enter #[#{E:tree-mode-border-style},acs]x#[default] Paste selected %1\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          p #[#{E:tree-mode-border-style},acs]x#[default] Paste selected %1\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          P #[#{E:tree-mode-border-style},acs]x#[default] Paste tagged %1s\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          d #[#{E:tree-mode-border-style},acs]x#[default] Delete selected %1\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          D #[#{E:tree-mode-border-style},acs]x#[default] Delete tagged %1s\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          e #[#{E:tree-mode-border-style},acs]x#[default] Open %1 in editor\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          f #[#{E:tree-mode-border-style},acs]x#[default] Enter a filter\0"
        as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
unsafe extern "C" fn window_buffer_help(
    mut width: *mut u_int,
    mut item: *mut *const ::core::ffi::c_char,
) -> *mut *const ::core::ffi::c_char {
    *width = 0 as u_int;
    *item = b"buffer\0" as *const u8 as *const ::core::ffi::c_char;
    return &raw mut window_buffer_help_lines as *mut *const ::core::ffi::c_char;
}
unsafe extern "C" fn window_buffer_init(
    mut wme: *mut window_mode_entry,
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_buffer_modedata = ::core::ptr::null_mut::<window_buffer_modedata>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    data = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_buffer_modedata>() as size_t,
    ) as *mut window_buffer_modedata;
    (*wme).data = data as *mut ::core::ffi::c_void;
    (*data).wp = wp;
    cmd_find_copy_state(&raw mut (*data).fs, fs);
    if args.is_null() || args_has(args, 'F' as i32 as u_char) == 0 {
        (*data).format = xstrdup(WINDOW_BUFFER_DEFAULT_FORMAT.as_ptr());
    } else {
        (*data).format = xstrdup(args_get(args, 'F' as i32 as u_char));
    }
    if args.is_null() || args_has(args, 'K' as i32 as u_char) == 0 {
        (*data).key_format = xstrdup(WINDOW_BUFFER_DEFAULT_KEY_FORMAT.as_ptr());
    } else {
        (*data).key_format = xstrdup(args_get(args, 'K' as i32 as u_char));
    }
    if args.is_null() || args_count(args) == 0 as u_int {
        (*data).command = xstrdup(WINDOW_BUFFER_DEFAULT_COMMAND.as_ptr());
    } else {
        (*data).command = xstrdup(args_string(args, 0 as u_int));
    }
    (*data).data = mode_tree_start(
        wp,
        args,
        Some(
            window_buffer_build
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut sort_criteria,
                    *mut uint64_t,
                    *const ::core::ffi::c_char,
                ) -> (),
        ),
        Some(
            window_buffer_draw
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    *mut screen_write_ctx,
                    u_int,
                    u_int,
                ) -> (),
        ),
        Some(
            window_buffer_search
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    *const ::core::ffi::c_char,
                    ::core::ffi::c_int,
                ) -> ::core::ffi::c_int,
        ),
        Some(
            window_buffer_menu
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut client, key_code) -> (),
        ),
        None,
        Some(
            window_buffer_get_key
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    u_int,
                ) -> key_code,
        ),
        None,
        Some(window_buffer_sort as unsafe extern "C" fn(*mut sort_criteria) -> ()),
        Some(
            window_buffer_help
                as unsafe extern "C" fn(
                    *mut u_int,
                    *mut *const ::core::ffi::c_char,
                ) -> *mut *const ::core::ffi::c_char,
        ),
        data as *mut ::core::ffi::c_void,
        &raw const window_buffer_menu_items as *const menu_item,
        &raw mut s,
    );
    mode_tree_zoom((*data).data, args);
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    return s;
}
unsafe extern "C" fn window_buffer_free(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_buffer_modedata = (*wme).data as *mut window_buffer_modedata;
    let mut i: u_int = 0;
    if data.is_null() {
        return;
    }
    if !(*data).editor.is_null() {
        spawn_cancel_editor((*data).editor);
        window_buffer_finish_edit((*data).edit as *mut window_buffer_editdata);
    }
    mode_tree_free((*data).data);
    i = 0 as u_int;
    while i < (*data).item_size {
        window_buffer_free_item(*(*data).item_list.offset(i as isize));
        i = i.wrapping_add(1);
    }
    free((*data).item_list as *mut ::core::ffi::c_void);
    free((*data).format as *mut ::core::ffi::c_void);
    free((*data).key_format as *mut ::core::ffi::c_void);
    free((*data).command as *mut ::core::ffi::c_void);
    free(data as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_buffer_resize(
    mut wme: *mut window_mode_entry,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_buffer_modedata = (*wme).data as *mut window_buffer_modedata;
    mode_tree_resize((*data).data, sx, sy);
}
unsafe extern "C" fn window_buffer_update(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_buffer_modedata = (*wme).data as *mut window_buffer_modedata;
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    window_buffer_draw_waiting(data);
    (*(*data).wp).flags |= PANE_REDRAW;
}
unsafe extern "C" fn window_buffer_do_delete(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut c: *mut client,
    mut key: key_code,
) {
    let mut data: *mut window_buffer_modedata = modedata as *mut window_buffer_modedata;
    let mut item: *mut window_buffer_itemdata = itemdata as *mut window_buffer_itemdata;
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    if item == mode_tree_get_current((*data).data) as *mut window_buffer_itemdata
        && mode_tree_down((*data).data, 0 as ::core::ffi::c_int) == 0
    {
        mode_tree_up((*data).data, 0 as ::core::ffi::c_int);
    }
    pb = paste_get_name((*item).name);
    if !pb.is_null() {
        paste_free(pb);
    }
}
unsafe extern "C" fn window_buffer_do_paste(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut c: *mut client,
    mut key: key_code,
) {
    let mut data: *mut window_buffer_modedata = modedata as *mut window_buffer_modedata;
    let mut item: *mut window_buffer_itemdata = itemdata as *mut window_buffer_itemdata;
    if !paste_get_name((*item).name).is_null() {
        mode_tree_run_command(
            c,
            ::core::ptr::null_mut::<cmd_find_state>(),
            (*data).command,
            (*item).name,
        );
    }
}
unsafe extern "C" fn window_buffer_finish_edit(mut ed: *mut window_buffer_editdata) {
    free((*ed).name as *mut ::core::ffi::c_void);
    free(ed as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_buffer_draw_waiting(mut data: *mut window_buffer_modedata) {
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
    let mut s: *mut screen = (*(*data).wp).screen;
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
    let mut text: [::core::ffi::c_char; 128] = [0; 128];
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut box_w: u_int = 0;
    let mut box_h: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut text_x: u_int = 0;
    let mut textlen: size_t = 0;
    let mut pid: pid_t = 0;
    if (*data).editor.is_null() {
        return;
    }
    sx = (*(*s).grid).sx;
    sy = (*(*s).grid).sy;
    if sx == 0 as u_int || sy == 0 as u_int {
        return;
    }
    pid = spawn_get_editor_pid((*data).editor);
    if pid == -(1 as ::core::ffi::c_int) {
        xsnprintf(
            &raw mut text as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
            b"WAITING FOR EDITOR\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        xsnprintf(
            &raw mut text as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
            b"WAITING FOR EDITOR (PID %ld)\0" as *const u8 as *const ::core::ffi::c_char,
            pid as ::core::ffi::c_long,
        );
    }
    textlen = strlen(&raw mut text as *mut ::core::ffi::c_char);
    box_w = textlen.wrapping_add(4 as size_t) as u_int;
    box_h = 3 as u_int;
    if sx < box_w || sy < box_h {
        return;
    }
    x = sx.wrapping_sub(box_w).wrapping_div(2 as u_int);
    y = sy.wrapping_sub(box_h).wrapping_div(2 as u_int);
    text_x = (x as size_t).wrapping_add(
        (box_w as size_t)
            .wrapping_sub(textlen)
            .wrapping_div(2 as size_t),
    ) as u_int;
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    screen_write_start(&raw mut ctx, s);
    screen_write_cursormove(
        &raw mut ctx,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_box(
        &raw mut ctx,
        box_w,
        box_h,
        BOX_LINES_DEFAULT,
        &raw mut gc,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
    screen_write_cursormove(
        &raw mut ctx,
        x.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        y.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_clearcharacter(&raw mut ctx, box_w.wrapping_sub(2 as u_int), gc.bg as u_int);
    screen_write_cursormove(
        &raw mut ctx,
        text_x as ::core::ffi::c_int,
        y.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_nputs(
        &raw mut ctx,
        box_w.wrapping_sub(2 as u_int) as ssize_t,
        &raw mut gc,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut text as *mut ::core::ffi::c_char,
    );
    screen_write_stop(&raw mut ctx);
}
unsafe extern "C" fn window_buffer_edit_close_cb(
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut ed: *mut window_buffer_editdata = arg as *mut window_buffer_editdata;
    let mut oldlen: size_t = 0;
    let mut oldbuf: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut data: *mut window_buffer_modedata = ::core::ptr::null_mut::<window_buffer_modedata>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    wp = window_pane_find_by_id((*ed).wp_id);
    if !wp.is_null() {
        wme = (*wp).modes.tqh_first;
        if !wme.is_null() && (*wme).mode == &raw const window_buffer_mode {
            data = (*wme).data as *mut window_buffer_modedata;
            if (*data).editor == (*ed).editor {
                (*data).editor = ::core::ptr::null_mut::<spawn_editor_state>();
                (*data).edit = ::core::ptr::null_mut::<window_buffer_editdata>();
            }
        }
    }
    if buf.is_null() || len == 0 as size_t {
        window_buffer_finish_edit(ed);
        return;
    }
    pb = paste_get_name((*ed).name);
    if pb.is_null() || pb != (*ed).pb {
        window_buffer_finish_edit(ed);
        return;
    }
    oldbuf = paste_buffer_data(pb, &raw mut oldlen);
    if oldlen != 0 as size_t
        && *oldbuf.offset(oldlen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            != '\n' as i32
        && *buf.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int == '\n' as i32
    {
        len = len.wrapping_sub(1);
    }
    if len != 0 as size_t {
        paste_replace(pb, buf, len);
    }
    wp = window_pane_find_by_id((*ed).wp_id);
    if !wp.is_null() {
        wme = (*wp).modes.tqh_first;
        if !wme.is_null() && (*wme).mode == &raw const window_buffer_mode {
            data = (*wme).data as *mut window_buffer_modedata;
            mode_tree_build((*data).data);
            mode_tree_draw((*data).data);
            window_buffer_draw_waiting(data);
        }
        (*wp).flags |= PANE_REDRAW;
    }
    window_buffer_finish_edit(ed);
}
unsafe extern "C" fn window_buffer_start_edit(
    mut data: *mut window_buffer_modedata,
    mut item: *mut window_buffer_itemdata,
    mut c: *mut client,
) {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut buf: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut ed: *mut window_buffer_editdata = ::core::ptr::null_mut::<window_buffer_editdata>();
    if !(*data).editor.is_null() {
        return;
    }
    pb = paste_get_name((*item).name);
    if pb.is_null() {
        return;
    }
    buf = paste_buffer_data(pb, &raw mut len);
    ed = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_buffer_editdata>() as size_t,
    ) as *mut window_buffer_editdata;
    (*ed).wp_id = (*(*data).wp).id;
    (*ed).name = xstrdup(paste_buffer_name(pb));
    (*ed).pb = pb;
    (*ed).editor = spawn_editor(
        c,
        buf,
        len,
        Some(
            window_buffer_edit_close_cb
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_char,
                    size_t,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        ed as *mut ::core::ffi::c_void,
    );
    if (*ed).editor.is_null() {
        window_buffer_finish_edit(ed);
    } else {
        (*data).editor = (*ed).editor;
        (*data).edit = ed as *mut window_buffer_editdata;
    };
}
unsafe extern "C" fn window_buffer_key(
    mut wme: *mut window_mode_entry,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_buffer_modedata = (*wme).data as *mut window_buffer_modedata;
    let mut mtd: *mut mode_tree_data = (*data).data;
    let mut item: *mut window_buffer_itemdata = ::core::ptr::null_mut::<window_buffer_itemdata>();
    let mut finished: ::core::ffi::c_int = 0;
    if paste_is_empty() != 0 {
        finished = 1 as ::core::ffi::c_int;
    } else if !(*data).editor.is_null() {
        if key == 'q' as i32 as key_code
            || key == '\u{1b}' as i32 as key_code
            || key == '\u{3}' as i32 as key_code
        {
            finished = 1 as ::core::ffi::c_int;
        } else {
            finished = 0 as ::core::ffi::c_int;
        }
    } else {
        finished = mode_tree_key(
            mtd,
            c,
            &raw mut key,
            m,
            ::core::ptr::null_mut::<u_int>(),
            ::core::ptr::null_mut::<u_int>(),
        );
        match key {
            101 => {
                item = mode_tree_get_current(mtd) as *mut window_buffer_itemdata;
                window_buffer_start_edit(data, item, c);
            }
            100 => {
                item = mode_tree_get_current(mtd) as *mut window_buffer_itemdata;
                window_buffer_do_delete(
                    data as *mut ::core::ffi::c_void,
                    item as *mut ::core::ffi::c_void,
                    c,
                    key,
                );
                mode_tree_build(mtd);
            }
            68 => {
                mode_tree_each_tagged(
                    mtd,
                    Some(
                        window_buffer_do_delete
                            as unsafe extern "C" fn(
                                *mut ::core::ffi::c_void,
                                *mut ::core::ffi::c_void,
                                *mut client,
                                key_code,
                            ) -> (),
                    ),
                    c,
                    key,
                    0 as ::core::ffi::c_int,
                );
                mode_tree_build(mtd);
            }
            80 => {
                mode_tree_each_tagged(
                    mtd,
                    Some(
                        window_buffer_do_paste
                            as unsafe extern "C" fn(
                                *mut ::core::ffi::c_void,
                                *mut ::core::ffi::c_void,
                                *mut client,
                                key_code,
                            ) -> (),
                    ),
                    c,
                    key,
                    0 as ::core::ffi::c_int,
                );
                finished = 1 as ::core::ffi::c_int;
            }
            112 | 13 => {
                item = mode_tree_get_current(mtd) as *mut window_buffer_itemdata;
                window_buffer_do_paste(
                    data as *mut ::core::ffi::c_void,
                    item as *mut ::core::ffi::c_void,
                    c,
                    key,
                );
                finished = 1 as ::core::ffi::c_int;
            }
            _ => {}
        }
    }
    if finished != 0 || paste_is_empty() != 0 {
        window_pane_reset_mode(wp);
    } else {
        mode_tree_draw(mtd);
        window_buffer_draw_waiting(data);
        (*wp).flags |= PANE_REDRAW;
    };
}

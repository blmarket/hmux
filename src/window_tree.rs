pub use crate::src::shared::pane::{
    PANE_REDRAW, window_pane_offset, window_pane_resize, window_pane_resize_entry,
    window_pane_resizes,
};
pub use crate::src::shared::menu::{menu_item};
pub use crate::src::shared::prompt::{
    PROMPT_ACCEPT, PROMPT_CLOSE, PROMPT_CONTINUE, PROMPT_NOFORMAT, PROMPT_SINGLE,
    prompt_free_cb, prompt_result,
};
pub use crate::src::shared::abi::{__int32_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_citem, screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::tree::{RB_NEGINF};
pub use crate::src::shared::sort::{sort_criteria};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::layout::*;
use crate::src::shared::prompt::*;
use crate::src::shared::sort::*;
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
    fn sort_would_window_tree_swap(
        _: *mut sort_criteria,
        _: *mut winlink,
        _: *mut winlink,
    ) -> ::core::ffi::c_int;
    fn sort_get_sessions(_: *mut u_int, _: *mut sort_criteria) -> *mut *mut session;
    fn sort_get_panes_window(
        _: *mut window,
        _: *mut u_int,
        _: *mut sort_criteria,
    ) -> *mut *mut window_pane;
    fn sort_get_winlinks_session(
        _: *mut session,
        _: *mut u_int,
        _: *mut sort_criteria,
    ) -> *mut *mut winlink;
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
    fn format_single(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut ::core::ffi::c_char;
    fn format_defaults(
        _: *mut format_tree,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    );
    fn format_draw(
        _: *mut screen_write_ctx,
        _: *const grid_cell,
        _: u_int,
        _: *const ::core::ffi::c_char,
        _: *mut style_ranges,
        _: ::core::ffi::c_int,
    );
    fn format_width(_: *const ::core::ffi::c_char) -> u_int;
    fn format_trim_left(_: *const ::core::ffi::c_char, _: u_int) -> *mut ::core::ffi::c_char;
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn args_count(_: *mut args) -> u_int;
    fn args_string(_: *mut args, _: u_int) -> *const ::core::ffi::c_char;
    fn cmd_find_clear_state(_: *mut cmd_find_state, _: ::core::ffi::c_int);
    fn cmd_find_from_winlink_pane(
        _: *mut cmd_find_state,
        _: *mut winlink,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    );
    fn cmdq_get_callback1(
        _: *const ::core::ffi::c_char,
        _: cmdq_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut cmdq_item;
    fn cmdq_append(_: *mut client, _: *mut cmdq_item) -> *mut cmdq_item;
    fn key_string_lookup_string(_: *const ::core::ffi::c_char) -> key_code;
    fn server_set_marked(_: *mut session, _: *mut winlink, _: *mut window_pane);
    fn server_clear_marked();
    fn server_redraw_session_group(_: *mut session);
    fn server_kill_pane(_: *mut window_pane);
    fn server_kill_window(_: *mut window, _: ::core::ffi::c_int);
    fn server_renumber_all();
    fn server_destroy_session(_: *mut session);
    fn recalculate_sizes();
    static grid_default_cell: grid_cell;
    fn screen_write_puts(
        _: *mut screen_write_ctx,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn screen_write_putc(_: *mut screen_write_ctx, _: *const grid_cell, _: u_char);
    fn screen_write_hline(
        _: *mut screen_write_ctx,
        _: u_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: box_lines,
        _: *const grid_cell,
    );
    fn screen_write_vline(
        _: *mut screen_write_ctx,
        _: u_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
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
    fn screen_write_preview(_: *mut screen_write_ctx, _: *mut screen, _: u_int, _: u_int);
    fn screen_write_clearcharacter(_: *mut screen_write_ctx, _: u_int, _: u_int);
    fn screen_write_cursormove(
        _: *mut screen_write_ctx,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn winlinks_RB_NEXT(_: *mut winlink) -> *mut winlink;
    fn winlinks_RB_MINMAX(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn winlink_find_by_index(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn winlink_count(_: *mut winlinks) -> u_int;
    fn window_has_pane(_: *mut window, _: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_index(_: *mut window_pane, _: *mut u_int) -> ::core::ffi::c_int;
    fn window_count_panes(_: *mut window, _: ::core::ffi::c_int) -> u_int;
    fn window_pane_find_by_id(_: u_int) -> *mut window_pane;
    fn window_pane_reset_mode(_: *mut window_pane);
    fn mode_tree_count_tagged(_: *mut mode_tree_data) -> u_int;
    fn mode_tree_get_current(_: *mut mode_tree_data) -> *mut ::core::ffi::c_void;
    fn mode_tree_expand_current(_: *mut mode_tree_data);
    fn mode_tree_expand(_: *mut mode_tree_data, _: uint64_t);
    fn mode_tree_set_current(_: *mut mode_tree_data, _: uint64_t) -> ::core::ffi::c_int;
    fn mode_tree_each_tagged(
        _: *mut mode_tree_data,
        _: mode_tree_each_cb,
        _: *mut client,
        _: key_code,
        _: ::core::ffi::c_int,
    );
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
    fn mode_tree_view_name(_: *mut mode_tree_data, _: *const ::core::ffi::c_char);
    fn mode_tree_align(_: *mut mode_tree_item, _: ::core::ffi::c_int);
    fn mode_tree_remove(_: *mut mode_tree_data, _: *mut mode_tree_item);
    fn mode_tree_draw(_: *mut mode_tree_data);
    fn mode_tree_key(
        _: *mut mode_tree_data,
        _: *mut client,
        _: *mut key_code,
        _: *mut mouse_event,
        _: *mut u_int,
        _: *mut u_int,
    ) -> ::core::ffi::c_int;
    fn mode_tree_set_prompt(
        _: *mut mode_tree_data,
        _: *mut client,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: prompt_type,
        _: ::core::ffi::c_int,
        _: mode_tree_prompt_input_cb,
        _: prompt_free_cb,
        _: *mut ::core::ffi::c_void,
    );
    fn mode_tree_run_command(
        _: *mut client,
        _: *mut cmd_find_state,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
    );
    fn session_find_by_id(_: u_int) -> *mut session;
    fn session_destroy(_: *mut session, _: ::core::ffi::c_int, _: *const ::core::ffi::c_char);
    fn session_set_current(_: *mut session, _: *mut winlink) -> ::core::ffi::c_int;
    fn session_group_contains(_: *mut session) -> *mut session_group;
    fn session_group_synchronize_from(_: *mut session);
    fn osdep_get_name(
        _: ::core::ffi::c_int,
        _: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
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
pub struct session_group {
    pub name: *const ::core::ffi::c_char,
    pub sessions: C2RustUnnamed_40,
    pub entry: C2RustUnnamed_39,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_39 {
    pub rbe_left: *mut session_group,
    pub rbe_right: *mut session_group,
    pub rbe_parent: *mut session_group,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_40 {
    pub tqh_first: *mut session,
    pub tqh_last: *mut *mut session,
}
pub type cmdq_cb =
    Option<unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval>;
pub type mode_tree_prompt_input_cb = Option<
    unsafe extern "C" fn(
        *mut client,
        *mut ::core::ffi::c_void,
        *const ::core::ffi::c_char,
        prompt_key_result,
    ) -> prompt_result,
>;
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
pub struct window_tree_modedata {
    pub wp: *mut window_pane,
    pub dead: ::core::ffi::c_int,
    pub references: ::core::ffi::c_int,
    pub data: *mut mode_tree_data,
    pub format: *mut ::core::ffi::c_char,
    pub key_format: *mut ::core::ffi::c_char,
    pub command: *mut ::core::ffi::c_char,
    pub squash_groups: ::core::ffi::c_int,
    pub hide_preview_this_pane: ::core::ffi::c_int,
    pub preview_is_info: ::core::ffi::c_int,
    pub prompt_flags: ::core::ffi::c_int,
    pub item_list: *mut *mut window_tree_itemdata,
    pub item_size: u_int,
    pub entered: *const ::core::ffi::c_char,
    pub fs: cmd_find_state,
    pub type_0: window_tree_type,
    pub offset: ::core::ffi::c_int,
    pub left: ::core::ffi::c_int,
    pub right: ::core::ffi::c_int,
    pub start: u_int,
    pub end: u_int,
    pub each: u_int,
}
pub type window_tree_type = ::core::ffi::c_uint;
pub const WINDOW_TREE_PANE: window_tree_type = 3;
pub const WINDOW_TREE_WINDOW: window_tree_type = 2;
pub const WINDOW_TREE_SESSION: window_tree_type = 1;
pub const WINDOW_TREE_NONE: window_tree_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_tree_itemdata {
    pub type_0: window_tree_type,
    pub session: ::core::ffi::c_int,
    pub winlink: ::core::ffi::c_int,
    pub pane: ::core::ffi::c_int,
}
#[inline]
unsafe extern "C" fn tolower(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_tolower_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}
pub const FORMAT_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FORMAT_PANE: ::core::ffi::c_uint = 0x80000000 as ::core::ffi::c_uint;
pub const FORMAT_WINDOW: ::core::ffi::c_uint = 0x40000000 as ::core::ffi::c_uint;
pub const WINDOW_TREE_DEFAULT_COMMAND: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"switch-client -Zt '%%'\0")
};
pub const WINDOW_TREE_DEFAULT_FORMAT: [::core::ffi::c_char; 582] = unsafe {
    ::core::mem::transmute::<
        [u8; 582],
        [::core::ffi::c_char; 582],
    >(
        *b"#{?pane_format,#{?pane_marked,#[fg=thememagenta],}#{?pane_floating_flag,#[underscore],}#{pane_current_command}#[fg=themelightgrey]#{pane_flags}#{?#{&&:#{pane_title},#{!=:#{pane_title},#{host_short}}},: \"#{pane_title}\",},window_format,#{?window_marked_flag,#[fg=thememagenta],}#{window_name}#[fg=themelightgrey]#{window_flags}#{?#{&&:#{==:#{window_panes},1},#{&&:#{pane_title},#{!=:#{pane_title},#{host_short}}}},: \"#{pane_title}\",},#[fg=themelightgrey]#{session_windows} windows#{?session_grouped, (group #{session_group}: #{session_group_list}),}#{?session_attached, (attached),}}\0",
    )
};
pub const WINDOW_TREE_DEFAULT_KEY_FORMAT: [::core::ffi::c_char; 83] = unsafe {
    ::core::mem::transmute::<[u8; 83], [::core::ffi::c_char; 83]>(
        *b"#{?#{e|<:#{line},10},#{line},#{e|<:#{line},36},M-#{a:#{e|+:97,#{e|-:#{line},10}}}}\0",
    )
};
static mut window_tree_menu_items: [menu_item; 13] = [
    menu_item {
        name: b"Select\0" as *const u8 as *const ::core::ffi::c_char,
        key: '\r' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Expand\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Mark\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'm' as i32 as key_code,
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
        name: b"Kill\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'x' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Kill Tagged\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'X' as i32 as key_code,
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
pub static mut window_tree_mode: window_mode = unsafe {
    window_mode {
        name: b"tree-mode\0" as *const u8 as *const ::core::ffi::c_char,
        default_format: WINDOW_TREE_DEFAULT_FORMAT.as_ptr(),
        flags: 0,
        init: Some(
            window_tree_init
                as unsafe extern "C" fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_tree_free as unsafe extern "C" fn(*mut window_mode_entry) -> ()),
        resize: Some(
            window_tree_resize as unsafe extern "C" fn(*mut window_mode_entry, u_int, u_int) -> (),
        ),
        update: Some(window_tree_update as unsafe extern "C" fn(*mut window_mode_entry) -> ()),
        style_changed: None,
        key: Some(
            window_tree_key
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
static mut window_tree_order_seq: [sort_order; 5] =
    [SORT_INDEX, SORT_NAME, SORT_ACTIVITY, SORT_Z, SORT_END];
static mut window_tree_pane_info_lines: [*const ::core::ffi::c_char; 8] = [
    b"#[fg=themelightgrey]Pane          #[#{E:tree-mode-border-style},acs]x#[default] #{pane_index} #[fg=themelightgrey](#{pane_id})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Title         #[#{E:tree-mode-border-style},acs]x#[default] #{pane_title}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Command       #[#{E:tree-mode-border-style},acs]x#[default] #{pane_current_command} #[fg=themelightgrey](PID #{pane_pid})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Path          #[#{E:tree-mode-border-style},acs]x#[default] #{pane_current_path}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]TTY           #[#{E:tree-mode-border-style},acs]x#[default] #{pane_tty}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Position      #[#{E:tree-mode-border-style},acs]x#[default] #{pane_x},#{pane_y} #{pane_width}x#{pane_height}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Mode          #[#{E:tree-mode-border-style},acs]x#[default] #{?pane_in_mode,#{pane_mode},none}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Flags         #[#{E:tree-mode-border-style},acs]x#[default] #{?pane_active,#[fg=themegreen],#[fg=themelightgrey]}active#[default] #{?window_zoomed_flag,#[fg=themegreen],#[fg=themelightgrey]}zoomed#[default] #{?pane_marked,#[fg=themegreen],#[fg=themelightgrey]}marked#[default] #{?pane_synchronized,#[fg=themegreen],#[fg=themelightgrey]}sync#[default] #{?pane_dead,#[fg=themegreen],#[fg=themelightgrey]}dead#[default] #{?pane_pipe,#[fg=themegreen],#[fg=themelightgrey]}piped#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
];
static mut window_tree_window_info_lines: [*const ::core::ffi::c_char; 6] = [
    b"#[fg=themelightgrey]Window        #[#{E:tree-mode-border-style},acs]x#[default] #{window_index}: #{window_name} #[fg=themelightgrey](#{window_id})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Size          #[#{E:tree-mode-border-style},acs]x#[default] #{window_width}x#{window_height}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Panes         #[#{E:tree-mode-border-style},acs]x#[default] #{window_panes}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Activity Time #[#{E:tree-mode-border-style},acs]x#[default] #{t:window_activity} #[fg=themelightgrey](#{t/r:window_activity})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Sessions      #[#{E:tree-mode-border-style},acs]x#[default] #{s/,/ /:window_linked_sessions_list}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Flags         #[#{E:tree-mode-border-style},acs]x#[default] #{?window_active,#[fg=themegreen],#[fg=themelightgrey]}active#[default] #{?window_last_flag,#[fg=themegreen],#[fg=themelightgrey]}last#[default] #{?window_bell_flag,#[fg=themegreen],#[fg=themelightgrey]}bell#[default] #{?window_activity_flag,#[fg=themegreen],#[fg=themelightgrey]}activity#[default] #{?window_silence_flag,#[fg=themegreen],#[fg=themelightgrey]}silence#[default] #{?window_zoomed_flag,#[fg=themegreen],#[fg=themelightgrey]}zoomed#[default] #{?window_marked_flag,#[fg=themegreen],#[fg=themelightgrey]}marked#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
];
static mut window_tree_session_info_lines: [*const ::core::ffi::c_char; 9] = [
    b"#[fg=themelightgrey]Session       #[#{E:tree-mode-border-style},acs]x#[default] #{session_name} #[fg=themelightgrey](#{session_id})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Created Time  #[#{E:tree-mode-border-style},acs]x#[default] #{t:session_created} #[fg=themelightgrey](#{t/r:session_created})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Activity Time #[#{E:tree-mode-border-style},acs]x#[default] #{t:session_activity} #[fg=themelightgrey](#{t/r:session_activity})#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Attached Time #[#{E:tree-mode-border-style},acs]x#[default] #{?#{t:session_last_attached},#{t:session_last_attached} #[fg=themelightgrey](#{t/r:session_last_attached})#[default],never}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Clients       #[#{E:tree-mode-border-style},acs]x#[default] #{s/,/ /:session_attached_list}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Windows       #[#{E:tree-mode-border-style},acs]x#[default] #{session_windows}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Path          #[#{E:tree-mode-border-style},acs]x#[default] #{session_path}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Group         #[#{E:tree-mode-border-style},acs]x#[default] #{?session_grouped,#{session_group} (#{session_group_size}),none}\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]Flags         #[#{E:tree-mode-border-style},acs]x#[default] #{?session_attached,#[fg=themegreen],#[fg=themelightgrey]}attached#[default] #{?session_grouped,#[fg=themegreen],#[fg=themelightgrey]}grouped#[default] #{?session_marked,#[fg=themegreen],#[fg=themelightgrey]}marked#[default] #{?session_bell_flag,#[fg=themegreen],#[fg=themelightgrey]}bell#[default] #{?session_activity_flag,#[fg=themegreen],#[fg=themelightgrey]}activity#[default] #{?session_silence_flag,#[fg=themegreen],#[fg=themelightgrey]}silence#[default]\0"
        as *const u8 as *const ::core::ffi::c_char,
];
unsafe extern "C" fn window_tree_pull_item(
    mut item: *mut window_tree_itemdata,
    mut sp: *mut *mut session,
    mut wlp: *mut *mut winlink,
    mut wp: *mut *mut window_pane,
) {
    *wp = ::core::ptr::null_mut::<window_pane>();
    *wlp = ::core::ptr::null_mut::<winlink>();
    *sp = session_find_by_id((*item).session as u_int);
    if (*sp).is_null() {
        return;
    }
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        *wlp = (**sp).curw;
        *wp = (*(**wlp).window).active;
        return;
    }
    *wlp = winlink_find_by_index(&raw mut (**sp).windows, (*item).winlink);
    if (*wlp).is_null() {
        *sp = ::core::ptr::null_mut::<session>();
        return;
    }
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        *wp = (*(**wlp).window).active;
        return;
    }
    *wp = window_pane_find_by_id((*item).pane as u_int);
    if window_has_pane((**wlp).window, *wp) == 0 {
        *wp = ::core::ptr::null_mut::<window_pane>();
    }
    if (*wp).is_null() {
        *sp = ::core::ptr::null_mut::<session>();
        *wlp = ::core::ptr::null_mut::<winlink>();
        return;
    }
}
unsafe extern "C" fn window_tree_add_item(
    mut data: *mut window_tree_modedata,
) -> *mut window_tree_itemdata {
    let mut item: *mut window_tree_itemdata = ::core::ptr::null_mut::<window_tree_itemdata>();
    (*data).item_list = xreallocarray(
        (*data).item_list as *mut ::core::ffi::c_void,
        (*data).item_size.wrapping_add(1 as u_int) as size_t,
        ::core::mem::size_of::<*mut window_tree_itemdata>() as size_t,
    ) as *mut *mut window_tree_itemdata;
    let fresh3 = (*data).item_size;
    (*data).item_size = (*data).item_size.wrapping_add(1);
    let ref mut fresh4 = *(*data).item_list.offset(fresh3 as isize);
    *fresh4 = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_tree_itemdata>() as size_t,
    ) as *mut window_tree_itemdata;
    item = *fresh4;
    return item;
}
unsafe extern "C" fn window_tree_free_item(mut item: *mut window_tree_itemdata) {
    free(item as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_tree_build_pane(
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut modedata: *mut ::core::ffi::c_void,
    mut parent: *mut mode_tree_item,
) {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut item: *mut window_tree_itemdata = ::core::ptr::null_mut::<window_tree_itemdata>();
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut idx: u_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    window_pane_index(wp, &raw mut idx);
    item = window_tree_add_item(data);
    (*item).type_0 = WINDOW_TREE_PANE;
    (*item).session = (*s).id as ::core::ffi::c_int;
    (*item).winlink = (*wl).idx;
    (*item).pane = (*wp).id as ::core::ffi::c_int;
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        (FORMAT_PANE | (*wp).id) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    format_defaults(ft, ::core::ptr::null_mut::<client>(), s, wl, wp);
    text = format_expand(ft, (*data).format);
    xasprintf(
        &raw mut name,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        idx,
    );
    format_free(ft);
    mti = mode_tree_add(
        (*data).data,
        parent,
        item as *mut ::core::ffi::c_void,
        wp as uint64_t,
        name,
        text,
        -(1 as ::core::ffi::c_int),
    ) as *mut mode_tree_item;
    free(text as *mut ::core::ffi::c_void);
    free(name as *mut ::core::ffi::c_void);
    mode_tree_align(mti, 1 as ::core::ffi::c_int);
}
unsafe extern "C" fn window_tree_filter_pane(
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut filter: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut result: ::core::ffi::c_int = 0;
    if filter.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    cp = format_single(
        ::core::ptr::null_mut::<cmdq_item>(),
        filter,
        ::core::ptr::null_mut::<client>(),
        s,
        wl,
        wp,
    );
    result = format_true(cp);
    free(cp as *mut ::core::ffi::c_void);
    return result;
}
unsafe extern "C" fn window_tree_build_window(
    mut s: *mut session,
    mut wl: *mut winlink,
    mut modedata: *mut ::core::ffi::c_void,
    mut sort_crit: *mut sort_criteria,
    mut parent: *mut mode_tree_item,
    mut filter: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut item: *mut window_tree_itemdata = ::core::ptr::null_mut::<window_tree_itemdata>();
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut l: *mut *mut window_pane = ::core::ptr::null_mut::<*mut window_pane>();
    let mut n: u_int = 0;
    let mut i: u_int = 0;
    let mut found: u_int = 0;
    let mut expanded: ::core::ffi::c_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut tag: uint64_t = FORMAT_NONE as uint64_t;
    item = window_tree_add_item(data);
    (*item).type_0 = WINDOW_TREE_WINDOW;
    (*item).session = (*s).id as ::core::ffi::c_int;
    (*item).winlink = (*wl).idx;
    (*item).pane = -(1 as ::core::ffi::c_int);
    if !(*wl).window.is_null() && !(*(*wl).window).active.is_null() {
        tag = (FORMAT_PANE | (*(*(*wl).window).active).id) as uint64_t;
    }
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        tag as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    format_defaults(
        ft,
        ::core::ptr::null_mut::<client>(),
        s,
        wl,
        ::core::ptr::null_mut::<window_pane>(),
    );
    text = format_expand(ft, (*data).format);
    xasprintf(
        &raw mut name,
        b"%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*wl).idx,
    );
    format_free(ft);
    if (*data).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*data).type_0 as ::core::ffi::c_uint
            == WINDOW_TREE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        expanded = 0 as ::core::ffi::c_int;
    } else {
        expanded = 1 as ::core::ffi::c_int;
    }
    mti = mode_tree_add(
        (*data).data,
        parent,
        item as *mut ::core::ffi::c_void,
        wl as uint64_t,
        name,
        text,
        expanded,
    ) as *mut mode_tree_item;
    free(text as *mut ::core::ffi::c_void);
    free(name as *mut ::core::ffi::c_void);
    mode_tree_align(mti, 1 as ::core::ffi::c_int);
    l = sort_get_panes_window((*wl).window, &raw mut n, sort_crit);
    found = 0 as u_int;
    i = 0 as u_int;
    while i < n {
        if !(window_tree_filter_pane(s, wl, *l.offset(i as isize), filter) == 0) {
            found = found.wrapping_add(1);
            if !((*data).hide_preview_this_pane != 0 && *l.offset(i as isize) == (*data).wp) {
                window_tree_build_pane(s, wl, *l.offset(i as isize), modedata, mti);
            }
        }
        i = i.wrapping_add(1);
    }
    if found == 0 as u_int {
        window_tree_free_item(item);
        (*data).item_size = (*data).item_size.wrapping_sub(1);
        mode_tree_remove((*data).data, mti);
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_tree_build_session(
    mut s: *mut session,
    mut modedata: *mut ::core::ffi::c_void,
    mut sort_crit: *mut sort_criteria,
    mut filter: *const ::core::ffi::c_char,
) {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut item: *mut window_tree_itemdata = ::core::ptr::null_mut::<window_tree_itemdata>();
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut wl: *mut winlink = (*s).curw;
    let mut l: *mut *mut winlink = ::core::ptr::null_mut::<*mut winlink>();
    let mut n: u_int = 0;
    let mut i: u_int = 0;
    let mut empty: u_int = 0;
    let mut expanded: ::core::ffi::c_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut tag: uint64_t = FORMAT_NONE as uint64_t;
    item = window_tree_add_item(data);
    (*item).type_0 = WINDOW_TREE_SESSION;
    (*item).session = (*s).id as ::core::ffi::c_int;
    (*item).winlink = -(1 as ::core::ffi::c_int);
    (*item).pane = -(1 as ::core::ffi::c_int);
    if !wl.is_null() && !(*wl).window.is_null() && !(*(*wl).window).active.is_null() {
        tag = (FORMAT_PANE | (*(*(*wl).window).active).id) as uint64_t;
    }
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        tag as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    format_defaults(
        ft,
        ::core::ptr::null_mut::<client>(),
        s,
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    text = format_expand(ft, (*data).format);
    format_free(ft);
    if (*data).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        expanded = 0 as ::core::ffi::c_int;
    } else {
        expanded = 1 as ::core::ffi::c_int;
    }
    mti = mode_tree_add(
        (*data).data,
        ::core::ptr::null_mut::<mode_tree_item>(),
        item as *mut ::core::ffi::c_void,
        s as uint64_t,
        (*s).name,
        text,
        expanded,
    ) as *mut mode_tree_item;
    free(text as *mut ::core::ffi::c_void);
    l = sort_get_winlinks_session(s, &raw mut n, sort_crit);
    empty = 0 as u_int;
    i = 0 as u_int;
    while i < n {
        if window_tree_build_window(s, *l.offset(i as isize), modedata, sort_crit, mti, filter) == 0
        {
            empty = empty.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    if empty == n {
        window_tree_free_item(item);
        (*data).item_size = (*data).item_size.wrapping_sub(1);
        mode_tree_remove((*data).data, mti);
    }
}
unsafe extern "C" fn window_tree_build(
    mut modedata: *mut ::core::ffi::c_void,
    mut sort_crit: *mut sort_criteria,
    mut tag: *mut uint64_t,
    mut filter: *const ::core::ffi::c_char,
) {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut squash_groups: ::core::ffi::c_int = (*data).squash_groups;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut l: *mut *mut session = ::core::ptr::null_mut::<*mut session>();
    let mut sg: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut current: *mut session_group = ::core::ptr::null_mut::<session_group>();
    let mut n: u_int = 0;
    let mut i: u_int = 0;
    current = session_group_contains((*data).fs.s);
    i = 0 as u_int;
    while i < (*data).item_size {
        window_tree_free_item(*(*data).item_list.offset(i as isize));
        i = i.wrapping_add(1);
    }
    free((*data).item_list as *mut ::core::ffi::c_void);
    (*data).item_list = ::core::ptr::null_mut::<*mut window_tree_itemdata>();
    (*data).item_size = 0 as u_int;
    l = sort_get_sessions(&raw mut n, sort_crit);
    if n == 0 as u_int {
        return;
    }
    let mut current_block_12: u64;
    i = 0 as u_int;
    while i < n {
        s = *l.offset(i as isize);
        if squash_groups != 0 && {
            sg = session_group_contains(s);
            !sg.is_null()
        } {
            if sg == current && s != (*data).fs.s || sg != current && s != (*sg).sessions.tqh_first
            {
                current_block_12 = 3640593987805443782;
            } else {
                current_block_12 = 4166486009154926805;
            }
        } else {
            current_block_12 = 4166486009154926805;
        }
        match current_block_12 {
            4166486009154926805 => {
                window_tree_build_session(s, modedata, sort_crit, filter);
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
    match (*data).type_0 as ::core::ffi::c_uint {
        1 => {
            *tag = (*data).fs.s as uint64_t;
        }
        2 => {
            *tag = (*data).fs.wl as uint64_t;
        }
        3 => {
            if window_count_panes((*(*data).fs.wl).window, 1 as ::core::ffi::c_int) == 1 as u_int {
                *tag = (*data).fs.wl as uint64_t;
            } else {
                *tag = (*data).fs.wp as uint64_t;
            }
        }
        0 | _ => {}
    };
}
unsafe extern "C" fn window_tree_draw_label(
    mut ctx: *mut screen_write_ctx,
    mut px: u_int,
    mut py: u_int,
    mut sx: u_int,
    mut sy: u_int,
    mut border_gc: *const grid_cell,
    mut label_gc: *const grid_cell,
    mut label: *const ::core::ffi::c_char,
) {
    let mut width: u_int = 0;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut new_label: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if sx < 5 as u_int || sy < 3 as u_int {
        return;
    }
    width = format_width(label);
    if width > sx.wrapping_sub(4 as u_int) {
        new_label = format_trim_left(label, sx.wrapping_sub(4 as u_int));
        label = new_label;
        width = format_width(new_label);
    }
    if width == 0 as u_int {
        return;
    }
    ox = sx
        .wrapping_sub(width)
        .wrapping_add(1 as u_int)
        .wrapping_div(2 as u_int);
    oy = sy.wrapping_add(1 as u_int).wrapping_div(2 as u_int);
    screen_write_cursormove(
        ctx,
        px.wrapping_add(ox).wrapping_sub(2 as u_int) as ::core::ffi::c_int,
        py.wrapping_add(oy).wrapping_sub(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_box(
        ctx,
        width.wrapping_add(4 as u_int),
        3 as u_int,
        BOX_LINES_DEFAULT,
        border_gc,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
    screen_write_cursormove(
        ctx,
        px.wrapping_add(ox).wrapping_sub(1 as u_int) as ::core::ffi::c_int,
        py.wrapping_add(oy) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_clearcharacter(
        ctx,
        width.wrapping_add(2 as u_int),
        (*border_gc).bg as u_int,
    );
    screen_write_cursormove(
        ctx,
        px.wrapping_add(ox) as ::core::ffi::c_int,
        py.wrapping_add(oy) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    format_draw(
        ctx,
        label_gc,
        width,
        label,
        ::core::ptr::null_mut::<style_ranges>(),
        0 as ::core::ffi::c_int,
    );
    free(new_label as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_tree_border_cell(
    mut gc: *mut grid_cell,
    mut oo: *mut options,
    mut ft: *mut format_tree,
) {
    memcpy(
        gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    style_apply(
        gc,
        oo,
        b"tree-mode-border-style\0" as *const u8 as *const ::core::ffi::c_char,
        ft,
    );
}
unsafe extern "C" fn window_tree_draw_session(
    mut data: *mut window_tree_modedata,
    mut s: *mut session,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut cx: u_int = (*(*ctx).s).cx;
    let mut cy: u_int = (*(*ctx).s).cy;
    let mut loop_0: u_int = 0;
    let mut total: u_int = 0;
    let mut visible: u_int = 0;
    let mut each: u_int = 0;
    let mut width: u_int = 0;
    let mut offset: u_int = 0;
    let mut current: u_int = 0;
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    let mut remaining: u_int = 0;
    let mut i: u_int = 0;
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
    let mut label_gc: grid_cell = grid_cell {
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
    let mut left: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut label: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut format: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    total = winlink_count(&raw mut (*s).windows);
    if sx.wrapping_div(total) < 24 as u_int {
        visible = sx.wrapping_div(24 as u_int);
        if visible == 0 as u_int {
            visible = 1 as u_int;
        }
    } else {
        visible = total;
    }
    current = 0 as u_int;
    wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        if wl == (*s).curw {
            break;
        }
        current = current.wrapping_add(1);
        wl = winlinks_RB_NEXT(wl);
    }
    if current < visible {
        start = 0 as u_int;
        end = visible;
    } else if current >= total.wrapping_sub(visible) {
        start = total.wrapping_sub(visible);
        end = total;
    } else {
        start = current.wrapping_sub(visible.wrapping_div(2 as u_int));
        end = start.wrapping_add(visible);
    }
    if (*data).offset < -(start as ::core::ffi::c_int) {
        (*data).offset = -(start as ::core::ffi::c_int);
    }
    if (*data).offset > total.wrapping_sub(end) as ::core::ffi::c_int {
        (*data).offset = total.wrapping_sub(end) as ::core::ffi::c_int;
    }
    start = start.wrapping_add((*data).offset as u_int);
    end = end.wrapping_add((*data).offset as u_int);
    left = (start != 0 as u_int) as ::core::ffi::c_int;
    right = (end != total) as ::core::ffi::c_int;
    if left != 0 && right != 0 && sx <= 6 as u_int || (left != 0 || right != 0) && sx <= 3 as u_int
    {
        right = 0 as ::core::ffi::c_int;
        left = right;
    }
    if left != 0 && right != 0 {
        each = sx.wrapping_sub(6 as u_int).wrapping_div(visible);
        remaining = sx
            .wrapping_sub(6 as u_int)
            .wrapping_sub(visible.wrapping_mul(each));
    } else if left != 0 || right != 0 {
        each = sx.wrapping_sub(3 as u_int).wrapping_div(visible);
        remaining = sx
            .wrapping_sub(3 as u_int)
            .wrapping_sub(visible.wrapping_mul(each));
    } else {
        each = sx.wrapping_div(visible);
        remaining = sx.wrapping_sub(visible.wrapping_mul(each));
    }
    if each == 0 as u_int {
        return;
    }
    window_tree_border_cell(
        &raw mut gc,
        (*(*(*data).wp).window).options,
        ::core::ptr::null_mut::<format_tree>(),
    );
    if left != 0 {
        (*data).left = cx.wrapping_add(2 as u_int) as ::core::ffi::c_int;
        screen_write_cursormove(
            ctx,
            cx.wrapping_add(2 as u_int) as ::core::ffi::c_int,
            cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_vline(
            ctx,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            &raw mut gc,
        );
        screen_write_cursormove(
            ctx,
            cx as ::core::ffi::c_int,
            cy.wrapping_add(sy.wrapping_div(2 as u_int)) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_puts(
            ctx,
            &raw mut gc,
            b"<\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        (*data).left = -(1 as ::core::ffi::c_int);
    }
    if right != 0 {
        (*data).right = cx.wrapping_add(sx).wrapping_sub(3 as u_int) as ::core::ffi::c_int;
        screen_write_cursormove(
            ctx,
            cx.wrapping_add(sx).wrapping_sub(3 as u_int) as ::core::ffi::c_int,
            cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_vline(
            ctx,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            &raw mut gc,
        );
        screen_write_cursormove(
            ctx,
            cx.wrapping_add(sx).wrapping_sub(1 as u_int) as ::core::ffi::c_int,
            cy.wrapping_add(sy.wrapping_div(2 as u_int)) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_puts(
            ctx,
            &raw mut gc,
            b">\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        (*data).right = -(1 as ::core::ffi::c_int);
    }
    (*data).start = start;
    (*data).end = end;
    (*data).each = each;
    loop_0 = 0 as u_int;
    i = loop_0;
    wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        if loop_0 == end {
            break;
        }
        if loop_0 < start {
            loop_0 = loop_0.wrapping_add(1);
        } else {
            w = (*wl).window;
            oo = (*w).options;
            ft = format_create(
                ::core::ptr::null_mut::<client>(),
                ::core::ptr::null_mut::<cmdq_item>(),
                (FORMAT_WINDOW | (*w).id) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            format_defaults(
                ft,
                ::core::ptr::null_mut::<client>(),
                s,
                wl,
                ::core::ptr::null_mut::<window_pane>(),
            );
            window_tree_border_cell(&raw mut gc, oo, ft);
            memcpy(
                &raw mut label_gc as *mut ::core::ffi::c_void,
                &raw const grid_default_cell as *const ::core::ffi::c_void,
                ::core::mem::size_of::<grid_cell>() as size_t,
            );
            style_apply(
                &raw mut label_gc,
                oo,
                b"tree-mode-preview-style\0" as *const u8 as *const ::core::ffi::c_char,
                ft,
            );
            label_gc.bg = gc.bg;
            if left != 0 {
                offset = (3 as u_int).wrapping_add(i.wrapping_mul(each));
            } else {
                offset = i.wrapping_mul(each);
            }
            if loop_0 == end.wrapping_sub(1 as u_int) {
                width = each.wrapping_add(remaining);
            } else {
                width = each.wrapping_sub(1 as u_int);
            }
            screen_write_cursormove(
                ctx,
                cx.wrapping_add(offset) as ::core::ffi::c_int,
                cy as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_preview(ctx, &raw mut (*(*w).active).base, width, sy);
            format = options_get_string(
                oo,
                b"tree-mode-preview-format\0" as *const u8 as *const ::core::ffi::c_char,
            );
            if *format as ::core::ffi::c_int != '\0' as i32 {
                label = format_expand(ft, format);
                if *label as ::core::ffi::c_int != '\0' as i32 {
                    window_tree_draw_label(
                        ctx,
                        cx.wrapping_add(offset),
                        cy,
                        width,
                        sy,
                        &raw mut gc,
                        &raw mut label_gc,
                        label,
                    );
                }
                free(label as *mut ::core::ffi::c_void);
            }
            format_free(ft);
            if loop_0 != end.wrapping_sub(1 as u_int) {
                screen_write_cursormove(
                    ctx,
                    cx.wrapping_add(offset).wrapping_add(width) as ::core::ffi::c_int,
                    cy as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                screen_write_vline(
                    ctx,
                    sy,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    &raw mut gc,
                );
            }
            loop_0 = loop_0.wrapping_add(1);
            i = i.wrapping_add(1);
        }
        wl = winlinks_RB_NEXT(wl);
    }
}
unsafe extern "C" fn window_tree_draw_window(
    mut data: *mut window_tree_modedata,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut w: *mut window = (*wl).window;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut cx: u_int = (*(*ctx).s).cx;
    let mut cy: u_int = (*(*ctx).s).cy;
    let mut loop_0: u_int = 0;
    let mut total: u_int = 0;
    let mut visible: u_int = 0;
    let mut each: u_int = 0;
    let mut width: u_int = 0;
    let mut offset: u_int = 0;
    let mut current: u_int = 0;
    let mut start: u_int = 0;
    let mut end: u_int = 0;
    let mut remaining: u_int = 0;
    let mut i: u_int = 0;
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
    let mut label_gc: grid_cell = grid_cell {
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
    let mut left: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut label: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut format: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    total = window_count_panes(w, 1 as ::core::ffi::c_int);
    if (*data).hide_preview_this_pane != 0 && (*(*data).wp).window == w {
        total = total.wrapping_sub(1);
    }
    if total == 0 as u_int {
        return;
    }
    if sx.wrapping_div(total) < 24 as u_int {
        visible = sx.wrapping_div(24 as u_int);
        if visible == 0 as u_int {
            visible = 1 as u_int;
        }
    } else {
        visible = total;
    }
    current = 0 as u_int;
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if !((*data).hide_preview_this_pane != 0 && wp == (*data).wp) {
            if wp == (*w).active {
                break;
            }
            current = current.wrapping_add(1);
        }
        wp = (*wp).entry.tqe_next;
    }
    if current < visible {
        start = 0 as u_int;
        end = visible;
    } else if current >= total.wrapping_sub(visible) {
        start = total.wrapping_sub(visible);
        end = total;
    } else {
        start = current.wrapping_sub(visible.wrapping_div(2 as u_int));
        end = start.wrapping_add(visible);
    }
    if (*data).offset < -(start as ::core::ffi::c_int) {
        (*data).offset = -(start as ::core::ffi::c_int);
    }
    if (*data).offset > total.wrapping_sub(end) as ::core::ffi::c_int {
        (*data).offset = total.wrapping_sub(end) as ::core::ffi::c_int;
    }
    start = start.wrapping_add((*data).offset as u_int);
    end = end.wrapping_add((*data).offset as u_int);
    left = (start != 0 as u_int) as ::core::ffi::c_int;
    right = (end != total) as ::core::ffi::c_int;
    if left != 0 && right != 0 && sx <= 6 as u_int || (left != 0 || right != 0) && sx <= 3 as u_int
    {
        right = 0 as ::core::ffi::c_int;
        left = right;
    }
    if left != 0 && right != 0 {
        each = sx.wrapping_sub(6 as u_int).wrapping_div(visible);
        remaining = sx
            .wrapping_sub(6 as u_int)
            .wrapping_sub(visible.wrapping_mul(each));
    } else if left != 0 || right != 0 {
        each = sx.wrapping_sub(3 as u_int).wrapping_div(visible);
        remaining = sx
            .wrapping_sub(3 as u_int)
            .wrapping_sub(visible.wrapping_mul(each));
    } else {
        each = sx.wrapping_div(visible);
        remaining = sx.wrapping_sub(visible.wrapping_mul(each));
    }
    if each == 0 as u_int {
        return;
    }
    window_tree_border_cell(
        &raw mut gc,
        (*(*(*data).wp).window).options,
        ::core::ptr::null_mut::<format_tree>(),
    );
    if left != 0 {
        (*data).left = cx.wrapping_add(2 as u_int) as ::core::ffi::c_int;
        screen_write_cursormove(
            ctx,
            cx.wrapping_add(2 as u_int) as ::core::ffi::c_int,
            cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_vline(
            ctx,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            &raw mut gc,
        );
        screen_write_cursormove(
            ctx,
            cx as ::core::ffi::c_int,
            cy.wrapping_add(sy.wrapping_div(2 as u_int)) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_puts(
            ctx,
            &raw mut gc,
            b"<\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        (*data).left = -(1 as ::core::ffi::c_int);
    }
    if right != 0 {
        (*data).right = cx.wrapping_add(sx).wrapping_sub(3 as u_int) as ::core::ffi::c_int;
        screen_write_cursormove(
            ctx,
            cx.wrapping_add(sx).wrapping_sub(3 as u_int) as ::core::ffi::c_int,
            cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_vline(
            ctx,
            sy,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            &raw mut gc,
        );
        screen_write_cursormove(
            ctx,
            cx.wrapping_add(sx).wrapping_sub(1 as u_int) as ::core::ffi::c_int,
            cy.wrapping_add(sy.wrapping_div(2 as u_int)) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_puts(
            ctx,
            &raw mut gc,
            b">\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        (*data).right = -(1 as ::core::ffi::c_int);
    }
    (*data).start = start;
    (*data).end = end;
    (*data).each = each;
    loop_0 = 0 as u_int;
    i = loop_0;
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if !((*data).hide_preview_this_pane != 0 && wp == (*data).wp) {
            if loop_0 == end {
                break;
            }
            if loop_0 < start {
                loop_0 = loop_0.wrapping_add(1);
            } else {
                oo = (*wp).options;
                ft = format_create(
                    ::core::ptr::null_mut::<client>(),
                    ::core::ptr::null_mut::<cmdq_item>(),
                    (FORMAT_PANE | (*wp).id) as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                format_defaults(ft, ::core::ptr::null_mut::<client>(), s, wl, wp);
                window_tree_border_cell(&raw mut gc, oo, ft);
                memcpy(
                    &raw mut label_gc as *mut ::core::ffi::c_void,
                    &raw const grid_default_cell as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<grid_cell>() as size_t,
                );
                style_apply(
                    &raw mut label_gc,
                    oo,
                    b"tree-mode-preview-style\0" as *const u8 as *const ::core::ffi::c_char,
                    ft,
                );
                label_gc.bg = gc.bg;
                if left != 0 {
                    offset = (3 as u_int).wrapping_add(i.wrapping_mul(each));
                } else {
                    offset = i.wrapping_mul(each);
                }
                if loop_0 == end.wrapping_sub(1 as u_int) {
                    width = each.wrapping_add(remaining);
                } else {
                    width = each.wrapping_sub(1 as u_int);
                }
                screen_write_cursormove(
                    ctx,
                    cx.wrapping_add(offset) as ::core::ffi::c_int,
                    cy as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                screen_write_preview(ctx, &raw mut (*wp).base, width, sy);
                format = options_get_string(
                    oo,
                    b"tree-mode-preview-format\0" as *const u8 as *const ::core::ffi::c_char,
                );
                if *format as ::core::ffi::c_int != '\0' as i32 {
                    label = format_expand(ft, format);
                    if *label as ::core::ffi::c_int != '\0' as i32 {
                        window_tree_draw_label(
                            ctx,
                            cx.wrapping_add(offset),
                            cy,
                            width,
                            sy,
                            &raw mut gc,
                            &raw mut label_gc,
                            label,
                        );
                    }
                    free(label as *mut ::core::ffi::c_void);
                }
                format_free(ft);
                if loop_0 != end.wrapping_sub(1 as u_int) {
                    screen_write_cursormove(
                        ctx,
                        cx.wrapping_add(offset).wrapping_add(width) as ::core::ffi::c_int,
                        cy as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    screen_write_vline(
                        ctx,
                        sy,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        &raw mut gc,
                    );
                }
                loop_0 = loop_0.wrapping_add(1);
                i = i.wrapping_add(1);
            }
        }
        wp = (*wp).entry.tqe_next;
    }
}
unsafe extern "C" fn window_tree_draw_info(
    mut data: *mut window_tree_modedata,
    mut itemdata: *mut ::core::ffi::c_void,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut item: *mut window_tree_itemdata = itemdata as *mut window_tree_itemdata;
    let mut s: *mut screen = (*ctx).s;
    let mut sp: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
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
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut k: u_int = 0;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut lines: [*const *const ::core::ffi::c_char; 3] =
        [::core::ptr::null::<*const ::core::ffi::c_char>(); 3];
    let mut count: [u_int; 3] = [0; 3];
    let mut n: u_int = 0 as u_int;
    window_tree_pull_item(item, &raw mut sp, &raw mut wl, &raw mut wp);
    if sp.is_null() || wp.is_null() {
        return;
    }
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        lines[n as usize] = &raw mut window_tree_pane_info_lines as *mut *const ::core::ffi::c_char;
        let fresh0 = n;
        n = n.wrapping_add(1);
        count[fresh0 as usize] = (::core::mem::size_of::<[*const ::core::ffi::c_char; 8]>()
            as usize)
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>() as usize)
            as u_int;
    }
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*item).type_0 as ::core::ffi::c_uint
            == WINDOW_TREE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        lines[n as usize] =
            &raw mut window_tree_window_info_lines as *mut *const ::core::ffi::c_char;
        let fresh1 = n;
        n = n.wrapping_add(1);
        count[fresh1 as usize] = (::core::mem::size_of::<[*const ::core::ffi::c_char; 6]>()
            as usize)
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>() as usize)
            as u_int;
    }
    lines[n as usize] = &raw mut window_tree_session_info_lines as *mut *const ::core::ffi::c_char;
    let fresh2 = n;
    n = n.wrapping_add(1);
    count[fresh2 as usize] = (::core::mem::size_of::<[*const ::core::ffi::c_char; 9]>() as usize)
        .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>() as usize)
        as u_int;
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(ft, ::core::ptr::null_mut::<client>(), sp, wl, wp);
    i = 0 as u_int;
    j = 0 as u_int;
    while j < n {
        if j != 0 as u_int {
            if i == sy {
                break;
            }
            window_tree_border_cell(
                &raw mut gc,
                (*(*(*data).wp).window).options,
                ::core::ptr::null_mut::<format_tree>(),
            );
            screen_write_cursormove(
                ctx,
                cx as ::core::ffi::c_int,
                cy.wrapping_add(i) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_hline(
                ctx,
                sx,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                BOX_LINES_DEFAULT,
                &raw mut gc,
            );
            if sx > 14 as u_int {
                gc.attr = (gc.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
                screen_write_cursormove(
                    ctx,
                    cx.wrapping_add(14 as u_int) as ::core::ffi::c_int,
                    cy.wrapping_add(i) as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                screen_write_putc(ctx, &raw mut gc, 'n' as i32 as u_char);
            }
            i = i.wrapping_add(1);
        }
        k = 0 as u_int;
        while k < count[j as usize] {
            if i == sy {
                break;
            }
            expanded = format_expand(ft, *lines[j as usize].offset(k as isize));
            screen_write_cursormove(
                ctx,
                cx as ::core::ffi::c_int,
                cy.wrapping_add(i) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            format_draw(
                ctx,
                &raw const grid_default_cell,
                sx,
                expanded,
                ::core::ptr::null_mut::<style_ranges>(),
                0 as ::core::ffi::c_int,
            );
            free(expanded as *mut ::core::ffi::c_void);
            i = i.wrapping_add(1);
            k = k.wrapping_add(1);
        }
        if i == sy {
            break;
        }
        j = j.wrapping_add(1);
    }
    if sx > 14 as u_int && i < sy {
        window_tree_border_cell(
            &raw mut gc,
            (*(*(*data).wp).window).options,
            ::core::ptr::null_mut::<format_tree>(),
        );
        screen_write_cursormove(
            ctx,
            cx.wrapping_add(14 as u_int) as ::core::ffi::c_int,
            cy.wrapping_add(i) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        screen_write_vline(
            ctx,
            sy.wrapping_sub(i),
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            &raw mut gc,
        );
    }
    format_free(ft);
}
unsafe extern "C" fn window_tree_draw(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut item: *mut window_tree_itemdata = itemdata as *mut window_tree_itemdata;
    let mut sp: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    window_tree_pull_item(item, &raw mut sp, &raw mut wl, &raw mut wp);
    if wp.is_null() {
        return;
    }
    if (*data).preview_is_info != 0 {
        window_tree_draw_info(data, item as *mut ::core::ffi::c_void, ctx, sx, sy);
        return;
    }
    match (*item).type_0 as ::core::ffi::c_uint {
        1 => {
            window_tree_draw_session(modedata as *mut window_tree_modedata, sp, ctx, sx, sy);
        }
        2 => {
            window_tree_draw_window(modedata as *mut window_tree_modedata, sp, wl, ctx, sx, sy);
        }
        3 => {
            if (*data).hide_preview_this_pane == 0 || wp != (*data).wp {
                screen_write_preview(ctx, &raw mut (*wp).base, sx, sy);
            }
        }
        0 | _ => {}
    };
}
unsafe extern "C" fn window_tree_search(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut ss: *const ::core::ffi::c_char,
    mut icase: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut item: *mut window_tree_itemdata = itemdata as *mut window_tree_itemdata;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut retval: ::core::ffi::c_int = 0;
    window_tree_pull_item(item, &raw mut s, &raw mut wl, &raw mut wp);
    match (*item).type_0 as ::core::ffi::c_uint {
        0 => return 0 as ::core::ffi::c_int,
        1 => {
            if s.is_null() {
                return 0 as ::core::ffi::c_int;
            }
            if icase != 0 {
                return (strcasestr((*s).name, ss) != NULL as *mut ::core::ffi::c_char)
                    as ::core::ffi::c_int;
            }
            return (strstr((*s).name, ss) != NULL as *mut ::core::ffi::c_char)
                as ::core::ffi::c_int;
        }
        2 => {
            if s.is_null() || wl.is_null() {
                return 0 as ::core::ffi::c_int;
            }
            if icase != 0 {
                return (strcasestr((*(*wl).window).name, ss) != NULL as *mut ::core::ffi::c_char)
                    as ::core::ffi::c_int;
            }
            return (strstr((*(*wl).window).name, ss) != NULL as *mut ::core::ffi::c_char)
                as ::core::ffi::c_int;
        }
        3 => {
            if !(s.is_null() || wl.is_null() || wp.is_null()) {
                cmd = osdep_get_name((*wp).fd, &raw mut (*wp).tty as *mut ::core::ffi::c_char);
                if cmd.is_null() || *cmd as ::core::ffi::c_int == '\0' as i32 {
                    free(cmd as *mut ::core::ffi::c_void);
                    return 0 as ::core::ffi::c_int;
                }
                if icase != 0 {
                    retval = (strcasestr(cmd, ss) != NULL as *mut ::core::ffi::c_char)
                        as ::core::ffi::c_int;
                } else {
                    retval =
                        (strstr(cmd, ss) != NULL as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
                }
                free(cmd as *mut ::core::ffi::c_void);
                return retval;
            }
        }
        _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_tree_menu(
    mut modedata: *mut ::core::ffi::c_void,
    mut c: *mut client,
    mut key: key_code,
) {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut wp: *mut window_pane = (*data).wp;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    wme = (*wp).modes.tqh_first;
    if wme.is_null() || (*wme).data != modedata {
        return;
    }
    window_tree_key(
        wme,
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        key,
        ::core::ptr::null_mut::<mouse_event>(),
    );
}
unsafe extern "C" fn window_tree_get_key(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut line: u_int,
) -> key_code {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut item: *mut window_tree_itemdata = itemdata as *mut window_tree_itemdata;
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: key_code = 0;
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    window_tree_pull_item(item, &raw mut s, &raw mut wl, &raw mut wp);
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        format_defaults(
            ft,
            ::core::ptr::null_mut::<client>(),
            s,
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
    } else if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        format_defaults(
            ft,
            ::core::ptr::null_mut::<client>(),
            s,
            wl,
            ::core::ptr::null_mut::<window_pane>(),
        );
    } else {
        format_defaults(ft, ::core::ptr::null_mut::<client>(), s, wl, wp);
    }
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
unsafe extern "C" fn window_tree_swap(
    mut cur_itemdata: *mut ::core::ffi::c_void,
    mut other_itemdata: *mut ::core::ffi::c_void,
    mut sort_crit: *mut sort_criteria,
) -> ::core::ffi::c_int {
    let mut cur: *mut window_tree_itemdata = cur_itemdata as *mut window_tree_itemdata;
    let mut other: *mut window_tree_itemdata = other_itemdata as *mut window_tree_itemdata;
    let mut cur_session: *mut session = ::core::ptr::null_mut::<session>();
    let mut other_session: *mut session = ::core::ptr::null_mut::<session>();
    let mut cur_winlink: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut other_winlink: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut cur_window: *mut window = ::core::ptr::null_mut::<window>();
    let mut other_window: *mut window = ::core::ptr::null_mut::<window>();
    let mut cur_pane: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut other_pane: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if (*cur).type_0 as ::core::ffi::c_uint != (*other).type_0 as ::core::ffi::c_uint {
        return 0 as ::core::ffi::c_int;
    }
    if (*cur).type_0 as ::core::ffi::c_uint
        != WINDOW_TREE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    window_tree_pull_item(
        cur,
        &raw mut cur_session,
        &raw mut cur_winlink,
        &raw mut cur_pane,
    );
    window_tree_pull_item(
        other,
        &raw mut other_session,
        &raw mut other_winlink,
        &raw mut other_pane,
    );
    if cur_session.is_null() || cur_winlink.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if other_session.is_null() || other_winlink.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if cur_session != other_session {
        return 0 as ::core::ffi::c_int;
    }
    if sort_would_window_tree_swap(sort_crit, cur_winlink, other_winlink) != 0 {
        return 0 as ::core::ffi::c_int;
    }
    other_window = (*other_winlink).window;
    if !(*other_winlink).wentry.tqe_next.is_null() {
        (*(*other_winlink).wentry.tqe_next).wentry.tqe_prev = (*other_winlink).wentry.tqe_prev;
    } else {
        (*other_window).winlinks.tqh_last = (*other_winlink).wentry.tqe_prev;
    }
    *(*other_winlink).wentry.tqe_prev = (*other_winlink).wentry.tqe_next;
    cur_window = (*cur_winlink).window;
    if !(*cur_winlink).wentry.tqe_next.is_null() {
        (*(*cur_winlink).wentry.tqe_next).wentry.tqe_prev = (*cur_winlink).wentry.tqe_prev;
    } else {
        (*cur_window).winlinks.tqh_last = (*cur_winlink).wentry.tqe_prev;
    }
    *(*cur_winlink).wentry.tqe_prev = (*cur_winlink).wentry.tqe_next;
    (*other_winlink).window = cur_window;
    (*other_winlink).wentry.tqe_next = ::core::ptr::null_mut::<winlink>();
    (*other_winlink).wentry.tqe_prev = (*cur_window).winlinks.tqh_last;
    *(*cur_window).winlinks.tqh_last = other_winlink;
    (*cur_window).winlinks.tqh_last = &raw mut (*other_winlink).wentry.tqe_next;
    (*cur_winlink).window = other_window;
    (*cur_winlink).wentry.tqe_next = ::core::ptr::null_mut::<winlink>();
    (*cur_winlink).wentry.tqe_prev = (*other_window).winlinks.tqh_last;
    *(*other_window).winlinks.tqh_last = cur_winlink;
    (*other_window).winlinks.tqh_last = &raw mut (*cur_winlink).wentry.tqe_next;
    if (*cur_session).curw == cur_winlink {
        session_set_current(cur_session, other_winlink);
    } else if (*cur_session).curw == other_winlink {
        session_set_current(cur_session, cur_winlink);
    }
    session_group_synchronize_from(cur_session);
    server_redraw_session_group(cur_session);
    recalculate_sizes();
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_tree_sort(mut sort_crit: *mut sort_criteria) {
    (*sort_crit).order_seq = &raw mut window_tree_order_seq as *mut sort_order;
    if (*sort_crit).order as ::core::ffi::c_uint
        == SORT_END as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*sort_crit).order = *(*sort_crit)
            .order_seq
            .offset(0 as ::core::ffi::c_int as isize);
    }
}
static mut window_tree_help_lines: [*const ::core::ffi::c_char; 14] = [
    b"#[fg=themelightgrey]      Enter #[#{E:tree-mode-border-style},acs]x#[default] Choose selected item\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]       S-Up #[#{E:tree-mode-border-style},acs]x#[default] Swap current and previous window\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]     S-Down #[#{E:tree-mode-border-style},acs]x#[default] Swap current and next window\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          x #[#{E:tree-mode-border-style},acs]x#[default] Kill selected item\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          X #[#{E:tree-mode-border-style},acs]x#[default] Kill tagged items\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          < #[#{E:tree-mode-border-style},acs]x#[default] Scroll previews left\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          > #[#{E:tree-mode-border-style},acs]x#[default] Scroll previews right\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          m #[#{E:tree-mode-border-style},acs]x#[default] Set the marked pane\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          M #[#{E:tree-mode-border-style},acs]x#[default] Clear the marked pane\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          i #[#{E:tree-mode-border-style},acs]x#[default] Toggle session, window and pane information\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          : #[#{E:tree-mode-border-style},acs]x#[default] Run a command for each tagged item\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          f #[#{E:tree-mode-border-style},acs]x#[default] Enter a format\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          H #[#{E:tree-mode-border-style},acs]x#[default] Jump to the starting pane\0"
        as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
unsafe extern "C" fn window_tree_help(
    mut width: *mut u_int,
    mut item: *mut *const ::core::ffi::c_char,
) -> *mut *const ::core::ffi::c_char {
    *width = 51 as u_int;
    *item = b"item\0" as *const u8 as *const ::core::ffi::c_char;
    return &raw mut window_tree_help_lines as *mut *const ::core::ffi::c_char;
}
unsafe extern "C" fn window_tree_init(
    mut wme: *mut window_mode_entry,
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_tree_modedata = ::core::ptr::null_mut::<window_tree_modedata>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    data = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_tree_modedata>() as size_t,
    ) as *mut window_tree_modedata;
    (*wme).data = data as *mut ::core::ffi::c_void;
    (*data).wp = wp;
    (*data).references = 1 as ::core::ffi::c_int;
    if args_has(args, 's' as i32 as u_char) != 0 {
        (*data).type_0 = WINDOW_TREE_SESSION;
    } else if args_has(args, 'w' as i32 as u_char) != 0 {
        (*data).type_0 = WINDOW_TREE_WINDOW;
    } else {
        (*data).type_0 = WINDOW_TREE_PANE;
    }
    memcpy(
        &raw mut (*data).fs as *mut ::core::ffi::c_void,
        fs as *const ::core::ffi::c_void,
        ::core::mem::size_of::<cmd_find_state>() as size_t,
    );
    if args.is_null() || args_has(args, 'F' as i32 as u_char) == 0 {
        (*data).format = xstrdup(WINDOW_TREE_DEFAULT_FORMAT.as_ptr());
    } else {
        (*data).format = xstrdup(args_get(args, 'F' as i32 as u_char));
    }
    if args.is_null() || args_has(args, 'K' as i32 as u_char) == 0 {
        (*data).key_format = xstrdup(WINDOW_TREE_DEFAULT_KEY_FORMAT.as_ptr());
    } else {
        (*data).key_format = xstrdup(args_get(args, 'K' as i32 as u_char));
    }
    if args.is_null() || args_count(args) == 0 as u_int {
        (*data).command = xstrdup(WINDOW_TREE_DEFAULT_COMMAND.as_ptr());
    } else {
        (*data).command = xstrdup(args_string(args, 0 as u_int));
    }
    (*data).squash_groups = (args_has(args, 'G' as i32 as u_char) == 0) as ::core::ffi::c_int;
    (*data).hide_preview_this_pane = args_has(args, 'h' as i32 as u_char);
    if args_has(args, 'y' as i32 as u_char) != 0 {
        (*data).prompt_flags = PROMPT_ACCEPT;
    }
    (*data).data = mode_tree_start(
        wp,
        args,
        Some(
            window_tree_build
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut sort_criteria,
                    *mut uint64_t,
                    *const ::core::ffi::c_char,
                ) -> (),
        ),
        Some(
            window_tree_draw
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    *mut screen_write_ctx,
                    u_int,
                    u_int,
                ) -> (),
        ),
        Some(
            window_tree_search
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    *const ::core::ffi::c_char,
                    ::core::ffi::c_int,
                ) -> ::core::ffi::c_int,
        ),
        Some(
            window_tree_menu
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut client, key_code) -> (),
        ),
        None,
        Some(
            window_tree_get_key
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    u_int,
                ) -> key_code,
        ),
        Some(
            window_tree_swap
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    *mut sort_criteria,
                ) -> ::core::ffi::c_int,
        ),
        Some(window_tree_sort as unsafe extern "C" fn(*mut sort_criteria) -> ()),
        Some(
            window_tree_help
                as unsafe extern "C" fn(
                    *mut u_int,
                    *mut *const ::core::ffi::c_char,
                ) -> *mut *const ::core::ffi::c_char,
        ),
        data as *mut ::core::ffi::c_void,
        &raw const window_tree_menu_items as *const menu_item,
        &raw mut s,
    );
    mode_tree_zoom((*data).data, args);
    mode_tree_view_name(
        (*data).data,
        b"preview\0" as *const u8 as *const ::core::ffi::c_char,
    );
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*data).type_0 = WINDOW_TREE_NONE;
    return s;
}
unsafe extern "C" fn window_tree_destroy(mut data: *mut window_tree_modedata) {
    let mut i: u_int = 0;
    (*data).references -= 1;
    if (*data).references != 0 as ::core::ffi::c_int {
        return;
    }
    i = 0 as u_int;
    while i < (*data).item_size {
        window_tree_free_item(*(*data).item_list.offset(i as isize));
        i = i.wrapping_add(1);
    }
    free((*data).item_list as *mut ::core::ffi::c_void);
    free((*data).format as *mut ::core::ffi::c_void);
    free((*data).key_format as *mut ::core::ffi::c_void);
    free((*data).command as *mut ::core::ffi::c_void);
    free(data as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_tree_free(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_tree_modedata = (*wme).data as *mut window_tree_modedata;
    if data.is_null() {
        return;
    }
    (*data).dead = 1 as ::core::ffi::c_int;
    mode_tree_free((*data).data);
    window_tree_destroy(data);
}
unsafe extern "C" fn window_tree_resize(
    mut wme: *mut window_mode_entry,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_tree_modedata = (*wme).data as *mut window_tree_modedata;
    mode_tree_resize((*data).data, sx, sy);
}
unsafe extern "C" fn window_tree_update(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_tree_modedata = (*wme).data as *mut window_tree_modedata;
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*(*data).wp).flags |= PANE_REDRAW;
}
unsafe extern "C" fn window_tree_get_target(
    mut item: *mut window_tree_itemdata,
    mut fs: *mut cmd_find_state,
) -> *mut ::core::ffi::c_char {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut target: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    window_tree_pull_item(item, &raw mut s, &raw mut wl, &raw mut wp);
    target = ::core::ptr::null_mut::<::core::ffi::c_char>();
    match (*item).type_0 as ::core::ffi::c_uint {
        1 => {
            if !s.is_null() {
                xasprintf(
                    &raw mut target,
                    b"=%s:\0" as *const u8 as *const ::core::ffi::c_char,
                    (*s).name,
                );
            }
        }
        2 => {
            if !(s.is_null() || wl.is_null()) {
                xasprintf(
                    &raw mut target,
                    b"=%s:%u.\0" as *const u8 as *const ::core::ffi::c_char,
                    (*s).name,
                    (*wl).idx,
                );
            }
        }
        3 => {
            if !(s.is_null() || wl.is_null() || wp.is_null()) {
                xasprintf(
                    &raw mut target,
                    b"=%s:%u.%%%u\0" as *const u8 as *const ::core::ffi::c_char,
                    (*s).name,
                    (*wl).idx,
                    (*wp).id,
                );
            }
        }
        0 | _ => {}
    }
    if target.is_null() {
        cmd_find_clear_state(fs, 0 as ::core::ffi::c_int);
    } else {
        cmd_find_from_winlink_pane(fs, wl, wp, 0 as ::core::ffi::c_int);
    }
    return target;
}
unsafe extern "C" fn window_tree_command_each(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut c: *mut client,
    mut key: key_code,
) {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut item: *mut window_tree_itemdata = itemdata as *mut window_tree_itemdata;
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    name = window_tree_get_target(item, &raw mut fs);
    if !name.is_null() {
        mode_tree_run_command(c, &raw mut fs, (*data).entered, name);
    }
    free(name as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_tree_command_done(
    mut item: *mut cmdq_item,
    mut modedata: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    if (*data).dead == 0 {
        mode_tree_build((*data).data);
        mode_tree_draw((*data).data);
        (*(*data).wp).flags |= PANE_REDRAW;
    }
    window_tree_destroy(data);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn window_tree_command_callback(
    mut c: *mut client,
    mut modedata: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    (*data).entered = s;
    mode_tree_each_tagged(
        (*data).data,
        Some(
            window_tree_command_each
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    *mut client,
                    key_code,
                ) -> (),
        ),
        c,
        KEYC_NONE as ::core::ffi::c_ulong as key_code,
        1 as ::core::ffi::c_int,
    );
    (*data).entered = ::core::ptr::null::<::core::ffi::c_char>();
    (*data).references += 1;
    cmdq_append(
        c,
        cmdq_get_callback1(
            b"window_tree_command_done\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                window_tree_command_done
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            data as *mut ::core::ffi::c_void,
        ),
    );
    return PROMPT_CLOSE;
}
unsafe extern "C" fn window_tree_command_free(mut modedata: *mut ::core::ffi::c_void) {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    window_tree_destroy(data);
}
unsafe extern "C" fn window_tree_kill_each(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut c: *mut client,
    mut key: key_code,
) {
    let mut item: *mut window_tree_itemdata = itemdata as *mut window_tree_itemdata;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    window_tree_pull_item(item, &raw mut s, &raw mut wl, &raw mut wp);
    match (*item).type_0 as ::core::ffi::c_uint {
        1 => {
            if !s.is_null() {
                server_destroy_session(s);
                session_destroy(
                    s,
                    1 as ::core::ffi::c_int,
                    b"window_tree_kill_each\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        2 => {
            if !wl.is_null() {
                server_kill_window((*wl).window, 0 as ::core::ffi::c_int);
            }
        }
        3 => {
            if !wp.is_null() {
                server_kill_pane(wp);
            }
        }
        0 | _ => {}
    };
}
unsafe extern "C" fn window_tree_kill_current_callback(
    mut c: *mut client,
    mut modedata: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut mtd: *mut mode_tree_data = (*data).data;
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if ({
        let mut __res: ::core::ffi::c_int = 0;
        if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
            if 0 != 0 {
                let mut __c: ::core::ffi::c_int =
                    *s.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int;
                __res = (if __c < -(128 as ::core::ffi::c_int) || __c > 255 as ::core::ffi::c_int {
                    __c as __int32_t
                } else {
                    *(*__ctype_tolower_loc()).offset(__c as isize)
                }) as ::core::ffi::c_int;
            } else {
                __res = tolower(
                    *s.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int
                );
            }
        } else {
            __res = *(*__ctype_tolower_loc()).offset(*s.offset(0 as ::core::ffi::c_int as isize)
                as u_char as ::core::ffi::c_int
                as isize) as ::core::ffi::c_int;
        }
        __res
    }) != 'y' as i32
        || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
    {
        return PROMPT_CLOSE;
    }
    window_tree_kill_each(
        data as *mut ::core::ffi::c_void,
        mode_tree_get_current(mtd),
        c,
        KEYC_NONE as ::core::ffi::c_ulong as key_code,
    );
    server_renumber_all();
    (*data).references += 1;
    cmdq_append(
        c,
        cmdq_get_callback1(
            b"window_tree_command_done\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                window_tree_command_done
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            data as *mut ::core::ffi::c_void,
        ),
    );
    return PROMPT_CLOSE;
}
unsafe extern "C" fn window_tree_kill_tagged_callback(
    mut c: *mut client,
    mut modedata: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut data: *mut window_tree_modedata = modedata as *mut window_tree_modedata;
    let mut mtd: *mut mode_tree_data = (*data).data;
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if ({
        let mut __res: ::core::ffi::c_int = 0;
        if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
            if 0 != 0 {
                let mut __c: ::core::ffi::c_int =
                    *s.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int;
                __res = (if __c < -(128 as ::core::ffi::c_int) || __c > 255 as ::core::ffi::c_int {
                    __c as __int32_t
                } else {
                    *(*__ctype_tolower_loc()).offset(__c as isize)
                }) as ::core::ffi::c_int;
            } else {
                __res = tolower(
                    *s.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int
                );
            }
        } else {
            __res = *(*__ctype_tolower_loc()).offset(*s.offset(0 as ::core::ffi::c_int as isize)
                as u_char as ::core::ffi::c_int
                as isize) as ::core::ffi::c_int;
        }
        __res
    }) != 'y' as i32
        || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
    {
        return PROMPT_CLOSE;
    }
    mode_tree_each_tagged(
        mtd,
        Some(
            window_tree_kill_each
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    *mut client,
                    key_code,
                ) -> (),
        ),
        c,
        KEYC_NONE as ::core::ffi::c_ulong as key_code,
        1 as ::core::ffi::c_int,
    );
    server_renumber_all();
    (*data).references += 1;
    cmdq_append(
        c,
        cmdq_get_callback1(
            b"window_tree_command_done\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                window_tree_command_done
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            data as *mut ::core::ffi::c_void,
        ),
    );
    return PROMPT_CLOSE;
}
unsafe extern "C" fn window_tree_mouse(
    mut data: *mut window_tree_modedata,
    mut key: key_code,
    mut x: u_int,
    mut item: *mut window_tree_itemdata,
) -> key_code {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut loop_0: u_int = 0;
    if key != KEYC_MOUSEDOWN1_PANE as ::core::ffi::c_ulong as key_code {
        return KEYC_NONE as ::core::ffi::c_ulong as key_code;
    }
    if (*data).left != -(1 as ::core::ffi::c_int) && x <= (*data).left as u_int {
        return '<' as i32 as key_code;
    }
    if (*data).right != -(1 as ::core::ffi::c_int) && x >= (*data).right as u_int {
        return '>' as i32 as key_code;
    }
    if (*data).left != -(1 as ::core::ffi::c_int) {
        x = x.wrapping_sub((*data).left as u_int);
    } else if x != 0 as u_int {
        x = x.wrapping_sub(1);
    }
    if x == 0 as u_int || (*data).end == 0 as u_int {
        x = 0 as u_int;
    } else {
        x = x.wrapping_div((*data).each);
        if (*data).start.wrapping_add(x) >= (*data).end {
            x = (*data).end.wrapping_sub(1 as u_int);
        }
    }
    window_tree_pull_item(item, &raw mut s, &raw mut wl, &raw mut wp);
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if s.is_null() {
            return KEYC_NONE as ::core::ffi::c_ulong as key_code;
        }
        mode_tree_expand_current((*data).data);
        loop_0 = 0 as u_int;
        wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
        while !wl.is_null() {
            if loop_0 == (*data).start.wrapping_add(x) {
                break;
            }
            loop_0 = loop_0.wrapping_add(1);
            wl = winlinks_RB_NEXT(wl);
        }
        if !wl.is_null() {
            mode_tree_set_current((*data).data, wl as uint64_t);
        }
        return '\r' as i32 as key_code;
    }
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_TREE_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if wl.is_null() {
            return KEYC_NONE as ::core::ffi::c_ulong as key_code;
        }
        mode_tree_expand_current((*data).data);
        loop_0 = 0 as u_int;
        wp = (*(*wl).window).panes.tqh_first;
        while !wp.is_null() {
            if loop_0 == (*data).start.wrapping_add(x) {
                break;
            }
            loop_0 = loop_0.wrapping_add(1);
            wp = (*wp).entry.tqe_next;
        }
        if !wp.is_null() {
            mode_tree_set_current((*data).data, wp as uint64_t);
        }
        return '\r' as i32 as key_code;
    }
    return KEYC_NONE as ::core::ffi::c_ulong as key_code;
}
unsafe extern "C" fn window_tree_key(
    mut wme: *mut window_mode_entry,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_tree_modedata = (*wme).data as *mut window_tree_modedata;
    let mut item: *mut window_tree_itemdata = ::core::ptr::null_mut::<window_tree_itemdata>();
    let mut new_item: *mut window_tree_itemdata = ::core::ptr::null_mut::<window_tree_itemdata>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut prompt: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut fsp: *mut cmd_find_state = &raw mut (*data).fs;
    let mut finished: ::core::ffi::c_int = 0;
    let mut tagged: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut idx: u_int = 0;
    let mut ns: *mut session = ::core::ptr::null_mut::<session>();
    let mut nwl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut nwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    item = mode_tree_get_current((*data).data) as *mut window_tree_itemdata;
    finished = mode_tree_key((*data).data, c, &raw mut key, m, &raw mut x, &raw mut y);
    loop {
        new_item = mode_tree_get_current((*data).data) as *mut window_tree_itemdata;
        if item != new_item {
            item = new_item;
            (*data).offset = 0 as ::core::ffi::c_int;
        }
        if !((key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int)
            && !m.is_null())
        {
            break;
        }
        key = window_tree_mouse(data, key, x, item);
    }
    match key {
        60 => {
            (*data).offset -= 1;
        }
        62 => {
            (*data).offset += 1;
        }
        72 => {
            mode_tree_expand((*data).data, (*fsp).s as uint64_t);
            mode_tree_expand((*data).data, (*fsp).wl as uint64_t);
            if mode_tree_set_current((*data).data, (*wme).wp as uint64_t) == 0 {
                mode_tree_set_current((*data).data, (*fsp).wl as uint64_t);
            }
        }
        109 => {
            window_tree_pull_item(item, &raw mut ns, &raw mut nwl, &raw mut nwp);
            server_set_marked(ns, nwl, nwp);
            mode_tree_build((*data).data);
        }
        77 => {
            server_clear_marked();
            mode_tree_build((*data).data);
        }
        105 => {
            (*data).preview_is_info = ((*data).preview_is_info == 0) as ::core::ffi::c_int;
            if (*data).preview_is_info != 0 {
                mode_tree_view_name(
                    (*data).data,
                    b"info\0" as *const u8 as *const ::core::ffi::c_char,
                );
            } else {
                mode_tree_view_name(
                    (*data).data,
                    b"preview\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        120 => {
            window_tree_pull_item(item, &raw mut ns, &raw mut nwl, &raw mut nwp);
            match (*item).type_0 as ::core::ffi::c_uint {
                1 => {
                    if !ns.is_null() {
                        xasprintf(
                            &raw mut prompt,
                            b"Kill session %s? \0" as *const u8 as *const ::core::ffi::c_char,
                            (*ns).name,
                        );
                    }
                }
                2 => {
                    if !nwl.is_null() {
                        xasprintf(
                            &raw mut prompt,
                            b"Kill window %u? \0" as *const u8 as *const ::core::ffi::c_char,
                            (*nwl).idx,
                        );
                    }
                }
                3 => {
                    if !(nwp.is_null()
                        || window_pane_index(nwp, &raw mut idx) != 0 as ::core::ffi::c_int)
                    {
                        xasprintf(
                            &raw mut prompt,
                            b"Kill pane %u? \0" as *const u8 as *const ::core::ffi::c_char,
                            idx,
                        );
                    }
                }
                0 | _ => {}
            }
            if !prompt.is_null() {
                (*data).references += 1;
                mode_tree_set_prompt(
                    (*data).data,
                    c,
                    prompt,
                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                    PROMPT_TYPE_COMMAND,
                    PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                    Some(
                        window_tree_kill_current_callback
                            as unsafe extern "C" fn(
                                *mut client,
                                *mut ::core::ffi::c_void,
                                *const ::core::ffi::c_char,
                                prompt_key_result,
                            ) -> prompt_result,
                    ),
                    Some(
                        window_tree_command_free
                            as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
                    ),
                    data as *mut ::core::ffi::c_void,
                );
                free(prompt as *mut ::core::ffi::c_void);
            }
        }
        88 => {
            tagged = mode_tree_count_tagged((*data).data);
            if !(tagged == 0 as u_int) {
                xasprintf(
                    &raw mut prompt,
                    b"Kill %u tagged? \0" as *const u8 as *const ::core::ffi::c_char,
                    tagged,
                );
                (*data).references += 1;
                mode_tree_set_prompt(
                    (*data).data,
                    c,
                    prompt,
                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                    PROMPT_TYPE_COMMAND,
                    PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                    Some(
                        window_tree_kill_tagged_callback
                            as unsafe extern "C" fn(
                                *mut client,
                                *mut ::core::ffi::c_void,
                                *const ::core::ffi::c_char,
                                prompt_key_result,
                            ) -> prompt_result,
                    ),
                    Some(
                        window_tree_command_free
                            as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
                    ),
                    data as *mut ::core::ffi::c_void,
                );
                free(prompt as *mut ::core::ffi::c_void);
            }
        }
        58 => {
            tagged = mode_tree_count_tagged((*data).data);
            if tagged != 0 as u_int {
                xasprintf(
                    &raw mut prompt,
                    b"(%u tagged) \0" as *const u8 as *const ::core::ffi::c_char,
                    tagged,
                );
            } else {
                xasprintf(
                    &raw mut prompt,
                    b"(current) \0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            (*data).references += 1;
            mode_tree_set_prompt(
                (*data).data,
                c,
                prompt,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
                PROMPT_TYPE_COMMAND,
                PROMPT_NOFORMAT,
                Some(
                    window_tree_command_callback
                        as unsafe extern "C" fn(
                            *mut client,
                            *mut ::core::ffi::c_void,
                            *const ::core::ffi::c_char,
                            prompt_key_result,
                        ) -> prompt_result,
                ),
                Some(
                    window_tree_command_free
                        as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
                ),
                data as *mut ::core::ffi::c_void,
            );
            free(prompt as *mut ::core::ffi::c_void);
        }
        13 => {
            name = window_tree_get_target(item, &raw mut fs);
            if !name.is_null() {
                mode_tree_run_command(
                    c,
                    ::core::ptr::null_mut::<cmd_find_state>(),
                    (*data).command,
                    name,
                );
            }
            finished = 1 as ::core::ffi::c_int;
            free(name as *mut ::core::ffi::c_void);
        }
        _ => {}
    }
    if finished != 0 {
        window_pane_reset_mode(wp);
    } else {
        mode_tree_draw((*data).data);
        (*wp).flags |= PANE_REDRAW;
    };
}

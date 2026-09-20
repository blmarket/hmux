pub use crate::src::shared::limits::SIZE_MAX;
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{CMD_LIST_PRINT_ESCAPED, CMD_LIST_PRINT_NO_GROUPS};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
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
    pub type environ;
    pub type options;
    pub type menu_data;
    pub type window_pane_prompt;
    pub type prompt;
    pub type format_tree;
    pub type cmdq_item;
    pub type input_ctx;
    pub type spawn_editor_state;
    pub type input_request;
    pub type redraw_scene;
    pub type tty_key;
    pub type tty_code;
    pub type format_job_tree;
    pub type control_state;
    pub type cmdq_list;
    pub type options_array_item;
    pub type options_entry;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strlcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    fn strlcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(_: *mut ::core::ffi::c_void, _: size_t) -> *mut ::core::ffi::c_void;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn xvasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    static mut global_options: *mut options;
    fn options_get_only(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_array_first(_: *mut options_entry) -> *mut options_array_item;
    fn options_array_next(_: *mut options_array_item) -> *mut options_array_item;
    fn options_array_item_value(_: *mut options_array_item) -> *mut options_value;
    fn args_parse(
        _: *const args_parse,
        _: *mut args_value,
        _: u_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut args;
    fn args_copy(
        _: *mut args,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut args;
    fn args_free(_: *mut args);
    fn args_print(_: *mut args) -> *mut ::core::ffi::c_char;
    fn args_escape(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn winlink_find_by_window(_: *mut winlinks, _: *mut window) -> *mut winlink;
    fn window_find_by_id(_: u_int) -> *mut window;
    fn window_has_pane(_: *mut window, _: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_find_by_id(_: u_int) -> *mut window_pane;
    fn session_find_by_id(_: u_int) -> *mut session;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
    static cmd_attach_session_entry: cmd_entry;
    static cmd_bind_key_entry: cmd_entry;
    static cmd_break_pane_entry: cmd_entry;
    static cmd_capture_pane_entry: cmd_entry;
    static cmd_choose_buffer_entry: cmd_entry;
    static cmd_choose_client_entry: cmd_entry;
    static cmd_choose_tree_entry: cmd_entry;
    static cmd_clear_history_entry: cmd_entry;
    static cmd_clear_prompt_history_entry: cmd_entry;
    static cmd_clock_mode_entry: cmd_entry;
    static cmd_command_prompt_entry: cmd_entry;
    static cmd_confirm_before_entry: cmd_entry;
    static cmd_copy_mode_entry: cmd_entry;
    static cmd_customize_mode_entry: cmd_entry;
    static cmd_delete_buffer_entry: cmd_entry;
    static cmd_detach_client_entry: cmd_entry;
    static cmd_display_menu_entry: cmd_entry;
    static cmd_display_message_entry: cmd_entry;
    static cmd_display_popup_entry: cmd_entry;
    static cmd_display_panes_entry: cmd_entry;
    static cmd_find_window_entry: cmd_entry;
    static cmd_has_session_entry: cmd_entry;
    static cmd_if_shell_entry: cmd_entry;
    static cmd_join_pane_entry: cmd_entry;
    static cmd_kill_pane_entry: cmd_entry;
    static cmd_kill_server_entry: cmd_entry;
    static cmd_kill_session_entry: cmd_entry;
    static cmd_kill_window_entry: cmd_entry;
    static cmd_last_pane_entry: cmd_entry;
    static cmd_last_window_entry: cmd_entry;
    static cmd_link_window_entry: cmd_entry;
    static cmd_list_buffers_entry: cmd_entry;
    static cmd_list_clients_entry: cmd_entry;
    static cmd_list_commands_entry: cmd_entry;
    static cmd_list_keys_entry: cmd_entry;
    static cmd_list_panes_entry: cmd_entry;
    static cmd_list_sessions_entry: cmd_entry;
    static cmd_list_windows_entry: cmd_entry;
    static cmd_load_buffer_entry: cmd_entry;
    static cmd_lock_client_entry: cmd_entry;
    static cmd_lock_server_entry: cmd_entry;
    static cmd_lock_session_entry: cmd_entry;
    static cmd_move_pane_entry: cmd_entry;
    static cmd_move_window_entry: cmd_entry;
    static cmd_new_pane_entry: cmd_entry;
    static cmd_new_session_entry: cmd_entry;
    static cmd_new_window_entry: cmd_entry;
    static cmd_next_layout_entry: cmd_entry;
    static cmd_next_window_entry: cmd_entry;
    static cmd_paste_buffer_entry: cmd_entry;
    static cmd_pipe_pane_entry: cmd_entry;
    static cmd_previous_layout_entry: cmd_entry;
    static cmd_previous_window_entry: cmd_entry;
    static cmd_refresh_client_entry: cmd_entry;
    static cmd_rename_session_entry: cmd_entry;
    static cmd_rename_window_entry: cmd_entry;
    static cmd_resize_pane_entry: cmd_entry;
    static cmd_resize_window_entry: cmd_entry;
    static cmd_respawn_pane_entry: cmd_entry;
    static cmd_respawn_window_entry: cmd_entry;
    static cmd_rotate_window_entry: cmd_entry;
    static cmd_run_shell_entry: cmd_entry;
    static cmd_save_buffer_entry: cmd_entry;
    static cmd_select_layout_entry: cmd_entry;
    static cmd_select_pane_entry: cmd_entry;
    static cmd_select_window_entry: cmd_entry;
    static cmd_send_keys_entry: cmd_entry;
    static cmd_send_prefix_entry: cmd_entry;
    static cmd_server_access_entry: cmd_entry;
    static cmd_set_buffer_entry: cmd_entry;
    static cmd_set_environment_entry: cmd_entry;
    static cmd_set_hook_entry: cmd_entry;
    static cmd_set_option_entry: cmd_entry;
    static cmd_set_window_option_entry: cmd_entry;
    static cmd_show_buffer_entry: cmd_entry;
    static cmd_show_environment_entry: cmd_entry;
    static cmd_show_hooks_entry: cmd_entry;
    static cmd_show_messages_entry: cmd_entry;
    static cmd_show_options_entry: cmd_entry;
    static cmd_show_prompt_history_entry: cmd_entry;
    static cmd_show_window_options_entry: cmd_entry;
    static cmd_source_file_entry: cmd_entry;
    static cmd_split_window_entry: cmd_entry;
    static cmd_start_server_entry: cmd_entry;
    static cmd_suspend_client_entry: cmd_entry;
    static cmd_swap_pane_entry: cmd_entry;
    static cmd_swap_window_entry: cmd_entry;
    static cmd_switch_client_entry: cmd_entry;
    static cmd_switch_mode_entry: cmd_entry;
    static cmd_unbind_key_entry: cmd_entry;
    static cmd_unlink_window_entry: cmd_entry;
    static cmd_wait_for_entry: cmd_entry;
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
pub struct cmds {
    pub tqh_first: *mut cmd,
    pub tqh_last: *mut *mut cmd,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd {
    pub entry: *const cmd_entry,
    pub args: *mut args,
    pub group: u_int,
    pub file: *mut ::core::ffi::c_char,
    pub line: u_int,
    pub parse_flags: ::core::ffi::c_int,
    pub qentry: C2RustUnnamed_33,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_33 {
    pub tqe_next: *mut cmd,
    pub tqe_prev: *mut *mut cmd,
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
pub struct cmd_entry_flag {
    pub flag: ::core::ffi::c_char,
    pub type_0: cmd_find_type,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_parse {
    pub template: *const ::core::ffi::c_char,
    pub lower: ::core::ffi::c_int,
    pub upper: ::core::ffi::c_int,
    pub cb: args_parse_cb,
}
pub type args_parse_cb = Option<
    unsafe extern "C" fn(*mut args, u_int, *mut *mut ::core::ffi::c_char) -> args_parse_type,
>;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_value {
    pub type_0: args_type,
    pub c2rust_unnamed: C2RustUnnamed_37,
    pub cached: *mut ::core::ffi::c_char,
    pub entry: C2RustUnnamed_36,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_36 {
    pub tqe_next: *mut args_value,
    pub tqe_prev: *mut *mut args_value,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_37 {
    pub string: *mut ::core::ffi::c_char,
    pub cmdlist: *mut cmd_list,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_array {
    pub rbh_root: *mut options_array_item,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union options_value {
    pub string: *mut ::core::ffi::c_char,
    pub number: ::core::ffi::c_longlong,
    pub style: style,
    pub array: options_array,
    pub cmdlist: *mut cmd_list,
}
pub const DQ: C2RustUnnamed_38 = 2;
pub type C2RustUnnamed_38 = ::core::ffi::c_uint;
pub const SQ: C2RustUnnamed_38 = 1;
pub const NQ: C2RustUnnamed_38 = 0;

#[no_mangle]
pub static mut cmd_table: [*const cmd_entry; 93] = unsafe {
    [
        &raw const cmd_attach_session_entry,
        &raw const cmd_bind_key_entry,
        &raw const cmd_break_pane_entry,
        &raw const cmd_capture_pane_entry,
        &raw const cmd_choose_buffer_entry,
        &raw const cmd_choose_client_entry,
        &raw const cmd_choose_tree_entry,
        &raw const cmd_clear_history_entry,
        &raw const cmd_clear_prompt_history_entry,
        &raw const cmd_clock_mode_entry,
        &raw const cmd_command_prompt_entry,
        &raw const cmd_confirm_before_entry,
        &raw const cmd_copy_mode_entry,
        &raw const cmd_customize_mode_entry,
        &raw const cmd_delete_buffer_entry,
        &raw const cmd_detach_client_entry,
        &raw const cmd_display_menu_entry,
        &raw const cmd_display_message_entry,
        &raw const cmd_display_popup_entry,
        &raw const cmd_display_panes_entry,
        &raw const cmd_find_window_entry,
        &raw const cmd_has_session_entry,
        &raw const cmd_if_shell_entry,
        &raw const cmd_join_pane_entry,
        &raw const cmd_kill_pane_entry,
        &raw const cmd_kill_server_entry,
        &raw const cmd_kill_session_entry,
        &raw const cmd_kill_window_entry,
        &raw const cmd_last_pane_entry,
        &raw const cmd_last_window_entry,
        &raw const cmd_link_window_entry,
        &raw const cmd_list_buffers_entry,
        &raw const cmd_list_clients_entry,
        &raw const cmd_list_commands_entry,
        &raw const cmd_list_keys_entry,
        &raw const cmd_list_panes_entry,
        &raw const cmd_list_sessions_entry,
        &raw const cmd_list_windows_entry,
        &raw const cmd_load_buffer_entry,
        &raw const cmd_lock_client_entry,
        &raw const cmd_lock_server_entry,
        &raw const cmd_lock_session_entry,
        &raw const cmd_move_pane_entry,
        &raw const cmd_move_window_entry,
        &raw const cmd_new_pane_entry,
        &raw const cmd_new_session_entry,
        &raw const cmd_new_window_entry,
        &raw const cmd_next_layout_entry,
        &raw const cmd_next_window_entry,
        &raw const cmd_paste_buffer_entry,
        &raw const cmd_pipe_pane_entry,
        &raw const cmd_previous_layout_entry,
        &raw const cmd_previous_window_entry,
        &raw const cmd_refresh_client_entry,
        &raw const cmd_rename_session_entry,
        &raw const cmd_rename_window_entry,
        &raw const cmd_resize_pane_entry,
        &raw const cmd_resize_window_entry,
        &raw const cmd_respawn_pane_entry,
        &raw const cmd_respawn_window_entry,
        &raw const cmd_rotate_window_entry,
        &raw const cmd_run_shell_entry,
        &raw const cmd_save_buffer_entry,
        &raw const cmd_select_layout_entry,
        &raw const cmd_select_pane_entry,
        &raw const cmd_select_window_entry,
        &raw const cmd_send_keys_entry,
        &raw const cmd_send_prefix_entry,
        &raw const cmd_server_access_entry,
        &raw const cmd_set_buffer_entry,
        &raw const cmd_set_environment_entry,
        &raw const cmd_set_hook_entry,
        &raw const cmd_set_option_entry,
        &raw const cmd_set_window_option_entry,
        &raw const cmd_show_buffer_entry,
        &raw const cmd_show_environment_entry,
        &raw const cmd_show_hooks_entry,
        &raw const cmd_show_messages_entry,
        &raw const cmd_show_options_entry,
        &raw const cmd_show_prompt_history_entry,
        &raw const cmd_show_window_options_entry,
        &raw const cmd_source_file_entry,
        &raw const cmd_split_window_entry,
        &raw const cmd_start_server_entry,
        &raw const cmd_suspend_client_entry,
        &raw const cmd_swap_pane_entry,
        &raw const cmd_swap_window_entry,
        &raw const cmd_switch_client_entry,
        &raw const cmd_switch_mode_entry,
        &raw const cmd_unbind_key_entry,
        &raw const cmd_unlink_window_entry,
        &raw const cmd_wait_for_entry,
        ::core::ptr::null::<cmd_entry>(),
    ]
};
static mut cmd_list_next_group: u_int = 1 as u_int;
#[no_mangle]
pub unsafe extern "C" fn cmd_log_argv(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut prefix: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ap: ::core::ffi::VaList;
    let mut i: ::core::ffi::c_int = 0;
    ap = args.clone();
    xvasprintf(&raw mut prefix, fmt, ap);
    i = 0 as ::core::ffi::c_int;
    while i < argc {
        log_debug(
            b"%s: argv[%d]=%s\0" as *const u8 as *const ::core::ffi::c_char,
            prefix,
            i,
            *argv.offset(i as isize),
        );
        i += 1;
    }
    free(prefix as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_prepend_argv(
    mut argc: *mut ::core::ffi::c_int,
    mut argv: *mut *mut *mut ::core::ffi::c_char,
    mut arg: *const ::core::ffi::c_char,
) {
    let mut new_argv: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0;
    new_argv = xreallocarray(
        NULL,
        (*argc + 1 as ::core::ffi::c_int) as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
    ) as *mut *mut ::core::ffi::c_char;
    let ref mut fresh0 = *new_argv.offset(0 as ::core::ffi::c_int as isize);
    *fresh0 = xstrdup(arg);
    i = 0 as ::core::ffi::c_int;
    while i < *argc {
        let ref mut fresh1 = *new_argv.offset((1 as ::core::ffi::c_int + i) as isize);
        *fresh1 = *(*argv).offset(i as isize);
        i += 1;
    }
    free(*argv as *mut ::core::ffi::c_void);
    *argv = new_argv;
    *argc += 1;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_append_argv(
    mut argc: *mut ::core::ffi::c_int,
    mut argv: *mut *mut *mut ::core::ffi::c_char,
    mut arg: *const ::core::ffi::c_char,
) {
    *argv = xreallocarray(
        *argv as *mut ::core::ffi::c_void,
        (*argc + 1 as ::core::ffi::c_int) as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
    ) as *mut *mut ::core::ffi::c_char;
    let fresh2 = *argc;
    *argc = *argc + 1;
    let ref mut fresh3 = *(*argv).offset(fresh2 as isize);
    *fresh3 = xstrdup(arg);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_pack_argv(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut arglen: size_t = 0;
    let mut i: ::core::ffi::c_int = 0;
    if argc == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    cmd_log_argv(
        argc,
        argv,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_pack_argv\0" as *const u8 as *const ::core::ffi::c_char,
    );
    *buf = '\0' as i32 as ::core::ffi::c_char;
    i = 0 as ::core::ffi::c_int;
    while i < argc {
        if strlcpy(buf, *argv.offset(i as isize), len) as size_t >= len {
            return -(1 as ::core::ffi::c_int);
        }
        arglen = strlen(*argv.offset(i as isize)).wrapping_add(1 as size_t);
        buf = buf.offset(arglen as isize);
        len = len.wrapping_sub(arglen);
        i += 1;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_unpack_argv(
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut arglen: size_t = 0;
    if argc == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if argc < 0 as ::core::ffi::c_int || argc > 1000 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    *argv = xcalloc(
        argc as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
    ) as *mut *mut ::core::ffi::c_char;
    *buf.offset(len.wrapping_sub(1 as size_t) as isize) = '\0' as i32 as ::core::ffi::c_char;
    i = 0 as ::core::ffi::c_int;
    while i < argc {
        if len == 0 as size_t {
            cmd_free_argv(argc, *argv);
            return -(1 as ::core::ffi::c_int);
        }
        arglen = strlen(buf).wrapping_add(1 as size_t);
        let ref mut fresh4 = *(*argv).offset(i as isize);
        *fresh4 = xstrdup(buf);
        buf = buf.offset(arglen as isize);
        len = len.wrapping_sub(arglen);
        i += 1;
    }
    cmd_log_argv(
        argc,
        *argv,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_unpack_argv\0" as *const u8 as *const ::core::ffi::c_char,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_copy_argv(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) -> *mut *mut ::core::ffi::c_char {
    let mut new_argv: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0;
    if argc == 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    }
    new_argv = xcalloc(
        (argc + 1 as ::core::ffi::c_int) as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
    ) as *mut *mut ::core::ffi::c_char;
    i = 0 as ::core::ffi::c_int;
    while i < argc {
        if !(*argv.offset(i as isize)).is_null() {
            let ref mut fresh5 = *new_argv.offset(i as isize);
            *fresh5 = xstrdup(*argv.offset(i as isize));
        }
        i += 1;
    }
    return new_argv;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_free_argv(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) {
    let mut i: ::core::ffi::c_int = 0;
    if argc == 0 as ::core::ffi::c_int {
        return;
    }
    i = 0 as ::core::ffi::c_int;
    while i < argc {
        free(*argv.offset(i as isize) as *mut ::core::ffi::c_void);
        i += 1;
    }
    free(argv as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_stringify_argv(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0 as size_t;
    let mut i: ::core::ffi::c_int = 0;
    if argc == 0 as ::core::ffi::c_int {
        return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    }
    i = 0 as ::core::ffi::c_int;
    while i < argc {
        s = args_escape(*argv.offset(i as isize));
        log_debug(
            b"%s: %u %s = %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_stringify_argv\0" as *const u8 as *const ::core::ffi::c_char,
            i,
            *argv.offset(i as isize),
            s,
        );
        len = len.wrapping_add(strlen(s).wrapping_add(1 as size_t));
        buf = xrealloc(buf as *mut ::core::ffi::c_void, len) as *mut ::core::ffi::c_char;
        if i == 0 as ::core::ffi::c_int {
            *buf = '\0' as i32 as ::core::ffi::c_char;
        } else {
            strlcat(buf, b" \0" as *const u8 as *const ::core::ffi::c_char, len);
        }
        strlcat(buf, s, len);
        free(s as *mut ::core::ffi::c_void);
        i += 1;
    }
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_get_entry(mut cmd: *mut cmd) -> *const cmd_entry {
    return (*cmd).entry;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_get_args(mut cmd: *mut cmd) -> *mut args {
    return (*cmd).args;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_get_group(mut cmd: *mut cmd) -> u_int {
    return (*cmd).group;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_get_source(
    mut cmd: *mut cmd,
    mut file: *mut *const ::core::ffi::c_char,
    mut line: *mut u_int,
) {
    if !file.is_null() {
        *file = (*cmd).file;
    }
    if !line.is_null() {
        *line = (*cmd).line;
    }
}
#[no_mangle]
pub unsafe extern "C" fn cmd_get_parse_flags(mut cmd: *mut cmd) -> ::core::ffi::c_int {
    return (*cmd).parse_flags;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_get_alias(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut wanted: size_t = 0;
    let mut n: size_t = 0;
    let mut equals: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    o = options_get_only(
        global_options,
        b"command-alias\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if o.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    wanted = strlen(name);
    a = options_array_first(o);
    while !a.is_null() {
        ov = options_array_item_value(a);
        equals = strchr((*ov).string, '=' as i32);
        if !equals.is_null() {
            n = equals.offset_from((*ov).string) as ::core::ffi::c_long as size_t;
            if n == wanted && strncmp(name, (*ov).string, n) == 0 as ::core::ffi::c_int {
                return xstrdup(equals.offset(1 as ::core::ffi::c_int as isize));
            }
        }
        a = options_array_next(a);
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find(
    mut name: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *const cmd_entry {
    let mut loop_0: *mut *const cmd_entry = ::core::ptr::null_mut::<*const cmd_entry>();
    let mut entry: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
    let mut found: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
    let mut ambiguous: ::core::ffi::c_int = 0;
    let mut s: [::core::ffi::c_char; 8192] = [0; 8192];
    ambiguous = 0 as ::core::ffi::c_int;
    loop_0 = &raw mut cmd_table as *mut *const cmd_entry;
    while !(*loop_0).is_null() {
        entry = *loop_0;
        if !(*entry).alias.is_null() && strcmp((*entry).alias, name) == 0 as ::core::ffi::c_int {
            ambiguous = 0 as ::core::ffi::c_int;
            found = entry;
            break;
        } else {
            if !(strncmp((*entry).name, name, strlen(name)) != 0 as ::core::ffi::c_int) {
                if !found.is_null() {
                    ambiguous = 1 as ::core::ffi::c_int;
                }
                found = entry;
                if strcmp((*entry).name, name) == 0 as ::core::ffi::c_int {
                    break;
                }
            }
            loop_0 = loop_0.offset(1);
        }
    }
    if ambiguous != 0 {
        *(&raw mut s as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
        loop_0 = &raw mut cmd_table as *mut *const cmd_entry;
        while !(*loop_0).is_null() {
            entry = *loop_0;
            if !(strncmp((*entry).name, name, strlen(name)) != 0 as ::core::ffi::c_int) {
                if strlcat(
                    &raw mut s as *mut ::core::ffi::c_char,
                    (*entry).name,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as size_t,
                ) as usize
                    >= ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as usize
                {
                    break;
                }
                if strlcat(
                    &raw mut s as *mut ::core::ffi::c_char,
                    b", \0" as *const u8 as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as size_t,
                ) as usize
                    >= ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as usize
                {
                    break;
                }
            }
            loop_0 = loop_0.offset(1);
        }
        s[strlen(&raw mut s as *mut ::core::ffi::c_char).wrapping_sub(2 as size_t) as usize] =
            '\0' as i32 as ::core::ffi::c_char;
        xasprintf(
            cause,
            b"ambiguous command: %s, could be: %s\0" as *const u8 as *const ::core::ffi::c_char,
            name,
            &raw mut s as *mut ::core::ffi::c_char,
        );
        return ::core::ptr::null::<cmd_entry>();
    } else {
        if found.is_null() {
            xasprintf(
                cause,
                b"unknown command: %s\0" as *const u8 as *const ::core::ffi::c_char,
                name,
            );
            return ::core::ptr::null::<cmd_entry>();
        }
        return found;
    };
}
#[no_mangle]
pub unsafe extern "C" fn cmd_parse(
    mut values: *mut args_value,
    mut count: u_int,
    mut file: *const ::core::ffi::c_char,
    mut line: u_int,
    mut parse_flags: ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut cmd {
    let mut entry: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
    let mut cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut args: *mut args = ::core::ptr::null_mut::<args>();
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if count == 0 as u_int
        || (*values.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        xasprintf(
            cause,
            b"no command\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null_mut::<cmd>();
    }
    entry = cmd_find(
        (*values.offset(0 as ::core::ffi::c_int as isize))
            .c2rust_unnamed
            .string,
        cause,
    );
    if entry.is_null() {
        return ::core::ptr::null_mut::<cmd>();
    }
    args = args_parse(&raw const (*entry).args, values, count, &raw mut error);
    if args.is_null() && error.is_null() {
        xasprintf(
            cause,
            b"usage: %s %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*entry).name,
            (*entry).usage,
        );
        return ::core::ptr::null_mut::<cmd>();
    }
    if args.is_null() {
        xasprintf(
            cause,
            b"command %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*entry).name,
            error,
        );
        free(error as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<cmd>();
    }
    cmd = xcalloc(1 as size_t, ::core::mem::size_of::<cmd>() as size_t) as *mut cmd;
    (*cmd).entry = entry;
    (*cmd).args = args;
    (*cmd).parse_flags = parse_flags;
    if !file.is_null() {
        (*cmd).file = xstrdup(file);
    }
    (*cmd).line = line;
    return cmd;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_free(mut cmd: *mut cmd) {
    free((*cmd).file as *mut ::core::ffi::c_void);
    args_free((*cmd).args);
    free(cmd as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_copy(
    mut cmd: *mut cmd,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) -> *mut cmd {
    let mut new_cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    new_cmd = xcalloc(1 as size_t, ::core::mem::size_of::<cmd>() as size_t) as *mut cmd;
    (*new_cmd).entry = (*cmd).entry;
    (*new_cmd).args = args_copy((*cmd).args, argc, argv);
    if !(*cmd).file.is_null() {
        (*new_cmd).file = xstrdup((*cmd).file);
    }
    (*new_cmd).line = (*cmd).line;
    return new_cmd;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_print(mut cmd: *mut cmd) -> *mut ::core::ffi::c_char {
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    s = args_print((*cmd).args);
    if *s as ::core::ffi::c_int != '\0' as i32 {
        xasprintf(
            &raw mut out,
            b"%s %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*cmd).entry).name,
            s,
        );
    } else {
        out = xstrdup((*(*cmd).entry).name);
    }
    free(s as *mut ::core::ffi::c_void);
    return out;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_new() -> *mut cmd_list {
    let mut cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    cmdlist = xcalloc(1 as size_t, ::core::mem::size_of::<cmd_list>() as size_t) as *mut cmd_list;
    (*cmdlist).references = 1 as ::core::ffi::c_int;
    let fresh6 = cmd_list_next_group;
    cmd_list_next_group = cmd_list_next_group.wrapping_add(1);
    (*cmdlist).group = fresh6;
    (*cmdlist).list = xcalloc(1 as size_t, ::core::mem::size_of::<cmds>() as size_t) as *mut cmds;
    (*(*cmdlist).list).tqh_first = ::core::ptr::null_mut::<cmd>();
    (*(*cmdlist).list).tqh_last = &raw mut (*(*cmdlist).list).tqh_first;
    return cmdlist;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_append(mut cmdlist: *mut cmd_list, mut cmd: *mut cmd) {
    (*cmd).group = (*cmdlist).group;
    (*cmd).qentry.tqe_next = ::core::ptr::null_mut::<cmd>();
    (*cmd).qentry.tqe_prev = (*(*cmdlist).list).tqh_last;
    *(*(*cmdlist).list).tqh_last = cmd;
    (*(*cmdlist).list).tqh_last = &raw mut (*cmd).qentry.tqe_next;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_append_all(mut cmdlist: *mut cmd_list, mut from: *mut cmd_list) {
    let mut cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    cmd = (*(*from).list).tqh_first;
    while !cmd.is_null() {
        (*cmd).group = (*cmdlist).group;
        cmd = (*cmd).qentry.tqe_next;
    }
    if !(*(*from).list).tqh_first.is_null() {
        *(*(*cmdlist).list).tqh_last = (*(*from).list).tqh_first;
        (*(*(*from).list).tqh_first).qentry.tqe_prev = (*(*cmdlist).list).tqh_last;
        (*(*cmdlist).list).tqh_last = (*(*from).list).tqh_last;
        (*(*from).list).tqh_first = ::core::ptr::null_mut::<cmd>();
        (*(*from).list).tqh_last = &raw mut (*(*from).list).tqh_first;
    }
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_move(mut cmdlist: *mut cmd_list, mut from: *mut cmd_list) {
    if !(*(*from).list).tqh_first.is_null() {
        *(*(*cmdlist).list).tqh_last = (*(*from).list).tqh_first;
        (*(*(*from).list).tqh_first).qentry.tqe_prev = (*(*cmdlist).list).tqh_last;
        (*(*cmdlist).list).tqh_last = (*(*from).list).tqh_last;
        (*(*from).list).tqh_first = ::core::ptr::null_mut::<cmd>();
        (*(*from).list).tqh_last = &raw mut (*(*from).list).tqh_first;
    }
    let fresh8 = cmd_list_next_group;
    cmd_list_next_group = cmd_list_next_group.wrapping_add(1);
    (*cmdlist).group = fresh8;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_free(mut cmdlist: *mut cmd_list) {
    let mut cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut cmd1: *mut cmd = ::core::ptr::null_mut::<cmd>();
    (*cmdlist).references -= 1;
    if (*cmdlist).references != 0 as ::core::ffi::c_int {
        return;
    }
    cmd = (*(*cmdlist).list).tqh_first;
    while !cmd.is_null() && {
        cmd1 = (*cmd).qentry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(*cmd).qentry.tqe_next.is_null() {
            (*(*cmd).qentry.tqe_next).qentry.tqe_prev = (*cmd).qentry.tqe_prev;
        } else {
            (*(*cmdlist).list).tqh_last = (*cmd).qentry.tqe_prev;
        }
        *(*cmd).qentry.tqe_prev = (*cmd).qentry.tqe_next;
        cmd_free(cmd);
        cmd = cmd1;
    }
    free((*cmdlist).list as *mut ::core::ffi::c_void);
    free(cmdlist as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_copy(
    mut cmdlist: *const cmd_list,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) -> *mut cmd_list {
    let mut cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut new_cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    let mut new_cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut group: u_int = (*cmdlist).group;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    s = cmd_list_print(cmdlist, 0 as ::core::ffi::c_int);
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_list_copy\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    free(s as *mut ::core::ffi::c_void);
    new_cmdlist = cmd_list_new();
    cmd = (*(*cmdlist).list).tqh_first;
    while !cmd.is_null() {
        if (*cmd).group != group {
            let fresh7 = cmd_list_next_group;
            cmd_list_next_group = cmd_list_next_group.wrapping_add(1);
            (*new_cmdlist).group = fresh7;
            group = (*cmd).group;
        }
        new_cmd = cmd_copy(cmd, argc, argv);
        cmd_list_append(new_cmdlist, new_cmd);
        cmd = (*cmd).qentry.tqe_next;
    }
    s = cmd_list_print(new_cmdlist, 0 as ::core::ffi::c_int);
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_list_copy\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    free(s as *mut ::core::ffi::c_void);
    return new_cmdlist;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_print(
    mut cmdlist: *const cmd_list,
    mut flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut next: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut this: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut separator: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut escaped: ::core::ffi::c_int = flags & CMD_LIST_PRINT_ESCAPED;
    let mut no_groups: ::core::ffi::c_int = flags & CMD_LIST_PRINT_NO_GROUPS;
    let mut single_separator: *const ::core::ffi::c_char = if escaped != 0 {
        b" \\; \0" as *const u8 as *const ::core::ffi::c_char
    } else {
        b" ; \0" as *const u8 as *const ::core::ffi::c_char
    };
    let mut double_separator: *const ::core::ffi::c_char = if escaped != 0 {
        b" \\;\\; \0" as *const u8 as *const ::core::ffi::c_char
    } else {
        b" ;; \0" as *const u8 as *const ::core::ffi::c_char
    };
    len = 1 as size_t;
    buf = xcalloc(1 as size_t, len) as *mut ::core::ffi::c_char;
    cmd = (*(*cmdlist).list).tqh_first;
    while !cmd.is_null() {
        this = cmd_print(cmd);
        len = len.wrapping_add(strlen(this).wrapping_add(6 as size_t));
        buf = xrealloc(buf as *mut ::core::ffi::c_void, len) as *mut ::core::ffi::c_char;
        strlcat(buf, this, len);
        next = (*cmd).qentry.tqe_next;
        if !next.is_null() {
            if no_groups == 0 && (*cmd).group != (*next).group {
                separator = double_separator;
            } else {
                separator = single_separator;
            }
            strlcat(buf, separator, len);
        }
        free(this as *mut ::core::ffi::c_void);
        cmd = (*cmd).qentry.tqe_next;
    }
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_first(mut cmdlist: *mut cmd_list) -> *mut cmd {
    return (*(*cmdlist).list).tqh_first;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_next(mut cmd: *mut cmd) -> *mut cmd {
    return (*cmd).qentry.tqe_next;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_all_have(
    mut cmdlist: *mut cmd_list,
    mut flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    cmd = (*(*cmdlist).list).tqh_first;
    while !cmd.is_null() {
        if !(*(*cmd).entry).flags & flag != 0 {
            return 0 as ::core::ffi::c_int;
        }
        cmd = (*cmd).qentry.tqe_next;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_any_have(
    mut cmdlist: *mut cmd_list,
    mut flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    cmd = (*(*cmdlist).list).tqh_first;
    while !cmd.is_null() {
        if (*(*cmd).entry).flags & flag != 0 {
            return 1 as ::core::ffi::c_int;
        }
        cmd = (*cmd).qentry.tqe_next;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_mouse_at(
    mut wp: *mut window_pane,
    mut m: *mut mouse_event,
    mut xp: *mut u_int,
    mut yp: *mut u_int,
    mut last: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if last != 0 {
        x = (*m).lx.wrapping_add((*m).ox);
        y = (*m).ly.wrapping_add((*m).oy);
    } else {
        x = (*m).x.wrapping_add((*m).ox);
        y = (*m).y.wrapping_add((*m).oy);
    }
    log_debug(
        b"%s: x=%u, y=%u%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_mouse_at\0" as *const u8 as *const ::core::ffi::c_char,
        x,
        y,
        if last != 0 {
            b" (last)\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
    );
    if (*m).statusat == 0 as ::core::ffi::c_int && y >= (*m).statuslines {
        y = y.wrapping_sub((*m).statuslines);
    }
    if (x as ::core::ffi::c_int) < (*wp).xoff
        || x as ::core::ffi::c_int >= (*wp).xoff + (*wp).sx as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    if (y as ::core::ffi::c_int) < (*wp).yoff
        || y as ::core::ffi::c_int >= (*wp).yoff + (*wp).sy as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    if !xp.is_null() {
        *xp = x.wrapping_sub((*wp).xoff as u_int);
    }
    if !yp.is_null() {
        *yp = y.wrapping_sub((*wp).yoff as u_int);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_mouse_window(
    mut m: *mut mouse_event,
    mut sp: *mut *mut session,
) -> *mut winlink {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if (*m).valid == 0 {
        return ::core::ptr::null_mut::<winlink>();
    }
    if (*m).s == -(1 as ::core::ffi::c_int) || {
        s = session_find_by_id((*m).s as u_int);
        s.is_null()
    } {
        return ::core::ptr::null_mut::<winlink>();
    }
    if (*m).w == -(1 as ::core::ffi::c_int) {
        wl = (*s).curw;
    } else {
        w = window_find_by_id((*m).w as u_int);
        if w.is_null() {
            return ::core::ptr::null_mut::<winlink>();
        }
        wl = winlink_find_by_window(&raw mut (*s).windows, w);
    }
    if !sp.is_null() {
        *sp = s;
    }
    return wl;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_mouse_pane(
    mut m: *mut mouse_event,
    mut sp: *mut *mut session,
    mut wlp: *mut *mut winlink,
) -> *mut window_pane {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    wl = cmd_mouse_window(m, sp);
    if wl.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    if (*m).wp == -(1 as ::core::ffi::c_int) {
        wp = (*(*wl).window).active;
    } else {
        wp = window_pane_find_by_id((*m).wp as u_int);
        if wp.is_null() {
            return ::core::ptr::null_mut::<window_pane>();
        }
        if window_has_pane((*wl).window, wp) == 0 {
            return ::core::ptr::null_mut::<window_pane>();
        }
    }
    if !(*(*wl).window).modal.is_null() && wp != (*(*wl).window).modal {
        return ::core::ptr::null_mut::<window_pane>();
    }
    if !wlp.is_null() {
        *wlp = wl;
    }
    return wp;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_template_replace(
    mut template: *const ::core::ffi::c_char,
    mut s: *const ::core::ffi::c_char,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut ch: ::core::ffi::c_char = 0;
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ptr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let dquote: [::core::ffi::c_char; 6] =
        ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"\"\\$;~\0");
    let mut replaced: ::core::ffi::c_int = 0;
    let mut len: size_t = 0;
    let mut slen: size_t = 0;
    let mut quote: C2RustUnnamed_38 = NQ;
    if strchr(template, '%' as i32).is_null() {
        return xstrdup(template);
    }
    buf = xmalloc(1 as size_t) as *mut ::core::ffi::c_char;
    *buf = '\0' as i32 as ::core::ffi::c_char;
    len = 0 as size_t;
    replaced = 0 as ::core::ffi::c_int;
    ptr = template;
    let mut current_block_39: u64;
    while *ptr as ::core::ffi::c_int != '\0' as i32 {
        let fresh9 = ptr;
        ptr = ptr.offset(1);
        ch = *fresh9;
        match ch as ::core::ffi::c_int {
            37 => {
                if *ptr as ::core::ffi::c_int >= '1' as i32
                    && *ptr as ::core::ffi::c_int <= '9' as i32
                    && *ptr as ::core::ffi::c_int - '0' as i32 == idx
                {
                    ptr = ptr.offset(1);
                    quote = NQ;
                    if *ptr as ::core::ffi::c_int == '%' as i32 {
                        quote = DQ;
                        ptr = ptr.offset(1);
                    }
                    current_block_39 = 26972500619410423;
                } else if *ptr as ::core::ffi::c_int != '%' as i32 || replaced != 0 {
                    current_block_39 = 17500079516916021833;
                } else {
                    replaced = 1 as ::core::ffi::c_int;
                    ptr = ptr.offset(1);
                    quote = SQ;
                    if *ptr as ::core::ffi::c_int == '%' as i32 {
                        quote = DQ;
                        ptr = ptr.offset(1);
                    }
                    current_block_39 = 26972500619410423;
                }
                match current_block_39 {
                    17500079516916021833 => {}
                    _ => {
                        slen = strlen(s);
                        if slen >= (SIZE_MAX as size_t).wrapping_div(4 as size_t)
                            || len
                                > (SIZE_MAX as size_t)
                                    .wrapping_sub(slen.wrapping_mul(4 as size_t))
                                    .wrapping_sub(1 as size_t)
                        {
                            fatalx(
                                b"argument too long\0" as *const u8 as *const ::core::ffi::c_char,
                            );
                        }
                        buf = xrealloc(
                            buf as *mut ::core::ffi::c_void,
                            len.wrapping_add(slen.wrapping_mul(4 as size_t))
                                .wrapping_add(1 as size_t),
                        ) as *mut ::core::ffi::c_char;
                        cp = s;
                        while *cp as ::core::ffi::c_int != '\0' as i32 {
                            if quote as ::core::ffi::c_uint
                                == SQ as ::core::ffi::c_int as ::core::ffi::c_uint
                                && *cp as ::core::ffi::c_int == '\'' as i32
                            {
                                let fresh10 = len;
                                len = len.wrapping_add(1);
                                *buf.offset(fresh10 as isize) = '\'' as i32 as ::core::ffi::c_char;
                                let fresh11 = len;
                                len = len.wrapping_add(1);
                                *buf.offset(fresh11 as isize) = '\\' as i32 as ::core::ffi::c_char;
                                let fresh12 = len;
                                len = len.wrapping_add(1);
                                *buf.offset(fresh12 as isize) = '\'' as i32 as ::core::ffi::c_char;
                                let fresh13 = len;
                                len = len.wrapping_add(1);
                                *buf.offset(fresh13 as isize) = '\'' as i32 as ::core::ffi::c_char;
                            } else {
                                if quote as ::core::ffi::c_uint
                                    == DQ as ::core::ffi::c_int as ::core::ffi::c_uint
                                    && !strchr(
                                        &raw const dquote as *const ::core::ffi::c_char,
                                        *cp as ::core::ffi::c_int,
                                    )
                                    .is_null()
                                {
                                    let fresh14 = len;
                                    len = len.wrapping_add(1);
                                    *buf.offset(fresh14 as isize) =
                                        '\\' as i32 as ::core::ffi::c_char;
                                }
                                let fresh15 = len;
                                len = len.wrapping_add(1);
                                *buf.offset(fresh15 as isize) = *cp;
                            }
                            cp = cp.offset(1);
                        }
                        *buf.offset(len as isize) = '\0' as i32 as ::core::ffi::c_char;
                        continue;
                    }
                }
            }
            _ => {}
        }
        if len > (SIZE_MAX as size_t).wrapping_sub(2 as size_t) {
            fatalx(b"argument too long\0" as *const u8 as *const ::core::ffi::c_char);
        }
        buf = xrealloc(
            buf as *mut ::core::ffi::c_void,
            len.wrapping_add(2 as size_t),
        ) as *mut ::core::ffi::c_char;
        let fresh16 = len;
        len = len.wrapping_add(1);
        *buf.offset(fresh16 as isize) = ch;
        *buf.offset(len as isize) = '\0' as i32 as ::core::ffi::c_char;
    }
    log_debug(
        b"%s: %s -> %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_template_replace\0" as *const u8 as *const ::core::ffi::c_char,
        template,
        buf,
    );
    return buf;
}

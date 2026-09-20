pub use crate::src::shared::monitor::{
    MONITOR_ALL_PANES, MONITOR_ALL_WINDOWS, MONITOR_NOTIFY_INITIAL, MONITOR_NOTIFY_TRUE,
    MONITOR_PANE, MONITOR_SESSION, MONITOR_WINDOW, monitor_type,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
pub use crate::src::shared::event::{EV_TIMEOUT};
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
    fn sscanf(
        __s: *const ::core::ffi::c_char,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn event_add(ev: *mut event, timeout: *const timeval) -> ::core::ffi::c_int;
    fn event_del(_: *mut event) -> ::core::ffi::c_int;
    fn event_pending(
        ev: *const event,
        events: ::core::ffi::c_short,
        tv: *mut timeval,
    ) -> ::core::ffi::c_int;
    fn event_initialized(ev: *const event) -> ::core::ffi::c_int;
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
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn format_true(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn format_create(
        _: *mut client,
        _: *mut cmdq_item,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut format_tree;
    fn format_free(_: *mut format_tree);
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
    static mut current_time: time_t;
    fn winlinks_RB_MINMAX(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn winlinks_RB_NEXT(_: *mut winlink) -> *mut winlink;
    fn window_find_by_id(_: u_int) -> *mut window;
    fn window_pane_find_by_id(_: u_int) -> *mut window_pane;
    static mut sessions: sessions;
    fn sessions_RB_MINMAX(_: *mut sessions, _: ::core::ffi::c_int) -> *mut session;
    fn session_find_by_id(_: u_int) -> *mut session;
    fn session_add_ref(_: *mut session, _: *const ::core::ffi::c_char);
    fn session_remove_ref(_: *mut session, _: *const ::core::ffi::c_char);
    fn log_debug(_: *const ::core::ffi::c_char, ...);
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
pub struct sessions {
    pub rbh_root: *mut session,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_change {
    pub name: *const ::core::ffi::c_char,
    pub value: *const ::core::ffi::c_char,
    pub last: *const ::core::ffi::c_char,
    pub c: *mut client,
    pub s: *mut session,
    pub wl: *mut winlink,
    pub wp: *mut window_pane,
}
pub type monitor_cb =
    Option<unsafe extern "C" fn(*mut monitor_change, *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_set {
    pub client: *mut client,
    pub session: *mut session,
    pub cb: monitor_cb,
    pub data: *mut ::core::ffi::c_void,
    pub items: monitor_items,
    pub timer: event,
    pub generation: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_items {
    pub rbh_root: *mut monitor_item,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_item {
    pub name: *mut ::core::ffi::c_char,
    pub format: *mut ::core::ffi::c_char,
    pub type_0: monitor_type,
    pub id: u_int,
    pub flags: ::core::ffi::c_int,
    pub last: *mut ::core::ffi::c_char,
    pub panes: monitor_panes,
    pub windows: monitor_windows,
    pub fire_count: u_int,
    pub fire_time: time_t,
    pub entry: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub rbe_left: *mut monitor_item,
    pub rbe_right: *mut monitor_item,
    pub rbe_parent: *mut monitor_item,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_windows {
    pub rbh_root: *mut monitor_window,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_window {
    pub window: u_int,
    pub idx: u_int,
    pub last: *mut ::core::ffi::c_char,
    pub generation: u_int,
    pub entry: C2RustUnnamed_36,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_36 {
    pub rbe_left: *mut monitor_window,
    pub rbe_right: *mut monitor_window,
    pub rbe_parent: *mut monitor_window,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_panes {
    pub rbh_root: *mut monitor_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct monitor_pane {
    pub pane: u_int,
    pub idx: u_int,
    pub last: *mut ::core::ffi::c_char,
    pub generation: u_int,
    pub entry: C2RustUnnamed_37,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
    pub rbe_left: *mut monitor_pane,
    pub rbe_right: *mut monitor_pane,
    pub rbe_parent: *mut monitor_pane,
    pub rbe_color: ::core::ffi::c_int,
}
pub const FORMAT_NOJOBS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
unsafe extern "C" fn monitor_get_session(mut ms: *mut monitor_set) -> *mut session {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    if !(*ms).client.is_null() {
        return (*(*ms).client).session;
    }
    s = (*ms).session;
    if s.is_null() {
        return sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    }
    if session_find_by_id((*s).id) != s {
        return ::core::ptr::null_mut::<session>();
    }
    return s;
}
unsafe extern "C" fn monitor_create_formats(
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) -> *mut format_tree {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    ft = format_create(
        ::core::ptr::null_mut::<client>(),
        ::core::ptr::null_mut::<cmdq_item>(),
        0 as ::core::ffi::c_int,
        FORMAT_NOJOBS,
    );
    format_defaults(ft, c, s, wl, wp);
    return ft;
}
unsafe extern "C" fn monitor_item_cmp(
    mut m1: *mut monitor_item,
    mut m2: *mut monitor_item,
) -> ::core::ffi::c_int {
    return strcmp((*m1).name, (*m2).name);
}
unsafe extern "C" fn monitor_items_RB_MINMAX(
    mut head: *mut monitor_items,
    mut val: ::core::ffi::c_int,
) -> *mut monitor_item {
    let mut tmp: *mut monitor_item = (*head).rbh_root;
    let mut parent: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    while !tmp.is_null() {
        parent = tmp;
        if val < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else {
            tmp = (*tmp).entry.rbe_right;
        }
    }
    return parent;
}
unsafe extern "C" fn monitor_items_RB_NEXT(mut elm: *mut monitor_item) -> *mut monitor_item {
    if !(*elm).entry.rbe_right.is_null() {
        elm = (*elm).entry.rbe_right;
        while !(*elm).entry.rbe_left.is_null() {
            elm = (*elm).entry.rbe_left;
        }
    } else if !(*elm).entry.rbe_parent.is_null() && elm == (*(*elm).entry.rbe_parent).entry.rbe_left
    {
        elm = (*elm).entry.rbe_parent;
    } else {
        while !(*elm).entry.rbe_parent.is_null()
            && elm == (*(*elm).entry.rbe_parent).entry.rbe_right
        {
            elm = (*elm).entry.rbe_parent;
        }
        elm = (*elm).entry.rbe_parent;
    }
    return elm;
}
unsafe extern "C" fn monitor_items_RB_REMOVE_COLOR(
    mut head: *mut monitor_items,
    mut parent: *mut monitor_item,
    mut elm: *mut monitor_item,
) {
    let mut tmp: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    while (elm.is_null() || (*elm).entry.rbe_color == RB_BLACK) && elm != (*head).rbh_root {
        if (*parent).entry.rbe_left == elm {
            tmp = (*parent).entry.rbe_right;
            if (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_RED;
                tmp = (*parent).entry.rbe_right;
                (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*parent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                tmp = (*parent).entry.rbe_right;
            }
            if ((*tmp).entry.rbe_left.is_null()
                || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK)
                && ((*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK)
            {
                (*tmp).entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).entry.rbe_parent;
            } else {
                if (*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK
                {
                    let mut oleft: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
                    oleft = (*tmp).entry.rbe_left;
                    if !oleft.is_null() {
                        (*oleft).entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).entry.rbe_color = RB_RED;
                    oleft = (*tmp).entry.rbe_left;
                    (*tmp).entry.rbe_left = (*oleft).entry.rbe_right;
                    if !(*tmp).entry.rbe_left.is_null() {
                        (*(*oleft).entry.rbe_right).entry.rbe_parent = tmp;
                    }
                    (*oleft).entry.rbe_parent = (*tmp).entry.rbe_parent;
                    if !(*oleft).entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).entry.rbe_parent).entry.rbe_left {
                            (*(*tmp).entry.rbe_parent).entry.rbe_left = oleft;
                        } else {
                            (*(*tmp).entry.rbe_parent).entry.rbe_right = oleft;
                        }
                    } else {
                        (*head).rbh_root = oleft;
                    }
                    (*oleft).entry.rbe_right = tmp;
                    (*tmp).entry.rbe_parent = oleft;
                    !(*oleft).entry.rbe_parent.is_null();
                    tmp = (*parent).entry.rbe_right;
                }
                (*tmp).entry.rbe_color = (*parent).entry.rbe_color;
                (*parent).entry.rbe_color = RB_BLACK;
                if !(*tmp).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).entry.rbe_right;
                (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*parent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        } else {
            tmp = (*parent).entry.rbe_left;
            if (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_RED;
                tmp = (*parent).entry.rbe_left;
                (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*parent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                tmp = (*parent).entry.rbe_left;
            }
            if ((*tmp).entry.rbe_left.is_null()
                || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK)
                && ((*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK)
            {
                (*tmp).entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).entry.rbe_parent;
            } else {
                if (*tmp).entry.rbe_left.is_null()
                    || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK
                {
                    let mut oright: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
                    oright = (*tmp).entry.rbe_right;
                    if !oright.is_null() {
                        (*oright).entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).entry.rbe_color = RB_RED;
                    oright = (*tmp).entry.rbe_right;
                    (*tmp).entry.rbe_right = (*oright).entry.rbe_left;
                    if !(*tmp).entry.rbe_right.is_null() {
                        (*(*oright).entry.rbe_left).entry.rbe_parent = tmp;
                    }
                    (*oright).entry.rbe_parent = (*tmp).entry.rbe_parent;
                    if !(*oright).entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).entry.rbe_parent).entry.rbe_left {
                            (*(*tmp).entry.rbe_parent).entry.rbe_left = oright;
                        } else {
                            (*(*tmp).entry.rbe_parent).entry.rbe_right = oright;
                        }
                    } else {
                        (*head).rbh_root = oright;
                    }
                    (*oright).entry.rbe_left = tmp;
                    (*tmp).entry.rbe_parent = oright;
                    !(*oright).entry.rbe_parent.is_null();
                    tmp = (*parent).entry.rbe_left;
                }
                (*tmp).entry.rbe_color = (*parent).entry.rbe_color;
                (*parent).entry.rbe_color = RB_BLACK;
                if !(*tmp).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).entry.rbe_left;
                (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*parent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        }
    }
    if !elm.is_null() {
        (*elm).entry.rbe_color = RB_BLACK;
    }
}
unsafe extern "C" fn monitor_items_RB_INSERT_COLOR(
    mut head: *mut monitor_items,
    mut elm: *mut monitor_item,
) {
    let mut parent: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut gparent: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut tmp: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    loop {
        parent = (*elm).entry.rbe_parent;
        if !(!parent.is_null() && (*parent).entry.rbe_color == RB_RED) {
            break;
        }
        gparent = (*parent).entry.rbe_parent;
        if parent == (*gparent).entry.rbe_left {
            tmp = (*gparent).entry.rbe_right;
            if !tmp.is_null() && (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).entry.rbe_right == elm {
                    tmp = (*parent).entry.rbe_right;
                    (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                    if !(*parent).entry.rbe_right.is_null() {
                        (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                    }
                    (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                    if !(*tmp).entry.rbe_parent.is_null() {
                        if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                            (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                        } else {
                            (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).entry.rbe_left = parent;
                    (*parent).entry.rbe_parent = tmp;
                    !(*tmp).entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                tmp = (*gparent).entry.rbe_left;
                (*gparent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*gparent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = gparent;
                }
                (*tmp).entry.rbe_parent = (*gparent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).entry.rbe_parent).entry.rbe_left {
                        (*(*gparent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = gparent;
                (*gparent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
            }
        } else {
            tmp = (*gparent).entry.rbe_left;
            if !tmp.is_null() && (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).entry.rbe_left == elm {
                    tmp = (*parent).entry.rbe_left;
                    (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                    if !(*parent).entry.rbe_left.is_null() {
                        (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                    }
                    (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                    if !(*tmp).entry.rbe_parent.is_null() {
                        if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                            (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                        } else {
                            (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).entry.rbe_right = parent;
                    (*parent).entry.rbe_parent = tmp;
                    !(*tmp).entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                tmp = (*gparent).entry.rbe_right;
                (*gparent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*gparent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = gparent;
                }
                (*tmp).entry.rbe_parent = (*gparent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).entry.rbe_parent).entry.rbe_left {
                        (*(*gparent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = gparent;
                (*gparent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
            }
        }
    }
    (*(*head).rbh_root).entry.rbe_color = RB_BLACK;
}
unsafe extern "C" fn monitor_items_RB_INSERT(
    mut head: *mut monitor_items,
    mut elm: *mut monitor_item,
) -> *mut monitor_item {
    let mut tmp: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut parent: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = monitor_item_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<monitor_item>();
    (*elm).entry.rbe_left = (*elm).entry.rbe_right;
    (*elm).entry.rbe_color = RB_RED;
    if !parent.is_null() {
        if comp < 0 as ::core::ffi::c_int {
            (*parent).entry.rbe_left = elm;
        } else {
            (*parent).entry.rbe_right = elm;
        }
    } else {
        (*head).rbh_root = elm;
    }
    monitor_items_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<monitor_item>();
}
unsafe extern "C" fn monitor_items_RB_REMOVE(
    mut head: *mut monitor_items,
    mut elm: *mut monitor_item,
) -> *mut monitor_item {
    let mut current_block: u64;
    let mut child: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut parent: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut old: *mut monitor_item = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
        elm = (*elm).entry.rbe_right;
        loop {
            left = (*elm).entry.rbe_left;
            if left.is_null() {
                break;
            }
            elm = left;
        }
        child = (*elm).entry.rbe_right;
        parent = (*elm).entry.rbe_parent;
        color = (*elm).entry.rbe_color;
        if !child.is_null() {
            (*child).entry.rbe_parent = parent;
        }
        if !parent.is_null() {
            if (*parent).entry.rbe_left == elm {
                (*parent).entry.rbe_left = child;
            } else {
                (*parent).entry.rbe_right = child;
            }
        } else {
            (*head).rbh_root = child;
        }
        if (*elm).entry.rbe_parent == old {
            parent = elm;
        }
        (*elm).entry = (*old).entry;
        if !(*old).entry.rbe_parent.is_null() {
            if (*(*old).entry.rbe_parent).entry.rbe_left == old {
                (*(*old).entry.rbe_parent).entry.rbe_left = elm;
            } else {
                (*(*old).entry.rbe_parent).entry.rbe_right = elm;
            }
        } else {
            (*head).rbh_root = elm;
        }
        (*(*old).entry.rbe_left).entry.rbe_parent = elm;
        if !(*old).entry.rbe_right.is_null() {
            (*(*old).entry.rbe_right).entry.rbe_parent = elm;
        }
        if !parent.is_null() {
            left = parent;
            loop {
                left = (*left).entry.rbe_parent;
                if left.is_null() {
                    break;
                }
            }
        }
        current_block = 12669146338688518024;
    }
    match current_block {
        7245201122033322888 => {
            parent = (*elm).entry.rbe_parent;
            color = (*elm).entry.rbe_color;
            if !child.is_null() {
                (*child).entry.rbe_parent = parent;
            }
            if !parent.is_null() {
                if (*parent).entry.rbe_left == elm {
                    (*parent).entry.rbe_left = child;
                } else {
                    (*parent).entry.rbe_right = child;
                }
            } else {
                (*head).rbh_root = child;
            }
        }
        _ => {}
    }
    if color == RB_BLACK {
        monitor_items_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn monitor_items_RB_FIND(
    mut head: *mut monitor_items,
    mut elm: *mut monitor_item,
) -> *mut monitor_item {
    let mut tmp: *mut monitor_item = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = monitor_item_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<monitor_item>();
}
unsafe extern "C" fn monitor_pane_cmp(
    mut mp1: *mut monitor_pane,
    mut mp2: *mut monitor_pane,
) -> ::core::ffi::c_int {
    if (*mp1).pane < (*mp2).pane {
        return -(1 as ::core::ffi::c_int);
    }
    if (*mp1).pane > (*mp2).pane {
        return 1 as ::core::ffi::c_int;
    }
    if (*mp1).idx < (*mp2).idx {
        return -(1 as ::core::ffi::c_int);
    }
    if (*mp1).idx > (*mp2).idx {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn monitor_panes_RB_REMOVE_COLOR(
    mut head: *mut monitor_panes,
    mut parent: *mut monitor_pane,
    mut elm: *mut monitor_pane,
) {
    let mut tmp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    while (elm.is_null() || (*elm).entry.rbe_color == RB_BLACK) && elm != (*head).rbh_root {
        if (*parent).entry.rbe_left == elm {
            tmp = (*parent).entry.rbe_right;
            if (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_RED;
                tmp = (*parent).entry.rbe_right;
                (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*parent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                tmp = (*parent).entry.rbe_right;
            }
            if ((*tmp).entry.rbe_left.is_null()
                || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK)
                && ((*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK)
            {
                (*tmp).entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).entry.rbe_parent;
            } else {
                if (*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK
                {
                    let mut oleft: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
                    oleft = (*tmp).entry.rbe_left;
                    if !oleft.is_null() {
                        (*oleft).entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).entry.rbe_color = RB_RED;
                    oleft = (*tmp).entry.rbe_left;
                    (*tmp).entry.rbe_left = (*oleft).entry.rbe_right;
                    if !(*tmp).entry.rbe_left.is_null() {
                        (*(*oleft).entry.rbe_right).entry.rbe_parent = tmp;
                    }
                    (*oleft).entry.rbe_parent = (*tmp).entry.rbe_parent;
                    if !(*oleft).entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).entry.rbe_parent).entry.rbe_left {
                            (*(*tmp).entry.rbe_parent).entry.rbe_left = oleft;
                        } else {
                            (*(*tmp).entry.rbe_parent).entry.rbe_right = oleft;
                        }
                    } else {
                        (*head).rbh_root = oleft;
                    }
                    (*oleft).entry.rbe_right = tmp;
                    (*tmp).entry.rbe_parent = oleft;
                    !(*oleft).entry.rbe_parent.is_null();
                    tmp = (*parent).entry.rbe_right;
                }
                (*tmp).entry.rbe_color = (*parent).entry.rbe_color;
                (*parent).entry.rbe_color = RB_BLACK;
                if !(*tmp).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).entry.rbe_right;
                (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*parent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        } else {
            tmp = (*parent).entry.rbe_left;
            if (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_RED;
                tmp = (*parent).entry.rbe_left;
                (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*parent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                tmp = (*parent).entry.rbe_left;
            }
            if ((*tmp).entry.rbe_left.is_null()
                || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK)
                && ((*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK)
            {
                (*tmp).entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).entry.rbe_parent;
            } else {
                if (*tmp).entry.rbe_left.is_null()
                    || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK
                {
                    let mut oright: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
                    oright = (*tmp).entry.rbe_right;
                    if !oright.is_null() {
                        (*oright).entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).entry.rbe_color = RB_RED;
                    oright = (*tmp).entry.rbe_right;
                    (*tmp).entry.rbe_right = (*oright).entry.rbe_left;
                    if !(*tmp).entry.rbe_right.is_null() {
                        (*(*oright).entry.rbe_left).entry.rbe_parent = tmp;
                    }
                    (*oright).entry.rbe_parent = (*tmp).entry.rbe_parent;
                    if !(*oright).entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).entry.rbe_parent).entry.rbe_left {
                            (*(*tmp).entry.rbe_parent).entry.rbe_left = oright;
                        } else {
                            (*(*tmp).entry.rbe_parent).entry.rbe_right = oright;
                        }
                    } else {
                        (*head).rbh_root = oright;
                    }
                    (*oright).entry.rbe_left = tmp;
                    (*tmp).entry.rbe_parent = oright;
                    !(*oright).entry.rbe_parent.is_null();
                    tmp = (*parent).entry.rbe_left;
                }
                (*tmp).entry.rbe_color = (*parent).entry.rbe_color;
                (*parent).entry.rbe_color = RB_BLACK;
                if !(*tmp).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).entry.rbe_left;
                (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*parent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        }
    }
    if !elm.is_null() {
        (*elm).entry.rbe_color = RB_BLACK;
    }
}
unsafe extern "C" fn monitor_panes_RB_REMOVE(
    mut head: *mut monitor_panes,
    mut elm: *mut monitor_pane,
) -> *mut monitor_pane {
    let mut current_block: u64;
    let mut child: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut parent: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut old: *mut monitor_pane = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
        elm = (*elm).entry.rbe_right;
        loop {
            left = (*elm).entry.rbe_left;
            if left.is_null() {
                break;
            }
            elm = left;
        }
        child = (*elm).entry.rbe_right;
        parent = (*elm).entry.rbe_parent;
        color = (*elm).entry.rbe_color;
        if !child.is_null() {
            (*child).entry.rbe_parent = parent;
        }
        if !parent.is_null() {
            if (*parent).entry.rbe_left == elm {
                (*parent).entry.rbe_left = child;
            } else {
                (*parent).entry.rbe_right = child;
            }
        } else {
            (*head).rbh_root = child;
        }
        if (*elm).entry.rbe_parent == old {
            parent = elm;
        }
        (*elm).entry = (*old).entry;
        if !(*old).entry.rbe_parent.is_null() {
            if (*(*old).entry.rbe_parent).entry.rbe_left == old {
                (*(*old).entry.rbe_parent).entry.rbe_left = elm;
            } else {
                (*(*old).entry.rbe_parent).entry.rbe_right = elm;
            }
        } else {
            (*head).rbh_root = elm;
        }
        (*(*old).entry.rbe_left).entry.rbe_parent = elm;
        if !(*old).entry.rbe_right.is_null() {
            (*(*old).entry.rbe_right).entry.rbe_parent = elm;
        }
        if !parent.is_null() {
            left = parent;
            loop {
                left = (*left).entry.rbe_parent;
                if left.is_null() {
                    break;
                }
            }
        }
        current_block = 11011803464182723597;
    }
    match current_block {
        7245201122033322888 => {
            parent = (*elm).entry.rbe_parent;
            color = (*elm).entry.rbe_color;
            if !child.is_null() {
                (*child).entry.rbe_parent = parent;
            }
            if !parent.is_null() {
                if (*parent).entry.rbe_left == elm {
                    (*parent).entry.rbe_left = child;
                } else {
                    (*parent).entry.rbe_right = child;
                }
            } else {
                (*head).rbh_root = child;
            }
        }
        _ => {}
    }
    if color == RB_BLACK {
        monitor_panes_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn monitor_panes_RB_NEXT(mut elm: *mut monitor_pane) -> *mut monitor_pane {
    if !(*elm).entry.rbe_right.is_null() {
        elm = (*elm).entry.rbe_right;
        while !(*elm).entry.rbe_left.is_null() {
            elm = (*elm).entry.rbe_left;
        }
    } else if !(*elm).entry.rbe_parent.is_null() && elm == (*(*elm).entry.rbe_parent).entry.rbe_left
    {
        elm = (*elm).entry.rbe_parent;
    } else {
        while !(*elm).entry.rbe_parent.is_null()
            && elm == (*(*elm).entry.rbe_parent).entry.rbe_right
        {
            elm = (*elm).entry.rbe_parent;
        }
        elm = (*elm).entry.rbe_parent;
    }
    return elm;
}
unsafe extern "C" fn monitor_panes_RB_MINMAX(
    mut head: *mut monitor_panes,
    mut val: ::core::ffi::c_int,
) -> *mut monitor_pane {
    let mut tmp: *mut monitor_pane = (*head).rbh_root;
    let mut parent: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    while !tmp.is_null() {
        parent = tmp;
        if val < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else {
            tmp = (*tmp).entry.rbe_right;
        }
    }
    return parent;
}
unsafe extern "C" fn monitor_panes_RB_FIND(
    mut head: *mut monitor_panes,
    mut elm: *mut monitor_pane,
) -> *mut monitor_pane {
    let mut tmp: *mut monitor_pane = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = monitor_pane_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<monitor_pane>();
}
unsafe extern "C" fn monitor_panes_RB_INSERT(
    mut head: *mut monitor_panes,
    mut elm: *mut monitor_pane,
) -> *mut monitor_pane {
    let mut tmp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut parent: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = monitor_pane_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<monitor_pane>();
    (*elm).entry.rbe_left = (*elm).entry.rbe_right;
    (*elm).entry.rbe_color = RB_RED;
    if !parent.is_null() {
        if comp < 0 as ::core::ffi::c_int {
            (*parent).entry.rbe_left = elm;
        } else {
            (*parent).entry.rbe_right = elm;
        }
    } else {
        (*head).rbh_root = elm;
    }
    monitor_panes_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<monitor_pane>();
}
unsafe extern "C" fn monitor_panes_RB_INSERT_COLOR(
    mut head: *mut monitor_panes,
    mut elm: *mut monitor_pane,
) {
    let mut parent: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut gparent: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut tmp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    loop {
        parent = (*elm).entry.rbe_parent;
        if !(!parent.is_null() && (*parent).entry.rbe_color == RB_RED) {
            break;
        }
        gparent = (*parent).entry.rbe_parent;
        if parent == (*gparent).entry.rbe_left {
            tmp = (*gparent).entry.rbe_right;
            if !tmp.is_null() && (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).entry.rbe_right == elm {
                    tmp = (*parent).entry.rbe_right;
                    (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                    if !(*parent).entry.rbe_right.is_null() {
                        (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                    }
                    (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                    if !(*tmp).entry.rbe_parent.is_null() {
                        if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                            (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                        } else {
                            (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).entry.rbe_left = parent;
                    (*parent).entry.rbe_parent = tmp;
                    !(*tmp).entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                tmp = (*gparent).entry.rbe_left;
                (*gparent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*gparent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = gparent;
                }
                (*tmp).entry.rbe_parent = (*gparent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).entry.rbe_parent).entry.rbe_left {
                        (*(*gparent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = gparent;
                (*gparent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
            }
        } else {
            tmp = (*gparent).entry.rbe_left;
            if !tmp.is_null() && (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).entry.rbe_left == elm {
                    tmp = (*parent).entry.rbe_left;
                    (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                    if !(*parent).entry.rbe_left.is_null() {
                        (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                    }
                    (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                    if !(*tmp).entry.rbe_parent.is_null() {
                        if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                            (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                        } else {
                            (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).entry.rbe_right = parent;
                    (*parent).entry.rbe_parent = tmp;
                    !(*tmp).entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                tmp = (*gparent).entry.rbe_right;
                (*gparent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*gparent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = gparent;
                }
                (*tmp).entry.rbe_parent = (*gparent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).entry.rbe_parent).entry.rbe_left {
                        (*(*gparent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = gparent;
                (*gparent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
            }
        }
    }
    (*(*head).rbh_root).entry.rbe_color = RB_BLACK;
}
unsafe extern "C" fn monitor_window_cmp(
    mut mw1: *mut monitor_window,
    mut mw2: *mut monitor_window,
) -> ::core::ffi::c_int {
    if (*mw1).window < (*mw2).window {
        return -(1 as ::core::ffi::c_int);
    }
    if (*mw1).window > (*mw2).window {
        return 1 as ::core::ffi::c_int;
    }
    if (*mw1).idx < (*mw2).idx {
        return -(1 as ::core::ffi::c_int);
    }
    if (*mw1).idx > (*mw2).idx {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn monitor_windows_RB_INSERT_COLOR(
    mut head: *mut monitor_windows,
    mut elm: *mut monitor_window,
) {
    let mut parent: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut gparent: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut tmp: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    loop {
        parent = (*elm).entry.rbe_parent;
        if !(!parent.is_null() && (*parent).entry.rbe_color == RB_RED) {
            break;
        }
        gparent = (*parent).entry.rbe_parent;
        if parent == (*gparent).entry.rbe_left {
            tmp = (*gparent).entry.rbe_right;
            if !tmp.is_null() && (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).entry.rbe_right == elm {
                    tmp = (*parent).entry.rbe_right;
                    (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                    if !(*parent).entry.rbe_right.is_null() {
                        (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                    }
                    (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                    if !(*tmp).entry.rbe_parent.is_null() {
                        if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                            (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                        } else {
                            (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).entry.rbe_left = parent;
                    (*parent).entry.rbe_parent = tmp;
                    !(*tmp).entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                tmp = (*gparent).entry.rbe_left;
                (*gparent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*gparent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = gparent;
                }
                (*tmp).entry.rbe_parent = (*gparent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).entry.rbe_parent).entry.rbe_left {
                        (*(*gparent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = gparent;
                (*gparent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
            }
        } else {
            tmp = (*gparent).entry.rbe_left;
            if !tmp.is_null() && (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).entry.rbe_left == elm {
                    tmp = (*parent).entry.rbe_left;
                    (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                    if !(*parent).entry.rbe_left.is_null() {
                        (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                    }
                    (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                    if !(*tmp).entry.rbe_parent.is_null() {
                        if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                            (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                        } else {
                            (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).entry.rbe_right = parent;
                    (*parent).entry.rbe_parent = tmp;
                    !(*tmp).entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).entry.rbe_color = RB_BLACK;
                (*gparent).entry.rbe_color = RB_RED;
                tmp = (*gparent).entry.rbe_right;
                (*gparent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*gparent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = gparent;
                }
                (*tmp).entry.rbe_parent = (*gparent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).entry.rbe_parent).entry.rbe_left {
                        (*(*gparent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = gparent;
                (*gparent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
            }
        }
    }
    (*(*head).rbh_root).entry.rbe_color = RB_BLACK;
}
unsafe extern "C" fn monitor_windows_RB_INSERT(
    mut head: *mut monitor_windows,
    mut elm: *mut monitor_window,
) -> *mut monitor_window {
    let mut tmp: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut parent: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = monitor_window_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<monitor_window>();
    (*elm).entry.rbe_left = (*elm).entry.rbe_right;
    (*elm).entry.rbe_color = RB_RED;
    if !parent.is_null() {
        if comp < 0 as ::core::ffi::c_int {
            (*parent).entry.rbe_left = elm;
        } else {
            (*parent).entry.rbe_right = elm;
        }
    } else {
        (*head).rbh_root = elm;
    }
    monitor_windows_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<monitor_window>();
}
unsafe extern "C" fn monitor_windows_RB_FIND(
    mut head: *mut monitor_windows,
    mut elm: *mut monitor_window,
) -> *mut monitor_window {
    let mut tmp: *mut monitor_window = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = monitor_window_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<monitor_window>();
}
unsafe extern "C" fn monitor_windows_RB_NEXT(mut elm: *mut monitor_window) -> *mut monitor_window {
    if !(*elm).entry.rbe_right.is_null() {
        elm = (*elm).entry.rbe_right;
        while !(*elm).entry.rbe_left.is_null() {
            elm = (*elm).entry.rbe_left;
        }
    } else if !(*elm).entry.rbe_parent.is_null() && elm == (*(*elm).entry.rbe_parent).entry.rbe_left
    {
        elm = (*elm).entry.rbe_parent;
    } else {
        while !(*elm).entry.rbe_parent.is_null()
            && elm == (*(*elm).entry.rbe_parent).entry.rbe_right
        {
            elm = (*elm).entry.rbe_parent;
        }
        elm = (*elm).entry.rbe_parent;
    }
    return elm;
}
unsafe extern "C" fn monitor_windows_RB_REMOVE_COLOR(
    mut head: *mut monitor_windows,
    mut parent: *mut monitor_window,
    mut elm: *mut monitor_window,
) {
    let mut tmp: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    while (elm.is_null() || (*elm).entry.rbe_color == RB_BLACK) && elm != (*head).rbh_root {
        if (*parent).entry.rbe_left == elm {
            tmp = (*parent).entry.rbe_right;
            if (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_RED;
                tmp = (*parent).entry.rbe_right;
                (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*parent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                tmp = (*parent).entry.rbe_right;
            }
            if ((*tmp).entry.rbe_left.is_null()
                || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK)
                && ((*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK)
            {
                (*tmp).entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).entry.rbe_parent;
            } else {
                if (*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK
                {
                    let mut oleft: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
                    oleft = (*tmp).entry.rbe_left;
                    if !oleft.is_null() {
                        (*oleft).entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).entry.rbe_color = RB_RED;
                    oleft = (*tmp).entry.rbe_left;
                    (*tmp).entry.rbe_left = (*oleft).entry.rbe_right;
                    if !(*tmp).entry.rbe_left.is_null() {
                        (*(*oleft).entry.rbe_right).entry.rbe_parent = tmp;
                    }
                    (*oleft).entry.rbe_parent = (*tmp).entry.rbe_parent;
                    if !(*oleft).entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).entry.rbe_parent).entry.rbe_left {
                            (*(*tmp).entry.rbe_parent).entry.rbe_left = oleft;
                        } else {
                            (*(*tmp).entry.rbe_parent).entry.rbe_right = oleft;
                        }
                    } else {
                        (*head).rbh_root = oleft;
                    }
                    (*oleft).entry.rbe_right = tmp;
                    (*tmp).entry.rbe_parent = oleft;
                    !(*oleft).entry.rbe_parent.is_null();
                    tmp = (*parent).entry.rbe_right;
                }
                (*tmp).entry.rbe_color = (*parent).entry.rbe_color;
                (*parent).entry.rbe_color = RB_BLACK;
                if !(*tmp).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).entry.rbe_right;
                (*parent).entry.rbe_right = (*tmp).entry.rbe_left;
                if !(*parent).entry.rbe_right.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_left = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        } else {
            tmp = (*parent).entry.rbe_left;
            if (*tmp).entry.rbe_color == RB_RED {
                (*tmp).entry.rbe_color = RB_BLACK;
                (*parent).entry.rbe_color = RB_RED;
                tmp = (*parent).entry.rbe_left;
                (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*parent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                tmp = (*parent).entry.rbe_left;
            }
            if ((*tmp).entry.rbe_left.is_null()
                || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK)
                && ((*tmp).entry.rbe_right.is_null()
                    || (*(*tmp).entry.rbe_right).entry.rbe_color == RB_BLACK)
            {
                (*tmp).entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).entry.rbe_parent;
            } else {
                if (*tmp).entry.rbe_left.is_null()
                    || (*(*tmp).entry.rbe_left).entry.rbe_color == RB_BLACK
                {
                    let mut oright: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
                    oright = (*tmp).entry.rbe_right;
                    if !oright.is_null() {
                        (*oright).entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).entry.rbe_color = RB_RED;
                    oright = (*tmp).entry.rbe_right;
                    (*tmp).entry.rbe_right = (*oright).entry.rbe_left;
                    if !(*tmp).entry.rbe_right.is_null() {
                        (*(*oright).entry.rbe_left).entry.rbe_parent = tmp;
                    }
                    (*oright).entry.rbe_parent = (*tmp).entry.rbe_parent;
                    if !(*oright).entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).entry.rbe_parent).entry.rbe_left {
                            (*(*tmp).entry.rbe_parent).entry.rbe_left = oright;
                        } else {
                            (*(*tmp).entry.rbe_parent).entry.rbe_right = oright;
                        }
                    } else {
                        (*head).rbh_root = oright;
                    }
                    (*oright).entry.rbe_left = tmp;
                    (*tmp).entry.rbe_parent = oright;
                    !(*oright).entry.rbe_parent.is_null();
                    tmp = (*parent).entry.rbe_left;
                }
                (*tmp).entry.rbe_color = (*parent).entry.rbe_color;
                (*parent).entry.rbe_color = RB_BLACK;
                if !(*tmp).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_left).entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).entry.rbe_left;
                (*parent).entry.rbe_left = (*tmp).entry.rbe_right;
                if !(*parent).entry.rbe_left.is_null() {
                    (*(*tmp).entry.rbe_right).entry.rbe_parent = parent;
                }
                (*tmp).entry.rbe_parent = (*parent).entry.rbe_parent;
                if !(*tmp).entry.rbe_parent.is_null() {
                    if parent == (*(*parent).entry.rbe_parent).entry.rbe_left {
                        (*(*parent).entry.rbe_parent).entry.rbe_left = tmp;
                    } else {
                        (*(*parent).entry.rbe_parent).entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).entry.rbe_right = parent;
                (*parent).entry.rbe_parent = tmp;
                !(*tmp).entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        }
    }
    if !elm.is_null() {
        (*elm).entry.rbe_color = RB_BLACK;
    }
}
unsafe extern "C" fn monitor_windows_RB_REMOVE(
    mut head: *mut monitor_windows,
    mut elm: *mut monitor_window,
) -> *mut monitor_window {
    let mut current_block: u64;
    let mut child: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut parent: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut old: *mut monitor_window = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
        elm = (*elm).entry.rbe_right;
        loop {
            left = (*elm).entry.rbe_left;
            if left.is_null() {
                break;
            }
            elm = left;
        }
        child = (*elm).entry.rbe_right;
        parent = (*elm).entry.rbe_parent;
        color = (*elm).entry.rbe_color;
        if !child.is_null() {
            (*child).entry.rbe_parent = parent;
        }
        if !parent.is_null() {
            if (*parent).entry.rbe_left == elm {
                (*parent).entry.rbe_left = child;
            } else {
                (*parent).entry.rbe_right = child;
            }
        } else {
            (*head).rbh_root = child;
        }
        if (*elm).entry.rbe_parent == old {
            parent = elm;
        }
        (*elm).entry = (*old).entry;
        if !(*old).entry.rbe_parent.is_null() {
            if (*(*old).entry.rbe_parent).entry.rbe_left == old {
                (*(*old).entry.rbe_parent).entry.rbe_left = elm;
            } else {
                (*(*old).entry.rbe_parent).entry.rbe_right = elm;
            }
        } else {
            (*head).rbh_root = elm;
        }
        (*(*old).entry.rbe_left).entry.rbe_parent = elm;
        if !(*old).entry.rbe_right.is_null() {
            (*(*old).entry.rbe_right).entry.rbe_parent = elm;
        }
        if !parent.is_null() {
            left = parent;
            loop {
                left = (*left).entry.rbe_parent;
                if left.is_null() {
                    break;
                }
            }
        }
        current_block = 9601630785265386486;
    }
    match current_block {
        7245201122033322888 => {
            parent = (*elm).entry.rbe_parent;
            color = (*elm).entry.rbe_color;
            if !child.is_null() {
                (*child).entry.rbe_parent = parent;
            }
            if !parent.is_null() {
                if (*parent).entry.rbe_left == elm {
                    (*parent).entry.rbe_left = child;
                } else {
                    (*parent).entry.rbe_right = child;
                }
            } else {
                (*head).rbh_root = child;
            }
        }
        _ => {}
    }
    if color == RB_BLACK {
        monitor_windows_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn monitor_windows_RB_MINMAX(
    mut head: *mut monitor_windows,
    mut val: ::core::ffi::c_int,
) -> *mut monitor_window {
    let mut tmp: *mut monitor_window = (*head).rbh_root;
    let mut parent: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    while !tmp.is_null() {
        parent = tmp;
        if val < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else {
            tmp = (*tmp).entry.rbe_right;
        }
    }
    return parent;
}
unsafe extern "C" fn monitor_free_item(mut ms: *mut monitor_set, mut me: *mut monitor_item) {
    let mut mp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut mp1: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut mw: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut mw1: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    mp = monitor_panes_RB_MINMAX(&raw mut (*me).panes, RB_NEGINF);
    while !mp.is_null() && {
        mp1 = monitor_panes_RB_NEXT(mp);
        1 as ::core::ffi::c_int != 0
    } {
        monitor_panes_RB_REMOVE(&raw mut (*me).panes, mp);
        free((*mp).last as *mut ::core::ffi::c_void);
        free(mp as *mut ::core::ffi::c_void);
        mp = mp1;
    }
    mw = monitor_windows_RB_MINMAX(&raw mut (*me).windows, RB_NEGINF);
    while !mw.is_null() && {
        mw1 = monitor_windows_RB_NEXT(mw);
        1 as ::core::ffi::c_int != 0
    } {
        monitor_windows_RB_REMOVE(&raw mut (*me).windows, mw);
        free((*mw).last as *mut ::core::ffi::c_void);
        free(mw as *mut ::core::ffi::c_void);
        mw = mw1;
    }
    free((*me).last as *mut ::core::ffi::c_void);
    monitor_items_RB_REMOVE(&raw mut (*ms).items, me);
    free((*me).name as *mut ::core::ffi::c_void);
    free((*me).format as *mut ::core::ffi::c_void);
    free(me as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn monitor_report(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut value: *const ::core::ffi::c_char,
    mut last: *const ::core::ffi::c_char,
) {
    let mut change: monitor_change = monitor_change {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        value: ::core::ptr::null::<::core::ffi::c_char>(),
        last: ::core::ptr::null::<::core::ffi::c_char>(),
        c: ::core::ptr::null_mut::<client>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
    };
    log_debug(
        b"%s: %s changed to %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"monitor_report\0" as *const u8 as *const ::core::ffi::c_char,
        (*me).name,
        value,
    );
    (*me).fire_count = (*me).fire_count.wrapping_add(1);
    (*me).fire_time = current_time;
    change.name = (*me).name;
    change.value = value;
    change.last = last;
    change.c = (*ms).client;
    change.s = s;
    change.wl = wl;
    change.wp = wp;
    (*ms).cb.expect("non-null function pointer")(&raw mut change, (*ms).data);
}
unsafe extern "C" fn monitor_check_value(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
    mut value: *mut ::core::ffi::c_char,
    mut last: *mut *mut ::core::ffi::c_char,
) {
    if (*last).is_null() {
        *last = value;
        if (*me).flags & MONITOR_NOTIFY_INITIAL != 0
            && (!(*me).flags & MONITOR_NOTIFY_TRUE != 0 || format_true(value) != 0)
        {
            monitor_report(
                ms,
                me,
                s,
                wl,
                wp,
                value,
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
        }
        return;
    }
    if strcmp(value, *last) == 0 as ::core::ffi::c_int {
        free(value as *mut ::core::ffi::c_void);
        return;
    }
    if !(*me).flags & MONITOR_NOTIFY_TRUE != 0 || format_true(value) != 0 {
        monitor_report(ms, me, s, wl, wp, value, *last);
    }
    free(*last as *mut ::core::ffi::c_void);
    *last = value;
}
unsafe extern "C" fn monitor_check_session(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut ft: *mut format_tree,
) {
    let mut s: *mut session = monitor_get_session(ms);
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    value = format_expand(ft, (*me).format);
    monitor_check_value(
        ms,
        me,
        s,
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
        value,
        &raw mut (*me).last,
    );
}
unsafe extern "C" fn monitor_check_pane(mut ms: *mut monitor_set, mut me: *mut monitor_item) {
    let mut c: *mut client = (*ms).client;
    let mut s: *mut session = monitor_get_session(ms);
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut find: monitor_pane = monitor_pane {
        pane: 0,
        idx: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        generation: 0,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null_mut::<monitor_pane>(),
            rbe_right: ::core::ptr::null_mut::<monitor_pane>(),
            rbe_parent: ::core::ptr::null_mut::<monitor_pane>(),
            rbe_color: 0,
        },
    };
    wp = window_pane_find_by_id((*me).id);
    if wp.is_null() || (*wp).fd == -(1 as ::core::ffi::c_int) {
        return;
    }
    w = (*wp).window as *mut window;
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if !((*wl).session != s) {
            ft = monitor_create_formats(c, s, wl, wp);
            value = format_expand(ft, (*me).format);
            format_free(ft);
            find.pane = (*wp).id;
            find.idx = (*wl).idx as u_int;
            mp = monitor_panes_RB_FIND(&raw mut (*me).panes, &raw mut find);
            if mp.is_null() {
                mp = xcalloc(
                    1 as size_t,
                    ::core::mem::size_of::<monitor_pane>() as size_t,
                ) as *mut monitor_pane;
                (*mp).pane = (*wp).id;
                (*mp).idx = (*wl).idx as u_int;
                monitor_panes_RB_INSERT(&raw mut (*me).panes, mp);
            }
            monitor_check_value(ms, me, s, wl, wp, value, &raw mut (*mp).last);
        }
        wl = (*wl).wentry.tqe_next;
    }
}
unsafe extern "C" fn monitor_check_all_panes_one(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut ft: *mut format_tree,
    mut wl: *mut winlink,
    mut wp: *mut window_pane,
) {
    let mut s: *mut session = monitor_get_session(ms);
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut find: monitor_pane = monitor_pane {
        pane: 0,
        idx: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        generation: 0,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null_mut::<monitor_pane>(),
            rbe_right: ::core::ptr::null_mut::<monitor_pane>(),
            rbe_parent: ::core::ptr::null_mut::<monitor_pane>(),
            rbe_color: 0,
        },
    };
    value = format_expand(ft, (*me).format);
    find.pane = (*wp).id;
    find.idx = (*wl).idx as u_int;
    mp = monitor_panes_RB_FIND(&raw mut (*me).panes, &raw mut find);
    if mp.is_null() {
        mp = xcalloc(
            1 as size_t,
            ::core::mem::size_of::<monitor_pane>() as size_t,
        ) as *mut monitor_pane;
        (*mp).pane = (*wp).id;
        (*mp).idx = (*wl).idx as u_int;
        monitor_panes_RB_INSERT(&raw mut (*me).panes, mp);
    }
    (*mp).generation = (*ms).generation;
    monitor_check_value(ms, me, s, wl, wp, value, &raw mut (*mp).last);
}
unsafe extern "C" fn monitor_sweep_all_panes(mut me: *mut monitor_item, mut generation: u_int) {
    let mut mp: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    let mut mp1: *mut monitor_pane = ::core::ptr::null_mut::<monitor_pane>();
    mp = monitor_panes_RB_MINMAX(&raw mut (*me).panes, RB_NEGINF);
    while !mp.is_null() && {
        mp1 = monitor_panes_RB_NEXT(mp);
        1 as ::core::ffi::c_int != 0
    } {
        if !((*mp).generation == generation) {
            monitor_panes_RB_REMOVE(&raw mut (*me).panes, mp);
            free((*mp).last as *mut ::core::ffi::c_void);
            free(mp as *mut ::core::ffi::c_void);
        }
        mp = mp1;
    }
}
unsafe extern "C" fn monitor_check_window(mut ms: *mut monitor_set, mut me: *mut monitor_item) {
    let mut c: *mut client = (*ms).client;
    let mut s: *mut session = monitor_get_session(ms);
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mw: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut find: monitor_window = monitor_window {
        window: 0,
        idx: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        generation: 0,
        entry: C2RustUnnamed_36 {
            rbe_left: ::core::ptr::null_mut::<monitor_window>(),
            rbe_right: ::core::ptr::null_mut::<monitor_window>(),
            rbe_parent: ::core::ptr::null_mut::<monitor_window>(),
            rbe_color: 0,
        },
    };
    w = window_find_by_id((*me).id);
    if w.is_null() {
        return;
    }
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if !((*wl).session != s) {
            ft = monitor_create_formats(c, s, wl, ::core::ptr::null_mut::<window_pane>());
            value = format_expand(ft, (*me).format);
            format_free(ft);
            find.window = (*w).id;
            find.idx = (*wl).idx as u_int;
            mw = monitor_windows_RB_FIND(&raw mut (*me).windows, &raw mut find);
            if mw.is_null() {
                mw = xcalloc(
                    1 as size_t,
                    ::core::mem::size_of::<monitor_window>() as size_t,
                ) as *mut monitor_window;
                (*mw).window = (*w).id;
                (*mw).idx = (*wl).idx as u_int;
                monitor_windows_RB_INSERT(&raw mut (*me).windows, mw);
            }
            monitor_check_value(
                ms,
                me,
                s,
                wl,
                ::core::ptr::null_mut::<window_pane>(),
                value,
                &raw mut (*mw).last,
            );
        }
        wl = (*wl).wentry.tqe_next;
    }
}
unsafe extern "C" fn monitor_check_all_windows_one(
    mut ms: *mut monitor_set,
    mut me: *mut monitor_item,
    mut ft: *mut format_tree,
    mut wl: *mut winlink,
) {
    let mut s: *mut session = monitor_get_session(ms);
    let mut w: *mut window = (*wl).window;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut mw: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut find: monitor_window = monitor_window {
        window: 0,
        idx: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        generation: 0,
        entry: C2RustUnnamed_36 {
            rbe_left: ::core::ptr::null_mut::<monitor_window>(),
            rbe_right: ::core::ptr::null_mut::<monitor_window>(),
            rbe_parent: ::core::ptr::null_mut::<monitor_window>(),
            rbe_color: 0,
        },
    };
    value = format_expand(ft, (*me).format);
    find.window = (*w).id;
    find.idx = (*wl).idx as u_int;
    mw = monitor_windows_RB_FIND(&raw mut (*me).windows, &raw mut find);
    if mw.is_null() {
        mw = xcalloc(
            1 as size_t,
            ::core::mem::size_of::<monitor_window>() as size_t,
        ) as *mut monitor_window;
        (*mw).window = (*w).id;
        (*mw).idx = (*wl).idx as u_int;
        monitor_windows_RB_INSERT(&raw mut (*me).windows, mw);
    }
    (*mw).generation = (*ms).generation;
    monitor_check_value(
        ms,
        me,
        s,
        wl,
        ::core::ptr::null_mut::<window_pane>(),
        value,
        &raw mut (*mw).last,
    );
}
unsafe extern "C" fn monitor_sweep_all_windows(mut me: *mut monitor_item, mut generation: u_int) {
    let mut mw: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    let mut mw1: *mut monitor_window = ::core::ptr::null_mut::<monitor_window>();
    mw = monitor_windows_RB_MINMAX(&raw mut (*me).windows, RB_NEGINF);
    while !mw.is_null() && {
        mw1 = monitor_windows_RB_NEXT(mw);
        1 as ::core::ffi::c_int != 0
    } {
        if !((*mw).generation == generation) {
            monitor_windows_RB_REMOVE(&raw mut (*me).windows, mw);
            free((*mw).last as *mut ::core::ffi::c_void);
            free(mw as *mut ::core::ffi::c_void);
        }
        mw = mw1;
    }
}
unsafe extern "C" fn monitor_check_sessions(mut ms: *mut monitor_set) {
    let mut c: *mut client = (*ms).client;
    let mut s: *mut session = monitor_get_session(ms);
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut me1: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    ft = monitor_create_formats(
        c,
        s,
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    me = monitor_items_RB_MINMAX(&raw mut (*ms).items, RB_NEGINF);
    while !me.is_null() && {
        me1 = monitor_items_RB_NEXT(me);
        1 as ::core::ffi::c_int != 0
    } {
        if (*me).type_0 as ::core::ffi::c_uint
            == MONITOR_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            monitor_check_session(ms, me, ft);
        }
        me = me1;
    }
    format_free(ft);
}
unsafe extern "C" fn monitor_check_panes_windows(mut ms: *mut monitor_set) {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut me1: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    me = monitor_items_RB_MINMAX(&raw mut (*ms).items, RB_NEGINF);
    while !me.is_null() && {
        me1 = monitor_items_RB_NEXT(me);
        1 as ::core::ffi::c_int != 0
    } {
        match (*me).type_0 as ::core::ffi::c_uint {
            1 => {
                monitor_check_pane(ms, me);
            }
            3 => {
                monitor_check_window(ms, me);
            }
            0 | 2 | 4 | _ => {}
        }
        me = me1;
    }
}
unsafe extern "C" fn monitor_check_all_panes(mut ms: *mut monitor_set) {
    let mut c: *mut client = (*ms).client;
    let mut s: *mut session = monitor_get_session(ms);
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut me1: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    (*ms).generation = (*ms).generation.wrapping_add(1);
    if (*ms).generation == 0 as u_int {
        (*ms).generation = 1 as u_int;
    }
    wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        wp = (*(*wl).window).panes.tqh_first;
        while !wp.is_null() {
            ft = monitor_create_formats(c, s, wl, wp);
            me = monitor_items_RB_MINMAX(&raw mut (*ms).items, RB_NEGINF);
            while !me.is_null() && {
                me1 = monitor_items_RB_NEXT(me);
                1 as ::core::ffi::c_int != 0
            } {
                if !((*me).type_0 as ::core::ffi::c_uint
                    != MONITOR_ALL_PANES as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    monitor_check_all_panes_one(ms, me, ft, wl, wp);
                }
                me = me1;
            }
            format_free(ft);
            wp = (*wp).entry.tqe_next;
        }
        wl = winlinks_RB_NEXT(wl);
    }
    me = monitor_items_RB_MINMAX(&raw mut (*ms).items, RB_NEGINF);
    while !me.is_null() && {
        me1 = monitor_items_RB_NEXT(me);
        1 as ::core::ffi::c_int != 0
    } {
        if (*me).type_0 as ::core::ffi::c_uint
            == MONITOR_ALL_PANES as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            monitor_sweep_all_panes(me, (*ms).generation);
        }
        me = me1;
    }
}
unsafe extern "C" fn monitor_check_all_windows(mut ms: *mut monitor_set) {
    let mut c: *mut client = (*ms).client;
    let mut s: *mut session = monitor_get_session(ms);
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut me1: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    (*ms).generation = (*ms).generation.wrapping_add(1);
    if (*ms).generation == 0 as u_int {
        (*ms).generation = 1 as u_int;
    }
    wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
    while !wl.is_null() {
        ft = monitor_create_formats(c, s, wl, ::core::ptr::null_mut::<window_pane>());
        me = monitor_items_RB_MINMAX(&raw mut (*ms).items, RB_NEGINF);
        while !me.is_null() && {
            me1 = monitor_items_RB_NEXT(me);
            1 as ::core::ffi::c_int != 0
        } {
            if !((*me).type_0 as ::core::ffi::c_uint
                != MONITOR_ALL_WINDOWS as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                monitor_check_all_windows_one(ms, me, ft, wl);
            }
            me = me1;
        }
        format_free(ft);
        wl = winlinks_RB_NEXT(wl);
    }
    me = monitor_items_RB_MINMAX(&raw mut (*ms).items, RB_NEGINF);
    while !me.is_null() && {
        me1 = monitor_items_RB_NEXT(me);
        1 as ::core::ffi::c_int != 0
    } {
        if (*me).type_0 as ::core::ffi::c_uint
            == MONITOR_ALL_WINDOWS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            monitor_sweep_all_windows(me, (*ms).generation);
        }
        me = me1;
    }
}
unsafe extern "C" fn monitor_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut ms: *mut monitor_set = data as *mut monitor_set;
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut tv: timeval = timeval {
        tv_sec: 1 as __time_t,
        tv_usec: 0,
    };
    let mut have_session: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut have_all_panes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut have_all_windows: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    log_debug(
        b"%s: timer fired\0" as *const u8 as *const ::core::ffi::c_char,
        b"monitor_timer\0" as *const u8 as *const ::core::ffi::c_char,
    );
    event_add(&raw mut (*ms).timer, &raw mut tv);
    if monitor_get_session(ms).is_null() {
        return;
    }
    me = monitor_items_RB_MINMAX(&raw mut (*ms).items, RB_NEGINF);
    while !me.is_null() {
        match (*me).type_0 as ::core::ffi::c_uint {
            0 => {
                have_session = 1 as ::core::ffi::c_int;
            }
            2 => {
                have_all_panes = 1 as ::core::ffi::c_int;
            }
            4 => {
                have_all_windows = 1 as ::core::ffi::c_int;
            }
            1 | 3 | _ => {}
        }
        me = monitor_items_RB_NEXT(me);
    }
    if have_session != 0 {
        monitor_check_sessions(ms);
    }
    monitor_check_panes_windows(ms);
    if have_all_panes != 0 {
        monitor_check_all_panes(ms);
    }
    if have_all_windows != 0 {
        monitor_check_all_windows(ms);
    }
}
unsafe extern "C" fn monitor_create(
    mut cb: monitor_cb,
    mut data: *mut ::core::ffi::c_void,
) -> *mut monitor_set {
    let mut ms: *mut monitor_set = ::core::ptr::null_mut::<monitor_set>();
    ms = xcalloc(1 as size_t, ::core::mem::size_of::<monitor_set>() as size_t) as *mut monitor_set;
    (*ms).cb = cb;
    (*ms).data = data;
    (*ms).items.rbh_root = ::core::ptr::null_mut::<monitor_item>();
    return ms;
}
#[no_mangle]
pub unsafe extern "C" fn monitor_create_client(
    mut c: *mut client,
    mut cb: monitor_cb,
    mut data: *mut ::core::ffi::c_void,
) -> *mut monitor_set {
    let mut ms: *mut monitor_set = ::core::ptr::null_mut::<monitor_set>();
    ms = monitor_create(cb, data);
    (*ms).client = c;
    return ms as *mut monitor_set;
}
#[no_mangle]
pub unsafe extern "C" fn monitor_create_session(
    mut s: *mut session,
    mut cb: monitor_cb,
    mut data: *mut ::core::ffi::c_void,
) -> *mut monitor_set {
    let mut ms: *mut monitor_set = ::core::ptr::null_mut::<monitor_set>();
    ms = monitor_create(cb, data);
    (*ms).session = s;
    if !s.is_null() {
        session_add_ref(
            s,
            b"monitor_create_session\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    return ms;
}
#[no_mangle]
pub unsafe extern "C" fn monitor_destroy(mut ms: *mut monitor_set) {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut me1: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    if !ms.is_null() {
        if event_initialized(&raw mut (*ms).timer) != 0 {
            event_del(&raw mut (*ms).timer);
        }
        me = monitor_items_RB_MINMAX(&raw mut (*ms).items, RB_NEGINF);
        while !me.is_null() && {
            me1 = monitor_items_RB_NEXT(me);
            1 as ::core::ffi::c_int != 0
        } {
            monitor_free_item(ms, me);
            me = me1;
        }
        if !(*ms).session.is_null() {
            session_remove_ref(
                (*ms).session,
                b"monitor_destroy\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        free(ms as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn monitor_parse(
    mut value: *const ::core::ffi::c_char,
    mut name: *mut *mut ::core::ffi::c_char,
    mut type_0: *mut monitor_type,
    mut id: *mut ::core::ffi::c_int,
    mut format: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut what: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut split: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    copy = xstrdup(value);
    *id = -(1 as ::core::ffi::c_int);
    what = strchr(copy, ':' as i32);
    if !what.is_null() {
        let fresh0 = what;
        what = what.offset(1);
        *fresh0 = '\0' as i32 as ::core::ffi::c_char;
        split = strchr(what, ':' as i32);
        if !split.is_null() {
            let fresh1 = split;
            split = split.offset(1);
            *fresh1 = '\0' as i32 as ::core::ffi::c_char;
            if strcmp(what, b"%*\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                *type_0 = MONITOR_ALL_PANES;
                current_block = 3512920355445576850;
            } else if sscanf(
                what,
                b"%%%d\0" as *const u8 as *const ::core::ffi::c_char,
                id,
            ) == 1 as ::core::ffi::c_int
                && *id >= 0 as ::core::ffi::c_int
            {
                *type_0 = MONITOR_PANE;
                current_block = 3512920355445576850;
            } else if strcmp(what, b"@*\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                *type_0 = MONITOR_ALL_WINDOWS;
                current_block = 3512920355445576850;
            } else if sscanf(
                what,
                b"@%d\0" as *const u8 as *const ::core::ffi::c_char,
                id,
            ) == 1 as ::core::ffi::c_int
                && *id >= 0 as ::core::ffi::c_int
            {
                *type_0 = MONITOR_WINDOW;
                current_block = 3512920355445576850;
            } else if *what as ::core::ffi::c_int == '\0' as i32 {
                *type_0 = MONITOR_SESSION;
                current_block = 3512920355445576850;
            } else {
                current_block = 7799373935801088419;
            }
            match current_block {
                7799373935801088419 => {}
                _ => {
                    *name = xstrdup(copy);
                    *format = xstrdup(split);
                    free(copy as *mut ::core::ffi::c_void);
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
    }
    free(copy as *mut ::core::ffi::c_void);
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn monitor_add(
    mut ms: *mut monitor_set,
    mut name: *const ::core::ffi::c_char,
    mut type_0: monitor_type,
    mut id: ::core::ffi::c_int,
    mut format: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut find: monitor_item = monitor_item {
        name: name as *mut ::core::ffi::c_char,
        format: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        type_0: MONITOR_SESSION,
        id: 0,
        flags: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        panes: monitor_panes {
            rbh_root: ::core::ptr::null_mut::<monitor_pane>(),
        },
        windows: monitor_windows {
            rbh_root: ::core::ptr::null_mut::<monitor_window>(),
        },
        fire_count: 0,
        fire_time: 0,
        entry: C2RustUnnamed_35 {
            rbe_left: ::core::ptr::null_mut::<monitor_item>(),
            rbe_right: ::core::ptr::null_mut::<monitor_item>(),
            rbe_parent: ::core::ptr::null_mut::<monitor_item>(),
            rbe_color: 0,
        },
    };
    let mut tv: timeval = timeval {
        tv_sec: 1 as __time_t,
        tv_usec: 0,
    };
    me = monitor_items_RB_FIND(&raw mut (*ms).items, &raw mut find);
    if !me.is_null() {
        monitor_free_item(ms, me);
    }
    me = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<monitor_item>() as size_t,
    ) as *mut monitor_item;
    (*me).name = xstrdup(name);
    (*me).format = xstrdup(format);
    (*me).type_0 = type_0;
    (*me).id = id as u_int;
    (*me).flags = flags;
    (*me).panes.rbh_root = ::core::ptr::null_mut::<monitor_pane>();
    (*me).windows.rbh_root = ::core::ptr::null_mut::<monitor_window>();
    monitor_items_RB_INSERT(&raw mut (*ms).items, me);
    if event_initialized(&raw mut (*ms).timer) == 0 {
        event_set(
            &raw mut (*ms).timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            Some(
                monitor_timer
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            ms as *mut ::core::ffi::c_void,
        );
    }
    if event_pending(
        &raw mut (*ms).timer,
        EV_TIMEOUT as ::core::ffi::c_short,
        ::core::ptr::null_mut::<timeval>(),
    ) == 0
    {
        event_add(&raw mut (*ms).timer, &raw mut tv);
    }
}
#[no_mangle]
pub unsafe extern "C" fn monitor_remove(
    mut ms: *mut monitor_set,
    mut name: *const ::core::ffi::c_char,
) {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut find: monitor_item = monitor_item {
        name: name as *mut ::core::ffi::c_char,
        format: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        type_0: MONITOR_SESSION,
        id: 0,
        flags: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        panes: monitor_panes {
            rbh_root: ::core::ptr::null_mut::<monitor_pane>(),
        },
        windows: monitor_windows {
            rbh_root: ::core::ptr::null_mut::<monitor_window>(),
        },
        fire_count: 0,
        fire_time: 0,
        entry: C2RustUnnamed_35 {
            rbe_left: ::core::ptr::null_mut::<monitor_item>(),
            rbe_right: ::core::ptr::null_mut::<monitor_item>(),
            rbe_parent: ::core::ptr::null_mut::<monitor_item>(),
            rbe_color: 0,
        },
    };
    me = monitor_items_RB_FIND(&raw mut (*ms).items, &raw mut find);
    if !me.is_null() {
        monitor_free_item(ms, me);
    }
    if (*ms).items.rbh_root.is_null() && event_initialized(&raw mut (*ms).timer) != 0 {
        event_del(&raw mut (*ms).timer);
    }
}
#[no_mangle]
pub unsafe extern "C" fn monitor_get_fire_count(
    mut ms: *mut monitor_set,
    mut name: *const ::core::ffi::c_char,
) -> u_int {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut find: monitor_item = monitor_item {
        name: name as *mut ::core::ffi::c_char,
        format: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        type_0: MONITOR_SESSION,
        id: 0,
        flags: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        panes: monitor_panes {
            rbh_root: ::core::ptr::null_mut::<monitor_pane>(),
        },
        windows: monitor_windows {
            rbh_root: ::core::ptr::null_mut::<monitor_window>(),
        },
        fire_count: 0,
        fire_time: 0,
        entry: C2RustUnnamed_35 {
            rbe_left: ::core::ptr::null_mut::<monitor_item>(),
            rbe_right: ::core::ptr::null_mut::<monitor_item>(),
            rbe_parent: ::core::ptr::null_mut::<monitor_item>(),
            rbe_color: 0,
        },
    };
    me = monitor_items_RB_FIND(&raw mut (*ms).items, &raw mut find);
    if me.is_null() {
        return 0 as u_int;
    }
    return (*me).fire_count;
}
#[no_mangle]
pub unsafe extern "C" fn monitor_get_fire_time(
    mut ms: *mut monitor_set,
    mut name: *const ::core::ffi::c_char,
) -> time_t {
    let mut me: *mut monitor_item = ::core::ptr::null_mut::<monitor_item>();
    let mut find: monitor_item = monitor_item {
        name: name as *mut ::core::ffi::c_char,
        format: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        type_0: MONITOR_SESSION,
        id: 0,
        flags: 0,
        last: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        panes: monitor_panes {
            rbh_root: ::core::ptr::null_mut::<monitor_pane>(),
        },
        windows: monitor_windows {
            rbh_root: ::core::ptr::null_mut::<monitor_window>(),
        },
        fire_count: 0,
        fire_time: 0,
        entry: C2RustUnnamed_35 {
            rbe_left: ::core::ptr::null_mut::<monitor_item>(),
            rbe_right: ::core::ptr::null_mut::<monitor_item>(),
            rbe_parent: ::core::ptr::null_mut::<monitor_item>(),
            rbe_color: 0,
        },
    };
    me = monitor_items_RB_FIND(&raw mut (*ms).items, &raw mut find);
    if me.is_null() {
        return 0 as time_t;
    }
    return (*me).fire_time;
}

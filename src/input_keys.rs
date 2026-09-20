pub use crate::src::shared::screen::{
    ALL_MOUSE_MODES, EXTENDED_KEY_MODES, MODE_BRACKETPASTE, MODE_KCURSOR, MODE_KEYS_EXTENDED,
    MODE_KEYS_EXTENDED_2, MODE_KKEYPAD, MODE_MOUSE_ALL, MODE_MOUSE_BUTTON, MODE_MOUSE_SGR,
    MODE_MOUSE_STANDARD, MODE_MOUSE_UTF8, screen, screen_sel, screen_titles,
};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{
    MOUSE_MASK_BUTTONS, MOUSE_MASK_DRAG, MOUSE_PARAM_BTN_OFF, MOUSE_PARAM_MAX,
    MOUSE_PARAM_POS_OFF, MOUSE_PARAM_UTF8_MAX, mouse_event,
};
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
use crate::src::shared::utf8::*;
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
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strcspn(
        __s: *const ::core::ffi::c_char,
        __reject: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_ulong;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn bufferevent_write(
        bufev: *mut bufferevent,
        data: *const ::core::ffi::c_void,
        size: size_t,
    ) -> ::core::ffi::c_int;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xsnprintf(
        _: *mut ::core::ffi::c_char,
        _: size_t,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    static mut global_options: *mut options;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn cmd_mouse_at(
        _: *mut window_pane,
        _: *mut mouse_event,
        _: *mut u_int,
        _: *mut u_int,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn key_string_lookup_key(_: key_code, _: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn window_pane_is_visible(_: *mut window_pane) -> ::core::ffi::c_int;
    fn utf8_towc(_: *const utf8_data, _: *mut wchar_t) -> utf8_state;
    fn utf8_to_data(_: utf8_char, _: *mut utf8_data);
    fn log_get_level() -> ::core::ffi::c_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
}
pub type wchar_t = ::libc::wchar_t;
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
pub type C2RustUnnamed_35 = ::core::ffi::c_uint;
pub const C0_US: C2RustUnnamed_35 = 31;
pub const C0_RS: C2RustUnnamed_35 = 30;
pub const C0_GS: C2RustUnnamed_35 = 29;
pub const C0_FS: C2RustUnnamed_35 = 28;
pub const C0_ESC: C2RustUnnamed_35 = 27;
pub const C0_SUB: C2RustUnnamed_35 = 26;
pub const C0_EM: C2RustUnnamed_35 = 25;
pub const C0_CAN: C2RustUnnamed_35 = 24;
pub const C0_ETB: C2RustUnnamed_35 = 23;
pub const C0_SYN: C2RustUnnamed_35 = 22;
pub const C0_NAK: C2RustUnnamed_35 = 21;
pub const C0_DC4: C2RustUnnamed_35 = 20;
pub const C0_DC3: C2RustUnnamed_35 = 19;
pub const C0_DC2: C2RustUnnamed_35 = 18;
pub const C0_DC1: C2RustUnnamed_35 = 17;
pub const C0_DLE: C2RustUnnamed_35 = 16;
pub const C0_SI: C2RustUnnamed_35 = 15;
pub const C0_SO: C2RustUnnamed_35 = 14;
pub const C0_CR: C2RustUnnamed_35 = 13;
pub const C0_FF: C2RustUnnamed_35 = 12;
pub const C0_VT: C2RustUnnamed_35 = 11;
pub const C0_LF: C2RustUnnamed_35 = 10;
pub const C0_HT: C2RustUnnamed_35 = 9;
pub const C0_BS: C2RustUnnamed_35 = 8;
pub const C0_BEL: C2RustUnnamed_35 = 7;
pub const C0_ASC: C2RustUnnamed_35 = 6;
pub const C0_ENQ: C2RustUnnamed_35 = 5;
pub const C0_EOT: C2RustUnnamed_35 = 4;
pub const C0_ETX: C2RustUnnamed_35 = 3;
pub const C0_STX: C2RustUnnamed_35 = 2;
pub const C0_SOH: C2RustUnnamed_35 = 1;
pub const C0_NUL: C2RustUnnamed_35 = 0;
pub type C2RustUnnamed_36 = ::core::ffi::c_ulong;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_key_entry {
    pub key: key_code,
    pub data: *const ::core::ffi::c_char,
    pub entry: C2RustUnnamed_37,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
    pub rbe_left: *mut input_key_entry,
    pub rbe_right: *mut input_key_entry,
    pub rbe_parent: *mut input_key_entry,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_key_tree {
    pub rbh_root: *mut input_key_entry,
}
pub const MOTION_MOUSE_MODES: ::core::ffi::c_int = MODE_MOUSE_BUTTON | MODE_MOUSE_ALL;
unsafe extern "C" fn input_key_tree_RB_INSERT_COLOR(
    mut head: *mut input_key_tree,
    mut elm: *mut input_key_entry,
) {
    let mut parent: *mut input_key_entry = ::core::ptr::null_mut::<input_key_entry>();
    let mut gparent: *mut input_key_entry = ::core::ptr::null_mut::<input_key_entry>();
    let mut tmp: *mut input_key_entry = ::core::ptr::null_mut::<input_key_entry>();
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
unsafe extern "C" fn input_key_tree_RB_FIND(
    mut head: *mut input_key_tree,
    mut elm: *mut input_key_entry,
) -> *mut input_key_entry {
    let mut tmp: *mut input_key_entry = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = input_key_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<input_key_entry>();
}
unsafe extern "C" fn input_key_tree_RB_MINMAX(
    mut head: *mut input_key_tree,
    mut val: ::core::ffi::c_int,
) -> *mut input_key_entry {
    let mut tmp: *mut input_key_entry = (*head).rbh_root;
    let mut parent: *mut input_key_entry = ::core::ptr::null_mut::<input_key_entry>();
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
unsafe extern "C" fn input_key_tree_RB_INSERT(
    mut head: *mut input_key_tree,
    mut elm: *mut input_key_entry,
) -> *mut input_key_entry {
    let mut tmp: *mut input_key_entry = ::core::ptr::null_mut::<input_key_entry>();
    let mut parent: *mut input_key_entry = ::core::ptr::null_mut::<input_key_entry>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = input_key_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<input_key_entry>();
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
    input_key_tree_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<input_key_entry>();
}
unsafe extern "C" fn input_key_tree_RB_NEXT(mut elm: *mut input_key_entry) -> *mut input_key_entry {
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
#[no_mangle]
pub static mut input_key_tree: input_key_tree = input_key_tree {
    rbh_root: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
};
static mut input_key_defaults: [input_key_entry; 85] = [
    input_key_entry {
        key: KEYC_PASTE_START as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[200~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_PASTE_START as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
        data: b"\x1B[200~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_PASTE_END as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[201~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_PASTE_END as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
        data: b"\x1B[201~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code,
        data: b"\x1BOP\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code,
        data: b"\x1BOQ\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code,
        data: b"\x1BOR\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code,
        data: b"\x1BOS\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[15~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[17~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[18~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[19~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[20~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[21~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[23~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[24~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_IC as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[2~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_DC as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[3~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[1~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_END as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[4~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[6~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[5~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_BTAB as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[Z\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
        data: b"\x1BOA\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
        data: b"\x1BOB\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
        data: b"\x1BOC\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
        data: b"\x1BOD\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_UP as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[A\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[B\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[C\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code,
        data: b"\x1B[D\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_SLASH as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOo\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_STAR as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOj\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_MINUS as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOm\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_SEVEN as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOw\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_EIGHT as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOx\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_NINE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOy\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_PLUS as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOk\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_FOUR as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOt\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_FIVE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOu\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_SIX as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOv\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_ONE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOq\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_TWO as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOr\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_THREE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOs\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_ENTER as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOM\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_ZERO as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOp\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_PERIOD as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
        data: b"\x1BOn\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_SLASH as ::core::ffi::c_ulong as key_code,
        data: b"/\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_STAR as ::core::ffi::c_ulong as key_code,
        data: b"*\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_MINUS as ::core::ffi::c_ulong as key_code,
        data: b"-\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_SEVEN as ::core::ffi::c_ulong as key_code,
        data: b"7\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_EIGHT as ::core::ffi::c_ulong as key_code,
        data: b"8\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_NINE as ::core::ffi::c_ulong as key_code,
        data: b"9\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_PLUS as ::core::ffi::c_ulong as key_code,
        data: b"+\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_FOUR as ::core::ffi::c_ulong as key_code,
        data: b"4\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_FIVE as ::core::ffi::c_ulong as key_code,
        data: b"5\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_SIX as ::core::ffi::c_ulong as key_code,
        data: b"6\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_ONE as ::core::ffi::c_ulong as key_code,
        data: b"1\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_TWO as ::core::ffi::c_ulong as key_code,
        data: b"2\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_THREE as ::core::ffi::c_ulong as key_code,
        data: b"3\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_ENTER as ::core::ffi::c_ulong as key_code,
        data: b"\n\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_ZERO as ::core::ffi::c_ulong as key_code,
        data: b"0\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_KP_PERIOD as ::core::ffi::c_ulong as key_code,
        data: b".\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_P\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_Q\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_R\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_S\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[15;_~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[17;_~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[18;_~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[19;_~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[20;_~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[21;_~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[23;_~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[24;_~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_A\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_B\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_C\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_D\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_H\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_END as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[1;_F\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[5;_~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[6;_~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_IC as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[2;_~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
    input_key_entry {
        key: KEYC_DC as ::core::ffi::c_ulong as key_code | KEYC_BUILD_MODIFIERS,
        data: b"\x1B[3;_~\0" as *const u8 as *const ::core::ffi::c_char,
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    },
];
static mut input_key_modifiers: [key_code; 9] = [
    0 as ::core::ffi::c_int as key_code,
    0 as ::core::ffi::c_int as key_code,
    KEYC_SHIFT,
    KEYC_META | KEYC_IMPLIED_META,
    KEYC_SHIFT | KEYC_META | KEYC_IMPLIED_META,
    KEYC_CTRL,
    KEYC_SHIFT | KEYC_CTRL,
    KEYC_META | KEYC_IMPLIED_META | KEYC_CTRL,
    KEYC_SHIFT | KEYC_META | KEYC_IMPLIED_META | KEYC_CTRL,
];
unsafe extern "C" fn input_key_cmp(
    mut ike1: *mut input_key_entry,
    mut ike2: *mut input_key_entry,
) -> ::core::ffi::c_int {
    if (*ike1).key < (*ike2).key {
        return -(1 as ::core::ffi::c_int);
    }
    if (*ike1).key > (*ike2).key {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_key_get(mut key: key_code) -> *mut input_key_entry {
    let mut entry: input_key_entry = input_key_entry {
        key: key,
        data: ::core::ptr::null::<::core::ffi::c_char>(),
        entry: C2RustUnnamed_37 {
            rbe_left: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_right: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_parent: ::core::ptr::null::<input_key_entry>() as *mut input_key_entry,
            rbe_color: 0,
        },
    };
    return input_key_tree_RB_FIND(&raw mut input_key_tree, &raw mut entry);
}
unsafe extern "C" fn input_key_split2(mut c: u_int, mut dst: *mut u_char) -> size_t {
    if c > 0x7f as u_int {
        *dst.offset(0 as ::core::ffi::c_int as isize) =
            (c >> 6 as ::core::ffi::c_int | 0xc0 as u_int) as u_char;
        *dst.offset(1 as ::core::ffi::c_int as isize) =
            (c & 0x3f as u_int | 0x80 as u_int) as u_char;
        return 2 as size_t;
    }
    *dst.offset(0 as ::core::ffi::c_int as isize) = c as u_char;
    return 1 as size_t;
}
#[no_mangle]
pub unsafe extern "C" fn input_key_build() {
    let mut ike: *mut input_key_entry = ::core::ptr::null_mut::<input_key_entry>();
    let mut new: *mut input_key_entry = ::core::ptr::null_mut::<input_key_entry>();
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut data: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: key_code = 0;
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[input_key_entry; 85]>() as usize)
            .wrapping_div(::core::mem::size_of::<input_key_entry>() as usize)
    {
        ike = (&raw mut input_key_defaults as *mut input_key_entry).offset(i as isize)
            as *mut input_key_entry;
        if !((*ike).key as ::core::ffi::c_ulonglong) & KEYC_BUILD_MODIFIERS != 0 {
            input_key_tree_RB_INSERT(&raw mut input_key_tree, ike);
        } else {
            j = 2 as u_int;
            while (j as usize)
                < (::core::mem::size_of::<[key_code; 9]>() as usize)
                    .wrapping_div(::core::mem::size_of::<key_code>() as usize)
            {
                key = ((*ike).key as ::core::ffi::c_ulonglong & !KEYC_BUILD_MODIFIERS) as key_code;
                data = xstrdup((*ike).data);
                *data.offset(
                    strcspn(data, b"_\0" as *const u8 as *const ::core::ffi::c_char) as isize,
                ) = ('0' as i32 as u_int).wrapping_add(j) as ::core::ffi::c_char;
                new = xcalloc(
                    1 as size_t,
                    ::core::mem::size_of::<input_key_entry>() as size_t,
                ) as *mut input_key_entry;
                (*new).key = key | input_key_modifiers[j as usize];
                (*new).data = data;
                input_key_tree_RB_INSERT(&raw mut input_key_tree, new);
                j = j.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
    ike = input_key_tree_RB_MINMAX(&raw mut input_key_tree, RB_NEGINF);
    while !ike.is_null() {
        log_debug(
            b"%s: 0x%llx (%s) is %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"input_key_build\0" as *const u8 as *const ::core::ffi::c_char,
            (*ike).key,
            key_string_lookup_key((*ike).key, 1 as ::core::ffi::c_int),
            (*ike).data,
        );
        ike = input_key_tree_RB_NEXT(ike);
    }
}
#[no_mangle]
pub unsafe extern "C" fn input_key_pane(
    mut wp: *mut window_pane,
    mut key: key_code,
    mut m: *mut mouse_event,
) -> ::core::ffi::c_int {
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(
            b"writing key 0x%llx (%s) to %%%u\0" as *const u8 as *const ::core::ffi::c_char,
            key,
            key_string_lookup_key(key, 1 as ::core::ffi::c_int),
            (*wp).id,
        );
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
        if !m.is_null() && (*m).wp != -(1 as ::core::ffi::c_int) && (*m).wp as u_int == (*wp).id {
            input_key_mouse(wp, m);
        }
        return 0 as ::core::ffi::c_int;
    }
    return input_key((*wp).screen, (*wp).event, key);
}
unsafe extern "C" fn input_key_write(
    mut from: *const ::core::ffi::c_char,
    mut bev: *mut bufferevent,
    mut data: *const ::core::ffi::c_char,
    mut size: size_t,
) {
    log_debug(
        b"%s: %.*s\0" as *const u8 as *const ::core::ffi::c_char,
        from,
        size as ::core::ffi::c_int,
        data,
    );
    bufferevent_write(bev, data as *const ::core::ffi::c_void, size);
}
unsafe extern "C" fn input_key_extended(
    mut bev: *mut bufferevent,
    mut key: key_code,
) -> ::core::ffi::c_int {
    let mut tmp: [::core::ffi::c_char; 64] = [0; 64];
    let mut modifier: ::core::ffi::c_char = 0;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut wc: wchar_t = 0;
    match key as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS {
        KEYC_SHIFT => {
            modifier = '2' as i32 as ::core::ffi::c_char;
        }
        KEYC_META => {
            modifier = '3' as i32 as ::core::ffi::c_char;
        }
        87960930222080 => {
            modifier = '4' as i32 as ::core::ffi::c_char;
        }
        KEYC_CTRL => {
            modifier = '5' as i32 as ::core::ffi::c_char;
        }
        105553116266496 => {
            modifier = '6' as i32 as ::core::ffi::c_char;
        }
        52776558133248 => {
            modifier = '7' as i32 as ::core::ffi::c_char;
        }
        123145302310912 => {
            modifier = '8' as i32 as ::core::ffi::c_char;
        }
        _ => return -(1 as ::core::ffi::c_int),
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
        == (KEYC_TYPE_UNICODE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
            << 32 as ::core::ffi::c_int
        && key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY > 0x7f as ::core::ffi::c_ulonglong
    {
        utf8_to_data(
            (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY) as utf8_char,
            &raw mut ud,
        );
        if utf8_towc(&raw mut ud, &raw mut wc) as ::core::ffi::c_uint
            == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            key = wc as key_code;
        } else {
            return -(1 as ::core::ffi::c_int);
        }
    } else {
        key &= KEYC_MASK_KEY;
    }
    if options_get_number(
        global_options,
        b"extended-keys-format\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 1 as ::core::ffi::c_longlong
    {
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
            b"\x1B[27;%c;%llu~\0" as *const u8 as *const ::core::ffi::c_char,
            modifier as ::core::ffi::c_int,
            key,
        );
    } else {
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
            b"\x1B[%llu;%cu\0" as *const u8 as *const ::core::ffi::c_char,
            key,
            modifier as ::core::ffi::c_int,
        );
    }
    input_key_write(
        b"input_key_extended\0" as *const u8 as *const ::core::ffi::c_char,
        bev,
        &raw mut tmp as *mut ::core::ffi::c_char,
        strlen(&raw mut tmp as *mut ::core::ffi::c_char),
    );
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_key_vt10x(
    mut bev: *mut bufferevent,
    mut key: key_code,
) -> ::core::ffi::c_int {
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut onlykey: key_code = 0;
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    static mut standard_map: [*const ::core::ffi::c_char; 2] = [
        b"1!9(0)=+;:'\",<.>/-8? 2\0" as *const u8 as *const ::core::ffi::c_char,
        b"119900=+;;'',,..\x1F\x1F\x7F\x7F\0\0\0" as *const u8 as *const ::core::ffi::c_char,
    ];
    log_debug(
        b"%s: key in %llx\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_key_vt10x\0" as *const u8 as *const ::core::ffi::c_char,
        key,
    );
    if key as ::core::ffi::c_ulonglong & KEYC_META != 0 {
        input_key_write(
            b"input_key_vt10x\0" as *const u8 as *const ::core::ffi::c_char,
            bev,
            b"\x1B\0" as *const u8 as *const ::core::ffi::c_char,
            1 as size_t,
        );
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
        == (KEYC_TYPE_UNICODE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
            << 32 as ::core::ffi::c_int
        && key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY > 0x7f as ::core::ffi::c_ulonglong
    {
        utf8_to_data(key as utf8_char, &raw mut ud);
        input_key_write(
            b"input_key_vt10x\0" as *const u8 as *const ::core::ffi::c_char,
            bev,
            &raw mut ud.data as *mut u_char as *const ::core::ffi::c_char,
            ud.size as size_t,
        );
        return 0 as ::core::ffi::c_int;
    }
    onlykey = (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY) as key_code;
    if onlykey == '\r' as i32 as key_code
        || onlykey == '\n' as i32 as key_code
        || onlykey == '\t' as i32 as key_code
    {
        key &= !KEYC_CTRL;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_CTRL != 0 {
        p = strchr(
            standard_map[0 as ::core::ffi::c_int as usize],
            onlykey as ::core::ffi::c_int,
        );
        if !p.is_null() {
            key = *standard_map[1 as ::core::ffi::c_int as usize].offset(
                p.offset_from(standard_map[0 as ::core::ffi::c_int as usize]) as ::core::ffi::c_long
                    as isize,
            ) as key_code;
        } else if onlykey >= '3' as i32 as key_code && onlykey <= '7' as i32 as key_code {
            key = onlykey.wrapping_sub('\u{18}' as i32 as key_code);
        } else if onlykey >= '@' as i32 as key_code && onlykey <= '~' as i32 as key_code {
            key = onlykey & 0x1f as key_code;
        } else {
            return -(1 as ::core::ffi::c_int);
        }
    }
    log_debug(
        b"%s: key out %llx\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_key_vt10x\0" as *const u8 as *const ::core::ffi::c_char,
        key,
    );
    ud.data[0 as ::core::ffi::c_int as usize] = (key & 0x7f as key_code) as u_char;
    input_key_write(
        b"input_key_vt10x\0" as *const u8 as *const ::core::ffi::c_char,
        bev,
        (&raw mut ud.data as *mut u_char).offset(0 as ::core::ffi::c_int as isize) as *mut u_char
            as *const ::core::ffi::c_char,
        1 as size_t,
    );
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_key_mode1(
    mut bev: *mut bufferevent,
    mut key: key_code,
) -> ::core::ffi::c_int {
    let mut onlykey: key_code = 0;
    log_debug(
        b"%s: key in %llx\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_key_mode1\0" as *const u8 as *const ::core::ffi::c_char,
        key,
    );
    if key as ::core::ffi::c_ulonglong & (KEYC_CTRL | KEYC_META) == KEYC_META {
        return input_key_vt10x(bev, key);
    }
    onlykey = (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY) as key_code;
    if key as ::core::ffi::c_ulonglong & KEYC_CTRL != 0
        && (onlykey == ' ' as i32 as key_code
            || onlykey == '/' as i32 as key_code
            || onlykey == '@' as i32 as key_code
            || onlykey == '^' as i32 as key_code
            || onlykey >= '2' as i32 as key_code && onlykey <= '8' as i32 as key_code
            || onlykey >= '@' as i32 as key_code && onlykey <= '~' as i32 as key_code)
    {
        return input_key_vt10x(bev, key);
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn input_key(
    mut s: *mut screen,
    mut bev: *mut bufferevent,
    mut key: key_code,
) -> ::core::ffi::c_int {
    let mut ike: *mut input_key_entry = ::core::ptr::null_mut::<input_key_entry>();
    let mut newkey: key_code = 0;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_LITERAL != 0 {
        ud.data[0 as ::core::ffi::c_int as usize] = key as u_char;
        input_key_write(
            b"input_key\0" as *const u8 as *const ::core::ffi::c_char,
            bev,
            (&raw mut ud.data as *mut u_char).offset(0 as ::core::ffi::c_int as isize)
                as *mut u_char as *const ::core::ffi::c_char,
            1 as size_t,
        );
        return 0 as ::core::ffi::c_int;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_BSPACE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
    {
        newkey = options_get_number(
            global_options,
            b"backspace\0" as *const u8 as *const ::core::ffi::c_char,
        ) as key_code;
        log_debug(
            b"%s: key 0x%llx is backspace -> 0x%llx\0" as *const u8 as *const ::core::ffi::c_char,
            b"input_key\0" as *const u8 as *const ::core::ffi::c_char,
            key,
            newkey,
        );
        if key as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS == 0 as ::core::ffi::c_ulonglong {
            ud.data[0 as ::core::ffi::c_int as usize] = 255 as u_char;
            if newkey as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS
                == 0 as ::core::ffi::c_ulonglong
            {
                ud.data[0 as ::core::ffi::c_int as usize] = newkey as u_char;
            } else if newkey as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS == KEYC_CTRL {
                newkey &= KEYC_MASK_KEY;
                if newkey == '?' as i32 as key_code {
                    ud.data[0 as ::core::ffi::c_int as usize] = 0x7f as u_char;
                } else if newkey >= '@' as i32 as key_code && newkey <= '_' as i32 as key_code {
                    ud.data[0 as ::core::ffi::c_int as usize] =
                        newkey.wrapping_sub(0x40 as key_code) as u_char;
                } else if newkey >= 'a' as i32 as key_code && newkey <= 'z' as i32 as key_code {
                    ud.data[0 as ::core::ffi::c_int as usize] =
                        newkey.wrapping_sub(0x60 as key_code) as u_char;
                }
            }
            if ud.data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                != 255 as ::core::ffi::c_int
            {
                input_key_write(
                    b"input_key\0" as *const u8 as *const ::core::ffi::c_char,
                    bev,
                    (&raw mut ud.data as *mut u_char).offset(0 as ::core::ffi::c_int as isize)
                        as *mut u_char as *const ::core::ffi::c_char,
                    1 as size_t,
                );
            }
            return 0 as ::core::ffi::c_int;
        }
        key = (newkey as ::core::ffi::c_ulonglong
            | key as ::core::ffi::c_ulonglong & (KEYC_MASK_FLAGS | KEYC_MASK_MODIFIERS))
            as key_code;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_BTAB as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
    {
        if (*s).mode & MODE_KEYS_EXTENDED_2 != 0 {
            key = ('\t' as i32 as ::core::ffi::c_ulonglong
                | key as ::core::ffi::c_ulonglong & !KEYC_MASK_KEY
                | KEYC_SHIFT) as key_code;
        } else {
            key &= !KEYC_MASK_MODIFIERS;
        }
    }
    if key as ::core::ffi::c_ulonglong & !KEYC_MASK_KEY == 0 {
        if key == C0_HT as ::core::ffi::c_int as key_code
            || key == C0_CR as ::core::ffi::c_int as key_code
            || key == C0_ESC as ::core::ffi::c_int as key_code
            || key >= 0x20 as key_code && key <= 0x7f as key_code
        {
            ud.data[0 as ::core::ffi::c_int as usize] = key as u_char;
            input_key_write(
                b"input_key\0" as *const u8 as *const ::core::ffi::c_char,
                bev,
                (&raw mut ud.data as *mut u_char).offset(0 as ::core::ffi::c_int as isize)
                    as *mut u_char as *const ::core::ffi::c_char,
                1 as size_t,
            );
            return 0 as ::core::ffi::c_int;
        }
        if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            == (KEYC_TYPE_UNICODE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY > 0x7f as ::core::ffi::c_ulonglong
        {
            utf8_to_data(key as utf8_char, &raw mut ud);
            input_key_write(
                b"input_key\0" as *const u8 as *const ::core::ffi::c_char,
                bev,
                &raw mut ud.data as *mut u_char as *const ::core::ffi::c_char,
                ud.size as size_t,
            );
            return 0 as ::core::ffi::c_int;
        }
    }
    if !(*s).mode & MODE_KKEYPAD != 0 {
        key &= !KEYC_KEYPAD;
    }
    if !(*s).mode & MODE_KCURSOR != 0 {
        key &= !KEYC_CURSOR;
    }
    if ike.is_null() {
        ike = input_key_get(key);
    }
    if ike.is_null()
        && key as ::core::ffi::c_ulonglong & KEYC_META != 0
        && !(key as ::core::ffi::c_ulonglong) & KEYC_IMPLIED_META != 0
    {
        ike = input_key_get(key & !KEYC_META);
    }
    if ike.is_null() && key as ::core::ffi::c_ulonglong & KEYC_CURSOR != 0 {
        ike = input_key_get(key & !KEYC_CURSOR);
    }
    if ike.is_null() && key as ::core::ffi::c_ulonglong & KEYC_KEYPAD != 0 {
        ike = input_key_get(key & !KEYC_KEYPAD);
    }
    if !ike.is_null() {
        log_debug(
            b"%s: found key 0x%llx: \"%s\"\0" as *const u8 as *const ::core::ffi::c_char,
            b"input_key\0" as *const u8 as *const ::core::ffi::c_char,
            key,
            (*ike).data,
        );
        if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            == (KEYC_TYPE_FUNCTION as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                == KEYC_PASTE_START as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                || key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == KEYC_PASTE_END as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong)
            && !(*s).mode & MODE_BRACKETPASTE != 0
        {
            return 0 as ::core::ffi::c_int;
        }
        if key as ::core::ffi::c_ulonglong & KEYC_META != 0
            && !(key as ::core::ffi::c_ulonglong) & KEYC_IMPLIED_META != 0
        {
            input_key_write(
                b"input_key\0" as *const u8 as *const ::core::ffi::c_char,
                bev,
                b"\x1B\0" as *const u8 as *const ::core::ffi::c_char,
                1 as size_t,
            );
        }
        input_key_write(
            b"input_key\0" as *const u8 as *const ::core::ffi::c_char,
            bev,
            (*ike).data,
            strlen((*ike).data),
        );
        return 0 as ::core::ffi::c_int;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
        == (KEYC_TYPE_USER as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
            << 32 as ::core::ffi::c_int
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            == (KEYC_TYPE_FUNCTION as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
        || (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int)
    {
        log_debug(
            b"%s: ignoring key 0x%llx\0" as *const u8 as *const ::core::ffi::c_char,
            b"input_key\0" as *const u8 as *const ::core::ffi::c_char,
            key,
        );
        return 0 as ::core::ffi::c_int;
    }
    match (*s).mode & EXTENDED_KEY_MODES {
        MODE_KEYS_EXTENDED_2 => return input_key_extended(bev, key),
        MODE_KEYS_EXTENDED => {
            if input_key_mode1(bev, key) == -(1 as ::core::ffi::c_int) {
                return input_key_extended(bev, key);
            }
            return 0 as ::core::ffi::c_int;
        }
        _ => return input_key_vt10x(bev, key),
    };
}
#[no_mangle]
pub unsafe extern "C" fn input_key_get_mouse(
    mut s: *mut screen,
    mut m: *mut mouse_event,
    mut x: u_int,
    mut y: u_int,
    mut rbuf: *mut *const ::core::ffi::c_char,
    mut rlen: *mut size_t,
) -> ::core::ffi::c_int {
    static mut buf: [::core::ffi::c_char; 40] = [0; 40];
    let mut len: size_t = 0;
    *rbuf = ::core::ptr::null::<::core::ffi::c_char>();
    *rlen = 0 as size_t;
    if (*m).b & MOUSE_MASK_DRAG as u_int != 0
        && (*s).mode & MOTION_MOUSE_MODES == 0 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if (*s).mode & ALL_MOUSE_MODES == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if (*m).sgr_type != ' ' as i32 as u_int {
        if (*m).sgr_b & MOUSE_MASK_DRAG as u_int != 0
            && (*m).sgr_b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
            && !(*s).mode & MODE_MOUSE_ALL != 0
        {
            return 0 as ::core::ffi::c_int;
        }
    } else if (*m).b & MOUSE_MASK_DRAG as u_int != 0
        && (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
        && (*m).lb & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
        && !(*s).mode & MODE_MOUSE_ALL != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    if (*m).sgr_type != ' ' as i32 as u_int && (*s).mode & MODE_MOUSE_SGR != 0 {
        len = xsnprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 40]>() as size_t,
            b"\x1B[<%u;%u;%u%c\0" as *const u8 as *const ::core::ffi::c_char,
            (*m).sgr_b,
            x.wrapping_add(1 as u_int),
            y.wrapping_add(1 as u_int),
            (*m).sgr_type,
        ) as size_t;
    } else if (*s).mode & MODE_MOUSE_UTF8 != 0 {
        if (*m).b > (MOUSE_PARAM_UTF8_MAX - MOUSE_PARAM_BTN_OFF) as u_int
            || x > (MOUSE_PARAM_UTF8_MAX - MOUSE_PARAM_POS_OFF) as u_int
            || y > (MOUSE_PARAM_UTF8_MAX - MOUSE_PARAM_POS_OFF) as u_int
        {
            return 0 as ::core::ffi::c_int;
        }
        len = xsnprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 40]>() as size_t,
            b"\x1B[M\0" as *const u8 as *const ::core::ffi::c_char,
        ) as size_t;
        len = len.wrapping_add(input_key_split2(
            (*m).b.wrapping_add(MOUSE_PARAM_BTN_OFF as u_int),
            (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize)
                as *mut ::core::ffi::c_char as *mut u_char,
        ));
        len = len.wrapping_add(input_key_split2(
            x.wrapping_add(MOUSE_PARAM_POS_OFF as u_int),
            (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize)
                as *mut ::core::ffi::c_char as *mut u_char,
        ));
        len = len.wrapping_add(input_key_split2(
            y.wrapping_add(MOUSE_PARAM_POS_OFF as u_int),
            (&raw mut buf as *mut ::core::ffi::c_char).offset(len as isize)
                as *mut ::core::ffi::c_char as *mut u_char,
        ));
    } else {
        if (*m).b.wrapping_add(MOUSE_PARAM_BTN_OFF as u_int) > MOUSE_PARAM_MAX as u_int {
            return 0 as ::core::ffi::c_int;
        }
        len = xsnprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 40]>() as size_t,
            b"\x1B[M\0" as *const u8 as *const ::core::ffi::c_char,
        ) as size_t;
        let fresh0 = len;
        len = len.wrapping_add(1);
        buf[fresh0 as usize] =
            (*m).b.wrapping_add(MOUSE_PARAM_BTN_OFF as u_int) as ::core::ffi::c_char;
        if x.wrapping_add(MOUSE_PARAM_POS_OFF as u_int) > MOUSE_PARAM_MAX as u_int {
            let fresh1 = len;
            len = len.wrapping_add(1);
            buf[fresh1 as usize] = MOUSE_PARAM_MAX as ::core::ffi::c_char;
        } else {
            let fresh2 = len;
            len = len.wrapping_add(1);
            buf[fresh2 as usize] =
                x.wrapping_add(MOUSE_PARAM_POS_OFF as u_int) as ::core::ffi::c_char;
        }
        if y.wrapping_add(MOUSE_PARAM_POS_OFF as u_int) > MOUSE_PARAM_MAX as u_int {
            let fresh3 = len;
            len = len.wrapping_add(1);
            buf[fresh3 as usize] = MOUSE_PARAM_MAX as ::core::ffi::c_char;
        } else {
            let fresh4 = len;
            len = len.wrapping_add(1);
            buf[fresh4 as usize] =
                y.wrapping_add(MOUSE_PARAM_POS_OFF as u_int) as ::core::ffi::c_char;
        }
    }
    *rbuf = &raw mut buf as *mut ::core::ffi::c_char;
    *rlen = len;
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_key_mouse(mut wp: *mut window_pane, mut m: *mut mouse_event) {
    let mut s: *mut screen = (*wp).screen;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut buf: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    if (*m).ignore != 0 || (*s).mode & ALL_MOUSE_MODES == 0 as ::core::ffi::c_int {
        return;
    }
    if cmd_mouse_at(wp, m, &raw mut x, &raw mut y, 0 as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        return;
    }
    if window_pane_is_visible(wp) == 0 {
        return;
    }
    if input_key_get_mouse(s, m, x, y, &raw mut buf, &raw mut len) == 0 {
        return;
    }
    log_debug(
        b"writing mouse %.*s to %%%u\0" as *const u8 as *const ::core::ffi::c_char,
        len as ::core::ffi::c_int,
        buf,
        (*wp).id,
    );
    input_key_write(
        b"input_key_mouse\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).event,
        buf,
        len,
    );
}

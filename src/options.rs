pub use crate::src::shared::pane::{
    PANE_CHANGED, PANE_STYLECHANGED, PANE_THEMECHANGED, window_pane_offset, window_pane_resize,
    window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::tty::{TTY_OPENED};
pub use crate::src::shared::tree::{RB_BLACK, RB_NEGINF, RB_RED};
pub use crate::src::shared::options::{
    OPTIONS_TABLE_IS_ARRAY, OPTIONS_TABLE_IS_COLOUR, OPTIONS_TABLE_IS_STYLE, OPTIONS_TABLE_NONE,
    OPTIONS_TABLE_PANE, OPTIONS_TABLE_SERVER, OPTIONS_TABLE_SESSION, OPTIONS_TABLE_WINDOW,
};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::options::*;
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
    pub type environ;
    pub type cmds;
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
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn fnmatch(
        __pattern: *const ::core::ffi::c_char,
        __name: *const ::core::ffi::c_char,
        __flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
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
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strsep(
        __stringp: *mut *mut ::core::ffi::c_char,
        __delim: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn strtonum(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xstrndup(_: *const ::core::ffi::c_char, _: size_t) -> *mut ::core::ffi::c_char;
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
    fn xsnprintf(
        _: *mut ::core::ffi::c_char,
        _: size_t,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    static mut global_options: *mut options;
    static mut global_s_options: *mut options;
    static mut global_w_options: *mut options;
    fn checkshell(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn format_expand(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn hooks_monitor_free(_: *mut ::core::ffi::c_void);
    static options_table: [options_table_entry; 0];
    static options_other_names: [options_name_map; 0];
    fn tty_invalidate(_: *mut tty);
    fn tty_keys_build(_: *mut tty);
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn cmd_list_free(_: *mut cmd_list);
    fn cmd_list_print(_: *const cmd_list, _: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn cmd_parse_from_string(
        _: *const ::core::ffi::c_char,
        _: *mut cmd_parse_input,
    ) -> *mut cmd_parse_result;
    fn key_string_lookup_string(_: *const ::core::ffi::c_char) -> key_code;
    fn key_string_lookup_key(_: key_code, _: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn alerts_reset_all();
    static mut clients: clients;
    static mut current_time: time_t;
    fn server_client_set_key_table(_: *mut client, _: *const ::core::ffi::c_char);
    fn server_redraw_client(_: *mut client);
    fn server_client_update_theme_colours(_: *mut client);
    fn status_timer_start_all();
    fn status_update_cache(_: *mut session);
    fn recalculate_sizes();
    fn input_set_buffer_size(_: size_t);
    fn colour_tostring(_: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn colour_fromstring(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn colour_palette_from_option(_: *mut colour_palette, _: *mut options);
    static grid_default_cell: grid_cell;
    fn redraw_invalidate_all_scenes();
    static mut windows: windows;
    static mut all_window_panes: window_pane_tree;
    fn windows_RB_NEXT(_: *mut window) -> *mut window;
    fn windows_RB_MINMAX(_: *mut windows, _: ::core::ffi::c_int) -> *mut window;
    fn window_pane_tree_RB_MINMAX(
        _: *mut window_pane_tree,
        _: ::core::ffi::c_int,
    ) -> *mut window_pane;
    fn window_pane_tree_RB_NEXT(_: *mut window_pane) -> *mut window_pane;
    fn window_pane_default_cursor(_: *mut window_pane);
    fn window_pane_scrollbar_hide(_: *mut window_pane);
    fn window_set_fill_cells(_: *mut window);
    fn layout_fix_panes(_: *mut window, _: *mut window_pane);
    static mut sessions: sessions;
    fn sessions_RB_NEXT(_: *mut session) -> *mut session;
    fn sessions_RB_MINMAX(_: *mut sessions, _: ::core::ffi::c_int) -> *mut session;
    fn session_update_history(_: *mut session);
    fn utf8_update_width_cache();
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
    fn style_parse(
        _: *mut style,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn style_parse_colour(
        _: *mut style,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn style_set(_: *mut style, _: *const grid_cell);
    fn style_set_scrollbar_style_from_option(_: *mut style, _: *mut options);
}
pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2RustUnnamed = 8;
pub const _ISpunct: C2RustUnnamed = 4;
pub const _IScntrl: C2RustUnnamed = 2;
pub const _ISblank: C2RustUnnamed = 1;
pub const _ISgraph: C2RustUnnamed = 32768;
pub const _ISprint: C2RustUnnamed = 16384;
pub const _ISspace: C2RustUnnamed = 8192;
pub const _ISxdigit: C2RustUnnamed = 4096;
pub const _ISdigit: C2RustUnnamed = 2048;
pub const _ISalpha: C2RustUnnamed = 1024;
pub const _ISlower: C2RustUnnamed = 512;
pub const _ISupper: C2RustUnnamed = 256;
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
    pub entry: C2RustUnnamed_11,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_11 {
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
    pub entry: C2RustUnnamed_12,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_12 {
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
pub union C2RustUnnamed_13 {
    pub offset: u_int,
    pub data: C2RustUnnamed_14,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_14 {
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
pub struct options {
    pub tree: options_tree,
    pub parent: *mut options,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_tree {
    pub rbh_root: *mut options_entry,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_entry {
    pub owner: *mut options,
    pub name: *const ::core::ffi::c_char,
    pub tableentry: *const options_table_entry,
    pub value: options_value,
    pub cached: ::core::ffi::c_int,
    pub style: style,
    pub monitor_data: *mut ::core::ffi::c_void,
    pub fire_count: u_int,
    pub fire_time: time_t,
    pub entry: C2RustUnnamed_17,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_17 {
    pub rbe_left: *mut options_entry,
    pub rbe_right: *mut options_entry,
    pub rbe_parent: *mut options_entry,
    pub rbe_color: ::core::ffi::c_int,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_list {
    pub references: ::core::ffi::c_int,
    pub group: u_int,
    pub list: *mut cmds,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_array {
    pub rbh_root: *mut options_array_item,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_array_item {
    pub key: *mut ::core::ffi::c_char,
    pub value: options_value,
    pub entry: C2RustUnnamed_18,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_18 {
    pub rbe_left: *mut options_array_item,
    pub rbe_right: *mut options_array_item,
    pub rbe_parent: *mut options_array_item,
    pub rbe_color: ::core::ffi::c_int,
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
    pub entry: C2RustUnnamed_21,
    pub wentry: C2RustUnnamed_20,
    pub sentry: C2RustUnnamed_19,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_19 {
    pub tqe_next: *mut winlink,
    pub tqe_prev: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_20 {
    pub tqe_next: *mut winlink,
    pub tqe_prev: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_21 {
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
    pub alerts_entry: C2RustUnnamed_24,
    pub options: *mut options,
    pub references: u_int,
    pub winlinks: C2RustUnnamed_23,
    pub entry: C2RustUnnamed_22,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_22 {
    pub rbe_left: *mut window,
    pub rbe_right: *mut window,
    pub rbe_parent: *mut window,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_23 {
    pub tqh_first: *mut winlink,
    pub tqh_last: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_24 {
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
    pub entry: C2RustUnnamed_25,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_25 {
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
    pub modes: C2RustUnnamed_30,
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
    pub entry: C2RustUnnamed_29,
    pub sentry: C2RustUnnamed_28,
    pub zentry: C2RustUnnamed_27,
    pub tree_entry: C2RustUnnamed_26,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_26 {
    pub rbe_left: *mut window_pane,
    pub rbe_right: *mut window_pane,
    pub rbe_parent: *mut window_pane,
    pub rbe_color: ::core::ffi::c_int,
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
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_29 {
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_30 {
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
    pub entry: C2RustUnnamed_31,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_31 {
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
    pub entry: C2RustUnnamed_34,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_34 {
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
    pub entry: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub rbe_left: *mut key_binding,
    pub rbe_right: *mut key_binding,
    pub rbe_parent: *mut key_binding,
    pub rbe_color: ::core::ffi::c_int,
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
    pub entry: C2RustUnnamed_37,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
    pub le_next: *mut tty_term,
    pub le_prev: *mut *mut tty_term,
}
pub type C2RustUnnamed_38 = ::core::ffi::c_ulong;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_pane_tree {
    pub rbh_root: *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct windows {
    pub rbh_root: *mut window,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sessions {
    pub rbh_root: *mut session,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cmd_parse_result {
    pub status: cmd_parse_status,
    pub cmdlist: *mut cmd_list,
    pub error: *mut ::core::ffi::c_char,
}
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct clients {
    pub tqh_first: *mut client,
    pub tqh_last: *mut *mut client,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct options_name_map {
    pub from: *const ::core::ffi::c_char,
    pub to: *const ::core::ffi::c_char,
}
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
unsafe extern "C" fn options_array_key_to_number(
    mut key: *const ::core::ffi::c_char,
    mut idx: *mut u_int,
) -> ::core::ffi::c_int {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut n: ::core::ffi::c_longlong = 0;
    if *key as ::core::ffi::c_int == '\0' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    let mut cp: *const ::core::ffi::c_char = key;
    while *cp as ::core::ffi::c_int != '\0' as i32 {
        if *(*__ctype_b_loc()).offset(*cp as u_char as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            == 0
        {
            return 0 as ::core::ffi::c_int;
        }
        cp = cp.offset(1);
    }
    n = strtonum(
        key,
        0 as ::core::ffi::c_longlong,
        UINT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    );
    if !errstr.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if !idx.is_null() {
        *idx = n as u_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn options_array_correct_key(
    mut key: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut idx: u_int = 0;
    let mut numeric: ::core::ffi::c_int = 0;
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    numeric = options_array_key_to_number(key, &raw mut idx);
    if numeric == -(1 as ::core::ffi::c_int) {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if numeric == 1 as ::core::ffi::c_int {
        xasprintf(
            &raw mut out,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            idx,
        );
        return out;
    }
    return xstrdup(key);
}
unsafe extern "C" fn options_array_cmp(
    mut a1: *mut options_array_item,
    mut a2: *mut options_array_item,
) -> ::core::ffi::c_int {
    let mut i1: u_int = 0;
    let mut i2: u_int = 0;
    let mut n1: ::core::ffi::c_int = 0;
    let mut n2: ::core::ffi::c_int = 0;
    n1 = options_array_key_to_number((*a1).key, &raw mut i1);
    n2 = options_array_key_to_number((*a2).key, &raw mut i2);
    if n1 != 0 && n2 != 0 {
        if i1 < i2 {
            return -(1 as ::core::ffi::c_int);
        }
        if i1 > i2 {
            return 1 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
    if n1 != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if n2 != 0 {
        return 1 as ::core::ffi::c_int;
    }
    return strcmp((*a1).key, (*a2).key);
}
unsafe extern "C" fn options_array_RB_REMOVE(
    mut head: *mut options_array,
    mut elm: *mut options_array_item,
) -> *mut options_array_item {
    let mut current_block: u64;
    let mut child: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut parent: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut old: *mut options_array_item = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
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
        current_block = 5235237659983604069;
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
        options_array_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn options_array_RB_MINMAX(
    mut head: *mut options_array,
    mut val: ::core::ffi::c_int,
) -> *mut options_array_item {
    let mut tmp: *mut options_array_item = (*head).rbh_root;
    let mut parent: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
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
unsafe extern "C" fn options_array_RB_REMOVE_COLOR(
    mut head: *mut options_array,
    mut parent: *mut options_array_item,
    mut elm: *mut options_array_item,
) {
    let mut tmp: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
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
                    let mut oleft: *mut options_array_item =
                        ::core::ptr::null_mut::<options_array_item>();
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
                    let mut oright: *mut options_array_item =
                        ::core::ptr::null_mut::<options_array_item>();
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
unsafe extern "C" fn options_array_RB_FIND(
    mut head: *mut options_array,
    mut elm: *mut options_array_item,
) -> *mut options_array_item {
    let mut tmp: *mut options_array_item = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = options_array_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<options_array_item>();
}
unsafe extern "C" fn options_array_RB_INSERT(
    mut head: *mut options_array,
    mut elm: *mut options_array_item,
) -> *mut options_array_item {
    let mut tmp: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut parent: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = options_array_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<options_array_item>();
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
    options_array_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<options_array_item>();
}
unsafe extern "C" fn options_array_RB_INSERT_COLOR(
    mut head: *mut options_array,
    mut elm: *mut options_array_item,
) {
    let mut parent: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut gparent: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut tmp: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
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
unsafe extern "C" fn options_array_RB_NEXT(
    mut elm: *mut options_array_item,
) -> *mut options_array_item {
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
unsafe extern "C" fn options_tree_RB_NEXT(mut elm: *mut options_entry) -> *mut options_entry {
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
unsafe extern "C" fn options_tree_RB_MINMAX(
    mut head: *mut options_tree,
    mut val: ::core::ffi::c_int,
) -> *mut options_entry {
    let mut tmp: *mut options_entry = (*head).rbh_root;
    let mut parent: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
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
unsafe extern "C" fn options_tree_RB_INSERT(
    mut head: *mut options_tree,
    mut elm: *mut options_entry,
) -> *mut options_entry {
    let mut tmp: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut parent: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = options_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<options_entry>();
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
    options_tree_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<options_entry>();
}
unsafe extern "C" fn options_tree_RB_REMOVE_COLOR(
    mut head: *mut options_tree,
    mut parent: *mut options_entry,
    mut elm: *mut options_entry,
) {
    let mut tmp: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
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
                    let mut oleft: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
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
                    let mut oright: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
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
unsafe extern "C" fn options_tree_RB_INSERT_COLOR(
    mut head: *mut options_tree,
    mut elm: *mut options_entry,
) {
    let mut parent: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut gparent: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut tmp: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
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
unsafe extern "C" fn options_tree_RB_REMOVE(
    mut head: *mut options_tree,
    mut elm: *mut options_entry,
) -> *mut options_entry {
    let mut current_block: u64;
    let mut child: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut parent: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut old: *mut options_entry = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
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
        current_block = 3778725392729552373;
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
        options_tree_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
unsafe extern "C" fn options_tree_RB_FIND(
    mut head: *mut options_tree,
    mut elm: *mut options_entry,
) -> *mut options_entry {
    let mut tmp: *mut options_entry = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = options_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<options_entry>();
}
unsafe extern "C" fn options_cmp(
    mut lhs: *mut options_entry,
    mut rhs: *mut options_entry,
) -> ::core::ffi::c_int {
    return strcmp((*lhs).name, (*rhs).name);
}
unsafe extern "C" fn options_map_name(
    mut name: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut map: *const options_name_map = ::core::ptr::null::<options_name_map>();
    map = &raw const options_other_names as *const options_name_map;
    while !(*map).from.is_null() {
        if strcmp((*map).from, name) == 0 as ::core::ffi::c_int {
            return (*map).to;
        }
        map = map.offset(1);
    }
    return name;
}
unsafe extern "C" fn options_parent_table_entry(
    mut oo: *mut options,
    mut s: *const ::core::ffi::c_char,
) -> *const options_table_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if (*oo).parent.is_null() {
        fatalx(
            b"no parent options for %s\0" as *const u8 as *const ::core::ffi::c_char,
            s,
        );
    }
    o = options_get((*oo).parent, s);
    if o.is_null() {
        fatalx(
            b"%s not in parent options\0" as *const u8 as *const ::core::ffi::c_char,
            s,
        );
    }
    return (*o).tableentry;
}
unsafe extern "C" fn options_value_free(mut o: *mut options_entry, mut ov: *mut options_value) {
    if (*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        free((*ov).string as *mut ::core::ffi::c_void);
    }
    if !(*o).tableentry.is_null()
        && (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*ov).cmdlist.is_null()
    {
        cmd_list_free((*ov).cmdlist);
    }
}
unsafe extern "C" fn options_value_to_string(
    mut o: *mut options_entry,
    mut ov: *mut options_value,
    mut numeric: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(*o).tableentry.is_null()
        && (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return cmd_list_print((*ov).cmdlist, 0 as ::core::ffi::c_int);
    }
    if !(*o).tableentry.is_null()
        && ((*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_COLOUR as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        match (*(*o).tableentry).type_0 as ::core::ffi::c_uint {
            1 => {
                xasprintf(
                    &raw mut s,
                    b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                    (*ov).number,
                );
            }
            2 => {
                s = xstrdup(key_string_lookup_key(
                    (*ov).number as key_code,
                    0 as ::core::ffi::c_int,
                ));
            }
            3 => {
                s = xstrdup(colour_tostring((*ov).number as ::core::ffi::c_int));
            }
            4 => {
                if numeric != 0 {
                    xasprintf(
                        &raw mut s,
                        b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                        (*ov).number,
                    );
                } else {
                    s = xstrdup(if (*ov).number != 0 {
                        b"on\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        b"off\0" as *const u8 as *const ::core::ffi::c_char
                    });
                }
            }
            5 => {
                s = xstrdup(*(*(*o).tableentry).choices.offset((*ov).number as isize));
            }
            _ => {
                fatalx(b"not a number option type\0" as *const u8 as *const ::core::ffi::c_char);
            }
        }
        return s;
    }
    if (*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return xstrdup((*ov).string);
    }
    return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn options_create(mut parent: *mut options) -> *mut options {
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    oo = xcalloc(1 as size_t, ::core::mem::size_of::<options>() as size_t) as *mut options;
    (*oo).tree.rbh_root = ::core::ptr::null_mut::<options_entry>();
    (*oo).parent = parent;
    return oo;
}
#[no_mangle]
pub unsafe extern "C" fn options_free(mut oo: *mut options) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut tmp: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_tree_RB_MINMAX(&raw mut (*oo).tree, RB_NEGINF);
    while !o.is_null() && {
        tmp = options_tree_RB_NEXT(o);
        1 as ::core::ffi::c_int != 0
    } {
        options_remove(o);
        o = tmp;
    }
    free(oo as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn options_get_parent(mut oo: *mut options) -> *mut options {
    return (*oo).parent;
}
#[no_mangle]
pub unsafe extern "C" fn options_set_parent(mut oo: *mut options, mut parent: *mut options) {
    (*oo).parent = parent;
}
#[no_mangle]
pub unsafe extern "C" fn options_first(mut oo: *mut options) -> *mut options_entry {
    return options_tree_RB_MINMAX(&raw mut (*oo).tree, RB_NEGINF);
}
#[no_mangle]
pub unsafe extern "C" fn options_next(mut o: *mut options_entry) -> *mut options_entry {
    return options_tree_RB_NEXT(o);
}
#[no_mangle]
pub unsafe extern "C" fn options_get_only(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *mut options_entry {
    let mut o: options_entry = options_entry {
        owner: ::core::ptr::null_mut::<options>(),
        name: name,
        tableentry: ::core::ptr::null::<options_table_entry>(),
        value: options_value {
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        cached: 0,
        style: style {
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
        },
        monitor_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        fire_count: 0,
        fire_time: 0,
        entry: C2RustUnnamed_17 {
            rbe_left: ::core::ptr::null_mut::<options_entry>(),
            rbe_right: ::core::ptr::null_mut::<options_entry>(),
            rbe_parent: ::core::ptr::null_mut::<options_entry>(),
            rbe_color: 0,
        },
    };
    let mut found: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    found = options_tree_RB_FIND(&raw mut (*oo).tree, &raw mut o);
    if found.is_null() {
        o.name = options_map_name(name);
        return options_tree_RB_FIND(&raw mut (*oo).tree, &raw mut o);
    }
    return found;
}
#[no_mangle]
pub unsafe extern "C" fn options_get(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_get_only(oo, name);
    while o.is_null() {
        oo = (*oo).parent;
        if oo.is_null() {
            break;
        }
        o = options_get_only(oo, name);
    }
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_empty(
    mut oo: *mut options,
    mut oe: *const options_table_entry,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_add(oo, (*oe).name);
    (*o).tableentry = oe;
    if (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        (*o).value.array.rbh_root = ::core::ptr::null_mut::<options_array_item>();
    }
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_default(
    mut oo: *mut options,
    mut oe: *const options_table_entry,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut key: [::core::ffi::c_char; 32] = [0; 32];
    let mut i: u_int = 0;
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    o = options_empty(oo, oe);
    ov = &raw mut (*o).value;
    if (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        if (*oe).default_arr.is_null() {
            options_array_assign(
                o,
                (*oe).default_str,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
            return o;
        }
        i = 0 as u_int;
        while !(*(*oe).default_arr.offset(i as isize)).is_null() {
            xsnprintf(
                &raw mut key as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                i,
            );
            options_array_set(
                o,
                &raw mut key as *mut ::core::ffi::c_char,
                *(*oe).default_arr.offset(i as isize),
                0 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
            i = i.wrapping_add(1);
        }
        return o;
    }
    match (*oe).type_0 as ::core::ffi::c_uint {
        0 => {
            (*ov).string = xstrdup((*oe).default_str);
        }
        6 => {
            pr = cmd_parse_from_string(
                (*oe).default_str,
                ::core::ptr::null_mut::<cmd_parse_input>(),
            );
            match (*pr).status as ::core::ffi::c_uint {
                0 => {
                    free((*pr).error as *mut ::core::ffi::c_void);
                }
                1 => {
                    (*ov).cmdlist = (*pr).cmdlist;
                }
                _ => {}
            }
        }
        _ => {
            (*ov).number = (*oe).default_num;
        }
    }
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_default_to_string(
    mut oe: *const options_table_entry,
) -> *mut ::core::ffi::c_char {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    match (*oe).type_0 as ::core::ffi::c_uint {
        0 | 6 => {
            s = xstrdup((*oe).default_str);
        }
        1 => {
            xasprintf(
                &raw mut s,
                b"%lld\0" as *const u8 as *const ::core::ffi::c_char,
                (*oe).default_num,
            );
        }
        2 => {
            s = xstrdup(key_string_lookup_key(
                (*oe).default_num as key_code,
                0 as ::core::ffi::c_int,
            ));
        }
        3 => {
            s = xstrdup(colour_tostring((*oe).default_num as ::core::ffi::c_int));
        }
        4 => {
            s = xstrdup(if (*oe).default_num != 0 {
                b"on\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"off\0" as *const u8 as *const ::core::ffi::c_char
            });
        }
        5 => {
            s = xstrdup(*(*oe).choices.offset((*oe).default_num as isize));
        }
        _ => {
            fatalx(b"unknown option type\0" as *const u8 as *const ::core::ffi::c_char);
        }
    }
    return s;
}
unsafe extern "C" fn options_add(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_get_only(oo, name);
    if !o.is_null() {
        options_remove(o);
    }
    o = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<options_entry>() as size_t,
    ) as *mut options_entry;
    (*o).owner = oo;
    (*o).name = xstrdup(name);
    options_tree_RB_INSERT(&raw mut (*oo).tree, o);
    return o;
}
unsafe extern "C" fn options_remove(mut o: *mut options_entry) {
    let mut oo: *mut options = (*o).owner;
    if !(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        options_array_clear(o);
    } else {
        options_value_free(o, &raw mut (*o).value);
    }
    if !(*o).monitor_data.is_null() {
        hooks_monitor_free((*o).monitor_data);
    }
    options_tree_RB_REMOVE(&raw mut (*oo).tree, o);
    free((*o).name as *mut ::core::ffi::c_void);
    free(o as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn options_name(mut o: *mut options_entry) -> *const ::core::ffi::c_char {
    return (*o).name;
}
#[no_mangle]
pub unsafe extern "C" fn options_owner(mut o: *mut options_entry) -> *mut options {
    return (*o).owner;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_monitor_data(
    mut o: *mut options_entry,
) -> *mut ::core::ffi::c_void {
    return (*o).monitor_data;
}
#[no_mangle]
pub unsafe extern "C" fn options_set_monitor_data(
    mut o: *mut options_entry,
    mut data: *mut ::core::ffi::c_void,
) {
    (*o).monitor_data = data;
}
#[no_mangle]
pub unsafe extern "C" fn options_hook_fired(mut o: *mut options_entry) {
    (*o).fire_count = (*o).fire_count.wrapping_add(1);
    (*o).fire_time = current_time;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_fire_count(mut o: *mut options_entry) -> u_int {
    return (*o).fire_count;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_fire_time(mut o: *mut options_entry) -> time_t {
    return (*o).fire_time;
}
#[no_mangle]
pub unsafe extern "C" fn options_table_entry(
    mut o: *mut options_entry,
) -> *const options_table_entry {
    return (*o).tableentry;
}
unsafe extern "C" fn options_array_item(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
) -> *mut options_array_item {
    let mut a: options_array_item = options_array_item {
        key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        value: options_value {
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        entry: C2RustUnnamed_18 {
            rbe_left: ::core::ptr::null_mut::<options_array_item>(),
            rbe_right: ::core::ptr::null_mut::<options_array_item>(),
            rbe_parent: ::core::ptr::null_mut::<options_array_item>(),
            rbe_color: 0,
        },
    };
    a.key = key as *mut ::core::ffi::c_char;
    return options_array_RB_FIND(&raw mut (*o).value.array, &raw mut a);
}
unsafe extern "C" fn options_array_new(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
) -> *mut options_array_item {
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    a = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<options_array_item>() as size_t,
    ) as *mut options_array_item;
    (*a).key = xstrdup(key);
    options_array_RB_INSERT(&raw mut (*o).value.array, a);
    return a;
}
unsafe extern "C" fn options_array_free(mut o: *mut options_entry, mut a: *mut options_array_item) {
    options_value_free(o, &raw mut (*a).value);
    options_array_RB_REMOVE(&raw mut (*o).value.array, a);
    free((*a).key as *mut ::core::ffi::c_void);
    free(a as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn options_array_clear(mut o: *mut options_entry) {
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut a1: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    if !(!(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0) {
        return;
    }
    a = options_array_RB_MINMAX(&raw mut (*o).value.array, RB_NEGINF);
    while !a.is_null() && {
        a1 = options_array_RB_NEXT(a);
        1 as ::core::ffi::c_int != 0
    } {
        options_array_free(o, a);
        a = a1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn options_array_get(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
) -> *mut options_value {
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut new_key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(!(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0) {
        return ::core::ptr::null_mut::<options_value>();
    }
    new_key = options_array_correct_key(key);
    if new_key.is_null() {
        return ::core::ptr::null_mut::<options_value>();
    }
    a = options_array_item(o, new_key);
    free(new_key as *mut ::core::ffi::c_void);
    if a.is_null() {
        return ::core::ptr::null_mut::<options_value>();
    }
    return &raw mut (*a).value;
}
#[no_mangle]
pub unsafe extern "C" fn options_array_getv(
    mut o: *mut options_entry,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> *mut options_value {
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut ap: ::core::ffi::VaList;
    let mut key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ap = args.clone();
    xvasprintf(&raw mut key, fmt, ap);
    ov = options_array_get(o, key);
    free(key as *mut ::core::ffi::c_void);
    return ov;
}
#[no_mangle]
pub unsafe extern "C" fn options_array_set(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut append: ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut new: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut new_key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut number: ::core::ffi::c_longlong = 0;
    if !(!(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0) {
        if !cause.is_null() {
            *cause = xstrdup(b"not an array\0" as *const u8 as *const ::core::ffi::c_char);
        }
        return -(1 as ::core::ffi::c_int);
    }
    new_key = options_array_correct_key(key);
    if new_key.is_null() {
        if !cause.is_null() {
            xasprintf(
                cause,
                b"bad array key: %s\0" as *const u8 as *const ::core::ffi::c_char,
                key,
            );
        }
        return -(1 as ::core::ffi::c_int);
    }
    if value.is_null() {
        a = options_array_item(o, new_key);
        if !a.is_null() {
            options_array_free(o, a);
        }
        free(new_key as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if !(*o).tableentry.is_null()
        && (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        pr = cmd_parse_from_string(value, ::core::ptr::null_mut::<cmd_parse_input>());
        match (*pr).status as ::core::ffi::c_uint {
            0 => {
                if !cause.is_null() {
                    *cause = (*pr).error;
                } else {
                    free((*pr).error as *mut ::core::ffi::c_void);
                }
                free(new_key as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            1 | _ => {}
        }
        a = options_array_item(o, new_key);
        if a.is_null() {
            a = options_array_new(o, new_key);
        } else {
            options_value_free(o, &raw mut (*a).value);
        }
        (*a).value.cmdlist = (*pr).cmdlist;
        free(new_key as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if (*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        a = options_array_item(o, new_key);
        if !a.is_null() && append != 0 {
            xasprintf(
                &raw mut new,
                b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*a).value.string,
                value,
            );
        } else {
            new = xstrdup(value);
        }
        if a.is_null() {
            a = options_array_new(o, new_key);
        } else {
            options_value_free(o, &raw mut (*a).value);
        }
        (*a).value.string = new;
        free(new_key as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if (*(*o).tableentry).type_0 as ::core::ffi::c_uint
        == OPTIONS_TABLE_COLOUR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        number = colour_fromstring(value) as ::core::ffi::c_longlong;
        if number == -(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong {
            xasprintf(
                cause,
                b"bad colour: %s\0" as *const u8 as *const ::core::ffi::c_char,
                value,
            );
            free(new_key as *mut ::core::ffi::c_void);
            return -(1 as ::core::ffi::c_int);
        }
        a = options_array_item(o, new_key);
        if a.is_null() {
            a = options_array_new(o, new_key);
        } else {
            options_value_free(o, &raw mut (*a).value);
        }
        (*a).value.number = number;
        free(new_key as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    if !cause.is_null() {
        *cause = xstrdup(b"wrong array type\0" as *const u8 as *const ::core::ffi::c_char);
    }
    free(new_key as *mut ::core::ffi::c_void);
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn options_array_assign(
    mut o: *mut options_entry,
    mut s: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut separator: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut string: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: [::core::ffi::c_char; 32] = [0; 32];
    let mut i: u_int = 0;
    separator = (*(*o).tableentry).separator;
    if separator.is_null() {
        separator = b" ,\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if *separator as ::core::ffi::c_int == '\0' as i32 {
        if *s as ::core::ffi::c_int == '\0' as i32 {
            return 0 as ::core::ffi::c_int;
        }
        i = 0 as u_int;
        while i < UINT_MAX {
            if options_array_getv(o, b"%u\0" as *const u8 as *const ::core::ffi::c_char, i)
                .is_null()
            {
                break;
            }
            i = i.wrapping_add(1);
        }
        xsnprintf(
            &raw mut key as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        return options_array_set(
            o,
            &raw mut key as *mut ::core::ffi::c_char,
            s,
            0 as ::core::ffi::c_int,
            cause,
        );
    }
    if *s as ::core::ffi::c_int == '\0' as i32 {
        return 0 as ::core::ffi::c_int;
    }
    string = xstrdup(s);
    copy = string;
    loop {
        next = strsep(&raw mut string, separator);
        if next.is_null() {
            break;
        }
        if *next as ::core::ffi::c_int == '\0' as i32 {
            continue;
        }
        i = 0 as u_int;
        while i < UINT_MAX {
            if options_array_getv(o, b"%u\0" as *const u8 as *const ::core::ffi::c_char, i)
                .is_null()
            {
                break;
            }
            i = i.wrapping_add(1);
        }
        if i == UINT_MAX {
            break;
        }
        xsnprintf(
            &raw mut key as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"%u\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        if options_array_set(
            o,
            &raw mut key as *mut ::core::ffi::c_char,
            next,
            0 as ::core::ffi::c_int,
            cause,
        ) != 0 as ::core::ffi::c_int
        {
            free(copy as *mut ::core::ffi::c_void);
            return -(1 as ::core::ffi::c_int);
        }
    }
    free(copy as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn options_array_first(mut o: *mut options_entry) -> *mut options_array_item {
    if !(!(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0) {
        return ::core::ptr::null_mut::<options_array_item>();
    }
    return options_array_RB_MINMAX(&raw mut (*o).value.array, RB_NEGINF);
}
#[no_mangle]
pub unsafe extern "C" fn options_array_next(
    mut a: *mut options_array_item,
) -> *mut options_array_item {
    return options_array_RB_NEXT(a);
}
#[no_mangle]
pub unsafe extern "C" fn options_array_item_key(
    mut a: *mut options_array_item,
) -> *const ::core::ffi::c_char {
    return (*a).key;
}
#[no_mangle]
pub unsafe extern "C" fn options_array_item_value(
    mut a: *mut options_array_item,
) -> *mut options_value {
    return &raw mut (*a).value;
}
#[no_mangle]
pub unsafe extern "C" fn options_is_array(mut o: *mut options_entry) -> ::core::ffi::c_int {
    return (!(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn options_is_string(mut o: *mut options_entry) -> ::core::ffi::c_int {
    return ((*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn options_to_string(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
    mut numeric: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut result: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut last: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut new_key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(*o).tableentry.is_null() && (*(*o).tableentry).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        if key.is_null() {
            a = options_array_RB_MINMAX(&raw mut (*o).value.array, RB_NEGINF);
            while !a.is_null() {
                next = options_value_to_string(o, &raw mut (*a).value, numeric);
                if last.is_null() {
                    result = next;
                } else {
                    xasprintf(
                        &raw mut result,
                        b"%s %s\0" as *const u8 as *const ::core::ffi::c_char,
                        last,
                        next,
                    );
                    free(last as *mut ::core::ffi::c_void);
                    free(next as *mut ::core::ffi::c_void);
                }
                last = result;
                a = options_array_RB_NEXT(a);
            }
            if result.is_null() {
                return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
            }
            return result;
        }
        new_key = options_array_correct_key(key);
        if new_key.is_null() {
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        }
        a = options_array_item(o, new_key);
        free(new_key as *mut ::core::ffi::c_void);
        if a.is_null() {
            return xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        }
        return options_value_to_string(o, &raw mut (*a).value, numeric);
    }
    return options_value_to_string(o, &raw mut (*o).value, numeric);
}
#[no_mangle]
pub unsafe extern "C" fn options_parse(
    mut name: *const ::core::ffi::c_char,
    mut key: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut raw: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut new_key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *name as ::core::ffi::c_int == '\0' as i32 {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    *key = ::core::ptr::null_mut::<::core::ffi::c_char>();
    copy = xstrdup(name);
    cp = strchr(copy, '[' as i32);
    if cp.is_null() {
        return copy;
    }
    end = strchr(cp.offset(1 as ::core::ffi::c_int as isize), ']' as i32);
    if end.is_null()
        || *end.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
        || end == cp.offset(1 as ::core::ffi::c_int as isize)
    {
        free(copy as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    raw = xstrndup(
        cp.offset(1 as ::core::ffi::c_int as isize),
        end.offset_from(cp.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_long
            as size_t,
    );
    new_key = options_array_correct_key(raw);
    free(raw as *mut ::core::ffi::c_void);
    if new_key.is_null() {
        free(copy as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    *key = new_key;
    *cp = '\0' as i32 as ::core::ffi::c_char;
    return copy;
}
#[no_mangle]
pub unsafe extern "C" fn options_parse_get(
    mut oo: *mut options,
    mut s: *const ::core::ffi::c_char,
    mut key: *mut *mut ::core::ffi::c_char,
    mut only: ::core::ffi::c_int,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    name = options_parse(s, key);
    if name.is_null() {
        return ::core::ptr::null_mut::<options_entry>();
    }
    if only != 0 {
        o = options_get_only(oo, name);
    } else {
        o = options_get(oo, name);
    }
    free(name as *mut ::core::ffi::c_void);
    if o.is_null() {
        free(*key as *mut ::core::ffi::c_void);
        *key = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_search(
    mut name: *const ::core::ffi::c_char,
) -> *const options_table_entry {
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    oe = &raw const options_table as *const options_table_entry;
    while !(*oe).name.is_null() {
        if strcmp((*oe).name, name) == 0 as ::core::ffi::c_int {
            return oe;
        }
        oe = oe.offset(1);
    }
    return ::core::ptr::null::<options_table_entry>();
}
#[no_mangle]
pub unsafe extern "C" fn options_match(
    mut s: *const ::core::ffi::c_char,
    mut key: *mut *mut ::core::ffi::c_char,
    mut ambiguous: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut found: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut parsed: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut namelen: size_t = 0;
    parsed = options_parse(s, key);
    if parsed.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if *parsed as ::core::ffi::c_int == '@' as i32 {
        *ambiguous = 0 as ::core::ffi::c_int;
        return parsed;
    }
    name = options_map_name(parsed);
    namelen = strlen(name);
    found = ::core::ptr::null::<options_table_entry>();
    oe = &raw const options_table as *const options_table_entry;
    while !(*oe).name.is_null() {
        if strcmp((*oe).name, name) == 0 as ::core::ffi::c_int {
            found = oe;
            break;
        } else {
            if strncmp((*oe).name, name, namelen) == 0 as ::core::ffi::c_int {
                if !found.is_null() {
                    *ambiguous = 1 as ::core::ffi::c_int;
                    free(parsed as *mut ::core::ffi::c_void);
                    free(*key as *mut ::core::ffi::c_void);
                    *key = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                found = oe;
            }
            oe = oe.offset(1);
        }
    }
    free(parsed as *mut ::core::ffi::c_void);
    if found.is_null() {
        *ambiguous = 0 as ::core::ffi::c_int;
        free(*key as *mut ::core::ffi::c_void);
        *key = ::core::ptr::null_mut::<::core::ffi::c_char>();
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return xstrdup((*found).name);
}
#[no_mangle]
pub unsafe extern "C" fn options_match_get(
    mut oo: *mut options,
    mut s: *const ::core::ffi::c_char,
    mut key: *mut *mut ::core::ffi::c_char,
    mut only: ::core::ffi::c_int,
    mut ambiguous: *mut ::core::ffi::c_int,
) -> *mut options_entry {
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    name = options_match(s, key, ambiguous);
    if name.is_null() {
        return ::core::ptr::null_mut::<options_entry>();
    }
    *ambiguous = 0 as ::core::ffi::c_int;
    if only != 0 {
        o = options_get_only(oo, name);
    } else {
        o = options_get(oo, name);
    }
    free(name as *mut ::core::ffi::c_void);
    if o.is_null() {
        free(*key as *mut ::core::ffi::c_void);
        *key = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_string(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_get(oo, name);
    if o.is_null() {
        fatalx(
            b"missing option %s\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    if !((*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        fatalx(
            b"option %s is not a string\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    return (*o).value.string;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_number(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_longlong {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_get(oo, name);
    if o.is_null() {
        fatalx(
            b"missing option %s\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    if !(!(*o).tableentry.is_null()
        && ((*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_COLOUR as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint))
    {
        fatalx(
            b"option %s is not a number\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    return (*o).value.number;
}
#[no_mangle]
pub unsafe extern "C" fn options_get_command(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
) -> *mut cmd_list {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    o = options_get(oo, name);
    if o.is_null() {
        fatalx(
            b"missing option %s\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    if !(!(*o).tableentry.is_null()
        && (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        fatalx(
            b"option %s is not a command\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    return (*o).value.cmdlist;
}
#[no_mangle]
pub unsafe extern "C" fn options_set_string(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut append: ::core::ffi::c_int,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut ap: ::core::ffi::VaList;
    let mut separator: *const ::core::ffi::c_char =
        b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ap = args.clone();
    xvasprintf(&raw mut s, fmt, ap);
    o = options_get_only(oo, name);
    if !o.is_null()
        && append != 0
        && ((*o).tableentry.is_null()
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        if *name as ::core::ffi::c_int != '@' as i32 {
            separator = (*(*o).tableentry).separator;
            if separator.is_null() {
                separator = b"\0" as *const u8 as *const ::core::ffi::c_char;
            }
        }
        xasprintf(
            &raw mut value,
            b"%s%s%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*o).value.string,
            separator,
            s,
        );
        free(s as *mut ::core::ffi::c_void);
    } else {
        value = s;
    }
    if o.is_null() && *name as ::core::ffi::c_int == '@' as i32 {
        o = options_add(oo, name);
    } else if o.is_null() {
        o = options_default(oo, options_parent_table_entry(oo, name));
        if o.is_null() {
            return ::core::ptr::null_mut::<options_entry>();
        }
    }
    if !((*o).tableentry.is_null()
        || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        fatalx(
            b"option %s is not a string\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    free((*o).value.string as *mut ::core::ffi::c_void);
    (*o).value.string = value;
    (*o).cached = 0 as ::core::ffi::c_int;
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_set_number(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut value: ::core::ffi::c_longlong,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if *name as ::core::ffi::c_int == '@' as i32 {
        fatalx(
            b"user option %s must be a string\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    o = options_get_only(oo, name);
    if o.is_null() {
        o = options_default(oo, options_parent_table_entry(oo, name));
        if o.is_null() {
            return ::core::ptr::null_mut::<options_entry>();
        }
    }
    if !(!(*o).tableentry.is_null()
        && ((*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_COLOUR as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint))
    {
        fatalx(
            b"option %s is not a number\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    (*o).value.number = value;
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_set_command(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut value: *mut cmd_list,
) -> *mut options_entry {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if *name as ::core::ffi::c_int == '@' as i32 {
        fatalx(
            b"user option %s must be a string\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    o = options_get_only(oo, name);
    if o.is_null() {
        o = options_default(oo, options_parent_table_entry(oo, name));
        if o.is_null() {
            return ::core::ptr::null_mut::<options_entry>();
        }
    }
    if !(!(*o).tableentry.is_null()
        && (*(*o).tableentry).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        fatalx(
            b"option %s is not a command\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
    }
    if !(*o).value.cmdlist.is_null() {
        cmd_list_free((*o).value.cmdlist);
    }
    (*o).value.cmdlist = value;
    return o;
}
#[no_mangle]
pub unsafe extern "C" fn options_scope_from_name(
    mut args: *mut args,
    mut window: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
    mut oo: *mut *mut options,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut s: *mut session = (*fs).s;
    let mut wl: *mut winlink = (*fs).wl;
    let mut wp: *mut window_pane = (*fs).wp;
    let mut target: *const ::core::ffi::c_char = args_get(args, 't' as i32 as u_char);
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut scope: ::core::ffi::c_int = OPTIONS_TABLE_NONE;
    if *name as ::core::ffi::c_int == '@' as i32 {
        return options_scope_from_flags(args, window, fs, oo, cause);
    }
    oe = &raw const options_table as *const options_table_entry;
    while !(*oe).name.is_null() {
        if strcmp((*oe).name, name) == 0 as ::core::ffi::c_int {
            break;
        }
        oe = oe.offset(1);
    }
    if (*oe).name.is_null() {
        xasprintf(
            cause,
            b"unknown option: %s\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
        return 0 as ::core::ffi::c_int;
    }
    let mut current_block_38: u64;
    match (*oe).scope {
        OPTIONS_TABLE_SERVER => {
            *oo = global_options;
            scope = OPTIONS_TABLE_SERVER;
            current_block_38 = 980989089337379490;
        }
        OPTIONS_TABLE_SESSION => {
            if args_has(args, 'g' as i32 as u_char) != 0 {
                *oo = global_s_options;
                scope = OPTIONS_TABLE_SESSION;
            } else if s.is_null() && !target.is_null() {
                xasprintf(
                    cause,
                    b"no such session: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else if s.is_null() {
                xasprintf(
                    cause,
                    b"no current session\0" as *const u8 as *const ::core::ffi::c_char,
                );
            } else {
                *oo = (*s).options;
                scope = OPTIONS_TABLE_SESSION;
            }
            current_block_38 = 980989089337379490;
        }
        12 => {
            if args_has(args, 'p' as i32 as u_char) != 0 {
                if wp.is_null() && !target.is_null() {
                    xasprintf(
                        cause,
                        b"no such pane: %s\0" as *const u8 as *const ::core::ffi::c_char,
                        target,
                    );
                } else if wp.is_null() {
                    xasprintf(
                        cause,
                        b"no current pane\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                } else {
                    *oo = (*wp).options;
                    scope = OPTIONS_TABLE_PANE;
                }
                current_block_38 = 980989089337379490;
            } else {
                current_block_38 = 7230205663690532434;
            }
        }
        OPTIONS_TABLE_WINDOW => {
            current_block_38 = 7230205663690532434;
        }
        _ => {
            current_block_38 = 980989089337379490;
        }
    }
    match current_block_38 {
        7230205663690532434 => {
            if args_has(args, 'g' as i32 as u_char) != 0 {
                *oo = global_w_options;
                scope = OPTIONS_TABLE_WINDOW;
            } else if wl.is_null() && !target.is_null() {
                xasprintf(
                    cause,
                    b"no such window: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else if wl.is_null() {
                xasprintf(
                    cause,
                    b"no current window\0" as *const u8 as *const ::core::ffi::c_char,
                );
            } else {
                *oo = (*(*wl).window).options;
                scope = OPTIONS_TABLE_WINDOW;
            }
        }
        _ => {}
    }
    return scope;
}
#[no_mangle]
pub unsafe extern "C" fn options_scope_from_flags(
    mut args: *mut args,
    mut window: ::core::ffi::c_int,
    mut fs: *mut cmd_find_state,
    mut oo: *mut *mut options,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut s: *mut session = (*fs).s;
    let mut wl: *mut winlink = (*fs).wl;
    let mut wp: *mut window_pane = (*fs).wp;
    let mut target: *const ::core::ffi::c_char = args_get(args, 't' as i32 as u_char);
    if args_has(args, 's' as i32 as u_char) != 0 {
        *oo = global_options;
        return 0x1 as ::core::ffi::c_int;
    }
    if args_has(args, 'p' as i32 as u_char) != 0 {
        if wp.is_null() {
            if !target.is_null() {
                xasprintf(
                    cause,
                    b"no such pane: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else {
                xasprintf(
                    cause,
                    b"no current pane\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return 0 as ::core::ffi::c_int;
        }
        *oo = (*wp).options;
        return 0x8 as ::core::ffi::c_int;
    } else if window != 0 || args_has(args, 'w' as i32 as u_char) != 0 {
        if args_has(args, 'g' as i32 as u_char) != 0 {
            *oo = global_w_options;
            return 0x4 as ::core::ffi::c_int;
        }
        if wl.is_null() {
            if !target.is_null() {
                xasprintf(
                    cause,
                    b"no such window: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else {
                xasprintf(
                    cause,
                    b"no current window\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return 0 as ::core::ffi::c_int;
        }
        *oo = (*(*wl).window).options;
        return 0x4 as ::core::ffi::c_int;
    } else {
        if args_has(args, 'g' as i32 as u_char) != 0 {
            *oo = global_s_options;
            return 0x2 as ::core::ffi::c_int;
        }
        if s.is_null() {
            if !target.is_null() {
                xasprintf(
                    cause,
                    b"no such session: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    target,
                );
            } else {
                xasprintf(
                    cause,
                    b"no current session\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return 0 as ::core::ffi::c_int;
        }
        *oo = (*s).options;
        return 0x2 as ::core::ffi::c_int;
    };
}
#[no_mangle]
pub unsafe extern "C" fn options_string_to_style(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut ft: *mut format_tree,
) -> *mut style {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut dgc: *const grid_cell = &raw const grid_default_cell;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut failed: ::core::ffi::c_int = 0;
    o = options_get(oo, name);
    if o.is_null()
        || !((*o).tableentry.is_null()
            || (*(*o).tableentry).type_0 as ::core::ffi::c_uint
                == OPTIONS_TABLE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        return ::core::ptr::null_mut::<style>();
    }
    if (*o).cached != 0 {
        return &raw mut (*o).style;
    }
    s = (*o).value.string;
    oe = (*o).tableentry;
    log_debug(
        b"%s: %s is '%s'\0" as *const u8 as *const ::core::ffi::c_char,
        b"options_string_to_style\0" as *const u8 as *const ::core::ffi::c_char,
        name,
        s,
    );
    style_set(&raw mut (*o).style, dgc);
    (*o).cached = (strstr(s, b"#{\0" as *const u8 as *const ::core::ffi::c_char)
        == NULL as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
    if !ft.is_null() && (*o).cached == 0 {
        expanded = format_expand(ft, s);
        if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_COLOUR != 0 {
            failed = style_parse_colour(&raw mut (*o).style, dgc, expanded);
        } else {
            failed = style_parse(&raw mut (*o).style, dgc, expanded);
        }
        free(expanded as *mut ::core::ffi::c_void);
        if failed != 0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<style>();
        }
    } else {
        if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_COLOUR != 0 {
            failed = style_parse_colour(&raw mut (*o).style, dgc, s);
        } else {
            failed = style_parse(&raw mut (*o).style, dgc, s);
        }
        if failed != 0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<style>();
        }
    }
    return &raw mut (*o).style;
}
unsafe extern "C" fn options_from_string_check(
    mut oe: *const options_table_entry,
    mut value: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut sy: style = style {
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
    if oe.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if strcmp(
        (*oe).name,
        b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        && checkshell(value) == 0
    {
        xasprintf(
            cause,
            b"not a suitable shell: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    if !(*oe).pattern.is_null()
        && fnmatch((*oe).pattern, value, 0 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int
    {
        xasprintf(
            cause,
            b"value is invalid: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    if (*oe).flags & OPTIONS_TABLE_IS_STYLE != 0
        && strstr(value, b"#{\0" as *const u8 as *const ::core::ffi::c_char).is_null()
        && style_parse(&raw mut sy, &raw const grid_default_cell, value) != 0 as ::core::ffi::c_int
    {
        xasprintf(
            cause,
            b"invalid style: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    if (*oe).flags & OPTIONS_TABLE_IS_COLOUR != 0
        && strstr(value, b"#{\0" as *const u8 as *const ::core::ffi::c_char).is_null()
        && style_parse_colour(&raw mut sy, &raw const grid_default_cell, value)
            != 0 as ::core::ffi::c_int
    {
        xasprintf(
            cause,
            b"invalid colour: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn options_from_string_flag(
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut flag: ::core::ffi::c_int = 0;
    if value.is_null() || *value as ::core::ffi::c_int == '\0' as i32 {
        flag = (options_get_number(oo, name) == 0) as ::core::ffi::c_int;
    } else if strcmp(value, b"1\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcasecmp(value, b"on\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        || strcasecmp(value, b"yes\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        flag = 1 as ::core::ffi::c_int;
    } else if strcmp(value, b"0\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcasecmp(value, b"off\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        || strcasecmp(value, b"no\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        flag = 0 as ::core::ffi::c_int;
    } else {
        xasprintf(
            cause,
            b"bad value: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    options_set_number(oo, name, flag as ::core::ffi::c_longlong);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn options_find_choice(
    mut oe: *const options_table_entry,
    mut value: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut cp: *mut *const ::core::ffi::c_char =
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut choice: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    cp = (*oe).choices;
    while !(*cp).is_null() {
        if strcmp(*cp, value) == 0 as ::core::ffi::c_int {
            choice = n;
        }
        n += 1;
        cp = cp.offset(1);
    }
    if choice == -(1 as ::core::ffi::c_int) {
        xasprintf(
            cause,
            b"unknown value: %s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        return -(1 as ::core::ffi::c_int);
    }
    return choice;
}
unsafe extern "C" fn options_from_string_choice(
    mut oe: *const options_table_entry,
    mut oo: *mut options,
    mut name: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut choice: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    if value.is_null() {
        choice = options_get_number(oo, name) as ::core::ffi::c_int;
        if choice < 2 as ::core::ffi::c_int {
            choice = (choice == 0) as ::core::ffi::c_int;
        }
    } else {
        choice = options_find_choice(oe, value, cause);
        if choice < 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
    }
    options_set_number(oo, name, choice as ::core::ffi::c_longlong);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn options_from_string(
    mut oo: *mut options,
    mut oe: *const options_table_entry,
    mut name: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut append: ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut type_0: options_table_type = OPTIONS_TABLE_STRING;
    let mut number: ::core::ffi::c_longlong = 0;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut new: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut old: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut key: key_code = 0;
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    if !oe.is_null() {
        if value.is_null()
            && (*oe).type_0 as ::core::ffi::c_uint
                != OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*oe).type_0 as ::core::ffi::c_uint
                != OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            xasprintf(
                cause,
                b"empty value\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        type_0 = (*oe).type_0;
    } else {
        if *name as ::core::ffi::c_int != '@' as i32 {
            xasprintf(
                cause,
                b"bad option name\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return -(1 as ::core::ffi::c_int);
        }
        type_0 = OPTIONS_TABLE_STRING;
    }
    match type_0 as ::core::ffi::c_uint {
        0 => {
            old = xstrdup(options_get_string(oo, name));
            options_set_string(
                oo,
                name,
                append,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                value,
            );
            new = options_get_string(oo, name);
            if options_from_string_check(oe, new, cause) != 0 as ::core::ffi::c_int {
                options_set_string(
                    oo,
                    name,
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    old,
                );
                free(old as *mut ::core::ffi::c_void);
                return -(1 as ::core::ffi::c_int);
            }
            free(old as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        }
        1 => {
            number = strtonum(
                value,
                (*oe).minimum as ::core::ffi::c_longlong,
                (*oe).maximum as ::core::ffi::c_longlong,
                &raw mut errstr,
            );
            if !errstr.is_null() {
                xasprintf(
                    cause,
                    b"value is %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    errstr,
                    value,
                );
                return -(1 as ::core::ffi::c_int);
            }
            options_set_number(oo, name, number);
            return 0 as ::core::ffi::c_int;
        }
        2 => {
            key = key_string_lookup_string(value);
            if key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
                xasprintf(
                    cause,
                    b"bad key: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    value,
                );
                return -(1 as ::core::ffi::c_int);
            }
            options_set_number(oo, name, key as ::core::ffi::c_longlong);
            return 0 as ::core::ffi::c_int;
        }
        3 => {
            number = colour_fromstring(value) as ::core::ffi::c_longlong;
            if number == -(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong {
                xasprintf(
                    cause,
                    b"bad colour: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    value,
                );
                return -(1 as ::core::ffi::c_int);
            }
            options_set_number(oo, name, number);
            return 0 as ::core::ffi::c_int;
        }
        4 => return options_from_string_flag(oo, name, value, cause),
        5 => return options_from_string_choice(oe, oo, name, value, cause),
        6 => {
            pr = cmd_parse_from_string(value, ::core::ptr::null_mut::<cmd_parse_input>());
            match (*pr).status as ::core::ffi::c_uint {
                0 => {
                    *cause = (*pr).error;
                    return -(1 as ::core::ffi::c_int);
                }
                1 => {
                    options_set_command(oo, name, (*pr).cmdlist);
                    return 0 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        _ => {}
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn options_push_changes(mut name: *const ::core::ffi::c_char) {
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"options_push_changes\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    if strcmp(name, b"theme\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strncmp(
            name,
            b"dark-theme-\0" as *const u8 as *const ::core::ffi::c_char,
            11 as size_t,
        ) == 0 as ::core::ffi::c_int
        || strncmp(
            name,
            b"light-theme-\0" as *const u8 as *const ::core::ffi::c_char,
            12 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            server_client_update_theme_colours(loop_0);
            if (*loop_0).tty.flags & TTY_OPENED != 0 {
                tty_invalidate(&raw mut (*loop_0).tty);
            }
            server_redraw_client(loop_0);
            loop_0 = (*loop_0).entry.tqe_next;
        }
    }
    if strcmp(
        name,
        b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
        while !w.is_null() {
            if !(*w).active.is_null() {
                if options_get_number((*w).options, name) != 0 {
                    (*(*w).active).flags |= PANE_CHANGED;
                }
            }
            w = windows_RB_NEXT(w);
        }
    }
    if strcmp(
        name,
        b"cursor-colour\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            window_pane_default_cursor(wp);
            wp = window_pane_tree_RB_NEXT(wp);
        }
    }
    if strcmp(
        name,
        b"cursor-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            window_pane_default_cursor(wp);
            wp = window_pane_tree_RB_NEXT(wp);
        }
    }
    if strcmp(
        name,
        b"fill-character\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
        while !w.is_null() {
            window_set_fill_cells(w);
            w = windows_RB_NEXT(w);
        }
    }
    if strcmp(
        name,
        b"key-table\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            server_client_set_key_table(loop_0, ::core::ptr::null::<::core::ffi::c_char>());
            loop_0 = (*loop_0).entry.tqe_next;
        }
    }
    if strcmp(
        name,
        b"user-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            if (*loop_0).tty.flags & TTY_OPENED != 0 {
                tty_keys_build(&raw mut (*loop_0).tty);
            }
            loop_0 = (*loop_0).entry.tqe_next;
        }
    }
    if strcmp(name, b"status\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"status-interval\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        status_timer_start_all();
    }
    if strcmp(name, b"status\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"status-position\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-border-indicators\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-border-status\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars-timeout\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars-position\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars-style\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        redraw_invalidate_all_scenes();
    }
    if strcmp(
        name,
        b"monitor-silence\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        alerts_reset_all();
    }
    if strcmp(
        name,
        b"window-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"window-active-style\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            (*wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
            wp = window_pane_tree_RB_NEXT(wp);
        }
    }
    if *name as ::core::ffi::c_int == '@' as i32 {
        wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            (*wp).flags |= PANE_STYLECHANGED;
            wp = window_pane_tree_RB_NEXT(wp);
        }
    }
    if strcmp(
        name,
        b"pane-colours\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            colour_palette_from_option(&raw mut (*wp).palette, (*wp).options);
            wp = window_pane_tree_RB_NEXT(wp);
        }
    }
    if strcmp(
        name,
        b"pane-border-status\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        || strcmp(
            name,
            b"pane-scrollbars-position\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
        while !w.is_null() {
            (*w).sb = options_get_number(
                (*w).options,
                b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
            (*w).sb_pos = options_get_number(
                (*w).options,
                b"pane-scrollbars-position\0" as *const u8 as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
            layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
            w = windows_RB_NEXT(w);
        }
    }
    if strcmp(
        name,
        b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            window_pane_scrollbar_hide(wp);
            wp = window_pane_tree_RB_NEXT(wp);
        }
    }
    if strcmp(
        name,
        b"pane-scrollbars-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
        while !wp.is_null() {
            style_set_scrollbar_style_from_option(&raw mut (*wp).scrollbar_style, (*wp).options);
            wp = window_pane_tree_RB_NEXT(wp);
        }
        w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
        while !w.is_null() {
            layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
            w = windows_RB_NEXT(w);
        }
    }
    if strcmp(
        name,
        b"codepoint-widths\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        utf8_update_width_cache();
    }
    if strcmp(
        name,
        b"input-buffer-size\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        input_set_buffer_size(options_get_number(global_options, name) as size_t);
    }
    if strcmp(
        name,
        b"history-limit\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
        while !s.is_null() {
            session_update_history(s);
            s = sessions_RB_NEXT(s);
        }
    }
    s = sessions_RB_MINMAX(&raw mut sessions, RB_NEGINF);
    while !s.is_null() {
        status_update_cache(s);
        s = sessions_RB_NEXT(s);
    }
    recalculate_sizes();
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        if !(*loop_0).session.is_null() {
            server_redraw_client(loop_0);
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn options_remove_or_default(
    mut o: *mut options_entry,
    mut key: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut oo: *mut options = (*o).owner;
    if key.is_null() {
        if !(*o).tableentry.is_null()
            && (oo == global_options || oo == global_s_options || oo == global_w_options)
        {
            options_default(oo, (*o).tableentry);
        } else {
            options_remove(o);
        }
    } else if options_array_set(
        o,
        key,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
        cause,
    ) != 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    return 0 as ::core::ffi::c_int;
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

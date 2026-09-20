use crate::src::shared::client::*;
use crate::src::shared::tty::*;
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
    pub type hyperlinks;
    pub type screen_write_cline;
    pub type screen_sel;
    pub type screen_titles;
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
    pub type format_job_tree;
    pub type control_state;
    pub type cmdq_list;
    pub type options_array_item;
    pub type options_entry;
    fn tigetflag(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn tigetnum(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn tigetstr(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn tiparm_s(
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    ) -> *mut ::core::ffi::c_char;
    fn fnmatch(
        __pattern: *const ::core::ffi::c_char,
        __name: *const ::core::ffi::c_char,
        __flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
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
    fn strcspn(
        __s: *const ::core::ffi::c_char,
        __reject: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_ulong;
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    static mut cur_term: *mut TERMINAL;
    fn del_curterm(_: *mut TERMINAL) -> ::core::ffi::c_int;
    fn setupterm(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn strnvis(
        _: *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: size_t,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn strunvis(_: *mut ::core::ffi::c_char, _: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn strtonum(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
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
    static mut global_options: *mut options;
    fn options_get_only(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_array_first(_: *mut options_entry) -> *mut options_array_item;
    fn options_array_next(_: *mut options_array_item) -> *mut options_array_item;
    fn options_array_item_value(_: *mut options_array_item) -> *mut options_value;
    fn environ_find(_: *mut environ, _: *const ::core::ffi::c_char) -> *mut environ_entry;
    fn tty_parse_client_features(
        _: *mut client,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
    );
    fn tty_apply_features(_: *mut tty_term) -> ::core::ffi::c_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub offset: u_int,
    pub data: C2RustUnnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct termtype {
    pub term_names: *mut ::core::ffi::c_char,
    pub str_table: *mut ::core::ffi::c_char,
    pub Booleans: *mut ::core::ffi::c_char,
    pub Numbers: *mut ::core::ffi::c_short,
    pub Strings: *mut *mut ::core::ffi::c_char,
    pub ext_str_table: *mut ::core::ffi::c_char,
    pub ext_Names: *mut *mut ::core::ffi::c_char,
    pub num_Booleans: ::core::ffi::c_ushort,
    pub num_Numbers: ::core::ffi::c_ushort,
    pub num_Strings: ::core::ffi::c_ushort,
    pub ext_Booleans: ::core::ffi::c_ushort,
    pub ext_Numbers: ::core::ffi::c_ushort,
    pub ext_Strings: ::core::ffi::c_ushort,
}
pub type TERMTYPE = termtype;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct term {
    pub type_0: TERMTYPE,
}
pub type TERMINAL = term;
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
    pub entry: C2RustUnnamed_12,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_12 {
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
    pub entry: C2RustUnnamed_13,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_13 {
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
pub struct tty_code {
    pub type_0: tty_code_type,
    pub value: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_35 {
    pub string: *mut ::core::ffi::c_char,
    pub number: ::core::ffi::c_int,
    pub flag: ::core::ffi::c_int,
}
pub type tty_code_type = ::core::ffi::c_uint;
pub const TTYCODE_FLAG: tty_code_type = 3;
pub const TTYCODE_NUMBER: tty_code_type = 2;
pub const TTYCODE_STRING: tty_code_type = 1;
pub const TTYCODE_NONE: tty_code_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct environ_entry {
    pub name: *mut ::core::ffi::c_char,
    pub value: *mut ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_36,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_36 {
    pub rbe_left: *mut environ_entry,
    pub rbe_right: *mut environ_entry,
    pub rbe_parent: *mut environ_entry,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_terms {
    pub lh_first: *mut tty_term,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_term_code_entry {
    pub type_0: tty_code_type,
    pub name: *const ::core::ffi::c_char,
}
pub const OK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const VIS_OCTAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VIS_CSTYLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VIS_TAB: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const VIS_NL: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const TERM_NOAM: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const TERM_DECSLRM: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const TERM_DECFRA: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const TERM_RGBCOLOURS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const TERM_VT100LIKE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const TERM_SIXEL: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const TERM_INVALIDMS: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
#[no_mangle]
pub static mut tty_terms: tty_terms = tty_terms {
    lh_first: ::core::ptr::null::<tty_term>() as *mut tty_term,
};
static mut tty_term_codes: [tty_term_code_entry; 234] = [
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"acsc\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: b"am\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: b"AX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: b"bce\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"bel\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Bidi\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"blink\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"bold\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"civis\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"clear\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Clmg\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Cmg\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cnorm\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_NUMBER,
        name: b"colors\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Cr\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Cs\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"csr\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cub\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cub1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cud\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cud1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cuf\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cuf1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cup\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cuu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cuu1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"cvvis\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"dch\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"dch1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"dim\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"dl\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"dl1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Dsbp\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Dseks\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Dsfcs\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Dsmg\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"E3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"ech\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"ed\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"el\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"el1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"enacs\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Enbp\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Eneks\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Enfcs\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Enmg\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"fsl\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Hls\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"home\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"hpa\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"ich\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"ich1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"il\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"il1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"ind\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"indn\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"invis\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kcbt\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kcub1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kcud1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kcuf1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kcuu1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDC\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDC3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDC4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDC5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDC6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDC7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kdch1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDN\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDN3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDN4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDN5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDN6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kDN7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kend\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kEND\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kEND3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kEND4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kEND5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kEND6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kEND7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf10\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf11\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf12\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf13\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf14\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf15\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf16\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf17\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf18\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf19\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf2\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf20\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf21\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf22\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf23\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf24\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf25\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf26\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf27\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf28\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf29\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf30\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf31\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf32\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf33\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf34\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf35\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf36\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf37\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf38\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf39\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf40\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf41\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf42\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf43\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf44\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf45\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf46\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf47\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf48\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf49\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf50\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf51\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf52\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf53\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf54\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf55\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf56\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf57\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf58\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf59\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf60\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf61\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf62\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf63\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf8\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kf9\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kHOM\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kHOM3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kHOM4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kHOM5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kHOM6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kHOM7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"khome\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kIC\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kIC3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kIC4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kIC5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kIC6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kIC7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kich1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kind\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kLFT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kLFT3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kLFT4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kLFT5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kLFT6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kLFT7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kmous\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"knp\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kNXT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kNXT3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kNXT4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kNXT5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kNXT6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kNXT7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kpp\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kPRV\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kPRV3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kPRV4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kPRV5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kPRV6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kPRV7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kri\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kRIT\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kRIT3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kRIT4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kRIT5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kRIT6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kRIT7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kUP\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kUP3\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kUP4\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kUP5\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kUP6\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"kUP7\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Ms\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Nobr\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"ol\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"op\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Rect\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"rev\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: b"RGB\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"ri\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"rin\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"rmacs\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"rmcup\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"rmkx\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Se\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"setab\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"setaf\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"setal\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"setrgbb\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"setrgbf\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Setulc\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Setulc1\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"sgr0\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"sitm\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"smacs\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"smcup\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"smkx\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Smol\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"smso\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"smul\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Smulx\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"smxx\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Spb\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: b"Sxl\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Ss\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Swd\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"Sync\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: b"Tc\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"tsl\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_NUMBER,
        name: b"U8\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_STRING,
        name: b"vpa\0" as *const u8 as *const ::core::ffi::c_char,
    },
    tty_term_code_entry {
        type_0: TTYCODE_FLAG,
        name: b"XT\0" as *const u8 as *const ::core::ffi::c_char,
    },
];
#[no_mangle]
pub unsafe extern "C" fn tty_term_ncodes() -> u_int {
    return (::core::mem::size_of::<[tty_term_code_entry; 234]>() as usize)
        .wrapping_div(::core::mem::size_of::<tty_term_code_entry>() as usize) as u_int;
}
unsafe extern "C" fn tty_term_strip(mut s: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut ptr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    static mut buf: [::core::ffi::c_char; 8192] = [0; 8192];
    let mut len: size_t = 0;
    if strchr(s, '$' as i32).is_null() {
        return xstrdup(s);
    }
    len = 0 as size_t;
    ptr = s;
    while *ptr as ::core::ffi::c_int != '\0' as i32 {
        if *ptr as ::core::ffi::c_int == '$' as i32
            && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '<' as i32
        {
            while *ptr as ::core::ffi::c_int != '\0' as i32
                && *ptr as ::core::ffi::c_int != '>' as i32
            {
                ptr = ptr.offset(1);
            }
            if *ptr as ::core::ffi::c_int == '>' as i32 {
                ptr = ptr.offset(1);
            }
            if *ptr as ::core::ffi::c_int == '\0' as i32 {
                break;
            }
        }
        let fresh3 = len;
        len = len.wrapping_add(1);
        buf[fresh3 as usize] = *ptr;
        if len
            == (::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as usize)
                .wrapping_sub(1 as usize)
        {
            break;
        }
        ptr = ptr.offset(1);
    }
    buf[len as usize] = '\0' as i32 as ::core::ffi::c_char;
    return xstrdup(&raw mut buf as *mut ::core::ffi::c_char);
}
unsafe extern "C" fn tty_term_override_next(
    mut s: *const ::core::ffi::c_char,
    mut offset: *mut size_t,
) -> *mut ::core::ffi::c_char {
    static mut value: [::core::ffi::c_char; 8192] = [0; 8192];
    let mut n: size_t = 0 as size_t;
    let mut at: size_t = *offset;
    if *s.offset(at as isize) as ::core::ffi::c_int == '\0' as i32 {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    while *s.offset(at as isize) as ::core::ffi::c_int != '\0' as i32 {
        if *s.offset(at as isize) as ::core::ffi::c_int == ':' as i32 {
            if !(*s.offset(at.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                == ':' as i32)
            {
                break;
            }
            let fresh1 = n;
            n = n.wrapping_add(1);
            value[fresh1 as usize] = ':' as i32 as ::core::ffi::c_char;
            at = at.wrapping_add(2 as size_t);
        } else {
            let fresh2 = n;
            n = n.wrapping_add(1);
            value[fresh2 as usize] = *s.offset(at as isize);
            at = at.wrapping_add(1);
        }
        if n == (::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as usize)
            .wrapping_sub(1 as usize)
        {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    }
    if *s.offset(at as isize) as ::core::ffi::c_int != '\0' as i32 {
        *offset = at.wrapping_add(1 as size_t);
    } else {
        *offset = at;
    }
    value[n as usize] = '\0' as i32 as ::core::ffi::c_char;
    return &raw mut value as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_apply(
    mut term: *mut tty_term,
    mut capabilities: *const ::core::ffi::c_char,
    mut quiet: ::core::ffi::c_int,
) {
    let mut ent: *const tty_term_code_entry = ::core::ptr::null::<tty_term_code_entry>();
    let mut code: *mut tty_code = ::core::ptr::null_mut::<tty_code>();
    let mut offset: size_t = 0 as size_t;
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = (*term).name;
    let mut i: u_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut remove: ::core::ffi::c_int = 0;
    loop {
        s = tty_term_override_next(capabilities, &raw mut offset);
        if s.is_null() {
            break;
        }
        if *s as ::core::ffi::c_int == '\0' as i32 {
            continue;
        }
        value = ::core::ptr::null_mut::<::core::ffi::c_char>();
        remove = 0 as ::core::ffi::c_int;
        cp = strchr(s, '=' as i32);
        if !cp.is_null() {
            let fresh0 = cp;
            cp = cp.offset(1);
            *fresh0 = '\0' as i32 as ::core::ffi::c_char;
            value = xstrdup(cp);
            if strunvis(value, cp) == -(1 as ::core::ffi::c_int) {
                free(value as *mut ::core::ffi::c_void);
                value = xstrdup(cp);
            }
        } else if *s.offset(strlen(s).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            == '@' as i32
        {
            *s.offset(strlen(s).wrapping_sub(1 as size_t) as isize) =
                '\0' as i32 as ::core::ffi::c_char;
            remove = 1 as ::core::ffi::c_int;
        } else {
            value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        }
        if quiet == 0 {
            if remove != 0 {
                log_debug(
                    b"%s override: %s@\0" as *const u8 as *const ::core::ffi::c_char,
                    name,
                    s,
                );
            } else if *value as ::core::ffi::c_int == '\0' as i32 {
                log_debug(
                    b"%s override: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    name,
                    s,
                );
            } else {
                log_debug(
                    b"%s override: %s=%s\0" as *const u8 as *const ::core::ffi::c_char,
                    name,
                    s,
                    value,
                );
            }
        }
        i = 0 as u_int;
        while i < tty_term_ncodes() {
            ent = (&raw const tty_term_codes as *const tty_term_code_entry).offset(i as isize)
                as *const tty_term_code_entry;
            if !(strcmp(s, (*ent).name) != 0 as ::core::ffi::c_int) {
                code = (*term).codes.offset(i as isize) as *mut tty_code;
                if remove != 0 {
                    (*code).type_0 = TTYCODE_NONE;
                } else {
                    match (*ent).type_0 as ::core::ffi::c_uint {
                        1 => {
                            if (*code).type_0 as ::core::ffi::c_uint
                                == TTYCODE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                free((*code).value.string as *mut ::core::ffi::c_void);
                            }
                            (*code).value.string = xstrdup(value);
                            (*code).type_0 = (*ent).type_0;
                        }
                        2 => {
                            n = strtonum(
                                value,
                                0 as ::core::ffi::c_longlong,
                                INT_MAX as ::core::ffi::c_longlong,
                                &raw mut errstr,
                            ) as ::core::ffi::c_int;
                            if errstr.is_null() {
                                (*code).value.number = n;
                                (*code).type_0 = (*ent).type_0;
                            }
                        }
                        3 => {
                            (*code).value.flag = 1 as ::core::ffi::c_int;
                            (*code).type_0 = (*ent).type_0;
                        }
                        0 | _ => {}
                    }
                }
            }
            i = i.wrapping_add(1);
        }
        free(value as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_apply_overrides(mut term: *mut tty_term) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut acs: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut offset: size_t = 0;
    let mut first: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    o = options_get_only(
        global_options,
        b"terminal-overrides\0" as *const u8 as *const ::core::ffi::c_char,
    );
    a = options_array_first(o);
    while !a.is_null() {
        ov = options_array_item_value(a);
        s = (*ov).string;
        offset = 0 as size_t;
        first = tty_term_override_next(s, &raw mut offset);
        if !first.is_null()
            && fnmatch(first, (*term).name, 0 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int
        {
            tty_term_apply(term, s.offset(offset as isize), 0 as ::core::ffi::c_int);
        }
        a = options_array_next(a);
    }
    log_debug(
        b"SIXEL flag is %d\0" as *const u8 as *const ::core::ffi::c_char,
        ((*term).flags & TERM_SIXEL != 0) as ::core::ffi::c_int,
    );
    if tty_term_has(term, TTYC_SETRGBF) != 0 && tty_term_has(term, TTYC_SETRGBB) != 0 {
        (*term).flags |= TERM_RGBCOLOURS;
    } else {
        (*term).flags &= !TERM_RGBCOLOURS;
    }
    log_debug(
        b"RGBCOLOURS flag is %d\0" as *const u8 as *const ::core::ffi::c_char,
        ((*term).flags & TERM_RGBCOLOURS != 0) as ::core::ffi::c_int,
    );
    if tty_term_has(term, TTYC_CMG) != 0 && tty_term_has(term, TTYC_CLMG) != 0 {
        (*term).flags |= TERM_DECSLRM;
    } else {
        (*term).flags &= !TERM_DECSLRM;
    }
    log_debug(
        b"DECSLRM flag is %d\0" as *const u8 as *const ::core::ffi::c_char,
        ((*term).flags & TERM_DECSLRM != 0) as ::core::ffi::c_int,
    );
    if tty_term_has(term, TTYC_RECT) != 0 {
        (*term).flags |= TERM_DECFRA;
    } else {
        (*term).flags &= !TERM_DECFRA;
    }
    log_debug(
        b"DECFRA flag is %d\0" as *const u8 as *const ::core::ffi::c_char,
        ((*term).flags & TERM_DECFRA != 0) as ::core::ffi::c_int,
    );
    if tty_term_flag(term, TTYC_AM) == 0 {
        (*term).flags |= TERM_NOAM;
    } else {
        (*term).flags &= !TERM_NOAM;
    }
    log_debug(
        b"NOAM flag is %d\0" as *const u8 as *const ::core::ffi::c_char,
        ((*term).flags & TERM_NOAM != 0) as ::core::ffi::c_int,
    );
    memset(
        &raw mut (*term).acs as *mut [::core::ffi::c_char; 2] as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[[::core::ffi::c_char; 2]; 256]>() as size_t,
    );
    if tty_term_has(term, TTYC_ACSC) != 0 {
        acs = tty_term_string(term, TTYC_ACSC);
    } else {
        acs =
            b"a#j+k+l+m+n+o-p-q-r-s-t+u+v+w+x|y<z>~.\0" as *const u8 as *const ::core::ffi::c_char;
    }
    while *acs.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
        && *acs.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
    {
        (*term).acs[*acs.offset(0 as ::core::ffi::c_int as isize) as u_char as usize]
            [0 as ::core::ffi::c_int as usize] = *acs.offset(1 as ::core::ffi::c_int as isize);
        acs = acs.offset(2 as ::core::ffi::c_int as isize);
    }
    tty_term_validate(term);
}
unsafe extern "C" fn tty_term_validate(mut term: *mut tty_term) {
    let mut code: *mut tty_code =
        (*term).codes.offset(TTYC_MS as ::core::ffi::c_int as isize) as *mut tty_code;
    if (*code).type_0 as ::core::ffi::c_uint
        != TTYCODE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    if *tty_term_string_ss(
        term,
        TTYC_MS,
        b"c\0" as *const u8 as *const ::core::ffi::c_char,
        b"?\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int
        != '\0' as i32
    {
        (*term).flags &= !TERM_INVALIDMS;
        return;
    }
    log_debug(b"removing invalid Ms capability\0" as *const u8 as *const ::core::ffi::c_char);
    (*term).flags |= TERM_INVALIDMS;
    free((*code).value.string as *mut ::core::ffi::c_void);
    (*code).type_0 = TTYCODE_NONE;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_create(
    mut tty: *mut tty,
    mut name: *mut ::core::ffi::c_char,
    mut caps: *mut *mut ::core::ffi::c_char,
    mut ncaps: u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut tty_term {
    let mut c: *mut client = (*tty).client;
    let mut term: *mut tty_term = ::core::ptr::null_mut::<tty_term>();
    let mut ent: *const tty_term_code_entry = ::core::ptr::null::<tty_term_code_entry>();
    let mut code: *mut tty_code = ::core::ptr::null_mut::<tty_code>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut offset: size_t = 0;
    let mut namelen: size_t = 0;
    let mut first: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut n: ::core::ffi::c_int = 0;
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    log_debug(
        b"adding term %s\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    term = xcalloc(1 as size_t, ::core::mem::size_of::<tty_term>() as size_t) as *mut tty_term;
    (*term).tty = tty as *mut tty;
    (*term).name = xstrdup(name);
    (*term).codes = xcalloc(
        tty_term_ncodes() as size_t,
        ::core::mem::size_of::<tty_code>() as size_t,
    ) as *mut tty_code;
    (*term).entry.le_next = tty_terms.lh_first;
    if !(*term).entry.le_next.is_null() {
        (*tty_terms.lh_first).entry.le_prev = &raw mut (*term).entry.le_next;
    }
    tty_terms.lh_first = term;
    (*term).entry.le_prev = &raw mut tty_terms.lh_first;
    i = 0 as u_int;
    while i < ncaps {
        namelen = strcspn(
            *caps.offset(i as isize),
            b"=\0" as *const u8 as *const ::core::ffi::c_char,
        ) as size_t;
        if !(namelen == 0 as size_t) {
            value = (*caps.offset(i as isize))
                .offset(namelen as isize)
                .offset(1 as ::core::ffi::c_int as isize);
            j = 0 as u_int;
            while j < tty_term_ncodes() {
                ent = (&raw const tty_term_codes as *const tty_term_code_entry).offset(j as isize)
                    as *const tty_term_code_entry;
                if !(strncmp((*ent).name, *caps.offset(i as isize), namelen)
                    != 0 as ::core::ffi::c_int)
                {
                    if !(*(*ent).name.offset(namelen as isize) as ::core::ffi::c_int != '\0' as i32)
                    {
                        code = (*term).codes.offset(j as isize) as *mut tty_code;
                        (*code).type_0 = TTYCODE_NONE;
                        match (*ent).type_0 as ::core::ffi::c_uint {
                            1 => {
                                (*code).type_0 = TTYCODE_STRING;
                                (*code).value.string = tty_term_strip(value);
                            }
                            2 => {
                                n = strtonum(
                                    value,
                                    0 as ::core::ffi::c_longlong,
                                    INT_MAX as ::core::ffi::c_longlong,
                                    &raw mut errstr,
                                ) as ::core::ffi::c_int;
                                if !errstr.is_null() {
                                    log_debug(
                                        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                                        (*ent).name,
                                        errstr,
                                    );
                                } else {
                                    (*code).type_0 = TTYCODE_NUMBER;
                                    (*code).value.number = n;
                                }
                            }
                            3 => {
                                (*code).type_0 = TTYCODE_FLAG;
                                (*code).value.flag = (*value as ::core::ffi::c_int == '1' as i32)
                                    as ::core::ffi::c_int;
                            }
                            0 | _ => {}
                        }
                    }
                }
                j = j.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
    o = options_get_only(
        global_options,
        b"terminal-features\0" as *const u8 as *const ::core::ffi::c_char,
    );
    a = options_array_first(o);
    while !a.is_null() {
        ov = options_array_item_value(a);
        s = (*ov).string;
        offset = 0 as size_t;
        first = tty_term_override_next(s, &raw mut offset);
        if !first.is_null()
            && fnmatch(first, (*term).name, 0 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int
        {
            tty_parse_client_features(
                c,
                s.offset(offset as isize),
                b":\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        a = options_array_next(a);
    }
    del_curterm(cur_term);
    envent = environ_find(
        (*c).environ,
        b"COLORTERM\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if !envent.is_null() {
        log_debug(
            b"%s COLORTERM=%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            (*envent).value,
        );
        if strcasecmp(
            (*envent).value,
            b"truecolor\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
            || strcasecmp(
                (*envent).value,
                b"24bit\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            tty_parse_client_features(
                c,
                b"RGB\0" as *const u8 as *const ::core::ffi::c_char,
                b",\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else if !strstr(
            (*envent).value,
            b"256\0" as *const u8 as *const ::core::ffi::c_char,
        )
        .is_null()
        {
            tty_parse_client_features(
                c,
                b"256\0" as *const u8 as *const ::core::ffi::c_char,
                b",\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    }
    tty_term_apply_overrides(term);
    if tty_term_has(term, TTYC_CLEAR) == 0 {
        xasprintf(
            cause,
            b"terminal does not support clear\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else if tty_term_has(term, TTYC_CUP) == 0 {
        xasprintf(
            cause,
            b"terminal does not support cup\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        s = tty_term_string(term, TTYC_CLEAR);
        if tty_term_flag(term, TTYC_XT) != 0
            || strncmp(
                s,
                b"\x1B[\0" as *const u8 as *const ::core::ffi::c_char,
                2 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            (*term).flags |= TERM_VT100LIKE;
            tty_parse_client_features(
                c,
                b"bpaste,focus,title\0" as *const u8 as *const ::core::ffi::c_char,
                b",\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if (tty_term_flag(term, TTYC_TC) != 0 || tty_term_has(term, TTYC_RGB) != 0)
            && (tty_term_has(term, TTYC_SETRGBF) == 0 || tty_term_has(term, TTYC_SETRGBB) == 0)
        {
            tty_parse_client_features(
                c,
                b"RGB\0" as *const u8 as *const ::core::ffi::c_char,
                b",\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if tty_apply_features(term) != 0 {
            tty_term_apply_overrides(term);
        }
        i = 0 as u_int;
        while i < tty_term_ncodes() {
            log_debug(
                b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                name,
                tty_term_describe(term, i as tty_code_code),
            );
            i = i.wrapping_add(1);
        }
        return term;
    }
    tty_term_free(term);
    return ::core::ptr::null_mut::<tty_term>();
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_free(mut term: *mut tty_term) {
    let mut i: u_int = 0;
    log_debug(
        b"removing term %s\0" as *const u8 as *const ::core::ffi::c_char,
        (*term).name,
    );
    i = 0 as u_int;
    while i < tty_term_ncodes() {
        if (*(*term).codes.offset(i as isize)).type_0 as ::core::ffi::c_uint
            == TTYCODE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            free((*(*term).codes.offset(i as isize)).value.string as *mut ::core::ffi::c_void);
        }
        i = i.wrapping_add(1);
    }
    free((*term).codes as *mut ::core::ffi::c_void);
    if !(*term).entry.le_next.is_null() {
        (*(*term).entry.le_next).entry.le_prev = (*term).entry.le_prev;
    }
    *(*term).entry.le_prev = (*term).entry.le_next;
    free((*term).name as *mut ::core::ffi::c_void);
    free(term as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_read_list(
    mut name: *const ::core::ffi::c_char,
    mut fd: ::core::ffi::c_int,
    mut caps: *mut *mut *mut ::core::ffi::c_char,
    mut ncaps: *mut u_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut ent: *const tty_term_code_entry = ::core::ptr::null::<tty_term_code_entry>();
    let mut error: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut i: u_int = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tmp: [::core::ffi::c_char; 11] = [0; 11];
    if setupterm(name as *mut ::core::ffi::c_char, fd, &raw mut error) != OK {
        match error {
            1 => {
                xasprintf(
                    cause,
                    b"can't use hardcopy terminal: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    name,
                );
            }
            0 => {
                xasprintf(
                    cause,
                    b"missing or unsuitable terminal: %s\0" as *const u8
                        as *const ::core::ffi::c_char,
                    name,
                );
            }
            -1 => {
                xasprintf(
                    cause,
                    b"can't find terminfo database\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            _ => {
                xasprintf(
                    cause,
                    b"unknown error\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        return -(1 as ::core::ffi::c_int);
    }
    *ncaps = 0 as u_int;
    *caps = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut current_block_23: u64;
    i = 0 as u_int;
    while i < tty_term_ncodes() {
        ent = (&raw const tty_term_codes as *const tty_term_code_entry).offset(i as isize)
            as *const tty_term_code_entry;
        match (*ent).type_0 as ::core::ffi::c_uint {
            0 => {
                current_block_23 = 1856101646708284338;
            }
            1 => {
                s = tigetstr((*ent).name as *mut ::core::ffi::c_char);
                if s.is_null()
                    || s == -(1 as ::core::ffi::c_int) as *mut ::core::ffi::c_char
                        as *const ::core::ffi::c_char
                {
                    current_block_23 = 1856101646708284338;
                } else {
                    current_block_23 = 14763689060501151050;
                }
            }
            2 => {
                n = tigetnum((*ent).name as *mut ::core::ffi::c_char);
                if n == -(1 as ::core::ffi::c_int) || n == -(2 as ::core::ffi::c_int) {
                    current_block_23 = 1856101646708284338;
                } else {
                    xsnprintf(
                        &raw mut tmp as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 11]>() as size_t,
                        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
                        n,
                    );
                    s = &raw mut tmp as *mut ::core::ffi::c_char;
                    current_block_23 = 14763689060501151050;
                }
            }
            3 => {
                n = tigetflag((*ent).name as *mut ::core::ffi::c_char);
                if n == -(1 as ::core::ffi::c_int) {
                    current_block_23 = 1856101646708284338;
                } else {
                    if n != 0 {
                        s = b"1\0" as *const u8 as *const ::core::ffi::c_char;
                    } else {
                        s = b"0\0" as *const u8 as *const ::core::ffi::c_char;
                    }
                    current_block_23 = 14763689060501151050;
                }
            }
            _ => {
                fatalx(b"unknown capability type\0" as *const u8 as *const ::core::ffi::c_char);
            }
        }
        match current_block_23 {
            14763689060501151050 => {
                *caps = xreallocarray(
                    *caps as *mut ::core::ffi::c_void,
                    (*ncaps).wrapping_add(1 as u_int) as size_t,
                    ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
                ) as *mut *mut ::core::ffi::c_char;
                xasprintf(
                    (*caps).offset(*ncaps as isize) as *mut *mut ::core::ffi::c_char,
                    b"%s=%s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*ent).name,
                    s,
                );
                *ncaps = (*ncaps).wrapping_add(1);
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
    del_curterm(cur_term);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_free_list(
    mut caps: *mut *mut ::core::ffi::c_char,
    mut ncaps: u_int,
) {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < ncaps {
        free(*caps.offset(i as isize) as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    free(caps as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_has(
    mut term: *mut tty_term,
    mut code: tty_code_code,
) -> ::core::ffi::c_int {
    return ((*(*term).codes.offset(code as isize)).type_0 as ::core::ffi::c_uint
        != TTYCODE_NONE as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_has_name(
    mut term: *mut tty_term,
    mut name: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < tty_term_ncodes() {
        if strcmp(tty_term_codes[i as usize].name, name) == 0 as ::core::ffi::c_int {
            return tty_term_has(term, i as tty_code_code);
        }
        i = i.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string(
    mut term: *mut tty_term,
    mut code: tty_code_code,
) -> *const ::core::ffi::c_char {
    if tty_term_has(term, code) == 0 {
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if (*(*term).codes.offset(code as isize)).type_0 as ::core::ffi::c_uint
        != TTYCODE_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fatalx(
            b"not a string: %d\0" as *const u8 as *const ::core::ffi::c_char,
            code as ::core::ffi::c_uint,
        );
    }
    return (*(*term).codes.offset(code as isize)).value.string;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string_i(
    mut term: *mut tty_term,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, x, a);
    if s.is_null() {
        log_debug(
            b"could not expand %s\0" as *const u8 as *const ::core::ffi::c_char,
            tty_term_codes[code as usize].name,
        );
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string_ii(
    mut term: *mut tty_term,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(2 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, x, a, b);
    if s.is_null() {
        log_debug(
            b"could not expand %s\0" as *const u8 as *const ::core::ffi::c_char,
            tty_term_codes[code as usize].name,
        );
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string_iii(
    mut term: *mut tty_term,
    mut code: tty_code_code,
    mut a: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(3 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, x, a, b, c);
    if s.is_null() {
        log_debug(
            b"could not expand %s\0" as *const u8 as *const ::core::ffi::c_char,
            tty_term_codes[code as usize].name,
        );
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string_s(
    mut term: *mut tty_term,
    mut code: tty_code_code,
    mut a: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int, x, a);
    if s.is_null() {
        log_debug(
            b"could not expand %s\0" as *const u8 as *const ::core::ffi::c_char,
            tty_term_codes[code as usize].name,
        );
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_string_ss(
    mut term: *mut tty_term,
    mut code: tty_code_code,
    mut a: *const ::core::ffi::c_char,
    mut b: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut x: *const ::core::ffi::c_char = tty_term_string(term, code);
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    s = tiparm_s(2 as ::core::ffi::c_int, 3 as ::core::ffi::c_int, x, a, b);
    if s.is_null() {
        log_debug(
            b"could not expand %s\0" as *const u8 as *const ::core::ffi::c_char,
            tty_term_codes[code as usize].name,
        );
        return b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return s;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_number(
    mut term: *mut tty_term,
    mut code: tty_code_code,
) -> ::core::ffi::c_int {
    if tty_term_has(term, code) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*(*term).codes.offset(code as isize)).type_0 as ::core::ffi::c_uint
        != TTYCODE_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fatalx(
            b"not a number: %d\0" as *const u8 as *const ::core::ffi::c_char,
            code as ::core::ffi::c_uint,
        );
    }
    return (*(*term).codes.offset(code as isize)).value.number;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_flag(
    mut term: *mut tty_term,
    mut code: tty_code_code,
) -> ::core::ffi::c_int {
    if tty_term_has(term, code) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*(*term).codes.offset(code as isize)).type_0 as ::core::ffi::c_uint
        != TTYCODE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fatalx(
            b"not a flag: %d\0" as *const u8 as *const ::core::ffi::c_char,
            code as ::core::ffi::c_uint,
        );
    }
    return (*(*term).codes.offset(code as isize)).value.flag;
}
#[no_mangle]
pub unsafe extern "C" fn tty_term_describe(
    mut term: *mut tty_term,
    mut code: tty_code_code,
) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 256] = [0; 256];
    let mut out: [::core::ffi::c_char; 128] = [0; 128];
    match (*(*term).codes.offset(code as isize)).type_0 as ::core::ffi::c_uint {
        0 => {
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
                b"%4u: %s: [missing]\0" as *const u8 as *const ::core::ffi::c_char,
                code as ::core::ffi::c_uint,
                tty_term_codes[code as usize].name,
            );
        }
        1 => {
            strnvis(
                &raw mut out as *mut ::core::ffi::c_char,
                (*(*term).codes.offset(code as isize)).value.string,
                ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
                VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL,
            );
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
                b"%4u: %s: (string) %s\0" as *const u8 as *const ::core::ffi::c_char,
                code as ::core::ffi::c_uint,
                tty_term_codes[code as usize].name,
                &raw mut out as *mut ::core::ffi::c_char,
            );
        }
        2 => {
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
                b"%4u: %s: (number) %d\0" as *const u8 as *const ::core::ffi::c_char,
                code as ::core::ffi::c_uint,
                tty_term_codes[code as usize].name,
                (*(*term).codes.offset(code as isize)).value.number,
            );
        }
        3 => {
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
                b"%4u: %s: (flag) %s\0" as *const u8 as *const ::core::ffi::c_char,
                code as ::core::ffi::c_uint,
                tty_term_codes[code as usize].name,
                if (*(*term).codes.offset(code as isize)).value.flag != 0 {
                    b"true\0" as *const u8 as *const ::core::ffi::c_char
                } else {
                    b"false\0" as *const u8 as *const ::core::ffi::c_char
                },
            );
        }
        _ => {}
    }
    return &raw mut s as *mut ::core::ffi::c_char;
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

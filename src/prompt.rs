pub use crate::src::shared::prompt::{
    PROMPT_ACCEPT, PROMPT_BSPACE_EXIT, PROMPT_CLOSE, PROMPT_COMMANDMODE, PROMPT_CONTINUE,
    PROMPT_EDITARROWS, PROMPT_INCREMENTAL, PROMPT_ISMODE, PROMPT_ISPANE, PROMPT_KEY,
    PROMPT_NOFORMAT, PROMPT_NOFREEZE, PROMPT_NTYPES, PROMPT_NUMERIC, PROMPT_QUOTENEXT,
    PROMPT_SINGLE, prompt_free_cb, prompt_input_cb, prompt_result,
};
pub use crate::src::shared::abi::{__compar_fn_t, ssize_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_citem, screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::prompt::*;
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
use crate::src::shared::utf8::*;
extern "C" {
    pub type args;
    pub type tmuxpeer;
    pub type environ;
    pub type options;
    pub type menu_data;
    pub type window_pane_prompt;
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
    pub type options_array_item;
    pub type options_entry;
    fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
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
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strlcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
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
    static mut global_options: *mut options;
    static mut global_s_options: *mut options;
    fn paste_buffer_data(_: *mut paste_buffer, _: *mut size_t) -> *const ::core::ffi::c_char;
    fn paste_get_top(_: *mut *mut ::core::ffi::c_char) -> *mut paste_buffer;
    fn format_free(_: *mut format_tree);
    fn format_add(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn format_expand_time(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn format_create_defaults(
        _: *mut cmdq_item,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut format_tree;
    fn format_create_from_state(
        _: *mut cmdq_item,
        _: *mut client,
        _: *mut cmd_find_state,
    ) -> *mut format_tree;
    fn format_draw(
        _: *mut screen_write_ctx,
        _: *const grid_cell,
        _: u_int,
        _: *const ::core::ffi::c_char,
        _: *mut style_ranges,
        _: ::core::ffi::c_int,
    );
    fn format_width(_: *const ::core::ffi::c_char) -> u_int;
    fn options_get_only(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_array_first(_: *mut options_entry) -> *mut options_array_item;
    fn options_array_next(_: *mut options_array_item) -> *mut options_array_item;
    fn options_array_item_value(_: *mut options_array_item) -> *mut options_value;
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn cmd_find_clear_state(_: *mut cmd_find_state, _: ::core::ffi::c_int);
    fn cmd_find_valid_state(_: *mut cmd_find_state) -> ::core::ffi::c_int;
    fn cmd_find_copy_state(_: *mut cmd_find_state, _: *mut cmd_find_state);
    static mut cmd_table: [*const cmd_entry; 0];
    fn key_string_lookup_key(_: key_code, _: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn prompt_up_history(_: *mut u_int, _: u_int) -> *const ::core::ffi::c_char;
    fn prompt_down_history(_: *mut u_int, _: u_int) -> *const ::core::ffi::c_char;
    fn prompt_add_history(_: *const ::core::ffi::c_char, _: u_int);
    static grid_default_cell: grid_cell;
    fn screen_write_clearcharacter(_: *mut screen_write_ctx, _: u_int, _: u_int);
    fn screen_write_cursormove(
        _: *mut screen_write_ctx,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn screen_write_cell(_: *mut screen_write_ctx, _: *const grid_cell);
    fn screen_set_cursor_style(_: u_int, _: *mut screen_cursor_style, _: *mut ::core::ffi::c_int);
    fn utf8_to_data(_: utf8_char, _: *mut utf8_data);
    fn utf8_set(_: *mut utf8_data, _: u_char);
    fn utf8_copy(_: *mut utf8_data, _: *const utf8_data);
    fn utf8_open(_: *mut utf8_data, _: u_char) -> utf8_state;
    fn utf8_append(_: *mut utf8_data, _: u_char) -> utf8_state;
    fn utf8_strlen(_: *const utf8_data) -> size_t;
    fn utf8_strwidth(_: *const utf8_data, _: ssize_t) -> u_int;
    fn utf8_fromcstr(_: *const ::core::ffi::c_char) -> *mut utf8_data;
    fn utf8_tocstr(_: *mut utf8_data) -> *mut ::core::ffi::c_char;
    fn utf8_cstrwidth(_: *const ::core::ffi::c_char) -> u_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
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
pub struct prompt {
    pub string: *mut ::core::ffi::c_char,
    pub buffer: *mut utf8_data,
    pub state: cmd_find_state,
    pub last: *mut ::core::ffi::c_char,
    pub index: size_t,
    pub inputcb: prompt_input_cb,
    pub freecb: prompt_free_cb,
    pub data: *mut ::core::ffi::c_void,
    pub message_format: *mut ::core::ffi::c_char,
    pub keys: ::core::ffi::c_int,
    pub word_separators: *mut ::core::ffi::c_char,
    pub style: grid_cell,
    pub command_style: grid_cell,
    pub style_str: *mut ::core::ffi::c_char,
    pub command_style_str: *mut ::core::ffi::c_char,
    pub cstyle: screen_cursor_style,
    pub command_cstyle: screen_cursor_style,
    pub ccolour: ::core::ffi::c_int,
    pub command_ccolour: ::core::ffi::c_int,
    pub cmode: ::core::ffi::c_int,
    pub command_cmode: ::core::ffi::c_int,
    pub type_0: prompt_type,
    pub flags: ::core::ffi::c_int,
    pub closed: ::core::ffi::c_int,
    pub hindex: [u_int; 2],
    pub copied: *mut utf8_data,
    pub complete_list: *mut *mut ::core::ffi::c_char,
    pub complete_size: u_int,
    pub complete_display: *mut ::core::ffi::c_char,
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
pub struct prompt_create_data {
    pub fs: *mut cmd_find_state,
    pub prompt: *const ::core::ffi::c_char,
    pub input: *const ::core::ffi::c_char,
    pub type_0: prompt_type,
    pub flags: ::core::ffi::c_int,
    pub style: grid_cell,
    pub command_style: grid_cell,
    pub style_str: *const ::core::ffi::c_char,
    pub command_style_str: *const ::core::ffi::c_char,
    pub cstyle: screen_cursor_style,
    pub command_cstyle: screen_cursor_style,
    pub ccolour: ::core::ffi::c_int,
    pub command_ccolour: ::core::ffi::c_int,
    pub cmode: ::core::ffi::c_int,
    pub command_cmode: ::core::ffi::c_int,
    pub message_format: *const ::core::ffi::c_char,
    pub keys: ::core::ffi::c_int,
    pub word_separators: *const ::core::ffi::c_char,
    pub inputcb: prompt_input_cb,
    pub freecb: prompt_free_cb,
    pub data: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct prompt_draw_data {
    pub ctx: *mut screen_write_ctx,
    pub cursor_x: *mut u_int,
    pub area_x: u_int,
    pub area_width: u_int,
    pub prompt_line: u_int,
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct prompt_layout {
    pub area_x: u_int,
    pub area_width: u_int,
    pub content_x: u_int,
    pub content_width: u_int,
    pub label_width: u_int,
    pub input_x: u_int,
    pub cursor_x: u_int,
    pub input_offset: u_int,
    pub input_width: u_int,
}
pub const MODEKEY_VI: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
unsafe extern "C" fn prompt_flags_to_string(
    mut flags: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    static mut tmp: [::core::ffi::c_char; 256] = [0; 256];
    *(&raw mut tmp as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if flags & PROMPT_SINGLE != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"SINGLE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_NUMERIC != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"NUMERIC,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_INCREMENTAL != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"INCREMENTAL,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_NOFORMAT != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"NOFORMAT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_KEY != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"KEY,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_ACCEPT != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"ACCEPT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_QUOTENEXT != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"QUOTENEXT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_BSPACE_EXIT != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"BSPACE_EXIT,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_NOFREEZE != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"NOFREEZE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_COMMANDMODE != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"COMMANDMODE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_ISPANE != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"ISPANE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_ISMODE != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"ISMODE,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if flags & PROMPT_EDITARROWS != 0 {
        strlcat(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"EDITARROWS,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if *(&raw mut tmp as *mut ::core::ffi::c_char) as ::core::ffi::c_int != '\0' as i32 {
        tmp[strlen(&raw mut tmp as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
            '\0' as i32 as ::core::ffi::c_char;
    }
    return &raw mut tmp as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn prompt_set_options(mut pd: *mut prompt_create_data, mut s: *mut session) {
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
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
    let mut n: u_int = 0;
    if !s.is_null() {
        oo = (*s).options;
    } else {
        oo = global_s_options;
    }
    style_apply(
        &raw mut (*pd).style,
        oo,
        b"message-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    style_apply(
        &raw mut (*pd).command_style,
        oo,
        b"message-command-style\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    (*pd).style_str = options_get_string(
        oo,
        b"message-style\0" as *const u8 as *const ::core::ffi::c_char,
    );
    (*pd).command_style_str = options_get_string(
        oo,
        b"message-command-style\0" as *const u8 as *const ::core::ffi::c_char,
    );
    n = options_get_number(
        oo,
        b"prompt-cursor-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    screen_set_cursor_style(n, &raw mut (*pd).cstyle, &raw mut (*pd).cmode);
    n = options_get_number(
        oo,
        b"prompt-command-cursor-style\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    screen_set_cursor_style(
        n,
        &raw mut (*pd).command_cstyle,
        &raw mut (*pd).command_cmode,
    );
    style_apply(
        &raw mut gc,
        oo,
        b"prompt-cursor-colour\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    (*pd).ccolour = gc.fg;
    style_apply(
        &raw mut gc,
        oo,
        b"prompt-command-cursor-colour\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<format_tree>(),
    );
    (*pd).command_ccolour = gc.fg;
    (*pd).message_format = options_get_string(
        oo,
        b"message-format\0" as *const u8 as *const ::core::ffi::c_char,
    );
    (*pd).keys = options_get_number(
        oo,
        b"status-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    (*pd).word_separators = options_get_string(
        oo,
        b"word-separators\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn prompt_create(mut pd: *const prompt_create_data) -> *mut prompt {
    let mut pr: *mut prompt = ::core::ptr::null_mut::<prompt>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut input: *const ::core::ffi::c_char = (*pd).input;
    let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    pr = xcalloc(1 as size_t, ::core::mem::size_of::<prompt>() as size_t) as *mut prompt;
    if !(*pd).fs.is_null() {
        ft = format_create_from_state(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<client>(),
            (*pd).fs,
        );
        cmd_find_copy_state(&raw mut (*pr).state, (*pd).fs);
    } else {
        ft = format_create_defaults(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
        cmd_find_clear_state(&raw mut (*pr).state, 0 as ::core::ffi::c_int);
    }
    if input.is_null() {
        input = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    (*pr).string = xstrdup((*pd).prompt);
    if (*pd).flags & PROMPT_NOFORMAT != 0 {
        tmp = xstrdup(input);
    } else {
        tmp = format_expand_time(ft, input);
    }
    if (*pd).flags & PROMPT_INCREMENTAL != 0 {
        (*pr).last = xstrdup(tmp);
        (*pr).buffer = utf8_fromcstr(b"\0" as *const u8 as *const ::core::ffi::c_char);
    } else {
        (*pr).last = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*pr).buffer = utf8_fromcstr(tmp);
    }
    (*pr).index = utf8_strlen((*pr).buffer);
    free(tmp as *mut ::core::ffi::c_void);
    (*pr).inputcb = (*pd).inputcb;
    (*pr).freecb = (*pd).freecb;
    (*pr).data = (*pd).data;
    (*pr).flags = (*pd).flags;
    (*pr).type_0 = (*pd).type_0;
    memcpy(
        &raw mut (*pr).style as *mut ::core::ffi::c_void,
        &raw const (*pd).style as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    memcpy(
        &raw mut (*pr).command_style as *mut ::core::ffi::c_void,
        &raw const (*pd).command_style as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    (*pr).style_str = xstrdup((*pd).style_str);
    (*pr).command_style_str = xstrdup((*pd).command_style_str);
    (*pr).cstyle = (*pd).cstyle;
    (*pr).command_cstyle = (*pd).command_cstyle;
    (*pr).ccolour = (*pd).ccolour;
    (*pr).command_ccolour = (*pd).command_ccolour;
    (*pr).cmode = (*pd).cmode;
    (*pr).command_cmode = (*pd).command_cmode;
    (*pr).message_format = xstrdup((*pd).message_format);
    (*pr).keys = (*pd).keys;
    (*pr).word_separators = xstrdup((*pd).word_separators);
    format_free(ft);
    return pr;
}
#[no_mangle]
pub unsafe extern "C" fn prompt_free(mut pr: *mut prompt) {
    if !pr.is_null() {
        if (*pr).freecb.is_some() && !(*pr).data.is_null() {
            (*pr).freecb.expect("non-null function pointer")((*pr).data);
        }
        free((*pr).message_format as *mut ::core::ffi::c_void);
        free((*pr).style_str as *mut ::core::ffi::c_void);
        free((*pr).command_style_str as *mut ::core::ffi::c_void);
        free((*pr).word_separators as *mut ::core::ffi::c_void);
        free((*pr).last as *mut ::core::ffi::c_void);
        free((*pr).string as *mut ::core::ffi::c_void);
        free((*pr).buffer as *mut ::core::ffi::c_void);
        free((*pr).copied as *mut ::core::ffi::c_void);
        prompt_clear_complete(pr);
        free(pr as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn prompt_fire_callback(
    mut pr: *mut prompt,
    mut s: *const ::core::ffi::c_char,
    mut type_0: prompt_key_result,
    mut redraw: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut result: prompt_result = PROMPT_CONTINUE;
    result = (*pr).inputcb.expect("non-null function pointer")((*pr).data, s, type_0);
    if result as ::core::ffi::c_uint == PROMPT_CLOSE as ::core::ffi::c_int as ::core::ffi::c_uint {
        (*pr).closed = 1 as ::core::ffi::c_int;
        return 1 as ::core::ffi::c_int;
    }
    if !redraw.is_null() {
        *redraw = 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn prompt_incremental_start(mut pr: *mut prompt) {
    let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*pr).flags & PROMPT_INCREMENTAL != 0 {
        tmp = utf8_tocstr((*pr).buffer);
        xasprintf(
            &raw mut cp,
            b"=%s\0" as *const u8 as *const ::core::ffi::c_char,
            tmp,
        );
        prompt_fire_callback(
            pr,
            cp,
            PROMPT_KEY_HANDLED,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        free(cp as *mut ::core::ffi::c_void);
        free(tmp as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn prompt_update(
    mut pr: *mut prompt,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
) {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if cmd_find_valid_state(&raw mut (*pr).state) != 0 {
        ft = format_create_from_state(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<client>(),
            &raw mut (*pr).state,
        );
    } else {
        ft = format_create_defaults(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
    }
    free((*pr).string as *mut ::core::ffi::c_void);
    (*pr).string = xstrdup(msg);
    if input.is_null() {
        input = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    free((*pr).buffer as *mut ::core::ffi::c_void);
    if (*pr).flags & PROMPT_NOFORMAT != 0 {
        tmp = xstrdup(input);
    } else {
        tmp = format_expand_time(ft, input);
    }
    (*pr).buffer = utf8_fromcstr(tmp);
    (*pr).index = utf8_strlen((*pr).buffer);
    free(tmp as *mut ::core::ffi::c_void);
    memset(
        &raw mut (*pr).hindex as *mut u_int as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[u_int; 2]>() as size_t,
    );
    (*pr).closed = 0 as ::core::ffi::c_int;
    prompt_clear_complete(pr);
    format_free(ft);
}
#[no_mangle]
pub unsafe extern "C" fn prompt_closed(mut pr: *mut prompt) -> ::core::ffi::c_int {
    return (*pr).closed;
}
unsafe extern "C" fn prompt_redraw_character(
    mut ctx: *mut screen_write_ctx,
    mut offset: u_int,
    mut pwidth: u_int,
    mut width: *mut u_int,
    mut gc: *mut grid_cell,
    mut ud: *const utf8_data,
) -> ::core::ffi::c_int {
    let mut ch: u_char = 0;
    if *width < offset {
        *width = (*width).wrapping_add((*ud).width as u_int);
        return 1 as ::core::ffi::c_int;
    }
    if *width >= offset.wrapping_add(pwidth) {
        return 0 as ::core::ffi::c_int;
    }
    *width = (*width).wrapping_add((*ud).width as u_int);
    if *width > offset.wrapping_add(pwidth) {
        return 0 as ::core::ffi::c_int;
    }
    ch = *(&raw const (*ud).data as *const u_char);
    if (*ud).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        && (ch as ::core::ffi::c_int <= 0x1f as ::core::ffi::c_int
            || ch as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int)
    {
        (*gc).data.data[0 as ::core::ffi::c_int as usize] = '^' as i32 as u_char;
        (*gc).data.data[1 as ::core::ffi::c_int as usize] =
            (if ch as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int {
                '?' as i32
            } else {
                ch as ::core::ffi::c_int | 0x40 as ::core::ffi::c_int
            }) as u_char;
        (*gc).data.have = 2 as u_char;
        (*gc).data.size = (*gc).data.have;
        (*gc).data.width = 2 as u_char;
    } else {
        utf8_copy(&raw mut (*gc).data, ud);
    }
    screen_write_cell(ctx, gc);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn prompt_redraw_quote(
    mut pr: *const prompt,
    mut pcursor: u_int,
    mut input_x: u_int,
    mut ctx: *mut screen_write_ctx,
    mut offset: u_int,
    mut pw: u_int,
    mut w: *mut u_int,
    mut gc: *mut grid_cell,
) -> ::core::ffi::c_int {
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    if (*pr).flags & PROMPT_QUOTENEXT != 0
        && pcursor >= offset
        && (*(*ctx).s).cx == input_x.wrapping_add(pcursor).wrapping_sub(offset)
    {
        utf8_set(&raw mut ud, '^' as i32 as u_char);
        return prompt_redraw_character(ctx, offset, pw, w, gc, &raw mut ud);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn prompt_draw_complete(
    mut pr: *mut prompt,
    mut ctx: *mut screen_write_ctx,
    mut ax: u_int,
    mut aw: u_int,
    mut cx: u_int,
    mut py: u_int,
    mut base: *const grid_cell,
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
    let mut ud: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut avail: u_int = 0;
    let mut width: u_int = 0;
    let mut i: u_int = 0;
    if (*pr).complete_display.is_null() {
        return;
    }
    if (*pr).index != utf8_strlen((*pr).buffer) {
        return;
    }
    if cx < ax || cx.wrapping_sub(ax) >= aw {
        return;
    }
    avail = aw.wrapping_sub(cx.wrapping_sub(ax));
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        base as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    gc.attr = (gc.attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE) as u_short;
    screen_write_cursormove(
        ctx,
        cx as ::core::ffi::c_int,
        py as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    width = 0 as u_int;
    ud = utf8_fromcstr((*pr).complete_display);
    i = 0 as u_int;
    while (*ud.offset(i as isize)).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        if width.wrapping_add((*ud.offset(i as isize)).width as u_int) > avail {
            break;
        }
        utf8_copy(&raw mut gc.data, ud.offset(i as isize) as *mut utf8_data);
        screen_write_cell(ctx, &raw mut gc);
        width = width.wrapping_add((*ud.offset(i as isize)).width as u_int);
        i = i.wrapping_add(1);
    }
    free(ud as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn prompt_format_tree(mut pr: *mut prompt) -> *mut format_tree {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if cmd_find_valid_state(&raw mut (*pr).state) != 0 {
        ft = format_create_from_state(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<client>(),
            &raw mut (*pr).state,
        );
    } else {
        ft = format_create_defaults(
            ::core::ptr::null_mut::<cmdq_item>(),
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<session>(),
            ::core::ptr::null_mut::<winlink>(),
            ::core::ptr::null_mut::<window_pane>(),
        );
    }
    tmp = utf8_tocstr((*pr).buffer);
    format_add(
        ft,
        b"prompt_input\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        tmp,
    );
    free(tmp as *mut ::core::ffi::c_void);
    format_add(
        ft,
        b"prompt_flags\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        prompt_flags_to_string((*pr).flags),
    );
    format_add(
        ft,
        b"prompt_type\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        prompt_type_string((*pr).type_0),
    );
    if (*pr).flags & PROMPT_COMMANDMODE != 0 {
        format_add(
            ft,
            b"command_prompt\0" as *const u8 as *const ::core::ffi::c_char,
            b"1\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        format_add(
            ft,
            b"command_prompt\0" as *const u8 as *const ::core::ffi::c_char,
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    return ft;
}
unsafe extern "C" fn prompt_expand1(
    mut pr: *mut prompt,
    mut ft: *mut format_tree,
) -> *mut ::core::ffi::c_char {
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut prompt: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    prompt = format_expand_time(ft, (*pr).string);
    format_add(
        ft,
        b"message\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        prompt,
    );
    expanded = format_expand_time(ft, (*pr).message_format);
    free(prompt as *mut ::core::ffi::c_void);
    return expanded;
}
unsafe extern "C" fn prompt_effective_style(
    mut pr: *mut prompt,
    mut sy: *mut style,
    mut ft: *mut format_tree,
) {
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut gc: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*pr).flags & PROMPT_COMMANDMODE != 0 {
        s = (*pr).command_style_str;
        gc = &raw mut (*pr).command_style;
    } else {
        s = (*pr).style_str;
        gc = &raw mut (*pr).style;
    }
    style_set(sy, gc);
    if !s.is_null() {
        expanded = format_expand_time(ft, s);
        if style_parse(sy, &raw const grid_default_cell, expanded) != 0 as ::core::ffi::c_int {
            style_set(sy, gc);
        }
        free(expanded as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn prompt_layout(
    mut pr: *mut prompt,
    mut ax: u_int,
    mut aw: u_int,
    mut pl: *mut prompt_layout,
    mut expanded: *mut *mut ::core::ffi::c_char,
    mut sy: *mut style,
) {
    let mut local: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut pcursor: u_int = 0;
    let mut pwidth: u_int = 0;
    let mut end: u_int = 0;
    let mut width: u_int = 0;
    let mut offset: u_int = 0;
    let mut avail: u_int = 0;
    memset(
        pl as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<prompt_layout>() as size_t,
    );
    (*pl).area_x = ax;
    (*pl).area_width = aw;
    ft = prompt_format_tree(pr);
    if !sy.is_null() {
        prompt_effective_style(pr, sy, ft);
    }
    if !expanded.is_null() {
        *expanded = prompt_expand1(pr, ft);
    } else {
        local = prompt_expand1(pr, ft);
        expanded = &raw mut local;
    }
    format_free(ft);
    if aw == 0 as u_int {
        free(local as *mut ::core::ffi::c_void);
        return;
    }
    (*pl).label_width = format_width(*expanded);
    if (*pl).label_width > aw {
        (*pl).label_width = aw;
    }
    pcursor = utf8_strwidth((*pr).buffer, (*pr).index as ssize_t);
    pwidth = utf8_strwidth((*pr).buffer, -(1 as ::core::ffi::c_int) as ssize_t);
    if (*pr).flags & PROMPT_QUOTENEXT != 0 {
        pwidth = pwidth.wrapping_add(1);
    }
    avail = aw.wrapping_sub((*pl).label_width);
    if avail == 0 as u_int {
        (*pl).input_offset = 0 as u_int;
        (*pl).input_width = 0 as u_int;
        (*pl).cursor_x = (*pl).label_width;
    } else {
        if pcursor >= avail {
            offset = pcursor.wrapping_sub(avail).wrapping_add(1 as u_int);
            width = avail;
        } else {
            offset = 0 as u_int;
            width = pwidth;
        }
        if width > avail {
            width = avail;
        }
        (*pl).input_offset = offset;
        (*pl).input_width = width;
        (*pl).cursor_x = (*pl).label_width.wrapping_add(pcursor).wrapping_sub(offset);
    }
    (*pl).content_width = (*pl).label_width.wrapping_add((*pl).input_width);
    if !(*pr).complete_display.is_null()
        && (*pr).index == utf8_strlen((*pr).buffer)
        && (*pl).cursor_x < aw
    {
        avail = aw.wrapping_sub((*pl).cursor_x);
        width = utf8_cstrwidth((*pr).complete_display);
        if width > avail {
            width = avail;
        }
        end = (*pl).cursor_x.wrapping_add(width);
        if end > (*pl).content_width {
            (*pl).content_width = end;
        }
    }
    if (*pl).content_width > aw {
        (*pl).content_width = aw;
    }
    if !sy.is_null() {
        match (*sy).align as ::core::ffi::c_uint {
            2 | 4 => {
                (*pl).content_x = ax.wrapping_add(
                    aw.wrapping_sub((*pl).content_width)
                        .wrapping_div(2 as u_int),
                );
            }
            3 => {
                (*pl).content_x = ax.wrapping_add(aw).wrapping_sub((*pl).content_width);
            }
            _ => {
                (*pl).content_x = ax;
            }
        }
    } else {
        (*pl).content_x = ax;
    }
    (*pl).input_x = (*pl).content_x.wrapping_add((*pl).label_width);
    (*pl).cursor_x = (*pl).cursor_x.wrapping_add((*pl).content_x);
    free(local as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn prompt_mouse_complete(
    mut pr: *mut prompt,
    mut x: u_int,
    mut cx: u_int,
    mut ax: u_int,
    mut aw: u_int,
    mut redraw: *mut ::core::ffi::c_int,
) -> prompt_key_result {
    let mut replace: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut avail: u_int = 0;
    let mut clicked: u_int = 0;
    let mut end: u_int = 0;
    let mut i: u_int = 0;
    let mut start: u_int = 0;
    let mut width: u_int = 0;
    if (*pr).complete_display.is_null() || (*pr).complete_size == 0 as u_int {
        return PROMPT_KEY_NOT_HANDLED;
    }
    if (*pr).index != utf8_strlen((*pr).buffer) {
        return PROMPT_KEY_NOT_HANDLED;
    }
    if cx < ax || cx.wrapping_sub(ax) >= aw || x < cx {
        return PROMPT_KEY_NOT_HANDLED;
    }
    avail = aw.wrapping_sub(cx.wrapping_sub(ax));
    clicked = x.wrapping_sub(cx);
    width = utf8_cstrwidth((*pr).complete_display);
    if width > avail {
        width = avail;
    }
    if clicked >= width {
        return PROMPT_KEY_NOT_HANDLED;
    }
    end = 0 as u_int;
    i = 0 as u_int;
    while i < (*pr).complete_size {
        start = end.wrapping_add(1 as u_int);
        end = start.wrapping_add(utf8_cstrwidth(*(*pr).complete_list.offset(i as isize)));
        if clicked < start || clicked >= end {
            i = i.wrapping_add(1);
        } else {
            xasprintf(
                &raw mut replace,
                b"%s \0" as *const u8 as *const ::core::ffi::c_char,
                *(*pr).complete_list.offset(i as isize),
            );
            if prompt_replace_complete(pr, replace) != 0 {
                prompt_clear_complete(pr);
                if !redraw.is_null() {
                    *redraw = 1 as ::core::ffi::c_int;
                }
            }
            free(replace as *mut ::core::ffi::c_void);
            return PROMPT_KEY_HANDLED;
        }
    }
    return PROMPT_KEY_HANDLED;
}
#[no_mangle]
pub unsafe extern "C" fn prompt_draw(mut pr: *mut prompt, mut pd: *mut prompt_draw_data) {
    let mut ctx: *mut screen_write_ctx = (*pd).ctx;
    let mut s: *mut screen = (*ctx).s;
    let mut ax: u_int = (*pd).area_x;
    let mut py: u_int = (*pd).prompt_line;
    let mut cx: *mut u_int = ::core::ptr::null_mut::<u_int>();
    let mut aw: u_int = (*pd).area_width;
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
    let mut pl: prompt_layout = prompt_layout {
        area_x: 0,
        area_width: 0,
        content_x: 0,
        content_width: 0,
        label_width: 0,
        input_x: 0,
        cursor_x: 0,
        input_offset: 0,
        input_width: 0,
    };
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
    let mut i: u_int = 0;
    let mut width: u_int = 0;
    let mut pcursor: u_int = 0;
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*pr).flags & PROMPT_COMMANDMODE != 0 {
        (*s).default_cstyle = (*pr).command_cstyle;
        (*s).default_mode = (*pr).command_cmode;
        (*s).default_ccolour = (*pr).command_ccolour;
    } else {
        (*s).default_cstyle = (*pr).cstyle;
        (*s).default_mode = (*pr).cmode;
        (*s).default_ccolour = (*pr).ccolour;
    }
    prompt_layout(pr, ax, aw, &raw mut pl, &raw mut expanded, &raw mut sy);
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw mut sy.gc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    cx = (*pd).cursor_x;
    *cx = pl.cursor_x;
    screen_write_cursormove(
        ctx,
        ax as ::core::ffi::c_int,
        py as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if sy.fill != 8 as ::core::ffi::c_int {
        screen_write_clearcharacter(ctx, aw, sy.fill as u_int);
    }
    pcursor = utf8_strwidth((*pr).buffer, (*pr).index as ssize_t);
    if pl.content_width != 0 as u_int {
        screen_write_cursormove(
            ctx,
            pl.content_x as ::core::ffi::c_int,
            py as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        if pl.label_width != 0 as u_int {
            format_draw(
                ctx,
                &raw mut gc,
                pl.label_width,
                expanded,
                ::core::ptr::null_mut::<style_ranges>(),
                0 as ::core::ffi::c_int,
            );
        }
        screen_write_cursormove(
            ctx,
            pl.input_x as ::core::ffi::c_int,
            py as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        width = 0 as u_int;
        i = 0 as u_int;
        while (*(*pr).buffer.offset(i as isize)).size as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            if prompt_redraw_quote(
                pr,
                pcursor,
                pl.input_x,
                ctx,
                pl.input_offset,
                pl.input_width,
                &raw mut width,
                &raw mut gc,
            ) == 0
            {
                break;
            }
            if prompt_redraw_character(
                ctx,
                pl.input_offset,
                pl.input_width,
                &raw mut width,
                &raw mut gc,
                (*pr).buffer.offset(i as isize) as *mut utf8_data,
            ) == 0
            {
                break;
            }
            i = i.wrapping_add(1);
        }
        prompt_redraw_quote(
            pr,
            pcursor,
            pl.input_x,
            ctx,
            pl.input_offset,
            pl.input_width,
            &raw mut width,
            &raw mut gc,
        );
        prompt_draw_complete(
            pr,
            ctx,
            pl.content_x,
            pl.content_width,
            pl.cursor_x,
            py,
            &raw mut gc,
        );
    }
    free(expanded as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn prompt_mouse(
    mut pr: *mut prompt,
    mut x: u_int,
    mut ax: u_int,
    mut aw: u_int,
    mut redraw: *mut ::core::ffi::c_int,
) -> prompt_key_result {
    let mut ud: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut result: prompt_key_result = PROMPT_KEY_NOT_HANDLED;
    let mut pl: prompt_layout = prompt_layout {
        area_x: 0,
        area_width: 0,
        content_x: 0,
        content_width: 0,
        label_width: 0,
        input_x: 0,
        cursor_x: 0,
        input_offset: 0,
        input_width: 0,
    };
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
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pwidth: u_int = 0;
    let mut width: u_int = 0;
    let mut target: u_int = 0;
    let mut idx: size_t = 0;
    if x < ax || x >= ax.wrapping_add(aw) {
        return PROMPT_KEY_NOT_HANDLED;
    }
    prompt_layout(pr, ax, aw, &raw mut pl, &raw mut expanded, &raw mut sy);
    free(expanded as *mut ::core::ffi::c_void);
    if pl.input_width == 0 as u_int {
        return PROMPT_KEY_HANDLED;
    }
    pwidth = utf8_strwidth((*pr).buffer, -(1 as ::core::ffi::c_int) as ssize_t);
    if (*pr).flags & PROMPT_QUOTENEXT != 0 {
        pwidth = pwidth.wrapping_add(1);
    }
    result = prompt_mouse_complete(pr, x, pl.cursor_x, pl.content_x, pl.content_width, redraw);
    if result as ::core::ffi::c_uint
        != PROMPT_KEY_NOT_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return result;
    }
    if x <= pl.input_x {
        target = pl.input_offset;
    } else {
        target = pl.input_offset.wrapping_add(x).wrapping_sub(pl.input_x);
    }
    if target > pwidth {
        target = pwidth;
    }
    width = 0 as u_int;
    idx = 0 as size_t;
    while (*(*pr).buffer.offset(idx as isize)).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        ud = (*pr).buffer.offset(idx as isize) as *mut utf8_data;
        if width >= target {
            break;
        }
        width = width.wrapping_add((*ud).width as u_int);
        idx = idx.wrapping_add(1);
    }
    if idx == (*pr).index {
        return PROMPT_KEY_HANDLED;
    }
    (*pr).index = idx;
    prompt_clear_complete(pr);
    if !redraw.is_null() {
        *redraw = 1 as ::core::ffi::c_int;
    }
    return PROMPT_KEY_HANDLED;
}
unsafe extern "C" fn prompt_in_list(
    mut ws: *const ::core::ffi::c_char,
    mut ud: *const utf8_data,
) -> ::core::ffi::c_int {
    if (*ud).size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        || (*ud).width as ::core::ffi::c_int != 1 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    return (strchr(
        ws,
        *(&raw const (*ud).data as *const u_char) as ::core::ffi::c_int,
    ) != NULL as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
}
unsafe extern "C" fn prompt_space(mut ud: *const utf8_data) -> ::core::ffi::c_int {
    if (*ud).size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        || (*ud).width as ::core::ffi::c_int != 1 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    return (*(&raw const (*ud).data as *const u_char) as ::core::ffi::c_int == ' ' as i32)
        as ::core::ffi::c_int;
}
unsafe extern "C" fn prompt_keypad_key(mut key: key_code) -> key_code {
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS != 0 {
        return key;
    }
    match key {
        8589934623 => return '/' as i32 as key_code,
        8589934624 => return '*' as i32 as key_code,
        8589934625 => return '-' as i32 as key_code,
        8589934626 => return '7' as i32 as key_code,
        8589934627 => return '8' as i32 as key_code,
        8589934628 => return '9' as i32 as key_code,
        8589934629 => return '+' as i32 as key_code,
        8589934630 => return '4' as i32 as key_code,
        8589934631 => return '5' as i32 as key_code,
        8589934632 => return '6' as i32 as key_code,
        8589934633 => return '1' as i32 as key_code,
        8589934634 => return '2' as i32 as key_code,
        8589934635 => return '3' as i32 as key_code,
        8589934636 => return '\r' as i32 as key_code,
        8589934637 => return '0' as i32 as key_code,
        8589934638 => return '.' as i32 as key_code,
        _ => {}
    }
    return key;
}
unsafe extern "C" fn prompt_translate_key(
    mut pr: *mut prompt,
    mut key: key_code,
    mut new_key: *mut key_code,
    mut redraw: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if !(*pr).flags & PROMPT_COMMANDMODE != 0 {
        match key {
            35184372088929 | 35184372088931 | 35184372088933 | 35184372088935 | 35184372088936
            | 9 | 35184372088939 | 35184372088942 | 35184372088944 | 35184372088948
            | 35184372088949 | 35184372088950 | 35184372088951 | 35184372088953 | 10 | 13
            | 35192962023453 | 35192962023454 | 8589934599 | 8589934613 | 8589934620
            | 8589934615 | 8589934614 | 8589934621 | 8589934622 | 8589934619 => {
                *new_key = key;
                return 1 as ::core::ffi::c_int;
            }
            27 | 35184372088923 => {
                (*pr).flags |= PROMPT_COMMANDMODE;
                if (*pr).index != 0 as size_t {
                    (*pr).index = (*pr).index.wrapping_sub(1);
                }
                *redraw = 1 as ::core::ffi::c_int;
                return 0 as ::core::ffi::c_int;
            }
            _ => {}
        }
        *new_key = key;
        return 2 as ::core::ffi::c_int;
    }
    match key {
        8589934599 => {
            *new_key = KEYC_LEFT as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        65 | 73 | 67 | 115 | 97 => {
            (*pr).flags &= !PROMPT_COMMANDMODE;
            *redraw = 1 as ::core::ffi::c_int;
        }
        83 => {
            (*pr).flags &= !PROMPT_COMMANDMODE;
            *redraw = 1 as ::core::ffi::c_int;
            *new_key = ('u' as i32 as ::core::ffi::c_ulonglong | KEYC_CTRL) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        105 => {
            (*pr).flags &= !PROMPT_COMMANDMODE;
            *redraw = 1 as ::core::ffi::c_int;
            return 0 as ::core::ffi::c_int;
        }
        27 | 35184372088923 => return 0 as ::core::ffi::c_int,
        _ => {}
    }
    match key {
        65 | 36 => {
            *new_key = KEYC_END as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        73 | 48 | 94 => {
            *new_key = KEYC_HOME as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        67 | 68 => {
            *new_key = ('k' as i32 as ::core::ffi::c_ulonglong | KEYC_CTRL) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        8589934599 | 88 => {
            *new_key = KEYC_BSPACE as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        98 => {
            *new_key = ('b' as i32 as ::core::ffi::c_ulonglong | KEYC_META) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        66 => {
            *new_key = ('B' as i32 as ::core::ffi::c_ulonglong | KEYC_VI) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        100 => {
            *new_key = ('u' as i32 as ::core::ffi::c_ulonglong | KEYC_CTRL) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        101 => {
            *new_key = ('e' as i32 as ::core::ffi::c_ulonglong | KEYC_VI) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        69 => {
            *new_key = ('E' as i32 as ::core::ffi::c_ulonglong | KEYC_VI) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        119 => {
            *new_key = ('w' as i32 as ::core::ffi::c_ulonglong | KEYC_VI) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        87 => {
            *new_key = ('W' as i32 as ::core::ffi::c_ulonglong | KEYC_VI) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        112 => {
            *new_key = ('y' as i32 as ::core::ffi::c_ulonglong | KEYC_CTRL) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        113 => {
            *new_key = ('c' as i32 as ::core::ffi::c_ulonglong | KEYC_CTRL) as key_code;
            return 1 as ::core::ffi::c_int;
        }
        115 | 8589934613 | 120 => {
            *new_key = KEYC_DC as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        8589934620 | 106 => {
            *new_key = KEYC_DOWN as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        8589934621 | 104 => {
            *new_key = KEYC_LEFT as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        97 | 8589934622 | 108 => {
            *new_key = KEYC_RIGHT as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        8589934619 | 107 => {
            *new_key = KEYC_UP as ::core::ffi::c_ulong as key_code;
            return 1 as ::core::ffi::c_int;
        }
        35184372088936 | 35184372088931 | 10 | 13 => return 1 as ::core::ffi::c_int,
        _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn prompt_paste(mut pr: *mut prompt) -> ::core::ffi::c_int {
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut bufdata: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut size: size_t = 0;
    let mut n: size_t = 0;
    let mut bufsize: size_t = 0;
    let mut i: u_int = 0;
    let mut ud: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut udp: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut more: utf8_state = UTF8_MORE;
    size = utf8_strlen((*pr).buffer);
    if !(*pr).copied.is_null() {
        ud = (*pr).copied;
        n = utf8_strlen((*pr).copied);
    } else {
        pb = paste_get_top(::core::ptr::null_mut::<*mut ::core::ffi::c_char>());
        if pb.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        bufdata = paste_buffer_data(pb, &raw mut bufsize);
        udp = xreallocarray(
            NULL,
            bufsize.wrapping_add(1 as size_t),
            ::core::mem::size_of::<utf8_data>() as size_t,
        ) as *mut utf8_data;
        ud = udp;
        i = 0 as u_int;
        while i as size_t != bufsize {
            more = utf8_open(udp, *bufdata.offset(i as isize) as u_char);
            if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                loop {
                    i = i.wrapping_add(1);
                    if !(i as size_t != bufsize
                        && more as ::core::ffi::c_uint
                            == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        break;
                    }
                    more = utf8_append(udp, *bufdata.offset(i as isize) as u_char);
                }
                if more as ::core::ffi::c_uint
                    == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    udp = udp.offset(1);
                    continue;
                } else {
                    i = i.wrapping_sub((*udp).have as u_int);
                }
            }
            if *bufdata.offset(i as isize) as ::core::ffi::c_int <= 31 as ::core::ffi::c_int
                || *bufdata.offset(i as isize) as ::core::ffi::c_int >= 127 as ::core::ffi::c_int
            {
                break;
            }
            utf8_set(udp, *bufdata.offset(i as isize) as u_char);
            udp = udp.offset(1);
            i = i.wrapping_add(1);
        }
        (*udp).size = 0 as u_char;
        n = udp.offset_from(ud) as ::core::ffi::c_long as size_t;
    }
    if n != 0 as size_t {
        (*pr).buffer = xreallocarray(
            (*pr).buffer as *mut ::core::ffi::c_void,
            size.wrapping_add(n).wrapping_add(1 as size_t),
            ::core::mem::size_of::<utf8_data>() as size_t,
        ) as *mut utf8_data;
        if (*pr).index == size {
            memcpy(
                (*pr).buffer.offset((*pr).index as isize) as *mut ::core::ffi::c_void,
                ud as *const ::core::ffi::c_void,
                n.wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
            );
            (*pr).index = (*pr).index.wrapping_add(n);
            (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
        } else {
            memmove(
                (*pr).buffer.offset((*pr).index as isize).offset(n as isize)
                    as *mut ::core::ffi::c_void,
                (*pr).buffer.offset((*pr).index as isize) as *const ::core::ffi::c_void,
                size.wrapping_add(1 as size_t)
                    .wrapping_sub((*pr).index)
                    .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
            );
            memcpy(
                (*pr).buffer.offset((*pr).index as isize) as *mut ::core::ffi::c_void,
                ud as *const ::core::ffi::c_void,
                n.wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
            );
            (*pr).index = (*pr).index.wrapping_add(n);
        }
    }
    if ud != (*pr).copied {
        free(ud as *mut ::core::ffi::c_void);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn prompt_replace_complete(
    mut pr: *mut prompt,
    mut s: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut word: [::core::ffi::c_char; 64] = [0; 64];
    let mut allocated: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut size: size_t = 0;
    let mut n: size_t = 0;
    let mut off: size_t = 0;
    let mut idx: size_t = 0;
    let mut used: size_t = 0;
    let mut first: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut last: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    let mut ud: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
    idx = (*pr).index;
    if idx != 0 as size_t {
        idx = idx.wrapping_sub(1);
    }
    size = utf8_strlen((*pr).buffer);
    first = (*pr).buffer.offset(idx as isize) as *mut utf8_data;
    while first > (*pr).buffer && prompt_space(first) == 0 {
        first = first.offset(-1);
    }
    while (*first).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int && prompt_space(first) != 0
    {
        first = first.offset(1);
    }
    last = (*pr).buffer.offset(idx as isize) as *mut utf8_data;
    while (*last).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int && prompt_space(last) == 0 {
        last = last.offset(1);
    }
    while last > (*pr).buffer && prompt_space(last) != 0 {
        last = last.offset(-1);
    }
    if (*last).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        last = last.offset(1);
    }
    if last < first {
        return 0 as ::core::ffi::c_int;
    }
    if s.is_null() {
        used = 0 as size_t;
        ud = first;
        while ud < last {
            if used.wrapping_add((*ud).size as size_t)
                >= ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as usize
            {
                break;
            }
            memcpy(
                (&raw mut word as *mut ::core::ffi::c_char).offset(used as isize)
                    as *mut ::core::ffi::c_void,
                &raw mut (*ud).data as *mut u_char as *const ::core::ffi::c_void,
                (*ud).size as size_t,
            );
            used = used.wrapping_add((*ud).size as size_t);
            ud = ud.offset(1);
        }
        if ud != last {
            return 0 as ::core::ffi::c_int;
        }
        word[used as usize] = '\0' as i32 as ::core::ffi::c_char;
    }
    if s.is_null() {
        allocated = prompt_complete(
            pr,
            &raw mut word as *mut ::core::ffi::c_char,
            first.offset_from((*pr).buffer) as ::core::ffi::c_long as u_int,
        );
        if allocated.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        s = allocated;
    }
    n = size
        .wrapping_sub(last.offset_from((*pr).buffer) as ::core::ffi::c_long as size_t)
        .wrapping_add(1 as size_t);
    memmove(
        first as *mut ::core::ffi::c_void,
        last as *const ::core::ffi::c_void,
        n.wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
    );
    size = size.wrapping_sub(last.offset_from(first) as ::core::ffi::c_long as size_t);
    size = size.wrapping_add(strlen(s));
    off = first.offset_from((*pr).buffer) as ::core::ffi::c_long as size_t;
    (*pr).buffer = xreallocarray(
        (*pr).buffer as *mut ::core::ffi::c_void,
        size.wrapping_add(1 as size_t),
        ::core::mem::size_of::<utf8_data>() as size_t,
    ) as *mut utf8_data;
    first = (*pr).buffer.offset(off as isize);
    memmove(
        first.offset(strlen(s) as isize) as *mut ::core::ffi::c_void,
        first as *const ::core::ffi::c_void,
        n.wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
    );
    idx = 0 as size_t;
    while idx < strlen(s) {
        utf8_set(
            first.offset(idx as isize) as *mut utf8_data,
            *s.offset(idx as isize) as u_char,
        );
        idx = idx.wrapping_add(1);
    }
    (*pr).index =
        (first.offset_from((*pr).buffer) as ::core::ffi::c_long as size_t).wrapping_add(strlen(s));
    free(allocated as *mut ::core::ffi::c_void);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn prompt_forward_word(
    mut pr: *mut prompt,
    mut size: size_t,
    mut vi: ::core::ffi::c_int,
    mut separators: *const ::core::ffi::c_char,
) {
    let mut idx: size_t = (*pr).index;
    let mut word_is_separators: ::core::ffi::c_int = 0;
    if vi == 0 {
        while idx != size && prompt_space((*pr).buffer.offset(idx as isize) as *mut utf8_data) != 0
        {
            idx = idx.wrapping_add(1);
        }
    }
    if idx == size {
        (*pr).index = idx;
        return;
    }
    word_is_separators = (prompt_in_list(
        separators,
        (*pr).buffer.offset(idx as isize) as *mut utf8_data,
    ) != 0
        && prompt_space((*pr).buffer.offset(idx as isize) as *mut utf8_data) == 0)
        as ::core::ffi::c_int;
    loop {
        idx = idx.wrapping_add(1);
        if prompt_space((*pr).buffer.offset(idx as isize) as *mut utf8_data) != 0 {
            if vi != 0 {
                while idx != size
                    && prompt_space((*pr).buffer.offset(idx as isize) as *mut utf8_data) != 0
                {
                    idx = idx.wrapping_add(1);
                }
            }
            break;
        } else if !(idx != size
            && word_is_separators
                == prompt_in_list(
                    separators,
                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                ))
        {
            break;
        }
    }
    (*pr).index = idx;
}
unsafe extern "C" fn prompt_end_word(
    mut pr: *mut prompt,
    mut size: size_t,
    mut separators: *const ::core::ffi::c_char,
) {
    let mut idx: size_t = (*pr).index;
    let mut word_is_separators: ::core::ffi::c_int = 0;
    if idx == size {
        return;
    }
    loop {
        idx = idx.wrapping_add(1);
        if idx == size {
            (*pr).index = idx;
            return;
        }
        if !(prompt_space((*pr).buffer.offset(idx as isize) as *mut utf8_data) != 0) {
            break;
        }
    }
    word_is_separators = prompt_in_list(
        separators,
        (*pr).buffer.offset(idx as isize) as *mut utf8_data,
    );
    loop {
        idx = idx.wrapping_add(1);
        if idx == size {
            break;
        }
        if !(prompt_space((*pr).buffer.offset(idx as isize) as *mut utf8_data) == 0
            && word_is_separators
                == prompt_in_list(
                    separators,
                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                ))
        {
            break;
        }
    }
    (*pr).index = idx.wrapping_sub(1 as size_t);
}
unsafe extern "C" fn prompt_backward_word(
    mut pr: *mut prompt,
    mut separators: *const ::core::ffi::c_char,
) {
    let mut idx: size_t = (*pr).index;
    let mut word_is_separators: ::core::ffi::c_int = 0;
    while idx != 0 as size_t {
        idx = idx.wrapping_sub(1);
        if prompt_space((*pr).buffer.offset(idx as isize) as *mut utf8_data) == 0 {
            break;
        }
    }
    word_is_separators = prompt_in_list(
        separators,
        (*pr).buffer.offset(idx as isize) as *mut utf8_data,
    );
    while idx != 0 as size_t {
        idx = idx.wrapping_sub(1);
        if !(prompt_space((*pr).buffer.offset(idx as isize) as *mut utf8_data) != 0
            || word_is_separators
                != prompt_in_list(
                    separators,
                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                ))
        {
            continue;
        }
        idx = idx.wrapping_add(1);
        break;
    }
    (*pr).index = idx;
}
unsafe extern "C" fn prompt_done(
    mut pr: *mut prompt,
    mut s: *const ::core::ffi::c_char,
    mut redraw: *mut ::core::ffi::c_int,
) -> prompt_key_result {
    if prompt_fire_callback(pr, s, PROMPT_KEY_CLOSE, redraw) != 0 {
        return PROMPT_KEY_CLOSE;
    }
    return PROMPT_KEY_HANDLED;
}
unsafe extern "C" fn prompt_check_move(
    mut pr: *mut prompt,
    mut key: key_code,
) -> prompt_key_result {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
        return PROMPT_KEY_NOT_HANDLED;
    }
    match key {
        8589934619 | 8589934620 | 8589934617 | 8589934616 => {}
        8589934621 | 8589934622 => {
            if (*pr).flags & PROMPT_EDITARROWS != 0 {
                return PROMPT_KEY_NOT_HANDLED;
            }
        }
        _ => return PROMPT_KEY_NOT_HANDLED,
    }
    s = utf8_tocstr((*pr).buffer);
    if prompt_fire_callback(
        pr,
        s,
        PROMPT_KEY_MOVE,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
    ) != 0
    {
        free(s as *mut ::core::ffi::c_void);
        return PROMPT_KEY_CLOSE;
    }
    free(s as *mut ::core::ffi::c_void);
    return PROMPT_KEY_MOVE;
}
#[no_mangle]
pub unsafe extern "C" fn prompt_key(
    mut pr: *mut prompt,
    mut key: key_code,
    mut redraw: *mut ::core::ffi::c_int,
) -> prompt_key_result {
    let mut current_block: u64;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut prefix: ::core::ffi::c_char = '=' as i32 as ::core::ffi::c_char;
    let mut histstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ks: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut size: size_t = 0;
    let mut idx: size_t = 0;
    let mut tmp: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut result: prompt_key_result = PROMPT_KEY_HANDLED;
    let mut word_is_separators: ::core::ffi::c_int = 0;
    (*pr).closed = 0 as ::core::ffi::c_int;
    prompt_clear_complete(pr);
    if (*pr).flags & PROMPT_KEY != 0 {
        ks = key_string_lookup_key(key, 0 as ::core::ffi::c_int);
        if prompt_fire_callback(
            pr,
            ks,
            PROMPT_KEY_CLOSE,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ) == 0
        {
            (*pr).closed = 1 as ::core::ffi::c_int;
        }
        return PROMPT_KEY_CLOSE;
    }
    size = utf8_strlen((*pr).buffer);
    key &= !KEYC_MASK_FLAGS;
    key = prompt_keypad_key(key);
    if (*pr).flags & PROMPT_NUMERIC != 0 {
        if key >= '0' as i32 as key_code && key <= '9' as i32 as key_code {
            current_block = 1115217863795707468;
        } else {
            s = utf8_tocstr((*pr).buffer);
            if prompt_fire_callback(
                pr,
                s,
                PROMPT_KEY_CLOSE,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ) == 0
            {
                (*pr).closed = 1 as ::core::ffi::c_int;
            }
            free(s as *mut ::core::ffi::c_void);
            return PROMPT_KEY_NOT_HANDLED;
        }
    } else if (*pr).flags & (PROMPT_SINGLE | PROMPT_QUOTENEXT) != 0 {
        if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_BSPACE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        {
            key = 0x7f as key_code;
        } else if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY > 0x7f as ::core::ffi::c_ulonglong
        {
            if !(key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                == (KEYC_TYPE_UNICODE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    > 0x7f as ::core::ffi::c_ulonglong)
            {
                return PROMPT_KEY_HANDLED;
            }
            key &= KEYC_MASK_KEY;
        } else {
            key &= if key as ::core::ffi::c_ulonglong & KEYC_CTRL != 0 {
                0x1f as ::core::ffi::c_ulonglong
            } else {
                KEYC_MASK_KEY
            };
        }
        (*pr).flags &= !PROMPT_QUOTENEXT;
        current_block = 1115217863795707468;
    } else {
        if (*pr).keys == MODEKEY_VI {
            match prompt_translate_key(pr, key, &raw mut key, redraw) {
                1 => {
                    current_block = 11090587058695514569;
                }
                2 => {
                    current_block = 1115217863795707468;
                }
                _ => return PROMPT_KEY_HANDLED,
            }
        } else {
            current_block = 11090587058695514569;
        }
        match current_block {
            1115217863795707468 => {}
            _ => {
                result = prompt_check_move(pr, key);
                if result as ::core::ffi::c_uint
                    != PROMPT_KEY_NOT_HANDLED as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    return result;
                }
                result = PROMPT_KEY_HANDLED;
                match key {
                    8589934621 | 35184372088930 => {
                        current_block = 14309557416411021540;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    8589934622 | 35184372088934 => {
                        current_block = 9249683890344250718;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    8589934614 | 35184372088929 => {
                        current_block = 12295586438617123170;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    8589934615 | 35184372088933 => {
                        current_block = 14414701084776968413;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    9 => {
                        current_block = 460814018713664829;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    8589934599 | 35184372088936 => {
                        current_block = 2263409105760785053;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    8589934613 | 35184372088932 => {
                        current_block = 5070726005990265983;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35184372088949 => {
                        current_block = 1491684083417518595;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35184372088939 => {
                        current_block = 8994603623389184299;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35184372088951 => {
                        current_block = 12879184554692362543;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35192962023454 | 17592186044518 => {
                        current_block = 15759124641699640388;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    9007199254741061 => {
                        current_block = 15097225335540574411;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    9007199254741093 => {
                        current_block = 4818991882628172305;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    9007199254741079 => {
                        current_block = 5183579720934817709;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    9007199254741111 => {
                        current_block = 17166280686405466987;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    9007199254741058 => {
                        current_block = 227872999036956190;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35192962023453 | 17592186044514 => {
                        current_block = 8252555365493261901;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    8589934619 | 35184372088944 => {
                        current_block = 410082779658746517;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    8589934620 | 35184372088942 => {
                        current_block = 10877725664641927171;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35184372088953 => {
                        current_block = 12682153168616704965;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35184372088948 => {
                        current_block = 10468295484844996031;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    13 | 10 => {
                        current_block = 1087518874050103606;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    27 | 35184372088923 | 35184372088931 | 35184372088935 => {
                        current_block = 2284014684288272695;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35184372088946 => {
                        current_block = 11643096306346113746;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35184372088947 => {
                        current_block = 4238185747604537484;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    35184372088950 => {
                        current_block = 13376399739742518040;
                        match current_block {
                            13376399739742518040 => {
                                (*pr).flags |= PROMPT_QUOTENEXT;
                                current_block = 4485073238441121731;
                            }
                            10468295484844996031 => {
                                idx = (*pr).index;
                                if idx < size {
                                    idx = idx.wrapping_add(1);
                                }
                                if idx >= 2 as size_t {
                                    utf8_copy(
                                        &raw mut tmp,
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(2 as size_t) as isize)
                                            as *mut utf8_data,
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                    );
                                    utf8_copy(
                                        (*pr).buffer.offset(idx.wrapping_sub(1 as size_t) as isize)
                                            as *mut utf8_data,
                                        &raw mut tmp,
                                    );
                                    (*pr).index = idx;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            10877725664641927171 => {
                                histstr = prompt_down_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            410082779658746517 => {
                                histstr = prompt_up_history(
                                    &raw mut (*pr).hindex as *mut u_int,
                                    (*pr).type_0 as u_int,
                                );
                                if histstr.is_null() {
                                    current_block = 4485073238441121731;
                                } else {
                                    free((*pr).buffer as *mut ::core::ffi::c_void);
                                    (*pr).buffer = utf8_fromcstr(histstr);
                                    (*pr).index = utf8_strlen((*pr).buffer);
                                    current_block = 5848346009959455809;
                                }
                            }
                            12879184554692362543 => {
                                idx = (*pr).index;
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) == 0
                                    {
                                        break;
                                    }
                                }
                                word_is_separators = prompt_in_list(
                                    (*pr).word_separators,
                                    (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                );
                                while idx != 0 as size_t {
                                    idx = idx.wrapping_sub(1);
                                    if !(prompt_space(
                                        (*pr).buffer.offset(idx as isize) as *mut utf8_data
                                    ) != 0
                                        || word_is_separators
                                            != prompt_in_list(
                                                (*pr).word_separators,
                                                (*pr).buffer.offset(idx as isize) as *mut utf8_data,
                                            ))
                                    {
                                        continue;
                                    }
                                    idx = idx.wrapping_add(1);
                                    break;
                                }
                                free((*pr).copied as *mut ::core::ffi::c_void);
                                (*pr).copied = xcalloc(
                                    ::core::mem::size_of::<utf8_data>() as size_t,
                                    (*pr).index.wrapping_sub(idx).wrapping_add(1 as size_t),
                                ) as *mut utf8_data;
                                memcpy(
                                    (*pr).copied as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset(idx as isize)
                                        as *const ::core::ffi::c_void,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memmove(
                                    (*pr).buffer.offset(idx as isize)
                                        as *mut ::core::ffi::c_void,
                                    (*pr).buffer.offset((*pr).index as isize)
                                        as *const ::core::ffi::c_void,
                                    size
                                        .wrapping_add(1 as size_t)
                                        .wrapping_sub((*pr).index)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                memset(
                                    (*pr)
                                        .buffer
                                        .offset(size as isize)
                                        .offset(-((*pr).index.wrapping_sub(idx) as isize))
                                        as *mut ::core::ffi::c_void,
                                    '\0' as i32,
                                    (*pr)
                                        .index
                                        .wrapping_sub(idx)
                                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                                );
                                (*pr).index = idx;
                                current_block = 5848346009959455809;
                            }
                            8994603623389184299 => {
                                if (*pr).index < size {
                                    (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            1491684083417518595 => {
                                (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size =
                                    0 as u_char;
                                (*pr).index = 0 as size_t;
                                current_block = 5848346009959455809;
                            }
                            5070726005990265983 => {
                                if (*pr).index != size {
                                    memmove(
                                        (*pr).buffer.offset((*pr).index as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*pr)
                                            .buffer
                                            .offset((*pr).index as isize)
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        size.wrapping_sub((*pr).index).wrapping_mul(
                                            ::core::mem::size_of::<utf8_data>() as size_t,
                                        ),
                                    );
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2263409105760785053 => {
                                if (*pr).flags & PROMPT_BSPACE_EXIT != 0 && size == 0 as size_t {
                                    return prompt_done(
                                        pr,
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                        redraw,
                                    );
                                }
                                if (*pr).index != 0 as size_t {
                                    if (*pr).index == size {
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                        (*(*pr).buffer.offset((*pr).index as isize)).size =
                                            0 as u_char;
                                    } else {
                                        memmove(
                                            (*pr)
                                                .buffer
                                                .offset((*pr).index as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize))
                                                as *mut ::core::ffi::c_void,
                                            (*pr).buffer.offset((*pr).index as isize)
                                                as *const ::core::ffi::c_void,
                                            size.wrapping_add(1 as size_t)
                                                .wrapping_sub((*pr).index)
                                                .wrapping_mul(
                                                    ::core::mem::size_of::<utf8_data>() as size_t
                                                ),
                                        );
                                        (*pr).index = (*pr).index.wrapping_sub(1);
                                    }
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14414701084776968413 => {
                                if (*pr).index != size {
                                    (*pr).index = size;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            12295586438617123170 => {
                                if (*pr).index != 0 as size_t {
                                    (*pr).index = 0 as size_t;
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            9249683890344250718 => {
                                if (*pr).index < size {
                                    (*pr).index = (*pr).index.wrapping_add(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            14309557416411021540 => {
                                if (*pr).index > 0 as size_t {
                                    (*pr).index = (*pr).index.wrapping_sub(1);
                                    current_block = 4485073238441121731;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            11643096306346113746 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '-' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            4238185747604537484 => {
                                if !(*pr).flags & PROMPT_INCREMENTAL != 0 {
                                    current_block = 4485073238441121731;
                                } else {
                                    if (*(*pr).buffer.offset(0 as ::core::ffi::c_int as isize)).size
                                        as ::core::ffi::c_int
                                        == 0 as ::core::ffi::c_int
                                    {
                                        prefix = '=' as i32 as ::core::ffi::c_char;
                                        free((*pr).buffer as *mut ::core::ffi::c_void);
                                        (*pr).buffer = utf8_fromcstr((*pr).last);
                                        (*pr).index = utf8_strlen((*pr).buffer);
                                    } else {
                                        prefix = '+' as i32 as ::core::ffi::c_char;
                                    }
                                    current_block = 5848346009959455809;
                                }
                            }
                            460814018713664829 => {
                                if prompt_replace_complete(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                ) != 0
                                {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            15759124641699640388 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    0 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            15097225335540574411 => {
                                prompt_end_word(
                                    pr,
                                    size,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            4818991882628172305 => {
                                prompt_end_word(pr, size, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            5183579720934817709 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            17166280686405466987 => {
                                prompt_forward_word(
                                    pr,
                                    size,
                                    1 as ::core::ffi::c_int,
                                    (*pr).word_separators,
                                );
                                current_block = 5848346009959455809;
                            }
                            227872999036956190 => {
                                prompt_backward_word(
                                    pr,
                                    b"\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                current_block = 5848346009959455809;
                            }
                            8252555365493261901 => {
                                prompt_backward_word(pr, (*pr).word_separators);
                                current_block = 5848346009959455809;
                            }
                            12682153168616704965 => {
                                if prompt_paste(pr) != 0 {
                                    current_block = 5848346009959455809;
                                } else {
                                    current_block = 4485073238441121731;
                                }
                            }
                            2284014684288272695 => {
                                return prompt_done(
                                    pr,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    redraw,
                                );
                            }
                            _ => {
                                s = utf8_tocstr((*pr).buffer);
                                if *s as ::core::ffi::c_int != '\0' as i32 {
                                    prompt_add_history(s, (*pr).type_0 as u_int);
                                }
                                result = prompt_done(pr, s, redraw);
                                free(s as *mut ::core::ffi::c_void);
                                return result;
                            }
                        }
                        match current_block {
                            5848346009959455809 => {}
                            _ => {
                                *redraw = 1 as ::core::ffi::c_int;
                                return PROMPT_KEY_HANDLED;
                            }
                        }
                    }
                    _ => {
                        current_block = 1115217863795707468;
                    }
                }
            }
        }
    }
    match current_block {
        1115217863795707468 => {
            if key <= 0x7f as key_code {
                utf8_set(&raw mut tmp, key as u_char);
                if key <= 0x1f as key_code || key == 0x7f as key_code {
                    tmp.width = 2 as u_char;
                }
            } else if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                == (KEYC_TYPE_UNICODE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    > 0x7f as ::core::ffi::c_ulonglong
            {
                utf8_to_data(key as utf8_char, &raw mut tmp);
                if tmp.size as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                    return PROMPT_KEY_HANDLED;
                }
            } else {
                return PROMPT_KEY_HANDLED;
            }
            (*pr).buffer = xreallocarray(
                (*pr).buffer as *mut ::core::ffi::c_void,
                size.wrapping_add(2 as size_t),
                ::core::mem::size_of::<utf8_data>() as size_t,
            ) as *mut utf8_data;
            if (*pr).index == size {
                utf8_copy(
                    (*pr).buffer.offset((*pr).index as isize) as *mut utf8_data,
                    &raw mut tmp,
                );
                (*pr).index = (*pr).index.wrapping_add(1);
                (*(*pr).buffer.offset((*pr).index as isize)).size = 0 as u_char;
            } else {
                memmove(
                    (*pr)
                        .buffer
                        .offset((*pr).index as isize)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *mut ::core::ffi::c_void,
                    (*pr).buffer.offset((*pr).index as isize) as *const ::core::ffi::c_void,
                    size.wrapping_add(1 as size_t)
                        .wrapping_sub((*pr).index)
                        .wrapping_mul(::core::mem::size_of::<utf8_data>() as size_t),
                );
                utf8_copy(
                    (*pr).buffer.offset((*pr).index as isize) as *mut utf8_data,
                    &raw mut tmp,
                );
                (*pr).index = (*pr).index.wrapping_add(1);
            }
            if (*pr).flags & PROMPT_SINGLE != 0 {
                if utf8_strlen((*pr).buffer) != 1 as size_t {
                    (*pr).closed = 1 as ::core::ffi::c_int;
                    result = PROMPT_KEY_CLOSE;
                } else {
                    s = utf8_tocstr((*pr).buffer);
                    result = prompt_done(pr, s, redraw);
                    free(s as *mut ::core::ffi::c_void);
                }
            }
        }
        _ => {}
    }
    *redraw = 1 as ::core::ffi::c_int;
    if (*pr).flags & PROMPT_INCREMENTAL != 0 {
        s = utf8_tocstr((*pr).buffer);
        xasprintf(
            &raw mut cp,
            b"%c%s\0" as *const u8 as *const ::core::ffi::c_char,
            prefix as ::core::ffi::c_int,
            s,
        );
        prompt_fire_callback(
            pr,
            cp,
            PROMPT_KEY_HANDLED,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        free(cp as *mut ::core::ffi::c_void);
        free(s as *mut ::core::ffi::c_void);
    }
    return result;
}
unsafe extern "C" fn prompt_complete_add(
    mut list: *mut *mut *mut ::core::ffi::c_char,
    mut size: *mut u_int,
    mut s: *const ::core::ffi::c_char,
) {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < *size {
        if strcmp(*(*list).offset(i as isize), s) == 0 as ::core::ffi::c_int {
            return;
        }
        i = i.wrapping_add(1);
    }
    *list = xreallocarray(
        *list as *mut ::core::ffi::c_void,
        (*size).wrapping_add(1 as u_int) as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
    ) as *mut *mut ::core::ffi::c_char;
    let fresh0 = *size;
    *size = (*size).wrapping_add(1);
    let ref mut fresh1 = *(*list).offset(fresh0 as isize);
    *fresh1 = xstrdup(s);
}
unsafe extern "C" fn prompt_complete_commands(
    mut size: *mut u_int,
    mut s: *const ::core::ffi::c_char,
) -> *mut *mut ::core::ffi::c_char {
    let mut list: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cmdent: *mut *const cmd_entry = ::core::ptr::null_mut::<*const cmd_entry>();
    let mut slen: size_t = strlen(s);
    let mut valuelen: size_t = 0;
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    *size = 0 as u_int;
    cmdent = &raw mut cmd_table as *mut *const cmd_entry;
    while !(*cmdent).is_null() {
        if strncmp((**cmdent).name, s, slen) == 0 as ::core::ffi::c_int {
            prompt_complete_add(&raw mut list, size, (**cmdent).name);
        }
        cmdent = cmdent.offset(1);
    }
    o = options_get_only(
        global_options,
        b"command-alias\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if !o.is_null() {
        a = options_array_first(o);
        while !a.is_null() {
            value = (*options_array_item_value(a)).string;
            cp = strchr(value, '=' as i32);
            if !cp.is_null() {
                valuelen = cp.offset_from(value) as ::core::ffi::c_long as size_t;
                if !(slen > valuelen || strncmp(value, s, slen) != 0 as ::core::ffi::c_int) {
                    xasprintf(
                        &raw mut tmp,
                        b"%.*s\0" as *const u8 as *const ::core::ffi::c_char,
                        valuelen as ::core::ffi::c_int,
                        value,
                    );
                    prompt_complete_add(&raw mut list, size, tmp);
                    free(tmp as *mut ::core::ffi::c_void);
                }
            }
            a = options_array_next(a);
        }
    }
    return list;
}
unsafe extern "C" fn prompt_complete_prefix(
    mut list: *mut *mut ::core::ffi::c_char,
    mut size: u_int,
) -> *mut ::core::ffi::c_char {
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut j: size_t = 0;
    if list.is_null() || size == 0 as u_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    out = xstrdup(*list.offset(0 as ::core::ffi::c_int as isize));
    i = 1 as u_int;
    while i < size {
        j = 0 as size_t;
        while *out.offset(j as isize) as ::core::ffi::c_int != '\0' as i32
            && *(*list.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int != '\0' as i32
        {
            if *out.offset(j as isize) as ::core::ffi::c_int
                != *(*list.offset(i as isize)).offset(j as isize) as ::core::ffi::c_int
            {
                break;
            }
            j = j.wrapping_add(1);
        }
        *out.offset(j as isize) = '\0' as i32 as ::core::ffi::c_char;
        i = i.wrapping_add(1);
    }
    return out;
}
unsafe extern "C" fn prompt_complete_sort(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut aa: *mut *const ::core::ffi::c_char = a as *mut *const ::core::ffi::c_char;
    let mut bb: *mut *const ::core::ffi::c_char = b as *mut *const ::core::ffi::c_char;
    return strcmp(*aa, *bb);
}
unsafe extern "C" fn prompt_clear_complete(mut pr: *mut prompt) {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < (*pr).complete_size {
        free(*(*pr).complete_list.offset(i as isize) as *mut ::core::ffi::c_void);
        i = i.wrapping_add(1);
    }
    free((*pr).complete_list as *mut ::core::ffi::c_void);
    (*pr).complete_list = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    (*pr).complete_size = 0 as u_int;
    free((*pr).complete_display as *mut ::core::ffi::c_void);
    (*pr).complete_display = ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn prompt_store_complete(
    mut pr: *mut prompt,
    mut list: *mut *mut ::core::ffi::c_char,
    mut size: u_int,
) {
    let mut display: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    prompt_clear_complete(pr);
    (*pr).complete_list = list;
    (*pr).complete_size = size;
    display = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    i = 0 as u_int;
    while i < size {
        xasprintf(
            &raw mut cp,
            b"%s %s\0" as *const u8 as *const ::core::ffi::c_char,
            display,
            *list.offset(i as isize),
        );
        free(display as *mut ::core::ffi::c_void);
        display = cp;
        i = i.wrapping_add(1);
    }
    (*pr).complete_display = display;
}
unsafe extern "C" fn prompt_complete(
    mut pr: *mut prompt,
    mut word: *const ::core::ffi::c_char,
    mut offset: u_int,
) -> *mut ::core::ffi::c_char {
    let mut list: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut size: u_int = 0 as u_int;
    let mut i: u_int = 0;
    if (*pr).type_0 as ::core::ffi::c_uint
        != PROMPT_TYPE_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint
        || offset != 0 as u_int
        || *word as ::core::ffi::c_int == '\0' as i32
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    list = prompt_complete_commands(&raw mut size, word);
    if size == 0 as u_int {
        free(list as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    qsort(
        list as *mut ::core::ffi::c_void,
        size as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
        Some(
            prompt_complete_sort
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    i = 0 as u_int;
    while i < size {
        log_debug(
            b"complete %u: %s\0" as *const u8 as *const ::core::ffi::c_char,
            i,
            *list.offset(i as isize),
        );
        i = i.wrapping_add(1);
    }
    if size == 1 as u_int {
        xasprintf(
            &raw mut out,
            b"%s \0" as *const u8 as *const ::core::ffi::c_char,
            *list.offset(0 as ::core::ffi::c_int as isize),
        );
    } else {
        out = prompt_complete_prefix(list, size);
    }
    if !out.is_null() && strcmp(word, out) == 0 as ::core::ffi::c_int {
        free(out as *mut ::core::ffi::c_void);
        out = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if !out.is_null() || size <= 1 as u_int {
        i = 0 as u_int;
        while i < size {
            free(*list.offset(i as isize) as *mut ::core::ffi::c_void);
            i = i.wrapping_add(1);
        }
        free(list as *mut ::core::ffi::c_void);
        return out;
    }
    prompt_store_complete(pr, list, size);
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn prompt_type(mut type_0: *const ::core::ffi::c_char) -> prompt_type {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < PROMPT_NTYPES as u_int {
        if strcmp(type_0, prompt_type_string(i as prompt_type)) == 0 as ::core::ffi::c_int {
            return i as prompt_type;
        }
        i = i.wrapping_add(1);
    }
    return PROMPT_TYPE_INVALID;
}
#[no_mangle]
pub unsafe extern "C" fn prompt_type_string(mut type_0: prompt_type) -> *const ::core::ffi::c_char {
    match type_0 as ::core::ffi::c_uint {
        0 => return b"command\0" as *const u8 as *const ::core::ffi::c_char,
        1 => return b"search\0" as *const u8 as *const ::core::ffi::c_char,
        255 => return b"invalid\0" as *const u8 as *const ::core::ffi::c_char,
        _ => {}
    }
    return b"unknown\0" as *const u8 as *const ::core::ffi::c_char;
}

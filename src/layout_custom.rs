pub use crate::src::shared::limits::{__INT_MAX__, INT_MAX};
pub use crate::src::shared::window::{WINDOW_MAXIMUM};
pub use crate::src::shared::pane::{
    PANE_MAXIMUM, PANE_MINIMUM, window_pane_offset, window_pane_resize,
    window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::abi::{__compar_fn_t, __int64_t, int64_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
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
    pub type json_node;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
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
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn sscanf(
        __s: *const ::core::ffi::c_char,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
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
    fn events_fire_window(_: *const ::core::ffi::c_char, _: *mut window);
    fn recalculate_sizes();
    fn window_set_active_pane(
        _: *mut window,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn window_resize(
        _: *mut window,
        _: u_int,
        _: u_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn window_pane_index(_: *mut window_pane, _: *mut u_int) -> ::core::ffi::c_int;
    fn window_pane_zindex(_: *mut window_pane, _: *mut u_int) -> ::core::ffi::c_int;
    fn window_pane_last_index(_: *mut window_pane, _: *mut u_int) -> ::core::ffi::c_int;
    fn window_count_panes(_: *mut window, _: ::core::ffi::c_int) -> u_int;
    fn window_pane_stack_push(_: *mut window_panes, _: *mut window_pane);
    fn window_pane_stack_remove(_: *mut window_panes, _: *mut window_pane);
    fn window_pane_is_floating(_: *mut window_pane) -> ::core::ffi::c_int;
    fn layout_count_cells(_: *mut layout_cell, _: ::core::ffi::c_int) -> u_int;
    fn layout_create_cell(_: *mut layout_cell) -> *mut layout_cell;
    fn layout_free_cell(_: *mut layout_cell, _: ::core::ffi::c_int);
    fn layout_print_cell(_: *mut layout_cell, _: *const ::core::ffi::c_char, _: u_int);
    fn layout_destroy_cell(_: *mut window, _: *mut layout_cell, _: *mut *mut layout_cell);
    fn layout_set_size(
        _: *mut layout_cell,
        _: u_int,
        _: u_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn layout_make_leaf(_: *mut layout_cell, _: *mut window_pane);
    fn layout_cell_is_tiled(_: *mut layout_cell) -> ::core::ffi::c_int;
    fn layout_cell_has_tiled_child(_: *mut layout_cell) -> ::core::ffi::c_int;
    fn layout_fix_offsets(_: *mut window);
    fn layout_fix_panes(_: *mut window, _: *mut window_pane);
    fn layout_replace_with_node(
        _: *mut window,
        _: *mut layout_cell,
        _: layout_type,
    ) -> *mut layout_cell;
    fn json_parse(
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut json_node;
    fn json_destroy_node(_: *mut json_node);
    fn json_find(_: *mut json_node, _: *const ::core::ffi::c_char) -> *mut json_node;
    fn json_array_first(_: *mut json_node) -> *mut json_node;
    fn json_array_next(_: *mut json_node) -> *mut json_node;
    fn json_get_object(_: *mut json_node, _: *mut *mut json_node) -> ::core::ffi::c_int;
    fn json_find_string(
        _: *mut json_node,
        _: *const ::core::ffi::c_char,
        _: *mut *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn json_find_number(
        _: *mut json_node,
        _: *const ::core::ffi::c_char,
        _: *mut int64_t,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn json_find_boolean(
        _: *mut json_node,
        _: *const ::core::ffi::c_char,
        _: *mut ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn json_find_object(
        _: *mut json_node,
        _: *const ::core::ffi::c_char,
        _: *mut *mut json_node,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn json_find_array(
        _: *mut json_node,
        _: *const ::core::ffi::c_char,
        _: *mut *mut json_node,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
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
    pub entry: C2RustUnnamed_19,
    pub wentry: C2RustUnnamed_18,
    pub sentry: C2RustUnnamed_17,
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
    pub tqe_next: *mut winlink,
    pub tqe_prev: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_19 {
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
    pub alerts_entry: C2RustUnnamed_22,
    pub options: *mut options,
    pub references: u_int,
    pub winlinks: C2RustUnnamed_21,
    pub entry: C2RustUnnamed_20,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_20 {
    pub rbe_left: *mut window,
    pub rbe_right: *mut window,
    pub rbe_parent: *mut window,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_21 {
    pub tqh_first: *mut winlink,
    pub tqh_last: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_22 {
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
    pub entry: C2RustUnnamed_23,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_23 {
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
    pub modes: C2RustUnnamed_28,
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
    pub entry: C2RustUnnamed_27,
    pub sentry: C2RustUnnamed_26,
    pub zentry: C2RustUnnamed_25,
    pub tree_entry: C2RustUnnamed_24,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_24 {
    pub rbe_left: *mut window_pane,
    pub rbe_right: *mut window_pane,
    pub rbe_parent: *mut window_pane,
    pub rbe_color: ::core::ffi::c_int,
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
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_28 {
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
    pub entry: C2RustUnnamed_29,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_29 {
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
    pub entry: C2RustUnnamed_32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_32 {
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
    pub entry: C2RustUnnamed_33,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_33 {
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
pub struct layout_string {
    pub dat: *mut ::core::ffi::c_char,
    pub size: size_t,
    pub capacity: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct layout_parse_ctx {
    pub version: int64_t,
    pub num_active: ::core::ffi::c_int,
    pub root: *mut layout_cell,
    pub cause: *mut *mut ::core::ffi::c_char,
    pub size: ::core::ffi::c_int,
    pub capacity: ::core::ffi::c_int,
    pub cctxs: *mut layout_parse_cell_ctx,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct layout_parse_cell_ctx {
    pub lc: *mut layout_cell,
    pub active: ::core::ffi::c_int,
    pub last: ::core::ffi::c_int,
    pub index: ::core::ffi::c_int,
    pub zindex: ::core::ffi::c_int,
}

unsafe extern "C" fn layout_parse_index_cmp(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut cca: *const layout_parse_cell_ctx = a as *const layout_parse_cell_ctx;
    let mut ccb: *const layout_parse_cell_ctx = b as *const layout_parse_cell_ctx;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cca).index < (*ccb).index {
        retval = -(1 as ::core::ffi::c_int);
    }
    if (*cca).index > (*ccb).index {
        retval = 1 as ::core::ffi::c_int;
    }
    return retval;
}
unsafe extern "C" fn layout_parse_zindex_cmp(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut cca: *const layout_parse_cell_ctx = a as *const layout_parse_cell_ctx;
    let mut ccb: *const layout_parse_cell_ctx = b as *const layout_parse_cell_ctx;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cca).zindex > (*ccb).zindex {
        retval = -(1 as ::core::ffi::c_int);
    }
    if (*cca).zindex < (*ccb).zindex {
        retval = 1 as ::core::ffi::c_int;
    }
    return retval;
}
unsafe extern "C" fn layout_parse_last_cmp(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut cca: *const layout_parse_cell_ctx = a as *const layout_parse_cell_ctx;
    let mut ccb: *const layout_parse_cell_ctx = b as *const layout_parse_cell_ctx;
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*cca).last > (*ccb).last {
        retval = -(1 as ::core::ffi::c_int);
    }
    if (*cca).last < (*ccb).last {
        retval = 1 as ::core::ffi::c_int;
    }
    return retval;
}
unsafe extern "C" fn layout_string_init(mut ls: *mut layout_string) {
    (*ls).capacity = 1024 as size_t;
    (*ls).dat = xmalloc((*ls).capacity) as *mut ::core::ffi::c_char;
    *(*ls).dat.offset(0 as ::core::ffi::c_int as isize) = '\0' as i32 as ::core::ffi::c_char;
    (*ls).size = 0 as size_t;
}
unsafe extern "C" fn layout_string_free(mut ls: *mut layout_string) {
    free((*ls).dat as *mut ::core::ffi::c_void);
    (*ls).dat = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*ls).size = 0 as size_t;
    (*ls).capacity = 0 as size_t;
}
unsafe extern "C" fn layout_string_write(
    mut ls: *mut layout_string,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut slen: ::core::ffi::c_int = 0;
    ap = args.clone();
    slen = xvasprintf(&raw mut s, fmt, ap);
    while (*ls)
        .size
        .wrapping_add(slen as size_t)
        .wrapping_add(1 as size_t)
        > (*ls).capacity
    {
        (*ls).dat = xreallocarray(
            (*ls).dat as *mut ::core::ffi::c_void,
            2 as size_t,
            (*ls).capacity,
        ) as *mut ::core::ffi::c_char;
        (*ls).capacity = (*ls).capacity.wrapping_mul(2 as size_t);
    }
    memcpy(
        (*ls).dat.offset((*ls).size as isize) as *mut ::core::ffi::c_void,
        s as *const ::core::ffi::c_void,
        slen as size_t,
    );
    (*ls).size = (*ls).size.wrapping_add(slen as size_t);
    *(*ls).dat.offset((*ls).size as isize) = '\0' as i32 as ::core::ffi::c_char;
    free(s as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn layout_parse_init_ctx(
    mut pctx: *mut layout_parse_ctx,
    mut cause: *mut *mut ::core::ffi::c_char,
) {
    (*pctx).version = -(1 as ::core::ffi::c_int) as int64_t;
    (*pctx).num_active = 0 as ::core::ffi::c_int;
    (*pctx).root = ::core::ptr::null_mut::<layout_cell>();
    (*pctx).cause = cause;
    (*pctx).size = 0 as ::core::ffi::c_int;
    (*pctx).capacity = 64 as ::core::ffi::c_int;
    (*pctx).cctxs = xcalloc(
        (*pctx).capacity as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
    ) as *mut layout_parse_cell_ctx;
}
unsafe extern "C" fn layout_parse_free_ctx(mut pctx: *mut layout_parse_ctx) {
    layout_free_cell((*pctx).root, 0 as ::core::ffi::c_int);
    (*pctx).root = ::core::ptr::null_mut::<layout_cell>();
    free((*pctx).cctxs as *mut ::core::ffi::c_void);
    (*pctx).cctxs = ::core::ptr::null_mut::<layout_parse_cell_ctx>();
    (*pctx).size = 0 as ::core::ffi::c_int;
    (*pctx).capacity = 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn layout_parse_add_cctx(
    mut pctx: *mut layout_parse_ctx,
    mut lc: *mut layout_cell,
    mut active: ::core::ffi::c_int,
    mut last: ::core::ffi::c_int,
    mut index: ::core::ffi::c_int,
    mut zindex: ::core::ffi::c_int,
) {
    let mut cctx: *mut layout_parse_cell_ctx = ::core::ptr::null_mut::<layout_parse_cell_ctx>();
    if (*pctx).size >= (*pctx).capacity {
        (*pctx).capacity *= 2 as ::core::ffi::c_int;
        (*pctx).cctxs = xreallocarray(
            (*pctx).cctxs as *mut ::core::ffi::c_void,
            (*pctx).capacity as size_t,
            ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        ) as *mut layout_parse_cell_ctx;
    }
    let fresh0 = (*pctx).size;
    (*pctx).size = (*pctx).size + 1;
    cctx = (*pctx).cctxs.offset(fresh0 as isize) as *mut layout_parse_cell_ctx;
    (*cctx).lc = lc;
    (*cctx).active = active;
    (*cctx).last = last;
    (*cctx).index = index;
    (*cctx).zindex = zindex;
}
unsafe extern "C" fn layout_parse_remove_cctx(
    mut pctx: *mut layout_parse_ctx,
    mut lc: *mut layout_cell,
) -> ::core::ffi::c_int {
    let mut cctx: *mut layout_parse_cell_ctx = ::core::ptr::null_mut::<layout_parse_cell_ctx>();
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < (*pctx).size {
        if lc == (*(*pctx).cctxs.offset(i as isize)).lc {
            (*pctx).size -= 1;
            cctx = (*pctx).cctxs.offset((*pctx).size as isize) as *mut layout_parse_cell_ctx;
            memmove(
                (*pctx).cctxs.offset(i as isize) as *mut layout_parse_cell_ctx
                    as *mut ::core::ffi::c_void,
                cctx as *const ::core::ffi::c_void,
                ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
            );
            return 0 as ::core::ffi::c_int;
        }
        i += 1;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn layout_find_bottomright(mut lc: *mut layout_cell) -> *mut layout_cell {
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return lc;
    }
    lc = *(*((*lc).cells.tqh_last as *mut layout_cells)).tqh_last;
    return layout_find_bottomright(lc);
}
unsafe extern "C" fn layout_checksum(mut layout: *const ::core::ffi::c_char) -> u_short {
    let mut csum: u_short = 0;
    csum = 0 as u_short;
    while *layout as ::core::ffi::c_int != '\0' as i32 {
        csum = ((csum as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
            + ((csum as ::core::ffi::c_int & 1 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int))
            as u_short;
        csum = (csum as ::core::ffi::c_int + *layout as ::core::ffi::c_int) as u_short;
        layout = layout.offset(1);
    }
    return csum;
}
#[no_mangle]
pub unsafe extern "C" fn layout_dump(
    mut w: *mut window,
    mut lcroot: *mut layout_cell,
    mut flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut layout_string: layout_string = layout_string {
        dat: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
        capacity: 0,
    };
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lcroot.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    layout_string_init(&raw mut layout_string);
    if layout_append(lcroot, &raw mut layout_string, flags) == 0 as ::core::ffi::c_int {
        if flags & LAYOUT_CUSTOM_OLD_FORMAT != 0 {
            xasprintf(
                &raw mut out,
                b"%04hx,%s\0" as *const u8 as *const ::core::ffi::c_char,
                layout_checksum(layout_string.dat) as ::core::ffi::c_int,
                layout_string.dat,
            );
        } else {
            xasprintf(
                &raw mut out,
                b"{\"V\":2,\"L\":%s}\0" as *const u8 as *const ::core::ffi::c_char,
                layout_string.dat,
            );
        }
    }
    layout_string_free(&raw mut layout_string);
    return out;
}
unsafe extern "C" fn layout_append_v2(
    mut lc: *mut layout_cell,
    mut ls: *mut layout_string,
) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut type_0: layout_type = LAYOUT_LEFTRIGHT;
    let mut c: ::core::ffi::c_char = 0;
    let mut i: u_int = 0;
    let mut n: u_int = 0;
    if lc.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    type_0 = (*lc).type_0;
    if type_0 as ::core::ffi::c_uint
        == LAYOUT_TOPBOTTOM as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        c = 'v' as i32 as ::core::ffi::c_char;
    } else if type_0 as ::core::ffi::c_uint
        == LAYOUT_LEFTRIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        c = 'h' as i32 as ::core::ffi::c_char;
    } else if type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        c = 'p' as i32 as ::core::ffi::c_char;
    } else {
        return -(1 as ::core::ffi::c_int);
    }
    layout_string_write(
        ls,
        b"{\"t\":\"%c\",\"w\":%u,\"h\":%u,\"x\":%d,\"y\":%d\0" as *const u8
            as *const ::core::ffi::c_char,
        c as ::core::ffi::c_int,
        (*lc).g.sx,
        (*lc).g.sy,
        (*lc).g.xoff,
        (*lc).g.yoff,
    );
    if type_0 as ::core::ffi::c_uint
        != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        layout_string_write(ls, b",\"c\":[\0" as *const u8 as *const ::core::ffi::c_char);
        n = 0 as u_int;
        lcchild = (*lc).cells.tqh_first;
        while !lcchild.is_null() {
            if layout_append_v2(lcchild, ls) != 0 as ::core::ffi::c_int {
                return -(1 as ::core::ffi::c_int);
            }
            layout_string_write(ls, b",\0" as *const u8 as *const ::core::ffi::c_char);
            n = n.wrapping_add(1);
            lcchild = (*lcchild).entry.tqe_next;
        }
        if n == 0 as u_int {
            return -(1 as ::core::ffi::c_int);
        }
        (*ls).size = (*ls).size.wrapping_sub(1);
        *(*ls).dat.offset((*ls).size as isize) = '\0' as i32 as ::core::ffi::c_char;
        layout_string_write(ls, b"]\0" as *const u8 as *const ::core::ffi::c_char);
    } else {
        wp = (*lc).wp;
        if wp.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        if wp == (*(*wp).window).active {
            layout_string_write(
                ls,
                b",\"a\":true\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else if window_pane_last_index(wp, &raw mut i) == 0 as ::core::ffi::c_int {
            layout_string_write(
                ls,
                b",\"l\":%u\0" as *const u8 as *const ::core::ffi::c_char,
                i,
            );
        }
        if window_pane_index(wp, &raw mut i) != 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
        layout_string_write(
            ls,
            b",\"i\":%u\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        if (*lc).flags & LAYOUT_CELL_FLOATING != 0
            && window_pane_zindex(wp, &raw mut i) == 0 as ::core::ffi::c_int
        {
            layout_string_write(
                ls,
                b",\"z\":%u\0" as *const u8 as *const ::core::ffi::c_char,
                i,
            );
        }
        layout_string_write(
            ls,
            b",\"I\":\"%%%u\"\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
        );
    }
    layout_string_write(ls, b"}\0" as *const u8 as *const ::core::ffi::c_char);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn layout_append_v1(
    mut lc: *mut layout_cell,
    mut ls: *mut layout_string,
) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut brackets: *const ::core::ffi::c_char =
        b"[]\0" as *const u8 as *const ::core::ffi::c_char;
    if lc.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if !(*lc).wp.is_null() {
        layout_string_write(
            ls,
            b"%ux%u,%d,%d,%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*lc).g.sx,
            (*lc).g.sy,
            (*lc).g.xoff,
            (*lc).g.yoff,
            (*(*lc).wp).id,
        );
    } else {
        layout_string_write(
            ls,
            b"%ux%u,%d,%d\0" as *const u8 as *const ::core::ffi::c_char,
            (*lc).g.sx,
            (*lc).g.sy,
            (*lc).g.xoff,
            (*lc).g.yoff,
        );
    }
    let mut current_block_16: u64;
    match (*lc).type_0 as ::core::ffi::c_uint {
        0 => {
            brackets = b"{}\0" as *const u8 as *const ::core::ffi::c_char;
            current_block_16 = 14129903220312603109;
        }
        1 => {
            current_block_16 = 14129903220312603109;
        }
        2 | _ => {
            current_block_16 = 1054647088692577877;
        }
    }
    match current_block_16 {
        14129903220312603109 => {
            layout_string_write(
                ls,
                b"%c\0" as *const u8 as *const ::core::ffi::c_char,
                *brackets.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            );
            lcchild = (*lc).cells.tqh_first;
            while !lcchild.is_null() {
                if layout_append_v1(lcchild, ls) != 0 as ::core::ffi::c_int {
                    return -(1 as ::core::ffi::c_int);
                }
                layout_string_write(ls, b",\0" as *const u8 as *const ::core::ffi::c_char);
                lcchild = (*lcchild).entry.tqe_next;
            }
            (*ls).size = (*ls).size.wrapping_sub(1);
            *(*ls).dat.offset((*ls).size as isize) = '\0' as i32 as ::core::ffi::c_char;
            layout_string_write(
                ls,
                b"%c\0" as *const u8 as *const ::core::ffi::c_char,
                *brackets.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            );
        }
        _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn layout_custom_copy_layout(mut lc: *mut layout_cell) -> *mut layout_cell {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcnewchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lconly: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcnew: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*lc).flags & LAYOUT_CELL_FLOATING != 0
    {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    lcnew = layout_create_cell(::core::ptr::null_mut::<layout_cell>());
    (*lcnew).type_0 = (*lc).type_0;
    (*lcnew).flags = (*lc).flags;
    if (*lc).type_0 as ::core::ffi::c_uint
        == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*lcnew).wp = (*lc).wp;
    }
    layout_set_size(lcnew, (*lc).g.sx, (*lc).g.sy, (*lc).g.xoff, (*lc).g.yoff);
    match (*lc).type_0 as ::core::ffi::c_uint {
        1 | 0 => {
            lcchild = (*lc).cells.tqh_first;
            while !lcchild.is_null() {
                lcnewchild = layout_custom_copy_layout(lcchild);
                if !lcnewchild.is_null() {
                    (*lcnewchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
                    (*lcnewchild).entry.tqe_prev = (*lcnew).cells.tqh_last;
                    *(*lcnew).cells.tqh_last = lcnewchild;
                    (*lcnew).cells.tqh_last = &raw mut (*lcnewchild).entry.tqe_next;
                    (*lcnewchild).parent = lcnew;
                }
                lcchild = (*lcchild).entry.tqe_next;
            }
            lconly = (*lcnew).cells.tqh_first;
            if lconly.is_null() {
                layout_free_cell(lcnew, 0 as ::core::ffi::c_int);
                return ::core::ptr::null_mut::<layout_cell>();
            }
            if (*lconly).entry.tqe_next.is_null() {
                if !(*lconly).entry.tqe_next.is_null() {
                    (*(*lconly).entry.tqe_next).entry.tqe_prev = (*lconly).entry.tqe_prev;
                } else {
                    (*lcnew).cells.tqh_last = (*lconly).entry.tqe_prev;
                }
                *(*lconly).entry.tqe_prev = (*lconly).entry.tqe_next;
                (*lconly).parent = ::core::ptr::null_mut::<layout_cell>();
                layout_free_cell(lcnew, 0 as ::core::ffi::c_int);
                return lconly;
            }
        }
        2 | _ => {}
    }
    return lcnew;
}
unsafe extern "C" fn layout_custom_create_compat(mut lcroot: *mut layout_cell) -> *mut layout_cell {
    let mut lccompat: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    lccompat = layout_custom_copy_layout(lcroot);
    if !lccompat.is_null() && layout_cell_is_tiled(lccompat) != 0 {
        (*lccompat).g.xoff = 0 as ::core::ffi::c_int;
        (*lccompat).g.yoff = 0 as ::core::ffi::c_int;
    }
    return lccompat;
}
unsafe extern "C" fn layout_custom_unlink_panes(mut lc: *mut layout_cell) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    match (*lc).type_0 as ::core::ffi::c_uint {
        2 => {
            (*lc).wp = ::core::ptr::null_mut::<window_pane>();
        }
        0 | 1 => {
            lcchild = (*lc).cells.tqh_first;
            while !lcchild.is_null() {
                layout_custom_unlink_panes(lcchild);
                lcchild = (*lcchild).entry.tqe_next;
            }
        }
        _ => {}
    };
}
unsafe extern "C" fn layout_custom_free_compat(mut lcroot: *mut layout_cell) {
    if lcroot.is_null() {
        return;
    }
    layout_custom_unlink_panes(lcroot);
    layout_free_cell(lcroot, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn layout_append(
    mut lcroot: *mut layout_cell,
    mut ls: *mut layout_string,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lccompat: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut result: ::core::ffi::c_int = 0;
    if flags & LAYOUT_CUSTOM_OLD_FORMAT != 0 {
        if layout_cell_is_tiled(lcroot) == 0 && layout_cell_has_tiled_child(lcroot) == 0 {
            return -(1 as ::core::ffi::c_int);
        }
        lccompat = layout_custom_create_compat(lcroot);
        result = layout_append_v1(lccompat, ls);
        layout_custom_free_compat(lccompat);
    } else {
        result = layout_append_v2(lcroot, ls);
    }
    return result;
}
unsafe extern "C" fn layout_check(mut lc: *mut layout_cell) -> ::core::ffi::c_int {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut n: u_int = 0 as u_int;
    match (*lc).type_0 as ::core::ffi::c_uint {
        0 => {
            lcchild = (*lc).cells.tqh_first;
            while !lcchild.is_null() {
                if !(layout_cell_is_tiled(lcchild) == 0
                    && layout_cell_has_tiled_child(lcchild) == 0)
                {
                    if (*lcchild).g.sy != (*lc).g.sy {
                        return 0 as ::core::ffi::c_int;
                    }
                    if layout_check(lcchild) == 0 {
                        return 0 as ::core::ffi::c_int;
                    }
                    n = n.wrapping_add((*lcchild).g.sx.wrapping_add(1 as u_int));
                }
                lcchild = (*lcchild).entry.tqe_next;
            }
            if n != 0 as u_int && n.wrapping_sub(1 as u_int) != (*lc).g.sx {
                return 0 as ::core::ffi::c_int;
            }
        }
        1 => {
            lcchild = (*lc).cells.tqh_first;
            while !lcchild.is_null() {
                if !(layout_cell_is_tiled(lcchild) == 0
                    && layout_cell_has_tiled_child(lcchild) == 0)
                {
                    if (*lcchild).g.sx != (*lc).g.sx {
                        return 0 as ::core::ffi::c_int;
                    }
                    if layout_check(lcchild) == 0 {
                        return 0 as ::core::ffi::c_int;
                    }
                    n = n.wrapping_add((*lcchild).g.sy.wrapping_add(1 as u_int));
                }
                lcchild = (*lcchild).entry.tqe_next;
            }
            if n != 0 as u_int && n.wrapping_sub(1 as u_int) != (*lc).g.sy {
                return 0 as ::core::ffi::c_int;
            }
        }
        2 | _ => {}
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn layout_parse(
    mut w: *mut window,
    mut input: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut pctx: layout_parse_ctx = layout_parse_ctx {
        version: 0,
        num_active: 0,
        root: ::core::ptr::null_mut::<layout_cell>(),
        cause: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        size: 0,
        capacity: 0,
        cctxs: ::core::ptr::null_mut::<layout_parse_cell_ctx>(),
    };
    let mut npanes: u_int = 0;
    let mut ncells: u_int = 0;
    let mut sx: u_int = 0 as u_int;
    let mut sy: u_int = 0 as u_int;
    let mut with_floating: ::core::ffi::c_int = 0;
    layout_parse_init_ctx(&raw mut pctx, cause);
    if layout_construct(input, &raw mut pctx) != 0 as ::core::ffi::c_int {
        layout_parse_free_ctx(&raw mut pctx);
        return -(1 as ::core::ffi::c_int);
    }
    with_floating = (pctx.version > 1 as int64_t) as ::core::ffi::c_int;
    npanes = window_count_panes(w, with_floating);
    if npanes == 0 as u_int {
        xasprintf(
            cause,
            b"window @%u has no panes\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
        );
    } else {
        loop {
            ncells = layout_count_cells(pctx.root, with_floating);
            if npanes > ncells {
                xasprintf(
                    cause,
                    b"have %u panes but need %u\0" as *const u8 as *const ::core::ffi::c_char,
                    npanes,
                    ncells,
                );
                current_block = 4277046812173491162;
                break;
            } else {
                if npanes == ncells {
                    current_block = 15976848397966268834;
                    break;
                }
                lcchild = layout_find_bottomright(pctx.root);
                if pctx.version > 1 as int64_t
                    && layout_parse_remove_cctx(&raw mut pctx, lcchild) != 0 as ::core::ffi::c_int
                {
                    *cause = xstrdup(
                        b"empty/missing layout parse context\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                    current_block = 4277046812173491162;
                    break;
                } else {
                    layout_destroy_cell(
                        ::core::ptr::null_mut::<window>(),
                        lcchild,
                        &raw mut pctx.root,
                    );
                }
            }
        }
        match current_block {
            4277046812173491162 => {}
            _ => {
                lc = pctx.root;
                pctx.root = ::core::ptr::null_mut::<layout_cell>();
                match (*lc).type_0 as ::core::ffi::c_uint {
                    0 => {
                        lcchild = (*lc).cells.tqh_first;
                        while !lcchild.is_null() {
                            if layout_cell_is_tiled(lcchild) != 0
                                || layout_cell_has_tiled_child(lcchild) != 0
                            {
                                sy = (*lcchild).g.sy.wrapping_add(1 as u_int);
                                sx = sx.wrapping_add((*lcchild).g.sx.wrapping_add(1 as u_int));
                            }
                            lcchild = (*lcchild).entry.tqe_next;
                        }
                    }
                    1 => {
                        lcchild = (*lc).cells.tqh_first;
                        while !lcchild.is_null() {
                            if layout_cell_is_tiled(lcchild) != 0
                                || layout_cell_has_tiled_child(lcchild) != 0
                            {
                                sx = (*lcchild).g.sx.wrapping_add(1 as u_int);
                                sy = sy.wrapping_add((*lcchild).g.sy.wrapping_add(1 as u_int));
                            }
                            lcchild = (*lcchild).entry.tqe_next;
                        }
                    }
                    2 | _ => {}
                }
                if (*lc).type_0 as ::core::ffi::c_uint
                    != LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
                    && sx != 0 as u_int
                    && sy != 0 as u_int
                    && ((*lc).g.sx != sx || (*lc).g.sy != sy)
                {
                    layout_print_cell(
                        lc,
                        b"layout_parse\0" as *const u8 as *const ::core::ffi::c_char,
                        0 as u_int,
                    );
                    (*lc).g.sx = sx.wrapping_sub(1 as u_int);
                    (*lc).g.sy = sy.wrapping_sub(1 as u_int);
                }
                if layout_check(lc) == 0 {
                    *cause = xstrdup(
                        b"size mismatch after applying layout\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                } else {
                    if layout_cell_is_tiled(lc) != 0 || layout_cell_has_tiled_child(lc) != 0 {
                        window_resize(
                            w,
                            (*lc).g.sx,
                            (*lc).g.sy,
                            -(1 as ::core::ffi::c_int),
                            -(1 as ::core::ffi::c_int),
                        );
                    }
                    if pctx.version == 1 as int64_t {
                        wp = (*w).panes.tqh_first;
                        while !wp.is_null() {
                            if !(window_pane_is_floating(wp) == 0) {
                                lcchild = (*wp).layout_cell as *mut layout_cell;
                                if !(*lcchild).entry.tqe_next.is_null() {
                                    (*(*lcchild).entry.tqe_next).entry.tqe_prev =
                                        (*lcchild).entry.tqe_prev;
                                } else {
                                    (*(*lcchild).parent).cells.tqh_last = (*lcchild).entry.tqe_prev;
                                }
                                *(*lcchild).entry.tqe_prev = (*lcchild).entry.tqe_next;
                                (*lcchild).parent = ::core::ptr::null_mut::<layout_cell>();
                            }
                            wp = (*wp).entry.tqe_next;
                        }
                    }
                    layout_free_cell((*w).layout_root, 0 as ::core::ffi::c_int);
                    (*w).layout_root = lc;
                    layout_assign(w, &raw mut pctx);
                    layout_fix_offsets(w);
                    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
                    if pctx.version > 1 as int64_t {
                        layout_parse_apply_ctx(w, &raw mut pctx);
                    }
                    recalculate_sizes();
                    layout_print_cell(
                        lc,
                        b"layout_parse\0" as *const u8 as *const ::core::ffi::c_char,
                        0 as u_int,
                    );
                    if pctx.version == 1 as int64_t {
                        events_fire_window(
                            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
                            w,
                        );
                    }
                    layout_parse_free_ctx(&raw mut pctx);
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
    }
    layout_free_cell(lc, 0 as ::core::ffi::c_int);
    layout_parse_free_ctx(&raw mut pctx);
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn layout_assign_from_ctx(mut w: *mut window, mut pctx: *mut layout_parse_ctx) {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut i: ::core::ffi::c_int = 0;
    qsort(
        (*pctx).cctxs as *mut ::core::ffi::c_void,
        (*pctx).size as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        Some(
            layout_parse_index_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    wp = (*w).panes.tqh_first;
    i = 0 as ::core::ffi::c_int;
    while i < (*pctx).size {
        lc = (*(*pctx).cctxs.offset(i as isize)).lc;
        layout_make_leaf(lc, wp);
        wp = (*wp).entry.tqe_next;
        i += 1;
    }
}
unsafe extern "C" fn layout_assign_fallback_tiled(
    mut wp: *mut *mut window_pane,
    mut lc: *mut layout_cell,
) {
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if lc.is_null() {
        return;
    }
    match (*lc).type_0 as ::core::ffi::c_uint {
        2 => {
            while !(*wp).is_null() && !(**wp).layout_cell.is_null() {
                *wp = (**wp).entry.tqe_next;
            }
            if (*wp).is_null() {
                return;
            }
            layout_make_leaf(lc, *wp);
            *wp = (**wp).entry.tqe_next;
            return;
        }
        0 | 1 => {
            lcchild = (*lc).cells.tqh_first;
            while !lcchild.is_null() {
                layout_assign_fallback_tiled(wp, lcchild);
                lcchild = (*lcchild).entry.tqe_next;
            }
            return;
        }
        _ => {}
    };
}
unsafe extern "C" fn layout_assign_fallback(mut w: *mut window, mut lcroot: *mut layout_cell) {
    let mut wp: *mut window_pane = (*w).panes.tqh_first;
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    layout_assign_fallback_tiled(&raw mut wp, lcroot);
    if window_count_panes(w, 1 as ::core::ffi::c_int) > 1 as u_int
        && (*lcroot).type_0 as ::core::ffi::c_uint
            == LAYOUT_WINDOWPANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        lcroot = layout_replace_with_node(w, lcroot, LAYOUT_TOPBOTTOM);
    }
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if window_pane_is_floating(wp) != 0 {
            lc = (*wp).layout_cell as *mut layout_cell;
            (*lc).parent = lcroot;
            (*lc).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
            (*lc).entry.tqe_prev = (*lcroot).cells.tqh_last;
            *(*lcroot).cells.tqh_last = lc;
            (*lcroot).cells.tqh_last = &raw mut (*lc).entry.tqe_next;
        }
        wp = (*wp).entry.tqe_next;
    }
}
unsafe extern "C" fn layout_assign(mut w: *mut window, mut pctx: *mut layout_parse_ctx) {
    if (*pctx).size > 0 as ::core::ffi::c_int {
        layout_assign_from_ctx(w, pctx);
    } else {
        layout_assign_fallback(w, (*w).layout_root);
    };
}
unsafe extern "C" fn layout_construct_cell(
    mut lcparent: *mut layout_cell,
    mut layout: *mut *const ::core::ffi::c_char,
) -> *mut layout_cell {
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut saved: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if *(*__ctype_b_loc()).offset(**layout as u_char as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        == 0
    {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    if sscanf(
        *layout,
        b"%ux%u,%d,%d\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut sx,
        &raw mut sy,
        &raw mut xoff,
        &raw mut yoff,
    ) != 4 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    while *(*__ctype_b_loc()).offset(**layout as u_char as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        *layout = (*layout).offset(1);
    }
    if **layout as ::core::ffi::c_int != 'x' as i32 {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    *layout = (*layout).offset(1);
    while *(*__ctype_b_loc()).offset(**layout as u_char as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        *layout = (*layout).offset(1);
    }
    if **layout as ::core::ffi::c_int != ',' as i32 {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    *layout = (*layout).offset(1);
    while *(*__ctype_b_loc()).offset(**layout as u_char as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        *layout = (*layout).offset(1);
    }
    if **layout as ::core::ffi::c_int != ',' as i32 {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    *layout = (*layout).offset(1);
    while *(*__ctype_b_loc()).offset(**layout as u_char as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        *layout = (*layout).offset(1);
    }
    if **layout as ::core::ffi::c_int == ',' as i32 {
        saved = *layout;
        *layout = (*layout).offset(1);
        while *(*__ctype_b_loc()).offset(**layout as u_char as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        {
            *layout = (*layout).offset(1);
        }
        if **layout as ::core::ffi::c_int == 'x' as i32 {
            *layout = saved;
        }
    }
    lc = layout_create_cell(lcparent);
    (*lc).g.sx = sx;
    (*lc).g.sy = sy;
    (*lc).g.xoff = xoff;
    (*lc).g.yoff = yoff;
    return lc;
}
unsafe extern "C" fn layout_construct_v1(
    mut lcparent: *mut layout_cell,
    mut layout: *mut *const ::core::ffi::c_char,
    mut depth: u_int,
) -> *mut layout_cell {
    let mut current_block: u64;
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if depth > LAYOUT_V1_MAX_DEPTH as u_int {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    lc = layout_construct_cell(lcparent, layout);
    if lc.is_null() {
        return ::core::ptr::null_mut::<layout_cell>();
    }
    match **layout as ::core::ffi::c_int {
        44 | 125 | 93 | 0 => return lc,
        123 => {
            (*lc).type_0 = LAYOUT_LEFTRIGHT;
            current_block = 1917311967535052937;
        }
        91 => {
            (*lc).type_0 = LAYOUT_TOPBOTTOM;
            current_block = 1917311967535052937;
        }
        _ => {
            current_block = 17291956987205268033;
        }
    }
    loop {
        match current_block {
            17291956987205268033 => {
                layout_free_cell(lc, 0 as ::core::ffi::c_int);
                return ::core::ptr::null_mut::<layout_cell>();
            }
            _ => {
                *layout = (*layout).offset(1);
                lcchild = layout_construct_v1(lc, layout, depth.wrapping_add(1 as u_int));
                if lcchild.is_null() {
                    current_block = 17291956987205268033;
                    continue;
                }
                (*lcchild).entry.tqe_next = ::core::ptr::null_mut::<layout_cell>();
                (*lcchild).entry.tqe_prev = (*lc).cells.tqh_last;
                *(*lc).cells.tqh_last = lcchild;
                (*lc).cells.tqh_last = &raw mut (*lcchild).entry.tqe_next;
                if **layout as ::core::ffi::c_int == ',' as i32 {
                    current_block = 1917311967535052937;
                    continue;
                }
                match (*lc).type_0 as ::core::ffi::c_uint {
                    0 => {
                        if **layout as ::core::ffi::c_int != '}' as i32 {
                            current_block = 17291956987205268033;
                        } else {
                            break;
                        }
                    }
                    1 => {
                        if **layout as ::core::ffi::c_int != ']' as i32 {
                            current_block = 17291956987205268033;
                        } else {
                            break;
                        }
                    }
                    _ => {
                        current_block = 17291956987205268033;
                    }
                }
            }
        }
    }
    *layout = (*layout).offset(1);
    return lc;
}
unsafe extern "C" fn layout_parse_json(
    mut jnroot: *mut json_node,
    mut pctx: *mut layout_parse_ctx,
) -> ::core::ffi::c_int {
    let mut jn: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut object: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut num: int64_t = 0;
    let mut cause: *mut *mut ::core::ffi::c_char = (*pctx).cause;
    if json_get_object(jnroot, &raw mut jn) != 0 as ::core::ffi::c_int {
        *cause = xstrdup(b"invalid layout json\0" as *const u8 as *const ::core::ffi::c_char);
    } else if !(json_find_number(
        jn,
        b"V\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut num,
        cause,
    ) != 0 as ::core::ffi::c_int)
    {
        (*pctx).version = num;
        if !(json_find_object(
            jn,
            b"L\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut object,
            cause,
        ) != 0 as ::core::ffi::c_int)
        {
            (*pctx).root =
                layout_parse_json_layout(object, ::core::ptr::null_mut::<layout_cell>(), pctx);
            if !(*pctx).root.is_null() {
                json_destroy_node(jnroot);
                return 0 as ::core::ffi::c_int;
            }
        }
    }
    json_destroy_node(jnroot);
    if !(*pctx).root.is_null() {
        layout_free_cell((*pctx).root, 0 as ::core::ffi::c_int);
    }
    (*pctx).root = ::core::ptr::null_mut::<layout_cell>();
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn layout_parse_json_layout(
    mut node: *mut json_node,
    mut lcparent: *mut layout_cell,
    mut pctx: *mut layout_parse_ctx,
) -> *mut layout_cell {
    let mut current_block: u64;
    let mut member: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut array: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut lc: *mut layout_cell = layout_create_cell(lcparent);
    let mut lcchild: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut num: int64_t = 0;
    let mut cause: *mut *mut ::core::ffi::c_char = (*pctx).cause;
    let mut boolean: ::core::ffi::c_int = 0;
    let mut index: ::core::ffi::c_int = 0;
    let mut zindex: ::core::ffi::c_int = 0;
    let mut active: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut last: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    if !(json_find_string(
        node,
        b"t\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut str,
        cause,
    ) != 0 as ::core::ffi::c_int)
    {
        if strcmp(str, b"p\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
        {
            (*lc).type_0 = LAYOUT_WINDOWPANE;
            current_block = 1394248824506584008;
        } else if strcmp(str, b"v\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            (*lc).type_0 = LAYOUT_TOPBOTTOM;
            current_block = 1394248824506584008;
        } else if strcmp(str, b"h\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            (*lc).type_0 = LAYOUT_LEFTRIGHT;
            current_block = 1394248824506584008;
        } else {
            xasprintf(
                cause,
                b"unknown cell type \"%s\"\0" as *const u8 as *const ::core::ffi::c_char,
                str,
            );
            current_block = 14858222377052930936;
        }
        match current_block {
            14858222377052930936 => {}
            _ => {
                if !(json_find_number(
                    node,
                    b"w\0" as *const u8 as *const ::core::ffi::c_char,
                    &raw mut num,
                    cause,
                ) != 0 as ::core::ffi::c_int)
                {
                    if num < PANE_MINIMUM as int64_t || num > PANE_MAXIMUM as int64_t {
                        xasprintf(
                            cause,
                            b"invalid width %lld\0" as *const u8 as *const ::core::ffi::c_char,
                            num as ::core::ffi::c_longlong,
                        );
                    } else {
                        (*lc).g.sx = num as u_int;
                        if !(json_find_number(
                            node,
                            b"h\0" as *const u8 as *const ::core::ffi::c_char,
                            &raw mut num,
                            cause,
                        ) != 0 as ::core::ffi::c_int)
                        {
                            if num < PANE_MINIMUM as int64_t || num > PANE_MAXIMUM as int64_t {
                                xasprintf(
                                    cause,
                                    b"invalid height %lld\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                    num as ::core::ffi::c_longlong,
                                );
                            } else {
                                (*lc).g.sy = num as u_int;
                                if !(json_find_number(
                                    node,
                                    b"x\0" as *const u8 as *const ::core::ffi::c_char,
                                    &raw mut num,
                                    cause,
                                ) != 0 as ::core::ffi::c_int)
                                {
                                    if num < -WINDOW_MAXIMUM as int64_t
                                        || num > WINDOW_MAXIMUM as int64_t
                                    {
                                        xasprintf(
                                            cause,
                                            b"invalid x-offset %lld\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            num as ::core::ffi::c_longlong,
                                        );
                                    } else {
                                        (*lc).g.xoff = num as ::core::ffi::c_int;
                                        if !(json_find_number(
                                            node,
                                            b"y\0" as *const u8 as *const ::core::ffi::c_char,
                                            &raw mut num,
                                            cause,
                                        ) != 0 as ::core::ffi::c_int)
                                        {
                                            if num < -WINDOW_MAXIMUM as int64_t
                                                || num > WINDOW_MAXIMUM as int64_t
                                            {
                                                xasprintf(
                                                    cause,
                                                    b"invalid y-offset %lld\0" as *const u8
                                                        as *const ::core::ffi::c_char,
                                                    num as ::core::ffi::c_longlong,
                                                );
                                            } else {
                                                (*lc).g.yoff = num as ::core::ffi::c_int;
                                                if (*lc).type_0 as ::core::ffi::c_uint
                                                    == LAYOUT_WINDOWPANE as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    if !json_find(
                                                        node,
                                                        b"c\0" as *const u8
                                                            as *const ::core::ffi::c_char,
                                                    )
                                                    .is_null()
                                                    {
                                                        *cause = xstrdup(
                                                            b"panes cannot have children\0"
                                                                as *const u8
                                                                as *const ::core::ffi::c_char,
                                                        );
                                                        current_block = 14858222377052930936;
                                                    } else if json_find_number(
                                                        node,
                                                        b"i\0" as *const u8
                                                            as *const ::core::ffi::c_char,
                                                        &raw mut num,
                                                        cause,
                                                    ) != 0 as ::core::ffi::c_int
                                                    {
                                                        current_block = 14858222377052930936;
                                                    } else if num < 0 as int64_t
                                                        || num > INT_MAX as int64_t
                                                    {
                                                        xasprintf(
                                                            cause,
                                                            b"invalid index %lld\0" as *const u8
                                                                as *const ::core::ffi::c_char,
                                                            num as ::core::ffi::c_longlong,
                                                        );
                                                        current_block = 14858222377052930936;
                                                    } else {
                                                        index = num as ::core::ffi::c_int;
                                                        if !json_find(
                                                            node,
                                                            b"a\0" as *const u8
                                                                as *const ::core::ffi::c_char,
                                                        )
                                                        .is_null()
                                                        {
                                                            if json_find_boolean(
                                                                node,
                                                                b"a\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                &raw mut boolean,
                                                                cause,
                                                            ) != 0 as ::core::ffi::c_int
                                                            {
                                                                current_block =
                                                                    14858222377052930936;
                                                            } else {
                                                                active = boolean;
                                                                if active != 0 {
                                                                    (*pctx).num_active += 1;
                                                                }
                                                                current_block = 6450597802325118133;
                                                            }
                                                        } else if !json_find(
                                                            node,
                                                            b"l\0" as *const u8
                                                                as *const ::core::ffi::c_char,
                                                        )
                                                        .is_null()
                                                        {
                                                            if json_find_number(
                                                                node,
                                                                b"l\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                &raw mut num,
                                                                cause,
                                                            ) != 0 as ::core::ffi::c_int
                                                            {
                                                                current_block =
                                                                    14858222377052930936;
                                                            } else if num < 0 as int64_t
                                                                || num > INT_MAX as int64_t
                                                            {
                                                                xasprintf(
                                                                    cause,
                                                                    b"invalid last %lld\0" as *const u8
                                                                        as *const ::core::ffi::c_char,
                                                                    num as ::core::ffi::c_longlong,
                                                                );
                                                                current_block =
                                                                    14858222377052930936;
                                                            } else {
                                                                last = num as ::core::ffi::c_int;
                                                                current_block = 6450597802325118133;
                                                            }
                                                        } else {
                                                            current_block = 6450597802325118133;
                                                        }
                                                        match current_block {
                                                            14858222377052930936 => {}
                                                            _ => {
                                                                if !json_find(
                                                                        node,
                                                                        b"z\0" as *const u8 as *const ::core::ffi::c_char,
                                                                    )
                                                                    .is_null()
                                                                {
                                                                    if json_find_number(
                                                                        node,
                                                                        b"z\0" as *const u8 as *const ::core::ffi::c_char,
                                                                        &raw mut num,
                                                                        cause,
                                                                    ) != 0 as ::core::ffi::c_int
                                                                    {
                                                                        current_block = 14858222377052930936;
                                                                    } else if num < 0 as int64_t
                                                                        || num > (INT_MAX - 1 as ::core::ffi::c_int) as int64_t
                                                                    {
                                                                        xasprintf(
                                                                            cause,
                                                                            b"invalid floating zindex %lld\0" as *const u8
                                                                                as *const ::core::ffi::c_char,
                                                                            num as ::core::ffi::c_longlong,
                                                                        );
                                                                        current_block = 14858222377052930936;
                                                                    } else {
                                                                        zindex = num as ::core::ffi::c_int;
                                                                        (*lc).flags |= LAYOUT_CELL_FLOATING;
                                                                        current_block = 1345366029464561491;
                                                                    }
                                                                } else {
                                                                    zindex = INT_MAX;
                                                                    current_block = 1345366029464561491;
                                                                }
                                                                match current_block {
                                                                    14858222377052930936 => {}
                                                                    _ => {
                                                                        layout_parse_add_cctx(
                                                                            pctx, lc, active, last,
                                                                            index, zindex,
                                                                        );
                                                                        current_block =
                                                                            11441799814184323368;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else if json_find_array(
                                                    node,
                                                    b"c\0" as *const u8
                                                        as *const ::core::ffi::c_char,
                                                    &raw mut array,
                                                    cause,
                                                ) != 0 as ::core::ffi::c_int
                                                {
                                                    current_block = 14858222377052930936;
                                                } else {
                                                    member = json_array_first(array);
                                                    if member.is_null()
                                                        || json_array_next(member).is_null()
                                                    {
                                                        *cause = xstrdup(
                                                            b"nodes must have more than one child\0"
                                                                as *const u8
                                                                as *const ::core::ffi::c_char,
                                                        );
                                                        current_block = 14858222377052930936;
                                                    } else {
                                                        loop {
                                                            if member.is_null() {
                                                                current_block =
                                                                    11441799814184323368;
                                                                break;
                                                            }
                                                            lcchild = layout_parse_json_layout(
                                                                member, lc, pctx,
                                                            );
                                                            if lcchild.is_null() {
                                                                current_block =
                                                                    14858222377052930936;
                                                                break;
                                                            }
                                                            (*lcchild).entry.tqe_next =
                                                                ::core::ptr::null_mut::<layout_cell>(
                                                                );
                                                            (*lcchild).entry.tqe_prev =
                                                                (*lc).cells.tqh_last;
                                                            *(*lc).cells.tqh_last = lcchild;
                                                            (*lc).cells.tqh_last =
                                                                &raw mut (*lcchild).entry.tqe_next;
                                                            member = json_array_next(member);
                                                        }
                                                    }
                                                }
                                                match current_block {
                                                    14858222377052930936 => {}
                                                    _ => return lc,
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    layout_free_cell(lc, 0 as ::core::ffi::c_int);
    return ::core::ptr::null_mut::<layout_cell>();
}
unsafe extern "C" fn layout_construct(
    mut input: *const ::core::ffi::c_char,
    mut pctx: *mut layout_parse_ctx,
) -> ::core::ffi::c_int {
    let mut json: *mut json_node = ::core::ptr::null_mut::<json_node>();
    let mut csum: u_short = 0;
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while *(*__ctype_b_loc()).offset(*input as u_char as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        input = input.offset(1);
    }
    if *input as ::core::ffi::c_int != '{' as i32 {
        if sscanf(
            input,
            b"%hx,%n\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut csum,
            &raw mut n,
        ) != 1 as ::core::ffi::c_int
            || n != 5 as ::core::ffi::c_int
        {
            *(*pctx).cause =
                xstrdup(b"malformed layout header\0" as *const u8 as *const ::core::ffi::c_char);
            return -(1 as ::core::ffi::c_int);
        }
        input = input.offset(n as isize);
        if csum as ::core::ffi::c_int != layout_checksum(input) as ::core::ffi::c_int {
            *(*pctx).cause =
                xstrdup(b"invalid layout checksum\0" as *const u8 as *const ::core::ffi::c_char);
            return -(1 as ::core::ffi::c_int);
        }
        (*pctx).root = layout_construct_v1(
            ::core::ptr::null_mut::<layout_cell>(),
            &raw mut input,
            0 as u_int,
        );
        if (*pctx).root.is_null() {
            *(*pctx).cause =
                xstrdup(b"invalid layout\0" as *const u8 as *const ::core::ffi::c_char);
            return -(1 as ::core::ffi::c_int);
        }
        if *input as ::core::ffi::c_int != '\0' as i32 {
            *(*pctx).cause = xstrdup(b"trailing data\0" as *const u8 as *const ::core::ffi::c_char);
            return -(1 as ::core::ffi::c_int);
        }
        (*pctx).version = 1 as int64_t;
    } else {
        json = json_parse(input, (*pctx).cause);
        if json.is_null() {
            return -(1 as ::core::ffi::c_int);
        }
        if layout_parse_json(json, pctx) != 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
        if (*pctx).version != 2 as int64_t {
            *(*pctx).cause =
                xstrdup(b"version mismatch\0" as *const u8 as *const ::core::ffi::c_char);
            return -(1 as ::core::ffi::c_int);
        }
        if (*pctx).num_active > 1 as ::core::ffi::c_int {
            *(*pctx).cause =
                xstrdup(b"more than one active pane\0" as *const u8 as *const ::core::ffi::c_char);
            return -(1 as ::core::ffi::c_int);
        }
        if (*pctx).size == 0 as ::core::ffi::c_int {
            *(*pctx).cause = xstrdup(b"no panes\0" as *const u8 as *const ::core::ffi::c_char);
            return -(1 as ::core::ffi::c_int);
        }
        if layout_parse_ctx_check_indexes(pctx) == 0 {
            return -(1 as ::core::ffi::c_int);
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn layout_parse_apply_ctx(mut w: *mut window, mut pctx: *mut layout_parse_ctx) {
    let mut cctx: *mut layout_parse_cell_ctx = ::core::ptr::null_mut::<layout_parse_cell_ctx>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wpnext: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut i: ::core::ffi::c_int = 0;
    wp = (*w).z_index.tqh_first;
    while !wp.is_null() {
        wpnext = (*wp).zentry.tqe_next;
        if window_pane_is_floating(wp) != 0 {
            if !(*wp).zentry.tqe_next.is_null() {
                (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
            } else {
                (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
            }
            *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
        }
        wp = wpnext;
    }
    qsort(
        (*pctx).cctxs as *mut ::core::ffi::c_void,
        (*pctx).size as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        Some(
            layout_parse_zindex_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    i = 0 as ::core::ffi::c_int;
    while i < (*pctx).size {
        cctx = (*pctx).cctxs.offset(i as isize) as *mut layout_parse_cell_ctx;
        wp = (*(*cctx).lc).wp;
        if window_pane_is_floating(wp) != 0 {
            (*wp).zentry.tqe_next = (*w).z_index.tqh_first;
            if !(*wp).zentry.tqe_next.is_null() {
                (*(*w).z_index.tqh_first).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
            } else {
                (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
            }
            (*w).z_index.tqh_first = wp;
            (*wp).zentry.tqe_prev = &raw mut (*w).z_index.tqh_first;
        }
        i += 1;
    }
    i = 0 as ::core::ffi::c_int;
    while i < (*pctx).size {
        cctx = (*pctx).cctxs.offset(i as isize) as *mut layout_parse_cell_ctx;
        if (*cctx).active == 1 as ::core::ffi::c_int {
            window_set_active_pane(w, (*(*cctx).lc).wp, 1 as ::core::ffi::c_int);
            break;
        } else {
            i += 1;
        }
    }
    while !(*w).last_panes.tqh_first.is_null() {
        wp = (*w).last_panes.tqh_first;
        window_pane_stack_remove(&raw mut (*w).last_panes, wp);
    }
    qsort(
        (*pctx).cctxs as *mut ::core::ffi::c_void,
        (*pctx).size as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        Some(
            layout_parse_last_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    i = 0 as ::core::ffi::c_int;
    while i < (*pctx).size {
        cctx = (*pctx).cctxs.offset(i as isize) as *mut layout_parse_cell_ctx;
        wp = (*(*cctx).lc).wp;
        if !((*cctx).last < 0 as ::core::ffi::c_int || (*cctx).active == 1 as ::core::ffi::c_int) {
            window_pane_stack_push(&raw mut (*w).last_panes, wp);
        }
        i += 1;
    }
}
unsafe extern "C" fn layout_parse_ctx_check_indexes(
    mut pctx: *mut layout_parse_ctx,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    qsort(
        (*pctx).cctxs as *mut ::core::ffi::c_void,
        (*pctx).size as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        Some(
            layout_parse_index_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    i = 1 as ::core::ffi::c_int;
    while i < (*pctx).size {
        if (*(*pctx).cctxs.offset(i as isize)).index
            == (*(*pctx).cctxs.offset((i - 1 as ::core::ffi::c_int) as isize)).index
        {
            *(*pctx).cause =
                xstrdup(b"duplicate pane index\0" as *const u8 as *const ::core::ffi::c_char);
            return 0 as ::core::ffi::c_int;
        }
        i += 1;
    }
    qsort(
        (*pctx).cctxs as *mut ::core::ffi::c_void,
        (*pctx).size as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        Some(
            layout_parse_zindex_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    n = 0 as ::core::ffi::c_int;
    while n < (*pctx).size && (*(*pctx).cctxs.offset(n as isize)).zindex == INT_MAX {
        n += 1;
    }
    i = n + 1 as ::core::ffi::c_int;
    while i < (*pctx).size {
        if (*(*pctx).cctxs.offset(i as isize)).zindex
            == (*(*pctx).cctxs.offset((i - 1 as ::core::ffi::c_int) as isize)).zindex
        {
            *(*pctx).cause =
                xstrdup(b"duplicate pane z-index\0" as *const u8 as *const ::core::ffi::c_char);
            return 0 as ::core::ffi::c_int;
        }
        i += 1;
    }
    qsort(
        (*pctx).cctxs as *mut ::core::ffi::c_void,
        (*pctx).size as size_t,
        ::core::mem::size_of::<layout_parse_cell_ctx>() as size_t,
        Some(
            layout_parse_last_cmp
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    );
    n = 0 as ::core::ffi::c_int;
    while n < (*pctx).size && (*(*pctx).cctxs.offset(n as isize)).last >= 0 as ::core::ffi::c_int {
        n += 1;
    }
    i = 1 as ::core::ffi::c_int;
    while i < n {
        if (*(*pctx).cctxs.offset(i as isize)).last
            == (*(*pctx).cctxs.offset((i - 1 as ::core::ffi::c_int) as isize)).last
        {
            *(*pctx).cause =
                xstrdup(b"duplicate last pane index\0" as *const u8 as *const ::core::ffi::c_char);
            return 0 as ::core::ffi::c_int;
        }
        i += 1;
    }
    return 1 as ::core::ffi::c_int;
}

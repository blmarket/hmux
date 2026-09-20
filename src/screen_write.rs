pub use crate::src::shared::limits::{__INT_MAX__, UINT_MAX};
pub use crate::src::shared::utf8::{
    HANGULJAMO_STATE_CHOSEONG, HANGULJAMO_STATE_COMPOSABLE, HANGULJAMO_STATE_NOT_COMPOSABLE,
    HANGULJAMO_STATE_NOT_HANGULJAMO, hanguljamo_state,
};
pub use crate::src::shared::pane::{
    PANE_DROP, PANE_REDRAW, PANE_REDRAWSCROLLBAR, window_pane_offset, window_pane_resize,
    window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::screen::{
    EXTENDED_KEY_MODES, MODE_CURSOR, MODE_INSERT, MODE_KEYS_EXTENDED, MODE_KEYS_EXTENDED_2,
    MODE_ORIGIN, MODE_SYNC, MODE_WRAP, screen, screen_sel, screen_titles,
};
pub use crate::src::shared::menu::{menu, menu_item};
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::abi::{ssize_t};
pub use crate::src::shared::screen_write::{
    CLEAR, TEXT, screen_write_citem, screen_write_cline, screen_write_item_link,
    screen_write_item_type, screen_write_items,
};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::tty::{
    TTY_CTX_CELL_INVALIDATE, TTY_CTX_INVISIBLE_PANES, TTY_CTX_OVERLAY_SYNC,
    TTY_CTX_PANE_OBSCURED, TTY_CTX_SYNC, TTY_CTX_WINDOW_BIGGER, TTY_CTX_WRAPPED,
};
pub use crate::src::shared::event::{EV_TIMEOUT};
pub use crate::src::shared::client::{CLIENT_REDRAWWINDOW};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::layout::*;
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
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
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
    fn calloc(__nmemb: size_t, __size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xvasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    static mut global_options: *mut options;
    fn format_draw(
        _: *mut screen_write_ctx,
        _: *const grid_cell,
        _: u_int,
        _: *const ::core::ffi::c_char,
        _: *mut style_ranges,
        _: ::core::ffi::c_int,
    );
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn tty_window_offset(
        _: *mut tty,
        _: *mut u_int,
        _: *mut u_int,
        _: *mut u_int,
        _: *mut u_int,
    ) -> ::core::ffi::c_int;
    fn tty_update_window_offset(_: *mut window);
    fn tty_write(_: Option<unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()>, _: *mut tty_ctx);
    fn tty_cmd_alignmenttest(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_cell(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_cells(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_redrawline(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_clearendofscreen(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_clearscreen(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_clearstartofscreen(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_deletecharacter(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_clearcharacter(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_deleteline(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_insertcharacter(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_insertline(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_scrollup(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_scrolldown(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_reverseindex(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_setselection(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_rawstring(_: *mut tty, _: *const tty_ctx);
    fn tty_cmd_syncstart(_: *mut tty, _: *const tty_ctx);
    fn tty_default_colours(_: *mut grid_cell, _: *mut window_pane, _: *mut u_int);
    fn tty_acs_double_borders(_: ::core::ffi::c_int) -> *const utf8_data;
    fn tty_acs_heavy_borders(_: ::core::ffi::c_int) -> *const utf8_data;
    fn tty_acs_rounded_borders(_: ::core::ffi::c_int) -> *const utf8_data;
    fn server_redraw_window_borders(_: *mut window);
    fn status_at_line(_: *mut client) -> ::core::ffi::c_int;
    fn status_line_size(_: *mut client) -> u_int;
    static grid_default_cell: grid_cell;
    fn grid_cells_equal(_: *const grid_cell, _: *const grid_cell) -> ::core::ffi::c_int;
    fn grid_clear_history(_: *mut grid);
    fn grid_get_cell(_: *mut grid, _: u_int, _: u_int, _: *mut grid_cell);
    fn grid_get_line(_: *mut grid, _: u_int) -> *mut grid_line;
    fn grid_view_get_cell(_: *mut grid, _: u_int, _: u_int, _: *mut grid_cell);
    fn grid_view_set_cell(_: *mut grid, _: u_int, _: u_int, _: *const grid_cell);
    fn grid_view_set_padding(_: *mut grid, _: u_int, _: u_int, _: ::core::ffi::c_int);
    fn grid_view_set_cells(
        _: *mut grid,
        _: u_int,
        _: u_int,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
        _: size_t,
    );
    fn grid_view_clear_history(_: *mut grid, _: u_int);
    fn grid_view_clear(_: *mut grid, _: u_int, _: u_int, _: u_int, _: u_int, _: u_int);
    fn grid_view_scroll_region_up(_: *mut grid, _: u_int, _: u_int, _: u_int);
    fn grid_view_scroll_region_down(_: *mut grid, _: u_int, _: u_int, _: u_int);
    fn grid_view_insert_lines(_: *mut grid, _: u_int, _: u_int, _: u_int);
    fn grid_view_insert_lines_region(_: *mut grid, _: u_int, _: u_int, _: u_int, _: u_int);
    fn grid_view_delete_lines(_: *mut grid, _: u_int, _: u_int, _: u_int);
    fn grid_view_delete_lines_region(_: *mut grid, _: u_int, _: u_int, _: u_int, _: u_int);
    fn grid_view_insert_cells(_: *mut grid, _: u_int, _: u_int, _: u_int, _: u_int);
    fn grid_view_delete_cells(_: *mut grid, _: u_int, _: u_int, _: u_int, _: u_int);
    fn screen_reset_tabs(_: *mut screen);
    fn screen_check_selection(_: *mut screen, _: u_int, _: u_int) -> ::core::ffi::c_int;
    fn screen_select_cell(
        _: *mut screen,
        _: *mut grid_cell,
        _: *const grid_cell,
    ) -> ::core::ffi::c_int;
    fn screen_alternate_on(
        _: *mut screen,
        _: *mut grid_cell,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn screen_alternate_off(
        _: *mut screen,
        _: *mut grid_cell,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn screen_mode_to_string(_: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn window_pane_send_resize(_: *mut window_pane, _: u_int, _: u_int);
    fn window_pane_clear_resizes(_: *mut window_pane, _: *mut window_pane_resize);
    fn window_pane_scrollbar_overlay_visible(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_scrollbar_redraw(_: *mut window_pane);
    fn window_pane_is_floating(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_position_is_visible(_: *mut visible_ranges, _: u_int) -> ::core::ffi::c_int;
    fn window_visible_ranges(
        _: *mut window_pane,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: u_int,
        _: *mut visible_ranges,
    ) -> *mut visible_ranges;
    fn layout_fix_panes(_: *mut window, _: *mut window_pane);
    fn session_has(_: *mut session, _: *mut window) -> ::core::ffi::c_int;
    fn utf8_set(_: *mut utf8_data, _: u_char);
    fn utf8_copy(_: *mut utf8_data, _: *const utf8_data);
    fn utf8_open(_: *mut utf8_data, _: u_char) -> utf8_state;
    fn utf8_append(_: *mut utf8_data, _: u_char) -> utf8_state;
    fn utf8_fromcstr(_: *const ::core::ffi::c_char) -> *mut utf8_data;
    fn utf8_has_zwj(_: *const utf8_data) -> ::core::ffi::c_int;
    fn utf8_is_zwj(_: *const utf8_data) -> ::core::ffi::c_int;
    fn utf8_is_vs(_: *const utf8_data) -> ::core::ffi::c_int;
    fn utf8_is_hangul_filler(_: *const utf8_data) -> ::core::ffi::c_int;
    fn utf8_should_combine(_: *const utf8_data, _: *const utf8_data) -> ::core::ffi::c_int;
    fn hanguljamo_check_state(_: *const utf8_data, _: *const utf8_data) -> hanguljamo_state;
    fn log_get_level() -> ::core::ffi::c_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatal(_: *const ::core::ffi::c_char, ...) -> !;
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
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
pub union C2RustUnnamed_15 {
    pub offset: u_int,
    pub data: C2RustUnnamed_16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_16 {
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
    pub gentry: C2RustUnnamed_18,
    pub entry: C2RustUnnamed_17,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_17 {
    pub rbe_left: *mut session,
    pub rbe_right: *mut session,
    pub rbe_parent: *mut session,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_18 {
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
    pub entry: C2RustUnnamed_37,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
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
    pub c2rust_unnamed: C2RustUnnamed_38,
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
pub union C2RustUnnamed_38 {
    pub n: u_int,
    pub data: C2RustUnnamed_40,
    pub sel: C2RustUnnamed_39,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_39 {
    pub clip: *const ::core::ffi::c_char,
    pub data: *const ::core::ffi::c_char,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_40 {
    pub data: *const ::core::ffi::c_char,
    pub size: size_t,
}
pub type tty_ctx_set_client_cb =
    Option<unsafe extern "C" fn(*mut tty_ctx, *mut client) -> ::core::ffi::c_int>;
pub type tty_ctx_redraw_cb = Option<unsafe extern "C" fn(*const tty_ctx) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_41 {
    pub tqh_first: *mut screen_write_citem,
    pub tqh_last: *mut *mut screen_write_citem,
}

pub const CELL_UD: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CELL_LR: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CELL_RD: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const CELL_LD: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const CELL_RU: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const CELL_LU: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const CELL_URD: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const CELL_ULD: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const CELL_BORDERS: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b" xqlkmjwvtun~\0") };
pub const SIMPLE_BORDERS: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b" |-+++++++++.\0") };
pub const PADDED_BORDERS: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"             \0") };
pub const SCREEN_WRITE_SYNC: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const SCREEN_WRITE_OBSCURED: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const SCREEN_WRITE_CHECKED_IF_OBSCURED: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
#[no_mangle]
pub static mut screen_write_citem_freelist: C2RustUnnamed_41 = C2RustUnnamed_41 {
    tqh_first: ::core::ptr::null::<screen_write_citem>() as *mut screen_write_citem,
    tqh_last: ::core::ptr::null::<*mut screen_write_citem>() as *mut *mut screen_write_citem,
};
unsafe extern "C" fn screen_write_get_citem() -> *mut screen_write_citem {
    let mut ci: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    ci = screen_write_citem_freelist.tqh_first;
    if !ci.is_null() {
        if !(*ci).entry.tqe_next.is_null() {
            (*(*ci).entry.tqe_next).entry.tqe_prev = (*ci).entry.tqe_prev;
        } else {
            screen_write_citem_freelist.tqh_last = (*ci).entry.tqe_prev;
        }
        *(*ci).entry.tqe_prev = (*ci).entry.tqe_next;
        memset(
            ci as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<screen_write_citem>() as size_t,
        );
        return ci;
    }
    return xcalloc(
        1 as size_t,
        ::core::mem::size_of::<screen_write_citem>() as size_t,
    ) as *mut screen_write_citem;
}
unsafe extern "C" fn screen_write_free_citem(mut ci: *mut screen_write_citem) {
    (*ci).entry.tqe_next = ::core::ptr::null_mut::<screen_write_citem>();
    (*ci).entry.tqe_prev = screen_write_citem_freelist.tqh_last;
    *screen_write_citem_freelist.tqh_last = ci;
    screen_write_citem_freelist.tqh_last = &raw mut (*ci).entry.tqe_next;
}
unsafe extern "C" fn screen_write_offset_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut w: *mut window = data as *mut window;
    tty_update_window_offset(w);
}
unsafe extern "C" fn screen_write_set_cursor(
    mut ctx: *mut screen_write_ctx,
    mut cx: ::core::ffi::c_int,
    mut cy: ::core::ffi::c_int,
) {
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut s: *mut screen = (*ctx).s;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 10000 as __suseconds_t,
    };
    if cx != -(1 as ::core::ffi::c_int)
        && cx as u_int == (*s).cx
        && cy != -(1 as ::core::ffi::c_int)
        && cy as u_int == (*s).cy
    {
        return;
    }
    if cx != -(1 as ::core::ffi::c_int) {
        if cx as u_int > (*(*s).grid).sx {
            cx = (*(*s).grid).sx.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
        }
        (*s).cx = cx as u_int;
    }
    if cy != -(1 as ::core::ffi::c_int) {
        if cy as u_int > (*(*s).grid).sy.wrapping_sub(1 as u_int) {
            cy = (*(*s).grid).sy.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
        }
        (*s).cy = cy as u_int;
    }
    if wp.is_null() {
        return;
    }
    w = (*wp).window as *mut window;
    if event_initialized(&raw mut (*w).offset_timer) == 0 {
        event_set(
            &raw mut (*w).offset_timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            Some(
                screen_write_offset_timer
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            w as *mut ::core::ffi::c_void,
        );
    }
    if event_pending(
        &raw mut (*w).offset_timer,
        EV_TIMEOUT as ::core::ffi::c_short,
        ::core::ptr::null_mut::<timeval>(),
    ) == 0
    {
        event_add(&raw mut (*w).offset_timer, &raw mut tv);
    }
}
unsafe extern "C" fn screen_write_redraw_cb(mut ttyctx: *const tty_ctx) {
    let mut wp: *mut window_pane = (*ttyctx).arg as *mut window_pane;
    if !wp.is_null() {
        (*wp).flags |= PANE_REDRAW;
    }
}
unsafe extern "C" fn screen_write_set_client_cb(
    mut ttyctx: *mut tty_ctx,
    mut c: *mut client,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*ttyctx).arg as *mut window_pane;
    if (*ttyctx).flags & TTY_CTX_INVISIBLE_PANES != 0 {
        if session_has((*c).session, (*wp).window as *mut window) != 0 {
            return 1 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
    if (*(*(*c).session).curw).window != (*wp).window {
        return 0 as ::core::ffi::c_int;
    }
    if (*wp).layout_cell.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*wp).flags & (PANE_REDRAW | PANE_DROP) != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if (*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
        log_debug(
            b"%s: adding %%%u to deferred redraw\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_set_client_cb\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
        );
        (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR;
        return -(1 as ::core::ffi::c_int);
    }
    if tty_window_offset(
        &raw mut (*c).tty,
        &raw mut (*ttyctx).wox,
        &raw mut (*ttyctx).woy,
        &raw mut (*ttyctx).wsx,
        &raw mut (*ttyctx).wsy,
    ) != 0
    {
        (*ttyctx).flags |= TTY_CTX_WINDOW_BIGGER;
    } else {
        (*ttyctx).flags &= !TTY_CTX_WINDOW_BIGGER;
    }
    (*ttyctx).rxoff = (*wp).xoff;
    (*ttyctx).xoff = (*ttyctx).rxoff;
    (*ttyctx).ryoff = (*wp).yoff;
    (*ttyctx).yoff = (*ttyctx).ryoff;
    if status_at_line(c) == 0 as ::core::ffi::c_int {
        (*ttyctx).yoff = ((*ttyctx).yoff as u_int).wrapping_add(status_line_size(c))
            as ::core::ffi::c_int as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn screen_write_pane_is_obscured(
    mut ctx: *mut screen_write_ctx,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    if (*ctx).wp.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*ctx).flags & SCREEN_WRITE_CHECKED_IF_OBSCURED != 0 {
        if (*ctx).flags & SCREEN_WRITE_OBSCURED != 0 {
            return 1 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
    (*ctx).flags |= SCREEN_WRITE_CHECKED_IF_OBSCURED;
    if (*(*ctx).wp).xoff < 0 as ::core::ffi::c_int
        || (*(*ctx).wp).yoff < 0 as ::core::ffi::c_int
        || ((*(*ctx).wp).xoff as u_int).wrapping_add((*(*ctx).wp).sx) > (*(*(*ctx).wp).window).sx
        || ((*(*ctx).wp).yoff as u_int).wrapping_add((*(*ctx).wp).sy) > (*(*(*ctx).wp).window).sy
    {
        (*ctx).flags |= SCREEN_WRITE_OBSCURED;
        return 1 as ::core::ffi::c_int;
    }
    loop {
        wp = *(*((*wp).zentry.tqe_prev as *mut window_panes)).tqh_last;
        if wp.is_null() {
            break;
        }
        if window_pane_is_floating(wp) != 0
            && ((*wp).yoff >= (*(*ctx).wp).yoff
                && (*wp).yoff <= (*(*ctx).wp).yoff + (*(*ctx).wp).sy as ::core::ffi::c_int
                || (*wp).yoff + (*wp).sy as ::core::ffi::c_int >= (*(*ctx).wp).yoff
                    && ((*wp).yoff as u_int).wrapping_add((*wp).sy)
                        <= ((*(*ctx).wp).yoff as u_int).wrapping_add((*(*ctx).wp).sy))
            && ((*wp).xoff >= (*(*ctx).wp).xoff
                && (*wp).xoff <= (*(*ctx).wp).xoff + (*(*ctx).wp).sx as ::core::ffi::c_int
                || (*wp).xoff + (*wp).sx as ::core::ffi::c_int >= (*(*ctx).wp).xoff
                    && ((*wp).xoff as u_int).wrapping_add((*wp).sx)
                        <= ((*(*ctx).wp).xoff as u_int).wrapping_add((*(*ctx).wp).sx))
        {
            (*ctx).flags |= SCREEN_WRITE_OBSCURED;
            return 1 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn screen_write_should_draw_lines(
    mut ctx: *mut screen_write_ctx,
    mut y: u_int,
    mut ny: u_int,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut s: *mut screen = (*ctx).s;
    let mut sy: u_int = (*(*s).grid).sy;
    let mut bs: *mut bitstr_t = ::core::ptr::null_mut::<bitstr_t>();
    if !wp.is_null() && (*wp).flags & (PANE_REDRAW | PANE_DROP) != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*s).mode & MODE_SYNC != 0 {
        if !wp.is_null() && y < sy && ny != 0 as u_int {
            bs = (*wp).sync_dirty;
            if ny > sy.wrapping_sub(y) {
                ny = sy.wrapping_sub(y);
            }
            if bs.is_null() || (*wp).sync_dirty_size != sy {
                if !bs.is_null() && (*wp).sync_dirty_size != sy {
                    y = 0 as u_int;
                    ny = sy;
                }
                free(bs as *mut ::core::ffi::c_void);
                (*wp).sync_dirty = calloc(
                    (sy.wrapping_add(7 as u_int) >> 3 as ::core::ffi::c_int) as size_t,
                    ::core::mem::size_of::<bitstr_t>() as size_t,
                ) as *mut bitstr_t;
                bs = (*wp).sync_dirty;
                if bs.is_null() {
                    fatal(b"bit_alloc failed\0" as *const u8 as *const ::core::ffi::c_char);
                }
                (*wp).sync_dirty_size = sy;
            }
            let mut _name: *mut bitstr_t = bs;
            let mut _start: ::core::ffi::c_int = y as ::core::ffi::c_int;
            let mut _stop: ::core::ffi::c_int =
                y.wrapping_add(ny).wrapping_sub(1 as u_int) as ::core::ffi::c_int;
            while _start <= _stop {
                let ref mut fresh2 = *_name.offset((_start >> 3 as ::core::ffi::c_int) as isize);
                *fresh2 = (*fresh2 as ::core::ffi::c_int
                    | (1 as ::core::ffi::c_int) << (_start & 0x7 as ::core::ffi::c_int))
                    as bitstr_t;
                _start += 1;
            }
        }
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn screen_write_should_draw_line(
    mut ctx: *mut screen_write_ctx,
    mut y: u_int,
) -> ::core::ffi::c_int {
    return screen_write_should_draw_lines(ctx, y, 1 as u_int);
}
unsafe extern "C" fn screen_write_initctx(
    mut ctx: *mut screen_write_ctx,
    mut ttyctx: *mut tty_ctx,
    mut is_sync: ::core::ffi::c_int,
    mut check_obscured: ::core::ffi::c_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut palette: *mut colour_palette = ::core::ptr::null_mut::<colour_palette>();
    memset(
        ttyctx as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<tty_ctx>() as size_t,
    );
    (*ttyctx).s = s;
    (*ttyctx).sx = (*(*s).grid).sx;
    (*ttyctx).sy = (*(*s).grid).sy;
    (*ttyctx).ocx = (*s).cx;
    (*ttyctx).ocy = (*s).cy;
    (*ttyctx).orlower = (*s).rlower;
    (*ttyctx).orupper = (*s).rupper;
    if check_obscured != 0 && screen_write_pane_is_obscured(ctx) != 0 {
        (*ttyctx).flags |= TTY_CTX_PANE_OBSCURED;
    }
    memcpy(
        &raw mut (*ttyctx).defaults as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    (*ttyctx).style_ctx.defaults = &raw mut (*ttyctx).defaults;
    (*ttyctx).style_ctx.hyperlinks = (*(*ctx).s).hyperlinks;
    if (*ctx).init_ctx_cb.is_some() {
        (*ctx).init_ctx_cb.expect("non-null function pointer")(ctx, ttyctx);
        if !(*ttyctx).style_ctx.palette.is_null() {
            palette = (*ttyctx).style_ctx.palette;
            if (*ttyctx).defaults.fg == 8 as ::core::ffi::c_int {
                (*ttyctx).defaults.fg = (*palette).fg;
            }
            if (*ttyctx).defaults.bg == 8 as ::core::ffi::c_int {
                (*ttyctx).defaults.bg = (*palette).bg;
            }
        }
    } else {
        (*ttyctx).redraw_cb =
            Some(screen_write_redraw_cb as unsafe extern "C" fn(*const tty_ctx) -> ())
                as tty_ctx_redraw_cb;
        if !(*ctx).wp.is_null() {
            tty_default_colours(
                &raw mut (*ttyctx).defaults,
                (*ctx).wp as *mut window_pane,
                &raw mut (*ttyctx).style_ctx.dim,
            );
            (*ttyctx).style_ctx.palette = &raw mut (*(*ctx).wp).palette;
            (*ttyctx).set_client_cb = Some(
                screen_write_set_client_cb
                    as unsafe extern "C" fn(*mut tty_ctx, *mut client) -> ::core::ffi::c_int,
            ) as tty_ctx_set_client_cb;
            (*ttyctx).arg = (*ctx).wp as *mut ::core::ffi::c_void;
        }
    }
    if !(*ctx).flags & SCREEN_WRITE_SYNC != 0 {
        if !(*ctx).wp.is_null()
            && ((*ctx).wp != (*(*(*ctx).wp).window).active
                || (*(*ctx).wp).screen != &raw mut (*(*ctx).wp).base)
        {
            (*ttyctx).flags |= TTY_CTX_SYNC;
        } else {
            if (*ctx).wp.is_null() {
                (*ttyctx).flags |= TTY_CTX_OVERLAY_SYNC;
            }
            if is_sync != 0 {
                (*ttyctx).flags |= TTY_CTX_SYNC;
            }
        }
        tty_write(
            Some(tty_cmd_syncstart as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            ttyctx,
        );
        (*ctx).flags |= SCREEN_WRITE_SYNC;
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_make_list(mut s: *mut screen) {
    let mut y: u_int = 0;
    (*s).write_list = xcalloc(
        (*(*s).grid).sy as size_t,
        ::core::mem::size_of::<screen_write_cline>() as size_t,
    ) as *mut screen_write_cline;
    y = 0 as u_int;
    while y < (*(*s).grid).sy {
        let ref mut fresh0 = (*(*s).write_list.offset(y as isize)).items.tqh_first;
        *fresh0 = ::core::ptr::null_mut::<screen_write_citem>();
        let ref mut fresh1 = (*(*s).write_list.offset(y as isize)).items.tqh_last;
        *fresh1 = &raw mut (*(*s).write_list.offset(y as isize)).items.tqh_first;
        y = y.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_free_list(mut s: *mut screen) {
    let mut cl: *mut screen_write_cline = ::core::ptr::null_mut::<screen_write_cline>();
    let mut ci: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    let mut ci1: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    let mut y: u_int = 0;
    y = 0 as u_int;
    while y < (*(*s).grid).sy {
        cl = (*s).write_list.offset(y as isize) as *mut screen_write_cline;
        ci = (*cl).items.tqh_first;
        while !ci.is_null() && {
            ci1 = (*ci).entry.tqe_next;
            1 as ::core::ffi::c_int != 0
        } {
            if !(*ci).entry.tqe_next.is_null() {
                (*(*ci).entry.tqe_next).entry.tqe_prev = (*ci).entry.tqe_prev;
            } else {
                (*cl).items.tqh_last = (*ci).entry.tqe_prev;
            }
            *(*ci).entry.tqe_prev = (*ci).entry.tqe_next;
            screen_write_free_citem(ci);
            ci = ci1;
        }
        free((*cl).data as *mut ::core::ffi::c_void);
        y = y.wrapping_add(1);
    }
    free((*s).write_list as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn screen_write_init(mut ctx: *mut screen_write_ctx, mut s: *mut screen) {
    memset(
        ctx as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<screen_write_ctx>() as size_t,
    );
    (*ctx).s = s;
    if (*(*ctx).s).write_list.is_null() {
        screen_write_make_list((*ctx).s);
    }
    (*ctx).item = screen_write_get_citem();
    (*ctx).scrolled = 0 as u_int;
    (*ctx).bg = 8 as u_int;
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_start_pane(
    mut ctx: *mut screen_write_ctx,
    mut wp: *mut window_pane,
    mut s: *mut screen,
) {
    if s.is_null() {
        s = (*wp).screen;
    }
    screen_write_init(ctx, s);
    (*ctx).wp = wp as *mut window_pane;
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(
            b"%s: size %ux%u, pane %%%u (at %u,%u)\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_start_pane\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*(*ctx).s).grid).sx,
            (*(*(*ctx).s).grid).sy,
            (*wp).id,
            (*wp).xoff,
            (*wp).yoff,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_start_callback(
    mut ctx: *mut screen_write_ctx,
    mut s: *mut screen,
    mut cb: screen_write_init_ctx_cb,
    mut arg: *mut ::core::ffi::c_void,
) {
    screen_write_init(ctx, s);
    (*ctx).init_ctx_cb = cb;
    (*ctx).arg = arg;
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(
            b"%s: size %ux%u, with callback\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_start_callback\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*(*ctx).s).grid).sx,
            (*(*(*ctx).s).grid).sy,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_start(mut ctx: *mut screen_write_ctx, mut s: *mut screen) {
    screen_write_init(ctx, s);
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(
            b"%s: size %ux%u, no pane\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_start\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*(*ctx).s).grid).sx,
            (*(*(*ctx).s).grid).sy,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_stop(mut ctx: *mut screen_write_ctx) {
    screen_write_collect_end(ctx);
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_stop\0" as *const u8 as *const ::core::ffi::c_char,
    );
    screen_write_free_citem((*ctx).item);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_reset(mut ctx: *mut screen_write_ctx) {
    let mut s: *mut screen = (*ctx).s;
    screen_reset_tabs(s);
    screen_write_scrollregion(ctx, 0 as u_int, (*(*s).grid).sy.wrapping_sub(1 as u_int));
    (*s).mode = MODE_CURSOR | MODE_WRAP;
    if options_get_number(
        global_options,
        b"extended-keys\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 2 as ::core::ffi::c_longlong
    {
        (*s).mode = (*s).mode & !EXTENDED_KEY_MODES | MODE_KEYS_EXTENDED;
    }
    screen_write_clearscreen(ctx, 8 as u_int);
    screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_putc(
    mut ctx: *mut screen_write_ctx,
    mut gcp: *const grid_cell,
    mut ch: u_char,
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
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        gcp as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    utf8_set(&raw mut gc.data, ch);
    screen_write_cell(ctx, &raw mut gc);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_strlen(
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> size_t {
    let mut ap: ::core::ffi::VaList;
    let mut msg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut ptr: *mut u_char = ::core::ptr::null_mut::<u_char>();
    let mut left: size_t = 0;
    let mut size: size_t = 0 as size_t;
    let mut more: utf8_state = UTF8_MORE;
    ap = args.clone();
    xvasprintf(&raw mut msg, fmt, ap);
    ptr = msg as *mut u_char;
    while *ptr as ::core::ffi::c_int != '\0' as i32 {
        if *ptr as ::core::ffi::c_int > 0x7f as ::core::ffi::c_int
            && utf8_open(&raw mut ud, *ptr) as ::core::ffi::c_uint
                == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ptr = ptr.offset(1);
            left = strlen(ptr as *const ::core::ffi::c_char);
            if left < (ud.size as size_t).wrapping_sub(1 as size_t) {
                break;
            }
            loop {
                more = utf8_append(&raw mut ud, *ptr);
                if !(more as ::core::ffi::c_uint
                    == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    break;
                }
                ptr = ptr.offset(1);
            }
            ptr = ptr.offset(1);
            if more as ::core::ffi::c_uint == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                size = size.wrapping_add(ud.width as size_t);
            }
        } else {
            if *ptr as ::core::ffi::c_int == '\t' as i32
                || *ptr as ::core::ffi::c_int > 0x1f as ::core::ffi::c_int
                    && (*ptr as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
            {
                size = size.wrapping_add(1);
            }
            ptr = ptr.offset(1);
        }
    }
    free(msg as *mut ::core::ffi::c_void);
    return size;
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_text(
    mut ctx: *mut screen_write_ctx,
    mut cx: u_int,
    mut width: u_int,
    mut lines: u_int,
    mut more: ::core::ffi::c_int,
    mut gcp: *const grid_cell,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> ::core::ffi::c_int {
    let mut s: *mut screen = (*ctx).s;
    let mut ap: ::core::ffi::VaList;
    let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cy: u_int = (*s).cy;
    let mut i: u_int = 0;
    let mut end: u_int = 0;
    let mut next: u_int = 0;
    let mut idx: u_int = 0 as u_int;
    let mut at: u_int = 0;
    let mut left: u_int = 0;
    let mut text: *mut utf8_data = ::core::ptr::null_mut::<utf8_data>();
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
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        gcp as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    ap = args.clone();
    xvasprintf(&raw mut tmp, fmt, ap);
    text = utf8_fromcstr(tmp);
    free(tmp as *mut ::core::ffi::c_void);
    left = cx.wrapping_add(width).wrapping_sub((*s).cx);
    loop {
        at = 0 as u_int;
        end = idx;
        while (*text.offset(end as isize)).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            if (*text.offset(end as isize)).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                && (*text.offset(end as isize)).data[0 as ::core::ffi::c_int as usize]
                    as ::core::ffi::c_int
                    == '\n' as i32
            {
                break;
            }
            if at.wrapping_add((*text.offset(end as isize)).width as u_int) > left {
                break;
            }
            at = at.wrapping_add((*text.offset(end as isize)).width as u_int);
            end = end.wrapping_add(1);
        }
        if (*text.offset(end as isize)).size as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            next = end;
        } else if (*text.offset(end as isize)).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && (*text.offset(end as isize)).data[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                == '\n' as i32
        {
            next = end.wrapping_add(1 as u_int);
        } else if (*text.offset(end as isize)).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
            && (*text.offset(end as isize)).data[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                == ' ' as i32
        {
            next = end.wrapping_add(1 as u_int);
        } else {
            i = end;
            while i > idx {
                if (*text.offset(i as isize)).size as ::core::ffi::c_int == 1 as ::core::ffi::c_int
                    && (*text.offset(i as isize)).data[0 as ::core::ffi::c_int as usize]
                        as ::core::ffi::c_int
                        == ' ' as i32
                {
                    break;
                }
                i = i.wrapping_sub(1);
            }
            if i != idx {
                next = i.wrapping_add(1 as u_int);
                end = i;
            } else {
                next = end;
            }
        }
        i = idx;
        while i < end {
            utf8_copy(&raw mut gc.data, text.offset(i as isize) as *mut utf8_data);
            screen_write_cell(ctx, &raw mut gc);
            i = i.wrapping_add(1);
        }
        idx = next;
        if (*s).cy == cy.wrapping_add(lines).wrapping_sub(1 as u_int)
            || (*text.offset(idx as isize)).size as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        {
            break;
        }
        screen_write_cursormove(
            ctx,
            cx as ::core::ffi::c_int,
            (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        left = width;
    }
    if (*s).cy == cy.wrapping_add(lines).wrapping_sub(1 as u_int)
        && (more == 0 || (*s).cx == cx.wrapping_add(width))
        || (*text.offset(idx as isize)).size as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        free(text as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    free(text as *mut ::core::ffi::c_void);
    if more == 0 || (*s).cx == cx.wrapping_add(width) {
        screen_write_cursormove(
            ctx,
            cx as ::core::ffi::c_int,
            (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_puts(
    mut ctx: *mut screen_write_ctx,
    mut gcp: *const grid_cell,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    screen_write_vnputs(
        ctx,
        -(1 as ::core::ffi::c_int) as ssize_t,
        gcp,
        fmt,
        ap,
    );
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_nputs(
    mut ctx: *mut screen_write_ctx,
    mut maxlen: ssize_t,
    mut gcp: *const grid_cell,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    screen_write_vnputs(ctx, maxlen, gcp, fmt, ap);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_vnputs(
    mut ctx: *mut screen_write_ctx,
    mut maxlen: ssize_t,
    mut gcp: *const grid_cell,
    mut fmt: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
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
    let mut ud: *mut utf8_data = &raw mut gc.data;
    let mut msg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ptr: *mut u_char = ::core::ptr::null_mut::<u_char>();
    let mut left: size_t = 0;
    let mut size: size_t = 0 as size_t;
    let mut more: utf8_state = UTF8_MORE;
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        gcp as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    xvasprintf(&raw mut msg, fmt, ap);
    ptr = msg as *mut u_char;
    while *ptr as ::core::ffi::c_int != '\0' as i32 {
        if *ptr as ::core::ffi::c_int > 0x7f as ::core::ffi::c_int
            && utf8_open(ud, *ptr) as ::core::ffi::c_uint
                == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ptr = ptr.offset(1);
            left = strlen(ptr as *const ::core::ffi::c_char);
            if left < ((*ud).size as size_t).wrapping_sub(1 as size_t) {
                break;
            }
            loop {
                more = utf8_append(ud, *ptr);
                if !(more as ::core::ffi::c_uint
                    == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    break;
                }
                ptr = ptr.offset(1);
            }
            ptr = ptr.offset(1);
            if more as ::core::ffi::c_uint != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                continue;
            }
            if maxlen > 0 as ssize_t && size.wrapping_add((*ud).width as size_t) > maxlen as size_t
            {
                while size < maxlen as size_t {
                    screen_write_putc(ctx, &raw mut gc, ' ' as i32 as u_char);
                    size = size.wrapping_add(1);
                }
                break;
            } else {
                size = size.wrapping_add((*ud).width as size_t);
                screen_write_cell(ctx, &raw mut gc);
            }
        } else {
            if maxlen > 0 as ssize_t && size.wrapping_add(1 as size_t) > maxlen as size_t {
                break;
            }
            if *ptr as ::core::ffi::c_int == '\u{1}' as i32 {
                gc.attr = (gc.attr as ::core::ffi::c_int ^ GRID_ATTR_CHARSET) as u_short;
            } else if *ptr as ::core::ffi::c_int == '\n' as i32 {
                screen_write_linefeed(ctx, 0 as ::core::ffi::c_int, 8 as u_int);
                screen_write_carriagereturn(ctx);
            } else if *ptr as ::core::ffi::c_int == '\t' as i32
                || *ptr as ::core::ffi::c_int > 0x1f as ::core::ffi::c_int
                    && (*ptr as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
            {
                size = size.wrapping_add(1);
                screen_write_putc(ctx, &raw mut gc, *ptr);
            }
            ptr = ptr.offset(1);
        }
    }
    free(msg as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_fast_copy(
    mut ctx: *mut screen_write_ctx,
    mut src: *mut screen,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut ny: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut gd: *mut grid = (*src).grid;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut sgl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
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
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    let mut xoff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut yoff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    if nx == 0 as u_int || ny == 0 as u_int {
        return;
    }
    if !wp.is_null() {
        xoff = (*wp).xoff;
        yoff = (*wp).yoff;
    }
    yy = py;
    while yy < py.wrapping_add(ny) {
        if yy >= (*gd).hsize.wrapping_add((*gd).sy) {
            break;
        }
        (*s).cx = cx;
        screen_write_initctx(
            ctx,
            &raw mut ttyctx,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        r = window_visible_ranges(
            wp,
            (xoff as u_int).wrapping_add((*s).cx) as ::core::ffi::c_int,
            (*s).cy.wrapping_add(yoff as u_int) as ::core::ffi::c_int,
            nx,
            ::core::ptr::null_mut::<visible_ranges>(),
        );
        xx = px;
        while xx < px.wrapping_add(nx) {
            gl = grid_get_line(gd, yy);
            sgl = grid_get_line((*s).grid, (*s).cy);
            if xx >= (*gl).cellsize as u_int && (*s).cx >= (*sgl).cellsize as u_int {
                break;
            }
            grid_get_cell(gd, xx, yy, &raw mut gc);
            if xx.wrapping_add(gc.data.width as u_int) > px.wrapping_add(nx) {
                break;
            }
            grid_view_set_cell((*s).grid, (*s).cx, (*s).cy, &raw mut gc);
            if window_position_is_visible(r, (xoff as u_int).wrapping_add((*s).cx)) == 0 {
                break;
            }
            ttyctx.cell = &raw mut gc;
            ttyctx.flags &= TTY_CTX_OVERLAY_SYNC | TTY_CTX_SYNC;
            tty_write(
                Some(tty_cmd_cell as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
                &raw mut ttyctx,
            );
            ttyctx.ocx = ttyctx.ocx.wrapping_add(1);
            (*s).cx = (*s).cx.wrapping_add(1);
            xx = xx.wrapping_add(1);
        }
        (*s).cy = (*s).cy.wrapping_add(1);
        yy = yy.wrapping_add(1);
    }
    (*s).cx = cx;
    (*s).cy = cy;
}
unsafe extern "C" fn screen_write_box_border_set(
    mut lines: box_lines,
    mut cell_type: ::core::ffi::c_int,
    mut gc: *mut grid_cell,
) {
    match lines as ::core::ffi::c_int {
        1 => {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            utf8_copy(&raw mut (*gc).data, tty_acs_double_borders(cell_type));
        }
        2 => {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            utf8_copy(&raw mut (*gc).data, tty_acs_heavy_borders(cell_type));
        }
        4 => {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            utf8_copy(&raw mut (*gc).data, tty_acs_rounded_borders(cell_type));
        }
        3 => {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            utf8_set(
                &raw mut (*gc).data,
                SIMPLE_BORDERS[cell_type as usize] as u_char,
            );
        }
        5 => {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
            utf8_set(
                &raw mut (*gc).data,
                PADDED_BORDERS[cell_type as usize] as u_char,
            );
        }
        0 | -1 => {
            (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
            utf8_set(
                &raw mut (*gc).data,
                CELL_BORDERS[cell_type as usize] as u_char,
            );
        }
        6 | _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_hline(
    mut ctx: *mut screen_write_ctx,
    mut nx: u_int,
    mut left: ::core::ffi::c_int,
    mut right: ::core::ffi::c_int,
    mut lines: box_lines,
    mut border_gc: *const grid_cell,
) {
    let mut s: *mut screen = (*ctx).s;
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
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut i: u_int = 0;
    cx = (*s).cx;
    cy = (*s).cy;
    if !border_gc.is_null() {
        memcpy(
            &raw mut gc as *mut ::core::ffi::c_void,
            border_gc as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    } else {
        memcpy(
            &raw mut gc as *mut ::core::ffi::c_void,
            &raw const grid_default_cell as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    }
    gc.attr = (gc.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
    if left != 0 {
        screen_write_box_border_set(lines, CELL_URD, &raw mut gc);
    } else {
        screen_write_box_border_set(lines, CELL_LR, &raw mut gc);
    }
    screen_write_cell(ctx, &raw mut gc);
    screen_write_box_border_set(lines, CELL_LR, &raw mut gc);
    i = 1 as u_int;
    while i < nx.wrapping_sub(1 as u_int) {
        screen_write_cell(ctx, &raw mut gc);
        i = i.wrapping_add(1);
    }
    if right != 0 {
        screen_write_box_border_set(lines, CELL_ULD, &raw mut gc);
    } else {
        screen_write_box_border_set(lines, CELL_LR, &raw mut gc);
    }
    screen_write_cell(ctx, &raw mut gc);
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_vline(
    mut ctx: *mut screen_write_ctx,
    mut ny: u_int,
    mut top: ::core::ffi::c_int,
    mut bottom: ::core::ffi::c_int,
    mut gcp: *const grid_cell,
) {
    let mut s: *mut screen = (*ctx).s;
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
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut i: u_int = 0;
    cx = (*s).cx;
    cy = (*s).cy;
    if !gcp.is_null() {
        memcpy(
            &raw mut gc as *mut ::core::ffi::c_void,
            gcp as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    } else {
        memcpy(
            &raw mut gc as *mut ::core::ffi::c_void,
            &raw const grid_default_cell as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    }
    gc.attr = (gc.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
    screen_write_putc(
        ctx,
        &raw mut gc,
        (if top != 0 { 'w' as i32 } else { 'x' as i32 }) as u_char,
    );
    i = 1 as u_int;
    while i < ny.wrapping_sub(1 as u_int) {
        screen_write_set_cursor(
            ctx,
            cx as ::core::ffi::c_int,
            cy.wrapping_add(i) as ::core::ffi::c_int,
        );
        screen_write_putc(ctx, &raw mut gc, 'x' as i32 as u_char);
        i = i.wrapping_add(1);
    }
    screen_write_set_cursor(
        ctx,
        cx as ::core::ffi::c_int,
        cy.wrapping_add(ny).wrapping_sub(1 as u_int) as ::core::ffi::c_int,
    );
    screen_write_putc(
        ctx,
        &raw mut gc,
        (if bottom != 0 { 'v' as i32 } else { 'x' as i32 }) as u_char,
    );
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_menu(
    mut ctx: *mut screen_write_ctx,
    mut menu: *mut menu,
    mut choice: ::core::ffi::c_int,
    mut lines: box_lines,
    mut menu_gc: *const grid_cell,
    mut border_gc: *const grid_cell,
    mut choice_gc: *const grid_cell,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut default_gc: grid_cell = grid_cell {
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
    let mut gc: *const grid_cell = &raw mut default_gc;
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut width: u_int = (*menu).width;
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    cx = (*s).cx;
    cy = (*s).cy;
    memcpy(
        &raw mut default_gc as *mut ::core::ffi::c_void,
        menu_gc as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    screen_write_box(
        ctx,
        (*menu).width.wrapping_add(4 as u_int),
        (*menu).count.wrapping_add(2 as u_int),
        lines,
        border_gc,
        (*menu).title,
    );
    i = 0 as u_int;
    while i < (*menu).count {
        name = (*(*menu).items.offset(i as isize)).name;
        if name.is_null() {
            screen_write_cursormove(
                ctx,
                cx as ::core::ffi::c_int,
                cy.wrapping_add(1 as u_int).wrapping_add(i) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            screen_write_hline(
                ctx,
                width.wrapping_add(4 as u_int),
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
                lines,
                border_gc,
            );
        } else {
            if choice >= 0 as ::core::ffi::c_int
                && i == choice as u_int
                && *name as ::core::ffi::c_int != '-' as i32
            {
                gc = choice_gc;
            }
            screen_write_cursormove(
                ctx,
                cx.wrapping_add(1 as u_int) as ::core::ffi::c_int,
                cy.wrapping_add(1 as u_int).wrapping_add(i) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            j = 0 as u_int;
            while j < width.wrapping_add(2 as u_int) {
                screen_write_putc(ctx, gc, ' ' as i32 as u_char);
                j = j.wrapping_add(1);
            }
            screen_write_cursormove(
                ctx,
                cx.wrapping_add(2 as u_int) as ::core::ffi::c_int,
                cy.wrapping_add(1 as u_int).wrapping_add(i) as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            if *name as ::core::ffi::c_int == '-' as i32 {
                default_gc.attr =
                    (default_gc.attr as ::core::ffi::c_int | GRID_ATTR_DIM) as u_short;
                format_draw(
                    ctx,
                    gc,
                    width,
                    name.offset(1 as ::core::ffi::c_int as isize),
                    ::core::ptr::null_mut::<style_ranges>(),
                    0 as ::core::ffi::c_int,
                );
                default_gc.attr =
                    (default_gc.attr as ::core::ffi::c_int & !GRID_ATTR_DIM) as u_short;
            } else {
                format_draw(
                    ctx,
                    gc,
                    width,
                    name,
                    ::core::ptr::null_mut::<style_ranges>(),
                    0 as ::core::ffi::c_int,
                );
                gc = &raw mut default_gc;
            }
        }
        i = i.wrapping_add(1);
    }
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_box(
    mut ctx: *mut screen_write_ctx,
    mut nx: u_int,
    mut ny: u_int,
    mut lines: box_lines,
    mut gcp: *const grid_cell,
    mut title: *const ::core::ffi::c_char,
) {
    let mut s: *mut screen = (*ctx).s;
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
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut i: u_int = 0;
    cx = (*s).cx;
    cy = (*s).cy;
    if !gcp.is_null() {
        memcpy(
            &raw mut gc as *mut ::core::ffi::c_void,
            gcp as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    } else {
        memcpy(
            &raw mut gc as *mut ::core::ffi::c_void,
            &raw const grid_default_cell as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    }
    gc.attr = (gc.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
    gc.flags = (gc.flags as ::core::ffi::c_int | GRID_FLAG_NOPALETTE) as u_char;
    screen_write_box_border_set(lines, CELL_RD, &raw mut gc);
    screen_write_cell(ctx, &raw mut gc);
    screen_write_box_border_set(lines, CELL_LR, &raw mut gc);
    i = 1 as u_int;
    while i < nx.wrapping_sub(1 as u_int) {
        screen_write_cell(ctx, &raw mut gc);
        i = i.wrapping_add(1);
    }
    screen_write_box_border_set(lines, CELL_LD, &raw mut gc);
    screen_write_cell(ctx, &raw mut gc);
    screen_write_set_cursor(
        ctx,
        cx as ::core::ffi::c_int,
        cy.wrapping_add(ny).wrapping_sub(1 as u_int) as ::core::ffi::c_int,
    );
    screen_write_box_border_set(lines, CELL_RU, &raw mut gc);
    screen_write_cell(ctx, &raw mut gc);
    screen_write_box_border_set(lines, CELL_LR, &raw mut gc);
    i = 1 as u_int;
    while i < nx.wrapping_sub(1 as u_int) {
        screen_write_cell(ctx, &raw mut gc);
        i = i.wrapping_add(1);
    }
    screen_write_box_border_set(lines, CELL_LU, &raw mut gc);
    screen_write_cell(ctx, &raw mut gc);
    screen_write_box_border_set(lines, CELL_UD, &raw mut gc);
    i = 1 as u_int;
    while i < ny.wrapping_sub(1 as u_int) {
        screen_write_set_cursor(
            ctx,
            cx as ::core::ffi::c_int,
            cy.wrapping_add(i) as ::core::ffi::c_int,
        );
        screen_write_cell(ctx, &raw mut gc);
        screen_write_set_cursor(
            ctx,
            cx.wrapping_add(nx).wrapping_sub(1 as u_int) as ::core::ffi::c_int,
            cy.wrapping_add(i) as ::core::ffi::c_int,
        );
        screen_write_cell(ctx, &raw mut gc);
        i = i.wrapping_add(1);
    }
    if !title.is_null() {
        gc.attr = (gc.attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
        screen_write_cursormove(
            ctx,
            cx.wrapping_add(2 as u_int) as ::core::ffi::c_int,
            cy as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        format_draw(
            ctx,
            &raw mut gc,
            nx.wrapping_sub(4 as u_int),
            title,
            ::core::ptr::null_mut::<style_ranges>(),
            0 as ::core::ffi::c_int,
        );
    }
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_preview(
    mut ctx: *mut screen_write_ctx,
    mut src: *mut screen,
    mut nx: u_int,
    mut ny: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
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
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    cx = (*s).cx;
    cy = (*s).cy;
    if (*src).mode & MODE_CURSOR != 0 {
        px = (*src).cx;
        if px < nx.wrapping_div(3 as u_int) {
            px = 0 as u_int;
        } else {
            px = px.wrapping_sub(nx.wrapping_div(3 as u_int));
        }
        if px.wrapping_add(nx) > (*(*src).grid).sx {
            if nx > (*(*src).grid).sx {
                px = 0 as u_int;
            } else {
                px = (*(*src).grid).sx.wrapping_sub(nx);
            }
        }
        py = (*src).cy;
        if py < ny.wrapping_div(3 as u_int) {
            py = 0 as u_int;
        } else {
            py = py.wrapping_sub(ny.wrapping_div(3 as u_int));
        }
        if py.wrapping_add(ny) > (*(*src).grid).sy {
            if ny > (*(*src).grid).sy {
                py = 0 as u_int;
            } else {
                py = (*(*src).grid).sy.wrapping_sub(ny);
            }
        }
    } else {
        px = 0 as u_int;
        py = 0 as u_int;
    }
    screen_write_fast_copy(ctx, src, px, (*(*src).grid).hsize.wrapping_add(py), nx, ny);
    if (*src).mode & MODE_CURSOR != 0 {
        grid_view_get_cell((*src).grid, (*src).cx, (*src).cy, &raw mut gc);
        gc.attr = (gc.attr as ::core::ffi::c_int | GRID_ATTR_REVERSE) as u_short;
        screen_write_set_cursor(
            ctx,
            cx.wrapping_add((*src).cx.wrapping_sub(px)) as ::core::ffi::c_int,
            cy.wrapping_add((*src).cy.wrapping_sub(py)) as ::core::ffi::c_int,
        );
        screen_write_cell(ctx, &raw mut gc);
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_mode_set(
    mut ctx: *mut screen_write_ctx,
    mut mode: ::core::ffi::c_int,
) {
    let mut s: *mut screen = (*ctx).s;
    (*s).mode |= mode;
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_mode_set\0" as *const u8 as *const ::core::ffi::c_char,
            screen_mode_to_string(mode),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_mode_clear(
    mut ctx: *mut screen_write_ctx,
    mut mode: ::core::ffi::c_int,
) {
    let mut s: *mut screen = (*ctx).s;
    (*s).mode &= !mode;
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_mode_clear\0" as *const u8 as *const ::core::ffi::c_char,
            screen_mode_to_string(mode),
        );
    }
}
unsafe extern "C" fn screen_write_sync_callback(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = arg as *mut window_pane;
    log_debug(
        b"%s: %%%u sync timer expired\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_sync_callback\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
    event_del(&raw mut (*wp).sync_timer);
    if (*wp).base.mode & MODE_SYNC != 0 {
        (*wp).base.mode &= !MODE_SYNC;
        screen_write_flush_dirty(wp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_start_sync(mut wp: *mut window_pane) {
    let mut tv: timeval = timeval {
        tv_sec: 1 as __time_t,
        tv_usec: 0 as __suseconds_t,
    };
    if wp.is_null() {
        return;
    }
    (*wp).base.mode |= MODE_SYNC;
    if event_initialized(&raw mut (*wp).sync_timer) == 0 {
        event_set(
            &raw mut (*wp).sync_timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            Some(
                screen_write_sync_callback
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            wp as *mut ::core::ffi::c_void,
        );
    }
    event_add(&raw mut (*wp).sync_timer, &raw mut tv);
    log_debug(
        b"%s: %%%u started sync mode\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_start_sync\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_stop_sync(mut wp: *mut window_pane) {
    if wp.is_null() || !(*wp).base.mode & MODE_SYNC != 0 {
        return;
    }
    if event_initialized(&raw mut (*wp).sync_timer) != 0 {
        event_del(&raw mut (*wp).sync_timer);
    }
    (*wp).base.mode &= !MODE_SYNC;
    screen_write_flush_dirty(wp);
    log_debug(
        b"%s: %%%u stopped sync mode\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_stop_sync\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_end_sync(mut ctx: *mut screen_write_ctx) {
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    if wp.is_null() {
        return;
    }
    if (*wp).base.mode & MODE_SYNC != 0 {
        screen_write_collect_flush(
            ctx,
            0 as ::core::ffi::c_int,
            b"screen_write_end_sync\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    screen_write_stop_sync(wp);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_cursorup(mut ctx: *mut screen_write_ctx, mut ny: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    if ny == 0 as u_int {
        ny = 1 as u_int;
    }
    if cy < (*s).rupper {
        if ny > cy {
            ny = cy;
        }
    } else if ny > cy.wrapping_sub((*s).rupper) {
        ny = cy.wrapping_sub((*s).rupper);
    }
    if cx == (*(*s).grid).sx {
        cx = cx.wrapping_sub(1);
    }
    cy = cy.wrapping_sub(ny);
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_cursordown(mut ctx: *mut screen_write_ctx, mut ny: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    if ny == 0 as u_int {
        ny = 1 as u_int;
    }
    if cy > (*s).rlower {
        if ny > (*(*s).grid).sy.wrapping_sub(1 as u_int).wrapping_sub(cy) {
            ny = (*(*s).grid).sy.wrapping_sub(1 as u_int).wrapping_sub(cy);
        }
    } else if ny > (*s).rlower.wrapping_sub(cy) {
        ny = (*s).rlower.wrapping_sub(cy);
    }
    if cx == (*(*s).grid).sx {
        cx = cx.wrapping_sub(1);
    } else if ny == 0 as u_int {
        return;
    }
    cy = cy.wrapping_add(ny);
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_cursorright(mut ctx: *mut screen_write_ctx, mut nx: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    if nx == 0 as u_int {
        nx = 1 as u_int;
    }
    if nx > (*(*s).grid).sx.wrapping_sub(1 as u_int).wrapping_sub(cx) {
        nx = (*(*s).grid).sx.wrapping_sub(1 as u_int).wrapping_sub(cx);
    }
    if nx == 0 as u_int {
        return;
    }
    cx = cx.wrapping_add(nx);
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_cursorleft(mut ctx: *mut screen_write_ctx, mut nx: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    if nx == 0 as u_int {
        nx = 1 as u_int;
    }
    if nx > cx {
        nx = cx;
    }
    if nx == 0 as u_int {
        return;
    }
    cx = cx.wrapping_sub(nx);
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_backspace(mut ctx: *mut screen_write_ctx) {
    let mut s: *mut screen = (*ctx).s;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    if cx == 0 as u_int {
        if cy == 0 as u_int {
            return;
        }
        gl = grid_get_line(
            (*s).grid,
            (*(*s).grid).hsize.wrapping_add(cy).wrapping_sub(1 as u_int),
        );
        if (*gl).flags as ::core::ffi::c_int & GRID_LINE_WRAPPED != 0 {
            cy = cy.wrapping_sub(1);
            cx = (*(*s).grid).sx.wrapping_sub(1 as u_int);
        }
    } else {
        cx = cx.wrapping_sub(1);
    }
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
}
unsafe extern "C" fn screen_write_cell_is_single(mut gc: *const grid_cell) -> ::core::ffi::c_int {
    if (*gc).data.width as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if (*gc).data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if (*(&raw const (*gc).data.data as *const u_char) as ::core::ffi::c_int)
        < 0x20 as ::core::ffi::c_int
        || *(&raw const (*gc).data.data as *const u_char) as ::core::ffi::c_int
            == 0x7f as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_CLEARED != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn screen_write_redraw_line(
    mut ctx: *mut screen_write_ctx,
    mut ttyctx: *mut tty_ctx,
    mut yy: u_int,
) {
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut s: *mut screen = (*ctx).s;
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
    let mut ngc: grid_cell = grid_cell {
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
    let mut sx: u_int = (*(*s).grid).sx;
    let mut cx: u_int = 0;
    let mut i: u_int = 0;
    let mut xoff: ::core::ffi::c_int = (*wp).xoff;
    let mut yoff: ::core::ffi::c_int = (*wp).yoff;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    r = window_visible_ranges(
        wp,
        xoff,
        (yoff as u_int).wrapping_add(yy) as ::core::ffi::c_int,
        sx,
        ::core::ptr::null_mut::<visible_ranges>(),
    );
    i = 0 as u_int;
    while i < (*r).used {
        ri = (*r).ranges.offset(i as isize) as *mut visible_range;
        if !((*ri).nx == 0 as u_int) {
            cx = (*ri).px.wrapping_sub(xoff as u_int);
            if !(cx >= sx) {
                if cx.wrapping_add((*ri).nx) > sx {
                    (*ttyctx).c2rust_unnamed.n = sx.wrapping_sub(cx);
                } else {
                    (*ttyctx).c2rust_unnamed.n = (*ri).nx;
                }
                if !((*ttyctx).c2rust_unnamed.n == 0 as u_int) {
                    (*ttyctx).ocx = cx;
                    (*ttyctx).ocy = yy;
                    if (*ttyctx).c2rust_unnamed.n != 1 as u_int {
                        tty_write(
                            Some(
                                tty_cmd_redrawline
                                    as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> (),
                            ),
                            ttyctx,
                        );
                    } else {
                        grid_view_get_cell((*s).grid, cx, yy, &raw mut gc);
                        if screen_write_cell_is_single(&raw mut gc) == 0 {
                            tty_write(
                                Some(
                                    tty_cmd_redrawline
                                        as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> (),
                                ),
                                ttyctx,
                            );
                        } else {
                            if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_SELECTED != 0 {
                                (*ttyctx).cell = &raw mut gc;
                            } else {
                                screen_select_cell(s, &raw mut ngc, &raw mut gc);
                                (*ttyctx).cell = &raw mut ngc;
                            }
                            tty_write(
                                Some(
                                    tty_cmd_cell
                                        as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> (),
                                ),
                                ttyctx,
                            );
                        }
                    }
                }
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn screen_write_flush_dirty(mut wp: *mut window_pane) {
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
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut s: *mut screen = &raw mut (*wp).base;
    let mut y: u_int = 0;
    let mut sy: u_int = (*(*s).grid).sy;
    let mut lines: u_int = 0 as u_int;
    if (*wp).sync_dirty.is_null() {
        return;
    }
    screen_write_start_pane(&raw mut ctx, wp, s);
    screen_write_initctx(
        &raw mut ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    y = 0 as u_int;
    while y < sy {
        if *(*wp)
            .sync_dirty
            .offset((y >> 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << (y & 0x7 as u_int)
            != 0
        {
            screen_write_redraw_line(&raw mut ctx, &raw mut ttyctx, y);
            lines = lines.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    log_debug(
        b"%s: %%%u had %u dirty lines\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_flush_dirty\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        lines,
    );
    screen_write_stop(&raw mut ctx);
    screen_write_clear_dirty(wp);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clear_dirty(mut wp: *mut window_pane) {
    if !wp.is_null() && !(*wp).sync_dirty.is_null() {
        free((*wp).sync_dirty as *mut ::core::ffi::c_void);
        (*wp).sync_dirty = ::core::ptr::null_mut::<bitstr_t>();
        (*wp).sync_dirty_size = 0 as u_int;
    }
}
unsafe extern "C" fn screen_write_redraw_pane(
    mut ctx: *mut screen_write_ctx,
    mut ttyctx: *mut tty_ctx,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut yy: u_int = 0;
    yy = 0 as u_int;
    while yy < (*(*s).grid).sy {
        screen_write_redraw_line(ctx, ttyctx, yy);
        yy = yy.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_alignmenttest(mut ctx: *mut screen_write_ctx) {
    let mut s: *mut screen = (*ctx).s;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
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
    let mut xx: u_int = 0;
    let mut yy: u_int = 0;
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    utf8_set(&raw mut gc.data, 'E' as i32 as u_char);
    yy = 0 as u_int;
    while yy < (*(*s).grid).sy {
        xx = 0 as u_int;
        while xx < (*(*s).grid).sx {
            grid_view_set_cell((*s).grid, xx, yy, &raw mut gc);
            xx = xx.wrapping_add(1);
        }
        yy = yy.wrapping_add(1);
    }
    screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    (*s).rupper = 0 as u_int;
    (*s).rlower = (*(*s).grid).sy.wrapping_sub(1 as u_int);
    screen_write_collect_clear(ctx, 0 as u_int, (*(*s).grid).sy.wrapping_sub(1 as u_int));
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    if screen_write_should_draw_lines(ctx, 0 as u_int, (*(*s).grid).sy) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
        tty_write(
            Some(tty_cmd_alignmenttest as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    screen_write_redraw_pane(ctx, &raw mut ttyctx);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_insertcharacter(
    mut ctx: *mut screen_write_ctx,
    mut nx: u_int,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    if nx == 0 as u_int {
        nx = 1 as u_int;
    }
    if nx > (*(*s).grid).sx.wrapping_sub((*s).cx) {
        nx = (*(*s).grid).sx.wrapping_sub((*s).cx);
    }
    if nx == 0 as u_int {
        return;
    }
    if (*s).cx > (*(*s).grid).sx.wrapping_sub(1 as u_int) {
        return;
    }
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    grid_view_insert_cells((*s).grid, (*s).cx, (*s).cy, nx, bg);
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_insertcharacter\0" as *const u8 as *const ::core::ffi::c_char,
    );
    ttyctx.c2rust_unnamed.n = nx;
    if screen_write_should_draw_line(ctx, (*s).cy) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
        tty_write(
            Some(tty_cmd_insertcharacter as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    screen_write_redraw_line(ctx, &raw mut ttyctx, (*s).cy);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_deletecharacter(
    mut ctx: *mut screen_write_ctx,
    mut nx: u_int,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    if nx == 0 as u_int {
        nx = 1 as u_int;
    }
    if nx > (*(*s).grid).sx.wrapping_sub((*s).cx) {
        nx = (*(*s).grid).sx.wrapping_sub((*s).cx);
    }
    if nx == 0 as u_int {
        return;
    }
    if (*s).cx > (*(*s).grid).sx.wrapping_sub(1 as u_int) {
        return;
    }
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    grid_view_delete_cells((*s).grid, (*s).cx, (*s).cy, nx, bg);
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_deletecharacter\0" as *const u8 as *const ::core::ffi::c_char,
    );
    ttyctx.c2rust_unnamed.n = nx;
    if screen_write_should_draw_line(ctx, (*s).cy) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
        tty_write(
            Some(tty_cmd_deletecharacter as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    screen_write_redraw_line(ctx, &raw mut ttyctx, (*s).cy);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clearcharacter(
    mut ctx: *mut screen_write_ctx,
    mut nx: u_int,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    if nx == 0 as u_int {
        nx = 1 as u_int;
    }
    if nx > (*(*s).grid).sx.wrapping_sub((*s).cx) {
        nx = (*(*s).grid).sx.wrapping_sub((*s).cx);
    }
    if nx == 0 as u_int {
        return;
    }
    if (*s).cx > (*(*s).grid).sx.wrapping_sub(1 as u_int) {
        return;
    }
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    grid_view_clear((*s).grid, (*s).cx, (*s).cy, nx, 1 as u_int, bg);
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_clearcharacter\0" as *const u8 as *const ::core::ffi::c_char,
    );
    ttyctx.c2rust_unnamed.n = nx;
    if screen_write_should_draw_line(ctx, (*s).cy) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
        tty_write(
            Some(tty_cmd_clearcharacter as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    screen_write_redraw_line(ctx, &raw mut ttyctx, (*s).cy);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_insertline(
    mut ctx: *mut screen_write_ctx,
    mut ny: u_int,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut gd: *mut grid = (*s).grid;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut sy: u_int = (*(*s).grid).sy;
    if ny == 0 as u_int {
        ny = 1 as u_int;
    }
    if (*s).cy < (*s).rupper || (*s).cy > (*s).rlower {
        if ny > sy.wrapping_sub((*s).cy) {
            ny = sy.wrapping_sub((*s).cy);
        }
        if ny == 0 as u_int {
            return;
        }
        screen_write_initctx(
            ctx,
            &raw mut ttyctx,
            1 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
        ttyctx.bg = bg;
        grid_view_insert_lines(gd, (*s).cy, ny, bg);
        screen_write_collect_flush(
            ctx,
            0 as ::core::ffi::c_int,
            b"screen_write_insertline\0" as *const u8 as *const ::core::ffi::c_char,
        );
        ttyctx.c2rust_unnamed.n = ny;
        if screen_write_should_draw_lines(ctx, (*s).cy, sy.wrapping_sub((*s).cy)) == 0 {
            return;
        }
        if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
            tty_write(
                Some(tty_cmd_insertline as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
                &raw mut ttyctx,
            );
            return;
        }
        screen_write_redraw_pane(ctx, &raw mut ttyctx);
        return;
    }
    if ny > (*s).rlower.wrapping_add(1 as u_int).wrapping_sub((*s).cy) {
        ny = (*s).rlower.wrapping_add(1 as u_int).wrapping_sub((*s).cy);
    }
    if ny == 0 as u_int {
        return;
    }
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    if (*s).cy < (*s).rupper || (*s).cy > (*s).rlower {
        grid_view_insert_lines(gd, (*s).cy, ny, bg);
    } else {
        grid_view_insert_lines_region(gd, (*s).rlower, (*s).cy, ny, bg);
    }
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_insertline\0" as *const u8 as *const ::core::ffi::c_char,
    );
    ttyctx.c2rust_unnamed.n = ny;
    if screen_write_should_draw_lines(
        ctx,
        (*s).cy,
        (*s).rlower.wrapping_add(1 as u_int).wrapping_sub((*s).cy),
    ) == 0
    {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
        tty_write(
            Some(tty_cmd_insertline as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    screen_write_redraw_pane(ctx, &raw mut ttyctx);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_deleteline(
    mut ctx: *mut screen_write_ctx,
    mut ny: u_int,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut gd: *mut grid = (*s).grid;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut sy: u_int = (*(*s).grid).sy;
    let mut ry: u_int = 0;
    if ny == 0 as u_int {
        ny = 1 as u_int;
    }
    if (*s).cy < (*s).rupper || (*s).cy > (*s).rlower {
        if ny > sy.wrapping_sub((*s).cy) {
            ny = sy.wrapping_sub((*s).cy);
        }
        if ny == 0 as u_int {
            return;
        }
        screen_write_initctx(
            ctx,
            &raw mut ttyctx,
            1 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
        ttyctx.bg = bg;
        grid_view_delete_lines(gd, (*s).cy, ny, bg);
        screen_write_collect_flush(
            ctx,
            0 as ::core::ffi::c_int,
            b"screen_write_deleteline\0" as *const u8 as *const ::core::ffi::c_char,
        );
        ttyctx.c2rust_unnamed.n = ny;
        ry = (*s)
            .rlower
            .wrapping_add(1 as u_int)
            .wrapping_sub((*s).rupper);
        if screen_write_should_draw_lines(ctx, (*s).rupper, ry) == 0 {
            return;
        }
        if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
            tty_write(
                Some(tty_cmd_deleteline as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
                &raw mut ttyctx,
            );
            return;
        }
        screen_write_redraw_pane(ctx, &raw mut ttyctx);
        return;
    }
    ry = (*s).rlower.wrapping_add(1 as u_int).wrapping_sub((*s).cy);
    if ny > ry {
        ny = ry;
    }
    if ny == 0 as u_int {
        return;
    }
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    if (*s).cy < (*s).rupper || (*s).cy > (*s).rlower {
        grid_view_delete_lines(gd, (*s).cy, ny, bg);
    } else {
        grid_view_delete_lines_region(gd, (*s).rlower, (*s).cy, ny, bg);
    }
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_deleteline\0" as *const u8 as *const ::core::ffi::c_char,
    );
    ttyctx.c2rust_unnamed.n = ny;
    if screen_write_should_draw_lines(ctx, (*s).cy, ry) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
        tty_write(
            Some(tty_cmd_deleteline as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    screen_write_redraw_pane(ctx, &raw mut ttyctx);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clearline(mut ctx: *mut screen_write_ctx, mut bg: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut sx: u_int = (*(*s).grid).sx;
    let mut ci: *mut screen_write_citem = (*ctx).item;
    let mut od: osc133_data = osc133_data {
        prompt_col: 0,
        cmd_col: 0,
        out_start_col: 0,
        out_end_col: 0,
        exit_status: 0,
    };
    let mut flags: u_int = 0;
    gl = grid_get_line((*s).grid, (*(*s).grid).hsize.wrapping_add((*s).cy));
    if (*gl).cellsize as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        && (bg == 8 as u_int || bg == 9 as u_int)
    {
        return;
    }
    flags = ((*gl).flags as ::core::ffi::c_int & GRID_LINE_OSC133_FLAGS) as u_int;
    memcpy(
        &raw mut od as *mut ::core::ffi::c_void,
        &raw mut (*gl).osc133_data as *const ::core::ffi::c_void,
        ::core::mem::size_of::<osc133_data>() as size_t,
    );
    grid_view_clear((*s).grid, 0 as u_int, (*s).cy, sx, 1 as u_int, bg);
    gl = grid_get_line((*s).grid, (*(*s).grid).hsize.wrapping_add((*s).cy));
    (*gl).flags = ((*gl).flags as u_int | flags) as u_short;
    memcpy(
        &raw mut (*gl).osc133_data as *mut ::core::ffi::c_void,
        &raw mut od as *const ::core::ffi::c_void,
        ::core::mem::size_of::<osc133_data>() as size_t,
    );
    screen_write_collect_clear(ctx, (*s).cy, 1 as u_int);
    (*ci).x = 0 as u_int;
    (*ci).used = sx;
    (*ci).type_0 = CLEAR;
    (*ci).bg = bg;
    (*ci).entry.tqe_next = ::core::ptr::null_mut::<screen_write_citem>();
    (*ci).entry.tqe_prev = (*(*(*ctx).s).write_list.offset((*s).cy as isize))
        .items
        .tqh_last;
    let ref mut fresh9 = *(*(*(*ctx).s).write_list.offset((*s).cy as isize))
        .items
        .tqh_last;
    *fresh9 = ci;
    let ref mut fresh10 = (*(*(*ctx).s).write_list.offset((*s).cy as isize))
        .items
        .tqh_last;
    *fresh10 = &raw mut (*ci).entry.tqe_next;
    (*ctx).item = screen_write_get_citem();
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clearendofline(
    mut ctx: *mut screen_write_ctx,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut sx: u_int = (*(*s).grid).sx;
    let mut ci: *mut screen_write_citem = (*ctx).item;
    if (*s).cx == 0 as u_int {
        screen_write_clearline(ctx, bg);
        return;
    }
    gl = grid_get_line((*s).grid, (*(*s).grid).hsize.wrapping_add((*s).cy));
    if (*s).cx > sx.wrapping_sub(1 as u_int)
        || (*s).cx >= (*gl).cellsize as u_int && (bg == 8 as u_int || bg == 9 as u_int)
    {
        return;
    }
    grid_view_clear(
        (*s).grid,
        (*s).cx,
        (*s).cy,
        sx.wrapping_sub((*s).cx),
        1 as u_int,
        bg,
    );
    (*ci).x = (*s).cx;
    (*ci).used = sx.wrapping_sub((*s).cx);
    (*ci).type_0 = CLEAR;
    (*ci).bg = bg;
    screen_write_collect_insert(ctx, ci);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clearstartofline(
    mut ctx: *mut screen_write_ctx,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut sx: u_int = (*(*s).grid).sx;
    let mut ci: *mut screen_write_citem = (*ctx).item;
    if (*s).cx >= sx.wrapping_sub(1 as u_int) {
        screen_write_clearline(ctx, bg);
        return;
    }
    if (*s).cx > sx.wrapping_sub(1 as u_int) {
        grid_view_clear((*s).grid, 0 as u_int, (*s).cy, sx, 1 as u_int, bg);
    } else {
        grid_view_clear(
            (*s).grid,
            0 as u_int,
            (*s).cy,
            (*s).cx.wrapping_add(1 as u_int),
            1 as u_int,
            bg,
        );
    }
    (*ci).x = 0 as u_int;
    (*ci).used = (*s).cx.wrapping_add(1 as u_int);
    (*ci).type_0 = CLEAR;
    (*ci).bg = bg;
    screen_write_collect_insert(ctx, ci);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_cursormove(
    mut ctx: *mut screen_write_ctx,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
    mut origin: ::core::ffi::c_int,
) {
    let mut s: *mut screen = (*ctx).s;
    if origin != 0 && py != -(1 as ::core::ffi::c_int) && (*s).mode & MODE_ORIGIN != 0 {
        if py as u_int > (*s).rlower.wrapping_sub((*s).rupper) {
            py = (*s).rlower as ::core::ffi::c_int;
        } else {
            py =
                (py as u_int).wrapping_add((*s).rupper) as ::core::ffi::c_int as ::core::ffi::c_int;
        }
    }
    if px != -(1 as ::core::ffi::c_int) && px as u_int > (*(*s).grid).sx.wrapping_sub(1 as u_int) {
        px = (*(*s).grid).sx.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    }
    if py != -(1 as ::core::ffi::c_int) && py as u_int > (*(*s).grid).sy.wrapping_sub(1 as u_int) {
        py = (*(*s).grid).sy.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: from %u,%u to %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_cursormove\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).cx,
        (*s).cy,
        px,
        py,
    );
    screen_write_set_cursor(ctx, px, py);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_reverseindex(mut ctx: *mut screen_write_ctx, mut bg: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut ry: u_int = 0;
    if (*s).cy != (*s).rupper {
        if (*s).cy > 0 as u_int {
            screen_write_set_cursor(
                ctx,
                -(1 as ::core::ffi::c_int),
                (*s).cy.wrapping_sub(1 as u_int) as ::core::ffi::c_int,
            );
        }
        return;
    }
    grid_view_scroll_region_down((*s).grid, (*s).rupper, (*s).rlower, bg);
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_reverseindex\0" as *const u8 as *const ::core::ffi::c_char,
    );
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    ry = (*s)
        .rlower
        .wrapping_add(1 as u_int)
        .wrapping_sub((*s).rupper);
    if screen_write_should_draw_lines(ctx, (*s).rupper, ry) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
        tty_write(
            Some(tty_cmd_reverseindex as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    screen_write_redraw_pane(ctx, &raw mut ttyctx);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_scrollregion(
    mut ctx: *mut screen_write_ctx,
    mut rupper: u_int,
    mut rlower: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    if rupper > (*(*s).grid).sy.wrapping_sub(1 as u_int) {
        rupper = (*(*s).grid).sy.wrapping_sub(1 as u_int);
    }
    if rlower > (*(*s).grid).sy.wrapping_sub(1 as u_int) {
        rlower = (*(*s).grid).sy.wrapping_sub(1 as u_int);
    }
    if rupper >= rlower {
        return;
    }
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_scrollregion\0" as *const u8 as *const ::core::ffi::c_char,
    );
    screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    (*s).rupper = rupper;
    (*s).rlower = rlower;
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_linefeed(
    mut ctx: *mut screen_write_ctx,
    mut wrapped: ::core::ffi::c_int,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut gd: *mut grid = (*s).grid;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut rupper: u_int = (*s).rupper;
    let mut rlower: u_int = (*s).rlower;
    gl = grid_get_line(gd, (*gd).hsize.wrapping_add((*s).cy));
    if wrapped != 0 {
        (*gl).flags = ((*gl).flags as ::core::ffi::c_int | GRID_LINE_WRAPPED) as u_short;
    }
    log_debug(
        b"%s: at %u,%u (region %u-%u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_linefeed\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).cx,
        (*s).cy,
        rupper,
        rlower,
    );
    if bg != (*ctx).bg {
        screen_write_collect_flush(
            ctx,
            1 as ::core::ffi::c_int,
            b"screen_write_linefeed\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*ctx).bg = bg;
    }
    if (*s).cy != (*s).rlower {
        if (*s).cy < (*(*s).grid).sy.wrapping_sub(1 as u_int) {
            screen_write_set_cursor(
                ctx,
                -(1 as ::core::ffi::c_int),
                (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
            );
        }
        return;
    }
    grid_view_scroll_region_up(gd, (*s).rupper, (*s).rlower, bg);
    screen_write_collect_scroll(ctx, bg);
    (*ctx).scrolled = (*ctx).scrolled.wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_scrollup(
    mut ctx: *mut screen_write_ctx,
    mut lines: u_int,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut gd: *mut grid = (*s).grid;
    let mut i: u_int = 0;
    if lines == 0 as u_int {
        lines = 1 as u_int;
    } else if lines
        > (*s)
            .rlower
            .wrapping_sub((*s).rupper)
            .wrapping_add(1 as u_int)
    {
        lines = (*s)
            .rlower
            .wrapping_sub((*s).rupper)
            .wrapping_add(1 as u_int);
    }
    if bg != (*ctx).bg {
        screen_write_collect_flush(
            ctx,
            1 as ::core::ffi::c_int,
            b"screen_write_scrollup\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*ctx).bg = bg;
    }
    i = 0 as u_int;
    while i < lines {
        grid_view_scroll_region_up(gd, (*s).rupper, (*s).rlower, bg);
        screen_write_collect_scroll(ctx, bg);
        i = i.wrapping_add(1);
    }
    (*ctx).scrolled = (*ctx).scrolled.wrapping_add(lines);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_scrolldown(
    mut ctx: *mut screen_write_ctx,
    mut lines: u_int,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut gd: *mut grid = (*s).grid;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut i: u_int = 0;
    let mut ry: u_int = 0;
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    if lines == 0 as u_int {
        lines = 1 as u_int;
    } else if lines
        > (*s)
            .rlower
            .wrapping_sub((*s).rupper)
            .wrapping_add(1 as u_int)
    {
        lines = (*s)
            .rlower
            .wrapping_sub((*s).rupper)
            .wrapping_add(1 as u_int);
    }
    i = 0 as u_int;
    while i < lines {
        grid_view_scroll_region_down(gd, (*s).rupper, (*s).rlower, bg);
        i = i.wrapping_add(1);
    }
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_scrolldown\0" as *const u8 as *const ::core::ffi::c_char,
    );
    ttyctx.c2rust_unnamed.n = lines;
    ry = (*s)
        .rlower
        .wrapping_add(1 as u_int)
        .wrapping_sub((*s).rupper);
    if screen_write_should_draw_lines(ctx, (*s).rupper, ry) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 || (*ctx).wp.is_null() {
        tty_write(
            Some(tty_cmd_scrolldown as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    screen_write_redraw_pane(ctx, &raw mut ttyctx);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_carriagereturn(mut ctx: *mut screen_write_ctx) {
    screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int));
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clearendofscreen(
    mut ctx: *mut screen_write_ctx,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut gd: *mut grid = (*s).grid;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut sx: u_int = (*(*s).grid).sx;
    let mut sy: u_int = (*(*s).grid).sy;
    let mut y: u_int = 0;
    let mut i: u_int = 0;
    let mut xoff: u_int = 0;
    let mut yoff: u_int = 0;
    let mut ocx: u_int = 0;
    let mut ocy: u_int = 0;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    if (*s).cx == 0 as u_int
        && (*s).cy == 0 as u_int
        && (*gd).flags & GRID_HISTORY != 0
        && !(*ctx).wp.is_null()
        && options_get_number(
            (*(*ctx).wp).options,
            b"scroll-on-clear\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
    {
        grid_view_clear_history(gd, bg);
    } else {
        if (*s).cx <= sx.wrapping_sub(1 as u_int) {
            grid_view_clear(
                gd,
                (*s).cx,
                (*s).cy,
                sx.wrapping_sub((*s).cx),
                1 as u_int,
                bg,
            );
        }
        grid_view_clear(
            gd,
            0 as u_int,
            (*s).cy.wrapping_add(1 as u_int),
            sx,
            sy.wrapping_sub((*s).cy.wrapping_add(1 as u_int)),
            bg,
        );
    }
    screen_write_collect_clear(
        ctx,
        (*s).cy.wrapping_add(1 as u_int),
        sy.wrapping_sub((*s).cy.wrapping_add(1 as u_int)),
    );
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_clearendofscreen\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if screen_write_should_draw_lines(ctx, (*s).cy, sy.wrapping_sub((*s).cy)) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 {
        tty_write(
            Some(tty_cmd_clearendofscreen as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    ocx = (*s).cx;
    ocy = (*s).cy;
    if !(*ctx).wp.is_null() {
        xoff = (*(*ctx).wp).xoff as u_int;
        yoff = (*(*ctx).wp).yoff as u_int;
    } else {
        xoff = 0 as u_int;
        yoff = 0 as u_int;
    }
    if (*s).cx <= sx.wrapping_sub(1 as u_int) {
        r = window_visible_ranges(
            (*ctx).wp as *mut window_pane,
            xoff.wrapping_add((*s).cx) as ::core::ffi::c_int,
            yoff.wrapping_add((*s).cy) as ::core::ffi::c_int,
            sx.wrapping_sub((*s).cx),
            ::core::ptr::null_mut::<visible_ranges>(),
        );
        i = 0 as u_int;
        while i < (*r).used {
            ri = (*r).ranges.offset(i as isize) as *mut visible_range;
            if !((*ri).nx == 0 as u_int) {
                screen_write_collect_insert_clear(ctx, (*ri).px.wrapping_sub(xoff), (*ri).nx, bg);
            }
            i = i.wrapping_add(1);
        }
    }
    y = (*s).cy.wrapping_add(1 as u_int);
    while y < sy {
        screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, y as ::core::ffi::c_int);
        r = window_visible_ranges(
            (*ctx).wp as *mut window_pane,
            xoff as ::core::ffi::c_int,
            yoff.wrapping_add(y) as ::core::ffi::c_int,
            sx,
            ::core::ptr::null_mut::<visible_ranges>(),
        );
        i = 0 as u_int;
        while i < (*r).used {
            ri = (*r).ranges.offset(i as isize) as *mut visible_range;
            if !((*ri).nx == 0 as u_int) {
                screen_write_collect_insert_clear(ctx, (*ri).px.wrapping_sub(xoff), (*ri).nx, bg);
            }
            i = i.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    screen_write_set_cursor(ctx, ocx as ::core::ffi::c_int, ocy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clearstartofscreen(
    mut ctx: *mut screen_write_ctx,
    mut bg: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut sx: u_int = (*(*s).grid).sx;
    let mut y: u_int = 0;
    let mut i: u_int = 0;
    let mut xoff: u_int = 0;
    let mut yoff: u_int = 0;
    let mut ocx: u_int = 0;
    let mut ocy: u_int = 0;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    if (*s).cy > 0 as u_int {
        grid_view_clear((*s).grid, 0 as u_int, 0 as u_int, sx, (*s).cy, bg);
    }
    if (*s).cx > sx.wrapping_sub(1 as u_int) {
        grid_view_clear((*s).grid, 0 as u_int, (*s).cy, sx, 1 as u_int, bg);
    } else {
        grid_view_clear(
            (*s).grid,
            0 as u_int,
            (*s).cy,
            (*s).cx.wrapping_add(1 as u_int),
            1 as u_int,
            bg,
        );
    }
    screen_write_collect_clear(ctx, 0 as u_int, (*s).cy);
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_clearstartofscreen\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if screen_write_should_draw_lines(ctx, 0 as u_int, (*s).cy.wrapping_add(1 as u_int)) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 {
        tty_write(
            Some(
                tty_cmd_clearstartofscreen as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> (),
            ),
            &raw mut ttyctx,
        );
        return;
    }
    ocx = (*s).cx;
    ocy = (*s).cy;
    if !(*ctx).wp.is_null() {
        xoff = (*(*ctx).wp).xoff as u_int;
        yoff = (*(*ctx).wp).yoff as u_int;
    } else {
        xoff = 0 as u_int;
        yoff = 0 as u_int;
    }
    y = 0 as u_int;
    while y < (*s).cy {
        screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, y as ::core::ffi::c_int);
        r = window_visible_ranges(
            (*ctx).wp as *mut window_pane,
            xoff as ::core::ffi::c_int,
            yoff.wrapping_add(y) as ::core::ffi::c_int,
            sx,
            ::core::ptr::null_mut::<visible_ranges>(),
        );
        i = 0 as u_int;
        while i < (*r).used {
            ri = (*r).ranges.offset(i as isize) as *mut visible_range;
            if !((*ri).nx == 0 as u_int) {
                screen_write_collect_insert_clear(ctx, (*ri).px.wrapping_sub(xoff), (*ri).nx, bg);
            }
            i = i.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, (*s).cy as ::core::ffi::c_int);
    r = window_visible_ranges(
        (*ctx).wp as *mut window_pane,
        xoff as ::core::ffi::c_int,
        yoff.wrapping_add(ocy) as ::core::ffi::c_int,
        (*s).cx.wrapping_add(1 as u_int),
        ::core::ptr::null_mut::<visible_ranges>(),
    );
    i = 0 as u_int;
    while i < (*r).used {
        ri = (*r).ranges.offset(i as isize) as *mut visible_range;
        if !((*ri).nx == 0 as u_int) {
            screen_write_collect_insert_clear(ctx, (*ri).px.wrapping_sub(xoff), (*ri).nx, bg);
        }
        i = i.wrapping_add(1);
    }
    screen_write_set_cursor(ctx, ocx as ::core::ffi::c_int, ocy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clearscreen(mut ctx: *mut screen_write_ctx, mut bg: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut sx: u_int = (*(*s).grid).sx;
    let mut sy: u_int = (*(*s).grid).sy;
    let mut y: u_int = 0;
    let mut i: u_int = 0;
    let mut xoff: u_int = 0;
    let mut yoff: u_int = 0;
    let mut ocx: u_int = 0;
    let mut ocy: u_int = 0;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    ttyctx.bg = bg;
    if (*(*s).grid).flags & GRID_HISTORY != 0
        && !(*ctx).wp.is_null()
        && options_get_number(
            (*(*ctx).wp).options,
            b"scroll-on-clear\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
    {
        grid_view_clear_history((*s).grid, bg);
    } else {
        grid_view_clear((*s).grid, 0 as u_int, 0 as u_int, sx, sy, bg);
    }
    screen_write_collect_clear(ctx, 0 as u_int, sy);
    if screen_write_should_draw_lines(ctx, 0 as u_int, sy) == 0 {
        return;
    }
    if !ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 {
        tty_write(
            Some(tty_cmd_clearscreen as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
        return;
    }
    ocx = (*s).cx;
    ocy = (*s).cy;
    if !(*ctx).wp.is_null() {
        xoff = (*(*ctx).wp).xoff as u_int;
        yoff = (*(*ctx).wp).yoff as u_int;
    } else {
        xoff = 0 as u_int;
        yoff = 0 as u_int;
    }
    y = 0 as u_int;
    while y < sy {
        screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, y as ::core::ffi::c_int);
        r = window_visible_ranges(
            (*ctx).wp as *mut window_pane,
            xoff as ::core::ffi::c_int,
            yoff.wrapping_add(y) as ::core::ffi::c_int,
            sx,
            ::core::ptr::null_mut::<visible_ranges>(),
        );
        i = 0 as u_int;
        while i < (*r).used {
            ri = (*r).ranges.offset(i as isize) as *mut visible_range;
            if !((*ri).nx == 0 as u_int) {
                screen_write_collect_insert_clear(ctx, (*ri).px.wrapping_sub(xoff), (*ri).nx, bg);
            }
            i = i.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
    screen_write_set_cursor(ctx, ocx as ::core::ffi::c_int, ocy as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_clearhistory(mut ctx: *mut screen_write_ctx) {
    grid_clear_history((*(*ctx).s).grid);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_fullredraw(mut ctx: *mut screen_write_ctx) {
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_fullredraw\0" as *const u8 as *const ::core::ffi::c_char,
    );
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if ttyctx.redraw_cb.is_some() {
        ttyctx.redraw_cb.expect("non-null function pointer")(&raw mut ttyctx);
    }
}
unsafe extern "C" fn screen_write_collect_trim(
    mut ctx: *mut screen_write_ctx,
    mut y: u_int,
    mut x: u_int,
    mut used: u_int,
    mut wrapped: *mut ::core::ffi::c_int,
) -> *mut screen_write_citem {
    let mut cl: *mut screen_write_cline =
        (*(*ctx).s).write_list.offset(y as isize) as *mut screen_write_cline;
    let mut ci: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    let mut ci2: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    let mut tmp: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    let mut before: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    let mut sx: u_int = x;
    let mut ex: u_int = x.wrapping_add(used).wrapping_sub(1 as u_int);
    let mut csx: u_int = 0;
    let mut cex: u_int = 0;
    if (*cl).items.tqh_first.is_null() {
        return ::core::ptr::null_mut::<screen_write_citem>();
    }
    ci = (*cl).items.tqh_first;
    while !ci.is_null() && {
        tmp = (*ci).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        csx = (*ci).x;
        cex = (*ci).x.wrapping_add((*ci).used).wrapping_sub(1 as u_int);
        if cex < sx {
            log_debug(
                b"%s: %p %u-%u before %u-%u\0" as *const u8 as *const ::core::ffi::c_char,
                b"screen_write_collect_trim\0" as *const u8 as *const ::core::ffi::c_char,
                ci,
                csx,
                cex,
                sx,
                ex,
            );
        } else if csx > ex {
            log_debug(
                b"%s: %p %u-%u after %u-%u\0" as *const u8 as *const ::core::ffi::c_char,
                b"screen_write_collect_trim\0" as *const u8 as *const ::core::ffi::c_char,
                ci,
                csx,
                cex,
                sx,
                ex,
            );
            before = ci;
            break;
        } else if csx >= sx && cex <= ex {
            log_debug(
                b"%s: %p %u-%u inside %u-%u\0" as *const u8 as *const ::core::ffi::c_char,
                b"screen_write_collect_trim\0" as *const u8 as *const ::core::ffi::c_char,
                ci,
                csx,
                cex,
                sx,
                ex,
            );
            if !(*ci).entry.tqe_next.is_null() {
                (*(*ci).entry.tqe_next).entry.tqe_prev = (*ci).entry.tqe_prev;
            } else {
                (*cl).items.tqh_last = (*ci).entry.tqe_prev;
            }
            *(*ci).entry.tqe_prev = (*ci).entry.tqe_next;
            screen_write_free_citem(ci);
            if csx == 0 as u_int && (*ci).wrapped != 0 && !wrapped.is_null() {
                *wrapped = 1 as ::core::ffi::c_int;
            }
        } else if csx < sx && cex >= sx && cex <= ex {
            log_debug(
                b"%s: %p %u-%u start %u-%u\0" as *const u8 as *const ::core::ffi::c_char,
                b"screen_write_collect_trim\0" as *const u8 as *const ::core::ffi::c_char,
                ci,
                csx,
                cex,
                sx,
                ex,
            );
            (*ci).used = sx.wrapping_sub(csx);
            log_debug(
                b"%s: %p now %u-%u\0" as *const u8 as *const ::core::ffi::c_char,
                b"screen_write_collect_trim\0" as *const u8 as *const ::core::ffi::c_char,
                ci,
                (*ci).x,
                (*ci).x.wrapping_add((*ci).used).wrapping_add(1 as u_int),
            );
        } else if cex > ex && csx >= sx && csx <= ex {
            log_debug(
                b"%s: %p %u-%u end %u-%u\0" as *const u8 as *const ::core::ffi::c_char,
                b"screen_write_collect_trim\0" as *const u8 as *const ::core::ffi::c_char,
                ci,
                csx,
                cex,
                sx,
                ex,
            );
            (*ci).x = ex.wrapping_add(1 as u_int);
            (*ci).used = cex.wrapping_sub(ex);
            log_debug(
                b"%s: %p now %u-%u\0" as *const u8 as *const ::core::ffi::c_char,
                b"screen_write_collect_trim\0" as *const u8 as *const ::core::ffi::c_char,
                ci,
                (*ci).x,
                (*ci).x.wrapping_add((*ci).used).wrapping_add(1 as u_int),
            );
            before = ci;
            break;
        } else {
            log_debug(
                b"%s: %p %u-%u under %u-%u\0" as *const u8 as *const ::core::ffi::c_char,
                b"screen_write_collect_trim\0" as *const u8 as *const ::core::ffi::c_char,
                ci,
                csx,
                cex,
                sx,
                ex,
            );
            ci2 = screen_write_get_citem();
            (*ci2).type_0 = (*ci).type_0;
            (*ci2).bg = (*ci).bg;
            memcpy(
                &raw mut (*ci2).gc as *mut ::core::ffi::c_void,
                &raw mut (*ci).gc as *const ::core::ffi::c_void,
                ::core::mem::size_of::<grid_cell>() as size_t,
            );
            (*ci2).entry.tqe_next = (*ci).entry.tqe_next;
            if !(*ci2).entry.tqe_next.is_null() {
                (*(*ci2).entry.tqe_next).entry.tqe_prev = &raw mut (*ci2).entry.tqe_next;
            } else {
                (*cl).items.tqh_last = &raw mut (*ci2).entry.tqe_next;
            }
            (*ci).entry.tqe_next = ci2;
            (*ci2).entry.tqe_prev = &raw mut (*ci).entry.tqe_next;
            (*ci).used = sx.wrapping_sub(csx);
            (*ci2).x = ex.wrapping_add(1 as u_int);
            (*ci2).used = cex.wrapping_sub(ex);
            log_debug(
                b"%s: %p now %u-%u (%p) and %u-%u (%p)\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"screen_write_collect_trim\0" as *const u8 as *const ::core::ffi::c_char,
                ci,
                (*ci).x,
                (*ci).x.wrapping_add((*ci).used).wrapping_sub(1 as u_int),
                ci,
                (*ci2).x,
                (*ci2).x.wrapping_add((*ci2).used).wrapping_sub(1 as u_int),
                ci2,
            );
            before = ci2;
            break;
        }
        ci = tmp;
    }
    return before;
}
unsafe extern "C" fn screen_write_collect_clear(
    mut ctx: *mut screen_write_ctx,
    mut y: u_int,
    mut n: u_int,
) {
    let mut cl: *mut screen_write_cline = ::core::ptr::null_mut::<screen_write_cline>();
    let mut i: u_int = 0;
    i = y;
    while i < y.wrapping_add(n) {
        cl = (*(*ctx).s).write_list.offset(i as isize) as *mut screen_write_cline;
        if !(*cl).items.tqh_first.is_null() {
            *screen_write_citem_freelist.tqh_last = (*cl).items.tqh_first;
            (*(*cl).items.tqh_first).entry.tqe_prev = screen_write_citem_freelist.tqh_last;
            screen_write_citem_freelist.tqh_last = (*cl).items.tqh_last;
            (*cl).items.tqh_first = ::core::ptr::null_mut::<screen_write_citem>();
            (*cl).items.tqh_last = &raw mut (*cl).items.tqh_first;
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn screen_write_collect_scroll(mut ctx: *mut screen_write_ctx, mut bg: u_int) {
    let mut s: *mut screen = (*ctx).s;
    let mut cl: *mut screen_write_cline = ::core::ptr::null_mut::<screen_write_cline>();
    let mut y: u_int = 0;
    let mut saved: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ci: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    log_debug(
        b"%s: at %u,%u (region %u-%u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_collect_scroll\0" as *const u8 as *const ::core::ffi::c_char,
        (*s).cx,
        (*s).cy,
        (*s).rupper,
        (*s).rlower,
    );
    screen_write_collect_clear(ctx, (*s).rupper, 1 as u_int);
    saved = (*(*(*ctx).s).write_list.offset((*s).rupper as isize)).data;
    y = (*s).rupper;
    while y < (*s).rlower {
        cl = (*(*ctx).s)
            .write_list
            .offset(y.wrapping_add(1 as u_int) as isize) as *mut screen_write_cline;
        if !(*cl).items.tqh_first.is_null() {
            let ref mut fresh3 = *(*(*(*ctx).s).write_list.offset(y as isize)).items.tqh_last;
            *fresh3 = (*cl).items.tqh_first;
            (*(*cl).items.tqh_first).entry.tqe_prev =
                (*(*(*ctx).s).write_list.offset(y as isize)).items.tqh_last;
            let ref mut fresh4 = (*(*(*ctx).s).write_list.offset(y as isize)).items.tqh_last;
            *fresh4 = (*cl).items.tqh_last;
            (*cl).items.tqh_first = ::core::ptr::null_mut::<screen_write_citem>();
            (*cl).items.tqh_last = &raw mut (*cl).items.tqh_first;
        }
        let ref mut fresh5 = (*(*(*ctx).s).write_list.offset(y as isize)).data;
        *fresh5 = (*cl).data;
        y = y.wrapping_add(1);
    }
    let ref mut fresh6 = (*(*(*ctx).s).write_list.offset((*s).rlower as isize)).data;
    *fresh6 = saved;
    ci = screen_write_get_citem();
    (*ci).x = 0 as u_int;
    (*ci).used = (*(*s).grid).sx;
    (*ci).type_0 = CLEAR;
    (*ci).bg = bg;
    (*ci).entry.tqe_next = ::core::ptr::null_mut::<screen_write_citem>();
    (*ci).entry.tqe_prev = (*(*(*ctx).s).write_list.offset((*s).rlower as isize))
        .items
        .tqh_last;
    let ref mut fresh7 = *(*(*(*ctx).s).write_list.offset((*s).rlower as isize))
        .items
        .tqh_last;
    *fresh7 = ci;
    let ref mut fresh8 = (*(*(*ctx).s).write_list.offset((*s).rlower as isize))
        .items
        .tqh_last;
    *fresh8 = &raw mut (*ci).entry.tqe_next;
}
unsafe extern "C" fn screen_write_collect_flush_scrolled(
    mut ctx: *mut screen_write_ctx,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut s: *mut screen = (*ctx).s;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    if ttyctx.flags & TTY_CTX_PANE_OBSCURED != 0 && !wp.is_null() {
        screen_write_redraw_pane(ctx, &raw mut ttyctx);
        return 0 as ::core::ffi::c_int;
    }
    if !wp.is_null() && window_pane_scrollbar_overlay_visible(wp) != 0 {
        (*wp).flags |= PANE_REDRAW;
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: scrolled %u (region %u-%u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_collect_flush_scrolled\0" as *const u8 as *const ::core::ffi::c_char,
        (*ctx).scrolled,
        (*s).rupper,
        (*s).rlower,
    );
    if (*ctx).scrolled
        > (*s)
            .rlower
            .wrapping_sub((*s).rupper)
            .wrapping_add(1 as u_int)
    {
        (*ctx).scrolled = (*s)
            .rlower
            .wrapping_sub((*s).rupper)
            .wrapping_add(1 as u_int);
    }
    if !wp.is_null() && ((*wp).yoff as u_int).wrapping_add((*wp).sy) > (*(*wp).window).sy {
        ttyctx.orlower = ttyctx.orlower.wrapping_sub(
            ((*wp).yoff as u_int)
                .wrapping_add((*wp).sy)
                .wrapping_sub((*(*wp).window).sy),
        );
    }
    ttyctx.c2rust_unnamed.n = (*ctx).scrolled;
    ttyctx.bg = (*ctx).bg;
    tty_write(
        Some(tty_cmd_scrollup as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
        &raw mut ttyctx,
    );
    if !wp.is_null() {
        window_pane_scrollbar_redraw(wp);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn screen_write_collect_flush_line(
    mut ctx: *mut screen_write_ctx,
    mut y: u_int,
) -> u_int {
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut s: *mut screen = (*ctx).s;
    let mut ci: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    let mut tmp: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    let mut cl: *mut screen_write_cline =
        (*s).write_list.offset(y as isize) as *mut screen_write_cline;
    let mut last: u_int = UINT_MAX;
    let mut items: u_int = 0 as u_int;
    let mut wsx: u_int = 0;
    let mut wsy: u_int = 0;
    let mut w_length: u_int = 0;
    let mut i: u_int = 0;
    let mut w_start: ::core::ffi::c_int = 0;
    let mut w_end: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut written: ::core::ffi::c_int = 0;
    let mut r_start: ::core::ffi::c_int = 0;
    let mut r_end: ::core::ffi::c_int = 0;
    let mut c_start: ::core::ffi::c_int = 0;
    let mut c_end: ::core::ffi::c_int = 0;
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    if !wp.is_null() {
        wsx = (*(*wp).window).sx;
        wsy = (*(*wp).window).sy;
        xoff = (*wp).xoff;
        yoff = (*wp).yoff;
    } else {
        wsx = (*(*s).grid).sx;
        wsy = (*(*s).grid).sy;
        xoff = 0 as ::core::ffi::c_int;
        yoff = 0 as ::core::ffi::c_int;
    }
    if y.wrapping_add(yoff as u_int) >= wsy {
        return 0 as u_int;
    }
    r = window_visible_ranges(
        wp,
        0 as ::core::ffi::c_int,
        y.wrapping_add(yoff as u_int) as ::core::ffi::c_int,
        wsx,
        ::core::ptr::null_mut::<visible_ranges>(),
    );
    ci = (*cl).items.tqh_first;
    while !ci.is_null() && {
        tmp = (*ci).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        log_debug(
            b"collect list: x=%u (last %u), y=%u, used=%u\0" as *const u8
                as *const ::core::ffi::c_char,
            (*ci).x,
            last,
            y,
            (*ci).used,
        );
        if last != UINT_MAX && (*ci).x <= last {
            fatalx(
                b"collect list bad order: %u <= %u\0" as *const u8 as *const ::core::ffi::c_char,
                (*ci).x,
                last,
            );
        }
        w_length = 0 as u_int;
        written = 0 as ::core::ffi::c_int;
        i = 0 as u_int;
        while i < (*r).used {
            ri = (*r).ranges.offset(i as isize) as *mut visible_range;
            if !((*ri).nx == 0 as u_int) {
                r_start = (*ri).px as ::core::ffi::c_int;
                r_end = (*ri).px.wrapping_add((*ri).nx) as ::core::ffi::c_int;
                c_start = (*ci).x as ::core::ffi::c_int;
                c_end = (*ci).x.wrapping_add((*ci).used) as ::core::ffi::c_int;
                if !(c_start + xoff >= r_end || c_end + xoff <= r_start) {
                    if r_start > c_start + xoff {
                        w_start = r_start - xoff;
                    } else {
                        w_start = c_start;
                    }
                    if c_end + xoff > r_end {
                        w_end = r_end - xoff;
                    } else {
                        w_end = c_end;
                    }
                    if !(w_end <= w_start) {
                        w_length = (w_end - w_start) as u_int;
                        if !(w_length <= 0 as u_int) {
                            screen_write_set_cursor(ctx, w_start, y as ::core::ffi::c_int);
                            if (*ci).type_0 as ::core::ffi::c_uint
                                == CLEAR as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                screen_write_initctx(
                                    ctx,
                                    &raw mut ttyctx,
                                    1 as ::core::ffi::c_int,
                                    0 as ::core::ffi::c_int,
                                );
                                ttyctx.bg = (*ci).bg;
                                ttyctx.c2rust_unnamed.n = w_length;
                                tty_write(
                                    Some(
                                        tty_cmd_clearcharacter
                                            as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> (),
                                    ),
                                    &raw mut ttyctx,
                                );
                            } else {
                                screen_write_initctx(
                                    ctx,
                                    &raw mut ttyctx,
                                    0 as ::core::ffi::c_int,
                                    0 as ::core::ffi::c_int,
                                );
                                ttyctx.cell = &raw mut (*ci).gc;
                                if (*ci).wrapped != 0 {
                                    ttyctx.flags |= TTY_CTX_WRAPPED;
                                }
                                ttyctx.c2rust_unnamed.data.data =
                                    (*cl).data.offset(w_start as isize);
                                ttyctx.c2rust_unnamed.data.size = w_length as size_t;
                                tty_write(
                                    Some(
                                        tty_cmd_cells
                                            as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> (),
                                    ),
                                    &raw mut ttyctx,
                                );
                            }
                            items = items.wrapping_add(1);
                            written = 1 as ::core::ffi::c_int;
                        }
                    }
                }
            }
            i = i.wrapping_add(1);
        }
        if written != 0 {
            last = (*ci).x;
            if !(*ci).entry.tqe_next.is_null() {
                (*(*ci).entry.tqe_next).entry.tqe_prev = (*ci).entry.tqe_prev;
            } else {
                (*cl).items.tqh_last = (*ci).entry.tqe_prev;
            }
            *(*ci).entry.tqe_prev = (*ci).entry.tqe_next;
            screen_write_free_citem(ci);
        }
        ci = tmp;
    }
    return items;
}
unsafe extern "C" fn screen_write_collect_flush(
    mut ctx: *mut screen_write_ctx,
    mut scroll_only: ::core::ffi::c_int,
    mut from: *const ::core::ffi::c_char,
) {
    let mut current_block: u64;
    let mut s: *mut screen = (*ctx).s;
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut y: u_int = 0;
    let mut cx: u_int = 0;
    let mut cy: u_int = 0;
    let mut items: u_int = 0 as u_int;
    let mut ci: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    let mut tmp: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    let mut cl: *mut screen_write_cline = ::core::ptr::null_mut::<screen_write_cline>();
    if !(!wp.is_null() && (*wp).flags & (PANE_REDRAW | PANE_DROP) != 0) {
        if (*s).mode & MODE_SYNC != 0 {
            if (*ctx).scrolled != 0 as u_int {
                screen_write_should_draw_lines(
                    ctx,
                    (*s).rupper,
                    (*s).rlower
                        .wrapping_add(1 as u_int)
                        .wrapping_sub((*s).rupper),
                );
            }
            y = 0 as u_int;
            while y < (*(*s).grid).sy {
                cl = (*s).write_list.offset(y as isize) as *mut screen_write_cline;
                if !(*cl).items.tqh_first.is_null() {
                    screen_write_should_draw_line(ctx, y);
                }
                y = y.wrapping_add(1);
            }
        } else {
            if (*ctx).scrolled != 0 as u_int {
                if screen_write_collect_flush_scrolled(ctx) == 0 {
                    current_block = 9123463459983562265;
                } else {
                    (*ctx).scrolled = 0 as u_int;
                    current_block = 8236137900636309791;
                }
            } else {
                current_block = 8236137900636309791;
            }
            match current_block {
                9123463459983562265 => {}
                _ => {
                    (*ctx).bg = 8 as u_int;
                    if scroll_only != 0 {
                        return;
                    }
                    cx = (*s).cx;
                    cy = (*s).cy;
                    y = 0 as u_int;
                    while y < (*(*s).grid).sy {
                        items = items.wrapping_add(screen_write_collect_flush_line(ctx, y));
                        y = y.wrapping_add(1);
                    }
                    (*s).cx = cx;
                    (*s).cy = cy;
                    log_debug(
                        b"%s: flushed %u items (%s)\0" as *const u8 as *const ::core::ffi::c_char,
                        b"screen_write_collect_flush\0" as *const u8 as *const ::core::ffi::c_char,
                        items,
                        from,
                    );
                    return;
                }
            }
        }
    }
    y = 0 as u_int;
    while y < (*(*s).grid).sy {
        cl = (*s).write_list.offset(y as isize) as *mut screen_write_cline;
        ci = (*cl).items.tqh_first;
        while !ci.is_null() && {
            tmp = (*ci).entry.tqe_next;
            1 as ::core::ffi::c_int != 0
        } {
            if !(*ci).entry.tqe_next.is_null() {
                (*(*ci).entry.tqe_next).entry.tqe_prev = (*ci).entry.tqe_prev;
            } else {
                (*cl).items.tqh_last = (*ci).entry.tqe_prev;
            }
            *(*ci).entry.tqe_prev = (*ci).entry.tqe_next;
            screen_write_free_citem(ci);
            ci = tmp;
        }
        y = y.wrapping_add(1);
    }
    (*ctx).scrolled = 0 as u_int;
    (*ctx).bg = 8 as u_int;
}
unsafe extern "C" fn screen_write_collect_insert(
    mut ctx: *mut screen_write_ctx,
    mut ci: *mut screen_write_citem,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut cl: *mut screen_write_cline =
        (*s).write_list.offset((*s).cy as isize) as *mut screen_write_cline;
    let mut before: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    before = screen_write_collect_trim(ctx, (*s).cy, (*ci).x, (*ci).used, &raw mut (*ci).wrapped);
    if before.is_null() {
        (*ci).entry.tqe_next = ::core::ptr::null_mut::<screen_write_citem>();
        (*ci).entry.tqe_prev = (*cl).items.tqh_last;
        *(*cl).items.tqh_last = ci;
        (*cl).items.tqh_last = &raw mut (*ci).entry.tqe_next;
    } else {
        (*ci).entry.tqe_prev = (*before).entry.tqe_prev;
        (*ci).entry.tqe_next = before;
        *(*before).entry.tqe_prev = ci;
        (*before).entry.tqe_prev = &raw mut (*ci).entry.tqe_next;
    }
    (*ctx).item = screen_write_get_citem();
}
unsafe extern "C" fn screen_write_collect_insert_clear(
    mut ctx: *mut screen_write_ctx,
    mut px: u_int,
    mut nx: u_int,
    mut bg: u_int,
) {
    let mut ci: *mut screen_write_citem = (*ctx).item;
    if nx != 0 as u_int {
        (*ci).x = px;
        (*ci).used = nx;
        (*ci).type_0 = CLEAR;
        (*ci).bg = bg;
        screen_write_collect_insert(ctx, ci);
    }
}
unsafe extern "C" fn screen_write_clear_cell(mut gd: *mut grid, mut px: u_int, mut py: u_int) {
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
    let mut bg: ::core::ffi::c_int = 0;
    grid_view_get_cell(gd, px, py, &raw mut gc);
    bg = gc.bg;
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    gc.bg = bg;
    grid_view_set_cell(gd, px, py, &raw mut gc);
}
unsafe extern "C" fn screen_write_insert_clears(
    mut ctx: *mut screen_write_ctx,
    mut px: u_int,
    mut nx: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
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
    let mut xx: u_int = 0;
    let mut start: u_int = px;
    let mut n: u_int = 0;
    let mut bg: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
    xx = px;
    while xx < px.wrapping_add(nx) {
        grid_view_get_cell((*s).grid, xx, (*s).cy, &raw mut gc);
        if xx == start {
            bg = gc.bg;
        } else if gc.bg != bg {
            n = xx.wrapping_sub(start);
            log_debug(
                b"%s: from %u, size %u\0" as *const u8 as *const ::core::ffi::c_char,
                b"screen_write_insert_clears\0" as *const u8 as *const ::core::ffi::c_char,
                start,
                n,
            );
            screen_write_collect_insert_clear(ctx, start, n, bg as u_int);
            start = xx;
            bg = gc.bg;
        }
        xx = xx.wrapping_add(1);
    }
    log_debug(
        b"%s: from %u, size %u\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_insert_clears\0" as *const u8 as *const ::core::ffi::c_char,
        start,
        xx.wrapping_sub(start),
    );
    screen_write_collect_insert_clear(ctx, start, xx.wrapping_sub(start), bg as u_int);
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_collect_end(mut ctx: *mut screen_write_ctx) {
    let mut s: *mut screen = (*ctx).s;
    let mut ci: *mut screen_write_citem = (*ctx).item;
    let mut cl: *mut screen_write_cline =
        (*s).write_list.offset((*s).cy as isize) as *mut screen_write_cline;
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
    let mut xx: u_int = 0;
    let mut bx: u_int = 0 as u_int;
    let mut bnx: u_int = 0 as u_int;
    if (*ci).used == 0 as u_int {
        return;
    }
    (*ci).x = (*s).cx;
    screen_write_collect_insert(ctx, ci);
    log_debug(
        b"%s: %u %.*s (at %u,%u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_collect_end\0" as *const u8 as *const ::core::ffi::c_char,
        (*ci).used,
        (*ci).used as ::core::ffi::c_int,
        (*cl).data.offset((*ci).x as isize),
        (*s).cx,
        (*s).cy,
    );
    if (*s).cx != 0 as u_int {
        xx = (*s).cx;
        while xx > 0 as u_int {
            grid_view_get_cell((*s).grid, xx, (*s).cy, &raw mut gc);
            if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
                break;
            }
            screen_write_clear_cell((*s).grid, xx, (*s).cy);
            log_debug(
                b"%s: padding erased (before) at %u (cx %u)\0" as *const u8
                    as *const ::core::ffi::c_char,
                b"screen_write_collect_end\0" as *const u8 as *const ::core::ffi::c_char,
                xx,
                (*s).cx,
            );
            xx = xx.wrapping_sub(1);
        }
        if xx != (*s).cx {
            if xx == 0 as u_int {
                grid_view_get_cell((*s).grid, 0 as u_int, (*s).cy, &raw mut gc);
            }
            if gc.data.width as ::core::ffi::c_int > 1 as ::core::ffi::c_int
                || gc.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0
            {
                screen_write_clear_cell((*s).grid, xx, (*s).cy);
                log_debug(
                    b"%s: padding erased (before) at %u (cx %u)\0" as *const u8
                        as *const ::core::ffi::c_char,
                    b"screen_write_collect_end\0" as *const u8 as *const ::core::ffi::c_char,
                    xx,
                    (*s).cx,
                );
            }
            bx = xx;
            bnx = (*s).cx.wrapping_sub(xx);
        }
    }
    grid_view_set_cells(
        (*s).grid,
        (*s).cx,
        (*s).cy,
        &raw mut (*ci).gc,
        (*cl).data.offset((*ci).x as isize),
        (*ci).used as size_t,
    );
    if bnx != 0 as u_int {
        screen_write_insert_clears(ctx, bx, bnx);
    }
    screen_write_set_cursor(
        ctx,
        (*s).cx.wrapping_add((*ci).used) as ::core::ffi::c_int,
        -(1 as ::core::ffi::c_int),
    );
    xx = (*s).cx;
    while xx < (*(*s).grid).sx {
        grid_view_get_cell((*s).grid, xx, (*s).cy, &raw mut gc);
        if !(gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
            break;
        }
        screen_write_clear_cell((*s).grid, xx, (*s).cy);
        log_debug(
            b"%s: padding erased (after) at %u (cx %u)\0" as *const u8
                as *const ::core::ffi::c_char,
            b"screen_write_collect_end\0" as *const u8 as *const ::core::ffi::c_char,
            xx,
            (*s).cx,
        );
        xx = xx.wrapping_add(1);
    }
    if xx != (*s).cx {
        screen_write_insert_clears(ctx, (*s).cx, xx.wrapping_sub((*s).cx));
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_collect_add(
    mut ctx: *mut screen_write_ctx,
    mut gc: *const grid_cell,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut ci: *mut screen_write_citem = ::core::ptr::null_mut::<screen_write_citem>();
    let mut sx: u_int = (*(*s).grid).sx;
    let mut collect: ::core::ffi::c_int = 0;
    collect = 1 as ::core::ffi::c_int;
    if (*gc).data.width as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        || (*gc).data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        || *(&raw const (*gc).data.data as *const u_char) as ::core::ffi::c_int
            >= 0x7f as ::core::ffi::c_int
    {
        collect = 0 as ::core::ffi::c_int;
    } else if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_TAB != 0 {
        collect = 0 as ::core::ffi::c_int;
    } else if (*gc).attr as ::core::ffi::c_int & GRID_ATTR_CHARSET != 0 {
        collect = 0 as ::core::ffi::c_int;
    } else if !(*s).mode & MODE_WRAP != 0 {
        collect = 0 as ::core::ffi::c_int;
    } else if (*s).mode & MODE_INSERT != 0 {
        collect = 0 as ::core::ffi::c_int;
    } else if !(*s).sel.is_null() {
        collect = 0 as ::core::ffi::c_int;
    }
    if collect == 0 {
        screen_write_collect_end(ctx);
        screen_write_collect_flush(
            ctx,
            0 as ::core::ffi::c_int,
            b"screen_write_collect_add\0" as *const u8 as *const ::core::ffi::c_char,
        );
        screen_write_cell(ctx, gc);
        return;
    }
    if (*s).cx > sx.wrapping_sub(1 as u_int)
        || (*(*ctx).item).used > sx.wrapping_sub(1 as u_int).wrapping_sub((*s).cx)
    {
        screen_write_collect_end(ctx);
    }
    ci = (*ctx).item;
    if (*s).cx > sx.wrapping_sub(1 as u_int) {
        log_debug(
            b"%s: wrapped at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_collect_add\0" as *const u8 as *const ::core::ffi::c_char,
            (*s).cx,
            (*s).cy,
        );
        (*ci).wrapped = 1 as ::core::ffi::c_int;
        screen_write_linefeed(ctx, 1 as ::core::ffi::c_int, 8 as u_int);
        screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int));
    }
    if (*ci).used == 0 as u_int {
        memcpy(
            &raw mut (*ci).gc as *mut ::core::ffi::c_void,
            gc as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    }
    if (*(*(*ctx).s).write_list.offset((*s).cy as isize))
        .data
        .is_null()
    {
        let ref mut fresh11 = (*(*(*ctx).s).write_list.offset((*s).cy as isize)).data;
        *fresh11 = xmalloc((*(*(*ctx).s).grid).sx as size_t) as *mut ::core::ffi::c_char;
    }
    let fresh12 = (*ci).used;
    (*ci).used = (*ci).used.wrapping_add(1);
    *(*(*(*ctx).s).write_list.offset((*s).cy as isize))
        .data
        .offset((*s).cx.wrapping_add(fresh12) as isize) =
        (*gc).data.data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_cell(
    mut ctx: *mut screen_write_ctx,
    mut gc: *const grid_cell,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut gd: *mut grid = (*s).grid;
    let mut ud: *const utf8_data = &raw const (*gc).data;
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut gce: *mut grid_cell_entry = ::core::ptr::null_mut::<grid_cell_entry>();
    let mut tmp_gc: grid_cell = grid_cell {
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
    let mut now_gc: grid_cell = grid_cell {
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
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut sx: u_int = (*(*s).grid).sx;
    let mut sy: u_int = (*(*s).grid).sy;
    let mut width: u_int = (*ud).width as u_int;
    let mut xx: u_int = 0;
    let mut not_wrap: u_int = 0;
    let mut i: u_int = 0;
    let mut n: u_int = 0;
    let mut vis: u_int = 0;
    let mut selected: ::core::ffi::c_int = 0;
    let mut skip: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut yoff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut xoff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ri: *mut visible_range = ::core::ptr::null_mut::<visible_range>();
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        return;
    }
    if screen_write_combine(ctx, gc) != 0 as ::core::ffi::c_int {
        return;
    }
    screen_write_collect_flush(
        ctx,
        1 as ::core::ffi::c_int,
        b"screen_write_cell\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if !(*s).mode & MODE_WRAP != 0
        && width > 1 as u_int
        && (width > sx || (*s).cx != sx && (*s).cx > sx.wrapping_sub(width))
    {
        return;
    }
    if (*s).mode & MODE_INSERT != 0 {
        grid_view_insert_cells((*s).grid, (*s).cx, (*s).cy, width, 8 as u_int);
        skip = 0 as ::core::ffi::c_int;
    }
    if (*s).mode & MODE_WRAP != 0 && (*s).cx > sx.wrapping_sub(width) {
        log_debug(
            b"%s: wrapped at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_cell\0" as *const u8 as *const ::core::ffi::c_char,
            (*s).cx,
            (*s).cy,
        );
        screen_write_linefeed(ctx, 1 as ::core::ffi::c_int, 8 as u_int);
        screen_write_set_cursor(ctx, 0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int));
        screen_write_collect_flush(
            ctx,
            0 as ::core::ffi::c_int,
            b"screen_write_cell\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if (*s).cx > sx.wrapping_sub(width) || (*s).cy > sy.wrapping_sub(1 as u_int) {
        return;
    }
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    gl = grid_get_line((*s).grid, (*(*s).grid).hsize.wrapping_add((*s).cy));
    if (*gl).flags as ::core::ffi::c_int & GRID_LINE_EXTENDED != 0 {
        grid_view_get_cell(gd, (*s).cx, (*s).cy, &raw mut now_gc);
        if screen_write_overwrite(ctx, &raw mut now_gc, width) != 0 {
            redraw = 1 as ::core::ffi::c_int;
            skip = 0 as ::core::ffi::c_int;
        }
    }
    xx = (*s).cx.wrapping_add(1 as u_int);
    while xx < (*s).cx.wrapping_add(width) {
        log_debug(
            b"%s: new padding at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_cell\0" as *const u8 as *const ::core::ffi::c_char,
            xx,
            (*s).cy,
        );
        grid_view_set_padding(gd, xx, (*s).cy, (*gc).bg);
        skip = 0 as ::core::ffi::c_int;
        xx = xx.wrapping_add(1);
    }
    if skip != 0 {
        if (*s).cx >= (*gl).cellsize as u_int {
            skip = grid_cells_equal(gc, &raw const grid_default_cell);
        } else {
            gce = (*gl).celldata.offset((*s).cx as isize) as *mut grid_cell_entry;
            if (*gce).flags as ::core::ffi::c_int & GRID_FLAG_EXTENDED != 0 {
                skip = 0 as ::core::ffi::c_int;
            } else if (*gc).flags as ::core::ffi::c_int != (*gce).flags as ::core::ffi::c_int {
                skip = 0 as ::core::ffi::c_int;
            } else if (*gc).attr as ::core::ffi::c_int
                != (*gce).c2rust_unnamed.data.attr as ::core::ffi::c_int
            {
                skip = 0 as ::core::ffi::c_int;
            } else if (*gc).fg != (*gce).c2rust_unnamed.data.fg as ::core::ffi::c_int {
                skip = 0 as ::core::ffi::c_int;
            } else if (*gc).bg != (*gce).c2rust_unnamed.data.bg as ::core::ffi::c_int {
                skip = 0 as ::core::ffi::c_int;
            } else if (*gc).data.width as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
                skip = 0 as ::core::ffi::c_int;
            } else if (*gc).data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int {
                skip = 0 as ::core::ffi::c_int;
            } else if (*gce).c2rust_unnamed.data.data as ::core::ffi::c_int
                != (*gc).data.data[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            {
                skip = 0 as ::core::ffi::c_int;
            }
        }
    }
    selected = screen_check_selection(s, (*s).cx, (*s).cy);
    if selected != 0 && !((*gc).flags as ::core::ffi::c_int) & GRID_FLAG_SELECTED != 0 {
        memcpy(
            &raw mut tmp_gc as *mut ::core::ffi::c_void,
            gc as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
        tmp_gc.flags = (tmp_gc.flags as ::core::ffi::c_int | GRID_FLAG_SELECTED) as u_char;
        grid_view_set_cell(gd, (*s).cx, (*s).cy, &raw mut tmp_gc);
    } else if selected == 0 && (*gc).flags as ::core::ffi::c_int & GRID_FLAG_SELECTED != 0 {
        memcpy(
            &raw mut tmp_gc as *mut ::core::ffi::c_void,
            gc as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
        tmp_gc.flags = (tmp_gc.flags as ::core::ffi::c_int & !GRID_FLAG_SELECTED) as u_char;
        grid_view_set_cell(gd, (*s).cx, (*s).cy, &raw mut tmp_gc);
    } else if skip == 0 {
        grid_view_set_cell(gd, (*s).cx, (*s).cy, gc);
    }
    if selected != 0 {
        skip = 0 as ::core::ffi::c_int;
    }
    if !wp.is_null() {
        xoff = (*wp).xoff;
        yoff = (*wp).yoff;
    }
    r = window_visible_ranges(
        wp,
        (xoff as u_int).wrapping_add((*s).cx) as ::core::ffi::c_int,
        (*s).cy.wrapping_add(yoff as u_int) as ::core::ffi::c_int,
        width,
        ::core::ptr::null_mut::<visible_ranges>(),
    );
    not_wrap = ((*s).mode & MODE_WRAP == 0) as ::core::ffi::c_int as u_int;
    if (*s).cx <= sx.wrapping_sub(not_wrap).wrapping_sub(width) {
        screen_write_set_cursor(
            ctx,
            (*s).cx.wrapping_add(width) as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
        );
    } else {
        screen_write_set_cursor(
            ctx,
            sx.wrapping_sub(not_wrap) as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
        );
    }
    if (*s).mode & MODE_INSERT != 0 {
        screen_write_collect_flush(
            ctx,
            0 as ::core::ffi::c_int,
            b"screen_write_cell\0" as *const u8 as *const ::core::ffi::c_char,
        );
        ttyctx.c2rust_unnamed.n = width;
        if screen_write_should_draw_line(ctx, (*s).cy) != 0 {
            tty_write(
                Some(
                    tty_cmd_insertcharacter as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> (),
                ),
                &raw mut ttyctx,
            );
        }
    }
    if skip != 0 || screen_write_should_draw_line(ctx, (*s).cy) == 0 {
        return;
    }
    if redraw != 0 && !wp.is_null() {
        screen_write_redraw_line(ctx, &raw mut ttyctx, (*s).cy);
        return;
    }
    if selected != 0 {
        screen_select_cell(s, &raw mut tmp_gc, gc);
    } else {
        memcpy(
            &raw mut tmp_gc as *mut ::core::ffi::c_void,
            gc as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
    }
    ttyctx.cell = &raw mut tmp_gc;
    i = 0 as u_int;
    vis = 0 as u_int;
    while i < (*r).used {
        vis = vis.wrapping_add((*(*r).ranges.offset(i as isize)).nx);
        i = i.wrapping_add(1);
    }
    if vis >= width {
        if screen_write_should_draw_line(ctx, (*s).cy) != 0 {
            tty_write(
                Some(tty_cmd_cell as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
                &raw mut ttyctx,
            );
        }
        return;
    }
    utf8_set(&raw mut tmp_gc.data, ' ' as i32 as u_char);
    if screen_write_should_draw_line(ctx, (*s).cy) == 0 {
        return;
    }
    i = 0 as u_int;
    while i < (*r).used {
        ri = (*r).ranges.offset(i as isize) as *mut visible_range;
        if !((*ri).nx == 0 as u_int) {
            n = 0 as u_int;
            while n < (*ri).nx {
                ttyctx.ocx =
                    ((*ri).px as ::core::ffi::c_int - xoff + n as ::core::ffi::c_int) as u_int;
                tty_write(
                    Some(tty_cmd_cell as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
                    &raw mut ttyctx,
                );
                n = n.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn screen_write_combine(
    mut ctx: *mut screen_write_ctx,
    mut gc: *const grid_cell,
) -> ::core::ffi::c_int {
    let mut s: *mut screen = (*ctx).s;
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    let mut gd: *mut grid = (*s).grid;
    let mut ud: *const utf8_data = &raw const (*gc).data;
    let mut oo: *mut options = global_options;
    let mut i: u_int = 0;
    let mut n: u_int = 0;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    let mut vis: u_int = 0;
    let mut last: grid_cell = grid_cell {
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
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut force_wide: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut zero_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut xoff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut yoff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    if utf8_is_hangul_filler(ud) != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if utf8_is_zwj(ud) != 0 {
        zero_width = 1 as ::core::ffi::c_int;
    } else if utf8_is_vs(ud) != 0 {
        zero_width = 1 as ::core::ffi::c_int;
        if options_get_number(
            oo,
            b"variation-selector-always-wide\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
        {
            force_wide = 1 as ::core::ffi::c_int;
        }
    } else if (*ud).width as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        zero_width = 1 as ::core::ffi::c_int;
    }
    if ((*ud).size as ::core::ffi::c_int) < 2 as ::core::ffi::c_int || cx == 0 as u_int {
        return zero_width;
    }
    log_debug(
        b"%s: character %.*s at %u,%u (width %u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"screen_write_combine\0" as *const u8 as *const ::core::ffi::c_char,
        (*ud).size as ::core::ffi::c_int,
        &raw const (*ud).data as *const u_char,
        cx,
        cy,
        (*ud).width as ::core::ffi::c_int,
    );
    n = 1 as u_int;
    grid_view_get_cell(gd, cx.wrapping_sub(n), cy, &raw mut last);
    if cx != 1 as u_int && last.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        n = 2 as u_int;
        grid_view_get_cell(gd, cx.wrapping_sub(n), cy, &raw mut last);
    }
    if n != last.data.width as u_int || last.flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        return zero_width;
    }
    if zero_width == 0 {
        match hanguljamo_check_state(&raw mut last.data, ud) as ::core::ffi::c_uint {
            3 => return 1 as ::core::ffi::c_int,
            1 => return 0 as ::core::ffi::c_int,
            0 => {
                if utf8_should_combine(&raw mut last.data, ud) != 0 {
                    force_wide = 1 as ::core::ffi::c_int;
                } else if utf8_should_combine(ud, &raw mut last.data) != 0 {
                    force_wide = 1 as ::core::ffi::c_int;
                } else if utf8_has_zwj(&raw mut last.data) == 0 {
                    return 0 as ::core::ffi::c_int;
                }
            }
            2 | _ => {}
        }
    }
    if (last.data.size as ::core::ffi::c_int + (*ud).size as ::core::ffi::c_int) as usize
        > ::core::mem::size_of::<[u_char; 32]>() as usize
    {
        return zero_width;
    }
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_combine\0" as *const u8 as *const ::core::ffi::c_char,
    );
    log_debug(
        b"%s: %.*s -> %.*s at %u,%u (offset %u, width %u)\0" as *const u8
            as *const ::core::ffi::c_char,
        b"screen_write_combine\0" as *const u8 as *const ::core::ffi::c_char,
        (*ud).size as ::core::ffi::c_int,
        &raw const (*ud).data as *const u_char,
        last.data.size as ::core::ffi::c_int,
        &raw mut last.data.data as *mut u_char,
        cx.wrapping_sub(n),
        cy,
        n,
        last.data.width as ::core::ffi::c_int,
    );
    memcpy(
        (&raw mut last.data.data as *mut u_char)
            .offset(last.data.size as ::core::ffi::c_int as isize)
            as *mut ::core::ffi::c_void,
        &raw const (*ud).data as *const u_char as *const ::core::ffi::c_void,
        (*ud).size as size_t,
    );
    last.data.size =
        (last.data.size as ::core::ffi::c_int + (*ud).size as ::core::ffi::c_int) as u_char;
    if last.data.width as ::core::ffi::c_int == 1 as ::core::ffi::c_int && force_wide != 0 {
        last.data.width = 2 as u_char;
        n = 2 as u_int;
        cx = cx.wrapping_add(1);
    } else {
        force_wide = 0 as ::core::ffi::c_int;
    }
    grid_view_set_cell(gd, cx.wrapping_sub(n), cy, &raw mut last);
    if force_wide != 0 {
        grid_view_set_padding(gd, cx.wrapping_sub(1 as u_int), cy, last.bg);
    }
    if !wp.is_null() {
        xoff = (*wp).xoff;
        yoff = (*wp).yoff;
    }
    r = window_visible_ranges(
        wp,
        (xoff as u_int).wrapping_add(cx).wrapping_sub(n) as ::core::ffi::c_int,
        cy.wrapping_add(yoff as u_int) as ::core::ffi::c_int,
        n,
        ::core::ptr::null_mut::<visible_ranges>(),
    );
    i = 0 as u_int;
    vis = 0 as u_int;
    while i < (*r).used {
        vis = vis.wrapping_add((*(*r).ranges.offset(i as isize)).nx);
        i = i.wrapping_add(1);
    }
    if vis < n {
        return 1 as ::core::ffi::c_int;
    }
    screen_write_set_cursor(
        ctx,
        cx.wrapping_sub(n) as ::core::ffi::c_int,
        cy as ::core::ffi::c_int,
    );
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    ttyctx.cell = &raw mut last;
    if force_wide != 0 {
        ttyctx.flags |= TTY_CTX_CELL_INVALIDATE;
    }
    if screen_write_should_draw_line(ctx, cy) != 0 {
        tty_write(
            Some(tty_cmd_cell as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
            &raw mut ttyctx,
        );
    }
    screen_write_set_cursor(ctx, cx as ::core::ffi::c_int, cy as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn screen_write_overwrite(
    mut ctx: *mut screen_write_ctx,
    mut gc: *mut grid_cell,
    mut width: u_int,
) -> ::core::ffi::c_int {
    let mut s: *mut screen = (*ctx).s;
    let mut gd: *mut grid = (*s).grid;
    let mut tmp_gc: grid_cell = grid_cell {
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
    let mut xx: u_int = 0;
    let mut done: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*gc).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0 {
        xx = (*s).cx.wrapping_add(1 as u_int);
        loop {
            xx = xx.wrapping_sub(1);
            if !(xx > 0 as u_int) {
                break;
            }
            grid_view_get_cell(gd, xx, (*s).cy, &raw mut tmp_gc);
            if !(tmp_gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
                break;
            }
            log_debug(
                b"%s: padding at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
                b"screen_write_overwrite\0" as *const u8 as *const ::core::ffi::c_char,
                xx,
                (*s).cy,
            );
            screen_write_clear_cell(gd, xx, (*s).cy);
        }
        log_debug(
            b"%s: character at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"screen_write_overwrite\0" as *const u8 as *const ::core::ffi::c_char,
            xx,
            (*s).cy,
        );
        screen_write_clear_cell(gd, xx, (*s).cy);
        done = 1 as ::core::ffi::c_int;
    }
    if width != 1 as u_int
        || (*gc).data.width as ::core::ffi::c_int != 1 as ::core::ffi::c_int
        || (*gc).flags as ::core::ffi::c_int & GRID_FLAG_PADDING != 0
    {
        xx = (*s).cx.wrapping_add(width).wrapping_sub(1 as u_int);
        loop {
            xx = xx.wrapping_add(1);
            if !(xx < (*(*s).grid).sx) {
                break;
            }
            grid_view_get_cell(gd, xx, (*s).cy, &raw mut tmp_gc);
            if !(tmp_gc.flags as ::core::ffi::c_int) & GRID_FLAG_PADDING != 0 {
                break;
            }
            log_debug(
                b"%s: overwrite at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
                b"screen_write_overwrite\0" as *const u8 as *const ::core::ffi::c_char,
                xx,
                (*s).cy,
            );
            screen_write_clear_cell(gd, xx, (*s).cy);
            done = 1 as ::core::ffi::c_int;
        }
    }
    return done;
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_setselection(
    mut ctx: *mut screen_write_ctx,
    mut clip: *const ::core::ffi::c_char,
    mut str: *mut u_char,
    mut len: u_int,
) {
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    ttyctx.c2rust_unnamed.sel.clip = clip;
    ttyctx.c2rust_unnamed.sel.data = str as *const ::core::ffi::c_char;
    ttyctx.c2rust_unnamed.sel.size = len as size_t;
    tty_write(
        Some(tty_cmd_setselection as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
        &raw mut ttyctx,
    );
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_rawstring(
    mut ctx: *mut screen_write_ctx,
    mut str: *mut u_char,
    mut len: u_int,
    mut allow_invisible_panes: ::core::ffi::c_int,
) {
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if allow_invisible_panes != 0 {
        ttyctx.flags |= TTY_CTX_INVISIBLE_PANES;
    }
    ttyctx.c2rust_unnamed.data.data = str as *const ::core::ffi::c_char;
    ttyctx.c2rust_unnamed.data.size = len as size_t;
    tty_write(
        Some(tty_cmd_rawstring as unsafe extern "C" fn(*mut tty, *const tty_ctx) -> ()),
        &raw mut ttyctx,
    );
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_alternateon(
    mut ctx: *mut screen_write_ctx,
    mut gc: *mut grid_cell,
    mut cursor: ::core::ffi::c_int,
) {
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    if !wp.is_null()
        && options_get_number(
            (*wp).options,
            b"alternate-screen\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0
    {
        return;
    }
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_alternateon\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if screen_alternate_on((*ctx).s, gc, cursor) == 0 {
        return;
    }
    if !wp.is_null() {
        window_pane_clear_resizes(wp, ::core::ptr::null_mut::<window_pane_resize>());
        if event_initialized(&raw mut (*wp).resize_timer) != 0 {
            event_del(&raw mut (*wp).resize_timer);
        }
        layout_fix_panes(
            (*wp).window as *mut window,
            ::core::ptr::null_mut::<window_pane>(),
        );
        if !(*wp).resize_queue.tqh_first.is_null() {
            window_pane_send_resize(wp, (*wp).sx, (*wp).sy);
            window_pane_clear_resizes(wp, ::core::ptr::null_mut::<window_pane_resize>());
        }
        server_redraw_window_borders((*wp).window as *mut window);
    }
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if ttyctx.redraw_cb.is_some() {
        ttyctx.redraw_cb.expect("non-null function pointer")(&raw mut ttyctx);
    }
}
#[no_mangle]
pub unsafe extern "C" fn screen_write_alternateoff(
    mut ctx: *mut screen_write_ctx,
    mut gc: *mut grid_cell,
    mut cursor: ::core::ffi::c_int,
) {
    let mut ttyctx: tty_ctx = tty_ctx {
        s: ::core::ptr::null_mut::<screen>(),
        redraw_cb: None,
        set_client_cb: None,
        arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cell: ::core::ptr::null::<grid_cell>(),
        flags: 0,
        c2rust_unnamed: C2RustUnnamed_38 { n: 0 },
        ocx: 0,
        ocy: 0,
        orupper: 0,
        orlower: 0,
        xoff: 0,
        yoff: 0,
        rxoff: 0,
        ryoff: 0,
        sx: 0,
        sy: 0,
        bg: 0,
        defaults: grid_cell {
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
        style_ctx: tty_style_ctx {
            defaults: ::core::ptr::null::<grid_cell>(),
            palette: ::core::ptr::null_mut::<colour_palette>(),
            dim: 0,
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
        },
        wox: 0,
        woy: 0,
        wsx: 0,
        wsy: 0,
    };
    let mut wp: *mut window_pane = (*ctx).wp as *mut window_pane;
    if !wp.is_null()
        && options_get_number(
            (*wp).options,
            b"alternate-screen\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0
    {
        return;
    }
    screen_write_collect_flush(
        ctx,
        0 as ::core::ffi::c_int,
        b"screen_write_alternateoff\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if screen_alternate_off((*ctx).s, gc, cursor) == 0 {
        return;
    }
    if !wp.is_null() {
        layout_fix_panes(
            (*wp).window as *mut window,
            ::core::ptr::null_mut::<window_pane>(),
        );
        server_redraw_window_borders((*wp).window as *mut window);
    }
    screen_write_initctx(
        ctx,
        &raw mut ttyctx,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if ttyctx.redraw_cb.is_some() {
        ttyctx.redraw_cb.expect("non-null function pointer")(&raw mut ttyctx);
    }
}

unsafe extern "C" fn run_static_initializers() {
    screen_write_citem_freelist = C2RustUnnamed_41 {
        tqh_first: ::core::ptr::null_mut::<screen_write_citem>(),
        tqh_last: &raw mut screen_write_citem_freelist.tqh_first,
    };
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [run_static_initializers];

pub use crate::src::shared::posix_terminal::VERASE;
pub use crate::src::shared::abi::{ssize_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::tty::{
    TTY_ALL_REQUEST_FLAGS, TTY_BRACKETPASTE, TTY_HAVEDA, TTY_HAVEDA2, TTY_HAVESYNC, TTY_HAVEXDA,
    TTY_OSC52QUERY, TTY_TIMER, TTY_WAITBG, TTY_WAITFG, TTY_WINSIZEQUERY,
};
pub use crate::src::shared::event::{EV_TIMEOUT};
pub use crate::src::shared::client::{CLIENT_FOCUSED};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{
    MOUSE_MASK_BUTTONS, MOUSE_PARAM_BTN_OFF, MOUSE_PARAM_POS_OFF, MOUSE_WHEEL_DOWN,
    MOUSE_WHEEL_UP, mouse_event,
};
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
    pub type tty_code;
    pub type format_job_tree;
    pub type control_state;
    pub type cmdq_list;
    pub type options_array_item;
    pub type options_entry;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn sscanf(
        __s: *const ::core::ffi::c_char,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn __b64_pton(
        _: *const ::core::ffi::c_char,
        _: *mut ::core::ffi::c_uchar,
        _: size_t,
    ) -> ::core::ffi::c_int;
    fn strtol(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long;
    fn strtoul(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_ulong;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strcspn(
        __s: *const ::core::ffi::c_char,
        __reject: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_ulong;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strsep(
        __stringp: *mut *mut ::core::ffi::c_char,
        __delim: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
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
    fn evbuffer_get_length(buf: *const evbuffer) -> size_t;
    fn evbuffer_drain(buf: *mut evbuffer, len: size_t) -> ::core::ffi::c_int;
    fn evbuffer_pullup(buf: *mut evbuffer, size: ssize_t) -> *mut ::core::ffi::c_uchar;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut global_options: *mut options;
    fn paste_add(_: *const ::core::ffi::c_char, _: *mut ::core::ffi::c_char, _: size_t);
    fn events_fire_client(_: *const ::core::ffi::c_char, _: *mut client);
    fn options_get(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_array_getv(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
        ...
    ) -> *mut options_value;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn tty_set_size(_: *mut tty, _: u_int, _: u_int, _: u_int, _: u_int);
    fn tty_invalidate(_: *mut tty);
    fn tty_update_features(_: *mut tty);
    fn tty_term_string(_: *mut tty_term, _: tty_code_code) -> *const ::core::ffi::c_char;
    fn tty_parse_client_features(
        _: *mut client,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
    );
    fn tty_default_features(_: *mut client, _: *const ::core::ffi::c_char, _: u_int);
    fn key_string_lookup_key(_: key_code, _: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn server_client_handle_key(_: *mut client, _: *mut key_event) -> ::core::ffi::c_int;
    fn server_client_update_theme_colours(_: *mut client);
    fn input_request_reply(_: *mut client, _: input_request_type, _: *mut ::core::ffi::c_void);
    fn colour_tostring(_: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn colour_parseX11(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn window_update_focus(_: *mut window);
    fn session_theme_changed(_: *mut session);
    fn utf8_fromwc(wc: wchar_t, _: *mut utf8_data) -> utf8_state;
    fn utf8_from_data(_: *const utf8_data, _: *mut utf8_char) -> utf8_state;
    fn utf8_open(_: *mut utf8_data, _: u_char) -> utf8_state;
    fn utf8_append(_: *mut utf8_data, _: u_char) -> utf8_state;
    fn log_get_level() -> ::core::ffi::c_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
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
pub struct tty_key {
    pub ch: ::core::ffi::c_char,
    pub key: key_code,
    pub left: *mut tty_key,
    pub right: *mut tty_key,
    pub next: *mut tty_key,
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
pub type C2RustUnnamed_36 = ::core::ffi::c_uint;
pub const C0_US: C2RustUnnamed_36 = 31;
pub const C0_RS: C2RustUnnamed_36 = 30;
pub const C0_GS: C2RustUnnamed_36 = 29;
pub const C0_FS: C2RustUnnamed_36 = 28;
pub const C0_ESC: C2RustUnnamed_36 = 27;
pub const C0_SUB: C2RustUnnamed_36 = 26;
pub const C0_EM: C2RustUnnamed_36 = 25;
pub const C0_CAN: C2RustUnnamed_36 = 24;
pub const C0_ETB: C2RustUnnamed_36 = 23;
pub const C0_SYN: C2RustUnnamed_36 = 22;
pub const C0_NAK: C2RustUnnamed_36 = 21;
pub const C0_DC4: C2RustUnnamed_36 = 20;
pub const C0_DC3: C2RustUnnamed_36 = 19;
pub const C0_DC2: C2RustUnnamed_36 = 18;
pub const C0_DC1: C2RustUnnamed_36 = 17;
pub const C0_DLE: C2RustUnnamed_36 = 16;
pub const C0_SI: C2RustUnnamed_36 = 15;
pub const C0_SO: C2RustUnnamed_36 = 14;
pub const C0_CR: C2RustUnnamed_36 = 13;
pub const C0_FF: C2RustUnnamed_36 = 12;
pub const C0_VT: C2RustUnnamed_36 = 11;
pub const C0_LF: C2RustUnnamed_36 = 10;
pub const C0_HT: C2RustUnnamed_36 = 9;
pub const C0_BS: C2RustUnnamed_36 = 8;
pub const C0_BEL: C2RustUnnamed_36 = 7;
pub const C0_ASC: C2RustUnnamed_36 = 6;
pub const C0_ENQ: C2RustUnnamed_36 = 5;
pub const C0_EOT: C2RustUnnamed_36 = 4;
pub const C0_ETX: C2RustUnnamed_36 = 3;
pub const C0_STX: C2RustUnnamed_36 = 2;
pub const C0_SOH: C2RustUnnamed_36 = 1;
pub const C0_NUL: C2RustUnnamed_36 = 0;
pub type C2RustUnnamed_37 = ::core::ffi::c_ulong;
pub type input_request_type = ::core::ffi::c_uint;
pub const INPUT_REQUEST_QUEUE: input_request_type = 2;
pub const INPUT_REQUEST_CLIPBOARD: input_request_type = 1;
pub const INPUT_REQUEST_PALETTE: input_request_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_request_palette_data {
    pub idx: ::core::ffi::c_int,
    pub c: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_request_clipboard_data {
    pub buf: *mut ::core::ffi::c_char,
    pub len: size_t,
    pub clip: ::core::ffi::c_char,
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
pub struct tty_default_key_code {
    pub code: tty_code_code,
    pub key: key_code,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_default_key_raw {
    pub string: *const ::core::ffi::c_char,
    pub key: key_code,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tty_default_key_xterm {
    pub template: *const ::core::ffi::c_char,
    pub key: key_code,
}
pub const _POSIX_VDISABLE: ::core::ffi::c_int = '\0' as i32;

static mut tty_default_raw_keys: [tty_default_key_raw; 102] = [
    tty_default_key_raw {
        string: b"\x1BO[\0" as *const u8 as *const ::core::ffi::c_char,
        key: '\u{1b}' as i32 as key_code,
    },
    tty_default_key_raw {
        string: b"\x1BOo\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_SLASH as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: b"\x1BOj\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_STAR as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: b"\x1BOm\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_MINUS as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: b"\x1BOw\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_SEVEN as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: b"\x1BOx\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_EIGHT as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: b"\x1BOy\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_NINE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: b"\x1BOk\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_PLUS as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: b"\x1BOt\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_FOUR as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: b"\x1BOu\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_FIVE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: b"\x1BOv\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_SIX as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: b"\x1BOq\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_ONE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: b"\x1BOr\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_TWO as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: b"\x1BOs\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_THREE as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: b"\x1BOM\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_ENTER as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: b"\x1BOp\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_ZERO as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: b"\x1BOn\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_KP_PERIOD as ::core::ffi::c_ulong as key_code | KEYC_KEYPAD,
    },
    tty_default_key_raw {
        string: b"\x1BOA\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: b"\x1BOB\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: b"\x1BOC\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: b"\x1BOD\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: b"\x1B[A\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: b"\x1B[B\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: b"\x1B[C\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: b"\x1B[D\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_raw {
        string: b"\x1B\x1BOA\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: b"\x1B\x1BOB\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: b"\x1B\x1BOC\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: b"\x1B\x1BOD\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: b"\x1B\x1B[A\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: b"\x1B\x1B[B\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: b"\x1B\x1B[C\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: b"\x1B\x1B[D\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR | KEYC_META,
    },
    tty_default_key_raw {
        string: b"\x1BOH\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1BOF\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_END as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1B\x1BOH\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: b"\x1B\x1BOF\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_END as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: b"\x1B[H\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1B[F\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_END as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1B\x1B[H\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: b"\x1B\x1B[F\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_END as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: b"\x1BOa\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: b"\x1BOb\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: b"\x1BOc\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: b"\x1BOd\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: b"\x1B[a\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[b\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[c\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[d\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[11~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1B[12~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1B[13~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1B[14~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1B[15~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1B[17~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1B[18~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1B[19~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1B[20~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1B[21~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1B[23~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[24~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[25~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[26~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[28~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[29~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[31~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[32~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[33~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[34~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[23$\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[24$\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[11^\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: b"\x1B[12^\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: b"\x1B[13^\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: b"\x1B[14^\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: b"\x1B[15^\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: b"\x1B[17^\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: b"\x1B[18^\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: b"\x1B[19^\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: b"\x1B[20^\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: b"\x1B[21^\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: b"\x1B[23^\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: b"\x1B[24^\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_raw {
        string: b"\x1B[11@\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[12@\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[13@\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[14@\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[15@\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[17@\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[18@\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[19@\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[20@\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[21@\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[23@\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[24@\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[I\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_FOCUS_IN as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1B[O\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_FOCUS_OUT as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1B[200~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_PASTE_START as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: b"\x1B[201~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_PASTE_END as ::core::ffi::c_ulong as key_code | KEYC_IMPLIED_META,
    },
    tty_default_key_raw {
        string: b"\x1B[1;5Z\0" as *const u8 as *const ::core::ffi::c_char,
        key: '\t' as i32 as key_code | KEYC_CTRL | KEYC_SHIFT,
    },
    tty_default_key_raw {
        string: b"\x1B[?997;1n\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_REPORT_DARK_THEME as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_raw {
        string: b"\x1B[?997;2n\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_REPORT_LIGHT_THEME as ::core::ffi::c_ulong as key_code,
    },
];
static mut tty_default_xterm_keys: [tty_default_key_xterm; 30] = [
    tty_default_key_xterm {
        template: b"\x1B[1;_P\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1BO1;_P\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1BO_P\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[1;_Q\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1BO1;_Q\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1BO_Q\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[1;_R\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1BO1;_R\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1BO_R\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[1;_S\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1BO1;_S\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1BO_S\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[15;_~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[17;_~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[18;_~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[19;_~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[20;_~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[21;_~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[23;_~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[24;_~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[1;_A\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[1;_B\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[1;_C\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[1;_D\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[1;_H\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[1;_F\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_END as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[5;_~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[6;_~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[2;_~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_xterm {
        template: b"\x1B[3;_~\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code,
    },
];
static mut tty_default_xterm_modifiers: [key_code; 10] = [
    0 as ::core::ffi::c_int as key_code,
    0 as ::core::ffi::c_int as key_code,
    KEYC_SHIFT,
    KEYC_META | KEYC_IMPLIED_META,
    KEYC_SHIFT | KEYC_META | KEYC_IMPLIED_META,
    KEYC_CTRL,
    KEYC_SHIFT | KEYC_CTRL,
    KEYC_META | KEYC_IMPLIED_META | KEYC_CTRL,
    KEYC_SHIFT | KEYC_META | KEYC_IMPLIED_META | KEYC_CTRL,
    KEYC_META | KEYC_IMPLIED_META,
];
static mut tty_default_code_keys: [tty_default_key_code; 136] = [
    tty_default_key_code {
        code: TTYC_KF1,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF2,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF3,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF4,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF5,
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF6,
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF7,
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF8,
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF9,
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF10,
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF11,
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF12,
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KF13,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF14,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF15,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF16,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF17,
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF18,
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF19,
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF20,
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF21,
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF22,
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF23,
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF24,
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF25,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF26,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF27,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF28,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF29,
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF30,
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF31,
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF32,
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF33,
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF34,
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF35,
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF36,
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF37,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF38,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF39,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF40,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF41,
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF42,
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF43,
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF44,
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF45,
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF46,
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF47,
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF48,
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KF49,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF50,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF51,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF52,
        key: KEYC_F4 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF53,
        key: KEYC_F5 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF54,
        key: KEYC_F6 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF55,
        key: KEYC_F7 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF56,
        key: KEYC_F8 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF57,
        key: KEYC_F9 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF58,
        key: KEYC_F10 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF59,
        key: KEYC_F11 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF60,
        key: KEYC_F12 as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KF61,
        key: KEYC_F1 as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF62,
        key: KEYC_F2 as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KF63,
        key: KEYC_F3 as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KICH1,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KDCH1,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KHOME,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KEND,
        key: KEYC_END as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KNP,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KPP,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KCBT,
        key: KEYC_BTAB as ::core::ffi::c_ulong as key_code,
    },
    tty_default_key_code {
        code: TTYC_KCUU1,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_code {
        code: TTYC_KCUD1,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_code {
        code: TTYC_KCUB1,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_code {
        code: TTYC_KCUF1,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CURSOR,
    },
    tty_default_key_code {
        code: TTYC_KDC2,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KDC3,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KDC4,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KDC5,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KDC6,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KDC7,
        key: KEYC_DC as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KIND,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KDN2,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KDN3,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KDN4,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KDN5,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KDN6,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KDN7,
        key: KEYC_DOWN as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KEND2,
        key: KEYC_END as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KEND3,
        key: KEYC_END as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KEND4,
        key: KEYC_END as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KEND5,
        key: KEYC_END as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KEND6,
        key: KEYC_END as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KEND7,
        key: KEYC_END as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KHOM2,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KHOM3,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KHOM4,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KHOM5,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KHOM6,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KHOM7,
        key: KEYC_HOME as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KIC2,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KIC3,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KIC4,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KIC5,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KIC6,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KIC7,
        key: KEYC_IC as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KLFT2,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KLFT3,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KLFT4,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KLFT5,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KLFT6,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KLFT7,
        key: KEYC_LEFT as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KNXT2,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KNXT3,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KNXT4,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KNXT5,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KNXT6,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KNXT7,
        key: KEYC_NPAGE as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KPRV2,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KPRV3,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KPRV4,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KPRV5,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KPRV6,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KPRV7,
        key: KEYC_PPAGE as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KRIT2,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KRIT3,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KRIT4,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KRIT5,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KRIT6,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KRIT7,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KRI,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KUP2,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_SHIFT,
    },
    tty_default_key_code {
        code: TTYC_KUP3,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_META | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KUP4,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code
            | KEYC_SHIFT
            | KEYC_META
            | KEYC_IMPLIED_META,
    },
    tty_default_key_code {
        code: TTYC_KUP5,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KUP6,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code | KEYC_SHIFT | KEYC_CTRL,
    },
    tty_default_key_code {
        code: TTYC_KUP7,
        key: KEYC_UP as ::core::ffi::c_ulong as key_code
            | KEYC_META
            | KEYC_IMPLIED_META
            | KEYC_CTRL,
    },
];
unsafe extern "C" fn tty_keys_add(
    mut tty: *mut tty,
    mut s: *const ::core::ffi::c_char,
    mut key: key_code,
) {
    let mut tk: *mut tty_key = ::core::ptr::null_mut::<tty_key>();
    let mut size: size_t = 0;
    let mut keystr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    keystr = key_string_lookup_key(key, 1 as ::core::ffi::c_int);
    tk = tty_keys_find(tty, s, strlen(s), &raw mut size);
    if tk.is_null() {
        log_debug(
            b"new key %s: 0x%llx (%s)\0" as *const u8 as *const ::core::ffi::c_char,
            s,
            key,
            keystr,
        );
        tty_keys_add1(&raw mut (*tty).key_tree, s, key);
    } else {
        log_debug(
            b"replacing key %s: 0x%llx (%s)\0" as *const u8 as *const ::core::ffi::c_char,
            s,
            key,
            keystr,
        );
        (*tk).key = key;
    };
}
unsafe extern "C" fn tty_keys_add1(
    mut tkp: *mut *mut tty_key,
    mut s: *const ::core::ffi::c_char,
    mut key: key_code,
) {
    let mut tk: *mut tty_key = ::core::ptr::null_mut::<tty_key>();
    tk = *tkp;
    if tk.is_null() {
        *tkp = xcalloc(1 as size_t, ::core::mem::size_of::<tty_key>() as size_t) as *mut tty_key;
        tk = *tkp;
        (*tk).ch = *s;
        (*tk).key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
    }
    if *s as ::core::ffi::c_int == (*tk).ch as ::core::ffi::c_int {
        s = s.offset(1);
        if *s as ::core::ffi::c_int == '\0' as i32 {
            (*tk).key = key;
            return;
        }
        tkp = &raw mut (*tk).next;
    } else if (*s as ::core::ffi::c_int) < (*tk).ch as ::core::ffi::c_int {
        tkp = &raw mut (*tk).left;
    } else if *s as ::core::ffi::c_int > (*tk).ch as ::core::ffi::c_int {
        tkp = &raw mut (*tk).right;
    }
    tty_keys_add1(tkp, s, key);
}
#[no_mangle]
pub unsafe extern "C" fn tty_keys_build(mut tty: *mut tty) {
    let mut tdkr: *const tty_default_key_raw = ::core::ptr::null::<tty_default_key_raw>();
    let mut tdkx: *const tty_default_key_xterm = ::core::ptr::null::<tty_default_key_xterm>();
    let mut tdkc: *const tty_default_key_code = ::core::ptr::null::<tty_default_key_code>();
    let mut i: u_int = 0;
    let mut j: u_int = 0;
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut copy: [::core::ffi::c_char; 16] = [0; 16];
    let mut key: key_code = 0;
    if !(*tty).key_tree.is_null() {
        tty_keys_free(tty);
    }
    (*tty).key_tree = ::core::ptr::null_mut::<tty_key>();
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[tty_default_key_xterm; 30]>() as usize)
            .wrapping_div(::core::mem::size_of::<tty_default_key_xterm>() as usize)
    {
        tdkx = (&raw const tty_default_xterm_keys as *const tty_default_key_xterm)
            .offset(i as isize) as *const tty_default_key_xterm;
        j = 2 as u_int;
        while (j as usize)
            < (::core::mem::size_of::<[key_code; 10]>() as usize)
                .wrapping_div(::core::mem::size_of::<key_code>() as usize)
        {
            strlcpy(
                &raw mut copy as *mut ::core::ffi::c_char,
                (*tdkx).template,
                ::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t,
            );
            copy[strcspn(
                &raw mut copy as *mut ::core::ffi::c_char,
                b"_\0" as *const u8 as *const ::core::ffi::c_char,
            ) as usize] = ('0' as i32 as u_int).wrapping_add(j) as ::core::ffi::c_char;
            key = (*tdkx).key | tty_default_xterm_modifiers[j as usize];
            tty_keys_add(tty, &raw mut copy as *mut ::core::ffi::c_char, key);
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[tty_default_key_raw; 102]>() as usize)
            .wrapping_div(::core::mem::size_of::<tty_default_key_raw>() as usize)
    {
        tdkr = (&raw const tty_default_raw_keys as *const tty_default_key_raw).offset(i as isize)
            as *const tty_default_key_raw;
        s = (*tdkr).string;
        if *s as ::core::ffi::c_int != '\0' as i32 {
            tty_keys_add(tty, s, (*tdkr).key);
        }
        i = i.wrapping_add(1);
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[tty_default_key_code; 136]>() as usize)
            .wrapping_div(::core::mem::size_of::<tty_default_key_code>() as usize)
    {
        tdkc = (&raw const tty_default_code_keys as *const tty_default_key_code).offset(i as isize)
            as *const tty_default_key_code;
        s = tty_term_string((*tty).term, (*tdkc).code);
        if *s as ::core::ffi::c_int != '\0' as i32 {
            tty_keys_add(tty, s, (*tdkc).key);
        }
        i = i.wrapping_add(1);
    }
    o = options_get(
        global_options,
        b"user-keys\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if !o.is_null() {
        i = 0 as u_int;
        while i <= KEYC_NUSER as u_int {
            ov = options_array_getv(o, b"%u\0" as *const u8 as *const ::core::ffi::c_char, i);
            if !ov.is_null() {
                tty_keys_add(
                    tty,
                    (*ov).string,
                    (KEYC_USER as ::core::ffi::c_ulong).wrapping_add(i as ::core::ffi::c_ulong)
                        as key_code,
                );
            }
            i = i.wrapping_add(1);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn tty_keys_free(mut tty: *mut tty) {
    tty_keys_free1((*tty).key_tree);
}
unsafe extern "C" fn tty_keys_free1(mut tk: *mut tty_key) {
    if !(*tk).next.is_null() {
        tty_keys_free1((*tk).next);
    }
    if !(*tk).left.is_null() {
        tty_keys_free1((*tk).left);
    }
    if !(*tk).right.is_null() {
        tty_keys_free1((*tk).right);
    }
    free(tk as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn tty_keys_find(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> *mut tty_key {
    *size = 0 as size_t;
    return tty_keys_find1((*tty).key_tree, buf, len, size);
}
unsafe extern "C" fn tty_keys_find1(
    mut tk: *mut tty_key,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> *mut tty_key {
    if len == 0 as size_t {
        return ::core::ptr::null_mut::<tty_key>();
    }
    if tk.is_null() {
        return ::core::ptr::null_mut::<tty_key>();
    }
    if (*tk).ch as ::core::ffi::c_int == *buf as ::core::ffi::c_int {
        buf = buf.offset(1);
        len = len.wrapping_sub(1);
        *size = (*size).wrapping_add(1);
        if len == 0 as size_t
            || (*tk).next.is_null() && (*tk).key != KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code
        {
            return tk;
        }
        tk = (*tk).next;
    } else if (*buf as ::core::ffi::c_int) < (*tk).ch as ::core::ffi::c_int {
        tk = (*tk).left;
    } else if *buf as ::core::ffi::c_int > (*tk).ch as ::core::ffi::c_int {
        tk = (*tk).right;
    }
    return tty_keys_find1(tk, buf, len, size);
}
unsafe extern "C" fn tty_keys_partial_paste_end(
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    static mut paste_end: [::core::ffi::c_char; 7] =
        unsafe { ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"\x1B[201~\0") };
    let mut paste_end_len: size_t =
        (::core::mem::size_of::<[::core::ffi::c_char; 7]>() as size_t).wrapping_sub(1 as size_t);
    if len == 0 as size_t || len >= paste_end_len {
        return 0 as ::core::ffi::c_int;
    }
    return (memcmp(
        buf as *const ::core::ffi::c_void,
        &raw const paste_end as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        len,
    ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_keys_next1(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut key: *mut key_code,
    mut size: *mut size_t,
    mut expired: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut tk: *mut tty_key = ::core::ptr::null_mut::<tty_key>();
    let mut tk1: *mut tty_key = ::core::ptr::null_mut::<tty_key>();
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut more: utf8_state = UTF8_MORE;
    let mut uc: utf8_char = 0;
    let mut i: u_int = 0;
    log_debug(
        b"%s: next key is %zu (%.*s) (expired=%d)\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        len,
        len as ::core::ffi::c_int,
        buf,
        expired,
    );
    tk = tty_keys_find(tty, buf, len, size);
    if !tk.is_null() && (*tk).key != KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
        tk1 = tk;
        loop {
            log_debug(
                b"%s: keys in list: %#llx\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
                (*tk1).key,
            );
            tk1 = (*tk1).next;
            if tk1.is_null() {
                break;
            }
        }
        if !(*tk).next.is_null() && expired == 0 {
            return 1 as ::core::ffi::c_int;
        }
        *key = (*tk).key;
        if *key & KEYC_MASK_KEY
            == KEYC_PASTE_START as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        {
            (*tty).flags |= TTY_BRACKETPASTE;
        } else if *key & KEYC_MASK_KEY
            == KEYC_PASTE_END as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        {
            (*tty).flags &= !TTY_BRACKETPASTE;
        }
        return 0 as ::core::ffi::c_int;
    }
    more = utf8_open(&raw mut ud, *buf as u_char);
    if more as ::core::ffi::c_uint == UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint {
        *size = ud.size as size_t;
        if len < ud.size as size_t {
            if expired == 0 {
                return 1 as ::core::ffi::c_int;
            }
            return -(1 as ::core::ffi::c_int);
        }
        i = 1 as u_int;
        while i < ud.size as u_int {
            more = utf8_append(&raw mut ud, *buf.offset(i as isize) as u_char);
            i = i.wrapping_add(1);
        }
        if more as ::core::ffi::c_uint != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint {
            return -(1 as ::core::ffi::c_int);
        }
        if utf8_from_data(&raw mut ud, &raw mut uc) as ::core::ffi::c_uint
            != UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return -(1 as ::core::ffi::c_int);
        }
        *key = uc as key_code;
        log_debug(
            b"%s: UTF-8 key %.*s %#llx\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            ud.size as ::core::ffi::c_int,
            &raw mut ud.data as *mut u_char,
            *key,
        );
        return 0 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn tty_keys_winsz(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut end: size_t = 0;
    let mut tmp: [::core::ffi::c_char; 64] = [0; 64];
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xpixel: u_int = 0;
    let mut ypixel: u_int = 0;
    let mut char_x: u_int = 0;
    let mut char_y: u_int = 0;
    *size = 0 as size_t;
    if (*tty).flags & TTY_WINSIZEQUERY == 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '[' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    end = 2 as size_t;
    while end < len && end != ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as usize {
        if *buf.offset(end as isize) as ::core::ffi::c_int == 't' as i32 {
            break;
        }
        if *(*__ctype_b_loc())
            .offset(*buf.offset(end as isize) as u_char as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            == 0
            && *buf.offset(end as isize) as ::core::ffi::c_int != ';' as i32
        {
            break;
        }
        end = end.wrapping_add(1);
    }
    if end == len {
        return 1 as ::core::ffi::c_int;
    }
    if end == ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as usize
        || *buf.offset(end as isize) as ::core::ffi::c_int != 't' as i32
    {
        return -(1 as ::core::ffi::c_int);
    }
    memcpy(
        &raw mut tmp as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        buf.offset(2 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        end.wrapping_sub(2 as size_t),
    );
    tmp[end.wrapping_sub(2 as size_t) as usize] = '\0' as i32 as ::core::ffi::c_char;
    if sscanf(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"8;%u;%u\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut sy,
        &raw mut sx,
    ) == 2 as ::core::ffi::c_int
    {
        tty_set_size(tty, sx, sy, (*tty).xpixel, (*tty).ypixel);
        *size = end.wrapping_add(1 as size_t);
        return 0 as ::core::ffi::c_int;
    } else if sscanf(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"4;%u;%u\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut ypixel,
        &raw mut xpixel,
    ) == 2 as ::core::ffi::c_int
    {
        char_x = if xpixel != 0 && (*tty).sx != 0 {
            xpixel.wrapping_div((*tty).sx)
        } else {
            0 as u_int
        };
        char_y = if ypixel != 0 && (*tty).sy != 0 {
            ypixel.wrapping_div((*tty).sy)
        } else {
            0 as u_int
        };
        tty_set_size(tty, (*tty).sx, (*tty).sy, char_x, char_y);
        tty_invalidate(tty);
        (*tty).flags &= !TTY_WINSIZEQUERY;
        *size = end.wrapping_add(1 as size_t);
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: unrecognized window size sequence: %s\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        &raw mut tmp as *mut ::core::ffi::c_char,
    );
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn tty_keys_next(mut tty: *mut tty) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut c: *mut client = (*tty).client;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut buf: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut size: size_t = 0;
    let mut bspace: cc_t = 0;
    let mut delay: ::core::ffi::c_int = 0;
    let mut expired: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = 0;
    let mut bg: ::core::ffi::c_int = (*tty).bg;
    let mut key: key_code = 0;
    let mut onlykey: key_code = 0;
    let mut m: mouse_event = mouse_event {
        valid: 0 as ::core::ffi::c_int,
        ignore: 0,
        key: 0,
        statusat: 0,
        statuslines: 0,
        x: 0,
        y: 0,
        b: 0,
        lx: 0,
        ly: 0,
        lb: 0,
        ox: 0,
        oy: 0,
        s: 0,
        w: 0,
        wp: 0,
        sgr_type: 0,
        sgr_b: 0,
    };
    let mut event: *mut key_event = ::core::ptr::null_mut::<key_event>();
    buf = evbuffer_pullup((*tty).in_0, -(1 as ::core::ffi::c_int) as ssize_t)
        as *const ::core::ffi::c_char;
    len = evbuffer_get_length((*tty).in_0);
    if len == 0 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: keys are %zu (%.*s)\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        len,
        len as ::core::ffi::c_int,
        buf,
    );
    match tty_keys_clipboard(tty, buf, len, &raw mut size) {
        0 => {
            key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
            current_block = 5025795842197473417;
        }
        -1 => {
            current_block = 1917311967535052937;
        }
        1 => {
            current_block = 16977559109335092698;
        }
        _ => {
            current_block = 1917311967535052937;
        }
    }
    match current_block {
        1917311967535052937 => {
            match tty_keys_sync(tty, buf, len, &raw mut size) {
                0 => {
                    key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                    current_block = 5025795842197473417;
                }
                -1 => {
                    current_block = 4166486009154926805;
                }
                1 => {
                    current_block = 16977559109335092698;
                }
                _ => {
                    current_block = 4166486009154926805;
                }
            }
            match current_block {
                5025795842197473417 => {}
                16977559109335092698 => {}
                _ => {
                    match tty_keys_device_attributes(tty, buf, len, &raw mut size) {
                        0 => {
                            key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                            current_block = 5025795842197473417;
                        }
                        -1 => {
                            current_block = 15652330335145281839;
                        }
                        1 => {
                            current_block = 16977559109335092698;
                        }
                        _ => {
                            current_block = 15652330335145281839;
                        }
                    }
                    match current_block {
                        5025795842197473417 => {}
                        16977559109335092698 => {}
                        _ => {
                            match tty_keys_device_attributes2(tty, buf, len, &raw mut size) {
                                0 => {
                                    key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                                    current_block = 5025795842197473417;
                                }
                                -1 => {
                                    current_block = 224731115979188411;
                                }
                                1 => {
                                    current_block = 16977559109335092698;
                                }
                                _ => {
                                    current_block = 224731115979188411;
                                }
                            }
                            match current_block {
                                5025795842197473417 => {}
                                16977559109335092698 => {}
                                _ => {
                                    match tty_keys_extended_device_attributes(
                                        tty,
                                        buf,
                                        len,
                                        &raw mut size,
                                    ) {
                                        0 => {
                                            key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                                            current_block = 5025795842197473417;
                                        }
                                        -1 => {
                                            current_block = 17478428563724192186;
                                        }
                                        1 => {
                                            current_block = 16977559109335092698;
                                        }
                                        _ => {
                                            current_block = 17478428563724192186;
                                        }
                                    }
                                    match current_block {
                                        5025795842197473417 => {}
                                        16977559109335092698 => {}
                                        _ => {
                                            match tty_keys_colours(
                                                tty,
                                                buf,
                                                len,
                                                &raw mut size,
                                                &raw mut (*tty).fg,
                                                &raw mut (*tty).bg,
                                            ) {
                                                0 => {
                                                    key = KEYC_UNKNOWN as ::core::ffi::c_ulong
                                                        as key_code;
                                                    if (*tty).bg != bg {
                                                        server_client_update_theme_colours(c);
                                                    }
                                                    session_theme_changed((*c).session);
                                                    current_block = 5025795842197473417;
                                                }
                                                -1 => {
                                                    current_block = 1538046216550696469;
                                                }
                                                1 => {
                                                    if (*tty).bg != bg {
                                                        server_client_update_theme_colours(c);
                                                    }
                                                    session_theme_changed((*c).session);
                                                    current_block = 16977559109335092698;
                                                }
                                                _ => {
                                                    current_block = 1538046216550696469;
                                                }
                                            }
                                            match current_block {
                                                16977559109335092698 => {}
                                                5025795842197473417 => {}
                                                _ => {
                                                    match tty_keys_palette(
                                                        tty,
                                                        buf,
                                                        len,
                                                        &raw mut size,
                                                    ) {
                                                        0 => {
                                                            key = KEYC_UNKNOWN
                                                                as ::core::ffi::c_ulong
                                                                as key_code;
                                                            current_block = 5025795842197473417;
                                                        }
                                                        -1 => {
                                                            current_block = 1836292691772056875;
                                                        }
                                                        1 => {
                                                            current_block = 16977559109335092698;
                                                        }
                                                        _ => {
                                                            current_block = 1836292691772056875;
                                                        }
                                                    }
                                                    match current_block {
                                                        5025795842197473417 => {}
                                                        16977559109335092698 => {}
                                                        _ => {
                                                            match tty_keys_mouse(
                                                                tty,
                                                                buf,
                                                                len,
                                                                &raw mut size,
                                                                &raw mut m,
                                                            ) {
                                                                0 => {
                                                                    key = KEYC_MOUSE
                                                                        as ::core::ffi::c_ulong
                                                                        as key_code;
                                                                    current_block =
                                                                        5025795842197473417;
                                                                }
                                                                -1 => {
                                                                    current_block =
                                                                        17784502470059252271;
                                                                }
                                                                -2 => {
                                                                    key = KEYC_MOUSE
                                                                        as ::core::ffi::c_ulong
                                                                        as key_code;
                                                                    log_debug(
                                                                        b"%s: discard key %.*s %#llx\0" as *const u8
                                                                            as *const ::core::ffi::c_char,
                                                                        (*c).name,
                                                                        size as ::core::ffi::c_int,
                                                                        buf,
                                                                        key,
                                                                    );
                                                                    evbuffer_drain(
                                                                        (*tty).in_0,
                                                                        size,
                                                                    );
                                                                    return 1 as ::core::ffi::c_int;
                                                                }
                                                                1 => {
                                                                    current_block =
                                                                        16977559109335092698;
                                                                }
                                                                _ => {
                                                                    current_block =
                                                                        17784502470059252271;
                                                                }
                                                            }
                                                            match current_block {
                                                                5025795842197473417 => {}
                                                                16977559109335092698 => {}
                                                                _ => {
                                                                    match tty_keys_extended_key(
                                                                        tty,
                                                                        buf,
                                                                        len,
                                                                        &raw mut size,
                                                                        &raw mut key,
                                                                    ) {
                                                                        0 => {
                                                                            current_block =
                                                                                5025795842197473417;
                                                                        }
                                                                        -1 => {
                                                                            current_block =
                                                                                3938820862080741272;
                                                                        }
                                                                        1 => {
                                                                            current_block = 16977559109335092698;
                                                                        }
                                                                        _ => {
                                                                            current_block =
                                                                                3938820862080741272;
                                                                        }
                                                                    }
                                                                    match current_block {
                                                                        5025795842197473417 => {}
                                                                        16977559109335092698 => {}
                                                                        _ => {
                                                                            match tty_keys_winsz(
                                                                                tty,
                                                                                buf,
                                                                                len,
                                                                                &raw mut size,
                                                                            ) {
                                                                                0 => {
                                                                                    current_block = 1414802762261447502;
                                                                                    match current_block {
                                                                                        14661562966503102838 => {
                                                                                            current_block = 16977559109335092698;
                                                                                        }
                                                                                        _ => {
                                                                                            key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                                                                                            current_block = 5025795842197473417;
                                                                                        }
                                                                                    }
                                                                                }
                                                                                1 => {
                                                                                    current_block = 14661562966503102838;
                                                                                    match current_block {
                                                                                        14661562966503102838 => {
                                                                                            current_block = 16977559109335092698;
                                                                                        }
                                                                                        _ => {
                                                                                            key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                                                                                            current_block = 5025795842197473417;
                                                                                        }
                                                                                    }
                                                                                }
                                                                                -1 | _ => {
                                                                                    current_block = 12077302897653652224;
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
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    loop {
        match current_block {
            12077302897653652224 => {
                n = tty_keys_next1(tty, buf, len, &raw mut key, &raw mut size, expired);
                if n == 0 as ::core::ffi::c_int {
                    current_block = 5025795842197473417;
                    continue;
                }
                if n == 1 as ::core::ffi::c_int {
                    current_block = 16977559109335092698;
                    continue;
                }
                if *buf as ::core::ffi::c_int == '\u{1b}' as i32 && len > 1 as size_t {
                    n = tty_keys_next1(
                        tty,
                        buf.offset(1 as ::core::ffi::c_int as isize),
                        len.wrapping_sub(1 as size_t),
                        &raw mut key,
                        &raw mut size,
                        expired,
                    );
                    if n == 0 as ::core::ffi::c_int {
                        if key as ::core::ffi::c_ulonglong & KEYC_IMPLIED_META != 0 {
                            key = '\u{1b}' as i32 as key_code;
                            size = 1 as size_t;
                            current_block = 5025795842197473417;
                            continue;
                        } else {
                            key |= KEYC_META;
                            size = size.wrapping_add(1);
                            current_block = 5025795842197473417;
                            continue;
                        }
                    } else if n == 1 as ::core::ffi::c_int {
                        current_block = 16977559109335092698;
                        continue;
                    }
                }
                if *buf as ::core::ffi::c_int == '\u{1b}' as i32 && len >= 2 as size_t {
                    key = (*buf.offset(1 as ::core::ffi::c_int as isize) as u_char
                        as ::core::ffi::c_ulonglong
                        | KEYC_META) as key_code;
                    size = 2 as size_t;
                } else {
                    key = *buf.offset(0 as ::core::ffi::c_int as isize) as u_char as key_code;
                    size = 1 as size_t;
                }
                if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == C0_NUL as ::core::ffi::c_int as ::core::ffi::c_ulonglong
                {
                    key = (' ' as i32 as ::core::ffi::c_ulonglong
                        | KEYC_CTRL
                        | key as ::core::ffi::c_ulonglong & KEYC_META)
                        as key_code;
                }
                bspace = (*tty).tio.c_cc[VERASE as usize];
                if bspace as ::core::ffi::c_int != _POSIX_VDISABLE {
                    if key == bspace as key_code {
                        log_debug(
                            b"%s: key %#llx is BSpace\0" as *const u8 as *const ::core::ffi::c_char,
                            (*c).name,
                            key,
                        );
                        key = KEYC_BSPACE as ::core::ffi::c_ulong as key_code;
                    }
                    if key == bspace as ::core::ffi::c_ulonglong | KEYC_META {
                        log_debug(
                            b"%s: key %#llx is M-BSpace\0" as *const u8
                                as *const ::core::ffi::c_char,
                            (*c).name,
                            key,
                        );
                        key = (KEYC_BSPACE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                            | KEYC_META) as key_code;
                    }
                }
                onlykey = (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY) as key_code;
                if onlykey < 0x20 as key_code
                    && onlykey != C0_HT as ::core::ffi::c_int as key_code
                    && onlykey != C0_CR as ::core::ffi::c_int as key_code
                    && onlykey != C0_ESC as ::core::ffi::c_int as key_code
                {
                    onlykey |= 0x40 as key_code;
                    if onlykey >= 'A' as i32 as key_code && onlykey <= 'Z' as i32 as key_code {
                        onlykey |= 0x20 as key_code;
                    }
                    key = (onlykey as ::core::ffi::c_ulonglong
                        | KEYC_CTRL
                        | key as ::core::ffi::c_ulonglong & KEYC_META)
                        as key_code;
                }
                current_block = 5025795842197473417;
            }
            5025795842197473417 => {
                log_debug(
                    b"%s: complete key %.*s %#llx\0" as *const u8 as *const ::core::ffi::c_char,
                    (*c).name,
                    size as ::core::ffi::c_int,
                    buf,
                    key,
                );
                if event_initialized(&raw mut (*tty).key_timer) != 0 {
                    event_del(&raw mut (*tty).key_timer);
                }
                (*tty).flags &= !TTY_TIMER;
                if key == KEYC_FOCUS_OUT as ::core::ffi::c_ulong as key_code {
                    (*c).flags &= !CLIENT_FOCUSED as uint64_t;
                    window_update_focus((*(*(*c).session).curw).window);
                    events_fire_client(
                        b"client-focus-out\0" as *const u8 as *const ::core::ffi::c_char,
                        c,
                    );
                } else if key == KEYC_FOCUS_IN as ::core::ffi::c_ulong as key_code {
                    (*c).flags |= CLIENT_FOCUSED as uint64_t;
                    events_fire_client(
                        b"client-focus-in\0" as *const u8 as *const ::core::ffi::c_char,
                        c,
                    );
                    window_update_focus((*(*(*c).session).curw).window);
                }
                if key != KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
                    event = xcalloc(1 as size_t, ::core::mem::size_of::<key_event>() as size_t)
                        as *mut key_event;
                    (*event).key = key;
                    memcpy(
                        &raw mut (*event).m as *mut ::core::ffi::c_void,
                        &raw mut m as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<mouse_event>() as size_t,
                    );
                    (*event).buf = xmalloc(size) as *mut ::core::ffi::c_char;
                    (*event).len = size;
                    memcpy(
                        (*event).buf as *mut ::core::ffi::c_void,
                        buf as *const ::core::ffi::c_void,
                        (*event).len,
                    );
                    if server_client_handle_key(c, event) == 0 {
                        free((*event).buf as *mut ::core::ffi::c_void);
                        free(event as *mut ::core::ffi::c_void);
                    }
                }
                evbuffer_drain((*tty).in_0, size);
                return 1 as ::core::ffi::c_int;
            }
            _ => {
                log_debug(
                    b"%s: partial key %.*s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*c).name,
                    len as ::core::ffi::c_int,
                    buf,
                );
                if (*tty).flags & TTY_TIMER != 0 {
                    if event_initialized(&raw mut (*tty).key_timer) != 0
                        && event_pending(
                            &raw mut (*tty).key_timer,
                            EV_TIMEOUT as ::core::ffi::c_short,
                            ::core::ptr::null_mut::<timeval>(),
                        ) == 0
                    {
                        expired = 1 as ::core::ffi::c_int;
                        current_block = 12077302897653652224;
                    } else {
                        return 0 as ::core::ffi::c_int;
                    }
                } else {
                    delay = options_get_number(
                        global_options,
                        b"escape-time\0" as *const u8 as *const ::core::ffi::c_char,
                    ) as ::core::ffi::c_int;
                    if delay == 0 as ::core::ffi::c_int {
                        delay = 1 as ::core::ffi::c_int;
                    }
                    if (*tty).flags & TTY_BRACKETPASTE != 0
                        && tty_keys_partial_paste_end(buf, len) != 0
                    {
                        log_debug(
                            b"%s: increasing delay (partial paste end)\0" as *const u8
                                as *const ::core::ffi::c_char,
                            (*c).name,
                        );
                        if delay < 500 as ::core::ffi::c_int {
                            delay = 500 as ::core::ffi::c_int;
                        }
                    }
                    if (*tty).flags & (TTY_WAITFG | TTY_WAITBG) != 0
                        || (*tty).flags & (TTY_OSC52QUERY | TTY_WINSIZEQUERY) != 0
                        || (*tty).flags & TTY_ALL_REQUEST_FLAGS != TTY_ALL_REQUEST_FLAGS
                        || !(*c).input_requests.tqh_first.is_null()
                    {
                        log_debug(
                            b"%s: increasing delay (active query)\0" as *const u8
                                as *const ::core::ffi::c_char,
                            (*c).name,
                        );
                        if delay < 500 as ::core::ffi::c_int {
                            delay = 500 as ::core::ffi::c_int;
                        }
                    }
                    tv.tv_sec = (delay / 1000 as ::core::ffi::c_int) as __time_t;
                    tv.tv_usec = ((delay % 1000 as ::core::ffi::c_int) as ::core::ffi::c_long
                        * 1000 as ::core::ffi::c_long)
                        as __suseconds_t;
                    if event_initialized(&raw mut (*tty).key_timer) != 0 {
                        event_del(&raw mut (*tty).key_timer);
                    }
                    event_set(
                        &raw mut (*tty).key_timer,
                        -(1 as ::core::ffi::c_int),
                        0 as ::core::ffi::c_short,
                        Some(
                            tty_keys_callback
                                as unsafe extern "C" fn(
                                    ::core::ffi::c_int,
                                    ::core::ffi::c_short,
                                    *mut ::core::ffi::c_void,
                                ) -> (),
                        ),
                        tty as *mut ::core::ffi::c_void,
                    );
                    event_add(&raw mut (*tty).key_timer, &raw mut tv);
                    (*tty).flags |= TTY_TIMER;
                    return 0 as ::core::ffi::c_int;
                }
            }
        }
    }
}
unsafe extern "C" fn tty_keys_callback(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut tty: *mut tty = data as *mut tty;
    if (*tty).flags & TTY_TIMER != 0 {
        while tty_keys_next(tty) != 0 {}
    }
}
unsafe extern "C" fn tty_keys_extended_key(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
    mut key: *mut key_code,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut end: size_t = 0;
    let mut number: u_int = 0;
    let mut modifiers: u_int = 0;
    let mut tmp: [::core::ffi::c_char; 64] = [0; 64];
    let mut bspace: cc_t = 0;
    let mut nkey: key_code = 0;
    let mut onlykey: key_code = 0;
    let mut ud: utf8_data = utf8_data {
        data: [0; 32],
        have: 0,
        size: 0,
        width: 0,
    };
    let mut uc: utf8_char = 0;
    *size = 0 as size_t;
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '[' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    end = 2 as size_t;
    while end < len && end != ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as usize {
        if *buf.offset(end as isize) as ::core::ffi::c_int == '~' as i32 {
            break;
        }
        if *(*__ctype_b_loc())
            .offset(*buf.offset(end as isize) as u_char as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            == 0
            && *buf.offset(end as isize) as ::core::ffi::c_int != ';' as i32
        {
            break;
        }
        end = end.wrapping_add(1);
    }
    if end == len {
        return 1 as ::core::ffi::c_int;
    }
    if end == ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as usize
        || *buf.offset(end as isize) as ::core::ffi::c_int != '~' as i32
            && *buf.offset(end as isize) as ::core::ffi::c_int != 'u' as i32
    {
        return -(1 as ::core::ffi::c_int);
    }
    memcpy(
        &raw mut tmp as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        buf.offset(2 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        end.wrapping_sub(2 as size_t),
    );
    tmp[end.wrapping_sub(2 as size_t) as usize] = '\0' as i32 as ::core::ffi::c_char;
    if *buf.offset(end as isize) as ::core::ffi::c_int == '~' as i32 {
        if sscanf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"27;%u;%u\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut modifiers,
            &raw mut number,
        ) != 2 as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int);
        }
    } else if sscanf(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"%u;%u\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut number,
        &raw mut modifiers,
    ) != 2 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    *size = end.wrapping_add(1 as size_t);
    bspace = (*tty).tio.c_cc[VERASE as usize];
    if bspace as ::core::ffi::c_int != _POSIX_VDISABLE && number == bspace as u_int {
        nkey = KEYC_BSPACE as ::core::ffi::c_ulong as key_code;
    } else {
        nkey = number as key_code;
    }
    if nkey != KEYC_BSPACE as ::core::ffi::c_ulong as key_code
        && nkey & !(0x7f as ::core::ffi::c_int) as key_code != 0
    {
        if utf8_fromwc(nkey as wchar_t, &raw mut ud) as ::core::ffi::c_uint
            == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
            && utf8_from_data(&raw mut ud, &raw mut uc) as ::core::ffi::c_uint
                == UTF8_DONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            nkey = uc as key_code;
        } else {
            return -(1 as ::core::ffi::c_int);
        }
    }
    if modifiers > 0 as u_int {
        modifiers = modifiers.wrapping_sub(1);
        if modifiers & 1 as u_int != 0 {
            nkey |= KEYC_SHIFT;
        }
        if modifiers & 2 as u_int != 0 {
            nkey |= KEYC_META | KEYC_IMPLIED_META;
        }
        if modifiers & 4 as u_int != 0 {
            nkey |= KEYC_CTRL;
        }
        if modifiers & 8 as u_int != 0 {
            nkey |= KEYC_META | KEYC_IMPLIED_META;
        }
    }
    if nkey as ::core::ffi::c_ulonglong & KEYC_MASK_KEY == '\t' as i32 as ::core::ffi::c_ulonglong
        && nkey as ::core::ffi::c_ulonglong & KEYC_SHIFT != 0
    {
        nkey = (KEYC_BTAB as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            | nkey as ::core::ffi::c_ulonglong & !KEYC_MASK_KEY & !KEYC_SHIFT)
            as key_code;
    }
    onlykey = (nkey as ::core::ffi::c_ulonglong & KEYC_MASK_KEY) as key_code;
    if (onlykey > 0x20 as key_code && onlykey < 0x7f as key_code
        || nkey as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            == (KEYC_TYPE_UNICODE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && nkey as ::core::ffi::c_ulonglong & KEYC_MASK_KEY > 0x7f as ::core::ffi::c_ulonglong)
        && nkey as ::core::ffi::c_ulonglong & KEYC_MASK_MODIFIERS == KEYC_SHIFT
    {
        nkey &= !KEYC_SHIFT;
    }
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(
            b"%s: extended key %.*s is %llx (%s)\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            *size as ::core::ffi::c_int,
            buf,
            nkey,
            key_string_lookup_key(nkey, 1 as ::core::ffi::c_int),
        );
    }
    *key = nkey;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_keys_mouse(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
    mut m: *mut mouse_event,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut i: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut b: u_int = 0;
    let mut sgr_b: u_int = 0;
    let mut sgr_type: u_char = 0;
    let mut ch: u_char = 0;
    *size = 0 as size_t;
    sgr_b = 0 as u_int;
    b = sgr_b;
    y = b;
    x = y;
    sgr_type = ' ' as i32 as u_char;
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '[' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'M' as i32 {
        *size = 3 as size_t;
        i = 0 as u_int;
        while i < 3 as u_int {
            if len <= *size {
                return 1 as ::core::ffi::c_int;
            }
            let fresh0 = *size;
            *size = (*size).wrapping_add(1);
            ch = *buf.offset(fresh0 as isize) as u_char;
            if i == 0 as u_int {
                b = ch as u_int;
            } else if i == 1 as u_int {
                x = ch as u_int;
            } else {
                y = ch as u_int;
            }
            i = i.wrapping_add(1);
        }
        log_debug(
            b"%s: mouse input: %.*s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            *size as ::core::ffi::c_int,
            buf,
        );
        if b < MOUSE_PARAM_BTN_OFF as u_int
            || x < MOUSE_PARAM_POS_OFF as u_int
            || y < MOUSE_PARAM_POS_OFF as u_int
        {
            return -(2 as ::core::ffi::c_int);
        }
        b = b.wrapping_sub(MOUSE_PARAM_BTN_OFF as u_int);
        x = x.wrapping_sub(MOUSE_PARAM_POS_OFF as u_int);
        y = y.wrapping_sub(MOUSE_PARAM_POS_OFF as u_int);
    } else if *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '<' as i32 {
        *size = 3 as size_t;
        loop {
            if len <= *size {
                return 1 as ::core::ffi::c_int;
            }
            let fresh1 = *size;
            *size = (*size).wrapping_add(1);
            ch = *buf.offset(fresh1 as isize) as u_char;
            if ch as ::core::ffi::c_int == ';' as i32 {
                break;
            }
            if (ch as ::core::ffi::c_int) < '0' as i32 || ch as ::core::ffi::c_int > '9' as i32 {
                return -(1 as ::core::ffi::c_int);
            }
            sgr_b = (10 as u_int)
                .wrapping_mul(sgr_b)
                .wrapping_add((ch as ::core::ffi::c_int - '0' as i32) as u_int);
        }
        loop {
            if len <= *size {
                return 1 as ::core::ffi::c_int;
            }
            let fresh2 = *size;
            *size = (*size).wrapping_add(1);
            ch = *buf.offset(fresh2 as isize) as u_char;
            if ch as ::core::ffi::c_int == ';' as i32 {
                break;
            }
            if (ch as ::core::ffi::c_int) < '0' as i32 || ch as ::core::ffi::c_int > '9' as i32 {
                return -(1 as ::core::ffi::c_int);
            }
            x = (10 as u_int)
                .wrapping_mul(x)
                .wrapping_add((ch as ::core::ffi::c_int - '0' as i32) as u_int);
        }
        loop {
            if len <= *size {
                return 1 as ::core::ffi::c_int;
            }
            let fresh3 = *size;
            *size = (*size).wrapping_add(1);
            ch = *buf.offset(fresh3 as isize) as u_char;
            if ch as ::core::ffi::c_int == 'M' as i32 || ch as ::core::ffi::c_int == 'm' as i32 {
                break;
            }
            if (ch as ::core::ffi::c_int) < '0' as i32 || ch as ::core::ffi::c_int > '9' as i32 {
                return -(1 as ::core::ffi::c_int);
            }
            y = (10 as u_int)
                .wrapping_mul(y)
                .wrapping_add((ch as ::core::ffi::c_int - '0' as i32) as u_int);
        }
        log_debug(
            b"%s: mouse input (SGR): %.*s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            *size as ::core::ffi::c_int,
            buf,
        );
        if x < 1 as u_int || y < 1 as u_int {
            return -(2 as ::core::ffi::c_int);
        }
        x = x.wrapping_sub(1);
        y = y.wrapping_sub(1);
        b = sgr_b;
        sgr_type = ch;
        if sgr_type as ::core::ffi::c_int == 'm' as i32 {
            b = 3 as u_int;
        }
        if sgr_type as ::core::ffi::c_int == 'm' as i32
            && (sgr_b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
                || sgr_b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int)
        {
            return -(2 as ::core::ffi::c_int);
        }
    } else {
        return -(1 as ::core::ffi::c_int);
    }
    (*m).lx = (*tty).mouse_last_x;
    (*m).x = x;
    (*m).ly = (*tty).mouse_last_y;
    (*m).y = y;
    (*m).lb = (*tty).mouse_last_b;
    (*m).b = b;
    (*m).sgr_type = sgr_type as u_int;
    (*m).sgr_b = sgr_b;
    (*tty).mouse_last_x = x;
    (*tty).mouse_last_y = y;
    (*tty).mouse_last_b = b;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_keys_clipboard(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut end: size_t = 0;
    let mut terminator: size_t = 0 as size_t;
    let mut needed: size_t = 0;
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut clip: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
    let mut outlen: ::core::ffi::c_int = 0;
    let mut cd: input_request_clipboard_data = input_request_clipboard_data {
        buf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        len: 0,
        clip: 0,
    };
    *size = 0 as size_t;
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ']' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '5' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 3 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '2' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 4 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ';' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 5 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    end = 5 as size_t;
    while end < len {
        if *buf.offset(end as isize) as ::core::ffi::c_int == '\u{7}' as i32 {
            terminator = 1 as size_t;
            break;
        } else if end > 5 as size_t
            && *buf.offset(end.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                == '\u{1b}' as i32
            && *buf.offset(end as isize) as ::core::ffi::c_int == '\\' as i32
        {
            terminator = 2 as size_t;
            break;
        } else {
            end = end.wrapping_add(1);
        }
    }
    if end == len {
        return 1 as ::core::ffi::c_int;
    }
    *size = end.wrapping_add(1 as size_t);
    buf = buf.offset(5 as ::core::ffi::c_int as isize);
    end = end.wrapping_sub(5 as size_t);
    end = end.wrapping_sub(terminator.wrapping_sub(1 as size_t));
    if end >= 2 as size_t
        && *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ';' as i32
        && *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == ';' as i32
    {
        clip = *buf.offset(0 as ::core::ffi::c_int as isize);
    }
    while end != 0 as size_t && *buf as ::core::ffi::c_int != ';' as i32 {
        buf = buf.offset(1);
        end = end.wrapping_sub(1);
    }
    if end == 0 as size_t || end == 1 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    buf = buf.offset(1);
    end = end.wrapping_sub(1);
    copy = xmalloc(end.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    memcpy(
        copy as *mut ::core::ffi::c_void,
        buf as *const ::core::ffi::c_void,
        end,
    );
    *copy.offset(end as isize) = '\0' as i32 as ::core::ffi::c_char;
    needed = end
        .wrapping_add(3 as size_t)
        .wrapping_div(4 as size_t)
        .wrapping_mul(3 as size_t);
    if needed == 0 as size_t {
        free(copy as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    out = xmalloc(needed) as *mut ::core::ffi::c_char;
    outlen = __b64_pton(copy, out as *mut ::core::ffi::c_uchar, needed);
    if outlen == -(1 as ::core::ffi::c_int) {
        free(out as *mut ::core::ffi::c_void);
        free(copy as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    free(copy as *mut ::core::ffi::c_void);
    log_debug(
        b"%s: %.*s\0" as *const u8 as *const ::core::ffi::c_char,
        b"tty_keys_clipboard\0" as *const u8 as *const ::core::ffi::c_char,
        outlen,
        out,
    );
    cd.buf = out;
    cd.len = outlen as size_t;
    cd.clip = clip;
    input_request_reply(
        c,
        INPUT_REQUEST_CLIPBOARD,
        &raw mut cd as *mut ::core::ffi::c_void,
    );
    if (*tty).flags & TTY_OSC52QUERY != 0 {
        paste_add(
            ::core::ptr::null::<::core::ffi::c_char>(),
            out,
            outlen as size_t,
        );
        out = ::core::ptr::null_mut::<::core::ffi::c_char>();
        event_del(&raw mut (*tty).clipboard_timer);
        (*tty).flags &= !TTY_OSC52QUERY;
    }
    free(out as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_keys_device_attributes(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut i: u_int = 0;
    let mut n: u_int = 0 as u_int;
    let mut tmp: [::core::ffi::c_char; 128] = [0; 128];
    let mut endptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: [::core::ffi::c_char; 32] = [
        0 as ::core::ffi::c_int as ::core::ffi::c_char,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    *size = 0 as size_t;
    if (*tty).flags & TTY_HAVEDA != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '[' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '?' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 3 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    i = 0 as u_int;
    while (i as usize) < ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize {
        if (3 as u_int).wrapping_add(i) as size_t == len {
            return 1 as ::core::ffi::c_int;
        }
        if *buf.offset((3 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int >= 'a' as i32
            && *buf.offset((3 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int
                <= 'z' as i32
        {
            break;
        }
        tmp[i as usize] = *buf.offset((3 as u_int).wrapping_add(i) as isize);
        i = i.wrapping_add(1);
    }
    if i as usize == ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize {
        return -(1 as ::core::ffi::c_int);
    }
    if *buf.offset((3 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int != 'c' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    tmp[i as usize] = '\0' as i32 as ::core::ffi::c_char;
    *size = (4 as u_int).wrapping_add(i) as size_t;
    cp = &raw mut tmp as *mut ::core::ffi::c_char;
    loop {
        next = strsep(
            &raw mut cp,
            b";\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if next.is_null() {
            break;
        }
        p[n as usize] =
            strtoul(next, &raw mut endptr, 10 as ::core::ffi::c_int) as ::core::ffi::c_char;
        if *endptr as ::core::ffi::c_int != '\0' as i32 {
            p[n as usize] = 0 as ::core::ffi::c_char;
        }
        n = n.wrapping_add(1);
        if n as usize
            == (::core::mem::size_of::<[::core::ffi::c_char; 32]>() as usize)
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>() as usize)
        {
            break;
        }
    }
    match p[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int {
        61 | 62 | 63 | 64 | 65 => {
            i = 1 as u_int;
            while i < n {
                log_debug(
                    b"%s: DA feature: %d\0" as *const u8 as *const ::core::ffi::c_char,
                    (*c).name,
                    p[i as usize] as ::core::ffi::c_int,
                );
                if p[i as usize] as ::core::ffi::c_int == 4 as ::core::ffi::c_int {
                    tty_parse_client_features(
                        c,
                        b"sixel\0" as *const u8 as *const ::core::ffi::c_char,
                        b",\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                if p[i as usize] as ::core::ffi::c_int == 21 as ::core::ffi::c_int {
                    tty_parse_client_features(
                        c,
                        b"margins\0" as *const u8 as *const ::core::ffi::c_char,
                        b",\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                if p[i as usize] as ::core::ffi::c_int == 28 as ::core::ffi::c_int {
                    tty_parse_client_features(
                        c,
                        b"rectfill\0" as *const u8 as *const ::core::ffi::c_char,
                        b",\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                if p[i as usize] as ::core::ffi::c_int == 52 as ::core::ffi::c_int {
                    tty_parse_client_features(
                        c,
                        b"clipboard\0" as *const u8 as *const ::core::ffi::c_char,
                        b",\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                i = i.wrapping_add(1);
            }
        }
        _ => {}
    }
    log_debug(
        b"%s: received primary DA %.*s\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        *size as ::core::ffi::c_int,
        buf,
    );
    tty_update_features(tty);
    (*tty).flags |= TTY_HAVEDA;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_keys_sync(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    static mut prefix: [::core::ffi::c_char; 9] =
        unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"\x1B[?2026;\0") };
    let mut i: size_t = 0;
    let mut status: ::core::ffi::c_int = 0;
    *size = 0 as size_t;
    if (*tty).flags & TTY_HAVESYNC != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    i = 0 as size_t;
    while i < (::core::mem::size_of::<[::core::ffi::c_char; 9]>() as usize).wrapping_sub(1 as usize)
    {
        if i == len {
            return 1 as ::core::ffi::c_int;
        }
        if *buf.offset(i as isize) as ::core::ffi::c_int != prefix[i as usize] as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int);
        }
        i = i.wrapping_add(1);
    }
    if i == len {
        return 1 as ::core::ffi::c_int;
    }
    if (*buf.offset(i as isize) as ::core::ffi::c_int) < '0' as i32
        || *buf.offset(i as isize) as ::core::ffi::c_int > '4' as i32
    {
        return -(1 as ::core::ffi::c_int);
    }
    let fresh4 = i;
    i = i.wrapping_add(1);
    status = *buf.offset(fresh4 as isize) as ::core::ffi::c_int - '0' as i32;
    if i == len {
        return 1 as ::core::ffi::c_int;
    }
    let fresh5 = i;
    i = i.wrapping_add(1);
    if *buf.offset(fresh5 as isize) as ::core::ffi::c_int != '$' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if i == len {
        return 1 as ::core::ffi::c_int;
    }
    let fresh6 = i;
    i = i.wrapping_add(1);
    if *buf.offset(fresh6 as isize) as ::core::ffi::c_int != 'y' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    *size = i;
    if status == 1 as ::core::ffi::c_int
        || status == 2 as ::core::ffi::c_int
        || status == 3 as ::core::ffi::c_int
    {
        tty_parse_client_features(
            c,
            b"sync\0" as *const u8 as *const ::core::ffi::c_char,
            b",\0" as *const u8 as *const ::core::ffi::c_char,
        );
        tty_update_features(tty);
    }
    log_debug(
        b"%s: received DECRPM %.*s\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        *size as ::core::ffi::c_int,
        buf,
    );
    (*tty).flags |= TTY_HAVESYNC;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_keys_device_attributes2(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut i: u_int = 0;
    let mut n: u_int = 0 as u_int;
    let mut tmp: [::core::ffi::c_char; 128] = [0; 128];
    let mut endptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: [::core::ffi::c_char; 32] = [
        0 as ::core::ffi::c_int as ::core::ffi::c_char,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    *size = 0 as size_t;
    if (*tty).flags & TTY_HAVEDA2 != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '[' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '>' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 3 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    i = 0 as u_int;
    while (i as usize) < ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize {
        if (3 as u_int).wrapping_add(i) as size_t == len {
            return 1 as ::core::ffi::c_int;
        }
        if *buf.offset((3 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int >= 'a' as i32
            && *buf.offset((3 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int
                <= 'z' as i32
        {
            break;
        }
        tmp[i as usize] = *buf.offset((3 as u_int).wrapping_add(i) as isize);
        i = i.wrapping_add(1);
    }
    if i as usize == ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize {
        return -(1 as ::core::ffi::c_int);
    }
    if *buf.offset((3 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int != 'c' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    tmp[i as usize] = '\0' as i32 as ::core::ffi::c_char;
    *size = (4 as u_int).wrapping_add(i) as size_t;
    cp = &raw mut tmp as *mut ::core::ffi::c_char;
    loop {
        next = strsep(
            &raw mut cp,
            b";\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if next.is_null() {
            break;
        }
        p[n as usize] =
            strtoul(next, &raw mut endptr, 10 as ::core::ffi::c_int) as ::core::ffi::c_char;
        if *endptr as ::core::ffi::c_int != '\0' as i32 {
            p[n as usize] = 0 as ::core::ffi::c_char;
        }
        n = n.wrapping_add(1);
        if n as usize
            == (::core::mem::size_of::<[::core::ffi::c_char; 32]>() as usize)
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>() as usize)
        {
            break;
        }
    }
    match p[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int {
        77 => {
            tty_default_features(
                c,
                b"mintty\0" as *const u8 as *const ::core::ffi::c_char,
                0 as u_int,
            );
        }
        84 => {
            tty_default_features(
                c,
                b"tmux\0" as *const u8 as *const ::core::ffi::c_char,
                0 as u_int,
            );
        }
        85 => {
            tty_default_features(
                c,
                b"rxvt-unicode\0" as *const u8 as *const ::core::ffi::c_char,
                0 as u_int,
            );
        }
        _ => {}
    }
    log_debug(
        b"%s: received secondary DA %.*s\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        *size as ::core::ffi::c_int,
        buf,
    );
    tty_update_features(tty);
    (*tty).flags |= TTY_HAVEDA2;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_keys_extended_device_attributes(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut i: u_int = 0;
    let mut tmp: [::core::ffi::c_char; 128] = [0; 128];
    *size = 0 as size_t;
    if (*tty).flags & TTY_HAVEXDA != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 'P' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '>' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 3 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '|' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 4 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize).wrapping_sub(1 as usize)
    {
        if (4 as u_int).wrapping_add(i) as size_t == len {
            return 1 as ::core::ffi::c_int;
        }
        if *buf.offset((4 as u_int).wrapping_add(i).wrapping_sub(1 as u_int) as isize)
            as ::core::ffi::c_int
            == '\u{1b}' as i32
            && *buf.offset((4 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int
                == '\\' as i32
        {
            break;
        }
        tmp[i as usize] = *buf.offset((4 as u_int).wrapping_add(i) as isize);
        i = i.wrapping_add(1);
    }
    if i as usize
        == (::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize).wrapping_sub(1 as usize)
    {
        return -(1 as ::core::ffi::c_int);
    }
    *size = (5 as u_int).wrapping_add(i) as size_t;
    if i == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    tmp[i.wrapping_sub(1 as u_int) as usize] = '\0' as i32 as ::core::ffi::c_char;
    if strncmp(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"iTerm2 \0" as *const u8 as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        tty_default_features(
            c,
            b"iTerm2\0" as *const u8 as *const ::core::ffi::c_char,
            0 as u_int,
        );
    } else if strncmp(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"tmux \0" as *const u8 as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        tty_default_features(
            c,
            b"tmux\0" as *const u8 as *const ::core::ffi::c_char,
            0 as u_int,
        );
    } else if strncmp(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"XTerm(\0" as *const u8 as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        tty_default_features(
            c,
            b"XTerm\0" as *const u8 as *const ::core::ffi::c_char,
            0 as u_int,
        );
    } else if strncmp(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"mintty \0" as *const u8 as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        tty_default_features(
            c,
            b"mintty\0" as *const u8 as *const ::core::ffi::c_char,
            0 as u_int,
        );
    } else if strncmp(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"foot(\0" as *const u8 as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        tty_default_features(
            c,
            b"foot\0" as *const u8 as *const ::core::ffi::c_char,
            0 as u_int,
        );
    } else if strncmp(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"WezTerm \0" as *const u8 as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        tty_default_features(
            c,
            b"WezTerm\0" as *const u8 as *const ::core::ffi::c_char,
            0 as u_int,
        );
    } else if strncmp(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"ghostty \0" as *const u8 as *const ::core::ffi::c_char,
        8 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        tty_default_features(
            c,
            b"ghostty\0" as *const u8 as *const ::core::ffi::c_char,
            0 as u_int,
        );
    } else if strncmp(
        &raw mut tmp as *mut ::core::ffi::c_char,
        b"Rio \0" as *const u8 as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        tty_default_features(
            c,
            b"Rio\0" as *const u8 as *const ::core::ffi::c_char,
            0 as u_int,
        );
    }
    log_debug(
        b"%s: received extended DA %.*s\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        *size as ::core::ffi::c_int,
        buf,
    );
    free((*c).term_type as *mut ::core::ffi::c_void);
    (*c).term_type = xstrdup(&raw mut tmp as *mut ::core::ffi::c_char);
    tty_update_features(tty);
    (*tty).flags |= TTY_HAVEXDA;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn tty_keys_colours(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
    mut fg: *mut ::core::ffi::c_int,
    mut bg: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut i: u_int = 0;
    let mut tmp: [::core::ffi::c_char; 128] = [0; 128];
    let mut n: ::core::ffi::c_int = 0;
    *size = 0 as size_t;
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ']' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '1' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 3 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '0' as i32
        && *buf.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '1' as i32
    {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 4 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ';' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 5 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize).wrapping_sub(1 as usize)
    {
        if (5 as u_int).wrapping_add(i) as size_t == len {
            return 1 as ::core::ffi::c_int;
        }
        if *buf.offset((5 as u_int).wrapping_add(i).wrapping_sub(1 as u_int) as isize)
            as ::core::ffi::c_int
            == '\u{1b}' as i32
            && *buf.offset((5 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int
                == '\\' as i32
        {
            break;
        }
        if *buf.offset((5 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int
            == '\u{7}' as i32
        {
            break;
        }
        tmp[i as usize] = *buf.offset((5 as u_int).wrapping_add(i) as isize);
        i = i.wrapping_add(1);
    }
    if i as usize
        == (::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize).wrapping_sub(1 as usize)
    {
        return -(1 as ::core::ffi::c_int);
    }
    *size = (6 as u_int).wrapping_add(i) as size_t;
    if i == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    if tmp[i.wrapping_sub(1 as u_int) as usize] as ::core::ffi::c_int == '\u{1b}' as i32 {
        tmp[i.wrapping_sub(1 as u_int) as usize] = '\0' as i32 as ::core::ffi::c_char;
    } else {
        tmp[i as usize] = '\0' as i32 as ::core::ffi::c_char;
    }
    n = colour_parseX11(&raw mut tmp as *mut ::core::ffi::c_char);
    if n != -(1 as ::core::ffi::c_int)
        && *buf.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '0' as i32
    {
        if !c.is_null() {
            log_debug(
                b"%s fg is %s\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
                colour_tostring(n),
            );
        } else {
            log_debug(
                b"fg is %s\0" as *const u8 as *const ::core::ffi::c_char,
                colour_tostring(n),
            );
        }
        *fg = n;
        (*tty).flags &= !TTY_WAITFG;
    } else if n != -(1 as ::core::ffi::c_int) {
        if !c.is_null() {
            log_debug(
                b"%s bg is %s\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
                colour_tostring(n),
            );
        } else {
            log_debug(
                b"bg is %s\0" as *const u8 as *const ::core::ffi::c_char,
                colour_tostring(n),
            );
        }
        *bg = n;
        (*tty).flags &= !TTY_WAITBG;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn tty_keys_palette(
    mut tty: *mut tty,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut size: *mut size_t,
) -> ::core::ffi::c_int {
    let mut c: *mut client = (*tty).client;
    let mut i: u_int = 0;
    let mut tmp: [::core::ffi::c_char; 128] = [0; 128];
    let mut endptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_int = 0;
    let mut pd: input_request_palette_data = input_request_palette_data { idx: 0, c: 0 };
    *size = 0 as size_t;
    if *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\u{1b}' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 1 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ']' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 2 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '4' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 3 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    if *buf.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ';' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if len == 4 as size_t {
        return 1 as ::core::ffi::c_int;
    }
    i = 0 as u_int;
    while (i as usize)
        < (::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize).wrapping_sub(1 as usize)
    {
        if (4 as u_int).wrapping_add(i) as size_t == len {
            return 1 as ::core::ffi::c_int;
        }
        if *buf.offset((4 as u_int).wrapping_add(i).wrapping_sub(1 as u_int) as isize)
            as ::core::ffi::c_int
            == '\u{1b}' as i32
            && *buf.offset((4 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int
                == '\\' as i32
        {
            break;
        }
        if *buf.offset((4 as u_int).wrapping_add(i) as isize) as ::core::ffi::c_int
            == '\u{7}' as i32
        {
            break;
        }
        tmp[i as usize] = *buf.offset((4 as u_int).wrapping_add(i) as isize);
        i = i.wrapping_add(1);
    }
    if i as usize
        == (::core::mem::size_of::<[::core::ffi::c_char; 128]>() as usize).wrapping_sub(1 as usize)
    {
        return -(1 as ::core::ffi::c_int);
    }
    *size = (5 as u_int).wrapping_add(i) as size_t;
    if i == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    if tmp[i.wrapping_sub(1 as u_int) as usize] as ::core::ffi::c_int == '\u{1b}' as i32 {
        tmp[i.wrapping_sub(1 as u_int) as usize] = '\0' as i32 as ::core::ffi::c_char;
    } else {
        tmp[i as usize] = '\0' as i32 as ::core::ffi::c_char;
    }
    idx = strtol(
        &raw mut tmp as *mut ::core::ffi::c_char,
        &raw mut endptr,
        10 as ::core::ffi::c_int,
    ) as ::core::ffi::c_int;
    if *endptr as ::core::ffi::c_int != ';' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    if idx < 0 as ::core::ffi::c_int || idx > 255 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    pd.c = colour_parseX11(endptr.offset(1 as ::core::ffi::c_int as isize));
    if pd.c == -(1 as ::core::ffi::c_int) {
        return 0 as ::core::ffi::c_int;
    }
    pd.idx = idx;
    input_request_reply(
        c,
        INPUT_REQUEST_PALETTE,
        &raw mut pd as *mut ::core::ffi::c_void,
    );
    return 0 as ::core::ffi::c_int;
}

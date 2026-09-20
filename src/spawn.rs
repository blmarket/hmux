pub use crate::src::shared::limits::SIZE_MAX;
pub use crate::src::shared::posix_terminal::{winsize, TCSANOW, VERASE};
pub use crate::src::shared::signal::{
    __sigset_t, sigset_t, SIGCHLD, SIGHUP, SIG_BLOCK, SIG_SETMASK,
};
pub use crate::src::shared::spawn::{
    SPAWN_DETACHED, SPAWN_EMPTY, SPAWN_FLOATING, SPAWN_FLOATOVERZOOM, SPAWN_KILL, SPAWN_MODAL,
    SPAWN_NONOTIFY, SPAWN_RESPAWN, SPAWN_ZOOM,
};
pub use crate::src::shared::window::{
    WINDOW_ZOOMED, WINLINK_ACTIVITY, WINLINK_ALERTFLAGS, WINLINK_BELL, WINLINK_SILENCE,
};
pub use crate::src::shared::pane::{
    PANE_EMPTY, PANE_EXITED, PANE_FLOATOVERZOOM, PANE_STATUSDRAWN, PANE_STATUSREADY,
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::screen::{MODE_CRLF, MODE_CURSOR, screen, screen_sel, screen_titles};
pub use crate::src::shared::stdio::{
    FILE, _IO_FILE, _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data,
};
pub use crate::src::shared::abi::{__off64_t, __off_t};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::client::{CLIENT_CONTROL};
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
    pub type cmds;
    pub type input_request;
    pub type redraw_scene;
    pub type tty_key;
    pub type tty_code;
    pub type format_job_tree;
    pub type control_state;
    pub type cmdq_list;
    pub type event_payload;
    pub type options_entry;
    pub type tmuxproc;
    fn kill(__pid: __pid_t, __sig: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn sigfillset(__set: *mut sigset_t) -> ::core::ffi::c_int;
    fn sigprocmask(
        __how: ::core::ffi::c_int,
        __set: *const sigset_t,
        __oset: *mut sigset_t,
    ) -> ::core::ffi::c_int;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn closefrom(__lowfd: ::core::ffi::c_int);
    fn chdir(__path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn getcwd(__buf: *mut ::core::ffi::c_char, __size: size_t) -> *mut ::core::ffi::c_char;
    fn execl(
        __path: *const ::core::ffi::c_char,
        __arg: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn execvp(
        __file: *const ::core::ffi::c_char,
        __argv: *const *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn _exit(__status: ::core::ffi::c_int) -> !;
    fn getpid() -> __pid_t;
    fn unlink(__name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
    ) -> *mut FILE;
    fn fdopen(__fd: ::core::ffi::c_int, __modes: *const ::core::ffi::c_char) -> *mut FILE;
    fn fread(
        __ptr: *mut ::core::ffi::c_void,
        __size: size_t,
        __n: size_t,
        __stream: *mut FILE,
    ) -> ::core::ffi::c_ulong;
    fn fwrite(
        __ptr: *const ::core::ffi::c_void,
        __size: size_t,
        __n: size_t,
        __s: *mut FILE,
    ) -> ::core::ffi::c_ulong;
    fn fseeko(
        __stream: *mut FILE,
        __off: __off_t,
        __whence: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn ftello(__stream: *mut FILE) -> __off_t;
    fn mkstemp(__template: *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
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
    fn strrchr(
        __s: *const ::core::ffi::c_char,
        __c: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn strerror(__errnum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn tcgetattr(__fd: ::core::ffi::c_int, __termios_p: *mut termios) -> ::core::ffi::c_int;
    fn tcsetattr(
        __fd: ::core::ffi::c_int,
        __optional_actions: ::core::ffi::c_int,
        __termios_p: *const termios,
    ) -> ::core::ffi::c_int;
    fn utempter_add_record(
        master_fd: ::core::ffi::c_int,
        hostname: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn bufferevent_free(bufev: *mut bufferevent);
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn fdforkpty(
        _: ::core::ffi::c_int,
        _: *mut ::core::ffi::c_int,
        _: *mut ::core::ffi::c_char,
        _: *mut termios,
        _: *mut winsize,
    ) -> pid_t;
    fn systemd_move_to_new_cgroup(_: *mut *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
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
    static mut ptm_fd: ::core::ffi::c_int;
    fn checkshell(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn find_home() -> *const ::core::ffi::c_char;
    fn proc_clear_signals(_: *mut tmuxproc, _: ::core::ffi::c_int);
    fn format_single(
        _: *mut cmdq_item,
        _: *const ::core::ffi::c_char,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    ) -> *mut ::core::ffi::c_char;
    fn event_payload_create() -> *mut event_payload;
    fn event_payload_set_target(_: *mut event_payload, _: *mut cmd_find_state);
    fn event_payload_set_string(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn event_payload_set_int(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    );
    fn event_payload_set_session(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut session,
    );
    fn event_payload_set_window(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut window,
    );
    fn event_payload_set_pane(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut window_pane,
    );
    fn events_fire(_: *const ::core::ffi::c_char, _: *mut event_payload);
    fn events_fire_window(_: *const ::core::ffi::c_char, _: *mut window);
    fn events_fire_winlink(_: *const ::core::ffi::c_char, _: *mut winlink);
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn options_set_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
    ) -> *mut options_entry;
    fn environ_create() -> *mut environ;
    fn environ_free(_: *mut environ);
    fn environ_copy(_: *mut environ, _: *mut environ);
    fn environ_find(_: *mut environ, _: *const ::core::ffi::c_char) -> *mut environ_entry;
    fn environ_set(
        _: *mut environ,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn environ_push(_: *mut environ);
    fn environ_log(_: *mut environ, _: *const ::core::ffi::c_char, ...);
    fn environ_for_session(_: *mut session, _: ::core::ffi::c_int) -> *mut environ;
    fn cmd_find_from_winlink_pane(
        _: *mut cmd_find_state,
        _: *mut winlink,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    );
    fn cmd_log_argv(
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn cmd_copy_argv(
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut *mut ::core::ffi::c_char;
    fn cmd_free_argv(_: ::core::ffi::c_int, _: *mut *mut ::core::ffi::c_char);
    fn cmd_stringify_argv(
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_target(_: *mut cmdq_item) -> *mut cmd_find_state;
    static mut server_proc: *mut tmuxproc;
    static mut clients: clients;
    fn server_client_get_cwd(_: *mut client, _: *mut session) -> *const ::core::ffi::c_char;
    fn server_client_remove_pane(_: *mut window_pane);
    fn default_window_size(
        _: *mut client,
        _: *mut session,
        _: *mut window,
        _: *mut u_int,
        _: *mut u_int,
        _: *mut u_int,
        _: *mut u_int,
        _: ::core::ffi::c_int,
    );
    fn input_free(_: *mut input_ctx);
    fn screen_reinit(_: *mut screen, _: ::core::ffi::c_int);
    fn winlink_find_by_index(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn winlink_add(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn winlink_set_window(_: *mut winlink, _: *mut window);
    fn winlink_remove(_: *mut winlinks, _: *mut winlink);
    fn winlink_stack_remove(_: *mut winlink_stack, _: *mut winlink);
    fn window_create(_: u_int, _: u_int, _: u_int, _: u_int) -> *mut window;
    fn window_pane_set_event(_: *mut window_pane);
    fn window_set_active_pane(
        _: *mut window,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn window_redraw_active_switch(_: *mut window, _: *mut window_pane);
    fn window_add_pane(
        _: *mut window,
        _: *mut window_pane,
        _: u_int,
        _: ::core::ffi::c_int,
    ) -> *mut window_pane;
    fn window_push_zoom(
        _: *mut window,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn window_pop_zoom(_: *mut window) -> ::core::ffi::c_int;
    fn window_remove_pane(_: *mut window, _: *mut window_pane);
    fn window_pane_index(_: *mut window_pane, _: *mut u_int) -> ::core::ffi::c_int;
    fn window_destroy_panes(_: *mut window);
    fn window_pane_resize(_: *mut window_pane, _: u_int, _: u_int);
    fn window_pane_reset_mode_all(_: *mut window_pane);
    fn window_set_fill_cells(_: *mut window);
    fn layout_init(_: *mut window, _: *mut window_pane);
    fn layout_free(_: *mut window, _: ::core::ffi::c_int);
    fn layout_assign_pane(_: *mut layout_cell, _: *mut window_pane, _: ::core::ffi::c_int);
    fn layout_floating_pane(
        _: *mut window,
        _: *mut window_pane,
        _: *mut layout_geometry,
    ) -> *mut layout_cell;
    fn layout_close_pane(_: *mut window_pane);
    fn default_window_name(_: *mut window) -> *mut ::core::ffi::c_char;
    fn control_reset_pane(_: *mut client, _: *mut window_pane);
    fn session_select(_: *mut session, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn session_group_synchronize_from(_: *mut session);
    fn log_close();
    fn log_debug(_: *const ::core::ffi::c_char, ...);
}
pub type off_t = __off_t;

pub type uintmax_t = ::libc::uintmax_t;
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
pub struct spawn_editor_state {
    pub path: *mut ::core::ffi::c_char,
    pub pid: pid_t,
    pub cb: spawn_finish_edit_cb,
    pub arg: *mut ::core::ffi::c_void,
}
pub type spawn_finish_edit_cb =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_char, size_t, *mut ::core::ffi::c_void) -> ()>;
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
pub struct environ_entry {
    pub name: *mut ::core::ffi::c_char,
    pub value: *mut ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub rbe_left: *mut environ_entry,
    pub rbe_right: *mut environ_entry,
    pub rbe_parent: *mut environ_entry,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct clients {
    pub tqh_first: *mut client,
    pub tqh_last: *mut *mut client,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct spawn_context {
    pub item: *mut cmdq_item,
    pub s: *mut session,
    pub wl: *mut winlink,
    pub tc: *mut client,
    pub wp0: *mut window_pane,
    pub lc: *mut layout_cell,
    pub name: *const ::core::ffi::c_char,
    pub argv: *mut *mut ::core::ffi::c_char,
    pub argc: ::core::ffi::c_int,
    pub environ: *mut environ,
    pub idx: ::core::ffi::c_int,
    pub cwd: *const ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
}

pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;

pub const IUTF8: ::core::ffi::c_int = 0o40000 as ::core::ffi::c_int;

pub const _PATH_DEFPATH: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"/usr/bin:/bin\0") };
pub const _PATH_BSHELL: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"/bin/sh\0") };
pub const _PATH_TMP: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"/tmp/\0") };

unsafe extern "C" fn spawn_log(mut from: *const ::core::ffi::c_char, mut sc: *mut spawn_context) {
    let mut s: *mut session = (*sc).s;
    let mut wl: *mut winlink = (*sc).wl;
    let mut wp0: *mut window_pane = (*sc).wp0;
    let mut name: *const ::core::ffi::c_char = if (*sc).name.is_null() {
        b"none\0" as *const u8 as *const ::core::ffi::c_char
    } else {
        (*sc).name
    };
    let mut tmp: [::core::ffi::c_char; 128] = [0; 128];
    log_debug(
        b"%s: name=%s, flags=%#x\0" as *const u8 as *const ::core::ffi::c_char,
        from,
        name,
        (*sc).flags,
    );
    if !wl.is_null() && !wp0.is_null() {
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
            b"wl=%d wp0=%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wl).idx,
            (*wp0).id,
        );
    } else if !wl.is_null() {
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
            b"wl=%d wp0=none\0" as *const u8 as *const ::core::ffi::c_char,
            (*wl).idx,
        );
    } else if !wp0.is_null() {
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
            b"wl=none wp0=%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp0).id,
        );
    } else {
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
            b"wl=none wp0=none\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    log_debug(
        b"%s: s=$%u %s idx=%d\0" as *const u8 as *const ::core::ffi::c_char,
        from,
        (*s).id,
        &raw mut tmp as *mut ::core::ffi::c_char,
        (*sc).idx,
    );
}
unsafe extern "C" fn spawn_fire_pane_created(mut sc: *mut spawn_context, mut wp: *mut window_pane) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cwd: *const ::core::ffi::c_char = (*wp).cwd;
    ep = event_payload_create();
    cmd_find_from_winlink_pane(&raw mut fs, (*sc).wl, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_session(
        ep,
        b"session\0" as *const u8 as *const ::core::ffi::c_char,
        (*sc).s,
    );
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    event_payload_set_int(
        ep,
        b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
        (*(*sc).wl).idx,
    );
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    if (*wp).argc != 0 as ::core::ffi::c_int {
        cmd = cmd_stringify_argv((*wp).argc, (*wp).argv);
    }
    if !cmd.is_null() && *cmd as ::core::ffi::c_int != '\0' as i32 {
        event_payload_set_string(
            ep,
            b"pane_command\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            cmd,
        );
    } else if !(*wp).shell.is_null() {
        event_payload_set_string(
            ep,
            b"pane_command\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).shell,
        );
    }
    free(cmd as *mut ::core::ffi::c_void);
    if !cwd.is_null() {
        event_payload_set_string(
            ep,
            b"pane_current_path\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            cwd,
        );
    }
    if (*sc).flags & SPAWN_EMPTY != 0 {
        event_payload_set_int(
            ep,
            b"created_empty\0" as *const u8 as *const ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
    } else {
        event_payload_set_int(
            ep,
            b"created_empty\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    }
    if (*sc).flags & SPAWN_RESPAWN != 0 {
        event_payload_set_int(
            ep,
            b"created_respawn\0" as *const u8 as *const ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
    } else {
        event_payload_set_int(
            ep,
            b"created_respawn\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    }
    events_fire(
        b"pane-created\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
#[no_mangle]
pub unsafe extern "C" fn spawn_window(
    mut sc: *mut spawn_context,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut winlink {
    let mut s: *mut session = (*sc).s;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut idx: ::core::ffi::c_int = (*sc).idx;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut xpixel: u_int = 0;
    let mut ypixel: u_int = 0;
    spawn_log(
        b"spawn_window\0" as *const u8 as *const ::core::ffi::c_char,
        sc,
    );
    if (*sc).flags & SPAWN_RESPAWN != 0 {
        w = (*(*sc).wl).window;
        if !(*sc).flags & SPAWN_KILL != 0 {
            wp = (*w).panes.tqh_first;
            while !wp.is_null() {
                if (*wp).fd != -(1 as ::core::ffi::c_int) {
                    break;
                }
                wp = (*wp).entry.tqe_next;
            }
            if !wp.is_null() {
                xasprintf(
                    cause,
                    b"window %s:%d still active\0" as *const u8 as *const ::core::ffi::c_char,
                    (*s).name,
                    (*(*sc).wl).idx,
                );
                return ::core::ptr::null_mut::<winlink>();
            }
        }
        (*sc).wp0 = (*w).panes.tqh_first;
        if !(*(*sc).wp0).entry.tqe_next.is_null() {
            (*(*(*sc).wp0).entry.tqe_next).entry.tqe_prev = (*(*sc).wp0).entry.tqe_prev;
        } else {
            (*w).panes.tqh_last = (*(*sc).wp0).entry.tqe_prev;
        }
        *(*(*sc).wp0).entry.tqe_prev = (*(*sc).wp0).entry.tqe_next;
        layout_free(w, 0 as ::core::ffi::c_int);
        window_destroy_panes(w);
        (*(*sc).wp0).entry.tqe_next = (*w).panes.tqh_first;
        if !(*(*sc).wp0).entry.tqe_next.is_null() {
            (*(*w).panes.tqh_first).entry.tqe_prev = &raw mut (*(*sc).wp0).entry.tqe_next;
        } else {
            (*w).panes.tqh_last = &raw mut (*(*sc).wp0).entry.tqe_next;
        }
        (*w).panes.tqh_first = (*sc).wp0;
        (*(*sc).wp0).entry.tqe_prev = &raw mut (*w).panes.tqh_first;
        window_pane_resize((*sc).wp0, (*w).sx, (*w).sy);
        layout_init(w, (*sc).wp0);
        (*w).active = ::core::ptr::null_mut::<window_pane>();
        window_set_active_pane(w, (*sc).wp0, 0 as ::core::ffi::c_int);
    }
    if !(*sc).flags & SPAWN_RESPAWN != 0 && idx != -(1 as ::core::ffi::c_int) {
        wl = winlink_find_by_index(&raw mut (*s).windows, idx);
        if !wl.is_null() && !(*sc).flags & SPAWN_KILL != 0 {
            xasprintf(
                cause,
                b"index %d in use\0" as *const u8 as *const ::core::ffi::c_char,
                idx,
            );
            return ::core::ptr::null_mut::<winlink>();
        }
        if !wl.is_null() {
            (*wl).flags &= !WINLINK_ALERTFLAGS;
            events_fire_winlink(
                b"window-unlinked\0" as *const u8 as *const ::core::ffi::c_char,
                wl,
            );
            winlink_stack_remove(&raw mut (*s).lastw, wl);
            winlink_remove(&raw mut (*s).windows, wl);
            if (*s).curw == wl {
                (*s).curw = ::core::ptr::null_mut::<winlink>();
                (*sc).flags &= !SPAWN_DETACHED;
            }
        }
    }
    if !(*sc).flags & SPAWN_RESPAWN != 0 {
        if idx == -(1 as ::core::ffi::c_int) {
            idx = (-(1 as ::core::ffi::c_int) as ::core::ffi::c_longlong
                - options_get_number(
                    (*s).options,
                    b"base-index\0" as *const u8 as *const ::core::ffi::c_char,
                )) as ::core::ffi::c_int;
        }
        (*sc).wl = winlink_add(&raw mut (*s).windows, idx);
        if (*sc).wl.is_null() {
            xasprintf(
                cause,
                b"couldn't add window %d\0" as *const u8 as *const ::core::ffi::c_char,
                idx,
            );
            return ::core::ptr::null_mut::<winlink>();
        }
        default_window_size(
            (*sc).tc,
            s,
            ::core::ptr::null_mut::<window>(),
            &raw mut sx,
            &raw mut sy,
            &raw mut xpixel,
            &raw mut ypixel,
            -(1 as ::core::ffi::c_int),
        );
        w = window_create(sx, sy, xpixel, ypixel);
        if w.is_null() {
            winlink_remove(&raw mut (*s).windows, (*sc).wl);
            xasprintf(
                cause,
                b"couldn't create window %d\0" as *const u8 as *const ::core::ffi::c_char,
                idx,
            );
            return ::core::ptr::null_mut::<winlink>();
        }
        if (*s).curw.is_null() {
            (*s).curw = (*sc).wl;
        }
        (*(*sc).wl).session = s;
        (*w).latest = (*sc).tc as *mut ::core::ffi::c_void;
        winlink_set_window((*sc).wl, w);
    } else {
        w = ::core::ptr::null_mut::<window>();
    }
    (*sc).flags |= SPAWN_NONOTIFY;
    wp = spawn_pane(sc, cause);
    if wp.is_null() {
        if !(*sc).flags & SPAWN_RESPAWN != 0 {
            winlink_remove(&raw mut (*s).windows, (*sc).wl);
        }
        return ::core::ptr::null_mut::<winlink>();
    }
    if !(*sc).flags & SPAWN_RESPAWN != 0 {
        free((*w).name as *mut ::core::ffi::c_void);
        if (*sc).name.is_null() {
            (*w).name = default_window_name(w);
        } else {
            (*w).name = xstrdup((*sc).name);
            options_set_number(
                (*w).options,
                b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_longlong,
            );
        }
        window_set_fill_cells(w);
    }
    if !(*sc).flags & SPAWN_DETACHED != 0 {
        session_select(s, (*(*sc).wl).idx);
    }
    if !(*sc).flags & SPAWN_RESPAWN != 0 {
        events_fire_window(
            b"window-created\0" as *const u8 as *const ::core::ffi::c_char,
            w,
        );
        events_fire_winlink(
            b"window-linked\0" as *const u8 as *const ::core::ffi::c_char,
            (*sc).wl,
        );
    }
    session_group_synchronize_from(s);
    return (*sc).wl;
}
#[no_mangle]
pub unsafe extern "C" fn spawn_pane(
    mut sc: *mut spawn_context,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut window_pane {
    let mut item: *mut cmdq_item = (*sc).item;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut s: *mut session = (*sc).s;
    let mut ts: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window = (*(*sc).wl).window;
    let mut new_wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut child: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut ee: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut argv: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut argvp: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut argv0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cwd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut new_cwd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut path: [::core::ffi::c_char; 4096] = [0; 4096];
    let mut cmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tmp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut home: *const ::core::ffi::c_char = find_home();
    let mut actual_cwd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut argc: ::core::ffi::c_int = 0;
    let mut idx: u_int = 0;
    let mut now: termios = termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_line: 0,
        c_cc: [0; 32],
        c2rust_unnamed: termios_input_speed { __ispeed: 0 },
        c2rust_unnamed_0: termios_output_speed { __ospeed: 0 },
    };
    let mut hlimit: u_int = 0;
    let mut ws: winsize = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let mut set: sigset_t = __sigset_t { __val: [0; 16] };
    let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
    let mut key: key_code = 0;
    if !item.is_null() {
        ts = (*cmdq_get_target(item)).s;
        c = cmdq_get_client(item);
    } else {
        ts = s;
        c = (*sc).tc;
    }
    spawn_log(
        b"spawn_pane\0" as *const u8 as *const ::core::ffi::c_char,
        sc,
    );
    if (*sc).flags & SPAWN_MODAL != 0 {
        if !(*sc).flags & SPAWN_FLOATING != 0 {
            xasprintf(
                cause,
                b"modal pane must be floating\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return ::core::ptr::null_mut::<window_pane>();
        }
        if !(*w).modal.is_null() {
            xasprintf(
                cause,
                b"window already has a modal pane\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return ::core::ptr::null_mut::<window_pane>();
        }
    }
    if !(*sc).cwd.is_null() {
        if !item.is_null() {
            cwd = format_single(
                item,
                (*sc).cwd,
                c,
                ts,
                ::core::ptr::null_mut::<winlink>(),
                ::core::ptr::null_mut::<window_pane>(),
            );
        } else {
            cwd = xstrdup((*sc).cwd);
        }
        if *cwd as ::core::ffi::c_int != '/' as i32 {
            xasprintf(
                &raw mut new_cwd,
                b"%s%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                server_client_get_cwd(c, ts),
                if *cwd as ::core::ffi::c_int != '\0' as i32 {
                    b"/\0" as *const u8 as *const ::core::ffi::c_char
                } else {
                    b"\0" as *const u8 as *const ::core::ffi::c_char
                },
                cwd,
            );
            free(cwd as *mut ::core::ffi::c_void);
            cwd = new_cwd;
        }
    } else if !(*sc).flags & SPAWN_RESPAWN != 0 {
        cwd = xstrdup(server_client_get_cwd(c, ts));
    } else {
        cwd = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    hlimit = options_get_number(
        (*s).options,
        b"history-limit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    if (*sc).flags & SPAWN_RESPAWN != 0 {
        if (*(*sc).wp0).fd != -(1 as ::core::ffi::c_int) && !(*sc).flags & SPAWN_KILL != 0 {
            window_pane_index((*sc).wp0, &raw mut idx);
            xasprintf(
                cause,
                b"pane %s:%d.%u still active\0" as *const u8 as *const ::core::ffi::c_char,
                (*s).name,
                (*(*sc).wl).idx,
                idx,
            );
            free(cwd as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<window_pane>();
        }
        if !(*(*sc).wp0).event.is_null() {
            bufferevent_free((*(*sc).wp0).event);
            (*(*sc).wp0).event = ::core::ptr::null_mut::<bufferevent>();
        }
        if (*(*sc).wp0).fd != -(1 as ::core::ffi::c_int) {
            close((*(*sc).wp0).fd);
            (*(*sc).wp0).fd = -(1 as ::core::ffi::c_int);
        }
        window_pane_reset_mode_all((*sc).wp0);
        screen_reinit(&raw mut (*(*sc).wp0).base, 0 as ::core::ffi::c_int);
        if !(*(*sc).wp0).ictx.is_null() {
            input_free((*(*sc).wp0).ictx);
            (*(*sc).wp0).ictx = ::core::ptr::null_mut::<input_ctx>();
        }
        (*(*sc).wp0).offset.used = 0 as size_t;
        (*(*sc).wp0).base_offset = 0 as size_t;
        (*(*sc).wp0).pipe_offset.used = 0 as size_t;
        loop_0 = clients.tqh_first;
        while !loop_0.is_null() {
            if (*loop_0).flags & CLIENT_CONTROL as uint64_t != 0 {
                control_reset_pane(loop_0, (*sc).wp0);
            }
            loop_0 = (*loop_0).entry.tqe_next;
        }
        new_wp = (*sc).wp0;
        (*new_wp).flags &= !(PANE_STATUSREADY | PANE_STATUSDRAWN);
    } else {
        if (*sc).lc.is_null() {
            new_wp = window_add_pane(
                w,
                ::core::ptr::null_mut::<window_pane>(),
                hlimit,
                (*sc).flags,
            );
            layout_init(w, new_wp);
        } else {
            new_wp = window_add_pane(w, (*sc).wp0, hlimit, (*sc).flags);
            if (*sc).flags & SPAWN_ZOOM != 0 {
                layout_assign_pane((*sc).lc, new_wp, 1 as ::core::ffi::c_int);
            } else {
                layout_assign_pane((*sc).lc, new_wp, 0 as ::core::ffi::c_int);
            }
        }
        if (*sc).flags & SPAWN_FLOATING != 0 {
            (*(*new_wp).layout_cell).flags |= LAYOUT_CELL_FLOATING;
        }
        if (*sc).flags & SPAWN_FLOATOVERZOOM != 0 {
            (*new_wp).flags |= PANE_FLOATOVERZOOM;
        }
        if (*w).flags & WINDOW_ZOOMED != 0 {
            (*new_wp).saved_layout_cell = (*new_wp).layout_cell as *mut layout_cell;
        }
    }
    if (*sc).argc == 0 as ::core::ffi::c_int && !(*sc).flags & SPAWN_RESPAWN != 0 {
        cmd = options_get_string(
            (*s).options,
            b"default-command\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if !cmd.is_null() && *cmd as ::core::ffi::c_int != '\0' as i32 {
            argc = 1 as ::core::ffi::c_int;
            argv = &raw mut cmd as *mut *mut ::core::ffi::c_char;
        } else {
            argc = 0 as ::core::ffi::c_int;
            argv = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        }
    } else {
        argc = (*sc).argc;
        argv = (*sc).argv;
    }
    if !cwd.is_null() {
        free((*new_wp).cwd as *mut ::core::ffi::c_void);
        (*new_wp).cwd = cwd;
    }
    if argc > 0 as ::core::ffi::c_int {
        cmd_free_argv((*new_wp).argc, (*new_wp).argv);
        (*new_wp).argc = argc;
        (*new_wp).argv = cmd_copy_argv(argc, argv);
    }
    child = environ_for_session(s, 0 as ::core::ffi::c_int);
    if !(*sc).environ.is_null() {
        environ_copy((*sc).environ, child);
    }
    environ_set(
        child,
        b"TMUX_PANE\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
        b"%%%u\0" as *const u8 as *const ::core::ffi::c_char,
        (*new_wp).id,
    );
    if !c.is_null() && (*c).session.is_null() {
        ee = environ_find(
            (*c).environ,
            b"PATH\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if !ee.is_null() {
            environ_set(
                child,
                b"PATH\0" as *const u8 as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*ee).value,
            );
        }
    }
    if environ_find(child, b"PATH\0" as *const u8 as *const ::core::ffi::c_char).is_null() {
        environ_set(
            child,
            b"PATH\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            _PATH_DEFPATH.as_ptr(),
        );
    }
    if !(*sc).flags & SPAWN_RESPAWN != 0 {
        tmp = options_get_string(
            (*s).options,
            b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if checkshell(tmp) == 0 {
            tmp = _PATH_BSHELL.as_ptr();
        }
        free((*new_wp).shell as *mut ::core::ffi::c_void);
        (*new_wp).shell = xstrdup(tmp);
    }
    environ_set(
        child,
        b"SHELL\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        (*new_wp).shell,
    );
    log_debug(
        b"%s: shell=%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"spawn_pane\0" as *const u8 as *const ::core::ffi::c_char,
        (*new_wp).shell,
    );
    if (*new_wp).argc != 0 as ::core::ffi::c_int {
        cp = cmd_stringify_argv((*new_wp).argc, (*new_wp).argv);
        log_debug(
            b"%s: cmd=%s\0" as *const u8 as *const ::core::ffi::c_char,
            b"spawn_pane\0" as *const u8 as *const ::core::ffi::c_char,
            cp,
        );
        free(cp as *mut ::core::ffi::c_void);
    }
    log_debug(
        b"%s: cwd=%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"spawn_pane\0" as *const u8 as *const ::core::ffi::c_char,
        (*new_wp).cwd,
    );
    cmd_log_argv(
        (*new_wp).argc,
        (*new_wp).argv,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"spawn_pane\0" as *const u8 as *const ::core::ffi::c_char,
    );
    environ_log(
        child,
        b"%s: environment \0" as *const u8 as *const ::core::ffi::c_char,
        b"spawn_pane\0" as *const u8 as *const ::core::ffi::c_char,
    );
    memset(
        &raw mut ws as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<winsize>() as size_t,
    );
    ws.ws_col = (*(*new_wp).base.grid).sx as ::core::ffi::c_ushort;
    ws.ws_row = (*(*new_wp).base.grid).sy as ::core::ffi::c_ushort;
    ws.ws_xpixel = (*w).xpixel.wrapping_mul(ws.ws_col as u_int) as ::core::ffi::c_ushort;
    ws.ws_ypixel = (*w).ypixel.wrapping_mul(ws.ws_row as u_int) as ::core::ffi::c_ushort;
    sigfillset(&raw mut set);
    sigprocmask(SIG_BLOCK, &raw mut set, &raw mut oldset);
    if (*sc).flags & SPAWN_EMPTY != 0 {
        (*new_wp).flags |= PANE_EMPTY;
        (*new_wp).base.mode &= !MODE_CURSOR;
        (*new_wp).base.mode |= MODE_CRLF;
    } else {
        (*new_wp).flags &= !PANE_EMPTY;
        if !getcwd(
            &raw mut path as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>() as size_t,
        )
        .is_null()
        {
            if chdir((*new_wp).cwd) == 0 as ::core::ffi::c_int {
                actual_cwd = (*new_wp).cwd;
            } else if !home.is_null() && chdir(home) == 0 as ::core::ffi::c_int {
                actual_cwd = home;
            } else if chdir(b"/\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                actual_cwd = b"/\0" as *const u8 as *const ::core::ffi::c_char;
            }
        }
        (*new_wp).pid = fdforkpty(
            ptm_fd,
            &raw mut (*new_wp).fd,
            &raw mut (*new_wp).tty as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<termios>(),
            &raw mut ws,
        );
        if (*new_wp).pid == -(1 as ::core::ffi::c_int) {
            xasprintf(
                cause,
                b"fork failed: %s\0" as *const u8 as *const ::core::ffi::c_char,
                strerror(*__errno_location()),
            );
            (*new_wp).fd = -(1 as ::core::ffi::c_int);
            if !(*sc).flags & SPAWN_RESPAWN != 0 {
                server_client_remove_pane(new_wp);
                layout_close_pane(new_wp);
                window_remove_pane(w, new_wp);
            }
            sigprocmask(
                SIG_SETMASK,
                &raw mut oldset,
                ::core::ptr::null_mut::<sigset_t>(),
            );
            environ_free(child);
            return ::core::ptr::null_mut::<window_pane>();
        }
        if (*new_wp).pid != 0 as ::core::ffi::c_int {
            if !actual_cwd.is_null()
                && chdir(&raw mut path as *mut ::core::ffi::c_char) != 0 as ::core::ffi::c_int
                && (home.is_null() || chdir(home) != 0 as ::core::ffi::c_int)
            {
                chdir(b"/\0" as *const u8 as *const ::core::ffi::c_char);
            }
        } else {
            if systemd_move_to_new_cgroup(cause) < 0 as ::core::ffi::c_int {
                log_debug(
                    b"%s: moving pane to new cgroup failed: %s\0" as *const u8
                        as *const ::core::ffi::c_char,
                    b"spawn_pane\0" as *const u8 as *const ::core::ffi::c_char,
                    *cause,
                );
                free(*cause as *mut ::core::ffi::c_void);
            }
            if !actual_cwd.is_null() {
                environ_set(
                    child,
                    b"PWD\0" as *const u8 as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    actual_cwd,
                );
            }
            if tcgetattr(STDIN_FILENO, &raw mut now) != 0 as ::core::ffi::c_int {
                _exit(1 as ::core::ffi::c_int);
            }
            if !(*s).tio.is_null() {
                memcpy(
                    &raw mut now.c_cc as *mut cc_t as *mut ::core::ffi::c_void,
                    &raw mut (*(*s).tio).c_cc as *mut cc_t as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<[cc_t; 32]>() as size_t,
                );
            }
            key = options_get_number(
                global_options,
                b"backspace\0" as *const u8 as *const ::core::ffi::c_char,
            ) as key_code;
            if key >= 0x7f as key_code {
                now.c_cc[VERASE as usize] = '\u{7f}' as i32 as cc_t;
            } else {
                now.c_cc[VERASE as usize] = key as cc_t;
            }
            now.c_iflag |= IUTF8 as tcflag_t;
            if tcsetattr(STDIN_FILENO, TCSANOW, &raw mut now) != 0 as ::core::ffi::c_int {
                _exit(1 as ::core::ffi::c_int);
            }
            proc_clear_signals(server_proc, 1 as ::core::ffi::c_int);
            closefrom(STDERR_FILENO + 1 as ::core::ffi::c_int);
            sigprocmask(
                SIG_SETMASK,
                &raw mut oldset,
                ::core::ptr::null_mut::<sigset_t>(),
            );
            log_close();
            environ_push(child);
            if (*new_wp).argc != 0 as ::core::ffi::c_int
                && (*new_wp).argc != 1 as ::core::ffi::c_int
            {
                argvp = cmd_copy_argv((*new_wp).argc, (*new_wp).argv);
                execvp(
                    *argvp.offset(0 as ::core::ffi::c_int as isize),
                    argvp as *const *mut ::core::ffi::c_char,
                );
                _exit(1 as ::core::ffi::c_int);
            }
            cp = strrchr((*new_wp).shell, '/' as i32);
            if (*new_wp).argc == 1 as ::core::ffi::c_int {
                tmp = *(*new_wp).argv.offset(0 as ::core::ffi::c_int as isize);
                if !cp.is_null()
                    && *cp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        != '\0' as i32
                {
                    xasprintf(
                        &raw mut argv0,
                        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                        cp.offset(1 as ::core::ffi::c_int as isize),
                    );
                } else {
                    xasprintf(
                        &raw mut argv0,
                        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                        (*new_wp).shell,
                    );
                }
                execl(
                    (*new_wp).shell,
                    argv0,
                    b"-c\0" as *const u8 as *const ::core::ffi::c_char,
                    tmp,
                    NULL as *mut ::core::ffi::c_char,
                );
                _exit(1 as ::core::ffi::c_int);
            }
            if !cp.is_null()
                && *cp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
            {
                xasprintf(
                    &raw mut argv0,
                    b"-%s\0" as *const u8 as *const ::core::ffi::c_char,
                    cp.offset(1 as ::core::ffi::c_int as isize),
                );
            } else {
                xasprintf(
                    &raw mut argv0,
                    b"-%s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*new_wp).shell,
                );
            }
            execl((*new_wp).shell, argv0, NULL as *mut ::core::ffi::c_char);
            _exit(1 as ::core::ffi::c_int);
        }
    }
    if !(*new_wp).flags & PANE_EMPTY != 0 {
        xasprintf(
            &raw mut cp,
            b"tmux(%lu).%%%u\0" as *const u8 as *const ::core::ffi::c_char,
            getpid() as ::core::ffi::c_long,
            (*new_wp).id,
        );
        utempter_add_record((*new_wp).fd, cp);
        kill(getpid(), SIGCHLD);
        free(cp as *mut ::core::ffi::c_void);
    }
    (*new_wp).flags &= !PANE_EXITED;
    sigprocmask(
        SIG_SETMASK,
        &raw mut oldset,
        ::core::ptr::null_mut::<sigset_t>(),
    );
    window_pane_set_event(new_wp);
    environ_free(child);
    spawn_fire_pane_created(sc, new_wp);
    if (*sc).flags & SPAWN_RESPAWN != 0 {
        return new_wp;
    }
    if (*sc).flags & SPAWN_MODAL != 0 {
        (*w).modal_last = (*w).active;
        (*w).modal = new_wp;
        window_redraw_active_switch(w, new_wp);
        if (*sc).flags & SPAWN_NONOTIFY != 0 {
            window_set_active_pane(w, new_wp, 0 as ::core::ffi::c_int);
        } else {
            window_set_active_pane(w, new_wp, 1 as ::core::ffi::c_int);
        }
    } else if (!(*sc).flags & SPAWN_DETACHED != 0 || (*w).active.is_null()) && (*w).modal.is_null()
    {
        if (*sc).flags & SPAWN_NONOTIFY != 0 {
            window_set_active_pane(w, new_wp, 0 as ::core::ffi::c_int);
        } else {
            window_set_active_pane(w, new_wp, 1 as ::core::ffi::c_int);
        }
    }
    if !(*sc).flags & SPAWN_NONOTIFY != 0 {
        events_fire_window(
            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
            w,
        );
    }
    return new_wp;
}
unsafe extern "C" fn spawn_editor_free(mut es: *mut spawn_editor_state) {
    unlink((*es).path);
    free((*es).path as *mut ::core::ffi::c_void);
    free(es as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn spawn_cancel_editor(mut es: *mut spawn_editor_state) {
    if es.is_null() {
        return;
    }
    (*es).cb = None;
    (*es).arg = NULL;
}
#[no_mangle]
pub unsafe extern "C" fn spawn_get_editor_pid(mut es: *mut spawn_editor_state) -> pid_t {
    if es.is_null() {
        return -(1 as pid_t);
    }
    return (*es).pid;
}
#[no_mangle]
pub unsafe extern "C" fn spawn_editor_finish(mut wp: *mut window_pane) {
    let mut es: *mut spawn_editor_state = (*wp).editor as *mut spawn_editor_state;
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: off_t = 0 as off_t;
    let mut status: ::core::ffi::c_int = 128 as ::core::ffi::c_int + SIGHUP;
    if es.is_null() {
        return;
    }
    (*wp).editor = ::core::ptr::null_mut::<spawn_editor_state>();
    if (*wp).flags & PANE_STATUSREADY != 0 {
        if (*wp).status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            status = ((*wp).status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
        } else if (((*wp).status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
            as ::core::ffi::c_schar as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int
            > 0 as ::core::ffi::c_int
        {
            status = ((*wp).status & 0x7f as ::core::ffi::c_int) + 128 as ::core::ffi::c_int;
        }
    }
    if (*es).cb.is_none() {
        spawn_editor_free(es);
        return;
    }
    if status != 0 as ::core::ffi::c_int {
        (*es).cb.expect("non-null function pointer")(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as size_t,
            (*es).arg,
        );
        spawn_editor_free(es);
        return;
    }
    f = fopen(
        (*es).path,
        b"r\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut FILE;
    if !f.is_null() {
        if fseeko(f, 0 as __off_t, SEEK_END) == 0 as ::core::ffi::c_int {
            len = ftello(f) as off_t;
            if len > 0 as off_t && len as uintmax_t <= SIZE_MAX as uintmax_t {
                if fseeko(f, 0 as __off_t, SEEK_SET) == 0 as ::core::ffi::c_int {
                    buf = malloc(len as size_t) as *mut ::core::ffi::c_char;
                    if !buf.is_null()
                        && fread(
                            buf as *mut ::core::ffi::c_void,
                            len as size_t,
                            1 as size_t,
                            f,
                        ) != 1 as ::core::ffi::c_ulong
                    {
                        free(buf as *mut ::core::ffi::c_void);
                        buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        len = 0 as off_t;
                    }
                }
            } else {
                len = 0 as off_t;
            }
        }
        fclose(f);
    }
    (*es).cb.expect("non-null function pointer")(buf, len as size_t, (*es).arg);
    spawn_editor_free(es);
}
#[no_mangle]
pub unsafe extern "C" fn spawn_editor(
    mut c: *mut client,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut cb: spawn_finish_edit_cb,
    mut arg: *mut ::core::ffi::c_void,
) -> *mut spawn_editor_state {
    let mut es: *mut spawn_editor_state = ::core::ptr::null_mut::<spawn_editor_state>();
    let mut sc: spawn_context = spawn_context {
        item: ::core::ptr::null_mut::<cmdq_item>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        tc: ::core::ptr::null_mut::<client>(),
        wp0: ::core::ptr::null_mut::<window_pane>(),
        lc: ::core::ptr::null_mut::<layout_cell>(),
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        argv: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        argc: 0,
        environ: ::core::ptr::null_mut::<environ>(),
        idx: 0,
        cwd: ::core::ptr::null::<::core::ffi::c_char>(),
        flags: 0,
    };
    let mut s: *mut session = (*c).session;
    let mut wl: *mut winlink = (*s).curw;
    let mut w: *mut window = (*wl).window;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lg: layout_geometry = layout_geometry {
        sx: 0,
        sy: 0,
        xoff: 0,
        yoff: 0,
    };
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut path: [::core::ffi::c_char; 19] =
        ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"/tmp/tmux.XXXXXXXX\0");
    let mut editor: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut fd: ::core::ffi::c_int = 0;
    if !(*w).modal.is_null() {
        return ::core::ptr::null_mut::<spawn_editor_state>();
    }
    editor = options_get_string(
        global_options,
        b"editor\0" as *const u8 as *const ::core::ffi::c_char,
    );
    fd = mkstemp(&raw mut path as *mut ::core::ffi::c_char);
    if fd == -(1 as ::core::ffi::c_int) {
        return ::core::ptr::null_mut::<spawn_editor_state>();
    }
    f = fdopen(fd, b"w\0" as *const u8 as *const ::core::ffi::c_char);
    if f.is_null() {
        close(fd);
        unlink(&raw mut path as *mut ::core::ffi::c_char);
        return ::core::ptr::null_mut::<spawn_editor_state>();
    }
    if fwrite(buf as *const ::core::ffi::c_void, len, 1 as size_t, f) != 1 as ::core::ffi::c_ulong {
        fclose(f);
        unlink(&raw mut path as *mut ::core::ffi::c_char);
        return ::core::ptr::null_mut::<spawn_editor_state>();
    }
    fclose(f);
    es = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<spawn_editor_state>() as size_t,
    ) as *mut spawn_editor_state;
    (*es).path = xstrdup(&raw mut path as *mut ::core::ffi::c_char);
    (*es).cb = cb;
    (*es).arg = arg;
    lg.sx = (*w).sx.wrapping_mul(9 as u_int).wrapping_div(10 as u_int);
    lg.sy = (*w).sy.wrapping_mul(9 as u_int).wrapping_div(10 as u_int);
    lg.xoff = (*w)
        .sx
        .wrapping_div(2 as u_int)
        .wrapping_sub(lg.sx.wrapping_div(2 as u_int)) as ::core::ffi::c_int;
    lg.yoff = (*w)
        .sy
        .wrapping_div(2 as u_int)
        .wrapping_sub(lg.sy.wrapping_div(2 as u_int)) as ::core::ffi::c_int;
    window_push_zoom(w, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    lc = layout_floating_pane(w, ::core::ptr::null_mut::<window_pane>(), &raw mut lg);
    if lc.is_null() {
        window_pop_zoom(w);
        spawn_editor_free(es);
        return ::core::ptr::null_mut::<spawn_editor_state>();
    }
    xasprintf(
        &raw mut cmd,
        b"%s %s\0" as *const u8 as *const ::core::ffi::c_char,
        editor,
        &raw mut path as *mut ::core::ffi::c_char,
    );
    env = environ_create();
    sc.s = s;
    sc.wl = wl;
    sc.tc = c;
    sc.wp0 = (*w).active;
    sc.lc = lc;
    sc.argc = 1 as ::core::ffi::c_int;
    sc.argv = &raw mut cmd;
    sc.environ = env;
    sc.idx = -(1 as ::core::ffi::c_int);
    sc.cwd = _PATH_TMP.as_ptr();
    sc.flags = SPAWN_FLOATING | SPAWN_MODAL | SPAWN_FLOATOVERZOOM;
    wp = spawn_pane(&raw mut sc, &raw mut cause);
    free(cmd as *mut ::core::ffi::c_void);
    environ_free(env);
    if wp.is_null() {
        free(cause as *mut ::core::ffi::c_void);
        window_pop_zoom(w);
        spawn_editor_free(es);
        return ::core::ptr::null_mut::<spawn_editor_state>();
    }
    window_pop_zoom(w);
    options_set_number(
        (*wp).options,
        b"remain-on-exit\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_longlong,
    );
    (*es).pid = (*wp).pid;
    (*wp).editor = es as *mut spawn_editor_state;
    return es;
}

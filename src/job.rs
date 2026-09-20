pub use crate::src::shared::posix_terminal::{winsize, TIOCSWINSZ};
pub use crate::src::shared::socket::{
    __socket_type, AF_UNIX, PF_LOCAL, PF_UNIX, PF_UNSPEC, SOCK_CLOEXEC, SOCK_DCCP, SOCK_DGRAM,
    SOCK_NONBLOCK, SOCK_PACKET, SOCK_RAW, SOCK_RDM, SOCK_SEQPACKET, SOCK_STREAM,
};
pub use crate::src::shared::signal::{
    __sigset_t, sigset_t, SIGCONT, SIGTERM, SIGTTIN, SIGTTOU, SIG_BLOCK, SIG_SETMASK,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::event::{EV_READ, EV_WRITE};
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
    pub type tmuxproc;
    fn ioctl(__fd: ::core::ffi::c_int, __request: ::core::ffi::c_ulong, ...) -> ::core::ffi::c_int;
    fn socketpair(
        __domain: ::core::ffi::c_int,
        __type: ::core::ffi::c_int,
        __protocol: ::core::ffi::c_int,
        __fds: *mut ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn shutdown(__fd: ::core::ffi::c_int, __how: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn kill(__pid: __pid_t, __sig: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn killpg(__pgrp: __pid_t, __sig: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn sigfillset(__set: *mut sigset_t) -> ::core::ffi::c_int;
    fn sigprocmask(
        __how: ::core::ffi::c_int,
        __set: *const sigset_t,
        __oset: *mut sigset_t,
    ) -> ::core::ffi::c_int;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn closefrom(__lowfd: ::core::ffi::c_int);
    fn chdir(__path: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn dup2(__fd: ::core::ffi::c_int, __fd2: ::core::ffi::c_int) -> ::core::ffi::c_int;
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
    fn fork() -> __pid_t;
    fn open(
        __file: *const ::core::ffi::c_char,
        __oflag: ::core::ffi::c_int,
        ...
    ) -> ::core::ffi::c_int;
    fn setenv(
        __name: *const ::core::ffi::c_char,
        __value: *const ::core::ffi::c_char,
        __replace: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strlcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    fn evbuffer_get_length(buf: *const evbuffer) -> size_t;
    fn bufferevent_free(bufev: *mut bufferevent);
    fn bufferevent_get_output(bufev: *mut bufferevent) -> *mut evbuffer;
    fn bufferevent_enable(
        bufev: *mut bufferevent,
        event: ::core::ffi::c_short,
    ) -> ::core::ffi::c_int;
    fn bufferevent_disable(
        bufev: *mut bufferevent,
        event: ::core::ffi::c_short,
    ) -> ::core::ffi::c_int;
    fn bufferevent_new(
        fd: ::core::ffi::c_int,
        readcb: bufferevent_data_cb,
        writecb: bufferevent_data_cb,
        errorcb: bufferevent_event_cb,
        cbarg: *mut ::core::ffi::c_void,
    ) -> *mut bufferevent;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn fdforkpty(
        _: ::core::ffi::c_int,
        _: *mut ::core::ffi::c_int,
        _: *mut ::core::ffi::c_char,
        _: *mut termios,
        _: *mut winsize,
    ) -> pid_t;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    static mut global_s_options: *mut options;
    static mut ptm_fd: ::core::ffi::c_int;
    fn checkshell(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn setblocking(_: ::core::ffi::c_int, _: ::core::ffi::c_int);
    fn shell_argv0(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn find_home() -> *const ::core::ffi::c_char;
    fn proc_clear_signals(_: *mut tmuxproc, _: ::core::ffi::c_int);
    static mut cfg_finished: ::core::ffi::c_int;
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn environ_free(_: *mut environ);
    fn environ_copy(_: *mut environ, _: *mut environ);
    fn environ_set(
        _: *mut environ,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn environ_push(_: *mut environ);
    fn environ_for_session(_: *mut session, _: ::core::ffi::c_int) -> *mut environ;
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
    fn cmd_stringify_argv(
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn cmdq_print(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    static mut server_proc: *mut tmuxproc;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatal(_: *const ::core::ffi::c_char, ...) -> !;
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
}

pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const SHUT_RDWR: C2RustUnnamed = 2;
pub const SHUT_WR: C2RustUnnamed = 1;
pub const SHUT_RD: C2RustUnnamed = 0;
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
pub struct job {
    pub state: C2RustUnnamed_37,
    pub flags: ::core::ffi::c_int,
    pub cmd: *mut ::core::ffi::c_char,
    pub pid: pid_t,
    pub tty: [::core::ffi::c_char; 32],
    pub status: ::core::ffi::c_int,
    pub fd: ::core::ffi::c_int,
    pub event: *mut bufferevent,
    pub updatecb: job_update_cb,
    pub completecb: job_complete_cb,
    pub freecb: job_free_cb,
    pub data: *mut ::core::ffi::c_void,
    pub entry: C2RustUnnamed_36,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_36 {
    pub le_next: *mut job,
    pub le_prev: *mut *mut job,
}
pub type job_free_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type job_complete_cb = Option<unsafe extern "C" fn(*mut job) -> ()>;
pub type job_update_cb = Option<unsafe extern "C" fn(*mut job) -> ()>;
pub type C2RustUnnamed_37 = ::core::ffi::c_uint;
pub const JOB_CLOSED: C2RustUnnamed_37 = 2;
pub const JOB_DEAD: C2RustUnnamed_37 = 1;
pub const JOB_RUNNING: C2RustUnnamed_37 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct joblist {
    pub lh_first: *mut job,
}

pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const O_RDWR: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;
pub const _PATH_BSHELL: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"/bin/sh\0") };
pub const _PATH_DEVNULL: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"/dev/null\0") };
pub const JOB_NOWAIT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const JOB_KEEPWRITE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const JOB_PTY: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const JOB_DEFAULTSHELL: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const JOB_SHOWSTDERR: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
static mut all_jobs: joblist = joblist {
    lh_first: ::core::ptr::null::<job>() as *mut job,
};
#[no_mangle]
pub unsafe extern "C" fn job_run(
    mut cmd: *const ::core::ffi::c_char,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
    mut e: *mut environ,
    mut s: *mut session,
    mut cwd: *const ::core::ffi::c_char,
    mut updatecb: job_update_cb,
    mut completecb: job_complete_cb,
    mut freecb: job_free_cb,
    mut data: *mut ::core::ffi::c_void,
    mut flags: ::core::ffi::c_int,
    mut sx: ::core::ffi::c_int,
    mut sy: ::core::ffi::c_int,
) -> *mut job {
    let mut current_block: u64;
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut pid: pid_t = 0;
    let mut nullfd: ::core::ffi::c_int = 0;
    let mut out: [::core::ffi::c_int; 2] = [0; 2];
    let mut master: ::core::ffi::c_int = 0;
    let mut do_close: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut home: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut shell: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut set: sigset_t = __sigset_t { __val: [0; 16] };
    let mut oldset: sigset_t = __sigset_t { __val: [0; 16] };
    let mut ws: winsize = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let mut argvp: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut tty: [::core::ffi::c_char; 32] = [0; 32];
    let mut argv0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    env = environ_for_session(s, (cfg_finished == 0) as ::core::ffi::c_int);
    if !e.is_null() {
        environ_copy(e, env);
    }
    if !flags & JOB_DEFAULTSHELL != 0 {
        shell = _PATH_BSHELL.as_ptr();
    } else {
        if !s.is_null() {
            oo = (*s).options;
        } else {
            oo = global_s_options;
        }
        shell = options_get_string(
            oo,
            b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if checkshell(shell) == 0 {
            shell = _PATH_BSHELL.as_ptr();
        }
    }
    argv0 = shell_argv0(shell, 0 as ::core::ffi::c_int);
    sigfillset(&raw mut set);
    sigprocmask(SIG_BLOCK, &raw mut set, &raw mut oldset);
    if flags & JOB_PTY != 0 {
        memset(
            &raw mut ws as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<winsize>() as size_t,
        );
        ws.ws_col = sx as ::core::ffi::c_ushort;
        ws.ws_row = sy as ::core::ffi::c_ushort;
        pid = fdforkpty(
            ptm_fd,
            &raw mut master,
            &raw mut tty as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<termios>(),
            &raw mut ws,
        );
        current_block = 224731115979188411;
    } else if socketpair(
        AF_UNIX,
        SOCK_STREAM as ::core::ffi::c_int,
        PF_UNSPEC,
        &raw mut out as *mut ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        current_block = 12393940290395533062;
    } else {
        pid = fork() as pid_t;
        current_block = 224731115979188411;
    }
    match current_block {
        224731115979188411 => {
            if cmd.is_null() {
                cmd_log_argv(
                    argc,
                    argv,
                    b"%s:\0" as *const u8 as *const ::core::ffi::c_char,
                    b"job_run\0" as *const u8 as *const ::core::ffi::c_char,
                );
                log_debug(
                    b"%s: cwd=%s, shell=%s\0" as *const u8 as *const ::core::ffi::c_char,
                    b"job_run\0" as *const u8 as *const ::core::ffi::c_char,
                    if cwd.is_null() {
                        b"\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        cwd
                    },
                    shell,
                );
            } else {
                log_debug(
                    b"%s: cmd=%s, cwd=%s, shell=%s\0" as *const u8 as *const ::core::ffi::c_char,
                    b"job_run\0" as *const u8 as *const ::core::ffi::c_char,
                    cmd,
                    if cwd.is_null() {
                        b"\0" as *const u8 as *const ::core::ffi::c_char
                    } else {
                        cwd
                    },
                    shell,
                );
            }
            match pid {
                -1 => {
                    if !flags & JOB_PTY != 0 {
                        close(out[0 as ::core::ffi::c_int as usize]);
                        close(out[1 as ::core::ffi::c_int as usize]);
                    }
                }
                0 => {
                    proc_clear_signals(server_proc, 1 as ::core::ffi::c_int);
                    sigprocmask(
                        SIG_SETMASK,
                        &raw mut oldset,
                        ::core::ptr::null_mut::<sigset_t>(),
                    );
                    if !cwd.is_null() {
                        if chdir(cwd) == 0 as ::core::ffi::c_int {
                            environ_set(
                                env,
                                b"PWD\0" as *const u8 as *const ::core::ffi::c_char,
                                0 as ::core::ffi::c_int,
                                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                cwd,
                            );
                        } else {
                            home = find_home();
                            if !home.is_null() && chdir(home) == 0 as ::core::ffi::c_int {
                                environ_set(
                                    env,
                                    b"PWD\0" as *const u8 as *const ::core::ffi::c_char,
                                    0 as ::core::ffi::c_int,
                                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                                    home,
                                );
                            } else if chdir(b"/\0" as *const u8 as *const ::core::ffi::c_char)
                                == 0 as ::core::ffi::c_int
                            {
                                environ_set(
                                    env,
                                    b"PWD\0" as *const u8 as *const ::core::ffi::c_char,
                                    0 as ::core::ffi::c_int,
                                    b"/\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                            } else {
                                _exit(1 as ::core::ffi::c_int);
                            }
                        }
                    }
                    environ_push(env);
                    environ_free(env);
                    if !flags & JOB_PTY != 0 {
                        if dup2(out[1 as ::core::ffi::c_int as usize], STDIN_FILENO)
                            == -(1 as ::core::ffi::c_int)
                        {
                            _exit(1 as ::core::ffi::c_int);
                        }
                        do_close = (do_close != 0
                            && out[1 as ::core::ffi::c_int as usize] != STDIN_FILENO)
                            as ::core::ffi::c_int;
                        if dup2(out[1 as ::core::ffi::c_int as usize], STDOUT_FILENO)
                            == -(1 as ::core::ffi::c_int)
                        {
                            _exit(1 as ::core::ffi::c_int);
                        }
                        do_close = (do_close != 0
                            && out[1 as ::core::ffi::c_int as usize] != STDOUT_FILENO)
                            as ::core::ffi::c_int;
                        if flags & JOB_SHOWSTDERR != 0 {
                            if dup2(out[1 as ::core::ffi::c_int as usize], STDERR_FILENO)
                                == -(1 as ::core::ffi::c_int)
                            {
                                _exit(1 as ::core::ffi::c_int);
                            }
                            do_close = (do_close != 0
                                && out[1 as ::core::ffi::c_int as usize] != STDERR_FILENO)
                                as ::core::ffi::c_int;
                        } else {
                            nullfd = open(_PATH_DEVNULL.as_ptr(), O_RDWR);
                            if nullfd == -(1 as ::core::ffi::c_int) {
                                _exit(1 as ::core::ffi::c_int);
                            }
                            if dup2(nullfd, STDERR_FILENO) == -(1 as ::core::ffi::c_int) {
                                _exit(1 as ::core::ffi::c_int);
                            }
                            if nullfd != STDERR_FILENO {
                                close(nullfd);
                            }
                        }
                        if do_close != 0 {
                            close(out[1 as ::core::ffi::c_int as usize]);
                        }
                        close(out[0 as ::core::ffi::c_int as usize]);
                    }
                    closefrom(STDERR_FILENO + 1 as ::core::ffi::c_int);
                    if !cmd.is_null() {
                        if flags & JOB_DEFAULTSHELL != 0 {
                            setenv(
                                b"SHELL\0" as *const u8 as *const ::core::ffi::c_char,
                                shell,
                                1 as ::core::ffi::c_int,
                            );
                        }
                        execl(
                            shell,
                            argv0,
                            b"-c\0" as *const u8 as *const ::core::ffi::c_char,
                            cmd,
                            NULL as *mut ::core::ffi::c_char,
                        );
                        _exit(1 as ::core::ffi::c_int);
                    } else {
                        argvp = cmd_copy_argv(argc, argv);
                        execvp(
                            *argvp.offset(0 as ::core::ffi::c_int as isize),
                            argvp as *const *mut ::core::ffi::c_char,
                        );
                        _exit(1 as ::core::ffi::c_int);
                    }
                }
                _ => {
                    sigprocmask(
                        SIG_SETMASK,
                        &raw mut oldset,
                        ::core::ptr::null_mut::<sigset_t>(),
                    );
                    environ_free(env);
                    free(argv0 as *mut ::core::ffi::c_void);
                    job = xcalloc(1 as size_t, ::core::mem::size_of::<job>() as size_t) as *mut job;
                    (*job).state = JOB_RUNNING;
                    (*job).flags = flags;
                    if !cmd.is_null() {
                        (*job).cmd = xstrdup(cmd);
                    } else {
                        (*job).cmd = cmd_stringify_argv(argc, argv);
                    }
                    (*job).pid = pid;
                    if flags & JOB_PTY != 0 {
                        strlcpy(
                            &raw mut (*job).tty as *mut ::core::ffi::c_char,
                            &raw mut tty as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                        );
                    }
                    (*job).status = 0 as ::core::ffi::c_int;
                    (*job).entry.le_next = all_jobs.lh_first;
                    if !(*job).entry.le_next.is_null() {
                        (*all_jobs.lh_first).entry.le_prev = &raw mut (*job).entry.le_next;
                    }
                    all_jobs.lh_first = job;
                    (*job).entry.le_prev = &raw mut all_jobs.lh_first;
                    (*job).updatecb = updatecb;
                    (*job).completecb = completecb;
                    (*job).freecb = freecb;
                    (*job).data = data;
                    if !flags & JOB_PTY != 0 {
                        close(out[1 as ::core::ffi::c_int as usize]);
                        (*job).fd = out[0 as ::core::ffi::c_int as usize];
                    } else {
                        (*job).fd = master;
                    }
                    setblocking((*job).fd, 0 as ::core::ffi::c_int);
                    (*job).event = bufferevent_new(
                        (*job).fd,
                        Some(
                            job_read_callback
                                as unsafe extern "C" fn(
                                    *mut bufferevent,
                                    *mut ::core::ffi::c_void,
                                ) -> (),
                        ),
                        Some(
                            job_write_callback
                                as unsafe extern "C" fn(
                                    *mut bufferevent,
                                    *mut ::core::ffi::c_void,
                                ) -> (),
                        ),
                        Some(
                            job_error_callback
                                as unsafe extern "C" fn(
                                    *mut bufferevent,
                                    ::core::ffi::c_short,
                                    *mut ::core::ffi::c_void,
                                ) -> (),
                        ),
                        job as *mut ::core::ffi::c_void,
                    );
                    if (*job).event.is_null() {
                        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                    bufferevent_enable((*job).event, (EV_READ | EV_WRITE) as ::core::ffi::c_short);
                    log_debug(
                        b"run job %p: %s, pid %ld\0" as *const u8 as *const ::core::ffi::c_char,
                        job,
                        (*job).cmd,
                        (*job).pid as ::core::ffi::c_long,
                    );
                    return job;
                }
            }
        }
        _ => {}
    }
    sigprocmask(
        SIG_SETMASK,
        &raw mut oldset,
        ::core::ptr::null_mut::<sigset_t>(),
    );
    environ_free(env);
    free(argv0 as *mut ::core::ffi::c_void);
    return ::core::ptr::null_mut::<job>();
}
#[no_mangle]
pub unsafe extern "C" fn job_transfer(
    mut job: *mut job,
    mut pid: *mut pid_t,
    mut tty: *mut ::core::ffi::c_char,
    mut ttylen: size_t,
) -> ::core::ffi::c_int {
    let mut fd: ::core::ffi::c_int = (*job).fd;
    log_debug(
        b"transfer job %p: %s\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        (*job).cmd,
    );
    if !pid.is_null() {
        *pid = (*job).pid;
    }
    if !tty.is_null() {
        strlcpy(tty, &raw mut (*job).tty as *mut ::core::ffi::c_char, ttylen);
    }
    if !(*job).entry.le_next.is_null() {
        (*(*job).entry.le_next).entry.le_prev = (*job).entry.le_prev;
    }
    *(*job).entry.le_prev = (*job).entry.le_next;
    free((*job).cmd as *mut ::core::ffi::c_void);
    if (*job).freecb.is_some() && !(*job).data.is_null() {
        (*job).freecb.expect("non-null function pointer")((*job).data);
    }
    if !(*job).event.is_null() {
        bufferevent_free((*job).event);
    }
    free(job as *mut ::core::ffi::c_void);
    return fd;
}
#[no_mangle]
pub unsafe extern "C" fn job_free(mut job: *mut job) {
    log_debug(
        b"free job %p: %s\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        (*job).cmd,
    );
    if !(*job).entry.le_next.is_null() {
        (*(*job).entry.le_next).entry.le_prev = (*job).entry.le_prev;
    }
    *(*job).entry.le_prev = (*job).entry.le_next;
    free((*job).cmd as *mut ::core::ffi::c_void);
    if (*job).freecb.is_some() && !(*job).data.is_null() {
        (*job).freecb.expect("non-null function pointer")((*job).data);
    }
    if (*job).pid != -(1 as ::core::ffi::c_int) {
        kill((*job).pid as __pid_t, SIGTERM);
    }
    if !(*job).event.is_null() {
        bufferevent_free((*job).event);
    }
    if (*job).fd != -(1 as ::core::ffi::c_int) {
        close((*job).fd);
    }
    free(job as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn job_resize(mut job: *mut job, mut sx: u_int, mut sy: u_int) {
    let mut ws: winsize = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    if (*job).fd == -(1 as ::core::ffi::c_int) || !(*job).flags & JOB_PTY != 0 {
        return;
    }
    log_debug(
        b"resize job %p: %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        sx,
        sy,
    );
    memset(
        &raw mut ws as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<winsize>() as size_t,
    );
    ws.ws_col = sx as ::core::ffi::c_ushort;
    ws.ws_row = sy as ::core::ffi::c_ushort;
    if ioctl((*job).fd, TIOCSWINSZ as ::core::ffi::c_ulong, &raw mut ws)
        == -(1 as ::core::ffi::c_int)
    {
        fatal(b"ioctl failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
}
unsafe extern "C" fn job_read_callback(
    mut bufev: *mut bufferevent,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut job: *mut job = data as *mut job;
    if (*job).updatecb.is_some() {
        (*job).updatecb.expect("non-null function pointer")(job);
    }
}
unsafe extern "C" fn job_write_callback(
    mut bufev: *mut bufferevent,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut job: *mut job = data as *mut job;
    let mut len: size_t = evbuffer_get_length(bufferevent_get_output((*job).event));
    log_debug(
        b"job write %p: %s, pid %ld, output left %zu\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        (*job).cmd,
        (*job).pid as ::core::ffi::c_long,
        len,
    );
    if len == 0 as size_t && !(*job).flags & JOB_KEEPWRITE != 0 {
        shutdown((*job).fd, SHUT_WR as ::core::ffi::c_int);
        bufferevent_disable((*job).event, EV_WRITE as ::core::ffi::c_short);
    }
}
unsafe extern "C" fn job_error_callback(
    mut bufev: *mut bufferevent,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut job: *mut job = data as *mut job;
    log_debug(
        b"job error %p: %s, pid %ld\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        (*job).cmd,
        (*job).pid as ::core::ffi::c_long,
    );
    if (*job).state as ::core::ffi::c_uint == JOB_DEAD as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*job).completecb.is_some() {
            (*job).completecb.expect("non-null function pointer")(job);
        }
        job_free(job);
    } else {
        bufferevent_disable((*job).event, EV_READ as ::core::ffi::c_short);
        (*job).state = JOB_CLOSED;
    };
}
#[no_mangle]
pub unsafe extern "C" fn job_check_died(mut pid: pid_t, mut status: ::core::ffi::c_int) {
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    job = all_jobs.lh_first;
    while !job.is_null() {
        if pid == (*job).pid {
            break;
        }
        job = (*job).entry.le_next;
    }
    if job.is_null() {
        return;
    }
    if status & 0xff as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int {
        if (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int == SIGTTIN
            || (status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int == SIGTTOU
        {
            return;
        }
        killpg((*job).pid as __pid_t, SIGCONT);
        return;
    }
    log_debug(
        b"job died %p: %s, pid %ld\0" as *const u8 as *const ::core::ffi::c_char,
        job,
        (*job).cmd,
        (*job).pid as ::core::ffi::c_long,
    );
    (*job).status = status;
    if (*job).state as ::core::ffi::c_uint
        == JOB_CLOSED as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*job).completecb.is_some() {
            (*job).completecb.expect("non-null function pointer")(job);
        }
        job_free(job);
    } else {
        (*job).pid = -(1 as ::core::ffi::c_int) as pid_t;
        (*job).state = JOB_DEAD;
    };
}
#[no_mangle]
pub unsafe extern "C" fn job_get_status(mut job: *mut job) -> ::core::ffi::c_int {
    return (*job).status;
}
#[no_mangle]
pub unsafe extern "C" fn job_get_data(mut job: *mut job) -> *mut ::core::ffi::c_void {
    return (*job).data;
}
#[no_mangle]
pub unsafe extern "C" fn job_get_event(mut job: *mut job) -> *mut bufferevent {
    return (*job).event;
}
#[no_mangle]
pub unsafe extern "C" fn job_kill_all() {
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    job = all_jobs.lh_first;
    while !job.is_null() {
        if (*job).pid != -(1 as ::core::ffi::c_int) {
            kill((*job).pid as __pid_t, SIGTERM);
        }
        job = (*job).entry.le_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn job_still_running() -> ::core::ffi::c_int {
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    job = all_jobs.lh_first;
    while !job.is_null() {
        if !(*job).flags & JOB_NOWAIT != 0
            && (*job).state as ::core::ffi::c_uint
                == JOB_RUNNING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return 1 as ::core::ffi::c_int;
        }
        job = (*job).entry.le_next;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn job_print_summary(
    mut item: *mut cmdq_item,
    mut blank: ::core::ffi::c_int,
) {
    let mut job: *mut job = ::core::ptr::null_mut::<job>();
    let mut n: u_int = 0 as u_int;
    job = all_jobs.lh_first;
    while !job.is_null() {
        if blank != 0 {
            cmdq_print(
                item,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
            );
            blank = 0 as ::core::ffi::c_int;
        }
        cmdq_print(
            item,
            b"Job %u: %s [fd=%d, pid=%ld, status=%d]\0" as *const u8 as *const ::core::ffi::c_char,
            n,
            (*job).cmd,
            (*job).fd,
            (*job).pid as ::core::ffi::c_long,
            (*job).status,
        );
        n = n.wrapping_add(1);
        job = (*job).entry.le_next;
    }
}

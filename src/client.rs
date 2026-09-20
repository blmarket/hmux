pub use crate::src::shared::stdio::{
    FILE, _IO_FILE, _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data,
};
pub use crate::src::shared::abi::{
    __clock_t, __off64_t, __off_t, __socklen_t, __uid_t, __uint16_t, __uint32_t, socklen_t,
    ssize_t, uint16_t, uint32_t,
};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::command::{CMD_STARTSERVER};
pub use crate::src::shared::client::{
    CLIENT_CONTROL, CLIENT_CONTROLCONTROL, CLIENT_CONTROL_WAITEXIT, CLIENT_LOGIN,
    CLIENT_NOSTARTSERVER, CLIENT_STARTSERVER, CLIENT_WRITE_ACK,
};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::arguments::*;
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
use ::libc;
extern "C" {
    pub type sockaddr_x25;
    pub type sockaddr_ns;
    pub type sockaddr_iso;
    pub type sockaddr_ipx;
    pub type sockaddr_inarp;
    pub type sockaddr_eon;
    pub type sockaddr_dl;
    pub type sockaddr_ax25;
    pub type sockaddr_at;
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
    fn socket(
        __domain: ::core::ffi::c_int,
        __type: ::core::ffi::c_int,
        __protocol: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn connect(
        __fd: ::core::ffi::c_int,
        __addr: __CONST_SOCKADDR_ARG,
        __len: socklen_t,
    ) -> ::core::ffi::c_int;
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
    fn strerror(__errnum: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn strsignal(__sig: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn strlcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    fn kill(__pid: __pid_t, __sig: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn sigemptyset(__set: *mut sigset_t) -> ::core::ffi::c_int;
    fn sigaction(
        __sig: ::core::ffi::c_int,
        __act: *const sigaction,
        __oact: *mut sigaction,
    ) -> ::core::ffi::c_int;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn closefrom(__lowfd: ::core::ffi::c_int);
    fn dup(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    static mut environ: *mut *mut ::core::ffi::c_char;
    fn execl(
        __path: *const ::core::ffi::c_char,
        __arg: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn getpid() -> __pid_t;
    fn getppid() -> __pid_t;
    fn ttyname(__fd: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn isatty(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn unlink(__name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn waitpid(
        __pid: __pid_t,
        __stat_loc: *mut ::core::ffi::c_int,
        __options: ::core::ffi::c_int,
    ) -> __pid_t;
    fn open(
        __file: *const ::core::ffi::c_char,
        __oflag: ::core::ffi::c_int,
        ...
    ) -> ::core::ffi::c_int;
    fn flock(__fd: ::core::ffi::c_int, __operation: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn getenv(__name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn setenv(
        __name: *const ::core::ffi::c_char,
        __value: *const ::core::ffi::c_char,
        __replace: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn system(__command: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    static mut stdout: *mut FILE;
    static mut stderr: *mut FILE;
    fn fflush(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn cfgetospeed(__termios_p: *const termios) -> speed_t;
    fn cfgetispeed(__termios_p: *const termios) -> speed_t;
    fn cfsetospeed(__termios_p: *mut termios, __speed: speed_t) -> ::core::ffi::c_int;
    fn cfsetispeed(__termios_p: *mut termios, __speed: speed_t) -> ::core::ffi::c_int;
    fn tcgetattr(__fd: ::core::ffi::c_int, __termios_p: *mut termios) -> ::core::ffi::c_int;
    fn tcsetattr(
        __fd: ::core::ffi::c_int,
        __optional_actions: ::core::ffi::c_int,
        __termios_p: *const termios,
    ) -> ::core::ffi::c_int;
    fn cfmakeraw(__termios_p: *mut termios);
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn systemd_activated() -> ::core::ffi::c_int;
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
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
    static mut global_s_options: *mut options;
    static mut global_w_options: *mut options;
    static mut global_environ: *mut environ;
    static mut socket_path: *const ::core::ffi::c_char;
    static mut shell_command: *const ::core::ffi::c_char;
    static mut ptm_fd: ::core::ffi::c_int;
    fn setblocking(_: ::core::ffi::c_int, _: ::core::ffi::c_int);
    fn shell_argv0(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn find_cwd() -> *const ::core::ffi::c_char;
    fn find_home() -> *const ::core::ffi::c_char;
    fn proc_send(
        _: *mut tmuxpeer,
        _: msgtype,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_void,
        _: size_t,
    ) -> ::core::ffi::c_int;
    fn proc_start(_: *const ::core::ffi::c_char) -> *mut tmuxproc;
    fn proc_loop(_: *mut tmuxproc, _: Option<unsafe extern "C" fn() -> ::core::ffi::c_int>);
    fn proc_exit(_: *mut tmuxproc);
    fn proc_set_signals(
        _: *mut tmuxproc,
        _: Option<unsafe extern "C" fn(::core::ffi::c_int) -> ()>,
    );
    fn proc_clear_signals(_: *mut tmuxproc, _: ::core::ffi::c_int);
    fn proc_add_peer(
        _: *mut tmuxproc,
        _: ::core::ffi::c_int,
        _: Option<unsafe extern "C" fn(*mut imsg, *mut ::core::ffi::c_void) -> ()>,
        _: *mut ::core::ffi::c_void,
    ) -> *mut tmuxpeer;
    fn proc_flush_peer(_: *mut tmuxpeer);
    fn options_free(_: *mut options);
    fn environ_free(_: *mut environ);
    fn tty_term_read_list(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut *mut *mut ::core::ffi::c_char,
        _: *mut u_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn tty_term_free_list(_: *mut *mut ::core::ffi::c_char, _: u_int);
    fn args_from_vector(_: ::core::ffi::c_int, _: *mut *mut ::core::ffi::c_char)
        -> *mut args_value;
    fn args_free_values(_: *mut args_value, _: u_int);
    fn cmd_pack_argv(
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
        _: *mut ::core::ffi::c_char,
        _: size_t,
    ) -> ::core::ffi::c_int;
    fn cmd_list_free(_: *mut cmd_list);
    fn cmd_list_any_have(_: *mut cmd_list, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn cmd_parse_from_arguments(
        _: *mut args_value,
        _: u_int,
        _: *mut cmd_parse_input,
    ) -> *mut cmd_parse_result;
    fn file_write_left(_: *mut client_files) -> ::core::ffi::c_int;
    fn file_write_open(
        _: *mut client_files,
        _: *mut tmuxpeer,
        _: *mut imsg,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: client_file_cb,
        _: *mut ::core::ffi::c_void,
    );
    fn file_write_data(_: *mut client_files, _: *mut imsg);
    fn file_write_close(_: *mut client_files, _: *mut imsg);
    fn file_read_open(
        _: *mut client_files,
        _: *mut tmuxpeer,
        _: *mut imsg,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: client_file_cb,
        _: *mut ::core::ffi::c_void,
    );
    fn file_read_cancel(_: *mut client_files, _: *mut imsg);
    fn server_start(
        _: *mut tmuxproc,
        _: uint64_t,
        _: *mut event_base,
        _: ::core::ffi::c_int,
        _: *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn control_wait_exit(_: ::core::ffi::c_int);
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatal(_: *const ::core::ffi::c_char, ...) -> !;
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sigset_t {
    pub __val: [::core::ffi::c_ulong; 16],
}
pub type sigset_t = __sigset_t;
pub type __socket_type = ::core::ffi::c_uint;
pub const SOCK_NONBLOCK: __socket_type = 2048;
pub const SOCK_CLOEXEC: __socket_type = 524288;
pub const SOCK_PACKET: __socket_type = 10;
pub const SOCK_DCCP: __socket_type = 6;
pub const SOCK_SEQPACKET: __socket_type = 5;
pub const SOCK_RDM: __socket_type = 4;
pub const SOCK_RAW: __socket_type = 3;
pub const SOCK_DGRAM: __socket_type = 2;
pub const SOCK_STREAM: __socket_type = 1;
pub type sa_family_t = ::core::ffi::c_ushort;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr {
    pub sa_family: sa_family_t,
    pub sa_data: [::core::ffi::c_char; 14],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_un {
    pub sun_family: sa_family_t,
    pub sun_path: [::core::ffi::c_char; 108],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_in6 {
    pub sin6_family: sa_family_t,
    pub sin6_port: in_port_t,
    pub sin6_flowinfo: uint32_t,
    pub sin6_addr: in6_addr,
    pub sin6_scope_id: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct in6_addr {
    pub __in6_u: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub __u6_addr8: [uint8_t; 16],
    pub __u6_addr16: [uint16_t; 8],
    pub __u6_addr32: [uint32_t; 4],
}
pub type in_port_t = uint16_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_in {
    pub sin_family: sa_family_t,
    pub sin_port: in_port_t,
    pub sin_addr: in_addr,
    pub sin_zero: [::core::ffi::c_uchar; 8],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct in_addr {
    pub s_addr: in_addr_t,
}
pub type in_addr_t = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub union __CONST_SOCKADDR_ARG {
    pub __sockaddr__: *const sockaddr,
    pub __sockaddr_at__: *const sockaddr_at,
    pub __sockaddr_ax25__: *const sockaddr_ax25,
    pub __sockaddr_dl__: *const sockaddr_dl,
    pub __sockaddr_eon__: *const sockaddr_eon,
    pub __sockaddr_in__: *const sockaddr_in,
    pub __sockaddr_in6__: *const sockaddr_in6,
    pub __sockaddr_inarp__: *const sockaddr_inarp,
    pub __sockaddr_ipx__: *const sockaddr_ipx,
    pub __sockaddr_iso__: *const sockaddr_iso,
    pub __sockaddr_ns__: *const sockaddr_ns,
    pub __sockaddr_un__: *const sockaddr_un,
    pub __sockaddr_x25__: *const sockaddr_x25,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union sigval {
    pub sival_int: ::core::ffi::c_int,
    pub sival_ptr: *mut ::core::ffi::c_void,
}
pub type __sigval_t = sigval;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct siginfo_t {
    pub si_signo: ::core::ffi::c_int,
    pub si_errno: ::core::ffi::c_int,
    pub si_code: ::core::ffi::c_int,
    pub __pad0: ::core::ffi::c_int,
    pub _sifields: C2RustUnnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_0 {
    pub _pad: [::core::ffi::c_int; 28],
    pub _kill: C2RustUnnamed_9,
    pub _timer: C2RustUnnamed_8,
    pub _rt: C2RustUnnamed_7,
    pub _sigchld: C2RustUnnamed_6,
    pub _sigfault: C2RustUnnamed_3,
    pub _sigpoll: C2RustUnnamed_2,
    pub _sigsys: C2RustUnnamed_1,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_1 {
    pub _call_addr: *mut ::core::ffi::c_void,
    pub _syscall: ::core::ffi::c_int,
    pub _arch: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_2 {
    pub si_band: ::core::ffi::c_long,
    pub si_fd: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_3 {
    pub si_addr: *mut ::core::ffi::c_void,
    pub si_addr_lsb: ::core::ffi::c_short,
    pub _bounds: C2RustUnnamed_4,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_4 {
    pub _addr_bnd: C2RustUnnamed_5,
    pub _pkey: __uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_5 {
    pub _lower: *mut ::core::ffi::c_void,
    pub _upper: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_6 {
    pub si_pid: __pid_t,
    pub si_uid: __uid_t,
    pub si_status: ::core::ffi::c_int,
    pub si_utime: __clock_t,
    pub si_stime: __clock_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_7 {
    pub si_pid: __pid_t,
    pub si_uid: __uid_t,
    pub si_sigval: __sigval_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_8 {
    pub si_tid: ::core::ffi::c_int,
    pub si_overrun: ::core::ffi::c_int,
    pub si_sigval: __sigval_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_9 {
    pub si_pid: __pid_t,
    pub si_uid: __uid_t,
}
pub type __sighandler_t = Option<unsafe extern "C" fn(::core::ffi::c_int) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sigaction {
    pub __sigaction_handler: C2RustUnnamed_10,
    pub sa_mask: __sigset_t,
    pub sa_flags: ::core::ffi::c_int,
    pub sa_restorer: Option<unsafe extern "C" fn() -> ()>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_10 {
    pub sa_handler: __sighandler_t,
    pub sa_sigaction: Option<
        unsafe extern "C" fn(::core::ffi::c_int, *mut siginfo_t, *mut ::core::ffi::c_void) -> (),
    >,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ibuf {
    pub entry: C2RustUnnamed_22,
    pub buf: *mut ::core::ffi::c_uchar,
    pub size: size_t,
    pub max: size_t,
    pub wpos: size_t,
    pub rpos: size_t,
    pub fd: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_22 {
    pub tqe_next: *mut ibuf,
    pub tqe_prev: *mut *mut ibuf,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct imsg_hdr {
    pub type_0: uint32_t,
    pub len: uint32_t,
    pub peerid: uint32_t,
    pub pid: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct imsg {
    pub hdr: imsg_hdr,
    pub data: *mut ::core::ffi::c_void,
    pub buf: *mut ibuf,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_command {
    pub argc: ::core::ffi::c_int,
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
    pub entry: C2RustUnnamed_23,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_23 {
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
    pub entry: C2RustUnnamed_24,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_24 {
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
pub union C2RustUnnamed_25 {
    pub offset: u_int,
    pub data: C2RustUnnamed_26,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_26 {
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
    pub gentry: C2RustUnnamed_28,
    pub entry: C2RustUnnamed_27,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_27 {
    pub rbe_left: *mut session,
    pub rbe_right: *mut session,
    pub rbe_parent: *mut session,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_28 {
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
    pub entry: C2RustUnnamed_31,
    pub wentry: C2RustUnnamed_30,
    pub sentry: C2RustUnnamed_29,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_29 {
    pub tqe_next: *mut winlink,
    pub tqe_prev: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_30 {
    pub tqe_next: *mut winlink,
    pub tqe_prev: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_31 {
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
    pub alerts_entry: C2RustUnnamed_34,
    pub options: *mut options,
    pub references: u_int,
    pub winlinks: C2RustUnnamed_33,
    pub entry: C2RustUnnamed_32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_32 {
    pub rbe_left: *mut window,
    pub rbe_right: *mut window,
    pub rbe_parent: *mut window,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_33 {
    pub tqh_first: *mut winlink,
    pub tqh_last: *mut *mut winlink,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_34 {
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
    pub entry: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
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
    pub modes: C2RustUnnamed_40,
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
    pub entry: C2RustUnnamed_39,
    pub sentry: C2RustUnnamed_38,
    pub zentry: C2RustUnnamed_37,
    pub tree_entry: C2RustUnnamed_36,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_36 {
    pub rbe_left: *mut window_pane,
    pub rbe_right: *mut window_pane,
    pub rbe_parent: *mut window_pane,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_38 {
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_39 {
    pub tqe_next: *mut window_pane,
    pub tqe_prev: *mut *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_40 {
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
    pub entry: C2RustUnnamed_41,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_41 {
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
    pub entry: C2RustUnnamed_44,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_44 {
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
    pub entry: C2RustUnnamed_45,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_45 {
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
    pub entry: C2RustUnnamed_47,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_47 {
    pub le_next: *mut tty_term,
    pub le_prev: *mut *mut tty_term,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_value {
    pub type_0: args_type,
    pub c2rust_unnamed: C2RustUnnamed_49,
    pub cached: *mut ::core::ffi::c_char,
    pub entry: C2RustUnnamed_48,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_48 {
    pub tqe_next: *mut args_value,
    pub tqe_prev: *mut *mut args_value,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_49 {
    pub string: *mut ::core::ffi::c_char,
    pub cmdlist: *mut cmd_list,
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
pub const SIG_DFL: __sighandler_t = None;
pub const SIGTERM: ::core::ffi::c_int = 15;
pub const SIGHUP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PF_LOCAL: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PF_UNIX: ::core::ffi::c_int = PF_LOCAL;
pub const AF_UNIX: ::core::ffi::c_int = PF_UNIX;
pub const WNOHANG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SIGTSTP: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const SIGCONT: ::core::ffi::c_int = 18;
pub const SIGCHLD: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const SIGWINCH: ::core::ffi::c_int = 28;
pub const SA_RESTART: ::core::ffi::c_int = 0x10000000 as ::core::ffi::c_int;
pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ENAMETOOLONG: ::core::ffi::c_int = 36 as ::core::ffi::c_int;
pub const ECONNREFUSED: ::core::ffi::c_int = 111 as ::core::ffi::c_int;
pub const WAIT_ANY: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const O_WRONLY: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const O_CREAT: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const LOCK_EX: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LOCK_NB: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const ENOENT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const EINTR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const ECHILD: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const EAGAIN: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const VTIME: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const VMIN: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const ICRNL: ::core::ffi::c_int = 0o400 as ::core::ffi::c_int;
pub const IXANY: ::core::ffi::c_int = 0o4000 as ::core::ffi::c_int;
pub const OPOST: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const ONLCR: ::core::ffi::c_int = 0o4 as ::core::ffi::c_int;
pub const CS8: ::core::ffi::c_int = 0o60 as ::core::ffi::c_int;
pub const CREAD: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;
pub const HUPCL: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;
pub const TCSANOW: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const TCSAFLUSH: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const IMSG_HEADER_SIZE: usize = ::core::mem::size_of::<imsg_hdr>();
pub const MAX_IMSGSIZE: ::core::ffi::c_int = 16384 as ::core::ffi::c_int;
pub const PROTOCOL_VERSION: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
static mut client_proc: *mut tmuxproc = ::core::ptr::null::<tmuxproc>() as *mut tmuxproc;
static mut client_peer: *mut tmuxpeer = ::core::ptr::null::<tmuxpeer>() as *mut tmuxpeer;
static mut client_flags: uint64_t = 0;
static mut client_suspended: ::core::ffi::c_int = 0;
static mut client_exitreason: client_exit_reason = CLIENT_EXIT_NONE;
static mut client_exitflag: ::core::ffi::c_int = 0;
static mut client_exitval: ::core::ffi::c_int = 0;
static mut client_exittype: msgtype = 0 as msgtype;
static mut client_exitsession: *const ::core::ffi::c_char =
    ::core::ptr::null::<::core::ffi::c_char>();
static mut client_exitmessage: *mut ::core::ffi::c_char =
    ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char;
static mut client_execshell: *const ::core::ffi::c_char =
    ::core::ptr::null::<::core::ffi::c_char>();
static mut client_execcmd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
static mut client_attached: ::core::ffi::c_int = 0;
static mut client_files: client_files = client_files {
    rbh_root: ::core::ptr::null::<client_file>() as *mut client_file,
};
unsafe extern "C" fn client_get_lock(mut lockfile: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut lockfd: ::core::ffi::c_int = 0;
    log_debug(
        b"lock file is %s\0" as *const u8 as *const ::core::ffi::c_char,
        lockfile,
    );
    lockfd = open(lockfile, O_WRONLY | O_CREAT, 0o600 as ::core::ffi::c_int);
    if lockfd == -(1 as ::core::ffi::c_int) {
        log_debug(
            b"open failed: %s\0" as *const u8 as *const ::core::ffi::c_char,
            strerror(*__errno_location()),
        );
        return -(1 as ::core::ffi::c_int);
    }
    if flock(lockfd, LOCK_EX | LOCK_NB) == -(1 as ::core::ffi::c_int) {
        log_debug(
            b"flock failed: %s\0" as *const u8 as *const ::core::ffi::c_char,
            strerror(*__errno_location()),
        );
        if *__errno_location() != EAGAIN {
            return lockfd;
        }
        while flock(lockfd, LOCK_EX) == -(1 as ::core::ffi::c_int) && *__errno_location() == EINTR {
        }
        close(lockfd);
        return -(2 as ::core::ffi::c_int);
    }
    log_debug(b"flock succeeded\0" as *const u8 as *const ::core::ffi::c_char);
    return lockfd;
}
unsafe extern "C" fn client_connect(
    mut base: *mut event_base,
    mut path: *const ::core::ffi::c_char,
    mut flags: uint64_t,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut sa: sockaddr_un = sockaddr_un {
        sun_family: 0,
        sun_path: [0; 108],
    };
    let mut size: size_t = 0;
    let mut fd: ::core::ffi::c_int = 0;
    let mut lockfd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let mut locked: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut lockfile: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    memset(
        &raw mut sa as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<sockaddr_un>() as size_t,
    );
    sa.sun_family = AF_UNIX as sa_family_t;
    size = strlcpy(
        &raw mut sa.sun_path as *mut ::core::ffi::c_char,
        path,
        ::core::mem::size_of::<[::core::ffi::c_char; 108]>() as size_t,
    ) as size_t;
    if size >= ::core::mem::size_of::<[::core::ffi::c_char; 108]>() as usize {
        *__errno_location() = ENAMETOOLONG;
        return -(1 as ::core::ffi::c_int);
    }
    log_debug(
        b"socket is %s\0" as *const u8 as *const ::core::ffi::c_char,
        path,
    );
    loop {
        fd = socket(
            AF_UNIX,
            SOCK_STREAM as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        if fd == -(1 as ::core::ffi::c_int) {
            return -(1 as ::core::ffi::c_int);
        }
        log_debug(b"trying connect\0" as *const u8 as *const ::core::ffi::c_char);
        if !(connect(
            fd,
            __CONST_SOCKADDR_ARG {
                __sockaddr__: &raw mut sa as *mut sockaddr,
            },
            ::core::mem::size_of::<sockaddr_un>() as socklen_t,
        ) == -(1 as ::core::ffi::c_int))
        {
            current_block = 7172762164747879670;
            break;
        }
        log_debug(
            b"connect failed: %s\0" as *const u8 as *const ::core::ffi::c_char,
            strerror(*__errno_location()),
        );
        if *__errno_location() != ECONNREFUSED && *__errno_location() != ENOENT {
            current_block = 16524389688364091157;
            break;
        }
        if flags & CLIENT_NOSTARTSERVER as uint64_t != 0 {
            current_block = 16524389688364091157;
            break;
        }
        if !flags & CLIENT_STARTSERVER as uint64_t != 0 {
            current_block = 16524389688364091157;
            break;
        }
        close(fd);
        if locked == 0 {
            xasprintf(
                &raw mut lockfile,
                b"%s.lock\0" as *const u8 as *const ::core::ffi::c_char,
                path,
            );
            lockfd = client_get_lock(lockfile);
            if lockfd < 0 as ::core::ffi::c_int {
                log_debug(
                    b"didn't get lock (%d)\0" as *const u8 as *const ::core::ffi::c_char,
                    lockfd,
                );
                free(lockfile as *mut ::core::ffi::c_void);
                lockfile = ::core::ptr::null_mut::<::core::ffi::c_char>();
                if lockfd == -(2 as ::core::ffi::c_int) {
                    continue;
                }
            }
            log_debug(
                b"got lock (%d)\0" as *const u8 as *const ::core::ffi::c_char,
                lockfd,
            );
            locked = 1 as ::core::ffi::c_int;
        } else {
            if lockfd >= 0 as ::core::ffi::c_int
                && unlink(path) != 0 as ::core::ffi::c_int
                && *__errno_location() != ENOENT
            {
                free(lockfile as *mut ::core::ffi::c_void);
                close(lockfd);
                return -(1 as ::core::ffi::c_int);
            }
            fd = server_start(client_proc, flags, base, lockfd, lockfile);
            current_block = 7172762164747879670;
            break;
        }
    }
    match current_block {
        16524389688364091157 => {
            if locked != 0 {
                free(lockfile as *mut ::core::ffi::c_void);
                close(lockfd);
            }
            close(fd);
            return -(1 as ::core::ffi::c_int);
        }
        _ => {
            if locked != 0 && lockfd >= 0 as ::core::ffi::c_int {
                free(lockfile as *mut ::core::ffi::c_void);
                close(lockfd);
            }
            setblocking(fd, 0 as ::core::ffi::c_int);
            return fd;
        }
    };
}
unsafe extern "C" fn client_exit_message() -> *const ::core::ffi::c_char {
    static mut msg: [::core::ffi::c_char; 256] = [0; 256];
    match client_exitreason as ::core::ffi::c_uint {
        1 => {
            if !client_exitsession.is_null() {
                xsnprintf(
                    &raw mut msg as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
                    b"detached (from session %s)\0" as *const u8 as *const ::core::ffi::c_char,
                    client_exitsession,
                );
                return &raw mut msg as *mut ::core::ffi::c_char;
            }
            return b"detached\0" as *const u8 as *const ::core::ffi::c_char;
        }
        2 => {
            if !client_exitsession.is_null() {
                xsnprintf(
                    &raw mut msg as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
                    b"detached and SIGHUP (from session %s)\0" as *const u8
                        as *const ::core::ffi::c_char,
                    client_exitsession,
                );
                return &raw mut msg as *mut ::core::ffi::c_char;
            }
            return b"detached and SIGHUP\0" as *const u8 as *const ::core::ffi::c_char;
        }
        3 => return b"lost tty\0" as *const u8 as *const ::core::ffi::c_char,
        4 => return b"terminated\0" as *const u8 as *const ::core::ffi::c_char,
        5 => {
            return b"server exited unexpectedly\0" as *const u8 as *const ::core::ffi::c_char;
        }
        6 => return b"exited\0" as *const u8 as *const ::core::ffi::c_char,
        7 => return b"server exited\0" as *const u8 as *const ::core::ffi::c_char,
        8 => return client_exitmessage,
        0 | _ => {}
    }
    return b"unknown reason\0" as *const u8 as *const ::core::ffi::c_char;
}
unsafe extern "C" fn client_exit() {
    if file_write_left(&raw mut client_files) == 0 {
        proc_exit(client_proc);
    }
}
#[no_mangle]
pub unsafe extern "C" fn client_main(
    mut base: *mut event_base,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
    mut flags: uint64_t,
    mut feat: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut data: *mut msg_command = ::core::ptr::null_mut::<msg_command>();
    let mut fd: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut ttynam: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut termname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cwd: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ppid: pid_t = 0;
    let mut msg: msgtype = 0 as msgtype;
    let mut tio: termios = termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_line: 0,
        c_cc: [0; 32],
        c2rust_unnamed: termios_input_speed { __ispeed: 0 },
        c2rust_unnamed_0: termios_output_speed { __ospeed: 0 },
    };
    let mut saved_tio: termios = termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_line: 0,
        c_cc: [0; 32],
        c2rust_unnamed: termios_input_speed { __ispeed: 0 },
        c2rust_unnamed_0: termios_output_speed { __ospeed: 0 },
    };
    let mut size: size_t = 0;
    let mut caps: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ncaps: u_int = 0 as u_int;
    let mut values: *mut args_value = ::core::ptr::null_mut::<args_value>();
    if !shell_command.is_null() {
        msg = MSG_SHELL;
        flags |= CLIENT_STARTSERVER as uint64_t;
    } else if argc == 0 as ::core::ffi::c_int {
        msg = MSG_COMMAND;
        flags |= CLIENT_STARTSERVER as uint64_t;
    } else {
        msg = MSG_COMMAND;
        values = args_from_vector(argc, argv);
        pr = cmd_parse_from_arguments(
            values,
            argc as u_int,
            ::core::ptr::null_mut::<cmd_parse_input>(),
        );
        if (*pr).status as ::core::ffi::c_uint
            == CMD_PARSE_SUCCESS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if cmd_list_any_have((*pr).cmdlist, CMD_STARTSERVER) != 0 {
                flags |= CLIENT_STARTSERVER as uint64_t;
            }
            cmd_list_free((*pr).cmdlist);
        } else {
            free((*pr).error as *mut ::core::ffi::c_void);
        }
        args_free_values(values, argc as u_int);
        free(values as *mut ::core::ffi::c_void);
    }
    client_proc = proc_start(b"client\0" as *const u8 as *const ::core::ffi::c_char);
    proc_set_signals(
        client_proc,
        Some(client_signal as unsafe extern "C" fn(::core::ffi::c_int) -> ()),
    );
    client_flags = (flags as ::core::ffi::c_ulonglong | CLIENT_WRITE_ACK) as uint64_t;
    log_debug(
        b"flags are %#llx\0" as *const u8 as *const ::core::ffi::c_char,
        client_flags as ::core::ffi::c_ulonglong,
    );
    if systemd_activated() != 0 {
        fd = server_start(
            client_proc,
            flags,
            base,
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        );
    } else {
        fd = client_connect(base, socket_path, client_flags);
    }
    if fd == -(1 as ::core::ffi::c_int) {
        if *__errno_location() == ECONNREFUSED {
            fprintf(
                stderr,
                b"no server running on %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                socket_path,
            );
        } else {
            fprintf(
                stderr,
                b"error connecting to %s (%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
                socket_path,
                strerror(*__errno_location()),
            );
        }
        return 1 as ::core::ffi::c_int;
    }
    client_peer = proc_add_peer(
        client_proc,
        fd,
        Some(client_dispatch as unsafe extern "C" fn(*mut imsg, *mut ::core::ffi::c_void) -> ()),
        NULL,
    );
    cwd = find_cwd();
    if cwd.is_null() && {
        cwd = find_home();
        cwd.is_null()
    } {
        cwd = b"/\0" as *const u8 as *const ::core::ffi::c_char;
    }
    ttynam = ttyname(STDIN_FILENO);
    if ttynam.is_null() {
        ttynam = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    termname = getenv(b"TERM\0" as *const u8 as *const ::core::ffi::c_char);
    if termname.is_null() {
        termname = b"\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if 0 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        fatal(b"pledge failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if isatty(STDIN_FILENO) != 0
        && *termname as ::core::ffi::c_int != '\0' as i32
        && tty_term_read_list(
            termname,
            STDIN_FILENO,
            &raw mut caps,
            &raw mut ncaps,
            &raw mut cause,
        ) != 0 as ::core::ffi::c_int
    {
        fprintf(
            stderr,
            b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            cause,
        );
        free(cause as *mut ::core::ffi::c_void);
        return 1 as ::core::ffi::c_int;
    }
    if ptm_fd != -(1 as ::core::ffi::c_int) {
        close(ptm_fd);
    }
    options_free(global_options);
    options_free(global_s_options);
    options_free(global_w_options);
    environ_free(global_environ);
    if client_flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
        if tcgetattr(STDIN_FILENO, &raw mut saved_tio) != 0 as ::core::ffi::c_int {
            fprintf(
                stderr,
                b"tcgetattr failed: %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                strerror(*__errno_location()),
            );
            return 1 as ::core::ffi::c_int;
        }
        cfmakeraw(&raw mut tio);
        tio.c_iflag = (ICRNL | IXANY) as tcflag_t;
        tio.c_oflag = (OPOST | ONLCR) as tcflag_t;
        tio.c_cflag = (CREAD | CS8 | HUPCL) as tcflag_t;
        tio.c_cc[VMIN as usize] = 1 as cc_t;
        tio.c_cc[VTIME as usize] = 0 as cc_t;
        cfsetispeed(&raw mut tio, cfgetispeed(&raw mut saved_tio));
        cfsetospeed(&raw mut tio, cfgetospeed(&raw mut saved_tio));
        tcsetattr(STDIN_FILENO, TCSANOW, &raw mut tio);
    }
    client_send_identify(ttynam, termname, caps, ncaps, cwd, feat);
    tty_term_free_list(caps, ncaps);
    proc_flush_peer(client_peer);
    if msg as ::core::ffi::c_uint == MSG_COMMAND as ::core::ffi::c_int as ::core::ffi::c_uint {
        size = 0 as size_t;
        i = 0 as ::core::ffi::c_int;
        while i < argc {
            size = size.wrapping_add(strlen(*argv.offset(i as isize)).wrapping_add(1 as size_t));
            i += 1;
        }
        if size
            > (MAX_IMSGSIZE as usize).wrapping_sub(::core::mem::size_of::<msg_command>() as usize)
        {
            fprintf(
                stderr,
                b"command too long\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
            return 1 as ::core::ffi::c_int;
        }
        data = xmalloc((::core::mem::size_of::<msg_command>() as size_t).wrapping_add(size))
            as *mut msg_command;
        (*data).argc = argc;
        if cmd_pack_argv(
            argc,
            argv,
            data.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char,
            size,
        ) != 0 as ::core::ffi::c_int
        {
            fprintf(
                stderr,
                b"command too long\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
            free(data as *mut ::core::ffi::c_void);
            return 1 as ::core::ffi::c_int;
        }
        size = (size as ::core::ffi::c_ulong)
            .wrapping_add(::core::mem::size_of::<msg_command>() as usize as ::core::ffi::c_ulong)
            as size_t as size_t;
        if proc_send(
            client_peer,
            msg,
            -(1 as ::core::ffi::c_int),
            data as *const ::core::ffi::c_void,
            size,
        ) != 0 as ::core::ffi::c_int
        {
            fprintf(
                stderr,
                b"failed to send command\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
            free(data as *mut ::core::ffi::c_void);
            return 1 as ::core::ffi::c_int;
        }
        free(data as *mut ::core::ffi::c_void);
    } else if msg as ::core::ffi::c_uint == MSG_SHELL as ::core::ffi::c_int as ::core::ffi::c_uint {
        proc_send(
            client_peer,
            msg,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null::<::core::ffi::c_void>(),
            0 as size_t,
        );
    }
    proc_loop(client_proc, None);
    if client_exittype as ::core::ffi::c_uint
        == MSG_EXEC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if client_flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
            tcsetattr(STDOUT_FILENO, TCSAFLUSH, &raw mut saved_tio);
        }
        client_exec(client_execshell, client_execcmd);
    }
    if client_attached != 0 {
        if client_exitreason as ::core::ffi::c_uint
            != CLIENT_EXIT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            printf(
                b"[%s]\n\0" as *const u8 as *const ::core::ffi::c_char,
                client_exit_message(),
            );
        }
        ppid = getppid() as pid_t;
        if client_exittype as ::core::ffi::c_uint
            == MSG_DETACHKILL as ::core::ffi::c_int as ::core::ffi::c_uint
            && ppid > 1 as ::core::ffi::c_int
        {
            kill(ppid as __pid_t, SIGHUP);
        }
    } else if client_flags & CLIENT_CONTROL as uint64_t != 0 {
        if client_exitreason as ::core::ffi::c_uint
            != CLIENT_EXIT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            printf(
                b"%%exit %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                client_exit_message(),
            );
        } else {
            printf(b"%%exit\n\0" as *const u8 as *const ::core::ffi::c_char);
        }
        fflush(stdout);
        if client_flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_WAITEXIT != 0 {
            control_wait_exit(STDIN_FILENO);
        }
        if client_flags & CLIENT_CONTROLCONTROL as uint64_t != 0 {
            printf(b"\x1B\\\0" as *const u8 as *const ::core::ffi::c_char);
            fflush(stdout);
            tcsetattr(STDOUT_FILENO, TCSAFLUSH, &raw mut saved_tio);
        }
    } else if client_exitreason as ::core::ffi::c_uint
        != CLIENT_EXIT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        fprintf(
            stderr,
            b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            client_exit_message(),
        );
    }
    setblocking(STDIN_FILENO, 1 as ::core::ffi::c_int);
    setblocking(STDOUT_FILENO, 1 as ::core::ffi::c_int);
    setblocking(STDERR_FILENO, 1 as ::core::ffi::c_int);
    return client_exitval;
}
unsafe extern "C" fn client_send_identify(
    mut ttynam: *const ::core::ffi::c_char,
    mut termname: *const ::core::ffi::c_char,
    mut caps: *mut *mut ::core::ffi::c_char,
    mut ncaps: u_int,
    mut cwd: *const ::core::ffi::c_char,
    mut feat: ::core::ffi::c_int,
) {
    let mut ss: *mut *mut ::core::ffi::c_char = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut sslen: size_t = 0;
    let mut fd: ::core::ffi::c_int = 0;
    let mut flags: uint64_t = client_flags;
    let mut pid: pid_t = 0;
    let mut i: u_int = 0;
    proc_send(
        client_peer,
        MSG_IDENTIFY_LONGFLAGS,
        -(1 as ::core::ffi::c_int),
        &raw mut flags as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint64_t>() as size_t,
    );
    proc_send(
        client_peer,
        MSG_IDENTIFY_LONGFLAGS,
        -(1 as ::core::ffi::c_int),
        &raw mut client_flags as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint64_t>() as size_t,
    );
    proc_send(
        client_peer,
        MSG_IDENTIFY_TERM,
        -(1 as ::core::ffi::c_int),
        termname as *const ::core::ffi::c_void,
        strlen(termname).wrapping_add(1 as size_t),
    );
    proc_send(
        client_peer,
        MSG_IDENTIFY_FEATURES,
        -(1 as ::core::ffi::c_int),
        &raw mut feat as *const ::core::ffi::c_void,
        ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
    );
    proc_send(
        client_peer,
        MSG_IDENTIFY_TTYNAME,
        -(1 as ::core::ffi::c_int),
        ttynam as *const ::core::ffi::c_void,
        strlen(ttynam).wrapping_add(1 as size_t),
    );
    proc_send(
        client_peer,
        MSG_IDENTIFY_CWD,
        -(1 as ::core::ffi::c_int),
        cwd as *const ::core::ffi::c_void,
        strlen(cwd).wrapping_add(1 as size_t),
    );
    i = 0 as u_int;
    while i < ncaps {
        proc_send(
            client_peer,
            MSG_IDENTIFY_TERMINFO,
            -(1 as ::core::ffi::c_int),
            *caps.offset(i as isize) as *const ::core::ffi::c_void,
            strlen(*caps.offset(i as isize)).wrapping_add(1 as size_t),
        );
        i = i.wrapping_add(1);
    }
    fd = dup(STDIN_FILENO);
    if fd == -(1 as ::core::ffi::c_int) {
        fatal(b"dup failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    proc_send(
        client_peer,
        MSG_IDENTIFY_STDIN,
        fd,
        ::core::ptr::null::<::core::ffi::c_void>(),
        0 as size_t,
    );
    fd = dup(STDOUT_FILENO);
    if fd == -(1 as ::core::ffi::c_int) {
        fatal(b"dup failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    proc_send(
        client_peer,
        MSG_IDENTIFY_STDOUT,
        fd,
        ::core::ptr::null::<::core::ffi::c_void>(),
        0 as size_t,
    );
    pid = getpid() as pid_t;
    proc_send(
        client_peer,
        MSG_IDENTIFY_CLIENTPID,
        -(1 as ::core::ffi::c_int),
        &raw mut pid as *const ::core::ffi::c_void,
        ::core::mem::size_of::<pid_t>() as size_t,
    );
    ss = environ;
    while !(*ss).is_null() {
        sslen = strlen(*ss).wrapping_add(1 as size_t);
        if !(sslen > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE)) {
            proc_send(
                client_peer,
                MSG_IDENTIFY_ENVIRON,
                -(1 as ::core::ffi::c_int),
                *ss as *const ::core::ffi::c_void,
                sslen,
            );
        }
        ss = ss.offset(1);
    }
    proc_send(
        client_peer,
        MSG_IDENTIFY_DONE,
        -(1 as ::core::ffi::c_int),
        ::core::ptr::null::<::core::ffi::c_void>(),
        0 as size_t,
    );
}
unsafe extern "C" fn client_exec(
    mut shell: *const ::core::ffi::c_char,
    mut shellcmd: *const ::core::ffi::c_char,
) -> ! {
    let mut argv0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    log_debug(
        b"shell %s, command %s\0" as *const u8 as *const ::core::ffi::c_char,
        shell,
        shellcmd,
    );
    argv0 = shell_argv0(
        shell,
        (client_flags & CLIENT_LOGIN as uint64_t != 0) as ::core::ffi::c_int,
    );
    setenv(
        b"SHELL\0" as *const u8 as *const ::core::ffi::c_char,
        shell,
        1 as ::core::ffi::c_int,
    );
    proc_clear_signals(client_proc, 1 as ::core::ffi::c_int);
    setblocking(STDIN_FILENO, 1 as ::core::ffi::c_int);
    setblocking(STDOUT_FILENO, 1 as ::core::ffi::c_int);
    setblocking(STDERR_FILENO, 1 as ::core::ffi::c_int);
    closefrom(STDERR_FILENO + 1 as ::core::ffi::c_int);
    execl(
        shell,
        argv0,
        b"-c\0" as *const u8 as *const ::core::ffi::c_char,
        shellcmd,
        NULL as *mut ::core::ffi::c_char,
    );
    fatal(b"execl failed\0" as *const u8 as *const ::core::ffi::c_char);
}
unsafe extern "C" fn client_signal(mut sig: ::core::ffi::c_int) {
    let mut sigact: sigaction = sigaction {
        __sigaction_handler: C2RustUnnamed_10 { sa_handler: None },
        sa_mask: __sigset_t { __val: [0; 16] },
        sa_flags: 0,
        sa_restorer: None,
    };
    let mut status: ::core::ffi::c_int = 0;
    let mut pid: pid_t = 0;
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"client_signal\0" as *const u8 as *const ::core::ffi::c_char,
        strsignal(sig),
    );
    if sig == SIGCHLD {
        loop {
            pid = waitpid(WAIT_ANY, &raw mut status, WNOHANG) as pid_t;
            if pid == 0 as ::core::ffi::c_int {
                break;
            }
            if !(pid == -(1 as ::core::ffi::c_int)) {
                continue;
            }
            if *__errno_location() == ECHILD {
                break;
            }
            log_debug(
                b"waitpid failed: %s\0" as *const u8 as *const ::core::ffi::c_char,
                strerror(*__errno_location()),
            );
        }
    } else if client_attached == 0 {
        if sig == SIGTERM || sig == SIGHUP {
            proc_exit(client_proc);
        }
    } else {
        match sig {
            SIGHUP => {
                client_exitreason = CLIENT_EXIT_LOST_TTY;
                client_exitval = 1 as ::core::ffi::c_int;
                proc_send(
                    client_peer,
                    MSG_EXITING,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null::<::core::ffi::c_void>(),
                    0 as size_t,
                );
            }
            SIGTERM => {
                if client_suspended == 0 {
                    client_exitreason = CLIENT_EXIT_TERMINATED;
                }
                client_exitval = 1 as ::core::ffi::c_int;
                proc_send(
                    client_peer,
                    MSG_EXITING,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null::<::core::ffi::c_void>(),
                    0 as size_t,
                );
            }
            SIGWINCH => {
                proc_send(
                    client_peer,
                    MSG_RESIZE,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null::<::core::ffi::c_void>(),
                    0 as size_t,
                );
            }
            SIGCONT => {
                memset(
                    &raw mut sigact as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    ::core::mem::size_of::<sigaction>() as size_t,
                );
                sigemptyset(&raw mut sigact.sa_mask);
                sigact.sa_flags = SA_RESTART;
                sigact.__sigaction_handler.sa_handler =
                    ::core::mem::transmute::<::libc::intptr_t, __sighandler_t>(
                        1 as ::core::ffi::c_int as ::libc::intptr_t,
                    );
                if sigaction(
                    SIGTSTP,
                    &raw mut sigact,
                    ::core::ptr::null_mut::<sigaction>(),
                ) != 0 as ::core::ffi::c_int
                {
                    fatal(b"sigaction failed\0" as *const u8 as *const ::core::ffi::c_char);
                }
                proc_send(
                    client_peer,
                    MSG_WAKEUP,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null::<::core::ffi::c_void>(),
                    0 as size_t,
                );
                client_suspended = 0 as ::core::ffi::c_int;
            }
            _ => {}
        }
    };
}
unsafe extern "C" fn client_file_check_cb(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut error: ::core::ffi::c_int,
    mut closed: ::core::ffi::c_int,
    mut buffer: *mut evbuffer,
    mut data: *mut ::core::ffi::c_void,
) {
    if client_exitflag != 0 {
        client_exit();
    }
}
unsafe extern "C" fn client_dispatch(mut imsg: *mut imsg, mut arg: *mut ::core::ffi::c_void) {
    if imsg.is_null() {
        if client_exitflag == 0 {
            client_exitreason = CLIENT_EXIT_LOST_SERVER;
            client_exitval = 1 as ::core::ffi::c_int;
        }
        proc_exit(client_proc);
        return;
    }
    if client_attached != 0 {
        client_dispatch_attached(imsg);
    } else {
        client_dispatch_wait(imsg);
    };
}
unsafe extern "C" fn client_dispatch_exit_message(
    mut data: *mut ::core::ffi::c_char,
    mut datalen: size_t,
) {
    let mut retval: ::core::ffi::c_int = 0;
    if datalen < ::core::mem::size_of::<::core::ffi::c_int>() as usize && datalen != 0 as size_t {
        fatalx(b"bad MSG_EXIT size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if datalen >= ::core::mem::size_of::<::core::ffi::c_int>() as usize {
        memcpy(
            &raw mut retval as *mut ::core::ffi::c_void,
            data as *const ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
        );
        client_exitval = retval;
    }
    if datalen > ::core::mem::size_of::<::core::ffi::c_int>() as usize {
        datalen = (datalen as ::core::ffi::c_ulong)
            .wrapping_sub(
                ::core::mem::size_of::<::core::ffi::c_int>() as usize as ::core::ffi::c_ulong
            ) as size_t as size_t;
        data = data.offset(::core::mem::size_of::<::core::ffi::c_int>() as usize as isize);
        client_exitmessage = xmalloc(datalen) as *mut ::core::ffi::c_char;
        memcpy(
            client_exitmessage as *mut ::core::ffi::c_void,
            data as *const ::core::ffi::c_void,
            datalen,
        );
        *client_exitmessage.offset(datalen.wrapping_sub(1 as size_t) as isize) =
            '\0' as i32 as ::core::ffi::c_char;
        client_exitreason = CLIENT_EXIT_MESSAGE_PROVIDED;
    }
}
unsafe extern "C" fn client_dispatch_wait(mut imsg: *mut imsg) {
    let mut data: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut datalen: ssize_t = 0;
    static mut pledge_applied: ::core::ffi::c_int = 0;
    if pledge_applied == 0 {
        if 0 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            fatal(b"pledge failed\0" as *const u8 as *const ::core::ffi::c_char);
        }
        pledge_applied = 1 as ::core::ffi::c_int;
    }
    data = (*imsg).data as *mut ::core::ffi::c_char;
    datalen = ((*imsg).hdr.len as usize).wrapping_sub(IMSG_HEADER_SIZE) as ssize_t;
    match (*imsg).hdr.type_0 {
        203 | 210 => {
            client_dispatch_exit_message(data, datalen as size_t);
            client_exitflag = 1 as ::core::ffi::c_int;
            client_exit();
        }
        207 => {
            if datalen != 0 as ssize_t {
                fatalx(b"bad MSG_READY size\0" as *const u8 as *const ::core::ffi::c_char);
            }
            client_attached = 1 as ::core::ffi::c_int;
            proc_send(
                client_peer,
                MSG_RESIZE,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
        }
        12 => {
            if datalen != 0 as ssize_t {
                fatalx(b"bad MSG_VERSION size\0" as *const u8 as *const ::core::ffi::c_char);
            }
            fprintf(
                stderr,
                b"protocol version mismatch (client %d, server %u)\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                PROTOCOL_VERSION,
                (*imsg).hdr.peerid & 0xff as uint32_t,
            );
            client_exitval = 1 as ::core::ffi::c_int;
            proc_exit(client_proc);
        }
        218 => {
            if datalen as usize != ::core::mem::size_of::<uint64_t>() as usize {
                fatalx(b"bad MSG_FLAGS string\0" as *const u8 as *const ::core::ffi::c_char);
            }
            memcpy(
                &raw mut client_flags as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<uint64_t>() as size_t,
            );
            log_debug(
                b"new flags are %#llx\0" as *const u8 as *const ::core::ffi::c_char,
                client_flags as ::core::ffi::c_ulonglong,
            );
        }
        209 => {
            if datalen == 0 as ssize_t
                || *data.offset((datalen - 1 as ssize_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                fatalx(b"bad MSG_SHELL string\0" as *const u8 as *const ::core::ffi::c_char);
            }
            client_exec(data, shell_command);
        }
        201 | 202 => {
            proc_send(
                client_peer,
                MSG_EXITING,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
        }
        204 => {
            proc_exit(client_proc);
        }
        300 => {
            file_read_open(
                &raw mut client_files,
                client_peer,
                imsg,
                1 as ::core::ffi::c_int,
                (client_flags & CLIENT_CONTROL as uint64_t == 0) as ::core::ffi::c_int,
                Some(
                    client_file_check_cb
                        as unsafe extern "C" fn(
                            *mut client,
                            *const ::core::ffi::c_char,
                            ::core::ffi::c_int,
                            ::core::ffi::c_int,
                            *mut evbuffer,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
                NULL,
            );
        }
        307 => {
            file_read_cancel(&raw mut client_files, imsg);
        }
        303 => {
            file_write_open(
                &raw mut client_files,
                client_peer,
                imsg,
                1 as ::core::ffi::c_int,
                (client_flags & CLIENT_CONTROL as uint64_t == 0) as ::core::ffi::c_int,
                Some(
                    client_file_check_cb
                        as unsafe extern "C" fn(
                            *mut client,
                            *const ::core::ffi::c_char,
                            ::core::ffi::c_int,
                            ::core::ffi::c_int,
                            *mut evbuffer,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
                NULL,
            );
        }
        304 => {
            file_write_data(&raw mut client_files, imsg);
        }
        306 => {
            file_write_close(&raw mut client_files, imsg);
        }
        211 | 212 | 213 => {
            fprintf(
                stderr,
                b"server version is too old for client\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
            proc_exit(client_proc);
        }
        _ => {
            log_debug(
                b"unknown message type %u\0" as *const u8 as *const ::core::ffi::c_char,
                (*imsg).hdr.type_0,
            );
        }
    };
}
unsafe extern "C" fn client_dispatch_attached(mut imsg: *mut imsg) {
    let mut sigact: sigaction = sigaction {
        __sigaction_handler: C2RustUnnamed_10 { sa_handler: None },
        sa_mask: __sigset_t { __val: [0; 16] },
        sa_flags: 0,
        sa_restorer: None,
    };
    let mut data: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut datalen: ssize_t = 0;
    data = (*imsg).data as *mut ::core::ffi::c_char;
    datalen = ((*imsg).hdr.len as usize).wrapping_sub(IMSG_HEADER_SIZE) as ssize_t;
    match (*imsg).hdr.type_0 {
        218 => {
            if datalen as usize != ::core::mem::size_of::<uint64_t>() as usize {
                fatalx(b"bad MSG_FLAGS string\0" as *const u8 as *const ::core::ffi::c_char);
            }
            memcpy(
                &raw mut client_flags as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<uint64_t>() as size_t,
            );
            log_debug(
                b"new flags are %#llx\0" as *const u8 as *const ::core::ffi::c_char,
                client_flags as ::core::ffi::c_ulonglong,
            );
        }
        201 | 202 => {
            if datalen == 0 as ssize_t
                || *data.offset((datalen - 1 as ssize_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                fatalx(b"bad MSG_DETACH string\0" as *const u8 as *const ::core::ffi::c_char);
            }
            client_exitsession = xstrdup(data);
            client_exittype = (*imsg).hdr.type_0 as msgtype;
            if (*imsg).hdr.type_0 == MSG_DETACHKILL as ::core::ffi::c_int as uint32_t {
                client_exitreason = CLIENT_EXIT_DETACHED_HUP;
            } else {
                client_exitreason = CLIENT_EXIT_DETACHED;
            }
            proc_send(
                client_peer,
                MSG_EXITING,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
        }
        217 => {
            if datalen == 0 as ssize_t
                || *data.offset((datalen - 1 as ssize_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
                || strlen(data).wrapping_add(1 as size_t) == datalen as size_t
            {
                fatalx(b"bad MSG_EXEC string\0" as *const u8 as *const ::core::ffi::c_char);
            }
            client_execcmd = xstrdup(data);
            client_execshell = xstrdup(
                data.offset(strlen(data) as isize)
                    .offset(1 as ::core::ffi::c_int as isize),
            );
            client_exittype = (*imsg).hdr.type_0 as msgtype;
            proc_send(
                client_peer,
                MSG_EXITING,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
        }
        203 => {
            client_dispatch_exit_message(data, datalen as size_t);
            if client_exitreason as ::core::ffi::c_uint
                == CLIENT_EXIT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                client_exitreason = CLIENT_EXIT_EXITED;
            }
            proc_send(
                client_peer,
                MSG_EXITING,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
        }
        204 => {
            if datalen != 0 as ssize_t {
                fatalx(b"bad MSG_EXITED size\0" as *const u8 as *const ::core::ffi::c_char);
            }
            proc_exit(client_proc);
        }
        210 => {
            if datalen != 0 as ssize_t {
                fatalx(b"bad MSG_SHUTDOWN size\0" as *const u8 as *const ::core::ffi::c_char);
            }
            proc_send(
                client_peer,
                MSG_EXITING,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
            client_exitreason = CLIENT_EXIT_SERVER_EXITED;
            client_exitval = 1 as ::core::ffi::c_int;
        }
        214 => {
            if datalen != 0 as ssize_t {
                fatalx(b"bad MSG_SUSPEND size\0" as *const u8 as *const ::core::ffi::c_char);
            }
            memset(
                &raw mut sigact as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<sigaction>() as size_t,
            );
            sigemptyset(&raw mut sigact.sa_mask);
            sigact.sa_flags = SA_RESTART;
            sigact.__sigaction_handler.sa_handler = SIG_DFL;
            if sigaction(
                SIGTSTP,
                &raw mut sigact,
                ::core::ptr::null_mut::<sigaction>(),
            ) != 0 as ::core::ffi::c_int
            {
                fatal(b"sigaction failed\0" as *const u8 as *const ::core::ffi::c_char);
            }
            client_suspended = 1 as ::core::ffi::c_int;
            kill(getpid(), SIGTSTP);
        }
        206 => {
            if datalen == 0 as ssize_t
                || *data.offset((datalen - 1 as ssize_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                fatalx(b"bad MSG_LOCK string\0" as *const u8 as *const ::core::ffi::c_char);
            }
            system(data);
            proc_send(
                client_peer,
                MSG_UNLOCK,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
        }
        _ => {
            log_debug(
                b"unknown message type %u\0" as *const u8 as *const ::core::ffi::c_char,
                (*imsg).hdr.type_0,
            );
        }
    };
}

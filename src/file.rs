use ::c2rust_bitfields;
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type event_base;
    pub type evbuffer;
    pub type bufferevent_ops;
    pub type args;
    pub type tmuxpeer;
    pub type hyperlinks;
    pub type screen_write_cline;
    pub type screen_sel;
    pub type screen_titles;
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
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn open(
        __file: *const ::core::ffi::c_char,
        __oflag: ::core::ffi::c_int,
        ...
    ) -> ::core::ffi::c_int;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fopen(
        __filename: *const ::core::ffi::c_char,
        __modes: *const ::core::ffi::c_char,
    ) -> *mut FILE;
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
    fn ferror(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
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
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn dup(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn event_once(
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
        _: *const timeval,
    ) -> ::core::ffi::c_int;
    fn evbuffer_new() -> *mut evbuffer;
    fn evbuffer_free(buf: *mut evbuffer);
    fn evbuffer_get_length(buf: *const evbuffer) -> size_t;
    fn evbuffer_add(
        buf: *mut evbuffer,
        data: *const ::core::ffi::c_void,
        datlen: size_t,
    ) -> ::core::ffi::c_int;
    fn evbuffer_add_vprintf(
        buf: *mut evbuffer,
        fmt: *const ::core::ffi::c_char,
        ap: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    fn evbuffer_drain(buf: *mut evbuffer, len: size_t) -> ::core::ffi::c_int;
    fn evbuffer_pullup(buf: *mut evbuffer, size: ssize_t) -> *mut ::core::ffi::c_uchar;
    fn bufferevent_free(bufev: *mut bufferevent);
    fn bufferevent_write(
        bufev: *mut bufferevent,
        data: *const ::core::ffi::c_void,
        size: size_t,
    ) -> ::core::ffi::c_int;
    fn bufferevent_enable(
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
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(_: *mut ::core::ffi::c_void, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn find_home() -> *const ::core::ffi::c_char;
    fn proc_send(
        _: *mut tmuxpeer,
        _: msgtype,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_void,
        _: size_t,
    ) -> ::core::ffi::c_int;
    fn server_client_unref(_: *mut client);
    fn server_client_get_cwd(_: *mut client, _: *mut session) -> *const ::core::ffi::c_char;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
}
pub type __builtin_va_list = [__va_list_tag; 1];
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __va_list_tag {
    pub gp_offset: ::core::ffi::c_uint,
    pub fp_offset: ::core::ffi::c_uint,
    pub overflow_arg_area: *mut ::core::ffi::c_void,
    pub reg_save_area: *mut ::core::ffi::c_void,
}
pub type __u_char = ::core::ffi::c_uchar;
pub type __u_short = ::core::ffi::c_ushort;
pub type __u_int = ::core::ffi::c_uint;
pub type __uint8_t = u8;
pub type __uint32_t = u32;
pub type __uint64_t = u64;
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type __pid_t = ::core::ffi::c_int;
pub type __time_t = ::core::ffi::c_long;
pub type __suseconds_t = ::core::ffi::c_long;
pub type u_char = __u_char;
pub type u_short = __u_short;
pub type u_int = __u_int;
pub type pid_t = __pid_t;
pub type ssize_t = isize;
pub type time_t = __time_t;
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct timeval {
    pub tv_sec: __time_t,
    pub tv_usec: __suseconds_t,
}
pub type __gnuc_va_list = __builtin_va_list;
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: ::core::ffi::c_int,
    pub _IO_read_ptr: *mut ::core::ffi::c_char,
    pub _IO_read_end: *mut ::core::ffi::c_char,
    pub _IO_read_base: *mut ::core::ffi::c_char,
    pub _IO_write_base: *mut ::core::ffi::c_char,
    pub _IO_write_ptr: *mut ::core::ffi::c_char,
    pub _IO_write_end: *mut ::core::ffi::c_char,
    pub _IO_buf_base: *mut ::core::ffi::c_char,
    pub _IO_buf_end: *mut ::core::ffi::c_char,
    pub _IO_save_base: *mut ::core::ffi::c_char,
    pub _IO_backup_base: *mut ::core::ffi::c_char,
    pub _IO_save_end: *mut ::core::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::core::ffi::c_int,
    #[bitfield(name = "_flags2", ty = "::core::ffi::c_int", bits = "0..=23")]
    pub _flags2: [u8; 3],
    pub _short_backupbuf: [::core::ffi::c_char; 1],
    pub _old_offset: __off_t,
    pub _cur_column: ::core::ffi::c_ushort,
    pub _vtable_offset: ::core::ffi::c_schar,
    pub _shortbuf: [::core::ffi::c_char; 1],
    pub _lock: *mut ::core::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::core::ffi::c_void,
    pub _prevchain: *mut *mut _IO_FILE,
    pub _mode: ::core::ffi::c_int,
    pub _unused3: ::core::ffi::c_int,
    pub _total_written: __uint64_t,
    pub _unused2: [::core::ffi::c_char; 8],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
pub type va_list = __gnuc_va_list;
pub type cc_t = ::core::ffi::c_uchar;
pub type speed_t = ::core::ffi::c_uint;
pub type tcflag_t = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct termios {
    pub c_iflag: tcflag_t,
    pub c_oflag: tcflag_t,
    pub c_cflag: tcflag_t,
    pub c_lflag: tcflag_t,
    pub c_line: cc_t,
    pub c_cc: [cc_t; 32],
    pub c2rust_unnamed: C2RustUnnamed_0,
    pub c2rust_unnamed_0: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub __ospeed: speed_t,
    pub c_ospeed: speed_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_0 {
    pub __ispeed: speed_t,
    pub c_ispeed: speed_t,
}
pub type uint8_t = __uint8_t;
pub type uint32_t = __uint32_t;
pub type uint64_t = __uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct event {
    pub ev_evcallback: event_callback,
    pub ev_timeout_pos: C2RustUnnamed_6,
    pub ev_fd: ::core::ffi::c_int,
    pub ev_base: *mut event_base,
    pub ev_: C2RustUnnamed_1,
    pub ev_events: ::core::ffi::c_short,
    pub ev_res: ::core::ffi::c_short,
    pub ev_timeout: timeval,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_1 {
    pub ev_io: C2RustUnnamed_4,
    pub ev_signal: C2RustUnnamed_2,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_2 {
    pub ev_signal_next: C2RustUnnamed_3,
    pub ev_ncalls: ::core::ffi::c_short,
    pub ev_pncalls: *mut ::core::ffi::c_short,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_3 {
    pub le_next: *mut event,
    pub le_prev: *mut *mut event,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_4 {
    pub ev_io_next: C2RustUnnamed_5,
    pub ev_timeout: timeval,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_5 {
    pub le_next: *mut event,
    pub le_prev: *mut *mut event,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_6 {
    pub ev_next_with_common_timeout: C2RustUnnamed_7,
    pub min_heap_idx: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_7 {
    pub tqe_next: *mut event,
    pub tqe_prev: *mut *mut event,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_callback {
    pub evcb_active_next: C2RustUnnamed_9,
    pub evcb_flags: ::core::ffi::c_short,
    pub evcb_pri: uint8_t,
    pub evcb_closure: uint8_t,
    pub evcb_cb_union: C2RustUnnamed_8,
    pub evcb_arg: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_8 {
    pub evcb_callback: Option<
        unsafe extern "C" fn(
            ::core::ffi::c_int,
            ::core::ffi::c_short,
            *mut ::core::ffi::c_void,
        ) -> (),
    >,
    pub evcb_selfcb:
        Option<unsafe extern "C" fn(*mut event_callback, *mut ::core::ffi::c_void) -> ()>,
    pub evcb_evfinalize: Option<unsafe extern "C" fn(*mut event, *mut ::core::ffi::c_void) -> ()>,
    pub evcb_cbfinalize:
        Option<unsafe extern "C" fn(*mut event_callback, *mut ::core::ffi::c_void) -> ()>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_9 {
    pub tqe_next: *mut event_callback,
    pub tqe_prev: *mut *mut event_callback,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bufferevent {
    pub ev_base: *mut event_base,
    pub be_ops: *const bufferevent_ops,
    pub ev_read: event,
    pub ev_write: event,
    pub input: *mut evbuffer,
    pub output: *mut evbuffer,
    pub wm_read: event_watermark,
    pub wm_write: event_watermark,
    pub readcb: bufferevent_data_cb,
    pub writecb: bufferevent_data_cb,
    pub errorcb: bufferevent_event_cb,
    pub cbarg: *mut ::core::ffi::c_void,
    pub timeout_read: timeval,
    pub timeout_write: timeval,
    pub enabled: ::core::ffi::c_short,
}
pub type bufferevent_event_cb = Option<
    unsafe extern "C" fn(*mut bufferevent, ::core::ffi::c_short, *mut ::core::ffi::c_void) -> (),
>;
pub type bufferevent_data_cb =
    Option<unsafe extern "C" fn(*mut bufferevent, *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_watermark {
    pub low: size_t,
    pub high: size_t,
}
pub type bitstr_t = ::core::ffi::c_uchar;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ibuf {
    pub entry: C2RustUnnamed_10,
    pub buf: *mut ::core::ffi::c_uchar,
    pub size: size_t,
    pub max: size_t,
    pub wpos: size_t,
    pub rpos: size_t,
    pub fd: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_10 {
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
pub type msgtype = ::core::ffi::c_uint;
pub const MSG_WRITE_DONE: msgtype = 308;
pub const MSG_READ_CANCEL: msgtype = 307;
pub const MSG_WRITE_CLOSE: msgtype = 306;
pub const MSG_WRITE_READY: msgtype = 305;
pub const MSG_WRITE: msgtype = 304;
pub const MSG_WRITE_OPEN: msgtype = 303;
pub const MSG_READ_DONE: msgtype = 302;
pub const MSG_READ: msgtype = 301;
pub const MSG_READ_OPEN: msgtype = 300;
pub const MSG_FLAGS: msgtype = 218;
pub const MSG_EXEC: msgtype = 217;
pub const MSG_WAKEUP: msgtype = 216;
pub const MSG_UNLOCK: msgtype = 215;
pub const MSG_SUSPEND: msgtype = 214;
pub const MSG_OLDSTDOUT: msgtype = 213;
pub const MSG_OLDSTDIN: msgtype = 212;
pub const MSG_OLDSTDERR: msgtype = 211;
pub const MSG_SHUTDOWN: msgtype = 210;
pub const MSG_SHELL: msgtype = 209;
pub const MSG_RESIZE: msgtype = 208;
pub const MSG_READY: msgtype = 207;
pub const MSG_LOCK: msgtype = 206;
pub const MSG_EXITING: msgtype = 205;
pub const MSG_EXITED: msgtype = 204;
pub const MSG_EXIT: msgtype = 203;
pub const MSG_DETACHKILL: msgtype = 202;
pub const MSG_DETACH: msgtype = 201;
pub const MSG_COMMAND: msgtype = 200;
pub const MSG_IDENTIFY_TERMINFO: msgtype = 112;
pub const MSG_IDENTIFY_LONGFLAGS: msgtype = 111;
pub const MSG_IDENTIFY_STDOUT: msgtype = 110;
pub const MSG_IDENTIFY_FEATURES: msgtype = 109;
pub const MSG_IDENTIFY_CWD: msgtype = 108;
pub const MSG_IDENTIFY_CLIENTPID: msgtype = 107;
pub const MSG_IDENTIFY_DONE: msgtype = 106;
pub const MSG_IDENTIFY_ENVIRON: msgtype = 105;
pub const MSG_IDENTIFY_STDIN: msgtype = 104;
pub const MSG_IDENTIFY_OLDCWD: msgtype = 103;
pub const MSG_IDENTIFY_TTYNAME: msgtype = 102;
pub const MSG_IDENTIFY_TERM: msgtype = 101;
pub const MSG_IDENTIFY_FLAGS: msgtype = 100;
pub const MSG_VERSION: msgtype = 12;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_open {
    pub stream: ::core::ffi::c_int,
    pub fd: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_data {
    pub stream: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_done {
    pub stream: ::core::ffi::c_int,
    pub error: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_read_cancel {
    pub stream: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_open {
    pub stream: ::core::ffi::c_int,
    pub fd: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_data {
    pub stream: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_ready {
    pub stream: ::core::ffi::c_int,
    pub error: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_close {
    pub stream: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_write_done {
    pub stream: ::core::ffi::c_int,
    pub error: ::core::ffi::c_int,
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
    pub exit_type: C2RustUnnamed_34,
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
pub type key_code = ::core::ffi::c_ulonglong;
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
pub struct progress_bar {
    pub state: progress_bar_state,
    pub progress: ::core::ffi::c_int,
}
pub type progress_bar_state = ::core::ffi::c_uint;
pub const PROGRESS_BAR_PAUSED: progress_bar_state = 4;
pub const PROGRESS_BAR_INDETERMINATE: progress_bar_state = 3;
pub const PROGRESS_BAR_ERROR: progress_bar_state = 2;
pub const PROGRESS_BAR_NORMAL: progress_bar_state = 1;
pub const PROGRESS_BAR_HIDDEN: progress_bar_state = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct grid_cell {
    pub data: utf8_data,
    pub attr: u_short,
    pub flags: u_char,
    pub fg: ::core::ffi::c_int,
    pub bg: ::core::ffi::c_int,
    pub us: ::core::ffi::c_int,
    pub link: u_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct utf8_data {
    pub data: [u_char; 32],
    pub have: u_char,
    pub size: u_char,
    pub width: u_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct grid {
    pub flags: ::core::ffi::c_int,
    pub sx: u_int,
    pub sy: u_int,
    pub hscrolled: u_int,
    pub hsize: u_int,
    pub hlimit: u_int,
    pub scroll_added: u_int,
    pub scroll_collected: u_int,
    pub scroll_generation: u_int,
    pub linedata: *mut grid_line,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct grid_line {
    pub celldata: *mut grid_cell_entry,
    pub extddata: *mut grid_extd_entry,
    pub cellused: u_short,
    pub cellsize: u_short,
    pub extdsize: u_int,
    pub time: u_int,
    pub osc133_data: osc133_data,
    pub flags: u_short,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct osc133_data {
    pub prompt_col: u_short,
    pub cmd_col: u_short,
    pub out_start_col: u_short,
    pub out_end_col: u_short,
    pub exit_status: u_char,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct grid_extd_entry {
    pub data: utf8_char,
    pub attr: u_short,
    pub flags: u_char,
    pub fg: ::core::ffi::c_int,
    pub bg: ::core::ffi::c_int,
    pub us: ::core::ffi::c_int,
    pub link: u_int,
}
pub type utf8_char = u_int;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct grid_cell_entry {
    pub c2rust_unnamed: C2RustUnnamed_13,
    pub flags: u_char,
}
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
pub type screen_cursor_style = ::core::ffi::c_uint;
pub const SCREEN_CURSOR_BAR: screen_cursor_style = 3;
pub const SCREEN_CURSOR_UNDERLINE: screen_cursor_style = 2;
pub const SCREEN_CURSOR_BLOCK: screen_cursor_style = 1;
pub const SCREEN_CURSOR_DEFAULT: screen_cursor_style = 0;
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
pub struct style {
    pub gc: grid_cell,
    pub ignore: ::core::ffi::c_int,
    pub dim: ::core::ffi::c_int,
    pub fill: ::core::ffi::c_int,
    pub align: style_align,
    pub list: style_list,
    pub range_type: style_range_type,
    pub range_argument: u_int,
    pub range_string: [::core::ffi::c_char; 16],
    pub width: ::core::ffi::c_int,
    pub width_percentage: ::core::ffi::c_int,
    pub pad: ::core::ffi::c_int,
    pub default_type: style_default_type,
    pub link: u_int,
}
pub type style_default_type = ::core::ffi::c_uint;
pub const STYLE_DEFAULT_SET: style_default_type = 3;
pub const STYLE_DEFAULT_POP: style_default_type = 2;
pub const STYLE_DEFAULT_PUSH: style_default_type = 1;
pub const STYLE_DEFAULT_BASE: style_default_type = 0;
pub type style_range_type = ::core::ffi::c_uint;
pub const STYLE_RANGE_CONTROL: style_range_type = 7;
pub const STYLE_RANGE_USER: style_range_type = 6;
pub const STYLE_RANGE_SESSION: style_range_type = 5;
pub const STYLE_RANGE_WINDOW: style_range_type = 4;
pub const STYLE_RANGE_PANE: style_range_type = 3;
pub const STYLE_RANGE_RIGHT: style_range_type = 2;
pub const STYLE_RANGE_LEFT: style_range_type = 1;
pub const STYLE_RANGE_NONE: style_range_type = 0;
pub type style_list = ::core::ffi::c_uint;
pub const STYLE_LIST_RIGHT_MARKER: style_list = 4;
pub const STYLE_LIST_LEFT_MARKER: style_list = 3;
pub const STYLE_LIST_FOCUS: style_list = 2;
pub const STYLE_LIST_ON: style_list = 1;
pub const STYLE_LIST_OFF: style_list = 0;
pub type style_align = ::core::ffi::c_uint;
pub const STYLE_ALIGN_ABSOLUTE_CENTRE: style_align = 4;
pub const STYLE_ALIGN_RIGHT: style_align = 3;
pub const STYLE_ALIGN_CENTRE: style_align = 2;
pub const STYLE_ALIGN_LEFT: style_align = 1;
pub const STYLE_ALIGN_DEFAULT: style_align = 0;
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
pub struct window_pane_offset {
    pub used: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct style_line_entry {
    pub expanded: *mut ::core::ffi::c_char,
    pub ranges: style_ranges,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct style_ranges {
    pub tqh_first: *mut style_range,
    pub tqh_last: *mut *mut style_range,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct style_range {
    pub type_0: style_range_type,
    pub argument: u_int,
    pub string: [::core::ffi::c_char; 16],
    pub start: u_int,
    pub end: u_int,
    pub entry: C2RustUnnamed_30,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_30 {
    pub tqe_next: *mut style_range,
    pub tqe_prev: *mut *mut style_range,
}
pub type client_theme = ::core::ffi::c_uint;
pub const THEME_DARK: client_theme = 2;
pub const THEME_LIGHT: client_theme = 1;
pub const THEME_UNKNOWN: client_theme = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct colour_palette {
    pub fg: ::core::ffi::c_int,
    pub bg: ::core::ffi::c_int,
    pub palette: *mut ::core::ffi::c_int,
    pub default_palette: *mut ::core::ffi::c_int,
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
    pub entry: C2RustUnnamed_31,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_31 {
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
pub type layout_type = ::core::ffi::c_uint;
pub const LAYOUT_WINDOWPANE: layout_type = 2;
pub const LAYOUT_TOPBOTTOM: layout_type = 1;
pub const LAYOUT_LEFTRIGHT: layout_type = 0;
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
pub type C2RustUnnamed_34 = ::core::ffi::c_uint;
pub const CLIENT_EXIT_DETACH: C2RustUnnamed_34 = 2;
pub const CLIENT_EXIT_SHUTDOWN: C2RustUnnamed_34 = 1;
pub const CLIENT_EXIT_RETURN: C2RustUnnamed_34 = 0;
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
pub const EIO: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const E2BIG: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const EBADF: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const ENOMEM: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const O_WRONLY: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const O_CREAT: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const O_APPEND: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;
pub const O_NONBLOCK: ::core::ffi::c_int = 0o4000 as ::core::ffi::c_int;
pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const EV_TIMEOUT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const EV_READ: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const EV_WRITE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const BEV_EVENT_ERROR: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const EVBUFFER_ERROR: ::core::ffi::c_int = BEV_EVENT_ERROR;
pub const IMSG_HEADER_SIZE: usize = ::core::mem::size_of::<imsg_hdr>();
pub const MAX_IMSGSIZE: ::core::ffi::c_int = 16384 as ::core::ffi::c_int;
pub const CLIENT_ATTACHED: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const CLIENT_DEAD: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CLIENT_WRITE_ACK: ::core::ffi::c_ulonglong = 0x4000000000 as ::core::ffi::c_ulonglong;
static mut file_next_stream: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_PREV(mut elm: *mut client_file) -> *mut client_file {
    if !(*elm).entry.rbe_left.is_null() {
        elm = (*elm).entry.rbe_left;
        while !(*elm).entry.rbe_right.is_null() {
            elm = (*elm).entry.rbe_right;
        }
    } else if !(*elm).entry.rbe_parent.is_null()
        && elm == (*(*elm).entry.rbe_parent).entry.rbe_right
    {
        elm = (*elm).entry.rbe_parent;
    } else {
        while !(*elm).entry.rbe_parent.is_null() && elm == (*(*elm).entry.rbe_parent).entry.rbe_left
        {
            elm = (*elm).entry.rbe_parent;
        }
        elm = (*elm).entry.rbe_parent;
    }
    return elm;
}
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_MINMAX(
    mut head: *mut client_files,
    mut val: ::core::ffi::c_int,
) -> *mut client_file {
    let mut tmp: *mut client_file = (*head).rbh_root;
    let mut parent: *mut client_file = ::core::ptr::null_mut::<client_file>();
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
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_NFIND(
    mut head: *mut client_files,
    mut elm: *mut client_file,
) -> *mut client_file {
    let mut tmp: *mut client_file = (*head).rbh_root;
    let mut res: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = file_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            res = tmp;
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_REMOVE(
    mut head: *mut client_files,
    mut elm: *mut client_file,
) -> *mut client_file {
    let mut current_block: u64;
    let mut child: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut parent: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut old: *mut client_file = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut client_file = ::core::ptr::null_mut::<client_file>();
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
        current_block = 7372538432882326502;
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
        client_files_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_INSERT(
    mut head: *mut client_files,
    mut elm: *mut client_file,
) -> *mut client_file {
    let mut tmp: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut parent: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = file_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<client_file>();
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
    client_files_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<client_file>();
}
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_INSERT_COLOR(
    mut head: *mut client_files,
    mut elm: *mut client_file,
) {
    let mut parent: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut gparent: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut tmp: *mut client_file = ::core::ptr::null_mut::<client_file>();
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
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_REMOVE_COLOR(
    mut head: *mut client_files,
    mut parent: *mut client_file,
    mut elm: *mut client_file,
) {
    let mut tmp: *mut client_file = ::core::ptr::null_mut::<client_file>();
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
                    let mut oleft: *mut client_file = ::core::ptr::null_mut::<client_file>();
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
                    let mut oright: *mut client_file = ::core::ptr::null_mut::<client_file>();
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
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_FIND(
    mut head: *mut client_files,
    mut elm: *mut client_file,
) -> *mut client_file {
    let mut tmp: *mut client_file = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = file_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<client_file>();
}
#[no_mangle]
pub unsafe extern "C" fn client_files_RB_NEXT(mut elm: *mut client_file) -> *mut client_file {
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
unsafe extern "C" fn file_get_path(
    mut c: *mut client,
    mut file: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut home: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut full_path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if strncmp(
        file,
        b"~/\0" as *const u8 as *const ::core::ffi::c_char,
        2 as size_t,
    ) != 0 as ::core::ffi::c_int
    {
        path = xstrdup(file);
    } else {
        home = find_home();
        if home.is_null() {
            home = b"\0" as *const u8 as *const ::core::ffi::c_char;
        }
        xasprintf(
            &raw mut path,
            b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
            home,
            file.offset(1 as ::core::ffi::c_int as isize),
        );
    }
    if *path as ::core::ffi::c_int == '/' as i32 {
        return path;
    }
    xasprintf(
        &raw mut full_path,
        b"%s/%s\0" as *const u8 as *const ::core::ffi::c_char,
        server_client_get_cwd(c, ::core::ptr::null_mut::<session>()),
        path,
    );
    free(path as *mut ::core::ffi::c_void);
    return full_path;
}
#[no_mangle]
pub unsafe extern "C" fn file_cmp(
    mut cf1: *mut client_file,
    mut cf2: *mut client_file,
) -> ::core::ffi::c_int {
    if (*cf1).stream < (*cf2).stream {
        return -(1 as ::core::ffi::c_int);
    }
    if (*cf1).stream > (*cf2).stream {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_create_with_peer(
    mut peer: *mut tmuxpeer,
    mut files: *mut client_files,
    mut stream: ::core::ffi::c_int,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) -> *mut client_file {
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    cf = xcalloc(1 as size_t, ::core::mem::size_of::<client_file>() as size_t) as *mut client_file;
    (*cf).c = ::core::ptr::null_mut::<client>();
    (*cf).references = 1 as ::core::ffi::c_int;
    (*cf).stream = stream;
    (*cf).buffer = evbuffer_new();
    if (*cf).buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    (*cf).cb = cb;
    (*cf).data = cbdata;
    (*cf).peer = peer;
    (*cf).tree = files as *mut client_files;
    client_files_RB_INSERT(files, cf);
    return cf;
}
#[no_mangle]
pub unsafe extern "C" fn file_create_with_client(
    mut c: *mut client,
    mut stream: ::core::ffi::c_int,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) -> *mut client_file {
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if !c.is_null() && (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
        c = ::core::ptr::null_mut::<client>();
    }
    cf = xcalloc(1 as size_t, ::core::mem::size_of::<client_file>() as size_t) as *mut client_file;
    (*cf).c = c;
    (*cf).references = 1 as ::core::ffi::c_int;
    (*cf).stream = stream;
    (*cf).buffer = evbuffer_new();
    if (*cf).buffer.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    (*cf).cb = cb;
    (*cf).data = cbdata;
    if !(*cf).c.is_null() {
        (*cf).peer = (*(*cf).c).peer;
        (*cf).tree = &raw mut (*(*cf).c).files as *mut client_files;
        client_files_RB_INSERT(&raw mut (*(*cf).c).files, cf);
        (*(*cf).c).references += 1;
    }
    return cf;
}
#[no_mangle]
pub unsafe extern "C" fn file_free(mut cf: *mut client_file) {
    (*cf).references -= 1;
    if (*cf).references != 0 as ::core::ffi::c_int {
        return;
    }
    evbuffer_free((*cf).buffer);
    free((*cf).path as *mut ::core::ffi::c_void);
    if !(*cf).tree.is_null() {
        client_files_RB_REMOVE((*cf).tree as *mut client_files, cf);
    }
    if !(*cf).c.is_null() {
        server_client_unref((*cf).c);
    }
    free(cf as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn file_fire_done_cb(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    let mut c: *mut client = (*cf).c;
    if (*cf).cb.is_some()
        && ((*cf).closed != 0 || c.is_null() || !(*c).flags & CLIENT_DEAD as uint64_t != 0)
    {
        (*cf).cb.expect("non-null function pointer")(
            c,
            (*cf).path,
            (*cf).error,
            1 as ::core::ffi::c_int,
            (*cf).buffer,
            (*cf).data,
        );
    }
    file_free(cf);
}
#[no_mangle]
pub unsafe extern "C" fn file_fire_done(mut cf: *mut client_file) {
    event_once(
        -(1 as ::core::ffi::c_int),
        EV_TIMEOUT as ::core::ffi::c_short,
        Some(
            file_fire_done_cb
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        cf as *mut ::core::ffi::c_void,
        ::core::ptr::null::<timeval>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn file_fire_read(mut cf: *mut client_file) {
    if (*cf).cb.is_some() {
        (*cf).cb.expect("non-null function pointer")(
            (*cf).c,
            (*cf).path,
            (*cf).error,
            0 as ::core::ffi::c_int,
            (*cf).buffer,
            (*cf).data,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn file_can_print(mut c: *mut client) -> ::core::ffi::c_int {
    if c.is_null()
        || (*c).flags & CLIENT_ATTACHED as uint64_t != 0
        || (*c).flags & CLIENT_DEAD as uint64_t != 0
        || (*c).flags & CLIENT_CONTROL as uint64_t != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_print(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    file_vprint(c, fmt, ap);
}
#[no_mangle]
pub unsafe extern "C" fn file_vprint(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
) {
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: C2RustUnnamed_12 {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut msg: msg_write_open = msg_write_open {
        stream: 0,
        fd: 0,
        flags: 0,
    };
    if file_can_print(c) == 0 {
        return;
    }
    find.stream = 1 as ::core::ffi::c_int;
    cf = client_files_RB_FIND(&raw mut (*c).files, &raw mut find);
    if cf.is_null() {
        cf = file_create_with_client(c, 1 as ::core::ffi::c_int, None, NULL);
        (*cf).path = xstrdup(b"-\0" as *const u8 as *const ::core::ffi::c_char);
        evbuffer_add_vprintf((*cf).buffer, fmt, ap);
        msg.stream = 1 as ::core::ffi::c_int;
        msg.fd = STDOUT_FILENO;
        msg.flags = 0 as ::core::ffi::c_int;
        proc_send(
            (*c).peer,
            MSG_WRITE_OPEN,
            -(1 as ::core::ffi::c_int),
            &raw mut msg as *const ::core::ffi::c_void,
            ::core::mem::size_of::<msg_write_open>() as size_t,
        );
    } else {
        evbuffer_add_vprintf((*cf).buffer, fmt, ap);
        file_push(cf);
    };
}
#[no_mangle]
pub unsafe extern "C" fn file_print_buffer(
    mut c: *mut client,
    mut data: *mut ::core::ffi::c_void,
    mut size: size_t,
) {
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: C2RustUnnamed_12 {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut msg: msg_write_open = msg_write_open {
        stream: 0,
        fd: 0,
        flags: 0,
    };
    if file_can_print(c) == 0 {
        return;
    }
    find.stream = 1 as ::core::ffi::c_int;
    cf = client_files_RB_FIND(&raw mut (*c).files, &raw mut find);
    if cf.is_null() {
        cf = file_create_with_client(c, 1 as ::core::ffi::c_int, None, NULL);
        (*cf).path = xstrdup(b"-\0" as *const u8 as *const ::core::ffi::c_char);
        evbuffer_add((*cf).buffer, data, size);
        msg.stream = 1 as ::core::ffi::c_int;
        msg.fd = STDOUT_FILENO;
        msg.flags = 0 as ::core::ffi::c_int;
        proc_send(
            (*c).peer,
            MSG_WRITE_OPEN,
            -(1 as ::core::ffi::c_int),
            &raw mut msg as *const ::core::ffi::c_void,
            ::core::mem::size_of::<msg_write_open>() as size_t,
        );
    } else {
        evbuffer_add((*cf).buffer, data, size);
        file_push(cf);
    };
}
#[no_mangle]
pub unsafe extern "C" fn file_error(
    mut c: *mut client,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: C2RustUnnamed_12 {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut msg: msg_write_open = msg_write_open {
        stream: 0,
        fd: 0,
        flags: 0,
    };
    let mut ap: ::core::ffi::VaList;
    if file_can_print(c) == 0 {
        return;
    }
    ap = args.clone();
    find.stream = 2 as ::core::ffi::c_int;
    cf = client_files_RB_FIND(&raw mut (*c).files, &raw mut find);
    if cf.is_null() {
        cf = file_create_with_client(c, 2 as ::core::ffi::c_int, None, NULL);
        (*cf).path = xstrdup(b"-\0" as *const u8 as *const ::core::ffi::c_char);
        evbuffer_add_vprintf((*cf).buffer, fmt, ap);
        msg.stream = 2 as ::core::ffi::c_int;
        msg.fd = STDERR_FILENO;
        msg.flags = 0 as ::core::ffi::c_int;
        proc_send(
            (*c).peer,
            MSG_WRITE_OPEN,
            -(1 as ::core::ffi::c_int),
            &raw mut msg as *const ::core::ffi::c_void,
            ::core::mem::size_of::<msg_write_open>() as size_t,
        );
    } else {
        evbuffer_add_vprintf((*cf).buffer, fmt, ap);
        file_push(cf);
    };
}
#[no_mangle]
pub unsafe extern "C" fn file_write(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut bdata: *const ::core::ffi::c_void,
    mut bsize: size_t,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) {
    let mut current_block: u64;
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut msg: *mut msg_write_open = ::core::ptr::null_mut::<msg_write_open>();
    let mut msglen: size_t = 0;
    let mut fd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let fresh0 = file_next_stream;
    file_next_stream = file_next_stream + 1;
    let mut stream: u_int = fresh0 as u_int;
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut mode: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if strcmp(path, b"-\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        cf = file_create_with_client(c, stream as ::core::ffi::c_int, cb, cbdata);
        (*cf).path = xstrdup(b"-\0" as *const u8 as *const ::core::ffi::c_char);
        fd = STDOUT_FILENO;
        if c.is_null()
            || (*c).flags & CLIENT_ATTACHED as uint64_t != 0
            || (*c).flags & CLIENT_CONTROL as uint64_t != 0
        {
            (*cf).error = EBADF;
            current_block = 4636144702248558238;
        } else {
            current_block = 8821498768635335055;
        }
    } else {
        cf = file_create_with_client(c, stream as ::core::ffi::c_int, cb, cbdata);
        (*cf).path = file_get_path(c, path);
        if c.is_null() || (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
            if flags & O_APPEND != 0 {
                mode = b"ab\0" as *const u8 as *const ::core::ffi::c_char;
            } else {
                mode = b"wb\0" as *const u8 as *const ::core::ffi::c_char;
            }
            f = fopen((*cf).path, mode) as *mut FILE;
            if f.is_null() {
                (*cf).error = *__errno_location();
            } else if fwrite(bdata, 1 as size_t, bsize, f) as size_t != bsize {
                fclose(f);
                (*cf).error = EIO;
            } else {
                fclose(f);
            }
            current_block = 4636144702248558238;
        } else {
            current_block = 8821498768635335055;
        }
    }
    match current_block {
        8821498768635335055 => {
            evbuffer_add((*cf).buffer, bdata, bsize);
            msglen = strlen((*cf).path)
                .wrapping_add(1 as size_t)
                .wrapping_add(::core::mem::size_of::<msg_write_open>() as size_t);
            if msglen > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE) {
                (*cf).error = E2BIG;
            } else {
                msg = xmalloc(msglen) as *mut msg_write_open;
                (*msg).stream = (*cf).stream;
                (*msg).fd = fd;
                (*msg).flags = flags;
                memcpy(
                    msg.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                    (*cf).path as *const ::core::ffi::c_void,
                    msglen.wrapping_sub(::core::mem::size_of::<msg_write_open>() as size_t),
                );
                if proc_send(
                    (*cf).peer,
                    MSG_WRITE_OPEN,
                    -(1 as ::core::ffi::c_int),
                    msg as *const ::core::ffi::c_void,
                    msglen,
                ) != 0 as ::core::ffi::c_int
                {
                    free(msg as *mut ::core::ffi::c_void);
                    (*cf).error = EINVAL;
                } else {
                    free(msg as *mut ::core::ffi::c_void);
                    return;
                }
            }
        }
        _ => {}
    }
    file_fire_done(cf);
}
#[no_mangle]
pub unsafe extern "C" fn file_read(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) -> *mut client_file {
    let mut current_block: u64;
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut msg: *mut msg_read_open = ::core::ptr::null_mut::<msg_read_open>();
    let mut msglen: size_t = 0;
    let mut fd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
    let fresh1 = file_next_stream;
    file_next_stream = file_next_stream + 1;
    let mut stream: u_int = fresh1 as u_int;
    let mut f: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut size: size_t = 0;
    let mut buffer: [::core::ffi::c_char; 8192] = [0; 8192];
    if strcmp(path, b"-\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        cf = file_create_with_client(c, stream as ::core::ffi::c_int, cb, cbdata);
        (*cf).path = xstrdup(b"-\0" as *const u8 as *const ::core::ffi::c_char);
        fd = STDIN_FILENO;
        if c.is_null()
            || (*c).flags & CLIENT_ATTACHED as uint64_t != 0
            || (*c).flags & CLIENT_CONTROL as uint64_t != 0
        {
            (*cf).error = EBADF;
            current_block = 17369485759464587280;
        } else {
            current_block = 17710118112003399050;
        }
    } else {
        cf = file_create_with_client(c, stream as ::core::ffi::c_int, cb, cbdata);
        (*cf).path = file_get_path(c, path);
        if c.is_null() || (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
            f = fopen(
                (*cf).path,
                b"rb\0" as *const u8 as *const ::core::ffi::c_char,
            ) as *mut FILE;
            if f.is_null() {
                (*cf).error = *__errno_location();
            } else {
                loop {
                    size = fread(
                        &raw mut buffer as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                        1 as size_t,
                        ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as size_t,
                        f,
                    ) as size_t;
                    if ferror(f) != 0 {
                        (*cf).error = *__errno_location();
                        current_block = 17369485759464587280;
                        break;
                    } else if evbuffer_add(
                        (*cf).buffer,
                        &raw mut buffer as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                        size,
                    ) != 0 as ::core::ffi::c_int
                    {
                        (*cf).error = ENOMEM;
                        current_block = 17369485759464587280;
                        break;
                    } else if size != ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as usize
                    {
                        current_block = 4808432441040389987;
                        break;
                    }
                }
                match current_block {
                    17369485759464587280 => {}
                    _ => {
                        if ferror(f) != 0 {
                            (*cf).error = EIO;
                        }
                    }
                }
            }
            current_block = 17369485759464587280;
        } else {
            current_block = 17710118112003399050;
        }
    }
    match current_block {
        17710118112003399050 => {
            msglen = strlen((*cf).path)
                .wrapping_add(1 as size_t)
                .wrapping_add(::core::mem::size_of::<msg_read_open>() as size_t);
            if msglen > (MAX_IMSGSIZE as usize).wrapping_sub(IMSG_HEADER_SIZE) {
                (*cf).error = E2BIG;
            } else {
                msg = xmalloc(msglen) as *mut msg_read_open;
                (*msg).stream = (*cf).stream;
                (*msg).fd = fd;
                memcpy(
                    msg.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                    (*cf).path as *const ::core::ffi::c_void,
                    msglen.wrapping_sub(::core::mem::size_of::<msg_read_open>() as size_t),
                );
                if proc_send(
                    (*cf).peer,
                    MSG_READ_OPEN,
                    -(1 as ::core::ffi::c_int),
                    msg as *const ::core::ffi::c_void,
                    msglen,
                ) != 0 as ::core::ffi::c_int
                {
                    free(msg as *mut ::core::ffi::c_void);
                    (*cf).error = EINVAL;
                } else {
                    free(msg as *mut ::core::ffi::c_void);
                    return cf;
                }
            }
        }
        _ => {}
    }
    if !f.is_null() {
        fclose(f);
    }
    file_fire_done(cf);
    return ::core::ptr::null_mut::<client_file>();
}
#[no_mangle]
pub unsafe extern "C" fn file_cancel(mut cf: *mut client_file) {
    let mut msg: msg_read_cancel = msg_read_cancel { stream: 0 };
    log_debug(
        b"read cancel file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    if (*cf).closed != 0 {
        return;
    }
    (*cf).closed = 1 as ::core::ffi::c_int;
    msg.stream = (*cf).stream;
    proc_send(
        (*cf).peer,
        MSG_READ_CANCEL,
        -(1 as ::core::ffi::c_int),
        &raw mut msg as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_read_cancel>() as size_t,
    );
}
unsafe extern "C" fn file_push_cb(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    if (*cf).c.is_null() || !(*(*cf).c).flags & CLIENT_DEAD as uint64_t != 0 {
        file_push(cf);
    }
    file_free(cf);
}
#[no_mangle]
pub unsafe extern "C" fn file_push(mut cf: *mut client_file) {
    let mut msg: *mut msg_write_data = ::core::ptr::null_mut::<msg_write_data>();
    let mut msglen: size_t = 0;
    let mut sent: size_t = 0;
    let mut left: size_t = 0;
    let mut close_0: msg_write_close = msg_write_close { stream: 0 };
    msg = xmalloc(::core::mem::size_of::<msg_write_data>() as size_t) as *mut msg_write_data;
    left = evbuffer_get_length((*cf).buffer);
    while left != 0 as size_t {
        sent = left;
        if sent
            > (MAX_IMSGSIZE as usize)
                .wrapping_sub(IMSG_HEADER_SIZE)
                .wrapping_sub(::core::mem::size_of::<msg_write_data>() as usize)
        {
            sent = (MAX_IMSGSIZE as usize)
                .wrapping_sub(IMSG_HEADER_SIZE)
                .wrapping_sub(::core::mem::size_of::<msg_write_data>() as usize)
                as size_t;
        }
        msglen = (::core::mem::size_of::<msg_write_data>() as usize).wrapping_add(sent as usize)
            as size_t;
        msg = xrealloc(msg as *mut ::core::ffi::c_void, msglen) as *mut msg_write_data;
        (*msg).stream = (*cf).stream;
        memcpy(
            msg.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            evbuffer_pullup((*cf).buffer, -(1 as ::core::ffi::c_int) as ssize_t)
                as *const ::core::ffi::c_void,
            sent,
        );
        if proc_send(
            (*cf).peer,
            MSG_WRITE,
            -(1 as ::core::ffi::c_int),
            msg as *const ::core::ffi::c_void,
            msglen,
        ) != 0 as ::core::ffi::c_int
        {
            break;
        }
        evbuffer_drain((*cf).buffer, sent);
        left = evbuffer_get_length((*cf).buffer);
        log_debug(
            b"file %d sent %zu, left %zu\0" as *const u8 as *const ::core::ffi::c_char,
            (*cf).stream,
            sent,
            left,
        );
    }
    if left != 0 as size_t {
        (*cf).references += 1;
        event_once(
            -(1 as ::core::ffi::c_int),
            EV_TIMEOUT as ::core::ffi::c_short,
            Some(
                file_push_cb
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            cf as *mut ::core::ffi::c_void,
            ::core::ptr::null::<timeval>(),
        );
    } else if (*cf).stream > 2 as ::core::ffi::c_int {
        close_0.stream = (*cf).stream;
        proc_send(
            (*cf).peer,
            MSG_WRITE_CLOSE,
            -(1 as ::core::ffi::c_int),
            &raw mut close_0 as *const ::core::ffi::c_void,
            ::core::mem::size_of::<msg_write_close>() as size_t,
        );
        if (*cf).c.is_null()
            || !(*(*cf).c).flags as ::core::ffi::c_ulonglong & CLIENT_WRITE_ACK != 0
        {
            file_fire_done(cf);
        }
    }
    free(msg as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn file_write_left(mut files: *mut client_files) -> ::core::ffi::c_int {
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut left: size_t = 0;
    let mut waiting: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    cf = client_files_RB_MINMAX(files, RB_NEGINF);
    while !cf.is_null() {
        if !(*cf).event.is_null() {
            left = evbuffer_get_length((*(*cf).event).output);
            if left != 0 as size_t {
                waiting += 1;
                log_debug(
                    b"file %u %zu bytes left\0" as *const u8 as *const ::core::ffi::c_char,
                    (*cf).stream,
                    left,
                );
            }
        }
        cf = client_files_RB_NEXT(cf);
    }
    return (waiting != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C" fn file_write_finished(mut cf: *mut client_file) {
    let mut msg: msg_write_done = msg_write_done {
        stream: 0,
        error: 0,
    };
    if !(*cf).event.is_null() {
        bufferevent_free((*cf).event);
        (*cf).event = ::core::ptr::null_mut::<bufferevent>();
    }
    if (*cf).fd != -(1 as ::core::ffi::c_int) {
        if close((*cf).fd) != 0 as ::core::ffi::c_int && (*cf).error == 0 as ::core::ffi::c_int {
            (*cf).error = *__errno_location();
        }
        (*cf).fd = -(1 as ::core::ffi::c_int);
    }
    msg.stream = (*cf).stream;
    msg.error = (*cf).error;
    proc_send(
        (*cf).peer,
        MSG_WRITE_DONE,
        -(1 as ::core::ffi::c_int),
        &raw mut msg as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_write_done>() as size_t,
    );
    if (*cf).cb.is_some() {
        (*cf).cb.expect("non-null function pointer")(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<evbuffer>(),
            (*cf).data,
        );
    }
    file_free(cf);
}
unsafe extern "C" fn file_write_error_callback(
    mut bev: *mut bufferevent,
    mut what: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    let mut error: ::core::ffi::c_int = 0;
    if what as ::core::ffi::c_int & EVBUFFER_ERROR != 0 {
        error = *__errno_location();
    } else {
        error = EIO;
    }
    if error == 0 as ::core::ffi::c_int {
        error = EIO;
    }
    log_debug(
        b"write error file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    (*cf).error = error;
    bufferevent_free((*cf).event);
    (*cf).event = ::core::ptr::null_mut::<bufferevent>();
    close((*cf).fd);
    (*cf).fd = -(1 as ::core::ffi::c_int);
    if (*cf).closed != 0 {
        file_write_finished(cf);
    } else if (*cf).cb.is_some() {
        (*cf).cb.expect("non-null function pointer")(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<evbuffer>(),
            (*cf).data,
        );
    }
}
unsafe extern "C" fn file_write_callback(
    mut bev: *mut bufferevent,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    log_debug(
        b"write check file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    if (*cf).closed != 0 && evbuffer_get_length((*(*cf).event).output) == 0 as size_t {
        file_write_finished(cf);
    } else if (*cf).cb.is_some() {
        (*cf).cb.expect("non-null function pointer")(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<evbuffer>(),
            (*cf).data,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn file_write_open(
    mut files: *mut client_files,
    mut peer: *mut tmuxpeer,
    mut imsg: *mut imsg,
    mut allow_streams: ::core::ffi::c_int,
    mut close_received: ::core::ffi::c_int,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) {
    let mut msg: *mut msg_write_open = (*imsg).data as *mut msg_write_open;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut reply: msg_write_ready = msg_write_ready {
        stream: 0,
        error: 0,
    };
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: C2RustUnnamed_12 {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let flags: ::core::ffi::c_int = O_NONBLOCK | O_WRONLY | O_CREAT;
    let mut error: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if msglen < ::core::mem::size_of::<msg_write_open>() as usize {
        fatalx(b"bad MSG_WRITE_OPEN size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if msglen == ::core::mem::size_of::<msg_write_open>() as usize {
        path = b"-\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        path = msg.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_char;
    }
    log_debug(
        b"open write file %d %s\0" as *const u8 as *const ::core::ffi::c_char,
        (*msg).stream,
        path,
    );
    find.stream = (*msg).stream;
    if !client_files_RB_FIND(files, &raw mut find).is_null() {
        error = EBADF;
    } else {
        cf = file_create_with_peer(peer, files, (*msg).stream, cb, cbdata);
        if (*cf).closed != 0 {
            error = EBADF;
        } else {
            (*cf).fd = -(1 as ::core::ffi::c_int);
            if (*msg).fd == -(1 as ::core::ffi::c_int) {
                (*cf).fd = open(path, (*msg).flags | flags, 0o644 as ::core::ffi::c_int);
            } else if allow_streams != 0 {
                if (*msg).fd != STDOUT_FILENO && (*msg).fd != STDERR_FILENO {
                    *__errno_location() = EBADF;
                } else {
                    (*cf).fd = dup((*msg).fd);
                    if close_received != 0 {
                        close((*msg).fd);
                    }
                }
            } else {
                *__errno_location() = EBADF;
            }
            if (*cf).fd == -(1 as ::core::ffi::c_int) {
                error = *__errno_location();
            } else {
                (*cf).event = bufferevent_new(
                    (*cf).fd,
                    None,
                    Some(
                        file_write_callback
                            as unsafe extern "C" fn(
                                *mut bufferevent,
                                *mut ::core::ffi::c_void,
                            ) -> (),
                    ),
                    Some(
                        file_write_error_callback
                            as unsafe extern "C" fn(
                                *mut bufferevent,
                                ::core::ffi::c_short,
                                *mut ::core::ffi::c_void,
                            ) -> (),
                    ),
                    cf as *mut ::core::ffi::c_void,
                );
                if (*cf).event.is_null() {
                    fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
                }
                bufferevent_enable((*cf).event, EV_WRITE as ::core::ffi::c_short);
            }
        }
    }
    reply.stream = (*msg).stream;
    reply.error = error;
    proc_send(
        peer,
        MSG_WRITE_READY,
        -(1 as ::core::ffi::c_int),
        &raw mut reply as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_write_ready>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn file_write_data(mut files: *mut client_files, mut imsg: *mut imsg) {
    let mut msg: *mut msg_write_data = (*imsg).data as *mut msg_write_data;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: C2RustUnnamed_12 {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut size: size_t = msglen.wrapping_sub(::core::mem::size_of::<msg_write_data>() as size_t);
    if msglen < ::core::mem::size_of::<msg_write_data>() as usize {
        fatalx(b"bad MSG_WRITE size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    find.stream = (*msg).stream;
    cf = client_files_RB_FIND(files, &raw mut find);
    if cf.is_null() {
        fatalx(b"unknown stream number\0" as *const u8 as *const ::core::ffi::c_char);
    }
    log_debug(
        b"write %zu to file %d\0" as *const u8 as *const ::core::ffi::c_char,
        size,
        (*cf).stream,
    );
    if !(*cf).event.is_null() {
        bufferevent_write(
            (*cf).event,
            msg.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            size,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn file_write_close(mut files: *mut client_files, mut imsg: *mut imsg) {
    let mut msg: *mut msg_write_close = (*imsg).data as *mut msg_write_close;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: C2RustUnnamed_12 {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if msglen != ::core::mem::size_of::<msg_write_close>() as usize {
        fatalx(b"bad MSG_WRITE_CLOSE size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    find.stream = (*msg).stream;
    cf = client_files_RB_FIND(files, &raw mut find);
    if cf.is_null() {
        fatalx(b"unknown stream number\0" as *const u8 as *const ::core::ffi::c_char);
    }
    log_debug(
        b"close file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    (*cf).closed = 1 as ::core::ffi::c_int;
    if (*cf).event.is_null() || evbuffer_get_length((*(*cf).event).output) == 0 as size_t {
        file_write_finished(cf);
    }
}
unsafe extern "C" fn file_read_error_callback(
    mut bev: *mut bufferevent,
    mut what: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    let mut msg: msg_read_done = msg_read_done {
        stream: 0,
        error: 0,
    };
    log_debug(
        b"read error file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    msg.stream = (*cf).stream;
    msg.error = if what as ::core::ffi::c_int & EVBUFFER_ERROR != 0 {
        EIO
    } else {
        0 as ::core::ffi::c_int
    };
    proc_send(
        (*cf).peer,
        MSG_READ_DONE,
        -(1 as ::core::ffi::c_int),
        &raw mut msg as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_read_done>() as size_t,
    );
    bufferevent_free((*cf).event);
    close((*cf).fd);
    client_files_RB_REMOVE((*cf).tree as *mut client_files, cf);
    file_free(cf);
}
unsafe extern "C" fn file_read_callback(
    mut bev: *mut bufferevent,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut cf: *mut client_file = arg as *mut client_file;
    let mut bdata: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut bsize: size_t = 0;
    let mut msg: *mut msg_read_data = ::core::ptr::null_mut::<msg_read_data>();
    let mut msglen: size_t = 0;
    msg = xmalloc(::core::mem::size_of::<msg_read_data>() as size_t) as *mut msg_read_data;
    loop {
        bdata = evbuffer_pullup((*(*cf).event).input, -(1 as ::core::ffi::c_int) as ssize_t)
            as *mut ::core::ffi::c_void;
        bsize = evbuffer_get_length((*(*cf).event).input);
        if bsize == 0 as size_t {
            break;
        }
        if bsize
            > (MAX_IMSGSIZE as usize)
                .wrapping_sub(IMSG_HEADER_SIZE)
                .wrapping_sub(::core::mem::size_of::<msg_read_data>() as usize)
        {
            bsize = (MAX_IMSGSIZE as usize)
                .wrapping_sub(IMSG_HEADER_SIZE)
                .wrapping_sub(::core::mem::size_of::<msg_read_data>() as usize)
                as size_t;
        }
        log_debug(
            b"read %zu from file %d\0" as *const u8 as *const ::core::ffi::c_char,
            bsize,
            (*cf).stream,
        );
        msglen = (::core::mem::size_of::<msg_read_data>() as usize).wrapping_add(bsize as usize)
            as size_t;
        msg = xrealloc(msg as *mut ::core::ffi::c_void, msglen) as *mut msg_read_data;
        (*msg).stream = (*cf).stream;
        memcpy(
            msg.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            bdata,
            bsize,
        );
        proc_send(
            (*cf).peer,
            MSG_READ,
            -(1 as ::core::ffi::c_int),
            msg as *const ::core::ffi::c_void,
            msglen,
        );
        evbuffer_drain((*(*cf).event).input, bsize);
    }
    free(msg as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn file_read_open(
    mut files: *mut client_files,
    mut peer: *mut tmuxpeer,
    mut imsg: *mut imsg,
    mut allow_streams: ::core::ffi::c_int,
    mut close_received: ::core::ffi::c_int,
    mut cb: client_file_cb,
    mut cbdata: *mut ::core::ffi::c_void,
) {
    let mut msg: *mut msg_read_open = (*imsg).data as *mut msg_read_open;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut reply: msg_read_done = msg_read_done {
        stream: 0,
        error: 0,
    };
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: C2RustUnnamed_12 {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let flags: ::core::ffi::c_int = O_NONBLOCK | O_RDONLY;
    let mut error: ::core::ffi::c_int = 0;
    if msglen < ::core::mem::size_of::<msg_read_open>() as usize {
        fatalx(b"bad MSG_READ_OPEN size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if msglen == ::core::mem::size_of::<msg_read_open>() as usize {
        path = b"-\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        path = msg.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_char;
    }
    log_debug(
        b"open read file %d %s\0" as *const u8 as *const ::core::ffi::c_char,
        (*msg).stream,
        path,
    );
    find.stream = (*msg).stream;
    if !client_files_RB_FIND(files, &raw mut find).is_null() {
        error = EBADF;
    } else {
        cf = file_create_with_peer(peer, files, (*msg).stream, cb, cbdata);
        if (*cf).closed != 0 {
            error = EBADF;
        } else {
            (*cf).fd = -(1 as ::core::ffi::c_int);
            if (*msg).fd == -(1 as ::core::ffi::c_int) {
                (*cf).fd = open(path, flags);
            } else if allow_streams != 0 {
                if (*msg).fd != STDIN_FILENO {
                    *__errno_location() = EBADF;
                } else {
                    (*cf).fd = dup((*msg).fd);
                    if close_received != 0 {
                        close((*msg).fd);
                    }
                }
            } else {
                *__errno_location() = EBADF;
            }
            if (*cf).fd == -(1 as ::core::ffi::c_int) {
                error = *__errno_location();
            } else {
                (*cf).event = bufferevent_new(
                    (*cf).fd,
                    Some(
                        file_read_callback
                            as unsafe extern "C" fn(
                                *mut bufferevent,
                                *mut ::core::ffi::c_void,
                            ) -> (),
                    ),
                    None,
                    Some(
                        file_read_error_callback
                            as unsafe extern "C" fn(
                                *mut bufferevent,
                                ::core::ffi::c_short,
                                *mut ::core::ffi::c_void,
                            ) -> (),
                    ),
                    cf as *mut ::core::ffi::c_void,
                );
                if (*cf).event.is_null() {
                    fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
                }
                bufferevent_enable((*cf).event, EV_READ as ::core::ffi::c_short);
                return;
            }
        }
    }
    reply.stream = (*msg).stream;
    reply.error = error;
    proc_send(
        peer,
        MSG_READ_DONE,
        -(1 as ::core::ffi::c_int),
        &raw mut reply as *const ::core::ffi::c_void,
        ::core::mem::size_of::<msg_read_done>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn file_read_cancel(mut files: *mut client_files, mut imsg: *mut imsg) {
    let mut msg: *mut msg_read_cancel = (*imsg).data as *mut msg_read_cancel;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: C2RustUnnamed_12 {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if msglen != ::core::mem::size_of::<msg_read_cancel>() as usize {
        fatalx(b"bad MSG_READ_CANCEL size\0" as *const u8 as *const ::core::ffi::c_char);
    }
    find.stream = (*msg).stream;
    cf = client_files_RB_FIND(files, &raw mut find);
    if cf.is_null() {
        fatalx(b"unknown stream number\0" as *const u8 as *const ::core::ffi::c_char);
    }
    log_debug(
        b"cancel file %d\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    file_read_error_callback(
        ::core::ptr::null_mut::<bufferevent>(),
        0 as ::core::ffi::c_short,
        cf as *mut ::core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn file_write_ready(
    mut files: *mut client_files,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut msg: *mut msg_write_ready = (*imsg).data as *mut msg_write_ready;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: C2RustUnnamed_12 {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if msglen != ::core::mem::size_of::<msg_write_ready>() as usize {
        return -(1 as ::core::ffi::c_int);
    }
    find.stream = (*msg).stream;
    cf = client_files_RB_FIND(files, &raw mut find);
    if cf.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*msg).error != 0 as ::core::ffi::c_int {
        (*cf).error = (*msg).error;
        file_fire_done(cf);
    } else {
        file_push(cf);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_write_done(
    mut files: *mut client_files,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut msg: *mut msg_write_done = (*imsg).data as *mut msg_write_done;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: C2RustUnnamed_12 {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if msglen != ::core::mem::size_of::<msg_write_done>() as usize {
        return -(1 as ::core::ffi::c_int);
    }
    find.stream = (*msg).stream;
    cf = client_files_RB_FIND(files, &raw mut find);
    if cf.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*cf).c.is_null() || !(*(*cf).c).flags as ::core::ffi::c_ulonglong & CLIENT_WRITE_ACK != 0 {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"file %d write done\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    (*cf).error = (*msg).error;
    file_fire_done(cf);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_read_data(
    mut files: *mut client_files,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut msg: *mut msg_read_data = (*imsg).data as *mut msg_read_data;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: C2RustUnnamed_12 {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut bdata: *mut ::core::ffi::c_void =
        msg.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void;
    let mut bsize: size_t = msglen.wrapping_sub(::core::mem::size_of::<msg_read_data>() as size_t);
    if msglen < ::core::mem::size_of::<msg_read_data>() as usize {
        return -(1 as ::core::ffi::c_int);
    }
    find.stream = (*msg).stream;
    cf = client_files_RB_FIND(files, &raw mut find);
    if cf.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"file %d read %zu bytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
        bsize,
    );
    if (*cf).error == 0 as ::core::ffi::c_int && (*cf).closed == 0 {
        if evbuffer_add((*cf).buffer, bdata, bsize) != 0 as ::core::ffi::c_int {
            (*cf).error = ENOMEM;
            file_fire_done(cf);
        } else {
            file_fire_read(cf);
        }
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn file_read_done(
    mut files: *mut client_files,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut msg: *mut msg_read_done = (*imsg).data as *mut msg_read_done;
    let mut msglen: size_t = ((*imsg).hdr.len as size_t).wrapping_sub(IMSG_HEADER_SIZE);
    let mut find: client_file = client_file {
        c: ::core::ptr::null_mut::<client>(),
        peer: ::core::ptr::null_mut::<tmuxpeer>(),
        tree: ::core::ptr::null_mut::<client_files>(),
        references: 0,
        stream: 0,
        path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        buffer: ::core::ptr::null_mut::<evbuffer>(),
        event: ::core::ptr::null_mut::<bufferevent>(),
        fd: 0,
        error: 0,
        closed: 0,
        cb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        entry: C2RustUnnamed_12 {
            rbe_left: ::core::ptr::null_mut::<client_file>(),
            rbe_right: ::core::ptr::null_mut::<client_file>(),
            rbe_parent: ::core::ptr::null_mut::<client_file>(),
            rbe_color: 0,
        },
    };
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if msglen != ::core::mem::size_of::<msg_read_done>() as usize {
        return -(1 as ::core::ffi::c_int);
    }
    find.stream = (*msg).stream;
    cf = client_files_RB_FIND(files, &raw mut find);
    if cf.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"file %d read done\0" as *const u8 as *const ::core::ffi::c_char,
        (*cf).stream,
    );
    (*cf).error = (*msg).error;
    file_fire_done(cf);
    return 0 as ::core::ffi::c_int;
}

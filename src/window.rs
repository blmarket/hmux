use crate::src::shared::abi::*;
use crate::src::shared::colour::*;
use crate::src::shared::grid::*;
use crate::src::shared::key::*;
use crate::src::shared::style::*;
use ::c2rust_bitfields;
extern "C" {
    pub type re_dfa_t;
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
    pub type event_payload;
    fn ioctl(__fd: ::core::ffi::c_int, __request: ::core::ffi::c_ulong, ...) -> ::core::ffi::c_int;
    fn kill(__pid: __pid_t, __sig: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn getpid() -> __pid_t;
    fn gethostname(__name: *mut ::core::ffi::c_char, __len: size_t) -> ::core::ffi::c_int;
    fn __ctype_b_loc() -> *mut *const ::core::ffi::c_ushort;
    fn fnmatch(
        __pattern: *const ::core::ffi::c_char,
        __name: *const ::core::ffi::c_char,
        __flags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn regcomp(
        __preg: *mut regex_t,
        __pattern: *const ::core::ffi::c_char,
        __cflags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn regexec(
        __preg: *const regex_t,
        __String: *const ::core::ffi::c_char,
        __nmatch: size_t,
        __pmatch: *mut regmatch_t,
        __eflags: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn regfree(__preg: *mut regex_t);
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
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn gettimeofday(__tv: *mut timeval, __tz: *mut ::core::ffi::c_void) -> ::core::ffi::c_int;
    fn utempter_remove_record(master_fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn event_add(ev: *mut event, timeout: *const timeval) -> ::core::ffi::c_int;
    fn event_del(_: *mut event) -> ::core::ffi::c_int;
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
    fn strtonum(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
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
    static mut global_options: *mut options;
    static mut global_w_options: *mut options;
    fn setblocking(_: ::core::ffi::c_int, _: ::core::ffi::c_int);
    fn clean_name(_: *const ::core::ffi::c_char, _: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
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
    fn event_payload_set_uint(_: *mut event_payload, _: *const ::core::ffi::c_char, _: u_int);
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
    fn events_fire_pane(_: *const ::core::ffi::c_char, _: *mut window_pane);
    fn options_create(_: *mut options) -> *mut options;
    fn options_free(_: *mut options);
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn tty_update_window_offset(_: *mut window);
    fn tty_default_colours(_: *mut grid_cell, _: *mut window_pane, _: *mut u_int);
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn cmd_find_from_window(
        _: *mut cmd_find_state,
        _: *mut window,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_find_from_pane(
        _: *mut cmd_find_state,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_free_argv(_: ::core::ffi::c_int, _: *mut *mut ::core::ffi::c_char);
    fn cmd_mouse_at(
        _: *mut window_pane,
        _: *mut mouse_event,
        _: *mut u_int,
        _: *mut u_int,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_continue(_: *mut cmdq_item);
    fn alerts_queue(_: *mut window, _: ::core::ffi::c_int);
    fn file_read(
        _: *mut client,
        _: *const ::core::ffi::c_char,
        _: client_file_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut client_file;
    fn file_cancel(_: *mut client_file);
    static mut clients: clients;
    static mut marked_pane: cmd_find_state;
    fn server_clear_marked();
    fn server_check_marked() -> ::core::ffi::c_int;
    fn server_client_unref(_: *mut client);
    fn server_status_session(_: *mut session);
    fn server_redraw_window(_: *mut window);
    fn server_redraw_window_borders(_: *mut window);
    fn server_status_window(_: *mut window);
    fn server_kill_pane(_: *mut window_pane);
    fn server_destroy_pane(_: *mut window_pane, _: ::core::ffi::c_int);
    fn status_at_line(_: *mut client) -> ::core::ffi::c_int;
    fn prompt_set_options(_: *mut prompt_create_data, _: *mut session);
    fn prompt_create(_: *const prompt_create_data) -> *mut prompt;
    fn prompt_free(_: *mut prompt);
    fn prompt_incremental_start(_: *mut prompt);
    fn prompt_key(_: *mut prompt, _: key_code, _: *mut ::core::ffi::c_int) -> prompt_key_result;
    fn prompt_mouse(
        _: *mut prompt,
        _: u_int,
        _: u_int,
        _: u_int,
        _: *mut ::core::ffi::c_int,
    ) -> prompt_key_result;
    fn prompt_update(_: *mut prompt, _: *const ::core::ffi::c_char, _: *const ::core::ffi::c_char);
    fn prompt_closed(_: *mut prompt) -> ::core::ffi::c_int;
    fn prompt_type_string(_: prompt_type) -> *const ::core::ffi::c_char;
    fn input_init(
        _: *mut window_pane,
        _: *mut bufferevent,
        _: *mut colour_palette,
        _: *mut client,
    ) -> *mut input_ctx;
    fn input_free(_: *mut input_ctx);
    fn input_parse_pane(_: *mut window_pane);
    fn input_parse_buffer(_: *mut window_pane, _: *const u_char, _: size_t);
    fn input_key_pane(_: *mut window_pane, _: key_code, _: *mut mouse_event) -> ::core::ffi::c_int;
    fn colour_totheme(_: ::core::ffi::c_int) -> client_theme;
    fn colour_palette_init(_: *mut colour_palette);
    fn colour_palette_free(_: *mut colour_palette);
    fn colour_palette_get(_: *mut colour_palette, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn colour_palette_from_option(_: *mut colour_palette, _: *mut options);
    fn grid_cells_look_equal(_: *const grid_cell, _: *const grid_cell) -> ::core::ffi::c_int;
    fn grid_view_string_cells(
        _: *mut grid,
        _: u_int,
        _: u_int,
        _: u_int,
    ) -> *mut ::core::ffi::c_char;
    fn screen_write_stop_sync(_: *mut window_pane);
    fn screen_write_clear_dirty(_: *mut window_pane);
    fn redraw_invalidate_scene(_: *mut window);
    fn screen_init(_: *mut screen, _: u_int, _: u_int, _: u_int);
    fn screen_free(_: *mut screen);
    fn screen_set_default_cursor(_: *mut screen, _: *mut options);
    fn screen_set_title(
        _: *mut screen,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn screen_resize(_: *mut screen, _: u_int, _: u_int, _: ::core::ffi::c_int);
    fn layout_free_cell(_: *mut layout_cell, _: ::core::ffi::c_int);
    fn layout_fix_panes(_: *mut window, _: *mut window_pane);
    fn layout_init(_: *mut window, _: *mut window_pane);
    fn layout_free(_: *mut window, _: ::core::ffi::c_int);
    fn layout_assign_pane(_: *mut layout_cell, _: *mut window_pane, _: ::core::ffi::c_int);
    fn layout_floating_pane(
        _: *mut window,
        _: *mut window_pane,
        _: *mut layout_geometry,
    ) -> *mut layout_cell;
    static window_copy_mode: window_mode;
    static window_view_mode: window_mode;
    fn control_write_output(_: *mut client, _: *mut window_pane);
    fn session_has(_: *mut session, _: *mut window) -> ::core::ffi::c_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatal(_: *const ::core::ffi::c_char, ...) -> !;
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
    fn menu_destroy(_: *mut window);
    fn menu_resize(_: *mut menu_data, _: *mut window);
    fn style_set_scrollbar_style_from_option(_: *mut style, _: *mut options);
    fn style_ranges_init(_: *mut style_ranges);
    fn style_ranges_free(_: *mut style_ranges);
    fn style_ranges_get_range(_: *mut style_ranges, _: u_int) -> *mut style_range;
    fn spawn_editor_finish(_: *mut window_pane);
}
pub type ssize_t = isize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct winsize {
    pub ws_row: ::core::ffi::c_ushort,
    pub ws_col: ::core::ffi::c_ushort,
    pub ws_xpixel: ::core::ffi::c_ushort,
    pub ws_ypixel: ::core::ffi::c_ushort,
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
pub type __re_long_size_t = ::core::ffi::c_ulong;
pub type reg_syntax_t = ::core::ffi::c_ulong;
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct re_pattern_buffer {
    pub buffer: *mut re_dfa_t,
    pub allocated: __re_long_size_t,
    pub used: __re_long_size_t,
    pub syntax: reg_syntax_t,
    pub fastmap: *mut ::core::ffi::c_char,
    pub translate: *mut ::core::ffi::c_uchar,
    pub re_nsub: size_t,
    #[bitfield(name = "can_be_null", ty = "::core::ffi::c_uint", bits = "0..=0")]
    #[bitfield(name = "regs_allocated", ty = "::core::ffi::c_uint", bits = "1..=2")]
    #[bitfield(name = "fastmap_accurate", ty = "::core::ffi::c_uint", bits = "3..=3")]
    #[bitfield(name = "no_sub", ty = "::core::ffi::c_uint", bits = "4..=4")]
    #[bitfield(name = "not_bol", ty = "::core::ffi::c_uint", bits = "5..=5")]
    #[bitfield(name = "not_eol", ty = "::core::ffi::c_uint", bits = "6..=6")]
    #[bitfield(name = "newline_anchor", ty = "::core::ffi::c_uint", bits = "7..=7")]
    pub can_be_null_regs_allocated_fastmap_accurate_no_sub_not_bol_not_eol_newline_anchor: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 7],
}
pub type regex_t = re_pattern_buffer;
pub type regoff_t = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct regmatch_t {
    pub rm_so: regoff_t,
    pub rm_eo: regoff_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct termios {
    pub c_iflag: tcflag_t,
    pub c_oflag: tcflag_t,
    pub c_cflag: tcflag_t,
    pub c_lflag: tcflag_t,
    pub c_line: cc_t,
    pub c_cc: [cc_t; 32],
    pub c2rust_unnamed: C2RustUnnamed_1,
    pub c2rust_unnamed_0: C2RustUnnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_0 {
    pub __ospeed: speed_t,
    pub c_ospeed: speed_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_1 {
    pub __ispeed: speed_t,
    pub c_ispeed: speed_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct event {
    pub ev_evcallback: event_callback,
    pub ev_timeout_pos: C2RustUnnamed_7,
    pub ev_fd: ::core::ffi::c_int,
    pub ev_base: *mut event_base,
    pub ev_: C2RustUnnamed_2,
    pub ev_events: ::core::ffi::c_short,
    pub ev_res: ::core::ffi::c_short,
    pub ev_timeout: timeval,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_2 {
    pub ev_io: C2RustUnnamed_5,
    pub ev_signal: C2RustUnnamed_3,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_3 {
    pub ev_signal_next: C2RustUnnamed_4,
    pub ev_ncalls: ::core::ffi::c_short,
    pub ev_pncalls: *mut ::core::ffi::c_short,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_4 {
    pub le_next: *mut event,
    pub le_prev: *mut *mut event,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_5 {
    pub ev_io_next: C2RustUnnamed_6,
    pub ev_timeout: timeval,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_6 {
    pub le_next: *mut event,
    pub le_prev: *mut *mut event,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_7 {
    pub ev_next_with_common_timeout: C2RustUnnamed_8,
    pub min_heap_idx: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_8 {
    pub tqe_next: *mut event,
    pub tqe_prev: *mut *mut event,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_callback {
    pub evcb_active_next: C2RustUnnamed_10,
    pub evcb_flags: ::core::ffi::c_short,
    pub evcb_pri: uint8_t,
    pub evcb_closure: uint8_t,
    pub evcb_cb_union: C2RustUnnamed_9,
    pub evcb_arg: *mut ::core::ffi::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_9 {
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
pub struct C2RustUnnamed_10 {
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
pub struct window_pane_prompt {
    pub wp_id: u_int,
    pub c: *mut client,
    pub inputcb: status_prompt_input_cb,
    pub freecb: prompt_free_cb,
    pub data: *mut ::core::ffi::c_void,
    pub type_0: prompt_type,
}
pub type prompt_type = ::core::ffi::c_uint;
pub const PROMPT_TYPE_INVALID: prompt_type = 255;
pub const PROMPT_TYPE_SEARCH: prompt_type = 1;
pub const PROMPT_TYPE_COMMAND: prompt_type = 0;
pub type prompt_free_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type status_prompt_input_cb = Option<
    unsafe extern "C" fn(
        *mut client,
        *mut ::core::ffi::c_void,
        *const ::core::ffi::c_char,
        prompt_key_result,
    ) -> prompt_result,
>;
pub type prompt_key_result = ::core::ffi::c_uint;
pub const PROMPT_KEY_MOVE: prompt_key_result = 3;
pub const PROMPT_KEY_CLOSE: prompt_key_result = 2;
pub const PROMPT_KEY_HANDLED: prompt_key_result = 1;
pub const PROMPT_KEY_NOT_HANDLED: prompt_key_result = 0;
pub type prompt_result = ::core::ffi::c_uint;
pub const PROMPT_CLOSE: prompt_result = 1;
pub const PROMPT_CONTINUE: prompt_result = 0;
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
pub type C2RustUnnamed_36 = ::core::ffi::c_ulong;
pub const KEYC_TRIPLECLICK11_CONTROL9: C2RustUnnamed_36 = 51539610387;
pub const KEYC_TRIPLECLICK10_CONTROL9: C2RustUnnamed_36 = 51539610131;
pub const KEYC_TRIPLECLICK9_CONTROL9: C2RustUnnamed_36 = 51539609875;
pub const KEYC_TRIPLECLICK8_CONTROL9: C2RustUnnamed_36 = 51539609619;
pub const KEYC_TRIPLECLICK7_CONTROL9: C2RustUnnamed_36 = 51539609363;
pub const KEYC_TRIPLECLICK6_CONTROL9: C2RustUnnamed_36 = 51539609107;
pub const KEYC_TRIPLECLICK3_CONTROL9: C2RustUnnamed_36 = 51539608339;
pub const KEYC_TRIPLECLICK2_CONTROL9: C2RustUnnamed_36 = 51539608083;
pub const KEYC_TRIPLECLICK1_CONTROL9: C2RustUnnamed_36 = 51539607827;
pub const KEYC_TRIPLECLICK_CONTROL9: C2RustUnnamed_36 = 51539607571;
pub const KEYC_TRIPLECLICK11_CONTROL8: C2RustUnnamed_36 = 51539610386;
pub const KEYC_TRIPLECLICK10_CONTROL8: C2RustUnnamed_36 = 51539610130;
pub const KEYC_TRIPLECLICK9_CONTROL8: C2RustUnnamed_36 = 51539609874;
pub const KEYC_TRIPLECLICK8_CONTROL8: C2RustUnnamed_36 = 51539609618;
pub const KEYC_TRIPLECLICK7_CONTROL8: C2RustUnnamed_36 = 51539609362;
pub const KEYC_TRIPLECLICK6_CONTROL8: C2RustUnnamed_36 = 51539609106;
pub const KEYC_TRIPLECLICK3_CONTROL8: C2RustUnnamed_36 = 51539608338;
pub const KEYC_TRIPLECLICK2_CONTROL8: C2RustUnnamed_36 = 51539608082;
pub const KEYC_TRIPLECLICK1_CONTROL8: C2RustUnnamed_36 = 51539607826;
pub const KEYC_TRIPLECLICK_CONTROL8: C2RustUnnamed_36 = 51539607570;
pub const KEYC_TRIPLECLICK11_CONTROL7: C2RustUnnamed_36 = 51539610385;
pub const KEYC_TRIPLECLICK10_CONTROL7: C2RustUnnamed_36 = 51539610129;
pub const KEYC_TRIPLECLICK9_CONTROL7: C2RustUnnamed_36 = 51539609873;
pub const KEYC_TRIPLECLICK8_CONTROL7: C2RustUnnamed_36 = 51539609617;
pub const KEYC_TRIPLECLICK7_CONTROL7: C2RustUnnamed_36 = 51539609361;
pub const KEYC_TRIPLECLICK6_CONTROL7: C2RustUnnamed_36 = 51539609105;
pub const KEYC_TRIPLECLICK3_CONTROL7: C2RustUnnamed_36 = 51539608337;
pub const KEYC_TRIPLECLICK2_CONTROL7: C2RustUnnamed_36 = 51539608081;
pub const KEYC_TRIPLECLICK1_CONTROL7: C2RustUnnamed_36 = 51539607825;
pub const KEYC_TRIPLECLICK_CONTROL7: C2RustUnnamed_36 = 51539607569;
pub const KEYC_TRIPLECLICK11_CONTROL6: C2RustUnnamed_36 = 51539610384;
pub const KEYC_TRIPLECLICK10_CONTROL6: C2RustUnnamed_36 = 51539610128;
pub const KEYC_TRIPLECLICK9_CONTROL6: C2RustUnnamed_36 = 51539609872;
pub const KEYC_TRIPLECLICK8_CONTROL6: C2RustUnnamed_36 = 51539609616;
pub const KEYC_TRIPLECLICK7_CONTROL6: C2RustUnnamed_36 = 51539609360;
pub const KEYC_TRIPLECLICK6_CONTROL6: C2RustUnnamed_36 = 51539609104;
pub const KEYC_TRIPLECLICK3_CONTROL6: C2RustUnnamed_36 = 51539608336;
pub const KEYC_TRIPLECLICK2_CONTROL6: C2RustUnnamed_36 = 51539608080;
pub const KEYC_TRIPLECLICK1_CONTROL6: C2RustUnnamed_36 = 51539607824;
pub const KEYC_TRIPLECLICK_CONTROL6: C2RustUnnamed_36 = 51539607568;
pub const KEYC_TRIPLECLICK11_CONTROL5: C2RustUnnamed_36 = 51539610383;
pub const KEYC_TRIPLECLICK10_CONTROL5: C2RustUnnamed_36 = 51539610127;
pub const KEYC_TRIPLECLICK9_CONTROL5: C2RustUnnamed_36 = 51539609871;
pub const KEYC_TRIPLECLICK8_CONTROL5: C2RustUnnamed_36 = 51539609615;
pub const KEYC_TRIPLECLICK7_CONTROL5: C2RustUnnamed_36 = 51539609359;
pub const KEYC_TRIPLECLICK6_CONTROL5: C2RustUnnamed_36 = 51539609103;
pub const KEYC_TRIPLECLICK3_CONTROL5: C2RustUnnamed_36 = 51539608335;
pub const KEYC_TRIPLECLICK2_CONTROL5: C2RustUnnamed_36 = 51539608079;
pub const KEYC_TRIPLECLICK1_CONTROL5: C2RustUnnamed_36 = 51539607823;
pub const KEYC_TRIPLECLICK_CONTROL5: C2RustUnnamed_36 = 51539607567;
pub const KEYC_TRIPLECLICK11_CONTROL4: C2RustUnnamed_36 = 51539610382;
pub const KEYC_TRIPLECLICK10_CONTROL4: C2RustUnnamed_36 = 51539610126;
pub const KEYC_TRIPLECLICK9_CONTROL4: C2RustUnnamed_36 = 51539609870;
pub const KEYC_TRIPLECLICK8_CONTROL4: C2RustUnnamed_36 = 51539609614;
pub const KEYC_TRIPLECLICK7_CONTROL4: C2RustUnnamed_36 = 51539609358;
pub const KEYC_TRIPLECLICK6_CONTROL4: C2RustUnnamed_36 = 51539609102;
pub const KEYC_TRIPLECLICK3_CONTROL4: C2RustUnnamed_36 = 51539608334;
pub const KEYC_TRIPLECLICK2_CONTROL4: C2RustUnnamed_36 = 51539608078;
pub const KEYC_TRIPLECLICK1_CONTROL4: C2RustUnnamed_36 = 51539607822;
pub const KEYC_TRIPLECLICK_CONTROL4: C2RustUnnamed_36 = 51539607566;
pub const KEYC_TRIPLECLICK11_CONTROL3: C2RustUnnamed_36 = 51539610381;
pub const KEYC_TRIPLECLICK10_CONTROL3: C2RustUnnamed_36 = 51539610125;
pub const KEYC_TRIPLECLICK9_CONTROL3: C2RustUnnamed_36 = 51539609869;
pub const KEYC_TRIPLECLICK8_CONTROL3: C2RustUnnamed_36 = 51539609613;
pub const KEYC_TRIPLECLICK7_CONTROL3: C2RustUnnamed_36 = 51539609357;
pub const KEYC_TRIPLECLICK6_CONTROL3: C2RustUnnamed_36 = 51539609101;
pub const KEYC_TRIPLECLICK3_CONTROL3: C2RustUnnamed_36 = 51539608333;
pub const KEYC_TRIPLECLICK2_CONTROL3: C2RustUnnamed_36 = 51539608077;
pub const KEYC_TRIPLECLICK1_CONTROL3: C2RustUnnamed_36 = 51539607821;
pub const KEYC_TRIPLECLICK_CONTROL3: C2RustUnnamed_36 = 51539607565;
pub const KEYC_TRIPLECLICK11_CONTROL2: C2RustUnnamed_36 = 51539610380;
pub const KEYC_TRIPLECLICK10_CONTROL2: C2RustUnnamed_36 = 51539610124;
pub const KEYC_TRIPLECLICK9_CONTROL2: C2RustUnnamed_36 = 51539609868;
pub const KEYC_TRIPLECLICK8_CONTROL2: C2RustUnnamed_36 = 51539609612;
pub const KEYC_TRIPLECLICK7_CONTROL2: C2RustUnnamed_36 = 51539609356;
pub const KEYC_TRIPLECLICK6_CONTROL2: C2RustUnnamed_36 = 51539609100;
pub const KEYC_TRIPLECLICK3_CONTROL2: C2RustUnnamed_36 = 51539608332;
pub const KEYC_TRIPLECLICK2_CONTROL2: C2RustUnnamed_36 = 51539608076;
pub const KEYC_TRIPLECLICK1_CONTROL2: C2RustUnnamed_36 = 51539607820;
pub const KEYC_TRIPLECLICK_CONTROL2: C2RustUnnamed_36 = 51539607564;
pub const KEYC_TRIPLECLICK11_CONTROL1: C2RustUnnamed_36 = 51539610379;
pub const KEYC_TRIPLECLICK10_CONTROL1: C2RustUnnamed_36 = 51539610123;
pub const KEYC_TRIPLECLICK9_CONTROL1: C2RustUnnamed_36 = 51539609867;
pub const KEYC_TRIPLECLICK8_CONTROL1: C2RustUnnamed_36 = 51539609611;
pub const KEYC_TRIPLECLICK7_CONTROL1: C2RustUnnamed_36 = 51539609355;
pub const KEYC_TRIPLECLICK6_CONTROL1: C2RustUnnamed_36 = 51539609099;
pub const KEYC_TRIPLECLICK3_CONTROL1: C2RustUnnamed_36 = 51539608331;
pub const KEYC_TRIPLECLICK2_CONTROL1: C2RustUnnamed_36 = 51539608075;
pub const KEYC_TRIPLECLICK1_CONTROL1: C2RustUnnamed_36 = 51539607819;
pub const KEYC_TRIPLECLICK_CONTROL1: C2RustUnnamed_36 = 51539607563;
pub const KEYC_TRIPLECLICK11_CONTROL0: C2RustUnnamed_36 = 51539610378;
pub const KEYC_TRIPLECLICK10_CONTROL0: C2RustUnnamed_36 = 51539610122;
pub const KEYC_TRIPLECLICK9_CONTROL0: C2RustUnnamed_36 = 51539609866;
pub const KEYC_TRIPLECLICK8_CONTROL0: C2RustUnnamed_36 = 51539609610;
pub const KEYC_TRIPLECLICK7_CONTROL0: C2RustUnnamed_36 = 51539609354;
pub const KEYC_TRIPLECLICK6_CONTROL0: C2RustUnnamed_36 = 51539609098;
pub const KEYC_TRIPLECLICK3_CONTROL0: C2RustUnnamed_36 = 51539608330;
pub const KEYC_TRIPLECLICK2_CONTROL0: C2RustUnnamed_36 = 51539608074;
pub const KEYC_TRIPLECLICK1_CONTROL0: C2RustUnnamed_36 = 51539607818;
pub const KEYC_TRIPLECLICK_CONTROL0: C2RustUnnamed_36 = 51539607562;
pub const KEYC_TRIPLECLICK11_EMPTY: C2RustUnnamed_36 = 51539610377;
pub const KEYC_TRIPLECLICK10_EMPTY: C2RustUnnamed_36 = 51539610121;
pub const KEYC_TRIPLECLICK9_EMPTY: C2RustUnnamed_36 = 51539609865;
pub const KEYC_TRIPLECLICK8_EMPTY: C2RustUnnamed_36 = 51539609609;
pub const KEYC_TRIPLECLICK7_EMPTY: C2RustUnnamed_36 = 51539609353;
pub const KEYC_TRIPLECLICK6_EMPTY: C2RustUnnamed_36 = 51539609097;
pub const KEYC_TRIPLECLICK3_EMPTY: C2RustUnnamed_36 = 51539608329;
pub const KEYC_TRIPLECLICK2_EMPTY: C2RustUnnamed_36 = 51539608073;
pub const KEYC_TRIPLECLICK1_EMPTY: C2RustUnnamed_36 = 51539607817;
pub const KEYC_TRIPLECLICK_EMPTY: C2RustUnnamed_36 = 51539607561;
pub const KEYC_TRIPLECLICK11_SCROLLBAR_DOWN: C2RustUnnamed_36 = 51539610376;
pub const KEYC_TRIPLECLICK10_SCROLLBAR_DOWN: C2RustUnnamed_36 = 51539610120;
pub const KEYC_TRIPLECLICK9_SCROLLBAR_DOWN: C2RustUnnamed_36 = 51539609864;
pub const KEYC_TRIPLECLICK8_SCROLLBAR_DOWN: C2RustUnnamed_36 = 51539609608;
pub const KEYC_TRIPLECLICK7_SCROLLBAR_DOWN: C2RustUnnamed_36 = 51539609352;
pub const KEYC_TRIPLECLICK6_SCROLLBAR_DOWN: C2RustUnnamed_36 = 51539609096;
pub const KEYC_TRIPLECLICK3_SCROLLBAR_DOWN: C2RustUnnamed_36 = 51539608328;
pub const KEYC_TRIPLECLICK2_SCROLLBAR_DOWN: C2RustUnnamed_36 = 51539608072;
pub const KEYC_TRIPLECLICK1_SCROLLBAR_DOWN: C2RustUnnamed_36 = 51539607816;
pub const KEYC_TRIPLECLICK_SCROLLBAR_DOWN: C2RustUnnamed_36 = 51539607560;
pub const KEYC_TRIPLECLICK11_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 51539610375;
pub const KEYC_TRIPLECLICK10_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 51539610119;
pub const KEYC_TRIPLECLICK9_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 51539609863;
pub const KEYC_TRIPLECLICK8_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 51539609607;
pub const KEYC_TRIPLECLICK7_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 51539609351;
pub const KEYC_TRIPLECLICK6_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 51539609095;
pub const KEYC_TRIPLECLICK3_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 51539608327;
pub const KEYC_TRIPLECLICK2_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 51539608071;
pub const KEYC_TRIPLECLICK1_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 51539607815;
pub const KEYC_TRIPLECLICK_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 51539607559;
pub const KEYC_TRIPLECLICK11_SCROLLBAR_UP: C2RustUnnamed_36 = 51539610374;
pub const KEYC_TRIPLECLICK10_SCROLLBAR_UP: C2RustUnnamed_36 = 51539610118;
pub const KEYC_TRIPLECLICK9_SCROLLBAR_UP: C2RustUnnamed_36 = 51539609862;
pub const KEYC_TRIPLECLICK8_SCROLLBAR_UP: C2RustUnnamed_36 = 51539609606;
pub const KEYC_TRIPLECLICK7_SCROLLBAR_UP: C2RustUnnamed_36 = 51539609350;
pub const KEYC_TRIPLECLICK6_SCROLLBAR_UP: C2RustUnnamed_36 = 51539609094;
pub const KEYC_TRIPLECLICK3_SCROLLBAR_UP: C2RustUnnamed_36 = 51539608326;
pub const KEYC_TRIPLECLICK2_SCROLLBAR_UP: C2RustUnnamed_36 = 51539608070;
pub const KEYC_TRIPLECLICK1_SCROLLBAR_UP: C2RustUnnamed_36 = 51539607814;
pub const KEYC_TRIPLECLICK_SCROLLBAR_UP: C2RustUnnamed_36 = 51539607558;
pub const KEYC_TRIPLECLICK11_BORDER: C2RustUnnamed_36 = 51539610373;
pub const KEYC_TRIPLECLICK10_BORDER: C2RustUnnamed_36 = 51539610117;
pub const KEYC_TRIPLECLICK9_BORDER: C2RustUnnamed_36 = 51539609861;
pub const KEYC_TRIPLECLICK8_BORDER: C2RustUnnamed_36 = 51539609605;
pub const KEYC_TRIPLECLICK7_BORDER: C2RustUnnamed_36 = 51539609349;
pub const KEYC_TRIPLECLICK6_BORDER: C2RustUnnamed_36 = 51539609093;
pub const KEYC_TRIPLECLICK3_BORDER: C2RustUnnamed_36 = 51539608325;
pub const KEYC_TRIPLECLICK2_BORDER: C2RustUnnamed_36 = 51539608069;
pub const KEYC_TRIPLECLICK1_BORDER: C2RustUnnamed_36 = 51539607813;
pub const KEYC_TRIPLECLICK_BORDER: C2RustUnnamed_36 = 51539607557;
pub const KEYC_TRIPLECLICK11_STATUS_DEFAULT: C2RustUnnamed_36 = 51539610372;
pub const KEYC_TRIPLECLICK10_STATUS_DEFAULT: C2RustUnnamed_36 = 51539610116;
pub const KEYC_TRIPLECLICK9_STATUS_DEFAULT: C2RustUnnamed_36 = 51539609860;
pub const KEYC_TRIPLECLICK8_STATUS_DEFAULT: C2RustUnnamed_36 = 51539609604;
pub const KEYC_TRIPLECLICK7_STATUS_DEFAULT: C2RustUnnamed_36 = 51539609348;
pub const KEYC_TRIPLECLICK6_STATUS_DEFAULT: C2RustUnnamed_36 = 51539609092;
pub const KEYC_TRIPLECLICK3_STATUS_DEFAULT: C2RustUnnamed_36 = 51539608324;
pub const KEYC_TRIPLECLICK2_STATUS_DEFAULT: C2RustUnnamed_36 = 51539608068;
pub const KEYC_TRIPLECLICK1_STATUS_DEFAULT: C2RustUnnamed_36 = 51539607812;
pub const KEYC_TRIPLECLICK_STATUS_DEFAULT: C2RustUnnamed_36 = 51539607556;
pub const KEYC_TRIPLECLICK11_STATUS_RIGHT: C2RustUnnamed_36 = 51539610371;
pub const KEYC_TRIPLECLICK10_STATUS_RIGHT: C2RustUnnamed_36 = 51539610115;
pub const KEYC_TRIPLECLICK9_STATUS_RIGHT: C2RustUnnamed_36 = 51539609859;
pub const KEYC_TRIPLECLICK8_STATUS_RIGHT: C2RustUnnamed_36 = 51539609603;
pub const KEYC_TRIPLECLICK7_STATUS_RIGHT: C2RustUnnamed_36 = 51539609347;
pub const KEYC_TRIPLECLICK6_STATUS_RIGHT: C2RustUnnamed_36 = 51539609091;
pub const KEYC_TRIPLECLICK3_STATUS_RIGHT: C2RustUnnamed_36 = 51539608323;
pub const KEYC_TRIPLECLICK2_STATUS_RIGHT: C2RustUnnamed_36 = 51539608067;
pub const KEYC_TRIPLECLICK1_STATUS_RIGHT: C2RustUnnamed_36 = 51539607811;
pub const KEYC_TRIPLECLICK_STATUS_RIGHT: C2RustUnnamed_36 = 51539607555;
pub const KEYC_TRIPLECLICK11_STATUS_LEFT: C2RustUnnamed_36 = 51539610370;
pub const KEYC_TRIPLECLICK10_STATUS_LEFT: C2RustUnnamed_36 = 51539610114;
pub const KEYC_TRIPLECLICK9_STATUS_LEFT: C2RustUnnamed_36 = 51539609858;
pub const KEYC_TRIPLECLICK8_STATUS_LEFT: C2RustUnnamed_36 = 51539609602;
pub const KEYC_TRIPLECLICK7_STATUS_LEFT: C2RustUnnamed_36 = 51539609346;
pub const KEYC_TRIPLECLICK6_STATUS_LEFT: C2RustUnnamed_36 = 51539609090;
pub const KEYC_TRIPLECLICK3_STATUS_LEFT: C2RustUnnamed_36 = 51539608322;
pub const KEYC_TRIPLECLICK2_STATUS_LEFT: C2RustUnnamed_36 = 51539608066;
pub const KEYC_TRIPLECLICK1_STATUS_LEFT: C2RustUnnamed_36 = 51539607810;
pub const KEYC_TRIPLECLICK_STATUS_LEFT: C2RustUnnamed_36 = 51539607554;
pub const KEYC_TRIPLECLICK11_STATUS: C2RustUnnamed_36 = 51539610369;
pub const KEYC_TRIPLECLICK10_STATUS: C2RustUnnamed_36 = 51539610113;
pub const KEYC_TRIPLECLICK9_STATUS: C2RustUnnamed_36 = 51539609857;
pub const KEYC_TRIPLECLICK8_STATUS: C2RustUnnamed_36 = 51539609601;
pub const KEYC_TRIPLECLICK7_STATUS: C2RustUnnamed_36 = 51539609345;
pub const KEYC_TRIPLECLICK6_STATUS: C2RustUnnamed_36 = 51539609089;
pub const KEYC_TRIPLECLICK3_STATUS: C2RustUnnamed_36 = 51539608321;
pub const KEYC_TRIPLECLICK2_STATUS: C2RustUnnamed_36 = 51539608065;
pub const KEYC_TRIPLECLICK1_STATUS: C2RustUnnamed_36 = 51539607809;
pub const KEYC_TRIPLECLICK_STATUS: C2RustUnnamed_36 = 51539607553;
pub const KEYC_TRIPLECLICK11_PANE: C2RustUnnamed_36 = 51539610368;
pub const KEYC_TRIPLECLICK10_PANE: C2RustUnnamed_36 = 51539610112;
pub const KEYC_TRIPLECLICK9_PANE: C2RustUnnamed_36 = 51539609856;
pub const KEYC_TRIPLECLICK8_PANE: C2RustUnnamed_36 = 51539609600;
pub const KEYC_TRIPLECLICK7_PANE: C2RustUnnamed_36 = 51539609344;
pub const KEYC_TRIPLECLICK6_PANE: C2RustUnnamed_36 = 51539609088;
pub const KEYC_TRIPLECLICK3_PANE: C2RustUnnamed_36 = 51539608320;
pub const KEYC_TRIPLECLICK2_PANE: C2RustUnnamed_36 = 51539608064;
pub const KEYC_TRIPLECLICK1_PANE: C2RustUnnamed_36 = 51539607808;
pub const KEYC_TRIPLECLICK_PANE: C2RustUnnamed_36 = 51539607552;
pub const KEYC_DOUBLECLICK11_CONTROL9: C2RustUnnamed_36 = 47244643091;
pub const KEYC_DOUBLECLICK10_CONTROL9: C2RustUnnamed_36 = 47244642835;
pub const KEYC_DOUBLECLICK9_CONTROL9: C2RustUnnamed_36 = 47244642579;
pub const KEYC_DOUBLECLICK8_CONTROL9: C2RustUnnamed_36 = 47244642323;
pub const KEYC_DOUBLECLICK7_CONTROL9: C2RustUnnamed_36 = 47244642067;
pub const KEYC_DOUBLECLICK6_CONTROL9: C2RustUnnamed_36 = 47244641811;
pub const KEYC_DOUBLECLICK3_CONTROL9: C2RustUnnamed_36 = 47244641043;
pub const KEYC_DOUBLECLICK2_CONTROL9: C2RustUnnamed_36 = 47244640787;
pub const KEYC_DOUBLECLICK1_CONTROL9: C2RustUnnamed_36 = 47244640531;
pub const KEYC_DOUBLECLICK_CONTROL9: C2RustUnnamed_36 = 47244640275;
pub const KEYC_DOUBLECLICK11_CONTROL8: C2RustUnnamed_36 = 47244643090;
pub const KEYC_DOUBLECLICK10_CONTROL8: C2RustUnnamed_36 = 47244642834;
pub const KEYC_DOUBLECLICK9_CONTROL8: C2RustUnnamed_36 = 47244642578;
pub const KEYC_DOUBLECLICK8_CONTROL8: C2RustUnnamed_36 = 47244642322;
pub const KEYC_DOUBLECLICK7_CONTROL8: C2RustUnnamed_36 = 47244642066;
pub const KEYC_DOUBLECLICK6_CONTROL8: C2RustUnnamed_36 = 47244641810;
pub const KEYC_DOUBLECLICK3_CONTROL8: C2RustUnnamed_36 = 47244641042;
pub const KEYC_DOUBLECLICK2_CONTROL8: C2RustUnnamed_36 = 47244640786;
pub const KEYC_DOUBLECLICK1_CONTROL8: C2RustUnnamed_36 = 47244640530;
pub const KEYC_DOUBLECLICK_CONTROL8: C2RustUnnamed_36 = 47244640274;
pub const KEYC_DOUBLECLICK11_CONTROL7: C2RustUnnamed_36 = 47244643089;
pub const KEYC_DOUBLECLICK10_CONTROL7: C2RustUnnamed_36 = 47244642833;
pub const KEYC_DOUBLECLICK9_CONTROL7: C2RustUnnamed_36 = 47244642577;
pub const KEYC_DOUBLECLICK8_CONTROL7: C2RustUnnamed_36 = 47244642321;
pub const KEYC_DOUBLECLICK7_CONTROL7: C2RustUnnamed_36 = 47244642065;
pub const KEYC_DOUBLECLICK6_CONTROL7: C2RustUnnamed_36 = 47244641809;
pub const KEYC_DOUBLECLICK3_CONTROL7: C2RustUnnamed_36 = 47244641041;
pub const KEYC_DOUBLECLICK2_CONTROL7: C2RustUnnamed_36 = 47244640785;
pub const KEYC_DOUBLECLICK1_CONTROL7: C2RustUnnamed_36 = 47244640529;
pub const KEYC_DOUBLECLICK_CONTROL7: C2RustUnnamed_36 = 47244640273;
pub const KEYC_DOUBLECLICK11_CONTROL6: C2RustUnnamed_36 = 47244643088;
pub const KEYC_DOUBLECLICK10_CONTROL6: C2RustUnnamed_36 = 47244642832;
pub const KEYC_DOUBLECLICK9_CONTROL6: C2RustUnnamed_36 = 47244642576;
pub const KEYC_DOUBLECLICK8_CONTROL6: C2RustUnnamed_36 = 47244642320;
pub const KEYC_DOUBLECLICK7_CONTROL6: C2RustUnnamed_36 = 47244642064;
pub const KEYC_DOUBLECLICK6_CONTROL6: C2RustUnnamed_36 = 47244641808;
pub const KEYC_DOUBLECLICK3_CONTROL6: C2RustUnnamed_36 = 47244641040;
pub const KEYC_DOUBLECLICK2_CONTROL6: C2RustUnnamed_36 = 47244640784;
pub const KEYC_DOUBLECLICK1_CONTROL6: C2RustUnnamed_36 = 47244640528;
pub const KEYC_DOUBLECLICK_CONTROL6: C2RustUnnamed_36 = 47244640272;
pub const KEYC_DOUBLECLICK11_CONTROL5: C2RustUnnamed_36 = 47244643087;
pub const KEYC_DOUBLECLICK10_CONTROL5: C2RustUnnamed_36 = 47244642831;
pub const KEYC_DOUBLECLICK9_CONTROL5: C2RustUnnamed_36 = 47244642575;
pub const KEYC_DOUBLECLICK8_CONTROL5: C2RustUnnamed_36 = 47244642319;
pub const KEYC_DOUBLECLICK7_CONTROL5: C2RustUnnamed_36 = 47244642063;
pub const KEYC_DOUBLECLICK6_CONTROL5: C2RustUnnamed_36 = 47244641807;
pub const KEYC_DOUBLECLICK3_CONTROL5: C2RustUnnamed_36 = 47244641039;
pub const KEYC_DOUBLECLICK2_CONTROL5: C2RustUnnamed_36 = 47244640783;
pub const KEYC_DOUBLECLICK1_CONTROL5: C2RustUnnamed_36 = 47244640527;
pub const KEYC_DOUBLECLICK_CONTROL5: C2RustUnnamed_36 = 47244640271;
pub const KEYC_DOUBLECLICK11_CONTROL4: C2RustUnnamed_36 = 47244643086;
pub const KEYC_DOUBLECLICK10_CONTROL4: C2RustUnnamed_36 = 47244642830;
pub const KEYC_DOUBLECLICK9_CONTROL4: C2RustUnnamed_36 = 47244642574;
pub const KEYC_DOUBLECLICK8_CONTROL4: C2RustUnnamed_36 = 47244642318;
pub const KEYC_DOUBLECLICK7_CONTROL4: C2RustUnnamed_36 = 47244642062;
pub const KEYC_DOUBLECLICK6_CONTROL4: C2RustUnnamed_36 = 47244641806;
pub const KEYC_DOUBLECLICK3_CONTROL4: C2RustUnnamed_36 = 47244641038;
pub const KEYC_DOUBLECLICK2_CONTROL4: C2RustUnnamed_36 = 47244640782;
pub const KEYC_DOUBLECLICK1_CONTROL4: C2RustUnnamed_36 = 47244640526;
pub const KEYC_DOUBLECLICK_CONTROL4: C2RustUnnamed_36 = 47244640270;
pub const KEYC_DOUBLECLICK11_CONTROL3: C2RustUnnamed_36 = 47244643085;
pub const KEYC_DOUBLECLICK10_CONTROL3: C2RustUnnamed_36 = 47244642829;
pub const KEYC_DOUBLECLICK9_CONTROL3: C2RustUnnamed_36 = 47244642573;
pub const KEYC_DOUBLECLICK8_CONTROL3: C2RustUnnamed_36 = 47244642317;
pub const KEYC_DOUBLECLICK7_CONTROL3: C2RustUnnamed_36 = 47244642061;
pub const KEYC_DOUBLECLICK6_CONTROL3: C2RustUnnamed_36 = 47244641805;
pub const KEYC_DOUBLECLICK3_CONTROL3: C2RustUnnamed_36 = 47244641037;
pub const KEYC_DOUBLECLICK2_CONTROL3: C2RustUnnamed_36 = 47244640781;
pub const KEYC_DOUBLECLICK1_CONTROL3: C2RustUnnamed_36 = 47244640525;
pub const KEYC_DOUBLECLICK_CONTROL3: C2RustUnnamed_36 = 47244640269;
pub const KEYC_DOUBLECLICK11_CONTROL2: C2RustUnnamed_36 = 47244643084;
pub const KEYC_DOUBLECLICK10_CONTROL2: C2RustUnnamed_36 = 47244642828;
pub const KEYC_DOUBLECLICK9_CONTROL2: C2RustUnnamed_36 = 47244642572;
pub const KEYC_DOUBLECLICK8_CONTROL2: C2RustUnnamed_36 = 47244642316;
pub const KEYC_DOUBLECLICK7_CONTROL2: C2RustUnnamed_36 = 47244642060;
pub const KEYC_DOUBLECLICK6_CONTROL2: C2RustUnnamed_36 = 47244641804;
pub const KEYC_DOUBLECLICK3_CONTROL2: C2RustUnnamed_36 = 47244641036;
pub const KEYC_DOUBLECLICK2_CONTROL2: C2RustUnnamed_36 = 47244640780;
pub const KEYC_DOUBLECLICK1_CONTROL2: C2RustUnnamed_36 = 47244640524;
pub const KEYC_DOUBLECLICK_CONTROL2: C2RustUnnamed_36 = 47244640268;
pub const KEYC_DOUBLECLICK11_CONTROL1: C2RustUnnamed_36 = 47244643083;
pub const KEYC_DOUBLECLICK10_CONTROL1: C2RustUnnamed_36 = 47244642827;
pub const KEYC_DOUBLECLICK9_CONTROL1: C2RustUnnamed_36 = 47244642571;
pub const KEYC_DOUBLECLICK8_CONTROL1: C2RustUnnamed_36 = 47244642315;
pub const KEYC_DOUBLECLICK7_CONTROL1: C2RustUnnamed_36 = 47244642059;
pub const KEYC_DOUBLECLICK6_CONTROL1: C2RustUnnamed_36 = 47244641803;
pub const KEYC_DOUBLECLICK3_CONTROL1: C2RustUnnamed_36 = 47244641035;
pub const KEYC_DOUBLECLICK2_CONTROL1: C2RustUnnamed_36 = 47244640779;
pub const KEYC_DOUBLECLICK1_CONTROL1: C2RustUnnamed_36 = 47244640523;
pub const KEYC_DOUBLECLICK_CONTROL1: C2RustUnnamed_36 = 47244640267;
pub const KEYC_DOUBLECLICK11_CONTROL0: C2RustUnnamed_36 = 47244643082;
pub const KEYC_DOUBLECLICK10_CONTROL0: C2RustUnnamed_36 = 47244642826;
pub const KEYC_DOUBLECLICK9_CONTROL0: C2RustUnnamed_36 = 47244642570;
pub const KEYC_DOUBLECLICK8_CONTROL0: C2RustUnnamed_36 = 47244642314;
pub const KEYC_DOUBLECLICK7_CONTROL0: C2RustUnnamed_36 = 47244642058;
pub const KEYC_DOUBLECLICK6_CONTROL0: C2RustUnnamed_36 = 47244641802;
pub const KEYC_DOUBLECLICK3_CONTROL0: C2RustUnnamed_36 = 47244641034;
pub const KEYC_DOUBLECLICK2_CONTROL0: C2RustUnnamed_36 = 47244640778;
pub const KEYC_DOUBLECLICK1_CONTROL0: C2RustUnnamed_36 = 47244640522;
pub const KEYC_DOUBLECLICK_CONTROL0: C2RustUnnamed_36 = 47244640266;
pub const KEYC_DOUBLECLICK11_EMPTY: C2RustUnnamed_36 = 47244643081;
pub const KEYC_DOUBLECLICK10_EMPTY: C2RustUnnamed_36 = 47244642825;
pub const KEYC_DOUBLECLICK9_EMPTY: C2RustUnnamed_36 = 47244642569;
pub const KEYC_DOUBLECLICK8_EMPTY: C2RustUnnamed_36 = 47244642313;
pub const KEYC_DOUBLECLICK7_EMPTY: C2RustUnnamed_36 = 47244642057;
pub const KEYC_DOUBLECLICK6_EMPTY: C2RustUnnamed_36 = 47244641801;
pub const KEYC_DOUBLECLICK3_EMPTY: C2RustUnnamed_36 = 47244641033;
pub const KEYC_DOUBLECLICK2_EMPTY: C2RustUnnamed_36 = 47244640777;
pub const KEYC_DOUBLECLICK1_EMPTY: C2RustUnnamed_36 = 47244640521;
pub const KEYC_DOUBLECLICK_EMPTY: C2RustUnnamed_36 = 47244640265;
pub const KEYC_DOUBLECLICK11_SCROLLBAR_DOWN: C2RustUnnamed_36 = 47244643080;
pub const KEYC_DOUBLECLICK10_SCROLLBAR_DOWN: C2RustUnnamed_36 = 47244642824;
pub const KEYC_DOUBLECLICK9_SCROLLBAR_DOWN: C2RustUnnamed_36 = 47244642568;
pub const KEYC_DOUBLECLICK8_SCROLLBAR_DOWN: C2RustUnnamed_36 = 47244642312;
pub const KEYC_DOUBLECLICK7_SCROLLBAR_DOWN: C2RustUnnamed_36 = 47244642056;
pub const KEYC_DOUBLECLICK6_SCROLLBAR_DOWN: C2RustUnnamed_36 = 47244641800;
pub const KEYC_DOUBLECLICK3_SCROLLBAR_DOWN: C2RustUnnamed_36 = 47244641032;
pub const KEYC_DOUBLECLICK2_SCROLLBAR_DOWN: C2RustUnnamed_36 = 47244640776;
pub const KEYC_DOUBLECLICK1_SCROLLBAR_DOWN: C2RustUnnamed_36 = 47244640520;
pub const KEYC_DOUBLECLICK_SCROLLBAR_DOWN: C2RustUnnamed_36 = 47244640264;
pub const KEYC_DOUBLECLICK11_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 47244643079;
pub const KEYC_DOUBLECLICK10_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 47244642823;
pub const KEYC_DOUBLECLICK9_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 47244642567;
pub const KEYC_DOUBLECLICK8_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 47244642311;
pub const KEYC_DOUBLECLICK7_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 47244642055;
pub const KEYC_DOUBLECLICK6_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 47244641799;
pub const KEYC_DOUBLECLICK3_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 47244641031;
pub const KEYC_DOUBLECLICK2_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 47244640775;
pub const KEYC_DOUBLECLICK1_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 47244640519;
pub const KEYC_DOUBLECLICK_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 47244640263;
pub const KEYC_DOUBLECLICK11_SCROLLBAR_UP: C2RustUnnamed_36 = 47244643078;
pub const KEYC_DOUBLECLICK10_SCROLLBAR_UP: C2RustUnnamed_36 = 47244642822;
pub const KEYC_DOUBLECLICK9_SCROLLBAR_UP: C2RustUnnamed_36 = 47244642566;
pub const KEYC_DOUBLECLICK8_SCROLLBAR_UP: C2RustUnnamed_36 = 47244642310;
pub const KEYC_DOUBLECLICK7_SCROLLBAR_UP: C2RustUnnamed_36 = 47244642054;
pub const KEYC_DOUBLECLICK6_SCROLLBAR_UP: C2RustUnnamed_36 = 47244641798;
pub const KEYC_DOUBLECLICK3_SCROLLBAR_UP: C2RustUnnamed_36 = 47244641030;
pub const KEYC_DOUBLECLICK2_SCROLLBAR_UP: C2RustUnnamed_36 = 47244640774;
pub const KEYC_DOUBLECLICK1_SCROLLBAR_UP: C2RustUnnamed_36 = 47244640518;
pub const KEYC_DOUBLECLICK_SCROLLBAR_UP: C2RustUnnamed_36 = 47244640262;
pub const KEYC_DOUBLECLICK11_BORDER: C2RustUnnamed_36 = 47244643077;
pub const KEYC_DOUBLECLICK10_BORDER: C2RustUnnamed_36 = 47244642821;
pub const KEYC_DOUBLECLICK9_BORDER: C2RustUnnamed_36 = 47244642565;
pub const KEYC_DOUBLECLICK8_BORDER: C2RustUnnamed_36 = 47244642309;
pub const KEYC_DOUBLECLICK7_BORDER: C2RustUnnamed_36 = 47244642053;
pub const KEYC_DOUBLECLICK6_BORDER: C2RustUnnamed_36 = 47244641797;
pub const KEYC_DOUBLECLICK3_BORDER: C2RustUnnamed_36 = 47244641029;
pub const KEYC_DOUBLECLICK2_BORDER: C2RustUnnamed_36 = 47244640773;
pub const KEYC_DOUBLECLICK1_BORDER: C2RustUnnamed_36 = 47244640517;
pub const KEYC_DOUBLECLICK_BORDER: C2RustUnnamed_36 = 47244640261;
pub const KEYC_DOUBLECLICK11_STATUS_DEFAULT: C2RustUnnamed_36 = 47244643076;
pub const KEYC_DOUBLECLICK10_STATUS_DEFAULT: C2RustUnnamed_36 = 47244642820;
pub const KEYC_DOUBLECLICK9_STATUS_DEFAULT: C2RustUnnamed_36 = 47244642564;
pub const KEYC_DOUBLECLICK8_STATUS_DEFAULT: C2RustUnnamed_36 = 47244642308;
pub const KEYC_DOUBLECLICK7_STATUS_DEFAULT: C2RustUnnamed_36 = 47244642052;
pub const KEYC_DOUBLECLICK6_STATUS_DEFAULT: C2RustUnnamed_36 = 47244641796;
pub const KEYC_DOUBLECLICK3_STATUS_DEFAULT: C2RustUnnamed_36 = 47244641028;
pub const KEYC_DOUBLECLICK2_STATUS_DEFAULT: C2RustUnnamed_36 = 47244640772;
pub const KEYC_DOUBLECLICK1_STATUS_DEFAULT: C2RustUnnamed_36 = 47244640516;
pub const KEYC_DOUBLECLICK_STATUS_DEFAULT: C2RustUnnamed_36 = 47244640260;
pub const KEYC_DOUBLECLICK11_STATUS_RIGHT: C2RustUnnamed_36 = 47244643075;
pub const KEYC_DOUBLECLICK10_STATUS_RIGHT: C2RustUnnamed_36 = 47244642819;
pub const KEYC_DOUBLECLICK9_STATUS_RIGHT: C2RustUnnamed_36 = 47244642563;
pub const KEYC_DOUBLECLICK8_STATUS_RIGHT: C2RustUnnamed_36 = 47244642307;
pub const KEYC_DOUBLECLICK7_STATUS_RIGHT: C2RustUnnamed_36 = 47244642051;
pub const KEYC_DOUBLECLICK6_STATUS_RIGHT: C2RustUnnamed_36 = 47244641795;
pub const KEYC_DOUBLECLICK3_STATUS_RIGHT: C2RustUnnamed_36 = 47244641027;
pub const KEYC_DOUBLECLICK2_STATUS_RIGHT: C2RustUnnamed_36 = 47244640771;
pub const KEYC_DOUBLECLICK1_STATUS_RIGHT: C2RustUnnamed_36 = 47244640515;
pub const KEYC_DOUBLECLICK_STATUS_RIGHT: C2RustUnnamed_36 = 47244640259;
pub const KEYC_DOUBLECLICK11_STATUS_LEFT: C2RustUnnamed_36 = 47244643074;
pub const KEYC_DOUBLECLICK10_STATUS_LEFT: C2RustUnnamed_36 = 47244642818;
pub const KEYC_DOUBLECLICK9_STATUS_LEFT: C2RustUnnamed_36 = 47244642562;
pub const KEYC_DOUBLECLICK8_STATUS_LEFT: C2RustUnnamed_36 = 47244642306;
pub const KEYC_DOUBLECLICK7_STATUS_LEFT: C2RustUnnamed_36 = 47244642050;
pub const KEYC_DOUBLECLICK6_STATUS_LEFT: C2RustUnnamed_36 = 47244641794;
pub const KEYC_DOUBLECLICK3_STATUS_LEFT: C2RustUnnamed_36 = 47244641026;
pub const KEYC_DOUBLECLICK2_STATUS_LEFT: C2RustUnnamed_36 = 47244640770;
pub const KEYC_DOUBLECLICK1_STATUS_LEFT: C2RustUnnamed_36 = 47244640514;
pub const KEYC_DOUBLECLICK_STATUS_LEFT: C2RustUnnamed_36 = 47244640258;
pub const KEYC_DOUBLECLICK11_STATUS: C2RustUnnamed_36 = 47244643073;
pub const KEYC_DOUBLECLICK10_STATUS: C2RustUnnamed_36 = 47244642817;
pub const KEYC_DOUBLECLICK9_STATUS: C2RustUnnamed_36 = 47244642561;
pub const KEYC_DOUBLECLICK8_STATUS: C2RustUnnamed_36 = 47244642305;
pub const KEYC_DOUBLECLICK7_STATUS: C2RustUnnamed_36 = 47244642049;
pub const KEYC_DOUBLECLICK6_STATUS: C2RustUnnamed_36 = 47244641793;
pub const KEYC_DOUBLECLICK3_STATUS: C2RustUnnamed_36 = 47244641025;
pub const KEYC_DOUBLECLICK2_STATUS: C2RustUnnamed_36 = 47244640769;
pub const KEYC_DOUBLECLICK1_STATUS: C2RustUnnamed_36 = 47244640513;
pub const KEYC_DOUBLECLICK_STATUS: C2RustUnnamed_36 = 47244640257;
pub const KEYC_DOUBLECLICK11_PANE: C2RustUnnamed_36 = 47244643072;
pub const KEYC_DOUBLECLICK10_PANE: C2RustUnnamed_36 = 47244642816;
pub const KEYC_DOUBLECLICK9_PANE: C2RustUnnamed_36 = 47244642560;
pub const KEYC_DOUBLECLICK8_PANE: C2RustUnnamed_36 = 47244642304;
pub const KEYC_DOUBLECLICK7_PANE: C2RustUnnamed_36 = 47244642048;
pub const KEYC_DOUBLECLICK6_PANE: C2RustUnnamed_36 = 47244641792;
pub const KEYC_DOUBLECLICK3_PANE: C2RustUnnamed_36 = 47244641024;
pub const KEYC_DOUBLECLICK2_PANE: C2RustUnnamed_36 = 47244640768;
pub const KEYC_DOUBLECLICK1_PANE: C2RustUnnamed_36 = 47244640512;
pub const KEYC_DOUBLECLICK_PANE: C2RustUnnamed_36 = 47244640256;
pub const KEYC_SECONDCLICK11_CONTROL9: C2RustUnnamed_36 = 42949675795;
pub const KEYC_SECONDCLICK10_CONTROL9: C2RustUnnamed_36 = 42949675539;
pub const KEYC_SECONDCLICK9_CONTROL9: C2RustUnnamed_36 = 42949675283;
pub const KEYC_SECONDCLICK8_CONTROL9: C2RustUnnamed_36 = 42949675027;
pub const KEYC_SECONDCLICK7_CONTROL9: C2RustUnnamed_36 = 42949674771;
pub const KEYC_SECONDCLICK6_CONTROL9: C2RustUnnamed_36 = 42949674515;
pub const KEYC_SECONDCLICK3_CONTROL9: C2RustUnnamed_36 = 42949673747;
pub const KEYC_SECONDCLICK2_CONTROL9: C2RustUnnamed_36 = 42949673491;
pub const KEYC_SECONDCLICK1_CONTROL9: C2RustUnnamed_36 = 42949673235;
pub const KEYC_SECONDCLICK_CONTROL9: C2RustUnnamed_36 = 42949672979;
pub const KEYC_SECONDCLICK11_CONTROL8: C2RustUnnamed_36 = 42949675794;
pub const KEYC_SECONDCLICK10_CONTROL8: C2RustUnnamed_36 = 42949675538;
pub const KEYC_SECONDCLICK9_CONTROL8: C2RustUnnamed_36 = 42949675282;
pub const KEYC_SECONDCLICK8_CONTROL8: C2RustUnnamed_36 = 42949675026;
pub const KEYC_SECONDCLICK7_CONTROL8: C2RustUnnamed_36 = 42949674770;
pub const KEYC_SECONDCLICK6_CONTROL8: C2RustUnnamed_36 = 42949674514;
pub const KEYC_SECONDCLICK3_CONTROL8: C2RustUnnamed_36 = 42949673746;
pub const KEYC_SECONDCLICK2_CONTROL8: C2RustUnnamed_36 = 42949673490;
pub const KEYC_SECONDCLICK1_CONTROL8: C2RustUnnamed_36 = 42949673234;
pub const KEYC_SECONDCLICK_CONTROL8: C2RustUnnamed_36 = 42949672978;
pub const KEYC_SECONDCLICK11_CONTROL7: C2RustUnnamed_36 = 42949675793;
pub const KEYC_SECONDCLICK10_CONTROL7: C2RustUnnamed_36 = 42949675537;
pub const KEYC_SECONDCLICK9_CONTROL7: C2RustUnnamed_36 = 42949675281;
pub const KEYC_SECONDCLICK8_CONTROL7: C2RustUnnamed_36 = 42949675025;
pub const KEYC_SECONDCLICK7_CONTROL7: C2RustUnnamed_36 = 42949674769;
pub const KEYC_SECONDCLICK6_CONTROL7: C2RustUnnamed_36 = 42949674513;
pub const KEYC_SECONDCLICK3_CONTROL7: C2RustUnnamed_36 = 42949673745;
pub const KEYC_SECONDCLICK2_CONTROL7: C2RustUnnamed_36 = 42949673489;
pub const KEYC_SECONDCLICK1_CONTROL7: C2RustUnnamed_36 = 42949673233;
pub const KEYC_SECONDCLICK_CONTROL7: C2RustUnnamed_36 = 42949672977;
pub const KEYC_SECONDCLICK11_CONTROL6: C2RustUnnamed_36 = 42949675792;
pub const KEYC_SECONDCLICK10_CONTROL6: C2RustUnnamed_36 = 42949675536;
pub const KEYC_SECONDCLICK9_CONTROL6: C2RustUnnamed_36 = 42949675280;
pub const KEYC_SECONDCLICK8_CONTROL6: C2RustUnnamed_36 = 42949675024;
pub const KEYC_SECONDCLICK7_CONTROL6: C2RustUnnamed_36 = 42949674768;
pub const KEYC_SECONDCLICK6_CONTROL6: C2RustUnnamed_36 = 42949674512;
pub const KEYC_SECONDCLICK3_CONTROL6: C2RustUnnamed_36 = 42949673744;
pub const KEYC_SECONDCLICK2_CONTROL6: C2RustUnnamed_36 = 42949673488;
pub const KEYC_SECONDCLICK1_CONTROL6: C2RustUnnamed_36 = 42949673232;
pub const KEYC_SECONDCLICK_CONTROL6: C2RustUnnamed_36 = 42949672976;
pub const KEYC_SECONDCLICK11_CONTROL5: C2RustUnnamed_36 = 42949675791;
pub const KEYC_SECONDCLICK10_CONTROL5: C2RustUnnamed_36 = 42949675535;
pub const KEYC_SECONDCLICK9_CONTROL5: C2RustUnnamed_36 = 42949675279;
pub const KEYC_SECONDCLICK8_CONTROL5: C2RustUnnamed_36 = 42949675023;
pub const KEYC_SECONDCLICK7_CONTROL5: C2RustUnnamed_36 = 42949674767;
pub const KEYC_SECONDCLICK6_CONTROL5: C2RustUnnamed_36 = 42949674511;
pub const KEYC_SECONDCLICK3_CONTROL5: C2RustUnnamed_36 = 42949673743;
pub const KEYC_SECONDCLICK2_CONTROL5: C2RustUnnamed_36 = 42949673487;
pub const KEYC_SECONDCLICK1_CONTROL5: C2RustUnnamed_36 = 42949673231;
pub const KEYC_SECONDCLICK_CONTROL5: C2RustUnnamed_36 = 42949672975;
pub const KEYC_SECONDCLICK11_CONTROL4: C2RustUnnamed_36 = 42949675790;
pub const KEYC_SECONDCLICK10_CONTROL4: C2RustUnnamed_36 = 42949675534;
pub const KEYC_SECONDCLICK9_CONTROL4: C2RustUnnamed_36 = 42949675278;
pub const KEYC_SECONDCLICK8_CONTROL4: C2RustUnnamed_36 = 42949675022;
pub const KEYC_SECONDCLICK7_CONTROL4: C2RustUnnamed_36 = 42949674766;
pub const KEYC_SECONDCLICK6_CONTROL4: C2RustUnnamed_36 = 42949674510;
pub const KEYC_SECONDCLICK3_CONTROL4: C2RustUnnamed_36 = 42949673742;
pub const KEYC_SECONDCLICK2_CONTROL4: C2RustUnnamed_36 = 42949673486;
pub const KEYC_SECONDCLICK1_CONTROL4: C2RustUnnamed_36 = 42949673230;
pub const KEYC_SECONDCLICK_CONTROL4: C2RustUnnamed_36 = 42949672974;
pub const KEYC_SECONDCLICK11_CONTROL3: C2RustUnnamed_36 = 42949675789;
pub const KEYC_SECONDCLICK10_CONTROL3: C2RustUnnamed_36 = 42949675533;
pub const KEYC_SECONDCLICK9_CONTROL3: C2RustUnnamed_36 = 42949675277;
pub const KEYC_SECONDCLICK8_CONTROL3: C2RustUnnamed_36 = 42949675021;
pub const KEYC_SECONDCLICK7_CONTROL3: C2RustUnnamed_36 = 42949674765;
pub const KEYC_SECONDCLICK6_CONTROL3: C2RustUnnamed_36 = 42949674509;
pub const KEYC_SECONDCLICK3_CONTROL3: C2RustUnnamed_36 = 42949673741;
pub const KEYC_SECONDCLICK2_CONTROL3: C2RustUnnamed_36 = 42949673485;
pub const KEYC_SECONDCLICK1_CONTROL3: C2RustUnnamed_36 = 42949673229;
pub const KEYC_SECONDCLICK_CONTROL3: C2RustUnnamed_36 = 42949672973;
pub const KEYC_SECONDCLICK11_CONTROL2: C2RustUnnamed_36 = 42949675788;
pub const KEYC_SECONDCLICK10_CONTROL2: C2RustUnnamed_36 = 42949675532;
pub const KEYC_SECONDCLICK9_CONTROL2: C2RustUnnamed_36 = 42949675276;
pub const KEYC_SECONDCLICK8_CONTROL2: C2RustUnnamed_36 = 42949675020;
pub const KEYC_SECONDCLICK7_CONTROL2: C2RustUnnamed_36 = 42949674764;
pub const KEYC_SECONDCLICK6_CONTROL2: C2RustUnnamed_36 = 42949674508;
pub const KEYC_SECONDCLICK3_CONTROL2: C2RustUnnamed_36 = 42949673740;
pub const KEYC_SECONDCLICK2_CONTROL2: C2RustUnnamed_36 = 42949673484;
pub const KEYC_SECONDCLICK1_CONTROL2: C2RustUnnamed_36 = 42949673228;
pub const KEYC_SECONDCLICK_CONTROL2: C2RustUnnamed_36 = 42949672972;
pub const KEYC_SECONDCLICK11_CONTROL1: C2RustUnnamed_36 = 42949675787;
pub const KEYC_SECONDCLICK10_CONTROL1: C2RustUnnamed_36 = 42949675531;
pub const KEYC_SECONDCLICK9_CONTROL1: C2RustUnnamed_36 = 42949675275;
pub const KEYC_SECONDCLICK8_CONTROL1: C2RustUnnamed_36 = 42949675019;
pub const KEYC_SECONDCLICK7_CONTROL1: C2RustUnnamed_36 = 42949674763;
pub const KEYC_SECONDCLICK6_CONTROL1: C2RustUnnamed_36 = 42949674507;
pub const KEYC_SECONDCLICK3_CONTROL1: C2RustUnnamed_36 = 42949673739;
pub const KEYC_SECONDCLICK2_CONTROL1: C2RustUnnamed_36 = 42949673483;
pub const KEYC_SECONDCLICK1_CONTROL1: C2RustUnnamed_36 = 42949673227;
pub const KEYC_SECONDCLICK_CONTROL1: C2RustUnnamed_36 = 42949672971;
pub const KEYC_SECONDCLICK11_CONTROL0: C2RustUnnamed_36 = 42949675786;
pub const KEYC_SECONDCLICK10_CONTROL0: C2RustUnnamed_36 = 42949675530;
pub const KEYC_SECONDCLICK9_CONTROL0: C2RustUnnamed_36 = 42949675274;
pub const KEYC_SECONDCLICK8_CONTROL0: C2RustUnnamed_36 = 42949675018;
pub const KEYC_SECONDCLICK7_CONTROL0: C2RustUnnamed_36 = 42949674762;
pub const KEYC_SECONDCLICK6_CONTROL0: C2RustUnnamed_36 = 42949674506;
pub const KEYC_SECONDCLICK3_CONTROL0: C2RustUnnamed_36 = 42949673738;
pub const KEYC_SECONDCLICK2_CONTROL0: C2RustUnnamed_36 = 42949673482;
pub const KEYC_SECONDCLICK1_CONTROL0: C2RustUnnamed_36 = 42949673226;
pub const KEYC_SECONDCLICK_CONTROL0: C2RustUnnamed_36 = 42949672970;
pub const KEYC_SECONDCLICK11_EMPTY: C2RustUnnamed_36 = 42949675785;
pub const KEYC_SECONDCLICK10_EMPTY: C2RustUnnamed_36 = 42949675529;
pub const KEYC_SECONDCLICK9_EMPTY: C2RustUnnamed_36 = 42949675273;
pub const KEYC_SECONDCLICK8_EMPTY: C2RustUnnamed_36 = 42949675017;
pub const KEYC_SECONDCLICK7_EMPTY: C2RustUnnamed_36 = 42949674761;
pub const KEYC_SECONDCLICK6_EMPTY: C2RustUnnamed_36 = 42949674505;
pub const KEYC_SECONDCLICK3_EMPTY: C2RustUnnamed_36 = 42949673737;
pub const KEYC_SECONDCLICK2_EMPTY: C2RustUnnamed_36 = 42949673481;
pub const KEYC_SECONDCLICK1_EMPTY: C2RustUnnamed_36 = 42949673225;
pub const KEYC_SECONDCLICK_EMPTY: C2RustUnnamed_36 = 42949672969;
pub const KEYC_SECONDCLICK11_SCROLLBAR_DOWN: C2RustUnnamed_36 = 42949675784;
pub const KEYC_SECONDCLICK10_SCROLLBAR_DOWN: C2RustUnnamed_36 = 42949675528;
pub const KEYC_SECONDCLICK9_SCROLLBAR_DOWN: C2RustUnnamed_36 = 42949675272;
pub const KEYC_SECONDCLICK8_SCROLLBAR_DOWN: C2RustUnnamed_36 = 42949675016;
pub const KEYC_SECONDCLICK7_SCROLLBAR_DOWN: C2RustUnnamed_36 = 42949674760;
pub const KEYC_SECONDCLICK6_SCROLLBAR_DOWN: C2RustUnnamed_36 = 42949674504;
pub const KEYC_SECONDCLICK3_SCROLLBAR_DOWN: C2RustUnnamed_36 = 42949673736;
pub const KEYC_SECONDCLICK2_SCROLLBAR_DOWN: C2RustUnnamed_36 = 42949673480;
pub const KEYC_SECONDCLICK1_SCROLLBAR_DOWN: C2RustUnnamed_36 = 42949673224;
pub const KEYC_SECONDCLICK_SCROLLBAR_DOWN: C2RustUnnamed_36 = 42949672968;
pub const KEYC_SECONDCLICK11_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 42949675783;
pub const KEYC_SECONDCLICK10_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 42949675527;
pub const KEYC_SECONDCLICK9_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 42949675271;
pub const KEYC_SECONDCLICK8_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 42949675015;
pub const KEYC_SECONDCLICK7_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 42949674759;
pub const KEYC_SECONDCLICK6_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 42949674503;
pub const KEYC_SECONDCLICK3_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 42949673735;
pub const KEYC_SECONDCLICK2_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 42949673479;
pub const KEYC_SECONDCLICK1_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 42949673223;
pub const KEYC_SECONDCLICK_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 42949672967;
pub const KEYC_SECONDCLICK11_SCROLLBAR_UP: C2RustUnnamed_36 = 42949675782;
pub const KEYC_SECONDCLICK10_SCROLLBAR_UP: C2RustUnnamed_36 = 42949675526;
pub const KEYC_SECONDCLICK9_SCROLLBAR_UP: C2RustUnnamed_36 = 42949675270;
pub const KEYC_SECONDCLICK8_SCROLLBAR_UP: C2RustUnnamed_36 = 42949675014;
pub const KEYC_SECONDCLICK7_SCROLLBAR_UP: C2RustUnnamed_36 = 42949674758;
pub const KEYC_SECONDCLICK6_SCROLLBAR_UP: C2RustUnnamed_36 = 42949674502;
pub const KEYC_SECONDCLICK3_SCROLLBAR_UP: C2RustUnnamed_36 = 42949673734;
pub const KEYC_SECONDCLICK2_SCROLLBAR_UP: C2RustUnnamed_36 = 42949673478;
pub const KEYC_SECONDCLICK1_SCROLLBAR_UP: C2RustUnnamed_36 = 42949673222;
pub const KEYC_SECONDCLICK_SCROLLBAR_UP: C2RustUnnamed_36 = 42949672966;
pub const KEYC_SECONDCLICK11_BORDER: C2RustUnnamed_36 = 42949675781;
pub const KEYC_SECONDCLICK10_BORDER: C2RustUnnamed_36 = 42949675525;
pub const KEYC_SECONDCLICK9_BORDER: C2RustUnnamed_36 = 42949675269;
pub const KEYC_SECONDCLICK8_BORDER: C2RustUnnamed_36 = 42949675013;
pub const KEYC_SECONDCLICK7_BORDER: C2RustUnnamed_36 = 42949674757;
pub const KEYC_SECONDCLICK6_BORDER: C2RustUnnamed_36 = 42949674501;
pub const KEYC_SECONDCLICK3_BORDER: C2RustUnnamed_36 = 42949673733;
pub const KEYC_SECONDCLICK2_BORDER: C2RustUnnamed_36 = 42949673477;
pub const KEYC_SECONDCLICK1_BORDER: C2RustUnnamed_36 = 42949673221;
pub const KEYC_SECONDCLICK_BORDER: C2RustUnnamed_36 = 42949672965;
pub const KEYC_SECONDCLICK11_STATUS_DEFAULT: C2RustUnnamed_36 = 42949675780;
pub const KEYC_SECONDCLICK10_STATUS_DEFAULT: C2RustUnnamed_36 = 42949675524;
pub const KEYC_SECONDCLICK9_STATUS_DEFAULT: C2RustUnnamed_36 = 42949675268;
pub const KEYC_SECONDCLICK8_STATUS_DEFAULT: C2RustUnnamed_36 = 42949675012;
pub const KEYC_SECONDCLICK7_STATUS_DEFAULT: C2RustUnnamed_36 = 42949674756;
pub const KEYC_SECONDCLICK6_STATUS_DEFAULT: C2RustUnnamed_36 = 42949674500;
pub const KEYC_SECONDCLICK3_STATUS_DEFAULT: C2RustUnnamed_36 = 42949673732;
pub const KEYC_SECONDCLICK2_STATUS_DEFAULT: C2RustUnnamed_36 = 42949673476;
pub const KEYC_SECONDCLICK1_STATUS_DEFAULT: C2RustUnnamed_36 = 42949673220;
pub const KEYC_SECONDCLICK_STATUS_DEFAULT: C2RustUnnamed_36 = 42949672964;
pub const KEYC_SECONDCLICK11_STATUS_RIGHT: C2RustUnnamed_36 = 42949675779;
pub const KEYC_SECONDCLICK10_STATUS_RIGHT: C2RustUnnamed_36 = 42949675523;
pub const KEYC_SECONDCLICK9_STATUS_RIGHT: C2RustUnnamed_36 = 42949675267;
pub const KEYC_SECONDCLICK8_STATUS_RIGHT: C2RustUnnamed_36 = 42949675011;
pub const KEYC_SECONDCLICK7_STATUS_RIGHT: C2RustUnnamed_36 = 42949674755;
pub const KEYC_SECONDCLICK6_STATUS_RIGHT: C2RustUnnamed_36 = 42949674499;
pub const KEYC_SECONDCLICK3_STATUS_RIGHT: C2RustUnnamed_36 = 42949673731;
pub const KEYC_SECONDCLICK2_STATUS_RIGHT: C2RustUnnamed_36 = 42949673475;
pub const KEYC_SECONDCLICK1_STATUS_RIGHT: C2RustUnnamed_36 = 42949673219;
pub const KEYC_SECONDCLICK_STATUS_RIGHT: C2RustUnnamed_36 = 42949672963;
pub const KEYC_SECONDCLICK11_STATUS_LEFT: C2RustUnnamed_36 = 42949675778;
pub const KEYC_SECONDCLICK10_STATUS_LEFT: C2RustUnnamed_36 = 42949675522;
pub const KEYC_SECONDCLICK9_STATUS_LEFT: C2RustUnnamed_36 = 42949675266;
pub const KEYC_SECONDCLICK8_STATUS_LEFT: C2RustUnnamed_36 = 42949675010;
pub const KEYC_SECONDCLICK7_STATUS_LEFT: C2RustUnnamed_36 = 42949674754;
pub const KEYC_SECONDCLICK6_STATUS_LEFT: C2RustUnnamed_36 = 42949674498;
pub const KEYC_SECONDCLICK3_STATUS_LEFT: C2RustUnnamed_36 = 42949673730;
pub const KEYC_SECONDCLICK2_STATUS_LEFT: C2RustUnnamed_36 = 42949673474;
pub const KEYC_SECONDCLICK1_STATUS_LEFT: C2RustUnnamed_36 = 42949673218;
pub const KEYC_SECONDCLICK_STATUS_LEFT: C2RustUnnamed_36 = 42949672962;
pub const KEYC_SECONDCLICK11_STATUS: C2RustUnnamed_36 = 42949675777;
pub const KEYC_SECONDCLICK10_STATUS: C2RustUnnamed_36 = 42949675521;
pub const KEYC_SECONDCLICK9_STATUS: C2RustUnnamed_36 = 42949675265;
pub const KEYC_SECONDCLICK8_STATUS: C2RustUnnamed_36 = 42949675009;
pub const KEYC_SECONDCLICK7_STATUS: C2RustUnnamed_36 = 42949674753;
pub const KEYC_SECONDCLICK6_STATUS: C2RustUnnamed_36 = 42949674497;
pub const KEYC_SECONDCLICK3_STATUS: C2RustUnnamed_36 = 42949673729;
pub const KEYC_SECONDCLICK2_STATUS: C2RustUnnamed_36 = 42949673473;
pub const KEYC_SECONDCLICK1_STATUS: C2RustUnnamed_36 = 42949673217;
pub const KEYC_SECONDCLICK_STATUS: C2RustUnnamed_36 = 42949672961;
pub const KEYC_SECONDCLICK11_PANE: C2RustUnnamed_36 = 42949675776;
pub const KEYC_SECONDCLICK10_PANE: C2RustUnnamed_36 = 42949675520;
pub const KEYC_SECONDCLICK9_PANE: C2RustUnnamed_36 = 42949675264;
pub const KEYC_SECONDCLICK8_PANE: C2RustUnnamed_36 = 42949675008;
pub const KEYC_SECONDCLICK7_PANE: C2RustUnnamed_36 = 42949674752;
pub const KEYC_SECONDCLICK6_PANE: C2RustUnnamed_36 = 42949674496;
pub const KEYC_SECONDCLICK3_PANE: C2RustUnnamed_36 = 42949673728;
pub const KEYC_SECONDCLICK2_PANE: C2RustUnnamed_36 = 42949673472;
pub const KEYC_SECONDCLICK1_PANE: C2RustUnnamed_36 = 42949673216;
pub const KEYC_SECONDCLICK_PANE: C2RustUnnamed_36 = 42949672960;
pub const KEYC_MOUSEDRAGEND11_CONTROL9: C2RustUnnamed_36 = 30064773907;
pub const KEYC_MOUSEDRAGEND10_CONTROL9: C2RustUnnamed_36 = 30064773651;
pub const KEYC_MOUSEDRAGEND9_CONTROL9: C2RustUnnamed_36 = 30064773395;
pub const KEYC_MOUSEDRAGEND8_CONTROL9: C2RustUnnamed_36 = 30064773139;
pub const KEYC_MOUSEDRAGEND7_CONTROL9: C2RustUnnamed_36 = 30064772883;
pub const KEYC_MOUSEDRAGEND6_CONTROL9: C2RustUnnamed_36 = 30064772627;
pub const KEYC_MOUSEDRAGEND3_CONTROL9: C2RustUnnamed_36 = 30064771859;
pub const KEYC_MOUSEDRAGEND2_CONTROL9: C2RustUnnamed_36 = 30064771603;
pub const KEYC_MOUSEDRAGEND1_CONTROL9: C2RustUnnamed_36 = 30064771347;
pub const KEYC_MOUSEDRAGEND_CONTROL9: C2RustUnnamed_36 = 30064771091;
pub const KEYC_MOUSEDRAGEND11_CONTROL8: C2RustUnnamed_36 = 30064773906;
pub const KEYC_MOUSEDRAGEND10_CONTROL8: C2RustUnnamed_36 = 30064773650;
pub const KEYC_MOUSEDRAGEND9_CONTROL8: C2RustUnnamed_36 = 30064773394;
pub const KEYC_MOUSEDRAGEND8_CONTROL8: C2RustUnnamed_36 = 30064773138;
pub const KEYC_MOUSEDRAGEND7_CONTROL8: C2RustUnnamed_36 = 30064772882;
pub const KEYC_MOUSEDRAGEND6_CONTROL8: C2RustUnnamed_36 = 30064772626;
pub const KEYC_MOUSEDRAGEND3_CONTROL8: C2RustUnnamed_36 = 30064771858;
pub const KEYC_MOUSEDRAGEND2_CONTROL8: C2RustUnnamed_36 = 30064771602;
pub const KEYC_MOUSEDRAGEND1_CONTROL8: C2RustUnnamed_36 = 30064771346;
pub const KEYC_MOUSEDRAGEND_CONTROL8: C2RustUnnamed_36 = 30064771090;
pub const KEYC_MOUSEDRAGEND11_CONTROL7: C2RustUnnamed_36 = 30064773905;
pub const KEYC_MOUSEDRAGEND10_CONTROL7: C2RustUnnamed_36 = 30064773649;
pub const KEYC_MOUSEDRAGEND9_CONTROL7: C2RustUnnamed_36 = 30064773393;
pub const KEYC_MOUSEDRAGEND8_CONTROL7: C2RustUnnamed_36 = 30064773137;
pub const KEYC_MOUSEDRAGEND7_CONTROL7: C2RustUnnamed_36 = 30064772881;
pub const KEYC_MOUSEDRAGEND6_CONTROL7: C2RustUnnamed_36 = 30064772625;
pub const KEYC_MOUSEDRAGEND3_CONTROL7: C2RustUnnamed_36 = 30064771857;
pub const KEYC_MOUSEDRAGEND2_CONTROL7: C2RustUnnamed_36 = 30064771601;
pub const KEYC_MOUSEDRAGEND1_CONTROL7: C2RustUnnamed_36 = 30064771345;
pub const KEYC_MOUSEDRAGEND_CONTROL7: C2RustUnnamed_36 = 30064771089;
pub const KEYC_MOUSEDRAGEND11_CONTROL6: C2RustUnnamed_36 = 30064773904;
pub const KEYC_MOUSEDRAGEND10_CONTROL6: C2RustUnnamed_36 = 30064773648;
pub const KEYC_MOUSEDRAGEND9_CONTROL6: C2RustUnnamed_36 = 30064773392;
pub const KEYC_MOUSEDRAGEND8_CONTROL6: C2RustUnnamed_36 = 30064773136;
pub const KEYC_MOUSEDRAGEND7_CONTROL6: C2RustUnnamed_36 = 30064772880;
pub const KEYC_MOUSEDRAGEND6_CONTROL6: C2RustUnnamed_36 = 30064772624;
pub const KEYC_MOUSEDRAGEND3_CONTROL6: C2RustUnnamed_36 = 30064771856;
pub const KEYC_MOUSEDRAGEND2_CONTROL6: C2RustUnnamed_36 = 30064771600;
pub const KEYC_MOUSEDRAGEND1_CONTROL6: C2RustUnnamed_36 = 30064771344;
pub const KEYC_MOUSEDRAGEND_CONTROL6: C2RustUnnamed_36 = 30064771088;
pub const KEYC_MOUSEDRAGEND11_CONTROL5: C2RustUnnamed_36 = 30064773903;
pub const KEYC_MOUSEDRAGEND10_CONTROL5: C2RustUnnamed_36 = 30064773647;
pub const KEYC_MOUSEDRAGEND9_CONTROL5: C2RustUnnamed_36 = 30064773391;
pub const KEYC_MOUSEDRAGEND8_CONTROL5: C2RustUnnamed_36 = 30064773135;
pub const KEYC_MOUSEDRAGEND7_CONTROL5: C2RustUnnamed_36 = 30064772879;
pub const KEYC_MOUSEDRAGEND6_CONTROL5: C2RustUnnamed_36 = 30064772623;
pub const KEYC_MOUSEDRAGEND3_CONTROL5: C2RustUnnamed_36 = 30064771855;
pub const KEYC_MOUSEDRAGEND2_CONTROL5: C2RustUnnamed_36 = 30064771599;
pub const KEYC_MOUSEDRAGEND1_CONTROL5: C2RustUnnamed_36 = 30064771343;
pub const KEYC_MOUSEDRAGEND_CONTROL5: C2RustUnnamed_36 = 30064771087;
pub const KEYC_MOUSEDRAGEND11_CONTROL4: C2RustUnnamed_36 = 30064773902;
pub const KEYC_MOUSEDRAGEND10_CONTROL4: C2RustUnnamed_36 = 30064773646;
pub const KEYC_MOUSEDRAGEND9_CONTROL4: C2RustUnnamed_36 = 30064773390;
pub const KEYC_MOUSEDRAGEND8_CONTROL4: C2RustUnnamed_36 = 30064773134;
pub const KEYC_MOUSEDRAGEND7_CONTROL4: C2RustUnnamed_36 = 30064772878;
pub const KEYC_MOUSEDRAGEND6_CONTROL4: C2RustUnnamed_36 = 30064772622;
pub const KEYC_MOUSEDRAGEND3_CONTROL4: C2RustUnnamed_36 = 30064771854;
pub const KEYC_MOUSEDRAGEND2_CONTROL4: C2RustUnnamed_36 = 30064771598;
pub const KEYC_MOUSEDRAGEND1_CONTROL4: C2RustUnnamed_36 = 30064771342;
pub const KEYC_MOUSEDRAGEND_CONTROL4: C2RustUnnamed_36 = 30064771086;
pub const KEYC_MOUSEDRAGEND11_CONTROL3: C2RustUnnamed_36 = 30064773901;
pub const KEYC_MOUSEDRAGEND10_CONTROL3: C2RustUnnamed_36 = 30064773645;
pub const KEYC_MOUSEDRAGEND9_CONTROL3: C2RustUnnamed_36 = 30064773389;
pub const KEYC_MOUSEDRAGEND8_CONTROL3: C2RustUnnamed_36 = 30064773133;
pub const KEYC_MOUSEDRAGEND7_CONTROL3: C2RustUnnamed_36 = 30064772877;
pub const KEYC_MOUSEDRAGEND6_CONTROL3: C2RustUnnamed_36 = 30064772621;
pub const KEYC_MOUSEDRAGEND3_CONTROL3: C2RustUnnamed_36 = 30064771853;
pub const KEYC_MOUSEDRAGEND2_CONTROL3: C2RustUnnamed_36 = 30064771597;
pub const KEYC_MOUSEDRAGEND1_CONTROL3: C2RustUnnamed_36 = 30064771341;
pub const KEYC_MOUSEDRAGEND_CONTROL3: C2RustUnnamed_36 = 30064771085;
pub const KEYC_MOUSEDRAGEND11_CONTROL2: C2RustUnnamed_36 = 30064773900;
pub const KEYC_MOUSEDRAGEND10_CONTROL2: C2RustUnnamed_36 = 30064773644;
pub const KEYC_MOUSEDRAGEND9_CONTROL2: C2RustUnnamed_36 = 30064773388;
pub const KEYC_MOUSEDRAGEND8_CONTROL2: C2RustUnnamed_36 = 30064773132;
pub const KEYC_MOUSEDRAGEND7_CONTROL2: C2RustUnnamed_36 = 30064772876;
pub const KEYC_MOUSEDRAGEND6_CONTROL2: C2RustUnnamed_36 = 30064772620;
pub const KEYC_MOUSEDRAGEND3_CONTROL2: C2RustUnnamed_36 = 30064771852;
pub const KEYC_MOUSEDRAGEND2_CONTROL2: C2RustUnnamed_36 = 30064771596;
pub const KEYC_MOUSEDRAGEND1_CONTROL2: C2RustUnnamed_36 = 30064771340;
pub const KEYC_MOUSEDRAGEND_CONTROL2: C2RustUnnamed_36 = 30064771084;
pub const KEYC_MOUSEDRAGEND11_CONTROL1: C2RustUnnamed_36 = 30064773899;
pub const KEYC_MOUSEDRAGEND10_CONTROL1: C2RustUnnamed_36 = 30064773643;
pub const KEYC_MOUSEDRAGEND9_CONTROL1: C2RustUnnamed_36 = 30064773387;
pub const KEYC_MOUSEDRAGEND8_CONTROL1: C2RustUnnamed_36 = 30064773131;
pub const KEYC_MOUSEDRAGEND7_CONTROL1: C2RustUnnamed_36 = 30064772875;
pub const KEYC_MOUSEDRAGEND6_CONTROL1: C2RustUnnamed_36 = 30064772619;
pub const KEYC_MOUSEDRAGEND3_CONTROL1: C2RustUnnamed_36 = 30064771851;
pub const KEYC_MOUSEDRAGEND2_CONTROL1: C2RustUnnamed_36 = 30064771595;
pub const KEYC_MOUSEDRAGEND1_CONTROL1: C2RustUnnamed_36 = 30064771339;
pub const KEYC_MOUSEDRAGEND_CONTROL1: C2RustUnnamed_36 = 30064771083;
pub const KEYC_MOUSEDRAGEND11_CONTROL0: C2RustUnnamed_36 = 30064773898;
pub const KEYC_MOUSEDRAGEND10_CONTROL0: C2RustUnnamed_36 = 30064773642;
pub const KEYC_MOUSEDRAGEND9_CONTROL0: C2RustUnnamed_36 = 30064773386;
pub const KEYC_MOUSEDRAGEND8_CONTROL0: C2RustUnnamed_36 = 30064773130;
pub const KEYC_MOUSEDRAGEND7_CONTROL0: C2RustUnnamed_36 = 30064772874;
pub const KEYC_MOUSEDRAGEND6_CONTROL0: C2RustUnnamed_36 = 30064772618;
pub const KEYC_MOUSEDRAGEND3_CONTROL0: C2RustUnnamed_36 = 30064771850;
pub const KEYC_MOUSEDRAGEND2_CONTROL0: C2RustUnnamed_36 = 30064771594;
pub const KEYC_MOUSEDRAGEND1_CONTROL0: C2RustUnnamed_36 = 30064771338;
pub const KEYC_MOUSEDRAGEND_CONTROL0: C2RustUnnamed_36 = 30064771082;
pub const KEYC_MOUSEDRAGEND11_EMPTY: C2RustUnnamed_36 = 30064773897;
pub const KEYC_MOUSEDRAGEND10_EMPTY: C2RustUnnamed_36 = 30064773641;
pub const KEYC_MOUSEDRAGEND9_EMPTY: C2RustUnnamed_36 = 30064773385;
pub const KEYC_MOUSEDRAGEND8_EMPTY: C2RustUnnamed_36 = 30064773129;
pub const KEYC_MOUSEDRAGEND7_EMPTY: C2RustUnnamed_36 = 30064772873;
pub const KEYC_MOUSEDRAGEND6_EMPTY: C2RustUnnamed_36 = 30064772617;
pub const KEYC_MOUSEDRAGEND3_EMPTY: C2RustUnnamed_36 = 30064771849;
pub const KEYC_MOUSEDRAGEND2_EMPTY: C2RustUnnamed_36 = 30064771593;
pub const KEYC_MOUSEDRAGEND1_EMPTY: C2RustUnnamed_36 = 30064771337;
pub const KEYC_MOUSEDRAGEND_EMPTY: C2RustUnnamed_36 = 30064771081;
pub const KEYC_MOUSEDRAGEND11_SCROLLBAR_DOWN: C2RustUnnamed_36 = 30064773896;
pub const KEYC_MOUSEDRAGEND10_SCROLLBAR_DOWN: C2RustUnnamed_36 = 30064773640;
pub const KEYC_MOUSEDRAGEND9_SCROLLBAR_DOWN: C2RustUnnamed_36 = 30064773384;
pub const KEYC_MOUSEDRAGEND8_SCROLLBAR_DOWN: C2RustUnnamed_36 = 30064773128;
pub const KEYC_MOUSEDRAGEND7_SCROLLBAR_DOWN: C2RustUnnamed_36 = 30064772872;
pub const KEYC_MOUSEDRAGEND6_SCROLLBAR_DOWN: C2RustUnnamed_36 = 30064772616;
pub const KEYC_MOUSEDRAGEND3_SCROLLBAR_DOWN: C2RustUnnamed_36 = 30064771848;
pub const KEYC_MOUSEDRAGEND2_SCROLLBAR_DOWN: C2RustUnnamed_36 = 30064771592;
pub const KEYC_MOUSEDRAGEND1_SCROLLBAR_DOWN: C2RustUnnamed_36 = 30064771336;
pub const KEYC_MOUSEDRAGEND_SCROLLBAR_DOWN: C2RustUnnamed_36 = 30064771080;
pub const KEYC_MOUSEDRAGEND11_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 30064773895;
pub const KEYC_MOUSEDRAGEND10_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 30064773639;
pub const KEYC_MOUSEDRAGEND9_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 30064773383;
pub const KEYC_MOUSEDRAGEND8_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 30064773127;
pub const KEYC_MOUSEDRAGEND7_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 30064772871;
pub const KEYC_MOUSEDRAGEND6_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 30064772615;
pub const KEYC_MOUSEDRAGEND3_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 30064771847;
pub const KEYC_MOUSEDRAGEND2_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 30064771591;
pub const KEYC_MOUSEDRAGEND1_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 30064771335;
pub const KEYC_MOUSEDRAGEND_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 30064771079;
pub const KEYC_MOUSEDRAGEND11_SCROLLBAR_UP: C2RustUnnamed_36 = 30064773894;
pub const KEYC_MOUSEDRAGEND10_SCROLLBAR_UP: C2RustUnnamed_36 = 30064773638;
pub const KEYC_MOUSEDRAGEND9_SCROLLBAR_UP: C2RustUnnamed_36 = 30064773382;
pub const KEYC_MOUSEDRAGEND8_SCROLLBAR_UP: C2RustUnnamed_36 = 30064773126;
pub const KEYC_MOUSEDRAGEND7_SCROLLBAR_UP: C2RustUnnamed_36 = 30064772870;
pub const KEYC_MOUSEDRAGEND6_SCROLLBAR_UP: C2RustUnnamed_36 = 30064772614;
pub const KEYC_MOUSEDRAGEND3_SCROLLBAR_UP: C2RustUnnamed_36 = 30064771846;
pub const KEYC_MOUSEDRAGEND2_SCROLLBAR_UP: C2RustUnnamed_36 = 30064771590;
pub const KEYC_MOUSEDRAGEND1_SCROLLBAR_UP: C2RustUnnamed_36 = 30064771334;
pub const KEYC_MOUSEDRAGEND_SCROLLBAR_UP: C2RustUnnamed_36 = 30064771078;
pub const KEYC_MOUSEDRAGEND11_BORDER: C2RustUnnamed_36 = 30064773893;
pub const KEYC_MOUSEDRAGEND10_BORDER: C2RustUnnamed_36 = 30064773637;
pub const KEYC_MOUSEDRAGEND9_BORDER: C2RustUnnamed_36 = 30064773381;
pub const KEYC_MOUSEDRAGEND8_BORDER: C2RustUnnamed_36 = 30064773125;
pub const KEYC_MOUSEDRAGEND7_BORDER: C2RustUnnamed_36 = 30064772869;
pub const KEYC_MOUSEDRAGEND6_BORDER: C2RustUnnamed_36 = 30064772613;
pub const KEYC_MOUSEDRAGEND3_BORDER: C2RustUnnamed_36 = 30064771845;
pub const KEYC_MOUSEDRAGEND2_BORDER: C2RustUnnamed_36 = 30064771589;
pub const KEYC_MOUSEDRAGEND1_BORDER: C2RustUnnamed_36 = 30064771333;
pub const KEYC_MOUSEDRAGEND_BORDER: C2RustUnnamed_36 = 30064771077;
pub const KEYC_MOUSEDRAGEND11_STATUS_DEFAULT: C2RustUnnamed_36 = 30064773892;
pub const KEYC_MOUSEDRAGEND10_STATUS_DEFAULT: C2RustUnnamed_36 = 30064773636;
pub const KEYC_MOUSEDRAGEND9_STATUS_DEFAULT: C2RustUnnamed_36 = 30064773380;
pub const KEYC_MOUSEDRAGEND8_STATUS_DEFAULT: C2RustUnnamed_36 = 30064773124;
pub const KEYC_MOUSEDRAGEND7_STATUS_DEFAULT: C2RustUnnamed_36 = 30064772868;
pub const KEYC_MOUSEDRAGEND6_STATUS_DEFAULT: C2RustUnnamed_36 = 30064772612;
pub const KEYC_MOUSEDRAGEND3_STATUS_DEFAULT: C2RustUnnamed_36 = 30064771844;
pub const KEYC_MOUSEDRAGEND2_STATUS_DEFAULT: C2RustUnnamed_36 = 30064771588;
pub const KEYC_MOUSEDRAGEND1_STATUS_DEFAULT: C2RustUnnamed_36 = 30064771332;
pub const KEYC_MOUSEDRAGEND_STATUS_DEFAULT: C2RustUnnamed_36 = 30064771076;
pub const KEYC_MOUSEDRAGEND11_STATUS_RIGHT: C2RustUnnamed_36 = 30064773891;
pub const KEYC_MOUSEDRAGEND10_STATUS_RIGHT: C2RustUnnamed_36 = 30064773635;
pub const KEYC_MOUSEDRAGEND9_STATUS_RIGHT: C2RustUnnamed_36 = 30064773379;
pub const KEYC_MOUSEDRAGEND8_STATUS_RIGHT: C2RustUnnamed_36 = 30064773123;
pub const KEYC_MOUSEDRAGEND7_STATUS_RIGHT: C2RustUnnamed_36 = 30064772867;
pub const KEYC_MOUSEDRAGEND6_STATUS_RIGHT: C2RustUnnamed_36 = 30064772611;
pub const KEYC_MOUSEDRAGEND3_STATUS_RIGHT: C2RustUnnamed_36 = 30064771843;
pub const KEYC_MOUSEDRAGEND2_STATUS_RIGHT: C2RustUnnamed_36 = 30064771587;
pub const KEYC_MOUSEDRAGEND1_STATUS_RIGHT: C2RustUnnamed_36 = 30064771331;
pub const KEYC_MOUSEDRAGEND_STATUS_RIGHT: C2RustUnnamed_36 = 30064771075;
pub const KEYC_MOUSEDRAGEND11_STATUS_LEFT: C2RustUnnamed_36 = 30064773890;
pub const KEYC_MOUSEDRAGEND10_STATUS_LEFT: C2RustUnnamed_36 = 30064773634;
pub const KEYC_MOUSEDRAGEND9_STATUS_LEFT: C2RustUnnamed_36 = 30064773378;
pub const KEYC_MOUSEDRAGEND8_STATUS_LEFT: C2RustUnnamed_36 = 30064773122;
pub const KEYC_MOUSEDRAGEND7_STATUS_LEFT: C2RustUnnamed_36 = 30064772866;
pub const KEYC_MOUSEDRAGEND6_STATUS_LEFT: C2RustUnnamed_36 = 30064772610;
pub const KEYC_MOUSEDRAGEND3_STATUS_LEFT: C2RustUnnamed_36 = 30064771842;
pub const KEYC_MOUSEDRAGEND2_STATUS_LEFT: C2RustUnnamed_36 = 30064771586;
pub const KEYC_MOUSEDRAGEND1_STATUS_LEFT: C2RustUnnamed_36 = 30064771330;
pub const KEYC_MOUSEDRAGEND_STATUS_LEFT: C2RustUnnamed_36 = 30064771074;
pub const KEYC_MOUSEDRAGEND11_STATUS: C2RustUnnamed_36 = 30064773889;
pub const KEYC_MOUSEDRAGEND10_STATUS: C2RustUnnamed_36 = 30064773633;
pub const KEYC_MOUSEDRAGEND9_STATUS: C2RustUnnamed_36 = 30064773377;
pub const KEYC_MOUSEDRAGEND8_STATUS: C2RustUnnamed_36 = 30064773121;
pub const KEYC_MOUSEDRAGEND7_STATUS: C2RustUnnamed_36 = 30064772865;
pub const KEYC_MOUSEDRAGEND6_STATUS: C2RustUnnamed_36 = 30064772609;
pub const KEYC_MOUSEDRAGEND3_STATUS: C2RustUnnamed_36 = 30064771841;
pub const KEYC_MOUSEDRAGEND2_STATUS: C2RustUnnamed_36 = 30064771585;
pub const KEYC_MOUSEDRAGEND1_STATUS: C2RustUnnamed_36 = 30064771329;
pub const KEYC_MOUSEDRAGEND_STATUS: C2RustUnnamed_36 = 30064771073;
pub const KEYC_MOUSEDRAGEND11_PANE: C2RustUnnamed_36 = 30064773888;
pub const KEYC_MOUSEDRAGEND10_PANE: C2RustUnnamed_36 = 30064773632;
pub const KEYC_MOUSEDRAGEND9_PANE: C2RustUnnamed_36 = 30064773376;
pub const KEYC_MOUSEDRAGEND8_PANE: C2RustUnnamed_36 = 30064773120;
pub const KEYC_MOUSEDRAGEND7_PANE: C2RustUnnamed_36 = 30064772864;
pub const KEYC_MOUSEDRAGEND6_PANE: C2RustUnnamed_36 = 30064772608;
pub const KEYC_MOUSEDRAGEND3_PANE: C2RustUnnamed_36 = 30064771840;
pub const KEYC_MOUSEDRAGEND2_PANE: C2RustUnnamed_36 = 30064771584;
pub const KEYC_MOUSEDRAGEND1_PANE: C2RustUnnamed_36 = 30064771328;
pub const KEYC_MOUSEDRAGEND_PANE: C2RustUnnamed_36 = 30064771072;
pub const KEYC_MOUSEDRAG11_CONTROL9: C2RustUnnamed_36 = 25769806611;
pub const KEYC_MOUSEDRAG10_CONTROL9: C2RustUnnamed_36 = 25769806355;
pub const KEYC_MOUSEDRAG9_CONTROL9: C2RustUnnamed_36 = 25769806099;
pub const KEYC_MOUSEDRAG8_CONTROL9: C2RustUnnamed_36 = 25769805843;
pub const KEYC_MOUSEDRAG7_CONTROL9: C2RustUnnamed_36 = 25769805587;
pub const KEYC_MOUSEDRAG6_CONTROL9: C2RustUnnamed_36 = 25769805331;
pub const KEYC_MOUSEDRAG3_CONTROL9: C2RustUnnamed_36 = 25769804563;
pub const KEYC_MOUSEDRAG2_CONTROL9: C2RustUnnamed_36 = 25769804307;
pub const KEYC_MOUSEDRAG1_CONTROL9: C2RustUnnamed_36 = 25769804051;
pub const KEYC_MOUSEDRAG_CONTROL9: C2RustUnnamed_36 = 25769803795;
pub const KEYC_MOUSEDRAG11_CONTROL8: C2RustUnnamed_36 = 25769806610;
pub const KEYC_MOUSEDRAG10_CONTROL8: C2RustUnnamed_36 = 25769806354;
pub const KEYC_MOUSEDRAG9_CONTROL8: C2RustUnnamed_36 = 25769806098;
pub const KEYC_MOUSEDRAG8_CONTROL8: C2RustUnnamed_36 = 25769805842;
pub const KEYC_MOUSEDRAG7_CONTROL8: C2RustUnnamed_36 = 25769805586;
pub const KEYC_MOUSEDRAG6_CONTROL8: C2RustUnnamed_36 = 25769805330;
pub const KEYC_MOUSEDRAG3_CONTROL8: C2RustUnnamed_36 = 25769804562;
pub const KEYC_MOUSEDRAG2_CONTROL8: C2RustUnnamed_36 = 25769804306;
pub const KEYC_MOUSEDRAG1_CONTROL8: C2RustUnnamed_36 = 25769804050;
pub const KEYC_MOUSEDRAG_CONTROL8: C2RustUnnamed_36 = 25769803794;
pub const KEYC_MOUSEDRAG11_CONTROL7: C2RustUnnamed_36 = 25769806609;
pub const KEYC_MOUSEDRAG10_CONTROL7: C2RustUnnamed_36 = 25769806353;
pub const KEYC_MOUSEDRAG9_CONTROL7: C2RustUnnamed_36 = 25769806097;
pub const KEYC_MOUSEDRAG8_CONTROL7: C2RustUnnamed_36 = 25769805841;
pub const KEYC_MOUSEDRAG7_CONTROL7: C2RustUnnamed_36 = 25769805585;
pub const KEYC_MOUSEDRAG6_CONTROL7: C2RustUnnamed_36 = 25769805329;
pub const KEYC_MOUSEDRAG3_CONTROL7: C2RustUnnamed_36 = 25769804561;
pub const KEYC_MOUSEDRAG2_CONTROL7: C2RustUnnamed_36 = 25769804305;
pub const KEYC_MOUSEDRAG1_CONTROL7: C2RustUnnamed_36 = 25769804049;
pub const KEYC_MOUSEDRAG_CONTROL7: C2RustUnnamed_36 = 25769803793;
pub const KEYC_MOUSEDRAG11_CONTROL6: C2RustUnnamed_36 = 25769806608;
pub const KEYC_MOUSEDRAG10_CONTROL6: C2RustUnnamed_36 = 25769806352;
pub const KEYC_MOUSEDRAG9_CONTROL6: C2RustUnnamed_36 = 25769806096;
pub const KEYC_MOUSEDRAG8_CONTROL6: C2RustUnnamed_36 = 25769805840;
pub const KEYC_MOUSEDRAG7_CONTROL6: C2RustUnnamed_36 = 25769805584;
pub const KEYC_MOUSEDRAG6_CONTROL6: C2RustUnnamed_36 = 25769805328;
pub const KEYC_MOUSEDRAG3_CONTROL6: C2RustUnnamed_36 = 25769804560;
pub const KEYC_MOUSEDRAG2_CONTROL6: C2RustUnnamed_36 = 25769804304;
pub const KEYC_MOUSEDRAG1_CONTROL6: C2RustUnnamed_36 = 25769804048;
pub const KEYC_MOUSEDRAG_CONTROL6: C2RustUnnamed_36 = 25769803792;
pub const KEYC_MOUSEDRAG11_CONTROL5: C2RustUnnamed_36 = 25769806607;
pub const KEYC_MOUSEDRAG10_CONTROL5: C2RustUnnamed_36 = 25769806351;
pub const KEYC_MOUSEDRAG9_CONTROL5: C2RustUnnamed_36 = 25769806095;
pub const KEYC_MOUSEDRAG8_CONTROL5: C2RustUnnamed_36 = 25769805839;
pub const KEYC_MOUSEDRAG7_CONTROL5: C2RustUnnamed_36 = 25769805583;
pub const KEYC_MOUSEDRAG6_CONTROL5: C2RustUnnamed_36 = 25769805327;
pub const KEYC_MOUSEDRAG3_CONTROL5: C2RustUnnamed_36 = 25769804559;
pub const KEYC_MOUSEDRAG2_CONTROL5: C2RustUnnamed_36 = 25769804303;
pub const KEYC_MOUSEDRAG1_CONTROL5: C2RustUnnamed_36 = 25769804047;
pub const KEYC_MOUSEDRAG_CONTROL5: C2RustUnnamed_36 = 25769803791;
pub const KEYC_MOUSEDRAG11_CONTROL4: C2RustUnnamed_36 = 25769806606;
pub const KEYC_MOUSEDRAG10_CONTROL4: C2RustUnnamed_36 = 25769806350;
pub const KEYC_MOUSEDRAG9_CONTROL4: C2RustUnnamed_36 = 25769806094;
pub const KEYC_MOUSEDRAG8_CONTROL4: C2RustUnnamed_36 = 25769805838;
pub const KEYC_MOUSEDRAG7_CONTROL4: C2RustUnnamed_36 = 25769805582;
pub const KEYC_MOUSEDRAG6_CONTROL4: C2RustUnnamed_36 = 25769805326;
pub const KEYC_MOUSEDRAG3_CONTROL4: C2RustUnnamed_36 = 25769804558;
pub const KEYC_MOUSEDRAG2_CONTROL4: C2RustUnnamed_36 = 25769804302;
pub const KEYC_MOUSEDRAG1_CONTROL4: C2RustUnnamed_36 = 25769804046;
pub const KEYC_MOUSEDRAG_CONTROL4: C2RustUnnamed_36 = 25769803790;
pub const KEYC_MOUSEDRAG11_CONTROL3: C2RustUnnamed_36 = 25769806605;
pub const KEYC_MOUSEDRAG10_CONTROL3: C2RustUnnamed_36 = 25769806349;
pub const KEYC_MOUSEDRAG9_CONTROL3: C2RustUnnamed_36 = 25769806093;
pub const KEYC_MOUSEDRAG8_CONTROL3: C2RustUnnamed_36 = 25769805837;
pub const KEYC_MOUSEDRAG7_CONTROL3: C2RustUnnamed_36 = 25769805581;
pub const KEYC_MOUSEDRAG6_CONTROL3: C2RustUnnamed_36 = 25769805325;
pub const KEYC_MOUSEDRAG3_CONTROL3: C2RustUnnamed_36 = 25769804557;
pub const KEYC_MOUSEDRAG2_CONTROL3: C2RustUnnamed_36 = 25769804301;
pub const KEYC_MOUSEDRAG1_CONTROL3: C2RustUnnamed_36 = 25769804045;
pub const KEYC_MOUSEDRAG_CONTROL3: C2RustUnnamed_36 = 25769803789;
pub const KEYC_MOUSEDRAG11_CONTROL2: C2RustUnnamed_36 = 25769806604;
pub const KEYC_MOUSEDRAG10_CONTROL2: C2RustUnnamed_36 = 25769806348;
pub const KEYC_MOUSEDRAG9_CONTROL2: C2RustUnnamed_36 = 25769806092;
pub const KEYC_MOUSEDRAG8_CONTROL2: C2RustUnnamed_36 = 25769805836;
pub const KEYC_MOUSEDRAG7_CONTROL2: C2RustUnnamed_36 = 25769805580;
pub const KEYC_MOUSEDRAG6_CONTROL2: C2RustUnnamed_36 = 25769805324;
pub const KEYC_MOUSEDRAG3_CONTROL2: C2RustUnnamed_36 = 25769804556;
pub const KEYC_MOUSEDRAG2_CONTROL2: C2RustUnnamed_36 = 25769804300;
pub const KEYC_MOUSEDRAG1_CONTROL2: C2RustUnnamed_36 = 25769804044;
pub const KEYC_MOUSEDRAG_CONTROL2: C2RustUnnamed_36 = 25769803788;
pub const KEYC_MOUSEDRAG11_CONTROL1: C2RustUnnamed_36 = 25769806603;
pub const KEYC_MOUSEDRAG10_CONTROL1: C2RustUnnamed_36 = 25769806347;
pub const KEYC_MOUSEDRAG9_CONTROL1: C2RustUnnamed_36 = 25769806091;
pub const KEYC_MOUSEDRAG8_CONTROL1: C2RustUnnamed_36 = 25769805835;
pub const KEYC_MOUSEDRAG7_CONTROL1: C2RustUnnamed_36 = 25769805579;
pub const KEYC_MOUSEDRAG6_CONTROL1: C2RustUnnamed_36 = 25769805323;
pub const KEYC_MOUSEDRAG3_CONTROL1: C2RustUnnamed_36 = 25769804555;
pub const KEYC_MOUSEDRAG2_CONTROL1: C2RustUnnamed_36 = 25769804299;
pub const KEYC_MOUSEDRAG1_CONTROL1: C2RustUnnamed_36 = 25769804043;
pub const KEYC_MOUSEDRAG_CONTROL1: C2RustUnnamed_36 = 25769803787;
pub const KEYC_MOUSEDRAG11_CONTROL0: C2RustUnnamed_36 = 25769806602;
pub const KEYC_MOUSEDRAG10_CONTROL0: C2RustUnnamed_36 = 25769806346;
pub const KEYC_MOUSEDRAG9_CONTROL0: C2RustUnnamed_36 = 25769806090;
pub const KEYC_MOUSEDRAG8_CONTROL0: C2RustUnnamed_36 = 25769805834;
pub const KEYC_MOUSEDRAG7_CONTROL0: C2RustUnnamed_36 = 25769805578;
pub const KEYC_MOUSEDRAG6_CONTROL0: C2RustUnnamed_36 = 25769805322;
pub const KEYC_MOUSEDRAG3_CONTROL0: C2RustUnnamed_36 = 25769804554;
pub const KEYC_MOUSEDRAG2_CONTROL0: C2RustUnnamed_36 = 25769804298;
pub const KEYC_MOUSEDRAG1_CONTROL0: C2RustUnnamed_36 = 25769804042;
pub const KEYC_MOUSEDRAG_CONTROL0: C2RustUnnamed_36 = 25769803786;
pub const KEYC_MOUSEDRAG11_EMPTY: C2RustUnnamed_36 = 25769806601;
pub const KEYC_MOUSEDRAG10_EMPTY: C2RustUnnamed_36 = 25769806345;
pub const KEYC_MOUSEDRAG9_EMPTY: C2RustUnnamed_36 = 25769806089;
pub const KEYC_MOUSEDRAG8_EMPTY: C2RustUnnamed_36 = 25769805833;
pub const KEYC_MOUSEDRAG7_EMPTY: C2RustUnnamed_36 = 25769805577;
pub const KEYC_MOUSEDRAG6_EMPTY: C2RustUnnamed_36 = 25769805321;
pub const KEYC_MOUSEDRAG3_EMPTY: C2RustUnnamed_36 = 25769804553;
pub const KEYC_MOUSEDRAG2_EMPTY: C2RustUnnamed_36 = 25769804297;
pub const KEYC_MOUSEDRAG1_EMPTY: C2RustUnnamed_36 = 25769804041;
pub const KEYC_MOUSEDRAG_EMPTY: C2RustUnnamed_36 = 25769803785;
pub const KEYC_MOUSEDRAG11_SCROLLBAR_DOWN: C2RustUnnamed_36 = 25769806600;
pub const KEYC_MOUSEDRAG10_SCROLLBAR_DOWN: C2RustUnnamed_36 = 25769806344;
pub const KEYC_MOUSEDRAG9_SCROLLBAR_DOWN: C2RustUnnamed_36 = 25769806088;
pub const KEYC_MOUSEDRAG8_SCROLLBAR_DOWN: C2RustUnnamed_36 = 25769805832;
pub const KEYC_MOUSEDRAG7_SCROLLBAR_DOWN: C2RustUnnamed_36 = 25769805576;
pub const KEYC_MOUSEDRAG6_SCROLLBAR_DOWN: C2RustUnnamed_36 = 25769805320;
pub const KEYC_MOUSEDRAG3_SCROLLBAR_DOWN: C2RustUnnamed_36 = 25769804552;
pub const KEYC_MOUSEDRAG2_SCROLLBAR_DOWN: C2RustUnnamed_36 = 25769804296;
pub const KEYC_MOUSEDRAG1_SCROLLBAR_DOWN: C2RustUnnamed_36 = 25769804040;
pub const KEYC_MOUSEDRAG_SCROLLBAR_DOWN: C2RustUnnamed_36 = 25769803784;
pub const KEYC_MOUSEDRAG11_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 25769806599;
pub const KEYC_MOUSEDRAG10_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 25769806343;
pub const KEYC_MOUSEDRAG9_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 25769806087;
pub const KEYC_MOUSEDRAG8_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 25769805831;
pub const KEYC_MOUSEDRAG7_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 25769805575;
pub const KEYC_MOUSEDRAG6_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 25769805319;
pub const KEYC_MOUSEDRAG3_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 25769804551;
pub const KEYC_MOUSEDRAG2_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 25769804295;
pub const KEYC_MOUSEDRAG1_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 25769804039;
pub const KEYC_MOUSEDRAG_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 25769803783;
pub const KEYC_MOUSEDRAG11_SCROLLBAR_UP: C2RustUnnamed_36 = 25769806598;
pub const KEYC_MOUSEDRAG10_SCROLLBAR_UP: C2RustUnnamed_36 = 25769806342;
pub const KEYC_MOUSEDRAG9_SCROLLBAR_UP: C2RustUnnamed_36 = 25769806086;
pub const KEYC_MOUSEDRAG8_SCROLLBAR_UP: C2RustUnnamed_36 = 25769805830;
pub const KEYC_MOUSEDRAG7_SCROLLBAR_UP: C2RustUnnamed_36 = 25769805574;
pub const KEYC_MOUSEDRAG6_SCROLLBAR_UP: C2RustUnnamed_36 = 25769805318;
pub const KEYC_MOUSEDRAG3_SCROLLBAR_UP: C2RustUnnamed_36 = 25769804550;
pub const KEYC_MOUSEDRAG2_SCROLLBAR_UP: C2RustUnnamed_36 = 25769804294;
pub const KEYC_MOUSEDRAG1_SCROLLBAR_UP: C2RustUnnamed_36 = 25769804038;
pub const KEYC_MOUSEDRAG_SCROLLBAR_UP: C2RustUnnamed_36 = 25769803782;
pub const KEYC_MOUSEDRAG11_BORDER: C2RustUnnamed_36 = 25769806597;
pub const KEYC_MOUSEDRAG10_BORDER: C2RustUnnamed_36 = 25769806341;
pub const KEYC_MOUSEDRAG9_BORDER: C2RustUnnamed_36 = 25769806085;
pub const KEYC_MOUSEDRAG8_BORDER: C2RustUnnamed_36 = 25769805829;
pub const KEYC_MOUSEDRAG7_BORDER: C2RustUnnamed_36 = 25769805573;
pub const KEYC_MOUSEDRAG6_BORDER: C2RustUnnamed_36 = 25769805317;
pub const KEYC_MOUSEDRAG3_BORDER: C2RustUnnamed_36 = 25769804549;
pub const KEYC_MOUSEDRAG2_BORDER: C2RustUnnamed_36 = 25769804293;
pub const KEYC_MOUSEDRAG1_BORDER: C2RustUnnamed_36 = 25769804037;
pub const KEYC_MOUSEDRAG_BORDER: C2RustUnnamed_36 = 25769803781;
pub const KEYC_MOUSEDRAG11_STATUS_DEFAULT: C2RustUnnamed_36 = 25769806596;
pub const KEYC_MOUSEDRAG10_STATUS_DEFAULT: C2RustUnnamed_36 = 25769806340;
pub const KEYC_MOUSEDRAG9_STATUS_DEFAULT: C2RustUnnamed_36 = 25769806084;
pub const KEYC_MOUSEDRAG8_STATUS_DEFAULT: C2RustUnnamed_36 = 25769805828;
pub const KEYC_MOUSEDRAG7_STATUS_DEFAULT: C2RustUnnamed_36 = 25769805572;
pub const KEYC_MOUSEDRAG6_STATUS_DEFAULT: C2RustUnnamed_36 = 25769805316;
pub const KEYC_MOUSEDRAG3_STATUS_DEFAULT: C2RustUnnamed_36 = 25769804548;
pub const KEYC_MOUSEDRAG2_STATUS_DEFAULT: C2RustUnnamed_36 = 25769804292;
pub const KEYC_MOUSEDRAG1_STATUS_DEFAULT: C2RustUnnamed_36 = 25769804036;
pub const KEYC_MOUSEDRAG_STATUS_DEFAULT: C2RustUnnamed_36 = 25769803780;
pub const KEYC_MOUSEDRAG11_STATUS_RIGHT: C2RustUnnamed_36 = 25769806595;
pub const KEYC_MOUSEDRAG10_STATUS_RIGHT: C2RustUnnamed_36 = 25769806339;
pub const KEYC_MOUSEDRAG9_STATUS_RIGHT: C2RustUnnamed_36 = 25769806083;
pub const KEYC_MOUSEDRAG8_STATUS_RIGHT: C2RustUnnamed_36 = 25769805827;
pub const KEYC_MOUSEDRAG7_STATUS_RIGHT: C2RustUnnamed_36 = 25769805571;
pub const KEYC_MOUSEDRAG6_STATUS_RIGHT: C2RustUnnamed_36 = 25769805315;
pub const KEYC_MOUSEDRAG3_STATUS_RIGHT: C2RustUnnamed_36 = 25769804547;
pub const KEYC_MOUSEDRAG2_STATUS_RIGHT: C2RustUnnamed_36 = 25769804291;
pub const KEYC_MOUSEDRAG1_STATUS_RIGHT: C2RustUnnamed_36 = 25769804035;
pub const KEYC_MOUSEDRAG_STATUS_RIGHT: C2RustUnnamed_36 = 25769803779;
pub const KEYC_MOUSEDRAG11_STATUS_LEFT: C2RustUnnamed_36 = 25769806594;
pub const KEYC_MOUSEDRAG10_STATUS_LEFT: C2RustUnnamed_36 = 25769806338;
pub const KEYC_MOUSEDRAG9_STATUS_LEFT: C2RustUnnamed_36 = 25769806082;
pub const KEYC_MOUSEDRAG8_STATUS_LEFT: C2RustUnnamed_36 = 25769805826;
pub const KEYC_MOUSEDRAG7_STATUS_LEFT: C2RustUnnamed_36 = 25769805570;
pub const KEYC_MOUSEDRAG6_STATUS_LEFT: C2RustUnnamed_36 = 25769805314;
pub const KEYC_MOUSEDRAG3_STATUS_LEFT: C2RustUnnamed_36 = 25769804546;
pub const KEYC_MOUSEDRAG2_STATUS_LEFT: C2RustUnnamed_36 = 25769804290;
pub const KEYC_MOUSEDRAG1_STATUS_LEFT: C2RustUnnamed_36 = 25769804034;
pub const KEYC_MOUSEDRAG_STATUS_LEFT: C2RustUnnamed_36 = 25769803778;
pub const KEYC_MOUSEDRAG11_STATUS: C2RustUnnamed_36 = 25769806593;
pub const KEYC_MOUSEDRAG10_STATUS: C2RustUnnamed_36 = 25769806337;
pub const KEYC_MOUSEDRAG9_STATUS: C2RustUnnamed_36 = 25769806081;
pub const KEYC_MOUSEDRAG8_STATUS: C2RustUnnamed_36 = 25769805825;
pub const KEYC_MOUSEDRAG7_STATUS: C2RustUnnamed_36 = 25769805569;
pub const KEYC_MOUSEDRAG6_STATUS: C2RustUnnamed_36 = 25769805313;
pub const KEYC_MOUSEDRAG3_STATUS: C2RustUnnamed_36 = 25769804545;
pub const KEYC_MOUSEDRAG2_STATUS: C2RustUnnamed_36 = 25769804289;
pub const KEYC_MOUSEDRAG1_STATUS: C2RustUnnamed_36 = 25769804033;
pub const KEYC_MOUSEDRAG_STATUS: C2RustUnnamed_36 = 25769803777;
pub const KEYC_MOUSEDRAG11_PANE: C2RustUnnamed_36 = 25769806592;
pub const KEYC_MOUSEDRAG10_PANE: C2RustUnnamed_36 = 25769806336;
pub const KEYC_MOUSEDRAG9_PANE: C2RustUnnamed_36 = 25769806080;
pub const KEYC_MOUSEDRAG8_PANE: C2RustUnnamed_36 = 25769805824;
pub const KEYC_MOUSEDRAG7_PANE: C2RustUnnamed_36 = 25769805568;
pub const KEYC_MOUSEDRAG6_PANE: C2RustUnnamed_36 = 25769805312;
pub const KEYC_MOUSEDRAG3_PANE: C2RustUnnamed_36 = 25769804544;
pub const KEYC_MOUSEDRAG2_PANE: C2RustUnnamed_36 = 25769804288;
pub const KEYC_MOUSEDRAG1_PANE: C2RustUnnamed_36 = 25769804032;
pub const KEYC_MOUSEDRAG_PANE: C2RustUnnamed_36 = 25769803776;
pub const KEYC_MOUSEUP11_CONTROL9: C2RustUnnamed_36 = 21474839315;
pub const KEYC_MOUSEUP10_CONTROL9: C2RustUnnamed_36 = 21474839059;
pub const KEYC_MOUSEUP9_CONTROL9: C2RustUnnamed_36 = 21474838803;
pub const KEYC_MOUSEUP8_CONTROL9: C2RustUnnamed_36 = 21474838547;
pub const KEYC_MOUSEUP7_CONTROL9: C2RustUnnamed_36 = 21474838291;
pub const KEYC_MOUSEUP6_CONTROL9: C2RustUnnamed_36 = 21474838035;
pub const KEYC_MOUSEUP3_CONTROL9: C2RustUnnamed_36 = 21474837267;
pub const KEYC_MOUSEUP2_CONTROL9: C2RustUnnamed_36 = 21474837011;
pub const KEYC_MOUSEUP1_CONTROL9: C2RustUnnamed_36 = 21474836755;
pub const KEYC_MOUSEUP_CONTROL9: C2RustUnnamed_36 = 21474836499;
pub const KEYC_MOUSEUP11_CONTROL8: C2RustUnnamed_36 = 21474839314;
pub const KEYC_MOUSEUP10_CONTROL8: C2RustUnnamed_36 = 21474839058;
pub const KEYC_MOUSEUP9_CONTROL8: C2RustUnnamed_36 = 21474838802;
pub const KEYC_MOUSEUP8_CONTROL8: C2RustUnnamed_36 = 21474838546;
pub const KEYC_MOUSEUP7_CONTROL8: C2RustUnnamed_36 = 21474838290;
pub const KEYC_MOUSEUP6_CONTROL8: C2RustUnnamed_36 = 21474838034;
pub const KEYC_MOUSEUP3_CONTROL8: C2RustUnnamed_36 = 21474837266;
pub const KEYC_MOUSEUP2_CONTROL8: C2RustUnnamed_36 = 21474837010;
pub const KEYC_MOUSEUP1_CONTROL8: C2RustUnnamed_36 = 21474836754;
pub const KEYC_MOUSEUP_CONTROL8: C2RustUnnamed_36 = 21474836498;
pub const KEYC_MOUSEUP11_CONTROL7: C2RustUnnamed_36 = 21474839313;
pub const KEYC_MOUSEUP10_CONTROL7: C2RustUnnamed_36 = 21474839057;
pub const KEYC_MOUSEUP9_CONTROL7: C2RustUnnamed_36 = 21474838801;
pub const KEYC_MOUSEUP8_CONTROL7: C2RustUnnamed_36 = 21474838545;
pub const KEYC_MOUSEUP7_CONTROL7: C2RustUnnamed_36 = 21474838289;
pub const KEYC_MOUSEUP6_CONTROL7: C2RustUnnamed_36 = 21474838033;
pub const KEYC_MOUSEUP3_CONTROL7: C2RustUnnamed_36 = 21474837265;
pub const KEYC_MOUSEUP2_CONTROL7: C2RustUnnamed_36 = 21474837009;
pub const KEYC_MOUSEUP1_CONTROL7: C2RustUnnamed_36 = 21474836753;
pub const KEYC_MOUSEUP_CONTROL7: C2RustUnnamed_36 = 21474836497;
pub const KEYC_MOUSEUP11_CONTROL6: C2RustUnnamed_36 = 21474839312;
pub const KEYC_MOUSEUP10_CONTROL6: C2RustUnnamed_36 = 21474839056;
pub const KEYC_MOUSEUP9_CONTROL6: C2RustUnnamed_36 = 21474838800;
pub const KEYC_MOUSEUP8_CONTROL6: C2RustUnnamed_36 = 21474838544;
pub const KEYC_MOUSEUP7_CONTROL6: C2RustUnnamed_36 = 21474838288;
pub const KEYC_MOUSEUP6_CONTROL6: C2RustUnnamed_36 = 21474838032;
pub const KEYC_MOUSEUP3_CONTROL6: C2RustUnnamed_36 = 21474837264;
pub const KEYC_MOUSEUP2_CONTROL6: C2RustUnnamed_36 = 21474837008;
pub const KEYC_MOUSEUP1_CONTROL6: C2RustUnnamed_36 = 21474836752;
pub const KEYC_MOUSEUP_CONTROL6: C2RustUnnamed_36 = 21474836496;
pub const KEYC_MOUSEUP11_CONTROL5: C2RustUnnamed_36 = 21474839311;
pub const KEYC_MOUSEUP10_CONTROL5: C2RustUnnamed_36 = 21474839055;
pub const KEYC_MOUSEUP9_CONTROL5: C2RustUnnamed_36 = 21474838799;
pub const KEYC_MOUSEUP8_CONTROL5: C2RustUnnamed_36 = 21474838543;
pub const KEYC_MOUSEUP7_CONTROL5: C2RustUnnamed_36 = 21474838287;
pub const KEYC_MOUSEUP6_CONTROL5: C2RustUnnamed_36 = 21474838031;
pub const KEYC_MOUSEUP3_CONTROL5: C2RustUnnamed_36 = 21474837263;
pub const KEYC_MOUSEUP2_CONTROL5: C2RustUnnamed_36 = 21474837007;
pub const KEYC_MOUSEUP1_CONTROL5: C2RustUnnamed_36 = 21474836751;
pub const KEYC_MOUSEUP_CONTROL5: C2RustUnnamed_36 = 21474836495;
pub const KEYC_MOUSEUP11_CONTROL4: C2RustUnnamed_36 = 21474839310;
pub const KEYC_MOUSEUP10_CONTROL4: C2RustUnnamed_36 = 21474839054;
pub const KEYC_MOUSEUP9_CONTROL4: C2RustUnnamed_36 = 21474838798;
pub const KEYC_MOUSEUP8_CONTROL4: C2RustUnnamed_36 = 21474838542;
pub const KEYC_MOUSEUP7_CONTROL4: C2RustUnnamed_36 = 21474838286;
pub const KEYC_MOUSEUP6_CONTROL4: C2RustUnnamed_36 = 21474838030;
pub const KEYC_MOUSEUP3_CONTROL4: C2RustUnnamed_36 = 21474837262;
pub const KEYC_MOUSEUP2_CONTROL4: C2RustUnnamed_36 = 21474837006;
pub const KEYC_MOUSEUP1_CONTROL4: C2RustUnnamed_36 = 21474836750;
pub const KEYC_MOUSEUP_CONTROL4: C2RustUnnamed_36 = 21474836494;
pub const KEYC_MOUSEUP11_CONTROL3: C2RustUnnamed_36 = 21474839309;
pub const KEYC_MOUSEUP10_CONTROL3: C2RustUnnamed_36 = 21474839053;
pub const KEYC_MOUSEUP9_CONTROL3: C2RustUnnamed_36 = 21474838797;
pub const KEYC_MOUSEUP8_CONTROL3: C2RustUnnamed_36 = 21474838541;
pub const KEYC_MOUSEUP7_CONTROL3: C2RustUnnamed_36 = 21474838285;
pub const KEYC_MOUSEUP6_CONTROL3: C2RustUnnamed_36 = 21474838029;
pub const KEYC_MOUSEUP3_CONTROL3: C2RustUnnamed_36 = 21474837261;
pub const KEYC_MOUSEUP2_CONTROL3: C2RustUnnamed_36 = 21474837005;
pub const KEYC_MOUSEUP1_CONTROL3: C2RustUnnamed_36 = 21474836749;
pub const KEYC_MOUSEUP_CONTROL3: C2RustUnnamed_36 = 21474836493;
pub const KEYC_MOUSEUP11_CONTROL2: C2RustUnnamed_36 = 21474839308;
pub const KEYC_MOUSEUP10_CONTROL2: C2RustUnnamed_36 = 21474839052;
pub const KEYC_MOUSEUP9_CONTROL2: C2RustUnnamed_36 = 21474838796;
pub const KEYC_MOUSEUP8_CONTROL2: C2RustUnnamed_36 = 21474838540;
pub const KEYC_MOUSEUP7_CONTROL2: C2RustUnnamed_36 = 21474838284;
pub const KEYC_MOUSEUP6_CONTROL2: C2RustUnnamed_36 = 21474838028;
pub const KEYC_MOUSEUP3_CONTROL2: C2RustUnnamed_36 = 21474837260;
pub const KEYC_MOUSEUP2_CONTROL2: C2RustUnnamed_36 = 21474837004;
pub const KEYC_MOUSEUP1_CONTROL2: C2RustUnnamed_36 = 21474836748;
pub const KEYC_MOUSEUP_CONTROL2: C2RustUnnamed_36 = 21474836492;
pub const KEYC_MOUSEUP11_CONTROL1: C2RustUnnamed_36 = 21474839307;
pub const KEYC_MOUSEUP10_CONTROL1: C2RustUnnamed_36 = 21474839051;
pub const KEYC_MOUSEUP9_CONTROL1: C2RustUnnamed_36 = 21474838795;
pub const KEYC_MOUSEUP8_CONTROL1: C2RustUnnamed_36 = 21474838539;
pub const KEYC_MOUSEUP7_CONTROL1: C2RustUnnamed_36 = 21474838283;
pub const KEYC_MOUSEUP6_CONTROL1: C2RustUnnamed_36 = 21474838027;
pub const KEYC_MOUSEUP3_CONTROL1: C2RustUnnamed_36 = 21474837259;
pub const KEYC_MOUSEUP2_CONTROL1: C2RustUnnamed_36 = 21474837003;
pub const KEYC_MOUSEUP1_CONTROL1: C2RustUnnamed_36 = 21474836747;
pub const KEYC_MOUSEUP_CONTROL1: C2RustUnnamed_36 = 21474836491;
pub const KEYC_MOUSEUP11_CONTROL0: C2RustUnnamed_36 = 21474839306;
pub const KEYC_MOUSEUP10_CONTROL0: C2RustUnnamed_36 = 21474839050;
pub const KEYC_MOUSEUP9_CONTROL0: C2RustUnnamed_36 = 21474838794;
pub const KEYC_MOUSEUP8_CONTROL0: C2RustUnnamed_36 = 21474838538;
pub const KEYC_MOUSEUP7_CONTROL0: C2RustUnnamed_36 = 21474838282;
pub const KEYC_MOUSEUP6_CONTROL0: C2RustUnnamed_36 = 21474838026;
pub const KEYC_MOUSEUP3_CONTROL0: C2RustUnnamed_36 = 21474837258;
pub const KEYC_MOUSEUP2_CONTROL0: C2RustUnnamed_36 = 21474837002;
pub const KEYC_MOUSEUP1_CONTROL0: C2RustUnnamed_36 = 21474836746;
pub const KEYC_MOUSEUP_CONTROL0: C2RustUnnamed_36 = 21474836490;
pub const KEYC_MOUSEUP11_EMPTY: C2RustUnnamed_36 = 21474839305;
pub const KEYC_MOUSEUP10_EMPTY: C2RustUnnamed_36 = 21474839049;
pub const KEYC_MOUSEUP9_EMPTY: C2RustUnnamed_36 = 21474838793;
pub const KEYC_MOUSEUP8_EMPTY: C2RustUnnamed_36 = 21474838537;
pub const KEYC_MOUSEUP7_EMPTY: C2RustUnnamed_36 = 21474838281;
pub const KEYC_MOUSEUP6_EMPTY: C2RustUnnamed_36 = 21474838025;
pub const KEYC_MOUSEUP3_EMPTY: C2RustUnnamed_36 = 21474837257;
pub const KEYC_MOUSEUP2_EMPTY: C2RustUnnamed_36 = 21474837001;
pub const KEYC_MOUSEUP1_EMPTY: C2RustUnnamed_36 = 21474836745;
pub const KEYC_MOUSEUP_EMPTY: C2RustUnnamed_36 = 21474836489;
pub const KEYC_MOUSEUP11_SCROLLBAR_DOWN: C2RustUnnamed_36 = 21474839304;
pub const KEYC_MOUSEUP10_SCROLLBAR_DOWN: C2RustUnnamed_36 = 21474839048;
pub const KEYC_MOUSEUP9_SCROLLBAR_DOWN: C2RustUnnamed_36 = 21474838792;
pub const KEYC_MOUSEUP8_SCROLLBAR_DOWN: C2RustUnnamed_36 = 21474838536;
pub const KEYC_MOUSEUP7_SCROLLBAR_DOWN: C2RustUnnamed_36 = 21474838280;
pub const KEYC_MOUSEUP6_SCROLLBAR_DOWN: C2RustUnnamed_36 = 21474838024;
pub const KEYC_MOUSEUP3_SCROLLBAR_DOWN: C2RustUnnamed_36 = 21474837256;
pub const KEYC_MOUSEUP2_SCROLLBAR_DOWN: C2RustUnnamed_36 = 21474837000;
pub const KEYC_MOUSEUP1_SCROLLBAR_DOWN: C2RustUnnamed_36 = 21474836744;
pub const KEYC_MOUSEUP_SCROLLBAR_DOWN: C2RustUnnamed_36 = 21474836488;
pub const KEYC_MOUSEUP11_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 21474839303;
pub const KEYC_MOUSEUP10_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 21474839047;
pub const KEYC_MOUSEUP9_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 21474838791;
pub const KEYC_MOUSEUP8_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 21474838535;
pub const KEYC_MOUSEUP7_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 21474838279;
pub const KEYC_MOUSEUP6_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 21474838023;
pub const KEYC_MOUSEUP3_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 21474837255;
pub const KEYC_MOUSEUP2_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 21474836999;
pub const KEYC_MOUSEUP1_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 21474836743;
pub const KEYC_MOUSEUP_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 21474836487;
pub const KEYC_MOUSEUP11_SCROLLBAR_UP: C2RustUnnamed_36 = 21474839302;
pub const KEYC_MOUSEUP10_SCROLLBAR_UP: C2RustUnnamed_36 = 21474839046;
pub const KEYC_MOUSEUP9_SCROLLBAR_UP: C2RustUnnamed_36 = 21474838790;
pub const KEYC_MOUSEUP8_SCROLLBAR_UP: C2RustUnnamed_36 = 21474838534;
pub const KEYC_MOUSEUP7_SCROLLBAR_UP: C2RustUnnamed_36 = 21474838278;
pub const KEYC_MOUSEUP6_SCROLLBAR_UP: C2RustUnnamed_36 = 21474838022;
pub const KEYC_MOUSEUP3_SCROLLBAR_UP: C2RustUnnamed_36 = 21474837254;
pub const KEYC_MOUSEUP2_SCROLLBAR_UP: C2RustUnnamed_36 = 21474836998;
pub const KEYC_MOUSEUP1_SCROLLBAR_UP: C2RustUnnamed_36 = 21474836742;
pub const KEYC_MOUSEUP_SCROLLBAR_UP: C2RustUnnamed_36 = 21474836486;
pub const KEYC_MOUSEUP11_BORDER: C2RustUnnamed_36 = 21474839301;
pub const KEYC_MOUSEUP10_BORDER: C2RustUnnamed_36 = 21474839045;
pub const KEYC_MOUSEUP9_BORDER: C2RustUnnamed_36 = 21474838789;
pub const KEYC_MOUSEUP8_BORDER: C2RustUnnamed_36 = 21474838533;
pub const KEYC_MOUSEUP7_BORDER: C2RustUnnamed_36 = 21474838277;
pub const KEYC_MOUSEUP6_BORDER: C2RustUnnamed_36 = 21474838021;
pub const KEYC_MOUSEUP3_BORDER: C2RustUnnamed_36 = 21474837253;
pub const KEYC_MOUSEUP2_BORDER: C2RustUnnamed_36 = 21474836997;
pub const KEYC_MOUSEUP1_BORDER: C2RustUnnamed_36 = 21474836741;
pub const KEYC_MOUSEUP_BORDER: C2RustUnnamed_36 = 21474836485;
pub const KEYC_MOUSEUP11_STATUS_DEFAULT: C2RustUnnamed_36 = 21474839300;
pub const KEYC_MOUSEUP10_STATUS_DEFAULT: C2RustUnnamed_36 = 21474839044;
pub const KEYC_MOUSEUP9_STATUS_DEFAULT: C2RustUnnamed_36 = 21474838788;
pub const KEYC_MOUSEUP8_STATUS_DEFAULT: C2RustUnnamed_36 = 21474838532;
pub const KEYC_MOUSEUP7_STATUS_DEFAULT: C2RustUnnamed_36 = 21474838276;
pub const KEYC_MOUSEUP6_STATUS_DEFAULT: C2RustUnnamed_36 = 21474838020;
pub const KEYC_MOUSEUP3_STATUS_DEFAULT: C2RustUnnamed_36 = 21474837252;
pub const KEYC_MOUSEUP2_STATUS_DEFAULT: C2RustUnnamed_36 = 21474836996;
pub const KEYC_MOUSEUP1_STATUS_DEFAULT: C2RustUnnamed_36 = 21474836740;
pub const KEYC_MOUSEUP_STATUS_DEFAULT: C2RustUnnamed_36 = 21474836484;
pub const KEYC_MOUSEUP11_STATUS_RIGHT: C2RustUnnamed_36 = 21474839299;
pub const KEYC_MOUSEUP10_STATUS_RIGHT: C2RustUnnamed_36 = 21474839043;
pub const KEYC_MOUSEUP9_STATUS_RIGHT: C2RustUnnamed_36 = 21474838787;
pub const KEYC_MOUSEUP8_STATUS_RIGHT: C2RustUnnamed_36 = 21474838531;
pub const KEYC_MOUSEUP7_STATUS_RIGHT: C2RustUnnamed_36 = 21474838275;
pub const KEYC_MOUSEUP6_STATUS_RIGHT: C2RustUnnamed_36 = 21474838019;
pub const KEYC_MOUSEUP3_STATUS_RIGHT: C2RustUnnamed_36 = 21474837251;
pub const KEYC_MOUSEUP2_STATUS_RIGHT: C2RustUnnamed_36 = 21474836995;
pub const KEYC_MOUSEUP1_STATUS_RIGHT: C2RustUnnamed_36 = 21474836739;
pub const KEYC_MOUSEUP_STATUS_RIGHT: C2RustUnnamed_36 = 21474836483;
pub const KEYC_MOUSEUP11_STATUS_LEFT: C2RustUnnamed_36 = 21474839298;
pub const KEYC_MOUSEUP10_STATUS_LEFT: C2RustUnnamed_36 = 21474839042;
pub const KEYC_MOUSEUP9_STATUS_LEFT: C2RustUnnamed_36 = 21474838786;
pub const KEYC_MOUSEUP8_STATUS_LEFT: C2RustUnnamed_36 = 21474838530;
pub const KEYC_MOUSEUP7_STATUS_LEFT: C2RustUnnamed_36 = 21474838274;
pub const KEYC_MOUSEUP6_STATUS_LEFT: C2RustUnnamed_36 = 21474838018;
pub const KEYC_MOUSEUP3_STATUS_LEFT: C2RustUnnamed_36 = 21474837250;
pub const KEYC_MOUSEUP2_STATUS_LEFT: C2RustUnnamed_36 = 21474836994;
pub const KEYC_MOUSEUP1_STATUS_LEFT: C2RustUnnamed_36 = 21474836738;
pub const KEYC_MOUSEUP_STATUS_LEFT: C2RustUnnamed_36 = 21474836482;
pub const KEYC_MOUSEUP11_STATUS: C2RustUnnamed_36 = 21474839297;
pub const KEYC_MOUSEUP10_STATUS: C2RustUnnamed_36 = 21474839041;
pub const KEYC_MOUSEUP9_STATUS: C2RustUnnamed_36 = 21474838785;
pub const KEYC_MOUSEUP8_STATUS: C2RustUnnamed_36 = 21474838529;
pub const KEYC_MOUSEUP7_STATUS: C2RustUnnamed_36 = 21474838273;
pub const KEYC_MOUSEUP6_STATUS: C2RustUnnamed_36 = 21474838017;
pub const KEYC_MOUSEUP3_STATUS: C2RustUnnamed_36 = 21474837249;
pub const KEYC_MOUSEUP2_STATUS: C2RustUnnamed_36 = 21474836993;
pub const KEYC_MOUSEUP1_STATUS: C2RustUnnamed_36 = 21474836737;
pub const KEYC_MOUSEUP_STATUS: C2RustUnnamed_36 = 21474836481;
pub const KEYC_MOUSEUP11_PANE: C2RustUnnamed_36 = 21474839296;
pub const KEYC_MOUSEUP10_PANE: C2RustUnnamed_36 = 21474839040;
pub const KEYC_MOUSEUP9_PANE: C2RustUnnamed_36 = 21474838784;
pub const KEYC_MOUSEUP8_PANE: C2RustUnnamed_36 = 21474838528;
pub const KEYC_MOUSEUP7_PANE: C2RustUnnamed_36 = 21474838272;
pub const KEYC_MOUSEUP6_PANE: C2RustUnnamed_36 = 21474838016;
pub const KEYC_MOUSEUP3_PANE: C2RustUnnamed_36 = 21474837248;
pub const KEYC_MOUSEUP2_PANE: C2RustUnnamed_36 = 21474836992;
pub const KEYC_MOUSEUP1_PANE: C2RustUnnamed_36 = 21474836736;
pub const KEYC_MOUSEUP_PANE: C2RustUnnamed_36 = 21474836480;
pub const KEYC_MOUSEDOWN11_CONTROL9: C2RustUnnamed_36 = 17179872019;
pub const KEYC_MOUSEDOWN10_CONTROL9: C2RustUnnamed_36 = 17179871763;
pub const KEYC_MOUSEDOWN9_CONTROL9: C2RustUnnamed_36 = 17179871507;
pub const KEYC_MOUSEDOWN8_CONTROL9: C2RustUnnamed_36 = 17179871251;
pub const KEYC_MOUSEDOWN7_CONTROL9: C2RustUnnamed_36 = 17179870995;
pub const KEYC_MOUSEDOWN6_CONTROL9: C2RustUnnamed_36 = 17179870739;
pub const KEYC_MOUSEDOWN3_CONTROL9: C2RustUnnamed_36 = 17179869971;
pub const KEYC_MOUSEDOWN2_CONTROL9: C2RustUnnamed_36 = 17179869715;
pub const KEYC_MOUSEDOWN1_CONTROL9: C2RustUnnamed_36 = 17179869459;
pub const KEYC_MOUSEDOWN_CONTROL9: C2RustUnnamed_36 = 17179869203;
pub const KEYC_MOUSEDOWN11_CONTROL8: C2RustUnnamed_36 = 17179872018;
pub const KEYC_MOUSEDOWN10_CONTROL8: C2RustUnnamed_36 = 17179871762;
pub const KEYC_MOUSEDOWN9_CONTROL8: C2RustUnnamed_36 = 17179871506;
pub const KEYC_MOUSEDOWN8_CONTROL8: C2RustUnnamed_36 = 17179871250;
pub const KEYC_MOUSEDOWN7_CONTROL8: C2RustUnnamed_36 = 17179870994;
pub const KEYC_MOUSEDOWN6_CONTROL8: C2RustUnnamed_36 = 17179870738;
pub const KEYC_MOUSEDOWN3_CONTROL8: C2RustUnnamed_36 = 17179869970;
pub const KEYC_MOUSEDOWN2_CONTROL8: C2RustUnnamed_36 = 17179869714;
pub const KEYC_MOUSEDOWN1_CONTROL8: C2RustUnnamed_36 = 17179869458;
pub const KEYC_MOUSEDOWN_CONTROL8: C2RustUnnamed_36 = 17179869202;
pub const KEYC_MOUSEDOWN11_CONTROL7: C2RustUnnamed_36 = 17179872017;
pub const KEYC_MOUSEDOWN10_CONTROL7: C2RustUnnamed_36 = 17179871761;
pub const KEYC_MOUSEDOWN9_CONTROL7: C2RustUnnamed_36 = 17179871505;
pub const KEYC_MOUSEDOWN8_CONTROL7: C2RustUnnamed_36 = 17179871249;
pub const KEYC_MOUSEDOWN7_CONTROL7: C2RustUnnamed_36 = 17179870993;
pub const KEYC_MOUSEDOWN6_CONTROL7: C2RustUnnamed_36 = 17179870737;
pub const KEYC_MOUSEDOWN3_CONTROL7: C2RustUnnamed_36 = 17179869969;
pub const KEYC_MOUSEDOWN2_CONTROL7: C2RustUnnamed_36 = 17179869713;
pub const KEYC_MOUSEDOWN1_CONTROL7: C2RustUnnamed_36 = 17179869457;
pub const KEYC_MOUSEDOWN_CONTROL7: C2RustUnnamed_36 = 17179869201;
pub const KEYC_MOUSEDOWN11_CONTROL6: C2RustUnnamed_36 = 17179872016;
pub const KEYC_MOUSEDOWN10_CONTROL6: C2RustUnnamed_36 = 17179871760;
pub const KEYC_MOUSEDOWN9_CONTROL6: C2RustUnnamed_36 = 17179871504;
pub const KEYC_MOUSEDOWN8_CONTROL6: C2RustUnnamed_36 = 17179871248;
pub const KEYC_MOUSEDOWN7_CONTROL6: C2RustUnnamed_36 = 17179870992;
pub const KEYC_MOUSEDOWN6_CONTROL6: C2RustUnnamed_36 = 17179870736;
pub const KEYC_MOUSEDOWN3_CONTROL6: C2RustUnnamed_36 = 17179869968;
pub const KEYC_MOUSEDOWN2_CONTROL6: C2RustUnnamed_36 = 17179869712;
pub const KEYC_MOUSEDOWN1_CONTROL6: C2RustUnnamed_36 = 17179869456;
pub const KEYC_MOUSEDOWN_CONTROL6: C2RustUnnamed_36 = 17179869200;
pub const KEYC_MOUSEDOWN11_CONTROL5: C2RustUnnamed_36 = 17179872015;
pub const KEYC_MOUSEDOWN10_CONTROL5: C2RustUnnamed_36 = 17179871759;
pub const KEYC_MOUSEDOWN9_CONTROL5: C2RustUnnamed_36 = 17179871503;
pub const KEYC_MOUSEDOWN8_CONTROL5: C2RustUnnamed_36 = 17179871247;
pub const KEYC_MOUSEDOWN7_CONTROL5: C2RustUnnamed_36 = 17179870991;
pub const KEYC_MOUSEDOWN6_CONTROL5: C2RustUnnamed_36 = 17179870735;
pub const KEYC_MOUSEDOWN3_CONTROL5: C2RustUnnamed_36 = 17179869967;
pub const KEYC_MOUSEDOWN2_CONTROL5: C2RustUnnamed_36 = 17179869711;
pub const KEYC_MOUSEDOWN1_CONTROL5: C2RustUnnamed_36 = 17179869455;
pub const KEYC_MOUSEDOWN_CONTROL5: C2RustUnnamed_36 = 17179869199;
pub const KEYC_MOUSEDOWN11_CONTROL4: C2RustUnnamed_36 = 17179872014;
pub const KEYC_MOUSEDOWN10_CONTROL4: C2RustUnnamed_36 = 17179871758;
pub const KEYC_MOUSEDOWN9_CONTROL4: C2RustUnnamed_36 = 17179871502;
pub const KEYC_MOUSEDOWN8_CONTROL4: C2RustUnnamed_36 = 17179871246;
pub const KEYC_MOUSEDOWN7_CONTROL4: C2RustUnnamed_36 = 17179870990;
pub const KEYC_MOUSEDOWN6_CONTROL4: C2RustUnnamed_36 = 17179870734;
pub const KEYC_MOUSEDOWN3_CONTROL4: C2RustUnnamed_36 = 17179869966;
pub const KEYC_MOUSEDOWN2_CONTROL4: C2RustUnnamed_36 = 17179869710;
pub const KEYC_MOUSEDOWN1_CONTROL4: C2RustUnnamed_36 = 17179869454;
pub const KEYC_MOUSEDOWN_CONTROL4: C2RustUnnamed_36 = 17179869198;
pub const KEYC_MOUSEDOWN11_CONTROL3: C2RustUnnamed_36 = 17179872013;
pub const KEYC_MOUSEDOWN10_CONTROL3: C2RustUnnamed_36 = 17179871757;
pub const KEYC_MOUSEDOWN9_CONTROL3: C2RustUnnamed_36 = 17179871501;
pub const KEYC_MOUSEDOWN8_CONTROL3: C2RustUnnamed_36 = 17179871245;
pub const KEYC_MOUSEDOWN7_CONTROL3: C2RustUnnamed_36 = 17179870989;
pub const KEYC_MOUSEDOWN6_CONTROL3: C2RustUnnamed_36 = 17179870733;
pub const KEYC_MOUSEDOWN3_CONTROL3: C2RustUnnamed_36 = 17179869965;
pub const KEYC_MOUSEDOWN2_CONTROL3: C2RustUnnamed_36 = 17179869709;
pub const KEYC_MOUSEDOWN1_CONTROL3: C2RustUnnamed_36 = 17179869453;
pub const KEYC_MOUSEDOWN_CONTROL3: C2RustUnnamed_36 = 17179869197;
pub const KEYC_MOUSEDOWN11_CONTROL2: C2RustUnnamed_36 = 17179872012;
pub const KEYC_MOUSEDOWN10_CONTROL2: C2RustUnnamed_36 = 17179871756;
pub const KEYC_MOUSEDOWN9_CONTROL2: C2RustUnnamed_36 = 17179871500;
pub const KEYC_MOUSEDOWN8_CONTROL2: C2RustUnnamed_36 = 17179871244;
pub const KEYC_MOUSEDOWN7_CONTROL2: C2RustUnnamed_36 = 17179870988;
pub const KEYC_MOUSEDOWN6_CONTROL2: C2RustUnnamed_36 = 17179870732;
pub const KEYC_MOUSEDOWN3_CONTROL2: C2RustUnnamed_36 = 17179869964;
pub const KEYC_MOUSEDOWN2_CONTROL2: C2RustUnnamed_36 = 17179869708;
pub const KEYC_MOUSEDOWN1_CONTROL2: C2RustUnnamed_36 = 17179869452;
pub const KEYC_MOUSEDOWN_CONTROL2: C2RustUnnamed_36 = 17179869196;
pub const KEYC_MOUSEDOWN11_CONTROL1: C2RustUnnamed_36 = 17179872011;
pub const KEYC_MOUSEDOWN10_CONTROL1: C2RustUnnamed_36 = 17179871755;
pub const KEYC_MOUSEDOWN9_CONTROL1: C2RustUnnamed_36 = 17179871499;
pub const KEYC_MOUSEDOWN8_CONTROL1: C2RustUnnamed_36 = 17179871243;
pub const KEYC_MOUSEDOWN7_CONTROL1: C2RustUnnamed_36 = 17179870987;
pub const KEYC_MOUSEDOWN6_CONTROL1: C2RustUnnamed_36 = 17179870731;
pub const KEYC_MOUSEDOWN3_CONTROL1: C2RustUnnamed_36 = 17179869963;
pub const KEYC_MOUSEDOWN2_CONTROL1: C2RustUnnamed_36 = 17179869707;
pub const KEYC_MOUSEDOWN1_CONTROL1: C2RustUnnamed_36 = 17179869451;
pub const KEYC_MOUSEDOWN_CONTROL1: C2RustUnnamed_36 = 17179869195;
pub const KEYC_MOUSEDOWN11_CONTROL0: C2RustUnnamed_36 = 17179872010;
pub const KEYC_MOUSEDOWN10_CONTROL0: C2RustUnnamed_36 = 17179871754;
pub const KEYC_MOUSEDOWN9_CONTROL0: C2RustUnnamed_36 = 17179871498;
pub const KEYC_MOUSEDOWN8_CONTROL0: C2RustUnnamed_36 = 17179871242;
pub const KEYC_MOUSEDOWN7_CONTROL0: C2RustUnnamed_36 = 17179870986;
pub const KEYC_MOUSEDOWN6_CONTROL0: C2RustUnnamed_36 = 17179870730;
pub const KEYC_MOUSEDOWN3_CONTROL0: C2RustUnnamed_36 = 17179869962;
pub const KEYC_MOUSEDOWN2_CONTROL0: C2RustUnnamed_36 = 17179869706;
pub const KEYC_MOUSEDOWN1_CONTROL0: C2RustUnnamed_36 = 17179869450;
pub const KEYC_MOUSEDOWN_CONTROL0: C2RustUnnamed_36 = 17179869194;
pub const KEYC_MOUSEDOWN11_EMPTY: C2RustUnnamed_36 = 17179872009;
pub const KEYC_MOUSEDOWN10_EMPTY: C2RustUnnamed_36 = 17179871753;
pub const KEYC_MOUSEDOWN9_EMPTY: C2RustUnnamed_36 = 17179871497;
pub const KEYC_MOUSEDOWN8_EMPTY: C2RustUnnamed_36 = 17179871241;
pub const KEYC_MOUSEDOWN7_EMPTY: C2RustUnnamed_36 = 17179870985;
pub const KEYC_MOUSEDOWN6_EMPTY: C2RustUnnamed_36 = 17179870729;
pub const KEYC_MOUSEDOWN3_EMPTY: C2RustUnnamed_36 = 17179869961;
pub const KEYC_MOUSEDOWN2_EMPTY: C2RustUnnamed_36 = 17179869705;
pub const KEYC_MOUSEDOWN1_EMPTY: C2RustUnnamed_36 = 17179869449;
pub const KEYC_MOUSEDOWN_EMPTY: C2RustUnnamed_36 = 17179869193;
pub const KEYC_MOUSEDOWN11_SCROLLBAR_DOWN: C2RustUnnamed_36 = 17179872008;
pub const KEYC_MOUSEDOWN10_SCROLLBAR_DOWN: C2RustUnnamed_36 = 17179871752;
pub const KEYC_MOUSEDOWN9_SCROLLBAR_DOWN: C2RustUnnamed_36 = 17179871496;
pub const KEYC_MOUSEDOWN8_SCROLLBAR_DOWN: C2RustUnnamed_36 = 17179871240;
pub const KEYC_MOUSEDOWN7_SCROLLBAR_DOWN: C2RustUnnamed_36 = 17179870984;
pub const KEYC_MOUSEDOWN6_SCROLLBAR_DOWN: C2RustUnnamed_36 = 17179870728;
pub const KEYC_MOUSEDOWN3_SCROLLBAR_DOWN: C2RustUnnamed_36 = 17179869960;
pub const KEYC_MOUSEDOWN2_SCROLLBAR_DOWN: C2RustUnnamed_36 = 17179869704;
pub const KEYC_MOUSEDOWN1_SCROLLBAR_DOWN: C2RustUnnamed_36 = 17179869448;
pub const KEYC_MOUSEDOWN_SCROLLBAR_DOWN: C2RustUnnamed_36 = 17179869192;
pub const KEYC_MOUSEDOWN11_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 17179872007;
pub const KEYC_MOUSEDOWN10_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 17179871751;
pub const KEYC_MOUSEDOWN9_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 17179871495;
pub const KEYC_MOUSEDOWN8_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 17179871239;
pub const KEYC_MOUSEDOWN7_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 17179870983;
pub const KEYC_MOUSEDOWN6_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 17179870727;
pub const KEYC_MOUSEDOWN3_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 17179869959;
pub const KEYC_MOUSEDOWN2_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 17179869703;
pub const KEYC_MOUSEDOWN1_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 17179869447;
pub const KEYC_MOUSEDOWN_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 17179869191;
pub const KEYC_MOUSEDOWN11_SCROLLBAR_UP: C2RustUnnamed_36 = 17179872006;
pub const KEYC_MOUSEDOWN10_SCROLLBAR_UP: C2RustUnnamed_36 = 17179871750;
pub const KEYC_MOUSEDOWN9_SCROLLBAR_UP: C2RustUnnamed_36 = 17179871494;
pub const KEYC_MOUSEDOWN8_SCROLLBAR_UP: C2RustUnnamed_36 = 17179871238;
pub const KEYC_MOUSEDOWN7_SCROLLBAR_UP: C2RustUnnamed_36 = 17179870982;
pub const KEYC_MOUSEDOWN6_SCROLLBAR_UP: C2RustUnnamed_36 = 17179870726;
pub const KEYC_MOUSEDOWN3_SCROLLBAR_UP: C2RustUnnamed_36 = 17179869958;
pub const KEYC_MOUSEDOWN2_SCROLLBAR_UP: C2RustUnnamed_36 = 17179869702;
pub const KEYC_MOUSEDOWN1_SCROLLBAR_UP: C2RustUnnamed_36 = 17179869446;
pub const KEYC_MOUSEDOWN_SCROLLBAR_UP: C2RustUnnamed_36 = 17179869190;
pub const KEYC_MOUSEDOWN11_BORDER: C2RustUnnamed_36 = 17179872005;
pub const KEYC_MOUSEDOWN10_BORDER: C2RustUnnamed_36 = 17179871749;
pub const KEYC_MOUSEDOWN9_BORDER: C2RustUnnamed_36 = 17179871493;
pub const KEYC_MOUSEDOWN8_BORDER: C2RustUnnamed_36 = 17179871237;
pub const KEYC_MOUSEDOWN7_BORDER: C2RustUnnamed_36 = 17179870981;
pub const KEYC_MOUSEDOWN6_BORDER: C2RustUnnamed_36 = 17179870725;
pub const KEYC_MOUSEDOWN3_BORDER: C2RustUnnamed_36 = 17179869957;
pub const KEYC_MOUSEDOWN2_BORDER: C2RustUnnamed_36 = 17179869701;
pub const KEYC_MOUSEDOWN1_BORDER: C2RustUnnamed_36 = 17179869445;
pub const KEYC_MOUSEDOWN_BORDER: C2RustUnnamed_36 = 17179869189;
pub const KEYC_MOUSEDOWN11_STATUS_DEFAULT: C2RustUnnamed_36 = 17179872004;
pub const KEYC_MOUSEDOWN10_STATUS_DEFAULT: C2RustUnnamed_36 = 17179871748;
pub const KEYC_MOUSEDOWN9_STATUS_DEFAULT: C2RustUnnamed_36 = 17179871492;
pub const KEYC_MOUSEDOWN8_STATUS_DEFAULT: C2RustUnnamed_36 = 17179871236;
pub const KEYC_MOUSEDOWN7_STATUS_DEFAULT: C2RustUnnamed_36 = 17179870980;
pub const KEYC_MOUSEDOWN6_STATUS_DEFAULT: C2RustUnnamed_36 = 17179870724;
pub const KEYC_MOUSEDOWN3_STATUS_DEFAULT: C2RustUnnamed_36 = 17179869956;
pub const KEYC_MOUSEDOWN2_STATUS_DEFAULT: C2RustUnnamed_36 = 17179869700;
pub const KEYC_MOUSEDOWN1_STATUS_DEFAULT: C2RustUnnamed_36 = 17179869444;
pub const KEYC_MOUSEDOWN_STATUS_DEFAULT: C2RustUnnamed_36 = 17179869188;
pub const KEYC_MOUSEDOWN11_STATUS_RIGHT: C2RustUnnamed_36 = 17179872003;
pub const KEYC_MOUSEDOWN10_STATUS_RIGHT: C2RustUnnamed_36 = 17179871747;
pub const KEYC_MOUSEDOWN9_STATUS_RIGHT: C2RustUnnamed_36 = 17179871491;
pub const KEYC_MOUSEDOWN8_STATUS_RIGHT: C2RustUnnamed_36 = 17179871235;
pub const KEYC_MOUSEDOWN7_STATUS_RIGHT: C2RustUnnamed_36 = 17179870979;
pub const KEYC_MOUSEDOWN6_STATUS_RIGHT: C2RustUnnamed_36 = 17179870723;
pub const KEYC_MOUSEDOWN3_STATUS_RIGHT: C2RustUnnamed_36 = 17179869955;
pub const KEYC_MOUSEDOWN2_STATUS_RIGHT: C2RustUnnamed_36 = 17179869699;
pub const KEYC_MOUSEDOWN1_STATUS_RIGHT: C2RustUnnamed_36 = 17179869443;
pub const KEYC_MOUSEDOWN_STATUS_RIGHT: C2RustUnnamed_36 = 17179869187;
pub const KEYC_MOUSEDOWN11_STATUS_LEFT: C2RustUnnamed_36 = 17179872002;
pub const KEYC_MOUSEDOWN10_STATUS_LEFT: C2RustUnnamed_36 = 17179871746;
pub const KEYC_MOUSEDOWN9_STATUS_LEFT: C2RustUnnamed_36 = 17179871490;
pub const KEYC_MOUSEDOWN8_STATUS_LEFT: C2RustUnnamed_36 = 17179871234;
pub const KEYC_MOUSEDOWN7_STATUS_LEFT: C2RustUnnamed_36 = 17179870978;
pub const KEYC_MOUSEDOWN6_STATUS_LEFT: C2RustUnnamed_36 = 17179870722;
pub const KEYC_MOUSEDOWN3_STATUS_LEFT: C2RustUnnamed_36 = 17179869954;
pub const KEYC_MOUSEDOWN2_STATUS_LEFT: C2RustUnnamed_36 = 17179869698;
pub const KEYC_MOUSEDOWN1_STATUS_LEFT: C2RustUnnamed_36 = 17179869442;
pub const KEYC_MOUSEDOWN_STATUS_LEFT: C2RustUnnamed_36 = 17179869186;
pub const KEYC_MOUSEDOWN11_STATUS: C2RustUnnamed_36 = 17179872001;
pub const KEYC_MOUSEDOWN10_STATUS: C2RustUnnamed_36 = 17179871745;
pub const KEYC_MOUSEDOWN9_STATUS: C2RustUnnamed_36 = 17179871489;
pub const KEYC_MOUSEDOWN8_STATUS: C2RustUnnamed_36 = 17179871233;
pub const KEYC_MOUSEDOWN7_STATUS: C2RustUnnamed_36 = 17179870977;
pub const KEYC_MOUSEDOWN6_STATUS: C2RustUnnamed_36 = 17179870721;
pub const KEYC_MOUSEDOWN3_STATUS: C2RustUnnamed_36 = 17179869953;
pub const KEYC_MOUSEDOWN2_STATUS: C2RustUnnamed_36 = 17179869697;
pub const KEYC_MOUSEDOWN1_STATUS: C2RustUnnamed_36 = 17179869441;
pub const KEYC_MOUSEDOWN_STATUS: C2RustUnnamed_36 = 17179869185;
pub const KEYC_MOUSEDOWN11_PANE: C2RustUnnamed_36 = 17179872000;
pub const KEYC_MOUSEDOWN10_PANE: C2RustUnnamed_36 = 17179871744;
pub const KEYC_MOUSEDOWN9_PANE: C2RustUnnamed_36 = 17179871488;
pub const KEYC_MOUSEDOWN8_PANE: C2RustUnnamed_36 = 17179871232;
pub const KEYC_MOUSEDOWN7_PANE: C2RustUnnamed_36 = 17179870976;
pub const KEYC_MOUSEDOWN6_PANE: C2RustUnnamed_36 = 17179870720;
pub const KEYC_MOUSEDOWN3_PANE: C2RustUnnamed_36 = 17179869952;
pub const KEYC_MOUSEDOWN2_PANE: C2RustUnnamed_36 = 17179869696;
pub const KEYC_MOUSEDOWN1_PANE: C2RustUnnamed_36 = 17179869440;
pub const KEYC_MOUSEDOWN_PANE: C2RustUnnamed_36 = 17179869184;
pub const KEYC_WHEELUP11_CONTROL9: C2RustUnnamed_36 = 38654708499;
pub const KEYC_WHEELUP10_CONTROL9: C2RustUnnamed_36 = 38654708243;
pub const KEYC_WHEELUP9_CONTROL9: C2RustUnnamed_36 = 38654707987;
pub const KEYC_WHEELUP8_CONTROL9: C2RustUnnamed_36 = 38654707731;
pub const KEYC_WHEELUP7_CONTROL9: C2RustUnnamed_36 = 38654707475;
pub const KEYC_WHEELUP6_CONTROL9: C2RustUnnamed_36 = 38654707219;
pub const KEYC_WHEELUP3_CONTROL9: C2RustUnnamed_36 = 38654706451;
pub const KEYC_WHEELUP2_CONTROL9: C2RustUnnamed_36 = 38654706195;
pub const KEYC_WHEELUP1_CONTROL9: C2RustUnnamed_36 = 38654705939;
pub const KEYC_WHEELUP_CONTROL9: C2RustUnnamed_36 = 38654705683;
pub const KEYC_WHEELUP11_CONTROL8: C2RustUnnamed_36 = 38654708498;
pub const KEYC_WHEELUP10_CONTROL8: C2RustUnnamed_36 = 38654708242;
pub const KEYC_WHEELUP9_CONTROL8: C2RustUnnamed_36 = 38654707986;
pub const KEYC_WHEELUP8_CONTROL8: C2RustUnnamed_36 = 38654707730;
pub const KEYC_WHEELUP7_CONTROL8: C2RustUnnamed_36 = 38654707474;
pub const KEYC_WHEELUP6_CONTROL8: C2RustUnnamed_36 = 38654707218;
pub const KEYC_WHEELUP3_CONTROL8: C2RustUnnamed_36 = 38654706450;
pub const KEYC_WHEELUP2_CONTROL8: C2RustUnnamed_36 = 38654706194;
pub const KEYC_WHEELUP1_CONTROL8: C2RustUnnamed_36 = 38654705938;
pub const KEYC_WHEELUP_CONTROL8: C2RustUnnamed_36 = 38654705682;
pub const KEYC_WHEELUP11_CONTROL7: C2RustUnnamed_36 = 38654708497;
pub const KEYC_WHEELUP10_CONTROL7: C2RustUnnamed_36 = 38654708241;
pub const KEYC_WHEELUP9_CONTROL7: C2RustUnnamed_36 = 38654707985;
pub const KEYC_WHEELUP8_CONTROL7: C2RustUnnamed_36 = 38654707729;
pub const KEYC_WHEELUP7_CONTROL7: C2RustUnnamed_36 = 38654707473;
pub const KEYC_WHEELUP6_CONTROL7: C2RustUnnamed_36 = 38654707217;
pub const KEYC_WHEELUP3_CONTROL7: C2RustUnnamed_36 = 38654706449;
pub const KEYC_WHEELUP2_CONTROL7: C2RustUnnamed_36 = 38654706193;
pub const KEYC_WHEELUP1_CONTROL7: C2RustUnnamed_36 = 38654705937;
pub const KEYC_WHEELUP_CONTROL7: C2RustUnnamed_36 = 38654705681;
pub const KEYC_WHEELUP11_CONTROL6: C2RustUnnamed_36 = 38654708496;
pub const KEYC_WHEELUP10_CONTROL6: C2RustUnnamed_36 = 38654708240;
pub const KEYC_WHEELUP9_CONTROL6: C2RustUnnamed_36 = 38654707984;
pub const KEYC_WHEELUP8_CONTROL6: C2RustUnnamed_36 = 38654707728;
pub const KEYC_WHEELUP7_CONTROL6: C2RustUnnamed_36 = 38654707472;
pub const KEYC_WHEELUP6_CONTROL6: C2RustUnnamed_36 = 38654707216;
pub const KEYC_WHEELUP3_CONTROL6: C2RustUnnamed_36 = 38654706448;
pub const KEYC_WHEELUP2_CONTROL6: C2RustUnnamed_36 = 38654706192;
pub const KEYC_WHEELUP1_CONTROL6: C2RustUnnamed_36 = 38654705936;
pub const KEYC_WHEELUP_CONTROL6: C2RustUnnamed_36 = 38654705680;
pub const KEYC_WHEELUP11_CONTROL5: C2RustUnnamed_36 = 38654708495;
pub const KEYC_WHEELUP10_CONTROL5: C2RustUnnamed_36 = 38654708239;
pub const KEYC_WHEELUP9_CONTROL5: C2RustUnnamed_36 = 38654707983;
pub const KEYC_WHEELUP8_CONTROL5: C2RustUnnamed_36 = 38654707727;
pub const KEYC_WHEELUP7_CONTROL5: C2RustUnnamed_36 = 38654707471;
pub const KEYC_WHEELUP6_CONTROL5: C2RustUnnamed_36 = 38654707215;
pub const KEYC_WHEELUP3_CONTROL5: C2RustUnnamed_36 = 38654706447;
pub const KEYC_WHEELUP2_CONTROL5: C2RustUnnamed_36 = 38654706191;
pub const KEYC_WHEELUP1_CONTROL5: C2RustUnnamed_36 = 38654705935;
pub const KEYC_WHEELUP_CONTROL5: C2RustUnnamed_36 = 38654705679;
pub const KEYC_WHEELUP11_CONTROL4: C2RustUnnamed_36 = 38654708494;
pub const KEYC_WHEELUP10_CONTROL4: C2RustUnnamed_36 = 38654708238;
pub const KEYC_WHEELUP9_CONTROL4: C2RustUnnamed_36 = 38654707982;
pub const KEYC_WHEELUP8_CONTROL4: C2RustUnnamed_36 = 38654707726;
pub const KEYC_WHEELUP7_CONTROL4: C2RustUnnamed_36 = 38654707470;
pub const KEYC_WHEELUP6_CONTROL4: C2RustUnnamed_36 = 38654707214;
pub const KEYC_WHEELUP3_CONTROL4: C2RustUnnamed_36 = 38654706446;
pub const KEYC_WHEELUP2_CONTROL4: C2RustUnnamed_36 = 38654706190;
pub const KEYC_WHEELUP1_CONTROL4: C2RustUnnamed_36 = 38654705934;
pub const KEYC_WHEELUP_CONTROL4: C2RustUnnamed_36 = 38654705678;
pub const KEYC_WHEELUP11_CONTROL3: C2RustUnnamed_36 = 38654708493;
pub const KEYC_WHEELUP10_CONTROL3: C2RustUnnamed_36 = 38654708237;
pub const KEYC_WHEELUP9_CONTROL3: C2RustUnnamed_36 = 38654707981;
pub const KEYC_WHEELUP8_CONTROL3: C2RustUnnamed_36 = 38654707725;
pub const KEYC_WHEELUP7_CONTROL3: C2RustUnnamed_36 = 38654707469;
pub const KEYC_WHEELUP6_CONTROL3: C2RustUnnamed_36 = 38654707213;
pub const KEYC_WHEELUP3_CONTROL3: C2RustUnnamed_36 = 38654706445;
pub const KEYC_WHEELUP2_CONTROL3: C2RustUnnamed_36 = 38654706189;
pub const KEYC_WHEELUP1_CONTROL3: C2RustUnnamed_36 = 38654705933;
pub const KEYC_WHEELUP_CONTROL3: C2RustUnnamed_36 = 38654705677;
pub const KEYC_WHEELUP11_CONTROL2: C2RustUnnamed_36 = 38654708492;
pub const KEYC_WHEELUP10_CONTROL2: C2RustUnnamed_36 = 38654708236;
pub const KEYC_WHEELUP9_CONTROL2: C2RustUnnamed_36 = 38654707980;
pub const KEYC_WHEELUP8_CONTROL2: C2RustUnnamed_36 = 38654707724;
pub const KEYC_WHEELUP7_CONTROL2: C2RustUnnamed_36 = 38654707468;
pub const KEYC_WHEELUP6_CONTROL2: C2RustUnnamed_36 = 38654707212;
pub const KEYC_WHEELUP3_CONTROL2: C2RustUnnamed_36 = 38654706444;
pub const KEYC_WHEELUP2_CONTROL2: C2RustUnnamed_36 = 38654706188;
pub const KEYC_WHEELUP1_CONTROL2: C2RustUnnamed_36 = 38654705932;
pub const KEYC_WHEELUP_CONTROL2: C2RustUnnamed_36 = 38654705676;
pub const KEYC_WHEELUP11_CONTROL1: C2RustUnnamed_36 = 38654708491;
pub const KEYC_WHEELUP10_CONTROL1: C2RustUnnamed_36 = 38654708235;
pub const KEYC_WHEELUP9_CONTROL1: C2RustUnnamed_36 = 38654707979;
pub const KEYC_WHEELUP8_CONTROL1: C2RustUnnamed_36 = 38654707723;
pub const KEYC_WHEELUP7_CONTROL1: C2RustUnnamed_36 = 38654707467;
pub const KEYC_WHEELUP6_CONTROL1: C2RustUnnamed_36 = 38654707211;
pub const KEYC_WHEELUP3_CONTROL1: C2RustUnnamed_36 = 38654706443;
pub const KEYC_WHEELUP2_CONTROL1: C2RustUnnamed_36 = 38654706187;
pub const KEYC_WHEELUP1_CONTROL1: C2RustUnnamed_36 = 38654705931;
pub const KEYC_WHEELUP_CONTROL1: C2RustUnnamed_36 = 38654705675;
pub const KEYC_WHEELUP11_CONTROL0: C2RustUnnamed_36 = 38654708490;
pub const KEYC_WHEELUP10_CONTROL0: C2RustUnnamed_36 = 38654708234;
pub const KEYC_WHEELUP9_CONTROL0: C2RustUnnamed_36 = 38654707978;
pub const KEYC_WHEELUP8_CONTROL0: C2RustUnnamed_36 = 38654707722;
pub const KEYC_WHEELUP7_CONTROL0: C2RustUnnamed_36 = 38654707466;
pub const KEYC_WHEELUP6_CONTROL0: C2RustUnnamed_36 = 38654707210;
pub const KEYC_WHEELUP3_CONTROL0: C2RustUnnamed_36 = 38654706442;
pub const KEYC_WHEELUP2_CONTROL0: C2RustUnnamed_36 = 38654706186;
pub const KEYC_WHEELUP1_CONTROL0: C2RustUnnamed_36 = 38654705930;
pub const KEYC_WHEELUP_CONTROL0: C2RustUnnamed_36 = 38654705674;
pub const KEYC_WHEELUP11_EMPTY: C2RustUnnamed_36 = 38654708489;
pub const KEYC_WHEELUP10_EMPTY: C2RustUnnamed_36 = 38654708233;
pub const KEYC_WHEELUP9_EMPTY: C2RustUnnamed_36 = 38654707977;
pub const KEYC_WHEELUP8_EMPTY: C2RustUnnamed_36 = 38654707721;
pub const KEYC_WHEELUP7_EMPTY: C2RustUnnamed_36 = 38654707465;
pub const KEYC_WHEELUP6_EMPTY: C2RustUnnamed_36 = 38654707209;
pub const KEYC_WHEELUP3_EMPTY: C2RustUnnamed_36 = 38654706441;
pub const KEYC_WHEELUP2_EMPTY: C2RustUnnamed_36 = 38654706185;
pub const KEYC_WHEELUP1_EMPTY: C2RustUnnamed_36 = 38654705929;
pub const KEYC_WHEELUP_EMPTY: C2RustUnnamed_36 = 38654705673;
pub const KEYC_WHEELUP11_SCROLLBAR_DOWN: C2RustUnnamed_36 = 38654708488;
pub const KEYC_WHEELUP10_SCROLLBAR_DOWN: C2RustUnnamed_36 = 38654708232;
pub const KEYC_WHEELUP9_SCROLLBAR_DOWN: C2RustUnnamed_36 = 38654707976;
pub const KEYC_WHEELUP8_SCROLLBAR_DOWN: C2RustUnnamed_36 = 38654707720;
pub const KEYC_WHEELUP7_SCROLLBAR_DOWN: C2RustUnnamed_36 = 38654707464;
pub const KEYC_WHEELUP6_SCROLLBAR_DOWN: C2RustUnnamed_36 = 38654707208;
pub const KEYC_WHEELUP3_SCROLLBAR_DOWN: C2RustUnnamed_36 = 38654706440;
pub const KEYC_WHEELUP2_SCROLLBAR_DOWN: C2RustUnnamed_36 = 38654706184;
pub const KEYC_WHEELUP1_SCROLLBAR_DOWN: C2RustUnnamed_36 = 38654705928;
pub const KEYC_WHEELUP_SCROLLBAR_DOWN: C2RustUnnamed_36 = 38654705672;
pub const KEYC_WHEELUP11_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 38654708487;
pub const KEYC_WHEELUP10_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 38654708231;
pub const KEYC_WHEELUP9_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 38654707975;
pub const KEYC_WHEELUP8_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 38654707719;
pub const KEYC_WHEELUP7_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 38654707463;
pub const KEYC_WHEELUP6_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 38654707207;
pub const KEYC_WHEELUP3_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 38654706439;
pub const KEYC_WHEELUP2_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 38654706183;
pub const KEYC_WHEELUP1_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 38654705927;
pub const KEYC_WHEELUP_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 38654705671;
pub const KEYC_WHEELUP11_SCROLLBAR_UP: C2RustUnnamed_36 = 38654708486;
pub const KEYC_WHEELUP10_SCROLLBAR_UP: C2RustUnnamed_36 = 38654708230;
pub const KEYC_WHEELUP9_SCROLLBAR_UP: C2RustUnnamed_36 = 38654707974;
pub const KEYC_WHEELUP8_SCROLLBAR_UP: C2RustUnnamed_36 = 38654707718;
pub const KEYC_WHEELUP7_SCROLLBAR_UP: C2RustUnnamed_36 = 38654707462;
pub const KEYC_WHEELUP6_SCROLLBAR_UP: C2RustUnnamed_36 = 38654707206;
pub const KEYC_WHEELUP3_SCROLLBAR_UP: C2RustUnnamed_36 = 38654706438;
pub const KEYC_WHEELUP2_SCROLLBAR_UP: C2RustUnnamed_36 = 38654706182;
pub const KEYC_WHEELUP1_SCROLLBAR_UP: C2RustUnnamed_36 = 38654705926;
pub const KEYC_WHEELUP_SCROLLBAR_UP: C2RustUnnamed_36 = 38654705670;
pub const KEYC_WHEELUP11_BORDER: C2RustUnnamed_36 = 38654708485;
pub const KEYC_WHEELUP10_BORDER: C2RustUnnamed_36 = 38654708229;
pub const KEYC_WHEELUP9_BORDER: C2RustUnnamed_36 = 38654707973;
pub const KEYC_WHEELUP8_BORDER: C2RustUnnamed_36 = 38654707717;
pub const KEYC_WHEELUP7_BORDER: C2RustUnnamed_36 = 38654707461;
pub const KEYC_WHEELUP6_BORDER: C2RustUnnamed_36 = 38654707205;
pub const KEYC_WHEELUP3_BORDER: C2RustUnnamed_36 = 38654706437;
pub const KEYC_WHEELUP2_BORDER: C2RustUnnamed_36 = 38654706181;
pub const KEYC_WHEELUP1_BORDER: C2RustUnnamed_36 = 38654705925;
pub const KEYC_WHEELUP_BORDER: C2RustUnnamed_36 = 38654705669;
pub const KEYC_WHEELUP11_STATUS_DEFAULT: C2RustUnnamed_36 = 38654708484;
pub const KEYC_WHEELUP10_STATUS_DEFAULT: C2RustUnnamed_36 = 38654708228;
pub const KEYC_WHEELUP9_STATUS_DEFAULT: C2RustUnnamed_36 = 38654707972;
pub const KEYC_WHEELUP8_STATUS_DEFAULT: C2RustUnnamed_36 = 38654707716;
pub const KEYC_WHEELUP7_STATUS_DEFAULT: C2RustUnnamed_36 = 38654707460;
pub const KEYC_WHEELUP6_STATUS_DEFAULT: C2RustUnnamed_36 = 38654707204;
pub const KEYC_WHEELUP3_STATUS_DEFAULT: C2RustUnnamed_36 = 38654706436;
pub const KEYC_WHEELUP2_STATUS_DEFAULT: C2RustUnnamed_36 = 38654706180;
pub const KEYC_WHEELUP1_STATUS_DEFAULT: C2RustUnnamed_36 = 38654705924;
pub const KEYC_WHEELUP_STATUS_DEFAULT: C2RustUnnamed_36 = 38654705668;
pub const KEYC_WHEELUP11_STATUS_RIGHT: C2RustUnnamed_36 = 38654708483;
pub const KEYC_WHEELUP10_STATUS_RIGHT: C2RustUnnamed_36 = 38654708227;
pub const KEYC_WHEELUP9_STATUS_RIGHT: C2RustUnnamed_36 = 38654707971;
pub const KEYC_WHEELUP8_STATUS_RIGHT: C2RustUnnamed_36 = 38654707715;
pub const KEYC_WHEELUP7_STATUS_RIGHT: C2RustUnnamed_36 = 38654707459;
pub const KEYC_WHEELUP6_STATUS_RIGHT: C2RustUnnamed_36 = 38654707203;
pub const KEYC_WHEELUP3_STATUS_RIGHT: C2RustUnnamed_36 = 38654706435;
pub const KEYC_WHEELUP2_STATUS_RIGHT: C2RustUnnamed_36 = 38654706179;
pub const KEYC_WHEELUP1_STATUS_RIGHT: C2RustUnnamed_36 = 38654705923;
pub const KEYC_WHEELUP_STATUS_RIGHT: C2RustUnnamed_36 = 38654705667;
pub const KEYC_WHEELUP11_STATUS_LEFT: C2RustUnnamed_36 = 38654708482;
pub const KEYC_WHEELUP10_STATUS_LEFT: C2RustUnnamed_36 = 38654708226;
pub const KEYC_WHEELUP9_STATUS_LEFT: C2RustUnnamed_36 = 38654707970;
pub const KEYC_WHEELUP8_STATUS_LEFT: C2RustUnnamed_36 = 38654707714;
pub const KEYC_WHEELUP7_STATUS_LEFT: C2RustUnnamed_36 = 38654707458;
pub const KEYC_WHEELUP6_STATUS_LEFT: C2RustUnnamed_36 = 38654707202;
pub const KEYC_WHEELUP3_STATUS_LEFT: C2RustUnnamed_36 = 38654706434;
pub const KEYC_WHEELUP2_STATUS_LEFT: C2RustUnnamed_36 = 38654706178;
pub const KEYC_WHEELUP1_STATUS_LEFT: C2RustUnnamed_36 = 38654705922;
pub const KEYC_WHEELUP_STATUS_LEFT: C2RustUnnamed_36 = 38654705666;
pub const KEYC_WHEELUP11_STATUS: C2RustUnnamed_36 = 38654708481;
pub const KEYC_WHEELUP10_STATUS: C2RustUnnamed_36 = 38654708225;
pub const KEYC_WHEELUP9_STATUS: C2RustUnnamed_36 = 38654707969;
pub const KEYC_WHEELUP8_STATUS: C2RustUnnamed_36 = 38654707713;
pub const KEYC_WHEELUP7_STATUS: C2RustUnnamed_36 = 38654707457;
pub const KEYC_WHEELUP6_STATUS: C2RustUnnamed_36 = 38654707201;
pub const KEYC_WHEELUP3_STATUS: C2RustUnnamed_36 = 38654706433;
pub const KEYC_WHEELUP2_STATUS: C2RustUnnamed_36 = 38654706177;
pub const KEYC_WHEELUP1_STATUS: C2RustUnnamed_36 = 38654705921;
pub const KEYC_WHEELUP_STATUS: C2RustUnnamed_36 = 38654705665;
pub const KEYC_WHEELUP11_PANE: C2RustUnnamed_36 = 38654708480;
pub const KEYC_WHEELUP10_PANE: C2RustUnnamed_36 = 38654708224;
pub const KEYC_WHEELUP9_PANE: C2RustUnnamed_36 = 38654707968;
pub const KEYC_WHEELUP8_PANE: C2RustUnnamed_36 = 38654707712;
pub const KEYC_WHEELUP7_PANE: C2RustUnnamed_36 = 38654707456;
pub const KEYC_WHEELUP6_PANE: C2RustUnnamed_36 = 38654707200;
pub const KEYC_WHEELUP3_PANE: C2RustUnnamed_36 = 38654706432;
pub const KEYC_WHEELUP2_PANE: C2RustUnnamed_36 = 38654706176;
pub const KEYC_WHEELUP1_PANE: C2RustUnnamed_36 = 38654705920;
pub const KEYC_WHEELUP_PANE: C2RustUnnamed_36 = 38654705664;
pub const KEYC_WHEELDOWN11_CONTROL9: C2RustUnnamed_36 = 34359741203;
pub const KEYC_WHEELDOWN10_CONTROL9: C2RustUnnamed_36 = 34359740947;
pub const KEYC_WHEELDOWN9_CONTROL9: C2RustUnnamed_36 = 34359740691;
pub const KEYC_WHEELDOWN8_CONTROL9: C2RustUnnamed_36 = 34359740435;
pub const KEYC_WHEELDOWN7_CONTROL9: C2RustUnnamed_36 = 34359740179;
pub const KEYC_WHEELDOWN6_CONTROL9: C2RustUnnamed_36 = 34359739923;
pub const KEYC_WHEELDOWN3_CONTROL9: C2RustUnnamed_36 = 34359739155;
pub const KEYC_WHEELDOWN2_CONTROL9: C2RustUnnamed_36 = 34359738899;
pub const KEYC_WHEELDOWN1_CONTROL9: C2RustUnnamed_36 = 34359738643;
pub const KEYC_WHEELDOWN_CONTROL9: C2RustUnnamed_36 = 34359738387;
pub const KEYC_WHEELDOWN11_CONTROL8: C2RustUnnamed_36 = 34359741202;
pub const KEYC_WHEELDOWN10_CONTROL8: C2RustUnnamed_36 = 34359740946;
pub const KEYC_WHEELDOWN9_CONTROL8: C2RustUnnamed_36 = 34359740690;
pub const KEYC_WHEELDOWN8_CONTROL8: C2RustUnnamed_36 = 34359740434;
pub const KEYC_WHEELDOWN7_CONTROL8: C2RustUnnamed_36 = 34359740178;
pub const KEYC_WHEELDOWN6_CONTROL8: C2RustUnnamed_36 = 34359739922;
pub const KEYC_WHEELDOWN3_CONTROL8: C2RustUnnamed_36 = 34359739154;
pub const KEYC_WHEELDOWN2_CONTROL8: C2RustUnnamed_36 = 34359738898;
pub const KEYC_WHEELDOWN1_CONTROL8: C2RustUnnamed_36 = 34359738642;
pub const KEYC_WHEELDOWN_CONTROL8: C2RustUnnamed_36 = 34359738386;
pub const KEYC_WHEELDOWN11_CONTROL7: C2RustUnnamed_36 = 34359741201;
pub const KEYC_WHEELDOWN10_CONTROL7: C2RustUnnamed_36 = 34359740945;
pub const KEYC_WHEELDOWN9_CONTROL7: C2RustUnnamed_36 = 34359740689;
pub const KEYC_WHEELDOWN8_CONTROL7: C2RustUnnamed_36 = 34359740433;
pub const KEYC_WHEELDOWN7_CONTROL7: C2RustUnnamed_36 = 34359740177;
pub const KEYC_WHEELDOWN6_CONTROL7: C2RustUnnamed_36 = 34359739921;
pub const KEYC_WHEELDOWN3_CONTROL7: C2RustUnnamed_36 = 34359739153;
pub const KEYC_WHEELDOWN2_CONTROL7: C2RustUnnamed_36 = 34359738897;
pub const KEYC_WHEELDOWN1_CONTROL7: C2RustUnnamed_36 = 34359738641;
pub const KEYC_WHEELDOWN_CONTROL7: C2RustUnnamed_36 = 34359738385;
pub const KEYC_WHEELDOWN11_CONTROL6: C2RustUnnamed_36 = 34359741200;
pub const KEYC_WHEELDOWN10_CONTROL6: C2RustUnnamed_36 = 34359740944;
pub const KEYC_WHEELDOWN9_CONTROL6: C2RustUnnamed_36 = 34359740688;
pub const KEYC_WHEELDOWN8_CONTROL6: C2RustUnnamed_36 = 34359740432;
pub const KEYC_WHEELDOWN7_CONTROL6: C2RustUnnamed_36 = 34359740176;
pub const KEYC_WHEELDOWN6_CONTROL6: C2RustUnnamed_36 = 34359739920;
pub const KEYC_WHEELDOWN3_CONTROL6: C2RustUnnamed_36 = 34359739152;
pub const KEYC_WHEELDOWN2_CONTROL6: C2RustUnnamed_36 = 34359738896;
pub const KEYC_WHEELDOWN1_CONTROL6: C2RustUnnamed_36 = 34359738640;
pub const KEYC_WHEELDOWN_CONTROL6: C2RustUnnamed_36 = 34359738384;
pub const KEYC_WHEELDOWN11_CONTROL5: C2RustUnnamed_36 = 34359741199;
pub const KEYC_WHEELDOWN10_CONTROL5: C2RustUnnamed_36 = 34359740943;
pub const KEYC_WHEELDOWN9_CONTROL5: C2RustUnnamed_36 = 34359740687;
pub const KEYC_WHEELDOWN8_CONTROL5: C2RustUnnamed_36 = 34359740431;
pub const KEYC_WHEELDOWN7_CONTROL5: C2RustUnnamed_36 = 34359740175;
pub const KEYC_WHEELDOWN6_CONTROL5: C2RustUnnamed_36 = 34359739919;
pub const KEYC_WHEELDOWN3_CONTROL5: C2RustUnnamed_36 = 34359739151;
pub const KEYC_WHEELDOWN2_CONTROL5: C2RustUnnamed_36 = 34359738895;
pub const KEYC_WHEELDOWN1_CONTROL5: C2RustUnnamed_36 = 34359738639;
pub const KEYC_WHEELDOWN_CONTROL5: C2RustUnnamed_36 = 34359738383;
pub const KEYC_WHEELDOWN11_CONTROL4: C2RustUnnamed_36 = 34359741198;
pub const KEYC_WHEELDOWN10_CONTROL4: C2RustUnnamed_36 = 34359740942;
pub const KEYC_WHEELDOWN9_CONTROL4: C2RustUnnamed_36 = 34359740686;
pub const KEYC_WHEELDOWN8_CONTROL4: C2RustUnnamed_36 = 34359740430;
pub const KEYC_WHEELDOWN7_CONTROL4: C2RustUnnamed_36 = 34359740174;
pub const KEYC_WHEELDOWN6_CONTROL4: C2RustUnnamed_36 = 34359739918;
pub const KEYC_WHEELDOWN3_CONTROL4: C2RustUnnamed_36 = 34359739150;
pub const KEYC_WHEELDOWN2_CONTROL4: C2RustUnnamed_36 = 34359738894;
pub const KEYC_WHEELDOWN1_CONTROL4: C2RustUnnamed_36 = 34359738638;
pub const KEYC_WHEELDOWN_CONTROL4: C2RustUnnamed_36 = 34359738382;
pub const KEYC_WHEELDOWN11_CONTROL3: C2RustUnnamed_36 = 34359741197;
pub const KEYC_WHEELDOWN10_CONTROL3: C2RustUnnamed_36 = 34359740941;
pub const KEYC_WHEELDOWN9_CONTROL3: C2RustUnnamed_36 = 34359740685;
pub const KEYC_WHEELDOWN8_CONTROL3: C2RustUnnamed_36 = 34359740429;
pub const KEYC_WHEELDOWN7_CONTROL3: C2RustUnnamed_36 = 34359740173;
pub const KEYC_WHEELDOWN6_CONTROL3: C2RustUnnamed_36 = 34359739917;
pub const KEYC_WHEELDOWN3_CONTROL3: C2RustUnnamed_36 = 34359739149;
pub const KEYC_WHEELDOWN2_CONTROL3: C2RustUnnamed_36 = 34359738893;
pub const KEYC_WHEELDOWN1_CONTROL3: C2RustUnnamed_36 = 34359738637;
pub const KEYC_WHEELDOWN_CONTROL3: C2RustUnnamed_36 = 34359738381;
pub const KEYC_WHEELDOWN11_CONTROL2: C2RustUnnamed_36 = 34359741196;
pub const KEYC_WHEELDOWN10_CONTROL2: C2RustUnnamed_36 = 34359740940;
pub const KEYC_WHEELDOWN9_CONTROL2: C2RustUnnamed_36 = 34359740684;
pub const KEYC_WHEELDOWN8_CONTROL2: C2RustUnnamed_36 = 34359740428;
pub const KEYC_WHEELDOWN7_CONTROL2: C2RustUnnamed_36 = 34359740172;
pub const KEYC_WHEELDOWN6_CONTROL2: C2RustUnnamed_36 = 34359739916;
pub const KEYC_WHEELDOWN3_CONTROL2: C2RustUnnamed_36 = 34359739148;
pub const KEYC_WHEELDOWN2_CONTROL2: C2RustUnnamed_36 = 34359738892;
pub const KEYC_WHEELDOWN1_CONTROL2: C2RustUnnamed_36 = 34359738636;
pub const KEYC_WHEELDOWN_CONTROL2: C2RustUnnamed_36 = 34359738380;
pub const KEYC_WHEELDOWN11_CONTROL1: C2RustUnnamed_36 = 34359741195;
pub const KEYC_WHEELDOWN10_CONTROL1: C2RustUnnamed_36 = 34359740939;
pub const KEYC_WHEELDOWN9_CONTROL1: C2RustUnnamed_36 = 34359740683;
pub const KEYC_WHEELDOWN8_CONTROL1: C2RustUnnamed_36 = 34359740427;
pub const KEYC_WHEELDOWN7_CONTROL1: C2RustUnnamed_36 = 34359740171;
pub const KEYC_WHEELDOWN6_CONTROL1: C2RustUnnamed_36 = 34359739915;
pub const KEYC_WHEELDOWN3_CONTROL1: C2RustUnnamed_36 = 34359739147;
pub const KEYC_WHEELDOWN2_CONTROL1: C2RustUnnamed_36 = 34359738891;
pub const KEYC_WHEELDOWN1_CONTROL1: C2RustUnnamed_36 = 34359738635;
pub const KEYC_WHEELDOWN_CONTROL1: C2RustUnnamed_36 = 34359738379;
pub const KEYC_WHEELDOWN11_CONTROL0: C2RustUnnamed_36 = 34359741194;
pub const KEYC_WHEELDOWN10_CONTROL0: C2RustUnnamed_36 = 34359740938;
pub const KEYC_WHEELDOWN9_CONTROL0: C2RustUnnamed_36 = 34359740682;
pub const KEYC_WHEELDOWN8_CONTROL0: C2RustUnnamed_36 = 34359740426;
pub const KEYC_WHEELDOWN7_CONTROL0: C2RustUnnamed_36 = 34359740170;
pub const KEYC_WHEELDOWN6_CONTROL0: C2RustUnnamed_36 = 34359739914;
pub const KEYC_WHEELDOWN3_CONTROL0: C2RustUnnamed_36 = 34359739146;
pub const KEYC_WHEELDOWN2_CONTROL0: C2RustUnnamed_36 = 34359738890;
pub const KEYC_WHEELDOWN1_CONTROL0: C2RustUnnamed_36 = 34359738634;
pub const KEYC_WHEELDOWN_CONTROL0: C2RustUnnamed_36 = 34359738378;
pub const KEYC_WHEELDOWN11_EMPTY: C2RustUnnamed_36 = 34359741193;
pub const KEYC_WHEELDOWN10_EMPTY: C2RustUnnamed_36 = 34359740937;
pub const KEYC_WHEELDOWN9_EMPTY: C2RustUnnamed_36 = 34359740681;
pub const KEYC_WHEELDOWN8_EMPTY: C2RustUnnamed_36 = 34359740425;
pub const KEYC_WHEELDOWN7_EMPTY: C2RustUnnamed_36 = 34359740169;
pub const KEYC_WHEELDOWN6_EMPTY: C2RustUnnamed_36 = 34359739913;
pub const KEYC_WHEELDOWN3_EMPTY: C2RustUnnamed_36 = 34359739145;
pub const KEYC_WHEELDOWN2_EMPTY: C2RustUnnamed_36 = 34359738889;
pub const KEYC_WHEELDOWN1_EMPTY: C2RustUnnamed_36 = 34359738633;
pub const KEYC_WHEELDOWN_EMPTY: C2RustUnnamed_36 = 34359738377;
pub const KEYC_WHEELDOWN11_SCROLLBAR_DOWN: C2RustUnnamed_36 = 34359741192;
pub const KEYC_WHEELDOWN10_SCROLLBAR_DOWN: C2RustUnnamed_36 = 34359740936;
pub const KEYC_WHEELDOWN9_SCROLLBAR_DOWN: C2RustUnnamed_36 = 34359740680;
pub const KEYC_WHEELDOWN8_SCROLLBAR_DOWN: C2RustUnnamed_36 = 34359740424;
pub const KEYC_WHEELDOWN7_SCROLLBAR_DOWN: C2RustUnnamed_36 = 34359740168;
pub const KEYC_WHEELDOWN6_SCROLLBAR_DOWN: C2RustUnnamed_36 = 34359739912;
pub const KEYC_WHEELDOWN3_SCROLLBAR_DOWN: C2RustUnnamed_36 = 34359739144;
pub const KEYC_WHEELDOWN2_SCROLLBAR_DOWN: C2RustUnnamed_36 = 34359738888;
pub const KEYC_WHEELDOWN1_SCROLLBAR_DOWN: C2RustUnnamed_36 = 34359738632;
pub const KEYC_WHEELDOWN_SCROLLBAR_DOWN: C2RustUnnamed_36 = 34359738376;
pub const KEYC_WHEELDOWN11_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 34359741191;
pub const KEYC_WHEELDOWN10_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 34359740935;
pub const KEYC_WHEELDOWN9_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 34359740679;
pub const KEYC_WHEELDOWN8_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 34359740423;
pub const KEYC_WHEELDOWN7_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 34359740167;
pub const KEYC_WHEELDOWN6_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 34359739911;
pub const KEYC_WHEELDOWN3_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 34359739143;
pub const KEYC_WHEELDOWN2_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 34359738887;
pub const KEYC_WHEELDOWN1_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 34359738631;
pub const KEYC_WHEELDOWN_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 34359738375;
pub const KEYC_WHEELDOWN11_SCROLLBAR_UP: C2RustUnnamed_36 = 34359741190;
pub const KEYC_WHEELDOWN10_SCROLLBAR_UP: C2RustUnnamed_36 = 34359740934;
pub const KEYC_WHEELDOWN9_SCROLLBAR_UP: C2RustUnnamed_36 = 34359740678;
pub const KEYC_WHEELDOWN8_SCROLLBAR_UP: C2RustUnnamed_36 = 34359740422;
pub const KEYC_WHEELDOWN7_SCROLLBAR_UP: C2RustUnnamed_36 = 34359740166;
pub const KEYC_WHEELDOWN6_SCROLLBAR_UP: C2RustUnnamed_36 = 34359739910;
pub const KEYC_WHEELDOWN3_SCROLLBAR_UP: C2RustUnnamed_36 = 34359739142;
pub const KEYC_WHEELDOWN2_SCROLLBAR_UP: C2RustUnnamed_36 = 34359738886;
pub const KEYC_WHEELDOWN1_SCROLLBAR_UP: C2RustUnnamed_36 = 34359738630;
pub const KEYC_WHEELDOWN_SCROLLBAR_UP: C2RustUnnamed_36 = 34359738374;
pub const KEYC_WHEELDOWN11_BORDER: C2RustUnnamed_36 = 34359741189;
pub const KEYC_WHEELDOWN10_BORDER: C2RustUnnamed_36 = 34359740933;
pub const KEYC_WHEELDOWN9_BORDER: C2RustUnnamed_36 = 34359740677;
pub const KEYC_WHEELDOWN8_BORDER: C2RustUnnamed_36 = 34359740421;
pub const KEYC_WHEELDOWN7_BORDER: C2RustUnnamed_36 = 34359740165;
pub const KEYC_WHEELDOWN6_BORDER: C2RustUnnamed_36 = 34359739909;
pub const KEYC_WHEELDOWN3_BORDER: C2RustUnnamed_36 = 34359739141;
pub const KEYC_WHEELDOWN2_BORDER: C2RustUnnamed_36 = 34359738885;
pub const KEYC_WHEELDOWN1_BORDER: C2RustUnnamed_36 = 34359738629;
pub const KEYC_WHEELDOWN_BORDER: C2RustUnnamed_36 = 34359738373;
pub const KEYC_WHEELDOWN11_STATUS_DEFAULT: C2RustUnnamed_36 = 34359741188;
pub const KEYC_WHEELDOWN10_STATUS_DEFAULT: C2RustUnnamed_36 = 34359740932;
pub const KEYC_WHEELDOWN9_STATUS_DEFAULT: C2RustUnnamed_36 = 34359740676;
pub const KEYC_WHEELDOWN8_STATUS_DEFAULT: C2RustUnnamed_36 = 34359740420;
pub const KEYC_WHEELDOWN7_STATUS_DEFAULT: C2RustUnnamed_36 = 34359740164;
pub const KEYC_WHEELDOWN6_STATUS_DEFAULT: C2RustUnnamed_36 = 34359739908;
pub const KEYC_WHEELDOWN3_STATUS_DEFAULT: C2RustUnnamed_36 = 34359739140;
pub const KEYC_WHEELDOWN2_STATUS_DEFAULT: C2RustUnnamed_36 = 34359738884;
pub const KEYC_WHEELDOWN1_STATUS_DEFAULT: C2RustUnnamed_36 = 34359738628;
pub const KEYC_WHEELDOWN_STATUS_DEFAULT: C2RustUnnamed_36 = 34359738372;
pub const KEYC_WHEELDOWN11_STATUS_RIGHT: C2RustUnnamed_36 = 34359741187;
pub const KEYC_WHEELDOWN10_STATUS_RIGHT: C2RustUnnamed_36 = 34359740931;
pub const KEYC_WHEELDOWN9_STATUS_RIGHT: C2RustUnnamed_36 = 34359740675;
pub const KEYC_WHEELDOWN8_STATUS_RIGHT: C2RustUnnamed_36 = 34359740419;
pub const KEYC_WHEELDOWN7_STATUS_RIGHT: C2RustUnnamed_36 = 34359740163;
pub const KEYC_WHEELDOWN6_STATUS_RIGHT: C2RustUnnamed_36 = 34359739907;
pub const KEYC_WHEELDOWN3_STATUS_RIGHT: C2RustUnnamed_36 = 34359739139;
pub const KEYC_WHEELDOWN2_STATUS_RIGHT: C2RustUnnamed_36 = 34359738883;
pub const KEYC_WHEELDOWN1_STATUS_RIGHT: C2RustUnnamed_36 = 34359738627;
pub const KEYC_WHEELDOWN_STATUS_RIGHT: C2RustUnnamed_36 = 34359738371;
pub const KEYC_WHEELDOWN11_STATUS_LEFT: C2RustUnnamed_36 = 34359741186;
pub const KEYC_WHEELDOWN10_STATUS_LEFT: C2RustUnnamed_36 = 34359740930;
pub const KEYC_WHEELDOWN9_STATUS_LEFT: C2RustUnnamed_36 = 34359740674;
pub const KEYC_WHEELDOWN8_STATUS_LEFT: C2RustUnnamed_36 = 34359740418;
pub const KEYC_WHEELDOWN7_STATUS_LEFT: C2RustUnnamed_36 = 34359740162;
pub const KEYC_WHEELDOWN6_STATUS_LEFT: C2RustUnnamed_36 = 34359739906;
pub const KEYC_WHEELDOWN3_STATUS_LEFT: C2RustUnnamed_36 = 34359739138;
pub const KEYC_WHEELDOWN2_STATUS_LEFT: C2RustUnnamed_36 = 34359738882;
pub const KEYC_WHEELDOWN1_STATUS_LEFT: C2RustUnnamed_36 = 34359738626;
pub const KEYC_WHEELDOWN_STATUS_LEFT: C2RustUnnamed_36 = 34359738370;
pub const KEYC_WHEELDOWN11_STATUS: C2RustUnnamed_36 = 34359741185;
pub const KEYC_WHEELDOWN10_STATUS: C2RustUnnamed_36 = 34359740929;
pub const KEYC_WHEELDOWN9_STATUS: C2RustUnnamed_36 = 34359740673;
pub const KEYC_WHEELDOWN8_STATUS: C2RustUnnamed_36 = 34359740417;
pub const KEYC_WHEELDOWN7_STATUS: C2RustUnnamed_36 = 34359740161;
pub const KEYC_WHEELDOWN6_STATUS: C2RustUnnamed_36 = 34359739905;
pub const KEYC_WHEELDOWN3_STATUS: C2RustUnnamed_36 = 34359739137;
pub const KEYC_WHEELDOWN2_STATUS: C2RustUnnamed_36 = 34359738881;
pub const KEYC_WHEELDOWN1_STATUS: C2RustUnnamed_36 = 34359738625;
pub const KEYC_WHEELDOWN_STATUS: C2RustUnnamed_36 = 34359738369;
pub const KEYC_WHEELDOWN11_PANE: C2RustUnnamed_36 = 34359741184;
pub const KEYC_WHEELDOWN10_PANE: C2RustUnnamed_36 = 34359740928;
pub const KEYC_WHEELDOWN9_PANE: C2RustUnnamed_36 = 34359740672;
pub const KEYC_WHEELDOWN8_PANE: C2RustUnnamed_36 = 34359740416;
pub const KEYC_WHEELDOWN7_PANE: C2RustUnnamed_36 = 34359740160;
pub const KEYC_WHEELDOWN6_PANE: C2RustUnnamed_36 = 34359739904;
pub const KEYC_WHEELDOWN3_PANE: C2RustUnnamed_36 = 34359739136;
pub const KEYC_WHEELDOWN2_PANE: C2RustUnnamed_36 = 34359738880;
pub const KEYC_WHEELDOWN1_PANE: C2RustUnnamed_36 = 34359738624;
pub const KEYC_WHEELDOWN_PANE: C2RustUnnamed_36 = 34359738368;
pub const KEYC_MOUSEMOVE11_CONTROL9: C2RustUnnamed_36 = 12884904723;
pub const KEYC_MOUSEMOVE10_CONTROL9: C2RustUnnamed_36 = 12884904467;
pub const KEYC_MOUSEMOVE9_CONTROL9: C2RustUnnamed_36 = 12884904211;
pub const KEYC_MOUSEMOVE8_CONTROL9: C2RustUnnamed_36 = 12884903955;
pub const KEYC_MOUSEMOVE7_CONTROL9: C2RustUnnamed_36 = 12884903699;
pub const KEYC_MOUSEMOVE6_CONTROL9: C2RustUnnamed_36 = 12884903443;
pub const KEYC_MOUSEMOVE3_CONTROL9: C2RustUnnamed_36 = 12884902675;
pub const KEYC_MOUSEMOVE2_CONTROL9: C2RustUnnamed_36 = 12884902419;
pub const KEYC_MOUSEMOVE1_CONTROL9: C2RustUnnamed_36 = 12884902163;
pub const KEYC_MOUSEMOVE_CONTROL9: C2RustUnnamed_36 = 12884901907;
pub const KEYC_MOUSEMOVE11_CONTROL8: C2RustUnnamed_36 = 12884904722;
pub const KEYC_MOUSEMOVE10_CONTROL8: C2RustUnnamed_36 = 12884904466;
pub const KEYC_MOUSEMOVE9_CONTROL8: C2RustUnnamed_36 = 12884904210;
pub const KEYC_MOUSEMOVE8_CONTROL8: C2RustUnnamed_36 = 12884903954;
pub const KEYC_MOUSEMOVE7_CONTROL8: C2RustUnnamed_36 = 12884903698;
pub const KEYC_MOUSEMOVE6_CONTROL8: C2RustUnnamed_36 = 12884903442;
pub const KEYC_MOUSEMOVE3_CONTROL8: C2RustUnnamed_36 = 12884902674;
pub const KEYC_MOUSEMOVE2_CONTROL8: C2RustUnnamed_36 = 12884902418;
pub const KEYC_MOUSEMOVE1_CONTROL8: C2RustUnnamed_36 = 12884902162;
pub const KEYC_MOUSEMOVE_CONTROL8: C2RustUnnamed_36 = 12884901906;
pub const KEYC_MOUSEMOVE11_CONTROL7: C2RustUnnamed_36 = 12884904721;
pub const KEYC_MOUSEMOVE10_CONTROL7: C2RustUnnamed_36 = 12884904465;
pub const KEYC_MOUSEMOVE9_CONTROL7: C2RustUnnamed_36 = 12884904209;
pub const KEYC_MOUSEMOVE8_CONTROL7: C2RustUnnamed_36 = 12884903953;
pub const KEYC_MOUSEMOVE7_CONTROL7: C2RustUnnamed_36 = 12884903697;
pub const KEYC_MOUSEMOVE6_CONTROL7: C2RustUnnamed_36 = 12884903441;
pub const KEYC_MOUSEMOVE3_CONTROL7: C2RustUnnamed_36 = 12884902673;
pub const KEYC_MOUSEMOVE2_CONTROL7: C2RustUnnamed_36 = 12884902417;
pub const KEYC_MOUSEMOVE1_CONTROL7: C2RustUnnamed_36 = 12884902161;
pub const KEYC_MOUSEMOVE_CONTROL7: C2RustUnnamed_36 = 12884901905;
pub const KEYC_MOUSEMOVE11_CONTROL6: C2RustUnnamed_36 = 12884904720;
pub const KEYC_MOUSEMOVE10_CONTROL6: C2RustUnnamed_36 = 12884904464;
pub const KEYC_MOUSEMOVE9_CONTROL6: C2RustUnnamed_36 = 12884904208;
pub const KEYC_MOUSEMOVE8_CONTROL6: C2RustUnnamed_36 = 12884903952;
pub const KEYC_MOUSEMOVE7_CONTROL6: C2RustUnnamed_36 = 12884903696;
pub const KEYC_MOUSEMOVE6_CONTROL6: C2RustUnnamed_36 = 12884903440;
pub const KEYC_MOUSEMOVE3_CONTROL6: C2RustUnnamed_36 = 12884902672;
pub const KEYC_MOUSEMOVE2_CONTROL6: C2RustUnnamed_36 = 12884902416;
pub const KEYC_MOUSEMOVE1_CONTROL6: C2RustUnnamed_36 = 12884902160;
pub const KEYC_MOUSEMOVE_CONTROL6: C2RustUnnamed_36 = 12884901904;
pub const KEYC_MOUSEMOVE11_CONTROL5: C2RustUnnamed_36 = 12884904719;
pub const KEYC_MOUSEMOVE10_CONTROL5: C2RustUnnamed_36 = 12884904463;
pub const KEYC_MOUSEMOVE9_CONTROL5: C2RustUnnamed_36 = 12884904207;
pub const KEYC_MOUSEMOVE8_CONTROL5: C2RustUnnamed_36 = 12884903951;
pub const KEYC_MOUSEMOVE7_CONTROL5: C2RustUnnamed_36 = 12884903695;
pub const KEYC_MOUSEMOVE6_CONTROL5: C2RustUnnamed_36 = 12884903439;
pub const KEYC_MOUSEMOVE3_CONTROL5: C2RustUnnamed_36 = 12884902671;
pub const KEYC_MOUSEMOVE2_CONTROL5: C2RustUnnamed_36 = 12884902415;
pub const KEYC_MOUSEMOVE1_CONTROL5: C2RustUnnamed_36 = 12884902159;
pub const KEYC_MOUSEMOVE_CONTROL5: C2RustUnnamed_36 = 12884901903;
pub const KEYC_MOUSEMOVE11_CONTROL4: C2RustUnnamed_36 = 12884904718;
pub const KEYC_MOUSEMOVE10_CONTROL4: C2RustUnnamed_36 = 12884904462;
pub const KEYC_MOUSEMOVE9_CONTROL4: C2RustUnnamed_36 = 12884904206;
pub const KEYC_MOUSEMOVE8_CONTROL4: C2RustUnnamed_36 = 12884903950;
pub const KEYC_MOUSEMOVE7_CONTROL4: C2RustUnnamed_36 = 12884903694;
pub const KEYC_MOUSEMOVE6_CONTROL4: C2RustUnnamed_36 = 12884903438;
pub const KEYC_MOUSEMOVE3_CONTROL4: C2RustUnnamed_36 = 12884902670;
pub const KEYC_MOUSEMOVE2_CONTROL4: C2RustUnnamed_36 = 12884902414;
pub const KEYC_MOUSEMOVE1_CONTROL4: C2RustUnnamed_36 = 12884902158;
pub const KEYC_MOUSEMOVE_CONTROL4: C2RustUnnamed_36 = 12884901902;
pub const KEYC_MOUSEMOVE11_CONTROL3: C2RustUnnamed_36 = 12884904717;
pub const KEYC_MOUSEMOVE10_CONTROL3: C2RustUnnamed_36 = 12884904461;
pub const KEYC_MOUSEMOVE9_CONTROL3: C2RustUnnamed_36 = 12884904205;
pub const KEYC_MOUSEMOVE8_CONTROL3: C2RustUnnamed_36 = 12884903949;
pub const KEYC_MOUSEMOVE7_CONTROL3: C2RustUnnamed_36 = 12884903693;
pub const KEYC_MOUSEMOVE6_CONTROL3: C2RustUnnamed_36 = 12884903437;
pub const KEYC_MOUSEMOVE3_CONTROL3: C2RustUnnamed_36 = 12884902669;
pub const KEYC_MOUSEMOVE2_CONTROL3: C2RustUnnamed_36 = 12884902413;
pub const KEYC_MOUSEMOVE1_CONTROL3: C2RustUnnamed_36 = 12884902157;
pub const KEYC_MOUSEMOVE_CONTROL3: C2RustUnnamed_36 = 12884901901;
pub const KEYC_MOUSEMOVE11_CONTROL2: C2RustUnnamed_36 = 12884904716;
pub const KEYC_MOUSEMOVE10_CONTROL2: C2RustUnnamed_36 = 12884904460;
pub const KEYC_MOUSEMOVE9_CONTROL2: C2RustUnnamed_36 = 12884904204;
pub const KEYC_MOUSEMOVE8_CONTROL2: C2RustUnnamed_36 = 12884903948;
pub const KEYC_MOUSEMOVE7_CONTROL2: C2RustUnnamed_36 = 12884903692;
pub const KEYC_MOUSEMOVE6_CONTROL2: C2RustUnnamed_36 = 12884903436;
pub const KEYC_MOUSEMOVE3_CONTROL2: C2RustUnnamed_36 = 12884902668;
pub const KEYC_MOUSEMOVE2_CONTROL2: C2RustUnnamed_36 = 12884902412;
pub const KEYC_MOUSEMOVE1_CONTROL2: C2RustUnnamed_36 = 12884902156;
pub const KEYC_MOUSEMOVE_CONTROL2: C2RustUnnamed_36 = 12884901900;
pub const KEYC_MOUSEMOVE11_CONTROL1: C2RustUnnamed_36 = 12884904715;
pub const KEYC_MOUSEMOVE10_CONTROL1: C2RustUnnamed_36 = 12884904459;
pub const KEYC_MOUSEMOVE9_CONTROL1: C2RustUnnamed_36 = 12884904203;
pub const KEYC_MOUSEMOVE8_CONTROL1: C2RustUnnamed_36 = 12884903947;
pub const KEYC_MOUSEMOVE7_CONTROL1: C2RustUnnamed_36 = 12884903691;
pub const KEYC_MOUSEMOVE6_CONTROL1: C2RustUnnamed_36 = 12884903435;
pub const KEYC_MOUSEMOVE3_CONTROL1: C2RustUnnamed_36 = 12884902667;
pub const KEYC_MOUSEMOVE2_CONTROL1: C2RustUnnamed_36 = 12884902411;
pub const KEYC_MOUSEMOVE1_CONTROL1: C2RustUnnamed_36 = 12884902155;
pub const KEYC_MOUSEMOVE_CONTROL1: C2RustUnnamed_36 = 12884901899;
pub const KEYC_MOUSEMOVE11_CONTROL0: C2RustUnnamed_36 = 12884904714;
pub const KEYC_MOUSEMOVE10_CONTROL0: C2RustUnnamed_36 = 12884904458;
pub const KEYC_MOUSEMOVE9_CONTROL0: C2RustUnnamed_36 = 12884904202;
pub const KEYC_MOUSEMOVE8_CONTROL0: C2RustUnnamed_36 = 12884903946;
pub const KEYC_MOUSEMOVE7_CONTROL0: C2RustUnnamed_36 = 12884903690;
pub const KEYC_MOUSEMOVE6_CONTROL0: C2RustUnnamed_36 = 12884903434;
pub const KEYC_MOUSEMOVE3_CONTROL0: C2RustUnnamed_36 = 12884902666;
pub const KEYC_MOUSEMOVE2_CONTROL0: C2RustUnnamed_36 = 12884902410;
pub const KEYC_MOUSEMOVE1_CONTROL0: C2RustUnnamed_36 = 12884902154;
pub const KEYC_MOUSEMOVE_CONTROL0: C2RustUnnamed_36 = 12884901898;
pub const KEYC_MOUSEMOVE11_EMPTY: C2RustUnnamed_36 = 12884904713;
pub const KEYC_MOUSEMOVE10_EMPTY: C2RustUnnamed_36 = 12884904457;
pub const KEYC_MOUSEMOVE9_EMPTY: C2RustUnnamed_36 = 12884904201;
pub const KEYC_MOUSEMOVE8_EMPTY: C2RustUnnamed_36 = 12884903945;
pub const KEYC_MOUSEMOVE7_EMPTY: C2RustUnnamed_36 = 12884903689;
pub const KEYC_MOUSEMOVE6_EMPTY: C2RustUnnamed_36 = 12884903433;
pub const KEYC_MOUSEMOVE3_EMPTY: C2RustUnnamed_36 = 12884902665;
pub const KEYC_MOUSEMOVE2_EMPTY: C2RustUnnamed_36 = 12884902409;
pub const KEYC_MOUSEMOVE1_EMPTY: C2RustUnnamed_36 = 12884902153;
pub const KEYC_MOUSEMOVE_EMPTY: C2RustUnnamed_36 = 12884901897;
pub const KEYC_MOUSEMOVE11_SCROLLBAR_DOWN: C2RustUnnamed_36 = 12884904712;
pub const KEYC_MOUSEMOVE10_SCROLLBAR_DOWN: C2RustUnnamed_36 = 12884904456;
pub const KEYC_MOUSEMOVE9_SCROLLBAR_DOWN: C2RustUnnamed_36 = 12884904200;
pub const KEYC_MOUSEMOVE8_SCROLLBAR_DOWN: C2RustUnnamed_36 = 12884903944;
pub const KEYC_MOUSEMOVE7_SCROLLBAR_DOWN: C2RustUnnamed_36 = 12884903688;
pub const KEYC_MOUSEMOVE6_SCROLLBAR_DOWN: C2RustUnnamed_36 = 12884903432;
pub const KEYC_MOUSEMOVE3_SCROLLBAR_DOWN: C2RustUnnamed_36 = 12884902664;
pub const KEYC_MOUSEMOVE2_SCROLLBAR_DOWN: C2RustUnnamed_36 = 12884902408;
pub const KEYC_MOUSEMOVE1_SCROLLBAR_DOWN: C2RustUnnamed_36 = 12884902152;
pub const KEYC_MOUSEMOVE_SCROLLBAR_DOWN: C2RustUnnamed_36 = 12884901896;
pub const KEYC_MOUSEMOVE11_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 12884904711;
pub const KEYC_MOUSEMOVE10_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 12884904455;
pub const KEYC_MOUSEMOVE9_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 12884904199;
pub const KEYC_MOUSEMOVE8_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 12884903943;
pub const KEYC_MOUSEMOVE7_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 12884903687;
pub const KEYC_MOUSEMOVE6_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 12884903431;
pub const KEYC_MOUSEMOVE3_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 12884902663;
pub const KEYC_MOUSEMOVE2_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 12884902407;
pub const KEYC_MOUSEMOVE1_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 12884902151;
pub const KEYC_MOUSEMOVE_SCROLLBAR_SLIDER: C2RustUnnamed_36 = 12884901895;
pub const KEYC_MOUSEMOVE11_SCROLLBAR_UP: C2RustUnnamed_36 = 12884904710;
pub const KEYC_MOUSEMOVE10_SCROLLBAR_UP: C2RustUnnamed_36 = 12884904454;
pub const KEYC_MOUSEMOVE9_SCROLLBAR_UP: C2RustUnnamed_36 = 12884904198;
pub const KEYC_MOUSEMOVE8_SCROLLBAR_UP: C2RustUnnamed_36 = 12884903942;
pub const KEYC_MOUSEMOVE7_SCROLLBAR_UP: C2RustUnnamed_36 = 12884903686;
pub const KEYC_MOUSEMOVE6_SCROLLBAR_UP: C2RustUnnamed_36 = 12884903430;
pub const KEYC_MOUSEMOVE3_SCROLLBAR_UP: C2RustUnnamed_36 = 12884902662;
pub const KEYC_MOUSEMOVE2_SCROLLBAR_UP: C2RustUnnamed_36 = 12884902406;
pub const KEYC_MOUSEMOVE1_SCROLLBAR_UP: C2RustUnnamed_36 = 12884902150;
pub const KEYC_MOUSEMOVE_SCROLLBAR_UP: C2RustUnnamed_36 = 12884901894;
pub const KEYC_MOUSEMOVE11_BORDER: C2RustUnnamed_36 = 12884904709;
pub const KEYC_MOUSEMOVE10_BORDER: C2RustUnnamed_36 = 12884904453;
pub const KEYC_MOUSEMOVE9_BORDER: C2RustUnnamed_36 = 12884904197;
pub const KEYC_MOUSEMOVE8_BORDER: C2RustUnnamed_36 = 12884903941;
pub const KEYC_MOUSEMOVE7_BORDER: C2RustUnnamed_36 = 12884903685;
pub const KEYC_MOUSEMOVE6_BORDER: C2RustUnnamed_36 = 12884903429;
pub const KEYC_MOUSEMOVE3_BORDER: C2RustUnnamed_36 = 12884902661;
pub const KEYC_MOUSEMOVE2_BORDER: C2RustUnnamed_36 = 12884902405;
pub const KEYC_MOUSEMOVE1_BORDER: C2RustUnnamed_36 = 12884902149;
pub const KEYC_MOUSEMOVE_BORDER: C2RustUnnamed_36 = 12884901893;
pub const KEYC_MOUSEMOVE11_STATUS_DEFAULT: C2RustUnnamed_36 = 12884904708;
pub const KEYC_MOUSEMOVE10_STATUS_DEFAULT: C2RustUnnamed_36 = 12884904452;
pub const KEYC_MOUSEMOVE9_STATUS_DEFAULT: C2RustUnnamed_36 = 12884904196;
pub const KEYC_MOUSEMOVE8_STATUS_DEFAULT: C2RustUnnamed_36 = 12884903940;
pub const KEYC_MOUSEMOVE7_STATUS_DEFAULT: C2RustUnnamed_36 = 12884903684;
pub const KEYC_MOUSEMOVE6_STATUS_DEFAULT: C2RustUnnamed_36 = 12884903428;
pub const KEYC_MOUSEMOVE3_STATUS_DEFAULT: C2RustUnnamed_36 = 12884902660;
pub const KEYC_MOUSEMOVE2_STATUS_DEFAULT: C2RustUnnamed_36 = 12884902404;
pub const KEYC_MOUSEMOVE1_STATUS_DEFAULT: C2RustUnnamed_36 = 12884902148;
pub const KEYC_MOUSEMOVE_STATUS_DEFAULT: C2RustUnnamed_36 = 12884901892;
pub const KEYC_MOUSEMOVE11_STATUS_RIGHT: C2RustUnnamed_36 = 12884904707;
pub const KEYC_MOUSEMOVE10_STATUS_RIGHT: C2RustUnnamed_36 = 12884904451;
pub const KEYC_MOUSEMOVE9_STATUS_RIGHT: C2RustUnnamed_36 = 12884904195;
pub const KEYC_MOUSEMOVE8_STATUS_RIGHT: C2RustUnnamed_36 = 12884903939;
pub const KEYC_MOUSEMOVE7_STATUS_RIGHT: C2RustUnnamed_36 = 12884903683;
pub const KEYC_MOUSEMOVE6_STATUS_RIGHT: C2RustUnnamed_36 = 12884903427;
pub const KEYC_MOUSEMOVE3_STATUS_RIGHT: C2RustUnnamed_36 = 12884902659;
pub const KEYC_MOUSEMOVE2_STATUS_RIGHT: C2RustUnnamed_36 = 12884902403;
pub const KEYC_MOUSEMOVE1_STATUS_RIGHT: C2RustUnnamed_36 = 12884902147;
pub const KEYC_MOUSEMOVE_STATUS_RIGHT: C2RustUnnamed_36 = 12884901891;
pub const KEYC_MOUSEMOVE11_STATUS_LEFT: C2RustUnnamed_36 = 12884904706;
pub const KEYC_MOUSEMOVE10_STATUS_LEFT: C2RustUnnamed_36 = 12884904450;
pub const KEYC_MOUSEMOVE9_STATUS_LEFT: C2RustUnnamed_36 = 12884904194;
pub const KEYC_MOUSEMOVE8_STATUS_LEFT: C2RustUnnamed_36 = 12884903938;
pub const KEYC_MOUSEMOVE7_STATUS_LEFT: C2RustUnnamed_36 = 12884903682;
pub const KEYC_MOUSEMOVE6_STATUS_LEFT: C2RustUnnamed_36 = 12884903426;
pub const KEYC_MOUSEMOVE3_STATUS_LEFT: C2RustUnnamed_36 = 12884902658;
pub const KEYC_MOUSEMOVE2_STATUS_LEFT: C2RustUnnamed_36 = 12884902402;
pub const KEYC_MOUSEMOVE1_STATUS_LEFT: C2RustUnnamed_36 = 12884902146;
pub const KEYC_MOUSEMOVE_STATUS_LEFT: C2RustUnnamed_36 = 12884901890;
pub const KEYC_MOUSEMOVE11_STATUS: C2RustUnnamed_36 = 12884904705;
pub const KEYC_MOUSEMOVE10_STATUS: C2RustUnnamed_36 = 12884904449;
pub const KEYC_MOUSEMOVE9_STATUS: C2RustUnnamed_36 = 12884904193;
pub const KEYC_MOUSEMOVE8_STATUS: C2RustUnnamed_36 = 12884903937;
pub const KEYC_MOUSEMOVE7_STATUS: C2RustUnnamed_36 = 12884903681;
pub const KEYC_MOUSEMOVE6_STATUS: C2RustUnnamed_36 = 12884903425;
pub const KEYC_MOUSEMOVE3_STATUS: C2RustUnnamed_36 = 12884902657;
pub const KEYC_MOUSEMOVE2_STATUS: C2RustUnnamed_36 = 12884902401;
pub const KEYC_MOUSEMOVE1_STATUS: C2RustUnnamed_36 = 12884902145;
pub const KEYC_MOUSEMOVE_STATUS: C2RustUnnamed_36 = 12884901889;
pub const KEYC_MOUSEMOVE11_PANE: C2RustUnnamed_36 = 12884904704;
pub const KEYC_MOUSEMOVE10_PANE: C2RustUnnamed_36 = 12884904448;
pub const KEYC_MOUSEMOVE9_PANE: C2RustUnnamed_36 = 12884904192;
pub const KEYC_MOUSEMOVE8_PANE: C2RustUnnamed_36 = 12884903936;
pub const KEYC_MOUSEMOVE7_PANE: C2RustUnnamed_36 = 12884903680;
pub const KEYC_MOUSEMOVE6_PANE: C2RustUnnamed_36 = 12884903424;
pub const KEYC_MOUSEMOVE3_PANE: C2RustUnnamed_36 = 12884902656;
pub const KEYC_MOUSEMOVE2_PANE: C2RustUnnamed_36 = 12884902400;
pub const KEYC_MOUSEMOVE1_PANE: C2RustUnnamed_36 = 12884902144;
pub const KEYC_MOUSEMOVE_PANE: C2RustUnnamed_36 = 12884901888;
pub const KEYC_DOUBLECLICK: C2RustUnnamed_36 = 8589934643;
pub const KEYC_DRAGGING: C2RustUnnamed_36 = 8589934642;
pub const KEYC_MOUSE: C2RustUnnamed_36 = 8589934641;
pub const KEYC_REPORT_LIGHT_THEME: C2RustUnnamed_36 = 8589934640;
pub const KEYC_REPORT_DARK_THEME: C2RustUnnamed_36 = 8589934639;
pub const KEYC_KP_PERIOD: C2RustUnnamed_36 = 8589934638;
pub const KEYC_KP_ZERO: C2RustUnnamed_36 = 8589934637;
pub const KEYC_KP_ENTER: C2RustUnnamed_36 = 8589934636;
pub const KEYC_KP_THREE: C2RustUnnamed_36 = 8589934635;
pub const KEYC_KP_TWO: C2RustUnnamed_36 = 8589934634;
pub const KEYC_KP_ONE: C2RustUnnamed_36 = 8589934633;
pub const KEYC_KP_SIX: C2RustUnnamed_36 = 8589934632;
pub const KEYC_KP_FIVE: C2RustUnnamed_36 = 8589934631;
pub const KEYC_KP_FOUR: C2RustUnnamed_36 = 8589934630;
pub const KEYC_KP_PLUS: C2RustUnnamed_36 = 8589934629;
pub const KEYC_KP_NINE: C2RustUnnamed_36 = 8589934628;
pub const KEYC_KP_EIGHT: C2RustUnnamed_36 = 8589934627;
pub const KEYC_KP_SEVEN: C2RustUnnamed_36 = 8589934626;
pub const KEYC_KP_MINUS: C2RustUnnamed_36 = 8589934625;
pub const KEYC_KP_STAR: C2RustUnnamed_36 = 8589934624;
pub const KEYC_KP_SLASH: C2RustUnnamed_36 = 8589934623;
pub const KEYC_RIGHT: C2RustUnnamed_36 = 8589934622;
pub const KEYC_LEFT: C2RustUnnamed_36 = 8589934621;
pub const KEYC_DOWN: C2RustUnnamed_36 = 8589934620;
pub const KEYC_UP: C2RustUnnamed_36 = 8589934619;
pub const KEYC_BTAB: C2RustUnnamed_36 = 8589934618;
pub const KEYC_PPAGE: C2RustUnnamed_36 = 8589934617;
pub const KEYC_NPAGE: C2RustUnnamed_36 = 8589934616;
pub const KEYC_END: C2RustUnnamed_36 = 8589934615;
pub const KEYC_HOME: C2RustUnnamed_36 = 8589934614;
pub const KEYC_DC: C2RustUnnamed_36 = 8589934613;
pub const KEYC_IC: C2RustUnnamed_36 = 8589934612;
pub const KEYC_F12: C2RustUnnamed_36 = 8589934611;
pub const KEYC_F11: C2RustUnnamed_36 = 8589934610;
pub const KEYC_F10: C2RustUnnamed_36 = 8589934609;
pub const KEYC_F9: C2RustUnnamed_36 = 8589934608;
pub const KEYC_F8: C2RustUnnamed_36 = 8589934607;
pub const KEYC_F7: C2RustUnnamed_36 = 8589934606;
pub const KEYC_F6: C2RustUnnamed_36 = 8589934605;
pub const KEYC_F5: C2RustUnnamed_36 = 8589934604;
pub const KEYC_F4: C2RustUnnamed_36 = 8589934603;
pub const KEYC_F3: C2RustUnnamed_36 = 8589934602;
pub const KEYC_F2: C2RustUnnamed_36 = 8589934601;
pub const KEYC_F1: C2RustUnnamed_36 = 8589934600;
pub const KEYC_BSPACE: C2RustUnnamed_36 = 8589934599;
pub const KEYC_PASTE_END: C2RustUnnamed_36 = 8589934598;
pub const KEYC_PASTE_START: C2RustUnnamed_36 = 8589934597;
pub const KEYC_ANY: C2RustUnnamed_36 = 8589934596;
pub const KEYC_FOCUS_OUT: C2RustUnnamed_36 = 8589934595;
pub const KEYC_FOCUS_IN: C2RustUnnamed_36 = 8589934594;
pub const KEYC_UNKNOWN: C2RustUnnamed_36 = 8589934593;
pub const KEYC_NONE: C2RustUnnamed_36 = 8589934592;
pub const KEYC_USER: C2RustUnnamed_36 = 4294967296;
pub type pane_lines = ::core::ffi::c_uint;
pub const PANE_LINES_ROUNDED: pane_lines = 7;
pub const PANE_LINES_NONE: pane_lines = 6;
pub const PANE_LINES_SPACES: pane_lines = 5;
pub const PANE_LINES_NUMBER: pane_lines = 4;
pub const PANE_LINES_SIMPLE: pane_lines = 3;
pub const PANE_LINES_HEAVY: pane_lines = 2;
pub const PANE_LINES_DOUBLE: pane_lines = 1;
pub const PANE_LINES_SINGLE: pane_lines = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_panes_zindex {
    pub tqh_first: *mut window_pane,
    pub tqh_last: *mut *mut window_pane,
}
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
pub type prompt_input_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *const ::core::ffi::c_char,
        prompt_key_result,
    ) -> prompt_result,
>;
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
pub struct clients {
    pub tqh_first: *mut client,
    pub tqh_last: *mut *mut client,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_pane_input_data {
    pub item: *mut cmdq_item,
    pub wp: u_int,
    pub file: *mut client_file,
}
pub const TIOCSWINSZ: ::core::ffi::c_int = 0x5414 as ::core::ffi::c_int;
pub const FIONREAD: ::core::ffi::c_int = 0x541b as ::core::ffi::c_int;
pub const SIGCHLD: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const FNM_CASEFOLD: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << 4 as ::core::ffi::c_int;
pub const REG_EXTENDED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const REG_ICASE: ::core::ffi::c_int = (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const RB_BLACK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const RB_RED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const RB_INF: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const EV_READ: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const EV_WRITE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const DEFAULT_XPIXEL: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const DEFAULT_YPIXEL: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const KEYC_MASK_TYPE: ::core::ffi::c_ulonglong = 0xff00000000 as ::core::ffi::c_ulonglong;
pub const KEYC_MASK_FLAGS: ::core::ffi::c_ulonglong = 0xff000000000000 as ::core::ffi::c_ulonglong;
pub const KEYC_MASK_KEY: ::core::ffi::c_ulonglong = 0xffffffffff as ::core::ffi::c_ulonglong;
pub const MODE_BRACKETPASTE: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const MODE_FOCUSON: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const MODE_THEME_UPDATES: ::core::ffi::c_int = 0x80000 as ::core::ffi::c_int;
pub const WINDOW_PANE_NO_MODE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const WINDOW_PANE_COPY_MODE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const WINDOW_PANE_VIEW_MODE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const WINDOW_MODE_HIDE_PANE_STATUS: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINDOW_MODE_NO_STACK: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const WINDOW_MODE_HIDE_SCROLLBARS: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const PANE_REDRAW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_FOCUSED: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const PANE_VISITED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const PANE_ZOOMED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const PANE_INPUTOFF: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const PANE_CHANGED: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const PANE_EXITED: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const PANE_STATUSREADY: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const PANE_EMPTY: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const PANE_STYLECHANGED: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const PANE_THEMECHANGED: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const PANE_UNSEENCHANGES: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const PANE_REDRAWSCROLLBAR: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
pub const PANE_DESTROYED: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const PANE_FLOATOVERZOOM: ::core::ffi::c_int = 0x200000 as ::core::ffi::c_int;
pub const WINDOW_BELL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINDOW_ACTIVITY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const WINDOW_SILENCE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const WINDOW_ZOOMED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const WINDOW_WASZOOMED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const WINDOW_ALERTFLAGS: ::core::ffi::c_int = WINDOW_BELL | WINDOW_ACTIVITY | WINDOW_SILENCE;
pub const WINLINK_BELL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINLINK_ACTIVITY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const WINLINK_SILENCE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const WINLINK_ALERTFLAGS: ::core::ffi::c_int =
    WINLINK_BELL | WINLINK_ACTIVITY | WINLINK_SILENCE;
pub const WINLINK_VISITED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const PANE_STATUS_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_STATUS_TOP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_STATUS_BOTTOM: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PANE_STATUS_TOP_FLOATING: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const PANE_STATUS_BOTTOM_FLOATING: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_MODAL: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_ALWAYS: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_AUTOHIDE: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_LEFT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LAYOUT_CELL_FLOATING: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MOUSE_MASK_BUTTONS: ::core::ffi::c_int = 195 as ::core::ffi::c_int;
pub const MOUSE_MASK_DRAG: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CLIENT_EXIT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CLIENT_SUSPENDED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLIENT_EXITED: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const CLIENT_DEAD: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CLIENT_FOCUSED: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
pub const CLIENT_UNATTACHEDFLAGS: ::core::ffi::c_int = CLIENT_DEAD | CLIENT_SUSPENDED | CLIENT_EXIT;
pub const SPAWN_BEFORE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const SPAWN_FULLSIZE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const SPAWN_FLOATING: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
#[no_mangle]
pub static mut windows: windows = windows {
    rbh_root: ::core::ptr::null::<window>() as *mut window,
};
#[no_mangle]
pub static mut all_window_panes: window_pane_tree = window_pane_tree {
    rbh_root: ::core::ptr::null::<window_pane>() as *mut window_pane,
};
static mut next_window_pane_id: u_int = 0;
static mut next_window_id: u_int = 0;
static mut next_active_point: u_int = 0;
#[no_mangle]
pub unsafe extern "C" fn windows_RB_NEXT(mut elm: *mut window) -> *mut window {
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
#[no_mangle]
pub unsafe extern "C" fn windows_RB_NFIND(
    mut head: *mut windows,
    mut elm: *mut window,
) -> *mut window {
    let mut tmp: *mut window = (*head).rbh_root;
    let mut res: *mut window = ::core::ptr::null_mut::<window>();
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = window_cmp(elm, tmp);
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
pub unsafe extern "C" fn windows_RB_MINMAX(
    mut head: *mut windows,
    mut val: ::core::ffi::c_int,
) -> *mut window {
    let mut tmp: *mut window = (*head).rbh_root;
    let mut parent: *mut window = ::core::ptr::null_mut::<window>();
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
pub unsafe extern "C" fn windows_RB_INSERT(
    mut head: *mut windows,
    mut elm: *mut window,
) -> *mut window {
    let mut tmp: *mut window = ::core::ptr::null_mut::<window>();
    let mut parent: *mut window = ::core::ptr::null_mut::<window>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = window_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<window>();
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
    windows_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<window>();
}
#[no_mangle]
pub unsafe extern "C" fn windows_RB_PREV(mut elm: *mut window) -> *mut window {
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
pub unsafe extern "C" fn windows_RB_FIND(
    mut head: *mut windows,
    mut elm: *mut window,
) -> *mut window {
    let mut tmp: *mut window = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = window_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<window>();
}
#[no_mangle]
pub unsafe extern "C" fn windows_RB_REMOVE(
    mut head: *mut windows,
    mut elm: *mut window,
) -> *mut window {
    let mut current_block: u64;
    let mut child: *mut window = ::core::ptr::null_mut::<window>();
    let mut parent: *mut window = ::core::ptr::null_mut::<window>();
    let mut old: *mut window = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut window = ::core::ptr::null_mut::<window>();
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
        current_block = 11459827966151058445;
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
        windows_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
#[no_mangle]
pub unsafe extern "C" fn windows_RB_REMOVE_COLOR(
    mut head: *mut windows,
    mut parent: *mut window,
    mut elm: *mut window,
) {
    let mut tmp: *mut window = ::core::ptr::null_mut::<window>();
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
                    let mut oleft: *mut window = ::core::ptr::null_mut::<window>();
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
                    let mut oright: *mut window = ::core::ptr::null_mut::<window>();
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
pub unsafe extern "C" fn windows_RB_INSERT_COLOR(mut head: *mut windows, mut elm: *mut window) {
    let mut parent: *mut window = ::core::ptr::null_mut::<window>();
    let mut gparent: *mut window = ::core::ptr::null_mut::<window>();
    let mut tmp: *mut window = ::core::ptr::null_mut::<window>();
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
pub unsafe extern "C" fn winlinks_RB_INSERT_COLOR(mut head: *mut winlinks, mut elm: *mut winlink) {
    let mut parent: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut gparent: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut tmp: *mut winlink = ::core::ptr::null_mut::<winlink>();
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
pub unsafe extern "C" fn winlinks_RB_REMOVE_COLOR(
    mut head: *mut winlinks,
    mut parent: *mut winlink,
    mut elm: *mut winlink,
) {
    let mut tmp: *mut winlink = ::core::ptr::null_mut::<winlink>();
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
                    let mut oleft: *mut winlink = ::core::ptr::null_mut::<winlink>();
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
                    let mut oright: *mut winlink = ::core::ptr::null_mut::<winlink>();
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
pub unsafe extern "C" fn winlinks_RB_REMOVE(
    mut head: *mut winlinks,
    mut elm: *mut winlink,
) -> *mut winlink {
    let mut current_block: u64;
    let mut child: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut parent: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut old: *mut winlink = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).entry.rbe_left.is_null() {
        child = (*elm).entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).entry.rbe_right.is_null() {
        child = (*elm).entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut winlink = ::core::ptr::null_mut::<winlink>();
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
        current_block = 11350272964138407771;
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
        winlinks_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
#[no_mangle]
pub unsafe extern "C" fn winlinks_RB_INSERT(
    mut head: *mut winlinks,
    mut elm: *mut winlink,
) -> *mut winlink {
    let mut tmp: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut parent: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = winlink_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).entry.rbe_parent = parent;
    (*elm).entry.rbe_right = ::core::ptr::null_mut::<winlink>();
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
    winlinks_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<winlink>();
}
#[no_mangle]
pub unsafe extern "C" fn winlinks_RB_NFIND(
    mut head: *mut winlinks,
    mut elm: *mut winlink,
) -> *mut winlink {
    let mut tmp: *mut winlink = (*head).rbh_root;
    let mut res: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = winlink_cmp(elm, tmp);
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
pub unsafe extern "C" fn winlinks_RB_MINMAX(
    mut head: *mut winlinks,
    mut val: ::core::ffi::c_int,
) -> *mut winlink {
    let mut tmp: *mut winlink = (*head).rbh_root;
    let mut parent: *mut winlink = ::core::ptr::null_mut::<winlink>();
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
pub unsafe extern "C" fn winlinks_RB_PREV(mut elm: *mut winlink) -> *mut winlink {
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
pub unsafe extern "C" fn winlinks_RB_NEXT(mut elm: *mut winlink) -> *mut winlink {
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
#[no_mangle]
pub unsafe extern "C" fn winlinks_RB_FIND(
    mut head: *mut winlinks,
    mut elm: *mut winlink,
) -> *mut winlink {
    let mut tmp: *mut winlink = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = winlink_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<winlink>();
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_FIND(
    mut head: *mut window_pane_tree,
    mut elm: *mut window_pane,
) -> *mut window_pane {
    let mut tmp: *mut window_pane = (*head).rbh_root;
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = window_pane_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).tree_entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).tree_entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return ::core::ptr::null_mut::<window_pane>();
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_NFIND(
    mut head: *mut window_pane_tree,
    mut elm: *mut window_pane,
) -> *mut window_pane {
    let mut tmp: *mut window_pane = (*head).rbh_root;
    let mut res: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut comp: ::core::ffi::c_int = 0;
    while !tmp.is_null() {
        comp = window_pane_cmp(elm, tmp);
        if comp < 0 as ::core::ffi::c_int {
            res = tmp;
            tmp = (*tmp).tree_entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).tree_entry.rbe_right;
        } else {
            return tmp;
        }
    }
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_PREV(mut elm: *mut window_pane) -> *mut window_pane {
    if !(*elm).tree_entry.rbe_left.is_null() {
        elm = (*elm).tree_entry.rbe_left;
        while !(*elm).tree_entry.rbe_right.is_null() {
            elm = (*elm).tree_entry.rbe_right;
        }
    } else if !(*elm).tree_entry.rbe_parent.is_null()
        && elm == (*(*elm).tree_entry.rbe_parent).tree_entry.rbe_right
    {
        elm = (*elm).tree_entry.rbe_parent;
    } else {
        while !(*elm).tree_entry.rbe_parent.is_null()
            && elm == (*(*elm).tree_entry.rbe_parent).tree_entry.rbe_left
        {
            elm = (*elm).tree_entry.rbe_parent;
        }
        elm = (*elm).tree_entry.rbe_parent;
    }
    return elm;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_MINMAX(
    mut head: *mut window_pane_tree,
    mut val: ::core::ffi::c_int,
) -> *mut window_pane {
    let mut tmp: *mut window_pane = (*head).rbh_root;
    let mut parent: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    while !tmp.is_null() {
        parent = tmp;
        if val < 0 as ::core::ffi::c_int {
            tmp = (*tmp).tree_entry.rbe_left;
        } else {
            tmp = (*tmp).tree_entry.rbe_right;
        }
    }
    return parent;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_REMOVE_COLOR(
    mut head: *mut window_pane_tree,
    mut parent: *mut window_pane,
    mut elm: *mut window_pane,
) {
    let mut tmp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    while (elm.is_null() || (*elm).tree_entry.rbe_color == RB_BLACK) && elm != (*head).rbh_root {
        if (*parent).tree_entry.rbe_left == elm {
            tmp = (*parent).tree_entry.rbe_right;
            if (*tmp).tree_entry.rbe_color == RB_RED {
                (*tmp).tree_entry.rbe_color = RB_BLACK;
                (*parent).tree_entry.rbe_color = RB_RED;
                tmp = (*parent).tree_entry.rbe_right;
                (*parent).tree_entry.rbe_right = (*tmp).tree_entry.rbe_left;
                if !(*parent).tree_entry.rbe_right.is_null() {
                    (*(*tmp).tree_entry.rbe_left).tree_entry.rbe_parent = parent;
                }
                (*tmp).tree_entry.rbe_parent = (*parent).tree_entry.rbe_parent;
                if !(*tmp).tree_entry.rbe_parent.is_null() {
                    if parent == (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left {
                        (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left = tmp;
                    } else {
                        (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).tree_entry.rbe_left = parent;
                (*parent).tree_entry.rbe_parent = tmp;
                !(*tmp).tree_entry.rbe_parent.is_null();
                tmp = (*parent).tree_entry.rbe_right;
            }
            if ((*tmp).tree_entry.rbe_left.is_null()
                || (*(*tmp).tree_entry.rbe_left).tree_entry.rbe_color == RB_BLACK)
                && ((*tmp).tree_entry.rbe_right.is_null()
                    || (*(*tmp).tree_entry.rbe_right).tree_entry.rbe_color == RB_BLACK)
            {
                (*tmp).tree_entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).tree_entry.rbe_parent;
            } else {
                if (*tmp).tree_entry.rbe_right.is_null()
                    || (*(*tmp).tree_entry.rbe_right).tree_entry.rbe_color == RB_BLACK
                {
                    let mut oleft: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
                    oleft = (*tmp).tree_entry.rbe_left;
                    if !oleft.is_null() {
                        (*oleft).tree_entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).tree_entry.rbe_color = RB_RED;
                    oleft = (*tmp).tree_entry.rbe_left;
                    (*tmp).tree_entry.rbe_left = (*oleft).tree_entry.rbe_right;
                    if !(*tmp).tree_entry.rbe_left.is_null() {
                        (*(*oleft).tree_entry.rbe_right).tree_entry.rbe_parent = tmp;
                    }
                    (*oleft).tree_entry.rbe_parent = (*tmp).tree_entry.rbe_parent;
                    if !(*oleft).tree_entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).tree_entry.rbe_parent).tree_entry.rbe_left {
                            (*(*tmp).tree_entry.rbe_parent).tree_entry.rbe_left = oleft;
                        } else {
                            (*(*tmp).tree_entry.rbe_parent).tree_entry.rbe_right = oleft;
                        }
                    } else {
                        (*head).rbh_root = oleft;
                    }
                    (*oleft).tree_entry.rbe_right = tmp;
                    (*tmp).tree_entry.rbe_parent = oleft;
                    !(*oleft).tree_entry.rbe_parent.is_null();
                    tmp = (*parent).tree_entry.rbe_right;
                }
                (*tmp).tree_entry.rbe_color = (*parent).tree_entry.rbe_color;
                (*parent).tree_entry.rbe_color = RB_BLACK;
                if !(*tmp).tree_entry.rbe_right.is_null() {
                    (*(*tmp).tree_entry.rbe_right).tree_entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).tree_entry.rbe_right;
                (*parent).tree_entry.rbe_right = (*tmp).tree_entry.rbe_left;
                if !(*parent).tree_entry.rbe_right.is_null() {
                    (*(*tmp).tree_entry.rbe_left).tree_entry.rbe_parent = parent;
                }
                (*tmp).tree_entry.rbe_parent = (*parent).tree_entry.rbe_parent;
                if !(*tmp).tree_entry.rbe_parent.is_null() {
                    if parent == (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left {
                        (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left = tmp;
                    } else {
                        (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).tree_entry.rbe_left = parent;
                (*parent).tree_entry.rbe_parent = tmp;
                !(*tmp).tree_entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        } else {
            tmp = (*parent).tree_entry.rbe_left;
            if (*tmp).tree_entry.rbe_color == RB_RED {
                (*tmp).tree_entry.rbe_color = RB_BLACK;
                (*parent).tree_entry.rbe_color = RB_RED;
                tmp = (*parent).tree_entry.rbe_left;
                (*parent).tree_entry.rbe_left = (*tmp).tree_entry.rbe_right;
                if !(*parent).tree_entry.rbe_left.is_null() {
                    (*(*tmp).tree_entry.rbe_right).tree_entry.rbe_parent = parent;
                }
                (*tmp).tree_entry.rbe_parent = (*parent).tree_entry.rbe_parent;
                if !(*tmp).tree_entry.rbe_parent.is_null() {
                    if parent == (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left {
                        (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left = tmp;
                    } else {
                        (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).tree_entry.rbe_right = parent;
                (*parent).tree_entry.rbe_parent = tmp;
                !(*tmp).tree_entry.rbe_parent.is_null();
                tmp = (*parent).tree_entry.rbe_left;
            }
            if ((*tmp).tree_entry.rbe_left.is_null()
                || (*(*tmp).tree_entry.rbe_left).tree_entry.rbe_color == RB_BLACK)
                && ((*tmp).tree_entry.rbe_right.is_null()
                    || (*(*tmp).tree_entry.rbe_right).tree_entry.rbe_color == RB_BLACK)
            {
                (*tmp).tree_entry.rbe_color = RB_RED;
                elm = parent;
                parent = (*elm).tree_entry.rbe_parent;
            } else {
                if (*tmp).tree_entry.rbe_left.is_null()
                    || (*(*tmp).tree_entry.rbe_left).tree_entry.rbe_color == RB_BLACK
                {
                    let mut oright: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
                    oright = (*tmp).tree_entry.rbe_right;
                    if !oright.is_null() {
                        (*oright).tree_entry.rbe_color = RB_BLACK;
                    }
                    (*tmp).tree_entry.rbe_color = RB_RED;
                    oright = (*tmp).tree_entry.rbe_right;
                    (*tmp).tree_entry.rbe_right = (*oright).tree_entry.rbe_left;
                    if !(*tmp).tree_entry.rbe_right.is_null() {
                        (*(*oright).tree_entry.rbe_left).tree_entry.rbe_parent = tmp;
                    }
                    (*oright).tree_entry.rbe_parent = (*tmp).tree_entry.rbe_parent;
                    if !(*oright).tree_entry.rbe_parent.is_null() {
                        if tmp == (*(*tmp).tree_entry.rbe_parent).tree_entry.rbe_left {
                            (*(*tmp).tree_entry.rbe_parent).tree_entry.rbe_left = oright;
                        } else {
                            (*(*tmp).tree_entry.rbe_parent).tree_entry.rbe_right = oright;
                        }
                    } else {
                        (*head).rbh_root = oright;
                    }
                    (*oright).tree_entry.rbe_left = tmp;
                    (*tmp).tree_entry.rbe_parent = oright;
                    !(*oright).tree_entry.rbe_parent.is_null();
                    tmp = (*parent).tree_entry.rbe_left;
                }
                (*tmp).tree_entry.rbe_color = (*parent).tree_entry.rbe_color;
                (*parent).tree_entry.rbe_color = RB_BLACK;
                if !(*tmp).tree_entry.rbe_left.is_null() {
                    (*(*tmp).tree_entry.rbe_left).tree_entry.rbe_color = RB_BLACK;
                }
                tmp = (*parent).tree_entry.rbe_left;
                (*parent).tree_entry.rbe_left = (*tmp).tree_entry.rbe_right;
                if !(*parent).tree_entry.rbe_left.is_null() {
                    (*(*tmp).tree_entry.rbe_right).tree_entry.rbe_parent = parent;
                }
                (*tmp).tree_entry.rbe_parent = (*parent).tree_entry.rbe_parent;
                if !(*tmp).tree_entry.rbe_parent.is_null() {
                    if parent == (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left {
                        (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left = tmp;
                    } else {
                        (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).tree_entry.rbe_right = parent;
                (*parent).tree_entry.rbe_parent = tmp;
                !(*tmp).tree_entry.rbe_parent.is_null();
                elm = (*head).rbh_root;
                break;
            }
        }
    }
    if !elm.is_null() {
        (*elm).tree_entry.rbe_color = RB_BLACK;
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_REMOVE(
    mut head: *mut window_pane_tree,
    mut elm: *mut window_pane,
) -> *mut window_pane {
    let mut current_block: u64;
    let mut child: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut parent: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut old: *mut window_pane = elm;
    let mut color: ::core::ffi::c_int = 0;
    if (*elm).tree_entry.rbe_left.is_null() {
        child = (*elm).tree_entry.rbe_right;
        current_block = 7245201122033322888;
    } else if (*elm).tree_entry.rbe_right.is_null() {
        child = (*elm).tree_entry.rbe_left;
        current_block = 7245201122033322888;
    } else {
        let mut left: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
        elm = (*elm).tree_entry.rbe_right;
        loop {
            left = (*elm).tree_entry.rbe_left;
            if left.is_null() {
                break;
            }
            elm = left;
        }
        child = (*elm).tree_entry.rbe_right;
        parent = (*elm).tree_entry.rbe_parent;
        color = (*elm).tree_entry.rbe_color;
        if !child.is_null() {
            (*child).tree_entry.rbe_parent = parent;
        }
        if !parent.is_null() {
            if (*parent).tree_entry.rbe_left == elm {
                (*parent).tree_entry.rbe_left = child;
            } else {
                (*parent).tree_entry.rbe_right = child;
            }
        } else {
            (*head).rbh_root = child;
        }
        if (*elm).tree_entry.rbe_parent == old {
            parent = elm;
        }
        (*elm).tree_entry = (*old).tree_entry;
        if !(*old).tree_entry.rbe_parent.is_null() {
            if (*(*old).tree_entry.rbe_parent).tree_entry.rbe_left == old {
                (*(*old).tree_entry.rbe_parent).tree_entry.rbe_left = elm;
            } else {
                (*(*old).tree_entry.rbe_parent).tree_entry.rbe_right = elm;
            }
        } else {
            (*head).rbh_root = elm;
        }
        (*(*old).tree_entry.rbe_left).tree_entry.rbe_parent = elm;
        if !(*old).tree_entry.rbe_right.is_null() {
            (*(*old).tree_entry.rbe_right).tree_entry.rbe_parent = elm;
        }
        if !parent.is_null() {
            left = parent;
            loop {
                left = (*left).tree_entry.rbe_parent;
                if left.is_null() {
                    break;
                }
            }
        }
        current_block = 6103297587398279902;
    }
    match current_block {
        7245201122033322888 => {
            parent = (*elm).tree_entry.rbe_parent;
            color = (*elm).tree_entry.rbe_color;
            if !child.is_null() {
                (*child).tree_entry.rbe_parent = parent;
            }
            if !parent.is_null() {
                if (*parent).tree_entry.rbe_left == elm {
                    (*parent).tree_entry.rbe_left = child;
                } else {
                    (*parent).tree_entry.rbe_right = child;
                }
            } else {
                (*head).rbh_root = child;
            }
        }
        _ => {}
    }
    if color == RB_BLACK {
        window_pane_tree_RB_REMOVE_COLOR(head, parent, child);
    }
    return old;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_INSERT_COLOR(
    mut head: *mut window_pane_tree,
    mut elm: *mut window_pane,
) {
    let mut parent: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut gparent: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut tmp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    loop {
        parent = (*elm).tree_entry.rbe_parent;
        if !(!parent.is_null() && (*parent).tree_entry.rbe_color == RB_RED) {
            break;
        }
        gparent = (*parent).tree_entry.rbe_parent;
        if parent == (*gparent).tree_entry.rbe_left {
            tmp = (*gparent).tree_entry.rbe_right;
            if !tmp.is_null() && (*tmp).tree_entry.rbe_color == RB_RED {
                (*tmp).tree_entry.rbe_color = RB_BLACK;
                (*parent).tree_entry.rbe_color = RB_BLACK;
                (*gparent).tree_entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).tree_entry.rbe_right == elm {
                    tmp = (*parent).tree_entry.rbe_right;
                    (*parent).tree_entry.rbe_right = (*tmp).tree_entry.rbe_left;
                    if !(*parent).tree_entry.rbe_right.is_null() {
                        (*(*tmp).tree_entry.rbe_left).tree_entry.rbe_parent = parent;
                    }
                    (*tmp).tree_entry.rbe_parent = (*parent).tree_entry.rbe_parent;
                    if !(*tmp).tree_entry.rbe_parent.is_null() {
                        if parent == (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left {
                            (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left = tmp;
                        } else {
                            (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).tree_entry.rbe_left = parent;
                    (*parent).tree_entry.rbe_parent = tmp;
                    !(*tmp).tree_entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).tree_entry.rbe_color = RB_BLACK;
                (*gparent).tree_entry.rbe_color = RB_RED;
                tmp = (*gparent).tree_entry.rbe_left;
                (*gparent).tree_entry.rbe_left = (*tmp).tree_entry.rbe_right;
                if !(*gparent).tree_entry.rbe_left.is_null() {
                    (*(*tmp).tree_entry.rbe_right).tree_entry.rbe_parent = gparent;
                }
                (*tmp).tree_entry.rbe_parent = (*gparent).tree_entry.rbe_parent;
                if !(*tmp).tree_entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).tree_entry.rbe_parent).tree_entry.rbe_left {
                        (*(*gparent).tree_entry.rbe_parent).tree_entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).tree_entry.rbe_parent).tree_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).tree_entry.rbe_right = gparent;
                (*gparent).tree_entry.rbe_parent = tmp;
                !(*tmp).tree_entry.rbe_parent.is_null();
            }
        } else {
            tmp = (*gparent).tree_entry.rbe_left;
            if !tmp.is_null() && (*tmp).tree_entry.rbe_color == RB_RED {
                (*tmp).tree_entry.rbe_color = RB_BLACK;
                (*parent).tree_entry.rbe_color = RB_BLACK;
                (*gparent).tree_entry.rbe_color = RB_RED;
                elm = gparent;
            } else {
                if (*parent).tree_entry.rbe_left == elm {
                    tmp = (*parent).tree_entry.rbe_left;
                    (*parent).tree_entry.rbe_left = (*tmp).tree_entry.rbe_right;
                    if !(*parent).tree_entry.rbe_left.is_null() {
                        (*(*tmp).tree_entry.rbe_right).tree_entry.rbe_parent = parent;
                    }
                    (*tmp).tree_entry.rbe_parent = (*parent).tree_entry.rbe_parent;
                    if !(*tmp).tree_entry.rbe_parent.is_null() {
                        if parent == (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left {
                            (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_left = tmp;
                        } else {
                            (*(*parent).tree_entry.rbe_parent).tree_entry.rbe_right = tmp;
                        }
                    } else {
                        (*head).rbh_root = tmp;
                    }
                    (*tmp).tree_entry.rbe_right = parent;
                    (*parent).tree_entry.rbe_parent = tmp;
                    !(*tmp).tree_entry.rbe_parent.is_null();
                    tmp = parent;
                    parent = elm;
                    elm = tmp;
                }
                (*parent).tree_entry.rbe_color = RB_BLACK;
                (*gparent).tree_entry.rbe_color = RB_RED;
                tmp = (*gparent).tree_entry.rbe_right;
                (*gparent).tree_entry.rbe_right = (*tmp).tree_entry.rbe_left;
                if !(*gparent).tree_entry.rbe_right.is_null() {
                    (*(*tmp).tree_entry.rbe_left).tree_entry.rbe_parent = gparent;
                }
                (*tmp).tree_entry.rbe_parent = (*gparent).tree_entry.rbe_parent;
                if !(*tmp).tree_entry.rbe_parent.is_null() {
                    if gparent == (*(*gparent).tree_entry.rbe_parent).tree_entry.rbe_left {
                        (*(*gparent).tree_entry.rbe_parent).tree_entry.rbe_left = tmp;
                    } else {
                        (*(*gparent).tree_entry.rbe_parent).tree_entry.rbe_right = tmp;
                    }
                } else {
                    (*head).rbh_root = tmp;
                }
                (*tmp).tree_entry.rbe_left = gparent;
                (*gparent).tree_entry.rbe_parent = tmp;
                !(*tmp).tree_entry.rbe_parent.is_null();
            }
        }
    }
    (*(*head).rbh_root).tree_entry.rbe_color = RB_BLACK;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_INSERT(
    mut head: *mut window_pane_tree,
    mut elm: *mut window_pane,
) -> *mut window_pane {
    let mut tmp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut parent: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut comp: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    tmp = (*head).rbh_root;
    while !tmp.is_null() {
        parent = tmp;
        comp = window_pane_cmp(elm, parent);
        if comp < 0 as ::core::ffi::c_int {
            tmp = (*tmp).tree_entry.rbe_left;
        } else if comp > 0 as ::core::ffi::c_int {
            tmp = (*tmp).tree_entry.rbe_right;
        } else {
            return tmp;
        }
    }
    (*elm).tree_entry.rbe_parent = parent;
    (*elm).tree_entry.rbe_right = ::core::ptr::null_mut::<window_pane>();
    (*elm).tree_entry.rbe_left = (*elm).tree_entry.rbe_right;
    (*elm).tree_entry.rbe_color = RB_RED;
    if !parent.is_null() {
        if comp < 0 as ::core::ffi::c_int {
            (*parent).tree_entry.rbe_left = elm;
        } else {
            (*parent).tree_entry.rbe_right = elm;
        }
    } else {
        (*head).rbh_root = elm;
    }
    window_pane_tree_RB_INSERT_COLOR(head, elm);
    return ::core::ptr::null_mut::<window_pane>();
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_tree_RB_NEXT(mut elm: *mut window_pane) -> *mut window_pane {
    if !(*elm).tree_entry.rbe_right.is_null() {
        elm = (*elm).tree_entry.rbe_right;
        while !(*elm).tree_entry.rbe_left.is_null() {
            elm = (*elm).tree_entry.rbe_left;
        }
    } else if !(*elm).tree_entry.rbe_parent.is_null()
        && elm == (*(*elm).tree_entry.rbe_parent).tree_entry.rbe_left
    {
        elm = (*elm).tree_entry.rbe_parent;
    } else {
        while !(*elm).tree_entry.rbe_parent.is_null()
            && elm == (*(*elm).tree_entry.rbe_parent).tree_entry.rbe_right
        {
            elm = (*elm).tree_entry.rbe_parent;
        }
        elm = (*elm).tree_entry.rbe_parent;
    }
    return elm;
}
#[no_mangle]
pub unsafe extern "C" fn window_cmp(
    mut w1: *mut window,
    mut w2: *mut window,
) -> ::core::ffi::c_int {
    return (*w1).id.wrapping_sub((*w2).id) as ::core::ffi::c_int;
}
unsafe extern "C" fn window_fire_renamed(
    mut w: *mut window,
    mut old_name: *const ::core::ffi::c_char,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_window(&raw mut fs, w, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    event_payload_set_string(
        ep,
        b"old_name\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        old_name,
    );
    event_payload_set_string(
        ep,
        b"new_name\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).name,
    );
    events_fire(
        b"window-renamed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
unsafe extern "C" fn window_fire_pane_changed(
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut lastwp: *mut window_pane,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    event_payload_set_pane(
        ep,
        b"new_pane\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
    );
    if !lastwp.is_null() {
        event_payload_set_pane(
            ep,
            b"old_pane\0" as *const u8 as *const ::core::ffi::c_char,
            lastwp,
        );
    }
    events_fire(
        b"window-pane-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
#[no_mangle]
pub unsafe extern "C" fn window_fire_pane_moved(
    mut wp: *mut window_pane,
    mut old_w: *mut window,
    mut old_idx: ::core::ffi::c_int,
    mut new_w: *mut window,
    mut new_idx: ::core::ffi::c_int,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        new_w,
    );
    event_payload_set_window(
        ep,
        b"old_window\0" as *const u8 as *const ::core::ffi::c_char,
        old_w,
    );
    event_payload_set_window(
        ep,
        b"new_window\0" as *const u8 as *const ::core::ffi::c_char,
        new_w,
    );
    if old_idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            ep,
            b"old_window_index\0" as *const u8 as *const ::core::ffi::c_char,
            old_idx,
        );
    }
    if new_idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            new_idx,
        );
        event_payload_set_int(
            ep,
            b"new_window_index\0" as *const u8 as *const ::core::ffi::c_char,
            new_idx,
        );
    }
    events_fire(
        b"pane-moved\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
unsafe extern "C" fn window_fire_pane_mode_changed(
    mut name: *const ::core::ffi::c_char,
    mut wp: *mut window_pane,
    mut previous: *const ::core::ffi::c_char,
    mut current: *const ::core::ffi::c_char,
    mut entered: ::core::ffi::c_int,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    if !current.is_null() {
        event_payload_set_string(
            ep,
            b"current_mode\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            current,
        );
    }
    if !previous.is_null() {
        event_payload_set_string(
            ep,
            b"previous_mode\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            previous,
        );
    }
    event_payload_set_int(
        ep,
        b"mode_entered\0" as *const u8 as *const ::core::ffi::c_char,
        entered,
    );
    events_fire(name, ep);
}
unsafe extern "C" fn window_fire_pane_prompt(
    mut name: *const ::core::ffi::c_char,
    mut wp: *mut window_pane,
    mut type_0: prompt_type,
) {
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    let mut type_string: *const ::core::ffi::c_char = prompt_type_string(type_0);
    ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    event_payload_set_string(
        ep,
        b"prompt_type\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        type_string,
    );
    events_fire(name, ep);
}
#[no_mangle]
pub unsafe extern "C" fn winlink_cmp(
    mut wl1: *mut winlink,
    mut wl2: *mut winlink,
) -> ::core::ffi::c_int {
    return (*wl1).idx - (*wl2).idx;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_cmp(
    mut wp1: *mut window_pane,
    mut wp2: *mut window_pane,
) -> ::core::ffi::c_int {
    return (*wp1).id.wrapping_sub((*wp2).id) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn winlink_find_by_window(
    mut wwl: *mut winlinks,
    mut w: *mut window,
) -> *mut winlink {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_RB_MINMAX(wwl, RB_NEGINF);
    while !wl.is_null() {
        if (*wl).window == w {
            return wl;
        }
        wl = winlinks_RB_NEXT(wl);
    }
    return ::core::ptr::null_mut::<winlink>();
}
#[no_mangle]
pub unsafe extern "C" fn winlink_find_by_index(
    mut wwl: *mut winlinks,
    mut idx: ::core::ffi::c_int,
) -> *mut winlink {
    let mut wl: winlink = winlink {
        idx: 0,
        session: ::core::ptr::null_mut::<session>(),
        window: ::core::ptr::null_mut::<window>(),
        flags: 0,
        entry: C2RustUnnamed_19 {
            rbe_left: ::core::ptr::null_mut::<winlink>(),
            rbe_right: ::core::ptr::null_mut::<winlink>(),
            rbe_parent: ::core::ptr::null_mut::<winlink>(),
            rbe_color: 0,
        },
        wentry: C2RustUnnamed_18 {
            tqe_next: ::core::ptr::null_mut::<winlink>(),
            tqe_prev: ::core::ptr::null_mut::<*mut winlink>(),
        },
        sentry: C2RustUnnamed_17 {
            tqe_next: ::core::ptr::null_mut::<winlink>(),
            tqe_prev: ::core::ptr::null_mut::<*mut winlink>(),
        },
    };
    if idx < 0 as ::core::ffi::c_int {
        fatalx(b"bad index\0" as *const u8 as *const ::core::ffi::c_char);
    }
    wl.idx = idx;
    return winlinks_RB_FIND(wwl, &raw mut wl);
}
#[no_mangle]
pub unsafe extern "C" fn winlink_find_by_window_id(
    mut wwl: *mut winlinks,
    mut id: u_int,
) -> *mut winlink {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    wl = winlinks_RB_MINMAX(wwl, RB_NEGINF);
    while !wl.is_null() {
        if (*(*wl).window).id == id {
            return wl;
        }
        wl = winlinks_RB_NEXT(wl);
    }
    return ::core::ptr::null_mut::<winlink>();
}
unsafe extern "C" fn winlink_next_index(
    mut wwl: *mut winlinks,
    mut idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    i = idx;
    loop {
        if winlink_find_by_index(wwl, i).is_null() {
            return i;
        }
        if i == INT_MAX {
            i = 0 as ::core::ffi::c_int;
        } else {
            i += 1;
        }
        if !(i != idx) {
            break;
        }
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn winlink_count(mut wwl: *mut winlinks) -> u_int {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut n: u_int = 0;
    n = 0 as u_int;
    wl = winlinks_RB_MINMAX(wwl, RB_NEGINF);
    while !wl.is_null() {
        n = n.wrapping_add(1);
        wl = winlinks_RB_NEXT(wl);
    }
    return n;
}
#[no_mangle]
pub unsafe extern "C" fn winlink_add(
    mut wwl: *mut winlinks,
    mut idx: ::core::ffi::c_int,
) -> *mut winlink {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if idx < 0 as ::core::ffi::c_int {
        idx = winlink_next_index(wwl, -idx - 1 as ::core::ffi::c_int);
        if idx == -(1 as ::core::ffi::c_int) {
            return ::core::ptr::null_mut::<winlink>();
        }
    } else if !winlink_find_by_index(wwl, idx).is_null() {
        return ::core::ptr::null_mut::<winlink>();
    }
    wl = xcalloc(1 as size_t, ::core::mem::size_of::<winlink>() as size_t) as *mut winlink;
    (*wl).idx = idx;
    winlinks_RB_INSERT(wwl, wl);
    return wl;
}
#[no_mangle]
pub unsafe extern "C" fn winlink_set_window(mut wl: *mut winlink, mut w: *mut window) {
    if !(*wl).window.is_null() {
        if !(*wl).wentry.tqe_next.is_null() {
            (*(*wl).wentry.tqe_next).wentry.tqe_prev = (*wl).wentry.tqe_prev;
        } else {
            (*(*wl).window).winlinks.tqh_last = (*wl).wentry.tqe_prev;
        }
        *(*wl).wentry.tqe_prev = (*wl).wentry.tqe_next;
        window_remove_ref(
            (*wl).window,
            b"winlink_set_window\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    (*wl).wentry.tqe_next = ::core::ptr::null_mut::<winlink>();
    (*wl).wentry.tqe_prev = (*w).winlinks.tqh_last;
    *(*w).winlinks.tqh_last = wl;
    (*w).winlinks.tqh_last = &raw mut (*wl).wentry.tqe_next;
    (*wl).window = w;
    window_add_ref(
        w,
        b"winlink_set_window\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn winlink_remove(mut wwl: *mut winlinks, mut wl: *mut winlink) {
    let mut w: *mut window = (*wl).window;
    if !w.is_null() {
        if !(*wl).wentry.tqe_next.is_null() {
            (*(*wl).wentry.tqe_next).wentry.tqe_prev = (*wl).wentry.tqe_prev;
        } else {
            (*w).winlinks.tqh_last = (*wl).wentry.tqe_prev;
        }
        *(*wl).wentry.tqe_prev = (*wl).wentry.tqe_next;
        window_remove_ref(
            w,
            b"winlink_remove\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    winlinks_RB_REMOVE(wwl, wl);
    free(wl as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn winlink_next(mut wl: *mut winlink) -> *mut winlink {
    return winlinks_RB_NEXT(wl);
}
#[no_mangle]
pub unsafe extern "C" fn winlink_previous(mut wl: *mut winlink) -> *mut winlink {
    return winlinks_RB_PREV(wl);
}
#[no_mangle]
pub unsafe extern "C" fn winlink_next_by_number(
    mut wl: *mut winlink,
    mut s: *mut session,
    mut n: ::core::ffi::c_int,
) -> *mut winlink {
    while n > 0 as ::core::ffi::c_int {
        wl = winlinks_RB_NEXT(wl);
        if wl.is_null() {
            wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_NEGINF);
        }
        n -= 1;
    }
    return wl;
}
#[no_mangle]
pub unsafe extern "C" fn winlink_previous_by_number(
    mut wl: *mut winlink,
    mut s: *mut session,
    mut n: ::core::ffi::c_int,
) -> *mut winlink {
    while n > 0 as ::core::ffi::c_int {
        wl = winlinks_RB_PREV(wl);
        if wl.is_null() {
            wl = winlinks_RB_MINMAX(&raw mut (*s).windows, RB_INF);
        }
        n -= 1;
    }
    return wl;
}
#[no_mangle]
pub unsafe extern "C" fn winlink_stack_push(mut stack: *mut winlink_stack, mut wl: *mut winlink) {
    if wl.is_null() {
        return;
    }
    winlink_stack_remove(stack, wl);
    (*wl).sentry.tqe_next = (*stack).tqh_first;
    if !(*wl).sentry.tqe_next.is_null() {
        (*(*stack).tqh_first).sentry.tqe_prev = &raw mut (*wl).sentry.tqe_next;
    } else {
        (*stack).tqh_last = &raw mut (*wl).sentry.tqe_next;
    }
    (*stack).tqh_first = wl;
    (*wl).sentry.tqe_prev = &raw mut (*stack).tqh_first;
    (*wl).flags |= WINLINK_VISITED;
}
#[no_mangle]
pub unsafe extern "C" fn winlink_stack_remove(mut stack: *mut winlink_stack, mut wl: *mut winlink) {
    if !wl.is_null() && (*wl).flags & WINLINK_VISITED != 0 {
        if !(*wl).sentry.tqe_next.is_null() {
            (*(*wl).sentry.tqe_next).sentry.tqe_prev = (*wl).sentry.tqe_prev;
        } else {
            (*stack).tqh_last = (*wl).sentry.tqe_prev;
        }
        *(*wl).sentry.tqe_prev = (*wl).sentry.tqe_next;
        (*wl).flags &= !WINLINK_VISITED;
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_find_by_id_str(mut s: *const ::core::ffi::c_char) -> *mut window {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: u_int = 0;
    if *s as ::core::ffi::c_int != '@' as i32 {
        return ::core::ptr::null_mut::<window>();
    }
    id = strtonum(
        s.offset(1 as ::core::ffi::c_int as isize),
        0 as ::core::ffi::c_longlong,
        UINT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as u_int;
    if !errstr.is_null() {
        return ::core::ptr::null_mut::<window>();
    }
    return window_find_by_id(id);
}
#[no_mangle]
pub unsafe extern "C" fn window_find_by_id(mut id: u_int) -> *mut window {
    let mut w: window = window {
        id: 0,
        latest: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        name_event: event {
            ev_evcallback: event_callback {
                evcb_active_next: C2RustUnnamed_10 {
                    tqe_next: ::core::ptr::null_mut::<event_callback>(),
                    tqe_prev: ::core::ptr::null_mut::<*mut event_callback>(),
                },
                evcb_flags: 0,
                evcb_pri: 0,
                evcb_closure: 0,
                evcb_cb_union: C2RustUnnamed_9 {
                    evcb_callback: None,
                },
                evcb_arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
            ev_timeout_pos: C2RustUnnamed_7 {
                ev_next_with_common_timeout: C2RustUnnamed_8 {
                    tqe_next: ::core::ptr::null_mut::<event>(),
                    tqe_prev: ::core::ptr::null_mut::<*mut event>(),
                },
            },
            ev_fd: 0,
            ev_base: ::core::ptr::null_mut::<event_base>(),
            ev_: C2RustUnnamed_2 {
                ev_io: C2RustUnnamed_5 {
                    ev_io_next: C2RustUnnamed_6 {
                        le_next: ::core::ptr::null_mut::<event>(),
                        le_prev: ::core::ptr::null_mut::<*mut event>(),
                    },
                    ev_timeout: timeval {
                        tv_sec: 0,
                        tv_usec: 0,
                    },
                },
            },
            ev_events: 0,
            ev_res: 0,
            ev_timeout: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
        },
        name_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        alerts_timer: event {
            ev_evcallback: event_callback {
                evcb_active_next: C2RustUnnamed_10 {
                    tqe_next: ::core::ptr::null_mut::<event_callback>(),
                    tqe_prev: ::core::ptr::null_mut::<*mut event_callback>(),
                },
                evcb_flags: 0,
                evcb_pri: 0,
                evcb_closure: 0,
                evcb_cb_union: C2RustUnnamed_9 {
                    evcb_callback: None,
                },
                evcb_arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
            ev_timeout_pos: C2RustUnnamed_7 {
                ev_next_with_common_timeout: C2RustUnnamed_8 {
                    tqe_next: ::core::ptr::null_mut::<event>(),
                    tqe_prev: ::core::ptr::null_mut::<*mut event>(),
                },
            },
            ev_fd: 0,
            ev_base: ::core::ptr::null_mut::<event_base>(),
            ev_: C2RustUnnamed_2 {
                ev_io: C2RustUnnamed_5 {
                    ev_io_next: C2RustUnnamed_6 {
                        le_next: ::core::ptr::null_mut::<event>(),
                        le_prev: ::core::ptr::null_mut::<*mut event>(),
                    },
                    ev_timeout: timeval {
                        tv_sec: 0,
                        tv_usec: 0,
                    },
                },
            },
            ev_events: 0,
            ev_res: 0,
            ev_timeout: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
        },
        offset_timer: event {
            ev_evcallback: event_callback {
                evcb_active_next: C2RustUnnamed_10 {
                    tqe_next: ::core::ptr::null_mut::<event_callback>(),
                    tqe_prev: ::core::ptr::null_mut::<*mut event_callback>(),
                },
                evcb_flags: 0,
                evcb_pri: 0,
                evcb_closure: 0,
                evcb_cb_union: C2RustUnnamed_9 {
                    evcb_callback: None,
                },
                evcb_arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
            ev_timeout_pos: C2RustUnnamed_7 {
                ev_next_with_common_timeout: C2RustUnnamed_8 {
                    tqe_next: ::core::ptr::null_mut::<event>(),
                    tqe_prev: ::core::ptr::null_mut::<*mut event>(),
                },
            },
            ev_fd: 0,
            ev_base: ::core::ptr::null_mut::<event_base>(),
            ev_: C2RustUnnamed_2 {
                ev_io: C2RustUnnamed_5 {
                    ev_io_next: C2RustUnnamed_6 {
                        le_next: ::core::ptr::null_mut::<event>(),
                        le_prev: ::core::ptr::null_mut::<*mut event>(),
                    },
                    ev_timeout: timeval {
                        tv_sec: 0,
                        tv_usec: 0,
                    },
                },
            },
            ev_events: 0,
            ev_res: 0,
            ev_timeout: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
        },
        activity_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        creation_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        active: ::core::ptr::null_mut::<window_pane>(),
        modal: ::core::ptr::null_mut::<window_pane>(),
        modal_last: ::core::ptr::null_mut::<window_pane>(),
        was_zoomed: ::core::ptr::null_mut::<window_pane>(),
        last_panes: window_panes {
            tqh_first: ::core::ptr::null_mut::<window_pane>(),
            tqh_last: ::core::ptr::null_mut::<*mut window_pane>(),
        },
        z_index: window_panes {
            tqh_first: ::core::ptr::null_mut::<window_pane>(),
            tqh_last: ::core::ptr::null_mut::<*mut window_pane>(),
        },
        panes: window_panes {
            tqh_first: ::core::ptr::null_mut::<window_pane>(),
            tqh_last: ::core::ptr::null_mut::<*mut window_pane>(),
        },
        lastlayout: 0,
        layout_root: ::core::ptr::null_mut::<layout_cell>(),
        saved_layout_root: ::core::ptr::null_mut::<layout_cell>(),
        old_layout: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        sx: 0,
        sy: 0,
        manual_sx: 0,
        manual_sy: 0,
        xpixel: 0,
        ypixel: 0,
        new_sx: 0,
        new_sy: 0,
        new_xpixel: 0,
        new_ypixel: 0,
        redraw_scene_generation: 0,
        menu: ::core::ptr::null_mut::<menu_data>(),
        menu_last_px: 0,
        menu_last_py: 0,
        last_new_pane_x: 0,
        last_new_pane_y: 0,
        sb: 0,
        sb_pos: 0,
        inside_cell: grid_cell {
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
        outside_cell: grid_cell {
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
        flags: 0,
        alerts_queued: 0,
        alerts_entry: C2RustUnnamed_22 {
            tqe_next: ::core::ptr::null_mut::<window>(),
            tqe_prev: ::core::ptr::null_mut::<*mut window>(),
        },
        options: ::core::ptr::null_mut::<options>(),
        references: 0,
        winlinks: C2RustUnnamed_21 {
            tqh_first: ::core::ptr::null_mut::<winlink>(),
            tqh_last: ::core::ptr::null_mut::<*mut winlink>(),
        },
        entry: C2RustUnnamed_20 {
            rbe_left: ::core::ptr::null_mut::<window>(),
            rbe_right: ::core::ptr::null_mut::<window>(),
            rbe_parent: ::core::ptr::null_mut::<window>(),
            rbe_color: 0,
        },
    };
    w.id = id;
    return windows_RB_FIND(&raw mut windows, &raw mut w);
}
#[no_mangle]
pub unsafe extern "C" fn window_update_activity(mut w: *mut window) {
    gettimeofday(&raw mut (*w).activity_time, NULL);
    alerts_queue(w, WINDOW_ACTIVITY);
}
#[no_mangle]
pub unsafe extern "C" fn window_create(
    mut sx: u_int,
    mut sy: u_int,
    mut xpixel: u_int,
    mut ypixel: u_int,
) -> *mut window {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    if xpixel == 0 as u_int {
        xpixel = DEFAULT_XPIXEL as u_int;
    }
    if ypixel == 0 as u_int {
        ypixel = DEFAULT_YPIXEL as u_int;
    }
    w = xcalloc(1 as size_t, ::core::mem::size_of::<window>() as size_t) as *mut window;
    (*w).name = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
    (*w).flags = 0 as ::core::ffi::c_int;
    (*w).panes.tqh_first = ::core::ptr::null_mut::<window_pane>();
    (*w).panes.tqh_last = &raw mut (*w).panes.tqh_first;
    (*w).z_index.tqh_first = ::core::ptr::null_mut::<window_pane>();
    (*w).z_index.tqh_last = &raw mut (*w).z_index.tqh_first;
    (*w).last_panes.tqh_first = ::core::ptr::null_mut::<window_pane>();
    (*w).last_panes.tqh_last = &raw mut (*w).last_panes.tqh_first;
    (*w).active = ::core::ptr::null_mut::<window_pane>();
    (*w).lastlayout = -(1 as ::core::ffi::c_int);
    (*w).layout_root = ::core::ptr::null_mut::<layout_cell>();
    (*w).sx = sx;
    (*w).sy = sy;
    (*w).manual_sx = sx;
    (*w).manual_sy = sy;
    (*w).xpixel = xpixel;
    (*w).ypixel = ypixel;
    (*w).options = options_create(global_w_options);
    (*w).sb = options_get_number(
        (*w).options,
        b"pane-scrollbars\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    (*w).sb_pos = options_get_number(
        (*w).options,
        b"pane-scrollbars-position\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    (*w).references = 0 as u_int;
    (*w).winlinks.tqh_first = ::core::ptr::null_mut::<winlink>();
    (*w).winlinks.tqh_last = &raw mut (*w).winlinks.tqh_first;
    let fresh0 = next_window_id;
    next_window_id = next_window_id.wrapping_add(1);
    (*w).id = fresh0;
    windows_RB_INSERT(&raw mut windows, w);
    if gettimeofday(&raw mut (*w).creation_time, NULL) != 0 as ::core::ffi::c_int {
        fatal(b"gettimeofday failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    window_update_activity(w);
    log_debug(
        b"%s: @%u create %ux%u (%ux%u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_create\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        sx,
        sy,
        (*w).xpixel,
        (*w).ypixel,
    );
    return w;
}
unsafe extern "C" fn window_destroy(mut w: *mut window) {
    log_debug(
        b"window @%u destroyed (%d references)\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        (*w).references,
    );
    window_unzoom(w, 0 as ::core::ffi::c_int);
    windows_RB_REMOVE(&raw mut windows, w);
    layout_free_cell((*w).layout_root, 0 as ::core::ffi::c_int);
    layout_free_cell((*w).saved_layout_root, 0 as ::core::ffi::c_int);
    free((*w).old_layout as *mut ::core::ffi::c_void);
    menu_destroy(w);
    window_destroy_panes(w);
    if event_initialized(&raw mut (*w).name_event) != 0 {
        event_del(&raw mut (*w).name_event);
    }
    if event_initialized(&raw mut (*w).alerts_timer) != 0 {
        event_del(&raw mut (*w).alerts_timer);
    }
    if event_initialized(&raw mut (*w).offset_timer) != 0 {
        event_del(&raw mut (*w).offset_timer);
    }
    options_free((*w).options);
    free((*w).name as *mut ::core::ffi::c_void);
    free(w as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_destroy_ready(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_int = 0;
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int)
        && evbuffer_get_length((*(*wp).pipe_event).output) != 0 as size_t
    {
        return 0 as ::core::ffi::c_int;
    }
    if ioctl((*wp).fd, FIONREAD as ::core::ffi::c_ulong, &raw mut n) != -(1 as ::core::ffi::c_int)
        && n > 0 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if !(*wp).flags & PANE_EXITED != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if !(*wp).wait_item.is_null() && !(*wp).flags & PANE_STATUSREADY != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if !(*wp).editor.is_null() && !(*wp).flags & PANE_STATUSREADY != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_add_ref(mut w: *mut window, mut from: *const ::core::ffi::c_char) {
    (*w).references = (*w).references.wrapping_add(1);
    log_debug(
        b"%s: @%u %s, now %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_add_ref\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        from,
        (*w).references,
    );
}
#[no_mangle]
pub unsafe extern "C" fn window_remove_ref(
    mut w: *mut window,
    mut from: *const ::core::ffi::c_char,
) {
    if (*w).references == 1 as u_int {
        events_fire_window(
            b"window-closed\0" as *const u8 as *const ::core::ffi::c_char,
            w,
        );
    }
    (*w).references = (*w).references.wrapping_sub(1);
    log_debug(
        b"%s: @%u %s, now %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_remove_ref\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        from,
        (*w).references,
    );
    if (*w).references == 0 as u_int {
        window_destroy(w);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_add_ref(
    mut wp: *mut window_pane,
    mut from: *const ::core::ffi::c_char,
) {
    (*wp).references += 1;
    log_debug(
        b"%s: %%%u %s, now %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_pane_add_ref\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        from,
        (*wp).references,
    );
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_remove_ref(
    mut wp: *mut window_pane,
    mut from: *const ::core::ffi::c_char,
) {
    (*wp).references -= 1;
    log_debug(
        b"%s: %%%u %s, now %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_pane_remove_ref\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        from,
        (*wp).references,
    );
    if (*wp).references == 0 as ::core::ffi::c_int {
        window_pane_free(wp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_set_name(
    mut w: *mut window,
    mut new_name: *const ::core::ffi::c_char,
    mut untrusted: ::core::ffi::c_int,
) {
    let mut last: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    name = clean_name(new_name, untrusted);
    if !name.is_null() {
        last = xstrdup((*w).name);
        free((*w).name as *mut ::core::ffi::c_void);
        (*w).name = name;
        window_fire_renamed(w, last);
        free(last as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_resize(
    mut w: *mut window,
    mut sx: u_int,
    mut sy: u_int,
    mut xpixel: ::core::ffi::c_int,
    mut ypixel: ::core::ffi::c_int,
) {
    if xpixel == 0 as ::core::ffi::c_int {
        xpixel = DEFAULT_XPIXEL;
    }
    if ypixel == 0 as ::core::ffi::c_int {
        ypixel = DEFAULT_YPIXEL;
    }
    log_debug(
        b"%s: @%u resize %ux%u (%ux%u)\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_resize\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        sx,
        sy,
        if xpixel == -(1 as ::core::ffi::c_int) {
            (*w).xpixel
        } else {
            xpixel as u_int
        },
        if ypixel == -(1 as ::core::ffi::c_int) {
            (*w).ypixel
        } else {
            ypixel as u_int
        },
    );
    (*w).sx = sx;
    (*w).sy = sy;
    if !(*w).menu.is_null() {
        menu_resize((*w).menu, w);
        server_redraw_window(w);
    }
    if xpixel != -(1 as ::core::ffi::c_int) {
        (*w).xpixel = xpixel as u_int;
    }
    if ypixel != -(1 as ::core::ffi::c_int) {
        (*w).ypixel = ypixel as u_int;
    }
    redraw_invalidate_scene(w);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_send_resize(
    mut wp: *mut window_pane,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut ws: winsize = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    if (*wp).fd == -(1 as ::core::ffi::c_int) {
        return;
    }
    log_debug(
        b"%s: %%%u resize to %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_pane_send_resize\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
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
    ws.ws_xpixel = (*w).xpixel.wrapping_mul(ws.ws_col as u_int) as ::core::ffi::c_ushort;
    ws.ws_ypixel = (*w).ypixel.wrapping_mul(ws.ws_row as u_int) as ::core::ffi::c_ushort;
    if ioctl((*wp).fd, TIOCSWINSZ as ::core::ffi::c_ulong, &raw mut ws)
        == -(1 as ::core::ffi::c_int)
    {
        fatal(b"ioctl failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_has_floating_panes(mut w: *mut window) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if window_pane_is_floating(wp) != 0 {
            return 1 as ::core::ffi::c_int;
        }
        wp = (*wp).entry.tqe_next;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_has_pane(
    mut w: *mut window,
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    let mut wp1: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    wp1 = (*w).panes.tqh_first;
    while !wp1.is_null() {
        if wp1 == wp {
            return 1 as ::core::ffi::c_int;
        }
        wp1 = (*wp1).entry.tqe_next;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_contains(
    mut wp: *mut window_pane,
    mut x: u_int,
    mut y: u_int,
) -> ::core::ffi::c_int {
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if window_pane_is_visible(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    window_pane_full_size_offset(wp, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
    if window_pane_is_floating(wp) == 0 {
        if (x as ::core::ffi::c_int) < xoff || x > (xoff as u_int).wrapping_add(sx) {
            return 0 as ::core::ffi::c_int;
        }
        if (y as ::core::ffi::c_int) < yoff || y > (yoff as u_int).wrapping_add(sy) {
            return 0 as ::core::ffi::c_int;
        }
    } else if window_pane_get_pane_lines(wp) as ::core::ffi::c_uint
        == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (x as ::core::ffi::c_int) < xoff
            || x as ::core::ffi::c_int >= xoff + sx as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
        if (y as ::core::ffi::c_int) < yoff
            || y as ::core::ffi::c_int >= yoff + sy as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
    } else {
        if (x as ::core::ffi::c_int) < xoff - 1 as ::core::ffi::c_int
            || x > (xoff as u_int).wrapping_add(sx)
        {
            return 0 as ::core::ffi::c_int;
        }
        if (y as ::core::ffi::c_int) < yoff - 1 as ::core::ffi::c_int
            || y > (yoff as u_int).wrapping_add(sy)
        {
            return 0 as ::core::ffi::c_int;
        }
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_update_focus(mut w: *mut window) {
    if !w.is_null() {
        log_debug(
            b"%s: @%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"window_update_focus\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
        );
        window_pane_update_focus((*w).active);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_update_focus(mut wp: *mut window_pane) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut focused: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !wp.is_null() && !(*wp).flags & PANE_EXITED != 0 {
        if wp != (*(*wp).window).active {
            focused = 0 as ::core::ffi::c_int;
        } else {
            c = clients.tqh_first;
            while !c.is_null() {
                if !(*c).session.is_null()
                    && (*(*c).session).attached != 0 as u_int
                    && (*c).flags & CLIENT_FOCUSED as uint64_t != 0
                    && (*(*(*c).session).curw).window == (*wp).window
                    && (*c).overlay_draw.is_none()
                    && (*(*wp).window).menu.is_null()
                {
                    focused = 1 as ::core::ffi::c_int;
                    break;
                } else {
                    c = (*c).entry.tqe_next;
                }
            }
        }
        if focused == 0 && (*wp).flags & PANE_FOCUSED != 0 {
            log_debug(
                b"%s: %%%u focus out\0" as *const u8 as *const ::core::ffi::c_char,
                b"window_pane_update_focus\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            );
            if (*wp).base.mode & MODE_FOCUSON != 0 {
                bufferevent_write(
                    (*wp).event,
                    b"\x1B[O\0" as *const u8 as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    3 as size_t,
                );
            }
            events_fire_pane(
                b"pane-focus-out\0" as *const u8 as *const ::core::ffi::c_char,
                wp,
            );
            (*wp).flags &= !PANE_FOCUSED;
        } else if focused != 0 && !(*wp).flags & PANE_FOCUSED != 0 {
            log_debug(
                b"%s: %%%u focus in\0" as *const u8 as *const ::core::ffi::c_char,
                b"window_pane_update_focus\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            );
            if (*wp).base.mode & MODE_FOCUSON != 0 {
                bufferevent_write(
                    (*wp).event,
                    b"\x1B[I\0" as *const u8 as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    3 as size_t,
                );
            }
            events_fire_pane(
                b"pane-focus-in\0" as *const u8 as *const ::core::ffi::c_char,
                wp,
            );
            (*wp).flags |= PANE_FOCUSED;
        } else {
            log_debug(
                b"%s: %%%u focus unchanged\0" as *const u8 as *const ::core::ffi::c_char,
                b"window_pane_update_focus\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_set_active_pane(
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut notify: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut lastwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    log_debug(
        b"%s: pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_set_active_pane\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
    if wp == (*w).active {
        return 0 as ::core::ffi::c_int;
    }
    if !(*w).modal.is_null() && wp != (*w).modal {
        return 0 as ::core::ffi::c_int;
    }
    if (*w).flags & WINDOW_ZOOMED != 0 && window_pane_is_visible(wp) == 0 {
        window_unzoom(w, 1 as ::core::ffi::c_int);
    }
    lastwp = (*w).active;
    window_pane_stack_remove(&raw mut (*w).last_panes, wp);
    window_pane_stack_push(&raw mut (*w).last_panes, lastwp);
    (*w).active = wp;
    let fresh1 = next_active_point;
    next_active_point = next_active_point.wrapping_add(1);
    (*(*w).active).active_point = fresh1;
    (*(*w).active).flags |= PANE_CHANGED;
    if options_get_number(
        global_options,
        b"focus-events\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        window_pane_update_focus(lastwp);
        window_pane_update_focus((*w).active);
    }
    tty_update_window_offset(w);
    server_redraw_window(w);
    if notify != 0 {
        window_fire_pane_changed(w, (*w).active, lastwp);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_pane_get_palette(
    mut wp: *mut window_pane,
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if wp.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    return colour_palette_get(&raw mut (*wp).palette, c);
}
#[no_mangle]
pub unsafe extern "C" fn window_redraw_active_switch(mut w: *mut window, mut wp: *mut window_pane) {
    let mut gc1: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
    let mut gc2: *mut grid_cell = ::core::ptr::null_mut::<grid_cell>();
    let mut c1: ::core::ffi::c_int = 0;
    let mut c2: ::core::ffi::c_int = 0;
    if !(*w).modal.is_null() && wp != (*w).modal {
        return;
    }
    if wp == (*w).active {
        return;
    }
    loop {
        gc1 = &raw mut (*wp).cached_gc;
        gc2 = &raw mut (*wp).cached_active_gc;
        if grid_cells_look_equal(gc1, gc2) == 0 {
            (*wp).flags |= PANE_REDRAW;
        } else if (*wp).cached_dim != (*wp).cached_active_dim {
            (*wp).flags |= PANE_REDRAW;
        } else {
            c1 = window_pane_get_palette(wp, (*gc1).fg);
            c2 = window_pane_get_palette(wp, (*gc2).fg);
            if c1 != c2 {
                (*wp).flags |= PANE_REDRAW;
            } else {
                c1 = window_pane_get_palette(wp, (*gc1).bg);
                c2 = window_pane_get_palette(wp, (*gc2).bg);
                if c1 != c2 {
                    (*wp).flags |= PANE_REDRAW;
                }
            }
        }
        if wp == (*w).active {
            break;
        }
        if window_pane_is_floating(wp) != 0 {
            if !(*wp).zentry.tqe_next.is_null() {
                (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
            } else {
                (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
            }
            *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
            (*wp).zentry.tqe_next = (*w).z_index.tqh_first;
            if !(*wp).zentry.tqe_next.is_null() {
                (*(*w).z_index.tqh_first).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
            } else {
                (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
            }
            (*w).z_index.tqh_first = wp;
            (*wp).zentry.tqe_prev = &raw mut (*w).z_index.tqh_first;
            (*wp).flags |= PANE_REDRAW;
            redraw_invalidate_scene(w);
        }
        wp = (*w).active;
        if wp.is_null() {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_get_active_at(
    mut w: *mut window,
    mut x: u_int,
    mut y: u_int,
) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut pane_status: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    pane_status = window_get_pane_status(w);
    if !(*w).modal.is_null() {
        if window_pane_contains((*w).modal, x, y) != 0 {
            return (*w).modal;
        }
        return ::core::ptr::null_mut::<window_pane>();
    }
    if pane_status == PANE_STATUS_TOP {
        wp = (*w).z_index.tqh_first;
        while !wp.is_null() {
            if !(window_pane_is_visible(wp) == 0 || window_pane_is_floating(wp) != 0) {
                window_pane_full_size_offset(
                    wp,
                    &raw mut xoff,
                    &raw mut yoff,
                    &raw mut sx,
                    &raw mut sy,
                );
                if !((x as ::core::ffi::c_int) < xoff || x > (xoff as u_int).wrapping_add(sx)) {
                    if y as ::core::ffi::c_int == yoff - 1 as ::core::ffi::c_int {
                        return wp;
                    }
                }
            }
            wp = (*wp).zentry.tqe_next;
        }
    }
    let mut current_block_15: u64;
    wp = (*w).z_index.tqh_first;
    while !wp.is_null() {
        if !(window_pane_is_visible(wp) == 0) {
            window_pane_full_size_offset(
                wp,
                &raw mut xoff,
                &raw mut yoff,
                &raw mut sx,
                &raw mut sy,
            );
            if window_pane_is_floating(wp) == 0 {
                if (x as ::core::ffi::c_int) < xoff || x > (xoff as u_int).wrapping_add(sx) {
                    current_block_15 = 12349973810996921269;
                } else if pane_status == PANE_STATUS_TOP {
                    if (y as ::core::ffi::c_int) < yoff - 1 as ::core::ffi::c_int
                        || y > (yoff as u_int).wrapping_add(sy)
                    {
                        current_block_15 = 12349973810996921269;
                    } else {
                        current_block_15 = 8693738493027456495;
                    }
                } else if (y as ::core::ffi::c_int) < yoff || y > (yoff as u_int).wrapping_add(sy) {
                    current_block_15 = 12349973810996921269;
                } else {
                    current_block_15 = 8693738493027456495;
                }
            } else if window_pane_get_pane_lines(wp) as ::core::ffi::c_uint
                == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (x as ::core::ffi::c_int) < xoff
                    || x as ::core::ffi::c_int >= xoff + sx as ::core::ffi::c_int
                {
                    current_block_15 = 12349973810996921269;
                } else if (y as ::core::ffi::c_int) < yoff
                    || y as ::core::ffi::c_int >= yoff + sy as ::core::ffi::c_int
                {
                    current_block_15 = 12349973810996921269;
                } else {
                    current_block_15 = 8693738493027456495;
                }
            } else if (x as ::core::ffi::c_int) < xoff - 1 as ::core::ffi::c_int
                || x > (xoff as u_int).wrapping_add(sx)
            {
                current_block_15 = 12349973810996921269;
            } else if (y as ::core::ffi::c_int) < yoff - 1 as ::core::ffi::c_int
                || y > (yoff as u_int).wrapping_add(sy)
            {
                current_block_15 = 12349973810996921269;
            } else {
                current_block_15 = 8693738493027456495;
            }
            match current_block_15 {
                12349973810996921269 => {}
                _ => return wp,
            }
        }
        wp = (*wp).zentry.tqe_next;
    }
    return ::core::ptr::null_mut::<window_pane>();
}
#[no_mangle]
pub unsafe extern "C" fn window_find_string(
    mut w: *mut window,
    mut s: *const ::core::ffi::c_char,
) -> *mut window_pane {
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut top: u_int = 0 as u_int;
    let mut bottom: u_int = (*w).sy.wrapping_sub(1 as u_int);
    let mut status: ::core::ffi::c_int = 0;
    x = (*w).sx.wrapping_div(2 as u_int);
    y = (*w).sy.wrapping_div(2 as u_int);
    status = window_get_pane_status(w);
    if status == PANE_STATUS_TOP {
        top = top.wrapping_add(1);
    } else if status == PANE_STATUS_BOTTOM {
        bottom = bottom.wrapping_sub(1);
    }
    if strcasecmp(s, b"top\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        y = top;
    } else if strcasecmp(s, b"bottom\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        y = bottom;
    } else if strcasecmp(s, b"left\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        x = 0 as u_int;
    } else if strcasecmp(s, b"right\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        x = (*w).sx.wrapping_sub(1 as u_int);
    } else if strcasecmp(s, b"top-left\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        x = 0 as u_int;
        y = top;
    } else if strcasecmp(s, b"top-right\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        x = (*w).sx.wrapping_sub(1 as u_int);
        y = top;
    } else if strcasecmp(
        s,
        b"bottom-left\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = 0 as u_int;
        y = bottom;
    } else if strcasecmp(
        s,
        b"bottom-right\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        x = (*w).sx.wrapping_sub(1 as u_int);
        y = bottom;
    } else {
        return ::core::ptr::null_mut::<window_pane>();
    }
    return window_get_active_at(w, x, y);
}
#[no_mangle]
pub unsafe extern "C" fn window_zoom(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut wp1: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    let mut lg: layout_geometry = layout_geometry {
        sx: 0,
        sy: 0,
        xoff: 0,
        yoff: 0,
    };
    if (*w).flags & WINDOW_ZOOMED != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if window_count_panes(w, 1 as ::core::ffi::c_int) == 1 as u_int {
        return -(1 as ::core::ffi::c_int);
    }
    if (*w).active != wp
        && ((*w).active.is_null()
            || !(*(*w).active).flags & PANE_FLOATOVERZOOM != 0
            || window_pane_is_floating((*w).active) == 0)
    {
        window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
    }
    (*wp).flags |= PANE_ZOOMED;
    wp1 = (*w).panes.tqh_first;
    while !wp1.is_null() {
        (*wp1).saved_layout_cell = (*wp1).layout_cell as *mut layout_cell;
        (*wp1).layout_cell = ::core::ptr::null_mut::<layout_cell>();
        wp1 = (*wp1).entry.tqe_next;
    }
    (*w).saved_layout_root = (*w).layout_root;
    layout_init(w, wp);
    wp1 = (*w).panes.tqh_first;
    while !wp1.is_null() {
        lc = (*wp1).saved_layout_cell;
        if !(wp1 == wp
            || !(*wp1).flags & PANE_FLOATOVERZOOM != 0
            || lc.is_null()
            || !(*lc).flags & LAYOUT_CELL_FLOATING != 0)
        {
            memcpy(
                &raw mut lg as *mut ::core::ffi::c_void,
                &raw mut (*lc).g as *const ::core::ffi::c_void,
                ::core::mem::size_of::<layout_geometry>() as size_t,
            );
            lc = layout_floating_pane(w, wp, &raw mut lg);
            layout_assign_pane(lc, wp1, 0 as ::core::ffi::c_int);
        }
        wp1 = (*wp1).entry.tqe_next;
    }
    if (*(*wp).saved_layout_cell).flags & LAYOUT_CELL_FLOATING != 0 {
        if !(*wp).zentry.tqe_next.is_null() {
            (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
        } else {
            (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
        }
        *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
        (*wp).zentry.tqe_next = ::core::ptr::null_mut::<window_pane>();
        (*wp).zentry.tqe_prev = (*w).z_index.tqh_last;
        *(*w).z_index.tqh_last = wp;
        (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
    }
    (*w).flags |= WINDOW_ZOOMED;
    events_fire_window(
        b"window-zoomed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    events_fire_window(
        b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
        w,
    );
    redraw_invalidate_scene(w);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_unzoom(
    mut w: *mut window,
    mut notify: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut zoomed: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut slc: *mut layout_cell = ::core::ptr::null_mut::<layout_cell>();
    if !(*w).flags & WINDOW_ZOOMED != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if (*wp).flags & PANE_ZOOMED != 0 {
            zoomed = wp;
        }
        if !(!(*wp).flags & PANE_FLOATOVERZOOM != 0) {
            if !((*wp).flags & PANE_ZOOMED != 0) {
                slc = (*wp).saved_layout_cell;
                if !(slc.is_null() || (*wp).layout_cell.is_null()) {
                    memcpy(
                        &raw mut (*slc).g as *mut ::core::ffi::c_void,
                        &raw mut (*(*wp).layout_cell).g as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<layout_geometry>() as size_t,
                    );
                    memcpy(
                        &raw mut (*slc).fg as *mut ::core::ffi::c_void,
                        &raw mut (*(*wp).layout_cell).fg as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<layout_geometry>() as size_t,
                    );
                }
            }
        }
        wp = (*wp).entry.tqe_next;
    }
    (*w).flags &= !WINDOW_ZOOMED;
    layout_free(w, 0 as ::core::ffi::c_int);
    (*w).layout_root = (*w).saved_layout_root;
    (*w).saved_layout_root = ::core::ptr::null_mut::<layout_cell>();
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        (*wp).layout_cell = (*wp).saved_layout_cell as *mut layout_cell;
        (*wp).saved_layout_cell = ::core::ptr::null_mut::<layout_cell>();
        (*wp).flags &= !PANE_ZOOMED;
        wp = (*wp).entry.tqe_next;
    }
    if !zoomed.is_null() && window_pane_is_floating(zoomed) != 0 {
        if !(*zoomed).zentry.tqe_next.is_null() {
            (*(*zoomed).zentry.tqe_next).zentry.tqe_prev = (*zoomed).zentry.tqe_prev;
        } else {
            (*w).z_index.tqh_last = (*zoomed).zentry.tqe_prev;
        }
        *(*zoomed).zentry.tqe_prev = (*zoomed).zentry.tqe_next;
        if zoomed == (*w).active {
            (*zoomed).zentry.tqe_next = (*w).z_index.tqh_first;
            if !(*zoomed).zentry.tqe_next.is_null() {
                (*(*w).z_index.tqh_first).zentry.tqe_prev = &raw mut (*zoomed).zentry.tqe_next;
            } else {
                (*w).z_index.tqh_last = &raw mut (*zoomed).zentry.tqe_next;
            }
            (*w).z_index.tqh_first = zoomed;
            (*zoomed).zentry.tqe_prev = &raw mut (*w).z_index.tqh_first;
        } else {
            wp = (*w).z_index.tqh_first;
            while !wp.is_null() {
                if window_pane_is_floating(wp) == 0 {
                    break;
                }
                wp = (*wp).zentry.tqe_next;
            }
            if wp.is_null() {
                (*zoomed).zentry.tqe_next = ::core::ptr::null_mut::<window_pane>();
                (*zoomed).zentry.tqe_prev = (*w).z_index.tqh_last;
                *(*w).z_index.tqh_last = zoomed;
                (*w).z_index.tqh_last = &raw mut (*zoomed).zentry.tqe_next;
            } else {
                (*zoomed).zentry.tqe_prev = (*wp).zentry.tqe_prev;
                (*zoomed).zentry.tqe_next = wp;
                *(*wp).zentry.tqe_prev = zoomed;
                (*wp).zentry.tqe_prev = &raw mut (*zoomed).zentry.tqe_next;
            }
        }
    }
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    if notify != 0 {
        events_fire_window(
            b"window-unzoomed\0" as *const u8 as *const ::core::ffi::c_char,
            w,
        );
        events_fire_window(
            b"window-layout-changed\0" as *const u8 as *const ::core::ffi::c_char,
            w,
        );
    }
    redraw_invalidate_scene(w);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_zoomed_pane(mut w: *mut window) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if !(*w).flags & WINDOW_ZOOMED != 0 {
        return ::core::ptr::null_mut::<window_pane>();
    }
    wp = *(*((*w).z_index.tqh_last as *mut window_panes_zindex)).tqh_last;
    while !wp.is_null() {
        if !(*wp).layout_cell.is_null() && window_pane_is_floating(wp) == 0 {
            return wp;
        }
        wp = *(*((*wp).zentry.tqe_prev as *mut window_panes_zindex)).tqh_last;
    }
    return ::core::ptr::null_mut::<window_pane>();
}
#[no_mangle]
pub unsafe extern "C" fn window_active_pane_is_over_zoom(mut w: *mut window) -> ::core::ffi::c_int {
    if !(*w).flags & WINDOW_ZOOMED != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*w).active.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if !(*(*w).active).flags & PANE_FLOATOVERZOOM != 0 {
        return 0 as ::core::ffi::c_int;
    }
    return window_pane_is_floating((*w).active);
}
#[no_mangle]
pub unsafe extern "C" fn window_push_zoom(
    mut w: *mut window,
    mut always: ::core::ffi::c_int,
    mut flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = window_zoomed_pane(w);
    log_debug(
        b"%s: @%u %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_push_zoom\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        (flag != 0 && (*w).flags & WINDOW_ZOOMED != 0) as ::core::ffi::c_int,
    );
    if flag != 0 && (always != 0 || (*w).flags & WINDOW_ZOOMED != 0) {
        (*w).flags |= WINDOW_WASZOOMED;
    } else {
        (*w).flags &= !WINDOW_WASZOOMED;
    }
    if (*w).flags & WINDOW_WASZOOMED != 0 {
        (*w).was_zoomed = wp;
    } else {
        (*w).was_zoomed = ::core::ptr::null_mut::<window_pane>();
    }
    return (window_unzoom(w, 1 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pop_zoom(mut w: *mut window) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*w).was_zoomed;
    log_debug(
        b"%s: @%u %d\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_pop_zoom\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        ((*w).flags & WINDOW_WASZOOMED != 0) as ::core::ffi::c_int,
    );
    if (*w).flags & WINDOW_WASZOOMED != 0 {
        (*w).flags &= !WINDOW_WASZOOMED;
        (*w).was_zoomed = ::core::ptr::null_mut::<window_pane>();
        if !(*w).active.is_null()
            && (!(*(*w).active).flags & PANE_FLOATOVERZOOM != 0
                || window_pane_is_floating((*w).active) == 0)
        {
            wp = (*w).active;
        }
        if wp.is_null() || window_has_pane(w, wp) == 0 {
            wp = (*w).active;
        }
        if !wp.is_null() {
            return (window_zoom(wp) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_add_pane(
    mut w: *mut window,
    mut other: *mut window_pane,
    mut hlimit: u_int,
    mut flags: ::core::ffi::c_int,
) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if other.is_null() {
        other = (*w).active;
    }
    wp = window_pane_create(w, (*w).sx, (*w).sy, hlimit);
    if (*w).panes.tqh_first.is_null() {
        log_debug(
            b"%s: @%u at start\0" as *const u8 as *const ::core::ffi::c_char,
            b"window_add_pane\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
        );
        (*wp).entry.tqe_next = (*w).panes.tqh_first;
        if !(*wp).entry.tqe_next.is_null() {
            (*(*w).panes.tqh_first).entry.tqe_prev = &raw mut (*wp).entry.tqe_next;
        } else {
            (*w).panes.tqh_last = &raw mut (*wp).entry.tqe_next;
        }
        (*w).panes.tqh_first = wp;
        (*wp).entry.tqe_prev = &raw mut (*w).panes.tqh_first;
    } else if flags & SPAWN_BEFORE != 0 {
        log_debug(
            b"%s: @%u before %%%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"window_add_pane\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
            (*wp).id,
        );
        if flags & SPAWN_FULLSIZE != 0 {
            (*wp).entry.tqe_next = (*w).panes.tqh_first;
            if !(*wp).entry.tqe_next.is_null() {
                (*(*w).panes.tqh_first).entry.tqe_prev = &raw mut (*wp).entry.tqe_next;
            } else {
                (*w).panes.tqh_last = &raw mut (*wp).entry.tqe_next;
            }
            (*w).panes.tqh_first = wp;
            (*wp).entry.tqe_prev = &raw mut (*w).panes.tqh_first;
        } else {
            (*wp).entry.tqe_prev = (*other).entry.tqe_prev;
            (*wp).entry.tqe_next = other;
            *(*other).entry.tqe_prev = wp;
            (*other).entry.tqe_prev = &raw mut (*wp).entry.tqe_next;
        }
    } else {
        log_debug(
            b"%s: @%u after %%%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"window_add_pane\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
            (*wp).id,
        );
        if flags & (SPAWN_FULLSIZE | SPAWN_FLOATING) != 0 {
            (*wp).entry.tqe_next = ::core::ptr::null_mut::<window_pane>();
            (*wp).entry.tqe_prev = (*w).panes.tqh_last;
            *(*w).panes.tqh_last = wp;
            (*w).panes.tqh_last = &raw mut (*wp).entry.tqe_next;
        } else {
            (*wp).entry.tqe_next = (*other).entry.tqe_next;
            if !(*wp).entry.tqe_next.is_null() {
                (*(*wp).entry.tqe_next).entry.tqe_prev = &raw mut (*wp).entry.tqe_next;
            } else {
                (*w).panes.tqh_last = &raw mut (*wp).entry.tqe_next;
            }
            (*other).entry.tqe_next = wp;
            (*wp).entry.tqe_prev = &raw mut (*other).entry.tqe_next;
        }
    }
    if !flags & SPAWN_FLOATING != 0 {
        (*wp).zentry.tqe_next = ::core::ptr::null_mut::<window_pane>();
        (*wp).zentry.tqe_prev = (*w).z_index.tqh_last;
        *(*w).z_index.tqh_last = wp;
        (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
    } else if !(*w).modal.is_null() {
        (*wp).zentry.tqe_next = (*(*w).modal).zentry.tqe_next;
        if !(*wp).zentry.tqe_next.is_null() {
            (*(*wp).zentry.tqe_next).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
        } else {
            (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
        }
        (*(*w).modal).zentry.tqe_next = wp;
        (*wp).zentry.tqe_prev = &raw mut (*(*w).modal).zentry.tqe_next;
    } else {
        (*wp).zentry.tqe_next = (*w).z_index.tqh_first;
        if !(*wp).zentry.tqe_next.is_null() {
            (*(*w).z_index.tqh_first).zentry.tqe_prev = &raw mut (*wp).zentry.tqe_next;
        } else {
            (*w).z_index.tqh_last = &raw mut (*wp).zentry.tqe_next;
        }
        (*w).z_index.tqh_first = wp;
        (*wp).zentry.tqe_prev = &raw mut (*w).z_index.tqh_first;
    }
    redraw_invalidate_scene(w);
    return wp;
}
#[no_mangle]
pub unsafe extern "C" fn window_lost_pane(mut w: *mut window, mut wp: *mut window_pane) {
    let mut lastwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    log_debug(
        b"%s: @%u pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_lost_pane\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
        (*wp).id,
    );
    if wp == marked_pane.wp {
        server_clear_marked();
    }
    if wp == (*w).modal_last {
        (*w).modal_last = ::core::ptr::null_mut::<window_pane>();
    }
    if wp == (*w).was_zoomed {
        (*w).was_zoomed = ::core::ptr::null_mut::<window_pane>();
    }
    window_pane_stack_remove(&raw mut (*w).last_panes, wp);
    if wp == (*w).active {
        lastwp = ::core::ptr::null_mut::<window_pane>();
        if wp == (*w).modal {
            lastwp = (*w).modal_last;
            (*w).modal = ::core::ptr::null_mut::<window_pane>();
            (*w).modal_last = ::core::ptr::null_mut::<window_pane>();
        }
        if !lastwp.is_null() && window_has_pane(w, lastwp) != 0 {
            (*w).active = lastwp;
        } else {
            (*w).active = (*w).last_panes.tqh_first;
        }
        if (*w).active.is_null() {
            (*w).active = *(*((*wp).entry.tqe_prev as *mut window_panes)).tqh_last;
            if (*w).active.is_null() {
                (*w).active = (*wp).entry.tqe_next;
            }
        }
        if !(*w).active.is_null() {
            window_pane_stack_remove(&raw mut (*w).last_panes, (*w).active);
            (*(*w).active).flags |= PANE_CHANGED;
            window_fire_pane_changed(w, (*w).active, wp);
            window_update_focus(w);
        }
    } else if wp == (*w).modal {
        (*w).modal_last = ::core::ptr::null_mut::<window_pane>();
        (*w).modal = (*w).modal_last;
    }
    redraw_invalidate_scene(w);
}
#[no_mangle]
pub unsafe extern "C" fn window_remove_pane(mut w: *mut window, mut wp: *mut window_pane) {
    window_lost_pane(w, wp);
    if !(*wp).entry.tqe_next.is_null() {
        (*(*wp).entry.tqe_next).entry.tqe_prev = (*wp).entry.tqe_prev;
    } else {
        (*w).panes.tqh_last = (*wp).entry.tqe_prev;
    }
    *(*wp).entry.tqe_prev = (*wp).entry.tqe_next;
    if !(*wp).zentry.tqe_next.is_null() {
        (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
    } else {
        (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
    }
    *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
    redraw_invalidate_scene(w);
    window_pane_destroy(wp);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_at_index(
    mut w: *mut window,
    mut idx: u_int,
) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut n: u_int = 0;
    n = options_get_number(
        (*w).options,
        b"pane-base-index\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if n == idx {
            return wp;
        }
        n = n.wrapping_add(1);
        wp = (*wp).entry.tqe_next;
    }
    return ::core::ptr::null_mut::<window_pane>();
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_next_by_number(
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut n: u_int,
) -> *mut window_pane {
    while n > 0 as u_int {
        wp = (*wp).entry.tqe_next;
        if wp.is_null() {
            wp = (*w).panes.tqh_first;
        }
        n = n.wrapping_sub(1);
    }
    return wp;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_previous_by_number(
    mut w: *mut window,
    mut wp: *mut window_pane,
    mut n: u_int,
) -> *mut window_pane {
    while n > 0 as u_int {
        wp = *(*((*wp).entry.tqe_prev as *mut window_panes)).tqh_last;
        if wp.is_null() {
            wp = *(*((*w).panes.tqh_last as *mut window_panes)).tqh_last;
        }
        n = n.wrapping_sub(1);
    }
    return wp;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_index(
    mut wp: *mut window_pane,
    mut i: *mut u_int,
) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut wq: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    *i = options_get_number(
        (*w).options,
        b"pane-base-index\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    wq = (*w).panes.tqh_first;
    while !wq.is_null() {
        if wp == wq {
            return 0 as ::core::ffi::c_int;
        }
        *i = (*i).wrapping_add(1);
        wq = (*wq).entry.tqe_next;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_zindex(
    mut wp: *mut window_pane,
    mut i: *mut u_int,
) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut wq: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    *i = 0 as u_int;
    wq = (*w).z_index.tqh_first;
    while !wq.is_null() {
        if wq == wp {
            if window_pane_is_floating(wp) == 0 {
                *i = (*i).wrapping_add(1);
            }
            return 0 as ::core::ffi::c_int;
        }
        if window_pane_is_floating(wq) != 0 {
            *i = (*i).wrapping_add(1);
        }
        wq = (*wq).zentry.tqe_next;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_last_index(
    mut wp: *mut window_pane,
    mut i: *mut u_int,
) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut wq: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    *i = 0 as u_int;
    wq = (*w).last_panes.tqh_first;
    while !wq.is_null() {
        if wq == wp {
            return 0 as ::core::ffi::c_int;
        }
        *i = (*i).wrapping_add(1);
        wq = (*wq).sentry.tqe_next;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn window_count_panes(
    mut w: *mut window,
    mut with_floating: ::core::ffi::c_int,
) -> u_int {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut n: u_int = 0 as u_int;
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if with_floating != 0 || window_pane_is_floating(wp) == 0 {
            n = n.wrapping_add(1);
        }
        wp = (*wp).entry.tqe_next;
    }
    return n;
}
#[no_mangle]
pub unsafe extern "C" fn window_destroy_panes(mut w: *mut window) {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    while !(*w).last_panes.tqh_first.is_null() {
        wp = (*w).last_panes.tqh_first;
        window_pane_stack_remove(&raw mut (*w).last_panes, wp);
    }
    while !(*w).panes.tqh_first.is_null() {
        wp = (*w).panes.tqh_first;
        if !(*wp).entry.tqe_next.is_null() {
            (*(*wp).entry.tqe_next).entry.tqe_prev = (*wp).entry.tqe_prev;
        } else {
            (*w).panes.tqh_last = (*wp).entry.tqe_prev;
        }
        *(*wp).entry.tqe_prev = (*wp).entry.tqe_next;
        if !(*wp).zentry.tqe_next.is_null() {
            (*(*wp).zentry.tqe_next).zentry.tqe_prev = (*wp).zentry.tqe_prev;
        } else {
            (*w).z_index.tqh_last = (*wp).zentry.tqe_prev;
        }
        *(*wp).zentry.tqe_prev = (*wp).zentry.tqe_next;
        window_pane_destroy(wp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_printable_flags(
    mut wl: *mut winlink,
    mut escape: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    let mut s: *mut session = (*wl).session;
    static mut flags: [::core::ffi::c_char; 32] = [0; 32];
    let mut pos: u_int = 0 as u_int;
    if (*wl).flags & WINLINK_ACTIVITY != 0 {
        let fresh3 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh3 as usize] = '#' as i32 as ::core::ffi::c_char;
        if escape != 0 {
            let fresh4 = pos;
            pos = pos.wrapping_add(1);
            flags[fresh4 as usize] = '#' as i32 as ::core::ffi::c_char;
        }
    }
    if (*wl).flags & WINLINK_BELL != 0 {
        let fresh5 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh5 as usize] = '!' as i32 as ::core::ffi::c_char;
    }
    if (*wl).flags & WINLINK_SILENCE != 0 {
        let fresh6 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh6 as usize] = '~' as i32 as ::core::ffi::c_char;
    }
    if wl == (*s).curw {
        let fresh7 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh7 as usize] = '*' as i32 as ::core::ffi::c_char;
    }
    if wl == (*s).lastw.tqh_first {
        let fresh8 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh8 as usize] = '-' as i32 as ::core::ffi::c_char;
    }
    if server_check_marked() != 0 && wl == marked_pane.wl {
        let fresh9 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh9 as usize] = 'M' as i32 as ::core::ffi::c_char;
    }
    if !(*(*wl).window).modal.is_null() {
        let fresh10 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh10 as usize] = 'O' as i32 as ::core::ffi::c_char;
    }
    if (*(*wl).window).flags & WINDOW_ZOOMED != 0 {
        let fresh11 = pos;
        pos = pos.wrapping_add(1);
        flags[fresh11 as usize] = 'Z' as i32 as ::core::ffi::c_char;
    }
    flags[pos as usize] = '\0' as i32 as ::core::ffi::c_char;
    return &raw mut flags as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_printable_flags(
    mut wp: *mut window_pane,
) -> *const ::core::ffi::c_char {
    let mut w: *mut window = (*wp).window as *mut window;
    static mut flags: [::core::ffi::c_char; 32] = [0; 32];
    let mut pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if wp == (*w).active {
        let fresh12 = pos;
        pos = pos + 1;
        flags[fresh12 as usize] = '*' as i32 as ::core::ffi::c_char;
    }
    if wp == (*w).last_panes.tqh_first {
        let fresh13 = pos;
        pos = pos + 1;
        flags[fresh13 as usize] = '-' as i32 as ::core::ffi::c_char;
    }
    if (*wp).flags & PANE_ZOOMED != 0 {
        let fresh14 = pos;
        pos = pos + 1;
        flags[fresh14 as usize] = 'Z' as i32 as ::core::ffi::c_char;
    }
    if window_pane_is_floating(wp) != 0 {
        let fresh15 = pos;
        pos = pos + 1;
        flags[fresh15 as usize] = 'F' as i32 as ::core::ffi::c_char;
    }
    if (*wp).flags & PANE_FLOATOVERZOOM != 0 {
        let fresh16 = pos;
        pos = pos + 1;
        flags[fresh16 as usize] = 'A' as i32 as ::core::ffi::c_char;
    }
    if wp == (*w).modal {
        let fresh17 = pos;
        pos = pos + 1;
        flags[fresh17 as usize] = 'O' as i32 as ::core::ffi::c_char;
    }
    flags[pos as usize] = '\0' as i32 as ::core::ffi::c_char;
    return &raw mut flags as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_find_by_id_str(
    mut s: *const ::core::ffi::c_char,
) -> *mut window_pane {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: u_int = 0;
    if *s as ::core::ffi::c_int != '%' as i32 {
        return ::core::ptr::null_mut::<window_pane>();
    }
    id = strtonum(
        s.offset(1 as ::core::ffi::c_int as isize),
        0 as ::core::ffi::c_longlong,
        UINT_MAX as ::core::ffi::c_longlong,
        &raw mut errstr,
    ) as u_int;
    if !errstr.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    return window_pane_find_by_id(id);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_find_by_id(mut id: u_int) -> *mut window_pane {
    let mut wp: window_pane = window_pane {
        id: 0,
        references: 0,
        active_point: 0,
        window: ::core::ptr::null_mut::<window>(),
        options: ::core::ptr::null_mut::<options>(),
        layout_cell: ::core::ptr::null_mut::<layout_cell>(),
        saved_layout_cell: ::core::ptr::null_mut::<layout_cell>(),
        sx: 0,
        sy: 0,
        xoff: 0,
        yoff: 0,
        flags: 0,
        sync_dirty: ::core::ptr::null_mut::<bitstr_t>(),
        sync_dirty_size: 0,
        sb_slider_y: 0,
        sb_slider_h: 0,
        sb_auto_visible: 0,
        sb_auto_hover: 0,
        sb_auto_timer: event {
            ev_evcallback: event_callback {
                evcb_active_next: C2RustUnnamed_10 {
                    tqe_next: ::core::ptr::null_mut::<event_callback>(),
                    tqe_prev: ::core::ptr::null_mut::<*mut event_callback>(),
                },
                evcb_flags: 0,
                evcb_pri: 0,
                evcb_closure: 0,
                evcb_cb_union: C2RustUnnamed_9 {
                    evcb_callback: None,
                },
                evcb_arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
            ev_timeout_pos: C2RustUnnamed_7 {
                ev_next_with_common_timeout: C2RustUnnamed_8 {
                    tqe_next: ::core::ptr::null_mut::<event>(),
                    tqe_prev: ::core::ptr::null_mut::<*mut event>(),
                },
            },
            ev_fd: 0,
            ev_base: ::core::ptr::null_mut::<event_base>(),
            ev_: C2RustUnnamed_2 {
                ev_io: C2RustUnnamed_5 {
                    ev_io_next: C2RustUnnamed_6 {
                        le_next: ::core::ptr::null_mut::<event>(),
                        le_prev: ::core::ptr::null_mut::<*mut event>(),
                    },
                    ev_timeout: timeval {
                        tv_sec: 0,
                        tv_usec: 0,
                    },
                },
            },
            ev_events: 0,
            ev_res: 0,
            ev_timeout: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
        },
        argc: 0,
        argv: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        shell: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cwd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        pid: 0,
        tty: [0; 32],
        status: 0,
        dead_time: timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        wait_item: ::core::ptr::null_mut::<cmdq_item>(),
        editor: ::core::ptr::null_mut::<spawn_editor_state>(),
        output_generation: 0,
        last_output_time: 0,
        last_prompt_time: 0,
        cmd_start_time: 0,
        cmd_end_time: 0,
        cmd_status: 0,
        fd: 0,
        event: ::core::ptr::null_mut::<bufferevent>(),
        offset: window_pane_offset { used: 0 },
        base_offset: 0,
        resize_queue: window_pane_resizes {
            tqh_first: ::core::ptr::null_mut::<window_pane_resize>(),
            tqh_last: ::core::ptr::null_mut::<*mut window_pane_resize>(),
        },
        resize_timer: event {
            ev_evcallback: event_callback {
                evcb_active_next: C2RustUnnamed_10 {
                    tqe_next: ::core::ptr::null_mut::<event_callback>(),
                    tqe_prev: ::core::ptr::null_mut::<*mut event_callback>(),
                },
                evcb_flags: 0,
                evcb_pri: 0,
                evcb_closure: 0,
                evcb_cb_union: C2RustUnnamed_9 {
                    evcb_callback: None,
                },
                evcb_arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
            ev_timeout_pos: C2RustUnnamed_7 {
                ev_next_with_common_timeout: C2RustUnnamed_8 {
                    tqe_next: ::core::ptr::null_mut::<event>(),
                    tqe_prev: ::core::ptr::null_mut::<*mut event>(),
                },
            },
            ev_fd: 0,
            ev_base: ::core::ptr::null_mut::<event_base>(),
            ev_: C2RustUnnamed_2 {
                ev_io: C2RustUnnamed_5 {
                    ev_io_next: C2RustUnnamed_6 {
                        le_next: ::core::ptr::null_mut::<event>(),
                        le_prev: ::core::ptr::null_mut::<*mut event>(),
                    },
                    ev_timeout: timeval {
                        tv_sec: 0,
                        tv_usec: 0,
                    },
                },
            },
            ev_events: 0,
            ev_res: 0,
            ev_timeout: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
        },
        sync_timer: event {
            ev_evcallback: event_callback {
                evcb_active_next: C2RustUnnamed_10 {
                    tqe_next: ::core::ptr::null_mut::<event_callback>(),
                    tqe_prev: ::core::ptr::null_mut::<*mut event_callback>(),
                },
                evcb_flags: 0,
                evcb_pri: 0,
                evcb_closure: 0,
                evcb_cb_union: C2RustUnnamed_9 {
                    evcb_callback: None,
                },
                evcb_arg: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            },
            ev_timeout_pos: C2RustUnnamed_7 {
                ev_next_with_common_timeout: C2RustUnnamed_8 {
                    tqe_next: ::core::ptr::null_mut::<event>(),
                    tqe_prev: ::core::ptr::null_mut::<*mut event>(),
                },
            },
            ev_fd: 0,
            ev_base: ::core::ptr::null_mut::<event_base>(),
            ev_: C2RustUnnamed_2 {
                ev_io: C2RustUnnamed_5 {
                    ev_io_next: C2RustUnnamed_6 {
                        le_next: ::core::ptr::null_mut::<event>(),
                        le_prev: ::core::ptr::null_mut::<*mut event>(),
                    },
                    ev_timeout: timeval {
                        tv_sec: 0,
                        tv_usec: 0,
                    },
                },
            },
            ev_events: 0,
            ev_res: 0,
            ev_timeout: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
        },
        ictx: ::core::ptr::null_mut::<input_ctx>(),
        cached_gc: grid_cell {
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
        cached_active_gc: grid_cell {
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
        cached_dim: 0,
        cached_active_dim: 0,
        palette: colour_palette {
            fg: 0,
            bg: 0,
            palette: ::core::ptr::null_mut::<::core::ffi::c_int>(),
            default_palette: ::core::ptr::null_mut::<::core::ffi::c_int>(),
        },
        last_theme: THEME_UNKNOWN,
        border_status_line: style_line_entry {
            expanded: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ranges: style_ranges {
                tqh_first: ::core::ptr::null_mut::<style_range>(),
                tqh_last: ::core::ptr::null_mut::<*mut style_range>(),
            },
        },
        pipe_fd: 0,
        pipe_pid: 0,
        pipe_event: ::core::ptr::null_mut::<bufferevent>(),
        pipe_offset: window_pane_offset { used: 0 },
        screen: ::core::ptr::null_mut::<screen>(),
        base: screen {
            title: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            titles: ::core::ptr::null_mut::<screen_titles>(),
            ntitles: 0,
            grid: ::core::ptr::null_mut::<grid>(),
            cx: 0,
            cy: 0,
            cstyle: SCREEN_CURSOR_DEFAULT,
            default_cstyle: SCREEN_CURSOR_DEFAULT,
            ccolour: 0,
            default_ccolour: 0,
            rupper: 0,
            rlower: 0,
            mode: 0,
            default_mode: 0,
            saved_cx: 0,
            saved_cy: 0,
            saved_grid: ::core::ptr::null_mut::<grid>(),
            saved_cell: grid_cell {
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
            saved_flags: 0,
            tabs: ::core::ptr::null_mut::<bitstr_t>(),
            sel: ::core::ptr::null_mut::<screen_sel>(),
            write_list: ::core::ptr::null_mut::<screen_write_cline>(),
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
            progress_bar: progress_bar {
                state: PROGRESS_BAR_HIDDEN,
                progress: 0,
            },
        },
        status_screen: screen {
            title: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            path: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            titles: ::core::ptr::null_mut::<screen_titles>(),
            ntitles: 0,
            grid: ::core::ptr::null_mut::<grid>(),
            cx: 0,
            cy: 0,
            cstyle: SCREEN_CURSOR_DEFAULT,
            default_cstyle: SCREEN_CURSOR_DEFAULT,
            ccolour: 0,
            default_ccolour: 0,
            rupper: 0,
            rlower: 0,
            mode: 0,
            default_mode: 0,
            saved_cx: 0,
            saved_cy: 0,
            saved_grid: ::core::ptr::null_mut::<grid>(),
            saved_cell: grid_cell {
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
            saved_flags: 0,
            tabs: ::core::ptr::null_mut::<bitstr_t>(),
            sel: ::core::ptr::null_mut::<screen_sel>(),
            write_list: ::core::ptr::null_mut::<screen_write_cline>(),
            hyperlinks: ::core::ptr::null_mut::<hyperlinks>(),
            progress_bar: progress_bar {
                state: PROGRESS_BAR_HIDDEN,
                progress: 0,
            },
        },
        modes: C2RustUnnamed_28 {
            tqh_first: ::core::ptr::null_mut::<window_mode_entry>(),
            tqh_last: ::core::ptr::null_mut::<*mut window_mode_entry>(),
        },
        searchstr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        searchregex: 0,
        prompt: ::core::ptr::null_mut::<prompt>(),
        prompt_data: ::core::ptr::null_mut::<window_pane_prompt>(),
        prompt_cx: 0,
        border_gc_set: 0,
        border_gc: grid_cell {
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
        active_border_gc_set: 0,
        active_border_gc: grid_cell {
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
        control_bg: 0,
        control_fg: 0,
        scrollbar_style: style {
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
        r: visible_ranges {
            ranges: ::core::ptr::null_mut::<visible_range>(),
            used: 0,
            size: 0,
        },
        entry: C2RustUnnamed_27 {
            tqe_next: ::core::ptr::null_mut::<window_pane>(),
            tqe_prev: ::core::ptr::null_mut::<*mut window_pane>(),
        },
        sentry: C2RustUnnamed_26 {
            tqe_next: ::core::ptr::null_mut::<window_pane>(),
            tqe_prev: ::core::ptr::null_mut::<*mut window_pane>(),
        },
        zentry: C2RustUnnamed_25 {
            tqe_next: ::core::ptr::null_mut::<window_pane>(),
            tqe_prev: ::core::ptr::null_mut::<*mut window_pane>(),
        },
        tree_entry: C2RustUnnamed_24 {
            rbe_left: ::core::ptr::null_mut::<window_pane>(),
            rbe_right: ::core::ptr::null_mut::<window_pane>(),
            rbe_parent: ::core::ptr::null_mut::<window_pane>(),
            rbe_color: 0,
        },
    };
    wp.id = id;
    return window_pane_tree_RB_FIND(&raw mut all_window_panes, &raw mut wp);
}
unsafe extern "C" fn window_pane_create(
    mut w: *mut window,
    mut sx: u_int,
    mut sy: u_int,
    mut hlimit: u_int,
) -> *mut window_pane {
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut host: [::core::ffi::c_char; 65] = [0; 65];
    wp = xcalloc(1 as size_t, ::core::mem::size_of::<window_pane>() as size_t) as *mut window_pane;
    (*wp).references = 1 as ::core::ffi::c_int;
    (*wp).window = w as *mut window;
    (*wp).options = options_create((*w).options);
    (*wp).flags = PANE_STYLECHANGED;
    (*wp).cmd_status = -(1 as ::core::ffi::c_int);
    let fresh2 = next_window_pane_id;
    next_window_pane_id = next_window_pane_id.wrapping_add(1);
    (*wp).id = fresh2;
    window_pane_tree_RB_INSERT(&raw mut all_window_panes, wp);
    (*wp).fd = -(1 as ::core::ffi::c_int);
    (*wp).modes.tqh_first = ::core::ptr::null_mut::<window_mode_entry>();
    (*wp).modes.tqh_last = &raw mut (*wp).modes.tqh_first;
    (*wp).resize_queue.tqh_first = ::core::ptr::null_mut::<window_pane_resize>();
    (*wp).resize_queue.tqh_last = &raw mut (*wp).resize_queue.tqh_first;
    (*wp).sx = sx;
    (*wp).sy = sy;
    (*wp).pipe_fd = -(1 as ::core::ffi::c_int);
    (*wp).control_bg = -(1 as ::core::ffi::c_int);
    (*wp).control_fg = -(1 as ::core::ffi::c_int);
    style_set_scrollbar_style_from_option(&raw mut (*wp).scrollbar_style, (*wp).options);
    colour_palette_init(&raw mut (*wp).palette);
    colour_palette_from_option(&raw mut (*wp).palette, (*wp).options);
    screen_init(&raw mut (*wp).base, sx, sy, hlimit);
    (*wp).screen = &raw mut (*wp).base;
    window_pane_default_cursor(wp);
    screen_init(
        &raw mut (*wp).status_screen,
        1 as u_int,
        1 as u_int,
        0 as u_int,
    );
    style_ranges_init(&raw mut (*wp).border_status_line.ranges);
    event_set(
        &raw mut (*wp).sb_auto_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            window_pane_scrollbar_timer
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        wp as *mut ::core::ffi::c_void,
    );
    if gethostname(
        &raw mut host as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 65]>() as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        screen_set_title(
            &raw mut (*wp).base,
            &raw mut host as *mut ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        );
    }
    return wp;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_wait_finish(mut wp: *mut window_pane) {
    let mut item: *mut cmdq_item = (*wp).wait_item;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if item.is_null() {
        return;
    }
    (*wp).wait_item = ::core::ptr::null_mut::<cmdq_item>();
    if (*wp).flags & PANE_STATUSREADY != 0 {
        if (*wp).status & 0x7f as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            retval = ((*wp).status & 0xff00 as ::core::ffi::c_int) >> 8 as ::core::ffi::c_int;
        } else if (((*wp).status & 0x7f as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
            as ::core::ffi::c_schar as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int
            > 0 as ::core::ffi::c_int
        {
            retval = ((*wp).status & 0x7f as ::core::ffi::c_int) + 128 as ::core::ffi::c_int;
        }
    }
    c = cmdq_get_client(item);
    if !c.is_null() && (*c).session.is_null() {
        (*c).retval = retval;
    }
    cmdq_continue(item);
}
unsafe extern "C" fn window_pane_free_modes(mut wp: *mut window_pane) {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    while !(*wp).modes.tqh_first.is_null() {
        wme = (*wp).modes.tqh_first;
        if !(*wme).entry.tqe_next.is_null() {
            (*(*wme).entry.tqe_next).entry.tqe_prev = (*wme).entry.tqe_prev;
        } else {
            (*wp).modes.tqh_last = (*wme).entry.tqe_prev;
        }
        *(*wme).entry.tqe_prev = (*wme).entry.tqe_next;
        (*(*wme).mode).free.expect("non-null function pointer")(wme);
        free(wme as *mut ::core::ffi::c_void);
    }
    (*wp).screen = &raw mut (*wp).base;
}
unsafe extern "C" fn window_pane_scrollbar_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = arg as *mut window_pane;
    (*wp).sb_auto_hover = 0 as ::core::ffi::c_int;
    window_pane_scrollbar_hide(wp);
}
unsafe extern "C" fn window_pane_scrollbar_auto_hide(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    return ((*(*wp).window).sb == PANE_SCROLLBARS_MODAL
        || (*(*wp).window).sb == PANE_SCROLLBARS_AUTOHIDE) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_scrollbar_overlay_visible(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    return (window_pane_scrollbar_overlay(wp) != 0 && window_pane_scrollbar_visible(wp) != 0)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_scrollbar_redraw(mut wp: *mut window_pane) {
    if window_pane_scrollbar_visible(wp) == 0 {
        return;
    }
    if window_pane_scrollbar_overlay_visible(wp) != 0 {
        (*wp).flags |= PANE_REDRAW;
        return;
    }
    (*wp).flags |= PANE_REDRAWSCROLLBAR;
}
unsafe extern "C" fn window_pane_scrollbar_redraw_visibility(mut wp: *mut window_pane) {
    redraw_invalidate_scene((*wp).window as *mut window);
    (*wp).flags |= PANE_REDRAW;
    server_redraw_window((*wp).window as *mut window);
}
unsafe extern "C" fn window_pane_destroy(mut wp: *mut window_pane) {
    window_pane_wait_finish(wp);
    spawn_editor_finish(wp);
    window_pane_tree_RB_REMOVE(&raw mut all_window_panes, wp);
    (*wp).flags |= PANE_DESTROYED;
    window_pane_clear_prompt(wp);
    window_pane_free_modes(wp);
    screen_write_clear_dirty(wp);
    if (*wp).fd != -(1 as ::core::ffi::c_int) {
        utempter_remove_record((*wp).fd);
        kill(getpid(), SIGCHLD);
        bufferevent_free((*wp).event);
        (*wp).event = ::core::ptr::null_mut::<bufferevent>();
        close((*wp).fd);
        (*wp).fd = -(1 as ::core::ffi::c_int);
    }
    if !(*wp).ictx.is_null() {
        input_free((*wp).ictx);
        (*wp).ictx = ::core::ptr::null_mut::<input_ctx>();
    }
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
        bufferevent_free((*wp).pipe_event);
        (*wp).pipe_event = ::core::ptr::null_mut::<bufferevent>();
        close((*wp).pipe_fd);
        (*wp).pipe_fd = -(1 as ::core::ffi::c_int);
    }
    if event_initialized(&raw mut (*wp).resize_timer) != 0 {
        event_del(&raw mut (*wp).resize_timer);
    }
    if event_initialized(&raw mut (*wp).sync_timer) != 0 {
        event_del(&raw mut (*wp).sync_timer);
    }
    if event_initialized(&raw mut (*wp).sb_auto_timer) != 0 {
        event_del(&raw mut (*wp).sb_auto_timer);
    }
    window_pane_clear_resizes(wp, ::core::ptr::null_mut::<window_pane_resize>());
    window_pane_remove_ref(
        wp,
        b"window_pane_destroy\0" as *const u8 as *const ::core::ffi::c_char,
    );
}
unsafe extern "C" fn window_pane_free(mut wp: *mut window_pane) {
    log_debug(
        b"pane %%%u freed (%d references)\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        (*wp).references,
    );
    free((*wp).searchstr as *mut ::core::ffi::c_void);
    screen_free(&raw mut (*wp).status_screen);
    screen_free(&raw mut (*wp).base);
    free((*wp).r.ranges as *mut ::core::ffi::c_void);
    options_free((*wp).options);
    free((*wp).cwd as *mut ::core::ffi::c_void);
    free((*wp).shell as *mut ::core::ffi::c_void);
    cmd_free_argv((*wp).argc, (*wp).argv);
    colour_palette_free(&raw mut (*wp).palette);
    style_ranges_free(&raw mut (*wp).border_status_line.ranges);
    free((*wp).border_status_line.expanded as *mut ::core::ffi::c_void);
    free(wp as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_pane_read_callback(
    mut bufev: *mut bufferevent,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = data as *mut window_pane;
    let mut evb: *mut evbuffer = (*(*wp).event).input;
    let mut wpo: *mut window_pane_offset = &raw mut (*wp).pipe_offset;
    let mut size: size_t = evbuffer_get_length(evb);
    let mut new_data: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut new_size: size_t = 0;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
        new_data = window_pane_get_new_data(wp, wpo, &raw mut new_size) as *mut ::core::ffi::c_char;
        if new_size > 0 as size_t {
            bufferevent_write(
                (*wp).pipe_event,
                new_data as *const ::core::ffi::c_void,
                new_size,
            );
            window_pane_update_used_data(wp, wpo, new_size);
        }
    }
    log_debug(
        b"%%%u has %zu bytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        size,
    );
    c = clients.tqh_first;
    while !c.is_null() {
        if !(*c).session.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_write_output(c, wp);
        }
        c = (*c).entry.tqe_next;
    }
    input_parse_pane(wp);
    bufferevent_disable((*wp).event, EV_READ as ::core::ffi::c_short);
}
unsafe extern "C" fn window_pane_error_callback(
    mut bufev: *mut bufferevent,
    mut what: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = data as *mut window_pane;
    log_debug(
        b"%%%u error\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
    (*wp).flags |= PANE_EXITED;
    if window_pane_destroy_ready(wp) != 0 {
        server_destroy_pane(wp, 1 as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_set_event(mut wp: *mut window_pane) {
    setblocking((*wp).fd, 0 as ::core::ffi::c_int);
    (*wp).event = bufferevent_new(
        (*wp).fd,
        Some(
            window_pane_read_callback
                as unsafe extern "C" fn(*mut bufferevent, *mut ::core::ffi::c_void) -> (),
        ),
        None,
        Some(
            window_pane_error_callback
                as unsafe extern "C" fn(
                    *mut bufferevent,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        wp as *mut ::core::ffi::c_void,
    );
    if (*wp).event.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    (*wp).ictx = input_init(
        wp,
        (*wp).event,
        &raw mut (*wp).palette,
        ::core::ptr::null_mut::<client>(),
    );
    bufferevent_enable((*wp).event, (EV_READ | EV_WRITE) as ::core::ffi::c_short);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_clear_resizes(
    mut wp: *mut window_pane,
    mut except: *mut window_pane_resize,
) {
    let mut r: *mut window_pane_resize = ::core::ptr::null_mut::<window_pane_resize>();
    let mut r1: *mut window_pane_resize = ::core::ptr::null_mut::<window_pane_resize>();
    r = (*wp).resize_queue.tqh_first;
    while !r.is_null() && {
        r1 = (*r).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(r == except) {
            if !(*r).entry.tqe_next.is_null() {
                (*(*r).entry.tqe_next).entry.tqe_prev = (*r).entry.tqe_prev;
            } else {
                (*wp).resize_queue.tqh_last = (*r).entry.tqe_prev;
            }
            *(*r).entry.tqe_prev = (*r).entry.tqe_next;
            free(r as *mut ::core::ffi::c_void);
        }
        r = r1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_resize(
    mut wp: *mut window_pane,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut r: *mut window_pane_resize = ::core::ptr::null_mut::<window_pane_resize>();
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null::<cmd_find_state>() as *mut cmd_find_state,
        s: ::core::ptr::null::<session>() as *mut session,
        wl: ::core::ptr::null::<winlink>() as *mut winlink,
        w: ::core::ptr::null::<window>() as *mut window,
        wp: ::core::ptr::null::<window_pane>() as *mut window_pane,
        idx: 0,
    };
    if sx == (*wp).sx && sy == (*wp).sy {
        return;
    }
    screen_write_stop_sync(wp);
    r = xmalloc(::core::mem::size_of::<window_pane_resize>() as size_t) as *mut window_pane_resize;
    (*r).sx = sx;
    (*r).sy = sy;
    (*r).osx = (*wp).sx;
    (*r).osy = (*wp).sy;
    (*r).entry.tqe_next = ::core::ptr::null_mut::<window_pane_resize>();
    (*r).entry.tqe_prev = (*wp).resize_queue.tqh_last;
    *(*wp).resize_queue.tqh_last = r;
    (*wp).resize_queue.tqh_last = &raw mut (*r).entry.tqe_next;
    (*wp).sx = sx;
    (*wp).sy = sy;
    log_debug(
        b"%s: %%%u resize %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_pane_resize\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        sx,
        sy,
    );
    screen_resize(
        &raw mut (*wp).base,
        sx,
        sy,
        ((*wp).base.saved_grid == NULL as *mut grid) as ::core::ffi::c_int,
    );
    wme = (*wp).modes.tqh_first;
    if !wme.is_null() && (*(*wme).mode).resize.is_some() {
        (*(*wme).mode).resize.expect("non-null function pointer")(wme, sx, sy);
    }
    ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    event_payload_set_uint(
        ep,
        b"width\0" as *const u8 as *const ::core::ffi::c_char,
        sx,
    );
    event_payload_set_uint(
        ep,
        b"height\0" as *const u8 as *const ::core::ffi::c_char,
        sy,
    );
    event_payload_set_uint(
        ep,
        b"old_width\0" as *const u8 as *const ::core::ffi::c_char,
        (*r).osx,
    );
    event_payload_set_uint(
        ep,
        b"old_height\0" as *const u8 as *const ::core::ffi::c_char,
        (*r).osy,
    );
    events_fire(
        b"pane-resized\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_set_mode(
    mut wp: *mut window_pane,
    mut swp: *mut window_pane,
    mut mode: *const window_mode,
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> ::core::ffi::c_int {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut w: *mut window = (*wp).window as *mut window;
    let mut name: *const ::core::ffi::c_char = (*mode).name;
    let mut oname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !(*wp).modes.tqh_first.is_null() {
        if (*(*wp).modes.tqh_first).mode == mode {
            return 1 as ::core::ffi::c_int;
        }
        if (*(*(*wp).modes.tqh_first).mode).flags & WINDOW_MODE_NO_STACK != 0 {
            window_pane_reset_mode(wp);
        }
    }
    if !(*wp).modes.tqh_first.is_null() {
        oname = (*(*(*wp).modes.tqh_first).mode).name;
    }
    wme = (*wp).modes.tqh_first;
    while !wme.is_null() {
        if (*wme).mode == mode {
            break;
        }
        wme = (*wme).entry.tqe_next;
    }
    if !wme.is_null() {
        if !(*wme).entry.tqe_next.is_null() {
            (*(*wme).entry.tqe_next).entry.tqe_prev = (*wme).entry.tqe_prev;
        } else {
            (*wp).modes.tqh_last = (*wme).entry.tqe_prev;
        }
        *(*wme).entry.tqe_prev = (*wme).entry.tqe_next;
        (*wme).entry.tqe_next = (*wp).modes.tqh_first;
        if !(*wme).entry.tqe_next.is_null() {
            (*(*wp).modes.tqh_first).entry.tqe_prev = &raw mut (*wme).entry.tqe_next;
        } else {
            (*wp).modes.tqh_last = &raw mut (*wme).entry.tqe_next;
        }
        (*wp).modes.tqh_first = wme;
        (*wme).entry.tqe_prev = &raw mut (*wp).modes.tqh_first;
    } else {
        wme = xcalloc(
            1 as size_t,
            ::core::mem::size_of::<window_mode_entry>() as size_t,
        ) as *mut window_mode_entry;
        (*wme).wp = wp;
        (*wme).swp = swp;
        (*wme).mode = mode;
        (*wme).prefix = 1 as u_int;
        (*wme).entry.tqe_next = (*wp).modes.tqh_first;
        if !(*wme).entry.tqe_next.is_null() {
            (*(*wp).modes.tqh_first).entry.tqe_prev = &raw mut (*wme).entry.tqe_next;
        } else {
            (*wp).modes.tqh_last = &raw mut (*wme).entry.tqe_next;
        }
        (*wp).modes.tqh_first = wme;
        (*wme).entry.tqe_prev = &raw mut (*wp).modes.tqh_first;
        (*wme).screen =
            (*(*wme).mode).init.expect("non-null function pointer")(wme, item, fs, args);
        if (*wme).screen.is_null() {
            if !(*wme).entry.tqe_next.is_null() {
                (*(*wme).entry.tqe_next).entry.tqe_prev = (*wme).entry.tqe_prev;
            } else {
                (*wp).modes.tqh_last = (*wme).entry.tqe_prev;
            }
            *(*wme).entry.tqe_prev = (*wme).entry.tqe_next;
            free(wme as *mut ::core::ffi::c_void);
            return 1 as ::core::ffi::c_int;
        }
    }
    (*wme).kill = if !args.is_null() {
        args_has(args, 'k' as i32 as u_char)
    } else {
        0 as ::core::ffi::c_int
    };
    (*wp).screen = (*wme).screen;
    (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR | PANE_CHANGED;
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    server_redraw_window_borders((*wp).window as *mut window);
    server_status_window((*wp).window as *mut window);
    window_fire_pane_mode_changed(
        b"pane-mode-entered\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
        oname,
        name,
        1 as ::core::ffi::c_int,
    );
    window_fire_pane_mode_changed(
        b"pane-mode-changed\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
        oname,
        name,
        1 as ::core::ffi::c_int,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_reset_mode(mut wp: *mut window_pane) {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut next: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut w: *mut window = (*wp).window as *mut window;
    let mut kill_0: ::core::ffi::c_int = 0;
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*wp).modes.tqh_first.is_null() {
        return;
    }
    wme = (*wp).modes.tqh_first;
    p = (*(*wme).mode).name;
    kill_0 = (*wme).kill;
    if !(*wme).entry.tqe_next.is_null() {
        (*(*wme).entry.tqe_next).entry.tqe_prev = (*wme).entry.tqe_prev;
    } else {
        (*wp).modes.tqh_last = (*wme).entry.tqe_prev;
    }
    *(*wme).entry.tqe_prev = (*wme).entry.tqe_next;
    (*(*wme).mode).free.expect("non-null function pointer")(wme);
    free(wme as *mut ::core::ffi::c_void);
    next = (*wp).modes.tqh_first;
    if next.is_null() {
        (*wp).flags &= !PANE_UNSEENCHANGES;
        log_debug(
            b"%s: no next mode\0" as *const u8 as *const ::core::ffi::c_char,
            b"window_pane_reset_mode\0" as *const u8 as *const ::core::ffi::c_char,
        );
        (*wp).screen = &raw mut (*wp).base;
    } else {
        log_debug(
            b"%s: next mode is %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"window_pane_reset_mode\0" as *const u8 as *const ::core::ffi::c_char,
            (*(*next).mode).name,
        );
        (*wp).screen = (*next).screen;
        if (*(*next).mode).resize.is_some() {
            (*(*next).mode).resize.expect("non-null function pointer")(next, (*wp).sx, (*wp).sy);
        }
    }
    name = if next.is_null() {
        ::core::ptr::null::<::core::ffi::c_char>()
    } else {
        (*(*next).mode).name
    };
    (*wp).flags |= PANE_REDRAW | PANE_REDRAWSCROLLBAR | PANE_CHANGED;
    layout_fix_panes(w, ::core::ptr::null_mut::<window_pane>());
    server_redraw_window_borders((*wp).window as *mut window);
    server_status_window((*wp).window as *mut window);
    window_fire_pane_mode_changed(
        b"pane-mode-exited\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
        p,
        name,
        0 as ::core::ffi::c_int,
    );
    window_fire_pane_mode_changed(
        b"pane-mode-changed\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
        p,
        name,
        0 as ::core::ffi::c_int,
    );
    if kill_0 != 0 {
        server_kill_pane(wp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_reset_mode_all(mut wp: *mut window_pane) {
    while !(*wp).modes.tqh_first.is_null() {
        window_pane_reset_mode(wp);
    }
}
unsafe extern "C" fn window_pane_prompt_input_callback(
    mut data: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut wpp: *mut window_pane_prompt = data as *mut window_pane_prompt;
    if (*wpp).inputcb.is_some() {
        return (*wpp).inputcb.expect("non-null function pointer")((*wpp).c, (*wpp).data, s, key);
    }
    return PROMPT_CLOSE;
}
unsafe extern "C" fn window_pane_prompt_free_callback(mut data: *mut ::core::ffi::c_void) {
    let mut wpp: *mut window_pane_prompt = data as *mut window_pane_prompt;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    wp = window_pane_find_by_id((*wpp).wp_id);
    if !wp.is_null() && (*wp).prompt_data == wpp {
        (*wp).prompt_data = ::core::ptr::null_mut::<window_pane_prompt>();
    }
    if (*wpp).freecb.is_some() {
        (*wpp).freecb.expect("non-null function pointer")((*wpp).data);
    }
    free(wpp as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_set_prompt(
    mut wp: *mut window_pane,
    mut c: *mut client,
    mut fs: *mut cmd_find_state,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
    mut inputcb: status_prompt_input_cb,
    mut freecb: prompt_free_cb,
    mut data: *mut ::core::ffi::c_void,
    mut flags: ::core::ffi::c_int,
    mut type_0: prompt_type,
) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut pd: prompt_create_data = prompt_create_data {
        fs: ::core::ptr::null_mut::<cmd_find_state>(),
        prompt: ::core::ptr::null::<::core::ffi::c_char>(),
        input: ::core::ptr::null::<::core::ffi::c_char>(),
        type_0: PROMPT_TYPE_COMMAND,
        flags: 0,
        style: grid_cell {
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
        command_style: grid_cell {
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
        style_str: ::core::ptr::null::<::core::ffi::c_char>(),
        command_style_str: ::core::ptr::null::<::core::ffi::c_char>(),
        cstyle: SCREEN_CURSOR_DEFAULT,
        command_cstyle: SCREEN_CURSOR_DEFAULT,
        ccolour: 0,
        command_ccolour: 0,
        cmode: 0,
        command_cmode: 0,
        message_format: ::core::ptr::null::<::core::ffi::c_char>(),
        keys: 0,
        word_separators: ::core::ptr::null::<::core::ffi::c_char>(),
        inputcb: None,
        freecb: None,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut wpp: *mut window_pane_prompt = ::core::ptr::null_mut::<window_pane_prompt>();
    if !c.is_null() {
        s = (*c).session;
    }
    window_pane_clear_prompt(wp);
    wpp = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_pane_prompt>() as size_t,
    ) as *mut window_pane_prompt;
    (*wpp).wp_id = (*wp).id;
    (*wpp).c = c;
    (*wpp).inputcb = inputcb;
    (*wpp).freecb = freecb;
    (*wpp).data = data;
    (*wpp).type_0 = type_0;
    memset(
        &raw mut pd as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<prompt_create_data>() as size_t,
    );
    prompt_set_options(&raw mut pd, s);
    pd.fs = fs;
    pd.prompt = msg;
    pd.input = input;
    pd.type_0 = type_0;
    pd.flags = flags;
    pd.inputcb = Some(
        window_pane_prompt_input_callback
            as unsafe extern "C" fn(
                *mut ::core::ffi::c_void,
                *const ::core::ffi::c_char,
                prompt_key_result,
            ) -> prompt_result,
    ) as prompt_input_cb;
    pd.freecb = Some(
        window_pane_prompt_free_callback as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
    ) as prompt_free_cb;
    pd.data = wpp as *mut ::core::ffi::c_void;
    (*wp).prompt = prompt_create(&raw mut pd);
    (*wp).prompt_data = wpp;
    (*wp).flags |= PANE_REDRAW;
    prompt_incremental_start((*wp).prompt);
    window_fire_pane_prompt(
        b"pane-prompt-opened\0" as *const u8 as *const ::core::ffi::c_char,
        wp,
        type_0,
    );
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_clear_prompt(mut wp: *mut window_pane) {
    let mut prompt: *mut prompt = (*wp).prompt;
    let mut wpp: *mut window_pane_prompt = (*wp).prompt_data;
    let mut type_0: prompt_type = PROMPT_TYPE_INVALID;
    if !prompt.is_null() {
        if !wpp.is_null() {
            type_0 = (*wpp).type_0;
        }
        (*wp).prompt = ::core::ptr::null_mut::<prompt>();
        prompt_free(prompt);
        (*wp).flags |= PANE_REDRAW;
        if !(*wp).flags & PANE_DESTROYED != 0 {
            window_fire_pane_prompt(
                b"pane-prompt-closed\0" as *const u8 as *const ::core::ffi::c_char,
                wp,
                type_0,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_has_prompt(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    return ((*wp).prompt != NULL as *mut prompt) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_update_prompt(
    mut wp: *mut window_pane,
    mut msg: *const ::core::ffi::c_char,
    mut input: *const ::core::ffi::c_char,
) {
    if !(*wp).prompt.is_null() {
        prompt_update((*wp).prompt, msg, input);
        (*wp).flags |= PANE_REDRAW;
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_prompt_key(
    mut wp: *mut window_pane,
    mut c: *mut client,
    mut key: key_code,
    mut m: *mut mouse_event,
) -> prompt_key_result {
    let mut prompt: *mut prompt = (*wp).prompt;
    let mut wpp: *mut window_pane_prompt = (*wp).prompt_data;
    let mut result: prompt_key_result = PROMPT_KEY_NOT_HANDLED;
    let mut wp_id: u_int = (*wp).id;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut py: u_int = 0;
    let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if prompt.is_null() {
        return PROMPT_KEY_NOT_HANDLED;
    }
    if !wpp.is_null() {
        (*wpp).c = c;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        if m.is_null()
            || (*m).b & MOUSE_MASK_BUTTONS as u_int != MOUSE_BUTTON_1 as u_int
            || (*m).b & MOUSE_MASK_DRAG as u_int != 0
            || (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
            || cmd_mouse_at(wp, m, &raw mut x, &raw mut y, 0 as ::core::ffi::c_int)
                != 0 as ::core::ffi::c_int
        {
            result = PROMPT_KEY_NOT_HANDLED;
        } else {
            if !c.is_null() && status_at_line(c) == 0 as ::core::ffi::c_int {
                py = 0 as u_int;
            } else {
                py = (*wp).sy.wrapping_sub(1 as u_int);
            }
            if y == py {
                result = prompt_mouse(prompt, x, 0 as u_int, (*wp).sx, &raw mut redraw);
            } else {
                result = PROMPT_KEY_NOT_HANDLED;
            }
        }
    } else {
        result = prompt_key(prompt, key, &raw mut redraw);
    }
    wp = window_pane_find_by_id(wp_id);
    if wp.is_null() {
        return result;
    }
    if !wpp.is_null() && (*wp).prompt_data == wpp {
        (*wpp).c = ::core::ptr::null_mut::<client>();
    }
    if (*wp).prompt == prompt
        && (result as ::core::ffi::c_uint
            == PROMPT_KEY_CLOSE as ::core::ffi::c_int as ::core::ffi::c_uint
            || prompt_closed(prompt) != 0)
    {
        window_pane_clear_prompt(wp);
    }
    if redraw != 0 || (*wp).prompt != prompt {
        (*wp).flags |= PANE_REDRAW;
    }
    return result;
}
unsafe extern "C" fn window_pane_copy_paste(
    mut wp: *mut window_pane,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) {
    let mut loop_0: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    loop_0 = (*(*wp).window).panes.tqh_first;
    while !loop_0.is_null() {
        if loop_0 != wp
            && (*loop_0).modes.tqh_first.is_null()
            && (*loop_0).fd != -(1 as ::core::ffi::c_int)
            && !(*loop_0).flags & PANE_INPUTOFF != 0
            && window_pane_is_visible(loop_0) != 0
            && options_get_number(
                (*loop_0).options,
                b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
            ) != 0
        {
            log_debug(
                b"%s: %.*s\0" as *const u8 as *const ::core::ffi::c_char,
                b"window_pane_copy_paste\0" as *const u8 as *const ::core::ffi::c_char,
                len as ::core::ffi::c_int,
                buf,
            );
            bufferevent_write((*loop_0).event, buf as *const ::core::ffi::c_void, len);
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
}
unsafe extern "C" fn window_pane_copy_key(mut wp: *mut window_pane, mut key: key_code) {
    let mut loop_0: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    loop_0 = (*(*wp).window).panes.tqh_first;
    while !loop_0.is_null() {
        if loop_0 != wp
            && (*loop_0).modes.tqh_first.is_null()
            && (*loop_0).fd != -(1 as ::core::ffi::c_int)
            && !(*loop_0).flags & PANE_INPUTOFF != 0
            && window_pane_is_visible(loop_0) != 0
            && options_get_number(
                (*loop_0).options,
                b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
            ) != 0
        {
            input_key_pane(loop_0, key, ::core::ptr::null_mut::<mouse_event>());
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_paste(
    mut wp: *mut window_pane,
    mut key: key_code,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) {
    if !(*wp).modes.tqh_first.is_null() {
        return;
    }
    if (*wp).fd == -(1 as ::core::ffi::c_int) || (*wp).flags & PANE_INPUTOFF != 0 {
        return;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
        == (KEYC_TYPE_FUNCTION as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
            << 32 as ::core::ffi::c_int
        && (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_PASTE_START as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                == KEYC_PASTE_END as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong)
        && !(*(*wp).screen).mode & MODE_BRACKETPASTE != 0
    {
        return;
    }
    log_debug(
        b"%s: %.*s\0" as *const u8 as *const ::core::ffi::c_char,
        b"window_pane_paste\0" as *const u8 as *const ::core::ffi::c_char,
        len as ::core::ffi::c_int,
        buf,
    );
    bufferevent_write((*wp).event, buf as *const ::core::ffi::c_void, len);
    if options_get_number(
        (*wp).options,
        b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        window_pane_copy_paste(wp, buf, len);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_key(
    mut wp: *mut window_pane,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) -> ::core::ffi::c_int {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    if (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int)
        && m.is_null()
    {
        return -(1 as ::core::ffi::c_int);
    }
    wme = (*wp).modes.tqh_first;
    if !wme.is_null() {
        if key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            == (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
        if (*(*wme).mode).key.is_some() && !c.is_null() {
            key &= !KEYC_MASK_FLAGS;
            (*(*wme).mode).key.expect("non-null function pointer")(wme, c, s, wl, key, m);
        }
        return 0 as ::core::ffi::c_int;
    }
    if (*wp).fd == -(1 as ::core::ffi::c_int) || (*wp).flags & PANE_INPUTOFF != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if input_key_pane(wp, key, m) != 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        return 0 as ::core::ffi::c_int;
    }
    if options_get_number(
        (*wp).options,
        b"synchronize-panes\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        window_pane_copy_key(wp, key);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_is_visible(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    if !(*(*wp).window).flags & WINDOW_ZOOMED != 0 {
        return 1 as ::core::ffi::c_int;
    }
    return ((*wp).layout_cell != NULL as *mut layout_cell) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_exited(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    return ((*wp).fd == -(1 as ::core::ffi::c_int) || (*wp).flags & PANE_EXITED != 0)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_search(
    mut wp: *mut window_pane,
    mut term: *const ::core::ffi::c_char,
    mut regex: ::core::ffi::c_int,
    mut ignore: ::core::ffi::c_int,
) -> u_int {
    let mut s: *mut screen = &raw mut (*wp).base;
    let mut r: regex_t = re_pattern_buffer {
        buffer: ::core::ptr::null_mut::<re_dfa_t>(),
        allocated: 0,
        used: 0,
        syntax: 0,
        fastmap: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        translate: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
        re_nsub: 0,
        can_be_null_regs_allocated_fastmap_accurate_no_sub_not_bol_not_eol_newline_anchor: [0; 1],
        c2rust_padding: [0; 7],
    };
    let mut new: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut found: ::core::ffi::c_int = 0;
    let mut n: size_t = 0;
    if regex == 0 {
        if ignore != 0 {
            flags |= FNM_CASEFOLD;
        }
        xasprintf(
            &raw mut new,
            b"*%s*\0" as *const u8 as *const ::core::ffi::c_char,
            term,
        );
    } else {
        if ignore != 0 {
            flags |= REG_ICASE;
        }
        if regcomp(&raw mut r, term, flags | REG_EXTENDED) != 0 as ::core::ffi::c_int {
            return 0 as u_int;
        }
    }
    i = 0 as u_int;
    while i < (*(*s).grid).sy {
        line = grid_view_string_cells((*s).grid, 0 as u_int, i, (*(*s).grid).sx);
        n = strlen(line);
        while n > 0 as size_t {
            if *(*__ctype_b_loc()).offset(*line.offset(n.wrapping_sub(1 as size_t) as isize)
                as u_char as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                & _ISspace as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                == 0
            {
                break;
            }
            *line.offset(n.wrapping_sub(1 as size_t) as isize) = '\0' as i32 as ::core::ffi::c_char;
            n = n.wrapping_sub(1);
        }
        log_debug(
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"window_pane_search\0" as *const u8 as *const ::core::ffi::c_char,
            line,
        );
        if regex == 0 {
            found = (fnmatch(new, line, flags) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        } else {
            found = (regexec(
                &raw mut r,
                line,
                0 as size_t,
                ::core::ptr::null_mut::<regmatch_t>(),
                0 as ::core::ffi::c_int,
            ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        }
        free(line as *mut ::core::ffi::c_void);
        if found != 0 {
            break;
        }
        i = i.wrapping_add(1);
    }
    if regex == 0 {
        free(new as *mut ::core::ffi::c_void);
    } else {
        regfree(&raw mut r);
    }
    if i == (*(*s).grid).sy {
        return 0 as u_int;
    }
    return i.wrapping_add(1 as u_int);
}
unsafe extern "C" fn window_pane_choose_best(
    mut list: *mut *mut window_pane,
    mut size: u_int,
) -> *mut window_pane {
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut best: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut i: u_int = 0;
    if size == 0 as u_int {
        return ::core::ptr::null_mut::<window_pane>();
    }
    best = *list.offset(0 as ::core::ffi::c_int as isize);
    i = 1 as u_int;
    while i < size {
        next = *list.offset(i as isize);
        if (*next).active_point > (*best).active_point {
            best = next;
        }
        i = i.wrapping_add(1);
    }
    return best;
}
unsafe extern "C" fn window_pane_full_size_offset(
    mut wp: *mut window_pane,
    mut xoff: *mut ::core::ffi::c_int,
    mut yoff: *mut ::core::ffi::c_int,
    mut sx: *mut u_int,
    mut sy: *mut u_int,
) {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut sb_w: u_int = 0;
    if window_pane_scrollbar_reserve(wp) != 0 {
        sb_w = ((*wp).scrollbar_style.width + (*wp).scrollbar_style.pad) as u_int;
    } else {
        sb_w = 0 as u_int;
    }
    if (*w).sb_pos == PANE_SCROLLBARS_LEFT {
        *xoff = ((*wp).xoff as u_int).wrapping_sub(sb_w) as ::core::ffi::c_int;
        *sx = (*wp).sx.wrapping_add(sb_w);
    } else {
        *xoff = (*wp).xoff;
        *sx = (*wp).sx.wrapping_add(sb_w);
    }
    *yoff = (*wp).yoff;
    *sy = (*wp).sy;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_find_up(mut wp: *mut window_pane) -> *mut window_pane {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut best: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut list: *mut *mut window_pane = ::core::ptr::null_mut::<*mut window_pane>();
    let mut edge: ::core::ffi::c_int = 0;
    let mut left: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    let mut status: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut size: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    w = (*wp).window as *mut window;
    status = window_get_pane_status(w);
    list = ::core::ptr::null_mut::<*mut window_pane>();
    size = 0 as u_int;
    window_pane_full_size_offset(wp, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
    edge = yoff;
    if status == PANE_STATUS_TOP {
        if edge == 1 as ::core::ffi::c_int {
            edge = (*w).sy as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
        }
    } else if status == PANE_STATUS_BOTTOM {
        if edge == 0 as ::core::ffi::c_int {
            edge = (*w).sy as ::core::ffi::c_int;
        }
    } else if edge == 0 as ::core::ffi::c_int {
        edge = (*w).sy as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    }
    left = xoff;
    right = xoff + sx as ::core::ffi::c_int;
    next = (*w).panes.tqh_first;
    while !next.is_null() {
        window_pane_full_size_offset(next, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
        if !(next == wp) {
            if !(yoff + sy as ::core::ffi::c_int + 1 as ::core::ffi::c_int != edge) {
                end = xoff + sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                found = 0 as ::core::ffi::c_int;
                if xoff < left && end > right {
                    found = 1 as ::core::ffi::c_int;
                } else if xoff >= left && xoff <= right {
                    found = 1 as ::core::ffi::c_int;
                } else if end >= left && end <= right {
                    found = 1 as ::core::ffi::c_int;
                }
                if !(found == 0) {
                    list = xreallocarray(
                        list as *mut ::core::ffi::c_void,
                        size.wrapping_add(1 as u_int) as size_t,
                        ::core::mem::size_of::<*mut window_pane>() as size_t,
                    ) as *mut *mut window_pane;
                    let fresh18 = size;
                    size = size.wrapping_add(1);
                    let ref mut fresh19 = *list.offset(fresh18 as isize);
                    *fresh19 = next;
                }
            }
        }
        next = (*next).entry.tqe_next;
    }
    best = window_pane_choose_best(list, size);
    free(list as *mut ::core::ffi::c_void);
    return best;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_find_down(mut wp: *mut window_pane) -> *mut window_pane {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut best: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut list: *mut *mut window_pane = ::core::ptr::null_mut::<*mut window_pane>();
    let mut edge: ::core::ffi::c_int = 0;
    let mut left: ::core::ffi::c_int = 0;
    let mut right: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    let mut status: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut size: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    w = (*wp).window as *mut window;
    status = window_get_pane_status(w);
    list = ::core::ptr::null_mut::<*mut window_pane>();
    size = 0 as u_int;
    window_pane_full_size_offset(wp, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
    edge = yoff + sy as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    if status == PANE_STATUS_TOP {
        if edge >= (*w).sy as ::core::ffi::c_int {
            edge = 1 as ::core::ffi::c_int;
        }
    } else if status == PANE_STATUS_BOTTOM {
        if edge >= (*w).sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
            edge = 0 as ::core::ffi::c_int;
        }
    } else if edge >= (*w).sy as ::core::ffi::c_int {
        edge = 0 as ::core::ffi::c_int;
    }
    left = (*wp).xoff;
    right = (*wp).xoff + (*wp).sx as ::core::ffi::c_int;
    next = (*w).panes.tqh_first;
    while !next.is_null() {
        window_pane_full_size_offset(next, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
        if !(next == wp) {
            if !(yoff != edge) {
                end = xoff + sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                found = 0 as ::core::ffi::c_int;
                if xoff < left && end > right {
                    found = 1 as ::core::ffi::c_int;
                } else if xoff >= left && xoff <= right {
                    found = 1 as ::core::ffi::c_int;
                } else if end >= left && end <= right {
                    found = 1 as ::core::ffi::c_int;
                }
                if !(found == 0) {
                    list = xreallocarray(
                        list as *mut ::core::ffi::c_void,
                        size.wrapping_add(1 as u_int) as size_t,
                        ::core::mem::size_of::<*mut window_pane>() as size_t,
                    ) as *mut *mut window_pane;
                    let fresh20 = size;
                    size = size.wrapping_add(1);
                    let ref mut fresh21 = *list.offset(fresh20 as isize);
                    *fresh21 = next;
                }
            }
        }
        next = (*next).entry.tqe_next;
    }
    best = window_pane_choose_best(list, size);
    free(list as *mut ::core::ffi::c_void);
    return best;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_find_left(mut wp: *mut window_pane) -> *mut window_pane {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut best: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut list: *mut *mut window_pane = ::core::ptr::null_mut::<*mut window_pane>();
    let mut edge: ::core::ffi::c_int = 0;
    let mut top: ::core::ffi::c_int = 0;
    let mut bottom: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut size: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    w = (*wp).window as *mut window;
    list = ::core::ptr::null_mut::<*mut window_pane>();
    size = 0 as u_int;
    window_pane_full_size_offset(wp, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
    edge = xoff;
    if edge == 0 as ::core::ffi::c_int {
        edge = (*w).sx as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    }
    top = yoff;
    bottom = yoff + sy as ::core::ffi::c_int;
    next = (*w).panes.tqh_first;
    while !next.is_null() {
        window_pane_full_size_offset(next, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
        if !(next == wp) {
            if !(xoff + sx as ::core::ffi::c_int + 1 as ::core::ffi::c_int != edge) {
                end = yoff + sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                found = 0 as ::core::ffi::c_int;
                if yoff < top && end > bottom {
                    found = 1 as ::core::ffi::c_int;
                } else if yoff >= top && yoff <= bottom {
                    found = 1 as ::core::ffi::c_int;
                } else if end >= top && end <= bottom {
                    found = 1 as ::core::ffi::c_int;
                }
                if !(found == 0) {
                    list = xreallocarray(
                        list as *mut ::core::ffi::c_void,
                        size.wrapping_add(1 as u_int) as size_t,
                        ::core::mem::size_of::<*mut window_pane>() as size_t,
                    ) as *mut *mut window_pane;
                    let fresh22 = size;
                    size = size.wrapping_add(1);
                    let ref mut fresh23 = *list.offset(fresh22 as isize);
                    *fresh23 = next;
                }
            }
        }
        next = (*next).entry.tqe_next;
    }
    best = window_pane_choose_best(list, size);
    free(list as *mut ::core::ffi::c_void);
    return best;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_find_right(mut wp: *mut window_pane) -> *mut window_pane {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut next: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut best: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut list: *mut *mut window_pane = ::core::ptr::null_mut::<*mut window_pane>();
    let mut edge: ::core::ffi::c_int = 0;
    let mut top: ::core::ffi::c_int = 0;
    let mut bottom: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    let mut found: ::core::ffi::c_int = 0;
    let mut xoff: ::core::ffi::c_int = 0;
    let mut yoff: ::core::ffi::c_int = 0;
    let mut size: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    w = (*wp).window as *mut window;
    list = ::core::ptr::null_mut::<*mut window_pane>();
    size = 0 as u_int;
    window_pane_full_size_offset(wp, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
    edge = xoff + sx as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    if edge >= (*w).sx as ::core::ffi::c_int {
        edge = 0 as ::core::ffi::c_int;
    }
    top = (*wp).yoff;
    bottom = (*wp).yoff + (*wp).sy as ::core::ffi::c_int;
    next = (*w).panes.tqh_first;
    while !next.is_null() {
        window_pane_full_size_offset(next, &raw mut xoff, &raw mut yoff, &raw mut sx, &raw mut sy);
        if !(next == wp) {
            if !(xoff != edge) {
                end = yoff + sy as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
                found = 0 as ::core::ffi::c_int;
                if yoff < top && end > bottom {
                    found = 1 as ::core::ffi::c_int;
                } else if yoff >= top && yoff <= bottom {
                    found = 1 as ::core::ffi::c_int;
                } else if end >= top && end <= bottom {
                    found = 1 as ::core::ffi::c_int;
                }
                if !(found == 0) {
                    list = xreallocarray(
                        list as *mut ::core::ffi::c_void,
                        size.wrapping_add(1 as u_int) as size_t,
                        ::core::mem::size_of::<*mut window_pane>() as size_t,
                    ) as *mut *mut window_pane;
                    let fresh24 = size;
                    size = size.wrapping_add(1);
                    let ref mut fresh25 = *list.offset(fresh24 as isize);
                    *fresh25 = next;
                }
            }
        }
        next = (*next).entry.tqe_next;
    }
    best = window_pane_choose_best(list, size);
    free(list as *mut ::core::ffi::c_void);
    return best;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_stack_push(
    mut stack: *mut window_panes,
    mut wp: *mut window_pane,
) {
    if !wp.is_null() {
        window_pane_stack_remove(stack, wp);
        (*wp).sentry.tqe_next = (*stack).tqh_first;
        if !(*wp).sentry.tqe_next.is_null() {
            (*(*stack).tqh_first).sentry.tqe_prev = &raw mut (*wp).sentry.tqe_next;
        } else {
            (*stack).tqh_last = &raw mut (*wp).sentry.tqe_next;
        }
        (*stack).tqh_first = wp;
        (*wp).sentry.tqe_prev = &raw mut (*stack).tqh_first;
        (*wp).flags |= PANE_VISITED;
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_stack_remove(
    mut stack: *mut window_panes,
    mut wp: *mut window_pane,
) {
    if !wp.is_null() && (*wp).flags & PANE_VISITED != 0 {
        if !(*wp).sentry.tqe_next.is_null() {
            (*(*wp).sentry.tqe_next).sentry.tqe_prev = (*wp).sentry.tqe_prev;
        } else {
            (*stack).tqh_last = (*wp).sentry.tqe_prev;
        }
        *(*wp).sentry.tqe_prev = (*wp).sentry.tqe_next;
        (*wp).flags &= !PANE_VISITED;
    }
}
#[no_mangle]
pub unsafe extern "C" fn winlink_clear_flags(mut wl: *mut winlink) {
    let mut loop_0: *mut winlink = ::core::ptr::null_mut::<winlink>();
    (*(*wl).window).flags &= !WINDOW_ALERTFLAGS;
    loop_0 = (*(*wl).window).winlinks.tqh_first;
    while !loop_0.is_null() {
        if (*loop_0).flags & WINLINK_ALERTFLAGS != 0 as ::core::ffi::c_int {
            (*loop_0).flags &= !WINLINK_ALERTFLAGS;
            server_status_session((*loop_0).session);
        }
        loop_0 = (*loop_0).wentry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn winlink_shuffle_up(
    mut s: *mut session,
    mut wl: *mut winlink,
    mut before: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut idx: ::core::ffi::c_int = 0;
    let mut last: ::core::ffi::c_int = 0;
    if wl.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    if before != 0 {
        idx = (*wl).idx;
    } else {
        idx = (*wl).idx + 1 as ::core::ffi::c_int;
    }
    last = idx;
    while last < INT_MAX {
        if winlink_find_by_index(&raw mut (*s).windows, last).is_null() {
            break;
        }
        last += 1;
    }
    if last == INT_MAX {
        return -(1 as ::core::ffi::c_int);
    }
    while last > idx {
        wl = winlink_find_by_index(&raw mut (*s).windows, last - 1 as ::core::ffi::c_int);
        winlinks_RB_REMOVE(&raw mut (*s).windows, wl);
        (*wl).idx += 1;
        winlinks_RB_INSERT(&raw mut (*s).windows, wl);
        last -= 1;
    }
    return idx;
}
unsafe extern "C" fn window_pane_input_callback(
    mut c: *mut client,
    mut path: *const ::core::ffi::c_char,
    mut error: ::core::ffi::c_int,
    mut closed: ::core::ffi::c_int,
    mut buffer: *mut evbuffer,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut cdata: *mut window_pane_input_data = data as *mut window_pane_input_data;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut buf: *mut u_char =
        evbuffer_pullup(buffer, -(1 as ::core::ffi::c_int) as ssize_t) as *mut u_char;
    let mut len: size_t = evbuffer_get_length(buffer);
    wp = window_pane_find_by_id((*cdata).wp);
    if !(*cdata).file.is_null() && (wp.is_null() || (*c).flags & CLIENT_DEAD as uint64_t != 0) {
        if wp.is_null() {
            (*c).retval = 1 as ::core::ffi::c_int;
            (*c).flags |= CLIENT_EXIT as uint64_t;
        }
        file_cancel((*cdata).file);
    } else if (*cdata).file.is_null() || closed != 0 || error != 0 as ::core::ffi::c_int {
        cmdq_continue((*cdata).item);
        server_client_unref(c);
        free(cdata as *mut ::core::ffi::c_void);
    } else {
        input_parse_buffer(wp, buf, len);
    }
    evbuffer_drain(buffer, len);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_start_input(
    mut wp: *mut window_pane,
    mut item: *mut cmdq_item,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut c: *mut client = cmdq_get_client(item);
    let mut cdata: *mut window_pane_input_data = ::core::ptr::null_mut::<window_pane_input_data>();
    if !(*wp).flags & PANE_EMPTY != 0 {
        *cause = xstrdup(b"pane is not empty\0" as *const u8 as *const ::core::ffi::c_char);
        return -(1 as ::core::ffi::c_int);
    }
    if (*c).flags & (CLIENT_DEAD | CLIENT_EXITED) as uint64_t != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if !(*c).session.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    cdata = xmalloc(::core::mem::size_of::<window_pane_input_data>() as size_t)
        as *mut window_pane_input_data;
    (*cdata).item = item;
    (*cdata).wp = (*wp).id;
    (*cdata).file = file_read(
        c,
        b"-\0" as *const u8 as *const ::core::ffi::c_char,
        Some(
            window_pane_input_callback
                as unsafe extern "C" fn(
                    *mut client,
                    *const ::core::ffi::c_char,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                    *mut evbuffer,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        cdata as *mut ::core::ffi::c_void,
    );
    (*c).references += 1;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_new_data(
    mut wp: *mut window_pane,
    mut wpo: *mut window_pane_offset,
    mut size: *mut size_t,
) -> *mut ::core::ffi::c_void {
    let mut used: size_t = (*wpo).used.wrapping_sub((*wp).base_offset);
    *size = evbuffer_get_length((*(*wp).event).input).wrapping_sub(used);
    return evbuffer_pullup((*(*wp).event).input, -(1 as ::core::ffi::c_int) as ssize_t)
        .offset(used as isize) as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_update_used_data(
    mut wp: *mut window_pane,
    mut wpo: *mut window_pane_offset,
    mut size: size_t,
) {
    let mut used: size_t = (*wpo).used.wrapping_sub((*wp).base_offset);
    if size > evbuffer_get_length((*(*wp).event).input).wrapping_sub(used) {
        size = evbuffer_get_length((*(*wp).event).input).wrapping_sub(used);
    }
    (*wpo).used = (*wpo).used.wrapping_add(size);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_default_cursor(mut wp: *mut window_pane) {
    screen_set_default_cursor((*wp).screen, (*wp).options);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_mode(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    if !(*wp).modes.tqh_first.is_null() {
        if (*(*wp).modes.tqh_first).mode == &raw const window_copy_mode {
            return 1 as ::core::ffi::c_int;
        }
        if (*(*wp).modes.tqh_first).mode == &raw const window_view_mode {
            return 2 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_show_scrollbar(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    if !(*wp).base.saved_grid.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if (*w).flags & WINDOW_ZOOMED != 0 && !(*w).active.is_null() {
        wme = (*(*w).active).modes.tqh_first;
        if !wme.is_null() && (*(*wme).mode).flags & WINDOW_MODE_HIDE_SCROLLBARS != 0 {
            return 0 as ::core::ffi::c_int;
        }
    }
    if (*w).sb == PANE_SCROLLBARS_ALWAYS
        || (*w).sb == PANE_SCROLLBARS_AUTOHIDE
        || (*w).sb == PANE_SCROLLBARS_MODAL && window_pane_mode(wp) != WINDOW_PANE_NO_MODE
    {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_scrollbar_reserve(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    if window_pane_show_scrollbar(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    return ((*(*wp).window).sb == PANE_SCROLLBARS_ALWAYS) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_scrollbar_overlay(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    if window_pane_show_scrollbar(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    return window_pane_scrollbar_auto_hide(wp);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_scrollbar_visible(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    if window_pane_show_scrollbar(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if window_pane_scrollbar_auto_hide(wp) == 0 {
        return 1 as ::core::ffi::c_int;
    }
    return (*wp).sb_auto_visible;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_scrollbar_start_timer(mut wp: *mut window_pane) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut delay: u_int = 0;
    if window_pane_scrollbar_auto_hide(wp) == 0 || (*wp).sb_auto_visible == 0 {
        return;
    }
    delay = options_get_number(
        (*(*wp).window).options,
        b"pane-scrollbars-timeout\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    tv.tv_sec = delay.wrapping_div(1000 as u_int) as __time_t;
    tv.tv_usec = (delay.wrapping_rem(1000 as u_int) as ::core::ffi::c_long
        * 1000 as ::core::ffi::c_long) as __suseconds_t;
    event_del(&raw mut (*wp).sb_auto_timer);
    event_add(&raw mut (*wp).sb_auto_timer, &raw mut tv);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_scrollbar_show(
    mut wp: *mut window_pane,
    mut start_timer: ::core::ffi::c_int,
) {
    let mut changed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if window_pane_scrollbar_auto_hide(wp) == 0 {
        return;
    }
    if window_pane_show_scrollbar(wp) == 0 {
        return;
    }
    if (*wp).sb_auto_visible == 0 {
        (*wp).sb_auto_visible = 1 as ::core::ffi::c_int;
        changed = 1 as ::core::ffi::c_int;
    }
    event_del(&raw mut (*wp).sb_auto_timer);
    if start_timer != 0 {
        window_pane_scrollbar_start_timer(wp);
    }
    if changed != 0 {
        window_pane_scrollbar_redraw_visibility(wp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_scrollbar_hide(mut wp: *mut window_pane) {
    if event_initialized(&raw mut (*wp).sb_auto_timer) != 0 {
        event_del(&raw mut (*wp).sb_auto_timer);
    }
    (*wp).sb_auto_hover = 0 as ::core::ffi::c_int;
    if (*wp).sb_auto_visible == 0 {
        return;
    }
    (*wp).sb_auto_visible = 0 as ::core::ffi::c_int;
    window_pane_scrollbar_redraw_visibility(wp);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_bg(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    let mut defaults: grid_cell = grid_cell {
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
    c = window_pane_get_bg_control_client(wp);
    if c == -(1 as ::core::ffi::c_int) {
        tty_default_colours(&raw mut defaults, wp, ::core::ptr::null_mut::<u_int>());
        if defaults.bg == 8 as ::core::ffi::c_int || defaults.bg == 9 as ::core::ffi::c_int {
            c = window_get_bg_client(wp);
        } else {
            c = defaults.bg;
        }
    }
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn window_get_bg_client(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        if !((*loop_0).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            if !((*loop_0).session.is_null() || session_has((*loop_0).session, w) == 0) {
                if !((*loop_0).tty.bg == -(1 as ::core::ffi::c_int)) {
                    return (*loop_0).tty.bg;
                }
            }
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_bg_control_client(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if (*wp).control_bg == -(1 as ::core::ffi::c_int) {
        return -(1 as ::core::ffi::c_int);
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            return (*wp).control_bg;
        }
        c = (*c).entry.tqe_next;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_fg(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        if !((*loop_0).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            if !((*loop_0).session.is_null() || session_has((*loop_0).session, w) == 0) {
                if !((*loop_0).tty.fg == -(1 as ::core::ffi::c_int)) {
                    return (*loop_0).tty.fg;
                }
            }
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_fg_control_client(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    if (*wp).control_fg == -(1 as ::core::ffi::c_int) {
        return -(1 as ::core::ffi::c_int);
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            return (*wp).control_fg;
        }
        c = (*c).entry.tqe_next;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_theme(mut wp: *mut window_pane) -> client_theme {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut found_light: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut found_dark: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if wp.is_null() {
        return THEME_UNKNOWN;
    }
    w = (*wp).window as *mut window;
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        if !((*loop_0).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            if !((*loop_0).session.is_null() || session_has((*loop_0).session, w) == 0) {
                match (*loop_0).theme as ::core::ffi::c_uint {
                    1 => {
                        found_light = 1 as ::core::ffi::c_int;
                    }
                    2 => {
                        found_dark = 1 as ::core::ffi::c_int;
                    }
                    0 | _ => {}
                }
            }
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    if found_dark != 0 && found_light == 0 {
        return THEME_DARK;
    }
    if found_light != 0 && found_dark == 0 {
        return THEME_LIGHT;
    }
    return colour_totheme(window_pane_get_bg(wp));
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_send_theme_update(mut wp: *mut window_pane) {
    let mut theme: client_theme = THEME_UNKNOWN;
    if wp.is_null() || window_pane_exited(wp) != 0 {
        return;
    }
    if !(*wp).flags & PANE_THEMECHANGED != 0 {
        return;
    }
    if !(*(*wp).screen).mode & MODE_THEME_UPDATES != 0 {
        return;
    }
    theme = window_pane_get_theme(wp);
    if theme as ::core::ffi::c_uint == (*wp).last_theme as ::core::ffi::c_uint {
        return;
    }
    (*wp).last_theme = theme;
    (*wp).flags &= !PANE_THEMECHANGED;
    match theme as ::core::ffi::c_uint {
        1 => {
            log_debug(
                b"%s: %%%u light theme\0" as *const u8 as *const ::core::ffi::c_char,
                b"window_pane_send_theme_update\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            );
            bufferevent_write(
                (*wp).event,
                b"\x1B[?997;2n\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                9 as size_t,
            );
        }
        2 => {
            log_debug(
                b"%s: %%%u dark theme\0" as *const u8 as *const ::core::ffi::c_char,
                b"window_pane_send_theme_update\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            );
            bufferevent_write(
                (*wp).event,
                b"\x1B[?997;1n\0" as *const u8 as *const ::core::ffi::c_char
                    as *const ::core::ffi::c_void,
                9 as size_t,
            );
        }
        0 => {
            log_debug(
                b"%s: %%%u unknown theme\0" as *const u8 as *const ::core::ffi::c_char,
                b"window_pane_send_theme_update\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            );
        }
        _ => {}
    };
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_status_get_range(
    mut wp: *mut window_pane,
    mut x: u_int,
    mut y: u_int,
) -> *mut style_range {
    let mut srs: *mut style_ranges = ::core::ptr::null_mut::<style_ranges>();
    let mut line: u_int = 0;
    let mut pane_status: ::core::ffi::c_int = 0;
    if wp.is_null() {
        return ::core::ptr::null_mut::<style_range>();
    }
    srs = &raw mut (*wp).border_status_line.ranges;
    pane_status = window_pane_get_pane_status(wp);
    if pane_status == PANE_STATUS_TOP {
        line = ((*wp).yoff - 1 as ::core::ffi::c_int) as u_int;
    } else if pane_status == PANE_STATUS_BOTTOM {
        line = ((*wp).yoff as u_int).wrapping_add((*wp).sy);
    }
    if pane_status == PANE_STATUS_OFF || line != y {
        return ::core::ptr::null_mut::<style_range>();
    }
    return style_ranges_get_range(
        srs,
        x.wrapping_sub((*wp).xoff as u_int).wrapping_sub(2 as u_int),
    );
}
#[no_mangle]
pub unsafe extern "C" fn window_get_pane_lines(mut w: *mut window) -> pane_lines {
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    oo = (*w).options;
    return options_get_number(
        oo,
        b"pane-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
    ) as pane_lines;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_pane_lines(mut wp: *mut window_pane) -> pane_lines {
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    if window_pane_is_floating(wp) == 0 {
        oo = (*(*wp).window).options;
    } else {
        oo = (*wp).options;
    }
    return options_get_number(
        oo,
        b"pane-border-lines\0" as *const u8 as *const ::core::ffi::c_char,
    ) as pane_lines;
}
#[no_mangle]
pub unsafe extern "C" fn window_get_pane_status(mut w: *mut window) -> ::core::ffi::c_int {
    let mut status: ::core::ffi::c_int = 0;
    status = options_get_number(
        (*w).options,
        b"pane-border-status\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if status == PANE_STATUS_TOP_FLOATING || status == PANE_STATUS_BOTTOM_FLOATING {
        return 0 as ::core::ffi::c_int;
    }
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_get_pane_status(
    mut wp: *mut window_pane,
) -> ::core::ffi::c_int {
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut status: ::core::ffi::c_int = 0;
    wme = (*wp).modes.tqh_first;
    if !wme.is_null()
        && (*(*wme).mode).flags & WINDOW_MODE_HIDE_PANE_STATUS != 0
        && (*wp).flags & PANE_ZOOMED != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    if window_pane_is_floating(wp) == 0 {
        return window_get_pane_status((*wp).window as *mut window);
    }
    if window_pane_get_pane_lines(wp) as ::core::ffi::c_uint
        == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    status = options_get_number(
        (*wp).options,
        b"pane-border-status\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if status == PANE_STATUS_TOP_FLOATING {
        return 1 as ::core::ffi::c_int;
    }
    if status == PANE_STATUS_BOTTOM_FLOATING {
        return 2 as ::core::ffi::c_int;
    }
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn window_pane_is_floating(mut wp: *mut window_pane) -> ::core::ffi::c_int {
    let mut lc: *mut layout_cell = (*wp).layout_cell as *mut layout_cell;
    if lc.is_null() || (*lc).flags & LAYOUT_CELL_FLOATING == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

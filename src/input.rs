extern "C" {
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
    pub type screen_write_citem;
    pub type spawn_editor_state;
    pub type cmds;
    pub type redraw_scene;
    pub type tty_key;
    pub type tty_code;
    pub type format_job_tree;
    pub type control_state;
    pub type cmdq_list;
    pub type event_payload;
    pub type options_entry;
    fn __b64_ntop(
        _: *const ::core::ffi::c_uchar,
        _: size_t,
        _: *mut ::core::ffi::c_char,
        _: size_t,
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
    fn strpbrk(
        __s: *const ::core::ffi::c_char,
        __accept: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strstr(
        __haystack: *const ::core::ffi::c_char,
        __needle: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strsep(
        __stringp: *mut *mut ::core::ffi::c_char,
        __delim: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn time(__timer: *mut time_t) -> time_t;
    fn event_add(ev: *mut event, timeout: *const timeval) -> ::core::ffi::c_int;
    fn event_del(_: *mut event) -> ::core::ffi::c_int;
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
    fn evbuffer_new() -> *mut evbuffer;
    fn evbuffer_free(buf: *mut evbuffer);
    fn evbuffer_get_length(buf: *const evbuffer) -> size_t;
    fn evbuffer_add(
        buf: *mut evbuffer,
        data: *const ::core::ffi::c_void,
        datlen: size_t,
    ) -> ::core::ffi::c_int;
    fn evbuffer_drain(buf: *mut evbuffer, len: size_t) -> ::core::ffi::c_int;
    fn bufferevent_write(
        bufev: *mut bufferevent,
        data: *const ::core::ffi::c_void,
        size: size_t,
    ) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn strtonum(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
        _: ::core::ffi::c_longlong,
        _: *mut *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xrealloc(_: *mut ::core::ffi::c_void, _: size_t) -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xstrndup(_: *const ::core::ffi::c_char, _: size_t) -> *mut ::core::ffi::c_char;
    fn xvasprintf(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    fn xsnprintf(
        _: *mut ::core::ffi::c_char,
        _: size_t,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    static mut global_options: *mut options;
    static mut global_w_options: *mut options;
    fn get_timer() -> uint64_t;
    fn getversion() -> *const ::core::ffi::c_char;
    fn paste_buffer_data(_: *mut paste_buffer, _: *mut size_t) -> *const ::core::ffi::c_char;
    fn paste_get_top(_: *mut *mut ::core::ffi::c_char) -> *mut paste_buffer;
    fn paste_add(_: *const ::core::ffi::c_char, _: *mut ::core::ffi::c_char, _: size_t);
    fn event_payload_create() -> *mut event_payload;
    fn event_payload_set_target(_: *mut event_payload, _: *mut cmd_find_state);
    fn event_payload_set_string(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn event_payload_set_time(_: *mut event_payload, _: *const ::core::ffi::c_char, _: time_t);
    fn event_payload_set_int(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    );
    fn event_payload_set_uint(_: *mut event_payload, _: *const ::core::ffi::c_char, _: u_int);
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
    fn events_fire_pane(_: *const ::core::ffi::c_char, _: *mut window_pane);
    fn options_get_only(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn options_set_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
    ) -> *mut options_entry;
    fn options_remove_or_default(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn tty_putcode_ss(
        _: *mut tty,
        _: tty_code_code,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
    );
    fn tty_puts(_: *mut tty, _: *const ::core::ffi::c_char);
    fn tty_set_selection(
        _: *mut tty,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: size_t,
    );
    fn tty_default_colours(_: *mut grid_cell, _: *mut window_pane, _: *mut u_int);
    fn cmd_find_from_pane(
        _: *mut cmd_find_state,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn alerts_queue(_: *mut window, _: ::core::ffi::c_int);
    static mut clients: clients;
    fn server_redraw_window_borders(_: *mut window);
    fn server_status_window(_: *mut window);
    fn colour_join_rgb(_: u_char, _: u_char, _: u_char) -> ::core::ffi::c_int;
    fn colour_split_rgb(_: ::core::ffi::c_int, _: *mut u_char, _: *mut u_char, _: *mut u_char);
    fn colour_force_rgb(_: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn colour_parseX11(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn colour_palette_clear(_: *mut colour_palette);
    fn colour_palette_get(_: *mut colour_palette, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn colour_palette_set(
        _: *mut colour_palette,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    static grid_default_cell: grid_cell;
    fn grid_set_tab(_: *mut grid_cell, _: u_int);
    fn grid_cells_look_equal(_: *const grid_cell, _: *const grid_cell) -> ::core::ffi::c_int;
    fn grid_get_cell(_: *mut grid, _: u_int, _: u_int, _: *mut grid_cell);
    fn grid_get_line(_: *mut grid, _: u_int) -> *mut grid_line;
    fn screen_write_start_pane(_: *mut screen_write_ctx, _: *mut window_pane, _: *mut screen);
    fn screen_write_start(_: *mut screen_write_ctx, _: *mut screen);
    fn screen_write_start_callback(
        _: *mut screen_write_ctx,
        _: *mut screen,
        _: screen_write_init_ctx_cb,
        _: *mut ::core::ffi::c_void,
    );
    fn screen_write_stop(_: *mut screen_write_ctx);
    fn screen_write_reset(_: *mut screen_write_ctx);
    fn screen_write_backspace(_: *mut screen_write_ctx);
    fn screen_write_mode_set(_: *mut screen_write_ctx, _: ::core::ffi::c_int);
    fn screen_write_mode_clear(_: *mut screen_write_ctx, _: ::core::ffi::c_int);
    fn screen_write_start_sync(_: *mut window_pane);
    fn screen_write_stop_sync(_: *mut window_pane);
    fn screen_write_end_sync(_: *mut screen_write_ctx);
    fn screen_write_cursorup(_: *mut screen_write_ctx, _: u_int);
    fn screen_write_cursordown(_: *mut screen_write_ctx, _: u_int);
    fn screen_write_cursorright(_: *mut screen_write_ctx, _: u_int);
    fn screen_write_cursorleft(_: *mut screen_write_ctx, _: u_int);
    fn screen_write_alignmenttest(_: *mut screen_write_ctx);
    fn screen_write_insertcharacter(_: *mut screen_write_ctx, _: u_int, _: u_int);
    fn screen_write_deletecharacter(_: *mut screen_write_ctx, _: u_int, _: u_int);
    fn screen_write_clearcharacter(_: *mut screen_write_ctx, _: u_int, _: u_int);
    fn screen_write_insertline(_: *mut screen_write_ctx, _: u_int, _: u_int);
    fn screen_write_deleteline(_: *mut screen_write_ctx, _: u_int, _: u_int);
    fn screen_write_clearline(_: *mut screen_write_ctx, _: u_int);
    fn screen_write_clearendofline(_: *mut screen_write_ctx, _: u_int);
    fn screen_write_clearstartofline(_: *mut screen_write_ctx, _: u_int);
    fn screen_write_cursormove(
        _: *mut screen_write_ctx,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn screen_write_reverseindex(_: *mut screen_write_ctx, _: u_int);
    fn screen_write_scrollregion(_: *mut screen_write_ctx, _: u_int, _: u_int);
    fn screen_write_linefeed(_: *mut screen_write_ctx, _: ::core::ffi::c_int, _: u_int);
    fn screen_write_scrollup(_: *mut screen_write_ctx, _: u_int, _: u_int);
    fn screen_write_scrolldown(_: *mut screen_write_ctx, _: u_int, _: u_int);
    fn screen_write_carriagereturn(_: *mut screen_write_ctx);
    fn screen_write_clearendofscreen(_: *mut screen_write_ctx, _: u_int);
    fn screen_write_clearstartofscreen(_: *mut screen_write_ctx, _: u_int);
    fn screen_write_clearscreen(_: *mut screen_write_ctx, _: u_int);
    fn screen_write_clearhistory(_: *mut screen_write_ctx);
    fn screen_write_fullredraw(_: *mut screen_write_ctx);
    fn screen_write_collect_end(_: *mut screen_write_ctx);
    fn screen_write_collect_add(_: *mut screen_write_ctx, _: *const grid_cell);
    fn screen_write_setselection(
        _: *mut screen_write_ctx,
        _: *const ::core::ffi::c_char,
        _: *mut u_char,
        _: u_int,
    );
    fn screen_write_rawstring(
        _: *mut screen_write_ctx,
        _: *mut u_char,
        _: u_int,
        _: ::core::ffi::c_int,
    );
    fn screen_write_alternateon(_: *mut screen_write_ctx, _: *mut grid_cell, _: ::core::ffi::c_int);
    fn screen_write_alternateoff(
        _: *mut screen_write_ctx,
        _: *mut grid_cell,
        _: ::core::ffi::c_int,
    );
    fn screen_set_cursor_style(_: u_int, _: *mut screen_cursor_style, _: *mut ::core::ffi::c_int);
    fn screen_set_cursor_colour(_: *mut screen, _: ::core::ffi::c_int);
    fn screen_set_title(
        _: *mut screen,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn screen_set_path(
        _: *mut screen,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn screen_push_title(_: *mut screen);
    fn screen_pop_title(_: *mut screen);
    fn screen_set_progress_bar(_: *mut screen, _: progress_bar_state, _: ::core::ffi::c_int);
    fn window_update_activity(_: *mut window);
    fn window_set_name(_: *mut window, _: *const ::core::ffi::c_char, _: ::core::ffi::c_int);
    fn window_pane_get_new_data(
        _: *mut window_pane,
        _: *mut window_pane_offset,
        _: *mut size_t,
    ) -> *mut ::core::ffi::c_void;
    fn window_pane_update_used_data(_: *mut window_pane, _: *mut window_pane_offset, _: size_t);
    fn window_pane_get_bg(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_get_fg(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_get_fg_control_client(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_get_theme(_: *mut window_pane) -> client_theme;
    fn session_has(_: *mut session, _: *mut window) -> ::core::ffi::c_int;
    fn utf8_set(_: *mut utf8_data, _: u_char);
    fn utf8_copy(_: *mut utf8_data, _: *const utf8_data);
    fn utf8_open(_: *mut utf8_data, _: u_char) -> utf8_state;
    fn utf8_append(_: *mut utf8_data, _: u_char) -> utf8_state;
    fn utf8_isvalid(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatalx(_: *const ::core::ffi::c_char, ...) -> !;
    fn hyperlinks_put(
        _: *mut hyperlinks,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
    ) -> u_int;
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
pub type __uint64_t = u64;
pub type __pid_t = ::core::ffi::c_int;
pub type __time_t = ::core::ffi::c_long;
pub type __suseconds_t = ::core::ffi::c_long;
pub type u_char = __u_char;
pub type u_short = __u_short;
pub type u_int = __u_int;
pub type pid_t = __pid_t;
pub type time_t = __time_t;
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct timeval {
    pub tv_sec: __time_t,
    pub tv_usec: __suseconds_t,
}
pub type uint8_t = __uint8_t;
pub type uint64_t = __uint64_t;
pub type __gnuc_va_list = __builtin_va_list;
pub type va_list = __gnuc_va_list;
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
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
    pub exit_type: C2RustUnnamed_40,
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
    pub c2rust_unnamed: C2RustUnnamed_12,
    pub flags: u_char,
}
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
    pub entry: C2RustUnnamed_29,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_29 {
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
pub struct input_ctx {
    pub wp: *mut window_pane,
    pub event: *mut bufferevent,
    pub ctx: screen_write_ctx,
    pub palette: *mut colour_palette,
    pub c: *mut client,
    pub cell: input_cell,
    pub old_cell: input_cell,
    pub old_cx: u_int,
    pub old_cy: u_int,
    pub old_mode: ::core::ffi::c_int,
    pub interm_buf: [u_char; 4],
    pub interm_len: size_t,
    pub param_buf: [u_char; 64],
    pub param_len: size_t,
    pub input_buf: *mut u_char,
    pub input_len: size_t,
    pub input_space: size_t,
    pub input_end: input_end_type,
    pub param_list: [input_param; 24],
    pub param_list_len: u_int,
    pub utf8data: utf8_data,
    pub utf8started: ::core::ffi::c_int,
    pub ch: ::core::ffi::c_int,
    pub last: utf8_data,
    pub state: *const input_state,
    pub flags: ::core::ffi::c_int,
    pub requests: input_requests,
    pub request_count: u_int,
    pub request_timer: event,
    pub since_ground: *mut evbuffer,
    pub ground_timer: event,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_requests {
    pub tqh_first: *mut input_request,
    pub tqh_last: *mut *mut input_request,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_request {
    pub c: *mut client,
    pub ictx: *mut input_ctx,
    pub type_0: input_request_type,
    pub t: uint64_t,
    pub end: input_end_type,
    pub idx: ::core::ffi::c_int,
    pub data: *mut ::core::ffi::c_void,
    pub entry: C2RustUnnamed_31,
    pub centry: C2RustUnnamed_30,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_30 {
    pub tqe_next: *mut input_request,
    pub tqe_prev: *mut *mut input_request,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_31 {
    pub tqe_next: *mut input_request,
    pub tqe_prev: *mut *mut input_request,
}
pub type input_end_type = ::core::ffi::c_uint;
pub const INPUT_END_BEL: input_end_type = 1;
pub const INPUT_END_ST: input_end_type = 0;
pub type input_request_type = ::core::ffi::c_uint;
pub const INPUT_REQUEST_QUEUE: input_request_type = 2;
pub const INPUT_REQUEST_CLIPBOARD: input_request_type = 1;
pub const INPUT_REQUEST_PALETTE: input_request_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_state {
    pub name: *const ::core::ffi::c_char,
    pub enter: Option<unsafe extern "C" fn(*mut input_ctx) -> ()>,
    pub exit: Option<unsafe extern "C" fn(*mut input_ctx) -> ()>,
    pub transitions: *const input_transition,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_transition {
    pub first: ::core::ffi::c_int,
    pub last: ::core::ffi::c_int,
    pub handler: Option<unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int>,
    pub state: *const input_state,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_param {
    pub type_0: C2RustUnnamed_33,
    pub c2rust_unnamed: C2RustUnnamed_32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_32 {
    pub num: ::core::ffi::c_int,
    pub str_0: *mut ::core::ffi::c_char,
}
pub type C2RustUnnamed_33 = ::core::ffi::c_uint;
pub const INPUT_STRING: C2RustUnnamed_33 = 2;
pub const INPUT_NUMBER: C2RustUnnamed_33 = 1;
pub const INPUT_MISSING: C2RustUnnamed_33 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_cell {
    pub cell: grid_cell,
    pub set: ::core::ffi::c_int,
    pub g0set: ::core::ffi::c_int,
    pub g1set: ::core::ffi::c_int,
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
    pub c2rust_unnamed: C2RustUnnamed_34,
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
pub union C2RustUnnamed_34 {
    pub n: u_int,
    pub data: C2RustUnnamed_36,
    pub sel: C2RustUnnamed_35,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_35 {
    pub clip: *const ::core::ffi::c_char,
    pub data: *const ::core::ffi::c_char,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_36 {
    pub data: *const ::core::ffi::c_char,
    pub size: size_t,
}
pub type tty_ctx_set_client_cb =
    Option<unsafe extern "C" fn(*mut tty_ctx, *mut client) -> ::core::ffi::c_int>;
pub type tty_ctx_redraw_cb = Option<unsafe extern "C" fn(*const tty_ctx) -> ()>;
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
    pub entry: C2RustUnnamed_37,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
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
    pub entry: C2RustUnnamed_38,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_38 {
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
    pub entry: C2RustUnnamed_39,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_39 {
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
pub type C2RustUnnamed_40 = ::core::ffi::c_uint;
pub const CLIENT_EXIT_DETACH: C2RustUnnamed_40 = 2;
pub const CLIENT_EXIT_SHUTDOWN: C2RustUnnamed_40 = 1;
pub const CLIENT_EXIT_RETURN: C2RustUnnamed_40 = 0;
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
    pub entry: C2RustUnnamed_41,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_41 {
    pub le_next: *mut tty_term,
    pub le_prev: *mut *mut tty_term,
}
pub type tty_code_code = ::core::ffi::c_uint;
pub const TTYC_XT: tty_code_code = 233;
pub const TTYC_VPA: tty_code_code = 232;
pub const TTYC_U8: tty_code_code = 231;
pub const TTYC_TSL: tty_code_code = 230;
pub const TTYC_TC: tty_code_code = 229;
pub const TTYC_SYNC: tty_code_code = 228;
pub const TTYC_SWD: tty_code_code = 227;
pub const TTYC_SS: tty_code_code = 226;
pub const TTYC_SXL: tty_code_code = 225;
pub const TTYC_SPB: tty_code_code = 224;
pub const TTYC_SMXX: tty_code_code = 223;
pub const TTYC_SMULX: tty_code_code = 222;
pub const TTYC_SMUL: tty_code_code = 221;
pub const TTYC_SMSO: tty_code_code = 220;
pub const TTYC_SMOL: tty_code_code = 219;
pub const TTYC_SMKX: tty_code_code = 218;
pub const TTYC_SMCUP: tty_code_code = 217;
pub const TTYC_SMACS: tty_code_code = 216;
pub const TTYC_SITM: tty_code_code = 215;
pub const TTYC_SGR0: tty_code_code = 214;
pub const TTYC_SETULC1: tty_code_code = 213;
pub const TTYC_SETULC: tty_code_code = 212;
pub const TTYC_SETRGBF: tty_code_code = 211;
pub const TTYC_SETRGBB: tty_code_code = 210;
pub const TTYC_SETAL: tty_code_code = 209;
pub const TTYC_SETAF: tty_code_code = 208;
pub const TTYC_SETAB: tty_code_code = 207;
pub const TTYC_SE: tty_code_code = 206;
pub const TTYC_RMKX: tty_code_code = 205;
pub const TTYC_RMCUP: tty_code_code = 204;
pub const TTYC_RMACS: tty_code_code = 203;
pub const TTYC_RIN: tty_code_code = 202;
pub const TTYC_RI: tty_code_code = 201;
pub const TTYC_RGB: tty_code_code = 200;
pub const TTYC_REV: tty_code_code = 199;
pub const TTYC_RECT: tty_code_code = 198;
pub const TTYC_OP: tty_code_code = 197;
pub const TTYC_OL: tty_code_code = 196;
pub const TTYC_NOBR: tty_code_code = 195;
pub const TTYC_MS: tty_code_code = 194;
pub const TTYC_KUP7: tty_code_code = 193;
pub const TTYC_KUP6: tty_code_code = 192;
pub const TTYC_KUP5: tty_code_code = 191;
pub const TTYC_KUP4: tty_code_code = 190;
pub const TTYC_KUP3: tty_code_code = 189;
pub const TTYC_KUP2: tty_code_code = 188;
pub const TTYC_KRIT7: tty_code_code = 187;
pub const TTYC_KRIT6: tty_code_code = 186;
pub const TTYC_KRIT5: tty_code_code = 185;
pub const TTYC_KRIT4: tty_code_code = 184;
pub const TTYC_KRIT3: tty_code_code = 183;
pub const TTYC_KRIT2: tty_code_code = 182;
pub const TTYC_KRI: tty_code_code = 181;
pub const TTYC_KPRV7: tty_code_code = 180;
pub const TTYC_KPRV6: tty_code_code = 179;
pub const TTYC_KPRV5: tty_code_code = 178;
pub const TTYC_KPRV4: tty_code_code = 177;
pub const TTYC_KPRV3: tty_code_code = 176;
pub const TTYC_KPRV2: tty_code_code = 175;
pub const TTYC_KPP: tty_code_code = 174;
pub const TTYC_KNXT7: tty_code_code = 173;
pub const TTYC_KNXT6: tty_code_code = 172;
pub const TTYC_KNXT5: tty_code_code = 171;
pub const TTYC_KNXT4: tty_code_code = 170;
pub const TTYC_KNXT3: tty_code_code = 169;
pub const TTYC_KNXT2: tty_code_code = 168;
pub const TTYC_KNP: tty_code_code = 167;
pub const TTYC_KMOUS: tty_code_code = 166;
pub const TTYC_KLFT7: tty_code_code = 165;
pub const TTYC_KLFT6: tty_code_code = 164;
pub const TTYC_KLFT5: tty_code_code = 163;
pub const TTYC_KLFT4: tty_code_code = 162;
pub const TTYC_KLFT3: tty_code_code = 161;
pub const TTYC_KLFT2: tty_code_code = 160;
pub const TTYC_KIND: tty_code_code = 159;
pub const TTYC_KICH1: tty_code_code = 158;
pub const TTYC_KIC7: tty_code_code = 157;
pub const TTYC_KIC6: tty_code_code = 156;
pub const TTYC_KIC5: tty_code_code = 155;
pub const TTYC_KIC4: tty_code_code = 154;
pub const TTYC_KIC3: tty_code_code = 153;
pub const TTYC_KIC2: tty_code_code = 152;
pub const TTYC_KHOME: tty_code_code = 151;
pub const TTYC_KHOM7: tty_code_code = 150;
pub const TTYC_KHOM6: tty_code_code = 149;
pub const TTYC_KHOM5: tty_code_code = 148;
pub const TTYC_KHOM4: tty_code_code = 147;
pub const TTYC_KHOM3: tty_code_code = 146;
pub const TTYC_KHOM2: tty_code_code = 145;
pub const TTYC_KF9: tty_code_code = 144;
pub const TTYC_KF8: tty_code_code = 143;
pub const TTYC_KF7: tty_code_code = 142;
pub const TTYC_KF63: tty_code_code = 141;
pub const TTYC_KF62: tty_code_code = 140;
pub const TTYC_KF61: tty_code_code = 139;
pub const TTYC_KF60: tty_code_code = 138;
pub const TTYC_KF6: tty_code_code = 137;
pub const TTYC_KF59: tty_code_code = 136;
pub const TTYC_KF58: tty_code_code = 135;
pub const TTYC_KF57: tty_code_code = 134;
pub const TTYC_KF56: tty_code_code = 133;
pub const TTYC_KF55: tty_code_code = 132;
pub const TTYC_KF54: tty_code_code = 131;
pub const TTYC_KF53: tty_code_code = 130;
pub const TTYC_KF52: tty_code_code = 129;
pub const TTYC_KF51: tty_code_code = 128;
pub const TTYC_KF50: tty_code_code = 127;
pub const TTYC_KF5: tty_code_code = 126;
pub const TTYC_KF49: tty_code_code = 125;
pub const TTYC_KF48: tty_code_code = 124;
pub const TTYC_KF47: tty_code_code = 123;
pub const TTYC_KF46: tty_code_code = 122;
pub const TTYC_KF45: tty_code_code = 121;
pub const TTYC_KF44: tty_code_code = 120;
pub const TTYC_KF43: tty_code_code = 119;
pub const TTYC_KF42: tty_code_code = 118;
pub const TTYC_KF41: tty_code_code = 117;
pub const TTYC_KF40: tty_code_code = 116;
pub const TTYC_KF4: tty_code_code = 115;
pub const TTYC_KF39: tty_code_code = 114;
pub const TTYC_KF38: tty_code_code = 113;
pub const TTYC_KF37: tty_code_code = 112;
pub const TTYC_KF36: tty_code_code = 111;
pub const TTYC_KF35: tty_code_code = 110;
pub const TTYC_KF34: tty_code_code = 109;
pub const TTYC_KF33: tty_code_code = 108;
pub const TTYC_KF32: tty_code_code = 107;
pub const TTYC_KF31: tty_code_code = 106;
pub const TTYC_KF30: tty_code_code = 105;
pub const TTYC_KF3: tty_code_code = 104;
pub const TTYC_KF29: tty_code_code = 103;
pub const TTYC_KF28: tty_code_code = 102;
pub const TTYC_KF27: tty_code_code = 101;
pub const TTYC_KF26: tty_code_code = 100;
pub const TTYC_KF25: tty_code_code = 99;
pub const TTYC_KF24: tty_code_code = 98;
pub const TTYC_KF23: tty_code_code = 97;
pub const TTYC_KF22: tty_code_code = 96;
pub const TTYC_KF21: tty_code_code = 95;
pub const TTYC_KF20: tty_code_code = 94;
pub const TTYC_KF2: tty_code_code = 93;
pub const TTYC_KF19: tty_code_code = 92;
pub const TTYC_KF18: tty_code_code = 91;
pub const TTYC_KF17: tty_code_code = 90;
pub const TTYC_KF16: tty_code_code = 89;
pub const TTYC_KF15: tty_code_code = 88;
pub const TTYC_KF14: tty_code_code = 87;
pub const TTYC_KF13: tty_code_code = 86;
pub const TTYC_KF12: tty_code_code = 85;
pub const TTYC_KF11: tty_code_code = 84;
pub const TTYC_KF10: tty_code_code = 83;
pub const TTYC_KF1: tty_code_code = 82;
pub const TTYC_KEND7: tty_code_code = 81;
pub const TTYC_KEND6: tty_code_code = 80;
pub const TTYC_KEND5: tty_code_code = 79;
pub const TTYC_KEND4: tty_code_code = 78;
pub const TTYC_KEND3: tty_code_code = 77;
pub const TTYC_KEND2: tty_code_code = 76;
pub const TTYC_KEND: tty_code_code = 75;
pub const TTYC_KDN7: tty_code_code = 74;
pub const TTYC_KDN6: tty_code_code = 73;
pub const TTYC_KDN5: tty_code_code = 72;
pub const TTYC_KDN4: tty_code_code = 71;
pub const TTYC_KDN3: tty_code_code = 70;
pub const TTYC_KDN2: tty_code_code = 69;
pub const TTYC_KDCH1: tty_code_code = 68;
pub const TTYC_KDC7: tty_code_code = 67;
pub const TTYC_KDC6: tty_code_code = 66;
pub const TTYC_KDC5: tty_code_code = 65;
pub const TTYC_KDC4: tty_code_code = 64;
pub const TTYC_KDC3: tty_code_code = 63;
pub const TTYC_KDC2: tty_code_code = 62;
pub const TTYC_KCUU1: tty_code_code = 61;
pub const TTYC_KCUF1: tty_code_code = 60;
pub const TTYC_KCUD1: tty_code_code = 59;
pub const TTYC_KCUB1: tty_code_code = 58;
pub const TTYC_KCBT: tty_code_code = 57;
pub const TTYC_INVIS: tty_code_code = 56;
pub const TTYC_INDN: tty_code_code = 55;
pub const TTYC_IND: tty_code_code = 54;
pub const TTYC_IL1: tty_code_code = 53;
pub const TTYC_IL: tty_code_code = 52;
pub const TTYC_ICH1: tty_code_code = 51;
pub const TTYC_ICH: tty_code_code = 50;
pub const TTYC_HPA: tty_code_code = 49;
pub const TTYC_HOME: tty_code_code = 48;
pub const TTYC_HLS: tty_code_code = 47;
pub const TTYC_FSL: tty_code_code = 46;
pub const TTYC_ENMG: tty_code_code = 45;
pub const TTYC_ENFCS: tty_code_code = 44;
pub const TTYC_ENEKS: tty_code_code = 43;
pub const TTYC_ENBP: tty_code_code = 42;
pub const TTYC_ENACS: tty_code_code = 41;
pub const TTYC_EL1: tty_code_code = 40;
pub const TTYC_EL: tty_code_code = 39;
pub const TTYC_ED: tty_code_code = 38;
pub const TTYC_ECH: tty_code_code = 37;
pub const TTYC_E3: tty_code_code = 36;
pub const TTYC_DSMG: tty_code_code = 35;
pub const TTYC_DSFCS: tty_code_code = 34;
pub const TTYC_DSEKS: tty_code_code = 33;
pub const TTYC_DSBP: tty_code_code = 32;
pub const TTYC_DL1: tty_code_code = 31;
pub const TTYC_DL: tty_code_code = 30;
pub const TTYC_DIM: tty_code_code = 29;
pub const TTYC_DCH1: tty_code_code = 28;
pub const TTYC_DCH: tty_code_code = 27;
pub const TTYC_CVVIS: tty_code_code = 26;
pub const TTYC_CUU1: tty_code_code = 25;
pub const TTYC_CUU: tty_code_code = 24;
pub const TTYC_CUP: tty_code_code = 23;
pub const TTYC_CUF1: tty_code_code = 22;
pub const TTYC_CUF: tty_code_code = 21;
pub const TTYC_CUD1: tty_code_code = 20;
pub const TTYC_CUD: tty_code_code = 19;
pub const TTYC_CUB1: tty_code_code = 18;
pub const TTYC_CUB: tty_code_code = 17;
pub const TTYC_CSR: tty_code_code = 16;
pub const TTYC_CS: tty_code_code = 15;
pub const TTYC_CR: tty_code_code = 14;
pub const TTYC_COLORS: tty_code_code = 13;
pub const TTYC_CNORM: tty_code_code = 12;
pub const TTYC_CMG: tty_code_code = 11;
pub const TTYC_CLMG: tty_code_code = 10;
pub const TTYC_CLEAR: tty_code_code = 9;
pub const TTYC_CIVIS: tty_code_code = 8;
pub const TTYC_BOLD: tty_code_code = 7;
pub const TTYC_BLINK: tty_code_code = 6;
pub const TTYC_BIDI: tty_code_code = 5;
pub const TTYC_BEL: tty_code_code = 4;
pub const TTYC_BCE: tty_code_code = 3;
pub const TTYC_AX: tty_code_code = 2;
pub const TTYC_AM: tty_code_code = 1;
pub const TTYC_ACSC: tty_code_code = 0;
pub type utf8_state = ::core::ffi::c_uint;
pub const UTF8_ERROR: utf8_state = 2;
pub const UTF8_DONE: utf8_state = 1;
pub const UTF8_MORE: utf8_state = 0;
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
pub struct clients {
    pub tqh_first: *mut client,
    pub tqh_last: *mut *mut client,
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
    pub name_entry: C2RustUnnamed_43,
    pub time_entry: C2RustUnnamed_42,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_42 {
    pub rbe_left: *mut paste_buffer,
    pub rbe_right: *mut paste_buffer,
    pub rbe_parent: *mut paste_buffer,
    pub rbe_color: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_43 {
    pub rbe_left: *mut paste_buffer,
    pub rbe_right: *mut paste_buffer,
    pub rbe_parent: *mut paste_buffer,
    pub rbe_color: ::core::ffi::c_int,
}
pub const INPUT_ESC_ST: input_esc_type = 14;
pub const INPUT_ESC_SCSG1_OFF: input_esc_type = 12;
pub const INPUT_ESC_SCSG1_ON: input_esc_type = 13;
pub const INPUT_ESC_SCSG0_OFF: input_esc_type = 10;
pub const INPUT_ESC_SCSG0_ON: input_esc_type = 11;
pub const INPUT_ESC_DECALN: input_esc_type = 0;
pub const INPUT_ESC_DECRC: input_esc_type = 3;
pub const INPUT_ESC_DECSC: input_esc_type = 4;
pub const INPUT_ESC_DECKPNM: input_esc_type = 2;
pub const INPUT_ESC_DECKPAM: input_esc_type = 1;
pub const INPUT_ESC_RI: input_esc_type = 8;
pub const INPUT_ESC_HTS: input_esc_type = 5;
pub const INPUT_ESC_NEL: input_esc_type = 7;
pub const INPUT_ESC_IND: input_esc_type = 6;
pub const INPUT_ESC_RIS: input_esc_type = 9;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct input_table_entry {
    pub ch: ::core::ffi::c_int,
    pub interm: *const ::core::ffi::c_char,
    pub type_0: ::core::ffi::c_int,
}
pub const INPUT_CSI_XDA: input_csi_type = 40;
pub const INPUT_CSI_DECSCUSR: input_csi_type = 11;
pub const INPUT_CSI_VPA: input_csi_type = 38;
pub const INPUT_CSI_TBC: input_csi_type = 37;
pub const INPUT_CSI_SD: input_csi_type = 31;
pub const INPUT_CSI_SU: input_csi_type = 36;
pub const INPUT_CSI_SM_GRAPHICS: input_csi_type = 34;
pub const INPUT_CSI_SM_PRIVATE: input_csi_type = 35;
pub const INPUT_CSI_SM: input_csi_type = 33;
pub const INPUT_CSI_SGR: input_csi_type = 32;
pub const INPUT_CSI_SCP: input_csi_type = 30;
pub const INPUT_CSI_RM_PRIVATE: input_csi_type = 29;
pub const INPUT_CSI_RM: input_csi_type = 28;
pub const INPUT_CSI_RCP: input_csi_type = 26;
pub const INPUT_CSI_REP: input_csi_type = 27;
pub const INPUT_CSI_IL: input_csi_type = 21;
pub const INPUT_CSI_ICH: input_csi_type = 20;
pub const INPUT_CSI_HPA: input_csi_type = 19;
pub const INPUT_CSI_EL: input_csi_type = 18;
pub const INPUT_CSI_ED: input_csi_type = 17;
pub const INPUT_CSI_DSR: input_csi_type = 14;
pub const INPUT_CSI_QUERY_PRIVATE: input_csi_type = 25;
pub const INPUT_CSI_QUERY: input_csi_type = 24;
pub const INPUT_CSI_DSR_PRIVATE: input_csi_type = 15;
pub const INPUT_CSI_DL: input_csi_type = 13;
pub const INPUT_CSI_DECSTBM: input_csi_type = 12;
pub const INPUT_CSI_DCH: input_csi_type = 10;
pub const INPUT_CSI_ECH: input_csi_type = 16;
pub const INPUT_CSI_DA_TWO: input_csi_type = 9;
pub const INPUT_CSI_DA: input_csi_type = 8;
pub const INPUT_CSI_CPL: input_csi_type = 2;
pub const INPUT_CSI_CNL: input_csi_type = 1;
pub const INPUT_CSI_CUU: input_csi_type = 7;
pub const INPUT_CSI_WINOPS: input_csi_type = 39;
pub const INPUT_CSI_MODOFF: input_csi_type = 22;
pub const INPUT_CSI_MODSET: input_csi_type = 23;
pub const INPUT_CSI_CUP: input_csi_type = 6;
pub const INPUT_CSI_CUF: input_csi_type = 5;
pub const INPUT_CSI_CUD: input_csi_type = 4;
pub const INPUT_CSI_CUB: input_csi_type = 3;
pub const INPUT_CSI_CBT: input_csi_type = 0;
pub type input_esc_type = ::core::ffi::c_uint;
pub type input_csi_type = ::core::ffi::c_uint;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[inline]
unsafe extern "C" fn bsearch(
    mut __key: *const ::core::ffi::c_void,
    mut __base: *const ::core::ffi::c_void,
    mut __nmemb: size_t,
    mut __size: size_t,
    mut __compar: __compar_fn_t,
) -> *mut ::core::ffi::c_void {
    let mut __p: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
    let mut __comparison: ::core::ffi::c_int = 0;
    while __nmemb != 0 {
        __p = (__base as *const ::core::ffi::c_char)
            .offset((__nmemb >> 1 as ::core::ffi::c_int).wrapping_mul(__size) as isize)
            as *const ::core::ffi::c_void;
        __comparison = Some(__compar.expect("non-null function pointer"))
            .expect("non-null function pointer")(__key, __p);
        if __comparison == 0 as ::core::ffi::c_int {
            return __p as *mut ::core::ffi::c_void;
        }
        if __comparison > 0 as ::core::ffi::c_int {
            __base = (__p as *const ::core::ffi::c_char).offset(__size as isize)
                as *const ::core::ffi::c_void;
            __nmemb = __nmemb.wrapping_sub(1);
        }
        __nmemb >>= 1 as ::core::ffi::c_int;
    }
    return NULL;
}
pub const MODE_CURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MODE_INSERT: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const MODE_KCURSOR: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const MODE_KKEYPAD: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const MODE_WRAP: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const MODE_MOUSE_STANDARD: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const MODE_MOUSE_BUTTON: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const MODE_CURSOR_BLINKING: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const MODE_MOUSE_UTF8: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const MODE_MOUSE_SGR: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const MODE_BRACKETPASTE: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const MODE_FOCUSON: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const MODE_MOUSE_ALL: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const MODE_ORIGIN: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const MODE_CRLF: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const MODE_KEYS_EXTENDED: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
pub const MODE_CURSOR_VERY_VISIBLE: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const MODE_CURSOR_BLINKING_SET: ::core::ffi::c_int = 0x20000 as ::core::ffi::c_int;
pub const MODE_KEYS_EXTENDED_2: ::core::ffi::c_int = 0x40000 as ::core::ffi::c_int;
pub const MODE_THEME_UPDATES: ::core::ffi::c_int = 0x80000 as ::core::ffi::c_int;
pub const MODE_SYNC: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;
pub const ALL_MOUSE_MODES: ::core::ffi::c_int =
    MODE_MOUSE_STANDARD | MODE_MOUSE_BUTTON | MODE_MOUSE_ALL;
pub const EXTENDED_KEY_MODES: ::core::ffi::c_int = MODE_KEYS_EXTENDED | MODE_KEYS_EXTENDED_2;
pub const COLOUR_FLAG_256: ::core::ffi::c_int = 0x1000000 as ::core::ffi::c_int;
pub const GRID_ATTR_BRIGHT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const GRID_ATTR_DIM: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const GRID_ATTR_BLINK: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const GRID_ATTR_REVERSE: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const GRID_ATTR_HIDDEN: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const GRID_ATTR_ITALICS: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const GRID_ATTR_CHARSET: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const GRID_ATTR_STRIKETHROUGH: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_2: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_3: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_4: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const GRID_ATTR_UNDERSCORE_5: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const GRID_ATTR_OVERLINE: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const GRID_ATTR_ALL_UNDERSCORE: ::core::ffi::c_int = GRID_ATTR_UNDERSCORE
    | GRID_ATTR_UNDERSCORE_2
    | GRID_ATTR_UNDERSCORE_3
    | GRID_ATTR_UNDERSCORE_4
    | GRID_ATTR_UNDERSCORE_5;
pub const GRID_LINE_START_PROMPT: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const GRID_LINE_SECOND_PROMPT: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const GRID_LINE_START_COMMAND: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const GRID_LINE_START_OUTPUT: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const GRID_LINE_END_OUTPUT: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const PANE_CHANGED: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const PANE_STYLECHANGED: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const PANE_THEMECHANGED: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const PANE_UNSEENCHANGES: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const PANE_CMDRUNNING: ::core::ffi::c_int = 0x20000 as ::core::ffi::c_int;
pub const PANE_ACTIVITY: ::core::ffi::c_int = 0x40000 as ::core::ffi::c_int;
pub const WINDOW_BELL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const TTY_STARTED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CLIENT_EXIT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CLIENT_SUSPENDED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLIENT_DEAD: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const CLIENT_UNATTACHEDFLAGS: ::core::ffi::c_int = CLIENT_DEAD | CLIENT_SUSPENDED | CLIENT_EXIT;
pub const INPUT_BUF_DEFAULT_SIZE: ::core::ffi::c_int = 1048576 as ::core::ffi::c_int;
pub const INPUT_REQUEST_TIMEOUT: ::core::ffi::c_int = 500 as ::core::ffi::c_int;
pub const INPUT_BUF_START: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const INPUT_DISCARD: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const INPUT_LAST: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
static mut input_esc_table: [input_table_entry; 15] = [
    input_table_entry {
        ch: '0' as i32,
        interm: b"(\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_ESC_SCSG0_ON as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: '0' as i32,
        interm: b")\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_ESC_SCSG1_ON as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: '7' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_ESC_DECSC as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: '8' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_ESC_DECRC as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: '8' as i32,
        interm: b"#\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_ESC_DECALN as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: '=' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_ESC_DECKPAM as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: '>' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_ESC_DECKPNM as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'B' as i32,
        interm: b"(\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_ESC_SCSG0_OFF as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'B' as i32,
        interm: b")\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_ESC_SCSG1_OFF as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'D' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_ESC_IND as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'E' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_ESC_NEL as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'H' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_ESC_HTS as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'M' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_ESC_RI as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: '\\' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_ESC_ST as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'c' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_ESC_RIS as ::core::ffi::c_int,
    },
];
static mut input_csi_table: [input_table_entry; 43] = [
    input_table_entry {
        ch: '@' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_ICH as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'A' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_CUU as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'B' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_CUD as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'C' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_CUF as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'D' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_CUB as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'E' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_CNL as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'F' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_CPL as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'G' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_HPA as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'H' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_CUP as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'J' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_ED as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'K' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_EL as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'L' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_IL as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'M' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_DL as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'P' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_DCH as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'S' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_SU as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'S' as i32,
        interm: b"?\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_SM_GRAPHICS as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'T' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_SD as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'X' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_ECH as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'Z' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_CBT as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: '`' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_HPA as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'b' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_REP as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'c' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_DA as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'c' as i32,
        interm: b">\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_DA_TWO as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'd' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_VPA as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'f' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_CUP as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'g' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_TBC as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'h' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_SM as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'h' as i32,
        interm: b"?\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_SM_PRIVATE as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'l' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_RM as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'l' as i32,
        interm: b"?\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_RM_PRIVATE as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'm' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_SGR as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'm' as i32,
        interm: b">\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_MODSET as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'n' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_DSR as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'n' as i32,
        interm: b">\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_MODOFF as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'n' as i32,
        interm: b"?\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_DSR_PRIVATE as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'p' as i32,
        interm: b"$\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_QUERY as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'p' as i32,
        interm: b"?$\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_QUERY_PRIVATE as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'q' as i32,
        interm: b" \0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_DECSCUSR as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'q' as i32,
        interm: b">\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_XDA as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'r' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_DECSTBM as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 's' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_SCP as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 't' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_WINOPS as ::core::ffi::c_int,
    },
    input_table_entry {
        ch: 'u' as i32,
        interm: b"\0" as *const u8 as *const ::core::ffi::c_char,
        type_0: INPUT_CSI_RCP as ::core::ffi::c_int,
    },
];
static mut input_state_ground: input_state = unsafe {
    input_state {
        name: b"ground\0" as *const u8 as *const ::core::ffi::c_char,
        enter: Some(input_ground as unsafe extern "C" fn(*mut input_ctx) -> ()),
        exit: None,
        transitions: &raw const input_state_ground_table as *const input_transition,
    }
};
static mut input_state_esc_enter: input_state = unsafe {
    input_state {
        name: b"esc_enter\0" as *const u8 as *const ::core::ffi::c_char,
        enter: Some(input_clear as unsafe extern "C" fn(*mut input_ctx) -> ()),
        exit: None,
        transitions: &raw const input_state_esc_enter_table as *const input_transition,
    }
};
static mut input_state_esc_intermediate: input_state = unsafe {
    input_state {
        name: b"esc_intermediate\0" as *const u8 as *const ::core::ffi::c_char,
        enter: None,
        exit: None,
        transitions: &raw const input_state_esc_intermediate_table as *const input_transition,
    }
};
static mut input_state_csi_enter: input_state = unsafe {
    input_state {
        name: b"csi_enter\0" as *const u8 as *const ::core::ffi::c_char,
        enter: Some(input_clear as unsafe extern "C" fn(*mut input_ctx) -> ()),
        exit: None,
        transitions: &raw const input_state_csi_enter_table as *const input_transition,
    }
};
static mut input_state_csi_parameter: input_state = unsafe {
    input_state {
        name: b"csi_parameter\0" as *const u8 as *const ::core::ffi::c_char,
        enter: None,
        exit: None,
        transitions: &raw const input_state_csi_parameter_table as *const input_transition,
    }
};
static mut input_state_csi_intermediate: input_state = unsafe {
    input_state {
        name: b"csi_intermediate\0" as *const u8 as *const ::core::ffi::c_char,
        enter: None,
        exit: None,
        transitions: &raw const input_state_csi_intermediate_table as *const input_transition,
    }
};
static mut input_state_csi_ignore: input_state = unsafe {
    input_state {
        name: b"csi_ignore\0" as *const u8 as *const ::core::ffi::c_char,
        enter: None,
        exit: None,
        transitions: &raw const input_state_csi_ignore_table as *const input_transition,
    }
};
static mut input_state_dcs_enter: input_state = unsafe {
    input_state {
        name: b"dcs_enter\0" as *const u8 as *const ::core::ffi::c_char,
        enter: Some(input_enter_dcs as unsafe extern "C" fn(*mut input_ctx) -> ()),
        exit: None,
        transitions: &raw const input_state_dcs_enter_table as *const input_transition,
    }
};
static mut input_state_dcs_parameter: input_state = unsafe {
    input_state {
        name: b"dcs_parameter\0" as *const u8 as *const ::core::ffi::c_char,
        enter: None,
        exit: None,
        transitions: &raw const input_state_dcs_parameter_table as *const input_transition,
    }
};
static mut input_state_dcs_intermediate: input_state = unsafe {
    input_state {
        name: b"dcs_intermediate\0" as *const u8 as *const ::core::ffi::c_char,
        enter: None,
        exit: None,
        transitions: &raw const input_state_dcs_intermediate_table as *const input_transition,
    }
};
static mut input_state_dcs_handler: input_state = unsafe {
    input_state {
        name: b"dcs_handler\0" as *const u8 as *const ::core::ffi::c_char,
        enter: None,
        exit: None,
        transitions: &raw const input_state_dcs_handler_table as *const input_transition,
    }
};
static mut input_state_dcs_escape: input_state = unsafe {
    input_state {
        name: b"dcs_escape\0" as *const u8 as *const ::core::ffi::c_char,
        enter: None,
        exit: None,
        transitions: &raw const input_state_dcs_escape_table as *const input_transition,
    }
};
static mut input_state_dcs_ignore: input_state = unsafe {
    input_state {
        name: b"dcs_ignore\0" as *const u8 as *const ::core::ffi::c_char,
        enter: None,
        exit: None,
        transitions: &raw const input_state_dcs_ignore_table as *const input_transition,
    }
};
static mut input_state_osc_string: input_state = unsafe {
    input_state {
        name: b"osc_string\0" as *const u8 as *const ::core::ffi::c_char,
        enter: Some(input_enter_osc as unsafe extern "C" fn(*mut input_ctx) -> ()),
        exit: Some(input_exit_osc as unsafe extern "C" fn(*mut input_ctx) -> ()),
        transitions: &raw const input_state_osc_string_table as *const input_transition,
    }
};
static mut input_state_apc_string: input_state = unsafe {
    input_state {
        name: b"apc_string\0" as *const u8 as *const ::core::ffi::c_char,
        enter: Some(input_enter_apc as unsafe extern "C" fn(*mut input_ctx) -> ()),
        exit: Some(input_exit_apc as unsafe extern "C" fn(*mut input_ctx) -> ()),
        transitions: &raw const input_state_apc_string_table as *const input_transition,
    }
};
static mut input_state_rename_string: input_state = unsafe {
    input_state {
        name: b"rename_string\0" as *const u8 as *const ::core::ffi::c_char,
        enter: Some(input_enter_rename as unsafe extern "C" fn(*mut input_ctx) -> ()),
        exit: Some(input_exit_rename as unsafe extern "C" fn(*mut input_ctx) -> ()),
        transitions: &raw const input_state_rename_string_table as *const input_transition,
    }
};
static mut input_state_consume_st: input_state = unsafe {
    input_state {
        name: b"consume_st\0" as *const u8 as *const ::core::ffi::c_char,
        enter: Some(input_enter_rename as unsafe extern "C" fn(*mut input_ctx) -> ()),
        exit: None,
        transitions: &raw const input_state_consume_st_table as *const input_transition,
    }
};
static mut input_state_ground_table: [input_transition; 10] = unsafe {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(
                input_print as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0x7f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x80 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(
                input_top_bit_set as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_esc_enter_table: [input_transition; 23] = unsafe {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(
                input_intermediate as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_esc_intermediate,
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x4f as ::core::ffi::c_int,
            handler: Some(
                input_esc_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x50 as ::core::ffi::c_int,
            last: 0x50 as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_dcs_enter,
        },
        input_transition {
            first: 0x51 as ::core::ffi::c_int,
            last: 0x57 as ::core::ffi::c_int,
            handler: Some(
                input_esc_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x58 as ::core::ffi::c_int,
            last: 0x58 as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_consume_st,
        },
        input_transition {
            first: 0x59 as ::core::ffi::c_int,
            last: 0x59 as ::core::ffi::c_int,
            handler: Some(
                input_esc_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x5a as ::core::ffi::c_int,
            last: 0x5a as ::core::ffi::c_int,
            handler: Some(
                input_esc_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x5b as ::core::ffi::c_int,
            last: 0x5b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_csi_enter,
        },
        input_transition {
            first: 0x5c as ::core::ffi::c_int,
            last: 0x5c as ::core::ffi::c_int,
            handler: Some(
                input_esc_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x5d as ::core::ffi::c_int,
            last: 0x5d as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_osc_string,
        },
        input_transition {
            first: 0x5e as ::core::ffi::c_int,
            last: 0x5e as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_consume_st,
        },
        input_transition {
            first: 0x5f as ::core::ffi::c_int,
            last: 0x5f as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_apc_string,
        },
        input_transition {
            first: 0x60 as ::core::ffi::c_int,
            last: 0x6a as ::core::ffi::c_int,
            handler: Some(
                input_esc_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x6b as ::core::ffi::c_int,
            last: 0x6b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_rename_string,
        },
        input_transition {
            first: 0x6c as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(
                input_esc_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_esc_intermediate_table: [input_transition; 10] = unsafe {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(
                input_intermediate as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(
                input_esc_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_csi_enter_table: [input_transition; 14] = unsafe {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(
                input_intermediate as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_csi_intermediate,
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x39 as ::core::ffi::c_int,
            handler: Some(
                input_parameter as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_csi_parameter,
        },
        input_transition {
            first: 0x3a as ::core::ffi::c_int,
            last: 0x3a as ::core::ffi::c_int,
            handler: Some(
                input_parameter as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_csi_parameter,
        },
        input_transition {
            first: 0x3b as ::core::ffi::c_int,
            last: 0x3b as ::core::ffi::c_int,
            handler: Some(
                input_parameter as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_csi_parameter,
        },
        input_transition {
            first: 0x3c as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: Some(
                input_intermediate as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_csi_parameter,
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(
                input_csi_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_csi_parameter_table: [input_transition; 14] = unsafe {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(
                input_intermediate as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_csi_intermediate,
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x39 as ::core::ffi::c_int,
            handler: Some(
                input_parameter as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x3a as ::core::ffi::c_int,
            last: 0x3a as ::core::ffi::c_int,
            handler: Some(
                input_parameter as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x3b as ::core::ffi::c_int,
            last: 0x3b as ::core::ffi::c_int,
            handler: Some(
                input_parameter as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x3c as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_csi_ignore,
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(
                input_csi_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_csi_intermediate_table: [input_transition; 11] = unsafe {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(
                input_intermediate as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_csi_ignore,
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(
                input_csi_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_csi_ignore_table: [input_transition; 10] = unsafe {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_dcs_enter_table: [input_transition; 14] = unsafe {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(
                input_intermediate as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_dcs_intermediate,
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x39 as ::core::ffi::c_int,
            handler: Some(
                input_parameter as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_dcs_parameter,
        },
        input_transition {
            first: 0x3a as ::core::ffi::c_int,
            last: 0x3a as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_dcs_ignore,
        },
        input_transition {
            first: 0x3b as ::core::ffi::c_int,
            last: 0x3b as ::core::ffi::c_int,
            handler: Some(
                input_parameter as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_dcs_parameter,
        },
        input_transition {
            first: 0x3c as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: Some(
                input_intermediate as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_dcs_parameter,
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(
                input_input as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_dcs_handler,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_dcs_parameter_table: [input_transition; 14] = unsafe {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(
                input_intermediate as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_dcs_intermediate,
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x39 as ::core::ffi::c_int,
            handler: Some(
                input_parameter as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x3a as ::core::ffi::c_int,
            last: 0x3a as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_dcs_ignore,
        },
        input_transition {
            first: 0x3b as ::core::ffi::c_int,
            last: 0x3b as ::core::ffi::c_int,
            handler: Some(
                input_parameter as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x3c as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_dcs_ignore,
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(
                input_input as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_dcs_handler,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_dcs_intermediate_table: [input_transition; 11] = unsafe {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0x2f as ::core::ffi::c_int,
            handler: Some(
                input_intermediate as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x30 as ::core::ffi::c_int,
            last: 0x3f as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_dcs_ignore,
        },
        input_transition {
            first: 0x40 as ::core::ffi::c_int,
            last: 0x7e as ::core::ffi::c_int,
            handler: Some(
                input_input as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_dcs_handler,
        },
        input_transition {
            first: 0x7f as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_dcs_handler_table: [input_transition; 4] = unsafe {
    [
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(
                input_input as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_dcs_escape,
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(
                input_input as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_dcs_escape_table: [input_transition; 4] = unsafe {
    [
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x5b as ::core::ffi::c_int,
            handler: Some(
                input_input as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_dcs_handler,
        },
        input_transition {
            first: 0x5c as ::core::ffi::c_int,
            last: 0x5c as ::core::ffi::c_int,
            handler: Some(
                input_dcs_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x5d as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(
                input_input as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_dcs_handler,
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_dcs_ignore_table: [input_transition; 8] = unsafe {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_osc_string_table: [input_transition; 10] = unsafe {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x6 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x7 as ::core::ffi::c_int,
            last: 0x7 as ::core::ffi::c_int,
            handler: Some(
                input_end_bel as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x8 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(
                input_input as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_apc_string_table: [input_transition; 8] = unsafe {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(
                input_input as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_rename_string_table: [input_transition; 8] = unsafe {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: Some(
                input_input as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_state_consume_st_table: [input_transition; 8] = unsafe {
    [
        input_transition {
            first: 0x18 as ::core::ffi::c_int,
            last: 0x18 as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1a as ::core::ffi::c_int,
            last: 0x1a as ::core::ffi::c_int,
            handler: Some(
                input_c0_dispatch as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int,
            ),
            state: &raw const input_state_ground,
        },
        input_transition {
            first: 0x1b as ::core::ffi::c_int,
            last: 0x1b as ::core::ffi::c_int,
            handler: None,
            state: &raw const input_state_esc_enter,
        },
        input_transition {
            first: 0 as ::core::ffi::c_int,
            last: 0x17 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x19 as ::core::ffi::c_int,
            last: 0x19 as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x1c as ::core::ffi::c_int,
            last: 0x1f as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: 0x20 as ::core::ffi::c_int,
            last: 0xff as ::core::ffi::c_int,
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
        input_transition {
            first: -(1 as ::core::ffi::c_int),
            last: -(1 as ::core::ffi::c_int),
            handler: None,
            state: ::core::ptr::null::<input_state>(),
        },
    ]
};
static mut input_buffer_size: size_t = INPUT_BUF_DEFAULT_SIZE as size_t;
unsafe extern "C" fn input_table_compare(
    mut key: *const ::core::ffi::c_void,
    mut value: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut ictx: *const input_ctx = key as *const input_ctx;
    let mut entry: *const input_table_entry = value as *const input_table_entry;
    if (*ictx).ch != (*entry).ch {
        return (*ictx).ch - (*entry).ch;
    }
    return strcmp(
        &raw const (*ictx).interm_buf as *const u_char as *const ::core::ffi::c_char,
        (*entry).interm,
    );
}
unsafe extern "C" fn input_stop_utf8(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    static mut rc: utf8_data = unsafe {
        utf8_data {
            data: ::core::mem::transmute::<[u8; 32], [u_char; 32]>(
                *b"\xEF\xBF\xBD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            have: 3 as u_char,
            size: 3 as u_char,
            width: 1 as u_char,
        }
    };
    if (*ictx).utf8started != 0 {
        utf8_copy(&raw mut (*ictx).cell.cell.data, &raw mut rc);
        screen_write_collect_add(sctx, &raw mut (*ictx).cell.cell);
    }
    (*ictx).utf8started = 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_fire_pane_title_changed(
    mut wp: *mut window_pane,
    mut title: *const ::core::ffi::c_char,
) {
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
        b"new_title\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        title,
    );
    events_fire(
        b"pane-title-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
unsafe extern "C" fn input_ground_timer_callback(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut ictx: *mut input_ctx = arg as *mut input_ctx;
    log_debug(
        b"%s: %s expired\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_ground_timer_callback\0" as *const u8 as *const ::core::ffi::c_char,
        (*(*ictx).state).name,
    );
    input_reset(ictx, 0 as ::core::ffi::c_int);
}
unsafe extern "C" fn input_start_ground_timer(mut ictx: *mut input_ctx) {
    let mut tv: timeval = timeval {
        tv_sec: 5 as __time_t,
        tv_usec: 0 as __suseconds_t,
    };
    event_del(&raw mut (*ictx).ground_timer);
    event_add(&raw mut (*ictx).ground_timer, &raw mut tv);
}
unsafe extern "C" fn input_reset_cell(mut ictx: *mut input_ctx) {
    memcpy(
        &raw mut (*ictx).cell.cell as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    (*ictx).cell.set = 0 as ::core::ffi::c_int;
    (*ictx).cell.g1set = 0 as ::core::ffi::c_int;
    (*ictx).cell.g0set = (*ictx).cell.g1set;
    memcpy(
        &raw mut (*ictx).old_cell as *mut ::core::ffi::c_void,
        &raw mut (*ictx).cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<input_cell>() as size_t,
    );
    (*ictx).old_cx = 0 as u_int;
    (*ictx).old_cy = 0 as u_int;
}
unsafe extern "C" fn input_save_state(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut s: *mut screen = (*sctx).s;
    memcpy(
        &raw mut (*ictx).old_cell as *mut ::core::ffi::c_void,
        &raw mut (*ictx).cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<input_cell>() as size_t,
    );
    (*ictx).old_cx = (*s).cx;
    (*ictx).old_cy = (*s).cy;
    (*ictx).old_mode = (*s).mode;
}
unsafe extern "C" fn input_restore_state(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    memcpy(
        &raw mut (*ictx).cell as *mut ::core::ffi::c_void,
        &raw mut (*ictx).old_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<input_cell>() as size_t,
    );
    if (*ictx).old_mode & MODE_ORIGIN != 0 {
        screen_write_mode_set(sctx, MODE_ORIGIN);
    } else {
        screen_write_mode_clear(sctx, MODE_ORIGIN);
    }
    screen_write_cursormove(
        sctx,
        (*ictx).old_cx as ::core::ffi::c_int,
        (*ictx).old_cy as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn input_init(
    mut wp: *mut window_pane,
    mut bev: *mut bufferevent,
    mut palette: *mut colour_palette,
    mut c: *mut client,
) -> *mut input_ctx {
    let mut ictx: *mut input_ctx = ::core::ptr::null_mut::<input_ctx>();
    ictx = xcalloc(1 as size_t, ::core::mem::size_of::<input_ctx>() as size_t) as *mut input_ctx;
    (*ictx).wp = wp;
    (*ictx).event = bev;
    (*ictx).palette = palette;
    (*ictx).c = c;
    (*ictx).input_space = INPUT_BUF_START as size_t;
    (*ictx).input_buf = xmalloc(INPUT_BUF_START as size_t) as *mut u_char;
    (*ictx).since_ground = evbuffer_new();
    if (*ictx).since_ground.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    event_set(
        &raw mut (*ictx).ground_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            input_ground_timer_callback
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        ictx as *mut ::core::ffi::c_void,
    );
    (*ictx).requests.tqh_first = ::core::ptr::null_mut::<input_request>();
    (*ictx).requests.tqh_last = &raw mut (*ictx).requests.tqh_first;
    event_set(
        &raw mut (*ictx).request_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            input_request_timer_callback
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        ictx as *mut ::core::ffi::c_void,
    );
    input_reset(ictx, 0 as ::core::ffi::c_int);
    return ictx;
}
#[no_mangle]
pub unsafe extern "C" fn input_free(mut ictx: *mut input_ctx) {
    let mut ir: *mut input_request = ::core::ptr::null_mut::<input_request>();
    let mut ir1: *mut input_request = ::core::ptr::null_mut::<input_request>();
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < (*ictx).param_list_len {
        if (*ictx).param_list[i as usize].type_0 as ::core::ffi::c_uint
            == INPUT_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            free((*ictx).param_list[i as usize].c2rust_unnamed.str_0 as *mut ::core::ffi::c_void);
        }
        i = i.wrapping_add(1);
    }
    ir = (*ictx).requests.tqh_first;
    while !ir.is_null() && {
        ir1 = (*ir).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        input_free_request(ir);
        ir = ir1;
    }
    event_del(&raw mut (*ictx).request_timer);
    free((*ictx).input_buf as *mut ::core::ffi::c_void);
    evbuffer_free((*ictx).since_ground);
    event_del(&raw mut (*ictx).ground_timer);
    screen_write_stop_sync((*ictx).wp);
    free(ictx as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn input_reset(mut ictx: *mut input_ctx, mut clear: ::core::ffi::c_int) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut wp: *mut window_pane = (*ictx).wp;
    input_reset_cell(ictx);
    if clear != 0 && !wp.is_null() {
        if (*wp).modes.tqh_first.is_null() {
            screen_write_start_pane(sctx, wp, &raw mut (*wp).base);
        } else {
            screen_write_start(sctx, &raw mut (*wp).base);
        }
        screen_write_reset(sctx);
        screen_write_stop(sctx);
    }
    input_clear(ictx);
    (*ictx).state = &raw const input_state_ground as *const input_state;
    (*ictx).flags = 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn input_pending(mut ictx: *mut input_ctx) -> *mut evbuffer {
    return (*ictx).since_ground;
}
unsafe extern "C" fn input_set_state(mut ictx: *mut input_ctx, mut itr: *const input_transition) {
    if (*(*ictx).state).exit.is_some() {
        (*(*ictx).state).exit.expect("non-null function pointer")(ictx);
    }
    (*ictx).state = (*itr).state as *const input_state;
    if (*(*ictx).state).enter.is_some() {
        (*(*ictx).state).enter.expect("non-null function pointer")(ictx);
    }
}
unsafe extern "C" fn input_parse(
    mut ictx: *mut input_ctx,
    mut buf: *const u_char,
    mut len: size_t,
) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut state: *const input_state = ::core::ptr::null::<input_state>();
    let mut itr: *const input_transition = ::core::ptr::null::<input_transition>();
    let mut off: size_t = 0 as size_t;
    while off < len {
        let fresh16 = off;
        off = off.wrapping_add(1);
        (*ictx).ch = *buf.offset(fresh16 as isize) as ::core::ffi::c_int;
        if (*ictx).state != state
            || itr.is_null()
            || (*ictx).ch < (*itr).first
            || (*ictx).ch > (*itr).last
        {
            itr = (*(*ictx).state).transitions;
            while (*itr).first != -(1 as ::core::ffi::c_int)
                && (*itr).last != -(1 as ::core::ffi::c_int)
            {
                if (*ictx).ch >= (*itr).first && (*ictx).ch <= (*itr).last {
                    break;
                }
                itr = itr.offset(1);
            }
            if (*itr).first == -(1 as ::core::ffi::c_int)
                || (*itr).last == -(1 as ::core::ffi::c_int)
            {
                fatalx(b"no transition from state\0" as *const u8 as *const ::core::ffi::c_char);
            }
        }
        state = (*ictx).state as *const input_state;
        if (*itr).handler
            != Some(input_print as unsafe extern "C" fn(*mut input_ctx) -> ::core::ffi::c_int)
        {
            screen_write_collect_end(sctx);
        }
        if (*itr).handler.is_some()
            && (*itr).handler.expect("non-null function pointer")(ictx) != 0 as ::core::ffi::c_int
        {
            continue;
        }
        if !(*itr).state.is_null() {
            input_set_state(ictx, itr);
        }
        if (*ictx).state != &raw const input_state_ground {
            evbuffer_add(
                (*ictx).since_ground,
                &raw mut (*ictx).ch as *const ::core::ffi::c_void,
                1 as size_t,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn input_parse_pane(mut wp: *mut window_pane) {
    let mut new_data: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut new_size: size_t = 0;
    new_data = window_pane_get_new_data(wp, &raw mut (*wp).offset, &raw mut new_size);
    if new_size != 0 as size_t {
        (*wp).last_output_time = time(::core::ptr::null_mut::<time_t>());
    }
    input_parse_buffer(wp, new_data as *const u_char, new_size);
    window_pane_update_used_data(wp, &raw mut (*wp).offset, new_size);
}
#[no_mangle]
pub unsafe extern "C" fn input_parse_buffer(
    mut wp: *mut window_pane,
    mut buf: *const u_char,
    mut len: size_t,
) {
    let mut ictx: *mut input_ctx = (*wp).ictx;
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    if len == 0 as size_t {
        return;
    }
    (*wp).output_generation = (*wp).output_generation.wrapping_add(1);
    window_update_activity((*wp).window as *mut window);
    if !(*wp).flags & PANE_ACTIVITY != 0 {
        (*wp).flags |= PANE_ACTIVITY;
        events_fire_pane(
            b"pane-activity\0" as *const u8 as *const ::core::ffi::c_char,
            wp,
        );
    }
    (*wp).flags |= PANE_CHANGED;
    if !(*wp).modes.tqh_first.is_null() {
        (*wp).flags |= PANE_UNSEENCHANGES;
    }
    if (*wp).modes.tqh_first.is_null() {
        screen_write_start_pane(sctx, wp, &raw mut (*wp).base);
    } else {
        screen_write_start(sctx, &raw mut (*wp).base);
    }
    log_debug(
        b"%s: %%%u %s, %zu bytes: %.*s\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_parse_buffer\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        (*(*ictx).state).name,
        len,
        len as ::core::ffi::c_int,
        buf,
    );
    input_parse(ictx, buf, len);
    screen_write_stop(sctx);
}
#[no_mangle]
pub unsafe extern "C" fn input_parse_screen(
    mut ictx: *mut input_ctx,
    mut s: *mut screen,
    mut cb: screen_write_init_ctx_cb,
    mut arg: *mut ::core::ffi::c_void,
    mut buf: *const u_char,
    mut len: size_t,
) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    if len == 0 as size_t {
        return;
    }
    screen_write_start_callback(sctx, s, cb, arg);
    input_parse(ictx, buf, len);
    screen_write_stop(sctx);
}
unsafe extern "C" fn input_split(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ip: *mut input_param = ::core::ptr::null_mut::<input_param>();
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < (*ictx).param_list_len {
        if (*ictx).param_list[i as usize].type_0 as ::core::ffi::c_uint
            == INPUT_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            free((*ictx).param_list[i as usize].c2rust_unnamed.str_0 as *mut ::core::ffi::c_void);
        }
        i = i.wrapping_add(1);
    }
    (*ictx).param_list_len = 0 as u_int;
    if (*ictx).param_len == 0 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    ip = (&raw mut (*ictx).param_list as *mut input_param).offset(0 as ::core::ffi::c_int as isize)
        as *mut input_param;
    ptr = &raw mut (*ictx).param_buf as *mut u_char as *mut ::core::ffi::c_char;
    loop {
        out = strsep(
            &raw mut ptr,
            b";\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if out.is_null() {
            break;
        }
        if *out as ::core::ffi::c_int == '\0' as i32 {
            (*ip).type_0 = INPUT_MISSING;
        } else if !strchr(out, ':' as i32).is_null() {
            (*ip).type_0 = INPUT_STRING;
            (*ip).c2rust_unnamed.str_0 = xstrdup(out);
        } else {
            (*ip).type_0 = INPUT_NUMBER;
            (*ip).c2rust_unnamed.num = strtonum(
                out,
                0 as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as ::core::ffi::c_int;
            if !errstr.is_null() {
                return -(1 as ::core::ffi::c_int);
            }
        }
        (*ictx).param_list_len = (*ictx).param_list_len.wrapping_add(1);
        ip = (&raw mut (*ictx).param_list as *mut input_param)
            .offset((*ictx).param_list_len as isize) as *mut input_param;
        if (*ictx).param_list_len as usize
            == (::core::mem::size_of::<[input_param; 24]>() as usize)
                .wrapping_div(::core::mem::size_of::<input_param>() as usize)
        {
            return -(1 as ::core::ffi::c_int);
        }
    }
    i = 0 as u_int;
    while i < (*ictx).param_list_len {
        ip = (&raw mut (*ictx).param_list as *mut input_param).offset(i as isize)
            as *mut input_param;
        if (*ip).type_0 as ::core::ffi::c_uint
            == INPUT_MISSING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            log_debug(
                b"parameter %u: missing\0" as *const u8 as *const ::core::ffi::c_char,
                i,
            );
        } else if (*ip).type_0 as ::core::ffi::c_uint
            == INPUT_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            log_debug(
                b"parameter %u: string %s\0" as *const u8 as *const ::core::ffi::c_char,
                i,
                (*ip).c2rust_unnamed.str_0,
            );
        } else if (*ip).type_0 as ::core::ffi::c_uint
            == INPUT_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            log_debug(
                b"parameter %u: number %d\0" as *const u8 as *const ::core::ffi::c_char,
                i,
                (*ip).c2rust_unnamed.num,
            );
        }
        i = i.wrapping_add(1);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_get(
    mut ictx: *mut input_ctx,
    mut validx: u_int,
    mut minval: ::core::ffi::c_int,
    mut defval: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut ip: *mut input_param = ::core::ptr::null_mut::<input_param>();
    let mut retval: ::core::ffi::c_int = 0;
    if validx >= (*ictx).param_list_len {
        return defval;
    }
    ip = (&raw mut (*ictx).param_list as *mut input_param).offset(validx as isize)
        as *mut input_param;
    if (*ip).type_0 as ::core::ffi::c_uint
        == INPUT_MISSING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return defval;
    }
    if (*ip).type_0 as ::core::ffi::c_uint
        == INPUT_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return -(1 as ::core::ffi::c_int);
    }
    retval = (*ip).c2rust_unnamed.num;
    if retval < minval {
        return minval;
    }
    return retval;
}
unsafe extern "C" fn input_send_reply(
    mut ictx: *mut input_ctx,
    mut reply: *const ::core::ffi::c_char,
) {
    if !(*ictx).event.is_null() {
        log_debug(
            b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"input_send_reply\0" as *const u8 as *const ::core::ffi::c_char,
            reply,
        );
        bufferevent_write(
            (*ictx).event,
            reply as *const ::core::ffi::c_void,
            strlen(reply),
        );
    }
}
unsafe extern "C" fn input_reply(
    mut ictx: *mut input_ctx,
    mut add: ::core::ffi::c_int,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ir: *mut input_request = ::core::ptr::null_mut::<input_request>();
    let mut ap: ::core::ffi::VaList;
    let mut reply: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    ap = args.clone();
    xvasprintf(&raw mut reply, fmt, ap);
    if add != 0 && !(*ictx).requests.tqh_first.is_null() {
        ir = input_make_request(ictx, INPUT_REQUEST_QUEUE);
        (*ir).data = reply as *mut ::core::ffi::c_void;
    } else {
        input_send_reply(ictx, reply);
        free(reply as *mut ::core::ffi::c_void);
    };
}
unsafe extern "C" fn input_clear(mut ictx: *mut input_ctx) {
    event_del(&raw mut (*ictx).ground_timer);
    *(&raw mut (*ictx).interm_buf as *mut u_char) = '\0' as i32 as u_char;
    (*ictx).interm_len = 0 as size_t;
    *(&raw mut (*ictx).param_buf as *mut u_char) = '\0' as i32 as u_char;
    (*ictx).param_len = 0 as size_t;
    *(*ictx).input_buf = '\0' as i32 as u_char;
    (*ictx).input_len = 0 as size_t;
    (*ictx).input_end = INPUT_END_ST;
    (*ictx).flags &= !INPUT_DISCARD;
}
unsafe extern "C" fn input_ground(mut ictx: *mut input_ctx) {
    event_del(&raw mut (*ictx).ground_timer);
    evbuffer_drain(
        (*ictx).since_ground,
        evbuffer_get_length((*ictx).since_ground),
    );
    if (*ictx).input_space > INPUT_BUF_START as size_t {
        (*ictx).input_space = INPUT_BUF_START as size_t;
        (*ictx).input_buf = xrealloc(
            (*ictx).input_buf as *mut ::core::ffi::c_void,
            INPUT_BUF_START as size_t,
        ) as *mut u_char;
    }
}
unsafe extern "C" fn input_print(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut set: ::core::ffi::c_int = 0;
    input_stop_utf8(ictx);
    set = if (*ictx).cell.set == 0 as ::core::ffi::c_int {
        (*ictx).cell.g0set
    } else {
        (*ictx).cell.g1set
    };
    if set == 1 as ::core::ffi::c_int {
        (*ictx).cell.cell.attr =
            ((*ictx).cell.cell.attr as ::core::ffi::c_int | GRID_ATTR_CHARSET) as u_short;
    } else {
        (*ictx).cell.cell.attr =
            ((*ictx).cell.cell.attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
    }
    utf8_set(&raw mut (*ictx).cell.cell.data, (*ictx).ch as u_char);
    screen_write_collect_add(sctx, &raw mut (*ictx).cell.cell);
    utf8_copy(&raw mut (*ictx).last, &raw mut (*ictx).cell.cell.data);
    (*ictx).flags |= INPUT_LAST;
    (*ictx).cell.cell.attr =
        ((*ictx).cell.cell.attr as ::core::ffi::c_int & !GRID_ATTR_CHARSET) as u_short;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_intermediate(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    if (*ictx).interm_len
        == (::core::mem::size_of::<[u_char; 4]>() as usize).wrapping_sub(1 as usize)
    {
        (*ictx).flags |= INPUT_DISCARD;
    } else {
        let fresh15 = (*ictx).interm_len;
        (*ictx).interm_len = (*ictx).interm_len.wrapping_add(1);
        (*ictx).interm_buf[fresh15 as usize] = (*ictx).ch as u_char;
        (*ictx).interm_buf[(*ictx).interm_len as usize] = '\0' as i32 as u_char;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_parameter(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    if (*ictx).param_len
        == (::core::mem::size_of::<[u_char; 64]>() as usize).wrapping_sub(1 as usize)
    {
        (*ictx).flags |= INPUT_DISCARD;
    } else {
        let fresh14 = (*ictx).param_len;
        (*ictx).param_len = (*ictx).param_len.wrapping_add(1);
        (*ictx).param_buf[fresh14 as usize] = (*ictx).ch as u_char;
        (*ictx).param_buf[(*ictx).param_len as usize] = '\0' as i32 as u_char;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_input(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut available: size_t = 0;
    available = (*ictx).input_space;
    while (*ictx).input_len.wrapping_add(1 as size_t) >= available {
        available = available.wrapping_mul(2 as size_t);
        if available > input_buffer_size {
            (*ictx).flags |= INPUT_DISCARD;
            return 0 as ::core::ffi::c_int;
        }
        (*ictx).input_buf =
            xrealloc((*ictx).input_buf as *mut ::core::ffi::c_void, available) as *mut u_char;
        (*ictx).input_space = available;
    }
    let fresh1 = (*ictx).input_len;
    (*ictx).input_len = (*ictx).input_len.wrapping_add(1);
    *(*ictx).input_buf.offset(fresh1 as isize) = (*ictx).ch as u_char;
    *(*ictx).input_buf.offset((*ictx).input_len as isize) = '\0' as i32 as u_char;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_c0_dispatch(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut s: *mut screen = (*sctx).s;
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
    let mut first_gc: grid_cell = grid_cell {
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
    let mut line: u_int = 0;
    let mut width: u_int = 0;
    let mut has_content: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    input_stop_utf8(ictx);
    log_debug(
        b"%s: '%c'\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_c0_dispatch\0" as *const u8 as *const ::core::ffi::c_char,
        (*ictx).ch,
    );
    match (*ictx).ch {
        0 => {}
        7 => {
            if !wp.is_null() {
                events_fire_pane(
                    b"pane-bell\0" as *const u8 as *const ::core::ffi::c_char,
                    wp,
                );
                alerts_queue((*wp).window as *mut window, WINDOW_BELL);
            }
        }
        8 => {
            screen_write_backspace(sctx);
        }
        9 => {
            cx = (*s).cx;
            if !(cx >= (*(*s).grid).sx.wrapping_sub(1 as u_int)) {
                line = (*s).cy.wrapping_add((*(*s).grid).hsize);
                grid_get_cell((*s).grid, cx, line, &raw mut first_gc);
                loop {
                    if has_content == 0 {
                        grid_get_cell((*s).grid, cx, line, &raw mut gc);
                        if gc.data.size as ::core::ffi::c_int != 1 as ::core::ffi::c_int
                            || *(&raw mut gc.data.data as *mut u_char) as ::core::ffi::c_int
                                != ' ' as i32
                            || grid_cells_look_equal(&raw mut gc, &raw mut first_gc) == 0
                        {
                            has_content = 1 as ::core::ffi::c_int;
                        }
                    }
                    cx = cx.wrapping_add(1);
                    if *(*s).tabs.offset((cx >> 3 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << (cx & 0x7 as u_int)
                        != 0
                    {
                        break;
                    }
                    if !(cx < (*(*s).grid).sx.wrapping_sub(1 as u_int)) {
                        break;
                    }
                }
                width = cx.wrapping_sub((*s).cx);
                if has_content != 0
                    || width as usize > ::core::mem::size_of::<[u_char; 32]>() as usize
                {
                    (*s).cx = cx;
                } else {
                    grid_get_cell((*s).grid, (*s).cx, line, &raw mut gc);
                    grid_set_tab(&raw mut gc, width);
                    screen_write_collect_add(sctx, &raw mut gc);
                }
            }
        }
        10 | 11 | 12 => {
            screen_write_linefeed(sctx, 0 as ::core::ffi::c_int, (*ictx).cell.cell.bg as u_int);
            if (*s).mode & MODE_CRLF != 0 {
                screen_write_carriagereturn(sctx);
            }
        }
        13 => {
            screen_write_carriagereturn(sctx);
        }
        14 => {
            (*ictx).cell.set = 1 as ::core::ffi::c_int;
        }
        15 => {
            (*ictx).cell.set = 0 as ::core::ffi::c_int;
        }
        _ => {
            log_debug(
                b"%s: unknown '%c'\0" as *const u8 as *const ::core::ffi::c_char,
                b"input_c0_dispatch\0" as *const u8 as *const ::core::ffi::c_char,
                (*ictx).ch,
            );
        }
    }
    (*ictx).flags &= !INPUT_LAST;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_esc_dispatch(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut s: *mut screen = (*sctx).s;
    let mut entry: *const input_table_entry = ::core::ptr::null::<input_table_entry>();
    if (*ictx).flags & INPUT_DISCARD != 0 {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: '%c', %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_esc_dispatch\0" as *const u8 as *const ::core::ffi::c_char,
        (*ictx).ch,
        &raw mut (*ictx).interm_buf as *mut u_char,
    );
    entry = bsearch(
        ictx as *const ::core::ffi::c_void,
        &raw const input_esc_table as *const input_table_entry as *const ::core::ffi::c_void,
        (::core::mem::size_of::<[input_table_entry; 15]>() as size_t)
            .wrapping_div(::core::mem::size_of::<input_table_entry>() as size_t),
        ::core::mem::size_of::<input_table_entry>() as size_t,
        Some(
            input_table_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    ) as *const input_table_entry;
    if entry.is_null() {
        log_debug(
            b"%s: unknown '%c'\0" as *const u8 as *const ::core::ffi::c_char,
            b"input_esc_dispatch\0" as *const u8 as *const ::core::ffi::c_char,
            (*ictx).ch,
        );
        return 0 as ::core::ffi::c_int;
    }
    match (*entry).type_0 {
        9 => {
            colour_palette_clear((*ictx).palette);
            input_reset_cell(ictx);
            screen_write_reset(sctx);
            screen_write_fullredraw(sctx);
        }
        6 => {
            screen_write_linefeed(sctx, 0 as ::core::ffi::c_int, (*ictx).cell.cell.bg as u_int);
        }
        7 => {
            screen_write_carriagereturn(sctx);
            screen_write_linefeed(sctx, 0 as ::core::ffi::c_int, (*ictx).cell.cell.bg as u_int);
        }
        5 => {
            if (*s).cx < (*(*s).grid).sx {
                let ref mut fresh0 = *(*s)
                    .tabs
                    .offset(((*s).cx >> 3 as ::core::ffi::c_int) as isize);
                *fresh0 = (*fresh0 as ::core::ffi::c_int
                    | (1 as ::core::ffi::c_int) << ((*s).cx & 0x7 as u_int))
                    as bitstr_t;
            }
        }
        8 => {
            screen_write_reverseindex(sctx, (*ictx).cell.cell.bg as u_int);
        }
        1 => {
            screen_write_mode_set(sctx, MODE_KKEYPAD);
        }
        2 => {
            screen_write_mode_clear(sctx, MODE_KKEYPAD);
        }
        4 => {
            input_save_state(ictx);
        }
        3 => {
            input_restore_state(ictx);
        }
        0 => {
            screen_write_alignmenttest(sctx);
        }
        11 => {
            (*ictx).cell.g0set = 1 as ::core::ffi::c_int;
        }
        10 => {
            (*ictx).cell.g0set = 0 as ::core::ffi::c_int;
        }
        13 => {
            (*ictx).cell.g1set = 1 as ::core::ffi::c_int;
        }
        12 => {
            (*ictx).cell.g1set = 0 as ::core::ffi::c_int;
        }
        14 | _ => {}
    }
    (*ictx).flags &= !INPUT_LAST;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_csi_dispatch(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut s: *mut screen = (*sctx).s;
    let mut entry: *const input_table_entry = ::core::ptr::null::<input_table_entry>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut i: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut m: ::core::ffi::c_int = 0;
    let mut ek: ::core::ffi::c_int = 0;
    let mut set: ::core::ffi::c_int = 0;
    let mut p: ::core::ffi::c_int = 0;
    let mut cx: u_int = 0;
    let mut bg: u_int = (*ictx).cell.cell.bg as u_int;
    if (*ictx).flags & INPUT_DISCARD != 0 {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: '%c' \"%s\" \"%s\"\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_csi_dispatch\0" as *const u8 as *const ::core::ffi::c_char,
        (*ictx).ch,
        &raw mut (*ictx).interm_buf as *mut u_char,
        &raw mut (*ictx).param_buf as *mut u_char,
    );
    if input_split(ictx) != 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    entry = bsearch(
        ictx as *const ::core::ffi::c_void,
        &raw const input_csi_table as *const input_table_entry as *const ::core::ffi::c_void,
        (::core::mem::size_of::<[input_table_entry; 43]>() as size_t)
            .wrapping_div(::core::mem::size_of::<input_table_entry>() as size_t),
        ::core::mem::size_of::<input_table_entry>() as size_t,
        Some(
            input_table_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    ) as *const input_table_entry;
    if entry.is_null() {
        log_debug(
            b"%s: unknown '%c'\0" as *const u8 as *const ::core::ffi::c_char,
            b"input_csi_dispatch\0" as *const u8 as *const ::core::ffi::c_char,
            (*ictx).ch,
        );
        return 0 as ::core::ffi::c_int;
    }
    match (*entry).type_0 {
        0 => {
            cx = (*s).cx;
            if cx > (*(*s).grid).sx.wrapping_sub(1 as u_int) {
                cx = (*(*s).grid).sx.wrapping_sub(1 as u_int);
            }
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if !(n == -(1 as ::core::ffi::c_int)) {
                while cx > 0 as u_int && {
                    let fresh10 = n;
                    n = n - 1;
                    fresh10 > 0 as ::core::ffi::c_int
                } {
                    loop {
                        cx = cx.wrapping_sub(1);
                        if !(cx > 0 as u_int
                            && *(*s).tabs.offset((cx >> 3 as ::core::ffi::c_int) as isize)
                                as ::core::ffi::c_int
                                & (1 as ::core::ffi::c_int) << (cx & 0x7 as u_int)
                                == 0)
                        {
                            break;
                        }
                    }
                }
                (*s).cx = cx;
            }
        }
        3 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) {
                screen_write_cursorleft(sctx, n as u_int);
            }
        }
        4 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) {
                screen_write_cursordown(sctx, n as u_int);
            }
        }
        5 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) {
                screen_write_cursorright(sctx, n as u_int);
            }
        }
        6 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            m = input_get(
                ictx,
                1 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) && m != -(1 as ::core::ffi::c_int) {
                screen_write_cursormove(
                    sctx,
                    m - 1 as ::core::ffi::c_int,
                    n - 1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
            }
        }
        23 => {
            n = input_get(
                ictx,
                0 as u_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            if !(n != 4 as ::core::ffi::c_int) {
                m = input_get(
                    ictx,
                    1 as u_int,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                );
                ek = options_get_number(
                    global_options,
                    b"extended-keys\0" as *const u8 as *const ::core::ffi::c_char,
                ) as ::core::ffi::c_int;
                if !(ek == 0 as ::core::ffi::c_int) {
                    screen_write_mode_clear(sctx, EXTENDED_KEY_MODES);
                    if m == 2 as ::core::ffi::c_int {
                        screen_write_mode_set(sctx, MODE_KEYS_EXTENDED_2);
                    } else if m == 1 as ::core::ffi::c_int || ek == 2 as ::core::ffi::c_int {
                        screen_write_mode_set(sctx, MODE_KEYS_EXTENDED);
                    }
                }
            }
        }
        22 => {
            n = input_get(
                ictx,
                0 as u_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            if !(n != 4 as ::core::ffi::c_int) {
                screen_write_mode_clear(sctx, MODE_KEYS_EXTENDED | MODE_KEYS_EXTENDED_2);
                if options_get_number(
                    global_options,
                    b"extended-keys\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 2 as ::core::ffi::c_longlong
                {
                    screen_write_mode_set(sctx, MODE_KEYS_EXTENDED);
                }
            }
        }
        39 => {
            input_csi_dispatch_winops(ictx);
        }
        7 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) {
                screen_write_cursorup(sctx, n as u_int);
            }
        }
        1 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) {
                screen_write_carriagereturn(sctx);
                screen_write_cursordown(sctx, n as u_int);
            }
        }
        2 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) {
                screen_write_carriagereturn(sctx);
                screen_write_cursorup(sctx, n as u_int);
            }
        }
        8 => {
            match input_get(
                ictx,
                0 as u_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            ) {
                -1 => {}
                0 => {
                    input_reply(
                        ictx,
                        1 as ::core::ffi::c_int,
                        b"\x1B[?1;2c\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                _ => {
                    log_debug(
                        b"%s: unknown '%c'\0" as *const u8 as *const ::core::ffi::c_char,
                        b"input_csi_dispatch\0" as *const u8 as *const ::core::ffi::c_char,
                        (*ictx).ch,
                    );
                }
            }
        }
        9 => {
            match input_get(
                ictx,
                0 as u_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            ) {
                -1 => {}
                0 => {
                    input_reply(
                        ictx,
                        1 as ::core::ffi::c_int,
                        b"\x1B[>84;0;0c\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                _ => {
                    log_debug(
                        b"%s: unknown '%c'\0" as *const u8 as *const ::core::ffi::c_char,
                        b"input_csi_dispatch\0" as *const u8 as *const ::core::ffi::c_char,
                        (*ictx).ch,
                    );
                }
            }
        }
        16 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) {
                screen_write_clearcharacter(sctx, n as u_int, bg);
            }
        }
        10 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) {
                screen_write_deletecharacter(sctx, n as u_int, bg);
            }
        }
        12 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            m = input_get(
                ictx,
                1 as u_int,
                1 as ::core::ffi::c_int,
                (*(*s).grid).sy as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) && m != -(1 as ::core::ffi::c_int) {
                screen_write_scrollregion(
                    sctx,
                    (n - 1 as ::core::ffi::c_int) as u_int,
                    (m - 1 as ::core::ffi::c_int) as u_int,
                );
            }
        }
        13 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) {
                screen_write_deleteline(sctx, n as u_int, bg);
            }
        }
        15 => {
            match input_get(
                ictx,
                0 as u_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            ) {
                996 => {
                    input_report_current_theme(ictx);
                }
                _ => {}
            }
        }
        24 => {
            m = input_get(
                ictx,
                0 as u_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            match m {
                4 => {
                    n = if (*s).mode & MODE_INSERT != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                _ => {
                    n = 0 as ::core::ffi::c_int;
                }
            }
            if m > 0 as ::core::ffi::c_int {
                input_reply(
                    ictx,
                    1 as ::core::ffi::c_int,
                    b"\x1B[%d;%d$y\0" as *const u8 as *const ::core::ffi::c_char,
                    m,
                    n,
                );
            }
        }
        25 => {
            m = input_get(
                ictx,
                0 as u_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            match m {
                1 => {
                    n = if (*s).mode & MODE_KCURSOR != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                3 => {
                    n = 4 as ::core::ffi::c_int;
                }
                6 => {
                    n = if (*s).mode & MODE_ORIGIN != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                7 => {
                    n = if (*s).mode & MODE_WRAP != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                12 => {
                    if (*s).cstyle as ::core::ffi::c_uint
                        != SCREEN_CURSOR_DEFAULT as ::core::ffi::c_int as ::core::ffi::c_uint
                        || (*s).mode & MODE_CURSOR_BLINKING_SET != 0
                    {
                        n = if (*s).mode & MODE_CURSOR_BLINKING != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    } else {
                        if !(*ictx).wp.is_null() {
                            oo = (*(*ictx).wp).options;
                        } else {
                            oo = global_w_options;
                        }
                        p = options_get_number(
                            oo,
                            b"cursor-style\0" as *const u8 as *const ::core::ffi::c_char,
                        ) as ::core::ffi::c_int;
                        n = if p == 1 as ::core::ffi::c_int
                            || p == 3 as ::core::ffi::c_int
                            || p == 5 as ::core::ffi::c_int
                        {
                            1 as ::core::ffi::c_int
                        } else {
                            2 as ::core::ffi::c_int
                        };
                    }
                }
                25 => {
                    n = if (*s).mode & MODE_CURSOR != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                47 | 1047 | 1049 => {
                    n = if !(*s).saved_grid.is_null() {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                1000 => {
                    n = if (*s).mode & MODE_MOUSE_STANDARD != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                1002 => {
                    n = if (*s).mode & MODE_MOUSE_BUTTON != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                1003 => {
                    n = if (*s).mode & MODE_MOUSE_ALL != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                1004 => {
                    n = if (*s).mode & MODE_FOCUSON != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                1005 => {
                    n = if (*s).mode & MODE_MOUSE_UTF8 != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                1006 => {
                    n = if (*s).mode & MODE_MOUSE_SGR != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                2004 => {
                    n = if (*s).mode & MODE_BRACKETPASTE != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                2026 => {
                    n = if (*s).mode & MODE_SYNC != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                2031 => {
                    n = if (*s).mode & MODE_THEME_UPDATES != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                _ => {
                    n = 0 as ::core::ffi::c_int;
                }
            }
            if m > 0 as ::core::ffi::c_int {
                input_reply(
                    ictx,
                    1 as ::core::ffi::c_int,
                    b"\x1B[?%d;%d$y\0" as *const u8 as *const ::core::ffi::c_char,
                    m,
                    n,
                );
            }
        }
        14 => {
            match input_get(
                ictx,
                0 as u_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            ) {
                -1 => {}
                5 => {
                    input_reply(
                        ictx,
                        1 as ::core::ffi::c_int,
                        b"\x1B[0n\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                }
                6 => {
                    input_reply(
                        ictx,
                        1 as ::core::ffi::c_int,
                        b"\x1B[%u;%uR\0" as *const u8 as *const ::core::ffi::c_char,
                        (*s).cy.wrapping_add(1 as u_int),
                        (*s).cx.wrapping_add(1 as u_int),
                    );
                }
                _ => {
                    log_debug(
                        b"%s: unknown '%c'\0" as *const u8 as *const ::core::ffi::c_char,
                        b"input_csi_dispatch\0" as *const u8 as *const ::core::ffi::c_char,
                        (*ictx).ch,
                    );
                }
            }
        }
        17 => {
            match input_get(
                ictx,
                0 as u_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            ) {
                -1 => {}
                0 => {
                    screen_write_clearendofscreen(sctx, bg);
                }
                1 => {
                    screen_write_clearstartofscreen(sctx, bg);
                }
                2 => {
                    screen_write_clearscreen(sctx, bg);
                }
                3 => {
                    if input_get(
                        ictx,
                        1 as u_int,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    ) == 0 as ::core::ffi::c_int
                    {
                        screen_write_clearhistory(sctx);
                    }
                }
                _ => {
                    log_debug(
                        b"%s: unknown '%c'\0" as *const u8 as *const ::core::ffi::c_char,
                        b"input_csi_dispatch\0" as *const u8 as *const ::core::ffi::c_char,
                        (*ictx).ch,
                    );
                }
            }
        }
        18 => {
            match input_get(
                ictx,
                0 as u_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            ) {
                -1 => {}
                0 => {
                    screen_write_clearendofline(sctx, bg);
                }
                1 => {
                    screen_write_clearstartofline(sctx, bg);
                }
                2 => {
                    screen_write_clearline(sctx, bg);
                }
                _ => {
                    log_debug(
                        b"%s: unknown '%c'\0" as *const u8 as *const ::core::ffi::c_char,
                        b"input_csi_dispatch\0" as *const u8 as *const ::core::ffi::c_char,
                        (*ictx).ch,
                    );
                }
            }
        }
        19 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) {
                screen_write_cursormove(
                    sctx,
                    n - 1 as ::core::ffi::c_int,
                    -(1 as ::core::ffi::c_int),
                    1 as ::core::ffi::c_int,
                );
            }
        }
        20 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) {
                screen_write_insertcharacter(sctx, n as u_int, bg);
            }
        }
        21 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) {
                screen_write_insertline(sctx, n as u_int, bg);
            }
        }
        27 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if !(n == -(1 as ::core::ffi::c_int)) {
                m = (*(*s).grid).sx.wrapping_sub((*s).cx) as ::core::ffi::c_int;
                if n > m {
                    n = m;
                }
                if !(!(*ictx).flags & INPUT_LAST != 0) {
                    set = if (*ictx).cell.set == 0 as ::core::ffi::c_int {
                        (*ictx).cell.g0set
                    } else {
                        (*ictx).cell.g1set
                    };
                    if set == 1 as ::core::ffi::c_int {
                        (*ictx).cell.cell.attr = ((*ictx).cell.cell.attr as ::core::ffi::c_int
                            | GRID_ATTR_CHARSET)
                            as u_short;
                    } else {
                        (*ictx).cell.cell.attr = ((*ictx).cell.cell.attr as ::core::ffi::c_int
                            & !GRID_ATTR_CHARSET)
                            as u_short;
                    }
                    utf8_copy(&raw mut (*ictx).cell.cell.data, &raw mut (*ictx).last);
                    i = 0 as ::core::ffi::c_int;
                    while i < n {
                        screen_write_collect_add(sctx, &raw mut (*ictx).cell.cell);
                        i += 1;
                    }
                }
            }
        }
        26 => {
            input_restore_state(ictx);
        }
        28 => {
            input_csi_dispatch_rm(ictx);
        }
        29 => {
            input_csi_dispatch_rm_private(ictx);
        }
        30 => {
            input_save_state(ictx);
        }
        32 => {
            input_csi_dispatch_sgr(ictx);
        }
        33 => {
            input_csi_dispatch_sm(ictx);
        }
        35 => {
            input_csi_dispatch_sm_private(ictx);
        }
        34 => {
            input_csi_dispatch_sm_graphics(ictx);
        }
        36 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) {
                screen_write_scrollup(sctx, n as u_int, bg);
            }
        }
        31 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) {
                screen_write_scrolldown(sctx, n as u_int, bg);
            }
        }
        37 => {
            match input_get(
                ictx,
                0 as u_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            ) {
                -1 => {}
                0 => {
                    if (*s).cx < (*(*s).grid).sx {
                        let ref mut fresh11 = *(*s)
                            .tabs
                            .offset(((*s).cx >> 3 as ::core::ffi::c_int) as isize);
                        *fresh11 = (*fresh11 as ::core::ffi::c_int
                            & !((1 as ::core::ffi::c_int) << ((*s).cx & 0x7 as u_int)))
                            as bitstr_t;
                    }
                }
                3 => {
                    let mut _name: *mut bitstr_t = (*s).tabs;
                    let mut _start: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut _stop: ::core::ffi::c_int =
                        (*(*s).grid).sx.wrapping_sub(1 as u_int) as ::core::ffi::c_int;
                    while _start <= _stop {
                        let ref mut fresh12 =
                            *_name.offset((_start >> 3 as ::core::ffi::c_int) as isize);
                        *fresh12 = (*fresh12 as ::core::ffi::c_int
                            & !((1 as ::core::ffi::c_int) << (_start & 0x7 as ::core::ffi::c_int)))
                            as bitstr_t;
                        _start += 1;
                    }
                }
                _ => {
                    log_debug(
                        b"%s: unknown '%c'\0" as *const u8 as *const ::core::ffi::c_char,
                        b"input_csi_dispatch\0" as *const u8 as *const ::core::ffi::c_char,
                        (*ictx).ch,
                    );
                }
            }
        }
        38 => {
            n = input_get(
                ictx,
                0 as u_int,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            if n != -(1 as ::core::ffi::c_int) {
                screen_write_cursormove(
                    sctx,
                    -(1 as ::core::ffi::c_int),
                    n - 1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
            }
        }
        11 => {
            n = input_get(
                ictx,
                0 as u_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            if !(n == -(1 as ::core::ffi::c_int)) {
                screen_set_cursor_style(n as u_int, &raw mut (*s).cstyle, &raw mut (*s).mode);
                if n == 0 as ::core::ffi::c_int {
                    screen_write_mode_clear(sctx, MODE_CURSOR_BLINKING_SET);
                }
            }
        }
        40 => {
            n = input_get(
                ictx,
                0 as u_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            );
            if n == 0 as ::core::ffi::c_int {
                input_reply(
                    ictx,
                    1 as ::core::ffi::c_int,
                    b"\x1BP>|tmux %s\x1B\\\0" as *const u8 as *const ::core::ffi::c_char,
                    getversion(),
                );
            }
        }
        _ => {}
    }
    (*ictx).flags &= !INPUT_LAST;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_csi_dispatch_rm(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < (*ictx).param_list_len {
        match input_get(ictx, i, 0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int)) {
            -1 => {}
            4 => {
                screen_write_mode_clear(sctx, MODE_INSERT);
            }
            34 => {
                screen_write_mode_set(sctx, MODE_CURSOR_VERY_VISIBLE);
            }
            _ => {
                log_debug(
                    b"%s: unknown '%c'\0" as *const u8 as *const ::core::ffi::c_char,
                    b"input_csi_dispatch_rm\0" as *const u8 as *const ::core::ffi::c_char,
                    (*ictx).ch,
                );
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn input_csi_dispatch_rm_private(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut gc: *mut grid_cell = &raw mut (*ictx).cell.cell;
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < (*ictx).param_list_len {
        match input_get(ictx, i, 0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int)) {
            -1 => {}
            1 => {
                screen_write_mode_clear(sctx, MODE_KCURSOR);
            }
            3 => {
                screen_write_cursormove(
                    sctx,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                screen_write_clearscreen(sctx, (*gc).bg as u_int);
            }
            6 => {
                screen_write_mode_clear(sctx, MODE_ORIGIN);
                screen_write_cursormove(
                    sctx,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
            }
            7 => {
                screen_write_mode_clear(sctx, MODE_WRAP);
            }
            12 => {
                screen_write_mode_clear(sctx, MODE_CURSOR_BLINKING);
                screen_write_mode_set(sctx, MODE_CURSOR_BLINKING_SET);
            }
            25 => {
                screen_write_mode_clear(sctx, MODE_CURSOR);
            }
            1000 | 1001 | 1002 | 1003 => {
                screen_write_mode_clear(sctx, ALL_MOUSE_MODES);
            }
            1004 => {
                screen_write_mode_clear(sctx, MODE_FOCUSON);
            }
            1005 => {
                screen_write_mode_clear(sctx, MODE_MOUSE_UTF8);
            }
            1006 => {
                screen_write_mode_clear(sctx, MODE_MOUSE_SGR);
            }
            47 | 1047 => {
                screen_write_alternateoff(sctx, gc, 0 as ::core::ffi::c_int);
            }
            1049 => {
                screen_write_alternateoff(sctx, gc, 1 as ::core::ffi::c_int);
            }
            2004 => {
                screen_write_mode_clear(sctx, MODE_BRACKETPASTE);
            }
            2026 => {
                screen_write_end_sync(sctx);
            }
            2031 => {
                screen_write_mode_clear(sctx, MODE_THEME_UPDATES);
                if !(*ictx).wp.is_null() {
                    (*(*ictx).wp).flags &= !PANE_THEMECHANGED;
                }
            }
            _ => {
                log_debug(
                    b"%s: unknown '%c'\0" as *const u8 as *const ::core::ffi::c_char,
                    b"input_csi_dispatch_rm_private\0" as *const u8 as *const ::core::ffi::c_char,
                    (*ictx).ch,
                );
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn input_csi_dispatch_sm(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < (*ictx).param_list_len {
        match input_get(ictx, i, 0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int)) {
            -1 => {}
            4 => {
                screen_write_mode_set(sctx, MODE_INSERT);
            }
            34 => {
                screen_write_mode_clear(sctx, MODE_CURSOR_VERY_VISIBLE);
            }
            _ => {
                log_debug(
                    b"%s: unknown '%c'\0" as *const u8 as *const ::core::ffi::c_char,
                    b"input_csi_dispatch_sm\0" as *const u8 as *const ::core::ffi::c_char,
                    (*ictx).ch,
                );
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn input_csi_dispatch_sm_private(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut gc: *mut grid_cell = &raw mut (*ictx).cell.cell;
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < (*ictx).param_list_len {
        match input_get(ictx, i, 0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int)) {
            -1 => {}
            1 => {
                screen_write_mode_set(sctx, MODE_KCURSOR);
            }
            3 => {
                screen_write_cursormove(
                    sctx,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                screen_write_clearscreen(sctx, (*ictx).cell.cell.bg as u_int);
            }
            6 => {
                screen_write_mode_set(sctx, MODE_ORIGIN);
                screen_write_cursormove(
                    sctx,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
            }
            7 => {
                screen_write_mode_set(sctx, MODE_WRAP);
            }
            12 => {
                screen_write_mode_set(sctx, MODE_CURSOR_BLINKING);
                screen_write_mode_set(sctx, MODE_CURSOR_BLINKING_SET);
            }
            25 => {
                screen_write_mode_set(sctx, MODE_CURSOR);
            }
            1000 => {
                screen_write_mode_clear(sctx, ALL_MOUSE_MODES);
                screen_write_mode_set(sctx, MODE_MOUSE_STANDARD);
            }
            1002 => {
                screen_write_mode_clear(sctx, ALL_MOUSE_MODES);
                screen_write_mode_set(sctx, MODE_MOUSE_BUTTON);
            }
            1003 => {
                screen_write_mode_clear(sctx, ALL_MOUSE_MODES);
                screen_write_mode_set(sctx, MODE_MOUSE_ALL);
            }
            1004 => {
                screen_write_mode_set(sctx, MODE_FOCUSON);
            }
            1005 => {
                screen_write_mode_set(sctx, MODE_MOUSE_UTF8);
            }
            1006 => {
                screen_write_mode_set(sctx, MODE_MOUSE_SGR);
            }
            47 | 1047 => {
                screen_write_alternateon(sctx, gc, 0 as ::core::ffi::c_int);
            }
            1049 => {
                screen_write_alternateon(sctx, gc, 1 as ::core::ffi::c_int);
            }
            2004 => {
                screen_write_mode_set(sctx, MODE_BRACKETPASTE);
            }
            2031 => {
                screen_write_mode_set(sctx, MODE_THEME_UPDATES);
                if !(*ictx).wp.is_null() {
                    (*(*ictx).wp).last_theme = window_pane_get_theme((*ictx).wp);
                    (*(*ictx).wp).flags &= !PANE_THEMECHANGED;
                }
            }
            2026 => {
                screen_write_start_sync((*ictx).wp);
            }
            _ => {
                log_debug(
                    b"%s: unknown '%c'\0" as *const u8 as *const ::core::ffi::c_char,
                    b"input_csi_dispatch_sm_private\0" as *const u8 as *const ::core::ffi::c_char,
                    (*ictx).ch,
                );
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn input_csi_dispatch_sm_graphics(mut ictx: *mut input_ctx) {}
unsafe extern "C" fn input_csi_dispatch_winops(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut s: *mut screen = (*sctx).s;
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut x: u_int = (*(*s).grid).sx;
    let mut y: u_int = (*(*s).grid).sy;
    let mut n: ::core::ffi::c_int = 0;
    let mut m: ::core::ffi::c_int = 0;
    if !wp.is_null() {
        w = (*wp).window as *mut window;
    }
    m = 0 as ::core::ffi::c_int;
    loop {
        n = input_get(
            ictx,
            m as u_int,
            0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
        );
        if !(n != -(1 as ::core::ffi::c_int)) {
            break;
        }
        let mut current_block_25: u64;
        match n {
            1 | 2 | 5 | 6 | 7 | 11 | 13 | 20 | 21 | 24 => {
                current_block_25 = 980989089337379490;
            }
            3 | 4 | 8 => {
                m += 1;
                if input_get(
                    ictx,
                    m as u_int,
                    0 as ::core::ffi::c_int,
                    -(1 as ::core::ffi::c_int),
                ) == -(1 as ::core::ffi::c_int)
                {
                    return;
                }
                current_block_25 = 8019652857213515700;
            }
            9 | 10 => {
                current_block_25 = 8019652857213515700;
            }
            14 => {
                if w.is_null() {
                    current_block_25 = 980989089337379490;
                } else {
                    input_reply(
                        ictx,
                        1 as ::core::ffi::c_int,
                        b"\x1B[4;%u;%ut\0" as *const u8 as *const ::core::ffi::c_char,
                        y.wrapping_mul((*w).ypixel),
                        x.wrapping_mul((*w).xpixel),
                    );
                    current_block_25 = 980989089337379490;
                }
            }
            15 => {
                if w.is_null() {
                    current_block_25 = 980989089337379490;
                } else {
                    input_reply(
                        ictx,
                        1 as ::core::ffi::c_int,
                        b"\x1B[5;%u;%ut\0" as *const u8 as *const ::core::ffi::c_char,
                        y.wrapping_mul((*w).ypixel),
                        x.wrapping_mul((*w).xpixel),
                    );
                    current_block_25 = 980989089337379490;
                }
            }
            16 => {
                if w.is_null() {
                    current_block_25 = 980989089337379490;
                } else {
                    input_reply(
                        ictx,
                        1 as ::core::ffi::c_int,
                        b"\x1B[6;%u;%ut\0" as *const u8 as *const ::core::ffi::c_char,
                        (*w).ypixel,
                        (*w).xpixel,
                    );
                    current_block_25 = 980989089337379490;
                }
            }
            18 => {
                input_reply(
                    ictx,
                    1 as ::core::ffi::c_int,
                    b"\x1B[8;%u;%ut\0" as *const u8 as *const ::core::ffi::c_char,
                    y,
                    x,
                );
                current_block_25 = 980989089337379490;
            }
            19 => {
                input_reply(
                    ictx,
                    1 as ::core::ffi::c_int,
                    b"\x1B[9;%u;%ut\0" as *const u8 as *const ::core::ffi::c_char,
                    y,
                    x,
                );
                current_block_25 = 980989089337379490;
            }
            22 => {
                m += 1;
                match input_get(
                    ictx,
                    m as u_int,
                    0 as ::core::ffi::c_int,
                    -(1 as ::core::ffi::c_int),
                ) {
                    -1 => return,
                    0 | 2 => {
                        screen_push_title((*sctx).s);
                    }
                    _ => {}
                }
                current_block_25 = 980989089337379490;
            }
            23 => {
                m += 1;
                match input_get(
                    ictx,
                    m as u_int,
                    0 as ::core::ffi::c_int,
                    -(1 as ::core::ffi::c_int),
                ) {
                    -1 => return,
                    0 | 2 => {
                        screen_pop_title((*sctx).s);
                        if !wp.is_null() {
                            input_fire_pane_title_changed(wp, (*(*sctx).s).title);
                            server_redraw_window_borders(w);
                            server_status_window(w);
                        }
                    }
                    _ => {}
                }
                current_block_25 = 980989089337379490;
            }
            _ => {
                log_debug(
                    b"%s: unknown '%c'\0" as *const u8 as *const ::core::ffi::c_char,
                    b"input_csi_dispatch_winops\0" as *const u8 as *const ::core::ffi::c_char,
                    (*ictx).ch,
                );
                current_block_25 = 980989089337379490;
            }
        }
        match current_block_25 {
            8019652857213515700 => {
                m += 1;
                if input_get(
                    ictx,
                    m as u_int,
                    0 as ::core::ffi::c_int,
                    -(1 as ::core::ffi::c_int),
                ) == -(1 as ::core::ffi::c_int)
                {
                    return;
                }
            }
            _ => {}
        }
        m += 1;
    }
}
unsafe extern "C" fn input_csi_dispatch_sgr_256_do(
    mut ictx: *mut input_ctx,
    mut fgbg: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut gc: *mut grid_cell = &raw mut (*ictx).cell.cell;
    if c == -(1 as ::core::ffi::c_int) || c > 255 as ::core::ffi::c_int {
        if fgbg == 38 as ::core::ffi::c_int {
            (*gc).fg = 8 as ::core::ffi::c_int;
        } else if fgbg == 48 as ::core::ffi::c_int {
            (*gc).bg = 8 as ::core::ffi::c_int;
        }
    } else if fgbg == 38 as ::core::ffi::c_int {
        (*gc).fg = c | COLOUR_FLAG_256;
    } else if fgbg == 48 as ::core::ffi::c_int {
        (*gc).bg = c | COLOUR_FLAG_256;
    } else if fgbg == 58 as ::core::ffi::c_int {
        (*gc).us = c | COLOUR_FLAG_256;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_csi_dispatch_sgr_256(
    mut ictx: *mut input_ctx,
    mut fgbg: ::core::ffi::c_int,
    mut i: *mut u_int,
) {
    let mut c: ::core::ffi::c_int = 0;
    c = input_get(
        ictx,
        (*i).wrapping_add(1 as u_int),
        0 as ::core::ffi::c_int,
        -(1 as ::core::ffi::c_int),
    );
    if input_csi_dispatch_sgr_256_do(ictx, fgbg, c) != 0 {
        *i = (*i).wrapping_add(1);
    }
}
unsafe extern "C" fn input_csi_dispatch_sgr_rgb_do(
    mut ictx: *mut input_ctx,
    mut fgbg: ::core::ffi::c_int,
    mut r: ::core::ffi::c_int,
    mut g: ::core::ffi::c_int,
    mut b: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut gc: *mut grid_cell = &raw mut (*ictx).cell.cell;
    if r == -(1 as ::core::ffi::c_int) || r > 255 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if g == -(1 as ::core::ffi::c_int) || g > 255 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if b == -(1 as ::core::ffi::c_int) || b > 255 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if fgbg == 38 as ::core::ffi::c_int {
        (*gc).fg = colour_join_rgb(r as u_char, g as u_char, b as u_char);
    } else if fgbg == 48 as ::core::ffi::c_int {
        (*gc).bg = colour_join_rgb(r as u_char, g as u_char, b as u_char);
    } else if fgbg == 58 as ::core::ffi::c_int {
        (*gc).us = colour_join_rgb(r as u_char, g as u_char, b as u_char);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_csi_dispatch_sgr_rgb(
    mut ictx: *mut input_ctx,
    mut fgbg: ::core::ffi::c_int,
    mut i: *mut u_int,
) {
    let mut r: ::core::ffi::c_int = 0;
    let mut g: ::core::ffi::c_int = 0;
    let mut b: ::core::ffi::c_int = 0;
    r = input_get(
        ictx,
        (*i).wrapping_add(1 as u_int),
        0 as ::core::ffi::c_int,
        -(1 as ::core::ffi::c_int),
    );
    g = input_get(
        ictx,
        (*i).wrapping_add(2 as u_int),
        0 as ::core::ffi::c_int,
        -(1 as ::core::ffi::c_int),
    );
    b = input_get(
        ictx,
        (*i).wrapping_add(3 as u_int),
        0 as ::core::ffi::c_int,
        -(1 as ::core::ffi::c_int),
    );
    if input_csi_dispatch_sgr_rgb_do(ictx, fgbg, r, g, b) != 0 {
        *i = (*i).wrapping_add(3 as u_int);
    }
}
unsafe extern "C" fn input_csi_dispatch_sgr_colon(mut ictx: *mut input_ctx, mut i: u_int) {
    let mut gc: *mut grid_cell = &raw mut (*ictx).cell.cell;
    let mut s: *mut ::core::ffi::c_char = (*ictx).param_list[i as usize].c2rust_unnamed.str_0;
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: [::core::ffi::c_int; 8] = [0; 8];
    let mut n: u_int = 0;
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    n = 0 as u_int;
    while (n as usize)
        < (::core::mem::size_of::<[::core::ffi::c_int; 8]>() as usize)
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>() as usize)
    {
        p[n as usize] = -(1 as ::core::ffi::c_int);
        n = n.wrapping_add(1);
    }
    n = 0 as u_int;
    copy = xstrdup(s);
    ptr = copy;
    loop {
        out = strsep(
            &raw mut ptr,
            b":\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if out.is_null() {
            break;
        }
        if *out as ::core::ffi::c_int != '\0' as i32 {
            let fresh13 = n;
            n = n.wrapping_add(1);
            p[fresh13 as usize] = strtonum(
                out,
                0 as ::core::ffi::c_longlong,
                INT_MAX as ::core::ffi::c_longlong,
                &raw mut errstr,
            ) as ::core::ffi::c_int;
            if !errstr.is_null()
                || n as usize
                    == (::core::mem::size_of::<[::core::ffi::c_int; 8]>() as usize)
                        .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>() as usize)
            {
                free(copy as *mut ::core::ffi::c_void);
                return;
            }
        } else {
            n = n.wrapping_add(1);
            if n as usize
                == (::core::mem::size_of::<[::core::ffi::c_int; 8]>() as usize)
                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_int>() as usize)
            {
                free(copy as *mut ::core::ffi::c_void);
                return;
            }
        }
        log_debug(
            b"%s: %u = %d\0" as *const u8 as *const ::core::ffi::c_char,
            b"input_csi_dispatch_sgr_colon\0" as *const u8 as *const ::core::ffi::c_char,
            n.wrapping_sub(1 as u_int),
            p[n.wrapping_sub(1 as u_int) as usize],
        );
    }
    free(copy as *mut ::core::ffi::c_void);
    if n == 0 as u_int {
        return;
    }
    if p[0 as ::core::ffi::c_int as usize] == 4 as ::core::ffi::c_int {
        if n != 2 as u_int {
            return;
        }
        match p[1 as ::core::ffi::c_int as usize] {
            0 => {
                (*gc).attr =
                    ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
            }
            1 => {
                (*gc).attr =
                    ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE) as u_short;
            }
            2 => {
                (*gc).attr =
                    ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE_2) as u_short;
            }
            3 => {
                (*gc).attr =
                    ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE_3) as u_short;
            }
            4 => {
                (*gc).attr =
                    ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE_4) as u_short;
            }
            5 => {
                (*gc).attr =
                    ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ALL_UNDERSCORE) as u_short;
                (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE_5) as u_short;
            }
            _ => {}
        }
        return;
    }
    if n < 2 as u_int
        || p[0 as ::core::ffi::c_int as usize] != 38 as ::core::ffi::c_int
            && p[0 as ::core::ffi::c_int as usize] != 48 as ::core::ffi::c_int
            && p[0 as ::core::ffi::c_int as usize] != 58 as ::core::ffi::c_int
    {
        return;
    }
    match p[1 as ::core::ffi::c_int as usize] {
        2 => {
            if !(n < 3 as u_int) {
                if n == 5 as u_int {
                    i = 2 as u_int;
                } else {
                    i = 3 as u_int;
                }
                if !(n < i.wrapping_add(3 as u_int)) {
                    input_csi_dispatch_sgr_rgb_do(
                        ictx,
                        p[0 as ::core::ffi::c_int as usize],
                        p[i as usize],
                        p[i.wrapping_add(1 as u_int) as usize],
                        p[i.wrapping_add(2 as u_int) as usize],
                    );
                }
            }
        }
        5 => {
            if !(n < 3 as u_int) {
                input_csi_dispatch_sgr_256_do(
                    ictx,
                    p[0 as ::core::ffi::c_int as usize],
                    p[2 as ::core::ffi::c_int as usize],
                );
            }
        }
        _ => {}
    };
}
unsafe extern "C" fn input_csi_dispatch_sgr(mut ictx: *mut input_ctx) {
    let mut gc: *mut grid_cell = &raw mut (*ictx).cell.cell;
    let mut i: u_int = 0;
    let mut link: u_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    if (*ictx).param_list_len == 0 as u_int {
        memcpy(
            gc as *mut ::core::ffi::c_void,
            &raw const grid_default_cell as *const ::core::ffi::c_void,
            ::core::mem::size_of::<grid_cell>() as size_t,
        );
        return;
    }
    i = 0 as u_int;
    while i < (*ictx).param_list_len {
        if (*ictx).param_list[i as usize].type_0 as ::core::ffi::c_uint
            == INPUT_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            input_csi_dispatch_sgr_colon(ictx, i);
        } else {
            n = input_get(ictx, i, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
            if !(n == -(1 as ::core::ffi::c_int)) {
                if n == 38 as ::core::ffi::c_int
                    || n == 48 as ::core::ffi::c_int
                    || n == 58 as ::core::ffi::c_int
                {
                    i = i.wrapping_add(1);
                    match input_get(ictx, i, 0 as ::core::ffi::c_int, -(1 as ::core::ffi::c_int)) {
                        2 => {
                            input_csi_dispatch_sgr_rgb(ictx, n, &raw mut i);
                        }
                        5 => {
                            input_csi_dispatch_sgr_256(ictx, n, &raw mut i);
                        }
                        _ => {}
                    }
                } else {
                    match n {
                        0 => {
                            link = (*gc).link;
                            memcpy(
                                gc as *mut ::core::ffi::c_void,
                                &raw const grid_default_cell as *const ::core::ffi::c_void,
                                ::core::mem::size_of::<grid_cell>() as size_t,
                            );
                            (*gc).link = link;
                        }
                        1 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_BRIGHT) as u_short;
                        }
                        2 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_DIM) as u_short;
                        }
                        3 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_ITALICS) as u_short;
                        }
                        4 => {
                            (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                & !GRID_ATTR_ALL_UNDERSCORE)
                                as u_short;
                            (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE)
                                as u_short;
                        }
                        5 | 6 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_BLINK) as u_short;
                        }
                        7 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_REVERSE) as u_short;
                        }
                        8 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_HIDDEN) as u_short;
                        }
                        9 => {
                            (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                | GRID_ATTR_STRIKETHROUGH)
                                as u_short;
                        }
                        21 => {
                            (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                & !GRID_ATTR_ALL_UNDERSCORE)
                                as u_short;
                            (*gc).attr = ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_UNDERSCORE_2)
                                as u_short;
                        }
                        22 => {
                            (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                & !(GRID_ATTR_BRIGHT | GRID_ATTR_DIM))
                                as u_short;
                        }
                        23 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_ITALICS) as u_short;
                        }
                        24 => {
                            (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                & !GRID_ATTR_ALL_UNDERSCORE)
                                as u_short;
                        }
                        25 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_BLINK) as u_short;
                        }
                        27 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_REVERSE) as u_short;
                        }
                        28 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_HIDDEN) as u_short;
                        }
                        29 => {
                            (*gc).attr = ((*gc).attr as ::core::ffi::c_int
                                & !GRID_ATTR_STRIKETHROUGH)
                                as u_short;
                        }
                        30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 => {
                            (*gc).fg = n - 30 as ::core::ffi::c_int;
                        }
                        39 => {
                            (*gc).fg = 8 as ::core::ffi::c_int;
                        }
                        40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 => {
                            (*gc).bg = n - 40 as ::core::ffi::c_int;
                        }
                        49 => {
                            (*gc).bg = 8 as ::core::ffi::c_int;
                        }
                        53 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int | GRID_ATTR_OVERLINE) as u_short;
                        }
                        55 => {
                            (*gc).attr =
                                ((*gc).attr as ::core::ffi::c_int & !GRID_ATTR_OVERLINE) as u_short;
                        }
                        59 => {
                            (*gc).us = 8 as ::core::ffi::c_int;
                        }
                        90 | 91 | 92 | 93 | 94 | 95 | 96 | 97 => {
                            (*gc).fg = n;
                        }
                        100 | 101 | 102 | 103 | 104 | 105 | 106 | 107 => {
                            (*gc).bg = n - 10 as ::core::ffi::c_int;
                        }
                        _ => {}
                    }
                }
            }
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn input_end_bel(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    log_debug(
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_end_bel\0" as *const u8 as *const ::core::ffi::c_char,
    );
    (*ictx).input_end = INPUT_END_BEL;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_enter_dcs(mut ictx: *mut input_ctx) {
    log_debug(
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_enter_dcs\0" as *const u8 as *const ::core::ffi::c_char,
    );
    input_clear(ictx);
    input_start_ground_timer(ictx);
    (*ictx).flags &= !INPUT_LAST;
}
unsafe extern "C" fn input_handle_decrqss(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut buf: *mut u_char = (*ictx).input_buf;
    let mut len: size_t = (*ictx).input_len;
    let mut s: *mut screen = (*sctx).s;
    let mut ps: ::core::ffi::c_int = 0;
    let mut opt_ps: ::core::ffi::c_int = 0;
    let mut blinking: ::core::ffi::c_int = 0;
    if len < 3 as size_t
        || *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ' ' as i32
        || *buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 'q' as i32
    {
        input_reply(
            ictx,
            1 as ::core::ffi::c_int,
            b"\x1BP0$r\x1B\\\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return 0 as ::core::ffi::c_int;
    } else {
        if (*s).cstyle as ::core::ffi::c_uint
            == SCREEN_CURSOR_BLOCK as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*s).cstyle as ::core::ffi::c_uint
                == SCREEN_CURSOR_UNDERLINE as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*s).cstyle as ::core::ffi::c_uint
                == SCREEN_CURSOR_BAR as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            blinking =
                ((*s).mode & MODE_CURSOR_BLINKING != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
            match (*s).cstyle as ::core::ffi::c_uint {
                1 => {
                    ps = if blinking != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        2 as ::core::ffi::c_int
                    };
                }
                2 => {
                    ps = if blinking != 0 {
                        3 as ::core::ffi::c_int
                    } else {
                        4 as ::core::ffi::c_int
                    };
                }
                3 => {
                    ps = if blinking != 0 {
                        5 as ::core::ffi::c_int
                    } else {
                        6 as ::core::ffi::c_int
                    };
                }
                _ => {
                    ps = 0 as ::core::ffi::c_int;
                }
            }
        } else {
            if !wp.is_null() {
                oo = (*wp).options;
            } else {
                oo = global_w_options;
            }
            opt_ps = options_get_number(
                oo,
                b"cursor-style\0" as *const u8 as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int;
            if opt_ps < 0 as ::core::ffi::c_int || opt_ps > 6 as ::core::ffi::c_int {
                opt_ps = 0 as ::core::ffi::c_int;
            }
            ps = opt_ps;
        }
        log_debug(
            b"%s: DECRQSS cursor -> Ps=%d (cstyle=%d mode=%#x)\0" as *const u8
                as *const ::core::ffi::c_char,
            b"input_handle_decrqss\0" as *const u8 as *const ::core::ffi::c_char,
            ps,
            (*s).cstyle as ::core::ffi::c_uint,
            (*s).mode,
        );
        input_reply(
            ictx,
            1 as ::core::ffi::c_int,
            b"\x1BP1$r q%d q\x1B\\\0" as *const u8 as *const ::core::ffi::c_char,
            ps,
        );
        return 0 as ::core::ffi::c_int;
    };
}
unsafe extern "C" fn input_dcs_dispatch(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut buf: *mut u_char = (*ictx).input_buf;
    let mut len: size_t = (*ictx).input_len;
    let prefix: [::core::ffi::c_char; 6] =
        ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"tmux;\0");
    let prefixlen: u_int = (::core::mem::size_of::<[::core::ffi::c_char; 6]>() as usize)
        .wrapping_sub(1 as usize) as u_int;
    let mut allow_passthrough: ::core::ffi::c_longlong = 0 as ::core::ffi::c_longlong;
    if wp.is_null() {
        oo = global_w_options;
    } else {
        oo = (*wp).options;
    }
    if (*ictx).flags & INPUT_DISCARD != 0 {
        log_debug(
            b"%s: %zu bytes (discard)\0" as *const u8 as *const ::core::ffi::c_char,
            b"input_dcs_dispatch\0" as *const u8 as *const ::core::ffi::c_char,
            len,
        );
        return 0 as ::core::ffi::c_int;
    }
    if (*ictx).interm_len == 1 as size_t
        && (*ictx).interm_buf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == '$' as i32
    {
        if len >= 1 as size_t
            && *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == 'q' as i32
        {
            return input_handle_decrqss(ictx);
        }
    }
    allow_passthrough = options_get_number(
        oo,
        b"allow-passthrough\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if allow_passthrough == 0 {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: \"%s\"\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_dcs_dispatch\0" as *const u8 as *const ::core::ffi::c_char,
        buf,
    );
    if len >= prefixlen as size_t
        && strncmp(
            buf as *const ::core::ffi::c_char,
            &raw const prefix as *const ::core::ffi::c_char,
            prefixlen as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        screen_write_rawstring(
            sctx,
            buf.offset(prefixlen as isize),
            len.wrapping_sub(prefixlen as size_t) as u_int,
            (allow_passthrough == 2 as ::core::ffi::c_longlong) as ::core::ffi::c_int,
        );
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_enter_osc(mut ictx: *mut input_ctx) {
    log_debug(
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_enter_osc\0" as *const u8 as *const ::core::ffi::c_char,
    );
    input_clear(ictx);
    input_start_ground_timer(ictx);
    (*ictx).flags &= !INPUT_LAST;
}
unsafe extern "C" fn input_exit_osc(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut p: *mut u_char = (*ictx).input_buf;
    let mut option: u_int = 0;
    if (*ictx).flags & INPUT_DISCARD != 0 {
        return;
    }
    if (*ictx).input_len < 1 as size_t
        || (*p as ::core::ffi::c_int) < '0' as i32
        || *p as ::core::ffi::c_int > '9' as i32
    {
        return;
    }
    log_debug(
        b"%s: \"%s\" (end %s)\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_exit_osc\0" as *const u8 as *const ::core::ffi::c_char,
        p,
        if (*ictx).input_end as ::core::ffi::c_uint
            == INPUT_END_ST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            b"ST\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"BEL\0" as *const u8 as *const ::core::ffi::c_char
        },
    );
    option = 0 as u_int;
    while *p as ::core::ffi::c_int >= '0' as i32 && *p as ::core::ffi::c_int <= '9' as i32 {
        let fresh2 = p;
        p = p.offset(1);
        option = option
            .wrapping_mul(10 as u_int)
            .wrapping_add(*fresh2 as u_int)
            .wrapping_sub('0' as i32 as u_int);
    }
    if *p as ::core::ffi::c_int != ';' as i32 && *p as ::core::ffi::c_int != '\0' as i32 {
        return;
    }
    if *p as ::core::ffi::c_int == ';' as i32 {
        p = p.offset(1);
    }
    match option {
        0 | 2 => {
            if !wp.is_null()
                && options_get_number(
                    (*wp).options,
                    b"allow-set-title\0" as *const u8 as *const ::core::ffi::c_char,
                ) != 0
                && screen_set_title(
                    (*sctx).s,
                    p as *const ::core::ffi::c_char,
                    1 as ::core::ffi::c_int,
                ) != 0
            {
                input_fire_pane_title_changed(wp, p as *const ::core::ffi::c_char);
                server_redraw_window_borders((*wp).window as *mut window);
                server_status_window((*wp).window as *mut window);
            }
        }
        4 => {
            input_osc_4(ictx, p as *const ::core::ffi::c_char);
        }
        7 => {
            if !wp.is_null()
                && screen_set_path(
                    (*sctx).s,
                    p as *const ::core::ffi::c_char,
                    1 as ::core::ffi::c_int,
                ) != 0
            {
                server_redraw_window_borders((*wp).window as *mut window);
                server_status_window((*wp).window as *mut window);
            }
        }
        8 => {
            input_osc_8(ictx, p as *const ::core::ffi::c_char);
        }
        9 => {
            input_osc_9(ictx, p as *const ::core::ffi::c_char);
        }
        10 => {
            input_osc_10(ictx, p as *const ::core::ffi::c_char);
        }
        11 => {
            input_osc_11(ictx, p as *const ::core::ffi::c_char);
        }
        12 => {
            input_osc_12(ictx, p as *const ::core::ffi::c_char);
        }
        52 => {
            input_osc_52(ictx, p as *const ::core::ffi::c_char);
        }
        104 => {
            input_osc_104(ictx, p as *const ::core::ffi::c_char);
        }
        110 => {
            input_osc_110(ictx, p as *const ::core::ffi::c_char);
        }
        111 => {
            input_osc_111(ictx, p as *const ::core::ffi::c_char);
        }
        112 => {
            input_osc_112(ictx, p as *const ::core::ffi::c_char);
        }
        133 => {
            input_osc_133(ictx, p as *const ::core::ffi::c_char);
        }
        _ => {
            log_debug(
                b"%s: unknown '%u'\0" as *const u8 as *const ::core::ffi::c_char,
                b"input_exit_osc\0" as *const u8 as *const ::core::ffi::c_char,
                option,
            );
        }
    };
}
unsafe extern "C" fn input_enter_apc(mut ictx: *mut input_ctx) {
    log_debug(
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_enter_apc\0" as *const u8 as *const ::core::ffi::c_char,
    );
    input_clear(ictx);
    input_start_ground_timer(ictx);
    (*ictx).flags &= !INPUT_LAST;
}
unsafe extern "C" fn input_exit_apc(mut ictx: *mut input_ctx) {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut wp: *mut window_pane = (*ictx).wp;
    if (*ictx).flags & INPUT_DISCARD != 0 {
        return;
    }
    log_debug(
        b"%s: \"%s\"\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_exit_apc\0" as *const u8 as *const ::core::ffi::c_char,
        (*ictx).input_buf,
    );
    if !wp.is_null()
        && options_get_number(
            (*wp).options,
            b"allow-set-title\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
        && screen_set_title(
            (*sctx).s,
            (*ictx).input_buf as *const ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        ) != 0
    {
        input_fire_pane_title_changed(wp, (*ictx).input_buf as *const ::core::ffi::c_char);
        server_redraw_window_borders((*wp).window as *mut window);
        server_status_window((*wp).window as *mut window);
    }
}
unsafe extern "C" fn input_enter_rename(mut ictx: *mut input_ctx) {
    log_debug(
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_enter_rename\0" as *const u8 as *const ::core::ffi::c_char,
    );
    input_clear(ictx);
    input_start_ground_timer(ictx);
    (*ictx).flags &= !INPUT_LAST;
}
unsafe extern "C" fn input_exit_rename(mut ictx: *mut input_ctx) {
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if wp.is_null() {
        return;
    }
    if (*ictx).flags & INPUT_DISCARD != 0 {
        return;
    }
    if options_get_number(
        (*(*ictx).wp).options,
        b"allow-rename\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0
    {
        return;
    }
    log_debug(
        b"%s: \"%s\"\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_exit_rename\0" as *const u8 as *const ::core::ffi::c_char,
        (*ictx).input_buf,
    );
    if utf8_isvalid((*ictx).input_buf as *const ::core::ffi::c_char) == 0 {
        return;
    }
    w = (*wp).window as *mut window;
    if (*ictx).input_len == 0 as size_t {
        o = options_get_only(
            (*w).options,
            b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if !o.is_null() {
            options_remove_or_default(
                o,
                ::core::ptr::null::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
        }
        if options_get_number(
            (*w).options,
            b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0
        {
            window_set_name(
                w,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
                1 as ::core::ffi::c_int,
            );
        }
    } else {
        options_set_number(
            (*w).options,
            b"automatic-rename\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_longlong,
        );
        window_set_name(
            w,
            (*ictx).input_buf as *const ::core::ffi::c_char,
            1 as ::core::ffi::c_int,
        );
    }
    server_redraw_window_borders(w);
    server_status_window(w);
}
unsafe extern "C" fn input_top_bit_set(mut ictx: *mut input_ctx) -> ::core::ffi::c_int {
    let mut sctx: *mut screen_write_ctx = &raw mut (*ictx).ctx;
    let mut ud: *mut utf8_data = &raw mut (*ictx).utf8data;
    (*ictx).flags &= !INPUT_LAST;
    if (*ictx).utf8started == 0 {
        (*ictx).utf8started = 1 as ::core::ffi::c_int;
        if utf8_open(ud, (*ictx).ch as u_char) as ::core::ffi::c_uint
            != UTF8_MORE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            input_stop_utf8(ictx);
        }
        return 0 as ::core::ffi::c_int;
    }
    match utf8_append(ud, (*ictx).ch as u_char) as ::core::ffi::c_uint {
        0 => return 0 as ::core::ffi::c_int,
        2 => {
            input_stop_utf8(ictx);
            return 0 as ::core::ffi::c_int;
        }
        1 | _ => {}
    }
    (*ictx).utf8started = 0 as ::core::ffi::c_int;
    log_debug(
        b"%s %hhu '%*s' (width %hhu)\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_top_bit_set\0" as *const u8 as *const ::core::ffi::c_char,
        (*ud).size as ::core::ffi::c_int,
        (*ud).size as ::core::ffi::c_int,
        &raw mut (*ud).data as *mut u_char,
        (*ud).width as ::core::ffi::c_int,
    );
    utf8_copy(&raw mut (*ictx).cell.cell.data, ud);
    screen_write_collect_add(sctx, &raw mut (*ictx).cell.cell);
    utf8_copy(&raw mut (*ictx).last, &raw mut (*ictx).cell.cell.data);
    (*ictx).flags |= INPUT_LAST;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_osc_colour_reply(
    mut ictx: *mut input_ctx,
    mut add: ::core::ffi::c_int,
    mut n: u_int,
    mut idx: ::core::ffi::c_int,
    mut c: ::core::ffi::c_int,
    mut end_type: input_end_type,
) {
    let mut r: u_char = 0;
    let mut g: u_char = 0;
    let mut b: u_char = 0;
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if c != -(1 as ::core::ffi::c_int) {
        c = colour_force_rgb(c);
    }
    if c == -(1 as ::core::ffi::c_int) {
        return;
    }
    colour_split_rgb(c, &raw mut r, &raw mut g, &raw mut b);
    if end_type as ::core::ffi::c_uint == INPUT_END_BEL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        end = b"\x07\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        end = b"\x1B\\\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if n == 4 as u_int {
        input_reply(
            ictx,
            add,
            b"\x1B]%u;%d;rgb:%02hhx%02hhx/%02hhx%02hhx/%02hhx%02hhx%s\0" as *const u8
                as *const ::core::ffi::c_char,
            n,
            idx,
            r as ::core::ffi::c_int,
            r as ::core::ffi::c_int,
            g as ::core::ffi::c_int,
            g as ::core::ffi::c_int,
            b as ::core::ffi::c_int,
            b as ::core::ffi::c_int,
            end,
        );
    } else {
        input_reply(
            ictx,
            add,
            b"\x1B]%u;rgb:%02hhx%02hhx/%02hhx%02hhx/%02hhx%02hhx%s\0" as *const u8
                as *const ::core::ffi::c_char,
            n,
            r as ::core::ffi::c_int,
            r as ::core::ffi::c_int,
            g as ::core::ffi::c_int,
            g as ::core::ffi::c_int,
            b as ::core::ffi::c_int,
            b as ::core::ffi::c_int,
            end,
        );
    };
}
unsafe extern "C" fn input_osc_4(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_long = 0;
    let mut c: ::core::ffi::c_int = 0;
    let mut bad: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut palette: *mut colour_palette = (*ictx).palette;
    s = xstrdup(p);
    copy = s;
    while !s.is_null() && *s as ::core::ffi::c_int != '\0' as i32 {
        idx = strtol(s, &raw mut next, 10 as ::core::ffi::c_int);
        let fresh9 = next;
        next = next.offset(1);
        if *fresh9 as ::core::ffi::c_int != ';' as i32 {
            bad = 1 as ::core::ffi::c_int;
            break;
        } else if idx < 0 as ::core::ffi::c_long || idx >= 256 as ::core::ffi::c_long {
            bad = 1 as ::core::ffi::c_int;
            break;
        } else {
            s = strsep(
                &raw mut next,
                b";\0" as *const u8 as *const ::core::ffi::c_char,
            );
            if strcmp(s, b"?\0" as *const u8 as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
            {
                c = colour_palette_get(
                    palette,
                    (idx | COLOUR_FLAG_256 as ::core::ffi::c_long) as ::core::ffi::c_int,
                );
                if c != -(1 as ::core::ffi::c_int) {
                    input_osc_colour_reply(
                        ictx,
                        1 as ::core::ffi::c_int,
                        4 as u_int,
                        idx as ::core::ffi::c_int,
                        c,
                        (*ictx).input_end,
                    );
                    s = next;
                } else {
                    input_add_request(ictx, INPUT_REQUEST_PALETTE, idx as ::core::ffi::c_int);
                    s = next;
                }
            } else {
                c = colour_parseX11(s);
                if c == -(1 as ::core::ffi::c_int) {
                    s = next;
                } else {
                    if colour_palette_set(palette, idx as ::core::ffi::c_int, c) != 0 {
                        redraw = 1 as ::core::ffi::c_int;
                    }
                    s = next;
                }
            }
        }
    }
    if bad != 0 {
        log_debug(
            b"bad OSC 4: %s\0" as *const u8 as *const ::core::ffi::c_char,
            p,
        );
    }
    if redraw != 0 {
        screen_write_fullredraw(&raw mut (*ictx).ctx);
    }
    free(copy as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn input_osc_8(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut current_block: u64;
    let mut hl: *mut hyperlinks = (*(*ictx).ctx.s).hyperlinks;
    let mut gc: *mut grid_cell = &raw mut (*ictx).cell.cell;
    let mut start: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut uri: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut id: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    start = p;
    loop {
        end = strpbrk(start, b":;\0" as *const u8 as *const ::core::ffi::c_char);
        if end.is_null() {
            current_block = 10886091980245723256;
            break;
        }
        if end.offset_from(start) as ::core::ffi::c_long >= 4 as ::core::ffi::c_long
            && strncmp(
                start,
                b"id=\0" as *const u8 as *const ::core::ffi::c_char,
                3 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            if !id.is_null() {
                current_block = 9416799868769213755;
                break;
            }
            id = xstrndup(
                start.offset(3 as ::core::ffi::c_int as isize),
                (end.offset_from(start) as ::core::ffi::c_long - 3 as ::core::ffi::c_long)
                    as size_t,
            );
        }
        if *end as ::core::ffi::c_int == ';' as i32 {
            current_block = 10886091980245723256;
            break;
        }
        start = end.offset(1 as ::core::ffi::c_int as isize);
    }
    match current_block {
        10886091980245723256 => {
            if !(end.is_null() || *end as ::core::ffi::c_int != ';' as i32) {
                uri = end.offset(1 as ::core::ffi::c_int as isize);
                if *uri as ::core::ffi::c_int == '\0' as i32 {
                    (*gc).link = 0 as u_int;
                    free(id as *mut ::core::ffi::c_void);
                    return;
                }
                (*gc).link = hyperlinks_put(hl, uri, id);
                if id.is_null() {
                    log_debug(
                        b"hyperlink (anonymous) %s = %u\0" as *const u8
                            as *const ::core::ffi::c_char,
                        uri,
                        (*gc).link,
                    );
                } else {
                    log_debug(
                        b"hyperlink (id=%s) %s = %u\0" as *const u8 as *const ::core::ffi::c_char,
                        id,
                        uri,
                        (*gc).link,
                    );
                }
                free(id as *mut ::core::ffi::c_void);
                return;
            }
        }
        _ => {}
    }
    log_debug(
        b"bad OSC 8 %s\0" as *const u8 as *const ::core::ffi::c_char,
        p,
    );
    free(id as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn input_set_progress_bar(
    mut ictx: *mut input_ctx,
    mut state: progress_bar_state,
    mut p: ::core::ffi::c_int,
) {
    screen_set_progress_bar((*ictx).ctx.s, state, p);
    if !(*ictx).wp.is_null() {
        server_redraw_window_borders((*(*ictx).wp).window as *mut window);
        server_status_window((*(*ictx).wp).window as *mut window);
    }
}
unsafe extern "C" fn input_osc_9(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut current_block: u64;
    let mut pb: *const ::core::ffi::c_char = p;
    let mut state: progress_bar_state = PROGRESS_BAR_HIDDEN;
    let mut progress: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let fresh4 = pb;
    pb = pb.offset(1);
    if *fresh4 as ::core::ffi::c_int != '4' as i32 {
        return;
    }
    if *pb as ::core::ffi::c_int == '\0' as i32
        || *pb as ::core::ffi::c_int == ';' as i32
            && *pb.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
    {
        return;
    }
    let fresh5 = pb;
    pb = pb.offset(1);
    if *fresh5 as ::core::ffi::c_int != ';' as i32 {
        return;
    }
    if !((*pb as ::core::ffi::c_int) < '0' as i32 || *pb as ::core::ffi::c_int > '4' as i32) {
        let fresh6 = pb;
        pb = pb.offset(1);
        state = (*fresh6 as ::core::ffi::c_int - '0' as i32) as progress_bar_state;
        if *pb as ::core::ffi::c_int == '\0' as i32
            || *pb as ::core::ffi::c_int == ';' as i32
                && *pb.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
        {
            input_set_progress_bar(ictx, state, -(1 as ::core::ffi::c_int));
            return;
        }
        let fresh7 = pb;
        pb = pb.offset(1);
        if !(*fresh7 as ::core::ffi::c_int != ';' as i32) {
            loop {
                if !(*pb as ::core::ffi::c_int >= '0' as i32
                    && *pb as ::core::ffi::c_int <= '9' as i32)
                {
                    current_block = 10599921512955367680;
                    break;
                }
                if progress > 100 as ::core::ffi::c_int {
                    current_block = 12757115586032245927;
                    break;
                }
                let fresh8 = pb;
                pb = pb.offset(1);
                progress = progress * 10 as ::core::ffi::c_int + *fresh8 as ::core::ffi::c_int
                    - '0' as i32;
            }
            match current_block {
                12757115586032245927 => {}
                _ => {
                    if !(*pb as ::core::ffi::c_int != '\0' as i32
                        || progress < 0 as ::core::ffi::c_int
                        || progress > 100 as ::core::ffi::c_int)
                    {
                        input_set_progress_bar(ictx, state, progress);
                        return;
                    }
                }
            }
        }
    }
    log_debug(
        b"bad OSC 9;4 %s\0" as *const u8 as *const ::core::ffi::c_char,
        p,
    );
}
unsafe extern "C" fn input_osc_10(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut wp: *mut window_pane = (*ictx).wp;
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
    let mut c: ::core::ffi::c_int = 0;
    if strcmp(p, b"?\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        if wp.is_null() {
            return;
        }
        c = window_pane_get_fg_control_client(wp);
        if c == -(1 as ::core::ffi::c_int) {
            tty_default_colours(&raw mut defaults, wp, ::core::ptr::null_mut::<u_int>());
            if defaults.fg == 8 as ::core::ffi::c_int || defaults.fg == 9 as ::core::ffi::c_int {
                c = window_pane_get_fg(wp);
            } else {
                c = defaults.fg;
            }
        }
        input_osc_colour_reply(
            ictx,
            1 as ::core::ffi::c_int,
            10 as u_int,
            0 as ::core::ffi::c_int,
            c,
            (*ictx).input_end,
        );
        return;
    }
    c = colour_parseX11(p);
    if c == -(1 as ::core::ffi::c_int) {
        log_debug(
            b"bad OSC 10: %s\0" as *const u8 as *const ::core::ffi::c_char,
            p,
        );
        return;
    }
    if !(*ictx).palette.is_null() {
        (*(*ictx).palette).fg = c;
        if !wp.is_null() {
            (*wp).flags |= PANE_STYLECHANGED;
        }
        screen_write_fullredraw(&raw mut (*ictx).ctx);
    }
}
unsafe extern "C" fn input_osc_110(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut wp: *mut window_pane = (*ictx).wp;
    if *p as ::core::ffi::c_int != '\0' as i32 {
        return;
    }
    if !(*ictx).palette.is_null() {
        (*(*ictx).palette).fg = 8 as ::core::ffi::c_int;
        if !wp.is_null() {
            (*wp).flags |= PANE_STYLECHANGED;
        }
        screen_write_fullredraw(&raw mut (*ictx).ctx);
    }
}
unsafe extern "C" fn input_osc_11(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut c: ::core::ffi::c_int = 0;
    if strcmp(p, b"?\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        if wp.is_null() {
            return;
        }
        c = window_pane_get_bg(wp);
        input_osc_colour_reply(
            ictx,
            1 as ::core::ffi::c_int,
            11 as u_int,
            0 as ::core::ffi::c_int,
            c,
            (*ictx).input_end,
        );
        return;
    }
    c = colour_parseX11(p);
    if c == -(1 as ::core::ffi::c_int) {
        log_debug(
            b"bad OSC 11: %s\0" as *const u8 as *const ::core::ffi::c_char,
            p,
        );
        return;
    }
    if !(*ictx).palette.is_null() {
        (*(*ictx).palette).bg = c;
        if !wp.is_null() {
            (*wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
        }
        screen_write_fullredraw(&raw mut (*ictx).ctx);
    }
}
unsafe extern "C" fn input_osc_111(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut wp: *mut window_pane = (*ictx).wp;
    if *p as ::core::ffi::c_int != '\0' as i32 {
        return;
    }
    if !(*ictx).palette.is_null() {
        (*(*ictx).palette).bg = 8 as ::core::ffi::c_int;
        if !wp.is_null() {
            (*wp).flags |= PANE_STYLECHANGED | PANE_THEMECHANGED;
        }
        screen_write_fullredraw(&raw mut (*ictx).ctx);
    }
}
unsafe extern "C" fn input_osc_12(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut c: ::core::ffi::c_int = 0;
    if strcmp(p, b"?\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        if !wp.is_null() {
            c = (*(*ictx).ctx.s).ccolour;
            if c == -(1 as ::core::ffi::c_int) {
                c = (*(*ictx).ctx.s).default_ccolour;
            }
            input_osc_colour_reply(
                ictx,
                1 as ::core::ffi::c_int,
                12 as u_int,
                0 as ::core::ffi::c_int,
                c,
                (*ictx).input_end,
            );
        }
        return;
    }
    c = colour_parseX11(p);
    if c == -(1 as ::core::ffi::c_int) {
        log_debug(
            b"bad OSC 12: %s\0" as *const u8 as *const ::core::ffi::c_char,
            p,
        );
        return;
    }
    screen_set_cursor_colour((*ictx).ctx.s, c);
}
unsafe extern "C" fn input_osc_112(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    if *p as ::core::ffi::c_int == '\0' as i32 {
        screen_set_cursor_colour((*ictx).ctx.s, -(1 as ::core::ffi::c_int));
    }
}
unsafe extern "C" fn input_osc_133_exit_status(
    mut p: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut errstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut status: ::core::ffi::c_longlong = 0;
    if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != ';' as i32
        || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
        || strchr(p.offset(2 as ::core::ffi::c_int as isize), '=' as i32)
            == p.offset(2 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char
    {
        return 0 as ::core::ffi::c_int;
    }
    end = strchr(p.offset(2 as ::core::ffi::c_int as isize), ';' as i32);
    if end == p.offset(2 as ::core::ffi::c_int as isize) {
        return 0 as ::core::ffi::c_int;
    }
    if end.is_null() {
        copy = xstrdup(p.offset(2 as ::core::ffi::c_int as isize));
    } else {
        copy = xstrndup(
            p.offset(2 as ::core::ffi::c_int as isize),
            end.offset_from(p.offset(2 as ::core::ffi::c_int as isize)) as ::core::ffi::c_long
                as size_t,
        );
    }
    if !strchr(copy, '=' as i32).is_null() {
        free(copy as *mut ::core::ffi::c_void);
        return 0 as ::core::ffi::c_int;
    }
    status = strtonum(
        copy,
        0 as ::core::ffi::c_longlong,
        255 as ::core::ffi::c_longlong,
        &raw mut errstr,
    );
    free(copy as *mut ::core::ffi::c_void);
    if !errstr.is_null() {
        return 255 as ::core::ffi::c_int;
    }
    return status as ::core::ffi::c_int;
}
unsafe extern "C" fn input_fire_command_event(
    mut wp: *mut window_pane,
    mut name: *const ::core::ffi::c_char,
) {
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
    let mut tstart: time_t = (*wp).cmd_start_time;
    let mut end: time_t = 0;
    let mut tend: time_t = (*wp).cmd_end_time;
    ep = event_payload_create();
    cmd_find_from_pane(&raw mut fs, wp, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    if !fs.s.is_null() {
        event_payload_set_session(
            ep,
            b"session\0" as *const u8 as *const ::core::ffi::c_char,
            fs.s,
        );
    }
    if !fs.wl.is_null() {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*fs.wl).idx,
        );
    }
    event_payload_set_window(
        ep,
        b"window\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).window as *mut window,
    );
    event_payload_set_pane(ep, b"pane\0" as *const u8 as *const ::core::ffi::c_char, wp);
    if (*wp).cmd_status != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            ep,
            b"command_status\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).cmd_status,
        );
    }
    if tstart != 0 as time_t {
        event_payload_set_time(
            ep,
            b"command_start_time\0" as *const u8 as *const ::core::ffi::c_char,
            tstart,
        );
    }
    if tend != 0 as time_t {
        event_payload_set_time(
            ep,
            b"command_end_time\0" as *const u8 as *const ::core::ffi::c_char,
            tend,
        );
    }
    if tstart != 0 as time_t {
        if (*wp).flags & PANE_CMDRUNNING != 0 {
            end = time(::core::ptr::null_mut::<time_t>());
        } else {
            end = tend;
        }
        if end < tstart {
            end = tstart;
        }
        end -= tstart;
        event_payload_set_uint(
            ep,
            b"command_duration\0" as *const u8 as *const ::core::ffi::c_char,
            end as u_int,
        );
    }
    events_fire(name, ep);
}
unsafe extern "C" fn input_osc_133(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut s: *mut screen = (*ictx).ctx.s;
    let mut gd: *mut grid = (*s).grid;
    let mut line: u_int = (*s).cy.wrapping_add((*gd).hsize);
    let mut gl: *mut grid_line = ::core::ptr::null_mut::<grid_line>();
    let mut cp: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut status: ::core::ffi::c_int = 0;
    if line < (*gd).hsize.wrapping_add((*gd).sy) {
        gl = grid_get_line(gd, line);
    }
    match *p as ::core::ffi::c_int {
        65 | 78 => {
            if !gl.is_null() {
                memset(
                    &raw mut (*gl).osc133_data as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    ::core::mem::size_of::<osc133_data>() as size_t,
                );
                (*gl).osc133_data.prompt_col = (*s).cx as u_short;
                (*gl).flags =
                    ((*gl).flags as ::core::ffi::c_int | GRID_LINE_START_PROMPT) as u_short;
            }
            if !wp.is_null() {
                (*wp).last_prompt_time = time(::core::ptr::null_mut::<time_t>());
                events_fire_pane(
                    b"pane-shell-prompt\0" as *const u8 as *const ::core::ffi::c_char,
                    wp,
                );
            }
        }
        80 => {
            if !gl.is_null() {
                cp = strstr(p, b";k=s\0" as *const u8 as *const ::core::ffi::c_char);
                if !cp.is_null()
                    && (*cp.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ';' as i32
                        || *cp.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\0' as i32)
                {
                    (*gl).flags =
                        ((*gl).flags as ::core::ffi::c_int | GRID_LINE_SECOND_PROMPT) as u_short;
                } else {
                    (*gl).flags =
                        ((*gl).flags as ::core::ffi::c_int | GRID_LINE_START_PROMPT) as u_short;
                }
                (*gl).osc133_data.prompt_col = (*s).cx as u_short;
            }
        }
        66 | 73 => {
            if !gl.is_null() {
                (*gl).flags =
                    ((*gl).flags as ::core::ffi::c_int | GRID_LINE_START_COMMAND) as u_short;
                (*gl).osc133_data.cmd_col = (*s).cx as u_short;
            }
        }
        67 => {
            if !gl.is_null() {
                (*gl).flags =
                    ((*gl).flags as ::core::ffi::c_int | GRID_LINE_START_OUTPUT) as u_short;
                (*gl).osc133_data.out_start_col = (*s).cx as u_short;
            }
            if !wp.is_null() {
                (*wp).cmd_start_time = time(::core::ptr::null_mut::<time_t>());
                (*wp).cmd_end_time = 0 as time_t;
                (*wp).flags |= PANE_CMDRUNNING;
                (*wp).cmd_status = -(1 as ::core::ffi::c_int);
                input_fire_command_event(
                    wp,
                    b"pane-command-started\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        68 => {
            status = input_osc_133_exit_status(p);
            if !wp.is_null() {
                (*wp).cmd_end_time = time(::core::ptr::null_mut::<time_t>());
                (*wp).flags &= !PANE_CMDRUNNING;
                (*wp).cmd_status = status;
                input_fire_command_event(
                    wp,
                    b"pane-command-finished\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            if !gl.is_null() {
                (*gl).flags = ((*gl).flags as ::core::ffi::c_int | GRID_LINE_END_OUTPUT) as u_short;
                (*gl).osc133_data.out_end_col = (*s).cx as u_short;
                (*gl).osc133_data.exit_status = status as u_char;
            }
        }
        _ => {}
    };
}
unsafe extern "C" fn input_osc_52_reply(mut ictx: *mut input_ctx, mut clip: ::core::ffi::c_char) {
    let mut ev: *mut bufferevent = (*ictx).event;
    let mut pb: *mut paste_buffer = ::core::ptr::null_mut::<paste_buffer>();
    let mut state: ::core::ffi::c_int = 0;
    let mut buf: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    state = options_get_number(
        global_options,
        b"get-clipboard\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if state == 0 as ::core::ffi::c_int {
        return;
    }
    if state == 1 as ::core::ffi::c_int {
        pb = paste_get_top(::core::ptr::null_mut::<*mut ::core::ffi::c_char>());
        if pb.is_null() {
            return;
        }
        buf = paste_buffer_data(pb, &raw mut len);
        if (*ictx).input_end as ::core::ffi::c_uint
            == INPUT_END_BEL as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            input_reply_clipboard(
                ev,
                buf,
                len,
                b"\x07\0" as *const u8 as *const ::core::ffi::c_char,
                clip,
            );
        } else {
            input_reply_clipboard(
                ev,
                buf,
                len,
                b"\x1B\\\0" as *const u8 as *const ::core::ffi::c_char,
                clip,
            );
        }
        return;
    }
    input_add_request(
        ictx,
        INPUT_REQUEST_CLIPBOARD,
        (*ictx).input_end as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn input_osc_52_parse(
    mut ictx: *mut input_ctx,
    mut p: *const ::core::ffi::c_char,
    mut out: *mut *mut u_char,
    mut outlen: *mut ::core::ffi::c_int,
    mut clip: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut allow: *const ::core::ffi::c_char =
        b"cpqs01234567\0" as *const u8 as *const ::core::ffi::c_char;
    let mut i: u_int = 0;
    let mut j: u_int = 0 as u_int;
    if options_get_number(
        global_options,
        b"set-clipboard\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 2 as ::core::ffi::c_longlong
    {
        return 0 as ::core::ffi::c_int;
    }
    end = strchr(p, ';' as i32);
    if end.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    end = end.offset(1);
    if *end as ::core::ffi::c_int == '\0' as i32 {
        return 0 as ::core::ffi::c_int;
    }
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_osc_52_parse\0" as *const u8 as *const ::core::ffi::c_char,
        end,
    );
    i = 0 as u_int;
    while p.offset(i as isize) != end {
        if !strchr(allow, *p.offset(i as isize) as ::core::ffi::c_int).is_null()
            && strchr(clip, *p.offset(i as isize) as ::core::ffi::c_int).is_null()
        {
            let fresh3 = j;
            j = j.wrapping_add(1);
            *clip.offset(fresh3 as isize) = *p.offset(i as isize);
        }
        i = i.wrapping_add(1);
    }
    log_debug(
        b"%s: %.*s %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_osc_52_parse\0" as *const u8 as *const ::core::ffi::c_char,
        (end.offset_from(p) as ::core::ffi::c_long - 1 as ::core::ffi::c_long)
            as ::core::ffi::c_int,
        p,
        clip,
    );
    if strcmp(end, b"?\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int {
        input_osc_52_reply(ictx, *clip);
        return 0 as ::core::ffi::c_int;
    }
    len = strlen(end)
        .wrapping_add(3 as size_t)
        .wrapping_div(4 as size_t)
        .wrapping_mul(3 as size_t);
    if len == 0 as size_t {
        return 0 as ::core::ffi::c_int;
    }
    *out = xmalloc(len) as *mut u_char;
    *outlen = __b64_pton(end, *out, len);
    if *outlen == -(1 as ::core::ffi::c_int) {
        free(*out as *mut ::core::ffi::c_void);
        *out = ::core::ptr::null_mut::<u_char>();
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_osc_52(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut wp: *mut window_pane = (*ictx).wp;
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
    let mut out: *mut u_char = ::core::ptr::null_mut::<u_char>();
    let mut outlen: ::core::ffi::c_int = 0;
    let mut clip: [::core::ffi::c_char; 13] = ::core::mem::transmute::<
        [u8; 13],
        [::core::ffi::c_char; 13],
    >(*b"\0\0\0\0\0\0\0\0\0\0\0\0\0");
    if input_osc_52_parse(
        ictx,
        p,
        &raw mut out,
        &raw mut outlen,
        &raw mut clip as *mut ::core::ffi::c_char,
    ) == 0
    {
        return;
    }
    if wp.is_null() {
        if (*ictx).c.is_null() {
            free(out as *mut ::core::ffi::c_void);
            return;
        }
        tty_set_selection(
            &raw mut (*(*ictx).c).tty,
            &raw mut clip as *mut ::core::ffi::c_char,
            out as *const ::core::ffi::c_char,
            outlen as size_t,
        );
        paste_add(
            ::core::ptr::null::<::core::ffi::c_char>(),
            out as *mut ::core::ffi::c_char,
            outlen as size_t,
        );
    } else {
        screen_write_start_pane(&raw mut ctx, wp, ::core::ptr::null_mut::<screen>());
        screen_write_setselection(
            &raw mut ctx,
            &raw mut clip as *mut ::core::ffi::c_char,
            out,
            outlen as u_int,
        );
        screen_write_stop(&raw mut ctx);
        events_fire_pane(
            b"pane-set-clipboard\0" as *const u8 as *const ::core::ffi::c_char,
            wp,
        );
        paste_add(
            ::core::ptr::null::<::core::ffi::c_char>(),
            out as *mut ::core::ffi::c_char,
            outlen as size_t,
        );
    };
}
unsafe extern "C" fn input_osc_104(mut ictx: *mut input_ctx, mut p: *const ::core::ffi::c_char) {
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_long = 0;
    let mut bad: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut redraw: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *p as ::core::ffi::c_int == '\0' as i32 {
        colour_palette_clear((*ictx).palette);
        screen_write_fullredraw(&raw mut (*ictx).ctx);
        return;
    }
    s = xstrdup(p);
    copy = s;
    while *s as ::core::ffi::c_int != '\0' as i32 {
        idx = strtol(s, &raw mut s, 10 as ::core::ffi::c_int);
        if *s as ::core::ffi::c_int != '\0' as i32 && *s as ::core::ffi::c_int != ';' as i32 {
            bad = 1 as ::core::ffi::c_int;
            break;
        } else if idx < 0 as ::core::ffi::c_long || idx >= 256 as ::core::ffi::c_long {
            bad = 1 as ::core::ffi::c_int;
            break;
        } else {
            if colour_palette_set(
                (*ictx).palette,
                idx as ::core::ffi::c_int,
                -(1 as ::core::ffi::c_int),
            ) != 0
            {
                redraw = 1 as ::core::ffi::c_int;
            }
            if *s as ::core::ffi::c_int == ';' as i32 {
                s = s.offset(1);
            }
        }
    }
    if bad != 0 {
        log_debug(
            b"bad OSC 104: %s\0" as *const u8 as *const ::core::ffi::c_char,
            p,
        );
    }
    if redraw != 0 {
        screen_write_fullredraw(&raw mut (*ictx).ctx);
    }
    free(copy as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn input_reply_clipboard(
    mut bev: *mut bufferevent,
    mut buf: *const ::core::ffi::c_char,
    mut len: size_t,
    mut end: *const ::core::ffi::c_char,
    mut clip: ::core::ffi::c_char,
) {
    let mut out: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut outlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !buf.is_null() && len != 0 as size_t {
        if len
            >= (INT_MAX as size_t)
                .wrapping_mul(3 as size_t)
                .wrapping_div(4 as size_t)
                .wrapping_sub(1 as size_t)
        {
            return;
        }
        outlen = (4 as size_t)
            .wrapping_mul(len.wrapping_add(2 as size_t).wrapping_div(3 as size_t))
            .wrapping_add(1 as size_t) as ::core::ffi::c_int;
        out = xmalloc(outlen as size_t) as *mut ::core::ffi::c_char;
        outlen = __b64_ntop(
            buf as *const ::core::ffi::c_uchar,
            len,
            out,
            outlen as size_t,
        );
        if outlen == -(1 as ::core::ffi::c_int) {
            free(out as *mut ::core::ffi::c_void);
            return;
        }
    }
    bufferevent_write(
        bev,
        b"\x1B]52;\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        5 as size_t,
    );
    if clip as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        bufferevent_write(
            bev,
            &raw mut clip as *const ::core::ffi::c_void,
            1 as size_t,
        );
    }
    bufferevent_write(
        bev,
        b";\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
        1 as size_t,
    );
    if outlen != 0 as ::core::ffi::c_int {
        bufferevent_write(bev, out as *const ::core::ffi::c_void, outlen as size_t);
    }
    bufferevent_write(bev, end as *const ::core::ffi::c_void, strlen(end));
    free(out as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn input_set_buffer_size(mut buffer_size: size_t) {
    log_debug(
        b"%s: %lu -> %lu\0" as *const u8 as *const ::core::ffi::c_char,
        b"input_set_buffer_size\0" as *const u8 as *const ::core::ffi::c_char,
        input_buffer_size,
        buffer_size,
    );
    input_buffer_size = buffer_size;
}
unsafe extern "C" fn input_request_timer_callback(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut ictx: *mut input_ctx = arg as *mut input_ctx;
    let mut ir: *mut input_request = ::core::ptr::null_mut::<input_request>();
    let mut ir1: *mut input_request = ::core::ptr::null_mut::<input_request>();
    let mut t: uint64_t = get_timer();
    ir = (*ictx).requests.tqh_first;
    while !ir.is_null() && {
        ir1 = (*ir).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !((*ir).t >= t.wrapping_sub(INPUT_REQUEST_TIMEOUT as uint64_t)) {
            if (*ir).type_0 as ::core::ffi::c_uint
                == INPUT_REQUEST_QUEUE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                input_send_reply((*ir).ictx, (*ir).data as *const ::core::ffi::c_char);
            }
            input_free_request(ir);
        }
        ir = ir1;
    }
    if (*ictx).request_count != 0 as u_int {
        input_start_request_timer(ictx);
    }
}
unsafe extern "C" fn input_start_request_timer(mut ictx: *mut input_ctx) {
    let mut tv: timeval = timeval {
        tv_sec: 0 as __time_t,
        tv_usec: 100000 as __suseconds_t,
    };
    event_del(&raw mut (*ictx).request_timer);
    event_add(&raw mut (*ictx).request_timer, &raw mut tv);
}
unsafe extern "C" fn input_make_request(
    mut ictx: *mut input_ctx,
    mut type_0: input_request_type,
) -> *mut input_request {
    let mut ir: *mut input_request = ::core::ptr::null_mut::<input_request>();
    ir = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<input_request>() as size_t,
    ) as *mut input_request;
    (*ir).type_0 = type_0;
    (*ir).ictx = ictx;
    (*ir).t = get_timer();
    (*ictx).request_count = (*ictx).request_count.wrapping_add(1);
    if (*ictx).request_count == 1 as u_int {
        input_start_request_timer(ictx);
    }
    (*ir).entry.tqe_next = ::core::ptr::null_mut::<input_request>();
    (*ir).entry.tqe_prev = (*ictx).requests.tqh_last;
    *(*ictx).requests.tqh_last = ir;
    (*ictx).requests.tqh_last = &raw mut (*ir).entry.tqe_next;
    return ir;
}
unsafe extern "C" fn input_free_request(mut ir: *mut input_request) {
    let mut ictx: *mut input_ctx = (*ir).ictx;
    if !(*ir).c.is_null() {
        if !(*ir).centry.tqe_next.is_null() {
            (*(*ir).centry.tqe_next).centry.tqe_prev = (*ir).centry.tqe_prev;
        } else {
            (*(*ir).c).input_requests.tqh_last = (*ir).centry.tqe_prev;
        }
        *(*ir).centry.tqe_prev = (*ir).centry.tqe_next;
    }
    (*ictx).request_count = (*ictx).request_count.wrapping_sub(1);
    if !(*ir).entry.tqe_next.is_null() {
        (*(*ir).entry.tqe_next).entry.tqe_prev = (*ir).entry.tqe_prev;
    } else {
        (*ictx).requests.tqh_last = (*ir).entry.tqe_prev;
    }
    *(*ir).entry.tqe_prev = (*ir).entry.tqe_next;
    free((*ir).data);
    free(ir as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn input_add_request(
    mut ictx: *mut input_ctx,
    mut type_0: input_request_type,
    mut idx: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut wp: *mut window_pane = (*ictx).wp;
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut ir: *mut input_request = ::core::ptr::null_mut::<input_request>();
    let mut s: [::core::ffi::c_char; 64] = [0; 64];
    if wp.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    w = (*wp).window as *mut window;
    loop_0 = clients.tqh_first;
    while !loop_0.is_null() {
        if !((*loop_0).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
            if !((*loop_0).session.is_null() || session_has((*loop_0).session, w) == 0) {
                if !(!(*loop_0).tty.flags & TTY_STARTED != 0) {
                    if c.is_null() {
                        c = loop_0;
                    } else if if (*loop_0).activity_time.tv_sec == (*c).activity_time.tv_sec {
                        ((*loop_0).activity_time.tv_usec > (*c).activity_time.tv_usec)
                            as ::core::ffi::c_int
                    } else {
                        ((*loop_0).activity_time.tv_sec > (*c).activity_time.tv_sec)
                            as ::core::ffi::c_int
                    } != 0
                    {
                        c = loop_0;
                    }
                }
            }
        }
        loop_0 = (*loop_0).entry.tqe_next;
    }
    if c.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    ir = input_make_request(ictx, type_0);
    (*ir).c = c;
    (*ir).idx = idx;
    (*ir).end = (*ictx).input_end;
    (*ir).centry.tqe_next = ::core::ptr::null_mut::<input_request>();
    (*ir).centry.tqe_prev = (*c).input_requests.tqh_last;
    *(*c).input_requests.tqh_last = ir;
    (*c).input_requests.tqh_last = &raw mut (*ir).centry.tqe_next;
    match type_0 as ::core::ffi::c_uint {
        0 => {
            xsnprintf(
                &raw mut s as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 64]>() as size_t,
                b"\x1B]4;%d;?\x1B\\\0" as *const u8 as *const ::core::ffi::c_char,
                idx,
            );
            tty_puts(&raw mut (*c).tty, &raw mut s as *mut ::core::ffi::c_char);
        }
        1 => {
            tty_putcode_ss(
                &raw mut (*c).tty,
                TTYC_MS,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
                b"?\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        2 | _ => {}
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn input_request_palette_reply(
    mut ir: *mut input_request,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut pd: *mut input_request_palette_data = data as *mut input_request_palette_data;
    input_osc_colour_reply(
        (*ir).ictx,
        0 as ::core::ffi::c_int,
        4 as u_int,
        (*pd).idx,
        (*pd).c,
        (*ir).end,
    );
}
unsafe extern "C" fn input_request_clipboard_reply(
    mut ir: *mut input_request,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut ictx: *mut input_ctx = (*ir).ictx;
    let mut ev: *mut bufferevent = (*ictx).event;
    let mut cd: *mut input_request_clipboard_data = data as *mut input_request_clipboard_data;
    let mut state: ::core::ffi::c_int = 0;
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    state = options_get_number(
        global_options,
        b"get-clipboard\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if state == 0 as ::core::ffi::c_int || state == 1 as ::core::ffi::c_int {
        return;
    }
    if state == 3 as ::core::ffi::c_int {
        copy = xmalloc((*cd).len) as *mut ::core::ffi::c_char;
        memcpy(
            copy as *mut ::core::ffi::c_void,
            (*cd).buf as *const ::core::ffi::c_void,
            (*cd).len,
        );
        paste_add(::core::ptr::null::<::core::ffi::c_char>(), copy, (*cd).len);
    }
    if (*ir).idx == INPUT_END_BEL as ::core::ffi::c_int {
        input_reply_clipboard(
            ev,
            (*cd).buf,
            (*cd).len,
            b"\x07\0" as *const u8 as *const ::core::ffi::c_char,
            (*cd).clip,
        );
    } else {
        input_reply_clipboard(
            ev,
            (*cd).buf,
            (*cd).len,
            b"\x1B\\\0" as *const u8 as *const ::core::ffi::c_char,
            (*cd).clip,
        );
    };
}
#[no_mangle]
pub unsafe extern "C" fn input_request_reply(
    mut c: *mut client,
    mut type_0: input_request_type,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut ir: *mut input_request = ::core::ptr::null_mut::<input_request>();
    let mut ir1: *mut input_request = ::core::ptr::null_mut::<input_request>();
    let mut found: *mut input_request = ::core::ptr::null_mut::<input_request>();
    let mut pd: *mut input_request_palette_data = data as *mut input_request_palette_data;
    let mut complete: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    ir = (*c).input_requests.tqh_first;
    while !ir.is_null() && {
        ir1 = (*ir).centry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if (*ir).type_0 as ::core::ffi::c_uint != type_0 as ::core::ffi::c_uint {
            input_free_request(ir);
        } else if type_0 as ::core::ffi::c_uint
            == INPUT_REQUEST_PALETTE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*pd).idx != (*ir).idx {
                input_free_request(ir);
            } else {
                found = ir;
                break;
            }
        } else if type_0 as ::core::ffi::c_uint
            == INPUT_REQUEST_CLIPBOARD as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            found = ir;
            break;
        }
        ir = ir1;
    }
    if found.is_null() {
        return;
    }
    ir = (*(*found).ictx).requests.tqh_first;
    while !ir.is_null() && {
        ir1 = (*ir).entry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if complete != 0
            && (*ir).type_0 as ::core::ffi::c_uint
                != INPUT_REQUEST_QUEUE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            break;
        }
        if (*ir).type_0 as ::core::ffi::c_uint
            == INPUT_REQUEST_QUEUE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            input_send_reply((*ir).ictx, (*ir).data as *const ::core::ffi::c_char);
        } else if ir == found {
            if (*ir).type_0 as ::core::ffi::c_uint
                == INPUT_REQUEST_PALETTE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                input_request_palette_reply(ir, data);
            } else if (*ir).type_0 as ::core::ffi::c_uint
                == INPUT_REQUEST_CLIPBOARD as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                input_request_clipboard_reply(ir, data);
            }
            complete = 1 as ::core::ffi::c_int;
        }
        input_free_request(ir);
        ir = ir1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn input_cancel_requests(mut c: *mut client) {
    let mut ir: *mut input_request = ::core::ptr::null_mut::<input_request>();
    let mut ir1: *mut input_request = ::core::ptr::null_mut::<input_request>();
    ir = (*c).input_requests.tqh_first;
    while !ir.is_null() && {
        ir1 = (*ir).centry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        input_free_request(ir);
        ir = ir1;
    }
}
unsafe extern "C" fn input_report_current_theme(mut ictx: *mut input_ctx) {
    let mut wp: *mut window_pane = (*ictx).wp;
    if !wp.is_null() {
        (*wp).last_theme = window_pane_get_theme(wp);
        (*wp).flags &= !PANE_THEMECHANGED;
        match (*wp).last_theme as ::core::ffi::c_uint {
            2 => {
                log_debug(
                    b"%s: %%%u dark theme\0" as *const u8 as *const ::core::ffi::c_char,
                    b"input_report_current_theme\0" as *const u8 as *const ::core::ffi::c_char,
                    (*wp).id,
                );
                input_reply(
                    ictx,
                    0 as ::core::ffi::c_int,
                    b"\x1B[?997;1n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            1 => {
                log_debug(
                    b"%s: %%%u light theme\0" as *const u8 as *const ::core::ffi::c_char,
                    b"input_report_current_theme\0" as *const u8 as *const ::core::ffi::c_char,
                    (*wp).id,
                );
                input_reply(
                    ictx,
                    0 as ::core::ffi::c_int,
                    b"\x1B[?997;2n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            0 => {
                log_debug(
                    b"%s: %%%u unknown theme\0" as *const u8 as *const ::core::ffi::c_char,
                    b"input_report_current_theme\0" as *const u8 as *const ::core::ffi::c_char,
                    (*wp).id,
                );
            }
            _ => {}
        }
    }
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

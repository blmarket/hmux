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
    pub type cmdq_state;
    pub type event_payload;
    pub type options_entry;
    pub type tmuxproc;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strsep(
        __stringp: *mut *mut ::core::ffi::c_char,
        __delim: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    fn access(__name: *const ::core::ffi::c_char, __type: ::core::ffi::c_int)
        -> ::core::ffi::c_int;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn ttyname(__fd: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn isatty(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn gettimeofday(__tv: *mut timeval, __tz: *mut ::core::ffi::c_void) -> ::core::ffi::c_int;
    fn sscanf(
        __s: *const ::core::ffi::c_char,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn event_add(ev: *mut event, timeout: *const timeval) -> ::core::ffi::c_int;
    fn event_del(_: *mut event) -> ::core::ffi::c_int;
    fn event_pending(
        ev: *const event,
        events: ::core::ffi::c_short,
        tv: *mut timeval,
    ) -> ::core::ffi::c_int;
    fn event_initialized(ev: *const event) -> ::core::ffi::c_int;
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
    fn evbuffer_add(
        buf: *mut evbuffer,
        data: *const ::core::ffi::c_void,
        datlen: size_t,
    ) -> ::core::ffi::c_int;
    fn evbuffer_readln(
        buffer: *mut evbuffer,
        n_read_out: *mut size_t,
        eol_style: evbuffer_eol_style,
    ) -> *mut ::core::ffi::c_char;
    fn evbuffer_drain(buf: *mut evbuffer, len: size_t) -> ::core::ffi::c_int;
    fn evbuffer_pullup(buf: *mut evbuffer, size: ssize_t) -> *mut ::core::ffi::c_uchar;
    fn bufferevent_enable(
        bufev: *mut bufferevent,
        event: ::core::ffi::c_short,
    ) -> ::core::ffi::c_int;
    fn bufferevent_disable(
        bufev: *mut bufferevent,
        event: ::core::ffi::c_short,
    ) -> ::core::ffi::c_int;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn imsg_get_fd(_: *mut imsg) -> ::core::ffi::c_int;
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xrecallocarray(
        _: *mut ::core::ffi::c_void,
        _: size_t,
        _: size_t,
        _: size_t,
    ) -> *mut ::core::ffi::c_void;
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
    fn checkshell(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn setblocking(_: ::core::ffi::c_int, _: ::core::ffi::c_int);
    fn find_home() -> *const ::core::ffi::c_char;
    fn proc_send(
        _: *mut tmuxpeer,
        _: msgtype,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_void,
        _: size_t,
    ) -> ::core::ffi::c_int;
    fn proc_add_peer(
        _: *mut tmuxproc,
        _: ::core::ffi::c_int,
        _: Option<unsafe extern "C" fn(*mut imsg, *mut ::core::ffi::c_void) -> ()>,
        _: *mut ::core::ffi::c_void,
    ) -> *mut tmuxpeer;
    fn proc_remove_peer(_: *mut tmuxpeer);
    fn proc_kill_peer(_: *mut tmuxpeer);
    static mut cfg_finished: ::core::ffi::c_int;
    static mut cfg_client: *mut client;
    fn start_cfg();
    fn format_create(
        _: *mut client,
        _: *mut cmdq_item,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    ) -> *mut format_tree;
    fn format_free(_: *mut format_tree);
    fn format_expand_time(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn format_expand(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn format_defaults(
        _: *mut format_tree,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: *mut window_pane,
    );
    fn format_lost_client(_: *mut client);
    fn event_payload_create() -> *mut event_payload;
    fn event_payload_set_target(_: *mut event_payload, _: *mut cmd_find_state);
    fn event_payload_set_int(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    );
    fn event_payload_set_uint(_: *mut event_payload, _: *const ::core::ffi::c_char, _: u_int);
    fn event_payload_set_client(
        _: *mut event_payload,
        _: *const ::core::ffi::c_char,
        _: *mut client,
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
    fn events_fire_client(_: *const ::core::ffi::c_char, _: *mut client);
    fn options_get_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn options_get_command(_: *mut options, _: *const ::core::ffi::c_char) -> *mut cmd_list;
    fn options_set_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
    ) -> *mut options_entry;
    fn environ_create() -> *mut environ;
    fn environ_free(_: *mut environ);
    fn environ_find(_: *mut environ, _: *const ::core::ffi::c_char) -> *mut environ_entry;
    fn environ_put(_: *mut environ, _: *const ::core::ffi::c_char, _: ::core::ffi::c_int);
    fn tty_window_offset(
        _: *mut tty,
        _: *mut u_int,
        _: *mut u_int,
        _: *mut u_int,
        _: *mut u_int,
    ) -> ::core::ffi::c_int;
    fn tty_update_client_offset(_: *mut client);
    fn tty_reset(_: *mut tty);
    fn tty_region_off(_: *mut tty);
    fn tty_margin_off(_: *mut tty);
    fn tty_cursor(_: *mut tty, _: u_int, _: u_int);
    fn tty_init(_: *mut tty, _: *mut client) -> ::core::ffi::c_int;
    fn tty_resize(_: *mut tty);
    fn tty_invalidate(_: *mut tty);
    fn tty_start_tty(_: *mut tty);
    fn tty_send_requests(_: *mut tty);
    fn tty_repeat_requests(_: *mut tty, _: ::core::ffi::c_int);
    fn tty_stop_tty(_: *mut tty);
    fn tty_set_title(_: *mut tty, _: *const ::core::ffi::c_char);
    fn tty_set_path(_: *mut tty, _: *const ::core::ffi::c_char);
    fn tty_set_progress_bar(_: *mut tty, _: *mut progress_bar);
    fn tty_update_mode(_: *mut tty, _: ::core::ffi::c_int, _: *mut screen);
    fn tty_sync_end(_: *mut tty);
    fn tty_open(_: *mut tty, _: *mut *mut ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn tty_close(_: *mut tty);
    fn tty_free(_: *mut tty);
    fn tty_term_free_list(_: *mut *mut ::core::ffi::c_char, _: u_int);
    fn tty_term_has(_: *mut tty_term, _: tty_code_code) -> ::core::ffi::c_int;
    fn tty_get_features(_: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn args_from_vector(_: ::core::ffi::c_int, _: *mut *mut ::core::ffi::c_char)
        -> *mut args_value;
    fn args_free_values(_: *mut args_value, _: u_int);
    fn cmd_find_from_client(
        _: *mut cmd_find_state,
        _: *mut client,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_find_from_mouse(
        _: *mut cmd_find_state,
        _: *mut mouse_event,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_unpack_argv(
        _: *mut ::core::ffi::c_char,
        _: size_t,
        _: ::core::ffi::c_int,
        _: *mut *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn cmd_free_argv(_: ::core::ffi::c_int, _: *mut *mut ::core::ffi::c_char);
    fn cmd_list_free(_: *mut cmd_list);
    fn cmd_list_all_have(_: *mut cmd_list, _: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn cmd_parse_from_arguments(
        _: *mut args_value,
        _: u_int,
        _: *mut cmd_parse_input,
    ) -> *mut cmd_parse_result;
    fn cmdq_new() -> *mut cmdq_list;
    fn cmdq_free(_: *mut cmdq_list);
    fn cmdq_get_client(_: *mut cmdq_item) -> *mut client;
    fn cmdq_get_command(_: *mut cmd_list, _: *mut cmdq_state) -> *mut cmdq_item;
    fn cmdq_get_callback1(
        _: *const ::core::ffi::c_char,
        _: cmdq_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut cmdq_item;
    fn cmdq_get_error(_: *const ::core::ffi::c_char) -> *mut cmdq_item;
    fn cmdq_insert_after(_: *mut cmdq_item, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_append(_: *mut client, _: *mut cmdq_item) -> *mut cmdq_item;
    fn cmdq_error(_: *mut cmdq_item, _: *const ::core::ffi::c_char, ...);
    fn key_bindings_get_table(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut key_table;
    fn key_bindings_unref_table(_: *mut key_table);
    fn key_bindings_get(_: *mut key_table, _: key_code) -> *mut key_binding;
    fn key_bindings_dispatch(
        _: *mut key_binding,
        _: *mut cmdq_item,
        _: *mut client,
        _: *mut key_event,
        _: *mut cmd_find_state,
    ) -> *mut cmdq_item;
    fn key_string_lookup_key(_: key_code, _: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn alerts_check_session(_: *mut session);
    fn client_files_RB_MINMAX(_: *mut client_files, _: ::core::ffi::c_int) -> *mut client_file;
    fn client_files_RB_NEXT(_: *mut client_file) -> *mut client_file;
    fn file_fire_done(_: *mut client_file);
    fn file_print(_: *mut client, _: *const ::core::ffi::c_char, ...);
    fn file_write_ready(_: *mut client_files, _: *mut imsg) -> ::core::ffi::c_int;
    fn file_write_done(_: *mut client_files, _: *mut imsg) -> ::core::ffi::c_int;
    fn file_read_data(_: *mut client_files, _: *mut imsg) -> ::core::ffi::c_int;
    fn file_read_done(_: *mut client_files, _: *mut imsg) -> ::core::ffi::c_int;
    static mut server_proc: *mut tmuxproc;
    static mut clients: clients;
    static mut current_time: time_t;
    fn server_update_socket();
    fn server_add_accept(_: ::core::ffi::c_int);
    fn server_redraw_client(_: *mut client);
    fn server_status_client(_: *mut client);
    fn server_redraw_window_borders(_: *mut window);
    fn server_status_window(_: *mut window);
    fn server_kill_pane(_: *mut window_pane);
    fn server_destroy_pane(_: *mut window_pane, _: ::core::ffi::c_int);
    fn server_check_unattached();
    fn status_timer_start(_: *mut client);
    fn status_at_line(_: *mut client) -> ::core::ffi::c_int;
    fn status_line_size(_: *mut client) -> u_int;
    fn status_get_range(_: *mut client, _: u_int, _: u_int) -> *mut style_range;
    fn status_init(_: *mut client);
    fn status_free(_: *mut client);
    fn status_message_clear(_: *mut client);
    fn status_prompt_clear(_: *mut client);
    fn status_prompt_cursor(_: *mut client, _: *mut u_int, _: *mut u_int);
    fn status_prompt_key(_: *mut client, _: key_code, _: *mut mouse_event) -> prompt_key_result;
    fn prompt_free(_: *mut prompt);
    fn resize_window(
        _: *mut window,
        _: u_int,
        _: u_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn recalculate_size(_: *mut window, _: ::core::ffi::c_int);
    fn recalculate_sizes();
    fn input_cancel_requests(_: *mut client);
    fn colour_totheme(_: ::core::ffi::c_int) -> client_theme;
    fn colour_fromstring(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn colour_theme_option(_: u_int, _: client_theme) -> *const ::core::ffi::c_char;
    fn colour_theme_terminal_colour(_: u_int) -> ::core::ffi::c_int;
    fn redraw_screen(_: *mut client);
    fn redraw_pane(_: *mut client, _: *mut window_pane);
    fn redraw_pane_scrollbar(_: *mut client, _: *mut window_pane);
    fn redraw_free_scene(_: *mut redraw_scene);
    fn screen_mode_to_string(_: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    static mut windows: windows;
    static mut all_window_panes: window_pane_tree;
    fn windows_RB_NEXT(_: *mut window) -> *mut window;
    fn windows_RB_MINMAX(_: *mut windows, _: ::core::ffi::c_int) -> *mut window;
    fn window_pane_tree_RB_MINMAX(
        _: *mut window_pane_tree,
        _: ::core::ffi::c_int,
    ) -> *mut window_pane;
    fn window_pane_tree_RB_NEXT(_: *mut window_pane) -> *mut window_pane;
    fn winlink_find_by_index(_: *mut winlinks, _: ::core::ffi::c_int) -> *mut winlink;
    fn window_get_active_at(_: *mut window, _: u_int, _: u_int) -> *mut window_pane;
    fn window_pane_contains(_: *mut window_pane, _: u_int, _: u_int) -> ::core::ffi::c_int;
    fn window_set_active_pane(
        _: *mut window,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn window_update_focus(_: *mut window);
    fn window_redraw_active_switch(_: *mut window, _: *mut window_pane);
    fn window_pane_send_resize(_: *mut window_pane, _: u_int, _: u_int);
    fn window_pane_find_by_id(_: u_int) -> *mut window_pane;
    fn window_pane_clear_resizes(_: *mut window_pane, _: *mut window_pane_resize);
    fn window_pane_set_mode(
        _: *mut window_pane,
        _: *mut window_pane,
        _: *const window_mode,
        _: *mut cmdq_item,
        _: *mut cmd_find_state,
        _: *mut args,
    ) -> ::core::ffi::c_int;
    fn window_pane_key(
        _: *mut window_pane,
        _: *mut client,
        _: *mut session,
        _: *mut winlink,
        _: key_code,
        _: *mut mouse_event,
    ) -> ::core::ffi::c_int;
    fn window_pane_paste(_: *mut window_pane, _: key_code, _: *mut ::core::ffi::c_char, _: size_t);
    fn window_pane_has_prompt(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_prompt_key(
        _: *mut window_pane,
        _: *mut client,
        _: key_code,
        _: *mut mouse_event,
    ) -> prompt_key_result;
    fn window_pane_is_visible(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_get_new_data(
        _: *mut window_pane,
        _: *mut window_pane_offset,
        _: *mut size_t,
    ) -> *mut ::core::ffi::c_void;
    fn window_pane_scrollbar_reserve(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_scrollbar_visible(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_scrollbar_overlay(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_scrollbar_overlay_visible(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_scrollbar_show(_: *mut window_pane, _: ::core::ffi::c_int);
    fn window_pane_scrollbar_start_timer(_: *mut window_pane);
    fn window_pane_send_theme_update(_: *mut window_pane);
    fn window_pane_get_pane_lines(_: *mut window_pane) -> pane_lines;
    fn window_pane_get_pane_status(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_pane_status_get_range(_: *mut window_pane, _: u_int, _: u_int) -> *mut style_range;
    fn window_pane_is_floating(_: *mut window_pane) -> ::core::ffi::c_int;
    fn window_position_is_visible(_: *mut visible_ranges, _: u_int) -> ::core::ffi::c_int;
    fn window_visible_ranges(
        _: *mut window_pane,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: u_int,
        _: *mut visible_ranges,
    ) -> *mut visible_ranges;
    static window_view_mode: window_mode;
    fn window_copy_add(
        _: *mut window_pane,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn check_window_name(_: *mut window);
    fn control_discard(_: *mut client);
    fn control_discard_all(_: *mut client);
    fn control_start(_: *mut client);
    fn control_ready(_: *mut client);
    fn control_stop(_: *mut client);
    fn control_pane_offset(
        _: *mut client,
        _: *mut window_pane,
        _: *mut ::core::ffi::c_int,
    ) -> *mut window_pane_offset;
    fn control_reset_offsets(_: *mut client);
    fn control_write(_: *mut client, _: *const ::core::ffi::c_char, ...);
    fn control_all_done(_: *mut client) -> ::core::ffi::c_int;
    fn session_find_by_id(_: u_int) -> *mut session;
    fn session_update_activity(_: *mut session, _: *mut timeval);
    fn session_theme_changed(_: *mut session);
    fn utf8_stravisx(
        _: *mut *mut ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: size_t,
        _: ::core::ffi::c_int,
    ) -> size_t;
    fn utf8_sanitize(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn log_get_level() -> ::core::ffi::c_int;
    fn log_debug(_: *const ::core::ffi::c_char, ...);
    fn fatal(_: *const ::core::ffi::c_char, ...) -> !;
    fn menu_close(_: *mut window);
    fn menu_screen(_: *mut menu_data) -> *mut screen;
    fn menu_get_cursor(_: *mut menu_data, _: *mut u_int, _: *mut u_int);
    fn menu_key(_: *mut client, _: *mut menu_data, _: *mut key_event) -> ::core::ffi::c_int;
}
pub type __u_char = ::core::ffi::c_uchar;
pub type __u_short = ::core::ffi::c_ushort;
pub type __u_int = ::core::ffi::c_uint;
pub type __uint8_t = u8;
pub type __uint32_t = u32;
pub type __uint64_t = u64;
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
pub type evbuffer_eol_style = ::core::ffi::c_uint;
pub const EVBUFFER_EOL_NUL: evbuffer_eol_style = 4;
pub const EVBUFFER_EOL_LF: evbuffer_eol_style = 3;
pub const EVBUFFER_EOL_CRLF_STRICT: evbuffer_eol_style = 2;
pub const EVBUFFER_EOL_CRLF: evbuffer_eol_style = 1;
pub const EVBUFFER_EOL_ANY: evbuffer_eol_style = 0;
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
pub type key_code_type = ::core::ffi::c_uint;
pub const KEYC_TYPE_NOTYPE: key_code_type = 13;
pub const KEYC_TYPE_TRIPLECLICK: key_code_type = 12;
pub const KEYC_TYPE_DOUBLECLICK: key_code_type = 11;
pub const KEYC_TYPE_SECONDCLICK: key_code_type = 10;
pub const KEYC_TYPE_WHEELUP: key_code_type = 9;
pub const KEYC_TYPE_WHEELDOWN: key_code_type = 8;
pub const KEYC_TYPE_MOUSEDRAGEND: key_code_type = 7;
pub const KEYC_TYPE_MOUSEDRAG: key_code_type = 6;
pub const KEYC_TYPE_MOUSEUP: key_code_type = 5;
pub const KEYC_TYPE_MOUSEDOWN: key_code_type = 4;
pub const KEYC_TYPE_MOUSEMOVE: key_code_type = 3;
pub const KEYC_TYPE_FUNCTION: key_code_type = 2;
pub const KEYC_TYPE_USER: key_code_type = 1;
pub const KEYC_TYPE_UNICODE: key_code_type = 0;
pub type key_code_mouse_location = ::core::ffi::c_uint;
pub const KEYC_MOUSE_LOCATION_NOWHERE: key_code_mouse_location = 20;
pub const KEYC_MOUSE_LOCATION_CONTROL9: key_code_mouse_location = 19;
pub const KEYC_MOUSE_LOCATION_CONTROL8: key_code_mouse_location = 18;
pub const KEYC_MOUSE_LOCATION_CONTROL7: key_code_mouse_location = 17;
pub const KEYC_MOUSE_LOCATION_CONTROL6: key_code_mouse_location = 16;
pub const KEYC_MOUSE_LOCATION_CONTROL5: key_code_mouse_location = 15;
pub const KEYC_MOUSE_LOCATION_CONTROL4: key_code_mouse_location = 14;
pub const KEYC_MOUSE_LOCATION_CONTROL3: key_code_mouse_location = 13;
pub const KEYC_MOUSE_LOCATION_CONTROL2: key_code_mouse_location = 12;
pub const KEYC_MOUSE_LOCATION_CONTROL1: key_code_mouse_location = 11;
pub const KEYC_MOUSE_LOCATION_CONTROL0: key_code_mouse_location = 10;
pub const KEYC_MOUSE_LOCATION_EMPTY: key_code_mouse_location = 9;
pub const KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN: key_code_mouse_location = 8;
pub const KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER: key_code_mouse_location = 7;
pub const KEYC_MOUSE_LOCATION_SCROLLBAR_UP: key_code_mouse_location = 6;
pub const KEYC_MOUSE_LOCATION_BORDER: key_code_mouse_location = 5;
pub const KEYC_MOUSE_LOCATION_STATUS_DEFAULT: key_code_mouse_location = 4;
pub const KEYC_MOUSE_LOCATION_STATUS_RIGHT: key_code_mouse_location = 3;
pub const KEYC_MOUSE_LOCATION_STATUS_LEFT: key_code_mouse_location = 2;
pub const KEYC_MOUSE_LOCATION_STATUS: key_code_mouse_location = 1;
pub const KEYC_MOUSE_LOCATION_PANE: key_code_mouse_location = 0;
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
pub struct window_pane_tree {
    pub rbh_root: *mut window_pane,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct windows {
    pub rbh_root: *mut window,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct environ_entry {
    pub name: *mut ::core::ffi::c_char,
    pub value: *mut ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_37,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
    pub rbe_left: *mut environ_entry,
    pub rbe_right: *mut environ_entry,
    pub rbe_parent: *mut environ_entry,
    pub rbe_color: ::core::ffi::c_int,
}
pub type args_type = ::core::ffi::c_uint;
pub const ARGS_COMMANDS: args_type = 2;
pub const ARGS_STRING: args_type = 1;
pub const ARGS_NONE: args_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct args_value {
    pub type_0: args_type,
    pub c2rust_unnamed: C2RustUnnamed_39,
    pub cached: *mut ::core::ffi::c_char,
    pub entry: C2RustUnnamed_38,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_38 {
    pub tqe_next: *mut args_value,
    pub tqe_prev: *mut *mut args_value,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_39 {
    pub string: *mut ::core::ffi::c_char,
    pub cmdlist: *mut cmd_list,
}
pub type cmd_retval = ::core::ffi::c_int;
pub const CMD_RETURN_STOP: cmd_retval = 2;
pub const CMD_RETURN_WAIT: cmd_retval = 1;
pub const CMD_RETURN_NORMAL: cmd_retval = 0;
pub const CMD_RETURN_ERROR: cmd_retval = -1;
pub type cmd_parse_status = ::core::ffi::c_uint;
pub const CMD_PARSE_SUCCESS: cmd_parse_status = 1;
pub const CMD_PARSE_ERROR: cmd_parse_status = 0;
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
pub type cmdq_cb =
    Option<unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval>;
pub type prompt_key_result = ::core::ffi::c_uint;
pub const PROMPT_KEY_MOVE: prompt_key_result = 3;
pub const PROMPT_KEY_CLOSE: prompt_key_result = 2;
pub const PROMPT_KEY_HANDLED: prompt_key_result = 1;
pub const PROMPT_KEY_NOT_HANDLED: prompt_key_result = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct clients {
    pub tqh_first: *mut client,
    pub tqh_last: *mut *mut client,
}
pub const EINTR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const UINT_MAX: ::core::ffi::c_uint = (__INT_MAX__ as ::core::ffi::c_uint)
    .wrapping_mul(2 as ::core::ffi::c_uint)
    .wrapping_add(1 as ::core::ffi::c_uint);
pub const STDIN_FILENO: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const STDOUT_FILENO: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const STDERR_FILENO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const X_OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const _PATH_BSHELL: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"/bin/sh\0") };
pub const _PATH_TTY: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"/dev/tty\0") };
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const RB_NEGINF: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const EV_TIMEOUT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const EV_READ: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VIS_OCTAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VIS_CSTYLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VIS_NOSLASH: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const IMSG_HEADER_SIZE: usize = ::core::mem::size_of::<imsg_hdr>();
pub const KEYC_META: ::core::ffi::c_ulonglong = 0x100000000000 as ::core::ffi::c_ulonglong;
pub const KEYC_CTRL: ::core::ffi::c_ulonglong = 0x200000000000 as ::core::ffi::c_ulonglong;
pub const KEYC_SHIFT: ::core::ffi::c_ulonglong = 0x400000000000 as ::core::ffi::c_ulonglong;
pub const KEYC_SENT: ::core::ffi::c_ulonglong = 0x40000000000000 as ::core::ffi::c_ulonglong;
pub const KEYC_MASK_TYPE: ::core::ffi::c_ulonglong = 0xff00000000 as ::core::ffi::c_ulonglong;
pub const KEYC_MASK_MODIFIERS: ::core::ffi::c_ulonglong =
    0xff0000000000 as ::core::ffi::c_ulonglong;
pub const KEYC_MASK_KEY: ::core::ffi::c_ulonglong = 0xffffffffff as ::core::ffi::c_ulonglong;
pub const KEYC_CLICK_TIMEOUT: ::core::ffi::c_int = 300 as ::core::ffi::c_int;
pub const KEYC_MOUSE_LOCATION_SHIFT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const KEYC_MOUSE_BUTTON_SHIFT: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const MODE_CURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const MODE_MOUSE_STANDARD: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const MODE_MOUSE_BUTTON: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const MODE_CURSOR_BLINKING: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const MODE_BRACKETPASTE: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const MODE_MOUSE_ALL: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const MODE_CURSOR_VERY_VISIBLE: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const MODE_SYNC: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;
pub const ALL_MOUSE_MODES: ::core::ffi::c_int =
    MODE_MOUSE_STANDARD | MODE_MOUSE_BUTTON | MODE_MOUSE_ALL;
pub const CURSOR_MODES: ::core::ffi::c_int =
    MODE_CURSOR | MODE_CURSOR_BLINKING | MODE_CURSOR_VERY_VISIBLE;
pub const COLOUR_FLAG_THEME: ::core::ffi::c_int = 0x4000000 as ::core::ffi::c_int;
pub const COLOUR_THEME_COUNT: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const PANE_REDRAW: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const PANE_EXITED: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const PANE_STYLECHANGED: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const PANE_REDRAWSCROLLBAR: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
pub const PANE_ACTIVITY: ::core::ffi::c_int = 0x40000 as ::core::ffi::c_int;
pub const PANE_CLOSEONCLICK: ::core::ffi::c_int = 0x80000 as ::core::ffi::c_int;
pub const PANE_CAPTUREALLKEYS: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;
pub const PANE_CLOSEONCANCEL: ::core::ffi::c_int = 0x400000 as ::core::ffi::c_int;
pub const WINDOW_RESIZE: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const WINLINK_BELL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINLINK_ACTIVITY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const WINLINK_SILENCE: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const WINLINK_ALERTFLAGS: ::core::ffi::c_int =
    WINLINK_BELL | WINLINK_ACTIVITY | WINLINK_SILENCE;
pub const WINDOW_SIZE_LATEST: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const PANE_STATUS_OFF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_STATUS_TOP: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_STATUS_BOTTOM: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_MODAL: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_AUTOHIDE: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_RIGHT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const PANE_SCROLLBARS_LEFT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const MOUSE_MASK_BUTTONS: ::core::ffi::c_int = 195 as ::core::ffi::c_int;
pub const MOUSE_MASK_SHIFT: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const MOUSE_MASK_META: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const MOUSE_MASK_CTRL: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const MOUSE_MASK_DRAG: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const MOUSE_WHEEL_UP: ::core::ffi::c_int = 64 as ::core::ffi::c_int;
pub const MOUSE_WHEEL_DOWN: ::core::ffi::c_int = 65 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_2: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_3: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_6: ::core::ffi::c_int = 66 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_7: ::core::ffi::c_int = 67 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_8: ::core::ffi::c_int = 128 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_9: ::core::ffi::c_int = 129 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_10: ::core::ffi::c_int = 130 as ::core::ffi::c_int;
pub const MOUSE_BUTTON_11: ::core::ffi::c_int = 131 as ::core::ffi::c_int;
pub const TTY_NOCURSOR: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const TTY_FREEZE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const TTY_OPENED: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const TTY_BLOCK: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const CMD_READONLY: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CLIENT_PASTE_TIME_LIMIT: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const CLIENT_TERMINAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const CLIENT_EXIT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const CLIENT_REDRAWWINDOW: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSTATUS: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const CLIENT_REPEAT: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const CLIENT_SUSPENDED: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const CLIENT_ATTACHED: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const CLIENT_EXITED: ::core::ffi::c_int = 0x100 as ::core::ffi::c_int;
pub const CLIENT_DEAD: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const CLIENT_REDRAWBORDERS: ::core::ffi::c_int = 0x400 as ::core::ffi::c_int;
pub const CLIENT_READONLY: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CLIENT_FOCUSED: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
pub const CLIENT_UTF8: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const CLIENT_IGNORESIZE: ::core::ffi::c_int = 0x20000 as ::core::ffi::c_int;
pub const CLIENT_IDENTIFIED: ::core::ffi::c_int = 0x40000 as ::core::ffi::c_int;
pub const CLIENT_STATUSFORCE: ::core::ffi::c_int = 0x80000 as ::core::ffi::c_int;
pub const CLIENT_DOUBLECLICK: ::core::ffi::c_int = 0x100000 as ::core::ffi::c_int;
pub const CLIENT_TRIPLECLICK: ::core::ffi::c_int = 0x200000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSTATUSALWAYS: ::core::ffi::c_int = 0x1000000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWOVERLAY: ::core::ffi::c_int = 0x2000000 as ::core::ffi::c_int;
pub const CLIENT_CONTROL_NOOUTPUT: ::core::ffi::c_int = 0x4000000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWMENU: ::core::ffi::c_int = 0x20000000 as ::core::ffi::c_int;
pub const CLIENT_REDRAWSCROLLBARS: ::core::ffi::c_ulonglong =
    0x80000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_CONTROL_PAUSEAFTER: ::core::ffi::c_ulonglong =
    0x100000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_CONTROL_WAITEXIT: ::core::ffi::c_ulonglong =
    0x200000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_CONTROL_NEWLAYOUTS: ::core::ffi::c_ulonglong =
    0x800000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_BRACKETPASTING: ::core::ffi::c_ulonglong =
    0x1000000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_ASSUMEPASTING: ::core::ffi::c_ulonglong = 0x2000000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_NO_DETACH_ON_DESTROY: ::core::ffi::c_ulonglong =
    0x8000000000 as ::core::ffi::c_ulonglong;
pub const CLIENT_ALLREDRAWFLAGS: ::core::ffi::c_int = CLIENT_REDRAWWINDOW
    | CLIENT_REDRAWSTATUS
    | CLIENT_REDRAWSTATUSALWAYS
    | CLIENT_REDRAWBORDERS
    | CLIENT_REDRAWOVERLAY
    | CLIENT_REDRAWMENU;
pub const CLIENT_UNATTACHEDFLAGS: ::core::ffi::c_int = CLIENT_DEAD | CLIENT_SUSPENDED | CLIENT_EXIT;
pub const CLIENT_NODETACHFLAGS: ::core::ffi::c_int = CLIENT_DEAD | CLIENT_EXIT;
pub const KEY_BINDING_REPEAT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const FORMAT_NOJOBS: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const FORMAT_NONE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn server_client_how_many() -> u_int {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut n: u_int = 0;
    n = 0 as u_int;
    c = clients.tqh_first;
    while !c.is_null() {
        if !(*c).session.is_null() && !(*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0 {
            n = n.wrapping_add(1);
        }
        c = (*c).entry.tqe_next;
    }
    return n;
}
unsafe extern "C" fn server_client_overlay_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    server_client_clear_overlay(data as *mut client);
}
#[no_mangle]
pub unsafe extern "C" fn server_client_set_overlay(
    mut c: *mut client,
    mut delay: u_int,
    mut checkcb: overlay_check_cb,
    mut modecb: overlay_mode_cb,
    mut drawcb: overlay_draw_cb,
    mut keycb: overlay_key_cb,
    mut freecb: overlay_free_cb,
    mut resizecb: overlay_resize_cb,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    if (*c).overlay_draw.is_some() {
        server_client_clear_overlay(c);
    }
    tv.tv_sec = delay.wrapping_div(1000 as u_int) as __time_t;
    tv.tv_usec = (delay.wrapping_rem(1000 as u_int) as ::core::ffi::c_long
        * 1000 as ::core::ffi::c_long) as __suseconds_t;
    if event_initialized(&raw mut (*c).overlay_timer) != 0 {
        event_del(&raw mut (*c).overlay_timer);
    }
    event_set(
        &raw mut (*c).overlay_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            server_client_overlay_timer
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        c as *mut ::core::ffi::c_void,
    );
    if delay != 0 as u_int {
        event_add(&raw mut (*c).overlay_timer, &raw mut tv);
    }
    (*c).overlay_check = checkcb;
    (*c).overlay_mode = modecb;
    (*c).overlay_draw = drawcb;
    (*c).overlay_key = keycb;
    (*c).overlay_free = freecb;
    (*c).overlay_resize = resizecb;
    (*c).overlay_data = data;
    if (*c).overlay_check.is_none() {
        (*c).tty.flags |= TTY_FREEZE;
    }
    if (*c).overlay_mode.is_none() {
        (*c).tty.flags |= TTY_NOCURSOR;
    }
    window_update_focus((*(*(*c).session).curw).window);
    server_redraw_client(c);
}
#[no_mangle]
pub unsafe extern "C" fn server_client_clear_overlay(mut c: *mut client) {
    if (*c).overlay_draw.is_none() {
        return;
    }
    if event_initialized(&raw mut (*c).overlay_timer) != 0 {
        event_del(&raw mut (*c).overlay_timer);
    }
    if (*c).overlay_free.is_some() {
        (*c).overlay_free.expect("non-null function pointer")(c, (*c).overlay_data);
    }
    (*c).overlay_check = None;
    (*c).overlay_mode = None;
    (*c).overlay_draw = None;
    (*c).overlay_key = None;
    (*c).overlay_free = None;
    (*c).overlay_resize = None;
    (*c).overlay_data = NULL;
    (*c).tty.flags &= !(TTY_FREEZE | TTY_NOCURSOR);
    if !(*c).session.is_null() {
        window_update_focus((*(*(*c).session).curw).window);
    }
    server_redraw_client(c);
}
#[no_mangle]
pub unsafe extern "C" fn server_client_ranges_is_empty(
    mut r: *mut visible_ranges,
) -> ::core::ffi::c_int {
    let mut i: u_int = 0;
    i = 0 as u_int;
    while i < (*r).used {
        if (*(*r).ranges.offset(i as isize)).nx != 0 as u_int {
            return 0 as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_ensure_ranges(mut r: *mut visible_ranges, mut n: u_int) {
    if (*r).size >= n {
        return;
    }
    (*r).ranges = xrecallocarray(
        (*r).ranges as *mut ::core::ffi::c_void,
        (*r).size as size_t,
        n as size_t,
        ::core::mem::size_of::<visible_range>() as size_t,
    ) as *mut visible_range;
    (*r).size = n;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_overlay_range(
    mut x: u_int,
    mut y: u_int,
    mut sx: u_int,
    mut sy: u_int,
    mut px: u_int,
    mut py: u_int,
    mut nx: u_int,
    mut r: *mut visible_ranges,
) {
    let mut ox: u_int = 0;
    let mut onx: u_int = 0;
    if py < y || py > y.wrapping_add(sy).wrapping_sub(1 as u_int) {
        server_client_ensure_ranges(r, 1 as u_int);
        (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).px = px;
        (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).nx = nx;
        (*r).used = 1 as u_int;
        return;
    }
    server_client_ensure_ranges(r, 2 as u_int);
    if px < x {
        (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).px = px;
        (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).nx = x.wrapping_sub(px);
        if (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).nx > nx {
            (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).nx = nx;
        }
    } else {
        (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).px = 0 as u_int;
        (*(*r).ranges.offset(0 as ::core::ffi::c_int as isize)).nx = 0 as u_int;
    }
    ox = x.wrapping_add(sx);
    if px > ox {
        ox = px;
    }
    onx = px.wrapping_add(nx);
    if onx > ox {
        (*(*r).ranges.offset(1 as ::core::ffi::c_int as isize)).px = ox;
        (*(*r).ranges.offset(1 as ::core::ffi::c_int as isize)).nx = onx.wrapping_sub(ox);
    } else {
        (*(*r).ranges.offset(1 as ::core::ffi::c_int as isize)).px = 0 as u_int;
        (*(*r).ranges.offset(1 as ::core::ffi::c_int as isize)).nx = 0 as u_int;
    }
    (*r).used = 2 as u_int;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_check_nested(mut c: *mut client) -> ::core::ffi::c_int {
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    envent = environ_find(
        (*c).environ,
        b"TMUX\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if envent.is_null() || *(*envent).value as ::core::ffi::c_int == '\0' as i32 {
        return 0 as ::core::ffi::c_int;
    }
    wp = window_pane_tree_RB_MINMAX(&raw mut all_window_panes, RB_NEGINF);
    while !wp.is_null() {
        if strcmp(&raw mut (*wp).tty as *mut ::core::ffi::c_char, (*c).ttyname)
            == 0 as ::core::ffi::c_int
        {
            return 1 as ::core::ffi::c_int;
        }
        wp = window_pane_tree_RB_NEXT(wp);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_set_key_table(
    mut c: *mut client,
    mut name: *const ::core::ffi::c_char,
) {
    if name.is_null() {
        name = server_client_get_key_table(c);
    }
    key_bindings_unref_table((*c).keytable as *mut key_table);
    (*c).keytable = key_bindings_get_table(name, 1 as ::core::ffi::c_int) as *mut key_table;
    (*(*c).keytable).references = (*(*c).keytable).references.wrapping_add(1);
    if gettimeofday(&raw mut (*(*c).keytable).activity_time, NULL) != 0 as ::core::ffi::c_int {
        fatal(b"gettimeofday failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
}
unsafe extern "C" fn server_client_key_table_activity_diff(mut c: *mut client) -> uint64_t {
    let mut diff: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    diff.tv_sec = (*c).activity_time.tv_sec - (*(*c).keytable).activity_time.tv_sec;
    diff.tv_usec = (*c).activity_time.tv_usec - (*(*c).keytable).activity_time.tv_usec;
    if diff.tv_usec < 0 as __suseconds_t {
        diff.tv_sec -= 1;
        diff.tv_usec += 1000000 as __suseconds_t;
    }
    return (diff.tv_sec as ::core::ffi::c_ulonglong)
        .wrapping_mul(1000 as ::core::ffi::c_ulonglong)
        .wrapping_add(
            (diff.tv_usec as ::core::ffi::c_ulonglong)
                .wrapping_div(1000 as ::core::ffi::c_ulonglong),
        ) as uint64_t;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_get_key_table(
    mut c: *mut client,
) -> *const ::core::ffi::c_char {
    let mut s: *mut session = (*c).session;
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if s.is_null() {
        return b"root\0" as *const u8 as *const ::core::ffi::c_char;
    }
    name = options_get_string(
        (*s).options,
        b"key-table\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if *name as ::core::ffi::c_int == '\0' as i32 {
        return b"root\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return name;
}
unsafe extern "C" fn server_client_is_default_key_table(
    mut c: *mut client,
    mut table: *mut key_table,
) -> ::core::ffi::c_int {
    return (strcmp((*table).name, server_client_get_key_table(c)) == 0 as ::core::ffi::c_int)
        as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_create(mut fd: ::core::ffi::c_int) -> *mut client {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut i: u_int = 0;
    setblocking(fd, 0 as ::core::ffi::c_int);
    c = xcalloc(1 as size_t, ::core::mem::size_of::<client>() as size_t) as *mut client;
    (*c).references = 1 as ::core::ffi::c_int;
    (*c).peer = proc_add_peer(
        server_proc,
        fd,
        Some(
            server_client_dispatch
                as unsafe extern "C" fn(*mut imsg, *mut ::core::ffi::c_void) -> (),
        ),
        c as *mut ::core::ffi::c_void,
    );
    if gettimeofday(&raw mut (*c).creation_time, NULL) != 0 as ::core::ffi::c_int {
        fatal(b"gettimeofday failed\0" as *const u8 as *const ::core::ffi::c_char);
    }
    memcpy(
        &raw mut (*c).activity_time as *mut ::core::ffi::c_void,
        &raw mut (*c).creation_time as *const ::core::ffi::c_void,
        ::core::mem::size_of::<timeval>() as size_t,
    );
    (*c).environ = environ_create();
    (*c).fd = -(1 as ::core::ffi::c_int);
    (*c).out_fd = -(1 as ::core::ffi::c_int);
    (*c).queue = cmdq_new();
    (*c).files.rbh_root = ::core::ptr::null_mut::<client_file>();
    (*c).tty.sx = 80 as u_int;
    (*c).tty.sy = 24 as u_int;
    i = 0 as u_int;
    while i < COLOUR_THEME_COUNT as u_int {
        (*c).theme_colours[i as usize] = 8 as ::core::ffi::c_int;
        i = i.wrapping_add(1);
    }
    (*c).theme = THEME_UNKNOWN;
    status_init(c);
    (*c).flags |= CLIENT_FOCUSED as uint64_t;
    (*c).keytable = key_bindings_get_table(
        b"root\0" as *const u8 as *const ::core::ffi::c_char,
        1 as ::core::ffi::c_int,
    ) as *mut key_table;
    (*(*c).keytable).references = (*(*c).keytable).references.wrapping_add(1);
    event_set(
        &raw mut (*c).repeat_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            server_client_repeat_timer
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        c as *mut ::core::ffi::c_void,
    );
    event_set(
        &raw mut (*c).click_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            server_client_click_timer
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        c as *mut ::core::ffi::c_void,
    );
    event_set(
        &raw mut (*c).exit_timer,
        -(1 as ::core::ffi::c_int),
        0 as ::core::ffi::c_short,
        Some(
            server_client_exit_timer
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_short,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        c as *mut ::core::ffi::c_void,
    );
    (*c).click_wp = -(1 as ::core::ffi::c_int);
    (*c).input_requests.tqh_first = ::core::ptr::null_mut::<input_request>();
    (*c).input_requests.tqh_last = &raw mut (*c).input_requests.tqh_first;
    (*c).entry.tqe_next = ::core::ptr::null_mut::<client>();
    (*c).entry.tqe_prev = clients.tqh_last;
    *clients.tqh_last = c;
    clients.tqh_last = &raw mut (*c).entry.tqe_next;
    log_debug(
        b"new client %p\0" as *const u8 as *const ::core::ffi::c_char,
        c,
    );
    return c;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_open(
    mut c: *mut client,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut ttynam: *const ::core::ffi::c_char = _PATH_TTY.as_ptr();
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if strcmp((*c).ttyname, ttynam) == 0 as ::core::ffi::c_int
        || (isatty(STDIN_FILENO) != 0
            && {
                ttynam = ttyname(STDIN_FILENO);
                !ttynam.is_null()
            }
            && strcmp((*c).ttyname, ttynam) == 0 as ::core::ffi::c_int
            || isatty(STDOUT_FILENO) != 0
                && {
                    ttynam = ttyname(STDOUT_FILENO);
                    !ttynam.is_null()
                }
                && strcmp((*c).ttyname, ttynam) == 0 as ::core::ffi::c_int
            || isatty(STDERR_FILENO) != 0
                && {
                    ttynam = ttyname(STDERR_FILENO);
                    !ttynam.is_null()
                }
                && strcmp((*c).ttyname, ttynam) == 0 as ::core::ffi::c_int)
    {
        xasprintf(
            cause,
            b"can't use %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).ttyname,
        );
        return -(1 as ::core::ffi::c_int);
    }
    if (*c).flags & CLIENT_TERMINAL as uint64_t == 0 {
        *cause = xstrdup(b"not a terminal\0" as *const u8 as *const ::core::ffi::c_char);
        return -(1 as ::core::ffi::c_int);
    }
    if tty_open(&raw mut (*c).tty, cause) != 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    server_client_update_theme_colours(c);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_attached_lost(mut c: *mut client) {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut loop_0: *mut client = ::core::ptr::null_mut::<client>();
    let mut found: *mut client = ::core::ptr::null_mut::<client>();
    log_debug(
        b"lost attached client %p\0" as *const u8 as *const ::core::ffi::c_char,
        c,
    );
    w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
    while !w.is_null() {
        if !((*w).latest != c as *mut ::core::ffi::c_void) {
            found = ::core::ptr::null_mut::<client>();
            loop_0 = clients.tqh_first;
            while !loop_0.is_null() {
                s = (*loop_0).session;
                if !(loop_0 == c || s.is_null() || (*(*s).curw).window != w) {
                    if found.is_null()
                        || (if (*loop_0).activity_time.tv_sec == (*found).activity_time.tv_sec {
                            ((*loop_0).activity_time.tv_usec > (*found).activity_time.tv_usec)
                                as ::core::ffi::c_int
                        } else {
                            ((*loop_0).activity_time.tv_sec > (*found).activity_time.tv_sec)
                                as ::core::ffi::c_int
                        }) != 0
                    {
                        found = loop_0;
                    }
                }
                loop_0 = (*loop_0).entry.tqe_next;
            }
            if !found.is_null() {
                server_client_update_latest(found);
            }
        }
        w = windows_RB_NEXT(w);
    }
}
unsafe extern "C" fn server_client_fire_session_changed(mut c: *mut client, mut old: *mut session) {
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
    cmd_find_from_client(&raw mut fs, c, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_client(
        ep,
        b"client\0" as *const u8 as *const ::core::ffi::c_char,
        c,
    );
    if !fs.s.is_null() {
        event_payload_set_session(
            ep,
            b"session\0" as *const u8 as *const ::core::ffi::c_char,
            fs.s,
        );
        event_payload_set_session(
            ep,
            b"new_session\0" as *const u8 as *const ::core::ffi::c_char,
            fs.s,
        );
    }
    if !old.is_null() {
        event_payload_set_session(
            ep,
            b"old_session\0" as *const u8 as *const ::core::ffi::c_char,
            old,
        );
    }
    if !fs.w.is_null() {
        event_payload_set_window(
            ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            fs.w,
        );
    }
    if !fs.wl.is_null() {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*fs.wl).idx,
        );
    } else if fs.idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            fs.idx,
        );
    }
    if !fs.wp.is_null() {
        event_payload_set_pane(
            ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            fs.wp,
        );
    }
    events_fire(
        b"client-session-changed\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
unsafe extern "C" fn server_client_fire_resized(
    mut c: *mut client,
    mut old_sx: u_int,
    mut old_sy: u_int,
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
    cmd_find_from_client(&raw mut fs, c, 0 as ::core::ffi::c_int);
    event_payload_set_target(ep, &raw mut fs);
    event_payload_set_client(
        ep,
        b"client\0" as *const u8 as *const ::core::ffi::c_char,
        c,
    );
    if !fs.s.is_null() {
        event_payload_set_session(
            ep,
            b"session\0" as *const u8 as *const ::core::ffi::c_char,
            fs.s,
        );
    }
    if !fs.w.is_null() {
        event_payload_set_window(
            ep,
            b"window\0" as *const u8 as *const ::core::ffi::c_char,
            fs.w,
        );
    }
    if !fs.wl.is_null() {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            (*fs.wl).idx,
        );
    } else if fs.idx != -(1 as ::core::ffi::c_int) {
        event_payload_set_int(
            ep,
            b"window_index\0" as *const u8 as *const ::core::ffi::c_char,
            fs.idx,
        );
    }
    if !fs.wp.is_null() {
        event_payload_set_pane(
            ep,
            b"pane\0" as *const u8 as *const ::core::ffi::c_char,
            fs.wp,
        );
    }
    event_payload_set_uint(
        ep,
        b"width\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).tty.sx,
    );
    event_payload_set_uint(
        ep,
        b"height\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).tty.sy,
    );
    event_payload_set_uint(
        ep,
        b"old_width\0" as *const u8 as *const ::core::ffi::c_char,
        old_sx,
    );
    event_payload_set_uint(
        ep,
        b"old_height\0" as *const u8 as *const ::core::ffi::c_char,
        old_sy,
    );
    events_fire(
        b"client-resized\0" as *const u8 as *const ::core::ffi::c_char,
        ep,
    );
}
#[no_mangle]
pub unsafe extern "C" fn server_client_set_session(mut c: *mut client, mut s: *mut session) {
    let mut old: *mut session = (*c).session;
    if !s.is_null() && !(*c).session.is_null() && (*c).session != s {
        (*c).last_session = (*c).session;
    } else if s.is_null() {
        (*c).last_session = ::core::ptr::null_mut::<session>();
    }
    (*c).session = s;
    (*c).flags |= CLIENT_FOCUSED as uint64_t;
    if !old.is_null() && !(*old).curw.is_null() {
        window_update_focus((*(*old).curw).window);
    }
    if !s.is_null() {
        (*(*(*s).curw).window).latest = c as *mut ::core::ffi::c_void;
        recalculate_sizes();
        window_update_focus((*(*s).curw).window);
        session_update_activity(s, ::core::ptr::null_mut::<timeval>());
        session_theme_changed(s);
        gettimeofday(&raw mut (*s).last_attached_time, NULL);
        (*(*s).curw).flags &= !WINLINK_ALERTFLAGS;
        alerts_check_session(s);
        tty_update_client_offset(c);
        status_timer_start(c);
        server_client_fire_session_changed(c, old);
        server_redraw_client(c);
    }
    server_check_unattached();
    server_update_socket();
}
#[no_mangle]
pub unsafe extern "C" fn server_client_lost(mut c: *mut client) {
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut cf1: *mut client_file = ::core::ptr::null_mut::<client_file>();
    if cfg_client == c {
        cfg_client = ::core::ptr::null_mut::<client>();
    }
    (*c).flags |= CLIENT_DEAD as uint64_t;
    server_client_clear_overlay(c);
    status_prompt_clear(c);
    status_message_clear(c);
    cf = client_files_RB_MINMAX(&raw mut (*c).files, RB_NEGINF);
    while !cf.is_null() && {
        cf1 = client_files_RB_NEXT(cf);
        1 as ::core::ffi::c_int != 0
    } {
        (*cf).error = EINTR;
        file_fire_done(cf);
        cf = cf1;
    }
    if !(*c).entry.tqe_next.is_null() {
        (*(*c).entry.tqe_next).entry.tqe_prev = (*c).entry.tqe_prev;
    } else {
        clients.tqh_last = (*c).entry.tqe_prev;
    }
    *(*c).entry.tqe_prev = (*c).entry.tqe_next;
    log_debug(
        b"lost client %p\0" as *const u8 as *const ::core::ffi::c_char,
        c,
    );
    if (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
        server_client_attached_lost(c);
        events_fire_client(
            b"client-detached\0" as *const u8 as *const ::core::ffi::c_char,
            c,
        );
    }
    if !(*c).name.is_null() && (*c).flags & (CLIENT_CONTROL | CLIENT_TERMINAL) as uint64_t != 0 {
        events_fire_client(
            b"client-closed\0" as *const u8 as *const ::core::ffi::c_char,
            c,
        );
    }
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        control_stop(c);
    }
    if (*c).flags & CLIENT_TERMINAL as uint64_t != 0 {
        tty_free(&raw mut (*c).tty);
    }
    free((*c).ttyname as *mut ::core::ffi::c_void);
    free((*c).clipboard_panes as *mut ::core::ffi::c_void);
    free((*c).term_name as *mut ::core::ffi::c_void);
    free((*c).term_type as *mut ::core::ffi::c_void);
    tty_term_free_list((*c).term_caps, (*c).term_ncaps);
    status_free(c);
    input_cancel_requests(c);
    free((*c).title as *mut ::core::ffi::c_void);
    free((*c).path as *mut ::core::ffi::c_void);
    free((*c).cwd as *mut ::core::ffi::c_void);
    free((*c).exit_session as *mut ::core::ffi::c_void);
    free((*c).exit_message as *mut ::core::ffi::c_void);
    event_del(&raw mut (*c).repeat_timer);
    event_del(&raw mut (*c).click_timer);
    event_del(&raw mut (*c).exit_timer);
    if event_initialized(&raw mut (*c).cycle_timer) != 0 {
        event_del(&raw mut (*c).cycle_timer);
    }
    key_bindings_unref_table((*c).keytable as *mut key_table);
    free((*c).message_string as *mut ::core::ffi::c_void);
    if event_initialized(&raw mut (*c).message_timer) != 0 {
        event_del(&raw mut (*c).message_timer);
    }
    prompt_free((*c).prompt);
    format_lost_client(c);
    environ_free((*c).environ);
    proc_remove_peer((*c).peer);
    (*c).peer = ::core::ptr::null_mut::<tmuxpeer>();
    if (*c).out_fd != -(1 as ::core::ffi::c_int) {
        close((*c).out_fd);
    }
    if (*c).fd != -(1 as ::core::ffi::c_int) {
        close((*c).fd);
        (*c).fd = -(1 as ::core::ffi::c_int);
    }
    server_client_unref(c);
    server_add_accept(0 as ::core::ffi::c_int);
    recalculate_sizes();
    server_check_unattached();
    server_update_socket();
}
#[no_mangle]
pub unsafe extern "C" fn server_client_unref(mut c: *mut client) {
    log_debug(
        b"unref client %p (%d references)\0" as *const u8 as *const ::core::ffi::c_char,
        c,
        (*c).references,
    );
    (*c).references -= 1;
    if (*c).references == 0 as ::core::ffi::c_int {
        event_once(
            -(1 as ::core::ffi::c_int),
            EV_TIMEOUT as ::core::ffi::c_short,
            Some(
                server_client_free
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            c as *mut ::core::ffi::c_void,
            ::core::ptr::null::<timeval>(),
        );
    }
}
unsafe extern "C" fn server_client_free(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = arg as *mut client;
    log_debug(
        b"free client %p (%d references)\0" as *const u8 as *const ::core::ffi::c_char,
        c,
        (*c).references,
    );
    redraw_free_scene((*c).redraw_scene);
    cmdq_free((*c).queue);
    if (*c).references == 0 as ::core::ffi::c_int {
        free((*c).name as *mut ::core::ffi::c_void);
        free((*c).user as *mut ::core::ffi::c_void);
        free(c as *mut ::core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_client_suspend(mut c: *mut client) {
    let mut s: *mut session = (*c).session;
    if s.is_null() || (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0 {
        return;
    }
    tty_stop_tty(&raw mut (*c).tty);
    (*c).flags |= CLIENT_SUSPENDED as uint64_t;
    proc_send(
        (*c).peer,
        MSG_SUSPEND,
        -(1 as ::core::ffi::c_int),
        ::core::ptr::null::<::core::ffi::c_void>(),
        0 as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn server_client_detach(mut c: *mut client, mut msgtype: msgtype) {
    let mut s: *mut session = (*c).session;
    if s.is_null() || (*c).flags & CLIENT_NODETACHFLAGS as uint64_t != 0 {
        return;
    }
    (*c).flags |= CLIENT_EXIT as uint64_t;
    (*c).exit_type = CLIENT_EXIT_DETACH;
    (*c).exit_msgtype = msgtype;
    (*c).exit_session = xstrdup((*s).name);
}
#[no_mangle]
pub unsafe extern "C" fn server_client_exec(
    mut c: *mut client,
    mut cmd: *const ::core::ffi::c_char,
) {
    let mut s: *mut session = (*c).session;
    let mut msg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut shell: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut cmdsize: size_t = 0;
    let mut shellsize: size_t = 0;
    if *cmd as ::core::ffi::c_int == '\0' as i32 {
        return;
    }
    cmdsize = strlen(cmd).wrapping_add(1 as size_t);
    if !s.is_null() {
        shell = options_get_string(
            (*s).options,
            b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        shell = options_get_string(
            global_s_options,
            b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if checkshell(shell) == 0 {
        shell = _PATH_BSHELL.as_ptr();
    }
    shellsize = strlen(shell).wrapping_add(1 as size_t);
    msg = xmalloc(cmdsize.wrapping_add(shellsize)) as *mut ::core::ffi::c_char;
    memcpy(
        msg as *mut ::core::ffi::c_void,
        cmd as *const ::core::ffi::c_void,
        cmdsize,
    );
    memcpy(
        msg.offset(cmdsize as isize) as *mut ::core::ffi::c_void,
        shell as *const ::core::ffi::c_void,
        shellsize,
    );
    proc_send(
        (*c).peer,
        MSG_EXEC,
        -(1 as ::core::ffi::c_int),
        msg as *const ::core::ffi::c_void,
        cmdsize.wrapping_add(shellsize),
    );
    free(msg as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn server_client_in_scrollbar_area(
    mut wp: *mut window_pane,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut width: u_int = 0;
    let mut pad: u_int = 0;
    let mut total: u_int = 0;
    let mut start: ::core::ffi::c_int = 0;
    let mut end: ::core::ffi::c_int = 0;
    if window_pane_scrollbar_overlay(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    if py < (*wp).yoff || py >= (*wp).yoff + (*wp).sy as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    width = (*wp).scrollbar_style.width as u_int;
    pad = (*wp).scrollbar_style.pad as u_int;
    total = width.wrapping_add(pad);
    if total == 0 as u_int || total > (*wp).sx {
        total = (*wp).sx;
    }
    if (*w).sb_pos == PANE_SCROLLBARS_LEFT {
        start = (*wp).xoff;
        end = (*wp).xoff + total as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
    } else {
        end = (*wp).xoff + (*wp).sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        start = end - total as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    }
    return (px >= start && px <= end) as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_update_scrollbar_hover(
    mut c: *mut client,
    mut type_0: ::core::ffi::c_int,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
) {
    let mut w: *mut window = (*(*(*c).session).curw).window;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if type_0 != KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int {
        return;
    }
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if !(window_pane_is_visible(wp) == 0) {
            if server_client_in_scrollbar_area(wp, px, py) != 0 {
                (*wp).sb_auto_hover = 1 as ::core::ffi::c_int;
                window_pane_scrollbar_show(wp, 1 as ::core::ffi::c_int);
            } else {
                (*wp).sb_auto_hover = 0 as ::core::ffi::c_int;
                window_pane_scrollbar_start_timer(wp);
            }
        }
        wp = (*wp).entry.tqe_next;
    }
}
unsafe extern "C" fn server_client_check_mouse_in_pane(
    mut wp: *mut window_pane,
    mut px: ::core::ffi::c_int,
    mut py: ::core::ffi::c_int,
    mut sl_mpos: *mut u_int,
) -> key_code_mouse_location {
    let mut w: *mut window = (*wp).window as *mut window;
    let mut fwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut pane_status: ::core::ffi::c_int = 0;
    let mut sb_w: ::core::ffi::c_int = 0;
    let mut sb_pad: ::core::ffi::c_int = 0;
    let mut pane_status_line: ::core::ffi::c_int = 0;
    let mut sl_top: ::core::ffi::c_int = 0;
    let mut sl_bottom: ::core::ffi::c_int = 0;
    let mut bdr_bottom: ::core::ffi::c_int = 0;
    let mut bdr_top: ::core::ffi::c_int = 0;
    let mut bdr_left: ::core::ffi::c_int = 0;
    let mut bdr_right: ::core::ffi::c_int = 0;
    let mut sb_start: ::core::ffi::c_int = 0;
    let mut sb_end: ::core::ffi::c_int = 0;
    let mut sb_overlay: ::core::ffi::c_int = 0;
    pane_status = window_pane_get_pane_status(wp);
    sb_overlay = window_pane_scrollbar_overlay(wp);
    if window_pane_scrollbar_visible(wp) != 0 {
        sb_w = (*wp).scrollbar_style.width;
        sb_pad = (*wp).scrollbar_style.pad;
        if sb_overlay != 0 && sb_w > (*wp).sx as ::core::ffi::c_int {
            sb_w = (*wp).sx as ::core::ffi::c_int;
        }
    } else {
        sb_w = 0 as ::core::ffi::c_int;
        sb_pad = 0 as ::core::ffi::c_int;
    }
    if pane_status == PANE_STATUS_TOP {
        pane_status_line = (*wp).yoff - 1 as ::core::ffi::c_int;
    } else if pane_status == PANE_STATUS_BOTTOM {
        pane_status_line = ((*wp).yoff as u_int).wrapping_add((*wp).sy) as ::core::ffi::c_int;
    } else {
        pane_status_line = -(1 as ::core::ffi::c_int);
    }
    bdr_left = (*wp).xoff - 1 as ::core::ffi::c_int;
    if sb_overlay == 0 && (*w).sb_pos == PANE_SCROLLBARS_LEFT {
        bdr_left -= sb_pad + sb_w;
    }
    if sb_overlay != 0
        && sb_w != 0 as ::core::ffi::c_int
        && py >= (*wp).yoff
        && py < (*wp).yoff + (*wp).sy as ::core::ffi::c_int
        && px >= (*wp).xoff
        && px < (*wp).xoff + (*wp).sx as ::core::ffi::c_int
    {
        if (*w).sb_pos == PANE_SCROLLBARS_LEFT {
            sb_start = (*wp).xoff;
            sb_end = sb_start + sb_w - 1 as ::core::ffi::c_int;
        } else {
            sb_end = (*wp).xoff + (*wp).sx as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
            sb_start = sb_end - sb_w + 1 as ::core::ffi::c_int;
        }
        if px >= sb_start && px <= sb_end {
            sl_top = ((*wp).yoff as u_int).wrapping_add((*wp).sb_slider_y) as ::core::ffi::c_int;
            sl_bottom = ((*wp).yoff as u_int)
                .wrapping_add((*wp).sb_slider_y)
                .wrapping_add((*wp).sb_slider_h)
                .wrapping_sub(1 as u_int) as ::core::ffi::c_int;
            if py < sl_top {
                return KEYC_MOUSE_LOCATION_SCROLLBAR_UP;
            } else if py >= sl_top && py <= sl_bottom {
                *sl_mpos = (py as u_int)
                    .wrapping_sub((*wp).sb_slider_y)
                    .wrapping_sub((*wp).yoff as u_int);
                return KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER;
            } else {
                return KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN;
            }
        }
        return KEYC_MOUSE_LOCATION_PANE;
    }
    if (pane_status != PANE_STATUS_OFF
        && py != pane_status_line
        && py != (*wp).yoff + (*wp).sy as ::core::ffi::c_int
        || (*wp).yoff == 0 as ::core::ffi::c_int && py < (*wp).sy as ::core::ffi::c_int
        || py >= (*wp).yoff && py < (*wp).yoff + (*wp).sy as ::core::ffi::c_int)
        && ((*w).sb_pos == PANE_SCROLLBARS_RIGHT
            && px < (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_pad + sb_w
            || (*w).sb_pos == PANE_SCROLLBARS_LEFT
                && px < (*wp).xoff + (*wp).sx as ::core::ffi::c_int - sb_pad - sb_w)
    {
        if (*w).sb_pos == PANE_SCROLLBARS_RIGHT
            && (px >= (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_pad
                && px < (*wp).xoff + (*wp).sx as ::core::ffi::c_int + sb_pad + sb_w)
            || (*w).sb_pos == PANE_SCROLLBARS_LEFT
                && (px >= (*wp).xoff - sb_pad - sb_w && px < (*wp).xoff - sb_pad)
        {
            sl_top = ((*wp).yoff as u_int).wrapping_add((*wp).sb_slider_y) as ::core::ffi::c_int;
            sl_bottom = ((*wp).yoff as u_int)
                .wrapping_add((*wp).sb_slider_y)
                .wrapping_add((*wp).sb_slider_h)
                .wrapping_sub(1 as u_int) as ::core::ffi::c_int;
            if py < sl_top {
                return KEYC_MOUSE_LOCATION_SCROLLBAR_UP;
            } else if py >= sl_top && py <= sl_bottom {
                *sl_mpos = (py as u_int)
                    .wrapping_sub((*wp).sb_slider_y)
                    .wrapping_sub((*wp).yoff as u_int);
                return KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER;
            } else {
                return KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN;
            }
        } else if window_pane_is_floating(wp) != 0
            && window_pane_get_pane_lines(wp) as ::core::ffi::c_uint
                != PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
            && (px == bdr_left
                || py == (*wp).yoff - 1 as ::core::ffi::c_int
                || py == (*wp).yoff + (*wp).sy as ::core::ffi::c_int)
        {
            return KEYC_MOUSE_LOCATION_BORDER;
        } else {
            return KEYC_MOUSE_LOCATION_PANE;
        }
    } else {
        fwp = (*w).panes.tqh_first;
        while !fwp.is_null() {
            if !(window_pane_is_visible(fwp) == 0) {
                if !(window_pane_is_floating(fwp) != 0
                    && window_pane_get_pane_lines(fwp) as ::core::ffi::c_uint
                        == PANE_LINES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    if window_pane_scrollbar_reserve(fwp) != 0 {
                        sb_w = (*fwp).scrollbar_style.width;
                        sb_pad = (*fwp).scrollbar_style.pad;
                    } else {
                        sb_w = 0 as ::core::ffi::c_int;
                        sb_pad = 0 as ::core::ffi::c_int;
                    }
                    bdr_top = (*fwp).yoff - 1 as ::core::ffi::c_int;
                    bdr_bottom =
                        ((*fwp).yoff as u_int).wrapping_add((*fwp).sy) as ::core::ffi::c_int;
                    bdr_left = (*fwp).xoff - 1 as ::core::ffi::c_int;
                    if (*w).sb_pos == PANE_SCROLLBARS_LEFT {
                        bdr_left -= sb_pad + sb_w;
                        bdr_right =
                            ((*fwp).xoff as u_int).wrapping_add((*fwp).sx) as ::core::ffi::c_int;
                    } else {
                        bdr_right = ((*fwp).xoff as u_int)
                            .wrapping_add((*fwp).sx)
                            .wrapping_add(sb_pad as u_int)
                            .wrapping_add(sb_w as u_int)
                            as ::core::ffi::c_int;
                    }
                    if py >= (*fwp).yoff - 1 as ::core::ffi::c_int
                        && py <= (*fwp).yoff + (*fwp).sy as ::core::ffi::c_int
                    {
                        if px == bdr_right {
                            break;
                        }
                        if window_pane_is_floating(wp) != 0 {
                            if px == bdr_left {
                                break;
                            }
                        }
                    }
                    if px >= bdr_left && px <= (*fwp).xoff + (*fwp).sx as ::core::ffi::c_int {
                        bdr_bottom =
                            ((*fwp).yoff as u_int).wrapping_add((*fwp).sy) as ::core::ffi::c_int;
                        if py == bdr_bottom {
                            break;
                        }
                        if py == bdr_top {
                            break;
                        }
                    }
                }
            }
            fwp = (*fwp).entry.tqe_next;
        }
        if !fwp.is_null() {
            return KEYC_MOUSE_LOCATION_BORDER;
        }
    }
    return KEYC_MOUSE_LOCATION_NOWHERE;
}
unsafe extern "C" fn server_client_check_mouse(
    mut c: *mut client,
    mut event: *mut key_event,
) -> key_code {
    let mut current_block: u64;
    let mut m: *mut mouse_event = &raw mut (*event).m;
    let mut s: *mut session = (*c).session;
    let mut fs: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window = (*(*s).curw).window;
    let mut fwl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut fwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut lwp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut px: u_int = 0;
    let mut py: u_int = 0;
    let mut n: u_int = 0;
    let mut sl_mpos: u_int = 0 as u_int;
    let mut b: u_int = 0;
    let mut bn: u_int = 0;
    let mut ignore: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut modal_drag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut key: key_code = 0;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut sr: *mut style_range = ::core::ptr::null_mut::<style_range>();
    let mut type_0: key_code_type = KEYC_TYPE_NOTYPE;
    let mut loc: key_code_mouse_location = KEYC_MOUSE_LOCATION_NOWHERE;
    log_debug(
        b"%s mouse %02x at %u,%u (last %u,%u) (%d)\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        (*m).b,
        (*m).x,
        (*m).y,
        (*m).lx,
        (*m).ly,
        (*c).tty.mouse_drag_flag,
    );
    if (*c).tty.mouse_last_pane != -(1 as ::core::ffi::c_int) {
        lwp = window_pane_find_by_id((*c).tty.mouse_last_pane as u_int);
        if !lwp.is_null() {
            log_debug(
                b"%s mouse last pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
                (*lwp).id,
            );
        }
    }
    if (*event).key == KEYC_DOUBLECLICK as ::core::ffi::c_ulong as key_code {
        type_0 = KEYC_TYPE_DOUBLECLICK;
        x = (*m).x;
        y = (*m).y;
        b = (*m).b;
        ignore = 1 as ::core::ffi::c_int;
        log_debug(
            b"double-click at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            x,
            y,
        );
    } else if (*m).sgr_type != ' ' as i32 as u_int
        && (*m).sgr_b & MOUSE_MASK_DRAG as u_int != 0
        && (*m).sgr_b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
        || (*m).sgr_type == ' ' as i32 as u_int
            && (*m).b & MOUSE_MASK_DRAG as u_int != 0
            && (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
            && (*m).lb & MOUSE_MASK_BUTTONS as u_int == 3 as u_int
    {
        type_0 = KEYC_TYPE_MOUSEMOVE;
        x = (*m).x;
        y = (*m).y;
        b = 0 as u_int;
        log_debug(
            b"move at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            x,
            y,
        );
    } else if (*m).b & MOUSE_MASK_DRAG as u_int != 0 {
        type_0 = KEYC_TYPE_MOUSEDRAG;
        if (*c).tty.mouse_drag_flag != 0 {
            x = (*m).x;
            y = (*m).y;
            b = (*m).b;
            if x == (*m).lx && y == (*m).ly {
                return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
            }
            log_debug(
                b"drag update at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
                x,
                y,
            );
        } else {
            x = (*m).lx;
            y = (*m).ly;
            b = (*m).lb;
            log_debug(
                b"drag start at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
                x,
                y,
            );
        }
    } else if (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int
        || (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_DOWN as u_int
    {
        if (*m).b & MOUSE_MASK_BUTTONS as u_int == MOUSE_WHEEL_UP as u_int {
            type_0 = KEYC_TYPE_WHEELUP;
        } else {
            type_0 = KEYC_TYPE_WHEELDOWN;
        }
        x = (*m).x;
        y = (*m).y;
        b = (*m).b;
        log_debug(
            b"wheel at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            x,
            y,
        );
    } else if (*m).b & MOUSE_MASK_BUTTONS as u_int == 3 as u_int {
        type_0 = KEYC_TYPE_MOUSEUP;
        x = (*m).x;
        y = (*m).y;
        b = (*m).lb;
        if (*m).sgr_type == 'm' as i32 as u_int {
            b = (*m).sgr_b;
        }
        log_debug(
            b"up at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            x,
            y,
        );
    } else {
        if (*c).flags & CLIENT_DOUBLECLICK as uint64_t != 0 {
            event_del(&raw mut (*c).click_timer);
            (*c).flags &= !CLIENT_DOUBLECLICK as uint64_t;
            type_0 = KEYC_TYPE_SECONDCLICK;
            x = (*m).x;
            y = (*m).y;
            b = (*m).b;
            log_debug(
                b"second-click at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
                x,
                y,
            );
            (*c).flags |= CLIENT_TRIPLECLICK as uint64_t;
            current_block = 16799951812150840583;
        } else if (*c).flags & CLIENT_TRIPLECLICK as uint64_t != 0 {
            event_del(&raw mut (*c).click_timer);
            (*c).flags &= !CLIENT_TRIPLECLICK as uint64_t;
            type_0 = KEYC_TYPE_TRIPLECLICK;
            x = (*m).x;
            y = (*m).y;
            b = (*m).b;
            log_debug(
                b"triple-click at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
                x,
                y,
            );
            current_block = 5614288427414743461;
        } else {
            current_block = 16799951812150840583;
        }
        match current_block {
            5614288427414743461 => {}
            _ => {
                if type_0 as ::core::ffi::c_uint
                    == KEYC_TYPE_NOTYPE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    type_0 = KEYC_TYPE_MOUSEDOWN;
                    x = (*m).x;
                    y = (*m).y;
                    b = (*m).b;
                    log_debug(
                        b"down at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
                        x,
                        y,
                    );
                    (*c).flags |= CLIENT_DOUBLECLICK as uint64_t;
                }
            }
        }
    }
    if type_0 as ::core::ffi::c_uint
        == KEYC_TYPE_NOTYPE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
    }
    (*m).s = (*s).id as ::core::ffi::c_int;
    (*m).w = -(1 as ::core::ffi::c_int);
    (*m).wp = -(1 as ::core::ffi::c_int);
    (*m).ignore = ignore;
    (*m).statusat = status_at_line(c);
    (*m).statuslines = status_line_size(c);
    if (*m).statusat != -(1 as ::core::ffi::c_int)
        && y >= (*m).statusat as u_int
        && y < ((*m).statusat as u_int).wrapping_add((*m).statuslines)
    {
        sr = status_get_range(c, x, y.wrapping_sub((*m).statusat as u_int));
        if sr.is_null() {
            loc = KEYC_MOUSE_LOCATION_STATUS_DEFAULT;
        } else {
            match (*sr).type_0 as ::core::ffi::c_uint {
                0 => return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code,
                1 => {
                    log_debug(b"mouse range: left\0" as *const u8 as *const ::core::ffi::c_char);
                    loc = KEYC_MOUSE_LOCATION_STATUS_LEFT;
                }
                2 => {
                    log_debug(b"mouse range: right\0" as *const u8 as *const ::core::ffi::c_char);
                    loc = KEYC_MOUSE_LOCATION_STATUS_RIGHT;
                }
                3 => {
                    fwp = window_pane_find_by_id((*sr).argument);
                    if fwp.is_null() {
                        return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                    }
                    (*m).wp = (*sr).argument as ::core::ffi::c_int;
                    log_debug(
                        b"mouse range: pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
                        (*m).wp,
                    );
                    loc = KEYC_MOUSE_LOCATION_STATUS;
                }
                4 => {
                    fwl = winlink_find_by_index(
                        &raw mut (*s).windows,
                        (*sr).argument as ::core::ffi::c_int,
                    );
                    if fwl.is_null() {
                        return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                    }
                    (*m).w = (*(*fwl).window).id as ::core::ffi::c_int;
                    log_debug(
                        b"mouse range: window @%u\0" as *const u8 as *const ::core::ffi::c_char,
                        (*m).w,
                    );
                    loc = KEYC_MOUSE_LOCATION_STATUS;
                }
                5 => {
                    fs = session_find_by_id((*sr).argument);
                    if fs.is_null() {
                        return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
                    }
                    (*m).s = (*sr).argument as ::core::ffi::c_int;
                    log_debug(
                        b"mouse range: session $%u\0" as *const u8 as *const ::core::ffi::c_char,
                        (*m).s,
                    );
                    loc = KEYC_MOUSE_LOCATION_STATUS;
                }
                6 => {
                    log_debug(b"mouse range: user\0" as *const u8 as *const ::core::ffi::c_char);
                    loc = KEYC_MOUSE_LOCATION_STATUS;
                }
                7 => {
                    n = (*sr).argument;
                    log_debug(
                        b"mouse range: control %u\0" as *const u8 as *const ::core::ffi::c_char,
                        n,
                    );
                    loc = (KEYC_MOUSE_LOCATION_CONTROL0 as ::core::ffi::c_int as u_int)
                        .wrapping_add(n) as key_code_mouse_location;
                }
                _ => {}
            }
        }
    }
    if loc as ::core::ffi::c_uint
        == KEYC_MOUSE_LOCATION_NOWHERE as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*c).tty.mouse_scrolling_flag != 0
    {
        if !lwp.is_null() {
            loc = KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER;
            (*m).wp = (*lwp).id as ::core::ffi::c_int;
            (*m).w = (*(*lwp).window).id as ::core::ffi::c_int;
        }
    } else if loc as ::core::ffi::c_uint
        == KEYC_MOUSE_LOCATION_NOWHERE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        px = x;
        if (*m).statusat == 0 as ::core::ffi::c_int && y >= (*m).statuslines {
            py = y.wrapping_sub((*m).statuslines);
        } else if (*m).statusat > 0 as ::core::ffi::c_int && y >= (*m).statusat as u_int {
            py = ((*m).statusat - 1 as ::core::ffi::c_int) as u_int;
        } else {
            py = y;
        }
        tty_window_offset(
            &raw mut (*c).tty,
            &raw mut (*m).ox,
            &raw mut (*m).oy,
            &raw mut sx,
            &raw mut sy,
        );
        log_debug(
            b"mouse window @%u at %u,%u (%ux%u)\0" as *const u8 as *const ::core::ffi::c_char,
            (*w).id,
            (*m).ox,
            (*m).oy,
            sx,
            sy,
        );
        if px > sx || py > sy {
            server_client_update_scrollbar_hover(
                c,
                type_0 as ::core::ffi::c_int,
                -(1 as ::core::ffi::c_int),
                -(1 as ::core::ffi::c_int),
            );
            return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
        }
        px = px.wrapping_add((*m).ox);
        py = py.wrapping_add((*m).oy);
        if !(*w).modal.is_null() && window_pane_contains((*w).modal, px, py) == 0 {
            if lwp == (*w).modal
                && (*c).tty.mouse_drag_flag != 0 as ::core::ffi::c_int
                && (type_0 as ::core::ffi::c_uint
                    == KEYC_TYPE_MOUSEDRAG as ::core::ffi::c_int as ::core::ffi::c_uint
                    || type_0 as ::core::ffi::c_uint
                        == KEYC_TYPE_MOUSEUP as ::core::ffi::c_int as ::core::ffi::c_uint)
            {
                modal_drag = 1 as ::core::ffi::c_int;
                wp = lwp;
                loc = KEYC_MOUSE_LOCATION_PANE;
                (*m).wp = (*wp).id as ::core::ffi::c_int;
                (*m).w = (*(*wp).window).id as ::core::ffi::c_int;
            } else {
                server_client_update_scrollbar_hover(
                    c,
                    type_0 as ::core::ffi::c_int,
                    -(1 as ::core::ffi::c_int),
                    -(1 as ::core::ffi::c_int),
                );
                (*c).tty.mouse_drag_update = None;
                (*c).tty.mouse_drag_release = None;
                (*c).tty.mouse_drag_flag = 0 as ::core::ffi::c_int;
                (*c).tty.mouse_scrolling_flag = 0 as ::core::ffi::c_int;
                (*c).tty.mouse_slider_mpos = -(1 as ::core::ffi::c_int);
                (*c).tty.mouse_last_pane = -(1 as ::core::ffi::c_int);
                if (*(*w).modal).flags & PANE_CLOSEONCLICK != 0
                    && (type_0 as ::core::ffi::c_uint
                        == KEYC_TYPE_MOUSEDOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                        || type_0 as ::core::ffi::c_uint
                            == KEYC_TYPE_SECONDCLICK as ::core::ffi::c_int as ::core::ffi::c_uint
                        || type_0 as ::core::ffi::c_uint
                            == KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    server_kill_pane((*w).modal);
                }
                return KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
            }
        }
        server_client_update_scrollbar_hover(
            c,
            type_0 as ::core::ffi::c_int,
            px as ::core::ffi::c_int,
            py as ::core::ffi::c_int,
        );
        if !(modal_drag != 0) {
            if type_0 as ::core::ffi::c_uint
                == KEYC_TYPE_MOUSEDRAG as ::core::ffi::c_int as ::core::ffi::c_uint
                && !lwp.is_null()
            {
                wp = lwp;
            } else {
                wp = window_get_active_at(w, px, py);
            }
        }
        if wp.is_null() {
            loc = KEYC_MOUSE_LOCATION_EMPTY;
            (*m).w = (*w).id as ::core::ffi::c_int;
            log_debug(
                b"mouse %u,%u on empty area\0" as *const u8 as *const ::core::ffi::c_char,
                x,
                y,
            );
        } else {
            if modal_drag == 0 {
                loc = server_client_check_mouse_in_pane(
                    wp,
                    px as ::core::ffi::c_int,
                    py as ::core::ffi::c_int,
                    &raw mut sl_mpos,
                );
            }
            if loc as ::core::ffi::c_uint
                == KEYC_MOUSE_LOCATION_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                log_debug(
                    b"mouse %u,%u on pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
                    x,
                    y,
                    (*wp).id,
                );
            } else if loc as ::core::ffi::c_uint
                == KEYC_MOUSE_LOCATION_BORDER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                sr = window_pane_status_get_range(wp, px, py);
                if !sr.is_null() {
                    n = (*sr).argument;
                    loc = (KEYC_MOUSE_LOCATION_CONTROL0 as ::core::ffi::c_int as u_int)
                        .wrapping_add(n) as key_code_mouse_location;
                }
                log_debug(
                    b"mouse on pane %%%u border\0" as *const u8 as *const ::core::ffi::c_char,
                    (*wp).id,
                );
            } else if loc as ::core::ffi::c_uint
                == KEYC_MOUSE_LOCATION_SCROLLBAR_UP as ::core::ffi::c_int as ::core::ffi::c_uint
                || loc as ::core::ffi::c_uint
                    == KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER as ::core::ffi::c_int
                        as ::core::ffi::c_uint
                || loc as ::core::ffi::c_uint
                    == KEYC_MOUSE_LOCATION_SCROLLBAR_DOWN as ::core::ffi::c_int
                        as ::core::ffi::c_uint
            {
                log_debug(
                    b"mouse on pane %%%u scrollbar\0" as *const u8 as *const ::core::ffi::c_char,
                    (*wp).id,
                );
            }
            (*m).wp = (*wp).id as ::core::ffi::c_int;
            (*m).w = (*(*wp).window).id as ::core::ffi::c_int;
        }
    } else {
        server_client_update_scrollbar_hover(
            c,
            type_0 as ::core::ffi::c_int,
            -(1 as ::core::ffi::c_int),
            -(1 as ::core::ffi::c_int),
        );
    }
    if type_0 as ::core::ffi::c_uint
        == KEYC_TYPE_MOUSEDOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        || type_0 as ::core::ffi::c_uint
            == KEYC_TYPE_SECONDCLICK as ::core::ffi::c_int as ::core::ffi::c_uint
        || type_0 as ::core::ffi::c_uint
            == KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if type_0 as ::core::ffi::c_uint
            != KEYC_TYPE_MOUSEDOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && ((*m).b != (*c).click_button
                || loc as ::core::ffi::c_uint
                    != (*c).click_loc as key_code_mouse_location as ::core::ffi::c_uint
                || (*m).wp != (*c).click_wp)
        {
            type_0 = KEYC_TYPE_MOUSEDOWN;
            log_debug(
                b"click sequence reset at %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
                x,
                y,
            );
            (*c).flags &= !CLIENT_TRIPLECLICK as uint64_t;
            (*c).flags |= CLIENT_DOUBLECLICK as uint64_t;
        }
        if type_0 as ::core::ffi::c_uint
            != KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_uint
            && KEYC_CLICK_TIMEOUT != 0 as ::core::ffi::c_int
        {
            memcpy(
                &raw mut (*c).click_event as *mut ::core::ffi::c_void,
                m as *const ::core::ffi::c_void,
                ::core::mem::size_of::<mouse_event>() as size_t,
            );
            (*c).click_button = (*m).b;
            (*c).click_loc = loc as ::core::ffi::c_int;
            (*c).click_wp = (*m).wp;
            log_debug(b"click timer started\0" as *const u8 as *const ::core::ffi::c_char);
            tv.tv_sec = (KEYC_CLICK_TIMEOUT / 1000 as ::core::ffi::c_int) as __time_t;
            tv.tv_usec = ((KEYC_CLICK_TIMEOUT % 1000 as ::core::ffi::c_int) as ::core::ffi::c_long
                * 1000 as ::core::ffi::c_long) as __suseconds_t;
            event_del(&raw mut (*c).click_timer);
            event_add(&raw mut (*c).click_timer, &raw mut tv);
        }
    }
    key = KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code;
    if type_0 as ::core::ffi::c_uint
        != KEYC_TYPE_MOUSEDRAG as ::core::ffi::c_int as ::core::ffi::c_uint
        && type_0 as ::core::ffi::c_uint
            != KEYC_TYPE_WHEELUP as ::core::ffi::c_int as ::core::ffi::c_uint
        && type_0 as ::core::ffi::c_uint
            != KEYC_TYPE_WHEELDOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && type_0 as ::core::ffi::c_uint
            != KEYC_TYPE_DOUBLECLICK as ::core::ffi::c_int as ::core::ffi::c_uint
        && type_0 as ::core::ffi::c_uint
            != KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*c).tty.mouse_drag_flag != 0 as ::core::ffi::c_int
    {
        if (*c).tty.mouse_drag_release.is_some() {
            (*c).tty
                .mouse_drag_release
                .expect("non-null function pointer")(c, m);
        }
        (*c).tty.mouse_drag_update = None;
        (*c).tty.mouse_drag_release = None;
        (*c).tty.mouse_scrolling_flag = 0 as ::core::ffi::c_int;
        type_0 = KEYC_TYPE_MOUSEDRAGEND;
        (*c).tty.mouse_drag_flag = 0 as ::core::ffi::c_int;
        (*c).tty.mouse_slider_mpos = -(1 as ::core::ffi::c_int);
        (*c).tty.mouse_last_pane = -(1 as ::core::ffi::c_int);
    }
    if type_0 as ::core::ffi::c_uint
        == KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_uint
        && loc as ::core::ffi::c_uint
            == KEYC_MOUSE_LOCATION_PANE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        key = KEYC_MOUSEMOVE_PANE as ::core::ffi::c_ulong as key_code;
        if !wp.is_null()
            && wp != (*w).active
            && options_get_number(
                (*s).options,
                b"focus-follows-mouse\0" as *const u8 as *const ::core::ffi::c_char,
            ) != 0
        {
            window_redraw_active_switch(w, wp);
            window_set_active_pane(w, wp, 1 as ::core::ffi::c_int);
            server_redraw_window_borders(w);
            server_status_window(w);
        }
    }
    if type_0 as ::core::ffi::c_uint
        == KEYC_TYPE_MOUSEDRAG as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*c).tty.mouse_drag_update.is_some() {
            key = KEYC_DRAGGING as ::core::ffi::c_ulong as key_code;
        }
        if (*c).tty.mouse_drag_flag == 0 as ::core::ffi::c_int {
            (*c).tty.mouse_drag_x = px;
            (*c).tty.mouse_drag_y = py;
        }
        (*c).tty.mouse_drag_flag =
            (b & MOUSE_MASK_BUTTONS as u_int).wrapping_add(1 as u_int) as ::core::ffi::c_int;
        if lwp.is_null() {
            wp = window_get_active_at(w, px, py);
            lwp = wp;
            if !wp.is_null() {
                (*c).tty.mouse_last_pane = (*wp).id as ::core::ffi::c_int;
            }
        }
        if (*c).tty.mouse_scrolling_flag == 0 as ::core::ffi::c_int
            && loc as ::core::ffi::c_uint
                == KEYC_MOUSE_LOCATION_SCROLLBAR_SLIDER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*c).tty.mouse_scrolling_flag = 1 as ::core::ffi::c_int;
            if (*m).statusat == 0 as ::core::ffi::c_int {
                (*c).tty.mouse_slider_mpos =
                    sl_mpos.wrapping_add((*m).statuslines) as ::core::ffi::c_int;
            } else {
                (*c).tty.mouse_slider_mpos = sl_mpos as ::core::ffi::c_int;
            }
        }
    }
    if key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
        if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_1 as u_int {
            bn = 1 as u_int;
        } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_2 as u_int {
            bn = 2 as u_int;
        } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_3 as u_int {
            bn = 3 as u_int;
        } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_6 as u_int {
            bn = 6 as u_int;
        } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_7 as u_int {
            bn = 7 as u_int;
        } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_8 as u_int {
            bn = 8 as u_int;
        } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_9 as u_int {
            bn = 9 as u_int;
        } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_10 as u_int {
            bn = 10 as u_int;
        } else if b & MOUSE_MASK_BUTTONS as u_int == MOUSE_BUTTON_11 as u_int {
            bn = 11 as u_int;
        } else {
            bn = 0 as u_int;
        }
        key = ((type_0 as ::core::ffi::c_ulonglong) << 32 as ::core::ffi::c_int
            | (bn as ::core::ffi::c_ulonglong) << KEYC_MOUSE_BUTTON_SHIFT
            | (loc as ::core::ffi::c_ulonglong) << KEYC_MOUSE_LOCATION_SHIFT)
            as key_code;
    }
    if b & MOUSE_MASK_META as u_int != 0 {
        key |= KEYC_META;
    }
    if b & MOUSE_MASK_CTRL as u_int != 0 {
        key |= KEYC_CTRL;
    }
    if b & MOUSE_MASK_SHIFT as u_int != 0 {
        key |= KEYC_SHIFT;
    }
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(
            b"mouse key is %s\0" as *const u8 as *const ::core::ffi::c_char,
            key_string_lookup_key(key, 1 as ::core::ffi::c_int),
        );
    }
    return key;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_update_theme_colours(mut c: *mut client) {
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut theme: client_theme = THEME_UNKNOWN;
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut colour: ::core::ffi::c_int = 0;
    let mut option: ::core::ffi::c_int = 0;
    if c.is_null() {
        return;
    }
    option = options_get_number(
        global_options,
        b"theme\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if option == 1 as ::core::ffi::c_int {
        i = 0 as u_int;
        while i < COLOUR_THEME_COUNT as u_int {
            (*c).theme_colours[i as usize] = colour_theme_terminal_colour(i);
            i = i.wrapping_add(1);
        }
        return;
    }
    ft = format_create(
        c,
        ::core::ptr::null_mut::<cmdq_item>(),
        FORMAT_NONE,
        FORMAT_NOJOBS,
    );
    format_defaults(
        ft,
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    theme = (*c).theme;
    if theme as ::core::ffi::c_uint == THEME_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint {
        theme = colour_totheme((*c).tty.bg);
    }
    if option == 2 as ::core::ffi::c_int {
        theme = THEME_LIGHT;
    } else if option == 3 as ::core::ffi::c_int {
        theme = THEME_DARK;
    }
    i = 0 as u_int;
    while i < COLOUR_THEME_COUNT as u_int {
        (*c).theme_colours[i as usize] = 8 as ::core::ffi::c_int;
        name = colour_theme_option(i, theme);
        if !name.is_null() {
            value = options_get_string(global_options, name);
            expanded = format_expand(ft, value);
            colour = colour_fromstring(expanded);
            free(expanded as *mut ::core::ffi::c_void);
            if !(colour == -(1 as ::core::ffi::c_int) || colour & COLOUR_FLAG_THEME != 0) {
                (*c).theme_colours[i as usize] = colour;
            }
        }
        i = i.wrapping_add(1);
    }
    format_free(ft);
}
unsafe extern "C" fn server_client_is_bracket_paste(
    mut c: *mut client,
    mut key: key_code,
) -> ::core::ffi::c_int {
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_PASTE_START as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
    {
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong | CLIENT_BRACKETPASTING) as uint64_t;
        (*c).paste_time = current_time;
        log_debug(
            b"%s: bracket paste on\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
        return 0 as ::core::ffi::c_int;
    }
    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_PASTE_END as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
    {
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong & !CLIENT_BRACKETPASTING) as uint64_t;
        log_debug(
            b"%s: bracket paste off\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
        return 0 as ::core::ffi::c_int;
    }
    return ((*c).flags as ::core::ffi::c_ulonglong & CLIENT_BRACKETPASTING != 0)
        as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_is_assume_paste(mut c: *mut client) -> ::core::ffi::c_int {
    let mut s: *mut session = (*c).session;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut t: ::core::ffi::c_int = 0;
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_BRACKETPASTING != 0 {
        return 0 as ::core::ffi::c_int;
    }
    t = options_get_number(
        (*s).options,
        b"assume-paste-time\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if t == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if tty_term_has((*c).tty.term, TTYC_ENBP) != 0 {
        return 0 as ::core::ffi::c_int;
    }
    tv.tv_sec = (*c).activity_time.tv_sec - (*c).last_activity_time.tv_sec;
    tv.tv_usec = (*c).activity_time.tv_usec - (*c).last_activity_time.tv_usec;
    if tv.tv_usec < 0 as __suseconds_t {
        tv.tv_sec -= 1;
        tv.tv_usec += 1000000 as __suseconds_t;
    }
    if tv.tv_sec == 0 as __time_t && tv.tv_usec < (t * 1000 as ::core::ffi::c_int) as __suseconds_t
    {
        if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_ASSUMEPASTING != 0 {
            return 1 as ::core::ffi::c_int;
        }
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong | CLIENT_ASSUMEPASTING) as uint64_t;
        (*c).paste_time = current_time;
        log_debug(
            b"%s: assume paste on\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
        return 0 as ::core::ffi::c_int;
    }
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_ASSUMEPASTING != 0 {
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong & !CLIENT_ASSUMEPASTING) as uint64_t;
        log_debug(
            b"%s: assume paste off\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_update_latest(mut c: *mut client) {
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    if (*c).session.is_null() {
        return;
    }
    w = (*(*(*c).session).curw).window;
    if (*w).latest == c as *mut ::core::ffi::c_void {
        return;
    }
    (*w).latest = c as *mut ::core::ffi::c_void;
    if options_get_number(
        (*w).options,
        b"window-size\0" as *const u8 as *const ::core::ffi::c_char,
    ) == WINDOW_SIZE_LATEST as ::core::ffi::c_longlong
    {
        recalculate_size(w, 0 as ::core::ffi::c_int);
    }
    events_fire_client(
        b"client-active\0" as *const u8 as *const ::core::ffi::c_char,
        c,
    );
}
unsafe extern "C" fn server_client_repeat_time(
    mut c: *mut client,
    mut bd: *mut key_binding,
) -> u_int {
    let mut s: *mut session = (*c).session;
    let mut repeat: u_int = 0;
    let mut initial: u_int = 0;
    if !(*bd).flags & KEY_BINDING_REPEAT != 0 {
        return 0 as u_int;
    }
    repeat = options_get_number(
        (*s).options,
        b"repeat-time\0" as *const u8 as *const ::core::ffi::c_char,
    ) as u_int;
    if repeat == 0 as u_int {
        return 0 as u_int;
    }
    if !(*c).flags & CLIENT_REPEAT as uint64_t != 0 || (*bd).key != (*c).last_key {
        initial = options_get_number(
            (*s).options,
            b"initial-repeat-time\0" as *const u8 as *const ::core::ffi::c_char,
        ) as u_int;
        if initial != 0 as u_int {
            repeat = initial;
        }
    }
    return repeat;
}
unsafe extern "C" fn server_client_handle_dead_key(
    mut wp: *mut window_pane,
    mut key: key_code,
) -> ::core::ffi::c_int {
    let mut remain_on_exit: ::core::ffi::c_int = 0;
    if wp.is_null()
        || !(*wp).flags & PANE_EXITED != 0
        || (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
            == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
            || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
                && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int)
        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            == (KEYC_TYPE_FUNCTION as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                == KEYC_PASTE_START as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                || key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == KEYC_PASTE_END as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong)
    {
        return 0 as ::core::ffi::c_int;
    }
    remain_on_exit = options_get_number(
        (*wp).options,
        b"remain-on-exit\0" as *const u8 as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int;
    if remain_on_exit != 3 as ::core::ffi::c_int && remain_on_exit != 4 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    options_set_number(
        (*wp).options,
        b"remain-on-exit\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_longlong,
    );
    server_destroy_pane(wp, 0 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_key_callback(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut current_block: u64;
    let mut event: *mut key_event = data as *mut key_event;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut ec: *mut client = (*event).client;
    let mut key: key_code = (*event).key;
    let mut m: *mut mouse_event = &raw mut (*event).m;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut table: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut first: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut repeat: u_int = 0;
    let mut flags: uint64_t = 0;
    let mut prefix_delay: uint64_t = 0;
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut key0: key_code = 0;
    let mut prefix: key_code = 0;
    let mut prefix2: key_code = 0;
    if !ec.is_null() {
        c = ec;
    } else {
        c = cmdq_get_client(item);
    }
    s = (*c).session;
    if !(s.is_null() || (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0) {
        wl = (*s).curw;
        memcpy(
            &raw mut (*c).last_activity_time as *mut ::core::ffi::c_void,
            &raw mut (*c).activity_time as *const ::core::ffi::c_void,
            ::core::mem::size_of::<timeval>() as size_t,
        );
        if gettimeofday(&raw mut (*c).activity_time, NULL) != 0 as ::core::ffi::c_int {
            fatal(b"gettimeofday failed\0" as *const u8 as *const ::core::ffi::c_char);
        }
        session_update_activity(s, &raw mut (*c).activity_time);
        (*m).valid = 0 as ::core::ffi::c_int;
        if key == KEYC_MOUSE as ::core::ffi::c_ulong as key_code
            || key == KEYC_DOUBLECLICK as ::core::ffi::c_ulong as key_code
        {
            if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
                current_block = 1578459965781631232;
            } else {
                key = server_client_check_mouse(c, event);
                if key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code {
                    current_block = 1578459965781631232;
                } else {
                    (*m).valid = 1 as ::core::ffi::c_int;
                    (*m).key = key;
                    if key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                        == KEYC_DRAGGING as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                    {
                        (*c).tty
                            .mouse_drag_update
                            .expect("non-null function pointer")(c, m);
                        current_block = 1578459965781631232;
                    } else {
                        (*event).key = key;
                        current_block = 4495394744059808450;
                    }
                }
            }
        } else {
            current_block = 4495394744059808450;
        }
        match current_block {
            1578459965781631232 => {}
            _ => {
                if !(key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                    || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                        >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                            << 32 as ::core::ffi::c_int
                        && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                            <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                                as ::core::ffi::c_ulonglong)
                                << 32 as ::core::ffi::c_int)
                    || cmd_find_from_mouse(&raw mut fs, m, 0 as ::core::ffi::c_int)
                        != 0 as ::core::ffi::c_int
                {
                    cmd_find_from_client(&raw mut fs, c, 0 as ::core::ffi::c_int);
                }
                wp = fs.wp;
                if (key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                    == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                    || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                        >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                            << 32 as ::core::ffi::c_int
                        && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                            <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                                as ::core::ffi::c_ulonglong)
                                << 32 as ::core::ffi::c_int)
                    && options_get_number(
                        (*s).options,
                        b"mouse\0" as *const u8 as *const ::core::ffi::c_char,
                    ) == 0
                {
                    current_block = 15469183920764600035;
                } else {
                    if server_client_is_bracket_paste(c, key) != 0 {
                        current_block = 3436715649514806935;
                    } else if !(key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                        || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int
                                as ::core::ffi::c_ulonglong)
                                << 32 as ::core::ffi::c_int
                            && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                                    as ::core::ffi::c_ulonglong)
                                    << 32 as ::core::ffi::c_int)
                        && key != KEYC_FOCUS_IN as ::core::ffi::c_ulong as key_code
                        && key != KEYC_FOCUS_OUT as ::core::ffi::c_ulong as key_code
                        && !(key as ::core::ffi::c_ulonglong) & KEYC_SENT != 0
                        && server_client_is_assume_paste(c) != 0
                    {
                        current_block = 3436715649514806935;
                    } else if !wp.is_null()
                        && (*wp).flags & PANE_CAPTUREALLKEYS != 0
                        && !(*wp).flags & PANE_EXITED != 0
                        && !(key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                            == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                            || key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int
                                    as ::core::ffi::c_ulonglong)
                                    << 32 as ::core::ffi::c_int
                                && key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                    <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                                        as ::core::ffi::c_ulonglong)
                                        << 32 as ::core::ffi::c_int)
                        && (*wp).modes.tqh_first.is_null()
                    {
                        current_block = 15469183920764600035;
                    } else if key == KEYC_FOCUS_IN as ::core::ffi::c_ulong as key_code
                        || key == KEYC_FOCUS_OUT as ::core::ffi::c_ulong as key_code
                    {
                        current_block = 15469183920764600035;
                    } else {
                        if server_client_is_default_key_table(c, (*c).keytable as *mut key_table)
                            != 0
                            && !wp.is_null()
                            && {
                                wme = (*wp).modes.tqh_first;
                                !wme.is_null()
                            }
                            && (*(*wme).mode).key_table.is_some()
                        {
                            table = key_bindings_get_table(
                                (*(*wme).mode).key_table.expect("non-null function pointer")(wme),
                                1 as ::core::ffi::c_int,
                            );
                        } else {
                            table = (*c).keytable as *mut key_table;
                        }
                        first = table;
                        '_table_changed: loop {
                            prefix = options_get_number(
                                (*s).options,
                                b"prefix\0" as *const u8 as *const ::core::ffi::c_char,
                            ) as key_code;
                            prefix2 = options_get_number(
                                (*s).options,
                                b"prefix2\0" as *const u8 as *const ::core::ffi::c_char,
                            ) as key_code;
                            key0 = (key as ::core::ffi::c_ulonglong
                                & (KEYC_MASK_KEY | KEYC_MASK_MODIFIERS))
                                as key_code;
                            if (key0
                                == prefix as ::core::ffi::c_ulonglong
                                    & (KEYC_MASK_KEY | KEYC_MASK_MODIFIERS)
                                || key0
                                    == prefix2 as ::core::ffi::c_ulonglong
                                        & (KEYC_MASK_KEY | KEYC_MASK_MODIFIERS))
                                && strcmp(
                                    (*table).name,
                                    b"prefix\0" as *const u8 as *const ::core::ffi::c_char,
                                ) != 0 as ::core::ffi::c_int
                            {
                                server_client_set_key_table(
                                    c,
                                    b"prefix\0" as *const u8 as *const ::core::ffi::c_char,
                                );
                                server_status_client(c);
                                current_block = 1578459965781631232;
                                break;
                            } else {
                                flags = (*c).flags;
                                loop {
                                    if wp.is_null() {
                                        log_debug(
                                            b"key table %s (no pane)\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            (*table).name,
                                        );
                                    } else {
                                        log_debug(
                                            b"key table %s (pane %%%u)\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            (*table).name,
                                            (*wp).id,
                                        );
                                    }
                                    if (*c).flags & CLIENT_REPEAT as uint64_t != 0 {
                                        log_debug(
                                            b"currently repeating\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                        );
                                    }
                                    bd = key_bindings_get(table, key0);
                                    prefix_delay = options_get_number(
                                        global_options,
                                        b"prefix-timeout\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                    )
                                        as uint64_t;
                                    if prefix_delay > 0 as uint64_t
                                        && strcmp(
                                            (*table).name,
                                            b"prefix\0" as *const u8 as *const ::core::ffi::c_char,
                                        ) == 0 as ::core::ffi::c_int
                                        && server_client_key_table_activity_diff(c) > prefix_delay
                                    {
                                        if !bd.is_null()
                                            && (*c).flags & CLIENT_REPEAT as uint64_t != 0
                                            && (*bd).flags & KEY_BINDING_REPEAT != 0
                                        {
                                            log_debug(
                                                b"prefix timeout ignored, repeat is active\0"
                                                    as *const u8
                                                    as *const ::core::ffi::c_char,
                                            );
                                        } else {
                                            log_debug(
                                                b"prefix timeout exceeded\0" as *const u8
                                                    as *const ::core::ffi::c_char,
                                            );
                                            server_client_set_key_table(
                                                c,
                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                            );
                                            table = (*c).keytable as *mut key_table;
                                            first = table;
                                            server_status_client(c);
                                            continue '_table_changed;
                                        }
                                    }
                                    if !bd.is_null() {
                                        if (*c).flags & CLIENT_REPEAT as uint64_t != 0
                                            && !(*bd).flags & KEY_BINDING_REPEAT != 0
                                        {
                                            current_block = 5891011138178424807;
                                            break;
                                        } else {
                                            current_block = 13763002826403452995;
                                            break;
                                        }
                                    } else if key0 != KEYC_ANY as ::core::ffi::c_ulong as key_code {
                                        key0 = KEYC_ANY as ::core::ffi::c_ulong as key_code;
                                    } else {
                                        if key
                                            == KEYC_MOUSEMOVE_PANE as ::core::ffi::c_ulong
                                                as key_code
                                            || key
                                                == KEYC_MOUSEMOVE_STATUS as ::core::ffi::c_ulong
                                                    as key_code
                                            || key
                                                == KEYC_MOUSEMOVE_STATUS_LEFT
                                                    as ::core::ffi::c_ulong
                                                    as key_code
                                            || key
                                                == KEYC_MOUSEMOVE_STATUS_RIGHT
                                                    as ::core::ffi::c_ulong
                                                    as key_code
                                            || key
                                                == KEYC_MOUSEMOVE_STATUS_DEFAULT
                                                    as ::core::ffi::c_ulong
                                                    as key_code
                                            || key
                                                == KEYC_MOUSEMOVE_BORDER as ::core::ffi::c_ulong
                                                    as key_code
                                        {
                                            current_block = 15469183920764600035;
                                            break '_table_changed;
                                        }
                                        log_debug(
                                            b"not found in key table %s\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            (*table).name,
                                        );
                                        if server_client_is_default_key_table(c, table) == 0
                                            || (*c).flags & CLIENT_REPEAT as uint64_t != 0
                                        {
                                            current_block = 13484060386966298149;
                                            break;
                                        } else {
                                            current_block = 11796148217846552555;
                                            break;
                                        }
                                    }
                                }
                                match current_block {
                                    11796148217846552555 => {
                                        if first != table && !flags & CLIENT_REPEAT as uint64_t != 0
                                        {
                                            current_block = 10435735846551762309;
                                            break;
                                        } else {
                                            current_block = 15469183920764600035;
                                            break;
                                        }
                                    }
                                    13763002826403452995 => {
                                        log_debug(
                                            b"found in key table %s\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            (*table).name,
                                        );
                                        (*table).references = (*table).references.wrapping_add(1);
                                        repeat = server_client_repeat_time(c, bd);
                                        if repeat != 0 as u_int {
                                            (*c).flags |= CLIENT_REPEAT as uint64_t;
                                            (*c).last_key = (*bd).key;
                                            tv.tv_sec =
                                                repeat.wrapping_div(1000 as u_int) as __time_t;
                                            tv.tv_usec = (repeat.wrapping_rem(1000 as u_int)
                                                as ::core::ffi::c_long
                                                * 1000 as ::core::ffi::c_long)
                                                as __suseconds_t;
                                            event_del(&raw mut (*c).repeat_timer);
                                            event_add(&raw mut (*c).repeat_timer, &raw mut tv);
                                        } else {
                                            (*c).flags &= !CLIENT_REPEAT as uint64_t;
                                            server_client_set_key_table(
                                                c,
                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                            );
                                        }
                                        server_status_client(c);
                                        key_bindings_dispatch(bd, item, c, event, &raw mut fs);
                                        key_bindings_unref_table(table);
                                        current_block = 1578459965781631232;
                                        break;
                                    }
                                    13484060386966298149 => {
                                        log_debug(
                                            b"trying in root table\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                        );
                                        server_client_set_key_table(
                                            c,
                                            ::core::ptr::null::<::core::ffi::c_char>(),
                                        );
                                        table = (*c).keytable as *mut key_table;
                                        if (*c).flags & CLIENT_REPEAT as uint64_t != 0 {
                                            first = table;
                                        }
                                        (*c).flags &= !CLIENT_REPEAT as uint64_t;
                                        server_status_client(c);
                                    }
                                    _ => {
                                        log_debug(
                                            b"found in key table %s (not repeating)\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            (*table).name,
                                        );
                                        server_client_set_key_table(
                                            c,
                                            ::core::ptr::null::<::core::ffi::c_char>(),
                                        );
                                        table = (*c).keytable as *mut key_table;
                                        first = table;
                                        (*c).flags &= !CLIENT_REPEAT as uint64_t;
                                        server_status_client(c);
                                    }
                                }
                            }
                        }
                        match current_block {
                            1578459965781631232 => {}
                            15469183920764600035 => {}
                            _ => {
                                server_client_set_key_table(
                                    c,
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                );
                                server_status_client(c);
                                current_block = 1578459965781631232;
                            }
                        }
                    }
                    match current_block {
                        1578459965781631232 => {}
                        15469183920764600035 => {}
                        _ => {
                            if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
                                current_block = 1578459965781631232;
                            } else {
                                if !(*event).buf.is_null() {
                                    window_pane_paste(wp, key, (*event).buf, (*event).len);
                                }
                                key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
                                current_block = 1578459965781631232;
                            }
                        }
                    }
                }
                match current_block {
                    1578459965781631232 => {}
                    _ => {
                        if !(server_client_handle_dead_key(wp, key) != 0) {
                            if !((*c).flags & CLIENT_READONLY as uint64_t != 0) {
                                if !wp.is_null() {
                                    window_pane_key(wp, c, s, wl, key, m);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !s.is_null() && key != KEYC_FOCUS_OUT as ::core::ffi::c_ulong as key_code {
        server_client_update_latest(c);
    }
    if !ec.is_null() {
        server_client_unref(ec);
    }
    free((*event).buf as *mut ::core::ffi::c_void);
    free(event as *mut ::core::ffi::c_void);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn server_client_handle_menu_key(
    mut c: *mut client,
    mut event: *mut key_event,
) -> ::core::ffi::c_int {
    let mut w: *mut window = (*(*(*c).session).curw).window;
    let mut new_event: key_event = key_event {
        client: ::core::ptr::null_mut::<client>(),
        key: 0,
        m: mouse_event {
            valid: 0,
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
        },
        buf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        len: 0,
    };
    let mut m: *mut mouse_event = ::core::ptr::null_mut::<mouse_event>();
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    if (*w).menu.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    memcpy(
        &raw mut new_event as *mut ::core::ffi::c_void,
        event as *const ::core::ffi::c_void,
        ::core::mem::size_of::<key_event>() as size_t,
    );
    if (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
        || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                << 32 as ::core::ffi::c_int
            && (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                    << 32 as ::core::ffi::c_int
    {
        m = &raw mut new_event.m;
        (*m).statusat = status_at_line(c);
        (*m).statuslines = status_line_size(c);
        tty_window_offset(
            &raw mut (*c).tty,
            &raw mut ox,
            &raw mut oy,
            &raw mut sx,
            &raw mut sy,
        );
        (*m).x = (*m).x.wrapping_add(ox);
        if (*m).statusat == 0 as ::core::ffi::c_int {
            if (*m).y < (*m).statuslines {
                (*m).y = UINT_MAX as u_int;
                (*m).x = (*m).y;
            } else {
                (*m).y = (*m).y.wrapping_sub((*m).statuslines).wrapping_add(oy);
            }
        } else if (*m).statusat > 0 as ::core::ffi::c_int && (*m).y >= (*m).statusat as u_int {
            (*m).y = UINT_MAX as u_int;
            (*m).x = (*m).y;
        } else {
            (*m).y = (*m).y.wrapping_add(oy);
        }
    }
    if menu_key(c, (*w).menu, &raw mut new_event) == 1 as ::core::ffi::c_int {
        menu_close(w);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_handle_key0(
    mut c: *mut client,
    mut event: *mut key_event,
    mut after: *mut cmdq_item,
    mut next: *mut *mut cmdq_item,
) -> ::core::ffi::c_int {
    let mut s: *mut session = (*c).session;
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if s.is_null() || (*c).flags & CLIENT_UNATTACHEDFLAGS as uint64_t != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if (*event).key == KEYC_REPORT_LIGHT_THEME as ::core::ffi::c_ulong as key_code {
        server_client_report_theme(c, THEME_LIGHT);
        return 0 as ::core::ffi::c_int;
    }
    if (*event).key == KEYC_REPORT_DARK_THEME as ::core::ffi::c_ulong as key_code {
        server_client_report_theme(c, THEME_DARK);
        return 0 as ::core::ffi::c_int;
    }
    if !(*c).flags & CLIENT_READONLY as uint64_t != 0 {
        if !(*c).message_string.is_null() {
            if (*c).message_ignore_keys != 0 {
                return 0 as ::core::ffi::c_int;
            }
            status_message_clear(c);
        }
        if (*c).overlay_key.is_some() {
            match (*c).overlay_key.expect("non-null function pointer")(c, (*c).overlay_data, event)
            {
                0 => return 0 as ::core::ffi::c_int,
                1 => {
                    server_client_clear_overlay(c);
                    return 0 as ::core::ffi::c_int;
                }
                _ => {}
            }
        }
        server_client_clear_overlay(c);
        wp = (*(*(*s).curw).window).active;
        if server_client_handle_dead_key(wp, (*event).key) != 0 {
            return 0 as ::core::ffi::c_int;
        }
        if !wp.is_null()
            && wp == (*(*wp).window).modal
            && (*wp).flags & PANE_CLOSEONCANCEL != 0
            && ((*event).key == '\u{1b}' as i32 as key_code
                || (*event).key == 'c' as i32 as ::core::ffi::c_ulonglong | KEYC_CTRL)
        {
            server_kill_pane(wp);
            return 0 as ::core::ffi::c_int;
        }
        if !wp.is_null()
            && (*wp).flags & PANE_CAPTUREALLKEYS != 0
            && (*wp).modes.tqh_first.is_null()
            && !((*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                    >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int as ::core::ffi::c_ulonglong)
                        << 32 as ::core::ffi::c_int
                    && (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                        <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                            as ::core::ffi::c_ulonglong)
                            << 32 as ::core::ffi::c_int)
        {
            if !(*wp).flags & PANE_EXITED != 0 {
                window_pane_key(wp, c, s, (*s).curw, (*event).key, &raw mut (*event).m);
                return 0 as ::core::ffi::c_int;
            }
        }
        if server_client_handle_menu_key(c, event) != 0 {
            return 0 as ::core::ffi::c_int;
        }
        if !(*c).prompt.is_null() {
            match status_prompt_key(c, (*event).key, &raw mut (*event).m) as ::core::ffi::c_uint {
                1 | 2 => return 0 as ::core::ffi::c_int,
                0 | 3 | _ => {}
            }
        }
        wp = (*(*(*s).curw).window).active;
        if wp.is_null() || window_pane_has_prompt(wp) == 0 {
            wp = (*(*(*s).curw).window).panes.tqh_first;
            while !wp.is_null() {
                if window_pane_has_prompt(wp) != 0 && window_pane_is_visible(wp) != 0 {
                    break;
                }
                wp = (*wp).entry.tqe_next;
            }
        }
        if !wp.is_null() && window_pane_has_prompt(wp) != 0 && window_pane_is_visible(wp) != 0 {
            match window_pane_prompt_key(wp, c, (*event).key, &raw mut (*event).m)
                as ::core::ffi::c_uint
            {
                1 | 2 | 3 => return 0 as ::core::ffi::c_int,
                0 => {
                    if (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_KEY
                        == KEYC_MOUSE as ::core::ffi::c_ulong as ::core::ffi::c_ulonglong
                        || (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                            >= (KEYC_TYPE_MOUSEMOVE as ::core::ffi::c_int
                                as ::core::ffi::c_ulonglong)
                                << 32 as ::core::ffi::c_int
                            && (*event).key as ::core::ffi::c_ulonglong & KEYC_MASK_TYPE
                                <= (KEYC_TYPE_TRIPLECLICK as ::core::ffi::c_int
                                    as ::core::ffi::c_ulonglong)
                                    << 32 as ::core::ffi::c_int
                    {
                        return 0 as ::core::ffi::c_int;
                    }
                }
                _ => {}
            }
        }
    }
    item = cmdq_get_callback1(
        b"server_client_key_callback\0" as *const u8 as *const ::core::ffi::c_char,
        Some(
            server_client_key_callback
                as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
        ),
        event as *mut ::core::ffi::c_void,
    );
    if !after.is_null() {
        (*event).client = c;
        (*c).references += 1;
        item = cmdq_insert_after(after, item);
        if !next.is_null() {
            *next = item;
        }
        return 1 as ::core::ffi::c_int;
    }
    cmdq_append(c, item);
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_handle_key(
    mut c: *mut client,
    mut event: *mut key_event,
) -> ::core::ffi::c_int {
    return server_client_handle_key0(
        c,
        event,
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<*mut cmdq_item>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn server_client_handle_key_after(
    mut c: *mut client,
    mut event: *mut key_event,
    mut after: *mut cmdq_item,
    mut next: *mut *mut cmdq_item,
) -> ::core::ffi::c_int {
    return server_client_handle_key0(c, event, after, next);
}
#[no_mangle]
pub unsafe extern "C" fn server_client_loop() {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
    while !w.is_null() {
        server_client_check_window_resize(w);
        w = windows_RB_NEXT(w);
    }
    w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
    while !w.is_null() {
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if (*wp).flags & PANE_STYLECHANGED != 0 {
                wme = (*wp).modes.tqh_first;
                if !wme.is_null() && (*(*wme).mode).style_changed.is_some() {
                    (*(*wme).mode)
                        .style_changed
                        .expect("non-null function pointer")(wme);
                }
            }
            wp = (*wp).entry.tqe_next;
        }
        w = windows_RB_NEXT(w);
    }
    c = clients.tqh_first;
    while !c.is_null() {
        server_client_check_exit(c, 0 as ::core::ffi::c_int);
        if !(*c).session.is_null() && !(*(*c).session).curw.is_null() {
            server_client_check_modes(c);
            server_client_check_redraw(c);
            server_client_reset_state(c);
        }
        c = (*c).entry.tqe_next;
    }
    w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
    while !w.is_null() {
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if (*wp).fd != -(1 as ::core::ffi::c_int) {
                server_client_check_pane_resize(wp);
                server_client_check_pane_buffer(wp);
            }
            (*wp).flags &= !(PANE_REDRAW | PANE_REDRAWSCROLLBAR | PANE_ACTIVITY);
            wp = (*wp).entry.tqe_next;
        }
        check_window_name(w);
        w = windows_RB_NEXT(w);
    }
    w = windows_RB_MINMAX(&raw mut windows, RB_NEGINF);
    while !w.is_null() {
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            window_pane_send_theme_update(wp);
            wp = (*wp).entry.tqe_next;
        }
        w = windows_RB_NEXT(w);
    }
}
unsafe extern "C" fn server_client_check_window_resize(mut w: *mut window) {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if !(*w).flags & WINDOW_RESIZE != 0 {
        return;
    }
    wl = (*w).winlinks.tqh_first;
    while !wl.is_null() {
        if (*(*wl).session).attached != 0 as u_int && (*(*wl).session).curw == wl {
            break;
        }
        wl = (*wl).wentry.tqe_next;
    }
    if wl.is_null() {
        return;
    }
    log_debug(
        b"%s: resizing window @%u\0" as *const u8 as *const ::core::ffi::c_char,
        b"server_client_check_window_resize\0" as *const u8 as *const ::core::ffi::c_char,
        (*w).id,
    );
    resize_window(
        w,
        (*w).new_sx,
        (*w).new_sy,
        (*w).new_xpixel as ::core::ffi::c_int,
        (*w).new_ypixel as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn server_client_resize_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut wp: *mut window_pane = data as *mut window_pane;
    log_debug(
        b"%s: %%%u resize timer expired\0" as *const u8 as *const ::core::ffi::c_char,
        b"server_client_resize_timer\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
    event_del(&raw mut (*wp).resize_timer);
}
unsafe extern "C" fn server_client_check_pane_resize(mut wp: *mut window_pane) {
    let mut r: *mut window_pane_resize = ::core::ptr::null_mut::<window_pane_resize>();
    let mut first: *mut window_pane_resize = ::core::ptr::null_mut::<window_pane_resize>();
    let mut last: *mut window_pane_resize = ::core::ptr::null_mut::<window_pane_resize>();
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 250000 as __suseconds_t,
    };
    if (*wp).resize_queue.tqh_first.is_null() {
        return;
    }
    if event_initialized(&raw mut (*wp).resize_timer) == 0 {
        event_set(
            &raw mut (*wp).resize_timer,
            -(1 as ::core::ffi::c_int),
            0 as ::core::ffi::c_short,
            Some(
                server_client_resize_timer
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        ::core::ffi::c_short,
                        *mut ::core::ffi::c_void,
                    ) -> (),
            ),
            wp as *mut ::core::ffi::c_void,
        );
    }
    if event_pending(
        &raw mut (*wp).resize_timer,
        EV_TIMEOUT as ::core::ffi::c_short,
        ::core::ptr::null_mut::<timeval>(),
    ) != 0
    {
        return;
    }
    log_debug(
        b"%s: %%%u needs to be resized\0" as *const u8 as *const ::core::ffi::c_char,
        b"server_client_check_pane_resize\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
    );
    r = (*wp).resize_queue.tqh_first;
    while !r.is_null() {
        log_debug(
            b"queued resize: %ux%u -> %ux%u\0" as *const u8 as *const ::core::ffi::c_char,
            (*r).osx,
            (*r).osy,
            (*r).sx,
            (*r).sy,
        );
        r = (*r).entry.tqe_next;
    }
    first = (*wp).resize_queue.tqh_first;
    last = *(*((*wp).resize_queue.tqh_last as *mut window_pane_resizes)).tqh_last;
    if first == last {
        window_pane_send_resize(wp, (*first).sx, (*first).sy);
        if !(*first).entry.tqe_next.is_null() {
            (*(*first).entry.tqe_next).entry.tqe_prev = (*first).entry.tqe_prev;
        } else {
            (*wp).resize_queue.tqh_last = (*first).entry.tqe_prev;
        }
        *(*first).entry.tqe_prev = (*first).entry.tqe_next;
        free(first as *mut ::core::ffi::c_void);
    } else if (*last).sx != (*first).osx || (*last).sy != (*first).osy {
        window_pane_send_resize(wp, (*last).sx, (*last).sy);
        window_pane_clear_resizes(wp, ::core::ptr::null_mut::<window_pane_resize>());
    } else {
        r = *(*((*last).entry.tqe_prev as *mut window_pane_resizes)).tqh_last;
        window_pane_send_resize(wp, (*r).sx, (*r).sy);
        window_pane_clear_resizes(wp, last);
        tv.tv_usec = 10000 as __suseconds_t;
    }
    event_add(&raw mut (*wp).resize_timer, &raw mut tv);
}
unsafe extern "C" fn server_client_check_pane_buffer(mut wp: *mut window_pane) {
    let mut evb: *mut evbuffer = (*(*wp).event).input;
    let mut minimum: size_t = 0;
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut wpo: *mut window_pane_offset = ::core::ptr::null_mut::<window_pane_offset>();
    let mut off: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut flag: ::core::ffi::c_int = 0;
    let mut attached_clients: u_int = 0 as u_int;
    let mut new_size: size_t = 0;
    minimum = (*wp).offset.used;
    if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) && (*wp).pipe_offset.used < minimum {
        minimum = (*wp).pipe_offset.used;
    }
    c = clients.tqh_first;
    while !c.is_null() {
        if !(*c).session.is_null() {
            attached_clients = attached_clients.wrapping_add(1);
            if !(*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                off = 0 as ::core::ffi::c_int;
            } else {
                wpo = control_pane_offset(c, wp, &raw mut flag);
                if wpo.is_null() {
                    if flag == 0 {
                        off = 0 as ::core::ffi::c_int;
                    }
                } else {
                    if flag == 0 {
                        off = 0 as ::core::ffi::c_int;
                    }
                    window_pane_get_new_data(wp, wpo, &raw mut new_size);
                    log_debug(
                        b"%s: %s has %zu bytes used and %zu left for %%%u\0" as *const u8
                            as *const ::core::ffi::c_char,
                        b"server_client_check_pane_buffer\0" as *const u8
                            as *const ::core::ffi::c_char,
                        (*c).name,
                        (*wpo).used.wrapping_sub((*wp).base_offset),
                        new_size,
                        (*wp).id,
                    );
                    if (*wpo).used < minimum {
                        minimum = (*wpo).used;
                    }
                }
            }
        }
        c = (*c).entry.tqe_next;
    }
    if attached_clients == 0 as u_int {
        off = 0 as ::core::ffi::c_int;
    }
    minimum = minimum.wrapping_sub((*wp).base_offset);
    if !(minimum == 0 as size_t) {
        log_debug(
            b"%s: %%%u has %zu minimum (of %zu) bytes used\0" as *const u8
                as *const ::core::ffi::c_char,
            b"server_client_check_pane_buffer\0" as *const u8 as *const ::core::ffi::c_char,
            (*wp).id,
            minimum,
            evbuffer_get_length(evb),
        );
        evbuffer_drain(evb, minimum);
        if (*wp).base_offset > (SIZE_MAX as size_t).wrapping_sub(minimum) {
            log_debug(
                b"%s: %%%u base offset has wrapped\0" as *const u8 as *const ::core::ffi::c_char,
                b"server_client_check_pane_buffer\0" as *const u8 as *const ::core::ffi::c_char,
                (*wp).id,
            );
            (*wp).offset.used = (*wp).offset.used.wrapping_sub((*wp).base_offset);
            if (*wp).pipe_fd != -(1 as ::core::ffi::c_int) {
                (*wp).pipe_offset.used = (*wp).pipe_offset.used.wrapping_sub((*wp).base_offset);
            }
            c = clients.tqh_first;
            while !c.is_null() {
                if !((*c).session.is_null() || !(*c).flags & CLIENT_CONTROL as uint64_t != 0) {
                    wpo = control_pane_offset(c, wp, &raw mut flag);
                    if !wpo.is_null() && flag == 0 {
                        (*wpo).used = (*wpo).used.wrapping_sub((*wp).base_offset);
                    }
                }
                c = (*c).entry.tqe_next;
            }
            (*wp).base_offset = minimum;
        } else {
            (*wp).base_offset = (*wp).base_offset.wrapping_add(minimum);
        }
    }
    log_debug(
        b"%s: pane %%%u is %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"server_client_check_pane_buffer\0" as *const u8 as *const ::core::ffi::c_char,
        (*wp).id,
        if off != 0 {
            b"off\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"on\0" as *const u8 as *const ::core::ffi::c_char
        },
    );
    if off != 0 {
        bufferevent_disable((*wp).event, EV_READ as ::core::ffi::c_short);
    } else {
        bufferevent_enable((*wp).event, EV_READ as ::core::ffi::c_short);
    };
}
unsafe extern "C" fn server_client_prompt_cursor(
    mut c: *mut client,
    mut wp: *mut window_pane,
    mut mode: *mut ::core::ffi::c_int,
    mut cx: *mut u_int,
    mut cy: *mut u_int,
) -> ::core::ffi::c_int {
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut px: ::core::ffi::c_int = 0;
    let mut py: ::core::ffi::c_int = 0;
    if window_pane_has_prompt(wp) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    *mode &= !MODE_CURSOR;
    tty_window_offset(tty, &raw mut ox, &raw mut oy, &raw mut sx, &raw mut sy);
    if status_at_line(c) == 0 as ::core::ffi::c_int {
        py = (*wp).yoff;
    } else {
        py = ((*wp).yoff as u_int)
            .wrapping_add((*wp).sy)
            .wrapping_sub(1 as u_int) as ::core::ffi::c_int;
    }
    px = ((*wp).xoff as u_int).wrapping_add((*wp).prompt_cx) as ::core::ffi::c_int;
    if px < ox as ::core::ffi::c_int
        || px > ox.wrapping_add(sx) as ::core::ffi::c_int
        || py < oy as ::core::ffi::c_int
        || py > oy.wrapping_add(sy) as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    *cx = (px as u_int).wrapping_sub(ox);
    *cy = (py as u_int).wrapping_sub(oy);
    r = window_visible_ranges(
        wp,
        *cx as ::core::ffi::c_int,
        *cy as ::core::ffi::c_int,
        1 as u_int,
        ::core::ptr::null_mut::<visible_ranges>(),
    );
    if window_position_is_visible(r, *cx) != 0 {
        if status_at_line(c) == 0 as ::core::ffi::c_int {
            *cy = (*cy).wrapping_add(status_line_size(c));
        }
        *mode |= MODE_CURSOR;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_reset_state(mut c: *mut client) {
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut w: *mut window = (*(*(*c).session).curw).window;
    let mut wp: *mut window_pane = (*w).active;
    let mut loop_0: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    let mut oo: *mut options = (*(*c).session).options;
    let mut mode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cursor: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0;
    let mut pane_mode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cx: u_int = 0 as u_int;
    let mut cy: u_int = 0 as u_int;
    let mut ox: u_int = 0;
    let mut oy: u_int = 0;
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut prompt: u_int = 0 as u_int;
    let mut sb_w: u_int = 0;
    let mut r: *mut visible_ranges = ::core::ptr::null_mut::<visible_ranges>();
    if (*c).flags & (CLIENT_CONTROL | CLIENT_SUSPENDED) as uint64_t != 0 {
        return;
    }
    flags = (*tty).flags & TTY_BLOCK;
    (*tty).flags &= !TTY_BLOCK;
    if (*c).overlay_draw.is_some() {
        if (*c).overlay_mode.is_some() {
            s = (*c).overlay_mode.expect("non-null function pointer")(
                c,
                (*c).overlay_data,
                &raw mut cx,
                &raw mut cy,
            );
        }
    } else if !(*w).menu.is_null() {
        menu_get_cursor((*w).menu, &raw mut cx, &raw mut cy);
        s = menu_screen((*w).menu);
    } else if !wp.is_null() && (*c).prompt.is_null() {
        s = (*wp).screen;
    } else {
        s = (*c).status.active;
    }
    if !s.is_null() {
        mode = (*s).mode;
    }
    if log_get_level() != 0 as ::core::ffi::c_int {
        log_debug(
            b"%s: client %s mode %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"server_client_reset_state\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            screen_mode_to_string(mode),
        );
    }
    tty_region_off(tty);
    tty_margin_off(tty);
    if !(*c).prompt.is_null() {
        prompt = 1 as u_int;
        status_prompt_cursor(c, &raw mut cx, &raw mut cy);
    } else if !wp.is_null() && (*c).overlay_draw.is_none() {
        if !(*w).menu.is_null() {
            tty_window_offset(tty, &raw mut ox, &raw mut oy, &raw mut sx, &raw mut sy);
            if cx < ox || cx >= ox.wrapping_add(sx) || cy < oy || cy >= oy.wrapping_add(sy) {
                mode &= !MODE_CURSOR;
            } else {
                cx = cx.wrapping_sub(ox);
                cy = cy.wrapping_sub(oy);
                if status_at_line(c) == 0 as ::core::ffi::c_int {
                    cy = cy.wrapping_add(status_line_size(c));
                }
            }
            prompt = 1 as u_int;
        } else {
            prompt = server_client_prompt_cursor(c, wp, &raw mut mode, &raw mut cx, &raw mut cy)
                as u_int;
        }
        if prompt == 0 {
            cursor = 0 as ::core::ffi::c_int;
            pane_mode = (*wp).base.mode;
            tty_window_offset(tty, &raw mut ox, &raw mut oy, &raw mut sx, &raw mut sy);
            if (*wp).xoff + (*s).cx as ::core::ffi::c_int >= ox as ::core::ffi::c_int
                && (*wp).xoff + (*s).cx as ::core::ffi::c_int
                    <= ox as ::core::ffi::c_int + sx as ::core::ffi::c_int
                && (*wp).yoff + (*s).cy as ::core::ffi::c_int >= oy as ::core::ffi::c_int
                && (*wp).yoff + (*s).cy as ::core::ffi::c_int
                    <= oy as ::core::ffi::c_int + sy as ::core::ffi::c_int
            {
                cursor = 1 as ::core::ffi::c_int;
                cx = ((*wp).xoff + (*s).cx as ::core::ffi::c_int - ox as ::core::ffi::c_int)
                    as u_int;
                cy = ((*wp).yoff + (*s).cy as ::core::ffi::c_int - oy as ::core::ffi::c_int)
                    as u_int;
                r = window_visible_ranges(
                    wp,
                    cx as ::core::ffi::c_int,
                    cy as ::core::ffi::c_int,
                    1 as u_int,
                    ::core::ptr::null_mut::<visible_ranges>(),
                );
                if window_position_is_visible(r, cx) == 0 {
                    cursor = 0 as ::core::ffi::c_int;
                }
                if window_pane_scrollbar_overlay_visible(wp) != 0 {
                    sb_w = (*wp).scrollbar_style.width as u_int;
                    if sb_w > (*wp).sx {
                        sb_w = (*wp).sx;
                    }
                    if sb_w != 0 as u_int && (*w).sb_pos == PANE_SCROLLBARS_LEFT {
                        if (*s).cx < sb_w {
                            cursor = 0 as ::core::ffi::c_int;
                        }
                    } else if sb_w != 0 as u_int && (*s).cx >= (*wp).sx.wrapping_sub(sb_w) {
                        cursor = 0 as ::core::ffi::c_int;
                    }
                }
                if status_at_line(c) == 0 as ::core::ffi::c_int {
                    cy = cy.wrapping_add(status_line_size(c));
                }
            }
            if cursor == 0 {
                mode &= !MODE_CURSOR;
            }
        }
    } else if (*c).overlay_mode.is_none() || s.is_null() {
        mode &= !MODE_CURSOR;
    }
    if !pane_mode & MODE_SYNC != 0 {
        log_debug(
            b"%s: cursor to %u,%u\0" as *const u8 as *const ::core::ffi::c_char,
            b"server_client_reset_state\0" as *const u8 as *const ::core::ffi::c_char,
            cx,
            cy,
        );
        tty_cursor(tty, cx, cy);
    } else {
        mode &= !CURSOR_MODES;
        mode |= (*tty).mode & CURSOR_MODES;
        s = ::core::ptr::null_mut::<screen>();
    }
    if options_get_number(oo, b"mouse\0" as *const u8 as *const ::core::ffi::c_char) != 0 {
        if (*c).overlay_draw.is_none() && (*w).menu.is_null() {
            mode &= !ALL_MOUSE_MODES;
            loop_0 = (*w).panes.tqh_first;
            while !loop_0.is_null() {
                if (*(*loop_0).screen).mode & MODE_MOUSE_ALL != 0 {
                    mode |= MODE_MOUSE_ALL;
                }
                loop_0 = (*loop_0).entry.tqe_next;
            }
        }
        if options_get_number(
            oo,
            b"focus-follows-mouse\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
            || (*w).sb == PANE_SCROLLBARS_MODAL
            || (*w).sb == PANE_SCROLLBARS_AUTOHIDE
        {
            mode |= MODE_MOUSE_ALL;
        } else if !mode & MODE_MOUSE_ALL != 0 {
            mode |= MODE_MOUSE_BUTTON;
        }
    }
    if (*c).overlay_draw.is_none() && prompt != 0 {
        mode &= !MODE_BRACKETPASTE;
    }
    tty_update_mode(tty, mode, s);
    tty_reset(tty);
    tty_sync_end(tty);
    (*tty).flags |= flags;
}
unsafe extern "C" fn server_client_repeat_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = data as *mut client;
    if (*c).flags & CLIENT_REPEAT as uint64_t != 0 {
        server_client_set_key_table(c, ::core::ptr::null::<::core::ffi::c_char>());
        (*c).flags &= !CLIENT_REPEAT as uint64_t;
        server_status_client(c);
    }
}
unsafe extern "C" fn server_client_click_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = data as *mut client;
    let mut event: *mut key_event = ::core::ptr::null_mut::<key_event>();
    log_debug(b"click timer expired\0" as *const u8 as *const ::core::ffi::c_char);
    if (*c).flags & CLIENT_TRIPLECLICK as uint64_t != 0 {
        event =
            xcalloc(1 as size_t, ::core::mem::size_of::<key_event>() as size_t) as *mut key_event;
        (*event).key = KEYC_DOUBLECLICK as ::core::ffi::c_ulong as key_code;
        memcpy(
            &raw mut (*event).m as *mut ::core::ffi::c_void,
            &raw mut (*c).click_event as *const ::core::ffi::c_void,
            ::core::mem::size_of::<mouse_event>() as size_t,
        );
        if server_client_handle_key(c, event) == 0 {
            free((*event).buf as *mut ::core::ffi::c_void);
            free(event as *mut ::core::ffi::c_void);
        }
    }
    (*c).flags &= !(CLIENT_DOUBLECLICK | CLIENT_TRIPLECLICK) as uint64_t;
}
unsafe extern "C" fn server_client_start_exit_timer(mut c: *mut client) {
    let mut tv: timeval = timeval {
        tv_sec: 10 as __time_t,
        tv_usec: 0,
    };
    if event_pending(
        &raw mut (*c).exit_timer,
        EV_TIMEOUT as ::core::ffi::c_short,
        ::core::ptr::null_mut::<timeval>(),
    ) == 0
    {
        event_add(&raw mut (*c).exit_timer, &raw mut tv);
    }
}
unsafe extern "C" fn server_client_exit_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    let mut c: *mut client = data as *mut client;
    if (*c).flags & (CLIENT_DEAD | CLIENT_SUSPENDED) as uint64_t != 0 {
        return;
    }
    if (*c).flags & CLIENT_EXITED as uint64_t != 0 {
        log_debug(
            b"%s: %s took too long to exit\0" as *const u8 as *const ::core::ffi::c_char,
            b"server_client_exit_timer\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
        server_client_lost(c);
    } else if (*c).flags & CLIENT_EXIT as uint64_t != 0 {
        log_debug(
            b"%s: %s took too long to flush\0" as *const u8 as *const ::core::ffi::c_char,
            b"server_client_exit_timer\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
        server_client_check_exit(c, 1 as ::core::ffi::c_int);
    }
}
unsafe extern "C" fn server_client_check_exit(mut c: *mut client, mut force: ::core::ffi::c_int) {
    let mut cf: *mut client_file = ::core::ptr::null_mut::<client_file>();
    let mut name: *const ::core::ffi::c_char = (*c).exit_session;
    let mut data: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut size: size_t = 0;
    let mut msize: size_t = 0;
    if (*c).flags & (CLIENT_DEAD | CLIENT_EXITED) as uint64_t != 0 {
        return;
    }
    if !(*c).flags & CLIENT_EXIT as uint64_t != 0 {
        return;
    }
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        if force != 0 {
            control_discard_all(c);
        } else {
            control_discard(c);
            if control_all_done(c) == 0 {
                server_client_start_exit_timer(c);
                return;
            }
        }
    }
    if force == 0 {
        cf = client_files_RB_MINMAX(&raw mut (*c).files, RB_NEGINF);
        while !cf.is_null() {
            if evbuffer_get_length((*cf).buffer) != 0 as size_t {
                server_client_start_exit_timer(c);
                return;
            }
            cf = client_files_RB_NEXT(cf);
        }
    }
    (*c).flags |= CLIENT_EXITED as uint64_t;
    event_del(&raw mut (*c).exit_timer);
    server_client_start_exit_timer(c);
    match (*c).exit_type as ::core::ffi::c_uint {
        0 => {
            if !(*c).exit_message.is_null() {
                msize = strlen((*c).exit_message).wrapping_add(1 as size_t);
            } else {
                msize = 0 as size_t;
            }
            size = (::core::mem::size_of::<::core::ffi::c_int>() as usize)
                .wrapping_add(msize as usize) as size_t;
            data = xmalloc(size) as *mut ::core::ffi::c_char;
            memcpy(
                data as *mut ::core::ffi::c_void,
                &raw mut (*c).retval as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
            );
            if !(*c).exit_message.is_null() {
                memcpy(
                    data.offset(::core::mem::size_of::<::core::ffi::c_int>() as usize as isize)
                        as *mut ::core::ffi::c_void,
                    (*c).exit_message as *const ::core::ffi::c_void,
                    msize,
                );
            }
            proc_send(
                (*c).peer,
                MSG_EXIT,
                -(1 as ::core::ffi::c_int),
                data as *const ::core::ffi::c_void,
                size,
            );
            free(data as *mut ::core::ffi::c_void);
        }
        1 => {
            proc_send(
                (*c).peer,
                MSG_SHUTDOWN,
                -(1 as ::core::ffi::c_int),
                ::core::ptr::null::<::core::ffi::c_void>(),
                0 as size_t,
            );
        }
        2 => {
            proc_send(
                (*c).peer,
                (*c).exit_msgtype,
                -(1 as ::core::ffi::c_int),
                name as *const ::core::ffi::c_void,
                strlen(name).wrapping_add(1 as size_t),
            );
        }
        _ => {}
    };
}
unsafe extern "C" fn server_client_redraw_timer(
    mut fd: ::core::ffi::c_int,
    mut events: ::core::ffi::c_short,
    mut data: *mut ::core::ffi::c_void,
) {
    log_debug(b"redraw timer fired\0" as *const u8 as *const ::core::ffi::c_char);
}
unsafe extern "C" fn server_client_check_modes(mut c: *mut client) {
    let mut w: *mut window = (*(*(*c).session).curw).window;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    if (*c).flags & (CLIENT_CONTROL | CLIENT_SUSPENDED) as uint64_t != 0 {
        return;
    }
    if !(*c).flags & CLIENT_REDRAWSTATUS as uint64_t != 0 {
        return;
    }
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        wme = (*wp).modes.tqh_first;
        if !wme.is_null() && (*(*wme).mode).update.is_some() {
            (*(*wme).mode).update.expect("non-null function pointer")(wme);
        }
        wp = (*wp).entry.tqe_next;
    }
}
unsafe extern "C" fn server_client_any_pane_redraw(mut c: *mut client) -> ::core::ffi::c_int {
    let mut s: *mut session = (*c).session;
    let mut w: *mut window = (*(*s).curw).window;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    if (*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
        return 1 as ::core::ffi::c_int;
    }
    wp = (*w).panes.tqh_first;
    while !wp.is_null() {
        if (*wp).flags & (PANE_REDRAW | PANE_REDRAWSCROLLBAR) != 0 {
            return 1 as ::core::ffi::c_int;
        }
        wp = (*wp).entry.tqe_next;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_check_redraw(mut c: *mut client) {
    let mut s: *mut session = (*c).session;
    let mut tty: *mut tty = &raw mut (*c).tty;
    let mut w: *mut window = (*(*s).curw).window;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut needed: ::core::ffi::c_int = 0;
    let mut tflags: ::core::ffi::c_int = 0;
    let mut mode: ::core::ffi::c_int = (*tty).mode;
    let mut tv: timeval = timeval {
        tv_sec: 0,
        tv_usec: 1000 as __suseconds_t,
    };
    static mut ev: event = event {
        ev_evcallback: event_callback {
            evcb_active_next: C2RustUnnamed_9 {
                tqe_next: ::core::ptr::null::<event_callback>() as *mut event_callback,
                tqe_prev: ::core::ptr::null::<*mut event_callback>() as *mut *mut event_callback,
            },
            evcb_flags: 0,
            evcb_pri: 0,
            evcb_closure: 0,
            evcb_cb_union: C2RustUnnamed_8 {
                evcb_callback: None,
            },
            evcb_arg: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        },
        ev_timeout_pos: C2RustUnnamed_6 {
            ev_next_with_common_timeout: C2RustUnnamed_7 {
                tqe_next: ::core::ptr::null::<event>() as *mut event,
                tqe_prev: ::core::ptr::null::<*mut event>() as *mut *mut event,
            },
        },
        ev_fd: 0,
        ev_base: ::core::ptr::null::<event_base>() as *mut event_base,
        ev_: C2RustUnnamed_1 {
            ev_io: C2RustUnnamed_4 {
                ev_io_next: C2RustUnnamed_5 {
                    le_next: ::core::ptr::null::<event>() as *mut event,
                    le_prev: ::core::ptr::null::<*mut event>() as *mut *mut event,
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
    };
    let mut n: size_t = 0;
    if (*c).flags & (CLIENT_CONTROL | CLIENT_SUSPENDED) as uint64_t != 0 {
        return;
    }
    if (*c).flags & CLIENT_ALLREDRAWFLAGS as uint64_t != 0 {
        log_debug(
            b"%s: redraw%s%s%s%s%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            if (*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
                b" window\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            },
            if (*c).flags & CLIENT_REDRAWSTATUS as uint64_t != 0 {
                b" status\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            },
            if (*c).flags & CLIENT_REDRAWBORDERS as uint64_t != 0 {
                b" borders\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            },
            if (*c).flags & CLIENT_REDRAWOVERLAY as uint64_t != 0 {
                b" overlay\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            },
            if (*c).flags & CLIENT_REDRAWMENU as uint64_t != 0 {
                b" menu\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            },
        );
    }
    needed = 0 as ::core::ffi::c_int;
    if (*c).flags as ::core::ffi::c_ulonglong
        & (CLIENT_ALLREDRAWFLAGS as ::core::ffi::c_ulonglong | CLIENT_REDRAWSCROLLBARS)
        != 0
    {
        needed = 1 as ::core::ffi::c_int;
    } else if server_client_any_pane_redraw(c) != 0 {
        needed = 1 as ::core::ffi::c_int;
    }
    if needed == 0 {
        (*c).flags &= !CLIENT_STATUSFORCE as uint64_t;
        return;
    }
    n = evbuffer_get_length((*tty).out);
    if n != 0 as size_t || (*tty).flags & TTY_BLOCK != 0 {
        if n != 0 as size_t {
            log_debug(
                b"%s: redraw deferred (%zu left)\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
                n,
            );
        } else {
            log_debug(
                b"%s: redraw deferred (blocked)\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
            );
        }
        if event_initialized(&raw mut ev) == 0 {
            event_set(
                &raw mut ev,
                -(1 as ::core::ffi::c_int),
                0 as ::core::ffi::c_short,
                Some(
                    server_client_redraw_timer
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            ::core::ffi::c_short,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
            );
        }
        if event_pending(
            &raw mut ev,
            EV_TIMEOUT as ::core::ffi::c_short,
            ::core::ptr::null_mut::<timeval>(),
        ) == 0
        {
            log_debug(b"redraw timer started\0" as *const u8 as *const ::core::ffi::c_char);
            event_add(&raw mut ev, &raw mut tv);
        }
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if (*wp).flags & PANE_REDRAW != 0 {
                (*c).flags |= CLIENT_REDRAWWINDOW as uint64_t;
                break;
            } else {
                if (*wp).flags & PANE_REDRAWSCROLLBAR != 0 {
                    (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong | CLIENT_REDRAWSCROLLBARS)
                        as uint64_t;
                }
                wp = (*wp).entry.tqe_next;
            }
        }
        return;
    }
    log_debug(
        b"%s: redraw needed\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
    );
    tflags = (*tty).flags & (TTY_BLOCK | TTY_FREEZE | TTY_NOCURSOR);
    (*tty).flags = (*tty).flags & !(TTY_BLOCK | TTY_FREEZE) | TTY_NOCURSOR;
    if !(*c).flags & CLIENT_REDRAWWINDOW as uint64_t != 0 {
        wp = (*w).panes.tqh_first;
        while !wp.is_null() {
            if (*wp).flags & PANE_REDRAW != 0 {
                log_debug(
                    b"%s: redraw pane %%%u\0" as *const u8 as *const ::core::ffi::c_char,
                    b"server_client_check_redraw\0" as *const u8 as *const ::core::ffi::c_char,
                    (*wp).id,
                );
                redraw_pane(c, wp);
            } else if (*wp).flags & PANE_REDRAWSCROLLBAR != 0
                || (*c).flags as ::core::ffi::c_ulonglong & CLIENT_REDRAWSCROLLBARS != 0
            {
                log_debug(
                    b"%s: redraw scrollbar %%%u\0" as *const u8 as *const ::core::ffi::c_char,
                    b"server_client_check_redraw\0" as *const u8 as *const ::core::ffi::c_char,
                    (*wp).id,
                );
                redraw_pane_scrollbar(c, wp);
            }
            wp = (*wp).entry.tqe_next;
        }
    }
    if (*c).flags & CLIENT_ALLREDRAWFLAGS as uint64_t != 0 {
        if options_get_number(
            (*s).options,
            b"set-titles\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0
        {
            server_client_set_title(c);
            server_client_set_path(c);
        }
        server_client_set_progress_bar(c);
        redraw_screen(c);
    }
    (*tty).flags = (*tty).flags & !TTY_NOCURSOR | tflags & TTY_NOCURSOR;
    tty_update_mode(tty, mode, ::core::ptr::null_mut::<screen>());
    (*tty).flags = (*tty).flags & !(TTY_BLOCK | TTY_FREEZE | TTY_NOCURSOR) | tflags;
    (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong
        & !(CLIENT_ALLREDRAWFLAGS as ::core::ffi::c_ulonglong
            | CLIENT_REDRAWSCROLLBARS
            | CLIENT_STATUSFORCE as ::core::ffi::c_ulonglong)) as uint64_t;
    (*c).redraw = evbuffer_get_length((*tty).out);
    log_debug(
        b"%s: redraw added %zu bytes\0" as *const u8 as *const ::core::ffi::c_char,
        (*c).name,
        (*c).redraw,
    );
}
unsafe extern "C" fn server_client_set_title(mut c: *mut client) {
    let mut s: *mut session = (*c).session;
    let mut template: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut title: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    template = options_get_string(
        (*s).options,
        b"set-titles-string\0" as *const u8 as *const ::core::ffi::c_char,
    );
    ft = format_create(
        c,
        ::core::ptr::null_mut::<cmdq_item>(),
        FORMAT_NONE,
        0 as ::core::ffi::c_int,
    );
    format_defaults(
        ft,
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        ::core::ptr::null_mut::<window_pane>(),
    );
    title = format_expand_time(ft, template);
    if (*c).title.is_null() || strcmp(title, (*c).title) != 0 as ::core::ffi::c_int {
        free((*c).title as *mut ::core::ffi::c_void);
        (*c).title = xstrdup(title);
        tty_set_title(&raw mut (*c).tty, (*c).title);
    }
    free(title as *mut ::core::ffi::c_void);
    format_free(ft);
}
unsafe extern "C" fn server_client_set_path(mut c: *mut client) {
    let mut s: *mut session = (*c).session;
    let mut path: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*s).curw.is_null() || (*(*(*s).curw).window).active.is_null() {
        return;
    }
    if (*(*(*(*s).curw).window).active).base.path.is_null() {
        path = b"\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        path = (*(*(*(*s).curw).window).active).base.path;
    }
    if (*c).path.is_null() || strcmp(path, (*c).path) != 0 as ::core::ffi::c_int {
        free((*c).path as *mut ::core::ffi::c_void);
        (*c).path = xstrdup(path);
        tty_set_path(&raw mut (*c).tty, (*c).path);
    }
}
unsafe extern "C" fn server_client_set_progress_bar(mut c: *mut client) {
    let mut s: *mut session = (*c).session;
    let mut pane_pb: *mut progress_bar = ::core::ptr::null_mut::<progress_bar>();
    if (*s).curw.is_null() || (*(*(*s).curw).window).active.is_null() {
        return;
    }
    pane_pb = &raw mut (*(*(*(*s).curw).window).active).base.progress_bar;
    if (*pane_pb).state as ::core::ffi::c_uint == (*c).progress_bar.state as ::core::ffi::c_uint
        && (*pane_pb).progress == (*c).progress_bar.progress
    {
        return;
    }
    memcpy(
        &raw mut (*c).progress_bar as *mut ::core::ffi::c_void,
        pane_pb as *const ::core::ffi::c_void,
        ::core::mem::size_of::<progress_bar>() as size_t,
    );
    tty_set_progress_bar(&raw mut (*c).tty, &raw mut (*c).progress_bar);
}
unsafe extern "C" fn server_client_dispatch(
    mut imsg: *mut imsg,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut current_block: u64;
    let mut c: *mut client = arg as *mut client;
    let mut datalen: ssize_t = 0;
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut old_sx: u_int = 0;
    let mut old_sy: u_int = 0;
    if (*c).flags & CLIENT_DEAD as uint64_t != 0 {
        return;
    }
    if imsg.is_null() {
        server_client_lost(c);
        return;
    }
    datalen = ((*imsg).hdr.len as usize).wrapping_sub(IMSG_HEADER_SIZE) as ssize_t;
    match (*imsg).hdr.type_0 {
        107 | 108 | 105 | 109 | 100 | 111 | 104 | 110 | 101 | 112 | 102 | 106 => {
            if server_client_dispatch_identify(c, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        200 => {
            if server_client_dispatch_command(c, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        208 => {
            if datalen != 0 as ssize_t {
                current_block = 13639960948656484833;
            } else if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                current_block = 14945149239039849694;
            } else {
                server_client_update_latest(c);
                old_sx = (*c).tty.sx;
                old_sy = (*c).tty.sy;
                tty_resize(&raw mut (*c).tty);
                tty_repeat_requests(&raw mut (*c).tty, 0 as ::core::ffi::c_int);
                recalculate_sizes();
                if (*c).overlay_resize.is_none() {
                    server_client_clear_overlay(c);
                } else {
                    (*c).overlay_resize.expect("non-null function pointer")(c, (*c).overlay_data);
                }
                server_redraw_client(c);
                if !(*c).session.is_null() {
                    server_client_fire_resized(c, old_sx, old_sy);
                }
                current_block = 14945149239039849694;
            }
        }
        205 => {
            if datalen != 0 as ssize_t {
                current_block = 13639960948656484833;
            } else {
                server_client_set_session(c, ::core::ptr::null_mut::<session>());
                recalculate_sizes();
                tty_close(&raw mut (*c).tty);
                proc_send(
                    (*c).peer,
                    MSG_EXITED,
                    -(1 as ::core::ffi::c_int),
                    ::core::ptr::null::<::core::ffi::c_void>(),
                    0 as size_t,
                );
                current_block = 14945149239039849694;
            }
        }
        216 | 215 => {
            if datalen != 0 as ssize_t {
                current_block = 13639960948656484833;
            } else if (*c).flags & CLIENT_SUSPENDED as uint64_t == 0 {
                current_block = 14945149239039849694;
            } else {
                (*c).flags &= !CLIENT_SUSPENDED as uint64_t;
                if (*c).fd == -(1 as ::core::ffi::c_int) || (*c).session.is_null() {
                    current_block = 14945149239039849694;
                } else {
                    s = (*c).session;
                    if gettimeofday(&raw mut (*c).activity_time, NULL) != 0 as ::core::ffi::c_int {
                        fatal(b"gettimeofday failed\0" as *const u8 as *const ::core::ffi::c_char);
                    }
                    tty_start_tty(&raw mut (*c).tty);
                    server_redraw_client(c);
                    recalculate_sizes();
                    if !s.is_null() {
                        session_update_activity(s, &raw mut (*c).activity_time);
                    }
                    current_block = 14945149239039849694;
                }
            }
        }
        209 => {
            if datalen != 0 as ssize_t {
                current_block = 13639960948656484833;
            } else if server_client_dispatch_shell(c) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        305 => {
            if file_write_ready(&raw mut (*c).files, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        308 => {
            if file_write_done(&raw mut (*c).files, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        301 => {
            if file_read_data(&raw mut (*c).files, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        302 => {
            if file_read_done(&raw mut (*c).files, imsg) != 0 as ::core::ffi::c_int {
                current_block = 13639960948656484833;
            } else {
                current_block = 14945149239039849694;
            }
        }
        _ => {
            current_block = 14945149239039849694;
        }
    }
    match current_block {
        14945149239039849694 => return,
        _ => {
            log_debug(
                b"client %p invalid message type %d\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                (*imsg).hdr.type_0,
            );
            proc_kill_peer((*c).peer);
            return;
        }
    };
}
unsafe extern "C" fn server_client_read_only(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    cmdq_error(
        item,
        b"client is read-only\0" as *const u8 as *const ::core::ffi::c_char,
    );
    return CMD_RETURN_ERROR;
}
unsafe extern "C" fn server_client_default_command(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut c: *mut client = cmdq_get_client(item);
    let mut cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    cmdlist = options_get_command(
        global_options,
        b"default-client-command\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if (*c).flags & CLIENT_READONLY as uint64_t != 0
        && cmd_list_all_have(cmdlist, CMD_READONLY) == 0
    {
        new_item = cmdq_get_callback1(
            b"server_client_read_only\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                server_client_read_only
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        );
    } else {
        new_item = cmdq_get_command(cmdlist, ::core::ptr::null_mut::<cmdq_state>());
    }
    cmdq_insert_after(item, new_item);
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn server_client_command_done(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut c: *mut client = cmdq_get_client(item);
    if !(*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
        (*c).flags |= CLIENT_EXIT as uint64_t;
    } else if !(*c).flags & CLIENT_EXIT as uint64_t != 0 {
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_ready(c);
        }
        tty_send_requests(&raw mut (*c).tty);
    }
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn server_client_dispatch_command(
    mut c: *mut client,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let mut data: msg_command = msg_command { argc: 0 };
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut argc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut argv: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut values: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut new_item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    if (*c).flags & CLIENT_EXIT as uint64_t != 0 {
        return 0 as ::core::ffi::c_int;
    }
    if ((*imsg).hdr.len as usize).wrapping_sub(IMSG_HEADER_SIZE)
        < ::core::mem::size_of::<msg_command>() as usize
    {
        return -(1 as ::core::ffi::c_int);
    }
    memcpy(
        &raw mut data as *mut ::core::ffi::c_void,
        (*imsg).data,
        ::core::mem::size_of::<msg_command>() as size_t,
    );
    buf = ((*imsg).data as *mut ::core::ffi::c_char)
        .offset(::core::mem::size_of::<msg_command>() as usize as isize);
    len = ((*imsg).hdr.len as usize)
        .wrapping_sub(IMSG_HEADER_SIZE)
        .wrapping_sub(::core::mem::size_of::<msg_command>() as usize) as size_t;
    if len > 0 as size_t
        && *buf.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int != '\0' as i32
    {
        return -(1 as ::core::ffi::c_int);
    }
    if cmd_unpack_argv(buf, len, data.argc, &raw mut argv) != 0 as ::core::ffi::c_int {
        cause = xstrdup(b"command too long\0" as *const u8 as *const ::core::ffi::c_char);
    } else {
        argc = data.argc;
        if argc == 0 as ::core::ffi::c_int {
            new_item = cmdq_get_callback1(
                b"server_client_default_command\0" as *const u8 as *const ::core::ffi::c_char,
                Some(
                    server_client_default_command
                        as unsafe extern "C" fn(
                            *mut cmdq_item,
                            *mut ::core::ffi::c_void,
                        ) -> cmd_retval,
                ),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
            );
            current_block = 13472856163611868459;
        } else {
            values = args_from_vector(argc, argv);
            pr = cmd_parse_from_arguments(
                values,
                argc as u_int,
                ::core::ptr::null_mut::<cmd_parse_input>(),
            );
            match (*pr).status as ::core::ffi::c_uint {
                0 => {
                    cause = (*pr).error;
                    current_block = 12680788052841528405;
                }
                1 | _ => {
                    args_free_values(values, argc as u_int);
                    free(values as *mut ::core::ffi::c_void);
                    cmd_free_argv(argc, argv);
                    if (*c).flags & CLIENT_READONLY as uint64_t != 0
                        && cmd_list_all_have((*pr).cmdlist, CMD_READONLY) == 0
                    {
                        new_item = cmdq_get_callback1(
                            b"server_client_read_only\0" as *const u8 as *const ::core::ffi::c_char,
                            Some(
                                server_client_read_only
                                    as unsafe extern "C" fn(
                                        *mut cmdq_item,
                                        *mut ::core::ffi::c_void,
                                    )
                                        -> cmd_retval,
                            ),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        );
                    } else {
                        new_item =
                            cmdq_get_command((*pr).cmdlist, ::core::ptr::null_mut::<cmdq_state>());
                    }
                    cmd_list_free((*pr).cmdlist);
                    current_block = 13472856163611868459;
                }
            }
        }
        match current_block {
            12680788052841528405 => {}
            _ => {
                cmdq_append(c, new_item);
                cmdq_append(
                    c,
                    cmdq_get_callback1(
                        b"server_client_command_done\0" as *const u8 as *const ::core::ffi::c_char,
                        Some(
                            server_client_command_done
                                as unsafe extern "C" fn(
                                    *mut cmdq_item,
                                    *mut ::core::ffi::c_void,
                                )
                                    -> cmd_retval,
                        ),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ),
                );
                return 0 as ::core::ffi::c_int;
            }
        }
    }
    cmd_free_argv(argc, argv);
    cmdq_append(c, cmdq_get_error(cause));
    free(cause as *mut ::core::ffi::c_void);
    (*c).flags |= CLIENT_EXIT as uint64_t;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_dispatch_identify(
    mut c: *mut client,
    mut imsg: *mut imsg,
) -> ::core::ffi::c_int {
    let mut data: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut home: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut datalen: size_t = 0;
    let mut flags: ::core::ffi::c_int = 0;
    let mut feat: ::core::ffi::c_int = 0;
    let mut longflags: uint64_t = 0;
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*c).flags & CLIENT_IDENTIFIED as uint64_t != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    data = (*imsg).data as *const ::core::ffi::c_char;
    datalen = ((*imsg).hdr.len as usize).wrapping_sub(IMSG_HEADER_SIZE) as size_t;
    match (*imsg).hdr.type_0 {
        109 => {
            if datalen != ::core::mem::size_of::<::core::ffi::c_int>() as usize {
                return -(1 as ::core::ffi::c_int);
            }
            memcpy(
                &raw mut feat as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
            );
            (*c).term_features |= feat;
            log_debug(
                b"client %p IDENTIFY_FEATURES %s\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                tty_get_features(feat),
            );
        }
        100 => {
            if datalen != ::core::mem::size_of::<::core::ffi::c_int>() as usize {
                return -(1 as ::core::ffi::c_int);
            }
            memcpy(
                &raw mut flags as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_int>() as size_t,
            );
            (*c).flags |= flags as uint64_t;
            log_debug(
                b"client %p IDENTIFY_FLAGS %#x\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                flags,
            );
        }
        111 => {
            if datalen != ::core::mem::size_of::<uint64_t>() as usize {
                return -(1 as ::core::ffi::c_int);
            }
            memcpy(
                &raw mut longflags as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<uint64_t>() as size_t,
            );
            (*c).flags |= longflags;
            log_debug(
                b"client %p IDENTIFY_LONGFLAGS %#llx\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                longflags as ::core::ffi::c_ulonglong,
            );
        }
        101 => {
            if datalen == 0 as size_t
                || *data.offset(datalen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                return -(1 as ::core::ffi::c_int);
            }
            (*c).term_name = xstrdup(data);
            log_debug(
                b"client %p IDENTIFY_TERM %s\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                data,
            );
        }
        112 => {
            if datalen == 0 as size_t
                || *data.offset(datalen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                return -(1 as ::core::ffi::c_int);
            }
            (*c).term_caps = xreallocarray(
                (*c).term_caps as *mut ::core::ffi::c_void,
                (*c).term_ncaps.wrapping_add(1 as u_int) as size_t,
                ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
            ) as *mut *mut ::core::ffi::c_char;
            let fresh0 = (*c).term_ncaps;
            (*c).term_ncaps = (*c).term_ncaps.wrapping_add(1);
            let ref mut fresh1 = *(*c).term_caps.offset(fresh0 as isize);
            *fresh1 = xstrdup(data);
            log_debug(
                b"client %p IDENTIFY_TERMINFO %s\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                data,
            );
        }
        102 => {
            if datalen == 0 as size_t
                || *data.offset(datalen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                return -(1 as ::core::ffi::c_int);
            }
            (*c).ttyname = xstrdup(data);
            log_debug(
                b"client %p IDENTIFY_TTYNAME %s\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                data,
            );
        }
        108 => {
            if datalen == 0 as size_t
                || *data.offset(datalen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                return -(1 as ::core::ffi::c_int);
            }
            if access(data, X_OK) == 0 as ::core::ffi::c_int {
                (*c).cwd = xstrdup(data);
            } else {
                home = find_home();
                if !home.is_null() {
                    (*c).cwd = xstrdup(home);
                } else {
                    (*c).cwd = xstrdup(b"/\0" as *const u8 as *const ::core::ffi::c_char);
                }
            }
            log_debug(
                b"client %p IDENTIFY_CWD %s\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                data,
            );
        }
        104 => {
            if datalen != 0 as size_t {
                return -(1 as ::core::ffi::c_int);
            }
            (*c).fd = imsg_get_fd(imsg);
            log_debug(
                b"client %p IDENTIFY_STDIN %d\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                (*c).fd,
            );
        }
        110 => {
            if datalen != 0 as size_t {
                return -(1 as ::core::ffi::c_int);
            }
            (*c).out_fd = imsg_get_fd(imsg);
            log_debug(
                b"client %p IDENTIFY_STDOUT %d\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                (*c).out_fd,
            );
        }
        105 => {
            if datalen == 0 as size_t
                || *data.offset(datalen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    != '\0' as i32
            {
                return -(1 as ::core::ffi::c_int);
            }
            if !strchr(data, '=' as i32).is_null() {
                environ_put((*c).environ, data, 0 as ::core::ffi::c_int);
            }
            log_debug(
                b"client %p IDENTIFY_ENVIRON %s\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                data,
            );
        }
        107 => {
            if datalen != ::core::mem::size_of::<pid_t>() as usize {
                return -(1 as ::core::ffi::c_int);
            }
            memcpy(
                &raw mut (*c).pid as *mut ::core::ffi::c_void,
                data as *const ::core::ffi::c_void,
                ::core::mem::size_of::<pid_t>() as size_t,
            );
            log_debug(
                b"client %p IDENTIFY_CLIENTPID %ld\0" as *const u8 as *const ::core::ffi::c_char,
                c,
                (*c).pid as ::core::ffi::c_long,
            );
        }
        _ => {}
    }
    if (*imsg).hdr.type_0 != MSG_IDENTIFY_DONE as ::core::ffi::c_int as uint32_t {
        return 0 as ::core::ffi::c_int;
    }
    (*c).flags |= CLIENT_IDENTIFIED as uint64_t;
    if (*c).term_name.is_null() || *(*c).term_name as ::core::ffi::c_int == '\0' as i32 {
        free((*c).term_name as *mut ::core::ffi::c_void);
        (*c).term_name = xstrdup(b"unknown\0" as *const u8 as *const ::core::ffi::c_char);
    }
    if !(*c).ttyname.is_null() && *(*c).ttyname as ::core::ffi::c_int != '\0' as i32 {
        name = xstrdup((*c).ttyname);
    } else {
        xasprintf(
            &raw mut name,
            b"client-%ld\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).pid as ::core::ffi::c_long,
        );
    }
    (*c).name = name;
    log_debug(
        b"client %p name is %s\0" as *const u8 as *const ::core::ffi::c_char,
        c,
        (*c).name,
    );
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        control_start(c);
    } else if (*c).fd != -(1 as ::core::ffi::c_int) {
        if tty_init(&raw mut (*c).tty, c) != 0 as ::core::ffi::c_int {
            close((*c).fd);
            (*c).fd = -(1 as ::core::ffi::c_int);
        } else {
            tty_resize(&raw mut (*c).tty);
            (*c).flags |= CLIENT_TERMINAL as uint64_t;
        }
        if (*c).out_fd != -(1 as ::core::ffi::c_int) {
            close((*c).out_fd);
        }
        (*c).out_fd = -(1 as ::core::ffi::c_int);
    }
    if (*c).flags & (CLIENT_CONTROL | CLIENT_TERMINAL) as uint64_t != 0 {
        events_fire_client(
            b"client-created\0" as *const u8 as *const ::core::ffi::c_char,
            c,
        );
    }
    if (*c).flags as ::core::ffi::c_ulonglong & (CLIENT_BRACKETPASTING | CLIENT_ASSUMEPASTING) != 0
        && current_time - (*c).paste_time > CLIENT_PASTE_TIME_LIMIT as time_t
    {
        log_debug(
            b"%s: paste time limit exceeded\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
        (*c).flags = ((*c).flags as ::core::ffi::c_ulonglong
            & !(CLIENT_BRACKETPASTING | CLIENT_ASSUMEPASTING)) as uint64_t;
    }
    if !(*c).flags & CLIENT_EXIT as uint64_t != 0 && cfg_finished == 0 && c == clients.tqh_first {
        start_cfg();
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn server_client_dispatch_shell(mut c: *mut client) -> ::core::ffi::c_int {
    let mut shell: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    shell = options_get_string(
        global_s_options,
        b"default-shell\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if checkshell(shell) == 0 {
        shell = _PATH_BSHELL.as_ptr();
    }
    proc_send(
        (*c).peer,
        MSG_SHELL,
        -(1 as ::core::ffi::c_int),
        shell as *const ::core::ffi::c_void,
        strlen(shell).wrapping_add(1 as size_t),
    );
    proc_kill_peer((*c).peer);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_get_cwd(
    mut c: *mut client,
    mut s: *mut session,
) -> *const ::core::ffi::c_char {
    let mut home: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if cfg_finished == 0 && !cfg_client.is_null() {
        return (*cfg_client).cwd;
    }
    if !c.is_null() && (*c).session.is_null() && !(*c).cwd.is_null() {
        return (*c).cwd;
    }
    if !s.is_null() && !(*s).cwd.is_null() {
        return (*s).cwd;
    }
    if !c.is_null()
        && {
            s = (*c).session;
            !s.is_null()
        }
        && !(*s).cwd.is_null()
    {
        return (*s).cwd;
    }
    home = find_home();
    if !home.is_null() {
        return home;
    }
    return b"/\0" as *const u8 as *const ::core::ffi::c_char;
}
unsafe extern "C" fn server_client_control_flags(
    mut c: *mut client,
    mut next: *const ::core::ffi::c_char,
) -> uint64_t {
    if strcmp(
        next,
        b"pause-after\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        (*c).pause_age = 0 as u_int;
        return 0x100000000 as uint64_t;
    }
    if sscanf(
        next,
        b"pause-after=%u\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut (*c).pause_age,
    ) == 1 as ::core::ffi::c_int
    {
        (*c).pause_age = (*c).pause_age.wrapping_mul(1000 as u_int);
        return 0x100000000 as uint64_t;
    }
    if strcmp(
        next,
        b"no-output\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        return 0x4000000 as uint64_t;
    }
    if strcmp(
        next,
        b"wait-exit\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        return 0x200000000 as uint64_t;
    }
    if strcmp(
        next,
        b"new-layouts\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        return 0x800000000 as uint64_t;
    }
    return 0 as uint64_t;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_set_flags(
    mut c: *mut client,
    mut flags: *const ::core::ffi::c_char,
) {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut flag: uint64_t = 0;
    let mut not: ::core::ffi::c_int = 0;
    copy = xstrdup(flags);
    s = copy;
    loop {
        next = strsep(
            &raw mut s,
            b",\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if next.is_null() {
            break;
        }
        not = (*next as ::core::ffi::c_int == '!' as i32) as ::core::ffi::c_int;
        if not != 0 {
            next = next.offset(1);
        }
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            flag = server_client_control_flags(c, next);
        } else {
            flag = 0 as uint64_t;
        }
        if strcmp(
            next,
            b"read-only\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            flag = CLIENT_READONLY as uint64_t;
        } else if strcmp(
            next,
            b"ignore-size\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            flag = CLIENT_IGNORESIZE as uint64_t;
        } else if strcmp(
            next,
            b"no-detach-on-destroy\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            flag = CLIENT_NO_DETACH_ON_DESTROY as uint64_t;
        }
        if flag == 0 as uint64_t {
            continue;
        }
        log_debug(
            b"client %s set flag %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            next,
        );
        if not != 0 {
            if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
                flag &= !CLIENT_READONLY as uint64_t;
            }
            (*c).flags &= !flag;
        } else {
            (*c).flags |= flag;
        }
        if flag == CLIENT_CONTROL_NOOUTPUT as uint64_t {
            control_reset_offsets(c);
        }
    }
    free(copy as *mut ::core::ffi::c_void);
    proc_send(
        (*c).peer,
        MSG_FLAGS,
        -(1 as ::core::ffi::c_int),
        &raw mut (*c).flags as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint64_t>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn server_client_get_flags(mut c: *mut client) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 256] = [0; 256];
    let mut tmp: [::core::ffi::c_char; 32] = [0; 32];
    *(&raw mut s as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
    if (*c).flags & CLIENT_ATTACHED as uint64_t != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"attached,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags & CLIENT_FOCUSED as uint64_t != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"focused,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"control-mode,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags & CLIENT_IGNORESIZE as uint64_t != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"ignore-size,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_NO_DETACH_ON_DESTROY != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"no-detach-on-destroy,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags & CLIENT_CONTROL_NOOUTPUT as uint64_t != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"no-output,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_WAITEXIT != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"wait-exit,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_NEWLAYOUTS != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"new-layouts,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags as ::core::ffi::c_ulonglong & CLIENT_CONTROL_PAUSEAFTER != 0 {
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"pause-after=%u,\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).pause_age.wrapping_div(1000 as u_int),
        );
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags & CLIENT_READONLY as uint64_t != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"read-only,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags & CLIENT_SUSPENDED as uint64_t != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"suspended,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if (*c).flags & CLIENT_UTF8 as uint64_t != 0 {
        strlcat(
            &raw mut s as *mut ::core::ffi::c_char,
            b"UTF-8,\0" as *const u8 as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
        );
    }
    if *(&raw mut s as *mut ::core::ffi::c_char) as ::core::ffi::c_int != '\0' as i32 {
        s[strlen(&raw mut s as *mut ::core::ffi::c_char).wrapping_sub(1 as size_t) as usize] =
            '\0' as i32 as ::core::ffi::c_char;
    }
    return &raw mut s as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn server_client_remove_pane(mut wp: *mut window_pane) {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    c = clients.tqh_first;
    while !c.is_null() {
        if (*c).tty.mouse_last_pane == (*wp).id as ::core::ffi::c_int {
            (*c).tty.mouse_last_pane = -(1 as ::core::ffi::c_int);
            (*c).tty.mouse_drag_update = None;
            (*c).tty.mouse_scrolling_flag = 0 as ::core::ffi::c_int;
        }
        c = (*c).entry.tqe_next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn server_client_print(
    mut c: *mut client,
    mut parse: ::core::ffi::c_int,
    mut evb: *mut evbuffer,
) {
    let mut data: *mut ::core::ffi::c_void =
        evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t) as *mut ::core::ffi::c_void;
    let mut size: size_t = evbuffer_get_length(evb);
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut sanitized: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut msg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut empty: ::core::ffi::c_char = '\0' as i32 as ::core::ffi::c_char;
    if parse == 0 {
        utf8_stravisx(
            &raw mut msg,
            data as *const ::core::ffi::c_char,
            size,
            VIS_OCTAL | VIS_CSTYLE | VIS_NOSLASH,
        );
    } else if size == 0 as size_t {
        msg = &raw mut empty;
    } else {
        msg =
            evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t) as *mut ::core::ffi::c_char;
        if *msg.offset(size.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int != '\0' as i32
        {
            evbuffer_add(
                evb,
                b"\0" as *const u8 as *const ::core::ffi::c_char as *const ::core::ffi::c_void,
                1 as size_t,
            );
        }
    }
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"server_client_print\0" as *const u8 as *const ::core::ffi::c_char,
        msg,
    );
    if !c.is_null() {
        if (*c).session.is_null() || (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            if !(*c).flags & CLIENT_UTF8 as uint64_t != 0 {
                sanitized = utf8_sanitize(msg);
                if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                    control_write(
                        c,
                        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                        sanitized,
                    );
                } else {
                    file_print(
                        c,
                        b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                        sanitized,
                    );
                }
                free(sanitized as *mut ::core::ffi::c_void);
            } else if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
                control_write(c, b"%s\0" as *const u8 as *const ::core::ffi::c_char, msg);
            } else {
                file_print(c, b"%s\n\0" as *const u8 as *const ::core::ffi::c_char, msg);
            }
        } else {
            wp = (*(*(*(*c).session).curw).window).active;
            wme = (*wp).modes.tqh_first;
            if wme.is_null() || (*wme).mode != &raw const window_view_mode {
                window_pane_set_mode(
                    wp,
                    ::core::ptr::null_mut::<window_pane>(),
                    &raw const window_view_mode,
                    ::core::ptr::null_mut::<cmdq_item>(),
                    ::core::ptr::null_mut::<cmd_find_state>(),
                    ::core::ptr::null_mut::<args>(),
                );
            }
            if parse != 0 {
                loop {
                    line = evbuffer_readln(evb, ::core::ptr::null_mut::<size_t>(), EVBUFFER_EOL_LF);
                    if !line.is_null() {
                        window_copy_add(
                            wp,
                            1 as ::core::ffi::c_int,
                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                            line,
                        );
                        free(line as *mut ::core::ffi::c_void);
                    }
                    if line.is_null() {
                        break;
                    }
                }
                size = evbuffer_get_length(evb);
                if size != 0 as size_t {
                    line = evbuffer_pullup(evb, -(1 as ::core::ffi::c_int) as ssize_t)
                        as *mut ::core::ffi::c_char;
                    window_copy_add(
                        wp,
                        1 as ::core::ffi::c_int,
                        b"%.*s\0" as *const u8 as *const ::core::ffi::c_char,
                        size as ::core::ffi::c_int,
                        line,
                    );
                }
            } else {
                window_copy_add(
                    wp,
                    0 as ::core::ffi::c_int,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    msg,
                );
            }
        }
    }
    if parse == 0 {
        free(msg as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn server_client_report_theme(mut c: *mut client, mut theme: client_theme) {
    let mut old: client_theme = (*c).theme;
    if theme as ::core::ffi::c_uint == THEME_LIGHT as ::core::ffi::c_int as ::core::ffi::c_uint {
        (*c).theme = THEME_LIGHT;
        events_fire_client(
            b"client-light-theme\0" as *const u8 as *const ::core::ffi::c_char,
            c,
        );
    } else {
        (*c).theme = THEME_DARK;
        events_fire_client(
            b"client-dark-theme\0" as *const u8 as *const ::core::ffi::c_char,
            c,
        );
    }
    if (*c).theme as ::core::ffi::c_uint != old as ::core::ffi::c_uint {
        server_client_update_theme_colours(c);
        if (*c).tty.flags & TTY_OPENED != 0 {
            tty_invalidate(&raw mut (*c).tty);
        }
        server_redraw_client(c);
    }
    tty_repeat_requests(&raw mut (*c).tty, 1 as ::core::ffi::c_int);
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

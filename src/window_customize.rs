pub use crate::src::shared::limits::{__INT_MAX__, INT_MAX};
pub use crate::src::shared::pane::{
    PANE_REDRAW, window_pane_offset, window_pane_resize, window_pane_resize_entry,
    window_pane_resizes,
};
pub use crate::src::shared::menu::{menu_item};
pub use crate::src::shared::prompt::{
    PROMPT_ACCEPT, PROMPT_CLOSE, PROMPT_CONTINUE, PROMPT_NOFORMAT, PROMPT_SINGLE,
    prompt_free_cb, prompt_result,
};
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::abi::{__int32_t, ssize_t};
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::{screen_write_citem, screen_write_cline};
pub use crate::src::shared::hyperlinks::{hyperlinks};
pub use crate::src::shared::colour::{
    COLOUR_FLAG_THEME, COLOUR_THEME_BLACK, COLOUR_THEME_BLUE, COLOUR_THEME_CYAN,
    COLOUR_THEME_DARK_GREY, COLOUR_THEME_GREEN, COLOUR_THEME_LIGHT_GREY, COLOUR_THEME_MAGENTA,
    COLOUR_THEME_RED, COLOUR_THEME_WHITE, COLOUR_THEME_YELLOW, colour_theme,
};
pub use crate::src::shared::options::{
    OPTIONS_TABLE_IS_ARRAY, OPTIONS_TABLE_IS_COLOUR, OPTIONS_TABLE_IS_HOOK,
    OPTIONS_TABLE_IS_STYLE, OPTIONS_TABLE_PANE, OPTIONS_TABLE_SERVER, OPTIONS_TABLE_SESSION,
    OPTIONS_TABLE_WINDOW,
};
pub use crate::src::shared::sort::{sort_criteria};
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::layout::{layout_geometry};
pub use crate::src::shared::mouse::{mouse_event};
use crate::src::shared::client::*;
use crate::src::shared::layout::*;
use crate::src::shared::prompt::*;
use crate::src::shared::options::*;
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
    pub type mode_tree_data;
    pub type options_array_item;
    pub type options_entry;
    pub type mode_tree_item;
    fn __ctype_tolower_loc() -> *mut *const __int32_t;
    fn __ctype_toupper_loc() -> *mut *const __int32_t;
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
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strcspn(
        __s: *const ::core::ffi::c_char,
        __reject: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_ulong;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strlcat(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_ulong;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn xmalloc(_: size_t) -> *mut ::core::ffi::c_void;
    fn xcalloc(_: size_t, _: size_t) -> *mut ::core::ffi::c_void;
    fn xreallocarray(_: *mut ::core::ffi::c_void, _: size_t, _: size_t)
        -> *mut ::core::ffi::c_void;
    fn xstrdup(_: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn xstrndup(_: *const ::core::ffi::c_char, _: size_t) -> *mut ::core::ffi::c_char;
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
    fn format_true(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn format_free(_: *mut format_tree);
    fn format_add(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn format_pretty_time(_: time_t, _: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn format_expand(
        _: *mut format_tree,
        _: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn format_create_from_state(
        _: *mut cmdq_item,
        _: *mut client,
        _: *mut cmd_find_state,
    ) -> *mut format_tree;
    fn hooks_add_event(_: *const ::core::ffi::c_char);
    fn hooks_is_event(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn hooks_monitor_to_string(_: *mut options_entry) -> *mut ::core::ffi::c_char;
    fn hooks_monitor_get_fire_count(_: *mut options_entry) -> u_int;
    fn hooks_monitor_get_fire_time(_: *mut options_entry) -> time_t;
    fn options_create(_: *mut options) -> *mut options;
    fn options_free(_: *mut options);
    fn options_get_parent(_: *mut options) -> *mut options;
    fn options_first(_: *mut options) -> *mut options_entry;
    fn options_next(_: *mut options_entry) -> *mut options_entry;
    fn options_default(_: *mut options, _: *const options_table_entry) -> *mut options_entry;
    fn options_default_to_string(_: *const options_table_entry) -> *mut ::core::ffi::c_char;
    fn options_name(_: *mut options_entry) -> *const ::core::ffi::c_char;
    fn options_owner(_: *mut options_entry) -> *mut options;
    fn options_get_monitor_data(_: *mut options_entry) -> *mut ::core::ffi::c_void;
    fn options_get_fire_count(_: *mut options_entry) -> u_int;
    fn options_get_fire_time(_: *mut options_entry) -> time_t;
    fn options_table_entry(_: *mut options_entry) -> *const options_table_entry;
    fn options_get_only(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_get(_: *mut options, _: *const ::core::ffi::c_char) -> *mut options_entry;
    fn options_array_get(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
    ) -> *mut options_value;
    fn options_array_getv(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
        ...
    ) -> *mut options_value;
    fn options_array_set(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn options_array_first(_: *mut options_entry) -> *mut options_array_item;
    fn options_array_next(_: *mut options_array_item) -> *mut options_array_item;
    fn options_array_item_key(_: *mut options_array_item) -> *const ::core::ffi::c_char;
    fn options_to_string(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn options_match(
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
        _: *mut ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn options_get_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_longlong;
    fn options_set_string(
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    ) -> *mut options_entry;
    fn options_set_number(
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_longlong,
    ) -> *mut options_entry;
    fn options_from_string(
        _: *mut options,
        _: *const options_table_entry,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn options_push_changes(_: *const ::core::ffi::c_char);
    fn options_remove_or_default(
        _: *mut options_entry,
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    static options_table: [options_table_entry; 0];
    fn environ_first(_: *mut environ) -> *mut environ_entry;
    fn environ_next(_: *mut environ_entry) -> *mut environ_entry;
    fn environ_find(_: *mut environ, _: *const ::core::ffi::c_char) -> *mut environ_entry;
    fn environ_set(
        _: *mut environ,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn environ_clear(_: *mut environ, _: *const ::core::ffi::c_char);
    fn environ_unset(_: *mut environ, _: *const ::core::ffi::c_char);
    fn args_has(_: *mut args, _: u_char) -> ::core::ffi::c_int;
    fn args_get(_: *mut args, _: u_char) -> *const ::core::ffi::c_char;
    fn cmd_find_valid_state(_: *mut cmd_find_state) -> ::core::ffi::c_int;
    fn cmd_find_copy_state(_: *mut cmd_find_state, _: *mut cmd_find_state);
    fn cmd_find_from_pane(
        _: *mut cmd_find_state,
        _: *mut window_pane,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn cmd_list_free(_: *mut cmd_list);
    fn cmd_list_print(_: *const cmd_list, _: ::core::ffi::c_int) -> *mut ::core::ffi::c_char;
    fn cmd_parse_from_string(
        _: *const ::core::ffi::c_char,
        _: *mut cmd_parse_input,
    ) -> *mut cmd_parse_result;
    fn key_bindings_get_table(
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut key_table;
    fn key_bindings_first_table() -> *mut key_table;
    fn key_bindings_next_table(_: *mut key_table) -> *mut key_table;
    fn key_bindings_get(_: *mut key_table, _: key_code) -> *mut key_binding;
    fn key_bindings_get_default(_: *mut key_table, _: key_code) -> *mut key_binding;
    fn key_bindings_first(_: *mut key_table) -> *mut key_binding;
    fn key_bindings_next(_: *mut key_table, _: *mut key_binding) -> *mut key_binding;
    fn key_bindings_add(
        _: *const ::core::ffi::c_char,
        _: key_code,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
        _: *mut cmd_list,
    );
    fn key_bindings_remove(_: *const ::core::ffi::c_char, _: key_code);
    fn key_bindings_reset(_: *const ::core::ffi::c_char, _: key_code);
    fn key_string_lookup_string(_: *const ::core::ffi::c_char) -> key_code;
    fn key_string_lookup_key(_: key_code, _: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn status_message_set(
        _: *mut client,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: *const ::core::ffi::c_char,
        ...
    );
    static grid_default_cell: grid_cell;
    fn screen_write_start(_: *mut screen_write_ctx, _: *mut screen);
    fn screen_write_stop(_: *mut screen_write_ctx);
    fn screen_write_text(
        _: *mut screen_write_ctx,
        _: u_int,
        _: u_int,
        _: u_int,
        _: ::core::ffi::c_int,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn screen_write_nputs(
        _: *mut screen_write_ctx,
        _: ssize_t,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
        ...
    );
    fn screen_write_box(
        _: *mut screen_write_ctx,
        _: u_int,
        _: u_int,
        _: box_lines,
        _: *const grid_cell,
        _: *const ::core::ffi::c_char,
    );
    fn screen_write_clearcharacter(_: *mut screen_write_ctx, _: u_int, _: u_int);
    fn screen_write_cursormove(
        _: *mut screen_write_ctx,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
        _: ::core::ffi::c_int,
    );
    fn window_pane_index(_: *mut window_pane, _: *mut u_int) -> ::core::ffi::c_int;
    fn window_pane_find_by_id(_: u_int) -> *mut window_pane;
    fn window_pane_reset_mode(_: *mut window_pane);
    fn mode_tree_count_tagged(_: *mut mode_tree_data) -> u_int;
    fn mode_tree_get_current(_: *mut mode_tree_data) -> *mut ::core::ffi::c_void;
    fn mode_tree_get_current_name(_: *mut mode_tree_data) -> *const ::core::ffi::c_char;
    fn mode_tree_each_tagged(
        _: *mut mode_tree_data,
        _: mode_tree_each_cb,
        _: *mut client,
        _: key_code,
        _: ::core::ffi::c_int,
    );
    fn mode_tree_up(_: *mut mode_tree_data, _: ::core::ffi::c_int);
    fn mode_tree_start(
        _: *mut window_pane,
        _: *mut args,
        _: mode_tree_build_cb,
        _: mode_tree_draw_cb,
        _: mode_tree_search_cb,
        _: mode_tree_menu_cb,
        _: mode_tree_height_cb,
        _: mode_tree_key_cb,
        _: mode_tree_swap_cb,
        _: mode_tree_sort_cb,
        _: mode_tree_help_cb,
        _: *mut ::core::ffi::c_void,
        _: *const menu_item,
        _: *mut *mut screen,
    ) -> *mut mode_tree_data;
    fn mode_tree_zoom(_: *mut mode_tree_data, _: *mut args);
    fn mode_tree_build(_: *mut mode_tree_data);
    fn mode_tree_free(_: *mut mode_tree_data);
    fn mode_tree_resize(_: *mut mode_tree_data, _: u_int, _: u_int);
    fn mode_tree_add(
        _: *mut mode_tree_data,
        _: *mut mode_tree_item,
        _: *mut ::core::ffi::c_void,
        _: uint64_t,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: ::core::ffi::c_int,
    ) -> *mut mode_tree_item;
    fn mode_tree_draw_as_parent(_: *mut mode_tree_item);
    fn mode_tree_no_tag(_: *mut mode_tree_item);
    fn mode_tree_remove(_: *mut mode_tree_data, _: *mut mode_tree_item);
    fn mode_tree_draw(_: *mut mode_tree_data);
    fn mode_tree_key(
        _: *mut mode_tree_data,
        _: *mut client,
        _: *mut key_code,
        _: *mut mouse_event,
        _: *mut u_int,
        _: *mut u_int,
    ) -> ::core::ffi::c_int;
    fn mode_tree_set_prompt(
        _: *mut mode_tree_data,
        _: *mut client,
        _: *const ::core::ffi::c_char,
        _: *const ::core::ffi::c_char,
        _: prompt_type,
        _: ::core::ffi::c_int,
        _: mode_tree_prompt_input_cb,
        _: prompt_free_cb,
        _: *mut ::core::ffi::c_void,
    );
    fn style_apply(
        _: *mut grid_cell,
        _: *mut options,
        _: *const ::core::ffi::c_char,
        _: *mut format_tree,
    );
    fn spawn_editor(
        _: *mut client,
        _: *const ::core::ffi::c_char,
        _: size_t,
        _: spawn_finish_edit_cb,
        _: *mut ::core::ffi::c_void,
    ) -> *mut spawn_editor_state;
    fn spawn_cancel_editor(_: *mut spawn_editor_state);
    fn spawn_get_editor_pid(_: *mut spawn_editor_state) -> pid_t;
}
pub type uintptr_t = usize;
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
    pub c2rust_unnamed: C2RustUnnamed_35,
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
pub union C2RustUnnamed_35 {
    pub n: u_int,
    pub data: C2RustUnnamed_37,
    pub sel: C2RustUnnamed_36,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_36 {
    pub clip: *const ::core::ffi::c_char,
    pub data: *const ::core::ffi::c_char,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_37 {
    pub data: *const ::core::ffi::c_char,
    pub size: size_t,
}
pub type tty_ctx_set_client_cb =
    Option<unsafe extern "C" fn(*mut tty_ctx, *mut client) -> ::core::ffi::c_int>;
pub type tty_ctx_redraw_cb = Option<unsafe extern "C" fn(*const tty_ctx) -> ()>;
pub type C2RustUnnamed_38 = ::core::ffi::c_ulong;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct environ_entry {
    pub name: *mut ::core::ffi::c_char,
    pub value: *mut ::core::ffi::c_char,
    pub flags: ::core::ffi::c_int,
    pub entry: C2RustUnnamed_39,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_39 {
    pub rbe_left: *mut environ_entry,
    pub rbe_right: *mut environ_entry,
    pub rbe_parent: *mut environ_entry,
    pub rbe_color: ::core::ffi::c_int,
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
pub type mode_tree_prompt_input_cb = Option<
    unsafe extern "C" fn(
        *mut client,
        *mut ::core::ffi::c_void,
        *const ::core::ffi::c_char,
        prompt_key_result,
    ) -> prompt_result,
>;
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
pub struct options_table_entry {
    pub name: *const ::core::ffi::c_char,
    pub alternative_name: *const ::core::ffi::c_char,
    pub type_0: options_table_type,
    pub scope: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
    pub minimum: u_int,
    pub maximum: u_int,
    pub choices: *mut *const ::core::ffi::c_char,
    pub default_str: *const ::core::ffi::c_char,
    pub default_num: ::core::ffi::c_longlong,
    pub default_arr: *mut *const ::core::ffi::c_char,
    pub separator: *const ::core::ffi::c_char,
    pub pattern: *const ::core::ffi::c_char,
    pub text: *const ::core::ffi::c_char,
    pub unit: *const ::core::ffi::c_char,
}
pub type mode_tree_build_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *mut sort_criteria,
        *mut uint64_t,
        *const ::core::ffi::c_char,
    ) -> (),
>;
pub type mode_tree_draw_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *mut ::core::ffi::c_void,
        *mut screen_write_ctx,
        u_int,
        u_int,
    ) -> (),
>;
pub type mode_tree_search_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *mut ::core::ffi::c_void,
        *const ::core::ffi::c_char,
        ::core::ffi::c_int,
    ) -> ::core::ffi::c_int,
>;
pub type mode_tree_menu_cb =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut client, key_code) -> ()>;
pub type mode_tree_height_cb =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, u_int) -> u_int>;
pub type mode_tree_key_cb = Option<
    unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut ::core::ffi::c_void, u_int) -> key_code,
>;
pub type mode_tree_swap_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *mut ::core::ffi::c_void,
        *mut sort_criteria,
    ) -> ::core::ffi::c_int,
>;
pub type mode_tree_sort_cb = Option<unsafe extern "C" fn(*mut sort_criteria) -> ()>;
pub type mode_tree_each_cb = Option<
    unsafe extern "C" fn(
        *mut ::core::ffi::c_void,
        *mut ::core::ffi::c_void,
        *mut client,
        key_code,
    ) -> (),
>;
pub type mode_tree_help_cb = Option<
    unsafe extern "C" fn(
        *mut u_int,
        *mut *const ::core::ffi::c_char,
    ) -> *mut *const ::core::ffi::c_char,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_customize_modedata {
    pub wp: *mut window_pane,
    pub dead: ::core::ffi::c_int,
    pub references: ::core::ffi::c_int,
    pub data: *mut mode_tree_data,
    pub editor: *mut spawn_editor_state,
    pub edit: *mut window_customize_editdata,
    pub format: *mut ::core::ffi::c_char,
    pub hide_global: ::core::ffi::c_int,
    pub hide_default: ::core::ffi::c_int,
    pub prompt_flags: ::core::ffi::c_int,
    pub item_list: *mut *mut window_customize_itemdata,
    pub item_size: u_int,
    pub fs: cmd_find_state,
    pub change: window_customize_change,
}
pub type window_customize_change = ::core::ffi::c_uint;
pub const WINDOW_CUSTOMIZE_RESET: window_customize_change = 1;
pub const WINDOW_CUSTOMIZE_UNSET: window_customize_change = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_customize_itemdata {
    pub data: *mut window_customize_modedata,
    pub type_0: window_customize_item_type,
    pub option_type: window_customize_option_type,
    pub scope: window_customize_scope,
    pub table: *mut ::core::ffi::c_char,
    pub key: key_code,
    pub oo: *mut options,
    pub environ: *mut environ,
    pub environ_flags: ::core::ffi::c_int,
    pub name: *mut ::core::ffi::c_char,
    pub array_key: *mut ::core::ffi::c_char,
}
pub type window_customize_scope = ::core::ffi::c_uint;
pub const WINDOW_CUSTOMIZE_SESSION_ENVIRONMENT: window_customize_scope = 9;
pub const WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT: window_customize_scope = 8;
pub const WINDOW_CUSTOMIZE_PANE: window_customize_scope = 7;
pub const WINDOW_CUSTOMIZE_WINDOW: window_customize_scope = 6;
pub const WINDOW_CUSTOMIZE_GLOBAL_WINDOW: window_customize_scope = 5;
pub const WINDOW_CUSTOMIZE_SESSION: window_customize_scope = 4;
pub const WINDOW_CUSTOMIZE_GLOBAL_SESSION: window_customize_scope = 3;
pub const WINDOW_CUSTOMIZE_SERVER: window_customize_scope = 2;
pub const WINDOW_CUSTOMIZE_KEY: window_customize_scope = 1;
pub const WINDOW_CUSTOMIZE_NONE: window_customize_scope = 0;
pub type window_customize_option_type = ::core::ffi::c_uint;
pub const WINDOW_CUSTOMIZE_HOOKS: window_customize_option_type = 1;
pub const WINDOW_CUSTOMIZE_OPTIONS: window_customize_option_type = 0;
pub type window_customize_item_type = ::core::ffi::c_uint;
pub const WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT: window_customize_item_type = 2;
pub const WINDOW_CUSTOMIZE_ITEM_KEY: window_customize_item_type = 1;
pub const WINDOW_CUSTOMIZE_ITEM_OPTION: window_customize_item_type = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct window_customize_editdata {
    pub wp_id: u_int,
    pub edit_type: window_customize_edit_type,
    pub item: *mut window_customize_itemdata,
    pub editor: *mut spawn_editor_state,
}
pub type window_customize_edit_type = ::core::ffi::c_uint;
pub const WINDOW_CUSTOMIZE_EDIT_ENVIRONMENT: window_customize_edit_type = 3;
pub const WINDOW_CUSTOMIZE_EDIT_KEY_NOTE: window_customize_edit_type = 2;
pub const WINDOW_CUSTOMIZE_EDIT_KEY_COMMAND: window_customize_edit_type = 1;
pub const WINDOW_CUSTOMIZE_EDIT_OPTION: window_customize_edit_type = 0;
pub type spawn_finish_edit_cb =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_char, size_t, *mut ::core::ffi::c_void) -> ()>;
#[inline]
unsafe extern "C" fn tolower(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_tolower_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}
#[inline]
unsafe extern "C" fn toupper(mut __c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    return if __c >= -(128 as ::core::ffi::c_int) && __c < 256 as ::core::ffi::c_int {
        *(*__ctype_toupper_loc()).offset(__c as isize) as ::core::ffi::c_int
    } else {
        __c
    };
}

pub const ENVIRON_HIDDEN: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const KEY_BINDING_REPEAT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const WINDOW_CUSTOMIZE_DEFAULT_FORMAT: [::core::ffi::c_char; 227] = unsafe {
    ::core::mem::transmute::<
        [u8; 227],
        [::core::ffi::c_char; 227],
    >(
        *b"#{?is_option,#{?option_is_global,,#[reverse](#{option_scope})#[default] }#[fg=themelightgrey]#[ignore]#{option_value}#{?option_unit, #{option_unit},},#{?is_environment,#[fg=themelightgrey]#[ignore]#{environment_value},#{key}}}\0",
    )
};
static mut window_customize_menu_items: [menu_item; 12] = [
    menu_item {
        name: b"Select\0" as *const u8 as *const ::core::ffi::c_char,
        key: '\r' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Edit\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'e' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Expand\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_RIGHT as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Tag\0" as *const u8 as *const ::core::ffi::c_char,
        key: 't' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Tag All\0" as *const u8 as *const ::core::ffi::c_char,
        key: '\u{14}' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Tag None\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'T' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Changed Only\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'C' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"\0" as *const u8 as *const ::core::ffi::c_char,
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: b"Cancel\0" as *const u8 as *const ::core::ffi::c_char,
        key: 'q' as i32 as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    menu_item {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        key: KEYC_NONE as ::core::ffi::c_ulong as key_code,
        command: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];
#[no_mangle]
pub static mut window_customize_mode: window_mode = unsafe {
    window_mode {
        name: b"options-mode\0" as *const u8 as *const ::core::ffi::c_char,
        default_format: WINDOW_CUSTOMIZE_DEFAULT_FORMAT.as_ptr(),
        flags: 0,
        init: Some(
            window_customize_init
                as unsafe extern "C" fn(
                    *mut window_mode_entry,
                    *mut cmdq_item,
                    *mut cmd_find_state,
                    *mut args,
                ) -> *mut screen,
        ),
        free: Some(window_customize_free as unsafe extern "C" fn(*mut window_mode_entry) -> ()),
        resize: Some(
            window_customize_resize
                as unsafe extern "C" fn(*mut window_mode_entry, u_int, u_int) -> (),
        ),
        update: Some(window_customize_update as unsafe extern "C" fn(*mut window_mode_entry) -> ()),
        style_changed: None,
        key: Some(
            window_customize_key
                as unsafe extern "C" fn(
                    *mut window_mode_entry,
                    *mut client,
                    *mut session,
                    *mut winlink,
                    key_code,
                    *mut mouse_event,
                ) -> (),
        ),
        key_table: None,
        command: None,
        formats: None,
        get_screen: None,
    }
};
unsafe extern "C" fn window_customize_get_tag(
    mut o: *mut options_entry,
    mut a: *mut options_array_item,
    mut oe: *const options_table_entry,
) -> uint64_t {
    let mut offset: uint64_t = 0;
    if !a.is_null() {
        return a as uintptr_t as uint64_t;
    }
    if oe.is_null() {
        return o as uint64_t;
    }
    offset = ((oe as *mut ::core::ffi::c_char).offset_from(
        &raw const options_table as *const options_table_entry as *mut ::core::ffi::c_char,
    ) as ::core::ffi::c_long as usize)
        .wrapping_div(::core::mem::size_of::<options_table_entry>() as usize)
        as uint64_t;
    return ((2 as ::core::ffi::c_ulonglong) << 62 as ::core::ffi::c_int
        | (offset << 32 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
        | 1 as ::core::ffi::c_ulonglong) as uint64_t;
}
unsafe extern "C" fn window_customize_get_tree(
    mut scope: window_customize_scope,
    mut fs: *mut cmd_find_state,
) -> *mut options {
    match scope as ::core::ffi::c_uint {
        0 | 1 => return ::core::ptr::null_mut::<options>(),
        2 => return global_options,
        3 => return global_s_options,
        4 => return (*(*fs).s).options,
        5 => return global_w_options,
        6 => return (*(*fs).w).options,
        7 => return (*(*fs).wp).options,
        8 | 9 => return ::core::ptr::null_mut::<options>(),
        _ => {}
    }
    return ::core::ptr::null_mut::<options>();
}
unsafe extern "C" fn window_customize_get_environment(
    mut scope: window_customize_scope,
    mut fs: *mut cmd_find_state,
) -> *mut environ {
    match scope as ::core::ffi::c_uint {
        8 => return global_environ,
        9 => return (*(*fs).s).environ,
        _ => return ::core::ptr::null_mut::<environ>(),
    };
}
unsafe extern "C" fn window_customize_check_item(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
    mut fsp: *mut cmd_find_state,
) -> ::core::ffi::c_int {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    if fsp.is_null() {
        fsp = &raw mut fs;
    }
    if cmd_find_valid_state(&raw mut (*data).fs) != 0 {
        cmd_find_copy_state(fsp, &raw mut (*data).fs);
    } else {
        cmd_find_from_pane(fsp, (*data).wp, 0 as ::core::ffi::c_int);
    }
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ((*item).environ == window_customize_get_environment((*item).scope, fsp))
            as ::core::ffi::c_int;
    }
    return ((*item).oo == window_customize_get_tree((*item).scope, fsp)) as ::core::ffi::c_int;
}
unsafe extern "C" fn window_customize_get_key(
    mut item: *mut window_customize_itemdata,
    mut ktp: *mut *mut key_table,
    mut bdp: *mut *mut key_binding,
) -> ::core::ffi::c_int {
    let mut kt: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    kt = key_bindings_get_table((*item).table, 0 as ::core::ffi::c_int);
    if kt.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    bd = key_bindings_get(kt, (*item).key);
    if bd.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    if !ktp.is_null() {
        *ktp = kt;
    }
    if !bdp.is_null() {
        *bdp = bd;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_customize_scope_text(
    mut scope: window_customize_scope,
    mut fs: *mut cmd_find_state,
) -> *mut ::core::ffi::c_char {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut idx: u_int = 0;
    match scope as ::core::ffi::c_uint {
        7 => {
            window_pane_index((*fs).wp, &raw mut idx);
            xasprintf(
                &raw mut s,
                b"pane %u\0" as *const u8 as *const ::core::ffi::c_char,
                idx,
            );
        }
        4 | 9 => {
            xasprintf(
                &raw mut s,
                b"session %s\0" as *const u8 as *const ::core::ffi::c_char,
                (*(*fs).s).name,
            );
        }
        6 => {
            xasprintf(
                &raw mut s,
                b"window %u\0" as *const u8 as *const ::core::ffi::c_char,
                (*(*fs).wl).idx,
            );
        }
        _ => {
            s = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        }
    }
    return s;
}
unsafe extern "C" fn window_customize_write_hook_fire(
    mut ctx: *mut screen_write_ctx,
    mut cx: u_int,
    mut sx: u_int,
    mut sy: u_int,
    mut o: *mut options_entry,
) -> ::core::ffi::c_int {
    let mut fire_time_string: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fire_count: u_int = 0;
    let mut fire_time: time_t = 0;
    if !options_get_monitor_data(o).is_null() {
        fire_count = hooks_monitor_get_fire_count(o);
        fire_time = hooks_monitor_get_fire_time(o);
    } else {
        fire_count = options_get_fire_count(o);
        fire_time = options_get_fire_time(o);
    }
    if fire_time != 0 as time_t {
        fire_time_string = format_pretty_time(fire_time, 0 as ::core::ffi::c_int);
        if screen_write_text(
            ctx,
            cx,
            sx,
            sy,
            0 as ::core::ffi::c_int,
            &raw const grid_default_cell,
            b"This hook has been fired %u times, last %s.\0" as *const u8
                as *const ::core::ffi::c_char,
            fire_count,
            fire_time_string,
        ) == 0
        {
            free(fire_time_string as *mut ::core::ffi::c_void);
            return 0 as ::core::ffi::c_int;
        }
        free(fire_time_string as *mut ::core::ffi::c_void);
        return 1 as ::core::ffi::c_int;
    }
    return screen_write_text(
        ctx,
        cx,
        sx,
        sy,
        0 as ::core::ffi::c_int,
        &raw const grid_default_cell,
        b"This hook has been fired %u times.\0" as *const u8 as *const ::core::ffi::c_char,
        fire_count,
    );
}
unsafe extern "C" fn window_customize_add_item(
    mut data: *mut window_customize_modedata,
) -> *mut window_customize_itemdata {
    let mut item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    (*data).item_list = xreallocarray(
        (*data).item_list as *mut ::core::ffi::c_void,
        (*data).item_size.wrapping_add(1 as u_int) as size_t,
        ::core::mem::size_of::<*mut window_customize_itemdata>() as size_t,
    ) as *mut *mut window_customize_itemdata;
    let fresh0 = (*data).item_size;
    (*data).item_size = (*data).item_size.wrapping_add(1);
    let ref mut fresh1 = *(*data).item_list.offset(fresh0 as isize);
    *fresh1 = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_customize_itemdata>() as size_t,
    ) as *mut window_customize_itemdata;
    item = *fresh1;
    return item;
}
unsafe extern "C" fn window_customize_write_value(
    mut ctx: *mut screen_write_ctx,
    mut cx: u_int,
    mut sx: u_int,
    mut sy: u_int,
    mut more: ::core::ffi::c_int,
    mut label: *const ::core::ffi::c_char,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> ::core::ffi::c_int {
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
    let mut ap: ::core::ffi::VaList;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cy: u_int = (*s).cy;
    let mut retval: ::core::ffi::c_int = 0;
    if sy == 0 as u_int {
        return 0 as ::core::ffi::c_int;
    }
    if screen_write_text(
        ctx,
        cx,
        sx,
        sy,
        1 as ::core::ffi::c_int,
        &raw const grid_default_cell,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        label,
    ) == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    if (*s).cy.wrapping_sub(cy) >= sy {
        return 0 as ::core::ffi::c_int;
    }
    sy = sy.wrapping_sub((*s).cy.wrapping_sub(cy));
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    gc.fg = COLOUR_THEME_LIGHT_GREY as ::core::ffi::c_int | COLOUR_FLAG_THEME;
    ap = args.clone();
    xvasprintf(&raw mut value, fmt, ap);
    retval = screen_write_text(
        ctx,
        cx,
        sx,
        sy,
        more,
        &raw mut gc,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        value,
    );
    free(value as *mut ::core::ffi::c_void);
    return retval;
}
unsafe extern "C" fn window_customize_free_item(mut item: *mut window_customize_itemdata) {
    free((*item).table as *mut ::core::ffi::c_void);
    free((*item).name as *mut ::core::ffi::c_void);
    free((*item).array_key as *mut ::core::ffi::c_void);
    free(item as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_customize_copy_item(
    mut item: *mut window_customize_itemdata,
) -> *mut window_customize_itemdata {
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    new_item = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_customize_itemdata>() as size_t,
    ) as *mut window_customize_itemdata;
    (*new_item).data = (*item).data;
    (*new_item).type_0 = (*item).type_0;
    (*new_item).option_type = (*item).option_type;
    (*new_item).scope = (*item).scope;
    (*new_item).key = (*item).key;
    (*new_item).oo = (*item).oo;
    (*new_item).environ = (*item).environ;
    (*new_item).environ_flags = (*item).environ_flags;
    if !(*item).table.is_null() {
        (*new_item).table = xstrdup((*item).table);
    }
    if !(*item).name.is_null() {
        (*new_item).name = xstrdup((*item).name);
    }
    if !(*item).array_key.is_null() {
        (*new_item).array_key = xstrdup((*item).array_key);
    }
    return new_item;
}
unsafe extern "C" fn window_customize_finish_edit(mut ed: *mut window_customize_editdata) {
    window_customize_free_item((*ed).item);
    free(ed as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_customize_draw_waiting(mut data: *mut window_customize_modedata) {
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
    let mut s: *mut screen = (*(*data).wp).screen;
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
    let mut text: [::core::ffi::c_char; 128] = [0; 128];
    let mut sx: u_int = 0;
    let mut sy: u_int = 0;
    let mut box_w: u_int = 0;
    let mut box_h: u_int = 0;
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    let mut text_x: u_int = 0;
    let mut textlen: size_t = 0;
    let mut pid: pid_t = 0;
    if (*data).editor.is_null() {
        return;
    }
    sx = (*(*s).grid).sx;
    sy = (*(*s).grid).sy;
    if sx == 0 as u_int || sy == 0 as u_int {
        return;
    }
    pid = spawn_get_editor_pid((*data).editor);
    if pid == -(1 as ::core::ffi::c_int) {
        xsnprintf(
            &raw mut text as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
            b"WAITING FOR EDITOR\0" as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        xsnprintf(
            &raw mut text as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 128]>() as size_t,
            b"WAITING FOR EDITOR (PID %ld)\0" as *const u8 as *const ::core::ffi::c_char,
            pid as ::core::ffi::c_long,
        );
    }
    textlen = strlen(&raw mut text as *mut ::core::ffi::c_char);
    box_w = textlen.wrapping_add(4 as size_t) as u_int;
    box_h = 3 as u_int;
    if sx < box_w || sy < box_h {
        return;
    }
    x = sx.wrapping_sub(box_w).wrapping_div(2 as u_int);
    y = sy.wrapping_sub(box_h).wrapping_div(2 as u_int);
    text_x = (x as size_t).wrapping_add(
        (box_w as size_t)
            .wrapping_sub(textlen)
            .wrapping_div(2 as size_t),
    ) as u_int;
    memcpy(
        &raw mut gc as *mut ::core::ffi::c_void,
        &raw const grid_default_cell as *const ::core::ffi::c_void,
        ::core::mem::size_of::<grid_cell>() as size_t,
    );
    screen_write_start(&raw mut ctx, s);
    screen_write_cursormove(
        &raw mut ctx,
        x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_box(
        &raw mut ctx,
        box_w,
        box_h,
        BOX_LINES_DEFAULT,
        &raw mut gc,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
    screen_write_cursormove(
        &raw mut ctx,
        x.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        y.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_clearcharacter(&raw mut ctx, box_w.wrapping_sub(2 as u_int), gc.bg as u_int);
    screen_write_cursormove(
        &raw mut ctx,
        text_x as ::core::ffi::c_int,
        y.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    screen_write_nputs(
        &raw mut ctx,
        box_w.wrapping_sub(2 as u_int) as ssize_t,
        &raw mut gc,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut text as *mut ::core::ffi::c_char,
    );
    screen_write_stop(&raw mut ctx);
}
unsafe extern "C" fn window_customize_set_option_value(
    mut item: *mut window_customize_itemdata,
    mut s: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut oo: *mut options = (*item).oo;
    let mut name: *const ::core::ffi::c_char = (*item).name;
    let mut array_key: *const ::core::ffi::c_char = (*item).array_key;
    let mut idx: u_int = 0;
    let mut keybuf: [::core::ffi::c_char; 32] = [0; 32];
    o = options_get(oo, name);
    if o.is_null() {
        return -(1 as ::core::ffi::c_int);
    }
    oe = options_table_entry(o);
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        if array_key.is_null() {
            idx = 0 as u_int;
            while idx < INT_MAX as u_int {
                if options_array_getv(o, b"%u\0" as *const u8 as *const ::core::ffi::c_char, idx)
                    .is_null()
                {
                    break;
                }
                idx = idx.wrapping_add(1);
            }
            xsnprintf(
                &raw mut keybuf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                idx,
            );
            array_key = &raw mut keybuf as *mut ::core::ffi::c_char;
        }
        if options_array_set(o, array_key, s, 0 as ::core::ffi::c_int, cause)
            != 0 as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int);
        }
    } else if options_from_string(oo, oe, name, s, 0 as ::core::ffi::c_int, cause)
        != 0 as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    if (*item).option_type as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_HOOKS as ::core::ffi::c_int as ::core::ffi::c_uint
        && *name as ::core::ffi::c_int == '@' as i32
    {
        hooks_add_event(name);
    }
    options_push_changes((*item).name);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_customize_option_editable(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) -> ::core::ffi::c_int {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    if (*item).type_0 as ::core::ffi::c_uint
        != WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return 0 as ::core::ffi::c_int;
    }
    o = options_get((*item).oo, (*item).name);
    if o.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    oe = options_table_entry(o);
    if oe.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    if (*oe).type_0 as ::core::ffi::c_uint
        == OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*oe).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 0 as ::core::ffi::c_int;
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_customize_set_command_value(
    mut item: *mut window_customize_itemdata,
    mut s: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    if window_customize_get_key(item, ::core::ptr::null_mut::<*mut key_table>(), &raw mut bd) == 0 {
        return -(1 as ::core::ffi::c_int);
    }
    pr = cmd_parse_from_string(s, ::core::ptr::null_mut::<cmd_parse_input>());
    match (*pr).status as ::core::ffi::c_uint {
        0 => {
            *cause = (*pr).error;
            return -(1 as ::core::ffi::c_int);
        }
        1 | _ => {}
    }
    cmd_list_free((*bd).cmdlist);
    (*bd).cmdlist = (*pr).cmdlist;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_customize_set_note_value(
    mut item: *mut window_customize_itemdata,
    mut s: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    if window_customize_get_key(item, ::core::ptr::null_mut::<*mut key_table>(), &raw mut bd) == 0 {
        return -(1 as ::core::ffi::c_int);
    }
    free((*bd).note as *mut ::core::ffi::c_void);
    if *s as ::core::ffi::c_int == '\0' as i32 {
        (*bd).note = ::core::ptr::null::<::core::ffi::c_char>();
    } else {
        (*bd).note = xstrdup(s);
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_customize_set_environment_value(
    mut item: *mut window_customize_itemdata,
    mut s: *const ::core::ffi::c_char,
) {
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut flags: ::core::ffi::c_int = 0;
    flags = (*item).environ_flags;
    envent = environ_find((*item).environ, (*item).name);
    if !envent.is_null() {
        flags = (*envent).flags;
    }
    environ_set(
        (*item).environ,
        (*item).name,
        flags,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
}
unsafe extern "C" fn window_customize_option_is_changed(
    mut o: *mut options_entry,
    mut array_key: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut oe: *const options_table_entry = options_table_entry(o);
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut defaults: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut default_ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut default_value: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut changed: ::core::ffi::c_int = 0;
    if oe.is_null() || !options_get_monitor_data(o).is_null() {
        return 1 as ::core::ffi::c_int;
    }
    if *options_name(o) as ::core::ffi::c_int == '@' as i32 && hooks_is_event(options_name(o)) != 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        oo = options_create(::core::ptr::null_mut::<options>());
        defaults = options_default(oo, oe);
        if !array_key.is_null() {
            ov = options_array_get(o, array_key);
            default_ov = options_array_get(defaults, array_key);
            if ov.is_null() || default_ov.is_null() {
                changed = (ov != default_ov) as ::core::ffi::c_int;
                options_free(oo);
                return changed;
            }
        }
        value = options_to_string(o, array_key, 0 as ::core::ffi::c_int);
        default_value = options_to_string(defaults, array_key, 0 as ::core::ffi::c_int);
        changed = (strcmp(value, default_value) != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        free(value as *mut ::core::ffi::c_void);
        free(default_value as *mut ::core::ffi::c_void);
        options_free(oo);
        return changed;
    }
    value = options_to_string(
        o,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
    );
    default_value = options_default_to_string(oe);
    changed = (strcmp(value, default_value) != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    free(value as *mut ::core::ffi::c_void);
    free(default_value as *mut ::core::ffi::c_void);
    return changed;
}
unsafe extern "C" fn window_customize_key_is_changed(
    mut kt: *mut key_table,
    mut bd: *mut key_binding,
) -> ::core::ffi::c_int {
    let mut default_bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut default_cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut changed: ::core::ffi::c_int = 0;
    default_bd = key_bindings_get_default(kt, (*bd).key);
    if default_bd.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    if (*bd).flags != (*default_bd).flags {
        return 1 as ::core::ffi::c_int;
    }
    if ((*bd).note == NULL as *const ::core::ffi::c_char) as ::core::ffi::c_int
        != ((*default_bd).note == NULL as *const ::core::ffi::c_char) as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    if !(*bd).note.is_null() && strcmp((*bd).note, (*default_bd).note) != 0 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    cmd = cmd_list_print((*bd).cmdlist, 0 as ::core::ffi::c_int);
    default_cmd = cmd_list_print((*default_bd).cmdlist, 0 as ::core::ffi::c_int);
    changed = (strcmp(cmd, default_cmd) != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    free(cmd as *mut ::core::ffi::c_void);
    free(default_cmd as *mut ::core::ffi::c_void);
    return changed;
}
unsafe extern "C" fn window_customize_build_array(
    mut data: *mut window_customize_modedata,
    mut top: *mut mode_tree_item,
    mut scope: window_customize_scope,
    mut o: *mut options_entry,
    mut ft: *mut format_tree,
) -> u_int {
    let mut oe: *const options_table_entry = options_table_entry(o);
    let mut oo: *mut options = options_owner(o);
    let mut item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut ai: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tag: uint64_t = 0;
    let mut array_key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut count: u_int = 0 as u_int;
    ai = options_array_first(o);
    while !ai.is_null() {
        array_key = options_array_item_key(ai);
        if (*data).hide_default != 0 && window_customize_option_is_changed(o, array_key) == 0 {
            ai = options_array_next(ai);
        } else {
            xasprintf(
                &raw mut name,
                b"%s[%s]\0" as *const u8 as *const ::core::ffi::c_char,
                options_name(o),
                array_key,
            );
            format_add(
                ft,
                b"option_name\0" as *const u8 as *const ::core::ffi::c_char,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                name,
            );
            value = options_to_string(o, array_key, 0 as ::core::ffi::c_int);
            format_add(
                ft,
                b"option_value\0" as *const u8 as *const ::core::ffi::c_char,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                value,
            );
            item = window_customize_add_item(data);
            (*item).type_0 = WINDOW_CUSTOMIZE_ITEM_OPTION;
            if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0 {
                (*item).option_type = WINDOW_CUSTOMIZE_HOOKS;
            }
            (*item).scope = scope;
            (*item).oo = oo;
            (*item).name = xstrdup(options_name(o));
            (*item).array_key = xstrdup(array_key);
            text = format_expand(ft, (*data).format);
            tag = window_customize_get_tag(o, ai, oe);
            mode_tree_add(
                (*data).data,
                top,
                item as *mut ::core::ffi::c_void,
                tag,
                name,
                text,
                -(1 as ::core::ffi::c_int),
            );
            free(text as *mut ::core::ffi::c_void);
            free(name as *mut ::core::ffi::c_void);
            free(value as *mut ::core::ffi::c_void);
            count = count.wrapping_add(1);
            ai = options_array_next(ai);
        }
    }
    return count;
}
unsafe extern "C" fn window_customize_build_option(
    mut data: *mut window_customize_modedata,
    mut top: *mut mode_tree_item,
    mut scope: window_customize_scope,
    mut o: *mut options_entry,
    mut ft: *mut format_tree,
    mut filter: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
    mut type_0: window_customize_option_type,
) -> u_int {
    let mut oe: *const options_table_entry = options_table_entry(o);
    let mut oo: *mut options = options_owner(o);
    let mut name: *const ::core::ffi::c_char = options_name(o);
    let mut item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut global: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut array: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut is_hook: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut is_monitor: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut is_user_hook: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut is_any_hook: ::core::ffi::c_int = 0;
    let mut tag: uint64_t = 0;
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0 {
        is_hook = 1 as ::core::ffi::c_int;
    }
    if !options_get_monitor_data(o).is_null() {
        is_monitor = 1 as ::core::ffi::c_int;
    }
    if *name as ::core::ffi::c_int == '@' as i32 && hooks_is_event(name) != 0 {
        is_user_hook = 1 as ::core::ffi::c_int;
    }
    is_any_hook = (is_hook != 0 || is_monitor != 0 || is_user_hook != 0) as ::core::ffi::c_int;
    match type_0 as ::core::ffi::c_uint {
        0 => {
            if is_any_hook != 0 {
                return 0 as u_int;
            }
        }
        1 => {
            if is_any_hook == 0 {
                return 0 as u_int;
            }
        }
        _ => {}
    }
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        array = 1 as ::core::ffi::c_int;
    }
    if scope as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_SERVER as ::core::ffi::c_int as ::core::ffi::c_uint
        || scope as ::core::ffi::c_uint
            == WINDOW_CUSTOMIZE_GLOBAL_SESSION as ::core::ffi::c_int as ::core::ffi::c_uint
        || scope as ::core::ffi::c_uint
            == WINDOW_CUSTOMIZE_GLOBAL_WINDOW as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        global = 1 as ::core::ffi::c_int;
    }
    if (*data).hide_global != 0 && global != 0 {
        return 0 as u_int;
    }
    if (*data).hide_default != 0
        && window_customize_option_is_changed(o, ::core::ptr::null::<::core::ffi::c_char>()) == 0
    {
        return 0 as u_int;
    }
    format_add(
        ft,
        b"option_name\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    format_add(
        ft,
        b"option_is_global\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        global,
    );
    format_add(
        ft,
        b"option_is_array\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        array,
    );
    format_add(
        ft,
        b"option_is_hook\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        is_hook,
    );
    format_add(
        ft,
        b"option_is_monitor\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        is_monitor,
    );
    text = window_customize_scope_text(scope, fs);
    format_add(
        ft,
        b"option_scope\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        text,
    );
    free(text as *mut ::core::ffi::c_void);
    if !oe.is_null() && !(*oe).unit.is_null() {
        format_add(
            ft,
            b"option_unit\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*oe).unit,
        );
    } else {
        format_add(
            ft,
            b"option_unit\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if is_monitor != 0 {
        value = hooks_monitor_to_string(o);
        if !value.is_null() {
            format_add(
                ft,
                b"option_monitor\0" as *const u8 as *const ::core::ffi::c_char,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                value,
            );
            free(value as *mut ::core::ffi::c_void);
        } else {
            format_add(
                ft,
                b"option_monitor\0" as *const u8 as *const ::core::ffi::c_char,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                b"\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    } else {
        format_add(
            ft,
            b"option_monitor\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            b"\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if array == 0 {
        value = options_to_string(
            o,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
        );
        format_add(
            ft,
            b"option_value\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        free(value as *mut ::core::ffi::c_void);
    }
    if !filter.is_null() {
        expanded = format_expand(ft, filter);
        if format_true(expanded) == 0 {
            free(expanded as *mut ::core::ffi::c_void);
            return 0 as u_int;
        }
        free(expanded as *mut ::core::ffi::c_void);
    }
    item = window_customize_add_item(data);
    (*item).type_0 = WINDOW_CUSTOMIZE_ITEM_OPTION;
    (*item).option_type = type_0;
    (*item).oo = oo;
    (*item).scope = scope;
    (*item).name = xstrdup(name);
    if array != 0 {
        text = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        text = format_expand(ft, (*data).format);
    }
    tag = window_customize_get_tag(o, ::core::ptr::null_mut::<options_array_item>(), oe);
    top = mode_tree_add(
        (*data).data,
        top,
        item as *mut ::core::ffi::c_void,
        tag,
        name,
        text,
        0 as ::core::ffi::c_int,
    ) as *mut mode_tree_item;
    free(text as *mut ::core::ffi::c_void);
    if array == 0 {
        return 1 as u_int;
    }
    return (1 as u_int).wrapping_add(window_customize_build_array(data, top, scope, o, ft));
}
unsafe extern "C" fn window_customize_find_user_options(
    mut oo: *mut options,
    mut list: *mut *mut *const ::core::ffi::c_char,
    mut size: *mut u_int,
) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    o = options_first(oo);
    while !o.is_null() {
        name = options_name(o);
        if *name as ::core::ffi::c_int != '@' as i32 {
            o = options_next(o);
        } else {
            i = 0 as u_int;
            while i < *size {
                if strcmp(*(*list).offset(i as isize), name) == 0 as ::core::ffi::c_int {
                    break;
                }
                i = i.wrapping_add(1);
            }
            if i != *size {
                o = options_next(o);
            } else {
                *list = xreallocarray(
                    *list as *mut ::core::ffi::c_void,
                    (*size).wrapping_add(1 as u_int) as size_t,
                    ::core::mem::size_of::<*const ::core::ffi::c_char>() as size_t,
                ) as *mut *const ::core::ffi::c_char;
                let fresh2 = *size;
                *size = (*size).wrapping_add(1);
                let ref mut fresh3 = *(*list).offset(fresh2 as isize);
                *fresh3 = name;
                o = options_next(o);
            }
        }
    }
}
unsafe extern "C" fn window_customize_build_options(
    mut data: *mut window_customize_modedata,
    mut title: *const ::core::ffi::c_char,
    mut tag: uint64_t,
    mut scope0: window_customize_scope,
    mut oo0: *mut options,
    mut scope1: window_customize_scope,
    mut oo1: *mut options,
    mut scope2: window_customize_scope,
    mut oo2: *mut options,
    mut ft: *mut format_tree,
    mut filter: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
    mut type_0: window_customize_option_type,
) {
    let mut top: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut loop_0: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut list: *mut *const ::core::ffi::c_char =
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut size: u_int = 0 as u_int;
    let mut i: u_int = 0;
    let mut count: u_int = 0 as u_int;
    let mut scope: window_customize_scope = WINDOW_CUSTOMIZE_NONE;
    top = mode_tree_add(
        (*data).data,
        ::core::ptr::null_mut::<mode_tree_item>(),
        NULL,
        tag,
        title,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
    ) as *mut mode_tree_item;
    mode_tree_no_tag(top);
    window_customize_find_user_options(oo0, &raw mut list, &raw mut size);
    if !oo1.is_null() {
        window_customize_find_user_options(oo1, &raw mut list, &raw mut size);
    }
    if !oo2.is_null() {
        window_customize_find_user_options(oo2, &raw mut list, &raw mut size);
    }
    i = 0 as u_int;
    while i < size {
        o = ::core::ptr::null_mut::<options_entry>();
        if !oo2.is_null() {
            o = options_get(oo2, *list.offset(i as isize));
        }
        if o.is_null() && !oo1.is_null() {
            o = options_get(oo1, *list.offset(i as isize));
        }
        if o.is_null() {
            o = options_get(oo0, *list.offset(i as isize));
        }
        if options_owner(o) == oo2 {
            scope = scope2;
        } else if options_owner(o) == oo1 {
            scope = scope1;
        } else {
            scope = scope0;
        }
        count = count.wrapping_add(window_customize_build_option(
            data, top, scope, o, ft, filter, fs, type_0,
        ));
        i = i.wrapping_add(1);
    }
    free(list as *mut ::core::ffi::c_void);
    loop_0 = options_first(oo0);
    while !loop_0.is_null() {
        name = options_name(loop_0);
        if *name as ::core::ffi::c_int == '@' as i32 {
            loop_0 = options_next(loop_0);
        } else {
            if !oo2.is_null() {
                o = options_get(oo2, name);
            } else if !oo1.is_null() {
                o = options_get(oo1, name);
            } else {
                o = loop_0;
            }
            if options_owner(o) == oo2 {
                scope = scope2;
            } else if options_owner(o) == oo1 {
                scope = scope1;
            } else {
                scope = scope0;
            }
            count = count.wrapping_add(window_customize_build_option(
                data, top, scope, o, ft, filter, fs, type_0,
            ));
            loop_0 = options_next(loop_0);
        }
    }
    if (*data).hide_default != 0 && count == 0 as u_int {
        mode_tree_remove((*data).data, top);
    }
}
unsafe extern "C" fn window_customize_key_tag(
    mut ptr: *const ::core::ffi::c_void,
    mut type_0: u_int,
) -> uint64_t {
    return ptr as uintptr_t as uint64_t | type_0 as uint64_t;
}
unsafe extern "C" fn window_customize_build_keys(
    mut data: *mut window_customize_modedata,
    mut kt: *mut key_table,
    mut ft: *mut format_tree,
    mut filter: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
) {
    let mut top: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut child: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut mti: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut title: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tmp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut flag: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut count: u_int = 0 as u_int;
    xasprintf(
        &raw mut title,
        b"Key Table - %s\0" as *const u8 as *const ::core::ffi::c_char,
        (*kt).name,
    );
    top = mode_tree_add(
        (*data).data,
        ::core::ptr::null_mut::<mode_tree_item>(),
        NULL,
        window_customize_key_tag(kt as *const ::core::ffi::c_void, 0 as u_int),
        title,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
    ) as *mut mode_tree_item;
    mode_tree_no_tag(top);
    free(title as *mut ::core::ffi::c_void);
    ft = format_create_from_state(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        fs,
    );
    format_add(
        ft,
        b"is_option\0" as *const u8 as *const ::core::ffi::c_char,
        b"0\0" as *const u8 as *const ::core::ffi::c_char,
    );
    format_add(
        ft,
        b"is_key\0" as *const u8 as *const ::core::ffi::c_char,
        b"1\0" as *const u8 as *const ::core::ffi::c_char,
    );
    format_add(
        ft,
        b"is_environment\0" as *const u8 as *const ::core::ffi::c_char,
        b"0\0" as *const u8 as *const ::core::ffi::c_char,
    );
    bd = key_bindings_first(kt);
    while !bd.is_null() {
        if (*data).hide_default != 0 && window_customize_key_is_changed(kt, bd) == 0 {
            bd = key_bindings_next(kt, bd);
        } else {
            format_add(
                ft,
                b"key\0" as *const u8 as *const ::core::ffi::c_char,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                key_string_lookup_key((*bd).key, 0 as ::core::ffi::c_int),
            );
            if !(*bd).note.is_null() {
                format_add(
                    ft,
                    b"key_note\0" as *const u8 as *const ::core::ffi::c_char,
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*bd).note,
                );
            }
            if !filter.is_null() {
                expanded = format_expand(ft, filter);
                if format_true(expanded) == 0 {
                    free(expanded as *mut ::core::ffi::c_void);
                    bd = key_bindings_next(kt, bd);
                    continue;
                } else {
                    free(expanded as *mut ::core::ffi::c_void);
                }
            }
            item = window_customize_add_item(data);
            (*item).type_0 = WINDOW_CUSTOMIZE_ITEM_KEY;
            (*item).scope = WINDOW_CUSTOMIZE_KEY;
            (*item).table = xstrdup((*kt).name);
            (*item).key = (*bd).key;
            (*item).name = xstrdup(key_string_lookup_key((*item).key, 0 as ::core::ffi::c_int));
            expanded = format_expand(ft, (*data).format);
            child = mode_tree_add(
                (*data).data,
                top,
                item as *mut ::core::ffi::c_void,
                window_customize_key_tag(bd as *const ::core::ffi::c_void, 0 as u_int),
                expanded,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
            ) as *mut mode_tree_item;
            free(expanded as *mut ::core::ffi::c_void);
            tmp = cmd_list_print((*bd).cmdlist, 0 as ::core::ffi::c_int);
            xasprintf(
                &raw mut text,
                b"#[fg=themelightgrey]#[ignore]%s\0" as *const u8 as *const ::core::ffi::c_char,
                tmp,
            );
            free(tmp as *mut ::core::ffi::c_void);
            mti = mode_tree_add(
                (*data).data,
                child,
                item as *mut ::core::ffi::c_void,
                window_customize_key_tag(bd as *const ::core::ffi::c_void, 1 as u_int),
                b"Command\0" as *const u8 as *const ::core::ffi::c_char,
                text,
                -(1 as ::core::ffi::c_int),
            ) as *mut mode_tree_item;
            mode_tree_draw_as_parent(mti);
            mode_tree_no_tag(mti);
            free(text as *mut ::core::ffi::c_void);
            if !(*bd).note.is_null() {
                xasprintf(
                    &raw mut text,
                    b"#[fg=themelightgrey]#[ignore]%s\0" as *const u8 as *const ::core::ffi::c_char,
                    (*bd).note,
                );
            } else {
                text = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
            }
            mti = mode_tree_add(
                (*data).data,
                child,
                item as *mut ::core::ffi::c_void,
                window_customize_key_tag(bd as *const ::core::ffi::c_void, 2 as u_int),
                b"Note\0" as *const u8 as *const ::core::ffi::c_char,
                text,
                -(1 as ::core::ffi::c_int),
            ) as *mut mode_tree_item;
            mode_tree_draw_as_parent(mti);
            mode_tree_no_tag(mti);
            free(text as *mut ::core::ffi::c_void);
            if (*bd).flags & KEY_BINDING_REPEAT != 0 {
                flag = b"on\0" as *const u8 as *const ::core::ffi::c_char;
            } else {
                flag = b"off\0" as *const u8 as *const ::core::ffi::c_char;
            }
            xasprintf(
                &raw mut text,
                b"#[fg=themelightgrey]#[ignore]%s\0" as *const u8 as *const ::core::ffi::c_char,
                flag,
            );
            mti = mode_tree_add(
                (*data).data,
                child,
                item as *mut ::core::ffi::c_void,
                window_customize_key_tag(bd as *const ::core::ffi::c_void, 3 as u_int),
                b"Repeat\0" as *const u8 as *const ::core::ffi::c_char,
                text,
                -(1 as ::core::ffi::c_int),
            ) as *mut mode_tree_item;
            mode_tree_draw_as_parent(mti);
            mode_tree_no_tag(mti);
            free(text as *mut ::core::ffi::c_void);
            count = count.wrapping_add(1);
            bd = key_bindings_next(kt, bd);
        }
    }
    format_free(ft);
    if (*data).hide_default != 0 && count == 0 as u_int {
        mode_tree_remove((*data).data, top);
    }
}
unsafe extern "C" fn window_customize_build_environment(
    mut data: *mut window_customize_modedata,
    mut title: *const ::core::ffi::c_char,
    mut tag: uint64_t,
    mut scope: window_customize_scope,
    mut env: *mut environ,
    mut ft: *mut format_tree,
    mut filter: *const ::core::ffi::c_char,
    mut fs: *mut cmd_find_state,
) {
    let mut top: *mut mode_tree_item = ::core::ptr::null_mut::<mode_tree_item>();
    let mut item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut item_tag: uint64_t = 0;
    let mut global: ::core::ffi::c_int = 0;
    if (*data).hide_default != 0 {
        return;
    }
    top = mode_tree_add(
        (*data).data,
        ::core::ptr::null_mut::<mode_tree_item>(),
        NULL,
        tag,
        title,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ::core::ffi::c_int,
    ) as *mut mode_tree_item;
    mode_tree_no_tag(top);
    global = (scope as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint)
        as ::core::ffi::c_int;
    format_add(
        ft,
        b"is_option\0" as *const u8 as *const ::core::ffi::c_char,
        b"0\0" as *const u8 as *const ::core::ffi::c_char,
    );
    format_add(
        ft,
        b"is_key\0" as *const u8 as *const ::core::ffi::c_char,
        b"0\0" as *const u8 as *const ::core::ffi::c_char,
    );
    format_add(
        ft,
        b"is_environment\0" as *const u8 as *const ::core::ffi::c_char,
        b"1\0" as *const u8 as *const ::core::ffi::c_char,
    );
    format_add(
        ft,
        b"environment_is_global\0" as *const u8 as *const ::core::ffi::c_char,
        b"%d\0" as *const u8 as *const ::core::ffi::c_char,
        global,
    );
    text = window_customize_scope_text(scope, fs);
    format_add(
        ft,
        b"environment_scope\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        text,
    );
    free(text as *mut ::core::ffi::c_void);
    envent = environ_first(env);
    while !envent.is_null() {
        format_add(
            ft,
            b"environment_name\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*envent).name,
        );
        format_add(
            ft,
            b"environment_hidden\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            ((*envent).flags & ENVIRON_HIDDEN != 0) as ::core::ffi::c_int,
        );
        format_add(
            ft,
            b"environment_removed\0" as *const u8 as *const ::core::ffi::c_char,
            b"%d\0" as *const u8 as *const ::core::ffi::c_char,
            ((*envent).value == NULL as *mut ::core::ffi::c_char) as ::core::ffi::c_int,
        );
        if (*envent).value.is_null() {
            value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
        } else {
            value = xstrdup((*envent).value);
        }
        format_add(
            ft,
            b"environment_value\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            value,
        );
        if !filter.is_null() {
            expanded = format_expand(ft, filter);
            if format_true(expanded) == 0 {
                free(expanded as *mut ::core::ffi::c_void);
                free(value as *mut ::core::ffi::c_void);
                envent = environ_next(envent);
                continue;
            } else {
                free(expanded as *mut ::core::ffi::c_void);
            }
        }
        item = window_customize_add_item(data);
        (*item).type_0 = WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT;
        (*item).scope = scope;
        (*item).environ = env;
        (*item).environ_flags = (*envent).flags;
        (*item).name = xstrdup((*envent).name);
        if (*envent).value.is_null() {
            xasprintf(
                &raw mut name,
                b"-%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*envent).name,
            );
            text = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            name = xstrdup((*envent).name);
            text = format_expand(ft, (*data).format);
        }
        item_tag = ((2 as ::core::ffi::c_ulonglong) << 62 as ::core::ffi::c_int
            | envent as uintptr_t as uint64_t as ::core::ffi::c_ulonglong)
            as uint64_t;
        mode_tree_add(
            (*data).data,
            top,
            item as *mut ::core::ffi::c_void,
            item_tag,
            name,
            text,
            0 as ::core::ffi::c_int,
        );
        free(name as *mut ::core::ffi::c_void);
        free(text as *mut ::core::ffi::c_void);
        free(value as *mut ::core::ffi::c_void);
        envent = environ_next(envent);
    }
}
unsafe extern "C" fn window_customize_build(
    mut modedata: *mut ::core::ffi::c_void,
    mut sort_crit: *mut sort_criteria,
    mut tag: *mut uint64_t,
    mut filter: *const ::core::ffi::c_char,
) {
    let mut data: *mut window_customize_modedata = modedata as *mut window_customize_modedata;
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut i: u_int = 0;
    let mut kt: *mut key_table = ::core::ptr::null_mut::<key_table>();
    i = 0 as u_int;
    while i < (*data).item_size {
        window_customize_free_item(*(*data).item_list.offset(i as isize));
        i = i.wrapping_add(1);
    }
    free((*data).item_list as *mut ::core::ffi::c_void);
    (*data).item_list = ::core::ptr::null_mut::<*mut window_customize_itemdata>();
    (*data).item_size = 0 as u_int;
    if cmd_find_valid_state(&raw mut (*data).fs) != 0 {
        cmd_find_copy_state(&raw mut fs, &raw mut (*data).fs);
    } else {
        cmd_find_from_pane(&raw mut fs, (*data).wp, 0 as ::core::ffi::c_int);
    }
    ft = format_create_from_state(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        &raw mut fs,
    );
    format_add(
        ft,
        b"is_option\0" as *const u8 as *const ::core::ffi::c_char,
        b"1\0" as *const u8 as *const ::core::ffi::c_char,
    );
    format_add(
        ft,
        b"is_key\0" as *const u8 as *const ::core::ffi::c_char,
        b"0\0" as *const u8 as *const ::core::ffi::c_char,
    );
    format_add(
        ft,
        b"is_environment\0" as *const u8 as *const ::core::ffi::c_char,
        b"0\0" as *const u8 as *const ::core::ffi::c_char,
    );
    window_customize_build_options(
        data,
        b"Server Options\0" as *const u8 as *const ::core::ffi::c_char,
        ((3 as ::core::ffi::c_ulonglong) << 62 as ::core::ffi::c_int
            | (OPTIONS_TABLE_SERVER << 1 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
            | 1 as ::core::ffi::c_ulonglong) as uint64_t,
        WINDOW_CUSTOMIZE_SERVER,
        global_options,
        WINDOW_CUSTOMIZE_NONE,
        ::core::ptr::null_mut::<options>(),
        WINDOW_CUSTOMIZE_NONE,
        ::core::ptr::null_mut::<options>(),
        ft,
        filter,
        &raw mut fs,
        WINDOW_CUSTOMIZE_OPTIONS,
    );
    window_customize_build_options(
        data,
        b"Session Options\0" as *const u8 as *const ::core::ffi::c_char,
        ((3 as ::core::ffi::c_ulonglong) << 62 as ::core::ffi::c_int
            | (OPTIONS_TABLE_SESSION << 1 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
            | 1 as ::core::ffi::c_ulonglong) as uint64_t,
        WINDOW_CUSTOMIZE_GLOBAL_SESSION,
        global_s_options,
        WINDOW_CUSTOMIZE_SESSION,
        (*fs.s).options,
        WINDOW_CUSTOMIZE_NONE,
        ::core::ptr::null_mut::<options>(),
        ft,
        filter,
        &raw mut fs,
        WINDOW_CUSTOMIZE_OPTIONS,
    );
    window_customize_build_options(
        data,
        b"Window & Pane Options\0" as *const u8 as *const ::core::ffi::c_char,
        ((3 as ::core::ffi::c_ulonglong) << 62 as ::core::ffi::c_int
            | (OPTIONS_TABLE_WINDOW << 1 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
            | 1 as ::core::ffi::c_ulonglong) as uint64_t,
        WINDOW_CUSTOMIZE_GLOBAL_WINDOW,
        global_w_options,
        WINDOW_CUSTOMIZE_WINDOW,
        (*fs.w).options,
        WINDOW_CUSTOMIZE_PANE,
        (*fs.wp).options,
        ft,
        filter,
        &raw mut fs,
        WINDOW_CUSTOMIZE_OPTIONS,
    );
    window_customize_build_options(
        data,
        b"Session Hooks\0" as *const u8 as *const ::core::ffi::c_char,
        ((3 as ::core::ffi::c_ulonglong) << 62 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_ulonglong) << 8 as ::core::ffi::c_int
            | (OPTIONS_TABLE_SESSION << 1 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
            | 1 as ::core::ffi::c_ulonglong) as uint64_t,
        WINDOW_CUSTOMIZE_GLOBAL_SESSION,
        global_s_options,
        WINDOW_CUSTOMIZE_SESSION,
        (*fs.s).options,
        WINDOW_CUSTOMIZE_NONE,
        ::core::ptr::null_mut::<options>(),
        ft,
        filter,
        &raw mut fs,
        WINDOW_CUSTOMIZE_HOOKS,
    );
    window_customize_build_options(
        data,
        b"Window & Pane Hooks\0" as *const u8 as *const ::core::ffi::c_char,
        ((3 as ::core::ffi::c_ulonglong) << 62 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_ulonglong) << 8 as ::core::ffi::c_int
            | (OPTIONS_TABLE_WINDOW << 1 as ::core::ffi::c_int) as ::core::ffi::c_ulonglong
            | 1 as ::core::ffi::c_ulonglong) as uint64_t,
        WINDOW_CUSTOMIZE_GLOBAL_WINDOW,
        global_w_options,
        WINDOW_CUSTOMIZE_WINDOW,
        (*fs.w).options,
        WINDOW_CUSTOMIZE_PANE,
        (*fs.wp).options,
        ft,
        filter,
        &raw mut fs,
        WINDOW_CUSTOMIZE_HOOKS,
    );
    window_customize_build_environment(
        data,
        b"Global Environment\0" as *const u8 as *const ::core::ffi::c_char,
        ((3 as ::core::ffi::c_ulonglong) << 62 as ::core::ffi::c_int
            | (2 as ::core::ffi::c_ulonglong) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_ulonglong) as uint64_t,
        WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT,
        global_environ,
        ft,
        filter,
        &raw mut fs,
    );
    window_customize_build_environment(
        data,
        b"Session Environment\0" as *const u8 as *const ::core::ffi::c_char,
        ((3 as ::core::ffi::c_ulonglong) << 62 as ::core::ffi::c_int
            | (2 as ::core::ffi::c_ulonglong) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_ulonglong) as uint64_t,
        WINDOW_CUSTOMIZE_SESSION_ENVIRONMENT,
        (*fs.s).environ,
        ft,
        filter,
        &raw mut fs,
    );
    format_free(ft);
    ft = format_create_from_state(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        &raw mut fs,
    );
    format_add(
        ft,
        b"is_environment\0" as *const u8 as *const ::core::ffi::c_char,
        b"0\0" as *const u8 as *const ::core::ffi::c_char,
    );
    kt = key_bindings_first_table();
    while !kt.is_null() {
        if !(*kt).key_bindings.rbh_root.is_null() {
            window_customize_build_keys(data, kt, ft, filter, &raw mut fs);
        }
        kt = key_bindings_next_table(kt);
    }
    format_free(ft);
}
unsafe extern "C" fn window_customize_draw_key(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    let mut kt: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut default_bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut note: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut period: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut default_cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if item.is_null() || window_customize_get_key(item, &raw mut kt, &raw mut bd) == 0 {
        return;
    }
    note = (*bd).note;
    if note.is_null() {
        note = b"There is no note for this key.\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if *note as ::core::ffi::c_int != '\0' as i32
        && *note.offset(strlen(note).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            != '.' as i32
    {
        period = b".\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if screen_write_text(
        ctx,
        cx,
        sx,
        sy,
        0 as ::core::ffi::c_int,
        &raw const grid_default_cell,
        b"%s%s\0" as *const u8 as *const ::core::ffi::c_char,
        note,
        period,
    ) == 0
    {
        return;
    }
    screen_write_cursormove(
        ctx,
        cx as ::core::ffi::c_int,
        (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if (*s).cy >= cy.wrapping_add(sy).wrapping_sub(1 as u_int) {
        return;
    }
    if screen_write_text(
        ctx,
        cx,
        sx,
        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
        0 as ::core::ffi::c_int,
        &raw const grid_default_cell,
        b"This key is in the %s table.\0" as *const u8 as *const ::core::ffi::c_char,
        (*kt).name,
    ) == 0
    {
        return;
    }
    if window_customize_write_value(
        ctx,
        cx,
        sx,
        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
        0 as ::core::ffi::c_int,
        b"Repeat: \0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        if (*bd).flags & KEY_BINDING_REPEAT != 0 {
            b"on\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"off\0" as *const u8 as *const ::core::ffi::c_char
        },
    ) == 0
    {
        return;
    }
    screen_write_cursormove(
        ctx,
        cx as ::core::ffi::c_int,
        (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if (*s).cy >= cy.wrapping_add(sy).wrapping_sub(1 as u_int) {
        return;
    }
    cmd = cmd_list_print((*bd).cmdlist, 0 as ::core::ffi::c_int);
    if window_customize_write_value(
        ctx,
        cx,
        sx,
        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
        0 as ::core::ffi::c_int,
        b"Command: \0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        cmd,
    ) == 0
    {
        free(cmd as *mut ::core::ffi::c_void);
        return;
    }
    default_bd = key_bindings_get_default(kt, (*bd).key);
    if !default_bd.is_null() {
        default_cmd = cmd_list_print((*default_bd).cmdlist, 0 as ::core::ffi::c_int);
        if strcmp(cmd, default_cmd) != 0 as ::core::ffi::c_int
            && window_customize_write_value(
                ctx,
                cx,
                sx,
                sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                0 as ::core::ffi::c_int,
                b"The default is: \0" as *const u8 as *const ::core::ffi::c_char,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                default_cmd,
            ) == 0
        {
            free(default_cmd as *mut ::core::ffi::c_void);
            free(cmd as *mut ::core::ffi::c_void);
            return;
        }
        free(default_cmd as *mut ::core::ffi::c_void);
    }
    free(cmd as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_customize_draw_option(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut current_block: u64;
    let mut s: *mut screen = (*ctx).s;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut parent: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut go: *mut options = ::core::ptr::null_mut::<options>();
    let mut wo: *mut options = ::core::ptr::null_mut::<options>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
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
    let mut choice: *mut *const ::core::ffi::c_char =
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
    let mut text: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut array_key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut space: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut unit: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut expanded: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut monitor: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut label: [::core::ffi::c_char; 64] = [0; 64];
    let mut default_value: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut choices: [::core::ffi::c_char; 256] = ::core::mem::transmute::<
        [u8; 256],
        [::core::ffi::c_char; 256],
    >(
        *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    );
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut ft: *mut format_tree = ::core::ptr::null_mut::<format_tree>();
    let mut is_hook: ::core::ffi::c_int = 0;
    let mut is_monitor: ::core::ffi::c_int = 0;
    let mut is_user_hook: ::core::ffi::c_int = 0;
    let mut is_any_hook: ::core::ffi::c_int = 0;
    if window_customize_check_item(data, item, &raw mut fs) == 0 {
        return;
    }
    name = (*item).name;
    array_key = (*item).array_key;
    o = options_get((*item).oo, name);
    if o.is_null() {
        return;
    }
    oe = options_table_entry(o);
    is_hook = (!oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_HOOK != 0) as ::core::ffi::c_int;
    is_monitor = (options_get_monitor_data(o) != NULL) as ::core::ffi::c_int;
    is_user_hook = (*name as ::core::ffi::c_int == '@' as i32 && hooks_is_event(name) != 0)
        as ::core::ffi::c_int;
    is_any_hook = (is_hook != 0 || is_monitor != 0 || is_user_hook != 0) as ::core::ffi::c_int;
    if !oe.is_null() && !(*oe).unit.is_null() {
        space = b" \0" as *const u8 as *const ::core::ffi::c_char;
        unit = (*oe).unit;
    }
    ft = format_create_from_state(
        ::core::ptr::null_mut::<cmdq_item>(),
        ::core::ptr::null_mut::<client>(),
        &raw mut fs,
    );
    if oe.is_null() || (*oe).text.is_null() {
        if is_monitor != 0 {
            text = b"This hook runs when a monitor changes.\0" as *const u8
                as *const ::core::ffi::c_char;
        } else if is_user_hook != 0 {
            text = b"This hook doesn't have a description.\0" as *const u8
                as *const ::core::ffi::c_char;
        } else {
            text = b"This option doesn't have a description.\0" as *const u8
                as *const ::core::ffi::c_char;
        }
    } else {
        text = (*oe).text;
    }
    if !(screen_write_text(
        ctx,
        cx,
        sx,
        sy,
        0 as ::core::ffi::c_int,
        &raw const grid_default_cell,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        text,
    ) == 0)
    {
        screen_write_cursormove(
            ctx,
            cx as ::core::ffi::c_int,
            (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        if !((*s).cy >= cy.wrapping_add(sy).wrapping_sub(1 as u_int)) {
            if is_monitor != 0 {
                if screen_write_text(
                    ctx,
                    cx,
                    sx,
                    sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                    0 as ::core::ffi::c_int,
                    &raw const grid_default_cell,
                    b"This is a monitor hook.\0" as *const u8 as *const ::core::ffi::c_char,
                ) == 0
                {
                    current_block = 4086289836260337793;
                } else {
                    current_block = 14134146928577265803;
                }
            } else {
                if oe.is_null() {
                    text = b"user\0" as *const u8 as *const ::core::ffi::c_char;
                } else if (*oe).scope & (OPTIONS_TABLE_WINDOW | OPTIONS_TABLE_PANE)
                    == OPTIONS_TABLE_WINDOW | OPTIONS_TABLE_PANE
                {
                    text = b"window and pane\0" as *const u8 as *const ::core::ffi::c_char;
                } else if (*oe).scope & OPTIONS_TABLE_WINDOW != 0 {
                    text = b"window\0" as *const u8 as *const ::core::ffi::c_char;
                } else if (*oe).scope & OPTIONS_TABLE_SESSION != 0 {
                    text = b"session\0" as *const u8 as *const ::core::ffi::c_char;
                } else {
                    text = b"server\0" as *const u8 as *const ::core::ffi::c_char;
                }
                if is_user_hook != 0 {
                    if screen_write_text(
                        ctx,
                        cx,
                        sx,
                        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                        0 as ::core::ffi::c_int,
                        &raw const grid_default_cell,
                        b"This is a user hook.\0" as *const u8 as *const ::core::ffi::c_char,
                    ) == 0
                    {
                        current_block = 4086289836260337793;
                    } else {
                        current_block = 14134146928577265803;
                    }
                } else if is_hook != 0 {
                    if screen_write_text(
                        ctx,
                        cx,
                        sx,
                        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                        0 as ::core::ffi::c_int,
                        &raw const grid_default_cell,
                        b"This is a %s hook.\0" as *const u8 as *const ::core::ffi::c_char,
                        text,
                    ) == 0
                    {
                        current_block = 4086289836260337793;
                    } else {
                        current_block = 14134146928577265803;
                    }
                } else if screen_write_text(
                    ctx,
                    cx,
                    sx,
                    sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                    0 as ::core::ffi::c_int,
                    &raw const grid_default_cell,
                    b"This is a %s option.\0" as *const u8 as *const ::core::ffi::c_char,
                    text,
                ) == 0
                {
                    current_block = 4086289836260337793;
                } else {
                    current_block = 14134146928577265803;
                }
            }
            match current_block {
                4086289836260337793 => {}
                _ => {
                    monitor = hooks_monitor_to_string(o);
                    if !monitor.is_null() {
                        if window_customize_write_value(
                            ctx,
                            cx,
                            sx,
                            sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                            0 as ::core::ffi::c_int,
                            b"Monitor: \0" as *const u8 as *const ::core::ffi::c_char,
                            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                            monitor,
                        ) == 0
                        {
                            free(monitor as *mut ::core::ffi::c_void);
                            current_block = 4086289836260337793;
                        } else {
                            free(monitor as *mut ::core::ffi::c_void);
                            current_block = 13460095289871124136;
                        }
                    } else {
                        current_block = 13460095289871124136;
                    }
                    match current_block {
                        4086289836260337793 => {}
                        _ => {
                            if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
                                if is_hook != 0 {
                                    if array_key.is_null() {
                                        if screen_write_text(
                                            ctx,
                                            cx,
                                            sx,
                                            sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                            0 as ::core::ffi::c_int,
                                            &raw const grid_default_cell,
                                            b"This is an array hook.\0" as *const u8
                                                as *const ::core::ffi::c_char,
                                        ) == 0
                                        {
                                            current_block = 4086289836260337793;
                                        } else {
                                            window_customize_write_hook_fire(
                                                ctx,
                                                cx,
                                                sx,
                                                sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                                o,
                                            ) == 0;
                                            current_block = 4086289836260337793;
                                        }
                                    } else {
                                        current_block = 168769493162332264;
                                    }
                                } else if !array_key.is_null() {
                                    if screen_write_text(
                                        ctx,
                                        cx,
                                        sx,
                                        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                        0 as ::core::ffi::c_int,
                                        &raw const grid_default_cell,
                                        b"This is an array option, key %s.\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        array_key,
                                    ) == 0
                                    {
                                        current_block = 4086289836260337793;
                                    } else {
                                        current_block = 168769493162332264;
                                    }
                                } else if screen_write_text(
                                    ctx,
                                    cx,
                                    sx,
                                    sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                    0 as ::core::ffi::c_int,
                                    &raw const grid_default_cell,
                                    b"This is an array option.\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                ) == 0
                                {
                                    current_block = 4086289836260337793;
                                } else {
                                    current_block = 168769493162332264;
                                }
                                match current_block {
                                    4086289836260337793 => {}
                                    _ => {
                                        if array_key.is_null() {
                                            current_block = 4086289836260337793;
                                        } else {
                                            current_block = 8835654301469918283;
                                        }
                                    }
                                }
                            } else {
                                current_block = 8835654301469918283;
                            }
                            match current_block {
                                4086289836260337793 => {}
                                _ => {
                                    screen_write_cursormove(
                                        ctx,
                                        cx as ::core::ffi::c_int,
                                        (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
                                        0 as ::core::ffi::c_int,
                                    );
                                    if !((*s).cy >= cy.wrapping_add(sy).wrapping_sub(1 as u_int)) {
                                        value = options_to_string(
                                            o,
                                            array_key,
                                            0 as ::core::ffi::c_int,
                                        );
                                        if !oe.is_null() && array_key.is_null() {
                                            default_value = options_default_to_string(oe);
                                            if strcmp(default_value, value)
                                                == 0 as ::core::ffi::c_int
                                            {
                                                free(default_value as *mut ::core::ffi::c_void);
                                                default_value =
                                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                            }
                                        }
                                        if is_any_hook != 0 {
                                            if window_customize_write_value(
                                                ctx,
                                                cx,
                                                sx,
                                                sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                                0 as ::core::ffi::c_int,
                                                b"Hook command: \0" as *const u8
                                                    as *const ::core::ffi::c_char,
                                                b"%s%s%s\0" as *const u8
                                                    as *const ::core::ffi::c_char,
                                                value,
                                                space,
                                                unit,
                                            ) == 0
                                            {
                                                current_block = 4086289836260337793;
                                            } else if window_customize_write_hook_fire(
                                                ctx,
                                                cx,
                                                sx,
                                                sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                                o,
                                            ) == 0
                                            {
                                                current_block = 4086289836260337793;
                                            } else {
                                                current_block = 7385833325316299293;
                                            }
                                        } else if window_customize_write_value(
                                            ctx,
                                            cx,
                                            sx,
                                            sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                            0 as ::core::ffi::c_int,
                                            b"Option value: \0" as *const u8
                                                as *const ::core::ffi::c_char,
                                            b"%s%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                                            value,
                                            space,
                                            unit,
                                        ) == 0
                                        {
                                            current_block = 4086289836260337793;
                                        } else {
                                            current_block = 7385833325316299293;
                                        }
                                        match current_block {
                                            4086289836260337793 => {}
                                            _ => {
                                                if oe.is_null()
                                                    || (*oe).type_0 as ::core::ffi::c_uint
                                                        == OPTIONS_TABLE_STRING
                                                            as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                {
                                                    expanded = format_expand(ft, value);
                                                    if strcmp(expanded, value)
                                                        != 0 as ::core::ffi::c_int
                                                    {
                                                        if window_customize_write_value(
                                                            ctx,
                                                            cx,
                                                            sx,
                                                            sy.wrapping_sub(
                                                                (*s).cy.wrapping_sub(cy),
                                                            ),
                                                            0 as ::core::ffi::c_int,
                                                            b"This expands to: \0" as *const u8
                                                                as *const ::core::ffi::c_char,
                                                            b"%s\0" as *const u8
                                                                as *const ::core::ffi::c_char,
                                                            expanded,
                                                        ) == 0
                                                        {
                                                            free(
                                                                expanded
                                                                    as *mut ::core::ffi::c_void,
                                                            );
                                                            current_block = 4086289836260337793;
                                                        } else {
                                                            current_block = 479107131381816815;
                                                        }
                                                    } else {
                                                        current_block = 479107131381816815;
                                                    }
                                                    match current_block {
                                                        4086289836260337793 => {}
                                                        _ => {
                                                            free(
                                                                expanded
                                                                    as *mut ::core::ffi::c_void,
                                                            );
                                                            current_block = 16108440464692313034;
                                                        }
                                                    }
                                                } else {
                                                    current_block = 16108440464692313034;
                                                }
                                                match current_block {
                                                    4086289836260337793 => {}
                                                    _ => {
                                                        if !oe.is_null()
                                                            && (*oe).type_0 as ::core::ffi::c_uint
                                                                == OPTIONS_TABLE_CHOICE
                                                                    as ::core::ffi::c_int
                                                                    as ::core::ffi::c_uint
                                                        {
                                                            choice = (*oe).choices;
                                                            while !(*choice).is_null() {
                                                                strlcat(
                                                                    &raw mut choices
                                                                        as *mut ::core::ffi::c_char,
                                                                    *choice,
                                                                    ::core::mem::size_of::<
                                                                        [::core::ffi::c_char; 256],
                                                                    >(
                                                                    )
                                                                        as size_t,
                                                                );
                                                                strlcat(
                                                                    &raw mut choices as *mut ::core::ffi::c_char,
                                                                    b", \0" as *const u8 as *const ::core::ffi::c_char,
                                                                    ::core::mem::size_of::<[::core::ffi::c_char; 256]>()
                                                                        as size_t,
                                                                );
                                                                choice = choice.offset(1);
                                                            }
                                                            choices[strlen(
                                                                &raw mut choices
                                                                    as *mut ::core::ffi::c_char,
                                                            )
                                                            .wrapping_sub(2 as size_t)
                                                                as usize] =
                                                                '\0' as i32 as ::core::ffi::c_char;
                                                            if window_customize_write_value(
                                                                ctx,
                                                                cx,
                                                                sx,
                                                                sy.wrapping_sub(
                                                                    (*s).cy.wrapping_sub(cy),
                                                                ),
                                                                0 as ::core::ffi::c_int,
                                                                b"Available values are: \0"
                                                                    as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                b"%s\0" as *const u8
                                                                    as *const ::core::ffi::c_char,
                                                                &raw mut choices
                                                                    as *mut ::core::ffi::c_char,
                                                            ) == 0
                                                            {
                                                                current_block = 4086289836260337793;
                                                            } else {
                                                                current_block =
                                                                    17688141731389699982;
                                                            }
                                                        } else {
                                                            current_block = 17688141731389699982;
                                                        }
                                                        match current_block {
                                                            4086289836260337793 => {}
                                                            _ => {
                                                                if !oe.is_null()
                                                                    && (*oe).type_0
                                                                        as ::core::ffi::c_uint
                                                                        == OPTIONS_TABLE_COLOUR
                                                                            as ::core::ffi::c_int
                                                                            as ::core::ffi::c_uint
                                                                {
                                                                    if screen_write_text(
                                                                        ctx,
                                                                        cx,
                                                                        sx,
                                                                        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                                                        1 as ::core::ffi::c_int,
                                                                        &raw const grid_default_cell,
                                                                        b"This is a colour option: \0" as *const u8
                                                                            as *const ::core::ffi::c_char,
                                                                    ) == 0
                                                                    {
                                                                        current_block = 4086289836260337793;
                                                                    } else {
                                                                        memcpy(
                                                                            &raw mut gc as *mut ::core::ffi::c_void,
                                                                            &raw const grid_default_cell as *const ::core::ffi::c_void,
                                                                            ::core::mem::size_of::<grid_cell>() as size_t,
                                                                        );
                                                                        gc.fg = options_get_number((*item).oo, name)
                                                                            as ::core::ffi::c_int;
                                                                        if screen_write_text(
                                                                            ctx,
                                                                            cx,
                                                                            sx,
                                                                            sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                                                            0 as ::core::ffi::c_int,
                                                                            &raw mut gc,
                                                                            b"EXAMPLE\0" as *const u8 as *const ::core::ffi::c_char,
                                                                        ) == 0
                                                                        {
                                                                            current_block = 4086289836260337793;
                                                                        } else {
                                                                            current_block = 10887629115603254199;
                                                                        }
                                                                    }
                                                                } else {
                                                                    current_block =
                                                                        10887629115603254199;
                                                                }
                                                                match current_block {
                                                                    4086289836260337793 => {}
                                                                    _ => {
                                                                        if !oe.is_null()
                                                                            && (*oe).flags & OPTIONS_TABLE_IS_COLOUR != 0
                                                                        {
                                                                            if screen_write_text(
                                                                                ctx,
                                                                                cx,
                                                                                sx,
                                                                                sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                                                                1 as ::core::ffi::c_int,
                                                                                &raw const grid_default_cell,
                                                                                b"This is a colour option: \0" as *const u8
                                                                                    as *const ::core::ffi::c_char,
                                                                            ) == 0
                                                                            {
                                                                                current_block = 4086289836260337793;
                                                                            } else {
                                                                                style_apply(&raw mut gc, (*item).oo, name, ft);
                                                                                if screen_write_text(
                                                                                    ctx,
                                                                                    cx,
                                                                                    sx,
                                                                                    sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                                                                    0 as ::core::ffi::c_int,
                                                                                    &raw mut gc,
                                                                                    b"EXAMPLE\0" as *const u8 as *const ::core::ffi::c_char,
                                                                                ) == 0
                                                                                {
                                                                                    current_block = 4086289836260337793;
                                                                                } else {
                                                                                    current_block = 17995254032144898061;
                                                                                }
                                                                            }
                                                                        } else {
                                                                            current_block = 17995254032144898061;
                                                                        }
                                                                        match current_block {
                                                                            4086289836260337793 => {
                                                                            }
                                                                            _ => {
                                                                                if !oe.is_null()
                                                                                    && (*oe).flags & OPTIONS_TABLE_IS_STYLE != 0
                                                                                {
                                                                                    if screen_write_text(
                                                                                        ctx,
                                                                                        cx,
                                                                                        sx,
                                                                                        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                                                                        1 as ::core::ffi::c_int,
                                                                                        &raw const grid_default_cell,
                                                                                        b"This is a style option: \0" as *const u8
                                                                                            as *const ::core::ffi::c_char,
                                                                                    ) == 0
                                                                                    {
                                                                                        current_block = 4086289836260337793;
                                                                                    } else {
                                                                                        style_apply(&raw mut gc, (*item).oo, name, ft);
                                                                                        if screen_write_text(
                                                                                            ctx,
                                                                                            cx,
                                                                                            sx,
                                                                                            sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                                                                            0 as ::core::ffi::c_int,
                                                                                            &raw mut gc,
                                                                                            b"EXAMPLE\0" as *const u8 as *const ::core::ffi::c_char,
                                                                                        ) == 0
                                                                                        {
                                                                                            current_block = 4086289836260337793;
                                                                                        } else {
                                                                                            current_block = 1934991416718554651;
                                                                                        }
                                                                                    }
                                                                                } else {
                                                                                    current_block = 1934991416718554651;
                                                                                }
                                                                                match current_block {
                                                                                    4086289836260337793 => {}
                                                                                    _ => {
                                                                                        if !default_value.is_null() {
                                                                                            if window_customize_write_value(
                                                                                                ctx,
                                                                                                cx,
                                                                                                sx,
                                                                                                sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                                                                                0 as ::core::ffi::c_int,
                                                                                                b"The default is: \0" as *const u8
                                                                                                    as *const ::core::ffi::c_char,
                                                                                                b"%s%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                                                                                                default_value,
                                                                                                space,
                                                                                                unit,
                                                                                            ) == 0
                                                                                            {
                                                                                                current_block = 4086289836260337793;
                                                                                            } else {
                                                                                                current_block = 15622658527355336244;
                                                                                            }
                                                                                        } else {
                                                                                            current_block = 15622658527355336244;
                                                                                        }
                                                                                        match current_block {
                                                                                            4086289836260337793 => {}
                                                                                            _ => {
                                                                                                screen_write_cursormove(
                                                                                                    ctx,
                                                                                                    cx as ::core::ffi::c_int,
                                                                                                    (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
                                                                                                    0 as ::core::ffi::c_int,
                                                                                                );
                                                                                                if !((*s).cy > cy.wrapping_add(sy).wrapping_sub(1 as u_int))
                                                                                                {
                                                                                                    if !oe.is_null()
                                                                                                        && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0
                                                                                                    {
                                                                                                        wo = ::core::ptr::null_mut::<options>();
                                                                                                        go = ::core::ptr::null_mut::<options>();
                                                                                                    } else {
                                                                                                        match (*item).scope as ::core::ffi::c_uint {
                                                                                                            7 => {
                                                                                                                wo = options_get_parent((*item).oo);
                                                                                                                go = options_get_parent(wo);
                                                                                                            }
                                                                                                            6 | 4 => {
                                                                                                                wo = ::core::ptr::null_mut::<options>();
                                                                                                                go = options_get_parent((*item).oo);
                                                                                                            }
                                                                                                            _ => {
                                                                                                                wo = ::core::ptr::null_mut::<options>();
                                                                                                                go = ::core::ptr::null_mut::<options>();
                                                                                                            }
                                                                                                        }
                                                                                                    }
                                                                                                    if !wo.is_null() && options_owner(o) != wo {
                                                                                                        parent = options_get_only(wo, name);
                                                                                                        if !parent.is_null() {
                                                                                                            value = options_to_string(
                                                                                                                parent,
                                                                                                                ::core::ptr::null::<::core::ffi::c_char>(),
                                                                                                                0 as ::core::ffi::c_int,
                                                                                                            );
                                                                                                            xsnprintf(
                                                                                                                &raw mut label as *mut ::core::ffi::c_char,
                                                                                                                ::core::mem::size_of::<[::core::ffi::c_char; 64]>()
                                                                                                                    as size_t,
                                                                                                                b"Window value (from window %u): \0" as *const u8
                                                                                                                    as *const ::core::ffi::c_char,
                                                                                                                (*fs.wl).idx,
                                                                                                            );
                                                                                                            if window_customize_write_value(
                                                                                                                ctx,
                                                                                                                (*s).cx,
                                                                                                                sx,
                                                                                                                sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                                                                                                0 as ::core::ffi::c_int,
                                                                                                                &raw mut label as *mut ::core::ffi::c_char,
                                                                                                                b"%s%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                                                                                                                value,
                                                                                                                space,
                                                                                                                unit,
                                                                                                            ) == 0
                                                                                                            {
                                                                                                                current_block = 4086289836260337793;
                                                                                                            } else {
                                                                                                                current_block = 9073771928613846474;
                                                                                                            }
                                                                                                        } else {
                                                                                                            current_block = 9073771928613846474;
                                                                                                        }
                                                                                                    } else {
                                                                                                        current_block = 9073771928613846474;
                                                                                                    }
                                                                                                    match current_block {
                                                                                                        4086289836260337793 => {}
                                                                                                        _ => {
                                                                                                            if !go.is_null() && options_owner(o) != go {
                                                                                                                parent = options_get_only(go, name);
                                                                                                                if !parent.is_null() {
                                                                                                                    value = options_to_string(
                                                                                                                        parent,
                                                                                                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                                                                                                        0 as ::core::ffi::c_int,
                                                                                                                    );
                                                                                                                    window_customize_write_value(
                                                                                                                        ctx,
                                                                                                                        (*s).cx,
                                                                                                                        sx,
                                                                                                                        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
                                                                                                                        0 as ::core::ffi::c_int,
                                                                                                                        b"Global value: \0" as *const u8
                                                                                                                            as *const ::core::ffi::c_char,
                                                                                                                        b"%s%s%s\0" as *const u8 as *const ::core::ffi::c_char,
                                                                                                                        value,
                                                                                                                        space,
                                                                                                                        unit,
                                                                                                                    ) == 0;
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
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    free(value as *mut ::core::ffi::c_void);
    free(default_value as *mut ::core::ffi::c_void);
    format_free(ft);
}
unsafe extern "C" fn window_customize_draw_environment(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut s: *mut screen = (*ctx).s;
    let mut cx: u_int = (*s).cx;
    let mut cy: u_int = (*s).cy;
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut parent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut text: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if window_customize_check_item(data, item, &raw mut fs) == 0 {
        return;
    }
    envent = environ_find((*item).environ, (*item).name);
    if envent.is_null() {
        return;
    }
    if (*item).scope as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        text = b"global\0" as *const u8 as *const ::core::ffi::c_char;
    } else {
        text = b"session\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if screen_write_text(
        ctx,
        cx,
        sx,
        sy,
        0 as ::core::ffi::c_int,
        &raw const grid_default_cell,
        b"This is a %s environment variable.\0" as *const u8 as *const ::core::ffi::c_char,
        text,
    ) == 0
    {
        return;
    }
    if (*envent).flags & ENVIRON_HIDDEN != 0 {
        if screen_write_text(
            ctx,
            cx,
            sx,
            sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
            0 as ::core::ffi::c_int,
            &raw const grid_default_cell,
            b"This variable is hidden.\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0
        {
            return;
        }
    }
    screen_write_cursormove(
        ctx,
        cx as ::core::ffi::c_int,
        (*s).cy.wrapping_add(1 as u_int) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if (*s).cy >= cy.wrapping_add(sy).wrapping_sub(1 as u_int) {
        return;
    }
    if (*envent).value.is_null() {
        if screen_write_text(
            ctx,
            cx,
            sx,
            sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
            0 as ::core::ffi::c_int,
            &raw const grid_default_cell,
            b"Variable is removed.\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0
        {
            return;
        }
    } else if window_customize_write_value(
        ctx,
        cx,
        sx,
        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
        0 as ::core::ffi::c_int,
        b"Variable value: \0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        (*envent).value,
    ) == 0
    {
        return;
    }
    if (*item).scope as ::core::ffi::c_uint
        != WINDOW_CUSTOMIZE_SESSION_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    parent = environ_find(global_environ, (*item).name);
    if parent.is_null() {
        return;
    }
    if (*parent).value.is_null() {
        if screen_write_text(
            ctx,
            cx,
            sx,
            sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
            0 as ::core::ffi::c_int,
            &raw const grid_default_cell,
            b"Global variable is removed.\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0
        {
            return;
        }
    } else if window_customize_write_value(
        ctx,
        cx,
        sx,
        sy.wrapping_sub((*s).cy.wrapping_sub(cy)),
        0 as ::core::ffi::c_int,
        b"Global value: \0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        (*parent).value,
    ) == 0
    {
        return;
    }
}
unsafe extern "C" fn window_customize_draw(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut ctx: *mut screen_write_ctx,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_customize_modedata = modedata as *mut window_customize_modedata;
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    if item.is_null() {
        return;
    }
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        window_customize_draw_key(data, item, ctx, sx, sy);
    } else if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        window_customize_draw_environment(data, item, ctx, sx, sy);
    } else {
        window_customize_draw_option(data, item, ctx, sx, sy);
    };
}
unsafe extern "C" fn window_customize_menu(
    mut modedata: *mut ::core::ffi::c_void,
    mut c: *mut client,
    mut key: key_code,
) {
    let mut data: *mut window_customize_modedata = modedata as *mut window_customize_modedata;
    let mut wp: *mut window_pane = (*data).wp;
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    wme = (*wp).modes.tqh_first;
    if wme.is_null() || (*wme).data != modedata {
        return;
    }
    window_customize_key(
        wme,
        c,
        ::core::ptr::null_mut::<session>(),
        ::core::ptr::null_mut::<winlink>(),
        key,
        ::core::ptr::null_mut::<mouse_event>(),
    );
}
unsafe extern "C" fn window_customize_height(
    mut modedata: *mut ::core::ffi::c_void,
    mut height: u_int,
) -> u_int {
    return 12 as u_int;
}
static mut window_customize_help_lines: [*const ::core::ffi::c_char; 13] = [
    b"#[fg=themelightgrey]   Enter, s #[#{E:tree-mode-border-style},acs]x#[default] Set %1 value\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          S #[#{E:tree-mode-border-style},acs]x#[default] Set global %1 value\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          w #[#{E:tree-mode-border-style},acs]x#[default] Set window %1 value\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          d #[#{E:tree-mode-border-style},acs]x#[default] Set to default value\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          D #[#{E:tree-mode-border-style},acs]x#[default] Set tagged %1s to default value\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          u #[#{E:tree-mode-border-style},acs]x#[default] Unset an %1\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          U #[#{E:tree-mode-border-style},acs]x#[default] Unset tagged %1s\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          a #[#{E:tree-mode-border-style},acs]x#[default] Change array key\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          e #[#{E:tree-mode-border-style},acs]x#[default] Open %1 value in editor\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          f #[#{E:tree-mode-border-style},acs]x#[default] Enter a filter\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          C #[#{E:tree-mode-border-style},acs]x#[default] Toggle only changed items\0"
        as *const u8 as *const ::core::ffi::c_char,
    b"#[fg=themelightgrey]          v #[#{E:tree-mode-border-style},acs]x#[default] Toggle information\0"
        as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
unsafe extern "C" fn window_customize_help(
    mut width: *mut u_int,
    mut item: *mut *const ::core::ffi::c_char,
) -> *mut *const ::core::ffi::c_char {
    *width = 52 as u_int;
    *item = b"item\0" as *const u8 as *const ::core::ffi::c_char;
    return &raw mut window_customize_help_lines as *mut *const ::core::ffi::c_char;
}
unsafe extern "C" fn window_customize_init(
    mut wme: *mut window_mode_entry,
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut args: *mut args,
) -> *mut screen {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_customize_modedata =
        ::core::ptr::null_mut::<window_customize_modedata>();
    let mut s: *mut screen = ::core::ptr::null_mut::<screen>();
    data = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_customize_modedata>() as size_t,
    ) as *mut window_customize_modedata;
    (*wme).data = data as *mut ::core::ffi::c_void;
    (*data).wp = wp;
    (*data).references = 1 as ::core::ffi::c_int;
    memcpy(
        &raw mut (*data).fs as *mut ::core::ffi::c_void,
        fs as *const ::core::ffi::c_void,
        ::core::mem::size_of::<cmd_find_state>() as size_t,
    );
    if args.is_null() || args_has(args, 'F' as i32 as u_char) == 0 {
        (*data).format = xstrdup(WINDOW_CUSTOMIZE_DEFAULT_FORMAT.as_ptr());
    } else {
        (*data).format = xstrdup(args_get(args, 'F' as i32 as u_char));
    }
    if args_has(args, 'y' as i32 as u_char) != 0 {
        (*data).prompt_flags = PROMPT_ACCEPT;
    }
    (*data).data = mode_tree_start(
        wp,
        args,
        Some(
            window_customize_build
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut sort_criteria,
                    *mut uint64_t,
                    *const ::core::ffi::c_char,
                ) -> (),
        ),
        Some(
            window_customize_draw
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    *mut screen_write_ctx,
                    u_int,
                    u_int,
                ) -> (),
        ),
        None,
        Some(
            window_customize_menu
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut client, key_code) -> (),
        ),
        Some(
            window_customize_height
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, u_int) -> u_int,
        ),
        None,
        None,
        None,
        Some(
            window_customize_help
                as unsafe extern "C" fn(
                    *mut u_int,
                    *mut *const ::core::ffi::c_char,
                ) -> *mut *const ::core::ffi::c_char,
        ),
        data as *mut ::core::ffi::c_void,
        &raw const window_customize_menu_items as *const menu_item,
        &raw mut s,
    );
    mode_tree_zoom((*data).data, args);
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    return s;
}
unsafe extern "C" fn window_customize_destroy(mut data: *mut window_customize_modedata) {
    let mut i: u_int = 0;
    (*data).references -= 1;
    if (*data).references != 0 as ::core::ffi::c_int {
        return;
    }
    i = 0 as u_int;
    while i < (*data).item_size {
        window_customize_free_item(*(*data).item_list.offset(i as isize));
        i = i.wrapping_add(1);
    }
    free((*data).item_list as *mut ::core::ffi::c_void);
    free((*data).format as *mut ::core::ffi::c_void);
    free(data as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_customize_free(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_customize_modedata = (*wme).data as *mut window_customize_modedata;
    if data.is_null() {
        return;
    }
    (*data).dead = 1 as ::core::ffi::c_int;
    if !(*data).editor.is_null() {
        spawn_cancel_editor((*data).editor);
        window_customize_finish_edit((*data).edit as *mut window_customize_editdata);
    }
    mode_tree_free((*data).data);
    window_customize_destroy(data);
}
unsafe extern "C" fn window_customize_resize(
    mut wme: *mut window_mode_entry,
    mut sx: u_int,
    mut sy: u_int,
) {
    let mut data: *mut window_customize_modedata = (*wme).data as *mut window_customize_modedata;
    mode_tree_resize((*data).data, sx, sy);
}
unsafe extern "C" fn window_customize_update(mut wme: *mut window_mode_entry) {
    let mut data: *mut window_customize_modedata = (*wme).data as *mut window_customize_modedata;
    window_customize_draw_waiting(data);
}
unsafe extern "C" fn window_customize_free_callback(mut modedata: *mut ::core::ffi::c_void) {
    window_customize_destroy(modedata as *mut window_customize_modedata);
}
unsafe extern "C" fn window_customize_free_item_callback(mut itemdata: *mut ::core::ffi::c_void) {
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata = (*item).data as *mut window_customize_modedata;
    window_customize_free_item(item);
    window_customize_destroy(data);
}
unsafe extern "C" fn window_customize_set_option_callback(
    mut c: *mut client,
    mut itemdata: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut current_block: u64;
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata = (*item).data as *mut window_customize_modedata;
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut oo: *mut options = (*item).oo;
    let mut name: *const ::core::ffi::c_char = (*item).name;
    let mut array_key: *const ::core::ffi::c_char = (*item).array_key;
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut idx: u_int = 0;
    let mut keybuf: [::core::ffi::c_char; 32] = [0; 32];
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if item.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return PROMPT_CLOSE;
    }
    o = options_get(oo, name);
    if o.is_null() {
        return PROMPT_CLOSE;
    }
    oe = options_table_entry(o);
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        if array_key.is_null() {
            idx = 0 as u_int;
            while idx < INT_MAX as u_int {
                if options_array_getv(o, b"%u\0" as *const u8 as *const ::core::ffi::c_char, idx)
                    .is_null()
                {
                    break;
                }
                idx = idx.wrapping_add(1);
            }
            xsnprintf(
                &raw mut keybuf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"%u\0" as *const u8 as *const ::core::ffi::c_char,
                idx,
            );
            array_key = &raw mut keybuf as *mut ::core::ffi::c_char;
        }
        if options_array_set(o, array_key, s, 0 as ::core::ffi::c_int, &raw mut cause)
            != 0 as ::core::ffi::c_int
        {
            current_block = 1995505731522653903;
        } else {
            current_block = 4808432441040389987;
        }
    } else if options_from_string(oo, oe, name, s, 0 as ::core::ffi::c_int, &raw mut cause)
        != 0 as ::core::ffi::c_int
    {
        current_block = 1995505731522653903;
    } else {
        current_block = 4808432441040389987;
    }
    match current_block {
        1995505731522653903 => {
            *cause = ({
                let mut __res: ::core::ffi::c_int = 0;
                if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                    if 0 != 0 {
                        let mut __c: ::core::ffi::c_int = *cause as u_char as ::core::ffi::c_int;
                        __res = (if __c < -(128 as ::core::ffi::c_int)
                            || __c > 255 as ::core::ffi::c_int
                        {
                            __c as __int32_t
                        } else {
                            *(*__ctype_toupper_loc()).offset(__c as isize)
                        }) as ::core::ffi::c_int;
                    } else {
                        __res = toupper(*cause as u_char as ::core::ffi::c_int);
                    }
                } else {
                    __res = *(*__ctype_toupper_loc())
                        .offset(*cause as u_char as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int;
                }
                __res
            }) as ::core::ffi::c_char;
            status_message_set(
                c,
                -(1 as ::core::ffi::c_int),
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                cause,
            );
            free(cause as *mut ::core::ffi::c_void);
            return PROMPT_CLOSE;
        }
        _ => {
            if (*item).option_type as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_HOOKS as ::core::ffi::c_int as ::core::ffi::c_uint
                && *name as ::core::ffi::c_int == '@' as i32
            {
                hooks_add_event(name);
            }
            options_push_changes((*item).name);
            mode_tree_build((*data).data);
            mode_tree_draw((*data).data);
            (*(*data).wp).flags |= PANE_REDRAW;
            return PROMPT_CLOSE;
        }
    };
}
unsafe extern "C" fn window_customize_set_environment_callback(
    mut c: *mut client,
    mut itemdata: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata = (*item).data as *mut window_customize_modedata;
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut flags: ::core::ffi::c_int = 0;
    if s.is_null() || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if item.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return PROMPT_CLOSE;
    }
    flags = (*item).environ_flags;
    envent = environ_find((*item).environ, (*item).name);
    if !envent.is_null() {
        flags = (*envent).flags;
    }
    environ_set(
        (*item).environ,
        (*item).name,
        flags,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        s,
    );
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*(*data).wp).flags |= PANE_REDRAW;
    return PROMPT_CLOSE;
}
unsafe extern "C" fn window_customize_set_environment(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
    mut global: ::core::ffi::c_int,
) {
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut env: *mut environ = ::core::ptr::null_mut::<environ>();
    let mut scope: window_customize_scope = WINDOW_CUSTOMIZE_NONE;
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut space: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut prompt: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if item.is_null() || window_customize_check_item(data, item, &raw mut fs) == 0 {
        return;
    }
    envent = environ_find((*item).environ, (*item).name);
    if envent.is_null() {
        return;
    }
    if global != 0 {
        scope = WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT;
        env = global_environ;
    } else {
        scope = (*item).scope;
        env = (*item).environ;
    }
    text = window_customize_scope_text(scope, &raw mut fs);
    if *text as ::core::ffi::c_int != '\0' as i32 {
        space = b", for \0" as *const u8 as *const ::core::ffi::c_char;
    } else if scope as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        space = b", global\0" as *const u8 as *const ::core::ffi::c_char;
    }
    xasprintf(
        &raw mut prompt,
        b"(%s%s%s) \0" as *const u8 as *const ::core::ffi::c_char,
        (*item).name,
        space,
        text,
    );
    free(text as *mut ::core::ffi::c_void);
    new_item = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_customize_itemdata>() as size_t,
    ) as *mut window_customize_itemdata;
    (*new_item).data = data as *mut window_customize_modedata;
    (*new_item).type_0 = WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT;
    (*new_item).scope = scope;
    (*new_item).environ = env;
    (*new_item).environ_flags = (*envent).flags;
    (*new_item).name = xstrdup((*item).name);
    (*data).references += 1;
    mode_tree_set_prompt(
        (*data).data,
        c,
        prompt,
        if (*envent).value.is_null() {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            (*envent).value as *const ::core::ffi::c_char
        },
        PROMPT_TYPE_COMMAND,
        PROMPT_NOFORMAT,
        Some(
            window_customize_set_environment_callback
                as unsafe extern "C" fn(
                    *mut client,
                    *mut ::core::ffi::c_void,
                    *const ::core::ffi::c_char,
                    prompt_key_result,
                ) -> prompt_result,
        ),
        Some(
            window_customize_free_item_callback
                as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
        ),
        new_item as *mut ::core::ffi::c_void,
    );
    free(prompt as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_customize_add_option_callback(
    mut c: *mut client,
    mut itemdata: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata = (*item).data as *mut window_customize_modedata;
    let mut copy: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut array_key: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut what: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut ambiguous: ::core::ffi::c_int = 0;
    let mut namelen: size_t = 0;
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if item.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return PROMPT_CLOSE;
    }
    namelen = strcspn(s, b" \t\0" as *const u8 as *const ::core::ffi::c_char) as size_t;
    if namelen == 0 as size_t || *s.offset(namelen as isize) as ::core::ffi::c_int == '\0' as i32 {
        status_message_set(
            c,
            -(1 as ::core::ffi::c_int),
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            b"User option must be @name value\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return PROMPT_CLOSE;
    }
    value = s.offset(namelen as isize);
    while *value as ::core::ffi::c_int == ' ' as i32 || *value as ::core::ffi::c_int == '\t' as i32
    {
        value = value.offset(1);
    }
    if *value as ::core::ffi::c_int == '\0' as i32 {
        status_message_set(
            c,
            -(1 as ::core::ffi::c_int),
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            b"User option must be @name value\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return PROMPT_CLOSE;
    }
    copy = xstrndup(s, namelen);
    name = options_match(copy, &raw mut array_key, &raw mut ambiguous);
    free(copy as *mut ::core::ffi::c_void);
    if name.is_null() || *name as ::core::ffi::c_int != '@' as i32 || !array_key.is_null() {
        what = if (*item).option_type as ::core::ffi::c_uint
            == WINDOW_CUSTOMIZE_HOOKS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            b"hook\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"option\0" as *const u8 as *const ::core::ffi::c_char
        };
        status_message_set(
            c,
            -(1 as ::core::ffi::c_int),
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            b"User %s name must start with @\0" as *const u8 as *const ::core::ffi::c_char,
            what,
        );
        free(name as *mut ::core::ffi::c_void);
        free(array_key as *mut ::core::ffi::c_void);
        return PROMPT_CLOSE;
    }
    options_set_string(
        (*item).oo,
        name,
        0 as ::core::ffi::c_int,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        value,
    );
    if (*item).option_type as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_HOOKS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        hooks_add_event(name);
    }
    options_push_changes(name);
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*(*data).wp).flags |= PANE_REDRAW;
    free(name as *mut ::core::ffi::c_void);
    free(array_key as *mut ::core::ffi::c_void);
    return PROMPT_CLOSE;
}
unsafe extern "C" fn window_customize_add_option(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    mut scope: window_customize_scope,
    mut oo: *mut options,
    mut type_0: window_customize_option_type,
) {
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut prompt: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut what: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    what = if type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_HOOKS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        b"hook\0" as *const u8 as *const ::core::ffi::c_char
    } else {
        b"option\0" as *const u8 as *const ::core::ffi::c_char
    };
    xasprintf(
        &raw mut prompt,
        b"New user %s: \0" as *const u8 as *const ::core::ffi::c_char,
        what,
    );
    new_item = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_customize_itemdata>() as size_t,
    ) as *mut window_customize_itemdata;
    (*new_item).data = data as *mut window_customize_modedata;
    (*new_item).type_0 = WINDOW_CUSTOMIZE_ITEM_OPTION;
    (*new_item).option_type = type_0;
    (*new_item).scope = scope;
    (*new_item).oo = oo;
    (*data).references += 1;
    mode_tree_set_prompt(
        (*data).data,
        c,
        prompt,
        b"@\0" as *const u8 as *const ::core::ffi::c_char,
        PROMPT_TYPE_COMMAND,
        PROMPT_NOFORMAT,
        Some(
            window_customize_add_option_callback
                as unsafe extern "C" fn(
                    *mut client,
                    *mut ::core::ffi::c_void,
                    *const ::core::ffi::c_char,
                    prompt_key_result,
                ) -> prompt_result,
        ),
        Some(
            window_customize_free_item_callback
                as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
        ),
        new_item as *mut ::core::ffi::c_void,
    );
    free(prompt as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_customize_add_environment_callback(
    mut c: *mut client,
    mut itemdata: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata = (*item).data as *mut window_customize_modedata;
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if item.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return PROMPT_CLOSE;
    }
    if *s as ::core::ffi::c_int == '-' as i32 {
        if *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\0' as i32
            || !strchr(s.offset(1 as ::core::ffi::c_int as isize), '=' as i32).is_null()
        {
            status_message_set(
                c,
                -(1 as ::core::ffi::c_int),
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                b"Bad environment variable: %s\0" as *const u8 as *const ::core::ffi::c_char,
                s,
            );
            return PROMPT_CLOSE;
        }
        environ_clear((*item).environ, s.offset(1 as ::core::ffi::c_int as isize));
    } else {
        value = strchr(s, '=' as i32);
        if value.is_null() || value == s {
            status_message_set(
                c,
                -(1 as ::core::ffi::c_int),
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                b"Environment variable must be NAME=value\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
            return PROMPT_CLOSE;
        }
        name = xstrdup(s);
        *name.offset(strcspn(name, b"=\0" as *const u8 as *const ::core::ffi::c_char) as isize) =
            '\0' as i32 as ::core::ffi::c_char;
        environ_set(
            (*item).environ,
            name,
            0 as ::core::ffi::c_int,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            value.offset(1 as ::core::ffi::c_int as isize),
        );
        free(name as *mut ::core::ffi::c_void);
    }
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*(*data).wp).flags |= PANE_REDRAW;
    return PROMPT_CLOSE;
}
unsafe extern "C" fn window_customize_add_environment(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    mut scope: window_customize_scope,
    mut env: *mut environ,
) {
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    new_item = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_customize_itemdata>() as size_t,
    ) as *mut window_customize_itemdata;
    (*new_item).data = data as *mut window_customize_modedata;
    (*new_item).type_0 = WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT;
    (*new_item).scope = scope;
    (*new_item).environ = env;
    (*data).references += 1;
    mode_tree_set_prompt(
        (*data).data,
        c,
        b"New environment: \0" as *const u8 as *const ::core::ffi::c_char,
        b"\0" as *const u8 as *const ::core::ffi::c_char,
        PROMPT_TYPE_COMMAND,
        PROMPT_NOFORMAT,
        Some(
            window_customize_add_environment_callback
                as unsafe extern "C" fn(
                    *mut client,
                    *mut ::core::ffi::c_void,
                    *const ::core::ffi::c_char,
                    prompt_key_result,
                ) -> prompt_result,
        ),
        Some(
            window_customize_free_item_callback
                as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
        ),
        new_item as *mut ::core::ffi::c_void,
    );
}
unsafe extern "C" fn window_customize_edit_close_cb(
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut arg: *mut ::core::ffi::c_void,
) {
    let mut current_block: u64;
    let mut ed: *mut window_customize_editdata = arg as *mut window_customize_editdata;
    let mut item: *mut window_customize_itemdata = (*ed).item;
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    let mut wme: *mut window_mode_entry = ::core::ptr::null_mut::<window_mode_entry>();
    let mut data: *mut window_customize_modedata =
        ::core::ptr::null_mut::<window_customize_modedata>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    wp = window_pane_find_by_id((*ed).wp_id);
    if !wp.is_null() {
        wme = (*wp).modes.tqh_first;
        if !wme.is_null() && (*wme).mode == &raw const window_customize_mode {
            data = (*wme).data as *mut window_customize_modedata;
            if (*data).editor == (*ed).editor {
                (*data).editor = ::core::ptr::null_mut::<spawn_editor_state>();
                (*data).edit = ::core::ptr::null_mut::<window_customize_editdata>();
            }
        }
    }
    if buf.is_null() || len == 0 as size_t || data.is_null() || (*data).dead != 0 {
        free(buf as *mut ::core::ffi::c_void);
        window_customize_finish_edit(ed);
        return;
    }
    if *buf.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int == '\n' as i32 {
        len = len.wrapping_sub(1);
    }
    value = xmalloc(len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    memcpy(
        value as *mut ::core::ffi::c_void,
        buf as *const ::core::ffi::c_void,
        len,
    );
    *value.offset(len as isize) = '\0' as i32 as ::core::ffi::c_char;
    free(buf as *mut ::core::ffi::c_void);
    match (*ed).edit_type as ::core::ffi::c_uint {
        0 => {
            if window_customize_option_editable(data, item) != 0
                && window_customize_set_option_value(item, value, &raw mut cause)
                    != 0 as ::core::ffi::c_int
            {
                free(cause as *mut ::core::ffi::c_void);
                current_block = 8846462416050848735;
            } else {
                current_block = 1608152415753874203;
            }
        }
        1 => {
            if window_customize_set_command_value(item, value, &raw mut cause)
                != 0 as ::core::ffi::c_int
            {
                free(cause as *mut ::core::ffi::c_void);
                current_block = 8846462416050848735;
            } else {
                current_block = 1608152415753874203;
            }
        }
        2 => {
            if window_customize_set_note_value(item, value) != 0 as ::core::ffi::c_int {
                current_block = 8846462416050848735;
            } else {
                current_block = 1608152415753874203;
            }
        }
        3 => {
            if window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>())
                == 0
            {
                current_block = 8846462416050848735;
            } else {
                window_customize_set_environment_value(item, value);
                current_block = 1608152415753874203;
            }
        }
        _ => {
            current_block = 1608152415753874203;
        }
    }
    match current_block {
        1608152415753874203 => {
            mode_tree_build((*data).data);
            mode_tree_draw((*data).data);
            (*wp).flags |= PANE_REDRAW;
        }
        _ => {}
    }
    free(value as *mut ::core::ffi::c_void);
    window_customize_finish_edit(ed);
}
unsafe extern "C" fn window_customize_start_edit(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
    mut c: *mut client,
) {
    let mut ed: *mut window_customize_editdata =
        ::core::ptr::null_mut::<window_customize_editdata>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut envent: *mut environ_entry = ::core::ptr::null_mut::<environ_entry>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut buf: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut edit_type: window_customize_edit_type = WINDOW_CUSTOMIZE_EDIT_OPTION;
    if !(*data).editor.is_null() || item.is_null() {
        return;
    }
    if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if window_customize_option_editable(data, item) == 0 {
            return;
        }
        o = options_get((*item).oo, (*item).name);
        if o.is_null() {
            return;
        }
        value = options_to_string(o, (*item).array_key, 0 as ::core::ffi::c_int);
        edit_type = WINDOW_CUSTOMIZE_EDIT_OPTION;
    } else if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        name = mode_tree_get_current_name((*data).data);
        if window_customize_get_key(item, ::core::ptr::null_mut::<*mut key_table>(), &raw mut bd)
            == 0
        {
            return;
        }
        if strcmp(
            name,
            b"Command\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            value = cmd_list_print((*bd).cmdlist, 0 as ::core::ffi::c_int);
            edit_type = WINDOW_CUSTOMIZE_EDIT_KEY_COMMAND;
        } else if strcmp(name, b"Note\0" as *const u8 as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            if (*bd).note.is_null() {
                value = xstrdup(b"\0" as *const u8 as *const ::core::ffi::c_char);
            } else {
                value = xstrdup((*bd).note);
            }
            edit_type = WINDOW_CUSTOMIZE_EDIT_KEY_NOTE;
        } else {
            return;
        }
    } else if (*item).type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0 {
            return;
        }
        envent = environ_find((*item).environ, (*item).name);
        if envent.is_null() || (*envent).value.is_null() {
            return;
        }
        value = xstrdup((*envent).value);
        edit_type = WINDOW_CUSTOMIZE_EDIT_ENVIRONMENT;
    } else {
        return;
    }
    ed = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_customize_editdata>() as size_t,
    ) as *mut window_customize_editdata;
    (*ed).wp_id = (*(*data).wp).id;
    (*ed).edit_type = edit_type;
    (*ed).item = window_customize_copy_item(item);
    buf = value;
    len = strlen(value);
    if len == 0 as size_t {
        buf = b"\n\0" as *const u8 as *const ::core::ffi::c_char;
        len = 1 as size_t;
    }
    (*ed).editor = spawn_editor(
        c,
        buf,
        len,
        Some(
            window_customize_edit_close_cb
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_char,
                    size_t,
                    *mut ::core::ffi::c_void,
                ) -> (),
        ),
        ed as *mut ::core::ffi::c_void,
    );
    free(value as *mut ::core::ffi::c_void);
    if (*ed).editor.is_null() {
        window_customize_finish_edit(ed);
    } else {
        (*data).editor = (*ed).editor;
        (*data).edit = ed as *mut window_customize_editdata;
    };
}
unsafe extern "C" fn window_customize_set_option(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
    mut global: ::core::ffi::c_int,
    mut pane: ::core::ffi::c_int,
) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut oe: *const options_table_entry = ::core::ptr::null::<options_table_entry>();
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut flag: ::core::ffi::c_int = 0;
    let mut scope: window_customize_scope = WINDOW_CUSTOMIZE_NONE;
    let mut choice: u_int = 0;
    let mut name: *const ::core::ffi::c_char = (*item).name;
    let mut space: *const ::core::ffi::c_char = b"\0" as *const u8 as *const ::core::ffi::c_char;
    let mut array_key: *const ::core::ffi::c_char = (*item).array_key;
    let mut prompt: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    if item.is_null() || window_customize_check_item(data, item, &raw mut fs) == 0 {
        return;
    }
    o = options_get((*item).oo, name);
    if o.is_null() {
        return;
    }
    oe = options_table_entry(o);
    if !oe.is_null() && !(*oe).scope & OPTIONS_TABLE_PANE != 0 {
        pane = 0 as ::core::ffi::c_int;
    }
    if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
        scope = (*item).scope;
        oo = (*item).oo;
    } else {
        if global != 0 {
            match (*item).scope as ::core::ffi::c_uint {
                0 | 1 | 2 | 3 | 5 | 8 | 9 => {
                    scope = (*item).scope;
                }
                4 => {
                    scope = WINDOW_CUSTOMIZE_GLOBAL_SESSION;
                }
                6 | 7 => {
                    scope = WINDOW_CUSTOMIZE_GLOBAL_WINDOW;
                }
                _ => {}
            }
        } else {
            match (*item).scope as ::core::ffi::c_uint {
                0 | 1 | 2 | 4 => {
                    scope = (*item).scope;
                }
                6 | 7 => {
                    if pane != 0 {
                        scope = WINDOW_CUSTOMIZE_PANE;
                    } else {
                        scope = WINDOW_CUSTOMIZE_WINDOW;
                    }
                }
                3 => {
                    scope = WINDOW_CUSTOMIZE_SESSION;
                }
                5 => {
                    if pane != 0 {
                        scope = WINDOW_CUSTOMIZE_PANE;
                    } else {
                        scope = WINDOW_CUSTOMIZE_WINDOW;
                    }
                }
                8 | 9 => {
                    scope = (*item).scope;
                }
                _ => {}
            }
        }
        if scope as ::core::ffi::c_uint == (*item).scope as ::core::ffi::c_uint {
            oo = (*item).oo;
        } else {
            oo = window_customize_get_tree(scope, &raw mut fs);
        }
    }
    if !oe.is_null()
        && (*oe).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_FLAG as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flag = options_get_number(oo, name) as ::core::ffi::c_int;
        options_set_number(
            oo,
            name,
            (flag == 0) as ::core::ffi::c_int as ::core::ffi::c_longlong,
        );
    } else if !oe.is_null()
        && (*oe).type_0 as ::core::ffi::c_uint
            == OPTIONS_TABLE_CHOICE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        choice = options_get_number(oo, name) as u_int;
        if (*(*oe)
            .choices
            .offset(choice.wrapping_add(1 as u_int) as isize))
        .is_null()
        {
            choice = 0 as u_int;
        } else {
            choice = choice.wrapping_add(1);
        }
        options_set_number(oo, name, choice as ::core::ffi::c_longlong);
    } else {
        text = window_customize_scope_text(scope, &raw mut fs);
        if *text as ::core::ffi::c_int != '\0' as i32 {
            space = b", for \0" as *const u8 as *const ::core::ffi::c_char;
        } else if scope as ::core::ffi::c_uint
            != WINDOW_CUSTOMIZE_SERVER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            space = b", global\0" as *const u8 as *const ::core::ffi::c_char;
        }
        if !oe.is_null() && (*oe).flags & OPTIONS_TABLE_IS_ARRAY != 0 {
            if array_key.is_null() {
                xasprintf(
                    &raw mut prompt,
                    b"(%s[+]%s%s) \0" as *const u8 as *const ::core::ffi::c_char,
                    name,
                    space,
                    text,
                );
            } else {
                xasprintf(
                    &raw mut prompt,
                    b"(%s[%s]%s%s) \0" as *const u8 as *const ::core::ffi::c_char,
                    name,
                    array_key,
                    space,
                    text,
                );
            }
        } else {
            xasprintf(
                &raw mut prompt,
                b"(%s%s%s) \0" as *const u8 as *const ::core::ffi::c_char,
                name,
                space,
                text,
            );
        }
        free(text as *mut ::core::ffi::c_void);
        value = options_to_string(o, array_key, 0 as ::core::ffi::c_int);
        new_item = xcalloc(
            1 as size_t,
            ::core::mem::size_of::<window_customize_itemdata>() as size_t,
        ) as *mut window_customize_itemdata;
        (*new_item).data = data as *mut window_customize_modedata;
        (*new_item).type_0 = WINDOW_CUSTOMIZE_ITEM_OPTION;
        (*new_item).option_type = (*item).option_type;
        (*new_item).scope = scope;
        (*new_item).oo = oo;
        (*new_item).name = xstrdup(name);
        if !array_key.is_null() {
            (*new_item).array_key = xstrdup(array_key);
        }
        (*data).references += 1;
        mode_tree_set_prompt(
            (*data).data,
            c,
            prompt,
            value,
            PROMPT_TYPE_COMMAND,
            PROMPT_NOFORMAT,
            Some(
                window_customize_set_option_callback
                    as unsafe extern "C" fn(
                        *mut client,
                        *mut ::core::ffi::c_void,
                        *const ::core::ffi::c_char,
                        prompt_key_result,
                    ) -> prompt_result,
            ),
            Some(
                window_customize_free_item_callback
                    as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
            ),
            new_item as *mut ::core::ffi::c_void,
        );
        free(prompt as *mut ::core::ffi::c_void);
        free(value as *mut ::core::ffi::c_void);
    };
}
unsafe extern "C" fn window_customize_set_array_key_callback(
    mut c: *mut client,
    mut itemdata: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata =
        ::core::ptr::null_mut::<window_customize_modedata>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut array_key: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cause: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if item.is_null() {
        return PROMPT_CLOSE;
    }
    data = (*item).data as *mut window_customize_modedata;
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    name = (*item).name;
    array_key = (*item).array_key;
    if array_key.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return PROMPT_CLOSE;
    }
    o = options_get((*item).oo, name);
    if o.is_null() {
        return PROMPT_CLOSE;
    }
    if !options_array_get(o, s).is_null() {
        return PROMPT_CLOSE;
    }
    value = options_to_string(o, array_key, 0 as ::core::ffi::c_int);
    if options_array_set(o, s, value, 0 as ::core::ffi::c_int, &raw mut cause)
        != 0 as ::core::ffi::c_int
    {
        free(value as *mut ::core::ffi::c_void);
        *cause = ({
            let mut __res: ::core::ffi::c_int = 0;
            if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                if 0 != 0 {
                    let mut __c: ::core::ffi::c_int = *cause as u_char as ::core::ffi::c_int;
                    __res =
                        (if __c < -(128 as ::core::ffi::c_int) || __c > 255 as ::core::ffi::c_int {
                            __c as __int32_t
                        } else {
                            *(*__ctype_toupper_loc()).offset(__c as isize)
                        }) as ::core::ffi::c_int;
                } else {
                    __res = toupper(*cause as u_char as ::core::ffi::c_int);
                }
            } else {
                __res = *(*__ctype_toupper_loc())
                    .offset(*cause as u_char as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int;
            }
            __res
        }) as ::core::ffi::c_char;
        status_message_set(
            c,
            -(1 as ::core::ffi::c_int),
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            cause,
        );
        free(cause as *mut ::core::ffi::c_void);
        return PROMPT_CLOSE;
    } else {
        free(value as *mut ::core::ffi::c_void);
        options_array_set(
            o,
            array_key,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        );
        options_push_changes((*item).name);
        mode_tree_build((*data).data);
        mode_tree_draw((*data).data);
        (*(*data).wp).flags |= PANE_REDRAW;
        return PROMPT_CLOSE;
    };
}
unsafe extern "C" fn window_customize_set_array_key(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) {
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut prompt: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if item.is_null()
        || (*item).array_key.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return;
    }
    xasprintf(
        &raw mut prompt,
        b"(%s[%s]) \0" as *const u8 as *const ::core::ffi::c_char,
        (*item).name,
        (*item).array_key,
    );
    new_item = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_customize_itemdata>() as size_t,
    ) as *mut window_customize_itemdata;
    (*new_item).data = data as *mut window_customize_modedata;
    (*new_item).type_0 = WINDOW_CUSTOMIZE_ITEM_OPTION;
    (*new_item).option_type = (*item).option_type;
    (*new_item).scope = (*item).scope;
    (*new_item).oo = (*item).oo;
    (*new_item).name = xstrdup((*item).name);
    (*new_item).array_key = xstrdup((*item).array_key);
    (*data).references += 1;
    mode_tree_set_prompt(
        (*data).data,
        c,
        prompt,
        (*item).array_key,
        PROMPT_TYPE_COMMAND,
        PROMPT_NOFORMAT,
        Some(
            window_customize_set_array_key_callback
                as unsafe extern "C" fn(
                    *mut client,
                    *mut ::core::ffi::c_void,
                    *const ::core::ffi::c_char,
                    prompt_key_result,
                ) -> prompt_result,
        ),
        Some(
            window_customize_free_item_callback
                as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
        ),
        new_item as *mut ::core::ffi::c_void,
    );
    free(prompt as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_customize_unset_environment(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) {
    if item.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return;
    }
    if environ_find((*item).environ, (*item).name).is_null() {
        return;
    }
    if item == mode_tree_get_current((*data).data) as *mut window_customize_itemdata {
        mode_tree_up((*data).data, 0 as ::core::ffi::c_int);
    }
    environ_unset((*item).environ, (*item).name);
}
unsafe extern "C" fn window_customize_unset_option(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if item.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return;
    }
    o = options_get((*item).oo, (*item).name);
    if o.is_null() {
        return;
    }
    if !(*item).array_key.is_null()
        && item == mode_tree_get_current((*data).data) as *mut window_customize_itemdata
    {
        mode_tree_up((*data).data, 0 as ::core::ffi::c_int);
    }
    options_remove_or_default(
        o,
        (*item).array_key,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
}
unsafe extern "C" fn window_customize_reset_option(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) {
    let mut oo: *mut options = ::core::ptr::null_mut::<options>();
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    if item.is_null()
        || window_customize_check_item(data, item, ::core::ptr::null_mut::<cmd_find_state>()) == 0
    {
        return;
    }
    if !(*item).array_key.is_null() {
        return;
    }
    oo = (*item).oo;
    while !oo.is_null() {
        o = options_get_only(oo, (*item).name);
        if !o.is_null() {
            options_remove_or_default(
                o,
                ::core::ptr::null::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
        }
        oo = options_get_parent(oo);
    }
}
unsafe extern "C" fn window_customize_set_command_callback(
    mut c: *mut client,
    mut itemdata: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata = (*item).data as *mut window_customize_modedata;
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if item.is_null()
        || window_customize_get_key(item, ::core::ptr::null_mut::<*mut key_table>(), &raw mut bd)
            == 0
    {
        return PROMPT_CLOSE;
    }
    pr = cmd_parse_from_string(s, ::core::ptr::null_mut::<cmd_parse_input>());
    match (*pr).status as ::core::ffi::c_uint {
        0 => {
            error = (*pr).error;
            *error = ({
                let mut __res: ::core::ffi::c_int = 0;
                if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                    if 0 != 0 {
                        let mut __c: ::core::ffi::c_int = *error as u_char as ::core::ffi::c_int;
                        __res = (if __c < -(128 as ::core::ffi::c_int)
                            || __c > 255 as ::core::ffi::c_int
                        {
                            __c as __int32_t
                        } else {
                            *(*__ctype_toupper_loc()).offset(__c as isize)
                        }) as ::core::ffi::c_int;
                    } else {
                        __res = toupper(*error as u_char as ::core::ffi::c_int);
                    }
                } else {
                    __res = *(*__ctype_toupper_loc())
                        .offset(*error as u_char as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int;
                }
                __res
            }) as ::core::ffi::c_char;
            status_message_set(
                c,
                -(1 as ::core::ffi::c_int),
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                error,
            );
            free(error as *mut ::core::ffi::c_void);
            return PROMPT_CLOSE;
        }
        1 | _ => {
            cmd_list_free((*bd).cmdlist);
            (*bd).cmdlist = (*pr).cmdlist;
            mode_tree_build((*data).data);
            mode_tree_draw((*data).data);
            (*(*data).wp).flags |= PANE_REDRAW;
            return PROMPT_CLOSE;
        }
    };
}
unsafe extern "C" fn window_customize_set_note_callback(
    mut c: *mut client,
    mut itemdata: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata = (*item).data as *mut window_customize_modedata;
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if item.is_null()
        || window_customize_get_key(item, ::core::ptr::null_mut::<*mut key_table>(), &raw mut bd)
            == 0
    {
        return PROMPT_CLOSE;
    }
    free((*bd).note as *mut ::core::ffi::c_void);
    (*bd).note = xstrdup(s);
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*(*data).wp).flags |= PANE_REDRAW;
    return PROMPT_CLOSE;
}
unsafe extern "C" fn window_customize_set_key(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) {
    let mut key: key_code = (*item).key;
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut prompt: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut value: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    if item.is_null()
        || window_customize_get_key(item, ::core::ptr::null_mut::<*mut key_table>(), &raw mut bd)
            == 0
    {
        return;
    }
    s = mode_tree_get_current_name((*data).data);
    if strcmp(s, b"Repeat\0" as *const u8 as *const ::core::ffi::c_char) == 0 as ::core::ffi::c_int
    {
        (*bd).flags ^= KEY_BINDING_REPEAT;
    } else if strcmp(s, b"Command\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xasprintf(
            &raw mut prompt,
            b"(%s) \0" as *const u8 as *const ::core::ffi::c_char,
            key_string_lookup_key(key, 0 as ::core::ffi::c_int),
        );
        value = cmd_list_print((*bd).cmdlist, 0 as ::core::ffi::c_int);
        new_item = xcalloc(
            1 as size_t,
            ::core::mem::size_of::<window_customize_itemdata>() as size_t,
        ) as *mut window_customize_itemdata;
        (*new_item).data = data as *mut window_customize_modedata;
        (*new_item).type_0 = WINDOW_CUSTOMIZE_ITEM_KEY;
        (*new_item).scope = (*item).scope;
        (*new_item).table = xstrdup((*item).table);
        (*new_item).key = key;
        (*data).references += 1;
        mode_tree_set_prompt(
            (*data).data,
            c,
            prompt,
            value,
            PROMPT_TYPE_COMMAND,
            PROMPT_NOFORMAT,
            Some(
                window_customize_set_command_callback
                    as unsafe extern "C" fn(
                        *mut client,
                        *mut ::core::ffi::c_void,
                        *const ::core::ffi::c_char,
                        prompt_key_result,
                    ) -> prompt_result,
            ),
            Some(
                window_customize_free_item_callback
                    as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
            ),
            new_item as *mut ::core::ffi::c_void,
        );
        free(prompt as *mut ::core::ffi::c_void);
        free(value as *mut ::core::ffi::c_void);
    } else if strcmp(s, b"Note\0" as *const u8 as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int
    {
        xasprintf(
            &raw mut prompt,
            b"(%s) \0" as *const u8 as *const ::core::ffi::c_char,
            key_string_lookup_key(key, 0 as ::core::ffi::c_int),
        );
        new_item = xcalloc(
            1 as size_t,
            ::core::mem::size_of::<window_customize_itemdata>() as size_t,
        ) as *mut window_customize_itemdata;
        (*new_item).data = data as *mut window_customize_modedata;
        (*new_item).type_0 = WINDOW_CUSTOMIZE_ITEM_KEY;
        (*new_item).scope = (*item).scope;
        (*new_item).table = xstrdup((*item).table);
        (*new_item).key = key;
        (*data).references += 1;
        mode_tree_set_prompt(
            (*data).data,
            c,
            prompt,
            if (*bd).note.is_null() {
                b"\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                (*bd).note
            },
            PROMPT_TYPE_COMMAND,
            PROMPT_NOFORMAT,
            Some(
                window_customize_set_note_callback
                    as unsafe extern "C" fn(
                        *mut client,
                        *mut ::core::ffi::c_void,
                        *const ::core::ffi::c_char,
                        prompt_key_result,
                    ) -> prompt_result,
            ),
            Some(
                window_customize_free_item_callback
                    as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
            ),
            new_item as *mut ::core::ffi::c_void,
        );
        free(prompt as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn window_customize_add_key_callback(
    mut c: *mut client,
    mut itemdata: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key0: prompt_key_result,
) -> prompt_result {
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut data: *mut window_customize_modedata = (*item).data as *mut window_customize_modedata;
    let mut key: key_code = 0;
    let mut pr: *mut cmd_parse_result = ::core::ptr::null_mut::<cmd_parse_result>();
    let mut command: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut keystr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut keylen: size_t = 0;
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    keylen = strcspn(s, b" \t\0" as *const u8 as *const ::core::ffi::c_char) as size_t;
    if keylen == 0 as size_t || *s.offset(keylen as isize) as ::core::ffi::c_int == '\0' as i32 {
        status_message_set(
            c,
            -(1 as ::core::ffi::c_int),
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            b"Key binding must be key command\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return PROMPT_CLOSE;
    }
    command = s.offset(keylen as isize);
    while *command as ::core::ffi::c_int == ' ' as i32
        || *command as ::core::ffi::c_int == '\t' as i32
    {
        command = command.offset(1);
    }
    if *command as ::core::ffi::c_int == '\0' as i32 {
        status_message_set(
            c,
            -(1 as ::core::ffi::c_int),
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            b"Key binding must be key command\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return PROMPT_CLOSE;
    }
    keystr = xstrndup(s, keylen);
    key = key_string_lookup_string(keystr);
    if key == KEYC_NONE as ::core::ffi::c_ulong as key_code
        || key == KEYC_UNKNOWN as ::core::ffi::c_ulong as key_code
    {
        status_message_set(
            c,
            -(1 as ::core::ffi::c_int),
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            b"Unknown key: %s\0" as *const u8 as *const ::core::ffi::c_char,
            keystr,
        );
        free(keystr as *mut ::core::ffi::c_void);
        return PROMPT_CLOSE;
    }
    free(keystr as *mut ::core::ffi::c_void);
    pr = cmd_parse_from_string(command, ::core::ptr::null_mut::<cmd_parse_input>());
    match (*pr).status as ::core::ffi::c_uint {
        0 => {
            error = (*pr).error;
            *error = ({
                let mut __res: ::core::ffi::c_int = 0;
                if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
                    if 0 != 0 {
                        let mut __c: ::core::ffi::c_int = *error as u_char as ::core::ffi::c_int;
                        __res = (if __c < -(128 as ::core::ffi::c_int)
                            || __c > 255 as ::core::ffi::c_int
                        {
                            __c as __int32_t
                        } else {
                            *(*__ctype_toupper_loc()).offset(__c as isize)
                        }) as ::core::ffi::c_int;
                    } else {
                        __res = toupper(*error as u_char as ::core::ffi::c_int);
                    }
                } else {
                    __res = *(*__ctype_toupper_loc())
                        .offset(*error as u_char as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int;
                }
                __res
            }) as ::core::ffi::c_char;
            status_message_set(
                c,
                -(1 as ::core::ffi::c_int),
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                error,
            );
            free(error as *mut ::core::ffi::c_void);
            return PROMPT_CLOSE;
        }
        1 | _ => {
            key_bindings_add(
                (*item).table,
                key,
                ::core::ptr::null::<::core::ffi::c_char>(),
                0 as ::core::ffi::c_int,
                (*pr).cmdlist,
            );
            mode_tree_build((*data).data);
            mode_tree_draw((*data).data);
            (*(*data).wp).flags |= PANE_REDRAW;
            return PROMPT_CLOSE;
        }
    };
}
unsafe extern "C" fn window_customize_add_key(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
    mut table: *const ::core::ffi::c_char,
) {
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut prompt: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    xasprintf(
        &raw mut prompt,
        b"New key in %s: \0" as *const u8 as *const ::core::ffi::c_char,
        table,
    );
    new_item = xcalloc(
        1 as size_t,
        ::core::mem::size_of::<window_customize_itemdata>() as size_t,
    ) as *mut window_customize_itemdata;
    (*new_item).data = data as *mut window_customize_modedata;
    (*new_item).type_0 = WINDOW_CUSTOMIZE_ITEM_KEY;
    (*new_item).scope = WINDOW_CUSTOMIZE_KEY;
    (*new_item).table = xstrdup(table);
    (*data).references += 1;
    mode_tree_set_prompt(
        (*data).data,
        c,
        prompt,
        b"\0" as *const u8 as *const ::core::ffi::c_char,
        PROMPT_TYPE_COMMAND,
        PROMPT_NOFORMAT,
        Some(
            window_customize_add_key_callback
                as unsafe extern "C" fn(
                    *mut client,
                    *mut ::core::ffi::c_void,
                    *const ::core::ffi::c_char,
                    prompt_key_result,
                ) -> prompt_result,
        ),
        Some(
            window_customize_free_item_callback
                as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
        ),
        new_item as *mut ::core::ffi::c_void,
    );
    free(prompt as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_customize_unset_key(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) {
    let mut kt: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    if item.is_null() || window_customize_get_key(item, &raw mut kt, &raw mut bd) == 0 {
        return;
    }
    if item == mode_tree_get_current((*data).data) as *mut window_customize_itemdata {
        mode_tree_up((*data).data, 0 as ::core::ffi::c_int);
    }
    key_bindings_remove((*kt).name, (*bd).key);
}
unsafe extern "C" fn window_customize_reset_key(
    mut data: *mut window_customize_modedata,
    mut item: *mut window_customize_itemdata,
) {
    let mut kt: *mut key_table = ::core::ptr::null_mut::<key_table>();
    let mut dd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    let mut bd: *mut key_binding = ::core::ptr::null_mut::<key_binding>();
    if item.is_null() || window_customize_get_key(item, &raw mut kt, &raw mut bd) == 0 {
        return;
    }
    dd = key_bindings_get_default(kt, (*bd).key);
    if !dd.is_null() && (*bd).cmdlist == (*dd).cmdlist {
        return;
    }
    if dd.is_null() && item == mode_tree_get_current((*data).data) as *mut window_customize_itemdata
    {
        mode_tree_up((*data).data, 0 as ::core::ffi::c_int);
    }
    key_bindings_reset((*kt).name, (*bd).key);
}
unsafe extern "C" fn window_customize_change_each(
    mut modedata: *mut ::core::ffi::c_void,
    mut itemdata: *mut ::core::ffi::c_void,
    mut c: *mut client,
    mut key: key_code,
) {
    let mut data: *mut window_customize_modedata = modedata as *mut window_customize_modedata;
    let mut item: *mut window_customize_itemdata = itemdata as *mut window_customize_itemdata;
    let mut type_0: window_customize_item_type = (*item).type_0;
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        name = xstrdup((*item).name);
    }
    match (*data).change as ::core::ffi::c_uint {
        0 => {
            if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_unset_key(data, item);
            } else if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_unset_environment(data, item);
            } else {
                window_customize_unset_option(data, item);
            }
        }
        1 => {
            if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_reset_key(data, item);
            } else if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_reset_option(data, item);
            }
        }
        _ => {}
    }
    if type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        options_push_changes(name);
    }
    free(name as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn window_customize_change_current_callback(
    mut c: *mut client,
    mut modedata: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut data: *mut window_customize_modedata = modedata as *mut window_customize_modedata;
    let mut item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut type_0: window_customize_item_type = WINDOW_CUSTOMIZE_ITEM_OPTION;
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if ({
        let mut __res: ::core::ffi::c_int = 0;
        if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
            if 0 != 0 {
                let mut __c: ::core::ffi::c_int =
                    *s.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int;
                __res = (if __c < -(128 as ::core::ffi::c_int) || __c > 255 as ::core::ffi::c_int {
                    __c as __int32_t
                } else {
                    *(*__ctype_tolower_loc()).offset(__c as isize)
                }) as ::core::ffi::c_int;
            } else {
                __res = tolower(
                    *s.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int
                );
            }
        } else {
            __res = *(*__ctype_tolower_loc()).offset(*s.offset(0 as ::core::ffi::c_int as isize)
                as u_char as ::core::ffi::c_int
                as isize) as ::core::ffi::c_int;
        }
        __res
    }) != 'y' as i32
        || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
    {
        return PROMPT_CLOSE;
    }
    item = mode_tree_get_current((*data).data) as *mut window_customize_itemdata;
    if item.is_null() {
        return PROMPT_CLOSE;
    }
    type_0 = (*item).type_0;
    if type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        name = xstrdup((*item).name);
    }
    match (*data).change as ::core::ffi::c_uint {
        0 => {
            if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_unset_key(data, item);
            } else if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_unset_environment(data, item);
            } else {
                window_customize_unset_option(data, item);
            }
        }
        1 => {
            if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_reset_key(data, item);
            } else if type_0 as ::core::ffi::c_uint
                == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                window_customize_reset_option(data, item);
            }
        }
        _ => {}
    }
    if type_0 as ::core::ffi::c_uint
        == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        options_push_changes(name);
    }
    free(name as *mut ::core::ffi::c_void);
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*(*data).wp).flags |= PANE_REDRAW;
    return PROMPT_CLOSE;
}
unsafe extern "C" fn window_customize_change_tagged_callback(
    mut c: *mut client,
    mut modedata: *mut ::core::ffi::c_void,
    mut s: *const ::core::ffi::c_char,
    mut key: prompt_key_result,
) -> prompt_result {
    let mut data: *mut window_customize_modedata = modedata as *mut window_customize_modedata;
    if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 || (*data).dead != 0 {
        return PROMPT_CLOSE;
    }
    if ({
        let mut __res: ::core::ffi::c_int = 0;
        if ::core::mem::size_of::<u_char>() as usize > 1 as usize {
            if 0 != 0 {
                let mut __c: ::core::ffi::c_int =
                    *s.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int;
                __res = (if __c < -(128 as ::core::ffi::c_int) || __c > 255 as ::core::ffi::c_int {
                    __c as __int32_t
                } else {
                    *(*__ctype_tolower_loc()).offset(__c as isize)
                }) as ::core::ffi::c_int;
            } else {
                __res = tolower(
                    *s.offset(0 as ::core::ffi::c_int as isize) as u_char as ::core::ffi::c_int
                );
            }
        } else {
            __res = *(*__ctype_tolower_loc()).offset(*s.offset(0 as ::core::ffi::c_int as isize)
                as u_char as ::core::ffi::c_int
                as isize) as ::core::ffi::c_int;
        }
        __res
    }) != 'y' as i32
        || *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
    {
        return PROMPT_CLOSE;
    }
    mode_tree_each_tagged(
        (*data).data,
        Some(
            window_customize_change_each
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut ::core::ffi::c_void,
                    *mut client,
                    key_code,
                ) -> (),
        ),
        c,
        KEYC_NONE as ::core::ffi::c_ulong as key_code,
        0 as ::core::ffi::c_int,
    );
    mode_tree_build((*data).data);
    mode_tree_draw((*data).data);
    (*(*data).wp).flags |= PANE_REDRAW;
    return PROMPT_CLOSE;
}
unsafe extern "C" fn window_customize_add_current(
    mut c: *mut client,
    mut data: *mut window_customize_modedata,
) -> ::core::ffi::c_int {
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut table: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    name = mode_tree_get_current_name((*data).data);
    if cmd_find_valid_state(&raw mut (*data).fs) != 0 {
        cmd_find_copy_state(&raw mut fs, &raw mut (*data).fs);
    } else {
        cmd_find_from_pane(&raw mut fs, (*data).wp, 0 as ::core::ffi::c_int);
    }
    if strcmp(
        name,
        b"Server Options\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        window_customize_add_option(
            c,
            data,
            WINDOW_CUSTOMIZE_SERVER,
            global_options,
            WINDOW_CUSTOMIZE_OPTIONS,
        );
        return 1 as ::core::ffi::c_int;
    }
    if strcmp(
        name,
        b"Session Options\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        window_customize_add_option(
            c,
            data,
            WINDOW_CUSTOMIZE_SESSION,
            (*fs.s).options,
            WINDOW_CUSTOMIZE_OPTIONS,
        );
        return 1 as ::core::ffi::c_int;
    }
    if strcmp(
        name,
        b"Window & Pane Options\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        window_customize_add_option(
            c,
            data,
            WINDOW_CUSTOMIZE_PANE,
            (*fs.wp).options,
            WINDOW_CUSTOMIZE_OPTIONS,
        );
        return 1 as ::core::ffi::c_int;
    }
    if strcmp(
        name,
        b"Session Hooks\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        window_customize_add_option(
            c,
            data,
            WINDOW_CUSTOMIZE_SESSION,
            (*fs.s).options,
            WINDOW_CUSTOMIZE_HOOKS,
        );
        return 1 as ::core::ffi::c_int;
    }
    if strcmp(
        name,
        b"Window & Pane Hooks\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        window_customize_add_option(
            c,
            data,
            WINDOW_CUSTOMIZE_PANE,
            (*fs.wp).options,
            WINDOW_CUSTOMIZE_HOOKS,
        );
        return 1 as ::core::ffi::c_int;
    }
    if strcmp(
        name,
        b"Global Environment\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        window_customize_add_environment(
            c,
            data,
            WINDOW_CUSTOMIZE_GLOBAL_ENVIRONMENT,
            global_environ,
        );
        return 1 as ::core::ffi::c_int;
    }
    if strcmp(
        name,
        b"Session Environment\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        window_customize_add_environment(
            c,
            data,
            WINDOW_CUSTOMIZE_SESSION_ENVIRONMENT,
            (*fs.s).environ,
        );
        return 1 as ::core::ffi::c_int;
    }
    if strncmp(
        name,
        b"Key Table - \0" as *const u8 as *const ::core::ffi::c_char,
        12 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        table = name.offset(12 as ::core::ffi::c_int as isize);
        window_customize_add_key(c, data, table);
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn window_customize_key(
    mut wme: *mut window_mode_entry,
    mut c: *mut client,
    mut s: *mut session,
    mut wl: *mut winlink,
    mut key: key_code,
    mut m: *mut mouse_event,
) {
    let mut wp: *mut window_pane = (*wme).wp;
    let mut data: *mut window_customize_modedata = (*wme).data as *mut window_customize_modedata;
    let mut item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut new_item: *mut window_customize_itemdata =
        ::core::ptr::null_mut::<window_customize_itemdata>();
    let mut finished: ::core::ffi::c_int = 0;
    let mut prompt: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tagged: u_int = 0;
    item = mode_tree_get_current((*data).data) as *mut window_customize_itemdata;
    if !(*data).editor.is_null() {
        if key == 'q' as i32 as key_code
            || key == '\u{1b}' as i32 as key_code
            || key == '\u{3}' as i32 as key_code
        {
            finished = 1 as ::core::ffi::c_int;
        } else {
            finished = 0 as ::core::ffi::c_int;
        }
    } else {
        finished = mode_tree_key(
            (*data).data,
            c,
            &raw mut key,
            m,
            ::core::ptr::null_mut::<u_int>(),
            ::core::ptr::null_mut::<u_int>(),
        );
        new_item = mode_tree_get_current((*data).data) as *mut window_customize_itemdata;
        if item != new_item {
            item = new_item;
        }
        match key {
            101 => {
                window_customize_start_edit(data, item, c);
            }
            97 => {
                if !(item.is_null()
                    || (*item).type_0 as ::core::ffi::c_uint
                        != WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int
                            as ::core::ffi::c_uint)
                {
                    window_customize_set_array_key(c, data, item);
                }
            }
            13 | 115 => {
                if item.is_null() {
                    if window_customize_add_current(c, data) != 0 {
                        mode_tree_build((*data).data);
                    }
                } else {
                    if (*item).type_0 as ::core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        window_customize_set_key(c, data, item);
                    } else if (*item).type_0 as ::core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                    {
                        window_customize_set_environment(c, data, item, 0 as ::core::ffi::c_int);
                    } else {
                        window_customize_set_option(
                            c,
                            data,
                            item,
                            0 as ::core::ffi::c_int,
                            1 as ::core::ffi::c_int,
                        );
                        options_push_changes((*item).name);
                    }
                    mode_tree_build((*data).data);
                }
            }
            119 => {
                if !(item.is_null()
                    || (*item).type_0 as ::core::ffi::c_uint
                        != WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int
                            as ::core::ffi::c_uint)
                {
                    window_customize_set_option(
                        c,
                        data,
                        item,
                        0 as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                    );
                    options_push_changes((*item).name);
                    mode_tree_build((*data).data);
                }
            }
            83 | 87 => {
                if !(item.is_null()
                    || (*item).type_0 as ::core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_ITEM_KEY as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    if (*item).type_0 as ::core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                    {
                        window_customize_set_environment(c, data, item, 1 as ::core::ffi::c_int);
                    } else {
                        window_customize_set_option(
                            c,
                            data,
                            item,
                            1 as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        );
                        options_push_changes((*item).name);
                    }
                    mode_tree_build((*data).data);
                }
            }
            100 => {
                if !(item.is_null()
                    || (*item).type_0 as ::core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_ITEM_OPTION as ::core::ffi::c_int
                            as ::core::ffi::c_uint
                        && !(*item).array_key.is_null()
                    || (*item).type_0 as ::core::ffi::c_uint
                        == WINDOW_CUSTOMIZE_ITEM_ENVIRONMENT as ::core::ffi::c_int
                            as ::core::ffi::c_uint)
                {
                    xasprintf(
                        &raw mut prompt,
                        b"Reset %s to default? \0" as *const u8 as *const ::core::ffi::c_char,
                        (*item).name,
                    );
                    (*data).references += 1;
                    (*data).change = WINDOW_CUSTOMIZE_RESET;
                    mode_tree_set_prompt(
                        (*data).data,
                        c,
                        prompt,
                        b"\0" as *const u8 as *const ::core::ffi::c_char,
                        PROMPT_TYPE_COMMAND,
                        PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                        Some(
                            window_customize_change_current_callback
                                as unsafe extern "C" fn(
                                    *mut client,
                                    *mut ::core::ffi::c_void,
                                    *const ::core::ffi::c_char,
                                    prompt_key_result,
                                )
                                    -> prompt_result,
                        ),
                        Some(
                            window_customize_free_callback
                                as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
                        ),
                        data as *mut ::core::ffi::c_void,
                    );
                    free(prompt as *mut ::core::ffi::c_void);
                }
            }
            68 => {
                tagged = mode_tree_count_tagged((*data).data);
                if !(tagged == 0 as u_int) {
                    xasprintf(
                        &raw mut prompt,
                        b"Reset %u tagged to default? \0" as *const u8
                            as *const ::core::ffi::c_char,
                        tagged,
                    );
                    (*data).references += 1;
                    (*data).change = WINDOW_CUSTOMIZE_RESET;
                    mode_tree_set_prompt(
                        (*data).data,
                        c,
                        prompt,
                        b"\0" as *const u8 as *const ::core::ffi::c_char,
                        PROMPT_TYPE_COMMAND,
                        PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                        Some(
                            window_customize_change_tagged_callback
                                as unsafe extern "C" fn(
                                    *mut client,
                                    *mut ::core::ffi::c_void,
                                    *const ::core::ffi::c_char,
                                    prompt_key_result,
                                )
                                    -> prompt_result,
                        ),
                        Some(
                            window_customize_free_callback
                                as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
                        ),
                        data as *mut ::core::ffi::c_void,
                    );
                    free(prompt as *mut ::core::ffi::c_void);
                }
            }
            117 => {
                if !item.is_null() {
                    if !(*item).array_key.is_null() {
                        xasprintf(
                            &raw mut prompt,
                            b"Unset %s[%s]? \0" as *const u8 as *const ::core::ffi::c_char,
                            (*item).name,
                            (*item).array_key,
                        );
                    } else {
                        xasprintf(
                            &raw mut prompt,
                            b"Unset %s? \0" as *const u8 as *const ::core::ffi::c_char,
                            (*item).name,
                        );
                    }
                    (*data).references += 1;
                    (*data).change = WINDOW_CUSTOMIZE_UNSET;
                    mode_tree_set_prompt(
                        (*data).data,
                        c,
                        prompt,
                        b"\0" as *const u8 as *const ::core::ffi::c_char,
                        PROMPT_TYPE_COMMAND,
                        PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                        Some(
                            window_customize_change_current_callback
                                as unsafe extern "C" fn(
                                    *mut client,
                                    *mut ::core::ffi::c_void,
                                    *const ::core::ffi::c_char,
                                    prompt_key_result,
                                )
                                    -> prompt_result,
                        ),
                        Some(
                            window_customize_free_callback
                                as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
                        ),
                        data as *mut ::core::ffi::c_void,
                    );
                    free(prompt as *mut ::core::ffi::c_void);
                }
            }
            85 => {
                tagged = mode_tree_count_tagged((*data).data);
                if !(tagged == 0 as u_int) {
                    xasprintf(
                        &raw mut prompt,
                        b"Unset %u tagged? \0" as *const u8 as *const ::core::ffi::c_char,
                        tagged,
                    );
                    (*data).references += 1;
                    (*data).change = WINDOW_CUSTOMIZE_UNSET;
                    mode_tree_set_prompt(
                        (*data).data,
                        c,
                        prompt,
                        b"\0" as *const u8 as *const ::core::ffi::c_char,
                        PROMPT_TYPE_COMMAND,
                        PROMPT_SINGLE | PROMPT_NOFORMAT | (*data).prompt_flags,
                        Some(
                            window_customize_change_tagged_callback
                                as unsafe extern "C" fn(
                                    *mut client,
                                    *mut ::core::ffi::c_void,
                                    *const ::core::ffi::c_char,
                                    prompt_key_result,
                                )
                                    -> prompt_result,
                        ),
                        Some(
                            window_customize_free_callback
                                as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
                        ),
                        data as *mut ::core::ffi::c_void,
                    );
                    free(prompt as *mut ::core::ffi::c_void);
                }
            }
            72 => {
                (*data).hide_global = ((*data).hide_global == 0) as ::core::ffi::c_int;
                mode_tree_build((*data).data);
            }
            67 => {
                (*data).hide_default = ((*data).hide_default == 0) as ::core::ffi::c_int;
                mode_tree_build((*data).data);
            }
            _ => {}
        }
    }
    if finished != 0 {
        window_pane_reset_mode(wp);
    } else {
        mode_tree_draw((*data).data);
        window_customize_draw_waiting(data);
        (*wp).flags |= PANE_REDRAW;
    };
}

pub use crate::src::arguments::args_parse;
use crate::src::arguments::{args_copy, args_escape_cstring, args_free, args_print_cstring};
use crate::src::cmd_attach_session::cmd_attach_session_entry;
use crate::src::cmd_bind_key::cmd_bind_key_entry;
use crate::src::cmd_break_pane::cmd_break_pane_entry;
use crate::src::cmd_capture_pane::{cmd_capture_pane_entry, cmd_clear_history_entry};
use crate::src::cmd_choose_tree::{
    cmd_choose_buffer_entry, cmd_choose_client_entry, cmd_choose_tree_entry,
    cmd_customize_mode_entry, cmd_display_panes_entry, cmd_switch_mode_entry,
};
use crate::src::cmd_command_prompt::cmd_command_prompt_entry;
use crate::src::cmd_confirm_before::cmd_confirm_before_entry;
use crate::src::cmd_copy_mode::{cmd_clock_mode_entry, cmd_copy_mode_entry};
use crate::src::cmd_detach_client::{cmd_detach_client_entry, cmd_suspend_client_entry};
use crate::src::cmd_display_menu::{cmd_display_menu_entry, cmd_display_popup_entry};
use crate::src::cmd_display_message::cmd_display_message_entry;
use crate::src::cmd_find_window::cmd_find_window_entry;
use crate::src::cmd_if_shell::cmd_if_shell_entry;
use crate::src::cmd_join_pane::{cmd_join_pane_entry, cmd_move_pane_entry};
use crate::src::cmd_kill_pane::cmd_kill_pane_entry;
use crate::src::cmd_kill_server::{cmd_kill_server_entry, cmd_start_server_entry};
use crate::src::cmd_kill_session::cmd_kill_session_entry;
use crate::src::cmd_kill_window::{cmd_kill_window_entry, cmd_unlink_window_entry};
use crate::src::cmd_list_buffers::cmd_list_buffers_entry;
use crate::src::cmd_list_clients::cmd_list_clients_entry;
use crate::src::cmd_list_commands::cmd_list_commands_entry;
use crate::src::cmd_list_keys::cmd_list_keys_entry;
use crate::src::cmd_list_panes::cmd_list_panes_entry;
use crate::src::cmd_list_sessions::cmd_list_sessions_entry;
use crate::src::cmd_list_windows::cmd_list_windows_entry;
use crate::src::cmd_load_buffer::cmd_load_buffer_entry;
use crate::src::cmd_lock_server::{
    cmd_lock_client_entry, cmd_lock_server_entry, cmd_lock_session_entry,
};
use crate::src::cmd_move_window::{cmd_link_window_entry, cmd_move_window_entry};
use crate::src::cmd_new_session::{cmd_has_session_entry, cmd_new_session_entry};
use crate::src::cmd_new_window::cmd_new_window_entry;
use crate::src::cmd_paste_buffer::cmd_paste_buffer_entry;
use crate::src::cmd_pipe_pane::cmd_pipe_pane_entry;
use crate::src::cmd_refresh_client::cmd_refresh_client_entry;
use crate::src::cmd_rename_session::cmd_rename_session_entry;
use crate::src::cmd_rename_window::cmd_rename_window_entry;
use crate::src::cmd_resize_pane::cmd_resize_pane_entry;
use crate::src::cmd_resize_window::cmd_resize_window_entry;
use crate::src::cmd_respawn_pane::cmd_respawn_pane_entry;
use crate::src::cmd_respawn_window::cmd_respawn_window_entry;
use crate::src::cmd_rotate_window::cmd_rotate_window_entry;
use crate::src::cmd_run_shell::cmd_run_shell_entry;
use crate::src::cmd_save_buffer::{cmd_save_buffer_entry, cmd_show_buffer_entry};
use crate::src::cmd_select_layout::{
    cmd_next_layout_entry, cmd_previous_layout_entry, cmd_select_layout_entry,
};
use crate::src::cmd_select_pane::{cmd_last_pane_entry, cmd_select_pane_entry};
use crate::src::cmd_select_window::{
    cmd_last_window_entry, cmd_next_window_entry, cmd_previous_window_entry,
    cmd_select_window_entry,
};
use crate::src::cmd_send_keys::{cmd_send_keys_entry, cmd_send_prefix_entry};
use crate::src::cmd_server_access::cmd_server_access_entry;
use crate::src::cmd_set_buffer::{cmd_delete_buffer_entry, cmd_set_buffer_entry};
use crate::src::cmd_set_environment::cmd_set_environment_entry;
use crate::src::cmd_set_option::{
    cmd_set_hook_entry, cmd_set_option_entry, cmd_set_window_option_entry,
};
use crate::src::cmd_show_environment::cmd_show_environment_entry;
use crate::src::cmd_show_messages::cmd_show_messages_entry;
use crate::src::cmd_show_options::{
    cmd_show_hooks_entry, cmd_show_options_entry, cmd_show_window_options_entry,
};
use crate::src::cmd_show_prompt_history::{
    cmd_clear_prompt_history_entry, cmd_show_prompt_history_entry,
};
use crate::src::cmd_source_file::cmd_source_file_entry;
use crate::src::cmd_split_window::{cmd_new_pane_entry, cmd_split_window_entry};
use crate::src::cmd_swap_pane::cmd_swap_pane_entry;
use crate::src::cmd_swap_window::cmd_swap_window_entry;
use crate::src::cmd_switch_client::cmd_switch_client_entry;
use crate::src::cmd_unbind_key::cmd_unbind_key_entry;
use crate::src::cmd_wait_for::cmd_wait_for_entry;
use crate::src::ffi::libc::{free, strchr, strcmp, strlcat, strlcpy, strlen, strncmp};
use crate::src::log::{fatalx, log_debug};
use crate::src::options::{
    options_array_first, options_array_item_value, options_array_next, options_get_only,
};
use crate::src::session::session_find_by_id;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{
    args, args_parse_cb, args_value, args_value_c2rust_unnamed, args_value_entry,
};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmd_qentry, cmdq_item, cmdq_list,
    cmds,
};
pub use crate::src::shared::command::{CMD_LIST_PRINT_ESCAPED, CMD_LIST_PRINT_NO_GROUPS};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::format::{format_job_tree, format_tree};
use crate::src::shared::grid::*;
pub use crate::src::shared::hyperlinks::hyperlinks;
pub use crate::src::shared::input::{input_ctx, input_request, input_requests};
use crate::src::shared::key::*;
pub use crate::src::shared::key::{
    key_binding, key_binding_entry, key_bindings, key_event, key_table, key_table_entry,
};
pub use crate::src::shared::layout::layout_geometry;
use crate::src::shared::layout::*;
pub use crate::src::shared::layout::{layout_cell, layout_cell_entry, layout_cells};
pub use crate::src::shared::limits::SIZE_MAX;
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::{
    options, options_array, options_array_item, options_entry, options_value,
};
pub use crate::src::shared::pane::{
    window_pane, window_pane_entry, window_pane_modes, window_pane_prompt, window_pane_sentry,
    window_pane_tree_entry, window_pane_zentry, window_panes,
};
pub use crate::src::shared::pane::{
    window_pane_offset, window_pane_resize, window_pane_resize_entry, window_pane_resizes,
};
pub use crate::src::shared::process::tmuxpeer;
pub use crate::src::shared::prompt::prompt;
pub use crate::src::shared::redraw::redraw_scene;
pub use crate::src::shared::screen::{screen, screen_sel, screen_titles};
pub use crate::src::shared::screen_write::screen_write_cline;
pub use crate::src::shared::session::{session, session_entry, session_gentry};
pub use crate::src::shared::spawn::spawn_editor_state;
pub use crate::src::shared::status::status_line;
use crate::src::shared::style::*;
use crate::src::shared::terminal::*;
pub use crate::src::shared::tty::{tty, tty_code, tty_key, tty_term, tty_term_entry};
pub use crate::src::shared::variadic::{__builtin_va_list, __va_list_tag, va_list};
pub use crate::src::shared::window::{
    window, window_alerts_entry, window_entry, window_mode, window_mode_entry,
    window_mode_entry_entry, window_winlinks, winlink, winlink_entry, winlink_sentry,
    winlink_stack, winlink_wentry, winlinks,
};
use crate::src::tmux::global_options;
use crate::src::window::{
    window_find_by_id, window_has_pane, window_pane_find_by_id, winlink_find_by_window,
};
use crate::src::xmalloc::{
    xasprintf, xcalloc, xmalloc, xrealloc, xreallocarray, xstrdup, xvasprintf_cstring,
};
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub const DQ: C2RustUnnamed_38 = 2;
pub type C2RustUnnamed_38 = ::core::ffi::c_uint;
pub const SQ: C2RustUnnamed_38 = 1;
pub const NQ: C2RustUnnamed_38 = 0;

#[no_mangle]
pub static mut cmd_table: [*const cmd_entry; 93] = unsafe {
    [
        &raw const cmd_attach_session_entry,
        &raw const cmd_bind_key_entry,
        &raw const cmd_break_pane_entry,
        &raw const cmd_capture_pane_entry,
        &raw const cmd_choose_buffer_entry,
        &raw const cmd_choose_client_entry,
        &raw const cmd_choose_tree_entry,
        &raw const cmd_clear_history_entry,
        &raw const cmd_clear_prompt_history_entry,
        &raw const cmd_clock_mode_entry,
        &raw const cmd_command_prompt_entry,
        &raw const cmd_confirm_before_entry,
        &raw const cmd_copy_mode_entry,
        &raw const cmd_customize_mode_entry,
        &raw const cmd_delete_buffer_entry,
        &raw const cmd_detach_client_entry,
        &raw const cmd_display_menu_entry,
        &raw const cmd_display_message_entry,
        &raw const cmd_display_popup_entry,
        &raw const cmd_display_panes_entry,
        &raw const cmd_find_window_entry,
        &raw const cmd_has_session_entry,
        &raw const cmd_if_shell_entry,
        &raw const cmd_join_pane_entry,
        &raw const cmd_kill_pane_entry,
        &raw const cmd_kill_server_entry,
        &raw const cmd_kill_session_entry,
        &raw const cmd_kill_window_entry,
        &raw const cmd_last_pane_entry,
        &raw const cmd_last_window_entry,
        &raw const cmd_link_window_entry,
        &raw const cmd_list_buffers_entry,
        &raw const cmd_list_clients_entry,
        &raw const cmd_list_commands_entry,
        &raw const cmd_list_keys_entry,
        &raw const cmd_list_panes_entry,
        &raw const cmd_list_sessions_entry,
        &raw const cmd_list_windows_entry,
        &raw const cmd_load_buffer_entry,
        &raw const cmd_lock_client_entry,
        &raw const cmd_lock_server_entry,
        &raw const cmd_lock_session_entry,
        &raw const cmd_move_pane_entry,
        &raw const cmd_move_window_entry,
        &raw const cmd_new_pane_entry,
        &raw const cmd_new_session_entry,
        &raw const cmd_new_window_entry,
        &raw const cmd_next_layout_entry,
        &raw const cmd_next_window_entry,
        &raw const cmd_paste_buffer_entry,
        &raw const cmd_pipe_pane_entry,
        &raw const cmd_previous_layout_entry,
        &raw const cmd_previous_window_entry,
        &raw const cmd_refresh_client_entry,
        &raw const cmd_rename_session_entry,
        &raw const cmd_rename_window_entry,
        &raw const cmd_resize_pane_entry,
        &raw const cmd_resize_window_entry,
        &raw const cmd_respawn_pane_entry,
        &raw const cmd_respawn_window_entry,
        &raw const cmd_rotate_window_entry,
        &raw const cmd_run_shell_entry,
        &raw const cmd_save_buffer_entry,
        &raw const cmd_select_layout_entry,
        &raw const cmd_select_pane_entry,
        &raw const cmd_select_window_entry,
        &raw const cmd_send_keys_entry,
        &raw const cmd_send_prefix_entry,
        &raw const cmd_server_access_entry,
        &raw const cmd_set_buffer_entry,
        &raw const cmd_set_environment_entry,
        &raw const cmd_set_hook_entry,
        &raw const cmd_set_option_entry,
        &raw const cmd_set_window_option_entry,
        &raw const cmd_show_buffer_entry,
        &raw const cmd_show_environment_entry,
        &raw const cmd_show_hooks_entry,
        &raw const cmd_show_messages_entry,
        &raw const cmd_show_options_entry,
        &raw const cmd_show_prompt_history_entry,
        &raw const cmd_show_window_options_entry,
        &raw const cmd_source_file_entry,
        &raw const cmd_split_window_entry,
        &raw const cmd_start_server_entry,
        &raw const cmd_suspend_client_entry,
        &raw const cmd_swap_pane_entry,
        &raw const cmd_swap_window_entry,
        &raw const cmd_switch_client_entry,
        &raw const cmd_switch_mode_entry,
        &raw const cmd_unbind_key_entry,
        &raw const cmd_unlink_window_entry,
        &raw const cmd_wait_for_entry,
        ::core::ptr::null::<cmd_entry>(),
    ]
};
static mut cmd_list_next_group: u_int = 1 as u_int;
#[no_mangle]
pub unsafe extern "C" fn cmd_log_argv(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    let mut i: ::core::ffi::c_int = 0;
    ap = args.clone();
    let prefix = xvasprintf_cstring(fmt, ap);
    i = 0 as ::core::ffi::c_int;
    while i < argc {
        log_debug(
            b"%s: argv[%d]=%s\0" as *const u8 as *const ::core::ffi::c_char,
            prefix.as_ptr(),
            i,
            *argv.offset(i as isize),
        );
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn cmd_prepend_argv(
    mut argc: *mut ::core::ffi::c_int,
    mut argv: *mut *mut *mut ::core::ffi::c_char,
    mut arg: *const ::core::ffi::c_char,
) {
    let mut new_argv: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0;
    new_argv = xreallocarray(
        NULL,
        (*argc + 1 as ::core::ffi::c_int) as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
    ) as *mut *mut ::core::ffi::c_char;
    let ref mut fresh0 = *new_argv.offset(0 as ::core::ffi::c_int as isize);
    *fresh0 = xstrdup(arg);
    i = 0 as ::core::ffi::c_int;
    while i < *argc {
        let ref mut fresh1 = *new_argv.offset((1 as ::core::ffi::c_int + i) as isize);
        *fresh1 = *(*argv).offset(i as isize);
        i += 1;
    }
    free(*argv as *mut ::core::ffi::c_void);
    *argv = new_argv;
    *argc += 1;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_append_argv(
    mut argc: *mut ::core::ffi::c_int,
    mut argv: *mut *mut *mut ::core::ffi::c_char,
    mut arg: *const ::core::ffi::c_char,
) {
    *argv = xreallocarray(
        *argv as *mut ::core::ffi::c_void,
        (*argc + 1 as ::core::ffi::c_int) as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
    ) as *mut *mut ::core::ffi::c_char;
    let fresh2 = *argc;
    *argc = *argc + 1;
    let ref mut fresh3 = *(*argv).offset(fresh2 as isize);
    *fresh3 = xstrdup(arg);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_pack_argv(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut arglen: size_t = 0;
    let mut i: ::core::ffi::c_int = 0;
    if argc == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    cmd_log_argv(
        argc,
        argv,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_pack_argv\0" as *const u8 as *const ::core::ffi::c_char,
    );
    *buf = '\0' as i32 as ::core::ffi::c_char;
    i = 0 as ::core::ffi::c_int;
    while i < argc {
        if strlcpy(buf, *argv.offset(i as isize), len) as size_t >= len {
            return -(1 as ::core::ffi::c_int);
        }
        arglen = strlen(*argv.offset(i as isize)).wrapping_add(1 as size_t);
        buf = buf.offset(arglen as isize);
        len = len.wrapping_sub(arglen);
        i += 1;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_unpack_argv(
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut arglen: size_t = 0;
    if argc == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if argc < 0 as ::core::ffi::c_int || argc > 1000 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    *argv = xcalloc(
        argc as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
    ) as *mut *mut ::core::ffi::c_char;
    *buf.offset(len.wrapping_sub(1 as size_t) as isize) = '\0' as i32 as ::core::ffi::c_char;
    i = 0 as ::core::ffi::c_int;
    while i < argc {
        if len == 0 as size_t {
            cmd_free_argv(argc, *argv);
            return -(1 as ::core::ffi::c_int);
        }
        arglen = strlen(buf).wrapping_add(1 as size_t);
        let ref mut fresh4 = *(*argv).offset(i as isize);
        *fresh4 = xstrdup(buf);
        buf = buf.offset(arglen as isize);
        len = len.wrapping_sub(arglen);
        i += 1;
    }
    cmd_log_argv(
        argc,
        *argv,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_unpack_argv\0" as *const u8 as *const ::core::ffi::c_char,
    );
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_copy_argv(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) -> *mut *mut ::core::ffi::c_char {
    let mut new_argv: *mut *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0;
    if argc == 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
    }
    new_argv = xcalloc(
        (argc + 1 as ::core::ffi::c_int) as size_t,
        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
    ) as *mut *mut ::core::ffi::c_char;
    i = 0 as ::core::ffi::c_int;
    while i < argc {
        if !(*argv.offset(i as isize)).is_null() {
            let ref mut fresh5 = *new_argv.offset(i as isize);
            *fresh5 = xstrdup(*argv.offset(i as isize));
        }
        i += 1;
    }
    return new_argv;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_free_argv(
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) {
    let mut i: ::core::ffi::c_int = 0;
    if argc == 0 as ::core::ffi::c_int {
        return;
    }
    i = 0 as ::core::ffi::c_int;
    while i < argc {
        free(*argv.offset(i as isize) as *mut ::core::ffi::c_void);
        i += 1;
    }
    free(argv as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn cmd_stringify_argv(
    argc: ::core::ffi::c_int,
    argv: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    // The exported result is C-owned; retained users and callback results free it.
    cmd_stringify_argv_cstring(argc, argv)
        .map_or(::core::ptr::null_mut(), |text| xstrdup(text.as_ptr()))
}

pub(crate) unsafe fn cmd_stringify_argv_cstring(
    argc: ::core::ffi::c_int,
    argv: *mut *mut ::core::ffi::c_char,
) -> Option<CString> {
    // The original C function returned NULL for a negative count.
    if argc < 0 {
        return None;
    }
    let mut bytes = Vec::new();
    for i in 0..argc {
        let argument = *argv.offset(i as isize);
        let escaped = args_escape_cstring(CStr::from_ptr(argument));
        log_debug(
            b"%s: %u %s = %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_stringify_argv\0" as *const u8 as *const ::core::ffi::c_char,
            i,
            argument,
            escaped.as_ptr(),
        );
        if i != 0 {
            bytes.push(b' ');
        }
        bytes.extend_from_slice(escaped.as_bytes());
    }
    Some(CString::new(bytes).expect("escaped argv contains no interior NUL"))
}
#[no_mangle]
pub unsafe extern "C" fn cmd_get_entry(mut cmd: *mut cmd) -> *const cmd_entry {
    return (*cmd).entry;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_get_args(mut cmd: *mut cmd) -> *mut args {
    return (*cmd).args;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_get_group(mut cmd: *mut cmd) -> u_int {
    return (*cmd).group;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_get_source(
    mut cmd: *mut cmd,
    mut file: *mut *const ::core::ffi::c_char,
    mut line: *mut u_int,
) {
    if !file.is_null() {
        *file = (*cmd).file;
    }
    if !line.is_null() {
        *line = (*cmd).line;
    }
}
#[no_mangle]
pub unsafe extern "C" fn cmd_get_parse_flags(mut cmd: *mut cmd) -> ::core::ffi::c_int {
    return (*cmd).parse_flags;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_get_alias(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut o: *mut options_entry = ::core::ptr::null_mut::<options_entry>();
    let mut a: *mut options_array_item = ::core::ptr::null_mut::<options_array_item>();
    let mut ov: *mut options_value = ::core::ptr::null_mut::<options_value>();
    let mut wanted: size_t = 0;
    let mut n: size_t = 0;
    let mut equals: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    o = options_get_only(
        global_options,
        b"command-alias\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if o.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    wanted = strlen(name);
    a = options_array_first(o);
    while !a.is_null() {
        ov = options_array_item_value(a);
        equals = strchr((*ov).string, '=' as i32);
        if !equals.is_null() {
            n = equals.offset_from((*ov).string) as ::core::ffi::c_long as size_t;
            if n == wanted && strncmp(name, (*ov).string, n) == 0 as ::core::ffi::c_int {
                return xstrdup(equals.offset(1 as ::core::ffi::c_int as isize));
            }
        }
        a = options_array_next(a);
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn cmd_find(
    mut name: *const ::core::ffi::c_char,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *const cmd_entry {
    let mut loop_0: *mut *const cmd_entry = ::core::ptr::null_mut::<*const cmd_entry>();
    let mut entry: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
    let mut found: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
    let mut ambiguous: ::core::ffi::c_int = 0;
    let mut s: [::core::ffi::c_char; 8192] = [0; 8192];
    ambiguous = 0 as ::core::ffi::c_int;
    loop_0 = &raw mut cmd_table as *mut *const cmd_entry;
    while !(*loop_0).is_null() {
        entry = *loop_0;
        if !(*entry).alias.is_null() && strcmp((*entry).alias, name) == 0 as ::core::ffi::c_int {
            ambiguous = 0 as ::core::ffi::c_int;
            found = entry;
            break;
        } else {
            if !(strncmp((*entry).name, name, strlen(name)) != 0 as ::core::ffi::c_int) {
                if !found.is_null() {
                    ambiguous = 1 as ::core::ffi::c_int;
                }
                found = entry;
                if strcmp((*entry).name, name) == 0 as ::core::ffi::c_int {
                    break;
                }
            }
            loop_0 = loop_0.offset(1);
        }
    }
    if ambiguous != 0 {
        *(&raw mut s as *mut ::core::ffi::c_char) = '\0' as i32 as ::core::ffi::c_char;
        loop_0 = &raw mut cmd_table as *mut *const cmd_entry;
        while !(*loop_0).is_null() {
            entry = *loop_0;
            if !(strncmp((*entry).name, name, strlen(name)) != 0 as ::core::ffi::c_int) {
                if strlcat(
                    &raw mut s as *mut ::core::ffi::c_char,
                    (*entry).name,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as size_t,
                ) as usize
                    >= ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as usize
                {
                    break;
                }
                if strlcat(
                    &raw mut s as *mut ::core::ffi::c_char,
                    b", \0" as *const u8 as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as size_t,
                ) as usize
                    >= ::core::mem::size_of::<[::core::ffi::c_char; 8192]>() as usize
                {
                    break;
                }
            }
            loop_0 = loop_0.offset(1);
        }
        s[strlen(&raw mut s as *mut ::core::ffi::c_char).wrapping_sub(2 as size_t) as usize] =
            '\0' as i32 as ::core::ffi::c_char;
        xasprintf(
            cause,
            b"ambiguous command: %s, could be: %s\0" as *const u8 as *const ::core::ffi::c_char,
            name,
            &raw mut s as *mut ::core::ffi::c_char,
        );
        return ::core::ptr::null::<cmd_entry>();
    } else {
        if found.is_null() {
            xasprintf(
                cause,
                b"unknown command: %s\0" as *const u8 as *const ::core::ffi::c_char,
                name,
            );
            return ::core::ptr::null::<cmd_entry>();
        }
        return found;
    };
}
/// The public command borrows its source filename from this stable box.
#[repr(C)]
struct CmdOwner {
    node: cmd,
    file: Option<CString>,
}

const _: () = assert!(::core::mem::offset_of!(CmdOwner, node) == 0);

unsafe fn cmd_new_owned(file: *const ::core::ffi::c_char) -> *mut cmd {
    let mut owner = Box::new(CmdOwner {
        node: ::core::mem::zeroed::<cmd>(),
        file: if file.is_null() {
            None
        } else {
            Some(CStr::from_ptr(file).to_owned())
        },
    });
    owner.node.file = owner
        .file
        .as_ref()
        .map_or(::core::ptr::null_mut(), |file| file.as_ptr() as *mut _);
    Box::into_raw(owner).cast::<cmd>()
}

#[no_mangle]
pub unsafe extern "C" fn cmd_parse(
    mut values: *mut args_value,
    mut count: u_int,
    mut file: *const ::core::ffi::c_char,
    mut line: u_int,
    mut parse_flags: ::core::ffi::c_int,
    mut cause: *mut *mut ::core::ffi::c_char,
) -> *mut cmd {
    let mut entry: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
    let mut cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut args: *mut args = ::core::ptr::null_mut::<args>();
    let mut error: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if count == 0 as u_int
        || (*values.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        xasprintf(
            cause,
            b"no command\0" as *const u8 as *const ::core::ffi::c_char,
        );
        return ::core::ptr::null_mut::<cmd>();
    }
    entry = cmd_find(
        (*values.offset(0 as ::core::ffi::c_int as isize))
            .c2rust_unnamed
            .string,
        cause,
    );
    if entry.is_null() {
        return ::core::ptr::null_mut::<cmd>();
    }
    args = args_parse(&raw const (*entry).args, values, count, &raw mut error);
    if args.is_null() && error.is_null() {
        xasprintf(
            cause,
            b"usage: %s %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*entry).name,
            (*entry).usage,
        );
        return ::core::ptr::null_mut::<cmd>();
    }
    if args.is_null() {
        xasprintf(
            cause,
            b"command %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*entry).name,
            error,
        );
        free(error as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<cmd>();
    }
    cmd = cmd_new_owned(file);
    (*cmd).entry = entry;
    (*cmd).args = args;
    (*cmd).parse_flags = parse_flags;
    (*cmd).line = line;
    return cmd;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_free(mut cmd: *mut cmd) {
    args_free((*cmd).args);
    drop(Box::from_raw(cmd.cast::<CmdOwner>()));
}
#[no_mangle]
pub unsafe extern "C" fn cmd_copy(
    mut cmd: *mut cmd,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) -> *mut cmd {
    let mut new_cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    new_cmd = cmd_new_owned((*cmd).file);
    (*new_cmd).entry = (*cmd).entry;
    (*new_cmd).args = args_copy((*cmd).args, argc, argv);
    (*new_cmd).line = (*cmd).line;
    return new_cmd;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_print(cmd: *mut cmd) -> *mut ::core::ffi::c_char {
    let printed = cmd_print_cstring(cmd);
    xstrdup(printed.as_ptr())
}

pub(crate) unsafe fn cmd_print_cstring(cmd: *mut cmd) -> CString {
    let args = args_print_cstring((*cmd).args);
    let arguments = args.as_bytes();
    let name = CStr::from_ptr((*(*cmd).entry).name).to_bytes();
    let mut buf = Vec::with_capacity(
        name.len()
            + if arguments.is_empty() {
                0
            } else {
                arguments.len() + 1
            },
    );
    buf.extend_from_slice(name);
    if !arguments.is_empty() {
        buf.push(b' ');
        buf.extend_from_slice(arguments);
    }
    CString::new(buf).expect("command print contains no interior NUL")
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_new() -> *mut cmd_list {
    let mut cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    cmdlist = Box::into_raw(Box::new(::core::mem::zeroed::<cmd_list>()));
    (*cmdlist).references = 1 as ::core::ffi::c_int;
    let fresh6 = cmd_list_next_group;
    cmd_list_next_group = cmd_list_next_group.wrapping_add(1);
    (*cmdlist).group = fresh6;
    (*cmdlist).list = Box::into_raw(Box::new(::core::mem::zeroed::<cmds>()));
    (*(*cmdlist).list).tqh_first = ::core::ptr::null_mut::<cmd>();
    (*(*cmdlist).list).tqh_last = &raw mut (*(*cmdlist).list).tqh_first;
    return cmdlist;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_append(mut cmdlist: *mut cmd_list, mut cmd: *mut cmd) {
    (*cmd).group = (*cmdlist).group;
    (*cmd).qentry.tqe_next = ::core::ptr::null_mut::<cmd>();
    (*cmd).qentry.tqe_prev = (*(*cmdlist).list).tqh_last;
    *(*(*cmdlist).list).tqh_last = cmd;
    (*(*cmdlist).list).tqh_last = &raw mut (*cmd).qentry.tqe_next;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_append_all(mut cmdlist: *mut cmd_list, mut from: *mut cmd_list) {
    let mut cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    cmd = (*(*from).list).tqh_first;
    while !cmd.is_null() {
        (*cmd).group = (*cmdlist).group;
        cmd = (*cmd).qentry.tqe_next;
    }
    if !(*(*from).list).tqh_first.is_null() {
        *(*(*cmdlist).list).tqh_last = (*(*from).list).tqh_first;
        (*(*(*from).list).tqh_first).qentry.tqe_prev = (*(*cmdlist).list).tqh_last;
        (*(*cmdlist).list).tqh_last = (*(*from).list).tqh_last;
        (*(*from).list).tqh_first = ::core::ptr::null_mut::<cmd>();
        (*(*from).list).tqh_last = &raw mut (*(*from).list).tqh_first;
    }
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_move(mut cmdlist: *mut cmd_list, mut from: *mut cmd_list) {
    if !(*(*from).list).tqh_first.is_null() {
        *(*(*cmdlist).list).tqh_last = (*(*from).list).tqh_first;
        (*(*(*from).list).tqh_first).qentry.tqe_prev = (*(*cmdlist).list).tqh_last;
        (*(*cmdlist).list).tqh_last = (*(*from).list).tqh_last;
        (*(*from).list).tqh_first = ::core::ptr::null_mut::<cmd>();
        (*(*from).list).tqh_last = &raw mut (*(*from).list).tqh_first;
    }
    let fresh8 = cmd_list_next_group;
    cmd_list_next_group = cmd_list_next_group.wrapping_add(1);
    (*cmdlist).group = fresh8;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_free(mut cmdlist: *mut cmd_list) {
    let mut cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut cmd1: *mut cmd = ::core::ptr::null_mut::<cmd>();
    (*cmdlist).references -= 1;
    if (*cmdlist).references != 0 as ::core::ffi::c_int {
        return;
    }
    cmd = (*(*cmdlist).list).tqh_first;
    while !cmd.is_null() && {
        cmd1 = (*cmd).qentry.tqe_next;
        1 as ::core::ffi::c_int != 0
    } {
        if !(*cmd).qentry.tqe_next.is_null() {
            (*(*cmd).qentry.tqe_next).qentry.tqe_prev = (*cmd).qentry.tqe_prev;
        } else {
            (*(*cmdlist).list).tqh_last = (*cmd).qentry.tqe_prev;
        }
        *(*cmd).qentry.tqe_prev = (*cmd).qentry.tqe_next;
        cmd_free(cmd);
        cmd = cmd1;
    }
    drop(Box::from_raw((*cmdlist).list));
    drop(Box::from_raw(cmdlist));
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_copy(
    mut cmdlist: *const cmd_list,
    mut argc: ::core::ffi::c_int,
    mut argv: *mut *mut ::core::ffi::c_char,
) -> *mut cmd_list {
    let mut cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut new_cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    let mut new_cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut group: u_int = (*cmdlist).group;
    let s = cmd_list_print_cstring(cmdlist, 0);
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_list_copy\0" as *const u8 as *const ::core::ffi::c_char,
        s.as_ptr(),
    );
    new_cmdlist = cmd_list_new();
    cmd = (*(*cmdlist).list).tqh_first;
    while !cmd.is_null() {
        if (*cmd).group != group {
            let fresh7 = cmd_list_next_group;
            cmd_list_next_group = cmd_list_next_group.wrapping_add(1);
            (*new_cmdlist).group = fresh7;
            group = (*cmd).group;
        }
        new_cmd = cmd_copy(cmd, argc, argv);
        cmd_list_append(new_cmdlist, new_cmd);
        cmd = (*cmd).qentry.tqe_next;
    }
    let s = cmd_list_print_cstring(new_cmdlist, 0);
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_list_copy\0" as *const u8 as *const ::core::ffi::c_char,
        s.as_ptr(),
    );
    return new_cmdlist;
}
pub(crate) unsafe fn cmd_list_print_cstring(cmdlist: *const cmd_list, flags: i32) -> CString {
    let mut buf = Vec::new();
    let mut cmd = (*(*cmdlist).list).tqh_first;
    while !cmd.is_null() {
        let this = cmd_print_cstring(cmd);
        buf.extend_from_slice(this.as_bytes());

        let next = (*cmd).qentry.tqe_next;
        if !next.is_null() {
            let grouped = flags & CMD_LIST_PRINT_NO_GROUPS == 0 && (*cmd).group != (*next).group;
            let separator: &[u8] = match (flags & CMD_LIST_PRINT_ESCAPED != 0, grouped) {
                (false, false) => b" ; ",
                (false, true) => b" ;; ",
                (true, false) => b" \\; ",
                (true, true) => b" \\;\\; ",
            };
            buf.extend_from_slice(separator);
        }
        cmd = next;
    }
    // All fragments came from C strings or nonzero literal bytes.
    CString::new(buf).expect("command list contains no interior NUL")
}

#[no_mangle]
pub unsafe extern "C" fn cmd_list_print(
    cmdlist: *const cmd_list,
    flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let printed = cmd_list_print_cstring(cmdlist, flags);
    xstrdup(printed.as_ptr())
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_first(mut cmdlist: *mut cmd_list) -> *mut cmd {
    return (*(*cmdlist).list).tqh_first;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_next(mut cmd: *mut cmd) -> *mut cmd {
    return (*cmd).qentry.tqe_next;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_all_have(
    mut cmdlist: *mut cmd_list,
    mut flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    cmd = (*(*cmdlist).list).tqh_first;
    while !cmd.is_null() {
        if !(*(*cmd).entry).flags & flag != 0 {
            return 0 as ::core::ffi::c_int;
        }
        cmd = (*cmd).qentry.tqe_next;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_list_any_have(
    mut cmdlist: *mut cmd_list,
    mut flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    cmd = (*(*cmdlist).list).tqh_first;
    while !cmd.is_null() {
        if (*(*cmd).entry).flags & flag != 0 {
            return 1 as ::core::ffi::c_int;
        }
        cmd = (*cmd).qentry.tqe_next;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_mouse_at(
    mut wp: *mut window_pane,
    mut m: *mut mouse_event,
    mut xp: *mut u_int,
    mut yp: *mut u_int,
    mut last: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut x: u_int = 0;
    let mut y: u_int = 0;
    if last != 0 {
        x = (*m).lx.wrapping_add((*m).ox);
        y = (*m).ly.wrapping_add((*m).oy);
    } else {
        x = (*m).x.wrapping_add((*m).ox);
        y = (*m).y.wrapping_add((*m).oy);
    }
    log_debug(
        b"%s: x=%u, y=%u%s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_mouse_at\0" as *const u8 as *const ::core::ffi::c_char,
        x,
        y,
        if last != 0 {
            b" (last)\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
    );
    if (*m).statusat == 0 as ::core::ffi::c_int && y >= (*m).statuslines {
        y = y.wrapping_sub((*m).statuslines);
    }
    if (x as ::core::ffi::c_int) < (*wp).xoff
        || x as ::core::ffi::c_int >= (*wp).xoff + (*wp).sx as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    if (y as ::core::ffi::c_int) < (*wp).yoff
        || y as ::core::ffi::c_int >= (*wp).yoff + (*wp).sy as ::core::ffi::c_int
    {
        return -(1 as ::core::ffi::c_int);
    }
    if !xp.is_null() {
        *xp = x.wrapping_sub((*wp).xoff as u_int);
    }
    if !yp.is_null() {
        *yp = y.wrapping_sub((*wp).yoff as u_int);
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_mouse_window(
    mut m: *mut mouse_event,
    mut sp: *mut *mut session,
) -> *mut winlink {
    let mut s: *mut session = ::core::ptr::null_mut::<session>();
    let mut w: *mut window = ::core::ptr::null_mut::<window>();
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    if (*m).valid == 0 {
        return ::core::ptr::null_mut::<winlink>();
    }
    if (*m).s == -(1 as ::core::ffi::c_int) || {
        s = session_find_by_id((*m).s as u_int);
        s.is_null()
    } {
        return ::core::ptr::null_mut::<winlink>();
    }
    if (*m).w == -(1 as ::core::ffi::c_int) {
        wl = (*s).curw;
    } else {
        w = window_find_by_id((*m).w as u_int);
        if w.is_null() {
            return ::core::ptr::null_mut::<winlink>();
        }
        wl = winlink_find_by_window(&raw mut (*s).windows, w);
    }
    if !sp.is_null() {
        *sp = s;
    }
    return wl;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_mouse_pane(
    mut m: *mut mouse_event,
    mut sp: *mut *mut session,
    mut wlp: *mut *mut winlink,
) -> *mut window_pane {
    let mut wl: *mut winlink = ::core::ptr::null_mut::<winlink>();
    let mut wp: *mut window_pane = ::core::ptr::null_mut::<window_pane>();
    wl = cmd_mouse_window(m, sp);
    if wl.is_null() {
        return ::core::ptr::null_mut::<window_pane>();
    }
    if (*m).wp == -(1 as ::core::ffi::c_int) {
        wp = (*(*wl).window).active;
    } else {
        wp = window_pane_find_by_id((*m).wp as u_int);
        if wp.is_null() {
            return ::core::ptr::null_mut::<window_pane>();
        }
        if window_has_pane((*wl).window, wp) == 0 {
            return ::core::ptr::null_mut::<window_pane>();
        }
    }
    if !(*(*wl).window).modal.is_null() && wp != (*(*wl).window).modal {
        return ::core::ptr::null_mut::<window_pane>();
    }
    if !wlp.is_null() {
        *wlp = wl;
    }
    return wp;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_template_replace(
    template: *const ::core::ffi::c_char,
    s: *const ::core::ffi::c_char,
    idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if strchr(template, '%' as i32).is_null() {
        return xstrdup(template);
    }
    let text = cmd_template_replace_cstring(template, s, idx);
    // Callers retain the exported malloc/free contract.
    xstrdup(text.as_ptr())
}

pub(crate) unsafe fn cmd_template_replace_cstring(
    template: *const ::core::ffi::c_char,
    s: *const ::core::ffi::c_char,
    idx: ::core::ffi::c_int,
) -> CString {
    let template_bytes = CStr::from_ptr(template).to_bytes();
    if !template_bytes.contains(&b'%') {
        return CStr::from_ptr(template).to_owned();
    }

    let mut output = Vec::new();
    let mut replaced = false;
    let mut offset = 0;
    while offset < template_bytes.len() {
        let ch = template_bytes[offset];
        offset += 1;
        if ch == b'%' {
            let next = template_bytes.get(offset).copied().unwrap_or(0);
            let quote = if (b'1'..=b'9').contains(&next) && (next - b'0') as i32 == idx {
                offset += 1;
                if template_bytes.get(offset) == Some(&b'%') {
                    offset += 1;
                    Some(DQ)
                } else {
                    Some(NQ)
                }
            } else if next == b'%' && !replaced {
                replaced = true;
                offset += 1;
                if template_bytes.get(offset) == Some(&b'%') {
                    offset += 1;
                    Some(DQ)
                } else {
                    Some(SQ)
                }
            } else {
                None
            };
            if let Some(quote) = quote {
                let replacement = CStr::from_ptr(s).to_bytes();
                if replacement.len() >= usize::MAX / 4
                    || output.len() > usize::MAX - replacement.len() * 4 - 1
                {
                    fatalx(b"argument too long\0".as_ptr().cast());
                }
                output.reserve(replacement.len() * 4);
                for &byte in replacement {
                    if quote == SQ && byte == b'\'' {
                        output.extend_from_slice(b"'\\''");
                    } else {
                        if quote == DQ && b"\"\\$;~".contains(&byte) {
                            output.push(b'\\');
                        }
                        output.push(byte);
                    }
                }
                continue;
            }
        }
        if output.len() > usize::MAX - 2 {
            fatalx(b"argument too long\0".as_ptr().cast());
        }
        output.push(ch);
    }
    let text = CString::new(output).expect("C string template contains no embedded NUL");
    log_debug(
        b"%s: %s -> %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_template_replace\0" as *const u8 as *const ::core::ffi::c_char,
        template,
        text.as_ptr(),
    );
    text
}

#[cfg(test)]
mod template_replace_tests {
    use super::*;

    #[test]
    fn preserves_template_substitution_and_quoting_bytes() {
        let cases: &[(&[u8], &[u8], i32, &[u8])] = &[
            (b"plain\0", b"unused\0", 1, b"plain"),
            (b"A%1B\0", b"\xff\0", 1, b"A\xffB"),
            (b"A%2B\0", b"x\0", 1, b"A%2B"),
            (b"A%%B%%C\0", b"a'b\0", 1, b"Aa'\\''bB%%C"),
            (b"A%1%B\0", b"\"\\$;~\0", 1, b"A\\\"\\\\\\$\\;\\~B"),
        ];
        for &(template, replacement, idx, expected) in cases {
            let actual = unsafe {
                cmd_template_replace_cstring(
                    template.as_ptr().cast(),
                    replacement.as_ptr().cast(),
                    idx,
                )
            };
            assert_eq!(actual.as_bytes(), expected);
        }

        // The public ABI still returns an independently freeable C allocation.
        let raw =
            unsafe { cmd_template_replace(b"%1\0".as_ptr().cast(), b"x\0".as_ptr().cast(), 1) };
        assert_eq!(unsafe { CStr::from_ptr(raw) }.to_bytes(), b"x");
        unsafe { free(raw.cast()) };
    }
}

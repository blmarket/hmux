pub use crate::src::arguments::args_parse;
use crate::src::arguments::ArgsParseError;
use crate::src::arguments::{args_copy, args_escape_cstring, args_free, args_print_cstring};
use crate::src::cmd::entries::attach_session::cmd_attach_session_entry;
use crate::src::cmd::entries::bind_key::cmd_bind_key_entry;
use crate::src::cmd::entries::break_pane::cmd_break_pane_entry;
use crate::src::cmd::entries::capture_pane::{cmd_capture_pane_entry, cmd_clear_history_entry};
use crate::src::cmd::entries::choose_tree::{
    cmd_choose_buffer_entry, cmd_choose_client_entry, cmd_choose_tree_entry,
    cmd_customize_mode_entry, cmd_display_panes_entry, cmd_switch_mode_entry,
};
use crate::src::cmd::entries::command_prompt::cmd_command_prompt_entry;
use crate::src::cmd::entries::confirm_before::cmd_confirm_before_entry;
use crate::src::cmd::entries::copy_mode::{cmd_clock_mode_entry, cmd_copy_mode_entry};
use crate::src::cmd::entries::detach_client::{cmd_detach_client_entry, cmd_suspend_client_entry};
use crate::src::cmd::entries::display_menu::{cmd_display_menu_entry, cmd_display_popup_entry};
use crate::src::cmd::entries::display_message::cmd_display_message_entry;
use crate::src::cmd::entries::find_window::cmd_find_window_entry;
use crate::src::cmd::entries::if_shell::cmd_if_shell_entry;
use crate::src::cmd::entries::join_pane::{cmd_join_pane_entry, cmd_move_pane_entry};
use crate::src::cmd::entries::kill_pane::cmd_kill_pane_entry;
use crate::src::cmd::entries::kill_server::{cmd_kill_server_entry, cmd_start_server_entry};
use crate::src::cmd::entries::kill_session::cmd_kill_session_entry;
use crate::src::cmd::entries::kill_window::{cmd_kill_window_entry, cmd_unlink_window_entry};
use crate::src::cmd::entries::list_buffers::cmd_list_buffers_entry;
use crate::src::cmd::entries::list_clients::cmd_list_clients_entry;
use crate::src::cmd::entries::list_commands::cmd_list_commands_entry;
use crate::src::cmd::entries::list_keys::cmd_list_keys_entry;
use crate::src::cmd::entries::list_panes::cmd_list_panes_entry;
use crate::src::cmd::entries::list_sessions::cmd_list_sessions_entry;
use crate::src::cmd::entries::list_windows::cmd_list_windows_entry;
use crate::src::cmd::entries::load_buffer::cmd_load_buffer_entry;
use crate::src::cmd::entries::lock_server::{
    cmd_lock_client_entry, cmd_lock_server_entry, cmd_lock_session_entry,
};
use crate::src::cmd::entries::move_window::{cmd_link_window_entry, cmd_move_window_entry};
use crate::src::cmd::entries::new_session::{cmd_has_session_entry, cmd_new_session_entry};
use crate::src::cmd::entries::new_window::cmd_new_window_entry;
use crate::src::cmd::entries::paste_buffer::cmd_paste_buffer_entry;
use crate::src::cmd::entries::pipe_pane::cmd_pipe_pane_entry;
use crate::src::cmd::entries::refresh_client::cmd_refresh_client_entry;
use crate::src::cmd::entries::rename_session::cmd_rename_session_entry;
use crate::src::cmd::entries::rename_window::cmd_rename_window_entry;
use crate::src::cmd::entries::resize_pane::cmd_resize_pane_entry;
use crate::src::cmd::entries::resize_window::cmd_resize_window_entry;
use crate::src::cmd::entries::respawn_pane::cmd_respawn_pane_entry;
use crate::src::cmd::entries::respawn_window::cmd_respawn_window_entry;
use crate::src::cmd::entries::rotate_window::cmd_rotate_window_entry;
use crate::src::cmd::entries::run_shell::cmd_run_shell_entry;
use crate::src::cmd::entries::save_buffer::{cmd_save_buffer_entry, cmd_show_buffer_entry};
use crate::src::cmd::entries::select_layout::{
    cmd_next_layout_entry, cmd_previous_layout_entry, cmd_select_layout_entry,
};
use crate::src::cmd::entries::select_pane::{cmd_last_pane_entry, cmd_select_pane_entry};
use crate::src::cmd::entries::select_window::{
    cmd_last_window_entry, cmd_next_window_entry, cmd_previous_window_entry,
    cmd_select_window_entry,
};
use crate::src::cmd::entries::send_keys::{cmd_send_keys_entry, cmd_send_prefix_entry};
use crate::src::cmd::entries::server_access::cmd_server_access_entry;
use crate::src::cmd::entries::set_buffer::{cmd_delete_buffer_entry, cmd_set_buffer_entry};
use crate::src::cmd::entries::set_environment::cmd_set_environment_entry;
use crate::src::cmd::entries::set_option::{
    cmd_set_hook_entry, cmd_set_option_entry, cmd_set_window_option_entry,
};
use crate::src::cmd::entries::show_environment::cmd_show_environment_entry;
use crate::src::cmd::entries::show_messages::cmd_show_messages_entry;
use crate::src::cmd::entries::show_options::{
    cmd_show_hooks_entry, cmd_show_options_entry, cmd_show_window_options_entry,
};
use crate::src::cmd::entries::show_prompt_history::{
    cmd_clear_prompt_history_entry, cmd_show_prompt_history_entry,
};
use crate::src::cmd::entries::source_file::cmd_source_file_entry;
use crate::src::cmd::entries::split_window::{cmd_new_pane_entry, cmd_split_window_entry};
use crate::src::cmd::entries::swap_pane::cmd_swap_pane_entry;
use crate::src::cmd::entries::swap_window::cmd_swap_window_entry;
use crate::src::cmd::entries::switch_client::cmd_switch_client_entry;
use crate::src::cmd::entries::unbind_key::cmd_unbind_key_entry;
use crate::src::cmd::entries::wait_for::cmd_wait_for_entry;
use crate::src::ffi::libc::{strchr, strcmp, strlcat, strlcpy, strlen, strncmp};
use crate::src::log::{fatalx, log_debug};
use crate::src::options::{
    options_array_first, options_array_item_value, options_array_next, options_get_only,
};
use crate::src::session::session_find_by_id;
use crate::src::shared::abi::*;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{args, args_value};
pub use crate::src::shared::command::{cmd, cmd_entry, cmd_list, cmdq_item};
pub use crate::src::shared::command::{CMD_LIST_PRINT_ESCAPED, CMD_LIST_PRINT_NO_GROUPS};
pub use crate::src::shared::environment::environ;
pub use crate::src::shared::format::{format_job_tree, format_tree};
pub use crate::src::shared::key::key_event;
pub use crate::src::shared::layout::layout_cell;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::{options, options_array_item, options_entry, options_value};
pub use crate::src::shared::pane::window_pane;
pub use crate::src::shared::session::session;
pub use crate::src::shared::tty::tty_term;
pub use crate::src::shared::window::{window, winlink};
use crate::src::tmux::global_options;
use crate::src::window::{
    window_find_by_id, window_has_pane, window_pane_find_by_id, winlink_find_by_window,
};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::{Mutex, OnceLock};

// `cmd_list_next` receives only an element pointer, so keep its owner and index
// in a side table rather than putting neighbor links back in `cmd`.
fn cmd_list_memberships() -> &'static Mutex<HashMap<usize, (usize, usize)>> {
    static MEMBERSHIPS: OnceLock<Mutex<HashMap<usize, (usize, usize)>>> = OnceLock::new();
    MEMBERSHIPS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub const DQ: C2RustUnnamed_38 = 2;
pub type C2RustUnnamed_38 = ::core::ffi::c_uint;
pub const SQ: C2RustUnnamed_38 = 1;
pub const NQ: C2RustUnnamed_38 = 0;
pub static mut cmd_table: [*const cmd_entry; 93] =  {
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
pub unsafe fn cmd_log_argv(argv: &Vec<CString>, prefix: &CStr) {
    for (i, arg) in argv.iter().enumerate() {
        log_debug(
            b"%s: argv[%d]=%s\0" as *const u8 as *const ::core::ffi::c_char,
            prefix.as_ptr(),
            i as ::core::ffi::c_int,
            arg.as_ptr(),
        );
    }
}
pub(crate) fn cmd_append_argv(argv: &mut Vec<CString>, arg: &CStr) {
    argv.push(arg.to_owned());
}
pub unsafe fn cmd_pack_argv(
    argv: &Vec<CString>,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut arglen: size_t = 0;
    let mut i: ::core::ffi::c_int = 0;
    if argv.is_empty() {
        return 0 as ::core::ffi::c_int;
    }
    cmd_log_argv(argv, c"cmd_pack_argv");
    *buf = '\0' as i32 as ::core::ffi::c_char;
    i = 0 as ::core::ffi::c_int;
    while (i as usize) < argv.len() {
        if strlcpy(buf, argv[i as usize].as_ptr(), len) as size_t >= len {
            return -(1 as ::core::ffi::c_int);
        }
        arglen = argv[i as usize].as_bytes_with_nul().len() as size_t;
        buf = buf.offset(arglen as isize);
        len = len.wrapping_sub(arglen);
        i += 1;
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn cmd_stringify_argv(argv: &Vec<CString>) -> Option<CString> {
    cmd_stringify_argv_cstring(argv)
}

pub(crate) unsafe fn cmd_stringify_argv_cstring(argv: &Vec<CString>) -> Option<CString> {
    let mut bytes = Vec::new();
    for (i, argument) in argv.iter().enumerate() {
        let escaped = args_escape_cstring(argument);
        log_debug(
            b"%s: %u %s = %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmd_stringify_argv\0" as *const u8 as *const ::core::ffi::c_char,
            i,
            argument.as_ptr(),
            escaped.as_ptr(),
        );
        if i != 0 {
            bytes.push(b' ');
        }
        bytes.extend_from_slice(escaped.as_bytes());
    }
    Some(CString::new(bytes).expect("escaped argv contains no interior NUL"))
}
pub unsafe fn cmd_get_entry(mut cmd: *mut cmd) -> *const cmd_entry {
    return (*cmd).entry;
}
pub unsafe fn cmd_get_args(mut cmd: *mut cmd) -> *mut args {
    return (*cmd).args;
}
pub unsafe fn cmd_get_group(mut cmd: *mut cmd) -> u_int {
    return (*cmd).group;
}
pub unsafe fn cmd_get_source(
    mut cmd: *mut cmd,
    mut file: *mut *const ::core::ffi::c_char,
    mut line: *mut u_int,
) {
    if !file.is_null() {
        *file = ((*cmd).file)
            .as_ref()
            .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
    }
    if !line.is_null() {
        *line = (*cmd).line;
    }
}
pub unsafe fn cmd_get_parse_flags(mut cmd: *mut cmd) -> ::core::ffi::c_int {
    return (*cmd).parse_flags;
}
pub unsafe fn cmd_get_alias(name: &CStr) -> Option<CString> {
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
        return None;
    }
    wanted = name.to_bytes().len();
    a = options_array_first(o);
    while !a.is_null() {
        ov = options_array_item_value(a);
        equals = strchr((*ov).string_ptr(), '=' as i32);
        if !equals.is_null() {
            n = equals.offset_from((*ov).string_ptr()) as ::core::ffi::c_long as size_t;
            if n == wanted
                && strncmp(name.as_ptr(), (*ov).string_ptr(), n) == 0 as ::core::ffi::c_int
            {
                return Some(
                    CStr::from_ptr(equals.offset(1 as ::core::ffi::c_int as isize)).to_owned(),
                );
            }
        }
        a = options_array_next(a);
    }
    None
}
pub unsafe fn cmd_find(name: &CStr) -> Result<*const cmd_entry, CString> {
    let mut loop_0: *mut *const cmd_entry = ::core::ptr::null_mut::<*const cmd_entry>();
    let mut entry: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
    let mut found: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
    let mut ambiguous: ::core::ffi::c_int = 0;
    let mut s: [::core::ffi::c_char; 8192] = [0; 8192];
    ambiguous = 0 as ::core::ffi::c_int;
    loop_0 = &raw mut cmd_table as *mut *const cmd_entry;
    while !(*loop_0).is_null() {
        entry = *loop_0;
        if !(*entry).alias.is_null()
            && strcmp((*entry).alias, name.as_ptr()) == 0 as ::core::ffi::c_int
        {
            ambiguous = 0 as ::core::ffi::c_int;
            found = entry;
            break;
        } else {
            if !(strncmp((*entry).name, name.as_ptr(), name.to_bytes().len())
                != 0 as ::core::ffi::c_int)
            {
                if !found.is_null() {
                    ambiguous = 1 as ::core::ffi::c_int;
                }
                found = entry;
                if strcmp((*entry).name, name.as_ptr()) == 0 as ::core::ffi::c_int {
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
            if !(strncmp((*entry).name, name.as_ptr(), name.to_bytes().len())
                != 0 as ::core::ffi::c_int)
            {
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
        let mut error = b"ambiguous command: ".to_vec();
        error.extend_from_slice(name.to_bytes());
        error.extend_from_slice(b", could be: ");
        error.extend_from_slice(CStr::from_ptr(s.as_ptr()).to_bytes());
        return Err(CString::new(error).expect("command diagnostic contains no NUL"));
    } else {
        if found.is_null() {
            let mut error = b"unknown command: ".to_vec();
            error.extend_from_slice(name.to_bytes());
            return Err(CString::new(error).expect("command diagnostic contains no NUL"));
        }
        return Ok(found);
    };
}

unsafe fn cmd_new_owned(file: Option<&CStr>) -> *mut cmd {
    let mut owner = Box::new(cmd {
        file: file.map(CStr::to_owned),
        ..cmd::empty()
    });

    Box::into_raw(owner).cast::<cmd>()
}

pub unsafe fn cmd_parse(
    mut values: *mut args_value,
    mut count: u_int,
    file: Option<&CStr>,
    mut line: u_int,
    mut parse_flags: ::core::ffi::c_int,
) -> Result<*mut cmd, CString> {
    let mut entry: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
    let mut cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut args: *mut args = ::core::ptr::null_mut::<args>();
    if count == 0 as u_int
        || (*values.offset(0 as ::core::ffi::c_int as isize)).type_0() as ::core::ffi::c_uint
            != ARGS_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return Err(CString::new("no command").unwrap());
    }
    entry = cmd_find(CStr::from_ptr(
        (*values.offset(0 as ::core::ffi::c_int as isize)).string_ptr(),
    ))?;
    args = match args_parse(&raw const (*entry).args, values, count) {
        Ok(args) => args,
        Err(ArgsParseError::Usage) => {
            let mut error = b"usage: ".to_vec();
            error.extend_from_slice(CStr::from_ptr((*entry).name).to_bytes());
            error.push(b' ');
            error.extend_from_slice(CStr::from_ptr((*entry).usage).to_bytes());
            return Err(CString::new(error).expect("command diagnostic contains no NUL"));
        }
        Err(ArgsParseError::Message(message)) => {
            let mut error = b"command ".to_vec();
            error.extend_from_slice(CStr::from_ptr((*entry).name).to_bytes());
            error.extend_from_slice(b": ");
            error.extend_from_slice(message.as_bytes());
            return Err(CString::new(error).expect("command diagnostic contains no NUL"));
        }
    };
    cmd = cmd_new_owned(file);
    (*cmd).entry = entry;
    (*cmd).args = args;
    (*cmd).parse_flags = parse_flags;
    (*cmd).line = line;
    return Ok(cmd);
}
pub unsafe fn cmd_free(mut cmd: *mut cmd) {
    args_free((*cmd).args);
    drop(Box::from_raw(cmd));
}
pub unsafe fn cmd_copy(cmd: &cmd, argv: &Vec<CString>) -> *mut cmd {
    let mut new_cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    new_cmd = cmd_new_owned(cmd.file.as_deref());
    (*new_cmd).entry = cmd.entry;
    (*new_cmd).args = args_copy(cmd.args, argv);
    (*new_cmd).line = cmd.line;
    return new_cmd;
}
pub unsafe fn cmd_print(cmd: &cmd) -> CString {
    cmd_print_cstring(cmd)
}

pub(crate) unsafe fn cmd_print_cstring(cmd: &cmd) -> CString {
    let args = args_print_cstring(cmd.args);
    let arguments = args.as_bytes();
    let name = CStr::from_ptr((*cmd.entry).name).to_bytes();
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
pub unsafe fn cmd_list_new() -> *mut cmd_list {
    let mut cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    cmdlist = Box::into_raw(Box::new(cmd_list::default()));
    (*cmdlist).references = 1 as ::core::ffi::c_int;
    let fresh6 = cmd_list_next_group;
    cmd_list_next_group = cmd_list_next_group.wrapping_add(1);
    (*cmdlist).group = fresh6;
    (*cmdlist).list = Box::into_raw(Box::new(Vec::<*mut cmd>::new()));
    return cmdlist;
}
pub unsafe fn cmd_list_append(mut cmdlist: *mut cmd_list, mut cmd: *mut cmd) {
    (*cmd).group = (*cmdlist).group;
    let commands = &mut *(*cmdlist).list;
    let mut memberships = cmd_list_memberships()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    memberships.insert(cmd as usize, (cmdlist as usize, commands.len()));
    commands.push(cmd);
}
pub unsafe fn cmd_list_append_all(mut cmdlist: *mut cmd_list, mut from: *mut cmd_list) {
    if cmdlist == from {
        return;
    }
    let destination = &mut *(*cmdlist).list;
    let source = &mut *(*from).list;
    let mut memberships = cmd_list_memberships()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let offset = destination.len();
    for (index, &cmd) in source.iter().enumerate() {
        (*cmd).group = (*cmdlist).group;
        memberships.insert(cmd as usize, (cmdlist as usize, offset + index));
    }
    destination.append(source);
}
pub unsafe fn cmd_list_move(mut cmdlist: *mut cmd_list, mut from: *mut cmd_list) {
    if cmdlist != from {
        let destination = &mut *(*cmdlist).list;
        let source = &mut *(*from).list;
        let mut memberships = cmd_list_memberships()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let offset = destination.len();
        for (index, &cmd) in source.iter().enumerate() {
            memberships.insert(cmd as usize, (cmdlist as usize, offset + index));
        }
        destination.append(source);
    }
    let fresh8 = cmd_list_next_group;
    cmd_list_next_group = cmd_list_next_group.wrapping_add(1);
    (*cmdlist).group = fresh8;
}
pub unsafe fn cmd_list_free(mut cmdlist: *mut cmd_list) {
    (*cmdlist).references -= 1;
    if (*cmdlist).references != 0 as ::core::ffi::c_int {
        return;
    }
    let commands = std::mem::take(&mut *(*cmdlist).list);
    {
        let mut memberships = cmd_list_memberships()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for &cmd in &commands {
            memberships.remove(&(cmd as usize));
        }
    }
    for cmd in commands {
        cmd_free(cmd);
    }
    drop(Box::from_raw((*cmdlist).list));
    drop(Box::from_raw(cmdlist));
}
pub unsafe fn cmd_list_copy(cmdlist: &cmd_list, argv: &Vec<CString>) -> *mut cmd_list {
    let mut new_cmdlist: *mut cmd_list = ::core::ptr::null_mut::<cmd_list>();
    let mut new_cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut group: u_int = cmdlist.group;
    let s = cmd_list_print_cstring(cmdlist, 0);
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_list_copy\0" as *const u8 as *const ::core::ffi::c_char,
        s.as_ptr(),
    );
    new_cmdlist = cmd_list_new();
    for &cmd in &*cmdlist.list {
        if (*cmd).group != group {
            let fresh7 = cmd_list_next_group;
            cmd_list_next_group = cmd_list_next_group.wrapping_add(1);
            (*new_cmdlist).group = fresh7;
            group = (*cmd).group;
        }
        new_cmd = cmd_copy(&*cmd, argv);
        cmd_list_append(new_cmdlist, new_cmd);
    }
    let s = cmd_list_print_cstring(&*new_cmdlist, 0);
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmd_list_copy\0" as *const u8 as *const ::core::ffi::c_char,
        s.as_ptr(),
    );
    return new_cmdlist;
}
pub(crate) unsafe fn cmd_list_print_cstring(cmdlist: &cmd_list, flags: i32) -> CString {
    let mut buf = Vec::new();
    let commands = &*cmdlist.list;
    for (index, &cmd) in commands.iter().enumerate() {
        let this = cmd_print_cstring(&*cmd);
        buf.extend_from_slice(this.as_bytes());

        if let Some(&next) = commands.get(index + 1) {
            let grouped = flags & CMD_LIST_PRINT_NO_GROUPS == 0 && (*cmd).group != (*next).group;
            let separator: &[u8] = match (flags & CMD_LIST_PRINT_ESCAPED != 0, grouped) {
                (false, false) => b" ; ",
                (false, true) => b" ;; ",
                (true, false) => b" \\; ",
                (true, true) => b" \\;\\; ",
            };
            buf.extend_from_slice(separator);
        }
    }
    // All fragments came from C strings or nonzero literal bytes.
    CString::new(buf).expect("command list contains no interior NUL")
}

pub unsafe fn cmd_list_print(cmdlist: &cmd_list, flags: ::core::ffi::c_int) -> CString {
    cmd_list_print_cstring(cmdlist, flags)
}
pub unsafe fn cmd_list_first(mut cmdlist: *mut cmd_list) -> *mut cmd {
    return (*(*cmdlist).list)
        .first()
        .copied()
        .unwrap_or(::core::ptr::null_mut());
}
pub unsafe fn cmd_list_next(mut cmd: *mut cmd) -> *mut cmd {
    if cmd.is_null() {
        return ::core::ptr::null_mut();
    }
    let memberships = cmd_list_memberships()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(&(cmdlist, index)) = memberships.get(&(cmd as usize)) {
        let commands = &*(*(cmdlist as *mut cmd_list)).list;
        return commands
            .get(index + 1)
            .copied()
            .unwrap_or(::core::ptr::null_mut());
    }
    return ::core::ptr::null_mut();
}
pub unsafe fn cmd_list_all_have(
    mut cmdlist: *mut cmd_list,
    mut flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    for &cmd in &*(*cmdlist).list {
        if !(*(*cmd).entry).flags & flag != 0 {
            return 0 as ::core::ffi::c_int;
        }
    }
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn cmd_list_any_have(
    mut cmdlist: *mut cmd_list,
    mut flag: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    for &cmd in &*(*cmdlist).list {
        if (*(*cmd).entry).flags & flag != 0 {
            return 1 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn cmd_mouse_at(
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
pub unsafe fn cmd_mouse_window(mut m: *mut mouse_event, mut sp: *mut *mut session) -> *mut winlink {
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
pub unsafe fn cmd_mouse_pane(
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
pub unsafe fn cmd_template_replace(template: &CStr, s: &CStr, idx: ::core::ffi::c_int) -> CString {
    cmd_template_replace_cstring(template, s, idx)
}

pub(crate) unsafe fn cmd_template_replace_cstring(
    template: &CStr,
    s: &CStr,
    idx: ::core::ffi::c_int,
) -> CString {
    let template_bytes = template.to_bytes();
    if !template_bytes.contains(&b'%') {
        return template.to_owned();
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
                let replacement = s.to_bytes();
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
        template.as_ptr(),
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
                    CStr::from_bytes_with_nul(template).unwrap(),
                    CStr::from_bytes_with_nul(replacement).unwrap(),
                    idx,
                )
            };
            assert_eq!(actual.as_bytes(), expected);
        }

        let owned = unsafe {
            cmd_template_replace(
                CStr::from_bytes_with_nul(b"%1\0").unwrap(),
                CStr::from_bytes_with_nul(b"x\0").unwrap(),
                1,
            )
        };
        assert_eq!(owned.as_bytes(), b"x");
    }
}

use crate::src::arguments::{
    args_count, args_first, args_first_value, args_get, args_next, args_next_value, args_print,
    args_string,
};
use crate::src::cfg::{cfg_add_cause, cfg_finished};
use crate::src::cmd::{
    cmd_get_args, cmd_get_entry, cmd_get_group, cmd_get_source, cmd_list_first, cmd_list_free,
    cmd_list_next, cmd_print_cstring,
};
use crate::src::cmd_find::{
    cmd_find_clear_state, cmd_find_client, cmd_find_copy_state, cmd_find_from_client,
    cmd_find_target, cmd_find_valid_state,
};
use crate::src::control::{control_write, control_write_guard};
use crate::src::events::events_fire;
use crate::src::events_payload::{
    event_payload_create, event_payload_set_pointer, event_payload_set_string,
    event_payload_set_target,
};
use crate::src::ffi::libc::{__ctype_toupper_loc, free, getpwuid, getuid, memcpy, time};
use crate::src::file::file_error;
use crate::src::format::{format_add, format_create, format_free, format_merge};
use crate::src::key_string::key_string_format;
use crate::src::log::{fatalx, log_debug, log_get_level};
use crate::src::proc::proc_get_peer_uid;
use crate::src::reactor::{evbuffer_add_vprintf, evbuffer_free, evbuffer_new};
use crate::src::server::server_add_message;
use crate::src::server_client::{server_client_print, server_client_unref};
use crate::src::shared::abi::*;
pub use crate::src::shared::abi::{__gid_t, __int32_t, __uid_t, uid_t};
pub use crate::src::shared::account::passwd;
use crate::src::shared::arguments::*;
pub use crate::src::shared::arguments::{
    args, args_entry, args_parse, args_parse_cb, args_value, args_value_c2rust_unnamed,
    args_value_entry,
};
use crate::src::shared::client::*;
pub use crate::src::shared::client::{
    client, client_entry, client_file, client_file_cb, client_file_entry, client_files,
    overlay_check_cb, overlay_draw_cb, overlay_free_cb, overlay_key_cb, overlay_mode_cb,
    overlay_resize_cb,
};
pub use crate::src::shared::client::{CLIENT_CONTROL, CLIENT_UTF8};
use crate::src::shared::colour::*;
use crate::src::shared::command::*;
pub use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_cb, cmdq_item, cmdq_item_entry,
    cmdq_item_list, cmdq_list, cmdq_state, cmdq_type, cmds,
};
pub use crate::src::shared::command::{
    CMDQ_FIRED, CMDQ_STATE_CONTROL, CMDQ_STATE_NOHOOKS, CMDQ_WAITING, CMD_AFTERHOOK,
    CMD_CLIENT_CANFAIL, CMD_CLIENT_CFLAG, CMD_CLIENT_TFLAG,
};
pub use crate::src::shared::control::control_state;
use crate::src::shared::display::*;
pub use crate::src::shared::display::{visible_range, visible_ranges};
pub use crate::src::shared::environment::environ;
use crate::src::shared::event::*;
pub use crate::src::shared::events::{
    event_payload, event_payload_free_cb, event_payload_print_cb,
};
pub use crate::src::shared::format::FORMAT_NONE;
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
pub use crate::src::shared::menu::menu_data;
use crate::src::shared::message::*;
pub use crate::src::shared::mouse::mouse_event;
pub use crate::src::shared::options::options;
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
use crate::src::status::status_message_set;
use crate::src::utf8::utf8_sanitize;
use crate::src::xmalloc::{xasprintf, xcalloc, xsnprintf, xstrdup, xvasprintf_cstring};
use std::ffi::{CStr, CString};

pub use crate::src::shared::grid::grid_cell_entry_data as C2RustUnnamed_13;
pub use crate::src::shared::grid::grid_cell_entry_storage as C2RustUnnamed_12;

pub const CMDQ_CALLBACK: cmdq_type = 1;
pub const CMDQ_COMMAND: cmdq_type = 0;

pub use crate::src::shared::key::key_code_enum as C2RustUnnamed_36;

unsafe extern "C" fn cmdq_name(mut c: *mut client) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 256] = [0; 256];
    if c.is_null() {
        return b"<global>\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if !(*c).name.is_null() {
        xsnprintf(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
            b"<%s>\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
        );
    } else {
        xsnprintf(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
            b"<%p>\0" as *const u8 as *const ::core::ffi::c_char,
            c,
        );
    }
    return &raw mut s as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn cmdq_get(mut c: *mut client) -> *mut cmdq_list {
    static mut global_queue: *mut cmdq_list = ::core::ptr::null::<cmdq_list>() as *mut cmdq_list;
    if c.is_null() {
        if global_queue.is_null() {
            global_queue = cmdq_new();
        }
        return global_queue;
    }
    return (*c).queue;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_new() -> *mut cmdq_list {
    let mut queue: *mut cmdq_list = ::core::ptr::null_mut::<cmdq_list>();
    queue = xcalloc(1 as size_t, ::core::mem::size_of::<cmdq_list>() as size_t) as *mut cmdq_list;
    (*queue).list.tqh_first = ::core::ptr::null_mut::<cmdq_item>();
    (*queue).list.tqh_last = &raw mut (*queue).list.tqh_first;
    return queue;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_free(mut queue: *mut cmdq_list) {
    if !(*queue).list.tqh_first.is_null() {
        fatalx(b"queue not empty\0" as *const u8 as *const ::core::ffi::c_char);
    }
    free(queue as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_name(mut item: *mut cmdq_item) -> *const ::core::ffi::c_char {
    return (*item).name;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_cmd(mut item: *mut cmdq_item) -> *mut cmd {
    return (*item).cmd;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_client(mut item: *mut cmdq_item) -> *mut client {
    return (*item).client;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_target_client(mut item: *mut cmdq_item) -> *mut client {
    return (*item).target_client;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_state(mut item: *mut cmdq_item) -> *mut cmdq_state {
    return (*item).state;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_target(mut item: *mut cmdq_item) -> *mut cmd_find_state {
    return &raw mut (*item).target;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_source(mut item: *mut cmdq_item) -> *mut cmd_find_state {
    return &raw mut (*item).source;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_event(mut item: *mut cmdq_item) -> *mut key_event {
    return &raw mut (*(*item).state).event;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_current(mut item: *mut cmdq_item) -> *mut cmd_find_state {
    return &raw mut (*(*item).state).current;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_flags(mut item: *mut cmdq_item) -> ::core::ffi::c_int {
    return (*(*item).state).flags;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_new_state(
    mut current: *mut cmd_find_state,
    mut event: *mut key_event,
    mut flags: ::core::ffi::c_int,
) -> *mut cmdq_state {
    let mut state: *mut cmdq_state = ::core::ptr::null_mut::<cmdq_state>();
    state = xcalloc(1 as size_t, ::core::mem::size_of::<cmdq_state>() as size_t) as *mut cmdq_state;
    (*state).references = 1 as ::core::ffi::c_int;
    (*state).flags = flags;
    if !event.is_null() {
        memcpy(
            &raw mut (*state).event as *mut ::core::ffi::c_void,
            event as *const ::core::ffi::c_void,
            ::core::mem::size_of::<key_event>() as size_t,
        );
    } else {
        (*state).event.key = KEYC_NONE as ::core::ffi::c_ulong as key_code;
    }
    if !current.is_null() && cmd_find_valid_state(current) != 0 {
        cmd_find_copy_state(&raw mut (*state).current, current);
    } else {
        cmd_find_clear_state(&raw mut (*state).current, 0 as ::core::ffi::c_int);
    }
    return state;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_link_state(mut state: *mut cmdq_state) -> *mut cmdq_state {
    (*state).references += 1;
    return state;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_copy_state(
    mut state: *mut cmdq_state,
    mut current: *mut cmd_find_state,
) -> *mut cmdq_state {
    if !current.is_null() {
        return cmdq_new_state(current, &raw mut (*state).event, (*state).flags);
    }
    return cmdq_new_state(
        &raw mut (*state).current,
        &raw mut (*state).event,
        (*state).flags,
    );
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_free_state(mut state: *mut cmdq_state) {
    (*state).references -= 1;
    if (*state).references != 0 as ::core::ffi::c_int {
        return;
    }
    if !(*state).formats.is_null() {
        format_free((*state).formats);
    }
    free(state as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_add_format(
    mut state: *mut cmdq_state,
    mut key: *const ::core::ffi::c_char,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    ap = args.clone();
    let value = xvasprintf_cstring(fmt, ap);
    if (*state).formats.is_null() {
        (*state).formats = format_create(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<cmdq_item>(),
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        );
    }
    format_add(
        (*state).formats,
        key,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        value.as_ptr(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_add_formats(mut state: *mut cmdq_state, mut ft: *mut format_tree) {
    if (*state).formats.is_null() {
        (*state).formats = format_create(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<cmdq_item>(),
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        );
    }
    format_merge((*state).formats, ft);
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_merge_formats(mut item: *mut cmdq_item, mut ft: *mut format_tree) {
    let mut entry: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
    if !(*item).cmd.is_null() {
        entry = cmd_get_entry((*item).cmd);
        format_add(
            ft,
            b"command\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*entry).name,
        );
    }
    if !(*(*item).state).formats.is_null() {
        format_merge(ft, (*(*item).state).formats);
    }
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_append(
    mut c: *mut client,
    mut item: *mut cmdq_item,
) -> *mut cmdq_item {
    let mut queue: *mut cmdq_list = cmdq_get(c);
    let mut next: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    loop {
        next = (*item).next;
        (*item).next = ::core::ptr::null_mut::<cmdq_item>();
        if !c.is_null() {
            (*c).references += 1;
        }
        (*item).client = c;
        (*item).queue = queue;
        (*item).entry.tqe_next = ::core::ptr::null_mut::<cmdq_item>();
        (*item).entry.tqe_prev = (*queue).list.tqh_last;
        *(*queue).list.tqh_last = item;
        (*queue).list.tqh_last = &raw mut (*item).entry.tqe_next;
        log_debug(
            b"%s %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_append\0" as *const u8 as *const ::core::ffi::c_char,
            cmdq_name(c),
            (*item).name,
        );
        item = next;
        if item.is_null() {
            break;
        }
    }
    return *(*((*queue).list.tqh_last as *mut cmdq_item_list)).tqh_last;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_insert_after(
    mut after: *mut cmdq_item,
    mut item: *mut cmdq_item,
) -> *mut cmdq_item {
    let mut c: *mut client = (*after).client;
    let mut queue: *mut cmdq_list = (*after).queue;
    let mut next: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    loop {
        next = (*item).next;
        (*item).next = (*after).next;
        (*after).next = item;
        if !c.is_null() {
            (*c).references += 1;
        }
        (*item).client = c;
        (*item).queue = queue;
        (*item).entry.tqe_next = (*after).entry.tqe_next;
        if !(*item).entry.tqe_next.is_null() {
            (*(*item).entry.tqe_next).entry.tqe_prev = &raw mut (*item).entry.tqe_next;
        } else {
            (*queue).list.tqh_last = &raw mut (*item).entry.tqe_next;
        }
        (*after).entry.tqe_next = item;
        (*item).entry.tqe_prev = &raw mut (*after).entry.tqe_next;
        log_debug(
            b"%s %s: %s after %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_insert_after\0" as *const u8 as *const ::core::ffi::c_char,
            cmdq_name(c),
            (*item).name,
            (*after).name,
        );
        after = item;
        item = next;
        if item.is_null() {
            break;
        }
    }
    return after;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_insert_hook(
    mut s: *mut session,
    mut item: *mut cmdq_item,
    mut current: *mut cmd_find_state,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut cmd: *mut cmd = (*item).cmd;
    let mut args_0: *mut args = cmd_get_args(cmd);
    let mut ae: *mut args_entry = ::core::ptr::null_mut::<args_entry>();
    let mut av: *mut args_value = ::core::ptr::null_mut::<args_value>();
    let mut ep: *mut event_payload = ::core::ptr::null_mut::<event_payload>();
    let mut ap: ::core::ffi::VaList;
    let mut tmp: [::core::ffi::c_char; 32] = [0; 32];
    let mut flag: ::core::ffi::c_char = 0;
    let mut arguments: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: u_int = 0;
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*(*item).state).flags & CMDQ_STATE_NOHOOKS != 0 {
        return;
    }
    ap = args.clone();
    let name = xvasprintf_cstring(fmt, ap);
    ep = event_payload_create();
    if !current.is_null() {
        event_payload_set_target(ep, current);
    }
    event_payload_set_pointer(
        ep,
        b"_cmdq_item\0" as *const u8 as *const ::core::ffi::c_char,
        item as *mut ::core::ffi::c_void,
        None,
        None,
    );
    arguments = args_print(args_0);
    event_payload_set_string(
        ep,
        b"arguments\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        arguments,
    );
    free(arguments as *mut ::core::ffi::c_void);
    i = 0 as u_int;
    while i < args_count(args_0) {
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"argument_%u\0" as *const u8 as *const ::core::ffi::c_char,
            i,
        );
        event_payload_set_string(
            ep,
            &raw mut tmp as *mut ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            args_string(args_0, i),
        );
        i = i.wrapping_add(1);
    }
    flag = args_first(args_0, &raw mut ae) as ::core::ffi::c_char;
    while flag as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        value = args_get(args_0, flag as u_char);
        xsnprintf(
            &raw mut tmp as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
            b"flag_%c\0" as *const u8 as *const ::core::ffi::c_char,
            flag as ::core::ffi::c_int,
        );
        if value.is_null() {
            event_payload_set_string(
                ep,
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"1\0" as *const u8 as *const ::core::ffi::c_char,
            );
        } else {
            event_payload_set_string(
                ep,
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                value,
            );
        }
        i = 0 as u_int;
        av = args_first_value(args_0, flag as u_char);
        while !av.is_null() {
            xsnprintf(
                &raw mut tmp as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>() as size_t,
                b"flag_%c_%u\0" as *const u8 as *const ::core::ffi::c_char,
                flag as ::core::ffi::c_int,
                i,
            );
            event_payload_set_string(
                ep,
                &raw mut tmp as *mut ::core::ffi::c_char,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                (*av).c2rust_unnamed.string,
            );
            i = i.wrapping_add(1);
            av = args_next_value(av);
        }
        flag = args_next(&raw mut ae) as ::core::ffi::c_char;
    }
    events_fire(name.as_ptr(), ep);
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_continue(mut item: *mut cmdq_item) {
    (*item).flags &= !CMDQ_WAITING;
}
unsafe extern "C" fn cmdq_remove(mut item: *mut cmdq_item) {
    if !(*item).client.is_null() {
        server_client_unref((*item).client);
    }
    if !(*item).cmdlist.is_null() {
        cmd_list_free((*item).cmdlist);
    }
    cmdq_free_state((*item).state);
    if !(*item).entry.tqe_next.is_null() {
        (*(*item).entry.tqe_next).entry.tqe_prev = (*item).entry.tqe_prev;
    } else {
        (*(*item).queue).list.tqh_last = (*item).entry.tqe_prev;
    }
    *(*item).entry.tqe_prev = (*item).entry.tqe_next;
    free((*item).name as *mut ::core::ffi::c_void);
    free(item as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn cmdq_remove_group(mut item: *mut cmdq_item) {
    let mut this: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut next: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    if (*item).group == 0 as u_int {
        return;
    }
    this = (*item).entry.tqe_next;
    while !this.is_null() {
        next = (*this).entry.tqe_next;
        if (*this).group == (*item).group {
            cmdq_remove(this);
        }
        this = next;
    }
}
unsafe extern "C" fn cmdq_empty_command(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    return CMD_RETURN_NORMAL;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_command(
    mut cmdlist: *mut cmd_list,
    mut state: *mut cmdq_state,
) -> *mut cmdq_item {
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut first: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut last: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut cmd: *mut cmd = ::core::ptr::null_mut::<cmd>();
    let mut entry: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
    let mut created: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    cmd = cmd_list_first(cmdlist);
    if cmd.is_null() {
        return cmdq_get_callback1(
            b"cmdq_empty_command\0" as *const u8 as *const ::core::ffi::c_char,
            Some(
                cmdq_empty_command
                    as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
            ),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        );
    }
    if state.is_null() {
        state = cmdq_new_state(
            ::core::ptr::null_mut::<cmd_find_state>(),
            ::core::ptr::null_mut::<key_event>(),
            0 as ::core::ffi::c_int,
        );
        created = 1 as ::core::ffi::c_int;
    }
    while !cmd.is_null() {
        entry = cmd_get_entry(cmd);
        item =
            xcalloc(1 as size_t, ::core::mem::size_of::<cmdq_item>() as size_t) as *mut cmdq_item;
        xasprintf(
            &raw mut (*item).name,
            b"[%s/%p]\0" as *const u8 as *const ::core::ffi::c_char,
            (*entry).name,
            item,
        );
        (*item).type_0 = CMDQ_COMMAND;
        (*item).group = cmd_get_group(cmd);
        (*item).state = cmdq_link_state(state);
        (*item).cmdlist = cmdlist;
        (*item).cmd = cmd;
        (*cmdlist).references += 1;
        log_debug(
            b"%s: %s group %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_get_command\0" as *const u8 as *const ::core::ffi::c_char,
            (*item).name,
            (*item).group,
        );
        if first.is_null() {
            first = item;
        }
        if !last.is_null() {
            (*last).next = item;
        }
        last = item;
        cmd = cmd_list_next(cmd);
    }
    if created != 0 {
        cmdq_free_state(state);
    }
    return first;
}
unsafe extern "C" fn cmdq_find_flag(
    mut item: *mut cmdq_item,
    mut fs: *mut cmd_find_state,
    mut flag: *const cmd_entry_flag,
) -> cmd_retval {
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*flag).flag as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        cmd_find_from_client(fs, (*item).target_client, 0 as ::core::ffi::c_int);
        return CMD_RETURN_NORMAL;
    }
    value = args_get(cmd_get_args((*item).cmd), (*flag).flag as u_char);
    if cmd_find_target(fs, item, value, (*flag).type_0, (*flag).flags) != 0 as ::core::ffi::c_int {
        cmd_find_clear_state(fs, 0 as ::core::ffi::c_int);
        return CMD_RETURN_ERROR;
    }
    return CMD_RETURN_NORMAL;
}
unsafe extern "C" fn cmdq_add_message(mut item: *mut cmdq_item) {
    let mut c: *mut client = (*item).client;
    let mut state: *mut cmdq_state = (*item).state;
    let mut uid: uid_t = 0;
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    let tmp = cmd_print_cstring((*item).cmd);
    if !c.is_null() {
        uid = proc_get_peer_uid((*c).peer);
        let user: CString = if uid != -(1 as ::core::ffi::c_int) as uid_t && uid != getuid() {
            pw = getpwuid(uid as __uid_t);
            if !pw.is_null() {
                let name = CStr::from_ptr((*pw).pw_name).to_bytes();
                let mut bytes = Vec::with_capacity(name.len() + 2);
                bytes.push(b'[');
                bytes.extend_from_slice(name);
                bytes.push(b']');
                CString::new(bytes).expect("passwd name is a C string")
            } else {
                c"[unknown]".to_owned()
            }
        } else {
            c"".to_owned()
        };
        if !(*c).session.is_null()
            && (*state).event.key != KEYC_NONE as ::core::ffi::c_ulong as key_code
        {
            let key = key_string_format((*state).event.key, false);
            server_add_message(
                b"%s%s key %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
                user.as_ptr(),
                key.as_ptr(),
                tmp.as_ptr(),
            );
        } else {
            server_add_message(
                b"%s%s command: %s\0" as *const u8 as *const ::core::ffi::c_char,
                (*c).name,
                user.as_ptr(),
                tmp.as_ptr(),
            );
        }
    } else {
        server_add_message(
            b"command: %s\0" as *const u8 as *const ::core::ffi::c_char,
            tmp.as_ptr(),
        );
    }
}
unsafe extern "C" fn cmdq_fire_command(mut item: *mut cmdq_item) -> cmd_retval {
    let mut current_block: u64;
    let mut name: *const ::core::ffi::c_char = cmdq_name((*item).client);
    let mut state: *mut cmdq_state = (*item).state;
    let mut cmd: *mut cmd = (*item).cmd;
    let mut args: *mut args = cmd_get_args(cmd);
    let mut entry: *const cmd_entry = cmd_get_entry(cmd);
    let mut tc: *mut client = ::core::ptr::null_mut::<client>();
    let mut saved: *mut client = (*item).client;
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    let mut fsp: *mut cmd_find_state = ::core::ptr::null_mut::<cmd_find_state>();
    let mut fs: cmd_find_state = cmd_find_state {
        flags: 0,
        current: ::core::ptr::null_mut::<cmd_find_state>(),
        s: ::core::ptr::null_mut::<session>(),
        wl: ::core::ptr::null_mut::<winlink>(),
        w: ::core::ptr::null_mut::<window>(),
        wp: ::core::ptr::null_mut::<window_pane>(),
        idx: 0,
    };
    let mut flags: ::core::ffi::c_int = 0;
    let mut quiet: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if cfg_finished != 0 {
        cmdq_add_message(item);
    }
    if log_get_level() > 1 as ::core::ffi::c_int {
        let tmp = cmd_print_cstring(cmd);
        log_debug(
            b"%s %s: (%u) %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_fire_command\0" as *const u8 as *const ::core::ffi::c_char,
            name,
            (*item).group,
            tmp.as_ptr(),
        );
    }
    flags = ((*state).flags & CMDQ_STATE_CONTROL != 0) as ::core::ffi::c_int;
    cmdq_guard(
        item,
        b"begin\0" as *const u8 as *const ::core::ffi::c_char,
        flags,
    );
    if (*item).client.is_null() {
        (*item).client = cmd_find_client(
            item,
            ::core::ptr::null::<::core::ffi::c_char>(),
            1 as ::core::ffi::c_int,
        );
    }
    if (*entry).flags & CMD_CLIENT_CANFAIL != 0 {
        quiet = 1 as ::core::ffi::c_int;
    }
    if (*entry).flags & CMD_CLIENT_CFLAG != 0 {
        tc = cmd_find_client(item, args_get(args, 'c' as i32 as u_char), quiet);
        if tc.is_null() && quiet == 0 {
            retval = CMD_RETURN_ERROR;
            current_block = 7379054416801212160;
        } else {
            current_block = 18317007320854588510;
        }
    } else if (*entry).flags & CMD_CLIENT_TFLAG != 0 {
        tc = cmd_find_client(item, args_get(args, 't' as i32 as u_char), quiet);
        if tc.is_null() && quiet == 0 {
            retval = CMD_RETURN_ERROR;
            current_block = 7379054416801212160;
        } else {
            current_block = 18317007320854588510;
        }
    } else {
        tc = cmd_find_client(
            item,
            ::core::ptr::null::<::core::ffi::c_char>(),
            1 as ::core::ffi::c_int,
        );
        current_block = 18317007320854588510;
    }
    match current_block {
        18317007320854588510 => {
            (*item).target_client = tc;
            retval = cmdq_find_flag(item, &raw mut (*item).source, &raw const (*entry).source);
            if !(retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int) {
                retval = cmdq_find_flag(item, &raw mut (*item).target, &raw const (*entry).target);
                if !(retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int) {
                    retval = (*entry).exec.expect("non-null function pointer")(cmd, item);
                    if !(retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int) {
                        if (*entry).flags & CMD_AFTERHOOK != 0 {
                            if cmd_find_valid_state(&raw mut (*item).target) != 0 {
                                fsp = &raw mut (*item).target;
                                current_block = 8704759739624374314;
                            } else if cmd_find_valid_state(&raw mut (*(*item).state).current) != 0 {
                                fsp = &raw mut (*(*item).state).current;
                                current_block = 8704759739624374314;
                            } else if cmd_find_from_client(
                                &raw mut fs,
                                (*item).client,
                                0 as ::core::ffi::c_int,
                            ) == 0 as ::core::ffi::c_int
                            {
                                fsp = &raw mut fs;
                                current_block = 8704759739624374314;
                            } else {
                                current_block = 7379054416801212160;
                            }
                            match current_block {
                                7379054416801212160 => {}
                                _ => {
                                    cmdq_insert_hook(
                                        (*fsp).s,
                                        item,
                                        fsp,
                                        b"after-%s\0" as *const u8 as *const ::core::ffi::c_char,
                                        (*entry).name,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    (*item).client = saved;
    if retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int {
        fsp = ::core::ptr::null_mut::<cmd_find_state>();
        if cmd_find_valid_state(&raw mut (*item).target) != 0 {
            fsp = &raw mut (*item).target;
        } else if cmd_find_valid_state(&raw mut (*(*item).state).current) != 0 {
            fsp = &raw mut (*(*item).state).current;
        } else if cmd_find_from_client(&raw mut fs, (*item).client, 0 as ::core::ffi::c_int)
            == 0 as ::core::ffi::c_int
        {
            fsp = &raw mut fs;
        }
        cmdq_insert_hook(
            if !fsp.is_null() {
                (*fsp).s
            } else {
                ::core::ptr::null_mut::<session>()
            },
            item,
            fsp,
            b"command-error\0" as *const u8 as *const ::core::ffi::c_char,
        );
        cmdq_guard(
            item,
            b"error\0" as *const u8 as *const ::core::ffi::c_char,
            flags,
        );
    } else {
        cmdq_guard(
            item,
            b"end\0" as *const u8 as *const ::core::ffi::c_char,
            flags,
        );
    }
    return retval;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_callback1(
    mut name: *const ::core::ffi::c_char,
    mut cb: cmdq_cb,
    mut data: *mut ::core::ffi::c_void,
) -> *mut cmdq_item {
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    item = xcalloc(1 as size_t, ::core::mem::size_of::<cmdq_item>() as size_t) as *mut cmdq_item;
    xasprintf(
        &raw mut (*item).name,
        b"[%s/%p]\0" as *const u8 as *const ::core::ffi::c_char,
        name,
        item,
    );
    (*item).type_0 = CMDQ_CALLBACK;
    (*item).group = 0 as u_int;
    (*item).state = cmdq_new_state(
        ::core::ptr::null_mut::<cmd_find_state>(),
        ::core::ptr::null_mut::<key_event>(),
        0 as ::core::ffi::c_int,
    );
    (*item).cb = cb;
    (*item).data = data;
    return item;
}
unsafe extern "C" fn cmdq_error_callback(
    mut item: *mut cmdq_item,
    mut data: *mut ::core::ffi::c_void,
) -> cmd_retval {
    let mut error: *mut ::core::ffi::c_char = data as *mut ::core::ffi::c_char;
    cmdq_error(
        item,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        error,
    );
    free(error as *mut ::core::ffi::c_void);
    return CMD_RETURN_NORMAL;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_get_error(mut error: *const ::core::ffi::c_char) -> *mut cmdq_item {
    return cmdq_get_callback1(
        b"cmdq_error_callback\0" as *const u8 as *const ::core::ffi::c_char,
        Some(
            cmdq_error_callback
                as unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval,
        ),
        xstrdup(error) as *mut ::core::ffi::c_void,
    );
}
unsafe extern "C" fn cmdq_fire_callback(mut item: *mut cmdq_item) -> cmd_retval {
    return (*item).cb.expect("non-null function pointer")(item, (*item).data);
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_next(mut c: *mut client) -> u_int {
    let mut current_block: u64;
    let mut queue: *mut cmdq_list = cmdq_get(c);
    let mut name: *const ::core::ffi::c_char = cmdq_name(c);
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    let mut items: u_int = 0 as u_int;
    static mut number: u_int = 0;
    if (*queue).list.tqh_first.is_null() {
        log_debug(
            b"%s %s: empty\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_next\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
        return 0 as u_int;
    }
    if (*(*queue).list.tqh_first).flags & CMDQ_WAITING != 0 {
        log_debug(
            b"%s %s: waiting\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_next\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
        return 0 as u_int;
    }
    log_debug(
        b"%s %s: enter\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmdq_next\0" as *const u8 as *const ::core::ffi::c_char,
        name,
    );
    loop {
        (*queue).item = (*queue).list.tqh_first;
        item = (*queue).item;
        if item.is_null() {
            current_block = 7056779235015430508;
            break;
        }
        log_debug(
            b"%s %s: %s (%d), flags %x\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_next\0" as *const u8 as *const ::core::ffi::c_char,
            name,
            (*item).name,
            (*item).type_0 as ::core::ffi::c_uint,
            (*item).flags,
        );
        if (*item).flags & CMDQ_WAITING != 0 {
            current_block = 8107417685847451382;
            break;
        }
        if !(*item).flags & CMDQ_FIRED != 0 {
            (*item).time = time(::core::ptr::null_mut::<time_t>());
            number = number.wrapping_add(1);
            (*item).number = number;
            match (*item).type_0 as ::core::ffi::c_uint {
                0 => {
                    retval = cmdq_fire_command(item);
                    if retval as ::core::ffi::c_int == CMD_RETURN_ERROR as ::core::ffi::c_int {
                        cmdq_remove_group(item);
                    }
                }
                1 => {
                    retval = cmdq_fire_callback(item);
                }
                _ => {
                    retval = CMD_RETURN_ERROR;
                }
            }
            (*item).flags |= CMDQ_FIRED;
            if retval as ::core::ffi::c_int == CMD_RETURN_WAIT as ::core::ffi::c_int {
                (*item).flags |= CMDQ_WAITING;
                current_block = 8107417685847451382;
                break;
            } else {
                items = items.wrapping_add(1);
            }
        }
        cmdq_remove(item);
    }
    match current_block {
        8107417685847451382 => {
            log_debug(
                b"%s %s: exit (wait)\0" as *const u8 as *const ::core::ffi::c_char,
                b"cmdq_next\0" as *const u8 as *const ::core::ffi::c_char,
                name,
            );
            return items;
        }
        _ => {
            (*queue).item = ::core::ptr::null_mut::<cmdq_item>();
            log_debug(
                b"%s %s: exit (empty)\0" as *const u8 as *const ::core::ffi::c_char,
                b"cmdq_next\0" as *const u8 as *const ::core::ffi::c_char,
                name,
            );
            return items;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_running(mut c: *mut client) -> *mut cmdq_item {
    let mut queue: *mut cmdq_list = cmdq_get(c);
    if (*queue).item.is_null() {
        return ::core::ptr::null_mut::<cmdq_item>();
    }
    if (*(*queue).item).flags & CMDQ_WAITING != 0 {
        return ::core::ptr::null_mut::<cmdq_item>();
    }
    return (*queue).item;
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_guard(
    mut item: *mut cmdq_item,
    mut guard: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) {
    let mut c: *mut client = (*item).client;
    let mut t: ::core::ffi::c_long = (*item).time as ::core::ffi::c_long;
    let mut number: u_int = (*item).number;
    if !c.is_null() && (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        control_write_guard(c, guard, t, number, flags);
    }
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_print_data(mut item: *mut cmdq_item, mut evb: *mut evbuffer) {
    server_client_print((*item).client, 1 as ::core::ffi::c_int, evb);
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_print(
    mut item: *mut cmdq_item,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut ap: ::core::ffi::VaList;
    let mut evb: *mut evbuffer = ::core::ptr::null_mut::<evbuffer>();
    evb = evbuffer_new();
    if evb.is_null() {
        fatalx(b"out of memory\0" as *const u8 as *const ::core::ffi::c_char);
    }
    ap = args.clone();
    evbuffer_add_vprintf(evb, fmt, ap);
    cmdq_print_data(item, evb);
    evbuffer_free(evb);
}
#[no_mangle]
pub unsafe extern "C" fn cmdq_error(
    mut item: *mut cmdq_item,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) {
    let mut c: *mut client = (*item).client;
    let mut cmd: *mut cmd = (*item).cmd;
    let mut ap: ::core::ffi::VaList;
    let mut file: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut line: u_int = 0;
    ap = args.clone();
    let mut msg = xvasprintf_cstring(fmt, ap);
    log_debug(
        b"%s: %s\0" as *const u8 as *const ::core::ffi::c_char,
        b"cmdq_error\0" as *const u8 as *const ::core::ffi::c_char,
        msg.as_ptr(),
    );
    if c.is_null() {
        cmd_get_source(cmd, &raw mut file, &raw mut line);
        if cfg_finished == 0 {
            if !file.is_null() {
                cfg_add_cause(
                    b"%s:%u: %s\0" as *const u8 as *const ::core::ffi::c_char,
                    file,
                    line,
                    msg.as_ptr(),
                );
            } else {
                cfg_add_cause(
                    b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                    msg.as_ptr(),
                );
            }
        } else if !file.is_null() {
            server_add_message(
                b"message: %s:%u: %s\0" as *const u8 as *const ::core::ffi::c_char,
                file,
                line,
                msg.as_ptr(),
            );
        } else {
            server_add_message(
                b"message: %s\0" as *const u8 as *const ::core::ffi::c_char,
                msg.as_ptr(),
            );
        }
    } else if (*c).session.is_null() || (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
        server_add_message(
            b"%s message: %s\0" as *const u8 as *const ::core::ffi::c_char,
            (*c).name,
            msg.as_ptr(),
        );
        if !(*c).flags & CLIENT_UTF8 as uint64_t != 0 {
            let sanitized = utf8_sanitize(msg.as_ptr());
            msg = CStr::from_ptr(sanitized).to_owned();
            free(sanitized as *mut ::core::ffi::c_void);
        }
        if (*c).flags & CLIENT_CONTROL as uint64_t != 0 {
            control_write(
                c,
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                msg.as_ptr(),
            );
        } else {
            file_error(
                c,
                b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                msg.as_ptr(),
            );
        }
        (*c).retval = 1 as ::core::ffi::c_int;
    } else {
        let mut bytes = msg.into_bytes_with_nul();
        bytes[0] = *(*__ctype_toupper_loc()).offset(bytes[0] as isize) as u8;
        // The bytes came from a CString; toupper maps NUL to NUL and a
        // non-NUL byte to a non-NUL byte, so the terminator stays at the end.
        msg = CString::from_vec_with_nul_unchecked(bytes);
        status_message_set(
            c,
            -(1 as ::core::ffi::c_int),
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            msg.as_ptr(),
        );
    }
}

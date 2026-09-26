use crate::src::arguments::{
    args_count, args_first, args_first_value, args_get, args_next, args_next_value,
    args_print_cstring, args_string,
};
use crate::src::cfg::{cfg_add_cause, cfg_finished};
use crate::src::cmd::find::{
    cmd_find_clear_state, cmd_find_client, cmd_find_copy_state, cmd_find_from_client,
    cmd_find_target, cmd_find_valid_state,
};
use crate::src::cmd::{
    cmd_get_args, cmd_get_entry, cmd_get_group, cmd_get_source, cmd_list_first, cmd_list_free,
    cmd_list_next, cmd_print_cstring,
};
use crate::src::control::{control_write, control_write_guard};
use crate::src::events::events_fire;
use crate::src::events_payload::{
    event_payload_create, event_payload_set_pointer, event_payload_set_string,
    event_payload_set_target,
};
use crate::src::ffi::libc::{__ctype_toupper_loc, getpwuid, getuid, time};
use crate::src::file::{file_cancel_cmdq_wait, file_error};
use crate::src::format::{format_add, format_add_cstr, format_create, format_free, format_merge};
use crate::src::key_string::key_string_format;
use crate::src::log::{fatalx, log_debug, log_get_level};
use crate::src::proc::proc_get_peer_uid;
use crate::src::reactor::{evbuffer_add_vprintf, evbuffer_free, evbuffer_new};
use crate::src::server::server_add_message;
use crate::src::server_client::{server_client_print, server_client_unref};
use crate::src::shared::abi::*;
use crate::src::shared::abi::{__uid_t, uid_t};
use crate::src::shared::account::passwd;
use crate::src::shared::arguments::{args, args_entry, args_value};
use crate::src::shared::client::{client, client_file};
use crate::src::shared::client::{CLIENT_CONTROL, CLIENT_UTF8};
use crate::src::shared::command::*;
use crate::src::shared::command::{
    cmd, cmd_entry, cmd_entry_flag, cmd_find_state, cmd_list, cmdq_cb, cmdq_item, cmdq_list,
    cmdq_state, cmdq_type,
};
use crate::src::shared::command::{
    CMDQ_FIRED, CMDQ_STATE_CONTROL, CMDQ_STATE_NOHOOKS, CMDQ_WAITING, CMD_AFTERHOOK,
    CMD_CLIENT_CANFAIL, CMD_CLIENT_CFLAG, CMD_CLIENT_TFLAG,
};
use crate::src::shared::event::*;
use crate::src::shared::events::event_payload;
use crate::src::shared::format::format_tree;
use crate::src::shared::format::FORMAT_NONE;
use crate::src::shared::key::key_event;
use crate::src::shared::key::*;
use crate::src::shared::pane::window_pane;
use crate::src::shared::session::session;
use crate::src::shared::window::{window, winlink};
use crate::src::status::status_message_set;
use crate::src::text::utf8::utf8_sanitize_cstring;
use crate::src::xmalloc::{xsnprintf, xvasprintf_cstring};
use std::ffi::{CStr, CString};

pub const CMDQ_CALLBACK: cmdq_type = 1;
pub const CMDQ_COMMAND: cmdq_type = 0;

unsafe fn cmdq_new_named_item(label: Option<&CStr>) -> *mut cmdq_item {
    let mut owner = Box::new(cmdq_item {
        name: None,
        cancel_data: None,
        wait_file: ::core::ptr::null_mut(),
        ..cmdq_item::empty()
    });
    let item = &raw mut *owner;
    let label = label.map_or(b"(null)".as_slice(), CStr::to_bytes);
    let address = format!("{item:p}");
    let mut bytes = Vec::with_capacity(label.len() + address.len() + 3);
    bytes.push(b'[');
    bytes.extend_from_slice(label);
    bytes.push(b'/');
    bytes.extend_from_slice(address.as_bytes());
    bytes.push(b']');
    (*item).name = Some(CString::new(bytes).expect("queue item label has no NUL"));

    // The caller owns this detached allocation until enqueue or explicit free.
    Box::into_raw(owner)
}

/// Register cleanup to run when an unfired callback item is removed.
pub(crate) fn cmdq_set_cancel_callback(item: &mut cmdq_item, cancel: Box<dyn FnOnce()>) {
    assert_eq!(item.type_0, CMDQ_CALLBACK);
    assert!(
        item.cancel_data.is_none(),
        "callback cancel hook already set"
    );
    item.cancel_data = Some(cancel);
}

unsafe fn cmdq_cancel_unfired_data(item: &mut cmdq_item) {
    if item.flags & CMDQ_FIRED != 0 {
        return;
    }
    if let Some(cancel) = item.cancel_data.take() {
        cancel();
    }
}

/// The file remains live until its terminal event, including when a local
/// file operation schedules immediate completion. The queue item only borrows
/// it while the command is waiting.
pub(crate) fn cmdq_set_wait_file(item: &mut cmdq_item, cf: *mut client_file) {
    assert!(!cf.is_null());
    assert!(
        item.wait_file.is_null(),
        "queue item already owns a file wait"
    );
    item.wait_file = cf;
}

pub(crate) fn cmdq_clear_wait_file(item: &mut cmdq_item, cf: *mut client_file) {
    if item.wait_file == cf {
        item.wait_file = ::core::ptr::null_mut();
    }
}

/// A dead client cannot resume a file-backed waiting command. Cancel its
/// callback data first, then remove the waiting item and its queued suffix.
/// Other wait families need their own cancellation before they can be drained.
pub(crate) unsafe fn cmdq_abort_file_wait(c: *mut client) {
    let queue = (*c).queue;
    let first = (*queue).first_ptr();
    if first.is_null() || (*first).flags & CMDQ_WAITING == 0 {
        return;
    }
    let cf = (*first).wait_file;
    if cf.is_null() {
        return;
    }
    file_cancel_cmdq_wait(cf);
    (*queue).item = ::core::ptr::null_mut();
    while !(*queue).list.is_empty() {
        cmdq_remove((*queue).first_ptr());
    }
}

/// Release an item that has not been linked into a command queue.
pub unsafe fn cmdq_free_detached(item: *mut cmdq_item) {
    let mut owner = Box::from_raw(item);
    cmdq_cancel_unfired_data(&mut *owner);
    if !(*item).client.is_null() {
        server_client_unref((*item).client);
    }
    if !(*item).cmdlist.is_null() {
        cmd_list_free((*item).cmdlist);
    }
    cmdq_free_state((*item).state);
    drop(owner);
}

unsafe fn cmdq_name(mut c: *mut client) -> *const ::core::ffi::c_char {
    static mut s: [::core::ffi::c_char; 256] = [0; 256];
    if c.is_null() {
        return b"<global>\0" as *const u8 as *const ::core::ffi::c_char;
    }
    if !(*c).name.is_none() {
        xsnprintf(
            &raw mut s as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 256]>() as size_t,
            b"<%s>\0" as *const u8 as *const ::core::ffi::c_char,
            ((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
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
unsafe fn cmdq_get(mut c: *mut client) -> *mut cmdq_list {
    static mut global_queue: *mut cmdq_list = ::core::ptr::null::<cmdq_list>() as *mut cmdq_list;
    if c.is_null() {
        if global_queue.is_null() {
            global_queue = cmdq_new();
        }
        return global_queue;
    }
    return (*c).queue;
}
pub unsafe fn cmdq_new() -> *mut cmdq_list {
    Box::into_raw(Box::new(cmdq_list {
        item: std::ptr::null_mut(),
        list: std::collections::VecDeque::new(),
    }))
}
pub unsafe fn cmdq_free(mut queue: *mut cmdq_list) {
    if !(*queue).list.is_empty() {
        fatalx(b"queue not empty\0" as *const u8 as *const ::core::ffi::c_char);
    }
    drop(Box::from_raw(queue));
}
pub unsafe fn cmdq_get_name(mut item: *mut cmdq_item) -> *const ::core::ffi::c_char {
    return ((*item).name)
        .as_ref()
        .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut());
}
pub unsafe fn cmdq_get_cmd(mut item: *mut cmdq_item) -> *mut cmd {
    return (*item).cmd;
}
pub unsafe fn cmdq_get_client(mut item: *mut cmdq_item) -> *mut client {
    return (*item).client;
}
pub unsafe fn cmdq_get_target_client(mut item: *mut cmdq_item) -> *mut client {
    return (*item).target_client;
}
pub unsafe fn cmdq_get_state(mut item: *mut cmdq_item) -> *mut cmdq_state {
    return (*item).state;
}
pub unsafe fn cmdq_get_target(mut item: *mut cmdq_item) -> *mut cmd_find_state {
    return &raw mut (*item).target;
}
pub unsafe fn cmdq_get_source(mut item: *mut cmdq_item) -> *mut cmd_find_state {
    return &raw mut (*item).source;
}
pub unsafe fn cmdq_get_event(mut item: *mut cmdq_item) -> *mut key_event {
    return &raw mut (*(*item).state).event;
}
pub unsafe fn cmdq_get_current(mut item: *mut cmdq_item) -> *mut cmd_find_state {
    return &raw mut (*(*item).state).current;
}
pub unsafe fn cmdq_get_flags(mut item: *mut cmdq_item) -> ::core::ffi::c_int {
    return (*(*item).state).flags;
}
pub unsafe fn cmdq_new_state(
    mut current: *mut cmd_find_state,
    mut event: *mut key_event,
    mut flags: ::core::ffi::c_int,
) -> *mut cmdq_state {
    let snapshot = if event.is_null() {
        key_event {
            client: ::core::ptr::null_mut(),
            key: KEYC_NONE as key_code,
            m: Default::default(),
            bytes: None,
        }
    } else {
        (*event).metadata_snapshot()
    };
    let state = crate::src::shared::rc::new(cmdq_state {
        flags,
        formats: ::core::ptr::null_mut(),
        event: snapshot,
        current: Default::default(),
    });
    if !current.is_null() && cmd_find_valid_state(current) != 0 {
        cmd_find_copy_state(&raw mut (*state).current, current);
    } else {
        cmd_find_clear_state(&raw mut (*state).current, 0 as ::core::ffi::c_int);
    }
    return state;
}
pub unsafe fn cmdq_link_state(mut state: *mut cmdq_state) -> *mut cmdq_state {
    crate::src::shared::rc::retain(state);
    return state;
}
pub unsafe fn cmdq_copy_state(
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
pub unsafe fn cmdq_free_state(mut state: *mut cmdq_state) {
    crate::src::shared::rc::release(state);
}
unsafe fn cmdq_destroy_state(state: *mut cmdq_state) {
    if !(*state).formats.is_null() {
        format_free((*state).formats);
    }
}
/// Copy a literal key and value into the command queue state's formats.
pub unsafe fn cmdq_add_format(state: *mut cmdq_state, key: &CStr, value: &CStr) {
    if (*state).formats.is_null() {
        (*state).formats = format_create(
            ::core::ptr::null_mut::<client>(),
            ::core::ptr::null_mut::<cmdq_item>(),
            FORMAT_NONE,
            0 as ::core::ffi::c_int,
        );
    }
    format_add_cstr((*state).formats, key, value);
}
pub unsafe fn cmdq_add_formats(mut state: *mut cmdq_state, mut ft: *mut format_tree) {
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
pub unsafe fn cmdq_merge_formats(mut item: *mut cmdq_item, mut ft: *mut format_tree) {
    let mut entry: *const cmd_entry = ::core::ptr::null::<cmd_entry>();
    if !(*item).cmd.is_null() {
        entry = cmd_get_entry((*item).cmd);
        format_add(
            ft,
            b"command\0" as *const u8 as *const ::core::ffi::c_char,
            b"%s\0" as *const u8 as *const ::core::ffi::c_char,
            (*entry).name.as_ptr(),
        );
    }
    if !(*(*item).state).formats.is_null() {
        format_merge(ft, (*(*item).state).formats);
    }
}
pub unsafe fn cmdq_append(mut c: *mut client, mut item: *mut cmdq_item) -> *mut cmdq_item {
    let mut queue: *mut cmdq_list = cmdq_get(c);
    let mut next: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    loop {
        next = (*item).next;
        (*item).next = ::core::ptr::null_mut::<cmdq_item>();
        if !c.is_null() {
            crate::src::shared::rc::retain(c);
        }
        (*item).client = c;
        (*item).queue = queue;
        // Enqueue consumes the detached allocation without moving the item.
        (*queue).list.push_back(Box::from_raw(item));
        log_debug(
            b"%s %s: %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_append\0" as *const u8 as *const ::core::ffi::c_char,
            cmdq_name(c),
            ((*item).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        );
        item = next;
        if item.is_null() {
            break;
        }
    }
    return std::ptr::from_ref(&**(*queue).list.back().expect("appended command item")).cast_mut();
}
pub unsafe fn cmdq_insert_after(
    mut after: *mut cmdq_item,
    mut item: *mut cmdq_item,
) -> *mut cmdq_item {
    let mut c: *mut client = (*after).client;
    let mut queue: *mut cmdq_list = (*after).queue;
    let mut next: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut position = (*queue).position(after) + 1;
    loop {
        next = (*item).next;
        (*item).next = (*after).next;
        (*after).next = item;
        if !c.is_null() {
            crate::src::shared::rc::retain(c);
        }
        (*item).client = c;
        (*item).queue = queue;
        // Enqueue consumes the detached allocation without moving the item.
        (*queue).list.insert(position, Box::from_raw(item));
        position += 1;
        log_debug(
            b"%s %s: %s after %s\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_insert_after\0" as *const u8 as *const ::core::ffi::c_char,
            cmdq_name(c),
            ((*item).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            ((*after).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
        );
        after = item;
        item = next;
        if item.is_null() {
            break;
        }
    }
    return after;
}
pub unsafe extern "C" fn cmdq_insert_hook(
    _s: *mut session,
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
        crate::src::shared::events::EventPayloadPointer::Raw(item as *mut ::core::ffi::c_void),
    );
    let arguments = args_print_cstring(args_0);
    event_payload_set_string(
        ep,
        b"arguments\0" as *const u8 as *const ::core::ffi::c_char,
        b"%s\0" as *const u8 as *const ::core::ffi::c_char,
        arguments.as_ptr(),
    );
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
                (*av).string_ptr(),
            );
            i = i.wrapping_add(1);
            av = args_next_value(av);
        }
        flag = args_next(&raw mut ae) as ::core::ffi::c_char;
    }
    events_fire(name.as_ptr(), ep);
}
pub unsafe fn cmdq_continue(mut item: *mut cmdq_item) {
    (*item).flags &= !CMDQ_WAITING;
}
unsafe fn cmdq_remove(mut item: *mut cmdq_item) {
    assert!(
        (*item).wait_file.is_null(),
        "file wait must finish or cancel before queue item removal"
    );
    cmdq_cancel_unfired_data(&mut *item);
    if !(*item).client.is_null() {
        server_client_unref((*item).client);
    }
    if !(*item).cmdlist.is_null() {
        cmd_list_free((*item).cmdlist);
    }
    cmdq_free_state((*item).state);
    let queue = (*item).queue;
    let position = (*queue).position(item);
    let owner = (*queue).list.remove(position).expect("queued command item");
    if (*queue).item == item {
        (*queue).item = std::ptr::null_mut();
    }
    drop(owner);
}
unsafe fn cmdq_remove_group(mut item: *mut cmdq_item) {
    if (*item).group == 0 as u_int {
        return;
    }
    let queue = (*item).queue;
    let mut position = (*queue).position(item) + 1;
    while let Some(owner) = (*queue).list.get(position) {
        let this = std::ptr::from_ref(&**owner).cast_mut();
        if (*this).group == (*item).group {
            cmdq_remove(this);
        } else {
            position += 1;
        }
    }
}
pub unsafe fn cmdq_get_command(
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
        return cmdq_get_callback_owned(
            b"cmdq_empty_command\0" as *const u8 as *const ::core::ffi::c_char,
            Some(Box::new(|_| CMD_RETURN_NORMAL)),
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
        item = cmdq_new_named_item(Some((*entry).name));
        (*item).type_0 = CMDQ_COMMAND;
        (*item).group = cmd_get_group(cmd);
        (*item).state = cmdq_link_state(state);
        (*item).cmdlist = cmdlist;
        (*item).cmd = cmd;
        crate::src::shared::rc::retain(cmdlist);
        log_debug(
            b"%s: %s group %u\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_get_command\0" as *const u8 as *const ::core::ffi::c_char,
            ((*item).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
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
unsafe fn cmdq_find_flag(
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
unsafe fn cmdq_add_message(mut item: *mut cmdq_item) {
    let mut c: *mut client = (*item).client;
    let mut state: *mut cmdq_state = (*item).state;
    let mut uid: uid_t = 0;
    let mut pw: *mut passwd = ::core::ptr::null_mut::<passwd>();
    let tmp = cmd_print_cstring(&*(*item).cmd);
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
                ((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
                user.as_ptr(),
                key.as_ptr(),
                tmp.as_ptr(),
            );
        } else {
            server_add_message(
                b"%s%s command: %s\0" as *const u8 as *const ::core::ffi::c_char,
                ((*c).name)
                    .as_ref()
                    .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
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
unsafe fn cmdq_fire_command(mut item: *mut cmdq_item) -> cmd_retval {
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
        let tmp = cmd_print_cstring(&*cmd);
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
                                        (*entry).name.as_ptr(),
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
pub unsafe fn cmdq_get_callback_owned(
    mut name: *const ::core::ffi::c_char,
    mut cb: cmdq_cb,
) -> *mut cmdq_item {
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    item = cmdq_new_named_item(if name.is_null() {
        None
    } else {
        Some(CStr::from_ptr(name))
    });
    (*item).type_0 = CMDQ_CALLBACK;
    (*item).group = 0 as u_int;
    (*item).state = cmdq_new_state(
        ::core::ptr::null_mut::<cmd_find_state>(),
        ::core::ptr::null_mut::<key_event>(),
        0 as ::core::ffi::c_int,
    );
    (*item).cb = cb;
    return item;
}

/// Compatibility adapter for callers that still provide an ABI callback.
pub unsafe fn cmdq_get_callback1(
    name: *const ::core::ffi::c_char,
    cb: Option<unsafe extern "C" fn(*mut cmdq_item, *mut ::core::ffi::c_void) -> cmd_retval>,
    data: *mut ::core::ffi::c_void,
) -> *mut cmdq_item {
    let callback = cb.map(|callback| {
        Box::new(move |item: std::ptr::NonNull<cmdq_item>| unsafe { callback(item.as_ptr(), data) })
            as Box<dyn FnOnce(std::ptr::NonNull<cmdq_item>) -> cmd_retval>
    });
    let item = cmdq_get_callback_owned(name, callback);
    (*item).data = data;
    item
}
pub unsafe fn cmdq_get_error(mut error: *const ::core::ffi::c_char) -> *mut cmdq_item {
    let error = CStr::from_ptr(error).to_owned();
    let data = error.as_ptr().cast_mut().cast();
    let item = cmdq_get_callback_owned(
        b"cmdq_error_callback\0" as *const u8 as *const ::core::ffi::c_char,
        Some(Box::new(move |item| unsafe {
            cmdq_error(
                item.as_ptr(),
                b"%s\0" as *const u8 as *const ::core::ffi::c_char,
                error.as_ptr(),
            );
            CMD_RETURN_NORMAL
        })),
    );
    (*item).data = data;
    item
}
unsafe fn cmdq_fire_callback(mut item: *mut cmdq_item) -> cmd_retval {
    (*item).flags |= CMDQ_FIRED;
    return (*item).cb.take().expect("non-null queue callback")(
        std::ptr::NonNull::new(item).expect("queue callback item is non-null"),
    );
}
pub unsafe fn cmdq_next(mut c: *mut client) -> u_int {
    let mut current_block: u64;
    let mut queue: *mut cmdq_list = cmdq_get(c);
    let mut name: *const ::core::ffi::c_char = cmdq_name(c);
    let mut item: *mut cmdq_item = ::core::ptr::null_mut::<cmdq_item>();
    let mut retval: cmd_retval = CMD_RETURN_NORMAL;
    let mut items: u_int = 0 as u_int;
    static mut number: u_int = 0;
    if (*queue).list.is_empty() {
        log_debug(
            b"%s %s: empty\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_next\0" as *const u8 as *const ::core::ffi::c_char,
            name,
        );
        return 0 as u_int;
    }
    if (*(*queue).first_ptr()).flags & CMDQ_WAITING != 0 {
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
        (*queue).item = (*queue).first_ptr();
        item = (*queue).item;
        if item.is_null() {
            current_block = 7056779235015430508;
            break;
        }
        log_debug(
            b"%s %s: %s (%d), flags %x\0" as *const u8 as *const ::core::ffi::c_char,
            b"cmdq_next\0" as *const u8 as *const ::core::ffi::c_char,
            name,
            ((*item).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
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
pub unsafe fn cmdq_running() -> *mut cmdq_item {
    let mut c: *mut client = ::core::ptr::null_mut::<client>();
    let mut queue: *mut cmdq_list = cmdq_get(c);
    if (*queue).item.is_null() {
        return ::core::ptr::null_mut::<cmdq_item>();
    }
    if (*(*queue).item).flags & CMDQ_WAITING != 0 {
        return ::core::ptr::null_mut::<cmdq_item>();
    }
    return (*queue).item;
}
pub unsafe fn cmdq_guard(
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
pub unsafe fn cmdq_print_data(mut item: *mut cmdq_item, mut evb: *mut evbuffer) {
    server_client_print((*item).client, 1 as ::core::ffi::c_int, evb);
}
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
            ((*c).name)
                .as_ref()
                .map_or(::core::ptr::null_mut(), |value| value.as_ptr().cast_mut()),
            msg.as_ptr(),
        );
        if !(*c).flags & CLIENT_UTF8 as uint64_t != 0 {
            msg = utf8_sanitize_cstring(msg.as_c_str());
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

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static DROPPED: AtomicUsize = AtomicUsize::new(0);

    struct Payload;

    impl Drop for Payload {
        fn drop(&mut self) {
            DROPPED.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn detached_unfired_callback_releases_its_payload() {
        let before = DROPPED.load(Ordering::SeqCst);
        unsafe {
            let payload = Box::new(Payload);
            let item = cmdq_get_callback_owned(c"cancel-payload".as_ptr(), None);
            cmdq_set_cancel_callback(&mut *item, Box::new(move || drop(payload)));
            cmdq_free_detached(item);
        }
        assert_eq!(DROPPED.load(Ordering::SeqCst), before + 1);
    }
}

impl Drop for cmdq_state {
    fn drop(&mut self) {
        unsafe { cmdq_destroy_state(self) }
    }
}
